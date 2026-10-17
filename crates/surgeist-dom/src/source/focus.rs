//! Affine qualified observations for the original Dialog focusing actions.
use crate::{
    DocumentId, DomError, DomHostedGlobalId, DomHostedHostId, DomSourceDialogRequest,
    DomSourceExecutionEntryScope, DomStamp,
};
/// The actual producer role needed by this exact Dialog action. Observation of
/// this target grants no execution authority or permission to move focus.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomDialogFocusTarget {
    RememberedHost(DomHostedHostId),
    RetiredDocumentWindow {
        document: DocumentId,
        global: DomHostedGlobalId,
        host: DomHostedHostId,
    },
}
/// Closed observations supported by the corresponding trusted producer role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomDialogFocusObservation {
    KnownNone,
    Denied,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FocusQualification {
    Remembered {
        host: DomHostedHostId,
        scope: DomSourceExecutionEntryScope,
    },
    RetiredDocument {
        document: DocumentId,
        global: DomHostedGlobalId,
        host: DomHostedHostId,
    },
}
impl FocusQualification {
    pub(crate) const fn target(self) -> DomDialogFocusTarget {
        match self {
            Self::Remembered { host, .. } => DomDialogFocusTarget::RememberedHost(host),
            Self::RetiredDocument {
                document,
                global,
                host,
            } => DomDialogFocusTarget::RetiredDocumentWindow {
                document,
                global,
                host,
            },
        }
    }
    pub(crate) const fn host(self) -> DomHostedHostId {
        match self {
            Self::Remembered { host, .. } | Self::RetiredDocument { host, .. } => host,
        }
    }
}
/// Exact action/occurrence, canonical basis and qualified producer role.
/// Preparation itself neither observes nor changes system focus.
#[derive(Debug)]
pub struct DomDialogFocusPreparation {
    pub(crate) request: DomSourceDialogRequest,
    pub(crate) qualification: FocusQualification,
    pub(crate) basis: DomStamp,
}
impl DomDialogFocusPreparation {
    pub fn request(&self) -> &DomSourceDialogRequest {
        &self.request
    }
    pub const fn target(&self) -> DomDialogFocusTarget {
        self.qualification.target()
    }
    /// Seal only the observation belonging to this qualified role. KnownNone
    /// requires complete-Host lifetime absence; Denied requires the exact
    /// retained Window's genuine never-activated state and ordered allow-focus
    /// decision. Neither observation attests positive focusing effects.
    pub fn publish(
        self,
        observation: DomDialogFocusObservation,
    ) -> Result<DomDialogFocusPublication, DomError> {
        if !matches!(
            (self.qualification, observation),
            (
                FocusQualification::Remembered { .. },
                DomDialogFocusObservation::KnownNone
            ) | (
                FocusQualification::RetiredDocument { .. },
                DomDialogFocusObservation::Denied
            )
        ) {
            return Err(DomError::InvalidSourceOperation);
        }
        Ok(DomDialogFocusPublication {
            request: self.request,
            qualification: self.qualification,
            basis: self.basis,
        })
    }
}
/// Move-only observation retained until the exact original Source accepts it.
#[derive(Debug)]
pub struct DomDialogFocusPublication {
    pub(crate) request: DomSourceDialogRequest,
    pub(crate) qualification: FocusQualification,
    pub(crate) basis: DomStamp,
}
impl DomDialogFocusPublication {
    pub fn request(&self) -> &DomSourceDialogRequest {
        &self.request
    }
}
