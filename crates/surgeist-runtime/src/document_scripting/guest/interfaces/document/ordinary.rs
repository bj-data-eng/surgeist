//! Retained custody indexed by exact native node retirement.

use super::*;
use surgeist_bindings_js::NativeProjectionPrototype;
use surgeist_dom::{DomNodeRetirementWatch, DomSourceEventRequest};

pub(super) struct OrdinaryBindings {
    inbox: RefCell<DomNodeRetirementInbox>,
    pins: RefCell<OrdinaryPins>,
    #[cfg(test)]
    pub(super) refuse_next_validation: Cell<bool>,
}

pub(super) enum NodePrototype<'value, 'scope> {
    Selected(&'value Value<'scope>),
    Creation(&'value RetainedCallbackObject),
}

impl<'value, 'scope> NodePrototype<'value, 'scope> {
    pub(super) fn projection(self) -> NativeProjectionPrototype<'value, 'scope> {
        match self {
            Self::Selected(prototype) => {
                NativeProjectionPrototype::Current(ObjectPrototype::Object(prototype))
            }
            Self::Creation(prototype) => NativeProjectionPrototype::Creation(prototype),
        }
    }
}

enum Projection<'prototype, 'scope, Publish> {
    Create(NodePrototype<'prototype, 'scope>, Publish),
    Watch(&'prototype RetainedCallbackObject, Publish),
}

#[derive(Clone, Copy)]
pub(super) struct Admission<'a> {
    owner: &'a NativeOwner,
    events: &'a GuestEvents,
    request: Option<&'a DomSourceEventRequest>,
}

impl<'a> Admission<'a> {
    fn ordinary(owner: &'a NativeOwner, events: &'a GuestEvents) -> Self {
        Self {
            owner,
            events,
            request: None,
        }
    }

    pub(super) fn source(
        owner: &'a NativeOwner,
        events: &'a GuestEvents,
        request: &'a DomSourceEventRequest,
    ) -> Self {
        Self {
            owner,
            events,
            request: Some(request),
        }
    }

    fn source_step(self) -> Result<(), DocumentGuestEventError> {
        if let Some(request) = self.request {
            let mut session = self
                .owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session
                .dom()
                .raw()
                .source_event_step(request, &self.owner.cancellation)?;
        }
        Ok(())
    }

    pub(super) fn step(self) -> Result<(), DocumentGuestEventError> {
        self.source_step()?;
        self.events.step()
    }

    fn reserve(self, bytes: usize) -> Result<GuestBacking, DocumentGuestEventError> {
        self.source_step()?;
        self.events.reserve(self.owner, bytes)
    }
}

#[derive(Default)]
struct OrdinaryPins {
    slots: Vec<Slot>,
    free: Option<usize>,
    slot_backing: Option<GuestBacking>,
}

impl Drop for OrdinaryPins {
    fn drop(&mut self) {
        for slot in self.slots.drain(..) {
            if let Slot::Retained(pin) = slot {
                let released = {
                    let allocation = pin;
                    *allocation
                };
                drop(released);
            }
        }
    }
}

enum Slot {
    Free(Option<usize>),
    Retained(Box<Pin>),
}

struct Pin {
    holder: RetainedCallbackObject,
    live: LivePin,
    lifetime: Rc<NodeLifetime>,
    _backing: GuestBacking,
}

pub(in crate::document_scripting::guest) struct NodeLifetime {
    retired: Cell<bool>,
    _backing: GuestBacking,
}

impl NodeLifetime {
    pub(in crate::document_scripting::guest) fn is_retired(&self) -> bool {
        self.retired.get()
    }
}

struct LivePin {
    node: NodeId,
    _watch: DomNodeRetirementWatch,
}

enum ReservedSlot<'a> {
    Append(&'a mut Vec<Slot>),
    Grow {
        slots: &'a mut Vec<Slot>,
        old_backing: &'a mut Option<GuestBacking>,
        replacement: Vec<Slot>,
        backing: GuestBacking,
    },
    Reuse {
        slot: &'a mut Slot,
        free: &'a mut Option<usize>,
        next: Option<usize>,
    },
}

impl ReservedSlot<'_> {
    fn publish(self, pin: Box<Pin>) {
        match self {
            Self::Append(slots) => slots.push(Slot::Retained(pin)),
            Self::Grow {
                slots,
                old_backing,
                mut replacement,
                backing,
            } => {
                replacement.append(slots);
                replacement.push(Slot::Retained(pin));
                drop(std::mem::replace(slots, replacement));
                *old_backing = Some(backing);
            }
            Self::Reuse { slot, free, next } => {
                *slot = Slot::Retained(pin);
                *free = next;
            }
        }
    }
}

