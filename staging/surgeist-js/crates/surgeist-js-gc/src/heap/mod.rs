//! Runtime-owned storage with checked identities and explicit root custody.

#![forbid(unsafe_code)]

use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    error::Error,
    fmt,
    marker::PhantomData,
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(test)]
mod tests;

static NEXT_HEAP_ID: AtomicU64 = AtomicU64::new(1);

/// Untyped identity of a particular allocation in a particular heap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId {
    heap: u64,
    slot: usize,
    generation: u64,
}

/// A typed identity. Keeping a handle does not keep its payload alive.
pub struct Handle<T> {
    id: ObjectId,
    invariant: PhantomData<fn(T) -> T>,
}

impl<T> Handle<T> {
    /// Erases the payload type while preserving allocation identity.
    #[must_use]
    pub const fn erase(self) -> ObjectId {
        self.id
    }

    const fn new(id: ObjectId) -> Self {
        Self {
            id,
            invariant: PhantomData,
        }
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Handle<T> {}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}

/// Owns a root registration for a handle. The last clone releases that root.
pub struct Root<T> {
    handle: Handle<T>,
    registration: Rc<()>,
}

impl<T> Root<T> {
    /// Returns the registered handle without transferring the registration.
    #[must_use]
    pub const fn handle(&self) -> Handle<T> {
        self.handle
    }
}

impl<T> Clone for Root<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            registration: Rc::clone(&self.registration),
        }
    }
}

/// Reasons a handle or one of its declared edges cannot be resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessError {
    /// The handle belongs to a different heap.
    ForeignHeap,
    /// The allocation was reclaimed, possibly followed by reuse of its slot.
    StaleHandle,
    /// The payload's concrete type differs from the requested type.
    WrongType,
}

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ForeignHeap => "handle belongs to a different JavaScript heap",
            Self::StaleHandle => "JavaScript heap handle is stale",
            Self::WrongType => "JavaScript heap payload has a different type",
        })
    }
}

impl Error for AccessError {}

/// An input the heap refused to publish, still owned by the caller.
pub struct Rejected<T> {
    /// Why the input was refused.
    pub reason: AccessError,
    /// The unpublished input.
    pub value: T,
}

impl<T> fmt::Debug for Rejected<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rejected")
            .field("reason", &self.reason)
            .finish_non_exhaustive()
    }
}

/// Enumerates the strong heap edges held by a payload.
///
/// Tracing only visits edges. It must leave the graph and root registrations
/// unchanged and must not reenter the heap. The collector may visit the same
/// payload more than once during an operation.
pub trait Trace: Any {
    /// Sends each strong edge to the visitor.
    fn trace(&self, edges: &mut EdgeVisitor<'_>);
}

/// Receives edges without giving the payload access to collection state.
pub struct EdgeVisitor<'a> {
    queue: &'a mut VecDeque<ObjectId>,
}

impl EdgeVisitor<'_> {
    /// Declares an edge that keeps its target alive while the owner is live.
    pub fn strong(&mut self, target: ObjectId) {
        self.queue.push_back(target);
    }
}

struct Slot {
    generation: u64,
    payload: Option<Box<dyn Trace>>,
}

/// Controls whether an allocation triggers collection before publishing its payload.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AllocationCollection {
    /// Collect only when requested.
    #[default]
    Manual,
    /// Collect on every allocation, protecting the incoming payload's edges.
    BeforeAllocation,
}

/// Owns reclaimed payloads after a successful collection.
///
/// Drop this outside the runtime's heap borrow. Reclamation invalidates handles
/// before transferring payloads here, and destructors run when custody ends.
#[derive(Default)]
pub struct Collection {
    retired: Vec<Box<dyn Trace>>,
}

impl Collection {
    /// Returns the number of payloads retired from the heap.
    #[must_use]
    pub fn reclaimed(&self) -> usize {
        self.retired.len()
    }
}

/// The new root and any reclamation performed to make room for it.
pub struct Allocation<T> {
    /// The first root for the new payload.
    pub root: Root<T>,
    /// Reclaimed payloads to dispose after releasing heap access.
    pub collection: Collection,
}

