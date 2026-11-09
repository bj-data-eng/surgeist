use surgeist_js_gc::heap::Trace;

use super::Array;
use crate::builtins::AsyncFromSyncIterator;
use crate::builtins::iterable::{IteratorRecord, StoredIteratorRecord};
use crate::builtins::promise::ResolvingFunctions;
use crate::error::{PanicError, RootingError, StoredJsError};
use crate::native_function::{
    CapturedObject, CapturedValue, CoroutineBranch, CoroutineSlot, CoroutineState, NativeCapture,
    NativeCoroutine,
};
use crate::object::{JsFunction, JsPromise};
use crate::vm::CompletionRecord;
use crate::{
    Context, JsArgs, JsError, JsExpect, JsNativeError, JsObject, JsResult, JsSymbol, JsValue,
    js_string,
};

impl Array {
    /// [`Array.fromAsync ( asyncItems [ , mapfn [ , thisArg ] ] )`][spec]
    ///
    /// The `Array.fromAsync()` static method creates a new,
    /// shallow-copied Array instance from a list or iterator of Promise-like values.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/proposal-array-from-async/#sec-array.fromAsync
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn from_async(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let C be the this value.
        // 2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
        let (promise, resolvers) = JsPromise::new_pending(context.access());

        let async_items = args.get_or_undefined(0);
        let mapfn = args.get_or_undefined(1);
        let this_arg = args.get_or_undefined(2).clone();

        // 3. Let fromAsyncClosure be a new Abstract Closure with no parameters that captures C, mapfn, and thisArg and
        //    performs the following steps when called:
        // 4. Perform AsyncFunctionStart(promiseCapability, fromAsyncClosure).
        // NOTE: We avoid putting more state onto the coroutines by preprocessing all we can before allocating
        //       the coroutines.
        let result: JsResult<()> = (|| {
            // a. If mapfn is undefined, let mapping be false.
            let mapfn = if mapfn.is_undefined() {
                None
            } else {
                // b. Else,
                //     i. If IsCallable(mapfn) is false, throw a TypeError exception.
                let Some(callable) = mapfn.as_callable() else {
                    return Err(JsNativeError::typ()
                        .with_message("Array.fromAsync: mapping function must be callable")
                        .into());
                };
                //     ii. Let mapping be true.
                Some(JsFunction::from_object_unchecked(callable))
            };

            // c. Let usingAsyncIterator be ? GetMethod(asyncItems, @@asyncIterator).
            // d. If usingAsyncIterator is undefined, then
            //     i. Let usingSyncIterator be ? GetMethod(asyncItems, @@iterator).
            // e. Let iteratorRecord be undefined.
            // f. If usingAsyncIterator is not undefined, then
            let iterator_record = if let Some(method) =
                async_items.get_method(JsSymbol::async_iterator(), context)?
            {
                // i. Set iteratorRecord to ? GetIterator(asyncItems, async, usingAsyncIterator).
                async_items.get_iterator_from_method(&method, context.access())?
            }
            // g. Else if usingSyncIterator is not undefined, then
            else if let Some(method) = async_items.get_method(JsSymbol::iterator(), context)? {
                // i. Set iteratorRecord to ? CreateAsyncFromSyncIterator(GetIterator(asyncItems, sync, usingSyncIterator)).
                AsyncFromSyncIterator::create(
                    async_items.get_iterator_from_method(&method, context.access())?,
                    context,
                )
            }
            // i. Else,
            else {
                // i. NOTE: asyncItems is neither an AsyncIterable nor an Iterable so assume it is an array-like object.
                // ii. Let arrayLike be ! ToObject(asyncItems).
                let array_like = async_items.to_object(context.access())?;

                // iii. Let len be ? LengthOfArrayLike(arrayLike).
                let len = array_like.length_of_array_like(context)?;
                // iv. If IsConstructor(C) is true, then
                let a = if let Some(c) = this.as_constructor() {
                    // 1. Let A be ? Construct(C, « 𝔽(len) »).
                    c.construct(&[len.into()], None, context.access())?
                }
                // v. Else,
                else {
                    // 1. Let A be ? ArrayCreate(len).
                    Array::array_create(len, None, context)?
                };

                let coroutine = NativeCoroutine::from_fn_with_state(
                    from_array_like,
                    GlobalState {
                        mapfn,
                        this_arg,
                        resolvers: resolvers.clone(),
                    },
                    ArrayLikeStateMachine::LoopStart {
                        array_like,
                        a,
                        len,
                        // iii. Let k be 0.
                        k: 0,
                    },
                    context,
                )?;

                return match coroutine.call(
                    CompletionRecord::Normal(JsValue::undefined()),
                    || {},
                    context,
                )? {
                    CoroutineState::Continue(value) => JsPromise::resolve(value, context.access())?
                        .await_native(coroutine, &resolvers.reject, context),
                    CoroutineState::Break(Err(err)) => Err(err),
                    CoroutineState::Break(Ok(())) => Ok(()),
                };
            };

            // h. If iteratorRecord is not undefined, then

            // i. If IsConstructor(C) is true, then
            let a = if let Some(c) = this.as_constructor() {
                // 1. Let A be ? Construct(C).
                c.construct(&[], None, context.access())?
            }
            // ii. Else,
            else {
                // 1. Let A be ! ArrayCreate(0).
                Array::array_create(0, None, context)?
            };

            let coroutine = NativeCoroutine::from_fn_with_state(
                from_async_iterator,
                GlobalState {
                    mapfn,
                    this_arg,
                    resolvers: resolvers.clone(),
                },
                AsyncIteratorStateMachine::LoopStart {
                    // vi. Let k be 0.
                    k: 0,
                    a,
                    iterator_record,
                },
                context,
            )?;

            match coroutine.call(
                CompletionRecord::Normal(JsValue::undefined()),
                || {},
                context,
            )? {
                CoroutineState::Continue(value) => JsPromise::resolve(value, context.access())?
                    .await_native(coroutine, &resolvers.reject, context),
                CoroutineState::Break(Err(err)) => Err(err),
                CoroutineState::Break(Ok(())) => Ok(()),
            }
        })();

        // AsyncFunctionStart ( promiseCapability, asyncFunctionBody )
        // https://tc39.es/ecma262/#sec-async-functions-abstract-operations-async-function-start
        // ->
        // AsyncBlockStart ( promiseCapability, asyncBody, asyncContext )
        // https://tc39.es/ecma262/#sec-asyncblockstart

        // i. Assert: result is a throw completion.
        if let Err(err) = result {
            // ii. Perform ! Call(promiseCapability.[[Reject]], undefined, « result.[[Value]] »).
            resolvers
                .reject
                .call(
                    &JsValue::undefined(),
                    &[err.into_opaque(context.access())?],
                    context.access(),
                )
                .js_expect("resolving functions cannot fail")?;
        }

        // 5. Return promiseCapability.[[Promise]].
        Ok(promise.into())
    }
}

