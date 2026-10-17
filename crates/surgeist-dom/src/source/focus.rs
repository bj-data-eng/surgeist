//! Affine qualified observations for the original Dialog focusing actions.
use crate::{
    DocumentId, DomError, DomHostedGlobalId, DomHostedHostId, DomSourceDialogRequest,
    DomSourceExecutionEntryScope, DomStamp,
};
/// One exact source-issued action of the called Dialog focusing algorithm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomSourceDialogFocusingRequest {
    pub(crate) operation: crate::DomSourceOperationId,
    pub(crate) subject: crate::NodeId,
    pub(crate) action: surgeist_html::HtmlDialogFocusingAction,
    pub(crate) occurrence: super::watcher::HookOccurrence,
}
impl DomSourceDialogFocusingRequest {
    pub const fn operation(&self) -> crate::DomSourceOperationId {
        self.operation
    }
    pub const fn subject(&self) -> crate::NodeId {
        self.subject
    }
    pub const fn action(&self) -> surgeist_html::HtmlDialogFocusingAction {
        self.action
    }
    pub const fn command(&self) -> usize {
        self.occurrence.command
    }
    pub const fn current_occurrence(&self) -> u64 {
        self.occurrence.ordinal
    }
}
/// The closed affine bridge accepts remembered-focus or a called allow-focus
/// observation. Permission denial never acknowledges the outer focusing call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomDialogFocusRequest {
    Remember(DomSourceDialogRequest),
    Allow(DomSourceDialogFocusingRequest),
}
impl DomDialogFocusRequest {
    pub fn operation(&self) -> crate::DomSourceOperationId {
        match self {
            Self::Remember(r) => r.operation(),
            Self::Allow(r) => r.operation(),
        }
    }
    pub fn subject(&self) -> crate::NodeId {
        match self {
            Self::Remember(r) => r.subject(),
            Self::Allow(r) => r.subject(),
        }
    }
}
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
    /// Actual current live Window whose ordered allowed-to-use check is false.
    DeniedDocumentWindow {
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
    DeniedDocument {
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
            Self::DeniedDocument {
                document,
                global,
                host,
            } => DomDialogFocusTarget::DeniedDocumentWindow {
                document,
                global,
                host,
            },
        }
    }
    pub(crate) const fn host(self) -> DomHostedHostId {
        match self {
            Self::Remembered { host, .. }
            | Self::RetiredDocument { host, .. }
            | Self::DeniedDocument { host, .. } => host,
        }
    }
}
/// Exact action/occurrence, canonical basis and qualified producer role.
/// Preparation itself neither observes nor changes system focus.
#[derive(Debug)]
pub struct DomDialogFocusPreparation {
    pub(crate) request: DomDialogFocusRequest,
    pub(crate) qualification: FocusQualification,
    pub(crate) basis: DomStamp,
}
impl DomDialogFocusPreparation {
    pub fn request(&self) -> &DomDialogFocusRequest {
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
                FocusQualification::RetiredDocument { .. }
                    | FocusQualification::DeniedDocument { .. },
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
    pub(crate) request: DomDialogFocusRequest,
    pub(crate) qualification: FocusQualification,
    pub(crate) basis: DomStamp,
}
impl DomDialogFocusPublication {
    pub fn request(&self) -> &DomDialogFocusRequest {
        &self.request
    }
}
