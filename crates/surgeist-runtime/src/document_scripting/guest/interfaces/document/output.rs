//! Output bindings over the current canonical element and its ordinary Text.

use super::{
    barred_control::BarredControl,
    custom_validity::CustomValidityControl,
    form_owner::{self, FormOwnerGetter},
    reflection::{self, Receiver, Reflection},
    text::{TextBuffer, collect_text},
    *,
};
use surgeist_bindings_js::conversion::types::DOM_STRING;
use surgeist_dom::source::DomSourceEdit;

const NAME: reflection::Member = (
    &ascii(b"name"),
    &ascii(b"get name"),
    &ascii(b"set name"),
    &ascii(b"name"),
    Reflection::String,
);

#[derive(Clone, Copy)]
enum OutputProperty {
    DefaultValue,
    Value,
}

struct ValueAccessor {
    name: &'static [u16],
    getter: &'static [u16],
    setter: &'static [u16],
    property: OutputProperty,
}

const VALUES: &[ValueAccessor] = &[
    ValueAccessor {
        name: &ascii(b"defaultValue"),
        getter: &ascii(b"get defaultValue"),
        setter: &ascii(b"set defaultValue"),
        property: OutputProperty::DefaultValue,
    },
    ValueAccessor {
        name: &ascii(b"value"),
        getter: &ascii(b"get value"),
        setter: &ascii(b"set value"),
        property: OutputProperty::Value,
    },
];

enum OutputValue {
    Override(DocumentString),
    Descendants(TextBuffer),
}

impl OutputValue {
    fn code_units(&self) -> &[u16] {
        match self {
            Self::Override(override_value) => override_value.code_units(),
            Self::Descendants(text) => text.code_units(),
        }
    }
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    form_owner::install(scope, prototype, capture, FormOwnerGetter::Output)?;
    reflection::install(scope, prototype, capture, Receiver::Output, &[NAME])?;
    BarredControl::Output.install_type(scope, prototype, capture)?;
    for &ValueAccessor {
        name,
        getter,
        setter,
        property,
    } in VALUES
    {
        let get = capture(size_of::<(Capture, OutputProperty)>())?;
        let set = capture(size_of::<(Capture, OutputProperty)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                get.read_output_value(scope, arguments, property)
            }),
            FunctionMetadata {
                name: getter,
                length: 0,
            },
        )?;
        let setter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                set.write_output_value(scope, arguments, property)
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
    BarredControl::Output.install_validation(scope, prototype, capture)?;
    CustomValidityControl::Output.install(scope, prototype, capture)
}

impl Capture {
    fn read_output_value<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        property: OutputProperty,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Output;
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
                if matches!(property, OutputProperty::DefaultValue) {
                    read.step()?;
                    let state = node
                        .as_element()
                        .and_then(|element| element.output_state())
                        .ok_or(DomError::WrongNodeKind)?;
                    if let Some(override_value) = state.default_value_override() {
                        return Ok(Some(OutputValue::Override(override_value.clone())));
                    }
                }
                collect_text(read, node).map(|value| Some(OutputValue::Descendants(value)))
            })?;
            let Some(value) = value else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            Ok(Completion::Return(
                self.copy_dom_units(scope, value.code_units())?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn write_output_value<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        property: OutputProperty,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Output;
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
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| conversion.value(&DOM_STRING, input))
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
                OutputProperty::DefaultValue => DomSourceEdit::set_output_default_value(id, value),
                OutputProperty::Value => DomSourceEdit::set_output_value(id, value),
            };
            self.apply_tree_edit(scope, &owner, &events, edit, Completion::Undefined)
        })();
        tree::tree_completion(scope, &events, result)
    }
}
