//! Invocation-local nominal vertical metrics and half-design-unit positioning.

use super::buffer::{hb_buffer_t, GlyphBuffer};
use crate::{
    DefaultIgnorableReceipt, Direction, Feature, GlyphInfo, ShapePlan, Shaper, ShapingRefusal,
    ShapingStatus, UnicodeBuffer,
};

/// Unscaled nominal vertical facts for one actual substituted glyph.
///
/// Advance is a nonnegative magnitude, origin-y is positive up, and origin-x
/// retains its exact doubled numerator. These are font-design units, not pixels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerticalNominalMetrics {
    advance: u32,
    origin_x_twice: i32,
    origin_y: i32,
}
impl VerticalNominalMetrics {
    /// Constructs a scalar carrier. The actual query checks representability in
    /// the positioning domain; unrequested glyphs are never pre-scanned.
    pub const fn new(advance: u32, origin_x_twice: i32, origin_y: i32) -> Self {
        Self {
            advance,
            origin_x_twice,
            origin_y,
        }
    }
    /// Nonnegative nominal vertical advance magnitude in design units.
    pub const fn advance(self) -> u32 {
        self.advance
    }
    /// Twice the signed nominal horizontal origin in design units.
    pub const fn origin_x_twice(self) -> i32 {
        self.origin_x_twice
    }
    /// Signed nominal vertical origin in design units, positive up.
    pub const fn origin_y(self) -> i32 {
        self.origin_y
    }
}

/// Actual positioning owner that could not represent a coordinate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalPositionStage {
    /// Encoding actual supplied nominal metrics.
    Nominal,
    /// OpenType values, anchors, or mark-width adjustment.
    Gpos,
    /// Propagating actual attachment offsets and advances.
    Attachment,
    /// Fallback space or mark geometry.
    Fallback,
}
/// A context not supported by the metric-supplied operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerticalShapeContextError {
    /// The actual input direction is not TopToBottom.
    Direction,
    /// The plan direction differs from the input.
    PlanDirection,
    /// The plan script differs from the input.
    PlanScript,
    /// The selected plan uses native morx substitution.
    Morx,
    /// The selected plan uses native kerx positioning.
    Kerx,
    /// The selected plan uses native kern positioning.
    Kern,
    /// The selected plan uses native tracking.
    Trak,
}
/// A failed metric-supplied shaping operation; no accepted output is returned.
#[derive(Debug, PartialEq, Eq)]
pub enum VerticalShapeError<E> {
    /// Invalid or unsupported actual selected context.
    Context(VerticalShapeContextError),
    /// Original supplier failure for the actual requested glyph.
    Supplier {
        /// Actual queried glyph in the configured face.
        glyph_id: u32,
        /// Original error returned by the supplier.
        error: E,
    },
    /// An actual positioning owner exceeded the half-unit coordinate range.
    CoordinateRange {
        /// Actual owner of the first coordinate-range failure.
        stage: VerticalPositionStage,
    },
    /// The first genuine resource or custody refusal in this buffer lifecycle.
    Refused(ShapingRefusal),
}

impl core::fmt::Display for VerticalPositionStage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Nominal => "nominal metric encoding",
            Self::Gpos => "OpenType positioning",
            Self::Attachment => "attachment propagation",
            Self::Fallback => "fallback positioning",
        })
    }
}
impl core::fmt::Display for VerticalShapeContextError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Direction => "metric-supplied shaping requires TopToBottom input",
            Self::PlanDirection => "shape plan direction differs from the input",
            Self::PlanScript => "shape plan script differs from the input",
            Self::Morx => "selected morx substitution is unsupported in the half-unit domain",
            Self::Kerx => "selected kerx positioning is unsupported in the half-unit domain",
            Self::Kern => "selected native kern positioning is unsupported in the half-unit domain",
            Self::Trak => "selected native tracking is unsupported in the half-unit domain",
        })
    }
}
impl core::error::Error for VerticalShapeContextError {}
impl<E: core::fmt::Display> core::fmt::Display for VerticalShapeError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Context(context) => core::fmt::Display::fmt(context, f),
            Self::Supplier { glyph_id, error } => write!(
                f,
                "vertical metric supplier failed for glyph {glyph_id}: {error}"
            ),
            Self::CoordinateRange { stage } => {
                write!(f, "half-unit coordinate range exceeded during {stage}")
            }
            Self::Refused(reason) => write!(f, "shaping refused: {reason:?}"),
        }
    }
}
impl<E: core::error::Error + 'static> core::error::Error for VerticalShapeError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Context(context) => Some(context),
            Self::Supplier { error, .. } => Some(error),
            Self::CoordinateRange { .. } | Self::Refused(_) => None,
        }
    }
}

