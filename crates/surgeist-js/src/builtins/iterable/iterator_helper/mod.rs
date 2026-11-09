//! Boa's implementation of the `Iterator Helper` objects.
//!
//! An Iterator Helper object is an ordinary object that represents a lazy transformation
//! of some specific source iterator object. There is not a named constructor for
//! Iterator Helper objects. Instead, Iterator Helper objects are created by calling
//! certain methods of Iterator instance objects.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-iterator-helper-objects

use std::ops::ControlFlow;

use crate::{
    Context, JsData, JsResult, JsValue,
    builtins::{BuiltInBuilder, IntrinsicObject, iterable::create_iter_result_object_trusted},
    context::intrinsics::Intrinsics,
    error::{PanicError, RootingError},
    js_error, js_string,
    native_function::{NativeCapture, NativeCoroutine, StoredNativeCoroutine},
    object::JsObject,
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
    vm::CompletionRecord,
};
use surgeist_js_gc::heap::{AccessError, Trace as HeapTrace};

/// `IfAbruptCloseIterator ( value, iteratorRecord )`, but specialized
/// for usage in coroutines.
///
/// More information:
///  - [ECMA reference][spec]
///
///  [spec]: https://tc39.es/ecma262/#sec-ifabruptcloseiterator
macro_rules! if_abrupt_close_iterator {
    ($value:expr, $iterator_record:expr, $context:expr) => {
        match $value {
            // 1. If value is an abrupt completion, return ? IteratorClose(iteratorRecord, value).
            Err(err) => {
                return $crate::native_function::CoroutineBranch::branch(
                    $iterator_record.close(Err(err), $context),
                )
            }
            // 2. Else if value is a Completion Record, set value to value.
            Ok(value) => value,
        }
    };
}

mod concat;
mod drop;
mod filter;
mod flat_map;
mod map;
mod take;

pub(crate) use concat::{Concat, IterableRecord};
pub(crate) use drop::Drop;
pub(crate) use filter::Filter;
pub(crate) use flat_map::FlatMap;
pub(crate) use map::Map;
pub(crate) use take::Take;

/// The internal representation of an `Iterator Helper` object.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator-helper-objects
#[derive(Debug, HeapTrace, JsData)]
pub(crate) struct IteratorHelper {
    state: HelperState,
}

#[derive(Debug, HeapTrace)]
enum HelperState {
    Suspended(StoredNativeCoroutine),
    Executing,
    Completed,
}

