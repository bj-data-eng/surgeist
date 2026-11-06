//! Runtime ownership shared by JavaScript contexts and realms.

use std::num::NonZeroUsize;
use surgeist_js_gc::heap::{AllocationCollection, RuntimeHeap};

/// A JavaScript runtime. Contexts created with the same runtime share object identity.
#[derive(Clone)]
pub struct Runtime {
    pub(crate) heap: RuntimeHeap,
}

impl Default for Runtime {
    fn default() -> Self {
        let heap = RuntimeHeap::default();
        let interval = NonZeroUsize::new(1_024).expect("collection floor is positive");
        heap.set_allocation_collection(AllocationCollection::Adaptive(interval))
            .expect("new runtime is available");
        Self { heap }
    }
}

impl Runtime {
    pub(crate) fn check_value(&self, value: &crate::JsValue) -> crate::JsResult<()> {
        if let Some(object) = value.as_object() {
            object.check_runtime(self)?;
        }
        Ok(())
    }

    pub(crate) fn check_descriptor(
        &self,
        descriptor: &crate::property::PropertyDescriptor,
    ) -> crate::JsResult<()> {
        for value in [descriptor.value(), descriptor.get(), descriptor.set()]
            .into_iter()
            .flatten()
        {
            self.check_value(value)?;
        }
        Ok(())
    }

    pub(crate) fn admit_callback_result(
        &self,
        result: crate::JsResult<crate::JsValue>,
    ) -> crate::JsResult<crate::JsValue> {
        match result {
            Ok(value) => {
                self.check_value(&value)?;
                Ok(value)
            }
            Err(error) => Err(self.admit_error(error)),
        }
    }

    pub(crate) fn admit_error(&self, error: crate::JsError) -> crate::JsError {
        error.admit_runtime(self)
    }

    /// Collects unreachable records on this runtime.
    pub fn collect(&self) -> Result<usize, surgeist_js_gc::heap::AccessError> {
        self.heap.collect()
    }

    /// Returns whether this and another owner share a single heap.
    #[must_use]
    pub fn same_heap(&self, other: &Self) -> bool {
        self.heap.same_heap(&other.heap)
    }
}

impl std::fmt::Debug for Runtime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Runtime").finish_non_exhaustive()
    }
}
