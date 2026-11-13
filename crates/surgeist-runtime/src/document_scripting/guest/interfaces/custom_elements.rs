//! Guest identity for the canonical custom element registry.

use super::*;
use std::collections::LinkedList;
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

impl PublicEventBindings {
    pub(super) fn has_disconnected_callback(
        &self,
        events: &GuestEvents,
        definition: DomRegistryDefinitionId,
    ) -> Result<bool, DocumentGuestEventError> {
        self.with_registry_step(
            || events.step(),
            |registry| registry.id == definition.registry(),
            |registry| {
                events.step()?;
                let definitions = registry
                    .definitions
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let row = definitions
                    .rows
                    .get(definition.ordinal())
                    .filter(|row| row.id == definition)
                    .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
                Ok(row.callbacks[1].is_some())
            },
        )
    }

    pub(super) fn schedule_backup_reactions(&self, scope: &mut Scope<'_>) -> Result<(), Error> {
        if !self.backup_reactions_scheduled.get() {
            let job = scope.open_callback_object(&self.backup_reaction_job)?;
            scope.enqueue_microtask(&job)?;
            self.backup_reactions_scheduled.set(true);
        }
        Ok(())
    }
}

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
    install_definition_methods(scope, &prototype, capture)?;
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
            if read.form {
                return Ok(registry_not_supported(scope)?);
            }
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
            let mut reactions = ReactionInvocation::new(&owner)?;
            for node in candidates.values.iter().copied() {
                reactions.stage_definition_candidate(&events, node)?;
            }
            drop(candidates);
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
            reactions.publish_definition_candidates(id);
            if let Some((resolver, constructor)) = resolver.zip(definition_value) {
                scope.resolve_promise(resolver, &constructor)?;
            }
            reactions.drain(self, scope, &events)?;
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

    pub(super) fn drain_native_reactions<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        operation: surgeist_dom::DomSourceOperationId,
    ) -> Result<(), DocumentGuestEventError> {
        self.drain_reactions(scope, owner, events, Some(operation))
            .map(drop)
    }

    pub(super) fn run_backup_reactions<'scope>(
        &self,
        scope: &mut Scope<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let result = (|| {
            while self.drain_reactions(scope, &owner, &events, None)? {}
            Ok(())
        })();
        public.backup_reactions_scheduled.set(false);
        match result {
            Ok(()) => Ok(Completion::Undefined),
            Err(DocumentGuestEventError::Document(DocumentScriptError::Script(error))) => {
                Err(error)
            }
            Err(error) => refuse(scope, &events.refusal, error),
        }
    }

    fn drain_reactions<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        operation: Option<surgeist_dom::DomSourceOperationId>,
    ) -> Result<bool, DocumentGuestEventError> {
        let registration = owner
            .attachment
            .get()
            .and_then(|attachment| attachment.registration.upgrade())
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let mut invocation = ReactionInvocation::new(owner)?;
        let mut received = false;
        let result = (|| {
            loop {
                let inspections = registration.custom_element_reaction_segments();
                let work =
                    u64::try_from(inspections).map_err(|_| DocumentGuestEventError::WorkLimit)?;
                events.charge_work(work)?;
                let batch = match operation {
                    Some(operation) => {
                        registration.take_custom_element_reactions(operation, inspections)?
                    }
                    None => registration.take_backup_custom_element_reactions(inspections)?,
                };
                let Some(mut batch) = batch else {
                    break;
                };
                received = true;
                while let Some(fact) = batch.front() {
                    let reaction = if matches!(
                        fact.kind,
                        surgeist_dom::DomCustomElementReactionKind::Upgrade
                    ) {
                        Some(QueuedReaction::Upgrade {
                            definition: Some(fact.definition),
                            from_definition: false,
                            _backing: events.reserve(owner, QueuedReaction::storage_bytes())?,
                        })
                    } else {
                        self.prepare_native_callback(scope, owner, events, fact)?
                            .map(QueuedReaction::Callback)
                    };
                    if let Some(reaction) = reaction {
                        let queue = invocation.queue(events, fact.element)?;
                        if operation.is_some()
                            && events
                                .public
                                .as_ref()
                                .is_some_and(|public| public.backup_reactions_scheduled.get())
                        {
                            self.take_older_backup_reactions(
                                scope,
                                owner,
                                events,
                                &registration,
                                &queue,
                            )?;
                        }
                        queue
                            .reactions
                            .try_borrow_mut()
                            .map_err(|_| DocumentScriptNativeError::Busy)?
                            .push_back(reaction);
                    }
                    batch.acknowledge_front();
                }
            }
            invocation.drain(self, scope, events)?;
            Ok(received)
        })();
        drop(invocation);
        result
    }

    fn take_older_backup_reactions<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        registration: &DomDocumentScriptRegistration,
        queue: &ActiveReactionQueue,
    ) -> Result<(), DocumentGuestEventError> {
        loop {
            let available = events.limits.work.saturating_sub(events.budget.work.get());
            let mut remaining = available;
            let backup = registration
                .take_backup_custom_element_reaction_for(queue.receiver, &mut remaining);
            events.charge_work(available - remaining)?;
            let Some(mut backup) = backup.map_err(|error| match error {
                surgeist_dom::DomError::SourceOutputLimit => DocumentGuestEventError::WorkLimit,
                other => DocumentGuestEventError::from(other),
            })?
            else {
                return Ok(());
            };
            let fact = backup
                .front()
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            if let Some(callback) = self.prepare_native_callback(scope, owner, events, fact)? {
                queue
                    .reactions
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?
                    .push_back(QueuedReaction::Callback(callback));
            }
            backup.acknowledge_front();
        }
    }

    fn invoke_native_upgrade<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        element: NodeId,
        definition: DomRegistryDefinitionId,
        accepted: &mut bool,
    ) -> Result<Option<Rc<ActiveReactionQueue>>, DocumentGuestEventError> {
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let (global, constructor, interface, initial_attribute, connected, observed) =
            self.callbacks.with_read(owner, events, |read, _| {
                public.with_registry(
                    read,
                    |registry| registry.id == definition.registry(),
                    |registry| {
                        read.step()?;
                        let rows = registry
                            .definitions
                            .try_borrow()
                            .map_err(|_| DocumentScriptNativeError::Busy)?;
                        let row = rows
                            .rows
                            .get(definition.ordinal())
                            .filter(|row| row.id == definition)
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
            element,
            UpgradeDefinition {
                id: definition,
                global,
                constructor: &constructor,
                interface,
                realm,
                initial_attribute: initial_attribute.as_ref(),
                connected: connected.as_ref(),
                observed: observed.as_deref(),
            },
            accepted,
        )
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
        accepted: &mut bool,
    ) -> Result<Option<Rc<ActiveReactionQueue>>, DocumentGuestEventError> {
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
            return Ok(None);
        };
        let receiver = public.project_node(scope, owner, events, node)?;
        let mut callbacks = LinkedList::new();
        if let Some(callback) = initial_attribute {
            for arguments in &attributes.values {
                callbacks.push_back(QueuedReaction::Callback(CallbackReaction::prepare(
                    scope,
                    (owner, events),
                    id,
                    (
                        DocumentGuestCustomElementCallback::AttributeChanged,
                        callback,
                        None,
                    ),
                    &receiver,
                    Some(ReactionArguments::Attribute(arguments.each_ref())),
                )?));
            }
        }
        if let Some(callback) = connected.filter(|_| is_connected) {
            callbacks.push_back(QueuedReaction::Callback(CallbackReaction::prepare(
                scope,
                (owner, events),
                id,
                (
                    DocumentGuestCustomElementCallback::Connected,
                    callback,
                    None,
                ),
                &receiver,
                None,
            )?));
        }
        let queue = self.prepare_callback_queue(owner, events, node, callbacks)?;
        public.document.ordinary.upgrade_existing(
            scope,
            (owner, events),
            (node, id, global),
            (constructor, interface, realm),
            (&receiver, queue, accepted),
        )
    }

    fn prepare_callback_queue(
        &self,
        owner: &NativeOwner,
        events: &GuestEvents,
        receiver: NodeId,
        reactions: LinkedList<QueuedReaction>,
    ) -> Result<Option<Rc<ActiveReactionQueue>>, DocumentGuestEventError> {
        if reactions.is_empty() {
            return Ok(None);
        }
        Ok(Some(ActiveReactionQueue::new(
            owner, events, receiver, reactions,
        )?))
    }

    fn prepare_native_callback<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        fact: surgeist_dom::DomCustomElementReaction<'_>,
    ) -> Result<Option<CallbackReaction>, DocumentGuestEventError> {
        use surgeist_dom::DomCustomElementReactionKind as Kind;
        let (kind, callback_index) = match fact.kind {
            Kind::AttributeChanged { .. } => {
                (DocumentGuestCustomElementCallback::AttributeChanged, 4)
            }
            Kind::Connected => (DocumentGuestCustomElementCallback::Connected, 0),
            Kind::Disconnected => (DocumentGuestCustomElementCallback::Disconnected, 1),
            Kind::ConnectedMove => (DocumentGuestCustomElementCallback::ConnectedMove, 2),
            Kind::Adopted { .. } => (DocumentGuestCustomElementCallback::Adopted, 3),
            _ => return Err(DocumentScriptNativeError::ProjectionCustody.into()),
        };
        let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
        let callback = self.callbacks.with_read(owner, events, |read, _| {
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
                    let Some(callback) = &row.callbacks[callback_index] else {
                        let fallback = match (kind, &row.callbacks[1], &row.callbacks[0]) {
                            (
                                DocumentGuestCustomElementCallback::ConnectedMove,
                                Some(first),
                                second,
                            ) => Some((
                                DocumentGuestCustomElementCallback::Disconnected,
                                first.clone(),
                                second.clone(),
                            )),
                            (
                                DocumentGuestCustomElementCallback::ConnectedMove,
                                None,
                                Some(second),
                            ) => Some((
                                DocumentGuestCustomElementCallback::Connected,
                                second.clone(),
                                None,
                            )),
                            _ => None,
                        };
                        return Ok(fallback);
                    };
                    let Kind::AttributeChanged { local_name, .. } = fact.kind else {
                        return Ok(Some((kind, callback.clone(), None)));
                    };
                    let Some(observed) = &row.observed else {
                        return Ok(None);
                    };
                    for (expected, _) in &observed.values {
                        read.step()?;
                        if registry_name_matches_with(local_name, expected.code_units(), || {
                            read.step()
                        })? {
                            return Ok(Some((kind, callback.clone(), None)));
                        }
                    }
                    Ok(None)
                },
            )
        })?;
        let Some((kind, callback, continuation)) = callback else {
            return Ok(None);
        };
        let receiver = public.project_node(scope, owner, events, fact.element)?;
        let bytes = events
            .exception_root_bytes
            .checked_add(size_of::<GuestBacking>())
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        scope
            .retain_invocation_result_backing(&receiver, Box::new(events.reserve(owner, bytes)?))?;
        let mut convert = |argument: Option<&[u16]>| {
            let value = match argument {
                Some(text) => self
                    .callbacks
                    .conversion
                    .run(scope, |cx| cx.copy_to_js(text))
                    .map_err(Error::from)?,
                None => scope.primitive(Primitive::Null)?,
            };
            scope.retain_invocation_result_backing(
                &value,
                Box::new(events.reserve(owner, bytes)?),
            )?;
            Ok::<_, DocumentGuestEventError>(value)
        };
        let arguments = match fact.kind {
            Kind::AttributeChanged {
                local_name,
                old_value,
                new_value,
                namespace,
            } => Some(ReactionArguments::Attribute([
                convert(Some(local_name))?,
                convert(old_value)?,
                convert(new_value)?,
                convert(namespace)?,
            ])),
            Kind::Adopted {
                old_document,
                new_document,
            } => {
                let mut document = |id: DocumentId| {
                    let value = public.project_node(scope, owner, events, id.node())?;
                    scope.retain_invocation_result_backing(
                        &value,
                        Box::new(events.reserve(owner, bytes)?),
                    )?;
                    Ok::<_, DocumentGuestEventError>(value)
                };
                Some(ReactionArguments::Adoption([
                    document(old_document)?,
                    document(new_document)?,
                ]))
            }
            _ => None,
        };
        let arguments = arguments.as_ref().map(|arguments| match arguments {
            ReactionArguments::Attribute(values) => ReactionArguments::Attribute(values.each_ref()),
            ReactionArguments::Adoption(values) => ReactionArguments::Adoption(values.each_ref()),
        });
        Ok(Some(CallbackReaction::prepare(
            scope,
            (owner, events),
            fact.definition,
            (kind, &callback, continuation),
            &receiver,
            arguments,
        )?))
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

