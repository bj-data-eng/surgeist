//! Ordinary Element traversal and live descendant queries.

use super::*;
use super::{element::HTML_NAMESPACE, reflection::html_element_is, tree_mutations::Receiver};
use surgeist_bindings_js::{
    DomString,
    conversion::{Argument, Nullable, Type, declarations},
};
use surgeist_dom::tree::{DomDescendantQueryRoot, DomHtmlCollectionLimits, DomNode};

const CHILDREN_REFERENCE: u32 = 11;
const GET_BY_TAG: &[u16] = &ascii(b"getElementsByTagName");
const GET_BY_TAG_NS: &[u16] = &ascii(b"getElementsByTagNameNS");
const GET_BY_CLASS: &[u16] = &ascii(b"getElementsByClassName");

#[derive(Clone, Copy)]
pub(super) struct DescendantCollection {
    member: &'static [u16],
    getter: &'static [u16],
    interface: &'static str,
    required: &'static [u16],
    local_name: &'static [u16],
    cache_reference: u32,
}

pub(super) const MAP_AREAS: DescendantCollection = DescendantCollection {
    member: &ascii(b"areas"),
    getter: &ascii(b"get areas"),
    interface: "HTMLMapElement",
    required: &ascii(b"HTMLMapElement required"),
    local_name: &ascii(b"area"),
    cache_reference: 19,
};

pub(super) const DATALIST_OPTIONS: DescendantCollection = DescendantCollection {
    member: &ascii(b"options"),
    getter: &ascii(b"get options"),
    interface: "HTMLDataListElement",
    required: &ascii(b"HTMLDataListElement required"),
    local_name: &ascii(b"option"),
    cache_reference: 20,
};

impl DescendantCollection {
    pub(super) fn install<'scope>(
        self,
        scope: &mut Scope<'scope>,
        prototype: &Value<'scope>,
        capture: CaptureFunding<'_>,
    ) -> Result<(), DocumentGuestEventError> {
        let handler = capture(size_of::<(Capture, Self)>())?;
        install_getter(
            scope,
            prototype,
            self.member,
            self.getter,
            true,
            Callback::new(move |scope, arguments| {
                handler.descendant_collection(scope, arguments, self)
            }),
        )?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum ParentMember {
    First,
    Last,
    Count,
}

#[derive(Clone, Copy)]
pub(super) enum QueryTarget {
    Document,
    Element,
}

type StringArgument = fn() -> Argument<Type<DomString>>;
type NamespaceArgument = fn() -> Argument<Nullable<Type<DomString>>>;

#[derive(Clone, Copy)]
enum Query {
    Qualified(StringArgument),
    Expanded {
        namespace: NamespaceArgument,
        local_name: StringArgument,
    },
    Classes(StringArgument),
}

impl QueryTarget {
    fn queries(self) -> [Query; 3] {
        use declarations::*;
        match self {
            Self::Document => [
                Query::Qualified(document_get_elements_by_tag_name_0_qualified_name),
                Query::Expanded {
                    namespace: document_get_elements_by_tag_name_ns_0_namespace,
                    local_name: document_get_elements_by_tag_name_ns_0_local_name,
                },
                Query::Classes(document_get_elements_by_class_name_0_class_names),
            ],
            Self::Element => [
                Query::Qualified(element_get_elements_by_tag_name_0_qualified_name),
                Query::Expanded {
                    namespace: element_get_elements_by_tag_name_ns_0_namespace,
                    local_name: element_get_elements_by_tag_name_ns_0_local_name,
                },
                Query::Classes(element_get_elements_by_class_name_0_class_names),
            ],
        }
    }

    fn root(self, node: GuestNode) -> Option<DomDescendantQueryRoot> {
        match (self, node) {
            (Self::Document, GuestNode::Document(document)) => {
                Some(DomDescendantQueryRoot::Document(document))
            }
            (Self::Element, GuestNode::Element(element)) => {
                Some(DomDescendantQueryRoot::Element(element))
            }
            _ => None,
        }
    }
}

impl Query {
    fn metadata(self) -> FunctionMetadata<'static> {
        match self {
            Self::Qualified(_) => FunctionMetadata {
                name: GET_BY_TAG,
                length: 1,
            },
            Self::Expanded { .. } => FunctionMetadata {
                name: GET_BY_TAG_NS,
                length: 2,
            },
            Self::Classes(_) => FunctionMetadata {
                name: GET_BY_CLASS,
                length: 1,
            },
        }
    }

    fn arguments(self) -> (Option<NamespaceArgument>, StringArgument) {
        match self {
            Self::Qualified(name) | Self::Classes(name) => (None, name),
            Self::Expanded {
                namespace,
                local_name,
            } => (Some(namespace), local_name),
        }
    }
}

