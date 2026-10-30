//! Immutable selected-line finalization and the standalone consuming chooser.

pub(crate) mod data;
mod finalize;
pub(crate) mod flow;
mod placement;
mod standalone;

use std::sync::Arc;

use crate::{Result, TextNumericError, TextNumericPhase, ValidatedOptions};

use super::{PreparedData, shape::ShapeContext, text_numeric_error};
use data::PlacedLayout;

/// Paint capability admission over the actual selected basis, before handles.
pub(crate) fn qualify_selected_paint(
    input: crate::protocol::input::PresentationInput<'_>,
    shaped: &data::ShapedData,
    participants: &[crate::PreparedParticipant],
) -> Result<()> {
    finalize::qualify_selected_paint(input, shaped, participants)
}

/// A removed source fallback retains font metadata without producing decoration.
pub(crate) fn metric_only_decoration_demand(
    run: &data::PreparedRun,
    shaped: &data::ShapedData,
    projection: &crate::projection::SourceProjection,
) -> bool {
    finalize::metric_only_decoration_demand(run, shaped, projection)
}

pub(crate) fn scope_font_contribution(
    metrics: &data::RunMetrics,
    line_height: f64,
) -> Result<crate::ScopeMetricContribution> {
    let (before, after) = finalize::font_strut(data::RunMetrics {
        line_height,
        ..*metrics
    })?;
    Ok(crate::ScopeMetricContribution {
        before,
        after,
        alignment: crate::LineBoxAlignment::Baseline,
    })
}

pub(crate) fn layout(
    prepared: Arc<PreparedData>,
    options: ValidatedOptions,
    scratch: &mut ShapeContext,
) -> Result<PlacedLayout> {
    standalone::layout(prepared, options, scratch)
}

/// Pure allocation geometry over the already retained selected basis.
pub(crate) fn finalize_selected(
    selected: Arc<crate::protocol::selection::SelectedData>,
    amounts: Vec<f64>,
    scopes: super::scope_alignment::FrameAllocations,
) -> Result<super::FinalizedLine> {
    finalize::finalize_scoped(selected, &amounts, scopes)
}

pub(crate) fn refinalize_selected(
    line: &super::FinalizedLine,
    amounts: Vec<f64>,
    scopes: super::scope_alignment::FrameAllocations,
) -> Result<super::FinalizedLine> {
    finalize::finalize_scoped(Arc::clone(&line.selected), &amounts, scopes)
}

fn finalize_standalone(
    prepared: Arc<PreparedData>,
    request: crate::LineSelectionRequest,
    quantize: bool,
    scratch: &mut ShapeContext,
    roles: &mut standalone::RawRoleBasis<'_>,
) -> Result<super::FinalizedLine> {
    use crate::protocol::selection::{LineMetricPolicy, select_with_scratch};
    let profile = if quantize {
        LineMetricPolicy::StandaloneQuantized
    } else {
        LineMetricPolicy::Exact
    };
    let mut request = request;
    request.context.formatted_line = roles.initial(request.context.end);
    loop {
        let selected = select_with_scratch(
            &crate::PreparedText {
                inner: Arc::clone(&prepared),
            },
            request.clone(),
            profile,
            scratch,
        )?
        .inner;
        let result = (|| {
            let first = finalize::finalize(
                Arc::clone(&selected),
                &vec![0.0; selected.spacing_slots.len()],
            )?;
            let amounts = finalize::standalone_amounts(
                &selected,
                &first.data.metrics,
                first
                    .data
                    .measurements
                    .as_ref()
                    .expect("finalized raw measures"),
            )?;
            if amounts.iter().any(|amount| *amount != 0.0) {
                finalize::refinalize(&first, &amounts, None)
            } else {
                Ok(first)
            }
        })();
        match result {
            Err(error) if missing_raw_role(&error, request.context.formatted_line) => {
                // A fresh selection owns the next complete spacing table. A
                // receipt from the failed natural/allocated pass never escapes.
                request.context.formatted_line =
                    roles.resolve(&prepared, &request, profile, scratch)?;
            }
            result => return result,
        }
    }
}

fn missing_raw_role(error: &crate::Error, context: crate::FormattedLineContext) -> bool {
    use crate::{ErrorDetail, PhaseTwoContext, ProtocolError};
    match error.detail() {
        Some(ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::MissingPhaseTwoContext(PhaseTwoContext::FirstFormattedLine),
        }) => context.first().is_none(),
        Some(ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::MissingPhaseTwoContext(PhaseTwoContext::LastFormattedLine),
        }) => context.last().is_none(),
        _ => false,
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
