//! Element projection over a current canonical Document read.

use super::*;
use surgeist_dom::DomError;
use surgeist_html::vocabulary::KnownHtmlElement;

const ELEMENT: &[u16] = &ascii(b"Element");
const HTML_ELEMENT: &[u16] = &ascii(b"HTMLElement");
const HTML_HTML_ELEMENT: &[u16] = &ascii(b"HTMLHtmlElement");

pub(super) struct ElementBindings {
    html_root: RetainedCallbackObject,
}

#[derive(Clone, Copy)]
enum ElementMember {
    Namespace,
    Prefix,
    LocalName,
    TagName,
}

enum RootReadError {
    Dom(DomError),
    Guest(DocumentGuestEventError),
}

impl From<DomError> for RootReadError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}

impl From<RootReadError> for DocumentGuestEventError {
    fn from(error: RootReadError) -> Self {
        match error {
            RootReadError::Dom(error) => DocumentScriptNativeError::from(error).into(),
            RootReadError::Guest(error) => error,
        }
    }
}

impl ElementBindings {
    pub(super) fn install<'scope>(
        scope: &mut Scope<'scope>,
        node: &Value<'scope>,
        node_constructor: &Value<'scope>,
        document: &Value<'scope>,
        capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
        fund: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
    ) -> Result<Self, DocumentGuestEventError> {
        let global = scope.global()?;
        let (constructor, element) = install(scope, &global, ELEMENT, node_constructor, node)?;
        let (html_constructor, html) =
            install(scope, &global, HTML_ELEMENT, &constructor, &element)?;
        let (_, html_root) = install(scope, &global, HTML_HTML_ELEMENT, &html_constructor, &html)?;
        for (member, key, name) in [
            (
                ElementMember::Namespace,
                &ascii(b"namespaceURI")[..],
                &ascii(b"get namespaceURI")[..],
            ),
            (
                ElementMember::Prefix,
                &ascii(b"prefix")[..],
                &ascii(b"get prefix")[..],
            ),
            (
                ElementMember::LocalName,
                &ascii(b"localName")[..],
                &ascii(b"get localName")[..],
            ),
            (
                ElementMember::TagName,
                &ascii(b"tagName")[..],
                &ascii(b"get tagName")[..],
            ),
        ] {
            let handler = capture(size_of::<(Capture, ElementMember)>())?;
            install_getter(
                scope,
                &element,
                key,
                name,
                true,
                Callback::new(move |scope, arguments| {
                    handler.element_member(scope, arguments, member)
                }),
            )?;
        }
        let handler = capture(size_of::<Capture>())?;
        install_getter(
            scope,
            document,
            &ascii(b"documentElement"),
            &ascii(b"get documentElement"),
            true,
            Callback::new(move |scope, arguments| handler.document_element(scope, arguments)),
        )?;
        let observed = scope.callback_object(&html_root)?;
        Ok(Self {
            html_root: scope.retain_callback_object(observed, fund)?,
        })
    }
}

fn install<'scope>(
    scope: &mut Scope<'scope>,
    global: &Value<'scope>,
    name: &[u16],
    parent_constructor: &Value<'scope>,
    parent: &Value<'scope>,
) -> Result<(Value<'scope>, Value<'scope>), Error> {
    let prototype = scope.object(ObjectPrototype::Object(parent))?;
    let callback = ConstructorCallback::bounded(
        0,
        |_| Err(Error::BackendUnavailable),
        |scope, _| {
            Ok(ConstructorCompletion::Throw(
                scope.type_error(&ascii(b"Illegal constructor"))?,
            ))
        },
    );
    let constructor = scope.constructor(&callback, FunctionMetadata { name, length: 0 })?;
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
    install_interface(scope, global, name, &constructor, &prototype)?;
    Ok((constructor, prototype))
}

impl Capture {
    fn document_element<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Document(document)) = receiver.and_then(|node| node.payload().id())
        else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Document required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let bindings = &events
                .public
                .as_ref()
                .ok_or(Error::ExpiredIdentity)?
                .document;
            let observed = self.with_native(&owner, |session, global| {
                let snapshot = session.snapshot();
                let root = snapshot.document_element_with(document, || {
                    events.step().map_err(RootReadError::Guest)
                })?;
                let Some(root) = root else {
                    return Ok(None);
                };
                events.step()?;
                let element = snapshot
                    .node(root)
                    .map_err(DocumentScriptNativeError::from)?
                    .as_element()
                    .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                events.step()?;
                let origin = snapshot.relevant_global(root).map_err(|error| {
                    DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Dom(error))
                })?;
                Ok(Some((
                    root,
                    origin,
                    global,
                    KnownHtmlElement::from_name(element.name()),
                )))
            })?;
            let Some((root, origin, global, kind)) = observed else {
                return Ok(scope.primitive(Primitive::Null)?);
            };
            if let Some(value) = bindings.ordinary.open(scope, &events, root)? {
                return Ok(value);
            }
            if origin != Some(global) {
                return Err(DocumentScriptNativeError::CreationGlobalUnavailable.into());
            }
            if kind != Some(KnownHtmlElement::Html) {
                return Err(Error::BackendUnavailable.into());
            }
            let prototype = scope.open_callback_object(&bindings.element.html_root)?;
            bindings
                .ordinary
                .project(scope, &owner, &events, &prototype, |inbox, key| {
                    self.with_native(&owner, |session, _| {
                        let watch = session
                            .dom()
                            .raw_mut()
                            .watch_node_retirement(
                                root,
                                inbox,
                                key,
                                events.limits.node_retirement,
                                &owner.cancellation,
                            )
                            .map_err(DocumentScriptNativeError::from)?;
                        Ok((GuestNode::Element(root), watch))
                    })
                })
        })();
        complete(scope, &events, result)
    }

    fn element_member<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        member: ElementMember,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Element(id)) = receiver.and_then(|node| node.payload().id()) else {
            return Ok(Completion::Throw(
                scope.type_error(&ascii(b"Element required"))?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let value = self.with_native(&owner, |session, _| {
                let snapshot = session.snapshot();
                let node = snapshot.node(id).map_err(DocumentScriptNativeError::from)?;
                let element = node
                    .as_element()
                    .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                let name = element.name();
                Ok(match member {
                    ElementMember::Namespace => name.namespace_uri().cloned(),
                    ElementMember::Prefix => name.prefix().cloned(),
                    ElementMember::LocalName => Some(name.local_name().clone()),
                    ElementMember::TagName => node.tag_name(),
                })
            })?;
            let Some(value) = value else {
                return Ok(scope.primitive(Primitive::Null)?);
            };
            Ok(self
                .callbacks
                .conversion
                .run(scope, |conversion| {
                    conversion.copy_to_js(value.code_units())
                })
                .map_err(Error::from)?)
        })();
        complete(scope, &events, result)
    }
}
