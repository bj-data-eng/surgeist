//! Pure CSS animation-list reconciliation over caller-resolved snapshots.
//!
//! Style supplies used timing, exact resolved names, and the winning rule payload
//! for each list slot. A matched payload may be an empty valid rule. Runtime
//! supplies immutable prior instances with coherent playback coordinates from
//! one update. This module neither resolves rules nor mutates playback records.
//!
//! New slots match the last remaining old occurrence, scanning last to first.
//! Rule identity/revision changes never participate in matching. Retained plans
//! sample old and new timing at the same playback coordinates and request CSS
//! play-state changes only when that property changed. The playback owner applies
//! those requests; no second playback state machine or clock is introduced here.
//!
//! ```
//! use surgeist_animation::css_animation::{CssAnimationEligibility, CssAnimationList,
//!     CssAnimationName, CssAnimationSlot, ResolvedCssAnimation};
//! use surgeist_animation::keyframe::KeyframeTiming;
//! use surgeist_animation::time::{AnimationDelay, AnimationDuration};
//! use surgeist_animation::timing::{AnimationDirection, AnimationPlayState,
//!     FillMode, IterationCount, TimingPhase};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let timing = KeyframeTiming::try_new(AnimationDelay::from_secs(1.0)?,
//!     AnimationDuration::from_secs(2.0)?, IterationCount::finite(1.0)?,
//!     AnimationDirection::Normal, FillMode::Both)?;
//! // () can represent an empty valid matched rule. Unmatched is a separate slot.
//! let matched = ResolvedCssAnimation::new(CssAnimationName::new("pulse"), (),
//!     timing, AnimationPlayState::Running)?;
//! let next = CssAnimationList::new(vec![CssAnimationSlot::Matched(matched)],
//!     CssAnimationEligibility::Eligible)?;
//! let plan = next.reconcile(&[])?;
//! assert!(plan.updates()[0].previous().is_none());
//! assert_eq!(plan.updates()[0].after().phase(), TimingPhase::Before);
//! # Ok(())
//! # }
//! ```

use std::collections::{HashMap, HashSet};

use crate::keyframe::KeyframeTiming;
use crate::playback::{AnimationId, EffectFacts, PlaybackCoordinates, PlaybackError};
use crate::time::{AnimationTime, PlaybackRate, TimeError};
use crate::timing::{
    AnimationPlayState, AnimationReplaceState, EffectAnimationState, EffectPlayState, EffectTiming,
    EffectTimingError, EffectTimingInput, EffectTimingOptions, EffectTimingSample,
};

/// An exact resolved CSS name, including empty or whitespace string names.
///
/// String and identifier syntax with the same value use the same name. Syntax
/// decoding belongs to CSS. Case and codepoints are retained. This is neither
/// the `none` keyword slot nor the native playback instance's [`AnimationId`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CssAnimationName(Box<str>);
impl CssAnimationName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into().into_boxed_str())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A matched rule payload and its validated used CSS animation properties.
///
/// `R` is caller-owned rule identity/revision and computed-keyframe data. No
/// equality, nonempty-property requirement, or CSS dependency is imposed on it.
/// CSS interval easing belongs to that payload, so effect timing is linear.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCssAnimation<R> {
    name: CssAnimationName,
    rule: R,
    timing: KeyframeTiming,
    play_state: AnimationPlayState,
    effect: EffectTiming,
}
impl<R> ResolvedCssAnimation<R> {
    /// Checks conversion from CSS seconds to the native millisecond kernel.
    pub fn new(
        name: CssAnimationName,
        rule: R,
        timing: KeyframeTiming,
        play_state: AnimationPlayState,
    ) -> Result<Self, CssAnimationError> {
        let delay = timing.delay().as_secs() * 1000.0;
        let duration = timing.duration().as_secs() * 1000.0;
        if !delay.is_finite() || !duration.is_finite() {
            return Err(CssAnimationError::new(
                CssAnimationErrorKind::UnrepresentableMilliseconds,
            ));
        }
        let effect = EffectTiming::try_new(EffectTimingOptions {
            start_delay_millis: delay,
            duration_millis: duration,
            iterations: timing.iterations(),
            direction: timing.direction(),
            fill: timing.fill_mode(),
            ..EffectTimingOptions::default()
        })?;
        Ok(Self {
            name,
            rule,
            timing,
            play_state,
            effect,
        })
    }
    pub const fn name(&self) -> &CssAnimationName {
        &self.name
    }
    pub const fn rule(&self) -> &R {
        &self.rule
    }
    pub const fn timing(&self) -> KeyframeTiming {
        self.timing
    }
    pub const fn play_state(&self) -> AnimationPlayState {
        self.play_state
    }
    pub const fn effect_timing(&self) -> &EffectTiming {
        &self.effect
    }

