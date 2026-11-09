//! Independently authored bounded table-row and existing-picture-slot operations.
//! Model edits only; repeated tags are consumed so no duplicate control IDs survive.
use std::collections::BTreeSet;
use std::sync::Arc;

use super::{CmdError, CmdResult, ControlKind, ControlRange, Document, Value, error, json};
use wordcraft_doc::{Block, ContentControl, InlineObject, Path, Row, StoryRef, Table};

pub(super) struct RowSlot {
    pub(super) table: usize,
    pub(super) row: usize,
    pub(super) tag: String,
    pub(super) prototype: Row,
}

pub(super) struct Resolved {
    pub(super) masked: Document,
    pub(super) rows: Vec<RowSlot>,
    pub(super) images: Vec<ControlRange>,
}

pub(super) fn needed(doc: &Document) -> bool {
    let mut found = false;
    for blocks in std::iter::once(&doc.body).chain(doc.parts.values().map(|p| &p.blocks)) {
        wordcraft_doc::control::walk_controls(blocks, &mut |c| {
            found |= c.tag.starts_with("rows:") || c.tag.starts_with("image:");
        });
    }
    found
}

fn field<'a>(tag: &'a str, prefix: &str) -> Result<&'a str, CmdError> {
    let key = tag.strip_prefix(prefix).unwrap_or("");
    if key.is_empty() || key.len() > 256 || key.split('.').any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')) {
        return Err(error("resolve", tag, "use a dotted ASCII field path"));
    }
    Ok(key)
}

fn ordinary(control: &ContentControl, tag: &str) -> Result<(), CmdError> {
    if !control.lock.can_edit() || !control.extra.is_empty() {
        return Err(error("resolve", tag, "unlock contents and remove data bindings/unknown control properties"));
    }
    Ok(())
}

pub(super) fn row_document(doc: &Document, row: &Row) -> Document {
    let mut mini = doc.clone();
    mini.parts.clear();
    let mut row = row.clone();
    row.controls = Default::default();
    mini.body = vec![Arc::new(Block::Table(Table { rows: vec![row], ..Default::default() }))];
    mini
}

fn check_row(doc: &Document, row: &Row, tag: &str, options: super::TemplateOptions) -> Result<(), CmdError> {
    if row.props.header || row.props.fmt_change.is_some() || row.cells.is_empty() {
        return Err(error("resolve", tag, "prototype must be a non-header data row without tracked formatting"));
    }
    for cell in &row.cells {
        if !cell.controls.is_empty()
            || cell.props.vmerge != wordcraft_doc::props::VMerge::None
            || cell.props.fmt_change.is_some()
            || cell.props.picture.is_some()
        {
            return Err(error(
                "resolve",
                tag,
                "cell wrappers, vertical merges, tracked formatting and cell background pictures are unsupported in repeated rows",
            ));
        }
        for block in &cell.blocks {
            let Block::Para(p) = block.as_ref() else {
                return Err(error("resolve", tag, "nested tables are unsupported in repeated rows"));
            };
            let resolved = doc.styles.resolve_para(&p.props);
            if p.section.is_some() || resolved.numbering.is_some() || p.props.fmt_change.is_some() || p.props.num_change.is_some() {
                return Err(error("resolve", tag, "sections, numbered paragraphs and tracked formatting are unsupported in repeated rows"));
            }
            if std::iter::once(&p.mark)
                .chain(p.runs.iter().map(|r| &r.props))
                .any(|r| r.ins.is_some() || r.del.is_some() || r.fmt_change.is_some() || r.link.is_some())
            {
                return Err(error("resolve", tag, "tracked changes and hyperlinks are unsupported in repeated rows"));
            }
            for object in &p.objects {
                match object {
                    InlineObject::ControlStart { control } if control.tag.starts_with("value:item.") => {}
                    InlineObject::ControlEnd => {}
                    _ => {
                        return Err(error(
                            "resolve",
                            tag,
                            "repeated rows support text and value:item.* controls only; remove bookmarks, fields, media and other objects",
                        ));
                    }
                }
            }
        }
    }
    let mini = row_document(doc, row);
    let targets = super::targets(&mini, options)?;
    if targets.is_empty() {
        return Err(error("resolve", tag, "unsupported prototype without item targets; add at least one value:item.* control"));
    }
    for (key, r) in targets {
        if !r.control.lock.can_delete() {
            return Err(error("resolve", tag, "unlock item controls so their template wrappers can be consumed"));
        }
        super::supported(&mini, &key, &r)?;
    }
    Ok(())
}

