//! Immutable CSS transition decisions over caller-supplied document facts.

use core::fmt;
use std::sync::Arc;

use crate::diagnostics::NumericError;
use crate::easing::{EasedProgress, Easing};
use crate::interpolation::{
    InterpolableValue, InterpolationError, InterpolationOutcome, InterpolationProgress, PropertyKey,
};
use crate::progress::{NormalizedProgress, UnitRatio};
use crate::time::{AnimationDelay, AnimationDuration, OriginRelativeTime, TimeError};

use super::TransitionTrackId;

/// Identifies one caller-owned target and canonical longhand property.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransitionKey<T> {
    target: T,
    property: PropertyKey,
}

impl<T> TransitionKey<T> {
    /// Combines a caller identity with its already-normalized longhand.
    /// Target identities and their equality remain stable while records exist.
    pub fn new(target: T, property: PropertyKey) -> Self {
        Self { target, property }
    }
    pub const fn target(&self) -> &T {
        &self.target
    }
    pub const fn property(&self) -> &PropertyKey {
        &self.property
    }
}

/// Holds the used after-change timing selected by Style.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionTiming {
    duration: AnimationDuration,
    delay: AnimationDelay,
    easing: Easing,
}

impl TransitionTiming {
    pub fn new(duration: AnimationDuration, delay: AnimationDelay, easing: Easing) -> Self {
        Self {
            duration,
            delay,
            easing,
        }
    }
    pub const fn duration(&self) -> AnimationDuration {
        self.duration
    }
    pub const fn delay(&self) -> AnimationDelay {
        self.delay
    }
    pub const fn easing(&self) -> &Easing {
        &self.easing
    }

    fn has_positive_combined_duration(&self) -> bool {
        // Avoid overflowing a sum when only its sign is required.
        if self.delay.as_secs() < 0.0 {
            self.duration.as_secs() > -self.delay.as_secs()
        } else {
            self.duration.as_secs() > 0.0 || self.delay.as_secs() > 0.0
        }
    }
}

/// Supplies coherent before/after values and the last matching used timing.
///
/// Style owns eligibility (presence in both document style events), computed
/// values, declarative-animation advancement, and longhand/list matching.
/// Runtime supplies a fresh `new_id` for each new run of this key, including
/// restarts and reversals.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionChange<T> {
    pub key: TransitionKey<T>,
    pub before: InterpolableValue,
    pub after: InterpolableValue,
    pub eligible: bool,
    pub matching: Option<TransitionTiming>,
    pub new_id: TransitionTrackId,
}

/// Classifies a particular computed-value pair, rather than a property alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transitionability {
    Continuous,
    Discrete,
    NotAnimatable,
}

/// Supplies property-specific pair compatibility and interpolation algorithms.
///
/// Implementations use the property's selected animation type, including
/// pair-specific discrete cases. For fixed inputs these operations must be
/// deterministic. They must not publish style or dispatch events. Unavailable
/// interpolation returns its typed unsupported outcome; it is not evidence
/// that an otherwise animatable property is not animatable.
pub trait TransitionPropertyPolicy {
    fn transitionability(
        &self,
        property: &PropertyKey,
        from: &InterpolableValue,
        to: &InterpolableValue,
    ) -> Transitionability;
    fn interpolate(
        &self,
        property: &PropertyKey,
        from: &InterpolableValue,
        to: &InterpolableValue,
        progress: InterpolationProgress,
    ) -> InterpolationOutcome;
}

/// Retains one validated running record and its original used timing.
#[derive(Debug, Clone, PartialEq)]
pub struct RunningTransition<T> {
    data: Arc<RunningData<T>>,
}

#[derive(Debug, PartialEq)]
struct RunningData<T> {
    key: TransitionKey<T>,
    id: TransitionTrackId,
    created_at: OriginRelativeTime,
    start: OriginRelativeTime,
    end: OriginRelativeTime,
    from: InterpolableValue,
    to: InterpolableValue,
    reversing_start: InterpolableValue,
    shortening: UnitRatio,
    timing: TransitionTiming,
    duration: AnimationDuration,
    delay: AnimationDelay,
}