enum ReactionArguments<T> {
    Attribute([T; 4]),
    Adoption([T; 2]),
}

impl<T> ReactionArguments<T> {
    fn map<U, E>(
        &self,
        mut project: impl FnMut(&T) -> Result<U, E>,
    ) -> Result<ReactionArguments<U>, E> {
        Ok(match self {
            Self::Attribute([name, old, new, namespace]) => ReactionArguments::Attribute([
                project(name)?,
                project(old)?,
                project(new)?,
                project(namespace)?,
            ]),
            Self::Adoption([old, new]) => {
                ReactionArguments::Adoption([project(old)?, project(new)?])
            }
        })
    }

    fn as_slice(&self) -> &[T] {
        match self {
            Self::Attribute(values) => values,
            Self::Adoption(values) => values,
        }
    }
}

struct CallbackReaction {
    definition: DomRegistryDefinitionId,
    kind: DocumentGuestCustomElementCallback,
    callback: RetainedCallbackObject,
    connect_after_disconnect: Option<RetainedCallbackObject>,
    receiver: surgeist_bindings_js::Root,
    arguments: Option<ReactionArguments<surgeist_bindings_js::Root>>,
    _backing: GuestBacking,
}

impl CallbackReaction {
    fn prepare<'scope>(
        scope: &Scope<'scope>,
        (owner, events): (&NativeOwner, &GuestEvents),
        definition: DomRegistryDefinitionId,
        (kind, callback, connect_after_disconnect): (
            DocumentGuestCustomElementCallback,
            &RetainedCallbackObject,
            Option<RetainedCallbackObject>,
        ),
        receiver: &Value<'scope>,
        arguments: Option<ReactionArguments<&Value<'scope>>>,
    ) -> Result<Self, DocumentGuestEventError> {
        let backing = events.reserve(owner, QueuedReaction::storage_bytes())?;
        Ok(Self {
            definition,
            kind,
            callback: callback.clone(),
            connect_after_disconnect,
            receiver: scope.retain(receiver),
            arguments: arguments.map(|arguments| match arguments {
                ReactionArguments::Attribute(values) => {
                    ReactionArguments::Attribute(values.map(|value| scope.retain(value)))
                }
                ReactionArguments::Adoption(values) => {
                    ReactionArguments::Adoption(values.map(|value| scope.retain(value)))
                }
            }),
            _backing: backing,
        })
    }
}

