//! Actual queue publication for the original Dialog Source cursor.
use super::*;
use crate::{DomElementTaskAssociation, DomManipulationTaskId, DomManipulationTaskSource};
use surgeist_html::{HtmlDialogAction, HtmlDialogState, HtmlDialogToggleDecision};

/// Immutable payload of one real queued Dialog toggle task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDialogToggleTaskInfo {
    pub(crate) id: DomManipulationTaskId,
    pub(crate) old: HtmlDialogState,
    pub(crate) new: HtmlDialogState,
    pub(crate) association: Option<DomElementTaskAssociation>,
}
impl DomDialogToggleTaskInfo {
    /// Exact task and receiver generation.
    pub const fn id(self) -> DomManipulationTaskId {
        self.id
    }
    /// First state retained by the HTML coalescing decision.
    pub const fn old_state(self) -> HtmlDialogState {
        self.old
    }
    /// Latest requested state, even when equal to old state.
    pub const fn new_state(self) -> HtmlDialogState {
        self.new
    }
    /// Original relevant Global and its associated Document, if genuinely hosted.
    pub const fn queue_association(self) -> Option<DomElementTaskAssociation> {
        self.association
    }
    /// This admitted Show contribution has an actual null source.
    pub const fn source(self) -> Option<NodeId> {
        None
    }
}
/// Affine permission belonging to the actual queued task, never a copied ID.
#[derive(Debug)]
pub struct DomDialogToggleTask {
    pub(crate) info: DomDialogToggleTaskInfo,
    pub(crate) binding: std::sync::Weak<crate::manipulation::Binding>,
}
impl DomDialogToggleTask {
    /// Immutable historical task inputs.
    pub const fn info(&self) -> DomDialogToggleTaskInfo {
        self.info
    }
}
/// Exact original request's prepared language decision and real task inputs.
#[derive(Clone, Copy, Debug)]
pub struct DomDialogToggleQueuePlan {
    operation: DomSourceOperationId,
    info: DomDialogToggleTaskInfo,
    replacing: Option<DomManipulationTaskId>,
}
impl DomDialogToggleQueuePlan {
    /// Actual candidate task payload.
    pub const fn task(self) -> DomDialogToggleTaskInfo {
        self.info
    }
    /// Only this still-queued task may be removed by replacement.
    pub const fn replacing(self) -> Option<DomManipulationTaskId> {
        self.replacing
    }
}
/// Move-only evidence of the actual no-callback queue/tracker publication.
#[derive(Debug)]
pub struct DomDialogToggleQueuePublication {
    operation: DomSourceOperationId,
    task: DomManipulationTaskId,
}
impl DomDialogToggleQueuePublication {
    /// Original operation which performed this exact real queue publication.
    pub const fn operation(&self) -> DomSourceOperationId {
        self.operation
    }
}
/// Exclusive canonical publication loan. No fallible work or user callback is
/// allowed between installing the prepared real queue and consuming this loan.
pub struct DomDialogToggleQueueCommit<'a, 'dom> {
    lease: &'a mut DomSourceLease<'dom>,
    prepared: Option<crate::store::PreparedPublication>,
    plan: DomDialogToggleQueuePlan,
    task: Option<DomDialogToggleTask>,
    canonical: storage::Payload,
    outputs: usize,
    remaining_phases: u64,
    checkpoint: Option<checkpoint::Checkpoint>,
}
impl DomDialogToggleQueueCommit<'_, '_> {
    /// Publish the already admitted canonical tracker after the actual owner
    /// removes/clears/appends/stores its prepared task. This fence cannot fail.
    pub fn publish(mut self) -> (DomDialogToggleTask, DomDialogToggleQueuePublication) {
        let commit = self
            .lease
            .dom
            .publish_prepared(self.prepared.take().expect("prepared fence"));
        let op = self
            .lease
            .dom
            .source_operation
            .as_mut()
            .expect("exclusive original operation");
        op.canonical_storage = self.canonical;
        op.canonical_history_bytes = self.lease.dom.history_bytes;
        op.outputs = self.outputs;
        op.phases += 1;
        op.published = true;
        op.event_exposed = true;
        op.dialog_queue = Some(self.plan.info.id);
        op.headroom = Some(headroom::Reservation {
            remaining_phases: self.remaining_phases,
        });
        op.command_admitted = true;
        drop(commit);
        self.checkpoint.take();
        (
            self.task.take().expect("actual task permission"),
            DomDialogToggleQueuePublication {
                operation: self.plan.operation,
                task: self.plan.info.id,
            },
        )
    }
}
impl Drop for DomDialogToggleQueueCommit<'_, '_> {
    fn drop(&mut self) {
        if let Some(checkpoint) = self.checkpoint.take() {
            checkpoint.restore(
                &mut self
                    .lease
                    .dom
                    .source_operation
                    .as_mut()
                    .expect("exclusive operation")
                    .draft,
            );
        }
    }
}
impl<'dom> DomSourceLease<'dom> {
    /// Exact prior Details intentions which must join this same source bank
    /// before its reached Dialog append. Ordinary idle-only intake is unchanged.
    pub fn dialog_toggle_prior_details<'snapshot>(
        &self,
        request: &DomSourceDialogRequest,
        snapshot: &'snapshot DomSnapshot,
    ) -> Result<&'snapshot [crate::DomDetailsToggleIntention], DomError> {
        if request.operation != self.id
            || !matches!(request.action, HtmlDialogAction::QueueToggle { .. })
        {
            return Err(DomError::InvalidSourceOperation);
        }
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let frame = operation
            .draft
            .dialog_show
            .ok_or(DomError::InvalidSourceOperation)?;
        if operation.draft.published_id(frame.subject) != Some(request.subject)
            || frame.cursor.action() != request.action
        {
            return Err(DomError::InvalidSourceOperation);
        }
        if snapshot.owner != self.dom.owner
            || snapshot.revision != self.dom.revision
            || !Arc::ptr_eq(&snapshot.state, &self.dom.state)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        Ok(&snapshot.state.details_intentions)
    }
    /// Admit actual task-owner storage through this exact Source permission.
    pub fn retain_manipulation_payload(
        &mut self,
        source: &DomManipulationTaskSource,
        bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<crate::DomManipulationTaskRetention, DomError> {
        source.retain_payload(self.dom, bytes, cancel)
    }
    /// Prepare the actual HTML queue decision without queueing or acknowledging.
    pub fn dialog_toggle_queue_plan(
        &mut self,
        request: &DomSourceDialogRequest,
        source: &DomManipulationTaskSource,
        serial: u64,
        cancel: &AtomicBool,
    ) -> Result<DomDialogToggleQueuePlan, DomError> {
        source.qualify(self.dom)?;
        let op = self
            .dom
            .source_operation
            .as_mut()
            .filter(|op| op.id == self.id && request.operation == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let frame = op
            .draft
            .dialog_show
            .ok_or(DomError::InvalidSourceOperation)?;
        if op.draft.published_id(frame.subject) != Some(request.subject)
            || frame.cursor.action() != request.action
            || op.dialog_queue.is_some()
        {
            return Err(DomError::InvalidSourceOperation);
        }
        let HtmlDialogAction::QueueToggle {
            old,
            new,
            source: surgeist_html::HtmlDialogEventSource::Null,
        } = request.action
        else {
            return Err(DomError::InvalidSourceOperation);
        };
        op.refresh_admission(cancel)?;
        let tracker = op.draft.dialog_state(frame.subject)?.toggle_tracker;
        let decision = HtmlDialogToggleDecision::new(old, new, tracker.map(|tracker| tracker.old));
        let info = DomDialogToggleTaskInfo {
            id: source.task_id(serial, request.subject)?,
            old: decision.old_state(),
            new: decision.new_state(),
            association: op.draft.element_task_association(frame.subject, cancel)?,
        };
        Ok(DomDialogToggleQueuePlan {
            operation: self.id,
            info,
            replacing: tracker.map(|tracker| tracker.task),
        })
    }
    /// Complete all fallible canonical formation before the actual queue fence.
    pub fn prepare_dialog_toggle_queue<'a>(
        &'a mut self,
        request: &DomSourceDialogRequest,
        source: &DomManipulationTaskSource,
        plan: DomDialogToggleQueuePlan,
        cancel: &AtomicBool,
    ) -> Result<DomDialogToggleQueueCommit<'a, 'dom>, DomError> {
        let actual = self.dialog_toggle_queue_plan(request, source, plan.info.id.serial, cancel)?;
        if actual.info != plan.info || actual.replacing != plan.replacing {
            return Err(DomError::InvalidSourceOperation);
        }
        let mut op = self
            .dom
            .source_operation
            .take()
            .expect("qualified original operation");
        let checkpoint = match op
            .input_bytes()
            .and_then(|bytes| op.draft.source_checkpoint(bytes, 0, cancel))
        {
            Ok(checkpoint) => checkpoint,
            Err(error) => {
                self.dom.source_operation = Some(op);
                return Err(error);
            }
        };
        let result = (|| {
            let formation = formation::Formation::new(&op.draft, cancel)?;
            if let Some(formation) = &formation {
                formation.retain_bytes(
                    size_of::<DomDialogToggleQueueCommit<'_, '_>>()
                        .checked_sub(size_of::<Draft>())
                        .and_then(|header| {
                            header.checked_add(size_of::<crate::dialog::DomDialogState>())
                        })
                        .ok_or(DomError::RepresentationExhausted)?,
                    cancel,
                )?;
            }
            let frame = op.draft.dialog_show.expect("qualified Dialog frame");
            let mut state = op.draft.dialog_state(frame.subject)?.clone();
            state.toggle_tracker = Some(crate::dialog::DomDialogToggleTracker {
                task: plan.info.id,
                old: plan.info.old,
            });
            op.draft.element_mut(frame.subject).set_dialog_state(state);
            op.draft.details_intentions.clear();
            op.draft.prepare_transition_phase(cancel)?;
            let mut prepared = op.draft.finish(self.dom, cancel)?;
            prepared.state.incomplete_source = Some(self.id);
            if self.dom.state.incomplete_source != Some(self.id) {
                formation::reserve(
                    &mut prepared.changes,
                    1,
                    prepared.formation.as_ref(),
                    cancel,
                )?;
                prepared.changes.push(DomChange::SourceOperation);
            }
            let document = op.draft.node(frame.subject)?.document;
            let original_window_active = match self
                .dom
                .snapshot()
                .relevant_global(plan.info.id.target)
                .map_err(|_| DomError::HostedSourceUnavailable)?
            {
                None => false,
                Some(global) => {
                    let associated = self
                        .dom
                        .snapshot()
                        .global_document(global)
                        .map_err(|_| DomError::HostedSourceUnavailable)?;
                    let key = op.draft.resolve(associated.node().into())?;
                    op.draft.document_fully_active(key, cancel)?
                }
            };
            let reached_hook = op.draft.document_fully_active(document, cancel)?
                && op.draft.shadow_connected(frame.subject, cancel)?;
            // Remember is a distinct actual Show occurrence after AddOpen;
            // the reached ordinary Hook has its own original shared ordinal.
            op.draft
                .admit_dialog_occurrences(1 + u64::from(reached_hook), cancel)?;
            let active_hook = reached_hook && original_window_active;
            let extra = u64::from(active_hook);
            let required_phases = 3 + extra;
            if op
                .phases
                .checked_add(required_phases)
                .is_none_or(|phases| phases > op.limits.phases)
            {
                return Err(DomError::SourcePublicationLimit);
            }
            self.dom
                .revision
                .0
                .checked_add(required_phases)
                .and_then(|revision| revision.checked_add(self.dom.hosted_revisions))
                .and_then(|revision| {
                    self.dom
                        .source_reserved_closing()
                        .ok()
                        .and_then(|closing| revision.checked_add(closing))
                })
                .ok_or(DomError::RevisionExhausted)?;
            let outputs = op
                .outputs
                .checked_add(prepared.changes.len())
                .ok_or(DomError::RepresentationExhausted)?;
            if outputs
                .checked_add(3 + usize::from(active_hook))
                .is_none_or(|outputs| outputs > op.limits.outputs)
            {
                return Err(DomError::SourceOutputLimit);
            }
            let canonical = storage::state(&prepared.state, || op.draft.source_step(cancel))?;
            op.history_after(self.dom, true, prepared.changes.len(), cancel)?;
            // Checkpoint already admits its inline Draft. The wrapper's remaining
            // inline fields and actual new Dialog Arc body were admitted above;
            // keep that same relative formation through the exclusive fence.
            if let (Some(destination), Some(formation)) = (prepared.formation.as_ref(), formation) {
                destination.absorb(formation);
            }
            let prepared = self.dom.prepare_publication(prepared, cancel)?;
            Ok((prepared, canonical, outputs, required_phases - 1))
        })();
        if result.is_err() {
            checkpoint.restore(&mut op.draft);
            self.dom.source_operation = Some(op);
            return Err(result.err().expect("failure"));
        }
        self.dom.source_operation = Some(op);
        let (prepared, canonical, outputs, remaining_phases) = result.expect("formed publication");
        Ok(DomDialogToggleQueueCommit {
            lease: self,
            prepared: Some(prepared),
            plan,
            task: Some(DomDialogToggleTask {
                info: plan.info,
                binding: Arc::downgrade(&source.binding),
            }),
            canonical,
            outputs,
            remaining_phases,
            checkpoint: Some(checkpoint),
        })
    }
    /// Accept the exact real queue publication once, preserving it on refusal.
    pub fn accept_dialog_toggle_queue(
        &mut self,
        request: &DomSourceDialogRequest,
        publication: &mut Option<DomDialogToggleQueuePublication>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let candidate = publication
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let op = self
            .dom
            .source_operation
            .as_mut()
            .filter(|op| op.id == self.id && request.operation == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let frame = op
            .draft
            .dialog_show
            .as_mut()
            .ok_or(DomError::InvalidSourceOperation)?;
        if candidate.operation != self.id
            || op.dialog_queue != Some(candidate.task)
            || candidate.task.target() != request.subject
            || frame.cursor.action() != request.action
        {
            return Err(DomError::InvalidSourceOperation);
        }
        op.draft.source_step(cancel)?;
        op.draft
            .dialog_show
            .as_mut()
            .expect("frame")
            .cursor
            .reply(surgeist_html::HtmlDialogReply::Applied)
            .map_err(DomError::DialogProtocol)?;
        op.dialog_queue = None;
        publication.take();
        Ok(())
    }
}

impl DomManipulationTaskSource {
    /// Begin the mandatory event/cleanup root of this actual affine queued task.
    /// The task remains in its original Runtime FIFO; no new task is queued.
    pub fn begin_dialog_toggle_task<'a>(
        &self,
        dom: &'a mut Dom,
        task: &mut Option<DomDialogToggleTask>,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'a>, DomError> {
        self.qualify(dom)?;
        dom.require_sync_source_idle()?;
        let actual = task.as_ref().ok_or(DomError::InvalidSourceOperation)?;
        if !actual
            .binding
            .upgrade()
            .is_some_and(|binding| Arc::ptr_eq(&binding, &self.binding))
        {
            return Err(DomError::InvalidSourceOperation);
        }
        // Classification is target-first; a retired generation is never
        // resolved through a reused slot or converted into another task.
        if dom
            .snapshot()
            .node(actual.info.id.target())?
            .as_dialog()
            .is_none()
        {
            return Err(DomError::InvalidSourceOperation);
        }
        let mut lease = dom.begin_source_root(
            OperationRoot::DialogToggleTask(actual.info),
            Arc::new(BatchIdentity),
            limits,
            cancel,
        )?;
        lease.task_authorization = Some(self.binding.clone());
        // Original engine formation (including closing headroom and executor)
        // can still refuse. Until its genuine observation exists, keep the
        // queued task's affine permission with the original caller. Ordinary
        // unpublished-root rollback then cannot strand that task on retry.
        if !matches!(lease.advance(cancel)?, DomSourceProgress::Request(DomSourceRequest::Event(ref request)) if request.kind() == crate::DomSourceEventKind::Toggle)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        task.take();
        Ok(lease)
    }
    /// Resume only this original actual task's canonical dispatch/cleanup root.
    /// A copied pending Source operation identity is insufficient authority.
    pub fn resume_dialog_toggle_task<'a>(
        &self,
        dom: &'a mut Dom,
        task: DomManipulationTaskId,
        operation: DomSourceOperationId,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'a>, DomError> {
        self.qualify(dom)?;
        if !dom.source_operation.as_ref().is_some_and(|current| current.id == operation && matches!(current.commands, OperationRoot::DialogToggleTask(info) if info.id == task)) {
            return Err(DomError::InvalidSourceOperation);
        }
        let mut lease = dom.resume_source_permission(operation, limits, cancel)?;
        lease.task_authorization = Some(self.binding.clone());
        Ok(lease)
    }
}
