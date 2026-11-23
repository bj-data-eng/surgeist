//! Per-node input and output aggregates, source and item order, and margin-collapse outputs.
//!
//! [`NodeInputOf`] is the normalized, layout-ready input of one box and [`NodeOutputOf`] its physical output.
//! [`LayoutInputOf`] owns the input of any participant (a box, inline text, a text control, a line break or an
//! inline boundary) and [`LayoutInputViewOf`] borrows it. [`SourceIndex`] is a node's zero-based source-sibling
//! index and [`ItemOrder`] its signed order value; the two are independent. [`CollapsibleMarginOf`] and
//! [`PhysicalBlockMarginCollapseOf`] carry the collapsible block margins of an output.

pub use crate::node_input::{
    ItemOrder, LayoutInput, LayoutInputOf, LayoutInputView, LayoutInputViewOf, NodeInput,
    NodeInputOf,
};
pub use crate::output::{
    CollapsibleMargin, CollapsibleMarginOf, NodeOutput, NodeOutputOf, PhysicalBlockMarginCollapse,
    PhysicalBlockMarginCollapseOf, SourceIndex,
};