struct GlobalState {
    mapfn: Option<JsFunction>,
    this_arg: JsValue,
    resolvers: ResolvingFunctions,
}

impl NativeCapture for GlobalState {
    type Stored = (
        Option<CapturedObject>,
        CapturedValue,
        CapturedObject,
        CapturedObject,
    );

    fn store(&self) -> Self::Stored {
        (
            self.mapfn.store(),
            self.this_arg.store(),
            self.resolvers.resolve.store(),
            self.resolvers.reject.store(),
        )
    }

    fn restore((mapfn, this_arg, resolve, reject): &Self::Stored) -> Result<Self, RootingError> {
        Ok(Self {
            mapfn: Option::restore(mapfn)?,
            this_arg: JsValue::restore(this_arg)?,
            resolvers: ResolvingFunctions {
                resolve: JsFunction::restore(resolve)?,
                reject: JsFunction::restore(reject)?,
            },
        })
    }
}

#[derive(Trace)]
enum AsyncIteratorState<O, I, V, E> {
    LoopStart {
        a: O,
        k: u64,
        iterator_record: I,
    },
    LoopContinue {
        a: O,
        k: u64,
        iterator_record: I,
    },
    LoopEnd {
        a: O,
        k: u64,
        iterator_record: I,
        mapped_value: Option<V>,
    },
    AsyncIteratorCloseStart {
        err: E,
        iterator: O,
    },
    AsyncIteratorCloseEnd {
        err: E,
    },
}