pub(super) fn install_parent<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    receiver: Receiver,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let children = capture(size_of::<(Capture, Receiver)>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"children"),
        &ascii(b"get children"),
        true,
        Callback::new(move |scope, arguments| {
            children.element_children(scope, arguments, receiver)
        }),
    )?;
    let getters: [(&[u16], &[u16], ParentMember); 3] = [
        (
            &ascii(b"firstElementChild"),
            &ascii(b"get firstElementChild"),
            ParentMember::First,
        ),
        (
            &ascii(b"lastElementChild"),
            &ascii(b"get lastElementChild"),
            ParentMember::Last,
        ),
        (
            &ascii(b"childElementCount"),
            &ascii(b"get childElementCount"),
            ParentMember::Count,
        ),
    ];
    for (key, name, member) in getters {
        let handler = capture(size_of::<(Capture, Receiver, ParentMember)>())?;
        install_getter(
            scope,
            prototype,
            key,
            name,
            true,
            Callback::new(move |scope, arguments| {
                handler.read_element_child(scope, arguments, receiver, member)
            }),
        )?;
    }
    Ok(())
}

pub(super) fn install_siblings<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    receiver: Receiver,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let getters: [(&[u16], &[u16], DomSiblingDirection); 2] = [
        (
            &ascii(b"previousElementSibling"),
            &ascii(b"get previousElementSibling"),
            DomSiblingDirection::Previous,
        ),
        (
            &ascii(b"nextElementSibling"),
            &ascii(b"get nextElementSibling"),
            DomSiblingDirection::Next,
        ),
    ];
    for (key, name, direction) in getters {
        let handler = capture(size_of::<(Capture, Receiver, DomSiblingDirection)>())?;
        install_getter(
            scope,
            prototype,
            key,
            name,
            true,
            Callback::new(move |scope, arguments| {
                handler.element_sibling(scope, arguments, receiver, direction)
            }),
        )?;
    }
    Ok(())
}

