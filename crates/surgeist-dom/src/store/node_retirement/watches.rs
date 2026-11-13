//! Exact watch slots within a single native node generation.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct WatchSlot(usize);

impl WatchSlot {
    pub(super) const FIRST: Self = Self(0);
}

pub(super) enum Watches {
    Single(Option<Box<Armed>>),
    Shared(Box<Shared>),
}

pub(super) struct Shared {
    node: NodeId,
    slots: Vec<Entry>,
    free: Option<WatchSlot>,
    live: usize,
    backing: Debit,
    slots_backing: Debit,
}

pub(super) enum Entry {
    Armed(Box<Armed>),
    Free(Option<WatchSlot>),
}

impl Watches {
    pub(super) fn one(armed: Armed) -> Self {
        Self::Single(Some(Box::new(armed)))
    }

    pub(super) fn node(&self) -> Option<NodeId> {
        match self {
            Self::Single(armed) => armed.as_ref().map(|armed| armed.notice.allocation.node),
            Self::Shared(shared) => Some(shared.node),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        match self {
            Self::Single(armed) => armed.is_none(),
            Self::Shared(shared) => shared.live == 0,
        }
    }

    pub(super) fn is_open(&self, slot: WatchSlot) -> bool {
        let armed = match self {
            Self::Single(armed) if slot.0 == 0 => armed.as_deref(),
            Self::Shared(shared) => match shared.slots.get(slot.0) {
                Some(Entry::Armed(armed)) => Some(armed.as_ref()),
                _ => None,
            },
            _ => None,
        };
        armed.is_some_and(|armed| lock(&armed.queue).open)
    }

    pub(super) fn delivery_work(&self) -> (usize, usize) {
        match self {
            Self::Single(armed) => (0, usize::from(armed.is_some())),
            Self::Shared(shared) => (shared.slots.len(), shared.live),
        }
    }

    pub(super) fn add(&mut self, admission: &WatchAdmission<'_>) -> Result<WatchSlot, DomError> {
        match self {
            Self::Single(original) => {
                let current = original.as_ref().ok_or(DomError::RepresentationExhausted)?;
                if Arc::ptr_eq(&current.queue, &admission.inbox.queue) {
                    return Err(DomError::NodeRetirementAlreadyWatched);
                }
                // Each indexed watch prepays its unlink or ordinary delivery.
                admission.step()?;
                admission.step()?;
                let bytes = slot_bytes(2)?;
                let total = bytes
                    .checked_add(size_of::<Shared>())
                    .ok_or(DomError::RepresentationExhausted)?;
                let (armed, mut retained) = admission.form(total)?;
                let backing = retained.take(size_of::<Shared>())?;
                let slots_backing = retained.take(bytes)?;
                let mut slots = empty_slots(2, admission)?;
                check_cancel(admission.cancel)?;
                let old = original.take().ok_or(DomError::RepresentationExhausted)?;
                slots.push(Entry::Armed(old));
                slots.push(Entry::Armed(Box::new(armed)));
                *self = Self::Shared(Box::new(Shared {
                    node: admission.node,
                    slots,
                    free: None,
                    live: 2,
                    backing,
                    slots_backing,
                }));
                Ok(WatchSlot(1))
            }
            Self::Shared(shared) => shared.add(admission),
        }
    }

    pub(super) fn disarm(&mut self, slot: WatchSlot) -> Option<Box<Armed>> {
        match self {
            Self::Single(armed) if slot.0 == 0 => armed.take(),
            Self::Single(_) => None,
            Self::Shared(shared) => shared.disarm(slot),
        }
    }

    pub(super) fn publish(self) {
        match self {
            Self::Single(Some(armed)) => publish_notice({
                let allocation = armed;
                *allocation
            }),
            Self::Single(None) => {}
            Self::Shared(shared) => {
                let Shared {
                    slots,
                    backing,
                    slots_backing,
                    ..
                } = {
                    let allocation = shared;
                    *allocation
                };
                drop(backing);
                for entry in slots {
                    if let Entry::Armed(armed) = entry {
                        publish_notice({
                            let allocation = armed;
                            *allocation
                        });
                    }
                }
                drop(slots_backing);
            }
        }
    }

    pub(super) fn release_empty(self) {
        if let Self::Shared(shared) = self {
            let Shared {
                slots,
                backing,
                slots_backing,
                ..
            } = {
                let allocation = shared;
                *allocation
            };
            drop(slots);
            drop(slots_backing);
            drop(backing);
        }
    }
}

impl Shared {
    fn add(&mut self, admission: &WatchAdmission<'_>) -> Result<WatchSlot, DomError> {
        for entry in &self.slots {
            admission.step()?;
            if let Entry::Armed(armed) = entry
                && Arc::ptr_eq(&armed.queue, &admission.inbox.queue)
            {
                return Err(DomError::NodeRetirementAlreadyWatched);
            }
        }
        let live = self
            .live
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        // A fresh watch prepays unlink or delivery even in a reused slot.
        admission.step()?;
        if let Some(slot) = self.free {
            let entry = self
                .slots
                .get_mut(slot.0)
                .ok_or(DomError::RepresentationExhausted)?;
            let Entry::Free(next) = entry else {
                return Err(DomError::RepresentationExhausted);
            };
            let (armed, _) = admission.form(0)?;
            check_cancel(admission.cancel)?;
            self.free = *next;
            *entry = Entry::Armed(Box::new(armed));
            self.live = live;
            return Ok(slot);
        }
        let slot = WatchSlot(self.slots.len());
        if self.slots.len() == self.slots.capacity() {
            let capacity = self
                .slots
                .len()
                .checked_mul(2)
                .ok_or(DomError::RepresentationExhausted)?;
            let bytes = slot_bytes(capacity)?;
            let (armed, mut retained) = admission.form(bytes)?;
            let backing = retained.take(bytes)?;
            let mut replacement = empty_slots(capacity, admission)?;
            for _ in &self.slots {
                admission.step()?;
            }
            check_cancel(admission.cancel)?;
            replacement.append(&mut self.slots);
            replacement.push(Entry::Armed(Box::new(armed)));
            drop(std::mem::replace(&mut self.slots, replacement));
            self.slots_backing = backing;
        } else {
            let (armed, _) = admission.form(0)?;
            check_cancel(admission.cancel)?;
            self.slots.push(Entry::Armed(Box::new(armed)));
        }
        self.live = live;
        Ok(slot)
    }

