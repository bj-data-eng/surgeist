//! HTMLMetaElement and its obsolete scheme member in pinned declaration order.

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
        &ascii(b"httpEquiv"),
        &ascii(b"get httpEquiv"),
        &ascii(b"set httpEquiv"),
        &ascii(b"http-equiv"),
        Reflection::String,
    ),
    (
        &ascii(b"content"),
        &ascii(b"get content"),
        &ascii(b"set content"),
        &ascii(b"content"),
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
        &ascii(b"scheme"),
        &ascii(b"get scheme"),
        &ascii(b"set scheme"),
        &ascii(b"scheme"),
        Reflection::String,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Meta, MEMBERS)
}