/// Decoded unscaled positions in font-design units, positive y up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VerticalGlyphPosition {
    /// Horizontal advance.
    pub x_advance: f64,
    /// Vertical advance; downward is negative.
    pub y_advance: f64,
    /// Horizontal offset from the current point.
    pub x_offset: f64,
    /// Vertical offset from the current point.
    pub y_offset: f64,
}
/// Completed metric-supplied output. Encoded counts cannot escape as an ordinary
/// GlyphBuffer; all coordinate views decode exactly to design units.
pub struct VerticalGlyphBuffer(GlyphBuffer);
impl VerticalGlyphBuffer {
    /// Number of actual final glyphs.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Whether the final output has no glyphs.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Actual final glyph and cluster identities.
    pub fn glyph_infos(&self) -> &[GlyphInfo] {
        self.0.glyph_infos()
    }
    /// Exact half-unit positions decoded to unscaled design units.
    pub fn glyph_positions(
        &self,
    ) -> impl ExactSizeIterator<Item = VerticalGlyphPosition> + DoubleEndedIterator + '_ {
        self.0
            .glyph_positions()
            .iter()
            .map(|p| VerticalGlyphPosition {
                x_advance: f64::from(p.x_advance) * 0.5,
                y_advance: f64::from(p.y_advance) * 0.5,
                x_offset: f64::from(p.x_offset) * 0.5,
                y_offset: f64::from(p.y_offset) * 0.5,
            })
    }
    /// Completion of the actual retained buffer lifecycle.
    pub fn shaping_status(&self) -> ShapingStatus {
        self.0.shaping_status()
    }
    /// Actual default-ignorable custody, using original input scalar ordinals.
    ///
    /// # Errors
    /// Returns the first retained lifecycle refusal rather than admitting a
    /// receipt for refused partial output. Current construction of this result
    /// admits only completed output; the delegated receipt preserves that owner.
    pub fn default_ignorable_receipt(&self) -> Result<DefaultIgnorableReceipt<'_>, ShapingRefusal> {
        self.0.default_ignorable_receipt()
    }
    /// Disposes the coordinate domain and returns a clean ordinary input buffer.
    pub fn clear(self) -> UnicodeBuffer {
        self.0.clear()
    }
}
impl core::fmt::Debug for VerticalGlyphBuffer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VerticalGlyphBuffer")
            .field("glyph_infos", &self.glyph_infos())
            .field("shaping_status", &self.shaping_status())
            .finish()
    }
}

impl Shaper<'_> {
    /// Shapes TopToBottom with caller-supplied nominal metrics for actual needed
    /// post-substitution glyphs, before positioning and attachment propagation.
    ///
    /// The supplier must describe this configured face and instance. It is
    /// invocation-local and receives no unrelated glyph queries. The result
    /// retains exact half-design-unit origins. Selected AAT paths are unsupported;
    /// dormant tables do not refuse an OpenType-selected plan. Ordinary shaping
    /// and its design-unit results are unchanged.
    ///
    /// # Errors
    /// Returns [`VerticalShapeError::Context`] for mismatched direction/script
    /// or a selected unsupported native plan path. An actual supplier failure
    /// preserves its original error and queried glyph in
    /// [`VerticalShapeError::Supplier`]. Unrepresentable coordinates return
    /// [`VerticalShapeError::CoordinateRange`] with the actual positioning stage.
    /// A prior or during-shaping resource/custody refusal returns
    /// [`VerticalShapeError::Refused`] with its first retained reason; a prior
    /// refusal prevents supplier invocation. No error returns accepted output.
    pub fn shape_vertical_with_metrics<E>(
        &self,
        plan: &ShapePlan,
        mut buffer: UnicodeBuffer,
        features: &[Feature],
        mut supplier: impl FnMut(u32) -> Result<VerticalNominalMetrics, E>,
    ) -> Result<VerticalGlyphBuffer, VerticalShapeError<E>> {
        if let ShapingStatus::Refused(reason) = buffer.0.shaping_status() {
            return Err(VerticalShapeError::Refused(reason));
        }
        let context = if buffer.0.direction != Direction::TopToBottom {
            Some(VerticalShapeContextError::Direction)
        } else if plan.direction != buffer.0.direction {
            Some(VerticalShapeContextError::PlanDirection)
        } else if plan.script.unwrap_or(crate::script::UNKNOWN)
            != buffer.0.script.unwrap_or(crate::script::UNKNOWN)
        {
            Some(VerticalShapeContextError::PlanScript)
        } else if plan.apply_morx {
            Some(VerticalShapeContextError::Morx)
        } else if plan.apply_kerx {
            Some(VerticalShapeContextError::Kerx)
        } else if plan.apply_kern {
            Some(VerticalShapeContextError::Kern)
        } else if plan.apply_trak {
            Some(VerticalShapeContextError::Trak)
        } else {
            None
        };
        if let Some(context) = context {
            return Err(VerticalShapeError::Context(context));
        }
        buffer.0.position_math = PositionMath::half();
        let mut failure = None;
        let result = {
            let mut query = |glyph| match supplier(glyph) {
                Ok(metrics) => Some(metrics),
                Err(error) => {
                    failure = Some((glyph, error));
                    None
                }
            };
            let mut source = MetricSource {
                query: &mut query,
                failed: false,
            };
            self.shape_with_plan_inner(plan, buffer, features, Some(&mut source))
        };
        if let ShapingStatus::Refused(reason) = result.shaping_status() {
            return Err(VerticalShapeError::Refused(reason));
        }
        if let Some((glyph_id, error)) = failure {
            return Err(VerticalShapeError::Supplier { glyph_id, error });
        }
        if let Some(stage) = result.0.position_math.failure {
            return Err(VerticalShapeError::CoordinateRange { stage });
        }
        Ok(VerticalGlyphBuffer(result))
    }
}

