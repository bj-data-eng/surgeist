//! Checked animation/effect ownership and immutable playback-coordinate observations.
//!
//! [`AnimationRecords`] owns one global creation-order list across its caller's
//! documents. Handles belong to that owner and removed slots are never reused.
//! Records begin with unresolved start/hold times and rate one. This boundary
//! supports idle association changes and explicit record retirement; playback
//! controls and deferred completion processing are not provided by this boundary.
//!
//! [`PlaybackCoordinates`] evaluates supplied immutable time state without
//! mutating a record or pretending to perform a seek/play-control procedure.
//! No clock, scheduler, document query, or Runtime dependency is involved.
//!
//! ```
//! use surgeist_animation::playback::{EffectFacts, PlaybackCoordinates};
//! use surgeist_animation::time::{PlaybackRate, TimelineTime};
//! use surgeist_animation::timeline::{AnimationSnapshot, TimelineId, TimelineMonotonicity, TimelineSnapshot};
//! use surgeist_animation::timing::{AnimationReplaceState, EffectTiming, EffectTimingOptions, PhaseProgress};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let timing = EffectTiming::try_new(EffectTimingOptions {
//!     duration_millis: 100.0, ..EffectTimingOptions::default()
//! })?;
//! let frame = TimelineSnapshot::new(TimelineId::new(1),
//!     Some(TimelineTime::from_millis(25.0)?), TimelineMonotonicity::Monotonic, None);
//! let coordinates = PlaybackCoordinates::new(AnimationSnapshot::new(
//!     PlaybackRate::new(1.0)?, Some(TimelineTime::from_millis(0.0)?), Some(frame)), None);
//! let observation = coordinates.observe(Some(EffectFacts::from_timing(&timing)), AnimationReplaceState::Active)?;
//! let sample = timing.sample(observation.effect_input())?;
//! assert_eq!(sample.simple_progress().map(PhaseProgress::value), Some(0.25));
//! # Ok(())
//! # }
//! ```
//!
//! ```compile_fail,E0451
//! use surgeist_animation::playback::AnimationId;
//! let forged = AnimationId { index: 0, owner: std::sync::Arc::new(()) };
//! ```

use core::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::time::{AnimationTime, PlaybackRate, TimeError};
use crate::timeline::{
    AnimationSnapshot, TimelineId, TimelineSnapshot, timeline_time_to_animation_time,
};
use crate::timing::{
    AnimationReplaceState, EffectAnimationState, EffectPlayState, EffectTiming, EffectTimingInput,
};

/// An opaque animation handle branded by its records owner.
#[derive(Debug, Clone)]
pub struct AnimationId {
    owner: Arc<()>,
    index: usize,
}

impl PartialEq for AnimationId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner) && self.index == other.index
    }
}
impl Eq for AnimationId {}
impl Hash for AnimationId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.owner).hash(state);
        self.index.hash(state);
    }
}

/// An opaque effect registration handle branded by its records owner.
#[derive(Debug, Clone)]
pub struct EffectId {
    owner: Arc<()>,
    index: usize,
}

impl PartialEq for EffectId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner) && self.index == other.index
    }
}
impl Eq for EffectId {}
impl Hash for EffectId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.owner).hash(state);
        self.index.hash(state);
    }
}

/// Immutable relative creation order within one global animation list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreationOrder(u64);

/// Whether an association request changed the owner graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssociationChange {
    Unchanged,
    Changed,
}

/// Validated associated-effect facts used for playback-state observation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectFacts {
    end: f64,
}

impl EffectFacts {
    /// Validates a nonnegative finite or positive-infinite effect end in milliseconds.
    pub fn try_new(end: f64) -> Result<Self, PlaybackError> {
        if end.is_nan() || end < 0.0 {
            return Err(PlaybackError::new(PlaybackErrorKind::InvalidEffectEnd {
                value: end,
            }));
        }
        Ok(Self { end })
    }
    pub const fn from_timing(timing: &EffectTiming) -> Self {
        Self {
            end: timing.end_time_millis(),
        }
    }
    pub const fn end_time_millis(self) -> f64 {
        self.end
    }
}

/// A native ownership, supplied-context, or checked arithmetic error.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackError {
    kind: PlaybackErrorKind,
}

/// Errors distinguish stale/foreign handles, missing facts and numeric limits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackErrorKind {
    ForeignAnimation,
    ForeignEffect,
    StaleAnimation,
    StaleEffect,
    EffectInUse,
    IdentityExhausted,
    Capacity,
    MissingTimeline {
        expected: TimelineId,
    },
    UnexpectedTimeline {
        actual: TimelineId,
    },
    TimelineMismatch {
        expected: TimelineId,
        actual: TimelineId,
    },
    InvalidEffectEnd {
        value: f64,
    },
    Arithmetic(TimeError),
}

