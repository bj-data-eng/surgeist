//! Initial Show event observation without an old canonical Draft.
use super::*;
use crate::internal::source::callback::{CallbackWork, DispatchScope};
use crate::internal::source::event::ExecutorBinding;
use crate::internal::source::*;
use std::sync::Weak;
use surgeist_html::elements::dialog::{HtmlDialogAction, HtmlDialogOutcome, HtmlDialogReply};

pub(super) struct ResumeWorkPolicy {
    work: Arc<CallbackWork>,
    old: u64,
    committed: bool,
}
impl ResumeWorkPolicy {
    pub(super) fn new(work: Arc<CallbackWork>, proposed: u64) -> Result<Self, DomError> {
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
    Observed(EventObservation),
    AttributeObserved(CallbackObservation),
    CustomElementObserved(CallbackObservation),
}
// Existing Style/child/installation/Select execution remains the original
// segmented engine. Its entry points select the Segment phase before use.
impl std::ops::Deref for OperationDraft {
    type Target = Draft;
    fn deref(&self) -> &Draft {
        match self {
            Self::Segment(draft) => draft,
            Self::Observed(_) | Self::AttributeObserved(_) | Self::CustomElementObserved(_) => {
                unreachable!("observation has no Draft")
            }
        }
    }
}
impl std::ops::DerefMut for OperationDraft {
    fn deref_mut(&mut self) -> &mut Draft {
        match self {
            Self::Segment(draft) => draft,
            Self::Observed(_) | Self::AttributeObserved(_) | Self::CustomElementObserved(_) => {
                unreachable!("observation has no Draft")
            }
        }
    }
}
pub(super) struct Usage {
    autofocus: usize,
    base: u64,
    encoding: u64,
    meta: u64,
    select: u64,
    semantic_work: usize,
    semantic_bytes: usize,
    input: input_processing::InputProcessing,
}
impl Usage {
    pub(super) fn capture(draft: &mut Draft, autofocus: usize) -> Self {
        Self {
            autofocus,
            base: draft.base_work.get(),
            encoding: draft.encoding_url_work.get(),
            meta: draft.meta_work.get(),
            select: draft.select_work.get(),
            semantic_work: draft.select_semantic_work.get(),
            semantic_bytes: draft.select_semantic_bytes.get(),
            input: std::mem::replace(
                &mut draft.input_processing,
                input_processing::InputProcessing::new(0, 0),
            ),
        }
    }

    pub(super) fn restore(self, draft: &mut Draft) {
        draft.base_work.set(self.base);
        draft.encoding_url_work.set(self.encoding);
        draft.meta_work.set(self.meta);
        draft.select_work.set(self.select);
        draft.select_semantic_work.set(self.semantic_work);
        draft.select_semantic_bytes.set(self.semantic_bytes);
        draft.input_processing = self.input;
    }

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
            return Err(DomError::Meta(
                crate::internal::meta::DomMetaError::WorkLimit {
                    limit: limits.meta_work,
                },
            ));
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
struct EventFiring {
    subject: NodeId,
    payload: crate::internal::source::event::DomSourceEventPayload,
    cancelable: bool,
    kind: DomSourceEventKind,
    frame: Option<crate::internal::store::dialog::MethodFrame>,
}
#[derive(Clone, Copy)]
enum FiringTime {
    Unavailable,
    Pending {
        global: crate::internal::hosted::DomHostedGlobalId,
        document: crate::internal::identity::DocumentId,
        origin: crate::internal::hosted::DomHostedMoment,
    },
    Observed(Result<DomSourceEventTime, DomSourceEventClockFailure>),
}

pub(super) struct CallbackObservation {
    pub(super) owner: Arc<BatchIdentity>,
    pub(super) work: Arc<CallbackWork>,
    pub(super) usage: Usage,
    pub(super) limits: DomSourceLimits,
    pub(super) inputs: storage::Payload,
}

/// Fallible callback admission completed before changing the operation phase.
/// No journal or canonical write is consumed by preparing this receipt.
pub(super) struct PreparedCallback {
    autofocus: usize,
    work: Arc<CallbackWork>,
    custody: Arc<formation::CallbackWorkCustody>,
}

impl PreparedCallback {
    pub(super) fn open_scope(&self) -> Result<Arc<DispatchScope>, DomError> {
        let scope = self
            .work
            .open_scope()
            .ok_or(DomError::InvalidSourceOperation)?;
        self.work.stage.store(1, Ordering::Relaxed);
        Ok(scope)
    }
}

impl CallbackObservation {
    fn payload(&self) -> Result<storage::Payload, DomError> {
        Ok(storage::Payload {
            // The Operation allocation and command storage remain owner-side.
            // The callback work and temporary buffers are in the ledger.
            bytes: size_of::<Operation>()
                .checked_add(self.inputs.bytes)
                .ok_or(DomError::RepresentationExhausted)?,
            units: self.inputs.units,
        })
    }
}

pub(super) struct EventObservation {
    request: DomSourceEventRequest,
    frame: Option<crate::internal::store::dialog::MethodFrame>,
    source_invocation: u64,
    pub(super) common: CallbackObservation,
    accepted: Option<DomSourceEventOutcome>,
    receiver: crate::internal::dispatch::DomDispatchCustody,
    time: FiringTime,
}
impl EventObservation {
    pub(super) fn status(&self, operation: &Operation) -> DomSourceStatus {
        DomSourceStatus {
            operation: operation.id,
            published: operation.published,
            cancelled: operation.cancelled,
            cleanup: operation.failure,
            work: self.common.work.work.load(Ordering::Relaxed),
            phases: operation.phases,
            parent: self.request.parent_operation(),
            phase: if operation.failure.is_some() {
                DomSourcePhase::PendingCleanup
            } else if matches!(self.time, FiringTime::Pending { .. }) {
                DomSourcePhase::EventFiring
            } else if self.accepted.is_some() {
                DomSourcePhase::EventReceiptAccepted
            } else {
                DomSourcePhase::EventObservation
            },
        }
    }
}
impl Drop for Operation {
    fn drop(&mut self) {
        if let Some(work) = self.event_work.upgrade() {
            work.stage.store(4, Ordering::Relaxed);
        }
        if let OperationDraft::AttributeObserved(observation)
        | OperationDraft::CustomElementObserved(observation) = &self.draft
        {
            observation.work.stage.store(4, Ordering::Relaxed);
        }
    }
}
impl Operation {
    pub(super) fn prepare_callback_observation(
        &self,
        cancel: &AtomicBool,
    ) -> Result<PreparedCallback, DomError> {
        let custody = formation::CallbackWorkCustody::prepare(
            self.draft.custody_ledger(),
            self.draft.source_work_custody(),
            self.draft.source_ancestor_custody(),
            |usage| self.draft.admit_custody_usage(usage, cancel),
        )?;
        let work = formation::allocated_arc(CallbackWork::new(
            self.draft.source_work.source_arc().clone(),
            self.limits.work,
            self.draft.source_ancestors.clone(),
            formation::SourceStorage::new(self.draft.custody_ledger().clone()),
        ));
        let autofocus = self.draft.autofocus_count(cancel)?;
        Ok(PreparedCallback {
            autofocus,
            custody: custody.finish(work.clone()),
            work,
        })
    }

