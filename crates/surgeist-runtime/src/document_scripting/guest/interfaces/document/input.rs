//! Input reflections and canonical live state on the attached element.

use super::{
    custom_validity::CustomValidityControl,
    form_owner::{self, FormOwnerGetter},
    reflection::{self, Receiver, Reflection},
    text::TextBuffer,
    *,
};
use surgeist_bindings_js::conversion::{
    declarations::html_input_element_value_setter, types::BOOLEAN,
};
use surgeist_dom::source::DomSourceEdit;
use surgeist_html::bounded::attributes::HtmlControlDimensionRule;

const BEFORE_CHECKED: &[reflection::Member] = &[
    (
        &ascii(b"accept"),
        &ascii(b"get accept"),
        &ascii(b"set accept"),
        &ascii(b"accept"),
        Reflection::String,
    ),
    (
        &ascii(b"alpha"),
        &ascii(b"get alpha"),
        &ascii(b"set alpha"),
        &ascii(b"alpha"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"alt"),
        &ascii(b"get alt"),
        &ascii(b"set alt"),
        &ascii(b"alt"),
        Reflection::String,
    ),
    reflection::CONTROL_AUTOCOMPLETE,
    (
        &ascii(b"defaultChecked"),
        &ascii(b"get defaultChecked"),
        &ascii(b"set defaultChecked"),
        &ascii(b"checked"),
        Reflection::Boolean,
    ),
];
const BEFORE_FORM: &[reflection::Member] = &[
    (
        &ascii(b"colorSpace"),
        &ascii(b"get colorSpace"),
        &ascii(b"set colorSpace"),
        &ascii(b"colorspace"),
        Reflection::InputColorSpace,
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
const BEFORE_INDETERMINATE: &[reflection::Member] = &[
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
];
const AFTER_INDETERMINATE: &[reflection::Member] = &[
    (
        &ascii(b"max"),
        &ascii(b"get max"),
        &ascii(b"set max"),
        &ascii(b"max"),
        Reflection::String,
    ),
    (
        &ascii(b"maxLength"),
        &ascii(b"get maxLength"),
        &ascii(b"set maxLength"),
        &ascii(b"maxlength"),
        Reflection::ControlLength,
    ),
    (
        &ascii(b"min"),
        &ascii(b"get min"),
        &ascii(b"set min"),
        &ascii(b"min"),
        Reflection::String,
    ),
    (
        &ascii(b"minLength"),
        &ascii(b"get minLength"),
        &ascii(b"set minLength"),
        &ascii(b"minlength"),
        Reflection::ControlLength,
    ),
    (
        &ascii(b"multiple"),
        &ascii(b"get multiple"),
        &ascii(b"set multiple"),
        &ascii(b"multiple"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"name"),
        &ascii(b"get name"),
        &ascii(b"set name"),
        &ascii(b"name"),
        Reflection::String,
    ),
    (
        &ascii(b"pattern"),
        &ascii(b"get pattern"),
        &ascii(b"set pattern"),
        &ascii(b"pattern"),
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
        &ascii(b"size"),
        &ascii(b"get size"),
        &ascii(b"set size"),
        &ascii(b"size"),
        Reflection::ControlDimension(HtmlControlDimensionRule::InputSize),
    ),
    (
        &ascii(b"step"),
        &ascii(b"get step"),
        &ascii(b"set step"),
        &ascii(b"step"),
        Reflection::String,
    ),
    (
        &ascii(b"type"),
        &ascii(b"get type"),
        &ascii(b"set type"),
        &ascii(b"type"),
        Reflection::InputType,
    ),
    (
        &ascii(b"defaultValue"),
        &ascii(b"get defaultValue"),
        &ascii(b"set defaultValue"),
        &ascii(b"value"),
        Reflection::String,
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
        &ascii(b"useMap"),
        &ascii(b"get useMap"),
        &ascii(b"set useMap"),
        &ascii(b"usemap"),
        Reflection::String,
    ),
];

#[derive(Clone, Copy)]
enum InputProperty {
    Flag(InputFlag),
    Value,
}

#[derive(Clone, Copy)]
enum InputFlag {
    Checked,
    Indeterminate,
}

enum InputObservation {
    Flag(bool),
    Value(TextBuffer),
}

impl InputProperty {
    fn names(self) -> (&'static [u16], &'static [u16], &'static [u16]) {
        match self {
            Self::Flag(InputFlag::Checked) => {
                const {
                    (
                        &ascii(b"checked"),
                        &ascii(b"get checked"),
                        &ascii(b"set checked"),
                    )
                }
            }
            Self::Flag(InputFlag::Indeterminate) => {
                const {
                    (
                        &ascii(b"indeterminate"),
                        &ascii(b"get indeterminate"),
                        &ascii(b"set indeterminate"),
                    )
                }
            }
            Self::Value => const { (&ascii(b"value"), &ascii(b"get value"), &ascii(b"set value")) },
        }
    }

    fn install<'scope>(
        self,
        scope: &mut Scope<'scope>,
        prototype: &Value<'scope>,
        capture: CaptureFunding<'_>,
    ) -> Result<(), DocumentGuestEventError> {
        let (name, getter, setter) = self.names();
        let get = capture(size_of::<(Capture, InputProperty)>())?;
        let set = capture(size_of::<(Capture, InputProperty)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| get.read_input_property(scope, arguments, self)),
            FunctionMetadata {
                name: getter,
                length: 0,
            },
        )?;
        let setter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                set.write_input_property(scope, arguments, self)
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
        Ok(())
    }
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Input, BEFORE_CHECKED)?;
    InputProperty::Flag(InputFlag::Checked).install(scope, prototype, capture)?;
    reflection::install(scope, prototype, capture, Receiver::Input, BEFORE_FORM)?;
    form_owner::install(scope, prototype, capture, FormOwnerGetter::Input)?;
    reflection::install(
        scope,
        prototype,
        capture,
        Receiver::Input,
        BEFORE_INDETERMINATE,
    )?;
    InputProperty::Flag(InputFlag::Indeterminate).install(scope, prototype, capture)?;
    reflection::install(
        scope,
        prototype,
        capture,
        Receiver::Input,
        AFTER_INDETERMINATE,
    )?;
    InputProperty::Value.install(scope, prototype, capture)?;
    super::control_validation::install(scope, prototype, capture, Receiver::Input)?;
    CustomValidityControl::Input.install(scope, prototype, capture)?;
    super::reflected_element::ElementTarget::InputPopover.install(scope, prototype, capture)?;
    reflection::install(scope, prototype, capture, Receiver::Input, OBSOLETE)
}

