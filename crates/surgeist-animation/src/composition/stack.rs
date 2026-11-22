//! Immutable ordering, value folds, and replacement decisions over supplied state.

use core::{cmp::Ordering, fmt};

use super::PropertyValueOperations;
use crate::interpolation::{InterpolableValue, PropertyKey};
use crate::keyframe::{
    ComputedKeyframes, KeyframeSampleError, KeyframeSampleInput, KeyframeTarget,
};
use crate::keyframe_input::CompositeOperation;
use crate::playback::{AnimationId, AnimationRecords, CreationOrder, EffectFacts, PlaybackError};
use crate::time::AnimationTime;
use crate::timeline::{TimelineMonotonicity, TimelineSnapshot};
use crate::timing::{
    AnimationReplaceState, EffectAnimationState, EffectPlayState, EffectTiming, EffectTimingError,
    EffectTimingInput, EffectTimingSample,
};

/// The creating interface's class; its numeric identity has no ordering meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimationClass(u64);
impl AnimationClass {
    pub const fn new(identity: u64) -> Self {
        Self(identity)
    }
    pub const fn identity(self) -> u64 {
        self.0
    }
}

/// An exact effect target identity, including any pseudo-element distinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectTargetId(u64);
impl EffectTargetId {
    pub const fn new(identity: u64) -> Self {
        Self(identity)
    }
    pub const fn identity(self) -> u64 {
        self.0
    }
}

/// Frozen ordering facts supplied by the creating interface and CSS owner.
///
/// Class identity does not encode CSS ownership. Inter-class ranks are consulted
/// only for different classes with defined ranks on both entries. Common-class
/// ranks are consulted only within that class. Missing/equal ranks fall through
/// to the checked global-list creation positions. Lower ranks sort earlier.
/// CSS adapters must supply only source-applicable ranks: orphan CSS animations
/// do not retain owning-element precedence. Orphan idle animations can have
/// undefined order; these observations are retained outside the ordered stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationOrder {
    Defined {
        class: AnimationClass,
        inter_class: Option<i64>,
        class_specific: Option<u64>,
    },
    Undefined {
        class: AnimationClass,
    },
}
impl AnimationOrder {
    pub const fn class(self) -> AnimationClass {
        match self {
            Self::Defined { class, .. } | Self::Undefined { class } => class,
        }
    }
}

/// Caller-computed associated effect data, validated by the snapshot constructor.
///
/// `not_animatable` lists represented properties excluded from target values.
/// Other represented properties are animatable by default; property-specific
/// operation selection remains with [`PropertyValueOperations`]. No exclusion
/// changes effect timing, completion, or event eligibility.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectStackEffect {
    pub timing: EffectTiming,
    pub keyframes: ComputedKeyframes,
    pub composite: CompositeOperation,
    pub not_animatable: Vec<PropertyKey>,
}

/// Coherent caller-owned observations for one registered animation in one update.
///
/// Playback/control owners supply `local_time` and `state`; this is not another
/// playback lifecycle. Construction checks identities, association, timeline
/// consistency, registered effect end, and effect timing's state/time rules.
/// Adapters own the truth of CSS ordering, target identity, markup-prescribed
/// existence, and document membership; this crate cannot infer them from IDs.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationStackInput {
    pub animation: AnimationId,
    pub timeline: Option<TimelineSnapshot>,
    pub local_time: Option<AnimationTime>,
    pub state: EffectAnimationState,
    pub order: AnimationOrder,
    pub target: Option<EffectTargetId>,
    pub prescribed_by_markup: bool,
    pub in_document: bool,
    pub effect: Option<EffectStackEffect>,
}