    fn disarm(&mut self, slot: WatchSlot) -> Option<Box<Armed>> {
        let entry = self.slots.get_mut(slot.0)?;
        let live = self.live.checked_sub(1)?;
        if let Entry::Armed(_) = entry
            && let Entry::Armed(armed) = std::mem::replace(entry, Entry::Free(self.free))
        {
            self.free = Some(slot);
            self.live = live;
            return Some(armed);
        }
        None
    }
}

fn slot_bytes(capacity: usize) -> Result<usize, DomError> {
    capacity
        .checked_mul(size_of::<Entry>())
        .ok_or(DomError::RepresentationExhausted)
}

fn empty_slots(capacity: usize, admission: &WatchAdmission<'_>) -> Result<Vec<Entry>, DomError> {
    // The last disarm can release every vacancy. Prepay that teardown or
    // the ordinary publication scan when the allocation is admitted.
    for _ in 0..capacity {
        admission.step()?;
    }
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(capacity)
        .map_err(|_| DomError::AllocationFailed)?;
    if slots.capacity() != capacity {
        return Err(DomError::AllocationFailed);
    }
    Ok(slots)
}

fn publish_notice(armed: Armed) {
    let Armed {
        mut notice,
        queue,
        backing,
    } = armed;
    drop(backing);
    let mut queue = lock(&queue);
    if queue.open {
        notice.allocation.next = queue.head.take();
        queue.head = Some(notice);
    }
}
