//! Independently authored bounded whole-block repetition and boolean inclusion.
//! Boundary resolution uses the document model's paired control events, never raw XML.
use std::{collections::BTreeSet, sync::Arc};

use super::{CmdError, CmdResult, Document, Value, error, json, stage4};
use wordcraft_doc::control::{ControlEvent, control_events, pair_events, walk_controls};
use wordcraft_doc::{Block, Blocks, ControlKind, InlineObject};

pub(super) struct Slot {
    pub(super) tag: String,
    pub(super) first: usize,
    pub(super) last: usize,
    pub(super) prototype: Document,
    pub(super) repeat: bool,
}

pub(super) fn needed(doc: &Document) -> bool {
    let mut found = false;
    for blocks in std::iter::once(&doc.body).chain(doc.parts.values().map(|p| &p.blocks)) {
        walk_controls(blocks, &mut |c| found |= c.tag.starts_with("repeat:") || c.tag.starts_with("if:"));
    }
    found
}

fn key(tag: &str) -> Result<&str, CmdError> {
    let (_, key) = tag.split_once(':').ok_or_else(|| error("resolve", tag, "missing prefix"))?;
    if key.is_empty() || key.len() > 256 || key.split('.').any(|s| s.is_empty() || !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')) {
        return Err(error("resolve", tag, "use a dotted ASCII field path"));
    }
    Ok(key)
}

fn boundary(blocks: &mut Blocks, index: usize, off: Option<usize>, opening: bool, tag: &str) -> Result<(), CmdError> {
    let b = blocks.get_mut(index).ok_or_else(|| error("resolve", tag, "boundary block missing"))?;
    match (Arc::make_mut(b), off) {
        (Block::Para(p), Some(off)) => {
            if (opening && off != 0) || (!opening && off.saturating_add(wordcraft_doc::para::OBJ.len_utf8()) != p.text.len()) {
                return Err(error("resolve", tag, "wrapper must include complete paragraphs, including its heading and spacing"));
            }
            p.delete(off, off.saturating_add(wordcraft_doc::para::OBJ.len_utf8())).map_err(|e| error("resolve", tag, &e.to_string()))?;
        }
        (Block::Table(t), None) => {
            if opening {
                if t.controls.open.first().is_none_or(|c| c.tag != tag) {
                    return Err(error("resolve", tag, "invalid table boundary"));
                }
                t.controls.open.remove(0);
            } else {
                t.controls.close = t.controls.close.checked_sub(1).ok_or_else(|| error("resolve", tag, "missing table close"))?;
            }
        }
        _ => return Err(error("resolve", tag, "invalid block boundary")),
    }
    Ok(())
}

