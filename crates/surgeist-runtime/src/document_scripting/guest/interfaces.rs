//! Public native interfaces on the authenticated, unpublished Window.

mod custom_elements;
mod document;
#[cfg(test)]
pub(in crate::document_scripting) use document::missing_pins_refuse_without_native_publication;
mod listener_methods;
mod operations;
mod security_policy_violation;
mod toggle_event;
mod trusted_policy;

use super::*;
pub(in crate::document_scripting) use custom_elements::document_registry as creation_document_registry;
#[cfg(test)]
pub(in crate::document_scripting) use custom_elements::document_registry_identity;
pub(in crate::document_scripting) use custom_elements::{ActiveConstructor, ActiveReactionQueue};
pub(in crate::document_scripting) use document::PendingConstructor;
pub(super) use document::{GuestNodeProjection, NodeLifetime};
use surgeist_bindings_js::{
    Arguments, ConstructorArguments, ConstructorCallback, ConstructorCompletion, FunctionMetadata,
    WellKnownSymbol,
    conversion::{
        ArgumentValue, ConversionAccount, Hosted, ResourceTable, WorkTable,
        declarations::{event_constructor_0_event_init_dict, event_constructor_0_type},
    },
};
use surgeist_dom::DocumentId;

const fn ascii<const N: usize>(bytes: &[u8; N]) -> [u16; N] {
    let mut units = [0; N];
    let mut source: &[u8] = bytes;
    let mut target: &mut [u16] = &mut units;
    while let ([byte, rest @ ..], [unit, pending @ ..]) = (source, target) {
        *unit = *byte as u16;
        source = rest;
        target = pending;
    }
    units
}

const PROTOTYPE: &[u16] = &ascii(b"prototype");
const CONSTRUCTOR: &[u16] = &ascii(b"constructor");
const TRUSTED: &[u16] = &ascii(b"isTrusted");
const EVENT: &[u16] = &ascii(b"Event");
const TARGET: &[u16] = &ascii(b"EventTarget");
pub(super) const EVENT_TARGET_REFERENCE: u32 = 0;

#[derive(Clone, Copy)]
enum Interface {
    Event,
    Target,
}

impl Interface {
    fn declaration(self) -> (&'static [u16], u32, u32) {
        match self {
            Self::Event => (EVENT, 1, 2),
            Self::Target => (TARGET, 0, 0),
        }
    }
}

#[derive(Clone, Copy)]
enum Attribute {
    Type,
    Target,
    CurrentTarget,
    CancelBubble,
    ReturnValue,
    Bubbles,
    Cancelable,
    Composed,
    DefaultPrevented,
    Phase,
    TimeStamp,
    Trusted,
}

struct GuestAccess {
    events: Weak<GuestEvents>,
    // The weak pointer still retains the allocation's target backing.
    _backing: Rc<GuestFixedBacking>,
}

pub(super) struct Callbacks {
    owner: Weak<NativeOwner>,
    permit: DocumentScriptPermit,
    access: OnceCell<GuestAccess>,
    conversion: ConversionAccount,
    _backing: GuestBacking,
}

struct Capture {
    callbacks: Rc<Callbacks>,
    origin: Option<Rc<OriginAuthority>>,
    _backing: GuestBacking,
}

struct OriginAuthority {
    global: DomHostedGlobalId,
    realm: RealmId,
    _backing: GuestBacking,
}

impl OriginAuthority {
    const STORAGE_BYTES: usize = size_of::<Self>() + 2 * size_of::<usize>();
}

struct OriginBindings {
    global: DomHostedGlobalId,
    target: RetainedCallbackObject,
    documents: document::DocumentPrototypes,
    elements: document::element::ElementBindings,
    custom_elements: Rc<custom_elements::Registry>,
    next: Option<Box<Self>>,
    _backing: GuestBacking,
}

#[derive(Clone, Copy, Default)]
struct OriginCost {
    bytes: usize,
    requests: u64,
}

impl OriginCost {
    fn include(self, bytes: usize) -> Result<Self, DocumentScriptNativeError> {
        Ok(Self {
            bytes: self
                .bytes
                .checked_add(bytes)
                .ok_or(DocumentScriptNativeError::StorageLimit)?,
            requests: self
                .requests
                .checked_add(1)
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?,
        })
    }
}

type CaptureFunding<'a> = &'a dyn Fn(usize) -> Result<Capture, DocumentGuestEventError>;

fn constructor_capture_bytes(action_bytes: usize) -> Result<usize, DocumentScriptNativeError> {
    action_bytes
        .checked_add(ConstructorCallback::behavior_storage_bytes())
        .ok_or(DocumentScriptNativeError::StorageLimit)
}

/// One action capture also keeps its constructor's shared behavior body funded.
fn constructor_capture(
    capture: CaptureFunding<'_>,
    action_bytes: usize,
) -> Result<Capture, DocumentGuestEventError> {
    capture(constructor_capture_bytes(action_bytes)?)
}

