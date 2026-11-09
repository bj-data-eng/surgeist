//! Borrowed access to the semantic operations of an owned context.

use std::{any::Any, ops::AsyncFnOnce, rc::Rc};

use super::{Clock, Context, HostHooks, intrinsics::Intrinsics};
use crate::{
    HostDefined, JsResult, JsString, JsValue, Runtime, Source,
    class::{Class, ClassConstructor},
    execution::{ExecutionOptions, ExecutionOutcome, ExecutionSetupError, ExecutionTask},
    job::Job,
    native_function::NativeFunction,
    object::{JsObject, shape::RootShape},
    optimizer::OptimizerOptions,
    property::{Attribute, PropertyKey},
    realm::Realm,
    vm::{ActiveRunnable, CallFrame, RuntimeLimits},
};
use surgeist_js_parser::source::ReadChar;

/// An exclusive borrowed capability for context operations.
///
/// The owned [`Context`] remains reserved for this borrow. Moving or dropping
/// this handle does not move the context. A shorter [`Self::reborrow`] grants
/// access for a nested operation while this handle remains borrowed.
/// Semantic methods use the same implementations as the owned context.
///
/// ```
/// use surgeist_js::{Context, Source};
///
/// let mut owner = Context::default();
/// let mut access = owner.access();
/// let value = access.eval(Source::from_bytes("40 + 2"))?;
/// assert_eq!(value.as_number(), Some(42.0));
/// # Ok::<(), surgeist_js::JsError>(())
/// ```
///
/// Access has no public mutable-owner projection:
///
/// ```compile_fail
/// use surgeist_js::Context;
///
/// let mut owner = Context::default();
/// let access = owner.access();
/// let extracted: &mut Context = access.into_owner();
/// ```
///
/// It cannot be dereferenced to replace the context either:
///
/// ```compile_fail
/// use surgeist_js::Context;
///
/// let mut owner = Context::default();
/// let mut access = owner.access();
/// let extracted = std::mem::take(&mut *access);
/// ```
#[must_use]
pub struct ContextAccess<'owner> {
    owner: &'owner mut Context,
}

impl std::fmt::Debug for ContextAccess<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ContextAccess")
            .finish_non_exhaustive()
    }
}

impl Context {
    /// Borrows this context's semantic operations without transferring its owner.
    #[inline]
    pub fn access(&mut self) -> ContextAccess<'_> {
        ContextAccess { owner: self }
    }
}

impl<'owner> ContextAccess<'owner> {
    /// Returns the stable identity of this access's exact owned Context.
    pub fn id(&self) -> crate::job::ContextId {
        self.owner.id()
    }

