//! Anchor text and scalar attributes on the native Element.

use super::{
    element::install_html_token_list,
    reflection::{self, Receiver, Reflection},
    text::collect_text,
    *,
};
use surgeist_bindings_js::conversion::types::DOM_STRING;
use surgeist_dom::source::DomSourceEdit;
use surgeist_html::elements::token_list::HtmlTokenListReflection;

const BEFORE_TEXT: &[reflection::Member] = &[
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

const AFTER_TEXT: &[reflection::Member] = &[
    (
        &ascii(b"referrerPolicy"),
        &ascii(b"get referrerPolicy"),
        &ascii(b"set referrerPolicy"),
        &ascii(b"referrerpolicy"),
        Reflection::ReferrerPolicy,
    ),
    (
        &ascii(b"coords"),
        &ascii(b"get coords"),
        &ascii(b"set coords"),
        &ascii(b"coords"),
        Reflection::String,
    ),
    (
        &ascii(b"charset"),
        &ascii(b"get charset"),
        &ascii(b"set charset"),
        &ascii(b"charset"),
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
        &ascii(b"rev"),
        &ascii(b"get rev"),
        &ascii(b"set rev"),
        &ascii(b"rev"),
        Reflection::String,
    ),
    (
        &ascii(b"shape"),
        &ascii(b"get shape"),
        &ascii(b"set shape"),
        &ascii(b"shape"),
        Reflection::String,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Anchor, BEFORE_TEXT)?;
    install_html_token_list(
        scope,
        prototype,
        capture,
        HtmlTokenListReflection::AnchorRel,
    )?;
    let read = capture(size_of::<Capture>())?;
    let write = capture(size_of::<Capture>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| read.read_anchor_text(scope, arguments)),
        FunctionMetadata {
            name: &ascii(b"get text"),
            length: 0,
        },
    )?;
    let setter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| write.write_anchor_text(scope, arguments)),
        FunctionMetadata {
            name: &ascii(b"set text"),
            length: 1,
        },
    )?;
    define_accessor(
        scope,
        prototype,
        &ascii(b"text"),
        &getter,
        Some(&setter),
        true,
    )?;
    reflection::install(scope, prototype, capture, Receiver::Anchor, AFTER_TEXT)
}

impl Capture {
    fn read_anchor_text<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Anchor;
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let text = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                collect_text(read, node).map(Some)
            })?;
            let Some(text) = text else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            Ok(Completion::Return(
                self.copy_dom_units(scope, text.code_units())?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn write_anchor_text<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Anchor;
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let qualified = self.callbacks.with_read(&owner, &events, |read, _| {
                receiver.matches(read.node(id)?, || read.step())
            })?;
            if !qualified {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            let absent;
            let input = match arguments.get(0) {
                Some(input) => input,
                None => {
                    absent = scope.primitive(Primitive::Undefined)?;
                    &absent
                }
            };
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| conversion.value(&DOM_STRING, input))
                .map_err(Error::from)?;
            scope.check_active()?;
            let qualified = self.callbacks.with_read(&owner, &events, |read, _| {
                receiver.matches(read.node(id)?, || read.step())
            })?;
            if !qualified {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            let (value, _backing) =
                copy_native_data(&owner, &events, converted.get().as_code_units())?;
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::set_text_content(id, value),
                Completion::Undefined,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}