struct OriginInterfaceFunding<'a> {
    capture: CaptureFunding<'a>,
    retain: &'a dyn Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
}

pub(super) struct PublicEventBindings {
    callbacks: Rc<Callbacks>,
    window: RetainedCallbackObject,
    trusted_types: RetainedCallbackObject,
    event: RetainedCallbackObject,
    toggle_event: RetainedCallbackObject,
    security_policy_violation: RetainedCallbackObject,
    target: RetainedCallbackObject,
    trusted: RetainedCallbackObject,
    dispatch_in_attachment: RetainedCallbackObject,
    backup_reaction_job: RetainedCallbackObject,
    backup_reactions_scheduled: Cell<bool>,
    document: document::DocumentBindings,
    custom_elements: Rc<custom_elements::Registry>,
    scoped_registries: RefCell<custom_elements::ScopedRegistries>,
    origins: RefCell<Option<Box<OriginBindings>>>,
    origin_cost: OriginCost,
}

impl Drop for PublicEventBindings {
    fn drop(&mut self) {
        let origins = self.origins.get_mut();
        while let Some(mut origin) = origins.take() {
            *origins = origin.next.take();
        }
    }
}

impl PublicEventBindings {
    pub(super) fn reap_nodes(&self, process: &mut Process, retired: &Cell<bool>) {
        self.document.reap_nodes(process, retired);
    }

    pub(super) fn install(
        script: &mut UnpublishedScriptEngine,
        owner: &Rc<NativeOwner>,
        admission: &GuestEventAdmission,
    ) -> Result<Self, DocumentGuestEventError> {
        let permit = DocumentScriptPermit {
            realm: script.authority.identity(),
            document: script.target.document(),
        };
        let (inbox, registry) = {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let snapshot = session.snapshot();
            let registry = custom_elements::document_registry(&snapshot, permit.document, true)?;
            let inbox = session
                .dom()
                .raw()
                .node_retirement_inbox(admission.limits.node_retirement, &owner.cancellation)?;
            (inbox, registry)
        };
        let global = script.target.global();
        let conversion_host = script
            .preparation
            .conversion_host(ConversionOrigin::General);
        script
            .process
            .execution()
            .in_realm(&script.realm, |scope| {
                Ok(Self::install_in_realm(
                    scope,
                    owner,
                    permit,
                    admission,
                    (inbox, registry, global),
                    conversion_host,
                ))
            })?
    }

