//! Computed clip-path branches over supplied sources, boxes and shapes.
//!
//! Style supplies resolved source identities and semantic reference boxes. This
//! module neither parses URLs nor fetches resources or evaluates used geometry.
//! Compatible shapes use the shared [`ShapeValue`] interpolation kernel. Other
//! branch pairs switch their complete value at the discrete midpoint.
//!
//! Masking's discrete properties and legacy `clip` rectangles use the existing
//! discrete and computed-component families. Property policy remains caller-owned;
//! in particular, numeric mask-border operands still animate discretely.
//!
//! ```
//! use surgeist_animation::masking::{ClipPathValue, ClipSource};
//! use surgeist_animation::interpolation::InterpolationProgress;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let source = ClipPathValue::Source(ClipSource::new("https://example.test/a.svg#clip")?);
//! let result = ClipPathValue::None.interpolate(&source, InterpolationProgress::new(0.5)?)?;
//! assert_eq!(result, source);
//! # Ok(())
//! # }
//! ```

use core::fmt;
use std::sync::Arc;

use crate::interpolation::InterpolationProgress;
use crate::shape::{ReferenceBox, ShapeError, ShapeValue};

/// An opaque caller-resolved clipping source identity with intrinsic text guards.
///
/// Identity is exact and case-sensitive. Nonblank text without control characters
/// is admitted without URL parsing, absolutization, resource access or security
/// policy. The supplying owner establishes those contextual facts. Whitespace is
/// retained, never trimmed or decoded.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClipSource(Arc<str>);

impl ClipSource {
    /// Guards a supplied identity without interpreting authored URL syntax.
    pub fn new(value: impl Into<String>) -> Result<Self, ClipSourceError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ClipSourceError {
                kind: ClipSourceErrorKind::Blank,
            });
        }
        if let Some((byte_index, _)) = value.char_indices().find(|(_, ch)| ch.is_control()) {
            return Err(ClipSourceError {
                kind: ClipSourceErrorKind::ControlCharacter { byte_index },
            });
        }
        Ok(Self(Arc::from(value)))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The intrinsic reason a supplied clipping source identity was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipSourceErrorKind {
    Blank,
    ControlCharacter { byte_index: usize },
}

/// A rejected opaque source identity, without a partially constructed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipSourceError {
    kind: ClipSourceErrorKind,
}
impl ClipSourceError {
    pub const fn kind(&self) -> ClipSourceErrorKind {
        self.kind
    }
}
impl fmt::Display for ClipSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid clipping source identity: {:?}", self.kind)
    }
}
impl std::error::Error for ClipSourceError {}

/// A complete computed clip-path branch with no inferred document geometry.
///
/// The supplier lowers an omitted shape box to its semantic default. Used SVG
/// box mapping and box-only corner geometry remain with their consuming owners.
#[derive(Debug, Clone, PartialEq)]
pub enum ClipPathValue {
    None,
    Source(ClipSource),
    Box(ReferenceBox),
    Shape(ShapeValue),
}
impl ClipPathValue {
    /// Samples shapes through their owner, otherwise switching complete branches.
    ///
    /// Finite extrapolated geometry is retained. Shape compatibility includes
    /// reference boxes, command structure and metadata; mixed Shape/Path results
    /// retain the shape owner's output kind even at exact endpoints.
    ///
    /// # Errors
    /// Returns the original typed shape arithmetic error without partial output.
    pub fn interpolate(
        &self,
        other: &Self,
        progress: InterpolationProgress,
    ) -> Result<Self, ShapeError> {
        if let (Self::Shape(from), Self::Shape(to)) = (self, other) {
            return from.interpolate(to, progress).map(Self::Shape);
        }
        Ok(if progress.value() < 0.5 { self } else { other }.clone())
    }
}
