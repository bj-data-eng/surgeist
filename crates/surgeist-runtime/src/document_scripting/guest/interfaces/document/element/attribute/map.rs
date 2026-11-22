//! The live NamedNodeMap of one native Element.

use super::{
    node::{NameArguments, attribute_name_buffer},
    *,
};
use surgeist_bindings_js::{IntegerKind, IntegerMode, IntegerValue, Intrinsic, ValueKind};
use surgeist_dom::tree::DomNamedNodeMap;

const ATTRIBUTES_REFERENCE: u32 = 1;
const ELEMENT_REFERENCE: u32 = 0;
const INTERFACE: &[u16] = &ascii(b"NamedNodeMap");
const GET_NAMED_ITEM: &[u16] = &ascii(b"getNamedItem");

struct Map {
    element: NodeId,
}

pub(in crate::document_scripting::guest::interfaces::document) struct MapProjection {
    prototype: RetainedCallbackObject,
    read: RetainedCallbackObject,
    keys: RetainedCallbackObject,
    named_getter: RetainedCallbackObject,
}

#[derive(Clone, Copy)]
enum Method {
    Length,
    Item,
    Get(NameArguments),
    Set,
    Remove(NameArguments),
}

pub(in crate::document_scripting::guest::interfaces::document::element) fn install<'scope>(
    scope: &mut Scope<'scope>,
    global: &Value<'scope>,
    element: &Value<'scope>,
    capture: CaptureFunding<'_>,
    fund: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
) -> Result<MapProjection, DocumentGuestEventError> {
    let prototype = scope.object(ObjectPrototype::Default)?;
    let constructor = scope.constructor(
        &illegal_constructor(capture)?,
        FunctionMetadata {
            name: INTERFACE,
            length: 0,
        },
    )?;
    scope.define_property(
        &constructor,
        PropertyKey::String(PROTOTYPE),
        PropertyDescriptor::Data {
            value: &prototype,
            writable: false,
            enumerable: false,
            configurable: false,
        },
    )?;
    let getter = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        element,
        &ascii(b"attributes"),
        &ascii(b"get attributes"),
        true,
        Callback::new(move |scope, arguments| getter.element_attributes(scope, arguments)),
    )?;
    let length = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        &prototype,
        &ascii(b"length"),
        &ascii(b"get length"),
        true,
        Callback::new(move |scope, arguments| {
            length.attribute_map_method(scope, arguments, Method::Length, 0)
        }),
    )?;
    let mut named_getter = None;
    for (name, method, length) in [
        (&ascii(b"item")[..], Method::Item, 1),
        (
            GET_NAMED_ITEM,
            Method::Get(NameArguments::qualified(
                declarations::named_node_map_get_named_item_0_qualified_name,
            )),
            1,
        ),
        (
            &ascii(b"getNamedItemNS")[..],
            Method::Get(NameArguments::expanded(
                declarations::named_node_map_get_named_item_ns_0_namespace,
                declarations::named_node_map_get_named_item_ns_0_local_name,
            )),
            2,
        ),
        (&ascii(b"setNamedItem")[..], Method::Set, 1),
        (&ascii(b"setNamedItemNS")[..], Method::Set, 1),
        (
            &ascii(b"removeNamedItem")[..],
            Method::Remove(NameArguments::qualified(
                declarations::named_node_map_remove_named_item_0_qualified_name,
            )),
            1,
        ),
        (
            &ascii(b"removeNamedItemNS")[..],
            Method::Remove(NameArguments::expanded(
                declarations::named_node_map_remove_named_item_ns_0_namespace,
                declarations::named_node_map_remove_named_item_ns_0_local_name,
            )),
            2,
        ),
    ] {
        let handler = capture(size_of::<(Capture, Method, u32)>())?;
        let function = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.attribute_map_method(scope, arguments, method, length)
            }),
            FunctionMetadata { name, length },
        )?;
        define_data(
            scope,
            &prototype,
            PropertyKey::String(name),
            &function,
            true,
            true,
            true,
        )?;
        if name == GET_NAMED_ITEM {
            named_getter = Some(function);
        }
    }
    install_interface(scope, global, INTERFACE, &constructor, &prototype)?;
    let iterator = scope.well_known_symbol(WellKnownSymbol::Iterator)?;
    let values = scope.intrinsic(Intrinsic::ArrayValues)?;
    define_data(
        scope,
        &prototype,
        PropertyKey::Value(&iterator),
        &values,
        true,
        false,
        true,
    )?;

    let read = capture(size_of::<Capture>())?;
    let read = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| read.attribute_map_property(scope, arguments)),
        FunctionMetadata {
            name: &[],
            length: 2,
        },
    )?;
    let keys = capture(size_of::<Capture>())?;
    let keys = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| keys.attribute_map_keys(scope, arguments)),
        FunctionMetadata {
            name: &[],
            length: 1,
        },
    )?;
    let prototype = scope.callback_object(&prototype)?;
    let read = scope.callback_object(&read)?;
    let keys = scope.callback_object(&keys)?;
    let named_getter = named_getter.ok_or(Error::BackendUnavailable)?;
    let named_getter = scope.callback_object(&named_getter)?;
    Ok(MapProjection {
        prototype: scope.retain_callback_object(prototype, fund)?,
        read: scope.retain_callback_object(read, fund)?,
        keys: scope.retain_callback_object(keys, fund)?,
        named_getter: scope.retain_callback_object(named_getter, fund)?,
    })
}