    fn install_in_realm(
        scope: &mut Scope<'_>,
        owner: &Rc<NativeOwner>,
        permit: DocumentScriptPermit,
        admission: &GuestEventAdmission,
        (inbox, registry, global): (
            surgeist_dom::DomNodeRetirementInbox,
            surgeist_dom::DomRegistryId,
            DomHostedGlobalId,
        ),
        conversion_host: crate::document_scripting::preparation::NativeConversionHost,
    ) -> Result<Self, DocumentGuestEventError> {
        let fund = |bytes| {
            admission
                .budget
                .reserve(AccountScope::Adapter, owner, bytes)
        };
        let dom_exception_metadata_bytes =
            Hosted::new(conversion_host.for_origin(ConversionOrigin::DomException))
                .storage_bytes::<GuestBacking>()
                .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let hosted = Hosted::new(conversion_host);
        let metadata_bytes = hosted
            .storage_bytes::<GuestBacking>()
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let account_bytes = size_of::<Callbacks>()
            .checked_add(metadata_bytes)
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let mut backing = fund(account_bytes)?;
        let metadata = backing.split_off(metadata_bytes)?;
        let work = admission.limits.work;
        let buffer_bytes = u64::try_from(admission.limits.storage_bytes)
            .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
        let conversion = hosted.install(
            scope,
            surgeist_ledger::Limits::new(
                WorkTable::splat(work),
                ResourceTable {
                    BufferBytes: buffer_bytes,
                },
            ),
            metadata,
        )?;
        let document_id = permit.document;
        let callbacks = Rc::new(Callbacks {
            owner: Rc::downgrade(owner),
            permit,
            access: OnceCell::new(),
            conversion,
            _backing: backing,
        });
        let capture = |bytes| -> Result<Capture, DocumentGuestEventError> {
            Ok(Capture {
                callbacks: callbacks.clone(),
                origin: None,
                _backing: fund(bytes)?,
            })
        };
        // Learn the size of one origin graph from the same installers in
        // the receiving realm. Their requested Rust extents are independent
        // of the realm in which the V8 objects are created.
        let origin_cost = Cell::new(OriginCost::default());
        let fund_origin = |bytes| -> Result<GuestBacking, DocumentGuestEventError> {
            let backing = fund(bytes)?;
            origin_cost.set(origin_cost.get().include(bytes)?);
            Ok(backing)
        };
        let capture_origin = |bytes| -> Result<Capture, DocumentGuestEventError> {
            Ok(Capture {
                callbacks: callbacks.clone(),
                origin: None,
                _backing: fund_origin(bytes)?,
            })
        };

        let event = scope.object(ObjectPrototype::Default)?;
        for (attribute, key, name) in [
            (
                Attribute::Type,
                &ascii(b"type")[..],
                &ascii(b"get type")[..],
            ),
            (Attribute::Target, &ascii(b"target"), &ascii(b"get target")),
            (
                Attribute::Target,
                &ascii(b"srcElement"),
                &ascii(b"get srcElement"),
            ),
            (
                Attribute::CurrentTarget,
                &ascii(b"currentTarget"),
                &ascii(b"get currentTarget"),
            ),
            (
                Attribute::Phase,
                &ascii(b"eventPhase"),
                &ascii(b"get eventPhase"),
            ),
            (
                Attribute::CancelBubble,
                &ascii(b"cancelBubble"),
                &ascii(b"get cancelBubble"),
            ),
            (
                Attribute::Bubbles,
                &ascii(b"bubbles"),
                &ascii(b"get bubbles"),
            ),
            (
                Attribute::Cancelable,
                &ascii(b"cancelable"),
                &ascii(b"get cancelable"),
            ),
            (
                Attribute::ReturnValue,
                &ascii(b"returnValue"),
                &ascii(b"get returnValue"),
            ),
            (
                Attribute::DefaultPrevented,
                &ascii(b"defaultPrevented"),
                &ascii(b"get defaultPrevented"),
            ),
            (
                Attribute::Composed,
                &ascii(b"composed"),
                &ascii(b"get composed"),
            ),
            (
                Attribute::TimeStamp,
                &ascii(b"timeStamp"),
                &ascii(b"get timeStamp"),
            ),
        ] {
            let getter = make_getter(
                scope,
                capture(size_of::<(Capture, Attribute)>())?,
                attribute,
                name,
            )?;
            let setter = operations::event_setter(scope, attribute, &capture)?;
            define_accessor(scope, &event, key, &getter, setter.as_ref(), true)?;
        }
        operations::install(scope, &event, &capture)?;
        let trusted = make_getter(
            scope,
            capture(size_of::<(Capture, Attribute)>())?,
            Attribute::Trusted,
            &ascii(b"get isTrusted"),
        )?;

        let target = install_target_prototype(scope, &capture_origin)?;

        install_event_interface(scope, Interface::Event, &event, &capture)?;
        let toggle_event = toggle_event::install(scope, &event, &capture)?;
        let toggle_event = scope.callback_object(&toggle_event)?;
        let toggle_event = scope.retain_callback_object(toggle_event, fund)?;
        let violation = security_policy_violation::install(scope, &event, &capture)?;
        let violation = scope.callback_object(&violation)?;
        let security_policy_violation = scope.retain_callback_object(violation, fund)?;
        install_event_interface(scope, Interface::Target, &target, &capture_origin)?;
        let observed = scope.callback_object(&event)?;
        let document = document::DocumentBindings::install(
            scope,
            document_id,
            &target,
            inbox,
            &capture,
            (&admission.budget, owner),
            OriginInterfaceFunding {
                capture: &capture_origin,
                retain: &fund_origin,
            },
        )?;
        let event = scope.retain_callback_object(observed, fund)?;
        let observed = scope.callback_object(&target)?;
        let target = scope.retain_callback_object(observed, fund_origin)?;
        let observed = scope.callback_object(&trusted)?;
        let trusted = scope.retain_callback_object(observed, fund)?;
        let window = scope.global()?;
        let custom_elements = custom_elements::install(
            scope,
            &window,
            (document_id, global),
            registry,
            &capture_origin,
            &fund_origin,
        )?;
        let trusted_types = trusted_policy::install(scope, &window, &capture, &fund)?;
        let window = scope.callback_object(&window)?;
        let window = scope.retain_callback_object(window, fund)?;
        let dispatch = capture(size_of::<Capture>())?;
        let dispatch_in_attachment = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                let (owner, events) = dispatch.active(scope)?;
                wrapper::call_dispatch_event(
                    &events,
                    &owner,
                    scope,
                    arguments,
                    wrapper::DispatchEntry::Attachment,
                )
            }),
            FunctionMetadata {
                name: &[],
                length: 1,
            },
        )?;
        let dispatch_in_attachment = scope.callback_object(&dispatch_in_attachment)?;
        let dispatch_in_attachment = scope.retain_callback_object(dispatch_in_attachment, fund)?;
        let backup = capture(size_of::<Capture>())?;
        let backup_reaction_job = scope.fixed_arity_function(
            &Callback::new(move |scope, _| backup.run_backup_reactions(scope)),
            FunctionMetadata {
                name: &[],
                length: 0,
            },
        )?;
        let backup_reaction_job = scope.callback_object(&backup_reaction_job)?;
        let backup_reaction_job = scope.retain_callback_object(backup_reaction_job, fund)?;
        let origin_cost = origin_cost
            .get()
            // The receiving origin replaces attachment-only constructors with
            // one shared illegal Document/Fragment callback and Text/Comment
            // callbacks. Their action captures were absent from this probe.
            .include(constructor_capture_bytes(size_of::<Capture>())?)?
            .include(constructor_capture_bytes(size_of::<Capture>())?)?
            .include(constructor_capture_bytes(size_of::<Capture>())?)?
            .include(size_of::<OriginBindings>())?
            .include(OriginAuthority::STORAGE_BYTES)?
            .include(
                dom_exception::constructor_target_bytes::<GuestBacking>()
                    .and_then(|bytes| bytes.checked_add(dom_exception_metadata_bytes))
                    .ok_or(DocumentScriptNativeError::StorageLimit)?,
            )?;
        Ok(Self {
            callbacks,
            window,
            trusted_types,
            event,
            toggle_event,
            security_policy_violation,
            target,
            trusted,
            dispatch_in_attachment,
            backup_reaction_job,
            backup_reactions_scheduled: Cell::new(false),
            document,
            custom_elements,
            scoped_registries: RefCell::default(),
            origins: RefCell::new(None),
            origin_cost,
        })
    }

    pub(super) fn origin_installation_cost(&self) -> (usize, u64) {
        (self.origin_cost.bytes, self.origin_cost.requests)
    }

    pub(super) fn install_origin(
        &self,
        scope: &mut Scope<'_>,
        events: &GuestEvents,
        dom_exception_limits: DomExceptionLimits,
        (global, document, registry): (DomHostedGlobalId, DocumentId, surgeist_dom::DomRegistryId),
        admission: OriginAdmission,
        conversion_host: preparation::NativeConversionHost,
    ) -> Result<(), DocumentGuestEventError> {
        let fund = |bytes| admission.take(events, bytes);
        let backing = fund(size_of::<OriginBindings>())?;
        let realm = scope.realm_authority()?.identity();
        let authority = Rc::new(OriginAuthority {
            global,
            realm,
            _backing: fund(OriginAuthority::STORAGE_BYTES)?,
        });
        let capture = |bytes| -> Result<Capture, DocumentGuestEventError> {
            Ok(Capture {
                callbacks: self.callbacks.clone(),
                origin: Some(authority.clone()),
                _backing: fund(bytes)?,
            })
        };
        let hosted = Hosted::new(conversion_host);
        let metadata_bytes = hosted
            .storage_bytes::<GuestBacking>()
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let target_bytes = dom_exception::constructor_target_bytes::<GuestBacking>()
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let mut constructor_backing = fund(
            target_bytes
                .checked_add(metadata_bytes)
                .ok_or(DocumentScriptNativeError::StorageLimit)?,
        )?;
        let conversion_backing = constructor_backing.split_off(metadata_bytes)?;
        dom_exception::install_hosted(
            scope,
            dom_exception_limits,
            hosted,
            conversion_backing,
            constructor_backing,
        )?;
        let target = install_target_prototype(scope, &capture)?;
        install_event_interface(scope, Interface::Target, &target, &capture)?;
        let (constructor, node) = document::install_node(scope, &target, &capture)?;
        let documents = document::DocumentPrototypes::install_origin(
            scope,
            &node,
            &constructor,
            &OriginInterfaceFunding {
                capture: &capture,
                retain: &fund,
            },
        )?;
        let elements = document::element::ElementBindings::install(
            scope,
            &node,
            &constructor,
            &capture,
            &fund,
        )?;
        let window = scope.global()?;
        let custom_elements = custom_elements::install(
            scope,
            &window,
            (document, global),
            registry,
            &capture,
            &fund,
        )?;
        let target = scope.callback_object(&target)?;
        let target = scope.retain_callback_object(target, fund)?;
        admission.finish()?;
        let mut origins = self
            .origins
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let record = Box::new(OriginBindings {
            global,
            target,
            documents,
            elements,
            custom_elements,
            next: origins.take(),
            _backing: backing,
        });
        *origins = Some(record);
        Ok(())
    }

    fn with_origin<T>(
        &self,
        events: &GuestEvents,
        global: DomHostedGlobalId,
        use_origin: impl FnOnce(&OriginBindings) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        self.with_origin_with(global, || events.step(), use_origin)
    }

    fn with_origin_with<T>(
        &self,
        global: DomHostedGlobalId,
        mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
        use_origin: impl FnOnce(&OriginBindings) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        let origins = self
            .origins
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut next = origins.as_deref();
        while let Some(origin) = next {
            step()?;
            if origin.global == global {
                return use_origin(origin);
            }
            next = origin.next.as_deref();
        }
        Err(DocumentScriptNativeError::CreationGlobalUnavailable.into())
    }

    pub(super) fn activate(&self, events: &Rc<GuestEvents>) {
        self.callbacks.access.get_or_init(|| GuestAccess {
            events: Rc::downgrade(events),
            _backing: events.backing.clone(),
        });
    }

    pub(super) fn window<'scope>(&self, scope: &mut Scope<'scope>) -> Result<Value<'scope>, Error> {
        scope.open_callback_object(&self.window)
    }

    pub(super) fn target_prototype<'scope>(
        &self,
        scope: &mut Scope<'scope>,
    ) -> Result<Value<'scope>, Error> {
        scope.open_callback_object(&self.target)
    }

    pub(super) fn event_prototype<'scope>(
        &self,
        scope: &mut Scope<'scope>,
    ) -> Result<Value<'scope>, Error> {
        scope.open_callback_object(&self.event)
    }

    pub(super) fn native_event_prototype<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        event: &NativeDomEvent,
    ) -> Result<Value<'scope>, Error> {
        if event.csp_violation().is_some() {
            return scope.open_callback_object(&self.security_policy_violation);
        }
        match event.toggle_event() {
            Some(_) => scope.open_callback_object(&self.toggle_event),
            None => self.event_prototype(scope),
        }
    }

    pub(super) fn trusted_getter<'scope>(
        &self,
        scope: &mut Scope<'scope>,
    ) -> Result<Value<'scope>, Error> {
        scope.open_callback_object(&self.trusted)
    }
}

