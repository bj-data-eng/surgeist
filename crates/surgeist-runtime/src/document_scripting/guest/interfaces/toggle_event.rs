//! The original native ToggleEvent payload in the attached Window.

use super::*;

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
    let (_, prototype) = document::install_derived_interface(
        scope,
        event,
        &event_constructor,
        FunctionMetadata {
            name: &ascii(b"ToggleEvent"),
            length: 1,
        },
        &document::illegal_constructor(),
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

fn get<'scope>(
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
    capture: &Capture,
    member: Member,
) -> Result<Completion<'scope>, Error> {
    let (_, events) = capture.active(scope)?;
    let toggle = scope
        .native_object::<GuestEventProjection>(arguments.receiver())?
        .and_then(|event| event.payload().0.toggle_event().copied());
    let Some(toggle) = toggle else {
        return Ok(Completion::Throw(
            scope.type_error(&ascii(b"ToggleEvent required"))?,
        ));
    };
    let result = (|| {
        events.step()?;
        match member {
            Member::OldState => Ok(state_value(scope, toggle.old_state())?),
            Member::NewState => Ok(state_value(scope, toggle.new_state())?),
            Member::Source => match toggle.source() {
                None => Ok(scope.primitive(Primitive::Null)?),
                // A non-null source requires retargeting against currentTarget.
                Some(_) => Err(DocumentScriptNativeError::ProjectionCustody.into()),
            },
        }
    })();
    document::complete(scope, &events, result)
}

fn state_value<'scope>(scope: &mut Scope<'scope>, state: &str) -> Result<Value<'scope>, Error> {
    // Native producers provide the bounded state spellings "open" and "closed".
    const MAX_STATE_UNITS: usize = "closed".len();
    let mut units = [0; MAX_STATE_UNITS];
    let mut remaining = units.iter_mut();
    for unit in state.encode_utf16() {
        *remaining.next().ok_or(Error::BackendUnavailable)? = unit;
    }
    let length = MAX_STATE_UNITS - remaining.len();
    scope.string(units.get(..length).ok_or(Error::BackendUnavailable)?)
}
