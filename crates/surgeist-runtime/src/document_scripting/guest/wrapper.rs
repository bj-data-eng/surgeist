use super::*;
use surgeist_bindings_js::{Arguments, FunctionMetadata, NativeObject};

pub(super) enum GuestReceiver {
    Projected(Rc<NativeObject<GuestTargetProjection>>),
    Window(DomHostedGlobalId),
    Node {
        node: surgeist_dom::NodeId,
        lifetime: Option<Rc<interfaces::NodeLifetime>>,
    },
}

impl GuestReceiver {
    pub(super) fn target(&self) -> DomEventTarget {
        match self {
            Self::Projected(projection) => projection.payload().native.clone().into(),
            Self::Window(global) => (*global).into(),
            Self::Node { node, .. } => (*node).into(),
        }
    }
}

pub(super) fn event_receiver<'scope>(
    owner: &NativeOwner,
    scope: &mut Scope<'scope>,
    receiver: &Value<'scope>,
) -> Result<Option<GuestReceiver>, Error> {
    if let Some(projection) = scope.native_object::<GuestTargetProjection>(receiver)? {
        return Ok(Some(GuestReceiver::Projected(projection)));
    }
    if let Some(node) = scope.native_object::<interfaces::GuestNodeProjection>(receiver)? {
        let id = node.payload().id().ok_or(Error::RegistryConflict)?;
        return Ok(Some(GuestReceiver::Node {
            node: id.node(),
            lifetime: node.payload().lifetime(),
        }));
    }
    let attachment = owner.attachment.get().ok_or(Error::ExpiredIdentity)?;
    let global = scope.global()?;
    Ok(scope
        .strict_equals(receiver, &global)?
        .then(|| GuestReceiver::Window(attachment.target.global())))
}

pub(super) fn event_target_value<'scope>(
    owner: &NativeOwner,
    events: &GuestEvents,
    scope: &mut Scope<'scope>,
    event: &Value<'scope>,
    target: &DomEventTarget,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    match target {
        DomEventTarget::Global(global) => {
            let attachment = owner.attachment.get().ok_or(Error::ExpiredIdentity)?;
            if *global == attachment.target.global() {
                return Ok(scope.global()?);
            }
        }
        DomEventTarget::Node(node) => {
            return events
                .public
                .as_ref()
                .ok_or(Error::ExpiredIdentity)?
                .document_object(scope, events, *node);
        }
        DomEventTarget::Standalone(expected) => {
            let value = scope
                .native_object_reference::<GuestEventProjection>(
                    event,
                    interfaces::EVENT_TARGET_REFERENCE,
                )?
                .ok_or(Error::RegistryConflict)?;
            if scope
                .native_object::<GuestTargetProjection>(&value)?
                .is_some_and(|projection| &projection.payload().native == expected)
            {
                return Ok(value);
            }
        }
    }
    Err(Error::RegistryConflict.into())
}

struct DispatchMethod {
    owner: Weak<NativeOwner>,
    events: Weak<GuestEvents>,
    _event_backing: Rc<GuestFixedBacking>,
    _backing: GuestBacking,
}

impl GuestEvents {
    pub(super) fn target_prototype<'scope>(
        self: &Rc<Self>,
        owner: &Rc<NativeOwner>,
        scope: &mut Scope<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        if let Some(public) = &self.public {
            return Ok(public.target_prototype(scope)?);
        }
        if let Some(prototype) = self.target_prototype.get() {
            return Ok(scope.open_callback_object(prototype)?);
        }
        let capture = DispatchMethod {
            owner: Rc::downgrade(owner),
            events: Rc::downgrade(self),
            _event_backing: self.backing.clone(),
            _backing: self.reserve(owner, size_of::<DispatchMethod>())?,
        };
        let prototype = scope.object(ObjectPrototype::Default)?;
        let observed = scope.callback_object(&prototype)?;
        let retained =
            scope.retain_callback_object(observed, |bytes| self.reserve(owner, bytes))?;
        let name = b"dispatchEvent".map(u16::from);
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| capture.call(scope, arguments)),
            FunctionMetadata {
                name: &name,
                length: 1,
            },
        )?;
        scope.define_new_property(
            &prototype,
            PropertyKey::String(&name),
            PropertyDescriptor::Data {
                value: &method,
                writable: true,
                enumerable: true,
                configurable: true,
            },
        )?;
        self.target_prototype
            .set(retained)
            .map_err(|_| DocumentGuestEventError::RegistryState)?;
        Ok(prototype)
    }

    fn dispatch_from_guest<'scope>(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'scope>,
        receiver: &Value<'scope>,
        target: DomEventTarget,
        argument: &Value<'scope>,
        event: &NativeDomEvent,
    ) -> Result<bool, DocumentGuestEventError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        if scope.realm_authority()?.identity() != attachment.realm.identity() {
            return Err(DocumentGuestEventError::CallbackRealmUnavailable);
        }
        {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            if owner.observe(attachment, &mut session)?.status() != HtmlScriptingStatus::Enabled {
                return Err(DocumentScriptNativeError::Revoked.into());
            }
        }
        event.check_dispatch_state()?;
        let mut reports = self.reports.reserve(self, owner)?;
        let invocation = self.prepare(owner, target, event)?;
        let receipt = self.run(
            owner,
            scope,
            invocation,
            receiver,
            argument,
            |scope, exception| reports.publish(scope, exception),
        )?;
        Ok(!receipt.default_prevented())
    }
}

impl DispatchMethod {
    fn call<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let events = self.events.upgrade().ok_or(Error::ExpiredIdentity)?;
        let owner = self.owner.upgrade().ok_or(Error::ExpiredIdentity)?;
        call_dispatch_event(&events, &owner, scope, arguments)
    }
}

pub(super) fn call_dispatch_event<'scope>(
    events: &GuestEvents,
    owner: &NativeOwner,
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    if let Err(error) = events.step() {
        return refuse(scope, &events.refusal, error);
    }
    let receiver = arguments.receiver();
    let Some(target) = event_receiver(owner, scope, receiver)? else {
        return Ok(Completion::Throw(
            scope.type_error(&b"EventTarget required".map(u16::from))?,
        ));
    };
    let Some(argument) = arguments.get(0) else {
        return Ok(Completion::Throw(
            scope.type_error(&b"Event required".map(u16::from))?,
        ));
    };
    let Some(event) = scope.native_object::<GuestEventProjection>(argument)? else {
        return Ok(Completion::Throw(
            scope.type_error(&b"Event required".map(u16::from))?,
        ));
    };
    match events.dispatch_from_guest(
        owner,
        scope,
        receiver,
        target.target(),
        argument,
        &event.payload().0,
    ) {
        Ok(result) => Ok(Completion::Return(
            scope.primitive(Primitive::Boolean(result))?,
        )),
        Err(DocumentGuestEventError::Dispatch(
            DomEventDispatchError::AlreadyDispatching | DomEventDispatchError::Uninitialized,
        )) => Ok(Completion::Throw(scope.dom_exception(
            dom_exception::DomExceptionName::InvalidStateError,
            &[],
        )?)),
        Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => Err(error),
        Err(error) => refuse(scope, &events.refusal, error),
    }
}
