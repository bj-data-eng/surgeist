use std::{
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
};

use crate::batch::{BatchIdentity, Command, Creation};
use crate::document::{DomDocumentMetadata, NATIVE_DOCUMENT_STRING_UNITS};
use crate::{
    CreatedNode, DocumentId, DocumentString, DomBatch, DomDocumentType, DomElement, DomNodeKind,
    DomOwnerId, DomProcessingInstruction, DomRevision, DomSnapshot, DomStamp, NodeId, NodeOperand,
};

mod attributes;
mod autofocus;
mod bank;
mod base;
pub(crate) mod canonical;
mod children;
mod control;
mod custody;
mod design_mode;
mod details;
mod dialog;
mod dispatch;
mod document;
mod element_tasks;
mod focus;
mod form;
mod fullscreen;
mod history;
pub(crate) mod hosted;
mod hyperlink;
use history::HistoryEntry;
mod image;
mod import;
mod input;
mod input_processing;
mod installation;
mod meta;
mod method;
mod navigable;
#[cfg(test)]
mod oracle_tests;
mod output;
pub(crate) mod parsed_installation;
mod range;
mod reflected;
pub(crate) mod registry;
#[cfg(test)]
pub(crate) mod scale_metrics;
#[cfg(test)]
mod scale_tests;
mod script;
mod select;
mod shadow;
mod shared;
mod slot;
pub(crate) mod source;
pub(crate) mod style;
mod style_custody;
#[cfg(test)]
mod style_custody_tests;
mod svg_script;
mod template;
mod textarea;
mod transaction;
#[cfg(test)]
pub(crate) use transaction::CopyObservation;
mod tree;
mod view_transition;
use crate::view_transition::*;
pub(crate) use bank::owned::{EmptyPath, OwnedBank};
pub(crate) use bank::{Bank, Iter as BankIter};
pub(crate) use shared::SharedVec;
pub(crate) use source::AllocationIds as KeyAliases;
pub(crate) use source::formation::{SourceRetention, SourceStorage, SourceWorkIdentity};
use view_transition::DraftTransitions;
pub(crate) use view_transition::TransitionMembership;

/// Configured capacity whose admission would be exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomResource {
    FullscreenExits,
    FullscreenEvents,
    Ranges,
    ManagedNodes,
    SlotReferences,
    AutofocusCandidates,
    SlotAssignmentInputs,
    SlotQueryOutput,
    SlotQueryScratch,
    StringCodeUnits,
    Commands,
    HistoryEntries,
    HistoryBytes,
    ViewTransitions,
    ViewTransitionRecords,
    ViewTransitionPseudos,
    ViewTransitionCaptureBytes,
    InputProcessingScratch,
    SelectScratch,
    SelectTasks,
    DetailsToggleIntentions,
    DetailsToggleOutput,
    BaseObservations,
    ImageUpdateCancellations,
    EncodingUrlObservations,
    /// Registered dispatch-custody index capacity.
    DispatchCustodyScopes,
    /// Actual simultaneously retained target references.
    DispatchCustodyReferences,
    /// Actual custody scope and weak-index payload.
    DispatchCustodyRetainedBytes,
    /// Temporary registration and retirement-publication banks.
    DispatchCustodyScratch,
    SourceRetainedBytes,
    SourceRetainedUnits,
    MetaColorSchemeRetainedBytes,
    MetaThemeColorScratch,
}

/// Typed edit/readback failure. Atomic batch failures publish no state or
/// identity; composed source failures retain their already-published phases
/// and mandatory continuation for recovery.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DomError {
    StaleFullscreenExit,
    FullscreenExitHandoffRequired,
    FullscreenExitAssertionFailed {
        document: DocumentId,
    },
    FullscreenExitCompositionUnsupported {
        document: DocumentId,
        fact: crate::DomFullscreenExitFact,
    },
    FullscreenExitDocumentRetired {
        invocation: crate::DomFullscreenExitId,
        document: DocumentId,
        step: crate::DomFullscreenExitStep,
    },
    FullscreenModelUnavailable {
        step: crate::DomFullscreenModelStep,
    },
    TopLayerAdoptionUnavailable {
        subject: Option<NodeId>,
        old_document: Option<DocumentId>,
    },
    StaleRange,
    RangeAssociationIneligible,
    RangePublicationUnavailable,
    /// A reached operation needs a current live hosted owner association.
    HostedSourceUnavailable,
    HostedCompositionRequired,
    /// A custody capability, target, or occurrence does not match this owner.
    InvalidDispatchCustody,
    /// The finite independent custody-processing allowance was exhausted.
    DispatchCustodyWorkLimit {
        limit: u64,
    },
    /// A raise-only Source policy call attempted to reduce an allowance.
    SourceLimitReduction,
    InvalidSourceEvent,
    SourceInvariantFailed {
        operation: crate::DomSourceOperationId,
        failure: crate::DomSourceInvariantFailure,
    },
    SourceReceiverRetired {
        operation: crate::DomSourceOperationId,
        subject: NodeId,
        action: surgeist_html::HtmlDialogAction,
    },
    SourceCloseWatcherReceiverRetired {
        operation: crate::DomSourceOperationId,
        subject: NodeId,
        action: surgeist_html::HtmlCloseWatcherRequestAction,
    },
    CloseWatcherProtocol(surgeist_html::HtmlCloseWatcherRequestError),
    SourceEventExecutorUnavailable,
    DialogInvalidState,
    NeedsEventfulOperation {
        subject: Option<NodeId>,
        action: surgeist_html::HtmlDialogAction,
    },
    SourceNestedOperationLimit {
        limit: usize,
    },
    SourceUrlInputsMismatch,
    SourceUrlUnavailable,
    ForeignOwner,
    StaleNode,
    ForeignBatch,
    UnavailableCreation,
    WrongNodeKind,
    WrongDocument,
    HierarchyRequest,
    InvalidCharacter,
    Namespace,
    NotSupported,
    NativeInitializationRequired(surgeist_html::HtmlElementClass),
    NonceInapplicable,
    TemplateContentsUnavailable,
    MarqueeInapplicable,
    DialogInapplicable,
    DialogDocumentStateUnavailable,
    AutofocusDocumentStateUnavailable,
    AutofocusInsertionUnavailable {
        action: surgeist_html::HtmlAutofocusInsertionAction,
    },
    AutofocusInsertionProtocol(surgeist_html::HtmlAutofocusInsertionError),
    DesignModeProcessingUnavailable {
        step: crate::DomSourceDesignModeStep,
    },
    DesignModeProtocol(surgeist_html::HtmlDesignModeError),
    DialogFocusingUnavailable {
        action: surgeist_html::HtmlDialogFocusingAction,
    },
    DialogFocusingProtocol(surgeist_html::HtmlDialogFocusingError),
    FocusDelegateProtocol(surgeist_html::HtmlFocusDelegateError),
    AutofocusDelegateProtocol(surgeist_html::HtmlAutofocusDelegateError),
    TopLayerAlreadyActive {
        subject: NodeId,
        document: DocumentId,
    },
    DialogProcessingUnavailable {
        action: surgeist_html::HtmlDialogAction,
    },
    DialogProtocol(surgeist_html::HtmlDialogCursorError),
    CustomValidityInapplicable,
    ButtonInapplicable,
    InputInapplicable,
    TextareaInapplicable,
    OutputInapplicable,
    OutputProtocol(surgeist_html::HtmlOutputCursorError),
    TextareaReadWorkLimit {
        limit: u64,
    },
    InputFilenameInvalidState,
    InputProcessingProtocol,
    InputProcessingWorkLimit {
        limit: u64,
    },
    InputNumeric(surgeist_html::bounded::RangeNumericError),
    InputColorProtocol(surgeist_css::CssColorProtocolError),
    InputColorUnavailable(surgeist_css::CssColorUnavailable),
    InputColorUnsupported(surgeist_css::CssColorUnsupported),
    InputColorFacts(surgeist_css::CssColorFactError),
    InputColorNumerical {
        error: surgeist_color::ColorError,
        input: surgeist_css::CssColorInputRange,
    },
    StyleCompositionRequired(surgeist_html::HtmlStyleUpdateAction),
    StyleOwnerAttributesCompositionRequired,
    StyleDisabledCompositionRequired,
    StyleProcessingUnavailable,
    Meta(crate::DomMetaError),
    StyleProtocol,
    SvgScriptProcessingUnavailable,
    SourceOperationPending,
    IncompleteSourceOperation,
    InvalidSourceOperation,
    SourcePublicationLimit,
    SourceOutputLimit,
    SourceIdentityLimit,
    StyleReceiptMismatch,
    StyleFactsMismatch,
    StyleCssomRootBusy,
    StyleCssomRootPoisoned,
    SourceWorkLimit {
        limit: u64,
    },
    SelectInapplicable,
    OptionInapplicable,
    SelectProtocol,
    SelectValueQueriesUnconfigured,
    SelectCapture(surgeist_html_semantics::HtmlCaptureError),
    SelectSemantic(surgeist_html_semantics::HtmlSemanticFailure),
    InvalidSelectSemanticLimit {
        resource: surgeist_html_semantics::HtmlSemanticResource,
        limit: usize,
        used: usize,
    },
    LinkProcessingUnavailable,
    ImageProcessingUnavailable,
    ImageUpdateExhausted,
    HyperlinkProcessingUnavailable,
    EncodingUrlFacts(crate::DomEncodingUrlFactsError),
    EncodingUrlWorkLimit {
        limit: u64,
    },
    BaseFacts(crate::DomBaseFactsError),
    BaseWorkLimit {
        limit: u64,
    },
    SelectWorkLimit {
        limit: u64,
    },
    InvalidNativeTask,
    SlotInapplicable,
    NonceEnvironmentUnavailable(surgeist_html::HtmlNonceMissing),
    NotFound,
    InvalidDestruction,
    IndexSize,
    Cancelled,
    ResourceLimit {
        resource: DomResource,
        limit: usize,
    },
    InvalidLimit(DomResource),
    AllocationFailed,
    RepresentationExhausted,
    OwnerIdentityExhausted,
    RevisionExhausted,
    ViewTransitionsUninitialized,
    ViewTransitionRevisionMismatch,
    StaleViewTransition,
    StaleViewTransitionRecord,
    StaleViewTransitionPseudo,
    InvalidViewTransitionState,
    DuplicateViewTransitionName,
    ViewTransitionOriginUnavailable,
    ViewTransitionCssom(DomViewTransitionCssomError),
    ViewTransitionCaptureWorkLimit {
        limit: u64,
    },
}
impl std::fmt::Display for DomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DOM operation rejected: {self:?}")
    }
}
impl std::error::Error for DomError {}

/// Explicit finite admission policy. No hidden or unlimited default is selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomLimits {
    fullscreen_exits: usize,
    fullscreen_events: usize,
    ranges: usize,
    dispatch_custody: crate::DomDispatchCustodyLimits,
    nodes: usize,
    units: usize,
    commands: usize,
    slot_references: usize,
    autofocus_candidates: usize,
    slot_assignment_inputs: usize,
    history_entries: usize,
    history_bytes: usize,
    transitions: usize,
    records: usize,
    pseudos: usize,
    input_scratch: usize,
    input_work: u64,
    details_intentions: usize,
    base_observations: usize,
    base_work: u64,
    encoding_url_observations: usize,
    encoding_url_work: u64,
    meta_work: u64,
    meta_theme_scratch: usize,
    meta_color_scheme: Option<surgeist_css::CssUtf16PropertyValueLimits>,
    meta_color_scheme_bytes: usize,
    select_tasks: usize,
    select_scratch: usize,
    select_work: u64,
    select_value_queries: Option<(
        surgeist_html_semantics::HtmlCaptureLimits,
        surgeist_html_semantics::HtmlSemanticLimits,
    )>,
}
impl DomLimits {
    pub fn with_fullscreen_exits(mut self, count: usize) -> Result<Self, DomError> {
        if count > isize::MAX as usize / size_of::<fullscreen::exit::Invocation>() {
            return Err(DomError::InvalidLimit(DomResource::FullscreenExits));
        }
        self.fullscreen_exits = count;
        Ok(self)
    }
    pub fn with_fullscreen_events(mut self, count: usize) -> Result<Self, DomError> {
        if count > isize::MAX as usize / size_of::<crate::DomFullscreenEvent>() {
            return Err(DomError::InvalidLimit(DomResource::FullscreenEvents));
        }
        self.fullscreen_events = count;
        Ok(self)
    }
    pub const fn fullscreen_exits(self) -> usize {
        self.fullscreen_exits
    }
    pub const fn fullscreen_events(self) -> usize {
        self.fullscreen_events
    }
    /// Maximum retained Range slots, including vacant slots and exhausted generations.
    pub fn with_ranges(mut self, ranges: usize) -> Result<Self, DomError> {
        if ranges
            > isize::MAX as usize
                / size_of::<crate::range::Slot<Key>>().max(size_of::<crate::range::Slot<NodeId>>())
        {
            return Err(DomError::InvalidLimit(DomResource::Ranges));
        }
        self.ranges = ranges;
        Ok(self)
    }
    pub fn ranges(self) -> usize {
        self.ranges
    }
    /// Aggregate installed candidate references across all Document histories.
    /// Native empty state needs no allowance; each growing insertion does.
    pub fn with_autofocus_candidates(mut self, candidates: usize) -> Result<Self, DomError> {
        if candidates > isize::MAX as usize / size_of::<NodeId>().max(size_of::<Key>()) {
            return Err(DomError::InvalidLimit(DomResource::AutofocusCandidates));
        }
        self.autofocus_candidates = candidates;
        Ok(self)
    }
    pub fn autofocus_candidates(self) -> usize {
        self.autofocus_candidates
    }

