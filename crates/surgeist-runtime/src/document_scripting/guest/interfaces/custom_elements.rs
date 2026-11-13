//! Guest identity for the canonical custom element registry.

use super::*;
use surgeist_bindings_js::{
    CustomElementRegistryIdentity, ValueKind, conversion::types, dom_exception::DomExceptionName,
};
use surgeist_dom::{
    DocumentString, DomDocumentScriptRegistration, DomNodeKind, DomRegistryBinding,
    DomRegistryDefinitionId, DomRegistryId, DomSnapshot, DomSourceCustomElementRequest,
};
use surgeist_html_semantics::HtmlCustomElementDefinition;

const INTERFACE: &[u16] = &ascii(b"CustomElementRegistry");

pub(in crate::document_scripting) struct ActiveConstructor {
    pub(in crate::document_scripting) request: DomSourceCustomElementRequest,
    pub(in crate::document_scripting) registration: Rc<DomDocumentScriptRegistration>,
    constructor: surgeist_bindings_js::Root,
    previous: Option<Rc<Self>>,
    _backing: GuestBacking,
}

pub(super) struct ConstructorEntry<'a> {
    slot: &'a RefCell<Option<Rc<ActiveConstructor>>>,
    previous: Option<Rc<ActiveConstructor>>,
}

impl Drop for ConstructorEntry<'_> {
    fn drop(&mut self) {
        self.slot.replace(self.previous.take());
    }
}

pub(super) fn active_constructor_for_target<'scope>(
    owner: &NativeOwner,
    events: &GuestEvents,
    scope: &mut Scope<'scope>,
    new_target: &Value<'scope>,
) -> Result<Option<Rc<ActiveConstructor>>, DocumentGuestEventError> {
    let active = owner
        .active_constructor
        .try_borrow()
        .map_err(|_| DocumentScriptNativeError::Busy)?;
    let mut current = active.as_ref();
    while let Some(frame) = current {
        events.step()?;
        let constructor = scope.import(&frame.constructor)?;
        if scope.strict_equals(&constructor, new_target)? {
            return Ok(Some(frame.clone()));
        }
        current = frame.previous.as_ref();
    }
    Ok(None)
}

impl PublicEventBindings {
    pub(super) fn enter_factory_constructor<'a, 'scope>(
        &self,
        owner: &'a NativeOwner,
        events: &GuestEvents,
        scope: &mut Scope<'scope>,
        request: DomSourceCustomElementRequest,
        constructor: &Value<'scope>,
        registration: Rc<DomDocumentScriptRegistration>,
    ) -> Result<ConstructorEntry<'a>, DocumentGuestEventError> {
        let backing = events.reserve(
            owner,
            size_of::<ActiveConstructor>() + 2 * size_of::<usize>(),
        )?;
        let previous = owner
            .active_constructor
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .clone();
        let frame = Rc::new(ActiveConstructor {
            request,
            registration,
            constructor: scope.retain(constructor),
            previous: previous.clone(),
            _backing: backing,
        });
        owner.active_constructor.replace(Some(frame));
        Ok(ConstructorEntry {
            slot: &owner.active_constructor,
            previous,
        })
    }

    pub(super) fn factory_constructor<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        request: DomSourceCustomElementRequest,
    ) -> Result<(Value<'scope>, RealmId), DocumentGuestEventError> {
        let id = request.definition();
        let open = |scope: &mut Scope<'scope>, registry: &GlobalRegistry| {
            events.step()?;
            if registry.id != id.registry() || registry.global != request.global() {
                return Err(DocumentScriptNativeError::ProjectionCustody.into());
            }
            let rows = registry
                .definitions
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let row = rows
                .rows
                .get(id.ordinal())
                .filter(|row| row.id == id)
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            Ok((
                scope.open_callback_object(&row.constructor)?,
                row.constructor.associated_realm().identity(),
            ))
        };
        if self.custom_elements.global == request.global() {
            open(scope, &self.custom_elements)
        } else {
            self.with_origin(events, request.global(), |origin| {
                open(scope, &origin.custom_elements)
            })
        }
    }
}

