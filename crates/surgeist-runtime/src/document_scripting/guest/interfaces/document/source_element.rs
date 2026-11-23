//! Local content attributes on the canonical HTML Source.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

const MEMBERS: &[reflection::Member] = &[
    (
        &ascii(b"type"),
        &ascii(b"get type"),
        &ascii(b"set type"),
        &ascii(b"type"),
        Reflection::String,
    ),
    (
        &ascii(b"srcset"),
        &ascii(b"get srcset"),
        &ascii(b"set srcset"),
        &ascii(b"srcset"),
        Reflection::UsvString,
    ),
    (
        &ascii(b"sizes"),
        &ascii(b"get sizes"),
        &ascii(b"set sizes"),
        &ascii(b"sizes"),
        Reflection::String,
    ),
    (
        &ascii(b"media"),
        &ascii(b"get media"),
        &ascii(b"set media"),
        &ascii(b"media"),
        Reflection::String,
    ),
    (
        &ascii(b"width"),
        &ascii(b"get width"),
        &ascii(b"set width"),
        &ascii(b"width"),
        Reflection::UnsignedLong,
    ),
    (
        &ascii(b"height"),
        &ascii(b"get height"),
        &ascii(b"set height"),
        &ascii(b"height"),
        Reflection::UnsignedLong,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Source, MEMBERS)
}
