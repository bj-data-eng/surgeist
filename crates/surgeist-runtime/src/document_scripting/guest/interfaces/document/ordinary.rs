//! Retained custody indexed by exact native node retirement.

#[cfg(test)]
mod custody_tests;
#[cfg(test)]
pub(in crate::document_scripting) use custody_tests::missing_pins_refuse_without_native_publication;

use super::super::PublicEventBindings;
use super::super::custom_elements::{ActiveReactionQueue, SynchronousReactionScope};
use super::*;
use surgeist_bindings_js::NativeProjectionPrototype;
use surgeist_dom::{
    DomDetachedNodeCreation, DomDocumentScriptRegistration, DomElementCreationName,
    DomElementCreationOptions, DomNodeRetirementWatch, DomSourceAttributeAttachmentOutput,
    DomSourceAttributeVerificationRequest, DomSourceCustomElementOutcome,
    DomSourceCustomElementRequest, DomSourceCustomElementReturn, DomSourceEventRequest,
    DomWatchedCustomElementResult,
};

pub(in crate::document_scripting::guest::interfaces) struct OrdinaryBindings {
    inbox: RefCell<DomNodeRetirementInbox>,
    pins: RefCell<OrdinaryPins>,
    #[cfg(test)]
    pub(super) refuse_next_validation: Cell<bool>,
    #[cfg(test)]
    pub(super) refuse_next_constructor_fallback: Cell<bool>,
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

#[derive(Clone, Copy)]
pub(super) struct Admission<'a> {
    owner: &'a NativeOwner,
    events: &'a GuestEvents,
    route: AdmissionRoute<'a>,
}

enum ElementFactoryResult<'scope> {
    Created(Value<'scope>),
    Constructor(surgeist_dom::DomSourceCustomElementRequest),
}

pub(in crate::document_scripting) struct PendingConstructor {
    request: DomSourceCustomElementRequest,
    registration: Rc<DomDocumentScriptRegistration>,
    outcome: Option<DomSourceCustomElementOutcome>,
    returned: Option<surgeist_bindings_js::Root>,
    receiver: Option<NodeId>,
    realm: RealmId,
    report: super::super::super::reports::ConstructorReportReservation,
    exception: Option<surgeist_bindings_js::Exception>,
    upgrade_callbacks: Option<Rc<ActiveReactionQueue>>,
    next: Option<Box<Self>>,
    _backing: GuestBacking,
}

impl Drop for PendingConstructor {
    fn drop(&mut self) {
        let mut pending = self.next.take();
        while let Some(mut frame) = pending {
            pending = frame.next.take();
        }
    }
}

impl NativeOwner {
    pub(in crate::document_scripting) fn reconcile_pending_constructors(
        &self,
    ) -> Result<(), DocumentGuestEventError> {
        let mut slot = self
            .pending_constructor
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        if slot.is_none() {
            return Ok(());
        }
        let mut session = self
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let dom = session.dom();
        let cancel = AtomicBool::new(false);
        let mut link = &mut *slot;
        while let Some(frame) = link.as_mut() {
            if dom.raw().retains_custom_element(frame.request, &cancel)? {
                link = &mut link
                    .as_mut()
                    .ok_or(DocumentScriptNativeError::SourceAccess)?
                    .next;
            } else {
                if let Some(queue) = frame.upgrade_callbacks.take() {
                    self.keep_stopped_reaction_queue(queue);
                }
                *link = frame.next.take();
            }
        }
        Ok(())
    }

    fn keep_constructor_result(&self, mut pending: Box<PendingConstructor>) {
        let mut slot = self.pending_constructor.borrow_mut();
        pending.next = slot.take();
        *slot = Some(pending);
    }

    pub(in crate::document_scripting) fn raise_stopped_constructor_limits(
        &self,
        limits: surgeist_dom::DomSourceLimits,
    ) -> Result<(), DocumentScriptNativeError> {
        let slot = self
            .pending_constructor
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let Some(pending) = slot.as_ref() else {
            return Ok(());
        };
        let mut session = self
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        session
            .dom()
            .raw_mut()
            .raise_stopped_constructor_ancestors(
                &pending.registration,
                limits,
                &AtomicBool::new(false),
            )?;
        Ok(())
    }