/// The category rejected while constructing an immutable stack snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectStackErrorKind {
    Playback,
    Timing,
    Capacity,
    DuplicateAnimation,
    TimelineMismatch,
    TargetMismatch,
    StateMismatch,
    EffectMismatch,
    PropertyPolicy,
    UndefinedOrder,
    InconsistentOrder,
}
/// A native snapshot error retaining any existing owning-boundary diagnostic.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectStackError {
    kind: EffectStackErrorKind,
    detail: StackFailure,
}
#[derive(Debug, Clone, PartialEq)]
enum StackFailure {
    None,
    Playback(PlaybackError),
    Timing(EffectTimingError),
}
impl EffectStackError {
    fn new(kind: EffectStackErrorKind) -> Self {
        Self {
            kind,
            detail: StackFailure::None,
        }
    }
    pub const fn kind(&self) -> EffectStackErrorKind {
        self.kind
    }
    pub const fn playback_error(&self) -> Option<&PlaybackError> {
        match &self.detail {
            StackFailure::Playback(error) => Some(error),
            _ => None,
        }
    }
    pub const fn timing_error(&self) -> Option<&EffectTimingError> {
        match &self.detail {
            StackFailure::Timing(error) => Some(error),
            _ => None,
        }
    }
}
impl From<PlaybackError> for EffectStackError {
    fn from(error: PlaybackError) -> Self {
        Self {
            kind: EffectStackErrorKind::Playback,
            detail: StackFailure::Playback(error),
        }
    }
}
impl From<EffectTimingError> for EffectStackError {
    fn from(error: EffectTimingError) -> Self {
        Self {
            kind: EffectStackErrorKind::Timing,
            detail: StackFailure::Timing(error),
        }
    }
}
impl fmt::Display for EffectStackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "effect stack: {:?}", self.kind)
    }
}
impl std::error::Error for EffectStackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.detail {
            StackFailure::None => None,
            StackFailure::Playback(error) => Some(error),
            StackFailure::Timing(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct StackEntry {
    input: AnimationStackInput,
    creation: CreationOrder,
    timing: Option<EffectTimingSample>,
}
impl StackEntry {
    fn targets(&self, property: &PropertyKey) -> bool {
        self.input.effect.as_ref().is_some_and(|effect| {
            !effect.not_animatable.contains(property)
                && effect
                    .keyframes
                    .frames()
                    .iter()
                    .any(|frame| frame.properties().iter().any(|(key, _)| key == property))
        })
    }
    fn replaceable(&self) -> bool {
        !self.input.prescribed_by_markup
            && self.input.state.play_state() == EffectPlayState::Finished
            && self.input.state.replace_state() != AnimationReplaceState::Removed
            && self
                .input
                .timeline
                .is_some_and(|timeline| timeline.monotonicity() == TimelineMonotonicity::Monotonic)
            && self.input.target.is_some()
            && self.timing.is_some_and(EffectTimingSample::is_in_effect)
    }
}

/// An immutable, validated composite order and one coherent replacement snapshot.
///
/// All IDs are checked against one records owner at construction. Later changes
/// to records, input values, or timelines do not rewrite this snapshot. Runtime
/// owns deciding when to replace a snapshot and publishing returned decisions.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectStackSnapshot {
    entries: Vec<StackEntry>,
    undefined: Vec<AnimationId>,
}

impl EffectStackSnapshot {
    /// Validates observations and constructs a total order without caller callbacks.
    ///
    /// Defined order is checked for every pair after bounded insertion. Invalid
    /// cyclic/non-total combinations return `InconsistentOrder`, never reach a
    /// library sort comparator, and never publish a partial snapshot.
    pub fn new(
        records: &AnimationRecords,
        inputs: Vec<AnimationStackInput>,
    ) -> Result<Self, EffectStackError> {
        let mut entries: Vec<StackEntry> = Vec::new();
        entries
            .try_reserve_exact(inputs.len())
            .map_err(|_| EffectStackError::new(EffectStackErrorKind::Capacity))?;
        let mut undefined = Vec::new();
        // Undefined idle observations participate in snapshot coherence checks
        // even though they do not participate in composite order.
        let mut observed: Vec<(Option<TimelineSnapshot>, Option<EffectTargetId>, bool)> =
            Vec::new();
        observed
            .try_reserve_exact(inputs.len())
            .map_err(|_| EffectStackError::new(EffectStackErrorKind::Capacity))?;
        for input in inputs {
            if entries
                .iter()
                .any(|entry| entry.input.animation == input.animation)
                || undefined.contains(&input.animation)
            {
                return Err(EffectStackError::new(
                    EffectStackErrorKind::DuplicateAnimation,
                ));
            }
            let record = records.snapshot(&input.animation, input.timeline)?;
            for (timeline, target, in_document) in &observed {
                if let (Some(first), Some(second)) = (*timeline, input.timeline)
                    && first.id() == second.id()
                    && first != second
                {
                    return Err(EffectStackError::new(
                        EffectStackErrorKind::TimelineMismatch,
                    ));
                }
                if target.is_some() && *target == input.target && *in_document != input.in_document
                {
                    return Err(EffectStackError::new(EffectStackErrorKind::TargetMismatch));
                }
            }
            observed.push((input.timeline, input.target, input.in_document));
            if input.state.timeline() != input.timeline.map(TimelineSnapshot::monotonicity) {
                return Err(EffectStackError::new(
                    EffectStackErrorKind::TimelineMismatch,
                ));
            }
            if matches!(
                (input.state.play_state(), input.local_time),
                (EffectPlayState::Idle, Some(_)) | (EffectPlayState::Finished, None)
            ) {
                return Err(EffectStackError::new(EffectStackErrorKind::StateMismatch));
            }
            let timing = match (record.effect(), &input.effect) {
                (None, None) => None,
                (Some(registered), Some(effect))
                    if registered.facts() == EffectFacts::from_timing(&effect.timing) =>
                {
                    for (index, property) in effect.not_animatable.iter().enumerate() {
                        if effect.not_animatable[..index].contains(property)
                            || !effect.keyframes.frames().iter().any(|frame| {
                                frame.properties().iter().any(|(key, _)| key == property)
                            })
                        {
                            return Err(EffectStackError::new(
                                EffectStackErrorKind::PropertyPolicy,
                            ));
                        }
                    }
                    Some(effect.timing.sample(EffectTimingInput::Associated {
                        local_time: input.local_time,
                        animation: input.state,
                    })?)
                }
                _ => return Err(EffectStackError::new(EffectStackErrorKind::EffectMismatch)),
            };
            if matches!(input.order, AnimationOrder::Undefined { .. }) {
                if input.state.play_state() != EffectPlayState::Idle {
                    return Err(EffectStackError::new(EffectStackErrorKind::UndefinedOrder));
                }
                undefined
                    .try_reserve(1)
                    .map_err(|_| EffectStackError::new(EffectStackErrorKind::Capacity))?;
                undefined.push(input.animation);
                continue;
            }
            let entry = StackEntry {
                input,
                creation: record.creation_order(),
                timing,
            };
            let position = entries
                .iter()
                .position(|existing| compare(&entry, existing).is_lt())
                .unwrap_or(entries.len());
            // The insertion index is a found slice position or its length.
            entries.insert(position, entry);
        }
        for (index, first) in entries.iter().enumerate() {
            for second in &entries[index + 1..] {
                if compare(first, second) != Ordering::Less {
                    return Err(EffectStackError::new(
                        EffectStackErrorKind::InconsistentOrder,
                    ));
                }
            }
        }
        Ok(Self { entries, undefined })
    }

