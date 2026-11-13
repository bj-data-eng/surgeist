//! Element projection over a current canonical Document read.

use super::character_data::CharacterKind;
use super::*;
use surgeist_dom::DomAttrId;
use surgeist_html::{
    bounded::lexical::validate_custom_element_name_with, vocabulary::KnownHtmlElement,
};

mod attribute;
pub(super) use attribute::AttributeResult;
mod svg;

const ELEMENT: &[u16] = &ascii(b"Element");
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
    constructor: RetainedCallbackObject,
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
    ShadowRoot,
}

enum ElementObservation {
    Raw(Option<DocumentString>),
    TagName(DomTagName),
    ShadowRoot(Option<NodeId>),
}

enum NativeProjection {
    Document(DocumentId),
    Fragment,
    ShadowRoot,
    Character(CharacterKind),
    Element(ElementInterface),
    Attr(DomAttrId),
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
        let (html_constructor, html) = install_html_constructor(
            scope,
            &global,
            "HTMLElement",
            &constructor,
            &element,
            capture,
        )?;
        let (media_constructor, media) = install_html_constructor(
            scope,
            &global,
            "HTMLMediaElement",
            &html_constructor,
            &html,
            capture,
        )?;
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
        for (interface, constructor, prototype) in [
            ("HTMLElement", &html_constructor, &html),
            ("HTMLMediaElement", &media_constructor, &media),
        ] {
            let prototype = scope.callback_object(prototype)?;
            let constructor = scope.callback_object(constructor)?;
            classes.push(HtmlPrototype {
                interface,
                prototype: scope.retain_callback_object(prototype, fund)?,
                constructor: scope.retain_callback_object(constructor, fund)?,
            });
        }
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
            let (constructor, prototype) = install_html_constructor(
                scope,
                &global,
                interface,
                parent_constructor,
                parent,
                capture,
            )?;
            let prototype = scope.callback_object(&prototype)?;
            let constructor = scope.callback_object(&constructor)?;
            classes.push(HtmlPrototype {
                interface,
                prototype: scope.retain_callback_object(prototype, fund)?,
                constructor: scope.retain_callback_object(constructor, fund)?,
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
            (
                ElementMember::ShadowRoot,
                &ascii(b"shadowRoot")[..],
                &ascii(b"get shadowRoot")[..],
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
        super::shadow::install_attachment(scope, &element, capture)?;
        custom_elements::install_node_getter(
            scope,
            &element,
            custom_elements::Receiver::Element,
            capture,
        )?;
        attribute::install(scope, &element, capture)?;
        super::parent_node::install(
            scope,
            &element,
            super::parent_node::Receiver::Element,
            capture,
        )?;
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

    pub(super) fn prototype_for_name(
        &self,
        name: &DomQualifiedName,
        mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
    ) -> Result<&RetainedCallbackObject, DocumentGuestEventError> {
        let interface = element_interface(name, &mut step)?;
        self.prototype(interface, step)
    }

    fn prototype(
        &self,
        interface: ElementInterface,
        mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
    ) -> Result<&RetainedCallbackObject, DocumentGuestEventError> {
        let interface = match interface {
            ElementInterface::Element => return Ok(&self.element),
            ElementInterface::Mathml => return Ok(&self.mathml),
            ElementInterface::Html(interface) => interface,
            ElementInterface::Svg(interface) => return self.svg.prototype(interface),
        };
        Ok(&self.html_interface(interface, &mut step)?.prototype)
    }

    fn html_interface(
        &self,
        interface: &'static str,
        mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
    ) -> Result<&HtmlPrototype, DocumentGuestEventError> {
        for entry in &self.html {
            step()?;
            if entry.interface == interface {
                return Ok(entry);
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
    mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
) -> Result<ElementInterface, DocumentGuestEventError> {
    step()?;
    match name.namespace_uri().map(DocumentString::code_units) {
        Some(HTML_NAMESPACE) => Ok(ElementInterface::Html(html_interface(
            name.local_name(),
            step,
        )?)),
        Some(MATHML_NAMESPACE) => Ok(ElementInterface::Mathml),
        Some(SVG_NAMESPACE) => Ok(ElementInterface::Svg(svg::interface(
            name.local_name(),
            step,
        )?)),
        _ => Ok(ElementInterface::Element),
    }
}

pub(in crate::document_scripting::guest::interfaces) fn html_interface(
    local: &DocumentString,
    mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
) -> Result<&'static str, DocumentGuestEventError> {
    let matches = |spelling: &str| {
        local
            .code_units()
            .iter()
            .copied()
            .eq(spelling.bytes().map(u16::from))
    };
    step()?;
    if let Some(known) = KnownHtmlElement::from_units(local.code_units()) {
        return Ok(known.inventory_row().interface());
    }
    for &spelling in OBSOLETE_UNKNOWN {
        step()?;
        if matches(spelling) {
            return Ok("HTMLUnknownElement");
        }
    }
    for &spelling in OBSOLETE_GENERIC {
        step()?;
        if matches(spelling) {
            return Ok("HTMLElement");
        }
    }
    for &(spelling, interface) in OBSOLETE_SPECIAL {
        step()?;
        if matches(spelling) {
            return Ok(interface);
        }
    }
    let valid = validate_custom_element_name_with(local, |_| step())?.is_ok();
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
    let callback = ConstructorCallback::bounded(
        0,
        |_| Err(Error::BackendUnavailable),
        |scope, _| {
            Ok(ConstructorCompletion::Throw(
                scope.type_error(&ascii(b"Illegal constructor"))?,
            ))
        },
    );
    install_constructor(scope, global, name, parent_constructor, parent, &callback)
}

fn install_constructor<'scope>(
    scope: &mut Scope<'scope>,
    global: &Value<'scope>,
    name: &[u16],
    parent_constructor: &Value<'scope>,
    parent: &Value<'scope>,
    callback: &ConstructorCallback,
) -> Result<(Value<'scope>, Value<'scope>), Error> {
    let prototype = scope.object(ObjectPrototype::Object(parent))?;
    let constructor = scope.constructor(callback, FunctionMetadata { name, length: 0 })?;
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

impl PublicEventBindings {
    pub(super) fn unknown_element_prototype(
        &self,
        events: &GuestEvents,
        global: DomHostedGlobalId,
    ) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
        let retain = |bindings: &ElementBindings| {
            bindings
                .prototype(ElementInterface::Html("HTMLUnknownElement"), || {
                    events.step()
                })
                .cloned()
        };
        if self.custom_elements.is_global(global) {
            retain(&self.document.element)
        } else {
            self.with_origin(events, global, |origin| retain(&origin.elements))
        }
    }

    fn html_element_prototype<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        primary_realm: RealmId,
        interface: &'static str,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let realm = scope.realm_authority()?.identity();
        let open = |scope: &mut Scope<'scope>, bindings: &ElementBindings| {
            let prototype =
                bindings.prototype(ElementInterface::Html(interface), || events.step())?;
            Ok(scope.open_callback_object(prototype)?)
        };
        if realm == primary_realm {
            return open(scope, &self.document.element);
        }
        let origins = self
            .origins
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut next = origins.as_deref();
        while let Some(origin) = next {
            events.step()?;
            if origin.target.associated_realm().identity() == realm {
                return open(scope, &origin.elements);
            }
            next = origin.next.as_deref();
        }
        Err(DocumentGuestEventError::CallbackRealmUnavailable)
    }
}

impl Capture {
    fn html_element_prototype<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        interface: &'static str,
    ) -> Result<Value<'scope>, Error> {
        let events = self
            .callbacks
            .access
            .get()
            .and_then(|access| access.events.upgrade())
            .ok_or(Error::ExpiredIdentity)?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        match public.html_element_prototype(scope, &events, self.callbacks.permit.realm, interface)
        {
            Ok(prototype) => Ok(prototype),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn construct_html_element<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: ConstructorArguments<'scope>,
        interface: &'static str,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let attached = owner
                .attachment
                .get()
                .ok_or(Error::ExpiredIdentity)?
                .target
                .global();
            let global = self
                .origin
                .as_ref()
                .map_or(attached, |origin| origin.global);
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let own = if self.origin.is_some() {
                public.with_origin(&events, global, |origin| {
                    let own = origin
                        .elements
                        .html_interface(interface, || events.step())?;
                    Ok(scope.open_callback_object(&own.constructor)?)
                })?
            } else {
                let own = public
                    .document
                    .element
                    .html_interface(interface, || events.step())?;
                scope.open_callback_object(&own.constructor)?
            };
            if scope.strict_equals(&own, arguments.new_target())? {
                return Ok(ConstructorCompletion::Throw(
                    scope.type_error(&ascii(b"Illegal constructor"))?,
                ));
            }
            let Some(frame) = custom_elements::active_constructor_for_target(
                &owner,
                &events,
                scope,
                arguments.new_target(),
            )?
            else {
                return Ok(ConstructorCompletion::Throw(
                    scope.type_error(&ascii(b"Illegal constructor"))?,
                ));
            };
            if frame.interface != interface {
                return Ok(ConstructorCompletion::Throw(
                    scope.type_error(&ascii(b"Illegal constructor"))?,
                ));
            }
            let selected = arguments.select_prototype(scope)?;
            if let Some(receiver) = frame.receiver.get() {
                if frame.constructed.get() {
                    return Ok(ConstructorCompletion::Throw(
                        scope.type_error(&ascii(b"Element already constructed"))?,
                    ));
                }
                let receiver = scope.import(receiver)?;
                match scope.set_prototype(&receiver, ObjectPrototype::Object(selected.prototype()))
                {
                    Err(Error::PropertyRejected) => {
                        return Ok(ConstructorCompletion::Throw(
                            scope.type_error(&ascii(b"Cannot set element prototype"))?,
                        ));
                    }
                    result => result?,
                }
                frame.constructed.set(true);
                return Ok(ConstructorCompletion::Return(receiver));
            }
            let element = public.document.ordinary.construct_autonomous(
                scope,
                (&owner, &events),
                &frame,
                global,
                selected.prototype(),
            )?;
            Ok(ConstructorCompletion::Return(element))
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

fn install_html_constructor<'scope>(
    scope: &mut Scope<'scope>,
    global: &Value<'scope>,
    interface: &'static str,
    parent_constructor: &Value<'scope>,
    parent: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(Value<'scope>, Value<'scope>), DocumentGuestEventError> {
    let mut buffer = [0; 48];
    let units = buffer
        .get_mut(..interface.len())
        .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
    for (unit, byte) in units.iter_mut().zip(interface.bytes()) {
        *unit = u16::from(byte);
    }
    let fallback = capture(size_of::<Capture>())?;
    let construct = capture(size_of::<Capture>())?;
    let callback = ConstructorCallback::bounded(
        0,
        move |scope| fallback.html_element_prototype(scope, interface),
        move |scope, arguments| construct.construct_html_element(scope, arguments, interface),
    );
    Ok(install_constructor(
        scope,
        global,
        units,
        parent_constructor,
        parent,
        &callback,
    )?)
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
        let request = owner.attached_event_request();
        let attribute = owner.active_attribute_registration()?;
        let active = owner.current_constructor_registration()?;
        let admission = match (&active, &attribute, &request) {
            (Some(active), _, _) => ordinary::Admission::constructor(owner, events, active),
            (None, Some((request, registration)), _) => {
                ordinary::Admission::attribute(owner, events, *request, registration)
            }
            (None, None, Some(request)) => ordinary::Admission::source(owner, events, request),
            (None, None, None) => ordinary::Admission::ordinary(owner, events),
        };
        if let Some(value) = bindings.find(scope, admission, node)? {
            return Ok(value);
        }
        let (origin, attached, projection) =
            self.callbacks.with_read(owner, events, |read, attached| {
                let node = read.node(node)?;
                if !matches!(
                    node.kind(),
                    DomNodeKind::Document(_)
                        | DomNodeKind::DocumentFragment
                        | DomNodeKind::ShadowRoot(_)
                        | DomNodeKind::Text(_)
                        | DomNodeKind::Comment(_)
                        | DomNodeKind::Element(_)
                        | DomNodeKind::Attr(_)
                ) {
                    return Err(DocumentScriptNativeError::ProjectionCustody.into());
                }
                read.step()?;
                let origin = node
                    .relevant_global()
                    .map_err(DomHostedRuntimeError::Dom)
                    .map_err(DocumentScriptNativeError::from)?;
                if let Some(document) = node.as_document() {
                    // Rust-created Documents with no Global acquire their
                    // attachment's Document interface on first exposure.
                    return Ok((
                        origin.unwrap_or(attached),
                        attached,
                        NativeProjection::Document(document.id()),
                    ));
                }
                let origin = origin.ok_or(DocumentScriptNativeError::CreationGlobalUnavailable)?;
                if matches!(node.kind(), DomNodeKind::DocumentFragment) {
                    return Ok((origin, attached, NativeProjection::Fragment));
                }
                if matches!(node.kind(), DomNodeKind::ShadowRoot(_)) {
                    return Ok((origin, attached, NativeProjection::ShadowRoot));
                }
                if let Some(kind) = match node.kind() {
                    DomNodeKind::Text(_) => Some(CharacterKind::Text),
                    DomNodeKind::Comment(_) => Some(CharacterKind::Comment),
                    _ => None,
                } {
                    return Ok((origin, attached, NativeProjection::Character(kind)));
                }
                if let Some(attribute) = node.as_attr() {
                    return Ok((origin, attached, NativeProjection::Attr(attribute.id())));
                }
                let element = node
                    .as_element()
                    .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                Ok((
                    origin,
                    attached,
                    NativeProjection::Element(if element.has_failed_construction_interface() {
                        ElementInterface::Html("HTMLUnknownElement")
                    } else {
                        element_interface(element.name(), || read.step())?
                    }),
                ))
            })?;
        let watch = |scope: &mut Scope<'scope>, node, prototype: &RetainedCallbackObject| match (
            &active, &attribute, &request,
        ) {
            (Some(_), _, _) | (None, Some(_), _) => bindings
                .ordinary
                .watch_registered_callback(scope, admission, node, prototype),
            (None, None, Some(request)) => bindings.ordinary.watch_source(
                scope,
                (owner, events),
                request,
                node,
                prototype,
                Ok(()),
            ),
            (None, None, None) => {
                bindings
                    .ordinary
                    .watch(scope, owner, events, &self.callbacks, node, prototype)
            }
        };
        let project = |documents: &DocumentPrototypes, elements: &ElementBindings| match projection
        {
            NativeProjection::Document(document) => {
                // Published Documents survive for the native owner's lifetime.
                bindings.publish_document(
                    scope,
                    admission,
                    ordinary::NodePrototype::Creation(&documents.document),
                    || Ok(document),
                )
            }
            NativeProjection::Fragment => {
                watch(scope, GuestNode::Fragment(node), &documents.fragment)
            }
            NativeProjection::ShadowRoot => {
                watch(scope, GuestNode::ShadowRoot(node), &documents.shadow)
            }
            NativeProjection::Character(kind) => watch(
                scope,
                kind.project(node),
                documents.character_data.prototype(kind),
            ),
            NativeProjection::Element(interface) => {
                let prototype = elements.prototype(interface, || admission.step())?;
                watch(scope, GuestNode::Element(node), prototype)
            }
            NativeProjection::Attr(attribute) => {
                watch(scope, GuestNode::Attr(attribute.node()), &documents.attr)
            }
        };
        if origin == attached {
            project(&bindings.prototypes, &bindings.element)
        } else {
            self.with_origin_with(
                origin,
                || admission.step(),
                |origin| project(&origin.documents, &origin.elements),
            )
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
        if let Some(units) = name.contiguous_units() {
            self.copy_dom_units(scope, units)
        } else {
            self.copy_name_units(
                scope,
                owner,
                events,
                name.code_unit_len(),
                name.code_units(),
            )
        }
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
            let root = self.callbacks.with_read(&owner, &events, |read, _| {
                read.snapshot
                    .document_element_with(document, || read.step())
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
            let value = self.callbacks.with_read(&owner, &events, |read, _| {
                let node = read.node(id)?;
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
                        node.tag_name_spelling_with(|| read.step())?
                            .ok_or(DocumentScriptNativeError::ProjectionCustody)?,
                    ),
                    ElementMember::ShadowRoot => {
                        ElementObservation::ShadowRoot(node.shadow_root_with(|| read.step())?)
                    }
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
                ElementObservation::ShadowRoot(None) => Ok(scope.primitive(Primitive::Null)?),
                ElementObservation::ShadowRoot(Some(root)) => events
                    .public
                    .as_ref()
                    .ok_or(Error::ExpiredIdentity)?
                    .project_node(scope, &owner, &events, root),
            }
        })();
        complete(scope, &events, result)
    }
}
