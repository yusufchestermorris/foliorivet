//! Context findings without population, asset access or output writes.
use std::collections::{BTreeMap, BTreeSet};

use super::{CmdResult, Document, TemplateOptions, Value, error, json, stage4, stage5};

#[derive(Clone, Copy, PartialEq)]
enum Activity {
    Active,
    Inactive,
    Undetermined,
}

#[derive(Default)]
struct Findings {
    required: BTreeSet<String>,
    inactive: BTreeSet<String>,
    undetermined: BTreeSet<String>,
    missing: BTreeSet<String>,
    consumed: BTreeSet<String>,
    diagnostics: Vec<Value>,
    consumers: BTreeMap<String, (String, String)>,
}

fn lookup<'a>(context: Option<&'a Value>, key: &str) -> Option<&'a Value> {
    key.split('.').try_fold(context?, |v, part| v.as_object()?.get(part))
}

fn path(prefix: &str, key: &str) -> String {
    if prefix.is_empty() { key.into() } else { format!("{prefix}.{key}") }
}

impl Findings {
    fn diagnostic(&mut self, scope: &str, target: &str, message: impl std::fmt::Display) {
        self.diagnostics.push(json!({"scope":scope,"target":target,"message":message.to_string()}));
    }

    fn consumer(&mut self, prefix: &str, key: &str, tag: &str, scope: &str, options: TemplateOptions) {
        let full = path(prefix, key);
        if let Some((first_tag, first_scope)) = self.consumers.get(&full) {
            if !(options.scalar_fanout && tag.starts_with("value:") && first_tag.starts_with("value:")) {
                let message = format!("ambiguous active context consumer: {tag} also uses the path consumed by {first_tag} in {first_scope}");
                self.diagnostic(scope, &full, message);
            }
        } else {
            self.consumers.insert(full, (tag.into(), scope.into()));
        }
    }

