//! The native custom validity message setter for the six attached controls.

use super::{reflection::Receiver, *};
use surgeist_bindings_js::{
    DomString,
    conversion::{
        Argument, ArgumentValue, Type,
        declarations::{
            html_button_element_set_custom_validity_0_error,
            html_field_set_element_set_custom_validity_0_error,
            html_input_element_set_custom_validity_0_error,
            html_output_element_set_custom_validity_0_error,
            html_select_element_set_custom_validity_0_error,
            html_text_area_element_set_custom_validity_0_error,
        },
    },
};
use surgeist_dom::source::DomSourceEdit;

type ErrorDescriptor = fn() -> Argument<Type<DomString>>;

#[derive(Clone, Copy)]
pub(super) enum CustomValidityControl {
    Button,
    Fieldset,
    Input,
    Output,
    Select,
    Textarea,
}

impl CustomValidityControl {
    fn binding(self) -> (Receiver, ErrorDescriptor) {
        match self {
            Self::Button => (
                Receiver::Button,
                html_button_element_set_custom_validity_0_error,
            ),
            Self::Fieldset => (
                Receiver::Fieldset,
                html_field_set_element_set_custom_validity_0_error,
            ),
            Self::Input => (
                Receiver::Input,
                html_input_element_set_custom_validity_0_error,
            ),
            Self::Output => (
                Receiver::Output,
                html_output_element_set_custom_validity_0_error,
            ),
            Self::Select => (
                Receiver::Select,
                html_select_element_set_custom_validity_0_error,
            ),
            Self::Textarea => (
                Receiver::Textarea,
                html_text_area_element_set_custom_validity_0_error,
            ),
        }
    }

    pub(super) fn install<'scope>(
        self,
        scope: &mut Scope<'scope>,
        prototype: &Value<'scope>,
        capture: CaptureFunding<'_>,
    ) -> Result<(), DocumentGuestEventError> {
        let handler = capture(size_of::<(Capture, Self)>())?;
        let name = &ascii(b"setCustomValidity");
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.control_set_custom_validity(scope, arguments, self)
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
}

impl Capture {
    fn control_set_custom_validity<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        control: CustomValidityControl,
    ) -> Result<Completion<'scope>, Error> {
        let (receiver, error_descriptor) = control.binding();
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
            let descriptor = error_descriptor();
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