impl Capture {
    fn read_input_property<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        property: InputProperty,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Input;
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
                Ok(Some(match property {
                    InputProperty::Flag(flag) => {
                        read.step()?;
                        let flags = node
                            .as_element()
                            .and_then(|element| element.input_state())
                            .ok_or(DomError::WrongNodeKind)?
                            .flags();
                        InputObservation::Flag(match flag {
                            InputFlag::Checked => flags.checkedness(),
                            InputFlag::Indeterminate => flags.indeterminate(),
                        })
                    }
                    InputProperty::Value => {
                        let input = node.as_input().ok_or(DomError::WrongNodeKind)?;
                        let units = input.value_with(|| read.step())?;
                        let mut text = TextBuffer::default();
                        text.append_units(read, units)?;
                        InputObservation::Value(text)
                    }
                }))
            })?;
            let Some(value) = value else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            let value = match value {
                InputObservation::Flag(flag) => scope.primitive(Primitive::Boolean(flag))?,
                InputObservation::Value(value) => self.copy_dom_units(scope, value.code_units())?,
            };
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn write_input_property<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        property: InputProperty,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Input;
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
            let (edit, _backing) = match property {
                InputProperty::Flag(flag) => {
                    let converted = self
                        .callbacks
                        .conversion
                        .run(scope, |conversion| conversion.value(&BOOLEAN, input))
                        .map_err(Error::from)?;
                    let value = *converted.get();
                    let edit = match flag {
                        InputFlag::Checked => DomSourceEdit::set_input_checked(id, value),
                        InputFlag::Indeterminate => {
                            DomSourceEdit::set_input_indeterminate(id, value)
                        }
                    };
                    (edit, None)
                }
                InputProperty::Value => {
                    let descriptor = html_input_element_value_setter();
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
                    let (value, backing) =
                        copy_native_data(&owner, &events, converted.get().as_code_units())?;
                    (DomSourceEdit::set_input_value(id, value), Some(backing))
                }
            };
            self.apply_tree_edit(scope, &owner, &events, edit, Completion::Undefined)
        })();
        tree::tree_completion(scope, &events, result)
    }
}
