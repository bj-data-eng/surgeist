//! Synchronous admission accounting for complete retained CSSOM state.
use crate::{model::State, *};

impl State {
    pub(crate) fn check_limits(&self, limits: &CssomLimits) -> Result<(), CssomError> {
        for context in std::iter::once(self.context.as_ref()).chain(self.inline_contexts()) {
            context.validate()?;
        }
        let mut linked = std::iter::once(self.context.as_ref())
            .chain(self.inline_contexts())
            .flat_map(|context| &context.linked)
            .map(|v| (&v.snapshot, 1usize))
            .collect::<Vec<_>>();
        let mut visited = 0usize;
        while let Some((snapshot, depth)) = linked.pop() {
            if depth > limits.max_depth {
                return Err(CssomError::Limit {
                    resource: "linked input depth",
                    maximum: limits.max_depth,
                });
            }
            visited = visited.checked_add(1).ok_or(CssomError::Limit {
                resource: "linked input visits",
                maximum: limits.max_entries,
            })?;
            if visited > limits.max_entries {
                return Err(CssomError::Limit {
                    resource: "linked input visits",
                    maximum: limits.max_entries,
                });
            }
            linked.extend(
                std::iter::once(snapshot.context())
                    .chain(snapshot.state.inline_contexts())
                    .flat_map(|context| &context.linked)
                    .map(|v| (&v.snapshot, depth + 1)),
            );
        }
        let objects = 1usize
            .checked_add(self.sheets.len())
            .and_then(|v| v.checked_add(self.rules.len()))
            .and_then(|v| v.checked_add(self.blocks.len()))
            .and_then(|v| v.checked_add(self.lists.len()))
            .and_then(|v| v.checked_add(self.media.len()))
            .and_then(|v| v.checked_add(self.maps.len()));
        limit(objects, limits.max_objects, "objects")?;
        let mut entries = Some(self.sheet_order.len());
        let mut strings = Some(0usize);
        fn add(total: &mut Option<usize>, count: usize) {
            *total = total.and_then(|v| v.checked_add(count));
        }
        fn version(total: &mut Option<usize>, value: &CssomInputVersion) {
            add(total, value.identity.len());
        }
        for input in std::iter::once(self.context.as_ref())
            .chain(
                self.rules
                    .values()
                    .filter_map(|rule| rule.selector_inputs.as_deref()),
            )
            .chain(self.inline_contexts())
            .flat_map(|context| &context.inputs)
        {
            version(&mut strings, &input.version);
            add(&mut entries, 1);
            match &input.data {
                CssomInputData::Document { identity } => add(&mut strings, identity.len()),
                CssomInputData::Window { identity, document } => {
                    add(&mut strings, identity.len());
                    add(&mut strings, document.len());
                }
                CssomInputData::Origin { origin, profile } => {
                    add(&mut strings, origin.len());
                    add(&mut strings, profile.len());
                }
                CssomInputData::Layer { path, .. } => {
                    add(&mut entries, path.len());
                    for part in path {
                        add(&mut strings, part.len());
                    }
                }
                CssomInputData::Base { url, location } => {
                    for value in [url, location].into_iter().flatten() {
                        add(&mut strings, value.len());
                    }
                }
                CssomInputData::Owner {
                    identity,
                    attribute,
                } => {
                    add(&mut strings, identity.len());
                    if let Some(v) = attribute {
                        add(&mut strings, v.len());
                    }
                }
                CssomInputData::Import { resolved_location } => {
                    if let Some(v) = resolved_location {
                        add(&mut strings, v.len());
                    }
                }
                CssomInputData::ParserMode(_) => {}
                CssomInputData::Support(support) => add(
                    &mut entries,
                    support.properties().len()
                        + support.page_descriptors().len()
                        + support.font_face_descriptors().len()
                        + usize::from(support.svg_glyph_orientation_vertical().is_some()),
                ),
            }
        }
        for linked in std::iter::once(self.context.as_ref())
            .chain(
                self.rules
                    .values()
                    .filter_map(|rule| rule.selector_inputs.as_deref()),
            )
            .chain(self.inline_contexts())
            .flat_map(|context| &context.linked)
        {
            version(&mut strings, &linked.version);
            linked.snapshot.sheet(&linked.sheet)?;
            add(&mut entries, 1);
        }
        for sheet in self.sheets.values() {
            if let Some(input) = sheet.authored.parse_source()
                && input.source().as_str().len() > limits.max_input_bytes
            {
                return Err(CssomError::Limit {
                    resource: "parse input bytes",
                    maximum: limits.max_input_bytes,
                });
            }
            // Retained raw input includes imports filtered from constructed sheets.
            add(&mut entries, sheet.authored.rules().len());
            version(&mut strings, &sheet.inputs.version);
            let title_bytes =
                sheet
                    .inputs
                    .title
                    .capacity()
                    .checked_mul(2)
                    .ok_or(CssomError::Limit {
                        resource: "host and live string bytes",
                        maximum: limits.max_string_bytes,
                    })?;
            add(&mut strings, title_bytes);
            for text in [
                &sheet.inputs.base_url,
                &sheet.inputs.location,
                &sheet.inputs.constructor_document,
                &sheet.inputs.owner,
            ]
            .into_iter()
            .flatten()
            {
                add(&mut strings, text.len());
            }
            add(&mut entries, sheet.diagnostics.len());
        }
        for list in self.lists.values() {
            add(&mut entries, list.len());
        }
        for media in self.media.values() {
            add(&mut entries, media.queries().len());
        }
        for block in self.blocks.values() {
            add(&mut entries, block.counter_edits.len());
            for value in block.counter_edits.iter() {
                if let Some(origin) = value.origin() {
                    add(&mut strings, origin.source().as_str().len());
                }
            }
            if let Some(CssomProjection::Available(selected)) = block.selected_font_face.as_deref()
            {
                add(&mut entries, selected.entries().len());
            }
            if let Some(input) = &block.replacement_input {
                add(&mut strings, input.source().len());
                add(&mut entries, input.diagnostics().len());
            }
            add(&mut entries, block.admission_inputs.len());
            for input in block.admission_inputs.iter() {
                version(&mut strings, input);
            }
            if block.inline_context().is_none()
                && let Some(owner) = block.owner()
            {
                version(&mut strings, owner);
            }
            match block.data() {
                CssomBlockData::Properties {
                    authored, selected, ..
                } => {
                    add(
                        &mut entries,
                        match authored {
                            CssomPropertyOccurrences::Ordinary(v)
                            | CssomPropertyOccurrences::RawContents(v) => v.len(),
                            CssomPropertyOccurrences::Keyframe(v) => v.len(),
                            CssomPropertyOccurrences::Margin(v) => v.properties().len(),
                        },
                    );
                    if let CssomProjection::Available(v) = selected {
                        add(&mut entries, v.entries().len());
                    }
                }
                CssomBlockData::Page { authored, selected } => {
                    add(&mut entries, authored.occurrences().len());
                    if let CssomProjection::Available(v) = selected {
                        add(&mut entries, v.entries().len());
                    }
                }
                CssomBlockData::CounterStyle(v) => add(&mut entries, v.occurrences().len()),
                CssomBlockData::FontFace(v) => add(&mut entries, v.occurrences().len()),
            }
        }
        for map in self.maps.values() {
            add(&mut entries, map.authored.len());
            add(&mut entries, map.entries.len());
            for entry in &map.entries {
                add(&mut strings, entry.key.len());
                if let CssomFeatureValue::Unsigned(v) = &entry.value {
                    add(&mut entries, v.len());
                }
            }
        }
        for rule in self.rules.values() {
            add(&mut entries, rule.admission_diagnostics.len());
            if let Some(origin) = &rule.selector_input {
                if origin.source().as_str().len() > limits.max_input_bytes {
                    return Err(CssomError::Limit {
                        resource: "retained selector input bytes",
                        maximum: limits.max_input_bytes,
                    });
                }
                add(&mut strings, origin.source().as_str().len());
            }
            match rule.data() {
                CssomRuleData::NamedSupports { name, body, .. } => {
                    add(&mut strings, name.as_str().len());
                    add(
                        &mut strings,
                        crate::named_supports_operations::retained_test_input_bytes(
                            name.origin(),
                            limits,
                        )?,
                    );
                    add(&mut entries, body.local_recovery_origins().len());
                    for child in body.children() {
                        let (count, bytes) =
                            crate::named_supports_operations::test_child_storage(child, limits)?;
                        add(&mut entries, count);
                        add(&mut strings, bytes);
                    }
                }
                CssomRuleData::SupportsTestChild(child) => {
                    let (count, bytes) = crate::named_supports_operations::test_child_storage(
                        child.as_ref(),
                        limits,
                    )?;
                    add(&mut entries, count);
                    add(&mut strings, bytes);
                    // Current and historical checked payloads are retained separately.
                    add(&mut entries, count);
                    add(&mut strings, bytes);
                }
                CssomRuleData::CounterStyle { name, .. } => add(&mut strings, name.len()),
                CssomRuleData::CustomMedia { name, .. } => add(&mut strings, name.as_str().len()),
                CssomRuleData::FontFeatureValues {
                    families,
                    family_input,
                    ..
                } => {
                    add(&mut entries, families.len());
                    for family in families {
                        add(&mut strings, family.as_str().len());
                    }
                    if let Some(input) = family_input {
                        add(&mut strings, input.source_bytes());
                        add(&mut entries, input.diagnostics().len());
                        add(&mut entries, input.inputs().len());
                        for input in input.inputs() {
                            version(&mut strings, input);
                        }
                    }
                }
                CssomRuleData::Keyframes { name, .. } => add(
                    &mut strings,
                    match name {
                        surgeist_css::rule::keyframes::CssKeyframesName::Ident(value) => {
                            value.as_str().len()
                        }
                        surgeist_css::rule::keyframes::CssKeyframesName::String(value) => {
                            value.as_str().len()
                        }
                        _ => 0,
                    },
                ),
                CssomRuleData::Import {
                    resolved_location,
                    input,
                    ..
                } => {
                    if let Some(v) = resolved_location {
                        add(&mut strings, v.len());
                    }
                    if let Some(v) = input {
                        version(&mut strings, v);
                    }
                }
                _ => {}
            }
        }
        limit(entries, limits.max_entries, "entries")?;
        self.check_source_limits(limits)?;
        limit(
            strings,
            limits.max_string_bytes,
            "host and live string bytes",
        )
    }
}
fn limit(actual: Option<usize>, maximum: usize, resource: &'static str) -> Result<(), CssomError> {
    if actual.is_none_or(|v| v > maximum) {
        Err(CssomError::Limit { resource, maximum })
    } else {
        Ok(())
    }
}