impl OrdinaryPins {
    fn reserve<'a>(
        &'a mut self,
        admission: Admission<'_>,
    ) -> Result<(usize, ReservedSlot<'a>), DocumentGuestEventError> {
        admission.step()?;
        if let Some(key) = self.free {
            let slot = self
                .slots
                .get_mut(key)
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            let Slot::Free(next) = slot else {
                return Err(DocumentScriptNativeError::ProjectionCustody.into());
            };
            let next = *next;
            return Ok((
                key,
                ReservedSlot::Reuse {
                    slot,
                    free: &mut self.free,
                    next,
                },
            ));
        }
        let key = self.slots.len();
        if key == self.slots.capacity() {
            let capacity = key
                .max(2)
                .checked_mul(2)
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
            let bytes = capacity
                .checked_mul(size_of::<Slot>())
                .ok_or(DocumentScriptNativeError::StorageLimit)?;
            let backing = admission.reserve(bytes)?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(capacity)
                .map_err(|_| Error::Allocation)?;
            if replacement.capacity() > capacity {
                return Err(DocumentScriptNativeError::StorageLimit.into());
            }
            for _ in &self.slots {
                admission.step()?;
            }
            return Ok((
                key,
                ReservedSlot::Grow {
                    slots: &mut self.slots,
                    old_backing: &mut self.slot_backing,
                    replacement,
                    backing,
                },
            ));
        }
        Ok((key, ReservedSlot::Append(&mut self.slots)))
    }
}

impl OrdinaryBindings {
    pub(super) fn new(inbox: DomNodeRetirementInbox) -> Self {
        Self {
            inbox: RefCell::new(inbox),
            pins: RefCell::new(OrdinaryPins::default()),
            #[cfg(test)]
            refuse_next_validation: Cell::new(false),
        }
    }