    fn sample(
        &self,
        coordinates: PlaybackCoordinates,
    ) -> Result<EffectTimingSample, CssAnimationError> {
        let observation = coordinates.observe(
            Some(EffectFacts::from_timing(&self.effect)),
            AnimationReplaceState::Active,
        )?;
        Ok(self.effect.sample(observation.effect_input())?)
    }
    fn initial_sample(&self) -> Result<EffectTimingSample, CssAnimationError> {
        Ok(self.effect.sample(EffectTimingInput::Associated {
            local_time: Some(AnimationTime::from_millis(0.0)?),
            animation: EffectAnimationState::new(
                PlaybackRate::new(1.0)?,
                match self.play_state {
                    AnimationPlayState::Running => EffectPlayState::Running,
                    AnimationPlayState::Paused => EffectPlayState::Paused,
                },
                None,
                AnimationReplaceState::Active,
            ),
        })?)
    }
}

/// One effective animation-name position, after list repetition and truncation.
#[derive(Debug, Clone, PartialEq)]
pub enum CssAnimationSlot<R> {
    /// The CSS keyword, distinct from a string whose resolved value is `none`.
    None,
    /// A name for which no valid matching rule currently resolves.
    Unmatched(CssAnimationName),
    /// A valid match; an empty rule remains a match.
    Matched(ResolvedCssAnimation<R>),
}
impl<R> CssAnimationSlot<R> {
    fn match_presence(&self) -> Option<(&CssAnimationName, bool)> {
        match self {
            Self::None => None,
            Self::Unmatched(name) => Some((name, false)),
            Self::Matched(animation) => Some((animation.name(), true)),
        }
    }
}

/// Caller-selected applicability, including the optional noninteractive-media policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssAnimationEligibility {
    Eligible,
    ElementDisplayNone,
    AncestorDisplayNone,
    IgnoredMedia,
}

/// A coherent resolved list for one target at one native style update.
#[derive(Debug, Clone, PartialEq)]
pub struct CssAnimationList<R> {
    slots: Box<[CssAnimationSlot<R>]>,
    eligibility: CssAnimationEligibility,
}
impl<R> CssAnimationList<R> {
    /// Rejects contradictory matching-rule availability for the same name.
    pub fn new(
        slots: Vec<CssAnimationSlot<R>>,
        eligibility: CssAnimationEligibility,
    ) -> Result<Self, CssAnimationError> {
        let mut names = HashMap::new();
        for (index, slot) in slots.iter().enumerate() {
            if let Some((name, present)) = slot.match_presence() {
                if let Some(&(first, previous)) = names.get(name) {
                    if present != previous {
                        return Err(CssAnimationError::new(
                            CssAnimationErrorKind::InconsistentRuleMatch { first },
                        )
                        .at_new(index));
                    }
                } else {
                    names.insert(name, (index, present));
                }
            }
        }
        Ok(Self {
            slots: slots.into_boxed_slice(),
            eligibility,
        })
    }
    pub fn slots(&self) -> &[CssAnimationSlot<R>] {
        &self.slots
    }
    pub const fn eligibility(&self) -> CssAnimationEligibility {
        self.eligibility
    }

