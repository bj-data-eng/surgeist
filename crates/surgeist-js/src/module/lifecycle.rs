//! Instance-specific source evaluation custody and prepared terminal graph walk.
use super::{
    Module, ModuleEdge,
    metadata::{Buffer, Control, Work},
};
use crate::{
    JsError, JsResult,
    builtins::promise::native_settlement::Signal,
    error::{EngineError, ModuleInterruption, RetainedInterruption},
    job::{JobFault, JobFaultCause, JobFaultOwner},
    vm::{
        call_frame::InvocationId,
        opcode::{AwaitPhase, AwaitRef, AwaitRoot},
    },
};
use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::{Rc, Weak},
    sync::Arc,
};
use surgeist_js_gc::heap::{AccessError, EdgeVisitor, RecordRef, Rooted, Trace};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    NotEntered,
    Incomplete,
    Complete,
    Interrupted,
}
#[derive(Default, Trace)]
struct Links {
    dfs_previous: Option<ModuleEdge>,
    dfs_parent: Option<EvaluationRef>,
    cycle: Option<EvaluationRef>,
    cycle_root: Option<EvaluationRef>,
    parents: Option<ParentKey>,
    work: Option<EvaluationRef>,
    notice: Option<EvaluationRef>,
    suspended: Option<AwaitRef>,
    ordinary_next: Option<ModuleEdge>,
}
#[derive(Clone, Trace)]
struct ParentKey {
    evaluation: EvaluationRef,
    module: ModuleEdge,
    request: usize,
}
#[derive(Default, Trace)]
struct ParentSlot {
    next: Option<ParentKey>,
    registered: bool,
    pending: bool,
    deferred_ready: bool,
}
pub(crate) struct EvaluationData {
    phase: Cell<Phase>,
    fault: Cell<Option<JobFault>>,
    invocation: Cell<Option<InvocationId>>,
    prepared: Rc<RetainedInterruption>,
    interrupted: OnceCell<Rc<RetainedInterruption>>,
    links: RefCell<Links>,
    slots: RefCell<Vec<ParentSlot>>,
    walked: Cell<bool>,
    issued: OnceCell<Arc<Signal>>,
}
impl Trace for EvaluationData {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        self.prepared.trace(edges)?;
        if let Some(cause) = self.interrupted.get() {
            cause.trace(edges)?;
        }
        self.links.trace(edges)?;
        self.slots.trace(edges)
    }
}
#[derive(Trace)]
struct EvaluationRecord(Rc<EvaluationData>);
/// Traced identity plus a weak local view; observation needs no heap acquisition.
#[derive(Clone, Trace)]
pub(crate) struct EvaluationRef {
    edge: RecordRef<EvaluationRecord>,
    #[heap_trace(skip)]
    data: Weak<EvaluationData>,
}
impl std::fmt::Debug for EvaluationRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Evaluation")
            .field(&self.edge.identity())
            .finish()
    }
}
/// Temporary construction custody; the published record retains only traced edges.
pub(super) struct PreparedEvaluationRecord {
    pub(super) evaluation: EvaluationRef,
    _root: Rooted<EvaluationRecord>,
}
impl EvaluationRef {
    pub(super) fn prepare(
        requests: usize,
        control: &mut Control<'_>,
    ) -> JsResult<PreparedEvaluationRecord> {
        let mut slots = Buffer::default();
        control.reserve_exact(&mut slots, requests)?;
        for _ in 0..requests {
            control.initialize(&mut slots, ParentSlot::default())?;
        }
        for bytes in RetainedInterruption::preparation_layout() {
            control.admit(Work::Record)?;
            control.admit(Work::Capacity(bytes))?;
        }
        control.header::<EvaluationData>()?;
        control.header::<EvaluationRecord>()?;
        let (prepared, anchor) = RetainedInterruption::prepare(control.context())?;
        let data = Rc::new(EvaluationData {
            phase: Cell::new(Phase::NotEntered),
            fault: Cell::new(None),
            invocation: Cell::new(None),
            prepared,
            interrupted: OnceCell::new(),
            links: RefCell::new(Links::default()),
            slots: RefCell::new(slots.into_vec()),
            walked: Cell::new(false),
            issued: OnceCell::new(),
        });
        let record = control
            .context()
            .runtime()
            .heap
            .allocate(EvaluationRecord(data.clone()))
            .map_err(|_| crate::error::EngineResourceError::Allocation)?;
        let result = Self {
            edge: record.record_ref(),
            data: Rc::downgrade(&data),
        };
        data.links.borrow_mut().cycle = Some(result.clone());
        data.links.borrow_mut().cycle_root = Some(result.clone());
        drop(anchor);
        Ok(PreparedEvaluationRecord {
            evaluation: result,
            _root: record,
        })
    }
    fn data(&self) -> Rc<EvaluationData> {
        self.data.upgrade().expect("rooted evaluation remains live")
    }
    fn checked_data(&self) -> Result<Rc<EvaluationData>, JobFault> {
        self.data.upgrade().ok_or_else(|| {
            JobFault::new(
                JobFaultOwner::ModuleEvaluation,
                JobFaultCause::OwnerMismatch,
            )
        })
    }
    pub(crate) fn is_complete(&self) -> JsResult<bool> {
        let data = self.checked_data()?;
        if data.phase.get() == Phase::Complete {
            return Ok(true);
        }
        data.fault
            .get()
            .map_or(Ok(false), |error| Err(error.into()))
    }
    pub(crate) fn same(&self, other: &Self) -> bool {
        self.edge == other.edge
    }
    fn record_fault(&self, fault: JobFault) -> JobFault {
        if let Some(data) = self.data.upgrade() {
            let first = data.fault.get().unwrap_or(fault);
            data.fault.set(Some(first));
            first
        } else {
            fault
        }
    }
    pub(crate) fn cause(&self) -> Option<Rc<RetainedInterruption>> {
        self.data().interrupted.get().cloned()
    }
    pub(crate) fn error(&self, module: &Module) -> Option<JsError> {
        self.cause().map(|cause| {
            EngineError::ModuleInterrupted(ModuleInterruption::module(cause, module.clone())).into()
        })
    }
    pub(crate) fn start_invocation(&self, invocation: InvocationId) {
        self.data().invocation.set(Some(invocation));
    }
    pub(crate) fn validate_invocation(&self, invocation: InvocationId) -> JsResult<()> {
        if self.data().invocation.get() == Some(invocation) {
            Ok(())
        } else {
            Err(EngineError::InvalidEntryPermit.into())
        }
    }
    pub(crate) fn enter(&self) {
        let data = self.data();
        if data.phase.get() == Phase::NotEntered {
            data.phase.set(Phase::Incomplete);
        }
    }
    pub(crate) fn complete(&self) {
        let data = self.data();
        if data.phase.get() != Phase::Interrupted {
            data.phase.set(Phase::Complete);
        }
    }
    pub(crate) fn issue(&self, signal: Arc<Signal>) {
        let data = self.data();
        if let Some(existing) = data.issued.get() {
            assert!(
                Arc::ptr_eq(existing, &signal),
                "published evaluation signal cannot be replaced"
            );
        } else {
            let _ = data.issued.set(signal);
        }
    }
    pub(crate) fn suspended(&self, state: AwaitRef) -> Result<(), JobFault> {
        let data = self.checked_data()?;
        let result = data
            .links
            .try_borrow_mut()
            .map_err(|_| {
                JobFault::new(
                    JobFaultOwner::ModuleEvaluation,
                    JobFaultCause::InaccessibleRegistry,
                )
            })
            .map(|mut links| links.suspended = Some(state));
        result.map_err(|fault| self.record_fault(fault))
    }
    pub(crate) fn consume(&self, state: &AwaitRef) -> Result<(), JobFault> {
        let data = self.checked_data()?;
        let result = data
            .links
            .try_borrow_mut()
            .map_err(|_| {
                JobFault::new(
                    JobFaultOwner::ModuleEvaluation,
                    JobFaultCause::InaccessibleRegistry,
                )
            })
            .map(|mut links| {
                if links
                    .suspended
                    .as_ref()
                    .is_some_and(|current| current.same(state))
                {
                    links.suspended = None;
                }
            });
        result.map_err(|fault| self.record_fault(fault))
    }
    pub(crate) fn has_invocation(&self) -> bool {
        self.data().invocation.get().is_some()
    }
    pub(crate) fn ordinary_next(&self) -> Option<Module> {
        self.data()
            .links
            .borrow()
            .ordinary_next
            .clone()
            .map(Module::from_edge)
    }
    pub(crate) fn set_ordinary_next(&self, next: Option<&Module>) {
        self.data().links.borrow_mut().ordinary_next = next.map(Module::edge);
    }
    pub(crate) fn take_ordinary_next(&self) -> Option<Module> {
        self.data()
            .links
            .borrow_mut()
            .ordinary_next
            .take()
            .map(Module::from_edge)
    }
    pub(crate) fn take_deferred_ready(&self) -> usize {
        let data = self.data();
        let mut count = 0;
        for slot in data.slots.borrow_mut().iter_mut() {
            if slot.pending && slot.deferred_ready {
                slot.pending = false;
                count += 1;
            }
            slot.deferred_ready = false;
        }
        count
    }
    pub(crate) fn cycle(&self, root: &Self, next: &Self) {
        let data = self.data();
        let mut links = data.links.borrow_mut();
        links.cycle_root = Some(root.clone());
        links.cycle = Some(next.clone());
    }
    pub(crate) fn register_parent(&self, parent: &Module, request: usize) {
        let parent_ref = parent
            .kind()
            .as_source_text()
            .expect("source parent")
            .evaluation
            .clone();
        let parent_data = parent_ref.data();
        let mut slots = parent_data.slots.borrow_mut();
        let slot = &mut slots[request];
        if slot.registered {
            return;
        }
        let data = self.data();
        let mut links = data.links.borrow_mut();
        slot.next = links.parents.take();
        slot.registered = true;
        slot.pending = true;
        links.parents = Some(ParentKey {
            evaluation: parent_ref.clone(),
            module: parent.edge(),
            request,
        });
    }
    pub(crate) fn parents(&self) -> Parents {
        Parents(self.data().links.borrow().parents.clone())
    }
    pub(crate) fn interrupt(&self, error: &JsError) {
        let data = self.data();
        if data.phase.get() != Phase::Incomplete {
            return;
        }
        let cause = data.prepared.seal(error);
        let mut work = None;
        let mut notices = None;
        let mut active = Some(self.clone());
        while let Some(record) = active {
            active = record.data().links.borrow().dfs_parent.clone();
            Self::mark(&record, &cause, &mut work, &mut notices);
        }
        while let Some(current) = work.take() {
            let current_data = current.data();
            work = current_data.links.borrow_mut().work.take();
            let root = current_data
                .links
                .borrow()
                .cycle_root
                .clone()
                .expect("cycle root");
            if !root.data().walked.replace(true) {
                let mut member = root.clone();
                loop {
                    Self::mark(&member, &cause, &mut work, &mut notices);
                    let next = member
                        .data()
                        .links
                        .borrow()
                        .cycle
                        .clone()
                        .expect("cycle member");
                    if next.edge == root.edge {
                        break;
                    }
                    member = next;
                }
            }
            for parent in current.parents() {
                Self::mark(&parent.key.evaluation, &cause, &mut work, &mut notices);
            }
        }
        let mut cursor = notices.clone();
        let mut cleanup = crate::job::UnwindCleanup::default();
        while let Some(record) = cursor {
            let data = record.data();
            cursor = data.links.borrow().notice.clone();
            let suspended = data.links.borrow_mut().suspended.take();
            if let Some(suspended) = suspended {
                cleanup.run(|| suspended.retire());
            }
            if let Some(signal) = data.issued.get() {
                signal.mark_module();
            }
        }
        while let Some(record) = notices {
            let data = record.data();
            notices = data.links.borrow_mut().notice.take();
            if let Some(signal) = data.issued.get() {
                cleanup.run(|| signal.notify_module());
            }
        }
        cleanup.finish();
    }
    fn mark(
        record: &Self,
        cause: &Rc<RetainedInterruption>,
        work: &mut Option<Self>,
        notices: &mut Option<Self>,
    ) {
        let data = record.data();
        if data.phase.get() != Phase::Incomplete {
            return;
        }
        let _ = data.interrupted.set(cause.clone());
        data.phase.set(Phase::Interrupted);
        let mut links = data.links.borrow_mut();
        links.work = work.take();
        links.notice = notices.take();
        *work = Some(record.clone());
        *notices = Some(record.clone());
    }
}
pub(crate) struct Parents(Option<ParentKey>);
pub(crate) struct Parent {
    key: ParentKey,
}
impl Iterator for Parents {
    type Item = Parent;
    fn next(&mut self) -> Option<Self::Item> {
        let key = self.0.take()?;
        self.0 = key.evaluation.data().slots.borrow()[key.request]
            .next
            .clone();
        Some(Parent { key })
    }
}
impl Parent {
    pub(crate) fn module(&self) -> Module {
        Module::from_edge(self.key.module.clone())
    }
    pub(crate) fn defer_ready(&self) {
        self.key.evaluation.data().slots.borrow_mut()[self.key.request].deferred_ready = true;
    }
    pub(crate) fn consume(&self) -> bool {
        let data = self.key.evaluation.data();
        let mut slots = data.slots.borrow_mut();
        slots[self.key.request].deferred_ready = false;
        std::mem::replace(&mut slots[self.key.request].pending, false)
    }
}
#[derive(Default)]
pub(crate) struct EvaluationStack {
    head: Option<Module>,
}
impl EvaluationStack {
    pub(crate) fn interrupt(&self, error: &JsError) {
        if let Some(head) = &self.head {
            head.kind()
                .as_source_text()
                .expect("source stack")
                .evaluation
                .interrupt(error);
        }
    }
    pub(crate) fn push(&mut self, module: Module) {
        let evaluation = &module
            .kind()
            .as_source_text()
            .expect("source stack")
            .evaluation;
        let data = evaluation.data();
        let mut links = data.links.borrow_mut();
        links.dfs_parent = self.head.as_ref().map(|module| {
            module
                .kind()
                .as_source_text()
                .expect("source stack")
                .evaluation
                .clone()
        });
        links.dfs_previous = self.head.take().map(|module| module.edge());
        drop(links);
        self.head = Some(module);
    }
    pub(crate) fn pop(&mut self) -> Option<Module> {
        let module = self.head.take()?;
        let data = module
            .kind()
            .as_source_text()
            .expect("source stack")
            .evaluation
            .data();
        let previous = {
            let mut links = data.links.borrow_mut();
            links.dfs_parent = None;
            links.dfs_previous.take()
        };
        self.head = previous.map(Module::from_edge);
        Some(module)
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.head.is_none()
    }
    pub(crate) fn contains(&self, module: &Module) -> bool {
        let mut head = self.head.clone();
        while let Some(current) = head {
            if &current == module {
                return true;
            }
            head = current
                .kind()
                .as_source_text()
                .expect("source stack")
                .evaluation
                .data()
                .links
                .borrow()
                .dfs_previous
                .clone()
                .map(Module::from_edge);
        }
        false
    }
}

