//! Document and Node bindings for the real attached Window.

use super::*;
use surgeist_bindings_js::conversion::declarations::node_get_root_node_0_options;
use surgeist_dom::{DomBatch, DomNodeKind, DomNodeRetirementInbox, DomTagName, NodeId};
use surgeist_html::values::DocumentString;

const NODE: &[u16] = &ascii(b"Node");
const DOCUMENT: &[u16] = &ascii(b"Document");
mod character_data;
pub(super) mod element;
mod fragment;
mod ordinary;
pub(in crate::document_scripting::guest) use ordinary::NodeLifetime;

#[derive(Clone, Copy)]
pub(in crate::document_scripting::guest) enum GuestNode {
    Document(DocumentId),
    Fragment(NodeId),
    Text(NodeId),
    Comment(NodeId),
    Element(NodeId),
}

impl GuestNode {
    pub(in crate::document_scripting::guest) fn node(self) -> NodeId {
        match self {
            Self::Document(id) => id.node(),
            Self::Fragment(id) | Self::Text(id) | Self::Comment(id) | Self::Element(id) => id,
        }
    }

    fn node_type(self, kind: DomNodeKind<'_>) -> Result<f64, DocumentScriptNativeError> {
        match (self, kind) {
            (Self::Document(_), DomNodeKind::Document(_)) => Ok(9.0),
            (Self::Fragment(_), DomNodeKind::DocumentFragment) => Ok(11.0),
            (Self::Text(_), DomNodeKind::Text(_)) => Ok(3.0),
            (Self::Comment(_), DomNodeKind::Comment(_)) => Ok(8.0),
            (Self::Element(_), DomNodeKind::Element(_)) => Ok(1.0),
            _ => Err(DocumentScriptNativeError::ProjectionCustody),
        }
    }
}

pub(in crate::document_scripting::guest) struct GuestNodeProjection {
    node: Cell<Option<GuestNode>>,
    lifetime: Option<Rc<NodeLifetime>>,
}

impl GuestNodeProjection {
    pub(in crate::document_scripting::guest) fn id(&self) -> Option<GuestNode> {
        self.node.get()
    }

    pub(in crate::document_scripting::guest) fn lifetime(&self) -> Option<Rc<NodeLifetime>> {
        self.lifetime.clone()
    }
}

struct DocumentPin {
    document: Option<DocumentId>,
    holder: RetainedCallbackObject,
    next: Option<Box<DocumentPin>>,
    _backing: GuestBacking,
}

pub(super) struct DocumentBindings {
    attached_id: DocumentId,
    attached: RetainedCallbackObject,
    prototype: RetainedCallbackObject,
    documents: RefCell<Option<Box<DocumentPin>>>,
    fragment: fragment::FragmentBindings,
    character_data: character_data::CharacterBindings,
    element: element::ElementBindings,
    ordinary: ordinary::OrdinaryBindings,
}

impl Drop for DocumentBindings {
    fn drop(&mut self) {
        let head = self.documents.get_mut();
        while let Some(pin) = head.take() {
            let mut pin = *pin;
            *head = pin.next.take();
        }
    }
}

#[derive(Clone, Copy)]
enum NodeMember {
    Type,
    Name,
    OwnerDocument,
    Parent,
    Connected,
}

enum Observation {
    Number(f64),
    String(DocumentString),
    TagName(DomTagName),
    Boolean(bool),
    Node(Option<NodeId>),
    Document(Option<DocumentId>),
}

