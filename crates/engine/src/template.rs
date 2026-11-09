//! Bounded template operations shared by CLI and native bindings.
//! Independently authored FolioRivet extension; no report schemas or calculations.
use std::collections::BTreeSet;

use serde_json::{Value, json};
use wordcraft_doc::control::{ControlKind, ControlRange, walk_controls};
use wordcraft_doc::{Document, StoryRef};

use crate::{CmdError, CmdResult, CommandSpec, Session};

mod analysis;
mod stage4;
mod stage5;
pub use analysis::analyze_context;

const MAX_JSON: usize = 1_048_576;
const MAX_VALUE: usize = 65_536;
const MAX_TARGETS: usize = 1024;

/// Strict by default; fan-out applies only to scalar destinations in one effective scope.
#[derive(Clone, Copy, Debug, Default)]
pub struct TemplateOptions {
    pub scalar_fanout: bool,
}

#[derive(Default)]
struct Budget {
    items: usize,
    rows: usize,
    bytes: usize,
}

impl Budget {
    fn charge(&mut self, items: usize, rows: usize, bytes: usize, scope: &str) -> Result<(), CmdError> {
        self.items = self.items.saturating_add(items);
        self.rows = self.rows.saturating_add(rows);
        self.bytes = self.bytes.saturating_add(bytes);
        for (name, used, limit) in [("repeat item", self.items, 1000), ("row", self.rows, 1000), ("expansion byte", self.bytes, 16 * 1024 * 1024)] {
            if used > limit {
                return Err(error("preflight", scope, &format!("document {name} budget exceeded: used={used}, limit={limit}, scope={scope}")));
            }
        }
        Ok(())
    }
}

fn error(operation: &str, target: &str, message: &str) -> CmdError {
    CmdError::Failed(format!("{operation}: target={target:?}, scope=document: {message}"))
}

pub fn parse_context(text: &str) -> Result<Value, CmdError> {
    if text.len() > MAX_JSON {
        return Err(error("preflight", "context", "JSON exceeds 1 MiB"));
    }
    serde_json::from_str(text).map_err(|e| error("preflight", "context", &format!("invalid JSON: {e}")))
}

/// One resolver for inspection, preflight and population, including header/footer stories.
fn targets(doc: &Document, options: TemplateOptions) -> Result<Vec<(String, ControlRange)>, CmdError> {
    if doc.has_unbalanced_controls() {
        return Err(error("resolve", "document", "unbalanced content controls; repair the template"));
    }
    let mut found = Vec::new();
    let mut names = BTreeSet::new();
    for story in std::iter::once(StoryRef::Body).chain(doc.parts.keys().map(|k| StoryRef::Part(*k))) {
        let ranges = doc.control_ranges(story);
        if let Some(blocks) = doc.story(story) {
            let mut tags = Vec::new();
            walk_controls(blocks, &mut |c| {
                if !c.tag.is_empty() {
                    tags.push(c.tag.clone());
                }
            });
            let matching: Vec<_> = ranges.into_iter().filter(|r| !r.control.tag.is_empty()).collect();
            if matching.len() != tags.len() {
                return Err(error("resolve", "document", "unsupported table/row/cell wrapper"));
            }
            for r in matching {
                let tag = r.control.tag.clone();
                if !(names.insert(tag.clone()) || options.scalar_fanout && tag.starts_with("value:")) {
                    return Err(error("resolve", &tag, "ambiguous target; tags must be unique across document stories"));
                }
                if found.len() >= MAX_TARGETS {
                    return Err(error("resolve", &tag, "more than 1024 tagged target occurrences"));
                }
                found.push((tag, r));
            }
        }
    }
    Ok(found)
}

