//! Local attributes on the canonical HTML Area element.

use super::{
    element::install_html_token_list,
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_html::elements::token_list::HtmlTokenListReflection;

const BEFORE_REL_LIST: &[reflection::Member] = &[
    (
        &ascii(b"alt"),
        &ascii(b"get alt"),
        &ascii(b"set alt"),
        &ascii(b"alt"),
        Reflection::String,
    ),
    (
        &ascii(b"coords"),
        &ascii(b"get coords"),
        &ascii(b"set coords"),
        &ascii(b"coords"),
        Reflection::String,
    ),
    (
        &ascii(b"shape"),
        &ascii(b"get shape"),
        &ascii(b"set shape"),
        &ascii(b"shape"),
        Reflection::String,
    ),
    (
        &ascii(b"download"),
        &ascii(b"get download"),
        &ascii(b"set download"),
        &ascii(b"download"),
        Reflection::String,
    ),
    (
        &ascii(b"ping"),
        &ascii(b"get ping"),
        &ascii(b"set ping"),
        &ascii(b"ping"),
        Reflection::UsvString,
    ),
    (
        &ascii(b"rel"),
        &ascii(b"get rel"),
        &ascii(b"set rel"),
        &ascii(b"rel"),
        Reflection::String,
    ),
];

const AFTER_REL_LIST: &[reflection::Member] = &[
    (
        &ascii(b"referrerPolicy"),
        &ascii(b"get referrerPolicy"),
        &ascii(b"set referrerPolicy"),
        &ascii(b"referrerpolicy"),
        Reflection::ReferrerPolicy,
    ),
    (
        &ascii(b"noHref"),
        &ascii(b"get noHref"),
        &ascii(b"set noHref"),
        &ascii(b"nohref"),
        Reflection::Boolean,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Area, BEFORE_REL_LIST)?;
    install_html_token_list(scope, prototype, capture, HtmlTokenListReflection::AreaRel)?;
    reflection::install(scope, prototype, capture, Receiver::Area, AFTER_REL_LIST)
}
