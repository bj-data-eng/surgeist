//! Boa's wrappers for native Rust functions to be compatible with ECMAScript calls.
//!
//! [`NativeFunction`] is the main type of this module, providing APIs to create native callables
//! from native Rust functions and closures.

use std::{any::Any, rc::Rc};

use surgeist_js_gc::heap::{AccessError, EdgeVisitor, Trace as HeapTrace};
use surgeist_js_string::JsString;

use crate::job::ContextTask;
use crate::object::internal_methods::InternalMethodCallContext;
use crate::value::JsVariant;
use crate::{
    Context, ContextAccess, JsNativeError, JsObject, JsResult, JsValue,
    builtins::{OrdinaryObject, function::ConstructorKind},
    context::intrinsics::StandardConstructors,
    error::RootingError,
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

/// The required signature for native host function pointers.
///
/// # Arguments
///
/// - The first argument represents the `this` variable of every ECMAScript function.
///
/// - The second argument represents the list of all arguments passed to the function.
///
/// - The last argument is a short [`ContextAccess`] capability for the reserved engine context.
pub type NativeFunctionPointer =
    for<'call> fn(&JsValue, &[JsValue], ContextAccess<'call>) -> JsResult<JsValue>;

/// Engine-owned builtin implementation, never supplied through a public host constructor.
pub(crate) type BuiltinFunctionPointer =
    fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>;

/// An owned native task factory receiving a short invocation capability.
pub type NativeAsyncFunctionPointer =
    for<'call> fn(JsValue, Vec<JsValue>, ContextAccess<'call>) -> JsResult<ContextTask<JsValue>>;

/// A native asynchronous callback whose stored state is passed explicitly.
pub type NativeAsyncFunctionWithCaptures<T> =
    for<'call> fn(JsValue, Vec<JsValue>, T, ContextAccess<'call>) -> JsResult<ContextTask<JsValue>>;

trait TraceableClosure: HeapTrace {
    fn root_captures(&self) -> Result<Rc<dyn Any>, RootingError>;
    fn call(&self, this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue>;
}

/// One stored/live capture boundary shared by synchronous and async callbacks.
#[derive(HeapTrace)]
struct CaptureStore<T: NativeCapture> {
    stored: T::Stored,
}

impl<T: NativeCapture> CaptureStore<T> {
    fn new(captures: &T) -> Self {
        Self {
            stored: captures.store(),
        }
    }

    fn restore(&self) -> Result<T, RootingError> {
        T::restore(&self.stored)
    }

    fn root_captures(&self) -> Result<Rc<dyn Any>, RootingError> {
        Ok(Rc::new(self.restore()?))
    }
}

/// Closed authority roles for synchronous captured callback bodies.
enum CapturedCall<T> {
    Host(for<'call> fn(&JsValue, &[JsValue], &T, ContextAccess<'call>) -> JsResult<JsValue>),
    Builtin(fn(&JsValue, &[JsValue], &T, &mut Context) -> JsResult<JsValue>),
}

#[derive(HeapTrace)]
struct Closure<T: NativeCapture> {
    #[heap_trace(skip)]
    callback: CapturedCall<T>,
    captures: CaptureStore<T>,
}

impl<T: NativeCapture> TraceableClosure for Closure<T> {
    fn root_captures(&self) -> Result<Rc<dyn Any>, RootingError> {
        self.captures.root_captures()
    }

    fn call(&self, this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let captures = self.captures.restore()?;
        match self.callback {
            CapturedCall::Host(callback) => callback(this, args, &captures, context.access()),
            CapturedCall::Builtin(callback) => callback(this, args, &captures, context),
        }
    }
}

#[derive(HeapTrace)]
struct AsyncClosure<T: NativeCapture> {
    #[heap_trace(skip)]
    callback: NativeAsyncFunctionWithCaptures<T>,
    captures: CaptureStore<T>,
}

impl<T: NativeCapture> TraceableClosure for AsyncClosure<T> {
    fn root_captures(&self) -> Result<Rc<dyn Any>, RootingError> {
        self.captures.root_captures()
    }

