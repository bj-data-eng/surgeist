//! Nonplaceable selected-presentation participation observations.
use crate::engine::shape::ShapeContext;
use crate::protocol::{
    boundary,
    input::PresentationInput,
    protocol_error,
    selection::{self, LineMetricPolicy},
};
use crate::{
    FlowObservation, FlowObservationRequest, PreparedFlowSegment, PreparedText, ProtocolError,
    Result,
};
use std::collections::BTreeMap;

pub(crate) fn observe(
    prepared: &PreparedText,
    segment: &PreparedFlowSegment,
    request: &FlowObservationRequest,
    profile: LineMetricPolicy,
    scratch: &mut ShapeContext,
) -> Result<FlowObservation> {
    let participants: Vec<_> = segment.participants().collect();
    let start = segment.start();
    let end = segment.end();
    let request_input = segment.admit(prepared, request, &participants, &start, &end)?;
    let mut presentation = selection::admit_geometry(prepared, request_input)?;
    let (boundaries, missing) = boundary::resolve(prepared, request_input, &mut presentation)?;
    if let Some(missing) = missing {
        return Err(protocol_error(ProtocolError::MissingBoundaryContext {
            occurrence: missing.index,
        }));
    }
    let input = PresentationInput {
        prepared: &prepared.inner,
        presentation: &presentation,
        boundaries: &boundaries,
        start: start.index,
        computed: request_input.incoming.is_some() || request_input.alternative.is_some(),
        metric_policy: profile,
        controls: &request.controls,
    };
    let (shaped, _, dispositions) = super::finalize::shape_and_collapse(input, scratch)?;
    let atomics: BTreeMap<_, _> = request
        .atomics
        .iter()
        .map(|row| {
            (
                prepared.inner.tables.atomics[row.occurrence.index].occurrence,
                row,
            )
        })
        .collect();
    let controls: BTreeMap<_, _> = request
        .controls
        .iter()
        .map(|row| {
            (
                prepared.inner.tables.controls[row.occurrence.index].occurrence,
                row,
            )
        })
        .collect();
    let retained = shaped.items.iter().any(|item| {
        crate::protocol::prefix::retained_item(input, item, &atomics, &controls, &dispositions)
    });
    let in_flow_content = shaped.items.iter().any(|item| {
        crate::protocol::prefix::in_flow_item(
            input,
            &shaped,
            item,
            &atomics,
            &controls,
            &dispositions,
        )
    });
    Ok(FlowObservation {
        segment: segment.clone(),
        request: request.clone(),
        profile,
        retained,
        in_flow_content,
        identity: crate::protocol::request_context(request_input),
    })
}
