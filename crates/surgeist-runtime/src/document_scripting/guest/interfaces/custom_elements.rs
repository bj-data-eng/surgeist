//! Guest identity for the canonical custom element registry.

use super::*;
use surgeist_bindings_js::{
    CustomElementRegistryIdentity, PromiseResolver, ValueKind, conversion::types,
    dom_exception::DomExceptionName,
};
use surgeist_dom::{
    DocumentString, DomDocumentScriptRegistration, DomNodeKind, DomRegistryBinding,
    DomRegistryDefinitionId, DomRegistryId, DomSnapshot, DomSourceCustomElementRequest, NodeId,
};
use surgeist_html_semantics::HtmlCustomElementDefinition;

const INTERFACE: &[u16] = &ascii(b"CustomElementRegistry");

pub(in crate::document_scripting) struct ActiveConstructor {
    pub(in crate::document_scripting) request: DomSourceCustomElementRequest,
    pub(in crate::document_scripting) registration: Rc<DomDocumentScriptRegistration>,
    constructor: surgeist_bindings_js::Root,
    pub(super) interface: &'static str,
    pub(super) receiver: std::cell::OnceCell<surgeist_bindings_js::Root>,
    pub(super) constructed: Cell<bool>,
    pub(super) receiver_registry_matches: Cell<bool>,
    previous: Option<Rc<Self>>,
    _backing: GuestBacking,
}

impl ActiveConstructor {
    pub(super) fn receiver_frame(
        &self,
        events: &GuestEvents,
    ) -> Result<Option<&Self>, DocumentGuestEventError> {
        let mut next = Some(self);
        while let Some(frame) = next {
            events.step()?;
            if frame.request.definition() == self.request.definition()
                && frame.receiver.get().is_some()
            {
                return Ok(Some(frame));
            }
            next = frame.previous.as_deref();
        }
        Ok(None)
    }
}

pub(super) struct ConstructorEntry<'a> {
    slot: &'a RefCell<Option<Rc<ActiveConstructor>>>,
    previous: Option<Rc<ActiveConstructor>>,
    pub(super) frame: Rc<ActiveConstructor>,
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

pub(super) fn reserve_constructor_frame(
    owner: &NativeOwner,
    events: &GuestEvents,
) -> Result<GuestBacking, DocumentGuestEventError> {
    events.reserve(
        owner,
        size_of::<ActiveConstructor>() + 2 * size_of::<usize>(),
    )
}

impl PublicEventBindings {
    pub(super) fn direct_constructor_definition<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        read: &NativeRead<'_>,
        global: DomHostedGlobalId,
        new_target: &Value<'scope>,
    ) -> Result<Option<(DomRegistryDefinitionId, &'static str)>, DocumentGuestEventError> {
        let mut lookup = |registry: &GlobalRegistry| {
            let rows = registry
                .definitions
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            for (ordinal, row) in rows.rows.iter().enumerate() {
                read.step()?;
                if row.id.registry() != registry.id || row.id.ordinal() != ordinal {
                    return Err(DocumentScriptNativeError::ProjectionCustody.into());
                }
                let constructor = scope.open_callback_object(&row.constructor)?;
                if scope.strict_equals(&constructor, new_target)? {
                    return Ok(Some((row.id, row.interface)));
                }
            }
            Ok(None)
        };
        self.with_registry(
            read,
            |registry| registry.global == global,
            |registry| lookup(registry),
        )
    }

    pub(super) fn enter_factory_constructor<'a, 'scope>(
        &self,
        owner: &'a NativeOwner,
        events: &GuestEvents,
        scope: &mut Scope<'scope>,
        request: DomSourceCustomElementRequest,
        (constructor, interface): (&Value<'scope>, &'static str),
        registration: Rc<DomDocumentScriptRegistration>,
    ) -> Result<ConstructorEntry<'a>, DocumentGuestEventError> {
        let backing = reserve_constructor_frame(owner, events)?;
        Self::enter_constructor(
            owner,
            request,
            registration,
            scope.retain(constructor),
            interface,
            backing,
        )
    }

    pub(super) fn enter_constructor<'a>(
        owner: &'a NativeOwner,
        request: DomSourceCustomElementRequest,
        registration: Rc<DomDocumentScriptRegistration>,
        constructor: surgeist_bindings_js::Root,
        interface: &'static str,
        backing: GuestBacking,
    ) -> Result<ConstructorEntry<'a>, DocumentGuestEventError> {
        let previous = owner
            .active_constructor
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .clone();
        let frame = Rc::new(ActiveConstructor {
            request,
            registration,
            constructor,
            interface,
            receiver: std::cell::OnceCell::new(),
            constructed: Cell::new(false),
            receiver_registry_matches: Cell::new(true),
            previous: previous.clone(),
            _backing: backing,
        });
        owner.active_constructor.replace(Some(frame.clone()));
        Ok(ConstructorEntry {
            slot: &owner.active_constructor,
            previous,
            frame,
        })
    }

    pub(super) fn factory_constructor<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        request: DomSourceCustomElementRequest,
    ) -> Result<(Value<'scope>, RealmId, &'static str, bool), DocumentGuestEventError> {
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
                row.interface,
                row.customized,
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
    waiting: RefCell<Option<Box<DefinitionWaiter>>>,
    running: Cell<bool>,
    _backing: GuestBacking,
}

