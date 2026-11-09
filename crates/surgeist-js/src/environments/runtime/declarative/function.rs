use std::cell::RefCell;
use surgeist_js_ast::scope::Scope;
use surgeist_js_gc::heap::Trace;

use crate::{
    JsNativeError, JsObject, JsResult, JsValue, builtins::function::OrdinaryFunction,
    object::ObjectEdge, value::StoredValue,
};

#[derive(Debug, Trace)]
pub(crate) struct FunctionEnvironment {
    bindings: RefCell<Vec<Option<StoredValue>>>,
    #[heap_trace(skip)]
    deletable_bindings: RefCell<Vec<bool>>,
    #[heap_trace(skip)]
    deleted_bindings: RefCell<Vec<bool>>,
    slots: Box<FunctionSlots>,

    #[heap_trace(skip)]
    scope: Scope,
}

impl FunctionEnvironment {
    /// Creates a new `FunctionEnvironment`.
    pub(crate) fn new(bindings_count: u32, slots: FunctionSlots, scope: Scope) -> Self {
        Self {
            bindings: RefCell::new(vec![None; bindings_count as usize]),
            deletable_bindings: RefCell::new(vec![false; bindings_count as usize]),
            deleted_bindings: RefCell::new(vec![false; bindings_count as usize]),
            slots: Box::new(slots),
            scope,
        }
    }

    /// Gets the slots of this function environment.
    pub(crate) const fn slots(&self) -> &FunctionSlots {
        &self.slots
    }

