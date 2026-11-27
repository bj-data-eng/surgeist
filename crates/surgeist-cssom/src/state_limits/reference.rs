//! The census of the whole retained state, kept as the independent reference the
//! measured census is compared with.
//!
//! It accounts for a state in one pass: the totals of every record, the checks
//! of the contexts and objects, and the walk of every retained source into one
//! collector, followed by the thresholds in their order. [`of`] returns the work
//! it did instead of charging it to a host, and what a state or a walk would own
//! is a free function here.
use crate::{model::State, *};
use std::hash::{BuildHasherDefault, DefaultHasher};
use std::{collections::HashSet, sync::Arc};
use surgeist_css::component::{CssComponentValue, CssComponentValueRef, CssValueOrigin};
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::query::supports::CssSupportsTestChildRef;
use surgeist_css::source::{
    CssSourceRetention, CssSourceRetentionError, CssSourceRetentionResource,
};

/// The refusal for a test child of a kind this CSSOM does not know.
fn unsupported_test_child() -> CssomError {
    CssomError::InvalidInput("unsupported named-supports test child")
}

/// The entries and host string bytes the retained state accounts for. A sum
/// that overflows is `None` and is refused by its threshold check.
///
/// It also counts the objects and entries the census runs that built it
/// examined, which are not part of what the state accounts for.
#[derive(Clone, Debug)]
pub(super) struct Totals {
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
    /// One object examined.
    fn examine(&mut self) {
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
                self.add_strings(retained_test_input_bytes(name.origin(), limits)?);
                self.add_entries(body.local_recovery_origins().len());
                for child in body.children() {
                    let (count, bytes) = test_child_storage(child, limits)?;
                    self.add_entries(count);
                    self.add_strings(bytes);
                }
            }
            CssomRuleData::SupportsTestChild(child) => {
                let (count, bytes) = test_child_storage(child.as_ref(), limits)?;
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

/// The resources of the refusals that report an exhausted source collector.
const BYTES_RESOURCE: &str = "retained parse source bytes";
const DEPTH_RESOURCE: &str = "retained source traversal depth";
const STORAGE_RESOURCE: &str = "retained source traversal storage";
const STEPS_RESOURCE: &str = "retained source traversal steps";

/// The refusal that reports an exhausted source collector.
fn source_refusal(error: CssSourceRetentionError, limits: &CssomLimits) -> CssomError {
    assert!(
        !error.is_caller_admission(),
        "standalone Source collector has no caller admission predicate"
    );
    match error.exhausted_limit() {
        Some(CssSourceRetentionResource::RetainedBytes) => CssomError::Limit {
            resource: BYTES_RESOURCE,
            maximum: limits.max_retained_source_bytes,
        },
        Some(CssSourceRetentionResource::Depth) => CssomError::Limit {
            resource: DEPTH_RESOURCE,
            maximum: limits.max_source_traversal_depth,
        },
        // An allocation refusal or exhausted collector storage.
        Some(CssSourceRetentionResource::Storage) | None => CssomError::Limit {
            resource: STORAGE_RESOURCE,
            maximum: limits.max_source_traversal_steps,
        },
        Some(_) => CssomError::Limit {
            resource: STEPS_RESOURCE,
            maximum: limits.max_source_traversal_steps,
        },
    }
}

/// Charges and visits the sources of one sheet record.
fn walk_sheet(
    sources: &mut CssSourceRetention,
    sheet: &CssomSheet,
) -> Result<(), CssSourceRetentionError> {
    sources.charge_step()?;
    sources.visit(sheet.authored.as_ref())?;
    sources.visit(sheet.diagnostics.as_slice())?;
    sources.visit(&sheet.inputs.media)
}

/// Charges and visits the sources of one rule record; the states its selector
/// inputs link join `links`.
fn walk_rule<'a>(
    sources: &mut CssSourceRetention,
    links: &mut Links<'a>,
    depth: usize,
    rule: &'a CssomRule,
) -> Result<(), CssSourceRetentionError> {
    sources.charge_step()?;
    if let Some(context) = rule.selector_inputs.as_deref() {
        links.enqueue(context, sources, depth)?;
    }
    sources.visit(rule.authored.as_ref())?;
    sources.visit(rule.data.as_ref())?;
    if let Some(origin) = &rule.insertion_origin {
        sources.visit(origin)?;
    }
    sources.visit(&rule.selector_input)?;
    sources.visit(rule.admission_diagnostics.as_slice())
}

/// Charges and visits the sources of one block record; the states its inline
/// context links join `links`.
fn walk_block<'a>(
    sources: &mut CssSourceRetention,
    links: &mut Links<'a>,
    depth: usize,
    block: &'a CssomBlock,
) -> Result<(), CssSourceRetentionError> {
    sources.charge_step()?;
    if let Some(context) = block.inline_context() {
        links.enqueue(context.context(), sources, depth)?;
    }
    sources.visit(block.data.as_ref())?;
    sources.visit(block.counter_edits.as_slice())?;
    if let Some(value) = &block.replacement_input {
        if let Some(source) = value.parse_source() {
            sources.source(source.source())?;
        }
        sources.visit(value.diagnostics())?;
    }
    if let Some(value) = &block.selected_font_face {
        sources.visit(value.as_ref())?;
    }
    Ok(())
}

