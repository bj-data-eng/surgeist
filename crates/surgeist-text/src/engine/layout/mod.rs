//! Immutable selected-line finalization and the standalone consuming chooser.

pub(crate) mod data;
mod finalize;
mod placement;
mod standalone;

use std::sync::Arc;

use crate::{Result, TextNumericError, TextNumericPhase, ValidatedOptions};

use super::{PreparedData, shape::ShapeContext, text_numeric_error};
use data::PlacedLayout;

pub(crate) fn layout(
    prepared: Arc<PreparedData>,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
) -> Result<PlacedLayout> {
    standalone::layout(prepared, options, scratch)
}

/// One selected operation; scratch is private and cannot select another font basis.
pub(crate) fn finalize_selected(
    selected: Arc<crate::protocol::selection::SelectedData>,
    amounts: Vec<f64>,
) -> Result<super::FinalizedLine> {
    finalize::finalize(selected, &amounts, &mut ShapeContext::default())
}

fn finalize_standalone(
    prepared: Arc<PreparedData>,
    request: crate::LineSelectionRequest,
    quantize: bool,
    scratch: &mut ShapeContext,
) -> Result<super::FinalizedLine> {
    use crate::protocol::selection::{LineMetricPolicy, select};
    let selected = select(
        &crate::PreparedText { inner: prepared },
        request,
        if quantize {
            LineMetricPolicy::StandaloneQuantized
        } else {
            LineMetricPolicy::Exact
        },
    )?
    .inner;
    let first = finalize::finalize(
        Arc::clone(&selected),
        &vec![0.0; selected.spacing_slots.len()],
        scratch,
    )?;
    let amounts = finalize::standalone_amounts(&selected, &first.data.metrics)?;
    if amounts.iter().any(|amount| *amount != 0.0) {
        finalize::finalize(selected, &amounts, scratch)
    } else {
        Ok(first)
    }
}

pub(super) fn finite(field: &'static str, phase: TextNumericPhase, value: f64) -> Result<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(text_numeric_error(
            field,
            phase,
            TextNumericError::NonFinite,
            value,
        ))
    }
}

/// Add an actually required logical contribution at its observation boundary.
/// Signed cancellation is valid; a requested nonzero adjustment cannot vanish.
pub(crate) fn add(
    field: &'static str,
    phase: TextNumericPhase,
    base: f64,
    delta: f64,
) -> Result<f64> {
    let actual = finite(field, phase, base + delta)?;
    if delta != 0.0 && actual == base {
        return Err(text_numeric_error(
            field,
            phase,
            TextNumericError::NonZeroLost,
            delta,
        ));
    }
    Ok(actual)
}
