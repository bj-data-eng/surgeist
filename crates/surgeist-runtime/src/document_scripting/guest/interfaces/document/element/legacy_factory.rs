//! Legacy Window element factories over canonical detached native creation.

use super::*;
use crate::document_scripting::guest::interfaces::document::character_data::copy_native_data;
use surgeist_bindings_js::{IntegerValue, conversion::declarations};
use surgeist_dom::edit::DomDetachedNodeCreation;

#[derive(Clone, Copy)]
enum Factory {
    Image,
    Option,
}

impl Factory {
    fn definition(self) -> (&'static str, &'static [u16], u32) {
        match self {
            Self::Image => ("HTMLImageElement", const { &ascii(b"Image") }, 2),
            Self::Option => ("HTMLOptionElement", const { &ascii(b"Option") }, 4),
        }
    }
}

impl ElementBindings {
    pub(in crate::document_scripting::guest::interfaces::document) fn install_legacy_element_factories<
        'scope,
    >(
        &self,
        scope: &mut Scope<'scope>,
        global: &Value<'scope>,
        capture: CaptureFunding<'_>,
    ) -> Result<(), DocumentGuestEventError> {
        for factory in [Factory::Option, Factory::Image] {
            let (interface, name, maximum) = factory.definition();
            let prototype = self.html_interface(interface, || Ok(()))?;
            let prototype = scope.open_callback_object(&prototype.prototype)?;
            let fallback = capture(size_of::<(Capture, &'static str)>())?;
            let action = constructor_capture(capture, size_of::<(Capture, Factory)>())?;
            let callback = ConstructorCallback::bounded(
                maximum,
                move |scope| fallback.html_element_prototype(scope, interface),
                move |scope, arguments| match factory {
                    Factory::Image => action.construct_image(scope, arguments),
                    Factory::Option => action.construct_option(scope, arguments),
                },
            );
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
        }
        Ok(())
    }
}

impl Capture {
    fn factory_document(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
    ) -> Result<DocumentId, DocumentGuestEventError> {
        let event = owner.active_event_request();
        self.callbacks
            .with_qualified_session(owner, event.as_ref(), |session, global| {
                session.dom().raw().dispatch_global_document_with(
                    global,
                    &owner.cancellation,
                    &mut || events.step(),
                )
            })
    }

    fn construct_image<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let width = declarations::html_image_element_image_width();
            let height = declarations::html_image_element_image_height();
            let (width, height) = self
                .callbacks
                .conversion
                .run(scope, |cx| {
                    Ok((
                        cx.argument(&width, arguments.get(width.index()))?,
                        cx.argument(&height, arguments.get(height.index()))?,
                    ))
                })
                .map_err(Error::from)?;
            let prototype = arguments.select_prototype(scope)?;
            scope.check_active()?;
            let document = self.factory_document(&owner, &events)?;
            let creation = DomDetachedNodeCreation::HtmlImage {
                width: converted_dimension(width.get())?,
                height: converted_dimension(height.get())?,
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
            let document = self.factory_document(&owner, &events)?;
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

fn converted_dimension(value: &ArgumentValue<IntegerValue>) -> Result<Option<u32>, Error> {
    match value {
        ArgumentValue::Missing => Ok(None),
        ArgumentValue::Present(IntegerValue::UnsignedLong(value)) => Ok(Some(*value)),
        ArgumentValue::Present(_) => Err(Error::BackendUnavailable),
    }
}
