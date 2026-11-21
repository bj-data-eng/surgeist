//! The three Dialog accessors over the canonical element and slot.

use super::{
    reflection::{self, Receiver, Reflection},
    text::TextBuffer,
    *,
};
use surgeist_bindings_js::conversion::types::DOM_STRING;
use surgeist_dom::source::DomSourceEdit;

const OPEN: reflection::Member = (
    &ascii(b"open"),
    &ascii(b"get open"),
    &ascii(b"set open"),
    &ascii(b"open"),
    Reflection::Boolean,
);
const CLOSED_BY: reflection::Member = (
    &ascii(b"closedBy"),
    &ascii(b"get closedBy"),
    &ascii(b"set closedBy"),
    &ascii(b"closedby"),
    Reflection::DialogClosedBy,
);

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Dialog, &[OPEN])?;
    let get = capture(size_of::<Capture>())?;
    let set = capture(size_of::<Capture>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| get.dialog_return_value(scope, arguments, false)),
        FunctionMetadata {
            name: &ascii(b"get returnValue"),
            length: 0,
        },
    )?;
    let setter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| set.dialog_return_value(scope, arguments, true)),
        FunctionMetadata {
            name: &ascii(b"set returnValue"),
            length: 1,
        },
    )?;
    scope.define_new_property(
        prototype,
        PropertyKey::String(&ascii(b"returnValue")),
        PropertyDescriptor::Accessor {
            get: Some(&getter),
            set: Some(&setter),
            enumerable: true,
            configurable: true,
        },
    )?;
    reflection::install(scope, prototype, capture, Receiver::Dialog, &[CLOSED_BY])
}

impl Capture {
    fn dialog_return_value<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        write: bool,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Dialog;
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
                let mut value = TextBuffer::default();
                if !write {
                    read.step()?;
                    let dialog = node.as_dialog().ok_or(DomError::WrongNodeKind)?;
                    value.append(read, dialog.return_value())?;
                }
                Ok(Some(value))
            })?;
            let Some(value) = value else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            if !write {
                return Ok(Completion::Return(
                    self.copy_dom_units(scope, value.code_units())?,
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
            let matches = self.callbacks.with_read(&owner, &events, |read, _| {
                receiver.matches(read.node(id)?, || read.step())
            })?;
            if !matches {
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
                DomSourceEdit::set_dialog_return_value(id, value),
                Completion::Undefined,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}
