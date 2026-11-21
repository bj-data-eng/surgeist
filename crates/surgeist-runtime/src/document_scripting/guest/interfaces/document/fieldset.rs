//! Local Fieldset accessors and barred constraint validation.

use super::{
    form_owner::{self, FormOwnerGetter},
    reflection::{self, Receiver, Reflection},
    *,
};
use surgeist_bindings_js::conversion::{
    ArgumentValue, declarations::html_field_set_element_set_custom_validity_0_error,
};
use surgeist_dom::source::DomSourceEdit;

const DISABLED: reflection::Member = (
    &ascii(b"disabled"),
    &ascii(b"get disabled"),
    &ascii(b"set disabled"),
    &ascii(b"disabled"),
    Reflection::Boolean,
);
const NAME: reflection::Member = (
    &ascii(b"name"),
    &ascii(b"get name"),
    &ascii(b"set name"),
    &ascii(b"name"),
    Reflection::String,
);

#[derive(Clone, Copy)]
enum FieldsetObservation {
    Type,
    WillValidate,
    ValidationMessage,
    ValidationSuccess,
}

const GETTERS: &[(&[u16], &[u16], FieldsetObservation)] = &[
    (
        &ascii(b"type"),
        &ascii(b"get type"),
        FieldsetObservation::Type,
    ),
    (
        &ascii(b"willValidate"),
        &ascii(b"get willValidate"),
        FieldsetObservation::WillValidate,
    ),
    (
        &ascii(b"validationMessage"),
        &ascii(b"get validationMessage"),
        FieldsetObservation::ValidationMessage,
    ),
];

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(scope, prototype, capture, Receiver::Fieldset, &[DISABLED])?;
    form_owner::install(scope, prototype, capture, FormOwnerGetter::Fieldset)?;
    reflection::install(scope, prototype, capture, Receiver::Fieldset, &[NAME])?;
    for &(name, getter, observation) in GETTERS {
        let handler = capture(size_of::<(Capture, FieldsetObservation)>())?;
        install_getter(
            scope,
            prototype,
            name,
            getter,
            true,
            Callback::new(move |scope, arguments| {
                handler.fieldset_observation(scope, arguments, observation)
            }),
        )?;
    }
    for name in [&ascii(b"checkValidity")[..], &ascii(b"reportValidity")[..]] {
        let handler = capture(size_of::<Capture>())?;
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.fieldset_observation(
                    scope,
                    arguments,
                    FieldsetObservation::ValidationSuccess,
                )
            }),
            FunctionMetadata { name, length: 0 },
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(name),
            &method,
            true,
            true,
            true,
        )?;
    }
    let handler = capture(size_of::<Capture>())?;
    let name = &ascii(b"setCustomValidity");
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            handler.fieldset_set_custom_validity(scope, arguments)
        }),
        FunctionMetadata { name, length: 1 },
    )?;
    define_data(
        scope,
        prototype,
        PropertyKey::String(name),
        &method,
        true,
        true,
        true,
    )?;
    Ok(())
}

impl Capture {
    fn fieldset_observation<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        observation: FieldsetObservation,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Fieldset;
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let matches = self.callbacks.with_read(&owner, &events, |read, _| {
                receiver.matches(read.node(id)?, || read.step())
            })?;
            if !matches {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            // Fieldset is barred from constraint validation even with a custom error.
            let value = match observation {
                FieldsetObservation::Type => self.copy_dom_units(scope, &ascii(b"fieldset"))?,
                FieldsetObservation::WillValidate => scope.primitive(Primitive::Boolean(false))?,
                FieldsetObservation::ValidationMessage => self.copy_dom_units(scope, &[])?,
                FieldsetObservation::ValidationSuccess => {
                    scope.primitive(Primitive::Boolean(true))?
                }
            };
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn fieldset_set_custom_validity<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::Fieldset;
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
            let descriptor = html_field_set_element_set_custom_validity_0_error();
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(&descriptor, arguments.get(descriptor.index()))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            if !matches_receiver()? {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            let ArgumentValue::Present(message) = converted.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            let (value, _backing) = copy_native_data(&owner, &events, message.as_code_units())?;
            self.apply_tree_edit(
                scope,
                &owner,
                &events,
                DomSourceEdit::set_custom_validity(id, value),
                Completion::Undefined,
            )
        })();
        tree::tree_completion(scope, &events, result)
    }
}
