//! The ECMAScript context.

use std::any::Any;
use std::{cell::Cell, path::Path, rc::Rc};

pub use crate::job::ContextId;
pub use access::ContextAccess;
pub use hooks::{DefaultHooks, HostHooks};
pub use host::HostContext;
#[cfg(feature = "intl")]
pub use icu::IcuError;
use intrinsics::Intrinsics;
use surgeist_js_ast::{
    operations::{
        ContainsSymbol, DirectEvalAdmission, all_private_identifiers_valid, contains,
        contains_arguments, try_all_private_identifiers_valid, try_contains,
        try_contains_arguments,
    },
    visitor::{NodeRef, VisitWith},
};
use surgeist_js_interner::Sym;
use surgeist_js_parser::{
    Parser,
    frontend::{FrontendControl, FrontendLimit, FrontendWork},
    source::ReadChar,
};
#[cfg(any(feature = "temporal", feature = "intl"))]
use timezone_provider::{experimental_tzif::ZeroCompiledTzdbProvider, provider::TimeZoneProvider};

use crate::job::Job;
use crate::js_error;
use crate::vm::{CodeBlock, RuntimeLimits};
use crate::{
    HostDefined, JsNativeError, JsResult, JsString, JsValue, Source, builtins,
    class::{Class, ClassBuilder, ClassConstructor},
    error::{EngineError, RuntimeLimitError},
    execution::{Execution, ExecutionOptions, ExecutionOutcome, ExecutionSetupError},
    job::{JobExecutor, RuntimeJobQueues, SimpleJobExecutor},
    js_string,
    module::{IdleModuleLoader, ModuleLoader, SimpleModuleLoader},
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject, shape::RootShape},
    optimizer::OptimizerOptions,
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    script::Script,
    vm::{ActiveRunnable, CallFrame, Vm},
};

pub mod time;
use crate::context::time::StdClock;
pub use time::Clock;

mod access;
mod hooks;
mod host;
mod module_run;
use module_run::ModuleRunFailure;
#[cfg(feature = "intl")]
pub(crate) mod icu;
pub mod intrinsics;

thread_local! {
    static CANNOT_BLOCK_COUNTER: Cell<u64> = const { Cell::new(0) };
}

struct ThreadBlocking(bool);

impl ThreadBlocking {
    fn acquire(can_block: bool) -> JsResult<Self> {
        if can_block {
            if CANNOT_BLOCK_COUNTER.get() > 0 {
                return Err(JsNativeError::typ()
                    .with_message(
                        "a context that can block must be the only active context in its current thread",
                    )
                    .into());
            }
        } else {
            CANNOT_BLOCK_COUNTER.set(CANNOT_BLOCK_COUNTER.get() + 1);
        }
        Ok(Self(can_block))
    }
}

impl Drop for ThreadBlocking {
    fn drop(&mut self) {
        if !self.0 {
            CANNOT_BLOCK_COUNTER.set(CANNOT_BLOCK_COUNTER.get() - 1);
        }
    }
}

/// ECMAScript context. It is the primary way to interact with the runtime.
///
/// Contexts built with the same [`Runtime`](crate::Runtime) can share objects
/// on their owning thread. A default context starts a fresh runtime.
///
/// # Examples
///
/// ## Execute Function of Script File
///
/// ```rust
/// use surgeist_js::{
///     Context, Source, js_string,
///     object::ObjectInitializer,
///     property::{Attribute, PropertyDescriptor},
/// };
///
/// let script = r#"
///     function test(arg1) {
///         if(arg1 != null) {
///             return arg1.x;
///         }
///         return 112233;
///     }
/// "#;
///
/// let mut context = Context::builder().build().unwrap();
///
/// // Populate the script definition to the context.
/// context.eval(Source::from_bytes(script)).unwrap();
///
/// // Create an object that can be used in eval calls.
/// let arg = ObjectInitializer::new(context.access())
///     .property(js_string!("x"), 12, Attribute::READONLY)
///     .build().unwrap();
/// context
///     .register_global_property(js_string!("arg"), arg, Attribute::all())
///     .expect("property shouldn't exist");
///
/// let value = context.eval(Source::from_bytes("test(arg)")).unwrap();
///
/// assert_eq!(value.as_number(), Some(12.0))
/// ```
pub struct Context {
    /// Execute in strict mode,
    strict: bool,

    /// Number of instructions remaining before a forced exit
    #[cfg(feature = "fuzz")]
    pub(crate) instructions_remaining: usize,

    pub(crate) vm: Vm,

    pub(crate) execution: Option<Execution>,
    module_run_failure: Option<ModuleRunFailure>,

    pub(crate) kept_alive: Vec<JsObject>,

    thread_blocking: ThreadBlocking,

    #[cfg(any(feature = "temporal", feature = "intl"))]
    timezone_provider: Box<dyn TimeZoneProvider>,

    /// Intl data provider.
    #[cfg(feature = "intl")]
    intl_provider: icu::IntlProvider,

    host_hooks: Rc<dyn HostHooks>,

    clock: Rc<dyn Clock>,

    job_executor: Rc<dyn JobExecutor>,
    job_lifetime: Rc<crate::job::ContextLifetime>,

    // Active contexts own their built-in executor queues.
    simple_job_queues: Vec<Rc<RuntimeJobQueues>>,

    module_loader: Rc<dyn ModuleLoader>,

    optimizer_options: OptimizerOptions,
    root_shape: RootShape,

    data: HostDefined<dyn Any>,
}

impl Drop for Context {
    fn drop(&mut self) {
        let identity = self.id();
        self.job_lifetime.retire();
        for queue in &self.simple_job_queues {
            queue.retire_owner(&identity);
        }
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Context");

        debug
            .field("realm", &self.vm.frame().realm)
            .field("vm", &self.vm)
            .field("strict", &self.strict)
            .field("job_executor", &"JobExecutor")
            .field("hooks", &"HostHooks")
            .field("clock", &"Clock")
            .field("module_loader", &"ModuleLoader")
            .field("optimizer_options", &self.optimizer_options);

        #[cfg(feature = "intl")]
        debug.field("intl_provider", &self.intl_provider);

        // TODO: Support TimeZoneProvider debug names
        #[cfg(any(feature = "temporal", feature = "intl"))]
        debug.field("timezone_provider", &"TimeZoneProvider");

        debug.finish_non_exhaustive()
    }
}

// ==== Public API ====
impl Context {
    /// Creates a builder for a context and its host services.
    #[must_use]
    pub fn builder() -> ContextBuilder {
        ContextBuilder::default()
    }