impl MapProjection {
    fn project<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        element: NodeId,
        receiver: &Value<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let prototype = scope.open_callback_object(&self.prototype)?;
        let callback = scope.callback_object(&prototype)?;
        let target = (owner, events, element);
        if callback.associated_realm().identity() == scope.realm_authority()?.identity() {
            return self.project_in_realm(scope, target, &prototype, receiver);
        }
        let receiver = scope.retain(receiver);
        let root = scope.in_callback_object_realm(&callback, |scope, prototype| {
            let receiver = scope.import(&receiver)?;
            let map = self.project_in_realm(scope, target, prototype, &receiver)?;
            Ok::<_, DocumentGuestEventError>(scope.retain(&map))
        })?;
        Ok(scope.import(&root)?)
    }

    fn project_in_realm<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events, element): (&NativeOwner, &GuestEvents, NodeId),
        prototype: &Value<'scope>,
        receiver: &Value<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let read = scope.open_callback_object(&self.read)?;
        let keys = scope.open_callback_object(&self.keys)?;
        let named_getter = scope.open_callback_object(&self.named_getter)?;
        let admit = cached_projection_admission(scope, owner, events, 2)?;
        let map = scope.project_native_object_with_collection_properties(
            ObjectPrototype::Object(prototype),
            Map { element },
            &read,
            &keys,
            &named_getter,
            admit,
        )?;
        scope.set_native_object_reference::<Map>(&map, ELEMENT_REFERENCE, Some(receiver))?;
        scope.set_native_object_reference::<GuestNodeProjection>(
            receiver,
            ATTRIBUTES_REFERENCE,
            Some(&map),
        )?;
        Ok(map)
    }
}