pub(super) fn resolve(doc: &Document, scoped: bool, options: super::TemplateOptions) -> Result<Resolved, CmdError> {
    if doc.has_unbalanced_controls() || doc.settings.protection.is_some() || doc.settings.track_changes {
        return Err(error("resolve", "document", "unbalanced controls, protection or tracked-change editing are unsupported"));
    }
    let mut masked = doc.clone();
    let mut rows = Vec::new();
    let mut names = BTreeSet::new();
    for (ti, block) in doc.body.iter().enumerate() {
        let Block::Table(table) = block.as_ref() else { continue };
        for (ri, row) in table.rows.iter().enumerate() {
            let Some(control) = row.controls.open.iter().find(|c| c.tag.starts_with("rows:")) else { continue };
            field(&control.tag, "rows:")?;
            ordinary(control, &control.tag)?;
            if !control.lock.can_delete() {
                return Err(error("resolve", &control.tag, "unlock the prototype row wrapper before expansion"));
            }
            if !names.insert(control.tag.clone()) {
                return Err(error("resolve", &control.tag, "ambiguous row target"));
            }
            if row.controls.open.len() != 1 || row.controls.close != 1 || !table.controls.is_empty() || !matches!(control.kind, ControlKind::RichText)
            {
                return Err(error(
                    "resolve",
                    &control.tag,
                    "requires one complete ordinary rich-text row wrapper; multi-row/nested wrappers are unsupported",
                ));
            }
            // Removing a prototype cannot leave an empty table or a dangling vertical merge.
            if !table.rows.iter().any(|r| r.controls.is_empty())
                || table.rows.iter().any(|r| r.cells.iter().any(|c| c.props.vmerge != wordcraft_doc::props::VMerge::None))
            {
                return Err(error("resolve", &control.tag, "keep a separate static/header row; vertical merges in the target table are unsupported"));
            }
            check_row(doc, row, &control.tag, options)?;
            let table = masked.table_mut(StoryRef::Body, &Path::top(ti)).map_err(|e| error("resolve", &control.tag, &e.to_string()))?;
            let Some(mask) = table.rows.get_mut(ri) else { return Err(error("resolve", &control.tag, "prototype missing")) };
            mask.controls = Default::default();
            for c in &mut mask.cells {
                c.blocks = vec![wordcraft_doc::para_block(wordcraft_doc::Paragraph::new())];
            }
            rows.push(RowSlot { table: ti, row: ri, tag: control.tag.clone(), prototype: row.clone() });
        }
    }
    // Remaining row wrappers (headers, nested tables, multi-row closes etc.) fail
    // through the same scalar resolver instead of being silently ignored.
    let mut images = Vec::new();
    for story in std::iter::once(StoryRef::Body).chain(doc.parts.keys().map(|id| StoryRef::Part(*id))) {
        for r in masked.control_ranges(story) {
            if !r.control.tag.starts_with("image:") {
                continue;
            }
            field(&r.control.tag, "image:")?;
            ordinary(&r.control, &r.control.tag)?;
            if !names.insert(r.control.tag.clone()) {
                return Err(error("resolve", &r.control.tag, "ambiguous image target"));
            }
            if r.depth != 0 || r.start.path != r.end.path || r.control.kind != ControlKind::Picture {
                return Err(error("resolve", &r.control.tag, "requires an unnested inline picture content control"));
            }
            let p = masked.para_at(&r.start).ok_or_else(|| error("resolve", &r.control.tag, "paragraph missing"))?;
            let content = p.text.get(r.content_start().off..r.end.off);
            let object = p.objects_with_offsets().find(|(off, _)| *off == r.content_start().off).map(|(_, o)| o);
            if content != Some("\u{fffc}")
                || !matches!(object, Some(InlineObject::Image { w, h, float, crop, ole: None, .. })
                if w.is_finite() && h.is_finite() && *w > 0.0 && *h > 0.0 && *w <= 1584.0 && *h <= 1584.0
                && float.wrap == wordcraft_doc::para::Wrap::Inline && *crop == [0.0; 4])
            {
                return Err(error(
                    "resolve",
                    &r.control.tag,
                    "slot needs one uncropped inline image with finite dimensions up to 22 inches and no OLE object",
                ));
            }
            let control = masked.control_mut(&r.start).ok_or_else(|| error("resolve", &r.control.tag, "control missing"))?;
            control.tag.clear();
            images.push(r);
        }
    }
    let scalar_targets = super::targets(&masked, options)?;
    if names.len().saturating_add(scalar_targets.len()) > super::MAX_TARGETS {
        return Err(error("resolve", "document", "more than 1024 document-level tagged targets"));
    }
    if !scoped && scalar_targets.iter().any(|(tag, _)| tag.starts_with("value:item.")) {
        return Err(error("resolve", "value:item.*", "item fields require a supported rows: prototype scope"));
    }
    Ok(Resolved { masked, rows, images })
}

pub(super) fn inspect_with_options(doc: &Document, options: super::TemplateOptions) -> CmdResult {
    inspect_scope(doc, false, options)
}

