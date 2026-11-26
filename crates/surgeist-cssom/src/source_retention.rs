//! CSS owns source payload traversal and accounting; this adapter supplies the
//! actual retained canonical tables, including detached objects and edited data.
//!
//! The retained sources of a state are accounted record by record: each record
//! is measured alone, and the measurements are added up in the order a single
//! collector would have walked the state in. [`Sources`] is that sum: the steps
//! charged and a ledger of the distinct snapshots recorded.
use crate::{model::State, *};
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, DefaultHasher};
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::rule::CssImportTarget;
use surgeist_css::source::{
    CssRetainedSources, CssSourceBounds, CssSourceLedger, CssSourceMeter, CssSourceRetention,
    CssSourceRetentionError, CssSourceRetentionResource,
};

impl CssRetainedSources for CssomAuthoredRule {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Rule(value) => sources.visit(value),
            Self::Keyframe(value) => sources.visit(value),
            Self::Margin(value) => sources.visit(value),
            Self::RawNestedDeclarations(value) => sources.visit(value),
            Self::SupportsTestChild(value) => sources.visit(value),
        }
    }
}
impl CssRetainedSources for CssomGroupPrelude {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Supports(value) => sources.visit(value),
            Self::Container(value) => sources.visit(value),
            Self::Scope { start, end } => {
                sources.visit(start)?;
                sources.visit(end)
            }
            Self::When(value) => sources.visit(value),
            Self::Else(value) => sources.visit(value),
            Self::Media(_) | Self::Layer(_) => Ok(()), // list identity / decoded strings
        }
    }
}
impl CssRetainedSources for CssomRuleData {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::NamedSupports { name, body, .. } => {
                sources.visit(name.component())?;
                sources.visit(body)
            }
            Self::SupportsTestChild(value) => sources.visit(value),
            Self::Style { selectors, .. } => sources.visit(selectors),
            Self::Group { prelude, .. } => sources.visit(prelude),
            Self::Page { selectors, .. } => sources.visit(selectors),
            Self::Keyframe { selectors, .. } => sources.visit(selectors),
            Self::Import {
                target,
                supports,
                origin,
                ..
            } => {
                sources.visit(origin)?;
                if let CssImportTarget::Url(value) = target {
                    sources.visit(value.url())?;
                }
                if let Some(value) = supports {
                    sources.visit(value.condition())?;
                }
                Ok(())
            }
            Self::FontFeatureValues {
                family_input,
                font_display,
                ..
            } => {
                if let Some(value) = family_input {
                    sources.visit(value.value())?;
                    sources.visit(value.diagnostics())?;
                }
                sources.visit(font_display.as_slice())
            }
            Self::CustomMedia { name, .. } => sources.visit(name.component()),
            Self::Leaf { current } => sources.visit(current),
            Self::NestedDeclarations { .. }
            | Self::Margin { .. }
            | Self::Keyframes { .. }
            | Self::CounterStyle { .. }
            | Self::FontFace { .. } => Ok(()), // references / decoded names
        }
    }
}
impl<T: CssRetainedSources, E: CssRetainedSources> CssRetainedSources for CssomProjection<T, E> {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Available(value) => sources.visit(value),
            Self::Unavailable(value) => sources.visit(value),
        }
    }
}
impl CssRetainedSources for CssomPropertyOccurrences {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Ordinary(value) | Self::RawContents(value) => sources.visit(value),
            Self::Keyframe(value) => sources.visit(value),
            Self::Margin(value) => sources.visit(value),
        }
    }
}
impl CssRetainedSources for CssomBlockData {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        match self {
            Self::Properties {
                authored, selected, ..
            } => {
                sources.visit(authored)?;
                sources.visit(selected)
            }
            Self::Page { authored, selected } => {
                sources.visit(authored)?;
                sources.visit(selected)
            }
            Self::CounterStyle(value) => sources.visit(value.as_ref()),
            Self::FontFace(value) => sources.visit(value),
        }
    }
}

