//! Final private Source publication joined to the actual element task bank.

use super::*;
use crate::internal::{
    details::{DomDetailsToggleIntention, DomElementTaskAssociation},
    element_tasks::{Binding, DomElementTaskId, DomElementTaskSource},
};

#[derive(Clone, Copy)]
pub(in crate::internal::store) enum Kind {
    ObjectRepresentation { node_document: Key },
    TextSelection,
}

impl Kind {
    pub(in crate::internal::store) fn unavailable(self) -> DomError {
        match self {
            Self::ObjectRepresentation { .. } => DomError::ObjectTaskSourceUnavailable,
            Self::TextSelection => DomError::TextSelectionTaskSourceUnavailable,
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::internal::store) struct Pending {
    pub(in crate::internal::store) target: Key,
    pub(in crate::internal::store) association: DomElementTaskAssociation,
    pub(in crate::internal::store) kind: Kind,
}

impl Draft {
    pub(in crate::internal::store) fn queue_final_element_task(
        &mut self,
        pending: Pending,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut tasks = self
            .final_element_tasks
            .take()
            .unwrap_or_else(|| self.effect_vec());
        let admission = self.select_reserve(&mut tasks, 1, cancel);
        if admission.is_ok() {
            tasks.push(pending);
        }
        self.final_element_tasks = Some(tasks);
        admission
    }

    pub(in crate::internal::store) fn queue_text_selection_event(
        &mut self,
        target: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if let Some(association) = self.element_task_association(target, cancel)? {
            self.queue_final_element_task(
                Pending {
                    target,
                    association,
                    kind: Kind::TextSelection,
                },
                cancel,
            )?;
        }
        Ok(())
    }
}

/// An original completed private Source edit awaiting its exact element tasks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceElementTaskQueueRequest {
    operation: DomSourceOperationId,
    count: usize,
}

impl DomSourceElementTaskQueueRequest {
    pub(super) const fn new(operation: DomSourceOperationId, count: usize) -> Self {
        Self { operation, count }
    }

    /// The number of ordered task occurrences the edit reached.
    pub const fn count(self) -> usize {
        self.count
    }
}

/// Immutable inputs of an actual queued representation task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomObjectRepresentationTaskInfo {
    id: DomElementTaskId,
    association: DomElementTaskAssociation,
    node_document: DocumentId,
}

impl DomObjectRepresentationTaskInfo {
    /// The task serial and exact receiver generation.
    pub const fn id(self) -> DomElementTaskId {
        self.id
    }

    /// The original relevant Global and its associated Document.
    pub const fn queue_association(self) -> DomElementTaskAssociation {
        self.association
    }

    /// The node Document when this task was queued. Adoption can change the
    /// current node Document without moving the original task to another loop.
    pub const fn queued_node_document(self) -> DocumentId {
        self.node_document
    }
}

/// Affine permission to execute this original Object task.
#[derive(Debug)]
pub struct DomObjectRepresentationTask {
    info: DomObjectRepresentationTaskInfo,
    pub(in crate::internal::store) binding: std::sync::Weak<Binding>,
}

impl DomObjectRepresentationTask {
    /// Immutable task inputs retained by the admitted queue.
    pub const fn info(&self) -> DomObjectRepresentationTaskInfo {
        self.info
    }
}

/// One reached select event's immutable target and original task association.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomTextSelectionTaskInfo {
    id: DomElementTaskId,
    association: DomElementTaskAssociation,
}

impl DomTextSelectionTaskInfo {
    pub const fn id(self) -> DomElementTaskId {
        self.id
    }

    pub const fn queue_association(self) -> DomElementTaskAssociation {
        self.association
    }
}

/// Prospective element task data used to admit the actual bank entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceElementTaskInfo {
    ObjectRepresentation(DomObjectRepresentationTaskInfo),
    TextSelection(DomTextSelectionTaskInfo),
}

/// Owned task transferred at the original Source publication fence.
#[derive(Debug)]
pub enum DomQueuedSourceElementTask {
    ObjectRepresentation(DomObjectRepresentationTask),
    TextSelection(DomTextSelectionTaskInfo),
}

impl DomQueuedSourceElementTask {
    fn info(&self) -> DomSourceElementTaskInfo {
        match self {
            Self::ObjectRepresentation(task) => {
                DomSourceElementTaskInfo::ObjectRepresentation(task.info())
            }
            Self::TextSelection(info) => DomSourceElementTaskInfo::TextSelection(*info),
        }
    }
}

