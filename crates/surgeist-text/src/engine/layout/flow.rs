//! Nonplaceable selected-presentation participation observations.
use super::data::OrdinalMap;
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

pub(crate) fn observe(
    prepared: &PreparedText,
    segment: &PreparedFlowSegment,
    request: &FlowObservationRequest,
    profile: LineMetricPolicy,
    scratch: &mut ShapeContext,
    allowance: Option<crate::ScopeAlignmentAllowance>,
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
    segment.scope_input_roots(&prepared.inner, request)?;
    crate::engine::scope_alignment::operation_limits(&prepared.inner, allowance)?;
    let input = PresentationInput {
        prepared: &prepared.inner,
        presentation: &presentation,
        boundaries: &boundaries,
        start: start.index,
        computed: request_input.incoming.is_some() || request_input.alternative.is_some(),
        metric_policy: profile,
        controls: &request.controls,
    };
    let (shaped, _, dispositions) = super::finalize::shape_and_collapse(
        input,
        scratch,
        allowance,
        crate::engine::scope_alignment::PlanInput::Flow { segment, request },
    )?;
    let mut atomics = OrdinalMap::default();
    for row in &request.atomics {
        atomics.insert(
            prepared.inner.tables.atomics[row.occurrence.index].occurrence,
            row,
        );
    }
    let mut controls = OrdinalMap::default();
    for row in &request.controls {
        controls.insert(
            prepared.inner.tables.controls[row.occurrence.index].occurrence,
            row,
        );
    }
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