pub(super) fn inspect_scope(doc: &Document, scoped: bool, options: super::TemplateOptions) -> CmdResult {
    let resolved = resolve(doc, scoped, options)?;
    let mut result = super::inspect_scalars(&resolved.masked, options)?;
    let entries = result.get_mut("targets").and_then(Value::as_array_mut).ok_or_else(|| error("inspect", "document", "target list missing"))?;
    for slot in resolved.rows {
        entries.push(
            json!({"tag": slot.tag, "scope": "document", "story": "body", "table": slot.table, "row": slot.row, "kind": "row", "supported": true}),
        );
        for (tag, r) in super::targets(&row_document(doc, &slot.prototype), options)? {
            entries.push(json!({"tag": tag, "scope": slot.tag, "story": r.start.story, "path": r.start.path, "offset": r.start.off, "kind": r.control.kind.name(), "supported": true}));
        }
    }
    for r in resolved.images {
        entries.push(
            json!({"tag": r.control.tag, "scope": "document", "story": r.start.story, "path": r.start.path, "kind": "picture", "supported": true}),
        );
    }
    if entries.len() > super::MAX_TARGETS {
        return Err(error("inspect", "document", "more than 1024 tagged target occurrences"));
    }
    Ok(result)
}

pub(super) fn take(context: &mut Value, path: &str, tag: &str) -> Result<Value, CmdError> {
    let mut here = context;
    let mut parts = path.split('.').peekable();
    while let Some(part) = parts.next() {
        let obj = here.as_object_mut().ok_or_else(|| error("preflight", tag, "expected an object at the target's parent path"))?;
        if parts.peek().is_none() {
            return obj.remove(part).ok_or_else(|| error("preflight", tag, "missing required value"));
        }
        here = obj.get_mut(part).ok_or_else(|| error("preflight", tag, "missing required value"))?;
    }
    Err(error("preflight", tag, "empty field path"))
}

fn consume_controls(row: &mut Row) -> Result<(), CmdError> {
    row.controls = Default::default();
    for cell in &mut row.cells {
        for block in &mut cell.blocks {
            if let Block::Para(p) = Arc::make_mut(block) {
                let offsets: Vec<_> = p.objects_with_offsets().map(|(off, _)| off).collect();
                for off in offsets.into_iter().rev() {
                    p.delete(off, off.saturating_add(wordcraft_doc::para::OBJ.len_utf8())).map_err(|e| error("populate", "row", &e.to_string()))?;
                }
            }
        }
    }
    Ok(())
}

pub(super) fn scalar_finish(doc: &mut Document, context: &Value, options: super::TemplateOptions) -> CmdResult {
    if super::targets(doc, options)?.is_empty() {
        let mut pending = vec![(context, 0usize)];
        while let Some((value, depth)) = pending.pop() {
            let object = value.as_object().ok_or_else(|| error("preflight", "context", "missing required target for unused input field"))?;
            if depth > 32 || object.keys().any(|k| k.is_empty() || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')) {
                return Err(error("preflight", "context", "invalid field name or context nesting exceeds 32 levels"));
            }
            pending.extend(object.values().map(|v| (v, depth.saturating_add(1))));
        }
        return Ok(json!({"scalar_count": 0}));
    }
    super::populate_with_options(doc, context, options)
}

fn prepare(
    doc: &Document,
    context: &Value,
    assets_dir: &str,
    options: super::TemplateOptions,
    budget: &mut super::Budget,
) -> Result<(Document, Value), CmdError> {
    prepare_scope(doc, context, assets_dir, false, options, budget)
}

