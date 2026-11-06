//! Boa's wrappers for native Rust functions to be compatible with ECMAScript calls.
//!
//! [`NativeFunction`] is the main type of this module, providing APIs to create native callables
//! from native Rust functions and closures.

use std::{any::Any, cell::RefCell, rc::Rc};

use surgeist_js_gc::heap::{AccessError, EdgeVisitor, Trace as HeapTrace};
use surgeist_js_string::JsString;

use crate::job::BoxedFuture;
use crate::object::internal_methods::InternalMethodCallContext;
use crate::value::JsVariant;
use crate::{
    Context, JsNativeError, JsObject, JsResult, JsValue,
    builtins::{OrdinaryObject, function::ConstructorKind},
    context::intrinsics::StandardConstructors,
    object::{
        FunctionObjectBuilder, JsData, JsFunction, JsPromise,
        internal_methods::{
            CallValue, InternalObjectMethods, ORDINARY_INTERNAL_METHODS,
            get_prototype_from_constructor,
        },
    },
    realm::{Realm, RealmEdge},
};

mod capture;
mod continuation;

pub use capture::{CapturedObject, CapturedRealm, CapturedValue, NativeCapture};

#[cfg(feature = "experimental")]
pub(crate) use continuation::CoroutineSlot;
pub(crate) use continuation::{
    CoroutineBranch, CoroutineState, NativeCoroutine, StoredNativeCoroutine,
};

/// The required signature for all native built-in function pointers.
///
/// # Arguments
///
/// - The first argument represents the `this` variable of every ECMAScript function.
///
/// - The second argument represents the list of all arguments passed to the function.
///
/// - The last argument is the engine [`Context`].
pub type NativeFunctionPointer = fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>;

/// A native callback that returns a future borrowing its invocation inputs.
pub type NativeAsyncFunctionPointer =
    for<'a, 'ctx> fn(&'a JsValue, &'a [JsValue], &'a RefCell<&'ctx mut Context>) -> BoxedFuture<'a>;

/// A native asynchronous callback whose stored state is passed explicitly.
pub type NativeAsyncFunctionWithCaptures<T> = for<'a, 'ctx> fn(
    &'a JsValue,
    &'a [JsValue],
    &'a T,
    &'a RefCell<&'ctx mut Context>,
) -> BoxedFuture<'a>;

trait TraceableClosure: HeapTrace {
    fn root_captures(&self) -> Rc<dyn Any>;
    fn call(&self, this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue>;
}

#[derive(HeapTrace)]
struct Closure<T: NativeCapture> {
    #[heap_trace(skip)]
    callback: fn(&JsValue, &[JsValue], &T, &mut Context) -> JsResult<JsValue>,
    captures: T::Stored,
}

impl<T: NativeCapture> TraceableClosure for Closure<T> {
    fn root_captures(&self) -> Rc<dyn Any> {
        Rc::new(T::restore(&self.captures))
    }

    fn call(&self, this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let captures = T::restore(&self.captures);
        (self.callback)(this, args, &captures, context)
    }
}

#[derive(HeapTrace)]
struct AsyncClosure<T: NativeCapture> {
    #[heap_trace(skip)]
    callback: NativeAsyncFunctionWithCaptures<T>,
    captures: T::Stored,
}

impl<T: NativeCapture> TraceableClosure for AsyncClosure<T> {
    fn root_captures(&self) -> Rc<dyn Any> {
        Rc::new(T::restore(&self.captures))
    }

    fn call(&self, this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let callback = self.callback;
        let captures = T::restore(&self.captures);
        let this = this.clone();
        let args = args.to_vec();
        Ok(JsPromise::from_async_fn(
            async move |context| callback(&this, &args, &captures, context).await,
            context,
        )
        .into())
    }
}

#[derive(Clone, Debug, HeapTrace)]
/// The data of an object containing a `NativeFunction`.
pub struct NativeFunctionObject {
    /// The Rust function and its stored captures.
    pub(crate) f: StoredNativeFunction,

