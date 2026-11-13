//! One requested canonical edit and its retained source continuation.
mod attribute;
pub(crate) mod callback;
pub use attribute::*;
mod design_mode;
pub(crate) mod event;
pub use design_mode::{
    DomDesignModeRangeDisposition, DomSourceDesignModeRequest, DomSourceDesignModeStep,
};
mod focus;
mod scope;
mod url;
pub(crate) use focus::FocusQualification;
pub use focus::{
    DomFocusObservation, DomFocusPreparation, DomFocusPublication, DomFocusRequest, DomFocusTarget,
    DomSourceAutofocusInsertionRequest, DomSourceDialogFocusingRequest,
    DomSourceFocusDelegateRequest, DomSourceFocusDelegateStep,
};
pub use scope::DomSourceExecutionEntryScope;
pub(crate) mod watcher;
use crate::batch::{AttributeSetterMode, Command, Creation};
use crate::view_transition::*;
use crate::{DomDocumentStamp, DomError, DomSnapshot, DomStyleRequest, NodeId};
pub use event::*;
use std::sync::Arc;
use surgeist_css::values::view_transition::CssViewTransitionIdent;
use surgeist_html::values::{DocumentString, Namespace};
pub(crate) use url::UrlBasis;
pub use url::{
    DomSourceUrlObservation, DomSourceUrlOrigin, DomSourceUrlRequest, DomSourceUrlStage,
};
pub use watcher::{
    DomCloseWatcherHistoryWindows, DomCloseWatcherRequestPreparation,
    DomCloseWatcherRequestPublication, DomDialogWatcherPreparation, DomDialogWatcherPublication,
    DomSourceCleanupCause, DomSourceCloseWatcherInvocationId, DomSourceCloseWatcherRequest,
    DomSourceCloseWatcherStep, DomSourceInvariantFailure, DomSourceInvariantReason,
    DomSourceStoppedAction,
};

/// Absolute finite admission for one composed source operation. Resuming does
/// not reset work or publication/output counters. CSS and CSSOM retain their
/// independent producer limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceLimits {
    pub(crate) retained_bytes: usize,
    pub(crate) retained_units: usize,
    pub(crate) work: u64,
    pub(crate) phases: u64,
    pub(crate) outputs: usize,
    pub(crate) identities: u64,
    pub(crate) nested_operations: usize,
}
impl DomSourceLimits {
    pub const fn new(
        retained_bytes: usize,
        retained_units: usize,
        work: u64,
        phases: u64,
        outputs: usize,
        identities: u64,
        nested_operations: usize,
    ) -> Self {
        Self {
            retained_bytes,
            retained_units,
            work,
            phases,
            outputs,
            identities,
            nested_operations,
        }
    }
    /// Maximum simultaneous original Source owners, including this operation.
    pub const fn nested_operations(self) -> usize {
        self.nested_operations
    }
    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
    pub const fn retained_string_code_units(self) -> usize {
        self.retained_units
    }
    pub const fn work(self) -> u64 {
        self.work
    }
    pub const fn phase_publications(self) -> u64 {
        self.phases
    }
    pub const fn output_records(self) -> usize {
        self.outputs
    }
    pub const fn published_native_identities(self) -> u64 {
        self.identities
    }
}

/// One retained source operation, qualified by its canonical owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceOperationId {
    pub(crate) owner: crate::DomOwnerId,
    pub(crate) serial: u64,
}

/// Actual continuation phase, distinct from canonical tree publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourcePhase {
    /// Original private canonical segment.
    Staging,
    /// A standard attribute setter awaits its registered verifier.
    AttributeVerification,
    /// A guest attachment awaits custody of its replaced Attr.
    AttributeAttachmentOutput,
    /// A detached element factory awaits its registered constructor.
    CustomElementConstruction,
    /// A reached synchronous firing awaits its original clock observation.
    EventFiring,
    /// Coherent current-state observation awaiting actual event dispatch.
    EventObservation,
    /// Qualified event outcome retained before current-state rebase.
    EventReceiptAccepted,
    /// A Dialog action waiting for its qualified external service.
    DialogService,
    /// A common insertion action awaiting its qualified service.
    AutofocusInsertionService,
    /// The original designMode setter awaits an actual selection/focus service.
    DesignModeService,
    /// Exact step8 scheduling join, before any invocation prefix is published.
    FullscreenExitHandoff,
    /// Original HTML Script preparation is parked in the private segment.
    ScriptPreparation,
    /// An original Meta CSP insertion awaits its policy processor.
    MetaCsp,
    /// Actual original invocation cleanup remains after a native failure.
    PendingCleanup,
}
/// Actual publication status; structural snapshots remain readable while source
/// hooks are incomplete, and dependent semantic reads reject that state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceStatus {
    pub(crate) operation: DomSourceOperationId,
    pub(crate) published: bool,
    pub(crate) cancelled: bool,
    pub(crate) cleanup: Option<DomSourceCleanupCause>,
    pub(crate) work: u64,
    pub(crate) phases: u64,
    pub(crate) parent: Option<DomSourceOperationId>,
    pub(crate) phase: DomSourcePhase,
}
impl DomSourceStatus {
    /// Exact original native failure while actual invocation cleanup is pending.
    pub const fn cleanup_cause(self) -> Option<DomSourceCleanupCause> {
        self.cleanup
    }
    /// The actual retained parent operation, if this is a nested edit.
    pub fn parent_operation(self) -> Option<DomSourceOperationId> {
        self.parent
    }
    /// Current owner phase, independently of tree revision/publication.
    pub fn phase(self) -> DomSourcePhase {
        self.phase
    }
    pub fn operation(self) -> DomSourceOperationId {
        self.operation
    }
    pub fn has_published(self) -> bool {
        self.published
    }
    pub fn cancellation_requested(self) -> bool {
        self.cancelled
    }
    pub fn spent_work(self) -> u64 {
        self.work
    }
    pub fn published_phases(self) -> u64 {
        self.phases
    }
}
/// The actual service reached by the retained canonical source engine.
#[derive(Clone, Debug)]
pub enum DomSourceRequest {
    AttributeVerification(DomSourceAttributeVerificationRequest),
    AttributeAttachmentOutput(DomSourceAttributeAttachmentOutput),
    CustomElementConstruction(DomSourceCustomElementRequest),
    MetaCsp(DomSourceMetaCspRequest),
    FullscreenExitHandoff(crate::DomSourceFullscreenExitHandoffRequest),
    Style(DomStyleRequest),
    HostedChild(crate::DomHostedChildRequest),
    Event(DomSourceEventRequest),
    Dialog(DomSourceDialogRequest),
    DialogFocusing(DomSourceDialogFocusingRequest),
    FocusDelegate(DomSourceFocusDelegateRequest),
    AutofocusInsertion(DomSourceAutofocusInsertionRequest),
    CloseWatcher(DomSourceCloseWatcherRequest),
    DesignMode(DomSourceDesignModeRequest),
    Script(DomSourceScriptRequest),
}

