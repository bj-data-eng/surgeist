//! Native Slot delivery in the original Window agent's checkpoint.

use super::*;
use crate::internal::{
    dom_event::{
        DispatchHost, DispatchUsage, DomEventDispatch, DomEventDispatchError,
        DomEventDispatchOutcome, DomEventDispatcher, DomEventTimestampUnavailable,
        NativeSlotCheckpoint,
    },
    element_tasks::{DomElementTaskRunLimits, DomElementTaskRuntimeError},
};
use std::cell::Cell;
use surgeist_dom::{DomError, DomSnapshot, identity::NodeId, tree::DomNode};

struct SlotCheckpointLoan<'a> {
    event_loop: &'a RefCell<records::WindowLoop>,
    batch: Option<NativeSlotCheckpoint>,
}

impl Drop for SlotCheckpointLoan<'_> {
    fn drop(&mut self) {
        // WindowLoop::executing keeps another native task out. No loop loan
        // crosses the shared dispatcher or any of its callbacks.
        if let Some(batch) = self.batch.take() {
            self.event_loop.borrow_mut().slot_checkpoint = Some(batch);
        }
    }
}

impl NativeTaskRunning {
    pub(crate) fn deliver_slot_checkpoint<C>(
        &self,
        dom: &mut DomEventDom<'_>,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        (limits, usage): (DomElementTaskRunLimits, &mut DispatchUsage),
        started: &Cell<bool>,
        cancel: &AtomicBool,
    ) -> Result<Option<DomEventDispatchOutcome>, DomElementTaskRuntimeError> {
        let batch = self
            .event_loop
            .try_borrow_mut()
            .map_err(|_| DomElementTaskRuntimeError::Hosted(DomHostedRuntimeError::Busy))?
            .slot_checkpoint
            .take();
        let mut loan = SlotCheckpointLoan {
            event_loop: &self.event_loop,
            batch,
        };
        let pending = dom.raw().has_pending_native_slot_checkpoint();
        if pending {
            // Even a skipped body has reached and still owes its checkpoint.
            // Keep that original row across refusal before target transfer.
            started.set(true);
        }
        if (loan.batch.is_some() || pending) && dom.raw().has_active_document_script_registration()
        {
            return Err(DomElementTaskRuntimeError::Hosted(
                DomHostedRuntimeError::CheckpointUnavailable,
            ));
        }
        loop {
            if loan.batch.is_none() {
                if !dom.raw().has_pending_native_slot_checkpoint() {
                    return Ok(None);
                }
                // Only a new transfer needs proof about the global DOM queue.
                // A held batch already belongs to this exact Window loop.
                loan.batch = self.capture_slot_batch(
                    dom,
                    dispatcher,
                    limits.source.output_records(),
                    cancel,
                )?;
            }
            let batch = loan
                .batch
                .as_mut()
                .ok_or(DomElementTaskRuntimeError::Hosted(
                    DomHostedRuntimeError::CheckpointUnavailable,
                ))?;
            let Some(target) = batch.targets.get(batch.next).copied() else {
                self.with_hosted_state(dom, |state| {
                    state.slot_checkpoint_active = false;
                    Ok(())
                })?;
                loan.batch = None;
                continue;
            };
            if batch
                .dispatch
                .as_ref()
                .is_some_and(DomEventDispatch::is_abandoned)
            {
                self.with_hosted_state(dom, |state| {
                    state
                        .step(cancel)
                        .map_err(DomElementTaskRuntimeError::Hosted)
                })?;
                batch.advance_target();
                return Err(DomElementTaskRuntimeError::Dispatch(
                    DomEventDispatchError::Abandoned,
                ));
            }
            if batch
                .dispatch
                .as_ref()
                .is_some_and(|dispatch| !dispatch.has_started() && !dispatch.belongs_to(dispatcher))
            {
                batch.dispatch = None;
            }
            if batch
                .dispatch
                .as_ref()
                .is_none_or(|dispatch| !dispatch.has_started())
                && !self.slot_is_live(dom, target, cancel)?
            {
                batch.advance_target();
                continue;
            }
            if batch.dispatch.is_none() {
                let creation = self.sample_slot_event_creation(dom, target, cancel)?;
                batch.dispatch = Some(
                    dispatcher
                        .prepare_slot_change(dom, target, creation, cancel)
                        .map_err(DomElementTaskRuntimeError::Dispatch)?,
                );
            }
            let dispatch = batch
                .dispatch
                .as_mut()
                .ok_or(DomElementTaskRuntimeError::Binding)?;
            match dispatch.advance_with_host_usage(
                dom,
                dispatcher,
                context,
                (limits.dispatch, usage),
                cancel,
                DispatchHost::Unavailable,
            ) {
                Ok(DomEventDispatchOutcome::Complete(_)) => batch.advance_target(),
                Ok(retained) => return Ok(Some(retained)),
                Err(error) => {
                    // Shared dispatch proved abandonment. Report it once and
                    // leave the remaining targets for the original task.
                    if matches!(error, DomEventDispatchError::Abandoned) {
                        batch.advance_target();
                    }
                    return Err(DomElementTaskRuntimeError::Dispatch(error));
                }
            }
        }
    }