/// Charges and visits the sources of one media list.
fn walk_media(
    sources: &mut CssSourceRetention,
    media: &CssMediaQueryList,
) -> Result<(), CssSourceRetentionError> {
    sources.charge_step()?;
    sources.visit(media)
}

/// Charges and visits the sources of one feature map.
fn walk_map(
    sources: &mut CssSourceRetention,
    map: &CssomFeatureMap,
) -> Result<(), CssSourceRetentionError> {
    sources.charge_step()?;
    sources.visit(map.authored.as_slice())?;
    for entry in &map.entries {
        sources.charge_step()?;
        if let CssomFeatureValue::Authored(value) = &entry.value {
            sources.visit(value)?;
        }
    }
    Ok(())
}

/// The states a walk has visited, by identity. A check costs one step however
/// many states the walk has seen: the set's hash is a fixed function of the
/// state's address, without a random seed. The set holds at most `maximum`
/// states, the step limit, and reserves each insertion fallibly.
struct SeenStates {
    states: HashSet<usize, BuildHasherDefault<DefaultHasher>>,
    maximum: usize,
}
impl SeenStates {
    fn new(maximum: usize) -> Self {
        Self {
            states: HashSet::default(),
            maximum,
        }
    }
    /// Records `state`, and returns whether it was not recorded before.
    fn insert(&mut self, state: &State) -> Result<bool, CssSourceRetentionError> {
        let identity = std::ptr::from_ref(state).addr();
        if self.states.contains(&identity) {
            return Ok(false);
        }
        if self.states.len() >= self.maximum {
            return Err(CssSourceRetentionError::limit(
                CssSourceRetentionResource::Storage,
            ));
        }
        self.states
            .try_reserve(1)
            .map_err(|_| CssSourceRetentionError::allocation())?;
        Ok(self.states.insert(identity))
    }
}

fn reserve<T>(values: &mut Vec<T>, maximum: usize) -> Result<(), CssSourceRetentionError> {
    if values.len() >= maximum {
        return Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::Storage,
        ));
    }
    if values.len() == values.capacity() {
        let required = values
            .len()
            .checked_add(1)
            .ok_or(CssSourceRetentionError::limit(
                CssSourceRetentionResource::Storage,
            ))?;
        let capacity = values
            .capacity()
            .saturating_mul(2)
            .max(required)
            .min(maximum);
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|_| CssSourceRetentionError::allocation())?;
    }
    if values.capacity() > maximum {
        return Err(CssSourceRetentionError::limit(
            CssSourceRetentionResource::Storage,
        ));
    }
    Ok(())
}

/// The states, with their depth, that contexts met by a walk link and that the
/// walk has still to visit.
struct Links<'a> {
    pending: Vec<(&'a State, usize)>,
    maximum: usize,
}
impl<'a> Links<'a> {
    const fn new(maximum: usize) -> Self {
        Self {
            pending: Vec::new(),
            maximum,
        }
    }
    fn reserve(&mut self) -> Result<(), CssSourceRetentionError> {
        reserve(&mut self.pending, self.maximum)
    }
    /// Charges the context and queues the state of each of its linked inputs one level deeper.
    fn enqueue(
        &mut self,
        context: &'a CssomContext,
        sources: &mut CssSourceRetention,
        depth: usize,
    ) -> Result<(), CssSourceRetentionError> {
        sources.charge_step()?;
        for linked in &context.linked {
            sources.charge_step()?;
            self.reserve()?;
            self.pending.push((
                linked.snapshot.state.as_ref(),
                depth.checked_add(1).ok_or(CssSourceRetentionError::limit(
                    CssSourceRetentionResource::Depth,
                ))?,
            ));
        }
        Ok(())
    }
}

