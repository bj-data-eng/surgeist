//! Stable diagnostics separated from local Context liveness.
use super::{
    JobFault, JobFaultCause, JobFaultOwner, JobRegistrationError, JobRetirement, UnwindCleanup,
    reserve_registration,
};
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
    fn mark_fault(&self, fault: JobFault);
    fn fault(&self, fault: JobFault);
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
    quarantined: Option<Vec<Entry>>,
}
pub(crate) struct ContextLifetime {
    id: ContextId,
    alive: Cell<bool>,
    registry: RefCell<Registry>,
    first_fault: Cell<Option<JobFault>>,
    cleanup_pending: Cell<bool>,
    cleanup_driving: Cell<bool>,
    fault_delivery: Cell<usize>,
}
/// Declared before a registry borrow, hence released after that borrow on unwind.
struct RegistryRelease<'a> {
    owner: &'a ContextLifetime,
    unwinding_at_entry: bool,
    completed: bool,
}
impl Drop for RegistryRelease<'_> {
    fn drop(&mut self) {
        if !self.completed && !self.unwinding_at_entry && std::thread::panicking() {
            self.owner.fault(JobFaultCause::StateInconsistency);
        }
        self.owner.finish_cleanup();
    }
}
impl ContextLifetime {
    pub(crate) fn new() -> Rc<Self> {
        Rc::new(Self {
            id: ContextId::new(),
            alive: Cell::new(true),
            registry: RefCell::new(Registry::default()),
            first_fault: Cell::new(None),
            cleanup_pending: Cell::new(false),
            cleanup_driving: Cell::new(false),
            fault_delivery: Cell::new(0),
        })
    }
    pub(crate) fn id(&self) -> ContextId {
        self.id.clone()
    }
    fn fault(&self, cause: JobFaultCause) -> JobFault {
        let first = self
            .first_fault
            .get()
            .unwrap_or_else(|| JobFault::new(JobFaultOwner::ContextLifetime, cause));
        self.first_fault.set(Some(first));
        self.cleanup_pending.set(true);
        self.finish_cleanup();
        first
    }
    fn latch_fault(&self, cause: JobFaultCause) {
        if self.first_fault.get().is_none() {
            self.first_fault
                .set(Some(JobFault::new(JobFaultOwner::ContextLifetime, cause)));
        }
        self.cleanup_pending.set(true);
    }
    fn walk_physical(&self, mut visit: impl FnMut(Rc<dyn OwnerTask>)) -> bool {
        // These banks remain owned after fault notification, so later actual
        // Context destruction can still dispose every quarantined task.
        for quarantine in [true, false] {
            let length = match self.registry.try_borrow() {
                Ok(registry) => {
                    if quarantine {
                        registry.quarantined.as_ref().map_or(0, Vec::len)
                    } else {
                        registry.entries.len()
                    }
                }
                Err(_) => {
                    self.latch_fault(JobFaultCause::InaccessibleRegistry);
                    return false;
                }
            };
            for index in 0..length {
                let task = {
                    let Ok(registry) = self.registry.try_borrow() else {
                        self.latch_fault(JobFaultCause::InaccessibleRegistry);
                        return false;
                    };
                    let entries = if quarantine {
                        registry.quarantined.as_deref().unwrap_or(&[])
                    } else {
                        registry.entries.as_slice()
                    };
                    match entries.get(index) {
                        Some(Entry::Task(task)) => Some(task.clone()),
                        Some(Entry::Free(_)) => None,
                        None => {
                            self.latch_fault(JobFaultCause::InvalidIndex { index });
                            return false;
                        }
                    }
                };
                if let Some(task) = task.and_then(|task| task.upgrade()) {
                    visit(task);
                }
            }
        }
        true
    }
    fn notify_fault(&self, fault: JobFault, cleanup: &mut UnwindCleanup) -> bool {
        // Fault freezes both banks. Their fixed physical order, independent of
        // damaged free links, lets one owner cursor claim each callback once.
        loop {
            let task = {
                let Ok(registry) = self.registry.try_borrow() else {
                    self.latch_fault(JobFaultCause::InaccessibleRegistry);
                    return false;
                };
                let quarantined = registry.quarantined.as_deref().unwrap_or(&[]);
                let Some(length) = quarantined.len().checked_add(registry.entries.len()) else {
                    self.latch_fault(JobFaultCause::StateInconsistency);
                    return false;
                };
                let index = self.fault_delivery.get();
                if index == length {
                    return true;
                }
                let entry = if index < quarantined.len() {
                    quarantined.get(index)
                } else {
                    registry.entries.get(index - quarantined.len())
                };
                let Some(entry) = entry else {
                    self.latch_fault(JobFaultCause::InvalidIndex { index });
                    return false;
                };
                let Some(next) = index.checked_add(1) else {
                    self.latch_fault(JobFaultCause::StateInconsistency);
                    return false;
                };
                self.fault_delivery.set(next);
                match entry {
                    Entry::Task(task) => Some(task.clone()),
                    Entry::Free(_) => None,
                }
            };
            if let Some(task) = task.and_then(|task| task.upgrade()) {
                cleanup.run(move || task.fault(fault));
            }
        }
    }
    fn finish_cleanup(&self) {
        if !self.cleanup_pending.get() || self.cleanup_driving.get() {
            return;
        }
        let Ok(mut registry) = self.registry.try_borrow_mut() else {
            return;
        };
        registry.free = None;
        if registry.quarantined.is_none() {
            registry.quarantined = Some(std::mem::take(&mut registry.entries));
        }
        self.cleanup_pending.set(false);
        self.cleanup_driving.set(true);
        drop(registry);
        let mut cleanup = UnwindCleanup::default();
        let mut complete = true;
        let initial_fault = self.first_fault.get();
        if let Some(fault) = initial_fault {
            complete = self.walk_physical(|task| cleanup.run(move || task.mark_fault(fault)));
            if complete {
                complete = self.notify_fault(fault, &mut cleanup);
            }
        }
        if complete && !self.alive.get() {
            complete = self.walk_physical(|task| {
                cleanup.run(move || {
                    if let Some(fault) = self.first_fault.get() {
                        task.mark_fault(fault);
                    }
                    task.retire(JobRetirement::OwnerDropped);
                })
            });
            if complete
                && initial_fault.is_none()
                && let Some(fault) = self.first_fault.get()
            {
                complete = self.walk_physical(|task| cleanup.run(move || task.mark_fault(fault)));
                if complete {
                    complete = self.notify_fault(fault, &mut cleanup);
                }
            }
            // Physical weak-bank disposal occurs only after actual owner loss,
            // never merely because a fault detached the normal registry.
            if complete {
                if let Ok(mut registry) = self.registry.try_borrow_mut() {
                    let entries = std::mem::take(&mut registry.entries);
                    let quarantined = registry.quarantined.take();
                    self.fault_delivery.set(0);
                    drop(registry);
                    drop((entries, quarantined));
                } else {
                    self.latch_fault(JobFaultCause::InaccessibleRegistry);
                    complete = false;
                }
            }
        }
        self.cleanup_pending.set(!complete);
        self.cleanup_driving.set(false);
        cleanup.finish();
    }
    fn with_registry<T>(
        &self,
        run: impl FnOnce(&mut Registry) -> Result<T, JobRegistrationError>,
    ) -> Result<T, JobRegistrationError> {
        if let Some(fault) = self.first_fault.get() {
            return Err(fault.into());
        }
        let mut release = RegistryRelease {
            owner: self,
            unwinding_at_entry: std::thread::panicking(),
            completed: false,
        };
        let mut registry = match self.registry.try_borrow_mut() {
            Ok(registry) => registry,
            Err(_) => return Err(self.fault(JobFaultCause::InaccessibleRegistry).into()),
        };
        if registry.quarantined.is_some() {
            drop(registry);
            return Err(self.fault(JobFaultCause::StateInconsistency).into());
        }
        let mut result = run(&mut registry);
        release.completed = true;
        drop(registry);
        if let Err(JobRegistrationError::Fault(fault)) = &result {
            self.first_fault
                .set(Some(self.first_fault.get().unwrap_or(*fault)));
            self.cleanup_pending.set(true);
        }
        if let Some(fault) = self.first_fault.get() {
            result = Err(fault.into());
        }
        result
    }
    pub(crate) fn register(
        self: &Rc<Self>,
        task: Rc<dyn OwnerTask>,
    ) -> Result<OwnerRegistration, JobRegistrationError> {
        if !self.alive.get() {
            return Err(self.fault(JobFaultCause::StateInconsistency).into());
        }
        let expected = Rc::downgrade(&task);
        let index = self.with_registry(|registry| {
            let (index, next) = if let Some(index) = registry.free {
                match registry.entries.get(index) {
                    Some(Entry::Free(next))
                        if next.is_none_or(|next| {
                            next != index
                                && matches!(registry.entries.get(next), Some(Entry::Free(_)))
                        }) =>
                    {
                        (index, *next)
                    }
                    _ => {
                        return Err(JobFault::new(
                            JobFaultOwner::ContextLifetime,
                            JobFaultCause::InvalidFreeLink { index },
                        )
                        .into());
                    }
                }
            } else {
                reserve_registration(&mut registry.entries)?;
                (registry.entries.len(), None)
            };
            if index == registry.entries.len() {
                registry.entries.push(Entry::Task(expected.clone()));
            } else if let Some(entry) = registry.entries.get_mut(index) {
                *entry = Entry::Task(expected.clone());
            } else {
                return Err(JobFault::new(
                    JobFaultOwner::ContextLifetime,
                    JobFaultCause::InvalidIndex { index },
                )
                .into());
            }
            registry.free = next;
            Ok(index)
        })?;
        Ok(OwnerRegistration {
            owner: Rc::downgrade(self),
            index,
            expected,
        })
    }
    pub(crate) fn retire(&self) {
        if !self.alive.replace(false) {
            return;
        }
        self.cleanup_pending.set(true);
        if self.registry.try_borrow_mut().is_err() {
            self.fault(JobFaultCause::InaccessibleRegistry);
        } else {
            self.finish_cleanup();
        }
    }
}
impl Drop for ContextLifetime {
    fn drop(&mut self) {
        self.alive.set(false);
        let registry = self.registry.get_mut();
        let entries = std::mem::take(&mut registry.entries);
        let quarantined = registry.quarantined.take().unwrap_or_default();
        registry.free = None;
        let mut cleanup = UnwindCleanup::default();
        if let Some(fault) = self.first_fault.get() {
            for entry in quarantined.iter().chain(&entries) {
                if let Entry::Task(task) = entry
                    && let Some(task) = task.upgrade()
                {
                    cleanup.run(move || task.mark_fault(fault));
                }
            }
            for (index, entry) in quarantined.iter().chain(&entries).enumerate() {
                if index < self.fault_delivery.get() {
                    continue;
                }
                let Some(next) = index.checked_add(1) else {
                    self.latch_fault(JobFaultCause::StateInconsistency);
                    break;
                };
                self.fault_delivery.set(next);
                if let Entry::Task(task) = entry
                    && let Some(task) = task.upgrade()
                {
                    cleanup.run(move || task.fault(fault));
                }
            }
        }
        for entry in entries.iter().chain(&quarantined) {
            if let Entry::Task(task) = entry
                && let Some(task) = task.upgrade()
            {
                cleanup.run(move || task.retire(JobRetirement::OwnerDropped));
            }
        }
        drop((entries, quarantined));
        cleanup.finish();
    }
}
pub(crate) struct OwnerRegistration {
    owner: Weak<ContextLifetime>,
    index: usize,
    expected: Weak<dyn OwnerTask>,
}
impl Drop for OwnerRegistration {
    fn drop(&mut self) {
        let Some(owner) = self.owner.upgrade() else {
            return;
        };
        if !owner.alive.get() {
            owner.finish_cleanup();
            return;
        }
        let _result = owner.with_registry(|registry| {
            match registry.entries.get(self.index) {
                Some(Entry::Task(task)) if Weak::ptr_eq(task, &self.expected) => {}
                Some(_) => {
                    return Err(JobFault::new(
                        JobFaultOwner::ContextLifetime,
                        JobFaultCause::InvalidSlotIdentity { index: self.index },
                    )
                    .into());
                }
                None => {
                    return Err(JobFault::new(
                        JobFaultOwner::ContextLifetime,
                        JobFaultCause::InvalidIndex { index: self.index },
                    )
                    .into());
                }
            }
            if let Some(index) = registry.free
                && (index == self.index
                    || !matches!(registry.entries.get(index), Some(Entry::Free(_))))
            {
                return Err(JobFault::new(
                    JobFaultOwner::ContextLifetime,
                    JobFaultCause::InvalidFreeLink { index },
                )
                .into());
            }
            let next = registry.free;
            let Some(entry) = registry.entries.get_mut(self.index) else {
                return Err(JobFault::new(
                    JobFaultOwner::ContextLifetime,
                    JobFaultCause::InvalidIndex { index: self.index },
                )
                .into());
            };
            *entry = Entry::Free(next);
            registry.free = Some(self.index);
            Ok(())
        });
    }
}

#[cfg(test)]
mod fault_tests;
