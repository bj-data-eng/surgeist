//! Boa's implementation of ECMAScript's global `Promise` object.

pub(crate) mod native_settlement;
mod reaction;
use reaction::PublicationHold;
pub(crate) use reaction::{
    CommittedReactions, ReactionCustody, ReactionJobLifecycle, ReactionLifecycle,
    ReactionRegistration, RegistrationOutcome,
};
#[cfg(test)]
mod pending_bridge_cleanup_tests;
#[cfg(test)]
mod remaining_owner_tests;
#[cfg(test)]
mod tests;

use super::{
    BuiltInBuilder, BuiltInConstructor, IntrinsicObject,
    iterable::{IteratorHint, IteratorRecord},
};
#[cfg(feature = "experimental")]
use crate::object::internal_methods::InternalMethodPropertyContext;
#[cfg(feature = "experimental")]
use crate::property::PropertyKey;
use crate::{
    Context, JsArgs, JsError, JsExpect, JsResult, JsString,
    builtins::{Array, BuiltInObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    job::{JobCallback, PromiseJob, StoredJobCallback},
    js_string,
    native_function::{CapturedObject, NativeCapture, NativeFunction},
    object::{
        CONSTRUCTOR, FunctionObjectBuilder, JsFunction, JsObject, ObjectEdge,
        internal_methods::get_prototype_from_constructor,
    },
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
    value::{JsValue, StoredValue},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use surgeist_js_gc::heap::{RecordRef, Rooted, Trace as HeapTrace};
use surgeist_js_macros::JsData;

// ==================== Public API ====================

/// The current state of a [`Promise`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromiseState {
    /// The promise hasn't been resolved.
    Pending,
    /// The promise was fulfilled with a success value.
    Fulfilled(JsValue),
    /// The promise was rejected with a failure reason.
    Rejected(JsValue),
}

impl PromiseState {
    /// Gets the inner `JsValue` of a fulfilled promise state, or returns `None` if
    /// the state is not `Fulfilled`.
    #[must_use]
    pub const fn as_fulfilled(&self) -> Option<&JsValue> {
        match self {
            Self::Fulfilled(v) => Some(v),
            _ => None,
        }
    }

    /// Gets the inner `JsValue` of a rejected promise state, or returns `None` if
    /// the state is not `Rejected`.
    #[must_use]
    pub const fn as_rejected(&self) -> Option<&JsValue> {
        match self {
            Self::Rejected(v) => Some(v),
            _ => None,
        }
    }
}

/// The internal representation of a `Promise` object.
#[derive(Debug, HeapTrace, JsData)]
pub struct Promise {
    state: StoredPromiseState,
    handled: bool,
    native_settlement: Option<native_settlement::AssociationEdge>,
}

#[derive(Debug, HeapTrace)]
enum StoredPromiseState {
    Pending {
        fulfill_reactions: Vec<StoredReactionRecord>,
        reject_reactions: Vec<StoredReactionRecord>,
    },
    Fulfilled(StoredValue),
    Rejected(StoredValue),
}

/// The operation type of the [`HostPromiseRejectionTracker`][fn] abstract operation.
///
/// # Note
///
/// Per the spec:
///
/// > If operation is "handle", an implementation should not hold a reference to promise in a way
/// > that would interfere with garbage collection. An implementation may hold a reference to promise
/// > if operation is "reject", since it is expected that rejections will be rare and not on hot code paths.
///
/// [fn]: https://tc39.es/ecma262/#sec-host-promise-rejection-tracker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    /// A promise was rejected without any handlers.
    Reject,
    /// A handler was added to a rejected promise for the first time.
    Handle,
}

/// Functions used to resolve a pending promise.
///
/// This is equivalent to the parameters `resolveFunc` and `rejectFunc` of the executor passed to
/// the [`Promise()`] constructor.
///
/// Both functions are always associated with the promise from which they were created. This
/// means that by simply calling `resolve.call(this, &[values], context.access())` or
/// `reject.call(this, &[error], context.access())`, the state of the original promise will be updated with
/// the resolution value.
///
/// [`Promise()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/Promise
#[derive(Debug, Clone)]
pub struct ResolvingFunctions {
    /// The `resolveFunc` parameter of the executor passed to `Promise()`.
    pub resolve: JsFunction,
    /// The `rejectFunc` parameter of the executor passed to `Promise()`.
    pub reject: JsFunction,
}

// ==================== Private API ====================

/// `IfAbruptRejectPromise ( value, capability )`
///
/// `IfAbruptRejectPromise` is a shorthand for a sequence of algorithm steps that use a `PromiseCapability` Record.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ifabruptrejectpromise
macro_rules! if_abrupt_reject_promise {
    ($value:expr, $capability:expr, $context: expr) => {
        match $value {
            // 1. If value is an abrupt completion, then
            Err(err) => {
                let err = err.into_opaque($context.access())?;
                // a. Perform ? Call(capability.[[Reject]], undefined, « value.[[Value]] »).
                $capability
                    .reject()
                    .call(&JsValue::undefined(), &[err], $context.access())?;

                // b. Return capability.[[Promise]].
                return Ok($capability.promise().clone().into());
            }
            // 2. Else if value is a Completion Record, set value to value.[[Value]].
            Ok(value) => value,
        }
    };
}

pub(crate) use if_abrupt_reject_promise;

impl ResolvingFunctions {
    pub(crate) fn associate_job(
        &self,
        observer: &crate::job::JobObserver,
        context: &mut Context,
    ) -> JsResult<()> {
        if let Some(promise) = Promise::native_handler_target(&self.resolve) {
            Promise::ensure_native_association(&promise, context)?.bind_job(observer, false)?;
        }
        Ok(())
    }
}

/// The internal `PromiseCapability` data type.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-promisecapability-records
#[derive(Debug, Clone)]
pub(crate) struct PromiseCapability {
    /// The `[[Promise]]` field.
    pub(crate) promise: JsObject,

    /// The resolving functions,
    pub(crate) functions: ResolvingFunctions,
}

/// A capability retained inside a JavaScript record.
#[derive(Debug, Clone, surgeist_js_gc::heap::Trace)]
pub(crate) struct StoredPromiseCapability {
    promise: crate::object::ObjectEdge,
    resolve: crate::object::ObjectEdge,
    reject: crate::object::ObjectEdge,
}

impl StoredPromiseCapability {
    pub(crate) fn to_capability(&self) -> PromiseCapability {
        PromiseCapability {
            promise: JsObject::from_inner(self.promise.clone()),
            functions: ResolvingFunctions {
                resolve: JsFunction::from_record(self.resolve.clone()),
                reject: JsFunction::from_record(self.reject.clone()),
            },
        }
    }
}

impl NativeCapture for PromiseCapability {
    type Stored = StoredPromiseCapability;

    fn store(&self) -> Self::Stored {
        self.stored()
    }

    fn restore(stored: &Self::Stored) -> Self {
        stored.to_capability()
    }
}

/// The internal `PromiseReaction` data type.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-promisereaction-records
struct ReactionRecord {
    lifecycle: ReactionLifecycle,
    native_result_target: Option<JsObject<Promise>>,
    native_target: Option<JsObject<Promise>>,
    /// The `[[Capability]]` field.
    promise_capability: Option<PromiseCapability>,

    /// The `[[Type]]` field.
    reaction_type: ReactionType,

    /// The `[[Handler]]` field.
    handler: Option<JobCallback>,
}

impl ReactionRecord {
    fn to_stored(&self) -> StoredReactionRecord {
        StoredReactionRecord {
            lifecycle: self.lifecycle.clone(),
            native_result_target: self.native_result_target.as_ref().map(JsObject::edge),
            native_target: self.native_target.as_ref().map(JsObject::edge),
            promise_capability: self
                .promise_capability
                .as_ref()
                .map(PromiseCapability::stored),
            reaction_type: self.reaction_type,
            handler: self.handler.as_ref().map(JobCallback::to_stored),
        }
    }
}

#[derive(HeapTrace)]
struct StoredReactionRecord {
    lifecycle: ReactionLifecycle,
    native_result_target: Option<ObjectEdge>,
    native_target: Option<ObjectEdge>,
    promise_capability: Option<StoredPromiseCapability>,
    #[heap_trace(skip)]
    reaction_type: ReactionType,
    handler: Option<StoredJobCallback>,
}

impl StoredReactionRecord {
    fn to_reaction(&self) -> ReactionRecord {
        ReactionRecord {
            lifecycle: self.lifecycle.clone(),
            native_result_target: self
                .native_result_target
                .clone()
                .map(JsObject::from_inner)
                .map(|value| {
                    value
                        .downcast::<Promise>()
                        .expect("reaction target is a promise")
                }),
            native_target: self
                .native_target
                .clone()
                .map(JsObject::from_inner)
                .map(|value| {
                    value
                        .downcast::<Promise>()
                        .expect("reaction target is a promise")
                }),
            promise_capability: self
                .promise_capability
                .as_ref()
                .map(StoredPromiseCapability::to_capability),
            reaction_type: self.reaction_type,
            handler: self.handler.as_ref().map(StoredJobCallback::to_callback),
        }
    }
}

impl std::fmt::Debug for StoredReactionRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredReactionRecord")
            .field("promise_capability", &self.promise_capability)
            .field("reaction_type", &self.reaction_type)
            .finish_non_exhaustive()
    }
}

/// The `[[Type]]` field values of a `PromiseReaction` record.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-promisereaction-records
#[derive(Debug, Clone, Copy)]
enum ReactionType {
    Fulfill,
    Reject,
}

struct CombinatorElementCaptures {
    already_called: Rc<Cell<bool>>,
    index: usize,
    values: Rooted<RefCell<Vec<StoredValue>>>,
    capability: JsFunction,
    remaining_elements: Rc<Cell<i32>>,
}

#[derive(HeapTrace)]
struct StoredCombinatorElementCaptures {
    #[heap_trace(skip)]
    already_called: Rc<Cell<bool>>,
    index: usize,
    values: RecordRef<RefCell<Vec<StoredValue>>>,
    capability: CapturedObject,
    #[heap_trace(skip)]
    remaining_elements: Rc<Cell<i32>>,
}

impl NativeCapture for CombinatorElementCaptures {
    type Stored = StoredCombinatorElementCaptures;

    fn store(&self) -> Self::Stored {
        StoredCombinatorElementCaptures {
            already_called: self.already_called.clone(),
            index: self.index,
            values: self.values.store(),
            capability: self.capability.store(),
            remaining_elements: self.remaining_elements.clone(),
        }
    }

    fn restore(stored: &Self::Stored) -> Self {
        Self {
            already_called: stored.already_called.clone(),
            index: stored.index,
            values: Rooted::restore(&stored.values),
            capability: JsFunction::restore(&stored.capability),
            remaining_elements: stored.remaining_elements.clone(),
        }
    }
}

impl PromiseCapability {
    fn native_promise(&self) -> JsResult<JsObject<Promise>> {
        self.promise.clone().downcast::<Promise>().map_err(|_| {
            crate::job::JobFault::new(
                crate::job::JobFaultOwner::NativeSignal,
                crate::job::JobFaultCause::StateInconsistency,
            )
            .into()
        })
    }
    pub(crate) fn associate_primary_job(
        &self,
        observer: &crate::job::JobObserver,
        context: &mut Context,
    ) -> JsResult<()> {
        context.check_execution_stop()?;
        let promise = self.native_promise()?;
        Promise::ensure_native_association(&promise, context)?.bind_job(observer, true)
    }
    pub(crate) fn associate_required_job(
        &self,
        observer: &crate::job::JobObserver,
        context: &mut Context,
    ) -> JsResult<()> {
        context.check_execution_stop()?;
        let promise = self.native_promise()?;
        Promise::ensure_native_association(&promise, context)?.bind_job(observer, false)
    }

    pub(crate) fn stored(&self) -> StoredPromiseCapability {
        StoredPromiseCapability {
            promise: self.promise.edge(),
            resolve: self.functions.resolve.clone().into_record(),
            reject: self.functions.reject.clone().into_record(),
        }
    }

    /// `NewPromiseCapability ( C )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-newpromisecapability
    pub(crate) fn new(c: &JsObject, context: &mut Context) -> JsResult<Self> {
        #[derive(HeapTrace)]
        struct RejectResolve {
            reject: StoredValue,
            resolve: StoredValue,
        }

        // 1. If IsConstructor(C) is false, throw a TypeError exception.
        if !c.is_constructor() {
            return Err(JsNativeError::typ()
                .with_message("PromiseCapability: expected constructor")
                .into());
        }

        // 2. NOTE: C is assumed to be a constructor function that supports the parameter conventions of the Promise constructor (see 27.2.3.1).
        // 3. Let promiseCapability be the PromiseCapability Record { [[Promise]]: undefined, [[Resolve]]: undefined, [[Reject]]: undefined }.
        let promise_capability = context
            .runtime()
            .heap
            .allocate(RefCell::new(RejectResolve {
                reject: StoredValue::Undefined,
                resolve: StoredValue::Undefined,
            }))
            .expect("a new promise capability has no object edges");

        // 4. Let executorClosure be a new Abstract Closure with parameters (resolve, reject) that captures promiseCapability and performs the following steps when called:
        // 5. Let executor be CreateBuiltinFunction(executorClosure, 2, "", « »).
        let executor = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_builtin_with_captures(
                |_this, args: &[JsValue], captures, _| {
                    let mut promise_capability = captures.borrow_mut();
                    // a. If promiseCapability.[[Resolve]] is not undefined, throw a TypeError exception.
                    if !matches!(promise_capability.resolve, StoredValue::Undefined) {
                        return Err(JsNativeError::typ()
                            .with_message("promiseCapability.[[Resolve]] is not undefined")
                            .into());
                    }

                    // b. If promiseCapability.[[Reject]] is not undefined, throw a TypeError exception.
                    if !matches!(promise_capability.reject, StoredValue::Undefined) {
                        return Err(JsNativeError::typ()
                            .with_message("promiseCapability.[[Reject]] is not undefined")
                            .into());
                    }

                    let resolve = args.get_or_undefined(0);
                    let reject = args.get_or_undefined(1);

                    // c. Set promiseCapability.[[Resolve]] to resolve.
                    promise_capability.resolve = resolve.clone().into_stored();

                    // d. Set promiseCapability.[[Reject]] to reject.
                    promise_capability.reject = reject.clone().into_stored();

                    // e. Return undefined.
                    Ok(JsValue::undefined())
                },
                promise_capability.clone(),
            ),
        )
        .name("")
        .length(2)
        .build()?
        .into();

        // 6. Let promise be ? Construct(C, « executor »).
        let promise = c.construct(&[executor], None, context.access())?;

        let promise_capability = promise_capability.borrow();

        let resolve = promise_capability.resolve.to_value();
        let reject = promise_capability.reject.to_value();

        // 7. If IsCallable(promiseCapability.[[Resolve]]) is false, throw a TypeError exception.
        let resolve = resolve
            .as_object()
            .and_then(JsFunction::from_object)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("promiseCapability.[[Resolve]] is not callable")
            })?;

        // 8. If IsCallable(promiseCapability.[[Reject]]) is false, throw a TypeError exception.
        let reject = reject
            .as_object()
            .and_then(JsFunction::from_object)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("promiseCapability.[[Reject]] is not callable")
            })?;

        // 9. Set promiseCapability.[[Promise]] to promise.
        // 10. Return promiseCapability.
        Ok(Self {
            promise,
            functions: ResolvingFunctions { resolve, reject },
        })
    }

    /// Returns the promise object.
    pub(crate) const fn promise(&self) -> &JsObject {
        &self.promise
    }

    /// Returns the resolve function.
    pub(crate) const fn resolve(&self) -> &JsFunction {
        &self.functions.resolve
    }

    /// Returns the reject function.
    pub(crate) const fn reject(&self) -> &JsFunction {
        &self.functions.reject
    }
}

impl IntrinsicObject for Promise {
    fn init(realm: &Realm) {
        let get_species = BuiltInBuilder::callable(realm, Self::get_species)
            .name(js_string!("get [Symbol.species]"))
            .build();

        let builder = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(Self::all, js_string!("all"), 1)
            .static_method(Self::all_settled, js_string!("allSettled"), 1)
            .static_method(Self::any, js_string!("any"), 1)
            .static_method(Self::race, js_string!("race"), 1)
            .static_method(Self::reject, js_string!("reject"), 1)
            .static_method(Self::resolve, js_string!("resolve"), 1)
            .static_method(Self::r#try, js_string!("try"), 1)
            .static_method(Self::with_resolvers, js_string!("withResolvers"), 0)
            .static_accessor(
                JsSymbol::species(),
                Some(get_species),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::then, js_string!("then"), 2)
            .method(Self::catch, js_string!("catch"), 1)
            .method(Self::finally, js_string!("finally"), 1)
            // <https://tc39.es/ecma262/#sec-promise.prototype-@@tostringtag>
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            );

        #[cfg(feature = "experimental")]
        let builder = builder
            .static_method(Self::all_keyed, js_string!("allKeyed"), 1)
            .static_method(Self::all_settled_keyed, js_string!("allSettledKeyed"), 1);

        builder.build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Promise {
    const NAME: JsString = StaticJsStrings::PROMISE;
}

impl BuiltInConstructor for Promise {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 4;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 12;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::promise;

    /// `Promise ( executor )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise-executor
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Promise NewTarget cannot be undefined")
                .into());
        }

