//! Actual parallel destruction invocations and one cleanup task per generation.
//! Canonical DOM owns every request, captured list, cursor, proof and Work. The
//! original Runtime owns native registration, loop/task and checkpoint custody.
mod style_root;
use super::*;
use surgeist_dom::{
    DomHostedDestructionInvocation, DomHostedDestructionLease, DomHostedDestructionPublication,
    DomHostedDestructionStep, DomHostedOperationId,
};

pub(super) struct State {
    pub invocations: Vec<Invocation>,
    pub cleanups: Vec<Cleanup>,
}
impl State {
    pub fn new() -> Self {
        Self {
            invocations: Vec::new(),
            cleanups: Vec::new(),
        }
    }
    pub fn pending(&self) -> bool {
        !self.invocations.is_empty() || !self.cleanups.is_empty()
    }
    pub(super) fn retain(
        &mut self,
        permission: DomHostedDestructionInvocation,
        history: Option<DomHostedOperationId>,
    ) {
        self.invocations.push(Invocation {
            permission,
            info: None,
            parent: None,
            history,
            history_done: history.is_none(),
            children: Vec::new(),
        });
    }
}

pub(super) struct Invocation {
    permission: DomHostedDestructionInvocation,
    info: Option<Info>,
    parent: Option<DomHostedOperationId>,
    history: Option<DomHostedOperationId>,
    history_done: bool,
    children: Vec<Child>,
}
#[derive(Clone, Copy)]
struct Info {
    operation: DomHostedOperationId,
    cleanup: DomHostedOperationId,
    document: surgeist_dom::DocumentId,
    global: DomHostedGlobalId,
    navigable: DomHostedNavigableId,
    traversable: DomHostedNavigableId,
}
struct Child {
    operation: Option<DomHostedOperationId>,
    settled: bool,
}
pub(super) struct Cleanup {
    lease: DomHostedDestructionLease,
    info: Info,
    publication: Option<DomHostedDestructionPublication>,
    completion: usize,
}

