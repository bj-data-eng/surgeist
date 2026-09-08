//! Normalized keyframe animation tracks.
//!
//! Keyframe tracks contain root-supplied animation identities, validated timing,
//! and contiguous per-property segments. This crate samples those typed segments
//! but does not parse keyframes, resolve animation names, or synthesize
//! endpoints from style.

use core::fmt;
use core::ops::Range;
use std::collections::HashMap;
use std::sync::Arc;

use crate::timing::ActiveDuration;
use crate::{
    AnimationDelay, AnimationDirection, AnimationDuration, AnimationPlayState, Easing, ElapsedTime,
    FillMode, InterpolableValue, InterpolationError, InterpolationPair, InterpolationProgress,
    IterationCount, NormalizedProgress, PropertyKey, SampledPropertyResult, TimingError,
    TimingParameters, TimingSample, UnitRatio,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyframeAnimationId {
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyframeTiming {
    delay: AnimationDelay,
    duration: AnimationDuration,
    iterations: IterationCount,
    direction: AnimationDirection,
    fill_mode: FillMode,
    active_duration: ActiveDuration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyframeSegment {
    pair: InterpolationPair,
    from_offset: UnitRatio,
    to_offset: UnitRatio,
    easing: Easing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyframeTrack {
    id: KeyframeAnimationId,
    timing: KeyframeTiming,
    segments: Vec<KeyframeSegment>,
    lookup: SegmentLookup,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeError {
    EmptyAnimationId,
    EmptySegments {
        animation: KeyframeAnimationId,
    },
    DegenerateSegment {
        property: PropertyKey,
        offset: UnitRatio,
    },
    Interpolation(Box<InterpolationError>),
    MissingInitialOffset {
        property: PropertyKey,
    },
    MissingFinalOffset {
        property: PropertyKey,
    },
    OverlappingSegment {
        property: PropertyKey,
        previous_to: UnitRatio,
        actual: UnitRatio,
    },
    NonContiguousSegment {
        property: PropertyKey,
        expected: UnitRatio,
        actual: UnitRatio,
    },
    DuplicateOffset {
        property: PropertyKey,
        offset: UnitRatio,
    },
}

impl fmt::Display for KeyframeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAnimationId => f.write_str("keyframe animation id must not be empty"),
            Self::EmptySegments { animation } => write!(
                f,
                "keyframe animation {} must contain at least one segment",
                animation.as_str()
            ),
            Self::DegenerateSegment { property, offset } => write!(
                f,
                "keyframe property {} segment at offset {} must span distinct offsets",
                property.as_str(),
                offset.value()
            ),
            Self::Interpolation(_) => f.write_str("keyframe interpolation failed"),
            Self::MissingInitialOffset { property } => write!(
                f,
                "keyframe property {} is missing an initial offset",
                property.as_str()
            ),
            Self::MissingFinalOffset { property } => write!(
                f,
                "keyframe property {} is missing a final offset",
                property.as_str()
            ),
            Self::OverlappingSegment {
                property,
                previous_to,
                actual,
            } => write!(
                f,
                "keyframe property {} segment offset {} overlaps previous end {}",
                property.as_str(),
                actual.value(),
                previous_to.value()
            ),
            Self::NonContiguousSegment {
                property,
                expected,
                actual,
            } => write!(
                f,
                "keyframe property {} segment offset {} must start at previous end {}",
                property.as_str(),
                actual.value(),
                expected.value()
            ),
            Self::DuplicateOffset { property, offset } => write!(
                f,
                "keyframe property {} has duplicate offset {}",
                property.as_str(),
                offset.value()
            ),
        }
    }
}

impl std::error::Error for KeyframeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Interpolation(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum SegmentLookup {
    Single,
    Pair(Arc<[PropertySegmentGroup; 2]>),
    Grouped(Vec<PropertySegmentGroup>),
}

impl SegmentLookup {
    fn property_count(&self) -> usize {
        match self {
            Self::Single => 1,
            Self::Pair(_) => 2,
            Self::Grouped(groups) => groups.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PropertySegmentGroup {
    property: PropertyKey,
    membership: SegmentMembership,
}

#[derive(Debug, Clone, PartialEq)]
enum SegmentMembership {
    Contiguous(Range<usize>),
    Scattered(Vec<usize>),
}

impl SegmentMembership {
    #[inline]
    fn push(&mut self, index: usize) {
        match self {
            Self::Contiguous(range) if range.end == index => range.end += 1,
            Self::Contiguous(range) => {
                *self = Self::scattered_from_range(range.clone(), index);
            }
            Self::Scattered(indices) => indices.push(index),
        }
    }

    #[inline(never)]
    fn scattered_from_range(range: Range<usize>, index: usize) -> Self {
        // Promote at the first gap, retaining all earlier members;
        // subsequent occurrences append to the scattered list.
        let mut indices = Vec::with_capacity((range.end - range.start + 1).max(4));
        indices.extend(range);
        indices.push(index);
        Self::Scattered(indices)
    }

    fn view<'a>(&'a self, segments: &'a [KeyframeSegment]) -> PropertySegments<'a> {
        match self {
            Self::Contiguous(range) => PropertySegments::Direct(&segments[range.clone()]),
            Self::Scattered(indices) => PropertySegments::Indexed { segments, indices },
        }
    }
}

#[derive(Clone, Copy)]
enum PropertySegments<'a> {
    Direct(&'a [KeyframeSegment]),
    Indexed {
        segments: &'a [KeyframeSegment],
        indices: &'a [usize],
    },
}

impl<'a> PropertySegments<'a> {
    fn len(self) -> usize {
        match self {
            Self::Direct(segments) => segments.len(),
            Self::Indexed { indices, .. } => indices.len(),
        }
    }

    fn at(self, position: usize) -> &'a KeyframeSegment {
        match self {
            Self::Direct(segments) => &segments[position],
            Self::Indexed { segments, indices } => &segments[indices[position]],
        }
    }

    fn partition_point(
        self,
        end: usize,
        mut predicate: impl FnMut(&KeyframeSegment) -> bool,
    ) -> usize {
        match self {
            Self::Direct(segments) => segments[..end].partition_point(predicate),
            Self::Indexed { segments, indices } => {
                indices[..end].partition_point(|&index| predicate(&segments[index]))
            }
        }
    }
}

impl KeyframeAnimationId {
    pub fn new(name: impl Into<String>) -> Result<Self, KeyframeError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(KeyframeError::EmptyAnimationId);
        }

        Ok(Self { name })
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl KeyframeTiming {
    pub fn try_new(
        delay: AnimationDelay,
        duration: AnimationDuration,
        iterations: IterationCount,
        direction: AnimationDirection,
        fill_mode: FillMode,
    ) -> Result<Self, TimingError> {
        let active_duration = TimingParameters::validated_active_duration(duration, iterations)?;

        Ok(Self {
            delay,
            duration,
            iterations,
            direction,
            fill_mode,
            active_duration,
        })
    }

    pub const fn delay(self) -> AnimationDelay {
        self.delay
    }

    pub const fn duration(self) -> AnimationDuration {
        self.duration
    }

    pub const fn iterations(self) -> IterationCount {
        self.iterations
    }

    pub const fn direction(self) -> AnimationDirection {
        self.direction
    }

    pub const fn fill_mode(self) -> FillMode {
        self.fill_mode
    }

    pub fn to_timing_parameters(self) -> TimingParameters {
        TimingParameters::from_validated_parts(
            self.delay,
            self.duration,
            self.iterations,
            self.direction,
            self.fill_mode,
            Easing::linear(),
            self.active_duration,
        )
    }
}

impl KeyframeSegment {
    pub fn new(
        property: PropertyKey,
        from_offset: UnitRatio,
        to_offset: UnitRatio,
        from: InterpolableValue,
        to: InterpolableValue,
        easing: Easing,
    ) -> Result<Self, KeyframeError> {
        if from_offset.value() >= to_offset.value() {
            return Err(KeyframeError::DegenerateSegment {
                property,
                offset: from_offset,
            });
        }

        let pair = InterpolationPair::new(property, from, to)
            .map_err(|error| KeyframeError::Interpolation(Box::new(error)))?;

        Ok(Self {
            pair,
            from_offset,
            to_offset,
            easing,
        })
    }

    pub fn property(&self) -> &PropertyKey {
        self.pair.property()
    }

    pub const fn from_offset(&self) -> UnitRatio {
        self.from_offset
    }

    pub const fn to_offset(&self) -> UnitRatio {
        self.to_offset
    }

    pub fn from(&self) -> &InterpolableValue {
        self.pair.from()
    }

    pub fn to(&self) -> &InterpolableValue {
        self.pair.to()
    }

    pub const fn easing(&self) -> &Easing {
        &self.easing
    }
}

impl KeyframeTrack {
    pub fn new(
        id: KeyframeAnimationId,
        timing: KeyframeTiming,
        segments: Vec<KeyframeSegment>,
    ) -> Result<Self, KeyframeError> {
        if segments.is_empty() {
            return Err(KeyframeError::EmptySegments { animation: id });
        }

        let first_property = segments[0].property();
        let second_index = segments
            .iter()
            .position(|segment| segment.property() != first_property);
        let lookup = match second_index {
            None => {
                validate_property_segments(first_property, PropertySegments::Direct(&segments))?;
                SegmentLookup::Single
            }
            Some(second_index) => multiple_property_lookup(&segments, second_index)?,
        };

        Ok(Self {
            id,
            timing,
            segments,
            lookup,
        })
    }

    pub fn id(&self) -> &KeyframeAnimationId {
        &self.id
    }

    pub const fn timing(&self) -> KeyframeTiming {
        self.timing
    }

    pub fn segments(&self) -> &[KeyframeSegment] {
        &self.segments
    }

    pub fn sample(
        &self,
        elapsed: ElapsedTime,
        play_state: AnimationPlayState,
    ) -> Vec<SampledPropertyResult> {
        let mut output = Vec::with_capacity(self.lookup.property_count());
        self.sample_into(elapsed, play_state, &mut output);
        output
    }

    /// Replaces `output` with one owned sample per property, in first-appearance order.
    ///
    /// The existing allocation is retained. A buffer with capacity for every
    /// property can be reused without allocating or reallocating while sampling,
    /// including unsupported outcomes and arithmetic diagnostics.
    pub fn sample_into(
        &self,
        elapsed: ElapsedTime,
        play_state: AnimationPlayState,
        output: &mut Vec<SampledPropertyResult>,
    ) {
        output.clear();
        let property_count = self.lookup.property_count();
        output.reserve(property_count);
        let timing = self
            .timing
            .to_timing_parameters()
            .sample(elapsed, play_state);
        let Some(progress) = timing
            .directed_progress()
            .and_then(|progress| UnitRatio::new(progress.value()).ok())
        else {
            let sample = SampledPropertyResult::from_timing_and_interpolation_outcome(
                timing,
                self.segments[0].property(),
                None,
            );
            output.resize(property_count, sample);
            return;
        };

        let groups: &[PropertySegmentGroup] = match &self.lookup {
            SegmentLookup::Single => {
                sample_property(
                    self.segments[0].property(),
                    &self.segments,
                    KeyframeSegment::from_offset,
                    |index| &self.segments[index],
                    timing,
                    progress,
                    output,
                );
                return;
            }
            SegmentLookup::Pair(groups) => groups.as_ref(),
            SegmentLookup::Grouped(groups) => groups,
        };
        for group in groups {
            let property = &group.property;
            match &group.membership {
                SegmentMembership::Contiguous(range) => {
                    let segments = &self.segments[range.clone()];
                    sample_property(
                        property,
                        segments,
                        KeyframeSegment::from_offset,
                        |index| &segments[index],
                        timing,
                        progress,
                        output,
                    )
                }
                SegmentMembership::Scattered(indices) => sample_property(
                    property,
                    indices,
                    |&index| self.segments[index].from_offset(),
                    |position| &self.segments[indices[position]],
                    timing,
                    progress,
                    output,
                ),
            }
        }
    }
}

fn sample_property<'a, T>(
    property: &PropertyKey,
    entries: &[T],
    start_offset: impl Fn(&T) -> UnitRatio,
    segment_at: impl Fn(usize) -> &'a KeyframeSegment,
    timing: TimingSample,
    progress: UnitRatio,
    output: &mut Vec<SampledPropertyResult>,
) {
    // Construction proves a nonempty, contiguous group starting at zero.
    // The last start at or before progress selects the following segment at
    // an interior boundary, and the final segment at progress one.
    let after = entries.partition_point(|entry| start_offset(entry).value() <= progress.value());
    let segment = segment_at(after - 1);
    let outcome = if let Some(local_progress) = segment_local_progress(segment, progress) {
        let eased_progress = segment
            .easing()
            .evaluate_with_before_flag(local_progress, timing.easing_before_flag());
        InterpolationProgress::from_eased(eased_progress)
            .ok()
            .map(|interpolation_progress| segment.pair.sample(interpolation_progress))
    } else {
        None
    };
    output.push(
        SampledPropertyResult::from_timing_and_interpolation_outcome(timing, property, outcome),
    );
}

fn segment_local_progress(
    segment: &KeyframeSegment,
    progress: UnitRatio,
) -> Option<NormalizedProgress> {
    let from = segment.from_offset().value();
    let to = segment.to_offset().value();
    let local = (progress.value() - from) / (to - from);

    NormalizedProgress::new(local).ok()
}

fn multiple_property_lookup(
    segments: &[KeyframeSegment],
    second_index: usize,
) -> Result<SegmentLookup, KeyframeError> {
    // Detection proves the initial run has the first property. Preserve its
    // complete membership and both first key allocations when extending it.
    let mut groups = [
        PropertySegmentGroup {
            property: segments[0].property().clone(),
            membership: SegmentMembership::Contiguous(0..second_index),
        },
        PropertySegmentGroup {
            property: segments[second_index].property().clone(),
            membership: SegmentMembership::Contiguous(second_index..second_index + 1),
        },
    ];
    for (index, segment) in segments.iter().enumerate().skip(second_index + 1) {
        let group_index = if segment.property() == &groups[0].property {
            0
        } else if segment.property() == &groups[1].property {
            1
        } else {
            let groups = continue_segment_groups(segments, second_index, index, groups);
            for group in &groups {
                validate_property_segments(&group.property, group.membership.view(segments))?;
            }
            return Ok(SegmentLookup::Grouped(groups));
        };
        groups[group_index].membership.push(index);
    }

    for group in &groups {
        validate_property_segments(&group.property, group.membership.view(segments))?;
    }
    Ok(SegmentLookup::Pair(Arc::new(groups)))
}

fn continue_segment_groups(
    segments: &[KeyframeSegment],
    second_index: usize,
    from_index: usize,
    prefix: [PropertySegmentGroup; 2],
) -> Vec<PropertySegmentGroup> {
    // Move the completed prefix without rebuilding or rehashing its members.
    let mut groups = Vec::with_capacity(4);
    groups.extend(prefix);
    let mut by_property: HashMap<&PropertyKey, usize> = HashMap::with_capacity(8);
    by_property.insert(segments[0].property(), 0);
    by_property.insert(segments[second_index].property(), 1);

    for (index, segment) in segments.iter().enumerate().skip(from_index) {
        let group_index = *by_property.entry(segment.property()).or_insert_with(|| {
            let group_index = groups.len();
            groups.push(PropertySegmentGroup {
                property: segment.property().clone(),
                membership: SegmentMembership::Contiguous(index..index),
            });
            group_index
        });
        groups[group_index].membership.push(index);
    }

    groups
}

fn validate_property_segments(
    property: &PropertyKey,
    segments: PropertySegments<'_>,
) -> Result<(), KeyframeError> {
    let first = segments.at(0);
    if first.from_offset().value() != 0.0 {
        return Err(KeyframeError::MissingInitialOffset {
            property: property.clone(),
        });
    }

    let mut previous_to = first.to_offset();

    for position in 1..segments.len() {
        let segment = segments.at(position);
        let actual = segment.from_offset();

        if actual.value() < previous_to.value() {
            // Every accepted prefix is contiguous with strictly increasing starts.
            // Numeric comparison intentionally equates positive and negative zero.
            let duplicate = segments.partition_point(position, |segment| {
                segment.from_offset().value() < actual.value()
            });
            if duplicate < position && segments.at(duplicate).from_offset() == actual {
                return Err(KeyframeError::DuplicateOffset {
                    property: property.clone(),
                    offset: actual,
                });
            }
            return Err(KeyframeError::OverlappingSegment {
                property: property.clone(),
                previous_to,
                actual,
            });
        }

        if actual.value() > previous_to.value() {
            return Err(KeyframeError::NonContiguousSegment {
                property: property.clone(),
                expected: previous_to,
                actual,
            });
        }

        previous_to = segment.to_offset();
    }

    if previous_to.value() != 1.0 {
        return Err(KeyframeError::MissingFinalOffset {
            property: property.clone(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        KeyframeAnimationId, KeyframeError, KeyframeSegment, KeyframeTiming, KeyframeTrack,
    };
    use crate::{
        AnimationDelay, AnimationDirection, AnimationDuration, AnimationPlayState,
        CompletionStatus, Easing, ElapsedTime, FillMode, InterpolableNumber,
        InterpolablePercentage, InterpolableValue, InterpolationError, IterationCount,
        LinearControlPoint, LinearEasing, NextFrameHint, PropertyKey, SampleClassification,
        SampleValue, SampledPropertyResult, TimingError, TransformValue, UnitRatio, ValueFamily,
    };

    #[test]
    fn keyframe_track_preserves_resolved_identity_timing_and_segments() {
        let track = fade_track(
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                AnimationDuration::from_secs(2.0).unwrap(),
                IterationCount::finite(2.0).unwrap(),
                AnimationDirection::Alternate,
                FillMode::Both,
            )
            .unwrap(),
        );

        assert_eq!(track.id().as_str(), "fade");
        assert_eq!(track.timing().duration().as_secs(), 2.0);
        assert_eq!(track.segments().len(), 1);
        assert_eq!(track.segments()[0].property().as_str(), "opacity");
    }

    #[test]
    fn keyframe_timing_rejects_unrepresentable_finite_active_duration() {
        let duration = AnimationDuration::from_secs(f64::MAX).unwrap();
        let iterations = IterationCount::finite(2.0).unwrap();

        assert_eq!(
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                duration,
                iterations,
                AnimationDirection::Normal,
                FillMode::Both,
            ),
            Err(TimingError::UnrepresentableActiveDuration {
                duration,
                iterations: 2.0,
            })
        );
    }

    #[test]
    fn keyframe_track_rejects_empty_identity_and_empty_segments() {
        assert_eq!(
            KeyframeAnimationId::new(" "),
            Err(KeyframeError::EmptyAnimationId)
        );
        let id = KeyframeAnimationId::new("fade").unwrap();
        assert_eq!(
            KeyframeTrack::new(id.clone(), default_timing(), Vec::new()),
            Err(KeyframeError::EmptySegments { animation: id })
        );
    }

    #[test]
    fn keyframe_segment_rejects_degenerate_offsets_and_mismatched_values() {
        let property = PropertyKey::new("opacity").unwrap();
        let error = KeyframeSegment::new(
            property.clone(),
            UnitRatio::new(0.5).unwrap(),
            UnitRatio::new(0.5).unwrap(),
            number(0.0),
            number(1.0),
            Easing::linear(),
        )
        .unwrap_err();
        assert_eq!(
            error,
            KeyframeError::DegenerateSegment {
                property: property.clone(),
                offset: UnitRatio::new(0.5).unwrap(),
            }
        );

        let mismatch = KeyframeSegment::new(
            property,
            UnitRatio::new(0.0).unwrap(),
            UnitRatio::new(1.0).unwrap(),
            number(0.0),
            InterpolableValue::percentage(InterpolablePercentage::new(1.0).unwrap()),
            Easing::linear(),
        )
        .unwrap_err();
        assert!(matches!(
            mismatch,
            KeyframeError::Interpolation(error)
                if matches!(*error, InterpolationError::MismatchedFamilies { .. })
        ));
    }

    #[test]
    fn wrapped_keyframe_diagnostics_expose_interpolation_source() {
        let error = KeyframeSegment::new(
            PropertyKey::new("opacity").unwrap(),
            UnitRatio::new(0.0).unwrap(),
            UnitRatio::new(1.0).unwrap(),
            number(0.0),
            InterpolableValue::percentage(InterpolablePercentage::new(1.0).unwrap()),
            Easing::linear(),
        )
        .unwrap_err();

        assert_eq!(error.to_string(), "keyframe interpolation failed");
        assert_eq!(
            std::error::Error::source(&error).unwrap().to_string(),
            "property opacity cannot interpolate number to percentage"
        );
    }

    #[test]
    fn keyframe_track_rejects_missing_duplicate_and_noncontiguous_offsets() {
        let missing_initial = KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            default_timing(),
            vec![segment("opacity", 0.25, 1.0, 0.0, 1.0, Easing::linear())],
        )
        .unwrap_err();
        assert!(matches!(
            missing_initial,
            KeyframeError::MissingInitialOffset { .. }
        ));

        let missing_final = KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            default_timing(),
            vec![segment("opacity", 0.0, 0.75, 0.0, 0.75, Easing::linear())],
        )
        .unwrap_err();
        assert!(matches!(
            missing_final,
            KeyframeError::MissingFinalOffset { .. }
        ));

        let gap = KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            default_timing(),
            vec![
                segment("opacity", 0.0, 0.25, 0.0, 0.25, Easing::linear()),
                segment("opacity", 0.5, 1.0, 0.5, 1.0, Easing::linear()),
            ],
        )
        .unwrap_err();
        assert!(matches!(gap, KeyframeError::NonContiguousSegment { .. }));

        let duplicate = KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            default_timing(),
            vec![
                segment("opacity", 0.0, 0.5, 0.0, 0.5, Easing::linear()),
                segment("opacity", 0.5, 0.75, 0.5, 0.75, Easing::linear()),
                segment("opacity", 0.5, 1.0, 0.5, 1.0, Easing::linear()),
            ],
        )
        .unwrap_err();
        assert!(matches!(
            duplicate,
            KeyframeError::DuplicateOffset { offset, .. }
                if offset == UnitRatio::new(0.5).unwrap()
        ));

        let overlap = KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            default_timing(),
            vec![
                segment("opacity", 0.0, 0.75, 0.0, 0.75, Easing::linear()),
                segment("opacity", 0.5, 1.0, 0.5, 1.0, Easing::linear()),
            ],
        )
        .unwrap_err();
        assert!(matches!(overlap, KeyframeError::OverlappingSegment { .. }));
    }

    #[test]
    fn distinct_keyframe_segments_select_values_at_and_around_boundary() {
        let track = distinct_segment_track(default_timing(), Easing::linear());
        // Independently derived slopes: 50 before 0.4, 100 after 0.4.
        // The discontinuity makes choosing the wrong segment observable.
        for (elapsed, expected) in [
            (0.2, 20.0),
            (0.399_999, 29.999_95),
            (0.4, 80.0),
            (0.400_001, 80.000_1),
            (0.7, 110.0),
        ] {
            assert_distinct_sample(&track, elapsed, expected, false);
        }
    }

    #[test]
    fn active_keyframe_arithmetic_failure_is_failed_running_and_may_change() {
        let track = KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            default_timing(),
            vec![overflow_segment("opacity", Easing::linear())],
        )
        .unwrap();

        let samples = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Failed);
        assert_eq!(samples[0].completion(), CompletionStatus::Running);
        assert_eq!(samples[0].next_frame(), NextFrameHint::MayChange);
        assert_arithmetic_error(&samples[0], "opacity", ValueFamily::Number);
    }

    #[test]
    fn keyframe_percentage_sample_uses_public_front_door() {
        let track = KeyframeTrack::new(
            KeyframeAnimationId::new("width").unwrap(),
            default_timing(),
            vec![percentage_segment(
                "width",
                0.0,
                1.0,
                -0.5,
                1.5,
                Easing::linear(),
            )],
        )
        .unwrap();

        let samples = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Active);
        assert_sampled_percentage(&samples[0], "width", 0.5);
    }

    #[test]
    fn after_keyframe_arithmetic_failure_is_finished_and_stable() {
        let track = KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                AnimationDuration::from_secs(1.0).unwrap(),
                IterationCount::finite(2.5).unwrap(),
                AnimationDirection::Normal,
                FillMode::Forwards,
            )
            .unwrap(),
            vec![overflow_segment("opacity", Easing::linear())],
        )
        .unwrap();

        let samples = track.sample(
            ElapsedTime::from_secs(2.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Failed);
        assert_eq!(samples[0].completion(), CompletionStatus::Finished);
        assert_eq!(
            samples[0].next_frame(),
            NextFrameHint::StableUntilExternalInput
        );
        assert_arithmetic_error(&samples[0], "opacity", ValueFamily::Number);
    }

    #[test]
    fn later_exact_keyframe_endpoint_recovers_from_earlier_arithmetic_failure() {
        let track = KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                AnimationDuration::from_secs(1.0).unwrap(),
                IterationCount::finite(1.0).unwrap(),
                AnimationDirection::Normal,
                FillMode::Forwards,
            )
            .unwrap(),
            vec![overflow_segment("opacity", Easing::linear())],
        )
        .unwrap();

        let early = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );
        let later = track.sample(
            ElapsedTime::from_secs(1.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(early[0].classification(), SampleClassification::Failed);
        assert_eq!(later[0].classification(), SampleClassification::Filling);
        assert_eq!(later[0].completion(), CompletionStatus::Finished);
        assert_sampled_number(&later[0], "opacity", -f64::MAX);
    }

    #[test]
    fn multiple_properties_emit_one_sample_per_property() {
        let track = KeyframeTrack::new(
            KeyframeAnimationId::new("combo").unwrap(),
            default_timing(),
            vec![
                segment("opacity", 0.0, 1.0, 0.0, 1.0, Easing::linear()),
                segment("scale", 0.0, 1.0, 1.0, 2.0, Easing::linear()),
            ],
        )
        .unwrap();

        let samples = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples.len(), 2);
        assert_sampled_number(&samples[0], "opacity", 0.5);
        assert_sampled_number(&samples[1], "scale", 1.5);
    }

    #[test]
    fn second_segment_steps_use_local_progress() {
        let track = distinct_segment_track(
            default_timing(),
            Easing::steps(crate::Steps::new(2, crate::StepPosition::JumpEnd).unwrap()),
        );
        // Global 0.55 is local 0.25: jump-end holds the segment's initial 80.
        // Incorrectly easing global progress would produce 110.
        assert_distinct_sample(&track, 0.55, 80.0, false);
    }

    #[test]
    fn keyframe_segment_samples_owned_linear_function_through_public_front_door() {
        let linear = LinearEasing::new(vec![
            LinearControlPoint::new(0.0, 0.0),
            LinearControlPoint::new(1.0, 0.5),
        ])
        .unwrap();
        let track = KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            default_timing(),
            vec![segment(
                "opacity",
                0.0,
                1.0,
                0.0,
                1.0,
                Easing::linear_function(linear),
            )],
        )
        .unwrap();

        assert!(track.segments()[0].easing().as_linear_function().is_some());
        let samples = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_sampled_number(&samples[0], "opacity", 0.25);
    }

    #[test]
    fn endpoint_progress_selects_distinct_last_segment() {
        let track = distinct_segment_track(default_timing(), Easing::linear());
        assert_distinct_sample(&track, 1.0, 140.0, true);
    }

    #[test]
    fn before_fill_samples_initial_keyframe_when_fill_allows() {
        let track = fade_track(
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(1.0).unwrap(),
                AnimationDuration::from_secs(1.0).unwrap(),
                IterationCount::finite(1.0).unwrap(),
                AnimationDirection::Normal,
                FillMode::Backwards,
            )
            .unwrap(),
        );

        let samples = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Filling);
        assert_sampled_number(&samples[0], "opacity", 0.0);
    }

    #[test]
    fn delayed_keyframe_running_sample_next_frame_may_change() {
        let track = fade_track(
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(1.0).unwrap(),
                AnimationDuration::from_secs(1.0).unwrap(),
                IterationCount::finite(1.0).unwrap(),
                AnimationDirection::Normal,
                FillMode::Backwards,
            )
            .unwrap(),
        );

        let samples = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Filling);
        assert_eq!(samples[0].completion(), CompletionStatus::NotStarted);
        assert_eq!(samples[0].next_frame(), NextFrameHint::MayChange);
        assert_sampled_number(&samples[0], "opacity", 0.0);
    }

    #[test]
    fn backwards_fill_keyframe_step_easing_uses_before_flag() {
        let track = KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(1.0).unwrap(),
                AnimationDuration::from_secs(1.0).unwrap(),
                IterationCount::finite(1.0).unwrap(),
                AnimationDirection::Normal,
                FillMode::Backwards,
            )
            .unwrap(),
            vec![segment("opacity", 0.0, 1.0, 0.0, 1.0, Easing::step_start())],
        )
        .unwrap();

        let samples = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Filling);
        assert_sampled_number(&samples[0], "opacity", 0.0);
    }

    #[test]
    fn after_fill_samples_final_keyframe_when_fill_allows() {
        let track = fade_track(
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                AnimationDuration::from_secs(1.0).unwrap(),
                IterationCount::finite(1.0).unwrap(),
                AnimationDirection::Normal,
                FillMode::Forwards,
            )
            .unwrap(),
        );

        let samples = track.sample(
            ElapsedTime::from_secs(1.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Filling);
        assert_sampled_number(&samples[0], "opacity", 1.0);
    }

    #[test]
    fn after_fill_reverse_keyframe_step_easing_uses_before_flag() {
        let track = KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                AnimationDuration::from_secs(1.0).unwrap(),
                IterationCount::finite(1.0).unwrap(),
                AnimationDirection::Reverse,
                FillMode::Forwards,
            )
            .unwrap(),
            vec![segment("opacity", 0.0, 1.0, 0.0, 1.0, Easing::step_start())],
        )
        .unwrap();

        let samples = track.sample(
            ElapsedTime::from_secs(1.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Filling);
        assert_sampled_number(&samples[0], "opacity", 0.0);
    }

    #[test]
    fn alternate_direction_selects_distinct_segment_using_directed_progress() {
        let timing = KeyframeTiming::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(2.0).unwrap(),
            AnimationDirection::Alternate,
            FillMode::None,
        )
        .unwrap();
        let track = distinct_segment_track(timing, Easing::linear());
        // Second iteration reverses 0.3 into 0.7, selecting the 80..140 segment.
        assert_distinct_sample(&track, 1.3, 110.0, false);
    }

    #[test]
    fn fractional_final_iteration_after_fill_samples_fractional_endpoint() {
        let track = fade_track(
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                AnimationDuration::from_secs(1.0).unwrap(),
                IterationCount::finite(2.25).unwrap(),
                AnimationDirection::Normal,
                FillMode::Forwards,
            )
            .unwrap(),
        );

        let samples = track.sample(
            ElapsedTime::from_secs(2.25).unwrap(),
            AnimationPlayState::Running,
        );

        assert_sampled_number(&samples[0], "opacity", 0.25);
    }

    #[test]
    fn paused_keyframe_sample_is_stable_until_external_input() {
        let track = fade_track(default_timing());
        let samples = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Paused,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Active);
        assert_eq!(
            samples[0].next_frame(),
            NextFrameHint::StableUntilExternalInput
        );
    }

    #[test]
    fn zero_duration_keyframe_sample_uses_endpoint_fill_progress() {
        let track = fade_track(
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                AnimationDuration::from_secs(0.0).unwrap(),
                IterationCount::finite(1.0).unwrap(),
                AnimationDirection::Normal,
                FillMode::Forwards,
            )
            .unwrap(),
        );

        let samples = track.sample(
            ElapsedTime::from_secs(0.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Filling);
        assert_sampled_number(&samples[0], "opacity", 1.0);
    }

    #[test]
    fn zero_iteration_keyframe_forwards_fill_samples_first_endpoint() {
        let track = fade_track(
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                AnimationDuration::from_secs(1.0).unwrap(),
                IterationCount::finite(0.0).unwrap(),
                AnimationDirection::Normal,
                FillMode::Forwards,
            )
            .unwrap(),
        );

        let samples = track.sample(
            ElapsedTime::from_secs(0.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Filling);
        assert_sampled_number(&samples[0], "opacity", 0.0);
    }

    #[test]
    fn zero_iteration_keyframe_reverse_fill_samples_last_endpoint() {
        let track = fade_track(
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                AnimationDuration::from_secs(1.0).unwrap(),
                IterationCount::finite(0.0).unwrap(),
                AnimationDirection::Reverse,
                FillMode::Forwards,
            )
            .unwrap(),
        );

        let samples = track.sample(
            ElapsedTime::from_secs(0.0).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Filling);
        assert_sampled_number(&samples[0], "opacity", 1.0);
    }

    #[test]
    fn infinite_keyframe_sample_remains_active_at_late_elapsed_time() {
        let track = fade_track(
            KeyframeTiming::try_new(
                AnimationDelay::from_secs(0.0).unwrap(),
                AnimationDuration::from_secs(1.0).unwrap(),
                IterationCount::infinite(),
                AnimationDirection::Normal,
                FillMode::None,
            )
            .unwrap(),
        );

        let samples = track.sample(
            ElapsedTime::from_secs(10.25).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(samples[0].classification(), SampleClassification::Active);
        assert_sampled_number(&samples[0], "opacity", 0.25);
    }

    #[test]
    fn unsupported_keyframe_interpolation_reports_unsupported_sample() {
        let track = KeyframeTrack::new(
            KeyframeAnimationId::new("move").unwrap(),
            default_timing(),
            vec![
                KeyframeSegment::new(
                    PropertyKey::new("transform").unwrap(),
                    UnitRatio::new(0.0).unwrap(),
                    UnitRatio::new(1.0).unwrap(),
                    InterpolableValue::transform(TransformValue::unsupported("translate")),
                    InterpolableValue::transform(TransformValue::unsupported("scale")),
                    Easing::linear(),
                )
                .unwrap(),
            ],
        )
        .unwrap();

        let samples = track.sample(
            ElapsedTime::from_secs(0.5).unwrap(),
            AnimationPlayState::Running,
        );

        assert_eq!(
            samples[0].classification(),
            SampleClassification::Unsupported
        );
        assert!(matches!(samples[0].value(), SampleValue::Unsupported(_)));
    }

    fn distinct_segment_track(timing: KeyframeTiming, second_easing: Easing) -> KeyframeTrack {
        KeyframeTrack::new(
            KeyframeAnimationId::new("distinct").unwrap(),
            timing,
            vec![
                segment("opacity", 0.0, 0.4, 10.0, 30.0, Easing::linear()),
                segment("opacity", 0.4, 1.0, 80.0, 140.0, second_easing),
            ],
        )
        .unwrap()
    }

    fn assert_distinct_sample(track: &KeyframeTrack, elapsed: f64, expected: f64, finished: bool) {
        let samples = track.sample(
            ElapsedTime::from_secs(elapsed).unwrap(),
            AnimationPlayState::Running,
        );
        assert_eq!(samples.len(), 1);
        let sample = &samples[0];
        assert_eq!(
            sample.classification(),
            if finished {
                SampleClassification::Filling
            } else {
                SampleClassification::Active
            }
        );
        assert_eq!(
            sample.completion(),
            if finished {
                CompletionStatus::Finished
            } else {
                CompletionStatus::Running
            }
        );
        assert_eq!(
            sample.next_frame(),
            if finished {
                NextFrameHint::StableUntilExternalInput
            } else {
                NextFrameHint::MayChange
            }
        );
        let SampleValue::Value(value) = sample.value() else {
            panic!("expected value, got {sample:?}")
        };
        assert_eq!(value.property().as_str(), "opacity");
        let InterpolableValue::Number(value) = value.value() else {
            panic!("expected number")
        };
        assert!(
            (value.value() - expected).abs() < 1e-9,
            "at {elapsed}: expected {expected}, got {}",
            value.value()
        );
    }

    fn default_timing() -> KeyframeTiming {
        KeyframeTiming::try_new(
            AnimationDelay::from_secs(0.0).unwrap(),
            AnimationDuration::from_secs(1.0).unwrap(),
            IterationCount::finite(1.0).unwrap(),
            AnimationDirection::Normal,
            FillMode::Both,
        )
        .unwrap()
    }

    fn number(value: f64) -> InterpolableValue {
        InterpolableValue::number(InterpolableNumber::new(value).unwrap())
    }

    fn segment(
        property: &str,
        from_offset: f64,
        to_offset: f64,
        from: f64,
        to: f64,
        easing: Easing,
    ) -> KeyframeSegment {
        KeyframeSegment::new(
            PropertyKey::new(property).unwrap(),
            UnitRatio::new(from_offset).unwrap(),
            UnitRatio::new(to_offset).unwrap(),
            number(from),
            number(to),
            easing,
        )
        .unwrap()
    }

    fn percentage_segment(
        property: &str,
        from_offset: f64,
        to_offset: f64,
        from: f64,
        to: f64,
        easing: Easing,
    ) -> KeyframeSegment {
        KeyframeSegment::new(
            PropertyKey::new(property).unwrap(),
            UnitRatio::new(from_offset).unwrap(),
            UnitRatio::new(to_offset).unwrap(),
            InterpolableValue::percentage(InterpolablePercentage::new(from).unwrap()),
            InterpolableValue::percentage(InterpolablePercentage::new(to).unwrap()),
            easing,
        )
        .unwrap()
    }

    fn overflow_segment(property: &str, easing: Easing) -> KeyframeSegment {
        KeyframeSegment::new(
            PropertyKey::new(property).unwrap(),
            UnitRatio::new(0.0).unwrap(),
            UnitRatio::new(1.0).unwrap(),
            number(f64::MAX),
            number(-f64::MAX),
            easing,
        )
        .unwrap()
    }

    fn assert_sampled_number(sample: &SampledPropertyResult, property: &str, expected: f64) {
        let SampleValue::Value(sampled) = sample.value() else {
            panic!("expected sampled value, got {sample:?}");
        };
        assert_eq!(sampled.property().as_str(), property);
        let InterpolableValue::Number(number) = sampled.value() else {
            panic!("expected number value, got {:?}", sampled.value());
        };
        assert_eq!(number.value(), expected);
    }

    fn assert_sampled_percentage(sample: &SampledPropertyResult, property: &str, expected: f64) {
        let SampleValue::Value(sampled) = sample.value() else {
            panic!("expected sampled value, got {sample:?}");
        };
        assert_eq!(sampled.property().as_str(), property);
        let InterpolableValue::Percentage(percentage) = sampled.value() else {
            panic!("expected percentage value, got {:?}", sampled.value());
        };
        assert_eq!(percentage.value(), expected);
    }

    fn assert_arithmetic_error(
        sample: &SampledPropertyResult,
        property: &str,
        family: ValueFamily,
    ) {
        let SampleValue::Error(InterpolationError::NonFiniteInterpolationResult(result)) =
            sample.value()
        else {
            panic!("expected arithmetic error, got {sample:?}");
        };

        assert_eq!(result.property().as_str(), property);
        assert_eq!(result.family(), family);
        assert!(!result.value().is_finite());
    }

    fn fade_track(timing: KeyframeTiming) -> KeyframeTrack {
        KeyframeTrack::new(
            KeyframeAnimationId::new("fade").unwrap(),
            timing,
            vec![segment("opacity", 0.0, 1.0, 0.0, 1.0, Easing::linear())],
        )
        .unwrap()
    }
}
