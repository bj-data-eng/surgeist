//! Document and Node bindings for the real attached Window.

use super::*;
use surgeist_bindings_js::conversion::declarations::node_get_root_node_0_options;
use surgeist_dom::DomBatch;
use surgeist_html::values::DocumentString;

const NODE: &[u16] = &ascii(b"Node");
const DOCUMENT: &[u16] = &ascii(b"Document");

pub(in crate::document_scripting::guest) struct GuestDocumentProjection {
    document: Cell<Option<DocumentId>>,
}

impl GuestDocumentProjection {
    pub(in crate::document_scripting::guest) fn id(&self) -> Option<DocumentId> {
        self.document.get()
    }
}

struct ConstructedPin {
    document: Option<DocumentId>,
    holder: Rc<RetainedCallbackObject>,
    next: Option<Box<ConstructedPin>>,
    _backing: GuestBacking,
}

pub(super) struct DocumentBindings {
    attached_id: DocumentId,
    attached: RetainedCallbackObject,
    prototype: RetainedCallbackObject,
    constructed: RefCell<Option<Box<ConstructedPin>>>,
}

impl Drop for DocumentBindings {
    fn drop(&mut self) {
        let head = self.constructed.get_mut();
        while let Some(mut pin) = head.take() {
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
    Boolean(bool),
    Document(Option<DocumentId>),
}

impl DocumentBindings {
    pub(super) fn install<'scope>(
        scope: &mut Scope<'scope>,
        document: DocumentId,
        target_prototype: &Value<'scope>,
        capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
        fund: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
    ) -> Result<Self, DocumentGuestEventError> {
        let node = scope.object(ObjectPrototype::Object(target_prototype))?;
        let prototype = scope.object(ObjectPrototype::Object(&node))?;
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
        let target_constructor = scope.get(target_prototype, PropertyKey::String(CONSTRUCTOR))?;
        scope.set_prototype(
            &node_constructor,
            ObjectPrototype::Object(&target_constructor),
        )?;
        scope.set_prototype(
            &document_constructor,
            ObjectPrototype::Object(&node_constructor),
        )?;

        let global = scope.global()?;
        for (interface, prototype) in [
            (&node_constructor, &node),
            (&document_constructor, &prototype),
        ] {
            scope.define_property(
                interface,
                PropertyKey::String(PROTOTYPE),
                PropertyDescriptor::Data {
                    value: prototype,
                    writable: false,
                    enumerable: false,
                    configurable: false,
                },
            )?;
        }
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
                Callback::new(move |scope, arguments| {
                    handler.node_member(scope, arguments, member)
                }),
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
        for (name, interface, prototype) in [
            (NODE, &node_constructor, &node),
            (DOCUMENT, &document_constructor, &prototype),
        ] {
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
                &global,
                PropertyKey::String(name),
                interface,
                true,
                false,
                true,
            )?;
        }
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
            GuestDocumentProjection {
                document: Cell::new(Some(document)),
            },
            fund,
        )?;
        let observed = scope.callback_object(&prototype)?;
        let prototype = scope.retain_callback_object(observed, fund)?;
        Ok(Self {
            attached_id: document,
            attached,
            prototype,
            constructed: RefCell::new(None),
        })
    }

    pub(super) fn open<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        node: surgeist_dom::NodeId,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        events.step()?;
        if node == self.attached_id.node() {
            return Ok(scope.open_callback_object(&self.attached)?);
        }
        let holder = {
            let pins = self
                .constructed
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let mut next = pins.as_deref();
            loop {
                let pin = next.ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                events.step()?;
                if pin.document.is_some_and(|document| document.node() == node) {
                    break pin.holder.clone();
                }
                next = pin.next.as_deref();
            }
        };
        Ok(scope.open_callback_object(&holder)?)
    }
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
            bindings.open(scope, &events, bindings.attached_id.node())
        })();
        complete(scope, &events, result)
    }

    fn node_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: NodeMember,
    ) -> Result<Completion<'scope>, Error> {
        let Some(projection) =
            scope.native_object::<GuestDocumentProjection>(arguments.receiver())?
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
                let node = snapshot
                    .document(id)
                    .map_err(DocumentScriptNativeError::from)?
                    .node(id.node())
                    .map_err(DocumentScriptNativeError::from)?;
                Ok(match member {
                    NodeMember::Type => Observation::Number(9.0),
                    NodeMember::Name => Observation::String(node.node_name()),
                    NodeMember::OwnerDocument => Observation::Document(node.owner_document()),
                    NodeMember::Parent => {
                        let parent = node
                            .parent()
                            .map(|parent| {
                                snapshot
                                    .node(parent)
                                    .map_err(DocumentScriptNativeError::from)?
                                    .as_document()
                                    .map(|document| document.id())
                                    .ok_or(DocumentScriptNativeError::ProjectionCustody)
                            })
                            .transpose()?;
                        Observation::Document(parent)
                    }
                    NodeMember::Connected => Observation::Boolean(node.is_connected()),
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
                Observation::Document(None) => Ok(scope.primitive(Primitive::Null)?),
                Observation::Document(Some(document)) => events
                    .public
                    .as_ref()
                    .ok_or(Error::ExpiredIdentity)?
                    .document
                    .open(scope, &events, document.node()),
            }
        })();
        complete(scope, &events, result)
    }

    fn root_node<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(projection) =
            scope.native_object::<GuestDocumentProjection>(arguments.receiver())?
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
                session
                    .snapshot()
                    .document(id)
                    .map_err(DocumentScriptNativeError::from)?;
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
                let node = snapshot
                    .document(id)
                    .map_err(DocumentScriptNativeError::from)?
                    .node(id.node())
                    .map_err(DocumentScriptNativeError::from)?;
                Ok(if options.composed {
                    node.shadow_including_root()
                } else {
                    node.root()
                })
            })?;
            if root != id.node() {
                return Err(DocumentScriptNativeError::ProjectionCustody.into());
            }
            let receiver = scope.retain(arguments.receiver());
            Ok(scope.import(&receiver)?)
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
            let backing = events.reserve(
                &owner,
                size_of::<ConstructedPin>() + size_of::<RetainedCallbackObject>(),
            )?;
            let holder = scope.project_retained_native_object(
                ObjectPrototype::Object(selected.prototype()),
                GuestDocumentProjection {
                    document: Cell::new(None),
                },
                |bytes| events.reserve(&owner, bytes),
            )?;
            let value = scope.open_callback_object(&holder)?;
            let projection = scope
                .native_object::<GuestDocumentProjection>(&value)?
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            let mut pin = Box::new(ConstructedPin {
                document: None,
                holder: Rc::new(holder),
                next: None,
                _backing: backing,
            });
            let mut pins = bindings
                .constructed
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let attached = owner
                .attachment
                .get()
                .ok_or(DocumentScriptNativeError::Revoked)?
                .target;
            let document = self.with_native(&owner, |session, global| {
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
            })?;
            projection.payload().document.set(Some(document));
            pin.document = Some(document);
            pin.next = pins.take();
            *pins = Some(pin);
            Ok(value)
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
