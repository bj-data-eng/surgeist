//! Operands and results of batch edits, change coverage, child-list observers and
//! node-retirement watches.
//!
//! A `DomBatch` returns batch-local tickets (`CreatedNode`, `AttachedAttribute`,
//! `RemovedAttribute`, `AttachedShadowRoot`) that resolve to identities only through the
//! `DomCommit` of a successful publication, so a failed batch cannot leak an identity.
//! `NodeOperand` and `DomNodeOrString` are the operand types (strings retain exact
//! UTF-16). `DomChanges` and `DomChange` report the changed domains, and
//! `DomHistoryError` reports that a complete interval is no longer available, so a
//! consumer must recompute from a snapshot.
//!
//! The child-list observer types report one record per interested observer in discovery
//! order, and the retirement types watch one node generation for destruction.
//! `docs/canonical-tree.md`, `docs/child-list-observation.md` and
//! `docs/compound-mutations.md` own the contracts.

pub use crate::internal::batch::{
    AttachedAttribute, AttachedShadowRoot, CreatedNode, NodeOperand, RemovedAttribute,
};
pub use crate::internal::child_list::{
    DomChildListObservationLimits, DomChildListObserverId, DomChildListRecord,
};
pub use crate::internal::compound::DomNodeOrString;
pub use crate::internal::source::{
    DomDetachedNodeCreation, DomElementCreationName, DomElementCreationOptions,
    DomProcessingInstructionAttributeEdit,
};
pub use crate::internal::store::source::operation::DomSourceNodeOutputWatch;
pub use crate::internal::store::{
    DomChange, DomChanges, DomChildListNotification, DomHistoryError, DomNodeRetirementInbox,
    DomNodeRetirementLimits, DomNodeRetirementNotice, DomNodeRetirementWatch, DomWatchedNode,
};