    pub(super) fn begin_callback_observation(
        &mut self,
        prepared: PreparedCallback,
    ) -> CallbackObservation {
        self.event_custody = Some(prepared.custody);
        self.custody_bytes = 0;
        self.custody_units = 0;
        CallbackObservation {
            owner: self.draft.owner.clone(),
            work: prepared.work,
            usage: Usage::capture(&mut self.draft, prepared.autofocus),
            limits: self.limits,
            inputs: std::mem::take(&mut self.command_storage),
        }
    }

    pub(in crate::internal::store) fn dispatch_limit_formation(
        &self,
        cancel: &AtomicBool,
    ) -> Result<Option<formation::Formation>, DomError> {
        match &self.draft {
            OperationDraft::Segment(draft) => formation::Formation::new(draft, cancel),
            OperationDraft::Observed(_)
            | OperationDraft::AttributeObserved(_)
            | OperationDraft::CustomElementObserved(_) => Ok(None),
        }
    }
    pub(super) fn callback_observation(&self) -> Option<&CallbackObservation> {
        match &self.draft {
            OperationDraft::Segment(_) => None,
            OperationDraft::Observed(observation) => Some(&observation.common),
            OperationDraft::AttributeObserved(observation)
            | OperationDraft::CustomElementObserved(observation) => Some(observation),
        }
    }

    fn callback_observation_mut(&mut self) -> Option<&mut CallbackObservation> {
        match &mut self.draft {
            OperationDraft::Segment(_) => None,
            OperationDraft::Observed(observation) => Some(&mut observation.common),
            OperationDraft::AttributeObserved(observation)
            | OperationDraft::CustomElementObserved(observation) => Some(observation),
        }
    }

