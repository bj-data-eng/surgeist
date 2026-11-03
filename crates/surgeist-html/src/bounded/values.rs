//! Attribute storage and validation and shared value pools.

pub use crate::internal::values::bounded::{
    HtmlAttributeStorage, HtmlAttributeValidation, HtmlAttributeValidationAction,
    HtmlAttributeValidationError, HtmlAttributeValidationLayout, HtmlValidatedAttributes,
};
pub use crate::internal::values::name_index::{HtmlAttributeNameIndex, HtmlNameIndexSlot};
pub use crate::internal::values::sharing::{
    HtmlAttributePool, HtmlAttributePoolCandidate, HtmlAttributePoolOperation,
    HtmlAttributePoolResult, HtmlStringPool, HtmlStringPoolCandidate, HtmlStringPoolOperation,
    HtmlStringPoolResult, HtmlValuePoolAction, HtmlValuePoolActionKind, HtmlValuePoolChange,
    HtmlValuePoolError, HtmlValuePoolLayout, HtmlValuePoolMark, HtmlValuePoolOccurrence,
    HtmlValuePoolProgress, HtmlValuePoolRoot, HtmlValuePoolSeal, HtmlValuePoolSealRefusal,
    HtmlValuePoolUsage, HtmlValuePoolWork,
};
