use super::{
    AccessError, AllocationCollection, EdgeVisitor, Handle, Heap, ObjectId, Payload, Rejected,
    SharedAllocation, Trace,
};
use std::{
    any::Any,
    cell::{RefCell, RefMut},
    fmt,
    ops::Deref,
    rc::{Rc, Weak},
};

#[cfg(test)]
mod tests;

/// Shared access to the heap of one JavaScript runtime.
///
/// Keep this owner in runtime clients and rooted access values. A stored
/// payload uses non-owning handles for outgoing heap references so it does not
/// keep its runtime alive.
#[derive(Clone, Default)]
pub struct RuntimeHeap(Rc<RefCell<Heap>>);

impl RuntimeHeap {
    /// Returns whether both owners refer to the same heap.
    #[must_use]
    pub fn same_heap(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    /// Sets this runtime's policy for collection during allocation.
    pub fn set_allocation_collection(
        &self,
        policy: AllocationCollection,
    ) -> Result<(), AccessError> {
        self.borrow()?.set_allocation_collection(policy);
        Ok(())
    }

    /// Publishes a record with its first root, or returns the rejected record.
    pub fn allocate<T: Trace>(&self, value: T) -> Result<Rooted<T>, Rejected<T>> {
        let SharedAllocation {
            payload,
            collection,
        } = {
            let mut heap = match self.borrow() {
                Ok(heap) => heap,
                Err(reason) => return Err(Rejected { reason, value }),
            };
            heap.allocate_shared(value)?
        };
        // The shared payload already roots the allocation if disposal reenters.
        drop(collection);
        Ok(Rooted {
            payload,
            runtime: self.clone(),
        })
    }

    /// Validates a pending payload and holds its live edges while preparing
    /// and publishing its record, including during prerequisite allocations.
    ///
    /// Runs the operation outside the heap borrow. The operation still needs
    /// to publish the completed payload through the checked allocation API.
    pub fn with_pending_edges<T: Trace, R>(
        &self,
        value: T,
        publish: impl FnOnce(T) -> R,
    ) -> Result<R, AccessError> {
        let _roots = self.borrow()?.root_pending_edges(&value)?;
        Ok(publish(value))
    }

    /// Checks the payload type of a live erased identity.
    pub fn downcast<T: Trace>(&self, id: ObjectId) -> Result<Handle<T>, AccessError> {
        self.borrow()?.downcast(id)
    }

    /// Reads checked records under one shared heap borrow, without rooting them.
    /// The callback cannot return a reference borrowed from the heap.
    /// Operations requiring a mutable heap borrow fail during the callback.
    pub fn read<R>(&self, read: impl FnOnce(&Heap) -> R) -> Result<R, AccessError> {
        let heap = self.0.try_borrow().map_err(|_| AccessError::HeapBorrowed)?;
        Ok(read(&heap))
    }

    /// Installs a root for a live handle. A reclaimed handle from this heap
    /// returns `None`.
    pub fn upgrade<T: Trace>(&self, handle: Handle<T>) -> Result<Option<Rooted<T>>, AccessError> {
        let payload = {
            let heap = self.borrow()?;
            let payload: Rc<dyn Any> = match heap.payload(handle.erase()) {
                Ok(payload) => payload.clone(),
                Err(AccessError::StaleHandle) => return Ok(None),
                Err(error) => return Err(error),
            };
            payload.downcast().map_err(|_| AccessError::WrongType)?
        };
        Ok(Some(Rooted {
            payload,
            runtime: self.clone(),
        }))
    }

    /// Reclaims unreachable records and disposes of them after releasing the
    /// heap borrow.
    pub fn collect(&self) -> Result<usize, AccessError> {
        let collection = self.borrow()?.collect()?;
        let reclaimed = collection.reclaimed();
        drop(collection);
        Ok(reclaimed)
    }

    fn borrow(&self) -> Result<RefMut<'_, Heap>, AccessError> {
        self.0
            .try_borrow_mut()
            .map_err(|_| AccessError::HeapBorrowed)
    }
}

/// A rooted record that keeps its runtime alive and can be read without an
/// arena borrow. A record's outgoing edges should store handles, which do not
/// keep their targets rooted.
pub struct Rooted<T> {
    payload: Rc<Payload<T>>,
    runtime: RuntimeHeap,
}

impl<T> Rooted<T> {
    /// Returns a non-owning handle to the record.
    #[must_use]
    pub fn handle(&self) -> Handle<T> {
        self.payload.handle
    }

    /// Stores the allocation identity without keeping the record or heap alive.
    #[must_use]
    pub fn record_ref(&self) -> RecordRef<T> {
        RecordRef {
            handle: self.handle(),
            runtime: Rc::downgrade(&self.runtime.0),
        }
    }

    /// Returns the runtime that owns the record.
    #[must_use]
    pub const fn runtime(&self) -> &RuntimeHeap {
        &self.runtime
    }
}

impl<T> Clone for Rooted<T> {
    fn clone(&self) -> Self {
        Self {
            payload: Rc::clone(&self.payload),
            runtime: self.runtime.clone(),
        }
    }
}

impl<T> Deref for Rooted<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.payload.value
    }
}

impl<T> fmt::Debug for Rooted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Rooted").field(&self.handle()).finish()
    }
}

/// An allocation identity with a non-owning locator for its heap.
pub struct RecordRef<T> {
    handle: Handle<T>,
    runtime: Weak<RefCell<Heap>>,
}

impl<T> RecordRef<T> {
    /// Returns the originating heap while that heap is still alive.
    ///
    /// This only upgrades the heap's weak owner: it neither borrows the heap
    /// nor checks the record's liveness or installs a record root. `Some` also
    /// permits ordinary access to the surviving origin heap when this record
    /// has been reclaimed, including after its slot has been reused.
    /// Retaining the returned owner extends the heap's lifetime, but does not
    /// keep this record alive. `None` means the origin heap has expired.
    #[must_use]
    pub fn owning_heap(&self) -> Option<RuntimeHeap> {
        self.runtime.upgrade().map(RuntimeHeap)
    }

    /// Returns the typed, non-owning identity of the record.
    #[must_use]
    pub const fn handle(&self) -> Handle<T> {
        self.handle
    }

    /// Returns the stable identity, including the heap and slot generation.
    #[must_use]
    pub const fn identity(&self) -> ObjectId {
        self.handle.erase()
    }
}

impl<T: Trace> RecordRef<T> {
    /// Roots the record if both its heap and slot are still live.
    pub fn upgrade(&self) -> Result<Option<Rooted<T>>, AccessError> {
        let Some(heap) = self.runtime.upgrade() else {
            return Ok(None);
        };
        RuntimeHeap(heap).upgrade(self.handle)
    }

    /// Checks liveness without changing the root set.
    pub fn is_live(&self) -> Result<bool, AccessError> {
        let Some(heap) = self.runtime.upgrade() else {
            return Ok(false);
        };
        let runtime = RuntimeHeap(heap);
        match runtime.borrow()?.get(self.handle) {
            Ok(_) => Ok(true),
            Err(AccessError::StaleHandle) => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl<T> Clone for RecordRef<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            runtime: self.runtime.clone(),
        }
    }
}

impl<T> PartialEq for RecordRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl<T> Eq for RecordRef<T> {}

impl<T> fmt::Debug for RecordRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.handle.fmt(f)
    }
}

impl<T: 'static> Trace for RecordRef<T> {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        edges.strong(self.identity());
        Ok(())
    }
}
