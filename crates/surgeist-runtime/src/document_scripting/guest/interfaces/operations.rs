//! Event operations over the original native dispatch state.

use super::*;
use surgeist_bindings_js::conversion::declarations::{
    event_init_event_0_bubbles, event_init_event_0_cancelable, event_init_event_0_type,
};

#[derive(Clone, Copy)]
enum Method {
    Path,
    Stop,
    StopImmediate,
    PreventDefault,
    Initialize,
}

#[derive(Clone, Copy)]
enum Assignment {
    CancelBubble,
    ReturnValue,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(), DocumentGuestEventError> {
    for (operation, name, length, prefix) in [
        (Method::Path, &ascii(b"composedPath")[..], 0, 0),
        (Method::Stop, &ascii(b"stopPropagation"), 0, 0),
        (
            Method::StopImmediate,
            &ascii(b"stopImmediatePropagation"),
            0,
            0,
        ),
        (Method::PreventDefault, &ascii(b"preventDefault"), 0, 0),
        (Method::Initialize, &ascii(b"initEvent"), 1, 3),
    ] {
        let operation_capture = capture(size_of::<(Capture, Method)>())?;
        let method = scope.bounded_function(
            &Callback::new(move |scope, arguments| {
                operation_capture.call_event(scope, arguments, operation)
            }),
            FunctionMetadata { name, length },
            prefix,
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(name),
            &method,
            true,
            true,
            true,
        )?;
    }

    Ok(())
}

pub(super) fn event_setter<'scope>(
    scope: &mut Scope<'scope>,
    attribute: Attribute,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
    let (assignment, name): (_, &[u16]) = match attribute {
        Attribute::CancelBubble => (Assignment::CancelBubble, &ascii(b"set cancelBubble")),
        Attribute::ReturnValue => (Assignment::ReturnValue, &ascii(b"set returnValue")),
        _ => return Ok(None),
    };
    let setter = capture(size_of::<(Capture, Assignment)>())?;
    let value = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| setter.assign_event(scope, arguments, assignment)),
        FunctionMetadata { name, length: 1 },
    )?;
    Ok(Some(value))
}

pub(super) fn nullable_target<'scope>(
    owner: &NativeOwner,
    scope: &mut Scope<'scope>,
    event: &Value<'scope>,
    target: Option<DomEventTarget>,
) -> Result<Value<'scope>, Error> {
    match target {
        Some(target) => super::super::wrapper::event_target_value(owner, scope, event, &target),
        None => scope.primitive(Primitive::Null),
    }
}

impl Capture {
    fn call_event<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        method: Method,
    ) -> Result<Completion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let receiver = arguments.receiver();
        let Some(native) = scope.native_object::<GuestEventProjection>(receiver)? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Event required"))?,
            ));
        };
        if let Err(error) = events.step() {
            return refuse(scope, &events.refusal, error);
        }
        let event = &native.payload().0;
        match method {
            Method::Path => {
                let mut path = event.composed_path();
                let result = if let Some(target) = path.next() {
                    if path.next().is_some() {
                        return Err(Error::RegistryConflict);
                    }
                    let target = super::super::wrapper::event_target_value(
                        &owner, scope, receiver, &target,
                    )?;
                    scope.array(&[target])?
                } else {
                    scope.array(&[])?
                };
                return Ok(Completion::Return(result));
            }
            Method::Stop => event.stop_propagation(),
            Method::StopImmediate => event.stop_immediate_propagation(),
            Method::PreventDefault => event.prevent_default(),
            Method::Initialize => {
                match self.initialize_event(scope, &owner, &events, &arguments, event) {
                    Ok(()) => {}
                    Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                        return Err(error);
                    }
                    Err(error) => return refuse(scope, &events.refusal, error),
                }
            }
        }
        Ok(Completion::Undefined)
    }

    fn assign_event<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        assignment: Assignment,
    ) -> Result<Completion<'scope>, Error> {
        let (_, events) = self.active(scope)?;
        let Some(native) = scope.native_object::<GuestEventProjection>(arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Event required"))?,
            ));
        };
        if let Err(error) = events.step() {
            return refuse(scope, &events.refusal, error);
        }
        let flag = match arguments.get(0) {
            Some(value) => scope.boolean(value)?,
            None => false,
        };
        match assignment {
            Assignment::CancelBubble => native.payload().0.set_cancel_bubble(flag),
            Assignment::ReturnValue => native.payload().0.set_return_value(flag),
        }
        Ok(Completion::Undefined)
    }

    fn initialize_event<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        arguments: &Arguments<'scope>,
        event: &NativeDomEvent,
    ) -> Result<(), DocumentGuestEventError> {
        let ty = event_init_event_0_type();
        let bubbles = event_init_event_0_bubbles();
        let cancelable = event_init_event_0_cancelable();
        let (ty, bubbles, cancelable) = self
            .callbacks
            .conversion
            .run(scope, |conversion| {
                Ok((
                    conversion.argument(&ty, arguments.get(ty.index()))?,
                    conversion.argument(&bubbles, arguments.get(bubbles.index()))?,
                    conversion.argument(&cancelable, arguments.get(cancelable.index()))?,
                ))
            })
            .map_err(Error::from)?;
        let (
            ArgumentValue::Present(ty),
            ArgumentValue::Present(bubbles),
            ArgumentValue::Present(cancelable),
        ) = (ty.get(), bubbles.get(), cancelable.get())
        else {
            return Err(Error::BackendUnavailable.into());
        };
        events.initialize_projection(
            owner,
            scope,
            arguments.receiver(),
            event,
            ty.as_code_units(),
            [*bubbles, *cancelable],
        )
    }
}