type AsyncIteratorStateMachine = AsyncIteratorState<JsObject, IteratorRecord, JsValue, JsError>;
type StoredAsyncIteratorState =
    AsyncIteratorState<CapturedObject, StoredIteratorRecord, CapturedValue, StoredJsError>;

impl NativeCapture for AsyncIteratorStateMachine {
    type Stored = StoredAsyncIteratorState;

    fn store(&self) -> Self::Stored {
        match self {
            Self::LoopStart {
                a,
                k,
                iterator_record,
            } => StoredAsyncIteratorState::LoopStart {
                a: a.store(),
                k: *k,
                iterator_record: iterator_record.stored(),
            },
            Self::LoopContinue {
                a,
                k,
                iterator_record,
            } => StoredAsyncIteratorState::LoopContinue {
                a: a.store(),
                k: *k,
                iterator_record: iterator_record.stored(),
            },
            Self::LoopEnd {
                a,
                k,
                iterator_record,
                mapped_value,
            } => StoredAsyncIteratorState::LoopEnd {
                a: a.store(),
                k: *k,
                iterator_record: iterator_record.stored(),
                mapped_value: mapped_value.store(),
            },
            Self::AsyncIteratorCloseStart { err, iterator } => {
                StoredAsyncIteratorState::AsyncIteratorCloseStart {
                    err: err.clone().into_stored(),
                    iterator: iterator.store(),
                }
            }
            Self::AsyncIteratorCloseEnd { err } => {
                StoredAsyncIteratorState::AsyncIteratorCloseEnd {
                    err: err.clone().into_stored(),
                }
            }
        }
    }

    fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
        Ok(match stored {
            StoredAsyncIteratorState::LoopStart {
                a,
                k,
                iterator_record,
            } => Self::LoopStart {
                a: JsObject::restore(a)?,
                k: *k,
                iterator_record: iterator_record.to_iterator()?,
            },
            StoredAsyncIteratorState::LoopContinue {
                a,
                k,
                iterator_record,
            } => Self::LoopContinue {
                a: JsObject::restore(a)?,
                k: *k,
                iterator_record: iterator_record.to_iterator()?,
            },
            StoredAsyncIteratorState::LoopEnd {
                a,
                k,
                iterator_record,
                mapped_value,
            } => Self::LoopEnd {
                a: JsObject::restore(a)?,
                k: *k,
                iterator_record: iterator_record.to_iterator()?,
                mapped_value: Option::restore(mapped_value)?,
            },
            StoredAsyncIteratorState::AsyncIteratorCloseStart { err, iterator } => {
                Self::AsyncIteratorCloseStart {
                    err: err.to_error()?,
                    iterator: JsObject::restore(iterator)?,
                }
            }
            StoredAsyncIteratorState::AsyncIteratorCloseEnd { err } => {
                Self::AsyncIteratorCloseEnd {
                    err: err.to_error()?,
                }
            }
        })
    }
}

