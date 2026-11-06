use crate::{JsValue, value::StoredValue};
use surgeist_js_gc::{Finalize, GcRefCell, Trace};

#[derive(Debug, Trace, Finalize)]
pub(crate) struct GlobalEnvironment {
    bindings: GcRefCell<Vec<Option<StoredValue>>>,
}

impl GlobalEnvironment {
    /// Creates a new `GlobalEnvironment`.
    pub(crate) fn new() -> Self {
        Self {
            bindings: GcRefCell::new(Vec::new()),
        }
    }

    /// Gets the binding value from the environment by it's index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range or not initialized.
    #[track_caller]
    pub(crate) fn get(&self, index: u32) -> Option<JsValue> {
        self.bindings.borrow()[index as usize]
            .as_ref()
            .map(StoredValue::to_value)
    }

    /// Sets the binding value from the environment by index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    pub(crate) fn set(&self, index: u32, value: JsValue) {
        self.bindings.borrow_mut()[index as usize] = Some(value.into_stored());
    }

    /// Extends storage to include newly declared global bindings.
    pub(crate) fn extend_bindings(&self, length: usize) {
        let mut bindings = self.bindings.borrow_mut();
        if bindings.len() < length {
            bindings.resize(length, None);
        }
    }
}
