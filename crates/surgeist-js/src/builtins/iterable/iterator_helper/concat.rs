use std::{cell::RefCell, collections::VecDeque};

use surgeist_js_gc::heap::{AccessError, Rooted, Trace};

use crate::{
    Context, JsObject,
    builtins::iterable::IteratorRecord,
    error::RootingError,
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
    pub(crate) fn new(
        iterables: VecDeque<IterableRecord>,
        context: &Context,
    ) -> Result<NativeCoroutine, RootingError> {
        // 3. Let closure be a new Abstract Closure with no parameters that captures
        //    iterables and performs the following steps when called:
        iterables
            .len()
            .checked_mul(std::mem::size_of::<StoredIterableRecord>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or(RootingError::SizeOverflow)?;
        let mut pending = VecDeque::new();
        pending
            .try_reserve(iterables.len())
            .map_err(|_| RootingError::Allocation)?;
        for record in &iterables {
            pending.push_back(StoredIterableRecord {
                iterable: record.iterable.store(),
                open_method: record.open_method.store(),
            });
        }
        let pending = context
            .runtime()
            .heap
            .allocate(RefCell::new(pending))
            .map_err(|rejected| RootingError::Access(rejected.reason))?;
        NativeCoroutine::from_fn_with_state::<_, Option<IteratorRecord>>(
            |completion,
             pending: &Rooted<RefCell<VecDeque<StoredIterableRecord>>>,
             prepared,
             state,
             context| {
                let Some(mut current) = prepared else {
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
                        state.set(Some(iterator)).branch()?;
                        return CoroutineState::Continue(value);
                    }

                    let (iterable, open_method, removed) = {
                        let mut pending = pending
                            .try_borrow_mut()
                            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))
                            .branch()?;
                        let Some(record) = pending.front() else {
                            return CoroutineState::Break(Ok(()));
                        };
                        // Acquire both live roots before withdrawing the stored
                        // front. Capture restoration cannot reenter this owner.
                        let iterable: JsObject = JsObject::restore(&record.iterable).branch()?;
                        let open_method = JsFunction::restore(&record.open_method).branch()?;
                        (iterable, open_method, pending.pop_front())
                    };
                    drop(removed);
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
