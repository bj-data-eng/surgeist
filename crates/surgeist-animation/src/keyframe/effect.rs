//! Validated computed-frame sampling under the selected Web Animations algorithm.

use core::fmt;
use std::collections::HashSet;

use crate::composition::{PropertyValueOperations, compose_value};
use crate::easing::{Easing, EasingError};
use crate::interpolation::{
    InterpolableValue, InterpolationError, InterpolationProgress, PropertyKey,
};
use crate::keyframe_input::{CompositeOperation, NormalizedKeyframes};
use crate::progress::{EasingInput, UnitRatio};
use crate::timing::{EffectTimingSample, TimingPhase};

/// Selects native fallback or complete caller-assembled CSS endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyframeEndpointSource {
    /// Missing endpoints use current underlying value, replace, and linear.
    Native,
    /// Every animated property has caller-supplied zero and one endpoints.
    CssAssembled,
}

/// Supplies one computed frame without erasing its specified offset metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedKeyframeInput {
    pub specified_offset: Option<UnitRatio>,
    pub computed_offset: UnitRatio,
    pub easing: Easing,
    pub composite: Option<CompositeOperation>,
    pub properties: Vec<(PropertyKey, InterpolableValue)>,
}

/// Holds one frame in a validated, stably ordered computed collection.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedKeyframe {
    specified_offset: Option<UnitRatio>,
    computed_offset: UnitRatio,
    easing: Easing,
    composite: Option<CompositeOperation>,
    properties: Box<[(PropertyKey, InterpolableValue)]>,
}
impl ComputedKeyframe {
    pub const fn specified_offset(&self) -> Option<UnitRatio> {
        self.specified_offset
    }
    pub const fn computed_offset(&self) -> UnitRatio {
        self.computed_offset
    }
    pub const fn easing(&self) -> &Easing {
        &self.easing
    }
    pub const fn composite(&self) -> Option<CompositeOperation> {
        self.composite
    }
    pub fn properties(&self) -> &[(PropertyKey, InterpolableValue)] {
        &self.properties
    }
}

/// Stores caller-computed values, ordered offsets, and endpoint source.
///
/// Values must be recomputed by their caller for every influencing context
/// revision. This object neither resolves CSS values nor caches target context.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedKeyframes {
    frames: Box<[ComputedKeyframe]>,
    endpoint_source: KeyframeEndpointSource,
}

/// Identifies an invalid computed collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputedKeyframeErrorKind {
    SpecifiedOffsetMismatch,
    OffsetOrder,
    DuplicateProperty,
    MissingCssEndpoint,
}

/// Reports malformed computed input without publishing a partial collection.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedKeyframeError {
    kind: ComputedKeyframeErrorKind,
    frame_index: usize,
    property: Option<PropertyKey>,
}
impl ComputedKeyframeError {
    pub const fn kind(&self) -> ComputedKeyframeErrorKind {
        self.kind
    }
    pub const fn frame_index(&self) -> usize {
        self.frame_index
    }
    pub const fn property(&self) -> Option<&PropertyKey> {
        self.property.as_ref()
    }
}
impl fmt::Display for ComputedKeyframeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "computed frame {}: {:?}", self.frame_index, self.kind)?;
        if let Some(property) = &self.property {
            write!(f, " for {}", property.as_str())?;
        }
        Ok(())
    }
}
impl std::error::Error for ComputedKeyframeError {}

/// Supplies target applicability and the current underlying value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyframeTarget<'a> {
    Absent,
    Uncalculable,
    NotAnimatable,
    Available(&'a InterpolableValue),
}

/// Couples a sampling target with its effect-level composite operation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyframeSampleInput<'a> {
    pub target: KeyframeTarget<'a>,
    pub effect_composite: CompositeOperation,
}

/// Identifies the operation that rejected a property sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyframeSampleStage {
    Interval,
    Composition,
    Easing,
    Interpolation,
}