pub(super) fn prepare_scope(
    doc: &Document,
    context: &Value,
    assets_dir: &str,
    scoped: bool,
    options: super::TemplateOptions,
    budget: &mut super::Budget,
) -> Result<(Document, Value), CmdError> {
    if !context.is_object() || serde_json::to_vec(context).map_or(true, |v| v.len() > super::MAX_JSON) {
        return Err(error("preflight", "context", "expected a JSON object of at most 1 MiB"));
    }
    inspect_scope(doc, scoped, options)?;
    let Resolved { mut masked, rows, mut images } = resolve(doc, scoped, options)?;
    let mut context = context.clone();
    let mut expansions = Vec::new();
    let mut count = 0usize;
    for slot in rows {
        let input = take(&mut context, field(&slot.tag, "rows:")?, &slot.tag)?;
        let items = input.as_array().ok_or_else(|| error("preflight", &slot.tag, "expected an array of item objects"))?;
        count = count.saturating_add(items.len());
        let size = serde_json::to_vec(&slot.prototype).map_err(|e| error("preflight", &slot.tag, &e.to_string()))?.len();
        budget.charge(0, items.len(), size.saturating_mul(items.len()), &slot.tag)?;
        let mut expanded = Vec::new();
        for (index, item) in items.iter().enumerate() {
            let mut mini = row_document(doc, &slot.prototype);
            super::populate_with_options(&mut mini, &json!({"item": item}), options)
                .map_err(|e| error("populate", &slot.tag, &format!("scope={}[{index}]: {e}", slot.tag)))?;
            let mut row = mini
                .body
                .first()
                .and_then(|b| b.as_table())
                .and_then(|t| t.rows.first())
                .cloned()
                .ok_or_else(|| error("populate", &slot.tag, "prototype missing"))?;
            consume_controls(&mut row)?;
            let actual = serde_json::to_vec(&row).map_err(|e| error("populate", &slot.tag, &e.to_string()))?.len();
            budget.charge(0, 0, actual, &format!("{}[{index}]", slot.tag))?;
            expanded.push(row);
        }
        expansions.push((slot.table, slot.row, expanded));
    }
    // Replace pictures before row expansion changes paths; reverse order also
    // protects offsets when optional pictures become longer fallback text.
    images.sort_by(|a, b| b.start.cmp(&a.start));
    for r in &images {
        let input = take(&mut context, field(&r.control.tag, "image:")?, &r.control.tag)?;
        replace_image(&mut masked, r, &input, assets_dir)?;
    }
    // Scalar edits change text offsets but never row/block indexes.
    let scalars = scalar_finish(&mut masked, &if scoped { json!({"item": context}) } else { context }, options)?;
    expansions.sort_by_key(|(t, r, _)| (std::cmp::Reverse(*t), std::cmp::Reverse(*r)));
    for (ti, ri, expanded) in expansions {
        let table = masked.table_mut(StoryRef::Body, &Path::top(ti)).map_err(|e| error("populate", "row", &e.to_string()))?;
        if ri >= table.rows.len() || table.rows.len().saturating_sub(1).saturating_add(expanded.len()) > wordcraft_doc::table::MAX_ROWS {
            return Err(error("populate", "row", "target missing or table row limit exceeded"));
        }
        table.rows.splice(ri..ri.saturating_add(1), expanded);
    }
    Ok((masked, json!({"scalar_count": scalars["scalar_count"], "row_count": count, "image_count": images.len(), "save_verified": false})))
}

pub(super) fn validate_image_input(input: &Value, tag: &str) -> Result<(), CmdError> {
    let obj = input.as_object().ok_or_else(|| error("preflight", tag, "expected an image descriptor object"))?;
    if obj.keys().any(|k| !matches!(k.as_str(), "path" | "required" | "unavailable")) || obj.get("required").is_some_and(|v| !v.is_boolean()) {
        return Err(error("preflight", tag, "unknown picture option or non-boolean required flag"));
    }
    let value = obj.get("path").ok_or_else(|| error("preflight", tag, "missing required path option"))?;
    if value.is_null() && obj.get("required") == Some(&Value::Bool(false)) {
        let text = obj
            .get("unavailable")
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| error("preflight", tag, "optional absent chart requires Python-supplied unavailable text"))?;
        validate_text(text, tag)?;
    } else {
        let relative =
            value.as_str().filter(|v| !v.is_empty()).ok_or_else(|| error("preflight", tag, "required chart needs a nonempty relative asset path"))?;
        if relative.contains([':', '\\']) || std::path::Path::new(relative).components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
            return Err(error("preflight", tag, "expected a relative asset path without traversal"));
        }
    }
    Ok(())
}

