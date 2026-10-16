//! The actual parent-Global navigation task and retained top-level teardown.
use super::*;
use surgeist_dom::{
    DomHostedDestructionLease, DomHostedDestructionPublication, DomHostedDestructionStep,
};

pub(super) struct Destruction {
    host: DomHostedHostId,
    navigable: DomHostedNavigableId,
    lease: DomHostedDestructionLease,
    publication: Option<DomHostedDestructionPublication>,
    final_moves_admitted: bool,
}
impl HostedRuntime {
    /// Queue the actual parent-Global navigation-and-traversal destruction task.
    /// This initiating call returns Pending after enqueue; it does not run that
    /// queued body inline, even when the captured child set is empty.
    pub fn destroy_top_level(
        &mut self,
        dom: &mut DomEventDom<'_>,
        navigable: DomHostedNavigableId,
        cancel: &AtomicBool,
    ) -> Result<DomHostedProgress, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        if state.creation.is_some() || state.destruction.is_some() {
            return Err(DomHostedRuntimeError::OperationPending);
        }
        let (index, top_index) = state.find_top(navigable, cancel)?;
        let lease = dom
            .raw_mut()
            .begin_hosted_top_level_destruction(&mut state.source, navigable, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        let host = state.hosts[index].id;
        state.hosts[index].top_levels[top_index]
            .top
            .as_mut()
            .expect("actual selected top")
            .closing = true;
        state.destruction = Some(Destruction {
            host,
            navigable,
            lease,
            publication: None,
            final_moves_admitted: false,
        });
        state.resume_destruction(dom, cancel)
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
    fn reserve_navigation(
        &mut self,
        h: usize,
        t: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        let mut count = 0usize;
        for host in 0..self.hosts.len() {
            self.step(cancel)?;
            for record in 0..self.hosts[host].top_levels.len() {
                self.step(cancel)?;
                if let Some(cluster) = self.hosts[host].top_levels[record]
                    .group
                    .as_ref()
                    .and_then(|g| g.cluster.as_ref())
                {
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
        if count >= self.limits.tasks {
            return Err(DomHostedRuntimeError::Admission);
        }
        let loop_owner = self.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .cluster
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .agent
            .event_loop
            .clone();
        let mut event_loop = loop_owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let queue = &mut event_loop.navigation;
        if queue.len() < queue.capacity() {
            return Ok(());
        }
        let required = queue
            .len()
            .checked_add(1)
            .ok_or(DomHostedRuntimeError::Admission)?;
        // Drop the loop borrow while measuring all actual retained backing.
        drop(event_loop);
        let old = self.retained_bytes(cancel)?;
        let requested = required
            .checked_mul(size_of::<records::DestroyTask>())
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
            .checked_mul(size_of::<records::DestroyTask>())
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
        replacement.append(&mut event_loop.navigation);
        event_loop.navigation = replacement;
        Ok(())
    }
    pub(super) fn resume_destruction(
        &mut self,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomHostedProgress, DomHostedRuntimeError> {
        let navigable = self
            .destruction
            .as_ref()
            .ok_or(DomHostedRuntimeError::TargetNotFound)?
            .navigable;
        let (index, top_index) = self.find_top(navigable, cancel)?;
        // Queue capacity may grow only before the queued phase. Every subsequent
        // reached service is in-place and retains the original body on refusal.
        let enqueue = matches!(
            self.destruction
                .as_ref()
                .expect("retained destruction")
                .lease
                .step(dom.raw_mut())
                .map_err(DomHostedRuntimeError::Dom)?,
            DomHostedDestructionStep::EnqueueParentTask(_)
        );
        let already_queued = !self.hosts[index].top_levels[top_index]
            .group
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .cluster
            .as_ref()
            .ok_or(DomHostedRuntimeError::Protocol)?
            .agent
            .event_loop
            .try_borrow()
            .map_err(|_| DomHostedRuntimeError::Busy)?
            .navigation
            .is_empty();
        if enqueue && !already_queued {
            self.reserve_navigation(index, top_index, cancel)?;
        }
        if !self
            .destruction
            .as_ref()
            .expect("retained destruction")
            .final_moves_admitted
        {
            // No operation can append/remove Hosts while this destruction owns
            // the lifecycle cursor: create_host/create_top_level refuse it.
            // Thus this exact current length bounds final infallible removal.
            for _ in 0..self.hosts[index].top_levels.len() {
                self.step(cancel)?;
            }
            self.destruction
                .as_mut()
                .expect("retained destruction")
                .final_moves_admitted = true;
        }
        self.step(cancel)?;
        let Self {
            source,
            hosts,
            destruction,
            details,
            work,
            limits,
            ..
        } = self;
        let operation = destruction.as_mut().expect("retained destruction");
        let host = &mut hosts[index];
        let group = host.top_levels[top_index]
            .group
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        let cluster = group
            .cluster
            .as_mut()
            .ok_or(DomHostedRuntimeError::Protocol)?;
        let loop_owner = cluster.agent.event_loop.clone();
        let mut loop_borrow = loop_owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let event_loop = &mut *loop_borrow;
        match operation
            .lease
            .step(dom.raw_mut())
            .map_err(DomHostedRuntimeError::Dom)?
        {
            DomHostedDestructionStep::EnqueueParentTask(request) => {
                if request.child_navigable_count() != 0 {
                    return Err(DomHostedRuntimeError::Protocol);
                }
                if event_loop.navigation.is_empty() {
                    // The captured admitted child set is genuinely empty; its
                    // zero wait is satisfied before queuing the parent body.
                    event_loop.navigation.push(records::DestroyTask {
                        operation: request.operation(),
                        document: request.document(),
                        global: request.global(),
                        captured_children: Vec::new(),
                        completed_children: 0,
                        phase: records::DestroyPhase::WaitChildren,
                    });
                    event_loop.navigation[0].phase = records::DestroyPhase::Queued;
                }
                if operation.publication.is_none() {
                    operation.publication = Some(
                        source
                            .seal_destruction_queued(request, cancel)
                            .map_err(DomHostedRuntimeError::Dom)?,
                    );
                }
                operation
                    .lease
                    .accept_publication(dom.raw_mut(), source, &mut operation.publication, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?;
                Ok(DomHostedProgress::Pending {
                    host: operation.host,
                })
            }
            DomHostedDestructionStep::Cleanup(request) => {
                let task = event_loop
                    .navigation
                    .first_mut()
                    .ok_or(DomHostedRuntimeError::Protocol)?;
                if task.operation != request.operation()
                    || task.document != request.document()
                    || task.global != request.global()
                    || task.completed_children != task.captured_children.len()
                {
                    return Err(DomHostedRuntimeError::Protocol);
                }
                let details = details.as_ref().ok_or(DomHostedRuntimeError::Protocol)?;
                // Intake sees actual callback-created raw rows before source
                // cleanup; original queue Document, not adopted target, owns it.
                details
                    .ingest_source(dom, source, cancel)
                    .map_err(details_error)?;
                let pending_started = details
                    .cancel_document(dom, task.document, cancel)
                    .map_err(details_error)?;
                if pending_started {
                    return Ok(DomHostedProgress::Pending {
                        host: operation.host,
                    });
                }
                if let Some(records::CurrentTask::Details { task: old, .. }) = event_loop.current {
                    if details.has_task(old).map_err(details_error)? {
                        return Ok(DomHostedProgress::Pending {
                            host: operation.host,
                        });
                    }
                    // This was an aborted pre-dispatch body. Complete its actual
                    // native checkpoint, without claiming event delivery.
                    checkpoint(dom, work, limits.work, cancel)?;
                    event_loop.current = None;
                    event_loop.checkpoints = event_loop.next_checkpoint;
                }
                let current = records::CurrentTask::Destroy {
                    operation: task.operation,
                    document: task.document,
                };
                if event_loop.executing || event_loop.current.is_some_and(|old| old != current) {
                    return Ok(DomHostedProgress::Pending {
                        host: operation.host,
                    });
                }
                if event_loop.current.is_none() {
                    spend(work, limits.work, cancel)?;
                    event_loop.next_checkpoint = event_loop
                        .checkpoints
                        .checked_add(1)
                        .ok_or(DomHostedRuntimeError::Admission)?;
                    event_loop.current = Some(current);
                    task.phase = records::DestroyPhase::Cleanup;
                }
                if operation.publication.is_none() {
                    let DomHostedDestructionStep::Cleanup(current) = operation
                        .lease
                        .step(dom.raw_mut())
                        .map_err(DomHostedRuntimeError::Dom)?
                    else {
                        return Err(DomHostedRuntimeError::Protocol);
                    };
                    operation.publication = Some(
                        source
                            .seal_top_level_cleanup(current, cancel)
                            .map_err(DomHostedRuntimeError::Dom)?,
                    );
                }
                operation
                    .lease
                    .accept_publication(dom.raw_mut(), source, &mut operation.publication, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?;
                checkpoint(dom, work, limits.work, cancel)?;
                task.phase = records::DestroyPhase::CheckpointComplete;
                event_loop.checkpoints = event_loop.next_checkpoint;
                // Source transition succeeded; repeat only the explicit finish
                // if that final admission refuses, never the body/checkpoint.
                let _complete = operation
                    .lease
                    .finish(dom.raw_mut(), source, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?;
                let navigable = operation.navigable;
                // Final host moves/drop were admitted before canonical cleanup.
                // Source finish releases only the top-specific operation/roles; Host survives.
                hosts[index].top_levels.remove(top_index);
                *destruction = None;
                Ok(DomHostedProgress::Destroyed { navigable })
            }
            DomHostedDestructionStep::Complete => {
                if event_loop
                    .navigation
                    .first()
                    .is_none_or(|task| task.phase != records::DestroyPhase::CheckpointComplete)
                {
                    checkpoint(dom, work, limits.work, cancel)?;
                    let task = event_loop
                        .navigation
                        .first_mut()
                        .ok_or(DomHostedRuntimeError::Protocol)?;
                    task.phase = records::DestroyPhase::CheckpointComplete;
                    event_loop.checkpoints = event_loop.next_checkpoint;
                }
                let _complete = operation
                    .lease
                    .finish(dom.raw_mut(), source, cancel)
                    .map_err(DomHostedRuntimeError::Dom)?;
                let navigable = operation.navigable;
                hosts[index].top_levels.remove(top_index);
                *destruction = None;
                Ok(DomHostedProgress::Destroyed { navigable })
            }
        }
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
fn details_error(error: crate::dom_details::DomDetailsRuntimeError) -> DomHostedRuntimeError {
    use crate::dom_details::DomDetailsRuntimeError as Error;
    match error {
        Error::Hosted(error) => error,
        Error::Dom(error) => DomHostedRuntimeError::Dom(error.into()),
        Error::Busy => DomHostedRuntimeError::Busy,
        Error::OwnerDisposed => DomHostedRuntimeError::OwnerDisposed,
        Error::Admission | Error::IdentityExhausted => DomHostedRuntimeError::Admission,
        Error::Binding
        | Error::OutOfOrderTransfer
        | Error::PretransferredSession
        | Error::Dispatch(_) => DomHostedRuntimeError::Protocol,
    }
}
