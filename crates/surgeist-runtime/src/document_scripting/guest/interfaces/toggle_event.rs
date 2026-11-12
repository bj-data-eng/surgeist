//! ToggleEvent construction and observation in the attached Window.

use super::*;
use crate::dom_event::AuthoredToggleInit;
use document::{GuestNode, GuestNodeProjection};
use surgeist_bindings_js::conversion::declarations::toggle_event_constructor_0_type;

const SOURCE_REFERENCE: u32 = 1;

#[derive(Clone, Copy)]
enum Member {
    OldState,
    NewState,
    Source,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    event: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    let event_constructor = scope.get(event, PropertyKey::String(CONSTRUCTOR))?;
    let fallback = capture(size_of::<Capture>())?;
    let constructor = capture(size_of::<Capture>())?;
    let callback = ConstructorCallback::bounded(
        2,
        move |scope| {
            let (_, events) = fallback.active(scope)?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            scope.open_callback_object(&public.toggle_event)
        },
        move |scope, arguments| construct(scope, arguments, &constructor),
    );
    let (_, prototype) = document::install_derived_interface(
        scope,
        event,
        &event_constructor,
        FunctionMetadata {
            name: &ascii(b"ToggleEvent"),
            length: 1,
        },
        &callback,
    )?;
    for (member, key, name) in [
        (
            Member::OldState,
            &ascii(b"oldState")[..],
            &ascii(b"get oldState")[..],
        ),
        (
            Member::NewState,
            &ascii(b"newState")[..],
            &ascii(b"get newState")[..],
        ),
        (
            Member::Source,
            &ascii(b"source")[..],
            &ascii(b"get source")[..],
        ),
    ] {
        let capture = capture(size_of::<(Capture, Member)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| get(scope, arguments, &capture, member)),
            FunctionMetadata { name, length: 0 },
        )?;
        define_accessor(scope, &prototype, key, &getter, None, true)?;
    }
    Ok(prototype)
}

fn construct<'scope>(
    scope: &mut Scope<'scope>,
    arguments: ConstructorArguments<'scope>,
    capture: &Capture,
) -> Result<ConstructorCompletion<'scope>, Error> {
    let (owner, events) = capture.active(scope)?;
    let result = construct_native(scope, arguments, capture, &owner, &events);
    match result {
        Ok(value) => Ok(ConstructorCompletion::Return(value)),
        Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => Err(error),
        Err(error) => refuse(scope, &events.refusal, error),
    }
}

fn construct_native<'scope>(
    scope: &mut Scope<'scope>,
    arguments: ConstructorArguments<'scope>,
    capture: &Capture,
    owner: &NativeOwner,
    events: &GuestEvents,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    let ty = toggle_event_constructor_0_type();
    let (ty, init) = capture
        .callbacks
        .conversion
        .run(scope, |conversion| {
            let ty = conversion.argument(&ty, arguments.get(ty.index()))?;
            let init = conversion.toggle_event_init(
                arguments.get(1),
                |projection: &GuestNodeProjection| match projection.id()? {
                    GuestNode::Element(node) => Some(node),
                    _ => None,
                },
            )?;
            Ok((ty, init))
        })
        .map_err(Error::from)?;
    let ArgumentValue::Present(ty) = ty.get() else {
        return Err(Error::BackendUnavailable.into());
    };
    let selected = arguments.select_prototype(scope)?;
    scope.check_active()?;
    events.step()?;
    let native = capture.callbacks.with_native(owner, |session, global| {
        let init = AuthoredToggleInit {
            event: DomEventInit {
                bubbles: init.bubbles,
                cancelable: init.cancelable,
                composed: init.composed,
            },
            old_state: init
                .old_state
                .as_ref()
                .map_or(&[], |state| state.get().as_code_units()),
            new_state: init
                .new_state
                .as_ref()
                .map_or(&[], |state| state.get().as_code_units()),
            source: init.source.as_ref().map(|(node, _)| *node),
        };
        session
            .hosted_execution()
            .create_toggle_event(
                &mut session.dom(),
                global,
                ty.as_code_units(),
                init,
                &owner.cancellation,
            )
            .map_err(DocumentGuestEventError::Construction)
    })?;
    let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
    let trusted = public.trusted_getter(scope)?;
    let prototype = ObjectPrototype::Object(selected.prototype());
    let payload = GuestEventProjection(native);
    let fund = |bytes| events.reserve(owner, bytes);
    let value = match init.source {
        Some((_, source)) => {
            events.step()?;
            scope.project_native_object_with_reference(
                prototype,
                payload,
                SOURCE_REFERENCE,
                &source,
                fund,
            )?
        }
        None => scope.project_native_object(prototype, payload, fund)?,
    };
    expose_trust(scope, &value, &trusted)?;
    Ok(value)
}

fn get<'scope>(
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
    capture: &Capture,
    member: Member,
) -> Result<Completion<'scope>, Error> {
    let (owner, events) = capture.active(scope)?;
    let projection = scope.native_object::<GuestEventProjection>(arguments.receiver())?;
    let Some((event, toggle)) = projection.as_ref().and_then(|projection| {
        let event = &projection.payload().0;
        Some((event, event.toggle_event()?))
    }) else {
        return Ok(Completion::Throw(
            scope.type_error(&ascii(b"ToggleEvent required"))?,
        ));
    };
    let result = (|| {
        events.step()?;
        match member {
            Member::OldState | Member::NewState => {
                let units = match member {
                    Member::OldState => toggle.old_state(),
                    _ => toggle.new_state(),
                };
                Ok(capture
                    .callbacks
                    .conversion
                    .run(scope, |conversion| conversion.copy_to_js(units))
                    .map_err(Error::from)?)
            }
            Member::Source => {
                let Some(source) = toggle.source() else {
                    return Ok(scope.primitive(Primitive::Null)?);
                };
                let against = event.current_target().and_then(|target| target.node());
                let adjusted = capture.callbacks.with_read(&owner, &events, |read, _| {
                    read.snapshot
                        .retarget_node_with(source, against, || read.step())
                })?;
                if adjusted == source {
                    events.step()?;
                    return Ok(scope
                        .native_object_reference::<GuestEventProjection>(
                            arguments.receiver(),
                            SOURCE_REFERENCE,
                        )?
                        .ok_or(DocumentScriptNativeError::ProjectionCustody)?);
                }
                events
                    .public
                    .as_ref()
                    .ok_or(Error::ExpiredIdentity)?
                    .project_node(scope, &owner, &events, adjusted)
            }
        }
    })();
    document::complete(scope, &events, result)
}
