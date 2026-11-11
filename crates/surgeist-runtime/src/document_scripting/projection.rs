//! Retained projections for the explicit native Source Comment delegation.

use super::*;
use surgeist_bindings_js::ManagedObject;
use surgeist_dom::{
    Dom, DomNodeRetirementInbox, DomNodeRetirementWatch, DomSourceEventPayloadRetention,
    DomSourceEventRequest, NodeId,
};

#[derive(Default)]
pub(super) struct ProjectedComments {
    slots: Vec<Slot>,
    slots_backing: Option<DomSourceEventPayloadRetention>,
    free: Option<usize>,
    live: usize,
    inbox: Option<DomNodeRetirementInbox>,
}

enum Slot {
    Pin(Box<Pin>),
    Vacant(Option<usize>),
}

struct Pin {
    node: NodeId,
    object: ManagedObject,
    watch: DomNodeRetirementWatch,
    backing: DomSourceEventPayloadRetention,
}

impl Pin {
    fn release(self, process: &mut Process) {
        // The Process freezes any registry integrity failure. The exact
        // holder is released before its native debit.
        let _ = process.release_managed_object(self.object);
        drop(self.watch);
        drop(self.backing);
    }
}

#[derive(Clone, Copy)]
pub(super) struct SlotPlan {
    pub(super) key: usize,
    capacity: usize,
}

impl SlotPlan {
    pub(super) fn growth_bytes(self) -> Option<usize> {
        self.capacity.checked_mul(size_of::<Slot>())
    }
}

pub(super) struct VacantSlot<'a> {
    slot: &'a mut Slot,
    free: &'a mut Option<usize>,
    live: &'a mut usize,
    next: Option<usize>,
}

impl VacantSlot<'_> {
    pub(super) fn install(
        self,
        node: NodeId,
        object: ManagedObject,
        watch: DomNodeRetirementWatch,
        backing: DomSourceEventPayloadRetention,
    ) {
        *self.slot = Slot::Pin(Box::new(Pin {
            node,
            object,
            watch,
            backing,
        }));
        *self.free = self.next;
        *self.live += 1;
    }
}

impl ProjectedComments {
    pub(super) fn pin_bytes(process: &Process) -> Option<usize> {
        size_of::<Pin>().checked_add(process.managed_object_storage_bytes())
    }

    pub(super) fn slot_bytes(&self) -> Option<usize> {
        self.slots.capacity().checked_mul(size_of::<Slot>())
    }

    pub(super) fn len(&self) -> usize {
        self.live
    }

    pub(super) fn find(
        &self,
        node: NodeId,
        mut step: impl FnMut() -> Result<(), DocumentScriptNativeError>,
    ) -> Result<Option<usize>, DocumentScriptNativeError> {
        for (key, slot) in self.slots.iter().enumerate() {
            step()?;
            if matches!(slot, Slot::Pin(pin) if pin.node == node) {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }

    pub(super) fn plan_slot(&self) -> Option<SlotPlan> {
        let key = self.free.unwrap_or(self.slots.len());
        let capacity = if key != self.slots.capacity() {
            0
        } else if key == 0 {
            1
        } else {
            key.checked_mul(2)?
        };
        Some(SlotPlan { key, capacity })
    }

    pub(super) fn arm(
        &mut self,
        dom: &mut Dom,
        request: &DomSourceEventRequest,
        node: NodeId,
        plan: SlotPlan,
        cancel: &AtomicBool,
    ) -> Result<DomNodeRetirementWatch, DomError> {
        // Admit the later direct removal and every accepted slot moved into
        // replacement backing before accepting the pin.
        dom.source_event_step(request, cancel)?;
        if plan.capacity != 0 {
            for _ in &self.slots {
                dom.source_event_step(request, cancel)?;
            }
        }
        if self.inbox.is_none() {
            self.inbox = Some(dom.source_event_node_retirement_inbox(request, cancel)?);
        }
        let inbox = self
            .inbox
            .as_ref()
            .ok_or(DomError::RepresentationExhausted)?;
        dom.watch_source_event_node_retirement(request, node, inbox, plan.key, cancel)
    }

    pub(super) fn grow(
        &mut self,
        plan: SlotPlan,
        backing: Option<DomSourceEventPayloadRetention>,
    ) -> Result<(), DomError> {
        let Some(backing) = backing else {
            return Ok(());
        };
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(plan.capacity)
            .map_err(|_| DomError::AllocationFailed)?;
        if slots.capacity() != plan.capacity {
            return Err(DomError::RepresentationExhausted);
        }
        slots.append(&mut self.slots);
        drop(std::mem::replace(&mut self.slots, slots));
        drop(self.slots_backing.replace(backing));
        Ok(())
    }

    pub(super) fn vacant(&mut self, key: usize) -> Option<VacantSlot<'_>> {
        if self.free.is_none() && key == self.slots.len() && key < self.slots.capacity() {
            self.slots.push(Slot::Vacant(None));
            self.free = Some(key);
        }
        if self.free != Some(key) {
            return None;
        }
        let slot = self.slots.get_mut(key)?;
        let Slot::Vacant(next) = *slot else {
            return None;
        };
        Some(VacantSlot {
            slot,
            free: &mut self.free,
            live: &mut self.live,
            next,
        })
    }

    pub(super) fn object(&self, key: usize) -> Option<&ManagedObject> {
        match self.slots.get(key)? {
            Slot::Pin(pin) => Some(&pin.object),
            Slot::Vacant(_) => None,
        }
    }

    pub(super) fn drain_retirements(&mut self, process: &mut Process) {
        while let Some(notice) = self.inbox.as_mut().and_then(DomNodeRetirementInbox::take) {
            if let Some(pin) = self.take_retired(notice.custody_key(), notice.node()) {
                pin.release(process);
            }
        }
        if self.live == 0 {
            self.slots.clear();
            self.free = None;
        }
    }

    fn take_retired(&mut self, key: usize, node: NodeId) -> Option<Box<Pin>> {
        let slot = self.slots.get_mut(key)?;
        if !matches!(slot, Slot::Pin(pin) if pin.node == node) {
            return None;
        }
        let Slot::Pin(pin) = std::mem::replace(slot, Slot::Vacant(self.free)) else {
            return None;
        };
        self.free = Some(key);
        self.live -= 1;
        Some(pin)
    }

    pub(super) fn release_all(&mut self, process: &mut Process) {
        for slot in std::mem::take(&mut self.slots) {
            if let Slot::Pin(pin) = slot {
                pin.release(process);
            }
        }
        drop(self.slots_backing.take());
        self.live = 0;
        self.drain_retirements(process);
        drop(self.inbox.take());
    }
}
