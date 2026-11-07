use std::cell::RefCell;

use surgeist_js_gc::heap::Trace;

use crate::{JsValue, value::StoredValue};

#[derive(Debug, Trace)]
pub(crate) struct LexicalEnvironment {
    bindings: RefCell<Vec<Option<StoredValue>>>,
}

impl LexicalEnvironment {
    /// Creates a new `LexicalEnvironment`.
    pub(crate) fn new(bindings: u32) -> Self {
        Self {
            bindings: RefCell::new(vec![None; bindings as usize]),
        }
    }

    /// Allocate private eval slots without publishing a runtime environment.
    pub(crate) fn try_new(bindings: u32) -> crate::JsResult<Self> {
        let mut values = Vec::new();
        values
            .try_reserve(bindings as usize)
            .map_err(|_| surgeist_js_ast::scope::ScopeStateError::Allocation)?;
        values.resize(bindings as usize, None);
        Ok(Self {
            bindings: RefCell::new(values),
        })
    }

    /// Gets the binding value from the environment by it's index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range or not initialized.
    #[track_caller]
    pub(crate) fn get(&self, index: u32) -> Option<JsValue> {
        self.get_stored(index).map(StoredValue::into_value)
    }

    #[track_caller]
    pub(crate) fn get_stored(&self, index: u32) -> Option<StoredValue> {
        self.bindings.borrow()[index as usize].clone()
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