    /// Evaluates the given source by compiling down to bytecode, then interpreting the
    /// bytecode into a value.
    ///
    /// # Examples
    /// ```
    /// # use surgeist_js::{Context, Source};
    /// let mut context = Context::builder().build().unwrap();
    ///
    /// let source = Source::from_bytes("1 + 3");
    /// let value = context.eval(source).unwrap();
    ///
    /// assert!(value.is_number());
    /// assert_eq!(value.as_number().unwrap(), 4.0);
    /// ```
    ///
    /// Note that this won't run any scheduled promise jobs; you need to call [`Context::run_jobs`]
    /// on the context or [`JobExecutor::run_jobs`] on the provided queue to run them.
    #[allow(clippy::unit_arg, dropping_copy_types)]
    pub fn eval<R: ReadChar>(&mut self, src: Source<'_, R>) -> JsResult<JsValue> {
        if let Some(stop) = self.execution.as_ref().and_then(Execution::stop) {
            return Err(stop.clone());
        }
        Script::parse(src, None, self.access())?.evaluate(self.access())
    }

    /// Evaluates a script under execution budgets with optional profiling.
    ///
    /// Synchronous reentrant calls to [`Self::eval`] share this budget.
    /// Attempting to start another controlled evaluation in this context
    /// returns [`ExecutionSetupError::AlreadyActive`]. The report accompanies
    /// the result even if the script throws or the budget is exhausted.
    pub fn eval_with_budget<R: ReadChar>(
        &mut self,
        src: Source<'_, R>,
        options: ExecutionOptions,
    ) -> Result<ExecutionOutcome<JsValue>, ExecutionSetupError> {
        self.with_execution(options, |mut context| context.eval(src))
    }

