use std::task::Poll;
use surgeist_js::{
    JsValue, Source,
    job::{ContextTask, NativeAsyncJob},
};

fn factory_access_cannot_be_retained() {
    let _job = NativeAsyncJob::new(|mut access| {
        Ok(ContextTask::from_future(async move {
            access.eval(Source::from_bytes("42"))
        }))
    });
}

fn poll_holder_cannot_be_retained() {
    let _job = NativeAsyncJob::new(|_| {
        Ok(ContextTask::from_poll(|_, host| {
            let _escaped = ContextTask::from_future(async move {
                host.with(|mut access| access.eval(Source::from_bytes("42")))
            });
            Poll::Ready(Ok(JsValue::undefined()))
        }))
    });
}

fn main() {
    factory_access_cannot_be_retained();
    poll_holder_cannot_be_retained();
}