    /// Select explicit finite dispatch-custody allowances; the default is disabled.
    pub fn with_dispatch_custody(mut self, limits: crate::DomDispatchCustodyLimits) -> Self {
        self.dispatch_custody = limits;
        self
    }
    /// Current independent dispatch-custody allowances.
    pub fn dispatch_custody(self) -> crate::DomDispatchCustodyLimits {
        self.dispatch_custody
    }
    pub fn new(
        nodes: usize,
        units: usize,
        commands: usize,
        slot_references: usize,
        slot_assignment_inputs: usize,
        history_entries: usize,
        history_bytes: usize,
    ) -> Result<Self, DomError> {
        for (value, resource, width) in [
            (nodes, DomResource::ManagedNodes, size_of::<Slot>()),
            (
                slot_references,
                DomResource::SlotReferences,
                size_of::<NodeId>(),
            ),
            (
                slot_assignment_inputs,
                DomResource::SlotAssignmentInputs,
                size_of::<NodeOperand>(),
            ),
            (units, DomResource::StringCodeUnits, size_of::<u16>()),
            (commands, DomResource::Commands, size_of::<Command>()),
            (
                history_entries,
                DomResource::HistoryEntries,
                size_of::<HistoryEntry>(),
            ),
            (history_bytes, DomResource::HistoryBytes, 1),
        ] {
            if value > isize::MAX as usize / width {
                return Err(DomError::InvalidLimit(resource));
            }
        }
        Ok(Self {
            fullscreen_exits: 0,
            fullscreen_events: 0,
            ranges: 0,
            dispatch_custody: crate::DomDispatchCustodyLimits::default(),
            nodes,
            units,
            commands,
            slot_references,
            autofocus_candidates: 0,
            slot_assignment_inputs,
            history_entries,
            history_bytes,
            transitions: 0,
            records: 0,
            pseudos: 0,
            input_scratch: 0,
            input_work: 0,
            details_intentions: 0,
            base_observations: 0,
            base_work: 0,
            encoding_url_observations: 0,
            encoding_url_work: 0,
            meta_work: 0,
            meta_theme_scratch: 0,
            meta_color_scheme: None,
            meta_color_scheme_bytes: 0,
            select_tasks: 0,
            select_scratch: 0,
            select_work: 0,
            select_value_queries: None,
        })
    }
    /// Explicit finite VT admission; the unconfigured feature has zero capacity.
    pub fn with_view_transitions(
        mut self,
        transitions: usize,
        records: usize,
        pseudos: usize,
    ) -> Result<Self, DomError> {
        for (value, resource, width) in [
            (
                transitions,
                DomResource::ViewTransitions,
                size_of::<Transition>(),
            ),
            (
                records,
                DomResource::ViewTransitionRecords,
                size_of::<Record>(),
            ),
            (
                pseudos,
                DomResource::ViewTransitionPseudos,
                size_of::<bool>(),
            ),
        ] {
            if value > isize::MAX as usize / width {
                return Err(DomError::InvalidLimit(resource));
            }
        }
        self.transitions = transitions;
        self.records = records;
        self.pseudos = pseudos;
        Ok(self)
    }
    pub fn view_transitions(self) -> usize {
        self.transitions
    }
    /// Reached variable numeric/CSS processing only: peak logical transient
    /// bytes and cumulative work for one whole private batch. Unconfigured is0.
    pub fn with_input_processing(
        mut self,
        scratch_bytes: usize,
        work: u64,
    ) -> Result<Self, DomError> {
        if scratch_bytes > isize::MAX as usize {
            return Err(DomError::InvalidLimit(DomResource::InputProcessingScratch));
        }
        self.input_scratch = scratch_bytes;
        self.input_work = work;
        Ok(self)
    }
    pub fn input_processing_scratch_bytes(self) -> usize {
        self.input_scratch
    }
    pub fn input_processing_work(self) -> u64 {
        self.input_work
    }
    /// Reached Select-family effects use independent finite pending, scratch and work bounds.
    pub fn with_select_processing(
        mut self,
        pending_updates: usize,
        scratch_bytes: usize,
        work: u64,
    ) -> Result<Self, DomError> {
        if pending_updates > isize::MAX as usize / size_of::<select::Task<NodeId>>() {
            return Err(DomError::InvalidLimit(DomResource::SelectTasks));
        }
        if scratch_bytes > isize::MAX as usize {
            return Err(DomError::InvalidLimit(DomResource::SelectScratch));
        }
        self.select_tasks = pending_updates;
        self.select_scratch = scratch_bytes;
        self.select_work = work;
        Ok(self)
    }
    /// Admission for reached Select value setters. Capture retention is bounded
    /// separately for each sequential Option capture. Semantic work and allocated
    /// bytes are cumulative across all queries in one private batch; other
    /// semantic limits apply to each query. DOM adapter/effect storage and work
    /// remain governed by `with_select_processing`.
    pub fn with_select_value_queries(
        mut self,
        capture: surgeist_html_semantics::HtmlCaptureLimits,
        semantics: surgeist_html_semantics::HtmlSemanticLimits,
    ) -> Self {
        self.select_value_queries = Some((capture, semantics));
        self
    }
    /// Aggregate actual observation capacity and cumulative native Base work.
    /// Immutable fact strings and frozen URL serializations also consume the
    /// canonical string-unit limit. Stackless scans need no traversal bank.
    pub fn with_base_processing(
        mut self,
        observation_records: usize,
        work: u64,
    ) -> Result<Self, DomError> {
        if observation_records
            > isize::MAX as usize / size_of::<surgeist_html_semantics::HtmlBaseParseObservation>()
        {
            return Err(DomError::InvalidLimit(DomResource::BaseObservations));
        }
        self.base_observations = observation_records;
        self.base_work = work;
        Ok(self)
    }
    pub fn base_observation_records(self) -> usize {
        self.base_observations
    }
    pub fn base_work(self) -> u64 {
        self.base_work
    }
    /// Aggregate retained href-observation capacity and cumulative qualified
    /// lookup/binding work. Cache/input UTF16 extents use canonical string units.
    /// Absent href and namespace/equality skips need no parse admission.
    pub fn with_encoding_url_processing(
        mut self,
        observation_records: usize,
        work: u64,
    ) -> Result<Self, DomError> {
        if observation_records
            > isize::MAX as usize
                / size_of::<surgeist_html_semantics::HtmlEncodingUrlParseObservation>()
        {
            return Err(DomError::InvalidLimit(DomResource::EncodingUrlObservations));
        }
        self.encoding_url_observations = observation_records;
        self.encoding_url_work = work;
        Ok(self)
    }
    pub fn encoding_url_observation_records(self) -> usize {
        self.encoding_url_observations
    }
    pub fn encoding_url_work(self) -> u64 {
        self.encoding_url_work
    }
    /// Cumulative Meta work, retained through synchronous source suspension.
    pub const fn with_meta_processing_work(mut self, work: u64) -> Self {
        self.meta_work = work;
        self
    }
    /// Actual simultaneous banks/registers of one theme-color parse.
    pub fn with_meta_theme_color_scratch_bytes(mut self, bytes: usize) -> Result<Self, DomError> {
        if bytes > isize::MAX as usize {
            return Err(DomError::InvalidLimit(DomResource::MetaThemeColorScratch));
        }
        self.meta_theme_scratch = bytes;
        Ok(self)
    }
    pub const fn meta_theme_color_scratch_bytes(self) -> usize {
        self.meta_theme_scratch
    }
    /// Complete finite CSS-owned UTF16/property allowance; unconfigured is unavailable.
    pub const fn with_meta_color_scheme_limits(
        mut self,
        limits: surgeist_css::CssUtf16PropertyValueLimits,
    ) -> Self {
        self.meta_color_scheme = Some(limits);
        self
    }
    pub const fn meta_color_scheme_limits(
        self,
    ) -> Option<surgeist_css::CssUtf16PropertyValueLimits> {
        self.meta_color_scheme
    }
    /// Aggregate installed CSS report/source backing across all Documents.
    pub fn with_meta_color_scheme_retained_bytes(mut self, bytes: usize) -> Result<Self, DomError> {
        if bytes > isize::MAX as usize {
            return Err(DomError::InvalidLimit(
                DomResource::MetaColorSchemeRetainedBytes,
            ));
        }
        self.meta_color_scheme_bytes = bytes;
        Ok(self)
    }
    pub const fn meta_color_scheme_retained_bytes(self) -> usize {
        self.meta_color_scheme_bytes
    }
    /// Retained raw Details transitions, including staged cancellation evidence.
    /// Scans use canonical node/string/command bounds and per-step cancellation.
    pub fn with_details_toggle_intentions(mut self, max_pending: usize) -> Result<Self, DomError> {
        let width = size_of::<crate::DomDetailsToggleIntention>()
            .max(size_of::<details::Pending<Key>>())
            .max(size_of::<details::Pending<NodeId>>());
        if max_pending > isize::MAX as usize / width {
            return Err(DomError::InvalidLimit(DomResource::DetailsToggleIntentions));
        }
        self.details_intentions = max_pending;
        Ok(self)
    }
    pub fn details_toggle_intentions(self) -> usize {
        self.details_intentions
    }
    pub fn pending_select_updates(self) -> usize {
        self.select_tasks
    }
    pub fn select_scratch_bytes(self) -> usize {
        self.select_scratch
    }
    pub fn select_work(self) -> u64 {
        self.select_work
    }
    pub fn view_transition_records(self) -> usize {
        self.records
    }
    pub fn view_transition_pseudos(self) -> usize {
        self.pseudos
    }
    pub fn managed_nodes(self) -> usize {
        self.nodes
    }
    pub fn string_code_units(self) -> usize {
        self.units
    }
    pub fn slot_references(self) -> usize {
        self.slot_references
    }
    pub fn slot_assignment_inputs(self) -> usize {
        self.slot_assignment_inputs
    }
    pub fn commands_per_batch(self) -> usize {
        self.commands
    }
    pub fn history_entries(self) -> usize {
        self.history_entries
    }
    /// Retained entry records plus exact change-array payload bytes. Allocator
    /// bookkeeping and caller-held reports/snapshots are outside this budget.
    pub fn history_bytes(self) -> usize {
        self.history_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomUsage {
    ranges: usize,
    meta_color_scheme_bytes: usize,
    autofocus_candidates: usize,
    slot_references: usize,
    nodes: usize,
    units: usize,
    history_entries: usize,
    history_bytes: usize,
    transitions: usize,
    records: usize,
    pseudos: usize,
}
impl DomUsage {
    pub fn ranges(self) -> usize {
        self.ranges
    }
    pub const fn meta_color_scheme_retained_bytes(self) -> usize {
        self.meta_color_scheme_bytes
    }
    pub fn autofocus_candidates(self) -> usize {
        self.autofocus_candidates
    }
    pub fn slot_references(self) -> usize {
        self.slot_references
    }
    pub fn view_transitions(self) -> usize {
        self.transitions
    }
    pub fn view_transition_records(self) -> usize {
        self.records
    }
    pub fn view_transition_pseudos(self) -> usize {
        self.pseudos
    }
    pub fn managed_nodes(self) -> usize {
        self.nodes
    }
    pub fn string_code_units(self) -> usize {
        self.units
    }
    pub fn history_entries(self) -> usize {
        self.history_entries
    }
    pub fn history_bytes(self) -> usize {
        self.history_bytes
    }
}

/// Complete changed domains for this surface. Created/destroyed nodes cover
/// their entire state. Children includes derived Element-sibling/index changes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomChange {
    RangeCreated(crate::DomRangeId),
    RangeBoundary(crate::DomRangeId),
    RangeRetired(crate::DomRangeId),
    DocumentActiveRange(DocumentId),
    DocumentDesignMode(DocumentId),
    SourceOperation,
    HostedOperationState {
        operation: crate::hosted::DomHostedOperationId,
        complete: bool,
    },
    ImageUpdateCancelled {
        update: crate::DomImageUpdateId,
        flags: surgeist_html::HtmlImageUpdateFlags,
        reason: crate::DomImageUpdateCancellationReason,
    },
    /// Template contents ownership or a Document's appropriate-owner cache changed.
    TemplateRelations,
    /// Recompute Document-scoped URL/base-dependent facts and consumers.
    DocumentBase(DocumentId),
    DocumentMeta(DocumentId),
    DocumentInteractiveState(DocumentId),
    /// Raw-derived fullscreen, simple-document or keyboard-lock facts changed.
    DocumentFullscreenState(DocumentId),
    DocumentFullscreenEvents(DocumentId),
    DocumentAutofocusState(DocumentId),
    DocumentRenderBlocking(DocumentId),
    DocumentEncodingUrlFacts(DocumentId),
    SlotManual(NodeId),
    SlotAssignedNodes(NodeId),
    ManualAssignment(NodeId),
    StoredAssignedSlot(NodeId),
    /// Conservatively recompute current slot/flattened/flat projections.
    SlotQueries,
    IntrinsicTasks,
    DetailsToggleIntentions,
    DetailsToggleIntentionCancelled {
        operation: crate::DomDetailsToggleIntentionId,
        target: NodeId,
        old: surgeist_html::HtmlDetailsState,
        new: surgeist_html::HtmlDetailsState,
        reason: crate::DomDetailsToggleCancellationReason,
    },
    SelectTaskCancelled {
        operation: crate::DomNativeTaskId,
        select: NodeId,
        reason: crate::DomSelectTaskCancellationReason,
    },
    /// Native destruction canceled this exact-generation pending recipient.
    SlotSignalCancelled {
        slot: NodeId,
        reason: crate::DomSlotSignalCancellationReason,
    },
    Created(NodeId),
    Destroyed(NodeId),
    Data(NodeId),
    Attributes(NodeId),
    ElementState(NodeId),
    NodeDocument(NodeId),
    Parent(NodeId),
    Children(NodeId),
    /// The Element's canonical hosted-root link changed, including closed roots.
    ShadowRoot(NodeId),
    /// Conservatively recompute all shadow/tree-scope/connectedness relations
    /// from the new snapshot. Parent/Children remain ordinary-tree facts.
    ShadowRelations,
    ViewTransitionDocument(DocumentId),
    /// Complete transition state/topology invalidation, including pseudo flags.
    ViewTransition(DomViewTransitionId),
    ViewTransitionRecord(DomViewTransitionRecordId),
    ViewTransitionCaptured(DomViewTransitionSubject),
}

#[derive(Clone, Debug)]
pub struct DomChanges {
    from: DomStamp,
    to: DomStamp,
    changes: Arc<[DomChange]>,
}
impl DomChanges {
    pub fn from(&self) -> DomStamp {
        self.from
    }
    pub fn to(&self) -> DomStamp {
        self.to
    }
    pub fn changes(&self) -> &[DomChange] {
        &self.changes
    }
}

/// Complete interval unavailable; a consumer must recompute from a snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomHistoryError {
    ForeignOwner,
    FutureRevision,
    FullRecomputeRequired { available_from: DomRevision },
    AllocationFailed,
}
impl std::fmt::Display for DomHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DOM change coverage unavailable: {self:?}")
    }
}
impl std::error::Error for DomHistoryError {}

#[derive(Debug)]
pub struct DomCommit {
    created_ranges: Vec<Option<crate::DomRangeId>>,
    owner: Arc<BatchIdentity>,
    created: Vec<Option<NodeId>>,
    documents: Vec<bool>,
    transition_ids: Vec<Option<DomViewTransitionId>>,
    record_ids: Vec<Option<DomViewTransitionRecordId>>,
    changes: DomChanges,
}
pub(in crate::store) struct CommittedPublication {
    pub commit: DomCommit,
    pub committed: transaction::PublicationCommitted,
}
impl DomCommit {
    pub fn created_range(
        &self,
        ticket: &crate::CreatedRange,
    ) -> Result<Option<crate::DomRangeId>, DomError> {
        if !Arc::ptr_eq(&self.owner, &ticket.owner) {
            return Err(DomError::ForeignBatch);
        }
        self.created_ranges
            .get(ticket.index)
            .copied()
            .ok_or(DomError::UnavailableCreation)
    }
    pub(crate) fn source_output(&self) -> Option<crate::DomSourceOutput> {
        if let Some(Some(range)) = self.created_ranges.first() {
            return Some(crate::DomSourceOutput::Range(*range));
        }
        if let Some(Some(node)) = self.created.first() {
            return Some(if self.documents.first() == Some(&true) {
                crate::DomSourceOutput::Document(DocumentId(*node))
            } else {
                crate::DomSourceOutput::Node(*node)
            });
        }
        if let Some(Some(id)) = self.transition_ids.first() {
            return Some(crate::DomSourceOutput::ViewTransition(*id));
        }
        self.record_ids
            .first()
            .copied()
            .flatten()
            .map(crate::DomSourceOutput::ViewTransitionRecord)
    }
    pub fn view_transition(
        &self,
        ticket: &CreatedViewTransition,
    ) -> Result<Option<DomViewTransitionId>, DomError> {
        if !Arc::ptr_eq(&ticket.owner, &self.owner) {
            return Err(DomError::ForeignBatch);
        }
        self.transition_ids
            .get(ticket.index)
            .copied()
            .ok_or(DomError::UnavailableCreation)
    }
    pub fn view_transition_record(
        &self,
        ticket: &CreatedViewTransitionRecord,
    ) -> Result<Option<DomViewTransitionRecordId>, DomError> {
        if !Arc::ptr_eq(&ticket.owner, &self.owner) {
            return Err(DomError::ForeignBatch);
        }
        self.record_ids
            .get(ticket.index)
            .copied()
            .ok_or(DomError::UnavailableCreation)
    }
    pub fn changes(&self) -> &DomChanges {
        &self.changes
    }
    /// None means the batch destroyed its own unpublished creation.
    pub fn created(&self, ticket: &CreatedNode) -> Result<Option<NodeId>, DomError> {
        if !Arc::ptr_eq(&ticket.owner, &self.owner) {
            return Err(DomError::ForeignBatch);
        }
        self.created
            .get(ticket.index)
            .copied()
            .ok_or(DomError::UnavailableCreation)
    }
    pub fn document(&self, ticket: &CreatedNode) -> Result<DocumentId, DomError> {
        let id = self.created(ticket)?.ok_or(DomError::UnavailableCreation)?;
        if !self.documents[ticket.index] {
            return Err(DomError::WrongNodeKind);
        }
        Ok(DocumentId(id))
    }
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug)]
pub(crate) struct Node {
    pub(crate) receipts: transaction::NodeReceipts,
    pub(crate) fullscreen_events: SharedVec<crate::DomFullscreenEvent>,
    pub(crate) active_range: crate::DomActiveRangeState,
    pub(crate) document: DocumentId,
    pub(crate) relevant_global: Option<crate::hosted::DomHostedGlobalId>,
    pub(crate) kind: DomNodeKind,
    pub(crate) parent: Option<NodeId>,
    pub(crate) children: SharedVec<NodeId>,
    pub(crate) shadow_root: Option<NodeId>,
    pub(crate) host: Option<NodeId>,
    pub(crate) template_contents: Option<NodeId>,
    pub(crate) template_patch: Option<crate::template::TemplatePatch<NodeId>>,
    pub(crate) template_document: Option<crate::template::TemplateDocument<NodeId>>,
    pub(crate) manual_nodes: SharedVec<NodeId>,
    pub(crate) assigned_nodes: SharedVec<NodeId>,
    pub(crate) manual_slot: Option<NodeId>,
    pub(crate) assigned_slot: Option<NodeId>,
    pub(crate) reflected_targets: Option<crate::reflected::ReflectedTargets<NodeId>>,
    pub(crate) control_owner: Option<NodeId>,
    pub(crate) option_select: Option<NodeId>,
    pub(crate) form_past_names: Option<crate::form::FormPastNames<NodeId>>,
    pub(crate) script: Option<crate::script::ScriptState<NodeId>>,
    pub(crate) style_parser_document: Option<NodeId>,
    pub(crate) base: Option<crate::base::BaseState<NodeId>>,
    pub(crate) base_document: Option<canonical::BaseDocument>,
    pub(crate) link: Option<crate::base::LinkState<NodeId>>,
    pub(crate) image: Option<crate::image::ImageState<NodeId>>,
    pub(crate) hyperlink: Option<crate::hyperlink::HyperlinkState<NodeId>>,
    pub(crate) encoding_url_document: Option<crate::hyperlink::EncodingUrlDocument>,
    pub(crate) interactive_document: Option<canonical::Interactive>,
    pub(crate) autofocus_document: Option<canonical::Autofocus>,
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.fullscreen_events == other.fullscreen_events
            && self.active_range == other.active_range
            && self.document == other.document
            && self.relevant_global == other.relevant_global
            && self.kind == other.kind
            && self.parent == other.parent
            && self.children == other.children
            && self.shadow_root == other.shadow_root
            && self.host == other.host
            && self.template_contents == other.template_contents
            && self.template_patch == other.template_patch
            && self.template_document == other.template_document
            && self.manual_nodes == other.manual_nodes
            && self.assigned_nodes == other.assigned_nodes
            && self.manual_slot == other.manual_slot
            && self.assigned_slot == other.assigned_slot
            && self.reflected_targets == other.reflected_targets
            && self.control_owner == other.control_owner
            && self.option_select == other.option_select
            && self.form_past_names == other.form_past_names
            && self.script == other.script
            && self.style_parser_document == other.style_parser_document
            && self.base == other.base
            && self.base_document == other.base_document
            && self.link == other.link
            && self.image == other.image
            && self.hyperlink == other.hyperlink
            && self.encoding_url_document == other.encoding_url_document
            && self.interactive_document == other.interactive_document
            && self.autofocus_document == other.autofocus_document
    }
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, PartialEq)]
pub(crate) struct Slot {
    pub(crate) generation: u64,
    retired: bool,
    pub(crate) node: Option<Node>,
}
#[derive(Debug)]
pub(crate) struct State {
    identity: custody::MaterializationId,
    counts: transaction::Counts,
    inbound: transaction::inbound::InboundIndex,
    controls: transaction::controls::ControlIndex,
    slot_index: transaction::slot_index::SlotIndex,
    family_index: transaction::family_discovery::FamilyIndex,
    role_facts: transaction::role_facts::Index,
    container_index: hosted::containers::ContainerIndex,
    range_dependencies: transaction::ranges::RangeDependencies,
    signal_targets: transaction::queues::CanonicalTargets,
    native_targets: transaction::queues::CanonicalTargets,
    details_targets: transaction::queues::CanonicalTargets,
    pub(crate) ranges: bank::Bank<crate::range::Slot<NodeId>>,
    pub(crate) incomplete_source: Option<crate::DomSourceOperationId>,
    pub(crate) incomplete_hosted: Option<crate::hosted::DomHostedOperationId>,
    pub(crate) slots: bank::Bank<Slot>,
    pub(crate) registries: Option<Arc<registry::Bank>>,
    pub(crate) hosted_globals: Option<Arc<hosted::GlobalDocumentIndex>>,
    pub(crate) hosted_history: Option<Arc<hosted::history::Graph>>,
    pub(crate) view_transitions: ViewTransitions,
    pub(crate) signal_slots: SharedVec<NodeId>,
    pub(crate) notification_queued: bool,
    native_tasks: SharedVec<select::Task<NodeId>>,
    next_task: u64,
    pub(crate) details_intentions: SharedVec<crate::DomDetailsToggleIntention>,
    next_details_intention: u64,
    nodes: usize,
    units: usize,
}
impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.ranges == other.ranges
            && self.incomplete_source == other.incomplete_source
            && self.incomplete_hosted == other.incomplete_hosted
            && self.slots == other.slots
            && self.registries == other.registries
            && self.hosted_globals == other.hosted_globals
            && self.hosted_history == other.hosted_history
            && self.view_transitions == other.view_transitions
            && self.signal_slots == other.signal_slots
            && self.notification_queued == other.notification_queued
            && self.native_tasks == other.native_tasks
            && self.next_task == other.next_task
            && self.details_intentions == other.details_intentions
            && self.next_details_intention == other.next_details_intention
            && self.nodes == other.nodes
            && self.units == other.units
    }
}
impl State {
    #[cfg(test)]
    pub(crate) fn mutate_slot(&mut self, index: usize, change: impl FnOnce(&mut Slot)) {
        let mut slot = self.slots[index].clone();
        change(&mut slot);
        if let Some(node) = &mut slot.node {
            source::storage::refresh_node_receipts(node).unwrap();
        }
        let payload = source::storage::canonical_slot(&slot, || Ok(())).unwrap();
        let reusable = slot.node.is_none() && !slot.retired;
        self.slots
            .set(
                index,
                slot,
                reusable,
                bank::Totals {
                    bytes: payload.bytes,
                    units: payload.units,
                    occurrences: 1,
                },
            )
            .unwrap();
    }