// A copied block cannot carry identifiers/references whose remapping is unsupported.
// Ordinary pictures use the existing media store; the writer allocates drawing IDs.
fn check_copy(doc: &Document, blocks: &Blocks, tag: &str, depth: usize, repeat: bool) -> Result<(), CmdError> {
    if depth > 16 {
        return Err(error("resolve", tag, "table nesting exceeds 16"));
    }
    for b in blocks {
        match b.as_ref() {
            Block::Para(p) => {
                if let Some(n) = doc.styles.resolve_para(&p.props).numbering
                    && (repeat || !doc.numbering.level(n.num, n.level).is_some_and(|level| level.format == wordcraft_doc::section::NumFormat::Bullet))
                {
                    return Err(error(
                        "resolve",
                        tag,
                        "numbering requires a valid native bullet in a conditional; repeated numbering and numbered lists are unsupported",
                    ));
                }
                if p.section.is_some()
                    || p.props.fmt_change.is_some()
                    || p.props.num_change.is_some()
                    || std::iter::once(&p.mark)
                        .chain(p.runs.iter().map(|r| &r.props))
                        .any(|p| p.ins.is_some() || p.del.is_some() || p.fmt_change.is_some() || p.link.is_some())
                {
                    return Err(error("resolve", tag, "sections, tracked changes and hyperlinks are unsupported in variable blocks"));
                }
                for o in &p.objects {
                    match o {
                        InlineObject::ControlStart { control } if !control.tag.is_empty() && control.lock.can_delete() => {}
                        InlineObject::ControlEnd => {}
                        InlineObject::Image { float, crop, ole: None, .. }
                            if float.wrap == wordcraft_doc::para::Wrap::Inline && *crop == [0.0; 4] => {}
                        _ => {
                            return Err(error(
                                "resolve",
                                tag,
                                "unsupported reference/object in variable block; only text, template controls and ordinary inline pictures are supported",
                            ));
                        }
                    }
                }
            }
            Block::Table(t) => {
                if !t.controls.is_empty()
                    || t.rows
                        .iter()
                        .any(|r| r.controls.open.iter().any(|c| !c.tag.starts_with("rows:")) || r.cells.iter().any(|c| !c.controls.is_empty()))
                {
                    return Err(error("resolve", tag, "only supported rows: wrappers are allowed on tables, rows and cells in variable blocks"));
                }
                if t.props.fmt_change.is_some()
                    || t.rows
                        .iter()
                        .any(|r| r.props.fmt_change.is_some() || r.cells.iter().any(|c| c.props.fmt_change.is_some() || c.props.picture.is_some()))
                {
                    return Err(error("resolve", tag, "tracked tables and cell background pictures are unsupported in variable blocks"));
                }
                for r in &t.rows {
                    for c in &r.cells {
                        check_copy(doc, &c.blocks, tag, depth.saturating_add(1), repeat)?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn resolve(doc: &Document, options: super::TemplateOptions) -> Result<Vec<Slot>, CmdError> {
    if doc.has_unbalanced_controls() || doc.settings.protection.is_some() || doc.settings.track_changes {
        return Err(error("resolve", "document", "unbalanced controls, protection or tracked-change editing are unsupported"));
    }
    let events = control_events(&doc.body);
    let (pairs, _) = pair_events(&events);
    let mut slots = Vec::new();
    let mut child_count = 0usize;
    let mut names = BTreeSet::new();
    for (a, b, depth) in pairs {
        let (Some(ControlEvent::Open { block: first, off: start, control }), Some(ControlEvent::Close { block: last, off: end })) =
            (events.get(a), events.get(b))
        else {
            continue;
        };
        let repeat = control.tag.starts_with("repeat:");
        if !repeat && !control.tag.starts_with("if:") {
            continue;
        }
        key(&control.tag)?;
        if depth != 0 {
            // Child wrappers are validated by resolving the outer prototype below.
            continue;
        }
        if depth != 0
            || control.kind != ControlKind::RichText
            || !control.block
            || !control.extra.is_empty()
            || !control.lock.can_delete()
            || !control.lock.can_edit()
        {
            return Err(error(
                "resolve",
                &control.tag,
                "requires an unlocked top-level ordinary rich-text block wrapper; nested variable blocks are unsupported",
            ));
        }
        if !names.insert(control.tag.clone()) {
            return Err(error("resolve", &control.tag, "ambiguous required block target"));
        }
        let mut mini = doc.clone();
        mini.parts.clear();
        mini.body = doc.body.get(*first..last.saturating_add(1)).ok_or_else(|| error("resolve", &control.tag, "invalid block range"))?.to_vec();
        // Close first, so a single-paragraph wrapper's start offset remains valid.
        boundary(&mut mini.body, last.saturating_sub(*first), *end, false, &control.tag)?;
        boundary(&mut mini.body, 0, *start, true, &control.tag)?;
        check_copy(doc, &mini.body, &control.tag, 0, repeat)?;
        let mut nested = false;
        walk_controls(&mini.body, &mut |c| nested |= c.tag.starts_with("repeat:") || c.tag.starts_with("if:"));
        let inspection = if nested {
            if repeat {
                return Err(error("resolve", &control.tag, "nested variable blocks require an outer if: and immediate repeat: children"));
            }
            let mut nested_if = false;
            walk_controls(&mini.body, &mut |c| nested_if |= c.tag.starts_with("if:"));
            if nested_if {
                return Err(error("resolve", &control.tag, "nested conditionals are unsupported"));
            }
            let children = resolve(&mini, options)?;
            if children.iter().any(|child| !child.repeat) {
                return Err(error("resolve", &control.tag, "nested conditionals are unsupported; only repeat: children are allowed"));
            }
            child_count = child_count.saturating_add(children.len());
            inspect_with_options(&mini, options)?
        } else {
            stage4::inspect_scope(&mini, repeat, options)?
        };
        for entry in inspection["targets"].as_array().into_iter().flatten() {
            if entry.get("supported") == Some(&Value::Bool(false)) {
                return Err(error("resolve", &control.tag, &format!("unsupported scoped target: {entry}")));
            }
            if repeat
                && entry.get("scope").is_none_or(|scope| scope == "document")
                && entry["tag"].as_str().is_some_and(|s| s.starts_with("value:") && !s.starts_with("value:item."))
            {
                return Err(error("resolve", &control.tag, "repeated scalar fields must use value:item.*"));
            }
        }
        slots.push(Slot { tag: control.tag.clone(), first: *first, last: *last, prototype: mini, repeat });
    }
    slots.sort_by_key(|s| s.first);
    if slots.windows(2).any(|s| s.first().zip(s.get(1)).is_some_and(|(a, b)| a.last >= b.first)) {
        return Err(error("resolve", "document", "variable blocks must have separate complete block boundaries"));
    }
    // Every variable tag must have been resolved, including wrappers in other stories/cells.
    let mut count = 0usize;
    for blocks in std::iter::once(&doc.body).chain(doc.parts.values().map(|p| &p.blocks)) {
        walk_controls(blocks, &mut |c| count = count.saturating_add(usize::from(c.tag.starts_with("repeat:") || c.tag.starts_with("if:"))));
    }
    if count != slots.len().saturating_add(child_count) || count > super::MAX_TARGETS {
        return Err(error("resolve", "document", "variable blocks require complete top-level body boundaries; at most 1024 targets"));
    }
    Ok(slots)
}

pub(super) fn masked(doc: &Document, slots: &[Slot]) -> Document {
    let mut copy = doc.clone();
    for s in slots.iter().rev() {
        copy.body.drain(s.first..s.last.saturating_add(1));
    }
    copy
}

pub(super) fn inspect_with_options(doc: &Document, options: super::TemplateOptions) -> CmdResult {
    let slots = resolve(doc, options)?;
    let mut result = stage4::inspect_with_options(&masked(doc, &slots), options)?;
    let entries = result["targets"].as_array_mut().ok_or_else(|| error("inspect", "document", "target list missing"))?;
    for s in slots {
        entries.push(json!({"tag":s.tag,"scope":"document","kind":if s.repeat {"repeat"} else {"optional"},"supported":true,"first_block":s.first,"last_block":s.last}));
        let inner =
            if needed(&s.prototype) { inspect_with_options(&s.prototype, options)? } else { stage4::inspect_scope(&s.prototype, s.repeat, options)? };
        for mut entry in inner["targets"].as_array().cloned().unwrap_or_default() {
            let scope = entry["scope"].as_str().filter(|scope| *scope != "document");
            entry["scope"] = json!(scope.map_or_else(|| s.tag.clone(), |scope| format!("{} / {scope}", s.tag)));
            entries.push(entry);
        }
    }
    if entries.len() > super::MAX_TARGETS {
        return Err(error("inspect", "document", "more than 1024 tagged targets"));
    }
    Ok(result)
}

fn consume(blocks: &mut Blocks) -> Result<(), CmdError> {
    for b in blocks {
        match Arc::make_mut(b) {
            Block::Para(p) => {
                let offsets: Vec<_> = p
                    .objects_with_offsets()
                    .filter_map(|(off, o)| matches!(o, InlineObject::ControlStart { .. } | InlineObject::ControlEnd).then_some(off))
                    .collect();
                for off in offsets.into_iter().rev() {
                    p.delete(off, off.saturating_add(wordcraft_doc::para::OBJ.len_utf8())).map_err(|e| error("populate", "block", &e.to_string()))?;
                }
            }
            Block::Table(t) => {
                t.controls = Default::default();
                for r in &mut t.rows {
                    r.controls = Default::default();
                    for c in &mut r.cells {
                        c.controls = Default::default();
                        consume(&mut c.blocks)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn prepare(
    doc: &Document,
    context: &Value,
    assets: &str,
    options: super::TemplateOptions,
    budget: &mut super::Budget,
) -> Result<(Document, Value), CmdError> {
    if !context.is_object() || serde_json::to_vec(context).map_or(true, |v| v.len() > super::MAX_JSON) {
        return Err(error("preflight", "context", "expected an object of at most 1 MiB"));
    }
    inspect_with_options(doc, options)?;
    let slots = resolve(doc, options)?;
    let mut context = context.clone();
    let mut copy = doc.clone();
    // Unwrap all conditionals before resolving retained repeats together.
    if slots.iter().any(|s| !s.repeat) {
        let mut included = 0usize;
        for s in slots.iter().filter(|s| !s.repeat).rev() {
            let input = stage4::take(&mut context, key(&s.tag)?, &s.tag)?;
            let include = input.as_bool().ok_or_else(|| error("preflight", &s.tag, "expected an explicit boolean"))?;
            included += usize::from(include);
            let blocks = if include { s.prototype.body.clone() } else { Vec::new() };
            copy.body.splice(s.first..s.last.saturating_add(1), blocks);
        }
        if copy.body.is_empty() {
            copy.body.push(wordcraft_doc::para_block(wordcraft_doc::Paragraph::new()));
        }
        let (copy, mut result) = if needed(&copy) {
            prepare(&copy, &context, assets, options, budget)
        } else {
            stage4::prepare_scope(&copy, &context, assets, false, options, budget)
        }
        .map_err(|e| {
            error(
                "preflight",
                "conditional children",
                &format!("parent scopes={:?}: {e}", slots.iter().filter(|s| !s.repeat).map(|s| &s.tag).collect::<Vec<_>>()),
            )
        })?;
        result["included_optional_blocks"] = json!(included);
        if result.get("repeated_items").is_none() {
            result["repeated_items"] = json!(0);
        }
        return Ok((copy, result));
    }
    let mut edits = Vec::new();
    let mut count = 0usize;
    let mut included = 0usize;
    let mut scoped_counts = [0u64; 3];
    for s in &slots {
        let input = stage4::take(&mut context, key(&s.tag)?, &s.tag)?;
        let mut replacement = Vec::new();
        if s.repeat {
            let items = input.as_array().ok_or_else(|| error("preflight", &s.tag, "expected an array of item objects"))?;
            count = count.saturating_add(items.len());
            let size = serde_json::to_vec(&s.prototype.body).map_err(|e| error("preflight", &s.tag, &e.to_string()))?.len();
            budget.charge(items.len(), 0, size.saturating_mul(items.len()), &s.tag)?;
            for (i, item) in items.iter().enumerate() {
                let mut prototype = s.prototype.clone();
                prototype.media = copy.media.clone();
                let (mut rendered, child_counts) = stage4::prepare_scope(&prototype, item, assets, true, options, budget)
                    .map_err(|e| error("populate", &s.tag, &format!("scope={}[{i}]: {e}", s.tag)))?;
                for (total, key) in scoped_counts.iter_mut().zip(["scalar_count", "row_count", "image_count"]) {
                    *total = total.saturating_add(child_counts[key].as_u64().unwrap_or(0));
                }
                consume(&mut rendered.body)?;
                let actual = serde_json::to_vec(&rendered.body).map_err(|e| error("populate", &s.tag, &e.to_string()))?.len();
                budget.charge(0, 0, actual, &format!("{}[{i}]", s.tag))?;
                copy.media.extend(rendered.media);
                replacement.extend(rendered.body);
            }
        } else {
            let include = input.as_bool().ok_or_else(|| error("preflight", &s.tag, "expected an explicit boolean"))?;
            if include {
                included = included.saturating_add(1);
                replacement = s.prototype.body.clone();
            }
        }
        edits.push((s.first, s.last, replacement));
    }
    // Optional blocks must be included/removed before resolving global scalars.
    // Splice all slots with unique untagged placeholders for repeats, then splice
    // their final bodies after population using the adjusted block indexes.
    let mut repeat_positions = Vec::new();
    let mut body = Vec::new();
    let mut cursor = 0usize;
    for ((first, last, replacement), slot) in edits.iter().zip(&slots) {
        body.extend(copy.body.get(cursor..*first).unwrap_or_default().iter().cloned());
        if slot.repeat {
            repeat_positions.push((body.len(), replacement.clone()));
            body.push(wordcraft_doc::para_block(wordcraft_doc::Paragraph::new()));
        } else {
            body.extend(replacement.iter().cloned());
        }
        cursor = last.saturating_add(1);
    }
    body.extend(copy.body.get(cursor..).unwrap_or_default().iter().cloned());
    copy.body = body;
    let (mut copy, mut counts) = stage4::prepare_scope(&copy, &context, assets, false, options, budget)?;
    for (at, blocks) in repeat_positions.into_iter().rev() {
        copy.body.splice(at..at.saturating_add(1), blocks);
    }
    if copy.body.is_empty() {
        copy.body.push(wordcraft_doc::para_block(wordcraft_doc::Paragraph::new()));
    }
    if needed(&copy) || copy.has_unbalanced_controls() {
        return Err(error("populate", "document", "unresolved required block or invalid controls"));
    }
    counts["repeated_items"] = json!(count);
    counts["included_optional_blocks"] = json!(included);
    for (total, key) in scoped_counts.iter().zip(["scalar_count", "row_count", "image_count"]) {
        counts[key] = json!(counts[key].as_u64().unwrap_or(0).saturating_add(*total));
    }
    Ok((copy, counts))
}

pub(super) fn preflight_with_options(doc: &Document, context: &Value, assets: &str, options: super::TemplateOptions) -> CmdResult {
    prepare(doc, context, assets, options, &mut super::Budget::default()).map(|(_, v)| v)
}
pub(super) fn populate_with_options(doc: &mut Document, context: &Value, assets: &str, options: super::TemplateOptions) -> CmdResult {
    let (copy, result) = prepare(doc, context, assets, options, &mut super::Budget::default())?;
    *doc = copy;
    Ok(result)
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
    use wordcraft_doc::{CharProps, ContentControl, Pos, Table};

    fn text(tag: &str, value: &str) -> Arc<Block> {
        let mut doc = Document::new();
        let c = ContentControl { tag: tag.into(), id: Some(50), ..Default::default() };
        let at = doc.insert_object(&Pos::body(0, 0), InlineObject::ControlStart { control: Box::new(c) }, &CharProps::default()).unwrap();
        let at = doc.insert_text(&at, value, &CharProps { bold: Some(true), ..Default::default() }).unwrap();
        doc.insert_object(&at, InlineObject::ControlEnd, &CharProps::default()).unwrap();
        doc.body.remove(0)
    }
    fn wrap(doc: &mut Document, first: usize, last: usize, tag: &str) {
        let c = ContentControl { tag: tag.into(), block: true, id: Some(49), ..Default::default() };
        doc.insert_object(&Pos::body(first, 0), InlineObject::ControlStart { control: Box::new(c) }, &CharProps::default()).unwrap();
        let len = doc.body[last].as_para().unwrap().text.len();
        doc.insert_object(&Pos::body(last, len), InlineObject::ControlEnd, &CharProps::default()).unwrap();
    }
    fn fixture() -> Document {
        let mut doc = Document::new();
        let mut table = Table::new(2, 1, 240.0);
        table.rows[0].props.header = true;
        table.rows[1].cells[0].blocks = vec![text("value:item.score", "score")];
        doc.body = vec![
            text("value:title", "title"),
            text("value:item.name", "heading"),
            Arc::new(Block::Table(table)),
            wordcraft_doc::para_block(wordcraft_doc::Paragraph::with_text("Manual comments: ", Default::default())),
            wordcraft_doc::para_block(wordcraft_doc::Paragraph::with_text("Evaluation heading", Default::default())),
            text("value:evaluation", "pending"),
        ];
        wrap(&mut doc, 1, 3, "repeat:classes");
        wrap(&mut doc, 4, 5, "if:has_evaluation");
        doc
    }
    #[test]
    fn conditional_bullets_survive_and_repeated_numbering_is_rejected() {
        let mut doc = Document::new();
        let num = Arc::make_mut(&mut doc.numbering).add_list(wordcraft_doc::numbering::ListKind::Bullet);
        doc.body = ["First", "Second", "Outside"]
            .into_iter()
            .map(|text| wordcraft_doc::para_block(wordcraft_doc::Paragraph::with_text(text, Default::default())))
            .collect();
        for (level, block) in doc.body.iter_mut().take(2).enumerate() {
            if let Block::Para(p) = Arc::make_mut(block) {
                p.props.numbering = Some(wordcraft_doc::props::NumRef { num, level: level as u8 });
            }
        }
        wrap(&mut doc, 0, 1, "if:show");
        for show in [false, true] {
            let mut copy = doc.clone();
            populate(&mut copy, &json!({"show": show}), "").unwrap();
            let copy = wordcraft_docx::read(&wordcraft_docx::write(&copy).unwrap()).unwrap();
            assert_eq!(copy.body.len(), if show { 3 } else { 1 });
            if show {
                for (level, block) in copy.body.iter().take(2).enumerate() {
                    let n = copy.styles.resolve_para(&block.as_para().unwrap().props).numbering.unwrap();
                    assert_eq!(usize::from(n.level), level);
                    assert_eq!(copy.numbering.level(n.num, n.level).unwrap().format, wordcraft_doc::section::NumFormat::Bullet);
                }
            }
        }
        assert!(check_copy(&doc, &doc.body, "repeat:items", 0, true).unwrap_err().to_string().contains("numbering"));
    }

    #[test]
    fn conditional_repeat_children_are_active_only_when_included() {
        let mut doc = Document::new();
        doc.body =
            vec![text("value:item.name", "name"), wordcraft_doc::para_block(wordcraft_doc::Paragraph::with_text("Outside", Default::default()))];
        wrap(&mut doc, 0, 0, "repeat:items");
        // Use a separate heading so outer and child boundaries remain distinct.
        doc.body.insert(0, wordcraft_doc::para_block(wordcraft_doc::Paragraph::with_text("Heading", Default::default())));
        wrap(&mut doc, 0, 1, "if:show");
        assert!(inspect(&doc).is_ok());
        for n in [0, 1, 3] {
            let mut copy = doc.clone();
            let result = populate(&mut copy, &json!({"show":true,"items":vec![json!({"name":"Acme"});n]}), "").unwrap();
            assert_eq!(result["repeated_items"], n);
            assert_eq!(copy.body.len(), n + 2);
        }
        let mut copy = doc.clone();
        populate(&mut copy, &json!({"show":false}), "").unwrap();
        assert_eq!(copy.body.len(), 1);
        assert!(preflight(&doc, &json!({"show":false,"items":[]}), "").is_err());
    }

    #[test]
    fn independent_repeats_share_the_item_budget() {
        let mut doc = Document::new();
        doc.body = vec![text("value:item.name", "first"), text("value:item.name", "second")];
        wrap(&mut doc, 0, 0, "repeat:first");
        wrap(&mut doc, 1, 1, "repeat:second");
        let items = |n| vec![json!({"name":"x"}); n];
        assert_eq!(preflight(&doc, &json!({"first":items(500),"second":items(500)}), "").unwrap()["repeated_items"], 1000);
        let before = doc.clone();
        let err = populate(&mut doc, &json!({"first":items(600),"second":items(600)}), "").unwrap_err();
        assert!(err.to_string().contains("repeat item budget"));
        assert_eq!(doc, before);
    }

    #[test]
    fn blocks_match_items_and_remove_complete_optional_content() {
        for n in [0, 1, 3, 80] {
            for evaluation in [false, true] {
                let mut doc = fixture();
                let mut input = json!({"title":"Report","classes":(0..n).map(|i| json!({"name":format!("類別 {i}\nline"),"score":i})).collect::<Vec<_>>(),"has_evaluation":evaluation});
                if evaluation {
                    input["evaluation"] = json!("passed");
                }
                assert!(inspect(&doc).is_ok());
                assert_eq!(preflight(&doc, &input, "").unwrap()["repeated_items"], n);
                populate(&mut doc, &input, "").unwrap();
                assert_eq!(doc.body.len(), 1 + n * 3 + usize::from(evaluation) * 2);
                assert_eq!(doc.body.iter().filter(|b| b.as_table().is_some()).count(), n);
                let bytes = wordcraft_docx::write(&doc).unwrap();
                let back = wordcraft_docx::read(&bytes).unwrap();
                assert!(!needed(&back) && !back.has_unbalanced_controls());
                for i in 0..n {
                    assert_eq!(back.body[1 + i * 3].as_para().unwrap().plain_text(), format!("類別 {i}\nline"));
                    assert_eq!(back.body[1 + i * 3].as_para().unwrap().props_of_char(0).bold, Some(true));
                    assert!(back.body[2 + i * 3].as_table().unwrap().rows[0].props.header);
                    assert_eq!(back.body[3 + i * 3].as_para().unwrap().plain_text(), "Manual comments: ");
                }
            }
        }
    }
    #[test]
    fn block_failures_are_atomic_and_scoped() {
        for input in [
            json!({}),
            json!({"title":"x","classes":null,"has_evaluation":false}),
            json!({"title":"x","classes":[{}],"has_evaluation":false}),
            json!({"title":"x","classes":[],"has_evaluation":"false"}),
            json!({"title":"x","classes":[],"has_evaluation":true}),
            json!({"title":"x","classes":[],"has_evaluation":false,"unused":1}),
        ] {
            let mut doc = fixture();
            let before = doc.clone();
            let e = populate(&mut doc, &input, "").unwrap_err().to_string();
            assert!(e.contains("target=") && e.contains("scope="), "{e}");
            assert_eq!(doc, before);
        }
        let mut doc = fixture();
        doc.body.push(text("value:item.name", "outside"));
        assert!(inspect(&doc).is_err());
        let mut doc = fixture();
        let dup = doc.body[1..4].to_vec();
        doc.body.extend(dup);
        assert!(inspect(&doc).unwrap_err().to_string().contains("ambiguous"));
    }
    #[test]
    fn empty_input_still_validates_prototype_and_registry_undo() {
        let mut doc = fixture();
        let Block::Para(p) = Arc::make_mut(&mut doc.body[1]) else { panic!() };
        p.section = Some(Box::default());
        assert!(preflight(&doc, &json!({"title":"x","classes":[],"has_evaluation":false}), "").is_err());
        let doc = fixture();
        let before = doc.clone();
        let mut s = crate::Session::new(doc);
        s.run("template.populateBlocks", &json!({"context":{"title":"x","classes":[],"has_evaluation":false},"assets_dir":""})).unwrap();
        s.run("edit.undo", &json!({})).unwrap();
        assert_eq!(s.doc, before);
    }

    #[test]
    fn table_boundaries_and_partial_or_nested_wrappers() {
        let mut doc = Document::new();
        let mut t = Table::new(1, 1, 240.0);
        t.rows[0].cells[0].blocks = vec![text("value:item.name", "name")];
        t.controls.open.push(ContentControl { tag: "repeat:classes".into(), block: true, ..Default::default() });
        t.controls.close = 1;
        doc.body = vec![Arc::new(Block::Table(t))];
        populate(&mut doc, &json!({"classes":[{"name":"First"},{"name":"Second"}]}), "").unwrap();
        assert_eq!(doc.body.len(), 2);
        assert!(doc.body.iter().all(|b| b.as_table().unwrap().controls.is_empty()));
        for (i, name) in ["First", "Second"].iter().enumerate() {
            assert_eq!(doc.body[i].as_table().unwrap().rows[0].cells[0].blocks[0].as_para().unwrap().plain_text(), *name);
        }
        let mut doc = fixture();
        doc.insert_text(&Pos::body(1, 0), "Outside ", &CharProps::default()).unwrap();
        assert!(inspect(&doc).unwrap_err().to_string().contains("complete paragraphs"));
        let mut doc = fixture();
        wrap(&mut doc, 1, 3, "if:nested");
        assert!(inspect(&doc).is_ok());
        let mut doc = fixture();
        wrap(&mut doc, 4, 5, "if:nested");
        assert!(inspect(&doc).unwrap_err().to_string().contains("nested"));
    }

    #[test]
    fn repeated_distinct_pictures_do_not_alias_media() {
        let root = std::env::temp_dir().join(format!("foliorivet-stage5-media-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let mut encoded = Vec::new();
        for (name, size) in [("first.png", (20, 50)), ("second.png", (60, 30))] {
            let mut png = std::io::Cursor::new(Vec::new());
            image::DynamicImage::new_rgb8(size.0, size.1).write_to(&mut png, image::ImageFormat::Png).unwrap();
            let bytes = png.into_inner();
            std::fs::write(root.join(name), &bytes).unwrap();
            encoded.push(bytes);
        }
        let mut doc = Document::new();
        doc.body = vec![text("value:item.name", "heading"), wordcraft_doc::para_block(wordcraft_doc::Paragraph::new())];
        let c = ContentControl { tag: "image:chart".into(), kind: ControlKind::Picture, ..Default::default() };
        let at = doc.insert_object(&Pos::body(1, 0), InlineObject::ControlStart { control: Box::new(c) }, &CharProps::default()).unwrap();
        let at = doc
            .insert_object(
                &at,
                InlineObject::Image {
                    media: "placeholder.png".into(),
                    w: 100.0,
                    h: 100.0,
                    alt: "chart".into(),
                    float: Default::default(),
                    crop: [0.0; 4],
                    fx: Default::default(),
                    ole: None,
                },
                &CharProps::default(),
            )
            .unwrap();
        doc.insert_object(&at, InlineObject::ControlEnd, &CharProps::default()).unwrap();
        wrap(&mut doc, 0, 1, "repeat:classes");
        populate(
            &mut doc,
            &json!({"classes":[{"name":"A","chart":{"path":"first.png"}},{"name":"B","chart":{"path":"second.png"}}]}),
            root.to_str().unwrap(),
        )
        .unwrap();
        let mut keys = Vec::new();
        for i in [1, 3] {
            let p = doc.body[i].as_para().unwrap();
            let InlineObject::Image { media, .. } = &p.objects[0] else { panic!() };
            keys.push(media.clone());
        }
        assert_ne!(keys[0], keys[1]);
        assert_eq!(doc.media[&keys[0]].as_ref(), &encoded[0]);
        assert_eq!(doc.media[&keys[1]].as_ref(), &encoded[1]);
        let back = wordcraft_docx::read(&wordcraft_docx::write(&doc).unwrap()).unwrap();
        assert!(back.control_ranges(wordcraft_doc::StoryRef::Body).is_empty());
        for name in ["first.png", "second.png"] {
            std::fs::remove_file(root.join(name)).unwrap();
        }
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    fn row_items_and_block_items_have_separate_scopes() {
        let mut doc = fixture();
        let Block::Table(t) = Arc::make_mut(&mut doc.body[2]) else { panic!() };
        t.rows[1].controls.open.push(ContentControl { tag: "rows:metrics".into(), ..Default::default() });
        t.rows[1].controls.close = 1;
        let inspection = inspect(&doc).unwrap();
        assert!(
            inspection["targets"].as_array().unwrap().iter().any(|v| v["tag"] == "value:item.score" && v["scope"] == "repeat:classes / rows:metrics")
        );
        let result = populate(
            &mut doc,
            &json!({"title":"Report","has_evaluation":false,"classes":[{"name":"A","metrics":[{"score":1},{"score":2}]},{"name":"B","metrics":[]}]}),
            "",
        )
        .unwrap();
        assert_eq!(result["row_count"], 2);
        assert_eq!(doc.body[2].as_table().unwrap().rows.len(), 3);
        assert_eq!(doc.body[5].as_table().unwrap().rows.len(), 1);
        assert_eq!(doc.body[2].as_table().unwrap().rows[2].cells[0].blocks[0].as_para().unwrap().plain_text(), "2");
    }
    #[test]
    fn analysis_distinguishes_inactive_and_undetermined_children() {
        let mut doc = Document::new();
        doc.body = vec![text("value:item.name", "name"), text("value:detail", "detail")];
        wrap(&mut doc, 0, 0, "repeat:items");
        wrap(&mut doc, 0, 1, "if:show");
        let options = super::super::TemplateOptions::default();
        let result = super::super::analyze_context(&doc, &json!({"show":false}), options).unwrap();
        assert_eq!(result["missing_paths"], json!([]));
        assert!(result["inactive_paths"].as_array().unwrap().contains(&json!("items")));
        let result = super::super::analyze_context(&doc, &json!({"items":[{"name":"A","extra":1}]}), options).unwrap();
        assert_eq!(result["missing_paths"], json!(["show"]));
        assert_eq!(result["inactive_paths"], json!([]));
        assert!(!result["active_required_paths"].as_array().unwrap().contains(&json!("items[0].name")));
        let result = super::super::analyze_context(&doc, &json!({"show":true,"detail":"x","items":[{"name":"A","extra":1}]}), options).unwrap();
        assert_eq!(result["unused_supplied_paths"], json!(["items[0].extra"]));
        assert!(result["active_required_paths"].as_array().unwrap().contains(&json!("items[0].name")));
    }

    #[test]
    fn fanout_stays_inside_each_repeat_item() {
        let mut doc = Document::new();
        doc.body = vec![text("value:item.name", "first"), text("value:item.name", "second")];
        wrap(&mut doc, 0, 1, "repeat:items");
        let options = super::super::TemplateOptions { scalar_fanout: true };
        let context = json!({"items":[{"name":"Alpha"},{"name":"Beta"}]});
        assert!(preflight(&doc, &context, "").is_err());
        let counts = populate_with_options(&mut doc, &context, "", options).unwrap();
        assert_eq!(counts["scalar_count"], 4);
        let contents: Vec<_> = doc.body.iter().map(|b| b.as_para().unwrap().text.clone()).collect();
        assert_eq!(contents, vec!["Alpha", "Alpha", "Beta", "Beta"]);
    }

    #[test]
    fn inherited_bullets_keep_indentation_and_invalid_numbering_fails_even_inactive() {
        let mut doc = Document::new();
        let num = Arc::make_mut(&mut doc.numbering).add_list(wordcraft_doc::numbering::ListKind::Bullet);
        let mut style = wordcraft_doc::styles::Style { id: "BulletBase".into(), name: "Bullet Base".into(), ..Default::default() };
        style.para.numbering = Some(wordcraft_doc::props::NumRef { num, level: 1 });
        Arc::make_mut(&mut doc.styles).styles.push(style);
        Arc::make_mut(&mut doc.styles).styles.push(wordcraft_doc::styles::Style {
            id: "BulletChild".into(),
            name: "Bullet Child".into(),
            based_on: Some("BulletBase".into()),
            ..Default::default()
        });
        let mut p = wordcraft_doc::Paragraph::with_text("Inherited bullet", Default::default());
        p.props.style = Some("BulletChild".into());
        p.props.indent_left = Some(36.0);
        p.props.indent_first = Some(-18.0);
        doc.body = vec![wordcraft_doc::para_block(p), wordcraft_doc::para_block(wordcraft_doc::Paragraph::new())];
        wrap(&mut doc, 0, 0, "if:show");
        let mut copy = doc.clone();
        populate(&mut copy, &json!({"show":true}), "").unwrap();
        let back = wordcraft_docx::read(&wordcraft_docx::write(&copy).unwrap()).unwrap();
        let p = back.body[0].as_para().unwrap();
        assert_eq!(p.props.indent_left, Some(36.0));
        assert_eq!(p.props.indent_first, Some(-18.0));
        assert_eq!(back.styles.resolve_para(&p.props).numbering.unwrap().level, 1);
        for n in [wordcraft_doc::props::NumRef { num: 999999, level: 0 }, wordcraft_doc::props::NumRef { num, level: 9 }] {
            let mut bad = doc.clone();
            if let Block::Para(p) = Arc::make_mut(&mut bad.body[0]) {
                p.props.numbering = Some(n);
            }
            assert!(preflight(&bad, &json!({"show":false}), "").unwrap_err().to_string().contains("numbering"));
        }
    }
    #[test]
    fn conditional_sibling_repeats_share_limits_and_active_paths_win() {
        let mut doc = Document::new();
        doc.body =
            vec![text("value:name", "global"), text("value:name", "hidden"), text("value:item.name", "first"), text("value:item.name", "second")];
        wrap(&mut doc, 1, 1, "if:hide");
        wrap(&mut doc, 2, 2, "repeat:first");
        wrap(&mut doc, 3, 3, "repeat:second");
        // Child repeats may be siblings inside one outer conditional.
        doc.body.insert(2, wordcraft_doc::para_block(wordcraft_doc::Paragraph::with_text("Heading", Default::default())));
        wrap(&mut doc, 2, 4, "if:show");
        let options = super::super::TemplateOptions::default();
        let make = |n| json!({"hide":false,"show":true,"name":"global","first":vec![json!({"name":"A"});n],"second":vec![json!({"name":"B"});n]});
        let result = super::super::analyze_context(&doc, &make(1), options).unwrap();
        assert!(!result["inactive_paths"].as_array().unwrap().contains(&json!("name")));
        assert_eq!(preflight(&doc, &make(500), "").unwrap()["repeated_items"], 1000);
        let err = preflight(&doc, &make(600), "").unwrap_err().to_string();
        assert!(err.contains("used=1200") && err.contains("if:show"));
        // Invalid item targets must fail structure checks even behind a false condition.
        let mut bad = Document::new();
        bad.body = vec![text("value:global", "bad")];
        wrap(&mut bad, 0, 0, "repeat:items");
        bad.body.insert(0, wordcraft_doc::para_block(wordcraft_doc::Paragraph::new()));
        wrap(&mut bad, 0, 1, "if:show");
        assert!(preflight(&bad, &json!({"show":false}), "").is_err());
    }
    #[test]
    fn analysis_detects_active_conditional_scalar_ambiguity() {
        let mut doc = Document::new();
        doc.body = vec![text("value:name", "first"), text("value:name", "second")];
        wrap(&mut doc, 0, 0, "if:first");
        wrap(&mut doc, 1, 1, "if:second");
        for fanout in [false, true] {
            let options = super::super::TemplateOptions { scalar_fanout: fanout };
            for (first, second) in [(true, true), (true, false), (false, true)] {
                let context = json!({"first":first,"second":second,"name":"Example"});
                let result = super::super::analyze_context(&doc, &context, options).unwrap();
                let diagnostics = result["diagnostics"].as_array().unwrap();
                let ambiguous = first && second && !fanout;
                assert_eq!(!diagnostics.is_empty(), ambiguous);
                assert_eq!(preflight_with_options(&doc, &context, "", options).is_err(), ambiguous);
                if ambiguous {
                    assert_eq!(diagnostics[0]["target"], "name");
                    assert!(diagnostics[0]["scope"].as_str().unwrap().contains("if:second"));
                    assert!(diagnostics[0]["message"].as_str().unwrap().contains("if:first"));
                }
            }
        }
    }

    #[test]
    fn analysis_does_not_share_a_consumed_condition_with_a_scalar() {
        let mut doc = Document::new();
        doc.body = vec![text("value:show", "flag"), wordcraft_doc::para_block(wordcraft_doc::Paragraph::new())];
        wrap(&mut doc, 1, 1, "if:show");
        for fanout in [false, true] {
            let options = super::super::TemplateOptions { scalar_fanout: fanout };
            let context = json!({"show":true});
            let result = super::super::analyze_context(&doc, &context, options).unwrap();
            assert_eq!(result["diagnostics"][0]["target"], "show");
            assert!(result["diagnostics"][0]["message"].as_str().unwrap().contains("ambiguous active context consumer"));
            assert!(preflight_with_options(&doc, &context, "", options).is_err());
        }
    }
    #[test]
    fn independent_and_conditional_repeats_honor_exact_shared_byte_boundary() {
        let options = super::super::TemplateOptions::default();
        let limit = 16 * 1024 * 1024;
        for conditional in [false, true] {
            let mut doc = Document::new();
            doc.body = vec![text("value:item.name", "first"), text("value:item.name", "second")];
            wrap(&mut doc, 0, 0, "repeat:first");
            wrap(&mut doc, 1, 1, "repeat:second");
            let mut context = json!({"first":[{"name":"Alpha"}],"second":[{"name":"Beta"}]});
            if conditional {
                doc.body.insert(0, wordcraft_doc::para_block(wordcraft_doc::Paragraph::with_text("Heading", Default::default())));
                wrap(&mut doc, 0, 2, "if:show");
                context["show"] = json!(true);
            }
            let mut measured = super::super::Budget::default();
            prepare(&doc, &context, "", options, &mut measured).unwrap();
            assert!(measured.bytes > 0);
            for extra in [0, 1] {
                let mut budget = super::super::Budget::default();
                budget.charge(0, 0, limit - measured.bytes + extra, "earlier operations").unwrap();
                let result = prepare(&doc, &context, "", options, &mut budget);
                assert_eq!(budget.bytes, limit + extra);
                if extra == 0 {
                    assert!(result.is_ok());
                } else {
                    let message = result.unwrap_err().to_string();
                    assert!(message.contains("expansion byte budget") && message.contains("used=16777217") && message.contains("limit=16777216"));
                    if conditional {
                        assert!(message.contains("if:show"));
                    }
                }
            }
            if conditional {
                let mut budget = super::super::Budget::default();
                budget.charge(0, 0, limit, "earlier operations").unwrap();
                prepare(&doc, &json!({"show":false}), "", options, &mut budget).unwrap();
                assert_eq!(budget.bytes, limit);
                assert_eq!(budget.items, 0);
            }
        }
    }
}