/// The sum of the accounts of the sources a walk has reached: the steps it has
/// charged and the distinct snapshots it has recorded, under the limits of the walk.
///
/// A record is measured alone and added when its figures fit. A record that
/// would take the walk beyond a limit is run again as a collector that had
/// visited everything added so far would run it, so the refusal is the one a
/// single collector walking the state in this order raises, after the same steps.
pub(crate) struct Sources {
    ledger: CssSourceLedger,
    /// Measures one record after another; it holds no byte bound, which the ledger's total answers to.
    meter: CssSourceMeter,
    steps: usize,
    bounds: CssSourceBounds,
}

impl Sources {
    pub(crate) fn new(limits: &CssomLimits) -> Self {
        Self {
            ledger: CssSourceLedger::new(),
            meter: CssSourceMeter::new(CssSourceBounds::new(
                usize::MAX,
                limits.max_source_traversal_steps,
                limits.max_source_traversal_depth,
            )),
            steps: 0,
            bounds: CssSourceBounds::new(
                limits.max_retained_source_bytes,
                limits.max_source_traversal_steps,
                limits.max_source_traversal_depth,
            ),
        }
    }

    /// The steps charged so far, a refused one not counted.
    pub(crate) const fn steps(&self) -> usize {
        self.steps
    }

    /// The retained source bytes of the distinct snapshots recorded.
    #[cfg(test)]
    pub(crate) const fn bytes(&self) -> usize {
        self.ledger.bytes()
    }

    /// Charges `steps` steps of the walk's own traversal.
    fn charge(&mut self, steps: usize) -> Result<(), CssSourceRetentionError> {
        match self
            .steps
            .checked_add(steps)
            .filter(|total| *total <= self.bounds.steps())
        {
            Some(total) => {
                self.steps = total;
                Ok(())
            }
            None => {
                self.steps = self.bounds.steps();
                Err(CssSourceRetentionError::limit(
                    CssSourceRetentionResource::Visits,
                ))
            }
        }
    }

    /// Charges the context of a state and each input it links.
    fn charge_context(&mut self, context: &CssomContext) -> Result<(), CssSourceRetentionError> {
        self.charge(context.linked.len().saturating_add(1))
    }

    /// Adds what `walk` retains when run at `depth`.
    pub(crate) fn record(
        &mut self,
        depth: usize,
        walk: impl Fn(&mut CssSourceRetention<'_>) -> Result<(), CssSourceRetentionError>,
    ) -> Result<(), CssSourceRetentionError> {
        let remaining = self.bounds.steps().saturating_sub(self.steps);
        if self.meter.measure(depth, &walk).is_ok()
            && self.meter.steps() <= remaining
            && self.ledger.add_metered(&self.meter).is_ok()
        {
            if self.ledger.bytes() <= self.bounds.bytes() {
                self.steps = self.steps.saturating_add(self.meter.steps());
                return Ok(());
            }
            self.ledger.remove_metered(&self.meter);
        }
        let replay = self.ledger.replay(
            CssSourceBounds::new(self.bounds.bytes(), remaining, self.bounds.depth()),
            depth,
            &walk,
        );
        self.steps = self.steps.saturating_add(replay.steps());
        match replay.result() {
            Err(error) => Err(*error),
            // The record was refused alone, so the replay refuses it too; the
            // snapshots are added for a replay that does not.
            Ok(measurement) => self.ledger.add(measurement),
        }
    }
}

impl State {
    /// Walks the sources retained by this state, by every state its contexts
    /// link and by the `displaced` media lists, and sets `steps` to the steps it
    /// took, refused or not.
    pub(crate) fn walk_sources<'a>(
        &self,
        limits: &CssomLimits,
        displaced: impl Iterator<Item = &'a CssMediaQueryList>,
        steps: &mut usize,
    ) -> Result<Sources, CssomError> {
        let mut sources = Sources::new(limits);
        let result = self.walk_states(&mut sources, limits, displaced);
        #[cfg(test)]
        crate::state_limits::probe::steps(sources.steps());
        *steps = sources.steps();
        result
            .map(|()| sources)
            .map_err(|error| source_refusal(error, limits))
    }

