//! The original traversable's FIFO execution custody. Canonical DOM alone owns
//! Nav/Entry/DocumentState, attached and retained history collections and steps.
use super::*;
use surgeist_dom::{DomHostedOperationId, DomHostedQueuedHistory};

pub(super) struct Traversal {
    pub traversable: Option<DomHostedNavigableId>,
    pub rows: Vec<Queued>,
}
impl Traversal {
    pub fn new() -> Self {
        Self {
            traversable: None,
            rows: Vec::new(),
        }
    }
}

/// A real FIFO task may be pre-admitted before its canonical handoff. Only an
/// accepted non-Clone original continuation makes its body runnable.
pub(super) struct Queued {
    pub operation: DomHostedOperationId,
    pub initiating_loop: Rc<RefCell<records::WindowLoop>>,
    pub continuation: Option<DomHostedQueuedHistory>,
    pub publication: Option<surgeist_dom::DomHostedChildHistoryPublication>,
    pub removal: Option<surgeist_dom::DomHostedChildRemovalPublication>,
}

pub(super) fn reserve_traversal(
    queue: &mut Traversal,
    retained: usize,
    limits: DomHostedRuntimeLimits,
    work: &mut u64,
    cancel: &AtomicBool,
) -> Result<usize, DomHostedRuntimeError> {
    spend(work, limits.work, cancel)?;
    if queue.rows.len() < queue.rows.capacity() {
        return Ok(0);
    }
    let required = queue
        .rows
        .len()
        .checked_add(1)
        .ok_or(DomHostedRuntimeError::Admission)?;
    let requested = required
        .checked_mul(size_of::<Queued>())
        .ok_or(DomHostedRuntimeError::Admission)?;
    if requested > limits.scratch_bytes
        || retained
            .checked_add(requested)
            .is_none_or(|bytes| bytes > limits.storage_bytes)
    {
        return Err(DomHostedRuntimeError::Admission);
    }
    for _ in 0..queue.rows.len() {
        spend(work, limits.work, cancel)?;
    }
    let mut replacement = Vec::new();
    replacement
        .try_reserve_exact(required)
        .map_err(|_| DomHostedRuntimeError::Admission)?;
    let actual = replacement
        .capacity()
        .checked_mul(size_of::<Queued>())
        .ok_or(DomHostedRuntimeError::Admission)?;
    if actual > limits.scratch_bytes
        || retained
            .checked_add(actual)
            .is_none_or(|bytes| bytes > limits.storage_bytes)
    {
        return Err(DomHostedRuntimeError::Admission);
    }
    let old = queue
        .rows
        .capacity()
        .checked_mul(size_of::<Queued>())
        .ok_or(DomHostedRuntimeError::Admission)?;
    let added = actual
        .checked_sub(old)
        .ok_or(DomHostedRuntimeError::Admission)?;
    replacement.append(&mut queue.rows);
    queue.rows = replacement;
    Ok(added)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Job {
    pub operation: DomHostedOperationId,
    pub occurrence: u64,
    pub document: surgeist_dom::DocumentId,
    pub global: DomHostedGlobalId,
}
impl Job {
    fn from_request(request: &surgeist_dom::DomHostedHistoryJobRequest) -> Self {
        Self {
            operation: request.operation(),
            occurrence: request.occurrence(),
            document: request.document(),
            global: request.global(),
        }
    }
}

impl HostedState {
    pub(super) fn find_traversal(
        &mut self,
        traversable: DomHostedNavigableId,
        cancel: &AtomicBool,
    ) -> Result<(usize, usize), DomHostedRuntimeError> {
        for h in 0..self.hosts.len() {
            self.step(cancel)?;
            for t in 0..self.hosts[h].top_levels.len() {
                self.step(cancel)?;
                if self.hosts[h].top_levels[t]
                    .group
                    .as_ref()
                    .is_some_and(|group| group.traversal.traversable == Some(traversable))
                {
                    return Ok((h, t));
                }
            }
        }
        Err(DomHostedRuntimeError::TargetNotFound)
    }
}

impl HostedRuntime {
    /// Pump the actual traversable FIFO head, or one Global task required by
    /// that head. A running Details task/checkpoint is never pumped recursively.
    /// Refusal preserves the same original continuation, write receipt and Work.
    pub fn run_next(
        &mut self,
        dom: &mut DomEventDom<'_>,
        traversable: DomHostedNavigableId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedTaskProgress, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        let (h, t) = state.find_traversal(traversable, cancel)?;
        let empty = state.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .traversal
            .rows
            .is_empty();
        if empty && let Some(progress) = state.pump_destruction(dom, traversable, None, cancel)? {
            return Ok(progress);
        }
        let operation = {
            let group = state.hosts[h].top_levels[t]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            let Some(row) = group.traversal.rows.first() else {
                // The retained execution record alone never certifies current
                // identity when reporting an idle traversable.
                let current = dom
                    .raw()
                    .hosted_navigable(traversable)
                    .map_err(DomHostedRuntimeError::Dom)?;
                if current.parent().is_some() || current.traversable() != traversable {
                    return Err(DomHostedRuntimeError::Binding);
                }
                if state.completion_consumers_finished(dom, h, t, None, cancel)? {
                    state.hosts[h].top_levels[t]
                        .group
                        .as_mut()
                        .expect("prevalidated actual group")
                        .completions
                        .clear();
                }
                return Ok(DomHostedTaskProgress::Idle);
            };
            row.operation
        };
        state.step(cancel)?;
        {
            let group = state.hosts[h].top_levels[t]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            let initiating = group.traversal.rows[0]
                .initiating_loop
                .try_borrow()
                .map_err(|_| DomHostedRuntimeError::Busy)?;
            let same_history = matches!(initiating.current,
                Some(records::CurrentTask::History { operation: old, .. }) if old == operation);
            if initiating.executing || (initiating.current.is_some() && !same_history) {
                let dependency = if initiating.executing {
                    None
                } else {
                    match initiating.current {
                        Some(records::CurrentTask::Destroy { document, .. })
                        | Some(records::CurrentTask::DomManipulation { document, .. }) => {
                            Some(document)
                        }
                        _ => None,
                    }
                };
                drop(initiating);
                if let Some(document) = dependency
                    && let Some(progress) =
                        state.pump_destruction(dom, traversable, Some(document), cancel)?
                {
                    return Ok(progress);
                }
                return Ok(DomHostedTaskProgress::Pending {
                    operation,
                    reason: DomHostedTaskWait::CurrentTask,
                });
            }
        }
        let snapshot = dom.snapshot();
        if snapshot.incomplete_source_operation().is_some()
            || snapshot.incomplete_hosted_operation().is_some()
        {
            return Ok(DomHostedTaskProgress::Pending {
                operation,
                reason: DomHostedTaskWait::SourceTail,
            });
        }
        // No callback can grow this FIFO while the original state borrow is
        // held. Its exact existing length bounds a potential final head move.
        let row_moves = state.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .traversal
            .rows
            .len();
        for _ in 0..row_moves {
            state.step(cancel)?;
        }
        {
            let HostedState {
                hosts,
                work,
                limits,
                ..
            } = &mut *state;
            for _ in 0..hosts[h].top_levels.len() {
                spend(work, limits.work, cancel)?;
            }
            let group = hosts[h].top_levels[t]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            for cluster in &group.clusters {
                spend(work, limits.work, cancel)?;
                // Final closed-group pruning only examines these headers; no
                // unrelated canonical collection or later live-list is rebuilt.
                let event_loop = cluster
                    .agent
                    .event_loop
                    .try_borrow()
                    .map_err(|_| DomHostedRuntimeError::Busy)?;
                let _actual = (
                    cluster.agent.globals.len(),
                    event_loop.navigation.len(),
                    event_loop.current,
                    event_loop.executing,
                );
            }
        }
        let job = {
            let HostedState { hosts, source, .. } = &mut *state;
            let row = &mut hosts[h].top_levels[t]
                .group
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .traversal
                .rows[0];
            let continuation = row
                .continuation
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            match dom
                .raw_mut()
                .resume_queued_history(source, continuation, cancel)
                .map_err(DomHostedRuntimeError::Dom)?
            {
                surgeist_dom::DomHostedHistoryStep::Job(request) => {
                    Some(Job::from_request(request))
                }
                surgeist_dom::DomHostedHistoryStep::Complete => None,
            }
        };
        let Some(job) = job else {
            for _ in 0..state.destruction.invocations.len() {
                state.step(cancel)?;
            }
            let release_completions =
                state.completion_consumers_finished(dom, h, t, Some(operation), cancel)?;
            let HostedState {
                hosts,
                source,
                destruction,
                ..
            } = &mut *state;
            let group = hosts[h].top_levels[t]
                .group
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            let rows = &mut group.traversal.rows;
            rows[0]
                .continuation
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .finish(dom.raw_mut(), source, cancel)
                .map_err(DomHostedRuntimeError::Dom)?;
            rows.remove(0);
            destruction.history_finished(operation);
            if release_completions {
                group.completions.clear();
            }
            // A destruction wrapper can still retain this actual group/proof.
            // Its final cleanup performs the eventual empty composition close.
            return Ok(DomHostedTaskProgress::Complete { operation });
        };
        // Exact generation completion is inspected before any current Window or
        // Document authority. An inactive-but-undestroyed Document is a wait,
        // never a destruction settlement.
        if state.settle_destroyed_history_job(dom, h, t, job, cancel)? {
            return Ok(DomHostedTaskProgress::CanceledDocumentDestroyed {
                operation: job.operation,
                document: job.document,
                global: job.global,
            });
        }
        let (gh, gt, c, g) = match state.find_global(job.global, cancel, || Ok(())) {
            Ok(value) => value,
            Err(DomHostedRuntimeError::TargetNotFound) => {
                if let Some(progress) =
                    state.pump_destruction(dom, traversable, Some(job.document), cancel)?
                {
                    return Ok(progress);
                }
                return Err(DomHostedRuntimeError::TargetNotFound);
            }
            Err(error) => return Err(error),
        };
        let loop_owner = {
            let group = state.hosts[gh].top_levels[gt]
                .group
                .as_ref()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            let native = &group.clusters[c].agent.globals[g];
            if native.settings.document != Some(job.document) || !native.settings.ready {
                return Err(DomHostedRuntimeError::Binding);
            }
            group.clusters[c].agent.event_loop.clone()
        };
        let mut found = false;
        {
            let event_loop = loop_owner
                .try_borrow()
                .map_err(|_| DomHostedRuntimeError::Busy)?;
            for task in &event_loop.navigation {
                state.step(cancel)?;
                if let records::NavigationTask::History(task) = task
                    && task.operation == job.operation
                    && task.occurrence == job.occurrence
                {
                    if task.document != job.document || task.global != job.global {
                        return Err(DomHostedRuntimeError::Protocol);
                    }
                    found = true;
                    break;
                }
            }
        }
        if !found {
            state.reserve_navigation(&loop_owner, cancel)?;
            state.step(cancel)?;
            loop_owner
                .try_borrow_mut()
                .map_err(|_| DomHostedRuntimeError::Busy)?
                .navigation
                .push(records::NavigationTask::History(records::HistoryTask {
                    operation: job.operation,
                    occurrence: job.occurrence,
                    document: job.document,
                    global: job.global,
                    receipt: None,
                }));
        }
        let mut event_loop = loop_owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let current = records::CurrentTask::History {
            operation: job.operation,
            occurrence: job.occurrence,
            document: job.document,
        };
        if event_loop.executing || event_loop.current.is_some_and(|old| old != current) {
            let dependency = if event_loop.executing {
                None
            } else {
                match event_loop.current {
                    Some(records::CurrentTask::Destroy { document, .. }) => Some(document),
                    _ => None,
                }
            };
            drop(event_loop);
            if let Some(document) = dependency
                && let Some(progress) =
                    state.pump_destruction(dom, traversable, Some(document), cancel)?
            {
                return Ok(progress);
            }
            return Ok(DomHostedTaskProgress::Pending {
                operation,
                reason: DomHostedTaskWait::CurrentTask,
            });
        }
        let Some(records::NavigationTask::History(task)) = event_loop.navigation.first() else {
            let dependency = event_loop
                .navigation
                .first()
                .and_then(|task| task.destroy())
                .map(|task| task.document);
            drop(event_loop);
            if let Some(document) = dependency
                && let Some(progress) =
                    state.pump_destruction(dom, traversable, Some(document), cancel)?
            {
                return Ok(progress);
            }
            return Ok(DomHostedTaskProgress::Pending {
                operation,
                reason: DomHostedTaskWait::CurrentTask,
            });
        };
        if task.operation != job.operation || task.occurrence != job.occurrence {
            return Ok(DomHostedTaskProgress::Pending {
                operation,
                reason: DomHostedTaskWait::CurrentTask,
            });
        }
        if event_loop.current.is_none() {
            state.step(cancel)?;
            if dom
                .raw()
                .global_document(job.global)
                .map_err(DomHostedRuntimeError::Dom)?
                != job.document
            {
                return Err(DomHostedRuntimeError::Binding);
            }
            if !dom
                .snapshot()
                .document(job.document)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?
                .is_fully_active()
                .map_err(DomHostedRuntimeError::Dom)?
            {
                drop(event_loop);
                if let Some(progress) =
                    state.pump_destruction(dom, traversable, Some(job.document), cancel)?
                {
                    return Ok(progress);
                }
                return Ok(DomHostedTaskProgress::Pending {
                    operation,
                    reason: DomHostedTaskWait::Inactive,
                });
            }
            event_loop.next_checkpoint = event_loop
                .checkpoints
                .checked_add(1)
                .ok_or(DomHostedRuntimeError::Admission)?;
            event_loop.current = Some(current);
        }
        let navigation_moves = event_loop.navigation.len();
        let Some(records::NavigationTask::History(task)) = event_loop.navigation.first_mut() else {
            return Err(DomHostedRuntimeError::Protocol);
        };
        if task.receipt.is_none() {
            let HostedState { hosts, source, .. } = &mut *state;
            let continuation = hosts[h].top_levels[t]
                .group
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?
                .traversal
                .rows[0]
                .continuation
                .as_mut()
                .ok_or(DomHostedRuntimeError::Protocol)?;
            let surgeist_dom::DomHostedHistoryStep::Job(request) = dom
                .raw_mut()
                .resume_queued_history(source, continuation, cancel)
                .map_err(DomHostedRuntimeError::Dom)?
            else {
                return Err(DomHostedRuntimeError::Protocol);
            };
            if Job::from_request(request) != job {
                return Err(DomHostedRuntimeError::Protocol);
            }
            task.receipt = Some(
                continuation
                    .execute_job(dom.raw_mut(), source, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?,
            );
        }
        // The actual write receipt is already in the preadmitted task before
        // this fallible checkpoint. A retry never repeats the write/body.
        state.step(cancel)?;
        let snapshot = dom.snapshot();
        if snapshot.incomplete_source_operation().is_some()
            || snapshot.incomplete_hosted_operation().is_some()
            || !snapshot.pending_slot_signals().is_empty()
        {
            return Ok(DomHostedTaskProgress::Pending {
                operation,
                reason: DomHostedTaskWait::CheckpointUnavailable,
            });
        }
        // Prepay exact FIFO movement before the canonical receipt is consumed.
        for _ in 0..navigation_moves {
            state.step(cancel)?;
        }
        let HostedState { hosts, source, .. } = &mut *state;
        let continuation = hosts[h].top_levels[t]
            .group
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .traversal
            .rows[0]
            .continuation
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        continuation
            .settle_job(dom.raw_mut(), source, &mut task.receipt, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        event_loop.navigation.remove(0);
        event_loop.current = None;
        event_loop.checkpoints = event_loop.next_checkpoint;
        Ok(DomHostedTaskProgress::Executed {
            operation: job.operation,
            document: job.document,
            global: job.global,
            kind: DomHostedTaskKind::History,
        })
    }
}