/// Preserves the property, selected endpoint, and original operation failure.
#[derive(Debug, PartialEq)]
pub struct KeyframeSampleError<E> {
    property: PropertyKey,
    stage: KeyframeSampleStage,
    frame_index: Option<usize>,
    detail: SampleFailure<E>,
}
#[derive(Debug, PartialEq)]
enum SampleFailure<E> {
    Interval,
    Easing(Box<EasingError>),
    Progress(Box<InterpolationError>),
    Operation(Box<E>),
}
impl<E> KeyframeSampleError<E> {
    pub const fn property(&self) -> &PropertyKey {
        &self.property
    }
    pub const fn stage(&self) -> KeyframeSampleStage {
        self.stage
    }
    /// Returns `None` for synthetic endpoints or property interpolation.
    pub const fn frame_index(&self) -> Option<usize> {
        self.frame_index
    }
    pub fn operation_error(&self) -> Option<&E> {
        match &self.detail {
            SampleFailure::Operation(error) => Some(error.as_ref()),
            _ => None,
        }
    }
    pub fn easing_error(&self) -> Option<&EasingError> {
        match &self.detail {
            SampleFailure::Easing(error) => Some(error.as_ref()),
            _ => None,
        }
    }
    pub fn progress_error(&self) -> Option<&InterpolationError> {
        match &self.detail {
            SampleFailure::Progress(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}
impl<E: fmt::Display> fmt::Display for KeyframeSampleError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "keyframe {} {:?} failed",
            self.property.as_str(),
            self.stage
        )?;
        if let Some(index) = self.frame_index {
            write!(f, " at frame {index}")?;
        }
        match &self.detail {
            SampleFailure::Interval => f.write_str(": no valid interval"),
            SampleFailure::Easing(error) => write!(f, ": {error}"),
            SampleFailure::Progress(error) => write!(f, ": {error}"),
            SampleFailure::Operation(error) => write!(f, ": {error}"),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for KeyframeSampleError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.detail {
            SampleFailure::Easing(error) => Some(error.as_ref()),
            SampleFailure::Progress(error) => Some(error.as_ref()),
            SampleFailure::Operation(error) => Some(error.as_ref()),
            SampleFailure::Interval => None,
        }
    }
}

impl ComputedKeyframes {
    /// Validates offset consistency, computed order, and property uniqueness.
    ///
    /// CSS collections require zero/one endpoints for every represented property.
    /// Specified offsets must equal their computed offsets. Missing specified
    /// offsets remain missing; normalization owns their distribution. Ties retain
    /// their supplied order.
    pub fn new(
        frames: Vec<ComputedKeyframeInput>,
        endpoint_source: KeyframeEndpointSource,
    ) -> Result<Self, ComputedKeyframeError> {
        let mut previous = None;
        let mut properties = Vec::new();
        let mut all_properties = HashSet::new();
        for (index, frame) in frames.iter().enumerate() {
            if frame
                .specified_offset
                .is_some_and(|offset| offset != frame.computed_offset)
            {
                return Err(ComputedKeyframeError {
                    kind: ComputedKeyframeErrorKind::SpecifiedOffsetMismatch,
                    frame_index: index,
                    property: None,
                });
            }
            if previous.is_some_and(|offset| offset > frame.computed_offset.value()) {
                return Err(ComputedKeyframeError {
                    kind: ComputedKeyframeErrorKind::OffsetOrder,
                    frame_index: index,
                    property: None,
                });
            }
            previous = Some(frame.computed_offset.value());
            let mut seen = HashSet::new();
            for (property, _) in &frame.properties {
                if !seen.insert(property) {
                    return Err(ComputedKeyframeError {
                        kind: ComputedKeyframeErrorKind::DuplicateProperty,
                        frame_index: index,
                        property: Some(property.clone()),
                    });
                }
                if all_properties.insert(property) {
                    properties.push((index, property));
                }
            }
        }
        if endpoint_source == KeyframeEndpointSource::CssAssembled {
            for (index, property) in properties {
                for offset in [0.0, 1.0] {
                    if !frames.iter().any(|frame| {
                        frame.computed_offset.value() == offset
                            && frame
                                .properties
                                .iter()
                                .any(|(candidate, _)| candidate == property)
                    }) {
                        return Err(ComputedKeyframeError {
                            kind: ComputedKeyframeErrorKind::MissingCssEndpoint,
                            frame_index: index,
                            property: Some(property.clone()),
                        });
                    }
                }
            }
        }
        Ok(Self {
            endpoint_source,
            frames: frames
                .into_iter()
                .map(|frame| ComputedKeyframe {
                    specified_offset: frame.specified_offset,
                    computed_offset: frame.computed_offset,
                    easing: frame.easing,
                    composite: frame.composite,
                    properties: frame.properties.into_boxed_slice(),
                })
                .collect(),
        })
    }

