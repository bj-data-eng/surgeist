//! Hosted lifecycle: identities, limits, creation and permissions policy.
//!
//! Hosted creation and destruction run controlled retained operations through the runtime
//! and the same canonical owner. `DomHostedSource` is the unique issuer that the external
//! lifecycle owner retains, and the `DomHostedHostId`, `DomHostedGlobalId`,
//! `DomHostedNavigableId` and related types identify hosted objects. `DomHostedCreation`
//! holds the completed creation identities, and `DomHostedCreationStep`,
//! `DomHostedCreationLease` and `DomHostedCreationPublication` drive its phases.
//! `DomHostedLimits` and `DomHostedRecordLimits` bound storage and work.
//! `DomInitialPermissionsPolicy` and `DomPermissionsPolicyValue` are report data for the
//! initial permissions policy.
//!
//! Child navigables, history, destruction and constructed stylesheets have their own
//! submodules. A refused operation preserves the original operation and its unconsumed
//! publication. `docs/hosted-documents.md` owns the contract.

pub mod child;
pub mod destruction;
pub mod history;
pub mod sheet;

pub use crate::internal::hosted::{
    DomHostedBrowsingContextId, DomHostedCreation, DomHostedCreationLease,
    DomHostedCreationPublication, DomHostedCreationStep, DomHostedError, DomHostedExecutionRequest,
    DomHostedFinishLoadingRequest, DomHostedGlobalId, DomHostedHostId, DomHostedLimits,
    DomHostedMakeActiveRequest, DomHostedMoment, DomHostedNavigableId, DomHostedOperationId,
    DomHostedPolicyPublication, DomHostedPolicyRequest, DomHostedPreparedTopLevel,
    DomHostedRecordLimits, DomHostedResource, DomHostedRuntimeRetention, DomHostedSource,
    DomHostedSourceId, DomHostedSourceRevision, DomHostedTopLevelPreparation,
    DomInitialPermissionsPolicy, DomInitialPermissionsPolicyProfile, DomPermissionsPolicyValue,
};

pub use crate::internal::store::DomWindowNamedValue;
