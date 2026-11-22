//! Source-ordered playback transitions and immutable requests to the native scheduler.

use super::*;
use crate::time::{OriginRelativeTime, TimelineTime};
use crate::timeline::{
    AnimationTimeInput, animation_time_to_origin_relative_time, animation_time_to_timeline_time,
    timeline_time_to_origin_relative_time,
};

/// One supplied native-update context; every timeline identity occurs at most once.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlaybackContext {
    timelines: Vec<TimelineSnapshot>,
}
impl PlaybackContext {
    pub fn new(timelines: Vec<TimelineSnapshot>) -> Result<Self, PlaybackError> {
        let mut identities = std::collections::HashSet::new();
        for timeline in &timelines {
            if !identities.insert(timeline.id()) {
                return Err(PlaybackError::new(PlaybackErrorKind::DuplicateTimeline {
                    timeline: timeline.id(),
                }));
            }
        }
        Ok(Self { timelines })
    }
    pub(super) fn timeline(
        &self,
        id: Option<TimelineId>,
    ) -> Result<Option<TimelineSnapshot>, PlaybackError> {
        id.map(|expected| {
            self.timelines
                .iter()
                .find(|snapshot| snapshot.id() == expected)
                .copied()
                .ok_or_else(|| PlaybackError::new(PlaybackErrorKind::MissingTimeline { expected }))
        })
        .transpose()
    }
}

/// A source playback operation used to categorize an InvalidStateError.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackOperation {
    Play,
    Pause,
    Reverse,
    Finish,
}

/// A complete deterministic operation; Runtime supplies context and executes returned requests.
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackCommand {
    Seek(Option<AnimationTime>),
    SetStartTime(Option<TimelineTime>),
    Play { auto_rewind: bool },
    Pause,
    SetPlaybackRate(PlaybackRate),
    UpdatePlaybackRate(PlaybackRate),
    Reverse,
    Finish,
    Cancel,
    Tick,
    AssignTimeline(Option<TimelineId>),
    AssignEffect(Option<EffectId>),
}

