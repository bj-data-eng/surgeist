//! The pinned signed width reflection on Pre and its standard interface aliases.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_dom::tree::DomNode;

const MEMBERS: &[reflection::Member] = &[(
    &ascii(b"width"),
    &ascii(b"get width"),
    &ascii(b"set width"),
    &ascii(b"width"),
    Reflection::SignedLongZero,
)];

pub(super) fn is_pre(
    node: DomNode<'_>,
    step: impl FnMut() -> Result<(), DocumentGuestEventError>,
) -> Result<bool, DocumentGuestEventError> {
    let Some(element) = node.as_element() else {
        return Ok(false);
    };
    let name = element.name();
    if element.has_failed_construction_interface()
        || !name
            .namespace_uri()
            .is_some_and(|uri| uri.code_units() == element::HTML_NAMESPACE)
    {
        return Ok(false);
    }
    // HTML877d #elements-in-the-dom assigns listing and xmp the same interface
    // as pre. Consume the existing interface selector rather than a second list.
    Ok(element::html_interface(name.local_name(), step)? == "HTMLPreElement")
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Pre, MEMBERS)
}