/// Count actual retained payload entries without interpreting CSS grammar or
/// estimating the native writer's tariff. Source bytes are charged per retained
/// direct occurrence, as a logical input bound rather than allocator accounting.
pub(super) fn test_child_storage(
    child: CssSupportsTestChildRef<'_>,
    limits: &CssomLimits,
) -> Result<(usize, usize), CssomError> {
    enum Entry<'a> {
        Child(CssSupportsTestChildRef<'a>),
        Component(&'a CssComponentValue),
    }
    let mut pending = Vec::new();
    let mut entries = 0usize;
    fn push<'a>(
        pending: &mut Vec<Entry<'a>>,
        entry: Entry<'a>,
        entries: usize,
        limits: &CssomLimits,
    ) -> Result<(), CssomError> {
        if pending.len().saturating_add(entries) >= limits.max_entries {
            return Err(CssomError::Limit {
                resource: "test payload entries",
                maximum: limits.max_entries,
            });
        }
        pending.try_reserve(1).map_err(|_| CssomError::Limit {
            resource: "test payload capacity",
            maximum: limits.max_entries,
        })?;
        pending.push(entry);
        Ok(())
    }
    let origin = match child {
        CssSupportsTestChildRef::Declaration(v) => v.origin(),
        CssSupportsTestChildRef::QualifiedRule(v) => v.origin(),
        CssSupportsTestChildRef::AtRule(v) => v.origin(),
        _ => return Err(unsupported_test_child()),
    };
    let bytes = retained_test_input_bytes(origin, limits)?;
    push(&mut pending, Entry::Child(child), entries, limits)?;
    while let Some(entry) = pending.pop() {
        entries = entries
            .checked_add(1)
            .filter(|n| *n <= limits.max_entries)
            .ok_or(CssomError::Limit {
                resource: "test payload entries",
                maximum: limits.max_entries,
            })?;
        let (values, body) = match entry {
            Entry::Child(CssSupportsTestChildRef::Declaration(v)) => (v.components(), None),
            Entry::Child(CssSupportsTestChildRef::QualifiedRule(v)) => {
                (v.prelude().items(), Some(v.body()))
            }
            Entry::Child(CssSupportsTestChildRef::AtRule(v)) => {
                push(
                    &mut pending,
                    Entry::Component(v.at_keyword()),
                    entries,
                    limits,
                )?;
                (v.prelude().items(), v.body())
            }
            Entry::Child(_) => return Err(unsupported_test_child()),
            Entry::Component(v) => (
                match v.view() {
                    CssComponentValueRef::Function(v) => v.values().items(),
                    CssComponentValueRef::Block(v) => v.values().items(),
                    _ => &[],
                },
                None,
            ),
        };
        for value in values {
            push(&mut pending, Entry::Component(value), entries, limits)?;
        }
        if let Some(body) = body {
            entries = entries
                .checked_add(body.local_recovery_origins().len())
                .filter(|n| *n <= limits.max_entries)
                .ok_or(CssomError::Limit {
                    resource: "test payload entries",
                    maximum: limits.max_entries,
                })?;
            for child in body.children() {
                push(&mut pending, Entry::Child(child), entries, limits)?;
            }
        }
    }
    Ok((entries, bytes))
}

fn retained_test_input_bytes(
    origin: &CssValueOrigin,
    limits: &CssomLimits,
) -> Result<usize, CssomError> {
    let bytes = match origin {
        CssValueOrigin::Parsed(v) => v.source().as_str().len(),
        CssValueOrigin::ImplicitClosure { opening, .. } => opening.source().as_str().len(),
        _ => 0,
    };
    if bytes > limits.max_input_bytes {
        return Err(CssomError::Limit {
            resource: "retained test input bytes",
            maximum: limits.max_input_bytes,
        });
    }
    Ok(bytes)
}

/// What the census of a state accounts for.
pub(super) struct Census {
    pub(super) totals: Totals,
    pub(super) sources: CssSourceRetention<'static>,
    pub(super) objects: usize,
}

impl Census {
    /// What this census accounted for.
    pub(super) fn figures(&self) -> super::Figures {
        let (entries, strings) = self.totals.sums();
        super::Figures {
            entries,
            strings,
            objects: self.objects,
            retained_bytes: self.sources.usage().retained_bytes(),
            steps: self.sources.steps(),
        }
    }
}

/// One census run: its answer and the work it did, which the walk charges even
/// when it refuses.
pub(super) struct Run {
    pub(super) result: Result<Census, CssomError>,
    pub(super) examined: usize,
    pub(super) counted: usize,
    pub(super) steps: usize,
}

impl Totals {
    /// The entries and string bytes the state accounts for.
    pub(super) fn sums(&self) -> (Option<usize>, Option<usize>) {
        (self.entries, self.strings)
    }
    pub(super) fn parts(&self) -> (Option<usize>, Option<usize>, usize, usize) {
        (self.entries, self.strings, self.examined, self.counted)
    }
}

