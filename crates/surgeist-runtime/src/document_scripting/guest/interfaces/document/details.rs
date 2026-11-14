//! Details reflections through the original Source group and toggle hooks.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

const MEMBERS: &[reflection::Member] = &[
    (
        &ascii(b"name"),
        &ascii(b"get name"),
        &ascii(b"set name"),
        &ascii(b"name"),
        Reflection::String,
    ),
    (
        &ascii(b"open"),
        &ascii(b"get open"),
        &ascii(b"set open"),
        &ascii(b"open"),
        Reflection::Boolean,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Details, MEMBERS)
}
