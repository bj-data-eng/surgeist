use std::{cell::Cell, pin::pin, rc::Rc};

use futures_lite::future;

use crate::{
    JsValue, TestAction,
    context::{ContextBuilder, time::FixedClock},
    job::{ContextTask, GenericJob, JobExecutor, NativeAsyncJob, SimpleJobExecutor},
    run_test_actions_with,
};

#[test]
fn test_async_job_not_blocking_event_loop() {
    let clock = Rc::new(FixedClock::default());
    let context = &mut ContextBuilder::default()
        .clock(clock.clone())
        .build()
        .unwrap();

    run_test_actions_with(
        [TestAction::inspect_context_async(async move |ctx| {
            let executor = ctx.downcast_job_executor::<SimpleJobExecutor>().unwrap();
            let ctx = &ctx.host_access();

            let mut event_loop = pin!(future::poll_once(executor.run_jobs_async(ctx)));

            // There are no jobs in our queue. Push
            // an async job that will consistently yield to the executor.
            ctx.with(|mut access| {
                access
                    .enqueue_job(
                        NativeAsyncJob::new(|_| {
                            Ok(ContextTask::from_future(async {
                                loop {
                                    future::yield_now().await;
                                }
                            }))
                        })
                        .into(),
                    )
                    .unwrap()
                    .into_result()
                    .unwrap();
                Ok(())
            })
            .unwrap();

            // Then, start the event loop
            assert!(event_loop.as_mut().await.is_none());

            let checker = Rc::new(Cell::new(false));
            {
                let checker = checker.clone();
                // At this point, the event loop should have yielded again to the async executor.
                // Thus, enqueue a generic job that should resolve in the next loop.
                let realm = ctx.with(|access| Ok(access.realm().clone())).unwrap();
                ctx.with(|mut access| {
                    access
                        .enqueue_job(
                            GenericJob::new(
                                move |_| {
                                    checker.set(true);
                                    Ok(JsValue::undefined())
                                },
                                realm,
                            )
                            .into(),
                        )
                        .unwrap()
                        .into_result()
                        .unwrap();
                    Ok(())
                })
                .unwrap();
            }

            // Next iteration of the event loop
            assert!(event_loop.as_mut().await.is_none());

            // At this point, our generic job should have been executed.
            assert!(checker.get());
        })],
        context,
    );
}