impl IntrinsicObject for IteratorHelper {
    fn init(realm: &Realm) {
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .prototype(realm.intrinsics().constructors().iterator().prototype())
            .static_method(Self::next, js_string!("next"), 0)
            .static_method(Self::r#return, js_string!("return"), 0)
            .static_property(
                JsSymbol::to_string_tag(),
                js_string!("Iterator Helper"),
                Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().iterator_prototypes().iterator_helper()
    }
}

impl IteratorHelper {
    fn prepare_resume(&self) -> JsResult<Option<NativeCoroutine>> {
        match &self.state {
            HelperState::Suspended(coroutine) => Ok(Some(NativeCoroutine::restore(coroutine)?)),
            HelperState::Executing => {
                Err(js_error!(TypeError: "Iterator Helper is already executing"))
            }
            HelperState::Completed => Ok(None),
        }
    }

    /// `%IteratorHelperPrototype%.next ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%25iteratorhelperprototype%25.next
    pub(crate) fn next(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Return ? GeneratorResume(this value, undefined, "Iterator Helper").

        // `GeneratorResume ( generator, value, generatorBrand )`
        // <https://tc39.es/ecma262/#sec-generatorresume>
        //
        // NOTE: This function might not map directly to the spec, since we're
        // converting generators into state machines

        // 1. Let state be ? GeneratorValidate(generator, generatorBrand).
        let helper = Self::generator_validate(this)?;

        // `GeneratorValidate ( generator, generatorBrand )`
        // <https://tc39.es/ecma262/#sec-generatorvalidate>
        //
        // 4. Assert: generator has a [[GeneratorContext]] internal slot.
        // 5. Let state be generator.[[GeneratorState]].
        // 6. If state is executing, throw a TypeError exception.
        // 7. Return state.
        let mut helper_guard = helper
            .try_borrow_mut()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        let coroutine = helper_guard.data().prepare_resume()?;
        let Some(coroutine) = coroutine else {
            drop(helper_guard);
            return Ok(create_iter_result_object_trusted(
                JsValue::undefined(),
                true,
                context,
            ));
        };

        // 3. Assert: state is either suspended-start or suspended-yield.
        // 4. Let genContext be generator.[[GeneratorContext]].
        // 5. Let methodContext be the running execution context.
        // 6. Suspend methodContext.
        // 7. Set generator.[[GeneratorState]] to executing.
        // 8. Push genContext onto the execution context stack; genContext is now
        //    the running execution context.
        // 9. Resume the suspended evaluation of genContext using NormalCompletion(value)
        //    as the result of the operation that suspended it. Let result be the
        //    value returned by the resumed computation.
        // 10. Assert: When we return here, genContext has already been removed
        //     from the execution context stack and methodContext is the currently
        //     running execution context.
        //
        // All these steps don't map directly to the spec, but we can consider
        // the code below as "suspending" the underlying generator or returning
        // if the result is available.
        let result = coroutine.call(
            CompletionRecord::Normal(JsValue::undefined()),
            move || {
                helper_guard.data_mut().state = HelperState::Executing;
                drop(helper_guard);
            },
            context,
        )?;
        helper
            .try_borrow_mut()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?
            .data_mut()
            .state = if matches!(&result, ControlFlow::Continue(_)) {
            HelperState::Suspended(coroutine.store())
        } else {
            HelperState::Completed
        };

        match result {
            ControlFlow::Continue(value) => {
                Ok(create_iter_result_object_trusted(value, false, context))
            }
            // 2. If state is completed, return CreateIteratorResultObject(undefined, true).
            ControlFlow::Break(Ok(())) => Ok(create_iter_result_object_trusted(
                JsValue::undefined(),
                true,
                context,
            )),
            ControlFlow::Break(Err(err)) => Err(err),
        }
    }

    /// `%IteratorHelperPrototype%.return ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%25iteratorhelperprototype%25.return
    pub(crate) fn r#return(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be this value.
        // 2. Perform ? RequireInternalSlot(O, [[UnderlyingIterators]]).
        // 3. Assert: O has a [[GeneratorState]] internal slot.
        // 4. If O.[[GeneratorState]] is suspended-start, then
        //        a. Set O.[[GeneratorState]] to completed.
        //        b. NOTE: Once a generator enters the completed state it never leaves it and its associated execution context is never resumed. Any execution state associated with O can be discarded at this point.
        //        c. Perform ? IteratorCloseAll(O.[[UnderlyingIterators]], NormalCompletion(unused)).
        //        d. Return CreateIteratorResultObject(undefined, true).
        // 5. Let C be ReturnCompletion(undefined).
        // 6. Return ? GeneratorResumeAbrupt(O, C, "Iterator Helper").

        // `GeneratorResumeAbrupt ( generator, abruptCompletion, generatorBrand )`
        // <https://tc39.es/ecma262/#sec-generatorresumeabrupt>
        //
        // NOTE: This function might not map directly to the spec, since we're
        // converting generators into state machines

        // 1. Let state be ? GeneratorValidate(generator, generatorBrand).
        let helper = Self::generator_validate(this)?;

        // `GeneratorValidate ( generator, generatorBrand )`
        // <https://tc39.es/ecma262/#sec-generatorvalidate>
        //
        // 4. Assert: generator has a [[GeneratorContext]] internal slot.
        // 5. Let state be generator.[[GeneratorState]].
        // 6. If state is executing, throw a TypeError exception.
        // 7. Return state.
        let mut helper_guard = helper
            .try_borrow_mut()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        let coroutine = helper_guard.data().prepare_resume()?;
        let Some(coroutine) = coroutine else {
            drop(helper_guard);
            return Ok(create_iter_result_object_trusted(
                JsValue::undefined(),
                true,
                context,
            ));
        };

        // 2. If state is suspended-start, then
        //    a. Set generator.[[GeneratorState]] to completed.
        //    b. NOTE: Once a generator enters the completed state it never leaves
        //       it and its associated execution context is never resumed. Any
        //       execution state associated with generator can be discarded at this point.
        //    c. Set state to completed.
        // 3. If state is completed, then
        //    a. If abruptCompletion is a return completion, then
        //       i. Return CreateIteratorResultObject(abruptCompletion.[[Value]], true).
        //    b. Return ? abruptCompletion.
        // 4. Assert: state is suspended-yield.
        // 5. Let genContext be generator.[[GeneratorContext]].
        // 6. Let methodContext be the running execution context.
        // 7. Suspend methodContext.
        // 8. Set generator.[[GeneratorState]] to executing.
        // 9. Push genContext onto the execution context stack; genContext is now
        //    the running execution context.
        // 10. Resume the suspended evaluation of genContext using
        //     abruptCompletion as the result of the operation that suspended it.
        //     Let result be the Completion Record returned by the resumed
        //     computation.
        // 11. Assert: When we return here, genContext has already been removed
        //     from the execution context stack and methodContext is the currently
        //     running execution context.
        //
        // ... Delegate status tracking to each transformer.
        let result = match coroutine.call(
            CompletionRecord::Return(JsValue::undefined()),
            move || {
                helper_guard.data_mut().state = HelperState::Executing;
                drop(helper_guard);
            },
            context,
        )? {
            // We technically would need to follow step 3.b here and return
            // the yielded value, but our transformers shouldn't keep executing
            // after calling `return`, so it's better to panic here to catch bugs.
            ControlFlow::Continue(_) => Err(PanicError::new(
                "an iterator helper cannot yield after a return request",
            )
            .into()),
            // Step 3.a
            ControlFlow::Break(Ok(())) => Ok(create_iter_result_object_trusted(
                JsValue::undefined(),
                true,
                context,
            )),
            // Step 3.b
            ControlFlow::Break(Err(err)) => Err(err),
        };

        helper
            .try_borrow_mut()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?
            .data_mut()
            .state = HelperState::Completed;

        // 12. Return ? result.
        result
    }

    /// [`GeneratorValidate ( generator, generatorBrand )`][spec]
    ///
    /// Validates that `this` is an iterator helper and that is
    /// not already executing.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generatorvalidate
    #[track_caller]
    pub(crate) fn generator_validate(this: &JsValue) -> JsResult<JsObject<IteratorHelper>> {
        // 1. Perform ? RequireInternalSlot(generator, [[GeneratorState]]).
        // 2. Perform ? RequireInternalSlot(generator, [[GeneratorBrand]]).
        // 3. If generator.[[GeneratorBrand]] is not generatorBrand, throw a TypeError exception.
        //
        // Slightly differs from the spec since we don't have a `[[GeneratorState]]`,
        // and we defer checking for `Executing` to `Self::next` and `Self::return`.
        this.as_object()
            .and_then(|o| o.downcast::<Self>().ok())
            .ok_or_else(|| js_error!(TypeError: "Iterator Helper method called on non-object"))
    }

    /// Creates a new `IteratorHelper` object.
    pub(crate) fn create(op: NativeCoroutine, context: &mut Context) -> JsObject {
        // All eager iterators follow the following steps:
        //
        // i. Let result be CreateIteratorFromClosure(
        //         closure, "Iterator Helper",
        //         %IteratorHelperPrototype%, « [[UnderlyingIterators]] »
        //     ).
        // ii. Set result.[[UnderlyingIterators]] to « iterated ».
        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .objects()
                .iterator_prototypes()
                .iterator_helper(),
            Self {
                state: HelperState::Suspended(op.store()),
            },
        )
        .upcast()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use surgeist_js_gc::heap::Rooted;

    use super::*;
    use crate::{Source, error::EngineError, native_function::CoroutineBranch};

    fn resume(returning: bool, helper: &JsValue, context: &mut Context) -> JsResult<JsValue> {
        if returning {
            IteratorHelper::r#return(helper, &[], context)
        } else {
            IteratorHelper::next(helper, &[], context)
        }
    }

    #[test]
    fn closed_map_helper_keeps_suspension_on_preparation_refusal_and_retries() {
        for returning in [false, true] {
            let mut context = Context::default();
            let helper = context
                .eval(Source::from_bytes(
                    r#"
                    globalThis.sourceSteps = 0;
                    globalThis.sourceCloses = 0;
                    globalThis.mapperCalls = 0;
                    function checkReentry() {
                        let refused = false;
                        try { preparedHelper.next(); }
                        catch (error) {
                            if (!(error instanceof TypeError)) throw error;
                            refused = true;
                        }
                        if (!refused) throw new Error("executing helper accepted reentry");
                    }
                    globalThis.preparedHelper = Iterator.prototype.map.call({
                        next() {
                            sourceSteps++;
                            return {value: 4, done: false};
                        },
                        return() {
                            sourceCloses++;
                            checkReentry();
                            return {done: true};
                        }
                    }, value => {
                        mapperCalls++;
                        checkReentry();
                        return value * 2;
                    });
                    preparedHelper
                    "#,
                ))
                .unwrap();
            let typed_helper = IteratorHelper::generator_validate(&helper).unwrap();
            let heap = context.runtime().heap.clone();
            let error = heap
                .read(|_| resume(returning, &helper, &mut context))
                .unwrap()
                .unwrap_err();
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::Rooting(RootingError::Access(
                    AccessError::HeapBorrowed
                )))
            );
            assert!(matches!(
                typed_helper.try_borrow().unwrap().data().state,
                HelperState::Suspended(_)
            ));
            assert_eq!(
                context
                    .eval(Source::from_bytes(
                        "sourceSteps + sourceCloses + mapperCalls"
                    ))
                    .unwrap(),
                JsValue::new(0)
            );

            let result = resume(returning, &helper, &mut context)
                .unwrap()
                .as_object()
                .unwrap();
            assert_eq!(
                result.get(js_string!("done"), context.access()).unwrap(),
                JsValue::new(returning)
            );
            if returning {
                assert!(matches!(
                    typed_helper.try_borrow().unwrap().data().state,
                    HelperState::Completed
                ));
                assert_eq!(
                    context.eval(Source::from_bytes("sourceCloses")).unwrap(),
                    JsValue::new(1)
                );
            } else {
                assert_eq!(
                    result.get(js_string!("value"), context.access()).unwrap(),
                    JsValue::new(8)
                );
                assert_eq!(
                    context
                        .eval(Source::from_bytes("sourceSteps + mapperCalls"))
                        .unwrap(),
                    JsValue::new(2)
                );
            }
        }
    }

    #[test]
    fn entered_body_payload_refusal_completes_helper_without_replaying_work() {
        let mut context = Context::default();
        let calls = context
            .runtime()
            .heap
            .allocate(RefCell::new(0_u32))
            .unwrap();
        let source = context
            .runtime()
            .heap
            .allocate(RefCell::new(9_u32))
            .unwrap();
        let coroutine = NativeCoroutine::from_fn_with_state(
            |_, calls: &Rooted<RefCell<u32>>, prepared, _, _| {
                *calls.borrow_mut() += 1;
                let state: Rooted<RefCell<u32>> = prepared.unwrap();
                let value = *state
                    .try_borrow()
                    .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))
                    .branch()?;
                ControlFlow::Continue(JsValue::new(value))
            },
            calls.clone(),
            source.clone(),
            &context,
        )
        .unwrap();
        let helper = IteratorHelper::create(coroutine, &mut context);
        let typed_helper = helper.clone().downcast::<IteratorHelper>().unwrap();
        let held_payload = source.borrow_mut();

        let error = IteratorHelper::next(&helper.clone().into(), &[], &mut context).unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::Rooting(RootingError::Access(
                AccessError::PayloadBorrowed
            )))
        );
        assert_eq!(*calls.borrow(), 1);
        assert!(matches!(
            typed_helper.try_borrow().unwrap().data().state,
            HelperState::Completed
        ));

        drop(held_payload);
        let result = IteratorHelper::next(&helper.into(), &[], &mut context)
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(
            result.get(js_string!("done"), context.access()).unwrap(),
            JsValue::new(true)
        );
        assert_eq!(*calls.borrow(), 1);
    }
}