fn replace_image(doc: &mut Document, r: &ControlRange, input: &Value, assets_dir: &str) -> Result<(), CmdError> {
    let tag = &r.control.tag;
    validate_image_input(input, tag)?;
    let obj = input
        .as_object()
        .ok_or_else(|| error("preflight", tag, "expected {path: relative asset path} or {path: null, required: false, unavailable: text}"))?;
    if obj.keys().any(|k| !matches!(k.as_str(), "path" | "required" | "unavailable")) || obj.get("required").is_some_and(|v| !v.is_boolean()) {
        return Err(error("preflight", tag, "unknown picture option or non-boolean required flag"));
    }
    let Some(path) = obj.get("path") else { return Err(error("preflight", tag, "missing required path option")) };
    if path.is_null() && obj.get("required") == Some(&Value::Bool(false)) {
        let text = obj
            .get("unavailable")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| error("preflight", tag, "optional absent chart requires Python-supplied unavailable text"))?;
        // Reuse scalar validation and replacement, preserving slot paragraph/run properties.
        let c = doc.control_mut(&r.start).ok_or_else(|| error("populate", tag, "picture control missing"))?;
        c.tag = "value:unavailable".into();
        c.kind = ControlKind::RichText;
        let p = doc.para_mut(r.start.story, &r.start.path).map_err(|e| error("populate", tag, &e.to_string()))?;
        p.delete(r.content_start().off, r.end.off).map_err(|e| error("populate", tag, &e.to_string()))?;
        let current =
            doc.control_ranges(r.start.story).into_iter().find(|x| x.start == r.start).ok_or_else(|| error("populate", tag, "slot missing"))?;
        // Validate only this slot; other document tags must not affect the fallback.
        validate_text(text, tag)?;
        let props = doc.para_at(&current.start).map(|p| p.props_of_char(current.start.off).clone()).unwrap_or_default();
        doc.insert_text(&current.content_start(), text, &props).map_err(|e| error("populate", tag, &e.to_string()))?;
        if let Some(c) = doc.control_mut(&r.start) {
            c.tag.clear();
            c.showing_placeholder = false;
        }
        return Ok(());
    }
    let relative = path.as_str().filter(|s| !s.is_empty()).ok_or_else(|| {
        error("preflight", tag, "required chart needs a nonempty relative asset path; missing files never silently become unavailable")
    })?;
    let bytes = asset_bytes(assets_dir, relative, tag)?;
    let pic = wordcraft_geom::picture::probe(&bytes).map_err(|e| error("preflight", tag, &format!("unsupported or invalid image: {e:?}")))?;
    // Header dimensions alone do not establish that the encoded chart is usable.
    const DECODE_LIMIT: u64 = 64 * 1024 * 1024;
    if !pic.fits(DECODE_LIMIT) {
        return Err(error(
            "preflight",
            tag,
            &format!(
                "image allocation budget exceeded: {} x {} pixels; conservative peak estimate={} bytes, limit={} bytes (64 MiB); estimate includes native output, decoder-held storage and RGBA8 conversion buffers; no resizing is performed; {}",
                pic.width,
                pic.height,
                pic.peak_bytes(),
                DECODE_LIMIT,
                pic.allocation_description()
            ),
        ));
    }
    pic.decode(DECODE_LIMIT).map_err(|e| {
        error(
            "preflight",
            tag,
            &format!(
                "image decode failed: {} x {} pixels, estimated peak={} bytes, allocation limit={} bytes: {e}",
                pic.width,
                pic.height,
                pic.peak_bytes(),
                DECODE_LIMIT
            ),
        )
    })?;
    let (nw, nh) = (pic.width as f32, pic.height as f32);
    if nw <= 0.0 || nh <= 0.0 {
        return Err(error("preflight", tag, "image dimensions must be positive"));
    }
    let ext = crate::cmd::insert::image_ext(&bytes);
    let media = doc.add_media(bytes, ext);
    let p = doc.para_mut(r.start.story, &r.start.path).map_err(|e| error("populate", tag, &e.to_string()))?;
    let offset = r.content_start().off;
    let index = p.objects_with_offsets().position(|(off, _)| off == offset).ok_or_else(|| error("populate", tag, "slot image missing"))?;
    if let Some(InlineObject::Image { media: key, w, h, crop, .. }) = p.objects.get_mut(index) {
        let scale = (*w / nw).min(*h / nh);
        *w = nw * scale;
        *h = nh * scale;
        *key = media;
        *crop = [0.0; 4];
    } else {
        return Err(error("populate", tag, "slot is not a picture"));
    }
    p.touch();
    if let Some(c) = doc.control_mut(&r.start) {
        c.showing_placeholder = false;
    }
    Ok(())
}