    /// Plans retention, starts, and stops without mutating either snapshot.
    ///
    /// `previous` is in prior animation-name order, including finished instances
    /// still applied to this target. Runtime omits instances after publishing a
    /// stop and allocates new identities for starts. Updates are returned in new
    /// slot order and stops in old instance order; this is not event-dispatch order.
    ///
    /// Samples compare timing before playback requests are applied. Retention
    /// preserves the exact supplied start/hold/rate/timeline coordinates. Runtime
    /// applies any play-state request through its existing playback procedures,
    /// publishes coherently, and schedules/resamples from that published state.
    ///
    /// # Errors
    /// Duplicate old identities, playback arithmetic, or new timing sampling
    /// failure returns no partial plan. Unresolved time remains unresolved.
    pub fn reconcile<'a>(
        &'a self,
        previous: &'a [CssAnimationInstance<R>],
    ) -> Result<CssAnimationPlan<'a, R>, CssAnimationError> {
        let mut identities = HashMap::new();
        for (index, instance) in previous.iter().enumerate() {
            if let Some(first) = identities.insert(instance.id(), index) {
                return Err(
                    CssAnimationError::new(CssAnimationErrorKind::DuplicateAnimation { first })
                        .at_old(index),
                );
            }
        }
        let mut remaining: Vec<_> = previous.iter().enumerate().map(Some).collect();
        let mut updates = Vec::new();
        if self.eligibility == CssAnimationEligibility::Eligible {
            for (new_index, slot) in self.slots.iter().enumerate().rev() {
                let CssAnimationSlot::Matched(animation) = slot else {
                    continue;
                };
                let matched = remaining
                    .iter_mut()
                    .rev()
                    .find(|entry| {
                        entry
                            .as_ref()
                            .is_some_and(|(_, instance)| instance.animation.name == animation.name)
                    })
                    .and_then(Option::take);
                let after = match matched {
                    Some((_, instance)) => animation.sample(instance.coordinates),
                    None => animation.initial_sample(),
                }
                .map_err(|error| error.at_new(new_index))?;
                updates.push(CssAnimationUpdate {
                    new_index,
                    animation,
                    previous: matched,
                    after,
                });
            }
        }
        updates.reverse();
        let unmatched_names: HashSet<_> = self
            .slots
            .iter()
            .filter_map(|slot| match slot {
                CssAnimationSlot::Unmatched(name) => Some(name),
                _ => None,
            })
            .collect();
        let stops = remaining
            .into_iter()
            .flatten()
            .map(|(old_index, instance)| {
                let reason = if self.eligibility != CssAnimationEligibility::Eligible {
                    CssAnimationStopReason::Ineligible(self.eligibility)
                } else if unmatched_names.contains(instance.animation.name()) {
                    CssAnimationStopReason::MissingRule
                } else {
                    CssAnimationStopReason::NameRemoved
                };
                CssAnimationStop {
                    old_index,
                    instance,
                    reason,
                }
            })
            .collect();
        Ok(CssAnimationPlan {
            updates: updates.into_boxed_slice(),
            stops,
        })
    }
}

/// One immutable applied instance with a validated coherent timing observation.
#[derive(Debug, Clone, PartialEq)]
pub struct CssAnimationInstance<R> {
    id: AnimationId,
    animation: ResolvedCssAnimation<R>,
    coordinates: PlaybackCoordinates,
    sample: EffectTimingSample,
}
impl<R> CssAnimationInstance<R> {
    pub fn new(
        id: AnimationId,
        animation: ResolvedCssAnimation<R>,
        coordinates: PlaybackCoordinates,
    ) -> Result<Self, CssAnimationError> {
        let sample = animation.sample(coordinates)?;
        Ok(Self {
            id,
            animation,
            coordinates,
            sample,
        })
    }
    pub const fn id(&self) -> &AnimationId {
        &self.id
    }
    pub const fn animation(&self) -> &ResolvedCssAnimation<R> {
        &self.animation
    }
    pub const fn coordinates(&self) -> PlaybackCoordinates {
        self.coordinates
    }
    pub const fn sample(&self) -> EffectTimingSample {
        self.sample
    }
}