    pub(super) fn observation(&self) -> Option<&EventObservation> {
        match &self.draft {
            OperationDraft::Observed(observation) => Some(observation),
            OperationDraft::Segment(_)
            | OperationDraft::AttributeObserved(_)
            | OperationDraft::CustomElementObserved(_) => None,
        }
    }
    fn observation_mut(&mut self) -> Option<&mut EventObservation> {
        match &mut self.draft {
            OperationDraft::Observed(observation) => Some(observation),
            OperationDraft::Segment(_)
            | OperationDraft::AttributeObserved(_)
            | OperationDraft::CustomElementObserved(_) => None,
        }
    }
    pub(in crate::internal::store) fn id(&self) -> DomSourceOperationId {
        self.id
    }
    pub(in crate::internal::store) fn inline_feedback_observation(
        &self,
    ) -> Option<crate::internal::source::DomSourceEventId> {
        self.observation()
            .map(|observation| observation.request.identity())
    }
    pub(in crate::internal::store) fn is_observing_callback(&self) -> bool {
        self.callback_observation().is_some()
    }
    pub(in crate::internal::store) fn has_event_tail(&self) -> bool {
        self.event_exposed && self.observation().is_none()
    }
    pub(in crate::internal::store) fn permits_reentry(&self) -> bool {
        self.callback_observation()
            .is_some_and(|observation| observation.work.active())
            && match self.draft {
                OperationDraft::AttributeObserved(_) => self
                    .attribute_verification
                    .as_ref()
                    .is_some_and(super::attribute::Pending::registration_is_active),
                OperationDraft::CustomElementObserved(_) => self
                    .custom_element
                    .as_ref()
                    .is_some_and(super::custom_element::Pending::registration_is_active),
                _ => true,
            }
    }
    pub(super) fn admit_observation_limits(
        &mut self,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let observation = self
            .callback_observation_mut()
            .ok_or(DomError::InvalidSourceOperation)?;
        check_cancel(cancel)?;
        if limits.work < observation.work.work.load(Ordering::Relaxed)
            || limits.nested_operations == 0
        {
            return Err(DomError::InvalidLimit(DomResource::SourceRetainedBytes));
        }
        // Actual current whole-owner storage is qualified by the Dom front before
        // this policy is changed; no accepted event is consumed on refusal.
        observation
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
        hosted_events: bool,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        let HtmlDialogAction::FireBeforeToggle {
            old,
            new,
            cancelable,
            source,
        } = action
        else {
            return Err(DomError::InvalidSourceEvent.into());
        };
        if !matches!(&self.commands, OperationRoot::Edit([Command::ShowDialog { node: NodeOperand::Existing(actual) } | Command::ShowModalDialog { node: NodeOperand::Existing(actual) } | Command::CloseDialog { node: NodeOperand::Existing(actual), .. } | Command::RequestCloseDialog { node: NodeOperand::Existing(actual), .. }]) if *actual == subject)
            || !matches!(
                source,
                surgeist_html::elements::dialog::HtmlDialogEventSource::Null
                    | surgeist_html::elements::dialog::HtmlDialogEventSource::OperationSource
            )
            || !self.draft.created.is_empty()
            || !self.draft.tickets.is_empty()
            || !self.draft.effects.is_empty()
        {
            return Err(DomError::InvalidSourceEvent.into());
        }
        let Some(crate::internal::store::method::Frame::Dialog(frame)) = self.draft.method.as_ref()
        else {
            return Err(DomError::InvalidSourceEvent.into());
        };
        let frame = frame.before_focusing()?;
        self.observe_source_event(
            dom,
            EventFiring {
                subject,
                payload: crate::internal::source::event::DomSourceEventPayload::Toggle(
                    crate::internal::source::DomSourceToggleEvent { old, new },
                ),
                cancelable,
                kind: DomSourceEventKind::BeforeToggle,
                frame: Some(frame),
            },
            hosted_events,
            cancel,
        )
    }
    pub(super) fn observe_dialog_task(
        &mut self,
        dom: &mut Dom,
        task: crate::internal::store::source::operation::dialog_queue::DomDialogTaskInfo,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        self.observe_source_event(
            dom,
            EventFiring {
                subject: task.id().target(),
                payload: match task {
                    crate::internal::store::source::operation::dialog_queue::DomDialogTaskInfo::Toggle(info) => {
                        crate::internal::source::event::DomSourceEventPayload::Toggle(
                            crate::internal::source::DomSourceToggleEvent {
                                old: info.old_state(),
                                new: info.new_state(),
                            },
                        )
                    }
                    crate::internal::store::source::operation::dialog_queue::DomDialogTaskInfo::Close(_) => {
                        crate::internal::source::event::DomSourceEventPayload::Plain
                    }
                },
                cancelable: false,
                kind: match task {
                    crate::internal::store::source::operation::dialog_queue::DomDialogTaskInfo::Toggle(_) => DomSourceEventKind::Toggle,
                    crate::internal::store::source::operation::dialog_queue::DomDialogTaskInfo::Close(_) => DomSourceEventKind::Close,
                },
                frame: None,
            },
            false,
            cancel,
        )
    }
    pub(super) fn observe_request_close_cancel(
        &mut self,
        dom: &mut Dom,
        cancelable: bool,
        hosted_events: bool,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        let call = self
            .request_close
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        self.observe_source_event(
            dom,
            EventFiring {
                subject: call.watcher.subject(),
                payload: crate::internal::source::event::DomSourceEventPayload::Plain,
                cancelable,
                kind: DomSourceEventKind::Cancel,
                frame: Some(call.outer.before_focusing()?),
            },
            hosted_events,
            cancel,
        )
    }
    pub(super) fn observe_form_reset(
        &mut self,
        dom: &mut Dom,
        form: Key,
        hosted_events: bool,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        let subject = self
            .draft
            .published_id(form)
            .ok_or(DomError::InvalidSourceOperation)?;
        self.observe_source_event(
            dom,
            EventFiring {
                subject,
                payload: crate::internal::source::event::DomSourceEventPayload::Plain,
                cancelable: true,
                kind: DomSourceEventKind::Reset,
                frame: None,
            },
            hosted_events,
            cancel,
        )
    }