/// Part of [`Array.fromAsync ( asyncItems [ , mapfn [ , thisArg ] ] )`](https://tc39.es/proposal-array-from-async/#sec-array.fromAsync).
fn from_async_iterator(
    mut result: CompletionRecord,
    global_state: &GlobalState,
    prepared: Option<AsyncIteratorStateMachine>,
    state_machine: &CoroutineSlot<AsyncIteratorStateMachine>,
    context: &mut Context,
) -> CoroutineState {
    let result = (|| {
        let Some(mut sm) = prepared else {
            return CoroutineState::Break(Ok(()));
        };

        // iv. Repeat,
        loop {
            match sm {
                AsyncIteratorStateMachine::LoopStart {
                    a,
                    k,
                    iterator_record,
                } => {
                    // Inverted conditional makes for a simpler code.
                    if k < 2u64.pow(53) - 1 {
                        // 2. Let Pk be ! ToString(𝔽(k)).
                        // 3. Let nextResult be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]]).
                        let next_result = iterator_record
                            .next_method()
                            .call(&iterator_record.iterator().clone().into(), &[], context)
                            .branch()?;

                        state_machine
                            .set(AsyncIteratorStateMachine::LoopContinue {
                                a,
                                k,
                                iterator_record,
                            })
                            .branch()?;

                        // 4. Set nextResult to ? Await(nextResult).
                        return CoroutineState::Continue(next_result);
                    }

                    // 1. If k ≥ 2**53 - 1, then

                    // a. Let error be ThrowCompletion(a newly created TypeError object).
                    // b. Return ? AsyncIteratorClose(iteratorRecord, error).
                    sm = AsyncIteratorStateMachine::AsyncIteratorCloseStart {
                        err: JsNativeError::typ()
                            .with_message(
                                "Array.fromAsync: \
                                            reached the maximum number of elements in an array \
                                            (2^53 - 1)",
                            )
                            .into(),
                        iterator: iterator_record.iterator().clone(),
                    };
                }
                AsyncIteratorStateMachine::LoopContinue {
                    a,
                    k,
                    mut iterator_record,
                } => {
                    // `result` is `Await(nextResult)`.
                    let result = std::mem::replace(
                        &mut result,
                        CompletionRecord::Normal(JsValue::undefined()),
                    );

                    // 5. If nextResult is not an Object, throw a TypeError exception.
                    // Implicit on the call to `update_result`.
                    iterator_record
                        .update_result(result.branch()?, context)
                        .branch()?;

                    // 6. Let done be ? IteratorComplete(nextResult).
                    // 7. If done is true,
                    if iterator_record.done() {
                        // a. Perform ? Set(A, "length", 𝔽(k), true).
                        a.set(js_string!("length"), k, true, context.access())
                            .branch()?;

                        // b. Return Completion Record { [[Type]]: return, [[Value]]: A, [[Target]]: empty }.
                        // AsyncFunctionStart ( promiseCapability, asyncFunctionBody )
                        // https://tc39.es/ecma262/#sec-async-functions-abstract-operations-async-function-start
                        // ->
                        // AsyncBlockStart ( promiseCapability, asyncBody, asyncContext )
                        // https://tc39.es/ecma262/#sec-asyncblockstart

                        // g. Else if result is a return completion, then
                        //        i. Perform ! Call(promiseCapability.[[Resolve]], undefined, « result.[[Value]] »).
                        global_state
                            .resolvers
                            .resolve
                            .call(&JsValue::undefined(), &[a.into()], context.access())
                            .js_expect("resolving functions cannot fail")
                            .branch()?;

                        return CoroutineState::Break(Ok(()));
                    }

                    // 8. Let nextValue be ? IteratorValue(nextResult).
                    let next_value = iterator_record.value(context).branch()?;
                    // 9. If mapping is true, then
                    if let Some(mapfn) = &global_state.mapfn {
                        // a. Let mappedValue be Call(mapfn, thisArg, « nextValue, 𝔽(k) »).
                        // b. IfAbruptCloseAsyncIterator(mappedValue, iteratorRecord).
                        // https://tc39.es/proposal-array-from-async/#sec-ifabruptcloseasynciterator
                        let mapped_value = match mapfn.call(
                            &global_state.this_arg,
                            &[next_value, k.into()],
                            context.access(),
                        ) {
                            // 1. If value is an abrupt completion, then
                            Err(err) => {
                                // a. Perform ? AsyncIteratorClose(iteratorRecord, value).
                                // b. Return value.
                                sm = AsyncIteratorStateMachine::AsyncIteratorCloseStart {
                                    err,
                                    iterator: iterator_record.iterator().clone(),
                                };
                                continue;
                            }
                            // 2. Else if value is a Completion Record, set value to value.[[Value]].
                            Ok(value) => value,
                        };
                        state_machine
                            .set(AsyncIteratorStateMachine::LoopEnd {
                                a,
                                k,
                                iterator_record,
                                mapped_value: None,
                            })
                            .branch()?;
                        // c. Set mappedValue to Await(mappedValue).
                        return CoroutineState::Continue(mapped_value);
                    }

                    sm = AsyncIteratorStateMachine::LoopEnd {
                        a,
                        k,
                        iterator_record,
                        // 10. Else, let mappedValue be nextValue.
                        mapped_value: Some(next_value),
                    }
                }
                AsyncIteratorStateMachine::LoopEnd {
                    a,
                    k,
                    iterator_record,
                    mapped_value,
                } => {
                    // Either awaited `mappedValue` or directly set `mappedValue` to `nextValue`.
                    let result = match std::mem::replace(
                        &mut result,
                        CompletionRecord::Normal(JsValue::undefined()),
                    ) {
                        CompletionRecord::Normal(val) => Ok(val),
                        CompletionRecord::Throw(err) => Err(err),
                        CompletionRecord::Return(_) => {
                            // TODO: this seems to be correct, but maybe we can
                            // cleanup this code to also handle return completions
                            return CoroutineState::Break(Err(PanicError::new(
                                "from_async_iterator cannot resume with a return completion",
                            )
                            .into()));
                        }
                    };

                    // d. IfAbruptCloseAsyncIterator(mappedValue, iteratorRecord).
                    // https://tc39.es/proposal-array-from-async/#sec-ifabruptcloseasynciterator
                    let mapped_value = match mapped_value.map(Ok).unwrap_or(result) {
                        // 1. If value is an abrupt completion, then
                        Err(err) => {
                            // a. Perform ? AsyncIteratorClose(iteratorRecord, value).
                            // b. Return value.
                            sm = AsyncIteratorStateMachine::AsyncIteratorCloseStart {
                                err,
                                iterator: iterator_record.iterator().clone(),
                            };
                            continue;
                        }
                        // 2. Else if value is a Completion Record, set value to value.[[Value]].
                        Ok(value) => value,
                    };

                    // 11. Let defineStatus be CreateDataPropertyOrThrow(A, Pk, mappedValue).
                    sm = if let Err(err) =
                        a.create_data_property_or_throw(k, mapped_value, context.access())
                    {
                        // 12. If defineStatus is an abrupt completion, return ? AsyncIteratorClose(iteratorRecord, defineStatus).
                        AsyncIteratorStateMachine::AsyncIteratorCloseStart {
                            err,
                            iterator: iterator_record.iterator().clone(),
                        }
                    } else {
                        AsyncIteratorStateMachine::LoopStart {
                            a,
                            // 13. Set k to k + 1.
                            k: k + 1,
                            iterator_record,
                        }
                    };
                }
                // AsyncIteratorClose ( iteratorRecord, completion )
                // https://tc39.es/ecma262/#sec-asynciteratorclose
                // Simplified for only error completions.
                AsyncIteratorStateMachine::AsyncIteratorCloseStart { err, iterator } => {
                    // 1. Assert: iteratorRecord.[[Iterator]] is an Object.
                    // 2. Let iterator be iteratorRecord.[[Iterator]].
                    // 3. Let innerResult be Completion(GetMethod(iterator, "return")).
                    // 4. If innerResult is a normal completion, then
                    //     a. Let return be innerResult.[[Value]].
                    //     b. If return is undefined, return ? completion.
                    //     c. Set innerResult to Completion(Call(return, iterator)).
                    //     d. If innerResult is a normal completion, set innerResult to Completion(Await(innerResult.[[Value]])).
                    // 5. If completion is a throw completion, return ? completion.
                    let Ok(Some(ret)) = iterator.get_method(js_string!("return"), context) else {
                        return CoroutineState::Break(Err(err));
                    };

                    let Ok(value) = ret.call(&iterator.into(), &[], context.access()) else {
                        return CoroutineState::Break(Err(err));
                    };

                    let original_engine = err.as_engine().is_some().then(|| err.clone());
                    if let Err(error) =
                        state_machine.set(AsyncIteratorStateMachine::AsyncIteratorCloseEnd { err })
                    {
                        return CoroutineState::Break(Err(
                            original_engine.unwrap_or_else(|| error.into())
                        ));
                    }
                    return CoroutineState::Continue(value);
                }
                AsyncIteratorStateMachine::AsyncIteratorCloseEnd { err } => {
                    // Awaited `innerResult.[[Value]]`.
                    // Only need to return the original error.
                    return CoroutineState::Break(Err(err));
                }
            }
        }
    })();

    // AsyncFunctionStart ( promiseCapability, asyncFunctionBody )
    // https://tc39.es/ecma262/#sec-async-functions-abstract-operations-async-function-start
    // ->
    // AsyncBlockStart ( promiseCapability, asyncBody, asyncContext )
    // https://tc39.es/ecma262/#sec-asyncblockstart
    match result {
        // i. Assert: result is a throw completion.
        CoroutineState::Break(Err(err)) => {
            // ii. Perform ! Call(promiseCapability.[[Reject]], undefined, « result.[[Value]] »).
            global_state
                .resolvers
                .reject
                .call(
                    &JsValue::undefined(),
                    &[err.into_opaque(context.access()).branch()?],
                    context.access(),
                )
                .js_expect("resolving functions cannot fail")
                .branch()?;
            CoroutineState::Break(Ok(()))
        }
        result => result,
    }
}

