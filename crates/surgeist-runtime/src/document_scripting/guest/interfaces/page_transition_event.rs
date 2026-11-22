//! The standard PageTransitionEvent interface on the attached Window.

use super::*;
use surgeist_bindings_js::conversion::declarations::{
    page_transition_event_constructor_0_event_init_dict, page_transition_event_constructor_0_type,
};

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    event: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    let event_constructor = scope.get(event, PropertyKey::String(CONSTRUCTOR))?;
    let fallback = capture(size_of::<Capture>())?;
    let constructor = constructor_capture(capture, size_of::<Capture>())?;
    let callback = ConstructorCallback::bounded(
        2,
        move |scope| {
            let (_, events) = fallback.active(scope)?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            scope.open_callback_object(&public.page_transition_event)
        },
        move |scope, arguments| construct(scope, arguments, &constructor),
    );
    let (_, prototype) = document::install_derived_interface(
        scope,
        event,
        &event_constructor,
        FunctionMetadata {
            name: &ascii(b"PageTransitionEvent"),
            length: 1,
        },
        &callback,
    )?;
    let capture = capture(size_of::<Capture>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| get_persisted(scope, arguments, &capture)),
        FunctionMetadata {
            name: &ascii(b"get persisted"),
            length: 0,
        },
    )?;
    define_accessor(scope, &prototype, &ascii(b"persisted"), &getter, None, true)?;
    Ok(prototype)
}

fn construct<'scope>(
    scope: &mut Scope<'scope>,
    arguments: ConstructorArguments<'scope>,
    capture: &Capture,
) -> Result<ConstructorCompletion<'scope>, Error> {
    let (owner, events) = capture.active(scope)?;
    let result = (|| -> Result<_, DocumentGuestEventError> {
        let ty = page_transition_event_constructor_0_type();
        let init = page_transition_event_constructor_0_event_init_dict();
        let (ty, init) = capture
            .callbacks
            .conversion
            .run(scope, |conversion| {
                let ty = conversion.argument(&ty, arguments.get(ty.index()))?;
                let init = conversion.argument(&init, arguments.get(init.index()))?;
                Ok((ty, init))
            })
            .map_err(Error::from)?;
        let (ArgumentValue::Present(ty), ArgumentValue::Present(init)) = (ty.get(), init.get())
        else {
            return Err(Error::BackendUnavailable.into());
        };
        let selected = arguments.select_prototype(scope)?;
        scope.check_active()?;
        events.step()?;
        let native = capture.callbacks.with_native(&owner, |session, global| {
            session
                .hosted_execution()
                .create_page_transition_event(
                    &mut session.dom(),
                    global,
                    ty.as_code_units(),
                    DomEventInit {
                        bubbles: init.bubbles,
                        cancelable: init.cancelable,
                        composed: init.composed,
                    },
                    init.persisted,
                    &owner.cancellation,
                )
                .map_err(DocumentGuestEventError::Construction)
        })?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let trusted = public.trusted_getter(scope)?;
        let value = scope.project_native_object(
            ObjectPrototype::Object(selected.prototype()),
            GuestEventProjection(native),
            |bytes| events.reserve(&owner, bytes),
        )?;
        expose_trust(scope, &value, &trusted)?;
        Ok(value)
    })();
    match result {
        Ok(value) => Ok(ConstructorCompletion::Return(value)),
        Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => Err(error),
        Err(error) => refuse(scope, &events.refusal, error),
    }
}

fn get_persisted<'scope>(
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
    capture: &Capture,
) -> Result<Completion<'scope>, Error> {
    let (_, events) = capture.active(scope)?;
    let projection = scope.native_object::<GuestEventProjection>(arguments.receiver())?;
    let Some(persisted) = projection
        .as_ref()
        .and_then(|value| value.payload().0.page_transition_persisted())
    else {
        return Ok(Completion::Throw(
            scope.type_error(&ascii(b"PageTransitionEvent required"))?,
        ));
    };
    if let Err(error) = events.step() {
        return refuse(scope, &events.refusal, error);
    }
    Ok(Completion::Return(scope.primitive(
        surgeist_bindings_js::Primitive::Boolean(persisted),
    )?))
}
