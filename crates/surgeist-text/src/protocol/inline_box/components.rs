use crate::{Result, TextNumericError, TextNumericPhase};
use std::hash::{Hash, Hasher};

/// Native-lane used components of an authenticated non-atomic box edge.
/// Components and lane are retained; BP and signed flow are never caller totals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineBoxEdgeComponents {
    data: Components,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Components {
    F32 {
        margin: f32,
        border: f32,
        padding: f32,
    },
    F64 {
        margin: f64,
        border: f64,
        padding: f64,
    },
}
// Construction admits only finite values and canonicalizes every signed zero.
impl Eq for InlineBoxEdgeComponents {}
impl Hash for InlineBoxEdgeComponents {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.data {
            Components::F32 {
                margin,
                border,
                padding,
            } => {
                0u8.hash(state);
                margin.to_bits().hash(state);
                border.to_bits().hash(state);
                padding.to_bits().hash(state);
            }
            Components::F64 {
                margin,
                border,
                padding,
            } => {
                1u8.hash(state);
                margin.to_bits().hash(state);
                border.to_bits().hash(state);
                padding.to_bits().hash(state);
            }
        }
    }
}
macro_rules! construct {
    ($name:ident, $scalar:ty, $lane:ident) => {
        pub fn $name(margin: $scalar, border: $scalar, padding: $scalar) -> Result<Self> {
            for (field, value, nonnegative) in [
                ("inline edge margin", margin, false),
                ("inline edge border", border, true),
                ("inline edge padding", padding, true),
            ] {
                let category = if !value.is_finite() {
                    Some(TextNumericError::NonFinite)
                } else if nonnegative && value < 0.0 {
                    Some(TextNumericError::OutOfRange)
                } else {
                    None
                };
                if let Some(category) = category {
                    return Err(crate::error::text_numeric_error(
                        field,
                        TextNumericPhase::Selection,
                        category,
                        f64::from(value),
                    ));
                }
            }
            let margin = if margin == 0.0 { 0.0 } else { margin };
            let border = if border == 0.0 { 0.0 } else { border };
            let padding = if padding == 0.0 { 0.0 } else { padding };
            let extent = border + padding;
            if !extent.is_finite() {
                return Err(crate::error::text_numeric_error(
                    "inline edge border padding advance",
                    TextNumericPhase::Selection,
                    TextNumericError::NonFinite,
                    f64::from(extent),
                ));
            }
            let flow = margin + extent;
            if !flow.is_finite() {
                return Err(crate::error::text_numeric_error(
                    "inline edge flow advance",
                    TextNumericPhase::Selection,
                    TextNumericError::NonFinite,
                    f64::from(flow),
                ));
            }
            if margin != 0.0 && flow == extent {
                return Err(crate::error::text_numeric_error(
                    "inline edge margin flow",
                    TextNumericPhase::Selection,
                    TextNumericError::NonZeroLost,
                    f64::from(margin),
                ));
            }
            if extent != 0.0 && flow == margin {
                return Err(crate::error::text_numeric_error(
                    "inline edge strip flow",
                    TextNumericPhase::Selection,
                    TextNumericError::NonZeroLost,
                    f64::from(extent),
                ));
            }
            Ok(Self {
                data: Components::$lane {
                    margin,
                    border,
                    padding,
                },
            })
        }
    };
}
impl InlineBoxEdgeComponents {
    construct!(from_f32, f32, F32);
    construct!(from_f64, f64, F64);
    #[must_use]
    pub const fn margin(self) -> f64 {
        match self.data {
            Components::F32 { margin, .. } => margin as f64,
            Components::F64 { margin, .. } => margin,
        }
    }
    #[must_use]
    pub const fn border(self) -> f64 {
        match self.data {
            Components::F32 { border, .. } => border as f64,
            Components::F64 { border, .. } => border,
        }
    }
    #[must_use]
    pub const fn padding(self) -> f64 {
        match self.data {
            Components::F32 { padding, .. } => padding as f64,
            Components::F64 { padding, .. } => padding,
        }
    }
    #[must_use]
    pub const fn border_padding_advance(self) -> f64 {
        match self.data {
            Components::F32 {
                border, padding, ..
            } => (border + padding) as f64,
            Components::F64 {
                border, padding, ..
            } => border + padding,
        }
    }
    #[must_use]
    pub const fn flow_advance(self) -> f64 {
        let flow = match self.data {
            Components::F32 {
                margin,
                border,
                padding,
            } => (margin + (border + padding)) as f64,
            Components::F64 {
                margin,
                border,
                padding,
            } => margin + (border + padding),
        };
        if flow == 0.0 { 0.0 } else { flow }
    }
    pub(crate) fn participates(self) -> bool {
        self.margin() != 0.0 || self.border() != 0.0 || self.padding() != 0.0
    }
    pub(crate) fn blocks_hanging(self) -> bool {
        self.border() != 0.0 || self.padding() != 0.0
    }
    pub(crate) fn evaluate(
        self,
        pen: f64,
        side: crate::LineEdgeSide,
        phase: TextNumericPhase,
    ) -> Result<EdgeExecution> {
        use crate::engine::layout::add;
        let flow_end = add("inline edge flow end", phase, pen, self.flow_advance())?;
        let strip_start = if side == crate::LineEdgeSide::Start {
            add("inline edge margin recipient", phase, pen, self.margin())?
        } else {
            pen
        };
        let strip_end = add(
            "inline edge strip end",
            phase,
            strip_start,
            self.border_padding_advance(),
        )?;
        if self.margin() != 0.0 && side == crate::LineEdgeSide::End && strip_end == flow_end {
            return Err(crate::error::text_numeric_error(
                "inline edge margin recipient",
                phase,
                TextNumericError::NonZeroLost,
                self.margin(),
            ));
        }
        Ok(EdgeExecution {
            flow: (pen, flow_end),
            strip: (strip_start, strip_end),
        })
    }
}
/// The actual evaluated directed flow and nonnegative strip in one line frame.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EdgeExecution {
    pub(crate) flow: (f64, f64),
    pub(crate) strip: (f64, f64),
}
