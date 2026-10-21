//! Actual native Style composition and networking-task ownership.
//!
//! DOM owns each synchronous source continuation; CSS/CSSOM own parsing, recovery
//! and sheet identity. This owner binds actual root lists, services those source
//! requests, and queues completion only after a parsed sheet has no imports.
mod meta;
mod parsed_style;
mod sheet_sets;
pub use meta::parsed::{
    DomStyleParsedAdmissionError, DomStyleParsedInstallError, DomStyleParsedMetaError,
    DomStyleParsedMetaOutcome, DomStyleParsedMetaPreference,
};
pub use meta::{
    DomStyleMetaError, DomStyleMetaOrdinaryOutcome, DomStyleMetaPreference,
    DomStyleMetaSourceOutcome,
};
pub use parsed_style::DomStyleParsedPrefixError;
pub use sheet_sets::{DomStyleSheetSetCommit, DomStyleSheetSetObservation};

use crate::dom_event::{
    DomEventDispatchLimits, DomEventDispatchOutcome, DomEventDispatchReceipt, DomEventDispatcher,
    DomEventDom, NativeDomEvent, NativeDomEventOwner,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use surgeist_css::{
    CssParserContext, CssParserMode, CssRetainedPayloadError, CssRule, CssSourceRetention,
    CssUtf16InputLimits,
};
use surgeist_cssom::{
    CssomBatch, CssomCommit, CssomContext, CssomEditGuard, CssomInput, CssomInputData,
    CssomInputRole, CssomInputVersion, CssomOwnerMediaInput, CssomSheetAttributeEdit,
    CssomSheetConstruction, CssomSheetId, CssomSheetInputs, CssomSheetListTarget,
    CssomSheetOwnerAttributes, CssomSnapshot,
};
use surgeist_dom::{
    Dom, DomError, DomOwnerId, DomSnapshot, DomSourceLease, DomSourceOperationId, DomStyleAction,
    DomStyleCssomOwnerBinding, DomStyleCssomRoot, DomStyleCssomRootError,
    DomStyleNotificationCompletion, DomStyleNotificationToken, DomStyleRequest,
    DomStyleSheetPlacement, DomStyleSource, NodeId,
};
use surgeist_html::{
    HtmlDocumentMode, HtmlStyleNotificationAction, HtmlStyleNotificationCursor,
    HtmlStyleNotificationReply, HtmlStyleUpdateAction,
};

/// Independent orchestration bounds. CSS and CSSOM retain their own input,
/// graph and retained-source limits; dispatcher admission remains independent.
#[derive(Clone, Copy, Debug)]
pub struct DomStyleRuntimeLimits {
    /// Actual root-to-list bindings.
    pub roots: usize,
    /// Live waiting or queued completion records, including an executing task.
    pub completions: usize,
    /// Retained reports whose separate media diagnostics remain inspectable.
    pub reports: usize,
    /// Actual persistent Runtime payload/banks, labels, processing results and
    /// their shared source domain. DOM/CSSOM graph quotas remain producer-owned;
    /// temporary source-collector traversal storage is bounded by work.
    pub storage_bytes: usize,
    /// Monotone orchestration work across retries and source operations.
    pub work: u64,
    /// CSS-owned preparation and parsing admission for each real input.
    pub css_input: CssUtf16InputLimits,
}
/// A precise unavailable service or retained source boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomStyleUnavailable {
    /// No actual root stylesheet-list owner has been bound.
    RootList,
    /// Full current CSP policy/evaluation has not been supplied.
    InlinePolicy,
    /// The actual media/environment or rendering observation is unavailable.
    Environment,
    /// Imports exist; this owner has not obtained their critical-resource outcomes.
    CriticalResources,
    /// A native hosted object, operation, loop or task remains live.
    HostedLifecycle,
    /// A networking task is already executing at the FIFO head.
    RunningNetworkingTask,
}
/// Failure retains all acknowledged external work inside the composing owners.
#[derive(Debug)]
pub enum DomStyleRuntimeError {
    /// Original parsed association inputs remain under canonical DOM custody;
    /// no live #840 admission/execution service has consumed them.
    PendingParsedAssociations(surgeist_dom::DomParsedHtmlAssociationId),
    /// Canonical source admission or qualification failed.
    Dom(DomError),
    /// The CSSOM owner refused its actual mutation.
    Cssom(surgeist_cssom::CssomError),
    /// Real CSS input preparation/parsing failed, including cancellation.
    Css(surgeist_css::CssUtf16InputError<DomError>),
    /// Runtime's own finite bank/work/identity admission failed.
    Admission,
    /// A caller replaced an actual owner or supplied a conflicting binding.
    Binding,
    /// Source reply is inconsistent with the retained operation.
    Protocol,
    /// Actual shared DOM dispatch failed before advancing the task.
    Dispatch(crate::dom_event::DomEventDispatchError),
    /// The durable original session has ended.
    SessionUnavailable,
    /// Execution permission and canonical view belong to different sessions.
    WrongSession,
    /// A new session cannot reconstruct preexisting root/task custody.
    PreboundSession,
    /// The actual Document has no bound stylesheet association owner.
    SheetSetRootUnavailable {
        /// Exact live Document for which ownership is unavailable.
        root: NodeId,
    },
    /// Preference history belongs to Documents, not ShadowRoot or Element lists.
    SheetSetDocumentRequired {
        /// Actual non-Document root supplied by the caller.
        root: NodeId,
    },
    /// The canonical, preference or actual producer revision changed.
    StaleSheetSetObservation,
    /// An actual staged or committed Source publication retains exclusive custody.
    SheetSetPublicationPending,
    /// The one actual preference/producer output could not be admitted.
    SheetSetOutputLimit,
    /// An actual canonical source operation is already pending at bootstrap.
    SourcePending(DomSourceOperationId),
    /// The exact retained CSSOM resource is temporarily unavailable or refused work.
    Resource(DomStyleCssomRootError),
}
impl std::fmt::Display for DomStyleRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Style runtime operation refused: {self:?}")
    }
}
impl std::error::Error for DomStyleRuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            Self::Cssom(error) => Some(error),
            Self::Css(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Resource(error) => Some(error),
            _ => None,
        }
    }
}
impl From<DomStyleCssomRootError> for DomStyleRuntimeError {
    fn from(error: DomStyleCssomRootError) -> Self {
        Self::Resource(error)
    }
}
impl From<DomError> for DomStyleRuntimeError {
    fn from(e: DomError) -> Self {
        Self::Dom(e)
    }
}
impl From<surgeist_cssom::CssomError> for DomStyleRuntimeError {
    fn from(e: surgeist_cssom::CssomError) -> Self {
        Self::Cssom(e)
    }
}
/// Private result of servicing one actual yielded Style request.
pub(crate) enum DomStyleServiceProgress {
    Serviced,
    Pending {
        request: DomStyleRequest,
        reason: DomStyleUnavailable,
    },
}
struct Root {
    node: NodeId,
    label: String,
    store: std::rc::Rc<DomStyleCssomRoot>,
    serial: u64,
    sheet_sets: sheet_sets::RootSets,
}
/// Qualified processing output. Media diagnostics keep their own source
/// coordinate context; body diagnostics remain on the actual CSSOM sheet.
pub struct DomStyleProcessedSheet {
    root: NodeId,
    sheet: CssomSheetId,
    processing: Processing,
}
impl DomStyleProcessedSheet {
    /// Actual list root.
    pub fn root(&self) -> NodeId {
        self.root
    }
    /// Producer-issued sheet identity.
    pub fn sheet(&self) -> &CssomSheetId {
        &self.sheet
    }
    /// Recovered diagnostics from the actual separate media input.
    pub fn media_diagnostics(&self) -> &[surgeist_css::CssRecoveryDiagnostic] {
        match &self.processing {
            Processing::Created(c) => c.media_diagnostics(),
            Processing::Attributes(edit) => edit.diagnostics(),
        }
    }
}
/// One current Style disabled value, qualified by its exact canonical occurrence
/// and actual current CSSOM object. This does not certify resource readiness.
#[derive(Clone, Debug)]
pub struct DomStyleDisabledObservation {
    stamp: surgeist_dom::DomStamp,
    element: NodeId,
    association: Option<surgeist_dom::DomStyleAssociationId>,
    revision: Option<surgeist_cssom::CssomRevision>,
    disabled: bool,
}
impl DomStyleDisabledObservation {
    /// Canonical capture actually inspected.
    pub fn stamp(&self) -> surgeist_dom::DomStamp {
        self.stamp
    }
    /// Exact current target generation.
    pub fn element(&self) -> NodeId {
        self.element
    }
    /// Current association occurrence; None is an established null sheet.
    pub fn association(&self) -> Option<surgeist_dom::DomStyleAssociationId> {
        self.association
    }
    /// Current producer revision, distinct from the historical source receipt.
    pub fn cssom_revision(&self) -> Option<&surgeist_cssom::CssomRevision> {
        self.revision.as_ref()
    }
    /// Actual current sheet flag, or false for an established null sheet.
    pub fn disabled(&self) -> bool {
        self.disabled
    }
}
enum Processing {
    Created(CssomSheetConstruction),
    Attributes(CssomSheetAttributeEdit),
}
enum ProcessingRef<'a> {
    Created(&'a CssomSheetConstruction),
    Attributes(&'a CssomSheetAttributeEdit),
}
impl ProcessingRef<'_> {
    fn diagnostics(&self) -> &[surgeist_css::CssRecoveryDiagnostic] {
        match self {
            Self::Created(value) => value.media_diagnostics(),
            Self::Attributes(value) => value.diagnostics(),
        }
    }
    fn backing<E>(
        &self,
        admit: impl FnMut() -> Result<(), E>,
    ) -> Result<usize, CssRetainedPayloadError<E>> {
        match self {
            Self::Created(value) => value.retained_backing_bytes_with(admit),
            Self::Attributes(value) => value.retained_backing_bytes_with(admit),
        }
    }
}
fn payload_error(error: CssRetainedPayloadError<DomStyleRuntimeError>) -> DomStyleRuntimeError {
    match error {
        CssRetainedPayloadError::Admission(error) => error,
        CssRetainedPayloadError::CapacityOverflow => DomStyleRuntimeError::Admission,
    }
}
struct StagedPublication {
    request: DomStyleRequest,
    root: NodeId,
    root_index: usize,
    owner: std::rc::Rc<DomStyleCssomRoot>,
    batch: Option<CssomBatch>,
    serial: Option<u64>,
    created: Option<(CssomSheetConstruction, bool)>,
    attributes: Option<(CssomSheetId, CssomSheetAttributeEdit)>,
    sheet_sets: Option<sheet_sets::State>,
}
struct Publication {
    request: DomStyleRequest,
    root: NodeId,
    owner: std::rc::Rc<DomStyleCssomRoot>,
    commit: CssomCommit,
    snapshot: Option<CssomSnapshot>,
    created: Option<(CssomSheetId, CssomSheetConstruction, bool)>,
    attributes: Option<(CssomSheetId, CssomSheetAttributeEdit)>,
}
enum Delivery {
    NotStarted,
    Dispatching(NativeDomEventOwner),
    Delivered(DomEventDispatchReceipt),
}
struct Task {
    serial: u64,
    root: NodeId,
    owner: std::rc::Rc<DomStyleCssomRoot>,
    sheet: CssomSheetId,
    token: DomStyleNotificationToken,
    cursor: HtmlStyleNotificationCursor,
    delivery: Delivery,
    running: bool,
    dispatch_reserved: usize,
}
impl Task {
    fn id(&self) -> DomStyleNetworkingTaskId {
        DomStyleNetworkingTaskId {
            association: self.token.association(),
            serial: self.serial,
        }
    }
}
enum Completion {
    Waiting(Task),
    Queued(Task),
}
impl Completion {
    fn task(&self) -> &Task {
        match self {
            Self::Waiting(task) | Self::Queued(task) => task,
        }
    }
    fn task_mut(&mut self) -> &mut Task {
        match self {
            Self::Waiting(task) | Self::Queued(task) => task,
        }
    }
}
/// Exact task metadata; identity grants no independent execution authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomStyleNetworkingTaskId {
    association: surgeist_dom::DomStyleAssociationId,
    serial: u64,
}
/// Actual task delivery or a retained continuation.
#[derive(Debug)]
pub enum DomStyleNetworkingProgress {
    /// No queued networking task exists (critical-resource waits are separate).
    Empty,
    /// Delivery and the source's post-event canonical cleanup completed.
    Complete {
        /// Exact task that actually ran.
        task: DomStyleNetworkingTaskId,
        /// Actual completed shared dispatch.
        receipt: DomEventDispatchReceipt,
    },
    /// The exact target generation was retired before task dispatch began.
    TargetDestroyed {
        /// Exact canceled task identity.
        task: DomStyleNetworkingTaskId,
        /// Exact retired target generation.
        target: NodeId,
    },
    /// Actual dispatch retained its next invocation/listener after admission or reentry.
    Retained {
        /// Exact retained task identity.
        task: DomStyleNetworkingTaskId,
        /// Real dispatcher continuation status.
        dispatch: DomEventDispatchOutcome,
    },
    /// The currently running FIFO head cannot be executed a second time.
    Running(DomStyleNetworkingTaskId),
    /// Callback substitution cannot consume the original owner's exact task.
    /// Restore this permission in the composition context, then resume normally.
    RetainedAccess {
        /// Exact retained task identity.
        task: DomStyleNetworkingTaskId,
        /// Original session execution permission; this value does not own its resources.
        owner: DomStyleRuntime,
    },
}
/// The actual caller composition context supplies its existing Style owner.
/// This borrow ends before callbacks, allowing those callbacks to drive nested
/// canonical edits through the same owner without a second live DOM or registry.
pub trait DomStyleEventContext {
    /// Current Style composition owner.
    fn styles(&mut self) -> &mut DomStyleRuntime;
}
/// Owns real per-root CSSOM stores, recovery reports and networking continuations.
/// Dropping execution permission does not alter the DOM-owned synchronous tail.
struct StyleState {
    limits: DomStyleRuntimeLimits,
    roots: Vec<Root>,
    completions: Vec<Completion>,
    reports: Vec<Arc<DomStyleProcessedSheet>>,
    publication: Option<Publication>,
    staged: Option<StagedPublication>,
    next_task: u64,
    work: std::cell::Cell<u64>,
    dom: DomOwnerId,
}
// The current storage observation already includes the old backing. A full
// persistent bank additionally needs its entire requested replacement; spare
// capacity needs no replacement envelope.
fn persistent_bank_growth(
    length: usize,
    capacity: usize,
    element_bytes: usize,
) -> Result<(usize, usize), DomStyleRuntimeError> {
    if length < capacity {
        return Ok((0, 0));
    }
    let old = capacity
        .checked_mul(element_bytes)
        .ok_or(DomStyleRuntimeError::Admission)?;
    let requested = length
        .checked_add(1)
        .and_then(|count| count.checked_mul(element_bytes))
        .ok_or(DomStyleRuntimeError::Admission)?;
    Ok((old, requested))
}
impl StyleState {
    /// Construct an empty finite integration owner, without binding a fake root/list.
    pub fn new(dom: DomOwnerId, limits: DomStyleRuntimeLimits) -> Self {
        Self {
            limits,
            roots: Vec::new(),
            completions: Vec::new(),
            reports: Vec::new(),
            publication: None,
            staged: None,
            next_task: 1,
            work: std::cell::Cell::new(0),
            dom,
        }
    }
    fn step(&self, cancel: &AtomicBool) -> Result<(), DomStyleRuntimeError> {
        self.step_with_work(cancel, self.limits.work)
    }
    fn step_with_work(
        &self,
        cancel: &AtomicBool,
        maximum: u64,
    ) -> Result<(), DomStyleRuntimeError> {
        Self::step_counter(&self.work, cancel, maximum)
    }
    fn step_counter(
        work: &std::cell::Cell<u64>,
        cancel: &AtomicBool,
        maximum: u64,
    ) -> Result<(), DomStyleRuntimeError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(DomError::Cancelled.into());
        }
        let next = work
            .get()
            .checked_add(1)
            .ok_or(DomStyleRuntimeError::Admission)?;
        work.set(next);
        if next > maximum {
            return Err(DomStyleRuntimeError::Admission);
        }
        Ok(())
    }
    fn processing(
        &self,
        candidate: Option<&StagedPublication>,
        mut visit: impl FnMut(ProcessingRef<'_>) -> Result<(), DomStyleRuntimeError>,
    ) -> Result<(), DomStyleRuntimeError> {
        for report in &self.reports {
            visit(match &report.processing {
                Processing::Created(value) => ProcessingRef::Created(value),
                Processing::Attributes(value) => ProcessingRef::Attributes(value),
            })?;
        }
        for stage in self.staged.iter().chain(candidate) {
            if let Some((value, _)) = &stage.created {
                visit(ProcessingRef::Created(value))?;
            }
            if let Some((_, value)) = &stage.attributes {
                visit(ProcessingRef::Attributes(value))?;
            }
        }
        if let Some(publication) = &self.publication {
            if let Some((_, value, _)) = &publication.created {
                visit(ProcessingRef::Created(value))?;
            }
            if let Some((_, value)) = &publication.attributes {
                visit(ProcessingRef::Attributes(value))?;
            }
        }
        Ok(())
    }
    fn storage_headers(
        &self,
        candidate: Option<&StagedPublication>,
        mut step: impl FnMut() -> Result<(), DomStyleRuntimeError>,
    ) -> Result<usize, DomStyleRuntimeError> {
        fn add(total: &mut usize, value: usize) -> Result<(), DomStyleRuntimeError> {
            *total = total
                .checked_add(value)
                .ok_or(DomStyleRuntimeError::Admission)?;
            Ok(())
        }
        step()?;
        let mut bytes = size_of::<std::cell::RefCell<StyleState>>();
        for (capacity, element) in [
            (self.roots.capacity(), size_of::<Root>()),
            (self.completions.capacity(), size_of::<Completion>()),
            (
                self.reports.capacity(),
                size_of::<Arc<DomStyleProcessedSheet>>(),
            ),
        ] {
            add(
                &mut bytes,
                capacity
                    .checked_mul(element)
                    .ok_or(DomStyleRuntimeError::Admission)?,
            )?;
        }
        for root in &self.roots {
            step()?;
            add(&mut bytes, root.label.capacity())?;
            add(&mut bytes, root.sheet_sets.backing_bytes()?)?;
            add(&mut bytes, size_of::<DomStyleCssomRoot>())?;
        }
        for (index, completion) in self.completions.iter().enumerate() {
            step()?;
            let task = completion.task();
            let mut counted = false;
            for root in &self.roots {
                step()?;
                if std::rc::Rc::ptr_eq(&root.store, &task.owner) {
                    counted = true;
                    break;
                }
            }
            if !counted {
                for previous in &self.completions[..index] {
                    step()?;
                    if std::rc::Rc::ptr_eq(&previous.task().owner, &task.owner) {
                        counted = true;
                        break;
                    }
                }
            }
            if !counted {
                add(&mut bytes, size_of::<DomStyleCssomRoot>())?;
            }
            match &task.delivery {
                Delivery::NotStarted => {}
                Delivery::Dispatching(event) => add(
                    &mut bytes,
                    event
                        .retained_payload_bytes()
                        .ok_or(DomStyleRuntimeError::Admission)?
                        .max(task.dispatch_reserved),
                )?,
                Delivery::Delivered(receipt) => add(&mut bytes, receipt.retained_backing_bytes())?,
            }
        }
        for stage in self.staged.iter().chain(candidate) {
            if let Some(next) = &stage.sheet_sets {
                let mut previous = None;
                for root in &self.roots {
                    step()?;
                    if root.node == stage.root {
                        previous = root.sheet_sets.state();
                        break;
                    }
                }
                add(&mut bytes, next.additional_backing(previous)?)?;
            }
        }
        // Each Arc payload is formed once by this private bank. Public historical
        // observations cannot insert aliases back into it; count its Self once.
        for _ in &self.reports {
            step()?;
            add(&mut bytes, size_of::<DomStyleProcessedSheet>())?;
        }
        self.processing(candidate, |value| {
            step()?;
            add(&mut bytes, value.backing(&mut step).map_err(payload_error)?)
        })?;
        Ok(bytes)
    }
    fn storage(
        &self,
        candidate: Option<&StagedPublication>,
        cancel: &AtomicBool,
        work_limit: u64,
    ) -> Result<usize, DomStyleRuntimeError> {
        let bytes = self.storage_headers(candidate, || self.step_with_work(cancel, work_limit))?;
        let remaining = work_limit.saturating_sub(self.work.get());
        let mut sources = CssSourceRetention::new(
            usize::MAX,
            usize::try_from(remaining).unwrap_or(usize::MAX),
            usize::try_from(remaining).unwrap_or(usize::MAX),
        );
        // The existing collector owns its visit tariff. Bill that exact shared
        // bank once, including its consumed prefix on refusal, without inventing
        // a raw-length tariff or duplicating private source storage layouts.
        let result = self.processing(candidate, |value| {
            if cancel.load(Ordering::Relaxed) {
                return Err(DomError::Cancelled.into());
            }
            sources
                .charge_step()
                .map_err(|_| DomStyleRuntimeError::Admission)?;
            for diagnostic in value.diagnostics() {
                if cancel.load(Ordering::Relaxed) {
                    return Err(DomError::Cancelled.into());
                }
                sources
                    .visit(diagnostic)
                    .map_err(|_| DomStyleRuntimeError::Admission)?;
            }
            Ok(())
        });
        self.work.set(
            self.work
                .get()
                .checked_add(
                    u64::try_from(sources.steps()).map_err(|_| DomStyleRuntimeError::Admission)?,
                )
                .ok_or(DomStyleRuntimeError::Admission)?,
        );
        result?;
        bytes
            .checked_add(
                sources
                    .complete_payload_bytes_with(|| self.step_with_work(cancel, work_limit))
                    .map_err(payload_error)?,
            )
            .ok_or(DomStyleRuntimeError::Admission)
    }
    fn admit(&self, extra: usize, cancel: &AtomicBool) -> Result<(), DomStyleRuntimeError> {
        if self
            .storage(None, cancel, self.limits.work)?
            .checked_add(extra)
            .is_none_or(|bytes| bytes > self.limits.storage_bytes)
        {
            return Err(DomStyleRuntimeError::Admission);
        }
        Ok(())
    }
    fn admit_staged(
        &self,
        candidate: &StagedPublication,
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleRuntimeError> {
        let extra = if candidate.created.is_some() || candidate.attributes.is_some() {
            size_of::<DomStyleProcessedSheet>()
        } else {
            0
        };
        if self
            .storage(Some(candidate), cancel, self.limits.work)?
            .checked_add(extra)
            .is_none_or(|bytes| bytes > self.limits.storage_bytes)
        {
            return Err(DomStyleRuntimeError::Admission);
        }
        Ok(())
    }
    fn prune_retired_roots(
        &mut self,
        snapshot: &DomSnapshot,
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleRuntimeError> {
        if snapshot.incomplete_source_operation().is_some() {
            return Ok(());
        }
        let mut index = 0;
        while index < self.roots.len() {
            self.step(cancel)?;
            match snapshot.node(self.roots[index].node) {
                Err(DomError::StaleNode) => {
                    self.roots.remove(index);
                }
                Err(error) => return Err(error.into()),
                Ok(_) => index += 1,
            }
        }
        Ok(())
    }
    fn admit_context_clone(
        &self,
        context: &CssomContext,
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleRuntimeError> {
        fn string(
            state: &StyleState,
            value: &str,
            cancel: &AtomicBool,
        ) -> Result<(), DomStyleRuntimeError> {
            state.step(cancel)?;
            for _ in 0..value.len() {
                state.step(cancel)?;
            }
            Ok(())
        }
        // These immutable public operands are precisely what Context::clone
        // copies. Admit every row, String header/byte and nested Vec entry
        // before cloning; linked snapshots remain Arc observations, not copies
        // of their graphs. CSSOM retains ownership of graph/storage limits.
        for input in &context.inputs {
            self.step(cancel)?;
            string(self, &input.version.identity, cancel)?;
            match &input.data {
                CssomInputData::Document { identity } => string(self, identity, cancel)?,
                CssomInputData::Window { identity, document } => {
                    string(self, identity, cancel)?;
                    string(self, document, cancel)?;
                }
                CssomInputData::ParserMode(_) => {}
                CssomInputData::Origin { origin, profile } => {
                    string(self, origin, cancel)?;
                    string(self, profile, cancel)?;
                }
                CssomInputData::Layer { path, .. } => {
                    for component in path {
                        string(self, component, cancel)?;
                    }
                }
                CssomInputData::Base { url, location } => {
                    for value in [url, location] {
                        self.step(cancel)?;
                        if let Some(value) = value {
                            string(self, value, cancel)?;
                        }
                    }
                }
                CssomInputData::Owner {
                    identity,
                    attribute,
                } => {
                    string(self, identity, cancel)?;
                    self.step(cancel)?;
                    if let Some(value) = attribute {
                        string(self, value, cancel)?;
                    }
                }
                CssomInputData::Support(value) => {
                    for _ in value.properties() {
                        self.step(cancel)?;
                    }
                    for _ in value.page_descriptors() {
                        self.step(cancel)?;
                    }
                    for _ in value.font_face_descriptors() {
                        self.step(cancel)?;
                    }
                }
                CssomInputData::Import { resolved_location } => {
                    self.step(cancel)?;
                    if let Some(value) = resolved_location {
                        string(self, value, cancel)?;
                    }
                }
            }
        }
        for linked in &context.linked {
            self.step(cancel)?;
            string(self, &linked.version.identity, cancel)?;
        }
        Ok(())
    }
    fn source_parser_context(
        &self,
        snapshot: &CssomSnapshot,
        source: &DomStyleSource,
        root_label: &str,
        batch: &mut CssomBatch,
        cancel: &AtomicBool,
    ) -> Result<CssParserContext, DomStyleRuntimeError> {
        let mut existing = None;
        for (index, input) in snapshot.context().inputs.iter().enumerate() {
            self.step(cancel)?;
            if let CssomInputData::ParserMode(context) = input.data {
                existing = Some((index, context));
                break;
            }
        }
        let current = existing.map_or_else(CssParserContext::default, |(_, context)| context);
        let mode = match source.document_metadata().mode() {
            HtmlDocumentMode::Quirks => CssParserMode::Quirks,
            HtmlDocumentMode::LimitedQuirks | HtmlDocumentMode::NoQuirks => {
                CssParserMode::Standards
            }
        };
        if current.mode() == mode {
            return Ok(current);
        }
        let selected = current.with_mode(mode);
        // This is this composition's operation-input version, not an external
        // producer revision or a prediction/coverage claim about a future CSSOM
        // publication. Both actually captured owners participate: the canonical
        // source stamp names the DOM basis; the current owner-qualified CSSOM
        // counter distinguishes external profile edits at an unchanged DOM stamp.
        let revision = snapshot
            .revision()
            .value()
            .checked_add(1)
            .ok_or(DomStyleRuntimeError::Admission)?;
        let mut identity = String::new();
        identity
            .try_reserve_exact(
                root_label
                    .len()
                    .checked_add(40)
                    .ok_or(DomStyleRuntimeError::Admission)?,
            )
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        for character in root_label.chars() {
            self.step(cancel)?;
            identity.push(character);
        }
        use std::fmt::Write;
        write!(
            &mut identity,
            ":document-mode:{}",
            source.stamp().revision().get()
        )
        .map_err(|_| DomStyleRuntimeError::Admission)?;
        self.admit_context_clone(snapshot.context(), cancel)?;
        // CSSOM owns this staged configuration and its graph quotas. Preserve
        // all unrelated rows/linked objects and this parser's non-mode settings.
        let mut context = snapshot.context().clone();
        let input = CssomInput {
            version: CssomInputVersion {
                role: CssomInputRole::ParserMode,
                identity,
                revision,
            },
            data: CssomInputData::ParserMode(selected),
        };
        if let Some((index, _)) = existing {
            context.inputs[index] = input;
        } else {
            context
                .inputs
                .try_reserve_exact(1)
                .map_err(|_| DomStyleRuntimeError::Admission)?;
            context.inputs.push(input);
        }
        // Report admission and the context selection are in this same actual
        // batch. Its base guard refuses a concurrently replaced configuration.
        batch.update_context(context)?;
        Ok(selected)
    }
    /// Bind an actual initially empty CSSOM list to its exact canonical root.
    /// Existing Runtime-created sheets remain accessible through `snapshot`.
    pub fn bind_root(
        &mut self,
        snapshot: &DomSnapshot,
        label: String,
        input: &mut Option<DomStyleCssomRoot>,
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleRuntimeError> {
        self.step(cancel)?;
        if snapshot.owner() != self.dom {
            return Err(DomStyleRuntimeError::WrongSession);
        }
        self.prune_retired_roots(snapshot, cancel)?;
        let capability = input.as_ref().ok_or(DomStyleRuntimeError::Binding)?;
        let root = capability.root();
        let node = snapshot.node(root)?;
        if !matches!(
            node.kind(),
            surgeist_dom::DomNodeKind::Document(_) | surgeist_dom::DomNodeKind::ShadowRoot(_)
        ) || label.is_empty()
        {
            return Err(DomStyleRuntimeError::Binding);
        }
        for existing in &self.roots {
            self.step(cancel)?;
            if existing.node == root {
                return Err(DomStyleRuntimeError::Binding);
            }
            if existing.label.len() == label.len() {
                let mut equal = true;
                for (left, right) in existing.label.bytes().zip(label.bytes()) {
                    self.step(cancel)?;
                    if left != right {
                        equal = false;
                        break;
                    }
                }
                if equal {
                    return Err(DomStyleRuntimeError::Binding);
                }
            }
        }
        if !capability.snapshot()?.sheets().is_empty() {
            return Err(DomStyleRuntimeError::Binding);
        }
        if self.roots.len() >= self.limits.roots {
            return Err(DomStyleRuntimeError::Admission);
        }
        let document = matches!(node.kind(), surgeist_dom::DomNodeKind::Document(_));
        let incoming = label
            .capacity()
            .checked_add(if document { size_of::<Vec<u16>>() } else { 0 })
            .and_then(|n| n.checked_add(size_of::<DomStyleCssomRoot>()))
            .ok_or(DomStyleRuntimeError::Admission)?;
        let (old_bank, replacement) =
            persistent_bank_growth(self.roots.len(), self.roots.capacity(), size_of::<Root>())?;
        self.admit(
            replacement
                .checked_add(incoming)
                .ok_or(DomStyleRuntimeError::Admission)?,
            cancel,
        )?;
        self.roots
            .try_reserve_exact(1)
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        // storage() now sees the actual resulting capacity. Include the old
        // backing again to reconcile the very same replacement boundary.
        self.admit(
            old_bank
                .checked_add(incoming)
                .ok_or(DomStyleRuntimeError::Admission)?,
            cancel,
        )?;
        let store = std::rc::Rc::new(input.take().expect("validated retained input"));
        self.roots.push(Root {
            node: root,
            label,
            store,
            serial: 0,
            sheet_sets: sheet_sets::RootSets::new(document),
        });
        Ok(())
    }
    /// Current actual CSSOM capture; this alone makes no joined readiness claim.
    pub fn snapshot(&self, root: NodeId) -> Result<Option<CssomSnapshot>, DomStyleRuntimeError> {
        self.roots
            .iter()
            .find(|r| r.node == root)
            .map(|r| r.store.snapshot().map_err(Into::into))
            .transpose()
    }
    /// Critical-resource waits, distinct from networking tasks already queued.
    pub fn waiting_resources(&self) -> usize {
        self.completions
            .iter()
            .filter(|c| matches!(c, Completion::Waiting(_)))
            .count()
    }
    fn reserve_output(&mut self, cancel: &AtomicBool) -> Result<(), DomStyleRuntimeError> {
        if self.completions.len() >= self.limits.completions
            || self.reports.len() >= self.limits.reports
        {
            return Err(DomStyleRuntimeError::Admission);
        }
        self.next_task
            .checked_add(1)
            .ok_or(DomStyleRuntimeError::Admission)?;
        let (old_completions, replacement_completions) = persistent_bank_growth(
            self.completions.len(),
            self.completions.capacity(),
            size_of::<Completion>(),
        )?;
        let (_, replacement_reports) = persistent_bank_growth(
            self.reports.len(),
            self.reports.capacity(),
            size_of::<Arc<DomStyleProcessedSheet>>(),
        )?;
        // The allocations happen sequentially: the old completion bank is gone
        // before report growth. Preflight both requested boundaries before
        // either reserve; do not pretend both replacements coexist.
        let completion_growth = replacement_completions
            .checked_sub(old_completions)
            .ok_or(DomStyleRuntimeError::Admission)?;
        let reports_boundary = completion_growth
            .checked_add(replacement_reports)
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.admit(replacement_completions.max(reports_boundary), cancel)?;
        self.completions
            .try_reserve_exact(1)
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        self.admit(old_completions, cancel)?;
        self.reserve_reports(0, cancel)
    }
    fn reserve_reports(
        &mut self,
        incoming: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleRuntimeError> {
        if self.reports.len() >= self.limits.reports {
            return Err(DomStyleRuntimeError::Admission);
        }
        let (old_bank, replacement) = persistent_bank_growth(
            self.reports.len(),
            self.reports.capacity(),
            size_of::<Arc<DomStyleProcessedSheet>>(),
        )?;
        self.admit(
            replacement
                .checked_add(incoming)
                .ok_or(DomStyleRuntimeError::Admission)?,
            cancel,
        )?;
        self.reports
            .try_reserve_exact(1)
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        self.admit(
            old_bank
                .checked_add(incoming)
                .ok_or(DomStyleRuntimeError::Admission)?,
            cancel,
        )
    }
    fn commit_staged(&mut self) -> Result<(), DomStyleRuntimeError> {
        let staged = self.staged.as_mut().ok_or(DomStyleRuntimeError::Protocol)?;
        let root = self
            .roots
            .get(staged.root_index)
            .ok_or(DomStyleRuntimeError::Binding)?;
        if root.node != staged.root || !std::rc::Rc::ptr_eq(&root.store, &staged.owner) {
            return Err(DomStyleRuntimeError::Binding);
        }
        // Busy/Poisoned never consumes the actual unattempted producer batch.
        let commit = match staged.owner.commit(&mut staged.batch) {
            Ok(commit) => commit,
            Err(error @ DomStyleCssomRootError::Cssom(_)) => {
                // The producer rejected its attempted atomic batch. No external
                // publication was acknowledged; reforming on an exact source
                // retry is permitted, unlike replaying a committed publication.
                self.staged.take();
                return Err(error.into());
            }
            Err(error) => return Err(error.into()),
        };
        let staged = self.staged.take().expect("commit retained stage");
        let root = &mut self.roots[staged.root_index];
        if let Some(serial) = staged.serial {
            root.serial = serial;
        }
        if let Some(preferences) = staged.sheet_sets {
            root.sheet_sets.publish(preferences);
        }
        let created = staged.created.map(|(construction, imports)| {
            let sheet = commit
                .sheet(construction.ticket())
                .expect("actual committed construction")
                .clone();
            (sheet, construction, imports)
        });
        // Retain the real commit before snapshot acquisition or any polling.
        self.publication = Some(Publication {
            request: staged.request,
            root: staged.root,
            owner: staged.owner,
            commit,
            snapshot: None,
            created,
            attributes: staged.attributes,
        });
        Ok(())
    }
    fn acknowledge(&mut self, lease: &mut DomSourceLease<'_>) -> Result<(), DomStyleRuntimeError> {
        let publication = self
            .publication
            .as_mut()
            .ok_or(DomStyleRuntimeError::Protocol)?;
        if publication.snapshot.is_none() {
            publication.snapshot = Some(publication.owner.snapshot()?);
        }
        let Publication {
            request,
            root,
            owner,
            commit,
            snapshot,
            created,
            attributes,
        } = self.publication.take().expect("retained publication");
        let ticket = created
            .as_ref()
            .map(|(_, construction, _)| construction.ticket());
        match lease.accept_cssom_publication(
            &request,
            commit,
            snapshot.expect("captured current producer"),
            ticket,
        ) {
            Err(rejected) => {
                let (error, commit, snapshot) = rejected.into_parts();
                self.publication = Some(Publication {
                    request,
                    root,
                    owner,
                    commit,
                    snapshot: Some(snapshot),
                    created,
                    attributes,
                });
                Err(error.into())
            }
            Ok(token) => {
                if let Some((sheet, construction, imports)) = created {
                    let token = token.ok_or(DomStyleRuntimeError::Protocol)?;
                    let serial = self.next_task;
                    self.next_task += 1;
                    let task = Task {
                        serial,
                        root,
                        owner,
                        sheet: sheet.clone(),
                        token,
                        cursor: HtmlStyleNotificationCursor::new(true),
                        delivery: Delivery::NotStarted,
                        running: false,
                        dispatch_reserved: 0,
                    };
                    self.completions.push(if imports {
                        Completion::Waiting(task)
                    } else {
                        Completion::Queued(task)
                    });
                    self.reports.push(Arc::new(DomStyleProcessedSheet {
                        root,
                        sheet,
                        processing: Processing::Created(construction),
                    }));
                }
                if let Some((sheet, edit)) = attributes {
                    self.reports.push(Arc::new(DomStyleProcessedSheet {
                        root,
                        sheet,
                        processing: Processing::Attributes(edit),
                    }));
                }
                Ok(())
            }
        }
    }
    fn flush_source(
        &mut self,
        lease: &mut DomSourceLease<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleRuntimeError> {
        self.step(cancel)?;
        if self.staged.is_some() {
            self.commit_staged()?;
        }
        if self.publication.is_some() {
            self.acknowledge(lease)?;
        }
        Ok(())
    }
    fn service_source_request(
        &mut self,
        lease: &mut DomSourceLease<'_>,
        request: DomStyleRequest,
        cancel: &AtomicBool,
    ) -> Result<DomStyleServiceProgress, DomStyleRuntimeError> {
        self.step(cancel)?;
        match request.action() {
            DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp) => {
                return Ok(DomStyleServiceProgress::Pending {
                    request,
                    reason: DomStyleUnavailable::InlinePolicy,
                });
            }
            DomStyleAction::Update(HtmlStyleUpdateAction::CreateSheet)
            | DomStyleAction::Update(HtmlStyleUpdateAction::RemoveAssociatedSheet) => {
                let root = request.cssom_root().ok_or(DomStyleRuntimeError::Protocol)?;
                let Some(index) = self.roots.iter().position(|r| r.node == root) else {
                    return Ok(DomStyleServiceProgress::Pending {
                        request,
                        reason: DomStyleUnavailable::RootList,
                    });
                };
                lease.bind_cssom_root(&request, &self.roots[index].store, cancel)?;
                let snapshot = self.roots[index].store.snapshot()?;
                let request = lease.qualify_cssom(&request, root, &snapshot)?;
                let mut batch = self.roots[index]
                    .store
                    .batch(CssomEditGuard::from_snapshot(&snapshot))?;
                let mut sheet_sets = None;
                let created = if request.action()
                    == DomStyleAction::Update(HtmlStyleUpdateAction::CreateSheet)
                {
                    self.reserve_output(cancel)?;
                    let root = &self.roots[index];
                    let serial = root
                        .serial
                        .checked_add(1)
                        .ok_or(DomStyleRuntimeError::Admission)?;
                    let mut label = String::new();
                    label
                        .try_reserve_exact(
                            root.label
                                .len()
                                .checked_add(21)
                                .ok_or(DomStyleRuntimeError::Admission)?,
                        )
                        .map_err(|_| DomStyleRuntimeError::Admission)?;
                    for character in root.label.chars() {
                        self.step(cancel)?;
                        label.push(character);
                    }
                    label.push(':');
                    use std::fmt::Write;
                    write!(&mut label, "{serial}").map_err(|_| DomStyleRuntimeError::Admission)?;
                    let version = CssomInputVersion {
                        role: CssomInputRole::Owner,
                        identity: label.clone(),
                        revision: serial,
                    };
                    lease.bind_cssom_owner(
                        &request,
                        DomStyleCssomOwnerBinding::new(&request, version.clone(), label.clone())?,
                        cancel,
                    )?;
                    let poll = || {
                        if cancel.load(Ordering::Relaxed) {
                            Err(DomError::Cancelled)
                        } else {
                            Ok(())
                        }
                    };
                    let context = self.source_parser_context(
                        &snapshot,
                        request.source(),
                        &root.label,
                        &mut batch,
                        cancel,
                    )?;
                    let report = context
                        .parse_sheet_utf16_with_limits(
                            request.source().text().clone(),
                            self.limits.css_input,
                            poll,
                        )
                        .map_err(DomStyleRuntimeError::Css)?;
                    // HTML's critical CSS subresources are imported sheets;
                    // no @import means no transitive import graph exists.
                    let imports = report
                        .syntax()
                        .rules()
                        .iter()
                        .any(|r| matches!(r, CssRule::Import(_)));
                    let media = context
                        .parse_media_query_list_utf16_with_limits(
                            request
                                .source()
                                .media()
                                .map(|value| value.shared_code_units().clone())
                                .unwrap_or_default(),
                            self.limits.css_input,
                            poll,
                        )
                        .map_err(DomStyleRuntimeError::Css)?;
                    let mut inputs = CssomSheetInputs::external(version, true);
                    inputs.owner = Some(label);
                    if matches!(
                        request.source().root_kind(),
                        surgeist_dom::DomStyleRootKind::Document
                    ) && let Some(title) = request.source().title()
                    {
                        inputs.title = title.shared_code_units().clone();
                    }
                    sheet_sets = self.prepare_added_sheet_sets(
                        index,
                        &snapshot,
                        &mut batch,
                        &mut inputs,
                        cancel,
                    )?;
                    let construction =
                        batch.create_parsed_sheet_with_media_reports(report, media, inputs)?;

                    let list = snapshot.sheet_list().clone();
                    let bytes = request
                        .sheet_order()
                        .len()
                        .checked_mul(size_of::<CssomSheetListTarget<'_>>())
                        .ok_or(DomStyleRuntimeError::Admission)?;
                    self.admit(bytes, cancel)?;
                    let mut members = Vec::new();
                    members
                        .try_reserve_exact(request.sheet_order().len())
                        .map_err(|_| DomStyleRuntimeError::Admission)?;
                    self.admit(
                        members
                            .capacity()
                            .checked_mul(size_of::<CssomSheetListTarget<'_>>())
                            .ok_or(DomStyleRuntimeError::Admission)?,
                        cancel,
                    )?;
                    for entry in request.sheet_order() {
                        self.step(cancel)?;
                        members.push(match entry {
                            DomStyleSheetPlacement::Existing(id) => {
                                CssomSheetListTarget::Existing(id)
                            }
                            DomStyleSheetPlacement::ThisStyle => {
                                CssomSheetListTarget::Created(construction.ticket())
                            }
                        });
                    }
                    batch.set_sheet_list_membership(&list, &members)?;
                    Some((construction, imports))
                } else {
                    batch.remove_sheet(
                        request.sheet().ok_or(DomStyleRuntimeError::Protocol)?.id(),
                    )?;
                    None
                };
                let staged = StagedPublication {
                    request,
                    root,
                    root_index: index,
                    owner: self.roots[index].store.clone(),
                    batch: Some(batch),
                    serial: created.as_ref().map(|_| self.roots[index].serial + 1),
                    created,
                    attributes: None,
                    sheet_sets,
                };
                self.admit_staged(&staged, cancel)?;
                self.staged = Some(staged);
                self.commit_staged()?;
                self.acknowledge(lease)?;
            }
            DomStyleAction::SetDisabled(disabled) => {
                let root = request.cssom_root().ok_or(DomStyleRuntimeError::Protocol)?;
                let Some(index) = self.roots.iter().position(|entry| entry.node == root) else {
                    return Ok(DomStyleServiceProgress::Pending {
                        request,
                        reason: DomStyleUnavailable::RootList,
                    });
                };
                lease.bind_cssom_root(&request, &self.roots[index].store, cancel)?;
                let snapshot = self.roots[index].store.snapshot()?;
                let request = lease.qualify_cssom(&request, root, &snapshot)?;
                let sheet = request.sheet().ok_or(DomStyleRuntimeError::Protocol)?;
                let mut batch = self.roots[index]
                    .store
                    .batch(CssomEditGuard::from_snapshot(&snapshot))?;
                batch.set_disabled(sheet.id(), disabled)?;
                let staged = StagedPublication {
                    request,
                    root,
                    root_index: index,
                    owner: self.roots[index].store.clone(),
                    batch: Some(batch),
                    serial: None,
                    created: None,
                    attributes: None,
                    sheet_sets: None,
                };
                self.admit_staged(&staged, cancel)?;
                self.staged = Some(staged);
                self.commit_staged()?;
                self.acknowledge(lease)?;
            }
            DomStyleAction::OwnerAttributes => {
                let root_id = request.cssom_root().ok_or(DomStyleRuntimeError::Protocol)?;
                let Some(index) = self.roots.iter().position(|r| r.node == root_id) else {
                    return Ok(DomStyleServiceProgress::Pending {
                        request,
                        reason: DomStyleUnavailable::RootList,
                    });
                };
                self.reserve_reports(size_of::<DomStyleProcessedSheet>(), cancel)?;
                lease.bind_cssom_root(&request, &self.roots[index].store, cancel)?;
                let root = &self.roots[index];
                let snapshot = root.store.snapshot()?;
                let request = lease.qualify_cssom(&request, root_id, &snapshot)?;
                let sheet_id = request
                    .sheet()
                    .ok_or(DomStyleRuntimeError::Protocol)?
                    .id()
                    .clone();
                let sheet = snapshot.sheet(&sheet_id)?;
                let owner = sheet
                    .inputs()
                    .owner
                    .as_ref()
                    .ok_or(DomStyleRuntimeError::Binding)?
                    .clone();
                let mut version = sheet.inputs().version.clone();
                version.revision = version
                    .revision
                    .checked_add(1)
                    .ok_or(DomStyleRuntimeError::Admission)?;
                lease.bind_cssom_owner(
                    &request,
                    DomStyleCssomOwnerBinding::new(&request, version.clone(), owner.clone())?,
                    cancel,
                )?;
                let mut batch = root.store.batch(CssomEditGuard::from_snapshot(&snapshot))?;
                let context = self.source_parser_context(
                    &snapshot,
                    request.source(),
                    &root.label,
                    &mut batch,
                    cancel,
                )?;
                let report = context
                    .parse_media_query_list_utf16_with_limits(
                        request
                            .source()
                            .media()
                            .map(|value| value.shared_code_units().clone())
                            .unwrap_or_default(),
                        self.limits.css_input,
                        || {
                            if cancel.load(Ordering::Relaxed) {
                                Err(DomError::Cancelled)
                            } else {
                                Ok(())
                            }
                        },
                    )
                    .map_err(DomStyleRuntimeError::Css)?;
                let title = (request.source().root_kind()
                    == surgeist_dom::DomStyleRootKind::Document)
                    .then(|| request.source().title().map_or(&[][..], |s| s.code_units()));
                let edit = batch.update_sheet_owner_attributes(
                    &sheet_id,
                    CssomSheetOwnerAttributes {
                        owner: &owner,
                        version,
                        title,
                        media: CssomOwnerMediaInput::Parsed(report),
                    },
                )?;
                let staged = StagedPublication {
                    request,
                    root: root_id,
                    root_index: index,
                    owner: root.store.clone(),
                    batch: Some(batch),
                    serial: None,
                    created: None,
                    attributes: Some((sheet_id, edit)),
                    sheet_sets: None,
                };
                self.admit_staged(&staged, cancel)?;
                self.staged = Some(staged);
                self.commit_staged()?;
                self.acknowledge(lease)?;
            }
            _ => {
                return Ok(DomStyleServiceProgress::Pending {
                    request,
                    reason: DomStyleUnavailable::Environment,
                });
            }
        }
        Ok(DomStyleServiceProgress::Serviced)
    }
}

/// Finite admission for each actual owner in the same durable composition.
#[derive(Clone, Copy, Debug)]
pub struct DomSessionLimits {
    /// Durable Source event jobs and actual adapter admission work.
    pub source_events: crate::dom_dialog::DomSourceEventRuntimeLimits,
    /// Style source/resource/task orchestration admission.
    pub style: DomStyleRuntimeLimits,
    /// Shared element-task owner admission.
    pub element_tasks: crate::element_tasks::DomElementTaskRuntimeLimits,
    /// Native hosted lifecycle and its canonical source admission.
    pub hosted: crate::dom_lifecycle::DomHostedRuntimeLimits,
}
/// The owner which refused construction, with the original DOM still returned.
#[derive(Debug)]
pub enum DomSessionCreateFailure {
    /// The original Source event executor or its fixed owner was not admitted.
    SourceEvents(crate::dom_dialog::DomSourceEventRuntimeError),
    /// Existing Style source/resource custody prevents adoption or admission.
    Style(DomStyleRuntimeError),
    /// Qualified original parsed admission was not completed.
    Parsed(DomStyleParsedAdmissionError),
    /// The actual shared element-task owner could not be admitted.
    ElementTasks(crate::element_tasks::DomElementTaskRuntimeError),
    /// The actual hosted source/domain owner could not be admitted.
    Hosted(crate::dom_lifecycle::DomHostedRuntimeError),
}
impl std::fmt::Display for DomSessionCreateFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceEvents(error) => std::fmt::Display::fmt(error, f),
            Self::Style(error) => std::fmt::Display::fmt(error, f),
            Self::Parsed(error) => std::fmt::Display::fmt(error, f),
            Self::ElementTasks(error) => std::fmt::Display::fmt(error, f),
            Self::Hosted(error) => std::fmt::Display::fmt(error, f),
        }
    }
}
impl std::error::Error for DomSessionCreateFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SourceEvents(error) => Some(error),
            Self::Style(error) => Some(error),
            Self::Parsed(error) => Some(error),
            Self::ElementTasks(error) => Some(error),
            Self::Hosted(error) => Some(error),
        }
    }
}
/// Durable ownership of the original canonical DOM and its composed obligations.
/// Moving this non-Clone value transfers the whole composition. Execution
/// permissions never own or release its resources.
pub struct DomSession {
    dom: Dom,
    state: std::rc::Rc<std::cell::RefCell<StyleState>>,
    element_tasks: crate::element_tasks::ElementTaskOwner,
    hosted: crate::dom_lifecycle::HostedOwner,
    events: crate::dom_dialog::SourceEventOwner,
}
impl std::fmt::Debug for DomSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomSession")
            .field("dom", &self.dom.owner())
            .finish_non_exhaustive()
    }
}
/// Rejected creation returns the actual input owner. Structure, identities and
/// effects stay unchanged; explicit parsed admission retains its original
/// cumulative processing work and bound policy for a genuine retry.
pub struct DomSessionCreateError {
    dom: Dom,
    error: DomSessionCreateFailure,
}
impl DomSessionCreateError {
    /// Actual rejection reason.
    pub fn error(&self) -> &DomSessionCreateFailure {
        &self.error
    }
    /// Return the rejected input; established sessions expose no such extraction.
    pub fn into_dom(self) -> Dom {
        self.dom
    }
}
impl std::fmt::Debug for DomSessionCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomSessionCreateError")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}
impl std::fmt::Display for DomSessionCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.error, f)
    }
}
impl std::error::Error for DomSessionCreateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
/// Exact unresolved Style obligation during graceful shutdown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomShutdownReason {
    /// The actual beforetoggle event remains owned by this session.
    SourceEvent(surgeist_dom::DomSourceEventId),
    /// An actual Details source intention has not reached its task owner.
    DetailsIntentions,
    /// An actual queued/running element task has not completed or been canceled.
    DomElementTask(crate::element_tasks::DomElementTaskId),
    /// Canonical hooks are not complete.
    SourceTail(DomSourceOperationId),
    /// Original installed parser association history has not been consumed.
    ParsedAssociations(surgeist_dom::DomParsedHtmlAssociationId),
    /// A canonical root still retains its actual mutable resource.
    LiveResources,
    /// An actual queued/running task remains.
    NetworkingTask(DomStyleNetworkingTaskId),
    /// Critical imported resources have no real outcome yet.
    CriticalResources,
    /// A native hosted object, operation, loop or task remains live.
    HostedLifecycle,
    /// An owner is currently borrowed.
    Busy,
    /// An actual resource is poisoned; no successful cleanup is guessed.
    Poisoned,
    /// Runtime cleanup observation could not be admitted; custody is unchanged.
    Admission,
}
/// Graceful completion or the whole same pending session, never partial disposal.
#[expect(
    clippy::large_enum_variant,
    reason = "Pending returns the exact whole session without allocating or adding a failure at mandatory custody return"
)]
#[derive(Debug)]
pub enum DomShutdownOutcome {
    /// All admitted source/resource/task/lifecycle obligations were actually settled.
    Complete,
    /// Ownership is retained for real servicing or whole-session transfer.
    Pending {
        /// The whole same composition, transferable and serviceable for retry.
        session: DomSession,
        /// Actual unresolved source, task, resource or admission boundary.
        reason: DomShutdownReason,
    },
}
impl DomSession {
    /// Adopt an unbound original DOM. Prebound/source-pending input is rejected
    /// intact because an empty task bank cannot reconstruct prior obligations.
    #[expect(
        clippy::result_large_err,
        reason = "Creation refusal returns the actual unchanged input Dom without another allocation or resource-loss failure"
    )]
    pub fn new(dom: Dom, limits: DomSessionLimits) -> Result<Self, DomSessionCreateError> {
        Self::create(dom, limits, None)
    }
    /// Adopt actual complete supported parsed contributions before readiness.
    /// No stylesheet operation runs during admission. Refusal returns the same
    /// DOM with structure/effects unchanged; admitted original contribution work
    /// and raise-only policy remain with it, rather than resetting on retry.
    #[expect(
        clippy::result_large_err,
        reason = "Intake refusal returns the actual original Dom and retained contribution custody without allocation"
    )]
    pub fn admit_parsed(
        dom: Dom,
        limits: DomSessionLimits,
        processing: surgeist_dom::DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<Self, DomSessionCreateError> {
        Self::create(dom, limits, Some((processing, cancel)))
    }
    #[expect(
        clippy::result_large_err,
        reason = "Session construction refusal returns the actual original Dom without allocation or custody loss"
    )]
    fn create(
        mut dom: Dom,
        limits: DomSessionLimits,
        parsed: Option<(surgeist_dom::DomParsedHtmlAssociationLimits, &AtomicBool)>,
    ) -> Result<Self, DomSessionCreateError> {
        if dom.has_element_task_source() {
            return Err(DomSessionCreateError {
                dom,
                error: DomSessionCreateFailure::ElementTasks(
                    crate::element_tasks::DomElementTaskRuntimeError::PretransferredSession,
                ),
            });
        }
        if dom.has_source_event_executor() {
            return Err(DomSessionCreateError {
                dom,
                error: DomSessionCreateFailure::SourceEvents(
                    crate::dom_dialog::DomSourceEventRuntimeError::PreboundSession,
                ),
            });
        }
        let pending_parsed = if parsed.is_none() {
            dom.pending_parsed_html_associations()
                .next()
                .map(|entry| entry.id())
        } else {
            None
        };
        let reject = if let Some(id) = pending_parsed {
            Some(DomStyleRuntimeError::PendingParsedAssociations(id))
        } else if dom.has_style_cssom_roots() {
            Some(DomStyleRuntimeError::PreboundSession)
        } else if let Some(status) = dom.pending_source_edit() {
            Some(DomStyleRuntimeError::SourcePending(status.operation()))
        } else if std::mem::size_of::<std::cell::RefCell<StyleState>>() > limits.style.storage_bytes
        {
            Some(DomStyleRuntimeError::Admission)
        } else {
            None
        };
        if let Some(error) = reject {
            return Err(DomSessionCreateError {
                dom,
                error: DomSessionCreateFailure::Style(error),
            });
        }
        if dom.has_transferred_details_toggle_intentions() {
            return Err(DomSessionCreateError {
                dom,
                error: DomSessionCreateFailure::ElementTasks(
                    crate::element_tasks::DomElementTaskRuntimeError::PretransferredSession,
                ),
            });
        }
        if let Err(error) = crate::dom_dialog::SourceEventOwner::admit_initial(limits.source_events)
        {
            return Err(DomSessionCreateError {
                dom,
                error: DomSessionCreateFailure::SourceEvents(error),
            });
        }
        if let Err(error) = crate::dom_lifecycle::HostedOwner::admit_initial_headers(limits.hosted)
        {
            return Err(DomSessionCreateError {
                dom,
                error: DomSessionCreateFailure::Hosted(error),
            });
        }
        let parsed_work = if let Some((processing, cancel)) = parsed {
            match meta::parsed::admit(&mut dom, limits.style, processing, cancel) {
                Ok(work) => work,
                Err(error) => {
                    return Err(DomSessionCreateError {
                        dom,
                        error: DomSessionCreateFailure::Parsed(error),
                    });
                }
            }
        } else {
            0
        };
        let owner = dom.owner();
        let details = match crate::element_tasks::ElementTaskOwner::new(owner, limits.element_tasks)
        {
            Ok(details) => details,
            Err(error) => {
                return Err(DomSessionCreateError {
                    dom,
                    error: DomSessionCreateFailure::ElementTasks(error),
                });
            }
        };
        let hosted = match crate::dom_lifecycle::HostedOwner::new(&mut dom, limits.hosted) {
            Ok(hosted) => hosted,
            Err(error) => {
                return Err(DomSessionCreateError {
                    dom,
                    error: DomSessionCreateFailure::Hosted(error),
                });
            }
        };
        details.bind_hosted(hosted.execution());
        hosted.bind_element_tasks(details.execution());
        details.bind_source(&mut dom);
        // All binding preconditions were checked before Hosted adoption. This
        // exclusive construction has executed no callbacks or Source edits.
        let executor = dom
            .bind_source_event_executor(&AtomicBool::new(false))
            .expect("exclusive preflighted Source event binding");
        let events =
            crate::dom_dialog::SourceEventOwner::new(owner, executor, limits.source_events);
        details.bind_events(events.execution());
        let state = StyleState::new(owner, limits.style);
        state.work.set(parsed_work);
        Ok(Self {
            dom,
            state: std::rc::Rc::new(std::cell::RefCell::new(state)),
            element_tasks: details,
            hosted,
            events,
        })
    }
    /// Non-owning execution permission for this exact original session.
    pub fn style_execution(&self) -> DomStyleRuntime {
        DomStyleRuntime {
            state: std::rc::Rc::downgrade(&self.state),
        }
    }
    /// Weak permission for this exact session's original element-task owner.
    pub fn element_task_execution(&self) -> crate::element_tasks::DomElementTaskRuntime {
        self.element_tasks.execution()
    }
    /// Weak permission for this exact session's original native lifecycle owner.
    pub fn hosted_execution(&self) -> crate::dom_lifecycle::HostedRuntime {
        self.hosted.execution()
    }
    /// Weak scope permission for one actual Host of this original Session.
    pub fn source_execution_scope(
        &self,
        host: surgeist_dom::DomHostedHostId,
        cancel: &AtomicBool,
    ) -> Result<crate::dom_lifecycle::DomSourceExecutionScope, crate::DomHostedRuntimeError> {
        self.hosted
            .execution()
            .execution_scope(&self.dom, host, cancel)
    }
    /// Permission to execute one source operation through the original family owners.
    pub fn source_execution(&self) -> crate::dom_source::DomSourceRuntime {
        crate::dom_source::DomSourceRuntime::new(
            self.style_execution(),
            self.element_task_execution(),
            self.hosted_execution(),
            self.events.execution(),
        )
    }
    /// Controlled canonical access. No bound mutable owner can be extracted.
    pub fn dom(&mut self) -> DomEventDom<'_> {
        DomEventDom::borrow(&mut self.dom)
    }
    /// An immutable historical capture which does not retain mutable custody.
    pub fn snapshot(&self) -> DomSnapshot {
        self.dom.snapshot()
    }
    /// Finish only when actual composed obligations are settled. Terminal ordinary
    /// Drop releases all session/DOM/task-owned custody without callbacks or
    /// completion; independent public root capabilities may retain their Store.
    pub fn try_shutdown(self) -> DomShutdownOutcome {
        let events = match self.events.pending() {
            Ok(Some(event)) => Some(DomShutdownReason::SourceEvent(event)),
            Ok(None) => None,
            Err(crate::dom_dialog::DomSourceEventRuntimeError::Busy) => {
                Some(DomShutdownReason::Busy)
            }
            Err(_) => Some(DomShutdownReason::Admission),
        };
        let hosted = match self.hosted.pending() {
            Ok(false) => None,
            Ok(true) => Some(DomShutdownReason::HostedLifecycle),
            Err(crate::dom_lifecycle::DomHostedRuntimeError::Busy) => Some(DomShutdownReason::Busy),
            Err(_) => Some(DomShutdownReason::Admission),
        };
        let details = match self.element_tasks.pending() {
            Ok(Some(task)) => Some(DomShutdownReason::DomElementTask(task)),
            Ok(None) => None,
            Err(crate::element_tasks::DomElementTaskRuntimeError::Busy) => {
                Some(DomShutdownReason::Busy)
            }
            Err(_) => Some(DomShutdownReason::Admission),
        };
        let parsed = self
            .dom
            .pending_parsed_html_associations()
            .next()
            .map(|entry| entry.id());
        let reason = if let Some(pending) = self.dom.pending_source_edit() {
            Some(DomShutdownReason::SourceTail(pending.operation()))
        } else if let Some(id) = parsed {
            Some(DomShutdownReason::ParsedAssociations(id))
        } else if events.is_some() {
            events
        } else if self.dom.pending_details_toggle_intentions() != 0 {
            Some(DomShutdownReason::DetailsIntentions)
        } else if details.is_some() {
            details
        } else if hosted.is_some() {
            hosted
        } else {
            match self.state.try_borrow_mut() {
                Err(_) => Some(DomShutdownReason::Busy),
                Ok(mut state) => {
                    if state
                        .prune_retired_roots(&self.dom.snapshot(), &AtomicBool::new(false))
                        .is_err()
                    {
                        Some(DomShutdownReason::Admission)
                    } else if let Some(task) =
                        state.completions.iter().find_map(|entry| match entry {
                            Completion::Waiting(_) => None,
                            Completion::Queued(task) => Some(task.id()),
                        })
                    {
                        Some(DomShutdownReason::NetworkingTask(task))
                    } else if state.waiting_resources() != 0 {
                        Some(DomShutdownReason::CriticalResources)
                    } else {
                        let mut access = None;
                        for root in &state.roots {
                            match root.store.snapshot() {
                                Err(DomStyleCssomRootError::Busy) => {
                                    access = Some(DomShutdownReason::Busy);
                                    break;
                                }
                                Err(DomStyleCssomRootError::Poisoned) => {
                                    access = Some(DomShutdownReason::Poisoned);
                                    break;
                                }
                                Err(_) => {
                                    access = Some(DomShutdownReason::LiveResources);
                                    break;
                                }
                                Ok(_) => {}
                            }
                        }
                        access.or_else(|| {
                            (!state.roots.is_empty()
                                || state.publication.is_some()
                                || state.staged.is_some()
                                || self.dom.has_style_cssom_roots())
                            .then_some(DomShutdownReason::LiveResources)
                        })
                    }
                }
            }
        };
        match reason {
            Some(reason) => DomShutdownOutcome::Pending {
                session: self,
                reason,
            },
            None => DomShutdownOutcome::Complete,
        }
    }
}

