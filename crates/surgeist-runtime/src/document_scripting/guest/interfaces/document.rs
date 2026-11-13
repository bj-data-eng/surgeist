//! Document and Node bindings for the real attached Window.

use super::*;
use crate::DomSourceDriveProgress;
use surgeist_bindings_js::conversion::declarations::node_get_root_node_0_options;
use surgeist_dom::{
    DomBatch, DomDocumentInterface, DomNodeKind, DomNodeRetirementInbox, DomQualifiedName,
    DomSiblingDirection, DomTagName, NodeId,
};
use surgeist_html::values::DocumentString;

const NODE: &[u16] = &ascii(b"Node");
const DOCUMENT: &[u16] = &ascii(b"Document");
mod attr;
mod attr_factory;
mod character_data;
mod document_type;
mod dom_implementation;
pub(super) use character_data::copy_native_data;
pub(super) mod element;
mod element_factory;
mod fragment;
mod ordinary;
mod parent_node;
mod processing_instruction;
mod shadow;
mod style_policy;
mod template;
mod text;
mod title;
pub(super) mod tree;
pub(in crate::document_scripting::guest) use ordinary::NodeLifetime;
pub(in crate::document_scripting) use ordinary::PendingConstructor;

fn creation_realm_fallback(
    owner: &NativeOwner,
) -> Result<surgeist_dom::DomCreationRealmFallback, DocumentScriptNativeError> {
    owner
        .attachment
        .get()
        .and_then(|attachment| attachment.registration.upgrade())
        .map(|registration| registration.creation_realm_fallback())
        .ok_or(DocumentScriptNativeError::Revoked)
}

fn guest_source_services(
    style_policy: &mut dyn FnMut(
        &surgeist_dom::DomStyleSource,
    ) -> Option<surgeist_dom::DomStyleInlineCspObservation>,
) -> crate::DomSourceServices<'_> {
    crate::DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy,
        url: None,
    }
}

/// The guest receives the original DOM result. A completed composition's
/// producer receipts have already published and carry no further authority.
fn guest_source_terminal(
    result: crate::DomSourceDriveProgress,
) -> Option<Result<surgeist_dom::DomSourceCompletion, surgeist_dom::DomSourceFailure>> {
    use crate::DomSourceDriveProgress as Drive;
    use surgeist_dom::DomSourceProgress as Source;

    let source = match result {
        Drive::Complete(done) => return Some(Ok(done)),
        Drive::Failed(failure) => return Some(Err(failure)),
        Drive::InlineSource { source, .. } => source,
        Drive::MetaSource(outcome) => outcome.into_parts().0,
        Drive::InlineFullscreenHandoff { .. } | Drive::Pending { .. } => return None,
    };
    match source {
        Source::Complete(done) => Some(Ok(done)),
        Source::Failed(failure) => Some(Err(failure)),
        Source::Request(_) => None,
    }
}

