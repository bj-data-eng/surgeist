//! Element operations over the canonical attribute list.

use super::*;

use surgeist_bindings_js::{
    DomString,
    conversion::{Argument, Nullable, Type},
};

#[derive(Clone, Copy)]
enum Method {
    HasAny,
    Names,
    Read(Lookup),
    WriteValue { namespaced: bool },
    RemoveValue(NameArguments),
    SetNode,
    RemoveNode,
}

type NamespaceArgument = fn() -> Argument<Nullable<Type<DomString>>>;

#[derive(Clone, Copy)]
struct NameArguments {
    name: fn() -> Argument<Type<DomString>>,
    namespace: Option<NamespaceArgument>,
}

impl NameArguments {
    const fn qualified(name: fn() -> Argument<Type<DomString>>) -> Self {
        Self {
            name,
            namespace: None,
        }
    }

    const fn expanded(
        namespace: NamespaceArgument,
        name: fn() -> Argument<Type<DomString>>,
    ) -> Self {
        Self {
            name,
            namespace: Some(namespace),
        }
    }
}

#[derive(Clone, Copy)]
struct Lookup {
    arguments: NameArguments,
    result: LookupResult,
}

#[derive(Clone, Copy)]
enum LookupResult {
    Value,
    Node,
    Presence,
}

struct AttributeName {
    name: DocumentString,
    namespace: Option<Namespace>,
    _name_backing: GuestBacking,
    _namespace_backing: Option<GuestBacking>,
}