#[derive(Trace)]
#[allow(clippy::enum_variant_names)]
enum ArrayLikeState<O, V> {
    LoopStart {
        array_like: O,
        a: O,
        len: u64,
        k: u64,
    },
    LoopContinue {
        array_like: O,
        a: O,
        len: u64,
        k: u64,
    },
    LoopEnd {
        array_like: O,
        a: O,
        len: u64,
        k: u64,
        mapped_value: Option<V>,
    },
}

type ArrayLikeStateMachine = ArrayLikeState<JsObject, JsValue>;
type StoredArrayLikeState = ArrayLikeState<CapturedObject, CapturedValue>;

impl NativeCapture for ArrayLikeStateMachine {
    type Stored = StoredArrayLikeState;

    fn store(&self) -> Self::Stored {
        match self {
            Self::LoopStart {
                array_like,
                a,
                len,
                k,
            } => StoredArrayLikeState::LoopStart {
                array_like: array_like.store(),
                a: a.store(),
                len: *len,
                k: *k,
            },
            Self::LoopContinue {
                array_like,
                a,
                len,
                k,
            } => StoredArrayLikeState::LoopContinue {
                array_like: array_like.store(),
                a: a.store(),
                len: *len,
                k: *k,
            },
            Self::LoopEnd {
                array_like,
                a,
                len,
                k,
                mapped_value,
            } => StoredArrayLikeState::LoopEnd {
                array_like: array_like.store(),
                a: a.store(),
                len: *len,
                k: *k,
                mapped_value: mapped_value.store(),
            },
        }
    }

    fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
        Ok(match stored {
            StoredArrayLikeState::LoopStart {
                array_like,
                a,
                len,
                k,
            } => Self::LoopStart {
                array_like: JsObject::restore(array_like)?,
                a: JsObject::restore(a)?,
                len: *len,
                k: *k,
            },
            StoredArrayLikeState::LoopContinue {
                array_like,
                a,
                len,
                k,
            } => Self::LoopContinue {
                array_like: JsObject::restore(array_like)?,
                a: JsObject::restore(a)?,
                len: *len,
                k: *k,
            },
            StoredArrayLikeState::LoopEnd {
                array_like,
                a,
                len,
                k,
                mapped_value,
            } => Self::LoopEnd {
                array_like: JsObject::restore(array_like)?,
                a: JsObject::restore(a)?,
                len: *len,
                k: *k,
                mapped_value: Option::restore(mapped_value)?,
            },
        })
    }
}

/// Part of [`Array.fromAsync ( asyncItems [ , mapfn [ , thisArg ] ] )`](https://tc39.es/proposal-array-from-async/#sec-array.fromAsync).
fn from_array_like(
    mut result: CompletionRecord,
    global_state: &GlobalState,
    prepared: Option<ArrayLikeStateMachine>,
    state_machine: &CoroutineSlot<ArrayLikeStateMachine>,
    context: &mut Context,
) -> CoroutineState {
    let result = (|| {
        let Some(mut sm) = prepared else {
            return CoroutineState::Break(Ok(()));
        };

        loop {
            match sm {
                ArrayLikeStateMachine::LoopStart {
                    array_like,
                    a,
                    len,
                    k,
                } => {
                    // vii. Repeat, while k < len,
                    if k >= len {
                        // viii. Perform ? Set(A, "length", 𝔽(len), true).
                        a.set(js_string!("length"), len, true, context.access())
                            .branch()?;

                        // ix. Return Completion Record { [[Type]]: return, [[Value]]: A, [[Target]]: empty }.

                        // AsyncFunctionStart ( promiseCapability, asyncFunctionBody )
                        // https://tc39.es/ecma262/#sec-async-functions-abstract-operations-async-function-start
                        // ->
                        // AsyncBlockStart ( promiseCapability, asyncBody, asyncContext )
                        // https://tc39.es/ecma262/#sec-asyncblockstart

                        // g. Else if result is a return completion, then
                        //        i. Perform ! Call(promiseCapability.[[Resolve]], undefined, « result.[[Value]] »).
                        global_state
                            .resolvers
                            .resolve
                            .call(&JsValue::undefined(), &[a.into()], context.access())
                            .js_expect("resolving functions cannot fail")
                            .branch()?;

                        return CoroutineState::Break(Ok(()));
                    }

                    // 1. Let Pk be ! ToString(𝔽(k)).
                    // 2. Let kValue be ? Get(arrayLike, Pk).
                    let k_value = array_like.get(k, context.access()).branch()?;
                    state_machine
                        .set(ArrayLikeStateMachine::LoopContinue {
                            array_like,
                            a,
                            len,
                            k,
                        })
                        .branch()?;

                    // 3. Set kValue to ? Await(kValue).
                    return CoroutineState::Continue(k_value);
                }
                ArrayLikeStateMachine::LoopContinue {
                    array_like,
                    a,
                    len,
                    k,
                } => {
                    // Awaited kValue
                    let k_value = std::mem::replace(
                        &mut result,
                        CompletionRecord::Normal(JsValue::undefined()),
                    )
                    .branch()?;

                    // 4. If mapping is true, then
                    if let Some(mapfn) = &global_state.mapfn {
                        // a. Let mappedValue be ? Call(mapfn, thisArg, « kValue, 𝔽(k) »).
                        let mapped_value = mapfn
                            .call(
                                &global_state.this_arg,
                                &[k_value, k.into()],
                                context.access(),
                            )
                            .branch()?;
                        state_machine
                            .set(ArrayLikeStateMachine::LoopEnd {
                                array_like,
                                a,
                                len,
                                k,
                                mapped_value: None,
                            })
                            .branch()?;

                        // b. Set mappedValue to ? Await(mappedValue).
                        return CoroutineState::Continue(mapped_value);
                    }
                    // 5. Else, let mappedValue be kValue.
                    sm = ArrayLikeStateMachine::LoopEnd {
                        array_like,
                        a,
                        len,
                        k,
                        mapped_value: Some(k_value),
                    }
                }
                ArrayLikeStateMachine::LoopEnd {
                    array_like,
                    a,
                    len,
                    k,
                    mapped_value,
                } => {
                    // Either awaited `mappedValue` or directly set this from `kValue`.
                    let result = std::mem::replace(
                        &mut result,
                        CompletionRecord::Normal(JsValue::undefined()),
                    )
                    .branch()?;
                    let mapped_value = mapped_value.unwrap_or(result);

                    // 6. Perform ? CreateDataPropertyOrThrow(A, Pk, mappedValue).
                    a.create_data_property_or_throw(k, mapped_value, context.access())
                        .branch()?;

                    // 7. Set k to k + 1.
                    sm = ArrayLikeStateMachine::LoopStart {
                        array_like,
                        a,
                        len,
                        k: k + 1,
                    }
                }
            }
        }
    })();

    // AsyncFunctionStart ( promiseCapability, asyncFunctionBody )
    // https://tc39.es/ecma262/#sec-async-functions-abstract-operations-async-function-start
    // ->
    // AsyncBlockStart ( promiseCapability, asyncBody, asyncContext )
    // https://tc39.es/ecma262/#sec-asyncblockstart
    match result {
        // i. Assert: result is a throw completion.
        CoroutineState::Break(Err(err)) => {
            // ii. Perform ! Call(promiseCapability.[[Reject]], undefined, « result.[[Value]] »).
            global_state
                .resolvers
                .reject
                .call(
                    &JsValue::undefined(),
                    &[err.into_opaque(context.access()).branch()?],
                    context.access(),
                )
                .js_expect("resolving functions cannot fail")
                .branch()?;
            CoroutineState::Break(Ok(()))
        }
        result => result,
    }
}