enum QueuedReaction {
    Upgrade {
        definition: Option<DomRegistryDefinitionId>,
        from_definition: bool,
        _backing: GuestBacking,
    },
    Callback(CallbackReaction),
}

impl QueuedReaction {
    fn is_definition_upgrade(&self) -> bool {
        matches!(
            self,
            Self::Upgrade {
                from_definition: true,
                ..
            }
        )
    }

    const fn storage_bytes() -> usize {
        size_of::<Self>() + 2 * size_of::<usize>()
    }
}

struct InvocationEntry {
    queue: Rc<ActiveReactionQueue>,
    installed: bool,
    definition_reaction: LinkedList<QueuedReaction>,
    _backing: GuestBacking,
}

/// Makes all receivers of the native edit discoverable before its first
/// callback. A nested edit adds to the same FIFO and gets its own drain order.
struct ReactionInvocation<'a> {
    owner: &'a NativeOwner,
    original: Option<Rc<ActiveReactionQueue>>,
    entries: LinkedList<InvocationEntry>,
    completed: bool,
}

impl<'a> ReactionInvocation<'a> {
    fn new(owner: &'a NativeOwner) -> Result<Self, DocumentScriptNativeError> {
        let original = owner
            .active_reaction_queue
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .clone();
        Ok(Self {
            owner,
            original,
            entries: LinkedList::new(),
            completed: false,
        })
    }