/// A complete immutable publication plan borrowing its before/after snapshots.
#[derive(Debug)]
pub struct CssAnimationPlan<'a, R> {
    updates: Box<[CssAnimationUpdate<'a, R>]>,
    stops: Box<[CssAnimationStop<'a, R>]>,
}
impl<'a, R> CssAnimationPlan<'a, R> {
    pub fn updates(&self) -> &[CssAnimationUpdate<'a, R>] {
        &self.updates
    }
    pub fn stops(&self) -> &[CssAnimationStop<'a, R>] {
        &self.stops
    }
}

/// A new slot's start or retained-instance update and coherent timing samples.
#[derive(Debug)]
pub struct CssAnimationUpdate<'a, R> {
    new_index: usize,
    animation: &'a ResolvedCssAnimation<R>,
    previous: Option<(usize, &'a CssAnimationInstance<R>)>,
    after: EffectTimingSample,
}
impl<'a, R> CssAnimationUpdate<'a, R> {
    pub const fn new_index(&self) -> usize {
        self.new_index
    }
    pub const fn animation(&self) -> &'a ResolvedCssAnimation<R> {
        self.animation
    }
    pub fn old_index(&self) -> Option<usize> {
        self.previous.map(|(index, _)| index)
    }
    /// Returns `None` for a start, whose identity is allocated on publication.
    pub fn previous(&self) -> Option<&'a CssAnimationInstance<R>> {
        self.previous.map(|(_, instance)| instance)
    }
    pub fn before(&self) -> Option<EffectTimingSample> {
        self.previous().map(CssAnimationInstance::sample)
    }
    pub const fn after(&self) -> EffectTimingSample {
        self.after
    }
    /// Requests playback work only for a start or changed CSS play-state value.
    pub fn play_state_change(&self) -> Option<AnimationPlayState> {
        match self.previous() {
            Some(old) if old.animation.play_state == self.animation.play_state => None,
            _ => Some(self.animation.play_state),
        }
    }
}

/// Why an old applied instance no longer belongs to the resolved list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssAnimationStopReason {
    NameRemoved,
    MissingRule,
    Ineligible(CssAnimationEligibility),
}

/// A removed instance and its final captured timing for lifecycle consumers.
#[derive(Debug)]
pub struct CssAnimationStop<'a, R> {
    old_index: usize,
    instance: &'a CssAnimationInstance<R>,
    reason: CssAnimationStopReason,
}
impl<'a, R> CssAnimationStop<'a, R> {
    pub const fn old_index(&self) -> usize {
        self.old_index
    }
    pub const fn instance(&self) -> &'a CssAnimationInstance<R> {
        self.instance
    }
    pub const fn reason(&self) -> CssAnimationStopReason {
        self.reason
    }
    pub const fn sample(&self) -> EffectTimingSample {
        self.instance.sample
    }
}

/// A malformed snapshot or checked conversion/sampling failure.
#[derive(Debug, Clone, PartialEq)]
pub struct CssAnimationError {
    kind: CssAnimationErrorKind,
    old_index: Option<usize>,
    new_index: Option<usize>,
}
/// Semantic failure categories with original typed arithmetic causes.
#[derive(Debug, Clone, PartialEq)]
pub enum CssAnimationErrorKind {
    DuplicateAnimation { first: usize },
    InconsistentRuleMatch { first: usize },
    UnrepresentableMilliseconds,
    Playback(PlaybackError),
    Timing(EffectTimingError),
    Time(TimeError),
}
impl CssAnimationError {
    const fn new(kind: CssAnimationErrorKind) -> Self {
        Self {
            kind,
            old_index: None,
            new_index: None,
        }
    }
    fn at_old(mut self, index: usize) -> Self {
        self.old_index = Some(index);
        self
    }
    fn at_new(mut self, index: usize) -> Self {
        self.new_index = Some(index);
        self
    }
    pub const fn kind(&self) -> &CssAnimationErrorKind {
        &self.kind
    }
    pub const fn old_index(&self) -> Option<usize> {
        self.old_index
    }
    pub const fn new_index(&self) -> Option<usize> {
        self.new_index
    }
}
impl From<PlaybackError> for CssAnimationError {
    fn from(error: PlaybackError) -> Self {
        Self::new(CssAnimationErrorKind::Playback(error))
    }
}
impl From<EffectTimingError> for CssAnimationError {
    fn from(error: EffectTimingError) -> Self {
        Self::new(CssAnimationErrorKind::Timing(error))
    }
}
impl From<TimeError> for CssAnimationError {
    fn from(error: TimeError) -> Self {
        Self::new(CssAnimationErrorKind::Time(error))
    }
}
impl core::fmt::Display for CssAnimationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CSS animation reconciliation {:?}", self.kind)?;
        if let Some(index) = self.old_index {
            write!(f, " at old instance {index}")?;
        }
        if let Some(index) = self.new_index {
            write!(f, " at new slot {index}")?;
        }
        Ok(())
    }
}
impl std::error::Error for CssAnimationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            CssAnimationErrorKind::Playback(error) => Some(error),
            CssAnimationErrorKind::Timing(error) => Some(error),
            CssAnimationErrorKind::Time(error) => Some(error),
            _ => None,
        }
    }
}
