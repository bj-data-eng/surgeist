//! Inline-box edges, fragment families, pieces and supplied components.

pub use crate::inline_box_components::{
    InlineBoxComponentParticipation, SuppliedInlineBoxComponent, SuppliedInlineBoxComponentRow,
    SuppliedInlineBoxComponents, SuppliedInlineBoxComponentsInput,
};
pub use crate::protocol::{
    FinalizedInlineBoxPiece, InlineBoxEdgeComponents, InlineBoxEdgeContext, InlineBoxEdgeError,
    InlineBoxFragment, InlineBoxFragmentationError, InlineBoxPieceMember, InlineBoxSource,
    PlacedInlineBoxPiece, PreparedInlineBoxFamily, SelectedInlineBoxPiece,
};
