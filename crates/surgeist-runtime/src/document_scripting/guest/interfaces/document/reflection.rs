//! Reflected attributes over authenticated live elements and Source edits.

use super::{element::HTML_NAMESPACE, *};
use std::fmt::Write;
use surgeist_bindings_js::conversion::IntegerType;
use surgeist_bindings_js::conversion::types::{BOOLEAN, DOM_STRING};
use surgeist_bindings_js::{IntegerKind, IntegerMode, IntegerValue};
use surgeist_dom::{DomNode, DomSourceEdit, Namespace};
use surgeist_html::bounded::attributes::{
    dir_keyword_with, heading_offset_reflection_with, heading_offset_setter_value,
    popover_keyword_with, template_shadow_root_mode_keyword_with,
    template_shadow_root_slot_assignment_keyword_with,
};
use surgeist_html::vocabulary::KnownHtmlElement;

#[derive(Clone, Copy)]
pub(super) enum Reflection {
    String,
    Boolean,
    Mode,
    SlotAssignment,
    Direction,
    Popover,
    HeadingOffset,
}

#[derive(Clone, Copy)]
pub(super) enum Receiver {
    Element,
    Html,
    Template,
    Br,
    Hr,
    Meta,
    Slot,
}

impl Receiver {
    fn matches(self, node: DomNode<'_>) -> bool {
        match self {
            Self::Element => node.as_element().is_some(),
            Self::Html => node.as_element().is_some_and(|element| {
                element
                    .name()
                    .namespace_uri()
                    .is_some_and(|uri| uri.code_units() == HTML_NAMESPACE)
            }),
            Self::Template => node.as_template().is_some(),
            Self::Br => html_element_is(node, KnownHtmlElement::Br),
            Self::Hr => html_element_is(node, KnownHtmlElement::Hr),
            Self::Meta => html_element_is(node, KnownHtmlElement::Meta),
            Self::Slot => super::slot::is_slot(node),
        }
    }

    fn error_message(self) -> &'static [u16] {
        match self {
            Self::Element => const { &ascii(b"Element required") },
            Self::Html => const { &ascii(b"HTMLElement required") },
            Self::Template => const { &ascii(b"HTMLTemplateElement required") },
            Self::Br => const { &ascii(b"HTMLBRElement required") },
            Self::Hr => const { &ascii(b"HTMLHRElement required") },
            Self::Meta => const { &ascii(b"HTMLMetaElement required") },
            Self::Slot => const { &ascii(b"HTMLSlotElement required") },
        }
    }
}

fn html_element_is(node: DomNode<'_>, expected: KnownHtmlElement) -> bool {
    node.as_element().is_some_and(|element| {
        !element.has_failed_construction_interface()
            && KnownHtmlElement::from_name(element.name()) == Some(expected)
    })
}

pub(super) type Member = (
    &'static [u16],
    &'static [u16],
    &'static [u16],
    &'static [u16],
    Reflection,
);

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    receiver: Receiver,
    members: &[Member],
) -> Result<(), DocumentGuestEventError> {
    for &(member, get_name, set_name, attribute, reflection) in members {
        let get = capture(size_of::<(Capture, Receiver, &[u16], Reflection)>())?;
        let set = capture(size_of::<(Capture, Receiver, &[u16], Reflection)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                get.reflect_attribute(scope, arguments, receiver, attribute, reflection, false)
            }),
            FunctionMetadata {
                name: get_name,
                length: 0,
            },
        )?;
        let setter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                set.reflect_attribute(scope, arguments, receiver, attribute, reflection, true)
            }),
            FunctionMetadata {
                name: set_name,
                length: 1,
            },
        )?;
        scope.define_new_property(
            prototype,
            PropertyKey::String(member),
            PropertyDescriptor::Accessor {
                get: Some(&getter),
                set: Some(&setter),
                enumerable: true,
                configurable: true,
            },
        )?;
    }
    Ok(())
}

