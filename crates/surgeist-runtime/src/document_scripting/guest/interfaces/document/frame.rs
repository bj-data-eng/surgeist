//! Local content attributes on the canonical obsolete HTML Frame.

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
        &ascii(b"scrolling"),
        &ascii(b"get scrolling"),
        &ascii(b"set scrolling"),
        &ascii(b"scrolling"),
        Reflection::String,
    ),
    (
        &ascii(b"frameBorder"),
        &ascii(b"get frameBorder"),
        &ascii(b"set frameBorder"),
        &ascii(b"frameborder"),
        Reflection::String,
    ),
    (
        &ascii(b"noResize"),
        &ascii(b"get noResize"),
        &ascii(b"set noResize"),
        &ascii(b"noresize"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"marginHeight"),
        &ascii(b"get marginHeight"),
        &ascii(b"set marginHeight"),
        &ascii(b"marginheight"),
        Reflection::StringNullToEmpty,
    ),
    (
        &ascii(b"marginWidth"),
        &ascii(b"get marginWidth"),
        &ascii(b"set marginWidth"),
        &ascii(b"marginwidth"),
        Reflection::StringNullToEmpty,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Frame, MEMBERS)
}
