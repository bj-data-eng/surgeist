//! Native paragraph input: origins, connections, controls, anchors and boundary policies.
//!
//! A [`ParagraphInput`] is the complete native normalized input; its fields hold [`ParagraphOrigin`] (with
//! [`ParagraphOriginKind`] and [`CompleteTextNodeContribution`]), [`ParagraphConnection`] (with
//! [`ParagraphConnectionKind`]), [`ParagraphControl`] (with [`ParagraphControlKind`]),
//! [`ParagraphBoundaryPolicy`] (with [`ParagraphBoundaryDefinition`]), [`ParagraphReplacement`] (with
//! [`ParagraphReplacementDefinition`]) and [`ParagraphAnchor`] values. [`ParagraphAnchorItem`] gives the explicit
//! relative order of actual anchor occurrences. [`ParagraphBaseDirection`], [`ParagraphStrut`] and
//! [`ParagraphTerminalEdge`] capture the paragraph's direction, strut and relation of its end to its containing
//! block.

pub use crate::protocol::paragraph::{
    CompleteTextNodeContribution, ParagraphAnchor, ParagraphAnchorItem,
    ParagraphBoundaryDefinition, ParagraphBoundaryPolicy, ParagraphConnection,
    ParagraphConnectionKind, ParagraphControl, ParagraphControlKind, ParagraphOrigin,
    ParagraphOriginKind, ParagraphReplacement, ParagraphReplacementDefinition, ParagraphStrut,
};
pub use crate::protocol::{ParagraphBaseDirection, ParagraphInput, ParagraphTerminalEdge};