pub(super) struct GlobalRegistry {
    global: DomHostedGlobalId,
    id: DomRegistryId,
    object: RetainedCallbackObject,
    identity: CustomElementRegistryIdentity,
    definitions: RefCell<DefinitionBank>,
    running: Cell<bool>,
    _backing: GuestBacking,
}

type StringSequence = surgeist_bindings_js::conversion::Converted<
    Vec<surgeist_bindings_js::conversion::Converted<surgeist_bindings_js::DomString>>,
>;
const LIFECYCLE: [&[u16]; 5] = [
    &ascii(b"connectedCallback"),
    &ascii(b"disconnectedCallback"),
    &ascii(b"connectedMoveCallback"),
    &ascii(b"adoptedCallback"),
    &ascii(b"attributeChangedCallback"),
];
const FORM: [&[u16]; 4] = [
    &ascii(b"formAssociatedCallback"),
    &ascii(b"formResetCallback"),
    &ascii(b"formDisabledCallback"),
    &ascii(b"formStateRestoreCallback"),
];
const CALLBACK_COUNT: usize = LIFECYCLE.len() + FORM.len();

struct Definition {
    id: DomRegistryDefinitionId,
    name: DocumentString,
    constructor: RetainedCallbackObject,
    _callbacks: [Option<RetainedCallbackObject>; CALLBACK_COUNT],
    _observed: Option<ObservedAttributes>,
    _disable_internals: bool,
    _name_backing: GuestBacking,
}

struct ObservedAttributes {
    _values: Vec<(DocumentString, GuestBacking)>,
    _backing: GuestBacking,
}

impl ObservedAttributes {
    fn copy(
        owner: &NativeOwner,
        events: &GuestEvents,
        source: StringSequence,
    ) -> Result<Self, DocumentGuestEventError> {
        let count = source.get().len();
        let bytes = count
            .checked_mul(size_of::<(DocumentString, GuestBacking)>())
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let backing = events.reserve(owner, bytes)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| Error::Allocation)?;
        if values.capacity() > count {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        for value in source.get() {
            events.step()?;
            values.push(document::copy_native_data(
                owner,
                events,
                value.get().as_code_units(),
            )?);
        }
        Ok(Self {
            _values: values,
            _backing: backing,
        })
    }
}

#[derive(Default)]
struct DefinitionBank {
    rows: Vec<Definition>,
    backing: Option<GuestBacking>,
}

impl DefinitionBank {
    fn reserve(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
    ) -> Result<Option<DefinitionGrowth>, DocumentGuestEventError> {
        if self.rows.len() < self.rows.capacity() {
            return Ok(None);
        }
        let capacity = self
            .rows
            .len()
            .checked_mul(2)
            .map(|capacity| capacity.max(1))
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let bytes = capacity
            .checked_mul(size_of::<Definition>())
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let backing = events.reserve(owner, bytes)?;
        events.charge_work(
            u64::try_from(self.rows.len()).map_err(|_| DocumentGuestEventError::WorkLimit)?,
        )?;
        let mut rows = Vec::new();
        rows.try_reserve_exact(capacity)
            .map_err(|_| Error::Allocation)?;
        if rows.capacity() > capacity {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        Ok(Some(DefinitionGrowth { rows, backing }))
    }

    fn publish(&mut self, growth: Option<DefinitionGrowth>, definition: Definition) {
        if let Some(mut growth) = growth {
            growth.rows.append(&mut self.rows);
            self.rows = growth.rows;
            self.backing = Some(growth.backing);
        }
        self.rows.push(definition);
    }
}

struct DefinitionGrowth {
    rows: Vec<Definition>,
    backing: GuestBacking,
}

struct Running<'a>(&'a Cell<bool>);
impl Drop for Running<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
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
    (document, global): (DocumentId, DomHostedGlobalId),
    id: DomRegistryId,
    capture: CaptureFunding<'_>,
    fund: &dyn Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
) -> Result<Rc<GlobalRegistry>, DocumentGuestEventError> {
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
    // Test the actual registration boundary while upgrade and promise
    // publication are still being joined to the production interface.
    if cfg!(test) {
        install_definition_methods(scope, &prototype, capture)?;
    }
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
    let backing = fund(size_of::<GlobalRegistry>() + 2 * size_of::<usize>())?;
    Ok(Rc::new(GlobalRegistry {
        global,
        id,
        object,
        identity,
        definitions: RefCell::new(DefinitionBank::default()),
        running: Cell::new(false),
        _backing: backing,
    }))
}