struct DefinitionWaiter {
    name: DocumentString,
    resolver: PromiseResolver,
    next: Option<Box<Self>>,
    _record: GuestBacking,
    _name: GuestBacking,
}

impl Drop for GlobalRegistry {
    fn drop(&mut self) {
        let mut waiting = self.waiting.get_mut().take();
        while let Some(mut entry) = waiting {
            waiting = entry.next.take();
        }
    }
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
    interface: &'static str,
    customized: bool,
    id: DomRegistryDefinitionId,
    name: DocumentString,
    constructor: RetainedCallbackObject,
    callbacks: [Option<RetainedCallbackObject>; CALLBACK_COUNT],
    observed: Option<Rc<ObservedAttributes>>,
    _disable_internals: bool,
    _name_backing: GuestBacking,
}

struct ObservedAttributes {
    values: Vec<(DocumentString, GuestBacking)>,
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
            .and_then(|bytes| bytes.checked_add(size_of::<Self>() + 2 * size_of::<usize>()))
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
            values,
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
    executable: bool,
) -> Result<DomRegistryId, DocumentScriptNativeError> {
    let id = document_registry_identity(snapshot, document)?;
    let registry = snapshot.registry(id)?;
    if executable && registry.definition_count() != 0 {
        return Err(DocumentScriptNativeError::CustomElementRegistryCustodyUnavailable(id));
    }
    Ok(id)
}

