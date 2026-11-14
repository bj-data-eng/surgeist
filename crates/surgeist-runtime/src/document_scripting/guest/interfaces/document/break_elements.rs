//! The pinned BR and HR reflected members, including their obsolete partials.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

const BR_MEMBERS: &[reflection::Member] = &[(
    &ascii(b"clear"),
    &ascii(b"get clear"),
    &ascii(b"set clear"),
    &ascii(b"clear"),
    Reflection::String,
)];

const HR_MEMBERS: &[reflection::Member] = &[
    (
        &ascii(b"align"),
        &ascii(b"get align"),
        &ascii(b"set align"),
        &ascii(b"align"),
        Reflection::String,
    ),
    (
        &ascii(b"color"),
        &ascii(b"get color"),
        &ascii(b"set color"),
        &ascii(b"color"),
        Reflection::String,
    ),
    (
        &ascii(b"noShade"),
        &ascii(b"get noShade"),
        &ascii(b"set noShade"),
        &ascii(b"noshade"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"size"),
        &ascii(b"get size"),
        &ascii(b"set size"),
        &ascii(b"size"),
        Reflection::String,
    ),
    (
        &ascii(b"width"),
        &ascii(b"get width"),
        &ascii(b"set width"),
        &ascii(b"width"),
        Reflection::String,
    ),
];

pub(super) fn install_br<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Br, BR_MEMBERS)
}

pub(super) fn install_hr<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Hr, HR_MEMBERS)
}
