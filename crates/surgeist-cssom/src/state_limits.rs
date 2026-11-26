//! Synchronous admission accounting for complete retained CSSOM state.
//!
//! The census adds up what the retained state holds: objects, entries, host
//! string bytes and the sources its payloads retain. Each kind of object
//! contributes through its own function, so a census over the whole state is
//! the sum of those contributions followed by the threshold checks.
use crate::source_retention::{Links, source_refusal, walk_block, walk_map, walk_media, walk_rule};
use crate::{model::State, *};
use std::{collections::HashSet, sync::Arc};
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::source::CssSourceRetention;

/// The entries and host string bytes the retained state accounts for. A sum
/// that overflows is `None` and is refused by its threshold check.
///
/// It also counts the objects and entries the census runs that built it
/// examined, which are not part of what the state accounts for.
#[derive(Clone, Debug)]
pub(crate) struct Totals {
    entries: Option<usize>,
    strings: Option<usize>,
    examined: usize,
    counted: usize,
}

impl Totals {
    /// The totals of a state without objects: the sheet order is its only entry list.
    fn new(sheet_order: usize) -> Self {
        Self {
            entries: Some(sheet_order),
            strings: Some(0),
            examined: 0,
            counted: sheet_order,
        }
    }
    /// The entries and string bytes, without the census work.
    #[cfg(test)]
    fn sums(&self) -> (Option<usize>, Option<usize>) {
        (self.entries, self.strings)
    }
    /// One object examined.
    fn examine(&mut self) {
        #[cfg(test)]
        probe::examined(1);
        self.examined = self.examined.saturating_add(1);
    }
    fn add_entries(&mut self, count: usize) {
        self.counted = self.counted.saturating_add(count);
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
        self.examine();
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
        self.examine();
        self.add_entries(members);
    }
    fn media(&mut self, media: &CssMediaQueryList) {
        self.examine();
        self.add_entries(media.queries().len());
    }
    fn block(&mut self, block: &CssomBlock) {
        self.examine();
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
        self.examine();
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
        self.examine();
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

/// What a census of a whole state accounted for.
#[cfg(test)]
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Figures {
    pub(crate) entries: Option<usize>,
    pub(crate) strings: Option<usize>,
    pub(crate) objects: usize,
    pub(crate) retained_bytes: usize,
    pub(crate) steps: usize,
}

/// What the census of a state accounts for: its totals and the sources its
/// payloads retain, collected once.
pub(crate) struct Census {
    totals: Totals,
    sources: CssSourceRetention<'static>,
    /// The objects of the state, the state itself included.
    objects: usize,
}

/// What adoption has inserted into a state, and the entries it has added to
/// rule lists, since a census last accounted for that state.
///
/// Adoption only inserts: nothing it inserts replaces or changes an object
/// that is already retained, except that a rule list gains members.
#[derive(Debug, Default)]
pub(crate) struct Inserted {
    rules: Vec<CssomRuleId>,
    blocks: Vec<CssomBlockId>,
    media: Vec<CssomMediaListId>,
    maps: Vec<CssomFeatureMapId>,
    members: usize,
}

impl Inserted {
    pub(crate) fn rule(&mut self, id: CssomRuleId) {
        self.rules.push(id);
    }
    pub(crate) fn block(&mut self, id: CssomBlockId) {
        self.blocks.push(id);
    }
    pub(crate) fn media(&mut self, id: CssomMediaListId) {
        self.media.push(id);
    }
    pub(crate) fn map(&mut self, id: CssomFeatureMapId) {
        self.maps.push(id);
    }
    /// One rule identity appended to a rule list.
    pub(crate) fn member(&mut self) {
        self.members = self.members.saturating_add(1);
    }
    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.rules.is_empty()
            && self.blocks.is_empty()
            && self.media.is_empty()
            && self.maps.is_empty()
            && self.members == 0
    }
    /// The inserted objects as the state holds them, or `None` when one is
    /// missing or reaches states of other snapshots: only a census of the whole
    /// state accounts for those.
    fn resolve<'a>(&self, state: &'a State) -> Option<Resolved<'a>> {
        Some(Resolved {
            rules: self
                .rules
                .iter()
                .map(|id| {
                    state
                        .rules
                        .get(id)
                        .filter(|rule| rule.selector_inputs.is_none())
                })
                .collect::<Option<Vec<_>>>()?,
            blocks: self
                .blocks
                .iter()
                .map(|id| {
                    state
                        .blocks
                        .get(id)
                        .filter(|block| block.inline_context().is_none())
                })
                .collect::<Option<Vec<_>>>()?,
            media: self
                .media
                .iter()
                .map(|id| state.media.get(id))
                .collect::<Option<Vec<_>>>()?,
            maps: self
                .maps
                .iter()
                .map(|id| state.maps.get(id))
                .collect::<Option<Vec<_>>>()?,
        })
    }
}