    /// Iterates the defined order from lower to higher composite order.
    pub fn animations(&self) -> impl Iterator<Item = &AnimationId> {
        self.entries.iter().map(|entry| &entry.input.animation)
    }
    /// Iterates validated idle observations whose source defines no order.
    pub fn undefined_animations(&self) -> impl Iterator<Item = &AnimationId> {
        self.undefined.iter()
    }

    /// Folds only contributing, in-effect, nonremoved effects for this target/property.
    ///
    /// Each keyframe sampler receives the previous result as underlying. Its
    /// endpoint composition is already included; the result is not composed a
    /// second time. `None` means no contribution, not the base value's removal.
    pub fn sample<P: PropertyValueOperations + ?Sized>(
        &self,
        target: EffectTargetId,
        property: &PropertyKey,
        applicability: KeyframeTarget<'_>,
        operations: &P,
    ) -> Result<Option<CompositedProperty>, EffectStackSampleError<P::Error>> {
        let KeyframeTarget::Available(base) = applicability else {
            return Ok(None);
        };
        let mut underlying = base.clone();
        let mut top = None;
        for entry in &self.entries {
            if entry.input.target != Some(target)
                || entry.input.state.replace_state() == AnimationReplaceState::Removed
                || !entry.targets(property)
            {
                continue;
            }
            let (Some(effect), Some(timing)) = (&entry.input.effect, entry.timing) else {
                continue;
            };
            if !timing.is_in_effect() {
                continue;
            }
            let value = effect
                .keyframes
                .sample_timing(
                    property,
                    KeyframeSampleInput {
                        target: KeyframeTarget::Available(&underlying),
                        effect_composite: effect.composite,
                    },
                    timing,
                    operations,
                )
                .map_err(|error| EffectStackSampleError {
                    animation: entry.input.animation.clone(),
                    error,
                })?;
            if let Some(value) = value {
                underlying = value;
                top = Some(entry);
            }
        }
        Ok(top.map(|entry| CompositedProperty {
            value: underlying,
            animation: entry.input.animation.clone(),
            class: entry.input.order.class(),
        }))
    }

