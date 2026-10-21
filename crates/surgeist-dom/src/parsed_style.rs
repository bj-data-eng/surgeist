//! Original parsed Style processing and its bounded external-owner joins.
mod frontier;
mod notification;
mod sheet;
mod subject;
mod successor;
use crate::{
    DocumentId, DocumentString, DomError, DomParsedHtmlAssociationAdmissionError,
    DomParsedHtmlAssociationId, DomParsedHtmlHistoricalDocument,
};
pub use frontier::*;
pub use notification::*;
pub use sheet::*;
pub use subject::*;
pub use successor::*;
use surgeist_html::{
    HtmlCspAssociation, HtmlCspProtocolError, HtmlNodeId, HtmlStyleProtocolError, HtmlStyleState,
    HtmlStyleUpdateAction,
};
use surgeist_html_parser::{
    HtmlParseAssociationContextEpochId, HtmlParseAssociationContextRequirement,
    HtmlParseAssociationContextSlotId, HtmlParseAssociationOccurrenceId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomParsedHtmlStylePhase {
    Unstarted,
    AwaitingInitialPolicy,
    InlinePolicyPending,
    CreateSheetPending,
    InlinePolicyBlocked,
    ReadScriptBlockingPending,
    ReadRenderBlockingPending,
    /// This invocation finished rendering actions; the whole history is Pending.
    RenderingComplete,
}
/// Original facts still required by this rendering invocation. No current
/// environment or final-tree body observation can replace the retained basis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomParsedHtmlStyleRenderingRequirement {
    /// The nontrivial original media value requires an original environment.
    MediaEnvironment,
    /// A required original Document fact was unavailable or deliberately unbound.
    OriginalContext {
        role: surgeist_html_parser::HtmlParseAssociationContextRole,
        requirement: HtmlParseAssociationContextRequirement,
    },
}
/// Only supported rendering actions of one original invocation. Neither variant
/// acknowledges the contribution or establishes general Style readiness.
#[derive(Debug)]
pub enum DomParsedHtmlStyleRenderingProgress {
    /// The same original cursor remains at ReadRenderBlocking.
    Pending(DomParsedHtmlStyleRenderingRequirement),
    /// Actual DOM publication and narrow, operation-qualified membership evidence.
    Complete(DomParsedHtmlStyleRenderingCompletion),
}
pub use crate::store::pending_associations::DomParsedHtmlStyleRenderingCompletion;

#[derive(Debug)]
pub enum DomParsedHtmlStyleError {
    Admission(DomParsedHtmlAssociationAdmissionError),
    Busy {
        contribution: DomParsedHtmlAssociationId,
    },
    Poisoned {
        contribution: DomParsedHtmlAssociationId,
    },
    UnsupportedProfile {
        contribution: DomParsedHtmlAssociationId,
    },
    WrongRequest,
    ExecutionNotAdmitted {
        contribution: DomParsedHtmlAssociationId,
    },
    WrongExecutionProfile {
        profile: surgeist_html::HtmlStyleExecutionProfile,
    },
    AlreadyBound,
    MissingPolicy,
    ReceiverOccupied,
    NotificationTransferred,
    HandoffRequired {
        occurrence: HtmlParseAssociationOccurrenceId,
    },
    MissingObservation,
    InvalidSheetInput,
    WrongPhase {
        phase: DomParsedHtmlStylePhase,
    },
    DecisionAlreadyAccepted {
        contribution: DomParsedHtmlAssociationId,
    },
    PolicyContext {
        requirement: HtmlParseAssociationContextRequirement,
    },
    Csp(HtmlCspProtocolError),
    Protocol(HtmlStyleProtocolError),
}
impl From<DomError> for DomParsedHtmlStyleError {
    fn from(e: DomError) -> Self {
        Self::Admission(e.into())
    }
}
impl From<DomParsedHtmlAssociationAdmissionError> for DomParsedHtmlStyleError {
    fn from(e: DomParsedHtmlAssociationAdmissionError) -> Self {
        Self::Admission(e)
    }
}
impl From<HtmlCspProtocolError> for DomParsedHtmlStyleError {
    fn from(e: HtmlCspProtocolError) -> Self {
        Self::Csp(e)
    }
}
impl From<HtmlStyleProtocolError> for DomParsedHtmlStyleError {
    fn from(e: HtmlStyleProtocolError) -> Self {
        Self::Protocol(e)
    }
}
impl std::fmt::Display for DomParsedHtmlStyleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parsed Style prefix refused: {self:?}")
    }
}
impl std::error::Error for DomParsedHtmlStyleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Admission(e) => Some(e),
            Self::Csp(e) => Some(e),
            Self::Protocol(e) => Some(e),
            _ => None,
        }
    }
}