fn install_target_prototype<'scope>(
    scope: &mut Scope<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    let target = scope.object(ObjectPrototype::Default)?;
    listener_methods::install(scope, &target, capture)?;
    let dispatch = capture(size_of::<Capture>())?;
    let dispatch_name = &ascii(b"dispatchEvent");
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| dispatch.dispatch(scope, arguments)),
        FunctionMetadata {
            name: dispatch_name,
            length: 1,
        },
    )?;
    define_data(
        scope,
        &target,
        PropertyKey::String(dispatch_name),
        &method,
        true,
        true,
        true,
    )?;
    Ok(target)
}

fn install_event_interface<'scope>(
    scope: &mut Scope<'scope>,
    kind: Interface,
    prototype: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(), DocumentGuestEventError> {
    let fallback = capture(size_of::<(Capture, Interface)>())?;
    let constructor = constructor_capture(capture, size_of::<(Capture, Interface)>())?;
    let (name, length, prefix) = kind.declaration();
    let callback = ConstructorCallback::bounded(
        prefix,
        move |scope| fallback.prototype(scope, kind),
        move |scope, arguments| constructor.construct(scope, kind, arguments),
    );
    let constructor = scope.constructor(&callback, FunctionMetadata { name, length })?;
    scope.define_property(
        &constructor,
        PropertyKey::String(PROTOTYPE),
        PropertyDescriptor::Data {
            value: prototype,
            writable: false,
            enumerable: false,
            configurable: false,
        },
    )?;
    if matches!(kind, Interface::Event) {
        for (key, code) in [
            (&ascii(b"NONE")[..], 0.0),
            (&ascii(b"CAPTURING_PHASE")[..], 1.0),
            (&ascii(b"AT_TARGET")[..], 2.0),
            (&ascii(b"BUBBLING_PHASE")[..], 3.0),
        ] {
            let value = scope.primitive(Primitive::Number(code))?;
            for recipient in [&constructor, prototype] {
                define_data(
                    scope,
                    recipient,
                    PropertyKey::String(key),
                    &value,
                    false,
                    true,
                    false,
                )?;
            }
        }
    }
    let global = scope.global()?;
    document::install_interface(scope, &global, name, &constructor, prototype)?;
    Ok(())
}

