use std::{
    cell::{OnceCell, RefCell},
    collections::TryReserveError,
};

use surgeist_js_ast::scope::Scope;
use surgeist_js_gc::heap::Trace;

use crate::{
    JsString, JsValue,
    module::{Module, ModuleEdge},
    value::StoredValue,
};

/// Type of accessor used to access an indirect binding.
#[derive(Debug, Clone)]
enum BindingAccessor {
    Identifier(JsString),
    Index(u32),
}

/// An indirect reference to a binding inside an environment.
#[derive(Clone, Debug, Trace)]
struct IndirectBinding {
    module: ModuleEdge,
    #[heap_trace(skip)]
    accessor: RefCell<BindingAccessor>,
}

/// The type of binding a [`ModuleEnvironment`] can contain.
#[derive(Clone, Debug, Trace)]
enum BindingType {
    Direct(Option<StoredValue>),
    Indirect(IndirectBinding),
}

/// A [**Module Environment Record**][spec].
///
/// Module environments allow referencing bindings inside other environments, in addition
/// to the usual declarative environment functionality.
///
///
/// [spec]: https://tc39.es/ecma262/#sec-module-environment-records
#[derive(Debug, Trace)]
pub(crate) struct ModuleEnvironment {
    bindings: RefCell<Vec<BindingType>>,

    #[heap_trace(skip)]
    compile: OnceCell<Scope>,
}

impl ModuleEnvironment {
    /// Allocates private binding storage before any scope is installed.
    pub(crate) fn prepare(bindings: u32) -> Result<Self, TryReserveError> {
        let mut values = Vec::new();
        values.try_reserve_exact(bindings as usize)?;
        values.resize_with(bindings as usize, || BindingType::Direct(None));
        Ok(Self {
            bindings: RefCell::new(values),
            compile: OnceCell::new(),
        })
    }

    /// The publication owner calls this once, before exposing the environment.
    pub(crate) fn install_scope(&self, scope: Scope) {
        self.compile
            .set(scope)
            .expect("private module scope installs exactly once");
    }

    /// Gets the compile time environment of this module environment.
    pub(crate) fn compile(&self) -> &Scope {
        self.compile
            .get()
            .expect("published module environments retain installed scope authority")
    }

    /// Get the binding value from the environment by it's index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range or not initialized.
    #[track_caller]
    pub(crate) fn get(&self, index: u32) -> Option<JsValue> {
        let bindings = self.bindings.borrow();

        match &bindings[index as usize] {
            BindingType::Direct(v) => v.as_ref().map(StoredValue::to_value),
            BindingType::Indirect(IndirectBinding { module, accessor }) => {
                let env = Module::from_edge(module.clone()).environment()?;
                let binding = accessor.borrow().clone();

                match binding {
                    BindingAccessor::Identifier(name) => {
                        let index = env
                            .kind()
                            .as_module()
                            .expect("must be module environment")
                            .compile()
                            .get_binding(&name)
                            .expect("linking must ensure the binding exists");

                        let value = env.get(index.binding_index())?;

                        *accessor.borrow_mut() = BindingAccessor::Index(index.binding_index());

                        Some(value)
                    }
                    BindingAccessor::Index(index) => env.get(index),
                }
            }
        }
    }

    /// Sets the binding value from the environment by index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    pub(crate) fn set(&self, index: u32, value: JsValue) {
        let mut bindings = self.bindings.borrow_mut();

        match &mut bindings[index as usize] {
            BindingType::Direct(v) => *v = Some(value.into_stored()),
            BindingType::Indirect(_) => {
                panic!("cannot modify indirect references to other environments")
            }
        }
    }

    /// Creates an indirect binding reference to another environment binding.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    pub(crate) fn set_indirect(&self, index: u32, target_module: Module, target_binding: JsString) {
        let mut bindings = self.bindings.borrow_mut();

        bindings[index as usize] = BindingType::Indirect(IndirectBinding {
            module: target_module.edge(),
            accessor: RefCell::new(BindingAccessor::Identifier(target_binding)),
        });
    }
}
