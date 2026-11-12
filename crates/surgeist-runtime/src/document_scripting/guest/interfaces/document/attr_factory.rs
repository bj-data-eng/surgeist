//! Creation of canonical detached Attr nodes through a receiving Document.

use super::*;
use character_data::copy_native_data;
use surgeist_bindings_js::{conversion::declarations, dom_exception::DomExceptionName};
use surgeist_html::values::Namespace;

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(), DocumentGuestEventError> {
    for (name, namespaced, length) in [
        (&ascii(b"createAttribute")[..], false, 1),
        (&ascii(b"createAttributeNS")[..], true, 2),
    ] {
        let factory = capture(size_of::<(Capture, bool)>())?;
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                factory.create_attribute(scope, arguments, namespaced)
            }),
            FunctionMetadata { name, length },
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
    fn create_attribute<'scope>(
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
                scope.type_error(&ascii(b"Missing attribute name"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let (namespace, name) = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    let (namespace, name) = if namespaced {
                        let declaration = declarations::document_create_attribute_ns_0_namespace();
                        (
                            Some(
                                conversion
                                    .argument(&declaration, arguments.get(declaration.index()))?,
                            ),
                            declarations::document_create_attribute_ns_0_qualified_name(),
                        )
                    } else {
                        (None, declarations::document_create_attribute_0_local_name())
                    };
                    Ok((
                        namespace,
                        conversion.argument(&name, arguments.get(name.index()))?,
                    ))
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
            let mut batch = DomBatch::new();
            let ticket = if namespaced {
                let namespace = uri
                    .as_ref()
                    .map_or_else(Namespace::none, |(uri, _)| Namespace::from_uri(uri.clone()));
                batch.create_attribute_ns(document, namespace, name)
            } else {
                batch.create_attribute(document, name)
            };
            let create = |scope: &mut Scope<'scope>, prototypes: &DocumentPrototypes| {
                public.document.ordinary.create(
                    scope,
                    &owner,
                    &events,
                    ordinary::NodePrototype::Creation(&prototypes.attr),
                    |inbox, key| {
                        self.with_native(&owner, |session, _| {
                            let (_, node, watch) = session
                                .dom()
                                .raw_mut()
                                .apply_with_node_retirement_watch(
                                    batch,
                                    &ticket,
                                    inbox,
                                    key,
                                    events.limits.node_retirement,
                                    &owner.cancellation,
                                )
                                .map_err(DocumentScriptNativeError::from)?
                                .into_parts();
                            Ok((GuestNode::Attr(node), watch))
                        })
                    },
                )
            };
            if let Some(global) = public.other_document_origin(&owner, &events, document)? {
                public.with_origin(&events, global, |origin| create(scope, &origin.documents))
            } else {
                create(scope, &public.document.prototypes)
            }
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
            result => complete(scope, &events, result),
        }
    }
}
