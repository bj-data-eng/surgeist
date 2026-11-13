//! Retained custody indexed by exact native node retirement.

use super::super::PublicEventBindings;
use super::*;
use surgeist_bindings_js::NativeProjectionPrototype;
use surgeist_dom::{
    DomDetachedNodeCreation, DomDocumentScriptRegistration, DomElementCreationName,
    DomElementCreationOptions, DomNodeRetirementWatch, DomSourceAttributeAttachmentOutput,
    DomSourceAttributeVerificationRequest, DomSourceCustomElementOutcome,
    DomSourceCustomElementRequest, DomSourceEventRequest, DomWatchedCustomElementResult,
};

pub(super) struct OrdinaryBindings {
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
    realm: RealmId,
    report: super::super::super::reports::ConstructorReportReservation,
    exception: Option<surgeist_bindings_js::Exception>,
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

    pub(in crate::document_scripting) fn settle_stopped_constructor(
        &self,
        limits: surgeist_dom::DomSourceLimits,
    ) -> Result<(), DocumentGuestEventError> {
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

    pub(super) fn step(self) -> Result<(), DocumentGuestEventError> {
        self.source_step()?;
        self.events.step()
    }

    pub(super) fn reserve(self, bytes: usize) -> Result<GuestBacking, DocumentGuestEventError> {
        self.source_step()?;
        self.events.reserve(self.owner, bytes)
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
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let cancel = &owner.cancellation;
        let (constructor, registration, entry, backing, realm, report, returned_backing) = (|| {
            let (constructor, realm) = public.factory_constructor(scope, events, request)?;
            let registration = owner
                .attachment
                .get()
                .and_then(|attachment| attachment.registration.upgrade())
                .ok_or(DocumentScriptNativeError::Revoked)?;
            let backing = events.reserve(owner, size_of::<PendingConstructor>())?;
            let report = events.reports.reserve_constructor(events, owner)?;
            let returned_bytes = events
                .exception_root_bytes
                .checked_add(size_of::<GuestBacking>())
                .ok_or(DocumentScriptNativeError::StorageLimit)?;
            let returned_backing = events.reserve(owner, returned_bytes)?;
            let entry = public.enter_factory_constructor(
                owner,
                events,
                scope,
                request,
                &constructor,
                registration.clone(),
            )?;
            Ok::<_, DocumentGuestEventError>((
                constructor,
                registration,
                entry,
                backing,
                realm,
                report,
                Box::new(returned_backing),
            ))
        })(
        )
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
            realm,
            report,
            exception: None,
            next: None,
            _backing: backing,
        });
        let callback_scope = (|| {
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
                .map_err(DocumentGuestEventError::from)
        })();
        let callback_scope = match callback_scope {
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
        let result = scope.construct_without_arguments(&constructor);
        drop(callback_scope);
        drop(entry);
        let result = match result {
            Ok(value) => {
                if let Err(error) = scope.retain_invocation_result_backing(&value, returned_backing)
                {
                    pending.outcome = Some(DomSourceCustomElementOutcome::Failed);
                    owner.keep_constructor_result(pending);
                    return Err(error.into());
                }
                pending.returned = Some(scope.retain(&value));
                scope
                    .native_object::<GuestNodeProjection>(&value)
                    .map(|projection| projection.and_then(|native| native.payload().id()))
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
        match self.finish_constructor(scope, (owner, events), public, &mut pending) {
            Ok(value) => Ok(value),
            Err(error) => {
                owner.keep_constructor_result(pending);
                Err(error)
            }
        }
    }

    fn finish_constructor<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        public: &PublicEventBindings,
        pending: &mut PendingConstructor,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
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
        let returned = pending
            .returned
            .as_ref()
            .map(|root| scope.import(root))
            .transpose()?;
        if let Some(exception) = &pending.exception
            && pending.report.needs_backing()
        {
            pending.report.prepare(scope, exception)?;
        }
        let prototype = public.unknown_element_prototype(events, pending.request.global())?;
        let cancel = &owner.cancellation;
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
            if pending.outcome.is_some() {
                lease.accept_custom_element_callback(
                    pending.request,
                    &pending.registration,
                    &mut pending.outcome,
                    cancel,
                )?;
            }
            lease.finish_custom_element_with_watch(
                pending.request,
                (&inbox, key),
                events.limits.node_retirement,
                cancel,
                |name| {
                    name.step()?;
                    if pending.exception.is_none()
                        && name.custom_element_disposition() != Some(surgeist_dom::DomSourceCustomElementDisposition::Abrupt)
                    {
                        let value = match name.custom_element_disposition() {
                            Some(surgeist_dom::DomSourceCustomElementDisposition::InvalidInterface) => scope.type_error(&ascii(b"HTMLElement required"))?,
                            Some(surgeist_dom::DomSourceCustomElementDisposition::InvalidElement) => scope.dom_exception(
                                surgeist_bindings_js::dom_exception::DomExceptionName::NotSupportedError, &[],
                            )?,
                            _ => return Err(DocumentScriptNativeError::ProjectionCustody.into()),
                        };
                        let exception = scope.invocation_exception(&value);
                        pending.report.prepare(scope, &exception)?;
                        pending.exception = Some(exception);
                    }
                    #[cfg(test)]
                    if self.refuse_next_constructor_fallback.replace(false) {
                        return Err(DocumentScriptNativeError::ProjectionCustody.into());
                    }
                    scope.prepare_retained_native_object(
                        NodePrototype::Creation(&prototype).projection(),
                        pin.projection(),
                        |_, _| Ok::<_, DocumentGuestEventError>(backing),
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
                self.run_constructor(scope, owner, events, public, request)
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
        let result = capture
            .callbacks
            .with_qualified_session(owner, event, |session, _| {
                let mut dom = session.dom();
                let dom = dom.raw_mut();
                dom.create_detached_element_with_node_retirement_watch(
                    document,
                    (name, options),
                    (&inbox, key),
                    (events.limits.source_edits, events.limits.node_retirement),
                    &owner.cancellation,
                    |name| {
                        let step = || {
                            name.step()?;
                            events.step()
                        };
                        let prototype = elements.prototype_for_name(name.name(), step)?;
                        let prepared = scope.prepare_retained_native_object(
                            NodePrototype::Creation(prototype).projection(),
                            payload,
                            |_, _| Ok::<_, DocumentGuestEventError>(backing),
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
                    };
                    let mut dom = session.dom();
                    let dom = dom.raw_mut();
                    let cancel = &owner.cancellation;
                    let watched = if matches!(admission.route, AdmissionRoute::Ordinary) {
                        let mut batch = DomBatch::new();
                        let ticket = match creation {
                            DomDetachedNodeCreation::Attribute(name) => {
                                batch.create_attribute(document, name)
                            }
                            DomDetachedNodeCreation::AttributeNs(namespace, name) => {
                                batch.create_attribute_ns(document, namespace, name)
                            }
                            DomDetachedNodeCreation::Fragment => batch.create_fragment(document),
                            DomDetachedNodeCreation::Text(data) => {
                                batch.create_text(document, data)
                            }
                            DomDetachedNodeCreation::Comment(data) => {
                                batch.create_comment(document, data)
                            }
                        };
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
        mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        self.open_qualified(scope, node, &mut step, |_| Ok(()))
    }

    pub(super) fn open_attachment_output<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        node: NodeId,
        admission: Admission<'_>,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        self.open_qualified(
            scope,
            node,
            || admission.step(),
            |watch| admission.accept_output(watch),
        )
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
                let value = scope.open_callback_object(&pin.holder)?;
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