impl DocumentBindings {
    pub(super) fn install<'scope>(
        scope: &mut Scope<'scope>,
        document: DocumentId,
        target_prototype: &Value<'scope>,
        inbox: DomNodeRetirementInbox,
        capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
        fund: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
        origin: OriginInterfaceFunding<'_>,
    ) -> Result<Self, DocumentGuestEventError> {
        let (node_constructor, node) = install_node(scope, target_prototype, &origin.capture)?;
        let prototype = scope.object(ObjectPrototype::Object(&node))?;
        let fallback = capture(size_of::<Capture>())?;
        let construct = capture(size_of::<Capture>())?;
        let callback = ConstructorCallback::bounded(
            0,
            move |scope| {
                let (_, events) = fallback.active(scope)?;
                let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
                scope.open_callback_object(&public.document.prototype)
            },
            move |scope, arguments| construct.construct_document(scope, arguments),
        );
        let document_constructor = scope.constructor(
            &callback,
            FunctionMetadata {
                name: DOCUMENT,
                length: 0,
            },
        )?;
        scope.set_prototype(
            &document_constructor,
            ObjectPrototype::Object(&node_constructor),
        )?;

        scope.define_property(
            &document_constructor,
            PropertyKey::String(PROTOTYPE),
            PropertyDescriptor::Data {
                value: &prototype,
                writable: false,
                enumerable: false,
                configurable: false,
            },
        )?;
        fragment::install_factory(scope, &prototype, capture)?;
        let global = scope.global()?;
        install_interface(scope, &global, DOCUMENT, &document_constructor, &prototype)?;
        let fragment =
            fragment::FragmentBindings::install(scope, &node, &node_constructor, capture, fund)?;
        let character_data = character_data::CharacterBindings::install(
            scope,
            &node,
            &node_constructor,
            &prototype,
            capture,
            fund,
        )?;
        let element = element::ElementBindings::install(
            scope,
            &node,
            &node_constructor,
            &origin.capture,
            &origin.retain,
        )?;
        element::install_document_root(scope, &prototype, capture)?;
        let getter = capture(size_of::<Capture>())?;
        install_getter(
            scope,
            &global,
            &ascii(b"document"),
            &ascii(b"get document"),
            false,
            Callback::new(move |scope, arguments| getter.window_document(scope, arguments)),
        )?;
        let attached = scope.project_retained_native_object(
            ObjectPrototype::Object(&prototype),
            GuestNodeProjection {
                node: Cell::new(Some(GuestNode::Document(document))),
                lifetime: None,
            },
            fund,
        )?;
        let observed = scope.callback_object(&prototype)?;
        let prototype = scope.retain_callback_object(observed, fund)?;
        Ok(Self {
            attached_id: document,
            attached,
            prototype,
            documents: RefCell::new(None),
            fragment,
            character_data,
            element,
            ordinary: ordinary::OrdinaryBindings::new(inbox),
        })
    }

    fn find<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        node: surgeist_dom::NodeId,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        if let Some(value) = self.open_document(scope, events, node)? {
            return Ok(Some(value));
        }
        self.ordinary.open(scope, events, node)
    }

    fn open_document<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        node: NodeId,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        events.step()?;
        if node == self.attached_id.node() {
            return Ok(Some(scope.open_callback_object(&self.attached)?));
        }
        let pins = self
            .documents
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut next = pins.as_deref();
        while let Some(pin) = next {
            events.step()?;
            if pin.document.is_some_and(|document| document.node() == node) {
                return Ok(Some(scope.open_callback_object(&pin.holder)?));
            }
            next = pin.next.as_deref();
        }
        Ok(None)
    }

    fn publish_document<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        prototype: &Value<'scope>,
        resolve: impl FnOnce() -> Result<DocumentId, DocumentGuestEventError>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let mut pins = self
            .documents
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let backing = events.reserve(owner, size_of::<DocumentPin>())?;
        let holder = events.project(
            owner,
            scope,
            ObjectPrototype::Object(prototype),
            GuestNodeProjection {
                node: Cell::new(None),
                lifetime: None,
            },
        )?;
        let value = scope.open_callback_object(&holder)?;
        let projection = scope
            .native_object::<GuestNodeProjection>(&value)?
            .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
        let mut pin = Box::new(DocumentPin {
            document: None,
            holder,
            next: None,
            _backing: backing,
        });
        let document = match resolve() {
            Ok(document) => document,
            Err(error) => {
                let first = refuse(scope, &events.refusal, error);
                let pin = *pin;
                let _ = scope.release_callback_object(pin.holder);
                return first.map_err(Into::into);
            }
        };
        projection
            .payload()
            .node
            .set(Some(GuestNode::Document(document)));
        pin.document = Some(document);
        pin.next = pins.take();
        *pins = Some(pin);
        Ok(value)
    }

    pub(super) fn reap_nodes(&self, process: &mut Process, retired: &Cell<bool>) {
        self.ordinary.reap(process, retired);
    }
}