/// Retains a completed target for suppression and completion event metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct CompletedTransition<T> {
    key: TransitionKey<T>,
    id: TransitionTrackId,
    end_value: InterpolableValue,
    end_time: OriginRelativeTime,
    duration: AnimationDuration,
    delay: AnimationDelay,
}

/// Stores exactly one lifecycle state for an owner/property pair.
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionState<T> {
    Running(RunningTransition<T>),
    Completed(CompletedTransition<T>),
}

impl<T> TransitionState<T> {
    pub fn key(&self) -> &TransitionKey<T> {
        match self {
            Self::Running(run) => run.key(),
            Self::Completed(done) => done.key(),
        }
    }
}

/// Holds an immutable, duplicate-free set of transition records.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionSnapshot<T> {
    records: Vec<TransitionState<T>>,
}

impl<T> Default for TransitionSnapshot<T> {
    fn default() -> Self {
        Self {
            records: Vec::new(),
        }
    }
}

/// Distinguishes an unresolved sample, delay, active duration, and completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionSamplePhase {
    Unresolved,
    Delay,
    Active,
    Finished,
}

/// Carries a same-record, same-time value and its separate timing output.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionCurrentSample<T> {
    key: TransitionKey<T>,
    id: TransitionTrackId,
    time: Option<OriginRelativeTime>,
    phase: TransitionSamplePhase,
    value: Option<InterpolableValue>,
    easing_output: Option<EasedProgress>,
}

impl<T> TransitionCurrentSample<T> {
    pub const fn key(&self) -> &TransitionKey<T> {
        &self.key
    }
    pub const fn id(&self) -> TransitionTrackId {
        self.id
    }
    pub const fn time(&self) -> Option<OriginRelativeTime> {
        self.time
    }
    pub const fn phase(&self) -> TransitionSamplePhase {
        self.phase
    }
    pub const fn value(&self) -> Option<&InterpolableValue> {
        self.value.as_ref()
    }
    pub const fn easing_output(&self) -> Option<EasedProgress> {
        self.easing_output
    }
}

/// Describes a style or removal outcome for Runtime to publish and dispatch.
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionOutcome<T> {
    Started(RunningTransition<T>),
    Cancelled(RunningTransition<T>),
    CompletedRemoved(CompletedTransition<T>),
}

/// Separates complete promotion state from subsequent style-change outcomes.
///
/// Runtime publishes all `after_completion` promotions before processing a
/// same-time style event or dispatching completion callbacks. `state` is the
/// final candidate after style decisions. Neither snapshot is published here.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionUpdate<T> {
    after_completion: TransitionSnapshot<T>,
    state: TransitionSnapshot<T>,
    completions: Vec<CompletedTransition<T>>,
    outcomes: Vec<TransitionOutcome<T>>,
}

impl<T> TransitionUpdate<T> {
    pub const fn after_completion(&self) -> &TransitionSnapshot<T> {
        &self.after_completion
    }
    pub const fn state(&self) -> &TransitionSnapshot<T> {
        &self.state
    }
    pub fn completions(&self) -> &[CompletedTransition<T>] {
        &self.completions
    }
    pub fn outcomes(&self) -> &[TransitionOutcome<T>] {
        &self.outcomes
    }
}

/// Identifies a rejected snapshot, calculation, or property interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionDecisionErrorKind {
    DuplicateKey,
    DuplicateChange,
    UnrepresentableTime,
    InvalidProgress,
    Interpolation,
}

/// Reports a failed decision without exposing partial state or outcomes.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionDecisionError {
    failure: DecisionFailure,
}

#[derive(Debug, Clone, PartialEq)]
enum DecisionFailure {
    DuplicateKey(usize, usize),
    DuplicateChange(usize, usize),
    Time {
        operation: &'static str,
        cause: Option<TimeError>,
    },
    Numeric(NumericError),
    Interpolation(Box<InterpolationError>),
}

