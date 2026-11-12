//! The sole mutable owner of shared HTML construction records, and its direct operations.

pub use crate::internal::construction::bounded::attribute_list::{
    HtmlAttributeMerge, HtmlConstructionAttributeCapacity, HtmlConstructionAttributeError,
};
pub use crate::internal::construction::bounded::records::{
    HtmlConstructionBank, HtmlConstructionBankCapacity, HtmlConstructionBankGrowth,
    HtmlConstructionChange, HtmlConstructionGrowthRecord, HtmlConstructionPublishRefusal,
    HtmlConstructionRecordError, HtmlRecordAdmission, NameId,
};
pub use crate::internal::construction::bounded::text_data::{
    HtmlConstructionTextCapacity, HtmlConstructionTextError, HtmlConstructionTextGrowth,
    HtmlTextAdmission,
};
pub use crate::internal::construction::bounded::{HtmlConstructionNode, HtmlConstructionStorage};
