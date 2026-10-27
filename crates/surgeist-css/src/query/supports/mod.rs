//! Supports conditions and named supports test bodies.

pub(crate) mod construction;
pub(crate) mod named_supports;
pub(crate) mod named_supports_composition;
pub(crate) mod named_supports_serialization;
pub(crate) mod syntax;

#[doc(inline)]
pub use construction::{CssSupportsAtRule, CssSupportsConstructionError};
#[doc(inline)]
pub use named_supports::{
    CssSupportsAtRuleTest, CssSupportsConditionName, CssSupportsQualifiedRuleTest,
    CssSupportsTestBody, CssSupportsTestChild, CssSupportsTestChildRef, CssSupportsTestDeclaration,
    CssSupportsTestDeclarations, CssSupportsTestItem,
};
#[doc(inline)]
pub use syntax::{CssSupportsDeclaration, CssSupportsLeaf};