    fn walk_states<'d>(
        &self,
        sources: &mut Sources,
        limits: &CssomLimits,
        displaced: impl Iterator<Item = &'d CssMediaQueryList>,
    ) -> Result<(), CssSourceRetentionError> {
        let maximum = limits.max_source_traversal_steps;
        let mut pending: Vec<(&State, usize)> = Vec::new();
        sources.charge(1)?;
        reserve(&mut pending, maximum)?;
        pending.push((self, 0usize));
        let mut seen = SeenStates::new(maximum);
        while let Some((state, depth)) = pending.pop() {
            sources.charge(1)?;
            // One step for the identity check, whatever the states walked so far.
            sources.charge(1)?;
            if !seen.insert(state)? {
                continue;
            }
            if depth > limits.max_source_traversal_depth {
                return Err(CssSourceRetentionError::limit(
                    CssSourceRetentionResource::Depth,
                ));
            }
            sources.charge_context(&state.context)?;
            enqueue(&mut pending, &state.context, depth, maximum)?;
            for sheet in state.sheets.values() {
                sources.record(depth, |walk| walk_sheet(walk, sheet))?;
            }
            for rule in state.rules.values() {
                sources.record(depth, |walk| walk_rule(walk, rule))?;
                if let Some(context) = rule.selector_inputs.as_deref() {
                    enqueue(&mut pending, context, depth, maximum)?;
                }
            }
            for block in state.blocks.values() {
                sources.record(depth, |walk| walk_block(walk, block))?;
                if let Some(context) = block.inline_context() {
                    enqueue(&mut pending, context.context(), depth, maximum)?;
                }
            }
            for media in state.media.values() {
                sources.record(depth, |walk| walk_media(walk, media))?;
            }
            for map in state.maps.values() {
                sources.record(depth, |walk| walk_map(walk, map))?;
            }
        }
        for media in displaced {
            sources.record(0, |walk| walk.visit(media))?;
        }
        Ok(())
    }
}

/// The resources of the refusals that report an exhausted source collector.
const BYTES_RESOURCE: &str = "retained parse source bytes";
const DEPTH_RESOURCE: &str = "retained source traversal depth";
const STORAGE_RESOURCE: &str = "retained source traversal storage";
const STEPS_RESOURCE: &str = "retained source traversal steps";