    fn queue(
        &mut self,
        events: &GuestEvents,
        receiver: NodeId,
    ) -> Result<Rc<ActiveReactionQueue>, DocumentGuestEventError> {
        for entry in &self.entries {
            events.step()?;
            if entry.queue.receiver == receiver {
                return Ok(entry.queue.clone());
            }
        }
        Ok(self.add_receiver(events, receiver, false)?.queue.clone())
    }

    fn add_receiver(
        &mut self,
        events: &GuestEvents,
        receiver: NodeId,
        definition_candidates: bool,
    ) -> Result<&mut InvocationEntry, DocumentGuestEventError> {
        let mut active = self
            .owner
            .active_reaction_queue
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        // A definition visits each native candidate once. Search only queues
        // installed by an enclosing invocation during that unique traversal.
        let mut matching = if definition_candidates {
            self.original.clone()
        } else {
            active.clone()
        };
        while let Some(queue) = matching.as_ref() {
            events.step()?;
            if queue.receiver == receiver {
                break;
            }
            let previous = queue
                .previous
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?
                .clone();
            matching = previous;
        }
        let backing = events.reserve(
            self.owner,
            size_of::<InvocationEntry>() + 2 * size_of::<usize>(),
        )?;
        let installed = matching.is_none();
        let queue = match matching {
            Some(queue) => queue,
            None => {
                let queue =
                    ActiveReactionQueue::new(self.owner, events, receiver, LinkedList::new())?;
                queue.previous.replace(active.replace(queue.clone()));
                queue
            }
        };
        self.entries.push_back(InvocationEntry {
            queue: queue.clone(),
            installed,
            definition_reaction: LinkedList::new(),
            _backing: backing,
        });
        self.entries
            .back_mut()
            .ok_or(DocumentScriptNativeError::ProjectionCustody.into())
    }