    fn call(&self, this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let callback = self.callback;
        let captures = self.captures.restore()?;
        let this = this.clone();
        let args = args.to_vec();
        JsPromise::from_task(
            move |context| callback(this, args, captures, context),
            context.access(),
        )
        .map(Into::into)
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
pub struct StoredNativeFunction(
    Inner,
    Option<surgeist_js_gc::heap::RecordRef<std::cell::RefCell<Option<crate::object::ObjectEdge>>>>,
);

impl NativeCapture for NativeFunction {
    type Stored = StoredNativeFunction;

    fn store(&self) -> Self::Stored {
        self.stored()
    }

    fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
        let roots = match &stored.0 {
            Inner::Closure(closure) => Some(closure.root_captures()?),
            Inner::Builtin(_) | Inner::PointerFn(_) | Inner::AsyncPointer(_) => None,
        };
        let mut function = Self {
            inner: stored.clone(),
            _capture_roots: roots,
        };
        if let Some(target) = &stored.1 {
            function._capture_roots = Some(Rc::new((
                function._capture_roots.take(),
                target
                    .upgrade()
                    .map_err(RootingError::Access)?
                    .ok_or(RootingError::Expired)?,
            )));
        }
        Ok(function)
    }
}

#[derive(Clone)]
enum Inner {
    Builtin(BuiltinFunctionPointer),
    PointerFn(NativeFunctionPointer),
    AsyncPointer(NativeAsyncFunctionPointer),
    Closure(Rc<dyn TraceableClosure>),
}

impl HeapTrace for StoredNativeFunction {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        self.1.trace(edges)?;
        match &self.0 {
            Inner::Closure(closure) => closure.trace(edges),
            Inner::Builtin(_) | Inner::PointerFn(_) | Inner::AsyncPointer(_) => Ok(()),
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
    pub(crate) fn with_resolution_target(
        mut self,
        target: &surgeist_js_gc::heap::Rooted<
            std::cell::RefCell<Option<crate::object::ObjectEdge>>,
        >,
    ) -> Self {
        self.inner.1 = Some(target.record_ref());
        self._capture_roots = Some(Rc::new((self._capture_roots.take(), target.clone())));
        self
    }

    /// Keeps trusted builtin dispatch distinct from caller-supplied host callbacks.
    #[inline]
    pub(crate) fn from_builtin(function: BuiltinFunctionPointer) -> Self {
        Self {
            inner: StoredNativeFunction(Inner::Builtin(function), None),
            _capture_roots: None,
        }
    }

    /// Stores a fixed engine callback body with the same capture custody as host callbacks.
    pub(crate) fn from_builtin_with_captures<T: NativeCapture>(
        callback: fn(&JsValue, &[JsValue], &T, &mut Context) -> JsResult<JsValue>,
        captures: T,
    ) -> Self {
        Self::from_captured(CapturedCall::Builtin(callback), captures)
    }

    fn from_captured<T: NativeCapture>(callback: CapturedCall<T>, captures: T) -> Self {
        Self {
            inner: StoredNativeFunction(
                Inner::Closure(Rc::new(Closure::<T> {
                    callback,
                    captures: CaptureStore::new(&captures),
                })),
                None,
            ),
            _capture_roots: Some(Rc::new(captures)),
        }
    }

    /// Creates a `NativeFunction` from a function pointer.
    #[inline]
    pub fn from_fn_ptr(function: NativeFunctionPointer) -> Self {
        Self {
            inner: StoredNativeFunction(Inner::PointerFn(function), None),
            _capture_roots: None,
        }
    }

    /// Creates a native function whose owned task settles a JavaScript promise.
    ///
    /// ```
    /// use surgeist_js::{Context, JsArgs, JsResult, JsValue, NativeFunction};
    /// use surgeist_js::job::ContextTask;
    /// let callback = NativeFunction::from_async_fn(|_, args, mut access| {
    ///     let value = args.get_or_undefined(0).to_u32(access.reborrow())?;
    ///     Ok(ContextTask::from_future(async move { Ok(JsValue::new(value * 2)) }))
    /// });
    /// let mut context = Context::default();
    /// callback.call(&JsValue::undefined(), &[3.into()], context.access()).unwrap();
    /// context.run_jobs().unwrap();
    /// ```
    pub fn from_async_fn(callback: NativeAsyncFunctionPointer) -> Self {
        Self {
            inner: StoredNativeFunction(Inner::AsyncPointer(callback), None),
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
            inner: StoredNativeFunction(
                Inner::Closure(Rc::new(AsyncClosure::<T> {
                    callback,
                    captures: CaptureStore::new(&captures),
                })),
                None,
            ),
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
        callback: for<'call> fn(
            &JsValue,
            &[JsValue],
            &T,
            ContextAccess<'call>,
        ) -> JsResult<JsValue>,
        captures: T,
    ) -> Self {
        Self::from_captured(CapturedCall::Host(callback), captures)
    }

    /// Calls this `NativeFunction`, forwarding the arguments to the corresponding function.
    #[inline]
    pub fn call(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
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
    pub(crate) fn resolution_target(&self) -> Option<JsObject> {
        let slot = self
            .1
            .as_ref()?
            .upgrade()
            .expect("resolving metadata heap available")
            .expect("live resolving metadata");
        let target = slot.borrow().clone();
        target.map(JsObject::from_inner)
    }
    fn call(&self, this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        match self.0 {
            Inner::Builtin(function) => function(this, args, context),
            Inner::PointerFn(f) => f(this, args, context.access()),
            Inner::AsyncPointer(callback) => {
                let this = this.clone();
                let args = args.to_vec();
                JsPromise::from_task(
                    move |context| callback(this, args, context),
                    context.access(),
                )
                .map(Into::into)
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
    let args = context.vm.stack.pop_native_arguments(argument_count);
    let _func = context.vm.stack.pop();
    let this = context.vm.stack.pop();

    // We technically don't need this since native functions don't push any new frames to the
    // vm, but we'll eventually have to combine the native stack with the vm stack.
    if context.take_host_entry_permit(obj)?.is_none() {
        context.check_runtime_limits()?;
    }
    let this_function_object = obj.clone();

    let (function, name, constructor, mut realm) = {
        let record = obj
            .downcast_ref::<NativeFunctionObject>()
            .expect("the object should be a native function object");
        (
            record.f.clone(),
            record.name.clone(),
            record.constructor,
            context.realm().resolve_call_realm(record.realm.as_ref()),
        )
    };

    let pc = context.vm.frame().pc;
    let native_source_info = context.native_source_info();
    context
        .vm
        .shadow_stack
        .push_native(pc, name, native_source_info);

    context.swap_realm(&mut realm);
    let previous_function = context
        .vm
        .native_active_function
        .replace(this_function_object);

    context.note_execution_effect();
    let result = if constructor.is_some() {
        function.call(&JsValue::undefined(), args.as_slice(), context)
    } else {
        function.call(&this, args.as_slice(), context)
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
    if context.take_host_entry_permit(obj)?.is_none() {
        context.check_runtime_limits()?;
    }
    let this_function_object = obj.clone();

    let (function, name, constructor, mut realm) = {
        let record = obj
            .downcast_ref::<NativeFunctionObject>()
            .expect("the object should be a native function object");
        (
            record.f.clone(),
            record.name.clone(),
            record.constructor,
            context.realm().resolve_call_realm(record.realm.as_ref()),
        )
    };

    let pc = context.vm.frame().pc;
    let native_source_info = context.native_source_info();
    context
        .vm
        .shadow_stack
        .push_native(pc, name, native_source_info);

    context.swap_realm(&mut realm);
    let previous_function = context
        .vm
        .native_active_function
        .replace(this_function_object);

    let new_target = context.vm.stack.pop();
    let args = context.vm.stack.pop_native_arguments(argument_count);
    let _func = context.vm.stack.pop();
    let _this = context.vm.stack.pop();

    context.note_execution_effect();
    let result = function.call(&new_target, args.as_slice(), context);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Source, js_string, property::Attribute};

    fn record_receiver(
        this: &JsValue,
        args: &[JsValue],
        mut context: crate::ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        context.register_global_property(
            js_string!("recordedBuiltinReceiver"),
            this.clone(),
            Attribute::all(),
        )?;
        Ok(args.first().unwrap_or(this).clone())
    }

    #[test]
    fn stored_builtin_dispatch_retains_receiver_arguments_and_context_effects() {
        let builtin = NativeFunction::from_builtin(|this, args, context| {
            record_receiver(this, args, context.access())
        });
        let stored = builtin.store();
        drop(builtin);
        let restored = NativeFunction::restore(&stored).unwrap();
        let mut context = Context::default();
        let function = restored.to_js_function(context.realm()).unwrap();

        assert_eq!(
            function
                .call(&JsValue::new(17), &[JsValue::new(31)], context.access())
                .unwrap(),
            JsValue::new(31)
        );
        assert_eq!(
            context
                .eval(Source::from_bytes("recordedBuiltinReceiver"))
                .unwrap(),
            JsValue::new(17)
        );
    }

    #[test]
    fn host_pointer_dispatch_preserves_receiver_and_context_effects() {
        let host = NativeFunction::from_fn_ptr(record_receiver);
        let stored = host.store();
        drop(host);
        let restored = NativeFunction::restore(&stored).unwrap();
        let mut context = Context::default();

        assert_eq!(
            restored
                .call(&JsValue::new(19), &[], context.access())
                .unwrap(),
            JsValue::new(19)
        );
        assert_eq!(
            context
                .eval(Source::from_bytes("recordedBuiltinReceiver"))
                .unwrap(),
            JsValue::new(19)
        );
    }

    #[test]
    fn captured_roles_share_live_roots_stored_edges_and_invocation_custody() {
        fn captured_receiver(
            this: &JsValue,
            _: &[JsValue],
            capture: &JsObject,
            mut context: crate::ContextAccess<'_>,
        ) -> JsResult<JsValue> {
            // The invocation's restored roots survive collection and reentry.
            context.runtime().collect().unwrap();
            capture.set(
                js_string!("receiver"),
                this.clone(),
                true,
                context.reborrow(),
            )?;
            Ok(capture.clone().into())
        }

        for builtin in [false, true] {
            let mut context = Context::default();
            let capture = JsObject::with_null_proto(context.runtime());
            let weak_capture = capture.edge();
            let callback = if builtin {
                NativeFunction::from_builtin_with_captures(
                    |this, args, capture, context| {
                        captured_receiver(this, args, capture, context.access())
                    },
                    capture,
                )
            } else {
                NativeFunction::from_fn_with_captures(captured_receiver, capture)
            };
            let stored = callback.store();
            let restored = NativeFunction::restore(&stored).unwrap();
            drop(callback);

            context.runtime().collect().unwrap();
            assert!(weak_capture.is_live().unwrap());
            let function = restored.to_js_function(context.realm()).unwrap();
            context.runtime().collect().unwrap();
            assert!(weak_capture.is_live().unwrap());
            {
                let value = function
                    .call(&JsValue::new(27), &[], context.access())
                    .unwrap();
                let object = value.as_object().unwrap();
                assert_eq!(object, weak_capture.upgrade().unwrap().unwrap());
                assert_eq!(
                    object
                        .get(js_string!("receiver"), context.access())
                        .unwrap(),
                    JsValue::new(27)
                );
            }
            drop(function);
            drop(stored);
            context.runtime().collect().unwrap();
            assert!(!weak_capture.is_live().unwrap());
        }
    }

    #[test]
    fn shared_async_capture_storage_keeps_pending_job_roots_after_function_drop() {
        use crate::builtins::promise::PromiseState;

        let mut context = Context::default();
        let capture = JsObject::with_null_proto(context.runtime());
        let weak_capture = capture.edge();
        let callback = NativeFunction::from_async_fn_with_captures(
            |_, _, capture, _| Ok(ContextTask::from_future(async move { Ok(capture.into()) })),
            capture,
        );
        let stored = callback.store();
        let restored = NativeFunction::restore(&stored).unwrap();
        drop(callback);
        let promise = restored
            .call(&JsValue::undefined(), &[], context.access())
            .unwrap()
            .as_object()
            .unwrap();
        let promise = JsPromise::from_object(promise).unwrap();
        drop(restored);
        drop(stored);

        context.runtime().collect().unwrap();
        assert!(weak_capture.is_live().unwrap());
        context.run_jobs().unwrap();
        {
            let PromiseState::Fulfilled(value) = promise.state().unwrap() else {
                panic!("the pending host callback must fulfill with its capture")
            };
            assert_eq!(value.as_object(), weak_capture.upgrade().unwrap());
        }
        drop(promise);
        context.runtime().collect().unwrap();
        assert!(!weak_capture.is_live().unwrap());
    }

    // Resolver metadata and closure captures own independent restoration roots.
    use std::cell::{Cell, RefCell};

    #[derive(crate::JsData, HeapTrace)]
    struct ResolutionMetadataLeaf(#[heap_trace(skip)] Rc<Cell<usize>>);

    impl Drop for ResolutionMetadataLeaf {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    struct ResolutionMetadataCapture {
        object: JsObject,
        calls: Rc<Cell<usize>>,
    }

    #[derive(HeapTrace)]
    struct StoredResolutionMetadataCapture {
        object: CapturedObject,
        #[heap_trace(skip)]
        calls: Rc<Cell<usize>>,
    }

    impl NativeCapture for ResolutionMetadataCapture {
        type Stored = StoredResolutionMetadataCapture;

        fn store(&self) -> Self::Stored {
            StoredResolutionMetadataCapture {
                object: self.object.store(),
                calls: self.calls.clone(),
            }
        }

        fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
            Ok(Self {
                object: JsObject::restore(&stored.object)?,
                calls: stored.calls.clone(),
            })
        }
    }

    fn resolution_metadata_leaf(runtime: &crate::Runtime, drops: &Rc<Cell<usize>>) -> JsObject {
        JsObject::new_unique(runtime, None, ResolutionMetadataLeaf(drops.clone()))
            .unwrap()
            .upcast()
    }

    fn resolution_metadata_callback(object: JsObject, calls: &Rc<Cell<usize>>) -> NativeFunction {
        NativeFunction::from_fn_with_captures(
            |_, _, capture, _| {
                capture.calls.set(capture.calls.get() + 1);
                Ok(capture.object.clone().into())
            },
            ResolutionMetadataCapture {
                object,
                calls: calls.clone(),
            },
        )
    }

    #[test]
    fn restored_function_retains_closure_and_resolution_metadata_graph_until_last_owner_drop() {
        let mut context = Context::default();
        context
            .runtime()
            .heap
            .set_allocation_collection(surgeist_js_gc::heap::AllocationCollection::Manual)
            .unwrap();
        let capture_drops = Rc::new(Cell::new(0));
        let target_drops = Rc::new(Cell::new(0));
        let child_drops = Rc::new(Cell::new(0));
        let calls = Rc::new(Cell::new(0));
        let capture = resolution_metadata_leaf(context.runtime(), &capture_drops);
        let target = resolution_metadata_leaf(context.runtime(), &target_drops);
        let child = resolution_metadata_leaf(context.runtime(), &child_drops);
        let weak_capture = capture.edge();
        let weak_target = target.edge();
        let weak_child = child.edge();
        let capture_id = capture.identity();
        let target_id = target.identity();
        let child_id = child.identity();
        target
            .set(js_string!("child"), child.clone(), true, context.access())
            .unwrap();
        let metadata = context
            .runtime()
            .heap
            .allocate(RefCell::new(Some(target.edge())))
            .unwrap();
        let weak_metadata = metadata.record_ref();
        let original =
            resolution_metadata_callback(capture, &calls).with_resolution_target(&metadata);
        let stored = original.store();
        let restored = NativeFunction::restore(&stored).unwrap();
        let last_owner = restored.clone();
        drop(original);
        drop(metadata);
        drop(target);
        drop(child);
        drop(restored);

        context.runtime().collect().unwrap();
        assert!(weak_metadata.is_live().unwrap());
        assert!(weak_capture.is_live().unwrap());
        assert!(weak_target.is_live().unwrap());
        assert!(weak_child.is_live().unwrap());
        for drops in [&capture_drops, &target_drops, &child_drops] {
            assert_eq!(drops.get(), 0);
        }
        assert_eq!(calls.get(), 0, "restoration must not enter the callback");
        {
            let target = last_owner.inner.resolution_target().unwrap();
            assert_eq!(target.identity(), target_id);
            let child = target.get(js_string!("child"), context.access()).unwrap();
            assert_eq!(child.as_object().unwrap().identity(), child_id);
            let returned = last_owner
                .call(&JsValue::undefined(), &[], context.access())
                .unwrap();
            assert_eq!(returned.as_object().unwrap().identity(), capture_id);
        }
        assert_eq!(calls.get(), 1);
        drop(last_owner);

        context.runtime().collect().unwrap();
        assert!(!weak_metadata.is_live().unwrap());
        assert!(!weak_capture.is_live().unwrap());
        assert!(!weak_target.is_live().unwrap());
        assert!(!weak_child.is_live().unwrap());
        for drops in [&capture_drops, &target_drops, &child_drops] {
            assert_eq!(drops.get(), 1);
        }
        drop(stored);
        context.runtime().collect().unwrap();
        for drops in [&capture_drops, &target_drops, &child_drops] {
            assert_eq!(drops.get(), 1);
        }
    }

    #[test]
    fn resolution_metadata_heap_refusal_releases_previously_restored_closure_roots() {
        let context = Context::default();
        context
            .runtime()
            .heap
            .set_allocation_collection(surgeist_js_gc::heap::AllocationCollection::Manual)
            .unwrap();
        let metadata_heap = surgeist_js_gc::heap::RuntimeHeap::default();
        let metadata = metadata_heap
            .allocate(RefCell::new(None::<crate::object::ObjectEdge>))
            .unwrap();
        let weak_metadata = metadata.record_ref();
        let drops = Rc::new(Cell::new(0));
        let calls = Rc::new(Cell::new(0));
        let capture = resolution_metadata_leaf(context.runtime(), &drops);
        let weak_capture = capture.edge();
        let original =
            resolution_metadata_callback(capture, &calls).with_resolution_target(&metadata);
        let stored = original.store();
        let metadata_id = stored.1.as_ref().unwrap().identity();
        drop(original);
        drop(metadata);
        assert!(weak_capture.is_live().unwrap());

        metadata_heap
            .read(|_| {
                assert_eq!(
                    NativeFunction::restore(&stored).unwrap_err(),
                    RootingError::Access(AccessError::HeapBorrowed)
                );
            })
            .unwrap();
        assert_eq!(stored.1.as_ref().unwrap().identity(), metadata_id);
        assert_eq!(calls.get(), 0);
        assert_eq!(drops.get(), 0);
        context.runtime().collect().unwrap();
        assert_eq!(
            drops.get(),
            1,
            "failed metadata acquisition releases the closure prefix"
        );
        assert!(!weak_capture.is_live().unwrap());
        assert_eq!(metadata_heap.collect().unwrap(), 1);
        assert!(!weak_metadata.is_live().unwrap());
        drop(stored);
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn expired_resolution_metadata_releases_previously_restored_closure_roots() {
        let context = Context::default();
        context
            .runtime()
            .heap
            .set_allocation_collection(surgeist_js_gc::heap::AllocationCollection::Manual)
            .unwrap();
        let metadata_heap = surgeist_js_gc::heap::RuntimeHeap::default();
        let metadata = metadata_heap
            .allocate(RefCell::new(None::<crate::object::ObjectEdge>))
            .unwrap();
        let weak_metadata = metadata.record_ref();
        let drops = Rc::new(Cell::new(0));
        let calls = Rc::new(Cell::new(0));
        let capture = resolution_metadata_leaf(context.runtime(), &drops);
        let weak_capture = capture.edge();
        let original =
            resolution_metadata_callback(capture, &calls).with_resolution_target(&metadata);
        let stored = original.store();
        let metadata_id = stored.1.as_ref().unwrap().identity();
        drop(original);
        drop(metadata);
        assert_eq!(metadata_heap.collect().unwrap(), 1);
        assert!(!weak_metadata.is_live().unwrap());
        assert!(weak_capture.is_live().unwrap());

        assert_eq!(
            NativeFunction::restore(&stored).unwrap_err(),
            RootingError::Expired
        );
        assert_eq!(stored.1.as_ref().unwrap().identity(), metadata_id);
        assert_eq!(calls.get(), 0);
        assert_eq!(drops.get(), 0);
        context.runtime().collect().unwrap();
        assert_eq!(
            drops.get(),
            1,
            "expired metadata releases the closure prefix"
        );
        assert!(!weak_capture.is_live().unwrap());
        drop(stored);
        assert_eq!(calls.get(), 0);
    }
}
