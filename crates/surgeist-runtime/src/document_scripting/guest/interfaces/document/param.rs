//! The obsolete HTMLParamElement's source-qualified raw string reflections.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_dom::DomNode;

const MEMBERS: &[reflection::Member] = &[
    (
        &ascii(b"name"),
        &ascii(b"get name"),
        &ascii(b"set name"),
        &ascii(b"name"),
        Reflection::String,
    ),
    (
        &ascii(b"value"),
        &ascii(b"get value"),
        &ascii(b"set value"),
        &ascii(b"value"),
        Reflection::String,
    ),
    (
        &ascii(b"type"),
        &ascii(b"get type"),
        &ascii(b"set type"),
        &ascii(b"type"),
        Reflection::String,
    ),
    (
        &ascii(b"valueType"),
        &ascii(b"get valueType"),
        &ascii(b"set valueType"),
        &ascii(b"valuetype"),
        Reflection::String,
    ),
];

pub(super) fn is_param(node: DomNode<'_>) -> bool {
    // The obsolete param name is outside KnownHtmlElement's modern inventory.
    node.as_element().is_some_and(|element| {
        let name = element.name();
        !element.has_failed_construction_interface()
            && name.local_name().code_units() == ascii(b"param")
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
    reflection::install(scope, prototype, capture, Receiver::Param, MEMBERS)
}