/// Exact synchronous constructor occurrence reached by a Document factory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceCustomElementRequest {
    pub(crate) operation: DomSourceOperationId,
    pub(crate) command: usize,
    pub(crate) document: crate::DocumentId,
    pub(crate) global: crate::DomHostedGlobalId,
    pub(crate) definition: crate::DomRegistryDefinitionId,
}

impl DomSourceCustomElementRequest {
    pub const fn operation(self) -> DomSourceOperationId {
        self.operation
    }

    pub const fn document(self) -> crate::DocumentId {
        self.document
    }

    pub const fn global(self) -> crate::DomHostedGlobalId {
        self.global
    }

    pub const fn definition(self) -> crate::DomRegistryDefinitionId {
        self.definition
    }
}

/// Observation of one exact Meta CSP insertion in a private Source segment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceMetaCspRequest {
    pub(crate) operation: DomSourceOperationId,
    pub(crate) occurrence: watcher::HookOccurrence,
    pub(crate) subject: Option<NodeId>,
    pub(crate) document: Option<crate::DocumentId>,
}

impl DomSourceMetaCspRequest {
    /// The original retained Source operation.
    pub const fn operation(self) -> DomSourceOperationId {
        self.operation
    }

    /// The Meta node, if its identity was already published.
    pub const fn subject(self) -> Option<NodeId> {
        self.subject
    }

    /// The Document, if its identity was already published.
    pub const fn document(self) -> Option<crate::DocumentId> {
        self.document
    }
}

/// Actual reached boundary of an original native HTML Script preparation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceScriptStep {
    /// Read scripting settings for the Script's own relevant Global.
    ReadScripting,
    /// The live answer enabled scripting. Executable preparation is unavailable.
    ExecutablePreparation,
}

/// Immutable request for one exact original preparation occurrence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceScriptRequest {
    pub(crate) operation: DomSourceOperationId,
    pub(crate) occurrence: watcher::HookOccurrence,
    pub(crate) basis: crate::DomStamp,
    pub(crate) subject: Option<NodeId>,
    pub(crate) node_document: Option<crate::DocumentId>,
    pub(crate) relevant_global: Option<crate::DomHostedGlobalId>,
    pub(crate) step: DomSourceScriptStep,
}

impl DomSourceScriptRequest {
    pub const fn operation(self) -> DomSourceOperationId {
        self.operation
    }
    pub const fn basis(self) -> crate::DomStamp {
        self.basis
    }
    /// A private staged creation has no published Node identity yet.
    pub const fn subject(self) -> Option<NodeId> {
        self.subject
    }
    pub const fn node_document(self) -> Option<crate::DocumentId> {
        self.node_document
    }
    /// Creation provenance. Adoption does not substitute the node Document's Global.
    pub const fn relevant_global(self) -> Option<crate::DomHostedGlobalId> {
        self.relevant_global
    }
    pub const fn step(self) -> DomSourceScriptStep {
        self.step
    }
}

/// Live settings sampled by the trusted issuer for one exact Script request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceScriptSettings {
    pub(crate) request: DomSourceScriptRequest,
    pub(crate) registration: crate::hosted::ScriptRegistrationKey,
    pub(crate) enabled: bool,
}

impl DomSourceScriptSettings {
    /// The exact original read for which the observation was established.
    pub const fn request(self) -> DomSourceScriptRequest {
        self.request
    }

    /// Whether the original relevant Window settings permit scripting.
    pub const fn enabled(self) -> bool {
        self.enabled
    }
}

/// The value returned by a completed Source operation with an output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceOutput {
    /// Whether the Element attribute is present after a qualified toggle.
    ElementAttributePresent(bool),
    /// Whether the named pseudo-attribute is present after a toggle.
    ProcessingInstructionAttributePresent(bool),
    Range(crate::DomRangeId),
    Node(NodeId),
    Document(crate::DocumentId),
    RemovedAttribute(crate::DomAttrId),
    AttributeAttachment(crate::DomAttributeAttachment),
    ViewTransition(DomViewTransitionId),
    ViewTransitionRecord(DomViewTransitionRecordId),
}

/// One edit to a ProcessingInstruction's ordered pseudo-attribute map.
#[derive(Clone, Debug)]
pub enum DomProcessingInstructionAttributeEdit {
    Set {
        name: DocumentString,
        value: DocumentString,
    },
    Remove {
        name: DocumentString,
    },
    Toggle {
        name: DocumentString,
        force: Option<bool>,
    },
}