    pub(crate) fn parse_source<R: ReadChar, T>(
        &mut self,
        source: Source<'_, R>,
        parse: impl for<'control> FnOnce(Parser<'control, R>) -> JsResult<T>,
    ) -> JsResult<T> {
        self.with_frontend_execution(|execution, _| match execution {
            Some(execution) => execution.with_frontend_control(|control| {
                parse(Parser::with_control(source, Some(control)))
            }),
            None => parse(Parser::new(source)),
        })
    }

    pub(crate) fn prepare_script_scope(
        &mut self,
        script: surgeist_js_ast::Script,
        scope: &surgeist_js_ast::scope::Scope,
        interner: &surgeist_js_interner::Interner,
    ) -> JsResult<surgeist_js_ast::PreparedScript> {
        self.with_scope_admission(|admission| script.try_prepare(scope, interner, admission))
    }

    pub(crate) fn with_scope_admission<T>(
        &mut self,
        run: impl FnOnce(
            surgeist_js_ast::scope::ScopeAdmission<'_, surgeist_js_parser::frontend::FrontendLimit>,
        ) -> Result<
            T,
            surgeist_js_ast::scope::ScopeAnalysisError<surgeist_js_parser::frontend::FrontendLimit>,
        >,
    ) -> JsResult<T> {
        self.with_scope_work(|admission| run(admission).map_err(crate::JsError::from))
    }

    pub(crate) fn with_optimizer_work<T>(
        &mut self,
        run: impl FnOnce(crate::optimizer::control::OptimizerAdmission<'_>) -> JsResult<T>,
    ) -> JsResult<T> {
        self.with_frontend_execution(|execution, _| match execution {
            Some(execution) => execution.with_optimizer_control(|control| run(Some(control))),
            None => run(None),
        })
    }

    pub(crate) fn with_scope_work<T>(
        &mut self,
        run: impl FnOnce(
            surgeist_js_ast::scope::ScopeAdmission<'_, surgeist_js_parser::frontend::FrontendLimit>,
        ) -> JsResult<T>,
    ) -> JsResult<T> {
        self.with_frontend_execution(|execution, _| match execution {
            Some(execution) => execution.with_scope_control(|control| run(Some(control))),
            None => run(None),
        })
    }

    pub(crate) fn with_compilation_work<T>(
        &mut self,
        run: impl FnOnce(crate::bytecompiler::control::CompilerControl<'_>) -> JsResult<T>,
    ) -> JsResult<T> {
        self.with_frontend_execution(|execution, _| match execution {
            Some(execution) => execution.with_compilation_control(run),
            None => {
                let depth = crate::bytecompiler::control::CompilerDepth::new();
                run(crate::bytecompiler::control::CompilerControl::new(
                    None, &depth,
                ))
            }
        })
    }

    pub(crate) fn compiler_has_object_environment(&mut self) -> JsResult<bool> {
        self.with_frontend_execution(|execution, vm| match execution {
            Some(execution) => execution.with_compilation_control(|mut control| {
                vm.frame().environments.try_has_object_environment(|| {
                    control.admit(crate::bytecompiler::control::CompilerWork::EnvironmentProbe)
                })
            }),
            None => vm
                .frame()
                .environments
                .try_has_object_environment(|| Ok(())),
        })
    }

    pub(crate) fn with_installation_work<T>(
        &mut self,
        run: impl FnOnce(
            surgeist_js_ast::scope::ScopeAdmission<'_, surgeist_js_parser::frontend::FrontendLimit>,
        ) -> JsResult<T>,
    ) -> JsResult<T> {
        self.with_frontend_execution(|execution, _| match execution {
            Some(execution) => execution.with_installation_control(|control| run(Some(control))),
            None => run(None),
        })
    }

    pub(crate) fn early_error_contains(
        &mut self,
        node: &impl VisitWith,
        symbol: ContainsSymbol,
    ) -> JsResult<bool> {
        self.with_frontend_execution(|execution, _| match execution {
            Some(execution) => execution.with_frontend_control(|control| {
                try_contains(node, symbol, || control(FrontendWork::EarlyErrorVisit))
                    .map_err(|limit| RuntimeLimitError::from(limit).into())
            }),
            None => Ok(contains(node, symbol)),
        })
    }

    pub(crate) fn early_error_contains_arguments(
        &mut self,
        node: &impl VisitWith,
    ) -> JsResult<bool> {
        self.with_frontend_execution(|execution, _| match execution {
            Some(execution) => execution.with_frontend_control(|control| {
                try_contains_arguments(node, || control(FrontendWork::EarlyErrorVisit))
                    .map_err(|limit| RuntimeLimitError::from(limit).into())
            }),
            None => Ok(contains_arguments(node)),
        })
    }

    pub(crate) fn with_direct_eval_admission<T>(
        &mut self,
        construct: impl FnOnce(DirectEvalAdmission<'_, FrontendLimit>) -> Result<T, FrontendLimit>,
    ) -> JsResult<T> {
        self.with_frontend_execution(|execution, _| match execution {
            Some(execution) => execution.with_frontend_control(|control| {
                construct(Some(&mut || control(FrontendWork::DirectEvalVisit)))
                    .map_err(|limit| RuntimeLimitError::from(limit).into())
            }),
            None => construct(None).map_err(|limit| RuntimeLimitError::from(limit).into()),
        })
    }

    pub(crate) fn with_name_checks<T>(
        &mut self,
        check: impl FnOnce(Option<&mut FrontendControl<'_>>, &Vm) -> JsResult<T>,
    ) -> JsResult<T> {
        self.with_frontend_execution(|execution, vm| match execution {
            Some(execution) => execution.with_frontend_control(|control| check(Some(control), vm)),
            None => check(None, vm),
        })
    }

    pub(crate) fn all_private_identifiers_valid<'a, N>(
        &mut self,
        node: &'a N,
        private_names: Vec<Sym>,
    ) -> JsResult<bool>
    where
        &'a N: Into<NodeRef<'a>>,
    {
        self.with_name_checks(|control, _| match control {
            Some(control) => try_all_private_identifiers_valid(
                node,
                private_names,
                Some(&mut |work| control(work.into())),
            )
            .map_err(|limit| RuntimeLimitError::from(limit).into()),
            None => Ok(all_private_identifiers_valid(node, private_names)),
        })
    }

    fn with_frontend_execution<T>(
        &mut self,
        run: impl FnOnce(Option<&mut Execution>, &Vm) -> JsResult<T>,
    ) -> JsResult<T> {
        let Some(execution) = &mut self.execution else {
            return run(None, &self.vm);
        };
        if let Some(stop) = execution.stop() {
            return Err(stop.clone());
        }
        let mut result = run(Some(execution), &self.vm);
        if let Err(error) = &mut result {
            self.retain_preparation_error(error);
        }
        result
    }

    /// Retains the first uncatchable preparation failure across Rust reentry.
    pub(crate) fn retain_preparation_error(&mut self, error: &mut crate::JsError) {
        if self.execution.is_none() {
            return;
        }
        if matches!(
            error.as_engine(),
            Some(EngineError::ScopeAnalysis(_) | EngineError::Resource(_))
        ) || matches!(error.as_engine(), Some(EngineError::RuntimeLimit(limit)) if limit.is_frontend_limit())
        {
            self.capture_error_backtrace(error);
            if let Some(execution) = &mut self.execution {
                execution.retain_stop(error);
            }
        }
    }

    /// Register a global property.
    ///
    /// It will return an error if the property is already defined.
    ///
    /// # Example
    /// ```
    /// use surgeist_js::{
    ///     Context, js_string,
    ///     object::ObjectInitializer,
    ///     property::{Attribute, PropertyDescriptor},
    /// };
    ///
    /// let mut context = Context::builder().build().unwrap();
    ///
    /// context
    ///     .register_global_property(
    ///         js_string!("myPrimitiveProperty"),
    ///         10,
    ///         Attribute::all(),
    ///     )
    ///     .expect("property shouldn't exist");
    ///
    /// let object = ObjectInitializer::new(context.access())
    ///     .property(js_string!("x"), 0, Attribute::all())
    ///     .property(js_string!("y"), 1, Attribute::all())
    ///     .build().unwrap();
    /// context
    ///     .register_global_property(
    ///         js_string!("myObjectProperty"),
    ///         object,
    ///         Attribute::all(),
    ///     )
    ///     .expect("property shouldn't exist");
    /// ```
    pub fn register_global_property<K, V>(
        &mut self,
        key: K,
        value: V,
        attribute: Attribute,
    ) -> JsResult<()>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        self.global_object()?.define_property_or_throw(
            key,
            PropertyDescriptor::builder()
                .value(value)
                .writable(attribute.writable())
                .enumerable(attribute.enumerable())
                .configurable(attribute.configurable()),
            self.access(),
        )?;
        Ok(())
    }

    /// Register a global native callable.
    ///
    /// The function will be both `constructable` (call with `new <name>()`) and `callable` (call
    /// with `<name>()`).
    ///
    /// The function will be bound to the global object with `writable`, `non-enumerable`
    /// and `configurable` attributes. The same as when you create a function in JavaScript.
    ///
    /// # Note
    ///
    /// If you wish to only create the function object without binding it to the global object, you
    /// can use the [`FunctionObjectBuilder`] API.
    pub fn register_global_callable(
        &mut self,
        name: JsString,
        length: usize,
        body: NativeFunction,
    ) -> JsResult<()> {
        let function = FunctionObjectBuilder::new(self.realm(), body)
            .name(name.clone())
            .length(length)
            .constructor(true)
            .build()?;

        self.global_object()?.define_property_or_throw(
            name,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
            self.access(),
        )?;
        Ok(())
    }

    /// Register a global native function that is not a constructor.
    ///
    /// The function will be bound to the global object with `writable`, `non-enumerable`
    /// and `configurable` attributes. The same as when you create a function in JavaScript.
    ///
    /// # Note
    ///
    /// The difference to [`Context::register_global_callable`] is, that the function will not be
    /// `constructable`. Usage of the function as a constructor will produce a `TypeError`.
    pub fn register_global_builtin_callable(
        &mut self,
        name: JsString,
        length: usize,
        body: NativeFunction,
    ) -> JsResult<()> {
        let function = FunctionObjectBuilder::new(self.realm(), body)
            .name(name.clone())
            .length(length)
            .constructor(false)
            .build()?;

        self.global_object()?.define_property_or_throw(
            name,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
            self.access(),
        )?;
        Ok(())
    }

    /// Registers a global class `C` in the currently active realm.
    ///
    /// Errors if the class has already been registered.
    ///
    /// # Example
    /// ```ignore
    /// #[derive(Debug)]
    /// struct MyClass;
    ///
    /// impl Class for MyClass {
    ///    // ...
    /// }
    ///
    /// context.register_global_class::<MyClass>()?;
    /// ```
    pub fn register_global_class<C: Class>(&mut self) -> JsResult<()> {
        if self.realm().has_class::<C>()? {
            return Err(JsNativeError::typ()
                .with_message("cannot register a class twice")
                .into());
        }

        let mut class_builder = ClassBuilder::new::<C>(self.access());
        C::init(&mut class_builder)?;

        let class = class_builder.build()?;
        let property = PropertyDescriptor::builder()
            .value(class.constructor())
            .writable(C::ATTRIBUTES.writable())
            .enumerable(C::ATTRIBUTES.enumerable())
            .configurable(C::ATTRIBUTES.configurable());

        self.global_object()?.define_property_or_throw(
            js_string!(C::NAME),
            property,
            self.access(),
        )?;
        self.realm().register_class::<C>(class)?;

        Ok(())
    }

    /// Removes the global class `C` from the currently active realm, returning the constructor
    /// and prototype of the class if `C` was registered.
    ///
    /// # Note
    ///
    /// This makes the constructor return an error on further calls, but note that this won't protect
    /// static properties from being accessed within variables that stored the constructor before being
    /// unregistered.  If you need that functionality, you can use a static accessor that first checks
    /// if the class is registered ([`Context::has_global_class`]) before returning the static value.
    ///
    /// # Example
    /// ```ignore
    /// #[derive(Debug)]
    /// struct MyClass;
    ///
    /// impl Class for MyClass {
    ///    // ...
    /// }
    ///
    /// context.register_global_class::<MyClass>()?;
    /// // ... code
    /// context.unregister_global_class::<MyClass>()?;
    /// ```
    pub fn unregister_global_class<C: Class>(&mut self) -> JsResult<Option<ClassConstructor>> {
        self.global_object()?
            .delete_property_or_throw(js_string!(C::NAME), self.access())?;
        Ok(self.realm().unregister_class::<C>()?)
    }

    /// Checks if the currently active realm has the global class `C` registered.
    pub fn has_global_class<C: Class>(&self) -> Result<bool, crate::error::RootingError> {
        self.realm().has_class::<C>()
    }

    /// Gets the constructor and prototype of the global class `C` if the currently active realm has
    /// that class registered.
    pub fn get_global_class<C: Class>(
        &self,
    ) -> Result<Option<ClassConstructor>, crate::error::RootingError> {
        self.realm().get_class::<C>()
    }

    /// Returns the global object.
    #[inline]
    pub fn global_object(&self) -> Result<JsObject, crate::error::RootingError> {
        self.vm.frame().realm.global_object()
    }

    /// Returns the currently active intrinsic constructors and objects.
    #[inline]
    #[must_use]
    pub fn intrinsics(&self) -> &Intrinsics {
        self.vm.frame().realm.intrinsics()
    }

    /// Returns the amount of remaining instructions to be executed
    #[cfg(feature = "fuzz")]
    #[inline]
    #[must_use]
    pub fn instructions_remaining(&self) -> usize {
        self.instructions_remaining
    }

    /// Returns the currently active realm.
    #[inline]
    #[must_use]
    pub fn realm(&self) -> &Realm {
        &self.vm.frame().realm
    }

    /// Set the value of trace on the context
    #[cfg(feature = "trace")]
    #[inline]
    pub fn set_trace(&mut self, trace: bool) {
        self.vm.trace = trace;
    }

    /// Get optimizer options.
    #[inline]
    #[must_use]
    pub const fn optimizer_options(&self) -> OptimizerOptions {
        self.optimizer_options
    }
    /// Enable or disable optimizations
    #[inline]
    pub fn set_optimizer_options(&mut self, optimizer_options: OptimizerOptions) {
        self.optimizer_options = optimizer_options;
    }

    /// Changes the strictness mode of the context.
    #[inline]
    pub fn strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    /// Enqueues a [`Job`] on the [`JobExecutor`].
    #[inline]
    ///
    /// # Errors
    /// Returns the exact original input if the executor rejects it before
    /// acceptance. Accepted inline failures have a distinct disposition.
    #[expect(
        clippy::result_large_err,
        reason = "retain original job ownership without allocating to box the refused input"
    )]
    pub fn enqueue_job(
        &mut self,
        job: Job,
    ) -> Result<crate::job::JobAcceptance, crate::job::JobPublicationError> {
        match self.job_executor().enqueue_job(job, self.access()) {
            Ok(crate::job::JobAcceptance::AcceptedWithError(cause)) => Ok(
                crate::job::JobAcceptance::AcceptedWithError(cause.admit(self)),
            ),
            Ok(acceptance) => Ok(acceptance),
            Err(error) => Err(error.admit(self)),
        }
    }

    /// Runs all the jobs with the provided job executor.
    #[inline]
    pub fn run_jobs(&mut self) -> JsResult<()> {
        self.job_executor().run_jobs(self.access())
    }

    /// Abstract operation [`ClearKeptObjects`][clear].
    ///
    /// Clears all objects maintained alive by calls to the [`AddToKeptObjects`][add] abstract
    /// operation, used within the [`WeakRef`][weak] constructor.
    ///
    /// [clear]: https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-clear-kept-objects
    /// [add]: https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-addtokeptobjects
    /// [weak]: https://tc39.es/ecma262/multipage/managing-memory.html#sec-weak-ref-objects
    #[inline]
    pub fn clear_kept_objects(&mut self) {
        self.kept_alive.clear();
    }

    /// Retrieves the current stack trace of the context.
    ///
    /// The stack trace is returned ordered with the most recent frames first.
    #[inline]
    pub fn stack_trace(&self) -> impl Iterator<Item = &CallFrame> {
        // The first frame is always a dummy frame (see `Vm` implementation for more details),
        // so skip the dummy frame and return the reversed list so that the most recent frames are first.
        self.vm.frames.iter().skip(1).rev()
    }

    /// Replaces the active realm with a realm from the same runtime.
    /// Returns the previous realm on success.
    #[inline]
    pub fn enter_realm(&mut self, realm: Realm) -> JsResult<Realm> {
        self.check_realm(&realm)?;
        Ok(std::mem::replace(&mut self.vm.frame_mut().realm, realm))
    }

    pub(crate) fn check_realm(&self, realm: &Realm) -> JsResult<()> {
        crate::error::check_realm_ownership(self.runtime(), self.realm().identity(), realm)
            .map_err(crate::error::RealmOwnershipMismatch::into_direct)
    }

    pub(crate) fn check_publication_realm(
        &self,
        realm: &Realm,
    ) -> Result<(), crate::error::RealmOwnershipMismatch> {
        crate::error::check_realm_ownership(self.runtime(), self.realm().identity(), realm)
    }

    /// Create a new Realm with the default global bindings.
    pub fn create_realm(&mut self) -> JsResult<Realm> {
        let realm = Realm::create(self.host_hooks.as_ref(), &self.root_shape)?;

        let old_realm = self.enter_realm(realm)?;
        let bindings = builtins::set_default_global_bindings(self);
        let realm = self.enter_realm(old_realm)?;
        bindings?;
        Ok(realm)
    }

    /// Returns the runtime shared by this context and its realms.
    #[must_use]
    pub const fn runtime(&self) -> &crate::Runtime {
        self.root_shape.runtime()
    }

    /// Get the [`RootShape`].
    #[inline]
    #[must_use]
    pub const fn root_shape(&self) -> &RootShape {
        &self.root_shape
    }

    /// Gets the host hooks.
    #[inline]
    #[must_use]
    pub fn host_hooks(&self) -> Rc<dyn HostHooks> {
        self.host_hooks.clone()
    }

    /// Gets the internal clock.
    #[inline]
    #[must_use]
    pub fn clock(&self) -> &dyn Clock {
        self.clock.as_ref()
    }

    pub(crate) fn clock_handle(&self) -> Rc<dyn Clock> {
        self.clock.clone()
    }

    /// Gets the current job executor, or `None` if the current job executor
    /// is not a `T`.
    #[inline]
    #[must_use]
    pub fn downcast_job_executor<T: 'static>(&self) -> Option<Rc<T>> {
        Rc::downcast(self.job_executor.clone()).ok()
    }

    /// Gets the current module loader, or `None` if the current module loader
    /// is not a `T`.
    #[must_use]
    pub fn downcast_module_loader<T: 'static>(&self) -> Option<Rc<T>> {
        Rc::downcast(self.module_loader.clone()).ok()
    }

    /// Get the [`RuntimeLimits`].
    #[inline]
    #[must_use]
    pub const fn runtime_limits(&self) -> RuntimeLimits {
        self.vm.runtime_limits
    }

    /// Set the [`RuntimeLimits`].
    #[inline]
    pub fn set_runtime_limits(&mut self, runtime_limits: RuntimeLimits) {
        self.vm.runtime_limits = runtime_limits;
    }

    /// Get a mutable reference to the [`RuntimeLimits`].
    #[inline]
    pub fn runtime_limits_mut(&mut self) -> &mut RuntimeLimits {
        &mut self.vm.runtime_limits
    }

    /// Returns `true` if this context can be suspended by an `Atomics.wait` call.
    #[inline]
    #[must_use]
    pub fn can_block(&self) -> bool {
        self.thread_blocking.0
    }

    /// Gets a mutable reference to the inner [`HostDefined`] field.
    #[inline]
    pub fn host_defined_mut(&mut self) -> &mut HostDefined<dyn Any> {
        &mut self.data
    }

    /// Insert a type into the context-specific [`HostDefined`] field.
    #[inline]
    pub fn insert_data<T: Any>(&mut self, value: T) -> Option<Box<T>> {
        self.data.insert(value)
    }

    /// Check if the context-specific [`HostDefined`] has type T.
    #[inline]
    #[must_use]
    pub fn has_data<T: Any>(&self) -> bool {
        self.data.has::<T>()
    }

    /// Remove type T from the context-specific [`HostDefined`], if it exists.
    #[inline]
    pub fn remove_data<T: Any>(&mut self) -> Option<Box<T>> {
        self.data.remove::<T>()
    }

    /// Get type T from the context-specific [`HostDefined`], if it exists.
    #[inline]
    #[must_use]
    pub fn get_data<T: Any>(&self) -> Option<&T> {
        self.data.get::<T>()
    }
}

