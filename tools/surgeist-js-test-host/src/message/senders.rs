//! A collection of [`MessageSender`]s basic implementations.

use crate::message::MessageSender;
use crate::store::JsValueStore;
use futures::Stream;
use futures::channel::mpsc::UnboundedSender;
use std::task::Poll;
use surgeist_js::gc::heap::Trace as HeapTrace;
use surgeist_js::job::{ContextTask, JobObserver, NativeAsyncJob};
use surgeist_js::object::builtins::JsFunction;
use surgeist_js::value::TryIntoJs;
use surgeist_js::{Context, JsData, JsError, JsResult, JsString, JsValue, js_string};

/// A [`MessageSender`] that reads the `onMessageQueue` property of the global
/// object and calls it if it is a function. Note that this does not support
/// event listeners, only checks the (made-up) `onMessageQueue` property. It
/// also does not check the `targetOrigin` and accepts all messages.
#[derive(Debug, Clone, JsData, HeapTrace)]
pub struct OnMessageQueueSender {
    #[heap_trace(skip)]
    sender: UnboundedSender<JsValueStore>,
    #[heap_trace(skip)]
    observer: JobObserver,
}

impl MessageSender for OnMessageQueueSender {
    fn send(&self, message: JsValueStore, _target_origin: Option<JsString>) -> JsResult<()> {
        self.sender
            .unbounded_send(message)
            .map_err(JsError::from_rust)?;
        Ok(())
    }

    fn stop(&self) -> JsResult<()> {
        self.sender.close_channel();
        Ok(())
    }
}

impl OnMessageQueueSender {
    /// Create a `MessageQueueSender` that sends messages to a message queue
    /// array in the destination context. This type is send/sync and can be
    /// registered in any and many separate `Context` for `postMessage`.
    /// An executor's explicit discard policy returns an inert sender whose
    /// channel is closed; sending then returns a channel error.
    ///
    /// # Errors
    /// Returns an engine error if publication rejects the receiver or reports
    /// an accepted failure. Accepted receiver work is never cancelled on error.
    pub fn create(destination: &mut Context) -> JsResult<Self> {
        let (sender, receiver) = futures::channel::mpsc::unbounded::<JsValueStore>();

        let job = NativeAsyncJob::new(move |_| {
            let mut receiver = Box::pin(receiver);
            Ok(ContextTask::from_poll(move |task, host| {
                loop {
                    let store = match receiver.as_mut().poll_next(task) {
                        Poll::Ready(Some(store)) => store,
                        Poll::Ready(None) => return Poll::Ready(Ok(JsValue::undefined())),
                        Poll::Pending => return Poll::Pending,
                    };
                    let result = host.with(|mut context| {
                        let v = store.try_into_js(context.reborrow())?;
                        let global = context.global_object()?;
                        if let Some(x) = global
                            .get(js_string!("onMessageQueue"), context.reborrow())?
                            .as_callable()
                            .and_then(JsFunction::from_object)
                        {
                            x.call(&JsValue::undefined(), &[v], context.reborrow())?;
                        }
                        Ok(())
                    });
                    if let Err(error) = result {
                        return Poll::Ready(Err(error));
                    }
                }
            }))
        });
        let observer = job.observer();
        destination
            .enqueue_job(job.into())
            .map_err(surgeist_js::job::JobPublicationError::into_error)?
            .into_result()?;

        Ok(Self { sender, observer })
    }

    /// Observes completion, cancellation, or owner loss of the receiving task.
    ///
    /// This handle contains no engine access and can be used on another thread.
    #[must_use]
    pub fn observer(&self) -> JobObserver {
        self.observer.clone()
    }
}