    pub(super) fn create<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        prototype: NodePrototype<'_, 'scope>,
        create: impl FnOnce(
            &DomNodeRetirementInbox,
            usize,
        )
            -> Result<(GuestNode, DomNodeRetirementWatch), DocumentGuestEventError>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        self.project(
            scope,
            Admission::ordinary(owner, events),
            Projection::Create(prototype, create),
        )
    }

    pub(super) fn watch<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        callbacks: &Callbacks,
        node: GuestNode,
        prototype: &RetainedCallbackObject,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        self.project(
            scope,
            Admission::ordinary(owner, events),
            Projection::Watch(prototype, |inbox: &DomNodeRetirementInbox, key| {
                let watch = callbacks.with_native(owner, |session, _| {
                    session
                        .dom()
                        .raw_mut()
                        .watch_node_retirement(
                            node.node(),
                            inbox,
                            key,
                            events.limits.node_retirement,
                            &owner.cancellation,
                        )
                        .map_err(Into::into)
                })?;
                Ok((node, watch))
            }),
        )
    }

    pub(super) fn watch_source<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        request: &DomSourceEventRequest,
        node: GuestNode,
        prototype: &RetainedCallbackObject,
        handler: Result<(), Error>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        self.project(
            scope,
            Admission::source(owner, events, request),
            Projection::Watch(prototype, |inbox: &DomNodeRetirementInbox, key| {
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let watch = session.dom().raw_mut().watch_source_event_node_retirement(
                    request,
                    node.node(),
                    inbox,
                    key,
                    &owner.cancellation,
                )?;
                handler?;
                Ok((node, watch))
            }),
        )
    }

    fn project<'scope, Publish>(
        &self,
        scope: &mut Scope<'scope>,
        admission: Admission<'_>,
        projection: Projection<'_, 'scope, Publish>,
    ) -> Result<Value<'scope>, DocumentGuestEventError>
    where
        Publish: FnOnce(
            &DomNodeRetirementInbox,
            usize,
        )
            -> Result<(GuestNode, DomNodeRetirementWatch), DocumentGuestEventError>,
    {
        let mut pins = self
            .pins
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let (key, reserved) = pins.reserve(admission)?;
        let inbox = self
            .inbox
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let backing = admission.reserve(size_of::<Pin>())?;
        let lifetime_backing =
            admission.reserve(size_of::<NodeLifetime>() + 2 * size_of::<usize>())?;
        let lifetime = Rc::new(NodeLifetime {
            retired: Cell::new(false),
            _backing: lifetime_backing,
        });
        let payload = GuestNodeProjection {
            node: Cell::new(None),
            lifetime: Some(lifetime.clone()),
        };
        let allocation = Box::new_uninit();
        let (value, holder, live) = match projection {
            Projection::Create(prototype, create) => {
                admission.step()?;
                scope.try_project_retained_native_object(
                    prototype.projection(),
                    payload,
                    |payload, holder| {
                        Ok::<_, DocumentGuestEventError>((
                            admission.reserve(payload)?,
                            admission.reserve(holder)?,
                        ))
                    },
                    |payload| {
                        #[cfg(test)]
                        self.qualify_projection()?;
                        let (node, watch) = create(&inbox, key)?;
                        Ok(Self::bind_projection(payload, node, watch))
                    },
                )?
            }
            Projection::Watch(prototype, watch) => {
                let mut watched = None;
                let holder =
                    Self::form_in_prototype_realm(scope, admission, prototype, payload, || {
                        watched = Some(watch(&inbox, key)?);
                        Ok(())
                    })?;
                let (node, watch) = watched.ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                let qualified = (|| {
                    #[cfg(test)]
                    self.qualify_projection()?;
                    let value = scope.open_callback_object(&holder)?;
                    let projection = scope
                        .native_object::<GuestNodeProjection>(&value)?
                        .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                    Ok::<_, DocumentGuestEventError>((
                        value,
                        Self::bind_projection(projection.payload(), node, watch),
                    ))
                })();
                match qualified {
                    Ok((value, live)) => (value, holder, live),
                    Err(error) => {
                        let _ = scope.release_callback_object(holder);
                        return Err(error);
                    }
                }
            }
        };
        let pin = Box::write(
            allocation,
            Pin {
                holder,
                live,
                lifetime,
                _backing: backing,
            },
        );
        reserved.publish(pin);
        Ok(value)
    }

    #[cfg(test)]
    fn qualify_projection(&self) -> Result<(), DocumentGuestEventError> {
        if self.refuse_next_validation.replace(false) {
            return Err(Error::BackendUnavailable.into());
        }
        Ok(())
    }

    fn bind_projection(
        payload: &GuestNodeProjection,
        node: GuestNode,
        watch: DomNodeRetirementWatch,
    ) -> LivePin {
        payload.node.set(Some(node));
        LivePin {
            node: node.node(),
            _watch: watch,
        }
    }

    fn form_in_prototype_realm(
        scope: &mut Scope<'_>,
        admission: Admission<'_>,
        prototype: &RetainedCallbackObject,
        payload: GuestNodeProjection,
        admit_watch: impl FnOnce() -> Result<(), DocumentGuestEventError>,
    ) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
        admission.step()?;
        scope.project_retained_native_object_in_prototype_realm(
            prototype,
            payload,
            |payload, holder| {
                let payload = admission.reserve(payload)?;
                let holder = admission.reserve(holder)?;
                admit_watch()?;
                Ok((payload, holder))
            },
        )
    }

    pub(super) fn open<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        node: NodeId,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        self.open_with(scope, node, || events.step())
    }

    pub(super) fn open_with<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        node: NodeId,
        mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        let pins = self
            .pins
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        for slot in &pins.slots {
            step()?;
            if let Slot::Retained(pin) = slot
                && pin.live.node == node
            {
                return Ok(Some(scope.open_callback_object(&pin.holder)?));
            }
        }
        Ok(None)
    }

    pub(super) fn reap(&self, process: &mut Process, retired: &Cell<bool>) {
        let (Ok(mut inbox), Ok(mut pins)) =
            (self.inbox.try_borrow_mut(), self.pins.try_borrow_mut())
        else {
            return;
        };
        while let Some(notice) = inbox.take() {
            let key = notice.custody_key();
            let next = pins.free;
            let Some(slot) = pins.slots.get_mut(key) else {
                continue;
            };
            if !matches!(slot, Slot::Retained(pin)
                if pin.live.node == notice.node())
            {
                continue;
            }
            if let Slot::Retained(pin) = std::mem::replace(slot, Slot::Free(next)) {
                pins.free = Some(key);
                pin.lifetime.retired.set(true);
                retired.set(true);
                let Pin {
                    holder,
                    live,
                    lifetime,
                    _backing: backing,
                } = {
                    let allocation = pin;
                    *allocation
                };
                let _ = process.release_callback_object(holder);
                drop(live);
                drop(lifetime);
                drop(backing);
            }
        }
    }
}
