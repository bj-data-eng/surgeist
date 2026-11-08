//! Stable diagnostics separated from local Context liveness.
use super::JobRetirement;
use crate::{JsResult, error::EngineResourceError};
use std::{
    cell::{Cell, RefCell},
    fmt,
    rc::{Rc, Weak},
    sync::Arc,
};

/// Opaque exact-Context identity with no engine access or owner lifetime.
#[derive(Clone)]
pub struct ContextId(Arc<()>);
impl ContextId {
    fn new() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for ContextId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ContextId {}
impl fmt::Debug for ContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ContextId")
            .field(&Arc::as_ptr(&self.0))
            .finish()
    }
}
impl fmt::Display for ContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", Arc::as_ptr(&self.0))
    }
}

pub(crate) trait OwnerTask {
    fn retire(&self, reason: JobRetirement);
}
enum Entry {
    Free(Option<usize>),
    Task(Weak<dyn OwnerTask>),
}
#[derive(Default)]
struct Registry {
    entries: Vec<Entry>,
    free: Option<usize>,
}
pub(crate) struct ContextLifetime {
    id: ContextId,
    alive: Cell<bool>,
    registry: RefCell<Registry>,
}
impl ContextLifetime {
    pub(crate) fn new() -> Rc<Self> {
        Rc::new(Self {
            id: ContextId::new(),
            alive: Cell::new(true),
            registry: RefCell::new(Registry::default()),
        })
    }
    pub(crate) fn id(&self) -> ContextId {
        self.id.clone()
    }
    pub(crate) fn register(
        self: &Rc<Self>,
        task: Rc<dyn OwnerTask>,
    ) -> JsResult<OwnerRegistration> {
        let index = {
            let mut registry = self.registry.borrow_mut();
            if let Some(index) = registry.free {
                let Entry::Free(next) = registry.entries[index] else {
                    unreachable!("free owner entry")
                };
                registry.free = next;
                registry.entries[index] = Entry::Task(Rc::downgrade(&task));
                index
            } else {
                registry
                    .entries
                    .try_reserve(1)
                    .map_err(|_| EngineResourceError::Allocation)?;
                let index = registry.entries.len();
                registry.entries.push(Entry::Task(Rc::downgrade(&task)));
                index
            }
        };
        Ok(OwnerRegistration {
            owner: Rc::downgrade(self),
            index,
        })
    }
    pub(crate) fn retire(&self) {
        if !self.alive.replace(false) {
            return;
        }
        let entries = std::mem::take(&mut self.registry.borrow_mut().entries);
        for entry in entries {
            if let Entry::Task(task) = entry
                && let Some(task) = task.upgrade()
            {
                task.retire(JobRetirement::OwnerDropped);
            }
        }
    }
}
pub(crate) struct OwnerRegistration {
    owner: Weak<ContextLifetime>,
    index: usize,
}
impl Drop for OwnerRegistration {
    fn drop(&mut self) {
        if let Some(owner) = self.owner.upgrade()
            && owner.alive.get()
        {
            let mut registry = owner.registry.borrow_mut();
            let next = registry.free;
            registry.entries[self.index] = Entry::Free(next);
            registry.free = Some(self.index);
        }
    }
}
