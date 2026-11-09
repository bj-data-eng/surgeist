//! Tests for the `postMessage` extension.

use crate::message;
use crate::message::MessageSender;
use crate::message::senders::OnMessageQueueSender;
use crate::store::JsValueStore;
use crate::test::{TestAction, run_test_actions_with};
use futures_lite::future;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use surgeist_js::job::{JobExecutor, JobObserver, JobRetirement, JobStatus, SimpleJobExecutor};
use surgeist_js::{Context, JsValue, Source, js_string};

fn poll_receiver_once(context: &mut Context, executor: Rc<SimpleJobExecutor>) {
    let host = context.host_access();
    assert!(future::block_on(future::poll_once(executor.run_jobs_async(&host))).is_none());
}

#[test]
fn sender_and_observer_keep_their_cross_thread_contracts() {
    fn assert_traits<T: Send + Sync + Clone>() {}
    assert_traits::<OnMessageQueueSender>();
    assert_traits::<JobObserver>();
}

#[test]
fn rejected_receiver_startup_returns_an_error_and_disposes_its_task() {
    use crate::publication_test_support::PublicationExecutor;
    use surgeist_js::error::{EngineError, EngineResourceError};

    let executor = Rc::new(PublicationExecutor::new(None));
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let error = OnMessageQueueSender::create(&mut context).unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::Resource(EngineResourceError::Allocation))
    );
    assert_eq!(executor.attempts.get(), 1);
    assert_eq!(
        executor.observations.borrow()[0].status().unwrap(),
        JobStatus::Retired(JobRetirement::Dropped)
    );
}

#[test]
fn accepted_receiver_error_preserves_the_task_until_its_closed_channel_completes() {
    use crate::publication_test_support::PublicationExecutor;

    let executor = Rc::new(PublicationExecutor::new(Some(0)));
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    assert!(OnMessageQueueSender::create(&mut context).is_err());
    assert_eq!(
        executor.observations.borrow()[0].status().unwrap(),
        JobStatus::Pending
    );
    context.run_jobs().unwrap();
    assert_eq!(
        executor.observations.borrow()[0].status().unwrap(),
        JobStatus::Completed
    );
}

#[test]
fn idle_receiver_discard_returns_an_inert_closed_sender() {
    use surgeist_js::job::IdleJobExecutor;

    let mut context = Context::builder()
        .job_executor(Rc::new(IdleJobExecutor))
        .build()
        .unwrap();
    let sender = OnMessageQueueSender::create(&mut context).unwrap();
    assert_eq!(
        sender.observer().status().unwrap(),
        JobStatus::Retired(JobRetirement::Dropped)
    );
    let value = JsValueStore::try_from_js(&JsValue::new(42), context.access(), Vec::new()).unwrap();
    assert!(sender.send(value, None).is_err());
}

#[test]
fn pending_receiver_survives_a_dropped_drain_and_completes_when_its_channel_closes() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    context
        .eval(Source::from_bytes(
            "globalThis.received = 0; function onMessageQueue(value) { received = value; }",
        ))
        .unwrap();
    let sender = OnMessageQueueSender::create(&mut context).unwrap();
    let observer = sender.observer();
    let mut wait = Box::pin(observer.wait().unwrap());
    assert!(future::block_on(future::poll_once(wait.as_mut())).is_none());
    poll_receiver_once(&mut context, executor.clone());
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);

    let value = JsValueStore::try_from_js(&JsValue::new(42), context.access(), Vec::new()).unwrap();
    sender.send(value, None).unwrap();
    poll_receiver_once(&mut context, executor);
    assert_eq!(
        context.eval(Source::from_bytes("received")).unwrap(),
        JsValue::new(42)
    );
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    sender.stop().unwrap();
    context.run_jobs().unwrap();
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        future::block_on(future::poll_once(wait.as_mut())).map(Result::unwrap),
        Some(JobStatus::Completed)
    );
}

#[test]
fn pending_receiver_reports_executor_cancellation_to_its_native_waiter() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let sender = OnMessageQueueSender::create(&mut context).unwrap();
    let observer = sender.observer();
    let mut wait = Box::pin(observer.wait().unwrap());
    poll_receiver_once(&mut context, executor.clone());
    assert!(future::block_on(future::poll_once(wait.as_mut())).is_none());
    executor
        .get_cancellation_token()
        .store(true, Ordering::Relaxed);
    context.run_jobs().unwrap();
    let retired = JobStatus::Retired(JobRetirement::Cancelled);
    assert_eq!(observer.status().unwrap(), retired);
    assert_eq!(
        future::block_on(future::poll_once(wait.as_mut())).map(Result::unwrap),
        Some(retired)
    );
    let value = JsValueStore::try_from_js(&JsValue::new(42), context.access(), Vec::new()).unwrap();
    assert!(sender.send(value, None).is_err());
}

