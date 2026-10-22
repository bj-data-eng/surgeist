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
pub(super) fn add(
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
