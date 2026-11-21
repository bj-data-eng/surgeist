//! Local Button attributes over the canonical element and its current Source segment.

use super::{
    custom_validity::CustomValidityControl,
    element::HTML_NAMESPACE,
    form_owner::{self, FormOwnerGetter},
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_dom::tree::DomNode;
use surgeist_html::bounded::attributes::{HtmlButtonSubmitterFact, button_type_keyword_with};

const BEFORE_FORM: &[reflection::Member] = &[
    (
        &ascii(b"command"),
        &ascii(b"get command"),
        &ascii(b"set command"),
        &ascii(b"command"),
        Reflection::ButtonCommand,
    ),
    (
        &ascii(b"disabled"),
        &ascii(b"get disabled"),
        &ascii(b"set disabled"),
        &ascii(b"disabled"),
        Reflection::Boolean,
    ),
];

const AFTER_FORM: &[reflection::Member] = &[
    (
        &ascii(b"formEnctype"),
        &ascii(b"get formEnctype"),
        &ascii(b"set formEnctype"),
        &ascii(b"formenctype"),
        Reflection::FormEnctypeOverride,
    ),
    (
        &ascii(b"formMethod"),
        &ascii(b"get formMethod"),
        &ascii(b"set formMethod"),
        &ascii(b"formmethod"),
        Reflection::FormMethodOverride,
    ),
    (
        &ascii(b"formNoValidate"),
        &ascii(b"get formNoValidate"),
        &ascii(b"set formNoValidate"),
        &ascii(b"formnovalidate"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"formTarget"),
        &ascii(b"get formTarget"),
        &ascii(b"set formTarget"),
        &ascii(b"formtarget"),
        Reflection::String,
    ),
    (
        &ascii(b"name"),
        &ascii(b"get name"),
        &ascii(b"set name"),
        &ascii(b"name"),
        Reflection::String,
    ),
    (
        &ascii(b"type"),
        &ascii(b"get type"),
        &ascii(b"set type"),
        &ascii(b"type"),
        Reflection::ButtonType,
    ),
    (
        &ascii(b"value"),
        &ascii(b"get value"),
        &ascii(b"set value"),
        &ascii(b"value"),
        Reflection::String,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Button, BEFORE_FORM)?;
    form_owner::install(scope, prototype, capture, FormOwnerGetter::Button)?;
    reflection::install(scope, prototype, capture, Receiver::Button, AFTER_FORM)?;
    CustomValidityControl::Button.install(scope, prototype, capture)
}

pub(super) fn type_keyword(
    read: &NativeRead<'_>,
    node: DomNode<'_>,
    raw: Option<&[u16]>,
) -> Result<&'static str, DocumentGuestEventError> {
    button_type_keyword_with(
        raw,
        |_| read.step(),
        |fact| {
            let name = match fact {
                HtmlButtonSubmitterFact::CommandAttributePresent => &ascii(b"command")[..],
                HtmlButtonSubmitterFact::CommandforAttributePresent => &ascii(b"commandfor")[..],
                HtmlButtonSubmitterFact::ParentIsSelect => {
                    read.step()?;
                    let Some(parent) = node.parent() else {
                        return Ok(false);
                    };
                    let parent = read.node(parent)?;
                    read.step()?;
                    return Ok(parent.as_element().is_some_and(|parent| {
                        let name = parent.name();
                        name.local_name().code_units() == ascii(b"select")
                            && name
                                .namespace_uri()
                                .is_some_and(|uri| uri.code_units() == HTML_NAMESPACE)
                    }));
                }
            };
            let attributes = node.attributes().ok_or(DomError::WrongNodeKind)?;
            Ok(attributes
                .get_null_namespace_attribute_with(name, || read.step())?
                .is_some())
        },
    )
}
