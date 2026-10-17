//! Original native hosted-lifecycle ownership within the durable DOM session.
//!
//! Native global and clock records describe the selected Rust execution domain;
//! they do not acknowledge ECMAScript realm construction or JS execution.
mod agent;
mod child;
mod clock;
mod destruction;
mod focus;
mod history;
mod policy;
mod records;
mod uuid;
mod watcher;
pub(crate) use child::SourceProgress as HostedSourceProgress;
pub use focus::DomSourceExecutionScope;

use crate::dom_event::DomEventDom;
use std::cell::RefCell;
use std::fmt;
use std::mem::size_of;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use surgeist_dom::{
    Dom, DomHostedCreationLease, DomHostedCreationPublication, DomHostedCreationStep,
    DomHostedEntryId, DomHostedEntryInitialization, DomHostedError, DomHostedGlobalId,
    DomHostedHostId, DomHostedLimits, DomHostedNavigableId, DomHostedPolicyPublication,
    DomHostedPreparedTopLevel, DomHostedSource, DomHostedTopLevelPreparation, DomOwnerId,
};

/// Finite native orchestration bounds, separate from canonical DOM admission.
#[derive(Clone, Copy, Debug)]
pub struct DomHostedRuntimeLimits {
    /// Canonical role, registry, operation and source-work admission.
    pub canonical: DomHostedLimits,
    /// Actual lifecycle/navigation task rows, including retained running work.
    /// Details owns its own DOM-manipulation task-source bank and count bound.
    pub tasks: usize,
    /// Actual native owner/record/task/output backing, including spare capacity.
    pub storage_bytes: usize,
    /// Simultaneously owned preparation/copy scratch and replacement-bank overlap.
    pub scratch_bytes: usize,
    /// Monotone native reads, comparisons, preparation and publication work.
    pub work: u64,
}

/// A refused native operation preserves its original owner and obligations.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DomHostedRuntimeError {
    /// The canonical source or operation could not be qualified.
    Dom(DomHostedError),
    /// This execution permission's original session was disposed.
    OwnerDisposed,
    /// The original owner is temporarily borrowed by another operation.
    Busy,
    /// The permission and canonical view belong to different original owners.
    Binding,
    /// Actual retained/scratch/count/work admission refused the operation.
    Admission,
    /// This target has no selected native monotonic clock provider.
    ClockUnavailable,
    /// The native monotonic clock regressed; no time observation was issued.
    ClockRegressed,
    /// The actual coarsened native time is not representable.
    ClockOverflow,
    /// Native entropy is unavailable on this target.
    EntropyUnavailable,
    /// The actual native entropy operation failed; no UUID was published.
    EntropyFailed,
    /// Cancellation preserves the original lifecycle phase.
    Cancelled,
    /// Another retained operation must be resumed first.
    OperationPending,
    /// The exact live native host/global/navigable was not found.
    TargetNotFound,
    /// A retained phase disagrees with the actual canonical request.
    Protocol,
    /// The queued original Document is not presently runnable.
    Inactive,
    /// Actual native checkpoint work remains unsupported or source-pending.
    CheckpointUnavailable,
    /// The reached URL stage has no available qualified observation.
    UrlUnavailable,
}
impl fmt::Display for DomHostedRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native hosted lifecycle refused: {self:?}")
    }
}
impl std::error::Error for DomHostedRuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            _ => None,
        }
    }
}

/// Immutable identities of an actually initialized no-opener top-level.
/// This observation carries no creation, task or teardown authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomHostedTopLevel {
    host: DomHostedHostId,
    context: surgeist_dom::DomHostedBrowsingContextId,
    navigable: DomHostedNavigableId,
    entry: DomHostedEntryId,
    global: DomHostedGlobalId,
    document: surgeist_dom::DocumentId,
}
impl DomHostedTopLevel {
    /// Original native host membership.
    pub const fn host(self) -> DomHostedHostId {
        self.host
    }
    /// Actual browsing-context indirection.
    pub const fn browsing_context(self) -> surgeist_dom::DomHostedBrowsingContextId {
        self.context
    }
    /// Actual active/current-entry navigable.
    pub const fn navigable(self) -> DomHostedNavigableId {
        self.navigable
    }
    /// The initial current and active session-history entry.
    pub const fn entry(self) -> DomHostedEntryId {
        self.entry
    }
    /// The actual native Global EventTarget.
    pub const fn global(self) -> DomHostedGlobalId {
        self.global
    }
    /// Actual canonically created initial about:blank Document.
    pub const fn document(self) -> surgeist_dom::DocumentId {
        self.document
    }
}
/// Observation that the actual named destruction invocation handed off its
/// queued work. It carries no task, execution, source or completion authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomHostedDestructionQueued {
    host: DomHostedHostId,
    navigable: DomHostedNavigableId,
    traversable: DomHostedNavigableId,
    operation: surgeist_dom::DomHostedOperationId,
}
impl DomHostedDestructionQueued {
    /// Original actual Host membership.
    pub const fn host(self) -> DomHostedHostId {
        self.host
    }
    /// Exact named target of this destruction invocation.
    pub const fn navigable(self) -> DomHostedNavigableId {
        self.navigable
    }
    /// Actual original traversal queue, independent from later unlinking.
    pub const fn traversable(self) -> DomHostedNavigableId {
        self.traversable
    }
    /// Canonical operation retaining this invocation's original Work.
    pub const fn operation(self) -> surgeist_dom::DomHostedOperationId {
        self.operation
    }
}

/// Actual native task body advanced by the hosted lifecycle owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostedTaskKind {
    /// The original Global's source-selected History update job.
    History,
    /// The original Global's lifecycle destruction continuation.
    Destruction,
}

/// A retained queue head cannot presently execute its exact next step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostedTaskWait {
    /// The shared native loop still owns another task or its checkpoint.
    CurrentTask,
    /// The actual task Document is inactive but has not been destroyed.
    Inactive,
    /// The invocation's captured descendant completion dependencies remain.
    Descendants,
    /// Actual native checkpoint work remains, including pending Slot signals.
    CheckpointUnavailable,
    /// A genuinely unfinished synchronous Source hook still owns its tail.
    SourceTail,
}

/// Observation of actual queued execution, distinct from Source edit completion.
/// These scalar observations retain no queue, task, cursor or mutable owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostedTaskProgress {
    /// This qualified traversable has no remaining queued execution obligation.
    Idle,
    /// The same original operation and queue head remain retained.
    Pending {
        /// Actual canonical lifecycle operation owning this continuation.
        operation: surgeist_dom::DomHostedOperationId,
        /// The precise reached wait, without acknowledging its work.
        reason: DomHostedTaskWait,
    },
    /// One actual native task body and its checkpoint completed.
    Executed {
        /// Original canonical lifecycle operation.
        operation: surgeist_dom::DomHostedOperationId,
        /// Original task Document, independent of later node adoption.
        document: surgeist_dom::DocumentId,
        /// Actual original native Global.
        global: DomHostedGlobalId,
        /// The body that actually ran.
        kind: DomHostedTaskKind,
    },
    /// A captured History obligation settled from actual same-generation
    /// Document destruction, without executing its body or writing History.
    CanceledDocumentDestroyed {
        /// Original canonical lifecycle operation.
        operation: surgeist_dom::DomHostedOperationId,
        /// Actual destroyed task Document.
        document: surgeist_dom::DocumentId,
        /// Original Global whose task was not executed.
        global: DomHostedGlobalId,
    },
    /// The original queued continuation finished all its actual obligations.
    Complete {
        /// Actual completed canonical lifecycle operation.
        operation: surgeist_dom::DomHostedOperationId,
    },
}

impl DomHostedTaskProgress {
    /// The actual original operation owning this outcome; an idle queue has no
    /// operation to acknowledge, resume or compare with a later FIFO task.
    pub const fn operation(self) -> Option<surgeist_dom::DomHostedOperationId> {
        match self {
            Self::Idle => None,
            Self::Pending { operation, .. }
            | Self::Executed { operation, .. }
            | Self::CanceledDocumentDestroyed { operation, .. }
            | Self::Complete { operation } => Some(operation),
        }
    }
}

