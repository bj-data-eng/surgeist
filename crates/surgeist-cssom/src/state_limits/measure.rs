//! What each kind of retained record contributes to the census, measured
//! without the limits.
//!
//! One function per kind of record turns it into [`Counts`]: the entries and
//! host string bytes it accounts for, the base URL identity it holds and what
//! the checks against the limits need to know. A record is destructured
//! completely, so a field added to it fails to compile here until it is
//! accounted for or named as accounting for nothing. [`Counts::check`] raises
//! the refusals a record causes under a set of limits, naming the entries the
//! record had counted before it was refused.
//!
//! The census of a state is the sum of these counts followed by the threshold
//! checks, in that order.
use crate::named_supports_operations::{TestStorage, measure_test_child, test_input_bytes};
use crate::{model::State, *};
use std::sync::Arc;
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::rule::keyframes::CssKeyframesName;

/// The identity and bytes of the base URL a sheet holds. The bytes are counted
/// once for every identity, however many sheets share it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Base {
    /// The address of the shared base URL units.
    pub(crate) identity: usize,
    /// Their bytes, or `None` when the sum overflows.
    pub(crate) bytes: Option<usize>,
}

/// A record refused by a check, and the entries it had counted before.
#[derive(Debug)]
pub(crate) struct Refusal {
    pub(crate) error: CssomError,
    pub(crate) counted: usize,
}

/// What one record accounts for, and what the checks of its limits need.
#[derive(Clone, Debug, Default)]
pub(crate) struct Counts {
    /// The entries the record accounts for.
    pub(crate) entries: usize,
    /// The host string bytes the record accounts for, the base URL aside.
    pub(crate) strings: usize,
    /// The base URL of a sheet.
    pub(crate) base: Option<Base>,
    check: Check,
}

/// The facts a record's checks read, which depend on the record alone.
#[derive(Clone, Debug, Default)]
enum Check {
    #[default]
    Nothing,
    Sheet {
        /// The bytes of the exact parse source of the sheet.
        parse_input: Option<usize>,
        /// The entries counted before the title and the base URL.
        entries_before_title: usize,
        /// Whether the bytes of the title overflow.
        title_overflows: bool,
    },
    Rule {
        /// The entries of the admission diagnostics, counted first.
        diagnostics: usize,
        /// The bytes of the latest selector input.
        selector_input: Option<usize>,
        payload: Payload,
    },
}

/// The test payload a rule retains.
#[derive(Clone, Debug, Default)]
enum Payload {
    #[default]
    Nothing,
    /// A named-supports body: the bytes of the name's origin, the entries of
    /// the recovery origins and the storage of each child, counted in turn.
    Named {
        name_bytes: usize,
        recovery: usize,
        children: Vec<TestStorage>,
    },
    /// An inert test child.
    Child(TestStorage),
}

impl Counts {
    fn add_entries(&mut self, count: usize) {
        self.entries = self.entries.saturating_add(count);
    }
    fn add_strings(&mut self, count: usize) {
        self.strings = self.strings.saturating_add(count);
    }
    fn version(&mut self, value: &CssomInputVersion) {
        self.add_strings(value.identity.len());
    }

    /// The entries a sheet had counted when its base URL was reached.
    pub(crate) fn entries_before_base(&self) -> usize {
        match &self.check {
            Check::Sheet {
                entries_before_title,
                ..
            } => *entries_before_title,
            Check::Nothing | Check::Rule { .. } => 0,
        }
    }

