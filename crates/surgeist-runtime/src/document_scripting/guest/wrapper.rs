use super::*;
use surgeist_bindings_js::{Arguments, FunctionMetadata};

struct DispatchMethod {
    owner: Weak<NativeOwner>,
    events: Weak<GuestEvents>,
    _backing: GuestBacking,
}

impl GuestEvents {
    pub(super) fn target_prototype<'scope>(
        self: &Rc<Self>,
        owner: &Rc<NativeOwner>,
        scope: &mut Scope<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        if let Some(prototype) = self.target_prototype.get() {
            return Ok(scope.open_callback_object(prototype)?);
        }
        let capture = DispatchMethod {
            owner: Rc::downgrade(owner),
            events: Rc::downgrade(self),
            _backing: self.reserve(owner, size_of::<DispatchMethod>())?,
        };
        let prototype = scope.object(ObjectPrototype::Default)?;
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
        let observed = scope.callback_object(&prototype)?;
        let retained =
            scope.retain_callback_object(observed, |bytes| self.reserve(owner, bytes))?;
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
        target: &DomStandaloneEventTarget,
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
        let invocation = self.prepare(owner, target, event)?;
        let mut reports = self.reports.reserve(self, owner)?;
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
        if let Err(error) = events.step() {
            return refuse(scope, &events.refusal, error);
        }
        let receiver = arguments.receiver();
        let Some(target) = scope.native_object::<DomStandaloneEventTarget>(receiver)? else {
            return Ok(Completion::Throw(
                scope.type_error(&b"EventTarget required".map(u16::from))?,
            ));
        };
        let Some(argument) = arguments.get(0) else {
            return Ok(Completion::Throw(
                scope.type_error(&b"Event required".map(u16::from))?,
            ));
        };
        let Some(event) = scope.native_object::<NativeDomEvent>(argument)? else {
            return Ok(Completion::Throw(
                scope.type_error(&b"Event required".map(u16::from))?,
            ));
        };
        match events.dispatch_from_guest(
            &owner,
            scope,
            receiver,
            target.payload(),
            argument,
            event.payload(),
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
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }
}