/// Custody assigned only to the actual source-module top-level invocation.
#[derive(Clone, Debug, Trace)]
pub(crate) struct EvaluationInvocation {
    pub(crate) evaluation: EvaluationRef,
    #[heap_trace(skip)]
    pub(crate) invocation: InvocationId,
}

/// Error-aware metadata for the one engine-owned module reaction selected to run.
#[derive(Clone, Debug, Trace)]
pub(crate) struct ModuleAction {
    evaluation: EvaluationRef,
    module: ModuleEdge,
    resume: Option<AwaitRef>,
}
impl ModuleAction {
    pub(crate) fn complete(module: &Module) -> JsResult<Self> {
        let source = module
            .kind()
            .as_source_text()
            .ok_or(EngineError::InvalidEntryPermit)?;
        Ok(Self {
            evaluation: source.evaluation.clone(),
            module: module.edge(),
            resume: None,
        })
    }
    pub(crate) fn resume(module: &Module, state: AwaitRef) -> JsResult<Self> {
        let mut action = Self::complete(module)?;
        if !state.matches_evaluation(&action.evaluation)? {
            return Err(EngineError::InvalidEntryPermit.into());
        }
        action.resume = Some(state);
        Ok(action)
    }
    pub(crate) fn await_ref(&self) -> Option<&AwaitRef> {
        self.resume.as_ref()
    }
    pub(crate) fn hold(&self) -> JsResult<ModuleActionHold> {
        let data = self.evaluation.checked_data()?;
        if let Some(error) = data.fault.get() {
            return Err(error.into());
        }
        let await_root = self.resume.as_ref().map(AwaitRef::root).transpose()?;
        let inner = self
            .module
            .0
            .upgrade()
            .map_err(|error| {
                JobFault::new(
                    JobFaultOwner::ModuleEvaluation,
                    match error {
                        AccessError::HeapBorrowed | AccessError::PayloadBorrowed => {
                            JobFaultCause::InaccessibleRegistry
                        }
                        AccessError::ForeignHeap
                        | AccessError::StaleHandle
                        | AccessError::WrongType => JobFaultCause::OwnerMismatch,
                    },
                )
            })?
            .ok_or_else(|| {
                JobFault::new(
                    JobFaultOwner::ModuleEvaluation,
                    JobFaultCause::OwnerMismatch,
                )
            })?;
        Ok(ModuleActionHold {
            action: self.clone(),
            module: Module { inner },
            await_root,
        })
    }
    pub(crate) fn preparation_failed(&self, error: &JsError) -> Result<(), JobFault> {
        let result = self.preparation_failed_inner(error);
        result.map_err(|fault| self.evaluation.record_fault(fault))
    }
    fn preparation_failed_inner(&self, error: &JsError) -> Result<(), JobFault> {
        let Some(reference) = &self.resume else {
            return Ok(());
        };
        if matches!(
            reference.phase()?,
            AwaitPhase::Prepared | AwaitPhase::Uncommitted | AwaitPhase::PolicyDiscarded
        ) {
            return Ok(());
        }
        reference.preparation_failed()?;
        self.finish_failure(error)
    }
    fn finish_failure(&self, error: &JsError) -> Result<(), JobFault> {
        self.finish_failure_inner(error)
            .map_err(|fault| self.evaluation.record_fault(fault))
    }
    fn finish_failure_inner(&self, error: &JsError) -> Result<(), JobFault> {
        if error.as_engine().is_none() {
            return Ok(());
        }
        let data = self.evaluation.checked_data()?;
        if matches!(data.phase.get(), Phase::Complete | Phase::Interrupted) {
            return Ok(());
        }
        let suspended = data
            .links
            .try_borrow()
            .map_err(|_| {
                JobFault::new(
                    JobFaultOwner::ModuleEvaluation,
                    JobFaultCause::InaccessibleRegistry,
                )
            })?
            .suspended
            .clone();
        if let Some(current) = suspended
            && self.resume.as_ref().is_none_or(|old| !old.same(&current))
        {
            let phase = current.phase()?;
            if matches!(
                phase,
                AwaitPhase::SourcePending | AwaitPhase::ExecutorAccepted
            ) {
                return Ok(());
            }
        }
        self.evaluation.interrupt(error);
        Ok(())
    }
}
/// Root leases only. Drop cannot dispose an action or change phase.
#[derive(Clone)]
pub(crate) struct ModuleActionHold {
    action: ModuleAction,
    module: Module,
    await_root: Option<AwaitRoot>,
}
impl ModuleActionHold {
    pub(crate) fn await_root(&self) -> Option<&AwaitRoot> {
        self.await_root.as_ref()
    }
    pub(crate) fn job_owner(&self) -> ModuleActionRoot {
        ModuleActionRoot { hold: self.clone() }
    }
    pub(crate) fn preparation_failed(&self, error: &JsError) -> Result<(), JobFault> {
        self.action.preparation_failed(error)
    }
}
pub(crate) struct ModuleActionRoot {
    hold: ModuleActionHold,
}
impl ModuleActionRoot {
    pub(crate) fn await_root(&self) -> Option<&AwaitRoot> {
        self.hold.await_root()
    }
    pub(crate) fn check(&self) -> JsResult<()> {
        let data = self.hold.action.evaluation.checked_data()?;
        if let Some(cause) = data.interrupted.get() {
            return Err(EngineError::ModuleInterrupted(ModuleInterruption::module(
                cause.clone(),
                self.hold.module.clone(),
            ))
            .into());
        }
        if let Some(fault) = data.fault.get() {
            return Err(fault.into());
        }
        Ok(())
    }
    pub(crate) fn finish(&self, error: &JsError) -> Result<(), JobFault> {
        self.hold.action.finish_failure(error)
    }
}

#[cfg(test)]
pub(crate) mod metadata_cost_tests;
