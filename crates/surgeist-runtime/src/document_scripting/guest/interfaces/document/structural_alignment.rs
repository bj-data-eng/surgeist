//! The pinned Div, Paragraph and Heading obsolete align reflections.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

const MEMBERS: &[reflection::Member] = &[(
    &ascii(b"align"),
    &ascii(b"get align"),
    &ascii(b"set align"),
    &ascii(b"align"),
    Reflection::String,
)];

pub(super) fn install_div<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Div, MEMBERS)
}

pub(super) fn install_paragraph<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Paragraph, MEMBERS)
}

pub(super) fn install_heading<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Heading, MEMBERS)
}
