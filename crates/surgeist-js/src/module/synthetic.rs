use crate::ContextAccess;
use std::{cell::RefCell, rc::Rc};

use rustc_hash::FxHashSet;
use surgeist_js_ast::scope::{BindingLocator, PreparedScopes, ScopeHandle, ScopeWork};
use surgeist_js_gc::heap::Trace;

use super::{
    BindingName, ResolveExportError, ResolvedBinding,
    metadata::{Control, Index, Work},
};
use crate::{
    Context, JsExpect, JsNativeError, JsResult, JsString, JsValue, Module, SpannedSourceText,
    builtins::promise::ResolvingFunctions,
    bytecompiler::{
        ByteCompiler,
        control::{CompilerWork, payload_bytes},
    },
    class::{Class, ClassBuilder},
    environments::{
        DeclarativeEnvironmentEdge, DeclarativeEnvironmentRef, EnvironmentStack,
        StoredEnvironmentStack,
    },
    js_string,
    native_function::{NativeCapture, NativeFunction, StoredNativeFunction},
    object::{JsObject, JsPromise, ObjectEdge},
    vm::{ActiveRunnable, CallFrame, CodeBlock, source_info::SourcePath},
};

type InitializerFn<T> = fn(&SyntheticModule, &T, ContextAccess<'_>) -> JsResult<()>;

trait Initializer {
    fn stored_layout(&self) -> usize;
    fn store(&self) -> Box<dyn StoredInitializer>;
}

trait StoredInitializer: Trace {
    fn call(&self, module: &SyntheticModule, context: &mut Context) -> JsResult<()>;
}

struct Callback<T> {
    callback: InitializerFn<T>,
    captures: T,
}

#[derive(Trace)]
struct StoredCallback<T: NativeCapture> {
    #[heap_trace(skip)]
    callback: InitializerFn<T>,
    captures: T::Stored,
}

impl<T: NativeCapture> Initializer for Callback<T> {
    fn stored_layout(&self) -> usize {
        std::mem::size_of::<StoredCallback<T>>()
    }
    fn store(&self) -> Box<dyn StoredInitializer> {
        Box::new(StoredCallback::<T> {
            callback: self.callback,
            captures: self.captures.store(),
        })
    }
}

impl<T: NativeCapture> StoredInitializer for StoredCallback<T> {
    fn call(&self, module: &SyntheticModule, context: &mut Context) -> JsResult<()> {
        (self.callback)(module, &T::restore(&self.captures)?, context.access())
    }
}

#[derive(Trace)]
struct NativeExportsData {
    entries: Vec<(JsString, StoredNativeFunction)>,
}
struct NativeExportsInitializer {
    payload: Rc<NativeExportsData>,
    // Construction-only roots, never installed in the stored module payload.
    _functions: Vec<NativeFunction>,
}
#[derive(Trace)]
struct StoredNativeExports {
    payload: Rc<NativeExportsData>,
}
impl Initializer for NativeExportsInitializer {
    fn stored_layout(&self) -> usize {
        std::mem::size_of::<StoredNativeExports>()
    }
    fn store(&self) -> Box<dyn StoredInitializer> {
        Box::new(StoredNativeExports {
            payload: self.payload.clone(),
        })
    }
}
impl StoredInitializer for StoredNativeExports {
    fn call(&self, module: &SyntheticModule, context: &mut Context) -> JsResult<()> {
        // Preserve the original Vec capture's restore-before-execution order.
        // Restoration/instantiation belongs to evaluation, outside construction.
        self.payload
            .entries
            .len()
            .checked_mul(std::mem::size_of::<NativeFunction>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or(crate::error::RootingError::SizeOverflow)?;
        let mut functions = Vec::new();
        functions
            .try_reserve_exact(self.payload.entries.len())
            .map_err(|_| crate::error::RootingError::Allocation)?;
        for (_, function) in &self.payload.entries {
            functions.push(NativeFunction::restore(function)?);
        }
        for ((name, _), function) in self.payload.entries.iter().zip(functions) {
            module.set_export(name, function.to_js_function(context.realm())?.into())?;
        }
        Ok(())
    }
}
pub(super) fn native_exports(
    entries: Vec<(JsString, StoredNativeFunction)>,
    functions: Vec<NativeFunction>,
    control: &mut Control<'_>,
) -> JsResult<SyntheticModuleInitializer> {
    control.header::<NativeExportsData>()?;
    let payload = Rc::new(NativeExportsData { entries });
    control.header::<NativeExportsInitializer>()?;
    Ok(SyntheticModuleInitializer {
        inner: Rc::new(NativeExportsInitializer {
            payload,
            _functions: functions,
        }),
    })
}

/// The initializing steps of a synthetic module.
///
/// Pass owned callback state as explicit captures so the collector can trace it.
#[derive(Clone)]
pub struct SyntheticModuleInitializer {
    inner: Rc<dyn Initializer>,
}

impl std::fmt::Debug for SyntheticModuleInitializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyntheticModuleInitializer")
            .finish_non_exhaustive()
    }
}

