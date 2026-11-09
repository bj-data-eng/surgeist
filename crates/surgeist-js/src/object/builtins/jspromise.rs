//! A Rust API wrapper for Boa's promise Builtin ECMAScript Object

use super::{JsArray, JsFunction};
use crate::ContextAccess;
#[cfg(feature = "experimental")]
use crate::JsArgs;
use crate::builtins::promise::native_settlement::{Association, SettlementStatus, SettlementWait};
use crate::value::TryIntoJs;
use crate::{
    JsError, JsExpect, JsNativeError, JsResult, JsValue, NativeFunction,
    builtins::{
        Promise,
        promise::{PromiseState, ResolvingFunctions},
    },
    job::{ContextTask, NativeAsyncJob},
    object::JsObject,
    value::TryFromJs,
};
use std::{future::Future, pin::Pin, task};

/// An ECMAScript [promise] object.
///
/// Known as the concurrency primitive of ECMAScript, this is the main struct used to manipulate,
/// chain and inspect `Promises` from Rust code.
///
/// # Examples
///
/// ```
/// # use surgeist_js::{
/// #     builtins::promise::PromiseState,
/// #     js_string,
/// #     object::{builtins::JsPromise, FunctionObjectBuilder},
/// #     property::Attribute,
/// #     Context, JsArgs, JsError, JsValue, NativeFunction,
/// # };
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let context = &mut Context::default();
///
/// context.register_global_property(
///     js_string!("finally"),
///     false,
///     Attribute::all(),
/// );
///
/// let promise = JsPromise::new(
///     |resolvers, mut context| {
///         let result = js_string!("hello world!").into();
///         resolvers.resolve.call(
///             &JsValue::undefined(),
///             &[result],
///             context.reborrow(),
///         )?;
///         Ok(JsValue::undefined())
///     },
///     context.access(),
/// )?;
///
/// let promise = promise
///     .then(
///         Some(
///             NativeFunction::from_fn_ptr(|_, args, _| {
///                 Err(JsError::from_opaque(args.get_or_undefined(0).clone())
///                     .into())
///             })
///             .to_js_function(context.realm()).unwrap(),
///         ),
///         None,
///         context.access(),
///     )?
///     .catch(
///         NativeFunction::from_fn_ptr(|_, args, _| {
///             Ok(args.get_or_undefined(0).clone())
///         })
///         .to_js_function(context.realm()).unwrap(),
///         context.access(),
///     )?
///     .finally(
///         NativeFunction::from_fn_ptr(|_, _, mut context| {
///             context.global_object().clone().set(
///                 js_string!("finally"),
///                 JsValue::from(true),
///                 true,
///                 context.reborrow(),
///             )?;
///             Ok(JsValue::undefined())
///         })
///         .to_js_function(context.realm()).unwrap(),
///         context.access(),
///     )?;
///
/// context.run_jobs();
///
/// assert_eq!(
///     promise.state().unwrap(),
///     PromiseState::Fulfilled(js_string!("hello world!").into())
/// );
///
/// assert_eq!(
///     context
///         .global_object()
///         .clone()
///         .get(js_string!("finally"), context.access())?,
///     JsValue::from(true)
/// );
///
/// # Ok(())
/// # }
/// ```
///
/// [promise]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
#[derive(Debug, Clone)]
pub struct JsPromise {
    inner: JsObject<Promise>,
}

impl JsPromise {
    /// Creates a new promise object from an executor function.
    ///
    /// It is equivalent to calling the [`Promise()`] constructor, which makes it share the same
    /// execution semantics as the constructor:
    /// - The executor function `executor` is called synchronously just after the promise is created.
    /// - The executor return value is ignored.
    /// - A catchable error from `executor` in the current runtime calls the `reject` function
    ///   of the newly created promise, unless either `resolve` or `reject` were already called
    ///   beforehand.
    ///
    /// `executor` receives as an argument the [`ResolvingFunctions`] needed to settle the promise,
    /// which can be done by either calling the `resolve` function or the `reject` function.
    ///
    /// # Errors
    ///
    /// Returns an engine error from the executor unchanged. Returns a `TypeError` if
    /// the executor's catchable error contains an object or realm from another runtime.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, JsValue, js_string
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::new(
    ///     |resolvers, mut context| {
    ///         let result = js_string!("hello world").into();
    ///         resolvers.resolve.call(
    ///             &JsValue::undefined(),
    ///             &[result],
    ///             context.reborrow(),
    ///         )?;
    ///         Ok(JsValue::undefined())
    ///     },
    ///     context.access(),
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Fulfilled(js_string!("hello world").into())
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/Promise
    pub fn new<F>(executor: F, context: ContextAccess<'_>) -> JsResult<Self>
    where
        F: for<'call> FnOnce(&ResolvingFunctions, ContextAccess<'call>) -> JsResult<JsValue>,
    {
        let context = context.into_owner();
        let promise = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().promise().prototype(),
            Promise::new(),
        );
        let resolvers = Promise::create_resolving_functions(&promise, context);

        if let Err(e) = executor(&resolvers, context.access()) {
            context.check_execution_stop()?;
            let e = e.into_opaque(context.access())?;
            resolvers
                .reject
                .call(&JsValue::undefined(), &[e], context.access())
                .js_expect("default `reject` function cannot throw")?;
        }

        Ok(Self { inner: promise })
    }