pub(in crate::document_scripting) fn document_registry_identity(
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
        waiting: RefCell::new(None),
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
    let capture = capture(size_of::<Capture>())?;
    let name = &ascii(b"whenDefined");
    let function = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| capture.when_defined(scope, arguments)),
        FunctionMetadata { name, length: 1 },
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
    fn when_defined<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<Value<'scope>, DocumentGuestEventError> {
            let registry = scope.custom_element_registry_identity(arguments.receiver())?;
            let (Some(identity), Some(name)) = (registry, arguments.get(0)) else {
                let reason =
                    scope.type_error(&ascii(b"Invalid custom element registry arguments"))?;
                return settled_definition_promise(scope, &owner, &events, &reason, true);
            };
            let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
            let registry = self.callbacks.with_read(&owner, &events, |read, _| {
                public.with_registry(
                    read,
                    |registry| registry.identity == identity,
                    |registry| Ok(Rc::clone(registry)),
                )
            })?;
            let converted = self
                .callbacks
                .conversion
                .run(scope, |cx| cx.value(&types::DOM_STRING, name));
            let name = match converted.map_err(Error::from) {
                Ok(name) => name,
                Err(Error::Exception(exception)) => {
                    let reason = scope.import(exception.value())?;
                    return settled_definition_promise(scope, &owner, &events, &reason, true);
                }
                Err(error) => return Err(error.into()),
            };
            let (name, name_backing) =
                document::copy_native_data(&owner, &events, name.get().as_code_units())?;
            if !valid_registry_name(&events, &name)? {
                let reason = scope.dom_exception(DomExceptionName::SyntaxError, &[])?;
                return settled_definition_promise(scope, &owner, &events, &reason, true);
            }
            let existing = self.callbacks.with_read(&owner, &events, |read, _| {
                let constructor = registry.find_with(
                    scope,
                    || read.step(),
                    |_, row| {
                        registry_name_matches_with(row.name.code_units(), name.code_units(), || {
                            read.step()?;
                            read.step()
                        })
                    },
                )?;
                if let Some(constructor) = constructor {
                    return Ok(Some((constructor, true)));
                }
                let waiting = registry
                    .waiting
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let mut next = waiting.as_deref();
                while let Some(entry) = next {
                    read.step()?;
                    if registry_name_matches_with(
                        entry.name.code_units(),
                        name.code_units(),
                        || {
                            read.step()?;
                            read.step()
                        },
                    )? {
                        return Ok(Some((scope.resolver_promise(&entry.resolver)?, false)));
                    }
                    next = entry.next.as_deref();
                }
                Ok(None)
            })?;
            match existing {
                Some((constructor, true)) => {
                    return settled_definition_promise(scope, &owner, &events, &constructor, false);
                }
                Some((promise, false)) => return Ok(promise),
                None => (),
            }
            let mut waiters = registry
                .waiting
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let record = events.reserve(&owner, size_of::<DefinitionWaiter>())?;
            let resolver = new_definition_resolver(scope, &owner, &events)?;
            let promise = scope.resolver_promise(&resolver)?;
            let entry = Box::new(DefinitionWaiter {
                name,
                resolver,
                next: waiters.take(),
                _record: record,
                _name: name_backing,
            });
            *waiters = Some(entry);
            Ok(promise)
        })();
        match result {
            Ok(promise) => Ok(Completion::Return(promise)),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

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
            if !valid_registry_name(&events, &name)? {
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
                .map(|source| ObservedAttributes::copy(&owner, &events, source).map(Rc::new))
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
            let interface = match &local {
                Some((name, _)) => document::element::html_interface(name, || events.step())?,
                None => "HTMLElement",
            };
            let facts = HtmlCustomElementDefinition::new(
                name.clone(),
                local_name.clone(),
                read.form,
                read.disable_shadow,
            );
            let candidates = self.callbacks.with_read(&owner, &events, |read, _| {
                let document = read
                    .snapshot
                    .global_document_with(registry.global, &mut || read.step())?;
                let mut candidates = ReactionList::default();
                read.node(document.node())?
                    .visit_custom_element_upgrade_candidates_with(
                        (registry.id, &facts),
                        || read.step(),
                        |bytes| read.reserve(bytes),
                        |node| candidates.push(read, node),
                    )?;
                Ok(candidates)
            })?;
            let candidate_constructor = if candidates.values.is_empty() {
                None
            } else {
                Some((
                    scope.open_callback_object(&constructor)?,
                    constructor.associated_realm().identity(),
                ))
            };
            let mut reactions = if candidates.values.is_empty() {
                None
            } else {
                let backing = events.reserve(&owner, size_of::<PendingDefinitionReactions>())?;
                Some(Box::new(PendingDefinitionReactions {
                    candidates,
                    cursor: 0,
                    next: None,
                    _backing: backing,
                }))
            };
            let initial_attribute = callbacks[4].clone();
            let connected = callbacks[0].clone();
            let observed = observed_attributes.clone();
            let mut rows = registry
                .definitions
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let growth = rows.reserve(&owner, &events)?;
            let mut waiting = registry
                .waiting
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let waiter = matching_definition_waiter(&mut waiting, &events, &name)?;
            let definition_value = if waiter.is_some() {
                Some(scope.open_callback_object(&constructor)?)
            } else {
                None
            };
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
                    interface,
                    customized: local.is_some(),
                    id,
                    name,
                    constructor,
                    callbacks,
                    observed: observed_attributes,
                    _disable_internals: read.disable_internals,
                    _name_backing: name_backing,
                },
            );
            let resolver = waiter.and_then(|link| {
                link.take().map(|entry| {
                    let DefinitionWaiter { resolver, next, .. } = *entry;
                    *link = next;
                    resolver
                })
            });
            drop(rows);
            drop(waiting);
            let drive = (|| {
                if let Some((resolver, constructor)) = resolver.zip(definition_value) {
                    scope.resolve_promise(resolver, &constructor)?;
                }
                if let Some(((constructor, realm), reactions)) =
                    candidate_constructor.zip(reactions.as_mut())
                {
                    while let Some(&node) = reactions.candidates.values.get(reactions.cursor) {
                        self.invoke_upgrade_candidate(
                            scope,
                            (&owner, &events, public),
                            node,
                            UpgradeDefinition {
                                id,
                                global: registry.global,
                                constructor: &constructor,
                                interface,
                                realm,
                                initial_attribute: initial_attribute.as_ref(),
                                connected: connected.as_ref(),
                                observed: observed.as_deref(),
                            },
                        )?;
                        reactions.cursor += 1;
                    }
                }
                Ok::<(), DocumentGuestEventError>(())
            })();
            if drive.is_err()
                && let Some(mut reactions) = reactions
            {
                let mut pending = owner
                    .pending_definition_reactions
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                reactions.next = pending.take();
                *pending = Some(reactions);
            }
            drive?;
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

    pub(super) fn drain_insertion_upgrades<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        operation: surgeist_dom::DomSourceOperationId,
    ) -> Result<(), DocumentGuestEventError> {
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let registration = owner
            .attachment
            .get()
            .and_then(|attachment| attachment.registration.upgrade())
            .ok_or(DocumentScriptNativeError::Revoked)?;
        loop {
            let inspections = registration.custom_element_insertion_segments();
            let work =
                u64::try_from(inspections).map_err(|_| DocumentGuestEventError::WorkLimit)?;
            events.charge_work(work)?;
            let Some(mut batch) =
                registration.take_custom_element_insertions(operation, inspections)?
            else {
                break;
            };
            while let Some(&fact) = batch.facts().first() {
                let (global, constructor, interface, initial_attribute, connected, observed) =
                    self.callbacks.with_read(owner, events, |read, _| {
                        public.with_registry(
                            read,
                            |registry| registry.id == fact.definition.registry(),
                            |registry| {
                                read.step()?;
                                let rows = registry
                                    .definitions
                                    .try_borrow()
                                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                                let row = rows
                                    .rows
                                    .get(fact.definition.ordinal())
                                    .filter(|row| row.id == fact.definition)
                                    .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                                Ok((
                                    registry.global,
                                    row.constructor.clone(),
                                    row.interface,
                                    row.callbacks[4].clone(),
                                    row.callbacks[0].clone(),
                                    row.observed.clone(),
                                ))
                            },
                        )
                    })?;
                let realm = constructor.associated_realm().identity();
                let constructor = scope.open_callback_object(&constructor)?;
                self.invoke_upgrade_candidate(
                    scope,
                    (owner, events, public),
                    fact.element,
                    UpgradeDefinition {
                        id: fact.definition,
                        global,
                        constructor: &constructor,
                        interface,
                        realm,
                        initial_attribute: initial_attribute.as_ref(),
                        connected: connected.as_ref(),
                        observed: observed.as_deref(),
                    },
                )?;
                batch.acknowledge_front();
            }
        }
        Ok(())
    }

    fn invoke_upgrade_candidate<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        (owner, events, public): (&NativeOwner, &GuestEvents, &PublicEventBindings),
        node: NodeId,
        UpgradeDefinition {
            id,
            global,
            constructor,
            interface,
            realm,
            initial_attribute,
            connected,
            observed,
        }: UpgradeDefinition<'_, 'scope>,
    ) -> Result<(), DocumentGuestEventError> {
        let initial = self.callbacks.with_read(owner, events, |read, _| {
            let node = match read.node(node) {
                Ok(node) => node,
                Err(DocumentGuestEventError::Document(DocumentScriptError::Native(
                    DocumentScriptNativeError::Dom(surgeist_dom::DomError::StaleNode),
                ))) => return Ok(None),
                Err(error) => return Err(error),
            };
            let Some(element) = node.as_element() else {
                return Ok(None);
            };
            if element.custom_element_state() != surgeist_dom::HtmlCustomElementState::Undefined {
                return Ok(None);
            }
            let mut attributes = ReactionList::default();
            if let (Some(observed), Some(_)) = (observed, initial_attribute)
                && let Some(map) = node.attributes()
            {
                map.visit_with(
                    || read.step(),
                    |attribute| {
                        let name = attribute.name();
                        for (expected, _) in &observed.values {
                            read.step()?;
                            if registry_name_matches_with(
                                name.local_name().code_units(),
                                expected.code_units(),
                                || read.step(),
                            )? {
                                let local = self
                                    .callbacks
                                    .conversion
                                    .run(scope, |cx| cx.copy_to_js(name.local_name().code_units()))
                                    .map_err(Error::from)?;
                                let value = self
                                    .callbacks
                                    .conversion
                                    .run(scope, |cx| cx.copy_to_js(attribute.value().code_units()))
                                    .map_err(Error::from)?;
                                let namespace = match name.namespace_uri() {
                                    Some(uri) => self
                                        .callbacks
                                        .conversion
                                        .run(scope, |cx| cx.copy_to_js(uri.code_units()))
                                        .map_err(Error::from)?,
                                    None => scope.primitive(Primitive::Null)?,
                                };
                                let old = scope.primitive(Primitive::Null)?;
                                let root_bytes = events
                                    .exception_root_bytes
                                    .checked_add(size_of::<GuestBacking>())
                                    .ok_or(DocumentScriptNativeError::StorageLimit)?;
                                let arguments = [local, old, value, namespace];
                                for argument in &arguments {
                                    scope.retain_invocation_result_backing(
                                        argument,
                                        Box::new(read.reserve(root_bytes)?),
                                    )?;
                                }
                                attributes.push(read, arguments)?;
                                break;
                            }
                        }
                        Ok(())
                    },
                )?;
            }
            let is_connected = connected.is_some()
                && matches!(
                    node.root_with(true, || read.step())?.kind(),
                    DomNodeKind::Document(_)
                );
            Ok(Some((attributes, is_connected)))
        })?;
        let Some((attributes, is_connected)) = initial else {
            return Ok(());
        };
        let receiver = public.project_node(scope, owner, events, node)?;
        if public.document.ordinary.upgrade_existing(
            scope,
            (owner, events),
            (node, id, global),
            (constructor, interface, realm),
            &receiver,
        )? {
            if let Some(callback) = initial_attribute {
                for arguments in &attributes.values {
                    invoke_initial_reaction(
                        scope,
                        (owner, events),
                        (id, node, DocumentGuestUpgradeCallback::AttributeChanged),
                        callback,
                        &receiver,
                        arguments,
                    )?;
                }
            }
            if let Some(callback) = connected.filter(|_| is_connected) {
                invoke_initial_reaction(
                    scope,
                    (owner, events),
                    (id, node, DocumentGuestUpgradeCallback::Connected),
                    callback,
                    &receiver,
                    &[],
                )?;
            }
        }
        Ok(())
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

struct UpgradeDefinition<'a, 'scope> {
    id: DomRegistryDefinitionId,
    global: DomHostedGlobalId,
    constructor: &'a Value<'scope>,
    interface: &'static str,
    realm: RealmId,
    initial_attribute: Option<&'a RetainedCallbackObject>,
    connected: Option<&'a RetainedCallbackObject>,
    observed: Option<&'a ObservedAttributes>,
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
        matches: impl FnMut(&mut Scope<'scope>, &Definition) -> Result<bool, DocumentGuestEventError>,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        self.find_with(scope, || events.step(), matches)
    }

    fn find_with<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
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
            step()?;
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

pub(in crate::document_scripting) struct PendingDefinitionReactions {
    candidates: ReactionList<NodeId>,
    cursor: usize,
    next: Option<Box<Self>>,
    _backing: GuestBacking,
}

impl Drop for PendingDefinitionReactions {
    fn drop(&mut self) {
        let mut next = self.next.take();
        while let Some(mut item) = next {
            next = item.next.take();
        }
    }
}

impl NativeOwner {
    pub(in crate::document_scripting) fn discard_stopped_definition_reactions(
        &self,
    ) -> Result<usize, DocumentScriptNativeError> {
        let mut pending = self
            .pending_definition_reactions
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut discarded = 0usize;
        let mut next = pending.as_deref();
        while let Some(item) = next {
            discarded = discarded
                .checked_add(item.candidates.values.len() - item.cursor)
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
            next = item.next.as_deref();
        }
        pending.take();
        Ok(discarded)
    }
}

struct ReactionList<T> {
    values: Vec<T>,
    backing: Option<GuestBacking>,
}

impl<T> Default for ReactionList<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            backing: None,
        }
    }
}