        // 2. If IsCallable(executor) is false, throw a TypeError exception.
        let executor = args
            .get_or_undefined(0)
            .as_callable()
            .ok_or_else(|| JsNativeError::typ().with_message("Promise executor is not callable"))?;

        // 3. Let promise be ? OrdinaryCreateFromConstructor(NewTarget, "%Promise.prototype%", « [[PromiseState]], [[PromiseResult]], [[PromiseFulfillReactions]], [[PromiseRejectReactions]], [[PromiseIsHandled]] »).
        let promise =
            get_prototype_from_constructor(new_target, StandardConstructors::promise, context)?;

        let promise = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            promise,
            // 4. Set promise.[[PromiseState]] to pending.
            // 5. Set promise.[[PromiseFulfillReactions]] to a new empty List.
            // 6. Set promise.[[PromiseRejectReactions]] to a new empty List.
            // 7. Set promise.[[PromiseIsHandled]] to false.
            Self::new(),
        );

        // 8. Let resolvingFunctions be CreateResolvingFunctions(promise).
        let resolving_functions = Self::create_resolving_functions(&promise, context);

        // 9. Let completion Completion(Call(executor, undefined, « resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]] »)be ).
        let completion = executor.call(
            &JsValue::undefined(),
            &[
                resolving_functions.resolve.clone().into(),
                resolving_functions.reject.clone().into(),
            ],
            context.access(),
        );

        // 10. If completion is an abrupt completion, then
        if let Err(e) = completion {
            let e = e.into_opaque(context.access())?;
            // a. Perform ? Call(resolvingFunctions.[[Reject]], undefined, « completion.[[Value]] »).
            resolving_functions
                .reject
                .call(&JsValue::undefined(), &[e], context.access())?;
        }

        // 11. Return promise.
        Ok(promise.into())
    }
}

impl Promise {
    pub(crate) fn native_association(
        promise: &JsObject<Self>,
    ) -> JsResult<Option<Rooted<native_settlement::Association>>> {
        let edge = promise
            .try_borrow()
            .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?
            .data()
            .native_settlement
            .clone();
        edge.as_ref()
            .map(native_settlement::AssociationEdge::root)
            .transpose()
    }
    pub(crate) fn ensure_native_association(
        promise: &JsObject<Self>,
        context: &Context,
    ) -> JsResult<Rooted<native_settlement::Association>> {
        context.check_execution_stop()?;
        promise.check_runtime(context.runtime())?;
        if let Some(association) = Self::native_association(promise)? {
            association.check()?;
            return Ok(association);
        }
        let pending = matches!(
            promise
                .try_borrow()
                .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?
                .data()
                .state,
            StoredPromiseState::Pending { .. }
        );
        let association = native_settlement::Association::new(context)?;
        promise
            .try_borrow_mut()
            .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?
            .data_mut()
            .native_settlement = Some(native_settlement::Association::edge(&association));
        if !pending {
            association.settle();
        }
        Ok(association)
    }
    fn prepare_native_handoff(
        promise: &JsObject<Self>,
        observer: &crate::job::JobObserver,
    ) -> JsResult<Option<native_settlement::PreparedHandoff>> {
        Self::native_association(promise)?
            .map(|owner| native_settlement::Association::prepare_handoff(&owner, observer))
            .transpose()
    }
    fn native_handler_target(function: &JsFunction) -> Option<JsObject<Self>> {
        function
            .downcast_ref::<crate::native_function::NativeFunctionObject>()?
            .f
            .resolution_target()?
            .downcast::<Self>()
            .ok()
    }

    /// Creates a new, pending `Promise`.
    pub(crate) fn new() -> Self {
        Self {
            state: StoredPromiseState::Pending {
                fulfill_reactions: Vec::new(),
                reject_reactions: Vec::new(),
            },
            handled: false,
            native_settlement: None,
        }
    }

    /// Gets the current state of the promise.
    pub(crate) fn state(&self) -> PromiseState {
        match &self.state {
            StoredPromiseState::Pending { .. } => PromiseState::Pending,
            StoredPromiseState::Fulfilled(value) => PromiseState::Fulfilled(value.to_value()),
            StoredPromiseState::Rejected(reason) => PromiseState::Rejected(reason.to_value()),
        }
    }

    /// [`Promise.try ( callbackfn, ...args )`][spec]
    ///
    /// Calls the given function and returns a new promise that is resolved if the function
    /// completes normally and rejected if it throws.
    ///
    /// [spec]: https://tc39.es/proposal-promise-try/#sec-promise.try
    pub(crate) fn r#try(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let callback = args.get_or_undefined(0);
        let callback_args = args.get(1..).unwrap_or(&[]);