    fn observe_source_event(
        &mut self,
        dom: &mut Dom,
        firing: EventFiring,
        hosted_events: bool,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, AdvanceError> {
        let EventFiring {
            subject,
            payload,
            cancelable,
            kind,
            frame,
        } = firing;
        let form_reset = kind == DomSourceEventKind::Reset;
        let minimum = headroom::FINAL_PUBLICATION + u64::from(form_reset);
        // Protect the closing publication before exposing callbacks. A Form
        // first publishes the actual reset lock right here.
        let possible = match (kind, frame.as_ref().map(|frame| frame.locals)) {
            (
                DomSourceEventKind::BeforeToggle,
                Some(crate::internal::store::dialog::MethodLocals::Show { .. }),
            ) => headroom::dialog_show(true, self.is_show_modal_method()),
            (
                DomSourceEventKind::BeforeToggle,
                Some(crate::internal::store::dialog::MethodLocals::Close),
            ) => headroom::dialog_close(true),
            (
                DomSourceEventKind::Cancel,
                Some(crate::internal::store::dialog::MethodLocals::RequestClose { .. }),
            ) if self.request_close.is_some() => headroom::dialog_close(true),
            (DomSourceEventKind::Toggle | DomSourceEventKind::Close, None)
                if matches!(self.commands, OperationRoot::DialogTask(_)) =>
            {
                headroom::FINAL_PUBLICATION
            }
            (DomSourceEventKind::Reset, None) => minimum,
            _ => return Err(DomError::InvalidSourceEvent.into()),
        };
        let remaining = self.headroom.as_ref().map_or(possible, |reservation| {
            possible.max(reservation.remaining_phases)
        });
        // Caller phase policy can be raised on resume. Observation needs its
        // closing minimum; the actual queue/publication gates admit the reached
        // tail. Fixed canonical revision credit, unlike policy, remains whole.
        let close = self
            .phases
            .checked_add(minimum)
            .ok_or(DomError::RepresentationExhausted)?;
        if close > self.limits.phases {
            return Err(DomError::SourcePublicationLimit.into());
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
        let event_custody = formation::CallbackWorkCustody::prepare(
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
            work: formation::allocated_arc(CallbackWork::new(
                self.draft.source_work.source_arc().clone(),
                self.limits.work,
                self.draft.source_ancestors.clone(),
                formation::SourceStorage::new(self.draft.custody_ledger().clone()),
            )),
        };
        let event_custody = event_custody.finish(request.work.clone());
        let limits = self.limits;

        let receiver =
            dom.register_dispatch_custody_with_formation(&[subject], formation.as_ref(), cancel)?;
        let autofocus = self.draft.autofocus_count(cancel)?;
        let mut inputs = self.command_storage;
        if let Some(value) = &self.request_close_result {
            inputs.string(value)?;
        }
        let time = if hosted_events {
            self.firing_time(dom, subject, cancel)?
        } else {
            FiringTime::Unavailable
        };
        let prepared = matches!(time, FiringTime::Pending { .. });
        let form_publication = if form_reset {
            let previous = self.headroom.replace(headroom::Reservation {
                remaining_phases: remaining,
            });
            let segment = (|| {
                let segment = self
                    .prepare_segment(dom, true, PublicationTarget::None, None, cancel)
                    .map_err(|error| match error {
                        crate::internal::hosted::DomHostedError::Dom(error) => error,
                        _ => DomError::HostedSourceUnavailable,
                    })?;
                if segment
                    .prepared
                    .inline_recording
                    .as_ref()
                    .is_some_and(|recording| recording.has_inputs())
                {
                    return Err(DomError::IncompleteSourceOperation);
                }
                Ok(segment)
            })();
            match segment {
                Ok(segment) => Some(segment),
                Err(error) => {
                    self.headroom = previous;
                    return Err(error.into());
                }
            }
        } else {
            None
        };
        let usage = Usage::capture(&mut self.draft, autofocus);
        let observation = EventObservation {
            request: request.clone(),
            frame,
            source_invocation: if self.request_close.is_some() {
                occurrence
            } else {
                self.draft.source_invocation
            },
            common: CallbackObservation {
                owner: self.draft.owner.clone(),
                work: request.work.clone(),
                usage,
                limits,
                inputs,
            },
            accepted: None,
            receiver,
            time,
        };
        if let Some(segment) = form_publication {
            segment.publish(self, dom);
            self.command_admitted = true;
        } else {
            self.headroom = Some(headroom::Reservation {
                remaining_phases: remaining,
            });
        }
        self.event_custody = Some(event_custody);
        self.draft = OperationDraft::Observed(observation);
        if !prepared {
            self.event_exposed = true;
            self.event_work = Arc::downgrade(&request.work);
        }
        drop(formation); // Work is now owned by the admitted observation.
        self.command_storage = storage::Payload::default();
        self.custody_bytes = 0;
        self.custody_units = 0;
        if prepared {
            Err(AdvanceError::EventFiring)
        } else {
            Ok(DomSourceProgress::Request(DomSourceRequest::Event(request)))
        }
    }

    fn firing_time(
        &self,
        dom: &Dom,
        subject: NodeId,
        cancel: &AtomicBool,
    ) -> Result<FiringTime, DomError> {
        let origin = dom
            .selected_event_creation_origin(subject, &mut || {
                self.draft
                    .source_step(cancel)
                    .map_err(crate::internal::hosted::DomHostedError::from)
            })
            .map_err(|error| match error {
                crate::internal::hosted::DomHostedError::Dom(error) => error,
                _ => DomError::HostedSourceUnavailable,
            })?;
        Ok(match origin {
            Some((global, document, origin)) => FiringTime::Pending {
                global,
                document,
                origin,
            },
            None => FiringTime::Unavailable,
        })
    }
}
/// Exclusive acknowledgment of the original issuer's prepared firing.
#[must_use = "finish the firing or leave its exact observation parked"]
pub struct DomSourceEventFiringLoan<'a> {
    observation: &'a mut EventObservation,
    event_exposed: &'a mut bool,
    event_work: &'a mut Weak<CallbackWork>,
    source: &'a crate::internal::hosted::DomHostedSource,
    origin: crate::internal::hosted::DomHostedMoment,
    global: crate::internal::hosted::DomHostedGlobalId,
}

impl DomSourceEventFiringLoan<'_> {
    /// Supply the original issuer's single native clock result. No Event Job or
    /// dispatch scope exists here. Only the actual retained request is exposed.
    pub fn finish(
        self,
        occurrence: Result<u64, DomSourceEventClockFailure>,
    ) -> Result<DomSourceProgress, crate::internal::hosted::DomHostedError> {
        if occurrence.is_ok_and(|microseconds| !microseconds.is_multiple_of(100)) {
            return Err(crate::internal::hosted::DomHostedError::InvalidClockSample);
        }
        self.observation.time =
            FiringTime::Observed(occurrence.map(|microseconds| DomSourceEventTime {
                global: self.global,
                origin: self.origin,
                occurrence: crate::internal::hosted::DomHostedMoment {
                    source: self.source.id(),
                    microseconds,
                },
            }));
        let request = self.observation.request.clone();
        *self.event_exposed = true;
        *self.event_work = Arc::downgrade(&request.work);
        Ok(DomSourceProgress::Request(DomSourceRequest::Event(request)))
    }
}

impl DomSourceLease<'_> {
    pub(super) fn prepared_event_firing(&self) -> Result<DomSourceEventFiring, DomError> {
        let observation = self
            .dom
            .source_operation
            .as_ref()
            .filter(|operation| operation.id == self.id)
            .and_then(|operation| operation.observation())
            .ok_or(DomError::InvalidSourceEvent)?;
        let FiringTime::Pending {
            global,
            document,
            origin,
        } = observation.time
        else {
            return Err(DomError::InvalidSourceEvent);
        };
        Ok(DomSourceEventFiring {
            identity: observation.request.identity(),
            global,
            document,
            origin,
        })
    }

