//! Affine focus observations for original method and insertion continuations.
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
/// The exact unavailable query of an original called focus-delegate protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceFocusDelegateStep {
    Focus(surgeist_html::HtmlFocusDelegateAction),
    Autofocus(surgeist_html::HtmlAutofocusDelegateAction),
}
/// One source-issued descendant query. This observation supplies no focus fact
/// and grants no authority to complete the called algorithm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomSourceFocusDelegateRequest {
    pub(crate) operation: crate::DomSourceOperationId,
    pub(crate) subject: crate::NodeId,
    pub(crate) root: crate::NodeId,
    pub(crate) queried_node: crate::NodeId,
    pub(crate) step: DomSourceFocusDelegateStep,
    pub(crate) occurrence: super::watcher::HookOccurrence,
}
impl DomSourceFocusDelegateRequest {
    pub const fn operation(&self) -> crate::DomSourceOperationId {
        self.operation
    }
    pub const fn subject(&self) -> crate::NodeId {
        self.subject
    }
    pub const fn root(&self) -> crate::NodeId {
        self.root
    }
    pub const fn queried_node(&self) -> crate::NodeId {
        self.queried_node
    }
    pub const fn step(&self) -> DomSourceFocusDelegateStep {
        self.step
    }
    pub const fn command(&self) -> usize {
        self.occurrence.command
    }
    pub const fn current_occurrence(&self) -> u64 {
        self.occurrence.ordinal
    }
}
/// One exact action of an actual common Element insertion continuation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomSourceAutofocusInsertionRequest {
    pub(crate) operation: crate::DomSourceOperationId,
    pub(crate) subject: crate::NodeId,
    pub(crate) action: surgeist_html::HtmlAutofocusInsertionAction,
    pub(crate) occurrence: super::watcher::HookOccurrence,
}
impl DomSourceAutofocusInsertionRequest {
    pub const fn operation(&self) -> crate::DomSourceOperationId {
        self.operation
    }
    pub const fn subject(&self) -> crate::NodeId {
        self.subject
    }
    pub const fn action(&self) -> surgeist_html::HtmlAutofocusInsertionAction {
        self.action
    }
    pub const fn command(&self) -> usize {
        self.occurrence.command
    }
    pub const fn current_occurrence(&self) -> u64 {
        self.occurrence.ordinal
    }
}
/// The single affine bridge for remembered focus and allow-focus observations.
/// A denial fulfills only its exact allow-focus action; the original owner
/// continues the Dialog child or insertion cursor to its actual terminal state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomFocusRequest {
    RememberDialog(DomSourceDialogRequest),
    AllowDialog(DomSourceDialogFocusingRequest),
    AllowAutofocusInsertion(DomSourceAutofocusInsertionRequest),
}
impl DomFocusRequest {
    pub fn operation(&self) -> crate::DomSourceOperationId {
        match self {
            Self::RememberDialog(r) => r.operation(),
            Self::AllowDialog(r) => r.operation(),
            Self::AllowAutofocusInsertion(r) => r.operation(),
        }
    }
    pub fn subject(&self) -> crate::NodeId {
        match self {
            Self::RememberDialog(r) => r.subject(),
            Self::AllowDialog(r) => r.subject(),
            Self::AllowAutofocusInsertion(r) => r.subject(),
        }
    }
}
/// The actual producer role needed by this exact focus request. Observation of
/// this target grants no execution authority or permission to move focus.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomFocusTarget {
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
pub enum DomFocusObservation {
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
    pub(crate) const fn target(self) -> DomFocusTarget {
        match self {
            Self::Remembered { host, .. } => DomFocusTarget::RememberedHost(host),
            Self::RetiredDocument {
                document,
                global,
                host,
            } => DomFocusTarget::RetiredDocumentWindow {
                document,
                global,
                host,
            },
            Self::DeniedDocument {
                document,
                global,
                host,
            } => DomFocusTarget::DeniedDocumentWindow {
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
pub struct DomFocusPreparation {
    pub(crate) request: DomFocusRequest,
    pub(crate) qualification: FocusQualification,
    pub(crate) basis: DomStamp,
}
impl DomFocusPreparation {
    pub fn request(&self) -> &DomFocusRequest {
        &self.request
    }
    pub const fn target(&self) -> DomFocusTarget {
        self.qualification.target()
    }
    /// Seal only the observation belonging to this qualified role. KnownNone
    /// requires complete-Host lifetime absence; Denied requires the exact
    /// retained Window's genuine never-activated state and ordered allow-focus
    /// decision. Neither observation attests positive focusing effects.
    pub fn publish(
        self,
        observation: DomFocusObservation,
    ) -> Result<DomFocusPublication, DomError> {
        if !matches!(
            (self.qualification, observation),
            (
                FocusQualification::Remembered { .. },
                DomFocusObservation::KnownNone
            ) | (
                FocusQualification::RetiredDocument { .. }
                    | FocusQualification::DeniedDocument { .. },
                DomFocusObservation::Denied
            )
        ) {
            return Err(DomError::InvalidSourceOperation);
        }
        Ok(DomFocusPublication {
            request: self.request,
            qualification: self.qualification,
            basis: self.basis,
        })
    }
}
/// Move-only observation retained until the exact original Source accepts it.
#[derive(Debug)]
pub struct DomFocusPublication {
    pub(crate) request: DomFocusRequest,
    pub(crate) qualification: FocusQualification,
    pub(crate) basis: DomStamp,
}
impl DomFocusPublication {
    pub fn request(&self) -> &DomFocusRequest {
        &self.request
    }
}
