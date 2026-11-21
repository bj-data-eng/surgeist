//! Raw content-attribute reflections of the HTML Frameset interface.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_dom::tree::DomNode;

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

pub(super) fn is_frameset(node: DomNode<'_>) -> bool {
    node.as_element().is_some_and(|element| {
        let name = element.name();
        !element.has_failed_construction_interface()
            && name.local_name().code_units() == ascii(b"frameset")
            && name
                .namespace_uri()
                .is_some_and(|uri| uri.code_units() == element::HTML_NAMESPACE)
    })
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Frameset, MEMBERS)
}
