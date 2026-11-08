//! Owned host work with temporary engine access at each poll.

use crate::{ContextAccess, HostContext, JsResult};
use std::{
    fmt,
    future::Future,
    task::{Context as TaskContext, Poll},
};

type PollTask<T> = dyn FnMut(&mut TaskContext<'_>, &HostContext<'_>) -> Poll<JsResult<T>>;

/// Owned asynchronous work that cannot retain the engine access supplied to a poll.
pub struct ContextTask<T: 'static> {
    poll: Box<PollTask<T>>,
}

impl<T> fmt::Debug for ContextTask<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextTask").finish_non_exhaustive()
    }
}

impl<T: 'static> ContextTask<T> {
    /// Lifts an owned host future without lending it engine access.
    pub fn from_future<F>(future: F) -> Self
    where
        F: Future<Output = JsResult<T>> + 'static,
    {
        let mut future = Box::pin(future);
        Self::from_poll(move |cx, _| future.as_mut().poll(cx))
    }

    /// Creates an owned staged task whose poll receives temporary protected access.
    pub fn from_poll<F>(poll: F) -> Self
    where
        F: FnMut(&mut TaskContext<'_>, &HostContext<'_>) -> Poll<JsResult<T>> + 'static,
    {
        Self {
            poll: Box::new(poll),
        }
    }

    /// Converts a successful result in one synchronous protected completion.
    pub fn then<U: 'static, F>(mut self, complete: F) -> ContextTask<U>
    where
        F: for<'call> FnOnce(T, ContextAccess<'call>) -> JsResult<U> + 'static,
    {
        let mut complete = Some(complete);
        ContextTask::from_poll(move |cx, host| match self.poll(cx, host) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Ready(Ok(value)) => {
                let complete = complete.take().expect("completion runs once");
                Poll::Ready(host.with(|access| complete(value, access)))
            }
        })
    }

    /// Polls one stage using access that cannot escape into owned task state.
    pub fn poll(&mut self, cx: &mut TaskContext<'_>, host: &HostContext<'_>) -> Poll<JsResult<T>> {
        (self.poll)(cx, host)
    }
}
