use super::*;

/// One affine dispatch prepared for an exact Event, target and registry.
pub struct DomEventDispatch {
    owner: EventInvocationOwner,
    registry: u64,
}

pub(crate) struct NativeSlotCheckpoint {
    pub(crate) targets: std::sync::Arc<[NodeId]>,
    pub(crate) next: usize,
    pub(crate) dispatch: Option<DomEventDispatch>,
    _backing: EventAllocation,
}
impl NativeSlotCheckpoint {
    pub(crate) fn advance_target(&mut self) {
        self.dispatch = None;
        self.next += 1;
    }
}
impl std::fmt::Debug for DomEventDispatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomEventDispatch")
            .field("event", &self.owner.event)
            .field("target", &self.owner.state.origin.target())
            .finish()
    }
}

impl<C> DomEventDispatcher<C> {
    pub(crate) fn take_slot_checkpoint(
        &self,
        dom: &mut DomEventDom<'_>,
        output_limit: usize,
        cancel: &AtomicBool,
    ) -> Result<Option<NativeSlotCheckpoint>, DomEventDispatchError> {
        self.association.validate(dom.raw())?;
        let count = dom.snapshot().pending_slot_signals().len();
        let limits = surgeist_dom::shadow::DomSlotQueryLimits::new(output_limit, output_limit)?;
        self.storage.step(count, cancel)?;
        let target_bytes = count
            .checked_mul(std::mem::size_of::<NodeId>())
            .ok_or(DomEventDispatchError::Admission(DomEventResource::Storage))?;
        let bytes = target_bytes
            .checked_add(2 * std::mem::size_of::<usize>())
            .ok_or(DomEventDispatchError::Admission(DomEventResource::Storage))?;
        // Admit the formation overlap and release the temporary Vec debit as
        // soon as the canonical transfer returns its immutable target list.
        let _scratch = self
            .storage
            .reserve(dom, target_bytes, target_bytes, 0, cancel)?;
        let backing = self.storage.reserve(dom, bytes, bytes, 0, cancel)?;
        Ok(dom
            .raw_mut()
            .take_native_slot_checkpoint(limits, cancel)?
            .map(|targets| NativeSlotCheckpoint {
                targets,
                next: 0,
                dispatch: None,
                _backing: backing,
            }))
    }

    pub(crate) fn prepare_slot_change(
        &mut self,
        dom: &mut DomEventDom<'_>,
        target: NodeId,
        creation: Result<EventCreation, DomEventTimestampUnavailable>,
        cancel: &AtomicBool,
    ) -> Result<DomEventDispatch, DomEventDispatchError> {
        self.association.validate(dom.raw())?;
        self.validate_target(dom.raw(), &target.into(), cancel)?;
        self.storage.step(1, cancel)?;
        let backing = EventBodyCustody::owned(dom, &self.storage, cancel)?;
        Ok(DomEventDispatch {
            owner: NativeDomEvent::slot_change(target, creation, backing),
            registry: self.owner.id,
        })
    }

    /// Prepare fixed custody for one ordinary dispatch. Preparation keeps
    /// the Event idle. Its first admitted advance commits the dispatch flag.
    pub fn prepare_dispatch(
        &mut self,
        dom: &mut DomEventDom<'_>,
        target: impl Into<DomEventTarget>,
        event: &NativeDomEvent,
        cancel: &AtomicBool,
    ) -> Result<DomEventDispatch, DomEventDispatchError> {
        self.association.validate(dom.raw())?;
        event.check_dispatch_state()?;
        if event.0.owner != dom.owner() {
            return Err(DomError::ForeignOwner.into());
        }
        let target = target.into();
        if let DomEventTarget::Global(global) = &target {
            self.storage.step(1, cancel)?;
            dom.raw()
                .dispatch_global_document_with(*global, cancel, &mut || {
                    self.storage.step(1, cancel)
                })?;
        } else {
            self.validate_target(dom.raw(), &target, cancel)?;
            self.storage.step(1, cancel)?;
        }
        let bytes = EventBodyCustody::continuation_bytes();
        let allocation = self.storage.reserve(dom, bytes, bytes, 0, cancel)?;
        let owner = EventInvocationOwner::from_event(
            event.clone(),
            target.into(),
            BodyCustody {
                _storage: BodyStorage::Owned {
                    _retained: allocation,
                },
            },
        );
        Ok(DomEventDispatch {
            owner,
            registry: self.owner.id,
        })
    }

