//! Local reflections on the canonical HTML Iframe.

use super::{
    element::install_html_token_list,
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_html::elements::token_list::HtmlTokenListReflection;

const BEFORE_SANDBOX: &[reflection::Member] = &[(
    &ascii(b"name"),
    &ascii(b"get name"),
    &ascii(b"set name"),
    &ascii(b"name"),
    Reflection::String,
)];

const AFTER_SANDBOX: &[reflection::Member] = &[
    (
        &ascii(b"allow"),
        &ascii(b"get allow"),
        &ascii(b"set allow"),
        &ascii(b"allow"),
        Reflection::String,
    ),
    (
        &ascii(b"allowFullscreen"),
        &ascii(b"get allowFullscreen"),
        &ascii(b"set allowFullscreen"),
        &ascii(b"allowfullscreen"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"width"),
        &ascii(b"get width"),
        &ascii(b"set width"),
        &ascii(b"width"),
        Reflection::String,
    ),
    (
        &ascii(b"height"),
        &ascii(b"get height"),
        &ascii(b"set height"),
        &ascii(b"height"),
        Reflection::String,
    ),
    (
        &ascii(b"referrerPolicy"),
        &ascii(b"get referrerPolicy"),
        &ascii(b"set referrerPolicy"),
        &ascii(b"referrerpolicy"),
        Reflection::ReferrerPolicy,
    ),
    (
        &ascii(b"loading"),
        &ascii(b"get loading"),
        &ascii(b"set loading"),
        &ascii(b"loading"),
        Reflection::Loading,
    ),
];

const OBSOLETE: &[reflection::Member] = &[
    (
        &ascii(b"align"),
        &ascii(b"get align"),
        &ascii(b"set align"),
        &ascii(b"align"),
        Reflection::String,
    ),
    (
        &ascii(b"scrolling"),
        &ascii(b"get scrolling"),
        &ascii(b"set scrolling"),
        &ascii(b"scrolling"),
        Reflection::String,
    ),
    (
        &ascii(b"frameBorder"),
        &ascii(b"get frameBorder"),
        &ascii(b"set frameBorder"),
        &ascii(b"frameborder"),
        Reflection::String,
    ),
    (
        &ascii(b"marginHeight"),
        &ascii(b"get marginHeight"),
        &ascii(b"set marginHeight"),
        &ascii(b"marginheight"),
        Reflection::StringNullToEmpty,
    ),
    (
        &ascii(b"marginWidth"),
        &ascii(b"get marginWidth"),
        &ascii(b"set marginWidth"),
        &ascii(b"marginwidth"),
        Reflection::StringNullToEmpty,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Iframe, BEFORE_SANDBOX)?;
    install_html_token_list(
        scope,
        prototype,
        capture,
        HtmlTokenListReflection::IframeSandbox,
    )?;
    reflection::install(scope, prototype, capture, Receiver::Iframe, AFTER_SANDBOX)?;
    super::frame_content::install(scope, prototype, capture, Receiver::Iframe)?;
    reflection::install(scope, prototype, capture, Receiver::Iframe, OBSOLETE)
}