pub(crate) struct MetricSource<'a> {
    pub query: &'a mut dyn FnMut(u32) -> Option<VerticalNominalMetrics>,
    pub failed: bool,
}
impl MetricSource<'_> {
    pub fn get(&mut self, buffer: &mut hb_buffer_t, glyph: u32) -> Option<(i32, i32, i32)> {
        if buffer.shaping_status() != ShapingStatus::Complete
            || self.failed
            || buffer.position_math.failure.is_some()
        {
            return None;
        }
        let Some(metrics) = (self.query)(glyph) else {
            self.failed = true;
            return None;
        };
        let math = &mut buffer.position_math;
        let previous = math.stage;
        math.stage = VerticalPositionStage::Nominal;
        let advance = math.coordinate(-i64::from(metrics.advance) * 2);
        let x = math.coordinate(-i64::from(metrics.origin_x_twice));
        let y = math.coordinate(-i64::from(metrics.origin_y) * 2);
        math.stage = previous;
        math.failure.is_none().then_some((advance, x, y))
    }
}

/// One small concrete arithmetic owner; ordinary calls retain ordinary integer
/// semantics. Raw font/heuristic quantities are encoded once, never re-scaled.
#[derive(Clone, Copy)]
pub(crate) struct PositionMath {
    half: bool,
    pub failure: Option<VerticalPositionStage>,
    pub stage: VerticalPositionStage,
}
impl PositionMath {
    pub const fn ordinary() -> Self {
        Self {
            half: false,
            failure: None,
            stage: VerticalPositionStage::Nominal,
        }
    }
    pub const fn half() -> Self {
        Self {
            half: true,
            ..Self::ordinary()
        }
    }
    pub fn is_half(self) -> bool {
        self.half
    }
    pub fn coordinate(&mut self, value: i64) -> i32 {
        if let Ok(value) = i32::try_from(value) {
            value
        } else {
            self.failure.get_or_insert(self.stage);
            0
        }
    }
    pub fn encode(&mut self, value: i32) -> i32 {
        if self.half {
            self.coordinate(i64::from(value) * 2)
        } else {
            value
        }
    }
    pub fn add(&mut self, a: i32, b: i32) -> i32 {
        if self.half {
            self.coordinate(i64::from(a) + i64::from(b))
        } else {
            a + b
        }
    }
    pub fn sub(&mut self, a: i32, b: i32) -> i32 {
        if self.half {
            self.coordinate(i64::from(a) - i64::from(b))
        } else {
            a - b
        }
    }
    pub fn neg(&mut self, a: i32) -> i32 {
        if self.half {
            self.coordinate(-i64::from(a))
        } else {
            -a
        }
    }
    pub fn mul(&mut self, a: i32, b: i32) -> i32 {
        if self.half {
            self.coordinate(i64::from(a) * i64::from(b))
        } else {
            a * b
        }
    }
    /// Decode only a raw-design heuristic operand. A fractional propagated
    /// position is not a raw font integer and must never be truncated here.
    pub fn raw(&mut self, value: i32) -> i32 {
        if self.half {
            if value % 2 != 0 {
                self.failure.get_or_insert(self.stage);
            }
            value / 2
        } else {
            value
        }
    }
    pub fn divide_raw(&mut self, value: i32, divisor: i32) -> i32 {
        let raw = self.raw(value);
        self.encode(raw / divisor)
    }
}