impl SyntheticModuleInitializer {
    pub(super) fn prepare<T: NativeCapture>(
        callback: InitializerFn<T>,
        captures: T,
        control: &mut Control<'_>,
    ) -> JsResult<Self> {
        control.header::<Callback<T>>()?;
        Ok(Self::from_fn_with_captures(callback, captures))
    }

    /// Creates a callback with all stored state in its traceable captures.
    ///
    /// The callback receives a borrowed capability for the executing context.
    /// Reborrow it for nested operations without transferring the context owner.
    ///
    /// ```compile_fail
    /// use surgeist_js::module::SyntheticModuleInitializer;
    ///
    /// let hidden_state = 7;
    /// SyntheticModuleInitializer::from_fn_with_captures(
    ///     move |_, (), _| {
    ///         assert_eq!(hidden_state, 7);
    ///         Ok(())
    ///     },
    ///     (),
    /// );
    /// ```
    pub fn from_fn_with_captures<T: NativeCapture>(
        callback: fn(&SyntheticModule, &T, ContextAccess<'_>) -> JsResult<()>,
        captures: T,
    ) -> Self {
        Self {
            inner: Rc::new(Callback { callback, captures }),
        }
    }
}

/// Current status of a [`SyntheticModule`].
#[derive(Debug, Trace, Default)]
enum ModuleStatus {
    #[default]
    Unlinked,
    Preparing {
        #[heap_trace(skip)]
        attempt: Rc<()>,
    },
    Linked {
        environment: DeclarativeEnvironmentEdge,
        environments: StoredEnvironmentStack,
        #[heap_trace(skip)]
        codeblock: Rc<CodeBlock>,
    },
    Evaluated {
        environment: DeclarativeEnvironmentEdge,
        promise: ObjectEdge,
    },
}

impl ModuleStatus {
    /// Transition from one state to another, taking the current state by value to move data
    /// between states.
    fn transition<F>(&mut self, f: F)
    where
        F: FnOnce(Self) -> Self,
    {
        *self = f(std::mem::take(self));
    }
}

/// ECMAScript's [**Synthetic Module Records**][spec].
///
/// [spec]: https://tc39.es/proposal-json-modules/#sec-synthetic-module-records
#[derive(Trace)]
pub struct SyntheticModule {
    #[heap_trace(skip)]
    export_names: Index<JsString>,
    eval_steps: Box<dyn StoredInitializer>,
    state: RefCell<ModuleStatus>,
}

pub(super) struct SyntheticLinkCode {
    pub(super) scope: ScopeHandle,
    pub(super) code: Rc<CodeBlock>,
    pub(super) exports: Vec<BindingLocator>,
}

impl std::fmt::Debug for SyntheticModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyntheticModule")
            .field("export_names", &self.export_names)
            .finish_non_exhaustive()
    }
}

