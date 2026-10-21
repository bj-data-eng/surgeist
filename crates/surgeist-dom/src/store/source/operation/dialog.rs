//! Initial Show event observation without an old canonical Draft.
use super::*;
use crate::source::event::{EventWork, ExecutorBinding};
use crate::source::*;
use std::sync::{Mutex, Weak, atomic::AtomicU8};
use surgeist_html::{HtmlDialogAction, HtmlDialogOutcome, HtmlDialogReply};

pub(super) struct ResumeWorkPolicy {
    work: Arc<EventWork>,
    old: u64,
    committed: bool,
}
impl ResumeWorkPolicy {
    pub(super) fn new(work: Arc<EventWork>, proposed: u64) -> Result<Self, DomError> {
        if work.work.load(Ordering::Relaxed) > proposed {
            return Err(DomError::SourceWorkLimit { limit: proposed });
        }
        let old = work.work_limit.swap(proposed, Ordering::Relaxed);
        Ok(Self {
            work,
            old,
            committed: false,
        })
    }
    pub(super) fn commit(&mut self) {
        self.committed = true;
    }
}
impl Drop for ResumeWorkPolicy {
    fn drop(&mut self) {
        if !self.committed {
            self.work.work_limit.store(self.old, Ordering::Relaxed);
        }
    }
}
#[expect(
    clippy::large_enum_variant,
    reason = "Draft stays inline in the already-admitted Box<Operation>; another box would add an owned allocation and change Source payload accounting"
)]
pub(super) enum OperationDraft {
    Segment(Draft),
    Observed(DialogObservation),
}
// Existing Style/child/installation/Select execution remains the original
// segmented engine. Its entry points select the Segment phase before use.
impl std::ops::Deref for OperationDraft {
    type Target = Draft;
    fn deref(&self) -> &Draft {
        match self {
            Self::Segment(draft) => draft,
            Self::Observed(_) => unreachable!("observation has no Draft"),
        }
    }
}
impl std::ops::DerefMut for OperationDraft {
    fn deref_mut(&mut self) -> &mut Draft {
        match self {
            Self::Segment(draft) => draft,
            Self::Observed(_) => unreachable!("observation has no Draft"),
        }
    }
}
pub(super) struct Usage {
    autofocus: usize,
    work: Arc<AtomicU64>,
    base: u64,
    encoding: u64,
    meta: u64,
    select: u64,
    semantic_work: usize,
    semantic_bytes: usize,
    input: input_processing::InputProcessing,
}
impl Usage {
    pub(super) fn admit_limits(&self, limits: DomLimits) -> Result<(), DomError> {
        admit(
            self.autofocus,
            limits.autofocus_candidates,
            DomResource::AutofocusCandidates,
        )?;
        self.input
            .admit_limits(limits.input_scratch, limits.input_work)?;
        if self.base > limits.base_work {
            return Err(DomError::BaseWorkLimit {
                limit: limits.base_work,
            });
        }
        if self.encoding > limits.encoding_url_work {
            return Err(DomError::EncodingUrlWorkLimit {
                limit: limits.encoding_url_work,
            });
        }
        if self.meta > limits.meta_work {
            return Err(DomError::Meta(crate::DomMetaError::WorkLimit {
                limit: limits.meta_work,
            }));
        }
        if self.select > limits.select_work {
            return Err(DomError::SelectWorkLimit {
                limit: limits.select_work,
            });
        }
        if let Some((_, semantic)) = limits.select_value_queries {
            for (resource, limit, used) in [
                (
                    surgeist_html_semantics::HtmlSemanticResource::Work,
                    semantic.work(),
                    self.semantic_work,
                ),
                (
                    surgeist_html_semantics::HtmlSemanticResource::AllocatedBytes,
                    semantic.allocated_bytes(),
                    self.semantic_bytes,
                ),
            ] {
                if used > limit {
                    return Err(DomError::InvalidSelectSemanticLimit {
                        resource,
                        limit,
                        used,
                    });
                }
            }
        }
        Ok(())
    }
}
struct DialogFiring {
    subject: NodeId,
    payload: crate::source::event::DomSourceEventPayload,
    cancelable: bool,
    kind: DomSourceEventKind,
    frame: Option<crate::store::dialog::MethodFrame>,
}
pub(super) struct DialogObservation {
    request: DomSourceEventRequest,
    owner: Arc<BatchIdentity>,
    frame: Option<crate::store::dialog::MethodFrame>,
    source_invocation: u64,
    pub(super) usage: Usage,
    limits: DomSourceLimits,
    accepted: Option<DomSourceEventOutcome>,
    inputs: storage::Payload,
    receiver: crate::DomDispatchCustody,
}
impl DialogObservation {
    pub(super) fn status(&self, operation: &Operation) -> DomSourceStatus {
        DomSourceStatus {
            operation: operation.id,
            published: operation.published,
            cancelled: operation.cancelled,
            cleanup: operation.failure,
            work: self.usage.work.load(Ordering::Relaxed),
            phases: operation.phases,
            parent: self.request.parent_operation(),
            phase: if operation.failure.is_some() {
                DomSourcePhase::PendingCleanup
            } else if self.accepted.is_some() {
                DomSourcePhase::EventReceiptAccepted
            } else {
                DomSourcePhase::EventObservation
            },
        }
    }
    fn payload(&self) -> Result<storage::Payload, DomError> {
        Ok(storage::Payload {
            // The actual Operation allocation remains an external owner header
            // while its counter/event atoms and temporary buffers are ledgered.
            bytes: size_of::<Operation>()
                .checked_add(self.inputs.bytes)
                .ok_or(DomError::RepresentationExhausted)?,
            units: self.inputs.units,
        })
    }
}
impl Drop for Operation {
    fn drop(&mut self) {
        if let Some(work) = self.event_work.upgrade() {
            work.stage.store(4, Ordering::Relaxed);
        }
    }
}
impl Operation {
    pub(in crate::store) fn dispatch_limit_formation(
        &self,
        cancel: &AtomicBool,
    ) -> Result<Option<formation::Formation>, DomError> {
        match &self.draft {
            OperationDraft::Segment(draft) => formation::Formation::new(draft, cancel),
            OperationDraft::Observed(_) => Ok(None),
        }
    }
    pub(super) fn observed_event_work(&self) -> Option<Arc<EventWork>> {
        self.observation()
            .map(|observation| observation.request.work.clone())
    }
    pub(super) fn observation(&self) -> Option<&DialogObservation> {
        match &self.draft {
            OperationDraft::Observed(observation) => Some(observation),
            OperationDraft::Segment(_) => None,
        }
    }
    fn observation_mut(&mut self) -> Option<&mut DialogObservation> {
        match &mut self.draft {
            OperationDraft::Observed(observation) => Some(observation),
            OperationDraft::Segment(_) => None,
        }
    }
    pub(in crate::store) fn id(&self) -> DomSourceOperationId {
        self.id
    }
    pub(in crate::store) fn inline_feedback_observation(&self) -> Option<crate::DomSourceEventId> {
        self.observation()
            .map(|observation| observation.request.identity())
    }
    pub(in crate::store) fn is_observing_event(&self) -> bool {
        matches!(self.draft, OperationDraft::Observed(_))
    }
    pub(in crate::store) fn has_event_tail(&self) -> bool {
        self.event_exposed && self.observation().is_none()
    }
    pub(in crate::store) fn permits_reentry(&self) -> bool {
        self.observation()
            .is_some_and(|observation| observation.request.work.active())
    }
    pub(super) fn admit_observation_limits(
        &mut self,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let observation = self.observation_mut().ok_or(DomError::InvalidSourceEvent)?;
        check_cancel(cancel)?;
        if limits.work < observation.usage.work.load(Ordering::Relaxed)
            || limits.nested_operations == 0
        {
            return Err(DomError::InvalidLimit(DomResource::SourceRetainedBytes));
        }
        // Actual current whole-owner storage is qualified by the Dom front before
        // this policy is changed; no accepted event is consumed on refusal.
        observation
            .request
            .work
            .work_limit
            .store(limits.work, Ordering::Relaxed);
        observation.limits = limits;
        Ok(())
    }
    pub(super) fn observe_dialog_method(
        &mut self,
        dom: &mut Dom,
        subject: NodeId,
        action: HtmlDialogAction,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, DomError> {
        let HtmlDialogAction::FireBeforeToggle {
            old,
            new,
            cancelable,
            source,
        } = action
        else {
            return Err(DomError::InvalidSourceEvent);
        };
        if !matches!(&self.commands, OperationRoot::Edit([Command::ShowDialog { node: NodeOperand::Existing(actual) } | Command::ShowModalDialog { node: NodeOperand::Existing(actual) } | Command::CloseDialog { node: NodeOperand::Existing(actual), .. } | Command::RequestCloseDialog { node: NodeOperand::Existing(actual), .. }]) if *actual == subject)
            || !matches!(
                source,
                surgeist_html::HtmlDialogEventSource::Null
                    | surgeist_html::HtmlDialogEventSource::OperationSource
            )
            || !self.draft.created.is_empty()
            || !self.draft.tickets.is_empty()
            || !self.draft.effects.is_empty()
        {
            return Err(DomError::InvalidSourceEvent);
        }
        let Some(crate::store::method::Frame::Dialog(frame)) = self.draft.method.as_ref() else {
            return Err(DomError::InvalidSourceEvent);
        };
        let frame = frame.before_focusing()?;
        self.observe_dialog_event(
            dom,
            DialogFiring {
                subject,
                payload: crate::source::event::DomSourceEventPayload::Toggle(
                    crate::DomSourceToggleEvent { old, new },
                ),
                cancelable,
                kind: DomSourceEventKind::BeforeToggle,
                frame: Some(frame),
            },
            cancel,
        )
    }
    pub(super) fn observe_dialog_task(
        &mut self,
        dom: &mut Dom,
        task: crate::DomDialogTaskInfo,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, DomError> {
        self.observe_dialog_event(
            dom,
            DialogFiring {
                subject: task.id().target(),
                payload: match task {
                    crate::DomDialogTaskInfo::Toggle(info) => {
                        crate::source::event::DomSourceEventPayload::Toggle(
                            crate::DomSourceToggleEvent {
                                old: info.old_state(),
                                new: info.new_state(),
                            },
                        )
                    }
                    crate::DomDialogTaskInfo::Close(_) => {
                        crate::source::event::DomSourceEventPayload::Close
                    }
                },
                cancelable: false,
                kind: match task {
                    crate::DomDialogTaskInfo::Toggle(_) => DomSourceEventKind::Toggle,
                    crate::DomDialogTaskInfo::Close(_) => DomSourceEventKind::Close,
                },
                frame: None,
            },
            cancel,
        )
    }
    pub(super) fn observe_request_close_cancel(
        &mut self,
        dom: &mut Dom,
        cancelable: bool,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, DomError> {
        let call = self
            .request_close
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        self.observe_dialog_event(
            dom,
            DialogFiring {
                subject: call.watcher.subject(),
                payload: crate::source::event::DomSourceEventPayload::Cancel,
                cancelable,
                kind: DomSourceEventKind::Cancel,
                frame: Some(call.outer.before_focusing()?),
            },
            cancel,
        )
    }
    fn observe_dialog_event(
        &mut self,
        dom: &mut Dom,
        firing: DialogFiring,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, DomError> {
        let DialogFiring {
            subject,
            payload,
            cancelable,
            kind,
            frame,
        } = firing;
        // Before exposing a callback, protect every publication it can leave
        // mandatory, including a watcher the callback may establish or replace.
        let possible = match (kind, frame.as_ref().map(|frame| frame.locals)) {
            (
                DomSourceEventKind::BeforeToggle,
                Some(crate::store::dialog::MethodLocals::Show { .. }),
            ) => headroom::dialog_show(true, self.is_show_modal_method()),
            (DomSourceEventKind::BeforeToggle, Some(crate::store::dialog::MethodLocals::Close)) => {
                headroom::dialog_close(true)
            }
            (
                DomSourceEventKind::Cancel,
                Some(crate::store::dialog::MethodLocals::RequestClose { .. }),
            ) if self.request_close.is_some() => headroom::dialog_close(true),
            (DomSourceEventKind::Toggle | DomSourceEventKind::Close, None)
                if matches!(self.commands, OperationRoot::DialogTask(_)) =>
            {
                headroom::FINAL_PUBLICATION
            }
            _ => return Err(DomError::InvalidSourceEvent),
        };
        let remaining = self.headroom.as_ref().map_or(possible, |reservation| {
            possible.max(reservation.remaining_phases)
        });
        // Caller phase policy can be raised on resume. Observation needs its
        // closing minimum; the actual queue/publication gates admit the reached
        // tail. Fixed canonical revision credit, unlike policy, remains whole.
        let close = self
            .phases
            .checked_add(headroom::FINAL_PUBLICATION)
            .ok_or(DomError::RepresentationExhausted)?;
        if close > self.limits.phases {
            return Err(DomError::SourcePublicationLimit);
        }
        let protected = dom.source_reserved_closing(cancel)?;
        dom.revision
            .0
            .checked_add(dom.hosted_revisions)
            .and_then(|revision| revision.checked_add(protected))
            .and_then(|revision| revision.checked_add(remaining))
            .ok_or(DomError::RevisionExhausted)?;
        let executor = dom
            .source_event_executor
            .as_ref()
            .and_then(Weak::upgrade)
            .filter(|binding| binding.live.load(Ordering::Relaxed))
            .ok_or(DomError::SourceEventExecutorUnavailable)?;
        let formation = formation::Formation::new(&self.draft, cancel)?;
        let event_custody = formation::EventWorkCustody::prepare(
            self.draft.custody_ledger(),
            self.draft.source_work_custody(),
            self.draft.source_ancestor_custody(),
            |usage| self.draft.admit_custody_usage(usage, cancel),
        )?;
        let occurrence = if self.request_close.is_some() {
            self.draft
                .source_invocation
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?
        } else {
            0
        };
        let request = DomSourceEventRequest {
            work: formation::allocated_arc(EventWork {
                executor: Arc::downgrade(&executor),
                identity: DomSourceEventId {
                    operation: self.id,
                    occurrence,
                },
                parent: dom.source_parents.last().map(|parent| parent.id),
                subject,
                kind,
                payload,
                cancelable,
                stage: AtomicU8::new(0),
                work: self.draft.source_work.source_arc().clone(),
                work_limit: AtomicU64::new(self.limits.work),
                scope: Mutex::new(Weak::new()),
                ancestors: self.draft.source_ancestors.clone(),
                bytes: formation::allocated_arc(AtomicUsize::new(0)),
                units: formation::allocated_arc(AtomicUsize::new(0)),
                storage: formation::SourceStorage::new(self.draft.custody_ledger().clone()),
            }),
        };
        let event_custody = event_custody.finish(request.work.clone());
        let limits = self.limits;

        let receiver =
            dom.register_dispatch_custody_with_formation(&[subject], formation.as_ref(), cancel)?;
        let autofocus = self.draft.autofocus_count(cancel)?;
        let usage = Usage {
            autofocus,
            work: self.draft.source_work.source_arc().clone(),
            base: self.draft.base_work.get(),
            encoding: self.draft.encoding_url_work.get(),
            meta: self.draft.meta_work.get(),
            select: self.draft.select_work.get(),
            semantic_work: self.draft.select_semantic_work.get(),
            semantic_bytes: self.draft.select_semantic_bytes.get(),
            input: std::mem::replace(
                &mut self.draft.input_processing,
                input_processing::InputProcessing::new(0, 0),
            ),
        };
        let mut inputs = self.command_storage;
        if let Some(value) = &self.request_close_result {
            inputs.string(value)?;
        }
        let observation = DialogObservation {
            receiver,
            request: request.clone(),
            owner: self.draft.owner.clone(),
            frame,
            source_invocation: if self.request_close.is_some() {
                occurrence
            } else {
                self.draft.source_invocation
            },
            usage,
            limits,
            accepted: None,
            inputs,
        };
        self.event_custody = Some(event_custody);
        self.draft = OperationDraft::Observed(observation);
        self.headroom = Some(headroom::Reservation {
            remaining_phases: remaining,
        });
        self.event_exposed = true;
        self.event_work = Arc::downgrade(&request.work);
        drop(formation); // Work is now owned by the admitted observation.
        self.command_storage = storage::Payload::default();
        self.custody_bytes = 0;
        self.custody_units = 0;
        Ok(DomSourceProgress::Request(DomSourceRequest::Event(request)))
    }
}
impl Dom {
    pub(in crate::store) fn admit_source_owner_payload(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.source_operation.is_none() && self.source_parents.is_empty() {
            return Ok(());
        }
        let total = self.source_observation_total(storage::Payload::default(), cancel)?;
        self.admit_source_observers(total, cancel)?;
        let custody = self.source_owner_custody_bytes()?;
        let custody_units = self.fullscreen_custody_units()?;
        let Some(operation) = self.source_operation.as_mut() else {
            return Ok(());
        };
        if operation.observation().is_some() {
            return Ok(());
        }
        let old = operation.custody_bytes;
        let old_units = operation.custody_units;
        let old_other = operation.draft.source_other_bytes;
        let old_other_units = operation.draft.source_other_units;
        operation.custody_bytes = custody;
        operation.custody_units = custody_units;
        if let Err(error) = operation.refresh_admission(cancel) {
            operation.custody_bytes = old;
            operation.custody_units = old_units;
            operation.draft.source_other_bytes = old_other;
            operation.draft.source_other_units = old_other_units;
            return Err(error);
        }
        Ok(())
    }
    /// Actual unfinished top which prevents an unrelated dispatcher invocation.
    /// Event reentry is permitted only by the exact live affine scope.
    pub fn source_dispatch_blocker(&self) -> Option<DomSourceOperationId> {
        self.source_operation
            .as_ref()
            .filter(|operation| !operation.permits_reentry())
            .map(|operation| operation.id)
    }
    /// Qualify current canonical reads for the actual running dispatcher only.
    /// This lends no snapshot completeness or permission to run a Source tail.
    pub(in crate::store) fn dispatch_hosted_read_work(
        &self,
        cancel: &AtomicBool,
    ) -> Result<Option<Arc<EventWork>>, DomError> {
        check_cancel(cancel)?;
        if self.state.incomplete_hosted.is_some() {
            return Err(DomError::IncompleteSourceOperation);
        }
        let Some(operation) = self.source_operation.as_ref() else {
            return if self.state.incomplete_source.is_none() {
                Ok(None)
            } else {
                Err(DomError::IncompleteSourceOperation)
            };
        };
        let observation = operation
            .observation()
            .filter(|_| operation.permits_reentry())
            .ok_or(DomError::IncompleteSourceOperation)?;
        self.qualify_source_event(&observation.request, 1)?;
        let work = &observation.request.work;
        let executor = self
            .source_event_executor
            .as_ref()
            .and_then(Weak::upgrade)
            .filter(|binding| binding.live.load(Ordering::Relaxed))
            .ok_or(DomError::InvalidSourceEvent)?;
        if executor.owner != self.owner
            || !work
                .executor
                .upgrade()
                .is_some_and(|issuer| Arc::ptr_eq(&issuer, &executor))
        {
            return Err(DomError::InvalidSourceEvent);
        }
        work.step(cancel)?;
        if let Some(incomplete) = self.state.incomplete_source {
            let mut qualified = operation.published && operation.id == incomplete;
            if !qualified {
                for parent in self.source_parents.iter().rev() {
                    work.step(cancel)?;
                    if parent.id == incomplete && parent.published && parent.observation().is_some()
                    {
                        qualified = true;
                        break;
                    }
                }
            }
            if !qualified {
                return Err(DomError::IncompleteSourceOperation);
            }
        }
        Ok(Some(work.clone()))
    }
    pub fn has_source_event_executor(&self) -> bool {
        self.source_event_executor.is_some()
    }
    pub fn bind_source_event_executor(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<DomSourceEventExecutor, DomError> {
        self.require_source_idle()?;
        check_cancel(cancel)?;
        if self.source_event_executor.is_some() {
            return Err(DomError::InvalidSourceEvent);
        }
        let binding = formation::allocated_arc(ExecutorBinding {
            owner: self.owner,
            live: AtomicBool::new(true),
        });
        self.source_event_executor = Some(Arc::downgrade(&binding));
        Ok(DomSourceEventExecutor { binding })
    }
    pub(in crate::store) fn source_work_ancestors(
        &self,
        excluding: Option<&Arc<AtomicU64>>,
    ) -> Option<Arc<EventWork>> {
        self.source_operation
            .iter()
            .map(Box::as_ref)
            .chain(self.source_parents.iter().rev().map(Box::as_ref))
            .filter_map(|operation| operation.observation())
            .find(|observation| {
                excluding.is_none_or(|work| !Arc::ptr_eq(work, &observation.usage.work))
            })
            .map(|observation| observation.request.work.clone())
    }
    pub(in crate::store) fn source_work_ancestor_custody(
        &self,
        excluding: Option<&Arc<AtomicU64>>,
    ) -> Option<Arc<formation::EventWorkCustody>> {
        self.source_operation
            .iter()
            .map(Box::as_ref)
            .chain(self.source_parents.iter().rev().map(Box::as_ref))
            .find(|operation| {
                operation.observation().is_some_and(|observation| {
                    excluding.is_none_or(|work| !Arc::ptr_eq(work, &observation.usage.work))
                })
            })
            .and_then(|operation| operation.event_custody.clone())
    }
    pub(in crate::store) fn dispatch_registration_formation(
        &self,
        cancel: &AtomicBool,
    ) -> Result<Option<formation::Formation>, DomError> {
        if self
            .source_operation
            .as_ref()
            .is_some_and(|operation| !operation.permits_reentry())
        {
            return Err(DomError::SourceOperationPending);
        }
        self.source_observation_formation(cancel)
    }
    pub(in crate::store) fn source_observation_formation(
        &self,
        cancel: &AtomicBool,
    ) -> Result<Option<formation::Formation>, DomError> {
        let Some(operation) = self.source_operation.as_ref() else {
            check_cancel(cancel)?;
            return Ok(None);
        };
        let observation = operation
            .observation()
            .ok_or(DomError::SourceOperationPending)?;
        let base = self.source_observation_external(cancel)?;
        let controlled = observation.request.work.storage.ledger().total();
        let total = storage::Payload {
            bytes: base
                .bytes
                .checked_add(controlled.bytes())
                .ok_or(DomError::RepresentationExhausted)?,
            units: base
                .units
                .checked_add(controlled.units())
                .ok_or(DomError::RepresentationExhausted)?,
        };
        self.admit_source_observers(total, cancel)?;
        let mut limits = observation.limits;
        for ancestor in self.source_observers() {
            let ancestor = ancestor.observation().expect("observer");
            limits.retained_bytes = limits.retained_bytes.min(ancestor.limits.retained_bytes);
            limits.retained_units = limits.retained_units.min(ancestor.limits.retained_units);
        }
        Ok(Some(formation::Formation::from_event(
            operation
                .event_custody
                .as_ref()
                .ok_or(DomError::InvalidSourceEvent)?
                .clone(),
            base,
            limits,
        )))
    }
    fn source_observers(&self) -> impl Iterator<Item = &Operation> {
        self.source_parents
            .iter()
            .map(Box::as_ref)
            .chain(self.source_operation.as_deref())
            .filter(|operation| operation.observation().is_some())
    }
    fn source_observation_payload(
        &self,
        cancel: &AtomicBool,
    ) -> Result<storage::Payload, DomError> {
        let mut payload = storage::Payload::default();

        for operation in self.source_observers() {
            let observation = operation.observation().expect("observer");
            observation.request.work.step_own(cancel)?;
            payload.append(observation.payload()?)?;
        }
        Ok(payload)
    }
    pub(in crate::store) fn admit_source_resume(
        &self,
        id: DomSourceOperationId,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let operation = self
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == id)
            .ok_or(DomError::InvalidSourceOperation)?;
        if let Some(observation) = operation.observation() {
            if limits.phases
                < operation
                    .phases
                    .checked_add(headroom::FINAL_PUBLICATION)
                    .ok_or(DomError::RepresentationExhausted)?
            {
                return Err(DomError::SourcePublicationLimit);
            }
            let total = self.source_observation_total(storage::Payload::default(), cancel)?;
            self.admit_source_observers(total, cancel)?;
            admit(
                total.bytes,
                limits.retained_bytes,
                DomResource::SourceRetainedBytes,
            )?;
            admit(
                total.units,
                limits.retained_units,
                DomResource::SourceRetainedUnits,
            )?;
            if limits.work < observation.usage.work.load(Ordering::Relaxed)
                || limits.outputs < operation.outputs
                || limits.identities < operation.identities
                || limits.nested_operations == 0
            {
                return Err(DomError::InvalidLimit(DomResource::SourceRetainedBytes));
            }
        }
        Ok(())
    }
    pub(in crate::store) fn source_retained_stack_bytes(&self) -> Result<usize, DomError> {
        self.source_parents
            .capacity()
            .checked_mul(size_of::<Box<Operation>>())
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<Box<Operation>>>()))
            .ok_or(DomError::RepresentationExhausted)
    }
    pub(in crate::store) fn source_dispatch_custody_bytes(
        &self,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<usize, DomError> {
        self.dispatch_custody
            .retained_bytes(step)?
            .checked_add(size_of::<crate::store::dispatch::CustodyBank>())
            .ok_or(DomError::RepresentationExhausted)
    }
    pub(super) fn step_source_observers(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        for operation in self.source_observers() {
            let observation = operation.observation().expect("observer");
            observation.request.work.step_own(cancel)?;
        }
        Ok(())
    }
    #[cfg(test)]
    pub(in crate::store) fn source_measurement_ancestors(&self) -> custody::Usage {
        let mut payload = storage::Payload::default();
        for operation in self.source_observers() {
            let observation = operation.observation().expect("observed Source ancestor");
            payload
                .append(observation.payload().expect("retained ancestor receipt"))
                .expect("retained ancestor sum");
        }
        custody::Usage::new(payload.bytes, payload.units)
    }
    #[cfg(test)]
    pub(in crate::store) fn observe_source_lifecycle(&self) {
        if !scale_metrics::ENABLED {
            return;
        }
        let external = match self
            .source_operation
            .as_ref()
            .map(|operation| &operation.draft)
        {
            Some(OperationDraft::Segment(draft)) => {
                custody::Usage::new(draft.source_other_bytes, draft.source_other_units)
            }
            Some(OperationDraft::Observed(_)) => {
                // These are cached input receipts and fixed owner headers. The
                // only iteration is the actual Source ancestor stack; no bank,
                // canonical payload, or effect frontier is inspected or metered.
                let mut payload = storage::Payload::default();
                for operation in self.source_observers() {
                    payload
                        .append(
                            operation
                                .observation()
                                .expect("observed ancestor")
                                .payload()
                                .expect("retained observer receipt"),
                        )
                        .expect("retained ancestor sum");
                }
                payload
                    .add_bytes(
                        self.source_retained_stack_bytes()
                            .expect("retained stack capacity"),
                    )
                    .expect("retained stack sum");
                payload
                    .add_bytes(self.source_owner_custody_bytes().expect("owner headers"))
                    .expect("retained owner sum");
                payload
                    .add_units(self.fullscreen_custody_units().expect("owner units"))
                    .expect("retained owner units");
                payload
                    .add_bytes(
                        self.source_dispatch_custody_bytes(|| Ok(()))
                            .expect("dispatch headers"),
                    )
                    .expect("retained dispatch sum");
                custody::Usage::new(payload.bytes, payload.units)
            }
            None => custody::Usage::default(),
        };
        let ledger = self.custody_registry.ledger();
        scale_metrics::source_owner_changed(
            ledger.measurement_identity(),
            ledger.total(),
            external,
            if self
                .source_operation
                .as_ref()
                .is_some_and(|operation| matches!(operation.draft, OperationDraft::Segment(_)))
            {
                self.source_measurement_ancestors()
            } else {
                custody::Usage::default()
            },
            self.source_operation.as_ref().map(|operation| operation.id),
        );
    }
    fn source_observation_external(
        &self,
        cancel: &AtomicBool,
    ) -> Result<storage::Payload, DomError> {
        let mut payload = self.source_observation_payload(cancel)?;
        payload.add_bytes(self.source_retained_stack_bytes()?)?;
        payload.add_bytes(self.source_owner_custody_bytes()?)?;
        payload.add_units(self.fullscreen_custody_units()?)?;
        payload.add_bytes(
            self.source_dispatch_custody_bytes(|| self.step_source_observers(cancel))?,
        )?;
        #[cfg(test)]
        if scale_metrics::ENABLED {
            let ledger = self.custody_registry.ledger();
            scale_metrics::source_owner_changed(
                ledger.measurement_identity(),
                ledger.total(),
                custody::Usage::new(payload.bytes, payload.units),
                custody::Usage::default(),
                self.source_operation.as_ref().map(|operation| operation.id),
            );
        }
        Ok(payload)
    }
    fn source_observation_total(
        &self,
        extra: storage::Payload,
        cancel: &AtomicBool,
    ) -> Result<storage::Payload, DomError> {
        let mut payload = self.source_observation_external(cancel)?;
        self.custody_registry.reconcile_releases();
        let usage = self.custody_registry.ledger().total();
        payload.add_bytes(usage.bytes())?;
        payload.add_units(usage.units())?;
        payload.append(extra)?;
        Ok(payload)
    }
    fn admit_source_observers(
        &self,
        total: storage::Payload,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        for operation in self.source_observers() {
            let observation = operation.observation().expect("observer");
            observation.request.work.step_own(cancel)?;
            admit(
                total.bytes,
                observation.limits.retained_bytes,
                DomResource::SourceRetainedBytes,
            )?;
            admit(
                total.units,
                observation.limits.retained_units,
                DomResource::SourceRetainedUnits,
            )?;
        }
        Ok(())
    }
    pub(crate) fn qualify_source_event(
        &self,
        request: &DomSourceEventRequest,
        stage: u8,
    ) -> Result<(), DomError> {
        let operation = self
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == request.operation())
            .ok_or(DomError::InvalidSourceEvent)?;
        let observation = operation
            .observation()
            .ok_or(DomError::InvalidSourceEvent)?;
        if request.subject().owner() != self.owner
            || !Arc::ptr_eq(&observation.request.work, &request.work)
            || request.work.stage.load(Ordering::Relaxed) != stage
        {
            return Err(DomError::InvalidSourceEvent);
        }
        Ok(())
    }
    /// Raise the allowances of this exact retained event observation, including
    /// an ancestor beneath a pending child. All seven fields must be monotonic.
    /// This changes policy only: it does not resume a cursor, dispatch, replace a
    /// receipt, or change the stack/tree. Finite lookup consumes the target's
    /// original work under the proposed cap; failed attempts retain that work
    /// while restoring old policy. Other exhausted observers cannot block this
    /// nonexecuting recovery control. Ordinary top resume still owns lowering.
    pub fn raise_source_event_limits(
        &mut self,
        request: &DomSourceEventRequest,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        if request.subject().owner() != self.owner || request.work.stage.load(Ordering::Relaxed) > 3
        {
            return Err(DomError::InvalidSourceEvent);
        }
        if limits.work < request.work.work.load(Ordering::Relaxed) {
            return Err(DomError::SourceWorkLimit { limit: limits.work });
        }
        if limits.work < request.work.work_limit.load(Ordering::Relaxed) {
            return Err(DomError::SourceLimitReduction);
        }
        let mut policy = ResumeWorkPolicy::new(request.work.clone(), limits.work)?;
        let matches = |operation: &Operation| {
            operation.id == request.operation()
                && operation.observation().is_some_and(|observation| {
                    Arc::ptr_eq(&observation.request.work, &request.work)
                })
        };
        request.work.step_own(cancel)?;
        let top = self
            .source_operation
            .as_ref()
            .is_some_and(|operation| matches(operation));
        let mut parent = None;
        if !top {
            for (index, operation) in self.source_parents.iter().enumerate().rev() {
                request.work.step_own(cancel)?;
                if matches(operation) {
                    parent = Some(index);
                    break;
                }
            }
        }
        let depth = if top {
            1
        } else {
            let index = parent.ok_or(DomError::InvalidSourceEvent)?;
            self.source_parents
                .len()
                .checked_add(1)
                .and_then(|count| count.checked_sub(index))
                .ok_or(DomError::RepresentationExhausted)?
        };
        let target = if top {
            self.source_operation.as_mut().expect("qualified top")
        } else {
            self.source_parents
                .get_mut(parent.ok_or(DomError::InvalidSourceEvent)?)
                .expect("qualified retained parent")
        };
        let old = target.observation().expect("qualified observation").limits;
        if limits.phases
            < target
                .phases
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?
        {
            return Err(DomError::SourcePublicationLimit);
        }
        if limits.outputs < target.outputs {
            return Err(DomError::SourceOutputLimit);
        }
        if limits.identities < target.identities {
            return Err(DomError::SourceIdentityLimit);
        }
        if limits.nested_operations < depth {
            return Err(DomError::SourceNestedOperationLimit {
                limit: limits.nested_operations,
            });
        }
        if limits.retained_bytes < old.retained_bytes
            || limits.retained_units < old.retained_units
            || limits.work < old.work
            || limits.phases < old.phases
            || limits.outputs < old.outputs
            || limits.identities < old.identities
            || limits.nested_operations < old.nested_operations
        {
            return Err(DomError::SourceLimitReduction);
        }
        target.admit_observation_limits(limits, cancel)?;
        policy.commit();
        Ok(())
    }
    pub fn source_event_step(
        &self,
        request: &DomSourceEventRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if request.subject().owner() != self.owner || request.work.stage.load(Ordering::Relaxed) > 3
        {
            return Err(DomError::InvalidSourceEvent);
        }
        // No public constructor can forge this Work. Issuance retains it in
        // exactly one original Operation; every Operation Drop expires stage4.
        // Private moves only transfer it within this same owner. Thus matching
        // owner + nonexpired stage qualifies the retained issuer in O(1), even
        // below a child. This lends no dispatch/current-tree authority.
        request.work.step(cancel)
    }
    pub fn retain_source_event_payload(
        &self,
        request: &DomSourceEventRequest,
        bytes: usize,
        units: usize,
        cancel: &AtomicBool,
    ) -> Result<DomSourceEventRetention, DomError> {
        let operation = self
            .source_operation
            .as_ref()
            .ok_or(DomError::InvalidSourceEvent)?;
        let observation = operation
            .observation()
            .ok_or(DomError::InvalidSourceEvent)?;
        if !Arc::ptr_eq(&observation.request.work, &request.work)
            || request.work.stage.load(Ordering::Relaxed) > 3
        {
            return Err(DomError::InvalidSourceEvent);
        }
        let external = self.source_observation_external(cancel)?;
        request
            .work
            .bytes
            .load(Ordering::Relaxed)
            .checked_add(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        request
            .work
            .units
            .load(Ordering::Relaxed)
            .checked_add(units)
            .ok_or(DomError::RepresentationExhausted)?;
        let retained = formation::SourceRetention::retain(
            operation
                .event_custody
                .as_ref()
                .ok_or(DomError::InvalidSourceEvent)?
                .clone(),
            custody::Usage::new(bytes, units),
            |next| {
                let total =
                    next.checked_add(custody::Usage::new(external.bytes, external.units))?;
                self.admit_source_observers(
                    storage::Payload {
                        bytes: total.bytes(),
                        units: total.units(),
                    },
                    cancel,
                )
            },
        )?;
        request.work.bytes.fetch_add(bytes, Ordering::Relaxed);
        request.work.units.fetch_add(units, Ordering::Relaxed);
        Ok(DomSourceEventRetention {
            work: request.work.clone(),
            custody: Some(retained),
            bytes,
            units,
        })
    }
    pub(in crate::store) fn source_rebase_limits(
        &self,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLimits, DomError> {
        self.source_rebase_transferred_limits(limits, storage::Payload::default(), cancel)
    }
    // A method rebase retains the SAME command input; unlike callback edits it
    // forms no second string occurrence. New Draft admission includes that one
    // input while the old observation still owns its remaining header/control.
    fn source_rebase_transferred_limits(
        &self,
        mut limits: DomSourceLimits,
        transferred: storage::Payload,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLimits, DomError> {
        let mut retained = self.source_observation_payload(cancel)?;
        retained.bytes = retained
            .bytes
            .checked_sub(transferred.bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        retained.units = retained
            .units
            .checked_sub(transferred.units)
            .ok_or(DomError::RepresentationExhausted)?;
        for operation in self.source_observers() {
            let observation = operation.observation().expect("observer");
            observation.request.work.step_own(cancel)?;
            limits.retained_bytes = limits.retained_bytes.min(
                observation
                    .limits
                    .retained_bytes
                    .checked_sub(retained.bytes)
                    .ok_or(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: observation.limits.retained_bytes,
                    })?,
            );
            limits.retained_units = limits.retained_units.min(
                observation
                    .limits
                    .retained_units
                    .checked_sub(retained.units)
                    .ok_or(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedUnits,
                        limit: observation.limits.retained_units,
                    })?,
            );
        }
        Ok(limits)
    }
    pub(in crate::store) fn source_child_limits(
        &self,
        mut limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLimits, DomError> {
        let observers = self.source_observation_payload(cancel)?;
        if limits.nested_operations == 0 {
            return Err(DomError::SourceNestedOperationLimit { limit: 0 });
        }
        for (index, operation) in self.source_observers().enumerate() {
            let observation = operation.observation().expect("observer");
            observation.request.work.step_own(cancel)?;
            let depth = self.source_parents.len() + 2 - index;
            if depth > observation.limits.nested_operations {
                return Err(DomError::SourceNestedOperationLimit {
                    limit: observation.limits.nested_operations,
                });
            }
            limits.retained_bytes = limits.retained_bytes.min(
                observation
                    .limits
                    .retained_bytes
                    .checked_sub(observers.bytes)
                    .ok_or(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: observation.limits.retained_bytes,
                    })?,
            );
            limits.retained_units = limits.retained_units.min(
                observation
                    .limits
                    .retained_units
                    .checked_sub(observers.units)
                    .ok_or(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedUnits,
                        limit: observation.limits.retained_units,
                    })?,
            );
        }
        Ok(limits)
    }
    pub(in crate::store) fn push_source_observer(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let operation = self
            .source_operation
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        if !operation.permits_reentry() {
            return Err(DomError::SourceOperationPending);
        }
        if self.source_parents.len() == self.source_parents.capacity() {
            let requested = self
                .source_parents
                .len()
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            let header = size_of::<Vec<Box<Operation>>>();
            let requested_bytes = requested
                .checked_mul(size_of::<Box<Operation>>())
                .and_then(|bytes| bytes.checked_add(header))
                .ok_or(DomError::RepresentationExhausted)?;
            self.admit_source_observers(
                self.source_observation_total(
                    storage::Payload {
                        bytes: requested_bytes,
                        units: 0,
                    },
                    cancel,
                )?,
                cancel,
            )?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(requested)
                .map_err(|_| DomError::AllocationFailed)?;
            let actual = replacement
                .capacity()
                .checked_mul(size_of::<Box<Operation>>())
                .and_then(|bytes| bytes.checked_add(header))
                .ok_or(DomError::RepresentationExhausted)?;
            #[cfg(test)]
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: actual - header,
            });
            self.admit_source_observers(
                self.source_observation_total(
                    storage::Payload {
                        bytes: actual,
                        units: 0,
                    },
                    cancel,
                )?,
                cancel,
            )?;
            // Admit the finite actual moves before taking any original parent.
            for _ in 0..self.source_parents.len() {
                self.step_source_observers(cancel)?;
            }
            #[cfg(test)]
            let moved = self.source_parents.len() * size_of::<Box<Operation>>();
            replacement.append(&mut self.source_parents);
            #[cfg(test)]
            if moved != 0 {
                scale_metrics::record(scale_metrics::Event::SequenceMaterialized { bytes: moved });
            }
            let old = std::mem::replace(&mut self.source_parents, replacement);
            drop(old);
        }
        self.source_parents
            .push(self.source_operation.take().expect("qualified observer"));
        Ok(())
    }
    pub(in crate::store) fn source_observed_batch(
        &self,
        owner: Arc<BatchIdentity>,
        commands: &[Command],
        capacity: usize,
        cancel: &AtomicBool,
    ) -> Result<Draft, DomError> {
        let observation = self
            .source_operation
            .as_ref()
            .and_then(|operation| operation.observation())
            .ok_or(DomError::InvalidSourceEvent)?;
        let limits = self.source_rebase_limits(observation.limits, cancel)?;
        let (draft, _) = Draft::new_source_root_with_work(
            self,
            owner,
            super::super::initial::CommandStorage { commands, capacity },
            limits,
            super::super::initial::Replay {
                work: observation.usage.work.clone(),
                identity: self
                    .source_operation
                    .as_ref()
                    .and_then(|operation| operation.event_custody.as_ref())
                    .ok_or(DomError::InvalidSourceEvent)?
                    .source()
                    .identity(),
                // This callback batch is a child, not a resumed segment.
                names: None,
            },
            cancel,
        )?;
        Ok(draft)
    }
    pub(in crate::store) fn source_reserved_closing(
        &self,
        cancel: &AtomicBool,
    ) -> Result<u64, DomError> {
        check_cancel(cancel)?;
        let mut remaining = 0u64;
        // These are the actual retained original operations, not a count of
        // observers. Rebase and parent moves preserve each operation's credit.
        for operation in self.source_observers() {
            operation
                .observation()
                .expect("observer")
                .request
                .work
                .step_own(cancel)?;
            let reserved = operation
                .headroom
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?;
            remaining = remaining
                .checked_add(reserved.remaining_phases)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        // Cached actual parked invocation credit; this is reservation
        // accounting, not execution or a scan of unrelated invocation Work.
        check_cancel(cancel)?;
        remaining = remaining
            .checked_add(self.fullscreen_exits.closing_credit())
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(remaining)
    }
}
impl DomSourceLease<'_> {
    fn finish_dialog_noop(
        &mut self,
        canceled_operation: bool,
        dialog_toggle_disposition: Option<crate::DomDialogToggleTaskDisposition>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, DomError> {
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let observation = operation
            .observation()
            .ok_or(DomError::InvalidSourceEvent)?;
        let retained = self.dom.retain_source_event_payload(
            &observation.request,
            size_of::<DomSourceCompletion>(),
            0,
            cancel,
        )?;
        let stamp = DomStamp {
            owner: self.dom.owner,
            revision: self.dom.revision,
        };
        let commit = DomCommit {
            created_ranges: Vec::new(),
            owner: observation.owner.clone(),
            created: Vec::new(),
            documents: Vec::new(),
            transition_ids: Vec::new(),
            record_ids: Vec::new(),
            changes: DomChanges {
                from: stamp,
                to: stamp,
                changes: Arc::from([]),
            },
        };
        let result = DomSourceProgress::Complete(DomSourceCompletion {
            fullscreen_exit_return: None,
            operation: operation.id,
            commit,
            cancelled: canceled_operation || operation.cancelled,
            published: Vec::new(),
            installation: None,
            dialog_toggle_disposition,
            design_mode_range_disposition: None,
        });
        observation.request.work.stage.store(4, Ordering::Relaxed);
        self.complete = true;
        self.dom.source_operation = self.dom.source_parents.pop();
        drop(retained);
        Ok(result)
    }
    pub fn accept_source_event(
        &mut self,
        request: &DomSourceEventRequest,
        publication: &mut Option<DomSourceEventPublication>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let observation = operation
            .observation()
            .ok_or(DomError::InvalidSourceEvent)?;
        let candidate = publication.as_ref().ok_or(DomError::InvalidSourceEvent)?;
        if !Arc::ptr_eq(&observation.request.work, &request.work)
            || !Arc::ptr_eq(&request.work, &candidate.work)
            || request.work.stage.load(Ordering::Relaxed) != 2
            || observation.accepted.is_some()
        {
            return Err(DomError::InvalidSourceEvent);
        }
        self.dom.admit_source_observers(
            self.dom
                .source_observation_total(storage::Payload::default(), cancel)?,
            cancel,
        )?;
        let candidate = publication.take().expect("qualified publication");
        let observation = self
            .dom
            .source_operation
            .as_mut()
            .expect("operation")
            .observation_mut()
            .expect("observer");
        observation.accepted = Some(candidate.outcome);
        candidate.work.stage.store(3, Ordering::Relaxed);
        Ok(())
    }
    fn advance_dialog_task_observation(
        &mut self,
        execution: &mut Execution<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let observation = operation
            .observation()
            .ok_or(DomError::InvalidSourceEvent)?;
        if observation.request.kind() == DomSourceEventKind::Close {
            return self
                .finish_dialog_noop(false, None, cancel)
                .map_err(Into::into);
        }
        let subject = observation.request.subject();
        self.dom.source_event_step(&observation.request, cancel)?;
        if matches!(
            self.dom
                .dispatch_custody_association(&observation.receiver, 0, subject)?,
            crate::DomDispatchAssociation::Retired(_)
        ) {
            return self
                .finish_dialog_noop(
                    false,
                    Some(crate::DomDialogToggleTaskDisposition::TargetRetiredDuringDispatch),
                    cancel,
                )
                .map_err(Into::into);
        }
        self.rebase_dialog_observation(None, execution, cancel)
    }
    pub(super) fn advance_dialog_observation(
        &mut self,
        execution: &mut Execution<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let observation = operation
            .observation()
            .ok_or(DomError::InvalidSourceEvent)?;
        self.dom.source_event_step(&observation.request, cancel)?;
        if operation.request_close.is_none()
            && operation.cancelled
            && observation.request.work.stage.load(Ordering::Relaxed) == 0
        {
            return self
                .finish_dialog_noop(true, None, cancel)
                .map_err(Into::into);
        }
        if observation.accepted.is_none() {
            return Ok(DomSourceProgress::Request(DomSourceRequest::Event(
                observation.request.clone(),
            )));
        }
        if let Some((element, identity)) = self.dom.inline_feedback_observation()
            && identity == observation.request.identity()
        {
            return Err(DomError::InlineStyleOperationPending { element }.into());
        }
        // Feed the authenticated event outcome to the original language owner.
        // Replay only this pure local transition on admission retry; the actual
        // dispatch and its consumed receipt are never replayed.
        let outcome = observation.accepted.expect("accepted event");
        let mut frame = observation
            .frame
            .as_ref()
            .map(|frame| frame.before_focusing())
            .transpose()?;
        if frame.is_none() {
            return self.advance_dialog_task_observation(execution, cancel);
        }
        let mut frame = frame.take().expect("original method frame");
        self.dom.source_event_step(&observation.request, cancel)?;
        if observation.request.kind() == DomSourceEventKind::Cancel {
            let mut call = operation
                .request_close
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?
                .retry_copy()?;
            call.cursor
                .reply(
                    surgeist_html::HtmlCloseWatcherRequestReply::CancelActionResult(
                        outcome != DomSourceEventOutcome::Canceled,
                    ),
                )
                .map_err(DomError::CloseWatcherProtocol)?;
            call.occurrence = None;
            // Commit this pure transition only when rebase admission succeeds.
            return self.rebase_request_close_cancel(frame, call, execution, cancel);
        }
        frame
            .cursor
            .reply(if observation.request.cancelable() {
                HtmlDialogReply::EventCanceled(outcome == DomSourceEventOutcome::Canceled)
            } else {
                HtmlDialogReply::Applied
            })
            .map_err(DomError::DialogProtocol)?;
        self.dom.source_event_step(&observation.request, cancel)?;
        let action = frame.cursor.action();
        match action {
            HtmlDialogAction::Complete(HtmlDialogOutcome::Completed) => {
                return self
                    .finish_dialog_noop(false, None, cancel)
                    .map_err(Into::into);
            }
            HtmlDialogAction::Complete(HtmlDialogOutcome::InvalidState) => {
                return Err(DomError::DialogInvalidState.into());
            }
            HtmlDialogAction::ReadOpen => {}
            action => return Err(DomError::DialogProcessingUnavailable { action }.into()),
        }
        let subject = observation.request.subject();
        match self
            .dom
            .dispatch_custody_association(&observation.receiver, 0, subject)?
        {
            crate::DomDispatchAssociation::Current(_) => {}
            crate::DomDispatchAssociation::Retired(_)
                if operation.request_close.is_some() || operation.is_show_modal_method() =>
            {
                let cause = crate::DomSourceCleanupCause::ReceiverRetired {
                    subject,
                    action: crate::DomSourceStoppedAction::Dialog(action),
                };
                return self.rebase_dialog_failure(cause, execution, cancel);
            }
            crate::DomDispatchAssociation::Retired(_) => {
                let id = operation.id;
                let operation = self
                    .dom
                    .source_operation
                    .take()
                    .expect("original observation");
                operation
                    .observation()
                    .expect("observer")
                    .request
                    .work
                    .stage
                    .store(4, Ordering::Relaxed);
                self.complete = true;
                self.dom.source_operation = self.dom.source_parents.pop();
                return Err(DomError::SourceReceiverRetired {
                    operation: id,
                    subject,
                    action,
                }
                .into());
            }
        }
        self.rebase_dialog_observation(Some(frame), execution, cancel)
    }
    fn rebase_request_close_cancel(
        &mut self,
        frame: crate::store::dialog::MethodFrame,
        call: super::request_close::Call,
        execution: &mut Execution<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        let old = self
            .dom
            .source_operation
            .as_mut()
            .expect("original operation")
            .request_close
            .take();
        self.dom
            .source_operation
            .as_mut()
            .expect("original operation")
            .request_close = Some(call);
        let result = self.rebase_dialog_observation(Some(frame), execution, cancel);
        if let Some(operation) = self.dom.source_operation.as_mut()
            && operation.observation().is_some()
        {
            operation.request_close = old;
        }
        result
    }
    fn rebase_dialog_failure(
        &mut self,
        cause: crate::DomSourceCleanupCause,
        execution: &mut Execution<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        // Failure remains outside Draft rollback. It never supplies a normal
        // nested return or an Applied acknowledgement to RunCloseAction.
        let operation = self
            .dom
            .source_operation
            .as_mut()
            .expect("original operation");
        operation.remember_failure(cause);
        if let Some(call) = &mut operation.request_close {
            call.nested = false;
            call.occurrence = None;
        }
        self.rebase_dialog_observation(None, execution, cancel)
    }
    fn rebase_dialog_observation(
        &mut self,
        frame: Option<crate::store::dialog::MethodFrame>,
        execution: &mut Execution<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        let operation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let observation = operation
            .observation()
            .ok_or(DomError::InvalidSourceEvent)?;
        let subject = observation.request.subject();
        let owner = observation.owner.clone();
        let work = observation.usage.work.clone();
        let limits = self.dom.source_rebase_transferred_limits(
            observation.limits,
            observation.inputs,
            cancel,
        )?;
        let (mut draft, command) = Draft::new_source_root_with_work(
            self.dom,
            owner,
            super::super::initial::CommandStorage {
                commands: &operation.commands,
                capacity: 0,
            },
            limits,
            super::super::initial::Replay {
                work,
                identity: operation
                    .event_custody
                    .as_ref()
                    .ok_or(DomError::InvalidSourceEvent)?
                    .source()
                    .identity(),
                names: Some(operation.names.clone()),
            },
            cancel,
        )?;
        if operation.request_close.is_some() {
            draft.source_invocation = observation.source_invocation;
        }
        let key = match draft.resolve(subject.into()) {
            Ok(key) => Some(key),
            Err(DomError::StaleNode)
                if operation.request_close.is_some() || operation.failure.is_some() =>
            {
                None
            }
            Err(error) => return Err(error.into()),
        };
        if let Some(mut frame) = frame {
            if let Some(key) = key {
                frame.subject = key;
                draft
                    .source_set_method(Some(crate::store::method::Frame::Dialog(frame)), cancel)?;
            }
        } else if matches!(operation.commands, OperationRoot::DialogTask(_)) {
            draft.clear_dialog_toggle_tracker(key.ok_or(DomError::StaleNode)?, cancel)?;
        }
        let custody_bytes = self.dom.source_owner_custody_bytes()?;
        let custody_units = self.dom.fullscreen_custody_units()?;
        let mut operation = self.dom.source_operation.take().expect("retained observer");
        operation.prepare_rebased_inline(self.dom, &mut draft);
        let old = std::mem::replace(&mut operation.draft, OperationDraft::Segment(draft));
        let OperationDraft::Observed(observation) = old else {
            unreachable!("qualified observer")
        };
        let draft = &mut operation.draft;
        draft.base_work.set(observation.usage.base);
        draft.encoding_url_work.set(observation.usage.encoding);
        draft.meta_work.set(observation.usage.meta);
        draft.select_work.set(observation.usage.select);
        draft
            .select_semantic_work
            .set(observation.usage.semantic_work);
        draft
            .select_semantic_bytes
            .set(observation.usage.semantic_bytes);
        draft.input_processing = observation.usage.input;
        operation.command_storage = command;
        operation.custody_bytes = custody_bytes;
        operation.custody_units = custody_units;
        operation.limits = observation.limits;
        drop(observation.receiver);
        if operation.request_close.is_some() {
            observation.request.work.stage.store(4, Ordering::Relaxed);
        }
        drop(observation.request);
        let refreshed = self.dom.source_rebase_limits(operation.limits, cancel);
        match refreshed {
            Ok(limits) => operation.draft.source_limits = Some(limits),
            Err(error) => {
                self.dom.source_operation = Some(operation);
                return Err(error.into());
            }
        }
        let refreshed_aux = self.dom.source_retained_stack_bytes().and_then(|stack| {
            self.dom
                .source_dispatch_custody_bytes(|| operation.draft.source_step(cancel))
                .and_then(|bank| {
                    stack
                        .checked_add(bank)
                        .ok_or(DomError::RepresentationExhausted)
                })
        });
        match refreshed_aux {
            Ok(bytes) => operation.canonical_aux_bytes = bytes,
            Err(error) => {
                self.dom.source_operation = Some(operation);
                return Err(error.into());
            }
        }
        operation.started = true;
        let result = self.advance_operation(&mut operation, execution, cancel);
        if self.complete || matches!(result, Ok(DomSourceProgress::Complete(_))) {
            self.complete = true;
            self.dom.source_operation = self.dom.source_parents.pop();
        } else {
            self.dom.source_operation = Some(operation);
        }
        result
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod admission_tests;

#[cfg(test)]
mod element_task_tests;

#[cfg(test)]
mod watcher_payload_tests;

#[cfg(test)]
mod close_tests;

#[cfg(test)]
mod dispatch_hosted_read_tests;

#[cfg(test)]
mod task_disposition_tests;

#[cfg(test)]
mod headroom_tests;

#[cfg(test)]
mod custody_tests;