// ==== Private API ====

impl Context {
    pub(crate) fn job_lifetime(&self) -> Rc<crate::job::ContextLifetime> {
        self.job_lifetime.clone()
    }

    /// Returns this exact Context instance's stable diagnostic identity.
    pub fn id(&self) -> ContextId {
        self.job_lifetime.id()
    }

    pub(crate) fn prepare_simple_job_queue(
        &mut self,
    ) -> Result<(), crate::error::EngineResourceError> {
        self.simple_job_queues
            .len()
            .checked_add(1)
            .and_then(|count| count.checked_mul(std::mem::size_of::<Rc<RuntimeJobQueues>>()))
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or(crate::error::EngineResourceError::SizeOverflow)?;
        self.simple_job_queues
            .try_reserve(1)
            .map_err(|_| crate::error::EngineResourceError::Allocation)?;
        // Dispose obsolete queue custody before acquiring the executor registry.
        let mut index = 0;
        while index < self.simple_job_queues.len() {
            let obsolete = self
                .simple_job_queues
                .get(index)
                .is_some_and(|queue| Rc::weak_count(queue) == 0);
            if obsolete {
                drop(self.simple_job_queues.remove(index));
            } else {
                index += 1;
            }
        }
        Ok(())
    }

    pub(crate) fn retain_simple_job_queue(&mut self, queue: &Rc<RuntimeJobQueues>) {
        if !self
            .simple_job_queues
            .iter()
            .any(|attached| Rc::ptr_eq(attached, queue))
        {
            // Attachment reserved this slot before publishing either owner link.
            self.simple_job_queues.push(Rc::clone(queue));
        }
    }