impl SyntheticModule {
    /// Abstract operation [`SetSyntheticModuleExport ( module, exportName, exportValue )`][spec].
    ///
    /// Sets or changes the exported value for `exportName` in the synthetic module.
    ///
    /// # Note
    ///
    /// The default export corresponds to the name `"default"`, but note that it needs to
    /// be passed to the list of exported names in [`Module::synthetic`] beforehand.
    ///
    /// # Errors
    ///
    /// Returns an error if the module is unlinked, the export name was not declared, or the
    /// export value belongs to another runtime.
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-createsyntheticmodule
    pub fn set_export(&self, export_name: &JsString, export_value: JsValue) -> JsResult<()> {
        let env = self.environment().ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "cannot set name `{}` in an unlinked synthetic module",
                export_name.to_std_string_escaped()
            ))
        })?;
        let locator = env
            .kind()
            .as_module()
            .js_expect("must be module environment")?
            .compile()
            .get_binding(export_name)
            .ok_or_else(|| {
                JsNativeError::reference().with_message(format!(
                    "cannot set name `{}` which was not included in the list of exports",
                    export_name.to_std_string_escaped()
                ))
            })?;
        crate::Runtime {
            heap: env.runtime().clone(),
        }
        .check_value(&export_value)?;
        env.set(locator.binding_index(), export_value);

        Ok(())
    }

    /// Sets or changes the exported value for `C::NAME` in the synthetic module
    /// to the Class's constructor.
    pub fn export_class<C: Class>(&self, mut context: ContextAccess<'_>) -> JsResult<()> {
        self.export_named_class::<C>(&JsString::from(C::NAME), context.reborrow())
    }

    /// Sets or changes the exported value for `export_name` in the synthetic module
    /// to the Class's constructor.
    pub fn export_named_class<C: Class>(
        &self,
        export_name: &JsString,
        mut context: ContextAccess<'_>,
    ) -> JsResult<()> {
        let mut class_builder = ClassBuilder::new::<C>(context.reborrow());
        C::init(&mut class_builder)?;

        let class = class_builder.build()?;

        self.set_export(export_name, class.constructor().into())?;
        Ok(())
    }

    /// Creates a new synthetic module.
    pub(super) fn new(
        names: Index<JsString>,
        eval_steps: &SyntheticModuleInitializer,
        control: &mut Control<'_>,
    ) -> JsResult<Self> {
        control.header::<Self>()?;
        control.admit(Work::Record)?;
        control.admit(Work::Capacity(eval_steps.inner.stored_layout()))?;
        control.admit(Work::Checkpoint)?;
        // NativeCapture custody conversions are non-reentrant; arbitrary
        // conversion internals remain opaque, outside metadata tariffs.
        let eval_steps = eval_steps.inner.store();
        control.admit(Work::Checkpoint)?;
        Ok(Self {
            export_names: names,
            eval_steps,
            state: RefCell::default(),
        })
    }

    /// Concrete method [`LoadRequestedModules ( )`][spec].
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-smr-LoadRequestedModules
    pub(super) fn load(context: &mut Context) -> JsResult<JsPromise> {
        JsPromise::resolve(JsValue::undefined(), context.access())
    }

    /// Concrete method [`GetExportedNames ( [ exportStarSet ] )`][spec].
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-smr-getexportednames
    pub(super) fn get_exported_names(&self) -> FxHashSet<JsString> {
        // 1. Return module.[[ExportNames]].
        self.export_names.iter().cloned().collect()
    }

    /// Concrete method [`ResolveExport ( exportName )`][spec]
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-smr-resolveexport
    #[allow(clippy::mutable_key_type)]
    pub(super) fn resolve_export(
        &self,
        module_self: &Module,
        export_name: &JsString,
    ) -> Result<ResolvedBinding, ResolveExportError> {
        if self.export_names.contains(export_name) {
            // 2. Return ResolvedBinding Record { [[Module]]: module, [[BindingName]]: exportName }.
            Ok(ResolvedBinding {
                module: module_self.clone(),
                binding_name: BindingName::Name(export_name.clone()),
            })
        } else {
            // 1. If module.[[ExportNames]] does not contain exportName, return null.
            Err(ResolveExportError::NotFound)
        }
    }

    pub(super) fn begin_link(&self, identity: &Rc<()>) -> JsResult<bool> {
        let mut state = self.state.borrow_mut();
        match &*state {
            ModuleStatus::Unlinked => {
                *state = ModuleStatus::Preparing {
                    attempt: identity.clone(),
                };
                Ok(true)
            }
            ModuleStatus::Preparing { attempt } if Rc::ptr_eq(attempt, identity) => Ok(false),
            ModuleStatus::Preparing { .. } => Err(JsNativeError::typ()
                .with_message("module linking is already in progress")
                .into()),
            ModuleStatus::Linked { .. } | ModuleStatus::Evaluated { .. } => Ok(false),
        }
    }

    pub(super) fn abort_link(&self, identity: &Rc<()>) {
        let mut state = self.state.borrow_mut();
        if matches!(&*state, ModuleStatus::Preparing { attempt } if Rc::ptr_eq(attempt, identity)) {
            *state = ModuleStatus::Unlinked;
        }
    }

    pub(super) fn prepare_link(
        &self,
        module: &Module,
        scopes: &mut PreparedScopes,
        context: &mut Context,
    ) -> JsResult<SyntheticLinkCode> {
        let root = module.realm().scope().handle();
        let (scope, exports) = context.with_scope_admission(|mut admission| {
            if let Some(admit) = admission.as_mut() {
                admit(ScopeWork::Capacity {
                    elements: self.export_names.len(),
                })
                .map_err(surgeist_js_ast::scope::ScopeAnalysisError::Refused)?;
            }
            let mut names = Vec::new();
            names
                .try_reserve(self.export_names.len())
                .map_err(|_| surgeist_js_ast::scope::ScopeStateError::Allocation)
                .map_err(surgeist_js_ast::scope::ScopeAnalysisError::State)?;
            names.extend(self.export_names.iter().cloned());
            scopes.with_access(admission, |access| {
                access.create_module_scope(&root, &names)
            })
        })?;
        let (code, extension) = context.with_compilation_work(|mut control| {
            let _depth = control.enter()?;
            control.admit(CompilerWork::MetadataVisit)?;
            let mut extension = control.with_scope_admission(|admission| {
                let owner = scopes;
                owner
                    .begin_extension(admission)
                    .map_err(crate::JsError::from)
            })?;
            let mut interner = surgeist_js_interner::Interner::new();
            let compiler = ByteCompiler::new(
                js_string!("<synthetic>"),
                true,
                false,
                scope.clone(),
                scope.clone(),
                false,
                false,
                &mut interner,
                false,
                SpannedSourceText::new_empty(),
                SourcePath::None,
                &mut extension,
                control.reborrow(),
            )?;
            let code = compiler.finish(None)?;
            let bytes = payload_bytes::<CodeBlock>(1)?;
            control.admit(CompilerWork::Capacity { bytes })?;
            control.admit(CompilerWork::Copy { bytes })?;
            control.admit(CompilerWork::MetadataWrite)?;
            Ok((Rc::new(code), extension))
        })?;
        extension.commit_private();
        Ok(SyntheticLinkCode {
            scope,
            code,
            exports,
        })
    }

    pub(super) fn finish_link(
        &self,
        environment: DeclarativeEnvironmentRef,
        environments: EnvironmentStack,
        codeblock: Rc<CodeBlock>,
    ) {
        *self.state.borrow_mut() = ModuleStatus::Linked {
            environment: environment.record_ref(),
            environments: environments.stored(),
            codeblock,
        };
    }

    /// Concrete method [`Evaluate ( )`][spec].
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-smr-Evaluate
    pub(super) fn evaluate(
        &self,
        module_self: &Module,
        context: &mut Context,
    ) -> JsResult<JsPromise> {
        let (environments, codeblock) = match &*self.state.borrow() {
            ModuleStatus::Unlinked | ModuleStatus::Preparing { .. } => {
                let (promise, ResolvingFunctions { reject, .. }) =
                    JsPromise::new_pending(context.access());
                reject
                    .call(
                        &JsValue::undefined(),
                        &[JsNativeError::typ()
                            .with_message("cannot evaluate unlinked synthetic module")
                            .into_opaque(context.access())?
                            .into()],
                        context.access(),
                    )
                    .js_expect("native resolving functions cannot throw")?;
                return Ok(promise);
            }
            ModuleStatus::Linked {
                environments,
                codeblock,
                ..
            } => (environments.to_stack()?, codeblock.clone()),
            ModuleStatus::Evaluated { promise, .. } => {
                return JsPromise::from_object(JsObject::from_inner(promise.clone()));
            }
        };
        // 1. Let moduleContext be a new ECMAScript code execution context.

        let realm = module_self.realm();

        let env_fp = environments.len() as u32;
        let callframe = CallFrame::host_start(
            codeblock,
            // 4. Set the ScriptOrModule of moduleContext to module.
            Some(ActiveRunnable::Module(module_self.clone())),
            // 5. Set the VariableEnvironment of moduleContext to module.[[Environment]].
            // 6. Set the LexicalEnvironment of moduleContext to module.[[Environment]].
            environments,
            // 3. Set the Realm of moduleContext to module.[[Realm]].
            realm,
        )
        .with_env_fp(env_fp);

        // 2. Set the Function of moduleContext to null.
        // 7. Suspend the currently running execution context.
        // 8. Push moduleContext on to the execution context stack; moduleContext is now the running execution context.
        context.vm.push_host_start(callframe)?;

        // 9. Let steps be module.[[EvaluationSteps]].
        // 10. Let result be Completion(steps(module)).
        let result = self.eval_steps.call(self, context);

        // 11. Suspend moduleContext and remove it from the execution context stack.
        // 12. Resume the context that is now on the top of the execution context stack as the running execution context.
        let _frame = context
            .vm
            .pop_frame()
            .js_expect("there should be a frame")?;

        // 13. Let pc be ! NewPromiseCapability(%Promise%).
        let (promise, ResolvingFunctions { resolve, reject }) =
            JsPromise::new_pending(context.access());

        match result {
            // 15. Perform ! pc.[[Resolve]](result).
            Ok(()) => resolve.call(&JsValue::undefined(), &[], context.access()),
            // 14. IfAbruptRejectPromise(result, pc).
            Err(error) => {
                let error = context.runtime().admit_error(error);
                reject.call(
                    &JsValue::undefined(),
                    &[error.into_opaque(context.access())?],
                    context.access(),
                )
            }
        }
        .js_expect("default resolving functions cannot throw")?;

        self.state.borrow_mut().transition(|state| match state {
            ModuleStatus::Linked { environment, .. } => ModuleStatus::Evaluated {
                environment,
                promise: JsObject::from(promise.clone()).edge(),
            },
            _ => unreachable!("checks above ensure the module is linked"),
        });

        // 16. Return pc.[[Promise]].
        Ok(promise)
    }

    pub(crate) fn environment(&self) -> Option<DeclarativeEnvironmentRef> {
        match &*self.state.borrow() {
            ModuleStatus::Unlinked | ModuleStatus::Preparing { .. } => None,
            ModuleStatus::Linked { environment, .. }
            | ModuleStatus::Evaluated { environment, .. } => Some(
                environment
                    .upgrade()
                    .expect("module environment heap is available")
                    .expect("linked module retains its environment"),
            ),
        }
    }
}

#[cfg(test)]
mod metadata_tests;

#[cfg(test)]
mod header_cost_tests;

#[cfg(test)]
mod native_payload_tests;
