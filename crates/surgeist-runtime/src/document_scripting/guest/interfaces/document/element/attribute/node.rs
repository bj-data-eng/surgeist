//! Element operations over the canonical Attr identities.

use super::*;

#[derive(Clone, Copy)]
enum Method {
    Get,
    GetNs,
    Set,
    Remove,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    element: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (name, method, length) in [
        (&ascii(b"getAttributeNode")[..], Method::Get, 1),
        (&ascii(b"getAttributeNodeNS")[..], Method::GetNs, 2),
        (&ascii(b"setAttributeNode")[..], Method::Set, 1),
        (&ascii(b"setAttributeNodeNS")[..], Method::Set, 1),
        (&ascii(b"removeAttributeNode")[..], Method::Remove, 1),
    ] {
        let handler = capture(size_of::<(Capture, Method)>())?;
        let function = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.attribute_node(scope, arguments, method)
            }),
            FunctionMetadata { name, length },
        )?;
        define_data(
            scope,
            element,
            PropertyKey::String(name),
            &function,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Capture {
    fn attribute_node<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        method: Method,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Element(element)) = receiver.and_then(|node| node.payload().id())
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element required"))?,
            ));
        };
        let namespaced = matches!(method, Method::GetNs);
        let required = if namespaced { 2 } else { 1 };
        if arguments.len() < required {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Missing attribute argument"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        match method {
            Method::Get | Method::GetNs => {
                let result = self
                    .lookup_attribute(scope, &arguments, &owner, &events, element, namespaced)
                    .and_then(|attribute| project(scope, &owner, &events, attribute));
                complete(scope, &events, result)
            }
            Method::Set | Method::Remove => {
                let (_, values) = arguments.into_parts();
                let value = values.into_iter().next().ok_or(Error::BackendUnavailable)?;
                let operand = scope.native_object::<GuestNodeProjection>(&value)?;
                let Some(GuestNode::Attr(node)) = operand.and_then(|node| node.payload().id())
                else {
                    return Ok(Completion::Throw(
                        scope.type_error(&ascii(b"Attr required"))?,
                    ));
                };
                let result = (|| {
                    let attribute = self.callbacks.with_read(&owner, &events, |read, _| {
                        let attr = read
                            .node(node)?
                            .as_attr()
                            .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                        Ok(attr.id())
                    })?;
                    let (edit, result_kind) = if matches!(method, Method::Remove) {
                        (
                            DomSourceEdit::remove_attribute_node(element, attribute),
                            AttributeResult::Removed(attribute, value),
                        )
                    } else {
                        (
                            DomSourceEdit::attach_attribute(element, attribute),
                            AttributeResult::Attached(attribute, value),
                        )
                    };
                    self.apply_attribute(scope, &owner, &events, edit, None, result_kind)
                })();
                match result {
                    Ok(completion) => Ok(completion),
                    Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                        Err(error)
                    }
                    Err(error) => refuse(scope, &events.refusal, error),
                }
            }
        }
    }

    fn lookup_attribute<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: &Arguments<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        element: NodeId,
        namespaced: bool,
    ) -> Result<Option<DomAttrId>, DocumentGuestEventError> {
        let (namespace, name) = self
            .callbacks
            .conversion
            .run(scope, |cx| {
                let (namespace, name) = if namespaced {
                    let namespace = declarations::element_get_attribute_node_ns_0_namespace();
                    (
                        Some(cx.argument(&namespace, arguments.get(namespace.index()))?),
                        declarations::element_get_attribute_node_ns_0_local_name(),
                    )
                } else {
                    (
                        None,
                        declarations::element_get_attribute_node_0_qualified_name(),
                    )
                };
                Ok((namespace, cx.argument(&name, arguments.get(name.index()))?))
            })
            .map_err(Error::from)?;
        scope.check_active()?;
        let ArgumentValue::Present(name) = name.get() else {
            return Err(Error::BackendUnavailable.into());
        };
        let (name, _name_backing) = copy_native_data(owner, events, name.as_code_units())?;
        let uri = match namespace.as_ref().map(|namespace| namespace.get()) {
            Some(ArgumentValue::Present(Some(uri))) if !uri.as_code_units().is_empty() => {
                Some(copy_native_data(owner, events, uri.as_code_units())?)
            }
            Some(ArgumentValue::Missing) => return Err(Error::BackendUnavailable.into()),
            _ => None,
        };
        let (namespace, _uri_backing) = match uri {
            Some((uri, backing)) => (Namespace::from_uri(uri), Some(backing)),
            None => (Namespace::none(), None),
        };
        self.callbacks.with_read(owner, events, |read, _| {
            let attributes = read
                .node(element)?
                .attributes()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            let attribute = if namespaced {
                attributes.get_named_item_ns_with(&namespace, &name, || read.step())?
            } else {
                attributes.get_named_item_with(&name, || read.step())?
            };
            Ok(attribute.map(|attribute| attribute.id()))
        })
    }
}

pub(super) fn project<'scope>(
    scope: &mut Scope<'scope>,
    owner: &NativeOwner,
    events: &GuestEvents,
    attribute: Option<DomAttrId>,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    match attribute {
        Some(attribute) => events
            .public
            .as_ref()
            .ok_or(Error::ExpiredIdentity)?
            .project_node(scope, owner, events, attribute.node()),
        None => Ok(scope.primitive(Primitive::Null)?),
    }
}