    fn stage_definition_candidate(
        &mut self,
        events: &GuestEvents,
        receiver: NodeId,
    ) -> Result<(), DocumentGuestEventError> {
        let backing = events.reserve(self.owner, QueuedReaction::storage_bytes())?;
        let entry = self.add_receiver(events, receiver, true)?;
        entry
            .definition_reaction
            .push_back(QueuedReaction::Upgrade {
                definition: None,
                from_definition: true,
                _backing: backing,
            });
        Ok(())
    }

    fn publish_definition_candidates(&mut self, id: DomRegistryDefinitionId) {
        for entry in &mut self.entries {
            for reaction in &mut entry.definition_reaction {
                // The linked-list node is already admitted. Give it the exact
                // identity from canonical publication before any guest entry.
                if let QueuedReaction::Upgrade { definition, .. } = reaction {
                    *definition = Some(id);
                }
            }
            entry
                .queue
                .reactions
                .borrow_mut()
                .append(&mut entry.definition_reaction);
        }
    }

    fn drain<'scope>(
        &mut self,
        capture: &Capture,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
    ) -> Result<(), DocumentGuestEventError> {
        for entry in &self.entries {
            entry.queue.drain(capture, scope, self.owner, events)?;
        }
        self.completed = true;
        Ok(())
    }
}

impl Drop for ReactionInvocation<'_> {
    fn drop(&mut self) {
        self.owner
            .active_reaction_queue
            .replace(self.original.take());
        for entry in std::mem::take(&mut self.entries) {
            if entry.installed {
                entry.queue.previous.replace(None);
                if !self.completed {
                    self.owner.keep_stopped_reaction_queue(entry.queue);
                }
            }
        }
    }
}

pub(in crate::document_scripting) struct ActiveReactionQueue {
    receiver: NodeId,
    reactions: RefCell<LinkedList<QueuedReaction>>,
    stopped: Cell<bool>,
    previous: RefCell<Option<Rc<Self>>>,
    stopped_next: RefCell<Option<Rc<Self>>>,
    _backing: GuestBacking,
}

impl Drop for ActiveReactionQueue {
    fn drop(&mut self) {
        let mut next = self.stopped_next.get_mut().take();
        while let Some(mut queue) = next {
            let Some(queue) = Rc::get_mut(&mut queue) else {
                break;
            };
            next = queue.stopped_next.get_mut().take();
        }
    }
}

pub(in crate::document_scripting::guest::interfaces) struct ReactionQueueEntry<'a> {
    active: &'a RefCell<Option<Rc<ActiveReactionQueue>>>,
    queue: Rc<ActiveReactionQueue>,
}

impl Drop for ReactionQueueEntry<'_> {
    fn drop(&mut self) {
        self.active.replace(self.queue.previous.borrow_mut().take());
    }
}

struct ReactionEntry<'a> {
    queue: &'a RefCell<LinkedList<QueuedReaction>>,
    front: LinkedList<QueuedReaction>,
    accepted: bool,
}

impl Drop for ReactionEntry<'_> {
    fn drop(&mut self) {
        if !self.accepted {
            let mut queue = self.queue.borrow_mut();
            self.front.append(&mut queue);
            std::mem::swap(&mut self.front, &mut *queue);
        }
    }
}