        // 1. Let C be the this value.
        // 2. If C is not an Object, throw a TypeError exception.
        let c = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Promise.try() called on a non-object")
        })?;

        // 3. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(&c, context)?;

        // 4. Let status be Completion(Call(callbackfn, undefined, args)).
        let status = callback.call(&JsValue::undefined(), callback_args, context);

        match status {
            // 5. If status is an abrupt completion, then
            Err(err) => {
                let value = err.into_opaque(context.access())?;

                // a. Perform ? Call(promiseCapability.[[Reject]], undefined, « status.[[Value]] »).
                promise_capability.functions.reject.call(
                    &JsValue::undefined(),
                    &[value],
                    context.access(),
                )?;
            }
            // 6. Else,
            Ok(value) => {
                // a. Perform ? Call(promiseCapability.[[Resolve]], undefined, « status.[[Value]] »).
                promise_capability.functions.resolve.call(
                    &JsValue::undefined(),
                    &[value],
                    context.access(),
                )?;
            }
        }

        // 7. Return promiseCapability.[[Promise]].
        Ok(promise_capability.promise.clone().into())
    }

    /// [`Promise.withResolvers ( )`][spec]
    ///
    /// Creates a new promise that is pending, and returns that promise plus the resolve and reject
    /// functions associated with it.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.withResolvers
    pub(crate) fn with_resolvers(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let C be the this value.

        use super::OrdinaryObject;
        let c = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Promise.withResolvers() called on a non-object")
        })?;

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let PromiseCapability {
            promise,
            functions: ResolvingFunctions { resolve, reject },
        } = PromiseCapability::new(&c, context)?;

        // 3. Let obj be OrdinaryObjectCreate(%Object.prototype%).
        // 4. Perform ! CreateDataPropertyOrThrow(obj, "promise", promiseCapability.[[Promise]]).
        // 5. Perform ! CreateDataPropertyOrThrow(obj, "resolve", promiseCapability.[[Resolve]]).
        // 6. Perform ! CreateDataPropertyOrThrow(obj, "reject", promiseCapability.[[Reject]]).
        let obj = context.intrinsics().templates().with_resolvers().create(
            OrdinaryObject,
            vec![promise.into(), resolve.into(), reject.into()],
        );

        // 7. Return obj.
        Ok(obj.into())
    }

    /// `Promise.all ( iterable )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.all
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/all
    pub(crate) fn all(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let C be the this value.
        let c = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Promise.all() called on a non-object")
        })?;

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(&c, context)?;

        // 3. Let promiseResolve be Completion(GetPromiseResolve(C)).
        let promise_resolve = Self::get_promise_resolve(&c, context);

        // 4. IfAbruptRejectPromise(promiseResolve, promiseCapability).
        let promise_resolve =
            if_abrupt_reject_promise!(promise_resolve, promise_capability, context);

        // 5. Let iteratorRecord be Completion(GetIterator(iterable, sync)).
        let iterator_record = args
            .get_or_undefined(0)
            .get_iterator(IteratorHint::Sync, context.access());

        // 6. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
        let mut iterator_record =
            if_abrupt_reject_promise!(iterator_record, promise_capability, context);

        // 7. Let result be Completion(PerformPromiseAll(iteratorRecord, C, promiseCapability, promiseResolve)).
        let mut result = Self::perform_promise_all(
            &mut iterator_record,
            &c,
            &promise_capability,
            &promise_resolve,
            context,
        )
        .map(JsValue::from);

        // 8. If result is an abrupt completion, then
        if result.is_err() {
            // a. If iteratorRecord.[[Done]] is false, set result to Completion(IteratorClose(iteratorRecord, result)).
            if !iterator_record.done() {
                result = iterator_record.close(result, context);
            }

            // b. IfAbruptRejectPromise(result, promiseCapability).
            let result = if_abrupt_reject_promise!(result, promise_capability, context);

            return Ok(result);
        }

        // 9. Return ? result.
        result
    }

    /// `PerformPromiseAll ( iteratorRecord, constructor, resultCapability, promiseResolve )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performpromiseall
    pub(crate) fn perform_promise_all(
        iterator_record: &mut IteratorRecord,
        constructor: &JsObject,
        result_capability: &PromiseCapability,
        promise_resolve: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 1. Let values be a new empty List.
        let values = context
            .runtime()
            .heap
            .allocate(RefCell::new(Vec::<StoredValue>::new()))
            .expect("an empty result list has no object edges");

        // 2. Let remainingElementsCount be the Record { [[Value]]: 1 }.
        let remaining_elements = Rc::new(Cell::new(1));

        // 3. Let index be 0.
        let mut index = 0;

        // 4. Repeat,
        while let Some(next) = iterator_record.step_value(context)? {
            // c. Append undefined to values.
            values.borrow_mut().push(StoredValue::Undefined);

            // d. Let nextPromise be ? Call(promiseResolve, constructor, « next »).
            let next_promise =
                promise_resolve.call(&constructor.clone().into(), &[next], context.access())?;

            // e. Let steps be the algorithm steps defined in Promise.all Resolve Element Functions.
            // f. Let length be the number of non-optional parameters of the function definition in Promise.all Resolve Element Functions.
            // g. Let onFulfilled be CreateBuiltinFunction(steps, length, "", « [[AlreadyCalled]], [[Index]], [[Values]], [[Capability]], [[RemainingElements]] »).
            // h. Set onFulfilled.[[AlreadyCalled]] to false.
            // i. Set onFulfilled.[[Index]] to index.
            // j. Set onFulfilled.[[Values]] to values.
            // k. Set onFulfilled.[[Capability]] to resultCapability.
            // l. Set onFulfilled.[[RemainingElements]] to remainingElementsCount.
            let on_fulfilled = FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_builtin_with_captures(
                    |_, args, captures, context| {
                        // https://tc39.es/ecma262/#sec-promise.all-resolve-element-functions

                        // 1. Let F be the active function object.
                        // 2. If F.[[AlreadyCalled]] is true, return undefined.
                        if captures.already_called.get() {
                            return Ok(JsValue::undefined());
                        }

                        // 3. Set F.[[AlreadyCalled]] to true.
                        captures.already_called.set(true);

                        // 4. Let index be F.[[Index]].
                        // 5. Let values be F.[[Values]].
                        // 6. Let promiseCapability be F.[[Capability]].
                        // 7. Let remainingElementsCount be F.[[RemainingElements]].

                        // 8. Set values[index] to x.
                        captures.values.borrow_mut()[captures.index] =
                            args.get_or_undefined(0).clone().into_stored();

                        // 9. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                        captures
                            .remaining_elements
                            .set(captures.remaining_elements.get() - 1);

                        // 10. If remainingElementsCount.[[Value]] is 0, then
                        if captures.remaining_elements.get() == 0 {
                            // a. Let valuesArray be CreateArrayFromList(values).
                            let values_array = Array::create_array_from_list(
                                captures.values.borrow().iter().map(StoredValue::to_value),
                                context,
                            );

                            // b. Return ? Call(promiseCapability.[[Resolve]], undefined, « valuesArray »).
                            return captures.capability.call(
                                &JsValue::undefined(),
                                &[values_array.into()],
                                context.access(),
                            );
                        }

                        // 11. Return undefined.
                        Ok(JsValue::undefined())
                    },
                    CombinatorElementCaptures {
                        already_called: Rc::new(Cell::new(false)),
                        index,
                        values: values.clone(),
                        capability: result_capability.functions.resolve.clone(),
                        remaining_elements: remaining_elements.clone(),
                    },
                ),
            )
            .name("")
            .length(1)
            .constructor(false)
            .build()?;

            // m. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] + 1.
            remaining_elements.set(remaining_elements.get() + 1);

            // n. Perform ? Invoke(nextPromise, "then", « onFulfilled, resultCapability.[[Reject]] »).
            next_promise.invoke(
                js_string!("then"),
                &[
                    on_fulfilled.into(),
                    result_capability.functions.reject.clone().into(),
                ],
                context,
            )?;

            // o. Set index to index + 1.
            index += 1;
        }

        // b. If next is done, then
        //     i. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
        remaining_elements.set(remaining_elements.get() - 1);

        //     ii. If remainingElementsCount.[[Value]] = 0, then
        if remaining_elements.get() == 0 {
            // 1. Let valuesArray be CreateArrayFromList(values).
            let values_array = Array::create_array_from_list(
                values.borrow().iter().map(StoredValue::to_value),
                context,
            );

            // 2. Perform ? Call(resultCapability.[[Resolve]], undefined, « valuesArray »).
            result_capability.functions.resolve.call(
                &JsValue::undefined(),
                &[values_array.into()],
                context.access(),
            )?;
        }

        //     iii. Return resultCapability.[[Promise]].
        Ok(result_capability.promise.clone())
    }

    /// `Promise.allSettled ( iterable )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.allsettled
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/allSettled
    pub(crate) fn all_settled(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let C be the this value.
        let c = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Promise.allSettled() called on a non-object")
        })?;

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(&c, context)?;

        // 3. Let promiseResolve be Completion(GetPromiseResolve(C)).
        let promise_resolve = Self::get_promise_resolve(&c, context);

        // 4. IfAbruptRejectPromise(promiseResolve, promiseCapability).
        let promise_resolve =
            if_abrupt_reject_promise!(promise_resolve, promise_capability, context);

        // 5. Let iteratorRecord be Completion(GetIterator(iterable, sync)).
        let iterator_record = args
            .get_or_undefined(0)
            .get_iterator(IteratorHint::Sync, context.access());

        // 6. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
        let mut iterator_record =
            if_abrupt_reject_promise!(iterator_record, promise_capability, context);

        // 7. Let result be Completion(PerformPromiseAllSettled(iteratorRecord, C, promiseCapability, promiseResolve)).
        let mut result = Self::perform_promise_all_settled(
            &mut iterator_record,
            &c,
            &promise_capability,
            &promise_resolve,
            context,
        )
        .map(JsValue::from);

        // 8. If result is an abrupt completion, then
        if result.is_err() {
            // a. If iteratorRecord.[[Done]] is false, set result to Completion(IteratorClose(iteratorRecord, result)).
            if !iterator_record.done() {
                result = iterator_record.close(result, context);
            }

            // b. IfAbruptRejectPromise(result, promiseCapability).
            let result = if_abrupt_reject_promise!(result, promise_capability, context);

            return Ok(result);
        }

        // 9. Return ? result.
        result
    }

    /// `PerformPromiseAllSettled ( iteratorRecord, constructor, resultCapability, promiseResolve )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performpromiseallsettled
    pub(crate) fn perform_promise_all_settled(
        iterator_record: &mut IteratorRecord,
        constructor: &JsObject,
        result_capability: &PromiseCapability,
        promise_resolve: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 1. Let values be a new empty List.
        let values = context
            .runtime()
            .heap
            .allocate(RefCell::new(Vec::<StoredValue>::new()))
            .expect("an empty result list has no object edges");

        // 2. Let remainingElementsCount be the Record { [[Value]]: 1 }.
        let remaining_elements = Rc::new(Cell::new(1));

        // 3. Let index be 0.
        let mut index = 0;

        // 4. Repeat,
        while let Some(next) = iterator_record.step_value(context)? {
            // c. Append undefined to values.
            values.borrow_mut().push(StoredValue::Undefined);

            // d. Let nextPromise be ? Call(promiseResolve, constructor, « next »).
            let next_promise =
                promise_resolve.call(&constructor.clone().into(), &[next], context.access())?;

            // h. Let alreadyCalled be the Record { [[Value]]: false }.
            let already_called = Rc::new(Cell::new(false));

            // e. Let stepsFulfilled be the algorithm steps defined in Promise.allSettled Resolve Element Functions.
            // f. Let lengthFulfilled be the number of non-optional parameters of the function definition in Promise.allSettled Resolve Element Functions.
            // g. Let onFulfilled be CreateBuiltinFunction(stepsFulfilled, lengthFulfilled, "", « [[AlreadyCalled]], [[Index]], [[Values]], [[Capability]], [[RemainingElements]] »).
            // i. Set onFulfilled.[[AlreadyCalled]] to alreadyCalled.
            // j. Set onFulfilled.[[Index]] to index.
            // k. Set onFulfilled.[[Values]] to values.
            // l. Set onFulfilled.[[Capability]] to resultCapability.
            // m. Set onFulfilled.[[RemainingElements]] to remainingElementsCount.
            let on_fulfilled = FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_builtin_with_captures(
                    |_, args, captures, context| {
                        // https://tc39.es/ecma262/#sec-promise.allsettled-resolve-element-functions

                        // 1. Let F be the active function object.
                        // 2. Let alreadyCalled be F.[[AlreadyCalled]].

                        // 3. If alreadyCalled.[[Value]] is true, return undefined.
                        if captures.already_called.get() {
                            return Ok(JsValue::undefined());
                        }

                        // 4. Set alreadyCalled.[[Value]] to true.
                        captures.already_called.set(true);

                        // 5. Let index be F.[[Index]].
                        // 6. Let values be F.[[Values]].
                        // 7. Let promiseCapability be F.[[Capability]].
                        // 8. Let remainingElementsCount be F.[[RemainingElements]].

                        // 9. Let obj be OrdinaryObjectCreate(%Object.prototype%).
                        let obj = JsObject::with_object_proto(context.intrinsics());

                        // 10. Perform ! CreateDataPropertyOrThrow(obj, "status", "fulfilled").
                        obj.create_data_property_or_throw(
                            js_string!("status"),
                            js_string!("fulfilled"),
                            context.access(),
                        )
                        .js_expect("cannot fail per spec")?;

                        // 11. Perform ! CreateDataPropertyOrThrow(obj, "value", x).
                        obj.create_data_property_or_throw(
                            js_string!("value"),
                            args.get_or_undefined(0).clone(),
                            context.access(),
                        )
                        .js_expect("cannot fail per spec")?;

                        // 12. Set values[index] to obj.
                        captures.values.borrow_mut()[captures.index] =
                            JsValue::from(obj).into_stored();

                        // 13. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                        captures
                            .remaining_elements
                            .set(captures.remaining_elements.get() - 1);

                        // 14. If remainingElementsCount.[[Value]] is 0, then
                        if captures.remaining_elements.get() == 0 {
                            // a. Let valuesArray be CreateArrayFromList(values).
                            let values_array = Array::create_array_from_list(
                                captures.values.borrow().iter().map(StoredValue::to_value),
                                context,
                            );

                            // b. Return ? Call(promiseCapability.[[Resolve]], undefined, « valuesArray »).
                            return captures.capability.call(
                                &JsValue::undefined(),
                                &[values_array.into()],
                                context.access(),
                            );
                        }

                        // 15. Return undefined.
                        Ok(JsValue::undefined())
                    },
                    CombinatorElementCaptures {
                        already_called: already_called.clone(),
                        index,
                        values: values.clone(),
                        capability: result_capability.functions.resolve.clone(),
                        remaining_elements: remaining_elements.clone(),
                    },
                ),
            )
            .name("")
            .length(1)
            .constructor(false)
            .build()?;

            // n. Let stepsRejected be the algorithm steps defined in Promise.allSettled Reject Element Functions.
            // o. Let lengthRejected be the number of non-optional parameters of the function definition in Promise.allSettled Reject Element Functions.
            // p. Let onRejected be CreateBuiltinFunction(stepsRejected, lengthRejected, "", « [[AlreadyCalled]], [[Index]], [[Values]], [[Capability]], [[RemainingElements]] »).
            // q. Set onRejected.[[AlreadyCalled]] to alreadyCalled.
            // r. Set onRejected.[[Index]] to index.
            // s. Set onRejected.[[Values]] to values.
            // t. Set onRejected.[[Capability]] to resultCapability.
            // u. Set onRejected.[[RemainingElements]] to remainingElementsCount.
            let on_rejected = FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_builtin_with_captures(
                    |_, args, captures, context| {
                        // https://tc39.es/ecma262/#sec-promise.allsettled-reject-element-functions

                        // 1. Let F be the active function object.
                        // 2. Let alreadyCalled be F.[[AlreadyCalled]].

                        // 3. If alreadyCalled.[[Value]] is true, return undefined.
                        if captures.already_called.get() {
                            return Ok(JsValue::undefined());
                        }

                        // 4. Set alreadyCalled.[[Value]] to true.
                        captures.already_called.set(true);

                        // 5. Let index be F.[[Index]].
                        // 6. Let values be F.[[Values]].
                        // 7. Let promiseCapability be F.[[Capability]].
                        // 8. Let remainingElementsCount be F.[[RemainingElements]].

                        // 9. Let obj be OrdinaryObjectCreate(%Object.prototype%).
                        let obj = JsObject::with_object_proto(context.intrinsics());

                        // 10. Perform ! CreateDataPropertyOrThrow(obj, "status", "rejected").
                        obj.create_data_property_or_throw(
                            js_string!("status"),
                            js_string!("rejected"),
                            context.access(),
                        )
                        .js_expect("cannot fail per spec")?;

                        // 11. Perform ! CreateDataPropertyOrThrow(obj, "reason", x).
                        obj.create_data_property_or_throw(
                            js_string!("reason"),
                            args.get_or_undefined(0).clone(),
                            context.access(),
                        )
                        .js_expect("cannot fail per spec")?;

                        // 12. Set values[index] to obj.
                        captures.values.borrow_mut()[captures.index] =
                            JsValue::from(obj).into_stored();

                        // 13. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                        captures
                            .remaining_elements
                            .set(captures.remaining_elements.get() - 1);

                        // 14. If remainingElementsCount.[[Value]] is 0, then
                        if captures.remaining_elements.get() == 0 {
                            // a. Let valuesArray be CreateArrayFromList(values).
                            let values_array = Array::create_array_from_list(
                                captures.values.borrow().iter().map(StoredValue::to_value),
                                context,
                            );

                            // b. Return ? Call(promiseCapability.[[Resolve]], undefined, « valuesArray »).
                            return captures.capability.call(
                                &JsValue::undefined(),
                                &[values_array.into()],
                                context.access(),
                            );
                        }

                        // 15. Return undefined.
                        Ok(JsValue::undefined())
                    },
                    CombinatorElementCaptures {
                        already_called,
                        index,
                        values: values.clone(),
                        capability: result_capability.functions.resolve.clone(),
                        remaining_elements: remaining_elements.clone(),
                    },
                ),
            )
            .name("")
            .length(1)
            .constructor(false)
            .build()?;

            // v. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] + 1.
            remaining_elements.set(remaining_elements.get() + 1);

            // w. Perform ? Invoke(nextPromise, "then", « onFulfilled, onRejected »).
            next_promise.invoke(
                js_string!("then"),
                &[on_fulfilled.into(), on_rejected.into()],
                context,
            )?;

            // x. Set index to index + 1.
            index += 1;
        }

        // b. If next is done, then
        //     i. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
        remaining_elements.set(remaining_elements.get() - 1);

        //     ii. If remainingElementsCount.[[Value]] = 0, then
        if remaining_elements.get() == 0 {
            // 1. Let valuesArray be CreateArrayFromList(values).
            let values_array = Array::create_array_from_list(
                values.borrow().iter().map(StoredValue::to_value),
                context,
            );

            // 2. Perform ? Call(resultCapability.[[Resolve]], undefined, « valuesArray »).
            result_capability.functions.resolve.call(
                &JsValue::undefined(),
                &[values_array.into()],
                context.access(),
            )?;
        }

        //     iii. Return resultCapability.[[Promise]].
        Ok(result_capability.promise.clone())
    }

    /// `Promise.allKeyed ( promises )`
    ///
    /// More information:
    ///  - [TC39 proposal spec][spec]
    ///
    /// [spec]: https://tc39.es/proposal-await-dictionary/#sec-promise.allkeyed
    #[cfg(feature = "experimental")]
    pub(crate) fn all_keyed(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::keyed_common(this, args, context, KeyedVariant::All, "allKeyed")
    }

    /// `Promise.allSettledKeyed ( promises )`
    ///
    /// More information:
    ///  - [TC39 proposal spec][spec]
    ///
    /// [spec]: https://tc39.es/proposal-await-dictionary/#sec-promise.allsettledkeyed
    #[cfg(feature = "experimental")]
    pub(crate) fn all_settled_keyed(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::keyed_common(
            this,
            args,
            context,
            KeyedVariant::AllSettled,
            "allSettledKeyed",
        )
    }

    /// Shared entry-point logic for `Promise.allKeyed` and `Promise.allSettledKeyed`.
    #[cfg(feature = "experimental")]
    fn keyed_common(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
        variant: KeyedVariant,
        name: &str,
    ) -> JsResult<JsValue> {
        // 1. Let C be the this value.
        let c = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message(format!("Promise.{name}() called on a non-object"))
        })?;

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(&c, context)?;

        // 3. Let promiseResolve be Completion(GetPromiseResolve(C)).
        let promise_resolve = Self::get_promise_resolve(&c, context);

        // 4. IfAbruptRejectPromise(promiseResolve, promiseCapability).
        let promise_resolve =
            if_abrupt_reject_promise!(promise_resolve, promise_capability, context);

        // 5. If promises is not an Object, then
        let promises = args.get_or_undefined(0);
        let Some(promises_obj) = promises.as_object() else {
            // a. Let error be a newly created TypeError object.
            let error = JsNativeError::typ()
                .with_message(format!("Promise.{name}() expects an object argument"))
                .into_opaque(context.access())?;
            // b. Perform ? Call(promiseCapability.[[Reject]], undefined, « error »).
            promise_capability.functions.reject.call(
                &JsValue::undefined(),
                &[error.into()],
                context.access(),
            )?;
            // c. Return promiseCapability.[[Promise]].
            return Ok(promise_capability.promise.clone().into());
        };

        // 6. Let result be Completion(PerformPromiseAllKeyed(variant, promises, C, promiseCapability, promiseResolve)).
        let result = Self::perform_promise_all_keyed(
            variant,
            &promises_obj,
            &c,
            &promise_capability,
            &promise_resolve,
            context,
        );

        // 7. IfAbruptRejectPromise(result, promiseCapability).
        let _result = if_abrupt_reject_promise!(result, promise_capability, context);

        // 8. Return promiseCapability.[[Promise]].
        Ok(promise_capability.promise.clone().into())
    }

    /// `PerformPromiseAllKeyed ( variant, promises, constructor, resultCapability, promiseResolve )`
    ///
    /// More information:
    ///  - [TC39 proposal spec][spec]
    ///
    /// [spec]: https://tc39.es/proposal-await-dictionary/#sec-performpromiseallkeyed
    #[cfg(feature = "experimental")]
    fn perform_promise_all_keyed(
        variant: KeyedVariant,
        promises: &JsObject,
        constructor: &JsObject,
        result_capability: &PromiseCapability,
        promise_resolve: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        struct KeyedResolveCaptures {
            element: CombinatorElementCaptures,
            variant: KeyedVariant,
            keys: Rc<RefCell<Vec<PropertyKey>>>,
        }

        #[derive(HeapTrace)]
        struct StoredKeyedResolveCaptures {
            element: StoredCombinatorElementCaptures,
            #[heap_trace(skip)]
            variant: KeyedVariant,
            #[heap_trace(skip)]
            keys: Rc<RefCell<Vec<PropertyKey>>>,
        }

        impl NativeCapture for KeyedResolveCaptures {
            type Stored = StoredKeyedResolveCaptures;

            fn store(&self) -> Self::Stored {
                StoredKeyedResolveCaptures {
                    element: self.element.store(),
                    variant: self.variant,
                    keys: self.keys.clone(),
                }
            }

            fn restore(stored: &Self::Stored) -> Self {
                Self {
                    element: CombinatorElementCaptures::restore(&stored.element),
                    variant: stored.variant,
                    keys: stored.keys.clone(),
                }
            }
        }

        // 1. Let allKeys be ? promises.[[OwnPropertyKeys]]().
        let all_keys = promises.own_property_keys(context.access())?;

        // 2. Let keys be a new empty List.
        let keys = Rc::new(RefCell::new(Vec::new()));

        // 3. Let values be a new empty List.
        let values = context
            .runtime()
            .heap
            .allocate(RefCell::new(Vec::<StoredValue>::new()))
            .expect("an empty result list has no object edges");

        // 4. Let remainingElementsCount be the Record { [[Value]]: 1 }.
        let remaining_elements_count = Rc::new(Cell::new(1));

        // 5. Let index be 0.
        let mut index = 0;

        // 6. For each element key of allKeys, do
        for key in all_keys {
            // a. Let desc be ? promises.[[GetOwnProperty]](key).
            let desc = promises
                .__get_own_property__(&key, &mut InternalMethodPropertyContext::new(context))?;

            // b. If desc is not undefined and desc.[[Enumerable]] is true, then
            if let Some(desc) = desc
                && desc.expect_enumerable()
            {
                // i. Let value be ? Get(promises, key).
                let value = promises.get(key.clone(), context.access())?;

                // ii. Append key to keys.
                keys.borrow_mut().push(key);

                // iii. Append undefined to values.
                values.borrow_mut().push(StoredValue::Undefined);

                // iv. Let nextPromise be ? Call(promiseResolve, constructor, « value »).
                let next_promise = promise_resolve.call(
                    &JsValue::from(constructor.clone()),
                    &[value],
                    context.access(),
                )?;

                // v. Let alreadyCalled be the Record { [[Value]]: false }.
                let already_called = Rc::new(Cell::new(false));

                // vi. Let onFulfilled be a new Abstract Closure...
                let on_fulfilled = FunctionObjectBuilder::new(
                    context.realm(),
                    NativeFunction::from_builtin_with_captures(
                        |_, args, captures, context| {
                            let element = &captures.element;
                            // 1. If alreadyCalled.[[Value]] is true, return undefined.
                            if element.already_called.get() {
                                return Ok(JsValue::undefined());
                            }

                            // 2. Set alreadyCalled.[[Value]] to true.
                            element.already_called.set(true);

                            let x = args.get_or_undefined(0).clone();

                            // 3. If variant is all, then
                            if captures.variant == KeyedVariant::All {
                                // a. Set values[index] to x.
                                element.values.borrow_mut()[element.index] = x.into_stored();
                            } else {
                                // 4. Else (variant is all-settled)
                                // a. Let obj be OrdinaryObjectCreate(%Object.prototype%).
                                let obj = JsObject::with_object_proto(context.intrinsics());
                                // b. Perform ! CreateDataPropertyOrThrow(obj, "status", "fulfilled").
                                obj.create_data_property_or_throw(
                                    js_string!("status"),
                                    js_string!("fulfilled"),
                                    context.access(),
                                )
                                .js_expect("cannot fail per spec")?;
                                // c. Perform ! CreateDataPropertyOrThrow(obj, "value", x).
                                obj.create_data_property_or_throw(
                                    js_string!("value"),
                                    x,
                                    context.access(),
                                )
                                .js_expect("cannot fail per spec")?;
                                // d. Set values[index] to obj.
                                element.values.borrow_mut()[element.index] =
                                    JsValue::from(obj).into_stored();
                            }

                            // 5. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                            element
                                .remaining_elements
                                .set(element.remaining_elements.get() - 1);

                            // 6. If remainingElementsCount.[[Value]] = 0, then
                            if element.remaining_elements.get() == 0 {
                                // a. Let result be CreateKeyedPromiseCombinatorResultObject(keys, values).
                                let result = create_keyed_result_object(
                                    &captures.keys.borrow(),
                                    &element.values.borrow(),
                                    context,
                                );
                                // b. Return ? Call(resultCapability.[[Resolve]], undefined, « result »).
                                return element.capability.call(
                                    &JsValue::undefined(),
                                    &[result.into()],
                                    context.access(),
                                );
                            }

                            // 7. Return undefined.
                            Ok(JsValue::undefined())
                        },
                        KeyedResolveCaptures {
                            element: CombinatorElementCaptures {
                                already_called: already_called.clone(),
                                index,
                                values: values.clone(),
                                capability: result_capability.functions.resolve.clone(),
                                remaining_elements: remaining_elements_count.clone(),
                            },
                            variant,
                            keys: keys.clone(),
                        },
                    ),
                )
                .name("")
                .length(1)
                .constructor(false)
                .build()?;

                // vii-viii. Build onRejected
                let on_rejected = if variant == KeyedVariant::All {
                    // If variant is all, let onRejected be resultCapability.[[Reject]].
                    result_capability.functions.reject.clone().into()
                } else {
                    // Else (variant is all-settled), let onRejected be a new Abstract Closure...
                    let on_rejected_fn = FunctionObjectBuilder::new(
                        context.realm(),
                        NativeFunction::from_builtin_with_captures(
                            |_, args, captures, context| {
                                let element = &captures.element;
                                // 1. If alreadyCalled.[[Value]] is true, return undefined.
                                if element.already_called.get() {
                                    return Ok(JsValue::undefined());
                                }

                                // 2. Set alreadyCalled.[[Value]] to true.
                                element.already_called.set(true);

                                let x = args.get_or_undefined(0).clone();

                                // 3. Let obj be OrdinaryObjectCreate(%Object.prototype%).
                                let obj = JsObject::with_object_proto(context.intrinsics());
                                // 4. Perform ! CreateDataPropertyOrThrow(obj, "status", "rejected").
                                obj.create_data_property_or_throw(
                                    js_string!("status"),
                                    js_string!("rejected"),
                                    context.access(),
                                )
                                .js_expect("cannot fail per spec")?;
                                // 5. Perform ! CreateDataPropertyOrThrow(obj, "reason", x).
                                obj.create_data_property_or_throw(
                                    js_string!("reason"),
                                    x,
                                    context.access(),
                                )
                                .js_expect("cannot fail per spec")?;
                                // 6. Set values[index] to obj.
                                element.values.borrow_mut()[element.index] =
                                    JsValue::from(obj).into_stored();

                                // 7. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                                element
                                    .remaining_elements
                                    .set(element.remaining_elements.get() - 1);

                                // 8. If remainingElementsCount.[[Value]] = 0, then
                                if element.remaining_elements.get() == 0 {
                                    let result = create_keyed_result_object(
                                        &captures.keys.borrow(),
                                        &element.values.borrow(),
                                        context,
                                    );
                                    return element.capability.call(
                                        &JsValue::undefined(),
                                        &[result.into()],
                                        context.access(),
                                    );
                                }

                                // 9. Return undefined.
                                Ok(JsValue::undefined())
                            },
                            KeyedResolveCaptures {
                                element: CombinatorElementCaptures {
                                    already_called,
                                    index,
                                    values: values.clone(),
                                    capability: result_capability.functions.resolve.clone(),
                                    remaining_elements: remaining_elements_count.clone(),
                                },
                                variant,
                                keys: keys.clone(),
                            },
                        ),
                    )
                    .name("")
                    .length(1)
                    .constructor(false)
                    .build()?;

                    on_rejected_fn.into()
                };

                // ix. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] + 1.
                remaining_elements_count.set(remaining_elements_count.get() + 1);

                // x. Perform ? Invoke(nextPromise, "then", « onFulfilled, onRejected »).
                next_promise.invoke(
                    js_string!("then"),
                    &[on_fulfilled.into(), on_rejected],
                    context,
                )?;

                // xi. Set index to index + 1.
                index += 1;
            }
        }

        // 7. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
        remaining_elements_count.set(remaining_elements_count.get() - 1);

        // 8. If remainingElementsCount.[[Value]] = 0, then
        if remaining_elements_count.get() == 0 {
            // a. Let result be CreateKeyedPromiseCombinatorResultObject(keys, values).
            let result = create_keyed_result_object(&keys.borrow(), &values.borrow(), context);
            // b. Perform ? Call(resultCapability.[[Resolve]], undefined, « result »).
            result_capability.functions.resolve.call(
                &JsValue::undefined(),
                &[result.into()],
                context.access(),
            )?;
        }

        // 9. Return resultCapability.[[Promise]].
        Ok(result_capability.promise.clone().into())
    }

    /// `Promise.any ( iterable )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.any
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/any
    pub(crate) fn any(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let C be the this value.
        let c = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Promise.any() called on a non-object")
        })?;

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(&c, context)?;

        // 3. Let promiseResolve be Completion(GetPromiseResolve(C)).
        let promise_resolve = Self::get_promise_resolve(&c, context);

        // 4. IfAbruptRejectPromise(promiseResolve, promiseCapability).
        let promise_resolve =
            if_abrupt_reject_promise!(promise_resolve, promise_capability, context);

        // 5. Let iteratorRecord be Completion(GetIterator(iterable, sync)).
        let iterator_record = args
            .get_or_undefined(0)
            .get_iterator(IteratorHint::Sync, context.access());

        // 6. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
        let mut iterator_record =
            if_abrupt_reject_promise!(iterator_record, promise_capability, context);

        // 7. Let result be Completion(PerformPromiseAny(iteratorRecord, C, promiseCapability, promiseResolve)).
        let mut result = Self::perform_promise_any(
            &mut iterator_record,
            &c,
            &promise_capability,
            &promise_resolve,
            context,
        )
        .map(JsValue::from);

        // 8. If result is an abrupt completion, then
        if result.is_err() {
            // a. If iteratorRecord.[[Done]] is false, set result to Completion(IteratorClose(iteratorRecord, result)).
            if !iterator_record.done() {
                result = iterator_record.close(result, context);
            }

            // b. IfAbruptRejectPromise(result, promiseCapability).
            let result = if_abrupt_reject_promise!(result, promise_capability, context);

            return Ok(result);
        }

        // 9. Return ? result.
        result
    }

    /// `PerformPromiseAny ( iteratorRecord, constructor, resultCapability, promiseResolve )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performpromiseany
    pub(crate) fn perform_promise_any(
        iterator_record: &mut IteratorRecord,
        constructor: &JsObject,
        result_capability: &PromiseCapability,
        promise_resolve: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 1. Let errors be a new empty List.
        let errors = context
            .runtime()
            .heap
            .allocate(RefCell::new(Vec::<StoredValue>::new()))
            .expect("an empty result list has no object edges");

        // 2. Let remainingElementsCount be the Record { [[Value]]: 1 }.
        let remaining_elements = Rc::new(Cell::new(1));

        // 3. Let index be 0.
        let mut index = 0;

        // 4. Repeat,
        //     a. Let next be ? IteratorStepValue(iteratorRecord).
        while let Some(next) = iterator_record.step_value(context)? {
            // c. Append undefined to errors.
            errors.borrow_mut().push(StoredValue::Undefined);

            // d. Let nextPromise be ? Call(promiseResolve, constructor, « next »).
            let next_promise =
                promise_resolve.call(&constructor.clone().into(), &[next], context.access())?;

            // e. Let stepsRejected be the algorithm steps defined in Promise.any Reject Element Functions.
            // f. Let lengthRejected be the number of non-optional parameters of the function definition in Promise.any Reject Element Functions.
            // g. Let onRejected be CreateBuiltinFunction(stepsRejected, lengthRejected, "", « [[AlreadyCalled]], [[Index]], [[Errors]], [[Capability]], [[RemainingElements]] »).
            // h. Set onRejected.[[AlreadyCalled]] to false.
            // i. Set onRejected.[[Index]] to index.
            // j. Set onRejected.[[Errors]] to errors.
            // k. Set onRejected.[[Capability]] to resultCapability.
            // l. Set onRejected.[[RemainingElements]] to remainingElementsCount.
            let on_rejected = FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_builtin_with_captures(
                    |_, args, captures, context| {
                        // https://tc39.es/ecma262/#sec-promise.any-reject-element-functions

                        // 1. Let F be the active function object.

                        // 2. If F.[[AlreadyCalled]] is true, return undefined.
                        if captures.already_called.get() {
                            return Ok(JsValue::undefined());
                        }

                        // 3. Set F.[[AlreadyCalled]] to true.
                        captures.already_called.set(true);

                        // 4. Let index be F.[[Index]].
                        // 5. Let errors be F.[[Errors]].
                        // 6. Let promiseCapability be F.[[Capability]].
                        // 7. Let remainingElementsCount be F.[[RemainingElements]].

                        // 8. Set errors[index] to x.
                        captures.values.borrow_mut()[captures.index] =
                            args.get_or_undefined(0).clone().into_stored();

                        // 9. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
                        captures
                            .remaining_elements
                            .set(captures.remaining_elements.get() - 1);

                        // 10. If remainingElementsCount.[[Value]] is 0, then
                        if captures.remaining_elements.get() == 0 {
                            // a. Let error be a newly created AggregateError object.
                            // b. Perform ! DefinePropertyOrThrow(error, "errors", PropertyDescriptor { [[Configurable]]: true, [[Enumerable]]: false, [[Writable]]: true, [[Value]]: CreateArrayFromList(errors) }).
                            let error = JsNativeError::aggregate(
                                captures
                                    .values
                                    .borrow()
                                    .iter()
                                    .map(|value| JsError::from_opaque(value.to_value()))
                                    .collect(),
                            )
                            .with_message("no promise in Promise.any was fulfilled.");

                            // c. Return ? Call(promiseCapability.[[Reject]], undefined, « error »).
                            return captures.capability.call(
                                &JsValue::undefined(),
                                &[error.into_opaque(context.access())?.into()],
                                context.access(),
                            );
                        }

                        // 11. Return undefined.
                        Ok(JsValue::undefined())
                    },
                    CombinatorElementCaptures {
                        already_called: Rc::new(Cell::new(false)),
                        index,
                        values: errors.clone(),
                        capability: result_capability.functions.reject.clone(),
                        remaining_elements: remaining_elements.clone(),
                    },
                ),
            )
            .name("")
            .length(1)
            .constructor(false)
            .build()?;

            // m. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] + 1.
            remaining_elements.set(remaining_elements.get() + 1);

            // n. Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], onRejected »).
            next_promise.invoke(
                js_string!("then"),
                &[
                    result_capability.functions.resolve.clone().into(),
                    on_rejected.into(),
                ],
                context,
            )?;

            // o. Set index to index + 1.
            index += 1;
        }

        //     b. If next is done, then
        //         i. Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
        remaining_elements.set(remaining_elements.get() - 1);
        //         ii. If remainingElementsCount.[[Value]] = 0, then
        if remaining_elements.get() == 0 {
            // 1. Let error be a newly created AggregateError object.
            let error = JsNativeError::aggregate(
                errors
                    .borrow()
                    .iter()
                    .map(|value| JsError::from_opaque(value.to_value()))
                    .collect(),
            )
            .with_message("no promise in Promise.any was fulfilled.");

            // 2. Perform ! DefinePropertyOrThrow(error, "errors", PropertyDescriptor { [[Configurable]]: true, [[Enumerable]]: false, [[Writable]]: true, [[Value]]: CreateArrayFromList(errors) }).
            // 3. Return ThrowCompletion(error).
            return Err(error.into());
        }

        //         iii. Return resultCapability.[[Promise]].
        Ok(result_capability.promise.clone())
    }

    /// `Promise.race ( iterable )`
    ///
    /// The `race` function returns a new promise which is settled in the same way as the first
    /// passed promise to settle. It resolves all elements of the passed `iterable` to promises.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.race
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/race
    pub(crate) fn race(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let iterable = args.get_or_undefined(0);

        // 1. Let C be the this value.
        let c = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Promise.race() called on a non-object")
        })?;

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(&c, context)?;

        // 3. Let promiseResolve be Completion(GetPromiseResolve(C)).
        let promise_resolve = Self::get_promise_resolve(&c, context);

        // 4. IfAbruptRejectPromise(promiseResolve, promiseCapability).
        let promise_resolve =
            if_abrupt_reject_promise!(promise_resolve, promise_capability, context);

        // 5. Let iteratorRecord be Completion(GetIterator(iterable, sync)).
        let iterator_record = iterable.get_iterator(IteratorHint::Sync, context.access());

        // 6. IfAbruptRejectPromise(iteratorRecord, promiseCapability).
        let mut iterator_record =
            if_abrupt_reject_promise!(iterator_record, promise_capability, context);

        // 7. Let result be Completion(PerformPromiseRace(iteratorRecord, C, promiseCapability, promiseResolve)).
        let mut result = Self::perform_promise_race(
            &mut iterator_record,
            &c,
            &promise_capability,
            &promise_resolve,
            context,
        )
        .map(JsValue::from);

        // 8. If result is an abrupt completion, then
        if result.is_err() {
            // a. If iteratorRecord.[[Done]] is false, set result to Completion(IteratorClose(iteratorRecord, result)).
            if !iterator_record.done() {
                result = iterator_record.close(result, context);
            }

            // b. IfAbruptRejectPromise(result, promiseCapability).
            let result = if_abrupt_reject_promise!(result, promise_capability, context);

            Ok(result)
        } else {
            // 9. Return ? result.
            result
        }
    }

    /// `PerformPromiseRace ( iteratorRecord, constructor, resultCapability, promiseResolve )`
    ///
    /// The abstract operation `PerformPromiseRace` takes arguments `iteratorRecord`, `constructor`
    /// (a constructor), `resultCapability` (a [`PromiseCapability`] Record), and `promiseResolve`
    /// (a function object) and returns either a normal completion containing an ECMAScript
    /// language value or a throw completion.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performpromiserace
    pub(crate) fn perform_promise_race(
        iterator_record: &mut IteratorRecord,
        constructor: &JsObject,
        result_capability: &PromiseCapability,
        promise_resolve: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let constructor = constructor.clone().into();

        // 1. Repeat,
        //     a. Let next be ? IteratorStepValue(iteratorRecord).
        while let Some(next) = iterator_record.step_value(context)? {
            // c. Let nextPromise be ? Call(promiseResolve, constructor, « next »).
            let next_promise = promise_resolve.call(&constructor, &[next], context.access())?;
            // d. Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], resultCapability.[[Reject]] »).
            next_promise.invoke(
                js_string!("then"),
                &[
                    result_capability.functions.resolve.clone().into(),
                    result_capability.functions.reject.clone().into(),
                ],
                context,
            )?;
        }

        //     b. If next is done, then
        //         i. Return resultCapability.[[Promise]].
        Ok(result_capability.promise.clone())
    }

    /// `Promise.reject ( r )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.reject
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/reject
    pub(crate) fn reject(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let r = args.get_or_undefined(0).clone();

        // 1. Let C be the this value.
        let c = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Promise.reject() called on a non-object")
        })?;

        Self::promise_reject(&c, r, context).map(JsValue::from)
    }

    /// Utility function to create a rejected promise.
    pub(crate) fn promise_reject(
        c: &JsObject,
        reason: JsValue,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(c, context)?;

        // 3. Perform ? Call(promiseCapability.[[Reject]], undefined, « r »).
        promise_capability.functions.reject.call(
            &JsValue::undefined(),
            &[reason],
            context.access(),
        )?;

        // 4. Return promiseCapability.[[Promise]].
        Ok(promise_capability.promise.clone())
    }

    /// `Promise.resolve ( x )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.resolve
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/resolve
    pub(crate) fn resolve(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let x = args.get_or_undefined(0);

        // 1. Let C be the this value.
        // 2. If Type(C) is not Object, throw a TypeError exception.
        let c = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Promise.resolve() called on a non-object")
        })?;

        // 3. Return ? PromiseResolve(C, x).
        Self::promise_resolve(&c, x.clone(), context).map(JsValue::from)
    }

    /// `PromiseResolve ( C, x )`
    ///
    /// The abstract operation `PromiseResolve` takes arguments `C` (a constructor) and `x` (an
    /// ECMAScript language value) and returns either a normal completion containing an ECMAScript
    /// language value or a throw completion. It returns a new promise resolved with `x`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise-resolve
    pub(crate) fn promise_resolve(
        c: &JsObject,
        x: JsValue,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 1. If IsPromise(x) is true, then
        if let Some(x) = x.as_promise_object() {
            let x = x.upcast();
            // a. Let xConstructor be ? Get(x, "constructor").
            let x_constructor = x.get(CONSTRUCTOR, context.access())?;
            // b. If SameValue(xConstructor, C) is true, return x.
            if x_constructor
                .as_object()
                .is_some_and(|o| JsObject::equals(&o, c))
            {
                return Ok(x);
            }
        }

        // 2. Let promiseCapability be ? NewPromiseCapability(C).
        let promise_capability = PromiseCapability::new(&c.clone(), context)?;

        // 3. Perform ? Call(promiseCapability.[[Resolve]], undefined, « x »).
        promise_capability
            .functions
            .resolve
            .call(&JsValue::undefined(), &[x], context.access())?;

        // 4. Return promiseCapability.[[Promise]].
        Ok(promise_capability.promise.clone())
    }

    /// `get Promise [ @@species ]`
    ///
    /// The `Promise [ @@species ]` accessor property returns the Promise constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-promise-@@species
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/@@species
    #[allow(clippy::unnecessary_wraps)]
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// `Promise.prototype.catch ( onRejected )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.prototype.catch
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/catch
    pub(crate) fn catch(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let on_rejected = args.get_or_undefined(0);

        // 1. Let promise be the this value.
        let promise = this;
        // 2. Return ? Invoke(promise, "then", « undefined, onRejected »).
        promise.invoke(
            js_string!("then"),
            &[JsValue::undefined(), on_rejected.clone()],
            context,
        )
    }

    /// `Promise.prototype.finally ( onFinally )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.prototype.finally
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/finally
    pub(crate) fn finally(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let promise be the this value.
        let promise = this;

        // 2. If Type(promise) is not Object, throw a TypeError exception.
        let Some(promise) = promise.as_object() else {
            return Err(JsNativeError::typ()
                .with_message("finally called with a non-object promise")
                .into());
        };

        // 3. Let C be ? SpeciesConstructor(promise, %Promise%).
        let c = promise.species_constructor(StandardConstructors::promise, context)?;

        // 4. Assert: IsConstructor(C) is true.
        debug_assert!(c.is_constructor());

        let on_finally = args.get_or_undefined(0);

        let Some(on_finally) = on_finally.as_object().and_then(JsFunction::from_object) else {
            // 5. If IsCallable(onFinally) is false, then
            //    a. Let thenFinally be onFinally.
            //    b. Let catchFinally be onFinally.
            // 7. Return ? Invoke(promise, "then", « thenFinally, catchFinally »).
            let then = promise.get(js_string!("then"), context.access())?;
            return then.call(this, &[on_finally.clone(), on_finally.clone()], context);
        };

        let (then_finally, catch_finally) =
            Self::then_catch_finally_closures(c, on_finally, context)?;

        // 7. Return ? Invoke(promise, "then", « thenFinally, catchFinally »).
        let then = promise.get(js_string!("then"), context.access())?;
        then.call(this, &[then_finally.into(), catch_finally.into()], context)
    }

    pub(crate) fn then_catch_finally_closures(
        c: JsObject,
        on_finally: JsFunction,
        context: &mut Context,
    ) -> JsResult<(JsFunction, JsFunction)> {
        // a. Let thenFinallyClosure be a new Abstract Closure with parameters (value) that captures onFinally and C and performs the following steps when called:
        let then_finally_closure = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_builtin_with_captures(
                |_this, args, (on_finally, c), context| {
                    let value = args.get_or_undefined(0);

                    // i. Let result be ? Call(onFinally, undefined).
                    let result = on_finally.call(&JsValue::undefined(), &[], context.access())?;

                    // ii. Let promise be ? PromiseResolve(C, result).
                    let promise = Self::promise_resolve(c, result, context)?;

                    // iii. Let returnValue be a new Abstract Closure with no parameters that captures value and performs the following steps when called:
                    let return_value = FunctionObjectBuilder::new(
                        context.realm(),
                        NativeFunction::from_builtin_with_captures(
                            |_this, _args, captures, _context| {
                                // 1. Return value.
                                Ok(captures.clone())
                            },
                            value.clone(),
                        ),
                    );

                    // iv. Let valueThunk be CreateBuiltinFunction(returnValue, 0, "", « »).
                    let value_thunk = return_value.length(0).name("").build()?;

                    // v. Return ? Invoke(promise, "then", « valueThunk »).
                    promise.invoke(js_string!("then"), &[value_thunk.into()], context)
                },
                (on_finally.clone(), c.clone()),
            ),
        );

        // b. Let thenFinally be CreateBuiltinFunction(thenFinallyClosure, 1, "", « »).
        let then_finally = then_finally_closure.length(1).name("").build()?;

        // c. Let catchFinallyClosure be a new Abstract Closure with parameters (reason) that captures onFinally and C and performs the following steps when called:
        let catch_finally_closure = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_builtin_with_captures(
                |_this, args, (on_finally, c), context| {
                    let reason = args.get_or_undefined(0);

                    // i. Let result be ? Call(onFinally, undefined).
                    let result = on_finally.call(&JsValue::undefined(), &[], context.access())?;

                    // ii. Let promise be ? PromiseResolve(C, result).
                    let promise = Self::promise_resolve(c, result, context)?;

                    // iii. Let throwReason be a new Abstract Closure with no parameters that captures reason and performs the following steps when called:
                    let throw_reason = FunctionObjectBuilder::new(
                        context.realm(),
                        NativeFunction::from_builtin_with_captures(
                            |_this, _args, captures, _context| {
                                // 1. Return ThrowCompletion(reason).
                                Err(JsError::from_opaque(captures.clone()))
                            },
                            reason.clone(),
                        ),
                    );

                    // iv. Let thrower be CreateBuiltinFunction(throwReason, 0, "", « »).
                    let thrower = throw_reason.length(0).name("").build()?;

                    // v. Return ? Invoke(promise, "then", « thrower »).
                    promise.invoke(js_string!("then"), &[thrower.into()], context)
                },
                (on_finally, c),
            ),
        );

        // d. Let catchFinally be CreateBuiltinFunction(catchFinallyClosure, 1, "", « »).
        let catch_finally = catch_finally_closure.length(1).name("").build()?;

        Ok((then_finally, catch_finally))
    }

    /// `Promise.prototype.then ( onFulfilled, onRejected )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-promise.prototype.then
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/then
    pub(crate) fn then(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let promise be the this value.
        let promise = this;

        // 2. If IsPromise(promise) is false, throw a TypeError exception.
        let promise = promise.as_promise_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Promise.prototype.then: this is not a promise")
        })?;

        let on_fulfilled = args
            .get_or_undefined(0)
            .as_object()
            .and_then(JsFunction::from_object);
        let on_rejected = args
            .get_or_undefined(1)
            .as_object()
            .and_then(JsFunction::from_object);

        // continues in `Promise::inner_then`
        Self::inner_then(&promise, on_fulfilled, on_rejected, context).map(JsValue::from)
    }

    /// Schedules callback functions for the eventual completion of `promise` — either fulfillment
    /// or rejection.
    pub(crate) fn inner_then(
        promise: &JsObject<Promise>,
        on_fulfilled: Option<JsFunction>,
        on_rejected: Option<JsFunction>,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 3. Let C be ? SpeciesConstructor(promise, %Promise%).
        let c = promise
            .clone()
            .upcast()
            .species_constructor(StandardConstructors::promise, context)?;

        // 4. Let resultCapability be ? NewPromiseCapability(C).
        let result_capability = PromiseCapability::new(&c, context)?;
        let result_promise = result_capability.promise.clone();
        if Self::native_association(promise)?.is_some()
            && let Ok(result) = result_promise.clone().downcast::<Self>()
        {
            Self::ensure_native_association(&result, context)?;
        }

        // 5. Return PerformPromiseThen(promise, onFulfilled, onRejected, resultCapability).
        Self::perform_promise_then(
            promise,
            on_fulfilled,
            on_rejected,
            Some(result_capability),
            context,
        )
        .into_result()?;

        Ok(result_promise)
    }

    /// `PerformPromiseThen ( promise, onFulfilled, onRejected [ , resultCapability ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performpromisethen
    pub(crate) fn perform_promise_then(
        promise: &JsObject<Self>,
        on_fulfilled: Option<JsFunction>,
        on_rejected: Option<JsFunction>,
        result_capability: Option<PromiseCapability>,
        context: &mut Context,
    ) -> ReactionRegistration {
        let target = on_fulfilled
            .as_ref()
            .and_then(Self::native_handler_target)
            .or_else(|| on_rejected.as_ref().and_then(Self::native_handler_target))
            .or_else(|| {
                result_capability
                    .as_ref()
                    .and_then(|cap| cap.promise.clone().downcast::<Self>().ok())
            });
        let target = match target {
            Some(target) => match Self::native_association(&target) {
                Ok(Some(_)) => Some(target),
                Ok(None) => None,
                Err(error) => return ReactionRegistration::uncommitted(error),
            },
            None => None,
        };
        Self::perform_promise_then_target(
            promise,
            on_fulfilled,
            on_rejected,
            result_capability,
            target,
            ReactionLifecycle::untracked(),
            context,
        )
    }
    pub(crate) fn perform_promise_then_with_lifecycle(
        promise: &JsObject<Self>,
        on_fulfilled: Option<JsFunction>,
        on_rejected: Option<JsFunction>,
        result_capability: Option<PromiseCapability>,
        lifecycle: ReactionLifecycle,
        context: &mut Context,
    ) -> ReactionRegistration {
        Self::perform_promise_then_target(
            promise,
            on_fulfilled,
            on_rejected,
            result_capability,
            None,
            lifecycle,
            context,
        )
    }
    pub(crate) fn perform_promise_then_with_native_target(
        promise: &JsObject<Self>,
        on_fulfilled: Option<JsFunction>,
        on_rejected: Option<JsFunction>,
        result_capability: Option<PromiseCapability>,
        target: &PromiseCapability,
        context: &mut Context,
    ) -> ReactionRegistration {
        let target = (|| {
            context.check_execution_stop()?;
            let target = target.native_promise()?;
            Self::ensure_native_association(&target, context)?;
            Ok::<_, JsError>(target)
        })();
        match target {
            Ok(target) => Self::perform_promise_then_target(
                promise,
                on_fulfilled,
                on_rejected,
                result_capability,
                Some(target),
                ReactionLifecycle::untracked(),
                context,
            ),
            Err(error) => ReactionRegistration::uncommitted(error),
        }
    }
    fn prepare_then_records(
        promise: &JsObject<Promise>,
        on_fulfilled: Option<JsFunction>,
        on_rejected: Option<JsFunction>,
        result_capability: Option<PromiseCapability>,
        native_target: Option<JsObject<Self>>,
        lifecycle: ReactionLifecycle,
        context: &mut Context,
    ) -> JsResult<(ReactionRecord, ReactionRecord)> {
        // 1. Assert: IsPromise(promise) is true.
        promise.check_runtime(context.runtime())?;

        if native_target.is_some() {
            Self::ensure_native_association(promise, context)?;
            if let Some(result) = result_capability
                .as_ref()
                .and_then(|cap| cap.promise.clone().downcast::<Self>().ok())
            {
                Self::ensure_native_association(&result, context)?;
            }
        }

        // 2. If resultCapability is not present, then
        //   a. Set resultCapability to undefined.

        // 3. If IsCallable(onFulfilled) is false, then
        //   a. Let onFulfilledJobCallback be empty.
        // Argument already asserts this.
        let on_fulfilled_job_callback = on_fulfilled
            // 4. Else,
            //   a. Let onFulfilledJobCallback be HostMakeJobCallback(onFulfilled).
            .map(|f| JobCallback::from_host(f, context))
            .transpose()?;

        // 5. If IsCallable(onRejected) is false, then
        //   a. Let onRejectedJobCallback be empty.
        // Argument already asserts this.
        context.check_execution_stop()?;
        let on_rejected_job_callback = on_rejected
            // 6. Else,
            //   a. Let onRejectedJobCallback be HostMakeJobCallback(onRejected).
            .map(|f| JobCallback::from_host(f, context))
            .transpose()?;

        context.check_execution_stop()?;
        let native_result_target = result_capability
            .as_ref()
            .and_then(|cap| cap.promise.clone().downcast::<Self>().ok());
        let native_result_target = match native_result_target {
            Some(target) if Self::native_association(&target)?.is_some() => Some(target),
            _ => None,
        }
        .filter(|target| {
            native_target
                .as_ref()
                .is_none_or(|primary| primary.identity() != target.identity())
        });
        // 7. Let fulfillReaction be the PromiseReaction { [[Capability]]: resultCapability, [[Type]]: Fulfill, [[Handler]]: onFulfilledJobCallback }.
        let fulfill_reaction = ReactionRecord {
            lifecycle: lifecycle.clone(),
            native_result_target: native_result_target.clone(),
            native_target: native_target.clone(),
            promise_capability: result_capability.clone(),
            reaction_type: ReactionType::Fulfill,
            handler: on_fulfilled_job_callback,
        };

        // 8. Let rejectReaction be the PromiseReaction { [[Capability]]: resultCapability, [[Type]]: Reject, [[Handler]]: onRejectedJobCallback }.
        let reject_reaction = ReactionRecord {
            lifecycle,
            native_result_target,
            native_target,
            promise_capability: result_capability,
            reaction_type: ReactionType::Reject,
            handler: on_rejected_job_callback,
        };

        Ok((fulfill_reaction, reject_reaction))
    }
    /// Installs the actual bridge pair while the source module's new canonical
    /// promise remains private. Only the source-module owner calls this command;
    /// target handoff remains uncommitted until canonical publication completes.
    pub(crate) fn prepare_pending_then(
        promise: &JsObject<Self>,
        fulfilled: JsFunction,
        rejected: JsFunction,
        target: &PromiseCapability,
        context: &mut Context,
    ) -> JsResult<PreparedPendingReactions> {
        context.check_execution_stop()?;
        let target_promise = target.native_promise()?;
        let (fulfill, reject) = Self::prepare_then_records(
            promise,
            Some(fulfilled),
            Some(rejected),
            None,
            Some(target_promise.clone()),
            ReactionLifecycle::untracked(),
            context,
        )?;
        let stored = (fulfill.to_stored(), reject.to_stored());
        let stored = context
            .runtime()
            .heap
            .with_pending_edges(stored, |stored| stored)
            .map_err(|error| {
                crate::job::JobFault::new(
                    crate::job::JobFaultOwner::NativeSignal,
                    match error {
                        surgeist_js_gc::heap::AccessError::ForeignHeap
                        | surgeist_js_gc::heap::AccessError::StaleHandle
                        | surgeist_js_gc::heap::AccessError::WrongType => {
                            crate::job::JobFaultCause::OwnerMismatch
                        }
                        surgeist_js_gc::heap::AccessError::HeapBorrowed
                        | surgeist_js_gc::heap::AccessError::PayloadBorrowed => {
                            crate::job::JobFaultCause::InaccessibleRegistry
                        }
                    },
                )
            })?;
        let owner = Self::ensure_native_association(&target_promise, context)?;
        let source = Self::ensure_native_association(promise, context)?;
        let handoff = native_settlement::Association::prepare_follow(&owner, &source)?;
        context.check_execution_stop()?;
        {
            let mut promise = promise
                .try_borrow_mut()
                .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?;
            let StoredPromiseState::Pending {
                fulfill_reactions,
                reject_reactions,
            } = &mut promise.data_mut().state
            else {
                return Err(crate::job::JobFault::new(
                    crate::job::JobFaultOwner::NativeSignal,
                    crate::job::JobFaultCause::StateInconsistency,
                )
                .into());
            };
            // Validate both stored records and reserve both destinations before
            // installing either record or changing the handled disposition.
            fulfill_reactions
                .try_reserve(1)
                .map_err(|_| crate::error::EngineResourceError::Allocation)?;
            reject_reactions
                .try_reserve(1)
                .map_err(|_| crate::error::EngineResourceError::Allocation)?;
            fulfill_reactions.push(stored.0);
            reject_reactions.push(stored.1);
            promise.data_mut().handled = true;
        }
        Ok(PreparedPendingReactions {
            promise: promise.clone(),
            roots: (fulfill, reject),
            handoff,
        })
    }
    fn perform_promise_then_target(
        promise: &JsObject<Promise>,
        on_fulfilled: Option<JsFunction>,
        on_rejected: Option<JsFunction>,
        result_capability: Option<PromiseCapability>,
        native_target: Option<JsObject<Self>>,
        lifecycle: ReactionLifecycle,
        context: &mut Context,
    ) -> ReactionRegistration {
        let mut preparation = RegistrationPreparation::new(lifecycle);
        let registration = (|| {
            context.check_execution_stop()?;
            let (fulfilled, rejected) = Self::prepare_then_records(
                promise,
                on_fulfilled,
                on_rejected,
                result_capability,
                native_target,
                preparation.lifecycle.clone(),
                context,
            )?;
            preparation.fulfilled = Some(fulfilled);
            preparation.rejected = Some(rejected);
            let (state, handled) = {
                let promise = promise
                    .try_borrow()
                    .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?;
                (promise.data().state(), promise.data().handled)
            };
            if matches!(state, PromiseState::Pending) {
                // Both actual live records remain owned before the first root
                // attempt. The lease survives commit, bookkeeping and notices.
                preparation.publication.hold = Some(preparation.lifecycle.hold()?);
                if let Some(source) = Self::native_association(promise)? {
                    let targets = {
                        let fulfilled = preparation.fulfilled()?;
                        [
                            fulfilled.native_target.clone(),
                            fulfilled.native_result_target.clone(),
                        ]
                    };
                    for (slot, target) in preparation
                        .publication
                        .notifications
                        .pending
                        .iter_mut()
                        .zip(targets)
                    {
                        if let Some(target) = target
                            && promise.identity() != target.identity()
                        {
                            let target = Self::ensure_native_association(&target, context)?;
                            *slot = Some(native_settlement::Association::prepare_follow(
                                &target, &source,
                            )?);
                        }
                    }
                }
            }
            let stored = (
                preparation.fulfilled()?.to_stored(),
                preparation.rejected()?.to_stored(),
            );
            let heap = context.runtime().heap.clone();
            let registration = heap
                .with_pending_edges(stored, |(stored_fulfill, stored_reject)| {
                    match state {
                        PromiseState::Pending => {
                            let hold = preparation
                                .publication
                                .hold
                                .as_ref()
                                .ok_or_else(reaction_state_error)?;
                            let mut promise = promise
                                .try_borrow_mut()
                                .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?;
                            let StoredPromiseState::Pending {
                                fulfill_reactions,
                                reject_reactions,
                            } = &mut promise.data_mut().state
                            else {
                                return Err(reaction_state_error());
                            };
                            fulfill_reactions
                                .try_reserve(1)
                                .map_err(|_| crate::error::EngineResourceError::Allocation)?;
                            reject_reactions
                                .try_reserve(1)
                                .map_err(|_| crate::error::EngineResourceError::Allocation)?;
                            for (notification, handoff) in preparation
                                .publication
                                .notifications
                                .committed
                                .iter_mut()
                                .zip(preparation.publication.notifications.pending.iter_mut())
                            {
                                if let Some(handoff) = handoff.take() {
                                    *notification = Some(handoff.commit());
                                }
                            }
                            fulfill_reactions.push(stored_fulfill);
                            reject_reactions.push(stored_reject);
                            // This sealed fact binds the actual Await before any
                            // notification can reenter and before live roots leave.
                            let registration = ReactionRegistration::committed(
                                ReactionCustody::SourcePending,
                                Ok(()),
                                hold,
                            );
                            promise.data_mut().handled = true;
                            Ok(registration)
                        }
                        PromiseState::Fulfilled(value) => {
                            let fulfilled = preparation
                                .fulfilled
                                .take()
                                .ok_or_else(reaction_state_error)?;
                            preparation.job.live = Some(fulfilled);
                            new_promise_reaction_job(&mut preparation.job, value, context)?;
                            let mut job = preparation.job.take_prepared()?;
                            job.commit();
                            Ok(job.publish(&mut preparation.publication, context))
                        }
                        PromiseState::Rejected(reason) => {
                            if !handled {
                                context.host_hooks().promise_rejection_tracker(
                                    promise,
                                    OperationType::Handle,
                                    context.access(),
                                );
                            }
                            let rejected = preparation
                                .rejected
                                .take()
                                .ok_or_else(reaction_state_error)?;
                            preparation.job.live = Some(rejected);
                            new_promise_reaction_job(&mut preparation.job, reason, context)?;
                            let mut job = preparation.job.take_prepared()?;
                            job.commit();
                            Ok(job.publish(&mut preparation.publication, context))
                        }
                    }
                })
                .map_err(|error| {
                    JsError::from(JsNativeError::typ().with_message(error.to_string()))
                })??;
            let registration = if registration.is_committed() {
                let handled = promise
                    .try_borrow_mut()
                    .map_err(|_| crate::error::EngineError::ObjectAccessBusy.into())
                    .map(|mut promise| {
                        promise.data_mut().handled = true;
                    });
                registration.complete(handled)
            } else {
                registration
            };
            Ok::<_, JsError>(registration)
        })();
        match registration {
            Ok(registration) => preparation.finish(registration),
            Err(error) => preparation.fail(error),
        }
    }

    /// `GetPromiseResolve ( promiseConstructor )`
    ///
    /// The abstract operation `GetPromiseResolve` takes argument `promiseConstructor` (a
    /// constructor) and returns either a normal completion containing a function object or a throw
    /// completion.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getpromiseresolve
    pub(crate) fn get_promise_resolve(
        promise_constructor: &JsObject,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 1. Let promiseResolve be ? Get(promiseConstructor, "resolve").
        let promise_resolve = promise_constructor.get(js_string!("resolve"), context.access())?;

        // 2. If IsCallable(promiseResolve) is false, throw a TypeError exception.
        promise_resolve.as_callable().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("retrieving a non-callable promise resolver")
                .into()
        })
    }

    /// `CreateResolvingFunctions ( promise )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createresolvingfunctions
    pub(crate) fn create_resolving_functions(
        promise: &JsObject<Promise>,
        context: &mut Context,
    ) -> ResolvingFunctions {
        fn run_resolution(
            promise: &JsObject<Promise>,
            context: &mut Context,
            run: impl FnOnce(&mut Context) -> JsResult<JsValue>,
        ) -> JsResult<JsValue> {
            context.check_execution_stop()?;
            let attempt = Promise::native_association(promise)?
                .map(|owner| owner.resolution_lease().map(|lease| (owner, lease)))
                .transpose()?;
            let result = run(context);
            let result = context.check_execution_stop().and(result);
            if result
                .as_ref()
                .is_err_and(|error| error.as_engine().is_some())
                && let Some((owner, lease)) = attempt
            {
                owner.interrupt_resolution(lease);
            }
            result
        }
        fn take_target(slot: &RefCell<Option<ObjectEdge>>) -> Option<JsObject<Promise>> {
            let mut slot = slot.borrow_mut();
            let promise = JsObject::from_inner(slot.as_ref()?.clone())
                .downcast::<Promise>()
                .expect("the resolving functions retain a Promise");
            *slot = None;
            Some(promise)
        }

        /// `TriggerPromiseReactions ( reactions, argument )`
        ///
        /// The abstract operation `TriggerPromiseReactions` takes arguments `reactions` (a `List` of
        /// `PromiseReaction` Records) and `argument` and returns unused. It enqueues a new `Job` for
        /// each record in `reactions`. Each such `Job` processes the `[[Type]]` and `[[Handler]]` of
        /// the `PromiseReaction` Record, and if the `[[Handler]]` is not `empty`, calls it passing the
        /// given argument. If the `[[Handler]]` is `empty`, the behaviour is determined by the
        /// `[[Type]]`.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-triggerpromisereactions
        fn prepare_promise_reactions(
            reactions: Vec<ReactionRecord>,
            association: Option<&Rooted<native_settlement::Association>>,
            argument: &JsValue,
            context: &mut Context,
        ) -> JsResult<Vec<PreparedReactionJob>> {
            let mut preparation = ReactionPreparation {
                remaining: reactions.into_iter(),
                prepared: Vec::new(),
                current: ReactionJobPreparation::new(),
                association,
            };
            if preparation
                .prepared
                .try_reserve(preparation.remaining.len())
                .is_err()
            {
                return Err(preparation.fail(crate::error::EngineResourceError::Allocation.into()));
            }
            while let Some(reaction) = preparation.remaining.next() {
                preparation.current.live = Some(reaction);
                if let Err(error) =
                    new_promise_reaction_job(&mut preparation.current, argument.clone(), context)
                {
                    return Err(preparation.fail(error));
                }
                match preparation.current.take_prepared() {
                    Ok(job) => preparation.prepared.push(job),
                    Err(error) => return Err(preparation.fail(error)),
                }
            }
            Ok(std::mem::take(&mut preparation.prepared))
        }

        /// `FulfillPromise ( promise, value )`
        ///
        /// The abstract operation `FulfillPromise` takes arguments `promise` and `value` and returns
        /// `unused`.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-fulfillpromise
        ///
        /// # Panics
        ///
        /// Panics if `Promise` is not pending.
        fn fulfill_promise(
            promise: &JsObject<Promise>,
            value: JsValue,
            context: &mut Context,
        ) -> JsResult<()> {
            context.check_execution_stop()?;
            let association = Promise::native_association(promise)?;
            if let Some(association) = &association
                && let Some(error) = native_settlement::Association::module_error(association)
            {
                return Err(error);
            }
            let mut setup = ReactionRestoration::new(promise, association.as_ref());
            let heap = context.runtime().heap.clone();
            let result = heap.with_pending_edges(ReactionSourceRoots(promise.clone()), |_| {
                let mut restoration = ReactionRestoration::new(promise, association.as_ref());
                setup.enabled = false;
                let result = (|| {
                    restoration.detach()?;
                    let reactions = restoration.restore(ReactionType::Fulfill)?;
                    let mut jobs = prepare_promise_reactions(
                        reactions,
                        association.as_ref(),
                        &value,
                        context,
                    )?;
                    for job in &mut jobs {
                        job.commit();
                    }
                    let pending = restoration.take_pending()?;
                    let batch = ReactionBatch::new(jobs, pending);
                    // 3–6. Store the result and mark the promise fulfilled.
                    promise
                        .try_borrow_mut()
                        .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?
                        .data_mut()
                        .state = StoredPromiseState::Fulfilled(value.clone().into_stored());
                    restoration.enabled = false;
                    // 7. Perform TriggerPromiseReactions(reactions, value).
                    if let Some(association) = &association {
                        association.settle();
                    }
                    batch.publish(context)
                })();
                restoration.finish(result)
            });
            setup.finish(result.map_err(reaction_access_error).flatten())
        }

        /// `RejectPromise ( promise, reason )`
        ///
        /// The abstract operation `RejectPromise` takes arguments `promise` and `reason` and returns
        /// `unused`.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-rejectpromise
        ///
        /// # Panics
        ///
        /// Panics if `Promise` is not pending.
        fn reject_promise(
            promise: &JsObject<Promise>,
            reason: JsValue,
            context: &mut Context,
        ) -> JsResult<()> {
            context.check_execution_stop()?;
            let association = Promise::native_association(promise)?;
            if let Some(association) = &association
                && let Some(error) = native_settlement::Association::module_error(association)
            {
                return Err(error);
            }
            let mut setup = ReactionRestoration::new(promise, association.as_ref());
            let heap = context.runtime().heap.clone();
            let result = heap.with_pending_edges(ReactionSourceRoots(promise.clone()), |_| {
                let mut restoration = ReactionRestoration::new(promise, association.as_ref());
                setup.enabled = false;
                let result = (|| {
                    let handled = restoration.detach()?;
                    let reactions = restoration.restore(ReactionType::Reject)?;
                    let mut jobs = prepare_promise_reactions(
                        reactions,
                        association.as_ref(),
                        &reason,
                        context,
                    )?;
                    for job in &mut jobs {
                        job.commit();
                    }
                    let pending = restoration.take_pending()?;
                    let batch = ReactionBatch::new(jobs, pending);
                    // 3–6. Store the reason and mark the promise rejected.
                    promise
                        .try_borrow_mut()
                        .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?
                        .data_mut()
                        .state = StoredPromiseState::Rejected(reason.clone().into_stored());
                    restoration.enabled = false;
                    if let Some(association) = &association {
                        association.settle();
                    }
                    // 7. Track an unhandled rejection after actual settlement.
                    if !handled {
                        context.host_hooks().promise_rejection_tracker(
                            promise,
                            OperationType::Reject,
                            context.access(),
                        );
                    }
                    // 8. Perform TriggerPromiseReactions(reactions, reason).
                    batch.publish(context)
                })();
                restoration.finish(result)
            });
            setup.finish(result.map_err(reaction_access_error).flatten())
        }

        // 1. Let alreadyResolved be the Record { [[Value]]: false }.
        // 5. Set resolve.[[Promise]] to promise.
        // 6. Set resolve.[[AlreadyResolved]] to alreadyResolved.
        let promise = context
            .runtime()
            .heap
            .allocate(RefCell::new(Some(promise.edge())))
            .expect("the promise belongs to the resolving function's heap");
        let native_target = promise.clone();

        // 2. Let stepsResolve be the algorithm steps defined in Promise Resolve Functions.
        // 3. Let lengthResolve be the number of non-optional parameters of the function definition in Promise Resolve Functions.
        // 4. Let resolve be CreateBuiltinFunction(stepsResolve, lengthResolve, "", « [[Promise]], [[AlreadyResolved]] »).
        let resolve = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_builtin_with_captures(
                |_this, args, captures, context| {
                    // https://tc39.es/ecma262/#sec-promise-resolve-functions

                    // 1. Let F be the active function object.
                    // 2. Assert: F has a [[Promise]] internal slot whose value is an Object.
                    // 3. Let promise be F.[[Promise]].
                    // 4. Let alreadyResolved be F.[[AlreadyResolved]].
                    // 5. If alreadyResolved.[[Value]] is true, return undefined.
                    // 6. Set alreadyResolved.[[Value]] to true.
                    let Some(promise) = take_target(captures) else {
                        return Ok(JsValue::undefined());
                    };

                    run_resolution(&promise, context, |context| {
                        let resolution = args.get_or_undefined(0);

                        // 7. If SameValue(resolution, promise) is true, then
                        if JsValue::same_value(resolution, &promise.clone().into()) {
                            //   a. Let selfResolutionError be a newly created TypeError object.
                            let self_resolution_error = JsNativeError::typ()
                                .with_message("SameValue(resolution, promise) is true")
                                .into_opaque(context.access())?;

                            //   b. Perform RejectPromise(promise, selfResolutionError).
                            reject_promise(&promise, self_resolution_error.into(), context)?;

                            //   c. Return undefined.
                            return Ok(JsValue::undefined());
                        }

                        let Some(then) = resolution.as_object() else {
                            // 8. If Type(resolution) is not Object, then
                            //   a. Perform FulfillPromise(promise, resolution).
                            fulfill_promise(&promise, resolution.clone(), context)?;

                            //   b. Return undefined.
                            return Ok(JsValue::undefined());
                        };

                        // 9. Let then be Completion(Get(resolution, "then")).
                        let then_completion = then.get(js_string!("then"), context.access());
                        context.check_execution_stop()?;
                        let then_action = match then_completion {
                            // 10. If then is an abrupt completion, then
                            Err(e) => {
                                //   a. Perform RejectPromise(promise, then.[[Value]]).
                                reject_promise(
                                    &promise,
                                    e.into_opaque(context.access())?,
                                    context,
                                )?;

                                //   b. Return undefined.
                                return Ok(JsValue::undefined());
                            }
                            // 11. Let thenAction be then.[[Value]].
                            Ok(then) => then,
                        };

                        // 12. If IsCallable(thenAction) is false, then
                        let Some(then_action) =
                            then_action.as_object().and_then(JsFunction::from_object)
                        else {
                            // a. Perform FulfillPromise(promise, resolution).
                            fulfill_promise(&promise, resolution.clone(), context)?;

                            //   b. Return undefined.
                            return Ok(JsValue::undefined());
                        };

                        // 13. Let thenJobCallback be HostMakeJobCallback(thenAction).
                        let result =
                            JobCallback::from_host(then_action, context).and_then(|callback| {
                                context.check_execution_stop()?;
                                let heap = context.runtime().heap.clone();
                                heap.with_pending_edges(callback.to_stored(), |_| {
                                    // 14. Let job be NewPromiseResolveThenableJob(promise, resolution, thenJobCallback).
                                    let (job, handoff) = new_promise_resolve_thenable_job(
                                        promise.clone(),
                                        resolution.clone(),
                                        callback,
                                        context,
                                    )?;

                                    let notifications = ReactionNotifications {
                                        pending: [None, None],
                                        committed: [
                                            handoff.map(native_settlement::PreparedHandoff::commit),
                                            None,
                                        ],
                                    };
                                    // 15. Perform HostEnqueuePromiseJob(job.[[Job]], job.[[Realm]]).
                                    let publication = crate::job::finish_publication(
                                        context.enqueue_job(job.into()),
                                    );
                                    notifications.finish();
                                    publication
                                })
                                .map_err(|error| {
                                    JsError::from(
                                        JsNativeError::typ().with_message(error.to_string()),
                                    )
                                })?
                            });
                        if let Err(error) = result {
                            context.check_execution_stop()?;
                            if error.as_engine().is_some() {
                                return Err(error);
                            }
                            reject_promise(
                                &promise,
                                error.into_opaque(context.access())?,
                                context,
                            )?;
                        }

                        // 16. Return undefined.
                        Ok(JsValue::undefined())
                    })
                },
                promise.clone(),
            )
            .with_resolution_target(&native_target),
        )
        .name("")
        .length(1)
        .constructor(false)
        .build()
        .expect("resolving function retains a promise from the current runtime");

        // 10. Set reject.[[Promise]] to promise.
        // 11. Set reject.[[AlreadyResolved]] to alreadyResolved.
        // 7. Let stepsReject be the algorithm steps defined in Promise Reject Functions.
        // 8. Let lengthReject be the number of non-optional parameters of the function definition in Promise Reject Functions.
        // 9. Let reject be CreateBuiltinFunction(stepsReject, lengthReject, "", « [[Promise]], [[AlreadyResolved]] »).
        let reject = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_builtin_with_captures(
                |_this, args, captures, context| {
                    // https://tc39.es/ecma262/#sec-promise-reject-functions

                    // 1. Let F be the active function object.
                    // 2. Assert: F has a [[Promise]] internal slot whose value is an Object.
                    // 3. Let promise be F.[[Promise]].
                    // 4. Let alreadyResolved be F.[[AlreadyResolved]].
                    // 5. If alreadyResolved.[[Value]] is true, return undefined.
                    // 6. Set alreadyResolved.[[Value]] to true.
                    let Some(promise) = take_target(captures) else {
                        return Ok(JsValue::undefined());
                    };

                    run_resolution(&promise, context, |context| {
                        // 7. Perform RejectPromise(promise, reason).
                        reject_promise(&promise, args.get_or_undefined(0).clone(), context)?;

                        // 8. Return undefined.
                        Ok(JsValue::undefined())
                    })
                },
                promise,
            )
            .with_resolution_target(&native_target),
        )
        .name("")
        .length(1)
        .constructor(false)
        .build()
        .expect("resolving function retains a promise from the current runtime");

        // 12. Return the Record { [[Resolve]]: resolve, [[Reject]]: reject }.
        ResolvingFunctions { resolve, reject }
    }
}