fn supported(doc: &Document, tag: &str, r: &ControlRange) -> Result<(), CmdError> {
    let Some(key) = tag.strip_prefix("value:") else {
        return Err(error("preflight", tag, "only value: scalar tags are supported in Stage 3"));
    };
    if key.is_empty() || key.len() > 256 || key.split('.').any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')) {
        return Err(error("preflight", tag, "use a dotted ASCII field path, for example value:metadata.model_id"));
    }
    if r.depth != 0 || r.start.path != r.end.path {
        return Err(error("preflight", tag, "nested or multi-paragraph scalar targets are unsupported"));
    }
    if !matches!(r.control.kind, ControlKind::RichText | ControlKind::Text { .. }) || !r.control.extra.is_empty() {
        return Err(error("preflight", tag, "requires an ordinary rich/plain text control without data binding or unknown properties"));
    }
    if !r.control.lock.can_edit() {
        return Err(error("preflight", tag, "unlock the control contents before population"));
    }
    let a = r.content_start();
    let para = doc.para_at(&a).ok_or_else(|| error("resolve", tag, "target paragraph is missing"))?;
    if para.text.get(a.off..r.end.off).is_none_or(|t| t.contains(wordcraft_doc::para::OBJ)) {
        return Err(error("preflight", tag, "scalar content must contain text only, without fields, pictures or nested controls"));
    }
    Ok(())
}

pub fn inspect(doc: &Document) -> CmdResult {
    inspect_with_options(doc, TemplateOptions::default())
}

pub fn inspect_with_options(doc: &Document, options: TemplateOptions) -> CmdResult {
    if stage5::needed(doc) {
        return stage5::inspect_with_options(doc, options);
    }
    if stage4::needed(doc) {
        return stage4::inspect_with_options(doc, options);
    }
    inspect_scalars(doc, options)
}

fn inspect_scalars(doc: &Document, options: TemplateOptions) -> CmdResult {
    let targets = targets(doc, options)?;
    Ok(json!({"scope": "document", "targets": targets.iter().map(|(tag, r)| {
        json!({"tag": tag, "story": r.start.story, "path": r.start.path, "offset": r.start.off, "scope": "document", "kind": r.control.kind.name(),
            "supported": supported(doc, tag, r).is_ok(),
            "diagnostic": supported(doc, tag, r).err().map(|e| e.to_string())})
    }).collect::<Vec<_>>() }))
}

fn values(doc: &Document, context: &Value, options: TemplateOptions) -> Result<Vec<(ControlRange, String)>, CmdError> {
    if doc.settings.protection.is_some() || doc.settings.track_changes {
        return Err(error("preflight", "document", "protected documents and tracked-change editing are unsupported in Stage 3"));
    }
    if !context.is_object() || serde_json::to_vec(context).map_or(true, |v| v.len() > MAX_JSON) {
        return Err(error("preflight", "context", "expected a JSON object of at most 1 MiB"));
    }
    let targets = targets(doc, options)?;
    if targets.is_empty() {
        return Err(error("preflight", "document", "no tagged scalar targets found"));
    }
    for (tag, range) in &targets {
        supported(doc, tag, range)?;
    }
    // Every supplied leaf declares a required target. This also catches a deleted
    // control or a mistyped context field instead of silently ignoring the value.
    let mut fields = vec![(String::new(), context, 0usize)];
    while let Some((path, value, depth)) = fields.pop() {
        if depth > 32 {
            return Err(error("preflight", &path, "context nesting exceeds 32 levels"));
        }
        if let Some(object) = value.as_object() {
            for (key, child) in object {
                let next = if path.is_empty() { key.clone() } else { format!("{path}.{key}") };
                if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    return Err(error("preflight", &next, "context field names must use ASCII letters, digits or underscores"));
                }
                fields.push((next, child, depth.saturating_add(1)));
            }
        } else {
            let tag = format!("value:{path}");
            if !targets.iter().any(|(name, _)| name == &tag) {
                return Err(error("preflight", &tag, "missing required target; add this tag to the template or remove the unused input field"));
            }
        }
    }
    let mut result = Vec::new();
    for (tag, r) in targets {
        supported(doc, &tag, &r)?;
        let key = tag.strip_prefix("value:").ok_or_else(|| error("resolve", &tag, "expected value: prefix"))?;
        let mut value = context;
        for part in key.split('.') {
            value = value
                .as_object()
                .and_then(|v| v.get(part))
                .ok_or_else(|| error("preflight", &tag, "missing required value; supply this dotted field in context"))?;
        }
        let text = scalar_text(value, &r, &tag)?;
        result.push((r, text));
    }
    Ok(result)
}