impl PlaybackError {
    const fn new(kind: PlaybackErrorKind) -> Self {
        Self { kind }
    }
    pub const fn kind(&self) -> &PlaybackErrorKind {
        &self.kind
    }
}
impl From<TimeError> for PlaybackError {
    fn from(error: TimeError) -> Self {
        Self::new(PlaybackErrorKind::Arithmetic(error))
    }
}
impl core::fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "playback model error: {:?}", self.kind)
    }
}
impl std::error::Error for PlaybackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            PlaybackErrorKind::Arithmetic(error) => Some(error),
            _ => None,
        }
    }
}

/// Immutable supplied start/rate/timeline state plus optional held animation time.
///
/// Both start and hold may be resolved. Hold takes precedence even with an
/// inactive/missing timeline or otherwise unrepresentable arithmetic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackCoordinates {
    animation: AnimationSnapshot,
    hold: Option<AnimationTime>,
}

impl PlaybackCoordinates {
    pub const fn new(animation: AnimationSnapshot, hold: Option<AnimationTime>) -> Self {
        Self { animation, hold }
    }
    pub const fn animation(self) -> AnimationSnapshot {
        self.animation
    }
    pub const fn hold_time(self) -> Option<AnimationTime> {
        self.hold
    }
    pub fn current_time(self) -> Result<Option<AnimationTime>, PlaybackError> {
        if self.hold.is_some() {
            return Ok(self.hold);
        }
        Ok(timeline_time_to_animation_time(
            self.animation
                .timeline()
                .and_then(TimelineSnapshot::current_time),
            self.animation,
        )?)
    }
    /// Derives the ordered play-state predicates for supplied state with no pending task.
    ///
    /// This observes coordinates; it does not clamp playback, update finished
    /// state, settle completions, or execute a playback operation.
    pub fn observe(
        self,
        effect: Option<EffectFacts>,
        replace: AnimationReplaceState,
    ) -> Result<PlaybackObservation, PlaybackError> {
        let current = self.current_time()?;
        let end = effect.map_or(0.0, EffectFacts::end_time_millis);
        let rate = self.animation.playback_rate();
        let play = if current.is_none() && self.animation.start_time().is_none() {
            EffectPlayState::Idle
        } else if self.animation.start_time().is_none() {
            EffectPlayState::Paused
        } else if current.is_some_and(|time| {
            (rate.value() > 0.0 && time.as_millis() >= end)
                || (rate.value() < 0.0 && time.as_millis() <= 0.0)
        }) {
            EffectPlayState::Finished
        } else {
            EffectPlayState::Running
        };
        let animation = EffectAnimationState::new(
            rate,
            play,
            self.animation
                .timeline()
                .map(TimelineSnapshot::monotonicity),
            replace,
        );
        Ok(PlaybackObservation {
            current,
            animation,
            has_effect: effect.is_some(),
        })
    }
}

/// A coherent no-pending-task observation suitable for effect sampling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackObservation {
    current: Option<AnimationTime>,
    animation: EffectAnimationState,
    has_effect: bool,
}

impl PlaybackObservation {
    pub const fn current_time(self) -> Option<AnimationTime> {
        self.current
    }
    pub const fn play_state(self) -> EffectPlayState {
        self.animation.play_state()
    }
    pub const fn effect_input(self) -> EffectTimingInput {
        if self.has_effect {
            EffectTimingInput::Associated {
                local_time: self.current,
                animation: self.animation,
            }
        } else {
            EffectTimingInput::Unassociated
        }
    }
}

/// A captured effect registration and its validated timing facts.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectSnapshot {
    id: EffectId,
    facts: EffectFacts,
}

impl EffectSnapshot {
    pub const fn id(&self) -> &EffectId {
        &self.id
    }
    pub const fn facts(&self) -> EffectFacts {
        self.facts
    }
}

/// An immutable owner-checked association and coordinate snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationRecordSnapshot {
    id: AnimationId,
    order: CreationOrder,
    effect: Option<EffectSnapshot>,
    coordinates: PlaybackCoordinates,
}

impl AnimationRecordSnapshot {
    pub const fn id(&self) -> &AnimationId {
        &self.id
    }
    pub const fn creation_order(&self) -> CreationOrder {
        self.order
    }
    pub const fn effect(&self) -> Option<&EffectSnapshot> {
        self.effect.as_ref()
    }
    pub const fn coordinates(&self) -> PlaybackCoordinates {
        self.coordinates
    }
    pub fn timeline(&self) -> Option<TimelineId> {
        self.coordinates
            .animation
            .timeline()
            .map(TimelineSnapshot::id)
    }
    pub fn observation(&self) -> Result<PlaybackObservation, PlaybackError> {
        self.coordinates.observe(
            self.effect.as_ref().map(EffectSnapshot::facts),
            AnimationReplaceState::Active,
        )
    }
}

