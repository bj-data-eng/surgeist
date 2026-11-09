//! Attribute storage and validation, shared value pools and the name index.

pub use crate::internal::values::attribute_name_index::HtmlAttributeNameIndex;
pub use crate::internal::values::bounded::{
    HtmlAttributeStorage, HtmlAttributeValidation, HtmlAttributeValidationAction,
    HtmlAttributeValidationError, HtmlAttributeValidationLayout, HtmlValidatedAttributes,
};
pub use crate::internal::values::finish::HtmlFinishFailure;
pub use crate::internal::values::name_index::{
    HtmlNameIndex, HtmlNameIndexError, HtmlNameIndexForeignProbe, HtmlNameIndexHasher,
    HtmlNameIndexInsertError, HtmlNameIndexLookup, HtmlNameIndexProbe, HtmlNameIndexSlot,
    HtmlNameIndexSlots, HtmlNameIndexStep, HtmlNameIndexStorageError, HtmlNameIndexTable,
    HtmlNameIndexVacancy,
};
pub use crate::internal::values::sharing::{
    HtmlAttributePool, HtmlAttributePoolCandidate, HtmlAttributePoolOperation,
    HtmlAttributePoolResult, HtmlStringPool, HtmlStringPoolCandidate, HtmlStringPoolOperation,
    HtmlStringPoolResult, HtmlValuePoolAction, HtmlValuePoolActionKind, HtmlValuePoolChange,
    HtmlValuePoolError, HtmlValuePoolLayout, HtmlValuePoolMark, HtmlValuePoolOccurrence,
    HtmlValuePoolProgress, HtmlValuePoolRoot, HtmlValuePoolSeal, HtmlValuePoolSealRefusal,
    HtmlValuePoolUsage,
};
