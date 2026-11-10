//! Attribute storage and validation and the name index.

pub use crate::internal::values::attribute_name_index::HtmlAttributeNameIndex;
pub use crate::internal::values::bounded::{
    HtmlAttributeStorage, HtmlAttributeValidation, HtmlAttributeValidationAction,
    HtmlAttributeValidationError, HtmlValidatedAttributes,
};
pub use crate::internal::values::finish::HtmlFinishFailure;
pub use crate::internal::values::name_index::{
    HtmlNameIndex, HtmlNameIndexError, HtmlNameIndexForeignProbe, HtmlNameIndexHasher,
    HtmlNameIndexInsertError, HtmlNameIndexLookup, HtmlNameIndexProbe, HtmlNameIndexSlot,
    HtmlNameIndexSlots, HtmlNameIndexStep, HtmlNameIndexStorageError, HtmlNameIndexTable,
    HtmlNameIndexVacancy,
};