    /// Qualify this still-private firing against the exact original issuer and
    /// admit the actual clock observation against its Source and ancestors.
    pub fn begin_event_firing<'a>(
        &'a mut self,
        firing: DomSourceEventFiring,
        source: &'a crate::internal::hosted::DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<DomSourceEventFiringLoan<'a>, crate::internal::hosted::DomHostedError> {
        let actual = self.prepared_event_firing()?;
        if actual.identity != firing.identity
            || actual.global != firing.global
            || actual.document != firing.document
            || actual.origin != firing.origin
            || source.id() != actual.origin.source()
            || source.id() != actual.global.source()
        {
            return Err(DomError::InvalidSourceEvent.into());
        }
        self.dom.validate_hosted_source(source)?;
        let operation = self
            .dom
            .source_operation
            .as_mut()
            .ok_or(DomError::InvalidSourceEvent)?;
        let OperationDraft::Observed(observation) = &mut operation.draft else {
            return Err(DomError::InvalidSourceEvent.into());
        };
        observation.request.work.step(cancel)?;
        Ok(DomSourceEventFiringLoan {
            observation,
            event_exposed: &mut operation.event_exposed,
            event_work: &mut operation.event_work,
            source,
            origin: actual.origin,
            global: actual.global,
        })
    }
}

impl Dom {
    pub(in crate::internal::store) fn admit_source_owner_payload(
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
        if operation.callback_observation().is_some() {
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
    pub(in crate::internal::store) fn dispatch_hosted_read_work(
        &self,
        cancel: &AtomicBool,
    ) -> Result<Option<Arc<CallbackWork>>, DomError> {
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
            .callback_observation()
            .filter(|_| operation.permits_reentry())
            .ok_or(DomError::IncompleteSourceOperation)?;
        let work = &observation.work;
        if let Some(event) = operation.observation() {
            self.qualify_source_event(work, 1)?;
            let executor = self
                .source_event_executor
                .as_ref()
                .and_then(Weak::upgrade)
                .filter(|binding| binding.live.load(Ordering::Relaxed))
                .ok_or(DomError::InvalidSourceEvent)?;
            event.request.validate_executor(&executor)?;
        } else if matches!(operation.draft, OperationDraft::CustomElementObserved(_)) {
            operation.qualify_active_custom_element_callback(self, cancel)?;
        } else {
            operation.qualify_active_attribute_callback(self, cancel)?;
        }
        work.step(cancel)?;
        if let Some(incomplete) = self.state.incomplete_source {
            let mut qualified = operation.published && operation.id == incomplete;
            if !qualified {
                for parent in self.source_parents.iter().rev() {
                    work.step(cancel)?;
                    if parent.id == incomplete
                        && parent.published
                        && parent.callback_observation().is_some()
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
    pub(in crate::internal::store) fn qualify_read_source(
        &self,
        cancel: &AtomicBool,
    ) -> Result<Option<crate::internal::store::read_source::ReadSource<'_>>, DomError> {
        use crate::internal::store::read_source::{ReadAuthority, ReadSource};

        let Some(work) = self.dispatch_hosted_read_work(cancel)? else {
            return Ok(None);
        };
        let operation = self
            .source_operation
            .as_ref()
            .ok_or(DomError::IncompleteSourceOperation)?;
        let authority = if let Some(event) = operation.observation() {
            ReadAuthority::Event(&event.request)
        } else {
            let registration = &self
                .script_registration
                .as_ref()
                .ok_or(DomError::HostedSourceUnavailable)?
                .0;
            let node =
                self.state
                    .node_with(self.owner, registration.key.document.node(), &mut || {
                        work.step(cancel)
                    })?;
            let crate::internal::snapshot::DomNodeKind::Document(metadata) = node.kind() else {
                return Err(DomError::WrongDocument);
            };
            let association = metadata
                .document_environment()
                .hosted
                .ok_or(DomError::HostedSourceUnavailable)?;
            ReadAuthority::Script {
                registration,
                association,
                attribute: matches!(operation.draft, OperationDraft::AttributeObserved(_)),
            }
        };
        ReadSource::new(work, authority).map(Some)
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
    pub(in crate::internal::store) fn source_work_ancestors(
        &self,
        excluding: Option<&Arc<AtomicU64>>,
    ) -> Option<Arc<CallbackWork>> {
        self.source_operation
            .iter()
            .map(Box::as_ref)
            .chain(self.source_parents.iter().rev().map(Box::as_ref))
            .filter_map(|operation| operation.callback_observation())
            .find(|observation| {
                excluding.is_none_or(|work| !Arc::ptr_eq(work, &observation.work.work))
            })
            .map(|observation| observation.work.clone())
    }
    pub(in crate::internal::store) fn source_work_ancestor_custody(
        &self,
        excluding: Option<&Arc<AtomicU64>>,
    ) -> Option<Arc<formation::CallbackWorkCustody>> {
        self.source_operation
            .iter()
            .map(Box::as_ref)
            .chain(self.source_parents.iter().rev().map(Box::as_ref))
            .find(|operation| {
                operation.callback_observation().is_some_and(|observation| {
                    excluding.is_none_or(|work| !Arc::ptr_eq(work, &observation.work.work))
                })
            })
            .and_then(|operation| operation.event_custody.clone())
    }
    pub(in crate::internal::store) fn dispatch_registration_formation(
        &self,
        cancel: &AtomicBool,
    ) -> Result<Option<formation::Formation>, DomError> {
        if self.source_operation.is_some() {
            self.require_source_entry(cancel)?;
        }
        self.source_observation_formation(cancel)
    }
    pub(in crate::internal::store) fn source_observation_formation(
        &self,
        cancel: &AtomicBool,
    ) -> Result<Option<formation::Formation>, DomError> {
        let Some(operation) = self.source_operation.as_ref() else {
            check_cancel(cancel)?;
            return Ok(None);
        };
        let observation = operation
            .callback_observation()
            .ok_or(DomError::SourceOperationPending)?;
        let base = self.source_observation_external(cancel)?;
        let controlled = observation.work.storage.ledger().total();
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
            let ancestor = ancestor.callback_observation().expect("observer");
            limits.retained_bytes = limits.retained_bytes.min(ancestor.limits.retained_bytes);
            limits.retained_units = limits.retained_units.min(ancestor.limits.retained_units);
        }
        Ok(Some(formation::Formation::from_callback(
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
            .filter(|operation| operation.callback_observation().is_some())
    }
    fn source_observation_payload(
        &self,
        cancel: &AtomicBool,
    ) -> Result<storage::Payload, DomError> {
        let mut payload = storage::Payload::default();

        for operation in self.source_observers() {
            let observation = operation.callback_observation().expect("observer");
            observation.work.step_own(cancel)?;
            payload.append(observation.payload()?)?;
        }
        Ok(payload)
    }
    pub(in crate::internal::store) fn admit_source_resume(
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
        if let Some(observation) = operation.callback_observation() {
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
            if limits.work < observation.work.work.load(Ordering::Relaxed)
                || limits.outputs < operation.outputs
                || limits.identities < operation.identities
                || limits.nested_operations == 0
            {
                return Err(DomError::InvalidLimit(DomResource::SourceRetainedBytes));
            }
        }
        Ok(())
    }
    pub(in crate::internal::store) fn source_retained_stack_bytes(
        &self,
    ) -> Result<usize, DomError> {
        self.source_parents
            .capacity()
            .checked_mul(size_of::<Box<Operation>>())
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<Box<Operation>>>()))
            .ok_or(DomError::RepresentationExhausted)
    }
    pub(in crate::internal::store) fn source_dispatch_custody_bytes(
        &self,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<usize, DomError> {
        self.dispatch_custody
            .retained_bytes(step)?
            .checked_add(size_of::<crate::internal::store::dispatch::CustodyBank>())
            .ok_or(DomError::RepresentationExhausted)
    }
    pub(super) fn step_source_observers(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        for operation in self.source_observers() {
            let observation = operation.callback_observation().expect("observer");
            observation.work.step_own(cancel)?;
        }
        Ok(())
    }
    #[cfg(test)]
    pub(in crate::internal::store) fn source_measurement_ancestors(&self) -> custody::Usage {
        let mut payload = storage::Payload::default();
        for operation in self.source_observers() {
            let observation = operation
                .callback_observation()
                .expect("observed Source ancestor");
            payload
                .append(observation.payload().expect("retained ancestor receipt"))
                .expect("retained ancestor sum");
        }
        custody::Usage::new(payload.bytes, payload.units)
    }
    #[cfg(test)]
    pub(in crate::internal::store) fn observe_source_lifecycle(&self) {
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
            Some(
                OperationDraft::Observed(_)
                | OperationDraft::AttributeObserved(_)
                | OperationDraft::CustomElementObserved(_),
            ) => {
                // These are cached input receipts and fixed owner headers. The
                // only iteration is the actual Source ancestor stack; no bank,
                // canonical payload, or effect frontier is inspected or metered.
                let mut payload = storage::Payload::default();
                for operation in self.source_observers() {
                    payload
                        .append(
                            operation
                                .callback_observation()
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
    pub(super) fn source_observation_external(
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
    pub(in crate::internal::store) fn admit_collection_output_usage(
        &self,
        usage: custody::Usage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut total = self.source_observation_external(cancel)?;
        total.add_bytes(usage.bytes())?;
        total.add_units(usage.units())?;
        self.admit_source_observers(total, cancel)
    }

    pub(super) fn source_observation_total(
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
    pub(super) fn admit_source_observers(
        &self,
        total: storage::Payload,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        for operation in self.source_observers() {
            let observation = operation.callback_observation().expect("observer");
            observation.work.step_own(cancel)?;
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
        work: &Arc<CallbackWork>,
        stage: u8,
    ) -> Result<&DomSourceEventRequest, DomError> {
        let operation = self
            .source_operation
            .as_ref()
            .ok_or(DomError::InvalidSourceEvent)?;
        let observation = operation
            .observation()
            .ok_or(DomError::InvalidSourceEvent)?;
        let request = &observation.request;
        if request.subject().owner() != self.owner
            || request.operation() != operation.id
            || matches!(observation.time, FiringTime::Pending { .. })
            || !Arc::ptr_eq(&request.work, work)
            || work.stage.load(Ordering::Relaxed) != stage
        {
            return Err(DomError::InvalidSourceEvent);
        }
        Ok(request)
    }
    /// Read the original retained firing observation for the exact real request
    /// before its executor starts dispatch. Unhosted requests can lack timing.
    pub fn source_event_time(
        &self,
        request: &DomSourceEventRequest,
    ) -> Result<Option<Result<DomSourceEventTime, DomSourceEventClockFailure>>, DomError> {
        self.qualify_source_event(&request.work, 0)?;
        let observation = self
            .source_operation
            .as_ref()
            .and_then(|operation| operation.observation())
            .ok_or(DomError::InvalidSourceEvent)?;
        Ok(match observation.time {
            FiringTime::Observed(result) => Some(result),
            FiringTime::Unavailable => None,
            FiringTime::Pending { .. } => return Err(DomError::InvalidSourceEvent),
        })
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
        let old = target
            .observation()
            .expect("qualified observation")
            .common
            .limits;
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
    pub(in crate::internal::store) fn source_rebase_limits(
        &self,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLimits, DomError> {
        self.source_rebase_transferred_limits(limits, storage::Payload::default(), cancel)
    }
    // A method rebase retains the SAME command input; unlike callback edits it
    // forms no second string occurrence. New Draft admission includes that one
    // input while the old observation still owns its remaining header/control.
    pub(super) fn source_rebase_transferred_limits(
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
            let observation = operation.callback_observation().expect("observer");
            observation.work.step_own(cancel)?;
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
    pub(in crate::internal::store) fn source_child_limits(
        &self,
        mut limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLimits, DomError> {
        let observers = self.source_observation_payload(cancel)?;
        if limits.nested_operations == 0 {
            return Err(DomError::SourceNestedOperationLimit { limit: 0 });
        }
        for (index, operation) in self.source_observers().enumerate() {
            let observation = operation.callback_observation().expect("observer");
            observation.work.step_own(cancel)?;
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
    pub(in crate::internal::store) fn push_source_observer(
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
    pub(in crate::internal::store) fn source_observed_batch(
        &self,
        owner: Arc<BatchIdentity>,
        commands: &[Command],
        capacity: usize,
        cancel: &AtomicBool,
    ) -> Result<Draft, DomError> {
        let observation = self
            .source_operation
            .as_ref()
            .and_then(|operation| operation.callback_observation())
            .ok_or(DomError::InvalidSourceEvent)?;
        let limits = self.source_rebase_limits(observation.limits, cancel)?;
        let (draft, _) = Draft::new_source_root_with_work(
            self,
            owner,
            super::super::initial::CommandStorage { commands, capacity },
            limits,
            super::super::initial::Replay {
                work: observation.work.work.clone(),
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
    pub(in crate::internal::store) fn source_reserved_closing(
        &self,
        cancel: &AtomicBool,
    ) -> Result<u64, DomError> {
        check_cancel(cancel)?;
        let mut remaining = 0u64;
        // These are the actual retained original operations, not a count of
        // observers. Rebase and parent moves preserve each operation's credit.
        for operation in self.source_observers() {
            operation
                .callback_observation()
                .expect("observer")
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
        dialog_toggle_disposition: Option<crate::internal::source::DomDialogToggleTaskDisposition>,
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
            owner: observation.common.owner.clone(),
            node_outputs: Vec::new(),
            node_output_kinds: Vec::new(),
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
            custom_element_disposition: None,
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
            crate::internal::dispatch::DomDispatchAssociation::Retired(_)
        ) {
            return self
                .finish_dialog_noop(
                    false,
                    Some(crate::internal::source::DomDialogToggleTaskDisposition::TargetRetiredDuringDispatch),
                    cancel,
                )
                .map_err(Into::into);
        }
        self.rebase_event_observation(None, execution, cancel)
    }
    pub(super) fn advance_event_observation(
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
        let form_reset = observation.request.kind() == DomSourceEventKind::Reset;
        if form_reset
            && operation.cancelled
            && observation.request.work.stage.load(Ordering::Relaxed) == 0
        {
            return self.rebase_event_observation(None, execution, cancel);
        }
        if matches!(observation.time, FiringTime::Pending { .. }) {
            if operation.request_close.is_none() && operation.cancelled {
                return self
                    .finish_dialog_noop(true, None, cancel)
                    .map_err(Into::into);
            }
            return Err(if execution.hosted_events {
                AdvanceError::EventFiring
            } else {
                DomError::HostedSourceUnavailable.into()
            });
        }
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
        if form_reset {
            return self.rebase_event_observation(None, execution, cancel);
        }
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
                    surgeist_html::elements::dialog::HtmlCloseWatcherRequestReply::CancelActionResult(
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
            crate::internal::dispatch::DomDispatchAssociation::Current(_) => {}
            crate::internal::dispatch::DomDispatchAssociation::Retired(_)
                if operation.request_close.is_some() || operation.is_show_modal_method() =>
            {
                let cause = crate::internal::source::DomSourceCleanupCause::ReceiverRetired {
                    subject,
                    action: crate::internal::source::DomSourceStoppedAction::Dialog(action),
                };
                return self.rebase_dialog_failure(cause, execution, cancel);
            }
            crate::internal::dispatch::DomDispatchAssociation::Retired(_) => {
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
        self.rebase_event_observation(Some(frame), execution, cancel)
    }
    fn rebase_request_close_cancel(
        &mut self,
        frame: crate::internal::store::dialog::MethodFrame,
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
        let result = self.rebase_event_observation(Some(frame), execution, cancel);
        if let Some(operation) = self.dom.source_operation.as_mut()
            && operation.observation().is_some()
        {
            operation.request_close = old;
        }
        result
    }
    fn rebase_dialog_failure(
        &mut self,
        cause: crate::internal::source::DomSourceCleanupCause,
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
        self.rebase_event_observation(None, execution, cancel)
    }
    fn rebase_event_observation(
        &mut self,
        frame: Option<crate::internal::store::dialog::MethodFrame>,
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
        let form_reset = observation.request.kind() == DomSourceEventKind::Reset;
        let owner = observation.common.owner.clone();
        let work = observation.common.work.work.clone();
        let limits = self.dom.source_rebase_transferred_limits(
            observation.common.limits,
            observation.common.inputs,
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
                if form_reset
                    || operation.request_close.is_some()
                    || operation.failure.is_some() =>
            {
                None
            }
            Err(error) => return Err(error.into()),
        };
        let mut reset_body = None;
        if form_reset {
            if let Some(key) = key {
                let reset = !operation.cancelled
                    && observation.accepted == Some(DomSourceEventOutcome::NotCanceled);
                if reset {
                    reset_body = Some(super::hosted_removals::ResetBody::start(
                        &mut draft, key, cancel,
                    )?);
                }
                draft.stage_form_reset_tail(key, reset, cancel)?;
            }
        } else if let Some(mut frame) = frame {
            if let Some(key) = key {
                frame.subject = key;
                draft.source_set_method(
                    Some(crate::internal::store::method::Frame::Dialog(frame)),
                    cancel,
                )?;
            }
        } else if matches!(operation.commands, OperationRoot::DialogTask(_)) {
            draft.clear_dialog_toggle_tracker(key.ok_or(DomError::StaleNode)?, cancel)?;
        }
        let custody_bytes = self.dom.source_owner_custody_bytes()?;
        let custody_units = self.dom.fullscreen_custody_units()?;
        let mut operation = self.dom.source_operation.take().expect("retained observer");
        operation.reset_body = reset_body;
        operation.prepare_rebased_inline(self.dom, &mut draft);
        let old = std::mem::replace(&mut operation.draft, OperationDraft::Segment(draft));
        let OperationDraft::Observed(observation) = old else {
            unreachable!("qualified observer")
        };
        observation.common.usage.restore(&mut operation.draft);
        operation.command_storage = command;
        operation.custody_bytes = custody_bytes;
        operation.custody_units = custody_units;
        operation.limits = observation.common.limits;
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
        if form_reset {
            operation.command_admitted = false;
        }
        let result = self.advance_operation(&mut operation, execution, None, cancel);
        if self.complete
            || matches!(
                &result,
                Ok(DomSourceProgress::Complete(_) | DomSourceProgress::Failed(_))
            )
        {
            self.complete = true;
            drop(operation);
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