/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-newpromisereactionjob
/// A private canonical promise already holds the installed pair. This command
/// retains its live roots and uncommitted handoff until publication has finished.
pub(crate) struct PreparedPendingReactions {
    promise: JsObject<Promise>,
    roots: (ReactionRecord, ReactionRecord),
    handoff: native_settlement::PreparedHandoff,
}
impl PreparedPendingReactions {
    pub(crate) fn notify(self) {
        // The caller has released all status/promise/cell borrows before disposal
        // of the previous native binding or arbitrary native wake callbacks.
        let Self {
            promise,
            roots,
            handoff,
        } = self;
        let (fulfilled, rejected) = roots;
        let mut cleanup = crate::job::UnwindCleanup::default();
        cleanup.run(|| handoff.commit().notify());
        cleanup.run(|| drop(promise));
        cleanup.run(|| drop(fulfilled));
        cleanup.run(|| drop(rejected));
        cleanup.finish();
    }
}

/// Owns native handoff cleanup while the corresponding job crosses its host
/// publication boundary. Each slot finishes independently during native unwind.
#[derive(Default)]
struct ReactionNotifications {
    pending: [Option<native_settlement::PreparedHandoff>; 2],
    committed: [Option<native_settlement::HandoffNotification>; 2],
}
impl ReactionNotifications {
    fn finish(self) {
        drop(self);
    }
}
impl Drop for ReactionNotifications {
    fn drop(&mut self) {
        let mut cleanup = crate::job::UnwindCleanup::default();
        for slot in &mut self.pending {
            let handoff = slot.take();
            cleanup.run(|| drop(handoff));
        }
        for slot in &mut self.committed {
            let notification = slot.take();
            cleanup.run(|| drop(notification));
        }
        cleanup.finish();
    }
}

