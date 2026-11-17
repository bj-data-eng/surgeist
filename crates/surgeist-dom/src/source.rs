//! The domain-independent Source continuation: edit, request, input, limits, lease,
//! status and cleanup.
//!
//! Use Source when a canonical operation reaches an external service.
//! `Dom::begin_source_edit` takes a `DomSourceInput`, which a `DomSourceEdit` (exactly
//! one requested edit; there is no command queue) converts into, runs it through the
//! canonical source engine and returns a `DomSourceLease`, the execution permission for
//! that operation. The service the engine reaches is a `DomSourceRequest`;
//! `DomSourceInput` is the owned input of the original entry, and `DomSourceProgress`,
//! `DomSourceStatus` and `DomSourceCompletion` report where the operation stands.
//! `DomSourceLimits` admits the whole composed operation. `DomSourceCleanupCause` is the
//! original native failure whose invocation cleanup is still pending, and
//! `DomSourceStoppedAction` the exact externally stopped native action retained while
//! cleanup is pending.
//!
//! Once a phase publishes, dropping the lease parks its mandatory continuation in the
//! DOM, and `Dom::resume_source_edit` reacquires that exact operation. Feature-specific
//! requests live with their modules (`dialog`, `focus`, `event`, `url` and others);
//! `source::attribute` holds the attribute verification hooks. `docs/source-and-style.md`
//! owns the contract.

pub mod attribute;

pub use crate::internal::source::{
    DomSourceCleanupCause, DomSourceCompletion, DomSourceEdit, DomSourceExecutionEntryScope,
    DomSourceFailure, DomSourceInput, DomSourceInvariantFailure, DomSourceInvariantReason,
    DomSourceLimits, DomSourceOperationId, DomSourceOutput, DomSourcePhase, DomSourceProgress,
    DomSourceRequest, DomSourceStatus, DomSourceStoppedAction,
};
pub use crate::internal::store::source::operation::DomSourceLease;