/// Current associated-Document capture of the actual native settings object.
/// Later edits do not alter this historical observation or retain the issuer.
#[derive(Clone, Debug)]
pub struct DomHostedSettings {
    snapshot: surgeist_dom::DomSnapshot,
    global: DomHostedGlobalId,
    document: surgeist_dom::DocumentId,
    ready: bool,
    time_origin: clock::CoarsenedSample,
    creation_url: surgeist_html_semantics::HtmlBaseUrlRecord,
    top_level_creation_url: surgeist_html_semantics::HtmlBaseUrlRecord,
    top_level_origin: surgeist_dom::DomOpaqueOrigin,
    cross_site: bool,
}
impl DomHostedSettings {
    /// Native Global whose actual settings were read.
    pub fn global(&self) -> DomHostedGlobalId {
        self.global
    }
    /// Associated current Document at this exact captured canonical revision.
    /// Current base/URL/encoding queries consume this Document's ordinary
    /// qualified semantic facts; creation URL is not a current-base substitute.
    pub fn document(&self) -> surgeist_dom::DomDocument<'_> {
        self.snapshot
            .document(self.document)
            .expect("captured associated Document")
    }
    /// Current associated Document's actual origin at this captured revision.
    pub fn origin(&self) -> &surgeist_dom::DomOpaqueOrigin {
        self.document().metadata().origin()
    }
    /// Current effective base chosen by the original bounded Base owner.
    /// Missing/unfrozen source facts refuse rather than guessing from URL text.
    pub fn document_base_url(
        &self,
    ) -> Result<&surgeist_html::DocumentString, surgeist_dom::DomError> {
        self.document().document_base_url()
    }
    /// Canonical supported initial Permissions Policy from that current Document.
    pub fn permissions_policy(&self) -> Option<surgeist_dom::DomInitialPermissionsPolicy> {
        self.document().metadata().initial_permissions_policy()
    }
    /// Actual make-active readiness; never inferred from an ID or native proof.
    pub fn environment_ready(&self) -> bool {
        self.ready
    }
    /// Original native clock domain, shared across this session's Globals.
    pub fn clock_source(&self) -> surgeist_dom::DomHostedSourceId {
        self.time_origin.source
    }
    /// Actual navigation-start sample, floored to 100µs in that domain.
    pub fn time_origin_microseconds(&self) -> u64 {
        self.time_origin.microseconds
    }
    /// Actual URL record sampled when the original native Settings was created.
    pub fn creation_url(&self) -> &surgeist_html_semantics::HtmlBaseUrlRecord {
        &self.creation_url
    }
    /// Actual top-level creation URL sampled by the original constructor.
    pub fn top_level_creation_url(&self) -> &surgeist_html_semantics::HtmlBaseUrlRecord {
        &self.top_level_creation_url
    }
    /// Original top-level origin passed to this actual Settings constructor.
    /// This does not change when the top traversable's active entry changes.
    pub fn top_level_origin(&self) -> &surgeist_dom::DomOpaqueOrigin {
        &self.top_level_origin
    }
    /// Group mode none does not provide cross-origin isolation.
    pub const fn cross_origin_isolated(&self) -> bool {
        false
    }
    /// Cross-site ancestry computed from this capture's actual canonical
    /// parent active-Document chain, not a Global's creation-time mirror.
    pub const fn cross_site(&self) -> bool {
        self.cross_site
    }
}

/// Immutable observation of the actual current initial session-history entry.
/// Its copied strings/identities confer no task or lifecycle authority.
#[derive(Clone, Debug)]
pub struct DomHostedHistoryEntry {
    entry: DomHostedEntryId,
    document: surgeist_dom::DocumentId,
    origin: surgeist_dom::DomOpaqueOrigin,
    key: [u16; 36],
    id: [u16; 36],
    step: Option<u64>,
}
impl DomHostedHistoryEntry {
    /// Exact canonical history-entry identity.
    pub const fn entry(&self) -> DomHostedEntryId {
        self.entry
    }
    /// Actual associated initial Document.
    pub const fn document(&self) -> surgeist_dom::DocumentId {
        self.document
    }
    /// Actual source-created opaque origin retained by the document state.
    pub fn origin(&self) -> &surgeist_dom::DomOpaqueOrigin {
        &self.origin
    }
    /// Exact independently generated navigation API key, in UTF-16 code units.
    pub fn key(&self) -> &[u16; 36] {
        &self.key
    }
    /// Exact independently generated navigation API ID, in UTF-16 code units.
    pub fn id(&self) -> &[u16; 36] {
        &self.id
    }
    /// Actual assigned step; None is the initial entry's source-defined Pending.
    pub const fn step(&self) -> Option<u64> {
        self.step
    }
}

pub(crate) struct HostedOwner {
    state: Rc<RefCell<HostedState>>,
}
struct HostedState {
    dom: DomOwnerId,
    source: DomHostedSource,
    clock: clock::Clock,
    limits: DomHostedRuntimeLimits,
    hosts: Vec<records::Host>,
    focus: focus::FocusState,
    work: u64,
    creation: Option<Creation>,
    children: Vec<child::Creation>,
    destruction: destruction::State,
    element_tasks: Option<crate::element_tasks::DomElementTaskRuntime>,
}
impl HostedOwner {
    pub(crate) fn admit_initial_headers(
        limits: DomHostedRuntimeLimits,
    ) -> Result<(), DomHostedRuntimeError> {
        let bytes = std::mem::size_of::<RefCell<HostedState>>();
        if bytes > limits.storage_bytes {
            return Err(DomHostedRuntimeError::Admission);
        }
        Ok(())
    }
    pub(crate) fn new(
        dom: &mut Dom,
        limits: DomHostedRuntimeLimits,
    ) -> Result<Self, DomHostedRuntimeError> {
        Self::admit_initial_headers(limits)?;
        let source = dom
            .register_hosted_source(limits.canonical, &AtomicBool::new(false))
            .map_err(DomHostedRuntimeError::Dom)?;
        let clock = clock::Clock::new(source.id());
        let focus_header = source
            .retain_runtime_payload(
                dom,
                focus::FocusState::header_bytes(),
                &AtomicBool::new(false),
            )
            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        Ok(Self {
            state: Rc::new(RefCell::new(HostedState {
                dom: dom.owner(),
                source,
                clock,
                limits,
                hosts: Vec::new(),
                focus: focus::FocusState::new(focus_header),
                work: 0,
                creation: None,
                children: Vec::new(),
                destruction: destruction::State::new(),
                element_tasks: None,
            })),
        })
    }
    pub(crate) fn execution(&self) -> HostedRuntime {
        HostedRuntime {
            state: Rc::downgrade(&self.state),
        }
    }
}

/// Weak execution permission for the original session's native lifecycle owner.
/// It cannot extract the canonical DOM, source issuer, loop or resource custody.
#[derive(Clone)]
pub struct HostedRuntime {
    state: Weak<RefCell<HostedState>>,
}
impl fmt::Debug for HostedRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HostedRuntime").finish_non_exhaustive()
    }
}

// These are original partially established values, not a second phase machine:
// the canonical lease alone selects the next source phase. Every successful
// external operation is retained here before any subsequent fallible step.
struct Creation {
    host: DomHostedHostId,
    composition: usize,
    preparation: Option<DomHostedTopLevelPreparation>,
    prepared: Option<DomHostedPreparedTopLevel>,
    context: Option<records::BrowsingContext>,
    sample: Option<clock::Sample>,
    lease: Option<DomHostedCreationLease>,
    policy: Option<DomHostedPolicyPublication>,
    publication: Option<DomHostedCreationPublication>,
    global: Option<DomHostedGlobalId>,
    entry: Option<DomHostedEntryId>,
    navigable: Option<DomHostedNavigableId>,
    uuids: Option<uuid::HistoryUuids>,
    entry_initialization: Option<DomHostedEntryInitialization>,
    finished_sample: Option<clock::CoarsenedSample>,
}
impl Creation {
    fn new(host: DomHostedHostId, composition: usize) -> Self {
        Self {
            host,
            composition,
            preparation: None,
            prepared: None,
            context: None,
            sample: None,
            lease: None,
            policy: None,
            publication: None,
            global: None,
            entry: None,
            navigable: None,
            uuids: None,
            entry_initialization: None,
            finished_sample: None,
        }
    }
}