    /// Copies normalized metadata whose caller has already computed its values.
    pub fn from_normalized(
        frames: &NormalizedKeyframes<InterpolableValue>,
        source: KeyframeEndpointSource,
    ) -> Result<Self, ComputedKeyframeError> {
        Self::new(
            frames
                .frames()
                .iter()
                .map(|frame| ComputedKeyframeInput {
                    specified_offset: frame.specified_offset(),
                    computed_offset: frame.computed_offset(),
                    easing: frame.easing().clone(),
                    composite: frame.composite(),
                    properties: frame.properties().to_vec(),
                })
                .collect(),
            source,
        )
    }

    pub fn frames(&self) -> &[ComputedKeyframe] {
        &self.frames
    }
    pub const fn endpoint_source(&self) -> KeyframeEndpointSource {
        self.endpoint_source
    }

    /// Samples already-selected finite iteration progress under current §5.3.4.
    ///
    /// Progress can be outside `[0,1]`. Native interval easing receives local
    /// distance without an additional before flag. Unresolved progress or an
    /// inapplicable target returns `None`; an empty property track returns its
    /// underlying value. The output has already composed its endpoints against
    /// underlying and must not be composited a second time by an effect stack.
    ///
    /// # Errors
    /// Returns typed easing, distance, or property operation failures atomically.
    ///
    /// ```
    /// use surgeist_animation::composition::PropertyValueOperations;
    /// use surgeist_animation::easing::Easing;
    /// use surgeist_animation::interpolation::{
    ///     DiscreteValue, InterpolableValue, InterpolationProgress, PropertyKey,
    /// };
    /// use surgeist_animation::keyframe::{
    ///     ComputedKeyframeInput, ComputedKeyframes, KeyframeEndpointSource,
    ///     KeyframeSampleInput, KeyframeTarget,
    /// };
    /// use surgeist_animation::keyframe_input::CompositeOperation;
    /// use surgeist_animation::progress::UnitRatio;
    ///
    /// // The caller selected a discrete animation type for this property.
    /// struct Discrete;
    /// impl PropertyValueOperations for Discrete {
    ///     type Error = &'static str;
    ///     fn interpolate(&self, _: &PropertyKey, a: &InterpolableValue,
    ///         b: &InterpolableValue, p: InterpolationProgress)
    ///         -> Result<InterpolableValue, Self::Error> {
    ///         Ok(if p.value() < 0.5 { a } else { b }.clone())
    ///     }
    ///     fn add(&self, _: &PropertyKey, _: &InterpolableValue,
    ///         _: &InterpolableValue) -> Result<InterpolableValue, Self::Error> {
    ///         Err("addition unavailable")
    ///     }
    ///     fn accumulate(&self, _: &PropertyKey, _: &InterpolableValue,
    ///         _: &InterpolableValue) -> Result<InterpolableValue, Self::Error> {
    ///         Err("accumulation unavailable")
    ///     }
    /// }
    /// let property = PropertyKey::new("--state").unwrap();
    /// let start = InterpolableValue::discrete(DiscreteValue::new("closed").unwrap());
    /// let end = InterpolableValue::discrete(DiscreteValue::new("open").unwrap());
    /// let frames = [start.clone(), end.clone()].into_iter().enumerate()
    ///     .map(|(index, value)| ComputedKeyframeInput {
    ///         specified_offset: None,
    ///         computed_offset: UnitRatio::new(index as f64).unwrap(),
    ///         easing: Easing::linear(), composite: None,
    ///         properties: vec![(property.clone(), value)],
    ///     }).collect();
    /// let effect = ComputedKeyframes::new(frames, KeyframeEndpointSource::Native)
    ///     .unwrap();
    /// let value = effect.sample(&property, KeyframeSampleInput {
    ///     target: KeyframeTarget::Available(&start),
    ///     effect_composite: CompositeOperation::Replace,
    /// }, Some(InterpolationProgress::new(0.5).unwrap()), &Discrete).unwrap();
    /// assert_eq!(value, Some(end));
    /// ```
    pub fn sample<P: PropertyValueOperations + ?Sized>(
        &self,
        property: &PropertyKey,
        input: KeyframeSampleInput<'_>,
        progress: Option<InterpolationProgress>,
        operations: &P,
    ) -> Result<Option<InterpolableValue>, KeyframeSampleError<P::Error>> {
        self.sample_inner(
            property,
            input,
            progress.map(|p| p.value()),
            false,
            false,
            operations,
        )
    }