impl Capture {
    fn element_attributes<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = arguments.receiver();
        let Some(GuestNode::Element(element)) = tree::projected_node(scope, receiver)? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            if let Some(cached) = scope
                .native_object_reference::<GuestNodeProjection>(receiver, ATTRIBUTES_REFERENCE)?
            {
                return Ok(cached);
            }
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let (origin, attached) =
                self.callbacks
                    .with_read(&owner, &events, |read, attached| {
                        let origin = read
                            .node(element)?
                            .relevant_global()?
                            .ok_or(DocumentScriptNativeError::CreationGlobalUnavailable)?;
                        Ok((origin, attached))
                    })?;
            if origin == attached {
                public
                    .document
                    .element
                    .attribute_map
                    .project(scope, &owner, &events, element, receiver)
            } else {
                public.with_origin(&events, origin, |origin| {
                    origin
                        .elements
                        .attribute_map
                        .project(scope, &owner, &events, element, receiver)
                })
            }
        })();
        complete(scope, &events, result)
    }

    fn with_attribute_map<T>(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        element: NodeId,
        inspect: impl FnOnce(&NativeRead<'_>, DomNamedNodeMap<'_>) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        self.callbacks.with_read(owner, events, |read, _| {
            let attributes = read
                .node(element)?
                .attributes()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            inspect(read, attributes)
        })
    }

    fn attribute_map_method<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        method: Method,
        length: u32,
    ) -> Result<Completion<'scope>, Error> {
        let Some(map) = scope.native_object::<Map>(arguments.receiver())? else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"NamedNodeMap required"))?,
            ));
        };
        if arguments.len() < length as usize {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Missing attribute argument"))?,
            ));
        }
        let element = map.payload().element;
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let observation = match method {
                Method::Length => {
                    let length =
                        self.with_attribute_map(&owner, &events, element, |_, attributes| {
                            u32::try_from(attributes.len()).map_err(|_| Error::ArrayTooLong.into())
                        })?;
                    Observation::Number(f64::from(length))
                }
                Method::Item => {
                    let declaration = declarations::named_node_map_item_0_index();
                    let index = self
                        .callbacks
                        .conversion
                        .run(scope, |conversion| {
                            conversion.argument(&declaration, arguments.get(declaration.index()))
                        })
                        .map_err(Error::from)?;
                    let ArgumentValue::Present(IntegerValue::UnsignedLong(index)) = index.get()
                    else {
                        return Err(Error::BackendUnavailable.into());
                    };
                    scope.check_active()?;
                    let attr =
                        self.with_attribute_map(&owner, &events, element, |read, attributes| {
                            Ok(attributes
                                .item_with(*index as usize, || read.step())?
                                .map(|attribute| attribute.id().node()))
                        })?;
                    Observation::Node(attr)
                }
                Method::Get(selected) | Method::Remove(selected) => {
                    let name = self
                        .attribute_name_arguments(scope, &arguments, &owner, &events, selected)?;
                    let attr =
                        self.with_attribute_map(&owner, &events, element, |read, attributes| {
                            Ok(name
                                .resolve(attributes, || read.step())?
                                .map(|attribute| attribute.id()))
                        })?;
                    if matches!(method, Method::Remove(_)) {
                        let Some(attr) = attr else {
                            return Ok(Completion::Throw(
                                scope.dom_exception(DomExceptionName::NotFoundError, &[])?,
                            ));
                        };
                        let value = events
                            .public
                            .as_ref()
                            .ok_or(Error::ExpiredIdentity)?
                            .project_node(scope, &owner, &events, attr.node())?;
                        let edit = match name.namespace {
                            Some(namespace) => {
                                DomSourceEdit::remove_named_item_ns(element, namespace, name.name)
                            }
                            None => DomSourceEdit::remove_named_item(element, name.name),
                        };
                        return self.apply_attribute(
                            scope,
                            &owner,
                            &events,
                            edit,
                            None,
                            AttributeResult::Removed(attr, value),
                        );
                    }
                    Observation::Node(attr.map(DomAttrId::node))
                }
                Method::Set => {
                    let (_, values) = arguments.into_parts();
                    let value = values.into_iter().next().ok_or(Error::BackendUnavailable)?;
                    return self.edit_attribute_node(
                        scope,
                        (&owner, &events, element),
                        value,
                        false,
                    );
                }
            };
            Ok(Completion::Return(self.project_observation(
                scope,
                &owner,
                &events,
                observation,
            )?))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn attribute_map_property<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let map = scope
            .native_object::<Map>(arguments.receiver())?
            .ok_or(Error::ExpectedNativeObject)?;
        let element = map.payload().element;
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let key = arguments.get(0).ok_or(Error::BackendUnavailable)?;
            let node = match scope.kind(key)? {
                ValueKind::Number => {
                    let IntegerValue::UnsignedLong(index) =
                        scope.integer(key, IntegerKind::UnsignedLong, IntegerMode::EnforceRange)?
                    else {
                        return Err(Error::ExpectedPropertyKey.into());
                    };
                    self.with_attribute_map(&owner, &events, element, |read, attributes| {
                        Ok(attributes
                            .item_with(index as usize, || read.step())?
                            .map(|attribute| attribute.id().node()))
                    })?
                }
                ValueKind::String => {
                    let units = scope.dom_string(key)?;
                    let (name, _backing) =
                        copy_native_data(&owner, &events, units.as_code_units())?;
                    self.with_attribute_map(&owner, &events, element, |read, attributes| {
                        Ok(attributes
                            .named_property_with(&name, || read.step())?
                            .map(|attribute| attribute.id().node()))
                    })?
                }
                _ => return Err(Error::ExpectedPropertyKey.into()),
            };
            if arguments.get(1).is_some() {
                return Ok(Completion::Return(
                    scope.primitive(Primitive::Boolean(node.is_some()))?,
                ));
            }
            match node {
                Some(node) => Ok(Completion::Return(self.project_observation(
                    scope,
                    &owner,
                    &events,
                    Observation::Node(Some(node)),
                )?)),
                None => Ok(Completion::Undefined),
            }
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn attribute_map_keys<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let map = scope
            .native_object::<Map>(arguments.receiver())?
            .ok_or(Error::ExpectedNativeObject)?;
        let element = map.payload().element;
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            if arguments.is_empty() {
                let count =
                    self.with_attribute_map(&owner, &events, element, |_, attributes| {
                        Ok(attributes.len())
                    })?;
                let bytes = count
                    .checked_mul(size_of::<Value<'scope>>())
                    .ok_or(Error::Allocation)?;
                let _backing = events.reserve(&owner, bytes)?;
                let mut keys = attribute_name_buffer(count)?;
                for index in 0..u32::try_from(count).map_err(|_| Error::ArrayTooLong)? {
                    events.step()?;
                    keys.push(scope.primitive(Primitive::Number(f64::from(index)))?);
                }
                return Ok(scope.array(&keys)?);
            }
            let (names, _backing) =
                self.with_attribute_map(&owner, &events, element, |read, attributes| {
                    let count = attributes.len();
                    let bytes = count
                        .checked_mul(size_of::<DomQualifiedName>())
                        .ok_or(Error::Allocation)?;
                    let backing = read.reserve(bytes)?;
                    let mut names = attribute_name_buffer(count)?;
                    attributes.visit_named_property_candidates_with(
                        || read.step(),
                        |name| {
                            names.push(name.clone());
                            Ok(())
                        },
                    )?;
                    Ok((names, backing))
                })?;
            let bytes = names
                .len()
                .checked_mul(size_of::<Value<'scope>>())
                .ok_or(Error::Allocation)?;
            let _output_backing = events.reserve(&owner, bytes)?;
            let mut keys = attribute_name_buffer(names.len())?;
            for name in names {
                keys.push(self.copy_attr_name(scope, &owner, &events, &name)?);
            }
            Ok(scope.array(&keys)?)
        })();
        complete(scope, &events, result)
    }
}
