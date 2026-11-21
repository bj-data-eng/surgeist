//! Canvas dimensions over the native none-context bitmap state.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_html::elements::media::HtmlCanvasDimension;

const DIMENSIONS: &[reflection::Member] = &[
    (
        &ascii(b"width"),
        &ascii(b"get width"),
        &ascii(b"set width"),
        &ascii(b"width"),
        Reflection::CanvasDimension(HtmlCanvasDimension::Width),
    ),
    (
        &ascii(b"height"),
        &ascii(b"get height"),
        &ascii(b"set height"),
        &ascii(b"height"),
        Reflection::CanvasDimension(HtmlCanvasDimension::Height),
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Canvas, DIMENSIONS)
}
