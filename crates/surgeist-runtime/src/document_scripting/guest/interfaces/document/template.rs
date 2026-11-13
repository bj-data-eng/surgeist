//! HTMLTemplateElement exposes the canonical inert fragment and reflected attributes.

use super::*;
use surgeist_bindings_js::conversion::types::{BOOLEAN, DOM_STRING};
use surgeist_dom::{DomSourceEdit, Namespace};
use surgeist_html::{
    bounded::attributes::{
        template_shadow_root_mode_keyword_with, template_shadow_root_slot_assignment_keyword_with,
    },
    vocabulary::KnownHtmlAttribute as Attribute,
};

#[derive(Clone, Copy)]
enum Reflection {
    String,
    Boolean,
    Mode,
    SlotAssignment,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"content"),
        &ascii(b"get content"),
        true,
        Callback::new(move |scope, arguments| handler.template_content(scope, arguments)),
    )?;
    for (member, get_name, set_name, attribute, reflection) in [
        (
            &ascii(b"htmlFor")[..],
            &ascii(b"get htmlFor")[..],
            &ascii(b"set htmlFor")[..],
            Attribute::For,
            Reflection::String,
        ),
        (
            &ascii(b"shadowRootMode")[..],
            &ascii(b"get shadowRootMode")[..],
            &ascii(b"set shadowRootMode")[..],
            Attribute::Shadowrootmode,
            Reflection::Mode,
        ),
        (
            &ascii(b"shadowRootSlotAssignment")[..],
            &ascii(b"get shadowRootSlotAssignment")[..],
            &ascii(b"set shadowRootSlotAssignment")[..],
            Attribute::Shadowrootslotassignment,
            Reflection::SlotAssignment,
        ),
        (
            &ascii(b"shadowRootCustomElementRegistry")[..],
            &ascii(b"get shadowRootCustomElementRegistry")[..],
            &ascii(b"set shadowRootCustomElementRegistry")[..],
            Attribute::Shadowrootcustomelementregistry,
            Reflection::String,
        ),
        (
            &ascii(b"shadowRootDelegatesFocus")[..],
            &ascii(b"get shadowRootDelegatesFocus")[..],
            &ascii(b"set shadowRootDelegatesFocus")[..],
            Attribute::Shadowrootdelegatesfocus,
            Reflection::Boolean,
        ),
        (
            &ascii(b"shadowRootSerializable")[..],
            &ascii(b"get shadowRootSerializable")[..],
            &ascii(b"set shadowRootSerializable")[..],
            Attribute::Shadowrootserializable,
            Reflection::Boolean,
        ),
        (
            &ascii(b"shadowRootClonable")[..],
            &ascii(b"get shadowRootClonable")[..],
            &ascii(b"set shadowRootClonable")[..],
            Attribute::Shadowrootclonable,
            Reflection::Boolean,
        ),
    ] {
        let get = capture(size_of::<(Capture, Attribute, Reflection)>())?;
        let set = capture(size_of::<(Capture, Attribute, Reflection)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                get.template_reflection(scope, arguments, (attribute, reflection), false)
            }),
            FunctionMetadata {
                name: get_name,
                length: 0,
            },
        )?;
        let setter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                set.template_reflection(scope, arguments, (attribute, reflection), true)
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
    fn template_content<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(receiver)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"HTMLTemplateElement required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let contents = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(receiver)?;
                node.as_template()
                    .map(|template| template.content_fragment_with(|| read.step()))
                    .transpose()
            })?;
            let Some(contents) = contents else {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"HTMLTemplateElement required"))?,
                ));
            };
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            Ok(Completion::Return(
                public.project_node(scope, &owner, &events, contents)?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn template_reflection<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        (attribute, reflection): (Attribute, Reflection),
        write: bool,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"HTMLTemplateElement required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if node.as_template().is_none() {
                    return Ok(None);
                }
                if write {
                    return Ok(Some(Reflected::Write));
                }
                let attrs = node.attributes().ok_or(DomError::WrongNodeKind)?;
                let stored = attrs.get_html_attribute_with(attribute, || read.step())?;
                let units = stored.map(|value| value.value().code_units());
                Ok(Some(match reflection {
                    Reflection::Boolean => Reflected::Boolean(stored.is_some()),
                    Reflection::String => {
                        Reflected::String(stored.map(|value| value.value().clone()))
                    }
                    Reflection::Mode => {
                        Reflected::Keyword(template_shadow_root_mode_keyword_with(units, |_| {
                            read.step()
                        })?)
                    }
                    Reflection::SlotAssignment => Reflected::Keyword(
                        template_shadow_root_slot_assignment_keyword_with(units, |_| read.step())?,
                    ),
                }))
            })?;
            let Some(value) = value else {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"HTMLTemplateElement required"))?,
                ));
            };
            let output = match value {
                Reflected::Write => None,
                Reflected::Boolean(value) => Some(scope.primitive(Primitive::Boolean(value))?),
                Reflected::String(value) => Some(self.copy_dom_units(
                    scope,
                    value.as_ref().map_or(&[], DocumentString::code_units),
                )?),
                Reflected::Keyword(keyword) => {
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
            let name = attribute.local_name();
            let mut units = [0_u16; 31];
            let units = units
                .get_mut(..name.len())
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
            for (unit, byte) in units.iter_mut().zip(name.bytes()) {
                *unit = u16::from(byte);
            }
            let (name, _name_backing) = copy_native_data(&owner, &events, units)?;
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
    String(Option<DocumentString>),
    Keyword(&'static str),
}