    /// Creates a new pending promise and returns it and its associated `ResolvingFunctions`.
    ///
    /// This can be useful when you want to manually settle a promise from Rust code, instead of
    /// running an `executor` function that automatically settles the promise on creation
    /// (see [`JsPromise::new`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, JsValue
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let (promise, resolvers) = JsPromise::new_pending(context.access());
    ///
    /// assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    ///
    /// resolvers
    ///     .reject
    ///     .call(&JsValue::undefined(), &[5.into()], context.access())?;
    ///
    /// assert_eq!(promise.state().unwrap(), PromiseState::Rejected(5.into()));
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new_pending(context: ContextAccess<'_>) -> (Self, ResolvingFunctions) {
        let context = context.into_owner();
        let promise = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().promise().prototype(),
            Promise::new(),
        );
        let resolvers = Promise::create_resolving_functions(&promise, context);

        (promise.into(), resolvers)
    }

    /// Wraps an existing object with the `JsPromise` interface, returning `Err` if the object
    /// is not a valid promise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, JsObject, JsValue, Source
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = context.eval(Source::from_bytes(
    ///     "new Promise((resolve, reject) => resolve())",
    /// ))?;
    /// let promise = promise.as_object().unwrap();
    ///
    /// let promise = JsPromise::from_object(promise)?;
    ///
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Fulfilled(JsValue::undefined())
    /// );
    ///
    /// assert!(JsPromise::from_object(JsObject::with_null_proto(context.runtime())).is_err());
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        let Ok(inner) = object.downcast::<Promise>() else {
            return Err(JsNativeError::typ()
                .with_message("`object` is not a Promise")
                .into());
        };
        Ok(Self { inner })
    }

    /// Creates a new `JsPromise` from a [`Future`]-like.
    ///
    /// If you want to convert a Rust async function into an ECMAScript async function, see
    /// [`NativeFunction::from_async_fn`][async_fn].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::error::Error;
    /// # use surgeist_js::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, JsResult, JsValue
    /// # };
    /// async fn f() -> JsResult<JsValue> {
    ///     Ok(JsValue::null())
    /// }
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::from_task(|_| Ok(surgeist_js::job::ContextTask::from_future(f())), context.access()).unwrap();
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(promise.state().unwrap(), PromiseState::Fulfilled(JsValue::null()));
    /// ```
    ///
    /// [async_fn]: crate::native_function::NativeFunction::from_async_fn
    /// Returns a native error if settlement-observer preparation cannot complete.
    /// The factory runs only after its job's preparation is admitted.
    pub fn from_task<F>(factory: F, context: ContextAccess<'_>) -> JsResult<Self>
    where
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<ContextTask<JsValue>> + 'static,
    {
        let context = context.into_owner();
        context.check_execution_stop()?;
        let (promise, resolvers) = Self::new_pending(context.access());
        let association = Promise::ensure_native_association(&promise.inner, context)?;
        let realm = context.realm().clone();
        let job = NativeAsyncJob::with_realm(
            move |access| {
                let mut task = match factory(access) {
                    Ok(task) => task,
                    Err(error) if error.as_engine().is_some() => return Err(error),
                    Err(error) => ContextTask::from_future(std::future::ready(Err(error))),
                };
                let mut resolvers = Some(resolvers);
                Ok(ContextTask::from_poll(move |cx, host| {
                    let result = match task.poll(cx, host) {
                        task::Poll::Pending => return task::Poll::Pending,
                        task::Poll::Ready(result) => result,
                    };
                    let resolvers = resolvers.take().expect("promise task completes once");
                    task::Poll::Ready(host.with(|access| {
                        let context = access.into_owner();
                        context.check_execution_stop()?;
                        match context.runtime().admit_callback_result(result) {
                            Ok(value) => resolvers.resolve.call(
                                &JsValue::undefined(),
                                &[value],
                                context.access(),
                            ),
                            Err(error) => {
                                let value = error.into_opaque(context.access())?;
                                resolvers.reject.call(
                                    &JsValue::undefined(),
                                    &[value],
                                    context.access(),
                                )
                            }
                        }
                    }))
                }))
            },
            realm,
        );
        association.bind_job(&job.observer(), true)?;
        context.enqueue_job(job.into());
        Ok(promise)
    }

    /// Creates a new `JsPromise` from a `Result<T, JsError>`, where `T` is the fulfilled value of
    /// the promise, and `JsError` is the rejection reason. This is a simpler way to create a
    /// promise that is either fulfilled or rejected based on the result of a computation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, JsResult, JsString, js_string, js_error
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// fn do_thing(success: bool) -> JsResult<JsString> {
    ///     success
    ///         .then(|| js_string!("resolved!"))
    ///         .ok_or(js_error!("rejected!"))
    /// }
    ///
    /// let promise = JsPromise::from_result(do_thing(true), context.access())?;
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Fulfilled(js_string!("resolved!").into())
    /// );
    ///
    /// let promise = JsPromise::from_result(do_thing(false), context.access())?;
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Rejected(js_string!("rejected!").into())
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_result<V: Into<JsValue>, E: Into<JsError>>(
        value: Result<V, E>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        match value {
            Ok(v) => Self::resolve(v, context.access()),
            Err(e) => Self::reject(e, context.access()),
        }
    }

    /// Resolves a `JsValue` into a `JsPromise`.
    ///
    /// Equivalent to the [`Promise.resolve()`] static method.
    ///
    /// This function is mainly used to wrap a plain `JsValue` into a fulfilled promise, but it can
    /// also flatten nested layers of [thenables], which essentially converts them into native
    /// promises.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, js_string
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::resolve(js_string!("resolved!"), context.access())?;
    ///
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Fulfilled(js_string!("resolved!").into())
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.resolve()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/resolve
    /// [thenables]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise#thenables
    pub fn resolve<V: Into<JsValue>>(value: V, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        Promise::promise_resolve(
            &context.intrinsics().constructors().promise().constructor(),
            value.into(),
            context,
        )
        .and_then(Self::from_object)
        .js_expect("default resolve functions cannot throw and must return a promise")
        .map_err(Into::into)
    }

    /// Creates a `JsPromise` that is rejected with the reason `error`.
    ///
    /// Equivalent to the [`Promise.reject`] static method.
    ///
    /// `JsPromise::reject` is pretty similar to [`JsPromise::resolve`], with the difference that
    /// it wraps a catchable error into a rejected promise, even if the error value is a promise or a [thenable].
    ///
    /// # Errors
    ///
    /// Returns an engine error unchanged. Returns a `TypeError` if a catchable error
    /// contains an object or realm from another runtime.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context, js_string, JsError
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::reject(
    ///     JsError::from_opaque(js_string!("oops!").into()),
    ///     context.access(),
    /// )?;
    ///
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Rejected(js_string!("oops!").into())
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.reject`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/reject
    /// [thenable]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise#thenables
    pub fn reject<E: Into<JsError>>(error: E, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        let reason = error.into().into_opaque(context.access())?;
        Promise::promise_reject(
            &context.intrinsics().constructors().promise().constructor(),
            reason,
            context,
        )
        .and_then(Self::from_object)
        .js_expect("default reject functions cannot throw and must return a promise")
        .map_err(Into::into)
    }

    /// Gets the current state of the promise.
    ///
    /// Returns [`crate::error::EngineError::ObjectAccessBusy`] while a mutable
    /// object alias holds its payload. Refusal leaves the promise unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #    object::builtins::JsPromise,
    /// #    builtins::promise::PromiseState,
    /// #    Context
    /// # };
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::new_pending(context.access()).0;
    ///
    /// assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    /// ```
    #[inline]
    pub fn state(&self) -> JsResult<PromiseState> {
        Ok(self
            .inner
            .try_borrow()
            .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?
            .data()
            .state())
    }

    /// Schedules callback functions to run when the promise settles.
    ///
    /// Equivalent to the [`Promise.prototype.then`] method.
    ///
    /// The return value is a promise that is always pending on return, regardless of the current
    /// state of the original promise. Two handlers can be provided as callbacks to be executed when
    /// the original promise settles:
    ///
    /// - If the original promise is fulfilled, `on_fulfilled` is called with the fulfillment value
    ///   of the original promise.
    /// - If the original promise is rejected, `on_rejected` is called with the rejection reason
    ///   of the original promise.
    ///
    /// The return value of the handlers can be used to mutate the state of the created promise. If
    /// the callback:
    ///
    /// - returns a value: the created promise gets fulfilled with the returned value.
    /// - doesn't return: the created promise gets fulfilled with undefined.
    /// - throws: the created promise gets rejected with the thrown error as its value.
    /// - returns a fulfilled promise: the created promise gets fulfilled with that promise's value as its value.
    /// - returns a rejected promise: the created promise gets rejected with that promise's value as its value.
    /// - returns another pending promise: the created promise remains pending but becomes settled with that
    ///   promise's value as its value immediately after that promise becomes settled.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #     builtins::promise::PromiseState,
    /// #     js_string,
    /// #     object::{builtins::JsPromise, FunctionObjectBuilder},
    /// #     Context, JsArgs, JsError, JsValue, NativeFunction,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::new(
    ///     |resolvers, mut context| {
    ///         resolvers.resolve.call(
    ///             &JsValue::undefined(),
    ///             &[255.255.into()],
    ///             context.reborrow(),
    ///         )?;
    ///         Ok(JsValue::undefined())
    ///     },
    ///     context.access(),
    /// )?
    /// .then(
    ///     Some(
    ///         NativeFunction::from_fn_ptr(|_, args, mut context| {
    ///             args.get_or_undefined(0)
    ///                 .to_string(context.reborrow())
    ///                 .map(JsValue::from)
    ///         })
    ///         .to_js_function(context.realm()).unwrap(),
    ///     ),
    ///     None,
    ///     context.access(),
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Fulfilled(js_string!("255.255").into())
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.prototype.then`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/then
    #[inline]
    #[allow(clippy::return_self_not_must_use)] // Could just be used to add handlers on an existing promise
    pub fn then(
        &self,
        on_fulfilled: Option<JsFunction>,
        on_rejected: Option<JsFunction>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        self.inner.check_runtime(context.runtime())?;
        for handler in on_fulfilled.iter().chain(&on_rejected) {
            handler.check_runtime(context.runtime())?;
        }

        Promise::inner_then(self, on_fulfilled, on_rejected, context).and_then(Self::from_object)
    }

    /// Schedules a callback to run when the promise is rejected.
    ///
    /// Equivalent to the [`Promise.prototype.catch`] method.
    ///
    /// This is essentially a shortcut for calling [`promise.then(None, Some(function))`][then], which
    /// only handles the error case and leaves the fulfilled case untouched.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #     js_string,
    /// #     builtins::promise::PromiseState,
    /// #     object::{builtins::JsPromise, FunctionObjectBuilder},
    /// #     Context, JsArgs, JsNativeError, JsValue, NativeFunction,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::new(
    ///     |resolvers, mut context| {
    ///         let error = JsNativeError::typ().with_message("thrown");
    ///         let error = error.into_opaque(context.reborrow())?;
    ///         resolvers.reject.call(
    ///             &JsValue::undefined(),
    ///             &[error.into()],
    ///             context.reborrow(),
    ///         )?;
    ///         Ok(JsValue::undefined())
    ///     },
    ///     context.access(),
    /// )?
    /// .catch(
    ///     NativeFunction::from_fn_ptr(|_, args, mut context| {
    ///         args.get_or_undefined(0)
    ///             .to_string(context.reborrow())
    ///             .map(JsValue::from)
    ///     })
    ///     .to_js_function(context.realm()).unwrap(),
    ///     context.access(),
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Fulfilled(js_string!("TypeError: thrown").into())
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.prototype.catch`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/catch
    /// [then]: JsPromise::then
    #[inline]
    #[allow(clippy::return_self_not_must_use)] // Could just be used to add a handler on an existing promise
    pub fn catch(&self, on_rejected: JsFunction, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        self.then(None, Some(on_rejected), context.access())
    }

    /// Schedules a callback to run when the promise is rejected.
    ///
    /// Equivalent to the [`Promise.prototype.finally()`] method.
    ///
    /// While this could be seen as a shortcut for calling [`promise.then(Some(function), Some(function))`][then],
    /// it has slightly different semantics than `then`:
    /// - `on_finally` doesn't receive any argument, unlike `on_fulfilled` and `on_rejected`.
    /// - `finally()` is transparent; a call like `Promise.resolve("first").finally(() => "second")`
    ///   returns a promise fulfilled with the value `"first"`, which would return `"second"` if `finally`
    ///   was a shortcut of `then`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #     object::{builtins::JsPromise, FunctionObjectBuilder},
    /// #     property::Attribute,
    /// #     Context, JsNativeError, JsValue, NativeFunction, js_string
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// context.register_global_property(
    ///     js_string!("finally"),
    ///     false,
    ///     Attribute::all(),
    /// )?;
    ///
    /// let promise = JsPromise::new(
    ///     |resolvers, mut context| {
    ///         let error = JsNativeError::typ().with_message("thrown");
    ///         let error = error.into_opaque(context.reborrow())?;
    ///         resolvers.reject.call(
    ///             &JsValue::undefined(),
    ///             &[error.into()],
    ///             context.reborrow(),
    ///         )?;
    ///         Ok(JsValue::undefined())
    ///     },
    ///     context.access(),
    /// )?
    /// .finally(
    ///     NativeFunction::from_fn_ptr(|_, _, mut context| {
    ///         context.global_object().clone().set(
    ///             js_string!("finally"),
    ///             JsValue::from(true),
    ///             true,
    ///             context.reborrow(),
    ///         )?;
    ///         Ok(JsValue::undefined())
    ///     })
    ///     .to_js_function(context.realm()).unwrap(),
    ///     context.access(),
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(
    ///     context
    ///         .global_object()
    ///         .clone()
    ///         .get(js_string!("finally"), context.access())?,
    ///     JsValue::from(true)
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.prototype.finally()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/finally
    /// [then]: JsPromise::then
    #[inline]
    #[allow(clippy::return_self_not_must_use)] // Could just be used to add a handler on an existing promise
    pub fn finally(&self, on_finally: JsFunction, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        self.inner.check_runtime(context.runtime())?;
        on_finally.check_runtime(context.runtime())?;

        let (then, catch) = Promise::then_catch_finally_closures(
            context.intrinsics().constructors().promise().constructor(),
            on_finally,
            context,
        )?;
        Promise::inner_then(self, Some(then), Some(catch), context).and_then(Self::from_object)
    }

    /// Waits for a list of promises to settle with fulfilled values, rejecting the aggregate promise
    /// when any of the inner promises is rejected.
    ///
    /// Equivalent to the [`Promise.all`] static method.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #     js_string,
    /// #     object::builtins::{JsArray, JsPromise},
    /// #     Context, JsNativeError, JsValue,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise1 = JsPromise::all(
    ///     [
    ///         JsPromise::resolve(0, context.access())?,
    ///         JsPromise::resolve(2, context.access())?,
    ///         JsPromise::resolve(4, context.access())?,
    ///     ],
    ///     context.access(),
    /// )?;
    ///
    /// let promise2 = JsPromise::all(
    ///     [
    ///         JsPromise::resolve(1, context.access())?,
    ///         JsPromise::reject(JsNativeError::typ(), context.access())?,
    ///         JsPromise::resolve(3, context.access())?,
    ///     ],
    ///     context.access(),
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// let array = promise1
    ///     .state().unwrap()
    ///     .as_fulfilled()
    ///     .and_then(JsValue::as_object)
    ///     .unwrap()
    ///     .clone();
    /// let array = JsArray::from_object(array)?;
    /// assert_eq!(array.at(0, context.access())?, 0.into());
    /// assert_eq!(array.at(1, context.access())?, 2.into());
    /// assert_eq!(array.at(2, context.access())?, 4.into());
    ///
    /// let error = promise2.state().unwrap().as_rejected().unwrap().clone();
    /// assert_eq!(error.to_string(context.access())?, js_string!("TypeError"));
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.all`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/all
    pub fn all<I>(promises: I, context: ContextAccess<'_>) -> JsResult<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let context = context.into_owner();
        let promises =
            JsArray::from_iter(promises.into_iter().map(JsValue::from), context.access())?;

        let c = &context
            .intrinsics()
            .constructors()
            .promise()
            .constructor()
            .into();

        let value = Promise::all(c, &[promises.into()], context)
            .js_expect("Promise.all cannot fail with the default `%Promise%` constructor")?;

        let object = value
            .as_object()
            .js_expect("`Promise.all` always returns an object on success")?;

        Self::from_object(object.clone())
            .js_expect("`Promise::all` with the default `%Promise%` constructor always returns a native `JsPromise`").map_err(Into::into)
    }

    /// Waits for a list of promises to settle, fulfilling with an array of the outcomes of every
    /// promise.
    ///
    /// Equivalent to the [`Promise.allSettled`] static method.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #     js_string,
    /// #     object::builtins::{JsArray, JsPromise},
    /// #     Context, JsNativeError, JsValue,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::all_settled(
    ///     [
    ///         JsPromise::resolve(1, context.access())?,
    ///         JsPromise::reject(JsNativeError::typ(), context.access())?,
    ///         JsPromise::resolve(3, context.access())?,
    ///     ],
    ///     context.access(),
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// let array = promise
    ///     .state().unwrap()
    ///     .as_fulfilled()
    ///     .and_then(JsValue::as_object)
    ///     .unwrap()
    ///     .clone();
    /// let array = JsArray::from_object(array)?;
    ///
    /// let a = array.at(0, context.access())?.as_object().unwrap().clone();
    /// assert_eq!(
    ///     a.get(js_string!("status"), context.access())?,
    ///     js_string!("fulfilled").into()
    /// );
    /// assert_eq!(a.get(js_string!("value"), context.access())?, 1.into());
    ///
    /// let b = array.at(1, context.access())?.as_object().unwrap().clone();
    /// assert_eq!(
    ///     b.get(js_string!("status"), context.access())?,
    ///     js_string!("rejected").into()
    /// );
    /// assert_eq!(
    ///     b.get(js_string!("reason"), context.access())?.to_string(context.access())?,
    ///     js_string!("TypeError")
    /// );
    ///
    /// let c = array.at(2, context.access())?.as_object().unwrap().clone();
    /// assert_eq!(
    ///     c.get(js_string!("status"), context.access())?,
    ///     js_string!("fulfilled").into()
    /// );
    /// assert_eq!(c.get(js_string!("value"), context.access())?, 3.into());
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.allSettled`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/allSettled
    pub fn all_settled<I>(promises: I, context: ContextAccess<'_>) -> JsResult<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let context = context.into_owner();
        let promises =
            JsArray::from_iter(promises.into_iter().map(JsValue::from), context.access())?;

        let c = &context
            .intrinsics()
            .constructors()
            .promise()
            .constructor()
            .into();

        let value = Promise::all_settled(c, &[promises.into()], context).js_expect(
            "`Promise.all_settled` cannot fail with the default `%Promise%` constructor",
        )?;

        let object = value
            .as_object()
            .js_expect("`Promise.all_settled` always returns an object on success")?;

        Self::from_object(object.clone())
            .js_expect("`Promise::all_settled` with the default `%Promise%` constructor always returns a native `JsPromise`").map_err(Into::into)
    }

    /// Returns the first promise that fulfills from a list of promises.
    ///
    /// Equivalent to the [`Promise.any`] static method.
    ///
    /// If after settling all promises in `promises` there isn't a fulfilled promise, the returned
    /// promise will be rejected with an `AggregatorError` containing the rejection values of every
    /// promise; this includes the case where `promises` is an empty iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #     builtins::promise::PromiseState,
    /// #     js_string,
    /// #     object::builtins::JsPromise,
    /// #     Context, JsNativeError,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let promise = JsPromise::any(
    ///     [
    ///         JsPromise::reject(JsNativeError::syntax(), context.access())?,
    ///         JsPromise::reject(JsNativeError::typ(), context.access())?,
    ///         JsPromise::resolve(js_string!("fulfilled"), context.access())?,
    ///         JsPromise::reject(JsNativeError::range(), context.access())?,
    ///     ],
    ///     context.access(),
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Fulfilled(js_string!("fulfilled").into())
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.any`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/any
    pub fn any<I>(promises: I, context: ContextAccess<'_>) -> JsResult<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let context = context.into_owner();
        let promises =
            JsArray::from_iter(promises.into_iter().map(JsValue::from), context.access())?;

        let c = &context
            .intrinsics()
            .constructors()
            .promise()
            .constructor()
            .into();

        let value = Promise::any(c, &[promises.into()], context)
            .js_expect("`Promise.any` cannot fail with the default `%Promise%` constructor")?;

        let object = value
            .as_object()
            .js_expect("`Promise.any` always returns an object on success")?;

        Self::from_object(object.clone())
            .js_expect("`Promise::any` with the default `%Promise%` constructor always returns a native `JsPromise`").map_err(Into::into)
    }

    /// Returns the first promise that settles from a list of promises.
    ///
    /// Equivalent to the [`Promise.race`] static method.
    ///
    /// If the provided iterator is empty, the returned promise will remain on the pending state
    /// forever.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #     builtins::promise::PromiseState,
    /// #     js_string,
    /// #     object::builtins::JsPromise,
    /// #     Context, JsValue,
    /// # };
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let (a, resolvers_a) = JsPromise::new_pending(context.access());
    /// let (b, resolvers_b) = JsPromise::new_pending(context.access());
    /// let (c, resolvers_c) = JsPromise::new_pending(context.access());
    ///
    /// let promise = JsPromise::race([a, b, c], context.access())?;
    ///
    /// resolvers_b
    ///     .reject
    ///     .call(&JsValue::undefined(), &[], context.access())?;
    /// resolvers_a
    ///     .resolve
    ///     .call(&JsValue::undefined(), &[5.into()], context.access())?;
    /// resolvers_c.reject.call(
    ///     &JsValue::undefined(),
    ///     &[js_string!("c error").into()],
    ///     context.access(),
    /// )?;
    ///
    /// context.run_jobs();
    ///
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Rejected(JsValue::undefined())
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Promise.race`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/race
    pub fn race<I>(promises: I, context: ContextAccess<'_>) -> JsResult<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let context = context.into_owner();
        let promises =
            JsArray::from_iter(promises.into_iter().map(JsValue::from), context.access())?;

        let c = &context
            .intrinsics()
            .constructors()
            .promise()
            .constructor()
            .into();

        let value = Promise::race(c, &[promises.into()], context)
            .js_expect("`Promise.race` cannot fail with the default `%Promise%` constructor")?;

        let object = value
            .as_object()
            .js_expect("`Promise.race` always returns an object on success")?;

        Self::from_object(object.clone())
            .js_expect("`Promise::race` with the default `%Promise%` constructor always returns a native `JsPromise`").map_err(Into::into)
    }

    /// Creates a `JsFuture` from this `JsPromise`.
    ///
    /// The returned `JsFuture` implements [`Future`], which means it can be `await`ed within Rust's
    /// async contexts (async functions and async blocks).
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # use surgeist_js::{
    /// #     builtins::promise::PromiseState,
    /// #     object::builtins::JsPromise,
    /// #     Context, JsValue, JsError
    /// # };
    /// # use futures_lite::future;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let context = &mut Context::default();
    ///
    /// let (promise, resolvers) = JsPromise::new_pending(context.access());
    /// let promise_future = promise.into_js_future(context.access())?;
    ///
    /// let future1 = async move { promise_future.await };
    ///
    /// let future2 = async move {
    ///     resolvers
    ///         .resolve
    ///         .call(&JsValue::undefined(), &[10.into()], context.access())?;
    ///     context.run_jobs();
    ///     Ok::<(), JsError>(())
    /// };
    ///
    /// let (result1, result2) = future::block_on(future::zip(future1, future2));
    ///
    /// assert_eq!(result1, Ok(JsValue::from(10)));
    /// assert_eq!(result2, Ok(()));
    ///
    /// # Ok(())
    /// # }
    /// ```
    /// Native task retirement reports [`crate::error::EngineError::JobRetired`],
    /// with already-published guest settlement taking precedence. Terminal source
    /// evaluation instead reports [`crate::error::EngineError::ModuleInterrupted`]
    /// with its immutable identity and original typed cause, including for a waiter
    /// registered before interruption. An unsettled guest promise stays pending;
    /// a larger budget cannot retry that module. An already-terminal request returns
    /// the error before registering guest handlers.
    pub fn into_js_future(self, context: ContextAccess<'_>) -> JsResult<JsFuture> {
        let context = context.into_owner();
        self.inner.check_runtime(context.runtime())?;
        let association = Promise::ensure_native_association(&self.inner, context)?;
        if let Some(error) = Association::module_error(&association) {
            return Err(error);
        }
        let native_wait = association.signal().wait()?;

        // Keep the ordinary species/host-hook/handled registration. Native
        // observation itself follows P, independently of these delivery jobs.
        let resolve = NativeFunction::from_builtin(|_, _, _| Ok(JsValue::undefined()))
            .to_js_function(context.realm())?;
        let reject = NativeFunction::from_builtin(|_, _, _| Ok(JsValue::undefined()))
            .to_js_function(context.realm())?;
        drop(self.then(Some(resolve), Some(reject), context.access())?);

        Ok(JsFuture {
            association,
            promise: self,
            native_wait,
        })
    }

    /// Run jobs until this promise is resolved or rejected. This could
    /// result in an infinite loop if the promise is never resolved or
    /// rejected (e.g. with a [`surgeist_js::job::JobExecutor`] that does
    /// not prioritize properly). If you need more control over how
    /// the promise handles timing out, consider using
    /// [`crate::Context::run_jobs`] directly.
    ///
    /// Returns [`Result::Ok`] if the promise resolved, or [`Result::Err`]
    /// if the promise was rejected. If the promise was already resolved,
    /// [`crate::Context::run_jobs`] is guaranteed to not be executed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js::{Context, JsArgs, JsValue, NativeFunction};
    /// # use surgeist_js::object::builtins::{JsFunction, JsPromise};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let context = &mut Context::default();
    ///
    /// let p1 = JsPromise::new(
    ///     |fns, mut context| {
    ///         fns.resolve
    ///             .call(&JsValue::undefined(), &[JsValue::new(1)], context.reborrow())
    ///     },
    ///     context.access(),
    /// )?;
    /// let p2 = p1.then(
    ///     Some(
    ///         NativeFunction::from_fn_ptr(|_, args, mut context| {
    ///             assert_eq!(*args.get_or_undefined(0), JsValue::new(1));
    ///             Ok(JsValue::new(2))
    ///         })
    ///         .to_js_function(context.realm()).unwrap(),
    ///     ),
    ///     None,
    ///     context.access(),
    /// )?;
    ///
    /// assert_eq!(p2.await_blocking(context.access()), Ok(JsValue::new(2)));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// This will not panic as `run_jobs()` is not executed.
    /// ```
    /// # use surgeist_js::{Context, JsValue, NativeFunction};
    /// # use surgeist_js::object::builtins::JsPromise;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let context = &mut Context::default();
    /// let p1 = JsPromise::new(
    ///     |fns, mut context| fns.resolve.call(&JsValue::undefined(), &[], context.reborrow()),
    ///     context.access(),
    /// )?
    /// .then(
    ///     Some(
    ///         NativeFunction::from_fn_ptr(|_, _, _| {
    ///             panic!("This will not happen.");
    ///         })
    ///         .to_js_function(context.realm()).unwrap(),
    ///     ),
    ///     None,
    ///     context.access(),
    /// )?;
    /// let p2 = JsPromise::resolve(1, context.access())?;
    ///
    /// assert_eq!(p2.await_blocking(context.access()), Ok(JsValue::new(1)));
    /// // Uncommenting the following line would panic.
    /// // context.run_jobs();
    /// # Ok(())
    /// # }
    /// ```
    /// A terminal source-module evaluation returns
    /// [`crate::error::EngineError::ModuleInterrupted`] without retrying or waiting
    /// exclusively on its unchanged pending guest promise. Its identity and original
    /// typed cause remain stable; increasing this call's budget cannot revive it.
    pub fn await_blocking(&self, context: ContextAccess<'_>) -> Result<JsValue, JsError> {
        let context = context.into_owner();
        loop {
            context.check_execution_stop()?;
            if let Some(association) = Promise::native_association(&self.inner)?
                && let Some(error) = Association::module_error(&association)
            {
                return Err(error);
            }
            match self.state()? {
                PromiseState::Pending => {
                    if let Some(association) = Promise::native_association(&self.inner)?
                        && let SettlementStatus::Retired(reason) = association
                            .signal()
                            .status()
                            .map_err(crate::JsError::from)?
                    {
                        return Err(crate::error::EngineError::JobRetired(reason).into());
                    }
                    context.run_jobs()?;
                }
                PromiseState::Fulfilled(f) => break Ok(f),
                PromiseState::Rejected(r) => break Err(JsError::from_opaque(r)),
            }
        }
    }

    #[cfg(feature = "experimental")]
    pub(crate) fn await_native(
        &self,
        continuation: crate::native_function::NativeCoroutine,
        reject: &JsFunction,
        context: &mut crate::Context,
    ) -> JsResult<()> {
        use crate::{
            builtins::generator::{GeneratorContext, StoredGeneratorContext},
            js_string,
            object::FunctionObjectBuilder,
            vm::CompletionRecord,
        };
        use std::{cell::RefCell, ops::ControlFlow};
        use surgeist_js_gc::heap::Rooted;

        // Native callbacks need ambient state, but do not suspend the caller's
        // bytecode invocation or acquire its continuation publication authority.
        let generator = GeneratorContext::snapshot_current(context);
        let frame = context
            .runtime()
            .heap
            .allocate(RefCell::new(Some(generator.stored())))
            .expect("generator context belongs to the continuation runtime");

        type Captures = (
            crate::native_function::NativeCoroutine,
            Rooted<RefCell<Option<StoredGeneratorContext>>>,
            JsFunction,
            bool,
        );
        let reaction = |rejected: bool| {
            NativeFunction::from_builtin_with_captures(
                |_this, args, (continuation, frame, reject, rejected): &Captures, context| {
                    let mut generator = frame
                        .borrow()
                        .as_ref()
                        .map(StoredGeneratorContext::to_context)
                        .js_expect("continuation should only run once")?;
                    *frame.borrow_mut() = None;
                    let value = args.get_or_undefined(0).clone();
                    let completion = if *rejected {
                        CompletionRecord::Throw(JsError::from_opaque(value))
                    } else {
                        CompletionRecord::Normal(value)
                    };
                    let result = generator.with_attached(context, |context| {
                        match continuation.call(completion, context) {
                            ControlFlow::Continue(value) => {
                                JsPromise::resolve(value, context.access()).and_then(|promise| {
                                    promise.await_native(continuation.clone(), reject, context)
                                })
                            }
                            ControlFlow::Break(result) => result,
                        }
                    });

                    if let Err(error) = result {
                        let reason = error.into_opaque(context.access())?;
                        reject.call(&JsValue::undefined(), &[reason], context.access())?;
                    }
                    Ok(JsValue::undefined())
                },
                (
                    continuation.clone(),
                    frame.clone(),
                    reject.clone(),
                    rejected,
                ),
            )
        };
        let on_fulfilled = FunctionObjectBuilder::new(context.realm(), reaction(false))
            .name(js_string!())
            .length(1)
            .build()
            .expect("native await reaction retains the current runtime");
        let on_rejected = FunctionObjectBuilder::new(context.realm(), reaction(true))
            .name(js_string!())
            .length(1)
            .build()
            .expect("native await reaction retains the current runtime");

        let result = Promise::perform_promise_then(
            &self.inner,
            Some(on_fulfilled),
            Some(on_rejected),
            None,
            context,
        );
        if result.is_err() {
            // A retained host callback must not later run an installation that
            // failed. A reaction already taken by an inline executor stays taken.
            let abandoned = frame.borrow_mut().take();
            drop(abandoned);
        }
        result
    }
}

impl From<JsObject<Promise>> for JsPromise {
    fn from(value: JsObject<Promise>) -> Self {
        Self { inner: value }
    }
}

impl From<JsPromise> for JsObject {
    #[inline]
    fn from(o: JsPromise) -> Self {
        o.inner.clone().upcast()
    }
}

impl From<JsPromise> for JsValue {
    #[inline]
    fn from(o: JsPromise) -> Self {
        o.inner.clone().into()
    }
}

impl std::ops::Deref for JsPromise {
    type Target = JsObject<Promise>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsPromise {
    fn try_from_js(value: &JsValue, _context: ContextAccess<'_>) -> JsResult<Self> {
        if let Some(o) = value.as_object() {
            Self::from_object(o.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("value is not a Promise object")
                .into())
        }
    }
}

impl TryIntoJs for JsPromise {
    fn try_into_js(&self, _: ContextAccess<'_>) -> JsResult<JsValue> {
        Ok(self.clone().into())
    }
}

/// A Rust's `Future` that becomes ready when a `JsPromise` fulfills.
///
/// This type allows `await`ing `JsPromise`s inside Rust's async contexts, which makes interfacing
/// between promises and futures a bit easier.
///
/// The only way to construct an instance of `JsFuture` is by calling [`JsPromise::into_js_future`].
/// A source-module interruption completes native observation with
/// [`crate::error::EngineError::ModuleInterrupted`], preserving its immutable cause
/// and leaving the guest promise pending. Ordinary completed modules keep their
/// published result; a larger budget does not resume an interrupted instance.
pub struct JsFuture {
    association: surgeist_js_gc::heap::Rooted<Association>,
    promise: JsPromise,
    native_wait: SettlementWait,
}

impl std::fmt::Debug for JsFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsFuture").finish_non_exhaustive()
    }
}

impl Future for JsFuture {
    type Output = JsResult<JsValue>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        fn settled(promise: &JsPromise) -> Option<JsResult<JsValue>> {
            match promise.state() {
                Ok(PromiseState::Pending) => None,
                Ok(PromiseState::Fulfilled(value)) => Some(Ok(value)),
                Ok(PromiseState::Rejected(value)) => Some(Err(JsError::from_opaque(value))),
                Err(error) => Some(Err(error)),
            }
        }

        let this = self.get_mut();
        if let Some(error) = Association::module_error(&this.association) {
            return task::Poll::Ready(Err(error));
        }
        if let Some(result) = settled(&this.promise) {
            return task::Poll::Ready(result);
        }
        let status = match this.native_wait.poll(cx) {
            task::Poll::Pending => return task::Poll::Pending,
            task::Poll::Ready(status) => status,
        };
        // Waker registration can reenter host code. Published settlement still
        // wins if that reentry settled P while the native signal was observed.
        if let Some(result) = settled(&this.promise) {
            return task::Poll::Ready(result);
        }
        if let Some(error) = Association::module_error(&this.association) {
            return task::Poll::Ready(Err(error));
        }
        match status {
            Ok(
                SettlementStatus::ModuleInterrupted
                | SettlementStatus::Settled
                | SettlementStatus::Pending,
            ) => {
                let fault = crate::job::JobFault::new(
                    crate::job::JobFaultOwner::NativeSignal,
                    crate::job::JobFaultCause::StateInconsistency,
                );
                task::Poll::Ready(Err(fault.into()))
            }
            Ok(SettlementStatus::Retired(reason)) => {
                task::Poll::Ready(Err(crate::error::EngineError::JobRetired(reason).into()))
            }
            Err(fault) => task::Poll::Ready(Err(fault.into())),
        }
    }
}
