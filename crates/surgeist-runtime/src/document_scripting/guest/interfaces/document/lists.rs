//! The pinned OL, UL and Menu modern and obsolete property reflections.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

const COMPACT: reflection::Member = (
    &ascii(b"compact"),
    &ascii(b"get compact"),
    &ascii(b"set compact"),
    &ascii(b"compact"),
    Reflection::Boolean,
);

const TYPE: reflection::Member = (
    &ascii(b"type"),
    &ascii(b"get type"),
    &ascii(b"set type"),
    &ascii(b"type"),
    Reflection::String,
);

const OL: &[reflection::Member] = &[
    (
        &ascii(b"reversed"),
        &ascii(b"get reversed"),
        &ascii(b"set reversed"),
        &ascii(b"reversed"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"start"),
        &ascii(b"get start"),
        &ascii(b"set start"),
        &ascii(b"start"),
        Reflection::SignedLongOne,
    ),
    TYPE,
    COMPACT,
];

const UL: &[reflection::Member] = &[COMPACT, TYPE];
const MENU: &[reflection::Member] = &[COMPACT];

pub(super) fn install_ol<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::OList, OL)
}

pub(super) fn install_ul<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::UList, UL)
}

pub(super) fn install_menu<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Menu, MENU)
}
