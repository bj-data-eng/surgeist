//! Explicit fitting authority for one actual in-flow atomic allocation.
use super::{AtomicBoxAllocation, AtomicFlowAllocation};
use crate::{Result, TextNumericError, TextNumericPhase};

/// The operation that determines this atomic's line-fit contribution.
/// Native occupied geometry never acquires CSS margin semantics by inference.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AtomicFitAllocation {
    NativeOccupied,
    CssMarginBox(CssMarginBoxFit),
}

/// Actual used CSS inline margins and border size in the allocation's local
/// line axis. Construction checks their canonical signed margin-box advance;
/// selection couples the receipt to the issued atomic's flow and geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssMarginBoxFit {
    margin_start: f64,
    border_inline_size: f64,
    margin_end: f64,
    advance: f64,
}

impl CssMarginBoxFit {
    pub fn try_new(margin_start: f64, border_inline_size: f64, margin_end: f64) -> Result<Self> {
        let phase = TextNumericPhase::Selection;
        let finite = |field, value: f64| {
            if value.is_finite() {
                Ok(value)
            } else {
                Err(crate::error::text_numeric_error(
                    field,
                    phase,
                    TextNumericError::NonFinite,
                    value,
                ))
            }
        };
        let margin_start = finite("CSS atomic start margin", margin_start)?;
        let margin_end = finite("CSS atomic end margin", margin_end)?;
        let border_inline_size = finite("CSS atomic border inline size", border_inline_size)?;
        if border_inline_size < 0.0 {
            return Err(crate::error::text_numeric_error(
                "CSS atomic border inline size",
                phase,
                TextNumericError::OutOfRange,
                border_inline_size,
            ));
        }
        let border_end = sum("CSS atomic border end", margin_start, border_inline_size)?;
        let advance = sum("CSS atomic margin-box advance", border_end, margin_end)?;
        let zero = |value| if value == 0.0 { 0.0 } else { value };
        Ok(Self {
            margin_start: zero(margin_start),
            border_inline_size: zero(border_inline_size),
            margin_end: zero(margin_end),
            advance: zero(advance),
        })
    }
    pub const fn margin_start(self) -> f64 {
        self.margin_start
    }
    pub const fn border_inline_size(self) -> f64 {
        self.border_inline_size
    }
    pub const fn margin_end(self) -> f64 {
        self.margin_end
    }
    pub const fn advance(self) -> f64 {
        self.advance
    }

    pub fn occupied_inline_bounds(self) -> (f64, f64) {
        (
            0.0_f64.min(self.margin_start).min(self.advance),
            0.0_f64
                .max(self.margin_start + self.border_inline_size)
                .max(self.advance),
        )
    }
    pub(crate) fn matches(self, flow: AtomicFlowAllocation, geometry: AtomicBoxAllocation) -> bool {
        let (start, end) = self.occupied_inline_bounds();
        self.advance == flow.advance
            && self.border_inline_size == geometry.border.size.width
            && start == flow.occupied_start
            && end == flow.occupied_end
    }
}

fn sum(field: &'static str, a: f64, b: f64) -> Result<f64> {
    let value = crate::engine::layout::add(field, TextNumericPhase::Selection, a, b)?;
    if a != 0.0 && value == b {
        return Err(crate::error::text_numeric_error(
            field,
            TextNumericPhase::Selection,
            TextNumericError::NonZeroLost,
            a,
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ErrorDetail, Rect};

    #[test]
    fn used_css_margin_receipt_preserves_signed_flow_and_real_envelope() {
        let css = CssMarginBoxFit::try_new(3.0, 10.0, -5.0).unwrap();
        assert_eq!(css.advance(), 8.0);
        assert_eq!(css.occupied_inline_bounds(), (0.0, 13.0));
        let geometry = AtomicBoxAllocation {
            border: Rect::new(3.0, -8.0, 10.0, 10.0),
            content: Rect::new(3.0, -8.0, 10.0, 10.0),
            baseline: 0.0,
        };
        let flow = AtomicFlowAllocation {
            advance: 8.0,
            before_baseline: 8.0,
            after_baseline: 2.0,
            occupied_start: 0.0,
            occupied_end: 13.0,
        };
        assert!(css.matches(flow, geometry));
        assert!(!css.matches(
            AtomicFlowAllocation {
                advance: 9.0,
                ..flow
            },
            geometry
        ));
        assert!(!css.matches(
            AtomicFlowAllocation {
                occupied_end: 12.0,
                ..flow
            },
            geometry
        ));
        assert!(!css.matches(
            flow,
            AtomicBoxAllocation {
                border: Rect::new(3.0, -8.0, 9.0, 10.0),
                ..geometry
            }
        ));
        let cancelled = CssMarginBoxFit::try_new(-5.0, 5.0, 0.0).unwrap();
        assert_eq!(cancelled.advance(), 0.0);
        assert_eq!(cancelled.occupied_inline_bounds(), (-5.0, 0.0));
        let zero = CssMarginBoxFit::try_new(-0.0, 0.0, -0.0).unwrap();
        assert!(!zero.margin_start().is_sign_negative());
        assert!(!zero.margin_end().is_sign_negative());
        assert!(!zero.advance().is_sign_negative());
    }

    #[test]
    fn css_fit_receipt_refuses_invalid_components_and_lost_terms() {
        for (args, category) in [
            ((f64::NAN, 10.0, 0.0), TextNumericError::NonFinite),
            ((0.0, f64::INFINITY, 0.0), TextNumericError::NonFinite),
            ((0.0, 10.0, f64::NEG_INFINITY), TextNumericError::NonFinite),
            ((0.0, -1.0, 0.0), TextNumericError::OutOfRange),
            ((f64::MAX, f64::MAX, 0.0), TextNumericError::NonFinite),
            (
                (18_014_398_509_481_984.0, 1.0, 0.0),
                TextNumericError::NonZeroLost,
            ),
            ((1e-300, 1.0, 0.0), TextNumericError::NonZeroLost),
            ((0.0, 1.0, 1e-300), TextNumericError::NonZeroLost),
        ] {
            let error = CssMarginBoxFit::try_new(args.0, args.1, args.2).unwrap_err();
            assert!(
                matches!(error.detail(), Some(ErrorDetail::InvalidTextNumeric {
                phase: TextNumericPhase::Selection, category: actual, ..
            }) if *actual == category)
            );
        }
    }
}