    /// Adapts a coherent effect timing sample without applying effect easing again.
    ///
    /// Native input uses eased iteration progress; its effect before flag has
    /// already been consumed and is not passed to interval easing (selected
    /// §5.3.4 step 8). CSS assembly instead uses directed progress, holds delay
    /// values without interval easing, and preserves its interval step-side
    /// flag as in normalized CSS tracks. This prevents whole-effect CSS easing
    /// from being applied in addition to CSS keyframe timing functions.
    pub fn sample_timing<P: PropertyValueOperations + ?Sized>(
        &self,
        property: &PropertyKey,
        input: KeyframeSampleInput<'_>,
        timing: EffectTimingSample,
        operations: &P,
    ) -> Result<Option<InterpolableValue>, KeyframeSampleError<P::Error>> {
        match self.endpoint_source {
            KeyframeEndpointSource::Native => self.sample_inner(
                property,
                input,
                timing.eased_progress().map(|p| p.value()),
                false,
                false,
                operations,
            ),
            KeyframeEndpointSource::CssAssembled => self.sample_inner(
                property,
                input,
                timing.directed_progress().map(|p| p.value()),
                timing.easing_before_flag(),
                timing.phase() == TimingPhase::Before,
                operations,
            ),
        }
    }

    fn sample_inner<P: PropertyValueOperations + ?Sized>(
        &self,
        property: &PropertyKey,
        input: KeyframeSampleInput<'_>,
        progress: Option<f64>,
        before_flag: bool,
        hold: bool,
        operations: &P,
    ) -> Result<Option<InterpolableValue>, KeyframeSampleError<P::Error>> {
        let Some(progress) = progress else {
            return Ok(None);
        };
        let KeyframeTarget::Available(underlying) = input.target else {
            return Ok(None);
        };
        let linear = Easing::linear();
        let mut endpoints: Vec<_> = self
            .frames
            .iter()
            .enumerate()
            .filter_map(|(index, frame)| {
                frame
                    .properties
                    .iter()
                    .find(|(key, _)| key == property)
                    .map(|(_, value)| Endpoint {
                        offset: frame.computed_offset.value(),
                        value,
                        easing: &frame.easing,
                        composite: frame.composite.unwrap_or(input.effect_composite),
                        frame_index: Some(index),
                    })
            })
            .collect();
        if endpoints.is_empty() {
            return Ok(Some(underlying.clone()));
        }
        if self.endpoint_source == KeyframeEndpointSource::Native {
            if !endpoints.iter().any(|frame| frame.offset == 0.0) {
                endpoints.insert(
                    0,
                    Endpoint {
                        offset: 0.0,
                        value: underlying,
                        easing: &linear,
                        composite: CompositeOperation::Replace,
                        frame_index: None,
                    },
                );
            }
            if !endpoints.iter().any(|frame| frame.offset == 1.0) {
                endpoints.push(Endpoint {
                    offset: 1.0,
                    value: underlying,
                    easing: &linear,
                    composite: CompositeOperation::Replace,
                    frame_index: None,
                });
            }
        }
        let single =
            if progress < 0.0 && endpoints.iter().filter(|frame| frame.offset == 0.0).count() > 1 {
                endpoints.first()
            } else if progress >= 1.0
                && endpoints.iter().filter(|frame| frame.offset == 1.0).count() > 1
            {
                endpoints.last()
            } else {
                None
            };
        if let Some(endpoint) = single {
            return compose_endpoint(property, underlying, endpoint, operations).map(Some);
        }
        let start = endpoints
            .iter()
            .rposition(|frame| frame.offset <= progress && frame.offset < 1.0)
            .or_else(|| endpoints.iter().rposition(|frame| frame.offset == 0.0));
        let pair = start.and_then(|index| {
            endpoints
                .get(index)
                .zip(index.checked_add(1).and_then(|next| endpoints.get(next)))
        });
        let Some((start, end)) = pair else {
            return Err(KeyframeSampleError {
                property: property.clone(),
                stage: KeyframeSampleStage::Interval,
                frame_index: None,
                detail: SampleFailure::Interval,
            });
        };
        let from = compose_endpoint(property, underlying, start, operations)?;
        let to = compose_endpoint(property, underlying, end, operations)?;
        let local = (progress - start.offset) / (end.offset - start.offset);
        if hold && local == 0.0 {
            return Ok(Some(from));
        }
        if hold && local == 1.0 {
            return Ok(Some(to));
        }
        let distance = if hold {
            InterpolationProgress::new(local).map_err(|error| KeyframeSampleError {
                property: property.clone(),
                stage: KeyframeSampleStage::Easing,
                frame_index: start.frame_index,
                detail: SampleFailure::Progress(Box::new(error)),
            })?
        } else {
            let input = EasingInput::new(local).map_err(|error| KeyframeSampleError {
                property: property.clone(),
                stage: KeyframeSampleStage::Easing,
                frame_index: start.frame_index,
                detail: SampleFailure::Easing(Box::new(error)),
            })?;
            let eased = start
                .easing
                .evaluate_unrestricted_with_before_flag(input, before_flag)
                .map_err(|error| KeyframeSampleError {
                    property: property.clone(),
                    stage: KeyframeSampleStage::Easing,
                    frame_index: start.frame_index,
                    detail: SampleFailure::Easing(Box::new(error)),
                })?;
            InterpolationProgress::from_eased(eased).map_err(|error| KeyframeSampleError {
                property: property.clone(),
                stage: KeyframeSampleStage::Easing,
                frame_index: start.frame_index,
                detail: SampleFailure::Progress(Box::new(error)),
            })?
        };
        operations
            .interpolate(property, &from, &to, distance)
            .map(Some)
            .map_err(|error| KeyframeSampleError {
                property: property.clone(),
                stage: KeyframeSampleStage::Interpolation,
                frame_index: None,
                detail: SampleFailure::Operation(Box::new(error)),
            })
    }
}

struct Endpoint<'a> {
    offset: f64,
    value: &'a InterpolableValue,
    easing: &'a Easing,
    composite: CompositeOperation,
    frame_index: Option<usize>,
}

fn compose_endpoint<P: PropertyValueOperations + ?Sized>(
    property: &PropertyKey,
    underlying: &InterpolableValue,
    endpoint: &Endpoint<'_>,
    operations: &P,
) -> Result<InterpolableValue, KeyframeSampleError<P::Error>> {
    compose_value(
        property,
        endpoint.composite,
        underlying,
        endpoint.value,
        operations,
    )
    .map_err(|error| KeyframeSampleError {
        property: property.clone(),
        stage: KeyframeSampleStage::Composition,
        frame_index: endpoint.frame_index,
        detail: SampleFailure::Operation(Box::new(error)),
    })
}
