use super::*;

/// One affine ordinary dispatch prepared for an exact Event, target and registry.
pub struct DomEventDispatch {
    owner: EventInvocationOwner,
    registry: u64,
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
        if event.is_dispatching() {
            return Err(DomEventDispatchError::AlreadyDispatching);
        }
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
}

impl DomEventDispatch {
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
        dispatcher.deliver(
            dom.raw_mut(),
            context,
            &self.owner.dispatch_observation(),
            DispatchCall {
                limits,
                cancel,
                host: false,
                funding: DispatchFunding(Funding::Owned(allocation)),
            },
        )
    }
}