pub(super) fn install_queries<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    target: QueryTarget,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for query in target.queries() {
        let handler = capture(size_of::<(Capture, QueryTarget, Query)>())?;
        let metadata = query.metadata();
        let method = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                handler.descendant_elements(scope, arguments, target, query)
            }),
            metadata,
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(metadata.name),
            &method,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Capture {
    fn element_children<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
    ) -> Result<Completion<'scope>, Error> {
        let wrapper = arguments.receiver();
        let Some(parent) =
            tree::projected_node(scope, wrapper)?.filter(|node| receiver.accepts(*node))
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"ParentNode receiver required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            // The traced collection can outlive its canonical parent.
            let origin = self
                .callbacks
                .with_read(&owner, &events, |read, attached| {
                    collection_origin(read.node(parent.node())?, attached)
                })?;
            if let Some(collection) =
                scope.native_object_reference::<GuestNodeProjection>(wrapper, CHILDREN_REFERENCE)?
            {
                return Ok(Completion::Return(collection));
            }
            let collection = self.callbacks.with_read(&owner, &events, |read, _| {
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                Ok(dom.raw().child_elements(parent.node(), read.cancel)?)
            })?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let value = public.with_collection_projection(&events, origin, |projection| {
                projection.project_for_root(
                    scope,
                    (&owner, &events, collection),
                    wrapper,
                    Some(CHILDREN_REFERENCE),
                    None,
                )
            })?;
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn read_element_child<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
        member: ParentMember,
    ) -> Result<Completion<'scope>, Error> {
        let Some(parent) = tree::projected_node(scope, arguments.receiver())?
            .filter(|node| receiver.accepts(*node))
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"ParentNode receiver required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let observed = self.callbacks.with_read(&owner, &events, |read, _| {
                read.step()?;
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let collection = dom.raw().child_elements(parent.node(), read.cancel)?;
                let view = dom.raw().read_html_collection(
                    &collection,
                    DomHtmlCollectionLimits::new(events.limits.work, 0, 0),
                    || events.budget.charge_work(read.account_scope, 1),
                    read.cancel,
                )?;
                Ok(match member {
                    ParentMember::First => Observation::Node(view.item(0)?),
                    ParentMember::Last => Observation::Node(view.last_item()?),
                    ParentMember::Count => {
                        let count =
                            u32::try_from(view.length()?).map_err(|_| Error::ArrayTooLong)?;
                        Observation::Number(f64::from(count))
                    }
                })
            })?;
            Ok(Completion::Return(
                self.project_observation(scope, &owner, &events, observed)?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn element_sibling<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
        direction: DomSiblingDirection,
    ) -> Result<Completion<'scope>, Error> {
        let Some(node) = tree::projected_node(scope, arguments.receiver())?
            .filter(|node| receiver.accepts(*node))
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element sibling receiver required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let sibling = self.callbacks.with_read(&owner, &events, |read, _| {
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                dom.raw().read_element_sibling(
                    node.node(),
                    direction,
                    events.limits.work,
                    || events.budget.charge_work(read.account_scope, 1),
                    read.cancel,
                )
            })?;
            Ok(Completion::Return(self.project_observation(
                scope,
                &owner,
                &events,
                Observation::Node(sibling),
            )?))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn descendant_collection<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        property: DescendantCollection,
    ) -> Result<Completion<'scope>, Error> {
        let wrapper = arguments.receiver();
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, wrapper)? else {
            return Ok(Completion::Throw(scope.type_error(property.required)?));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let qualified = self
                .callbacks
                .with_read(&owner, &events, |read, attached| {
                    let node = read.node(id)?;
                    read.step()?;
                    if html_element_is(node, property.interface) {
                        Ok(Some(collection_origin(node, attached)?))
                    } else {
                        Ok(None)
                    }
                })?;
            let Some(origin) = qualified else {
                return Ok(Completion::Throw(scope.type_error(property.required)?));
            };
            if let Some(cached) = scope
                .native_object_reference::<GuestNodeProjection>(wrapper, property.cache_reference)?
            {
                return Ok(Completion::Return(cached));
            }

            let limit = HTML_NAMESPACE.len() + property.local_name.len();
            let (namespace, name, backing) = character_data::copy_native_name_pair(
                &owner,
                &events,
                Some(HTML_NAMESPACE),
                property.local_name,
            )?;
            let collection = self.callbacks.with_read(&owner, &events, |read, _| {
                let dom = read
                    .dom
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                Ok(dom.raw().get_elements_by_tag_name_ns(
                    DomDescendantQueryRoot::Element(id),
                    namespace,
                    name,
                    limit,
                    read.cancel,
                )?)
            })?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let value = public.with_collection_projection(&events, origin, |projection| {
                projection.project_for_root(
                    scope,
                    (&owner, &events, collection),
                    wrapper,
                    Some(property.cache_reference),
                    Some(backing),
                )
            })?;
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn descendant_elements<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        target: QueryTarget,
        query: Query,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = arguments.receiver();
        let Some((node, root)) = tree::projected_node(scope, receiver)?
            .and_then(|node| target.root(node).map(|root| (node, root)))
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Collection query receiver required"))?,
            ));
        };
        let (namespace_argument, name_argument) = query.arguments();
        let name_argument = name_argument();
        if arguments.get(name_argument.index()).is_none() {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Missing collection query argument"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let (namespace, name) = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    let namespace = namespace_argument
                        .map(|declaration| {
                            let declaration = declaration();
                            conversion.argument(&declaration, arguments.get(declaration.index()))
                        })
                        .transpose()?;
                    let name = conversion
                        .argument(&name_argument, arguments.get(name_argument.index()))?;
                    Ok((namespace, name))
                })
                .map_err(Error::from)?;
            scope.check_active()?;
            let ArgumentValue::Present(name) = name.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            let namespace = match namespace.as_ref().map(|namespace| namespace.get()) {
                Some(ArgumentValue::Present(Some(namespace)))
                    if !namespace.as_code_units().is_empty() =>
                {
                    Some(namespace.as_code_units())
                }
                Some(ArgumentValue::Missing) => return Err(Error::BackendUnavailable.into()),
                _ => None,
            };
            let units = name.as_code_units();
            let limit = units
                .len()
                .checked_add(namespace.map_or(0, <[u16]>::len))
                .ok_or(Error::Allocation)?;
            let (namespace, name, backing) =
                character_data::copy_native_name_pair(&owner, &events, namespace, units)?;
            let (collection, origin) =
                self.callbacks
                    .with_read(&owner, &events, |read, attached| {
                        let origin = collection_origin(read.node(node.node())?, attached)?;
                        let dom = read
                            .dom
                            .try_borrow()
                            .map_err(|_| DocumentScriptNativeError::Busy)?;
                        let native = dom.raw();
                        let collection = match query {
                            Query::Qualified(_) => {
                                native.get_elements_by_tag_name(root, name, limit, read.cancel)
                            }
                            Query::Expanded { .. } => native.get_elements_by_tag_name_ns(
                                root,
                                namespace,
                                name,
                                limit,
                                read.cancel,
                            ),
                            Query::Classes(_) => {
                                native.get_elements_by_class_name(root, name, limit, read.cancel)
                            }
                        }?;
                        Ok((collection, origin))
                    })?;
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let value = public.with_collection_projection(&events, origin, |projection| {
                projection.project_for_root(
                    scope,
                    (&owner, &events, collection),
                    receiver,
                    None,
                    Some(backing),
                )
            })?;
            Ok(Completion::Return(value))
        })();
        tree::tree_completion(scope, &events, result)
    }
}

fn collection_origin(
    node: DomNode<'_>,
    attached: DomHostedGlobalId,
) -> Result<Option<DomHostedGlobalId>, DocumentGuestEventError> {
    Ok(node.relevant_global()?.filter(|origin| *origin != attached))
}