/// Immutable identity of one original Style operation. Copying it supplies no
/// binding, processing, policy or publication authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomParsedHtmlStyleQualification {
    pub(crate) contribution: DomParsedHtmlAssociationId,
    pub(crate) occurrence: HtmlParseAssociationOccurrenceId,
    pub(crate) subject: HtmlNodeId,
    pub(crate) original_document: DomParsedHtmlHistoricalDocument,
    pub(crate) document: DocumentId,
    pub(crate) context_epoch: HtmlParseAssociationContextEpochId,
}
impl DomParsedHtmlStyleQualification {
    pub fn contribution(&self) -> DomParsedHtmlAssociationId {
        self.contribution
    }
    pub fn occurrence(&self) -> &HtmlParseAssociationOccurrenceId {
        &self.occurrence
    }
    pub fn subject(&self) -> &HtmlNodeId {
        &self.subject
    }
    pub fn original_document(&self) -> &DomParsedHtmlHistoricalDocument {
        &self.original_document
    }
    pub fn document(&self) -> DocumentId {
        self.document
    }
    pub fn context_epoch(&self) -> &HtmlParseAssociationContextEpochId {
        &self.context_epoch
    }
}
/// The actual outstanding OfflineUnbound policy slot for one original operation.
/// A known original association never produces this binding request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomParsedHtmlStyleInitialPolicyRequest {
    pub(crate) qualification: DomParsedHtmlStyleQualification,
    pub(crate) slot: HtmlParseAssociationContextSlotId,
}
impl DomParsedHtmlStyleInitialPolicyRequest {
    pub fn qualification(&self) -> &DomParsedHtmlStyleQualification {
        &self.qualification
    }
    pub fn slot(&self) -> &HtmlParseAssociationContextSlotId {
        &self.slot
    }
}
/// Closed source proof retained without an additional policy graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StylePolicyOrigin {
    Offline(HtmlParseAssociationContextSlotId),
    Known {
        document: surgeist_html::HtmlCspDocumentId,
        generation: surgeist_html::HtmlCspAssociationGeneration,
        policy: surgeist_html::HtmlCspPolicyId,
        revision: surgeist_html::HtmlCspPolicyRevision,
        profile: Option<surgeist_html::HtmlCspProcessorProfile>,
    },
}
impl StylePolicyOrigin {
    pub(crate) fn known(policy: &HtmlCspAssociation) -> Self {
        Self::Known {
            document: policy.document().clone(),
            generation: policy.generation().clone(),
            policy: policy.policy().id().clone(),
            revision: policy.policy().revision(),
            profile: policy.policy().profile().cloned(),
        }
    }
}
/// One stored historical inline-policy request, not native current Style source
/// or a Meta CSP insertion request. The canonical hold consumes observations.
pub struct DomParsedHtmlStyleInlinePolicyRequest {
    pub(crate) qualification: DomParsedHtmlStyleQualification,
    pub(crate) text: DocumentString,
    pub(crate) nonce: Option<DocumentString>,
    pub(crate) state: HtmlStyleState,
    pub(crate) policy: HtmlCspAssociation,
    pub(crate) origin: StylePolicyOrigin,
    pub(crate) cursor: surgeist_html::HtmlStyleUpdateCursor,
    pub(crate) created_sheet: Option<crate::DomStyleSheet>,
}
impl DomParsedHtmlStyleInlinePolicyRequest {
    pub fn qualification(&self) -> &DomParsedHtmlStyleQualification {
        &self.qualification
    }
    pub fn text(&self) -> &DocumentString {
        &self.text
    }
    pub fn nonce(&self) -> Option<&DocumentString> {
        self.nonce.as_ref()
    }
    pub fn state(&self) -> HtmlStyleState {
        self.state
    }
    pub fn policy(&self) -> &HtmlCspAssociation {
        &self.policy
    }
    pub fn action(&self) -> HtmlStyleUpdateAction {
        self.cursor.action()
    }
}

/// An external supplier's decision for the exact immutable historical request.
/// This records an actual evaluation, not evaluator availability or a Meta CSP
/// processor attestation. No policy is evaluated by this constructor.
///
/// A reached request is formed once and never replaced or reset. Its original
/// qualification and immutable bound policy therefore identify its exact text
/// and nonce, without retaining those backings outside canonical custody.
/// Identity copies grant no execution or currentness authority.
///
/// Invoke an external supplier using the canonical read loan, then end that loan
/// before beginning the accepting command. `None` explicitly leaves evaluation
/// unavailable; it never means permission:
/// ```no_run
/// use surgeist_dom::*;
/// use std::sync::atomic::AtomicBool;
/// fn supply(dom: &mut Dom, id: DomParsedHtmlAssociationId,
///           limits: DomParsedHtmlAssociationLimits, cancel: &AtomicBool,
///           evaluate: impl FnOnce(&DomParsedHtmlStyleInlinePolicyRequest) -> Option<bool>) {
///     let mut input = {
///         let view = dom.parsed_html_style_prefix(id).unwrap();
///         let request = view.inline_policy_request().unwrap();
///         evaluate(request).map(|allowed|
///             DomParsedHtmlStyleInlinePolicyObservation::evaluated(request, allowed))
///     };
///     if input.is_some() {
///         let mut hold = dom.begin_parsed_html_style(id, limits, cancel).unwrap();
///         let _ = hold.accept_inline_policy(&mut input, cancel);
///         // A refused command leaves this same input available for retry.
///     }
/// }
/// ```
#[derive(Debug)]
pub struct DomParsedHtmlStyleInlinePolicyObservation {
    pub(crate) qualification: DomParsedHtmlStyleQualification,
    pub(crate) document: surgeist_html::HtmlCspDocumentId,
    pub(crate) generation: surgeist_html::HtmlCspAssociationGeneration,
    pub(crate) policy: surgeist_html::HtmlCspPolicyId,
    pub(crate) revision: surgeist_html::HtmlCspPolicyRevision,
    pub(crate) profile: Option<surgeist_html::HtmlCspProcessorProfile>,
    pub(crate) allowed: bool,
}
impl DomParsedHtmlStyleInlinePolicyObservation {
    pub fn evaluated(request: &DomParsedHtmlStyleInlinePolicyRequest, allowed: bool) -> Self {
        Self {
            qualification: request.qualification.clone(),
            document: request.policy.document().clone(),
            generation: request.policy.generation().clone(),
            policy: request.policy.policy().id().clone(),
            revision: request.policy.policy().revision(),
            profile: request.policy.policy().profile().cloned(),
            allowed,
        }
    }
    pub fn qualification(&self) -> &DomParsedHtmlStyleQualification {
        &self.qualification
    }
    pub fn allowed(&self) -> bool {
        self.allowed
    }
}
