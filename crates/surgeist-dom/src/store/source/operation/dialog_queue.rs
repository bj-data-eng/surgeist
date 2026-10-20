//! Actual queue publication for the original Dialog Source cursor.
use super::*;
use crate::{DomElementTaskAssociation, DomElementTaskId, DomElementTaskSource};
use surgeist_html::{HtmlDialogAction, HtmlDialogState, HtmlDialogToggleDecision};

/// Immutable payload of one real queued Dialog toggle task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDialogToggleTaskInfo {
    pub(crate) id: DomElementTaskId,
    pub(crate) old: HtmlDialogState,
    pub(crate) new: HtmlDialogState,
    pub(crate) association: Option<DomElementTaskAssociation>,
}
impl DomDialogToggleTaskInfo {
    /// Exact task and receiver generation.
    pub const fn id(self) -> DomElementTaskId {
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
    /// The admitted standard methods supply an actual null source.
    pub const fn source(self) -> Option<NodeId> {
        None
    }
}
/// Plain close notification, without ToggleEvent state or tracker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDialogCloseTaskInfo {
    pub(crate) id: DomElementTaskId,
    pub(crate) association: Option<DomElementTaskAssociation>,
}
impl DomDialogCloseTaskInfo {
    /// Exact task serial and target generation.
    pub const fn id(self) -> DomElementTaskId {
        self.id
    }
    /// Immutable original queue-associated Global and Document.
    pub const fn queue_association(self) -> Option<DomElementTaskAssociation> {
        self.association
    }
}
/// Closed Dialog element-task payload; its event source is intrinsic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomDialogTaskInfo {
    /// Coalesced ToggleEvent notification on the DOM-manipulation source.
    Toggle(DomDialogToggleTaskInfo),
    /// Noncoalesced plain close notification on the user-interaction source.
    Close(DomDialogCloseTaskInfo),
}
impl DomDialogTaskInfo {
    /// Exact task serial and target generation.
    pub const fn id(self) -> DomElementTaskId {
        match self {
            Self::Toggle(info) => info.id,
            Self::Close(info) => info.id,
        }
    }
    /// Immutable original queue-associated Global and Document.
    pub const fn queue_association(self) -> Option<DomElementTaskAssociation> {
        match self {
            Self::Toggle(info) => info.association,
            Self::Close(info) => info.association,
        }
    }
    /// Actual source intrinsic to the closed payload.
    pub const fn source(self) -> crate::DomElementTaskSourceKind {
        match self {
            Self::Toggle(_) => crate::DomElementTaskSourceKind::DomManipulation,
            Self::Close(_) => crate::DomElementTaskSourceKind::UserInteraction,
        }
    }
    /// Toggle data only; plain close notifications contain none.
    pub const fn toggle(self) -> Option<DomDialogToggleTaskInfo> {
        if let Self::Toggle(info) = self {
            Some(info)
        } else {
            None
        }
    }
}
/// Affine permission belonging to the actual queued task, never a copied ID.
#[derive(Debug)]
pub struct DomDialogTask {
    pub(crate) info: DomDialogTaskInfo,
    pub(crate) binding: std::sync::Weak<crate::element_tasks::Binding>,
}
impl DomDialogTask {
    /// Immutable historical task inputs.
    pub const fn info(&self) -> DomDialogTaskInfo {
        self.info
    }
}
/// Exact original request's prepared language decision and real task inputs.
#[derive(Clone, Copy, Debug)]
pub struct DomDialogTaskQueuePlan {
    operation: DomSourceOperationId,
    info: DomDialogTaskInfo,
    replacing: Option<DomElementTaskId>,
}
impl DomDialogTaskQueuePlan {
    /// Actual candidate task payload.
    pub const fn task(self) -> DomDialogTaskInfo {
        self.info
    }
    /// Only this still-queued task may be removed by replacement.
    pub const fn replacing(self) -> Option<DomElementTaskId> {
        self.replacing
    }
}
/// Move-only evidence of the actual no-callback queue/tracker publication.
#[derive(Debug)]
pub struct DomDialogTaskQueuePublication {
    operation: DomSourceOperationId,
    task: DomElementTaskId,
}
impl DomDialogTaskQueuePublication {
    /// Original operation which performed this exact real queue publication.
    pub const fn operation(&self) -> DomSourceOperationId {
        self.operation
    }
}
/// Exclusive canonical publication loan. No fallible work or user callback is
/// allowed between installing the prepared real queue and consuming this loan.
pub struct DomDialogTaskQueueCommit<'a, 'dom> {
    lease: &'a mut DomSourceLease<'dom>,
    prepared: Option<crate::store::PreparedPublication>,
    rebase: Option<transaction::PreparedRebase>,
    sharing: Option<value_sharing::CloseReady>,
    plan: DomDialogTaskQueuePlan,
    task: Option<DomDialogTask>,
    outputs: usize,
    remaining_phases: u64,
    checkpoint: Option<checkpoint::Checkpoint>,
}
impl DomDialogTaskQueueCommit<'_, '_> {
    /// Publish the already admitted canonical tracker after the actual owner
    /// removes/clears/appends/stores its prepared task. This fence cannot fail.
    pub fn publish(mut self) -> (DomDialogTask, DomDialogTaskQueuePublication) {
        let mut prepared = self.prepared.take().expect("prepared fence");
        let rebase = self
            .rebase
            .take()
            .expect("prepared current-root continuation");
        self.lease
            .dom
            .source_operation
            .as_mut()
            .expect("exclusive original operation")
            .draft
            .close_sharing_publication(
                &mut prepared,
                self.sharing.take(),
                self.checkpoint.as_ref(),
            );
        let crate::store::CommittedPublication { commit, committed } =
            self.lease.dom.publish_prepared_with_proof(prepared);
        let op = self
            .lease
            .dom
            .source_operation
            .as_mut()
            .expect("exclusive original operation");
        op.draft.commit_publication(rebase, &committed);
        op.outputs = self.outputs;
        op.phases += 1;
        op.published = true;
        op.event_exposed = true;
        op.dialog_queue = Some(self.plan.info.id());
        op.headroom = Some(headroom::Reservation {
            remaining_phases: self.remaining_phases,
        });
        op.command_admitted = true;
        drop(commit);
        self.checkpoint.take();
        (
            self.task.take().expect("actual task permission"),
            DomDialogTaskQueuePublication {
                operation: self.plan.operation,
                task: self.plan.info.id(),
            },
        )
    }
}
impl Drop for DomDialogTaskQueueCommit<'_, '_> {
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
    pub fn dialog_task_prior_details<'snapshot>(
        &self,
        request: &DomSourceDialogRequest,
        snapshot: &'snapshot DomSnapshot,
    ) -> Result<&'snapshot [crate::DomDetailsToggleIntention], DomError> {
        if request.operation != self.id
            || !matches!(
                request.action,
                HtmlDialogAction::QueueToggle { .. } | HtmlDialogAction::QueueClose
            )
        {
            return Err(DomError::InvalidSourceOperation);
        }
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let Some(crate::store::method::Frame::Dialog(frame)) = operation.draft.method.as_ref()
        else {
            return Err(DomError::InvalidSourceOperation);
        };
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
    pub fn retain_element_task_payload(
        &mut self,
        source: &DomElementTaskSource,
        bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<crate::DomElementTaskRetention, DomError> {
        source.retain_payload(self.dom, bytes, cancel)
    }
    /// Prepare the actual HTML queue decision without queueing or acknowledging.
    pub fn dialog_task_queue_plan(
        &mut self,
        request: &DomSourceDialogRequest,
        source: &DomElementTaskSource,
        serial: u64,
        cancel: &AtomicBool,
    ) -> Result<DomDialogTaskQueuePlan, DomError> {
        source.qualify(self.dom)?;
        let op = self
            .dom
            .source_operation
            .as_mut()
            .filter(|op| op.id == self.id && request.operation == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let Some(crate::store::method::Frame::Dialog(frame)) = op.draft.method.as_ref() else {
            return Err(DomError::InvalidSourceOperation);
        };
        if op.draft.published_id(frame.subject) != Some(request.subject)
            || frame.cursor.action() != request.action
            || op.dialog_queue.is_some()
        {
            return Err(DomError::InvalidSourceOperation);
        }
        let subject = frame.subject;
        op.refresh_admission(cancel)?;
        let id = source.task_id(serial, request.subject)?;
        let association = op.draft.element_task_association(subject, cancel)?;
        let (info, replacing) = match request.action {
            HtmlDialogAction::QueueToggle {
                old,
                new,
                source:
                    surgeist_html::HtmlDialogEventSource::Null
                    | surgeist_html::HtmlDialogEventSource::OperationSource,
            } => {
                let tracker = op.draft.dialog_state(subject)?.toggle_tracker;
                let decision =
                    HtmlDialogToggleDecision::new(old, new, tracker.map(|tracker| tracker.old));
                (
                    DomDialogTaskInfo::Toggle(DomDialogToggleTaskInfo {
                        id,
                        old: decision.old_state(),
                        new: decision.new_state(),
                        association,
                    }),
                    tracker.map(|tracker| tracker.task),
                )
            }
            HtmlDialogAction::QueueClose => (
                DomDialogTaskInfo::Close(DomDialogCloseTaskInfo { id, association }),
                None,
            ),
            _ => return Err(DomError::InvalidSourceOperation),
        };
        Ok(DomDialogTaskQueuePlan {
            operation: self.id,
            info,
            replacing,
        })
    }
    /// Complete all fallible canonical formation before the actual queue fence.
    pub fn prepare_dialog_task_queue<'a>(
        &'a mut self,
        request: &DomSourceDialogRequest,
        source: &DomElementTaskSource,
        plan: DomDialogTaskQueuePlan,
        cancel: &AtomicBool,
    ) -> Result<DomDialogTaskQueueCommit<'a, 'dom>, DomError> {
        let actual = self.dialog_task_queue_plan(request, source, plan.info.id().serial, cancel)?;
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
                    size_of::<DomDialogTaskQueueCommit<'_, '_>>()
                        .checked_sub(size_of::<checkpoint::Checkpoint>())
                        .ok_or(DomError::RepresentationExhausted)?,
                    cancel,
                )?;
            }
            let (subject, locals) = {
                let Some(crate::store::method::Frame::Dialog(frame)) = op.draft.method.as_ref()
                else {
                    return Err(DomError::InvalidSourceOperation);
                };
                (frame.subject, frame.locals)
            };
            if let DomDialogTaskInfo::Toggle(info) = plan.info {
                // The selected guard owns the complete new native body and
                // its actual string occurrences. The outer formation owns
                // only the queue fence wrapper, never a second native copy.
                let mut state = op.draft.dialog_state_mut(subject, cancel)?;
                state.toggle_tracker = Some(crate::dialog::DomDialogToggleTracker {
                    task: info.id,
                    old: info.old,
                });
                state.commit(cancel)?;
            }
            op.draft.queue_details_clear(cancel)?;
            let mut prepared = op.draft.finish_segment(self.dom, cancel)?;
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
            let (required_phases, future_outputs) = match (plan.info, locals) {
                (DomDialogTaskInfo::Close(_), _) => (1 + headroom::FINAL_PUBLICATION, 1),
                (DomDialogTaskInfo::Toggle(_), crate::store::dialog::MethodLocals::Close) => {
                    let state = op.draft.dialog_state(subject)?;
                    let watcher = state.watcher.is_some();
                    let modal = state.is_modal();
                    op.draft
                        .admit_dialog_occurrences(u64::from(watcher), cancel)?;
                    let document = op.draft.node(subject)?.document();
                    let member = match op.draft.node(document)?.interactive_document() {
                        Some(transaction::InteractiveRead::Known { open_dialogs, .. }) => {
                            let mut found = false;
                            for item in open_dialogs.iter() {
                                op.draft.source_step(cancel)?;
                                found |= item == subject;
                            }
                            found
                        }
                        _ => false,
                    };
                    let changed_result = match op.active_close_result() {
                        Some(result) => {
                            let old = op.draft.dialog_state(subject)?.return_value();
                            let mut same = old.len() == result.len();
                            if same {
                                for (old, new) in old.code_units().iter().zip(result.code_units()) {
                                    op.draft.source_step(cancel)?;
                                    if old != new {
                                        same = false;
                                        break;
                                    }
                                }
                            }
                            !same
                        }
                        _ => false,
                    };
                    let request_changes_pending = if modal {
                        op.draft.source_step(cancel)?;
                        let document_node = op.draft.node(document)?;
                        let Some(transaction::InteractiveRead::Known {
                            top_layer,
                            pending_top_layer_removals,
                            ..
                        }) = document_node.interactive_document()
                        else {
                            return Err(DomError::DialogDocumentStateUnavailable);
                        };
                        let mut raw = false;
                        for item in top_layer.iter() {
                            op.draft.source_step(cancel)?;
                            raw |= item == subject;
                        }
                        let mut pending = false;
                        if raw {
                            for item in pending_top_layer_removals.iter() {
                                op.draft.source_step(cancel)?;
                                pending |= item == subject;
                            }
                        }
                        raw && !pending
                    } else {
                        false
                    };
                    (
                        headroom::dialog_close(watcher),
                        2 + usize::from(member)
                            + usize::from(watcher || changed_result || modal)
                            // A watcher stop publishes open-list removal first.
                            // Without that stop the same Document row coalesces
                            // with its ordinary open-list change when present.
                            + usize::from(request_changes_pending && (watcher || !member)),
                    )
                }
                (
                    DomDialogTaskInfo::Toggle(_),
                    crate::store::dialog::MethodLocals::RequestClose { .. },
                ) => return Err(DomError::InvalidSourceOperation),
                (DomDialogTaskInfo::Toggle(_), crate::store::dialog::MethodLocals::Show { .. }) => {
                    let document = op.draft.node(subject)?.document();
                    let original_window_active = match self
                        .dom
                        .snapshot()
                        .relevant_global(plan.info.id().target())
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
                        && op.draft.shadow_connected(subject, cancel)?;
                    let active_hook = reached_hook && original_window_active;
                    let structural_modal = op.is_show_modal_method() && active_hook;
                    let method_occurrences = if op.is_show_modal_method() {
                        2 * u64::from(structural_modal)
                    } else {
                        2
                    };
                    op.draft.admit_dialog_occurrences(
                        method_occurrences + u64::from(reached_hook),
                        cancel,
                    )?;
                    let future_outputs = if op.is_show_modal_method() && !reached_hook {
                        // Actual inactive hook: open attribute, then own marker
                        // release. This prefix has no remembered-focus write.
                        2
                    } else {
                        3 + usize::from(active_hook)
                    };
                    let modal_outputs = if structural_modal {
                        let document_node = op.draft.node(document)?;
                        let raw = match document_node.interactive_document() {
                            Some(transaction::InteractiveRead::Known { top_layer, .. }) => {
                                top_layer
                            }
                            _ => return Err(DomError::DialogDocumentStateUnavailable),
                        };
                        let mut contains = false;
                        for member in raw.iter() {
                            op.draft.source_step(cancel)?;
                            contains |= member == subject;
                        }
                        1 + usize::from(!contains)
                    } else {
                        0
                    };
                    (
                        headroom::dialog_show(active_hook, structural_modal),
                        future_outputs + modal_outputs
                            // Autofocus history is a distinct canonical domain,
                            // possibly on a different captured top Document.
                            + op.draft.dialog_autofocus_tail_outputs(subject, cancel)?,
                    )
                }
            };
            if op
                .phases
                .checked_add(required_phases)
                .is_none_or(|phases| phases > op.limits.phases)
            {
                return Err(DomError::SourcePublicationLimit);
            }
            let protected = self.dom.source_reserved_closing(cancel)?;
            self.dom
                .revision
                .0
                .checked_add(required_phases)
                .and_then(|revision| revision.checked_add(self.dom.hosted_revisions))
                .and_then(|revision| revision.checked_add(protected))
                .ok_or(DomError::RevisionExhausted)?;
            let outputs = op
                .outputs
                .checked_add(prepared.changes.len())
                .ok_or(DomError::RepresentationExhausted)?;
            if outputs
                .checked_add(future_outputs)
                .is_none_or(|outputs| outputs > op.limits.outputs)
            {
                return Err(DomError::SourceOutputLimit);
            }
            // Checkpoint already admits its fixed header. The wrapper's remaining
            // inline fields were admitted above; the selected Dialog guard
            // owns its separate native body through the exclusive fence.
            if let (Some(destination), Some(formation)) = (prepared.formation.as_ref(), formation) {
                destination.absorb(formation);
            }
            let mut prepared = self.dom.prepare_publication(prepared, cancel)?;
            let rebase = op.draft.prepare_rebase(&mut prepared, cancel)?;
            let sharing = op.draft.prepare_sharing_publication(
                prepared.changed,
                Some(&checkpoint),
                cancel,
            )?;
            Ok((prepared, rebase, sharing, outputs, required_phases - 1))
        })();
        if result.is_err() {
            checkpoint.restore(&mut op.draft);
            self.dom.source_operation = Some(op);
            return Err(result.err().expect("failure"));
        }
        self.dom.source_operation = Some(op);
        let (prepared, rebase, sharing, outputs, remaining_phases) =
            result.expect("formed publication");
        Ok(DomDialogTaskQueueCommit {
            lease: self,
            prepared: Some(prepared),
            rebase: Some(rebase),
            sharing,
            plan,
            task: Some(DomDialogTask {
                info: plan.info,
                binding: Arc::downgrade(&source.binding),
            }),
            outputs,
            remaining_phases,
            checkpoint: Some(checkpoint),
        })
    }
    /// Accept the exact real queue publication once, preserving it on refusal.
    pub fn accept_dialog_task_queue(
        &mut self,
        request: &DomSourceDialogRequest,
        publication: &mut Option<DomDialogTaskQueuePublication>,
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
        let Some(crate::store::method::Frame::Dialog(frame)) = op.draft.method.as_mut() else {
            return Err(DomError::InvalidSourceOperation);
        };
        if candidate.operation != self.id
            || op.dialog_queue != Some(candidate.task)
            || candidate.task.target() != request.subject
            || frame.cursor.action() != request.action
        {
            return Err(DomError::InvalidSourceOperation);
        }
        op.draft.source_step(cancel)?;
        op.draft.source_prepare_dialog_reply(cancel)?;
        let Some(crate::store::method::Frame::Dialog(frame)) = op.draft.method.as_mut() else {
            return Err(DomError::InvalidSourceOperation);
        };
        frame
            .cursor
            .reply(surgeist_html::HtmlDialogReply::Applied)
            .map_err(DomError::DialogProtocol)?;
        op.dialog_queue = None;
        publication.take();
        Ok(())
    }
}