    /// Refuses the record when it exceeds `limits`: an oversized sheet or
    /// rule input, a title or test payload that does not fit.
    pub(crate) fn check(&self, limits: &CssomLimits) -> Result<(), Refusal> {
        match &self.check {
            Check::Nothing => Ok(()),
            Check::Sheet {
                parse_input,
                entries_before_title,
                title_overflows,
            } => {
                if parse_input.is_some_and(|bytes| bytes > limits.max_input_bytes) {
                    return Err(Refusal {
                        error: CssomError::Limit {
                            resource: "parse input bytes",
                            maximum: limits.max_input_bytes,
                        },
                        counted: 0,
                    });
                }
                if *title_overflows {
                    return Err(Refusal {
                        error: CssomError::Limit {
                            resource: "host and live string bytes",
                            maximum: limits.max_string_bytes,
                        },
                        counted: *entries_before_title,
                    });
                }
                Ok(())
            }
            Check::Rule {
                diagnostics,
                selector_input,
                payload,
            } => {
                if selector_input.is_some_and(|bytes| bytes > limits.max_input_bytes) {
                    return Err(Refusal {
                        error: CssomError::Limit {
                            resource: "retained selector input bytes",
                            maximum: limits.max_input_bytes,
                        },
                        counted: *diagnostics,
                    });
                }
                let mut counted = *diagnostics;
                match payload {
                    Payload::Nothing => {}
                    Payload::Named {
                        name_bytes,
                        recovery,
                        children,
                    } => {
                        if *name_bytes > limits.max_input_bytes {
                            return Err(Refusal {
                                error: CssomError::Limit {
                                    resource: "retained test input bytes",
                                    maximum: limits.max_input_bytes,
                                },
                                counted,
                            });
                        }
                        counted = counted.saturating_add(*recovery);
                        for child in children {
                            let (entries, _) = child
                                .check(limits)
                                .map_err(|error| Refusal { error, counted })?;
                            counted = counted.saturating_add(entries);
                        }
                    }
                    Payload::Child(storage) => {
                        storage.check(limits).map_err(|error| Refusal {
                            error,
                            counted: *diagnostics,
                        })?;
                    }
                }
                Ok(())
            }
        }
    }
}

/// The entries and string bytes of one input of a context.
fn input(counts: &mut Counts, input: &CssomInput) {
    let CssomInput { version, data } = input;
    counts.version(version);
    counts.add_entries(1);
    match data {
        CssomInputData::Document { identity } => counts.add_strings(identity.len()),
        CssomInputData::Window { identity, document } => {
            counts.add_strings(identity.len());
            counts.add_strings(document.len());
        }
        CssomInputData::Origin { origin, profile } => {
            counts.add_strings(origin.len());
            counts.add_strings(profile.len());
        }
        CssomInputData::Layer { path, ordinal: _ } => {
            counts.add_entries(path.len());
            for part in path {
                counts.add_strings(part.len());
            }
        }
        CssomInputData::Base { url, location } => {
            for value in [url, location].into_iter().flatten() {
                counts.add_strings(value.len());
            }
        }
        CssomInputData::Owner {
            identity,
            attribute,
        } => {
            counts.add_strings(identity.len());
            if let Some(v) = attribute {
                counts.add_strings(v.len());
            }
        }
        CssomInputData::Import { resolved_location } => {
            if let Some(v) = resolved_location {
                counts.add_strings(v.len());
            }
        }
        CssomInputData::ParserMode(_) => {}
        CssomInputData::Support(support) => counts.add_entries(
            support.properties().len()
                + support.page_descriptors().len()
                + support.font_face_descriptors().len()
                + usize::from(support.svg_glyph_orientation_vertical().is_some()),
        ),
    }
}

/// The inputs and linked inputs of one context, refused when a linked sheet
/// is missing from the snapshot it names.
pub(crate) fn context(context: &CssomContext) -> Result<Counts, Refusal> {
    let CssomContext { inputs, linked } = context;
    let mut counts = Counts::default();
    for value in inputs {
        input(&mut counts, value);
    }
    for value in linked {
        let CssomLinkedInput {
            version,
            snapshot,
            sheet,
        } = value;
        counts.version(version);
        snapshot.sheet(sheet).map_err(|error| Refusal {
            error,
            counted: counts.entries,
        })?;
        counts.add_entries(1);
    }
    Ok(counts)
}