impl DocumentScriptHost {
    fn deliver_resumed_source_reactions(
        &mut self,
        permit: &DocumentScriptPermit,
        terminal: GuestTerminalSource<'_>,
        shadow_reactions: Option<surgeist_dom::DomSynchronousReactionQueue>,
        original_guest_exception: Option<&Error>,
        reaction_failure: Option<&ReactionFailureSlot>,
    ) -> Result<(), DocumentGuestEventError> {
        if self.attachment()?.realm.check_owner_active().is_err()
            || self
                .native
                .session
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?
                .read_dom()
                .pending_source_edit()
                .is_some()
        {
            return Ok(());
        }
        let owner = self.native.clone();
        let Some(events) = self
            .engine
            .as_ref()
            .and_then(|engine| engine.guest_events.clone())
        else {
            return Ok(());
        };
        let Some(public) = &events.public else {
            return Ok(());
        };
        let registration = owner
            .attachment
            .get()
            .and_then(|attachment| attachment.registration.upgrade())
            .ok_or(DocumentScriptNativeError::Revoked)?;
        if registration.custom_element_reaction_segments() == 0 {
            return Ok(());
        }
        self.in_document(permit, |scope| {
            let drain = |scope: &mut Scope<'_>| {
                let capture = Capture {
                    callbacks: public.callbacks.clone(),
                    origin: None,
                    _backing: events.reserve(&owner, size_of::<Capture>())?,
                };
                capture.drain_source_and_routed_reactions(
                    scope,
                    &owner,
                    &events,
                    terminal.operation,
                    shadow_reactions,
                )
            };
            if let Some(original) = original_guest_exception {
                if shadow_reactions.is_some() {
                    let prior = reaction_failure.ok_or(Error::ExpiredIdentity)?;
                    return scope.complete_after_host_failure(
                        prior,
                        original.clone().into(),
                        drain,
                    );
                }
                return Ok(());
            }
            if let Some(cause) = terminal.error {
                if let Some(prior) = reaction_failure {
                    return scope.complete_after_host_failure(prior, cause.clone().into(), drain);
                }
                let prior = match prepare_reaction_failure(scope, &owner, &events) {
                    Ok(prior) => prior,
                    Err(_) => return refuse(scope, &events.refusal, cause.clone().into()),
                };
                return scope.complete_after_host_failure(&prior, cause.clone().into(), drain);
            }
            match drain(scope) {
                Ok(()) => Ok(()),
                Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                    Err(error)
                }
                Err(error) => refuse(scope, &events.refusal, error),
            }
        })?;
        Ok(())
    }
}

struct GuestTerminalSource<'a> {
    operation: surgeist_dom::DomSourceOperationId,
    error: Option<&'a surgeist_dom::DomError>,
}

fn guest_terminal_result(result: &DomSourceDriveProgress) -> Option<GuestTerminalSource<'_>> {
    use surgeist_dom::DomSourceProgress as Source;
    let complete = |done: &surgeist_dom::DomSourceCompletion| GuestTerminalSource {
        operation: done.operation(),
        error: done
            .cancellation_requested()
            .then_some(&surgeist_dom::DomError::Cancelled),
    };
    fn failed(failure: &surgeist_dom::DomSourceFailure) -> GuestTerminalSource<'_> {
        GuestTerminalSource {
            operation: failure.operation(),
            error: Some(failure.error()),
        }
    }
    let source = match result {
        DomSourceDriveProgress::Complete(done) => return Some(complete(done)),
        DomSourceDriveProgress::Failed(failure) => return Some(failed(failure)),
        DomSourceDriveProgress::InlineSource { source, .. } => source,
        DomSourceDriveProgress::MetaSource(outcome) => outcome.source(),
        DomSourceDriveProgress::InlineFullscreenHandoff { .. }
        | DomSourceDriveProgress::Pending { .. } => return None,
    };
    match source {
        Source::Complete(done) => Some(complete(done)),
        Source::Failed(failure) => Some(failed(failure)),
        Source::Request(_) => None,
    }
}

#[derive(Clone, Copy)]
pub(in crate::document_scripting::guest) enum GuestNode {
    Document(DocumentId),
    DocumentType(NodeId),
    Fragment(NodeId),
    ShadowRoot(NodeId),
    Text(NodeId),
    Comment(NodeId),
    ProcessingInstruction(NodeId),
    Element(NodeId),
    Attr(NodeId),
}

impl GuestNode {
    pub(in crate::document_scripting::guest) fn node(self) -> NodeId {
        match self {
            Self::Document(id) => id.node(),
            Self::DocumentType(id)
            | Self::Fragment(id)
            | Self::ShadowRoot(id)
            | Self::Text(id)
            | Self::Comment(id)
            | Self::ProcessingInstruction(id)
            | Self::Element(id)
            | Self::Attr(id) => id,
        }
    }

