//! Element projection over a current canonical Document read.

use super::*;
use surgeist_dom::DomQualifiedName;
use surgeist_html::{
    bounded::lexical::validate_custom_element_name_with, vocabulary::KnownHtmlElement,
};

mod svg;

const ELEMENT: &[u16] = &ascii(b"Element");
const HTML_ELEMENT: &[u16] = &ascii(b"HTMLElement");
const HTML_MEDIA_ELEMENT: &[u16] = &ascii(b"HTMLMediaElement");
const MATHML_ELEMENT: &[u16] = &ascii(b"MathMLElement");
const HTML_NAMESPACE: &[u16] = &ascii(b"http://www.w3.org/1999/xhtml");
const MATHML_NAMESPACE: &[u16] = &ascii(b"http://www.w3.org/1998/Math/MathML");
const SVG_NAMESPACE: &[u16] = &ascii(b"http://www.w3.org/2000/svg");

// The HTML element interface algorithm and the defining obsolete clauses
// supplement the language owner's individually named element inventory.
const OBSOLETE_UNKNOWN: &[&str] = &[
    "applet", "bgsound", "blink", "isindex", "keygen", "multicol", "nextid", "spacer",
];
const OBSOLETE_GENERIC: &[&str] = &[
    "acronym",
    "basefont",
    "big",
    "center",
    "nobr",
    "noembed",
    "noframes",
    "plaintext",
    "rb",
    "rtc",
    "strike",
    "tt",
];
const OBSOLETE_SPECIAL: &[(&str, &str)] = &[
    ("dir", "HTMLDirectoryElement"),
    ("font", "HTMLFontElement"),
    ("frame", "HTMLFrameElement"),
    ("frameset", "HTMLFrameSetElement"),
    ("listing", "HTMLPreElement"),
    ("marquee", "HTMLMarqueeElement"),
    ("param", "HTMLParamElement"),
    ("xmp", "HTMLPreElement"),
];

pub(in crate::document_scripting::guest::interfaces) struct ElementBindings {
    element: RetainedCallbackObject,
    mathml: RetainedCallbackObject,
    html: Vec<HtmlPrototype>,
    svg: svg::SvgBindings,
    _html_backing: GuestBacking,
}

struct HtmlPrototype {
    interface: &'static str,
    prototype: RetainedCallbackObject,
}

enum ElementInterface {
    Element,
    Html(&'static str),
    Mathml,
    Svg(svg::Interface),
}

#[derive(Clone, Copy)]
enum ElementMember {
    Namespace,
    Prefix,
    LocalName,
    TagName,
}

enum ElementObservation {
    Raw(Option<DocumentString>),
    TagName(DomTagName),
}

enum NativeProjection {
    Document(DocumentId),
    Element {
        origin: DomHostedGlobalId,
        attached: DomHostedGlobalId,
        interface: ElementInterface,
    },
}

impl ElementBindings {
    pub(in crate::document_scripting::guest::interfaces) fn install<'scope>(
        scope: &mut Scope<'scope>,
        node: &Value<'scope>,
        node_constructor: &Value<'scope>,
        capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
        fund: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
    ) -> Result<Self, DocumentGuestEventError> {
        let global = scope.global()?;
        let (constructor, element) = install(scope, &global, ELEMENT, node_constructor, node)?;
        let (_, mathml) = install(scope, &global, MATHML_ELEMENT, &constructor, &element)?;
        let svg = svg::SvgBindings::install(scope, &global, &constructor, &element, fund)?;
        let (html_constructor, html) =
            install(scope, &global, HTML_ELEMENT, &constructor, &element)?;
        let (media_constructor, media) =
            install(scope, &global, HTML_MEDIA_ELEMENT, &html_constructor, &html)?;
        let capacity = KnownHtmlElement::ALL.len() + OBSOLETE_SPECIAL.len() + 2;
        let bytes = capacity
            .checked_mul(size_of::<HtmlPrototype>())
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let html_backing = fund(bytes)?;
        let mut classes = Vec::new();
        classes
            .try_reserve_exact(capacity)
            .map_err(|_| DocumentScriptNativeError::StorageLimit)?;
        if classes.capacity() != capacity {
            return Err(DocumentScriptNativeError::RepresentationExhausted.into());
        }
        let observed = scope.callback_object(&html)?;
        classes.push(HtmlPrototype {
            interface: "HTMLElement",
            prototype: scope.retain_callback_object(observed, fund)?,
        });
        for interface in KnownHtmlElement::ALL
            .iter()
            .map(|element| element.inventory_row().interface())
            .chain(OBSOLETE_SPECIAL.iter().map(|&(_, interface)| interface))
            .chain(["HTMLUnknownElement"])
        {
            if classes.iter().any(|class| class.interface == interface) {
                continue;
            }
            let (parent_constructor, parent) = match interface {
                "HTMLAudioElement" | "HTMLVideoElement" => (&media_constructor, &media),
                _ => (&html_constructor, &html),
            };
            let (_, prototype) =
                install_named(scope, &global, interface, parent_constructor, parent)?;
            let observed = scope.callback_object(&prototype)?;
            classes.push(HtmlPrototype {
                interface,
                prototype: scope.retain_callback_object(observed, fund)?,
            });
        }
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
        let base = scope.callback_object(&element)?;
        let mathml = scope.callback_object(&mathml)?;
        Ok(Self {
            element: scope.retain_callback_object(base, fund)?,
            mathml: scope.retain_callback_object(mathml, fund)?,
            html: classes,
            svg,
            _html_backing: html_backing,
        })
    }