impl Capture {
    fn reflect_attribute<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
        attribute: &[u16],
        reflection: Reflection,
        write: bool,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node) {
                    return Ok(None);
                }
                if write {
                    return Ok(Some(Reflected::Write));
                }
                let attrs = node.attributes().ok_or(DomError::WrongNodeKind)?;
                let stored = attrs.get_null_namespace_attribute_with(attribute, || read.step())?;
                let units = stored.map(|value| value.value().code_units());
                Ok(Some(match reflection {
                    Reflection::Boolean => Reflected::Boolean(stored.is_some()),
                    Reflection::String => {
                        Reflected::String(stored.map(|value| value.value().clone()))
                    }
                    Reflection::Mode => Reflected::Keyword(Some(
                        template_shadow_root_mode_keyword_with(units, |_| read.step())?,
                    )),
                    Reflection::SlotAssignment => Reflected::Keyword(Some(
                        template_shadow_root_slot_assignment_keyword_with(units, |_| read.step())?,
                    )),
                    Reflection::Direction => {
                        Reflected::Keyword(Some(dir_keyword_with(units, |_| read.step())?))
                    }
                    Reflection::Popover => {
                        Reflected::Keyword(popover_keyword_with(units, |_| read.step())?)
                    }
                    Reflection::HeadingOffset => {
                        Reflected::Unsigned(heading_offset_reflection_with(units, |_| read.step())?)
                    }
                }))
            })?;
            let Some(value) = value else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            let output = match value {
                Reflected::Write => None,
                Reflected::Boolean(value) => Some(scope.primitive(Primitive::Boolean(value))?),
                Reflected::Unsigned(value) => {
                    Some(scope.primitive(Primitive::Number(f64::from(value)))?)
                }
                Reflected::String(value) => Some(self.copy_dom_units(
                    scope,
                    value.as_ref().map_or(&[], DocumentString::code_units),
                )?),
                Reflected::Keyword(None) => Some(scope.primitive(Primitive::Null)?),
                Reflected::Keyword(Some(keyword)) => {
                    let mut units = [0_u16; 6];
                    let units = units
                        .get_mut(..keyword.len())
                        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
                    for (unit, byte) in units.iter_mut().zip(keyword.bytes()) {
                        *unit = u16::from(byte);
                    }
                    Some(self.copy_dom_units(scope, units)?)
                }
            };
            if let Some(output) = output {
                return Ok(Completion::Return(output));
            }
            let absent;
            let input = match arguments.get(0) {
                Some(input) => input,
                None => {
                    absent = scope.primitive(Primitive::Undefined)?;
                    &absent
                }
            };
            let value = if matches!(reflection, Reflection::Boolean) {
                let converted = self
                    .callbacks
                    .conversion
                    .run(scope, |cx| cx.value(&BOOLEAN, input))
                    .map_err(Error::from)?;
                if *converted.get() {
                    Some(copy_native_data(&owner, &events, &[])?)
                } else {
                    None
                }
            } else if matches!(reflection, Reflection::Popover) {
                let converted = self
                    .callbacks
                    .conversion
                    .run(scope, |cx| cx.value(&DOM_STRING.nullable(), input))
                    .map_err(Error::from)?;
                scope.check_active()?;
                converted
                    .get()
                    .as_ref()
                    .map(|value| copy_native_data(&owner, &events, value.as_code_units()))
                    .transpose()?
            } else if matches!(reflection, Reflection::HeadingOffset) {
                let converted = self
                    .callbacks
                    .conversion
                    .run(scope, |cx| {
                        cx.value(
                            &IntegerType::new(IntegerKind::UnsignedLong, IntegerMode::Wrap),
                            input,
                        )
                    })
                    .map_err(Error::from)?;
                scope.check_active()?;
                let IntegerValue::UnsignedLong(value) = *converted.get() else {
                    return Err(Error::BackendUnavailable.into());
                };
                events.step()?;
                let value = heading_offset_setter_value(value);
                // A u32 decimal has at most ten digits. Fund the requested
                // UTF-8 capacity before formatting, then copy via stack UTF-16.
                let _format_backing = events.reserve(&owner, 10)?;
                let mut text = String::new();
                text.try_reserve_exact(10).map_err(|_| Error::Allocation)?;
                events.step()?;
                write!(&mut text, "{value}")
                    .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
                let mut units = [0_u16; 10];
                for (byte, unit) in text.as_bytes().iter().zip(&mut units) {
                    events.step()?;
                    *unit = u16::from(*byte);
                }
                let units = units
                    .get(..text.len())
                    .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
                Some(copy_native_data(&owner, &events, units)?)
            } else {
                let converted = self
                    .callbacks
                    .conversion
                    .run(scope, |cx| cx.value(&DOM_STRING, input))
                    .map_err(Error::from)?;
                scope.check_active()?;
                Some(copy_native_data(
                    &owner,
                    &events,
                    converted.get().as_code_units(),
                )?)
            };
            let (name, _name_backing) = copy_native_data(&owner, &events, attribute)?;
            match value {
                Some((value, _backing)) => self.apply_attribute(
                    scope,
                    &owner,
                    &events,
                    DomSourceEdit::set_attribute_ns(id, Namespace::none(), name, value),
                    None,
                    element::AttributeResult::Value,
                ),
                None => self.apply_tree_edit(
                    scope,
                    &owner,
                    &events,
                    DomSourceEdit::remove_attribute_ns(id, Namespace::none(), name),
                    Completion::Undefined,
                ),
            }
        })();
        tree::tree_completion(scope, &events, result)
    }
}

enum Reflected {
    Write,
    Boolean(bool),
    Unsigned(u8),
    String(Option<DocumentString>),
    Keyword(Option<&'static str>),
}