    /// Grants a shorter exclusive borrow of the same context.
    #[inline]
    pub fn reborrow(&mut self) -> ContextAccess<'_> {
        ContextAccess { owner: self.owner }
    }

    /// Consumes access for a private engine adapter, never for host dispatch.
    #[inline]
    pub(crate) fn into_owner(self) -> &'owner mut Context {
        self.owner
    }

    /// Evaluates source using the context's current execution, if any.
    ///
    /// # Errors
    /// Returns the same parsing, compilation and execution errors as
    /// [`Context::eval`]. Scheduled jobs are not automatically drained.
    #[inline]
    pub fn eval<R: ReadChar>(&mut self, source: Source<'_, R>) -> JsResult<JsValue> {
        self.reborrow().into_owner().eval(source)
    }

    /// Evaluates source under a new execution budget, when none is active.
    ///
    /// # Errors
    /// Returns [`ExecutionSetupError::AlreadyActive`] for an active operation,
    /// or another setup error from [`Context::eval_with_budget`].
    #[inline]
    pub fn eval_with_budget<R: ReadChar>(
        &mut self,
        source: Source<'_, R>,
        options: ExecutionOptions,
    ) -> Result<ExecutionOutcome<JsValue>, ExecutionSetupError> {
        self.owner.eval_with_budget(source, options)
    }

    /// Runs a controlled callback on this same owner.
    ///
    /// # Errors
    /// Returns [`ExecutionSetupError::AlreadyActive`] before invoking the
    /// callback when this access belongs to an active operation. Other setup
    /// failures are the same as [`Context::with_execution`].
    #[inline]
    pub fn with_execution<T>(
        &mut self,
        options: ExecutionOptions,
        run: impl for<'call> FnOnce(ContextAccess<'call>) -> JsResult<T>,
    ) -> Result<ExecutionOutcome<T>, ExecutionSetupError> {
        self.reborrow().into_owner().with_execution(options, run)
    }

    /// Reserves this access's owner for a controlled async callback.
    ///
    /// The task borrows this handle only for the current reborrow, until it
    /// completes or is dropped. Its callback receives shorter protected access.
    ///
    /// # Errors
    /// Returns [`ExecutionSetupError::AlreadyActive`] before invoking the
    /// callback when an operation is active. Other setup failures are the same
    /// as [`Context::execution_task`].
    #[inline]
    pub fn execution_task<'access, T, F>(
        &'access mut self,
        options: ExecutionOptions,
        run: F,
    ) -> Result<ExecutionTask<'access, T>, ExecutionSetupError>
    where
        T: 'access,
        F: for<'call> AsyncFnOnce(ContextAccess<'call>) -> JsResult<T> + 'access,
    {
        self.reborrow().into_owner().execution_task(options, run)
    }

    /// Registers a property on the current global object.
    ///
    /// # Errors
    /// Returns the errors from [`Context::register_global_property`].
    #[inline]
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
        self.owner.register_global_property(key, value, attribute)
    }

    /// Registers a native function that is callable and constructable.
    ///
    /// # Errors
    /// Returns the errors from [`Context::register_global_callable`].
    #[inline]
    pub fn register_global_callable(
        &mut self,
        name: JsString,
        length: usize,
        body: NativeFunction,
    ) -> JsResult<()> {
        self.owner.register_global_callable(name, length, body)
    }

    /// Registers a native function that is not a constructor.
    ///
    /// # Errors
    /// Returns the errors from [`Context::register_global_builtin_callable`].
    #[inline]
    pub fn register_global_builtin_callable(
        &mut self,
        name: JsString,
        length: usize,
        body: NativeFunction,
    ) -> JsResult<()> {
        self.owner
            .register_global_builtin_callable(name, length, body)
    }

    /// Registers a native class in the current realm.
    ///
    /// # Errors
    /// Returns the errors from [`Context::register_global_class`].
    #[inline]
    pub fn register_global_class<C: Class>(&mut self) -> JsResult<()> {
        self.owner.register_global_class::<C>()
    }

    /// Removes a native class registration from the current realm.
    ///
    /// # Errors
    /// Returns the errors from [`Context::unregister_global_class`].
    #[inline]
    pub fn unregister_global_class<C: Class>(&mut self) -> JsResult<Option<ClassConstructor>> {
        self.owner.unregister_global_class::<C>()
    }

    /// Checks whether the current realm has the class registered.
    #[must_use]
    #[inline]
    pub fn has_global_class<C: Class>(&self) -> bool {
        self.owner.has_global_class::<C>()
    }

    /// Returns a class's constructor and prototype from the current realm.
    #[must_use]
    #[inline]
    pub fn get_global_class<C: Class>(&self) -> Option<ClassConstructor> {
        self.owner.get_global_class::<C>()
    }

    /// Returns the current global object.
    #[must_use]
    #[inline]
    pub fn global_object(&self) -> JsObject {
        self.owner.global_object()
    }

    /// Returns the current realm's intrinsic constructors and objects.
    #[must_use]
    #[inline]
    pub fn intrinsics(&self) -> &Intrinsics {
        self.owner.intrinsics()
    }

    /// Returns the current realm.
    #[must_use]
    #[inline]
    pub fn realm(&self) -> &Realm {
        self.owner.realm()
    }

    /// Returns the runtime shared by the context and its realms.
    #[must_use]
    #[inline]
    pub fn runtime(&self) -> &Runtime {
        self.owner.runtime()
    }

    /// Returns the root shape used for object construction.
    #[must_use]
    #[inline]
    pub fn root_shape(&self) -> &RootShape {
        self.owner.root_shape()
    }

    /// Returns the configured host hooks.
    #[must_use]
    #[inline]
    pub fn host_hooks(&self) -> Rc<dyn HostHooks> {
        self.owner.host_hooks()
    }

    /// Returns the configured clock.
    #[must_use]
    #[inline]
    pub fn clock(&self) -> &dyn Clock {
        self.owner.clock()
    }

    /// Returns the configured job executor when its concrete type matches.
    #[must_use]
    #[inline]
    pub fn downcast_job_executor<T: 'static>(&self) -> Option<Rc<T>> {
        self.owner.downcast_job_executor::<T>()
    }

    /// Returns the configured module loader when its concrete type matches.
    #[must_use]
    #[inline]
    pub fn downcast_module_loader<T: 'static>(&self) -> Option<Rc<T>> {
        self.owner.downcast_module_loader::<T>()
    }

    /// Returns the active Script or module, if any.
    #[must_use]
    #[inline]
    pub fn get_active_script_or_module(&self) -> Option<ActiveRunnable> {
        self.owner.get_active_script_or_module()
    }

    /// Returns the current call frames, most recent first.
    #[inline]
    pub fn stack_trace(&self) -> impl Iterator<Item = &CallFrame> {
        self.owner.stack_trace()
    }

    /// Returns the current optimizer options.
    #[must_use]
    #[inline]
    pub fn optimizer_options(&self) -> OptimizerOptions {
        self.owner.optimizer_options()
    }

    /// Changes the optimizer options using the existing context operation.
    #[inline]
    pub fn set_optimizer_options(&mut self, options: OptimizerOptions) {
        self.owner.set_optimizer_options(options);
    }

    /// Changes the context's strictness mode.
    #[inline]
    pub fn strict(&mut self, strict: bool) {
        self.owner.strict(strict);
    }

    /// Enables or disables bytecode tracing.
    #[cfg(feature = "trace")]
    #[inline]
    pub fn set_trace(&mut self, trace: bool) {
        self.owner.set_trace(trace);
    }

    /// Returns the remaining fuzz instruction allowance.
    #[cfg(feature = "fuzz")]
    #[must_use]
    #[inline]
    pub fn instructions_remaining(&self) -> usize {
        self.owner.instructions_remaining()
    }

    /// Enqueues a job on the configured executor.
    #[inline]
    ///
    /// # Errors
    /// Returns the exact original job and the receiving engine cause if the
    /// executor rejected the input before acceptance.
    #[expect(
        clippy::result_large_err,
        reason = "retain original job ownership without allocating to box the refused input"
    )]
    pub fn enqueue_job(
        &mut self,
        job: Job,
    ) -> Result<crate::job::JobAcceptance, crate::job::JobPublicationError> {
        self.owner.enqueue_job(job)
    }

    /// Runs jobs using the configured executor.
    ///
    /// # Errors
    /// Returns the errors from [`Context::run_jobs`].
    #[inline]
    pub fn run_jobs(&mut self) -> JsResult<()> {
        self.owner.run_jobs()
    }

    /// Clears the objects kept alive for the current job.
    #[inline]
    pub fn clear_kept_objects(&mut self) {
        self.owner.clear_kept_objects();
    }

    /// Selects a realm from the same runtime, returning the previous realm.
    ///
    /// # Errors
    /// Returns the runtime mismatch error from [`Context::enter_realm`].
    #[inline]
    pub fn enter_realm(&mut self, realm: Realm) -> JsResult<Realm> {
        self.owner.enter_realm(realm)
    }

    /// Creates a realm with the default global bindings.
    ///
    /// # Errors
    /// Returns the errors from [`Context::create_realm`].
    #[inline]
    pub fn create_realm(&mut self) -> JsResult<Realm> {
        self.owner.create_realm()
    }

    /// Returns a copy of the context's runtime limits.
    #[must_use]
    #[inline]
    pub fn runtime_limits(&self) -> RuntimeLimits {
        self.owner.runtime_limits()
    }

    /// Updates runtime limits without lending mutable VM storage.
    #[inline]
    pub fn set_runtime_limits(&mut self, limits: RuntimeLimits) {
        self.owner.set_runtime_limits(limits);
    }

    /// Reports whether Atomics.wait may suspend this context's thread.
    #[must_use]
    #[inline]
    pub fn can_block(&self) -> bool {
        self.owner.can_block()
    }

    /// Grants mutable access to context-specific host data.
    #[inline]
    pub fn host_defined_mut(&mut self) -> &mut HostDefined<dyn Any> {
        self.owner.host_defined_mut()
    }

    /// Inserts typed context-specific host data and returns the previous value.
    #[inline]
    pub fn insert_data<T: Any>(&mut self, value: T) -> Option<Box<T>> {
        self.owner.insert_data(value)
    }

    /// Checks whether typed context-specific host data exists.
    #[must_use]
    #[inline]
    pub fn has_data<T: Any>(&self) -> bool {
        self.owner.has_data::<T>()
    }

    /// Removes typed context-specific host data.
    #[inline]
    pub fn remove_data<T: Any>(&mut self) -> Option<Box<T>> {
        self.owner.remove_data::<T>()
    }

    /// Reads typed context-specific host data.
    #[must_use]
    #[inline]
    pub fn get_data<T: Any>(&self) -> Option<&T> {
        self.owner.get_data::<T>()
    }
}