enum SourceReadRequest<'a> {
    Event(&'a surgeist_dom::DomSourceEventRequest),
    Attribute {
        request: surgeist_dom::DomSourceAttributeVerificationRequest,
        registration: Rc<surgeist_dom::DomDocumentScriptRegistration>,
    },
    Constructor {
        request: surgeist_dom::DomSourceCustomElementRequest,
        registration: Rc<surgeist_dom::DomDocumentScriptRegistration>,
    },
}

struct NativeRead<'a> {
    snapshot: DomSnapshot,
    events: &'a GuestEvents,
    account_scope: AccountScope,
    source: Option<SourceReadRequest<'a>>,
    dom: RefCell<crate::DomEventDom<'a>>,
    hosted: HostedRuntime,
    cancel: &'a AtomicBool,
}

enum NativeTextUnits {
    Event {
        _retained: surgeist_dom::DomSourceEventRetention,
    },
    Attribute {
        _retained: surgeist_dom::DomSourceAttributePayloadRetention,
    },
    Constructor {
        _retained: surgeist_dom::DomSourceCustomElementRetention,
    },
}

impl NativeRead<'_> {
    fn source_step(&self) -> Result<(), DocumentGuestEventError> {
        if self.cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(DomError::Cancelled.into());
        }
        if let Some(request) = &self.source {
            let dom = self
                .dom
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            match request {
                SourceReadRequest::Event(request) => {
                    dom.raw().source_event_step(request, self.cancel)?
                }
                SourceReadRequest::Attribute {
                    request,
                    registration,
                } => {
                    dom.raw().source_attribute_callback_step(
                        *request,
                        registration,
                        self.cancel,
                    )?;
                }
                SourceReadRequest::Constructor {
                    request,
                    registration,
                } => {
                    dom.raw().source_custom_element_callback_step(
                        *request,
                        registration,
                        self.cancel,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn step(&self) -> Result<(), DocumentGuestEventError> {
        self.source_step()?;
        self.events.budget.charge_work(self.account_scope, 1)
    }

    fn reserve(&self, bytes: usize) -> Result<GuestBacking, DocumentGuestEventError> {
        self.source_step()?;
        self.events
            .budget
            .reserve_with(self.account_scope, bytes, || {
                let mut dom = self
                    .dom
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                self.hosted
                    .retain_script_payload(&mut dom, bytes, self.cancel)
                    .map_err(DocumentScriptNativeError::from)
            })
    }

    fn prepare_root(
        &self,
        request: surgeist_bindings_js::preparation::Request,
    ) -> Result<crate::document_scripting::preparation::Backing, Error> {
        let result = (|| {
            self.source_step()?;
            let bytes = request.bytes();
            let backing =
                self.events
                    .budget
                    .reserve_with(AccountScope::Preparation, bytes, || {
                        let mut dom = self
                            .dom
                            .try_borrow_mut()
                            .map_err(|_| DocumentScriptNativeError::Busy)?;
                        self.hosted
                            .retain_script_payload(&mut dom, bytes, self.cancel)
                            .map_err(DocumentScriptNativeError::from)
                    })?;
            Ok::<_, DocumentGuestEventError>(
                crate::document_scripting::preparation::Backing::Guest(
                    accounting::CaptureBacking::new(backing, self.events.refusal.clone()),
                ),
            )
        })();
        result.map_err(|error| accounting::capture_refusal(&self.events.refusal, error))
    }

    fn retain_text_units(
        &self,
        units: usize,
    ) -> Result<Option<NativeTextUnits>, DocumentGuestEventError> {
        let Some(request) = &self.source else {
            return Ok(None);
        };
        let dom = self
            .dom
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let retained = match request {
            SourceReadRequest::Event(request) => NativeTextUnits::Event {
                _retained: dom
                    .raw()
                    .retain_source_event_payload(request, 0, units, self.cancel)?,
            },
            SourceReadRequest::Attribute {
                request,
                registration,
            } => NativeTextUnits::Attribute {
                _retained: dom.raw().retain_source_attribute_callback_units(
                    *request,
                    registration,
                    units,
                    self.cancel,
                )?,
            },
            SourceReadRequest::Constructor {
                request,
                registration,
            } => NativeTextUnits::Constructor {
                _retained: dom.raw().retain_source_custom_element_units(
                    *request,
                    registration,
                    units,
                    self.cancel,
                )?,
            },
        };
        Ok(Some(retained))
    }

    fn node(
        &self,
        id: surgeist_dom::NodeId,
    ) -> Result<surgeist_dom::DomNode<'_>, DocumentGuestEventError> {
        self.snapshot.node_with(id, || self.step())
    }
}

impl Callbacks {
    fn with_native<T>(
        &self,
        owner: &NativeOwner,
        action: impl FnOnce(&mut DomSession, DomHostedGlobalId) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        self.with_qualified_session(owner, None, action)
    }

    fn with_read<T>(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        action: impl FnOnce(&NativeRead<'_>, DomHostedGlobalId) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        self.with_accounted_read(owner, events, AccountScope::Adapter, action)
    }

    fn with_accounted_read<T>(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        account_scope: AccountScope,
        action: impl FnOnce(&NativeRead<'_>, DomHostedGlobalId) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        let request = owner.active_event_request();
        self.with_qualified_session(owner, request.as_ref(), |session, global| {
            let snapshot = session.snapshot();
            let hosted = session.hosted_execution();
            let dom = session.dom();
            let request = if let Some((request, registration)) =
                owner.active_constructor_registration(dom.pending_source_edit())?
            {
                Some(SourceReadRequest::Constructor {
                    request,
                    registration,
                })
            } else if let Some((request, registration)) = owner.active_attribute_registration()? {
                Some(SourceReadRequest::Attribute {
                    request,
                    registration,
                })
            } else {
                request.as_ref().map(SourceReadRequest::Event)
            };
            let read = NativeRead {
                snapshot,
                events,
                account_scope,
                source: request,
                dom: RefCell::new(dom),
                hosted,
                cancel: &owner.cancellation,
            };
            action(&read, global)
        })
    }

    fn qualify_listener_registry(
        &self,
        owner: &NativeOwner,
    ) -> Result<(), DocumentGuestEventError> {
        let request = owner.attached_event_request();
        self.with_qualified_session(owner, request.as_ref(), |_, _| Ok(()))
    }

    fn with_qualified_session<T>(
        &self,
        owner: &NativeOwner,
        request: Option<&surgeist_dom::DomSourceEventRequest>,
        action: impl FnOnce(&mut DomSession, DomHostedGlobalId) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        if attachment.realm.identity() != self.permit.realm
            || attachment.target.document() != self.permit.document
        {
            return Err(DocumentScriptNativeError::ForeignPermit.into());
        }
        let mut session = owner
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let current = session.read_dom().pending_source_edit();
        let status = if let Some((request, registration)) =
            owner.active_constructor_registration(current)?
        {
            session.dom().raw().source_custom_element_callback_step(
                request,
                &registration,
                &owner.cancellation,
            )?;
            HtmlScriptingStatus::Enabled
        } else if let Some((request, registration)) = owner.active_attribute_registration()? {
            session.dom().raw().source_attribute_callback_step(
                request,
                &registration,
                &owner.cancellation,
            )?;
            HtmlScriptingStatus::Enabled
        } else {
            match request {
                Some(request) => owner.qualify_event(attachment, &mut session, request)?,
                None => owner.observe(attachment, &mut session)?.status(),
            }
        };
        if status != HtmlScriptingStatus::Enabled {
            return Err(DocumentScriptNativeError::Revoked.into());
        }
        action(&mut session, attachment.target.global())
    }
}

impl Capture {
    fn active(&self, scope: &mut Scope<'_>) -> Result<(Rc<NativeOwner>, Rc<GuestEvents>), Error> {
        let owner = self
            .callbacks
            .owner
            .upgrade()
            .ok_or(Error::ExpiredIdentity)?;
        let events = self
            .callbacks
            .access
            .get()
            .and_then(|access| access.events.upgrade())
            .ok_or(Error::ExpiredIdentity)?;
        let realm = self
            .origin
            .as_ref()
            .map_or(self.callbacks.permit.realm, |origin| origin.realm);
        if scope.realm_authority()?.identity() != realm {
            return refuse(
                scope,
                &events.refusal,
                DocumentGuestEventError::CallbackRealmUnavailable,
            );
        }
        Ok((owner, events))
    }

    fn prototype<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        kind: Interface,
    ) -> Result<Value<'scope>, Error> {
        let (_, events) = self.active(scope)?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        if let (Interface::Target, Some(origin)) = (kind, &self.origin) {
            return match public.with_origin(&events, origin.global, |bindings| {
                Ok(scope.open_callback_object(&bindings.target)?)
            }) {
                Ok(value) => Ok(value),
                Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                    Err(error)
                }
                Err(error) => refuse(scope, &events.refusal, error),
            };
        }
        scope.open_callback_object(match kind {
            Interface::Event => &public.event,
            Interface::Target => &public.target,
        })
    }

    fn dispatch<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let entry = if self.origin.is_some() {
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            wrapper::DispatchEntry::Origin(&public.dispatch_in_attachment)
        } else {
            wrapper::DispatchEntry::Direct
        };
        wrapper::call_dispatch_event(&events, &owner, scope, arguments, entry)
    }

    fn construct<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        kind: Interface,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = match kind {
            Interface::Event => self.construct_event(scope, &owner, &events, arguments),
            Interface::Target => self.construct_target(scope, &owner, &events, arguments),
        };
        match result {
            Ok(value) => Ok(ConstructorCompletion::Return(value)),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn construct_event<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let ty = event_constructor_0_type();
        let init = event_constructor_0_event_init_dict();
        let (ty, init) = self
            .callbacks
            .conversion
            .run(scope, |conversion| {
                Ok((
                    conversion.argument(&ty, arguments.get(ty.index()))?,
                    conversion.argument(&init, arguments.get(init.index()))?,
                ))
            })
            .map_err(Error::from)?;
        let (ArgumentValue::Present(ty), ArgumentValue::Present(init)) = (ty.get(), init.get())
        else {
            return Err(Error::BackendUnavailable.into());
        };
        let selected = arguments.select_prototype(scope)?;
        scope.check_active()?;
        events.step()?;
        let native = self.with_native(owner, |session, global| {
            session
                .hosted_execution()
                .create_event(
                    &mut session.dom(),
                    global,
                    ty.as_code_units(),
                    DomEventInit {
                        bubbles: init.bubbles,
                        cancelable: init.cancelable,
                        composed: init.composed,
                    },
                    &owner.cancellation,
                )
                .map_err(DocumentGuestEventError::Construction)
        })?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let trusted = public.trusted_getter(scope)?;
        let value = scope.project_native_object(
            ObjectPrototype::Object(selected.prototype()),
            GuestEventProjection(native),
            |bytes| events.reserve(owner, bytes),
        )?;
        expose_trust(scope, &value, &trusted)?;
        Ok(value)
    }

    fn construct_target<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let selected = arguments.select_prototype(scope)?;
        scope.check_active()?;
        events.step()?;
        let native = self.with_native(owner, |session, global| {
            let global = self.origin.as_ref().map_or(global, |origin| origin.global);
            let registry = events
                .registry
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session
                .hosted_execution()
                .create_event_target(&mut session.dom(), &registry, global, &owner.cancellation)
                .map_err(DocumentGuestEventError::Construction)
        })?;
        scope.project_native_object(
            ObjectPrototype::Object(selected.prototype()),
            events.target_projection(owner, native)?,
            |bytes| events.reserve(owner, bytes),
        )
    }

    fn with_native<T>(
        &self,
        owner: &NativeOwner,
        action: impl FnOnce(&mut DomSession, DomHostedGlobalId) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        self.callbacks.with_native(owner, action)
    }

    fn get<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        attribute: Attribute,
    ) -> Result<Completion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let Some(native) = scope.native_object::<GuestEventProjection>(arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Event required"))?,
            ));
        };
        if let Err(error) = events.step() {
            return refuse(scope, &events.refusal, error);
        }
        let event = &native.payload().0;
        let value = match attribute {
            Attribute::Type => {
                let ty = event.event_type();
                self.callbacks
                    .conversion
                    .run(scope, |conversion| conversion.copy_to_js(ty.code_units()))?
            }
            Attribute::Target => operations::nullable_target(
                &owner,
                &events,
                scope,
                arguments.receiver(),
                event.target(),
            )?,
            Attribute::CurrentTarget => operations::nullable_target(
                &owner,
                &events,
                scope,
                arguments.receiver(),
                event.current_target(),
            )?,
            Attribute::CancelBubble => {
                scope.primitive(Primitive::Boolean(event.cancel_bubble()))?
            }
            Attribute::ReturnValue => scope.primitive(Primitive::Boolean(event.return_value()))?,
            Attribute::Bubbles => scope.primitive(Primitive::Boolean(event.bubbles()))?,
            Attribute::Cancelable => scope.primitive(Primitive::Boolean(event.cancelable()))?,
            Attribute::Composed => scope.primitive(Primitive::Boolean(event.composed()))?,
            Attribute::DefaultPrevented => {
                scope.primitive(Primitive::Boolean(event.default_prevented()))?
            }
            Attribute::Trusted => scope.primitive(Primitive::Boolean(event.is_trusted()))?,
            Attribute::Phase => {
                use crate::DomEventPhase;
                let phase = match event.phase() {
                    DomEventPhase::None => 0.0,
                    DomEventPhase::Capturing => 1.0,
                    DomEventPhase::AtTarget => 2.0,
                    DomEventPhase::Bubbling => 3.0,
                };
                scope.primitive(Primitive::Number(phase))?
            }
            Attribute::TimeStamp => {
                let time = match event.time_stamp() {
                    Ok(time) => time,
                    Err(error) => {
                        return refuse(
                            scope,
                            &events.refusal,
                            DocumentGuestEventError::Timestamp(error),
                        );
                    }
                };
                scope.primitive(Primitive::Number(time))?
            }
        };
        Ok(Completion::Return(value))
    }
}