    /// Admit the fixed native firing before the signal state changes.
    pub(super) fn admit_signal_abort(
        &self,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<SignalAbortFiring, DomEventDispatchError> {
        let pending = self.reserve_pending()?;
        self.storage.step(1, cancel)?;
        let backing = EventBodyCustody::owned(dom, &self.storage, cancel)?;
        let bytes = std::mem::size_of::<PrepaidAbortReservation>();
        let funding = self.storage.reserve(dom, bytes, bytes, 0, cancel)?;
        Ok(SignalAbortFiring {
            pending: Rc::new(PrepaidAbortReservation {
                reservation: pending,
                _funding: funding,
            }),
            backing,
        })
    }

    #[cfg(feature = "document-scripting")]
    pub(crate) fn prepare_parsing_event(
        &mut self,
        dom: &mut DomEventDom<'_>,
        event: ParsingEvent,
        (global, document): (
            surgeist_dom::hosted::DomHostedGlobalId,
            surgeist_dom::identity::DocumentId,
        ),
        creation: &mut Option<Result<EventCreation, DomEventTimestampUnavailable>>,
        cancel: &AtomicBool,
    ) -> Result<DomEventDispatch, DomEventDispatchError> {
        self.association.validate(dom.raw())?;
        if creation.is_none() {
            return Err(DomEventDispatchError::WrongContinuation);
        }
        let associated = dom
            .raw()
            .dispatch_global_document_with(global, cancel, &mut || self.storage.step(1, cancel))?;
        if associated != document {
            return Err(DomError::WrongDocument.into());
        }
        self.storage.step(1, cancel)?;
        let backing = EventBodyCustody::owned(dom, &self.storage, cancel)?;
        let creation = creation
            .take()
            .ok_or(DomEventDispatchError::WrongContinuation)?;
        let origin = match event {
            ParsingEvent::WindowLoad | ParsingEvent::PageShow => {
                NativeDispatchOrigin::WindowParsing {
                    global,
                    document,
                    event,
                }
            }
            ParsingEvent::ReadyStateChange | ParsingEvent::DomContentLoaded => {
                NativeDispatchOrigin::Node {
                    node: document.node(),
                    cause: NativeEventCause::Parsing(event),
                }
            }
        };
        let page_show = event == ParsingEvent::PageShow;
        let payload = if page_show {
            EventPayload::PageTransition { persisted: false }
        } else {
            EventPayload::None
        };
        Ok(DomEventDispatch {
            owner: EventInvocationOwner::from_native(
                origin,
                payload,
                DomEventInit {
                    bubbles: event == ParsingEvent::DomContentLoaded || page_show,
                    cancelable: page_show,
                    composed: false,
                },
                creation,
                backing,
            ),
            registry: self.owner.id,
        })
    }

    /// Prepare the report's selected native delivery. Refused preparation
    /// leaves the task's original creation observation available to retry.
    #[cfg(feature = "document-scripting")]
    pub(crate) fn prepare_csp_violation(
        &mut self,
        dom: &mut DomEventDom<'_>,
        document: surgeist_dom::identity::DocumentId,
        report: Rc<QueuedViolation>,
        creation: &mut Option<Result<EventCreation, DomEventTimestampUnavailable>>,
        cancel: &AtomicBool,
    ) -> Result<DomEventDispatch, DomEventDispatchError> {
        if creation.is_none() {
            return Err(DomEventDispatchError::WrongContinuation);
        }
        self.association.validate(dom.raw())?;
        let mut node = document.node();
        if let Some(element) = report.element() {
            let snapshot = dom.snapshot();
            let mut candidate = element;
            loop {
                match snapshot.node_with(candidate, || self.storage.step(1, cancel)) {
                    Ok(current) if current.id() == document.node() => {
                        node = element;
                        break;
                    }
                    Ok(current) => {
                        let Some(parent) = current.shadow_including_parent() else {
                            break;
                        };
                        candidate = parent;
                    }
                    Err(DomEventDispatchError::Dom(DomError::StaleNode)) => break,
                    Err(error) => return Err(error),
                }
            }
        }
        self.validate_target(dom.raw(), &node.into(), cancel)?;
        self.storage.step(1, cancel)?;
        let payload = csp::CspEventData::native(dom, &self.storage, document, report, cancel)?;

        let backing = EventBodyCustody::owned(dom, &self.storage, cancel)?;
        let creation = creation
            .take()
            .ok_or(DomEventDispatchError::WrongContinuation)?;
        Ok(DomEventDispatch {
            owner: EventInvocationOwner::new(
                node,
                EventData::CspViolation(payload),
                creation,
                backing,
            ),
            registry: self.owner.id,
        })
    }
}

pub(super) struct SignalAbortFiring {
    pending: Rc<PrepaidAbortReservation>,
    backing: EventBodyCustody,
}
impl SignalAbortFiring {
    pub(super) fn prepare(
        self,
        target: DomStandaloneEventTarget,
        creation: Result<EventCreation, DomEventTimestampUnavailable>,
    ) -> DomEventDispatch {
        let owner = EventInvocationOwner::from_native(
            NativeDispatchOrigin::SignalAbort(target),
            EventPayload::None,
            DomEventInit::default(),
            creation,
            self.backing,
        );
        let registry = self.pending.reservation.0.id;
        owner._custody.prepaid_abort.set(Some(self.pending));
        DomEventDispatch { owner, registry }
    }
}

impl DomEventDispatch {
    pub(crate) fn has_started(&self) -> bool {
        self.owner.has_started()
    }