    /// JavaScript name of the function.
    #[heap_trace(skip)]
    pub(crate) name: JsString,

    /// The kind of the function constructor if it is a constructor.
    #[heap_trace(skip)]
    pub(crate) constructor: Option<ConstructorKind>,

    /// The realm where the function is defined, if it has been initialized.
    pub(crate) realm: Option<RealmEdge>,
}

impl JsData for NativeFunctionObject {
    fn internal_methods(&self) -> &'static InternalObjectMethods {
        static FUNCTION: InternalObjectMethods = InternalObjectMethods {
            __call__: native_function_call,
            ..ORDINARY_INTERNAL_METHODS
        };

        static CONSTRUCTOR: InternalObjectMethods = InternalObjectMethods {
            __call__: native_function_call,
            __construct__: native_function_construct,
            ..ORDINARY_INTERNAL_METHODS
        };

        if self.constructor.is_some() {
            &CONSTRUCTOR
        } else {
            &FUNCTION
        }
    }
}

/// A callable Rust function that can be invoked by the engine.
///
/// Native functions use synchronous or asynchronous function pointers.
/// Supply owned state separately as traceable captures.
#[derive(Clone)]
pub struct NativeFunction {
    inner: StoredNativeFunction,
    _capture_roots: Option<Rc<dyn Any>>,
}

#[derive(Clone)]
/// Non-owning capture form of a native function.
pub struct StoredNativeFunction(Inner);

impl NativeCapture for NativeFunction {
    type Stored = StoredNativeFunction;

    fn store(&self) -> Self::Stored {
        self.stored()
    }

    fn restore(stored: &Self::Stored) -> Self {
        let roots = match &stored.0 {
            Inner::Closure(closure) => Some(closure.root_captures()),
            Inner::PointerFn(_) | Inner::AsyncPointer(_) => None,
        };
        Self {
            inner: stored.clone(),
            _capture_roots: roots,
        }
    }
}

#[derive(Clone)]
enum Inner {
    PointerFn(NativeFunctionPointer),
    AsyncPointer(NativeAsyncFunctionPointer),
    Closure(Rc<dyn TraceableClosure>),
}

impl HeapTrace for StoredNativeFunction {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        match &self.0 {
            Inner::Closure(closure) => closure.trace(edges),
            Inner::PointerFn(_) | Inner::AsyncPointer(_) => Ok(()),
        }
    }
}

impl std::fmt::Debug for StoredNativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredNativeFunction")
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeFunction").finish_non_exhaustive()
    }
}

impl NativeFunction {
    /// Creates a `NativeFunction` from a function pointer.
    #[inline]
    pub fn from_fn_ptr(function: NativeFunctionPointer) -> Self {
        Self {
            inner: StoredNativeFunction(Inner::PointerFn(function)),
            _capture_roots: None,
        }
    }

    /// Creates a `NativeFunction` from a pointer to an asynchronous callback.
    ///
    /// The returned `NativeFunction` will return an ECMAScript `Promise` that will be fulfilled
    /// or rejected when the returned `Future` completes.
    ///
    /// To schedule a one-shot asynchronous operation, see
    /// [`JsPromise::from_async_fn`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::{any::Any, cell::RefCell, rc::Rc};
    /// # use surgeist_js::{
    /// #   JsValue,
    /// #   Context,
    /// #   JsResult,
    /// #   JsSymbol,
    /// #   NativeFunction,
    /// #   JsArgs,
    /// #   builtins::promise::PromiseState,
    /// #   object::builtins::JsPromise,
    /// # };
    /// async fn test(
    ///     _this: &JsValue,
    ///     args: &[JsValue],
    ///     context: &RefCell<&mut Context>,
    /// ) -> JsResult<JsValue> {
    ///     let arg = args.get_or_undefined(0).clone();
    ///     std::future::ready(()).await;
    ///     let value = arg.to_u32(&mut context.borrow_mut())?;
    ///     Ok(JsValue::from(value * 2))
    /// }
    /// let callback = NativeFunction::from_async_fn(
    ///     |this, args, context| Box::pin(test(this, args, context)),
    /// );
    /// let mut context = Context::default();
    /// let receiver = JsValue::undefined();
    /// let fulfilled = callback.call(&receiver, &[JsValue::from(3)], &mut context).unwrap();
    /// let rejected = callback.call(
    ///     &receiver,
    ///     &[JsValue::from(JsSymbol::iterator())],
    ///     &mut context,
    /// ).unwrap();
    /// let fulfilled = JsPromise::from_object(fulfilled.as_object().unwrap()).unwrap();
    /// let rejected = JsPromise::from_object(rejected.as_object().unwrap()).unwrap();
    /// context.run_jobs().unwrap();
    /// assert_eq!(fulfilled.state(), PromiseState::Fulfilled(JsValue::from(6)));
    /// assert!(matches!(rejected.state(), PromiseState::Rejected(_)));
    /// ```
    pub fn from_async_fn(callback: NativeAsyncFunctionPointer) -> Self {
        Self {
            inner: StoredNativeFunction(Inner::AsyncPointer(callback)),
            _capture_roots: None,
        }
    }

