//! The Window Option factory over canonical detached native creation.

use super::*;
use crate::document_scripting::guest::interfaces::document::character_data::copy_native_data;
use surgeist_bindings_js::conversion::declarations;
use surgeist_dom::edit::DomDetachedNodeCreation;

const OPTION_INTERFACE: &str = "HTMLOptionElement";

impl ElementBindings {
    pub(in crate::document_scripting::guest::interfaces::document) fn install_option_factory<
        'scope,
    >(
        &self,
        scope: &mut Scope<'scope>,
        global: &Value<'scope>,
        capture: CaptureFunding<'_>,
    ) -> Result<(), DocumentGuestEventError> {
        let prototype = self.html_interface(OPTION_INTERFACE, || Ok(()))?;
        let prototype = scope.open_callback_object(&prototype.prototype)?;
        let fallback = capture(size_of::<Capture>())?;
        let action = constructor_capture(capture, size_of::<Capture>())?;
        let callback = ConstructorCallback::bounded(
            4,
            move |scope| fallback.html_element_prototype(scope, OPTION_INTERFACE),
            move |scope, arguments| action.construct_option(scope, arguments),
        );
        let name = &ascii(b"Option");
        let factory = scope.constructor(&callback, FunctionMetadata { name, length: 0 })?;
        scope.define_property(
            &factory,
            PropertyKey::String(PROTOTYPE),
            PropertyDescriptor::Data {
                value: &prototype,
                writable: false,
                enumerable: false,
                configurable: false,
            },
        )?;
        define_data(
            scope,
            global,
            PropertyKey::String(name),
            &factory,
            true,
            false,
            true,
        )?;
        Ok(())
    }
}

impl Capture {
    fn construct_option<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let text = declarations::html_option_element_option_text();
            let value = declarations::html_option_element_option_value();
            let default_selected = declarations::html_option_element_option_default_selected();
            let selected = declarations::html_option_element_option_selected();
            let (text, value, default_selected, selected) = self
                .callbacks
                .conversion
                .run(scope, |cx| {
                    let text = cx.argument(&text, arguments.get(text.index()))?;
                    let value = cx.argument(&value, arguments.get(value.index()))?;
                    let default_selected =
                        cx.argument(&default_selected, arguments.get(default_selected.index()))?;
                    let selected = cx.argument(&selected, arguments.get(selected.index()))?;
                    Ok((text, value, default_selected, selected))
                })
                .map_err(Error::from)?;
            let prototype = arguments.select_prototype(scope)?;
            scope.check_active()?;
            let (
                ArgumentValue::Present(text),
                ArgumentValue::Present(default_selected),
                ArgumentValue::Present(selected),
            ) = (text.get(), default_selected.get(), selected.get())
            else {
                return Err(Error::BackendUnavailable.into());
            };
            let event = owner.active_event_request();
            let document = self.callbacks.with_qualified_session(
                &owner,
                event.as_ref(),
                |session, global| {
                    session.dom().raw().dispatch_global_document_with(
                        global,
                        &owner.cancellation,
                        &mut || events.step(),
                    )
                },
            )?;
            let (text, _text_backing) = copy_native_data(&owner, &events, text.as_code_units())?;
            let (value, _value_backing) = match value.get() {
                ArgumentValue::Missing => (None, None),
                ArgumentValue::Present(value) => {
                    let (value, backing) =
                        copy_native_data(&owner, &events, value.as_code_units())?;
                    (Some(value), Some(backing))
                }
            };
            let creation = DomDetachedNodeCreation::HtmlOption {
                text,
                value,
                default_selected: *default_selected,
                selected: *selected,
            };
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            public.document.ordinary.create_detached(
                scope,
                self,
                (&owner, &events),
                ordinary::NodePrototype::Selected(prototype.prototype()),
                (document, creation),
            )
        })();
        match result {
            Ok(value) => Ok(ConstructorCompletion::Return(value)),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }
}