impl DomElementTaskSource {
    /// Begin the mandatory event/cleanup root of this actual affine queued task.
    /// The task remains in its original Runtime FIFO; no new task is queued.
    pub fn begin_dialog_task<'a>(
        &self,
        dom: &'a mut Dom,
        task: &mut Option<DomDialogTask>,
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
            .node(actual.info.id().target())?
            .as_dialog()
            .is_none()
        {
            return Err(DomError::InvalidSourceOperation);
        }
        let mut lease = dom.begin_source_root(
            OperationRoot::DialogTask(actual.info),
            Arc::new(BatchIdentity),
            limits,
            cancel,
        )?;
        lease.task_authorization = Some(self.binding.clone());
        // Original engine formation (including closing headroom and executor)
        // can still refuse. Until its genuine observation exists, keep the
        // queued task's affine permission with the original caller. Ordinary
        // unpublished-root rollback then cannot strand that task on retry.
        if !matches!(lease.advance(cancel)?, DomSourceProgress::Request(DomSourceRequest::Event(ref request)) if matches!(request.kind(), crate::DomSourceEventKind::Toggle | crate::DomSourceEventKind::Close))
        {
            return Err(DomError::InvalidSourceOperation);
        }
        task.take();
        Ok(lease)
    }
    /// Resume only this original actual task's canonical dispatch/cleanup root.
    /// A copied pending Source operation identity is insufficient authority.
    pub fn resume_dialog_task<'a>(
        &self,
        dom: &'a mut Dom,
        task: DomElementTaskId,
        operation: DomSourceOperationId,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'a>, DomError> {
        self.qualify(dom)?;
        if !dom.source_operation.as_ref().is_some_and(|current| {
            current.id == operation
                && matches!(current.commands, OperationRoot::DialogTask(info) if info.id() == task)
        }) {
            return Err(DomError::InvalidSourceOperation);
        }
        let mut lease = dom.resume_source_permission(operation, limits, cancel)?;
        lease.task_authorization = Some(self.binding.clone());
        Ok(lease)
    }
}