    /// Creates an asynchronous native function with explicit traceable captures.
    ///
    /// Each invocation keeps the captures alive while its job is pending.
    pub fn from_async_fn_with_captures<T: NativeCapture>(
        callback: NativeAsyncFunctionWithCaptures<T>,
        captures: T,
    ) -> Self {
        Self {
            inner: StoredNativeFunction(Inner::Closure(Rc::new(AsyncClosure::<T> {
                callback,
                captures: captures.store(),
            }))),
            _capture_roots: Some(Rc::new(captures)),
        }
    }

    /// Creates a native function with all stored state in its traceable captures.
    ///
    /// ```compile_fail
    /// use surgeist_js::{JsValue, NativeFunction};
    ///
    /// let hidden_state = 7;
    /// NativeFunction::from_fn_with_captures(
    ///     move |_, _, (), _| Ok(JsValue::from(hidden_state)),
    ///     (),
    /// );
    /// ```
    pub fn from_fn_with_captures<T: NativeCapture>(
        callback: fn(&JsValue, &[JsValue], &T, &mut Context) -> JsResult<JsValue>,
        captures: T,
    ) -> Self {
        Self {
            inner: StoredNativeFunction(Inner::Closure(Rc::new(Closure::<T> {
                callback,
                captures: captures.store(),
            }))),
            _capture_roots: Some(Rc::new(captures)),
        }
    }

    /// Calls this `NativeFunction`, forwarding the arguments to the corresponding function.
    #[inline]
    pub fn call(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        context.runtime().check_value(this)?;
        for arg in args {
            context.runtime().check_value(arg)?;
        }
        let result = self.inner.call(this, args, context);
        context.runtime().admit_callback_result(result)
    }

    pub(crate) fn stored(&self) -> StoredNativeFunction {
        self.inner.clone()
    }

    /// Converts this `NativeFunction` into a `JsFunction` without setting its name or length.
    ///
    /// Useful to create functions that will only be used once, such as callbacks.
    pub fn to_js_function(self, realm: &Realm) -> JsResult<JsFunction> {
        FunctionObjectBuilder::new(realm, self).build()
    }
}

impl StoredNativeFunction {
    fn call(&self, this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        match self.0 {
            Inner::PointerFn(f) => f(this, args, context),
            Inner::AsyncPointer(callback) => {
                let this = this.clone();
                let args = args.to_vec();
                Ok(JsPromise::from_async_fn(
                    async move |context| callback(&this, &args, context).await,
                    context,
                )
                .into())
            }
            Inner::Closure(ref c) => c.call(this, args, context),
        }
    }
}

