//! Retained custody indexed by exact native node retirement.

use super::*;
use surgeist_dom::DomNodeRetirementWatch;

pub(super) struct OrdinaryBindings {
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
    lifetime: Rc<NodeLifetime>,
    _backing: GuestBacking,
}

pub(in crate::document_scripting::guest) struct NodeLifetime {
    retired: Cell<bool>,
    _backing: GuestBacking,
}

impl NodeLifetime {
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

impl OrdinaryBindings {
    pub(super) fn new(inbox: DomNodeRetirementInbox) -> Self {
        Self {
            inbox: RefCell::new(inbox),
            pins: RefCell::new(OrdinaryPins::default()),
        }
    }

    pub(super) fn project<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        owner: &NativeOwner,
        events: &GuestEvents,
        prototype: &Value<'scope>,
        publish: impl FnOnce(
            &DomNodeRetirementInbox,
            usize,
        )
            -> Result<(GuestNode, DomNodeRetirementWatch), DocumentGuestEventError>,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let mut pins = self
            .pins
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let (key, reserved) = pins.reserve(owner, events)?;
        let inbox = self
            .inbox
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let backing = events.reserve(owner, size_of::<Pin>())?;
        let lifetime_backing =
            events.reserve(owner, size_of::<NodeLifetime>() + 2 * size_of::<usize>())?;
        let lifetime = Rc::new(NodeLifetime {
            retired: Cell::new(false),
            _backing: lifetime_backing,
        });
        let holder = events.project(
            owner,
            scope,
            ObjectPrototype::Object(prototype),
            GuestNodeProjection {
                node: Cell::new(None),
                lifetime: Some(lifetime.clone()),
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
        let (node, watch) = match publish(&inbox, key) {
            Ok(created) => created,
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
        projection.payload().node.set(Some(node));
        pin.live = Some(LivePin {
            node: node.node(),
            _watch: watch,
        });
        reserved.publish(pin);
        Ok(value)
    }

    pub(super) fn open<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        events: &GuestEvents,
        node: NodeId,
    ) -> Result<Option<Value<'scope>>, DocumentGuestEventError> {
        let pins = self
            .pins
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        for slot in &pins.slots {
            events.step()?;
            if let Slot::Retained(pin) = slot
                && pin.live.as_ref().is_some_and(|live| live.node == node)
            {
                return Ok(Some(scope.open_callback_object(&pin.holder)?));
            }
        }
        Ok(None)
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
