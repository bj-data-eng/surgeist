//! Shared presentation facts. Selection and issued flow retain distinct authorization.
use super::*;

#[derive(Clone, Copy)]
pub(crate) struct RequestInput<'a> {
    pub(crate) segment: &'a PreparedSourceSegment,
    pub(crate) inline_boxes: Option<&'a PreparedInlineBoxFamily>,
    pub(crate) start: &'a PreparedBoundary,
    pub(crate) end: &'a PreparedBoundary,
    pub(crate) participants: &'a [PreparedParticipant],
    pub(crate) alternative: Option<&'a PreparedAlternative>,
    pub(crate) incoming: Option<&'a HyphenationContinuation>,
    pub(crate) context: &'a LineContext,
    pub(crate) atomics: &'a [AtomicAllocation],
    pub(crate) controls: &'a [ControlAllocation],
    pub(crate) boundary_context: &'a [InlineBoundaryContextAllocation],
    pub(crate) line_strut: Option<LineStrut>,
    pub(crate) line_box_struts: &'a [LineBoxStrut],
}
impl<'a> From<&'a LineSelectionRequest> for RequestInput<'a> {
    fn from(value: &'a LineSelectionRequest) -> Self {
        Self {
            segment: &value.segment,
            inline_boxes: value.inline_boxes.as_ref(),
            start: &value.start,
            end: &value.end,
            participants: &value.participants,
            alternative: value.alternative.as_ref(),
            incoming: value.incoming.as_ref(),
            context: &value.context,
            atomics: &value.atomics,
            controls: &value.controls,
            boundary_context: &value.boundary_context,
            line_strut: value.line_strut,
            line_box_struts: &value.line_box_struts,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PresentationInput<'a> {
    pub(crate) prepared: &'a crate::engine::PreparedData,
    pub(crate) presentation: &'a prepared::SelectedPresentation,
    pub(crate) boundaries: &'a boundary::Resolved,
    pub(crate) start: usize,
    pub(crate) computed: bool,
    pub(crate) controls: &'a [ControlAllocation],
    pub(crate) metric_policy: selection::LineMetricPolicy,
}