fn make_getter<'scope>(
    scope: &mut Scope<'scope>,
    capture: Capture,
    attribute: Attribute,
    name: &[u16],
) -> Result<Value<'scope>, Error> {
    scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| capture.get(scope, arguments, attribute)),
        FunctionMetadata { name, length: 0 },
    )
}

pub(super) fn expose_trust<'scope>(
    scope: &mut Scope<'scope>,
    event: &Value<'scope>,
    trusted: &Value<'scope>,
) -> Result<(), Error> {
    define_accessor(scope, event, TRUSTED, trusted, None, false)
}

fn define_accessor<'scope>(
    scope: &mut Scope<'scope>,
    object: &Value<'scope>,
    name: &[u16],
    getter: &Value<'scope>,
    setter: Option<&Value<'scope>>,
    configurable: bool,
) -> Result<(), Error> {
    scope.define_new_property(
        object,
        PropertyKey::String(name),
        PropertyDescriptor::Accessor {
            get: Some(getter),
            set: setter,
            enumerable: true,
            configurable,
        },
    )
}

fn define_data<'scope>(
    scope: &mut Scope<'scope>,
    object: &Value<'scope>,
    key: PropertyKey<'_, 'scope>,
    value: &Value<'scope>,
    writable: bool,
    enumerable: bool,
    configurable: bool,
) -> Result<(), Error> {
    scope.define_new_property(
        object,
        key,
        PropertyDescriptor::Data {
            value,
            writable,
            enumerable,
            configurable,
        },
    )
}
