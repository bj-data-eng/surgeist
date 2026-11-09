//! Owned host work with temporary engine access at each poll.

use crate::{ContextAccess, HostContext, JsResult, error::JobError};
use std::{
    fmt,
    future::Future,
    task::{Context as TaskContext, Poll},
};

type PollTask<T> = dyn FnMut(&mut TaskContext<'_>, &HostContext<'_>) -> Poll<JsResult<T>>;

enum TaskState {
    Active,
    Terminal,
}

enum Completion<F> {
    Waiting(F),
    Consumed,
}

/// Owned asynchronous work that cannot retain the engine access supplied to a poll.
pub struct ContextTask<T: 'static> {
    poll: Box<PollTask<T>>,
    state: TaskState,
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
            state: TaskState::Active,
        }
    }

    /// Converts a successful result in one synchronous protected completion.
    pub fn then<U: 'static, F>(mut self, complete: F) -> ContextTask<U>
    where
        F: for<'call> FnOnce(T, ContextAccess<'call>) -> JsResult<U> + 'static,
    {
        let mut complete = Completion::Waiting(complete);
        ContextTask::from_poll(move |cx, host| match self.poll(cx, host) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Ready(Ok(value)) => {
                match std::mem::replace(&mut complete, Completion::Consumed) {
                    Completion::Waiting(complete) => {
                        Poll::Ready(host.with(|access| complete(value, access)))
                    }
                    Completion::Consumed => Poll::Ready(Err(JobError::TaskCompleted.into())),
                }
            }
        })
    }

    /// Polls one stage using access that cannot escape into owned task state.
    ///
    /// A terminal task returns an uncatchable completed-task error without
    /// polling its command again. Ready and Rust unwinding are terminal;
    /// only Pending permits another poll. Captured storage remains owned until
    /// this task is dropped, preserving its caller's cleanup ordering.
    pub fn poll(&mut self, cx: &mut TaskContext<'_>, host: &HostContext<'_>) -> Poll<JsResult<T>> {
        if matches!(self.state, TaskState::Terminal) {
            return Poll::Ready(Err(JobError::TaskCompleted.into()));
        }
        // Publish terminality before host work so unwinding cannot reopen it.
        self.state = TaskState::Terminal;
        let result = (self.poll)(cx, host);
        if result.is_pending() {
            self.state = TaskState::Active;
        }
        result
    }
}
