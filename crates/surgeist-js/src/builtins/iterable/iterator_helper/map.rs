use crate::{
    Context, JsValue,
    builtins::iterable::IteratorRecord,
    error::RootingError,
    native_function::{CoroutineBranch, CoroutineState, NativeCoroutine},
    object::JsFunction,
};

/// [`Iterator.prototype.map(mapper)`][spec]
///
/// Maps all values of the underlying iterator with `mapper`.
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.map
pub(crate) struct Map;

impl Map {
    #[allow(
        clippy::new_ret_no_self,
        reason = "slightly cleaner to have this be a `new` method"
    )]
    pub(crate) fn new(
        iterated: IteratorRecord,
        mapper: JsFunction,
        context: &Context,
    ) -> Result<NativeCoroutine, RootingError> {
        // 6. Let closure be a new Abstract Closure with no parameters that captures
        //    iterated and mapper and performs the following steps when called:
        NativeCoroutine::from_fn_with_state(
            // a. Let counter be 0.
            // b. Repeat,
            |completion, (), prepared, state, context| {
                let Some((mut iterated, mapper, counter)) = prepared else {
                    return CoroutineState::Break(Ok(()));
                };
                // vi. IfAbruptCloseIterator(completion, iterated).
                iterated.if_abrupt_close_iterator(completion, context)?;

                // i. Let value be ? IteratorStepValue(iterated).
                // ii. If value is done, return ReturnCompletion(undefined).
                let Some(value) = iterated.step_value(context).branch()? else {
                    return CoroutineState::Break(Ok(()));
                };

                // iii. Let mapped be Completion(Call(mapper, undefined, « value, 𝔽(counter) »)).
                // iv. IfAbruptCloseIterator(mapped, iterated).
                let value = if_abrupt_close_iterator!(
                    mapper.call(
                        &JsValue::undefined(),
                        &[value, counter.into()],
                        context.access(),
                    ),
                    iterated,
                    context
                );

                // vii. Set counter to counter + 1.
                state.set((iterated, mapper, counter + 1)).branch()?;
                // v. Let completion be Completion(Yield(mapped)).
                CoroutineState::Continue(value)
            },
            (),
            (iterated, mapper, 0_u64),
            context,
        )
    }
}