    pub(crate) fn node(&self, owner: DomOwnerId, id: NodeId) -> Result<&Node, DomError> {
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::CanonicalRecordRead);
        }

        if id.owner != owner {
            return Err(DomError::ForeignOwner);
        }
        self.slots
            .get(id.slot)
            .filter(|s| s.generation == id.generation)
            .and_then(|s| s.node.as_ref())
            .ok_or(DomError::StaleNode)
    }
}

/// Exclusive canonical mutation owner. Snapshots never borrow its mutable state.
/// The whole owner is Send, including an exact parked source tail and retained
/// mutable CSSOM custody. Moving it does not replay or rebind that operation.
/// Shared mutation and Sync are not supplied by this exclusive-owner contract.
pub struct Dom {
    fullscreen_exits: fullscreen::custody::Invocations,
    next_fullscreen_exit: u64,
    element_tasks: Option<std::sync::Weak<crate::element_tasks::Binding>>,
    dispatch_custody: dispatch::CustodyBank,
    owner: DomOwnerId,
    revision: DomRevision,
    limits: DomLimits,
    state: Arc<State>,
    state_custody: transaction::StateCustody,
    custody_registry: bank::CustodyRegistry,
    history: history::History,
    document_environment: crate::DomDocumentEnvironment,
    active_task: std::sync::Weak<crate::select::Authorization>,
    source_operation: Option<Box<source::operation::Operation>>,
    next_source_operation: u64,
    pub(in crate::store) source_parents: Vec<Box<source::operation::Operation>>,
    source_event_executor: Option<std::sync::Weak<crate::source::event::ExecutorBinding>>,
    style_cssom_roots: style_custody::Bank,
    hosted_source: Option<crate::hosted::SourceObservation>,
    next_hosted_source: u64,
    details_transferred: bool,
    hosted_operations: hosted::operations::Operations,
    hosted_revisions: u64,
    _custody_administration: custody::Debit,
}
impl Dom {
    pub fn new(limits: DomLimits) -> Result<Self, DomError> {
        Self::with_native_input_platform(limits, crate::DomNativeInputPlatform::headless())
    }
    /// Constructs one immutable native platform; `new` selects the named
    /// headless Supported/Supported profile. Image acquisition is disabled.
    pub fn with_native_input_platform(
        limits: DomLimits,
        input: crate::DomNativeInputPlatform,
    ) -> Result<Self, DomError> {
        let ledger = custody::Ledger::default();
        let administration = ledger.retain(custody::Ledger::administration_usage(), |_| Ok(()))?;
        let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(()))?;
        let state = State {
            identity: custody::MaterializationId::fresh()?,
            counts: transaction::Counts::default(),
            inbound: transaction::inbound::InboundIndex::default(),
            controls: transaction::controls::ControlIndex::default(),
            slot_index: transaction::slot_index::SlotIndex::default(),
            family_index: transaction::family_discovery::FamilyIndex::default(),
            role_facts: transaction::role_facts::Index::default(),
            container_index: hosted::containers::ContainerIndex::default(),
            range_dependencies: transaction::ranges::RangeDependencies::default(),
            signal_targets: transaction::queues::CanonicalTargets::default(),
            native_targets: transaction::queues::CanonicalTargets::default(),
            details_targets: transaction::queues::CanonicalTargets::default(),
            ranges: bank::Bank::default(),
            incomplete_source: None,
            incomplete_hosted: None,
            slots: bank::Bank::default(),
            registries: None,
            hosted_globals: None,
            hosted_history: None,
            view_transitions: ViewTransitions::new(),
            signal_slots: SharedVec::default(),
            notification_queued: false,
            native_tasks: SharedVec::default(),
            next_task: 0,
            details_intentions: SharedVec::default(),
            next_details_intention: 0,
            nodes: 0,
            units: 0,
        };
        let state_custody = transaction::StateCustody::initial(&state, &registry, |_| Ok(()))?;
        Ok(Self {
            style_cssom_roots: style_custody::Bank::new(&registry),
            fullscreen_exits: fullscreen::custody::Invocations::new(&registry),
            next_fullscreen_exit: 1,
            dispatch_custody: dispatch::CustodyBank::default(),
            element_tasks: None,
            owner: DomOwnerId::allocate()?,
            revision: DomRevision(0),
            limits,
            state: Arc::new(state),
            state_custody,
            custody_registry: registry,
            _custody_administration: administration,
            active_task: std::sync::Weak::new(),
            source_operation: None,
            next_source_operation: 0,
            source_parents: Vec::new(),
            source_event_executor: None,
            hosted_source: None,
            next_hosted_source: 0,
            details_transferred: false,
            hosted_operations: hosted::operations::Operations::default(),
            hosted_revisions: 0,
            history: history::History::new(),
            document_environment: crate::DomDocumentEnvironment::new(input),
        })
    }
    pub fn owner(&self) -> DomOwnerId {
        self.owner
    }
    pub fn revision(&self) -> DomRevision {
        self.revision
    }
    pub fn limits(&self) -> DomLimits {
        self.limits
    }
    pub fn usage(&self) -> DomUsage {
        let (transitions, records, pseudos) = self.state.view_transitions.usage();
        DomUsage {
            ranges: self
                .state
                .ranges
                .values()
                .filter(|slot| slot.record.is_some())
                .count(),
            meta_color_scheme_bytes: self
                .state
                .slots
                .values()
                .filter_map(|slot| slot.node.as_ref())
                .map(|node| match &node.kind {
                    DomNodeKind::Document(metadata) => metadata.meta_color_scheme_bytes(),
                    _ => 0,
                })
                .sum(),
            autofocus_candidates: self
                .state
                .slots
                .values()
                .filter_map(|s| s.node.as_ref())
                .map(|n| match &n.autofocus_document {
                    Some(canonical::Autofocus::Known { candidates, .. }) => candidates.len(),
                    _ => 0,
                })
                .sum(),
            slot_references: self.state.slot_reference_count(),
            transitions,
            records,
            pseudos,
            nodes: self.state.nodes,
            units: self.state.units,
            history_entries: self.history.entries(),
            history_bytes: self.history.bytes(),
        }
    }
    pub fn snapshot(&self) -> DomSnapshot {
        DomSnapshot {
            source_tail: self
                .source_operation
                .as_ref()
                .filter(|operation| operation.has_event_tail())
                .map(|operation| operation.id()),
            owner: self.owner,
            revision: self.revision,
            state: self.state.clone(),
        }
    }
    /// Live bounds cannot fall below admitted state. Smaller history bounds
    /// discard coverage explicitly, without changing semantic revision.
    /// Retained event observations must settle before these bounds can change.
    pub fn set_limits(&mut self, limits: DomLimits) -> Result<(), DomError> {
        // This legacy policy controller supports the original single staged
        // Source recovery, but not event-observation reentry. Refuse before
        // traversing canonical banks without the observing owners' Work.
        if !self.source_parents.is_empty()
            || self
                .source_operation
                .as_ref()
                .is_some_and(|operation| operation.is_observing_event())
        {
            return Err(DomError::SourceOperationPending);
        }
        admit(self.state.ranges.len(), limits.ranges, DomResource::Ranges)?;
        admit(
            self.fullscreen_exits.len(),
            limits.fullscreen_exits,
            DomResource::FullscreenExits,
        )?;
        let events = self
            .state
            .slots
            .values()
            .filter_map(|slot| slot.node.as_ref())
            .try_fold(0usize, |count, node| {
                count
                    .checked_add(node.fullscreen_events.len())
                    .ok_or(DomError::RepresentationExhausted)
            })?;
        admit(
            events,
            limits.fullscreen_events,
            DomResource::FullscreenEvents,
        )?;
        if let Some(op) = &self.source_operation {
            op.admit_dom_limits(limits)?;
        }
        for parent in &self.source_parents {
            parent.admit_dom_limits(limits)?;
        }
        let mut encoding_url_records = 0usize;
        let mut records = 0usize;
        for slot in &self.state.slots {
            if let Some(node) = &slot.node {
                encoding_url_records = encoding_url_records
                    .checked_add(
                        node.encoding_url_document
                            .as_ref()
                            .map_or(0, crate::hyperlink::EncodingUrlDocument::records),
                    )
                    .ok_or(DomError::RepresentationExhausted)?;
                records = records
                    .checked_add(node.base_document.as_ref().map_or(0, |document| {
                        document
                            .facts
                            .as_ref()
                            .map_or(0, |facts| facts.formation_usage().record_capacity())
                    }))
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        admit(
            records,
            limits.base_observations,
            DomResource::BaseObservations,
        )?;
        admit(
            encoding_url_records,
            limits.encoding_url_observations,
            DomResource::EncodingUrlObservations,
        )?;
        admit(
            self.state.slot_reference_count(),
            limits.slot_references,
            DomResource::SlotReferences,
        )?;
        admit(
            self.usage().autofocus_candidates(),
            limits.autofocus_candidates,
            DomResource::AutofocusCandidates,
        )?;
        admit(self.state.nodes, limits.nodes, DomResource::ManagedNodes)?;
        admit(self.state.units, limits.units, DomResource::StringCodeUnits)?;
        admit(
            self.usage().meta_color_scheme_retained_bytes(),
            limits.meta_color_scheme_bytes,
            DomResource::MetaColorSchemeRetainedBytes,
        )?;
        let (transitions, records, pseudos) = self.state.view_transitions.usage();
        admit(
            transitions,
            limits.transitions,
            DomResource::ViewTransitions,
        )?;
        admit(records, limits.records, DomResource::ViewTransitionRecords)?;
        admit(pseudos, limits.pseudos, DomResource::ViewTransitionPseudos)?;
        admit(
            self.state
                .native_tasks
                .iter()
                .filter(|t| matches!(t, select::Task::Update { .. }))
                .count(),
            limits.select_tasks,
            DomResource::SelectTasks,
        )?;
        admit(
            self.state.details_intentions.len(),
            limits.details_intentions,
            DomResource::DetailsToggleIntentions,
        )?;
        let cancel = AtomicBool::new(false);
        let custody_formation = match &self.source_operation {
            Some(op) if self.dispatch_custody.has_indexed_scopes() => {
                match op.dispatch_limit_formation(&cancel)? {
                    Some(formation) => Some(formation),
                    None => self.source_observation_formation(&cancel)?,
                }
            }
            _ => None,
        };
        self.dispatch_custody.admit_limits(
            limits.dispatch_custody,
            custody_formation.as_ref(),
            &cancel,
        )?;
        if let Some(op) = &mut self.source_operation {
            op.update_dom_limits(limits);
        }
        self.limits = limits;
        self.trim_history();
        Ok(())
    }
    pub fn apply(
        &mut self,
        batch: DomBatch,
        cancelled: &AtomicBool,
    ) -> Result<DomCommit, DomError> {
        let result = (|| {
            let prepared = self.prepare(batch, cancelled)?;
            self.publish(prepared, cancelled)
        })();
        #[cfg(test)]
        self.observe_source_lifecycle();
        result
    }
    /// Canonical whole-batch input policy, before any command is resolved or
    /// staged. Each caller supplies its actual cancellation/work admission.
    fn admit_batch_inputs(
        &self,
        commands: &[Command],
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        admit(commands.len(), self.limits.commands, DomResource::Commands)?;
        let mut incoming = 0usize;
        for command in commands {
            step()?;
            if let Command::AssignSlot {
                incoming: nodes, ..
            } = command
            {
                incoming = incoming
                    .checked_add(nodes.len())
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(
                    incoming,
                    self.limits.slot_assignment_inputs,
                    DomResource::SlotAssignmentInputs,
                )?;
            }
        }
        Ok(())
    }
    fn prepare(&self, batch: DomBatch, cancelled: &AtomicBool) -> Result<Prepared, DomError> {
        self.require_source_entry()?;
        check_cancel(cancelled)?;
        self.admit_batch_inputs(&batch.commands, || check_cancel(cancelled))?;
        let mut draft = if self
            .source_operation
            .as_ref()
            .is_some_and(|op| op.permits_reentry())
        {
            self.source_observed_batch(
                batch.owner,
                &batch.commands,
                batch.commands.capacity(),
                cancelled,
            )?
        } else {
            Draft::new(self, batch.owner, cancelled)?
        };
        for command in batch.commands {
            check_cancel(cancelled)?;
            if let Err(error) = draft.apply(&command, self.limits, cancelled) {
                if let DomError::DialogProcessingUnavailable { action } = error
                    && let Some(pending) = draft
                        .pending_dialog_hook()
                        .filter(|pending| pending.action == action)
                {
                    return Err(DomError::NeedsEventfulOperation {
                        subject: draft.published_id(pending.subject),
                        action,
                    });
                }
                return Err(error);
            }
        }
        check_cancel(cancelled)?;
        let mut prepared = draft.finish(self, cancelled)?;
        // An ordinary callback batch does not complete its observing Source.
        // Keep that owner's already-published marker and closing obligation;
        // only its original Source publication can advance or clear the marker.
        prepared.state.incomplete_source = self.state.incomplete_source;
        Ok(prepared)
    }
    fn publish(
        &mut self,
        prepared: Prepared,
        cancelled: &AtomicBool,
    ) -> Result<DomCommit, DomError> {
        let publication = self.prepare_publication(prepared, cancelled)?;
        Ok(self.publish_prepared(publication))
    }
    fn prepare_publication(
        &mut self,
        mut prepared: Prepared,
        cancelled: &AtomicBool,
    ) -> Result<PreparedPublication, DomError> {
        check_cancel(cancelled)?;
        let usage = prepared.state.root_usage()?;
        transaction::nodes::resize_debit(&mut prepared.root_debit, usage, |usage| match &prepared
            .formation
        {
            Some(budget) => budget.admit_custody_usage(usage, cancelled),
            None => check_cancel(cancelled),
        })?;
        let receipt = prepared
            .root_debit
            .share(|usage| match &prepared.formation {
                Some(budget) => budget.admit_custody_usage(usage, cancelled),
                None => check_cancel(cancelled),
            })?;
        let guarded = PreparedRoot {
            value: prepared,
            receipt,
        };
        let state_custody = transaction::StateCustody::from_parts_retained(
            &guarded.value.state,
            guarded.value.state_frontiers.clone(),
            &self.custody_registry,
            &guarded.receipt,
            |usage| match &guarded.value.formation {
                Some(budget) => budget.admit_custody_usage(usage, cancelled),
                None => check_cancel(cancelled),
            },
        )?;
        let changed = guarded.value.changed_from(&self.state);
        let Prepared {
            formation,
            state_frontiers: _,
            root_debit: _,
            semantic_changed: _,
            signal_ack,
            native_ack,
            details_ack,
            transition_ack,
            _transition_outputs,
            created_ranges,
            retirements,
            state,
            allocation_ids,
            owner,
            created,
            documents,
            transition_ids,
            record_ids,
            changes,
        } = guarded.value;
        let revision = if changed {
            let protected = self.source_reserved_closing(cancelled)?;
            self.revision
                .0
                .checked_add(1)
                .and_then(|next| next.checked_add(self.hosted_revisions))
                .and_then(|next| next.checked_add(protected))
                .ok_or(DomError::RevisionExhausted)?;
            DomRevision(
                self.revision
                    .0
                    .checked_add(1)
                    .ok_or(DomError::RevisionExhausted)?,
            )
        } else {
            self.revision
        };
        let from = DomStamp {
            owner: self.owner,
            revision: self.revision,
        };
        let to = DomStamp {
            owner: self.owner,
            revision,
        };
        if let Some(budget) = &formation {
            // Vec -> Arc<[T]> allocates the destination while the admitted
            // Vec backing is still live. State's inline fields likewise move
            // into a new Arc allocation before the old canonical Arc retires.
            budget.retain_bytes(
                changes
                    .len()
                    .checked_mul(size_of::<DomChange>())
                    .ok_or(DomError::RepresentationExhausted)?,
                cancelled,
            )?;
        }
        let old_changes_bytes = changes
            .capacity()
            .checked_mul(size_of::<DomChange>())
            .ok_or(DomError::RepresentationExhausted)?;
        let changes: Arc<[DomChange]> = changes.into();
        #[cfg(test)]
        if scale_metrics::ENABLED && !changes.is_empty() {
            let bytes = changes.len() * size_of::<DomChange>();
            scale_metrics::record(scale_metrics::Event::StructuralAllocation { bytes });
            scale_metrics::record(scale_metrics::Event::SequenceMaterialized { bytes });
        }
        if let Some(budget) = &formation {
            budget.release_payload(source::storage::Payload {
                bytes: old_changes_bytes,
                units: 0,
            });
        }
        let history_plan = self.history.prepare(
            self,
            &changes,
            changed,
            |usage| match &formation {
                Some(budget) => budget.admit_custody_usage(usage, cancelled),
                None => check_cancel(cancelled),
            },
            || source::formation::step(formation.as_ref(), cancelled),
        )?;
        let (state, state_custody) = if changed {
            let state = Arc::new(state);
            #[cfg(test)]
            if scale_metrics::ENABLED {
                scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                    bytes: size_of::<State>(),
                });
            }
            (state, state_custody)
        } else {
            drop(state);
            drop(state_custody);
            (self.state.clone(), self.state_custody.clone())
        };
        let retired_authority = self.active_task.upgrade().filter(|auth| {
            changes.iter().any(|change| matches!(change,
                DomChange::SelectTaskCancelled { operation, select, reason: crate::DomSelectTaskCancellationReason::TargetDestroyed }
                if *operation == auth.id && *select == auth.target
            ))
        });
        let retirement_publication = self.dispatch_custody.prepare_publication(
            &retirements,
            to,
            self.limits.dispatch_custody,
            formation.as_ref(),
            cancelled,
        )?;
        // This independently held candidate is prepared last: its registry
        // owns its actual peak, and this non-retaining gate includes all prior
        // publication storage before the final fence.
        let style_custody = self.style_cssom_roots.prepare_publication(
            changes.iter().filter_map(|change| match change {
                DomChange::Destroyed(id) => Some(*id),
                _ => None,
            }),
            changes.iter().filter_map(|change| match change {
                DomChange::NodeDocument(id) => Some((
                    *id,
                    state
                        .node(self.owner, *id)
                        .expect("NodeDocument change names a validated live candidate")
                        .document,
                )),
                _ => None,
            }),
            |extra| match &formation {
                Some(budget) => budget.admit_custody_usage(extra, cancelled),
                None => check_cancel(cancelled),
            },
            || source::formation::step(formation.as_ref(), cancelled),
        )?;
        // Last cancellation observation is the publication boundary. No user
        // callbacks or external effects occur while prepared state is private.
        check_cancel(cancelled)?;
        Ok(PreparedPublication {
            state_custody,
            allocation_ids,
            signal_ack,
            native_ack,
            details_ack,
            transition_ack,
            _transition_outputs,
            created_ranges,
            state,
            changed,
            revision,
            history_plan,
            retirement_publication,
            style_custody,
            retired_authority,
            owner,
            created,
            documents,
            transition_ids,
            record_ids,
            from,
            to,
            changes,
            _formation: formation,
        })
    }
    fn publish_prepared(&mut self, publication: PreparedPublication) -> DomCommit {
        self.publish_prepared_with_proof(publication).commit
    }
    fn publish_prepared_with_proof(
        &mut self,
        publication: PreparedPublication,
    ) -> CommittedPublication {
        let PreparedPublication {
            state_custody,
            allocation_ids: _,
            signal_ack: _,
            native_ack: _,
            details_ack: _,
            transition_ack: _,
            _transition_outputs: _,
            created_ranges,
            state,
            changed,
            revision,
            history_plan,
            retirement_publication,
            style_custody,
            retired_authority,
            owner,
            created,
            documents,
            transition_ids,
            record_ids,
            from,
            to,
            changes,
            _formation,
        } = publication;
        if changed {
            #[cfg(test)]
            if scale_metrics::ENABLED {
                scale_metrics::record(scale_metrics::Event::PublishedRoot);
            }
            self.state = state;
            self.state_custody = state_custody;
            if let Some(style_custody) = style_custody {
                self.style_cssom_roots = style_custody;
            }
            self.revision = revision;
            self.history.commit(history_plan);
        }
        retirement_publication.publish();
        if let Some(auth) = retired_authority {
            auth.destroy_if_pending();
            self.active_task = std::sync::Weak::new();
        }
        CommittedPublication {
            committed: transaction::PublicationCommitted::at_fence(),
            commit: DomCommit {
                created_ranges,
                owner,
                created,
                documents,
                transition_ids,
                record_ids,
                changes: DomChanges { from, to, changes },
            },
        }
    }
    fn trim_history(&mut self) {
        self.history.trim(self.limits);
    }
    /// Complete chronological change records, or an explicit recompute result.
    pub fn changes_since(&self, baseline: DomStamp) -> Result<DomChanges, DomHistoryError> {
        if baseline.owner != self.owner {
            return Err(DomHistoryError::ForeignOwner);
        }
        if baseline.revision > self.revision {
            return Err(DomHistoryError::FutureRevision);
        }
        let changes = self
            .history
            .changes_since(baseline.revision, self.revision)?;
        Ok(DomChanges {
            from: baseline,
            to: DomStamp {
                owner: self.owner,
                revision: self.revision,
            },
            changes,
        })
    }
}