fn install_definition_methods<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (name, length, define) in [
        (&ascii(b"define")[..], 2, true),
        (&ascii(b"get")[..], 1, false),
    ] {
        let capture = capture(size_of::<(Capture, bool)>())?;
        let function = scope.bounded_function(
            &Callback::new(move |scope, arguments| {
                capture.registry_method(scope, arguments, define)
            }),
            FunctionMetadata { name, length },
            length + u32::from(define),
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(name),
            &function,
            true,
            true,
            true,
        )?;
    }
    Ok(())
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
    fn registry_method<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        define: bool,
    ) -> Result<Completion<'scope>, Error> {
        let Some(identity) = scope.custom_element_registry_identity(arguments.receiver())? else {
            return registry_type_error(scope);
        };
        let (Some(name), constructor) = (arguments.get(0), arguments.get(1)) else {
            return registry_type_error(scope);
        };
        if define && constructor.is_none() {
            return registry_type_error(scope);
        }
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<Completion<'scope>, DocumentGuestEventError> {
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let registry = self.callbacks.with_read(&owner, &events, |read, _| {
                public.with_registry(
                    read,
                    |registry| registry.identity == identity,
                    |registry| Ok(Rc::clone(registry)),
                )
            })?;
            let name = self
                .callbacks
                .conversion
                .run(scope, |cx| cx.value(&types::DOM_STRING, name))
                .map_err(Error::from)?;
            let Some(constructor) = constructor.filter(|_| define) else {
                let result = registry.find(scope, &events, |_, row| {
                    registry_name_matches(
                        &events,
                        row.name.code_units(),
                        name.get().as_code_units(),
                    )
                })?;
                return Ok(Completion::Return(match result {
                    Some(value) => value,
                    None => scope.primitive(Primitive::Undefined)?,
                }));
            };
            events.step()?;
            if !scope.is_callable(constructor)? {
                return Ok(registry_type_error(scope)?);
            }
            let extends = match self.registry_extends(scope, &events, arguments.get(2)) {
                Err(DocumentGuestEventError::Document(DocumentScriptError::Script(
                    Error::ExpectedObject,
                ))) => return Ok(registry_type_error(scope)?),
                result => result?,
            };
            if !scope.is_constructor(constructor)? {
                return Ok(registry_type_error(scope)?);
            }
            let (name, name_backing) =
                document::copy_native_data(&owner, &events, name.get().as_code_units())?;
            if surgeist_html::bounded::lexical::validate_custom_element_name_with(
                &name,
                |action| {
                    use surgeist_html::bounded::lexical::HtmlCustomNameScanAction;
                    events.charge_work(match action {
                        HtmlCustomNameScanAction::ReservedPair => 2,
                        _ => 1,
                    })
                },
            )?
            .is_err()
            {
                return Ok(Completion::Throw(
                    scope.dom_exception(DomExceptionName::SyntaxError, &[])?,
                ));
            }
            let named = registry.find(scope, &events, |_, row| {
                registry_name_matches(&events, row.name.code_units(), name.code_units())
            })?;
            if named.is_some() {
                return Ok(registry_not_supported(scope)?);
            }
            let reused = registry.find(scope, &events, |scope, row| {
                let stored = scope.open_callback_object(&row.constructor)?;
                Ok(scope.strict_equals(&stored, constructor)?)
            })?;
            if reused.is_some() {
                return Ok(registry_not_supported(scope)?);
            }
            let local = match extends {
                Some(extends) => {
                    let units = extends.get().as_code_units();
                    let local = document::copy_native_data(&owner, &events, units)?;
                    if surgeist_html::bounded::lexical::validate_custom_element_name_with(
                        &local.0,
                        |_| events.step(),
                    )?
                    .is_ok()
                        || document::element::html_interface(&local.0, || events.step())?
                            == "HTMLUnknownElement"
                    {
                        return Ok(registry_not_supported(scope)?);
                    }
                    Some(local)
                }
                None => None,
            };
            if registry.running.get() {
                return Ok(registry_not_supported(scope)?);
            }
            registry.running.set(true);
            let running = Running(&registry.running);
            let Some(read) = self.read_definition(scope, &events, constructor)? else {
                return Ok(registry_type_error(scope)?);
            };
            drop(running);
            let observed_attributes = read
                .observed
                .map(|source| ObservedAttributes::copy(&owner, &events, source))
                .transpose()?;
            let undefined = scope.primitive(Primitive::Undefined)?;
            let mut callbacks = [const { None }; CALLBACK_COUNT];
            for (output, input) in callbacks.iter_mut().zip(read.callbacks) {
                if let Some(input) = input {
                    events.step()?;
                    let observed = scope.callback_object(&input)?;
                    *output =
                        Some(scope.retain_callback_object(observed, |bytes| {
                            events.reserve(&owner, bytes)
                        })?);
                }
            }
            events.step()?;
            let observed = scope.callback_object(constructor)?;
            let constructor =
                scope.retain_callback_object(observed, |bytes| events.reserve(&owner, bytes))?;
            let local_name = local.as_ref().map_or(&name, |(value, _)| value);
            let facts = HtmlCustomElementDefinition::new(
                name.clone(),
                local_name.clone(),
                read.form,
                read.disable_shadow,
            );
            let mut rows = registry
                .definitions
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let growth = rows.reserve(&owner, &events)?;
            scope.check_active()?;
            let id = self.with_native(&owner, |session, _| {
                let hosted = session.hosted_execution();
                hosted
                    .publish_script_registry_definition(
                        &mut session.dom(),
                        registry.global,
                        registry.id,
                        &facts,
                        events.limits.source_edits,
                        &owner.cancellation,
                    )
                    .map_err(DocumentScriptNativeError::from)
                    .map_err(Into::into)
            })?;
            rows.publish(
                growth,
                Definition {
                    id,
                    name,
                    constructor,
                    _callbacks: callbacks,
                    _observed: observed_attributes,
                    _disable_internals: read.disable_internals,
                    _name_backing: name_backing,
                },
            );
            Ok(Completion::Return(undefined))
        })();
        match result {
            Ok(result) => Ok(result),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn registry_extends<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        options: Option<&Value<'scope>>,
    ) -> Result<
        Option<surgeist_bindings_js::conversion::Converted<surgeist_bindings_js::DomString>>,
        DocumentGuestEventError,
    > {
        let Some(options) = options else {
            return Ok(None);
        };
        match scope.kind(options)? {
            ValueKind::Undefined | ValueKind::Null => return Ok(None),
            ValueKind::Object => {}
            _ => return Err(Error::ExpectedObject.into()),
        }
        events.step()?;
        let value = scope.get(options, PropertyKey::String(&ascii(b"extends")))?;
        if scope.kind(&value)? == ValueKind::Undefined {
            return Ok(None);
        }
        self.callbacks
            .conversion
            .run(scope, |cx| cx.value(&types::DOM_STRING, &value))
            .map(Some)
            .map_err(Error::from)
            .map_err(Into::into)
    }

    fn read_definition<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        constructor: &Value<'scope>,
    ) -> Result<Option<DefinitionRead<'scope>>, DocumentGuestEventError> {
        events.step()?;
        let prototype = scope.get(constructor, PropertyKey::String(PROTOTYPE))?;
        if scope.kind(&prototype)? != ValueKind::Object {
            return Ok(None);
        }
        let mut callbacks = [const { None }; CALLBACK_COUNT];
        for (output, name) in callbacks.iter_mut().zip(LIFECYCLE) {
            if !read_callback(scope, events, &prototype, name, output)? {
                return Ok(None);
            }
        }
        let observed = if callbacks[LIFECYCLE.len() - 1].is_some() {
            self.read_string_sequence(scope, events, constructor, &ascii(b"observedAttributes"))?
        } else {
            None
        };
        let mut disable_shadow = false;
        let mut disable_internals = false;
        if let Some(features) =
            self.read_string_sequence(scope, events, constructor, &ascii(b"disabledFeatures"))?
        {
            for feature in features.get() {
                events.step()?;
                let units = feature.get().as_code_units();
                disable_shadow |= units == ascii(b"shadow");
                disable_internals |= units == ascii(b"internals");
            }
        }
        events.step()?;
        let form = scope.get(constructor, PropertyKey::String(&ascii(b"formAssociated")))?;
        let form = scope.boolean(&form)?;
        if form {
            for (output, name) in callbacks[LIFECYCLE.len()..].iter_mut().zip(FORM) {
                if !read_callback(scope, events, &prototype, name, output)? {
                    return Ok(None);
                }
            }
        }
        Ok(Some(DefinitionRead {
            callbacks,
            observed,
            form,
            disable_shadow,
            disable_internals,
        }))
    }

    fn read_string_sequence<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        constructor: &Value<'scope>,
        name: &[u16],
    ) -> Result<Option<StringSequence>, DocumentGuestEventError> {
        events.step()?;
        let value = scope.get(constructor, PropertyKey::String(name))?;
        if scope.kind(&value)? == ValueKind::Undefined {
            return Ok(None);
        }
        self.callbacks
            .conversion
            .run(scope, |cx| cx.value(&types::DOM_STRING.sequence(), &value))
            .map(Some)
            .map_err(Error::from)
            .map_err(Into::into)
    }

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

