//! Creation of canonical detached Elements through a receiving Document.

use super::*;
use character_data::copy_native_data;
use surgeist_bindings_js::{
    conversion::declarations, declarations::ElementCreationArgument,
    dom_exception::DomExceptionName,
};
use surgeist_dom::{
    custom_element::DomRegistryBinding,
    edit::{DomElementCreationName, DomElementCreationOptions},
};
use surgeist_html::values::Namespace;

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(), DocumentGuestEventError> {
    for (name, namespaced, length, maximum) in [
        (
            &ascii(b"createElement")[..],
            false,
            1,
            declarations::DOCUMENT_CREATE_ELEMENT_MAX_ARGUMENTS,
        ),
        (
            &ascii(b"createElementNS")[..],
            true,
            2,
            declarations::DOCUMENT_CREATE_ELEMENT_NS_MAX_ARGUMENTS,
        ),
    ] {
        let factory = capture(size_of::<(Capture, bool)>())?;
        let method = scope.bounded_function(
            &Callback::new(move |scope, arguments| {
                factory.create_element(scope, arguments, namespaced)
            }),
            FunctionMetadata { name, length },
            maximum,
        )?;
        define_data(
            scope,
            document,
            PropertyKey::String(name),
            &method,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Capture {
    fn create_element<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        namespaced: bool,
    ) -> Result<Completion<'scope>, Error> {
        let projection = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Document(document)) = projection.and_then(|node| node.payload().id())
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let (namespace, name, options) = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    if namespaced {
                        let converted = declarations::convert_document_create_element_ns(
                            conversion, &arguments,
                        )?;
                        Ok((
                            Some(converted.namespace),
                            converted.qualified_name,
                            converted.options,
                        ))
                    } else {
                        let converted =
                            declarations::convert_document_create_element(conversion, &arguments)?;
                        Ok((None, converted.local_name, converted.options))
                    }
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let (name, _name_backing) =
                copy_native_data(&owner, &events, name.get().as_code_units())?;
            let uri = namespace
                .as_ref()
                .and_then(|value| value.get().as_ref())
                .filter(|uri| !uri.as_code_units().is_empty())
                .map(|uri| copy_native_data(&owner, &events, uri.as_code_units()))
                .transpose()?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let name = if namespaced {
                let namespace = uri
                    .as_ref()
                    .map_or_else(Namespace::none, |(uri, _)| Namespace::from_uri(uri.clone()));
                DomElementCreationName::Qualified(namespace, name)
            } else {
                DomElementCreationName::Local(name)
            };
            let (is, registry) = match options.get() {
                ElementCreationArgument::Options(options) => (
                    options
                        .is
                        .as_ref()
                        .map(|value| copy_native_data(&owner, &events, value.as_code_units()))
                        .transpose()?,
                    options.custom_element_registry,
                ),
                ElementCreationArgument::String(_) => (None, None),
            };
            let (is, _is_backing) = is.map_or((None, None), |(value, backing)| {
                (Some(value), Some(backing))
            });
            let registry = match registry {
                None => None,
                Some(None) => Some(DomRegistryBinding::Null),
                Some(Some(identity)) => Some(DomRegistryBinding::Registry(
                    self.callbacks.with_read(&owner, &events, |read, _| {
                        public.native_registry(read, identity)
                    })?,
                )),
            };
            let options = DomElementCreationOptions { is, registry };
            public.document.ordinary.create_element(
                scope,
                self,
                (&owner, &events),
                public,
                (document, name, options),
            )
        })();
        match result {
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::InvalidCharacter),
            ))) => Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::InvalidCharacterError, &[])?,
            )),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::Namespace),
            ))) => Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::NamespaceError, &[])?,
            )),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                DocumentScriptNativeError::Dom(DomError::NotSupported),
            ))) => Ok(Completion::Throw(
                scope.dom_exception(DomExceptionName::NotSupportedError, &[])?,
            )),
            result => complete(scope, &events, result),
        }
    }
}
