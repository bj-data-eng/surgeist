//! One requested canonical edit and its retained source continuation.
pub(crate) mod event;
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
use crate::batch::{Command, Creation};
use crate::view_transition::*;
use crate::{DocumentString, DomDocumentStamp, DomSnapshot, DomStyleRequest, Namespace, NodeId};
pub use event::*;
use std::sync::Arc;
use surgeist_css::CssViewTransitionIdent;
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
    /// Coherent current-state observation awaiting actual event dispatch.
    EventObservation,
    /// Qualified event outcome retained before current-state rebase.
    EventReceiptAccepted,
    /// A genuine Dialog action waiting for its qualified external service.
    DialogService,
    /// A genuine common insertion action awaiting its qualified service.
    AutofocusInsertionService,
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
    Style(DomStyleRequest),
    HostedChild(crate::DomHostedChildRequest),
    Event(DomSourceEventRequest),
    Dialog(DomSourceDialogRequest),
    DialogFocusing(DomSourceDialogFocusingRequest),
    FocusDelegate(DomSourceFocusDelegateRequest),
    AutofocusInsertion(DomSourceAutofocusInsertionRequest),
    CloseWatcher(DomSourceCloseWatcherRequest),
}
/// The final identity produced by one completed creation operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceOutput {
    Range(crate::DomRangeId),
    Node(NodeId),
    Document(crate::DocumentId),
    ViewTransition(DomViewTransitionId),
    ViewTransitionRecord(DomViewTransitionRecordId),
}
/// A service boundary or the completion of one requested source operation.
#[derive(Debug)]
pub enum DomSourceProgress {
    Request(DomSourceRequest),
    Complete(DomSourceCompletion),
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
    pub(crate) operation: DomSourceOperationId,
    pub(crate) commit: crate::DomCommit,
    pub(crate) cancelled: bool,
    pub(crate) published: Vec<Option<NodeId>>,
    pub(crate) installation: Option<crate::DomNativeHtmlInstallation>,
    pub(crate) dialog_toggle_disposition: Option<DomDialogToggleTaskDisposition>,
}
impl DomSourceCompletion {
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
    /// Historical first publication of this operation's root creation.
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
/// There is no sequence container, append operation, batch-local operand or
/// conversion from an arbitrary batch. Published identities name every input.
#[derive(Debug)]
pub struct DomSourceEdit {
    pub(crate) command: crate::batch::Command,
}
impl DomSourceEdit {
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
    /// Request one `bind_base_facts` operation on the canonical owner.
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
        let doctype = doctype.map(crate::NodeOperand::Existing);
        Self {
            command: Command::CreateXmlDocument {
                associated_document: associated_document.into(),
                namespace,
                qualified_name: qualified_name.into(),
                doctype,
            },
        }
    }
    /// Request one `create_html_document` operation on the canonical owner.
    pub fn create_html_document(
        associated_document: NodeId,
        title: Option<DocumentString>,
    ) -> Self {
        Self {
            command: Command::CreateHtmlDocument {
                associated_document: associated_document.into(),
                title,
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
    /// Request one `create_fragment` operation on the canonical owner.
    pub fn create_fragment(document: NodeId) -> Self {
        Self::create(Creation::Fragment, Some(document.into()))
    }
    /// Request one `create_element` operation on the canonical owner.
    pub fn create_element(document: NodeId, local_name: impl Into<DocumentString>) -> Self {
        Self::create(Creation::Element(local_name.into()), Some(document.into()))
    }
    /// Request one `create_element_ns` operation on the canonical owner.
    pub fn create_element_ns(
        document: NodeId,
        namespace: Namespace,
        qualified_name: impl Into<DocumentString>,
    ) -> Self {
        Self::create(
            Creation::ElementNs {
                namespace,
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
    /// Request one `detach` operation on the canonical owner.
    pub fn detach(node: NodeId) -> Self {
        Self {
            command: Command::Detach(node.into()),
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
    /// Request one `set_attribute` operation on the canonical owner.
    pub fn set_attribute(
        node: NodeId,
        name: impl Into<DocumentString>,
        value: impl Into<DocumentString>,
    ) -> Self {
        Self {
            command: Command::SetAttribute {
                node: node.into(),
                namespace: None,
                name: name.into(),
                value: value.into(),
            },
        }
    }
    /// Request one `set_attribute_ns` operation on the canonical owner.
    pub fn set_attribute_ns(
        node: NodeId,
        namespace: Namespace,
        qualified_name: impl Into<DocumentString>,
        value: impl Into<DocumentString>,
    ) -> Self {
        Self {
            command: Command::SetAttribute {
                node: node.into(),
                namespace: Some(namespace),
                name: qualified_name.into(),
                value: value.into(),
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
