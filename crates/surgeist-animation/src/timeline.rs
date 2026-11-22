//! Supplied timeline snapshots and pure Web Animations time conversions.
//!
//! The caller captures one snapshot per native update and supplies document
//! activity and its origin-relative frame timestamp. This module never reads a
//! clock. Animation snapshots capture their own associated timeline and start/rate
//! state; later changes cannot alter conversions of a captured snapshot.
//!
//! Nullable coordinates are `Option<T>`: `None` is unresolved, distinct from
//! zero and from [`TimeError`]. [`AnimationTimeInput::Infinite`] is an explicit
//! algorithm input that converts to unresolved. Finite coordinate types prevent
//! accidental reuse of animation time as timeline or origin-relative time.
//!
//! ```
//! use surgeist_animation::time::{AnimationTime, OriginRelativeTime, PlaybackRate, TimelineTime};
//! use surgeist_animation::timeline::{
//!     AnimationSnapshot, AnimationTimeInput, DocumentClock, DocumentTimeline,
//!     TimelineId, animation_time_to_origin_relative_time,
//! };
//! # fn main() -> Result<(), surgeist_animation::time::TimeError> {
//! let mut timeline = DocumentTimeline::new(
//!     TimelineId::new(1), OriginRelativeTime::from_millis(40.0)?,
//! );
//! let frame = timeline.update(DocumentClock::Active(OriginRelativeTime::from_millis(150.0)?))?;
//! let animation = AnimationSnapshot::new(
//!     PlaybackRate::new(2.0)?, Some(TimelineTime::from_millis(100.0)?), Some(frame),
//! );
//! let input = AnimationTimeInput::Finite(AnimationTime::from_millis(30.0)?);
//! let scheduled = animation_time_to_origin_relative_time(input, animation)?;
//! assert_eq!(scheduled.map(OriginRelativeTime::as_millis), Some(155.0));
//! # Ok(())
//! # }
//! ```
//!
//! ```compile_fail,E0308
//! use surgeist_animation::time::AnimationTime;
//! use surgeist_animation::timeline::{TimelineSnapshot, timeline_time_to_origin_relative_time};
//! fn wrong_space(snapshot: TimelineSnapshot) {
//!     let time = AnimationTime::from_millis(30.0).unwrap();
//!     let _ = timeline_time_to_origin_relative_time(Some(time), snapshot);
//! }
//! ```

use crate::time::{
    AnimationTime, OriginRelativeTime, PlaybackRate, TimeError, TimeOperation, TimelineTime,
};

/// A caller-assigned timeline identity preserved by immutable snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimelineId(u64);

impl TimelineId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Whether a timeline's successive reported coordinates can decrease.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineMonotonicity {
    Monotonic,
    NonMonotonic,
}

/// A timeline's stable state during one native update.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineSnapshot {
    id: TimelineId,
    current_time: Option<TimelineTime>,
    monotonicity: TimelineMonotonicity,
    origin_offset: Option<OriginRelativeTime>,
}

impl TimelineSnapshot {
    /// Captures an external timeline's coordinate and conversion capability.
    ///
    /// `origin_offset` enables fixed-offset origin-relative conversion. `None`
    /// denotes no such conversion, even when current time is resolved. External
    /// producers own the truth of their monotonicity classification across
    /// reports; [`DocumentTimeline`] checks its own reports.
    pub const fn new(
        id: TimelineId,
        current_time: Option<TimelineTime>,
        monotonicity: TimelineMonotonicity,
        origin_offset: Option<OriginRelativeTime>,
    ) -> Self {
        Self {
            id,
            current_time,
            monotonicity,
            origin_offset,
        }
    }
    pub const fn id(self) -> TimelineId {
        self.id
    }
    pub const fn current_time(self) -> Option<TimelineTime> {
        self.current_time
    }
    pub const fn monotonicity(self) -> TimelineMonotonicity {
        self.monotonicity
    }
    pub const fn is_active(self) -> bool {
        self.current_time.is_some()
    }
}