    /// Gets the current job executor.
    pub(crate) fn job_executor(&self) -> Rc<dyn JobExecutor> {
        self.job_executor.clone()
    }

    /// Gets the current module loader.
    pub(crate) fn module_loader(&self) -> Rc<dyn ModuleLoader> {
        self.module_loader.clone()
    }

    /// Swaps the currently active realm with `realm`.
    pub(crate) fn swap_realm(&mut self, realm: &mut Realm) {
        std::mem::swap(&mut self.vm.frame_mut().realm, realm);
    }

    /// `CanDeclareGlobalFunction ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalfunction
    pub(crate) fn can_declare_global_function(&mut self, name: &JsString) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object()?;

        // 3. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
        let name = name.clone().into();
        let existing_prop = global_object.__get_own_property__(&name, &mut self.into())?;

        // 4. If existingProp is undefined, return ? IsExtensible(globalObject).
        let Some(existing_prop) = existing_prop else {
            return global_object.is_extensible(self.access());
        };

        // 5. If existingProp.[[Configurable]] is true, return true.
        if existing_prop.configurable() == Some(true) {
            return Ok(true);
        }

        // 6. If IsDataDescriptor(existingProp) is true and existingProp has attribute values { [[Writable]]: true, [[Enumerable]]: true }, return true.
        if existing_prop.is_data_descriptor()
            && existing_prop.writable() == Some(true)
            && existing_prop.enumerable() == Some(true)
        {
            return Ok(true);
        }

