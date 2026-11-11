//! Retained projections for the explicit native Source Comment delegation.

use super::*;
use surgeist_bindings_js::ManagedObject;
use surgeist_dom::{DomSourceEventPayloadRetention, NodeId};

#[derive(Default)]
pub(super) struct ProjectedComments {
    head: Option<Box<Pin>>,
    len: usize,
}

struct Pin {
    node: NodeId,
    object: ManagedObject,
    next: Option<Box<Pin>>,
    backing: DomSourceEventPayloadRetention,
}

impl Pin {
    fn release(self, process: &mut Process) {
        // The Process freezes any registry integrity failure. The exact
        // holder is released before its native debit.
        let _ = process.release_managed_object(self.object);
        drop(self.backing);
    }
}

impl ProjectedComments {
    pub(super) fn pin_bytes(process: &Process) -> Option<usize> {
        size_of::<Pin>().checked_add(process.managed_object_storage_bytes())
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn find(
        &self,
        node: NodeId,
        mut step: impl FnMut() -> Result<(), DocumentScriptNativeError>,
    ) -> Result<Option<&ManagedObject>, DocumentScriptNativeError> {
        let mut next = self.head.as_deref();
        while let Some(pin) = next {
            step()?;
            if pin.node == node {
                return Ok(Some(&pin.object));
            }
            next = pin.next.as_deref();
        }
        Ok(None)
    }

    pub(super) fn insert(
        &mut self,
        node: NodeId,
        object: ManagedObject,
        backing: DomSourceEventPayloadRetention,
    ) -> &ManagedObject {
        self.len += 1;
        let next = self.head.take();
        &self
            .head
            .insert(Box::new(Pin {
                node,
                object,
                next,
                backing,
            }))
            .object
    }

    pub(super) fn retire(&mut self, snapshot: &DomSnapshot, process: &mut Process) {
        let mut remaining = self.head.take();
        let mut tail = &mut self.head;
        while let Some(mut pin) = remaining {
            remaining = pin.next.take();
            if matches!(snapshot.node(pin.node), Err(DomError::StaleNode)) {
                self.len -= 1;
                pin.release(process);
            } else {
                tail = &mut tail.insert(pin).next;
            }
        }
    }

    pub(super) fn release_all(&mut self, process: &mut Process) {
        while let Some(mut pin) = self.head.take() {
            self.head = pin.next.take();
            pin.release(process);
        }
        self.len = 0;
    }
}

impl Drop for ProjectedComments {
    fn drop(&mut self) {
        while let Some(mut pin) = self.head.take() {
            self.head = pin.next.take();
        }
    }
}
