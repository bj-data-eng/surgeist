//! Admission joins for the actual external shared task owner.
use super::*;
use crate::manipulation::{Binding, DomManipulationTaskRetention, DomManipulationTaskSource};
use std::sync::atomic::AtomicUsize;
impl Dom {
    /// Whether a task source was already installed, including an expired one.
    pub fn has_manipulation_task_source(&self) -> bool {
        self.manipulation_tasks.is_some()
    }
    /// Install the one trusted task source while the canonical owner is idle.
    pub fn bind_manipulation_task_source(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<DomManipulationTaskSource, DomError> {
        self.require_sync_source_idle()?;
        check_cancel(cancel)?;
        if self.has_manipulation_task_source() {
            return Err(DomError::InvalidSourceOperation);
        }
        let binding = Arc::new(Binding {
            owner: self.owner,
            bytes: AtomicUsize::new(0),
        });
        self.manipulation_tasks = Some(Arc::downgrade(&binding));
        Ok(DomManipulationTaskSource { binding })
    }
}
impl DomManipulationTaskSource {
    pub(in crate::store) fn qualify(&self, dom: &Dom) -> Result<(), DomError> {
        if self.binding.owner != dom.owner
            || !dom
                .manipulation_tasks
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .is_some_and(|actual| Arc::ptr_eq(&actual, &self.binding))
        {
            return Err(DomError::InvalidSourceOperation);
        }
        Ok(())
    }
    /// Admit actual owner storage before allocating it, under every current
    /// Source ancestor and the current private segment's original limits.
    pub fn retain_payload(
        &self,
        dom: &mut Dom,
        bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<DomManipulationTaskRetention, DomError> {
        self.qualify(dom)?;
        check_cancel(cancel)?;
        self.binding
            .bytes
            .load(Ordering::Relaxed)
            .checked_add(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        self.binding.bytes.fetch_add(bytes, Ordering::Relaxed);
        let retained = DomManipulationTaskRetention {
            binding: self.binding.clone(),
            bytes,
        };
        dom.admit_source_owner_payload(cancel)?;
        Ok(retained)
    }
}
