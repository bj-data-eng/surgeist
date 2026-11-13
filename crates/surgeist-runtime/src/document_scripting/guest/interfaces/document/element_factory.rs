//! Creation of canonical detached Elements through a receiving Document.

use super::*;
use character_data::copy_native_data;
use surgeist_bindings_js::{
    conversion::declarations, declarations::ElementCreationArgument,
    dom_exception::DomExceptionName,
};
use surgeist_dom::{DomElementCreationName, DomElementCreationOptions, DomRegistryBinding};
use surgeist_html::values::Namespace;

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(), DocumentGuestEventError> {
    for (name, namespaced, length) in [
        (&ascii(b"createElement")[..], false, 1),
        (&ascii(b"createElementNS")[..], true, 2),
    ] {
        let factory = capture(size_of::<(Capture, bool)>())?;
        let method = scope.bounded_function(
            &Callback::new(move |scope, arguments| {
                factory.create_element(scope, arguments, namespaced)
            }),
            FunctionMetadata { name, length },
            length + 1,
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
        if arguments.len() < if namespaced { 2 } else { 1 } {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Missing element name"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let (namespace, name, options) = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    let (namespace, name) = if namespaced {
                        let declaration = declarations::document_create_element_ns_0_namespace();
                        (
                            Some(
                                conversion
                                    .argument(&declaration, arguments.get(declaration.index()))?,
                            ),
                            declarations::document_create_element_ns_0_qualified_name(),
                        )
                    } else {
                        (None, declarations::document_create_element_0_local_name())
                    };
                    let name = conversion.argument(&name, arguments.get(name.index()))?;
                    let options = if namespaced {
                        declarations::document_create_element_ns_0_options()
                    } else {
                        declarations::document_create_element_0_options()
                    };
                    let options = conversion.argument(&options, arguments.get(options.index()))?;
                    Ok((namespace, name, options))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let ArgumentValue::Present(name) = name.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            let (name, _name_backing) = copy_native_data(&owner, &events, name.as_code_units())?;
            let uri = match namespace.as_ref().map(|value| value.get()) {
                Some(ArgumentValue::Present(Some(uri))) if !uri.as_code_units().is_empty() => {
                    Some(copy_native_data(&owner, &events, uri.as_code_units())?)
                }
                Some(ArgumentValue::Missing) => return Err(Error::BackendUnavailable.into()),
                _ => None,
            };
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
                ArgumentValue::Present(ElementCreationArgument::Options(options)) => (
                    options
                        .is
                        .as_ref()
                        .map(|value| copy_native_data(&owner, &events, value.as_code_units()))
                        .transpose()?,
                    options.custom_element_registry,
                ),
                ArgumentValue::Present(ElementCreationArgument::String(_)) => (None, None),
                ArgumentValue::Missing => return Err(Error::BackendUnavailable.into()),
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