/// Owned transfer of the original tasks to the Runtime bank. The original
/// temporary backing stays charged until the transfer iterator is dropped.
pub struct DomQueuedSourceElementTasks {
    tasks: std::vec::IntoIter<DomQueuedSourceElementTask>,
    _formation: Option<formation::Formation>,
}

impl Iterator for DomQueuedSourceElementTasks {
    type Item = DomQueuedSourceElementTask;

    fn next(&mut self) -> Option<Self::Item> {
        self.tasks.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.tasks.size_hint()
    }
}

impl ExactSizeIterator for DomQueuedSourceElementTasks {}

/// Exclusive no-callback publication after the actual task bank has prepared
/// room and work for these exact tasks.
pub struct DomSourceElementTaskQueueCommit<'a, 'dom> {
    lease: &'a mut DomSourceLease<'dom>,
    segment: Option<SegmentPublication>,
    tasks: Vec<DomQueuedSourceElementTask>,
    formation: Option<formation::Formation>,
    checkpoint: Option<checkpoint::Checkpoint>,
}

impl DomSourceElementTaskQueueCommit<'_, '_> {
    /// Prospective payloads for the actual Runtime bank reservation.
    pub fn tasks(&self) -> impl ExactSizeIterator<Item = DomSourceElementTaskInfo> + '_ {
        self.tasks.iter().map(DomQueuedSourceElementTask::info)
    }

    /// Complete the canonical edit and transfer exactly its task permits. The
    /// task owner installs its preadmitted rows in the same exclusive fence.
    pub fn publish(
        mut self,
    ) -> Result<(DomSourceCompletion, DomQueuedSourceElementTasks), DomError> {
        let segment = self
            .segment
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        let mut op = self
            .lease
            .dom
            .source_operation
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        let commit = segment.publish(&mut op, self.lease.dom).commit;
        let completion = DomSourceCompletion {
            operation: op.id,
            cancelled: false,
            published: std::mem::take(&mut op.historical),
            commit,
            installation: None,
            custom_element_disposition: op
                .custom_element
                .as_ref()
                .and_then(custom_element::Pending::disposition),
            fullscreen_exit_return: None,
            design_mode_range_disposition: None,
            dialog_toggle_disposition: None,
        };
        self.checkpoint.take();
        drop(op);
        self.lease.complete = true;
        self.lease.dom.source_operation = self.lease.dom.source_parents.pop();
        #[cfg(test)]
        self.lease.dom.observe_source_lifecycle();
        Ok((
            completion,
            DomQueuedSourceElementTasks {
                tasks: std::mem::take(&mut self.tasks).into_iter(),
                _formation: self.formation.take(),
            },
        ))
    }
}

impl Drop for DomSourceElementTaskQueueCommit<'_, '_> {
    fn drop(&mut self) {
        if let Some(checkpoint) = self.checkpoint.take()
            && let Some(op) = self.lease.dom.source_operation.as_mut()
        {
            checkpoint.restore(&mut op.draft);
        }
    }
}

impl Dom {
    /// Discard one original unpublished edit awaiting element task admission.
    /// Existing task-bank entries and earlier canonical state are preserved.
    pub fn cancel_pending_element_task_admission(
        &mut self,
        operation: DomSourceOperationId,
    ) -> Result<(), DomError> {
        if !self.source_operation.as_ref().is_some_and(|op| {
            op.id == operation && !op.published && op.pending_element_queue.is_some()
        }) {
            return Err(DomError::InvalidSourceOperation);
        }
        self.source_operation = self.source_parents.pop();
        #[cfg(test)]
        self.observe_source_lifecycle();
        Ok(())
    }
}

impl<'dom> DomSourceLease<'dom> {
    fn qualify_element_task_request(
        &self,
        request: DomSourceElementTaskQueueRequest,
    ) -> Result<(), DomError> {
        if request.operation == self.id
            && self
                .dom
                .source_operation
                .as_ref()
                .is_some_and(|op| !op.cancelled && op.pending_element_queue == Some(request))
        {
            Ok(())
        } else {
            Err(DomError::InvalidSourceOperation)
        }
    }