impl HostedState {
    pub(super) fn reserve_invocation(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        if self.destruction.invocations.len() < self.destruction.invocations.capacity() {
            return Ok(());
        }
        let retained = self.retained_bytes(cancel)?;
        let required = self
            .destruction
            .invocations
            .len()
            .checked_add(1)
            .ok_or(DomHostedRuntimeError::Admission)?;
        let requested = required
            .checked_mul(size_of::<Invocation>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if requested > self.limits.scratch_bytes
            || retained
                .checked_add(requested)
                .is_none_or(|n| n > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        for _ in 0..self.destruction.invocations.len() {
            self.step(cancel)?;
        }
        let mut next = Vec::new();
        next.try_reserve_exact(required)
            .map_err(|_| DomHostedRuntimeError::Admission)?;
        let actual = next
            .capacity()
            .checked_mul(size_of::<Invocation>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if actual > self.limits.scratch_bytes
            || retained
                .checked_add(actual)
                .is_none_or(|n| n > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        next.append(&mut self.destruction.invocations);
        self.destruction.invocations = next;
        Ok(())
    }
    fn reserve_cleanup(&mut self, cancel: &AtomicBool) -> Result<(), DomHostedRuntimeError> {
        if self.destruction.cleanups.len() < self.destruction.cleanups.capacity() {
            return Ok(());
        }
        let retained = self.retained_bytes(cancel)?;
        let required = self
            .destruction
            .cleanups
            .len()
            .checked_add(1)
            .ok_or(DomHostedRuntimeError::Admission)?;
        let requested = required
            .checked_mul(size_of::<Cleanup>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if requested > self.limits.scratch_bytes
            || retained
                .checked_add(requested)
                .is_none_or(|n| n > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        for _ in 0..self.destruction.cleanups.len() {
            self.step(cancel)?;
        }
        let mut next = Vec::new();
        next.try_reserve_exact(required)
            .map_err(|_| DomHostedRuntimeError::Admission)?;
        let actual = next
            .capacity()
            .checked_mul(size_of::<Cleanup>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if actual > self.limits.scratch_bytes
            || retained
                .checked_add(actual)
                .is_none_or(|n| n > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        next.append(&mut self.destruction.cleanups);
        self.destruction.cleanups = next;
        Ok(())
    }
    pub(super) fn retain_invocation(&mut self, permission: DomHostedDestructionInvocation) {
        // The caller reserved this header and its transfer before asking the
        // canonical owner for the unique permission. No fallible custody gap.
        self.destruction.retain(permission, None);
    }
    fn initialize_invocation(
        &mut self,
        dom: &DomEventDom<'_>,
        index: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        if self.destruction.invocations[index].info.is_some() {
            return Ok(());
        }
        self.step(cancel)?;
        let request = self.destruction.invocations[index]
            .permission
            .request(dom.raw())
            .map_err(DomHostedRuntimeError::Dom)?;
        let info = Info {
            operation: request.operation(),
            cleanup: request.cleanup_operation(),
            document: request.document(),
            global: request.global(),
            navigable: request.navigable(),
            traversable: request.traversable(),
        };
        let count = request.child_documents().len();
        let retained = self.retained_bytes(cancel)?;
        let requested = count
            .checked_mul(size_of::<Child>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if requested > self.limits.scratch_bytes
            || retained
                .checked_add(requested)
                .is_none_or(|n| n > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        for _ in 0..count {
            self.step(cancel)?;
        }
        let mut children = Vec::new();
        children
            .try_reserve_exact(count)
            .map_err(|_| DomHostedRuntimeError::Admission)?;
        let actual = children
            .capacity()
            .checked_mul(size_of::<Child>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if actual > self.limits.scratch_bytes
            || retained
                .checked_add(actual)
                .is_none_or(|n| n > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        for _ in 0..count {
            children.push(Child {
                operation: None,
                settled: false,
            });
        }
        let row = &mut self.destruction.invocations[index];
        row.children = children;
        row.info = Some(info);
        Ok(())
    }
    fn register_descendants(
        &mut self,
        dom: &mut DomEventDom<'_>,
        traversable: DomHostedNavigableId,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        // Iterative actual continuation registration avoids an unbounded native
        // call stack. Each source begin registers the original captured-index
        // waiter before its child wrapper can queue or disappear.
        let mut index = 0;
        while index < self.destruction.invocations.len() {
            self.step(cancel)?;
            self.initialize_invocation(dom, index, cancel)?;
            let info = self.destruction.invocations[index]
                .info
                .expect("initialized original invocation");
            if info.traversable != traversable {
                index += 1;
                continue;
            }
            let count = self.destruction.invocations[index].children.len();
            for child_index in 0..count {
                self.step(cancel)?;
                if self.destruction.invocations[index].children[child_index]
                    .operation
                    .is_some()
                {
                    continue;
                }
                self.reserve_invocation(cancel)?;
                self.step(cancel)?;
                let Self {
                    source,
                    destruction,
                    ..
                } = self;
                let permission = destruction.invocations[index]
                    .permission
                    .begin_child(dom.raw_mut(), source, child_index, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?;
                let operation = permission.operation();
                destruction.invocations.push(Invocation {
                    permission,
                    info: None,
                    parent: Some(
                        destruction.invocations[index]
                            .info
                            .expect("initialized parent")
                            .operation,
                    ),
                    history: None,
                    history_done: true,
                    children: Vec::new(),
                });
                destruction.invocations[index].children[child_index].operation = Some(operation);
            }
            index += 1;
        }
        Ok(())
    }
}

impl HostedRuntime {
    /// Hand off the named top-level's real parallel destruction invocation.
    /// Retrying the same target retains its original wrapper and captured set.
    /// All execution is serviced by run_next on its actual traversable queue.
    pub fn destroy_top_level(
        &mut self,
        dom: &mut DomEventDom<'_>,
        navigable: DomHostedNavigableId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedDestructionQueued, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        if state.creation.is_some() {
            return Err(DomHostedRuntimeError::OperationPending);
        }
        let mut found = None;
        for index in 0..state.destruction.invocations.len() {
            state.step(cancel)?;
            if state.destruction.invocations[index].permission.navigable() == navigable {
                found = Some(index);
                break;
            }
        }
        let index = if let Some(index) = found {
            index
        } else {
            let (h, t) = state.find_top(navigable, cancel)?;
            state.reserve_invocation(cancel)?;
            state.step(cancel)?;
            let permission = dom
                .raw_mut()
                .begin_hosted_top_level_destruction(&mut state.source, navigable, cancel)
                .map_err(DomHostedRuntimeError::Dom)?;
            state.retain_invocation(permission);
            state.hosts[h].top_levels[t]
                .top
                .as_mut()
                .expect("actual selected top membership")
                .closing = true;
            state.destruction.invocations.len() - 1
        };
        state.initialize_invocation(dom, index, cancel)?;
        let info = state.destruction.invocations[index]
            .info
            .expect("retained actual invocation");
        let (h, _) = state.find_traversal(info.traversable, cancel)?;
        // This actual parallel continuation is now durably queued. The source
        // Global task is enqueued only after real captured-child completion.
        Ok(DomHostedDestructionQueued {
            host: state.hosts[h].id,
            navigable,
            traversable: info.traversable,
            operation: info.operation,
        })
    }
}

impl HostedState {
    fn find_top(
        &mut self,
        navigable: DomHostedNavigableId,
        cancel: &AtomicBool,
    ) -> Result<(usize, usize), DomHostedRuntimeError> {
        for h in 0..self.hosts.len() {
            self.step(cancel)?;
            for t in 0..self.hosts[h].top_levels.len() {
                self.step(cancel)?;
                if self.hosts[h].top_levels[t]
                    .top
                    .as_ref()
                    .is_some_and(|top| top.id == navigable)
                {
                    return Ok((h, t));
                }
            }
        }
        Err(DomHostedRuntimeError::TargetNotFound)
    }
    pub(super) fn reserve_navigation(
        &mut self,
        loop_owner: &Rc<RefCell<records::WindowLoop>>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let count = native_task_count(&self.hosts, &mut self.work, self.limits.work, cancel)?;
        if count >= self.limits.tasks {
            return Err(DomHostedRuntimeError::Admission);
        }
        let required = {
            let event_loop = loop_owner
                .try_borrow()
                .map_err(|_| DomHostedRuntimeError::Busy)?;
            if event_loop.navigation.len() < event_loop.navigation.capacity() {
                return Ok(());
            }
            event_loop
                .navigation
                .len()
                .checked_add(1)
                .ok_or(DomHostedRuntimeError::Admission)?
        };
        let old = self.retained_bytes(cancel)?;
        let requested = required
            .checked_mul(size_of::<records::NavigationTask>())
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
        let mut next = Vec::new();
        next.try_reserve_exact(required)
            .map_err(|_| DomHostedRuntimeError::Admission)?;
        let actual = next
            .capacity()
            .checked_mul(size_of::<records::NavigationTask>())
            .ok_or(DomHostedRuntimeError::Admission)?;
        if actual > self.limits.scratch_bytes
            || old
                .checked_add(actual)
                .is_none_or(|n| n > self.limits.storage_bytes)
        {
            return Err(DomHostedRuntimeError::Admission);
        }
        let mut event_loop = loop_owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        next.append(&mut event_loop.navigation);
        event_loop.navigation = next;
        Ok(())
    }
}

impl State {
    pub(super) fn history_finished(&mut self, operation: DomHostedOperationId) {
        for row in &mut self.invocations {
            if row.history == Some(operation) {
                row.history_done = true;
            }
        }
    }
}

impl HostedState {
    /// Pre-admit release at the original final consumer boundary. The supplied
    /// operation still owns its row until its canonical finish succeeds; every
    /// other admitted continuation in this group must already have released.
    pub(super) fn completion_consumers_finished(
        &mut self,
        dom: &DomEventDom<'_>,
        h: usize,
        t: usize,
        finishing: Option<DomHostedOperationId>,
        cancel: &AtomicBool,
    ) -> Result<bool, DomHostedRuntimeError> {
        let snapshot = dom.snapshot();
        if snapshot.incomplete_source_operation().is_some()
            || snapshot.incomplete_hosted_operation().is_some()
            || !self.children.is_empty()
        {
            return Ok(false);
        }
        let group = self.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        let traversable = group
            .traversal
            .traversable
            .ok_or(DomHostedRuntimeError::Protocol)?;
        for row in &self.destruction.invocations {
            spend(&mut self.work, self.limits.work, cancel)?;
            if Some(row.permission.operation()) != finishing
                && row.info.is_none_or(|info| info.traversable == traversable)
            {
                return Ok(false);
            }
        }
        for row in &self.destruction.cleanups {
            spend(&mut self.work, self.limits.work, cancel)?;
            if row.info.traversable == traversable {
                return Ok(false);
            }
        }
        // A not-yet-expanded C/H still owns its captured future Global jobs.
        for row in &group.traversal.rows {
            spend(&mut self.work, self.limits.work, cancel)?;
            if Some(row.operation) != finishing {
                return Ok(false);
            }
        }
        for cluster in &group.clusters {
            spend(&mut self.work, self.limits.work, cancel)?;
            let event_loop = cluster
                .agent
                .event_loop
                .try_borrow()
                .map_err(|_| DomHostedRuntimeError::Busy)?;
            if !event_loop.navigation.is_empty()
                || event_loop.current.is_some()
                || event_loop.executing
            {
                return Ok(false);
            }
        }
        // Capacity stays in native_retained_bytes. Only obsolete proof values
        // retire, and their entire drop is admitted before irreversible finish.
        for _ in &group.completions {
            spend(&mut self.work, self.limits.work, cancel)?;
        }
        Ok(true)
    }

    fn destruction_proof(
        &mut self,
        h: usize,
        t: usize,
        document: surgeist_dom::DocumentId,
        global: DomHostedGlobalId,
        cancel: &AtomicBool,
    ) -> Result<Option<surgeist_dom::DomHostedDocumentDestructionProof>, DomHostedRuntimeError>
    {
        let Self {
            hosts,
            work,
            limits,
            ..
        } = self;
        let group = hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        for row in &group.completions {
            spend(work, limits.work, cancel)?;
            if row.document == document && row.global == global {
                return Ok(row.proof.clone());
            }
        }
        Ok(None)
    }

    /// Service one real invocation/cleanup continuation. None means that only
    /// its already handed-off history remains, so the same C head may proceed.
    pub(super) fn pump_destruction(
        &mut self,
        dom: &mut DomEventDom<'_>,
        traversable: DomHostedNavigableId,
        required_document: Option<surgeist_dom::DocumentId>,
        cancel: &AtomicBool,
    ) -> Result<Option<DomHostedTaskProgress>, DomHostedRuntimeError> {
        let snapshot = dom.snapshot();
        if snapshot.incomplete_source_operation().is_some()
            || snapshot.incomplete_hosted_operation().is_some()
        {
            for row in &self.destruction.invocations {
                spend(&mut self.work, self.limits.work, cancel)?;
                if row.info.is_some_and(|info| info.traversable == traversable) {
                    return Ok(Some(DomHostedTaskProgress::Pending {
                        operation: row.permission.operation(),
                        reason: DomHostedTaskWait::SourceTail,
                    }));
                }
            }
            return Ok(None);
        }
        self.register_descendants(dom, traversable, cancel)?;
        let (h, t) = self.find_traversal(traversable, cancel)?;
        let mut candidate = None;
        for index in 0..self.destruction.invocations.len() {
            self.step(cancel)?;
            let info = self.destruction.invocations[index]
                .info
                .ok_or(DomHostedRuntimeError::Protocol)?;
            if info.traversable != traversable {
                continue;
            }
            if let Some(document) = required_document {
                let mut ancestor = Some(info.operation);
                let mut relevant = false;
                while let Some(operation) = ancestor {
                    let mut next = None;
                    for row in &self.destruction.invocations {
                        spend(&mut self.work, self.limits.work, cancel)?;
                        if row.info.is_some_and(|value| value.operation == operation) {
                            relevant = row.info.is_some_and(|value| value.document == document);
                            next = row.parent;
                            break;
                        }
                    }
                    if relevant {
                        break;
                    }
                    ancestor = next;
                }
                if !relevant {
                    continue;
                }
            }

            for child_index in 0..self.destruction.invocations[index].children.len() {
                self.step(cancel)?;
                let child = &self.destruction.invocations[index].children[child_index];
                if child.settled {
                    continue;
                }
                let operation = child.operation.ok_or(DomHostedRuntimeError::Protocol)?;
                let mut child_info = None;
                for other in &self.destruction.invocations {
                    spend(&mut self.work, self.limits.work, cancel)?;
                    if other.info.is_some_and(|value| value.operation == operation) {
                        child_info = other.info;
                        break;
                    }
                }
                // A completed wrapper may already have gone. Its captured
                // Document is still available from this original invocation.
                let (document, global) = if let Some(value) = child_info {
                    (value.document, value.global)
                } else {
                    let request = self.destruction.invocations[index]
                        .permission
                        .request(dom.raw())
                        .map_err(DomHostedRuntimeError::Dom)?;
                    let creation = request
                        .child_documents()
                        .get(child_index)
                        .ok_or(DomHostedRuntimeError::Protocol)?;
                    (creation.document(), creation.global())
                };
                if let Some(proof) = self.destruction_proof(h, t, document, global, cancel)? {
                    let Self {
                        source,
                        destruction,
                        ..
                    } = self;
                    destruction.invocations[index]
                        .permission
                        .settle_child(dom.raw_mut(), source, child_index, &proof, cancel)
                        .map_err(DomHostedRuntimeError::Dom)?;
                    destruction.invocations[index].children[child_index].settled = true;
                }
            }
            for _ in 0..self.destruction.invocations[index].children.len() {
                self.step(cancel)?;
            }
            let children_done = self.destruction.invocations[index]
                .children
                .iter()
                .all(|child| child.settled);
            if !children_done {
                continue;
            }
            if let Some(proof) = self.destruction_proof(h, t, info.document, info.global, cancel)? {
                let history_pending = !self.destruction.invocations[index].history_done;
                for _ in 0..self.destruction.cleanups.len() {
                    self.step(cancel)?;
                    self.step(cancel)?;
                }
                let cleanup_pending = self
                    .destruction
                    .cleanups
                    .iter()
                    .any(|row| row.info.cleanup == info.cleanup);
                if cleanup_pending {
                    candidate = self
                        .destruction
                        .cleanups
                        .iter()
                        .position(|row| row.info.cleanup == info.cleanup);
                    break;
                }
                if history_pending {
                    continue;
                }
                for _ in index..self.destruction.invocations.len() {
                    self.step(cancel)?;
                }
                for _ in 0..self.hosts[h].top_levels.len() {
                    self.step(cancel)?;
                }
                for cluster in &self.hosts[h].top_levels[t]
                    .group
                    .as_ref()
                    .ok_or(DomHostedRuntimeError::Protocol)?
                    .clusters
                {
                    spend(&mut self.work, self.limits.work, cancel)?;
                    let _loop = cluster
                        .agent
                        .event_loop
                        .try_borrow()
                        .map_err(|_| DomHostedRuntimeError::Busy)?;
                }
                for _ in 0..self.destruction.invocations.len() {
                    self.step(cancel)?;
                }
                for _ in 0..self.destruction.cleanups.len() {
                    self.step(cancel)?;
                }
                let can_prune =
                    self.destruction
                        .invocations
                        .iter()
                        .enumerate()
                        .all(|(other, row)| {
                            other == index
                                || row
                                    .info
                                    .is_none_or(|value| value.traversable != traversable)
                        })
                        && self
                            .destruction
                            .cleanups
                            .iter()
                            .all(|row| row.info.traversable != traversable);
                let release_completions =
                    self.completion_consumers_finished(dom, h, t, Some(info.operation), cancel)?;
                let Self {
                    source,
                    destruction,
                    hosts,
                    creation,
                    ..
                } = self;
                destruction.invocations[index]
                    .permission
                    .finish(dom.raw_mut(), source, &proof, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?;
                destruction.invocations.remove(index);
                if release_completions {
                    hosts[h].top_levels[t]
                        .group
                        .as_mut()
                        .expect("prevalidated actual group")
                        .completions
                        .clear();
                }
                if can_prune
                    && prune_closed_composition(&mut hosts[h], t)
                    && let Some(creator) = creation.as_mut()
                    && creator.host == hosts[h].id
                    && creator.composition > t
                {
                    creator.composition -= 1;
                }
                return Ok(Some(DomHostedTaskProgress::Complete {
                    operation: info.operation,
                }));
            }
            let mut cleanup_index = None;
            for (other, row) in self.destruction.cleanups.iter().enumerate() {
                spend(&mut self.work, self.limits.work, cancel)?;
                if row.info.cleanup == info.cleanup {
                    cleanup_index = Some(other);
                    break;
                }
            }
            let cleanup_index = if let Some(value) = cleanup_index {
                value
            } else {
                self.reserve_cleanup(cancel)?;
                let completion =
                    self.reserve_completion(h, t, info.document, info.global, cancel)?;
                self.step(cancel)?;
                let Self {
                    source,
                    destruction,
                    hosts,
                    ..
                } = self;
                let lease = destruction.invocations[index]
                    .permission
                    .cleanup(dom.raw_mut(), source, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?;
                if let Some(proof) = lease.destruction_proof() {
                    hosts[h].top_levels[t]
                        .group
                        .as_mut()
                        .ok_or(DomHostedRuntimeError::Protocol)?
                        .completions[completion]
                        .proof = Some(proof.clone());
                    continue;
                }
                let value = destruction.cleanups.len();
                destruction.cleanups.push(Cleanup {
                    lease,
                    info,
                    publication: None,
                    completion,
                });
                value
            };
            let request = match self.destruction.cleanups[cleanup_index]
                .lease
                .step(dom.raw())
                .map_err(DomHostedRuntimeError::Dom)?
            {
                DomHostedDestructionStep::EnqueueParentTask(request)
                | DomHostedDestructionStep::Cleanup(request) => Some(request),
                DomHostedDestructionStep::Complete => None,
            };
            let mut ready = true;
            if let Some(request) = request {
                for creation in request.child_documents() {
                    self.step(cancel)?;
                    if self
                        .destruction_proof(h, t, creation.document(), creation.global(), cancel)?
                        .is_none()
                    {
                        ready = false;
                        break;
                    }
                }
            }
            if !ready {
                continue;
            }
            candidate = Some(cleanup_index);
            break;
        }
        let Some(index) = candidate else {
            for row in &self.destruction.invocations {
                spend(&mut self.work, self.limits.work, cancel)?;
                if row.info.is_some_and(|info| info.traversable == traversable) {
                    return Ok(Some(DomHostedTaskProgress::Pending {
                        operation: row.permission.operation(),
                        reason: DomHostedTaskWait::Descendants,
                    }));
                }
            }
            return Ok(None);
        };
        self.pump_cleanup(dom, index, cancel).map(Some)
    }

    fn pump_cleanup(
        &mut self,
        dom: &mut DomEventDom<'_>,
        mut index: usize,
        cancel: &AtomicBool,
    ) -> Result<DomHostedTaskProgress, DomHostedRuntimeError> {
        let info = self.destruction.cleanups[index].info;
        let (h, t, c, _) = self.find_global(info.global, cancel, || Ok(()))?;
        let loop_owner = self.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .clusters[c]
            .agent
            .event_loop
            .clone();
        // The first queued lifecycle body on this actual Global loop is a
        // predecessor of the requested cleanup, not a later traversable C.
        // Service that real FIFO predecessor before the requested body.
        let preceding = {
            let event_loop = loop_owner
                .try_borrow()
                .map_err(|_| DomHostedRuntimeError::Busy)?;
            let mut found = None;
            for task in &event_loop.navigation {
                self.step(cancel)?;
                if let records::NavigationTask::Destroy(task) = task {
                    found = Some(task.operation);
                    break;
                }
            }
            found
        };
        if let Some(operation) = preceding {
            for (other, row) in self.destruction.cleanups.iter().enumerate() {
                spend(&mut self.work, self.limits.work, cancel)?;
                if row.info.cleanup == operation {
                    index = other;
                    break;
                }
            }
        }
        let info = self.destruction.cleanups[index].info;
        let (selected_h, selected_t, selected_c, _) =
            self.find_global(info.global, cancel, || Ok(()))?;
        let selected_loop = &self.hosts[selected_h].top_levels[selected_t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .clusters[selected_c]
            .agent
            .event_loop;
        if !Rc::ptr_eq(&loop_owner, selected_loop) {
            return Err(DomHostedRuntimeError::Binding);
        }
        let enqueue = matches!(
            self.destruction.cleanups[index]
                .lease
                .step(dom.raw())
                .map_err(DomHostedRuntimeError::Dom)?,
            DomHostedDestructionStep::EnqueueParentTask(_)
        );
        if enqueue {
            let present = {
                let event_loop = loop_owner
                    .try_borrow()
                    .map_err(|_| DomHostedRuntimeError::Busy)?;
                for _ in 0..event_loop.navigation.len() {
                    self.step(cancel)?;
                }
                event_loop.navigation.iter().any(|task|matches!(task,records::NavigationTask::Destroy(task) if task.operation==info.cleanup))
            };
            if !present {
                self.reserve_navigation(&loop_owner, cancel)?;
                self.step(cancel)?;
                loop_owner
                    .try_borrow_mut()
                    .map_err(|_| DomHostedRuntimeError::Busy)?
                    .navigation
                    .push(records::NavigationTask::Destroy(records::DestroyTask {
                        operation: info.cleanup,
                        document: info.document,
                        global: info.global,
                        phase: records::DestroyPhase::Queued,
                    }));
            }
            let Self {
                source,
                destruction,
                ..
            } = self;
            let row = &mut destruction.cleanups[index];
            if row.publication.is_none() {
                let DomHostedDestructionStep::EnqueueParentTask(request) = row
                    .lease
                    .step(dom.raw())
                    .map_err(DomHostedRuntimeError::Dom)?
                else {
                    return Err(DomHostedRuntimeError::Protocol);
                };
                row.publication = Some(
                    source
                        .seal_destruction_queued(request, cancel)
                        .map_err(DomHostedRuntimeError::Dom)?,
                );
            }
            row.lease
                .accept_publication(dom.raw_mut(), source, &mut row.publication, cancel)
                .map_err(DomHostedRuntimeError::Dom)?;
            return Ok(DomHostedTaskProgress::Pending {
                operation: info.operation,
                reason: DomHostedTaskWait::Descendants,
            });
        }
        let mut event_loop = loop_owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let current = records::CurrentTask::Destroy {
            operation: info.cleanup,
            document: info.document,
        };
        if !event_loop.executing
            && let Some(records::CurrentTask::ElementTask { task, .. }) = event_loop.current
        {
            let details = self
                .element_tasks
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            details
                .ingest_source(dom, &self.source, cancel)
                .map_err(manipulation_error)?;
            details
                .cancel_document(dom, info.document, cancel)
                .map_err(manipulation_error)?;
            if !details.has_task(task).map_err(manipulation_error)? {
                checkpoint(dom, &mut self.work, self.limits.work, cancel)?;
                event_loop.current = None;
                event_loop.checkpoints = event_loop.next_checkpoint;
            }
        }
        if event_loop.executing || event_loop.current.is_some_and(|old| old != current) {
            return Ok(DomHostedTaskProgress::Pending {
                operation: info.operation,
                reason: DomHostedTaskWait::CurrentTask,
            });
        }
        let mut task_index = None;
        for (position, task) in event_loop.navigation.iter().enumerate() {
            self.step(cancel)?;
            if matches!(task,records::NavigationTask::Destroy(task) if task.operation==info.cleanup)
            {
                task_index = Some(position);
                break;
            }
        }
        let task_index = task_index.ok_or(DomHostedRuntimeError::Protocol)?;
        let actual = event_loop.navigation[task_index]
            .destroy()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        if actual.document != info.document || actual.global != info.global {
            return Err(DomHostedRuntimeError::Binding);
        }

        // A cleanup task can be selected while its traversal waits on an
        // inactive history job. It never executes or acknowledges that job.
        if event_loop.current.is_none() {
            self.step(cancel)?;
            event_loop.next_checkpoint = event_loop
                .checkpoints
                .checked_add(1)
                .ok_or(DomHostedRuntimeError::Admission)?;
            event_loop.current = Some(current);
            event_loop.navigation[task_index]
                .destroy_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .phase = records::DestroyPhase::Cleanup;
        }
        let complete = matches!(
            self.destruction.cleanups[index]
                .lease
                .step(dom.raw())
                .map_err(DomHostedRuntimeError::Dom)?,
            DomHostedDestructionStep::Complete
        );
        if !complete {
            let details = self
                .element_tasks
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            details
                .ingest_source(dom, &self.source, cancel)
                .map_err(manipulation_error)?;
            if details
                .cancel_document(dom, info.document, cancel)
                .map_err(manipulation_error)?
            {
                return Ok(DomHostedTaskProgress::Pending {
                    operation: info.operation,
                    reason: DomHostedTaskWait::CurrentTask,
                });
            }
            let Self {
                source,
                destruction,
                hosts,
                ..
            } = self;
            let row = &mut destruction.cleanups[index];
            if row.publication.is_none() {
                let DomHostedDestructionStep::Cleanup(request) = row
                    .lease
                    .step(dom.raw())
                    .map_err(DomHostedRuntimeError::Dom)?
                else {
                    return Err(DomHostedRuntimeError::Protocol);
                };
                row.publication = Some(
                    source
                        .seal_top_level_cleanup(request, cancel)
                        .map_err(DomHostedRuntimeError::Dom)?,
                );
            }
            row.lease
                .accept_publication(dom.raw_mut(), source, &mut row.publication, cancel)
                .map_err(DomHostedRuntimeError::Dom)?;
            let proof = row
                .lease
                .destruction_proof()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .clone();
            hosts[h].top_levels[t]
                .group
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .completions[row.completion]
                .proof = Some(proof);
        }
        if event_loop.navigation[task_index]
            .destroy()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .phase
            != records::DestroyPhase::CheckpointComplete
        {
            match checkpoint(dom, &mut self.work, self.limits.work, cancel) {
                Ok(()) => {}
                Err(DomHostedRuntimeError::CheckpointUnavailable) => {
                    return Ok(DomHostedTaskProgress::Pending {
                        operation: info.operation,
                        reason: DomHostedTaskWait::CheckpointUnavailable,
                    });
                }
                Err(error) => return Err(error),
            }
            event_loop.navigation[task_index]
                .destroy_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .phase = records::DestroyPhase::CheckpointComplete;
        }
        // All native movement is admitted before explicit canonical close.
        for cluster in &self.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .clusters
        {
            spend(&mut self.work, self.limits.work, cancel)?;
            for _ in 0..cluster.agent.globals.len() {
                spend(&mut self.work, self.limits.work, cancel)?;
            }
            // The actual Window phase mark and ordered retained-row compaction
            // both execute only after this original canonical close succeeds.
            for _ in 0..cluster.agent.windows.rows.len() {
                spend(&mut self.work, self.limits.work, cancel)?;
                spend(&mut self.work, self.limits.work, cancel)?;
            }
        }
        for _ in 0..event_loop.navigation.len() {
            self.step(cancel)?;
        }
        for _ in index..self.destruction.cleanups.len() {
            self.step(cancel)?;
        }
        self.destruction.cleanups[index]
            .lease
            .finish(dom.raw_mut(), &mut self.source, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        event_loop.navigation.remove(task_index);
        event_loop.current = None;
        event_loop.checkpoints = event_loop.next_checkpoint;
        drop(event_loop);
        retire_native_top(
            &mut self.hosts[h],
            t,
            info.global,
            info.cleanup,
            info.navigable,
        );
        self.destruction.cleanups.remove(index);
        Ok(DomHostedTaskProgress::Executed {
            operation: info.cleanup,
            document: info.document,
            global: info.global,
            kind: DomHostedTaskKind::Destruction,
        })
    }
}

fn checkpoint(
    dom: &DomEventDom<'_>,
    work: &mut u64,
    limit: u64,
    cancel: &AtomicBool,
) -> Result<(), DomHostedRuntimeError> {
    spend(work, limit, cancel)?;
    let snapshot = dom.snapshot();
    if snapshot.incomplete_source_operation().is_some()
        || snapshot.incomplete_hosted_operation().is_some()
        || !snapshot.pending_slot_signals().is_empty()
    {
        return Err(DomHostedRuntimeError::CheckpointUnavailable);
    }
    Ok(())
}
fn manipulation_error(
    error: crate::element_tasks::DomElementTaskRuntimeError,
) -> DomHostedRuntimeError {
    use crate::element_tasks::DomElementTaskRuntimeError as Error;
    match error {
        Error::Hosted(error) => error,
        Error::Dom(error) => DomHostedRuntimeError::Dom(error.into()),
        Error::Busy => DomHostedRuntimeError::Busy,
        Error::OwnerDisposed => DomHostedRuntimeError::OwnerDisposed,
        Error::Admission | Error::IdentityExhausted => DomHostedRuntimeError::Admission,
        Error::Binding
        | Error::OutOfOrderTransfer
        | Error::PretransferredSession
        | Error::Dispatch(_)
        | Error::SourceEvent(_) => DomHostedRuntimeError::Protocol,
    }
}
/// One actual Document/Global generation's cleanup evidence. A reserved row is
/// not a completion: only the canonical cleanup issuer can fill its proof.
pub(super) struct Completion {
    pub document: surgeist_dom::DocumentId,
    pub global: DomHostedGlobalId,
    pub proof: Option<surgeist_dom::DomHostedDocumentDestructionProof>,
}

impl HostedState {
    pub(super) fn reserve_completion(
        &mut self,
        h: usize,
        t: usize,
        document: surgeist_dom::DocumentId,
        global: DomHostedGlobalId,
        cancel: &AtomicBool,
    ) -> Result<usize, DomHostedRuntimeError> {
        {
            let HostedState {
                hosts,
                work,
                limits,
                ..
            } = self;
            let rows = &hosts[h].top_levels[t]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .completions;
            for (index, row) in rows.iter().enumerate() {
                spend(work, limits.work, cancel)?;
                if row.document == document {
                    if row.global != global {
                        return Err(DomHostedRuntimeError::Binding);
                    }
                    return Ok(index);
                }
            }
        }
        let retained = self.retained_bytes(cancel)?;
        let HostedState {
            hosts,
            work,
            limits,
            ..
        } = self;
        let rows = &mut hosts[h].top_levels[t]
            .group
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .completions;
        if rows.len() == rows.capacity() {
            let required = rows
                .len()
                .checked_add(1)
                .ok_or(DomHostedRuntimeError::Admission)?;
            let requested = required
                .checked_mul(size_of::<Completion>())
                .ok_or(DomHostedRuntimeError::Admission)?;
            if requested > limits.scratch_bytes
                || retained
                    .checked_add(requested)
                    .is_none_or(|bytes| bytes > limits.storage_bytes)
            {
                return Err(DomHostedRuntimeError::Admission);
            }
            for _ in 0..rows.len() {
                spend(work, limits.work, cancel)?;
            }
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(required)
                .map_err(|_| DomHostedRuntimeError::Admission)?;
            let actual = replacement
                .capacity()
                .checked_mul(size_of::<Completion>())
                .ok_or(DomHostedRuntimeError::Admission)?;
            if actual > limits.scratch_bytes
                || retained
                    .checked_add(actual)
                    .is_none_or(|bytes| bytes > limits.storage_bytes)
            {
                return Err(DomHostedRuntimeError::Admission);
            }
            replacement.append(rows);
            *rows = replacement;
        }
        spend(work, limits.work, cancel)?;
        let index = rows.len();
        rows.push(Completion {
            document,
            global,
            proof: None,
        });
        Ok(index)
    }

    pub(super) fn settle_destroyed_history_job(
        &mut self,
        dom: &mut DomEventDom<'_>,
        h: usize,
        t: usize,
        job: history::Job,
        cancel: &AtomicBool,
    ) -> Result<bool, DomHostedRuntimeError> {
        // The issued job belongs to this actual group's original traversal.
        // No Global/current-Document query precedes exact completion evidence.
        let HostedState {
            hosts,
            source,
            work,
            limits,
            ..
        } = self;
        let group = hosts[h].top_levels[t]
            .group
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        let mut proof = None;
        for row in &group.completions {
            spend(work, limits.work, cancel)?;
            if row.document == job.document && row.global == job.global {
                proof = row.proof.as_ref();
                break;
            }
        }
        let Some(proof) = proof else {
            return Ok(false);
        };
        // Pre-admit actual Global FIFO removals before consuming the canonical
        // settlement. The group exclusively owns these loops outside callbacks.
        for cluster in &group.clusters {
            spend(work, limits.work, cancel)?;
            let event_loop = cluster
                .agent
                .event_loop
                .try_borrow()
                .map_err(|_| DomHostedRuntimeError::Busy)?;
            for _ in 0..event_loop.navigation.len() {
                spend(work, limits.work, cancel)?;
            }
        }
        let row = group
            .traversal
            .rows
            .first_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        if row.operation != job.operation {
            return Err(DomHostedRuntimeError::Protocol);
        }
        row.continuation
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .settle_destroyed_job(dom.raw_mut(), source, proof, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        for cluster in &group.clusters {
            let mut event_loop = cluster.agent.event_loop.borrow_mut();
            event_loop.navigation.retain(|task| {
                !matches!(task,
                records::NavigationTask::History(value)
                if value.operation == job.operation && value.occurrence == job.occurrence)
            });
        }
        Ok(true)
    }
}

// Every scan/move below was pre-admitted under the same exclusive state borrow.
// Immutable completion evidence alone does not keep a closed group alive.
fn retire_native_top(
    host: &mut records::Host,
    index: usize,
    global: DomHostedGlobalId,
    operation: surgeist_dom::DomHostedOperationId,
    navigable: DomHostedNavigableId,
) {
    let composition = &mut host.top_levels[index];
    if composition
        .top
        .as_ref()
        .is_some_and(|top| top.id == navigable)
    {
        composition.top = None;
    }
    let group = composition.group.as_mut().expect("actual original group");
    for cluster in &mut group.clusters {
        cluster.agent.globals.retain(|value| value.id != global);
        for window in &mut cluster.agent.windows.rows {
            if window.id == global {
                window.retired = true;
            }
        }
        cluster
            .agent
            .windows
            .rows
            .retain(|window| !window.retired || window.needed());
        let mut event_loop = cluster.agent.event_loop.borrow_mut();
        event_loop.navigation.retain(|task| {
            !matches!(task,
            records::NavigationTask::Destroy(value) if value.operation == operation)
        });
        if matches!(event_loop.current,Some(records::CurrentTask::Destroy {operation:old,..}) if old==operation)
        {
            event_loop.current = None;
            event_loop.checkpoints = event_loop.next_checkpoint;
        }
    }
    // The original wrappers and queued H may still need this group/proof.
}

pub(super) fn prune_closed_composition(host: &mut records::Host, index: usize) -> bool {
    let composition = &host.top_levels[index];
    if composition.top.is_some() {
        return false;
    }
    let group = composition.group.as_ref().expect("actual retained group");
    let empty = group.traversal.rows.is_empty()
        && group.fullscreen.rows.is_empty()
        && group.clusters.iter().all(|cluster| {
            let event_loop = cluster.agent.event_loop.borrow();
            cluster.agent.globals.is_empty()
                && cluster.agent.windows.rows.is_empty()
                && event_loop.navigation.is_empty()
                && event_loop.current.is_none()
                && !event_loop.executing
        });
    if empty {
        host.top_levels.remove(index);
    }
    empty
}

impl State {
    pub(super) fn retained_bytes(
        &self,
        work: &mut u64,
        limit: u64,
        cancel: &AtomicBool,
        admit: &mut impl FnMut() -> Result<(), DomHostedRuntimeError>,
    ) -> Result<usize, DomHostedRuntimeError> {
        let mut bytes = self
            .invocations
            .capacity()
            .checked_mul(size_of::<Invocation>())
            .and_then(|n| {
                n.checked_add(self.cleanups.capacity().checked_mul(size_of::<Cleanup>())?)
            })
            .ok_or(DomHostedRuntimeError::Admission)?;
        for invocation in &self.invocations {
            spend(work, limit, cancel)?;
            admit()?;
            bytes = bytes
                .checked_add(
                    invocation
                        .children
                        .capacity()
                        .checked_mul(size_of::<Child>())
                        .ok_or(DomHostedRuntimeError::Admission)?,
                )
                .ok_or(DomHostedRuntimeError::Admission)?;
        }
        Ok(bytes)
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
mod completion_retention_tests {
    use super::*;
    use crate::{
        DomElementTaskRuntimeLimits, DomSession, DomSessionLimits, DomSourceDriveProgress,
        DomSourceServices, DomStyleRuntimeLimits,
    };
    use surgeist_dom::{
        Dom, DomBatch, DomHostedLimits, DomHostedRecordLimits, DomLimits, DomSourceEdit,
        DomSourceLimits, Namespace, NodeId,
    };

    fn fixture() -> (DomSession, HostedRuntime, DomHostedTopLevel, NodeId, NodeId) {
        let cancel = AtomicBool::new(false);
        let mut session = DomSession::new(
            Dom::new(
                DomLimits::new(
                    256,
                    1_000_000,
                    512,
                    512,
                    512,
                    32,
                    4_000_000,
                    surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
            DomSessionLimits {
                source_events: crate::DomSourceEventRuntimeLimits {
                    jobs: 0,
                    storage_bytes: 16_000_000,
                    work: 1_000_000,
                },
                style: DomStyleRuntimeLimits {
                    roots: 8,
                    completions: 32,
                    reports: 32,
                    storage_bytes: 32_000_000,
                    work: 4_000_000,
                    css_input: surgeist_css::CssUtf16InputLimits::default(),
                },
                element_tasks: DomElementTaskRuntimeLimits {
                    tasks: 32,
                    storage_bytes: 32_000_000,
                    work: 4_000_000,
                },
                hosted: DomHostedRuntimeLimits {
                    canonical: DomHostedLimits::new(
                        DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                        32_000_000,
                        32_000_000,
                        4_000_000,
                    ),
                    tasks: 32,
                    storage_bytes: 32_000_000,
                    scratch_bytes: 32_000_000,
                    work: 4_000_000,
                },
            },
        )
        .unwrap();
        let mut hosted = session.hosted_execution();
        let top = {
            let mut dom = session.dom();
            let host = hosted.create_host(&mut dom, &cancel).unwrap();
            hosted.create_top_level(&mut dom, host, &cancel).unwrap()
        };
        let snapshot = session.snapshot();
        let html = snapshot.node(top.document().node()).unwrap().children()[0];
        let body = snapshot.node(html).unwrap().children()[1];
        let mut batch = DomBatch::new();
        let closed = batch.create_html_document(top.document(), None);
        let iframe = batch.create_element_ns(&closed, Namespace::html(), "iframe");
        let commit = session.dom().apply(batch, &cancel).unwrap();
        let iframe = commit.created(&iframe).unwrap().unwrap();
        (session, hosted, top, body, iframe)
    }

    fn edit(session: &mut DomSession, edit: DomSourceEdit) {
        let mut source = session.source_execution();
        assert!(matches!(
            source
                .apply(
                    &mut session.dom(),
                    edit,
                    DomSourceLimits::new(32_000_000, 1_000_000, 20_000_000, 256, 4096, 4096, 1),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &AtomicBool::new(false),
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
    }

    fn drain(
        session: &mut DomSession,
        hosted: &mut HostedRuntime,
        top: DomHostedTopLevel,
    ) -> Result<usize, DomHostedRuntimeError> {
        let mut destroyed = 0;
        for _ in 0..1024 {
            match hosted.run_next(&mut session.dom(), top.navigable(), &AtomicBool::new(false))? {
                DomHostedTaskProgress::Idle => return Ok(destroyed),
                DomHostedTaskProgress::Executed {
                    kind: DomHostedTaskKind::Destruction,
                    ..
                } => destroyed += 1,
                _ => {}
            }
        }
        panic!("the actual child execution closure did not settle within 1024 steps");
    }

    #[test]
    fn settled_child_releases_proof_when_live_group_has_no_admitted_consumers() {
        let (mut session, mut hosted, top, body, iframe) = fixture();
        edit(
            &mut session,
            DomSourceEdit::insert_before(iframe, body, None),
        );
        assert_eq!(drain(&mut session, &mut hosted, top).unwrap(), 0);
        edit(&mut session, DomSourceEdit::detach(iframe));
        assert_eq!(drain(&mut session, &mut hosted, top).unwrap(), 1);
        let owner = hosted.state().unwrap();
        let state = owner.borrow();
        let group = state.hosts[0].top_levels[0].group.as_ref().unwrap();
        assert!(state.hosts[0].top_levels[0].top.is_some());
        assert!(state.destruction.invocations.is_empty());
        assert!(state.destruction.cleanups.is_empty());
        assert!(group.traversal.rows.is_empty());
        assert!(
            group.completions.is_empty(),
            "a live parent must release proof after the child's actual execution closure"
        );
    }

    #[test]
    fn sequential_settled_children_reuse_one_completion_capacity_under_finite_scratch() {
        let (mut session, mut hosted, top, body, iframe) = fixture();
        edit(
            &mut session,
            DomSourceEdit::insert_before(iframe, body, None),
        );
        drain(&mut session, &mut hosted, top).unwrap();
        edit(&mut session, DomSourceEdit::detach(iframe));
        assert_eq!(drain(&mut session, &mut hosted, top).unwrap(), 1);
        let owner = hosted.state().unwrap();
        let (capacity, mut limits) = {
            let state = owner.borrow();
            let group = state.hosts[0].top_levels[0].group.as_ref().unwrap();
            (group.completions.capacity(), state.limits)
        };
        assert_eq!(
            capacity, 1,
            "the actual first cleanup reserved one proof slot"
        );
        // All first-round task/continuation banks remain allocated. The next
        // child reaches the same real cleanup with no new concurrent consumer;
        // finite scratch admits one proof, but cannot grow an obsolete history.
        for _ in 0..2 {
            edit(
                &mut session,
                DomSourceEdit::insert_before(iframe, body, None),
            );
            drain(&mut session, &mut hosted, top).unwrap();
            edit(&mut session, DomSourceEdit::detach(iframe));
            limits.scratch_bytes = capacity * size_of::<Completion>();
            hosted
                .set_limits(&mut session.dom(), limits, &AtomicBool::new(false))
                .unwrap();
            assert_eq!(
                drain(&mut session, &mut hosted, top)
                    .expect("settled earlier children must not refuse a new single cleanup"),
                1
            );
            let state = owner.borrow();
            let group = state.hosts[0].top_levels[0].group.as_ref().unwrap();
            assert_eq!(group.completions.capacity(), capacity);
            assert!(state.hosts[0].top_levels[0].top.is_some());
            drop(state);
            limits.scratch_bytes = 32_000_000;
            hosted
                .set_limits(&mut session.dom(), limits, &AtomicBool::new(false))
                .unwrap();
        }
        assert!(
            session
                .snapshot()
                .document(top.document())
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
    }

    mod late_registration_tests;
}
