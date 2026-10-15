use std::{
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::batch::{BatchIdentity, Command, Creation};
use crate::document::{DomDocumentMetadata, NATIVE_DOCUMENT_STRING_UNITS};
use crate::{
    CreatedNode, DocumentId, DocumentString, DomBatch, DomDocumentType, DomElement, DomNodeKind,
    DomOwnerId, DomProcessingInstruction, DomRevision, DomSnapshot, DomStamp, NodeId, NodeOperand,
};

mod attributes;
mod children;
mod control;
mod document;
mod form;
mod import;
mod input;
mod input_processing;
mod installation;
mod navigable;
mod output;
mod reflected;
mod script;
mod select;
mod shadow;
mod slot;
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
    ManagedNodes,
    SlotReferences,
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
    InputProcessingScratch,
    SelectScratch,
    SelectTasks,
}

/// Typed rejection; no rejected batch publishes any state or identity.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DomError {
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
    NativeInitializationRequired(surgeist_html::HtmlNativeInitializationRequirement),
    NonceInapplicable,
    MarqueeInapplicable,
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
    SelectInapplicable,
    OptionInapplicable,
    SelectProtocol,
    SelectValueQueriesUnconfigured,
    SelectCapture(surgeist_html_semantics::HtmlCaptureError),
    SelectSemantic(surgeist_html_semantics::HtmlSemanticFailure),
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
    nodes: usize,
    units: usize,
    commands: usize,
    slot_references: usize,
    slot_assignment_inputs: usize,
    history_entries: usize,
    history_bytes: usize,
    transitions: usize,
    records: usize,
    pseudos: usize,
    input_scratch: usize,
    input_work: u64,
    select_tasks: usize,
    select_scratch: usize,
    select_work: u64,
    select_value_queries: Option<(
        surgeist_html_semantics::HtmlCaptureLimits,
        surgeist_html_semantics::HtmlSemanticLimits,
    )>,
}
impl DomLimits {
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
            nodes,
            units,
            commands,
            slot_references,
            slot_assignment_inputs,
            history_entries,
            history_bytes,
            transitions: 0,
            records: 0,
            pseudos: 0,
            input_scratch: 0,
            input_work: 0,
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
    /// Template contents ownership or a Document's appropriate-owner cache changed.
    TemplateRelations,
    SlotManual(NodeId),
    SlotAssignedNodes(NodeId),
    ManualAssignment(NodeId),
    StoredAssignedSlot(NodeId),
    /// Conservatively recompute current slot/flattened/flat projections.
    SlotQueries,
    IntrinsicTasks,
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
    owner: Arc<BatchIdentity>,
    created: Vec<Option<NodeId>>,
    documents: Vec<bool>,
    transition_ids: Vec<Option<DomViewTransitionId>>,
    record_ids: Vec<Option<DomViewTransitionRecordId>>,
    changes: DomChanges,
}
impl DomCommit {
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Node {
    pub(crate) document: DocumentId,
    pub(crate) kind: DomNodeKind,
    pub(crate) parent: Option<NodeId>,
    pub(crate) children: Vec<NodeId>,
    pub(crate) shadow_root: Option<NodeId>,
    pub(crate) host: Option<NodeId>,
    pub(crate) template_contents: Option<NodeId>,
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
}
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Slot {
    pub(crate) generation: u64,
    retired: bool,
    pub(crate) node: Option<Node>,
}
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct State {
    pub(crate) slots: Vec<Slot>,
    pub(crate) view_transitions: ViewTransitions,
    pub(crate) signal_slots: Vec<NodeId>,
    pub(crate) notification_queued: bool,
    native_tasks: Vec<select::Task<NodeId>>,
    next_task: u64,
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
pub struct Dom {
    owner: DomOwnerId,
    revision: DomRevision,
    limits: DomLimits,
    state: Arc<State>,
    history: Option<Box<HistoryEntry>>,
    history_entries: usize,
    history_bytes: usize,
    native_environment: crate::DomNativeDocumentEnvironment,
    active_task: std::sync::Weak<crate::select::Authorization>,
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
            owner: DomOwnerId::allocate()?,
            revision: DomRevision(0),
            limits,
            state: Arc::new(State {
                slots: Vec::new(),
                view_transitions: ViewTransitions::new(),
                signal_slots: Vec::new(),
                notification_queued: false,
                native_tasks: Vec::new(),
                next_task: 0,
                nodes: 0,
                units: 0,
            }),
            active_task: std::sync::Weak::new(),
            history: None,
            history_entries: 0,
            history_bytes: 0,
            native_environment: crate::DomNativeDocumentEnvironment::new(input),
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
            owner: self.owner,
            revision: self.revision,
            state: self.state.clone(),
        }
    }
    /// Live bounds cannot fall below admitted state. Smaller history bounds
    /// discard coverage explicitly, without changing semantic revision.
    pub fn set_limits(&mut self, limits: DomLimits) -> Result<(), DomError> {
        admit(
            self.state.slot_reference_count(),
            limits.slot_references,
            DomResource::SlotReferences,
        )?;
        admit(self.state.nodes, limits.nodes, DomResource::ManagedNodes)?;
        admit(self.state.units, limits.units, DomResource::StringCodeUnits)?;
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
    fn prepare(&self, batch: DomBatch, cancelled: &AtomicBool) -> Result<Prepared, DomError> {
        check_cancel(cancelled)?;
        admit(
            batch.commands.len(),
            self.limits.commands,
            DomResource::Commands,
        )?;
        let mut incoming = 0usize;
        for command in &batch.commands {
            check_cancel(cancelled)?;
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
        let mut draft = Draft::new(self, batch.owner, cancelled)?;
        for command in batch.commands {
            check_cancel(cancelled)?;
            draft.apply(command, self.limits, cancelled)?;
        }
        check_cancel(cancelled)?;
        draft.finish(self, cancelled)
    }
    fn publish(
        &mut self,
        prepared: Prepared,
        cancelled: &AtomicBool,
    ) -> Result<DomCommit, DomError> {
        check_cancel(cancelled)?;
        let Prepared {
            state,
            owner,
            created,
            documents,
            transition_ids,
            record_ids,
            changes,
        } = prepared;
        let changed = state != *self.state;
        let revision = if changed {
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
        // Last cancellation observation is the publication boundary. No user
        // callbacks or external effects occur while prepared state is private.
        check_cancel(cancelled)?;
        if changed {
            self.state = state;
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
        if let Some(auth) = retired_authority {
            auth.destroy_if_pending();
            self.active_task = std::sync::Weak::new();
        }
        Ok(DomCommit {
            owner,
            created,
            documents,
            transition_ids,
            record_ids,
            changes: DomChanges { from, to, changes },
        })
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
fn copy<T: Clone>(values: &[T]) -> Result<Vec<T>, DomError> {
    let mut result = Vec::new();
    reserve(&mut result, values.len())?;
    result.extend_from_slice(values);
    Ok(result)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Key {
    Existing(usize),
    Created(usize),
}
struct DraftNode {
    document: Key,
    kind: DomNodeKind,
    parent: Option<Key>,
    children: Vec<Key>,
    shadow_root: Option<Key>,
    host: Option<Key>,
    template_contents: Option<Key>,
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
}
struct Draft {
    owner: Arc<BatchIdentity>,
    store: DomOwnerId,
    basis: DomStamp,
    view_transitions: DraftTransitions,
    signal_slots: Vec<Key>,
    notification_queued: bool,
    cancelled_signals: Vec<NodeId>,
    native_tasks: Vec<select::Pending>,
    cancelled_selects: Vec<(Option<u64>, NodeId)>,
    effects: Vec<select::EffectFrame>,
    effects_running: bool,
    effect_bytes: usize,
    effect_scoped: bool,
    select_work: std::cell::Cell<u64>,
    select_work_limit: u64,
    select_scratch_limit: usize,
    select_temporary_bytes: std::rc::Rc<std::cell::Cell<usize>>,
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
    native_environment: crate::DomNativeDocumentEnvironment,
    input_processing: input_processing::InputProcessing,
    #[cfg(test)]
    svg_processing_trace: Option<svg_script::ProcessingTrace>,
}
struct Prepared {
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
        self.initialize_template_contents(key, limits, cancel)?;
        if matches!(&self.node(key)?.kind,DomNodeKind::Element(element) if element.input_state().is_some())
        {
            self.initialize_input(key, limits, cancel)?;
        }
        Ok(())
    }
    fn new(dom: &Dom, owner: Arc<BatchIdentity>, cancelled: &AtomicBool) -> Result<Self, DomError> {
        let mut existing = Vec::new();
        let mut generations = Vec::new();
        reserve(&mut existing, dom.state.slots.len())?;
        reserve(&mut generations, dom.state.slots.len())?;
        for slot in &dom.state.slots {
            check_cancel(cancelled)?;
            let node = if let Some(node) = &slot.node {
                let mut children = Vec::new();
                reserve(&mut children, node.children.len())?;
                children.extend(node.children.iter().map(|id| Key::Existing(id.slot)));
                Some(DraftNode {
                    document: Key::Existing(node.document.0.slot),
                    kind: node.kind.clone(),
                    script: node
                        .script
                        .map(|value| value.map(|id| Key::Existing(id.slot))),
                    parent: node.parent.map(|id| Key::Existing(id.slot)),
                    shadow_root: node.shadow_root.map(|id| Key::Existing(id.slot)),
                    host: node.host.map(|id| Key::Existing(id.slot)),
                    template_contents: node.template_contents.map(|id| Key::Existing(id.slot)),
                    template_document: node
                        .template_document
                        .map(|value| value.map(|id| Key::Existing(id.slot))),
                    children,
                    manual_nodes: map_keys(&node.manual_nodes, cancelled)?,
                    assigned_nodes: map_keys(&node.assigned_nodes, cancelled)?,
                    manual_slot: node.manual_slot.map(|id| Key::Existing(id.slot)),
                    assigned_slot: node.assigned_slot.map(|id| Key::Existing(id.slot)),
                    control_owner: node.control_owner.map(|id| Key::Existing(id.slot)),
                    option_select: node.option_select.map(|id| Key::Existing(id.slot)),
                    form_past_names: node
                        .form_past_names
                        .as_ref()
                        .map(|map| form::map_names(map, |id| Key::Existing(id.slot), cancelled))
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
        Ok(Self {
            owner,
            #[cfg(test)]
            svg_processing_trace: None,
            native_environment: dom.native_environment,
            input_processing: input_processing::InputProcessing::new(
                dom.limits.input_scratch,
                dom.limits.input_work,
            ),
            store: dom.owner,
            basis: DomStamp {
                owner: dom.owner,
                revision: dom.revision,
            },
            view_transitions: DraftTransitions::new(&dom.state.view_transitions)?,
            signal_slots: map_keys(&dom.state.signal_slots, cancelled)?,
            notification_queued: dom.state.notification_queued,
            cancelled_signals: Vec::new(),
            native_tasks: select::draft_tasks(&dom.state.native_tasks, cancelled)?,
            cancelled_selects: Vec::new(),
            effects: Vec::new(),
            effects_running: false,
            effect_bytes: 0,
            effect_scoped: false,
            select_work: std::cell::Cell::new(0),
            select_work_limit: dom.limits.select_work,
            select_scratch_limit: dom.limits.select_scratch,
            select_temporary_bytes: std::rc::Rc::new(std::cell::Cell::new(0)),
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
            self.node_mut(parent)
                .children
                .retain(|child| *child != node);
            self.node_mut(node).parent = None;
            self.slot_removal(node, parent, limits, cancelled)?;
            // HTML removing steps observe the already detached subtree. Its
            // ordinary and shadow descendants retain their old node documents.
            self.native_removing_steps(node, limits, cancelled)?;
            self.children_changed(parent, limits, cancelled)?;
        }
        Ok(())
    }
    fn allocate(
        &mut self,
        kind: DomNodeKind,
        document: Key,
        limits: DomLimits,
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
        reserve(&mut self.created, 1)?;
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
        let form_past_names =
            matches!(&kind, DomNodeKind::Element(element) if element.form_state().is_some())
                .then(crate::form::FormPastNames::fresh);
        let template_document = matches!(kind, DomNodeKind::Document(_))
            .then_some(crate::template::TemplateDocument::Ordinary { inert: None });
        let script = matches!(&kind, DomNodeKind::Element(element) if element.is_html_script())
            .then(crate::script::ScriptState::fresh);
        self.created.push(Some(DraftNode {
            document,
            kind,
            parent: None,
            children: Vec::new(),
            shadow_root: None,
            host: None,
            template_contents: None,
            template_document,
            script,
            manual_nodes: Vec::new(),
            assigned_nodes: Vec::new(),
            manual_slot: None,
            assigned_slot: None,
            reflected_targets,
            control_owner: None,
            option_select: None,
            form_past_names,
        }));
        self.nodes = nodes;
        self.units = units;
        Ok(key)
    }
    fn register_ticket(&mut self, key: Key) -> Result<(), DomError> {
        reserve(&mut self.tickets, 1)?;
        self.tickets.push(key);
        Ok(())
    }
    fn apply(
        &mut self,
        command: Command,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        match command {
            Command::Select { node, edit } => {
                let key = self.resolve(node)?;
                self.select_edit(key, edit, limits, cancelled)?;
            }
            Command::Output { node, edit } => {
                let node = self.resolve(node)?;
                self.output_edit(node, edit, limits, cancelled)?;
            }
            Command::SetTextareaValue { node, value } => {
                let node = self.resolve(node)?;
                self.set_textarea_value(node, value, limits, cancelled)?;
            }
            Command::SetTextareaDefaultValue { node, value } => {
                let node = self.resolve(node)?;
                self.set_textarea_default_value(node, value, limits, cancelled)?;
            }
            Command::ResetTextarea { node } => {
                let node = self.resolve(node)?;
                self.reset_textarea(node, limits, cancelled)?;
            }
            Command::SetInputValue { node, value } => {
                let node = self.resolve(node)?;
                self.set_input_value(node, value, limits, cancelled)?;
            }
            Command::SetInputChecked { node, value } => {
                let node = self.resolve(node)?;
                self.set_input_checked(node, value, cancelled)?;
            }
            Command::SetInputIndeterminate { node, value } => {
                let node = self.resolve(node)?;
                self.set_input_indeterminate(node, value, cancelled)?;
            }
            Command::ResetInput { node } => {
                let node = self.resolve(node)?;
                self.reset_input(node, limits, cancelled)?;
            }
            Command::SetReflectedTarget { node, key, target } => {
                let node = self.resolve(node)?;
                let target = target.map(|target| self.resolve(target)).transpose()?;
                self.set_reflected_target(node, key, target, limits, cancelled)?;
            }
            Command::AssignSlot { slot, incoming } => {
                let slot = self.resolve(slot)?;
                self.assign_manual(slot, incoming, limits, cancelled)?;
            }
            Command::CreateHtmlDocument {
                associated_document,
                title,
            } => {
                let associated_document = self.resolve(associated_document)?;
                let key =
                    self.create_html_document(associated_document, title, limits, cancelled)?;
                self.register_ticket(key)?;
            }
            Command::AttachShadow { host, init } => {
                let host = self.resolve(host)?;
                let root = self.attach_shadow(host, init, limits, cancelled)?;
                self.register_ticket(root)?;
            }
            Command::ViewTransition { basis, edit } => {
                self.apply_view_transition(basis, edit, limits, cancelled)?
            }
            Command::Create { kind, document } => {
                let key = Key::Created(self.created.len());
                let document = if let Some(document) = document {
                    let document = self.resolve(document)?;
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
                let kind = match kind {
                    Creation::Document => {
                        DomNodeKind::Document(DomDocumentMetadata::native(self.native_environment))
                    }
                    Creation::Fragment => DomNodeKind::DocumentFragment,
                    Creation::Element(name) => {
                        let DomNodeKind::Document(metadata) = &self.node(document)?.kind else {
                            unreachable!("checked Document")
                        };
                        DomNodeKind::Element(DomElement::native(name, metadata, cancelled)?)
                    }
                    Creation::ElementNs { namespace, name } => {
                        DomNodeKind::Element(DomElement::native_ns(
                            namespace,
                            name,
                            self.native_environment(document)?,
                            cancelled,
                        )?)
                    }
                    Creation::Text(data) => DomNodeKind::Text(data),
                    Creation::Comment(data) => DomNodeKind::Comment(data),
                    Creation::ProcessingInstruction { target, data } => {
                        DomNodeKind::ProcessingInstruction(DomProcessingInstruction::new(
                            target, data, cancelled,
                        )?)
                    }
                    Creation::DocumentType {
                        name,
                        public_id,
                        system_id,
                    } => {
                        DomNodeKind::DocumentType(DomDocumentType::new(name, public_id, system_id)?)
                    }
                };
                let key = self.allocate(kind, document, limits)?;
                self.initialize_native_element(key, limits, cancelled)?;
                self.register_ticket(key)?;
            }
            Command::Import {
                snapshot,
                source,
                document,
                subtree,
            } => {
                let document = self.resolve(document)?;
                if !matches!(self.node(document)?.kind, DomNodeKind::Document(_)) {
                    return Err(DomError::WrongNodeKind);
                }
                let key =
                    self.import_node(&snapshot, source, document, subtree, limits, cancelled)?;
                self.register_ticket(key)?;
            }
            Command::Detach(node) => {
                let node = self.resolve(node)?;
                self.remove(node, limits, cancelled)?;
            }
            Command::Adopt { node, document } => {
                let node = self.resolve(node)?;
                let document = self.resolve(document)?;
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
                let node = self.resolve(node)?;
                self.set_attribute(node, namespace, name, value, limits, cancelled)?;
            }
            Command::SetCustomValidity { node, value } => {
                let node = self.resolve(node)?;
                self.set_custom_validity(node, value, limits, cancelled)?;
            }
            Command::SetNonce { node, value } => {
                let node = self.resolve(node)?;
                self.set_nonce(node, value, limits, cancelled)?;
            }
            Command::SetMarqueeRunning { node, running } => {
                let node = self.resolve(node)?;
                let DomNodeKind::Element(element) = &mut self.node_mut(node).kind else {
                    return Err(DomError::WrongNodeKind);
                };
                element.set_marquee_running(running)?;
            }
            Command::RemoveAttribute {
                node,
                namespace,
                name,
            } => {
                let node = self.resolve(node)?;
                self.remove_attribute(node, namespace, name, limits, cancelled)?;
            }
            Command::Destroy(node) => {
                let node = self.resolve(node)?;
                if self.node(node)?.parent.is_some()
                    || self.node(node)?.host.is_some()
                    || matches!(
                        self.node(node)?.kind,
                        DomNodeKind::Document(_) | DomNodeKind::ShadowRoot(_)
                    )
                {
                    return Err(DomError::InvalidDestruction);
                }
                let mut pending = Vec::new();
                let mut closure = Vec::new();
                reserve(&mut pending, self.nodes)?;
                reserve(&mut closure, self.nodes)?;
                pending.push(node);
                while let Some(key) = pending.pop() {
                    check_cancel(cancelled)?;
                    let current = self.node(key)?;
                    pending.extend(current.children.iter().rev().copied());
                    pending.extend(current.shadow_root);
                    pending.extend(current.template_contents);
                    closure.push(key);
                }
                // Run real remove/association effects while every referenced owner
                // still exists. Only then cancel all owned retired-target tasks.
                for &key in &closure {
                    while let Some(&child) = self.node(key)?.children.first() {
                        self.remove(child, limits, cancelled)?;
                    }
                }
                for &key in &closure {
                    self.cancel_select_tasks(key, cancelled)?;
                }
                for key in closure {
                    check_cancel(cancelled)?;
                    let node = match key {
                        Key::Existing(i) => self.existing[i].take(),
                        Key::Created(i) => self.created[i].take(),
                    }
                    .expect("owned closure");
                    self.nodes -= 1;
                    self.units -= kind_units(&node.kind)?;
                    self.units -= form::name_units(node.form_past_names.as_ref(), cancelled)?;
                    self.retire_form_relations(key, cancelled)?;
                    self.slot_retirement(key, limits, cancelled)?;
                    self.retire_reflected_target(key, cancelled)?;
                }
            }
            Command::ReplaceData {
                node,
                offset,
                count,
                data,
            } => {
                let node = self.resolve(node)?;
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
                let mut value = Vec::new();
                reserve(&mut value, length)?;
                value.extend_from_slice(&previous.code_units()[..offset]);
                value.extend_from_slice(data.code_units());
                value.extend_from_slice(&previous.code_units()[offset + removed..]);
                let value = DocumentString::from_utf16(value);
                let kind = match &self.node(node)?.kind {
                    DomNodeKind::Text(_) => DomNodeKind::Text(value),
                    DomNodeKind::Comment(_) => DomNodeKind::Comment(value),
                    DomNodeKind::ProcessingInstruction(instruction) => {
                        DomNodeKind::ProcessingInstruction(instruction.with_data(value, cancelled)?)
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
                if let Some(parent) = self.node(node)?.parent {
                    self.children_changed(parent, limits, cancelled)?;
                }
            }
            Command::Insert {
                node,
                parent,
                before,
            } => {
                let node = self.resolve(node)?;
                let parent = self.resolve(parent)?;
                let before = before.map(|id| self.resolve(id)).transpose()?;
                self.pre_insert(node, parent, before, limits, cancelled)?;
            }
            Command::ReplaceChild {
                node,
                parent,
                child,
            } => {
                let node = self.resolve(node)?;
                let parent = self.resolve(parent)?;
                let child = self.resolve(child)?;
                self.replace_child(node, parent, child, limits, cancelled)?;
            }
        }
        Ok(())
    }
    fn finish(self, dom: &Dom, cancelled: &AtomicBool) -> Result<Prepared, DomError> {
        self.finish_with_mapping(dom, cancelled, std::iter::empty())
            .map(|(prepared, _)| prepared)
    }

    fn finish_with_mapping(
        self,
        dom: &Dom,
        cancelled: &AtomicBool,
        requested: impl ExactSizeIterator<Item = Key>,
    ) -> Result<(Prepared, Vec<NodeId>), DomError> {
        let mut slots = Vec::new();
        let mut occupants = Vec::new();
        reserve(&mut slots, dom.state.slots.len())?;
        reserve(&mut occupants, dom.state.slots.len())?;
        for (index, old) in dom.state.slots.iter().enumerate() {
            check_cancel(cancelled)?;
            let destroyed = old.node.is_some() && self.existing[index].is_none();
            let generation = if destroyed {
                old.generation.checked_add(1).unwrap_or(old.generation)
            } else {
                old.generation
            };
            slots.push(Slot {
                generation,
                retired: old.retired || (destroyed && old.generation == u64::MAX),
                node: None,
            });
            occupants.push(self.existing[index].as_ref().map(|_| Key::Existing(index)));
        }
        let mut allocated_ids = Vec::new();
        reserve(&mut allocated_ids, self.created.len())?;
        for (index, node) in self.created.iter().enumerate() {
            check_cancel(cancelled)?;
            let Some(_) = node else {
                allocated_ids.push(None);
                continue;
            };
            let slot = occupants
                .iter()
                .enumerate()
                .position(|(i, key)| key.is_none() && !slots[i].retired);
            let slot = if let Some(slot) = slot {
                slot
            } else {
                reserve(&mut slots, 1)?;
                reserve(&mut occupants, 1)?;
                slots.push(Slot {
                    generation: 0,
                    retired: false,
                    node: None,
                });
                occupants.push(None);
                slots.len() - 1
            };
            occupants[slot] = Some(Key::Created(index));
            allocated_ids.push(Some(NodeId {
                owner: dom.owner,
                slot,
                generation: slots[slot].generation,
            }));
        }
        let id = |key| match key {
            Key::Existing(slot) => NodeId {
                owner: dom.owner,
                slot,
                generation: dom.state.slots[slot].generation,
            },
            Key::Created(index) => allocated_ids[index].expect("surviving creation reference"),
        };
        // Resolve requested private adapter identities only while the canonical
        // final allocation map exists. All fallible output precedes publication.
        let mut mapped = Vec::new();
        mapped
            .try_reserve_exact(requested.len())
            .map_err(|_| DomError::AllocationFailed)?;
        for key in requested {
            check_cancel(cancelled)?;
            self.node(key)?;
            mapped.push(id(key));
        }
        for (slot, key) in occupants.into_iter().enumerate() {
            check_cancel(cancelled)?;
            let Some(key) = key else {
                continue;
            };
            let source = self.node(key)?;
            let mut children = Vec::new();
            reserve(&mut children, source.children.len())?;
            children.extend(source.children.iter().copied().map(id));
            slots[slot].node = Some(Node {
                document: DocumentId(id(source.document)),
                kind: source.kind.clone(),
                script: source.script.map(|value| value.map(id)),
                parent: source.parent.map(id),
                shadow_root: source.shadow_root.map(id),
                host: source.host.map(id),
                template_contents: source.template_contents.map(id),
                template_document: source.template_document.map(|value| value.map(id)),
                children,
                manual_nodes: map_ids(&source.manual_nodes, &id, cancelled)?,
                assigned_nodes: map_ids(&source.assigned_nodes, &id, cancelled)?,
                manual_slot: source.manual_slot.map(id),
                assigned_slot: source.assigned_slot.map(id),
                reflected_targets: source.reflected_targets.map(|targets| targets.map(id)),
                control_owner: source.control_owner.map(id),
                option_select: source.option_select.map(id),
                form_past_names: source
                    .form_past_names
                    .as_ref()
                    .map(|map| form::map_names(map, id, cancelled))
                    .transpose()?,
            });
        }
        let select::FinishedTasks {
            tasks: native_tasks,
            next: next_task,
            cancellations,
        } = self.finish_select_tasks(dom, &id, cancelled)?;
        let finished = self.view_transitions.finish(dom.owner)?;
        let state = State {
            slots,
            view_transitions: finished.value,
            signal_slots: map_ids(&self.signal_slots, &id, cancelled)?,
            notification_queued: self.notification_queued,
            native_tasks,
            next_task,
            nodes: self.nodes,
            units: self.units,
        };
        let mut changes = difference(dom.owner, &dom.state, &state)?;
        for (operation, select) in cancellations {
            reserve(&mut changes, 1)?;
            changes.push(DomChange::SelectTaskCancelled {
                operation: crate::DomNativeTaskId {
                    owner: dom.owner,
                    serial: operation,
                },
                select,
                reason: crate::DomSelectTaskCancellationReason::TargetDestroyed,
            });
        }
        for slot in &self.cancelled_signals {
            reserve(&mut changes, 1)?;
            changes.push(DomChange::SlotSignalCancelled {
                slot: *slot,
                reason: crate::DomSlotSignalCancellationReason::Destroyed,
            });
        }
        let mut created = Vec::new();
        let mut documents = Vec::new();
        reserve(&mut created, self.tickets.len())?;
        reserve(&mut documents, self.tickets.len())?;
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
        Ok((
            Prepared {
                state,
                owner: self.owner,
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
fn map_keys(ids: &[NodeId], cancelled: &AtomicBool) -> Result<Vec<Key>, DomError> {
    let mut result = Vec::new();
    reserve(&mut result, ids.len())?;
    for id in ids {
        check_cancel(cancelled)?;
        result.push(Key::Existing(id.slot));
    }
    Ok(result)
}
fn map_ids(
    keys: &[Key],
    id: &impl Fn(Key) -> NodeId,
    cancelled: &AtomicBool,
) -> Result<Vec<NodeId>, DomError> {
    let mut result = Vec::new();
    reserve(&mut result, keys.len())?;
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
fn difference(owner: DomOwnerId, old: &State, new: &State) -> Result<Vec<DomChange>, DomError> {
    let mut changes = Vec::new();
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
                if let (DomNodeKind::Element(before), DomNodeKind::Element(after)) =
                    (&node.kind, &current.kind)
                    && before.captured_in_view_transition() != after.captured_in_view_transition()
                {
                    reserve(&mut changes, 1)?;
                    changes.push(DomChange::ViewTransitionCaptured(
                        DomViewTransitionSubject::Element(id),
                    ));
                }
                let intrinsic_changed = match (&node.kind, &current.kind) {
                    (DomNodeKind::Element(before), DomNodeKind::Element(after)) => {
                        before.attributes() != after.attributes()
                    }
                    _ => node.kind != current.kind,
                };
                if let (DomNodeKind::Element(before), DomNodeKind::Element(after)) =
                    (&node.kind, &current.kind)
                    && (!before.same_nonattribute_state(after)
                        || node.reflected_targets != current.reflected_targets
                        || node.control_owner != current.control_owner
                        || node.form_past_names != current.form_past_names
                        || node.script != current.script
                        || node.option_select != current.option_select)
                {
                    reserve(&mut changes, 1)?;
                    changes.push(DomChange::ElementState(id));
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
                    reserve(&mut changes, 1)?;
                    changes.push(change);
                }
            } else {
                reserve(&mut changes, 1)?;
                changes.push(DomChange::Destroyed(id));
            }
        }
        if let Some((generation, _)) = after
            && !same
        {
            reserve(&mut changes, 1)?;
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
            a.template_document != b.template_document || a.template_contents != b.template_contents
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
        reserve(&mut changes, 1)?;
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
        reserve(&mut changes, 1)?;
        changes.push(DomChange::ShadowRelations);
    }
    if old.signal_slots != new.signal_slots
        || old.notification_queued != new.notification_queued
        || old.native_tasks != new.native_tasks
    {
        reserve(&mut changes, 1)?;
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
        reserve(&mut changes, 1)?;
        changes.push(DomChange::SlotQueries);
    }
    new.view_transitions
        .changes(&old.view_transitions, &mut changes)?;
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
                reserve(&mut changes, 1)?;
                changes.push(change);
            }
        }
    }
    Ok(changes)
}

#[cfg(test)]
mod tests;