fn reaction_state_error() -> JsError {
    crate::job::JobFault::new(
        crate::job::JobFaultOwner::Task,
        crate::job::JobFaultCause::StateInconsistency,
    )
    .into()
}

fn reaction_access_error(error: surgeist_js_gc::heap::AccessError) -> JsError {
    use crate::job::{JobFault, JobFaultCause, JobFaultOwner};
    use surgeist_js_gc::heap::AccessError;
    let cause = match error {
        AccessError::PayloadBorrowed => return crate::error::EngineError::ObjectAccessBusy.into(),
        AccessError::HeapBorrowed => JobFaultCause::InaccessibleRegistry,
        AccessError::ForeignHeap | AccessError::StaleHandle => JobFaultCause::OwnerMismatch,
        AccessError::WrongType => JobFaultCause::StateInconsistency,
    };
    JobFault::new(JobFaultOwner::Task, cause).into()
}

/// Producer-only custody. Its root lease is released after every saved notice.
#[derive(Default)]
struct ReactionPublication {
    rejected: Option<crate::job::Job>,
    notifications: ReactionNotifications,
    hold: Option<PublicationHold>,
}
impl ReactionPublication {
    fn cleanup(&mut self, cleanup: &mut crate::job::UnwindCleanup) {
        let rejected = self.rejected.take();
        cleanup.run(|| drop(rejected));
        let notifications = std::mem::take(&mut self.notifications);
        cleanup.run(|| notifications.finish());
        let hold = self.hold.take();
        cleanup.run(|| drop(hold));
    }
    fn finish<T>(mut self, value: T) -> T {
        let mut cleanup = crate::job::UnwindCleanup::default();
        self.cleanup(&mut cleanup);
        cleanup.finish_with(value)
    }
}
impl Drop for ReactionPublication {
    fn drop(&mut self) {
        let mut cleanup = crate::job::UnwindCleanup::default();
        self.cleanup(&mut cleanup);
        cleanup.finish();
    }
}