impl TransitionDecisionError {
    /// Returns the property failure, distinguishing unsupported interpolation
    /// from recoverable sample-local arithmetic failure.
    pub fn interpolation_error(&self) -> Option<&InterpolationError> {
        match &self.failure {
            DecisionFailure::Interpolation(error) => Some(error.as_ref()),
            _ => None,
        }
    }
    pub const fn kind(&self) -> TransitionDecisionErrorKind {
        match self.failure {
            DecisionFailure::DuplicateKey(..) => TransitionDecisionErrorKind::DuplicateKey,
            DecisionFailure::DuplicateChange(..) => TransitionDecisionErrorKind::DuplicateChange,
            DecisionFailure::Time { .. } => TransitionDecisionErrorKind::UnrepresentableTime,
            DecisionFailure::Numeric(_) => TransitionDecisionErrorKind::InvalidProgress,
            DecisionFailure::Interpolation(_) => TransitionDecisionErrorKind::Interpolation,
        }
    }
    pub const fn duplicate_indices(&self) -> Option<(usize, usize)> {
        match self.failure {
            DecisionFailure::DuplicateKey(first, second)
            | DecisionFailure::DuplicateChange(first, second) => Some((first, second)),
            _ => None,
        }
    }
}

impl fmt::Display for TransitionDecisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.failure {
            DecisionFailure::DuplicateKey(a, b) => write!(
                f,
                "transition records {a} and {b} have the same target and longhand"
            ),
            DecisionFailure::DuplicateChange(a, b) => write!(
                f,
                "style changes {a} and {b} have the same target and longhand"
            ),
            DecisionFailure::Time { operation, .. } => {
                write!(f, "transition {operation} cannot be represented")
            }
            DecisionFailure::Numeric(error) => write!(f, "invalid transition progress: {error}"),
            DecisionFailure::Interpolation(error) => {
                write!(f, "transition interpolation failed: {error}")
            }
        }
    }
}