/// One sheet record.
pub(crate) fn sheet(sheet: &CssomSheet) -> Counts {
    let CssomSheet {
        inputs,
        rules: _,
        media: _,
        owner_rule: _,
        namespaces: _,
        authored,
        diagnostics,
    } = sheet;
    let CssomSheetInputs {
        version,
        base_url,
        location,
        constructor_document,
        owner,
        title,
        alternate: _,
        disabled: _,
        origin_clean: _,
        constructed: _,
        disallow_modification: _,
        media: _,
    } = inputs;
    let mut counts = Counts::default();
    // Retained raw input includes imports filtered from constructed sheets.
    let rules = authored.rules().len();
    counts.add_entries(rules);
    counts.version(version);
    let title_bytes = title.capacity().checked_mul(2);
    counts.add_strings(title_bytes.unwrap_or(0));
    counts.base = base_url.as_ref().map(|base| Base {
        identity: Arc::as_ptr(base).addr(),
        bytes: base.capacity().checked_mul(size_of::<u16>()),
    });
    for text in [location, constructor_document, owner]
        .into_iter()
        .flatten()
    {
        counts.add_strings(text.len());
    }
    counts.add_entries(diagnostics.len());
    counts.check = Check::Sheet {
        parse_input: authored
            .parse_source()
            .map(|input| input.source().as_str().len()),
        entries_before_title: rules,
        title_overflows: title_bytes.is_none(),
    };
    counts
}

/// One rule list holding `members` rule identities.
pub(crate) fn list(members: usize) -> Counts {
    Counts {
        entries: members,
        ..Counts::default()
    }
}

/// One media list.
pub(crate) fn media(media: &CssMediaQueryList) -> Counts {
    Counts {
        entries: media.queries().len(),
        ..Counts::default()
    }
}

/// One block record.
pub(crate) fn block(block: &CssomBlock) -> Counts {
    let CssomBlock {
        parent: _,
        binding,
        flags: _,
        data,
        admission_inputs,
        counter_edits,
        replacement_input,
        selected_font_face,
    } = block;
    let mut counts = Counts::default();
    counts.add_entries(counter_edits.len());
    for value in counter_edits.iter() {
        if let Some(origin) = value.origin() {
            counts.add_strings(origin.source().as_str().len());
        }
    }
    if let Some(CssomProjection::Available(selected)) = selected_font_face.as_deref() {
        counts.add_entries(selected.entries().len());
    }
    if let Some(input) = replacement_input {
        counts.add_strings(input.source().len());
        counts.add_entries(input.diagnostics().len());
    }
    counts.add_entries(admission_inputs.len());
    for input in admission_inputs.iter() {
        counts.version(input);
    }
    match binding {
        crate::inline_context::BlockBinding::Store(Some(owner)) => counts.version(owner),
        crate::inline_context::BlockBinding::Store(None)
        | crate::inline_context::BlockBinding::Inline(_) => {}
    }
    match &**data {
        CssomBlockData::Properties {
            domain: _,
            authored,
            selected,
        } => {
            counts.add_entries(match authored {
                CssomPropertyOccurrences::Ordinary(v)
                | CssomPropertyOccurrences::RawContents(v) => v.len(),
                CssomPropertyOccurrences::Keyframe(v) => v.len(),
                CssomPropertyOccurrences::Margin(v) => v.properties().len(),
            });
            if let CssomProjection::Available(v) = selected {
                counts.add_entries(v.entries().len());
            }
        }
        CssomBlockData::Page { authored, selected } => {
            counts.add_entries(authored.occurrences().len());
            if let CssomProjection::Available(v) = selected {
                counts.add_entries(v.entries().len());
            }
        }
        CssomBlockData::CounterStyle(v) => counts.add_entries(v.occurrences().len()),
        CssomBlockData::FontFace(v) => counts.add_entries(v.occurrences().len()),
    }
    counts
}

/// One feature map.
pub(crate) fn map(map: &CssomFeatureMap) -> Counts {
    let CssomFeatureMap {
        kind: _,
        entries,
        authored,
    } = map;
    let mut counts = Counts::default();
    counts.add_entries(authored.len());
    counts.add_entries(entries.len());
    for entry in entries {
        let CssomFeatureEntry { key, value } = entry;
        counts.add_strings(key.len());
        match value {
            CssomFeatureValue::Unsigned(v) => counts.add_entries(v.len()),
            CssomFeatureValue::Authored(_) => {}
        }
    }
    counts
}

