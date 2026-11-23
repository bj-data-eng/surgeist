//! Raw content-attribute reflections of the HTML Frameset interface.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

const MEMBERS: &[reflection::Member] = &[
    (
        &ascii(b"cols"),
        &ascii(b"get cols"),
        &ascii(b"set cols"),
        &ascii(b"cols"),
        Reflection::String,
    ),
    (
        &ascii(b"rows"),
        &ascii(b"get rows"),
        &ascii(b"set rows"),
        &ascii(b"rows"),
        Reflection::String,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Frameset, MEMBERS)
}