impl HostedState {
    fn step(&mut self, cancel: &AtomicBool) -> Result<(), DomHostedRuntimeError> {
        spend(&mut self.work, self.limits.work, cancel)
    }
    fn qualify(&self, dom: &DomEventDom<'_>) -> Result<(), DomHostedRuntimeError> {
        if self.dom != dom.owner() {
            return Err(DomHostedRuntimeError::Binding);
        }
        Ok(())
    }
    fn retained_bytes(&mut self, cancel: &AtomicBool) -> Result<usize, DomHostedRuntimeError> {
        native_retained_bytes(
            &self.hosts,
            &self.children,
            &self.destruction,
            &mut self.work,
            self.limits.work,
            cancel,
            || Ok(()),
        )
    }
    fn source_retained_bytes(
        &mut self,
        lease: &surgeist_dom::DomSourceLease<'_>,
        cancel: &AtomicBool,
    ) -> Result<usize, DomHostedRuntimeError> {
        native_retained_bytes(
            &self.hosts,
            &self.children,
            &self.destruction,
            &mut self.work,
            self.limits.work,
            cancel,
            || {
                lease
                    .admit_hosted_runtime_work(1, cancel)
                    .map_err(|error| DomHostedRuntimeError::Dom(error.into()))
            },
        )
    }
    fn reserve_host(
        &mut self,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        self.step(cancel)?;
        self.source
            .admit_runtime_work(dom.raw(), 1, cancel)
            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        if self.hosts.len() >= self.limits.canonical.records().hosts() {
            return Err(DomHostedRuntimeError::Admission);
        }
        if self.hosts.len() < self.hosts.capacity() {
            return Ok(());
        }
        let count = self
            .hosts
            .len()
            .checked_add(1)
            .ok_or(DomHostedRuntimeError::Admission)?;
        let old = native_retained_bytes(
            &self.hosts,
            &self.children,
            &self.destruction,
            &mut self.work,
            self.limits.work,
            cancel,
            || {
                self.source
                    .admit_runtime_work(dom.raw(), 1, cancel)
                    .map_err(|error| DomHostedRuntimeError::Dom(error.into()))
            },
        )?;
        let requested = count
            .checked_mul(size_of::<records::Host>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if requested > self.limits.scratch_bytes
            || old
                .checked_add(requested)
                .is_none_or(|bytes| bytes > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        let mut guard = self
            .source
            .retain_runtime_payload(dom.raw_mut(), requested, cancel)
            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        // Prepay every actual move before replacement publication.
        for _ in 0..self.hosts.len() {
            self.step(cancel)?;
            self.source
                .admit_runtime_work(dom.raw(), 1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        }
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(count)
            .map_err(|_| DomHostedRuntimeError::Admission)?;
        let actual = replacement
            .capacity()
            .checked_mul(size_of::<records::Host>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if actual > self.limits.scratch_bytes
            || old
                .checked_add(actual)
                .is_none_or(|bytes| bytes > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        if actual > requested {
            guard
                .absorb(
                    self.source
                        .retain_runtime_payload(dom.raw_mut(), actual - requested, cancel)
                        .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?,
                )
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        }
        replacement.append(&mut self.hosts);
        self.hosts = replacement;
        self.focus.host_backing = Some(guard);
        Ok(())
    }
}
impl HostedState {
    fn reserve_composition(
        &mut self,
        index: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        self.step(cancel)?;
        let bank = &self.hosts[index].top_levels;
        if bank.len() < bank.capacity() {
            return Ok(());
        }
        let required = bank
            .len()
            .checked_add(1)
            .ok_or(DomHostedRuntimeError::Admission)?;
        let old = self.retained_bytes(cancel)?;
        let requested = required
            .checked_mul(size_of::<records::Composition>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if requested > self.limits.scratch_bytes
            || old
                .checked_add(requested)
                .is_none_or(|n| n > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        for _ in 0..required {
            self.step(cancel)?;
        }
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(required)
            .map_err(|_| DomHostedRuntimeError::Admission)?;
        let actual = replacement
            .capacity()
            .checked_mul(size_of::<records::Composition>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if actual > self.limits.scratch_bytes
            || old
                .checked_add(actual)
                .is_none_or(|n| n > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        replacement.append(&mut self.hosts[index].top_levels);
        self.hosts[index].top_levels = replacement;
        Ok(())
    }
}
impl HostedState {
    fn find_global(
        &mut self,
        global: DomHostedGlobalId,
        cancel: &AtomicBool,
        mut admit: impl FnMut() -> Result<(), DomHostedRuntimeError>,
    ) -> Result<(usize, usize, usize, usize), DomHostedRuntimeError> {
        for h in 0..self.hosts.len() {
            self.step(cancel)?;
            admit()?;
            for t in 0..self.hosts[h].top_levels.len() {
                self.step(cancel)?;
                admit()?;
                let clusters = self.hosts[h].top_levels[t]
                    .group
                    .as_ref()
                    .map_or(0, |group| group.clusters.len());
                for c in 0..clusters {
                    self.step(cancel)?;
                    admit()?;
                    let count = self.hosts[h].top_levels[t]
                        .group
                        .as_ref()
                        .ok_or(DomHostedRuntimeError::Protocol)?
                        .clusters[c]
                        .agent
                        .globals
                        .len();
                    for g in 0..count {
                        self.step(cancel)?;
                        admit()?;
                        let group = self.hosts[h].top_levels[t]
                            .group
                            .as_ref()
                            .ok_or(DomHostedRuntimeError::Protocol)?;
                        let record = &group.clusters[c].agent.globals[g];
                        if record.id == global {
                            if record.context.group_root != group.context
                                || record.context.global != Some(global)
                            {
                                return Err(DomHostedRuntimeError::Protocol);
                            }
                            return Ok((h, t, c, g));
                        }
                    }
                }
            }
        }
        Err(DomHostedRuntimeError::TargetNotFound)
    }
}

impl HostedRuntime {
    pub(crate) fn validate_view(&self, dom: &DomEventDom<'_>) -> Result<(), DomHostedRuntimeError> {
        let state = self.state()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)
    }

    fn state(&self) -> Result<Rc<RefCell<HostedState>>, DomHostedRuntimeError> {
        self.state
            .upgrade()
            .ok_or(DomHostedRuntimeError::OwnerDisposed)
    }
    /// Create the actual native host record before top-level construction.
    /// The canonical issuer is retained only in the original durable session.
    pub fn create_host(
        &mut self,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedHostId, DomHostedRuntimeError> {
        self.create_host_with_focus(dom, focus::HostFocusDisposition::ExternalUnobserved, cancel)
    }
    /// Create a sealed native Host whose complete lifetime excludes content
    /// system focus, including every future admitted top-level and child.
    pub fn create_headless_host(
        &mut self,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedHostId, DomHostedRuntimeError> {
        self.create_host_with_focus(
            dom,
            focus::HostFocusDisposition::HeadlessNoContentSystemFocus,
            cancel,
        )
    }
    fn create_host_with_focus(
        &mut self,
        dom: &mut DomEventDom<'_>,
        focus: focus::HostFocusDisposition,
        cancel: &AtomicBool,
    ) -> Result<DomHostedHostId, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        if state.creation.is_some() {
            return Err(DomHostedRuntimeError::OperationPending);
        }
        state.reserve_host(dom, cancel)?;
        let id = state
            .source
            .issue_host(cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        // Header/capacity/move work was admitted before actual identity issuance.
        state.hosts.push(records::Host {
            id,
            top_levels: Vec::new(),
            focus,
        });
        Ok(id)
    }
}

impl HostedRuntime {
    /// Read actual settings joined to the current canonical Document.
    pub fn settings(
        &self,
        dom: &DomEventDom<'_>,
        global: DomHostedGlobalId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedSettings, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        let snapshot = dom.snapshot();
        let document = dom
            .raw()
            .global_document(global)
            .map_err(DomHostedRuntimeError::Dom)?;
        let cross_site = state.capture_cross_site(dom, document, cancel)?;
        let (h, t, c, g) = state.find_global(global, cancel, || Ok(()))?;
        let native = &state.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .clusters[c]
            .agent
            .globals[g];
        if native.settings.document != Some(document) {
            return Err(DomHostedRuntimeError::Protocol);
        }
        Ok(DomHostedSettings {
            snapshot,
            global,
            document,
            ready: native.settings.ready,
            time_origin: native.settings.navigation_start,
            creation_url: native.settings.creation_url.clone(),
            top_level_creation_url: native.settings.top_level_creation_url.clone(),
            top_level_origin: native.settings.top_level_origin.clone(),
            cross_site,
        })
    }
    /// Read the live original active/current-entry relation.
    pub fn top_level(
        &self,
        dom: &DomEventDom<'_>,
        navigable: DomHostedNavigableId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedTopLevel, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        for h in 0..state.hosts.len() {
            state.step(cancel)?;
            for t in 0..state.hosts[h].top_levels.len() {
                state.step(cancel)?;
                let host = &state.hosts[h];
                let record = &host.top_levels[t];
                let Some(top) = &record.top else { continue };
                if top.id != navigable {
                    continue;
                }
                let group = record
                    .group
                    .as_ref()
                    .ok_or(DomHostedRuntimeError::Protocol)?;
                let native = group.global().ok_or(DomHostedRuntimeError::Protocol)?;
                let canonical = dom
                    .raw()
                    .hosted_navigable(navigable)
                    .map_err(DomHostedRuntimeError::Dom)?;
                let document = canonical
                    .active_document()
                    .ok_or(DomHostedRuntimeError::Inactive)?;
                if canonical.parent().is_some()
                    || dom
                        .raw()
                        .global_document(native.id)
                        .map_err(DomHostedRuntimeError::Dom)?
                        != document
                {
                    return Err(DomHostedRuntimeError::Protocol);
                }
                return Ok(DomHostedTopLevel {
                    host: host.id,
                    context: group.context,
                    navigable,
                    entry: canonical.current_entry(),
                    global: native.id,
                    document,
                });
            }
        }
        Err(DomHostedRuntimeError::TargetNotFound)
    }
}
fn native_task_count(
    hosts: &[records::Host],
    work: &mut u64,
    limit: u64,
    cancel: &AtomicBool,
) -> Result<usize, DomHostedRuntimeError> {
    let mut count = 0usize;
    for host in hosts {
        spend(work, limit, cancel)?;
        for record in &host.top_levels {
            spend(work, limit, cancel)?;
            if let Some(group) = &record.group {
                spend(work, limit, cancel)?;
                count = count
                    .checked_add(group.traversal.rows.len())
                    .ok_or(DomHostedRuntimeError::Admission)?;
                for cluster in &group.clusters {
                    spend(work, limit, cancel)?;
                    count = count
                        .checked_add(
                            cluster
                                .agent
                                .event_loop
                                .try_borrow()
                                .map_err(|_| DomHostedRuntimeError::Busy)?
                                .navigation
                                .len(),
                        )
                        .ok_or(DomHostedRuntimeError::Admission)?;
                }
            }
        }
    }
    Ok(count)
}

fn native_retained_bytes(
    hosts: &Vec<records::Host>,
    children: &Vec<child::Creation>,
    destruction: &destruction::State,
    work: &mut u64,
    limit: u64,
    cancel: &AtomicBool,
    mut admit: impl FnMut() -> Result<(), DomHostedRuntimeError>,
) -> Result<usize, DomHostedRuntimeError> {
    let mut bytes = size_of::<RefCell<HostedState>>()
        .checked_add(
            hosts
                .capacity()
                .checked_mul(size_of::<records::Host>())
                .ok_or(DomHostedRuntimeError::Admission)?,
        )
        .ok_or(DomHostedRuntimeError::Admission)?;
    bytes = bytes
        .checked_add(
            children
                .capacity()
                .checked_mul(size_of::<child::Creation>())
                .ok_or(DomHostedRuntimeError::Admission)?,
        )
        .ok_or(DomHostedRuntimeError::Admission)?;
    bytes = bytes
        .checked_add(destruction.retained_bytes(work, limit, cancel, &mut admit)?)
        .ok_or(DomHostedRuntimeError::Admission)?;
    for host in hosts {
        spend(work, limit, cancel)?;
        admit()?;
        bytes = bytes
            .checked_add(
                host.top_levels
                    .capacity()
                    .checked_mul(size_of::<records::Composition>())
                    .ok_or(DomHostedRuntimeError::Admission)?,
            )
            .ok_or(DomHostedRuntimeError::Admission)?;
        for record in &host.top_levels {
            spend(work, limit, cancel)?;
            admit()?;
            if let Some(group) = &record.group {
                bytes = bytes
                    .checked_add(
                        group
                            .completions
                            .capacity()
                            .checked_mul(size_of::<destruction::Completion>())
                            .ok_or(DomHostedRuntimeError::Admission)?,
                    )
                    .ok_or(DomHostedRuntimeError::Admission)?;
                bytes = bytes
                    .checked_add(
                        group
                            .traversal
                            .rows
                            .capacity()
                            .checked_mul(size_of::<history::Queued>())
                            .ok_or(DomHostedRuntimeError::Admission)?,
                    )
                    .ok_or(DomHostedRuntimeError::Admission)?;
                bytes = bytes
                    .checked_add(
                        group
                            .clusters
                            .capacity()
                            .checked_mul(size_of::<records::OriginCluster>())
                            .ok_or(DomHostedRuntimeError::Admission)?,
                    )
                    .ok_or(DomHostedRuntimeError::Admission)?;
                for cluster in &group.clusters {
                    spend(work, limit, cancel)?;
                    admit()?;
                    bytes = bytes
                        .checked_add(
                            cluster
                                .agent
                                .globals
                                .capacity()
                                .checked_mul(size_of::<records::Global>())
                                .ok_or(DomHostedRuntimeError::Admission)?,
                        )
                        .ok_or(DomHostedRuntimeError::Admission)?;
                    bytes = bytes
                        .checked_add(
                            cluster
                                .agent
                                .windows
                                .rows
                                .capacity()
                                .checked_mul(size_of::<watcher::NativeWindow>())
                                .ok_or(DomHostedRuntimeError::Admission)?,
                        )
                        .ok_or(DomHostedRuntimeError::Admission)?;
                    for window in &cluster.agent.windows.rows {
                        spend(work, limit, cancel)?;
                        admit()?;
                        bytes = bytes
                            .checked_add(window.retained_bytes(work, limit, cancel, &mut admit)?)
                            .ok_or(DomHostedRuntimeError::Admission)?;
                    }
                    for global in &cluster.agent.globals {
                        spend(work, limit, cancel)?;
                        admit()?;
                        spend(work, limit, cancel)?;
                        admit()?;
                        bytes = bytes
                            .checked_add(settings_url_payload_bytes(
                                &global.settings.creation_url,
                                &global.settings.top_level_creation_url,
                            )?)
                            .ok_or(DomHostedRuntimeError::Admission)?;
                    }
                    let event_loop = cluster
                        .agent
                        .event_loop
                        .try_borrow()
                        .map_err(|_| DomHostedRuntimeError::Busy)?;
                    bytes = bytes
                        .checked_add(size_of::<RefCell<records::WindowLoop>>())
                        .and_then(|n| {
                            n.checked_add(
                                event_loop
                                    .navigation
                                    .capacity()
                                    .checked_mul(size_of::<records::NavigationTask>())?,
                            )
                        })
                        .ok_or(DomHostedRuntimeError::Admission)?;
                }
            }
        }
    }
    Ok(bytes)
}

// Each retained Settings field is one declared payload occurrence, including
// when both source inputs identify the same URL record. Inline handles already
// live in the admitted Composition bank. Producer receipts own private layout.
fn settings_url_payload_bytes(
    creation: &surgeist_html_semantics::HtmlBaseUrlRecord,
    top_level: &surgeist_html_semantics::HtmlBaseUrlRecord,
) -> Result<usize, DomHostedRuntimeError> {
    creation
        .retained_storage_usage()
        .retained_bytes()
        .checked_add(top_level.retained_storage_usage().retained_bytes())
        .ok_or(DomHostedRuntimeError::Admission)
}

fn spend(work: &mut u64, limit: u64, cancel: &AtomicBool) -> Result<(), DomHostedRuntimeError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(DomHostedRuntimeError::Cancelled);
    }
    let next = work
        .checked_add(1)
        .ok_or(DomHostedRuntimeError::Admission)?;
    if next > limit {
        return Err(DomHostedRuntimeError::Admission);
    }
    *work = next;
    Ok(())
}
fn clock_error(error: clock::Error) -> DomHostedRuntimeError {
    match error {
        clock::Error::Unavailable => DomHostedRuntimeError::ClockUnavailable,
        clock::Error::Regressed => DomHostedRuntimeError::ClockRegressed,
        clock::Error::Overflow => DomHostedRuntimeError::ClockOverflow,
    }
}

// No owner is detached while driving. On every error (including unwinding),
// already established values stay in the original in-place Creation/Host.
struct CreationInput<'a> {
    source: &'a mut DomHostedSource,
    clock: &'a mut clock::Clock,
    hosts: &'a mut [records::Host],
    work: &'a mut u64,
    limits: DomHostedRuntimeLimits,
    retained_before: usize,
}
fn drive_creation(
    input: CreationInput<'_>,
    creation: &mut Creation,
    dom: &mut DomEventDom<'_>,
    cancel: &AtomicBool,
) -> Result<DomHostedTopLevel, DomHostedRuntimeError> {
    let CreationInput {
        source,
        clock,
        hosts,
        work,
        limits,
        retained_before,
    } = input;
    let limit = limits.work;
    let storage_limit = limits.storage_bytes;
    let mut retained_added = 0usize;
    let mut index = None;
    for (candidate, host) in hosts.iter().enumerate() {
        spend(work, limit, cancel)?;
        if host.id == creation.host {
            index = Some(candidate);
            break;
        }
    }
    let host = &mut hosts[index.ok_or(DomHostedRuntimeError::TargetNotFound)?];
    let host_id = host.id;
    let host = host
        .top_levels
        .get_mut(creation.composition)
        .ok_or(DomHostedRuntimeError::Protocol)?;
    if creation.lease.is_none() {
        if creation.preparation.is_none() && creation.prepared.is_none() {
            spend(work, limit, cancel)?;
            let preparation = source
                .prepare_top_level(host_id, cancel)
                .map_err(DomHostedRuntimeError::Dom)?;
            // This is the actual no-opener group and BC construction. Their
            // inline owner storage was admitted before host identity issuance.
            host.group = Some(records::Group {
                context: preparation.browsing_context(),
                clusters: Vec::new(),
                traversal: history::Traversal::new(),
                completions: Vec::new(),
            });
            creation.context = Some(records::BrowsingContext {
                id: preparation.browsing_context(),
                group_root: preparation.browsing_context(),
                creator: None,
                global: None,
            });
            creation.preparation = Some(preparation);
        }
        if creation.sample.is_none() {
            spend(work, limit, cancel)?;
            creation.sample = Some(clock.sample().map_err(clock_error)?);
        }
        if creation.prepared.is_none() {
            creation.prepared = Some(
                source
                    .seal_top_level(&mut creation.preparation, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?,
            );
        }
        let lease = dom
            .raw_mut()
            .begin_hosted_top_level(source, &mut creation.prepared, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        creation.lease = Some(lease);
    }
    loop {
        spend(work, limit, cancel)?;
        let lease = creation
            .lease
            .as_mut()
            .expect("retained original creation lease");
        // Read only the reached phase's origin, then end that canonical loan
        // before preparing real Runtime banks through current Source admission.
        let origin = match lease
            .step(dom.raw_mut())
            .map_err(DomHostedRuntimeError::Dom)?
        {
            DomHostedCreationStep::NativeExecution(request) => Some(request.origin().clone()),
            _ => None,
        };
        if let Some(origin) = origin {
            let group = host.group.as_mut().ok_or(DomHostedRuntimeError::Protocol)?;
            let (cluster, added) = agent::ensure_cluster(
                group,
                &origin,
                retained_before
                    .checked_add(retained_added)
                    .ok_or(DomHostedRuntimeError::Admission)?,
                limits,
                work,
                cancel,
                |bytes| source.retain_runtime_payload(dom.raw_mut(), bytes, cancel),
            )?;
            retained_added = retained_added
                .checked_add(added)
                .ok_or(DomHostedRuntimeError::Admission)?;
            let added = watcher::reserve_window(
                &mut group.clusters[cluster].agent,
                retained_before
                    .checked_add(retained_added)
                    .ok_or(DomHostedRuntimeError::Admission)?,
                limits,
                work,
                |bytes| source.retain_runtime_payload(dom.raw_mut(), bytes, cancel),
                cancel,
            )?;
            retained_added = retained_added
                .checked_add(added)
                .ok_or(DomHostedRuntimeError::Admission)?;
        }
        let policy_phase;
        match lease
            .step(dom.raw_mut())
            .map_err(DomHostedRuntimeError::Dom)?
        {
            DomHostedCreationStep::PermissionsPolicy(request) => {
                policy_phase = true;
                if creation.policy.is_none() {
                    // Null container reaches PP's first source branch: enabled.
                    // No parent, declared map or URL parser is consulted here.
                    if request.parent_document().is_some() || request.embedder().is_some() {
                        return Err(DomHostedRuntimeError::Protocol);
                    }
                    creation.policy = Some(
                        source
                            .seal_permissions_policy(
                                request,
                                surgeist_dom::DomInitialPermissionsPolicy::new(
                                    surgeist_dom::DomPermissionsPolicyValue::Enabled,
                                    surgeist_dom::DomPermissionsPolicyValue::Enabled,
                                ),
                                cancel,
                            )
                            .map_err(DomHostedRuntimeError::Dom)?,
                    );
                }
            }
            DomHostedCreationStep::NativeExecution(request) => {
                policy_phase = false;
                let group = host.group.as_mut().ok_or(DomHostedRuntimeError::Protocol)?;
                if group.context != request.browsing_context() {
                    return Err(DomHostedRuntimeError::Protocol);
                }
                let cluster_index = agent::find_cluster(
                    group,
                    request.origin(),
                    work,
                    limits.work,
                    cancel,
                    || Ok(()),
                )?
                .ok_or(DomHostedRuntimeError::Protocol)?;
                let sample = creation
                    .sample
                    .as_ref()
                    .ok_or(DomHostedRuntimeError::Protocol)?
                    .coarsen()
                    .map_err(clock_error)?;
                if sample.source != source.id() {
                    return Err(DomHostedRuntimeError::Binding);
                }
                if creation.global.is_none() {
                    creation.global = Some(
                        source
                            .issue_global(request, cancel)
                            .map_err(DomHostedRuntimeError::Dom)?,
                    );
                }
                let global = creation.global.expect("original issued Global");
                let agent = &mut group.clusters[cluster_index].agent;
                let mut installed = false;
                for value in &agent.globals {
                    spend(work, limit, cancel)?;
                    if value.id == global {
                        installed = true;
                        break;
                    }
                }
                if !installed {
                    let bank_added = agent::reserve_global(
                        agent,
                        retained_before
                            .checked_add(retained_added)
                            .ok_or(DomHostedRuntimeError::Admission)?,
                        limits,
                        work,
                        cancel,
                    )?;
                    retained_added = retained_added
                        .checked_add(bank_added)
                        .ok_or(DomHostedRuntimeError::Admission)?;
                    spend(work, limit, cancel)?;
                    spend(work, limit, cancel)?;
                    let payload = settings_url_payload_bytes(
                        request.creation_url(),
                        request.top_level_creation_url(),
                    )?;
                    if retained_before
                        .checked_add(retained_added)
                        .and_then(|bytes| bytes.checked_add(payload))
                        .is_none_or(|bytes| bytes > storage_limit)
                    {
                        return Err(DomHostedRuntimeError::Admission);
                    }
                    let context = creation
                        .context
                        .as_ref()
                        .ok_or(DomHostedRuntimeError::Protocol)?;
                    if context.id != request.browsing_context()
                        || context.creator.is_some()
                        || context.group_root != group.context
                    {
                        return Err(DomHostedRuntimeError::Protocol);
                    }
                    let mut context = creation.context.take().expect("validated original BC");
                    context.global = Some(global);
                    agent
                        .windows
                        .rows
                        .push(watcher::NativeWindow::fresh(global));
                    agent.globals.push(records::Global {
                        context,
                        id: global,
                        settings: records::Settings {
                            navigation_start: sample,
                            document: None,
                            ready: false,
                            creation_url: request.creation_url().clone(),
                            top_level_creation_url: request.top_level_creation_url().clone(),
                            top_level_origin: request.top_level_origin().clone(),
                        },
                    });
                }
                if creation.publication.is_none() {
                    creation.publication = Some(
                        source
                            .seal_native_execution(request, global, sample.microseconds, cancel)
                            .map_err(DomHostedRuntimeError::Dom)?,
                    );
                }
            }
            DomHostedCreationStep::MakeActive(request) => {
                policy_phase = false;
                let native = host
                    .group
                    .as_mut()
                    .and_then(records::Group::global_mut)
                    .ok_or(DomHostedRuntimeError::Protocol)?;
                if native.id != request.global() {
                    return Err(DomHostedRuntimeError::Protocol);
                }
                // Actual settings/BC indirection now select this Document;
                // readiness is set only after canonical Document creation.
                native.settings.document = Some(request.document());
                native.settings.ready = true;
                if creation.publication.is_none() {
                    creation.publication = Some(
                        source
                            .seal_make_active(request, cancel)
                            .map_err(DomHostedRuntimeError::Dom)?,
                    );
                }
            }
            DomHostedCreationStep::FinishLoading(request) => {
                policy_phase = false;
                let native = host
                    .group
                    .as_ref()
                    .and_then(records::Group::global)
                    .ok_or(DomHostedRuntimeError::Protocol)?;
                let original = native.settings.navigation_start;
                if native.id != request.global()
                    || native.settings.document != Some(request.document())
                    || original.source != request.navigation_start().source()
                    || original.microseconds != request.navigation_start().microseconds()
                {
                    return Err(DomHostedRuntimeError::Protocol);
                }
                if creation.finished_sample.is_none() {
                    creation.finished_sample = Some(
                        clock
                            .sample()
                            .map_err(clock_error)?
                            .coarsen()
                            .map_err(clock_error)?,
                    );
                }
                let sample = creation
                    .finished_sample
                    .expect("original completion sample");
                if sample.source != source.id() {
                    return Err(DomHostedRuntimeError::Binding);
                }
                if creation.publication.is_none() {
                    creation.publication = Some(
                        source
                            .seal_finish_loading(request, sample.microseconds, cancel)
                            .map_err(DomHostedRuntimeError::Dom)?,
                    );
                }
            }
            DomHostedCreationStep::InitialEntry(request) => {
                policy_phase = false;
                if creation.uuids.is_none()
                    && creation.entry_initialization.is_none()
                    && creation.publication.is_none()
                {
                    spend(work, limit, cancel)?;
                    uuid::HistoryUuids::sample(&mut creation.uuids, cancel)?;
                }
                if creation.entry.is_none() {
                    creation.entry = Some(
                        source
                            .issue_entry(request, cancel)
                            .map_err(DomHostedRuntimeError::Dom)?,
                    );
                }
                if creation.navigable.is_none() {
                    creation.navigable = Some(
                        source
                            .issue_navigable(request, cancel)
                            .map_err(DomHostedRuntimeError::Dom)?,
                    );
                }
                let entry = creation.entry.expect("original issued entry");
                let navigable = creation.navigable.expect("original issued navigable");
                if creation.entry_initialization.is_none() && creation.publication.is_none() {
                    let values = creation
                        .uuids
                        .take()
                        .ok_or(DomHostedRuntimeError::Protocol)?;
                    creation.entry_initialization =
                        Some(DomHostedEntryInitialization::new(values.key, values.id));
                }
                if creation.publication.is_none() {
                    creation.publication = Some(
                        source
                            .seal_initial_entry(
                                request,
                                entry,
                                navigable,
                                &mut creation.entry_initialization,
                                cancel,
                            )
                            .map_err(DomHostedRuntimeError::Dom)?,
                    );
                }
                let group = host.group.as_mut().ok_or(DomHostedRuntimeError::Protocol)?;
                match group.traversal.traversable {
                    Some(original) if original != navigable => {
                        return Err(DomHostedRuntimeError::Protocol);
                    }
                    None => group.traversal.traversable = Some(navigable),
                    _ => {}
                }
                if host.top.is_none() {
                    host.top = Some(records::Top {
                        id: navigable,
                        closing: false,
                    });
                }
            }
            DomHostedCreationStep::Complete(document) => {
                let top = host.top.as_ref().ok_or(DomHostedRuntimeError::Protocol)?;
                let complete = lease
                    .finish(dom.raw_mut(), source, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?;
                if complete.document() != document || complete.navigable() != top.id {
                    return Err(DomHostedRuntimeError::Protocol);
                }
                return Ok(DomHostedTopLevel {
                    host: complete.host(),
                    context: complete.browsing_context(),
                    navigable: complete.navigable(),
                    entry: complete.entry(),
                    global: complete.global(),
                    document: complete.document(),
                });
            }
        }
        if policy_phase {
            lease
                .accept_permissions_policy(dom.raw_mut(), source, &mut creation.policy, cancel)
                .map_err(DomHostedRuntimeError::Dom)?;
        } else {
            lease
                .accept_publication(dom.raw_mut(), source, &mut creation.publication, cancel)
                .map_err(DomHostedRuntimeError::Dom)?;
        }
    }
}

impl HostedRuntime {
    /// Construct the real no-opener initial about:blank top-level for this host.
    /// Every reached phase and established input remains in the original session
    /// on refusal; resume retries the same operation without resampling.
    pub fn create_top_level(
        &mut self,
        dom: &mut DomEventDom<'_>,
        host: DomHostedHostId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedTopLevel, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        if state.creation.is_some() {
            return Err(DomHostedRuntimeError::OperationPending);
        }
        let mut found = None;
        for index in 0..state.hosts.len() {
            state.step(cancel)?;
            if state.hosts[index].id == host {
                found = Some(index);
                break;
            }
        }
        let index = found.ok_or(DomHostedRuntimeError::TargetNotFound)?;
        state.reserve_composition(index, cancel)?;
        let composition = state.hosts[index].top_levels.len();
        state.hosts[index].top_levels.push(records::Composition {
            group: None,
            top: None,
        });
        state.creation = Some(Creation::new(host, composition));
        state.resume_creation(dom, cancel)
    }
    /// Resume the original retained lifecycle operation. No permission owns or
    /// extracts its phase cursor, actual objects, issuer or canonical DOM.
    pub fn resume_top_level_creation(
        &mut self,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedTopLevel, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        state.resume_creation(dom, cancel)
    }
}
impl HostedState {
    fn resume_creation(
        &mut self,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedTopLevel, DomHostedRuntimeError> {
        let retained_before = self.retained_bytes(cancel)?;
        let Self {
            source,
            clock,
            hosts,
            work,
            limits,
            creation,
            ..
        } = self;
        let top = drive_creation(
            CreationInput {
                source,
                clock,
                hosts,
                work,
                limits: *limits,
                retained_before,
            },
            creation
                .as_mut()
                .ok_or(DomHostedRuntimeError::TargetNotFound)?,
            dom,
            cancel,
        )?;
        // Only actual canonical finish closes the operation. Public identity
        // observations cannot steal this cursor or acknowledge its phases.
        *creation = None;
        Ok(top)
    }
}
impl HostedOwner {
    pub(crate) fn pending(&self) -> Result<bool, DomHostedRuntimeError> {
        let state = self
            .state
            .try_borrow()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        Ok(state.creation.is_some()
            || !state.children.is_empty()
            || state.destruction.pending()
            || !state.hosts.is_empty())
    }
}

impl HostedRuntime {
    pub(crate) fn ingest_element_tasks(
        &self,
        element_tasks: &crate::element_tasks::DomElementTaskRuntime,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<usize, crate::element_tasks::DomElementTaskRuntimeError> {
        let owner = self
            .state()
            .map_err(crate::element_tasks::DomElementTaskRuntimeError::Hosted)?;
        let state = owner
            .try_borrow()
            .map_err(|_| crate::element_tasks::DomElementTaskRuntimeError::Busy)?;
        state
            .qualify(dom)
            .map_err(crate::element_tasks::DomElementTaskRuntimeError::Hosted)?;
        element_tasks.ingest_source(dom, &state.source, cancel)
    }
}

// The actual loop has stable private custody; public event observations cannot
// retain this guard or pin a loop after terminal session disposal.
pub(crate) struct NativeTaskRunning {
    state: Weak<RefCell<HostedState>>,
    event_loop: Rc<RefCell<records::WindowLoop>>,
    complete: bool,
}
impl NativeTaskRunning {
    pub(crate) fn checkpoint(
        &mut self,
        dom: &DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self
            .state
            .upgrade()
            .ok_or(DomHostedRuntimeError::OwnerDisposed)?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        state.step(cancel)?;
        let snapshot = dom.snapshot();
        if snapshot.incomplete_source_operation().is_some()
            || snapshot.incomplete_hosted_operation().is_some()
            || !snapshot.pending_slot_signals().is_empty()
        {
            return Err(DomHostedRuntimeError::CheckpointUnavailable);
        }
        Ok(())
    }
    pub(crate) fn complete(&mut self) {
        self.complete = true;
    }
}
impl Drop for NativeTaskRunning {
    fn drop(&mut self) {
        // No loop borrow crosses user code, and no public API lends Ref/RefMut.
        // Cleanup is constant-sized and infallible, independent of bank growth.
        let mut event_loop = self.event_loop.borrow_mut();
        event_loop.executing = false;
        if self.complete {
            event_loop.current = None;
            event_loop.checkpoints = event_loop.next_checkpoint;
        }
    }
}
impl HostedRuntime {
    pub(crate) fn enter_element_task(
        &self,
        dom: &DomEventDom<'_>,
        association: surgeist_dom::DomElementTaskAssociation,
        source: surgeist_dom::DomElementTaskSourceKind,
        task: crate::element_tasks::DomElementTaskId,
        retired_target: bool,
        cancel: &AtomicBool,
    ) -> Result<NativeTaskRunning, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        let document = association.document();
        let (h, t, c, g) = state.find_global(association.global(), cancel, || Ok(()))?;
        let cluster = &state.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .clusters[c];
        let global = &cluster.agent.globals[g];
        if global.settings.document != Some(document) || !global.settings.ready {
            return Err(DomHostedRuntimeError::Inactive);
        }
        let loop_owner = cluster.agent.event_loop.clone();
        let mut event_loop = loop_owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let current = records::CurrentTask::ElementTask {
            source,
            task,
            document,
        };
        if event_loop.executing || event_loop.current.is_some_and(|old| old != current) {
            return Err(DomHostedRuntimeError::Busy);
        }
        let fresh = event_loop.current.is_none();
        if fresh && !retired_target {
            let snapshot = dom.snapshot();
            if dom
                .raw()
                .global_document(association.global())
                .map_err(DomHostedRuntimeError::Dom)?
                != document
            {
                return Err(DomHostedRuntimeError::Binding);
            }
            if !snapshot
                .document(document)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?
                .is_fully_active()
                .map_err(DomHostedRuntimeError::Dom)?
            {
                return Err(DomHostedRuntimeError::Inactive);
            }
        }
        if fresh {
            state.step(cancel)?;
            event_loop.next_checkpoint = event_loop
                .checkpoints
                .checked_add(1)
                .ok_or(DomHostedRuntimeError::Admission)?;
            event_loop.current = Some(current);
        }
        event_loop.executing = true;
        drop(event_loop);
        Ok(NativeTaskRunning {
            state: Rc::downgrade(&owner),
            event_loop: loop_owner,
            complete: false,
        })
    }
}

impl HostedRuntime {
    /// Adjust finite admission on the same source/session, preserving all actual
    /// records, original samples, phase packets and cumulative spent work.
    pub fn set_limits(
        &mut self,
        dom: &mut DomEventDom<'_>,
        limits: DomHostedRuntimeLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        let state = &mut *state;
        let retained = native_retained_bytes(
            &state.hosts,
            &state.children,
            &state.destruction,
            &mut state.work,
            limits.work,
            cancel,
            || Ok(()),
        )?;
        let tasks = native_task_count(&state.hosts, &mut state.work, limits.work, cancel)?;
        spend(&mut state.work, limits.work, cancel)?;
        if retained > limits.storage_bytes || tasks > limits.tasks || state.work > limits.work {
            return Err(DomHostedRuntimeError::Admission);
        }
        state
            .source
            .set_limits(limits.canonical, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        // No fallible work after canonical policy acceptance. Existing active
        // cleanup work was charged, and its maximum bank size was preserved.
        state.limits = limits;
        Ok(())
    }
}

impl HostedOwner {
    pub(crate) fn bind_element_tasks(
        &self,
        element_tasks: crate::element_tasks::DomElementTaskRuntime,
    ) {
        self.state.borrow_mut().element_tasks = Some(element_tasks);
    }
}

impl HostedRuntime {
    /// Retire this actual Host only after its ordered top-level set and every
    /// pending lifecycle closure are empty. Other Hosts remain untouched.
    pub fn destroy_host(
        &mut self,
        dom: &mut DomEventDom<'_>,
        host: DomHostedHostId,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        if state.creation.is_some() {
            return Err(DomHostedRuntimeError::OperationPending);
        }
        let mut found = None;
        for h in 0..state.hosts.len() {
            state.step(cancel)?;
            if state.hosts[h].id == host {
                found = Some(h);
                break;
            }
        }
        let index = found.ok_or(DomHostedRuntimeError::TargetNotFound)?;
        if !state.hosts[index].top_levels.is_empty() {
            return Err(DomHostedRuntimeError::OperationPending);
        }
        for _ in index..state.hosts.len() {
            state.step(cancel)?;
        }
        state
            .source
            .retire_host(host, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        state.hosts.remove(index);
        Ok(())
    }
}

impl HostedRuntime {
    /// Read the actual current initial entry, with finite copying admission.
    pub fn history_entry(
        &self,
        dom: &DomEventDom<'_>,
        entry: DomHostedEntryId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedHistoryEntry, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        let retained = state.retained_bytes(cancel)?;
        let copy = size_of::<DomHostedHistoryEntry>();
        if copy > state.limits.scratch_bytes
            || retained
                .checked_add(copy)
                .is_none_or(|n| n > state.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        let record = dom
            .raw()
            .hosted_entry(entry)
            .map_err(DomHostedRuntimeError::Dom)?;
        let document = record.document().ok_or(DomHostedRuntimeError::Inactive)?;
        for _ in 0..72 {
            state.step(cancel)?;
        }
        let global = dom
            .raw()
            .relevant_global(document.node())
            .map_err(DomHostedRuntimeError::Dom)?
            .ok_or(DomHostedRuntimeError::Binding)?;
        state.find_global(global, cancel, || Ok(()))?;
        if dom
            .raw()
            .global_document(global)
            .map_err(DomHostedRuntimeError::Dom)?
            != document
        {
            return Err(DomHostedRuntimeError::Binding);
        }
        Ok(DomHostedHistoryEntry {
            entry,
            document,
            origin: record.origin().clone(),
            key: *record.navigation_key(),
            id: *record.navigation_id(),
            step: record.step(),
        })
    }
}

#[cfg(all(
    test,
    any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    )
))]
mod constructor_settings_tests {
    use super::*;
    use crate::{DomElementTaskRuntimeLimits, DomSession, DomSessionLimits, DomStyleRuntimeLimits};
    use surgeist_css::CssUtf16InputLimits;
    use surgeist_dom::{DomHostedRecordLimits, DomLimits};

    #[test]
    fn settings_url_payload_refusal_and_exact_retry_keep_original_execution() {
        let cancel = AtomicBool::new(false);
        let mut limits = DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 32,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        };
        let mut session = DomSession::new(
            Dom::new(DomLimits::new(128, 200_000, 256, 256, 256, 32, 1_000_000).unwrap()).unwrap(),
            DomSessionLimits {
                source_events: crate::DomSourceEventRuntimeLimits {
                    jobs: 0,
                    storage_bytes: 16_000_000,
                    work: 1_000_000,
                },
                style: DomStyleRuntimeLimits {
                    roots: 4,
                    completions: 32,
                    reports: 32,
                    storage_bytes: 16_000_000,
                    work: 1_000_000,
                    css_input: CssUtf16InputLimits::default(),
                },
                element_tasks: DomElementTaskRuntimeLimits {
                    tasks: 32,
                    storage_bytes: 16_000_000,
                    work: 1_000_000,
                },
                hosted: limits,
            },
        )
        .unwrap();
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        let owner = hosted.state().unwrap();
        // Reserve the actual original composition bank, so the measured limit
        // below isolates the reached Settings payload rather than Vec growth.
        let before_loop = {
            let mut state = owner.borrow_mut();
            state.reserve_composition(0, &cancel).unwrap();
            state.retained_bytes(&cancel).unwrap()
        };
        limits.storage_bytes = before_loop;
        hosted.set_limits(&mut dom, limits, &cancel).unwrap();
        assert!(matches!(
            hosted.create_top_level(&mut dom, host, &cancel),
            Err(DomHostedRuntimeError::Admission)
        ));
        // Form the real agent banks with ample admission, without constructing
        // Settings. This isolates URL payload refusal from allocator capacity.
        limits.storage_bytes = 16_000_000;
        hosted.set_limits(&mut dom, limits, &cancel).unwrap();
        {
            let mut state = owner.borrow_mut();
            let DomHostedCreationStep::NativeExecution(request) = state
                .creation
                .as_ref()
                .unwrap()
                .lease
                .as_ref()
                .unwrap()
                .step(dom.raw())
                .unwrap()
            else {
                panic!("the real constructor must retain its Execution request")
            };
            let origin = request.origin().clone();
            let retained = state.retained_bytes(&cancel).unwrap();
            {
                let HostedState {
                    hosts,
                    work,
                    source,
                    ..
                } = &mut *state;
                agent::ensure_cluster(
                    hosts[0].top_levels[0].group.as_mut().unwrap(),
                    &origin,
                    retained,
                    limits,
                    work,
                    &cancel,
                    |bytes| source.retain_runtime_payload(dom.raw_mut(), bytes, &cancel),
                )
                .unwrap();
            }
            let retained = state.retained_bytes(&cancel).unwrap();
            let HostedState {
                hosts,
                work,
                source,
                ..
            } = &mut *state;
            let native_agent =
                &mut hosts[0].top_levels[0].group.as_mut().unwrap().clusters[0].agent;
            let added =
                agent::reserve_global(native_agent, retained, limits, work, &cancel).unwrap();
            watcher::reserve_window(
                native_agent,
                retained + added,
                limits,
                work,
                |bytes| source.retain_runtime_payload(dom.raw_mut(), bytes, &cancel),
                &cancel,
            )
            .unwrap();
        }
        limits.storage_bytes = owner.borrow_mut().retained_bytes(&cancel).unwrap();
        hosted.set_limits(&mut dom, limits, &cancel).unwrap();
        assert!(matches!(
            hosted.resume_top_level_creation(&mut dom, &cancel),
            Err(DomHostedRuntimeError::Admission)
        ));
        let (operation, creation_url, top_url, exact, spent) = {
            let mut state = owner.borrow_mut();
            let retained = state.retained_bytes(&cancel).unwrap();
            assert_eq!(retained, limits.storage_bytes);
            let creation = state.creation.as_ref().unwrap();
            let DomHostedCreationStep::NativeExecution(request) =
                creation.lease.as_ref().unwrap().step(dom.raw()).unwrap()
            else {
                panic!("Settings refusal must retain the original Execution phase")
            };
            let creation_url = request.creation_url().clone();
            let top_url = request.top_level_creation_url().clone();
            // Two actual Settings fields retain two declared occurrences even
            // when the canonical constructor selected the same record for both.
            assert_eq!(creation_url, top_url);
            let exact = retained
                + creation_url.retained_storage_usage().retained_bytes()
                + top_url.retained_storage_usage().retained_bytes();
            assert!(
                state.hosts[0].top_levels[0]
                    .group
                    .as_ref()
                    .unwrap()
                    .clusters
                    .first()
                    .unwrap()
                    .agent
                    .globals
                    .is_empty()
            );
            (
                request.operation(),
                creation_url,
                top_url,
                exact,
                state.work,
            )
        };
        limits.storage_bytes = exact - 1;
        hosted.set_limits(&mut dom, limits, &cancel).unwrap();
        assert!(matches!(
            hosted.resume_top_level_creation(&mut dom, &cancel),
            Err(DomHostedRuntimeError::Admission)
        ));
        {
            let state = owner.borrow();
            let DomHostedCreationStep::NativeExecution(request) = state
                .creation
                .as_ref()
                .unwrap()
                .lease
                .as_ref()
                .unwrap()
                .step(dom.raw())
                .unwrap()
            else {
                panic!("one-short admission must preserve the same phase")
            };
            assert_eq!(request.operation(), operation);
            assert_eq!(request.creation_url(), &creation_url);
            assert_eq!(request.top_level_creation_url(), &top_url);
            assert!(state.work > spent);
        }
        limits.storage_bytes = exact;
        let cancelled = AtomicBool::new(true);
        assert_eq!(
            hosted.set_limits(&mut dom, limits, &cancelled),
            Err(DomHostedRuntimeError::Cancelled)
        );
        assert_eq!(owner.borrow().limits.storage_bytes, exact - 1);
        assert!(matches!(
            hosted.resume_top_level_creation(&mut dom, &cancelled),
            Err(DomHostedRuntimeError::Cancelled)
        ));
        hosted.set_limits(&mut dom, limits, &cancel).unwrap();
        let top = hosted.resume_top_level_creation(&mut dom, &cancel).unwrap();
        let settings = hosted.settings(&dom, top.global(), &cancel).unwrap();
        assert_eq!(settings.creation_url(), &creation_url);
        assert_eq!(settings.top_level_creation_url(), &top_url);
        let mut state = owner.borrow_mut();
        assert_eq!(state.retained_bytes(&cancel).unwrap(), exact);
        assert!(state.creation.is_none());
    }
}