    /// Borrow the exact earlier canonical Details occurrences for task-bank
    /// intake ahead of the newly reached occurrences.
    pub fn element_task_prior_details<'snapshot>(
        &self,
        request: DomSourceElementTaskQueueRequest,
        snapshot: &'snapshot DomSnapshot,
    ) -> Result<&'snapshot [DomDetailsToggleIntention], DomError> {
        self.qualify_element_task_request(request)?;
        if snapshot.stamp() != self.dom.snapshot().stamp()
            || !Arc::ptr_eq(&snapshot.state, &self.dom.state)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        Ok(&snapshot.state.details_intentions)
    }

    /// Prepare canonical publication using the original bank's reserved task
    /// serials. Dropping the fence leaves the private edit and its request live.
    pub fn prepare_element_task_queue<'a>(
        &'a mut self,
        request: DomSourceElementTaskQueueRequest,
        source: &DomElementTaskSource,
        first_serial: u64,
        cancel: &AtomicBool,
    ) -> Result<DomSourceElementTaskQueueCommit<'a, 'dom>, DomError> {
        self.qualify_element_task_request(request)?;
        source.qualify(self.dom)?;
        let mut op = self
            .dom
            .source_operation
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
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
        let prepared = (|| {
            let formation = formation::Formation::new(&op.draft, cancel)?;
            if let Some(formation) = &formation {
                formation.retain_bytes(
                    size_of::<DomSourceElementTaskQueueCommit<'_, '_>>()
                        .checked_sub(size_of::<checkpoint::Checkpoint>())
                        .ok_or(DomError::RepresentationExhausted)?,
                    cancel,
                )?;
            }
            op.draft.queue_details_clear(cancel)?;
            let segment = op
                .prepare_segment(self.dom, false, PublicationTarget::None, None, cancel)
                .map_err(|error| match error {
                    crate::internal::hosted::DomHostedError::Dom(error) => error,
                    _ => op
                        .draft
                        .final_element_tasks
                        .as_ref()
                        .and_then(|tasks| tasks.first())
                        .map_or(DomError::InvalidSourceOperation, |task| {
                            task.kind.unavailable()
                        }),
                })?;
            let staged = op
                .draft
                .final_element_tasks
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?;
            if staged.len() != request.count {
                return Err(DomError::InvalidSourceOperation);
            }
            let mut tasks = Vec::new();
            formation::reserve(&mut tasks, staged.len(), formation.as_ref(), cancel)?;
            for (offset, task) in staged.iter().enumerate() {
                op.draft.source_step(cancel)?;
                let target = match op.draft.live_key(task.target) {
                    Key::Existing(node) => node,
                    Key::Created(index) => segment
                        .prepared
                        .allocation_ids
                        .get(index)
                        .ok_or(DomError::UnavailableCreation)?,
                };
                let serial = first_serial
                    .checked_add(
                        u64::try_from(offset).map_err(|_| DomError::RepresentationExhausted)?,
                    )
                    .ok_or(DomError::RepresentationExhausted)?;
                let id = source.task_id(serial, target)?;
                tasks.push(match task.kind {
                    Kind::ObjectRepresentation { node_document } => {
                        let node_document = op
                            .draft
                            .published_id(node_document)
                            .map(DocumentId)
                            .ok_or(DomError::WrongDocument)?;
                        DomQueuedSourceElementTask::ObjectRepresentation(
                            DomObjectRepresentationTask {
                                info: DomObjectRepresentationTaskInfo {
                                    id,
                                    association: task.association,
                                    node_document,
                                },
                                binding: Arc::downgrade(&source.binding),
                            },
                        )
                    }
                    Kind::TextSelection => {
                        DomQueuedSourceElementTask::TextSelection(DomTextSelectionTaskInfo {
                            id,
                            association: task.association,
                        })
                    }
                });
            }
            Ok((segment, tasks, formation))
        })();
        match prepared {
            Ok((segment, tasks, formation)) => {
                self.dom.source_operation = Some(op);
                Ok(DomSourceElementTaskQueueCommit {
                    lease: self,
                    segment: Some(segment),
                    tasks,
                    formation,
                    checkpoint: Some(checkpoint),
                })
            }
            Err(error) => {
                checkpoint.restore(&mut op.draft);
                self.dom.source_operation = Some(op);
                Err(error)
            }
        }
    }
}