    fn supplied(&mut self, value: &Value, prefix: &str, paths: &mut BTreeSet<String>, depth: usize) {
        if depth > 32 {
            self.diagnostic("document", prefix, "context nesting exceeds 32 levels");
            paths.insert(prefix.into());
            return;
        }
        match value {
            Value::Object(object) => {
                for (key, value) in object {
                    let full = path(prefix, key);
                    if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        self.diagnostic("document", &full, "invalid context field name");
                    }
                    self.supplied(value, &full, paths, depth + 1);
                }
            }
            Value::Array(items) => {
                paths.insert(prefix.into());
                for (i, value) in items.iter().enumerate() {
                    let full = format!("{prefix}[{i}]");
                    if value.as_object().is_some_and(|o| o.is_empty()) {
                        paths.insert(full.clone());
                    }
                    self.supplied(value, &full, paths, depth + 1);
                }
            }
            _ => {
                paths.insert(prefix.into());
            }
        }
    }

    fn input<'a>(&mut self, context: Option<&'a Value>, key: &str, prefix: &str, activity: Activity, scope: &str) -> Option<&'a Value> {
        let full = path(prefix, key);
        let value = lookup(context, key);
        match activity {
            Activity::Active => {
                self.required.insert(full.clone());
                if value.is_some() {
                    self.consumed.insert(full);
                } else {
                    self.missing.insert(full.clone());
                    self.diagnostic(scope, &full, "missing required value");
                }
            }
            Activity::Inactive => {
                self.inactive.insert(full);
            }
            Activity::Undetermined => {
                self.undetermined.insert(full);
            }
        }
        value
    }

    #[allow(clippy::too_many_arguments)] // Scope and activity are explicit across bounded template recursion.
    fn visit(
        &mut self,
        doc: &Document,
        context: Option<&Value>,
        prefix: &str,
        scope: &str,
        activity: Activity,
        item: bool,
        options: TemplateOptions,
    ) -> Result<(), super::CmdError> {
        if stage5::needed(doc) {
            let slots = stage5::resolve(doc, options)?;
            self.visit(&stage5::masked(doc, &slots), context, prefix, scope, activity, item, options)?;
            for slot in slots {
                let key = slot.tag.split_once(':').map_or("", |(_, key)| key);
                let value = self.input(context, key, prefix, activity, scope);
                let child_scope = format!("{scope} / {}", slot.tag);
                if activity == Activity::Active {
                    self.consumer(prefix, key, &slot.tag, &child_scope, options);
                }
                if slot.repeat {
                    self.items(&slot.prototype, value, &path(prefix, key), &child_scope, activity, options)?;
                } else {
                    let child_activity = match (activity, value.and_then(Value::as_bool)) {
                        (Activity::Inactive, _) => Activity::Inactive,
                        (Activity::Undetermined, _) => Activity::Undetermined,
                        (_, Some(true)) => Activity::Active,
                        (_, Some(false)) => Activity::Inactive,
                        _ => {
                            if value.is_some() {
                                self.diagnostic(&child_scope, &slot.tag, "expected an explicit boolean; descendant activity undetermined");
                            }
                            Activity::Undetermined
                        }
                    };
                    self.visit(&slot.prototype, context, prefix, &child_scope, child_activity, item, options)?;
                }
            }
            return Ok(());
        }
        let resolved = stage4::resolve(doc, item, options)?;
        for slot in resolved.rows {
            let key = slot.tag.strip_prefix("rows:").unwrap_or("");
            let value = self.input(context, key, prefix, activity, scope);
            if activity == Activity::Active {
                self.consumer(prefix, key, &slot.tag, scope, options);
            }
            self.items(
                &stage4::row_document(doc, &slot.prototype),
                value,
                &path(prefix, key),
                &format!("{scope} / {}", slot.tag),
                activity,
                options,
            )?;
        }
        for range in resolved.images {
            let key = range.control.tag.strip_prefix("image:").unwrap_or("");
            if activity == Activity::Active {
                self.consumer(prefix, key, &range.control.tag, scope, options);
            }
            let value = self.input(context, key, prefix, activity, scope);
            if activity == Activity::Active
                && let Some(value) = value
            {
                // Image descriptor leaves belong to one consumer; assets are never opened here.
                let mut leaves = BTreeSet::new();
                self.supplied(value, &path(prefix, key), &mut leaves, 0);
                self.consumed.extend(leaves);
                if let Err(e) = stage4::validate_image_input(value, &range.control.tag) {
                    self.diagnostic(scope, &range.control.tag, e);
                }
            }
        }
        for (tag, range) in super::targets(&resolved.masked, options)? {
            super::supported(&resolved.masked, &tag, &range)?;
            let key = tag.strip_prefix("value:").unwrap_or("");
            let key = if item { key.strip_prefix("item.").unwrap_or(key) } else { key };
            if activity == Activity::Active {
                self.consumer(prefix, key, &tag, scope, options);
            }
            let value = self.input(context, key, prefix, activity, scope);
            if activity == Activity::Active
                && let Some(value) = value
                && let Err(e) = super::scalar_text(value, &range, &tag)
            {
                self.diagnostic(scope, &tag, e);
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn items(
        &mut self,
        prototype: &Document,
        value: Option<&Value>,
        prefix: &str,
        scope: &str,
        activity: Activity,
        options: TemplateOptions,
    ) -> Result<(), super::CmdError> {
        if let Some(items) = value.and_then(Value::as_array) {
            for (i, item) in items.iter().enumerate() {
                let indexed = format!("{prefix}[{i}]");
                if activity == Activity::Active && !item.is_object() {
                    self.diagnostic(&indexed, prefix, "expected an item object");
                }
                if activity == Activity::Active && item.as_object().is_some_and(|o| o.is_empty()) {
                    self.consumed.insert(indexed.clone());
                }
                self.visit(prototype, Some(item), &indexed, &format!("{scope}[{i}]"), activity, true, options)?;
            }
        } else {
            if activity == Activity::Active && value.is_some() {
                self.diagnostic(scope, prefix, "expected an array of item objects");
            }
            // Uninstantiated item fields are schema paths, never guessed missing values.
            let schema_activity = if activity == Activity::Inactive { Activity::Inactive } else { Activity::Undetermined };
            self.visit(prototype, None, &format!("{prefix}[]"), scope, schema_activity, true, options)?;
        }
        Ok(())
    }
}

/// Analyze supplied context without decoding images or writing output.
pub fn analyze_context(doc: &Document, context: &Value, options: TemplateOptions) -> CmdResult {
    if serde_json::to_vec(context).map_or(true, |v| v.len() > super::MAX_JSON) {
        return Err(error("analyze", "context", "expected a JSON object of at most 1 MiB"));
    }
    // Validate all prototypes even if their conditions are false or unavailable.
    super::inspect_with_options(doc, options)?;
    let mut supplied_paths = BTreeSet::new();
    let mut findings = Findings::default();
    if !context.is_object() {
        findings.diagnostic("document", "context", "expected a JSON object");
    }
    findings.supplied(context, "", &mut supplied_paths, 0);
    findings.visit(doc, Some(context), "", "document", Activity::Active, false, options)?;
    findings.inactive.retain(|p| !findings.required.contains(p) && !findings.undetermined.contains(p));
    let unused: Vec<_> = supplied_paths.difference(&findings.consumed).collect();
    Ok(json!({"active_required_paths":findings.required,"inactive_paths":findings.inactive,
        "missing_paths":findings.missing,"unused_supplied_paths":unused,"diagnostics":findings.diagnostics}))
}
