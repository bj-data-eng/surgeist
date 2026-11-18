//! Inline-box edges, fragment families, pieces and supplied components.
//!
//! [`InlineBoxSource`] is an actual captured box or a declared native endpoint pair; [`InlineBoxEdgeContext`],
//! [`InlineBoxEdgeComponents`] and [`InlineBoxEdgeError`] describe the edges of a non-atomic box.
//! [`PreparedInlineBoxFamily`] is one complete source-authenticated family, with [`InlineBoxFragment`],
//! [`SelectedInlineBoxPiece`], [`FinalizedInlineBoxPiece`], [`PlacedInlineBoxPiece`] and
//! [`InlineBoxPieceMember`] as its views and [`InlineBoxFragmentationError`] as its refusal.
//! [`SuppliedInlineBoxComponents::try_new`] validates a [`SuppliedInlineBoxComponentsInput`] and derives
//! [`SuppliedInlineBoxComponent`] records and [`SuppliedInlineBoxComponentRow`] associations;
//! [`InlineBoxComponentParticipation`] says whether a row can separate components.

pub use crate::inline_box_components::{
    InlineBoxComponentParticipation, SuppliedInlineBoxComponent, SuppliedInlineBoxComponentRow,
    SuppliedInlineBoxComponents, SuppliedInlineBoxComponentsInput,
};
pub use crate::protocol::{
    FinalizedInlineBoxPiece, InlineBoxEdgeComponents, InlineBoxEdgeContext, InlineBoxEdgeError,
    InlineBoxFragment, InlineBoxFragmentationError, InlineBoxPieceMember, InlineBoxSource,
    PlacedInlineBoxPiece, PreparedInlineBoxFamily, SelectedInlineBoxPiece,
};
