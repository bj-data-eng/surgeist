//! Dataset, nonce, autofocus and tabIndex on each including interface.

use super::{
    reflection::{self, Receiver, Reflection},
    text::TextBuffer,
    *,
};
use surgeist_bindings_js::conversion::types::DOM_STRING;
use surgeist_dom::source::DomSourceEdit;

const REFLECTIONS: &[reflection::Member] = &[
    (
        &ascii(b"autofocus"),
        &ascii(b"get autofocus"),
        &ascii(b"set autofocus"),
        &ascii(b"autofocus"),
        Reflection::Boolean,
    ),
    (
        &ascii(b"tabIndex"),
        &ascii(b"get tabIndex"),
        &ascii(b"set tabIndex"),
        &ascii(b"tabindex"),
        Reflection::TabIndex,
    ),
];
pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
    receiver: Receiver,
) -> Result<(), DocumentGuestEventError> {
    super::dataset::install_accessor(scope, prototype, capture, receiver)?;
    let get = capture(size_of::<(Capture, Receiver)>())?;
    let set = capture(size_of::<(Capture, Receiver)>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            get.element_nonce(scope, arguments, receiver, false)
        }),
        FunctionMetadata {
            name: &ascii(b"get nonce"),
            length: 0,
        },
    )?;
    let setter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| set.element_nonce(scope, arguments, receiver, true)),
        FunctionMetadata {
            name: &ascii(b"set nonce"),
            length: 1,
        },
    )?;
    scope.define_new_property(
        prototype,
        PropertyKey::String(&ascii(b"nonce")),
        PropertyDescriptor::Accessor {
            get: Some(&getter),
            set: Some(&setter),
            enumerable: true,
            configurable: true,
        },
    )?;
    reflection::install(scope, prototype, capture, receiver, REFLECTIONS)
}

impl Capture {
    fn element_nonce<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
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
            let nonce = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
                if !receiver.matches(node, || read.step())? {
                    return Ok(None);
                }
                let mut nonce = TextBuffer::default();
                if !write {
                    read.step()?;
                    let current = node
                        .as_element()
                        .and_then(|element| element.nonce())
                        .ok_or(DomError::NonceInapplicable)?;
                    nonce.append(read, current)?;
                }
                Ok(Some(nonce))
            })?;
            let Some(nonce) = nonce else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            if !write {
                return Ok(Completion::Return(
                    self.copy_dom_units(scope, nonce.code_units())?,
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
            let (nonce, _backing) =
                copy_native_data(&owner, &events, converted.get().as_code_units())?;
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::set_nonce(id, nonce),
                Completion::Undefined,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}