    pub(in crate::document_scripting) fn settle_stopped_constructor(
        &self,
        limits: surgeist_dom::DomSourceLimits,
    ) -> Result<(), DocumentGuestEventError> {
        self.raise_stopped_constructor_limits(limits)?;
        let mut session = self
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut dom = session.dom();
        let current = dom
            .pending_source_edit()
            .ok_or(DocumentScriptNativeError::SourceAccess)?
            .operation();
        let mut slot = self
            .pending_constructor
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut link = &mut *slot;
        while let Some(frame) = link.as_ref() {
            if frame.request.operation() == current {
                break;
            }
            link = &mut link
                .as_mut()
                .ok_or(DocumentScriptNativeError::SourceAccess)?
                .next;
        }
        let pending = link
            .as_mut()
            .ok_or(DocumentScriptNativeError::SourceAccess)?;
        let cancel = AtomicBool::new(false);
        let mut lease = dom.raw_mut().resume_source_edit(current, limits, &cancel)?;
        if pending.outcome.is_some() {
            lease.accept_custom_element_callback(
                pending.request,
                &pending.registration,
                &mut pending.outcome,
                &cancel,
            )?;
        }
        if !matches!(
            lease.advance(&cancel)?,
            surgeist_dom::DomSourceProgress::Complete(_)
        ) {
            return Err(DocumentScriptNativeError::SourceAccess.into());
        }
        if let Some(queue) = pending.upgrade_callbacks.take() {
            self.keep_stopped_reaction_queue(queue);
        }
        *link = pending.next.take();
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum AdmissionRoute<'a> {
    Ordinary,
    Event(&'a DomSourceEventRequest),
    Attribute(
        DomSourceAttributeVerificationRequest,
        &'a DomDocumentScriptRegistration,
    ),
    Constructor(
        DomSourceCustomElementRequest,
        &'a DomDocumentScriptRegistration,
    ),
    AttachmentOutput(
        DomSourceAttributeAttachmentOutput,
        &'a DomDocumentScriptRegistration,
    ),
}

impl<'a> Admission<'a> {
    pub(super) fn constructor(
        owner: &'a NativeOwner,
        events: &'a GuestEvents,
        (request, registration): &'a (
            DomSourceCustomElementRequest,
            Rc<DomDocumentScriptRegistration>,
        ),
    ) -> Self {
        Self {
            owner,
            events,
            route: AdmissionRoute::Constructor(*request, registration),
        }
    }

    pub(super) fn ordinary(owner: &'a NativeOwner, events: &'a GuestEvents) -> Self {
        Self {
            owner,
            events,
            route: AdmissionRoute::Ordinary,
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
            route: AdmissionRoute::Event(request),
        }
    }

    pub(super) fn attribute(
        owner: &'a NativeOwner,
        events: &'a GuestEvents,
        request: DomSourceAttributeVerificationRequest,
        registration: &'a DomDocumentScriptRegistration,
    ) -> Self {
        Self {
            owner,
            events,
            route: AdmissionRoute::Attribute(request, registration),
        }
    }

    pub(super) fn attachment_output(
        owner: &'a NativeOwner,
        events: &'a GuestEvents,
        request: DomSourceAttributeAttachmentOutput,
        registration: &'a DomDocumentScriptRegistration,
    ) -> Self {
        Self {
            owner,
            events,
            route: AdmissionRoute::AttachmentOutput(request, registration),
        }
    }

    fn source_step(self) -> Result<(), DocumentGuestEventError> {
        if matches!(self.route, AdmissionRoute::Ordinary) {
            return Ok(());
        }
        let mut session = self
            .owner
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let dom = session.dom();
        let cancel = &self.owner.cancellation;
        match self.route {
            AdmissionRoute::Ordinary => (),
            AdmissionRoute::Event(request) => dom.raw().source_event_step(request, cancel)?,
            AdmissionRoute::Attribute(request, registration) => {
                dom.raw()
                    .source_attribute_callback_step(request, registration, cancel)?;
            }
            AdmissionRoute::AttachmentOutput(request, registration) => {
                dom.raw()
                    .source_attribute_attachment_output_step(request, registration, cancel)?;
            }
            AdmissionRoute::Constructor(request, registration) => {
                dom.raw()
                    .source_custom_element_callback_step(request, registration, cancel)?;
            }
        }
        Ok(())
    }

    pub(super) fn phase<T>(
        self,
        run: impl FnOnce() -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        self.events.budget.phase(run)
    }

    pub(super) fn step(self) -> Result<(), DocumentGuestEventError> {
        self.source_step()?;
        self.events.budget.charge_work(AccountScope::Projection, 1)
    }

    pub(super) fn reserve(self, bytes: usize) -> Result<GuestBacking, DocumentGuestEventError> {
        self.source_step()?;
        self.events
            .budget
            .reserve(AccountScope::Projection, self.owner, bytes)
    }

    fn accept_output(self, watch: &DomNodeRetirementWatch) -> Result<(), DocumentGuestEventError> {
        if let AdmissionRoute::AttachmentOutput(request, registration) = self.route {
            let mut session = self
                .owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session
                .dom()
                .raw_mut()
                .accept_source_attribute_attachment_output(
                    request,
                    registration,
                    watch,
                    &self.owner.cancellation,
                )?;
        }
        Ok(())
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

struct PreparedPin {
    backing: GuestBacking,
    lifetime: Rc<NodeLifetime>,
    allocation: Box<std::mem::MaybeUninit<Pin>>,
}

impl PreparedPin {
    fn new(admission: Admission<'_>) -> Result<Self, DocumentGuestEventError> {
        let backing = admission.reserve(size_of::<Pin>())?;
        let lifetime_backing =
            admission.reserve(size_of::<NodeLifetime>() + 2 * size_of::<usize>())?;
        Ok(Self {
            backing,
            lifetime: Rc::new(NodeLifetime {
                retired: Cell::new(false),
                _backing: lifetime_backing,
            }),
            allocation: Box::new_uninit(),
        })
    }

    fn projection(&self) -> GuestNodeProjection {
        GuestNodeProjection {
            node: Cell::new(None),
            lifetime: Some(self.lifetime.clone()),
        }
    }

    fn publish(self, reserved: ReservedSlot<'_>, holder: RetainedCallbackObject, live: LivePin) {
        reserved.publish(Box::write(
            self.allocation,
            Pin {
                holder,
                live,
                lifetime: self.lifetime,
                _backing: self.backing,
            },
        ));
    }
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
    fn run_constructor<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        public: &PublicEventBindings,
        request: DomSourceCustomElementRequest,
    ) -> Result<
        (
            Value<'scope>,
            Option<surgeist_dom::DomSynchronousReactionQueue>,
        ),
        DocumentGuestEventError,
    > {
        let reaction_scope = SynchronousReactionScope::enter(owner)?;
        let cancel = &owner.cancellation;
        let (constructor, registration, entry, backing, realm, report, customized) = (|| {
            let (constructor, realm, interface, customized) =
                public.factory_constructor(scope, events, request)?;
            let registration = owner
                .attachment
                .get()
                .and_then(|attachment| attachment.registration.upgrade())
                .ok_or(DocumentScriptNativeError::Revoked)?;
            let backing = events.reserve(owner, size_of::<PendingConstructor>())?;
            let report = events.reports.reserve_constructor()?;
            let entry = public.enter_factory_constructor(
                owner,
                events,
                scope,
                request,
                (&constructor, interface),
                registration.clone(),
            )?;
            Ok::<_, DocumentGuestEventError>((
                constructor,
                registration,
                entry,
                backing,
                realm,
                report,
                customized,
            ))
        })()
        .inspect_err(|_| {
            if let Ok(mut session) = owner.session.try_borrow_mut() {
                let _ = session
                    .dom()
                    .raw_mut()
                    .discard_unstarted_custom_element(request);
            }
        })?;
        let mut pending = Box::new(PendingConstructor {
            request,
            registration,
            outcome: None,
            returned: None,
            receiver: None,
            realm,
            report,
            exception: None,
            upgrade_callbacks: None,
            next: None,
            _backing: backing,
        });
        let callback_scope = if customized {
            self.open_customized_callback(
                scope,
                (owner, events),
                public,
                request,
                &pending.registration,
            )
            .map(|(callback, receiver)| (callback, Some(receiver)))
        } else {
            (|| {
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let mut dom = session.dom();
                let mut lease = dom.raw_mut().resume_source_edit(
                    request.operation(),
                    events.limits.source_edits,
                    cancel,
                )?;
                lease
                    .open_custom_element_callback(request, &pending.registration, cancel)
                    .map(|callback| (callback, None))
                    .map_err(DocumentGuestEventError::from)
            })()
        };
        let (callback_scope, receiver) = match callback_scope {
            Ok(callback_scope) => callback_scope,
            Err(error) => {
                drop(entry);
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                session
                    .dom()
                    .raw_mut()
                    .discard_unstarted_custom_element(request)?;
                return Err(error);
            }
        };
        if let Some((node, receiver)) = receiver {
            pending.receiver = Some(node);
            pending.returned = Some(scope.retain(&receiver));
            let _ = entry.frame.receiver.set(scope.retain(&receiver));
        }
        let result = scope.construct_without_arguments(&constructor);
        let reaction_queue = reaction_scope
            .has_reactions()
            .then(|| reaction_scope.queue());
        drop(reaction_scope);
        drop(callback_scope);
        drop(entry);
        let result = match result {
            Ok(value) => {
                if pending.receiver.is_none() {
                    pending.returned = Some(scope.retain(&value));
                }
                if let Some(receiver) = pending.receiver {
                    let exact = pending
                        .returned
                        .as_ref()
                        .ok_or(Error::ExpiredIdentity)
                        .and_then(|root| scope.import(root))
                        .and_then(|original| scope.strict_equals(&original, &value));
                    exact.map(|same| same.then_some(GuestNode::Element(receiver)))
                } else {
                    scope
                        .native_object::<GuestNodeProjection>(&value)
                        .map(|projection| projection.and_then(|native| native.payload().id()))
                }
            }
            Err(error) => Err(error),
        };
        pending.outcome = Some(match result {
            Ok(Some(GuestNode::Element(node))) => DomSourceCustomElementOutcome::Returned(node),
            Ok(_) => DomSourceCustomElementOutcome::InvalidInterface,
            Err(Error::Exception(exception)) => {
                pending.exception = Some(exception);
                DomSourceCustomElementOutcome::Failed
            }
            Err(error) => {
                pending.outcome = Some(DomSourceCustomElementOutcome::Failed);
                owner.keep_constructor_result(pending);
                return Err(error.into());
            }
        });
        let result = if pending.receiver.is_some() {
            self.finish_customized_constructor(scope, owner, events, &mut pending)
        } else {
            self.finish_constructor(scope, (owner, events), public, &mut pending)
        };
        match result {
            Ok(value) => Ok((value, reaction_queue)),
            Err(error) => {
                owner.keep_constructor_result(pending);
                Err(error)
            }
        }
    }

    pub(in crate::document_scripting::guest::interfaces) fn upgrade_existing<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        (node, definition, global): (
            NodeId,
            surgeist_dom::DomRegistryDefinitionId,
            DomHostedGlobalId,
        ),
        (constructor, interface, realm): (&Value<'scope>, &'static str, RealmId),
        (receiver, callbacks, accepted): (
            &Value<'scope>,
            Option<Rc<ActiveReactionQueue>>,
            &mut bool,
        ),
    ) -> Result<Option<Rc<ActiveReactionQueue>>, DocumentGuestEventError> {
        let registration = owner
            .attachment
            .get()
            .and_then(|attachment| attachment.registration.upgrade())
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let backing = events.reserve(owner, size_of::<PendingConstructor>())?;
        let mut report = events.reports.reserve_constructor()?;
        let frame = super::super::custom_elements::reserve_constructor_frame(owner, events)?;
        let retained_constructor = scope.retain(constructor);
        let retained_receiver = scope.retain(receiver);
        let callback = {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session.dom().raw_mut().open_custom_element_upgrade(
                node,
                (definition, global),
                &registration,
                events.limits.source_edits,
                &owner.cancellation,
            )?
        };
        let (request, callback, receiver_registry_matches) = match callback {
            None => return Ok(None),
            Some(surgeist_dom::DomSourceCustomElementUpgrade::Callback {
                request,
                scope,
                receiver_registry_matches,
            }) => (request, scope, receiver_registry_matches),
            Some(surgeist_dom::DomSourceCustomElementUpgrade::ShadowForbidden) => {
                *accepted = true;
                let value = scope.dom_exception(
                    surgeist_bindings_js::dom_exception::DomExceptionName::NotSupportedError,
                    &[],
                )?;
                let exception = scope.invocation_exception(&value);
                report.prepare()?;
                report.publish(DocumentGuestConstructorException {
                    definition,
                    realm,
                    exception,
                });
                return Ok(None);
            }
        };
        let mut pending = Box::new(PendingConstructor {
            request,
            registration: registration.clone(),
            outcome: Some(DomSourceCustomElementOutcome::Failed),
            returned: Some(retained_receiver.clone()),
            receiver: Some(node),
            realm,
            report,
            exception: None,
            upgrade_callbacks: None,
            next: None,
            _backing: backing,
        });
        *accepted = true;
        let run = (|| {
            let entry = PublicEventBindings::enter_constructor(
                owner,
                request,
                registration.clone(),
                retained_constructor,
                interface,
                frame,
            )?;
            entry
                .frame
                .receiver_registry_matches
                .set(receiver_registry_matches);
            entry
                .frame
                .receiver
                .set(retained_receiver)
                .map_err(|_| DocumentScriptNativeError::ProjectionCustody)?;
            let _reactions = callbacks
                .as_ref()
                .map(|queue| queue.enter(owner))
                .transpose()?;
            Ok::<_, DocumentGuestEventError>(scope.construct_without_arguments(constructor))
        })();
        drop(callback);
        let result = (|| {
            pending.outcome = Some(match run? {
                Ok(value) if scope.strict_equals(receiver, &value)? => {
                    DomSourceCustomElementOutcome::Returned(node)
                }
                Ok(_) => DomSourceCustomElementOutcome::InvalidInterface,
                Err(Error::Exception(exception)) => {
                    pending.exception = Some(exception);
                    DomSourceCustomElementOutcome::Failed
                }
                Err(error) => return Err(error.into()),
            });
            if pending.outcome == Some(DomSourceCustomElementOutcome::Returned(node)) {
                pending.upgrade_callbacks = callbacks;
            }
            self.finish_customized_constructor(scope, owner, events, &mut pending)?;
            Ok(pending.upgrade_callbacks.take())
        })();
        if result.is_err() {
            owner.keep_constructor_result(pending);
        }
        result
    }

    fn open_customized_callback<'scope, 'registration>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        public: &PublicEventBindings,
        request: DomSourceCustomElementRequest,
        registration: &'registration DomDocumentScriptRegistration,
    ) -> Result<
        (
            surgeist_dom::DomSourceCustomElementCallbackScope<'registration>,
            (NodeId, Value<'scope>),
        ),
        DocumentGuestEventError,
    > {
        events.budget.phase(|| {
            let admission = Admission::ordinary(owner, events);
            let mut pins = self
                .pins
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let (key, reserved) = pins.reserve(admission)?;
            let inbox = self
                .inbox
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let pin = PreparedPin::new(admission)?;
            let (payload, holder) =
                scope.retained_native_object_bytes::<GuestNodeProjection, GuestBacking>()?;
            let backing = (admission.reserve(payload)?, admission.reserve(holder)?);
            let root_funding = scope.prepare_native_projection_root()?;
            let open = |elements: &element::ElementBindings| {
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let mut dom = session.dom();
                let mut lease = dom.raw_mut().resume_source_edit(
                    request.operation(),
                    events.limits.source_edits,
                    &owner.cancellation,
                )?;
                lease.open_customized_element_callback_with_watch(
                    request,
                    registration,
                    (&inbox, key),
                    events.limits.node_retirement,
                    &owner.cancellation,
                    |name| {
                        let prototype = elements.prototype_for_name(name, || {
                            events.budget.charge_work(AccountScope::Projection, 1)
                        })?;
                        let projection = scope.prepare_retained_native_object_funded(
                            NodePrototype::Creation(prototype).projection(),
                            pin.projection(),
                            |_, _| Ok::<_, DocumentGuestEventError>(backing),
                            root_funding,
                        )?;
                        Ok(projection)
                    },
                )
            };
            let (callback, watched, projection) =
                if public.custom_elements.is_global(request.global()) {
                    open(&public.document.element)
                } else {
                    public.with_origin(events, request.global(), |origin| open(&origin.elements))
                }?;
            let (_, node, watch) = watched.into_parts();
            let (value, holder, live) = projection
                .publish(|payload| Self::bind_projection(payload, GuestNode::Element(node), watch));
            pin.publish(reserved, holder, live);
            Ok((callback, (node, value)))
        })
    }

    fn finish_customized_constructor<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        pending: &mut PendingConstructor,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let receiver = pending
            .receiver
            .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
        let value = scope.import(
            pending
                .returned
                .as_ref()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?,
        )?;
        if pending.exception.is_none()
            && pending.outcome.is_some_and(|outcome| {
                outcome != DomSourceCustomElementOutcome::Returned(receiver)
                    && outcome != DomSourceCustomElementOutcome::Failed
            })
        {
            let error = scope.type_error(&ascii(b"Constructor did not return its receiver"))?;
            pending.exception = Some(scope.invocation_exception(&error));
        }
        if pending.exception.is_some() && pending.report.needs_preparation() {
            pending.report.prepare()?;
        }
        {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let mut dom = session.dom();
            let cancel = &owner.cancellation;
            let mut lease = dom.raw_mut().resume_source_edit(
                pending.request.operation(),
                events.limits.source_edits,
                cancel,
            )?;
            lease.accept_custom_element_callback(
                pending.request,
                &pending.registration,
                &mut pending.outcome,
                cancel,
            )?;
            if !matches!(
                lease.advance(cancel)?,
                surgeist_dom::DomSourceProgress::Complete(_)
            ) {
                return Err(DocumentScriptNativeError::SourceAccess.into());
            }
        }
        if let Some(exception) = pending.exception.take() {
            pending.report.publish(DocumentGuestConstructorException {
                definition: pending.request.definition(),
                realm: pending.realm,
                exception,
            });
        }
        Ok(value)
    }