/// Native counterpart of ready and finished completion categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionKind {
    Ready,
    Finished,
}
/// A completion identity is distinct from task generations and animation state revisions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompletionId {
    animation: AnimationId,
    kind: CompletionKind,
    serial: u64,
}
impl CompletionId {
    pub const fn animation(&self) -> &AnimationId {
        &self.animation
    }
    pub const fn kind(&self) -> CompletionKind {
        self.kind
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionState {
    Pending,
    Resolved,
    Aborted,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionSnapshot {
    id: CompletionId,
    state: CompletionState,
}
impl CompletionSnapshot {
    pub const fn id(&self) -> &CompletionId {
        &self.id
    }
    pub const fn state(&self) -> CompletionState {
        self.state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingTaskKind {
    Play,
    Pause,
}
/// Opaque readiness work, invalidated on replacement or association/setup changes.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadinessToken {
    animation: AnimationId,
    generation: u64,
    kind: PendingTaskKind,
    timeline: Option<TimelineId>,
    effect: Option<EffectSnapshot>,
}
impl ReadinessToken {
    pub const fn animation(&self) -> &AnimationId {
        &self.animation
    }
    pub const fn kind(&self) -> PendingTaskKind {
        self.kind
    }
    pub const fn timeline(&self) -> Option<TimelineId> {
        self.timeline
    }
    pub const fn effect(&self) -> Option<&EffectSnapshot> {
        self.effect.as_ref()
    }
}
/// A deferred per-animation live recheck, deliberately not bound to a completion identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishNotificationToken {
    animation: AnimationId,
    generation: u64,
}
impl FinishNotificationToken {
    pub const fn animation(&self) -> &AnimationId {
        &self.animation
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectReadiness {
    Pending,
    Ready,
}
/// Captured timeline time when effect setup/suspension became ready, independently of dispatch time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReadinessInput {
    timeline: TimelineSnapshot,
    effect: EffectReadiness,
}
impl ReadinessInput {
    pub const fn new(timeline: TimelineSnapshot, effect: EffectReadiness) -> Self {
        Self { timeline, effect }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackEventKind {
    Finish,
    Cancel,
}
/// Queue-time native playback payload and sorting key; DOM construction/dispatch stays external.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackEventIntent {
    animation: AnimationId,
    kind: PlaybackEventKind,
    current: Option<AnimationTime>,
    timeline_time: Option<TimelineTime>,
    snapshot: AnimationSnapshot,
    scheduled: Option<OriginRelativeTime>,
    order: CreationOrder,
}
impl PlaybackEventIntent {
    pub const fn animation(&self) -> &AnimationId {
        &self.animation
    }
    pub const fn kind(&self) -> PlaybackEventKind {
        self.kind
    }
    pub const fn current_time(&self) -> Option<AnimationTime> {
        self.current
    }
    pub const fn timeline_time(&self) -> Option<TimelineTime> {
        self.timeline_time
    }
    pub const fn animation_snapshot(&self) -> AnimationSnapshot {
        self.snapshot
    }
    pub const fn scheduled_time(&self) -> Option<OriginRelativeTime> {
        self.scheduled
    }
    pub const fn creation_order(&self) -> CreationOrder {
        self.order
    }
}

/// Execute requests in order after the entire model operation commits.
///
/// Scheduling requests are always deferred. In particular, an immediately-ready
/// play/pause request must never be completed inline in the command call.
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackRequest {
    ScheduleReadiness(ReadinessToken),
    CancelReadiness(ReadinessToken),
    ScheduleFinishNotification(FinishNotificationToken),
    CancelFinishNotification(FinishNotificationToken),
    CreateCompletion(CompletionSnapshot),
    SettleCompletion {
        completion: CompletionId,
        state: CompletionState,
    },
    QueueEvent(PlaybackEventIntent),
}
/// Creation publishes its initial completion identities and any displaced-owner requests atomically.
#[derive(Debug, Clone, PartialEq)]
pub struct CreatedAnimation {
    pub(super) animation: AnimationId,
    pub(super) requests: Vec<PlaybackRequest>,
}
impl CreatedAnimation {
    pub const fn animation(&self) -> &AnimationId {
        &self.animation
    }
    pub fn requests(&self) -> &[PlaybackRequest] {
        &self.requests
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct State {
    start: Option<TimelineTime>,
    hold: Option<AnimationTime>,
    rate: PlaybackRate,
    staged: Option<PlaybackRate>,
    previous: Option<AnimationTime>,
    pub(super) pending: Option<ReadinessToken>,
    pub(super) notification: Option<FinishNotificationToken>,
    pub(super) ready: CompletionSnapshot,
    pub(super) finished: CompletionSnapshot,
    next_identity: u64,
    ready_generation: u64,
    finish_generation: u64,
}
#[derive(Debug, Clone)]
pub(super) struct Work {
    pub id: AnimationId,
    pub order: CreationOrder,
    pub timeline: Option<TimelineSnapshot>,
    pub effect: Option<EffectSnapshot>,
}
impl Work {
    fn end(&self) -> f64 {
        self.effect.as_ref().map_or(0.0, |effect| effect.facts.end)
    }
    fn time(&self) -> Option<TimelineTime> {
        self.timeline.and_then(TimelineSnapshot::current_time)
    }
    fn finite_timeline(&self) -> bool {
        self.timeline.is_some_and(|timeline| {
            timeline.monotonicity() == crate::timeline::TimelineMonotonicity::NonMonotonic
        })
    }
}
impl State {
    pub(super) fn new(id: &AnimationId) -> Result<Self, PlaybackError> {
        Ok(Self {
            start: None,
            hold: None,
            rate: PlaybackRate::new(1.0)?,
            staged: None,
            previous: None,
            pending: None,
            notification: None,
            ready: CompletionSnapshot {
                id: CompletionId {
                    animation: id.clone(),
                    kind: CompletionKind::Ready,
                    serial: 0,
                },
                state: CompletionState::Resolved,
            },
            finished: CompletionSnapshot {
                id: CompletionId {
                    animation: id.clone(),
                    kind: CompletionKind::Finished,
                    serial: 1,
                },
                state: CompletionState::Pending,
            },
            next_identity: 2,
            ready_generation: 0,
            finish_generation: 0,
        })
    }
    pub(super) fn coordinates(&self, timeline: Option<TimelineSnapshot>) -> PlaybackCoordinates {
        PlaybackCoordinates::new(
            AnimationSnapshot::new(self.rate, self.start, timeline),
            self.hold,
        )
    }
    pub(super) fn staged_rate(&self) -> Option<PlaybackRate> {
        self.staged
    }
    fn effective_rate(&self) -> PlaybackRate {
        self.staged.unwrap_or(self.rate)
    }
    fn apply_rate(&mut self) {
        if let Some(rate) = self.staged.take() {
            self.rate = rate;
        }
    }
    pub(super) fn can_retire(&self) -> bool {
        self.start.is_none()
            && self.hold.is_none()
            && self.pending.is_none()
            && self.notification.is_none()
    }
    fn current(&self, work: &Work) -> Result<Option<AnimationTime>, PlaybackError> {
        self.coordinates(work.timeline).current_time()
    }
    pub(super) fn observation(&self, work: &Work) -> Result<PlaybackObservation, PlaybackError> {
        let current = self.current(work)?;
        let state = self.play_state(work)?;
        Ok(PlaybackObservation {
            current,
            animation: EffectAnimationState::new(
                self.rate,
                state,
                work.timeline.map(TimelineSnapshot::monotonicity),
                AnimationReplaceState::Active,
            ),
            has_effect: work.effect.is_some(),
        })
    }
    fn play_state(&self, work: &Work) -> Result<EffectPlayState, PlaybackError> {
        let current = self.current(work)?;
        Ok(
            if current.is_none() && self.start.is_none() && self.pending.is_none() {
                EffectPlayState::Idle
            } else if self
                .pending
                .as_ref()
                .is_some_and(|task| task.kind == PendingTaskKind::Pause)
                || (self.start.is_none()
                    && !self
                        .pending
                        .as_ref()
                        .is_some_and(|task| task.kind == PendingTaskKind::Play))
            {
                EffectPlayState::Paused
            } else if current.is_some_and(|time| {
                (self.effective_rate().value() > 0.0 && time.as_millis() >= work.end())
                    || (self.effective_rate().value() < 0.0 && time.as_millis() <= 0.0)
            }) {
                EffectPlayState::Finished
            } else {
                EffectPlayState::Running
            },
        )
    }
    fn replace_completion(
        &mut self,
        kind: CompletionKind,
        state: CompletionState,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        let next = increment(self.next_identity)?;
        let animation = self.ready.id.animation.clone();
        let completion = CompletionSnapshot {
            id: CompletionId {
                animation,
                kind,
                serial: self.next_identity,
            },
            state,
        };
        self.next_identity = next;
        requests.push(PlaybackRequest::CreateCompletion(completion.clone()));
        match kind {
            CompletionKind::Ready => self.ready = completion,
            CompletionKind::Finished => self.finished = completion,
        }
        Ok(())
    }
    fn settle(
        completion: &mut CompletionSnapshot,
        state: CompletionState,
        requests: &mut Vec<PlaybackRequest>,
    ) {
        if completion.state == CompletionState::Pending {
            completion.state = state;
            requests.push(PlaybackRequest::SettleCompletion {
                completion: completion.id.clone(),
                state,
            });
        }
    }
    fn cancel_ready(&mut self, requests: &mut Vec<PlaybackRequest>) -> bool {
        if let Some(token) = self.pending.take() {
            requests.push(PlaybackRequest::CancelReadiness(token));
            true
        } else {
            false
        }
    }
    fn schedule_ready(
        &mut self,
        kind: PendingTaskKind,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        self.ready_generation = increment(self.ready_generation)?;
        let token = ReadinessToken {
            animation: work.id.clone(),
            generation: self.ready_generation,
            kind,
            timeline: work.timeline.map(TimelineSnapshot::id),
            effect: work.effect.clone(),
        };
        self.pending = Some(token.clone());
        requests.push(PlaybackRequest::ScheduleReadiness(token));
        Ok(())
    }
    pub(super) fn reschedule(
        &mut self,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        if let Some(token) = self.pending.take() {
            requests.push(PlaybackRequest::CancelReadiness(token.clone()));
            self.schedule_ready(token.kind, work, requests)?;
        }
        Ok(())
    }
    pub(super) fn timeline_changed(
        &mut self,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        if self.start.is_some() {
            self.hold = None;
        }
        self.reschedule(work, requests)?;
        self.update_finished(work, false, false, requests)
    }
    fn silent_seek(
        &mut self,
        seek: Option<AnimationTime>,
        work: &Work,
    ) -> Result<(), PlaybackError> {
        let Some(seek) = seek else {
            if self.current(work)?.is_some() {
                return Err(PlaybackError::new(PlaybackErrorKind::TypeError));
            }
            return Ok(());
        };
        if self.hold.is_some()
            || self.start.is_none()
            || work.time().is_none()
            || self.rate.value() == 0.0
        {
            self.hold = Some(seek);
        } else if let Some(now) = work.time() {
            self.start = Some(start_for_time(now, seek, self.rate)?);
        }
        if work.time().is_none() {
            self.start = None;
        }
        self.previous = None;
        Ok(())
    }
    fn seek(
        &mut self,
        seek: Option<AnimationTime>,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        self.silent_seek(seek, work)?;
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.kind == PendingTaskKind::Pause)
        {
            self.hold = seek;
            self.apply_rate();
            self.start = None;
            self.cancel_ready(requests);
            Self::settle(&mut self.ready, CompletionState::Resolved, requests);
        }
        self.update_finished(work, true, false, requests)
    }
    fn set_start(
        &mut self,
        start: Option<TimelineTime>,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        if work.time().is_none() && start.is_some() {
            self.hold = None;
        }
        let previous = self.current(work)?;
        self.apply_rate();
        self.start = start;
        if start.is_some() {
            if self.rate.value() != 0.0 {
                self.hold = None;
            }
        } else {
            self.hold = previous;
        }
        if self.cancel_ready(requests) {
            Self::settle(&mut self.ready, CompletionState::Resolved, requests);
        }
        self.update_finished(work, true, false, requests)
    }
    fn play(
        &mut self,
        rewind: bool,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        let aborted_pause = self
            .pending
            .as_ref()
            .is_some_and(|task| task.kind == PendingTaskKind::Pause);
        let current = self.current(work)?;
        let mut seek = None;
        if rewind {
            if self.effective_rate().value() >= 0.0
                && current
                    .is_none_or(|time| time.as_millis() < 0.0 || time.as_millis() >= work.end())
            {
                seek = Some(AnimationTime::from_millis(0.0)?);
            } else if self.effective_rate().value() < 0.0
                && current
                    .is_none_or(|time| time.as_millis() <= 0.0 || time.as_millis() > work.end())
            {
                if work.end() == f64::INFINITY {
                    return Err(invalid(PlaybackOperation::Play));
                }
                seek = Some(AnimationTime::from_millis(work.end())?);
            }
        }
        if seek.is_none() && self.start.is_none() && current.is_none() {
            seek = Some(AnimationTime::from_millis(0.0)?);
        }
        if let Some(seek) = seek {
            if work.finite_timeline() {
                self.start = Some(TimelineTime::from_millis(seek.as_millis())?);
                self.hold = None;
                self.apply_rate();
            } else {
                self.hold = Some(seek);
            }
        }
        if self.hold.is_some() {
            self.start = None;
        }
        let had_pending = self.cancel_ready(requests);
        if self.hold.is_none() && seek.is_none() && !aborted_pause && self.staged.is_none() {
            return Ok(());
        }
        if !had_pending {
            self.replace_completion(CompletionKind::Ready, CompletionState::Pending, requests)?;
        }
        self.schedule_ready(PendingTaskKind::Play, work, requests)?;
        self.update_finished(work, false, false, requests)
    }
    fn pause(
        &mut self,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        if self
            .pending
            .as_ref()
            .is_some_and(|task| task.kind == PendingTaskKind::Pause)
            || self.play_state(work)? == EffectPlayState::Paused
        {
            return Ok(());
        }
        if self.current(work)?.is_none() {
            let seek = if self.rate.value() >= 0.0 {
                0.0
            } else {
                if work.end() == f64::INFINITY {
                    return Err(invalid(PlaybackOperation::Pause));
                }
                work.end()
            };
            if work.finite_timeline() {
                self.start = Some(TimelineTime::from_millis(seek)?);
            } else {
                self.hold = Some(AnimationTime::from_millis(seek)?);
            }
        }
        if !self.cancel_ready(requests) {
            self.replace_completion(CompletionKind::Ready, CompletionState::Pending, requests)?;
        }
        self.schedule_ready(PendingTaskKind::Pause, work, requests)?;
        self.update_finished(work, false, false, requests)
    }
    fn set_rate(
        &mut self,
        rate: PlaybackRate,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        self.staged = None;
        let previous = self.current(work)?;
        let old_rate = self.rate;
        self.rate = rate;
        if !work.finite_timeline() && work.timeline.is_some() && previous.is_some() {
            self.seek(previous, work, requests)?;
        } else {
            if work.finite_timeline()
                && work.end() != f64::INFINITY
                && ((old_rate.value() < 0.0) != (rate.value() < 0.0))
                && let Some(start) = self.start
            {
                self.start = Some(
                    TimelineTime::from_millis(work.end() - start.as_millis()).map_err(|_| {
                        PlaybackError::new(PlaybackErrorKind::UnrepresentableStartTime)
                    })?,
                );
            }
            self.update_finished(work, false, false, requests)?;
        }
        Ok(())
    }
    fn update_rate(
        &mut self,
        rate: PlaybackRate,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        let previous = self.play_state(work)?;
        self.staged = Some(rate);
        if self.pending.is_some() {
            return Ok(());
        }
        if matches!(previous, EffectPlayState::Idle | EffectPlayState::Paused)
            || self.current(work)?.is_none()
        {
            self.apply_rate();
        } else if previous == EffectPlayState::Finished {
            let unconstrained = self.unconstrained(work)?;
            self.start = match (work.time(), unconstrained) {
                (Some(now), _) if rate.value() == 0.0 => Some(now),
                (Some(now), Some(current)) => Some(start_for_time(now, current, rate)?),
                _ => None,
            };
            self.apply_rate();
            self.update_finished(work, false, false, requests)?;
        } else {
            self.play(false, work, requests)?;
        }
        Ok(())
    }
    pub(super) fn command(
        &mut self,
        command: &PlaybackCommand,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        match command {
            PlaybackCommand::Seek(time) => self.seek(*time, work, requests),
            PlaybackCommand::SetStartTime(time) => self.set_start(*time, work, requests),
            PlaybackCommand::Play { auto_rewind } => self.play(*auto_rewind, work, requests),
            PlaybackCommand::Pause => self.pause(work, requests),
            PlaybackCommand::SetPlaybackRate(rate) => self.set_rate(*rate, work, requests),
            PlaybackCommand::UpdatePlaybackRate(rate) => self.update_rate(*rate, work, requests),
            PlaybackCommand::Reverse => {
                if work.time().is_none() {
                    return Err(invalid(PlaybackOperation::Reverse));
                }
                self.staged = Some(PlaybackRate::new(-self.effective_rate().value())?);
                self.play(true, work, requests)
            }
            PlaybackCommand::Finish => self.finish(work, requests),
            PlaybackCommand::Cancel => self.cancel(work, requests),
            PlaybackCommand::Tick => self.update_finished(work, false, false, requests),
            // Association procedures also update another record and are owned by the registry.
            PlaybackCommand::AssignTimeline(_) | PlaybackCommand::AssignEffect(_) => {
                Err(PlaybackError::new(PlaybackErrorKind::AssociationContext))
            }
        }
    }
    pub(super) fn complete_ready(
        &mut self,
        token: &ReadinessToken,
        input: ReadinessInput,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        if self.pending.as_ref() != Some(token) {
            return Err(PlaybackError::new(PlaybackErrorKind::StaleReadiness));
        }
        let expected = work.timeline.map(TimelineSnapshot::id);
        if let Some(expected) = expected {
            if expected != input.timeline.id() {
                return Err(PlaybackError::new(PlaybackErrorKind::TimelineMismatch {
                    expected,
                    actual: input.timeline.id(),
                }));
            }
        } else {
            return Err(PlaybackError::new(PlaybackErrorKind::NotReady));
        }
        let Some(ready_time) = input.timeline.current_time() else {
            return Err(PlaybackError::new(PlaybackErrorKind::NotReady));
        };
        if input.effect != EffectReadiness::Ready {
            return Err(PlaybackError::new(PlaybackErrorKind::NotReady));
        }
        if let Some(dispatch) = work.timeline
            && (dispatch.monotonicity() != input.timeline.monotonicity()
                || (dispatch.monotonicity() == crate::timeline::TimelineMonotonicity::Monotonic
                    && dispatch
                        .current_time()
                        .is_some_and(|time| time.as_millis() < ready_time.as_millis())))
        {
            return Err(PlaybackError::new(PlaybackErrorKind::InvalidReadiness));
        }
        self.pending = None;
        match token.kind {
            PendingTaskKind::Play => {
                if let Some(hold) = self.hold {
                    self.apply_rate();
                    self.start = Some(if self.rate.value() == 0.0 {
                        ready_time
                    } else {
                        start_for_time(ready_time, hold, self.rate)?
                    });
                    if self.rate.value() != 0.0 {
                        self.hold = None;
                    }
                } else if self.start.is_some() && self.staged.is_some() {
                    let current = timeline_time_to_animation_time(
                        Some(ready_time),
                        AnimationSnapshot::new(self.rate, self.start, Some(input.timeline)),
                    )?;
                    let Some(current) = current else {
                        return Err(invalid(PlaybackOperation::Play));
                    };
                    self.apply_rate();
                    if self.rate.value() == 0.0 {
                        self.hold = Some(current);
                    }
                    self.start = Some(if self.rate.value() == 0.0 {
                        ready_time
                    } else {
                        start_for_time(ready_time, current, self.rate)?
                    });
                } else if self.start.is_none() {
                    return Err(invalid(PlaybackOperation::Play));
                }
            }
            PendingTaskKind::Pause => {
                if self.start.is_some() && self.hold.is_none() {
                    self.hold = timeline_time_to_animation_time(
                        Some(ready_time),
                        AnimationSnapshot::new(self.rate, self.start, Some(input.timeline)),
                    )?;
                }
                self.apply_rate();
                self.start = None;
            }
        }
        Self::settle(&mut self.ready, CompletionState::Resolved, requests);
        self.update_finished(work, false, false, requests)
    }
    fn unconstrained(&self, work: &Work) -> Result<Option<AnimationTime>, PlaybackError> {
        Ok(timeline_time_to_animation_time(
            work.time(),
            AnimationSnapshot::new(self.rate, self.start, work.timeline),
        )?)
    }
    pub(super) fn update_finished(
        &mut self,
        work: &Work,
        did_seek: bool,
        synchronous: bool,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        let unconstrained = if did_seek {
            self.current(work)?
        } else {
            self.unconstrained(work)?
        };
        if let Some(current) = unconstrained
            && self.start.is_some()
            && self.pending.is_none()
        {
            if self.rate.value() > 0.0 && current.as_millis() >= work.end() {
                self.hold = Some(if did_seek {
                    current
                } else {
                    AnimationTime::from_millis(
                        self.previous
                            .map_or(work.end(), |previous| previous.as_millis().max(work.end())),
                    )?
                });
            } else if self.rate.value() < 0.0 && current.as_millis() <= 0.0 {
                self.hold = Some(if did_seek {
                    current
                } else {
                    AnimationTime::from_millis(
                        self.previous
                            .map_or(0.0, |previous| previous.as_millis().min(0.0)),
                    )?
                });
            } else if self.rate.value() != 0.0 && work.time().is_some() {
                if did_seek && let (Some(hold), Some(now)) = (self.hold, work.time()) {
                    self.start = Some(start_for_time(now, hold, self.rate)?);
                }
                self.hold = None;
            }
        }
        self.previous = self.current(work)?;
        let finished = self.play_state(work)? == EffectPlayState::Finished;
        if finished && self.finished.state == CompletionState::Pending {
            if synchronous {
                if let Some(token) = self.notification.take() {
                    requests.push(PlaybackRequest::CancelFinishNotification(token));
                }
                self.notify(work, requests)?;
            } else if self.notification.is_none() {
                self.finish_generation = increment(self.finish_generation)?;
                let token = FinishNotificationToken {
                    animation: work.id.clone(),
                    generation: self.finish_generation,
                };
                self.notification = Some(token.clone());
                requests.push(PlaybackRequest::ScheduleFinishNotification(token));
            }
        }
        if !finished && self.finished.state == CompletionState::Resolved {
            self.replace_completion(CompletionKind::Finished, CompletionState::Pending, requests)?;
        }
        Ok(())
    }
    pub(super) fn process_finish(
        &mut self,
        token: &FinishNotificationToken,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        if self.notification.as_ref() != Some(token) {
            return Err(PlaybackError::new(
                PlaybackErrorKind::StaleFinishNotification,
            ));
        }
        self.notification = None;
        self.notify(work, requests)
    }
    fn notify(
        &mut self,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        if self.play_state(work)? != EffectPlayState::Finished {
            return Ok(());
        }
        Self::settle(&mut self.finished, CompletionState::Resolved, requests);
        requests.push(PlaybackRequest::QueueEvent(
            self.event(PlaybackEventKind::Finish, work)?,
        ));
        Ok(())
    }
    fn finish(
        &mut self,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        if self.effective_rate().value() == 0.0
            || (self.effective_rate().value() > 0.0 && work.end() == f64::INFINITY)
        {
            return Err(invalid(PlaybackOperation::Finish));
        }
        self.apply_rate();
        let limit = AnimationTime::from_millis(if self.rate.value() > 0.0 {
            work.end()
        } else {
            0.0
        })?;
        self.silent_seek(Some(limit), work)?;
        if self.start.is_none()
            && let Some(now) = work.time()
        {
            self.start = Some(start_for_time(now, limit, self.rate)?);
        }
        if self.pending.is_some() && self.start.is_some() {
            if self
                .pending
                .as_ref()
                .is_some_and(|task| task.kind == PendingTaskKind::Pause)
            {
                self.hold = None;
            }
            self.cancel_ready(requests);
            Self::settle(&mut self.ready, CompletionState::Resolved, requests);
        }
        self.update_finished(work, true, true, requests)
    }
    fn cancel(
        &mut self,
        work: &Work,
        requests: &mut Vec<PlaybackRequest>,
    ) -> Result<(), PlaybackError> {
        // Idle is exactly unresolved start/current with no pending task. A hold
        // is resolved current time; a resolved start is nonidle even when its
        // timeline is inactive. Do not evaluate unnecessary overflowing math.
        if self.start.is_some() || self.hold.is_some() || self.pending.is_some() {
            if self.cancel_ready(requests) {
                self.apply_rate();
                Self::settle(&mut self.ready, CompletionState::Aborted, requests);
                self.replace_completion(
                    CompletionKind::Ready,
                    CompletionState::Resolved,
                    requests,
                )?;
            }
            Self::settle(&mut self.finished, CompletionState::Aborted, requests);
            self.replace_completion(CompletionKind::Finished, CompletionState::Pending, requests)?;
            requests.push(PlaybackRequest::QueueEvent(
                self.event(PlaybackEventKind::Cancel, work)?,
            ));
        }
        self.hold = None;
        self.start = None;
        self.update_finished(work, false, false, requests)
    }
    fn event(
        &self,
        kind: PlaybackEventKind,
        work: &Work,
    ) -> Result<PlaybackEventIntent, PlaybackError> {
        let snapshot = AnimationSnapshot::new(self.rate, self.start, work.timeline);
        let scheduled = match kind {
            PlaybackEventKind::Finish => animation_time_to_origin_relative_time(
                if work.end() == f64::INFINITY {
                    AnimationTimeInput::Infinite
                } else {
                    AnimationTimeInput::Finite(AnimationTime::from_millis(work.end())?)
                },
                snapshot,
            )?,
            PlaybackEventKind::Cancel => {
                if let Some(timeline) = work.timeline {
                    timeline_time_to_origin_relative_time(timeline.current_time(), timeline)?
                } else {
                    None
                }
            }
        };
        Ok(PlaybackEventIntent {
            animation: work.id.clone(),
            kind,
            current: if kind == PlaybackEventKind::Cancel {
                None
            } else {
                self.current(work)?
            },
            timeline_time: work.time(),
            snapshot,
            scheduled,
            order: work.order,
        })
    }
}

fn increment(value: u64) -> Result<u64, PlaybackError> {
    value
        .checked_add(1)
        .ok_or_else(|| PlaybackError::new(PlaybackErrorKind::IdentityExhausted))
}
fn invalid(operation: PlaybackOperation) -> PlaybackError {
    PlaybackError::new(PlaybackErrorKind::InvalidState { operation })
}
fn start_for_time(
    now: TimelineTime,
    time: AnimationTime,
    rate: PlaybackRate,
) -> Result<TimelineTime, PlaybackError> {
    let converted = animation_time_to_timeline_time(
        AnimationTimeInput::Finite(AnimationTime::from_millis(-time.as_millis())?),
        AnimationSnapshot::new(rate, Some(now), None),
    )?;
    converted.ok_or_else(|| PlaybackError::new(PlaybackErrorKind::TypeError))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_completion_and_readiness_generations_do_not_publish_partial_play() {
        for readiness in [false, true] {
            let mut model = AnimationRecords::new();
            let id = model
                .create(None, None, &PlaybackContext::default())
                .unwrap()
                .animation()
                .clone();
            let record = model
                .animations
                .get_mut(id.index)
                .unwrap()
                .as_mut()
                .unwrap();
            if readiness {
                record.state.ready_generation = u64::MAX;
            } else {
                record.state.next_identity = u64::MAX;
            }
            let before = model.snapshot(&id, None).unwrap();
            assert_eq!(
                model
                    .apply(
                        &id,
                        PlaybackCommand::Play { auto_rewind: true },
                        &PlaybackContext::default()
                    )
                    .unwrap_err()
                    .kind(),
                &PlaybackErrorKind::IdentityExhausted
            );
            assert_eq!(model.snapshot(&id, None).unwrap(), before);
        }
    }

    #[test]
    fn exhausted_finish_generation_rolls_back_start_and_hold_updates() {
        let timeline = TimelineSnapshot::new(
            TimelineId::new(1),
            Some(TimelineTime::from_millis(0.0).unwrap()),
            crate::timeline::TimelineMonotonicity::Monotonic,
            None,
        );
        let context = PlaybackContext::new(vec![timeline]).unwrap();
        let mut model = AnimationRecords::new();
        let id = model
            .create(Some(timeline.id()), None, &context)
            .unwrap()
            .animation()
            .clone();
        model
            .animations
            .get_mut(id.index)
            .unwrap()
            .as_mut()
            .unwrap()
            .state
            .finish_generation = u64::MAX;
        let before = model.snapshot(&id, Some(timeline)).unwrap();
        assert_eq!(
            model
                .apply(
                    &id,
                    PlaybackCommand::SetStartTime(Some(TimelineTime::from_millis(0.0).unwrap())),
                    &context
                )
                .unwrap_err()
                .kind(),
            &PlaybackErrorKind::IdentityExhausted
        );
        assert_eq!(model.snapshot(&id, Some(timeline)).unwrap(), before);
    }
}
