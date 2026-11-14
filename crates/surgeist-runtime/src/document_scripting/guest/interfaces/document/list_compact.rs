//! The pinned DList and Directory compact boolean reflections.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_dom::DomNode;

const MEMBERS: &[reflection::Member] = &[(
    &ascii(b"compact"),
    &ascii(b"get compact"),
    &ascii(b"set compact"),
    &ascii(b"compact"),
    Reflection::Boolean,
)];

pub(super) fn is_directory(node: DomNode<'_>) -> bool {
    // The obsolete dir name is outside KnownHtmlElement's modern inventory.
    node.as_element().is_some_and(|element| {
        let name = element.name();
        !element.has_failed_construction_interface()
            && name.local_name().code_units() == ascii(b"dir")
            && name
                .namespace_uri()
                .is_some_and(|uri| uri.code_units() == element::HTML_NAMESPACE)
    })
}

pub(super) fn install_dlist<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::DList, MEMBERS)
}

pub(super) fn install_directory<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Directory, MEMBERS)
}
