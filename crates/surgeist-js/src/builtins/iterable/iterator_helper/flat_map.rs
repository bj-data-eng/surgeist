use crate::{
    Context, JsValue,
    builtins::iterable::{IteratorRecord, get_iterator_flattenable},
    error::RootingError,
    native_function::{CoroutineBranch, CoroutineState, NativeCoroutine},
    object::JsFunction,
    vm::CompletionRecord,
};

/// [`Iterator.prototype.flatMap(mapper)`][spec]
///
/// Maps all values of the underlying iterator with `mapper`, flattening
/// inner iterable values into a single stream of values.
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator.prototype.flatmap
pub(crate) struct FlatMap;

impl FlatMap {
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
                let Some((mut iterated, mapper, mut counter, mut inner)) = prepared else {
                    return CoroutineState::Break(Ok(()));
                };

                if let Some(inner_iterator) = &inner {
                    // Close the inner iterator before the outer iterator on abrupt resumption.
                    let abrupt = match completion {
                        CompletionRecord::Normal(_) => None,
                        CompletionRecord::Return(value) => Some(Ok(value)),
                        CompletionRecord::Throw(error) => Some(Err(error)),
                    };
                    if let Some(abrupt) = abrupt {
                        if_abrupt_close_iterator!(
                            inner_iterator.close(abrupt.clone(), context),
                            iterated,
                            context
                        );
                        return CoroutineState::Break(iterated.close(abrupt, context).map(|_| ()));
                    }
                } else {
                    iterated.if_abrupt_close_iterator(completion, context)?;
                }

                loop {
                    if let Some(mut inner_iterator) = inner.take() {
                        let inner_value = if_abrupt_close_iterator!(
                            inner_iterator.step_value(context),
                            iterated,
                            context
                        );
                        if let Some(value) = inner_value {
                            state
                                .set((iterated, mapper, counter, Some(inner_iterator)))
                                .branch()?;
                            return CoroutineState::Continue(value);
                        }
                        // The inner iterator is done. Advance the outer iterator.
                        counter += 1;
                    }

                    let Some(value) = iterated.step_value(context).branch()? else {
                        return CoroutineState::Break(Ok(()));
                    };
                    let mapped_value = if_abrupt_close_iterator!(
                        mapper.call(
                            &JsValue::undefined(),
                            &[value, counter.into()],
                            context.access()
                        ),
                        iterated,
                        context
                    );
                    inner = Some(if_abrupt_close_iterator!(
                        get_iterator_flattenable(&mapped_value, false, context),
                        iterated,
                        context
                    ));
                }
            },
            (),
            (iterated, mapper, 0_u64, None::<IteratorRecord>),
            context,
        )
    }
}