        // 7. Return false.
        Ok(false)
    }

    /// `CanDeclareGlobalVar ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalvar
    pub(crate) fn can_declare_global_var(&mut self, name: &JsString) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object()?;

        // 3. Let hasProperty be ? HasOwnProperty(globalObject, N).
        let has_property = global_object.has_own_property(name.clone(), self.access())?;

        // 4. If hasProperty is true, return true.
        if has_property {
            return Ok(true);
        }

        // 5. Return ? IsExtensible(globalObject).
        global_object.is_extensible(self.access())
    }

    /// `CreateGlobalVarBinding ( N, D )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createglobalvarbinding
    pub(crate) fn create_global_var_binding(
        &mut self,
        name: JsString,
        configurable: bool,
    ) -> JsResult<()> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object()?;

        // 3. Let hasProperty be ? HasOwnProperty(globalObject, N).
        let has_property = global_object.has_own_property(name.clone(), self.access())?;

        // 4. Let extensible be ? IsExtensible(globalObject).
        let extensible = global_object.is_extensible(self.access())?;

        // 5. If hasProperty is false and extensible is true, then
        if !has_property && extensible {
            // a. Perform ? ObjRec.CreateMutableBinding(N, D).
            // b. Perform ? ObjRec.InitializeBinding(N, undefined).
            global_object.define_declaration_property(
                name.into(),
                PropertyDescriptor::builder()
                    .value(JsValue::undefined())
                    .writable(true)
                    .enumerable(true)
                    .configurable(configurable)
                    .build(),
                self,
            )?;
        }

        // 6. If envRec.[[VarNames]] does not contain N, then
        //     a. Append N to envRec.[[VarNames]].
        // 7. Return unused.
        Ok(())
    }

    /// Keeps a newly compiled global function as a Rust recipe until the
    /// definition's real mutation or callback entry accepts. The recipe performs
    /// only callback-free installation and exposes preallocated active storage.
    pub(crate) fn create_global_function_binding_prepared(
        &mut self,
        name: JsString,
        configurable: bool,
        materialize: impl FnOnce(&mut Context) -> JsResult<JsObject>,
    ) -> JsResult<()> {
        let global_object = self.realm().global_object()?;
        let key = name.clone().into();
        let existing_prop = global_object.__get_own_property__(&key, &mut self.into())?;
        let descriptor = if existing_prop.is_none()
            || existing_prop.and_then(|property| property.configurable()) == Some(true)
        {
            PropertyDescriptor::builder()
                .writable(true)
                .enumerable(true)
                .configurable(configurable)
                .build()
        } else {
            PropertyDescriptor::builder().build()
        };
        let Some(function) = global_object.define_prepared_function(
            &key,
            descriptor,
            Box::new(materialize),
            self,
        )?
        else {
            return Err(JsNativeError::typ()
                .with_message(format!("cannot redefine property: {key}"))
                .into());
        };
        global_object.set(name, function, false, self.access())?;
        Ok(())
    }

    /// `HasRestrictedGlobalProperty ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasrestrictedglobalproperty
    pub(crate) fn has_restricted_global_property(&mut self, name: &JsString) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = self.realm().global_object()?;

        // 3. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
        let name = name.clone().into();
        let existing_prop = global_object.__get_own_property__(&name, &mut self.into())?;

        // 4. If existingProp is undefined, return false.
        let Some(existing_prop) = existing_prop else {
            return Ok(false);
        };

        // 5. If existingProp.[[Configurable]] is true, return false.
        if existing_prop.configurable() == Some(true) {
            return Ok(false);
        }

        // 6. Return true.
        Ok(true)
    }

    /// Returns `true` if this context is in strict mode.
    pub(crate) const fn is_strict(&self) -> bool {
        self.strict
    }

    /// `9.4.1 GetActiveScriptOrModule ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getactivescriptormodule
    #[must_use]
    pub fn get_active_script_or_module(&self) -> Option<ActiveRunnable> {
        // 1. If the execution context stack is empty, return null.
        // 2. Let ec be the topmost execution context on the execution context stack whose ScriptOrModule component is not null.
        // 3. If no such execution context exists, return null. Otherwise, return ec's ScriptOrModule.
        if let Some(active_runnable) = &self.vm.frame().active_runnable {
            return Some(active_runnable.clone());
        }

        self.vm
            .frames
            .iter()
            .rev()
            .find_map(|frame| frame.active_runnable.clone())
    }

    /// Get `active function object`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#active-function-object
    pub(crate) fn active_function_object(&self) -> Option<JsObject> {
        if self.vm.native_active_function.is_some() {
            return self.vm.native_active_function.clone();
        }
        if self.vm.frame().is_host() {
            return None;
        }

        self.vm.stack.get_function(self.vm.frame())
    }

    /// `GlobalDeclarationInstantiation ( script, env )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-globaldeclarationinstantiation
    pub(crate) fn preflight_global_declarations(
        &mut self,
        codeblock: &CodeBlock,
        declarations: &surgeist_js_ast::ScriptDeclarations,
    ) -> JsResult<()> {
        for name in declarations.lexical_names() {
            if self.has_restricted_global_property(name)? {
                return Err(
                    js_error!(SyntaxError: "cannot redefine non-configurable global property"),
                );
            }
        }
        for function in codeblock.global_fns.iter().rev() {
            let name = codeblock.constant_string(function.name_index as usize);
            if !self.can_declare_global_function(&name)? {
                return Err(js_error!(TypeError: "cannot declare global function"));
            }
        }
        for name in declarations.var_names() {
            if !codeblock
                .global_fns
                .iter()
                .any(|function| codeblock.constant_string(function.name_index as usize) == *name)
                && !self.can_declare_global_var(name)?
            {
                return Err(js_error!(TypeError: "cannot declare global variable"));
            }
        }
        Ok(())
    }

    /// Check eval's global declaration eligibility before private scope publication.
    pub(crate) fn preflight_eval_declarations(&mut self, codeblock: &CodeBlock) -> JsResult<()> {
        // 8. For each element d of varDeclarations, in reverse List order, do
        //    a. If d is not either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
        //       i. Assert: d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration.
        //       ii. NOTE: If there are multiple function declarations for the same name, the last declaration is used.
        //       iii. Let fn be the sole element of the BoundNames of d.
        for fun in codeblock.global_fns.iter().rev() {
            // ...
            // 1. Let fnDefinable be ? env.CanDeclareGlobalFunction(fn).
            // 2. If fnDefinable is false, throw a TypeError exception.
            let name = codeblock.constant_string(fun.name_index as usize);
            if !self.can_declare_global_function(&name)? {
                return Err(js_error!(TypeError: "cannot declare global function"));
            }
        }

        // 10. For each element d of varDeclarations, do
        for global_var in &codeblock.global_vars {
            // ...
            // a. Let vnDefinable be ? env.CanDeclareGlobalVar(vn).
            // b. If vnDefinable is false, throw a TypeError exception.
            let name = codeblock.constant_string(*global_var as usize);
            if !self.can_declare_global_var(&name)? {
                return Err(js_error!(TypeError: "cannot declare global variable"));
            }
        }

        Ok(())
    }
}

impl Context {
    /// Creates a `ContextCleanupGuard` that executes some cleanup after being dropped.
    pub(crate) fn guard<F>(&mut self, cleanup: F) -> ContextCleanupGuard<'_, F>
    where
        F: FnOnce(&mut Context) + 'static,
    {
        ContextCleanupGuard::new(self, cleanup)
    }

    /// Get the Intl data provider.
    #[cfg(feature = "intl")]
    pub(crate) const fn intl_provider(&self) -> &icu::IntlProvider {
        &self.intl_provider
    }

    /// Get the Time Zone Provider
    #[cfg(any(feature = "temporal", feature = "intl"))]
    pub(crate) fn timezone_provider(&self) -> &dyn TimeZoneProvider {
        self.timezone_provider.as_ref()
    }
}

/// Builder for the [`Context`] type.
#[derive(Default)]
pub struct ContextBuilder {
    runtime: Option<crate::Runtime>,
    host_hooks: Option<Rc<dyn HostHooks>>,
    clock: Option<Rc<dyn Clock>>,
    job_executor: Option<Rc<dyn JobExecutor>>,
    module_loader: Option<Rc<dyn ModuleLoader>>,
    can_block: bool,
    #[cfg(feature = "intl")]
    icu: Option<icu::IntlProvider>,
    #[cfg(any(feature = "temporal", feature = "intl"))]
    timezone_provider: Option<Box<dyn TimeZoneProvider>>,
    #[cfg(feature = "fuzz")]
    instructions_remaining: usize,
}