/// The caller's document-clock observation for a single native update.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DocumentClock {
    /// The caller has not established this document's time origin.
    Unestablished,
    /// The document is not active, irrespective of an established time origin.
    Inactive,
    /// The active document's supplied origin-relative frame timestamp.
    Active(OriginRelativeTime),
}

/// Fixed-origin document timing with checked monotonic active reports.
///
/// Identity and origin time stay fixed. Inactivity clears current time but
/// retains the last active report for monotonicity validation. Restoration uses
/// the next supplied timestamp without deducting the inactive interval. The
/// caller owns document identity, origin establishment, and update scheduling.
#[derive(Debug)]
pub struct DocumentTimeline {
    id: TimelineId,
    origin_time: OriginRelativeTime,
    current_time: Option<TimelineTime>,
    last_active_time: Option<TimelineTime>,
}

impl DocumentTimeline {
    /// Creates an initially inactive timeline with an immutable origin-time offset.
    ///
    /// The default document timeline uses origin time zero. Origins are offsets
    /// within the document time space, not wall-clock epochs.
    pub const fn new(id: TimelineId, origin_time: OriginRelativeTime) -> Self {
        Self {
            id,
            origin_time,
            current_time: None,
            last_active_time: None,
        }
    }
    pub const fn origin_time(&self) -> OriginRelativeTime {
        self.origin_time
    }
    pub const fn snapshot(&self) -> TimelineSnapshot {
        TimelineSnapshot::new(
            self.id,
            self.current_time,
            TimelineMonotonicity::Monotonic,
            Some(self.origin_time),
        )
    }

    /// Applies one supplied observation and returns the stable update snapshot.
    ///
    /// # Errors
    ///
    /// Returns a typed arithmetic error if `now - origin_time` cannot be
    /// represented, or a decreasing-report error if a resolved report would
    /// decrease. Either error preserves the complete previous state.
    pub fn update(&mut self, clock: DocumentClock) -> Result<TimelineSnapshot, TimeError> {
        let current_time = match clock {
            DocumentClock::Unestablished | DocumentClock::Inactive => None,
            DocumentClock::Active(now) => {
                let time = timeline_result(
                    now.as_millis() - self.origin_time.as_millis(),
                    TimeOperation::DocumentCurrentTime,
                )?;
                if let Some(previous) = self.last_active_time
                    && time.as_millis() < previous.as_millis()
                {
                    return Err(TimeError::decreasing(previous, time));
                }
                Some(time)
            }
        };
        self.current_time = current_time;
        if current_time.is_some() {
            self.last_active_time = current_time;
        }
        Ok(self.snapshot())
    }
}

/// An animation-coordinate algorithm input, including unresolved and infinite time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationTimeInput {
    Unresolved,
    Finite(AnimationTime),
    /// The extended infinity sentinel, for example an infinite associated-effect end.
    Infinite,
}

/// Immutable start/rate state and its associated timeline at capture time.
///
/// This is conversion state only. Hold-time resolution, play control, finishing,
/// and effect sampling remain separate operations owned by their callers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationSnapshot {
    playback_rate: PlaybackRate,
    start_time: Option<TimelineTime>,
    timeline: Option<TimelineSnapshot>,
}

impl AnimationSnapshot {
    pub const fn new(
        playback_rate: PlaybackRate,
        start_time: Option<TimelineTime>,
        timeline: Option<TimelineSnapshot>,
    ) -> Self {
        Self {
            playback_rate,
            start_time,
            timeline,
        }
    }
    pub const fn playback_rate(self) -> PlaybackRate {
        self.playback_rate
    }
    pub const fn start_time(self) -> Option<TimelineTime> {
        self.start_time
    }
    pub const fn timeline(self) -> Option<TimelineSnapshot> {
        self.timeline
    }
}