struct RegistrationPreparation {
    lifecycle: ReactionLifecycle,
    fulfilled: Option<ReactionRecord>,
    rejected: Option<ReactionRecord>,
    job: ReactionJobPreparation,
    publication: ReactionPublication,
}
impl RegistrationPreparation {
    fn new(lifecycle: ReactionLifecycle) -> Self {
        Self {
            lifecycle,
            fulfilled: None,
            rejected: None,
            job: ReactionJobPreparation::new(),
            publication: ReactionPublication::default(),
        }
    }
    fn fulfilled(&self) -> JsResult<&ReactionRecord> {
        self.fulfilled.as_ref().ok_or_else(reaction_state_error)
    }
    fn rejected(&self) -> JsResult<&ReactionRecord> {
        self.rejected.as_ref().ok_or_else(reaction_state_error)
    }
    fn cleanup(&mut self, cleanup: &mut crate::job::UnwindCleanup) {
        cleanup.run(|| self.job.unavailable());
        let fulfilled = self.fulfilled.take();
        cleanup.run(|| drop(fulfilled));
        let rejected = self.rejected.take();
        cleanup.run(|| drop(rejected));
        self.job.cleanup(cleanup);
        self.publication.cleanup(cleanup);
    }
    fn finish(mut self, registration: ReactionRegistration) -> ReactionRegistration {
        let mut cleanup = crate::job::UnwindCleanup::default();
        self.cleanup(&mut cleanup);
        cleanup.finish_with(registration)
    }
    fn fail(mut self, error: JsError) -> ReactionRegistration {
        let mut cleanup = crate::job::UnwindCleanup::default();
        cleanup.run(|| {
            let _secondary = self.lifecycle.preparation_failed(&error);
        });
        cleanup.run(|| self.job.preparation_failed(&error));
        self.cleanup(&mut cleanup);
        cleanup.finish_with(ReactionRegistration::uncommitted(error))
    }
}
impl Drop for RegistrationPreparation {
    fn drop(&mut self) {
        let mut cleanup = crate::job::UnwindCleanup::default();
        self.cleanup(&mut cleanup);
        cleanup.finish();
    }
}

