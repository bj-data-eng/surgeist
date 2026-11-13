//! Guest identity for the canonical custom element registry.

use super::*;
use surgeist_bindings_js::CustomElementRegistryIdentity;
use surgeist_dom::{DomNodeKind, DomRegistryBinding, DomRegistryId, DomSnapshot};

const INTERFACE: &[u16] = &ascii(b"CustomElementRegistry");

pub(super) struct GlobalRegistry {
    id: DomRegistryId,
    object: RetainedCallbackObject,
    identity: CustomElementRegistryIdentity,
}

pub(in crate::document_scripting) fn document_registry(
    snapshot: &DomSnapshot,
    document: DocumentId,
) -> Result<DomRegistryId, DocumentScriptNativeError> {
    let DomRegistryBinding::Registry(id) = snapshot
        .document(document)?
        .metadata()
        .custom_element_registry()
    else {
        return Err(DocumentScriptNativeError::ProjectionCustody);
    };
    snapshot.registry(id)?;
    Ok(id)
}

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    window: &Value<'scope>,
    document: DocumentId,
    id: DomRegistryId,
    capture: CaptureFunding<'_>,
    fund: &dyn Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
) -> Result<GlobalRegistry, DocumentGuestEventError> {
    let prototype = scope.object(ObjectPrototype::Default)?;
    let constructor = scope.constructor(
        &document::illegal_constructor(),
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
    document::install_interface(scope, window, INTERFACE, &constructor, &prototype)?;
    let (object, identity) = scope.project_custom_element_registry(
        ObjectPrototype::Object(&prototype),
        |payload, holder| Ok::<_, DocumentGuestEventError>((fund(payload)?, fund(holder)?)),
    )?;
    let handler = capture(size_of::<(Capture, DocumentId)>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.window_registry(scope, arguments, document)),
        FunctionMetadata {
            name: &ascii(b"get customElements"),
            length: 0,
        },
    )?;
    define_accessor(
        scope,
        window,
        &ascii(b"customElements"),
        &getter,
        None,
        true,
    )?;
    Ok(GlobalRegistry {
        id,
        object,
        identity,
    })
}

#[derive(Clone, Copy)]
pub(super) enum Receiver {
    DocumentOrShadowRoot,
    Element,
}

pub(super) fn install_node_getter<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    receiver: Receiver,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<(Capture, Receiver)>())?;
    let getter = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| handler.node_registry(scope, arguments, receiver)),
        FunctionMetadata {
            name: &ascii(b"get customElementRegistry"),
            length: 0,
        },
    )?;
    define_accessor(
        scope,
        prototype,
        &ascii(b"customElementRegistry"),
        &getter,
        None,
        true,
    )?;
    Ok(())
}

impl Capture {
    fn window_registry<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        document: DocumentId,
    ) -> Result<Completion<'scope>, Error> {
        let global = scope.global()?;
        if !scope.strict_equals(arguments.receiver(), &global)? {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Window required"))?,
            ));
        }
        self.registry_value(scope, document.node())
    }

    fn node_registry<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        receiver: Receiver,
    ) -> Result<Completion<'scope>, Error> {
        use document::GuestNode;
        let native = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let node = native.and_then(|native| native.payload().id());
        match (receiver, node) {
            (
                Receiver::DocumentOrShadowRoot,
                Some(node @ (GuestNode::Document(_) | GuestNode::ShadowRoot(_))),
            )
            | (Receiver::Element, Some(node @ GuestNode::Element(_))) => {
                self.registry_value(scope, node.node())
            }
            _ => Ok(Completion::Throw(
                scope.type_error(&ascii(b"Invalid registry receiver"))?,
            )),
        }
    }

    fn registry_value<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        node: surgeist_dom::NodeId,
    ) -> Result<Completion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = self.callbacks.with_read(&owner, &events, |read, _| {
            let node = read.node(node)?;
            read.step()?;
            let binding = match node.kind() {
                DomNodeKind::Document(metadata) => metadata.custom_element_registry(),
                DomNodeKind::Element(element) => element.custom_element_registry(),
                DomNodeKind::ShadowRoot(root) => root.custom_element_registry(),
                _ => return Err(DomError::WrongNodeKind.into()),
            };
            let DomRegistryBinding::Registry(id) = binding else {
                return Ok(scope.primitive(Primitive::Null)?);
            };
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            public.open_registry(scope, read, id)
        });
        document::complete(scope, &events, result)
    }
}

impl PublicEventBindings {
    pub(super) fn native_registry(
        &self,
        read: &NativeRead<'_>,
        identity: CustomElementRegistryIdentity,
    ) -> Result<DomRegistryId, DocumentGuestEventError> {
        self.with_registry(
            read,
            |registry| registry.identity == identity,
            |registry| Ok(registry.id),
        )
    }

    fn open_registry<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        read: &NativeRead<'_>,
        id: DomRegistryId,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        self.with_registry(
            read,
            |registry| registry.id == id,
            |registry| Ok(scope.open_callback_object(&registry.object)?),
        )
    }

    fn with_registry<T>(
        &self,
        read: &NativeRead<'_>,
        mut matches: impl FnMut(&GlobalRegistry) -> bool,
        use_registry: impl FnOnce(&GlobalRegistry) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        read.step()?;
        if matches(&self.custom_elements) {
            return use_registry(&self.custom_elements);
        }
        let origins = self
            .origins
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut next = origins.as_deref();
        while let Some(origin) = next {
            read.step()?;
            if matches(&origin.custom_elements) {
                return use_registry(&origin.custom_elements);
            }
            next = origin.next.as_deref();
        }
        Err(DocumentScriptNativeError::CreationGlobalUnavailable.into())
    }
}
