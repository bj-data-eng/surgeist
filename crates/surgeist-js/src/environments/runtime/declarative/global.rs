use std::cell::RefCell;

use crate::{JsValue, value::StoredValue};
use surgeist_js_gc::heap::Trace;

#[derive(Debug, Trace)]
pub(crate) struct GlobalEnvironment {
    bindings: RefCell<Vec<Option<StoredValue>>>,
}

impl GlobalEnvironment {
    /// Creates a new `GlobalEnvironment`.
    pub(crate) fn new() -> Self {
        Self {
            bindings: RefCell::new(Vec::new()),
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

    pub(crate) fn binding_count(&self) -> usize {
        self.bindings.borrow().len()
    }

    /// Reserves storage without creating any live binding slots.
    pub(crate) fn prepare_bindings(&self, length: usize) -> crate::JsResult<bool> {
        let mut bindings = self.bindings.borrow_mut();
        let additional = length.saturating_sub(bindings.len());
        bindings
            .try_reserve(additional)
            .map_err(|_| surgeist_js_ast::scope::ScopeStateError::Allocation)?;
        Ok(additional != 0)
    }

    /// Installs slots whose capacity was prepared before the execution transition.
    pub(crate) fn install_prepared_bindings(&self, length: usize) {
        let mut bindings = self.bindings.borrow_mut();
        assert!(
            bindings.capacity() >= length,
            "global binding capacity was not prepared"
        );
        if bindings.len() < length {
            bindings.resize(length, None);
        }
    }

    /// Extends storage to include newly declared global bindings.
    pub(crate) fn extend_bindings(&self, length: usize) {
        let mut bindings = self.bindings.borrow_mut();
        if bindings.len() < length {
            bindings.resize(length, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GlobalEnvironment;
    use crate::JsValue;

    #[test]
    fn preparation_preserves_live_values_and_defers_new_slots_until_installation() {
        let environment = GlobalEnvironment::new();
        environment.extend_bindings(1);
        environment.set(0, JsValue::new(41));
        assert!(environment.prepare_bindings(3).unwrap());
        assert_eq!(environment.get(0), Some(JsValue::new(41)));
        assert_eq!(environment.bindings.borrow().len(), 1);
        environment.install_prepared_bindings(3);
        assert_eq!(environment.get(0), Some(JsValue::new(41)));
        assert_eq!(environment.get(1), None);
        assert_eq!(environment.get(2), None);
        assert!(!environment.prepare_bindings(2).unwrap());
        environment.install_prepared_bindings(2);
        assert_eq!(environment.get(2), None);
    }
}
