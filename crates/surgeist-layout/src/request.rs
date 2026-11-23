//! The root request of a layout run and the contexts that describe where the root sits.
//!
//! [`LayoutRootRequestOf`] carries a sizing request, the explicit [`LayoutRootContextOf`] of the root and a
//! [`LayoutRoundingMode`]; it is built as a standalone, flex-item or bounded block-flow or positioned root, and
//! [`RootContextMismatch`] reports a declared context that disagrees with the actual root input. The contexts
//! describe the box's containing boundary ([`ContainingLayoutContext`]) and the formatting algorithm of its
//! generated parent ([`ParentFormattingContext`]); a flex-item root declares its actual flex-parent authority
//! ([`FlexItemRootContextOf`]), and the bounded block-flow and absolutely positioned roots have their own contexts
//! ([`BlockFlowRootContextOf`], [`PositionedRootContextOf`], [`RootLocalPositionedContextOf`]).
//! [`LayoutSizeInputErrorOf`] names the [`LayoutSizeInput`] that a direct compute constructor, such as
//! [`ComputeInputOf::leaf_layout`](crate::cache::ComputeInputOf::leaf_layout), rejected.

pub use crate::output::{
    BlockFlowRootContext, BlockFlowRootContextOf, ContainingLayoutContext, FlexItemRootContext,
    FlexItemRootContextOf, LayoutRootContext, LayoutRootContextOf, LayoutRootRequest,
    LayoutRootRequestOf, LayoutRoundingMode, LayoutSizeInput, LayoutSizeInputError,
    LayoutSizeInputErrorOf, ParentFormattingContext, PositionedRootContext,
    PositionedRootContextOf, RootContextMismatch, RootLocalPositionedContext,
    RootLocalPositionedContextOf,
};
