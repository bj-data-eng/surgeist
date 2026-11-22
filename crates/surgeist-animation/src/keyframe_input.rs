//! Typed native keyframe input normalization and computed offsets.
//!
//! Callers supply canonical property identities, parsed value payloads, and
//! easing validation results. `V` may retain caller-owned source provenance;
//! `E` preserves the caller's easing failure. No CSS parsing or serialization
//! occurs here. A missing value (`None`) records a rejected property value and
//! is discarded without removing its frame. A missing composite is `auto` and
//! inherits the effect composite when sampled.
//!
//! Normalization consumes a candidate and returns a complete immutable result
//! or an error. It does not mutate an existing effect. Computed offsets do not
//! imply that property values have been resolved in a target's live context.
//!
//! ```
//! use surgeist_animation::interpolation::PropertyKey;
//! use surgeist_animation::keyframe_input::{KeyframeInput, KeyframesInput};
//!
//! // The caller has already accepted this property value.
//! let input = KeyframesInput::Sequence(vec![KeyframeInput::<f64> {
//!     properties: vec![(PropertyKey::new("--distance").unwrap(), Some(12.0))],
//!     ..KeyframeInput::default()
//! }]);
//! let normalized = input.normalize().unwrap();
//! let frame = &normalized.frames()[0];
//! assert_eq!(frame.specified_offset(), None);
//! assert_eq!(frame.computed_offset().value(), 1.0);
//! assert_eq!(frame.properties()[0].1, 12.0);
//! ```

use core::convert::Infallible;
use core::fmt;
use std::collections::HashSet;

use crate::diagnostics::NumericError;
use crate::easing::Easing;
use crate::interpolation::PropertyKey;
use crate::progress::UnitRatio;

/// Specifies how an effect or keyframe combines with its underlying value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeOperation {
    Replace,
    Add,
    Accumulate,
}

/// Supplies absent, sequential, or property-indexed native keyframes.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyframesInput<V, E = Infallible> {
    Absent,
    Sequence(Vec<KeyframeInput<V, E>>),
    PropertyIndexed(PropertyIndexedInput<V, E>),
}

/// Holds one raw frame before offset and easing validation.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyframeInput<V, E = Infallible> {
    /// The specified offset; `None` means a missing offset.
    pub offset: Option<f64>,
    /// The caller's parsed easing or its validation failure.
    pub easing: Result<Easing, E>,
    /// A keyframe override; `None` is auto, inheriting the effect composite.
    pub composite: Option<CompositeOperation>,
    /// Unique canonical property keys and accepted or rejected value payloads.
    pub properties: Vec<(PropertyKey, Option<V>)>,
}

/// Holds property tracks and their positional metadata lists.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyIndexedInput<V, E = Infallible> {
    /// Unique canonical property keys and their values, including rejected slots.
    pub properties: Vec<(PropertyKey, Vec<Option<V>>)>,
    /// Assigns specified offsets positionally after intermediate track merging.
    /// Surplus offsets are ignored; short lists do not repeat.
    pub offsets: Vec<Option<f64>>,
    /// Repeats short lists; empty means linear. Every entry is validated,
    /// including entries beyond the number of merged frames.
    pub easings: Vec<Result<Easing, E>>,
    /// Repeats short lists; empty and `None` entries mean auto.
    pub composites: Vec<Option<CompositeOperation>>,
}

/// Contains fully normalized frames in their accepted order.
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedKeyframes<V> {
    frames: Vec<NormalizedKeyframe<V>>,
}

/// Preserves one frame's specified and computed offset independently.
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedKeyframe<V> {
    specified_offset: Option<UnitRatio>,
    computed_offset: UnitRatio,
    easing: Easing,
    composite: Option<CompositeOperation>,
    properties: Vec<(PropertyKey, V)>,
}

/// Identifies the rejected input category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyframeInputErrorKind {
    OffsetOrder,
    InvalidOffset,
    InvalidEasing,
    DuplicateProperty,
}