pub(super) fn install_node<'scope>(
    scope: &mut Scope<'scope>,
    target_prototype: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(Value<'scope>, Value<'scope>), DocumentGuestEventError> {
    let node = scope.object(ObjectPrototype::Object(target_prototype))?;
    let node_constructor = scope.constructor(
        &ConstructorCallback::bounded(
            0,
            // Node rejects construction before selecting any prototype.
            |_| Err(Error::BackendUnavailable),
            |scope, _| {
                Ok(ConstructorCompletion::Throw(
                    scope.type_error(&ascii(b"Illegal constructor"))?,
                ))
            },
        ),
        FunctionMetadata {
            name: NODE,
            length: 0,
        },
    )?;
    let target_constructor = scope.get(target_prototype, PropertyKey::String(CONSTRUCTOR))?;
    scope.set_prototype(
        &node_constructor,
        ObjectPrototype::Object(&target_constructor),
    )?;
    scope.define_property(
        &node_constructor,
        PropertyKey::String(PROTOTYPE),
        PropertyDescriptor::Data {
            value: &node,
            writable: false,
            enumerable: false,
            configurable: false,
        },
    )?;
    for (member, key, name) in [
        (
            NodeMember::Type,
            &ascii(b"nodeType")[..],
            &ascii(b"get nodeType")[..],
        ),
        (
            NodeMember::Name,
            &ascii(b"nodeName")[..],
            &ascii(b"get nodeName")[..],
        ),
        (
            NodeMember::Connected,
            &ascii(b"isConnected")[..],
            &ascii(b"get isConnected")[..],
        ),
        (
            NodeMember::OwnerDocument,
            &ascii(b"ownerDocument")[..],
            &ascii(b"get ownerDocument")[..],
        ),
        (
            NodeMember::Parent,
            &ascii(b"parentNode")[..],
            &ascii(b"get parentNode")[..],
        ),
    ] {
        let handler = capture(size_of::<(Capture, NodeMember)>())?;
        install_getter(
            scope,
            &node,
            key,
            name,
            true,
            Callback::new(move |scope, arguments| handler.node_member(scope, arguments, member)),
        )?;
    }
    let root = capture(size_of::<Capture>())?;
    let root_name = &ascii(b"getRootNode");
    let method = scope.bounded_function(
        &Callback::new(move |scope, arguments| root.root_node(scope, arguments)),
        FunctionMetadata {
            name: root_name,
            length: 0,
        },
        1,
    )?;
    define_data(
        scope,
        &node,
        PropertyKey::String(root_name),
        &method,
        true,
        true,
        true,
    )?;
    for (key, code) in [
        (&ascii(b"ELEMENT_NODE")[..], 1.0),
        (&ascii(b"ATTRIBUTE_NODE")[..], 2.0),
        (&ascii(b"TEXT_NODE")[..], 3.0),
        (&ascii(b"CDATA_SECTION_NODE")[..], 4.0),
        (&ascii(b"ENTITY_REFERENCE_NODE")[..], 5.0),
        (&ascii(b"ENTITY_NODE")[..], 6.0),
        (&ascii(b"PROCESSING_INSTRUCTION_NODE")[..], 7.0),
        (&ascii(b"COMMENT_NODE")[..], 8.0),
        (&ascii(b"DOCUMENT_NODE")[..], 9.0),
        (&ascii(b"DOCUMENT_TYPE_NODE")[..], 10.0),
        (&ascii(b"DOCUMENT_FRAGMENT_NODE")[..], 11.0),
        (&ascii(b"NOTATION_NODE")[..], 12.0),
    ] {
        let value = scope.primitive(Primitive::Number(code))?;
        for recipient in [&node_constructor, &node] {
            define_data(
                scope,
                recipient,
                PropertyKey::String(key),
                &value,
                false,
                true,
                false,
            )?;
        }
    }
    let global = scope.global()?;
    install_interface(scope, &global, NODE, &node_constructor, &node)?;
    Ok((node_constructor, node))
}

