use std::{cell::RefCell, collections::VecDeque};

use surgeist_js_gc::heap::{Rooted, Trace};

use crate::{
    Context, JsObject,
    builtins::iterable::IteratorRecord,
    js_error,
    native_function::{
        CapturedObject, CoroutineBranch, CoroutineState, NativeCapture, NativeCoroutine,
    },
    object::JsFunction,
};

use super::super::get_iterator_direct;

pub(crate) struct IterableRecord {
    pub(crate) iterable: JsObject,
    pub(crate) open_method: JsFunction,
}

#[derive(Trace)]
struct StoredIterableRecord {
    iterable: CapturedObject,
    open_method: CapturedObject,
}

/// [`Iterator.concat ( ...items )`][spec]
///
/// Concatenate a list of iterators.
///
/// [spec]: https://tc39.es/ecma262/#sec-iterator.concat
pub(crate) struct Concat;

impl Concat {
    #[allow(
        clippy::new_ret_no_self,
        reason = "slightly cleaner to have this be a `new` method"
    )]
    pub(crate) fn new(iterables: VecDeque<IterableRecord>, context: &Context) -> NativeCoroutine {
        // 3. Let closure be a new Abstract Closure with no parameters that captures
        //    iterables and performs the following steps when called:
        let pending: VecDeque<_> = iterables
            .iter()
            .map(|record| StoredIterableRecord {
                iterable: record.iterable.store(),
                open_method: record.open_method.store(),
            })
            .collect();
        let pending = context
            .runtime()
            .heap
            .allocate(RefCell::new(pending))
            .expect("iterables belong to the continuation runtime");
        NativeCoroutine::from_fn_with_state::<_, Option<IteratorRecord>>(
            |completion,
             pending: &Rooted<RefCell<VecDeque<StoredIterableRecord>>>,
             state,
             context| {
                let Some(mut current) = state.take() else {
                    return CoroutineState::Break(Ok(()));
                };

                if let Some(current) = &current {
                    current.if_abrupt_close_iterator(completion, context)?;
                } else {
                    completion.branch()?;
                }

                loop {
                    if let Some(mut iterator) = current.take()
                        && let Some(value) = iterator.step_value(context).branch()?
                    {
                        state.set(Some(iterator));
                        return CoroutineState::Continue(value);
                    }

                    let record = pending.borrow_mut().pop_front();
                    let Some(StoredIterableRecord {
                        iterable,
                        open_method,
                    }) = record
                    else {
                        return CoroutineState::Break(Ok(()));
                    };
                    let iterable: JsObject = JsObject::restore(&iterable);
                    let open_method = JsFunction::restore(&open_method);
                    let iter = open_method
                        .call(&iterable.into(), &[], context.access())
                        .branch()?;
                    let Some(iter) = iter.as_object() else {
                        return CoroutineState::Break(Err(
                            js_error!(TypeError: "Iterator.concat: open method of iterable did not return an object"),
                        ));
                    };
                    current = Some(get_iterator_direct(&iter, context).branch()?);
                }
            },
            pending,
            None::<IteratorRecord>,
            context,
        )
    }
}