/// One rule record.
pub(crate) fn rule(rule: &CssomRule) -> Counts {
    let CssomRule {
        parent: _,
        data,
        authored: _,
        insertion_origin: _,
        selector_input,
        selector_inputs: _,
        selector_namespaces: _,
        admission_diagnostics,
    } = rule;
    let mut counts = Counts::default();
    counts.add_entries(admission_diagnostics.len());
    let selector_input = selector_input
        .as_ref()
        .map(|origin| origin.source().as_str().len());
    counts.add_strings(selector_input.unwrap_or(0));
    let payload = match &**data {
        CssomRuleData::NamedSupports {
            name,
            body,
            children: _,
        } => {
            counts.add_strings(name.as_str().len());
            let name_bytes = test_input_bytes(name.origin());
            counts.add_strings(name_bytes);
            let recovery = body.local_recovery_origins().len();
            counts.add_entries(recovery);
            let mut children = Vec::new();
            for child in body.children() {
                let storage = measure_test_child(child);
                counts.add_entries(storage.entries());
                counts.add_strings(storage.bytes());
                children.push(storage);
            }
            Payload::Named {
                name_bytes,
                recovery,
                children,
            }
        }
        CssomRuleData::SupportsTestChild(child) => {
            let storage = measure_test_child(child.as_ref());
            // Current and historical checked payloads are retained separately.
            counts.add_entries(storage.entries());
            counts.add_strings(storage.bytes());
            counts.add_entries(storage.entries());
            counts.add_strings(storage.bytes());
            Payload::Child(storage)
        }
        CssomRuleData::CounterStyle { name, block: _ } => {
            counts.add_strings(name.len());
            Payload::Nothing
        }
        CssomRuleData::CustomMedia { name, query: _ } => {
            counts.add_strings(name.as_str().len());
            Payload::Nothing
        }
        CssomRuleData::FontFeatureValues {
            families,
            family_input,
            maps: _,
            font_display: _,
        } => {
            counts.add_entries(families.len());
            for family in families {
                counts.add_strings(family.as_str().len());
            }
            if let Some(input) = family_input {
                counts.add_strings(input.source_bytes());
                counts.add_entries(input.diagnostics().len());
                counts.add_entries(input.inputs().len());
                for input in input.inputs() {
                    counts.version(input);
                }
            }
            Payload::Nothing
        }
        CssomRuleData::Keyframes { name, children: _ } => {
            counts.add_strings(match name {
                CssKeyframesName::Ident(value) => value.as_str().len(),
                CssKeyframesName::String(value) => value.as_str().len(),
                _ => 0,
            });
            Payload::Nothing
        }
        CssomRuleData::Import {
            target: _,
            layer: _,
            supports: _,
            origin: _,
            media: _,
            child_sheet: _,
            resolved_location,
            input,
        } => {
            if let Some(v) = resolved_location {
                counts.add_strings(v.len());
            }
            if let Some(v) = input {
                counts.version(v);
            }
            Payload::Nothing
        }
        CssomRuleData::Style { .. }
        | CssomRuleData::Group { .. }
        | CssomRuleData::NestedDeclarations { .. }
        | CssomRuleData::Page { .. }
        | CssomRuleData::Margin { .. }
        | CssomRuleData::Keyframe { .. }
        | CssomRuleData::FontFace { .. }
        | CssomRuleData::Leaf { .. } => Payload::Nothing,
    };
    counts.check = Check::Rule {
        diagnostics: admission_diagnostics.len(),
        selector_input,
        payload,
    };
    counts
}

/// Every context the census accounts for: the state's, those the selector
/// inputs of its rules hold and those its inline declaration blocks hold.
pub(crate) fn contexts(state: &State) -> impl Iterator<Item = &CssomContext> {
    std::iter::once(state.context.as_ref())
        .chain(
            state
                .rules
                .values()
                .filter_map(|rule| rule.selector_inputs.as_deref()),
        )
        .chain(state.inline_contexts())
}
