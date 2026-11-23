//! Content attribute reflections for canonical Link and Style elements.

use super::{
    element::install_html_token_list,
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_html::elements::token_list::HtmlTokenListReflection;

const BEFORE_REL_LIST: &[reflection::Member] = &[
    (
        &ascii(b"crossOrigin"),
        &ascii(b"get crossOrigin"),
        &ascii(b"set crossOrigin"),
        &ascii(b"crossorigin"),
        Reflection::CrossOrigin,
    ),
    (
        &ascii(b"rel"),
        &ascii(b"get rel"),
        &ascii(b"set rel"),
        &ascii(b"rel"),
        Reflection::String,
    ),
    (
        &ascii(b"as"),
        &ascii(b"get as"),
        &ascii(b"set as"),
        &ascii(b"as"),
        Reflection::LinkAs,
    ),
];

const AFTER_REL_LIST: &[reflection::Member] = &[
    (
        &ascii(b"media"),
        &ascii(b"get media"),
        &ascii(b"set media"),
        &ascii(b"media"),
        Reflection::String,
    ),
    (
        &ascii(b"integrity"),
        &ascii(b"get integrity"),
        &ascii(b"set integrity"),
        &ascii(b"integrity"),
        Reflection::String,
    ),
    (
        &ascii(b"hreflang"),
        &ascii(b"get hreflang"),
        &ascii(b"set hreflang"),
        &ascii(b"hreflang"),
        Reflection::String,
    ),
    (
        &ascii(b"type"),
        &ascii(b"get type"),
        &ascii(b"set type"),
        &ascii(b"type"),
        Reflection::String,
    ),
];

const AFTER_SIZES: &[reflection::Member] = &[
    (
        &ascii(b"imageSrcset"),
        &ascii(b"get imageSrcset"),
        &ascii(b"set imageSrcset"),
        &ascii(b"imagesrcset"),
        Reflection::UsvString,
    ),
    (
        &ascii(b"imageSizes"),
        &ascii(b"get imageSizes"),
        &ascii(b"set imageSizes"),
        &ascii(b"imagesizes"),
        Reflection::String,
    ),
    (
        &ascii(b"referrerPolicy"),
        &ascii(b"get referrerPolicy"),
        &ascii(b"set referrerPolicy"),
        &ascii(b"referrerpolicy"),
        Reflection::ReferrerPolicy,
    ),
];

const AFTER_BLOCKING: &[reflection::Member] = &[
    (
        &ascii(b"disabled"),
        &ascii(b"get disabled"),
        &ascii(b"set disabled"),
        &ascii(b"disabled"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"fetchPriority"),
        &ascii(b"get fetchPriority"),
        &ascii(b"set fetchPriority"),
        &ascii(b"fetchpriority"),
        Reflection::FetchPriority,
    ),
    (
        &ascii(b"charset"),
        &ascii(b"get charset"),
        &ascii(b"set charset"),
        &ascii(b"charset"),
        Reflection::String,
    ),
    (
        &ascii(b"rev"),
        &ascii(b"get rev"),
        &ascii(b"set rev"),
        &ascii(b"rev"),
        Reflection::String,
    ),
    (
        &ascii(b"target"),
        &ascii(b"get target"),
        &ascii(b"set target"),
        &ascii(b"target"),
        Reflection::String,
    ),
];

const STYLE_MEDIA: &[reflection::Member] = &[(
    &ascii(b"media"),
    &ascii(b"get media"),
    &ascii(b"set media"),
    &ascii(b"media"),
    Reflection::String,
)];

const STYLE_TYPE: &[reflection::Member] = &[(
    &ascii(b"type"),
    &ascii(b"get type"),
    &ascii(b"set type"),
    &ascii(b"type"),
    Reflection::String,
)];

pub(super) fn install_link<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Link, BEFORE_REL_LIST)?;
    install_html_token_list(scope, prototype, capture, HtmlTokenListReflection::LinkRel)?;
    reflection::install(scope, prototype, capture, Receiver::Link, AFTER_REL_LIST)?;
    install_html_token_list(
        scope,
        prototype,
        capture,
        HtmlTokenListReflection::LinkSizes,
    )?;
    reflection::install(scope, prototype, capture, Receiver::Link, AFTER_SIZES)?;
    install_html_token_list(
        scope,
        prototype,
        capture,
        HtmlTokenListReflection::LinkBlocking,
    )?;
    reflection::install(scope, prototype, capture, Receiver::Link, AFTER_BLOCKING)
}

pub(super) fn install_style<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Style, STYLE_MEDIA)?;
    install_html_token_list(
        scope,
        prototype,
        capture,
        HtmlTokenListReflection::StyleBlocking,
    )?;
    reflection::install(scope, prototype, capture, Receiver::Style, STYLE_TYPE)
}