    /// Selects removals from this unchanged snapshot in ascending composite order.
    ///
    /// Coverage is per exact target/property and can come from several higher
    /// replaceable effects. Persisted effects cover lower effects but are never
    /// removal candidates. No predicate requires the covering operation to be
    /// `replace`. Empty targeted-property coverage is universally satisfied.
    pub fn replacements(&self) -> Result<Vec<ReplacementDecision>, EffectStackError> {
        let mut decisions = Vec::new();
        decisions
            .try_reserve_exact(self.entries.len())
            .map_err(|_| EffectStackError::new(EffectStackErrorKind::Capacity))?;
        for (index, entry) in self.entries.iter().enumerate() {
            if !entry.input.in_document
                || entry.input.state.replace_state() != AnimationReplaceState::Active
                || !entry.replaceable()
            {
                continue;
            }
            let Some(effect) = &entry.input.effect else {
                continue;
            };
            let covered = effect
                .keyframes
                .frames()
                .iter()
                .flat_map(|frame| frame.properties())
                .filter(|(key, _)| !effect.not_animatable.contains(key))
                .all(|(property, _)| {
                    self.entries[index + 1..].iter().any(|higher| {
                        higher.input.target == entry.input.target
                            && higher.replaceable()
                            && higher.targets(property)
                    })
                });
            if covered {
                decisions.push(ReplacementDecision {
                    animation: entry.input.animation.clone(),
                    current_time: entry.input.local_time,
                    timeline: entry.input.timeline,
                });
            }
        }
        Ok(decisions)
    }
}

fn compare(first: &StackEntry, second: &StackEntry) -> Ordering {
    if let (
        AnimationOrder::Defined {
            class: a,
            inter_class: inter_a,
            class_specific: within_a,
        },
        AnimationOrder::Defined {
            class: b,
            inter_class: inter_b,
            class_specific: within_b,
        },
    ) = (first.input.order, second.input.order)
    {
        let ranks = if a != b {
            inter_a.zip(inter_b).map(|(a, b)| a.cmp(&b))
        } else {
            within_a.zip(within_b).map(|(a, b)| a.cmp(&b))
        };
        if let Some(order) = ranks
            && order != Ordering::Equal
        {
            return order;
        }
    }
    first.creation.cmp(&second.creation)
}

/// A composed property with the highest contributing animation and its class.
#[derive(Debug, Clone, PartialEq)]
pub struct CompositedProperty {
    value: InterpolableValue,
    animation: AnimationId,
    class: AnimationClass,
}
impl CompositedProperty {
    pub const fn value(&self) -> &InterpolableValue {
        &self.value
    }
    pub const fn animation(&self) -> &AnimationId {
        &self.animation
    }
    pub const fn class(&self) -> AnimationClass {
        self.class
    }
}

/// A property sampling failure retaining the responsible animation and keyframe error.
#[derive(Debug, PartialEq)]
pub struct EffectStackSampleError<E> {
    animation: AnimationId,
    error: KeyframeSampleError<E>,
}
impl<E> EffectStackSampleError<E> {
    pub const fn animation(&self) -> &AnimationId {
        &self.animation
    }
    pub const fn keyframe_error(&self) -> &KeyframeSampleError<E> {
        &self.error
    }
}
impl<E: fmt::Display> fmt::Display for EffectStackSampleError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "effect stack sample: {}", self.error)
    }
}
impl<E: std::error::Error + 'static> std::error::Error for EffectStackSampleError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// An Active→Removed decision carrying queue-time coordinates for caller publication.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplacementDecision {
    animation: AnimationId,
    current_time: Option<AnimationTime>,
    timeline: Option<TimelineSnapshot>,
}
impl ReplacementDecision {
    pub const fn animation(&self) -> &AnimationId {
        &self.animation
    }
    pub const fn current_time(&self) -> Option<AnimationTime> {
        self.current_time
    }
    pub const fn timeline(&self) -> Option<TimelineSnapshot> {
        self.timeline
    }
    pub const fn replace_state(&self) -> AnimationReplaceState {
        AnimationReplaceState::Removed
    }
}