fn scalar_text(value: &Value, r: &ControlRange, tag: &str) -> Result<String, CmdError> {
    let text = match value {
        Value::String(s) => s.clone(),
        Value::Number(_) | Value::Bool(_) => value.to_string(),
        _ => return Err(error("preflight", tag, "expected a string, number or boolean; preformat unavailable values in Python")),
    };
    if text.len() > MAX_VALUE || text.chars().any(|c| (c.is_control() && c != '\n' && c != '\t') || matches!(c, '\u{fffc}' | '\u{fffe}' | '\u{ffff}'))
    {
        return Err(error("preflight", tag, "value exceeds 64 KiB or contains unsupported control/XML characters; use LF for line breaks"));
    }
    if text.contains('\n') && matches!(r.control.kind, ControlKind::Text { multi_line: false }) {
        return Err(error("preflight", tag, "enable multiline on this plain text control to insert line breaks"));
    }
    Ok(text)
}

pub fn preflight(doc: &Document, context: &Value) -> CmdResult {
    preflight_with_assets(doc, context, "")
}

pub fn preflight_with_assets(doc: &Document, context: &Value, assets_dir: &str) -> CmdResult {
    preflight_with_options(doc, context, assets_dir, TemplateOptions::default())
}

pub fn preflight_with_options(doc: &Document, context: &Value, assets_dir: &str, options: TemplateOptions) -> CmdResult {
    if stage5::needed(doc) {
        return stage5::preflight_with_options(doc, context, assets_dir, options);
    }
    if stage4::needed(doc) {
        return stage4::preflight_with_options(doc, context, assets_dir, options);
    }
    if !assets_dir.is_empty() {
        return Err(error("preflight", assets_dir, "assets are unsupported without image: targets; pass an empty assets_dir"));
    }
    Ok(json!({"scope": "document", "scalar_count": values(doc, context, options)?.len(), "save_verified": false}))
}

/// Apply to a private copy: callers never observe partial edits on failure.
pub fn populate(doc: &mut Document, context: &Value) -> CmdResult {
    populate_with_options(doc, context, TemplateOptions::default())
}