    fn prototype(
        &self,
        events: &GuestEvents,
        interface: ElementInterface,
    ) -> Result<&RetainedCallbackObject, DocumentGuestEventError> {
        let interface = match interface {
            ElementInterface::Element => return Ok(&self.element),
            ElementInterface::Mathml => return Ok(&self.mathml),
            ElementInterface::Html(interface) => interface,
            ElementInterface::Svg(interface) => return self.svg.prototype(interface),
        };
        for entry in &self.html {
            events.step()?;
            if entry.interface == interface {
                return Ok(&entry.prototype);
            }
        }
        Err(DocumentScriptNativeError::ProjectionCustody.into())
    }
}

pub(super) fn install_document_root<'scope>(
    scope: &mut Scope<'scope>,
    document: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(), DocumentGuestEventError> {
    let handler = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        document,
        &ascii(b"documentElement"),
        &ascii(b"get documentElement"),
        true,
        Callback::new(move |scope, arguments| handler.document_element(scope, arguments)),
    )?;
    Ok(())
}

fn element_interface(
    name: &DomQualifiedName,
    events: &GuestEvents,
) -> Result<ElementInterface, DocumentGuestEventError> {
    events.step()?;
    match name.namespace_uri().map(DocumentString::code_units) {
        Some(HTML_NAMESPACE) => Ok(ElementInterface::Html(html_interface(name, events)?)),
        Some(MATHML_NAMESPACE) => Ok(ElementInterface::Mathml),
        Some(SVG_NAMESPACE) => Ok(ElementInterface::Svg(svg::interface(
            name.local_name(),
            events,
        )?)),
        _ => Ok(ElementInterface::Element),
    }
}

fn html_interface(
    name: &DomQualifiedName,
    events: &GuestEvents,
) -> Result<&'static str, DocumentGuestEventError> {
    let local = name.local_name();
    let matches = |spelling: &str| {
        local
            .code_units()
            .iter()
            .copied()
            .eq(spelling.bytes().map(u16::from))
    };
    events.step()?;
    if let Some(known) = KnownHtmlElement::from_units(local.code_units()) {
        return Ok(known.inventory_row().interface());
    }
    for &spelling in OBSOLETE_UNKNOWN {
        events.step()?;
        if matches(spelling) {
            return Ok("HTMLUnknownElement");
        }
    }
    for &spelling in OBSOLETE_GENERIC {
        events.step()?;
        if matches(spelling) {
            return Ok("HTMLElement");
        }
    }
    for &(spelling, interface) in OBSOLETE_SPECIAL {
        events.step()?;
        if matches(spelling) {
            return Ok(interface);
        }
    }
    let valid = validate_custom_element_name_with(local, |_| events.step())?.is_ok();
    Ok(if valid {
        "HTMLElement"
    } else {
        "HTMLUnknownElement"
    })
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

fn install_named<'scope>(
    scope: &mut Scope<'scope>,
    global: &Value<'scope>,
    name: &str,
    parent_constructor: &Value<'scope>,
    parent: &Value<'scope>,
) -> Result<(Value<'scope>, Value<'scope>), DocumentGuestEventError> {
    let mut buffer = [0; 48];
    let units = buffer
        .get_mut(..name.len())
        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
    for (unit, byte) in units.iter_mut().zip(name.bytes()) {
        *unit = u16::from(byte);
    }
    Ok(install(scope, global, units, parent_constructor, parent)?)
}