    fn capture_slot_batch<C>(
        &self,
        dom: &mut DomEventDom<'_>,
        dispatcher: &DomEventDispatcher<C>,
        output_limit: usize,
        cancel: &AtomicBool,
    ) -> Result<Option<NativeSlotCheckpoint>, DomElementTaskRuntimeError> {
        let owner = self
            .state
            .upgrade()
            .ok_or(DomElementTaskRuntimeError::Hosted(
                DomHostedRuntimeError::OwnerDisposed,
            ))?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomElementTaskRuntimeError::Hosted(DomHostedRuntimeError::Busy))?;
        state
            .qualify(dom)
            .map_err(DomElementTaskRuntimeError::Hosted)?;
        if !state.window_agents.is_only(&self.event_loop)
            || state.slot_checkpoint_active
            || dom.raw().has_pending_media_removals()
        {
            return Err(DomElementTaskRuntimeError::Hosted(
                DomHostedRuntimeError::CheckpointUnavailable,
            ));
        }
        {
            let snapshot = dom.snapshot();
            for &target in snapshot.pending_slot_signals() {
                self.slot_realm(&mut state, &snapshot, target, cancel)?;
            }
        }
        let batch = dispatcher
            .take_slot_checkpoint(dom, output_limit, cancel)
            .map_err(DomElementTaskRuntimeError::Dispatch)?;
        state.slot_checkpoint_active = batch.is_some();
        Ok(batch)
    }

    fn with_hosted_state<T>(
        &self,
        dom: &DomEventDom<'_>,
        read: impl FnOnce(&mut HostedState) -> Result<T, DomElementTaskRuntimeError>,
    ) -> Result<T, DomElementTaskRuntimeError> {
        let owner = self
            .state
            .upgrade()
            .ok_or(DomElementTaskRuntimeError::Hosted(
                DomHostedRuntimeError::OwnerDisposed,
            ))?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomElementTaskRuntimeError::Hosted(DomHostedRuntimeError::Busy))?;
        state
            .qualify(dom)
            .map_err(DomElementTaskRuntimeError::Hosted)?;
        read(&mut state)
    }

    fn slot_is_live(
        &self,
        dom: &DomEventDom<'_>,
        target: NodeId,
        cancel: &AtomicBool,
    ) -> Result<bool, DomElementTaskRuntimeError> {
        self.with_hosted_state(dom, |state| {
            let snapshot = dom.snapshot();
            match slot_node(state, &snapshot, target, cancel) {
                Ok(node) => {
                    if let Some(document) = node.owner_document()
                        && let Some(global) = slot_node(state, &snapshot, document.node(), cancel)?
                            .relevant_global()
                            .map_err(|error| DomElementTaskRuntimeError::Hosted(error.into()))?
                    {
                        self.slot_agent_origin(state, global, cancel)?;
                    }
                    Ok(true)
                }
                Err(DomElementTaskRuntimeError::Dom(DomError::StaleNode)) => Ok(false),
                Err(error) => Err(error),
            }
        })
    }

    fn sample_slot_event_creation(
        &self,
        dom: &DomEventDom<'_>,
        target: NodeId,
        cancel: &AtomicBool,
    ) -> Result<Result<EventCreation, DomEventTimestampUnavailable>, DomElementTaskRuntimeError>
    {
        self.with_hosted_state(dom, |state| {
            let (global, origin) = self.slot_realm(state, &dom.snapshot(), target, cancel)?;
            let sample = state
                .sample_admitted_clock(dom, cancel)
                .map_err(DomElementTaskRuntimeError::Hosted)?;
            event::firing_creation_from_sample(global, origin, sample)
                .map_err(DomElementTaskRuntimeError::Hosted)
        })
    }

    fn slot_realm(
        &self,
        state: &mut HostedState,
        snapshot: &DomSnapshot,
        target: NodeId,
        cancel: &AtomicBool,
    ) -> Result<(DomHostedGlobalId, clock::CoarsenedSample), DomElementTaskRuntimeError> {
        let global = slot_node(state, snapshot, target, cancel)?
            .relevant_global()
            .map_err(|error| DomElementTaskRuntimeError::Hosted(error.into()))?
            .ok_or(DomElementTaskRuntimeError::Hosted(
                DomHostedRuntimeError::CheckpointUnavailable,
            ))?;
        Ok((global, self.slot_agent_origin(state, global, cancel)?))
    }

    fn slot_agent_origin(
        &self,
        state: &mut HostedState,
        global: DomHostedGlobalId,
        cancel: &AtomicBool,
    ) -> Result<clock::CoarsenedSample, DomElementTaskRuntimeError> {
        let (h, t, c, g) = state
            .find_global(global, cancel, |_| Ok(()))
            .map_err(DomElementTaskRuntimeError::Hosted)?;
        let agent = &state.hosts[h].top_levels[t]
            .group
            .as_ref()
            .ok_or(DomElementTaskRuntimeError::Hosted(
                DomHostedRuntimeError::Protocol,
            ))?
            .clusters[c]
            .agent;
        if !Rc::ptr_eq(&agent.event_loop, &self.event_loop) {
            return Err(DomElementTaskRuntimeError::Hosted(
                DomHostedRuntimeError::CheckpointUnavailable,
            ));
        }
        Ok(agent.globals[g].settings.navigation_start)
    }
}

fn slot_node<'snapshot>(
    state: &mut HostedState,
    snapshot: &'snapshot DomSnapshot,
    target: NodeId,
    cancel: &AtomicBool,
) -> Result<DomNode<'snapshot>, DomElementTaskRuntimeError> {
    snapshot.node_with(target, || {
        state
            .step(cancel)
            .map_err(DomElementTaskRuntimeError::Hosted)
    })
}
