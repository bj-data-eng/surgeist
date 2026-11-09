use crate::{
    Context,
    builtins::iterable::IteratorRecord,
    error::RootingError,
    native_function::{CoroutineBranch, CoroutineState, NativeCoroutine},
};

/// [`Iterator.prototype.drop(limit)`][spec]
///
/// Drops `limit` values from the underlying iterator, then yields the rest.
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.drop
pub(crate) struct Drop;

impl Drop {
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
            // c. Repeat,
            |completion, (), prepared, state, context| {
                let Some((mut iterated, limit)) = prepared else {
                    return CoroutineState::Break(Ok(()));
                };
                // iv. IfAbruptCloseIterator(completion, iterated).
                iterated.if_abrupt_close_iterator(completion, context)?;

                let Some(mut limit) = limit else {
                    // Drop all remaining values for a limit of +∞.
                    while !iterated.step(context).branch()? {}
                    return CoroutineState::Break(Ok(()));
                };
                // b. Repeat, while remaining > 0.
                while let Some(remaining) = limit.checked_sub(1) {
                    limit = remaining;
                    if iterated.step(context).branch()? {
                        return CoroutineState::Break(Ok(()));
                    }
                }

                // i. Let value be ? IteratorStepValue(iterated).
                match iterated.step_value(context).branch()? {
                    Some(value) => {
                        state.set((iterated, Some(0))).branch()?;
                        // iii. Let completion be Completion(Yield(value)).
                        CoroutineState::Continue(value)
                    }
                    // ii. If value is done, return ReturnCompletion(undefined).
                    None => CoroutineState::Break(Ok(())),
                }
            },
            (),
            (iterated, limit),
            context,
        )
    }
}
