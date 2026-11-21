//! Raw content-attribute reflections of the HTML Body interface.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

const MEMBERS: &[reflection::Member] = &[
    (
        &ascii(b"text"),
        &ascii(b"get text"),
        &ascii(b"set text"),
        &ascii(b"text"),
        Reflection::StringNullToEmpty,
    ),
    (
        &ascii(b"link"),
        &ascii(b"get link"),
        &ascii(b"set link"),
        &ascii(b"link"),
        Reflection::StringNullToEmpty,
    ),
    (
        &ascii(b"vLink"),
        &ascii(b"get vLink"),
        &ascii(b"set vLink"),
        &ascii(b"vlink"),
        Reflection::StringNullToEmpty,
    ),
    (
        &ascii(b"aLink"),
        &ascii(b"get aLink"),
        &ascii(b"set aLink"),
        &ascii(b"alink"),
        Reflection::StringNullToEmpty,
    ),
    (
        &ascii(b"bgColor"),
        &ascii(b"get bgColor"),
        &ascii(b"set bgColor"),
        &ascii(b"bgcolor"),
        Reflection::StringNullToEmpty,
    ),
    (
        &ascii(b"background"),
        &ascii(b"get background"),
        &ascii(b"set background"),
        &ascii(b"background"),
        Reflection::String,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Body, MEMBERS)
}
