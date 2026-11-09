use crate::{
    Context, JsValue,
    builtins::iterable::IteratorRecord,
    error::RootingError,
    native_function::{CoroutineBranch, CoroutineState, NativeCoroutine},
};

/// [`Iterator.prototype.take(limit)`][spec]
///
/// Takes `limit` values from the underlying iterator, and drops the rest.
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.take
pub(crate) struct Take;

impl Take {
    #[allow(
        clippy::new_ret_no_self,
        reason = "slightly cleaner to have this be a `new` method"
    )]
    pub(crate) fn new(
        iterated: IteratorRecord,
        limit: Option<u64>,
        context: &Context,
    ) -> Result<NativeCoroutine, RootingError> {
        // 10. Let closure be a new Abstract Closure with no parameters that
        //     captures iterated and integerLimit and performs the following steps
        //     when called:
        NativeCoroutine::from_fn_with_state(
            // a. Let remaining be integerLimit.
            // b. Repeat,
            |completion, (), prepared, state, context| {
                let Some((mut iterated, remaining)) = prepared else {
                    return CoroutineState::Break(Ok(()));
                };
                // vi. IfAbruptCloseIterator(completion, iterated).
                iterated.if_abrupt_close_iterator(completion, context)?;

                let remaining = if let Some(remaining) = remaining {
                    // i. If remaining = 0, return ? IteratorClose(iterated, ReturnCompletion(undefined)).
                    // ii. If remaining ≠ +∞, set remaining to remaining - 1.
                    let Some(remaining) = remaining.checked_sub(1) else {
                        iterated.close(Ok(JsValue::undefined()), context).branch()?;
                        return CoroutineState::Break(Ok(()));
                    };
                    Some(remaining)
                } else {
                    None
                };

                // iii. Let value be ? IteratorStepValue(iterated).
                match iterated.step_value(context).branch()? {
                    Some(value) => {
                        state.set((iterated, remaining)).branch()?;

                        // v. Let completion be Completion(Yield(value)).
                        CoroutineState::Continue(value)
                    }
                    // iv. If value is done, return ReturnCompletion(undefined).
                    None => CoroutineState::Break(Ok(())),
                }
            },
            (),
            (iterated, limit),
            context,
        )
    }
}