fn validate_text(text: &str, tag: &str) -> Result<(), CmdError> {
    if text.len() > super::MAX_VALUE
        || text.chars().any(|c| (c.is_control() && c != '\n' && c != '\t') || matches!(c, '\u{fffc}' | '\u{fffe}' | '\u{ffff}'))
    {
        return Err(error("preflight", tag, "unavailable text exceeds 64 KiB or contains unsupported control/XML characters"));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn asset_bytes(root: &str, relative: &str, tag: &str) -> Result<Vec<u8>, CmdError> {
    use std::path::{Component, Path};
    if root.is_empty() || relative.contains([':', '\\']) || Path::new(relative).components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(error("preflight", tag, "supply assets_dir and a relative asset path without traversal, drive prefixes or backslashes"));
    }
    let root = crate::io::canonical(Path::new(root)).ok_or_else(|| error("preflight", tag, "assets_dir is missing or unreadable"))?;
    let path = crate::io::canonical(&root.join(relative))
        .ok_or_else(|| error("preflight", tag, &format!("required asset {relative:?} is missing or unreadable")))?;
    if !path.starts_with(&root) {
        return Err(error("preflight", tag, "asset must remain inside assets_dir after resolving links"));
    }
    let path = path.to_str().ok_or_else(|| error("preflight", tag, "asset path must be UTF-8"))?;
    let bytes = crate::io::read_file_capped(path, 16 * 1024 * 1024, "chart asset").map_err(|e| error("preflight", tag, &e))?;
    Ok(bytes)
}

#[cfg(target_arch = "wasm32")]
fn asset_bytes(_: &str, _: &str, tag: &str) -> Result<Vec<u8>, CmdError> {
    Err(error("preflight", tag, "filesystem chart assets require a native build"))
}

pub(super) fn preflight_with_options(doc: &Document, context: &Value, assets_dir: &str, options: super::TemplateOptions) -> CmdResult {
    prepare(doc, context, assets_dir, options, &mut super::Budget::default()).map(|(_, counts)| counts)
}

pub(super) fn populate_with_options(doc: &mut Document, context: &Value, assets_dir: &str, options: super::TemplateOptions) -> CmdResult {
    let (copy, counts) = prepare(doc, context, assets_dir, options, &mut super::Budget::default())?;
    *doc = copy;
    Ok(counts)
}

#[cfg(test)]
fn inspect(doc: &Document) -> CmdResult {
    inspect_with_options(doc, super::TemplateOptions::default())
}
#[cfg(test)]
fn preflight(doc: &Document, context: &Value, assets: &str) -> CmdResult {
    preflight_with_options(doc, context, assets, super::TemplateOptions::default())
}
#[cfg(test)]
fn populate(doc: &mut Document, context: &Value, assets: &str) -> CmdResult {
    populate_with_options(doc, context, assets, super::TemplateOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordcraft_doc::{Cell, CharProps, ControlWrap, Pos};

    fn paragraph(tag: &str) -> Arc<Block> {
        let mut doc = Document::new();
        let c = ContentControl { tag: tag.into(), id: Some(71), ..Default::default() };
        let at = doc.insert_object(&Pos::body(0, 0), InlineObject::ControlStart { control: Box::new(c) }, &CharProps::default()).unwrap();
        let at = doc.insert_text(&at, "prototype", &CharProps { bold: Some(true), ..Default::default() }).unwrap();
        doc.insert_object(&at, InlineObject::ControlEnd, &CharProps::default()).unwrap();
        doc.body.remove(0)
    }

    fn fixture() -> Document {
        let mut doc = Document::new();
        let header = Row {
            props: wordcraft_doc::props::RowProps { header: true, ..Default::default() },
            cells: vec![Cell::with_text("Class")],
            ..Default::default()
        };
        let row = Row {
            cells: vec![Cell { blocks: vec![paragraph("value:item.name")], ..Cell::empty() }],
            controls: ControlWrap { open: vec![ContentControl { tag: "rows:classes".into(), id: Some(70), ..Default::default() }], close: 1 },
            ..Default::default()
        };
        let table = Table { grid: vec![240.0], rows: vec![header, row], ..Default::default() };
        doc.body = vec![Arc::new(Block::Table(table))];
        doc
    }

    fn chart(doc: &mut Document) {
        let mut mini = Document::new();
        let c = ContentControl { tag: "image:chart".into(), id: Some(72), kind: ControlKind::Picture, ..Default::default() };
        let at = mini.insert_object(&Pos::body(0, 0), InlineObject::ControlStart { control: Box::new(c) }, &CharProps::default()).unwrap();
        let at = mini
            .insert_object(
                &at,
                InlineObject::Image {
                    media: "placeholder.png".into(),
                    w: 180.0,
                    h: 120.0,
                    alt: "chart".into(),
                    float: Default::default(),
                    crop: [0.0; 4],
                    fx: Default::default(),
                    ole: None,
                },
                &CharProps::default(),
            )
            .unwrap();
        mini.insert_object(&at, InlineObject::ControlEnd, &CharProps::default()).unwrap();
        doc.body.extend(mini.body);
    }

    #[test]
    fn row_counts_values_formatting_and_unique_identifiers() {
        for count in [0, 1, 3, 150] {
            let mut doc = fixture();
            let before = doc.body[0].as_table().unwrap().clone();
            let items: Vec<_> = (0..count).map(|i| json!({"name": format!("類別 {i} — é & <x>\nSecond line")})).collect();
            let result = populate(&mut doc, &json!({"classes": items}), "").unwrap();
            assert_eq!(result["row_count"], count);
            let bytes = wordcraft_docx::write(&doc).unwrap();
            let back = wordcraft_docx::read(&bytes).unwrap();
            let table = back.body[0].as_table().unwrap();
            assert_eq!(table.rows.len(), count + 1);
            assert_eq!(table.grid, before.grid);
            assert!(table.rows[0].props.header);
            for (i, row) in table.rows.iter().skip(1).enumerate() {
                assert!(row.controls.is_empty());
                assert_eq!(row.cells[0].blocks[0].as_para().unwrap().plain_text(), format!("類別 {i} — é & <x>\nSecond line"));
                assert_eq!(row.cells[0].blocks[0].as_para().unwrap().props_of_char(0).bold, Some(true));
                assert_eq!(row.cells[0].props, before.rows[1].cells[0].props);
            }
            assert!(back.control_ranges(StoryRef::Body).is_empty());
            assert!(!back.has_unbalanced_controls());
        }
    }

    #[test]
    fn malformed_rows_and_scopes_fail_before_mutation() {
        for context in [
            json!({}),
            json!({"classes": null}),
            json!({"classes": [{"wrong": "x"}]}),
            json!({"classes": [null]}),
            json!({"classes": [], "unused": 42}),
            json!({"classes": vec![json!({"name": "x"}); 1001]}),
        ] {
            let mut doc = fixture();
            let before = doc.clone();
            assert!(populate(&mut doc, &context, "").is_err());
            assert_eq!(doc, before);
        }
        for variant in 0..7 {
            let mut doc = fixture();
            let table = doc.table_mut(StoryRef::Body, &Path::top(0)).unwrap();
            match variant {
                0 => table.rows[1].controls.close = 0,
                1 => table.rows[1].cells[0].props.vmerge = wordcraft_doc::props::VMerge::Restart,
                2 => table.rows[1].cells[0].blocks.push(Arc::new(Block::Table(Table::new(1, 1, 100.0)))),
                3 => table.rows[1].cells[0].blocks.push(paragraph("value:item.name")),
                4 => {
                    table.rows.remove(0);
                }
                5 => table.rows[1].controls.open[0].lock = wordcraft_doc::ControlLock::SdtLocked,
                _ => {
                    doc.body.push(paragraph("value:item.outside"));
                }
            }
            let before = doc.clone();
            assert!(populate(&mut doc, &json!({"classes": [{"name": "x"}]}), "").is_err());
            assert_eq!(doc, before);
        }
    }

    #[test]
    fn independent_row_scopes_and_global_scalars_share_registry_and_undo() {
        let mut doc = fixture();
        let mut second = doc.body[0].as_table().unwrap().clone();
        second.rows[1].controls.open[0].tag = "rows:other".into();
        doc.body.push(Arc::new(Block::Table(second)));
        doc.body.push(paragraph("value:title"));
        let inspection = inspect(&doc).unwrap();
        assert_eq!(inspection["targets"].as_array().unwrap().len(), 5);
        let before = doc.clone();
        let mut session = crate::Session::new(doc);
        session
            .run(
                "template.populateRowsAndImages",
                &json!({"context": {"classes": [{"name": "first"}], "other": [{"name": "second"}], "title": "Report"}, "assets_dir": ""}),
            )
            .unwrap();
        assert_eq!(session.doc.body[0].as_table().unwrap().rows[1].cells[0].blocks[0].as_para().unwrap().plain_text(), "first");
        assert_eq!(session.doc.body[1].as_table().unwrap().rows[1].cells[0].blocks[0].as_para().unwrap().plain_text(), "second");
        session.run("edit.undo", &json!({})).unwrap();
        assert_eq!(session.doc, before);
    }

    #[test]
    fn optional_chart_requires_explicit_absence_and_preserves_atomicity() {
        let mut doc = fixture();
        chart(&mut doc);
        for value in [
            json!({"path": null}),
            json!({"path": "missing.png", "required": false}),
            json!({"path": null, "required": false}),
            json!({"path": null, "required": false, "unavailable": "\u{0000}"}),
        ] {
            let before = doc.clone();
            assert!(populate(&mut doc, &json!({"classes": [], "chart": value}), "").is_err());
            assert_eq!(doc, before);
        }
        populate(&mut doc, &json!({"classes": [], "chart": {"path": null, "required": false, "unavailable": "Chart unavailable — Ω"}}), "").unwrap();
        assert_eq!(doc.body[1].as_para().unwrap().plain_text(), "Chart unavailable — Ω");
        assert_eq!(doc.body[0].as_table().unwrap().rows.len(), 1);
    }

    #[test]
    fn chart_fits_both_dimensions_and_invalid_bytes_are_rejected() {
        let root = std::env::temp_dir().join(format!("foliorivet-stage4-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("chart.png");
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(40, 200).write_to(&mut png, image::ImageFormat::Png).unwrap();
        std::fs::write(&path, png.into_inner()).unwrap();
        let mut doc = fixture();
        chart(&mut doc);
        let input = json!({"classes": [{"name": "A"}], "chart": {"path": "chart.png"}});
        assert_eq!(preflight(&doc, &input, root.to_str().unwrap()).unwrap()["image_count"], 1);
        populate(&mut doc, &input, root.to_str().unwrap()).unwrap();
        let p = doc.body[1].as_para().unwrap();
        let image = p.objects.iter().find(|o| matches!(o, InlineObject::Image { .. })).unwrap();
        assert!(matches!(image, InlineObject::Image { w, h, .. } if (*w - 24.0).abs() < 0.01 && (*h - 120.0).abs() < 0.01));
        for relative in ["../chart.png", "C:/chart.png", "sub\\chart.png", "/chart.png"] {
            assert!(asset_bytes(root.to_str().unwrap(), relative, "image:chart").is_err());
        }
        std::fs::write(&path, b"not a picture").unwrap();
        let mut bad = fixture();
        chart(&mut bad);
        let before = bad.clone();
        assert!(populate(&mut bad, &input, root.to_str().unwrap()).is_err());
        assert_eq!(bad, before);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn row_fanout_and_global_rows_share_the_budget() {
        let options = super::super::TemplateOptions { scalar_fanout: true };
        let mut doc = fixture();
        let table = doc.table_mut(StoryRef::Body, &Path::top(0)).unwrap();
        table.rows[1].cells[0].blocks.push(paragraph("value:item.name"));
        let context = json!({"classes":[{"name":"Alpha"},{"name":"Beta"}]});
        assert!(preflight(&doc, &context, "").is_err());
        assert_eq!(populate_with_options(&mut doc, &context, "", options).unwrap()["scalar_count"], 0);
        let table = doc.body[0].as_table().unwrap();
        for (i, row) in table.rows.iter().skip(1).enumerate() {
            for block in &row.cells[0].blocks {
                assert_eq!(block.as_para().unwrap().text, if i == 0 { "Alpha" } else { "Beta" });
            }
        }
        let mut budget = super::super::Budget::default();
        budget.charge(0, 999, 0, "other repeat").unwrap();
        let err = prepare_scope(&fixture(), &context, "", false, options, &mut budget).unwrap_err().to_string();
        assert!(err.contains("row budget") && err.contains("used=1001"));
    }
    #[test]
    fn analysis_does_not_open_required_images() {
        let mut doc = fixture();
        chart(&mut doc);
        let context = json!({"classes":[],"chart":{"path":"never-created.png"}});
        let result = super::super::analyze_context(&doc, &context, super::super::TemplateOptions::default()).unwrap();
        assert_eq!(result["missing_paths"], json!([]));
        assert_eq!(result["unused_supplied_paths"], json!([]));
        assert_eq!(result["diagnostics"], json!([]));
        assert!(preflight(&doc, &context, "").is_err());
    }
    #[test]
    fn rows_honor_remaining_document_byte_budget_at_exact_boundary() {
        let doc = fixture();
        let context = json!({"classes":[{"name":"Alpha"},{"name":"Beta"}]});
        let options = super::super::TemplateOptions::default();
        let limit = 16 * 1024 * 1024;
        let mut measured = super::super::Budget::default();
        prepare_scope(&doc, &context, "", false, options, &mut measured).unwrap();
        assert!(measured.bytes > 0);
        for extra in [0, 1] {
            let mut budget = super::super::Budget::default();
            budget.charge(0, 0, limit - measured.bytes + extra, "earlier repeat").unwrap();
            let result = prepare_scope(&doc, &context, "", false, options, &mut budget);
            assert_eq!(budget.bytes, limit + extra);
            if extra == 0 {
                assert!(result.is_ok());
            } else {
                let message = result.unwrap_err().to_string();
                assert!(message.contains("expansion byte budget") && message.contains("used=16777217") && message.contains("rows:classes[1]"));
            }
        }
    }
    #[test]
    fn progressive_chart_preserves_original_bytes_and_aspect_ratio() {
        let bytes: &[u8] = include_bytes!("../../../geom/tests/data/progressive-rgb.jpg");
        let root = std::env::temp_dir().join(format!("foliorivet-progressive-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("chart.jpg"), bytes).unwrap();
        let mut doc = fixture();
        chart(&mut doc);
        let input = json!({"classes":[],"chart":{"path":"chart.jpg"}});
        populate(&mut doc, &input, root.to_str().unwrap()).unwrap();
        let back = wordcraft_docx::read(&wordcraft_docx::write(&doc).unwrap()).unwrap();
        for doc in [&doc, &back] {
            let p = doc.body[1].as_para().unwrap();
            let (media, w, h) = p
                .objects
                .iter()
                .find_map(|object| match object {
                    InlineObject::Image { media, w, h, .. } => Some((media, *w, *h)),
                    _ => None,
                })
                .unwrap();
            assert_eq!(doc.media[media].as_ref().as_slice(), bytes);
            assert!((w - 180.0).abs() < 0.01 && (h - 90.0).abs() < 0.01);
        }
        std::fs::remove_file(root.join("chart.jpg")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
