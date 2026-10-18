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
mod base;
mod children;
mod control;
mod design_mode;
mod details;
mod dialog;
mod dispatch;
mod document;
mod element_tasks;
mod focus;
mod form;
pub(crate) mod hosted;
mod hyperlink;
mod image;
mod import;
mod input;
mod input_processing;
mod installation;
mod meta;
mod method;
mod navigable;
mod output;
pub(crate) mod parsed_installation;
mod range;
mod reflected;
pub(crate) mod registry;
mod script;
mod select;
mod shadow;
mod slot;
pub(crate) mod source;
pub(crate) mod style;
mod svg_script;
mod template;
mod textarea;
mod tree;
mod view_transition;
use crate::view_transition::*;
use view_transition::DraftTransitions;

/// Configured capacity whose admission would be exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomResource {
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

#[derive(Debug, PartialEq)]
pub(crate) struct Node {
    pub(crate) active_range: crate::DomActiveRangeState,
    pub(crate) document: DocumentId,
    pub(crate) relevant_global: Option<crate::hosted::DomHostedGlobalId>,
    pub(crate) kind: DomNodeKind,
    pub(crate) parent: Option<NodeId>,
    pub(crate) children: Vec<NodeId>,
    pub(crate) shadow_root: Option<NodeId>,
    pub(crate) host: Option<NodeId>,
    pub(crate) template_contents: Option<NodeId>,
    pub(crate) template_patch: Option<crate::template::TemplatePatch<NodeId>>,
    pub(crate) template_document: Option<crate::template::TemplateDocument<NodeId>>,
    pub(crate) manual_nodes: Vec<NodeId>,
    pub(crate) assigned_nodes: Vec<NodeId>,
    pub(crate) manual_slot: Option<NodeId>,
    pub(crate) assigned_slot: Option<NodeId>,
    pub(crate) reflected_targets: Option<crate::reflected::ReflectedTargets<NodeId>>,
    pub(crate) control_owner: Option<NodeId>,
    pub(crate) option_select: Option<NodeId>,
    pub(crate) form_past_names: Option<crate::form::FormPastNames<NodeId>>,
    pub(crate) script: Option<crate::script::ScriptState<NodeId>>,
    pub(crate) style_parser_document: Option<NodeId>,
    pub(crate) base: Option<crate::base::BaseState<NodeId>>,
    pub(crate) base_document: Option<crate::base::BaseDocument<NodeId>>,
    pub(crate) link: Option<crate::base::LinkState<NodeId>>,
    pub(crate) image: Option<crate::image::ImageState<NodeId>>,
    pub(crate) hyperlink: Option<crate::hyperlink::HyperlinkState<NodeId>>,
    pub(crate) encoding_url_document: Option<crate::hyperlink::EncodingUrlDocument>,
    pub(crate) interactive_document: Option<crate::dialog::DocumentInteractiveState<NodeId>>,
    pub(crate) autofocus_document: Option<crate::autofocus::DocumentAutofocusState<NodeId>>,
}
#[derive(Debug, PartialEq)]
pub(crate) struct Slot {
    pub(crate) generation: u64,
    retired: bool,
    pub(crate) node: Option<Node>,
}
#[derive(Debug, PartialEq)]
pub(crate) struct State {
    pub(crate) ranges: Vec<crate::range::Slot<NodeId>>,
    pub(crate) incomplete_source: Option<crate::DomSourceOperationId>,
    pub(crate) incomplete_hosted: Option<crate::hosted::DomHostedOperationId>,
    pub(crate) slots: Vec<Slot>,
    pub(crate) registries: Option<Arc<registry::Bank>>,
    pub(crate) hosted_globals: Option<Arc<hosted::GlobalDocumentIndex>>,
    pub(crate) hosted_history: Option<Arc<hosted::history::Graph>>,
    pub(crate) view_transitions: ViewTransitions,
    pub(crate) signal_slots: Vec<NodeId>,
    pub(crate) notification_queued: bool,
    native_tasks: Vec<select::Task<NodeId>>,
    next_task: u64,
    pub(crate) details_intentions: Vec<crate::DomDetailsToggleIntention>,
    next_details_intention: u64,
    nodes: usize,
    units: usize,
}
impl State {
    pub(crate) fn node(&self, owner: DomOwnerId, id: NodeId) -> Result<&Node, DomError> {
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

struct HistoryEntry {
    from: DomRevision,
    changes: Arc<[DomChange]>,
    bytes: usize,
    older: Option<Box<HistoryEntry>>,
}
fn discard_history(mut history: Option<Box<HistoryEntry>>) {
    while let Some(mut entry) = history {
        history = entry.older.take();
    }
}

/// Exclusive canonical mutation owner. Snapshots never borrow its mutable state.
/// The whole owner is Send, including an exact parked source tail and retained
/// mutable CSSOM custody. Moving it does not replay or rebind that operation.
/// Shared mutation and Sync are not supplied by this exclusive-owner contract.
pub struct Dom {
    element_tasks: Option<std::sync::Weak<crate::element_tasks::Binding>>,
    dispatch_custody: dispatch::CustodyBank,
    owner: DomOwnerId,
    revision: DomRevision,
    limits: DomLimits,
    state: Arc<State>,
    history: Option<Box<HistoryEntry>>,
    history_entries: usize,
    history_bytes: usize,
    document_environment: crate::DomDocumentEnvironment,
    active_task: std::sync::Weak<crate::select::Authorization>,
    source_operation: Option<Box<source::operation::Operation>>,
    next_source_operation: u64,
    pub(in crate::store) source_parents: Vec<Box<source::operation::Operation>>,
    source_event_executor: Option<std::sync::Weak<crate::source::event::ExecutorBinding>>,
    style_cssom_roots: Vec<crate::style::DomStyleCssomRoot>,
    hosted_source: Option<crate::hosted::SourceObservation>,
    next_hosted_source: u64,
    details_transferred: bool,
    hosted_operations: hosted::operations::Operations,
    hosted_revisions: u64,
}
impl Drop for Dom {
    fn drop(&mut self) {
        discard_history(self.history.take());
    }
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
        Ok(Self {
            dispatch_custody: dispatch::CustodyBank::default(),
            element_tasks: None,
            owner: DomOwnerId::allocate()?,
            revision: DomRevision(0),
            limits,
            state: Arc::new(State {
                ranges: Vec::new(),
                incomplete_source: None,
                incomplete_hosted: None,
                slots: Vec::new(),
                registries: None,
                hosted_globals: None,
                hosted_history: None,
                view_transitions: ViewTransitions::new(),
                signal_slots: Vec::new(),
                notification_queued: false,
                native_tasks: Vec::new(),
                next_task: 0,
                details_intentions: Vec::new(),
                next_details_intention: 0,
                nodes: 0,
                units: 0,
            }),
            active_task: std::sync::Weak::new(),
            source_operation: None,
            next_source_operation: 0,
            source_parents: Vec::new(),
            source_event_executor: None,
            style_cssom_roots: Vec::new(),
            hosted_source: None,
            next_hosted_source: 0,
            details_transferred: false,
            hosted_operations: hosted::operations::Operations::default(),
            hosted_revisions: 0,
            history: None,
            history_entries: 0,
            history_bytes: 0,
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
                .iter()
                .filter(|slot| slot.record.is_some())
                .count(),
            meta_color_scheme_bytes: self
                .state
                .slots
                .iter()
                .filter_map(|slot| slot.node.as_ref())
                .map(|node| match &node.kind {
                    DomNodeKind::Document(metadata) => metadata.meta_color_scheme_bytes(),
                    _ => 0,
                })
                .sum(),
            autofocus_candidates: self
                .state
                .slots
                .iter()
                .filter_map(|s| s.node.as_ref())
                .map(|n| match &n.autofocus_document {
                    Some(crate::autofocus::DocumentAutofocusState::Known {
                        candidates, ..
                    }) => candidates.len(),
                    _ => 0,
                })
                .sum(),
            slot_references: self.state.slot_reference_count(),
            transitions,
            records,
            pseudos,
            nodes: self.state.nodes,
            units: self.state.units,
            history_entries: self.history_entries,
            history_bytes: self.history_bytes,
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
                    .checked_add(
                        node.base_document
                            .as_ref()
                            .map_or(0, crate::base::BaseDocument::records),
                    )
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
        let prepared = self.prepare(batch, cancelled)?;
        self.publish(prepared, cancelled)
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
        prepared: Prepared,
        cancelled: &AtomicBool,
    ) -> Result<PreparedPublication, DomError> {
        check_cancel(cancelled)?;
        let Prepared {
            created_ranges,
            retirements,
            mut state,
            formation,
            allocation_ids: _allocation_ids,
            owner,
            created,
            documents,
            transition_ids,
            record_ids,
            changes,
        } = prepared;
        // All chosen storage has already been admitted. Aggregate units are
        // derived allocation accounting, not a semantic mutation: compare every
        // other State field, then restore the candidate's actual usage. A whole
        // semantic no-op retains the old canonical backing and its truthful usage.
        let candidate_units = state.units;
        state.units = self.state.units;
        let changed = state != *self.state;
        state.units = candidate_units;
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
                    .and_then(|bytes| bytes.checked_add(size_of::<State>()))
                    .ok_or(DomError::RepresentationExhausted)?,
                cancelled,
            )?;
        }
        let changes: Arc<[DomChange]> = changes.into();
        let bytes = size_of::<HistoryEntry>()
            .checked_add(
                changes
                    .len()
                    .checked_mul(size_of::<DomChange>())
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .ok_or(DomError::RepresentationExhausted)?;
        let entry =
            if changed && self.limits.history_entries != 0 && bytes <= self.limits.history_bytes {
                if let Some(budget) = &formation {
                    budget.retain_bytes(size_of::<HistoryEntry>(), cancelled)?;
                }
                Some(Box::new(HistoryEntry {
                    from: self.revision,
                    changes: changes.clone(),
                    bytes,
                    older: None,
                }))
            } else {
                None
            };
        let state = Arc::new(state);
        let retired_authority = self.active_task.upgrade().filter(|auth| {
            changes.iter().any(|change| matches!(change,
                DomChange::SelectTaskCancelled { operation, select, reason: crate::DomSelectTaskCancellationReason::TargetDestroyed }
                if *operation == auth.id && *select == auth.target
            ))
        });
        // The retirement scan is admitted before the publication fence. The
        // matching post-publication retain only drops exact retired handles.
        for _ in &self.style_cssom_roots {
            if let Some(budget) = &formation {
                budget.retain_bytes(0, cancelled)?;
            } else {
                check_cancel(cancelled)?;
            }
        }
        let retirement_publication = self.dispatch_custody.prepare_publication(
            &retirements,
            to,
            self.limits.dispatch_custody,
            formation.as_ref(),
            cancelled,
        )?;
        // Last cancellation observation is the publication boundary. No user
        // callbacks or external effects occur while prepared state is private.
        check_cancel(cancelled)?;
        Ok(PreparedPublication {
            created_ranges,
            state,
            changed,
            revision,
            entry,
            retirement_publication,
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
        let PreparedPublication {
            created_ranges,
            state,
            changed,
            revision,
            entry,
            retirement_publication,
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
            self.state = state;
            // Exact generation retirement releases canonical custody. Detach,
            // adoption and historical snapshots do not retire the live root.
            self.style_cssom_roots
                .retain(|root| self.state.node(self.owner, root.root()).is_ok());
            self.revision = revision;
            if let Some(mut entry) = entry {
                entry.older = self.history.take();
                self.history = Some(entry);
                self.trim_history();
            } else {
                discard_history(self.history.take());
                self.history_entries = 0;
                self.history_bytes = 0;
            }
        }
        retirement_publication.publish();
        if let Some(auth) = retired_authority {
            auth.destroy_if_pending();
            self.active_task = std::sync::Weak::new();
        }
        DomCommit {
            created_ranges,
            owner,
            created,
            documents,
            transition_ids,
            record_ids,
            changes: DomChanges { from, to, changes },
        }
    }
    fn trim_history(&mut self) {
        let mut entries = 0usize;
        let mut bytes = 0usize;
        let mut cursor = &mut self.history;
        loop {
            let keep = cursor.as_ref().is_some_and(|entry| {
                entries < self.limits.history_entries
                    && entry.bytes <= self.limits.history_bytes - bytes
            });
            if !keep {
                discard_history(cursor.take());
                break;
            }
            let entry = cursor.as_mut().expect("retained entry");
            entries += 1;
            bytes += entry.bytes;
            cursor = &mut entry.older;
        }
        self.history_entries = entries;
        self.history_bytes = bytes;
    }
    /// Complete chronological change records, or an explicit recompute result.
    pub fn changes_since(&self, baseline: DomStamp) -> Result<DomChanges, DomHistoryError> {
        if baseline.owner != self.owner {
            return Err(DomHistoryError::ForeignOwner);
        }
        if baseline.revision > self.revision {
            return Err(DomHistoryError::FutureRevision);
        }
        let mut entries = Vec::new();
        let mut cursor = self.history.as_deref();
        let mut available_from = self.revision;
        while let Some(entry) = cursor {
            available_from = entry.from;
            if entry.from >= baseline.revision {
                entries
                    .try_reserve(1)
                    .map_err(|_| DomHistoryError::AllocationFailed)?;
                entries.push(entry);
            }
            cursor = entry.older.as_deref();
        }
        if baseline.revision < available_from {
            return Err(DomHistoryError::FullRecomputeRequired { available_from });
        }
        let mut changes = Vec::new();
        for entry in entries.into_iter().rev() {
            changes
                .try_reserve(entry.changes.len())
                .map_err(|_| DomHistoryError::AllocationFailed)?;
            changes.extend_from_slice(&entry.changes);
        }
        Ok(DomChanges {
            from: baseline,
            to: DomStamp {
                owner: self.owner,
                revision: self.revision,
            },
            changes: changes.into(),
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
    values
        .try_reserve(additional)
        .map_err(|_| DomError::AllocationFailed)
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Key {
    Existing(usize),
    Created(usize),
}
struct DraftNode {
    active_range: crate::DomActiveRangeState,
    relevant_global: Option<crate::hosted::DomHostedGlobalId>,
    document: Key,
    kind: DomNodeKind,
    parent: Option<Key>,
    children: Vec<Key>,
    shadow_root: Option<Key>,
    host: Option<Key>,
    template_contents: Option<Key>,
    template_patch: Option<crate::template::TemplatePatch<Key>>,
    template_document: Option<crate::template::TemplateDocument<Key>>,
    manual_nodes: Vec<Key>,
    assigned_nodes: Vec<Key>,
    manual_slot: Option<Key>,
    assigned_slot: Option<Key>,
    reflected_targets: Option<crate::reflected::ReflectedTargets<Key>>,
    control_owner: Option<Key>,
    option_select: Option<Key>,
    form_past_names: Option<crate::form::FormPastNames<Key>>,
    script: Option<crate::script::ScriptState<Key>>,
    pub(crate) style_parser_document: Option<Key>,
    base: Option<crate::base::BaseState<Key>>,
    base_document: Option<crate::base::BaseDocument<Key>>,
    link: Option<crate::base::LinkState<Key>>,
    image: Option<crate::image::ImageState<Key>>,
    hyperlink: Option<crate::hyperlink::HyperlinkState<Key>>,
    encoding_url_document: Option<crate::hyperlink::EncodingUrlDocument>,
    interactive_document: Option<crate::dialog::DocumentInteractiveState<Key>>,
    autofocus_document: Option<crate::autofocus::DocumentAutofocusState<Key>>,
}
struct Draft {
    ranges: Vec<crate::range::Slot<Key>>,
    range_tickets: Vec<crate::DomRangeId>,
    owner: Arc<BatchIdentity>,
    store: DomOwnerId,
    registries: Option<Arc<registry::Bank>>,
    hosted_globals: Option<Arc<hosted::GlobalDocumentIndex>>,
    hosted_history: Option<Arc<hosted::history::Graph>>,
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
    phase_map: Option<source::PhaseMap>,
    base_work: std::cell::Cell<u64>,
    base_work_limit: u64,
    encoding_url_work: std::cell::Cell<u64>,
    encoding_url_work_limit: u64,
    meta_work: std::cell::Cell<u64>,
    meta_work_limit: u64,
    view_transitions: DraftTransitions,
    signal_slots: Vec<Key>,
    notification_queued: bool,
    cancelled_signals: Vec<NodeId>,
    native_tasks: Vec<select::Pending>,
    cancelled_selects: Vec<(Option<u64>, NodeId)>,
    details_intentions: Vec<details::Pending<Key>>,
    cancelled_details: Vec<details::Pending<NodeId>>,
    cancelled_images: Option<select::ScratchVec<image::Cancelled>>,
    effects: Vec<select::EffectFrame>,
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
    existing: Vec<Option<DraftNode>>,
    generations: Vec<u64>,
    created: Vec<Option<DraftNode>>,
    // Caller creation tickets exclude private allocations such as imported
    // descendants. They must not be interpreted as allocation indices.
    tickets: Vec<Key>,
    nodes: usize,
    units: usize,
    document_environment: crate::DomDocumentEnvironment,
    input_processing: input_processing::InputProcessing,
    #[cfg(test)]
    insertion_trace: Option<tree::InsertionTrace>,
    #[cfg(test)]
    svg_processing_trace: Option<svg_script::ProcessingTrace>,
}
pub(crate) struct PreparedPublication {
    created_ranges: Vec<Option<crate::DomRangeId>>,
    state: Arc<State>,
    changed: bool,
    revision: DomRevision,
    entry: Option<Box<HistoryEntry>>,
    retirement_publication: dispatch::PublicationPlan,
    retired_authority: Option<Arc<crate::select::Authorization>>,
    owner: Arc<BatchIdentity>,
    created: Vec<Option<NodeId>>,
    documents: Vec<bool>,
    transition_ids: Vec<Option<DomViewTransitionId>>,
    record_ids: Vec<Option<DomViewTransitionRecordId>>,
    from: DomStamp,
    to: DomStamp,
    changes: Arc<[DomChange]>,
    _formation: Option<source::formation::Formation>,
}
struct Prepared {
    created_ranges: Vec<Option<crate::DomRangeId>>,
    retirements: Vec<dispatch::Retirement>,
    formation: Option<source::formation::Formation>,
    allocation_ids: Vec<Option<NodeId>>,
    state: State,
    owner: Arc<BatchIdentity>,
    created: Vec<Option<NodeId>>,
    documents: Vec<bool>,
    transition_ids: Vec<Option<DomViewTransitionId>>,
    record_ids: Vec<Option<DomViewTransitionRecordId>>,
    changes: Vec<DomChange>,
}

impl Draft {
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
        let mut existing = Vec::new();
        let mut generations = Vec::new();
        source::initial::reserve(&mut existing, dom.state.slots.len(), initial, cancelled)?;
        source::initial::reserve(&mut generations, dom.state.slots.len(), initial, cancelled)?;
        for slot in &dom.state.slots {
            source::initial::step(initial, cancelled)?;
            let node = if let Some(node) = &slot.node {
                let mut children = Vec::new();
                source::initial::reserve(&mut children, node.children.len(), initial, cancelled)?;
                for id in &node.children {
                    source::initial::step(initial, cancelled)?;
                    children.push(Key::Existing(id.slot));
                }
                Some(DraftNode {
                    active_range: node.active_range,
                    relevant_global: node.relevant_global,
                    document: Key::Existing(node.document.0.slot),
                    kind: node.kind.clone(),
                    base: node
                        .base
                        .as_ref()
                        .map(|b| b.map(|id| Key::Existing(id.slot))),
                    base_document: node
                        .base_document
                        .as_ref()
                        .map(|b| source::initial::base_document(b, initial, cancelled))
                        .transpose()?,
                    link: node
                        .link
                        .as_ref()
                        .map(|b| b.map(|id| Key::Existing(id.slot))),
                    image: node
                        .image
                        .as_ref()
                        .map(|b| b.map(|id| Key::Existing(id.slot))),
                    hyperlink: node
                        .hyperlink
                        .as_ref()
                        .map(|h| h.map(|id| Key::Existing(id.slot))),
                    encoding_url_document: node.encoding_url_document.clone(),
                    autofocus_document: node
                        .autofocus_document
                        .as_ref()
                        .map(|state| {
                            use crate::autofocus::DocumentAutofocusState as A;
                            Ok::<_, DomError>(match state {
                                A::Known {
                                    candidates,
                                    processed,
                                } => A::Known {
                                    candidates: source::initial::map_keys(
                                        candidates, initial, cancelled,
                                    )?,
                                    processed: *processed,
                                },
                                A::Unavailable => A::Unavailable,
                            })
                        })
                        .transpose()?,
                    interactive_document: node
                        .interactive_document
                        .as_ref()
                        .map(|state| {
                            use crate::dialog::DocumentInteractiveState as Interactive;
                            Ok::<_, DomError>(match state {
                                Interactive::Known {
                                    open_dialogs,
                                    top_layer,
                                    pending_top_layer_removals,
                                } => Interactive::Known {
                                    open_dialogs: source::initial::map_keys(
                                        open_dialogs,
                                        initial,
                                        cancelled,
                                    )?,
                                    top_layer: source::initial::map_keys(
                                        top_layer, initial, cancelled,
                                    )?,
                                    pending_top_layer_removals: source::initial::map_keys(
                                        pending_top_layer_removals,
                                        initial,
                                        cancelled,
                                    )?,
                                },
                                Interactive::Unavailable => Interactive::Unavailable,
                            })
                        })
                        .transpose()?,
                    style_parser_document: node
                        .style_parser_document
                        .map(|id| Key::Existing(id.slot)),
                    script: node
                        .script
                        .map(|value| value.map(|id| Key::Existing(id.slot))),
                    parent: node.parent.map(|id| Key::Existing(id.slot)),
                    shadow_root: node.shadow_root.map(|id| Key::Existing(id.slot)),
                    host: node.host.map(|id| Key::Existing(id.slot)),
                    template_contents: node.template_contents.map(|id| Key::Existing(id.slot)),
                    template_patch: node
                        .template_patch
                        .map(|p| p.map(|id| Key::Existing(id.slot))),
                    template_document: node
                        .template_document
                        .map(|value| value.map(|id| Key::Existing(id.slot))),
                    children,
                    manual_nodes: source::initial::map_keys(
                        &node.manual_nodes,
                        initial,
                        cancelled,
                    )?,
                    assigned_nodes: source::initial::map_keys(
                        &node.assigned_nodes,
                        initial,
                        cancelled,
                    )?,
                    manual_slot: node.manual_slot.map(|id| Key::Existing(id.slot)),
                    assigned_slot: node.assigned_slot.map(|id| Key::Existing(id.slot)),
                    control_owner: node.control_owner.map(|id| Key::Existing(id.slot)),
                    option_select: node.option_select.map(|id| Key::Existing(id.slot)),
                    form_past_names: node
                        .form_past_names
                        .as_ref()
                        .map(|map| source::initial::form_names(map, initial, cancelled))
                        .transpose()?,
                    reflected_targets: node
                        .reflected_targets
                        .map(|targets| targets.map(|id| Key::Existing(id.slot))),
                })
            } else {
                None
            };
            existing.push(node);
            generations.push(slot.generation);
        }
        let view_transitions =
            DraftTransitions::new_formed(&dom.state.view_transitions, initial, cancelled)?;
        let signal_slots = source::initial::map_keys(&dom.state.signal_slots, initial, cancelled)?;
        let native_tasks = select::draft_tasks_formed(&dom.state.native_tasks, initial, cancelled)?;
        let details_intentions =
            details::draft_formed(&dom.state.details_intentions, initial, cancelled)?;
        let mut ranges = Vec::new();
        source::initial::reserve(&mut ranges, dom.state.ranges.len(), initial, cancelled)?;
        for slot in &dom.state.ranges {
            source::initial::step(initial, cancelled)?;
            ranges.push(crate::range::Slot {
                generation: slot.generation,
                retired: slot.retired,
                record: slot
                    .record
                    .map(|record| record.map(|node| Key::Existing(node.slot))),
            });
        }
        Ok(Self {
            ranges,
            range_tickets: Vec::new(),
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
            hosted_owner: None,
            basis: DomStamp {
                owner: dom.owner,
                revision: dom.revision,
            },
            source_limits: initial.map(source::initial::Initial::limits),
            source_work: Arc::new(AtomicU64::new(
                initial.map_or(0, source::initial::Initial::work),
            )),
            source_ancestors: initial.and_then(|initial| initial.ancestors().cloned()),
            source_formation_bytes: Arc::new(AtomicUsize::new(0)),
            source_temporary_units: Arc::new(AtomicUsize::new(0)),
            source_other_bytes: 0,
            source_other_units: 0,
            source_invocation: 0,
            source_headroom: None,
            method: None,
            dispatch_retirements: Vec::new(),
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
            cancelled_signals: Vec::new(),
            native_tasks,
            cancelled_selects: Vec::new(),
            details_intentions,
            cancelled_details: Vec::new(),
            cancelled_images: None,
            effects: Vec::new(),
            hosted_pending: None,
            effects_running: false,
            effect_floor: 0,
            effect_bytes: 0,
            effect_scoped: false,
            select_work: std::cell::Cell::new(0),
            select_work_limit: dom.limits.select_work,
            select_scratch_limit: dom.limits.select_scratch,
            select_temporary_bytes: Arc::new(AtomicUsize::new(0)),
            select_semantic_work: std::cell::Cell::new(0),
            select_semantic_bytes: std::cell::Cell::new(0),
            existing,
            generations,
            created: Vec::new(),
            tickets: Vec::new(),
            nodes: dom.state.nodes,
            units: dom.state.units,
        })
    }
    fn node(&self, key: Key) -> Result<&DraftNode, DomError> {
        match key {
            Key::Existing(i) => self.existing.get(i),
            Key::Created(i) => self.created.get(i),
        }
        .and_then(Option::as_ref)
        .ok_or(DomError::StaleNode)
    }
    fn node_mut(&mut self, key: Key) -> &mut DraftNode {
        match key {
            Key::Existing(i) => &mut self.existing[i],
            Key::Created(i) => &mut self.created[i],
        }
        .as_mut()
        .expect("validated staged node")
    }
    fn resolve(&self, operand: NodeOperand) -> Result<Key, DomError> {
        let key = match operand {
            NodeOperand::Existing(id) => {
                if id.owner != self.store {
                    return Err(DomError::ForeignOwner);
                }
                if let Some(map) = &self.phase_map
                    && let Some(index) = map.ids.iter().position(|candidate| *candidate == Some(id))
                {
                    let key = Key::Created(index);
                    self.node(key)?;
                    return Ok(key);
                }
                if self.generations.get(id.slot) != Some(&id.generation) {
                    return Err(DomError::StaleNode);
                }
                Key::Existing(id.slot)
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
            self.node_mut(parent)
                .children
                .retain(|child| *child != node);
            self.node_mut(node).parent = None;
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
        let nodes = self
            .nodes
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(nodes, limits.nodes, DomResource::ManagedNodes)?;
        let units = self
            .units
            .checked_add(kind_units(&kind)?)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        match self.source_reserve_amount::<Option<DraftNode>>(
            self.created.len(),
            self.created.capacity(),
            1,
            cancel,
        )? {
            Some(amount) => self
                .created
                .try_reserve_exact(amount)
                .map_err(|_| DomError::AllocationFailed)?,
            None => reserve(&mut self.created, 1)?,
        }
        self.source_admit(0, cancel)?;
        let key = Key::Created(self.created.len());
        let reflected_targets = match &kind {
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
            if matches!(&kind, DomNodeKind::Element(element) if element.form_state().is_some()) {
                budget.retain_bytes(size_of::<Vec<crate::form::FormPastName<Key>>>(), cancel)?;
            }
            if matches!(&kind, DomNodeKind::Element(element) if element.is_img()) {
                budget.retain_bytes(size_of::<Vec<u16>>(), cancel)?;
            }
        }
        let form_past_names =
            matches!(&kind, DomNodeKind::Element(element) if element.form_state().is_some())
                .then(crate::form::FormPastNames::fresh);
        let interactive_document = match &kind {
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
        let autofocus_document = match &kind {
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
        let template_document = matches!(kind, DomNodeKind::Document(_))
            .then_some(crate::template::TemplateDocument::Ordinary { inert: None });
        let script = matches!(&kind, DomNodeKind::Element(element) if element.is_html_script())
            .then(crate::script::ScriptState::fresh);
        let base = matches!(&kind,DomNodeKind::Element(e) if e.is_base())
            .then(crate::base::BaseState::fresh);
        let link = matches!(&kind,DomNodeKind::Element(e) if e.is_link())
            .then(crate::base::LinkState::fresh);
        let image = matches!(&kind,DomNodeKind::Element(e) if e.is_img())
            .then(|| crate::image::ImageState::fresh(key));
        let hyperlink = matches!(&kind,DomNodeKind::Element(e) if e.is_hyperlink())
            .then(crate::hyperlink::HyperlinkState::fresh);
        let encoding_url_document = matches!(&kind, DomNodeKind::Document(_))
            .then(crate::hyperlink::EncodingUrlDocument::fresh);
        let base_document =
            matches!(&kind, DomNodeKind::Document(_)).then(crate::base::BaseDocument::fresh);
        if let Some(map) = &self.phase_map {
            let amount = self
                .source_reserve_amount::<Option<NodeId>>(
                    map.ids.len(),
                    map.ids.capacity(),
                    1,
                    cancel,
                )?
                .expect("source map has limits");
            self.phase_map
                .as_mut()
                .expect("source map")
                .ids
                .try_reserve_exact(amount)
                .map_err(|_| DomError::AllocationFailed)?;
            self.source_admit(0, cancel)?;
            let map = self.phase_map.as_ref().expect("source map");
            let amount = self
                .source_reserve_amount::<bool>(
                    map.private.len(),
                    map.private.capacity(),
                    1,
                    cancel,
                )?
                .expect("source map has limits");
            self.phase_map
                .as_mut()
                .expect("source map")
                .private
                .try_reserve_exact(amount)
                .map_err(|_| DomError::AllocationFailed)?;
            self.source_admit(0, cancel)?;
            let map = self.phase_map.as_mut().expect("source map");
            map.ids.push(None);
            map.private.push(false);
        }
        let relevant_global = match &kind {
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
        self.created.push(Some(DraftNode {
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
            children: Vec::new(),
            shadow_root: None,
            host: None,
            template_contents: None,
            template_patch: None,
            template_document,
            script,
            style_parser_document: None,
            manual_nodes: Vec::new(),
            assigned_nodes: Vec::new(),
            manual_slot: None,
            assigned_slot: None,
            reflected_targets,
            control_owner: None,
            option_select: None,
            form_past_names,
        }));
        // Both related payloads now belong to the inserted node. No fallible
        // operation intervenes between that ownership transfer and release.
        drop(related_formation);
        self.nodes = nodes;
        self.units = units;
        Ok(key)
    }
    fn register_ticket(&mut self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        match self.source_reserve_amount::<Key>(
            self.tickets.len(),
            self.tickets.capacity(),
            1,
            cancel,
        )? {
            Some(amount) => self
                .tickets
                .try_reserve_exact(amount)
                .map_err(|_| DomError::AllocationFailed)?,
            None => reserve(&mut self.tickets, 1)?,
        }
        self.source_admit(0, cancel)?;
        self.tickets.push(key);
        Ok(())
    }
    fn apply(
        &mut self,
        command: &Command,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        match command {
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
                                Ok::<_, DomError>(crate::node_values::source_observation::bind(
                                    self.source_storage_bytes(cancelled)?
                                        .checked_add(self.source_other_bytes)
                                        .ok_or(DomError::RepresentationExhausted)?,
                                    self.source_formation_bytes.clone(),
                                    limits.retained_bytes,
                                    self.source_storage_units(cancelled)?
                                        .checked_add(self.source_other_units)
                                        .ok_or(DomError::RepresentationExhausted)?,
                                    self.source_temporary_units.clone(),
                                    limits.retained_units,
                                ))
                            })
                            .transpose()?;
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
                let key = self.allocate(kind, document, limits, cancelled)?;
                // Direct infallible ownership transfer: this payload is now in
                // the Draft's node graph, before any subsequent fallible hook.
                drop(metadata_formation);
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
                let DomNodeKind::Element(element) = &mut self.node_mut(node).kind else {
                    return Err(DomError::WrongNodeKind);
                };
                element.set_marquee_running(*running)?;
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
                self.select_reserve(&mut pending, self.nodes, cancelled)?;
                self.select_reserve(&mut closure, self.nodes, cancelled)?;
                pending.push(node);
                while !pending.is_empty() {
                    self.source_step(cancelled)?;
                    let key = pending.pop().expect("admitted destruction node");
                    let current = self.node(key)?;
                    for child in current.children.iter().rev() {
                        self.source_step(cancelled)?;
                        pending.push(*child);
                    }
                    pending.extend(current.shadow_root);
                    pending.extend(self.owned_template_contents(key, cancelled)?);
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
                let previous_units = kind_units(&self.node(node)?.kind)?;
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
                for unit in previous.code_units()[..offset]
                    .iter()
                    .chain(data.code_units())
                    .chain(&previous.code_units()[offset + removed..])
                {
                    self.source_step(cancelled)?;
                    value.push(*unit);
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
                let target = self.node_mut(node);
                target.kind = kind;
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
        self.range_publication_ready(cancelled)?;
        admit(
            self.autofocus_count(cancelled)?,
            dom.limits.autofocus_candidates,
            DomResource::AutofocusCandidates,
        )?;
        let formation = source::formation::Formation::new(self, cancelled)?;
        let budget = formation.as_ref();
        let source::PublicationMap {
            mut slots,
            occupants,
            ids: allocated_ids,
        } = self.publication_map(dom, budget, cancelled)?;
        let id = |key| match key {
            Key::Existing(slot) => NodeId {
                owner: dom.owner,
                slot,
                generation: self.generations[slot],
            },
            Key::Created(index) => {
                allocated_ids[index].expect("surviving public creation reference")
            }
        };
        // Resolve requested private adapter identities only while the canonical
        // final allocation map exists. All fallible output precedes publication.
        let mut mapped = Vec::new();
        if budget.is_some() {
            source::formation::reserve(&mut mapped, requested.len(), budget, cancelled)?;
        } else {
            // The ordinary immutable adapter admits this exact output bank;
            // preserve its established allocation policy outside composition.
            mapped
                .try_reserve_exact(requested.len())
                .map_err(|_| DomError::AllocationFailed)?;
        }
        for key in requested {
            check_cancel(cancelled)?;
            self.node(key)?;
            mapped.push(id(key));
        }
        let occupant_bytes = occupants.capacity() * size_of::<Option<Key>>();
        for (slot, key) in occupants.into_iter().enumerate() {
            check_cancel(cancelled)?;
            let Some(key) = key else {
                continue;
            };
            let source = self.node(key)?;
            if source
                .base
                .as_ref()
                .and_then(|b| b.frozen.as_ref())
                .is_some_and(|f| f.response == crate::base::Response::Pending)
            {
                return Err(DomError::BaseFacts(crate::DomBaseFactsError::Missing(
                    surgeist_html_semantics::HtmlBaseMissing::ResponseDomain,
                )));
            }
            // Clone each backing occurrence only after its full candidate
            // lifetime has been admitted, while occupants and Draft still live.
            if let Some(budget) = budget {
                let payload =
                    source::storage::candidate_node(source, || self.source_step(cancelled))?;
                budget.retain_payload(payload, cancelled)?;
            }
            let mut children = Vec::new();
            source::formation::reserve(&mut children, source.children.len(), budget, cancelled)?;
            children.extend(source.children.iter().copied().map(id));
            slots[slot].node = Some(Node {
                active_range: source.active_range,
                relevant_global: source.relevant_global,
                document: DocumentId(id(source.document)),
                kind: source.kind.clone(),
                script: source.script.map(|value| value.map(id)),
                style_parser_document: source.style_parser_document.map(id),
                parent: source.parent.map(id),
                shadow_root: source.shadow_root.map(id),
                host: source.host.map(id),
                template_contents: source.template_contents.map(id),
                template_patch: source.template_patch.map(|p| p.map(id)),
                template_document: source.template_document.map(|value| value.map(id)),
                children,
                manual_nodes: map_ids(&source.manual_nodes, &id, budget, cancelled)?,
                assigned_nodes: map_ids(&source.assigned_nodes, &id, budget, cancelled)?,
                manual_slot: source.manual_slot.map(id),
                assigned_slot: source.assigned_slot.map(id),
                reflected_targets: source.reflected_targets.map(|targets| targets.map(id)),
                control_owner: source.control_owner.map(id),
                option_select: source.option_select.map(id),
                base: source.base.as_ref().map(|b| b.map(id)),
                base_document: source
                    .base_document
                    .as_ref()
                    .map(|b| {
                        Ok::<_, DomError>(crate::base::BaseDocument {
                            facts: b.facts.clone(),
                            first: b.first.map(id),
                            render_blocking: map_ids(&b.render_blocking, &id, budget, cancelled)?,
                        })
                    })
                    .transpose()?,
                link: source.link.as_ref().map(|b| b.map(id)),
                image: source.image.as_ref().map(|b| b.map(id)),
                hyperlink: source.hyperlink.as_ref().map(|h| h.map(id)),
                encoding_url_document: source.encoding_url_document.clone(),
                autofocus_document: source
                    .autofocus_document
                    .as_ref()
                    .map(|state| {
                        use crate::autofocus::DocumentAutofocusState as A;
                        Ok::<_, DomError>(match state {
                            A::Known {
                                candidates,
                                processed,
                            } => A::Known {
                                candidates: map_ids(candidates, &id, budget, cancelled)?,
                                processed: *processed,
                            },
                            A::Unavailable => A::Unavailable,
                        })
                    })
                    .transpose()?,
                interactive_document: source
                    .interactive_document
                    .as_ref()
                    .map(|state| {
                        use crate::dialog::DocumentInteractiveState as Interactive;
                        Ok::<_, DomError>(match state {
                            Interactive::Known {
                                open_dialogs,
                                top_layer,
                                pending_top_layer_removals,
                            } => Interactive::Known {
                                open_dialogs: map_ids(open_dialogs, &id, budget, cancelled)?,
                                top_layer: map_ids(top_layer, &id, budget, cancelled)?,
                                pending_top_layer_removals: map_ids(
                                    pending_top_layer_removals,
                                    &id,
                                    budget,
                                    cancelled,
                                )?,
                            },
                            Interactive::Unavailable => Interactive::Unavailable,
                        })
                    })
                    .transpose()?,
                form_past_names: source
                    .form_past_names
                    .as_ref()
                    .map(|map| form::map_names_formed(map, id, budget, cancelled))
                    .transpose()?,
            });
            #[cfg(test)]
            if let Some(budget) = budget {
                budget.observe_candidate(
                    &slots,
                    occupant_bytes,
                    allocated_ids.capacity(),
                    mapped.capacity(),
                );
            }
        }
        if let Some(budget) = budget {
            budget.release_bytes(occupant_bytes);
        }
        let select::FinishedTasks {
            tasks: native_tasks,
            next: next_task,
            cancellations,
        } = self.finish_select_tasks(dom, &id, budget, cancelled)?;
        let details::Finished {
            intentions: details_intentions,
            cancellations: details_cancellations,
            next: next_details_intention,
        } = self.finish_details(dom, &id, budget, cancelled)?;
        let finished = self.view_transitions.finish(dom.owner, budget, cancelled)?;
        let mut ranges = Vec::new();
        source::formation::reserve(&mut ranges, self.ranges.len(), budget, cancelled)?;
        for (index, slot) in self.ranges.iter().enumerate() {
            self.source_step(cancelled)?;
            // An unpublished creation is not a canonical generation. Its
            // private serial still distinguishes tickets during this batch,
            // but an absent final record restores the published vacant basis.
            let (generation, retired) = if slot.record.is_none() {
                dom.state.ranges.get(index).map_or((0, false), |old| {
                    (
                        old.generation,
                        old.retired || (old.record.is_some() && old.generation == u64::MAX),
                    )
                })
            } else {
                (slot.generation, slot.retired)
            };
            ranges.push(crate::range::Slot {
                generation,
                retired,
                record: slot.record.map(|record| record.map(id)),
            });
        }
        while ranges.len() > dom.state.ranges.len()
            && ranges.last().is_some_and(|slot| slot.record.is_none())
        {
            self.source_step(cancelled)?;
            ranges.pop();
        }
        let mut created_ranges = Vec::new();
        source::formation::reserve(
            &mut created_ranges,
            self.range_tickets.len(),
            budget,
            cancelled,
        )?;
        for range in &self.range_tickets {
            self.source_step(cancelled)?;
            created_ranges.push(self.range_record(*range).ok().map(|_| *range));
        }
        let state = State {
            ranges,
            incomplete_source: None,
            incomplete_hosted: dom.state.incomplete_hosted,
            slots,
            registries: self.registries.clone(),
            hosted_globals: self.hosted_globals.clone(),
            hosted_history: self.hosted_history.clone(),
            view_transitions: finished.value,
            signal_slots: self.map_public_ids(&self.signal_slots, &id, budget, cancelled)?,
            notification_queued: self.notification_queued,
            native_tasks,
            next_task,
            details_intentions,
            next_details_intention,
            nodes: self.public_nodes(),
            units: self.public_units(cancelled)?,
        };
        let mut changes = difference(dom.owner, &dom.state, &state, budget, cancelled)?;
        if let Some(cancelled_images) = &self.cancelled_images {
            for item in cancelled_images.iter() {
                check_cancel(cancelled)?;
                source::formation::reserve(&mut changes, 1, budget, cancelled)?;
                changes.push(DomChange::ImageUpdateCancelled {
                    update: item.update,
                    flags: item.flags,
                    reason: crate::DomImageUpdateCancellationReason::TargetDestroyed,
                });
            }
        }

        let cancellation_bytes = cancellations.capacity() * size_of::<(u64, NodeId)>();
        for (operation, select) in cancellations {
            source::formation::reserve(&mut changes, 1, budget, cancelled)?;
            changes.push(DomChange::SelectTaskCancelled {
                operation: crate::DomNativeTaskId {
                    owner: dom.owner,
                    serial: operation,
                },
                select,
                reason: crate::DomSelectTaskCancellationReason::TargetDestroyed,
            });
        }
        if let Some(budget) = budget {
            budget.release_bytes(cancellation_bytes);
        }
        let details_bytes =
            details_cancellations.capacity() * size_of::<crate::DomDetailsToggleIntention>();
        for item in details_cancellations {
            source::formation::reserve(&mut changes, 1, budget, cancelled)?;
            changes.push(DomChange::DetailsToggleIntentionCancelled {
                operation: item.operation,
                target: item.target,
                old: item.old,
                new: item.new,
                reason: crate::DomDetailsToggleCancellationReason::TargetDestroyed,
            });
        }
        if let Some(budget) = budget {
            budget.release_bytes(details_bytes);
        }
        for slot in &self.cancelled_signals {
            source::formation::reserve(&mut changes, 1, budget, cancelled)?;
            changes.push(DomChange::SlotSignalCancelled {
                slot: *slot,
                reason: crate::DomSlotSignalCancellationReason::Destroyed,
            });
        }
        let mut created = Vec::new();
        let mut documents = Vec::new();
        source::formation::reserve(&mut created, self.tickets.len(), budget, cancelled)?;
        source::formation::reserve(&mut documents, self.tickets.len(), budget, cancelled)?;
        for ticket in &self.tickets {
            let Key::Created(index) = *ticket else {
                unreachable!("creation ticket allocation")
            };
            created.push(allocated_ids[index]);
            documents.push(
                self.created[index]
                    .as_ref()
                    .is_some_and(|node| matches!(node.kind, DomNodeKind::Document(_))),
            );
        }
        let mut retirements = Vec::new();
        source::formation::reserve(
            &mut retirements,
            self.dispatch_retirements.len(),
            budget,
            cancelled,
        )?;
        for &(target, document) in &self.dispatch_retirements {
            self.source_step(cancelled)?;
            if !dom
                .dispatch_custody
                .tracks(target, || self.source_step(cancelled))?
            {
                continue;
            }
            let document = DocumentId(id(document));
            retirements.push(dispatch::Retirement { target, document });
        }
        Ok((
            Prepared {
                created_ranges,
                retirements,
                formation,
                allocation_ids: allocated_ids,
                state,
                owner: self.owner.clone(),
                created,
                documents,
                transition_ids: finished.transition_ids,
                record_ids: finished.record_ids,
                changes,
            },
            mapped,
        ))
    }
}
fn map_ids(
    keys: &[Key],
    id: &impl Fn(Key) -> NodeId,
    budget: Option<&source::formation::Formation>,
    cancelled: &AtomicBool,
) -> Result<Vec<NodeId>, DomError> {
    let mut result = Vec::new();
    source::formation::reserve(&mut result, keys.len(), budget, cancelled)?;
    for key in keys {
        check_cancel(cancelled)?;
        result.push(id(*key));
    }
    Ok(result)
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
fn difference(
    owner: DomOwnerId,
    old: &State,
    new: &State,
    budget: Option<&source::formation::Formation>,
    cancelled: &AtomicBool,
) -> Result<Vec<DomChange>, DomError> {
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
        state.slots.iter().any(|slot| {
            slot.node.as_ref().is_some_and(|node| {
                node.template_contents.is_some()
                    || node.template_document.is_some_and(|state| {
                        matches!(state, crate::template::TemplateDocument::InertTemplate)
                            || state.inert().is_some()
                    })
            })
        })
    };
    let cache_changed = old.slots.iter().zip(&new.slots).any(|(a, b)| {
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
        state.slots.iter().any(|slot| {
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
            state.slots.iter().any(|s| {
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
    for transition in &new.view_transitions.transitions {
        if old
            .view_transitions
            .transitions
            .iter()
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