    fn finish_constructor<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        public: &PublicEventBindings,
        pending: &mut PendingConstructor,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let cancel = &owner.cancellation;
        let returned = pending
            .returned
            .as_ref()
            .map(|root| scope.import(root))
            .transpose()?;
        let disposition = {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let mut dom = session.dom();
            let mut lease = dom.raw_mut().resume_source_edit(
                pending.request.operation(),
                events.limits.source_edits,
                cancel,
            )?;
            if pending.outcome.is_some() {
                lease.accept_custom_element_callback(
                    pending.request,
                    &pending.registration,
                    &mut pending.outcome,
                    cancel,
                )?;
            }
            match lease.finish_returned_custom_element(pending.request, cancel)? {
                DomSourceCustomElementReturn::Constructed(_) => {
                    return returned.ok_or(DocumentScriptNativeError::ProjectionCustody.into());
                }
                DomSourceCustomElementReturn::Fallback(disposition) => disposition,
            }
        };
        let admission = Admission::ordinary(owner, events);
        admission.phase(|| {
            let mut pins = self
                .pins
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let (key, reserved) = pins.reserve(admission)?;
            let inbox = self
                .inbox
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let pin = PreparedPin::new(admission)?;
            let (payload, holder) =
                scope.retained_native_object_bytes::<GuestNodeProjection, GuestBacking>()?;
            let backing = (admission.reserve(payload)?, admission.reserve(holder)?);
            if pending.exception.is_some() && pending.report.needs_preparation() {
                pending.report.prepare()?;
            }
            let prototype = public.unknown_element_prototype(events, pending.request.global())?;
            if pending.exception.is_none()
                && disposition != surgeist_dom::DomSourceCustomElementDisposition::Abrupt
            {
                let value = match disposition {
                    surgeist_dom::DomSourceCustomElementDisposition::InvalidInterface => {
                        scope.type_error(&ascii(b"HTMLElement required"))?
                    }
                    surgeist_dom::DomSourceCustomElementDisposition::InvalidElement => scope
                        .dom_exception(
                        surgeist_bindings_js::dom_exception::DomExceptionName::NotSupportedError,
                        &[],
                    )?,
                    _ => return Err(DocumentScriptNativeError::ProjectionCustody.into()),
                };
                let exception = scope.invocation_exception(&value);
                pending.report.prepare()?;
                pending.exception = Some(exception);
            }
            let root_funding = scope.prepare_native_projection_root()?;
            let result = {
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let mut dom = session.dom();
                let mut lease = dom.raw_mut().resume_source_edit(
                    pending.request.operation(),
                    events.limits.source_edits,
                    cancel,
                )?;
                lease.finish_custom_element_with_watch(
                    pending.request,
                    (&inbox, key),
                    events.limits.node_retirement,
                    cancel,
                    |name| {
                        name.step()?;
                        if name.custom_element_disposition() != Some(disposition) {
                            return Err(DocumentScriptNativeError::ProjectionCustody.into());
                        }
                        #[cfg(test)]
                        if self.refuse_next_constructor_fallback.replace(false) {
                            return Err(DocumentScriptNativeError::ProjectionCustody.into());
                        }
                        scope.prepare_retained_native_object_funded(
                            NodePrototype::Creation(&prototype).projection(),
                            pin.projection(),
                            |_, _| Ok::<_, DocumentGuestEventError>(backing),
                            root_funding,
                        )
                    },
                )?
            };
            match result {
                DomWatchedCustomElementResult::Returned(_) => {
                    returned.ok_or(DocumentScriptNativeError::ProjectionCustody.into())
                }
                DomWatchedCustomElementResult::Fallback {
                    node,
                    watch,
                    projection,
                    disposition: _,
                } => {
                    let (value, holder, live) = projection.publish(|payload| {
                        Self::bind_projection(payload, GuestNode::Element(node), watch)
                    });
                    pin.publish(reserved, holder, live);
                    if let Some(exception) = pending.exception.take() {
                        pending.report.publish(DocumentGuestConstructorException {
                            definition: pending.request.definition(),
                            realm: pending.realm,
                            exception,
                        });
                    }
                    Ok(value)
                }
            }
        })
    }

    pub(super) fn create_element<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        capture: &Capture,
        (owner, events): (&NativeOwner, &GuestEvents),
        public: &PublicEventBindings,
        (document, name, options): (
            DocumentId,
            DomElementCreationName,
            DomElementCreationOptions,
        ),
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let event = owner.active_event_request();
        let attribute = owner.active_attribute_registration()?;
        let active = owner.current_constructor_registration()?;
        let admission = match (&active, &attribute, &event) {
            (Some(active), _, _) => Admission::constructor(owner, events, active),
            (None, Some((request, registration)), _) => {
                Admission::attribute(owner, events, *request, registration)
            }
            (None, None, Some(request)) => Admission::source(owner, events, request),
            (None, None, None) => Admission::ordinary(owner, events),
        };
        let project = |elements: &element::ElementBindings| {
            self.project_element(
                scope,
                capture,
                (admission, event.as_ref()),
                elements,
                (document, name, options),
            )
        };
        let result = if let Some(global) = public.other_document_origin(owner, events, document)? {
            public.with_origin_with(
                global,
                || admission.step(),
                |origin| project(&origin.elements),
            )
        } else {
            project(&public.document.element)
        }?;
        match result {
            ElementFactoryResult::Created(value) => Ok(value),
            ElementFactoryResult::Constructor(request) => {
                let (value, reaction_queue) =
                    self.run_constructor(scope, owner, events, public, request)?;
                if let Some(queue) = reaction_queue {
                    capture.drain_routed_reactions(scope, owner, events, Some(queue))?;
                }
                Ok(value)
            }
        }
    }

    pub(super) fn construct_autonomous<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        frame: &super::super::custom_elements::ActiveConstructor,
        global: DomHostedGlobalId,
        prototype: &Value<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let active = owner
            .current_constructor_registration()?
            .ok_or(DocumentScriptNativeError::SourceAccess)?;
        let admission = Admission::constructor(owner, events, &active);
        self.project(
            scope,
            admission,
            (NodePrototype::Selected(prototype), |inbox, key| {
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let watched = session
                    .dom()
                    .raw_mut()
                    .construct_autonomous_element_with_watch(
                        (active.0, frame.request),
                        &active.1,
                        global,
                        (inbox, key),
                        events.limits.node_retirement,
                        &owner.cancellation,
                    )?;
                let (_, node, watch) = watched.into_parts();
                Ok((GuestNode::Element(node), watch))
            }),
        )
    }

    pub(super) fn construct_direct<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        target: (surgeist_dom::DomRegistryDefinitionId, DomHostedGlobalId),
        prototype: &Value<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let registration = owner
            .attachment
            .get()
            .and_then(|attachment| attachment.registration.upgrade())
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let constructor = owner.current_constructor_registration()?;
        let attribute = owner.active_attribute_registration()?;
        let event = owner.active_event_request();
        let admission = match (&constructor, &attribute, &event) {
            (Some(request), _, _) => Admission::constructor(owner, events, request),
            (_, Some((request, registration)), _) => {
                Admission::attribute(owner, events, *request, registration)
            }
            (_, _, Some(request)) => Admission::source(owner, events, request),
            _ => Admission::ordinary(owner, events),
        };
        self.project(
            scope,
            admission,
            (NodePrototype::Selected(prototype), |inbox, key| {
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let watched = session
                    .dom()
                    .raw_mut()
                    .construct_registered_element_with_watch(
                        target,
                        &registration,
                        (inbox, key, events.limits.node_retirement),
                        events.limits.source_edits,
                        &owner.cancellation,
                    )?;
                let (_, node, watch) = watched.into_parts();
                Ok((GuestNode::Element(node), watch))
            }),
        )
    }

    fn project_element<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        capture: &Capture,
        (admission, event): (Admission<'_>, Option<&DomSourceEventRequest>),
        elements: &element::ElementBindings,
        (document, name, options): (
            DocumentId,
            DomElementCreationName,
            DomElementCreationOptions,
        ),
    ) -> Result<ElementFactoryResult<'scope>, DocumentGuestEventError> {
        admission.phase(|| {
            let Admission { owner, events, .. } = admission;
            let mut pins = self
                .pins
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let (key, reserved) = pins.reserve(admission)?;
            let inbox = self
                .inbox
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let pin = PreparedPin::new(admission)?;
            let payload = pin.projection();
            let (payload_bytes, holder_bytes) =
                scope.retained_native_object_bytes::<GuestNodeProjection, GuestBacking>()?;
            let backing = (
                admission.reserve(payload_bytes)?,
                admission.reserve(holder_bytes)?,
            );
            admission.step()?;
            let root_funding = scope.prepare_native_projection_root()?;
            let result = capture
                .callbacks
                .with_qualified_session(owner, event, |session, _| {
                    let mut dom = session.dom();
                    let dom = dom.raw_mut();
                    dom.create_detached_element_with_node_retirement_watch(
                        document,
                        (name, options, Some(creation_realm_fallback(owner)?)),
                        (&inbox, key),
                        (events.limits.source_edits, events.limits.node_retirement),
                        &owner.cancellation,
                        |name| {
                            let step = || {
                                name.step()?;
                                events.budget.charge_work(AccountScope::Projection, 1)
                            };
                            let prototype = elements.prototype_for_name(name.name(), step)?;
                            let prepared = scope.prepare_retained_native_object_funded(
                                NodePrototype::Creation(prototype).projection(),
                                payload,
                                |_, _| Ok::<_, DocumentGuestEventError>(backing),
                                root_funding,
                            )?;
                            #[cfg(test)]
                            self.qualify_projection()?;
                            Ok::<_, DocumentGuestEventError>(prepared)
                        },
                    )
                })?;
            let (watched, prepared) = match result {
                surgeist_dom::DomWatchedElementCreation::Created(watched, prepared) => {
                    (watched, prepared)
                }
                surgeist_dom::DomWatchedElementCreation::Constructor(request) => {
                    return Ok(ElementFactoryResult::Constructor(request));
                }
            };
            let (_, node, watch) = watched.into_parts();
            let (value, holder, live) = prepared
                .publish(|payload| Self::bind_projection(payload, GuestNode::Element(node), watch));
            pin.publish(reserved, holder, live);
            Ok(ElementFactoryResult::Created(value))
        })
    }

    pub(super) fn new(inbox: DomNodeRetirementInbox) -> Self {
        Self {
            inbox: RefCell::new(inbox),
            pins: RefCell::new(OrdinaryPins::default()),
            #[cfg(test)]
            refuse_next_validation: Cell::new(false),
            #[cfg(test)]
            refuse_next_constructor_fallback: Cell::new(false),
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
            (prototype, create),
        )
    }

    pub(super) fn create_detached<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        capture: &Capture,
        (owner, events): (&NativeOwner, &GuestEvents),
        prototype: NodePrototype<'_, 'scope>,
        (document, creation): (DocumentId, DomDetachedNodeCreation),
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let event = owner.active_event_request();
        let attribute = owner.active_attribute_registration()?;
        let active = owner.current_constructor_registration()?;
        let admission = match (&active, &attribute, &event) {
            (Some(active), _, _) => Admission::constructor(owner, events, active),
            (None, Some((request, registration)), _) => {
                Admission::attribute(owner, events, *request, registration)
            }
            (None, None, Some(request)) => Admission::source(owner, events, request),
            (None, None, None) => Admission::ordinary(owner, events),
        };
        let publish = |inbox: &DomNodeRetirementInbox, key| {
            capture
                .callbacks
                .with_qualified_session(owner, event.as_ref(), |session, _| {
                    let project = match &creation {
                        DomDetachedNodeCreation::Attribute(_)
                        | DomDetachedNodeCreation::AttributeNs(..) => GuestNode::Attr,
                        DomDetachedNodeCreation::Fragment => GuestNode::Fragment,
                        DomDetachedNodeCreation::Text(_) => GuestNode::Text,
                        DomDetachedNodeCreation::Comment(_) => GuestNode::Comment,
                        DomDetachedNodeCreation::ProcessingInstruction { .. } => {
                            GuestNode::ProcessingInstruction
                        }
                        DomDetachedNodeCreation::DocumentType { .. } => GuestNode::DocumentType,
                    };
                    let mut dom = session.dom();
                    let dom = dom.raw_mut();
                    let cancel = &owner.cancellation;
                    let watched = if matches!(admission.route, AdmissionRoute::Ordinary) {
                        let mut batch = DomBatch::new();
                        let ticket = batch.create_detached(document, creation);
                        dom.apply_with_node_retirement_watch(
                            batch,
                            &ticket,
                            inbox,
                            key,
                            events.limits.node_retirement,
                            cancel,
                        )?
                    } else {
                        dom.create_detached_with_node_retirement_watch(
                            document,
                            creation,
                            (inbox, key),
                            (events.limits.source_edits, events.limits.node_retirement),
                            cancel,
                        )?
                    };
                    let (_, node, watch) = watched.into_parts();
                    Ok((project(node), watch))
                })
        };
        self.project(scope, admission, (prototype, publish))
    }

    pub(super) fn project_source_node<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        prototype: &RetainedCallbackObject,
        run: impl FnOnce(
            &mut surgeist_dom::DomSourceNodeOutputWatch<'_>,
        ) -> Result<GuestNode, DocumentGuestEventError>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let event = owner.active_event_request();
        let attribute = owner.active_attribute_registration()?;
        let active = owner.current_constructor_registration()?;
        let admission = match (&active, &attribute, &event) {
            (Some(active), _, _) => Admission::constructor(owner, events, active),
            (None, Some((request, registration)), _) => {
                Admission::attribute(owner, events, *request, registration)
            }
            (None, None, Some(request)) => Admission::source(owner, events, request),
            (None, None, None) => Admission::ordinary(owner, events),
        };
        self.project(
            scope,
            admission,
            (
                NodePrototype::Creation(prototype),
                |inbox: &DomNodeRetirementInbox, key| {
                    let mut output = surgeist_dom::DomSourceNodeOutputWatch::new(
                        inbox,
                        key,
                        events.limits.node_retirement,
                    );
                    let node = run(&mut output)?;
                    let (_, watch) = output
                        .into_watch()
                        .filter(|(watched, _)| *watched == node.node())
                        .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                    Ok((node, watch))
                },
            ),
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
            (
                NodePrototype::Creation(prototype),
                |inbox: &DomNodeRetirementInbox, key| {
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
                },
            ),
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
            (
                NodePrototype::Creation(prototype),
                |inbox: &DomNodeRetirementInbox, key| {
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
                },
            ),
        )
    }

    pub(super) fn watch_registered_callback<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        admission: Admission<'_>,
        node: GuestNode,
        prototype: &RetainedCallbackObject,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        self.project(
            scope,
            admission,
            (
                NodePrototype::Creation(prototype),
                |inbox: &DomNodeRetirementInbox, key| {
                    let mut session = admission
                        .owner
                        .session
                        .try_borrow_mut()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    let mut dom = session.dom();
                    let dom = dom.raw_mut();
                    let cancel = &admission.owner.cancellation;
                    let watch = match admission.route {
                        AdmissionRoute::Attribute(request, registration) => dom
                            .watch_source_attribute_node_retirement(
                                request,
                                registration,
                                node.node(),
                                inbox,
                                key,
                                cancel,
                            )?,
                        AdmissionRoute::Constructor(request, registration) => dom
                            .watch_source_custom_element_node_retirement(
                                request,
                                registration,
                                node.node(),
                                (inbox, key),
                                cancel,
                            )?,
                        _ => return Err(DocumentScriptNativeError::SourceAccess.into()),
                    };
                    Ok((node, watch))
                },
            ),
        )
    }

    pub(super) fn project_attachment_output<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        admission: Admission<'_>,
        prototype: &RetainedCallbackObject,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let AdmissionRoute::AttachmentOutput(request, registration) = admission.route else {
            return Err(DocumentScriptNativeError::SourceAccess.into());
        };
        self.project(
            scope,
            admission,
            (
                NodePrototype::Creation(prototype),
                |inbox: &DomNodeRetirementInbox, key| {
                    let mut session = admission
                        .owner
                        .session
                        .try_borrow_mut()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    let watch = session
                        .dom()
                        .raw_mut()
                        .watch_source_attribute_attachment_output(
                            request,
                            registration,
                            (inbox, key),
                            admission.events.limits.node_retirement,
                            &admission.owner.cancellation,
                        )?;
                    Ok((GuestNode::Attr(request.replaced().node()), watch))
                },
            ),
        )
    }

    fn project<'scope, Publish>(
        &self,
        scope: &mut Scope<'scope>,
        admission: Admission<'_>,
        (prototype, create): (NodePrototype<'_, 'scope>, Publish),
    ) -> Result<Value<'scope>, DocumentGuestEventError>
    where
        Publish: FnOnce(
            &DomNodeRetirementInbox,
            usize,
        )
            -> Result<(GuestNode, DomNodeRetirementWatch), DocumentGuestEventError>,
    {
        admission.phase(|| {
            let mut pins = self
                .pins
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let (key, reserved) = pins.reserve(admission)?;
            let inbox = self
                .inbox
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let pin = PreparedPin::new(admission)?;
            admission.step()?;
            let (value, holder, live) = scope.try_project_retained_native_object(
                prototype.projection(),
                pin.projection(),
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
                    admission.accept_output(&watch)?;
                    Ok(Self::bind_projection(payload, node, watch))
                },
            )?;
            pin.publish(reserved, holder, live);
            Ok(value)
        })
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

    pub(super) fn open_with<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        node: NodeId,
        admission: Admission<'_>,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        admission.phase(|| self.open_qualified(scope, node, || admission.step(), |_| Ok(())))
    }

    pub(super) fn open_attachment_output<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        node: NodeId,
        admission: Admission<'_>,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        admission.phase(|| {
            self.open_qualified(
                scope,
                node,
                || admission.step(),
                |watch| admission.accept_output(watch),
            )
        })
    }

    fn open_qualified<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        node: NodeId,
        mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
        accept: impl FnOnce(&DomNodeRetirementWatch) -> Result<(), DocumentGuestEventError>,
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
                let value = open_node_pin(scope, &pin.holder)?;
                accept(&pin.live._watch)?;
                return Ok(Some(value));
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