    fn node_type(self, kind: DomNodeKind<'_>) -> Result<f64, DocumentScriptNativeError> {
        match (self, kind) {
            (Self::Document(_), DomNodeKind::Document(_)) => Ok(9.0),
            (Self::DocumentType(_), DomNodeKind::DocumentType(_)) => Ok(10.0),
            (Self::Fragment(_), DomNodeKind::DocumentFragment) => Ok(11.0),
            (Self::ShadowRoot(_), DomNodeKind::ShadowRoot(_)) => Ok(11.0),
            (Self::Text(_), DomNodeKind::Text(_)) => Ok(3.0),
            (Self::Comment(_), DomNodeKind::Comment(_)) => Ok(8.0),
            (Self::ProcessingInstruction(_), DomNodeKind::ProcessingInstruction(_)) => Ok(7.0),
            (Self::Element(_), DomNodeKind::Element(_)) => Ok(1.0),
            (Self::Attr(_), DomNodeKind::Attr(_)) => Ok(2.0),
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
    document: DocumentId,
    holder: RetainedCallbackObject,
    next: Option<Box<DocumentPin>>,
    _backing: GuestBacking,
}

pub(super) struct DocumentBindings {
    attached_id: DocumentId,
    attached: RetainedCallbackObject,
    documents: RefCell<Option<Box<DocumentPin>>>,
    prototypes: DocumentPrototypes,
    element: element::ElementBindings,
    pub(super) ordinary: ordinary::OrdinaryBindings,
}

pub(super) struct DocumentPrototypes {
    document: RetainedCallbackObject,
    xml_document: RetainedCallbackObject,
    document_type: RetainedCallbackObject,
    implementation: RetainedCallbackObject,
    fragment: RetainedCallbackObject,
    shadow: RetainedCallbackObject,
    attr: RetainedCallbackObject,
    character_data: character_data::CharacterBindings,
}

impl DocumentPrototypes {
    fn for_document(&self, interface: DomDocumentInterface) -> &RetainedCallbackObject {
        match interface {
            DomDocumentInterface::Document => &self.document,
            DomDocumentInterface::XmlDocument => &self.xml_document,
        }
    }

    pub(super) fn install_origin<'scope>(
        scope: &mut Scope<'scope>,
        node: &Value<'scope>,
        constructor: &Value<'scope>,
        funding: &OriginInterfaceFunding<'_>,
    ) -> Result<Self, DocumentGuestEventError> {
        let unavailable = illegal_constructor();
        let (prototypes, _) = Self::install(
            scope,
            node,
            constructor,
            (&unavailable, &unavailable),
            funding,
            None,
        )?;
        Ok(prototypes)
    }

    fn install<'scope>(
        scope: &mut Scope<'scope>,
        node: &Value<'scope>,
        constructor: &Value<'scope>,
        (document_callback, fragment_callback): (&ConstructorCallback, &ConstructorCallback),
        funding: &OriginInterfaceFunding<'_>,
        constructors: Option<CaptureFunding<'_>>,
    ) -> Result<(Self, Value<'scope>), DocumentGuestEventError> {
        let (document_constructor, document) = install_derived_interface(
            scope,
            node,
            constructor,
            FunctionMetadata {
                name: DOCUMENT,
                length: 0,
            },
            document_callback,
        )?;
        let (_, xml_document) = install_derived_interface(
            scope,
            &document,
            &document_constructor,
            FunctionMetadata {
                name: &ascii(b"XMLDocument"),
                length: 0,
            },
            &illegal_constructor(),
        )?;
        let (fragment_constructor, fragment) = install_derived_interface(
            scope,
            node,
            constructor,
            FunctionMetadata {
                name: fragment::FRAGMENT,
                length: 0,
            },
            fragment_callback,
        )?;
        let shadow = shadow::install(scope, &fragment_constructor, &fragment, &funding.capture)?;
        custom_elements::install_node_getter(
            scope,
            &document,
            custom_elements::Receiver::DocumentOrShadowRoot,
            funding.capture,
        )?;
        fragment::install_factory(scope, &document, &funding.capture)?;
        attr_factory::install(scope, &document, &funding.capture)?;
        element_factory::install(scope, &document, &funding.capture)?;
        tree::install_adoption(scope, &document, funding.capture)?;
        tree::install_import(scope, &document, funding.capture)?;
        element::install_document_root(scope, &document, &funding.capture)?;
        parent_node::install(
            scope,
            &document,
            parent_node::Receiver::Document,
            funding.capture,
        )?;
        parent_node::install(
            scope,
            &fragment,
            parent_node::Receiver::Fragment,
            funding.capture,
        )?;
        let character_data = character_data::CharacterBindings::install(
            scope,
            node,
            constructor,
            &document,
            funding,
            constructors,
        )?;
        let attr = attr::install(scope, node, constructor, funding)?;
        let document_type = document_type::install(scope, node, constructor, &document, funding)?;
        let implementation = dom_implementation::install(scope, &document, funding)?;
        let observed = scope.callback_object(&document)?;
        let retained_document = scope.retain_callback_object(observed, funding.retain)?;
        let observed = scope.callback_object(&xml_document)?;
        let retained_xml_document = scope.retain_callback_object(observed, funding.retain)?;
        let observed = scope.callback_object(&fragment)?;
        let retained_fragment = scope.retain_callback_object(observed, funding.retain)?;
        let observed = scope.callback_object(&shadow)?;
        let retained_shadow = scope.retain_callback_object(observed, funding.retain)?;
        Ok((
            Self {
                document: retained_document,
                xml_document: retained_xml_document,
                document_type,
                implementation,
                fragment: retained_fragment,
                shadow: retained_shadow,
                attr,
                character_data,
            },
            document,
        ))
    }
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
    ParentElement,
    FirstChild,
    LastChild,
    Sibling(DomSiblingDirection),
    HasChildren,
    Connected,
}