pub(in crate::document_scripting::guest::interfaces::document::element) fn install<'scope>(
    scope: &mut Scope<'scope>,
    element: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (name, method, length) in [
        (&ascii(b"hasAttributes")[..], Method::HasAny, 0),
        (&ascii(b"getAttributeNames")[..], Method::Names, 0),
        (
            &ascii(b"getAttribute")[..],
            Method::Read(Lookup {
                arguments: NameArguments::qualified(
                    declarations::element_get_attribute_0_qualified_name,
                ),
                result: LookupResult::Value,
            }),
            1,
        ),
        (
            &ascii(b"getAttributeNS")[..],
            Method::Read(Lookup {
                arguments: NameArguments::expanded(
                    declarations::element_get_attribute_ns_0_namespace,
                    declarations::element_get_attribute_ns_0_local_name,
                ),
                result: LookupResult::Value,
            }),
            2,
        ),
        (
            &ascii(b"setAttribute")[..],
            Method::WriteValue { namespaced: false },
            2,
        ),
        (
            &ascii(b"setAttributeNS")[..],
            Method::WriteValue { namespaced: true },
            3,
        ),
        (
            &ascii(b"removeAttribute")[..],
            Method::RemoveValue(NameArguments::qualified(
                declarations::element_remove_attribute_0_qualified_name,
            )),
            1,
        ),
        (
            &ascii(b"removeAttributeNS")[..],
            Method::RemoveValue(NameArguments::expanded(
                declarations::element_remove_attribute_ns_0_namespace,
                declarations::element_remove_attribute_ns_0_local_name,
            )),
            2,
        ),
        (
            &ascii(b"hasAttribute")[..],
            Method::Read(Lookup {
                arguments: NameArguments::qualified(
                    declarations::element_has_attribute_0_qualified_name,
                ),
                result: LookupResult::Presence,
            }),
            1,
        ),
        (
            &ascii(b"hasAttributeNS")[..],
            Method::Read(Lookup {
                arguments: NameArguments::expanded(
                    declarations::element_has_attribute_ns_0_namespace,
                    declarations::element_has_attribute_ns_0_local_name,
                ),
                result: LookupResult::Presence,
            }),
            2,
        ),
        (
            &ascii(b"getAttributeNode")[..],
            Method::Read(Lookup {
                arguments: NameArguments::qualified(
                    declarations::element_get_attribute_node_0_qualified_name,
                ),
                result: LookupResult::Node,
            }),
            1,
        ),
        (
            &ascii(b"getAttributeNodeNS")[..],
            Method::Read(Lookup {
                arguments: NameArguments::expanded(
                    declarations::element_get_attribute_node_ns_0_namespace,
                    declarations::element_get_attribute_node_ns_0_local_name,
                ),
                result: LookupResult::Node,
            }),
            2,
        ),
        (&ascii(b"setAttributeNode")[..], Method::SetNode, 1),
        (&ascii(b"setAttributeNodeNS")[..], Method::SetNode, 1),
        (&ascii(b"removeAttributeNode")[..], Method::RemoveNode, 1),
    ] {
        let handler = capture(size_of::<(Capture, Method)>())?;
        let function = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.attribute_entry(scope, arguments, method, length)
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
        length: u32,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Element(element)) = receiver.and_then(|node| node.payload().id())
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element required"))?,
            ));
        };
        if arguments.len() < length as usize {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Missing attribute argument"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        match method {
            Method::HasAny => {
                let result = self
                    .callbacks
                    .with_read(&owner, &events, |read, _| {
                        let attributes = read
                            .node(element)?
                            .attributes()
                            .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                        Ok(!attributes.is_empty())
                    })
                    .and_then(|present| Ok(scope.primitive(Primitive::Boolean(present))?));
                complete(scope, &events, result)
            }
            Method::Names => {
                let result = self.attribute_names(scope, &owner, &events, element);
                complete(scope, &events, result)
            }
            Method::RemoveValue(method) => {
                let result = (|| {
                    let AttributeName {
                        name,
                        namespace,
                        _name_backing,
                        _namespace_backing,
                    } =
                        self.attribute_name_arguments(scope, &arguments, &owner, &events, method)?;
                    let edit = match namespace {
                        Some(namespace) => {
                            DomSourceEdit::remove_attribute_ns(element, namespace, name)
                        }
                        None => DomSourceEdit::remove_attribute(element, name),
                    };
                    self.apply_tree_edit(scope, &owner, &events, edit, Completion::Undefined)
                })();
                tree::tree_completion(scope, &events, result)
            }
            Method::WriteValue { namespaced } => {
                self.write_attribute(scope, arguments, (&owner, &events, element), namespaced)
            }
            Method::Read(method) => {
                let result = self
                    .lookup_attribute(scope, &arguments, &owner, &events, element, method)
                    .and_then(|observation| {
                        self.project_observation(scope, &owner, &events, observation)
                    });
                complete(scope, &events, result)
            }
            Method::SetNode | Method::RemoveNode => {
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
                    let (edit, result_kind) = if matches!(method, Method::RemoveNode) {
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

    fn attribute_names<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        element: NodeId,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let (native_names, _backing) = self.callbacks.with_read(owner, events, |read, _| {
            let attributes = read
                .node(element)?
                .attributes()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            let count = attributes.len();
            let bytes = count
                .checked_mul(size_of::<DomQualifiedName>())
                .ok_or(Error::Allocation)?;
            let backing = read.reserve(bytes)?;
            let mut names = attribute_name_buffer(count)?;
            attributes.visit_with(
                || read.step(),
                |attr| {
                    names.push(attr.name().clone());
                    Ok(())
                },
            )?;
            Ok((names, backing))
        })?;
        let count = native_names.len();
        let array_backing = scope.argument_copy_storage_bytes(count)?;
        let bytes = count
            .checked_mul(size_of::<Value<'scope>>())
            .and_then(|bytes| bytes.checked_add(array_backing))
            .ok_or(Error::Allocation)?;
        let _output_backing = events.reserve(owner, bytes)?;
        let mut names = attribute_name_buffer(count)?;
        for name in native_names {
            names.push(self.copy_attr_name(scope, owner, events, &name)?);
        }
        Ok(scope.array(&names)?)
    }

    fn attribute_name_arguments<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: &Arguments<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        selected: NameArguments,
    ) -> Result<AttributeName, DocumentGuestEventError> {
        let name = (selected.name)();
        let namespace = selected.namespace.map(|namespace| namespace());
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
        let (name, name_backing) = copy_native_data(owner, events, name.as_code_units())?;
        let (namespace, namespace_backing) =
            match namespace.as_ref().map(|namespace| namespace.get()) {
                Some(ArgumentValue::Present(Some(uri))) if !uri.as_code_units().is_empty() => {
                    let (uri, backing) = copy_native_data(owner, events, uri.as_code_units())?;
                    (Some(Namespace::from_uri(uri)), Some(backing))
                }
                Some(ArgumentValue::Present(_)) => (Some(Namespace::none()), None),
                Some(ArgumentValue::Missing) => return Err(Error::BackendUnavailable.into()),
                None => (None, None),
            };
        Ok(AttributeName {
            name,
            namespace,
            _name_backing: name_backing,
            _namespace_backing: namespace_backing,
        })
    }

    fn lookup_attribute<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: &Arguments<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        element: NodeId,
        lookup: Lookup,
    ) -> Result<Observation, DocumentGuestEventError> {
        let name =
            self.attribute_name_arguments(scope, arguments, owner, events, lookup.arguments)?;
        self.callbacks.with_read(owner, events, |read, _| {
            let attributes = read
                .node(element)?
                .attributes()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            let attribute = if let Some(namespace) = &name.namespace {
                attributes.get_named_item_ns_with(namespace, &name.name, || read.step())?
            } else {
                attributes.get_named_item_with(&name.name, || read.step())?
            };
            Ok(match lookup.result {
                LookupResult::Value => {
                    Observation::String(attribute.map(|attr| attr.value().clone()))
                }
                LookupResult::Node => Observation::Node(attribute.map(|attr| attr.id().node())),
                LookupResult::Presence => Observation::Boolean(attribute.is_some()),
            })
        })
    }
}

fn attribute_name_buffer<T>(count: usize) -> Result<Vec<T>, Error> {
    let mut names = Vec::new();
    names
        .try_reserve_exact(count)
        .map_err(|_| Error::Allocation)?;
    if names.capacity() != count {
        return Err(Error::Allocation);
    }
    Ok(names)
}
