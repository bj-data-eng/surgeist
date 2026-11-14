//! The obsolete HTMLFontElement's source-qualified raw string reflections.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_dom::DomNode;

const MEMBERS: &[reflection::Member] = &[
    (
        &ascii(b"color"),
        &ascii(b"get color"),
        &ascii(b"set color"),
        &ascii(b"color"),
        Reflection::StringNullToEmpty,
    ),
    (
        &ascii(b"face"),
        &ascii(b"get face"),
        &ascii(b"set face"),
        &ascii(b"face"),
        Reflection::String,
    ),
    (
        &ascii(b"size"),
        &ascii(b"get size"),
        &ascii(b"set size"),
        &ascii(b"size"),
        Reflection::String,
    ),
];

pub(super) fn is_font(node: DomNode<'_>) -> bool {
    // The obsolete font name is outside KnownHtmlElement's modern inventory.
    node.as_element().is_some_and(|element| {
        let name = element.name();
        !element.has_failed_construction_interface()
            && name.local_name().code_units() == ascii(b"font")
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
    reflection::install(scope, prototype, capture, Receiver::Font, MEMBERS)
}
