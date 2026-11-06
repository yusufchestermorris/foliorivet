//! Bounded template operations shared by CLI and native bindings.
//! Independently authored FolioRivet extension; no report schemas or calculations.
use std::collections::BTreeMap;

use serde_json::{Value, json};
use wordcraft_doc::control::{ControlKind, ControlRange, walk_controls};
use wordcraft_doc::{Document, StoryRef};

use crate::{CmdError, CmdResult, CommandSpec, Session};

mod stage4;
mod stage5;

const MAX_JSON: usize = 1_048_576;
const MAX_VALUE: usize = 65_536;
const MAX_TARGETS: usize = 1024;

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
fn targets(doc: &Document) -> Result<BTreeMap<String, ControlRange>, CmdError> {
    if doc.has_unbalanced_controls() {
        return Err(error("resolve", "document", "unbalanced content controls; repair the template"));
    }
    let mut found = BTreeMap::new();
    for story in std::iter::once(StoryRef::Body).chain(doc.parts.keys().map(|k| StoryRef::Part(*k))) {
        let ranges = doc.control_ranges(story);
        if let Some(blocks) = doc.story(story) {
            let mut tags = Vec::new();
            walk_controls(blocks, &mut |c| {
                if !c.tag.is_empty() {
                    tags.push(c.tag.clone());
                }
            });
            for tag in tags {
                let matching: Vec<_> = ranges.iter().filter(|r| r.control.tag == tag).collect();
                if matching.len() != 1 {
                    return Err(error("resolve", &tag, "ambiguous target or unsupported table/row/cell wrapper"));
                }
                let Some(r) = matching.first() else { continue };
                if found.len() >= MAX_TARGETS {
                    return Err(error("resolve", &tag, "more than 1024 tagged targets"));
                }
                if found.insert(tag.clone(), (*r).clone()).is_some() {
                    return Err(error("resolve", &tag, "ambiguous target; tags must be unique across document stories"));
                }
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
    if stage5::needed(doc) {
        return stage5::inspect(doc);
    }
    if stage4::needed(doc) {
        return stage4::inspect(doc);
    }
    inspect_scalars(doc)
}

fn inspect_scalars(doc: &Document) -> CmdResult {
    let targets = targets(doc)?;
    Ok(json!({"scope": "document", "targets": targets.iter().map(|(tag, r)| {
        json!({"tag": tag, "story": r.start.story, "path": r.start.path, "kind": r.control.kind.name(),
            "supported": supported(doc, tag, r).is_ok(),
            "diagnostic": supported(doc, tag, r).err().map(|e| e.to_string())})
    }).collect::<Vec<_>>() }))
}

fn values(doc: &Document, context: &Value) -> Result<Vec<(ControlRange, String)>, CmdError> {
    if doc.settings.protection.is_some() || doc.settings.track_changes {
        return Err(error("preflight", "document", "protected documents and tracked-change editing are unsupported in Stage 3"));
    }
    if !context.is_object() || serde_json::to_vec(context).map_or(true, |v| v.len() > MAX_JSON) {
        return Err(error("preflight", "context", "expected a JSON object of at most 1 MiB"));
    }
    let targets = targets(doc)?;
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
            if !targets.contains_key(&tag) {
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
        let text = match value {
            Value::String(s) => s.clone(),
            Value::Number(_) | Value::Bool(_) => value.to_string(),
            _ => return Err(error("preflight", &tag, "expected a string, number or boolean; preformat unavailable values in Python")),
        };
        if text.len() > MAX_VALUE
            || text.chars().any(|c| (c.is_control() && c != '\n' && c != '\t') || matches!(c, '\u{fffc}' | '\u{fffe}' | '\u{ffff}'))
        {
            return Err(error("preflight", &tag, "value exceeds 64 KiB or contains unsupported control/XML characters; use LF for line breaks"));
        }
        if text.contains('\n') && matches!(r.control.kind, ControlKind::Text { multi_line: false }) {
            return Err(error("preflight", &tag, "enable multiline on this plain text control to insert line breaks"));
        }
        result.push((r, text));
    }
    Ok(result)
}

pub fn preflight(doc: &Document, context: &Value) -> CmdResult {
    preflight_with_assets(doc, context, "")
}

pub fn preflight_with_assets(doc: &Document, context: &Value, assets_dir: &str) -> CmdResult {
    if stage5::needed(doc) {
        return stage5::preflight(doc, context, assets_dir);
    }
    if stage4::needed(doc) {
        return stage4::preflight(doc, context, assets_dir);
    }
    if !assets_dir.is_empty() {
        return Err(error("preflight", assets_dir, "assets are unsupported without image: targets; pass an empty assets_dir"));
    }
    Ok(json!({"scope": "document", "scalar_count": values(doc, context)?.len(), "save_verified": false}))
}

/// Apply to a private copy: callers never observe partial edits on failure.
pub fn populate(doc: &mut Document, context: &Value) -> CmdResult {
    let mut edits = values(doc, context)?;
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
    if !std::path::Path::new(output_path).extension().is_some_and(|e| e.eq_ignore_ascii_case("docx")) {
        return Err(error("save", output_path, "output must have a .docx extension"));
    }
    crate::file_access::check_write(output_path).map_err(|e| error("save", output_path, &e))?;
    let mut session = Session::new(load(template_path)?);
    if stage5::needed(&session.doc) {
        stage5::populate(&mut session.doc, context, assets_dir)?;
    } else if stage4::needed(&session.doc) {
        stage4::populate(&mut session.doc, context, assets_dir)?;
    } else {
        preflight_with_assets(&session.doc, context, assets_dir)?;
        session.run("template.populateScalars", context).map_err(|e| error("populate", "document", &e.to_string()))?;
    }
    let bytes = crate::io::save_bytes("output.docx", &session.doc).map_err(|e| error("save", output_path, &e))?;
    crate::io::write_new_file(output_path, &bytes).map_err(|e| error("save", output_path, &e))?;
    Ok(output_path.into())
}

pub fn specs() -> Vec<CommandSpec> {
    #[allow(unused_mut)]
    let mut specs = vec![
        CommandSpec::new("template.populateBlocks", "Populate Template Blocks", "Automation", |s, v| {
            let context = v.get("context").ok_or_else(|| error("populate", "context", "required"))?;
            let assets = v.get("assets_dir").and_then(Value::as_str).ok_or_else(|| error("populate", "assets_dir", "expected a string"))?;
            let result = stage5::populate(&mut s.doc, context, assets)?;
            s.sel = crate::Selection::caret(s.doc.start_of(StoryRef::Body));
            Ok(result)
        })
        .params("{context, assets_dir}; bounded top-level repeat: and if: blocks"),
        CommandSpec::new("template.inspect", "Inspect Template Tags", "Automation", |s, _| inspect(&s.doc)).pure(),
        CommandSpec::new("template.preflight", "Validate Template", "Automation", |s, v| preflight(&s.doc, v)).pure(),
        CommandSpec::new("template.preflightWithAssets", "Validate Template Assets", "Automation", |s, v| {
            let context = v.get("context").ok_or_else(|| error("preflight", "context", "required"))?;
            let assets = v.get("assets_dir").and_then(Value::as_str).ok_or_else(|| error("preflight", "assets_dir", "expected a string"))?;
            preflight_with_assets(&s.doc, context, assets)
        })
        .pure(),
        CommandSpec::new("template.populateRowsAndImages", "Populate Template Rows and Images", "Automation", |s, v| {
            let context = v.get("context").ok_or_else(|| error("populate", "context", "required"))?;
            let assets = v.get("assets_dir").and_then(Value::as_str).ok_or_else(|| error("populate", "assets_dir", "expected a string"))?;
            let result = stage4::populate(&mut s.doc, context, assets)?;
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
            Ok(json!({"output_path": render_template(get("template_path")?, context, get("assets_dir")?, get("output_path")?)?}))
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
        let ranges = targets(&doc).unwrap();
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
            let r = targets(&doc).unwrap().remove("value:a").unwrap();
            assert_eq!(doc.control_text(&r), value.as_str().map(str::to_owned).unwrap_or_else(|| value.to_string()));
        }
        assert!(parse_context("{").is_err());
        assert!(preflight(&fixture(&["value:a"]), &json!([])).is_err());
        assert!(preflight(&fixture(&["value:a"]), &json!({"a": "x".repeat(MAX_VALUE + 1)})).is_err());
    }

    #[test]
    fn scalar_locks_placeholders_and_multiline_rules() {
        let mut doc = fixture(&["value:a"]);
        let r = targets(&doc).unwrap().remove("value:a").unwrap();
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
}