struct Resolved<'a> {
    rules: Vec<&'a CssomRule>,
    blocks: Vec<&'a CssomBlock>,
    media: Vec<&'a CssMediaQueryList>,
    maps: Vec<&'a CssomFeatureMap>,
}

/// How [`Census::extend`] answered.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Extension {
    /// The census accounts for the inserted objects.
    Applied,
    /// The inserted objects cannot be accounted for alone; the census is unchanged.
    WholeStateNeeded,
}

impl Census {
    /// The census of the whole `state` and of the `displaced` media lists, in the order the
    /// refusals arise: the contexts, the object count, the entries, the retained
    /// sources and the string bytes.
    pub(crate) fn of<'a>(
        state: &State,
        limits: &CssomLimits,
        displaced: impl Iterator<Item = &'a CssMediaQueryList> + Clone,
    ) -> Result<Self, CssomError> {
        #[cfg(test)]
        if probe::reference() {
            return Self::of_reference(state, limits, displaced);
        }
        let mut totals = Totals::new(state.sheet_order.len());
        let mut steps = 0;
        let mut objects = 0;
        let census = (|| {
            state.check_contexts(limits)?;
            objects = state.check_object_count(limits)?;
            state.add_totals(&mut totals, limits)?;
            for media in displaced.clone() {
                totals.add_entries(media.queries().len());
            }
            totals.check_entries(limits)?;
            let sources = state.walk_sources(limits, displaced, &mut steps)?;
            totals.check_strings(limits)?;
            Ok(sources)
        })();
        crate::operation_work::census(totals.examined, totals.counted, steps);
        #[cfg(test)]
        probe::census_run(totals.examined, totals.counted, steps);
        census.map(|sources| Self {
            totals,
            sources,
            objects,
        })
    }

    /// The census of the whole state computed by the reference, charged as a
    /// census run is.
    #[cfg(test)]
    fn of_reference<'a>(
        state: &State,
        limits: &CssomLimits,
        displaced: impl Iterator<Item = &'a CssMediaQueryList> + Clone,
    ) -> Result<Self, CssomError> {
        let run = reference::of(state, limits, displaced);
        crate::operation_work::census(run.examined, run.counted, run.steps);
        run.result.map(|census| {
            let (entries, strings, examined, counted) = census.totals.parts();
            Self {
                totals: Totals {
                    entries,
                    strings,
                    examined,
                    counted,
                },
                sources: census.sources,
                objects: census.objects,
            }
        })
    }

    /// What this census accounted for.
    #[cfg(test)]
    pub(crate) fn figures(&self) -> Figures {
        let (entries, strings) = self.totals.sums();
        Figures {
            entries,
            strings,
            objects: self.objects,
            retained_bytes: self.sources.usage().retained_bytes(),
            steps: self.sources.steps(),
        }
    }

    /// The size of the state this census accounted for, which its limits admitted.
    pub(crate) fn retained(&self) -> crate::operation_work::Retained {
        crate::operation_work::Retained::new(
            self.objects,
            self.totals.entries.unwrap_or(usize::MAX),
            self.totals.strings.unwrap_or(usize::MAX),
        )
    }

    /// Accounts for what adoption inserted into `state` since this census
    /// accounted for it, and applies the thresholds in the order of the census
    /// of the whole state.
    ///
    /// The result equals [`Census::of`] on `state` when this census accounted
    /// for `state` before the insertions and only `inserted` has happened since.
    /// Refusals and source walk steps are the same. After a refusal this census
    /// accounts for neither state.
    pub(crate) fn extend(
        &mut self,
        state: &State,
        inserted: &Inserted,
        limits: &CssomLimits,
    ) -> Result<Extension, CssomError> {
        let Some(resolved) = inserted.resolve(state) else {
            return Ok(Extension::WholeStateNeeded);
        };
        let (examined, counted, steps) = (
            self.totals.examined,
            self.totals.counted,
            self.sources.steps(),
        );
        let extension = self.extend_resolved(state, &resolved, inserted.members, limits);
        crate::operation_work::census(
            self.totals.examined.saturating_sub(examined),
            self.totals.counted.saturating_sub(counted),
            self.sources.steps().saturating_sub(steps),
        );
        extension
    }

    /// [`Census::extend`] once the inserted objects are resolved.
    fn extend_resolved(
        &mut self,
        state: &State,
        resolved: &Resolved<'_>,
        members: usize,
        limits: &CssomLimits,
    ) -> Result<Extension, CssomError> {
        self.objects = state.check_object_count(limits)?;
        for media in &resolved.media {
            self.totals.media(media);
        }
        for block in &resolved.blocks {
            self.totals.block(block);
        }
        for map in &resolved.maps {
            self.totals.map(map);
        }
        self.totals.add_entries(members);
        for rule in &resolved.rules {
            self.totals.rule(rule, limits)?;
        }
        self.totals.check_entries(limits)?;
        #[cfg(test)]
        let before = self.sources.steps();
        let walked = self.walk(resolved, limits);
        #[cfg(test)]
        probe::steps(self.sources.steps().saturating_sub(before));
        walked?;
        self.totals.check_strings(limits)?;
        #[cfg(test)]
        probe::extended();
        Ok(Extension::Applied)
    }

    /// Walks the retained sources of the inserted objects into the collector.
    fn walk(&mut self, inserted: &Resolved<'_>, limits: &CssomLimits) -> Result<(), CssomError> {
        // Resolved objects reach no linked state, so nothing joins `links`.
        let mut links = Links::new(limits.max_source_traversal_steps);
        self.sources
            .at_depth(0, |sources| {
                for rule in &inserted.rules {
                    walk_rule(sources, &mut links, 0, rule)?;
                }
                for block in &inserted.blocks {
                    walk_block(sources, &mut links, 0, block)?;
                }
                for media in &inserted.media {
                    walk_media(sources, media)?;
                }
                for map in &inserted.maps {
                    walk_map(sources, map)?;
                }
                Ok(())
            })
            .map_err(|error| source_refusal(error, limits))
    }

    /// Panics unless `outcome`, the answer of extending this census over the
    /// insertions into `state`, agrees with a census of the whole `state`:
    /// equal totals, retained source bytes and walk steps, and the same refusal.
    #[cfg(test)]
    pub(crate) fn assert_matches_whole_state(
        &self,
        outcome: &Result<Extension, CssomError>,
        state: &State,
        limits: &CssomLimits,
    ) {
        let whole = reference::of(state, limits, std::iter::empty()).result;
        match (outcome, whole) {
            (Ok(Extension::WholeStateNeeded), _) => {}
            (Ok(Extension::Applied), Ok(whole)) => {
                assert_eq!(self.totals.sums(), whole.totals.sums(), "totals");
                assert_eq!(
                    self.sources.usage(),
                    whole.sources.usage(),
                    "retained bytes"
                );
                assert_eq!(self.sources.steps(), whole.sources.steps(), "walk steps");
            }
            (Err(extended), Err(whole)) => {
                assert_eq!(format!("{extended:?}"), format!("{whole:?}"), "refusal");
            }
            (outcome, whole) => panic!(
                "extended census answered {outcome:?}, whole state answered {:?}",
                whole.map(|_| ())
            ),
        }
    }
}