impl ActiveReactionQueue {
    pub(super) fn enter<'a>(
        self: &Rc<Self>,
        owner: &'a NativeOwner,
    ) -> Result<ReactionQueueEntry<'a>, DocumentScriptNativeError> {
        let mut active = owner
            .active_reaction_queue
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        self.previous.replace(active.replace(self.clone()));
        Ok(ReactionQueueEntry {
            active: &owner.active_reaction_queue,
            queue: self.clone(),
        })
    }

    fn new(
        owner: &NativeOwner,
        events: &GuestEvents,
        receiver: NodeId,
        reactions: LinkedList<QueuedReaction>,
    ) -> Result<Rc<Self>, DocumentGuestEventError> {
        let backing = events.reserve(owner, size_of::<Self>() + 2 * size_of::<usize>())?;
        Ok(Rc::new(Self {
            receiver,
            reactions: RefCell::new(reactions),
            stopped: Cell::new(false),
            previous: RefCell::new(None),
            stopped_next: RefCell::new(None),
            _backing: backing,
        }))
    }

    fn drain<'scope>(
        &self,
        capture: &Capture,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
    ) -> Result<(), DocumentGuestEventError> {
        loop {
            let empty = self
                .reactions
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?
                .is_empty();
            if empty
                && events
                    .public
                    .as_ref()
                    .is_some_and(|public| public.backup_reactions_scheduled.get())
            {
                // A callback can append another backup occurrence for this
                // receiver. Finish its FIFO before advancing the element queue.
                let registration = owner
                    .attachment
                    .get()
                    .and_then(|attachment| attachment.registration.upgrade())
                    .ok_or(DocumentScriptNativeError::Revoked)?;
                capture.take_older_backup_reactions(scope, owner, events, &registration, self)?;
            }
            let upgrade = {
                let reactions = self
                    .reactions
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                match reactions.front() {
                    Some(QueuedReaction::Upgrade { definition, .. }) => {
                        Some(definition.ok_or(DocumentScriptNativeError::ProjectionCustody)?)
                    }
                    Some(QueuedReaction::Callback(_)) => None,
                    None => return Ok(()),
                }
            };
            if let Some(definition) = upgrade {
                let mut reactions = self
                    .reactions
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let tail = reactions.split_off(1);
                let front = std::mem::replace(&mut *reactions, tail);
                drop(reactions);
                let mut entry = ReactionEntry {
                    queue: &self.reactions,
                    front,
                    accepted: false,
                };
                let result = capture.invoke_native_upgrade(
                    scope,
                    owner,
                    events,
                    self.receiver,
                    definition,
                    &mut entry.accepted,
                );
                let initial = result?;
                entry.accepted = true;
                if let Some(initial) = initial {
                    let mut reactions = self
                        .reactions
                        .try_borrow_mut()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    let mut initial = initial
                        .reactions
                        .try_borrow_mut()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    initial.append(&mut reactions);
                    std::mem::swap(&mut *initial, &mut *reactions);
                }
                continue;
            }
            let next = {
                let reactions = self
                    .reactions
                    .try_borrow()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let Some(QueuedReaction::Callback(next)) = reactions.front() else {
                    return Err(DocumentScriptNativeError::ProjectionCustody.into());
                };
                let receiver = scope.import(&next.receiver)?;
                let callback = scope.open_callback_object(&next.callback)?;
                let arguments = next
                    .arguments
                    .as_ref()
                    .map(|arguments| arguments.map(|argument| scope.import(argument)))
                    .transpose()?;
                let connected = next
                    .connect_after_disconnect
                    .as_ref()
                    .map(|callback| scope.open_callback_object(callback))
                    .transpose()?;
                (
                    receiver,
                    callback,
                    arguments,
                    connected,
                    next.kind,
                    next.definition,
                    next.callback.associated_realm().identity(),
                )
            };
            let (receiver, callback, arguments, connected, mut kind, definition, mut realm) = next;
            let arguments = arguments
                .as_ref()
                .map_or(&[][..], ReactionArguments::as_slice);
            let mut report = events.reports.reserve_constructor(events, owner)?;
            let _arguments =
                events.reserve(owner, scope.argument_copy_storage_bytes(arguments.len())?)?;
            let mut entered = LinkedList::new();
            let mut result = scope.call_with_entry(&callback, &receiver, arguments, || {
                let mut queue = self
                    .reactions
                    .try_borrow_mut()
                    .map_err(|_| Error::RegistryConflict)?;
                if !matches!(queue.front(), Some(QueuedReaction::Callback(_))) {
                    return Err(Error::RegistryConflict);
                }
                let tail = queue.split_off(1);
                entered = std::mem::replace(&mut *queue, tail);
                Ok(())
            });
            if result.is_ok()
                && let Some(connected) = connected
                && let Some(QueuedReaction::Callback(reaction)) = entered.front_mut()
                && let Some(callback) = reaction.connect_after_disconnect.take()
            {
                kind = DocumentGuestCustomElementCallback::Connected;
                realm = callback.associated_realm().identity();
                reaction.kind = kind;
                reaction.callback = callback;
                let mut continuation = ReactionEntry {
                    queue: &self.reactions,
                    front: entered,
                    accepted: false,
                };
                result = scope.call_with_entry(&connected, &receiver, &[], || {
                    continuation.accepted = true;
                    Ok(())
                });
            }
            match result {
                Ok(_) => (),
                Err(Error::Exception(exception)) => {
                    report.prepare(scope, &exception)?;
                    report.publish_custom_element_callback(
                        DocumentGuestCustomElementCallbackException {
                            receiver: self.receiver,
                            callback: kind,
                            definition,
                            realm,
                            exception,
                        },
                    );
                }
                Err(error) => return Err(error.into()),
            }
        }
    }
}