enum Observation {
    Number(f64),
    String(Option<DocumentString>),
    TagName(DomTagName),
    AttrName(DomQualifiedName),
    Boolean(bool),
    Node(Option<NodeId>),
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
        let fallback = capture(size_of::<Capture>())?;
        let construct = capture(size_of::<Capture>())?;
        let callback = ConstructorCallback::bounded(
            0,
            move |scope| {
                let (_, events) = fallback.active(scope)?;
                let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
                scope.open_callback_object(&public.document.prototypes.document)
            },
            move |scope, arguments| construct.construct_document(scope, arguments),
        );
        let fragment = fragment::constructor(capture)?;
        let (prototypes, prototype) = DocumentPrototypes::install(
            scope,
            &node,
            &node_constructor,
            (&callback, &fragment),
            &origin,
            Some(capture),
        )?;
        let global = scope.global()?;
        let element = element::ElementBindings::install(
            scope,
            &node,
            &node_constructor,
            &origin.capture,
            &origin.retain,
        )?;
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
        Ok(Self {
            attached_id: document,
            attached,
            documents: RefCell::new(None),
            prototypes,
            element,
            ordinary: ordinary::OrdinaryBindings::new(inbox),
        })
    }

    fn find<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        admission: ordinary::Admission<'_>,
        node: NodeId,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        if let Some(value) = self.open_document(scope, node, || admission.step())? {
            return Ok(Some(value));
        }
        self.ordinary.open_with(scope, node, || admission.step())
    }

    fn open_document<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        node: NodeId,
        mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        step()?;
        if node == self.attached_id.node() {
            return Ok(Some(scope.open_callback_object(&self.attached)?));
        }
        let pins = self
            .documents
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut next = pins.as_deref();
        while let Some(pin) = next {
            step()?;
            if pin.document.node() == node {
                return Ok(Some(scope.open_callback_object(&pin.holder)?));
            }
            next = pin.next.as_deref();
        }
        Ok(None)
    }

    fn publish_document<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        admission: ordinary::Admission<'_>,
        prototype: ordinary::NodePrototype<'_, 'scope>,
        resolve: impl FnOnce() -> Result<DocumentId, DocumentGuestEventError>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let mut pins = self
            .documents
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let backing = admission.reserve(size_of::<DocumentPin>())?;
        let payload = GuestNodeProjection {
            node: Cell::new(None),
            lifetime: None,
        };
        admission.step()?;
        let allocation = Box::new_uninit();
        let (value, holder, document) = scope.try_project_retained_native_object(
            prototype.projection(),
            payload,
            |payload, holder| {
                Ok::<_, DocumentGuestEventError>((
                    admission.reserve(payload)?,
                    admission.reserve(holder)?,
                ))
            },
            |payload| {
                let document = resolve()?;
                payload.node.set(Some(GuestNode::Document(document)));
                Ok(document)
            },
        )?;
        let next = pins.take();
        *pins = Some(Box::write(
            allocation,
            DocumentPin {
                document,
                holder,
                next,
                _backing: backing,
            },
        ));
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
        (
            NodeMember::ParentElement,
            &ascii(b"parentElement")[..],
            &ascii(b"get parentElement")[..],
        ),
        (
            NodeMember::FirstChild,
            &ascii(b"firstChild")[..],
            &ascii(b"get firstChild")[..],
        ),
        (
            NodeMember::LastChild,
            &ascii(b"lastChild")[..],
            &ascii(b"get lastChild")[..],
        ),
        (
            NodeMember::Sibling(DomSiblingDirection::Previous),
            &ascii(b"previousSibling")[..],
            &ascii(b"get previousSibling")[..],
        ),
        (
            NodeMember::Sibling(DomSiblingDirection::Next),
            &ascii(b"nextSibling")[..],
            &ascii(b"get nextSibling")[..],
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
    let handler = capture(size_of::<Capture>())?;
    let name = &ascii(b"hasChildNodes");
    let method = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| {
            handler.node_member(scope, arguments, NodeMember::HasChildren)
        }),
        FunctionMetadata { name, length: 0 },
    )?;
    define_data(
        scope,
        &node,
        PropertyKey::String(name),
        &method,
        true,
        true,
        true,
    )?;
    text::install(scope, &node, capture)?;
    tree::install(scope, &node, capture)?;
    tree::install_position_constants(scope, &node_constructor, &node)?;
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

