//! Private consuming bridge for the two sealed Layout scalar lanes.
use crate::{InlineBoundaryEdgeOf, LayoutScalar};
use surgeist_text::inline_box::InlineBoxEdgeComponents;

pub(crate) fn from_native<S: LayoutScalar>(
    edge: InlineBoundaryEdgeOf<S>,
) -> surgeist_text::Result<InlineBoxEdgeComponents> {
    S::inline_box_edge_components(edge.margin(), edge.border(), edge.padding())
}
