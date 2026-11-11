//! Canonical ordinary DocumentFragment creation and projection custody.

use super::*;
use surgeist_dom::DomNodeRetirementWatch;

const FRAGMENT: &[u16] = &ascii(b"DocumentFragment");
const CREATE_FRAGMENT: &[u16] = &ascii(b"createDocumentFragment");

pub(super) struct FragmentBindings {
    prototype: RetainedCallbackObject,
    inbox: RefCell<DomNodeRetirementInbox>,
    pins: RefCell<OrdinaryPins>,
}

#[derive(Default)]
struct OrdinaryPins {
    slots: Vec<Slot>,
    free: Option<usize>,
    slot_backing: Option<GuestBacking>,
}

impl Drop for OrdinaryPins {
    fn drop(&mut self) {
        for slot in self.slots.drain(..) {
            if let Slot::Retained(pin) = slot {
                let released = {
                    let allocation = pin;
                    *allocation
                };
                drop(released);
            }
        }
    }
}

enum Slot {
    Free(Option<usize>),
    Retained(Box<Pin>),
}

struct Pin {
    holder: RetainedCallbackObject,
    live: Option<LivePin>,
    lifetime: Rc<FragmentLifetime>,
    _backing: GuestBacking,
}

pub(in crate::document_scripting::guest) struct FragmentLifetime {
    retired: Cell<bool>,
    _backing: GuestBacking,
}

impl FragmentLifetime {
    pub(in crate::document_scripting::guest) fn is_retired(&self) -> bool {
        self.retired.get()
    }
}

struct LivePin {
    node: NodeId,
    _watch: DomNodeRetirementWatch,
}

enum ReservedSlot<'a> {
    Append(&'a mut Vec<Slot>),
    Grow {
        slots: &'a mut Vec<Slot>,
        old_backing: &'a mut Option<GuestBacking>,
        replacement: Vec<Slot>,
        backing: GuestBacking,
    },
    Reuse {
        slot: &'a mut Slot,
        free: &'a mut Option<usize>,
        next: Option<usize>,
    },
}

impl ReservedSlot<'_> {
    fn publish(self, pin: Box<Pin>) {
        match self {
            Self::Append(slots) => slots.push(Slot::Retained(pin)),
            Self::Grow {
                slots,
                old_backing,
                mut replacement,
                backing,
            } => {
                replacement.append(slots);
                replacement.push(Slot::Retained(pin));
                drop(std::mem::replace(slots, replacement));
                *old_backing = Some(backing);
            }
            Self::Reuse { slot, free, next } => {
                *slot = Slot::Retained(pin);
                *free = next;
            }
        }
    }
}

impl OrdinaryPins {
    fn reserve<'a>(
        &'a mut self,
        owner: &NativeOwner,
        events: &GuestEvents,
    ) -> Result<(usize, ReservedSlot<'a>), DocumentGuestEventError> {
        events.step()?;
        if let Some(key) = self.free {
            let slot = self
                .slots
                .get_mut(key)
                .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
            let Slot::Free(next) = slot else {
                return Err(DocumentScriptNativeError::ProjectionCustody.into());
            };
            let next = *next;
            return Ok((
                key,
                ReservedSlot::Reuse {
                    slot,
                    free: &mut self.free,
                    next,
                },
            ));
        }
        let key = self.slots.len();
        if key == self.slots.capacity() {
            let capacity = key
                .max(2)
                .checked_mul(2)
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
            let bytes = capacity
                .checked_mul(size_of::<Slot>())
                .ok_or(DocumentScriptNativeError::StorageLimit)?;
            let backing = events.reserve(owner, bytes)?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(capacity)
                .map_err(|_| Error::Allocation)?;
            if replacement.capacity() > capacity {
                return Err(DocumentScriptNativeError::StorageLimit.into());
            }
            for _ in &self.slots {
                events.step()?;
            }
            return Ok((
                key,
                ReservedSlot::Grow {
                    slots: &mut self.slots,
                    old_backing: &mut self.slot_backing,
                    replacement,
                    backing,
                },
            ));
        }
        Ok((key, ReservedSlot::Append(&mut self.slots)))
    }
}