/// Identifies the zero-based source position of a normalization failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyframeInputLocation {
    Frame(usize),
    Property(usize),
    Easing(usize),
}

/// Reports a rejected candidate without publishing a partial normalized result.
#[derive(Debug, PartialEq)]
pub struct KeyframeInputError<E = Infallible> {
    location: KeyframeInputLocation,
    detail: ErrorDetail<E>,
}

#[derive(Debug, PartialEq)]
enum ErrorDetail<E> {
    OffsetOrder { previous: f64, actual: f64 },
    InvalidOffset { value: f64, cause: NumericError },
    InvalidEasing(E),
    DuplicateProperty(PropertyKey),
}

impl<V, E> Default for KeyframeInput<V, E> {
    fn default() -> Self {
        Self {
            offset: None,
            easing: Ok(Easing::linear()),
            composite: None,
            properties: Vec::new(),
        }
    }
}

impl<V, E> Default for PropertyIndexedInput<V, E> {
    fn default() -> Self {
        Self {
            properties: Vec::new(),
            offsets: Vec::new(),
            easings: Vec::new(),
            composites: Vec::new(),
        }
    }
}

impl<V, E> KeyframesInput<V, E> {
    /// Validates a candidate and computes its missing keyframe offsets.
    ///
    /// Sequence frames at equal offsets remain separate and ordered. For
    /// property-indexed input, intermediate computed offsets determine merging;
    /// specified offsets are then assigned positionally and computed afresh.
    ///
    /// # Errors
    ///
    /// Rejects duplicate dictionary keys, descending explicit offsets,
    /// non-finite or out-of-range offsets, and caller-rejected easing. Offset
    /// ordering is checked across the complete candidate before offset range,
    /// and both precede easing validation. Surplus property-indexed easing
    /// entries are validated after entries used by frames.
    pub fn normalize(self) -> Result<NormalizedKeyframes<V>, KeyframeInputError<E>> {
        match self {
            Self::Absent => Ok(NormalizedKeyframes { frames: Vec::new() }),
            Self::Sequence(frames) => {
                for (index, frame) in frames.iter().enumerate() {
                    validate_properties(frame.properties.iter().map(|(key, _)| key), |_| {
                        KeyframeInputLocation::Frame(index)
                    })?;
                }
                let specified = validate_offsets(&frames)?;
                normalize_frames(frames, specified)
            }
            Self::PropertyIndexed(input) => normalize_property_indexed(input),
        }
    }
}

impl<V> NormalizedKeyframes<V> {
    /// Returns the complete ordered frame set.
    pub fn frames(&self) -> &[NormalizedKeyframe<V>] {
        &self.frames
    }
}

impl<V> NormalizedKeyframe<V> {
    /// Returns the specified offset, preserving a missing input offset.
    pub const fn specified_offset(&self) -> Option<UnitRatio> {
        self.specified_offset
    }

    /// Returns the computed offset in the closed unit interval.
    pub const fn computed_offset(&self) -> UnitRatio {
        self.computed_offset
    }

    /// Returns the validated outgoing interval easing.
    pub const fn easing(&self) -> &Easing {
        &self.easing
    }

    /// Returns the override, or `None` for auto.
    pub const fn composite(&self) -> Option<CompositeOperation> {
        self.composite
    }

    /// Resolves auto against the caller's current effect composite.
    pub fn effective_composite(&self, effect: CompositeOperation) -> CompositeOperation {
        self.composite.unwrap_or(effect)
    }

    /// Returns accepted property values with caller payloads unchanged.
    pub fn properties(&self) -> &[(PropertyKey, V)] {
        &self.properties
    }
}

impl<E> KeyframeInputError<E> {
    /// Returns the failure category.
    pub const fn kind(&self) -> KeyframeInputErrorKind {
        match &self.detail {
            ErrorDetail::OffsetOrder { .. } => KeyframeInputErrorKind::OffsetOrder,
            ErrorDetail::InvalidOffset { .. } => KeyframeInputErrorKind::InvalidOffset,
            ErrorDetail::InvalidEasing(_) => KeyframeInputErrorKind::InvalidEasing,
            ErrorDetail::DuplicateProperty(_) => KeyframeInputErrorKind::DuplicateProperty,
        }
    }