#[derive(Debug)]
struct AnimationRecord {
    order: CreationOrder,
    timeline: Option<TimelineId>,
    effect: Option<EffectId>,
    coordinates: PlaybackCoordinates,
}
#[derive(Debug)]
struct EffectRecord {
    facts: EffectFacts,
    animation: Option<AnimationId>,
}

/// One creation-order universe with unique effect ownership and checked handles.
///
/// Registered records remain initially idle. Association changes are therefore
/// complete for every state constructible here: there are no pending tasks or
/// finished completions to reschedule. Coordinate evaluation is independent and
/// immutable. Retirement removes registration, not a runtime animation or DOM
/// object; the caller owns its lifecycle. Handles never resurrect after removal.
#[derive(Debug)]
pub struct AnimationRecords {
    owner: Arc<()>,
    animations: Vec<Option<AnimationRecord>>,
    effects: Vec<Option<EffectRecord>>,
    next_order: u64,
}

impl Default for AnimationRecords {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationRecords {
    pub fn new() -> Self {
        Self {
            owner: Arc::new(()),
            animations: Vec::new(),
            effects: Vec::new(),
            next_order: 0,
        }
    }
    pub fn register_effect(&mut self, facts: EffectFacts) -> Result<EffectId, PlaybackError> {
        self.effects
            .try_reserve(1)
            .map_err(|_| PlaybackError::new(PlaybackErrorKind::Capacity))?;
        let id = EffectId {
            owner: self.owner.clone(),
            index: self.effects.len(),
        };
        self.effects.push(Some(EffectRecord {
            facts,
            animation: None,
        }));
        Ok(id)
    }
    pub fn create(
        &mut self,
        timeline: Option<TimelineId>,
        effect: Option<&EffectId>,
    ) -> Result<AnimationId, PlaybackError> {
        if let Some(effect) = effect {
            self.effect(effect)?;
        }
        let next = self
            .next_order
            .checked_add(1)
            .ok_or_else(|| PlaybackError::new(PlaybackErrorKind::IdentityExhausted))?;
        let coordinates = PlaybackCoordinates::new(
            AnimationSnapshot::new(PlaybackRate::new(1.0)?, None, None),
            None,
        );
        self.animations
            .try_reserve(1)
            .map_err(|_| PlaybackError::new(PlaybackErrorKind::Capacity))?;
        let id = AnimationId {
            owner: self.owner.clone(),
            index: self.animations.len(),
        };
        self.animations.push(Some(AnimationRecord {
            order: CreationOrder(self.next_order),
            timeline,
            effect: None,
            coordinates,
        }));
        self.next_order = next;
        self.apply_effect(&id, None, effect);
        Ok(id)
    }
    /// Iterates live animation identities in their original global creation order.
    pub fn animations(&self) -> impl Iterator<Item = AnimationId> + '_ {
        self.animations
            .iter()
            .enumerate()
            .filter_map(|(index, record)| {
                record.as_ref().map(|_| AnimationId {
                    owner: self.owner.clone(),
                    index,
                })
            })
    }
    pub fn assign_timeline(
        &mut self,
        id: &AnimationId,
        timeline: Option<TimelineId>,
    ) -> Result<AssociationChange, PlaybackError> {
        if self.animation(id)?.timeline == timeline {
            return Ok(AssociationChange::Unchanged);
        }
        for (index, record) in self.animations.iter_mut().enumerate() {
            if index == id.index
                && let Some(record) = record
            {
                record.timeline = timeline;
            }
        }
        Ok(AssociationChange::Changed)
    }
    pub fn assign_effect(
        &mut self,
        id: &AnimationId,
        effect: Option<&EffectId>,
    ) -> Result<AssociationChange, PlaybackError> {
        let old = self.animation(id)?.effect.clone();
        if let Some(effect) = effect {
            self.effect(effect)?;
        }
        if old.as_ref() == effect {
            return Ok(AssociationChange::Unchanged);
        }
        self.apply_effect(id, old.as_ref(), effect);
        Ok(AssociationChange::Changed)
    }
    pub fn effect_owner(&self, id: &EffectId) -> Result<Option<AnimationId>, PlaybackError> {
        Ok(self.effect(id)?.animation.clone())
    }
    pub fn snapshot(
        &self,
        id: &AnimationId,
        timeline: Option<TimelineSnapshot>,
    ) -> Result<AnimationRecordSnapshot, PlaybackError> {
        let record = self.animation(id)?;
        match (record.timeline, timeline) {
            (Some(expected), None) => {
                return Err(PlaybackError::new(PlaybackErrorKind::MissingTimeline {
                    expected,
                }));
            }
            (None, Some(actual)) => {
                return Err(PlaybackError::new(PlaybackErrorKind::UnexpectedTimeline {
                    actual: actual.id(),
                }));
            }
            (Some(expected), Some(actual)) if expected != actual.id() => {
                return Err(PlaybackError::new(PlaybackErrorKind::TimelineMismatch {
                    expected,
                    actual: actual.id(),
                }));
            }
            _ => {}
        }
        let effect = record
            .effect
            .as_ref()
            .map(|id| {
                self.effect(id).map(|record| EffectSnapshot {
                    id: id.clone(),
                    facts: record.facts,
                })
            })
            .transpose()?;
        let coordinates = PlaybackCoordinates::new(
            AnimationSnapshot::new(
                record.coordinates.animation.playback_rate(),
                record.coordinates.animation.start_time(),
                timeline,
            ),
            record.coordinates.hold,
        );
        Ok(AnimationRecordSnapshot {
            id: id.clone(),
            order: record.order,
            effect,
            coordinates,
        })
    }
    pub fn remove_animation(&mut self, id: &AnimationId) -> Result<(), PlaybackError> {
        self.animation(id)?;
        for record in self.effects.iter_mut().flatten() {
            if record.animation.as_ref() == Some(id) {
                record.animation = None;
            }
        }
        for (index, record) in self.animations.iter_mut().enumerate() {
            if index == id.index {
                *record = None;
            }
        }
        Ok(())
    }
    pub fn remove_effect(&mut self, id: &EffectId) -> Result<(), PlaybackError> {
        if self.effect(id)?.animation.is_some() {
            return Err(PlaybackError::new(PlaybackErrorKind::EffectInUse));
        }
        for (index, record) in self.effects.iter_mut().enumerate() {
            if index == id.index {
                *record = None;
            }
        }
        Ok(())
    }
    fn animation(&self, id: &AnimationId) -> Result<&AnimationRecord, PlaybackError> {
        if !Arc::ptr_eq(&self.owner, &id.owner) {
            return Err(PlaybackError::new(PlaybackErrorKind::ForeignAnimation));
        }
        self.animations
            .get(id.index)
            .and_then(Option::as_ref)
            .ok_or_else(|| PlaybackError::new(PlaybackErrorKind::StaleAnimation))
    }
    fn effect(&self, id: &EffectId) -> Result<&EffectRecord, PlaybackError> {
        if !Arc::ptr_eq(&self.owner, &id.owner) {
            return Err(PlaybackError::new(PlaybackErrorKind::ForeignEffect));
        }
        self.effects
            .get(id.index)
            .and_then(Option::as_ref)
            .ok_or_else(|| PlaybackError::new(PlaybackErrorKind::StaleEffect))
    }
    // Called only after every external handle and fallible creation step is checked.
    // Iteration avoids indexing panic paths while updating both directions atomically.
    fn apply_effect(
        &mut self,
        animation: &AnimationId,
        old: Option<&EffectId>,
        new: Option<&EffectId>,
    ) {
        for record in self.animations.iter_mut().flatten() {
            if new.is_some() && record.effect.as_ref() == new {
                record.effect = None;
            }
        }
        for (index, record) in self.animations.iter_mut().enumerate() {
            if index == animation.index
                && let Some(record) = record
            {
                record.effect = new.cloned();
            }
        }
        for (index, record) in self.effects.iter_mut().enumerate() {
            if let Some(record) = record {
                if old.is_some_and(|id| id.index == index) {
                    record.animation = None;
                }
                if new.is_some_and(|id| id.index == index) {
                    record.animation = Some(animation.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_creation_order_preserves_existing_ownership() {
        let mut records = AnimationRecords::new();
        let effect = records
            .register_effect(EffectFacts::try_new(10.0).unwrap())
            .unwrap();
        let animation = records.create(None, Some(&effect)).unwrap();
        let before = records.snapshot(&animation, None).unwrap();
        records.next_order = u64::MAX;
        assert_eq!(
            records.create(None, Some(&effect)).unwrap_err().kind(),
            &PlaybackErrorKind::IdentityExhausted
        );
        assert_eq!(records.snapshot(&animation, None).unwrap(), before);
        assert_eq!(records.effect_owner(&effect).unwrap(), Some(animation));
    }
}