impl std::fmt::Debug for ContextBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Clone, Copy, Debug)]
        struct JobExecutor;
        #[derive(Clone, Copy, Debug)]
        struct HostHooks;
        #[derive(Clone, Copy, Debug)]
        struct Clock;
        #[derive(Clone, Copy, Debug)]
        struct ModuleLoader;

        let mut out = f.debug_struct("ContextBuilder");

        out.field("host_hooks", &self.host_hooks.as_ref().map(|_| HostHooks))
            .field("clock", &self.clock.as_ref().map(|_| Clock))
            .field(
                "job_executor",
                &self.job_executor.as_ref().map(|_| JobExecutor),
            )
            .field(
                "module_loader",
                &self.module_loader.as_ref().map(|_| ModuleLoader),
            )
            .field("can_block", &self.can_block);

        #[cfg(feature = "intl")]
        out.field("icu", &self.icu);

        #[cfg(any(feature = "temporal", feature = "intl"))]
        out.field(
            "timezone_provider",
            &self.timezone_provider.as_ref().map(|_| "TimeZoneProvider"),
        );

        #[cfg(feature = "fuzz")]
        out.field("instructions_remaining", &self.instructions_remaining);

        out.finish()
    }
}

impl ContextBuilder {
    /// Creates a new [`ContextBuilder`] with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds the context on an existing runtime.
    #[must_use]
    pub fn runtime(mut self, runtime: crate::Runtime) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Provides a [`BufferProvider`] data provider to the [`Context`].
    ///
    /// This function is only available if the `intl` feature is enabled.
    ///
    /// # Additional considerations
    ///
    /// If the data was generated using `icu_datagen`, make sure that the deduplication strategy is
    /// not set to [`Maximal`]. Otherwise, `icu_datagen` will delete base locales such as "en" from
    /// the list of supported locales if the required data for "en" is the same as "und".
    /// We recommend [`RetainBaseLanguages`] as a nice default, which will only deduplicate locales
    /// if the deduplication target is not "und".
    ///
    /// # Errors
    ///
    /// This returns `Err` if the provided provider doesn't have the required locale information
    /// to construct common tools used through `Intl`. Note that this doesn't
    /// mean that the provider will successfully construct all `Intl` services; that check is made
    /// until the creation of an instance of a service.
    ///
    /// [`Maximal`]: https://docs.rs/icu_datagen/latest/icu_datagen/enum.DeduplicationStrategy.html#variant.Maximal
    /// [`RetainBaseLanguages`]: https://docs.rs/icu_datagen/latest/icu_datagen/enum.DeduplicationStrategy.html#variant.RetainBaseLanguages
    /// [`BufferProvider`]: icu_provider::buf::BufferProvider
    #[cfg(feature = "intl")]
    pub fn icu_buffer_provider<
        T: icu_provider::prelude::DynamicDryDataProvider<icu_provider::prelude::BufferMarker>
            + 'static,
    >(
        mut self,
        provider: T,
    ) -> Result<Self, IcuError> {
        self.icu = Some(icu::IntlProvider::try_new_buffer(provider));
        Ok(self)
    }

    /// Set the [`timezone_provider::provider::TimeZoneProvider`] that should be used to source
    /// time zone data.
    ///
    /// ## Default
    ///
    /// If no time zone provider is provided, a compiled time zone provider will be used
    /// which includes the time zone data in the binary. This may increase binary sizes
    /// by up to 200 Kb.
    #[cfg(any(feature = "temporal", feature = "intl"))]
    #[must_use]
    pub fn timezone_provider<T: TimeZoneProvider + 'static>(mut self, provider: T) -> Self {
        self.timezone_provider = Some(Box::new(provider));
        self
    }

    /// Initializes the [`HostHooks`] for the context.
    ///
    /// [`Host Hooks`]: https://tc39.es/ecma262/#sec-host-hooks-summary
    #[must_use]
    pub fn host_hooks<H: HostHooks + 'static>(mut self, host_hooks: Rc<H>) -> Self {
        self.host_hooks = Some(host_hooks);
        self
    }

    /// Initializes the [`Clock`] for the context.
    #[must_use]
    pub fn clock<C: Clock + 'static>(mut self, clock: Rc<C>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Initializes the [`JobExecutor`] for the context.
    #[must_use]
    pub fn job_executor<Q: JobExecutor + 'static>(mut self, job_executor: Rc<Q>) -> Self {
        self.job_executor = Some(job_executor);
        self
    }

    /// Initializes the [`ModuleLoader`] for the context.
    #[must_use]
    pub fn module_loader<M: ModuleLoader + 'static>(mut self, module_loader: Rc<M>) -> Self {
        self.module_loader = Some(module_loader);
        self
    }

    /// [`AgentCanSuspend ( )`][spec] aka `[[CanBlock]]`
    ///
    /// Defines if this context can be suspended by calls to the [`Atomics.wait`][wait] function.
    ///
    /// # Note
    ///
    /// By the specification, multiple agents cannot share the same thread if any of them has its
    /// `[[CanBlock]]` field set to true. The builder will verify at build time that all contexts on
    /// the current thread fulfill this requisite.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-agentcansuspend
    /// [wait]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/wait
    #[must_use]
    pub const fn can_block(mut self, can_block: bool) -> Self {
        self.can_block = can_block;
        self
    }

    /// Specifies the number of instructions remaining to the [`Context`].
    ///
    /// This function is only available if the `fuzz` feature is enabled.
    #[cfg(feature = "fuzz")]
    #[must_use]
    pub const fn instructions_remaining(mut self, instructions_remaining: usize) -> Self {
        self.instructions_remaining = instructions_remaining;
        self
    }

    /// Builds a new [`Context`] with the provided parameters, and defaults
    /// all missing parameters to their default values.
    // TODO: try to use a custom error here, since most of the `JsError` APIs
    // require having a `Context` in the first place.
    pub fn build(self) -> JsResult<Context> {
        let thread_blocking = ThreadBlocking::acquire(self.can_block)?;
        let root_shape = RootShape::new(self.runtime.unwrap_or_default())?;

        let host_hooks = self.host_hooks.unwrap_or(Rc::new(DefaultHooks));
        let clock = self.clock.unwrap_or_else(|| Rc::new(StdClock::new()));
        let realm = Realm::create(host_hooks.as_ref(), &root_shape)?;
        let vm = Vm::new(realm);

        let module_loader: Rc<dyn ModuleLoader> = if let Some(loader) = self.module_loader {
            loader
        } else if let Ok(loader) = SimpleModuleLoader::new(Path::new(".")) {
            Rc::new(loader)
        } else {
            Rc::new(IdleModuleLoader)
        };

        let job_executor = self
            .job_executor
            .unwrap_or_else(|| Rc::new(SimpleJobExecutor::new()));

        let mut context = Context {
            vm,
            execution: None,
            module_run_failure: None,
            strict: false,
            #[cfg(any(feature = "temporal", feature = "intl"))]
            timezone_provider: if let Some(provider) = self.timezone_provider {
                provider
            } else {
                Box::new(ZeroCompiledTzdbProvider::default())
            },
            #[cfg(feature = "intl")]
            intl_provider: if let Some(icu) = self.icu {
                icu
            } else {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "intl_bundled")] {
                        icu::IntlProvider::try_new_buffer(surgeist_js_icu_provider::buffer())
                    } else {
                        return Err(JsNativeError::typ()
                            .with_message("missing Intl provider for context")
                            .into()
                        );
                    }
                }
            },
            #[cfg(feature = "fuzz")]
            instructions_remaining: self.instructions_remaining,
            kept_alive: Vec::new(),
            host_hooks,
            clock,
            job_executor,
            job_lifetime: crate::job::ContextLifetime::new(),
            simple_job_queues: Vec::new(),
            module_loader,
            optimizer_options: OptimizerOptions::OPTIMIZE_ALL,
            root_shape,
            thread_blocking,
            data: HostDefined::default(),
        };

        if let Some(executor) = context.downcast_job_executor::<SimpleJobExecutor>() {
            executor.attach_context(&mut context)?;
        }
        builtins::set_default_global_bindings(&mut context)?;

        Ok(context)
    }
}

