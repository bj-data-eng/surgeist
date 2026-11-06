use surgeist_js_gc::{Finalize, GcRefCell, Trace};

use crate::{JsValue, value::StoredValue};

#[derive(Debug, Trace, Finalize)]
pub(crate) struct LexicalEnvironment {
    bindings: GcRefCell<Vec<Option<StoredValue>>>,
}

impl LexicalEnvironment {
    /// Creates a new `LexicalEnvironment`.
    pub(crate) fn new(bindings: u32) -> Self {
        Self {
            bindings: GcRefCell::new(vec![None; bindings as usize]),
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
}
