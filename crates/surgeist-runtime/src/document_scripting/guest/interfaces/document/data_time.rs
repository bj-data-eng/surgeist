//! The pinned Data and Time raw string reflections.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

const DATA_MEMBERS: &[reflection::Member] = &[(
    &ascii(b"value"),
    &ascii(b"get value"),
    &ascii(b"set value"),
    &ascii(b"value"),
    Reflection::String,
)];

const TIME_MEMBERS: &[reflection::Member] = &[(
    &ascii(b"dateTime"),
    &ascii(b"get dateTime"),
    &ascii(b"set dateTime"),
    &ascii(b"datetime"),
    Reflection::String,
)];

pub(super) fn install_data<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Data, DATA_MEMBERS)
}

pub(super) fn install_time<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Time, TIME_MEMBERS)
}