impl NativeOwner {
    pub(in crate::document_scripting) fn keep_stopped_reaction_queue(
        &self,
        queue: Rc<ActiveReactionQueue>,
    ) {
        // No callback is running with a queue borrow. Stopped custody is only
        // accessed synchronously by the owning host after engine entry.
        if !queue.reactions.borrow().is_empty() && !queue.stopped.replace(true) {
            let mut stopped = self.stopped_reaction_queues.borrow_mut();
            queue.stopped_next.replace(stopped.replace(queue.clone()));
        }
    }

    pub(in crate::document_scripting) fn discard_stopped_queued_reactions(
        &self,
        from_definition: bool,
    ) -> Result<usize, DocumentScriptNativeError> {
        let mut stopped = self
            .stopped_reaction_queues
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut count = 0usize;
        let mut next = stopped.clone();
        while let Some(queue) = next {
            count = count
                .checked_add(
                    queue
                        .reactions
                        .borrow()
                        .iter()
                        .filter(|reaction| reaction.is_definition_upgrade() == from_definition)
                        .count(),
                )
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
            next = queue.stopped_next.borrow().clone();
        }
        let mut link = stopped.take();
        let mut retained = None;
        while let Some(queue) = link {
            link = queue.stopped_next.borrow_mut().take();
            let mut reactions = queue.reactions.borrow_mut();
            let mut remaining = std::mem::take(&mut *reactions);
            while let Some(front) = remaining.front() {
                let discard = front.is_definition_upgrade() == from_definition;
                let mut tail = remaining.split_off(1);
                if !discard {
                    reactions.append(&mut remaining);
                }
                std::mem::swap(&mut remaining, &mut tail);
            }
            if !reactions.is_empty() {
                queue.stopped_next.replace(retained.replace(queue.clone()));
            }
        }
        *stopped = retained;
        Ok(count)
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
        matches: impl FnMut(&GlobalRegistry) -> bool,
        use_registry: impl FnOnce(&Rc<GlobalRegistry>) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        self.with_registry_step(|| read.step(), matches, use_registry)
    }

    fn with_registry_step<T>(
        &self,
        mut step: impl FnMut() -> Result<(), DocumentGuestEventError>,
        mut matches: impl FnMut(&GlobalRegistry) -> bool,
        use_registry: impl FnOnce(&Rc<GlobalRegistry>) -> Result<T, DocumentGuestEventError>,
    ) -> Result<T, DocumentGuestEventError> {
        step()?;
        if matches(&self.custom_elements) {
            return use_registry(&self.custom_elements);
        }
        let origins = self
            .origins
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut next = origins.as_deref();
        while let Some(origin) = next {
            step()?;
            if matches(&origin.custom_elements) {
                return use_registry(&origin.custom_elements);
            }
            next = origin.next.as_deref();
        }
        Err(DocumentScriptNativeError::CreationGlobalUnavailable.into())
    }
}
