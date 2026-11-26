//! CSS owns source payload traversal and accounting; this adapter supplies the
//! actual retained canonical tables, including detached objects and edited data.
use crate::{model::State, *};
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, DefaultHasher};
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::rule::CssImportTarget;
use surgeist_css::source::{
    CssRetainedSources, CssSourceRetention, CssSourceRetentionError, CssSourceRetentionResource,
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
impl State {
    /// Walks the sources retained by this state, by every state its contexts
    /// link and by the `displaced` media lists into one collector, and sets
    /// `steps` to the steps it took, refused or not.
    pub(crate) fn walk_sources<'a>(
        &self,
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
            links.pending.push((self, 0usize));
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
        #[cfg(test)]
        crate::state_limits::probe::steps(sources.steps());
        *steps = sources.steps();
        result
            .map(|()| sources)
            .map_err(|error| source_refusal(error, limits))
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

/// Charges and visits the sources of one sheet record.
pub(crate) fn walk_sheet(
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
pub(crate) fn walk_rule<'a>(
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
pub(crate) fn walk_block<'a>(
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
pub(crate) struct Links<'a> {
    pending: Vec<(&'a State, usize)>,
    maximum: usize,
}
impl<'a> Links<'a> {
    pub(crate) const fn new(maximum: usize) -> Self {
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
