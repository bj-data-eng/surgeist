//! Intrinsic type and barred constraint validation for Fieldset and Output.

use super::{reflection::Receiver, *};

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
}