impl<T> ReactionList<T> {
    fn push(&mut self, read: &NativeRead<'_>, value: T) -> Result<(), DocumentGuestEventError> {
        read.step()?;
        if self.values.len() == self.values.capacity() {
            let capacity = self
                .values
                .capacity()
                .checked_mul(2)
                .map(|capacity| capacity.max(1))
                .ok_or(DocumentScriptNativeError::StorageLimit)?;
            let bytes = capacity
                .checked_mul(size_of::<T>())
                .ok_or(DocumentScriptNativeError::StorageLimit)?;
            let backing = read.reserve(bytes)?;
            let mut next = Vec::new();
            next.try_reserve_exact(capacity)
                .map_err(|_| Error::Allocation)?;
            if next.capacity() > capacity {
                return Err(DocumentScriptNativeError::StorageLimit.into());
            }
            for _ in &self.values {
                read.step()?;
            }
            next.append(&mut self.values);
            self.values = next;
            self.backing = Some(backing);
        }
        self.values.push(value);
        Ok(())
    }
}

fn invoke_initial_reaction<'scope>(
    scope: &mut Scope<'scope>,
    (owner, events): (&NativeOwner, &GuestEvents),
    (definition, node, kind): (
        DomRegistryDefinitionId,
        NodeId,
        DocumentGuestUpgradeCallback,
    ),
    callback: &RetainedCallbackObject,
    receiver: &Value<'scope>,
    arguments: &[Value<'scope>],
) -> Result<(), DocumentGuestEventError> {
    let mut report = events.reports.reserve_constructor(events, owner)?;
    let realm = callback.associated_realm().identity();
    let callback = scope.open_callback_object(callback)?;
    let _arguments = events.reserve(owner, scope.argument_copy_storage_bytes(arguments.len())?)?;
    match scope.call(&callback, receiver, arguments) {
        Ok(_) => Ok(()),
        Err(Error::Exception(exception)) => {
            report.prepare(scope, &exception)?;
            report.publish_upgrade(DocumentGuestUpgradeException {
                receiver: node,
                callback: kind,
                definition,
                realm,
                exception,
            });
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn matching_definition_waiter<'a>(
    mut link: &'a mut Option<Box<DefinitionWaiter>>,
    events: &GuestEvents,
    name: &DocumentString,
) -> Result<Option<&'a mut Option<Box<DefinitionWaiter>>>, DocumentGuestEventError> {
    while let Some(entry) = link.as_ref() {
        events.step()?;
        if registry_name_matches(events, entry.name.code_units(), name.code_units())? {
            return Ok(Some(link));
        }
        link = &mut link
            .as_mut()
            .ok_or(DocumentScriptNativeError::ProjectionCustody)?
            .next;
    }
    Ok(None)
}

fn new_definition_resolver(
    scope: &mut Scope<'_>,
    owner: &NativeOwner,
    events: &GuestEvents,
) -> Result<PromiseResolver, DocumentGuestEventError> {
    let bytes = events
        .exception_root_bytes
        .checked_add(size_of::<GuestBacking>())
        .ok_or(DocumentScriptNativeError::StorageLimit)?;
    Ok(scope.promise_resolver(Box::new(events.reserve(owner, bytes)?))?)
}

fn settled_definition_promise<'scope>(
    scope: &mut Scope<'scope>,
    owner: &NativeOwner,
    events: &GuestEvents,
    value: &Value<'scope>,
    reject: bool,
) -> Result<Value<'scope>, DocumentGuestEventError> {
    let resolver = new_definition_resolver(scope, owner, events)?;
    let promise = scope.resolver_promise(&resolver)?;
    if reject {
        scope.reject_promise(resolver, value)?;
    } else {
        scope.resolve_promise(resolver, value)?;
    }
    Ok(promise)
}

fn valid_registry_name(
    events: &GuestEvents,
    name: &DocumentString,
) -> Result<bool, DocumentGuestEventError> {
    Ok(
        surgeist_html::bounded::lexical::validate_custom_element_name_with(name, |action| {
            use surgeist_html::bounded::lexical::HtmlCustomNameScanAction;
            events.charge_work(match action {
                HtmlCustomNameScanAction::ReservedPair => 2,
                _ => 1,
            })
        })?
        .is_ok(),
    )
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
    registry_name_matches_with(left, right, || events.charge_work(2))
}

fn registry_name_matches_with(
    left: &[u16],
    right: &[u16],
    mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
) -> Result<bool, DocumentGuestEventError> {
    if left.len() != right.len() {
        return Ok(false);
    }
    for index in 0..left.len() {
        step()?;
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
