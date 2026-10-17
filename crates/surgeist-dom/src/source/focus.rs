//! Affine actual Host observation for the original remembered-focus action.
use crate::{DomHostedHostId, DomSourceDialogRequest, DomSourceExecutionEntryScope, DomStamp};
/// Exact operation/action/occurrence and canonical basis, prepared for its
/// original trusted Host producer. Preparation itself observes no system focus.
#[derive(Debug)]
pub struct DomRememberedFocusPreparation {
    pub(crate) request: DomSourceDialogRequest,
    pub(crate) host: DomHostedHostId,
    pub(crate) scope: DomSourceExecutionEntryScope,
    pub(crate) basis: DomStamp,
}
impl DomRememberedFocusPreparation {
    pub fn request(&self) -> &DomSourceDialogRequest {
        &self.request
    }
    pub const fn host(&self) -> DomHostedHostId {
        self.host
    }
    /// Attest actual complete-scope known absence. The trusted Host producer
    /// must enforce this fact for the entire original Host lifetime.
    pub fn publish_known_none(self) -> DomRememberedFocusPublication {
        DomRememberedFocusPublication {
            request: self.request,
            host: self.host,
            scope: self.scope,
            basis: self.basis,
        }
    }
}
/// Move-only known absence, retained until the exact original Source accepts it.
#[derive(Debug)]
pub struct DomRememberedFocusPublication {
    pub(crate) request: DomSourceDialogRequest,
    pub(crate) host: DomHostedHostId,
    pub(crate) scope: DomSourceExecutionEntryScope,
    pub(crate) basis: DomStamp,
}
impl DomRememberedFocusPublication {
    pub fn request(&self) -> &DomSourceDialogRequest {
        &self.request
    }
}
