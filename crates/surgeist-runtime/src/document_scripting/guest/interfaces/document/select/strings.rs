//! Select and Option string members over admitted native reads and Source edits.

use super::super::{element::AttributeResult, reflection::Receiver, text::TextBuffer, *};
use surgeist_bindings_js::conversion::types::DOM_STRING;
use surgeist_dom::{source::DomSourceEdit, tree::DomNode};
use surgeist_html::values::Namespace;

#[derive(Clone, Copy)]
pub(super) enum StringMember {
    SelectValue,
    OptionLabel,
    OptionValue,
    OptionText,
}

impl StringMember {
    fn receiver(self) -> Receiver {
        match self {
            Self::SelectValue => Receiver::Select,
            Self::OptionLabel | Self::OptionValue | Self::OptionText => Receiver::Option,
        }
    }

    fn names(self) -> (&'static [u16], &'static [u16], &'static [u16]) {
        match self {
            Self::SelectValue | Self::OptionValue => {
                const { (&ascii(b"value"), &ascii(b"get value"), &ascii(b"set value")) }
            }
            Self::OptionLabel => {
                const { (&ascii(b"label"), &ascii(b"get label"), &ascii(b"set label")) }
            }
            Self::OptionText => {
                const { (&ascii(b"text"), &ascii(b"get text"), &ascii(b"set text")) }
            }
        }
    }

    fn read(
        self,
        read: &NativeRead<'_>,
        node: DomNode<'_>,
    ) -> Result<TextBuffer, DocumentGuestEventError> {
        let mut text = TextBuffer::default();
        let step = || read.step();
        let reserve = |bytes| read.reserve(bytes);
        let emit = |units: &[u16]| text.append_units(read, units);
        match self {
            Self::SelectValue => node
                .as_select()
                .ok_or(DomError::WrongNodeKind)?
                .visit_value_with(step, reserve, emit)?,
            Self::OptionLabel => node
                .as_option()
                .ok_or(DomError::WrongNodeKind)?
                .visit_label_with(step, reserve, emit)?,
            Self::OptionValue => node
                .as_option()
                .ok_or(DomError::WrongNodeKind)?
                .visit_value_with(step, reserve, emit)?,
            Self::OptionText => node
                .as_option()
                .ok_or(DomError::WrongNodeKind)?
                .visit_text_with(step, reserve, emit)?,
        }
        Ok(text)
    }
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    member: StringMember,
) -> Result<(), DocumentGuestEventError> {
    let (name, get_name, set_name) = member.names();
    let get = capture(size_of::<(Capture, StringMember)>())?;
    let set = capture(size_of::<(Capture, StringMember)>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, args| get.select_string_get(scope, args, member)),
        FunctionMetadata {
            name: get_name,
            length: 0,
        },
    )?;
    let setter = scope.fixed_arity_function(
        &Callback::new(move |scope, args| set.select_string_set(scope, args, member)),
        FunctionMetadata {
            name: set_name,
            length: 1,
        },
    )?;
    scope.define_new_property(
        prototype,
        PropertyKey::String(name),
        PropertyDescriptor::Accessor {
            get: Some(&getter),
            set: Some(&setter),
            enumerable: true,
            configurable: true,
        },
    )?;
    Ok(())
}

impl Capture {
    fn select_string_get<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: StringMember,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = member.receiver();
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
                member.read(read, node).map(Some)
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

    fn select_string_set<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: StringMember,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = member.receiver();
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let matches = || {
                self.callbacks.with_read(&owner, &events, |read, _| {
                    receiver.matches(read.node(id)?, || read.step())
                })
            };
            if !matches()? {
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
            if !matches()? {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            let (value, _value_backing) =
                copy_native_data(&owner, &events, converted.get().as_code_units())?;
            let edit = match member {
                StringMember::SelectValue => DomSourceEdit::set_select_value(id, value),
                StringMember::OptionText => DomSourceEdit::set_text_content(id, value),
                StringMember::OptionLabel | StringMember::OptionValue => {
                    let (name, _name_backing) =
                        copy_native_data(&owner, &events, member.names().0)?;
                    return self.apply_attribute(
                        scope,
                        &owner,
                        &events,
                        DomSourceEdit::set_attribute_ns(id, Namespace::none(), name, value),
                        None,
                        AttributeResult::Value,
                    );
                }
            };
            self.apply_tree_edit(scope, &owner, &events, edit, Completion::Undefined)
        })();
        tree::tree_completion(scope, &events, result)
    }
}