/// A cleanup guard for a [`Context`] that is executed when dropped.
#[derive(Debug)]
pub(crate) struct ContextCleanupGuard<'a, F>
where
    F: FnOnce(&mut Context) + 'static,
{
    context: &'a mut Context,
    cleanup: Option<F>,
}

impl<'a, F> ContextCleanupGuard<'a, F>
where
    F: FnOnce(&mut Context) + 'static,
{
    /// Creates a new `ContextCleanupGuard` from the current context and its cleanup operation.
    pub(crate) fn new(context: &'a mut Context, cleanup: F) -> Self {
        Self {
            context,
            cleanup: Some(cleanup),
        }
    }
}

impl<F> std::ops::Deref for ContextCleanupGuard<'_, F>
where
    F: FnOnce(&mut Context) + 'static,
{
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.context
    }
}

impl<F> std::ops::DerefMut for ContextCleanupGuard<'_, F>
where
    F: FnOnce(&mut Context) + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.context
    }
}

impl<F> Drop for ContextCleanupGuard<'_, F>
where
    F: FnOnce(&mut Context) + 'static,
{
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup(self.context);
        }
    }
}

#[cfg(test)]
mod runtime_tests {
    use super::*;

    struct ForeignHookError(crate::JsError);

    impl HostHooks for ForeignHookError {
        fn ensure_can_compile_strings(
            &self,
            _: Realm,
            _: &[JsString],
            _: &JsString,
            _: bool,
            _: crate::ContextAccess<'_>,
        ) -> JsResult<()> {
            Err(self.0.clone())
        }

        fn ensure_can_add_private_element(
            &self,
            _: &JsObject,
            _: crate::ContextAccess<'_>,
        ) -> JsResult<()> {
            Err(self.0.clone())
        }
    }

    #[test]
    fn materialization_foreign_host_hook_errors_are_local_type_errors() {
        let foreign = Context::builder().build().unwrap();
        let foreign_error =
            crate::JsError::from_opaque(foreign.global_object().unwrap().into()).unwrap();
        let nested = JsNativeError::error()
            .with_cause(foreign_error.clone())
            .into();
        for error in [foreign_error, nested] {
            let mut context = Context::builder()
                .host_hooks(Rc::new(ForeignHookError(error)))
                .build()
                .unwrap();
            for source in [
                "try { eval('0'); } catch (error) { error instanceof TypeError; }",
                "try { new Function('0'); } catch (error) { error instanceof TypeError; }",
                "try { new class { #field = 0; }; } catch (error) { error instanceof TypeError; }",
            ] {
                let result = context.eval(Source::from_bytes(source)).unwrap();
                assert_eq!(result.as_boolean(), Some(true), "{source}");
                context.runtime().collect().unwrap();
            }
        }
    }

    struct ReentrantGlobal;

    impl HostHooks for ReentrantGlobal {
        fn create_global_object(&self, intrinsics: &intrinsics::Intrinsics) -> JsResult<JsObject> {
            let runtime = intrinsics.runtime()?;
            let nested = Context::builder()
                .runtime(runtime.clone())
                .build()
                .expect("nested context on the same runtime");
            assert!(nested.runtime().same_heap(&runtime));
            assert!(
                intrinsics
                    .constructors()
                    .object()
                    .prototype()?
                    .runtime()
                    .same_heap(&runtime)
            );
            Ok(nested.global_object()?)
        }
    }

    #[test]
    fn realm_host_hook_can_reenter_the_same_runtime() {
        let runtime = crate::Runtime::default();
        runtime
            .heap
            .set_allocation_collection(surgeist_js_gc::heap::AllocationCollection::Opportunistic(
                std::num::NonZeroUsize::MIN,
            ))
            .unwrap();
        let outer = Context::builder()
            .runtime(runtime.clone())
            .host_hooks(Rc::new(ReentrantGlobal))
            .build()
            .expect("outer realm accepts a same-runtime global");
        assert!(outer.global_object().unwrap().runtime().same_heap(&runtime));
    }

    #[test]
    fn entering_a_foreign_realm_leaves_the_active_realm_unchanged() {
        let mut local = Context::builder().build().unwrap();
        let foreign = Context::builder().build().unwrap();
        let previous = local.realm().clone();
        let error = local.enter_realm(foreign.realm().clone()).unwrap_err();
        assert!(
            error
                .as_native()
                .is_some_and(|error| error.kind() == &crate::JsNativeErrorKind::Type)
        );
        assert_eq!(local.realm(), &previous);
    }

    struct ForeignGlobal;

    impl HostHooks for ForeignGlobal {
        fn create_global_object(&self, _: &intrinsics::Intrinsics) -> JsResult<JsObject> {
            JsObject::with_null_proto(&crate::Runtime::default())
        }
    }

    #[test]
    fn realm_rejects_a_foreign_host_global_and_releases_its_thread_guard() {
        let error = Context::builder()
            .host_hooks(Rc::new(ForeignGlobal))
            .build()
            .expect_err("a different runtime cannot supply the global object");
        assert!(
            error
                .as_native()
                .is_some_and(|error| error.kind() == &crate::JsNativeErrorKind::Type)
        );
        assert!(Context::builder().can_block(true).build().is_ok());
    }
}