/// A service boundary or the completion of one requested source operation.
#[expect(
    clippy::large_enum_variant,
    reason = "Move the owned completion inline without adding allocation after canonical publication"
)]
#[derive(Debug)]
pub enum DomSourceProgress {
    Request(DomSourceRequest),
    Complete(DomSourceCompletion),
    /// A named compound algorithm ended with its authentic partial publication.
    Failed(DomSourceFailure),
}
/// Terminal ordinary failure after a compound algorithm's accepted effects.
/// Service/admission refusals remain errors on the retained operation instead.
#[derive(Debug)]
pub struct DomSourceFailure {
    operation: DomSourceOperationId,
    error: DomError,
    commit: crate::DomCommit,
}
impl DomSourceFailure {
    /// The original named invocation that has now ended.
    pub const fn operation(&self) -> DomSourceOperationId {
        self.operation
    }
    /// Its first ordinary algorithm error, preserved through later refusals.
    pub const fn error(&self) -> &DomError {
        &self.error
    }
    /// Authentic final publication; earlier phases remain canonical too.
    pub const fn commit(&self) -> &crate::DomCommit {
        &self.commit
    }
    /// Move the invocation identity, original error and terminal commit together.
    pub fn into_parts(self) -> (DomSourceOperationId, DomError, crate::DomCommit) {
        (self.operation, self.error, self.commit)
    }
}
impl DomSourceProgress {
    pub(crate) fn terminal(completion: DomSourceCompletion, failure: Option<DomError>) -> Self {
        match failure {
            Some(error) => Self::Failed(DomSourceFailure {
                operation: completion.operation,
                error,
                commit: completion.commit,
            }),
            None => Self::Complete(completion),
        }
    }
}
/// Completed source-owned cleanup after an actual queued Dialog Toggle dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomDialogToggleTaskDisposition {
    /// The unconditional tracker-clear step completed on the surviving Dialog.
    TrackerCleared,
    /// The exact target retired during dispatch, so no tracker write occurred.
    TargetRetiredDuringDispatch,
}
#[derive(Debug)]
pub struct DomSourceCompletion {
    pub(crate) fullscreen_exit_return: Option<crate::DomFullscreenExitReturn>,
    pub(crate) operation: DomSourceOperationId,
    pub(crate) commit: crate::DomCommit,
    pub(crate) cancelled: bool,
    pub(crate) published: Vec<Option<NodeId>>,
    pub(crate) installation: Option<crate::DomNativeHtmlInstallation>,
    pub(crate) dialog_toggle_disposition: Option<DomDialogToggleTaskDisposition>,
    pub(crate) design_mode_range_disposition: Option<DomDesignModeRangeDisposition>,
    pub(crate) custom_element_disposition: Option<crate::DomSourceCustomElementDisposition>,
}
impl DomSourceCompletion {
    /// The original constructor result qualification, if this factory ran one.
    pub const fn custom_element_disposition(
        &self,
    ) -> Option<crate::DomSourceCustomElementDisposition> {
        self.custom_element_disposition
    }

    pub const fn fullscreen_exit_return(&self) -> Option<crate::DomFullscreenExitReturn> {
        self.fullscreen_exit_return
    }
    /// The completed reset branch of a false-to-on setter. `None` means off,
    /// invalid, already-on or another operation, never an unknown completed reset.
    pub const fn design_mode_range_disposition(&self) -> Option<DomDesignModeRangeDisposition> {
        self.design_mode_range_disposition
    }
    /// Completed post-dispatch Toggle cleanup. `None` means another operation
    /// or cancellation before dispatch; it never means an unknown serviced
    /// Toggle outcome. Cancellation requested after dispatch does not suppress
    /// the actual cleanup disposition.
    pub const fn dialog_toggle_disposition(&self) -> Option<DomDialogToggleTaskDisposition> {
        self.dialog_toggle_disposition
    }
    pub fn operation(&self) -> DomSourceOperationId {
        self.operation
    }
    pub fn native_installation(&self) -> Option<&crate::DomNativeHtmlInstallation> {
        self.installation.as_ref()
    }
    pub fn commit(&self) -> &crate::DomCommit {
        &self.commit
    }
    pub fn cancellation_requested(&self) -> bool {
        self.cancelled
    }
    /// Final output of this one operation. A pending construction exposes no
    /// completion and therefore cannot expose an unfinished Document result.
    pub fn output(&self) -> Option<DomSourceOutput> {
        self.commit.source_output()
    }
    /// Historical identity of the first node output, including a reused root.
    pub fn published_node(&self) -> Option<NodeId> {
        self.published.first().copied().flatten()
    }
}

/// Separate producer/DOM-source allowances composed by native installation.
#[derive(Clone, Copy, Debug)]
pub struct DomSourceNativeHtmlInstallLimits {
    pub(crate) native: crate::DomNativeHtmlInstallLimits,
    pub(crate) source: DomSourceLimits,
}
impl DomSourceNativeHtmlInstallLimits {
    pub const fn new(native: crate::DomNativeHtmlInstallLimits, source: DomSourceLimits) -> Self {
        Self { native, source }
    }
    pub const fn native(self) -> crate::DomNativeHtmlInstallLimits {
        self.native
    }
    pub const fn source(self) -> DomSourceLimits {
        self.source
    }
}

/// Exactly one requested canonical edit, retaining the original command.
/// There is no command-sequence container, batch-local operand or
/// conversion from an arbitrary batch. Published identities name every input.
#[derive(Debug)]
pub struct DomSourceEdit {
    pub(crate) command: crate::batch::Command,
}

/// Name input for one detached Element in its receiving Document.
#[derive(Clone, Debug)]
pub enum DomElementCreationName {
    /// Apply the document-sensitive whole-local-name rule.
    Local(DocumentString),
    /// Validate and split the supplied namespace and qualified name.
    Qualified(Namespace, DocumentString),
}

/// Converted options for Document element creation. DOM checks their
/// combination after validating the requested element name.
#[derive(Clone, Debug, Default)]
pub struct DomElementCreationOptions {
    /// An exact saved `is` value supplied by the dictionary.
    pub is: Option<DocumentString>,
    /// An explicit registry selection, including null, or the receiver's default.
    pub registry: Option<crate::DomRegistryBinding>,
}

/// One ordinary detached node creation with no connected-tree effects.
#[derive(Debug)]
pub enum DomDetachedNodeCreation {
    Attribute(DocumentString),
    AttributeNs(Namespace, DocumentString),
    Fragment,
    Text(DocumentString),
    Comment(DocumentString),
    ProcessingInstruction {
        target: DocumentString,
        data: DocumentString,
        fallback: crate::DomCreationRealmFallback,
    },
    DocumentType {
        name: DocumentString,
        public_id: DocumentString,
        system_id: DocumentString,
        fallback: crate::DomCreationRealmFallback,
    },
}

impl DomDetachedNodeCreation {
    pub(crate) fn into_edit(self, document: crate::DocumentId) -> DomSourceEdit {
        match self {
            Self::Attribute(name) => DomSourceEdit::create_attribute(document, name),
            Self::AttributeNs(namespace, name) => {
                DomSourceEdit::create_attribute_ns(document, namespace, name)
            }
            Self::Fragment => DomSourceEdit::create_fragment(document.node()),
            Self::Text(data) => DomSourceEdit::create_text(document.node(), data),
            Self::Comment(data) => DomSourceEdit::create_comment(document.node(), data),
            Self::ProcessingInstruction {
                target,
                data,
                fallback,
            } => DomSourceEdit::create(
                Creation::ProcessingInstruction {
                    target,
                    data,
                    fallback: Some(fallback),
                },
                Some(document.into()),
            ),
            Self::DocumentType {
                name,
                public_id,
                system_id,
                fallback,
            } => DomSourceEdit::create(
                Creation::DocumentType {
                    name,
                    public_id,
                    system_id,
                    fallback: Some(fallback),
                },
                Some(document.into()),
            ),
        }
    }
}

