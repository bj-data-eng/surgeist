//! Intrinsic type and barred constraint validation for Fieldset and Output.

use super::{reflection::Receiver, *};
use surgeist_bindings_js::conversion::{
    ArgumentValue,
    declarations::{
        html_field_set_element_set_custom_validity_0_error,
        html_output_element_set_custom_validity_0_error,
    },
};
use surgeist_dom::source::DomSourceEdit;

#[derive(Clone, Copy)]
pub(super) enum BarredControl {
    Fieldset,
    Output,
}

impl BarredControl {
    fn receiver(self) -> Receiver {
        match self {
            Self::Fieldset => Receiver::Fieldset,
            Self::Output => Receiver::Output,
        }
    }

    fn type_units(self) -> &'static [u16] {
        match self {
            Self::Fieldset => const { &ascii(b"fieldset") },
            Self::Output => const { &ascii(b"output") },
        }
    }

    pub(super) fn install_type<'scope>(
        self,
        scope: &mut Scope<'scope>,
        prototype: &Value<'scope>,
        capture: CaptureFunding<'_>,
    ) -> Result<(), DocumentGuestEventError> {
        self.install_observation(
            scope,
            prototype,
            capture,
            &ascii(b"type"),
            &ascii(b"get type"),
            BarredObservation::Type,
        )
    }

    pub(super) fn install_validation<'scope>(
        self,
        scope: &mut Scope<'scope>,
        prototype: &Value<'scope>,
        capture: CaptureFunding<'_>,
    ) -> Result<(), DocumentGuestEventError> {
        self.install_observation(
            scope,
            prototype,
            capture,
            &ascii(b"willValidate"),
            &ascii(b"get willValidate"),
            BarredObservation::WillValidate,
        )?;
        self.install_observation(
            scope,
            prototype,
            capture,
            &ascii(b"validationMessage"),
            &ascii(b"get validationMessage"),
            BarredObservation::ValidationMessage,
        )?;
        for name in [&ascii(b"checkValidity")[..], &ascii(b"reportValidity")[..]] {
            let handler = capture(size_of::<(Capture, BarredControl)>())?;
            let method = scope.fixed_arity_function(
                &Callback::new(move |scope, arguments| {
                    handler.barred_control_observation(
                        scope,
                        arguments,
                        self,
                        BarredObservation::ValidationSuccess,
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
        let handler = capture(size_of::<(Capture, BarredControl)>())?;
        let name = &ascii(b"setCustomValidity");
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.barred_control_set_custom_validity(scope, arguments, self)
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

    fn install_observation<'scope>(
        self,
        scope: &mut Scope<'scope>,
        prototype: &Value<'scope>,
        capture: CaptureFunding<'_>,
        name: &[u16],
        getter: &[u16],
        observation: BarredObservation,
    ) -> Result<(), DocumentGuestEventError> {
        let handler = capture(size_of::<(Capture, BarredControl, BarredObservation)>())?;
        install_getter(
            scope,
            prototype,
            name,
            getter,
            true,
            Callback::new(move |scope, arguments| {
                handler.barred_control_observation(scope, arguments, self, observation)
            }),
        )?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum BarredObservation {
    Type,
    WillValidate,
    ValidationMessage,
    ValidationSuccess,
}

impl Capture {
    fn barred_control_observation<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        control: BarredControl,
        observation: BarredObservation,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = control.receiver();
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
            // Both controls are barred from constraint validation even with a custom error.
            let value = match observation {
                BarredObservation::Type => self.copy_dom_units(scope, control.type_units())?,
                BarredObservation::WillValidate => scope.primitive(Primitive::Boolean(false))?,
                BarredObservation::ValidationMessage => self.copy_dom_units(scope, &[])?,
                BarredObservation::ValidationSuccess => {
                    scope.primitive(Primitive::Boolean(true))?
                }
            };
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn barred_control_set_custom_validity<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        control: BarredControl,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = control.receiver();
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
            let descriptor = match control {
                BarredControl::Fieldset => html_field_set_element_set_custom_validity_0_error(),
                BarredControl::Output => html_output_element_set_custom_validity_0_error(),
            };
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