pub(crate) fn admit(value: usize, limit: usize, resource: DomResource) -> Result<(), DomError> {
    if value > limit {
        Err(DomError::ResourceLimit { resource, limit })
    } else {
        Ok(())
    }
}
pub(crate) fn check_cancel(cancelled: &AtomicBool) -> Result<(), DomError> {
    if cancelled.load(Ordering::Acquire) {
        Err(DomError::Cancelled)
    } else {
        Ok(())
    }
}
pub(crate) fn reserve<T>(values: &mut Vec<T>, additional: usize) -> Result<(), DomError> {
    #[cfg(test)]
    let before = (values.capacity(), values.as_ptr(), values.len());
    values
        .try_reserve(additional)
        .map_err(|_| DomError::AllocationFailed)?;
    #[cfg(test)]
    if scale_metrics::ENABLED && values.capacity() != before.0 {
        scale_metrics::record(scale_metrics::Event::StructuralAllocation {
            bytes: values.capacity() * size_of::<T>(),
        });
        if values.as_ptr() != before.1 {
            let mut moved = CopyObservation::default();
            moved.extended::<T>(before.2);
        }
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Key {
    Existing(NodeId),
    Created(usize),
}
struct DraftNode {
    fields: custody::Held<Box<transaction::NodeFields>>,
    children: transaction::Sequence<NodeId, Key>,
    manual_nodes: transaction::Sequence<NodeId, Key>,
    assigned_nodes: transaction::Sequence<NodeId, Key>,
    fullscreen_events: transaction::Sequence<crate::DomFullscreenEvent, crate::DomFullscreenEvent>,
    render_blocking: transaction::Sequence<NodeId, Key>,
    autofocus_candidates: transaction::Sequence<NodeId, Key>,
    open_dialogs: transaction::Sequence<NodeId, Key>,
    top_layer: transaction::Sequence<NodeId, Key>,
    pending_top_layer_removals: transaction::Sequence<NodeId, Key>,
}
impl std::ops::Deref for DraftNode {
    type Target = transaction::NodeFields;
    fn deref(&self) -> &Self::Target {
        self.fields.value()
    }
}

struct Draft {
    base: Arc<State>,
    dom_limits: DomLimits,
    base_custody: transaction::StateCustody,
    input_captures: bank::Bank<source::input::InputEntry>,
    input_captures_custody: bank::BankCustody<source::input::InputEntry>,
    custody_registry: bank::CustodyRegistry,
    journal: transaction::Journal<transaction::Undo>,
    fullscreen_exit: Option<fullscreen::exit::Frame>,
    // An actual published parallel invocation retains this same Work atom.
    fullscreen_work_custody: Option<crate::DomFullscreenExitId>,
    ranges: transaction::ranges::Ranges,
    range_tickets: Vec<crate::DomRangeId>,
    range_tickets_charge: custody::Debit,
    owner: Arc<BatchIdentity>,
    store: DomOwnerId,
    registries: Option<Arc<registry::Bank>>,
    hosted_globals: Option<Arc<hosted::GlobalDocumentIndex>>,
    hosted_history: Option<Arc<hosted::history::Graph>>,
    auxiliary: transaction::auxiliary::Auxiliary,
    hosted_owner: Option<crate::hosted::DomHostedOperationId>,
    basis: DomStamp,
    source_limits: Option<crate::DomSourceLimits>,
    // These ledgers are shared only by private temporaries of this exclusive
    // owner. Relaxed numerical loads/stores need no publication ordering; Arc
    // and atomics allow the complete parked owner to move between threads.
    source_work: Arc<AtomicU64>,
    source_ancestors: Option<Arc<crate::source::event::EventWork>>,
    source_formation_bytes: Arc<AtomicUsize>,
    source_temporary_units: Arc<AtomicUsize>,
    source_other_bytes: usize,
    source_other_units: usize,
    source_invocation: u64,
    source_headroom: Option<source::headroom::Scan>,
    method: Option<method::Frame>,
    dispatch_retirements: Vec<(NodeId, Key)>,
    dispatch_retirements_charge: custody::Debit,
    phase_map: Option<source::PhaseMap>,
    base_work: std::cell::Cell<u64>,
    base_work_limit: u64,
    encoding_url_work: std::cell::Cell<u64>,
    encoding_url_work_limit: u64,
    meta_work: std::cell::Cell<u64>,
    meta_work_limit: u64,
    view_transitions: DraftTransitions,
    signal_slots: transaction::queues::Signals,
    notification_queued: bool,
    cancelled_signals: transaction::queues::CancelledSignals,
    native_tasks: transaction::queues::NativeTasks,
    cancelled_selects: transaction::queues::CancelledSelects,
    details_intentions: transaction::queues::Details,
    cancelled_details: transaction::queues::CancelledDetails,
    cancelled_images: Option<select::ScratchVec<image::Cancelled>>,
    effects: Vec<select::EffectFrame>,
    effects_charge: custody::Debit,
    hosted_pending: Option<source::operation::hosted::ChildHook>,
    effects_running: bool,
    effect_floor: usize,
    effect_bytes: usize,
    effect_scoped: bool,
    select_work: std::cell::Cell<u64>,
    select_work_limit: u64,
    select_scratch_limit: usize,
    select_temporary_bytes: Arc<AtomicUsize>,
    select_semantic_work: std::cell::Cell<usize>,
    select_semantic_bytes: std::cell::Cell<usize>,
    existing: bank::Bank<usize>,
    existing_custody: bank::BankCustody<usize>,
    records: Vec<Option<DraftNode>>,
    records_charge: custody::Debit,
    created: Vec<Option<DraftNode>>,
    created_charge: custody::Debit,
    created_dirty: bank::Bank<bool>,
    created_dirty_custody: bank::BankCustody<bool>,
    counts: transaction::Counts,
    inbound: transaction::inbound::Inbound,
    controls: transaction::controls::Controls,
    slot_index: transaction::slot_index::Slots,
    slot_names: transaction::slot_names::Names,
    families: transaction::family_discovery::Families,
    role_facts: transaction::role_facts::Roles,
    containers: hosted::containers::Containers,
    private_nodes: usize,
    private_units: usize,
    private_counts: transaction::Counts,
    // Caller creation tickets exclude private allocations such as imported
    // descendants. They must not be interpreted as allocation indices.
    tickets: Vec<Key>,
    tickets_charge: custody::Debit,
    nodes: usize,
    units: usize,
    document_environment: crate::DomDocumentEnvironment,
    input_processing: input_processing::InputProcessing,
    #[cfg(test)]
    insertion_trace: Option<tree::InsertionTrace>,
    #[cfg(test)]
    svg_processing_trace: Option<svg_script::ProcessingTrace>,
    source_work_custody: source::formation::SourceWorkCustody,
    source_mirrors: source::formation::SourceMirrors,
    source_ancestor_custody: Option<Arc<source::formation::EventWorkCustody>>,
    _header_custody: custody::Debit,
}
pub(crate) struct PreparedPublication {
    created_ranges: Vec<Option<crate::DomRangeId>>,
    state: Arc<State>,
    changed: bool,
    revision: DomRevision,
    history_plan: history::Plan,
    retirement_publication: dispatch::PublicationPlan,
    style_custody: Option<style_custody::Bank>,
    retired_authority: Option<Arc<crate::select::Authorization>>,
    owner: Arc<BatchIdentity>,
    created: Vec<Option<NodeId>>,
    documents: Vec<bool>,
    transition_ids: Vec<Option<DomViewTransitionId>>,
    record_ids: Vec<Option<DomViewTransitionRecordId>>,
    from: DomStamp,
    to: DomStamp,
    changes: Arc<[DomChange]>,
    state_custody: transaction::StateCustody,
    allocation_ids: source::AllocationIds,
    signal_ack: Option<transaction::queues::QueueAck<Key, NodeId>>,
    native_ack: Option<transaction::queues::QueueAck<select::Pending, select::Task<NodeId>>>,
    details_ack: Option<
        transaction::queues::QueueAck<details::Pending<Key>, crate::DomDetailsToggleIntention>,
    >,
    transition_ack: view_transition::TransitionAck,
    _transition_outputs: custody::Debit,
    _formation: Option<source::formation::Formation>,
}
struct Prepared {
    state: State,
    created_ranges: Vec<Option<crate::DomRangeId>>,
    retirements: Vec<dispatch::Retirement>,
    owner: Arc<BatchIdentity>,
    created: Vec<Option<NodeId>>,
    documents: Vec<bool>,
    transition_ids: Vec<Option<DomViewTransitionId>>,
    record_ids: Vec<Option<DomViewTransitionRecordId>>,
    changes: Vec<DomChange>,
    semantic_changed: bool,
    allocation_ids: source::AllocationIds,
    signal_ack: Option<transaction::queues::QueueAck<Key, NodeId>>,
    native_ack: Option<transaction::queues::QueueAck<select::Pending, select::Task<NodeId>>>,
    details_ack: Option<
        transaction::queues::QueueAck<details::Pending<Key>, crate::DomDetailsToggleIntention>,
    >,
    transition_ack: view_transition::TransitionAck,
    state_frontiers: transaction::StateFrontiers,
    root_debit: custody::Debit,
    _transition_outputs: custody::Debit,
    formation: Option<source::formation::Formation>,
}

struct PreparedRoot {
    value: Prepared,
    receipt: custody::SharedDebit,
}

impl Prepared {
    fn changed_from(&self, before: &State) -> bool {
        self.semantic_changed
            || self.state.incomplete_source != before.incomplete_source
            || self.state.incomplete_hosted != before.incomplete_hosted
            || !same_auxiliary(&self.state.registries, &before.registries)
            || !same_auxiliary(&self.state.hosted_globals, &before.hosted_globals)
            || !same_auxiliary(&self.state.hosted_history, &before.hosted_history)
    }
}

impl Draft {
    pub(in crate::store) fn source_work_custody(&self) -> source::formation::SourceWorkCustody {
        self.source_work_custody.clone()
    }
    pub(in crate::store) fn source_ancestor_custody(
        &self,
    ) -> Option<Arc<source::formation::EventWorkCustody>> {
        self.source_ancestor_custody.clone()
    }
    pub(in crate::store) fn source_mirrors(&self) -> source::formation::SourceMirrors {
        self.source_mirrors.clone()
    }

    pub(in crate::store) fn custody_ledger(&self) -> &custody::Ledger {
        self.custody_registry.ledger()
    }

    pub(in crate::store) fn custody_registry(&self) -> &bank::CustodyRegistry {
        &self.custody_registry
    }

    pub(in crate::store) fn controlled_storage_usage(
        &self,
        cancel: &AtomicBool,
    ) -> Result<custody::Usage, DomError> {
        self.source_step(cancel)?;
        self.custody_registry.reconcile_releases();
        Ok(self.custody_ledger().total())
    }

    pub(in crate::store) fn prepare_undo_slots(
        &self,
        count: usize,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.journal
            .prepare_slots(count, self.custody_ledger(), |usage| {
                self.admit_custody_usage(usage, cancel)
            })
    }
    pub(in crate::store) fn prepare_undo(&self, cancel: &AtomicBool) -> Result<bool, DomError> {
        self.journal.prepare(self.custody_ledger(), |usage| {
            self.admit_custody_usage(usage, cancel)
        })
    }

    pub(in crate::store) fn record_range_undo_prepared(
        &self,
        undo: transaction::ranges::RangeUndo,
    ) {
        self.journal.push_prepared(transaction::Undo::Range(undo));
    }
    pub(in crate::store) fn record_transition_undo_prepared(
        &self,
        undo: view_transition::TransitionUndo,
    ) {
        self.journal
            .push_prepared(transaction::Undo::Transition(undo));
    }
    pub(in crate::store) fn record_inbound_undo_prepared(
        &self,
        undo: transaction::inbound::InboundUndo,
    ) {
        self.journal.push_prepared(transaction::Undo::Inbound(undo));
    }
    pub(in crate::store) fn record_queue_undo_prepared(
        &self,
        undo: transaction::queues::QueueUndo,
    ) {
        self.journal.push_prepared(transaction::Undo::Queue(undo));
    }
    pub(in crate::store) fn record_source_undo_prepared(&self, undo: source::ControlUndo) {
        self.journal.push_prepared(transaction::Undo::Source(undo));
    }

    pub(in crate::store) fn journal_mark(
        &mut self,
        other_live_bytes: usize,
        other_units: usize,
        cancel: &AtomicBool,
    ) -> Result<transaction::JournalMark, DomError> {
        let additional = custody::Usage::new(other_live_bytes, other_units);
        self.journal.mark(self.custody_ledger(), |usage| {
            self.admit_custody_usage(usage.checked_add(additional)?, cancel)
        })
    }

    pub(in crate::store) fn rollback_journal(&mut self, mark: transaction::JournalMark) {
        let journal = self.journal.clone();
        journal.rollback(mark, |undo| undo.restore(self));
    }

    fn initialize_native_element(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.select_element_registry(key, cancel)?;
        self.initialize_template_contents(key, limits, cancel)?;
        if self.node(key)?.hyperlink.is_some() {
            self.set_hyperlink_url(key, None, limits, cancel)?;
        }
        if self.node(key)?.image.is_some() {
            self.start_image_update(
                key,
                surgeist_html::HtmlImageUpdateFlags::new(false, false),
                cancel,
            )?;
        }
        if matches!(&self.node(key)?.kind,DomNodeKind::Element(element) if element.input_state().is_some())
        {
            self.initialize_input(key, limits, cancel)?;
        }
        Ok(())
    }
    fn new(dom: &Dom, owner: Arc<BatchIdentity>, cancelled: &AtomicBool) -> Result<Self, DomError> {
        Self::new_formed(dom, owner, None, cancelled)
    }
    fn new_formed(
        dom: &Dom,
        owner: Arc<BatchIdentity>,
        initial: Option<&source::initial::Initial>,
        cancelled: &AtomicBool,
    ) -> Result<Self, DomError> {
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::Entered(
                scale_metrics::Domain::Transaction,
            ));
        }
        let registry = dom.custody_registry.clone();
        let ledger = registry.ledger();
        let mut admit_initial = |usage: custody::Usage| {
            source::initial::step(initial, cancelled)?;
            if let Some(initial) = initial {
                let total = usage.checked_add(initial.external_usage())?;
                let limits = initial.limits();
                admit(
                    total.bytes(),
                    limits.retained_bytes,
                    DomResource::SourceRetainedBytes,
                )?;
                admit(
                    total.units(),
                    limits.retained_units,
                    DomResource::SourceRetainedUnits,
                )?;
            }
            Ok(())
        };
        let header = ledger.retain(
            custody::Usage::new(size_of::<Draft>(), 0),
            &mut admit_initial,
        )?;
        let journal = transaction::Journal::new(ledger, &mut admit_initial)?;
        #[cfg(test)]
        journal.measure_payloads(transaction::Undo::journal_bytes);
        let source_mirrors =
            source::formation::SourceMirrors::allocate(&registry, &mut admit_initial)?;
        let (source_work, source_work_custody) = match initial {
            Some(initial) => initial.source_work(&registry, &mut admit_initial)?,
            None => {
                source::formation::SourceWorkCustody::allocate(0, &registry, &mut admit_initial)?
            }
        };
        let existing = bank::Bank::default();
        let existing_custody = bank::BankCustody::default();
        let records_charge = ledger.retain(custody::Usage::ZERO, &mut admit_initial)?;
        let created_charge = ledger.retain(custody::Usage::ZERO, &mut admit_initial)?;
        let effects_charge = ledger.retain(custody::Usage::ZERO, &mut admit_initial)?;
        let range_tickets_charge = ledger.retain(custody::Usage::ZERO, &mut admit_initial)?;
        let tickets_charge = ledger.retain(custody::Usage::ZERO, &mut admit_initial)?;
        let dispatch_retirements_charge =
            ledger.retain(custody::Usage::ZERO, &mut admit_initial)?;
        let view_transitions =
            DraftTransitions::capture(&dom.state.view_transitions, &dom.state_custody.transitions);
        let signal_slots = transaction::queues::Signals::capture(
            &dom.state.signal_slots,
            dom.state.signal_slots.len(),
            &dom.state.signal_targets,
            &dom.state_custody.signal_targets,
            &registry,
            &mut admit_initial,
        )?;
        let native_tasks = transaction::queues::NativeTasks::capture(
            &dom.state.native_tasks,
            dom.state.counts.select_tasks,
            &dom.state.native_targets,
            &dom.state_custody.native_targets,
            &registry,
            &mut admit_initial,
        )?;
        let details_intentions = transaction::queues::Details::capture(
            &dom.state.details_intentions,
            dom.state.details_intentions.len(),
            &dom.state.details_targets,
            &dom.state_custody.details_targets,
            &registry,
            &mut admit_initial,
        )?;
        let cancelled_signals =
            transaction::queues::CancelledSignals::empty(&registry, &mut admit_initial)?;
        let cancelled_selects =
            transaction::queues::CancelledSelects::empty(&registry, &mut admit_initial)?;
        let cancelled_details =
            transaction::queues::CancelledDetails::empty(&registry, &mut admit_initial)?;
        let ranges = transaction::ranges::Ranges::new(&dom.state, &dom.state_custody);
        Ok(Self {
            base: dom.state.clone(),
            dom_limits: dom.limits,
            base_custody: dom.state_custody.clone(),
            input_captures: bank::Bank::default(),
            input_captures_custody: bank::BankCustody::default(),
            custody_registry: registry,
            journal,
            _header_custody: header,
            ranges,
            range_tickets: Vec::new(),
            range_tickets_charge,
            owner,
            #[cfg(test)]
            svg_processing_trace: None,
            #[cfg(test)]
            insertion_trace: None,
            document_environment: dom.document_environment.clone(),
            input_processing: input_processing::InputProcessing::new(
                dom.limits.input_scratch,
                dom.limits.input_work,
            ),
            store: dom.owner,
            registries: dom.state.registries.clone(),
            hosted_globals: dom.state.hosted_globals.clone(),
            hosted_history: dom.state.hosted_history.clone(),
            auxiliary: dom.state_custody.auxiliary.clone(),
            hosted_owner: None,
            basis: DomStamp {
                owner: dom.owner,
                revision: dom.revision,
            },
            source_limits: initial.map(source::initial::Initial::limits),
            source_work,
            source_work_custody,
            source_ancestor_custody: initial.and_then(source::initial::Initial::ancestor_custody),
            source_ancestors: initial.and_then(|initial| initial.ancestors().cloned()),
            source_formation_bytes: source_mirrors.bytes(),
            source_temporary_units: source_mirrors.units(),
            source_other_bytes: 0,
            source_other_units: 0,
            source_invocation: 0,
            source_headroom: None,
            method: None,
            fullscreen_exit: None,
            fullscreen_work_custody: None,
            dispatch_retirements: Vec::new(),
            dispatch_retirements_charge,
            phase_map: None,
            base_work: std::cell::Cell::new(0),
            base_work_limit: dom.limits.base_work,
            encoding_url_work: std::cell::Cell::new(0),
            encoding_url_work_limit: dom.limits.encoding_url_work,
            meta_work: std::cell::Cell::new(0),
            meta_work_limit: dom.limits.meta_work,
            view_transitions,
            signal_slots,
            notification_queued: dom.state.notification_queued,
            cancelled_signals,
            native_tasks,
            cancelled_selects,
            details_intentions,
            cancelled_details,
            cancelled_images: None,
            effects: Vec::new(),
            effects_charge,
            hosted_pending: None,
            effects_running: false,
            effect_floor: 0,
            effect_bytes: 0,
            effect_scoped: false,
            select_work: std::cell::Cell::new(0),
            select_work_limit: dom.limits.select_work,
            select_scratch_limit: dom.limits.select_scratch,
            select_temporary_bytes: source_mirrors.temporary(),
            source_mirrors,
            select_semantic_work: std::cell::Cell::new(0),
            select_semantic_bytes: std::cell::Cell::new(0),
            existing,
            existing_custody,
            records: Vec::new(),
            records_charge,
            created_charge,
            created_dirty: bank::Bank::default(),
            created_dirty_custody: bank::BankCustody::default(),
            counts: dom.state.counts,
            inbound: transaction::inbound::Inbound::capture(
                &dom.state.inbound,
                &dom.state_custody.inbound,
            ),
            controls: transaction::controls::Controls::capture(
                &dom.state.controls,
                &dom.state_custody.controls,
            ),
            slot_index: transaction::slot_index::Slots::capture(
                &dom.state.slot_index,
                &dom.state_custody.slot_index,
            ),
            slot_names: transaction::slot_names::Names::capture(
                &dom.state.slot_index.named,
                &dom.state_custody.slot_index.named,
            ),
            families: transaction::family_discovery::Families::capture(
                &dom.state.family_index,
                &dom.state_custody.families,
            ),
            role_facts: transaction::role_facts::Roles::capture(
                &dom.state.role_facts,
                &dom.state_custody.role_facts,
            ),
            containers: hosted::containers::Containers::capture(
                &dom.state.container_index,
                &dom.state_custody.containers,
            ),
            private_nodes: 0,
            private_units: 0,
            private_counts: transaction::Counts::default(),
            created: Vec::new(),
            tickets: Vec::new(),
            tickets_charge,
            nodes: dom.state.nodes,
            units: dom.state.units,
        })
    }
    fn node(&self, key: Key) -> Result<transaction::NodeRead<'_>, DomError> {
        let key = self.live_key(key);
        let read = match key {
            Key::Existing(id) => {
                if let Some(index) = self.existing.get(id.slot) {
                    return self.records[*index]
                        .as_ref()
                        .map(|n| self.alias_node_read(transaction::NodeRead::staged(n)))
                        .ok_or(DomError::StaleNode);
                }
                self.base
                    .node(self.store, id)
                    .map(transaction::NodeRead::canonical)
            }
            Key::Created(index) => match self.created.get(index).and_then(Option::as_ref) {
                Some(node) => Ok(transaction::NodeRead::staged(node)),
                None if self.created_dirty.get(index).is_some() => Err(DomError::StaleNode),
                None => self
                    .phase_map
                    .as_ref()
                    .and_then(|m| m.ids.get(index))
                    .ok_or(DomError::StaleNode)
                    .and_then(|id| {
                        self.base
                            .node(self.store, id)
                            .map(transaction::NodeRead::canonical)
                    }),
            },
        }?;
        Ok(self.alias_node_read(read))
    }
    fn alias_node_read<'a>(&'a self, read: transaction::NodeRead<'a>) -> transaction::NodeRead<'a> {
        match &self.phase_map {
            Some(map) => read.aliases(&map.ids),
            None => read,
        }
    }

    #[cfg(test)]
    fn existing_key(&self, slot: usize) -> Result<Key, DomError> {
        let record = self.base.slots.get(slot).ok_or(DomError::StaleNode)?;
        let key = Key::Existing(NodeId {
            owner: self.store,
            slot,
            generation: record.generation,
        });
        self.node(key)?;
        Ok(key)
    }
    fn resolve(&self, operand: NodeOperand) -> Result<Key, DomError> {
        let key = match operand {
            NodeOperand::Existing(id) => {
                if id.owner != self.store {
                    return Err(DomError::ForeignOwner);
                }
                if let Some(map) = &self.phase_map
                    && let Some(index) = map.ids.original(id)
                {
                    let key = Key::Created(index);
                    self.node(key)?;
                    return Ok(key);
                }
                self.base.node(self.store, id)?;
                Key::Existing(id)
            }
            NodeOperand::Created(ticket) => {
                if !Arc::ptr_eq(&self.owner, &ticket.owner) {
                    return Err(DomError::ForeignBatch);
                }
                self.tickets
                    .get(ticket.index)
                    .copied()
                    .filter(|key| self.node(*key).is_ok())
                    .ok_or(DomError::UnavailableCreation)?
            }
        };
        self.node(key)?;
        Ok(key)
    }
    fn remove(
        &mut self,
        node: Key,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(parent) = self.node(node)?.parent {
            self.range_pre_remove(node, parent, cancelled)?;
            self.relations_retain(
                parent,
                transaction::RelationField::Children,
                |child| child != node,
                cancelled,
            )?;
            self.with_node_fields(node, cancelled, |fields| fields.parent = None)?;
            self.slot_removal(node, parent, limits, cancelled)?;
            self.push_effect(
                select::Effect::Tree(tree::Frame::RemovedParent(parent)),
                self.effect_scoped,
                cancelled,
            )?;
            self.native_removing_steps(node, parent, limits, cancelled)?;
        }
        Ok(())
    }
    fn allocate(
        &mut self,
        kind: DomNodeKind,
        document: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Key, DomError> {
        self.allocate_formed(kind, document, limits, cancel, None)
    }
    pub(in crate::store) fn allocate_formed(
        &mut self,
        kind: DomNodeKind,
        document: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
        formation: Option<source::formation::Formation>,
    ) -> Result<Key, DomError> {
        let debit = match formation.and_then(source::formation::Formation::into_debit) {
            Some(debit) => debit,
            None => self
                .custody_ledger()
                .retain(custody::Usage::ZERO, |usage| {
                    self.admit_custody_usage(usage, cancel)
                })?,
        };
        self.allocate_retained(custody::Held::new(kind, debit), document, limits, cancel)
    }
    pub(in crate::store) fn allocate_charged(
        &mut self,
        kind: DomNodeKind,
        document: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
        charge: Option<source::checkpoint::Charge>,
    ) -> Result<Key, DomError> {
        let debit = match charge.and_then(source::checkpoint::Charge::into_debit) {
            Some(debit) => debit,
            None => self
                .custody_ledger()
                .retain(custody::Usage::ZERO, |usage| {
                    self.admit_custody_usage(usage, cancel)
                })?,
        };
        self.allocate_retained(custody::Held::new(kind, debit), document, limits, cancel)
    }
    fn allocate_retained(
        &mut self,
        kind: custody::Held<DomNodeKind>,
        document: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Key, DomError> {
        let nodes = self
            .nodes
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(nodes, limits.nodes, DomResource::ManagedNodes)?;
        let units = self
            .units
            .checked_add(kind_units(kind.value())?)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        self.reserve_created_arena(cancel)?;
        self.source_admit(0, cancel)?;
        let key = Key::Created(self.created.len());
        let reflected_targets = match kind.value() {
            DomNodeKind::Element(element) if element.button_state().is_some() => {
                Some(crate::reflected::ReflectedTargets::button())
            }
            DomNodeKind::Element(element) if element.input_state().is_some() => {
                Some(crate::reflected::ReflectedTargets::input())
            }
            _ => None,
        };
        let related_formation = source::formation::Formation::new(self, cancel)?;
        if let Some(budget) = &related_formation {
            if matches!(kind.value(), DomNodeKind::Element(element) if element.form_state().is_some())
            {
                budget.retain_bytes(size_of::<Vec<crate::form::FormPastName<Key>>>(), cancel)?;
            }
            if matches!(kind.value(), DomNodeKind::Element(element) if element.is_img()) {
                budget.retain_bytes(size_of::<Vec<u16>>(), cancel)?;
            }
        }
        let form_past_names =
            matches!(kind.value(), DomNodeKind::Element(element) if element.form_state().is_some())
                .then(crate::form::FormPastNames::fresh);
        let interactive_document = match kind.value() {
            DomNodeKind::Document(metadata) => Some(
                if matches!(
                    metadata.creation_cause(),
                    crate::DomDocumentCreationCause::Native
                        | crate::DomDocumentCreationCause::HostedInitialAboutBlank { .. }
                ) {
                    crate::dialog::DocumentInteractiveState::native_closed()
                } else {
                    crate::dialog::DocumentInteractiveState::Unavailable
                },
            ),
            _ => None,
        };
        let autofocus_document = match kind.value() {
            DomNodeKind::Document(metadata) => Some(
                if matches!(
                    metadata.creation_cause(),
                    crate::DomDocumentCreationCause::Native
                        | crate::DomDocumentCreationCause::HostedInitialAboutBlank { .. }
                ) {
                    crate::autofocus::DocumentAutofocusState::fresh()
                } else {
                    crate::autofocus::DocumentAutofocusState::Unavailable
                },
            ),
            _ => None,
        };
        let template_document = matches!(kind.value(), DomNodeKind::Document(_))
            .then_some(crate::template::TemplateDocument::Ordinary { inert: None });
        let script =
            matches!(kind.value(), DomNodeKind::Element(element) if element.is_html_script())
                .then(crate::script::ScriptState::fresh);
        let base = matches!(kind.value(),DomNodeKind::Element(e) if e.is_base())
            .then(crate::base::BaseState::fresh);
        let link = matches!(kind.value(),DomNodeKind::Element(e) if e.is_link())
            .then(crate::base::LinkState::fresh);
        let image = matches!(kind.value(),DomNodeKind::Element(e) if e.is_img())
            .then(|| crate::image::ImageState::fresh(key));
        let hyperlink = matches!(kind.value(),DomNodeKind::Element(e) if e.is_hyperlink())
            .then(crate::hyperlink::HyperlinkState::fresh);
        let encoding_url_document = matches!(kind.value(), DomNodeKind::Document(_))
            .then(crate::hyperlink::EncodingUrlDocument::fresh);
        let base_document =
            matches!(kind.value(), DomNodeKind::Document(_)).then(crate::base::BaseDocument::fresh);
        let relevant_global = match kind.value() {
            DomNodeKind::Document(metadata) => metadata
                .document_environment()
                .hosted
                .as_ref()
                .map(|association| association.global),
            _ => self
                .document_metadata(document)?
                .document_environment()
                .hosted
                .as_ref()
                .map(|association| association.global),
        };
        let mut kind = kind;
        if let Some(incoming) = related_formation.and_then(source::formation::Formation::into_debit)
        {
            kind.debit_mut()
                .absorb(incoming)
                .expect("same new-record ledger");
        }
        let (kind, incoming) = kind.into_parts();
        let fields = transaction::NodeFields {
            active_range: if matches!(
                &kind,
                DomNodeKind::Document(metadata) if matches!(
                    metadata.creation_cause(),
                    crate::DomDocumentCreationCause::Native
                        | crate::DomDocumentCreationCause::HostedInitialAboutBlank { .. }
                )
            ) {
                crate::DomActiveRangeState::Known(None)
            } else {
                crate::DomActiveRangeState::Unavailable
            },
            relevant_global,
            document,
            kind,
            base,
            link,
            image,
            hyperlink,
            encoding_url_document,
            interactive_document,
            autofocus_document,
            base_document,
            parent: None,
            shadow_root: None,
            host: None,
            template_contents: None,
            template_patch: None,
            template_document,
            script,
            style_parser_document: None,
            manual_slot: None,
            assigned_slot: None,
            reflected_targets,
            control_owner: None,
            option_select: None,
            form_past_names: form_past_names
                .map(|v| transaction::Names::staged(v, || self.source_step(cancel)))
                .transpose()?,
        };
        let ledger = self.custody_ledger().clone();
        let node = DraftNode::fresh(fields, incoming, &ledger, |usage| {
            self.admit_custody_usage(usage, cancel)
        })?;
        let counts =
            self.counts
                .checked_add(transaction::Counts::node(transaction::NodeRead::staged(
                    &node,
                ))?)?;
        self.admit_node_counts(counts, nodes, units, limits)?;
        self.set_created_dirty(self.created.len(), true, cancel)?;
        let undo = self.prepare_undo_slots(8, cancel)?;
        let inbound =
            self.prepare_inbound_node(key, transaction::NodeRead::staged(&node), true, cancel)?;
        let controls = self.prepare_control_created(key, node.fields.value(), cancel)?;
        let slots = self.prepare_slot_created(key, node.fields.value(), cancel)?;
        let node_read = transaction::NodeRead::staged(&node);
        let order = slots
            .as_ref()
            .map_or_else(|| self.order_view(), |plan| plan.order(self))
            .changed(key, node_read);
        let discovery = self.prepare_discovery_node(
            transaction::NodeChange {
                key,
                before: None,
                after: Some(node_read),
            },
            order,
            cancel,
        )?;
        self.commit_inbound(inbound);
        if let Some(plan) = controls {
            self.commit_control_plan(plan);
        }
        if let Some(plan) = slots {
            self.commit_slot_plan(plan);
        }
        self.commit_discovery(discovery, undo);
        let oldcounts = self.counts;
        let oldnodes = self.nodes;
        let oldunits = self.units;
        self.created.push(Some(node));
        #[cfg(test)]
        {
            let mut copied = transaction::CopyObservation::default();
            copied.pushed::<Option<DraftNode>>();
        }
        self.counts = counts;
        if undo {
            self.journal.push_prepared(transaction::Undo::Created {
                len: self.created.len() - 1,
                counts: oldcounts,
                nodes: oldnodes,
                units: oldunits,
            });
        }
        self.nodes = nodes;
        self.units = units;
        Ok(key)
    }
    fn register_ticket(&mut self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        let undo = self.prepare_undo_slots(2, cancel)?;
        if let Some(next) = self.prepare_output_buffer(&self.tickets, 1, cancel)? {
            let (values, debit) = next.into_parts();
            let old = std::mem::replace(&mut self.tickets, values);
            let debit = std::mem::replace(&mut self.tickets_charge, debit);
            let before = custody::Held::new(old, debit);
            if undo {
                self.journal
                    .push_prepared(transaction::Undo::TicketsCapacity(before));
            }
        }
        let len = self.tickets.len();
        self.source_step(cancel)?;
        self.tickets.push(key);
        #[cfg(test)]
        {
            let mut copied = transaction::CopyObservation::default();
            copied.pushed::<Key>();
        }
        if undo {
            self.journal
                .push_prepared(transaction::Undo::TicketsLength(len));
        }
        Ok(())
    }
    fn apply(
        &mut self,
        command: &Command,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        match command {
            Command::ExitFullscreen { document } => {
                let document = self.resolve(document.clone())?;
                self.exit_fullscreen_boundary(document, true, cancelled)?;
            }
            Command::FullscreenElementModel { node } => {
                let node = self.resolve(node.clone())?;
                self.fullscreen_element_model(node, cancelled)?;
            }
            Command::UnfullscreenElementModel { node } => {
                let node = self.resolve(node.clone())?;
                self.unfullscreen_element_model(node, cancelled)?;
            }
            Command::UnfullscreenDocumentModel { document } => {
                let document = self.resolve(document.clone())?;
                self.unfullscreen_document_model(document, cancelled)?;
            }
            Command::SetDesignMode { document, value } => {
                let document = self.resolve(document.clone())?;
                self.design_mode_boundary(document, value, cancelled)?;
            }
            Command::CreateRange { document, ticket } => {
                let document = self.resolve(document.clone())?;
                self.create_range(document, *ticket, limits, cancelled)?;
            }
            Command::SetRangeStart {
                range,
                container,
                offset,
            }
            | Command::SetRangeEnd {
                range,
                container,
                offset,
            } => {
                let id = self.resolve_range(range)?;
                let container = self.resolve(container.clone())?;
                self.set_range_boundary(
                    id,
                    container,
                    *offset,
                    matches!(command, Command::SetRangeStart { .. }),
                    cancelled,
                )?;
            }
            Command::SetActiveRange { document, range } => {
                let document = self.resolve(document.clone())?;
                let range = range
                    .as_ref()
                    .map(|operand| self.resolve_range(operand))
                    .transpose()?;
                self.set_active_range(document, range, cancelled)?;
            }
            Command::RetireRange(range) => {
                let range = self.resolve_range(range)?;
                self.retire_range(range, cancelled)?;
            }
            Command::BindEncodingUrlFacts { basis, facts } => {
                self.bind_encoding_url_facts(*basis, facts, limits, cancelled)?
            }
            Command::BindBaseFacts { basis, facts } => {
                self.bind_base_facts(*basis, facts, limits, cancelled)?
            }
            Command::Select { node, edit } => {
                let key = self.resolve(node.clone())?;
                self.select_edit(key, edit, limits, cancelled)?;
            }
            Command::SetStyleDisabled { node, disabled } => {
                let node = self.resolve(node.clone())?;
                self.set_style_disabled(node, *disabled, limits, cancelled)?;
            }
            Command::Output { node, edit } => {
                let node = self.resolve(node.clone())?;
                self.output_edit(node, edit, limits, cancelled)?;
            }
            Command::SetTextareaValue { node, value } => {
                let node = self.resolve(node.clone())?;
                self.set_textarea_value(node, value, limits, cancelled)?;
            }
            Command::SetTextareaDefaultValue { node, value } => {
                let node = self.resolve(node.clone())?;
                self.set_textarea_default_value(node, value, limits, cancelled)?;
            }
            Command::ResetTextarea { node } => {
                let node = self.resolve(node.clone())?;
                self.reset_textarea(node, limits, cancelled)?;
            }
            Command::SetInputValue { node, value } => {
                let node = self.resolve(node.clone())?;
                self.set_input_value(node, value, limits, cancelled)?;
            }
            Command::SetInputChecked { node, value } => {
                let node = self.resolve(node.clone())?;
                self.set_input_checked(node, *value, cancelled)?;
            }
            Command::SetInputIndeterminate { node, value } => {
                let node = self.resolve(node.clone())?;
                self.set_input_indeterminate(node, *value, cancelled)?;
            }
            Command::ResetInput { node } => {
                let node = self.resolve(node.clone())?;
                self.reset_input(node, limits, cancelled)?;
            }
            Command::SetReflectedTarget { node, key, target } => {
                let node = self.resolve(node.clone())?;
                let target = target
                    .as_ref()
                    .map(|target| self.resolve(target.clone()))
                    .transpose()?;
                self.set_reflected_target(node, *key, target, limits, cancelled)?;
            }
            Command::AssignSlot { slot, incoming } => {
                let slot = self.resolve(slot.clone())?;
                self.assign_manual(slot, incoming, limits, cancelled)?;
            }
            Command::CreateXmlDocument {
                associated_document,
                namespace,
                qualified_name,
                doctype,
            } => {
                let associated_document = self.resolve(associated_document.clone())?;
                self.create_xml_document(
                    associated_document,
                    namespace,
                    qualified_name,
                    doctype.as_ref(),
                    limits,
                    cancelled,
                )?;
            }
            Command::CreateHtmlDocument {
                associated_document,
                title,
            } => {
                let associated_document = self.resolve(associated_document.clone())?;
                let key = self.create_html_document(
                    associated_document,
                    title.as_ref(),
                    limits,
                    cancelled,
                )?;
                self.register_ticket(key, cancelled)?;
            }
            Command::AttachShadow { host, init } => {
                let host = self.resolve(host.clone())?;
                let root = self.attach_shadow(host, *init, limits, cancelled)?;
                self.register_ticket(root, cancelled)?;
            }
            Command::ViewTransition { basis, edit } => {
                self.apply_view_transition(*basis, edit, limits, cancelled)?
            }
            Command::Create { kind, document } => {
                let key = Key::Created(self.created.len());
                let document = if let Some(document) = document {
                    let document = self.resolve(document.clone())?;
                    if !matches!(self.node(document)?.kind, DomNodeKind::Document(_)) {
                        return Err(DomError::WrongNodeKind);
                    }
                    document
                } else {
                    key
                };
                let nodes = self
                    .nodes
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(nodes, limits.nodes, DomResource::ManagedNodes)?;
                let raw_units = match &kind {
                    Creation::Document => Some(NATIVE_DOCUMENT_STRING_UNITS),
                    Creation::Fragment => Some(0),
                    Creation::Element(name) => Some(name.len()),
                    // A QName separator is not stored in prefix/local fields.
                    // This cheap lower bound cannot reject an exact-capacity
                    // prefixed name; allocate admits the complete actual kind.
                    Creation::ElementNs { namespace, name } => name
                        .len()
                        .saturating_sub(1)
                        .checked_add(namespace.uri().map_or(0, DocumentString::len)),
                    Creation::Text(data) | Creation::Comment(data) => Some(data.len()),
                    Creation::ProcessingInstruction { target, data } => {
                        target.len().checked_add(data.len())
                    }
                    Creation::DocumentType {
                        name,
                        public_id,
                        system_id,
                    } => name
                        .len()
                        .checked_add(public_id.len())
                        .and_then(|units| units.checked_add(system_id.len())),
                }
                .ok_or(DomError::RepresentationExhausted)?;
                admit(
                    self.units
                        .checked_add(raw_units)
                        .ok_or(DomError::RepresentationExhausted)?,
                    limits.units,
                    DomResource::StringCodeUnits,
                )?;
                #[cfg(test)]
                let metadata_growth = if matches!(kind, Creation::Document) {
                    source::formation::mutation_observation::before(
                        self,
                        source::formation::mutation_observation::Site::Metadata,
                        cancelled,
                    )?
                } else {
                    None
                };
                let metadata_formation = source::formation::Formation::new(self, cancelled)?;
                let kind = match kind {
                    Creation::Document => DomNodeKind::Document(DomDocumentMetadata::native(
                        self.document_environment.clone(),
                        &mut |text| {
                            source::formation::string(text, metadata_formation.as_ref(), cancelled)
                        },
                    )?),
                    Creation::Fragment => DomNodeKind::DocumentFragment,
                    Creation::Element(name) => {
                        let DomNodeKind::Document(metadata) = &self.node(document)?.kind else {
                            unreachable!("checked Document")
                        };
                        DomNodeKind::Element(DomElement::native(
                            name,
                            metadata,
                            cancelled,
                            metadata_formation.as_ref(),
                        )?)
                    }
                    Creation::ElementNs { namespace, name } => {
                        DomNodeKind::Element(DomElement::native_ns(
                            namespace,
                            name,
                            self.document_metadata(document)?.document_environment(),
                            cancelled,
                            metadata_formation.as_ref(),
                        )?)
                    }
                    Creation::Text(data) => DomNodeKind::Text(source::formation::share(
                        data,
                        metadata_formation.as_ref(),
                        cancelled,
                    )?),
                    Creation::Comment(data) => DomNodeKind::Comment(source::formation::share(
                        data,
                        metadata_formation.as_ref(),
                        cancelled,
                    )?),
                    Creation::ProcessingInstruction { target, data } => {
                        #[cfg(test)]
                        let _source_observation = self
                            .source_limits
                            .filter(|_| crate::node_values::source_observation::active())
                            .map(|limits| {
                                let ledger = self.custody_ledger().clone();
                                let bytes = self.source_other_bytes;
                                let units = self.source_other_units;
                                crate::node_values::source_observation::bind(
                                    move || {
                                        let live = ledger.total();
                                        (bytes + live.bytes(), units + live.units())
                                    },
                                    limits.retained_bytes,
                                    limits.retained_units,
                                )
                            });
                        DomNodeKind::ProcessingInstruction(DomProcessingInstruction::new(
                            target,
                            data,
                            metadata_formation.as_ref(),
                            cancelled,
                        )?)
                    }
                    Creation::DocumentType {
                        name,
                        public_id,
                        system_id,
                    } => DomNodeKind::DocumentType(DomDocumentType::new(
                        name,
                        public_id,
                        system_id,
                        metadata_formation.as_ref(),
                        cancelled,
                    )?),
                };
                #[cfg(test)]
                if let Some(before) = metadata_growth {
                    let payload = source::storage::kind(&kind, || self.source_step(cancelled))?;
                    source::formation::mutation_observation::grew(Some(before), payload.bytes);
                }
                let key =
                    self.allocate_formed(kind, document, limits, cancelled, metadata_formation)?;
                self.initialize_native_element(key, limits, cancelled)?;
                self.register_ticket(key, cancelled)?;
            }
            Command::Import {
                snapshot,
                source,
                document,
                subtree,
            } => {
                let document = self.resolve(document.clone())?;
                if !matches!(self.node(document)?.kind, DomNodeKind::Document(_)) {
                    return Err(DomError::WrongNodeKind);
                }
                let key =
                    self.import_node(snapshot, *source, document, *subtree, limits, cancelled)?;
                self.register_ticket(key, cancelled)?;
            }
            Command::Detach(node) => {
                let node = self.resolve(node.clone())?;
                self.remove(node, limits, cancelled)?;
            }
            Command::Adopt { node, document } => {
                let node = self.resolve(node.clone())?;
                let document = self.resolve(document.clone())?;
                if !matches!(self.node(document)?.kind, DomNodeKind::Document(_)) {
                    return Err(DomError::WrongNodeKind);
                }
                if matches!(self.node(node)?.kind, DomNodeKind::Document(_)) {
                    return Err(DomError::NotSupported);
                }
                if matches!(self.node(node)?.kind, DomNodeKind::ShadowRoot(_)) {
                    return Err(DomError::HierarchyRequest);
                }
                self.adopt(node, document, limits, cancelled)?;
            }
            Command::SetAttribute {
                node,
                namespace,
                name,
                value,
            } => {
                let node = self.resolve(node.clone())?;
                self.set_attribute(node, namespace.as_ref(), name, value, limits, cancelled)?;
            }
            Command::SetCustomValidity { node, value } => {
                let node = self.resolve(node.clone())?;
                self.set_custom_validity(node, value, limits, cancelled)?;
            }
            Command::RequestCloseDialog { node, result } => {
                let key = self.resolve(node.clone())?;
                self.dialog_method_boundary(
                    key,
                    dialog::MethodInput::RequestClose(result.as_ref()),
                    limits,
                    cancelled,
                )?;
            }
            Command::CloseDialog { node, result } => {
                let key = self.resolve(node.clone())?;
                self.dialog_method_boundary(
                    key,
                    dialog::MethodInput::Close(result.as_ref()),
                    limits,
                    cancelled,
                )?;
            }
            Command::ShowModalDialog { node } => {
                let key = self.resolve(node.clone())?;
                self.dialog_method_boundary(
                    key,
                    dialog::MethodInput::ShowModal,
                    limits,
                    cancelled,
                )?;
            }
            Command::ShowDialog { node } => {
                let key = self.resolve(node.clone())?;
                self.dialog_method_boundary(key, dialog::MethodInput::Show, limits, cancelled)?;
            }
            Command::SetDialogReturnValue { node, value } => {
                let node = self.resolve(node.clone())?;
                self.set_dialog_return_value(node, value, limits, cancelled)?;
            }
            Command::SetNonce { node, value } => {
                let node = self.resolve(node.clone())?;
                self.set_nonce(node, value, limits, cancelled)?;
            }
            Command::SetMarqueeRunning { node, running } => {
                let node = self.resolve(node.clone())?;
                let mut element = self.element_fields_mut(node, cancelled)?;
                element.set_marquee_running(*running)?;
                element.commit(cancelled)?;
            }
            Command::RemoveAttribute {
                node,
                namespace,
                name,
            } => {
                let node = self.resolve(node.clone())?;
                self.remove_attribute(node, namespace.as_ref(), name, limits, cancelled)?;
            }
            Command::Destroy(node) => {
                let node = self.resolve(node.clone())?;
                if self.node(node)?.parent.is_some()
                    || self.node(node)?.host.is_some()
                    || matches!(
                        self.node(node)?.kind,
                        DomNodeKind::Document(_) | DomNodeKind::ShadowRoot(_)
                    )
                {
                    return Err(DomError::InvalidDestruction);
                }
                let mut pending = self.effect_vec();
                let mut closure = self.effect_vec();
                self.select_reserve(&mut pending, 1, cancelled)?;
                pending.push(node);
                while !pending.is_empty() {
                    self.source_step(cancelled)?;
                    let key = pending.pop().expect("admitted destruction node");
                    let current = self.node(key)?;
                    let contents = self.owned_template_contents(key, cancelled)?;
                    self.select_reserve(
                        &mut pending,
                        current.children.len()
                            + usize::from(current.shadow_root.is_some())
                            + usize::from(contents.is_some()),
                        cancelled,
                    )?;
                    self.select_reserve(&mut closure, 1, cancelled)?;
                    for child in current.children.iter().rev() {
                        self.source_step(cancelled)?;
                        pending.push(child);
                    }
                    pending.extend(current.shadow_root);
                    pending.extend(contents);
                    closure.push(key);
                }
                self.push_effect(
                    select::Effect::Tree(tree::Frame::Destroy {
                        closure,
                        index: 0,
                        stage: 0,
                    }),
                    self.effect_scoped,
                    cancelled,
                )?;
                self.run_effects(limits, cancelled)?;
            }
            Command::ReplaceData {
                node,
                offset,
                count,
                data,
            } => {
                let node = self.resolve(node.clone())?;
                let (offset, count) = (*offset, *count);
                let previous_units = kind_units(self.node(node)?.kind)?;
                let fixed_units = match &self.node(node)?.kind {
                    DomNodeKind::ProcessingInstruction(value) => value.target().len(),
                    _ => 0,
                };
                let previous = match &self.node(node)?.kind {
                    DomNodeKind::Text(data) | DomNodeKind::Comment(data) => data,
                    DomNodeKind::ProcessingInstruction(value) => value.data(),
                    _ => return Err(DomError::WrongNodeKind),
                };
                if offset > previous.len() {
                    return Err(DomError::IndexSize);
                }
                let removed = count.min(previous.len() - offset);
                let length = (previous.len() - removed)
                    .checked_add(data.len())
                    .ok_or(DomError::RepresentationExhausted)?;
                let raw_units = (self.units - previous_units)
                    .checked_add(fixed_units)
                    .and_then(|units| units.checked_add(length))
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(raw_units, limits.units, DomResource::StringCodeUnits)?;
                #[cfg(test)]
                let growth = source::formation::mutation_observation::before(
                    self,
                    source::formation::mutation_observation::Site::ReplaceData,
                    cancelled,
                )?;
                let formation = source::formation::Formation::new(self, cancelled)?;
                let mut value = match &formation {
                    Some(formation) => source::formation::utf16_buffer(
                        length,
                        limits.units,
                        Some(formation),
                        cancelled,
                    )?,
                    None => {
                        let mut output = Vec::new();
                        reserve(&mut output, length)?;
                        output
                    }
                };
                #[cfg(test)]
                let mut copied = transaction::CopyObservation::default();
                for unit in previous.code_units()[..offset]
                    .iter()
                    .chain(data.code_units())
                    .chain(&previous.code_units()[offset + removed..])
                {
                    self.source_step(cancelled)?;
                    value.push(*unit);
                    #[cfg(test)]
                    copied.pushed::<u16>();
                }
                let value = DocumentString::from_utf16(value);
                #[cfg(test)]
                source::formation::mutation_observation::grew(
                    growth,
                    size_of::<Vec<u16>>() + value.shared_code_units().capacity() * size_of::<u16>(),
                );
                let kind = match &self.node(node)?.kind {
                    DomNodeKind::Text(_) => DomNodeKind::Text(value),
                    DomNodeKind::Comment(_) => DomNodeKind::Comment(value),
                    DomNodeKind::ProcessingInstruction(instruction) => {
                        let instruction =
                            instruction.with_data(&value, formation.as_ref(), cancelled)?;
                        let mut temporary = source::storage::Payload::default();
                        temporary.string(&value)?;
                        drop(value);
                        if let Some(budget) = &formation {
                            budget.release_payload(temporary);
                        }
                        DomNodeKind::ProcessingInstruction(instruction)
                    }
                    _ => unreachable!("checked CharacterData kind"),
                };
                let units = (self.units - previous_units)
                    .checked_add(kind_units(&kind)?)
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(units, limits.units, DomResource::StringCodeUnits)?;
                self.with_node_fields(node, cancelled, |fields| fields.kind = kind)?;
                self.units = units;
                self.range_replace_data(node, offset, removed, data.len(), cancelled)?;
                drop(formation); // The actual replacement graph now belongs to Draft.
                if let Some(parent) = self.node(node)?.parent {
                    self.children_changed(parent, limits, cancelled)?;
                }
            }
            Command::Insert {
                node,
                parent,
                before,
            } => {
                let node = self.resolve(node.clone())?;
                let parent = self.resolve(parent.clone())?;
                let before = before
                    .as_ref()
                    .map(|id| self.resolve(id.clone()))
                    .transpose()?;
                self.pre_insert(node, parent, before, limits, cancelled)?;
            }
            Command::ReplaceChild {
                node,
                parent,
                child,
            } => {
                let node = self.resolve(node.clone())?;
                let parent = self.resolve(parent.clone())?;
                let child = self.resolve(child.clone())?;
                self.replace_child(node, parent, child, limits, cancelled)?;
            }
        }
        Ok(())
    }
    fn finish(&self, dom: &Dom, cancelled: &AtomicBool) -> Result<Prepared, DomError> {
        self.finish_with_mapping(dom, cancelled, std::iter::empty())
            .map(|(prepared, _)| prepared)
    }

    fn finish_with_mapping(
        &self,
        dom: &Dom,
        cancelled: &AtomicBool,
        requested: impl ExactSizeIterator<Item = Key>,
    ) -> Result<(Prepared, Vec<NodeId>), DomError> {
        self.finish_sparse(dom, cancelled, requested)
    }
}
fn kind_units(kind: &DomNodeKind) -> Result<usize, DomError> {
    match kind {
        DomNodeKind::Document(value) => value.units().ok_or(DomError::RepresentationExhausted),
        DomNodeKind::Element(value) => value.units().ok_or(DomError::RepresentationExhausted),
        DomNodeKind::Text(data) | DomNodeKind::Comment(data) => Ok(data.len()),
        DomNodeKind::ProcessingInstruction(value) => {
            value.units().ok_or(DomError::RepresentationExhausted)
        }
        DomNodeKind::DocumentType(value) => value.units().ok_or(DomError::RepresentationExhausted),
        _ => Ok(0),
    }
}
fn sparse_difference(
    owner: DomOwnerId,
    old: &State,
    new: &State,
    budget: Option<&source::formation::Formation>,
    cancelled: &AtomicBool,
    node_indices: &bank::Bank<bool>,
    range_indices: impl IntoIterator<Item = usize>,
) -> Result<Vec<DomChange>, DomError> {
    let mut changes = Vec::new();
    #[cfg(test)]
    let mut written = transaction::CopyObservation::default();
    for index in range_indices {
        check_cancel(cancelled)?;
        if let Some(budget) = budget {
            budget.retain_bytes(0, cancelled)?;
        }
        let before = old
            .ranges
            .get(index)
            .and_then(|slot| slot.record.as_ref().map(|record| (slot.generation, record)));
        let after = new
            .ranges
            .get(index)
            .and_then(|slot| slot.record.as_ref().map(|record| (slot.generation, record)));
        let same = before.zip(after).is_some_and(|(a, b)| a.0 == b.0);
        if let Some((generation, record)) = before {
            let id = crate::DomRangeId {
                owner,
                slot: index,
                generation,
            };
            if same {
                if record != after.expect("same Range").1 {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::RangeBoundary(id));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
            } else {
                source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                changes.push(DomChange::RangeRetired(id));
                #[cfg(test)]
                written.pushed::<DomChange>();
            }
        }
        if !same && let Some((generation, _)) = after {
            source::formation::reserve(&mut changes, 1, budget, cancelled)?;
            changes.push(DomChange::RangeCreated(crate::DomRangeId {
                owner,
                slot: index,
                generation,
            }));
            #[cfg(test)]
            written.pushed::<DomChange>();
        }
    }
    // Fullscreen facts depend on top-layer membership and flags of actual
    // members. Scalar Document writes do not enter that dependency domain.
    let scratch = budget.map(source::formation::Formation::fork);
    let mut fullscreen_documents = Vec::new();
    let mut selected_nodes = Vec::new();
    for (index, _) in node_indices.iter() {
        source::formation::reserve(&mut selected_nodes, 1, scratch.as_ref(), cancelled)?;
        selected_nodes.push(index);
        #[cfg(test)]
        written.pushed::<usize>();
        let before = old.slots.get(index).and_then(|s| s.node.as_ref());
        let after = new.slots.get(index).and_then(|s| s.node.as_ref());
        let flags = |node: Option<&Node>| {
            node.and_then(|n| match &n.kind {
                DomNodeKind::Element(e) => {
                    Some((n.document, e.fullscreen_flag(), e.keyboard_lock_flag()))
                }
                _ => None,
            })
        };
        let before = flags(before);
        let after = flags(after);
        if before != after && (before.is_some_and(|v| v.1) || after.is_some_and(|v| v.1)) {
            for doc in [before.map(|v| v.0), after.map(|v| v.0)]
                .into_iter()
                .flatten()
            {
                source::formation::reserve(
                    &mut fullscreen_documents,
                    1,
                    scratch.as_ref(),
                    cancelled,
                )?;
                fullscreen_documents.push(doc);
                #[cfg(test)]
                written.pushed::<DocumentId>();
                source::formation::reserve(&mut selected_nodes, 1, scratch.as_ref(), cancelled)?;
                selected_nodes.push(doc.node().slot);
                #[cfg(test)]
                written.pushed::<usize>();
            }
        }
    }
    fullscreen_documents.sort_unstable_by_key(|id| (id.node().slot, id.node().generation));
    fullscreen_documents.dedup();
    selected_nodes.sort_unstable();
    selected_nodes.dedup();
    for index in selected_nodes {
        let before = old
            .slots
            .get(index)
            .and_then(|slot| slot.node.as_ref().map(|node| (slot.generation, node)));
        let after = new
            .slots
            .get(index)
            .and_then(|slot| slot.node.as_ref().map(|node| (slot.generation, node)));
        let same = before.zip(after).is_some_and(|(a, b)| a.0 == b.0);
        if let Some((generation, node)) = before {
            let id = NodeId {
                owner,
                slot: index,
                generation,
            };
            if same {
                let current = after.expect("same live identity").1;
                if node.active_range != current.active_range {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentActiveRange(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if let (DomNodeKind::Document(before), DomNodeKind::Document(after)) =
                    (&node.kind, &current.kind)
                    && before.design_mode_enabled() != after.design_mode_enabled()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentDesignMode(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if let (DomNodeKind::Element(before), DomNodeKind::Element(after)) =
                    (&node.kind, &current.kind)
                    && before.captured_in_view_transition() != after.captured_in_view_transition()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::ViewTransitionCaptured(
                        DomViewTransitionSubject::Element(id),
                    ));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                let intrinsic_changed = match (&node.kind, &current.kind) {
                    (DomNodeKind::Element(before), DomNodeKind::Element(after)) => {
                        before.attributes() != after.attributes()
                    }
                    (DomNodeKind::Document(before), DomNodeKind::Document(after)) => {
                        !before.same_nondesign_mode_state(after)
                    }
                    _ => node.kind != current.kind,
                };
                if let (DomNodeKind::Element(before), DomNodeKind::Element(after)) =
                    (&node.kind, &current.kind)
                    && (!before.same_nonattribute_state(after)
                        || node.reflected_targets != current.reflected_targets
                        || node.control_owner != current.control_owner
                        || node.form_past_names != current.form_past_names
                        || node.base != current.base
                        || node.link != current.link
                        || node.image != current.image
                        || node.hyperlink != current.hyperlink
                        || node.script != current.script
                        || node.style_parser_document != current.style_parser_document
                        || node.option_select != current.option_select
                        || node.template_patch != current.template_patch)
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::ElementState(id));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if node.base != current.base
                    && let Some(frozen) = current.base.as_ref().and_then(|b| b.frozen.as_ref())
                {
                    let change = DomChange::DocumentBase(DocumentId(frozen.document));
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(change);
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if node.autofocus_document != current.autofocus_document {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentAutofocusState(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if node.interactive_document != current.interactive_document {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentInteractiveState(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if node.fullscreen_events != current.fullscreen_events {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentFullscreenEvents(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                let same_top_layer =
                    match (&node.interactive_document, &current.interactive_document) {
                        (
                            Some(canonical::Interactive::Known { top_layer: a, .. }),
                            Some(canonical::Interactive::Known { top_layer: b, .. }),
                        ) => a.same_storage(b),
                        (
                            Some(canonical::Interactive::Unavailable),
                            Some(canonical::Interactive::Unavailable),
                        )
                        | (None, None) => true,
                        _ => false,
                    };
                let fullscreen_dependency = fullscreen_documents
                    .binary_search_by_key(&(id.slot, id.generation), |doc| {
                        (doc.node().slot, doc.node().generation)
                    })
                    .is_ok()
                    || !same_top_layer;
                if matches!(node.kind, DomNodeKind::Document(_))
                    && fullscreen_dependency
                    && fullscreen::captured_facts(old, node, owner, budget, cancelled)?
                        != fullscreen::captured_facts(new, current, owner, budget, cancelled)?
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentFullscreenState(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if node.base_document != current.base_document {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentBase(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if node
                    .base_document
                    .as_ref()
                    .map(|state| &state.render_blocking)
                    != current
                        .base_document
                        .as_ref()
                        .map(|state| &state.render_blocking)
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentRenderBlocking(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if let (DomNodeKind::Document(before), DomNodeKind::Document(after)) =
                    (&node.kind, &current.kind)
                    && (before.meta_state() != after.meta_state()
                        || before.meta_theme_color() != after.meta_theme_color()
                        || before.meta_color_scheme != after.meta_color_scheme
                        || before.policy_container() != after.policy_container())
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentMeta(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                if node.encoding_url_document != current.encoding_url_document {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentEncodingUrlFacts(DocumentId(id)));
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
                let kind_change = if intrinsic_changed {
                    Some(
                        if matches!(
                            (&node.kind, &current.kind),
                            (DomNodeKind::Element(_), DomNodeKind::Element(_))
                        ) {
                            DomChange::Attributes(id)
                        } else {
                            DomChange::Data(id)
                        },
                    )
                } else {
                    None
                };
                for change in [
                    kind_change,
                    (node.manual_nodes != current.manual_nodes)
                        .then_some(DomChange::SlotManual(id)),
                    (node.assigned_nodes != current.assigned_nodes)
                        .then_some(DomChange::SlotAssignedNodes(id)),
                    (node.manual_slot != current.manual_slot)
                        .then_some(DomChange::ManualAssignment(id)),
                    (node.assigned_slot != current.assigned_slot)
                        .then_some(DomChange::StoredAssignedSlot(id)),
                    (node.shadow_root != current.shadow_root).then_some(DomChange::ShadowRoot(id)),
                    (node.document != current.document).then_some(DomChange::NodeDocument(id)),
                    (node.parent != current.parent).then_some(DomChange::Parent(id)),
                    (node.children != current.children).then_some(DomChange::Children(id)),
                ]
                .into_iter()
                .flatten()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(change);
                    #[cfg(test)]
                    written.pushed::<DomChange>();
                }
            } else {
                source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                changes.push(DomChange::Destroyed(id));
                #[cfg(test)]
                written.pushed::<DomChange>();
            }
        }
        if let Some((generation, _)) = after
            && !same
        {
            source::formation::reserve(&mut changes, 1, budget, cancelled)?;
            changes.push(DomChange::Created(NodeId {
                owner,
                slot: index,
                generation,
            }));
            #[cfg(test)]
            written.pushed::<DomChange>();
        }
    }
    let has_template = |state: &State| state.counts.templates != 0;
    let cache_changed = node_indices.iter().any(|(index, _)| {
        old.slots
            .get(index)
            .and_then(|slot| slot.node.as_ref())
            .zip(new.slots.get(index).and_then(|slot| slot.node.as_ref()))
            .is_some_and(|(a, b)| {
                a.template_document != b.template_document
                    || a.template_contents != b.template_contents
                    || a.template_patch != b.template_patch
            })
    });
    if cache_changed
        || ((has_template(old) || has_template(new))
            && changes.iter().any(|change| {
                matches!(
                    change,
                    DomChange::Created(_)
                        | DomChange::Destroyed(_)
                        | DomChange::Parent(_)
                        | DomChange::Children(_)
                        | DomChange::NodeDocument(_)
                )
            }))
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::TemplateRelations);
        #[cfg(test)]
        written.pushed::<DomChange>();
    }
    // All derived shadow relations are invalidated conservatively whenever a
    // structural edit can affect a managed shadow tree. Ordinary-only stores
    // retain their existing precise change contract.
    let has_shadow = |state: &State| state.counts.shadow_roots != 0;
    if (has_shadow(old) || has_shadow(new))
        && changes.iter().any(|change| {
            matches!(
                change,
                DomChange::Created(_)
                    | DomChange::Destroyed(_)
                    | DomChange::Parent(_)
                    | DomChange::Children(_)
                    | DomChange::NodeDocument(_)
                    | DomChange::ShadowRoot(_)
            )
        })
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::ShadowRelations);
        #[cfg(test)]
        written.pushed::<DomChange>();
    }
    if old.details_intentions != new.details_intentions
        || old.next_details_intention != new.next_details_intention
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::DetailsToggleIntentions);
        #[cfg(test)]
        written.pushed::<DomChange>();
    }
    if old.signal_slots != new.signal_slots
        || old.notification_queued != new.notification_queued
        || old.native_tasks != new.native_tasks
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::IntrinsicTasks);
        #[cfg(test)]
        written.pushed::<DomChange>();
    }
    if !changes.is_empty() && (old.counts.slots != 0 || new.counts.slots != 0) {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::SlotQueries);
        #[cfg(test)]
        written.pushed::<DomChange>();
    }
    Ok(changes)
}

#[cfg(test)]
fn full_difference(
    owner: DomOwnerId,
    old: &State,
    new: &State,
    cancelled: &AtomicBool,
) -> Result<Vec<DomChange>, DomError> {
    let budget: Option<&source::formation::Formation> = None;
    let mut changes = Vec::new();
    for index in 0..old.ranges.len().max(new.ranges.len()) {
        check_cancel(cancelled)?;
        if let Some(budget) = budget {
            budget.retain_bytes(0, cancelled)?;
        }
        let before = old
            .ranges
            .get(index)
            .and_then(|slot| slot.record.as_ref().map(|record| (slot.generation, record)));
        let after = new
            .ranges
            .get(index)
            .and_then(|slot| slot.record.as_ref().map(|record| (slot.generation, record)));
        let same = before.zip(after).is_some_and(|(a, b)| a.0 == b.0);
        if let Some((generation, record)) = before {
            let id = crate::DomRangeId {
                owner,
                slot: index,
                generation,
            };
            if same {
                if record != after.expect("same Range").1 {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::RangeBoundary(id));
                }
            } else {
                source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                changes.push(DomChange::RangeRetired(id));
            }
        }
        if !same && let Some((generation, _)) = after {
            source::formation::reserve(&mut changes, 1, budget, cancelled)?;
            changes.push(DomChange::RangeCreated(crate::DomRangeId {
                owner,
                slot: index,
                generation,
            }));
        }
    }
    for index in 0..old.slots.len().max(new.slots.len()) {
        let before = old
            .slots
            .get(index)
            .and_then(|slot| slot.node.as_ref().map(|node| (slot.generation, node)));
        let after = new
            .slots
            .get(index)
            .and_then(|slot| slot.node.as_ref().map(|node| (slot.generation, node)));
        let same = before.zip(after).is_some_and(|(a, b)| a.0 == b.0);
        if let Some((generation, node)) = before {
            let id = NodeId {
                owner,
                slot: index,
                generation,
            };
            if same {
                let current = after.expect("same live identity").1;
                if node.active_range != current.active_range {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentActiveRange(DocumentId(id)));
                }
                if let (DomNodeKind::Document(before), DomNodeKind::Document(after)) =
                    (&node.kind, &current.kind)
                    && before.design_mode_enabled() != after.design_mode_enabled()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentDesignMode(DocumentId(id)));
                }
                if let (DomNodeKind::Element(before), DomNodeKind::Element(after)) =
                    (&node.kind, &current.kind)
                    && before.captured_in_view_transition() != after.captured_in_view_transition()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::ViewTransitionCaptured(
                        DomViewTransitionSubject::Element(id),
                    ));
                }
                let intrinsic_changed = match (&node.kind, &current.kind) {
                    (DomNodeKind::Element(before), DomNodeKind::Element(after)) => {
                        before.attributes() != after.attributes()
                    }
                    (DomNodeKind::Document(before), DomNodeKind::Document(after)) => {
                        !before.same_nondesign_mode_state(after)
                    }
                    _ => node.kind != current.kind,
                };
                if let (DomNodeKind::Element(before), DomNodeKind::Element(after)) =
                    (&node.kind, &current.kind)
                    && (!before.same_nonattribute_state(after)
                        || node.reflected_targets != current.reflected_targets
                        || node.control_owner != current.control_owner
                        || node.form_past_names != current.form_past_names
                        || node.base != current.base
                        || node.link != current.link
                        || node.image != current.image
                        || node.hyperlink != current.hyperlink
                        || node.script != current.script
                        || node.style_parser_document != current.style_parser_document
                        || node.option_select != current.option_select
                        || node.template_patch != current.template_patch)
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::ElementState(id));
                }
                if node.base != current.base
                    && let Some(frozen) = current.base.as_ref().and_then(|b| b.frozen.as_ref())
                {
                    let change = DomChange::DocumentBase(DocumentId(frozen.document));
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(change);
                }
                if node.autofocus_document != current.autofocus_document {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentAutofocusState(DocumentId(id)));
                }
                if node.interactive_document != current.interactive_document {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentInteractiveState(DocumentId(id)));
                }
                if node.fullscreen_events != current.fullscreen_events {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentFullscreenEvents(DocumentId(id)));
                }
                if matches!(node.kind, DomNodeKind::Document(_))
                    && fullscreen::captured_facts(old, node, owner, budget, cancelled)?
                        != fullscreen::captured_facts(new, current, owner, budget, cancelled)?
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentFullscreenState(DocumentId(id)));
                }
                if node.base_document != current.base_document {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentBase(DocumentId(id)));
                }
                if node
                    .base_document
                    .as_ref()
                    .map(|state| &state.render_blocking)
                    != current
                        .base_document
                        .as_ref()
                        .map(|state| &state.render_blocking)
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentRenderBlocking(DocumentId(id)));
                }
                if let (DomNodeKind::Document(before), DomNodeKind::Document(after)) =
                    (&node.kind, &current.kind)
                    && (before.meta_state() != after.meta_state()
                        || before.meta_theme_color() != after.meta_theme_color()
                        || before.meta_color_scheme != after.meta_color_scheme
                        || before.policy_container() != after.policy_container())
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentMeta(DocumentId(id)));
                }
                if node.encoding_url_document != current.encoding_url_document {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(DomChange::DocumentEncodingUrlFacts(DocumentId(id)));
                }
                let kind_change = if intrinsic_changed {
                    Some(
                        if matches!(
                            (&node.kind, &current.kind),
                            (DomNodeKind::Element(_), DomNodeKind::Element(_))
                        ) {
                            DomChange::Attributes(id)
                        } else {
                            DomChange::Data(id)
                        },
                    )
                } else {
                    None
                };
                for change in [
                    kind_change,
                    (node.manual_nodes != current.manual_nodes)
                        .then_some(DomChange::SlotManual(id)),
                    (node.assigned_nodes != current.assigned_nodes)
                        .then_some(DomChange::SlotAssignedNodes(id)),
                    (node.manual_slot != current.manual_slot)
                        .then_some(DomChange::ManualAssignment(id)),
                    (node.assigned_slot != current.assigned_slot)
                        .then_some(DomChange::StoredAssignedSlot(id)),
                    (node.shadow_root != current.shadow_root).then_some(DomChange::ShadowRoot(id)),
                    (node.document != current.document).then_some(DomChange::NodeDocument(id)),
                    (node.parent != current.parent).then_some(DomChange::Parent(id)),
                    (node.children != current.children).then_some(DomChange::Children(id)),
                ]
                .into_iter()
                .flatten()
                {
                    source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                    changes.push(change);
                }
            } else {
                source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                changes.push(DomChange::Destroyed(id));
            }
        }
        if let Some((generation, _)) = after
            && !same
        {
            source::formation::reserve(&mut changes, 1, budget, cancelled)?;
            changes.push(DomChange::Created(NodeId {
                owner,
                slot: index,
                generation,
            }));
        }
    }
    let has_template = |state: &State| {
        state.slots.values().any(|slot| {
            slot.node.as_ref().is_some_and(|node| {
                node.template_contents.is_some()
                    || node.template_document.is_some_and(|state| {
                        matches!(state, crate::template::TemplateDocument::InertTemplate)
                            || state.inert().is_some()
                    })
            })
        })
    };
    let cache_changed = old.slots.values().zip(&new.slots).any(|(a, b)| {
        a.node.as_ref().zip(b.node.as_ref()).is_some_and(|(a, b)| {
            a.template_document != b.template_document
                || a.template_contents != b.template_contents
                || a.template_patch != b.template_patch
        })
    });
    if cache_changed
        || ((has_template(old) || has_template(new))
            && changes.iter().any(|change| {
                matches!(
                    change,
                    DomChange::Created(_)
                        | DomChange::Destroyed(_)
                        | DomChange::Parent(_)
                        | DomChange::Children(_)
                        | DomChange::NodeDocument(_)
                )
            }))
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::TemplateRelations);
    }
    // All derived shadow relations are invalidated conservatively whenever a
    // structural edit can affect a managed shadow tree. Ordinary-only stores
    // retain their existing precise change contract.
    let has_shadow = |state: &State| {
        state.slots.values().any(|slot| {
            slot.node
                .as_ref()
                .is_some_and(|node| matches!(node.kind, DomNodeKind::ShadowRoot(_)))
        })
    };
    if (has_shadow(old) || has_shadow(new))
        && changes.iter().any(|change| {
            matches!(
                change,
                DomChange::Created(_)
                    | DomChange::Destroyed(_)
                    | DomChange::Parent(_)
                    | DomChange::Children(_)
                    | DomChange::NodeDocument(_)
                    | DomChange::ShadowRoot(_)
            )
        })
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::ShadowRelations);
    }
    if old.details_intentions != new.details_intentions
        || old.next_details_intention != new.next_details_intention
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::DetailsToggleIntentions);
    }
    if old.signal_slots != new.signal_slots
        || old.notification_queued != new.notification_queued
        || old.native_tasks != new.native_tasks
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::IntrinsicTasks);
    }
    if !changes.is_empty()
        && [old, new].iter().any(|state| {
            state.slots.values().any(|s| {
                s.node
                    .as_ref()
                    .is_some_and(|n| crate::slot::is_slot(&n.kind))
            })
        })
    {
        source::formation::reserve(&mut changes, 1, budget, cancelled)?;
        changes.push(DomChange::SlotQueries);
    }
    new.view_transitions
        .changes(&old.view_transitions, &mut changes, budget, cancelled)?;
    // Complete pseudo origins derive from ordinary Document children. Report
    // that domain when a valid generic edit changes or removes their origin.
    for transition in new.view_transitions.transitions.values() {
        if old
            .view_transitions
            .transitions
            .values()
            .any(|t| t.id == transition.id)
        {
            let origin = |state: &State| {
                state
                    .node(owner, transition.document.node())
                    .ok()
                    .and_then(|document| {
                        document.children.iter().copied().find(|id| {
                            matches!(
                                state.node(owner, *id).map(|node| &node.kind),
                                Ok(DomNodeKind::Element(_))
                            )
                        })
                    })
            };
            let change = DomChange::ViewTransition(transition.id);
            if origin(old) != origin(new) && !changes.contains(&change) {
                source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                changes.push(change);
            }
        }
    }
    Ok(changes)
}