/// Call this object.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-built-in-function-objects-call-thisargument-argumentslist>
pub(crate) fn native_function_call(
    obj: &JsObject,
    argument_count: usize,
    context: &mut InternalMethodCallContext<'_>,
) -> JsResult<CallValue> {
    let args = context
        .vm
        .stack
        .calling_convention_pop_arguments(argument_count);
    let _func = context.vm.stack.pop();
    let this = context.vm.stack.pop();

    // We technically don't need this since native functions don't push any new frames to the
    // vm, but we'll eventually have to combine the native stack with the vm stack.
    context.check_runtime_limits()?;
    let this_function_object = obj.clone();

    let (function, name, constructor, realm) = {
        let record = obj
            .downcast_ref::<NativeFunctionObject>()
            .expect("the object should be a native function object");
        (
            record.f.clone(),
            record.name.clone(),
            record.constructor,
            record.realm.clone().map(Realm::from_edge),
        )
    };

    let pc = context.vm.frame().pc;
    let native_source_info = context.native_source_info();
    context
        .vm
        .shadow_stack
        .push_native(pc, name, native_source_info);

    let mut realm = realm.unwrap_or_else(|| context.realm().clone());

    context.swap_realm(&mut realm);
    let previous_function = context
        .vm
        .native_active_function
        .replace(this_function_object);

    let result = if constructor.is_some() {
        function.call(&JsValue::undefined(), &args, context)
    } else {
        function.call(&this, &args, context)
    };
    let result = context
        .runtime()
        .admit_callback_result(result)
        .map_err(|err| err.inject_realm(context.realm().clone()));

    context.vm.native_active_function = previous_function;
    context.swap_realm(&mut realm);

    context.vm.shadow_stack.pop();

    context.vm.stack.push(result?);

    Ok(CallValue::Complete)
}

/// Construct an instance of this object with the specified arguments.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-built-in-function-objects-construct-argumentslist-newtarget>
fn native_function_construct(
    obj: &JsObject,
    argument_count: usize,
    context: &mut InternalMethodCallContext<'_>,
) -> JsResult<CallValue> {
    // We technically don't need this since native functions don't push any new frames to the
    // vm, but we'll eventually have to combine the native stack with the vm stack.
    context.check_runtime_limits()?;
    let this_function_object = obj.clone();

    let (function, name, constructor, realm) = {
        let record = obj
            .downcast_ref::<NativeFunctionObject>()
            .expect("the object should be a native function object");
        (
            record.f.clone(),
            record.name.clone(),
            record.constructor,
            record.realm.clone().map(Realm::from_edge),
        )
    };

    let pc = context.vm.frame().pc;
    let native_source_info = context.native_source_info();
    context
        .vm
        .shadow_stack
        .push_native(pc, name, native_source_info);

    let mut realm = realm.unwrap_or_else(|| context.realm().clone());

    context.swap_realm(&mut realm);
    let previous_function = context
        .vm
        .native_active_function
        .replace(this_function_object);

    let new_target = context.vm.stack.pop();
    let args = context
        .vm
        .stack
        .calling_convention_pop_arguments(argument_count);
    let _func = context.vm.stack.pop();
    let _this = context.vm.stack.pop();

    let result = function.call(&new_target, &args, context);
    let result = context
        .runtime()
        .admit_callback_result(result)
        .map_err(|err| err.inject_realm(context.realm().clone()))
        .and_then(|v| match v.variant() {
            JsVariant::Object(o) => Ok(o.clone()),
            val => {
                if constructor.expect("must be a constructor").is_base() || val.is_undefined() {
                    let prototype = get_prototype_from_constructor(
                        &new_target,
                        StandardConstructors::object,
                        context,
                    )?;
                    Ok(JsObject::from_proto_and_data_with_shared_shape(
                        context.root_shape(),
                        prototype,
                        OrdinaryObject,
                    )
                    .upcast())
                } else {
                    Err(JsNativeError::typ()
                        .with_message("derived constructor can only return an Object or undefined")
                        .into())
                }
            }
        });

    context.vm.native_active_function = previous_function;
    context.swap_realm(&mut realm);

    context.vm.shadow_stack.pop();

    context.vm.stack.push(result?);

    Ok(CallValue::Complete)
}