#[test]
fn pending_receiver_reports_its_exact_owners_loss_to_its_native_waiter() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let sender = OnMessageQueueSender::create(&mut context).unwrap();
    let observer = sender.observer();
    let mut wait = Box::pin(observer.wait().unwrap());
    poll_receiver_once(&mut context, executor);
    assert!(future::block_on(future::poll_once(wait.as_mut())).is_none());
    drop(context);
    let retired = JobStatus::Retired(JobRetirement::OwnerDropped);
    assert_eq!(observer.status().unwrap(), retired);
    assert_eq!(
        future::block_on(future::poll_once(wait.as_mut())).map(Result::unwrap),
        Some(retired)
    );
}

/// Create a basic context and allow postMessage from the same context.
#[test]
fn basic() {
    let context = &mut Context::default();

    let sender = OnMessageQueueSender::create(context).unwrap();
    message::register(sender, None, context).unwrap();

    run_test_actions_with(
        [
            TestAction::harness(),
            TestAction::run(
                r#"
                let latestMessage = null;
                function onMessageQueue(message) {
                    latestMessage = message;
                }

                const message = { "hello": "world" };
                postMessage(message);
                assert(latestMessage === null);
            "#,
            ),
            TestAction::inspect_context(move |context| {
                let job_executor = context
                    .downcast_job_executor::<SimpleJobExecutor>()
                    .expect("");
                let host = context.host_access();
                drop(future::block_on(future::poll_once(
                    job_executor.run_jobs_async(&host),
                )));
            }),
            TestAction::run(
                r#"
                assert(latestMessage !== null);
                assert(latestMessage !== message);
                assertEq(latestMessage.hello, "world");
            "#,
            ),
        ],
        context,
    );
}

#[test]
fn shared_multi_thread() {
    let (sender, receiver) = std::sync::mpsc::channel::<OnMessageQueueSender>();

    let destination_handle = thread::spawn(move || {
        let context = &mut Context::default();

        // It's important to declare the `onMessageQueue` function before we might
        // receive any messages, as those will be lost.
        run_test_actions_with(
            [
                TestAction::harness(),
                TestAction::run(
                    r#"
                    done = false;
                    function onMessageQueue(message) {
                        assert(message.hello === "world");
                        done = true;
                    }
                "#,
                ),
            ],
            context,
        );

        sender
            .send(OnMessageQueueSender::create(context).unwrap())
            .unwrap();

        loop {
            thread::sleep(Duration::from_millis(10));
            context.run_jobs().unwrap();

            let global_object = context.global_object();
            if global_object
                .get(js_string!("done"), context.access())
                .unwrap()
                .as_boolean()
                == Some(true)
            {
                break;
            }
        }
    });

    let source_handle = thread::spawn(move || {
        let context = &mut Context::default();
        let message_sender = receiver.recv().unwrap();
        message::register(message_sender, None, context).unwrap();

        run_test_actions_with(
            [TestAction::run(
                r#"
                    const message = { "hello": "world" };
                    postMessage(message);
                "#,
            )],
            context,
        );
    });

    source_handle.join().unwrap();
    destination_handle.join().unwrap();
}

#[test]
fn shared_array_buffer() {
    let (sender, receiver) = std::sync::mpsc::channel::<OnMessageQueueSender>();

    let destination_handle = thread::spawn(move || {
        let context = &mut Context::default();

        // It's important to declare the `onMessageQueue` function before we might
        // receive any messages, as those will be lost.
        run_test_actions_with(
            [
                TestAction::harness(),
                TestAction::run(
                    r#"
                    done = false;
                    function onMessageQueue(message) {
                        const shared = message.shared;
                        const view = new DataView(shared);
                        while (true) {
                            if (view.getUint8(0) == 1) {
                                view.setUint8(0, 2);
                                break;
                            }
                        }

                        // Verify that it also comes back.
                        while (true) {
                            if (view.getUint8(0) == 3) {
                                done = true;
                                break;
                            }
                        }
                    }
                "#,
                ),
            ],
            context,
        );

        sender
            .send(OnMessageQueueSender::create(context).unwrap())
            .unwrap();

        loop {
            thread::sleep(Duration::from_millis(10));
            context.run_jobs().unwrap();

            let global_object = context.global_object();
            if global_object
                .get(js_string!("done"), context.access())
                .unwrap()
                .as_boolean()
                == Some(true)
            {
                break;
            }
        }
    });

    let source_handle = thread::spawn(move || {
        let context = &mut Context::default();

        let message_sender = receiver.recv().unwrap();
        message::register(message_sender, None, context).unwrap();

        run_test_actions_with(
            [TestAction::run(
                r#"
                    const shared = new SharedArrayBuffer(1);
                    const message = { shared };
                    postMessage(message);

                    // Set shared to 1.
                    const view = new DataView(shared);
                    view.setUint8(0, 1);

                    // This would never work if the two contexts are in the same context.
                    while (true) {
                        if (view.getUint8(0) == 2) {
                            view.setUint8(0, 3);
                            break;
                        }
                    }
                "#,
            )],
            context,
        );
    });

    source_handle.join().unwrap();
    destination_handle.join().unwrap();
}