struct PreparedReactionJob {
    job: PromiseJob,
    handoffs: [Option<native_settlement::PreparedHandoff>; 2],
    notifications: [Option<native_settlement::HandoffNotification>; 2],
    hold: PublicationHold,
}
impl PreparedReactionJob {
    fn commit(&mut self) {
        for (notification, handoff) in self.notifications.iter_mut().zip(self.handoffs.iter_mut()) {
            if let Some(handoff) = handoff.take() {
                *notification = Some(handoff.commit());
            }
        }
    }
    fn publish(
        self,
        publication: &mut ReactionPublication,
        context: &mut Context,
    ) -> ReactionRegistration {
        let Self {
            job,
            handoffs,
            notifications,
            hold,
        } = self;
        publication.notifications = ReactionNotifications {
            pending: handoffs,
            committed: notifications,
        };
        publication.hold = Some(hold);
        let hold = match publication.hold.as_ref() {
            Some(hold) => hold,
            None => {
                publication.rejected = Some(job.into());
                return ReactionRegistration::uncommitted(reaction_state_error());
            }
        };
        match context.enqueue_job(job.into()) {
            Ok(crate::job::JobAcceptance::Accepted) => {
                ReactionRegistration::committed(ReactionCustody::ExecutorAccepted, Ok(()), hold)
            }
            Ok(crate::job::JobAcceptance::AcceptedWithError(cause)) => {
                ReactionRegistration::committed(
                    ReactionCustody::ExecutorAccepted,
                    Err(cause.into_error()),
                    hold,
                )
            }
            Ok(crate::job::JobAcceptance::Discarded) => {
                ReactionRegistration::committed(ReactionCustody::PolicyDiscarded, Ok(()), hold)
            }
            Err(error) => {
                let (job, cause) = error.into_parts();
                publication.rejected = Some(job);
                let error = cause.into_error();
                let mut cleanup = crate::job::UnwindCleanup::default();
                cleanup.run(|| {
                    let _secondary = hold.refused(&error);
                });
                cleanup.finish_with(ReactionRegistration::uncommitted(error))
            }
        }
    }
    fn preparation_failed(&self, error: &JsError) {
        if let Err(fault) = self.hold.preparation_failed(error) {
            self.job.observer().record_fault(fault);
        }
    }
    fn dispose(self, cleanup: &mut crate::job::UnwindCleanup) {
        let Self {
            job,
            handoffs,
            notifications,
            hold,
        } = self;
        let publication = ReactionPublication {
            rejected: Some(job.into()),
            notifications: ReactionNotifications {
                pending: handoffs,
                committed: notifications,
            },
            hold: Some(hold),
        };
        cleanup.run(|| publication.finish(()));
    }
}

/// Caller-owned slots retain the live input or its partial job through failure.
struct ReactionJobPreparation {
    live: Option<ReactionRecord>,
    prepared: Option<PreparedReactionJob>,
}
impl ReactionJobPreparation {
    fn new() -> Self {
        Self {
            live: None,
            prepared: None,
        }
    }
    fn take_prepared(&mut self) -> JsResult<PreparedReactionJob> {
        self.prepared.take().ok_or_else(reaction_state_error)
    }
    fn is_empty(&self) -> bool {
        self.live.is_none() && self.prepared.is_none()
    }
    fn unavailable(&self) {
        // Intrinsic secondary faults are latched by the actual Await owner;
        // they cannot replace this preparation's earlier full error.
        if let Some(record) = &self.live {
            let _secondary = record.lifecycle.unavailable();
        }
        if let Some(job) = &self.prepared {
            let _secondary = job.hold.unavailable();
        }
    }
    fn preparation_failed(&self, error: &JsError) {
        if let Some(record) = &self.live {
            let _secondary = record.lifecycle.preparation_failed(error);
        }
        if let Some(job) = &self.prepared {
            job.preparation_failed(error);
        }
    }
    fn cleanup(&mut self, cleanup: &mut crate::job::UnwindCleanup) {
        let live = self.live.take();
        cleanup.run(|| drop(live));
        if let Some(job) = self.prepared.take() {
            job.dispose(cleanup);
        }
    }
}
impl Drop for ReactionJobPreparation {
    fn drop(&mut self) {
        let mut cleanup = crate::job::UnwindCleanup::default();
        cleanup.run(|| self.unavailable());
        self.cleanup(&mut cleanup);
        cleanup.finish();
    }
}

/// Snapshot the outgoing bank edges before moving either bank out of its owner.
/// The pending-edge barrier retains these direct roots throughout its callback.
struct ReactionSourceRoots(JsObject<Promise>);
impl HeapTrace for ReactionSourceRoots {
    fn trace(
        &self,
        edges: &mut surgeist_js_gc::heap::EdgeVisitor<'_>,
    ) -> Result<(), surgeist_js_gc::heap::AccessError> {
        let promise = self
            .0
            .try_borrow()
            .map_err(|_| surgeist_js_gc::heap::AccessError::PayloadBorrowed)?;
        let StoredPromiseState::Pending {
            fulfill_reactions,
            reject_reactions,
        } = &promise.data().state
        else {
            return Err(surgeist_js_gc::heap::AccessError::WrongType);
        };
        fulfill_reactions.trace(edges)?;
        reject_reactions.trace(edges)
    }
}

/// Owns original stored tails and the restored live prefix before host restore.
/// A detached instance never escapes the pending-edge barrier that roots it.
struct ReactionRestoration<'a> {
    source: &'a JsObject<Promise>,
    association: Option<&'a Rooted<native_settlement::Association>>,
    pending: Option<StoredPromiseState>,
    live: Vec<ReactionRecord>,
    detached: bool,
    availability_finished: bool,
    enabled: bool,
}
impl<'a> ReactionRestoration<'a> {
    fn new(
        source: &'a JsObject<Promise>,
        association: Option<&'a Rooted<native_settlement::Association>>,
    ) -> Self {
        Self {
            source,
            association,
            pending: None,
            live: Vec::new(),
            detached: false,
            availability_finished: false,
            enabled: true,
        }
    }
    fn detach(&mut self) -> JsResult<bool> {
        let mut source = self
            .source
            .try_borrow_mut()
            .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?;
        let handled = source.data().handled;
        let StoredPromiseState::Pending {
            fulfill_reactions,
            reject_reactions,
        } = &mut source.data_mut().state
        else {
            return Err(reaction_state_error());
        };
        self.pending = Some(StoredPromiseState::Pending {
            fulfill_reactions: std::mem::take(fulfill_reactions),
            reject_reactions: std::mem::take(reject_reactions),
        });
        self.detached = true;
        Ok(handled)
    }
    fn restore(&mut self, kind: ReactionType) -> JsResult<Vec<ReactionRecord>> {
        let Some(StoredPromiseState::Pending {
            fulfill_reactions,
            reject_reactions,
        }) = &self.pending
        else {
            return Err(reaction_state_error());
        };
        let records = match kind {
            ReactionType::Fulfill => fulfill_reactions,
            ReactionType::Reject => reject_reactions,
        };
        self.live
            .try_reserve(records.len())
            .map_err(|_| crate::error::EngineResourceError::Allocation)?;
        for record in records {
            self.live.push(record.to_reaction());
        }
        Ok(std::mem::take(&mut self.live))
    }
    fn take_pending(&mut self) -> JsResult<StoredPromiseState> {
        self.pending.take().ok_or_else(reaction_state_error)
    }
    fn attached_lifecycle(
        &self,
        kind: ReactionType,
        index: usize,
    ) -> JsResult<Option<ReactionLifecycle>> {
        let source = self
            .source
            .try_borrow()
            .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?;
        let StoredPromiseState::Pending {
            fulfill_reactions,
            reject_reactions,
        } = &source.data().state
        else {
            return Err(reaction_state_error());
        };
        let records = match kind {
            ReactionType::Fulfill => fulfill_reactions,
            ReactionType::Reject => reject_reactions,
        };
        Ok(records.get(index).map(|record| record.lifecycle.clone()))
    }
    fn unavailable_attached(&self, cleanup: &mut crate::job::UnwindCleanup) -> JsResult<()> {
        // Only setup refusal reaches this path: no opaque restore has run and
        // the original source still roots its banks. Read each closed edge fact,
        // release the source borrow, then perform the owning cleanup event.
        // This event only withdraws stored continuations and exact evaluation
        // links; it does not restore/drop opaque callback captures, collect the
        // heap, or notify native waiters. Valid Trace/NativeCapture operations
        // leave the owner unchanged, so original indices stay readable until
        // the full pass ends. Module failure and retirement notifications must
        // remain outside this attached-source loop.
        let lengths = {
            let source = self
                .source
                .try_borrow()
                .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?;
            let StoredPromiseState::Pending {
                fulfill_reactions,
                reject_reactions,
            } = &source.data().state
            else {
                return Err(reaction_state_error());
            };
            [fulfill_reactions.len(), reject_reactions.len()]
        };
        for (kind, length) in [ReactionType::Fulfill, ReactionType::Reject]
            .into_iter()
            .zip(lengths)
        {
            for index in 0..length {
                let lifecycle = self
                    .attached_lifecycle(kind, index)?
                    .ok_or_else(reaction_state_error)?;
                cleanup.run(|| {
                    let _secondary = lifecycle.unavailable();
                });
            }
        }
        Ok(())
    }
    fn forward_failure(&mut self, error: &JsError, cleanup: &mut crate::job::UnwindCleanup) {
        // This owner must outlive the source borrow and drain before source or
        // detached bank custody ends. EvaluationRef alone is not a GC root.
        let mut notices = crate::module::lifecycle::ModuleFailureNotices::default();
        let prepared = if let Some(StoredPromiseState::Pending {
            fulfill_reactions,
            reject_reactions,
        }) = &self.pending
        {
            for record in fulfill_reactions.iter().chain(reject_reactions) {
                cleanup.run(|| {
                    let _secondary = record
                        .lifecycle
                        .defer_preparation_failed(error, &mut notices);
                });
                cleanup.run(|| {
                    let _secondary = record.lifecycle.unavailable();
                });
            }
            Ok(())
        } else if !self.detached {
            (|| {
                let source = self
                    .source
                    .try_borrow()
                    .map_err(|_| crate::error::EngineError::ObjectAccessBusy)?;
                let StoredPromiseState::Pending {
                    fulfill_reactions,
                    reject_reactions,
                } = &source.data().state
                else {
                    return Err(reaction_state_error());
                };
                let fulfill_length = fulfill_reactions.len();
                let reject_length = reject_reactions.len();
                // The shared resolving slot was consumed before this cleanup.
                // Reentry can append records but cannot settle/replace these
                // banks. This pass performs no native notification: exact
                // closed continuation cleanup and graph marking happen first.
                for record in fulfill_reactions
                    .iter()
                    .take(fulfill_length)
                    .chain(reject_reactions.iter().take(reject_length))
                {
                    cleanup.run(|| {
                        let _secondary = record
                            .lifecycle
                            .defer_preparation_failed(error, &mut notices);
                    });
                    cleanup.run(|| {
                        let _secondary = record.lifecycle.unavailable();
                    });
                }
                Ok(())
            })()
        } else {
            Ok(())
        };
        if prepared.is_ok() {
            // No attached-source reads may follow arbitrary notice callbacks:
            // they could append records or retain an incompatible object alias.
            self.availability_finished = true;
        }
        cleanup.run(|| drop(prepared));
        // The source borrow has ended, but all original records and persistent
        // evaluation graph links still trace every queued notice through panic
        // cleanup. Only then can original record custody be released.
        cleanup.run(|| notices.finish());
    }
    fn cleanup(&mut self, cleanup: &mut crate::job::UnwindCleanup) {
        if self.enabled {
            if !self.availability_finished {
                if let Some(StoredPromiseState::Pending {
                    fulfill_reactions,
                    reject_reactions,
                }) = &self.pending
                {
                    for record in fulfill_reactions.iter().chain(reject_reactions) {
                        cleanup.run(|| {
                            let _secondary = record.lifecycle.unavailable();
                        });
                    }
                } else if !self.detached {
                    // An access refusal is a secondary checked setup failure,
                    // never authority to replace an existing cause.
                    let result = self.unavailable_attached(cleanup);
                    cleanup.run(|| drop(result));
                }
            }
            if let Some(association) = self.association {
                cleanup.run(|| association.retire_pending(crate::job::JobRetirement::Interrupted));
            }
            self.enabled = false;
        }
        for record in self.live.drain(..) {
            cleanup.run(|| drop(record));
        }
        match self.pending.take() {
            Some(StoredPromiseState::Pending {
                fulfill_reactions,
                reject_reactions,
            }) => {
                for record in fulfill_reactions.into_iter().chain(reject_reactions) {
                    cleanup.run(|| drop(record));
                }
            }
            Some(StoredPromiseState::Fulfilled(value) | StoredPromiseState::Rejected(value)) => {
                cleanup.run(|| drop(value));
            }
            None => {}
        }
    }
    fn finish<T>(mut self, value: JsResult<T>) -> JsResult<T> {
        let mut cleanup = crate::job::UnwindCleanup::default();
        if self.enabled
            && let Err(error) = &value
        {
            self.forward_failure(error, &mut cleanup);
        }
        self.cleanup(&mut cleanup);
        cleanup.finish_with(value)
    }
}
impl Drop for ReactionRestoration<'_> {
    fn drop(&mut self) {
        let mut cleanup = crate::job::UnwindCleanup::default();
        self.cleanup(&mut cleanup);
        cleanup.finish();
    }
}