impl PublicEventBindings {
    pub(in crate::document_scripting::guest) fn project_node<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        node: NodeId,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let bindings = &self.document;
        if let Some(value) = bindings.find(scope, events, node)? {
            return Ok(value);
        }
        let projection = self.callbacks.with_native(owner, |session, attached| {
            let snapshot = session.snapshot();
            let node = snapshot.node_with(node, || events.step())?;
            if !matches!(
                node.kind(),
                DomNodeKind::Document(_) | DomNodeKind::Element(_)
            ) {
                return Err(DocumentScriptNativeError::ProjectionCustody.into());
            }
            events.step()?;
            let origin = node
                .relevant_global()
                .map_err(DomHostedRuntimeError::Dom)
                .map_err(DocumentScriptNativeError::from)?
                .ok_or(DocumentScriptNativeError::CreationGlobalUnavailable)?;
            if let Some(document) = node.as_document() {
                if origin != attached {
                    return Err(DocumentScriptNativeError::CreationGlobalUnavailable.into());
                }
                return Ok(NativeProjection::Document(document.id()));
            }
            let element = node
                .as_element()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            Ok(NativeProjection::Element {
                origin,
                attached,
                interface: element_interface(element.name(), events)?,
            })
        })?;
        let (origin, attached, interface) = match projection {
            NativeProjection::Document(document) => {
                return self.project_unpinned_document(scope, owner, events, document);
            }
            NativeProjection::Element {
                origin,
                attached,
                interface,
            } => (origin, attached, interface),
        };
        let project = |elements: &ElementBindings| {
            let prototype = elements.prototype(events, interface)?;
            bindings.ordinary.project(
                scope,
                owner,
                events,
                ordinary::NodePrototype::Creation(prototype),
                |inbox, key| {
                    self.callbacks.with_native(owner, |session, _| {
                        let watch = session.dom().raw_mut().watch_node_retirement(
                            node,
                            inbox,
                            key,
                            events.limits.node_retirement,
                            &owner.cancellation,
                        )?;
                        Ok((GuestNode::Element(node), watch))
                    })
                },
            )
        };
        if origin == attached {
            project(&bindings.element)
        } else {
            self.with_origin(events, origin, |origin| project(&origin.elements))
        }
    }
}

impl Capture {
    pub(super) fn copy_tag_name<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        name: &DomTagName,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let _backing;
        let buffer;
        let units = if let Some(units) = name.contiguous_units() {
            units
        } else {
            let length = name.code_unit_len();
            let bytes = length
                .checked_mul(size_of::<u16>())
                .ok_or(Error::Allocation)?;
            _backing = events.reserve(owner, bytes)?;
            events.charge_work(u64::try_from(length).map_err(|_| Error::StringTooLong)?)?;
            let mut allocated = Vec::new();
            allocated
                .try_reserve_exact(length)
                .map_err(|_| Error::Allocation)?;
            if allocated.capacity() != length {
                return Err(Error::Allocation.into());
            }
            allocated.extend(name.code_units());
            buffer = allocated;
            &buffer
        };
        Ok(self
            .callbacks
            .conversion
            .run(scope, |conversion| conversion.copy_to_js(units))
            .map_err(Error::from)?)
    }

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
            let root = self.with_native(&owner, |session, _| {
                session
                    .snapshot()
                    .document_element_with(document, || events.step())
            })?;
            match root {
                Some(root) => events
                    .public
                    .as_ref()
                    .ok_or(Error::ExpiredIdentity)?
                    .project_node(scope, &owner, &events, root),
                None => Ok(scope.primitive(Primitive::Null)?),
            }
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
                let node = snapshot.node_with(id, || events.step())?;
                let element = node
                    .as_element()
                    .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                let name = element.name();
                Ok(match member {
                    ElementMember::Namespace => {
                        ElementObservation::Raw(name.namespace_uri().cloned())
                    }
                    ElementMember::Prefix => ElementObservation::Raw(name.prefix().cloned()),
                    ElementMember::LocalName => {
                        ElementObservation::Raw(Some(name.local_name().clone()))
                    }
                    ElementMember::TagName => ElementObservation::TagName(
                        node.tag_name_spelling_with(|| events.step())?
                            .ok_or(DocumentScriptNativeError::ProjectionCustody)?,
                    ),
                })
            })?;
            match value {
                ElementObservation::Raw(None) => Ok(scope.primitive(Primitive::Null)?),
                ElementObservation::Raw(Some(value)) => Ok(self
                    .callbacks
                    .conversion
                    .run(scope, |conversion| {
                        conversion.copy_to_js(value.code_units())
                    })
                    .map_err(Error::from)?),
                ElementObservation::TagName(name) => {
                    self.copy_tag_name(scope, &owner, &events, &name)
                }
            }
        })();
        complete(scope, &events, result)
    }
}