impl std::error::Error for TransitionDecisionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.failure {
            DecisionFailure::Time {
                cause: Some(error), ..
            } => Some(error),
            DecisionFailure::Numeric(error) => Some(error),
            DecisionFailure::Interpolation(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<NumericError> for TransitionDecisionError {
    fn from(error: NumericError) -> Self {
        Self {
            failure: DecisionFailure::Numeric(error),
        }
    }
}
impl From<InterpolationError> for TransitionDecisionError {
    fn from(error: InterpolationError) -> Self {
        Self {
            failure: DecisionFailure::Interpolation(Box::new(error)),
        }
    }
}

impl<T> RunningTransition<T> {
    pub fn key(&self) -> &TransitionKey<T> {
        &self.data.key
    }
    pub fn id(&self) -> TransitionTrackId {
        self.data.id
    }
    pub fn created_at(&self) -> OriginRelativeTime {
        self.data.created_at
    }
    pub fn start_time(&self) -> OriginRelativeTime {
        self.data.start
    }
    pub fn end_time(&self) -> OriginRelativeTime {
        self.data.end
    }
    pub fn start_value(&self) -> &InterpolableValue {
        &self.data.from
    }
    pub fn end_value(&self) -> &InterpolableValue {
        &self.data.to
    }
    pub fn reversing_adjusted_start_value(&self) -> &InterpolableValue {
        &self.data.reversing_start
    }
    pub fn shortening_factor(&self) -> UnitRatio {
        self.data.shortening
    }
    pub fn timing(&self) -> &TransitionTiming {
        &self.data.timing
    }
    pub fn duration(&self) -> AnimationDuration {
        self.data.duration
    }
    pub fn delay(&self) -> AnimationDelay {
        self.data.delay
    }
}

impl<T: Clone> RunningTransition<T> {
    /// Samples this record from supplied absolute time without retaining history.
    ///
    /// Unresolved time and times at/after the end contribute no value. Delay
    /// (including exact start) returns the raw start without interpolation.
    /// Old easing output is retained independently for reversal calculations.
    /// Supply time from the same document origin as the record. Runtime applies
    /// the result only to the applicable record and time in its current frame.
    ///
    /// # Errors
    /// Returns typed numeric or property interpolation failures. No fallback
    /// value, clamping of property values, or lifecycle mutation is performed.
    pub fn sample<P: TransitionPropertyPolicy + ?Sized>(
        &self,
        time: Option<OriginRelativeTime>,
        policy: &P,
    ) -> Result<TransitionCurrentSample<T>, TransitionDecisionError> {
        let mut sample = TransitionCurrentSample {
            key: self.key().clone(),
            id: self.id(),
            time,
            phase: TransitionSamplePhase::Unresolved,
            value: None,
            easing_output: None,
        };
        let Some(now) = time else { return Ok(sample) };
        if now.as_millis() >= self.end_time().as_millis() {
            sample.phase = TransitionSamplePhase::Finished;
            return Ok(sample);
        }
        if now.as_millis() <= self.start_time().as_millis() {
            sample.phase = TransitionSamplePhase::Delay;
            sample.value = Some(self.start_value().clone());
            let easing = self.timing().easing();
            // The selected transition rules use the before flag for a step
            // boundary. Linear easing keeps the last point at a duplicate input.
            let before_step = matches!(easing, Easing::Steps(_))
                && now.as_millis() < self.start_time().as_millis();
            sample.easing_output =
                Some(easing.evaluate_with_before_flag(NormalizedProgress::ZERO, before_step));
            return Ok(sample);
        }
        let progress = interval_progress(now, self.start_time(), self.end_time())?;
        let eased = self.timing().easing().evaluate(progress);
        let progress = InterpolationProgress::from_eased(eased)?;
        sample.phase = TransitionSamplePhase::Active;
        sample.value = Some(
            match policy.interpolate(
                self.key().property(),
                self.start_value(),
                self.end_value(),
                progress,
            ) {
                InterpolationOutcome::Value(value) => value,
                InterpolationOutcome::ArithmeticFailure(error) => {
                    return Err(InterpolationError::NonFiniteInterpolationResult(error).into());
                }
                InterpolationOutcome::ComputedFailure(error) => {
                    return Err(InterpolationError::Computed(error).into());
                }
                InterpolationOutcome::Unsupported(error) => return Err(error.into()),
            },
        );
        sample.easing_output = Some(eased);
        Ok(sample)
    }

    fn completed(&self) -> CompletedTransition<T> {
        CompletedTransition {
            key: self.key().clone(),
            id: self.id(),
            end_value: self.end_value().clone(),
            end_time: self.end_time(),
            duration: self.duration(),
            delay: self.delay(),
        }
    }
}

impl<T> CompletedTransition<T> {
    pub const fn key(&self) -> &TransitionKey<T> {
        &self.key
    }
    pub const fn id(&self) -> TransitionTrackId {
        self.id
    }
    pub const fn end_value(&self) -> &InterpolableValue {
        &self.end_value
    }
    pub const fn end_time(&self) -> OriginRelativeTime {
        self.end_time
    }
    pub const fn duration(&self) -> AnimationDuration {
        self.duration
    }
    pub const fn delay(&self) -> AnimationDelay {
        self.delay
    }
}

impl<T: Eq> TransitionSnapshot<T> {
    /// Validates that each caller target and longhand appears at most once.
    pub fn new(records: Vec<TransitionState<T>>) -> Result<Self, TransitionDecisionError> {
        for (index, record) in records.iter().enumerate() {
            if let Some(first) = records
                .iter()
                .take(index)
                .position(|previous| previous.key() == record.key())
            {
                return Err(TransitionDecisionError {
                    failure: DecisionFailure::DuplicateKey(first, index),
                });
            }
        }
        Ok(Self { records })
    }
    pub fn records(&self) -> &[TransitionState<T>] {
        &self.records
    }
    pub fn get(&self, key: &TransitionKey<T>) -> Option<&TransitionState<T>> {
        self.records.iter().find(|record| record.key() == key)
    }
}

impl<T: Clone + Eq> TransitionSnapshot<T> {
    /// Promotes every due record, then decides one atomic batch of style changes.
    ///
    /// An empty change list advances completion only. Unresolved time returns
    /// `Ok(None)` without promotion or style decisions. Caller snapshots and
    /// policy describe one coherent document style event. Current values are
    /// sampled internally from the same old record and event time; callers
    /// cannot substitute a sample from another record or time.
    ///
    /// # Errors
    /// Duplicate change keys, unrepresentable arithmetic, and unavailable
    /// interpolation reject the entire update, including tentative completions.
    ///
    /// ```
    /// use surgeist_animation::easing::Easing;
    /// use surgeist_animation::interpolation::{InterpolableNumber, InterpolableValue,
    ///     InterpolationOutcome, InterpolationPair, InterpolationProgress, PropertyKey};
    /// use surgeist_animation::time::{AnimationDelay, AnimationDuration, OriginRelativeTime};
    /// use surgeist_animation::transition::{TransitionChange, TransitionKey,
    ///     TransitionPropertyPolicy, TransitionSnapshot, TransitionState, TransitionTiming,
    ///     TransitionTrackId, Transitionability};
    ///
    /// // This example's caller has selected a numeric opacity animation type.
    /// struct Opacity;
    /// impl TransitionPropertyPolicy for Opacity {
    ///     fn transitionability(&self, _: &PropertyKey, _: &InterpolableValue,
    ///         _: &InterpolableValue) -> Transitionability { Transitionability::Continuous }
    ///     fn interpolate(&self, property: &PropertyKey, from: &InterpolableValue,
    ///         to: &InterpolableValue, p: InterpolationProgress) -> InterpolationOutcome {
    ///         match InterpolationPair::new(property.clone(), from.clone(), to.clone()) {
    ///             Ok(pair) => pair.sample(p),
    ///             Err(error) => InterpolationOutcome::Unsupported(error),
    ///         }
    ///     }
    /// }
    /// let key = TransitionKey::new(7_u64, PropertyKey::new("opacity").unwrap());
    /// let change = TransitionChange {
    ///     key: key.clone(),
    ///     before: InterpolableValue::number(InterpolableNumber::new(0.0).unwrap()),
    ///     after: InterpolableValue::number(InterpolableNumber::new(1.0).unwrap()),
    ///     eligible: true,
    ///     matching: Some(TransitionTiming::new(
    ///         AnimationDuration::from_secs(1.0).unwrap(),
    ///         AnimationDelay::from_secs(0.0).unwrap(), Easing::linear())),
    ///     new_id: TransitionTrackId::new(3),
    /// };
    /// let update = TransitionSnapshot::default().update(
    ///     Some(OriginRelativeTime::from_millis(0.0).unwrap()), &[change], &Opacity,
    /// ).unwrap().unwrap();
    /// let Some(TransitionState::Running(run)) = update.state().get(&key) else {
    ///     panic!("expected a newly created transition");
    /// };
    /// let sample = run.sample(Some(OriginRelativeTime::from_millis(500.0).unwrap()),
    ///     &Opacity).unwrap();
    /// assert_eq!(sample.value(), Some(&InterpolableValue::number(
    ///     InterpolableNumber::new(0.5).unwrap())));
    /// ```
    pub fn update<P: TransitionPropertyPolicy + ?Sized>(
        &self,
        now: Option<OriginRelativeTime>,
        changes: &[TransitionChange<T>],
        policy: &P,
    ) -> Result<Option<TransitionUpdate<T>>, TransitionDecisionError> {
        for (index, change) in changes.iter().enumerate() {
            if let Some(first) = changes
                .iter()
                .take(index)
                .position(|previous| previous.key == change.key)
            {
                return Err(TransitionDecisionError {
                    failure: DecisionFailure::DuplicateChange(first, index),
                });
            }
        }
        let Some(now) = now else { return Ok(None) };
        let mut state = self.clone();
        let mut completions = Vec::new();
        for record in &mut state.records {
            if let TransitionState::Running(run) = record
                && now.as_millis() >= run.end_time().as_millis()
            {
                let completed = run.completed();
                completions.push(completed.clone());
                *record = TransitionState::Completed(completed);
            }
        }
        let after_completion = state.clone();
        let mut outcomes = Vec::new();
        for change in changes {
            if change.eligible {
                state.decide_change(now, change, policy, &mut outcomes)?;
            }
        }
        Ok(Some(TransitionUpdate {
            after_completion,
            state,
            completions,
            outcomes,
        }))
    }

    /// Cancels and removes matching document targets without consulting a clock.
    ///
    /// This separate operation lets Runtime preserve the actual ordering of
    /// document removal and time updates. It neither promotes completion nor
    /// schedules events. The original snapshot remains available unchanged.
    pub fn remove_targets(&self, targets: &[T]) -> TransitionUpdate<T> {
        let mut state = self.clone();
        let mut outcomes = Vec::new();
        state.records.retain(|record| {
            if !targets.contains(record.key().target()) {
                return true;
            }
            outcomes.push(match record {
                TransitionState::Running(run) => TransitionOutcome::Cancelled(run.clone()),
                TransitionState::Completed(done) => {
                    TransitionOutcome::CompletedRemoved(done.clone())
                }
            });
            false
        });
        TransitionUpdate {
            after_completion: self.clone(),
            state,
            completions: Vec::new(),
            outcomes,
        }
    }

    fn remove(&mut self, key: &TransitionKey<T>) {
        self.records.retain(|record| record.key() != key);
    }

    fn decide_change<P: TransitionPropertyPolicy + ?Sized>(
        &mut self,
        now: OriginRelativeTime,
        change: &TransitionChange<T>,
        policy: &P,
        outcomes: &mut Vec<TransitionOutcome<T>>,
    ) -> Result<(), TransitionDecisionError> {
        match self.get(&change.key).cloned() {
            Some(TransitionState::Running(run)) => {
                let Some(timing) = change.matching.as_ref() else {
                    self.remove(&change.key);
                    outcomes.push(TransitionOutcome::Cancelled(run));
                    return Ok(());
                };
                // Timing edits alone do not alter an unchanged running target.
                if run.end_value() == &change.after {
                    return Ok(());
                }
                if !timing.has_positive_combined_duration() {
                    self.remove(&change.key);
                    outcomes.push(TransitionOutcome::Cancelled(run));
                    return Ok(());
                }
                let sample = run.sample(Some(now), policy)?;
                let Some(current) = sample.value() else {
                    return Err(time_failure("current-value progress", None));
                };
                if current == &change.after
                    || policy.transitionability(change.key.property(), current, &change.after)
                        != Transitionability::Continuous
                {
                    self.remove(&change.key);
                    outcomes.push(TransitionOutcome::Cancelled(run));
                    return Ok(());
                }
                let (reversing_start, shortening) =
                    if run.reversing_adjusted_start_value() == &change.after {
                        let Some(eased) = sample.easing_output() else {
                            return Err(time_failure("reversal progress", None));
                        };
                        let old = run.shortening_factor().value();
                        let shortening = (eased.value() * old + (1.0 - old)).abs().clamp(0.0, 1.0);
                        (run.end_value().clone(), UnitRatio::new(shortening)?)
                    } else {
                        (current.clone(), UnitRatio::new(1.0)?)
                    };
                let replacement = create_run(
                    change,
                    timing,
                    now,
                    current.clone(),
                    reversing_start,
                    shortening,
                )?;
                self.remove(&change.key);
                self.records
                    .push(TransitionState::Running(replacement.clone()));
                outcomes.push(TransitionOutcome::Cancelled(run));
                outcomes.push(TransitionOutcome::Started(replacement));
            }
            Some(TransitionState::Completed(done)) => {
                let same_target = done.end_value() == &change.after;
                if !same_target || change.matching.is_none() {
                    self.remove(&change.key);
                    outcomes.push(TransitionOutcome::CompletedRemoved(done));
                }
                if !same_target {
                    self.try_start(now, change, policy, outcomes)?;
                }
            }
            None => self.try_start(now, change, policy, outcomes)?,
        }
        Ok(())
    }

    fn try_start<P: TransitionPropertyPolicy + ?Sized>(
        &mut self,
        now: OriginRelativeTime,
        change: &TransitionChange<T>,
        policy: &P,
        outcomes: &mut Vec<TransitionOutcome<T>>,
    ) -> Result<(), TransitionDecisionError> {
        let Some(timing) = change.matching.as_ref() else {
            return Ok(());
        };
        if change.before == change.after
            || !timing.has_positive_combined_duration()
            || policy.transitionability(change.key.property(), &change.before, &change.after)
                != Transitionability::Continuous
        {
            return Ok(());
        }
        let run = create_run(
            change,
            timing,
            now,
            change.before.clone(),
            change.before.clone(),
            UnitRatio::new(1.0)?,
        )?;
        self.records.push(TransitionState::Running(run.clone()));
        outcomes.push(TransitionOutcome::Started(run));
        Ok(())
    }
}

fn create_run<T: Clone>(
    change: &TransitionChange<T>,
    timing: &TransitionTiming,
    now: OriginRelativeTime,
    from: InterpolableValue,
    reversing_start: InterpolableValue,
    shortening: UnitRatio,
) -> Result<RunningTransition<T>, TransitionDecisionError> {
    let factor = shortening.value();
    let duration = AnimationDuration::from_secs(timing.duration.as_secs() * factor)?;
    let delay = AnimationDelay::from_secs(if timing.delay.as_secs() < 0.0 {
        timing.delay.as_secs() * factor
    } else {
        timing.delay.as_secs()
    })?;
    if factor > 0.0
        && ((timing.duration.as_secs() > 0.0 && duration.as_secs() == 0.0)
            || (timing.delay.as_secs() < 0.0 && delay.as_secs() == 0.0))
    {
        return Err(time_failure("shortened timing", None));
    }
    let start = add_seconds(now, delay.as_secs(), "start time")?;
    let end = add_seconds(start, duration.as_secs(), "end time")?;
    if duration.as_secs() > 0.0 && end.as_millis() <= start.as_millis() {
        return Err(time_failure("positive duration", None));
    }
    Ok(RunningTransition {
        data: Arc::new(RunningData {
            key: change.key.clone(),
            id: change.new_id,
            created_at: now,
            start,
            end,
            from,
            to: change.after.clone(),
            reversing_start,
            shortening,
            timing: timing.clone(),
            duration,
            delay,
        }),
    })
}

fn time_failure(operation: &'static str, cause: Option<TimeError>) -> TransitionDecisionError {
    TransitionDecisionError {
        failure: DecisionFailure::Time { operation, cause },
    }
}

fn add_seconds(
    base: OriginRelativeTime,
    seconds: f64,
    operation: &'static str,
) -> Result<OriginRelativeTime, TransitionDecisionError> {
    let millis = seconds * 1000.0;
    // Preserve ordinary millisecond precision; scale only when the delta alone
    // overflows and cancellation with the signed coordinate can still fit.
    let value = if millis.is_finite() {
        base.as_millis() + millis
    } else {
        (base.as_millis() / 1000.0 + seconds) * 1000.0
    };
    if seconds != 0.0 && value == base.as_millis() {
        return Err(time_failure(operation, None));
    }
    OriginRelativeTime::from_millis(value).map_err(|cause| time_failure(operation, Some(cause)))
}

fn interval_progress(
    now: OriginRelativeTime,
    start: OriginRelativeTime,
    end: OriginRelativeTime,
) -> Result<NormalizedProgress, TransitionDecisionError> {
    let numerator = now.as_millis() - start.as_millis();
    let denominator = end.as_millis() - start.as_millis();
    let progress = if numerator.is_finite() && denominator.is_finite() {
        numerator / denominator
    } else {
        (now.as_millis() / 2.0 - start.as_millis() / 2.0)
            / (end.as_millis() / 2.0 - start.as_millis() / 2.0)
    };
    Ok(NormalizedProgress::new(progress)?)
}
