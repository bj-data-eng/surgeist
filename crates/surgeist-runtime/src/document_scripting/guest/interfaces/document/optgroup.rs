//! OptGroup reflections through the canonical Source attribute operations.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

const MEMBERS: &[reflection::Member] = &[
    (
        &ascii(b"disabled"),
        &ascii(b"get disabled"),
        &ascii(b"set disabled"),
        &ascii(b"disabled"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"label"),
        &ascii(b"get label"),
        &ascii(b"set label"),
        &ascii(b"label"),
        Reflection::String,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::OptGroup, MEMBERS)
}