pub(crate) struct StyleSourceGuard {
    state: std::rc::Rc<std::cell::RefCell<StyleState>>,
}
impl StyleSourceGuard {
    pub(crate) fn reserve_cleanup(&self, cancel: &AtomicBool) -> Result<(), DomStyleRuntimeError> {
        DomStyleRuntime::reserve_root_cleanup(&self.state, cancel)
    }
    pub(crate) fn flush(
        &self,
        lease: &mut DomSourceLease<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleRuntimeError> {
        self.state
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?
            .flush_source(lease, cancel)
    }
    pub(crate) fn service(
        &self,
        lease: &mut DomSourceLease<'_>,
        request: DomStyleRequest,
        cancel: &AtomicBool,
    ) -> Result<DomStyleServiceProgress, DomStyleRuntimeError> {
        self.state
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?
            .service_source_request(lease, request, cancel)
    }
    pub(crate) fn finish(self, snapshot: &DomSnapshot) {
        DomStyleRuntime::release_retired_roots(&self.state, snapshot);
    }
}

/// One actual Style composition owner. Reentry retains this exact private State;
/// replacing a context handle never redirects an executing task to another bank.
pub struct DomStyleRuntime {
    state: std::rc::Weak<std::cell::RefCell<StyleState>>,
}
impl std::fmt::Debug for DomStyleRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomStyleRuntime").finish_non_exhaustive()
    }
}
impl DomStyleRuntime {
    fn original(
        &self,
    ) -> Result<std::rc::Rc<std::cell::RefCell<StyleState>>, DomStyleRuntimeError> {
        self.state
            .upgrade()
            .ok_or(DomStyleRuntimeError::SessionUnavailable)
    }
    /// Consume an actual initially empty root capability only after validation
    /// and admission. Refusal leaves the caller's resource envelope untouched.
    pub fn bind_root(
        &mut self,
        dom: &DomEventDom<'_>,
        label: String,
        input: &mut Option<DomStyleCssomRoot>,
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleRuntimeError> {
        let state = self.original()?;
        let mut state = state
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        state.bind_root(&dom.snapshot(), label, input, cancel)
    }
    /// Historical CSSOM capture, without a current DOM-association certificate.
    pub fn snapshot(&self, root: NodeId) -> Result<Option<CssomSnapshot>, DomStyleRuntimeError> {
        let state = self.original()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        state.snapshot(root)
    }
    /// Adjust actual finite admission policy without resetting cumulative work
    /// or replacing any canonical/resource/task owner. Refusal keeps old limits.
    pub fn set_limits(
        &mut self,
        dom: &DomEventDom<'_>,
        limits: DomStyleRuntimeLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleRuntimeError> {
        let original = self.qualified(dom)?;
        let mut state = original
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        state.step_with_work(cancel, limits.work)?;
        if limits.completions > state.limits.completions {
            for entry in &state.completions {
                state.step_with_work(cancel, limits.work)?;
                if entry.task().running {
                    return Err(DomStyleRuntimeError::Admission);
                }
            }
        }
        if state.roots.len() > limits.roots
            || state.completions.len() > limits.completions
            || state.reports.len() > limits.reports
        {
            return Err(DomStyleRuntimeError::Admission);
        }
        let bytes = state.storage(None, cancel, limits.work)?;
        if bytes > limits.storage_bytes {
            return Err(DomStyleRuntimeError::Admission);
        }
        if cancel.load(Ordering::Relaxed) {
            return Err(DomError::Cancelled.into());
        }
        state.limits = limits;
        Ok(())
    }
    /// Read the current object through the canonical association and retained
    /// original Root. Unrelated sheet edits do not invalidate this observation.
    pub fn disabled(
        &self,
        dom: &DomEventDom<'_>,
        element: NodeId,
    ) -> Result<DomStyleDisabledObservation, DomStyleRuntimeError> {
        let original = self.qualified(dom)?;
        let snapshot = dom.snapshot();
        let node = snapshot.node(element)?;
        let style = node.as_style().ok_or(DomStyleRuntimeError::Binding)?;
        let Some(association) = style.sheet()? else {
            return Ok(DomStyleDisabledObservation {
                stamp: snapshot.stamp(),
                element,
                association: None,
                revision: None,
                disabled: false,
            });
        };
        let state = original
            .try_borrow()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        let root = state
            .roots
            .iter()
            .find(|root| root.node == association.root())
            .ok_or(DomStyleRuntimeError::Binding)?;
        let current = root.store.snapshot()?;
        let sheet = current.sheet(association.id())?;
        Ok(DomStyleDisabledObservation {
            stamp: snapshot.stamp(),
            element,
            association: Some(association.association()),
            revision: Some(current.revision().clone()),
            disabled: sheet.inputs().disabled,
        })
    }
    /// One immutable actual processing result, independent of execution permission.
    pub fn processed_sheet(
        &self,
        index: usize,
    ) -> Result<Option<Arc<DomStyleProcessedSheet>>, DomStyleRuntimeError> {
        let state = self.original()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        Ok(state.reports.get(index).cloned())
    }
    /// Genuine imported-resource waits.
    pub fn waiting_resources(&self) -> Result<usize, DomStyleRuntimeError> {
        let state = self.original()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        Ok(state.waiting_resources())
    }
    /// Exact historical root and sheet of an actual task.
    pub fn pending_sheet(
        &self,
        index: usize,
    ) -> Result<Option<(NodeId, CssomSheetId)>, DomStyleRuntimeError> {
        let state = self.original()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        Ok(state.completions.get(index).map(|entry| {
            let task = entry.task();
            (task.root, task.sheet.clone())
        }))
    }
    fn qualified(
        &self,
        dom: &DomEventDom<'_>,
    ) -> Result<std::rc::Rc<std::cell::RefCell<StyleState>>, DomStyleRuntimeError> {
        let state = self.original()?;
        if state
            .try_borrow()
            .map_err(|_| DomStyleRuntimeError::Binding)?
            .dom
            != dom.owner()
        {
            return Err(DomStyleRuntimeError::WrongSession);
        }
        Ok(state)
    }
    fn reserve_root_cleanup(
        state: &std::rc::Rc<std::cell::RefCell<StyleState>>,
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleRuntimeError> {
        let state = state
            .try_borrow()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        // Only bind_root pushes rows, and it requires this exact DOM's scoped
        // view. The source lease uniquely borrows that view throughout drive;
        // policy code cannot obtain another same-owner view. Internal drive
        // changes only root serials, so this length cannot grow before pruning.
        for _ in &state.roots {
            state.step(cancel)?;
        }
        Ok(())
    }
    fn release_retired_roots(
        state: &std::rc::Rc<std::cell::RefCell<StyleState>>,
        snapshot: &DomSnapshot,
    ) {
        // The scan was admitted before source work. Complete has ended its real
        // lease; no user work, allocation, cancellation or failure follows it.
        // Pending tasks independently retain the original capability they need.
        state
            .borrow_mut()
            .roots
            .retain(|root| !matches!(snapshot.node(root.node), Err(DomError::StaleNode)));
    }
    pub(crate) fn qualify_source(
        &self,
        dom: &DomEventDom<'_>,
    ) -> Result<StyleSourceGuard, DomStyleRuntimeError> {
        Ok(StyleSourceGuard {
            state: self.qualified(dom)?,
        })
    }
    /// Execute the actual networking FIFO head. The actual task never leaves
    /// its session bank; all owner borrows end before entering caller work.
    pub fn run_networking<C: DomStyleEventContext>(
        dom: &mut DomEventDom<'_>,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        limits: DomEventDispatchLimits,
        cancel: &AtomicBool,
    ) -> Result<DomStyleNetworkingProgress, DomStyleRuntimeError> {
        let original = context.styles().qualified(dom)?;
        let id = {
            let mut state = original.borrow_mut();
            state.step(cancel)?;
            let mut queued = None;
            for (index, entry) in state.completions.iter().enumerate() {
                state.step(cancel)?;
                if matches!(entry, Completion::Queued(_)) {
                    queued = Some(index);
                    break;
                }
            }
            let Some(index) = queued else {
                return Ok(DomStyleNetworkingProgress::Empty);
            };
            let task = state.completions[index].task();
            let id = task.id();
            if task.running {
                return Ok(DomStyleNetworkingProgress::Running(id));
            }
            // Reserve bounded exact-identity searches and marker cleanup before
            // callbacks. The native cursor takes at most thirteen full bank
            // searches; sixteen reserves its error/guard exits too. Reentry may
            // fill only the declared maximum; set_limits cannot raise that bound
            // while a task is running.
            let cleanup = u64::try_from(state.limits.completions)
                .ok()
                .and_then(|count| count.checked_mul(16))
                .ok_or(DomStyleRuntimeError::Admission)?;
            let next = state
                .work
                .get()
                .checked_add(cleanup)
                .ok_or(DomStyleRuntimeError::Admission)?;
            if next > state.limits.work {
                return Err(DomStyleRuntimeError::Admission);
            }
            state.work.set(next);
            state.admit(0, cancel)?;
            state.completions[index].task_mut().running = true;
            id
        };
        let _running = StyleTaskRunning {
            state: original.clone(),
            id,
        };
        loop {
            let action = {
                let mut state = original.borrow_mut();
                let index = state.task_index(id)?;
                let task = state.completions[index].task();
                let started = match &task.delivery {
                    Delivery::NotStarted => false,
                    Delivery::Dispatching(event) => event.has_started(),
                    Delivery::Delivered(_) => true,
                };
                if !started {
                    match dom.snapshot().node(task.token.target()) {
                        Err(DomError::StaleNode) => {
                            let target = task.token.target();
                            state.completions.remove(index);
                            return Ok(DomStyleNetworkingProgress::TargetDestroyed {
                                task: id,
                                target,
                            });
                        }
                        Err(error) => return Err(error.into()),
                        Ok(_) => {}
                    }
                }
                task.cursor.action()
            };
            match action {
                HtmlStyleNotificationAction::DispatchLoad
                | HtmlStyleNotificationAction::DispatchError => {
                    let event = {
                        let mut state = original.borrow_mut();
                        let index = state.task_index(id)?;
                        if matches!(
                            state.completions[index].task().delivery,
                            Delivery::NotStarted
                        ) {
                            state.admit(
                                NativeDomEventOwner::initial_payload_bytes()
                                    .ok_or(DomStyleRuntimeError::Admission)?,
                                cancel,
                            )?;
                            let target = state.completions[index].task().token.target();
                            let event = if action == HtmlStyleNotificationAction::DispatchLoad {
                                NativeDomEvent::style_load(target)
                            } else {
                                NativeDomEvent::style_error(target)
                            };
                            state.completions[index].task_mut().delivery =
                                Delivery::Dispatching(event);
                        }
                        match &state.completions[index].task().delivery {
                            Delivery::Dispatching(event) => Some((**event).clone()),
                            Delivery::Delivered(_) => None,
                            Delivery::NotStarted => unreachable!(),
                        }
                    };
                    if let Some(event) = event {
                        {
                            let mut state = original.borrow_mut();
                            let index = state.task_index(id)?;
                            let previous = state.completions[index].task().dispatch_reserved;
                            let Delivery::Dispatching(owner) =
                                &state.completions[index].task().delivery
                            else {
                                unreachable!("actual retained event")
                            };
                            let formation = owner
                                .dispatch_formation(limits)
                                .ok_or(DomStyleRuntimeError::Admission)?;
                            state.completions[index].task_mut().dispatch_reserved = formation.full;
                            if let Err(error) = state.admit(0, cancel) {
                                state.completions[index].task_mut().dispatch_reserved = previous;
                                return Err(error);
                            }
                        }
                        let outcome = if event.has_started() {
                            dispatcher.resume(dom.raw_mut(), context, &event, limits, cancel)
                        } else {
                            dispatcher.dispatch(dom.raw_mut(), context, &event, limits, cancel)
                        }
                        .map_err(DomStyleRuntimeError::Dispatch)?;
                        match outcome {
                            DomEventDispatchOutcome::Complete(receipt) => {
                                let mut state = original.borrow_mut();
                                let index = state.task_index(id)?;
                                state.completions[index].task_mut().delivery =
                                    Delivery::Delivered(receipt);
                            }
                            dispatch => {
                                if !std::rc::Weak::ptr_eq(
                                    &std::rc::Rc::downgrade(&original),
                                    &context.styles().state,
                                ) {
                                    return Ok(retained_access(&original, id));
                                }
                                return Ok(DomStyleNetworkingProgress::Retained {
                                    task: id,
                                    dispatch,
                                });
                            }
                        }
                    }
                    // Delivered is durable before this fallible source cursor
                    // transition; a later refusal can never replay listeners.
                    {
                        let mut state = original.borrow_mut();
                        let index = state.task_index(id)?;
                        state.completions[index]
                            .task_mut()
                            .cursor
                            .reply(HtmlStyleNotificationReply::Applied)
                            .map_err(|_| DomStyleRuntimeError::Protocol)?;
                    }
                    if !std::rc::Weak::ptr_eq(
                        &std::rc::Rc::downgrade(&original),
                        &context.styles().state,
                    ) {
                        return Ok(retained_access(&original, id));
                    }
                }
                HtmlStyleNotificationAction::ReadScriptBlocking => {
                    if !std::rc::Weak::ptr_eq(
                        &std::rc::Rc::downgrade(&original),
                        &context.styles().state,
                    ) {
                        return Ok(retained_access(&original, id));
                    }
                    let mut state = original.borrow_mut();
                    let index = state.task_index(id)?;
                    let task = state.completions[index].task_mut();
                    if let Ok(node) = dom.snapshot().node(task.token.target())
                        && node.as_style().is_none()
                    {
                        return Err(DomStyleRuntimeError::Binding);
                    }
                    task.cursor
                        .reply(HtmlStyleNotificationReply::ScriptBlocking(false))
                        .map_err(|_| DomStyleRuntimeError::Protocol)?;
                }
                HtmlStyleNotificationAction::RemoveScriptBlocking => {
                    return Err(DomStyleRuntimeError::Protocol);
                }
                HtmlStyleNotificationAction::UnblockRendering => {
                    let mut state = original.borrow_mut();
                    let index = state.task_index(id)?;
                    let task = state.completions[index].task_mut();
                    match dom
                        .raw_mut()
                        .finish_style_notification(&mut task.token, cancel)?
                    {
                        DomStyleNotificationCompletion::Complete(_)
                        | DomStyleNotificationCompletion::TargetDestroyed => {}
                    }
                    task.cursor
                        .reply(HtmlStyleNotificationReply::Applied)
                        .map_err(|_| DomStyleRuntimeError::Protocol)?;
                }
                HtmlStyleNotificationAction::Complete => {
                    let mut state = original.borrow_mut();
                    let index = state.task_index(id)?;
                    if !matches!(
                        state.completions[index].task().delivery,
                        Delivery::Delivered(_)
                    ) {
                        return Err(DomStyleRuntimeError::Protocol);
                    }
                    let task = match state.completions.remove(index) {
                        Completion::Queued(task) => task,
                        Completion::Waiting(_) => unreachable!(),
                    };
                    let Delivery::Delivered(receipt) = task.delivery else {
                        unreachable!()
                    };
                    return Ok(DomStyleNetworkingProgress::Complete { task: id, receipt });
                }
            }
        }
    }
}
impl StyleState {
    fn task_index(&self, id: DomStyleNetworkingTaskId) -> Result<usize, DomStyleRuntimeError> {
        self.completions
            .iter()
            .position(|entry| entry.task().id() == id)
            .ok_or(DomStyleRuntimeError::Protocol)
    }
}
fn retained_access(
    original: &std::rc::Rc<std::cell::RefCell<StyleState>>,
    task: DomStyleNetworkingTaskId,
) -> DomStyleNetworkingProgress {
    DomStyleNetworkingProgress::RetainedAccess {
        task,
        owner: DomStyleRuntime {
            state: std::rc::Rc::downgrade(original),
        },
    }
}
struct StyleTaskRunning {
    state: std::rc::Rc<std::cell::RefCell<StyleState>>,
    id: DomStyleNetworkingTaskId,
}
impl Drop for StyleTaskRunning {
    fn drop(&mut self) {
        // Every inner private borrow ends before caller code or unwinds before
        // this guard. No Ref/RefMut escapes this module, so this cannot collide
        // with a caller-held borrow; cleanup never invokes caller code.
        let mut state = self.state.borrow_mut();
        if let Some(entry) = state
            .completions
            .iter_mut()
            .find(|entry| entry.task().id() == self.id)
        {
            entry.task_mut().running = false;
            entry.task_mut().dispatch_reserved = 0;
        }
    }
}

#[cfg(test)]
use crate::{
    DomSourceDriveProgress, DomSourceRuntime, DomSourceRuntimeError, DomSourceServices,
    DomSourceUnavailable,
};
#[cfg(test)]
use surgeist_dom::DomStyleInlineCspObservation;
#[cfg(test)]
mod tests {
    mod meta;
    mod meta_source;
    mod parsed_associations;
    mod preferred_sets;
    pub(super) fn session_limits(style: super::DomStyleRuntimeLimits) -> super::DomSessionLimits {
        super::DomSessionLimits {
            source_events: crate::dom_dialog::DomSourceEventRuntimeLimits {
                jobs: 0,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style,
            element_tasks: crate::element_tasks::DomElementTaskRuntimeLimits {
                tasks: 32,
                storage_bytes: 16 * 1024 * 1024,
                work: 1_000_000,
            },
            hosted: crate::dom_lifecycle::DomHostedRuntimeLimits {
                canonical: surgeist_dom::DomHostedLimits::new(
                    surgeist_dom::DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                    16 * 1024 * 1024,
                    16 * 1024 * 1024,
                    1_000_000,
                ),
                tasks: 32,
                storage_bytes: 16 * 1024 * 1024,
                scratch_bytes: 16 * 1024 * 1024,
                work: 1_000_000,
            },
        }
    }

    use super::*;
    use crate::{DomEventCallback, DomEventListenerOptions, DomEventRegistryLimits, DomEventType};
    use surgeist_cssom::CssomStore;
    use surgeist_dom::{
        DomBatch, DomLimits, DomSourceEdit, DomSourceLimits, DomSourceProgress, Namespace,
    };
    struct Context {
        styles: DomStyleRuntime,
        source: DomSourceRuntime,
        events: Vec<(NodeId, NodeId)>,
    }
    impl DomStyleEventContext for Context {
        fn styles(&mut self) -> &mut DomStyleRuntime {
            &mut self.styles
        }
    }
    fn limits() -> DomStyleRuntimeLimits {
        DomStyleRuntimeLimits {
            roots: 8,
            completions: 32,
            reports: 32,
            storage_bytes: 1_000_000,
            work: 100_000,
            css_input: CssUtf16InputLimits::default(),
        }
    }
    fn source_limits() -> DomSourceLimits {
        DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1)
    }
    fn dispatch_limits() -> DomEventDispatchLimits {
        DomEventDispatchLimits {
            path_items: 128,
            snapshot_listeners: 128,
            failure_records: 128,
            callbacks: 128,
            work: 100_000,
        }
    }
    fn dom() -> Dom {
        Dom::new(
            DomLimits::new(
                256,
                100_000,
                1024,
                0,
                0,
                32,
                1_000_000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn dispatch_dom() -> Dom {
        Dom::new(
            DomLimits::new(
                256,
                100_000,
                1024,
                0,
                0,
                32,
                1_000_000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_event_recipients(
                surgeist_dom::DomEventRecipientLimits::new(64, 16_000_000, 16_000_000, 4_000_000)
                    .unwrap(),
            )
            .with_dispatch_custody(
                surgeist_dom::DomDispatchCustodyLimits::new(
                    64, 4096, 16_000_000, 16_000_000, 4_000_000,
                )
                .unwrap(),
            ),
        )
        .unwrap()
    }
    fn policy(source: &DomStyleSource) -> Option<DomStyleInlineCspObservation> {
        Some(DomStyleInlineCspObservation::complete_empty(source, 1))
    }
    #[test]
    fn actual_root_tree_order_and_two_same_parent_text_phases_preserve_distinct_sheets() {
        let cancel = AtomicBool::new(false);
        let mut dom = dom();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let container = batch.create_element_ns(&document, Namespace::html(), "div");
        let host = batch.create_element_ns(&document, Namespace::html(), "div");
        let shadow = batch.attach_shadow(
            &host,
            surgeist_dom::DomShadowRootInit::new(surgeist_html::HtmlShadowRootMode::Open),
        );
        batch.insert_before(&container, &document, None);
        batch.insert_before(&host, &container, None);
        let commit = dom.apply(batch, &cancel).unwrap();
        let document = commit.created(&document).unwrap().unwrap();
        let container = commit.created(&container).unwrap().unwrap();
        let shadow = commit.created(&shadow).unwrap().unwrap();
        let mut session = DomSession::new(dom, session_limits(limits())).unwrap();
        let mut styles = session.style_execution();
        let mut source = session.source_execution();
        let mut dom = session.dom();
        for (root, label) in [(document, "document"), (shadow, "shadow")] {
            styles
                .bind_root(
                    &dom,
                    label.into(),
                    &mut Some(DomStyleCssomRoot::new(
                        root,
                        CssomStore::new(Default::default(), Default::default()).unwrap(),
                    )),
                    &cancel,
                )
                .unwrap();
        }
        let mut batch = DomBatch::new();
        let later = batch.create_element_ns(document, Namespace::html(), "style");
        let earlier = batch.create_element_ns(document, Namespace::html(), "style");
        let isolated = batch.create_element_ns(document, Namespace::html(), "style");
        let later_text = batch.create_text(document, "b { color: blue }");
        let earlier_text = batch.create_text(document, "a { color: red }");
        let isolated_text = batch.create_text(document, "s { color: green }");
        batch.insert_before(&later_text, &later, None);
        batch.insert_before(&earlier_text, &earlier, None);
        batch.insert_before(&isolated_text, &isolated, None);
        let setup = dom.apply(batch, &cancel).unwrap();
        let later = setup.created(&later).unwrap().unwrap();
        let earlier = setup.created(&earlier).unwrap().unwrap();
        let isolated = setup.created(&isolated).unwrap().unwrap();
        let earlier_text = setup.created(&earlier_text).unwrap().unwrap();
        assert!(matches!(
            source
                .apply(
                    &mut dom,
                    DomSourceEdit::insert_before(later, container, None),
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        assert!(matches!(
            source
                .apply(
                    &mut dom,
                    DomSourceEdit::insert_before(earlier, container, Some(later)),
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        assert!(matches!(
            source
                .apply(
                    &mut dom,
                    DomSourceEdit::insert_before(isolated, shadow, None),
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));

        let snapshot = dom.snapshot();
        let sheet = |node| {
            snapshot
                .node(node)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet()
                .unwrap()
                .unwrap()
                .id()
                .clone()
        };
        let earlier_id = sheet(earlier);
        let later_id = sheet(later);
        let isolated_id = sheet(isolated);
        assert_eq!(
            styles.snapshot(document).unwrap().unwrap().sheets(),
            &[earlier_id.clone(), later_id.clone()]
        );
        assert_eq!(
            styles.snapshot(shadow).unwrap().unwrap().sheets(),
            std::slice::from_ref(&isolated_id)
        );
        let edit = DomSourceEdit::insert_before(later, container, Some(earlier));
        let progress = source
            .apply(
                &mut dom,
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));

        let moved = styles.snapshot(document).unwrap().unwrap();
        let moved_id = dom
            .snapshot()
            .node(later)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap()
            .id()
            .clone();
        assert_ne!(moved_id, later_id);
        assert!(moved.sheet(&later_id).unwrap().inputs().owner.is_none());
        assert_eq!(moved.sheets(), &[moved_id.clone(), earlier_id.clone()]);
        let edit = DomSourceEdit::insert_before(earlier_text, earlier, None);
        let progress = source
            .apply(
                &mut dom,
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));

        let during = styles.processed_sheet(4).unwrap().unwrap();
        let final_report = styles.processed_sheet(5).unwrap().unwrap();
        assert_ne!(during.sheet(), final_report.sheet());
        let current = styles.snapshot(document).unwrap().unwrap();
        assert!(
            current
                .sheet(during.sheet())
                .unwrap()
                .inputs()
                .owner
                .is_none()
        );
        assert!(current.sheet(&earlier_id).unwrap().inputs().owner.is_none());
        assert_eq!(current.sheets(), &[moved_id, final_report.sheet().clone()]);
        assert!(
            current
                .sheet(during.sheet())
                .unwrap()
                .authored()
                .rules()
                .is_empty()
        );
        assert_eq!(
            current
                .sheet(final_report.sheet())
                .unwrap()
                .authored()
                .rules()
                .len(),
            1
        );
        assert_eq!(
            styles.snapshot(shadow).unwrap().unwrap().sheets(),
            &[isolated_id]
        );
        assert!(
            styles.pending_sheet(5).unwrap().is_some(),
            "both distinct source-time completions remain queued"
        );
        assert!(styles.pending_sheet(6).unwrap().is_none());
    }
    #[test]
    fn media_and_exact_utf16_title_edit_the_actual_sheet_without_recreation_or_extra_load() {
        let cancel = AtomicBool::new(false);
        let mut dom = dom();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let commit = dom.apply(batch, &cancel).unwrap();
        let document = commit.created(&document).unwrap().unwrap();
        let mut session = DomSession::new(dom, session_limits(limits())).unwrap();
        let mut styles = session.style_execution();
        let mut source = session.source_execution();
        let mut dom = session.dom();
        styles
            .bind_root(
                &dom,
                "root".into(),
                &mut Some(DomStyleCssomRoot::new(
                    document,
                    CssomStore::new(Default::default(), Default::default()).unwrap(),
                )),
                &cancel,
            )
            .unwrap();
        let mut batch = DomBatch::new();
        let style = batch.create_element_ns(document, Namespace::html(), "style");
        let text = batch.create_text(document, "a { color: red }");
        batch.insert_before(&text, &style, None);
        let setup = dom.apply(batch, &cancel).unwrap();
        let style = setup.created(&style).unwrap().unwrap();
        let edit = DomSourceEdit::insert_before(style, document, None);
        let progress = source
            .apply(
                &mut dom,
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        let DomSourceDriveProgress::Complete(_) = progress else {
            panic!("initial sheet");
        };

        let before = styles.snapshot(document).unwrap().unwrap();
        let id = before.sheets()[0].clone();
        let original_dom = dom.snapshot();
        let original_association = original_dom
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap()
            .association();
        let title = surgeist_dom::DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
        let mut batch = DomBatch::new();
        batch.set_attribute(style, "title", title.clone());
        batch.set_attribute(style, "media", "screen and (min-width: 2px)");
        let atomic_before = dom.snapshot();
        assert_eq!(
            dom.apply(batch, &cancel).unwrap_err(),
            DomError::StyleOwnerAttributesCompositionRequired
        );
        assert_eq!(dom.snapshot().stamp(), atomic_before.stamp());
        assert!(matches!(
            source
                .apply(
                    &mut dom,
                    DomSourceEdit::set_attribute(style, "title", title.clone()),
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let edit = DomSourceEdit::set_attribute(style, "media", "screen and (min-width: 2px)");
        let progress = source
            .apply(
                &mut dom,
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));

        let after = styles.snapshot(document).unwrap().unwrap();
        assert_eq!(after.sheets(), std::slice::from_ref(&id));
        let sheet = after.sheet(&id).unwrap();
        assert_eq!(sheet.inputs().title.as_slice(), title.code_units());
        assert_eq!(after.media(sheet.media()).unwrap().queries().len(), 1);
        assert!(before.sheet(&id).unwrap().inputs().title.is_empty());
        assert_eq!(
            dom.snapshot()
                .node(style)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet()
                .unwrap()
                .unwrap()
                .association(),
            original_association
        );
        assert_eq!(styles.pending_sheet(0).unwrap().unwrap().1, id);
        assert!(
            styles.pending_sheet(1).unwrap().is_none(),
            "attribute updates do not queue another load"
        );
        assert!(
            styles.processed_sheet(2).unwrap().is_some(),
            "both actual media report outcomes retained"
        );
    }
    #[test]
    fn actual_parsed_sheet_networking_capture_event_and_unblock_survive_nested_removal() {
        let cancel = AtomicBool::new(false);
        let mut dom = dispatch_dom();
        let mut b = DomBatch::new();
        let document = b.create_document();
        let c = dom.apply(b, &cancel).unwrap();
        let document = c.created(&document).unwrap().unwrap();
        let mut session = DomSession::new(dom, session_limits(limits())).unwrap();
        let permission = session.style_execution();
        let source = session.source_execution();
        let mut dom = session.dom();
        let mut context = Context {
            styles: permission,
            source,
            events: Vec::new(),
        };
        context
            .styles
            .bind_root(
                &dom,
                "actual-root".into(),
                &mut Some(DomStyleCssomRoot::new(
                    document,
                    CssomStore::new(Default::default(), Default::default()).unwrap(),
                )),
                &cancel,
            )
            .unwrap();
        let mut b = DomBatch::new();
        let style = b.create_element_ns(document, Namespace::html(), "style");
        let text = b.create_text(document, "@unknown x; a { color: red }");
        b.insert_before(&text, &style, None);
        let setup = dom.apply(b, &cancel).unwrap();
        let style = setup.created(&style).unwrap().unwrap();
        let edit = DomSourceEdit::insert_before(style, document, None);
        let progress = context
            .source
            .apply(
                &mut dom,
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        let DomSourceDriveProgress::Complete(_) = progress else {
            panic!("real no-import processing completes")
        };

        let snapshot = context.styles.snapshot(document).unwrap().unwrap();
        let sheet = snapshot.sheets()[0].clone();
        assert_eq!(snapshot.sheet(&sheet).unwrap().diagnostics().len(), 1);
        assert!(
            snapshot
                .sheet(&sheet)
                .unwrap()
                .authored()
                .parse_source()
                .unwrap()
                .source()
                .original_utf16()
                .is_some()
        );
        let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 16,
            pending_events: 16,
            nested_dispatches: 8,
        })
        .unwrap();
        dispatcher
            .add_listener(
                &mut dom,
                document,
                DomEventType::Load,
                DomEventCallback::new(move |dom, _dispatcher, context: &mut Context, event| {
                    context.events.push((
                        event.target().unwrap(),
                        event.current_target().unwrap().node().unwrap(),
                    ));
                    let edit = DomSourceEdit::detach(style);
                    let progress = context
                        .source
                        .apply(
                            dom,
                            edit,
                            source_limits(),
                            &mut DomSourceServices {
                                meta: None,
                                execution_scope: None,
                                style_policy: &mut policy,
                                url: None,
                            },
                            &cancel,
                        )
                        .unwrap();
                    assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: true,
                    once: false,
                },
            )
            .unwrap();
        let result = DomStyleRuntime::run_networking(
            &mut dom,
            &mut dispatcher,
            &mut context,
            dispatch_limits(),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(matches!(
            result,
            DomStyleNetworkingProgress::Complete { .. }
        ));
        assert_eq!(context.events, [(style, document)]);
        assert!(!dom.snapshot().node(style).unwrap().is_connected());
        let current = context.styles.snapshot(document).unwrap().unwrap();
        assert!(current.sheets().is_empty());
        assert!(current.sheet(&sheet).unwrap().inputs().owner.is_none());
        assert!(matches!(
            DomStyleRuntime::run_networking(
                &mut dom,
                &mut dispatcher,
                &mut context,
                dispatch_limits(),
                &AtomicBool::new(false)
            )
            .unwrap(),
            DomStyleNetworkingProgress::Empty
        ));
    }
    fn bound_session() -> (DomSession, NodeId) {
        bound_session_with_context(CssomContext::default())
    }
    fn bound_session_with_context(context: CssomContext) -> (DomSession, NodeId) {
        bound_session_from(dom(), context)
    }
    fn bound_dispatch_session() -> (DomSession, NodeId) {
        bound_session_from(dispatch_dom(), CssomContext::default())
    }
    fn bound_session_from(mut owner: Dom, context: CssomContext) -> (DomSession, NodeId) {
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let container = batch.create_element_ns(&document, Namespace::html(), "div");
        batch.insert_before(&container, &document, None);
        let commit = owner.apply(batch, &AtomicBool::new(false)).unwrap();
        let document = commit.created(&document).unwrap().unwrap();
        let mut session = DomSession::new(owner, session_limits(limits())).unwrap();
        session
            .style_execution()
            .bind_root(
                &session.dom(),
                "current-root".into(),
                &mut Some(DomStyleCssomRoot::new(
                    document,
                    CssomStore::new(Default::default(), context).unwrap(),
                )),
                &AtomicBool::new(false),
            )
            .unwrap();
        (session, document)
    }
    fn append_style(session: &mut DomSession, document: NodeId, media: Option<&str>) -> NodeId {
        let mut source = session.source_execution();
        let mut batch = DomBatch::new();
        let style = batch.create_element_ns(document, Namespace::html(), "style");
        if let Some(media) = media {
            batch.set_attribute(&style, "media", media);
        }
        let setup = session.dom().apply(batch, &AtomicBool::new(false)).unwrap();
        let style = setup.created(&style).unwrap().unwrap();
        let container = session.snapshot().node(document).unwrap().children()[0];
        assert!(matches!(
            source
                .apply(
                    &mut session.dom(),
                    DomSourceEdit::insert_before(style, container, None),
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &AtomicBool::new(false)
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        style
    }
    #[test]
    fn current_disabled_join_survives_other_sheet_revisions_and_external_flags_without_certifying_readiness()
     {
        let cancel = AtomicBool::new(false);
        let (mut session, document) = bound_session();
        let a = append_style(&mut session, document, None);
        let permission = session.style_execution();
        let original = session.snapshot();
        let receipt = original
            .node(a)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap();
        let association = receipt.association();
        let sheet = receipt.id().clone();
        let historical = receipt.revision().clone();
        let b = append_style(&mut session, document, None);
        let current = permission.disabled(&session.dom(), a).unwrap();
        assert!(!current.disabled());
        assert_eq!(current.association(), Some(association));
        assert_ne!(current.cssom_revision(), Some(&historical));
        let root = session.dom().style_cssom_root(document).unwrap().unwrap();
        let snapshot = root.snapshot().unwrap();
        let mut batch = root
            .batch(CssomEditGuard::from_snapshot(&snapshot))
            .unwrap();
        batch.set_disabled(&sheet, true).unwrap();
        root.commit(&mut Some(batch)).unwrap();
        assert!(permission.disabled(&session.dom(), a).unwrap().disabled());
        assert!(!permission.disabled(&session.dom(), b).unwrap().disabled());
        let edit = DomSourceEdit::set_style_disabled(a, false);
        let permission = session.style_execution();
        let mut source = session.source_execution();
        assert!(matches!(
            source
                .apply(
                    &mut session.dom(),
                    edit,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        assert!(!permission.disabled(&session.dom(), a).unwrap().disabled());
        assert_eq!(
            permission
                .disabled(&session.dom(), a)
                .unwrap()
                .association(),
            Some(association)
        );
        assert!(
            permission.pending_sheet(2).unwrap().is_none(),
            "disabled changes create no extra task"
        );
        let edit = DomSourceEdit::detach(a);
        source
            .apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        let detached = permission.disabled(&session.dom(), a).unwrap();
        assert!(!detached.disabled());
        assert_eq!(detached.association(), None);
        assert_eq!(detached.cssom_revision(), None);
        let edit = DomSourceEdit::destroy(a);
        source
            .apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        assert!(matches!(
            permission.disabled(&session.dom(), a),
            Err(DomStyleRuntimeError::Dom(DomError::StaleNode))
        ));
        let other = dom();
        let foreign = other.owner();
        assert_ne!(foreign, session.snapshot().owner());
        let mut other = other;
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let commit = other.apply(batch, &cancel).unwrap();
        let foreign = commit.created(&document).unwrap().unwrap();
        assert!(matches!(
            permission.disabled(&session.dom(), foreign),
            Err(DomStyleRuntimeError::Dom(DomError::ForeignOwner))
        ));
        // The original immutable association receipt remains historical evidence.
        assert_eq!(receipt.revision(), &historical);
    }
    #[test]
    fn unacknowledged_cssom_commit_survives_execution_drop_and_whole_session_transfer() {
        let cancel = AtomicBool::new(false);
        let (mut session, document) = bound_dispatch_session();
        let target = append_style(&mut session, document, None);
        let mut context = Context {
            styles: session.style_execution(),
            source: session.source_execution(),
            events: Vec::new(),
        };
        let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 4,
            pending_events: 2,
            nested_dispatches: 2,
        })
        .unwrap();
        dispatcher
            .add_listener(
                &mut session.dom(),
                target,
                DomEventType::Load,
                DomEventCallback::new(|_, _, context: &mut Context, event| {
                    context.events.push((
                        event.target().unwrap(),
                        event.current_target().unwrap().node().unwrap(),
                    ));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let DomStyleNetworkingProgress::Complete { receipt, .. } = DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut context,
            dispatch_limits(),
            &cancel,
        )
        .unwrap() else {
            panic!("the original real Load must finish before the disabled edit");
        };
        assert_eq!(receipt.callbacks(), 1);
        assert_eq!(context.events, [(target, target)]);
        drop(context);
        let mut styles = session.style_execution();
        let preference_basis = styles
            .sheet_sets(&session.dom(), document, &cancel)
            .unwrap();
        let before_dom = session.snapshot();
        let before = styles.snapshot(document).unwrap().unwrap();
        let sheet = before.sheets()[0].clone();
        assert!(!before.sheet(&sheet).unwrap().inputs().disabled);
        let weak_state = std::rc::Rc::downgrade(&styles.original().unwrap());
        let operation;
        let cap;
        {
            let mut dom = session.dom();
            let mut lease = dom
                .begin_source_edit(
                    DomSourceEdit::set_style_disabled(target, true),
                    source_limits(),
                    &cancel,
                )
                .unwrap();
            operation = lease.operation();
            let DomSourceProgress::Request(surgeist_dom::DomSourceRequest::Style(request)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("the actual disabled setter must request its CSSOM write");
            };
            assert_eq!(request.action(), DomStyleAction::SetDisabled(true));
            assert!(lease.status().has_published());
            let root = styles.original().unwrap().borrow().roots[0].store.clone();
            // These are genuine, nonmutating producer qualifications on this
            // SAME retained request. Their spent work is never reset/refunded.
            // Warm the revision qualification before measuring its stable cost.
            lease.bind_cssom_root(&request, &root, &cancel).unwrap();
            let request = lease
                .qualify_cssom(&request, document, &root.snapshot().unwrap())
                .unwrap();
            let start = lease.status().spent_work();
            lease.bind_cssom_root(&request, &root, &cancel).unwrap();
            lease
                .qualify_cssom(&request, document, &root.snapshot().unwrap())
                .unwrap();
            let preparation = lease.status().spent_work() - start;
            assert!(preparation > 0);
            let start = lease.status().spent_work();
            drop(lease);
            let lease = dom
                .resume_source_edit(operation, source_limits(), &cancel)
                .unwrap();
            let resume = lease.status().spent_work() - start;
            let spent = lease.status().spent_work();
            drop(lease);
            // Admit the real resume + bind/qualify, excluding acknowledgment's
            // next retained-root visit. No Runtime commit/retry is calibrated.
            cap = spent + resume + preparation;
            let low = DomSourceLimits::new(16_000_000, 1_000_000, cap, 128, 4096, 4096, 1);
            let mut lease = dom.resume_source_edit(operation, low, &cancel).unwrap();
            {
                let original = styles.original().unwrap();
                let mut state = original.borrow_mut();
                assert!(matches!(
                    state.service_source_request(&mut lease, request, &cancel),
                    Err(DomStyleRuntimeError::Dom(DomError::SourceWorkLimit { limit }))
                        if limit == cap
                ));
                let publication = state.publication.as_ref().expect("real unacked commit");
                assert_eq!(
                    publication.request.action(),
                    DomStyleAction::SetDisabled(true)
                );
                assert!(publication.created.is_none());
                assert!(state.staged.is_none());
                assert!(state.completions.is_empty());
                assert_eq!(state.reports.len(), 1);
            }
            assert_eq!(lease.status().operation(), operation);
        }
        let committed = styles.snapshot(document).unwrap().unwrap();
        assert_eq!(committed.sheets(), std::slice::from_ref(&sheet));
        assert!(committed.sheet(&sheet).unwrap().inputs().disabled);
        assert_eq!(committed.revision().value(), before.revision().value() + 1);
        let parked_dom = session.snapshot();
        assert_eq!(parked_dom.incomplete_source_operation(), Some(operation));
        assert!(
            matches!(styles.sheet_sets(&session.dom(), document, &cancel),
            Err(DomStyleRuntimeError::SourcePending(id)) if id == operation)
        );
        assert!(
            matches!(styles.select_sheet_set(&session.dom(), &preference_basis, &"other".into(), 1, &cancel),
            Err(DomStyleRuntimeError::SourcePending(id)) if id == operation)
        );
        assert_eq!(
            styles.snapshot(document).unwrap().unwrap().revision(),
            committed.revision()
        );

        assert!(styles.pending_sheet(0).unwrap().is_none());
        drop(styles); // Sole execution permission; no strong State/root loan survives.
        let DomShutdownOutcome::Pending {
            session: transferred,
            reason: DomShutdownReason::SourceTail(actual),
        } = session.try_shutdown()
        else {
            panic!("shutdown must return the whole unacknowledged composition");
        };
        assert_eq!(actual, operation);
        fn move_session(session: DomSession) -> DomSession {
            session
        }
        let mut session = move_session(transferred);
        let mut styles = session.style_execution();
        assert!(std::rc::Weak::ptr_eq(
            &weak_state,
            &std::rc::Rc::downgrade(&styles.original().unwrap())
        ));
        let mut source = session.source_execution();
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            source.resume(
                &mut session.dom(),
                operation,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancelled,
            ),
            Err(crate::DomSourceDriveError::Source(
                DomSourceRuntimeError::Style(DomStyleRuntimeError::Dom(DomError::Cancelled))
            ))
        ));
        assert_eq!(
            styles.snapshot(document).unwrap().unwrap().revision(),
            committed.revision()
        );
        {
            let original = styles.original().unwrap();
            let state = original.borrow();
            assert!(state.publication.is_some());
        }
        {
            let mut dom = session.dom();
            let mut lease = dom
                .resume_source_edit(operation, source_limits(), &cancel)
                .unwrap();
            let DomSourceProgress::Request(surgeist_dom::DomSourceRequest::Style(request)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("no Source acknowledgment was consumed before recovery");
            };
            assert_eq!(request.action(), DomStyleAction::SetDisabled(true));
            assert_eq!(request.sheet().unwrap().id(), &sheet);
        }
        styles
            .set_limits(&session.dom(), limits(), &cancel)
            .unwrap();
        assert!(matches!(
            source
                .resume(
                    &mut session.dom(),
                    operation,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut |_| panic!("retry must not rerun CSP"),
                        url: None,
                    },
                    &cancel,
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        assert!(session.dom().pending_source_edit().is_none());
        let current = styles.snapshot(document).unwrap().unwrap();
        assert_eq!(current.revision(), committed.revision());
        assert_eq!(current.sheets(), std::slice::from_ref(&sheet));
        assert!(styles.disabled(&session.dom(), target).unwrap().disabled());
        assert!(styles.pending_sheet(0).unwrap().is_none());
        let mut context = Context {
            styles: session.style_execution(),
            source: session.source_execution(),
            events: Vec::new(),
        };
        assert!(matches!(
            DomStyleRuntime::run_networking(
                &mut session.dom(),
                &mut dispatcher,
                &mut context,
                dispatch_limits(),
                &cancel,
            )
            .unwrap(),
            DomStyleNetworkingProgress::Empty
        ));
        assert!(context.events.is_empty());
        drop(context);
        {
            let original = styles.original().unwrap();
            let state = original.borrow();
            assert!(state.publication.is_none() && state.staged.is_none());
            assert_eq!(state.reports.len(), 1);
        }
        assert!(!before.sheet(&sheet).unwrap().inputs().disabled);
        assert_eq!(
            before_dom
                .node(target)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet()
                .unwrap()
                .unwrap()
                .id(),
            &sheet
        );
        assert_eq!(parked_dom.incomplete_source_operation(), Some(operation));
        let DomShutdownOutcome::Pending {
            session,
            reason: DomShutdownReason::LiveResources,
        } = session.try_shutdown()
        else {
            panic!("the bound root still requires terminal disposal");
        };
        drop(source);
        drop(session);
        assert!(weak_state.upgrade().is_none());
        assert!(matches!(
            styles.snapshot(document),
            Err(DomStyleRuntimeError::SessionUnavailable)
        ));
    }
    #[test]
    fn processing_source_storage_refusal_and_policy_raise_resume_original_operation_without_recreation()
     {
        let cancel = AtomicBool::new(false);
        let (mut session, document) = bound_session();
        let mut permission = session.style_execution();
        let mut source = session.source_execution();
        let original_root = permission.snapshot(document).unwrap().unwrap();
        let mut low = limits();
        low.storage_bytes = 20_000;
        permission.set_limits(&session.dom(), low, &cancel).unwrap();
        let mut batch = DomBatch::new();
        let style = batch.create_element_ns(document, Namespace::html(), "style");
        let media = format!("screen and ({}", "x".repeat(50_000));
        batch.set_attribute(&style, "media", media);
        let container = session.snapshot().node(document).unwrap().children()[0];
        let setup = session.dom().apply(batch, &cancel).unwrap();
        let style = setup.created(&style).unwrap().unwrap();
        let edit = DomSourceEdit::insert_before(style, container, None);
        assert!(matches!(
            source.apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None
                },
                &cancel
            ),
            Err(crate::DomSourceDriveError::Source(
                DomSourceRuntimeError::Style(DomStyleRuntimeError::Admission)
            ))
        ));
        let operation = session.dom().pending_source_edit().unwrap().operation();
        assert_eq!(
            permission.snapshot(document).unwrap().unwrap().revision(),
            original_root.revision(),
            "real CSSOM commit must not precede complete processing-result admission"
        );
        assert!(permission.pending_sheet(0).unwrap().is_none());
        let before_policy = permission.original().unwrap().borrow().limits;
        let mut rejected = limits();
        rejected.storage_bytes = 0;
        assert!(matches!(
            permission.set_limits(&session.dom(), rejected, &cancel),
            Err(DomStyleRuntimeError::Admission)
        ));
        assert_eq!(
            permission.original().unwrap().borrow().limits.storage_bytes,
            before_policy.storage_bytes
        );
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            permission.set_limits(&session.dom(), limits(), &cancelled),
            Err(DomStyleRuntimeError::Dom(DomError::Cancelled))
        ));
        let mut raised = limits();
        raised.storage_bytes = 4_000_000;
        permission
            .set_limits(&session.dom(), raised, &cancel)
            .unwrap();
        assert!(matches!(
            source
                .resume(
                    &mut session.dom(),
                    operation,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        assert!(permission.pending_sheet(0).unwrap().is_some());
        assert!(permission.pending_sheet(1).unwrap().is_none());
        assert!(
            !permission
                .processed_sheet(0)
                .unwrap()
                .unwrap()
                .media_diagnostics()
                .is_empty()
        );
    }
    #[test]
    fn bootstrap_refuses_original_pending_and_prebound_dom_without_losing_either_actual_owner() {
        let cancel = AtomicBool::new(false);
        for bind in [false, true] {
            let mut owner = dom();
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            let commit = owner.apply(batch, &cancel).unwrap();
            let document = commit.created(&document).unwrap().unwrap();
            let identity = owner.owner();
            let root = DomStyleCssomRoot::new(
                document,
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            );
            let root_revision = root.snapshot().unwrap().revision().clone();
            let mut batch = DomBatch::new();
            let style = batch.create_element_ns(document, Namespace::html(), "style");
            let setup = owner.apply(batch, &cancel).unwrap();
            let style = setup.created(&style).unwrap().unwrap();
            let mut lease = owner
                .begin_source_edit(
                    DomSourceEdit::insert_before(style, document, None),
                    source_limits(),
                    &cancel,
                )
                .unwrap();
            let operation = lease.operation();
            let mut bound_request = None;
            loop {
                match lease.advance(&cancel).unwrap() {
                    DomSourceProgress::Request(surgeist_dom::DomSourceRequest::Style(request)) => {
                        match request.action() {
                            DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp) => {
                                if !bind {
                                    break;
                                }
                                lease
                                    .accept_inline_csp(
                                        &request,
                                        &DomStyleInlineCspObservation::complete_empty(
                                            request.source(),
                                            1,
                                        ),
                                        &cancel,
                                    )
                                    .unwrap();
                            }
                            DomStyleAction::Update(HtmlStyleUpdateAction::CreateSheet) => {
                                lease.bind_cssom_root(&request, &root, &cancel).unwrap();
                                bound_request = Some(request);
                                break;
                            }
                            action => panic!("unexpected actual source request {action:?}"),
                        }
                    }
                    DomSourceProgress::Request(surgeist_dom::DomSourceRequest::HostedChild(_)) => {
                        panic!("Style fixture cannot request a hosted child")
                    }
                    DomSourceProgress::Request(surgeist_dom::DomSourceRequest::DesignMode(_)) => {
                        panic!("Style fixture cannot request a designMode service")
                    }
                    DomSourceProgress::Request(
                        surgeist_dom::DomSourceRequest::FullscreenExitHandoff(_),
                    ) => {
                        panic!("Style fixture cannot request a fullscreen exit handoff")
                    }
                    DomSourceProgress::Request(
                        surgeist_dom::DomSourceRequest::Event(_)
                        | surgeist_dom::DomSourceRequest::Dialog(_)
                        | surgeist_dom::DomSourceRequest::DialogFocusing(_)
                        | surgeist_dom::DomSourceRequest::FocusDelegate(_)
                        | surgeist_dom::DomSourceRequest::AutofocusInsertion(_)
                        | surgeist_dom::DomSourceRequest::CloseWatcher(_),
                    ) => {
                        panic!("Style fixture cannot request a Dialog service")
                    }
                    DomSourceProgress::Complete(_) => panic!("real pending source was expected"),
                }
            }
            drop(lease);
            let before = owner.snapshot().stamp();
            let rejected = DomSession::new(owner, session_limits(limits())).unwrap_err();
            if bind {
                assert!(matches!(
                    rejected.error(),
                    DomSessionCreateFailure::Style(DomStyleRuntimeError::PreboundSession)
                ));
            } else {
                assert!(
                    matches!(rejected.error(),DomSessionCreateFailure::Style(DomStyleRuntimeError::SourcePending(id)) if *id==operation)
                );
            }
            let mut owner = rejected.into_dom();
            assert_eq!(owner.owner(), identity);
            assert_eq!(owner.snapshot().stamp(), before);
            assert_eq!(owner.pending_source_edit().unwrap().operation(), operation);
            if let Some(request) = bound_request {
                let lease = owner
                    .resume_source_edit(operation, source_limits(), &cancel)
                    .unwrap();
                assert_eq!(
                    lease
                        .cssom_root(&request)
                        .unwrap()
                        .snapshot()
                        .unwrap()
                        .revision(),
                    &root_revision
                );
            }
        }
    }
    #[test]
    fn root_label_comparison_is_bounded_and_refusal_preserves_the_actual_input_store() {
        let cancel = AtomicBool::new(false);
        let mut owner = dom();
        let mut batch = DomBatch::new();
        let first = batch.create_document();
        let second = batch.create_document();
        let commit = owner.apply(batch, &cancel).unwrap();
        let first = commit.created(&first).unwrap().unwrap();
        let second = commit.created(&second).unwrap().unwrap();
        let mut initial = limits();
        initial.work = 1000;
        let mut session = DomSession::new(owner, session_limits(initial)).unwrap();
        let mut permission = session.style_execution();
        let label = format!("{}a", "x".repeat(1024));
        permission
            .bind_root(
                &session.dom(),
                label,
                &mut Some(DomStyleCssomRoot::new(
                    first,
                    CssomStore::new(Default::default(), Default::default()).unwrap(),
                )),
                &cancel,
            )
            .unwrap();
        let mut input = Some(DomStyleCssomRoot::new(
            second,
            CssomStore::new(Default::default(), Default::default()).unwrap(),
        ));
        let original = input.as_ref().unwrap().snapshot().unwrap();
        let label = format!("{}b", "x".repeat(1024));
        assert!(matches!(
            permission.bind_root(&session.dom(), label.clone(), &mut input, &cancel),
            Err(DomStyleRuntimeError::Admission)
        ));
        assert_eq!(
            input.as_ref().unwrap().snapshot().unwrap().revision(),
            original.revision()
        );
        assert!(permission.snapshot(second).unwrap().is_none());
        permission
            .set_limits(&session.dom(), limits(), &cancel)
            .unwrap();
        permission
            .bind_root(&session.dom(), label, &mut input, &cancel)
            .unwrap();
        assert!(input.is_none());
        assert_eq!(
            permission.snapshot(second).unwrap().unwrap().revision(),
            original.revision()
        );
    }
    #[test]
    fn exact_result_payload_admission_counts_real_headers_backing_sources_and_ignores_external_arc_clones()
     {
        let cancel = AtomicBool::new(false);
        let (mut session, document) = bound_session();
        let first = append_style(&mut session, document, Some("screen and"));
        append_style(&mut session, document, Some("print and"));
        let mut permission = session.style_execution();
        let mut source = session.source_execution();
        let edit = DomSourceEdit::set_attribute(first, "title", "changed");
        assert!(matches!(
            source
                .apply(
                    &mut session.dom(),
                    edit,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let first_result = permission.processed_sheet(0).unwrap().unwrap();
        let attribute_result = permission.processed_sheet(2).unwrap().unwrap();
        let first_source = first_result.media_diagnostics()[0].source().unwrap();
        let next_source = attribute_result.media_diagnostics()[0].source().unwrap();
        assert!(!first_source.same_snapshot(next_source));
        assert!(
            Arc::ptr_eq(
                first_source.original_utf16().unwrap(),
                next_source.original_utf16().unwrap()
            ),
            "actual repeated processing borrows the same canonical media backing"
        );
        let original = permission.original().unwrap();
        // The owner-local inline/capacity terms come from this crate's real
        // allocations. Opaque diagnostics/source payloads come only from their
        // public producer receipts, not copied private layouts or logical text.
        let expected = {
            let state = original.borrow();
            let mut bytes = size_of::<std::cell::RefCell<StyleState>>()
                + state.roots.capacity() * size_of::<Root>()
                + state.completions.capacity() * size_of::<Completion>()
                + state.reports.capacity() * size_of::<Arc<DomStyleProcessedSheet>>()
                + state
                    .roots
                    .iter()
                    .map(|root| {
                        // Each Document in this fixture retains one empty
                        // preferred-name Vec target; no explicit selection ran.
                        root.label.capacity()
                            + size_of::<DomStyleCssomRoot>()
                            + size_of::<Vec<u16>>()
                    })
                    .sum::<usize>()
                + state.reports.len() * size_of::<DomStyleProcessedSheet>();
            let mut sources = CssSourceRetention::new(usize::MAX, 100_000, 128);
            for report in &state.reports {
                let diagnostics = match &report.processing {
                    Processing::Created(value) => {
                        bytes += value
                            .retained_backing_bytes_with(|| Ok::<_, ()>(()))
                            .unwrap();
                        value.media_diagnostics()
                    }
                    Processing::Attributes(value) => {
                        bytes += value
                            .retained_backing_bytes_with(|| Ok::<_, ()>(()))
                            .unwrap();
                        value.diagnostics()
                    }
                };
                assert!(!diagnostics.is_empty());
                for diagnostic in diagnostics {
                    sources.visit(diagnostic).unwrap();
                }
            }
            bytes
                + sources
                    .complete_payload_bytes_with(|| Ok::<_, ()>(()))
                    .unwrap()
        };
        let retained = permission.processed_sheet(0).unwrap().unwrap();
        let another = retained.clone();
        let mut exact = limits();
        exact.storage_bytes = expected;
        let mut short = exact;
        short.storage_bytes -= 1;
        assert!(matches!(
            permission.set_limits(&session.dom(), short, &cancel),
            Err(DomStyleRuntimeError::Admission)
        ));
        permission
            .set_limits(&session.dom(), exact, &cancel)
            .unwrap();
        drop(another);
        permission
            .set_limits(&session.dom(), exact, &cancel)
            .unwrap();
        assert!(!retained.media_diagnostics().is_empty());
        let mut invalid = exact;
        invalid.reports = 1;
        assert!(matches!(
            permission.set_limits(&session.dom(), invalid, &cancel),
            Err(DomStyleRuntimeError::Admission)
        ));
        assert_eq!(original.borrow().limits.reports, exact.reports);
    }
    #[test]
    fn work_refusal_before_callback_and_raised_policy_keep_the_exact_task_and_monotone_work() {
        let cancel = AtomicBool::new(false);
        let (mut session, document) = bound_dispatch_session();
        let target = append_style(&mut session, document, None);
        let mut context = Context {
            styles: session.style_execution(),
            source: session.source_execution(),
            events: Vec::new(),
        };
        let original = context.styles.original().unwrap();
        let task = original.borrow().completions[0].task().id();
        let spent = original.borrow().work.get();
        let mut policy_limits = limits();
        policy_limits.work = spent + 256;
        context
            .styles
            .set_limits(&session.dom(), policy_limits, &cancel)
            .unwrap();
        let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 4,
            pending_events: 1,
            nested_dispatches: 2,
        })
        .unwrap();
        dispatcher
            .add_listener(
                &mut session.dom(),
                target,
                DomEventType::Load,
                DomEventCallback::new(|_, _, context: &mut Context, event| {
                    context.events.push((
                        event.target().unwrap(),
                        event.current_target().unwrap().node().unwrap(),
                    ));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        assert!(matches!(
            DomStyleRuntime::run_networking(
                &mut session.dom(),
                &mut dispatcher,
                &mut context,
                dispatch_limits(),
                &cancel
            ),
            Err(DomStyleRuntimeError::Admission)
        ));
        assert!(context.events.is_empty());
        assert_eq!(original.borrow().completions[0].task().id(), task);
        let refused_work = original.borrow().work.get();
        assert!(refused_work > spent);
        context
            .styles
            .set_limits(&session.dom(), limits(), &cancel)
            .unwrap();
        assert!(original.borrow().work.get() >= refused_work);
        let DomStyleNetworkingProgress::Complete {
            task: completed,
            receipt,
        } = DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut context,
            dispatch_limits(),
            &cancel,
        )
        .unwrap()
        else {
            panic!("same admitted task completes")
        };
        assert_eq!(completed, task);
        assert_eq!(receipt.callbacks(), 1);
        assert_eq!(context.events.len(), 1);
    }
    #[test]
    fn ordinary_style_has_null_constructor_base_location_and_document_while_policy_keeps_document_context()
     {
        let cancel = AtomicBool::new(false);
        let (mut session, document) = bound_session();
        let styles = session.style_execution();
        let mut source = session.source_execution();
        let parent = session.snapshot().node(document).unwrap().children()[0];
        let mut batch = DomBatch::new();
        let target = batch.create_element_ns(document, Namespace::html(), "style");
        let setup = session.dom().apply(batch, &cancel).unwrap();
        let target = setup.created(&target).unwrap().unwrap();
        let edit = DomSourceEdit::insert_before(target, parent, None);
        let mut observed = None;
        let DomSourceDriveProgress::Complete(_) = source
            .apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |source| {
                        observed = Some((
                            source.base_url().clone(),
                            source.document_metadata().url().clone(),
                        ));
                        policy(source)
                    },
                    url: None,
                },
                &cancel,
            )
            .unwrap()
        else {
            panic!("actual ordinary Style completes");
        };
        assert_eq!(
            observed,
            Some(("about:blank".into(), "about:blank".into())),
            "the actual influencing Document context remains available to policy"
        );
        let canonical = session.snapshot();
        let association = canonical
            .node(target)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap();
        let current = styles.snapshot(document).unwrap().unwrap();
        let inputs = current.sheet(association.id()).unwrap().inputs();
        assert!(!inputs.constructed);
        assert_eq!(
            inputs.base_url, None,
            "ordinary inline Style has no constructed stylesheet-base override"
        );
        assert_eq!(inputs.location, None);
        assert_eq!(inputs.constructor_document, None);
    }

    fn mode_profile(parser: CssParserContext) -> CssomContext {
        CssomContext {
            inputs: vec![
                CssomInput {
                    version: CssomInputVersion {
                        role: CssomInputRole::Layer,
                        identity: "actual-layer".into(),
                        revision: 7,
                    },
                    data: CssomInputData::Layer {
                        path: vec!["theme".into(), "components".into()],
                        ordinal: 3,
                    },
                },
                CssomInput {
                    version: CssomInputVersion {
                        role: CssomInputRole::ParserMode,
                        identity: "external-parser".into(),
                        revision: 11,
                    },
                    data: CssomInputData::ParserMode(parser),
                },
            ],
            linked: Vec::new(),
        }
    }
    fn parser_input(snapshot: &CssomSnapshot) -> &CssomInput {
        snapshot
            .context()
            .inputs
            .iter()
            .find(|input| matches!(input.data, CssomInputData::ParserMode(_)))
            .unwrap()
    }
    #[test]
    fn current_profile_and_source_mode_compose_for_body_owner_attributes_and_unchanged_mode() {
        let cancel = AtomicBool::new(false);
        let mut linked = CssomStore::new(Default::default(), Default::default()).unwrap();
        let mut batch = linked
            .batch(CssomEditGuard::from_snapshot(&linked.snapshot()))
            .unwrap();
        let ticket = batch
            .create_parsed_sheet_report(
                CssParserContext::default().parse_sheet(""),
                CssomSheetInputs::external(
                    CssomInputVersion {
                        role: CssomInputRole::Owner,
                        identity: "linked-real-sheet".into(),
                        revision: 1,
                    },
                    true,
                ),
            )
            .unwrap();
        let done = linked.commit(batch).unwrap();
        let linked_sheet = done.sheet(&ticket).unwrap().clone();
        let mut profile = mode_profile(
            CssParserContext::new(CssParserMode::Quirks).with_svg_glyph_orientation_vertical(),
        );
        profile.linked.push(surgeist_cssom::CssomLinkedInput {
            version: CssomInputVersion {
                role: CssomInputRole::Import,
                identity: "observed-link".into(),
                revision: 2,
            },
            snapshot: linked.snapshot(),
            sheet: linked_sheet,
        });
        let original_profile = profile.clone();
        let (mut session, document) = bound_session_with_context(profile);
        let mut styles = session.style_execution();
        let mut source = session.source_execution();
        let mut raised = limits();
        raised.storage_bytes = 8_000_000;
        styles.set_limits(&session.dom(), raised, &cancel).unwrap();
        let initial = styles.snapshot(document).unwrap().unwrap();
        let parent = session.snapshot().node(document).unwrap().children()[0];
        let mut batch = DomBatch::new();
        let target = batch.create_element_ns(document, Namespace::html(), "style");
        let text = batch.create_text(document, ".a{width:1}");
        batch.insert_before(&text, &target, None);
        let setup = session.dom().apply(batch, &cancel).unwrap();
        let target = setup.created(&target).unwrap().unwrap();
        let edit = DomSourceEdit::insert_before(target, parent, None);
        let DomSourceDriveProgress::Complete(_) = source
            .apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap()
        else {
            panic!("actual source completes");
        };
        let body_snapshot = styles.snapshot(document).unwrap().unwrap();
        let sheet = body_snapshot.sheets()[0].clone();
        let association = session
            .snapshot()
            .node(target)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap()
            .association();
        let expected =
            CssParserContext::new(CssParserMode::Standards).with_svg_glyph_orientation_vertical();
        assert_eq!(body_snapshot.owner(), initial.owner());
        assert_eq!(
            body_snapshot.context().inputs[0],
            original_profile.inputs[0]
        );
        assert_eq!(body_snapshot.context().linked, original_profile.linked);
        assert!(
            matches!(parser_input(&body_snapshot).data, CssomInputData::ParserMode(value) if value == expected)
        );
        assert_eq!(
            body_snapshot.sheet(&sheet).unwrap().authored().rules()[0]
                .serialize_cssom()
                .unwrap(),
            ".a { }"
        );
        assert_eq!(
            body_snapshot
                .sheet(&sheet)
                .unwrap()
                .authored()
                .parse_source()
                .unwrap()
                .context(),
            expected
        );
        let cap = session.dom().style_cssom_root(document).unwrap().unwrap();
        let before = cap.snapshot().unwrap();
        let mut external = before.context().clone();
        let row = external
            .inputs
            .iter_mut()
            .find(|input| matches!(input.data, CssomInputData::ParserMode(_)))
            .unwrap();
        row.data = CssomInputData::ParserMode(CssParserContext::new(CssParserMode::Quirks));
        row.version.identity = "external-second-profile".into();
        row.version.revision = 19;
        let stamp = session.snapshot().stamp();
        let mut batch = cap.batch(CssomEditGuard::from_snapshot(&before)).unwrap();
        batch.update_context(external).unwrap();
        cap.commit(&mut Some(batch)).unwrap();
        assert_eq!(
            session.snapshot().stamp(),
            stamp,
            "actual external configuration edit changes no DOM state"
        );
        let edit = DomSourceEdit::set_attribute(target, "media", "screen and");
        assert!(matches!(
            source
                .apply(
                    &mut session.dom(),
                    edit,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let media_snapshot = styles.snapshot(document).unwrap().unwrap();
        assert_eq!(media_snapshot.sheets(), std::slice::from_ref(&sheet));
        assert_eq!(
            media_snapshot.context().inputs[0],
            original_profile.inputs[0]
        );
        assert_eq!(media_snapshot.context().linked, original_profile.linked);
        assert!(
            matches!(parser_input(&media_snapshot).data, CssomInputData::ParserMode(value) if value == CssParserContext::default())
        );
        assert_eq!(
            media_snapshot
                .sheet(&sheet)
                .unwrap()
                .inputs()
                .media
                .parse_source()
                .unwrap()
                .context(),
            CssParserContext::default()
        );
        assert_eq!(
            media_snapshot
                .sheet(&sheet)
                .unwrap()
                .authored()
                .parse_source()
                .unwrap()
                .context(),
            expected,
            "already checked body keeps its original non-mode profile"
        );
        assert_eq!(
            body_snapshot
                .sheet(&sheet)
                .unwrap()
                .authored()
                .parse_source()
                .unwrap()
                .context(),
            expected
        );
        assert_eq!(
            session
                .snapshot()
                .node(target)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet()
                .unwrap()
                .unwrap()
                .association(),
            association
        );
        assert!(
            styles.pending_sheet(1).unwrap().is_none(),
            "attribute processing queues no second load"
        );
        assert!(
            !styles
                .processed_sheet(1)
                .unwrap()
                .unwrap()
                .media_diagnostics()
                .is_empty()
        );
        let edit = DomSourceEdit::set_attribute(target, "title", "same-mode");
        assert!(matches!(
            source
                .apply(
                    &mut session.dom(),
                    edit,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let after = styles.snapshot(document).unwrap().unwrap();
        assert_eq!(
            after.context(),
            media_snapshot.context(),
            "correct mode preserves exact existing profile/version"
        );
    }
    #[test]
    fn composed_mode_input_versions_distinguish_external_configuration_edits_at_the_same_dom_stamp()
    {
        let cancel = AtomicBool::new(false);
        let (mut session, document) = bound_session_with_context(mode_profile(
            CssParserContext::new(CssParserMode::Quirks).with_svg_glyph_orientation_vertical(),
        ));
        let styles = session.style_execution();
        let mut source = session.source_execution();
        let parent = session.snapshot().node(document).unwrap().children()[0];
        let mut batch = DomBatch::new();
        let target = batch.create_element_ns(document, Namespace::html(), "style");
        let setup = session.dom().apply(batch, &cancel).unwrap();
        let target = setup.created(&target).unwrap().unwrap();
        let edit = DomSourceEdit::insert_before(target, parent, None);
        let DomSourceDriveProgress::Pending {
            request,
            reason: DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy),
        } = source
            .apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel,
            )
            .unwrap()
        else {
            panic!("real canonical pending source request");
        };
        let original = styles.original().unwrap();
        let cap = original.borrow().roots[0].store.clone();
        let dom_stamp = session.snapshot().stamp();
        let surgeist_dom::DomSourceRequest::Style(request) = request else {
            panic!("Style fixture cannot request a hosted child");
        };
        let source_stamp = request.source().stamp();
        let before = cap.snapshot().unwrap();
        let mut batch = cap.batch(CssomEditGuard::from_snapshot(&before)).unwrap();
        let selected = {
            let state = original.borrow();
            state
                .source_parser_context(
                    &before,
                    request.source(),
                    &state.roots[0].label,
                    &mut batch,
                    &cancel,
                )
                .unwrap()
        };
        cap.commit(&mut Some(batch)).unwrap();
        let first = cap.snapshot().unwrap();
        assert_eq!(
            selected,
            CssParserContext::default().with_svg_glyph_orientation_vertical()
        );
        assert_eq!(
            parser_input(&first).version.revision,
            before.revision().value() + 1
        );
        let mut external = first.context().clone();
        let row = external
            .inputs
            .iter_mut()
            .find(|input| matches!(input.data, CssomInputData::ParserMode(_)))
            .unwrap();
        row.data = CssomInputData::ParserMode(CssParserContext::new(CssParserMode::Quirks));
        row.version.identity = "second-external-profile".into();
        row.version.revision = 41;
        let mut batch = cap.batch(CssomEditGuard::from_snapshot(&first)).unwrap();
        batch.update_context(external).unwrap();
        cap.commit(&mut Some(batch)).unwrap();
        let changed = cap.snapshot().unwrap();
        let mut batch = cap.batch(CssomEditGuard::from_snapshot(&changed)).unwrap();
        let selected = {
            let state = original.borrow();
            state
                .source_parser_context(
                    &changed,
                    request.source(),
                    &state.roots[0].label,
                    &mut batch,
                    &cancel,
                )
                .unwrap()
        };
        cap.commit(&mut Some(batch)).unwrap();
        let second = cap.snapshot().unwrap();
        assert_eq!(selected, CssParserContext::default());
        assert_eq!(session.snapshot().stamp(), dom_stamp);
        assert_eq!(request.source().stamp(), source_stamp);
        assert_eq!(first.owner(), second.owner());
        assert_eq!(
            parser_input(&first).version.identity,
            parser_input(&second).version.identity
        );
        assert_ne!(
            parser_input(&first).version.revision,
            parser_input(&second).version.revision
        );
        assert_eq!(
            parser_input(&second).version.revision,
            changed.revision().value() + 1
        );
        assert!(first.sheets().is_empty() && second.sheets().is_empty());
        assert!(
            matches!(
                session.try_shutdown(),
                DomShutdownOutcome::Pending {
                    reason: DomShutdownReason::SourceTail(_),
                    ..
                }
            ),
            "context work does not manufacture source completion"
        );
    }
    #[test]
    fn context_copy_work_refusal_preserves_unpublished_store_and_resumes_the_original_operation() {
        let cancel = AtomicBool::new(false);
        let mut profile = mode_profile(CssParserContext::new(CssParserMode::Quirks));
        profile.inputs[0].data = CssomInputData::Layer {
            path: vec!["x".repeat(50_000)],
            ordinal: 3,
        };
        let (mut session, document) = bound_session_with_context(profile);
        let mut styles = session.style_execution();
        let mut source = session.source_execution();
        let initial = styles.snapshot(document).unwrap().unwrap();
        let original = styles.original().unwrap();
        let mut low = limits();
        low.work = original.borrow().work.get() + 128;
        styles.set_limits(&session.dom(), low, &cancel).unwrap();
        let parent = session.snapshot().node(document).unwrap().children()[0];
        let mut batch = DomBatch::new();
        let target = batch.create_element_ns(document, Namespace::html(), "style");
        let setup = session.dom().apply(batch, &cancel).unwrap();
        let target = setup.created(&target).unwrap().unwrap();
        let edit = DomSourceEdit::insert_before(target, parent, None);
        assert!(matches!(
            source.apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None
                },
                &cancel
            ),
            Err(crate::DomSourceDriveError::Source(
                DomSourceRuntimeError::Style(DomStyleRuntimeError::Admission)
            ))
        ));
        let pending = session.dom().pending_source_edit().unwrap().operation();
        let refused = styles.snapshot(document).unwrap().unwrap();
        assert_eq!(refused.revision(), initial.revision());
        assert_eq!(refused.context(), initial.context());
        assert!(refused.sheets().is_empty());
        assert!(styles.pending_sheet(0).unwrap().is_none());
        let spent = original.borrow().work.get();
        let mut raised = limits();
        raised.storage_bytes = 8_000_000;
        styles.set_limits(&session.dom(), raised, &cancel).unwrap();
        assert!(original.borrow().work.get() >= spent);
        assert!(matches!(
            source
                .resume(
                    &mut session.dom(),
                    pending,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let current = styles.snapshot(document).unwrap().unwrap();
        assert_eq!(current.owner(), initial.owner());
        assert_eq!(current.sheets().len(), 1);
        assert_eq!(current.context().inputs[0], initial.context().inputs[0]);
        assert!(
            matches!(parser_input(&current).data, CssomInputData::ParserMode(value) if value == CssParserContext::default())
        );
        assert!(styles.pending_sheet(0).unwrap().is_some());
    }

    fn shadow_root_mode_adoption(
        initial_quirks: bool,
        expected_rule: &str,
        expected_mode: CssParserMode,
    ) {
        use surgeist_cssom::{CssomContext, CssomInput, CssomInputData};
        use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};
        let cancel = AtomicBool::new(false);
        let mut owner = dom();
        let mut batch = DomBatch::new();
        let native = batch.create_document();
        let done = owner.apply(batch, &cancel).unwrap();
        let native = done.created(&native).unwrap().unwrap();
        let environment = {
            let snapshot = owner.snapshot();
            let document = snapshot.node(native).unwrap().as_document().unwrap();
            let metadata = document.metadata();
            // Explicit fixture observations: an actual about:blank URL and
            // UTF-8 encoding identity, independent of absent native Base facts.
            use surgeist_html_semantics::{
                HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
                HtmlBaseUrlScheme,
            };
            let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1);
            let url = HtmlBaseUrlRecord::observed(
                revision,
                "about:blank".into(),
                HtmlBaseUrlScheme::Other,
                true,
                None,
            )
            .unwrap();
            surgeist_dom::DomParsedDocumentEnvironment::headless(
                metadata.origin().clone(),
                url,
                HtmlBaseEncoding::fresh(),
                metadata.encoding().clone(),
                metadata.document_environment().input_platform(),
            )
        };
        let parse_limits = HtmlParseLimits::builder()
            .source_units(100_000)
            .buffered_utf16_units(100_000)
            .token_attributes(10_000)
            .auxiliary_entries(1_000_000)
            .managed_nodes(100_000)
            .managed_edges(100_000)
            .managed_attributes(100_000)
            .managed_utf16_units(100_000)
            .source_map_entries(100_000)
            .source_map_payload_units(100_000)
            .diagnostics(100_000)
            .diagnostic_payload_units(100_000)
            .context_effects(100_000)
            .open_elements(100)
            .tree_depth(100_000)
            .managed_capacity_bytes(100_000_000)
            .peak_transient_bytes(100_000_000)
            .atomic_layout_bytes(100_000_000)
            .work_credits(100_000_000)
            .finish()
            .unwrap();
        let report = Arc::new(
            HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), parse_limits)
                .parse()
                .unwrap(),
        );
        let installed = owner
            .install_parsed_html_document(
                owner.snapshot().stamp(),
                report,
                environment,
                surgeist_dom::DomParsedHtmlInstallLimits::new(
                    1024,
                    10_000_000,
                    16_000_000,
                    100_000_000,
                ),
                &cancel,
            )
            .unwrap();
        let quirks = installed.document().node();
        let body = installed
            .bindings()
            .iter()
            .find_map(|binding| {
                installed
                    .snapshot()
                    .node(binding.node())
                    .unwrap()
                    .as_element()
                    .filter(|element| {
                        element
                            .name()
                            .local_name()
                            .code_units()
                            .iter()
                            .copied()
                            .eq("body".encode_utf16())
                    })
                    .map(|_| binding.node())
            })
            .unwrap();
        assert_eq!(
            owner
                .snapshot()
                .node(quirks)
                .unwrap()
                .as_document()
                .unwrap()
                .metadata()
                .mode(),
            HtmlDocumentMode::Quirks
        );
        assert_eq!(
            owner
                .snapshot()
                .node(native)
                .unwrap()
                .as_document()
                .unwrap()
                .metadata()
                .mode(),
            HtmlDocumentMode::NoQuirks
        );
        let (initial, destination, parent) = if initial_quirks {
            (quirks, native, native)
        } else {
            (native, quirks, body)
        };
        let mut batch = DomBatch::new();
        let host = batch.create_element_ns(initial, Namespace::html(), "div");
        let shadow = batch.attach_shadow(
            &host,
            surgeist_dom::DomShadowRootInit::new(surgeist_html::HtmlShadowRootMode::Open),
        );
        let done = owner.apply(batch, &cancel).unwrap();
        let host = done.created(&host).unwrap().unwrap();
        let shadow = done.created(&shadow).unwrap().unwrap();
        let initial_mode = if initial_quirks {
            CssParserMode::Quirks
        } else {
            CssParserMode::Standards
        };
        let root_context = CssomContext {
            inputs: vec![CssomInput {
                version: CssomInputVersion {
                    role: CssomInputRole::ParserMode,
                    identity: "original-profile".into(),
                    revision: 1,
                },
                data: CssomInputData::ParserMode(CssParserContext::new(initial_mode)),
            }],
            linked: Vec::new(),
        };
        let mut runtime_limits = limits();
        runtime_limits.storage_bytes = 8_000_000;
        let mut session = DomSession::new(owner, session_limits(runtime_limits)).unwrap();
        let mut styles = session.style_execution();
        let mut source = session.source_execution();
        styles
            .bind_root(
                &session.dom(),
                "adopted-shadow".into(),
                &mut Some(DomStyleCssomRoot::new(
                    shadow,
                    CssomStore::new(Default::default(), root_context).unwrap(),
                )),
                &cancel,
            )
            .unwrap();
        let original = styles.snapshot(shadow).unwrap().unwrap();
        let edit = DomSourceEdit::insert_before(host, parent, None);
        assert!(matches!(
            source
                .apply(
                    &mut session.dom(),
                    edit,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        assert_eq!(
            session
                .snapshot()
                .node(shadow)
                .unwrap()
                .owner_document()
                .map(|id| id.node()),
            Some(destination)
        );
        let mut batch = DomBatch::new();
        let style = batch.create_element_ns(destination, Namespace::html(), "style");
        let text = batch.create_text(destination, ".a{width:1}");
        batch.insert_before(&text, &style, None);
        let setup = session.dom().apply(batch, &cancel).unwrap();
        let style = setup.created(&style).unwrap().unwrap();
        let edit = DomSourceEdit::insert_before(style, shadow, None);
        assert!(matches!(
            source
                .apply(
                    &mut session.dom(),
                    edit,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let current = styles.snapshot(shadow).unwrap().unwrap();
        assert_eq!(
            current.owner(),
            original.owner(),
            "adoption must preserve actual CSSOM owner"
        );
        assert!(
            original.sheets().is_empty(),
            "historical snapshot remains unchanged"
        );
        assert_eq!(current.sheets().len(), 1);
        let sheet = current.sheet(&current.sheets()[0]).unwrap();
        assert_eq!(
            sheet.authored().rules()[0].serialize_cssom().unwrap(),
            expected_rule,
            "stylesheet value must use the actual source-time document mode"
        );
        assert_eq!(
            sheet.authored().parse_source().unwrap().context().mode(),
            expected_mode
        );
        assert_eq!(
            sheet
                .inputs()
                .media
                .parse_source()
                .unwrap()
                .context()
                .mode(),
            expected_mode
        );
    }
    #[test]
    fn shadow_root_adoption_quirks_to_standards_uses_current_document_for_actual_stylesheet() {
        shadow_root_mode_adoption(true, ".a { }", CssParserMode::Standards);
    }
    #[test]
    fn shadow_root_adoption_standards_to_quirks_uses_current_document_for_actual_stylesheet() {
        shadow_root_mode_adoption(false, ".a { width: 1px; }", CssParserMode::Quirks);
    }

    #[test]
    fn initial_event_storage_refusal_keeps_the_original_unstarted_task_for_raised_policy_retry() {
        let cancel = AtomicBool::new(false);
        let (mut session, document) = bound_dispatch_session();
        let target = append_style(&mut session, document, None);
        let mut context = Context {
            styles: session.style_execution(),
            source: session.source_execution(),
            events: Vec::new(),
        };
        let original = context.styles.original().unwrap();
        let current = original
            .borrow()
            .storage(None, &cancel, limits().work)
            .unwrap();
        let id = original.borrow().completions[0].task().id();
        let mut tight = limits();
        tight.storage_bytes = current + 1;
        context
            .styles
            .set_limits(&session.dom(), tight, &cancel)
            .unwrap();
        let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 4,
            pending_events: 1,
            nested_dispatches: 2,
        })
        .unwrap();
        dispatcher
            .add_listener(
                &mut session.dom(),
                target,
                DomEventType::Load,
                DomEventCallback::new(|_, _, context: &mut Context, event| {
                    context.events.push((
                        event.target().unwrap(),
                        event.current_target().unwrap().node().unwrap(),
                    ));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        assert!(matches!(
            DomStyleRuntime::run_networking(
                &mut session.dom(),
                &mut dispatcher,
                &mut context,
                dispatch_limits(),
                &cancel
            ),
            Err(DomStyleRuntimeError::Admission)
        ));
        assert!(context.events.is_empty());
        {
            let state = original.borrow();
            let task = state.completions[0].task();
            assert_eq!(task.id(), id);
            assert!(!task.running);
            assert!(matches!(task.delivery, Delivery::NotStarted));
        }
        context
            .styles
            .set_limits(&session.dom(), limits(), &cancel)
            .unwrap();
        let DomStyleNetworkingProgress::Complete { task, receipt } =
            DomStyleRuntime::run_networking(
                &mut session.dom(),
                &mut dispatcher,
                &mut context,
                dispatch_limits(),
                &cancel,
            )
            .unwrap()
        else {
            panic!("same actual task completes after policy adjustment");
        };
        assert_eq!(task, id);
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.failures().is_empty());
        assert_eq!(context.events.len(), 1);
    }

    #[test]
    fn target_destroyed_after_prestart_dispatch_refusal_is_skipped_without_trying_a_retired_generation()
     {
        let cancel = AtomicBool::new(false);
        let mut owner = dom();
        owner
            .set_limits(
                owner.limits().with_event_recipients(
                    surgeist_dom::DomEventRecipientLimits::new(
                        64, 16_000_000, 16_000_000, 4_000_000,
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        let (mut session, document) = bound_session_from(owner, CssomContext::default());
        let target = append_style(&mut session, document, None);
        let mut context = Context {
            styles: session.style_execution(),
            source: session.source_execution(),
            events: Vec::new(),
        };
        let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 4,
            pending_events: 1,
            nested_dispatches: 2,
        })
        .unwrap();
        dispatcher
            .add_listener(
                &mut session.dom(),
                target,
                DomEventType::Load,
                DomEventCallback::new(|_, _, context: &mut Context, event| {
                    context.events.push((
                        event.target().unwrap(),
                        event.current_target().unwrap().node().unwrap(),
                    ));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let mut low = dispatch_limits();
        low.work = 0;
        assert!(matches!(
            DomStyleRuntime::run_networking(
                &mut session.dom(),
                &mut dispatcher,
                &mut context,
                low,
                &cancel
            ),
            Err(DomStyleRuntimeError::Dispatch(
                crate::DomEventDispatchError::Admission(crate::DomEventResource::Work)
            ))
        ));
        assert!(context.events.is_empty());
        assert!(matches!(
            context
                .source
                .apply(
                    &mut session.dom(),
                    DomSourceEdit::detach(target),
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let edit = DomSourceEdit::destroy(target);
        assert!(matches!(
            context
                .source
                .apply(
                    &mut session.dom(),
                    edit,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let next = DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut context,
            dispatch_limits(),
            &cancel,
        );
        assert!(
            matches!(next,Ok(DomStyleNetworkingProgress::TargetDestroyed {target:retired,..}) if retired==target),
            "not-started exact task must be retired, got {next:?}"
        );
        assert!(context.events.is_empty());
        assert!(context.styles.pending_sheet(0).unwrap().is_none());
    }
    #[test]
    fn active_cleanup_reservation_refuses_cap_growth_but_allows_nested_source_work_and_later_retry()
    {
        struct NestedContext {
            styles: DomStyleRuntime,
            source: DomSourceRuntime,
            observer: DomStyleRuntime,
            root: NodeId,
            parent: NodeId,
            refused: bool,
            safe_adjusted: bool,
            policy_observed: bool,
            nested: Option<NodeId>,
        }
        impl DomStyleEventContext for NestedContext {
            fn styles(&mut self) -> &mut DomStyleRuntime {
                &mut self.styles
            }
        }
        let cancel = AtomicBool::new(false);
        let (mut session, document) = bound_dispatch_session();
        let target = append_style(&mut session, document, None);
        let parent = session.snapshot().node(document).unwrap().children()[0];
        let mut context = NestedContext {
            styles: session.style_execution(),
            source: session.source_execution(),
            observer: session.style_execution(),
            root: document,
            parent,
            refused: false,
            safe_adjusted: false,
            policy_observed: false,
            nested: None,
        };
        let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 4,
            pending_events: 2,
            nested_dispatches: 2,
        })
        .unwrap();
        dispatcher
            .add_listener(
                &mut session.dom(),
                target,
                DomEventType::Load,
                DomEventCallback::new(|dom, _, context: &mut NestedContext, _| {
                    let cancel = AtomicBool::new(false);
                    let mut raised = limits();
                    raised.completions = 64;
                    raised.storage_bytes = 8_000_000;
                    raised.work = 200_000;
                    context.refused = matches!(
                        context.styles.set_limits(dom, raised, &cancel),
                        Err(DomStyleRuntimeError::Admission)
                    );
                    raised.completions = 32;
                    context.safe_adjusted = context.styles.set_limits(dom, raised, &cancel).is_ok();
                    let mut batch = DomBatch::new();
                    let nested = batch.create_element_ns(context.root, Namespace::html(), "style");
                    let setup = dom.apply(batch, &cancel).unwrap();
                    let nested = setup.created(&nested).unwrap().unwrap();
                    let edit = DomSourceEdit::insert_before(nested, context.parent, None);
                    let observer = &context.observer;
                    let seen = &mut context.policy_observed;
                    let root = context.root;
                    if let DomSourceDriveProgress::Complete(_) = context
                        .source
                        .apply(
                            dom,
                            edit,
                            source_limits(),
                            &mut DomSourceServices {
                                meta: None,
                                execution_scope: None,
                                style_policy: &mut |source| {
                                    *seen =
                                        observer.snapshot(root).is_ok_and(|value| value.is_some());
                                    policy(source)
                                },
                                url: None,
                            },
                            &cancel,
                        )
                        .unwrap()
                    {
                        context.nested = Some(nested);
                    }
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let result = DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut context,
            dispatch_limits(),
            &cancel,
        )
        .unwrap();
        let DomStyleNetworkingProgress::Complete { receipt, .. } = result else {
            panic!("outer actual task completes");
        };
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.failures().is_empty());
        assert!(context.refused);
        assert!(context.safe_adjusted);
        assert!(context.policy_observed);
        let nested = context.nested.expect("actual nested creation");
        assert!(
            session
                .snapshot()
                .node(nested)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet()
                .unwrap()
                .is_some()
        );
        assert!(context.styles.pending_sheet(0).unwrap().is_some());
        let mut raised = limits();
        raised.completions = 64;
        raised.storage_bytes = 8_000_000;
        raised.work = 200_000;
        context
            .styles
            .set_limits(&session.dom(), raised, &cancel)
            .unwrap();
    }

    #[test]
    fn retired_shadow_root_releases_current_binding_and_skips_original_queued_target_before_shutdown()
     {
        let cancel = AtomicBool::new(false);
        let mut owner = dom();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let host = batch.create_element_ns(&document, Namespace::html(), "div");
        let shadow = batch.attach_shadow(
            &host,
            surgeist_dom::DomShadowRootInit::new(surgeist_html::HtmlShadowRootMode::Open),
        );
        batch.insert_before(&host, &document, None);
        let done = owner.apply(batch, &cancel).unwrap();
        let document = done.created(&document).unwrap().unwrap();
        let host = done.created(&host).unwrap().unwrap();
        let shadow = done.created(&shadow).unwrap().unwrap();
        let mut session = DomSession::new(owner, session_limits(limits())).unwrap();
        let mut styles = session.style_execution();
        let mut source = session.source_execution();
        styles
            .bind_root(
                &session.dom(),
                "shadow".into(),
                &mut Some(DomStyleCssomRoot::new(
                    shadow,
                    CssomStore::new(Default::default(), Default::default()).unwrap(),
                )),
                &cancel,
            )
            .unwrap();
        let mut batch = DomBatch::new();
        let target = batch.create_element_ns(document, Namespace::html(), "style");
        let setup = session.dom().apply(batch, &cancel).unwrap();
        let target = setup.created(&target).unwrap().unwrap();
        let edit = DomSourceEdit::insert_before(target, shadow, None);
        let DomSourceDriveProgress::Complete(_) = source
            .apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap()
        else {
            panic!("actual creation completes");
        };
        let historical_dom = session.snapshot();
        let historical_cssom = styles.snapshot(shadow).unwrap().unwrap();
        let sheet = historical_cssom.sheets()[0].clone();
        assert!(matches!(
            source
                .apply(
                    &mut session.dom(),
                    DomSourceEdit::detach(host),
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let edit = DomSourceEdit::destroy(host);
        assert!(matches!(
            source
                .apply(
                    &mut session.dom(),
                    edit,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        assert!(styles.snapshot(shadow).unwrap().is_none());
        assert!(styles.pending_sheet(0).unwrap().is_some());
        let mut context = Context {
            styles,
            source,
            events: Vec::new(),
        };
        let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 4,
            pending_events: 2,
            nested_dispatches: 2,
        })
        .unwrap();
        assert!(
            matches!(DomStyleRuntime::run_networking(&mut session.dom(), &mut dispatcher,
            &mut context, dispatch_limits(), &cancel).unwrap(),
            DomStyleNetworkingProgress::TargetDestroyed { target: actual, .. } if actual == target)
        );
        assert!(context.styles.pending_sheet(0).unwrap().is_none());
        assert!(matches!(
            session.try_shutdown(),
            DomShutdownOutcome::Complete
        ));
        assert!(
            historical_dom
                .node(target)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet()
                .unwrap()
                .is_some()
        );
        assert!(historical_cssom.sheet(&sheet).is_ok());
        assert!(matches!(
            context.styles.waiting_resources(),
            Err(DomStyleRuntimeError::SessionUnavailable)
        ));
    }

    #[test]
    fn context_accessor_unwind_after_dispatch_preserves_delivered_receipt_and_clears_running_without_replay()
     {
        struct PanicContext {
            styles: DomStyleRuntime,
            calls: usize,
            panic_next: bool,
        }
        impl DomStyleEventContext for PanicContext {
            fn styles(&mut self) -> &mut DomStyleRuntime {
                if std::mem::take(&mut self.panic_next) {
                    panic!("actual context accessor unwind");
                }
                &mut self.styles
            }
        }
        let (mut session, document) = bound_dispatch_session();
        let target = append_style(&mut session, document, None);
        let mut context = PanicContext {
            styles: session.style_execution(),
            calls: 0,
            panic_next: false,
        };
        let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 4,
            pending_events: 1,
            nested_dispatches: 2,
        })
        .unwrap();
        dispatcher
            .add_listener(
                &mut session.dom(),
                target,
                DomEventType::Load,
                DomEventCallback::new(|_, _, context: &mut PanicContext, _| {
                    context.calls += 1;
                    context.panic_next = true;
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            DomStyleRuntime::run_networking(
                &mut session.dom(),
                &mut dispatcher,
                &mut context,
                dispatch_limits(),
                &AtomicBool::new(false),
            )
        }));
        assert!(failed.is_err());
        assert_eq!(context.calls, 1);
        let DomStyleNetworkingProgress::Complete { receipt, .. } = DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut context,
            dispatch_limits(),
            &AtomicBool::new(false),
        )
        .unwrap() else {
            panic!("actual retained delivery completes")
        };
        assert_eq!(receipt.callbacks(), 1);
        assert_eq!(context.calls, 1);
        assert!(receipt.failures().is_empty());
    }

    #[test]
    fn persistent_root_growth_admits_old_and_requested_bank_before_consuming_input() {
        let cancel = AtomicBool::new(false);
        let mut owner = dom();
        let mut batch = DomBatch::new();
        let documents = [0; 3].map(|_| batch.create_document());
        let done = owner.apply(batch, &cancel).unwrap();
        let documents = documents.map(|ticket| done.created(&ticket).unwrap().unwrap());
        let mut session = DomSession::new(owner, session_limits(limits())).unwrap();
        let mut permission = session.style_execution();
        for (document, label) in documents[..2].iter().zip(["a", "b"]) {
            permission
                .bind_root(
                    &session.dom(),
                    label.into(),
                    &mut Some(DomStyleCssomRoot::new(
                        *document,
                        CssomStore::new(Default::default(), Default::default()).unwrap(),
                    )),
                    &cancel,
                )
                .unwrap();
        }
        let original = permission.original().unwrap();
        let (current, capacity) = {
            let state = original.borrow();
            assert_eq!(state.roots.len(), state.roots.capacity());
            (
                state.storage(None, &cancel, limits().work).unwrap(),
                state.roots.capacity(),
            )
        };
        let label = String::from("c");
        // A new Document preference owns its empty Vec target in addition
        // to the root capability. Preserve the same one-short/exact envelope.
        let incoming = label.capacity() + size_of::<DomStyleCssomRoot>() + size_of::<Vec<u16>>();
        let replacement = (capacity + 1) * size_of::<Root>();
        let mut low = limits();
        low.storage_bytes = current + replacement + incoming - 1;
        permission.set_limits(&session.dom(), low, &cancel).unwrap();
        let mut input = Some(DomStyleCssomRoot::new(
            documents[2],
            CssomStore::new(Default::default(), Default::default()).unwrap(),
        ));
        let snapshot = input.as_ref().unwrap().snapshot().unwrap();
        assert!(
            matches!(
                permission.bind_root(&session.dom(), label.clone(), &mut input, &cancel),
                Err(DomStyleRuntimeError::Admission)
            ),
            "one-short replacement envelope must refuse before reserve"
        );
        assert_eq!(original.borrow().roots.capacity(), capacity);
        assert_eq!(original.borrow().roots.len(), 2);
        assert_eq!(
            input.as_ref().unwrap().snapshot().unwrap().revision(),
            snapshot.revision()
        );
        assert!(permission.snapshot(documents[2]).unwrap().is_none());
        let mut exact = low;
        exact.storage_bytes += 1;
        permission
            .set_limits(&session.dom(), exact, &cancel)
            .unwrap();
        permission
            .bind_root(&session.dom(), label, &mut input, &cancel)
            .unwrap();
        assert!(input.is_none());
        let actual = permission.snapshot(documents[2]).unwrap().unwrap();
        assert_eq!(actual.owner(), snapshot.owner());
        assert_eq!(actual.revision(), snapshot.revision());
        let state = original.borrow();
        assert_eq!(state.roots.len(), 3);
        assert!(state.storage(None, &cancel, exact.work).unwrap() <= exact.storage_bytes);
    }

    #[test]
    fn persistent_output_growth_admits_each_replacement_bank_without_issuing_a_task() {
        let cancel = AtomicBool::new(false);
        let (mut session, document) = bound_session();
        append_style(&mut session, document, None);
        append_style(&mut session, document, None);
        let original = session.style_execution().original().unwrap();
        let mut state = original.borrow_mut();
        assert_eq!(state.completions.len(), state.completions.capacity());
        assert_eq!(state.reports.len(), state.reports.capacity());
        let completion_capacity = state.completions.capacity();
        let report_capacity = state.reports.capacity();
        let task = state.next_task;
        let first = state.completions[0].task().id();
        let second = state.completions[1].task().id();
        let report = state.reports[0].clone();
        let current = state.storage(None, &cancel, limits().work).unwrap();
        // Completion reserve occurs first. At the report reserve, the new
        // completion bank is retained while the OLD report bank remains live.
        let completion_replacement = (completion_capacity + 1) * size_of::<Completion>();
        let report_replacement = (report_capacity + 1) * size_of::<Arc<DomStyleProcessedSheet>>();
        let peak = completion_replacement.max(size_of::<Completion>() + report_replacement);
        state.limits.storage_bytes = current + peak - 1;
        assert!(
            matches!(
                state.reserve_output(&cancel),
                Err(DomStyleRuntimeError::Admission)
            ),
            "one-short old-plus-requested bank must refuse before any reserve"
        );
        assert_eq!(state.completions.capacity(), completion_capacity);
        assert_eq!(state.reports.capacity(), report_capacity);
        assert_eq!(state.next_task, task);
        assert_eq!(state.completions[0].task().id(), first);
        assert_eq!(state.completions[1].task().id(), second);
        assert!(Arc::ptr_eq(&state.reports[0], &report));
        state.limits.storage_bytes += 1;
        state.reserve_output(&cancel).unwrap();
        assert_eq!(state.next_task, task);
        assert_eq!(state.completions.len(), 2);
        assert_eq!(state.reports.len(), 2);
        let expected = current
            + (state.completions.capacity() - completion_capacity) * size_of::<Completion>()
            + (state.reports.capacity() - report_capacity)
                * size_of::<Arc<DomStyleProcessedSheet>>();
        assert_eq!(
            state.storage(None, &cancel, limits().work).unwrap(),
            expected
        );
    }

    #[test]
    fn persistent_owner_attribute_report_refusal_keeps_same_source_and_cssom_owner_for_retry() {
        let cancel = AtomicBool::new(false);
        let (mut session, document) = bound_session();
        let target = append_style(&mut session, document, None);
        let mut permission = session.style_execution();
        let mut source = session.source_execution();
        let original = permission.original().unwrap();
        let before = permission.snapshot(document).unwrap().unwrap();
        let (current, capacity, task) = {
            let state = original.borrow();
            assert_eq!(state.reports.len(), state.reports.capacity());
            (
                state.storage(None, &cancel, limits().work).unwrap(),
                state.reports.capacity(),
                state.completions[0].task().id(),
            )
        };
        let mut low = limits();
        low.storage_bytes = current
            + (capacity + 1) * size_of::<Arc<DomStyleProcessedSheet>>()
            + size_of::<DomStyleProcessedSheet>()
            - 1;
        permission.set_limits(&session.dom(), low, &cancel).unwrap();
        let edit = DomSourceEdit::set_attribute(target, "title", "changed");
        assert!(
            matches!(
                source.apply(
                    &mut session.dom(),
                    edit,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                ),
                Err(crate::DomSourceDriveError::Source(
                    DomSourceRuntimeError::Style(DomStyleRuntimeError::Admission)
                ))
            ),
            "report replacement must be admitted before actual CSSOM owner edit"
        );
        let operation = session.dom().pending_source_edit().unwrap().operation();
        assert_eq!(
            permission.snapshot(document).unwrap().unwrap().revision(),
            before.revision()
        );
        {
            let state = original.borrow();
            assert_eq!(state.reports.capacity(), capacity);
            assert_eq!(state.reports.len(), 1);
            assert_eq!(state.completions.len(), 1);
            assert_eq!(state.completions[0].task().id(), task);
        }
        let mut exact = low;
        exact.storage_bytes += 1;
        permission
            .set_limits(&session.dom(), exact, &cancel)
            .unwrap();
        assert!(matches!(
            source
                .resume(
                    &mut session.dom(),
                    operation,
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None
                    },
                    &cancel
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
        let after = permission.snapshot(document).unwrap().unwrap();
        assert_eq!(after.owner(), before.owner());
        assert_ne!(after.revision(), before.revision());
        let state = original.borrow();
        assert_eq!(state.reports.len(), 2);
        assert_eq!(state.completions.len(), 1);
        assert_eq!(state.completions[0].task().id(), task);
        assert!(state.storage(None, &cancel, exact.work).unwrap() <= exact.storage_bytes);
    }

    #[test]
    fn persistent_root_admission_cancel_and_zero_work_preserve_input_without_growing() {
        for cancelled in [false, true] {
            let mut owner = dom();
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            let done = owner.apply(batch, &AtomicBool::new(false)).unwrap();
            let document = done.created(&document).unwrap().unwrap();
            let mut policy = limits();
            if !cancelled {
                policy.work = 0;
            }
            let mut session = DomSession::new(owner, session_limits(policy)).unwrap();
            let mut permission = session.style_execution();
            let mut input = Some(DomStyleCssomRoot::new(
                document,
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            ));
            let before = input.as_ref().unwrap().snapshot().unwrap();
            let result = permission.bind_root(
                &session.dom(),
                "root".into(),
                &mut input,
                &AtomicBool::new(cancelled),
            );
            if cancelled {
                assert!(matches!(
                    result,
                    Err(DomStyleRuntimeError::Dom(DomError::Cancelled))
                ));
            } else {
                assert!(matches!(result, Err(DomStyleRuntimeError::Admission)));
            }
            assert_eq!(
                input.as_ref().unwrap().snapshot().unwrap().revision(),
                before.revision()
            );
            let original = permission.original().unwrap();
            let state = original.borrow();
            assert_eq!(state.roots.capacity(), 0);
            assert!(state.roots.is_empty());
            assert_eq!(state.next_task, 1);
        }
    }
}

#[cfg(test)]
mod replacement_test {
    use super::tests::session_limits;
    use super::*;
    use crate::{DomEventCallback, DomEventListenerOptions, DomEventRegistryLimits, DomEventType};
    use surgeist_cssom::CssomStore;
    use surgeist_dom::{DomBatch, DomLimits, DomSourceEdit, DomSourceLimits, Namespace};
    struct Context {
        styles: DomStyleRuntime,
        source: DomSourceRuntime,
        calls: usize,
        replacement: Option<DomStyleRuntime>,
    }
    impl DomStyleEventContext for Context {
        fn styles(&mut self) -> &mut DomStyleRuntime {
            &mut self.styles
        }
    }
    fn limits() -> DomStyleRuntimeLimits {
        DomStyleRuntimeLimits {
            roots: 8,
            completions: 32,
            reports: 32,
            storage_bytes: 1_000_000,
            work: 100_000,
            css_input: CssUtf16InputLimits::default(),
        }
    }
    #[test]
    fn replacing_the_actual_style_owner_preserves_original_task_and_does_not_replay_event() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                64,
                10000,
                256,
                0,
                0,
                16,
                100000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_event_recipients(
                surgeist_dom::DomEventRecipientLimits::new(64, 16_000_000, 16_000_000, 4_000_000)
                    .unwrap(),
            )
            .with_dispatch_custody(
                surgeist_dom::DomDispatchCustodyLimits::new(
                    64, 4096, 16_000_000, 16_000_000, 4_000_000,
                )
                .unwrap(),
            ),
        )
        .unwrap();
        let mut b = DomBatch::new();
        let document = b.create_document();
        let c = dom.apply(b, &cancel).unwrap();
        let document = c.created(&document).unwrap().unwrap();
        let other_dom = Dom::new(
            DomLimits::new(
                64,
                10000,
                256,
                0,
                0,
                16,
                100000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let other_session = DomSession::new(other_dom, session_limits(limits())).unwrap();
        let mut session = DomSession::new(dom, session_limits(limits())).unwrap();
        let permission = session.style_execution();
        let source = session.source_execution();
        let mut dom = session.dom();
        let mut context = Context {
            styles: permission,
            source,
            calls: 0,
            replacement: Some(other_session.style_execution()),
        };
        context
            .styles
            .bind_root(
                &dom,
                "root".into(),
                &mut Some(DomStyleCssomRoot::new(
                    document,
                    CssomStore::new(Default::default(), Default::default()).unwrap(),
                )),
                &cancel,
            )
            .unwrap();
        let mut b = DomBatch::new();
        let style = b.create_element_ns(document, Namespace::html(), "style");
        let setup = dom.apply(b, &cancel).unwrap();
        let style = setup.created(&style).unwrap().unwrap();
        let edit = DomSourceEdit::insert_before(style, document, None);
        let DomSourceDriveProgress::Complete(_) = context
            .source
            .apply(
                &mut dom,
                edit,
                DomSourceLimits::new(16_000_000, 100000, 10_000_000, 64, 4096, 4096, 1),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |source| {
                        Some(DomStyleInlineCspObservation::complete_empty(source, 1))
                    },
                    url: None,
                },
                &cancel,
            )
            .unwrap()
        else {
            panic!("processed")
        };

        let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 8,
            pending_events: 8,
            nested_dispatches: 4,
        })
        .unwrap();
        dispatcher
            .add_listener(
                &mut dom,
                style,
                DomEventType::Load,
                DomEventCallback::new(|_, _, context: &mut Context, _| {
                    context.calls += 1;
                    // Drop the only public old owner handle. The active
                    // execution must return its actual retained owner itself.
                    drop(std::mem::replace(
                        &mut context.styles,
                        context.replacement.take().unwrap(),
                    ));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let dispatch = DomEventDispatchLimits {
            path_items: 64,
            snapshot_listeners: 64,
            failure_records: 16,
            callbacks: 64,
            work: 100000,
        };
        let first = DomStyleRuntime::run_networking(
            &mut dom,
            &mut dispatcher,
            &mut context,
            dispatch,
            &cancel,
        );
        let DomStyleNetworkingProgress::RetainedAccess { task: _, owner } = first.unwrap() else {
            panic!("substituted owner must return the original retained task owner");
        };
        context.styles = owner;
        assert!(
            matches!(
                DomStyleRuntime::run_networking(
                    &mut dom,
                    &mut dispatcher,
                    &mut context,
                    dispatch,
                    &cancel
                )
                .unwrap(),
                DomStyleNetworkingProgress::Complete { .. }
            ),
            "restored owner must resume its retained exact task"
        );
        assert_eq!(context.calls, 1, "delivered listener must never replay");
    }
}
