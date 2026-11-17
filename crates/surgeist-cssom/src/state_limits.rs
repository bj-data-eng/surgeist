//! Synchronous admission accounting for complete retained CSSOM state.
//!
//! The census adds up what the retained state holds: objects, entries, host
//! string bytes and the sources its payloads retain. Each kind of object
//! contributes through its own function, so a census over the whole state is
//! the sum of those contributions followed by the threshold checks.
use crate::{model::State, *};
use std::{collections::HashSet, sync::Arc};
use surgeist_css::query::media::CssMediaQueryList;

/// The entries and host string bytes the retained state accounts for. A sum
/// that overflows is `None` and is refused by its threshold check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Totals {
    entries: Option<usize>,
    strings: Option<usize>,
}

impl Totals {
    /// The totals of a state without objects: the sheet order is its only entry list.
    fn new(sheet_order: usize) -> Self {
        Self {
            entries: Some(sheet_order),
            strings: Some(0),
        }
    }
    fn add_entries(&mut self, count: usize) {
        self.entries = self.entries.and_then(|total| total.checked_add(count));
    }
    fn add_strings(&mut self, count: usize) {
        self.strings = self.strings.and_then(|total| total.checked_add(count));
    }
    fn version(&mut self, value: &CssomInputVersion) {
        self.add_strings(value.identity.len());
    }
    /// The inputs and linked inputs of one context.
    fn context(&mut self, context: &CssomContext) -> Result<(), CssomError> {
        for input in &context.inputs {
            self.input(input);
        }
        for linked in &context.linked {
            self.version(&linked.version);
            linked.snapshot.sheet(&linked.sheet)?;
            self.add_entries(1);
        }
        Ok(())
    }
    fn input(&mut self, input: &CssomInput) {
        self.version(&input.version);
        self.add_entries(1);
        match &input.data {
            CssomInputData::Document { identity } => self.add_strings(identity.len()),
            CssomInputData::Window { identity, document } => {
                self.add_strings(identity.len());
                self.add_strings(document.len());
            }
            CssomInputData::Origin { origin, profile } => {
                self.add_strings(origin.len());
                self.add_strings(profile.len());
            }
            CssomInputData::Layer { path, .. } => {
                self.add_entries(path.len());
                for part in path {
                    self.add_strings(part.len());
                }
            }
            CssomInputData::Base { url, location } => {
                for value in [url, location].into_iter().flatten() {
                    self.add_strings(value.len());
                }
            }
            CssomInputData::Owner {
                identity,
                attribute,
            } => {
                self.add_strings(identity.len());
                if let Some(v) = attribute {
                    self.add_strings(v.len());
                }
            }
            CssomInputData::Import { resolved_location } => {
                if let Some(v) = resolved_location {
                    self.add_strings(v.len());
                }
            }
            CssomInputData::ParserMode(_) => {}
            CssomInputData::Support(support) => self.add_entries(
                support.properties().len()
                    + support.page_descriptors().len()
                    + support.font_face_descriptors().len()
                    + usize::from(support.svg_glyph_orientation_vertical().is_some()),
            ),
        }
    }
    /// One sheet record. `bases` holds the base URL identities already counted.
    fn sheet(
        &mut self,
        sheet: &CssomSheet,
        bases: &mut HashSet<*const Vec<u16>>,
        limits: &CssomLimits,
    ) -> Result<(), CssomError> {
        #[cfg(test)]
        probe::examined(1);
        if let Some(input) = sheet.authored.parse_source()
            && input.source().as_str().len() > limits.max_input_bytes
        {
            return Err(CssomError::Limit {
                resource: "parse input bytes",
                maximum: limits.max_input_bytes,
            });
        }
        // Retained raw input includes imports filtered from constructed sheets.
        self.add_entries(sheet.authored.rules().len());
        self.version(&sheet.inputs.version);
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
        self.add_strings(title_bytes);
        if let Some(base) = &sheet.inputs.base_url {
            let identity = Arc::as_ptr(base);
            if !bases.contains(&identity) {
                bases.try_reserve(1).map_err(|_| CssomError::Limit {
                    resource: "base URL source identities",
                    maximum: limits.max_objects,
                })?;
                bases.insert(identity);
                self.strings = self.strings.and_then(|total| {
                    base.capacity()
                        .checked_mul(size_of::<u16>())
                        .and_then(|bytes| total.checked_add(bytes))
                });
            }
        }
        for text in [
            &sheet.inputs.location,
            &sheet.inputs.constructor_document,
            &sheet.inputs.owner,
        ]
        .into_iter()
        .flatten()
        {
            self.add_strings(text.len());
        }
        self.add_entries(sheet.diagnostics.len());
        Ok(())
    }
    /// One rule list holding `members` rule identities.
    fn list(&mut self, members: usize) {
        #[cfg(test)]
        probe::examined(1);
        self.add_entries(members);
    }
    fn media(&mut self, media: &CssMediaQueryList) {
        #[cfg(test)]
        probe::examined(1);
        self.add_entries(media.queries().len());
    }
    fn block(&mut self, block: &CssomBlock) {
        #[cfg(test)]
        probe::examined(1);
        self.add_entries(block.counter_edits.len());
        for value in block.counter_edits.iter() {
            if let Some(origin) = value.origin() {
                self.add_strings(origin.source().as_str().len());
            }
        }
        if let Some(CssomProjection::Available(selected)) = block.selected_font_face.as_deref() {
            self.add_entries(selected.entries().len());
        }
        if let Some(input) = &block.replacement_input {
            self.add_strings(input.source().len());
            self.add_entries(input.diagnostics().len());
        }
        self.add_entries(block.admission_inputs.len());
        for input in block.admission_inputs.iter() {
            self.version(input);
        }
        if block.inline_context().is_none()
            && let Some(owner) = block.owner()
        {
            self.version(owner);
        }
        match block.data() {
            CssomBlockData::Properties {
                authored, selected, ..
            } => {
                self.add_entries(match authored {
                    CssomPropertyOccurrences::Ordinary(v)
                    | CssomPropertyOccurrences::RawContents(v) => v.len(),
                    CssomPropertyOccurrences::Keyframe(v) => v.len(),
                    CssomPropertyOccurrences::Margin(v) => v.properties().len(),
                });
                if let CssomProjection::Available(v) = selected {
                    self.add_entries(v.entries().len());
                }
            }
            CssomBlockData::Page { authored, selected } => {
                self.add_entries(authored.occurrences().len());
                if let CssomProjection::Available(v) = selected {
                    self.add_entries(v.entries().len());
                }
            }
            CssomBlockData::CounterStyle(v) => self.add_entries(v.occurrences().len()),
            CssomBlockData::FontFace(v) => self.add_entries(v.occurrences().len()),
        }
    }
    fn map(&mut self, map: &CssomFeatureMap) {
        #[cfg(test)]
        probe::examined(1);
        self.add_entries(map.authored.len());
        self.add_entries(map.entries.len());
        for entry in &map.entries {
            self.add_strings(entry.key.len());
            if let CssomFeatureValue::Unsigned(v) = &entry.value {
                self.add_entries(v.len());
            }
        }
    }
    fn rule(&mut self, rule: &CssomRule, limits: &CssomLimits) -> Result<(), CssomError> {
        #[cfg(test)]
        probe::examined(1);
        self.add_entries(rule.admission_diagnostics.len());
        if let Some(origin) = &rule.selector_input {
            if origin.source().as_str().len() > limits.max_input_bytes {
                return Err(CssomError::Limit {
                    resource: "retained selector input bytes",
                    maximum: limits.max_input_bytes,
                });
            }
            self.add_strings(origin.source().as_str().len());
        }
        match rule.data() {
            CssomRuleData::NamedSupports { name, body, .. } => {
                self.add_strings(name.as_str().len());
                self.add_strings(crate::named_supports_operations::retained_test_input_bytes(
                    name.origin(),
                    limits,
                )?);
                self.add_entries(body.local_recovery_origins().len());
                for child in body.children() {
                    let (count, bytes) =
                        crate::named_supports_operations::test_child_storage(child, limits)?;
                    self.add_entries(count);
                    self.add_strings(bytes);
                }
            }
            CssomRuleData::SupportsTestChild(child) => {
                let (count, bytes) =
                    crate::named_supports_operations::test_child_storage(child.as_ref(), limits)?;
                self.add_entries(count);
                self.add_strings(bytes);
                // Current and historical checked payloads are retained separately.
                self.add_entries(count);
                self.add_strings(bytes);
            }
            CssomRuleData::CounterStyle { name, .. } => self.add_strings(name.len()),
            CssomRuleData::CustomMedia { name, .. } => self.add_strings(name.as_str().len()),
            CssomRuleData::FontFeatureValues {
                families,
                family_input,
                ..
            } => {
                self.add_entries(families.len());
                for family in families {
                    self.add_strings(family.as_str().len());
                }
                if let Some(input) = family_input {
                    self.add_strings(input.source_bytes());
                    self.add_entries(input.diagnostics().len());
                    self.add_entries(input.inputs().len());
                    for input in input.inputs() {
                        self.version(input);
                    }
                }
            }
            CssomRuleData::Keyframes { name, .. } => self.add_strings(match name {
                surgeist_css::rule::keyframes::CssKeyframesName::Ident(value) => {
                    value.as_str().len()
                }
                surgeist_css::rule::keyframes::CssKeyframesName::String(value) => {
                    value.as_str().len()
                }
                _ => 0,
            }),
            CssomRuleData::Import {
                resolved_location,
                input,
                ..
            } => {
                if let Some(v) = resolved_location {
                    self.add_strings(v.len());
                }
                if let Some(v) = input {
                    self.version(v);
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn check_entries(&self, limits: &CssomLimits) -> Result<(), CssomError> {
        limit(self.entries, limits.max_entries, "entries")
    }
    fn check_strings(&self, limits: &CssomLimits) -> Result<(), CssomError> {
        limit(
            self.strings,
            limits.max_string_bytes,
            "host and live string bytes",
        )
    }
}

impl State {
    pub(crate) fn check_limits(&self, limits: &CssomLimits) -> Result<(), CssomError> {
        self.check_limits_with_displaced_media(limits, std::iter::empty())
    }

    /// Refuses a state that exceeds `limits`: the contexts, the object count,
    /// the entries, the retained sources and the string bytes, in that order.
    pub(crate) fn check_limits_with_displaced_media<'a>(
        &self,
        limits: &CssomLimits,
        displaced: impl Iterator<Item = &'a CssMediaQueryList> + Clone,
    ) -> Result<(), CssomError> {
        self.check_contexts(limits)?;
        self.check_object_count(limits)?;
        let mut totals = self.totals(limits)?;
        for media in displaced.clone() {
            totals.add_entries(media.queries().len());
        }
        totals.check_entries(limits)?;
        self.walk_sources(limits, displaced)?;
        totals.check_strings(limits)
    }

    /// The contexts of the state and its inline declaration blocks: input
    /// roles, and the depth and number of the snapshots they link.
    fn check_contexts(&self, limits: &CssomLimits) -> Result<(), CssomError> {
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
        Ok(())
    }

    /// The state itself and every sheet, rule, block, list, media list and
    /// feature map it holds, detached ones included.
    fn check_object_count(&self, limits: &CssomLimits) -> Result<(), CssomError> {
        let objects = 1usize
            .checked_add(self.sheets.len())
            .and_then(|v| v.checked_add(self.rules.len()))
            .and_then(|v| v.checked_add(self.blocks.len()))
            .and_then(|v| v.checked_add(self.lists.len()))
            .and_then(|v| v.checked_add(self.media.len()))
            .and_then(|v| v.checked_add(self.maps.len()));
        limit(objects, limits.max_objects, "objects")
    }

    /// The entries and string bytes of the whole state, refusing an oversized
    /// sheet or rule input, a missing linked sheet and an oversized test payload.
    fn totals(&self, limits: &CssomLimits) -> Result<Totals, CssomError> {
        let mut totals = Totals::new(self.sheet_order.len());
        for context in std::iter::once(self.context.as_ref())
            .chain(
                self.rules
                    .values()
                    .filter_map(|rule| rule.selector_inputs.as_deref()),
            )
            .chain(self.inline_contexts())
        {
            totals.context(context)?;
        }
        let mut bases = HashSet::new();
        for sheet in self.sheets.values() {
            totals.sheet(sheet, &mut bases, limits)?;
        }
        for list in self.lists.values() {
            totals.list(list.len());
        }
        for media in self.media.values() {
            totals.media(media);
        }
        for block in self.blocks.values() {
            totals.block(block);
        }
        for map in self.maps.values() {
            totals.map(map);
        }
        for rule in self.rules.values() {
            totals.rule(rule, limits)?;
        }
        Ok(totals)
    }
}
fn limit(actual: Option<usize>, maximum: usize, resource: &'static str) -> Result<(), CssomError> {
    if actual.is_none_or(|v| v > maximum) {
        Err(CssomError::Limit { resource, maximum })
    } else {
        Ok(())
    }
}

/// Counts of the work census runs do on the calling thread.
#[cfg(test)]
pub(crate) mod probe {
    use std::cell::Cell;

    thread_local! {
        static EXAMINED: Cell<usize> = const { Cell::new(0) };
        static STEPS: Cell<usize> = const { Cell::new(0) };
    }

    /// Objects the census has examined and source walk steps it has charged.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) struct Work {
        pub(crate) examined: usize,
        pub(crate) steps: usize,
    }

    pub(crate) fn examined(objects: usize) {
        EXAMINED.with(|total| total.set(total.get().saturating_add(objects)));
    }
    pub(crate) fn steps(steps: usize) {
        STEPS.with(|total| total.set(total.get().saturating_add(steps)));
    }
    pub(crate) fn reset() {
        EXAMINED.with(|total| total.set(0));
        STEPS.with(|total| total.set(0));
    }
    pub(crate) fn read() -> Work {
        Work {
            examined: EXAMINED.with(Cell::get),
            steps: STEPS.with(Cell::get),
        }
    }
}

#[cfg(test)]
mod bounded_work_tests {
    use super::probe;
    use crate::*;

    /// The work census runs do while one batch creates a sheet of `rules` style rules.
    fn creation_work(rules: usize) -> probe::Work {
        let text = (0..rules)
            .map(|index| format!(".r{index} {{ color: red; margin: 1px 0 }}\n"))
            .collect::<String>();
        let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
        let mut edit = store
            .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
            .unwrap();
        let inputs = CssomSheetInputs::external(
            CssomInputVersion {
                role: CssomInputRole::Owner,
                identity: "census-work".into(),
                revision: 1,
            },
            true,
        );
        probe::reset();
        edit.create_parsed_sheet(&text, inputs).unwrap();
        let work = probe::read();
        store.commit(edit).unwrap();
        work
    }

    #[test]
    fn creating_a_sheet_does_census_work_in_proportion_to_its_rules() {
        let sizes = [100, 200, 400].map(creation_work);
        for pair in sizes.windows(2) {
            let (small, large) = (pair[0], pair[1]);
            // Doubling the rules at most a little more than doubles the work.
            assert!(
                large.examined * 4 <= small.examined * 9,
                "objects examined grew from {} to {} (100, 200, 400 rules: {sizes:?})",
                small.examined,
                large.examined
            );
            assert!(
                large.steps * 4 <= small.steps * 9,
                "walk steps grew from {} to {} (100, 200, 400 rules: {sizes:?})",
                small.steps,
                large.steps
            );
        }
    }
}
