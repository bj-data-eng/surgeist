use crate::{
    Context, JsValue,
    builtins::iterable::IteratorRecord,
    error::RootingError,
    native_function::{CoroutineBranch, CoroutineState, NativeCoroutine},
    object::JsFunction,
};

/// [`Iterator.prototype.filter(predicate)`][spec]
///
/// Yields only values that satisfy `predicate(value, counter) == true`.
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.filter
pub(crate) struct Filter;

impl Filter {
    #[allow(
        clippy::new_ret_no_self,
        reason = "slightly cleaner to have this be a `new` method"
    )]
    pub(crate) fn new(
        iterated: IteratorRecord,
        predicate: JsFunction,
        context: &Context,
    ) -> Result<NativeCoroutine, RootingError> {
        // 6. Let closure be a new Abstract Closure with no parameters that captures
        //    iterated and predicate and performs the following steps when called:
        NativeCoroutine::from_fn_with_state(
            // a. Let counter be 0.
            // b. Repeat,
            |completion, (), prepared, state, context| {
                let Some((mut iterated, predicate, mut counter)) = prepared else {
                    return CoroutineState::Break(Ok(()));
                };
                // 2. IfAbruptCloseIterator(completion, iterated).
                iterated.if_abrupt_close_iterator(completion, context)?;

                loop {
                    // i. Let value be ? IteratorStepValue(iterated).
                    // ii. If value is done, return ReturnCompletion(undefined).
                    let Some(value) = iterated.step_value(context).branch()? else {
                        return CoroutineState::Break(Ok(()));
                    };

                    // iii. Let selected be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).
                    let selected = match predicate.call(
                        &JsValue::undefined(),
                        &[value.clone(), counter.into()],
                        context.access(),
                    ) {
                        Ok(val) => val,
                        Err(err) => {
                            // iv. IfAbruptCloseIterator(selected, iterated).
                            return iterated.close(Err(err), context).branch();
                        }
                    };

                    // vi. Set counter to counter + 1.
                    counter += 1;

                    // v. If ToBoolean(selected) is true, then
                    if selected.to_boolean() {
                        state.set((iterated, predicate, counter)).branch()?;
                        // 1. Let completion be Completion(Yield(value)).
                        return CoroutineState::Continue(value);
                    }
                }
            },
            (),
            (iterated, predicate, 0_u64),
            context,
        )
    }
}
