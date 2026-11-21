//! Textarea bindings over current native control state and immediate child Text.

use super::{
    form_owner::{self, FormOwnerGetter},
    reflection::{self, Receiver, Reflection},
    text::TextBuffer,
    *,
};
use surgeist_bindings_js::conversion::{
    declarations::html_text_area_element_value_setter, types::DOM_STRING,
};
use surgeist_dom::{source::DomSourceEdit, tree::DomNode};
use surgeist_html::bounded::attributes::HtmlControlDimensionRule;

const BEFORE_FORM: &[reflection::Member] = &[
    (
        &ascii(b"cols"),
        &ascii(b"get cols"),
        &ascii(b"set cols"),
        &ascii(b"cols"),
        Reflection::ControlDimension(HtmlControlDimensionRule::TextareaCols),
    ),
    (
        &ascii(b"dirName"),
        &ascii(b"get dirName"),
        &ascii(b"set dirName"),
        &ascii(b"dirname"),
        Reflection::String,
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
        &ascii(b"maxLength"),
        &ascii(b"get maxLength"),
        &ascii(b"set maxLength"),
        &ascii(b"maxlength"),
        Reflection::ControlLength,
    ),
    (
        &ascii(b"minLength"),
        &ascii(b"get minLength"),
        &ascii(b"set minLength"),
        &ascii(b"minlength"),
        Reflection::ControlLength,
    ),
    (
        &ascii(b"name"),
        &ascii(b"get name"),
        &ascii(b"set name"),
        &ascii(b"name"),
        Reflection::String,
    ),
    (
        &ascii(b"placeholder"),
        &ascii(b"get placeholder"),
        &ascii(b"set placeholder"),
        &ascii(b"placeholder"),
        Reflection::String,
    ),
    (
        &ascii(b"readOnly"),
        &ascii(b"get readOnly"),
        &ascii(b"set readOnly"),
        &ascii(b"readonly"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"required"),
        &ascii(b"get required"),
        &ascii(b"set required"),
        &ascii(b"required"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"rows"),
        &ascii(b"get rows"),
        &ascii(b"set rows"),
        &ascii(b"rows"),
        Reflection::ControlDimension(HtmlControlDimensionRule::TextareaRows),
    ),
    (
        &ascii(b"wrap"),
        &ascii(b"get wrap"),
        &ascii(b"set wrap"),
        &ascii(b"wrap"),
        Reflection::String,
    ),
];

#[derive(Clone, Copy)]
enum TextareaProperty {
    DefaultValue,
    Value,
}

struct ValueAccessor {
    name: &'static [u16],
    getter: &'static [u16],
    setter: &'static [u16],
    property: TextareaProperty,
}

const VALUES: &[ValueAccessor] = &[
    ValueAccessor {
        name: &ascii(b"defaultValue"),
        getter: &ascii(b"get defaultValue"),
        setter: &ascii(b"set defaultValue"),
        property: TextareaProperty::DefaultValue,
    },
    ValueAccessor {
        name: &ascii(b"value"),
        getter: &ascii(b"get value"),
        setter: &ascii(b"set value"),
        property: TextareaProperty::Value,
    },
];

#[derive(Clone, Copy)]
enum TextareaRead {
    Type,
    Text(TextareaProperty),
    TextLength,
}

enum TextareaResult {
    Type,
    Text(TextBuffer),
    Length(u32),
}

impl TextareaRead {
    fn observe(
        self,
        read: &NativeRead<'_>,
        node: DomNode<'_>,
    ) -> Result<TextareaResult, DocumentGuestEventError> {
        match self {
            Self::Type => Ok(TextareaResult::Type),
            Self::Text(property) => {
                let mut text = TextBuffer::default();
                match property {
                    TextareaProperty::DefaultValue => {
                        node.visit_child_text_with(
                            || read.step(),
                            |value| text.append(read, value),
                        )?;
                    }
                    TextareaProperty::Value => {
                        let textarea = node.as_textarea().ok_or(DomError::WrongNodeKind)?;
                        textarea.visit_value_with(
                            || read.step(),
                            |unit| text.append_units(read, std::slice::from_ref(&unit)),
                        )?;
                    }
                }
                Ok(TextareaResult::Text(text))
            }
            Self::TextLength => {
                let textarea = node.as_textarea().ok_or(DomError::WrongNodeKind)?;
                let length = textarea.text_length_with(|| read.step())?;
                let length = u32::try_from(length)
                    .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
                Ok(TextareaResult::Length(length))
            }
        }
    }
}

fn install_read<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    name: &[u16],
    getter: &[u16],
    observation: TextareaRead,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<(Capture, TextareaRead)>())?;
    install_getter(
        scope,
        prototype,
        name,
        getter,
        true,
        Callback::new(move |scope, arguments| handler.read_textarea(scope, arguments, observation)),
    )?;
    Ok(())
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Textarea, BEFORE_FORM)?;
    form_owner::install(scope, prototype, capture, FormOwnerGetter::Textarea)?;
    reflection::install(scope, prototype, capture, Receiver::Textarea, AFTER_FORM)?;
    install_read(
        scope,
        prototype,
        capture,
        &ascii(b"type"),
        &ascii(b"get type"),
        TextareaRead::Type,
    )?;
    for &ValueAccessor {
        name,
        getter,
        setter,
        property,
    } in VALUES
    {
        let get = capture(size_of::<(Capture, TextareaProperty)>())?;
        let set = capture(size_of::<(Capture, TextareaProperty)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                get.read_textarea(scope, arguments, TextareaRead::Text(property))
            }),
            FunctionMetadata {
                name: getter,
                length: 0,
            },
        )?;
        let setter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                set.write_textarea_value(scope, arguments, property)
            }),
            FunctionMetadata {
                name: setter,
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
    }
    install_read(
        scope,
        prototype,
        capture,
        &ascii(b"textLength"),
        &ascii(b"get textLength"),
        TextareaRead::TextLength,
    )
}

impl Capture {
    fn read_textarea<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        observation: TextareaRead,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Textarea;
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
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                observation.observe(read, node).map(Some)
            })?;
            let Some(value) = value else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            let value = match value {
                TextareaResult::Type => self.copy_dom_units(scope, &ascii(b"textarea"))?,
                TextareaResult::Text(text) => self.copy_dom_units(scope, text.code_units())?,
                TextareaResult::Length(length) => {
                    scope.primitive(Primitive::Number(f64::from(length)))?
                }
            };
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn write_textarea_value<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        property: TextareaProperty,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Textarea;
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let matches_receiver = || {
                self.callbacks.with_read(&owner, &events, |read, _| {
                    receiver.matches(read.node(id)?, || read.step())
                })
            };
            if !matches_receiver()? {
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
            let descriptor = match property {
                TextareaProperty::DefaultValue => DOM_STRING,
                TextareaProperty::Value => html_text_area_element_value_setter(),
            };
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| conversion.value(&descriptor, input))
                .map_err(Error::from)?;
            scope.check_active()?;
            if !matches_receiver()? {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            let (value, _backing) =
                copy_native_data(&owner, &events, converted.get().as_code_units())?;
            let edit = match property {
                TextareaProperty::DefaultValue => {
                    DomSourceEdit::set_textarea_default_value(id, value)
                }
                TextareaProperty::Value => DomSourceEdit::set_textarea_value(id, value),
            };
            self.apply_tree_edit(scope, &owner, &events, edit, Completion::Undefined)
        })();
        tree::tree_completion(scope, &events, result)
    }
}