/// Current, already-prepared and unprepared records stay owned before rooting.
/// Failure invalidates every required tail before disposal or native notices.
struct ReactionPreparation<'a> {
    remaining: std::vec::IntoIter<ReactionRecord>,
    prepared: Vec<PreparedReactionJob>,
    current: ReactionJobPreparation,
    association: Option<&'a Rooted<native_settlement::Association>>,
}
impl ReactionPreparation<'_> {
    fn unavailable(&self, cleanup: &mut crate::job::UnwindCleanup) {
        cleanup.run(|| self.current.unavailable());
        for job in &self.prepared {
            cleanup.run(|| {
                let _secondary = job.hold.unavailable();
            });
        }
        for record in self.remaining.as_slice() {
            cleanup.run(|| {
                let _secondary = record.lifecycle.unavailable();
            });
        }
    }
    fn cleanup(&mut self, cleanup: &mut crate::job::UnwindCleanup) {
        self.current.cleanup(cleanup);
        for job in self.prepared.drain(..) {
            job.dispose(cleanup);
        }
        for record in self.remaining.by_ref() {
            cleanup.run(|| drop(record));
        }
    }
    fn fail(mut self, error: JsError) -> JsError {
        let mut cleanup = crate::job::UnwindCleanup::default();
        self.unavailable(&mut cleanup);
        cleanup.run(|| self.current.preparation_failed(&error));
        for job in &self.prepared {
            cleanup.run(|| job.preparation_failed(&error));
        }
        for record in self.remaining.as_slice() {
            cleanup.run(|| {
                let _secondary = record.lifecycle.preparation_failed(&error);
            });
        }
        if let Some(association) = self.association {
            cleanup.run(|| association.retire_pending(crate::job::JobRetirement::Interrupted));
        }
        self.cleanup(&mut cleanup);
        cleanup.finish_with(error)
    }
}
impl Drop for ReactionPreparation<'_> {
    fn drop(&mut self) {
        if self.current.is_empty()
            && self.prepared.is_empty()
            && self.remaining.as_slice().is_empty()
        {
            return;
        }
        let mut cleanup = crate::job::UnwindCleanup::default();
        self.unavailable(&mut cleanup);
        if let Some(association) = self.association {
            cleanup.run(|| association.retire_pending(crate::job::JobRetirement::Interrupted));
        }
        self.cleanup(&mut cleanup);
        cleanup.finish();
    }
}

/// Owns every prepared tail and the previous source roots before any host call.
/// Native unwind disposes this custody without publishing another reaction.
struct ReactionBatch {
    jobs: std::vec::IntoIter<PreparedReactionJob>,
    pending: Option<StoredPromiseState>,
    first_error: Option<JsError>,
}
impl ReactionBatch {
    fn new(jobs: Vec<PreparedReactionJob>, pending: StoredPromiseState) -> Self {
        Self {
            jobs: jobs.into_iter(),
            pending: Some(pending),
            first_error: None,
        }
    }
    fn publish(mut self, context: &mut Context) -> JsResult<()> {
        for prepared in self.jobs.by_ref() {
            let mut publication = ReactionPublication::default();
            let registration = prepared.publish(&mut publication, context);
            let registration = publication.finish(registration);
            if let Err(error) = registration.into_result() {
                if self.first_error.is_none() {
                    self.first_error = Some(error);
                } else {
                    let mut cleanup = crate::job::UnwindCleanup::default();
                    cleanup.run(|| drop(error));
                    cleanup.finish();
                }
            }
        }
        let result = match self.first_error.take() {
            Some(error) => Err(error),
            None => Ok(()),
        };
        let mut cleanup = crate::job::UnwindCleanup::default();
        self.cleanup(&mut cleanup);
        cleanup.finish_with(result)
    }
    fn cleanup(&mut self, cleanup: &mut crate::job::UnwindCleanup) {
        for prepared in self.jobs.by_ref() {
            prepared.dispose(cleanup);
        }
        match self.pending.take() {
            Some(StoredPromiseState::Pending {
                fulfill_reactions,
                reject_reactions,
            }) => {
                for reaction in fulfill_reactions.into_iter().chain(reject_reactions) {
                    cleanup.run(|| drop(reaction));
                }
            }
            Some(StoredPromiseState::Fulfilled(value) | StoredPromiseState::Rejected(value)) => {
                cleanup.run(|| drop(value));
            }
            None => {}
        }
        let error = self.first_error.take();
        cleanup.run(|| drop(error));
    }
}
impl Drop for ReactionBatch {
    fn drop(&mut self) {
        let mut cleanup = crate::job::UnwindCleanup::default();
        self.cleanup(&mut cleanup);
        cleanup.finish();
    }
}

fn new_promise_reaction_job(
    preparation: &mut ReactionJobPreparation,
    argument: JsValue,
    context: &mut Context,
) -> JsResult<()> {
    if preparation.prepared.is_some() {
        return Err(reaction_state_error());
    }
    let reaction = preparation.live.as_ref().ok_or_else(reaction_state_error)?;
    // Inverting order since `job` captures `reaction` by value.

    // 2. Let handlerRealm be null.
    // 3. If reaction.[[Handler]] is not empty, then
    //   a. Let getHandlerRealmResult be Completion(GetFunctionRealm(reaction.[[Handler]].[[Callback]])).
    //   b. If getHandlerRealmResult is a normal completion, set handlerRealm to getHandlerRealmResult.[[Value]].
    //   c. Else, set handlerRealm to the current Realm Record.
    //   d. NOTE: handlerRealm is never null unless the handler is undefined. When the handler is a
    // revoked Proxy and no ECMAScript code runs, handlerRealm is used to create error objects.
    let realm = reaction
        .handler
        .as_ref()
        .and_then(|handler| handler.callback().get_function_realm(context).ok())
        .unwrap_or_else(|| context.realm().clone());

    // 1. Let job be a new Job Abstract Closure with no parameters that captures reaction and argument and performs the following steps when called:
    let hold = reaction.lifecycle.hold()?;
    let lifecycle_owner = hold.job_owner();
    let native_result_target = reaction.native_result_target.clone();
    let native_target = reaction.native_target.clone();
    let mut reaction = preparation.live.take().ok_or_else(reaction_state_error)?;
    let job = move |context: crate::ContextAccess<'_>| {
        let context = context.into_owner();
        //   a. Let promiseCapability be reaction.[[Capability]].
        let promise_capability = reaction.promise_capability.take();
        //   b. Let type be reaction.[[Type]].
        let reaction_type = reaction.reaction_type;
        //   c. Let handler be reaction.[[Handler]].
        let handler = reaction.handler.take();

        let handler_result = match handler {
            // d. If handler is empty, then
            None => match reaction_type {
                // i. If type is Fulfill, let handlerResult be NormalCompletion(argument).
                ReactionType::Fulfill => Ok(argument.clone()),
                // ii. Else,
                //   1. Assert: type is Reject.
                ReactionType::Reject => {
                    // 2. Let handlerResult be ThrowCompletion(argument).
                    Err(JsError::from_opaque(argument.clone()))
                }
            },
            //   e. Else, let handlerResult be Completion(HostCallJobCallback(handler, undefined, « argument »)).
            Some(handler) => handler.call(
                &JsValue::undefined(),
                std::slice::from_ref(&argument),
                context,
            ),
        };

        context.check_execution_stop()?;
        match promise_capability {
            None => {
                // f. If promiseCapability is undefined, return empty for a normal completion.
                // Host custody errors still propagate to the job executor.
                handler_result.map(|_| JsValue::undefined())
            }
            Some(promise_capability_record) => {
                // g. Assert: promiseCapability is a PromiseCapability Record.
                let PromiseCapability {
                    promise: _,
                    functions: ResolvingFunctions { resolve, reject },
                } = &promise_capability_record;

                match handler_result {
                    // h. If handlerResult is an abrupt completion, then
                    Err(error) => {
                        // i. Return ? Call(promiseCapability.[[Reject]], undefined, « handlerResult.[[Value]] »).
                        let value = error.into_opaque(context.access())?;
                        reject.call(&JsValue::undefined(), &[value], context.access())
                    }

                    // i. Else,
                    Ok(value) => {
                        // i. Return ? Call(promiseCapability.[[Resolve]], undefined, « handlerResult.[[Value]] »).
                        resolve.call(&JsValue::undefined(), &[value], context.access())
                    }
                }
            }
        }
    };

    // 4. Return the Record { [[Job]]: job, [[Realm]]: handlerRealm }.
    let job = PromiseJob::with_realm(job, realm).with_reaction_lifecycle(lifecycle_owner);
    preparation.prepared = Some(PreparedReactionJob {
        job,
        handoffs: [None, None],
        notifications: [None, None],
        hold,
    });
    let prepared = preparation
        .prepared
        .as_mut()
        .ok_or_else(reaction_state_error)?;
    for (slot, target) in prepared
        .handoffs
        .iter_mut()
        .zip([native_target, native_result_target])
    {
        if let Some(target) = target {
            *slot = Promise::prepare_native_handoff(&target, &prepared.job.observer())?;
        }
    }
    Ok(())
}

/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-newpromiseresolvethenablejob
fn new_promise_resolve_thenable_job(
    promise_to_resolve: JsObject<Promise>,
    thenable: JsValue,
    then: JobCallback,
    context: &mut Context,
) -> JsResult<(PromiseJob, Option<native_settlement::PreparedHandoff>)> {
    // This job owns pending settlement even before a native bridge adopts its
    // receiver. Prepare custody now so later observers inherit the real job.
    Promise::ensure_native_association(&promise_to_resolve, context)?;

    // Inverting order since `job` captures variables by value.

    // 2. Let getThenRealmResult be Completion(GetFunctionRealm(then.[[Callback]])).
    // 3. If getThenRealmResult is a normal completion, let thenRealm be getThenRealmResult.[[Value]].
    // 4. Else, let thenRealm be the current Realm Record.
    // 5. NOTE: thenRealm is never null. When then.[[Callback]] is a revoked Proxy and no code runs, thenRealm is used to create error objects.
    let realm = then
        .callback()
        .get_function_realm(context)
        .unwrap_or_else(|_| context.realm().clone());

    // 1. Let job be a new Job Abstract Closure with no parameters that captures promiseToResolve, thenable, and then and performs the following steps when called:
    let native_target = promise_to_resolve.clone();
    let job = move |context: crate::ContextAccess<'_>| {
        let context = context.into_owner();
        //    a. Let resolvingFunctions be CreateResolvingFunctions(promiseToResolve).
        let resolving_functions = Promise::create_resolving_functions(&promise_to_resolve, context);

        //    b. Let thenCallResult be Completion(HostCallJobCallback(then, thenable, « resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]] »)).
        let then_call_result = then.call(
            &thenable,
            &[
                resolving_functions.resolve.clone().into(),
                resolving_functions.reject.clone().into(),
            ],
            context,
        );

        context.check_execution_stop()?;
        //    c. If thenCallResult is an abrupt completion, then
        if let Err(value) = then_call_result {
            let value = value.into_opaque(context.access())?;
            //    i. Return ? Call(resolvingFunctions.[[Reject]], undefined, « thenCallResult.[[Value]] »).
            return resolving_functions.reject.call(
                &JsValue::undefined(),
                &[value],
                context.access(),
            );
        }

        //    d. Return ? thenCallResult.
        then_call_result
    };

    // 6. Return the Record { [[Job]]: job, [[Realm]]: thenRealm }.
    let job = PromiseJob::with_realm(job, realm);
    let handoff = Promise::prepare_native_handoff(&native_target, &job.observer())?;
    Ok((job, handoff))
}

#[cfg(feature = "experimental")]
/// Variant for the `PerformPromiseAllKeyed` algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyedVariant {
    /// `Promise.allKeyed` — resolves when all promises resolve.
    All,
    /// `Promise.allSettledKeyed` — resolves when all promises settle.
    AllSettled,
}

#[cfg(feature = "experimental")]
/// `CreateKeyedPromiseCombinatorResultObject ( keys, values )`
///
/// Creates a null-prototype object with data properties mapping keys to values.
///
/// More information:
///  - [TC39 proposal spec][spec]
///
/// [spec]: https://tc39.es/proposal-await-dictionary/#sec-createkeyedpromisecombinatorresultobject
fn create_keyed_result_object(
    keys: &[PropertyKey],
    values: &[StoredValue],
    context: &mut Context,
) -> JsObject {
    // 1. Assert: The number of elements in keys is the same as the number of elements in values.
    debug_assert_eq!(keys.len(), values.len());

    // 2. Let obj be OrdinaryObjectCreate(null).
    let obj = JsObject::with_null_proto(context.runtime());

    // 3. For each integer i such that 0 ≤ i < the number of elements in keys, in ascending order, do
    for (key, value) in keys.iter().zip(values.iter()) {
        // a. Perform ! CreateDataPropertyOrThrow(obj, keys[i], values[i]).
        obj.create_data_property_or_throw(key.clone(), value.to_value(), context.access())
            .expect("cannot fail per spec");
    }

    // 4. Return obj.
    obj
}
