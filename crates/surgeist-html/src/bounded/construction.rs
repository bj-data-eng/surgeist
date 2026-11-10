//! The sole mutable owner of shared HTML construction records, and its controlled operations.

pub use crate::internal::construction::bounded::attribute_edit::{
    HtmlConstructionAttributeAction, HtmlConstructionAttributeCandidate,
    HtmlConstructionAttributeCandidateAction, HtmlConstructionAttributeCandidateObservation,
    HtmlConstructionAttributeCapacity, HtmlConstructionAttributeCompletion,
    HtmlConstructionAttributeDescription, HtmlConstructionAttributeEdit,
    HtmlConstructionAttributeEditMode, HtmlConstructionAttributeObservation,
    HtmlConstructionAttributePlanRetirement, HtmlConstructionAttributeRetirementAction,
    HtmlConstructionAttributeRetirementDescription, HtmlConstructionAttributeSlot,
    HtmlConstructionAttributeSlotAction, HtmlConstructionAttributeSlotObservation,
    HtmlConstructionAttributeWork, HtmlConstructionReadyAttributeSlot,
};
pub use crate::internal::construction::bounded::operations::{
    HtmlConstructionAbort, HtmlConstructionAction, HtmlConstructionActionDescription,
    HtmlConstructionActionFailure, HtmlConstructionCommand, HtmlConstructionCommit,
    HtmlConstructionContent, HtmlConstructionGrowth, HtmlConstructionObservation,
    HtmlConstructionOperation, HtmlConstructionReadyOperation, HtmlConstructionRetirement,
    HtmlConstructionRetirementAction, HtmlConstructionRetirementDescription,
    HtmlConstructionStepWork,
};
pub use crate::internal::construction::bounded::text_append::{
    HtmlConstructionTextAppend, HtmlConstructionTextAppendAction,
    HtmlConstructionTextAppendCapacity, HtmlConstructionTextAppendCompletion,
    HtmlConstructionTextAppendDescription, HtmlConstructionTextAppendObservation,
    HtmlConstructionTextAppendReady, HtmlConstructionTextAppendWork,
};
pub use crate::internal::construction::bounded::{
    HtmlConstructionAllocationId, HtmlConstructionAllocationRole,
    HtmlConstructionAllocationSnapshot, HtmlConstructionChargeFault,
    HtmlConstructionInitialization, HtmlConstructionInitializationAction,
    HtmlConstructionInitializationObservation, HtmlConstructionLease, HtmlConstructionNode,
    HtmlConstructionReadyInitialization, HtmlConstructionRecovery, HtmlConstructionRefusal,
    HtmlConstructionRetired, HtmlConstructionStorage,
};
