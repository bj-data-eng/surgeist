//! `@page` and margin rules and page projection.

pub(crate) mod page_projection;
pub(crate) mod page_serialization;
pub(crate) mod syntax;

#[doc(inline)]
pub use page_projection::{
    CssMarginRuleView, CssPageProjectionError, CssPageRuleView, CssSpecifiedPageDeclarationBlock,
    CssSpecifiedPageDeclarationEntry,
};
#[doc(inline)]
pub use syntax::{
    CssMarginBox, CssMarginDeclarationBlock, CssMarginRule, CssPageBlockError, CssPageBody,
    CssPageDeclaration, CssPageDeclarationBlock, CssPageDescriptor, CssPageDescriptorKind,
    CssPageDescriptorValue, CssPageDescriptorValueRef, CssPageMarks, CssPageNamedSize,
    CssPageOrientation, CssPageOutputOrientation, CssPagePseudo, CssPageRule, CssPageSelector,
    CssPageSelectorList, CssPageSize, CssPageSizeValue, CssPageSpecificity,
};
