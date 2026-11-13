//! Element operations over the canonical attribute list.

use super::*;

#[derive(Clone, Copy)]
enum Method {
    Read(ReadMethod),
    Set,
    Remove,
}

#[derive(Clone, Copy)]
enum ReadMethod {
    Value,
    ValueNs,
    Node,
    NodeNs,
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    element: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (name, method, length) in [
        (
            &ascii(b"getAttribute")[..],
            Method::Read(ReadMethod::Value),
            1,
        ),
        (
            &ascii(b"getAttributeNS")[..],
            Method::Read(ReadMethod::ValueNs),
            2,
        ),
        (
            &ascii(b"getAttributeNode")[..],
            Method::Read(ReadMethod::Node),
            1,
        ),
        (
            &ascii(b"getAttributeNodeNS")[..],
            Method::Read(ReadMethod::NodeNs),
            2,
        ),
        (&ascii(b"setAttributeNode")[..], Method::Set, 1),
        (&ascii(b"setAttributeNodeNS")[..], Method::Set, 1),
        (&ascii(b"removeAttributeNode")[..], Method::Remove, 1),
    ] {
        let handler = capture(size_of::<(Capture, Method)>())?;
        let function = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.attribute_entry(scope, arguments, method)
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
    fn attribute_entry<'scope>(
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
        let required = match method {
            Method::Read(ReadMethod::ValueNs | ReadMethod::NodeNs) => 2,
            _ => 1,
        };
        if arguments.len() < required {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Missing attribute argument"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        match method {
            Method::Read(method) => {
                let result = self
                    .lookup_attribute(scope, &arguments, &owner, &events, element, method)
                    .and_then(|observation| {
                        self.project_observation(scope, &owner, &events, observation)
                    });
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
                        let registration = owner
                            .attachment
                            .get()
                            .and_then(|attachment| attachment.registration.upgrade())
                            .ok_or(DocumentScriptNativeError::Revoked)?;
                        (
                            DomSourceEdit::attach_attribute_for_script(
                                element,
                                attribute,
                                &registration,
                            ),
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
        method: ReadMethod,
    ) -> Result<Observation, DocumentGuestEventError> {
        let (namespace, name) = match method {
            ReadMethod::Value => (None, declarations::element_get_attribute_0_qualified_name()),
            ReadMethod::ValueNs => (
                Some(declarations::element_get_attribute_ns_0_namespace()),
                declarations::element_get_attribute_ns_0_local_name(),
            ),
            ReadMethod::Node => (
                None,
                declarations::element_get_attribute_node_0_qualified_name(),
            ),
            ReadMethod::NodeNs => (
                Some(declarations::element_get_attribute_node_ns_0_namespace()),
                declarations::element_get_attribute_node_ns_0_local_name(),
            ),
        };
        let namespaced = namespace.is_some();
        let (namespace, name) = self
            .callbacks
            .conversion
            .run(scope, |cx| {
                let namespace = namespace
                    .as_ref()
                    .map(|namespace| cx.argument(namespace, arguments.get(namespace.index())))
                    .transpose()?;
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
            Ok(match method {
                ReadMethod::Value | ReadMethod::ValueNs => {
                    Observation::String(attribute.map(|attr| attr.value().clone()))
                }
                ReadMethod::Node | ReadMethod::NodeNs => {
                    Observation::Node(attribute.map(|attr| attr.id().node()))
                }
            })
        })
    }
}