pub(super) fn install_interface<'scope>(
    scope: &mut Scope<'scope>,
    global: &Value<'scope>,
    name: &[u16],
    interface: &Value<'scope>,
    prototype: &Value<'scope>,
) -> Result<(), Error> {
    define_data(
        scope,
        prototype,
        PropertyKey::String(CONSTRUCTOR),
        interface,
        true,
        false,
        true,
    )?;
    let tag = scope.well_known_symbol(WellKnownSymbol::ToStringTag)?;
    let value = scope.string(name)?;
    define_data(
        scope,
        prototype,
        PropertyKey::Value(&tag),
        &value,
        false,
        false,
        true,
    )?;
    define_data(
        scope,
        global,
        PropertyKey::String(name),
        interface,
        true,
        false,
        true,
    )
}

fn install_getter<'scope>(
    scope: &mut Scope<'scope>,
    object: &Value<'scope>,
    key: &[u16],
    name: &[u16],
    configurable: bool,
    callback: Callback,
) -> Result<(), Error> {
    let getter = scope.fixed_arity_function(&callback, FunctionMetadata { name, length: 0 })?;
    scope.define_new_property(
        object,
        PropertyKey::String(key),
        PropertyDescriptor::Accessor {
            get: Some(&getter),
            set: None,
            enumerable: true,
            configurable,
        },
    )
}

impl PublicEventBindings {
    fn project_document<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        document: DocumentId,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let bindings = &self.document;
        if let Some(value) = bindings.open_document(scope, events, document.node())? {
            return Ok(value);
        }
        let prototype = scope.open_callback_object(&bindings.prototype)?;
        // Published Document nodes survive for the native owner lifetime.
        // Their guest pins belong to this attachment, like new Document().
        bindings.publish_document(scope, owner, events, &prototype, || Ok(document))
    }
}