pub fn populate_with_options(doc: &mut Document, context: &Value, options: TemplateOptions) -> CmdResult {
    let mut edits = values(doc, context, options)?;
    edits.sort_by(|(a, _), (b, _)| b.start.cmp(&a.start));
    let mut copy = doc.clone();
    for (r, text) in &edits {
        let a = r.content_start();
        let para = copy.para_at(&a).ok_or_else(|| error("populate", &r.control.tag, "paragraph missing"))?;
        let mut props = para.props_of_char(if r.is_empty() { r.start.off } else { a.off }).clone();
        if r.control.showing_placeholder {
            props = r.control.run_props.clone().unwrap_or(props);
            if props.style.as_deref().is_some_and(|id| {
                id == crate::cmd::controls::PLACEHOLDER_STYLE || copy.styles.get(id).is_some_and(|s| s.name.eq_ignore_ascii_case("Placeholder Text"))
            }) {
                props.style = None;
            }
        }
        copy.delete_range(&a, &r.end).map_err(|e| error("populate", &r.control.tag, &e.to_string()))?;
        copy.insert_text(&a, text, &props).map_err(|e| error("populate", &r.control.tag, &e.to_string()))?;
        if let Some(control) = copy.control_mut(&r.start) {
            control.showing_placeholder = false;
        }
    }
    *doc = copy;
    Ok(json!({"scalar_count": edits.len()}))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load(path: &str) -> Result<Document, CmdError> {
    if !std::path::Path::new(path).extension().is_some_and(|e| e.eq_ignore_ascii_case("docx")) {
        return Err(error("load", path, "Stage 3 accepts .docx templates only"));
    }
    crate::io::open_path(std::path::Path::new(path)).map_err(|e| error("load", path, &e))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_template(template_path: &str, context: &Value, assets_dir: &str, output_path: &str) -> Result<String, CmdError> {
    render_template_with_options(template_path, context, assets_dir, output_path, TemplateOptions::default())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_template_with_options(
    template_path: &str,
    context: &Value,
    assets_dir: &str,
    output_path: &str,
    options: TemplateOptions,
) -> Result<String, CmdError> {
    if !std::path::Path::new(output_path).extension().is_some_and(|e| e.eq_ignore_ascii_case("docx")) {
        return Err(error("save", output_path, "output must have a .docx extension"));
    }
    crate::file_access::check_write(output_path).map_err(|e| error("save", output_path, &e))?;
    let mut session = Session::new(load(template_path)?);
    if stage5::needed(&session.doc) {
        stage5::populate_with_options(&mut session.doc, context, assets_dir, options)?;
    } else if stage4::needed(&session.doc) {
        stage4::populate_with_options(&mut session.doc, context, assets_dir, options)?;
    } else {
        preflight_with_options(&session.doc, context, assets_dir, options)?;
        populate_with_options(&mut session.doc, context, options)?;
    }
    let bytes = crate::io::save_bytes("output.docx", &session.doc).map_err(|e| error("save", output_path, &e))?;
    crate::io::write_new_file(output_path, &bytes).map_err(|e| error("save", output_path, &e))?;
    Ok(output_path.into())
}

fn command_options(v: &Value) -> Result<TemplateOptions, CmdError> {
    match v.get("scalar_fanout") {
        None => Ok(TemplateOptions::default()),
        Some(Value::Bool(value)) => Ok(TemplateOptions { scalar_fanout: *value }),
        _ => Err(error("options", "scalar_fanout", "expected a boolean")),
    }
}

pub fn specs() -> Vec<CommandSpec> {
    #[allow(unused_mut)]
    let mut specs = vec![
        CommandSpec::new("template.populateBlocks", "Populate Template Blocks", "Automation", |s, v| {
            let context = v.get("context").ok_or_else(|| error("populate", "context", "required"))?;
            let assets = v.get("assets_dir").and_then(Value::as_str).ok_or_else(|| error("populate", "assets_dir", "expected a string"))?;
            let result = stage5::populate_with_options(&mut s.doc, context, assets, command_options(v)?)?;
            s.sel = crate::Selection::caret(s.doc.start_of(StoryRef::Body));
            Ok(result)
        })
        .params("{context, assets_dir}; bounded top-level repeat: and if: blocks"),
        CommandSpec::new("template.inspect", "Inspect Template Tags", "Automation", |s, v| inspect_with_options(&s.doc, command_options(v)?)).pure(),
        CommandSpec::new("template.analyzeContext", "Analyze Template Context", "Automation", |s, v| {
            let context = v.get("context").ok_or_else(|| error("analyze", "context", "required"))?;
            analyze_context(&s.doc, context, command_options(v)?)
        })
        .params("{context, scalar_fanout?: false}; no decoding or output writing")
        .pure(),
        CommandSpec::new("template.preflight", "Validate Template", "Automation", |s, v| preflight(&s.doc, v)).pure(),
        CommandSpec::new("template.preflightWithAssets", "Validate Template Assets", "Automation", |s, v| {
            let context = v.get("context").ok_or_else(|| error("preflight", "context", "required"))?;
            let assets = v.get("assets_dir").and_then(Value::as_str).ok_or_else(|| error("preflight", "assets_dir", "expected a string"))?;
            preflight_with_options(&s.doc, context, assets, command_options(v)?)
        })
        .pure(),
        CommandSpec::new("template.populateRowsAndImages", "Populate Template Rows and Images", "Automation", |s, v| {
            let context = v.get("context").ok_or_else(|| error("populate", "context", "required"))?;
            let assets = v.get("assets_dir").and_then(Value::as_str).ok_or_else(|| error("populate", "assets_dir", "expected a string"))?;
            let result = stage4::populate_with_options(&mut s.doc, context, assets, command_options(v)?)?;
            s.sel = crate::Selection::caret(s.doc.start_of(StoryRef::Body));
            Ok(result)
        })
        .params("{context, assets_dir}; bounded single-row prototypes and inline picture slots"),
        CommandSpec::new("template.populateScalars", "Populate Scalar Template", "Automation", |s, v| populate(&mut s.doc, v))
            .params("context object; dotted fields map to unique value: tags"),
    ];
    #[cfg(not(target_arch = "wasm32"))]
    specs.push(
        CommandSpec::new("template.render", "Render Template to Staging", "Automation", |_, v| {
            let get = |key| v.get(key).and_then(Value::as_str).ok_or_else(|| error("render", key, "expected a string"));
            let context = v.get("context").ok_or_else(|| error("render", "context", "required"))?;
            Ok(json!({"output_path": render_template_with_options(get("template_path")?, context, get("assets_dir")?, get("output_path")?, command_options(v)?)?}))
        })
        .params("{template_path, context, assets_dir, output_path: new .docx staging path}")
        .pure(),
    );
    specs
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordcraft_doc::control::ContentControl;
    use wordcraft_doc::props::CharProps;
    use wordcraft_doc::{InlineObject, Pos};

    fn fixture(tags: &[&str]) -> Document {
        let mut doc = Document::new();
        let mut at = Pos::body(0, 0);
        for tag in tags {
            at = doc.insert_text(&at, "before ", &CharProps::default()).unwrap();
            let control = ContentControl { tag: (*tag).into(), ..Default::default() };
            at = doc.insert_object(&at, InlineObject::ControlStart { control: Box::new(control) }, &CharProps::default()).unwrap();
            at = doc.insert_text(&at, "placeholder", &CharProps { bold: Some(true), ..Default::default() }).unwrap();
            at = doc.insert_object(&at, InlineObject::ControlEnd, &CharProps::default()).unwrap();
            at = doc.insert_text(&at, " after ", &CharProps::default()).unwrap();
        }
        doc
    }

    #[test]
    fn scalar_roundtrip_keeps_neighbors_formatting_and_controls() {
        let mut doc = fixture(&["value:metadata.model_id", "value:count"]);
        let context = json!({"metadata": {"model_id": "模型 & <é>\nsecond\tline"}, "count": 17});
        populate(&mut doc, &context).unwrap();
        let bytes = crate::io::save_bytes("scalar.docx", &doc).unwrap();
        let doc = crate::io::open_bytes("scalar.docx", &bytes).unwrap();
        let ranges = targets(&doc, TemplateOptions::default()).unwrap().into_iter().collect::<std::collections::BTreeMap<_, _>>();
        let r = ranges.get("value:metadata.model_id").unwrap();
        assert_eq!(doc.control_text(r), "模型 & <é>\nsecond\tline");
        assert_eq!(doc.para_at(&r.content_start()).unwrap().props_of_char(r.content_start().off).bold, Some(true));
        assert_eq!(doc.control_text(ranges.get("value:count").unwrap()), "17");
        assert!(doc.para_at(&r.start).unwrap().text.starts_with("before "));
        assert!(doc.para_at(&r.start).unwrap().text.ends_with(" after "));
        assert!(!doc.has_unbalanced_controls());
    }

    #[test]
    fn scalar_failures_are_atomic_and_actionable() {
        for (tags, context, message) in [
            (vec!["value:a", "value:a"], json!({"a": "x"}), "ambiguous"),
            (vec!["value:a", "value:b"], json!({"a": "x"}), "missing required"),
            (vec!["value:a"], json!({"a": "x", "deleted": "required"}), "missing required target"),
            (vec!["value:a"], json!({"a": null}), "expected a string"),
            (vec!["value:a"], json!({"a": []}), "expected a string"),
            (vec!["value:a"], json!({"a": "\u{0000}"}), "unsupported control"),
            (vec!["rows:a"], json!({"a": []}), "only value:"),
            (vec!["value:a..b"], json!({}), "dotted ASCII"),
        ] {
            let mut doc = fixture(&tags);
            let before = serde_json::to_value(&doc).unwrap();
            let error = populate(&mut doc, &context).unwrap_err().to_string();
            assert!(error.contains(message), "{error}");
            assert!(error.contains("target=") && error.contains("scope=document"));
            assert_eq!(serde_json::to_value(&doc).unwrap(), before);
        }
    }

    #[test]
    fn scalar_empty_long_and_boolean_values() {
        for value in [json!(""), json!("é".repeat(10_000)), json!(false)] {
            let mut doc = fixture(&["value:a"]);
            populate(&mut doc, &json!({"a": value})).unwrap();
            let r = targets(&doc, TemplateOptions::default())
                .unwrap()
                .into_iter()
                .collect::<std::collections::BTreeMap<_, _>>()
                .remove("value:a")
                .unwrap();
            assert_eq!(doc.control_text(&r), value.as_str().map(str::to_owned).unwrap_or_else(|| value.to_string()));
        }
        assert!(parse_context("{").is_err());
        assert!(preflight(&fixture(&["value:a"]), &json!([])).is_err());
        assert!(preflight(&fixture(&["value:a"]), &json!({"a": "x".repeat(MAX_VALUE + 1)})).is_err());
    }

    #[test]
    fn scalar_locks_placeholders_and_multiline_rules() {
        let mut doc = fixture(&["value:a"]);
        let r =
            targets(&doc, TemplateOptions::default()).unwrap().into_iter().collect::<std::collections::BTreeMap<_, _>>().remove("value:a").unwrap();
        doc.control_mut(&r.start).unwrap().lock = wordcraft_doc::control::ControlLock::ContentLocked;
        assert!(preflight(&doc, &json!({"a": "x"})).unwrap_err().to_string().contains("unlock"));
        let c = doc.control_mut(&r.start).unwrap();
        c.lock = wordcraft_doc::control::ControlLock::Unlocked;
        c.kind = ControlKind::Text { multi_line: false };
        assert!(preflight(&doc, &json!({"a": "x\ny"})).unwrap_err().to_string().contains("multiline"));
        let c = doc.control_mut(&r.start).unwrap();
        c.kind = ControlKind::Text { multi_line: true };
        c.showing_placeholder = true;
        c.run_props = Some(CharProps { italic: Some(true), ..Default::default() });
        populate(&mut doc, &json!({"a": "x\ny"})).unwrap();
        assert!(!doc.control_at(&r.content_start()).unwrap().control.showing_placeholder);
        assert_eq!(doc.para_at(&r.start).unwrap().props_of_char(r.content_start().off).italic, Some(true));
    }
    #[test]
    fn fanout_preserves_each_destination_and_is_atomic() {
        let options = TemplateOptions { scalar_fanout: true };
        let mut doc = fixture(&["value:a", "value:a"]);
        assert!(preflight(&doc, &json!({"a":"x"})).is_err());
        let ranges = targets(&doc, options).unwrap();
        let last = &ranges[1].1;
        doc.control_mut(&last.start).unwrap().kind = ControlKind::Text { multi_line: false };
        doc.control_mut(&last.start).unwrap().showing_placeholder = true;
        doc.control_mut(&last.start).unwrap().run_props = Some(CharProps { italic: Some(true), bold: Some(false), ..Default::default() });
        let before = serde_json::to_value(&doc).unwrap();
        assert!(populate_with_options(&mut doc, &json!({"a":"x\ny"}), options).is_err());
        assert_eq!(serde_json::to_value(&doc).unwrap(), before);
        assert_eq!(inspect_with_options(&doc, options).unwrap()["targets"].as_array().unwrap().len(), 2);
        populate_with_options(&mut doc, &json!({"a":"filled"}), options).unwrap();
        let back = wordcraft_docx::read(&wordcraft_docx::write(&doc).unwrap()).unwrap();
        let ranges = targets(&back, options).unwrap();
        for (_, range) in &ranges {
            assert_eq!(back.control_text(range), "filled");
        }
        assert_eq!(back.para_at(&ranges[0].1.start).unwrap().props_of_char(ranges[0].1.content_start().off).bold, Some(true));
        assert_eq!(back.para_at(&ranges[1].1.start).unwrap().props_of_char(ranges[1].1.content_start().off).italic, Some(true));
    }

    #[test]
    fn analysis_returns_findings_and_validates_structure() {
        let doc = fixture(&["value:a", "value:b"]);
        let result = analyze_context(&doc, &json!({"a":null,"extra":1}), TemplateOptions::default()).unwrap();
        assert_eq!(result["active_required_paths"], json!(["a", "b"]));
        assert_eq!(result["missing_paths"], json!(["b"]));
        assert_eq!(result["unused_supplied_paths"], json!(["extra"]));
        assert_eq!(result["diagnostics"].as_array().unwrap().len(), 2);
        assert!(analyze_context(&fixture(&["value:a..b"]), &json!({}), TemplateOptions::default()).is_err());
    }

    #[test]
    fn document_budget_exact_boundaries_and_independent_counters() {
        let mut budget = Budget::default();
        budget.charge(600, 600, 8 * 1024 * 1024, "first").unwrap();
        budget.charge(400, 400, 8 * 1024 * 1024, "second").unwrap();
        for (items, rows, bytes, name) in [(1, 0, 0, "repeat item"), (0, 1, 0, "row"), (0, 0, 1, "expansion byte")] {
            let mut budget = Budget { items: 1000, rows: 1000, bytes: 16 * 1024 * 1024 };
            let message = budget.charge(items, rows, bytes, "outer / child[1]").unwrap_err().to_string();
            assert!(message.contains(name) && message.contains("used=") && message.contains("limit=") && message.contains("child[1]"));
        }
    }

    #[test]
    fn fanout_crosses_paragraphs_and_stories_but_counts_occurrences() {
        let options = TemplateOptions { scalar_fanout: true };
        let mut doc = fixture(&["value:a"]);
        doc.body.extend(fixture(&["value:a"]).body);
        for (id, kind) in [(1, wordcraft_doc::PartKind::Header), (2, wordcraft_doc::PartKind::Footer)] {
            doc.parts.insert(id, wordcraft_doc::Part { kind, blocks: fixture(&["value:a"]).body, ..Default::default() });
        }
        assert!(inspect(&doc).is_err());
        assert_eq!(inspect_with_options(&doc, options).unwrap()["targets"].as_array().unwrap().len(), 4);
        assert_eq!(populate_with_options(&mut doc, &json!({"a":"everywhere"}), options).unwrap()["scalar_count"], 4);
        for (_, r) in targets(&doc, options).unwrap() {
            assert_eq!(doc.control_text(&r), "everywhere");
        }
        let mut oversized = fixture(&["value:a"]);
        oversized.body = vec![oversized.body[0].clone(); MAX_TARGETS + 1];
        assert!(inspect_with_options(&oversized, options).unwrap_err().to_string().contains("1024"));
    }
    #[test]
    fn legacy_context_fields_are_not_interpreted_as_options() {
        let mut session = Session::new(fixture(&["value:context.name", "value:scalar_fanout"]));
        let context = json!({"context":{"name":"Report"},"scalar_fanout":true});
        session.run("template.preflight", &context).unwrap();
        session.run("template.populateScalars", &context).unwrap();
    }
    #[test]
    fn analysis_invalid_keys_do_not_hide_other_unused_inputs() {
        let result = analyze_context(
            &fixture(&["value:name"]),
            &json!({"!bad":1,"group":{"!bad":2,"unused":3},"name":"Example","z_unused":4}),
            TemplateOptions::default(),
        )
        .unwrap();
        assert_eq!(result["unused_supplied_paths"], json!(["!bad", "group.!bad", "group.unused", "z_unused"]));
        assert_eq!(result["diagnostics"].as_array().unwrap().len(), 2);
        assert_eq!(result["diagnostics"][1]["target"], "group.!bad");
    }
}