    /// Returns the source position; property-indexed easing uses its list index.
    pub const fn location(&self) -> KeyframeInputLocation {
        self.location
    }

    /// Returns the rejected offset for offset failures.
    pub const fn offset(&self) -> Option<f64> {
        match self.detail {
            ErrorDetail::OffsetOrder { actual, .. } => Some(actual),
            ErrorDetail::InvalidOffset { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Returns the prior explicit offset for an ordering failure.
    pub const fn previous_offset(&self) -> Option<f64> {
        match self.detail {
            ErrorDetail::OffsetOrder { previous, .. } => Some(previous),
            _ => None,
        }
    }

    /// Returns the caller's easing validation failure unchanged.
    pub const fn easing_error(&self) -> Option<&E> {
        match &self.detail {
            ErrorDetail::InvalidEasing(error) => Some(error),
            _ => None,
        }
    }

    /// Returns the duplicated property identity for a dictionary-shape failure.
    pub const fn property(&self) -> Option<&PropertyKey> {
        match &self.detail {
            ErrorDetail::DuplicateProperty(property) => Some(property),
            _ => None,
        }
    }
}

impl<E: fmt::Display> fmt::Display for KeyframeInputError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "keyframe input {:?}: ", self.location)?;
        match &self.detail {
            ErrorDetail::OffsetOrder { previous, actual } => {
                write!(f, "offset {actual} precedes explicit offset {previous}")
            }
            ErrorDetail::InvalidOffset { value, .. } => write!(f, "invalid offset {value}"),
            ErrorDetail::InvalidEasing(error) => write!(f, "invalid easing: {error}"),
            ErrorDetail::DuplicateProperty(property) => {
                write!(f, "duplicate property {}", property.as_str())
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for KeyframeInputError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.detail {
            ErrorDetail::InvalidOffset { cause, .. } => Some(cause),
            ErrorDetail::InvalidEasing(error) => Some(error),
            _ => None,
        }
    }
}

fn validate_properties<'a, E>(
    properties: impl Iterator<Item = &'a PropertyKey>,
    location: impl Fn(usize) -> KeyframeInputLocation,
) -> Result<(), KeyframeInputError<E>> {
    let mut seen = HashSet::new();
    for (index, property) in properties.enumerate() {
        if !seen.insert(property) {
            return Err(KeyframeInputError {
                location: location(index),
                detail: ErrorDetail::DuplicateProperty(property.clone()),
            });
        }
    }
    Ok(())
}

fn validate_offsets<V, E>(
    frames: &[KeyframeInput<V, E>],
) -> Result<Vec<Option<UnitRatio>>, KeyframeInputError<E>> {
    let mut previous = None;
    for (index, frame) in frames.iter().enumerate() {
        if let Some(actual) = frame.offset {
            if let Some(previous) = previous
                && actual < previous
            {
                return Err(KeyframeInputError {
                    location: KeyframeInputLocation::Frame(index),
                    detail: ErrorDetail::OffsetOrder { previous, actual },
                });
            }
            previous = Some(actual);
        }
    }
    frames
        .iter()
        .enumerate()
        .map(|(index, frame)| frame.offset.map(|value| ratio(value, index)).transpose())
        .collect()
}

fn ratio<E>(value: f64, index: usize) -> Result<UnitRatio, KeyframeInputError<E>> {
    UnitRatio::new(value).map_err(|cause| KeyframeInputError {
        location: KeyframeInputLocation::Frame(index),
        detail: ErrorDetail::InvalidOffset { value, cause },
    })
}

fn compute_offsets<E>(
    specified: &[Option<UnitRatio>],
) -> Result<Vec<UnitRatio>, KeyframeInputError<E>> {
    let mut anchors = specified.to_vec();
    if anchors.len() > 1
        && let Some(first) = anchors.first_mut()
        && first.is_none()
    {
        *first = Some(ratio(0.0, 0)?);
    }
    let last_index = anchors.len().saturating_sub(1);
    if let Some(last) = anchors.last_mut()
        && last.is_none()
    {
        *last = Some(ratio(1.0, last_index)?);
    }

    let mut computed = Vec::with_capacity(anchors.len());
    let mut previous: Option<(usize, UnitRatio)> = None;
    for (end_index, end) in anchors.into_iter().enumerate() {
        let Some(end) = end else { continue };
        if let Some((start_index, start)) = previous {
            for index in start_index + 1..=end_index {
                if index == end_index {
                    computed.push(end);
                    continue;
                }
                let fraction = (index - start_index) as f64 / (end_index - start_index) as f64;
                let value = start.value() + (end.value() - start.value()) * fraction;
                computed.push(ratio(value, index)?);
            }
        } else {
            computed.push(end);
        }
        previous = Some((end_index, end));
    }
    Ok(computed)
}

fn normalize_frames<V, E>(
    frames: Vec<KeyframeInput<V, E>>,
    specified: Vec<Option<UnitRatio>>,
) -> Result<NormalizedKeyframes<V>, KeyframeInputError<E>> {
    let computed = compute_offsets(&specified)?;
    let frames = frames
        .into_iter()
        .zip(specified)
        .zip(computed)
        .enumerate()
        .map(|(index, ((frame, specified_offset), computed_offset))| {
            let easing = frame.easing.map_err(|error| KeyframeInputError {
                location: KeyframeInputLocation::Frame(index),
                detail: ErrorDetail::InvalidEasing(error),
            })?;
            Ok(NormalizedKeyframe {
                specified_offset,
                computed_offset,
                easing,
                composite: frame.composite,
                properties: frame
                    .properties
                    .into_iter()
                    .filter_map(|(property, value)| value.map(|value| (property, value)))
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, KeyframeInputError<E>>>()?;
    Ok(NormalizedKeyframes { frames })
}

fn normalize_property_indexed<V, E>(
    input: PropertyIndexedInput<V, E>,
) -> Result<NormalizedKeyframes<V>, KeyframeInputError<E>> {
    validate_properties(
        input.properties.iter().map(|(key, _)| key),
        KeyframeInputLocation::Property,
    )?;
    let mut intermediate = Vec::new();
    for (property, values) in input.properties {
        let count = values.len();
        for (index, value) in values.into_iter().enumerate() {
            let offset = if count == 1 {
                1.0
            } else {
                index as f64 / (count - 1) as f64
            };
            intermediate.push((ratio(offset, index)?, property.clone(), value));
        }
    }
    intermediate.sort_by(|a, b| a.0.value().total_cmp(&b.0.value()));
    let mut merged: Vec<(UnitRatio, KeyframeInput<V, E>)> = Vec::new();
    for (offset, property, value) in intermediate {
        if let Some((previous, frame)) = merged.last_mut()
            && previous.value() == offset.value()
        {
            frame.properties.push((property, value));
        } else {
            merged.push((
                offset,
                KeyframeInput {
                    properties: vec![(property, value)],
                    ..KeyframeInput::default()
                },
            ));
        }
    }
    let mut frames: Vec<_> = merged.into_iter().map(|(_, frame)| frame).collect();
    for (frame, offset) in frames.iter_mut().zip(input.offsets) {
        frame.offset = offset;
    }
    let specified = validate_offsets(&frames)?;
    let easings = input
        .easings
        .into_iter()
        .enumerate()
        .map(|(index, easing)| {
            easing.map_err(|error| KeyframeInputError {
                location: KeyframeInputLocation::Easing(index),
                detail: ErrorDetail::InvalidEasing(error),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (index, frame) in frames.iter_mut().enumerate() {
        if !easings.is_empty() {
            frame.easing = Ok(easings[index % easings.len()].clone());
        }
        if !input.composites.is_empty() {
            frame.composite = input.composites[index % input.composites.len()];
        }
    }
    normalize_frames(frames, specified)
}