    pub(crate) fn belongs_to<C>(&self, dispatcher: &DomEventDispatcher<C>) -> bool {
        self.registry == dispatcher.owner.id
    }

    pub(crate) fn is_abandoned(&self) -> bool {
        matches!(
            self.owner.terminal_dispatch_error(),
            Some(DomEventDispatchError::Abandoned)
        )
    }

    /// Drive this same invocation synchronously through the canonical kernel.
    /// A host registration retains the cursor with `RetainedHostUnavailable`.
    pub fn advance<C>(
        &mut self,
        dom: &mut DomEventDom<'_>,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        limits: DomEventDispatchLimits,
        cancel: &AtomicBool,
    ) -> Result<DomEventDispatchOutcome, DomEventDispatchError> {
        self.advance_with_host(
            dom,
            dispatcher,
            context,
            limits,
            cancel,
            DispatchHost::Unavailable,
        )
    }

    pub(crate) fn advance_with_host<C>(
        &mut self,
        dom: &mut DomEventDom<'_>,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        limits: DomEventDispatchLimits,
        cancel: &AtomicBool,
        host: DispatchHost,
    ) -> Result<DomEventDispatchOutcome, DomEventDispatchError> {
        self.advance_with_host_usage(
            dom,
            dispatcher,
            context,
            (limits, &mut DispatchUsage::default()),
            cancel,
            host,
        )
    }

    /// Share the current call's budget across owned host listener selections.
    pub(crate) fn advance_with_host_usage<C>(
        &mut self,
        dom: &mut DomEventDom<'_>,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        (limits, usage): (DomEventDispatchLimits, &mut DispatchUsage),
        cancel: &AtomicBool,
        host: DispatchHost,
    ) -> Result<DomEventDispatchOutcome, DomEventDispatchError> {
        if self.registry != dispatcher.owner.id {
            return Err(DomEventDispatchError::WrongContinuation);
        }
        if self.owner.event.0.owner != dom.owner() {
            return Err(DomError::ForeignOwner.into());
        }
        dispatcher.association.validate(dom.raw())?;
        if let Some(error) = self.owner.terminal_dispatch_error() {
            return Err(error);
        }
        match self.owner.state.status.get() {
            Status::Ready if self.owner.event.is_dispatching() => {
                return Err(DomEventDispatchError::AlreadyDispatching);
            }
            Status::Complete => return Err(DomEventDispatchError::AlreadyCompleted),
            Status::Running | Status::HostPending => {
                return Err(DomEventDispatchError::AlreadyDispatching);
            }
            _ => (),
        }
        let formation = self
            .owner
            .dispatch_formation(limits)
            .ok_or(DomEventDispatchError::Admission(DomEventResource::Storage))?;
        let allocation =
            dispatcher
                .storage
                .reserve(dom, formation.full, formation.source, 0, cancel)?;
        dispatcher.deliver_with_usage(
            dom.raw_mut(),
            context,
            &self.owner.dispatch_observation(),
            DispatchCall {
                limits,
                cancel,
                host,
                funding: DispatchFunding(Funding::Owned(allocation)),
            },
            usage,
        )
    }
}
