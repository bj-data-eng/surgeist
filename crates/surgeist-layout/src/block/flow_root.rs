//! The single decision whether a box is a flow root of its parent's block
//! formatting context.
use crate::node_projection::CommonBoxProjection;
use crate::{BlockFlowContext, Display, Float, LayoutScalar, NodeInputOf, Position, WritingMode};

/// A box establishes an independent block formatting context when it is
/// floated or absolutely positioned, has hidden, scroll or auto overflow
/// (CSS 2.1 section 9.4.1), is replaced or a table wrapper, has a non-block
/// inner display, has a writing mode that differs from its parent's (Writing
/// Modes 4 section 3.2) or is declared `BlockFlowContext::Independent` by the
/// host.
///
/// Callers add their own questions, such as the parent's formatting role,
/// inline-level participation or zero-size collapse-through conditions.
pub(crate) fn establishes_flow_root<S: LayoutScalar>(
    display: Display,
    float: Float,
    block_flow_context: BlockFlowContext,
    common: &CommonBoxProjection<'_, S>,
    parent_writing_mode: WritingMode,
) -> bool {
    !float.is_none()
        || common.position == Position::Absolute
        || block_flow_context == BlockFlowContext::Independent
        || common.item_is_replaced
        || common.item_is_table
        || common.overflow.establishes_independent_formatting_context()
        || display.inner_display() != Display::Block
        || common.flow_axes.writing_mode() != parent_writing_mode
}

/// The same decision for a node's authored input.
pub(crate) fn node_establishes_flow_root<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    parent_writing_mode: WritingMode,
) -> bool {
    establishes_flow_root(
        style.display,
        style.float,
        style.block_flow_context,
        &CommonBoxProjection::from_node(style),
        parent_writing_mode,
    )
}
