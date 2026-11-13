//! Named policy methods and the string-valued Trusted Type prototypes.

use super::*;
use surgeist_bindings_js::TrustedTypeKind;

const METHODS: &[(TrustedTypeKind, &[u16])] = &[
    (TrustedTypeKind::Html, &ascii(b"createHTML")),
    (TrustedTypeKind::Script, &ascii(b"createScript")),
    (TrustedTypeKind::ScriptUrl, &ascii(b"createScriptURL")),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    policy_prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for &(kind, name) in METHODS {
        let method = NamedMethod {
            capture: capture(size_of::<NamedMethod>())?,
            kind,
            prototype: OnceCell::new(),
        };
        let function = scope.bounded_function(
            &Callback::new(move |scope, arguments| method.call(scope, arguments)),
            FunctionMetadata { name, length: 1 },
            u32::MAX,
        )?;
        define_data(
            scope,
            policy_prototype,
            PropertyKey::String(name),
            &function,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

struct NamedMethod {
    capture: Capture,
    kind: TrustedTypeKind,
    prototype: OnceCell<RetainedCallbackObject>,
}

impl NamedMethod {
    fn prototype<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        if self.prototype.get().is_none() {
            let prototype = scope.object(ObjectPrototype::Default)?;
            let kind = self.kind;
            for key in [&ascii(b"toString")[..], &ascii(b"toJSON")[..]] {
                let capture = Capture {
                    callbacks: self.capture.callbacks.clone(),
                    origin: None,
                    _backing: events.reserve(owner, size_of::<(Capture, TrustedTypeKind)>())?,
                };
                let stringifier = scope.fixed_arity_function(
                    &Callback::new(move |scope, arguments| {
                        stringify(&capture, scope, arguments, kind)
                    }),
                    FunctionMetadata {
                        name: key,
                        length: 0,
                    },
                )?;
                define_data(
                    scope,
                    &prototype,
                    PropertyKey::String(key),
                    &stringifier,
                    true,
                    true,
                    true,
                )?;
            }
            let observed = scope.callback_object(&prototype)?;
            let retained =
                scope.retain_callback_object(observed, |bytes| events.reserve(owner, bytes))?;
            self.prototype
                .set(retained)
                .map_err(|_| DocumentScriptNativeError::Busy)?;
        }
        let retained = self.prototype.get().ok_or(Error::ExpiredIdentity)?;
        Ok(scope.open_callback_object(retained)?)
    }

    fn call<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let (owner, events) = self.capture.active(scope)?;
        let (receiver, arguments) = arguments.into_parts();
        let Some(policy) = scope.native_object::<PolicyObject>(&receiver)? else {
            return type_error(scope, &ascii(b"TrustedTypePolicy required"));
        };
        let Some((input, trailing)) = arguments.split_first() else {
            return type_error(scope, &ascii(b"Policy input required"));
        };
        if let Err(error) = qualify(&self.capture, &owner, &events) {
            return refuse(scope, &events.refusal, error);
        }
        let account = &self.capture.callbacks.conversion;
        let input = account.run(scope, |cx| cx.value(&types::DOM_STRING, input))?;
        let Some(callback) = policy.payload().callback(self.kind) else {
            return type_error(scope, &ascii(b"Policy callback is missing"));
        };
        let data = account.run(scope, |cx| {
            cx.invoke_named_policy(callback, self.kind, input.get(), trailing)
        })?;
        if let Err(error) = qualify(&self.capture, &owner, &events) {
            return refuse(scope, &events.refusal, error);
        }
        let projected = (|| {
            let prototype = self.prototype(scope, &owner, &events)?;
            scope.project_named_trusted_type(ObjectPrototype::Object(&prototype), data, |bytes| {
                events.reserve(&owner, bytes)
            })
        })();
        match projected {
            Ok(value) => Ok(Completion::Return(value)),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }
}

fn stringify<'scope>(
    capture: &Capture,
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
    kind: TrustedTypeKind,
) -> Result<Completion<'scope>, Error> {
    let (owner, events) = capture.active(scope)?;
    let Some(value) = scope
        .trusted_type(arguments.receiver())?
        .filter(|brand| brand.kind() == kind)
    else {
        return type_error(scope, &ascii(b"Wrong Trusted Type receiver"));
    };
    if let Err(error) = qualify(capture, &owner, &events) {
        return refuse(scope, &events.refusal, error);
    }
    let string = capture
        .callbacks
        .conversion
        .run(scope, |cx| cx.copy_to_js(value.as_code_units()))?;
    Ok(Completion::Return(string))
}