#[cfg(test)]
mod tests;

fn same_auxiliary<T: PartialEq>(left: &Option<Arc<T>>, right: &Option<Arc<T>>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b) || a == b,
        _ => false,
    }
}

impl Prepared {
    pub(in crate::store) fn set_hosted_history(
        &mut self,
        next: Option<Arc<hosted::history::Graph>>,
        incoming: Option<source::formation::Formation>,
        registry: &bank::CustodyRegistry,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let incoming = incoming.and_then(source::formation::Formation::into_debit);
        let (next, custody) = match next {
            Some(next) => {
                let plan = transaction::auxiliary::prepare_history(
                    next,
                    incoming,
                    registry,
                    &mut |usage| match &self.formation {
                        Some(budget) => budget.admit_custody_usage(usage, cancel),
                        None => check_cancel(cancel),
                    },
                    &mut || source::formation::step(self.formation.as_ref(), cancel),
                )?;
                let (value, custody) = plan.into_parts();
                (Some(value), Some(custody))
            }
            None => {
                drop(incoming);
                (None, None)
            }
        };
        drop((
            std::mem::replace(&mut self.state.hosted_history, next),
            std::mem::replace(&mut self.state_frontiers.auxiliary.history, custody),
        ));
        Ok(())
    }
    pub(in crate::store) fn set_hosted_globals(
        &mut self,
        next: Option<Arc<hosted::GlobalDocumentIndex>>,
        incoming: Option<source::formation::Formation>,
        registry: &bank::CustodyRegistry,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let incoming = incoming.and_then(source::formation::Formation::into_debit);
        let (next, custody) = match next {
            Some(next) => {
                let plan = transaction::auxiliary::prepare_globals(
                    next,
                    incoming,
                    registry,
                    &mut |usage| match &self.formation {
                        Some(budget) => budget.admit_custody_usage(usage, cancel),
                        None => check_cancel(cancel),
                    },
                    &mut || source::formation::step(self.formation.as_ref(), cancel),
                )?;
                let (value, custody) = plan.into_parts();
                (Some(value), Some(custody))
            }
            None => {
                drop(incoming);
                (None, None)
            }
        };
        drop((
            std::mem::replace(&mut self.state.hosted_globals, next),
            std::mem::replace(&mut self.state_frontiers.auxiliary.globals, custody),
        ));
        Ok(())
    }
}