impl Capture {
    fn window_document<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let global = scope.global()?;
        if !scope.strict_equals(arguments.receiver(), &global)? {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Window required"))?,
            ));
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let bindings = &events
                .public
                .as_ref()
                .ok_or(Error::ExpiredIdentity)?
                .document;
            self.with_native(&owner, |session, _| {
                session
                    .snapshot()
                    .document(bindings.attached_id)
                    .map_err(DocumentScriptNativeError::from)?;
                Ok(())
            })?;
            bindings
                .open_document(scope, &events, bindings.attached_id.node())?
                .ok_or(DocumentScriptNativeError::ProjectionCustody.into())
        })();
        complete(scope, &events, result)
    }

    fn node_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: NodeMember,
    ) -> Result<Completion<'scope>, Error> {
        let Some(projection) = scope.native_object::<GuestNodeProjection>(arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let id = projection
                .payload()
                .id()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            events.step()?;
            let observation = self.with_native(&owner, |session, _| {
                let snapshot = session.snapshot();
                let node = snapshot.node_with(id.node(), || events.step())?;
                let node_type = id.node_type(node.kind())?;
                Ok(match member {
                    NodeMember::Type => Observation::Number(node_type),
                    NodeMember::Name => match node.tag_name_spelling_with(|| events.step())? {
                        Some(name) => Observation::TagName(name),
                        None => Observation::String(node.node_name()),
                    },
                    NodeMember::OwnerDocument => Observation::Document(node.owner_document()),
                    NodeMember::Parent => Observation::Node(node.parent()),
                    NodeMember::Connected => Observation::Boolean(
                        node.root_with(true, || events.step())?
                            .as_document()
                            .is_some(),
                    ),
                })
            })?;
            match observation {
                Observation::Number(value) => Ok(scope.primitive(Primitive::Number(value))?),
                Observation::Boolean(value) => Ok(scope.primitive(Primitive::Boolean(value))?),
                Observation::String(value) => Ok(self
                    .callbacks
                    .conversion
                    .run(scope, |conversion| {
                        conversion.copy_to_js(value.code_units())
                    })
                    .map_err(Error::from)?),
                Observation::TagName(name) => self.copy_tag_name(scope, &owner, &events, &name),
                Observation::Node(None) | Observation::Document(None) => {
                    Ok(scope.primitive(Primitive::Null)?)
                }
                Observation::Node(Some(node)) => events
                    .public
                    .as_ref()
                    .ok_or(Error::ExpiredIdentity)?
                    .project_node(scope, &owner, &events, node),
                Observation::Document(Some(document)) => events
                    .public
                    .as_ref()
                    .ok_or(Error::ExpiredIdentity)?
                    .project_document(scope, &owner, &events, document),
            }
        })();
        complete(scope, &events, result)
    }

    fn root_node<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(projection) = scope.native_object::<GuestNodeProjection>(arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Node required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let id = projection
                .payload()
                .id()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            events.step()?;
            self.with_native(&owner, |session, _| {
                session.snapshot().node_with(id.node(), || events.step())?;
                Ok(())
            })?;
            let options = node_get_root_node_0_options();
            let options = self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.argument(&options, arguments.get(options.index()))
                })
                .map_err(Error::from)?;
            let ArgumentValue::Present(options) = options.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            scope.check_active()?;
            events.step()?;
            let root = self.with_native(&owner, |session, _| {
                let snapshot = session.snapshot();
                let node = snapshot.node_with(id.node(), || events.step())?;
                Ok(node.root_with(options.composed, || events.step())?.id())
            })?;
            if root == id.node() {
                let receiver = scope.retain(arguments.receiver());
                return Ok(scope.import(&receiver)?);
            }
            events
                .public
                .as_ref()
                .ok_or(Error::ExpiredIdentity)?
                .project_node(scope, &owner, &events, root)
        })();
        complete(scope, &events, result)
    }

    fn construct_document<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let selected = arguments.select_prototype(scope)?;
            scope.check_active()?;
            events.step()?;
            let bindings = &events
                .public
                .as_ref()
                .ok_or(Error::ExpiredIdentity)?
                .document;
            let attached = owner
                .attachment
                .get()
                .ok_or(DocumentScriptNativeError::Revoked)?
                .target;
            bindings.publish_document(scope, &owner, &events, selected.prototype(), || {
                self.with_native(&owner, |session, global| {
                    let hosted = session.hosted_execution();
                    let mut dom = session.dom();
                    let context = hosted
                        .with_current_top_level(
                            &mut dom,
                            attached,
                            &owner.cancellation,
                            |source, dom| {
                                source
                                    .document_constructor_context(
                                        dom.raw(),
                                        global,
                                        &owner.cancellation,
                                    )
                                    .map_err(DomHostedRuntimeError::Dom)
                            },
                        )
                        .map_err(DocumentScriptNativeError::from)?;
                    let mut batch = DomBatch::new();
                    let ticket = batch.construct_document(context);
                    let commit = dom
                        .apply(batch, &owner.cancellation)
                        .map_err(DocumentScriptNativeError::from)?;
                    commit
                        .document(&ticket)
                        .map_err(|error| DocumentScriptNativeError::from(error).into())
                })
            })
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

fn complete<'scope>(
    scope: &mut Scope<'scope>,
    events: &GuestEvents,
    result: Result<Value<'scope>, DocumentGuestEventError>,
) -> Result<Completion<'scope>, Error> {
    match result {
        Ok(value) => Ok(Completion::Return(value)),
        Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => Err(error),
        Err(error) => refuse(scope, &events.refusal, error),
    }
}