    /// Gets the compile time environment of this function environment.
    pub(crate) const fn compile(&self) -> &Scope {
        &self.scope
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

    /// Reserve eval's appended slots without changing existing values or lengths.
    pub(crate) fn prepare_eval_bindings(&self, count: u32) -> JsResult<usize> {
        let count = count as usize;
        let mut bindings = self.bindings.borrow_mut();
        let expected = bindings.len();
        bindings
            .try_reserve(count.saturating_sub(expected))
            .map_err(|_| surgeist_js_ast::scope::ScopeStateError::Allocation)?;
        let mut deletable = self.deletable_bindings.borrow_mut();
        let len = deletable.len();
        deletable
            .try_reserve(count.saturating_sub(len))
            .map_err(|_| surgeist_js_ast::scope::ScopeStateError::Allocation)?;
        let mut deleted = self.deleted_bindings.borrow_mut();
        let len = deleted.len();
        deleted
            .try_reserve(count.saturating_sub(len))
            .map_err(|_| surgeist_js_ast::scope::ScopeStateError::Allocation)?;
        Ok(expected)
    }

    pub(crate) fn validate_eval_bindings(&self, expected: usize) -> JsResult<()> {
        if self.bindings.borrow().len() != expected
            || self.deletable_bindings.borrow().len() != expected
            || self.deleted_bindings.borrow().len() != expected
        {
            return Err(surgeist_js_ast::scope::ScopeStateError::Conflict.into());
        }
        Ok(())
    }

    /// Append defaults only; callback-owned writes to existing slots survive.
    pub(crate) fn install_eval_bindings(&self, count: u32) {
        let count = count as usize;
        let mut bindings = self.bindings.borrow_mut();
        let previous = bindings.len();
        if previous >= count {
            return;
        }
        assert!(
            bindings.capacity() >= count,
            "eval capacity was not prepared"
        );
        bindings.resize(count, None);
        let mut deletable = self.deletable_bindings.borrow_mut();
        assert!(
            deletable.capacity() >= count,
            "eval capacity was not prepared"
        );
        deletable.resize(count, true);
        let mut deleted = self.deleted_bindings.borrow_mut();
        assert!(
            deleted.capacity() >= count,
            "eval capacity was not prepared"
        );
        deleted.resize(count, false);
    }

    pub(crate) fn is_deleted_binding(&self, index: u32) -> bool {
        self.deleted_bindings
            .borrow()
            .get(index as usize)
            .copied()
            .unwrap_or_default()
    }

    #[track_caller]
    pub(crate) fn restore_deleted_binding(&self, index: u32) {
        let index = index as usize;
        if let Some(deleted) = self.deleted_bindings.borrow_mut().get_mut(index) {
            *deleted = false;
        }
    }

    #[track_caller]
    pub(crate) fn delete_binding(&self, index: u32) -> bool {
        let index = index as usize;

        if self
            .deleted_bindings
            .borrow()
            .get(index)
            .copied()
            .unwrap_or_default()
        {
            return true;
        }

        if !self
            .deletable_bindings
            .borrow()
            .get(index)
            .copied()
            .unwrap_or_default()
        {
            return false;
        }

        self.bindings.borrow_mut()[index] = None;
        self.deleted_bindings.borrow_mut()[index] = true;
        true
    }

    /// `BindThisValue`
    ///
    /// Sets the given value as the `this` binding of the environment.
    /// Returns `false` if the `this` binding has already been initialized.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bindthisvalue
    pub(crate) fn bind_this_value(&self, value: JsObject) -> JsResult<()> {
        let mut this = self.slots.this.borrow_mut();
        match &*this {
            ThisBindingStatus::Lexical => {
                unreachable!("1. Assert: envRec.[[ThisBindingStatus]] is not lexical.")
            }
            ThisBindingStatus::Initialized(_) => {
                // 2. If envRec.[[ThisBindingStatus]] is initialized, throw a ReferenceError exception.
                return Err(JsNativeError::reference()
                    .with_message("cannot reinitialize `this` binding")
                    .into());
            }
            ThisBindingStatus::Uninitialized => {
                // 3. Set envRec.[[ThisValue]] to V.
                // 4. Set envRec.[[ThisBindingStatus]] to initialized.
                *this = ThisBindingStatus::initialized(value.into());
            }
        }

        // 5. Return V.
        Ok(())
    }

    /// `HasSuperBinding`
    ///
    /// Returns `true` if the environment has a `super` binding.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-hassuperbinding
    ///
    /// Returns the exact access refusal when the function cannot be inspected.
    pub(crate) fn has_super_binding(&self) -> crate::JsResult<bool> {
        // 1.If envRec.[[ThisBindingStatus]] is lexical, return false.
        if matches!(&*self.slots.this.borrow(), ThisBindingStatus::Lexical) {
            return Ok(false);
        }

        // 2. If envRec.[[FunctionObject]].[[HomeObject]] is undefined, return false; otherwise, return true.
        Ok(self
            .slots
            .function_object()
            .downcast_ref::<OrdinaryFunction>()?
            .ok_or(crate::error::ObjectAccessError::WrongNativeBrand)?
            .get_home_object()
            .is_some())
    }

    /// `HasThisBinding`
    ///
    /// Returns `true` if the environment has a `this` binding.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-hasthisbinding
    pub(crate) fn has_this_binding(&self) -> bool {
        // 1. If envRec.[[ThisBindingStatus]] is lexical, return false; otherwise, return true.
        !matches!(&*self.slots.this.borrow(), ThisBindingStatus::Lexical)
    }

    /// `GetThisBinding`
    ///
    /// Returns the `this` binding of the current environment.
    ///
    /// Differs slightly from the spec where lexical this (arrow functions) doesn't get asserted,
    /// but instead is returned as `None`.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-getthisbinding
    pub(crate) fn get_this_binding(&self) -> JsResult<Option<JsValue>> {
        match &*self.slots.this.borrow() {
            ThisBindingStatus::Lexical => Ok(None),
            // 2. If envRec.[[ThisBindingStatus]] is uninitialized, throw a ReferenceError exception.
            ThisBindingStatus::Uninitialized => Err(JsNativeError::reference()
                .with_message(
                    "Must call super constructor in derived \
                    class before accessing 'this' or returning from derived constructor",
                )
                .into()),
            // 3. Return envRec.[[ThisValue]].
            ThisBindingStatus::Initialized(this) => Ok(Some(this.to_value())),
        }
    }
}

/// Describes the status of a `this` binding in function environments.
#[derive(Clone, Debug, Trace)]
pub(crate) enum ThisBindingStatus {
    /// Function doesn't have a `this` binding. (arrow functions and async arrow functions)
    Lexical,
    /// Function has a `this` binding, but is uninitialized. (derived constructors)
    Uninitialized,
    /// Function has an initialized `this` binding. (base constructors and most callable objects)
    Initialized(StoredValue),
}

impl ThisBindingStatus {
    pub(crate) fn initialized(value: JsValue) -> Self {
        Self::Initialized(value.into_stored())
    }
}

/// Holds the internal slots of a function environment.
#[derive(Clone, Debug, Trace)]
pub(crate) struct FunctionSlots {
    /// The `[[ThisValue]]` and `[[ThisBindingStatus]]`  internal slots.
    this: RefCell<ThisBindingStatus>,

    /// The `[[FunctionObject]]` internal slot.
    function_object: ObjectEdge,

    /// The `[[NewTarget]]` internal slot.
    new_target: Option<ObjectEdge>,
}

impl FunctionSlots {
    /// Creates a new `FunctionSluts`.
    pub(crate) fn new(
        this: ThisBindingStatus,
        function_object: JsObject,
        new_target: Option<JsObject>,
    ) -> Self {
        Self {
            this: RefCell::new(this),
            function_object: function_object.into_record(),
            new_target: new_target.map(JsObject::into_record),
        }
    }

    /// Returns the value of the `[[FunctionObject]]` internal slot.
    pub(crate) fn function_object(&self) -> JsObject {
        JsObject::from_inner(self.function_object.clone())
    }

    /// Returns the value of the `[[NewTarget]]` internal slot.
    pub(crate) fn new_target(&self) -> Option<JsObject> {
        self.new_target.clone().map(JsObject::from_inner)
    }
}
