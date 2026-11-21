//! Live Select and Option members backed by the canonical DOM and Source operations.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_bindings_js::{IntegerValue, conversion::declarations};
use surgeist_dom::source::DomSourceEdit;

mod strings;
use strings::StringMember;

const SELECT_REFLECTIONS: &[reflection::Member] = &[(
    &ascii(b"multiple"),
    &ascii(b"get multiple"),
    &ascii(b"set multiple"),
    &ascii(b"multiple"),
    Reflection::Boolean,
)];

const OPTION_REFLECTIONS: &[reflection::Member] = &[(
    &ascii(b"defaultSelected"),
    &ascii(b"get defaultSelected"),
    &ascii(b"set defaultSelected"),
    &ascii(b"selected"),
    Reflection::Boolean,
)];

pub(super) fn install_select<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(
        scope,
        prototype,
        capture,
        Receiver::Select,
        SELECT_REFLECTIONS,
    )?;
    install_selected_property(
        scope,
        prototype,
        capture,
        SelectedProperty::SelectIndex,
        &ascii(b"selectedIndex"),
        &ascii(b"get selectedIndex"),
        &ascii(b"set selectedIndex"),
    )?;
    strings::install(scope, prototype, capture, StringMember::SelectValue)
}

pub(super) fn install_option<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    strings::install(scope, prototype, capture, StringMember::OptionLabel)?;
    reflection::install(
        scope,
        prototype,
        capture,
        Receiver::Option,
        OPTION_REFLECTIONS,
    )?;
    install_selected_property(
        scope,
        prototype,
        capture,
        SelectedProperty::OptionSelected,
        &ascii(b"selected"),
        &ascii(b"get selected"),
        &ascii(b"set selected"),
    )?;
    strings::install(scope, prototype, capture, StringMember::OptionValue)?;
    strings::install(scope, prototype, capture, StringMember::OptionText)?;
    let get = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"index"),
        &ascii(b"get index"),
        true,
        Callback::new(move |scope, arguments| {
            get.read_option_member(scope, arguments, OptionRead::Index)
        }),
    )?;
    Ok(())
}

#[derive(Clone, Copy)]
enum SelectedProperty {
    SelectIndex,
    OptionSelected,
}

impl SelectedProperty {
    fn receiver(self) -> Receiver {
        match self {
            Self::SelectIndex => Receiver::Select,
            Self::OptionSelected => Receiver::Option,
        }
    }
}

#[derive(Clone, Copy)]
enum OptionRead {
    Index,
    Selected,
}

fn install_selected_property<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    property: SelectedProperty,
    name: &[u16],
    get_name: &[u16],
    set_name: &[u16],
) -> Result<(), DocumentGuestEventError> {
    let get = capture(size_of::<(Capture, SelectedProperty)>())?;
    let set = capture(size_of::<(Capture, SelectedProperty)>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| match property {
            SelectedProperty::SelectIndex => get.read_selected_index(scope, arguments),
            SelectedProperty::OptionSelected => {
                get.read_option_member(scope, arguments, OptionRead::Selected)
            }
        }),
        FunctionMetadata {
            name: get_name,
            length: 0,
        },
    )?;
    let setter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            set.write_selected_property(scope, arguments, property)
        }),
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
    fn read_selected_index<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        self.read_selected_member(scope, arguments, Receiver::Select, |read, node| {
            let select = node.as_select().ok_or(DomError::WrongNodeKind)?;
            let index = select.selected_index_with(|| read.step())?;
            let value = index.map_or(Ok(-1), to_long)?;
            Ok(Primitive::Number(f64::from(value)))
        })
    }

    fn read_option_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: OptionRead,
    ) -> Result<Completion<'scope>, Error> {
        self.read_selected_member(scope, arguments, Receiver::Option, |read, node| {
            Ok(match member {
                OptionRead::Index => {
                    let option = node.as_option().ok_or(DomError::WrongNodeKind)?;
                    let index = option.index_with(|| read.step())?;
                    Primitive::Number(f64::from(to_long(index)?))
                }
                OptionRead::Selected => {
                    read.step()?;
                    let state = node
                        .as_element()
                        .and_then(|element| element.option_state())
                        .ok_or(DomError::WrongNodeKind)?;
                    Primitive::Boolean(state.flags().selectedness())
                }
            })
        })
    }

    fn read_selected_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
        read_member: impl FnOnce(
            &NativeRead<'_>,
            surgeist_dom::tree::DomNode<'_>,
        ) -> Result<Primitive, DocumentGuestEventError>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                read_member(read, node).map(Some)
            })?;
            match value {
                Some(value) => Ok(Completion::Return(scope.primitive(value)?)),
                None => Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                )),
            }
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn write_selected_property<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        property: SelectedProperty,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = property.receiver();
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
            let edit = match property {
                SelectedProperty::SelectIndex => {
                    let descriptor = declarations::html_select_element_selected_index_setter();
                    let converted = self
                        .callbacks
                        .conversion
                        .run(scope, |conversion| conversion.value(&descriptor, input))
                        .map_err(Error::from)?;
                    scope.check_active()?;
                    let IntegerValue::Long(index) = *converted.get() else {
                        return Err(Error::BackendUnavailable.into());
                    };
                    if !matches()? {
                        return Ok(Completion::Throw(
                            scope.type_error(receiver.error_message())?,
                        ));
                    }
                    DomSourceEdit::set_select_selected_index(id, i64::from(index))
                }
                SelectedProperty::OptionSelected => {
                    let descriptor = declarations::html_option_element_selected_setter();
                    let converted = self
                        .callbacks
                        .conversion
                        .run(scope, |conversion| conversion.value(&descriptor, input))
                        .map_err(Error::from)?;
                    scope.check_active()?;
                    DomSourceEdit::set_option_selected(id, *converted.get())
                }
            };
            self.apply_tree_edit(scope, &owner, &events, edit, Completion::Undefined)
        })();
        tree::tree_completion(scope, &events, result)
    }
}

fn to_long(index: usize) -> Result<i32, DocumentGuestEventError> {
    i32::try_from(index).map_err(|_| DocumentScriptNativeError::RepresentationExhausted.into())
}