/// Converts a timeline coordinate through that snapshot's own origin capability.
///
/// Unresolved input, an inactive snapshot, or absent conversion capability yields
/// unresolved. Animation start/rate state is never applied here.
///
/// # Errors
///
/// Returns an arithmetic error if the finite sum cannot be represented.
pub fn timeline_time_to_origin_relative_time(
    time: Option<TimelineTime>,
    timeline: TimelineSnapshot,
) -> Result<Option<OriginRelativeTime>, TimeError> {
    let (Some(time), Some(offset)) = (time, timeline.origin_offset) else {
        return Ok(None);
    };
    if !timeline.is_active() {
        return Ok(None);
    }
    OriginRelativeTime::from_millis(time.as_millis() + offset.as_millis())
        .map(Some)
        .map_err(|_| TimeError::unrepresentable(TimeOperation::TimelineToOrigin))
}

/// Converts animation time using the supplied animation's start time and rate.
///
/// Calculates `time / playback_rate + start_time`. Unresolved/infinite input,
/// zero rate, or unresolved start yields unresolved. Timeline association is not
/// required for this coordinate transformation.
///
/// # Errors
///
/// Returns an arithmetic error if the finite result cannot be represented.
pub fn animation_time_to_timeline_time(
    time: AnimationTimeInput,
    animation: AnimationSnapshot,
) -> Result<Option<TimelineTime>, TimeError> {
    let (AnimationTimeInput::Finite(time), Some(start)) = (time, animation.start_time) else {
        return Ok(None);
    };
    let rate = animation.playback_rate.value();
    if rate == 0.0 {
        return Ok(None);
    }
    let quotient = time.as_millis() / rate;
    let value = if quotient.is_finite() {
        quotient + start.as_millis()
    } else {
        // A finite start can cancel an overflowing quotient only within twice
        // the finite range. Halving both terms preserves that recoverable case.
        (time.as_millis() * 0.5 / rate + start.as_millis() * 0.5) * 2.0
    };
    timeline_result(value, TimeOperation::AnimationToTimeline).map(Some)
}

/// Converts animation time, then invokes its captured associated timeline converter.
///
/// This is the native repair of the selected WA1 event converter's undefined
/// animation parameter and incorrect unresolved return. Any unresolved
/// intermediate remains unresolved; the original resolved input is never a
/// fallback. The timeline origin offset is applied exactly once.
///
/// # Errors
///
/// Propagates typed arithmetic errors from either conversion.
pub fn animation_time_to_origin_relative_time(
    time: AnimationTimeInput,
    animation: AnimationSnapshot,
) -> Result<Option<OriginRelativeTime>, TimeError> {
    let Some(time) = animation_time_to_timeline_time(time, animation)? else {
        return Ok(None);
    };
    let Some(timeline) = animation.timeline else {
        return Ok(None);
    };
    timeline_time_to_origin_relative_time(Some(time), timeline)
}

/// Maps a supplied timeline coordinate through resolved animation start/rate state.
///
/// Calculates `(time - start_time) * playback_rate` without frame history or
/// hold-time rules. Unresolved input/start yields unresolved. The caller supplies
/// a coordinate from the same timeline as the animation snapshot.
///
/// # Errors
///
/// Returns an arithmetic error if the finite result cannot be represented.
pub fn timeline_time_to_animation_time(
    time: Option<TimelineTime>,
    animation: AnimationSnapshot,
) -> Result<Option<AnimationTime>, TimeError> {
    let (Some(time), Some(start)) = (time, animation.start_time) else {
        return Ok(None);
    };
    let rate = animation.playback_rate.value();
    let difference = time.as_millis() - start.as_millis();
    let value = if difference.is_finite() {
        difference * rate
    } else if rate.abs() <= 1.0 {
        // Multiplication by a small rate can recover a finite scaled difference.
        time.as_millis() * rate - start.as_millis() * rate
    } else {
        difference
    };
    AnimationTime::from_millis(value)
        .map(Some)
        .map_err(|_| TimeError::unrepresentable(TimeOperation::TimelineToAnimation))
}

fn timeline_result(value: f64, operation: TimeOperation) -> Result<TimelineTime, TimeError> {
    TimelineTime::from_millis(value).map_err(|_| TimeError::unrepresentable(operation))
}
