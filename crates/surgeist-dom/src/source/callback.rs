//! The retained work and reentry scope shared by Source callbacks.
use crate::DomError;
use crate::store::{SourceStorage, source::formation::allocated_arc};
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering},
};

#[derive(Debug)]
pub(crate) struct DispatchScope;
#[derive(Debug)]
pub(crate) struct CallbackWork {
    pub(crate) stage: AtomicU8,
    pub(crate) work: Arc<AtomicU64>,
    pub(crate) work_limit: AtomicU64,
    scope: Mutex<Weak<DispatchScope>>,
    pub(crate) ancestors: Option<Arc<CallbackWork>>,
    pub(crate) bytes: Arc<AtomicUsize>,
    pub(crate) units: Arc<AtomicUsize>,
    pub(crate) storage: SourceStorage,
}
impl CallbackWork {
    pub(crate) fn new(
        work: Arc<AtomicU64>,
        work_limit: u64,
        ancestors: Option<Arc<Self>>,
        storage: SourceStorage,
    ) -> Self {
        Self {
            stage: AtomicU8::new(0),
            work,
            work_limit: AtomicU64::new(work_limit),
            scope: Mutex::new(Weak::new()),
            ancestors,
            bytes: allocated_arc(AtomicUsize::new(0)),
            units: allocated_arc(AtomicUsize::new(0)),
            storage,
        }
    }

    pub(crate) fn step_own(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(DomError::Cancelled);
        }
        let next = self
            .work
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let limit = self.work_limit.load(Ordering::Relaxed);
        if next > limit {
            return Err(DomError::SourceWorkLimit { limit });
        }
        self.work.store(next, Ordering::Relaxed);
        SourceStorage::record_step();
        Ok(())
    }

    pub(crate) fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.step_own(cancel)?;
        step_ancestors(self.ancestors.as_ref(), cancel)
    }
    pub(crate) fn active(&self) -> bool {
        self.stage.load(Ordering::Relaxed) == 1
            && self
                .scope
                .lock()
                .is_ok_and(|scope| scope.upgrade().is_some())
    }

    pub(crate) fn open_scope(&self) -> Option<Arc<DispatchScope>> {
        let mut slot = self.scope.lock().ok()?;
        if slot.upgrade().is_some() {
            return None;
        }
        let scope = Arc::new(DispatchScope);
        *slot = Arc::downgrade(&scope);
        Some(scope)
    }

    pub(crate) fn scope_closed(&self) -> bool {
        self.scope
            .lock()
            .is_ok_and(|scope| scope.upgrade().is_none())
    }
}

/// Charge each actual retained owner, without resetting or replacing its meter.
pub(crate) fn step_ancestors(
    mut ancestor: Option<&Arc<CallbackWork>>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    while let Some(work) = ancestor {
        work.step_own(cancel)?;
        ancestor = work.ancestors.as_ref();
    }
    Ok(())
}

pub(crate) fn remaining_ancestor_work(
    mut ancestor: Option<&Arc<CallbackWork>>,
    mut remaining: u64,
) -> Result<u64, DomError> {
    while let Some(work) = ancestor {
        let limit = work.work_limit.load(Ordering::Relaxed);
        remaining = remaining.min(
            limit
                .checked_sub(work.work.load(Ordering::Relaxed))
                .ok_or(DomError::SourceWorkLimit { limit })?,
        );
        ancestor = work.ancestors.as_ref();
    }
    Ok(remaining)
}
pub(crate) fn retain_ancestor_work(
    mut ancestor: Option<&Arc<CallbackWork>>,
    delta: u64,
) -> Result<(), DomError> {
    while let Some(work) = ancestor {
        let next = work
            .work
            .load(Ordering::Relaxed)
            .checked_add(delta)
            .ok_or(DomError::RepresentationExhausted)?;
        let limit = work.work_limit.load(Ordering::Relaxed);
        if next > limit {
            return Err(DomError::SourceWorkLimit { limit });
        }
        work.work.store(next, Ordering::Relaxed);
        ancestor = work.ancestors.as_ref();
    }
    Ok(())
}