impl State {
    pub(crate) fn check_limits(&self, limits: &CssomLimits) -> Result<(), CssomError> {
        self.check_limits_with_displaced_media(limits, std::iter::empty())
    }

    /// Refuses a state that exceeds `limits`; see [`Census::of`].
    pub(crate) fn check_limits_with_displaced_media<'a>(
        &self,
        limits: &CssomLimits,
        displaced: impl Iterator<Item = &'a CssMediaQueryList> + Clone,
    ) -> Result<(), CssomError> {
        Census::of(self, limits, displaced).map(drop)
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
    fn check_object_count(&self, limits: &CssomLimits) -> Result<usize, CssomError> {
        let objects = 1usize
            .checked_add(self.sheets.len())
            .and_then(|v| v.checked_add(self.rules.len()))
            .and_then(|v| v.checked_add(self.blocks.len()))
            .and_then(|v| v.checked_add(self.lists.len()))
            .and_then(|v| v.checked_add(self.media.len()))
            .and_then(|v| v.checked_add(self.maps.len()));
        limit(objects, limits.max_objects, "objects")?;
        Ok(objects.unwrap_or(usize::MAX))
    }

    /// Adds the entries and string bytes of the whole state to `totals`,
    /// refusing an oversized sheet or rule input, a missing linked sheet and an
    /// oversized test payload.
    fn add_totals(&self, totals: &mut Totals, limits: &CssomLimits) -> Result<(), CssomError> {
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
        Ok(())
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
        static EXTENSIONS: Cell<usize> = const { Cell::new(0) };
        static WHOLE_RUNS: Cell<usize> = const { Cell::new(0) };
        static WHOLE_STATE: Cell<bool> = const { Cell::new(false) };
        static CROSS_CHECK: Cell<bool> = const { Cell::new(true) };
        static REFERENCE: Cell<bool> = const { Cell::new(false) };
        static LAST_RUN: Cell<Option<Run>> = const { Cell::new(None) };
    }

    /// Whether every census of a whole state is the reference's instead of the
    /// measured census.
    pub(crate) fn reference() -> bool {
        REFERENCE.with(Cell::get)
    }
    pub(crate) fn set_reference(value: bool) {
        REFERENCE.with(|flag| flag.set(value));
    }

    /// The work of one census run of a whole state, as it charged it.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) struct Run {
        pub(crate) examined: usize,
        pub(crate) counted: usize,
        pub(crate) steps: usize,
    }
    pub(crate) fn census_run(examined: usize, counted: usize, steps: usize) {
        LAST_RUN.with(|run| {
            run.set(Some(Run {
                examined,
                counted,
                steps,
            }));
        });
    }
    /// The work of the last census run of a whole state on this thread.
    pub(crate) fn last_run() -> Option<Run> {
        LAST_RUN.with(Cell::get)
    }

    /// Whether adoption runs the census of the whole state after every rule
    /// instead of extending one census: the reference the incremental check
    /// must agree with.
    pub(crate) fn whole_state() -> bool {
        WHOLE_STATE.with(Cell::get)
    }
    pub(crate) fn set_whole_state(value: bool) {
        WHOLE_STATE.with(|flag| flag.set(value));
    }
    /// Whether adoption compares each extended census with a census of the
    /// whole state (on by default).
    pub(crate) fn cross_check() -> bool {
        CROSS_CHECK.with(Cell::get)
    }
    pub(crate) fn set_cross_check(value: bool) {
        CROSS_CHECK.with(|flag| flag.set(value));
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
    /// One census was extended instead of walking the whole state.
    pub(crate) fn extended() {
        EXTENSIONS.with(|total| total.set(total.get().saturating_add(1)));
    }
    pub(crate) fn extensions() -> usize {
        EXTENSIONS.with(Cell::get)
    }
    /// One adoption check walked the whole state.
    pub(crate) fn whole_run() {
        WHOLE_RUNS.with(|total| total.set(total.get().saturating_add(1)));
    }
    pub(crate) fn whole_runs() -> usize {
        WHOLE_RUNS.with(Cell::get)
    }
    pub(crate) fn reset() {
        EXAMINED.with(|total| total.set(0));
        STEPS.with(|total| total.set(0));
        EXTENSIONS.with(|total| total.set(0));
        WHOLE_RUNS.with(|total| total.set(0));
    }
    pub(crate) fn read() -> Work {
        Work {
            examined: EXAMINED.with(Cell::get),
            steps: STEPS.with(Cell::get),
        }
    }
}

#[cfg(test)]
mod differential_tests;
#[cfg(test)]
mod reference;

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
        // The cross-check runs the census of the whole state it is compared with.
        probe::set_cross_check(false);
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