/// The census of the whole `state` and of the `displaced` media lists, in the order the
/// refusals arise: the contexts, the object count, the entries, the retained
/// sources and the string bytes.
pub(super) fn of<'a>(
    state: &State,
    limits: &CssomLimits,
    displaced: impl Iterator<Item = &'a CssMediaQueryList> + Clone,
) -> Run {
    let mut totals = Totals::new(state.sheet_order.len());
    let mut steps = 0;
    let mut objects = 0;
    let census = (|| {
        check_contexts(state, limits)?;
        objects = check_object_count(state, limits)?;
        add_totals(state, &mut totals, limits)?;
        for media in displaced.clone() {
            totals.add_entries(media.queries().len());
        }
        totals.check_entries(limits)?;
        let sources = walk_sources(state, limits, displaced, &mut steps)?;
        totals.check_strings(limits)?;
        Ok(sources)
    })();
    let (examined, counted) = (totals.examined, totals.counted);
    Run {
        result: census.map(|sources| Census {
            totals,
            sources,
            objects,
        }),
        examined,
        counted,
        steps,
    }
}

/// The contexts of the state and its inline declaration blocks: input
/// roles, and the depth and number of the snapshots they link.
fn check_contexts(state: &State, limits: &CssomLimits) -> Result<(), CssomError> {
    for context in std::iter::once(state.context.as_ref()).chain(state.inline_contexts()) {
        context.validate()?;
    }
    let mut linked = std::iter::once(state.context.as_ref())
        .chain(state.inline_contexts())
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
fn check_object_count(state: &State, limits: &CssomLimits) -> Result<usize, CssomError> {
    let objects = 1usize
        .checked_add(state.sheets.len())
        .and_then(|v| v.checked_add(state.rules.len()))
        .and_then(|v| v.checked_add(state.blocks.len()))
        .and_then(|v| v.checked_add(state.lists.len()))
        .and_then(|v| v.checked_add(state.media.len()))
        .and_then(|v| v.checked_add(state.maps.len()));
    limit(objects, limits.max_objects, "objects")?;
    Ok(objects.unwrap_or(usize::MAX))
}

/// Adds the entries and string bytes of the whole state to `totals`,
/// refusing an oversized sheet or rule input, a missing linked sheet and an
/// oversized test payload.
fn add_totals(state: &State, totals: &mut Totals, limits: &CssomLimits) -> Result<(), CssomError> {
    for context in std::iter::once(state.context.as_ref())
        .chain(
            state
                .rules
                .values()
                .filter_map(|rule| rule.selector_inputs.as_deref()),
        )
        .chain(state.inline_contexts())
    {
        totals.context(context)?;
    }
    let mut bases = HashSet::new();
    for sheet in state.sheets.values() {
        totals.sheet(sheet, &mut bases, limits)?;
    }
    for list in state.lists.values() {
        totals.list(list.len());
    }
    for media in state.media.values() {
        totals.media(media);
    }
    for block in state.blocks.values() {
        totals.block(block);
    }
    for map in state.maps.values() {
        totals.map(map);
    }
    for rule in state.rules.values() {
        totals.rule(rule, limits)?;
    }
    Ok(())
}

fn limit(actual: Option<usize>, maximum: usize, resource: &'static str) -> Result<(), CssomError> {
    if actual.is_none_or(|v| v > maximum) {
        Err(CssomError::Limit { resource, maximum })
    } else {
        Ok(())
    }
}

/// Walks the sources retained by `state`, by every state its contexts
/// link and by the `displaced` media lists into one collector, and sets
/// `steps` to the steps it took, refused or not.
fn walk_sources<'a>(
    state: &State,
    limits: &CssomLimits,
    displaced: impl Iterator<Item = &'a CssMediaQueryList>,
    steps: &mut usize,
) -> Result<CssSourceRetention<'static>, CssomError> {
    let mut sources = CssSourceRetention::with_shared_original_utf16(
        limits.max_retained_source_bytes,
        limits.max_source_traversal_steps,
        limits.max_source_traversal_depth,
    );
    let result = (|| {
        let mut links = Links::new(limits.max_source_traversal_steps);
        sources.charge_step()?;
        links.reserve()?;
        links.pending.push((state, 0usize));
        let mut seen = SeenStates::new(limits.max_source_traversal_steps);
        while let Some((state, depth)) = links.pending.pop() {
            sources.charge_step()?;
            // One step for the identity check, whatever the states walked so far.
            sources.charge_step()?;
            if !seen.insert(state)? {
                continue;
            }
            sources.at_depth(depth, |sources| {
                links.enqueue(state.context.as_ref(), sources, depth)?;
                for sheet in state.sheets.values() {
                    walk_sheet(sources, sheet)?;
                }
                for rule in state.rules.values() {
                    walk_rule(sources, &mut links, depth, rule)?;
                }
                for block in state.blocks.values() {
                    walk_block(sources, &mut links, depth, block)?;
                }
                for media in state.media.values() {
                    walk_media(sources, media)?;
                }
                for map in state.maps.values() {
                    walk_map(sources, map)?;
                }
                Ok(())
            })?;
        }
        for media in displaced {
            sources.visit(media)?;
        }
        Ok(())
    })();
    *steps = sources.steps();
    result
        .map(|()| sources)
        .map_err(|error| source_refusal(error, limits))
}