pub(super) fn install_derived_interface<'scope>(
    scope: &mut Scope<'scope>,
    parent: &Value<'scope>,
    parent_constructor: &Value<'scope>,
    metadata: FunctionMetadata<'_>,
    callback: &ConstructorCallback,
) -> Result<(Value<'scope>, Value<'scope>), Error> {
    let prototype = scope.object(ObjectPrototype::Object(parent))?;
    let constructor = scope.constructor(callback, metadata)?;
    scope.set_prototype(&constructor, ObjectPrototype::Object(parent_constructor))?;
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
    let global = scope.global()?;
    install_interface(scope, &global, metadata.name, &constructor, &prototype)?;
    Ok((constructor, prototype))
}

pub(super) fn illegal_constructor() -> ConstructorCallback {
    ConstructorCallback::bounded(
        0,
        |_| Err(Error::BackendUnavailable),
        |scope, _| {
            Ok(ConstructorCompletion::Throw(
                scope.type_error(&ascii(b"Illegal constructor"))?,
            ))
        },
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
    #[cfg(test)]
    pub(in crate::document_scripting::guest) fn refuse_next_constructor_fallback(&self) {
        self.document
            .ordinary
            .refuse_next_constructor_fallback
            .set(true);
    }

    #[cfg(test)]
    pub(in crate::document_scripting::guest) fn refuse_next_ordinary_wrapper_validation(&self) {
        self.document.ordinary.refuse_next_validation.set(true);
    }

    fn other_document_origin(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        document: DocumentId,
    ) -> Result<Option<DomHostedGlobalId>, DocumentGuestEventError> {
        self.callbacks.with_read(owner, events, |read, attached| {
            let node = read.node(document.node())?;
            read.step()?;
            let origin = node
                .relevant_global()
                .map_err(DomHostedRuntimeError::Dom)
                .map_err(DocumentScriptNativeError::from)?;
            Ok(origin.filter(|origin| *origin != attached))
        })
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
            self.callbacks.with_read(&owner, &events, |read, _| {
                read.node(bindings.attached_id.node())?
                    .as_document()
                    .ok_or(DomError::WrongNodeKind)?;
                Ok(())
            })?;
            bindings
                .open_document(scope, bindings.attached_id.node(), || events.step())?
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
            let observation = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id.node())?;
                let node_type = id.node_type(node.kind())?;
                Ok(match member {
                    NodeMember::Type => Observation::Number(node_type),
                    NodeMember::Name => {
                        if let Some(attribute) = node.as_attr() {
                            Observation::AttrName(attribute.name().clone())
                        } else if let Some(name) = node.tag_name_spelling_with(|| read.step())? {
                            Observation::TagName(name)
                        } else {
                            Observation::String(Some(node.node_name()))
                        }
                    }
                    NodeMember::OwnerDocument => {
                        Observation::Node(node.owner_document().map(DocumentId::node))
                    }
                    NodeMember::Parent => Observation::Node(node.parent()),
                    NodeMember::ParentElement => {
                        Observation::Node(node.parent_element_with(|| read.step())?)
                    }
                    NodeMember::FirstChild => Observation::Node(node.children().first().copied()),
                    NodeMember::LastChild => Observation::Node(node.children().last().copied()),
                    NodeMember::Sibling(direction) => {
                        Observation::Node(node.sibling_with(direction, || read.step())?)
                    }
                    NodeMember::HasChildren => Observation::Boolean(!node.children().is_empty()),
                    NodeMember::Connected => Observation::Boolean(
                        node.root_with(true, || read.step())?
                            .as_document()
                            .is_some(),
                    ),
                })
            })?;
            self.project_observation(scope, &owner, &events, observation)
        })();
        complete(scope, &events, result)
    }

    fn project_observation<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        observation: Observation,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        match observation {
            Observation::Number(value) => Ok(scope.primitive(Primitive::Number(value))?),
            Observation::Boolean(value) => Ok(scope.primitive(Primitive::Boolean(value))?),
            Observation::String(Some(value)) => self.copy_dom_units(scope, value.code_units()),
            Observation::TagName(name) => self.copy_tag_name(scope, owner, events, &name),
            Observation::AttrName(name) => self.copy_attr_name(scope, owner, events, &name),
            Observation::String(None) | Observation::Node(None) => {
                Ok(scope.primitive(Primitive::Null)?)
            }
            Observation::Node(Some(node)) => events
                .public
                .as_ref()
                .ok_or(Error::ExpiredIdentity)?
                .project_node(scope, owner, events, node),
        }
    }

    fn copy_dom_units<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        units: &[u16],
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        Ok(self
            .callbacks
            .conversion
            .run(scope, |conversion| conversion.copy_to_js(units))
            .map_err(Error::from)?)
    }

    fn copy_name_units<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        length: usize,
        units: impl Iterator<Item = u16>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let bytes = length
            .checked_mul(size_of::<u16>())
            .ok_or(Error::Allocation)?;
        let _backing = events.reserve(owner, bytes)?;
        events.charge_work(u64::try_from(length).map_err(|_| Error::StringTooLong)?)?;
        let mut buffer = Vec::new();
        buffer
            .try_reserve_exact(length)
            .map_err(|_| Error::Allocation)?;
        if buffer.capacity() != length {
            return Err(Error::Allocation.into());
        }
        buffer.extend(units);
        self.copy_dom_units(scope, &buffer)
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
            self.callbacks.with_read(&owner, &events, |read, _| {
                read.node(id.node())?;
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
            let root = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id.node())?;
                Ok(node.root_with(options.composed, || read.step())?.id())
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
            bindings.publish_document(
                scope,
                ordinary::Admission::ordinary(&owner, &events),
                ordinary::NodePrototype::Selected(selected.prototype()),
                || {
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
                },
            )
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

pub(super) type ReactionFailureSlot = HostFailureSlot<DocumentGuestEventError, GuestBacking>;

pub(super) fn prepare_reaction_failure(
    scope: &mut Scope<'_>,
    owner: &NativeOwner,
    events: &GuestEvents,
) -> Result<ReactionFailureSlot, DocumentGuestEventError> {
    let backing = events.reserve(
        owner,
        ReactionFailureSlot::storage_bytes() + 2 * size_of::<usize>(),
    )?;
    Ok(ReactionFailureSlot::new(
        &scope.realm_authority()?,
        backing,
    )?)
}

pub(super) fn complete<'scope>(
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
