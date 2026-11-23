//! Per-node input and output aggregates, source indices and margin collapse outputs.

pub use crate::node_input::{
    ItemOrder, LayoutInput, LayoutInputOf, LayoutInputView, LayoutInputViewOf, NodeInput,
    NodeInputOf,
};
pub use crate::output::{
    CollapsibleMargin, CollapsibleMarginOf, NodeOutput, NodeOutputOf, PhysicalBlockMarginCollapse,
    PhysicalBlockMarginCollapseOf, SourceIndex,
};