impl FragmentBindings {
    pub(super) fn install<'scope>(
        scope: &mut Scope<'scope>,
        node: &Value<'scope>,
        node_constructor: &Value<'scope>,
        inbox: DomNodeRetirementInbox,
        capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
        fund: &impl Fn(usize) -> Result<GuestBacking, DocumentGuestEventError>,
    ) -> Result<Self, DocumentGuestEventError> {
        let prototype = scope.object(ObjectPrototype::Object(node))?;
        let fallback = capture(size_of::<Capture>())?;
        let constructor = capture(size_of::<Capture>())?;
        let callback = ConstructorCallback::bounded(
            0,
            move |scope| {
                let (_, events) = fallback.active(scope)?;
                let public = events.public.as_ref().ok_or(Error::ExpiredIdentity)?;
                scope.open_callback_object(&public.document.fragment.prototype)
            },
            move |scope, arguments| constructor.construct_fragment(scope, arguments),
        );
        let interface = scope.constructor(
            &callback,
            FunctionMetadata {
                name: FRAGMENT,
                length: 0,
            },
        )?;
        scope.set_prototype(&interface, ObjectPrototype::Object(node_constructor))?;
        scope.define_property(
            &interface,
            PropertyKey::String(PROTOTYPE),
            PropertyDescriptor::Data {
                value: &prototype,
                writable: false,
                enumerable: false,
                configurable: false,
            },
        )?;
        let global = scope.global()?;
        install_interface(scope, &global, FRAGMENT, &interface, &prototype)?;
        let observed = scope.callback_object(&prototype)?;
        Ok(Self {
            prototype: scope.retain_callback_object(observed, fund)?,
            inbox: RefCell::new(inbox),
            pins: RefCell::new(OrdinaryPins::default()),
        })
    }

    pub(super) fn open<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        node: NodeId,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let pins = self
            .pins
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        for slot in &pins.slots {
            events.step()?;
            if let Slot::Retained(pin) = slot
                && pin.live.as_ref().is_some_and(|live| live.node == node)
            {
                return Ok(scope.open_callback_object(&pin.holder)?);
            }
        }
        Err(DocumentScriptNativeError::ProjectionCustody.into())
    }

    pub(super) fn reap(&self, process: &mut Process, retired: &Cell<bool>) {
        let (Ok(mut inbox), Ok(mut pins)) =
            (self.inbox.try_borrow_mut(), self.pins.try_borrow_mut())
        else {
            return;
        };
        while let Some(notice) = inbox.take() {
            let key = notice.custody_key();
            let next = pins.free;
            let Some(slot) = pins.slots.get_mut(key) else {
                continue;
            };
            if !matches!(slot, Slot::Retained(pin)
                if pin.live.as_ref().is_some_and(|live| live.node == notice.node()))
            {
                continue;
            }
            if let Slot::Retained(pin) = std::mem::replace(slot, Slot::Free(next)) {
                pins.free = Some(key);
                pin.lifetime.retired.set(true);
                retired.set(true);
                let Pin {
                    holder,
                    live,
                    lifetime,
                    _backing: backing,
                } = {
                    let allocation = pin;
                    *allocation
                };
                let _ = process.release_callback_object(holder);
                drop(live);
                drop(lifetime);
                drop(backing);
            }
        }
    }
}

pub(super) fn install_factory<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: &impl Fn(usize) -> Result<Capture, DocumentGuestEventError>,
) -> Result<(), DocumentGuestEventError> {
    let factory = capture(size_of::<Capture>())?;
    let function = scope.fixed_arity_function(
        &Callback::new(move |scope, arguments| factory.create_fragment(scope, arguments)),
        FunctionMetadata {
            name: CREATE_FRAGMENT,
            length: 0,
        },
    )?;
    define_data(
        scope,
        prototype,
        PropertyKey::String(CREATE_FRAGMENT),
        &function,
        true,
        true,
        true,
    )?;
    Ok(())
}

impl Capture {
    fn construct_fragment<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        let (owner, events) = self.active(scope)?;
        let result = (|| -> Result<_, DocumentGuestEventError> {
            let selected = arguments.select_prototype(scope)?;
            scope.check_active()?;
            let attachment = owner
                .attachment
                .get()
                .ok_or(DocumentScriptNativeError::Revoked)?;
            self.publish_fragment(
                scope,
                &owner,
                &events,
                attachment.target.document(),
                selected.prototype(),
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

    fn create_fragment<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let projection = scope.native_object::<GuestNodeProjection>(arguments.receiver())?;
        let Some(GuestNode::Document(document)) = projection.and_then(|node| node.payload().id())
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
            let prototype = scope.open_callback_object(&bindings.fragment.prototype)?;
            self.publish_fragment(scope, &owner, &events, document, &prototype)
        })();
        complete(scope, &events, result)
    }

    fn publish_fragment<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        document: DocumentId,
        prototype: &Value<'scope>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let bindings = &events
            .public
            .as_ref()
            .ok_or(Error::ExpiredIdentity)?
            .document
            .fragment;
        let mut pins = bindings
            .pins
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let (key, reserved) = pins.reserve(owner, events)?;
        let inbox = bindings
            .inbox
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let backing = events.reserve(owner, size_of::<Pin>())?;
        let lifetime_backing = events.reserve(
            owner,
            size_of::<FragmentLifetime>() + 2 * size_of::<usize>(),
        )?;
        let lifetime = Rc::new(FragmentLifetime {
            retired: Cell::new(false),
            _backing: lifetime_backing,
        });
        let holder = events.project(
            owner,
            scope,
            ObjectPrototype::Object(prototype),
            GuestNodeProjection {
                node: Cell::new(None),
                fragment_lifetime: Some(lifetime.clone()),
            },
        )?;
        let value = scope.open_callback_object(&holder)?;
        let projection = scope
            .native_object::<GuestNodeProjection>(&value)?
            .ok_or(DocumentScriptNativeError::ProjectionCustody)?;
        let mut pin = Box::new(Pin {
            holder,
            live: None,
            lifetime,
            _backing: backing,
        });
        let created = self.with_native(owner, |session, _| {
            let mut batch = DomBatch::new();
            let ticket = batch.create_fragment(document);
            Ok(session
                .dom()
                .raw_mut()
                .apply_with_node_retirement_watch(
                    batch,
                    &ticket,
                    &inbox,
                    key,
                    events.limits.node_retirement,
                    &owner.cancellation,
                )
                .map_err(DocumentScriptNativeError::from)?)
        });
        let (_, node, watch) = match created {
            Ok(created) => created.into_parts(),
            Err(error) => {
                let first = refuse(scope, &events.refusal, error);
                let pin = {
                    let allocation = pin;
                    *allocation
                };
                let _ = scope.release_callback_object(pin.holder);
                return first.map_err(Into::into);
            }
        };
        projection
            .payload()
            .node
            .set(Some(GuestNode::Fragment(node)));
        pin.live = Some(LivePin {
            node,
            _watch: watch,
        });
        reserved.publish(pin);
        Ok(value)
    }
}