/// The refusal that reports an exhausted source collector.
pub(crate) fn source_refusal(error: CssSourceRetentionError, limits: &CssomLimits) -> CssomError {
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

/// Visits the payload of a part a record holds behind a shared handle, which
/// the probe counts as one part measured.
fn part<T: ?Sized>(
    sources: &mut CssSourceRetention<'_>,
    value: &T,
    visit: impl FnOnce(&mut CssSourceRetention<'_>, &T) -> Result<(), CssSourceRetentionError>,
) -> Result<(), CssSourceRetentionError> {
    #[cfg(test)]
    crate::state_limits::probe::part_measured();
    visit(sources, value)
}

/// Charges the context of a record or state, and each input it links.
fn charge_context(
    sources: &mut CssSourceRetention,
    context: &CssomContext,
) -> Result<(), CssSourceRetentionError> {
    sources.charge_step()?;
    for _ in &context.linked {
        sources.charge_step()?;
    }
    Ok(())
}

/// Charges and visits the sources of one sheet record.
pub(crate) fn walk_sheet(
    sources: &mut CssSourceRetention,
    sheet: &CssomSheet,
) -> Result<(), CssSourceRetentionError> {
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
        version: _,
        base_url: _,
        location: _,
        constructor_document: _,
        owner: _,
        title: _,
        alternate: _,
        disabled: _,
        origin_clean: _,
        constructed: _,
        disallow_modification: _,
        media,
    } = inputs;
    sources.charge_step()?;
    part(sources, authored.as_ref(), |sources, value| {
        sources.visit(value)
    })?;
    part(sources, diagnostics.as_slice(), |sources, value| {
        sources.visit(value)
    })?;
    sources.visit(media)
}

/// Charges and visits the sources of one rule record.
pub(crate) fn walk_rule(
    sources: &mut CssSourceRetention,
    rule: &CssomRule,
) -> Result<(), CssSourceRetentionError> {
    let CssomRule {
        parent: _,
        data,
        authored,
        insertion_origin,
        selector_input,
        selector_inputs,
        selector_namespaces: _,
        admission_diagnostics,
    } = rule;
    sources.charge_step()?;
    if let Some(context) = selector_inputs.as_deref() {
        charge_context(sources, context)?;
    }
    part(sources, authored.as_ref(), |sources, value| {
        sources.visit(value)
    })?;
    part(sources, data.as_ref(), |sources, value| {
        sources.visit(value)
    })?;
    if let Some(origin) = insertion_origin {
        sources.visit(origin)?;
    }
    sources.visit(selector_input)?;
    part(
        sources,
        admission_diagnostics.as_slice(),
        |sources, value| sources.visit(value),
    )
}

/// Charges and visits the sources of one block record.
pub(crate) fn walk_block(
    sources: &mut CssSourceRetention,
    block: &CssomBlock,
) -> Result<(), CssSourceRetentionError> {
    let CssomBlock {
        parent: _,
        binding,
        flags: _,
        data,
        admission_inputs: _,
        counter_edits,
        replacement_input,
        selected_font_face,
    } = block;
    sources.charge_step()?;
    if let crate::inline_context::BlockBinding::Inline(context) = binding {
        charge_context(sources, context.context())?;
    }
    part(sources, data.as_ref(), |sources, value| {
        sources.visit(value)
    })?;
    part(sources, counter_edits.as_slice(), |sources, value| {
        sources.visit(value)
    })?;
    if let Some(value) = replacement_input {
        part(sources, value.as_ref(), |sources, value| {
            if let Some(source) = value.parse_source() {
                sources.source(source.source())?;
            }
            sources.visit(value.diagnostics())
        })?;
    }
    if let Some(value) = selected_font_face {
        part(sources, value.as_ref(), |sources, value| {
            sources.visit(value)
        })?;
    }
    Ok(())
}

/// Charges and visits the sources of one media list.
pub(crate) fn walk_media(
    sources: &mut CssSourceRetention,
    media: &CssMediaQueryList,
) -> Result<(), CssSourceRetentionError> {
    sources.charge_step()?;
    sources.visit(media)
}

/// Charges and visits the sources of one feature map.
pub(crate) fn walk_map(
    sources: &mut CssSourceRetention,
    map: &CssomFeatureMap,
) -> Result<(), CssSourceRetentionError> {
    let CssomFeatureMap {
        kind: _,
        entries,
        authored,
    } = map;
    sources.charge_step()?;
    part(sources, authored.as_slice(), |sources, value| {
        sources.visit(value)
    })?;
    for entry in entries {
        let CssomFeatureEntry { key: _, value } = entry;
        sources.charge_step()?;
        if let CssomFeatureValue::Authored(value) = value {
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

/// Queues the state of each input `context` links, one level deeper than `depth`.
fn enqueue<'a>(
    pending: &mut Vec<(&'a State, usize)>,
    context: &'a CssomContext,
    depth: usize,
    maximum: usize,
) -> Result<(), CssSourceRetentionError> {
    for linked in &context.linked {
        reserve(pending, maximum)?;
        pending.push((
            linked.snapshot.state.as_ref(),
            depth.checked_add(1).ok_or(CssSourceRetentionError::limit(
                CssSourceRetentionResource::Depth,
            ))?,
        ));
    }
    Ok(())
}