struct DefinitionRead<'scope> {
    callbacks: [Option<Value<'scope>>; CALLBACK_COUNT],
    observed: Option<StringSequence>,
    form: bool,
    disable_shadow: bool,
    disable_internals: bool,
}

fn read_callback<'scope>(
    scope: &mut Scope<'scope>,
    events: &GuestEvents,
    prototype: &Value<'scope>,
    name: &[u16],
    output: &mut Option<Value<'scope>>,
) -> Result<bool, DocumentGuestEventError> {
    events.step()?;
    let callback = scope.get(prototype, PropertyKey::String(name))?;
    if scope.kind(&callback)? == ValueKind::Undefined {
        return Ok(true);
    }
    if !scope.is_callable(&callback)? {
        return Ok(false);
    }
    *output = Some(callback);
    Ok(true)
}

impl GlobalRegistry {
    pub(super) fn is_global(&self, global: DomHostedGlobalId) -> bool {
        self.global == global
    }

    fn find<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        mut matches: impl FnMut(
            &mut Scope<'scope>,
            &Definition,
        ) -> Result<bool, DocumentGuestEventError>,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        let rows = self
            .definitions
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        for (ordinal, row) in rows.rows.iter().enumerate() {
            events.step()?;
            if row.id.registry() != self.id || row.id.ordinal() != ordinal {
                return Err(DocumentScriptNativeError::ProjectionCustody.into());
            }
            if matches(scope, row)? {
                return Ok(Some(scope.open_callback_object(&row.constructor)?));
            }
        }
        Ok(None)
    }
}

fn registry_type_error<'scope>(scope: &mut Scope<'scope>) -> Result<Completion<'scope>, Error> {
    Ok(Completion::Throw(scope.type_error(&ascii(
        b"Invalid custom element registry arguments",
    ))?))
}

fn registry_name_matches(
    events: &GuestEvents,
    left: &[u16],
    right: &[u16],
) -> Result<bool, DocumentGuestEventError> {
    if left.len() != right.len() {
        return Ok(false);
    }
    for index in 0..left.len() {
        events.charge_work(2)?;
        if left[index] != right[index] {
            return Ok(false);
        }
    }
    Ok(true)
}

fn registry_not_supported<'scope>(scope: &mut Scope<'scope>) -> Result<Completion<'scope>, Error> {
    Ok(Completion::Throw(
        scope.dom_exception(DomExceptionName::NotSupportedError, &[])?,
    ))
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
        use_registry: impl FnOnce(&Rc<GlobalRegistry>) -> Result<T, DocumentGuestEventError>,
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