/// The runtime's allocation, root and collection authority.
pub struct Heap {
    id: u64,
    slots: Vec<Slot>,
    vacant: Vec<usize>,
    roots: HashMap<ObjectId, Weak<()>>,
    allocation_collection: AllocationCollection,
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heap {
    /// Creates an empty heap with its own identity.
    #[must_use]
    pub fn new() -> Self {
        let id = NEXT_HEAP_ID
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("JavaScript heap identities exhausted");
        Self {
            id,
            slots: Vec::new(),
            vacant: Vec::new(),
            roots: HashMap::new(),
            allocation_collection: AllocationCollection::Manual,
        }
    }

    /// Sets the allocation collection policy for this heap.
    pub fn set_allocation_collection(&mut self, policy: AllocationCollection) {
        self.allocation_collection = policy;
    }

    /// Publishes a payload and returns its first root.
    ///
    /// Invalid incoming edges prevent publication. A collection before publication
    /// visits the incoming payload's edges as temporary roots. Reclaimed payloads
    /// transfer to the caller without running their destructors during allocation.
    pub fn allocate<T: Trace>(&mut self, value: T) -> Result<Allocation<T>, Rejected<T>> {
        let prepared = self.checked_edges(&value).and_then(|edges| {
            if self.allocation_collection == AllocationCollection::BeforeAllocation {
                self.collect_from(edges)
            } else {
                Ok(Collection::default())
            }
        });
        let collection = match prepared {
            Ok(collection) => collection,
            Err(reason) => return Err(Rejected { reason, value }),
        };

        let slot = self.vacant.pop().unwrap_or_else(|| {
            let index = self.slots.len();
            self.slots.push(Slot {
                generation: 1,
                payload: None,
            });
            index
        });
        let entry = &mut self.slots[slot];
        entry.payload = Some(Box::new(value));
        let id = ObjectId {
            heap: self.id,
            slot,
            generation: entry.generation,
        };
        Ok(Allocation {
            root: self.register(Handle::new(id)),
            collection,
        })
    }

    /// Borrows the live typed payload for the lifetime of the heap borrow.
    pub fn get<T: Trace>(&self, handle: Handle<T>) -> Result<&T, AccessError> {
        let payload = self.payload(handle.id)? as &dyn Any;
        payload.downcast_ref().ok_or(AccessError::WrongType)
    }

    /// Recovers the typed identity of an erased live payload.
    pub fn downcast<T: Trace>(&self, id: ObjectId) -> Result<Handle<T>, AccessError> {
        let handle = Handle::new(id);
        self.get(handle)?;
        Ok(handle)
    }

    /// Replaces a live payload after checking that the replacement's edges resolve.
    ///
    /// Transfers the displaced payload to the caller for disposal outside the
    /// heap update. A rejected replacement also returns its input.
    pub fn replace<T: Trace>(
        &mut self,
        handle: Handle<T>,
        value: T,
    ) -> Result<Box<T>, Rejected<T>> {
        if let Err(reason) = self.get(handle).and_then(|_| self.checked_edges(&value)) {
            return Err(Rejected { reason, value });
        }
        let displaced = self.slots[handle.id.slot]
            .payload
            .replace(Box::new(value))
            .expect("validated payload");
        let erased: Box<dyn Any> = displaced;
        Ok(erased.downcast().expect("validated payload type"))
    }

    /// Installs a root if the target is still live in this heap.
    pub fn upgrade<T: Trace>(&mut self, handle: Handle<T>) -> Result<Option<Root<T>>, AccessError> {
        match self.get(handle) {
            Ok(_) => Ok(Some(self.register(handle))),
            Err(AccessError::StaleHandle) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Reclaims every payload unreachable from the current root registrations.
    ///
    /// Returns custody of reclaimed payloads. Dispose of them and dispatch any
    /// engine finalization outside the heap borrow.
    pub fn collect(&mut self) -> Result<Collection, AccessError> {
        self.collect_from(VecDeque::new())
    }

    fn payload(&self, id: ObjectId) -> Result<&dyn Trace, AccessError> {
        if id.heap != self.id {
            return Err(AccessError::ForeignHeap);
        }
        self.slots
            .get(id.slot)
            .filter(|entry| entry.generation == id.generation)
            .and_then(|entry| entry.payload.as_deref())
            .ok_or(AccessError::StaleHandle)
    }

    fn register<T>(&mut self, handle: Handle<T>) -> Root<T> {
        let registration = self.roots.entry(handle.id).or_default();
        let root = registration.upgrade().unwrap_or_else(|| {
            let root = Rc::new(());
            *registration = Rc::downgrade(&root);
            root
        });
        Root {
            handle,
            registration: root,
        }
    }

    fn checked_edges(&self, value: &dyn Trace) -> Result<VecDeque<ObjectId>, AccessError> {
        let mut queue = VecDeque::new();
        value.trace(&mut EdgeVisitor { queue: &mut queue });
        for &edge in &queue {
            self.payload(edge)?;
        }
        Ok(queue)
    }

    fn collect_from(&mut self, mut queue: VecDeque<ObjectId>) -> Result<Collection, AccessError> {
        queue.extend(
            self.roots
                .iter()
                .filter(|(_, root)| root.strong_count() != 0)
                .map(|(&id, _)| id),
        );

        // Mark into scratch storage so a malformed edge cannot partially sweep
        // the heap or advance a generation.
        let mut marked = vec![false; self.slots.len()];
        while let Some(id) = queue.pop_front() {
            let payload = self.payload(id)?;
            if !marked[id.slot] {
                marked[id.slot] = true;
                payload.trace(&mut EdgeVisitor { queue: &mut queue });
            }
        }

        let mut retired = Vec::new();
        for (index, (entry, reachable)) in self.slots.iter_mut().zip(marked).enumerate() {
            if reachable {
                continue;
            }
            if let Some(payload) = entry.payload.take() {
                retired.push(payload);
                if let Some(next_generation) = entry.generation.checked_add(1) {
                    entry.generation = next_generation;
                    self.vacant.push(index);
                }
            }
        }
        self.roots.retain(|_, root| root.strong_count() != 0);
        Ok(Collection { retired })
    }
}