impl DomSourceEdit {
    /// Edit the native map and synchronize the instruction's CharacterData.
    /// Toggle returns the resulting presence through the Source completion.
    pub fn edit_processing_instruction_attribute(
        node: NodeId,
        edit: DomProcessingInstructionAttributeEdit,
    ) -> Self {
        Self {
            command: Command::ProcessingInstructionAttribute { node, edit },
        }
    }
    pub fn exit_fullscreen(document: crate::DocumentId) -> Self {
        Self {
            command: Command::ExitFullscreen {
                document: document.into(),
            },
        }
    }
    /// The native model operation, without requestFullscreen platform admission.
    pub fn fullscreen_element_model(node: NodeId) -> Self {
        Self {
            command: Command::FullscreenElementModel { node: node.into() },
        }
    }
    pub fn unfullscreen_element_model(node: NodeId) -> Self {
        Self {
            command: Command::UnfullscreenElementModel { node: node.into() },
        }
    }
    pub fn unfullscreen_document_model(document: crate::DocumentId) -> Self {
        Self {
            command: Command::UnfullscreenDocumentModel {
                document: document.into(),
            },
        }
    }
    /// Retains the original ordered setter across a reached unavailable service.
    pub fn set_design_mode(document: crate::DocumentId, value: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::SetDesignMode {
                document: document.into(),
                value: value.into(),
            },
        }
    }
    pub fn create_range(document: crate::DocumentId) -> Self {
        Self {
            command: Command::CreateRange {
                document: document.into(),
                ticket: 0,
            },
        }
    }
    pub fn set_range_start(range: crate::DomRangeId, container: NodeId, offset: usize) -> Self {
        Self {
            command: Command::SetRangeStart {
                range: range.into(),
                container: container.into(),
                offset,
            },
        }
    }
    pub fn set_range_end(range: crate::DomRangeId, container: NodeId, offset: usize) -> Self {
        Self {
            command: Command::SetRangeEnd {
                range: range.into(),
                container: container.into(),
                offset,
            },
        }
    }
    pub fn set_active_range(document: crate::DocumentId, range: Option<crate::DomRangeId>) -> Self {
        Self {
            command: Command::SetActiveRange {
                document: document.into(),
                range: range.map(Into::into),
            },
        }
    }
    pub fn retire_range(range: crate::DomRangeId) -> Self {
        Self {
            command: Command::RetireRange(range.into()),
        }
    }
    /// Invoke standard null-source ShowModal through its native Source prefix.
    pub fn show_modal_dialog(node: NodeId) -> Self {
        Self {
            command: Command::ShowModalDialog { node: node.into() },
        }
    }
    /// Request to close this Dialog with the standard null source.
    pub fn request_close_dialog(node: NodeId, result: Option<DocumentString>) -> Self {
        Self {
            command: Command::RequestCloseDialog {
                node: node.into(),
                result,
            },
        }
    }

    /// Invoke the standard nonmodal close method with its null source. Omitted
    /// result preserves returnValue; supplied empty is a real empty value.
    pub fn close_dialog(node: NodeId, result: Option<DocumentString>) -> Self {
        Self {
            command: Command::CloseDialog {
                node: node.into(),
                result,
            },
        }
    }

    /// Invoke nonmodal Dialog show through the original Source engine.
    pub fn show_dialog(node: NodeId) -> Self {
        Self {
            command: Command::ShowDialog { node: node.into() },
        }
    }

    fn create(kind: crate::batch::Creation, document: Option<crate::NodeOperand>) -> Self {
        Self {
            command: crate::batch::Command::Create { kind, document },
        }
    }
    /// Request one `bind_encoding_url_facts` operation on the canonical owner.
    pub fn bind_encoding_url_facts(
        basis: DomDocumentStamp,
        facts: &surgeist_html_semantics::HtmlEncodingUrlParseFacts,
    ) -> Self {
        Self {
            command: Command::BindEncodingUrlFacts {
                basis,
                facts: facts.clone(),
            },
        }
    }
    /// Request the same current-Document and current-policy attestation as
    /// [`crate::DomBatch::bind_base_facts`]. Imported history is not automatically
    /// qualified to a later associated CSP policy.
    pub fn bind_base_facts(
        basis: DomDocumentStamp,
        facts: &surgeist_html_semantics::HtmlParseBaseFacts,
    ) -> Self {
        Self {
            command: Command::BindBaseFacts {
                basis,
                facts: facts.clone(),
            },
        }
    }
    /// Request one `initialize_view_transitions` operation on the canonical owner.
    pub fn initialize_view_transitions(
        basis: DomDocumentStamp,
        input: DomPreparedViewTransitionSheet,
    ) -> Self {
        Self {
            command: Command::ViewTransition {
                basis,
                edit: Edit::Initialize { input },
            },
        }
    }
    /// Request one `begin_view_transition` operation on the canonical owner.
    pub fn begin_view_transition(basis: DomDocumentStamp) -> Self {
        Self {
            command: Command::ViewTransition {
                basis,
                edit: Edit::Begin { ticket: 0 },
            },
        }
    }
    /// Request one `append_view_transition_record` operation on the canonical owner.
    pub fn append_view_transition_record(
        basis: DomDocumentStamp,
        transition: DomViewTransitionId,
        name: CssViewTransitionIdent,
    ) -> Self {
        Self {
            command: Command::ViewTransition {
                basis,
                edit: Edit::Append {
                    transition: transition.into(),
                    name: Arc::new(name),
                    ticket: 0,
                },
            },
        }
    }
    /// Request one `set_view_transition_branches` operation on the canonical owner.
    pub fn set_view_transition_branches(
        basis: DomDocumentStamp,
        record: DomViewTransitionRecordId,
        branches: DomViewTransitionBranches,
    ) -> Self {
        Self {
            command: Command::ViewTransition {
                basis,
                edit: Edit::Branches {
                    record: record.into(),
                    branches,
                },
            },
        }
    }
    /// Request one `set_view_transition_rules` operation on the canonical owner.
    pub fn set_view_transition_rules(
        basis: DomDocumentStamp,
        record: DomViewTransitionRecordId,
        input: DomPreparedViewTransitionRules,
    ) -> Self {
        Self {
            command: Command::ViewTransition {
                basis,
                edit: Edit::Rules {
                    record: record.into(),
                    input,
                },
            },
        }
    }
    /// Request one `clear_view_transition_rules` operation on the canonical owner.
    pub fn clear_view_transition_rules(
        basis: DomDocumentStamp,
        record: DomViewTransitionRecordId,
    ) -> Self {
        Self {
            command: Command::ViewTransition {
                basis,
                edit: Edit::ClearRules(record.into()),
            },
        }
    }
    /// Request one `set_view_transition_captured` operation on the canonical owner.
    pub fn set_view_transition_captured(
        basis: DomDocumentStamp,
        subject: DomViewTransitionSubject,
        captured: bool,
    ) -> Self {
        Self {
            command: Command::ViewTransition {
                basis,
                edit: Edit::Captured { subject, captured },
            },
        }
    }
    /// Request one `set_view_transition_rendering_suppression` operation on the canonical owner.
    pub fn set_view_transition_rendering_suppression(
        basis: DomDocumentStamp,
        suppressed: bool,
    ) -> Self {
        Self {
            command: Command::ViewTransition {
                basis,
                edit: Edit::Suppression(suppressed),
            },
        }
    }
    /// Request one `setup_view_transition_pseudos` operation on the canonical owner.
    pub fn setup_view_transition_pseudos(
        basis: DomDocumentStamp,
        transition: DomViewTransitionId,
    ) -> Self {
        Self {
            command: Command::ViewTransition {
                basis,
                edit: Edit::Setup(transition.into()),
            },
        }
    }
    /// Request one `clear_view_transition_state` operation on the canonical owner.
    pub fn clear_view_transition_state(
        basis: DomDocumentStamp,
        transition: DomViewTransitionId,
    ) -> Self {
        Self {
            command: Command::ViewTransition {
                basis,
                edit: Edit::Clear(transition.into()),
            },
        }
    }
    /// Request one `release_view_transition` operation on the canonical owner.
    pub fn release_view_transition(
        basis: DomDocumentStamp,
        transition: DomViewTransitionId,
    ) -> Self {
        Self {
            command: Command::ViewTransition {
                basis,
                edit: Edit::Release(transition.into()),
            },
        }
    }
    /// Request one `import_node` operation on the canonical owner.
    pub fn import_node(
        snapshot: &DomSnapshot,
        source: NodeId,
        target_document: NodeId,
        subtree: bool,
    ) -> Self {
        Self {
            command: Command::Import {
                snapshot: snapshot.clone(),
                source,
                document: target_document.into(),
                subtree,
            },
        }
    }
    /// Execute one authenticated standard Document constructor, retaining its
    /// original context through admission refusal without caller replay.
    pub fn construct_document(context: crate::DomDocumentConstructorContext) -> Self {
        Self {
            command: Command::ConstructDocument(context.data),
        }
    }
    /// Clone one live canonical node, including a Document, under one original
    /// Source operation. Mandatory descendant hooks retain their exact cursor.
    pub fn clone_node(source: NodeId, subtree: bool) -> Self {
        Self {
            command: Command::CloneNode {
                source: source.into(),
                subtree,
                fallback: None,
            },
        }
    }
    /// Clone for an authenticated scripting attachment, supplying its Global
    /// when the source node document has no native creation Global.
    pub fn clone_node_in_attachment(
        source: NodeId,
        subtree: bool,
        fallback: crate::DomCreationRealmFallback,
    ) -> Self {
        Self {
            command: Command::CloneNode {
                source: source.into(),
                subtree,
                fallback: Some(fallback),
            },
        }
    }
    /// Import the live node into the exact destination with the selected
    /// registry and authenticated attachment fallback.
    pub fn import_live_node(
        source: NodeId,
        document: crate::DocumentId,
        subtree: bool,
        registry: Option<crate::DomRegistryId>,
        fallback: crate::DomCreationRealmFallback,
    ) -> Self {
        Self {
            command: Command::ImportLiveNode {
                source,
                document,
                subtree,
                registry,
                fallback,
            },
        }
    }
    /// Normalize descendant Text through one retained source invocation, including
    /// each real append/removal hook and the special live Range transfers.
    pub fn normalize(node: NodeId) -> Self {
        Self {
            command: Command::Normalize(node.into()),
        }
    }
    /// Split Text at an exact UTF-16 offset under one original Source invocation.
    /// Completion returns the actual new Text through `DomSourceOutput::Node`.
    pub fn split_text(node: NodeId, offset: usize) -> Self {
        Self {
            command: Command::SplitText {
                node: node.into(),
                offset,
            },
        }
    }
    /// Request one `create_document` operation on the canonical owner.
    pub fn create_document() -> Self {
        Self::create(Creation::Document, None)
    }
    /// Request one `create_xml_document` operation on the canonical owner.
    pub fn create_xml_document(
        associated_document: NodeId,
        namespace: Namespace,
        qualified_name: impl Into<DocumentString>,
        doctype: Option<NodeId>,
    ) -> Self {
        Self::xml_document(
            associated_document,
            namespace,
            qualified_name.into(),
            doctype,
            None,
        )
    }
    /// Create through DOMImplementation with the receiver's established guest realm.
    pub fn create_xml_document_in_attachment(
        associated_document: crate::DocumentId,
        namespace: Namespace,
        qualified_name: DocumentString,
        doctype: Option<NodeId>,
        fallback: crate::DomCreationRealmFallback,
    ) -> Self {
        Self::xml_document(
            associated_document.node(),
            namespace,
            qualified_name,
            doctype,
            Some(fallback),
        )
    }
    fn xml_document(
        associated_document: NodeId,
        namespace: Namespace,
        qualified_name: DocumentString,
        doctype: Option<NodeId>,
        fallback: Option<crate::DomCreationRealmFallback>,
    ) -> Self {
        Self {
            command: Command::CreateXmlDocument {
                associated_document: associated_document.into(),
                namespace,
                qualified_name,
                doctype: doctype.map(crate::NodeOperand::Existing),
                fallback,
            },
        }
    }
    /// Request one `create_html_document` operation on the canonical owner.
    pub fn create_html_document(
        associated_document: NodeId,
        title: Option<DocumentString>,
    ) -> Self {
        Self::html_document(associated_document, title, None)
    }
    /// Create through DOMImplementation with the receiver's established guest realm.
    pub fn create_html_document_in_attachment(
        associated_document: crate::DocumentId,
        title: Option<DocumentString>,
        fallback: crate::DomCreationRealmFallback,
    ) -> Self {
        Self::html_document(associated_document.node(), title, Some(fallback))
    }
    fn html_document(
        associated_document: NodeId,
        title: Option<DocumentString>,
        fallback: Option<crate::DomCreationRealmFallback>,
    ) -> Self {
        Self {
            command: Command::CreateHtmlDocument {
                associated_document: associated_document.into(),
                title,
                fallback,
            },
        }
    }
    /// Request one `attach_shadow` operation on the canonical owner.
    pub fn attach_shadow(host: NodeId, init: impl Into<crate::DomShadowRootInit>) -> Self {
        Self {
            command: Command::AttachShadow {
                host: host.into(),
                init: init.into(),
            },
        }
    }
    /// Associate null registry slots in an ordinary subtree and enqueue the
    /// eligible upgrades in tree order.
    pub fn initialize_custom_element_registry(
        root: NodeId,
        registry: crate::DomRegistryId,
    ) -> Self {
        Self {
            command: Command::InitializeRegistry {
                root: root.into(),
                registry,
            },
        }
    }
    /// Enqueue eligible upgrades for this registry in shadow-including order.
    pub fn upgrade_custom_element_registry(root: NodeId, registry: crate::DomRegistryId) -> Self {
        Self {
            command: Command::UpgradeRegistry {
                root: root.into(),
                registry,
            },
        }
    }
    /// Request one `create_fragment` operation on the canonical owner.
    pub fn create_fragment(document: NodeId) -> Self {
        Self::create(Creation::Fragment, Some(document.into()))
    }
    /// Request one `create_element` operation on the canonical owner.
    pub fn create_element(document: NodeId, local_name: impl Into<DocumentString>) -> Self {
        Self::create_element_with_options(
            document,
            DomElementCreationName::Local(local_name.into()),
            DomElementCreationOptions::default(),
        )
    }
    /// Request one `create_element_ns` operation on the canonical owner.
    pub fn create_element_ns(
        document: NodeId,
        namespace: Namespace,
        qualified_name: impl Into<DocumentString>,
    ) -> Self {
        Self::create_element_with_options(
            document,
            DomElementCreationName::Qualified(namespace, qualified_name.into()),
            DomElementCreationOptions::default(),
        )
    }
    pub(crate) fn create_element_with_options(
        document: NodeId,
        name: DomElementCreationName,
        options: DomElementCreationOptions,
    ) -> Self {
        Self::create_element_in_realm(document, name, options, None)
    }
    pub(crate) fn create_element_in_realm(
        document: NodeId,
        name: DomElementCreationName,
        options: DomElementCreationOptions,
        fallback: Option<crate::DomCreationRealmFallback>,
    ) -> Self {
        Self::create(
            Creation::Element {
                name,
                options,
                fallback,
            },
            Some(document.into()),
        )
    }
    /// Request creation of one empty detached Attr in the selected Document.
    pub fn create_attribute(
        document: crate::DocumentId,
        local_name: impl Into<DocumentString>,
    ) -> Self {
        Self::create(
            Creation::Attr {
                namespace: None,
                name: local_name.into(),
            },
            Some(document.into()),
        )
    }
    /// Request creation of one empty detached Attr with an exact qualified name.
    pub fn create_attribute_ns(
        document: crate::DocumentId,
        namespace: Namespace,
        qualified_name: impl Into<DocumentString>,
    ) -> Self {
        Self::create(
            Creation::Attr {
                namespace: Some(namespace),
                name: qualified_name.into(),
            },
            Some(document.into()),
        )
    }
    /// Request one `set_command_for_element` operation on the canonical owner.
    pub fn set_command_for_element(node: NodeId, target: Option<NodeId>) -> Self {
        let target = target.map(crate::NodeOperand::Existing);
        Self {
            command: Command::SetReflectedTarget {
                node: node.into(),
                key: crate::reflected::ReflectedTarget::CommandFor,
                target,
            },
        }
    }
    /// Request one `set_popover_target_element` operation on the canonical owner.
    pub fn set_popover_target_element(node: NodeId, target: Option<NodeId>) -> Self {
        let target = target.map(crate::NodeOperand::Existing);
        Self {
            command: Command::SetReflectedTarget {
                node: node.into(),
                key: crate::reflected::ReflectedTarget::PopoverTarget,
                target,
            },
        }
    }
    /// Request one `set_input_value` operation on the canonical owner.
    pub fn set_input_value(node: NodeId, value: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::SetInputValue {
                node: node.into(),
                value: value.into(),
            },
        }
    }
    /// Request one `set_input_checked` operation on the canonical owner.
    pub fn set_input_checked(node: NodeId, value: bool) -> Self {
        Self {
            command: Command::SetInputChecked {
                node: node.into(),
                value,
            },
        }
    }
    /// Request one `set_input_indeterminate` operation on the canonical owner.
    pub fn set_input_indeterminate(node: NodeId, value: bool) -> Self {
        Self {
            command: Command::SetInputIndeterminate {
                node: node.into(),
                value,
            },
        }
    }
    /// Request one `reset_input` operation on the canonical owner.
    pub fn reset_input(node: NodeId) -> Self {
        Self {
            command: Command::ResetInput { node: node.into() },
        }
    }
    /// Request one `set_style_disabled` operation on the canonical owner.
    pub fn set_style_disabled(node: NodeId, disabled: bool) -> Self {
        Self {
            command: Command::SetStyleDisabled {
                node: node.into(),
                disabled,
            },
        }
    }
    /// Request one `set_output_value` operation on the canonical owner.
    pub fn set_output_value(node: NodeId, value: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::Output {
                node: node.into(),
                edit: crate::output::Edit::Value(value.into()),
            },
        }
    }
    /// Request one `set_output_default_value` operation on the canonical owner.
    pub fn set_output_default_value(node: NodeId, value: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::Output {
                node: node.into(),
                edit: crate::output::Edit::DefaultValue(value.into()),
            },
        }
    }
    /// Request one `reset_output` operation on the canonical owner.
    pub fn reset_output(node: NodeId) -> Self {
        Self {
            command: Command::Output {
                node: node.into(),
                edit: crate::output::Edit::Reset,
            },
        }
    }
    /// Request one `set_textarea_value` operation on the canonical owner.
    pub fn set_textarea_value(node: NodeId, value: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::SetTextareaValue {
                node: node.into(),
                value: value.into(),
            },
        }
    }
    /// Request one `set_textarea_default_value` operation on the canonical owner.
    pub fn set_textarea_default_value(node: NodeId, value: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::SetTextareaDefaultValue {
                node: node.into(),
                value: value.into(),
            },
        }
    }
    /// Request one `reset_textarea` operation on the canonical owner.
    pub fn reset_textarea(node: NodeId) -> Self {
        Self {
            command: Command::ResetTextarea { node: node.into() },
        }
    }
    /// Request one `set_custom_validity` operation on the canonical owner.
    pub fn set_custom_validity(node: NodeId, value: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::SetCustomValidity {
                node: node.into(),
                value: value.into(),
            },
        }
    }
    /// Request one `set_nonce` operation on the canonical owner.
    pub fn set_nonce(node: NodeId, value: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::SetNonce {
                node: node.into(),
                value: value.into(),
            },
        }
    }
    /// Request one `assign_slot` operation on the canonical owner.
    pub fn assign_slot(slot: NodeId, incoming: Vec<NodeId>) -> Self {
        let incoming = incoming
            .into_iter()
            .map(crate::NodeOperand::Existing)
            .collect();
        Self {
            command: Command::AssignSlot {
                slot: slot.into(),
                incoming,
            },
        }
    }
    /// Request one `set_dialog_return_value` operation on the canonical owner.
    pub fn set_dialog_return_value(node: NodeId, value: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::SetDialogReturnValue {
                node: node.into(),
                value: value.into(),
            },
        }
    }
    /// Request one `start_marquee` operation on the canonical owner.
    pub fn start_marquee(node: NodeId) -> Self {
        Self {
            command: Command::SetMarqueeRunning {
                node: node.into(),
                running: true,
            },
        }
    }
    /// Request one `stop_marquee` operation on the canonical owner.
    pub fn stop_marquee(node: NodeId) -> Self {
        Self {
            command: Command::SetMarqueeRunning {
                node: node.into(),
                running: false,
            },
        }
    }
    /// Request one `create_text` operation on the canonical owner.
    pub fn create_text(document: NodeId, data: impl Into<DocumentString>) -> Self {
        Self::create(Creation::Text(data.into()), Some(document.into()))
    }
    /// Request one `create_comment` operation on the canonical owner.
    pub fn create_comment(document: NodeId, data: impl Into<DocumentString>) -> Self {
        Self::create(Creation::Comment(data.into()), Some(document.into()))
    }
    /// Request one `create_processing_instruction` operation on the canonical owner.
    pub fn create_processing_instruction(
        document: NodeId,
        target: impl Into<DocumentString>,
        data: impl Into<DocumentString>,
    ) -> Self {
        Self::create(
            Creation::ProcessingInstruction {
                target: target.into(),
                data: data.into(),
                fallback: None,
            },
            Some(document.into()),
        )
    }
    /// Request one `create_document_type` operation on the canonical owner.
    pub fn create_document_type(
        document: NodeId,
        name: impl Into<DocumentString>,
        public_id: impl Into<DocumentString>,
        system_id: impl Into<DocumentString>,
    ) -> Self {
        Self::create(
            Creation::DocumentType {
                name: name.into(),
                public_id: public_id.into(),
                system_id: system_id.into(),
                fallback: None,
            },
            Some(document.into()),
        )
    }
    /// Request one `insert_before` operation on the canonical owner.
    pub fn insert_before(node: NodeId, parent: NodeId, before: Option<NodeId>) -> Self {
        let before = before.map(crate::NodeOperand::Existing);
        Self {
            command: Command::Insert {
                node: node.into(),
                parent: parent.into(),
                before,
            },
        }
    }
    /// Move an existing node within its shadow-including root, preserving its
    /// identity and running the dedicated native moving steps.
    pub fn move_before(node: NodeId, parent: NodeId, before: Option<NodeId>) -> Self {
        Self {
            command: Command::MoveBefore {
                node: node.into(),
                parent: parent.into(),
                before: before.map(crate::NodeOperand::Existing),
            },
        }
    }
    /// Request one `detach` operation on the canonical owner.
    pub fn detach(node: NodeId) -> Self {
        Self {
            command: Command::Detach(node.into()),
        }
    }
    /// Remove the exact ordinary child of the selected parent. A live node
    /// without that parent fails with `DomError::NotFound`.
    pub fn remove_child(parent: NodeId, child: NodeId) -> Self {
        Self {
            command: Command::RemoveChild {
                parent: parent.into(),
                child: child.into(),
            },
        }
    }
    /// Request one `replace_child` operation on the canonical owner.
    pub fn replace_child(node: NodeId, parent: NodeId, child: NodeId) -> Self {
        Self {
            command: Command::ReplaceChild {
                node: node.into(),
                parent: parent.into(),
                child: child.into(),
            },
        }
    }
    /// Request one `adopt` operation on the canonical owner.
    pub fn adopt(node: NodeId, document: NodeId) -> Self {
        Self {
            command: Command::Adopt {
                node: node.into(),
                document: document.into(),
            },
        }
    }
    /// Request the standard qualified attribute setter, including verification
    /// of values for reached Trusted Types sinks.
    pub fn set_attribute(
        node: NodeId,
        name: impl Into<DocumentString>,
        value: impl Into<DocumentString>,
    ) -> Self {
        Self::attribute_setter(node, None, name, value, AttributeSetterMode::Standard)
    }
    /// Request the standard namespace-aware attribute setter, including
    /// verification of values for reached Trusted Types sinks.
    pub fn set_attribute_ns(
        node: NodeId,
        namespace: Namespace,
        qualified_name: impl Into<DocumentString>,
        value: impl Into<DocumentString>,
    ) -> Self {
        Self::attribute_setter(
            node,
            Some(namespace),
            qualified_name,
            value,
            AttributeSetterMode::Standard,
        )
    }
    /// Write a qualified attribute as part of native canonical authoring.
    pub fn set_native_attribute(
        node: NodeId,
        name: impl Into<DocumentString>,
        value: impl Into<DocumentString>,
    ) -> Self {
        Self::attribute_setter(node, None, name, value, AttributeSetterMode::Native)
    }
    /// Write an expanded attribute as part of native canonical authoring.
    pub fn set_native_attribute_ns(
        node: NodeId,
        namespace: Namespace,
        qualified_name: impl Into<DocumentString>,
        value: impl Into<DocumentString>,
    ) -> Self {
        Self::attribute_setter(
            node,
            Some(namespace),
            qualified_name,
            value,
            AttributeSetterMode::Native,
        )
    }
    fn attribute_setter(
        node: NodeId,
        namespace: Option<Namespace>,
        name: impl Into<DocumentString>,
        value: impl Into<DocumentString>,
        mode: AttributeSetterMode,
    ) -> Self {
        Self {
            command: Command::SetAttribute {
                node: node.into(),
                namespace,
                name: name.into(),
                value: value.into(),
                mode,
            },
        }
    }
    /// Request one `remove_attribute` operation on the canonical owner.
    pub fn remove_attribute(node: NodeId, qualified_name: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::RemoveAttribute {
                node: node.into(),
                namespace: None,
                name: qualified_name.into(),
                return_identity: false,
            },
        }
    }
    /// Validate and toggle the first matching qualified attribute as one edit.
    /// An absent force reverses presence. An explicit force sets presence;
    /// when it already matches, the operation only returns the Boolean output.
    pub fn toggle_attribute(
        node: NodeId,
        qualified_name: impl Into<DocumentString>,
        force: Option<bool>,
    ) -> Self {
        Self {
            command: Command::ToggleAttribute {
                node: node.into(),
                name: qualified_name.into(),
                force,
            },
        }
    }
    /// Request one `remove_attribute_ns` operation on the canonical owner.
    pub fn remove_attribute_ns(
        node: NodeId,
        namespace: Namespace,
        local_name: impl Into<DocumentString>,
    ) -> Self {
        Self {
            command: Command::RemoveAttribute {
                node: node.into(),
                namespace: Some(namespace),
                name: local_name.into(),
                return_identity: false,
            },
        }
    }
    /// Request removal of the exact currently attached Attr.
    pub fn remove_attribute_node(node: NodeId, attribute: crate::DomAttrId) -> Self {
        Self {
            command: Command::RemoveAttributeNode {
                node: node.into(),
                attribute,
            },
        }
    }
    /// Set an existing Attr's value with standard attribute verification.
    pub fn set_attribute_value(
        attribute: crate::DomAttrId,
        value: impl Into<DocumentString>,
    ) -> Self {
        Self {
            command: Command::SetAttributeValue {
                attribute,
                value: value.into(),
            },
        }
    }
    /// Attach an Attr by expanded name with standard attribute verification.
    /// Element and NamedNodeMap attachment use this same operation.
    pub fn attach_attribute(node: NodeId, attribute: crate::DomAttrId) -> Self {
        Self::attribute_attachment(node, attribute, AttributeSetterMode::Standard)
    }
    /// Require the attached guest's output custody before replacing an Attr.
    pub fn attach_attribute_for_script(
        node: NodeId,
        attribute: crate::DomAttrId,
        registration: &crate::DomDocumentScriptRegistration,
    ) -> Self {
        Self {
            command: Command::AttachAttribute {
                node: node.into(),
                attribute,
                mode: AttributeSetterMode::Standard,
                output_registration: Some(registration.0.clone()),
            },
        }
    }
    /// Attach an already authored native Attr and return the canonical result.
    pub fn attach_native_attribute(node: NodeId, attribute: crate::DomAttrId) -> Self {
        Self::attribute_attachment(node, attribute, AttributeSetterMode::Native)
    }
    fn attribute_attachment(
        node: NodeId,
        attribute: crate::DomAttrId,
        mode: AttributeSetterMode,
    ) -> Self {
        Self {
            command: Command::AttachAttribute {
                node: node.into(),
                attribute,
                mode,
                output_registration: None,
            },
        }
    }
    /// Remove the current NamedNodeMap item and return its detached identity.
    pub fn remove_named_item(node: NodeId, qualified_name: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::RemoveAttribute {
                node: node.into(),
                namespace: None,
                name: qualified_name.into(),
                return_identity: true,
            },
        }
    }
    /// Remove the current NamedNodeMap item by expanded name.
    pub fn remove_named_item_ns(
        node: NodeId,
        namespace: Namespace,
        local_name: impl Into<DocumentString>,
    ) -> Self {
        Self {
            command: Command::RemoveAttribute {
                node: node.into(),
                namespace: Some(namespace),
                name: local_name.into(),
                return_identity: true,
            },
        }
    }
    /// Request one `destroy` operation on the canonical owner.
    pub fn destroy(node: NodeId) -> Self {
        Self {
            command: Command::Destroy(node.into()),
        }
    }
    /// Request one `replace_data` operation on the canonical owner.
    pub fn replace_data(
        node: NodeId,
        offset: usize,
        count: usize,
        data: impl Into<DocumentString>,
    ) -> Self {
        Self {
            command: Command::ReplaceData {
                node: node.into(),
                offset,
                count,
                data: data.into(),
            },
        }
    }
    /// Request one `set_select_value` operation on the canonical owner.
    pub fn set_select_value(node: NodeId, value: impl Into<DocumentString>) -> Self {
        Self {
            command: Command::Select {
                node: node.into(),
                edit: crate::select::Edit::Value(value.into()),
            },
        }
    }
    /// Request one `set_select_selected_index` operation on the canonical owner.
    pub fn set_select_selected_index(node: NodeId, index: i64) -> Self {
        Self {
            command: Command::Select {
                node: node.into(),
                edit: crate::select::Edit::Index(index),
            },
        }
    }
    /// Request one `reset_select` operation on the canonical owner.
    pub fn reset_select(node: NodeId) -> Self {
        Self {
            command: Command::Select {
                node: node.into(),
                edit: crate::select::Edit::Reset,
            },
        }
    }
    /// Request one `set_option_selected` operation on the canonical owner.
    pub fn set_option_selected(node: NodeId, selected: bool) -> Self {
        Self {
            command: Command::Select {
                node: node.into(),
                edit: crate::select::Edit::OptionSelected(selected),
            },
        }
    }
}
