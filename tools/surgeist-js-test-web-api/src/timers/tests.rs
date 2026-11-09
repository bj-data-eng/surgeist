use crate::test::{TestAction, run_test_actions_with};
use crate::timers as interval;
use futures_lite::future::poll_once;
use indoc::indoc;
use std::pin::pin;
use std::rc::Rc;
use surgeist_js::context::time::FixedClock;
use surgeist_js::context::{Clock, ContextBuilder};
use surgeist_js::job::{JobExecutor, SimpleJobExecutor};
use surgeist_js::{Context, js_str};

fn create_context(clock: Rc<impl Clock + 'static>) -> Context {
    let mut context = ContextBuilder::default().clock(clock).build().unwrap();
    interval::register(&mut context).unwrap();
    context
}

#[test]
fn rejected_timeout_and_interval_leave_no_active_timer() {
    use crate::publication_test_support::PublicationExecutor;
    use surgeist_js::{
        Source,
        error::{EngineError, EngineResourceError},
        interop::JsRest,
        object::builtins::JsFunction,
    };

    for interval in [false, true] {
        let executor = Rc::new(PublicationExecutor::new(false));
        let mut context = Context::builder().job_executor(executor).build().unwrap();
        let callback = context.eval(Source::from_bytes("(() => {})")).unwrap();
        let callback = JsFunction::from_object(callback.as_object().unwrap()).unwrap();
        let schedule = if interval {
            super::set_interval
        } else {
            super::set_timeout
        };
        let error = schedule(Some(callback), None, JsRest(&[]), context.access()).unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::Resource(EngineResourceError::Allocation))
        );
        assert!(
            super::IntervalInnerState::from_context(&mut context.access())
                .active_map
                .is_empty()
        );
    }
}

#[test]
fn discarded_timeout_and_interval_leave_no_active_timer() {
    use surgeist_js::{
        Source, interop::JsRest, job::IdleJobExecutor, object::builtins::JsFunction,
    };

    for interval in [false, true] {
        let mut context = Context::builder()
            .job_executor(Rc::new(IdleJobExecutor))
            .build()
            .unwrap();
        let callback = context.eval(Source::from_bytes("(() => {})")).unwrap();
        let callback = JsFunction::from_object(callback.as_object().unwrap()).unwrap();
        let schedule = if interval {
            super::set_interval
        } else {
            super::set_timeout
        };
        assert_ne!(
            schedule(Some(callback), None, JsRest(&[]), context.access()).unwrap(),
            0
        );
        assert!(
            super::IntervalInnerState::from_context(&mut context.access())
                .active_map
                .is_empty()
        );
    }
}

#[test]
fn accepted_timer_error_preserves_its_live_cancellation_custody() {
    use crate::publication_test_support::PublicationExecutor;
    use surgeist_js::{Source, interop::JsRest, object::builtins::JsFunction};

    for interval in [false, true] {
        let executor = Rc::new(PublicationExecutor::new(true));
        let mut context = Context::builder().job_executor(executor).build().unwrap();
        let callback = context.eval(Source::from_bytes("(() => {})")).unwrap();
        let callback = JsFunction::from_object(callback.as_object().unwrap()).unwrap();
        let schedule = if interval {
            super::set_interval
        } else {
            super::set_timeout
        };
        assert!(schedule(Some(callback), None, JsRest(&[]), context.access()).is_err());
        let tokens: Vec<_> = super::IntervalInnerState::from_context(&mut context.access())
            .active_map
            .values()
            .cloned()
            .collect();
        assert_eq!(tokens.len(), 1);
        assert!(!tokens[0].revoked());
        super::clear_all(context.access());
        assert!(tokens[0].revoked());
        assert!(
            super::IntervalInnerState::from_context(&mut context.access())
                .active_map
                .is_empty()
        );
    }
}

#[test]
fn two_zero_delay_timeouts_both_fire() {
    let clock = Rc::new(FixedClock::default());
    let context = &mut create_context(clock.clone());

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                order = [];
                setTimeout(() => order.push(1), 0);
                setTimeout(() => order.push(2), 0);
            "#}),
            TestAction::inspect_context(move |ctx| {
                clock.forward(1);
                ctx.run_jobs().unwrap();

                let order = ctx
                    .global_object()
                    .get(js_str!("order"), ctx.access())
                    .unwrap();
                let order = order.as_object().unwrap();
                assert_eq!(
                    order.get(js_str!("length"), ctx.access()).unwrap().as_i32(),
                    Some(2),
                    "both callbacks must fire"
                );
                assert_eq!(order.get(0usize, ctx.access()).unwrap().as_i32(), Some(1));
                assert_eq!(order.get(1usize, ctx.access()).unwrap().as_i32(), Some(2));
            }),
        ],
        context,
    );
}

#[test]
fn set_timeout_basic() {
    let clock = Rc::new(FixedClock::default());
    let context = &mut create_context(clock.clone());

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                called = false;
                setTimeout(() => { called = true; });
            "#}),
            TestAction::inspect_context(move |ctx| {
                let called = ctx
                    .global_object()
                    .get(js_str!("called"), ctx.access())
                    .unwrap();
                assert_eq!(called.as_boolean(), Some(false));

                clock.forward(1);
                ctx.run_jobs().unwrap();

                let called = ctx
                    .global_object()
                    .get(js_str!("called"), ctx.access())
                    .unwrap();
                assert_eq!(called.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn set_timeout_cancel() {
    let clock = Rc::new(FixedClock::default());
    let context = &mut create_context(clock.clone());
    let clock1 = clock.clone();
    let clock2 = clock.clone();

    run_test_actions_with(
        [
            TestAction::run(indoc! {r#"
                called = false;
                id = setTimeout(() => { called = true; }, 100);
            "#}),
            TestAction::inspect_context_async(async |ctx| {
                let job_executor = ctx.downcast_job_executor::<SimpleJobExecutor>().unwrap();
                let clock = clock1;

                let called = ctx
                    .global_object()
                    .get(js_str!("called"), ctx.access())
                    .unwrap();
                let ctx = &ctx.host_access();
                let mut event_loop = pin!(poll_once(job_executor.run_jobs_async(ctx)));

                assert_eq!(called.as_boolean(), Some(false));

                assert!(event_loop.as_mut().await.is_none());
                clock.forward(50);
                assert!(event_loop.as_mut().await.is_none());

                let global_object = ctx.with(|access| Ok(access.global_object())).unwrap();
                let called = ctx
                    .with(|access| global_object.get(js_str!("called"), access))
                    .unwrap();
                assert_eq!(called.as_boolean(), Some(false));
                assert!(event_loop.as_mut().await.is_none());
            }),
            TestAction::run("clearTimeout(id);"),
            TestAction::inspect_context(|ctx| {
                let clock = clock2;
                clock.forward(100);
                let called = ctx
                    .global_object()
                    .get(js_str!("called"), ctx.access())
                    .unwrap();
                // Should still be false, as it was cancelled.
                assert_eq!(called.as_boolean(), Some(false));
            }),
        ],
        context,
    );
}

#[test]
fn set_timeout_delay() {
    let clock = Rc::new(FixedClock::default());
    let context = &mut create_context(clock.clone());

    run_test_actions_with(
        [
            TestAction::run(
                r#"
            called = false;
            setTimeout(() => { called = true; }, 100);
        "#,
            ),
            TestAction::inspect_context_async(async move |ctx| {
                let job_executor = ctx.downcast_job_executor::<SimpleJobExecutor>().unwrap();
                let global_object = ctx.global_object();
                let ctx = &ctx.host_access();
                let mut event_loop = pin!(poll_once(job_executor.run_jobs_async(ctx)));

                // As long as the clock isn't updated, `called` will always be false.
                for _ in 0..5 {
                    let called = ctx
                        .with(|access| global_object.get(js_str!("called"), access))
                        .unwrap();
                    assert_eq!(called.as_boolean(), Some(false));
                    assert!(event_loop.as_mut().await.is_none());
                }

                // Move forward 50 milliseconds, `called` should still be false.
                clock.forward(50);
                assert!(event_loop.as_mut().await.is_none());
                let called = ctx
                    .with(|access| global_object.get(js_str!("called"), access))
                    .unwrap();
                assert_eq!(called.as_boolean(), Some(false));

                clock.forward(51);
                // Event loop should have exited since there are no more
                // jobs on the queue.
                assert!(event_loop.as_mut().await.is_some());
                let called = ctx
                    .with(|access| global_object.get(js_str!("called"), access))
                    .unwrap();
                assert_eq!(called.as_boolean(), Some(true));
            }),
        ],
        context,
    );
}

#[test]
fn set_interval_delay() {
    let clock = Rc::new(FixedClock::default());
    let context = &mut create_context(clock.clone());
    let clock1 = clock.clone(); // For the first test.
    let clock2 = clock.clone(); // For the first test.

    run_test_actions_with(
        [
            TestAction::run(
                r#"
                called = 0;
                id = setInterval(() => { called++; }, 100);
            "#,
            ),
            TestAction::inspect_context_async(async |ctx| {
                let clock = clock1;
                let global_object = ctx.global_object();

                let job_executor = ctx.downcast_job_executor::<SimpleJobExecutor>().unwrap();
                let ctx = &ctx.host_access();
                let mut event_loop = pin!(poll_once(job_executor.run_jobs_async(ctx)));

                // As long as the clock isn't updated, `called` will always be false.
                for _ in 0..5 {
                    let called = ctx
                        .with(|access| global_object.get(js_str!("called"), access))
                        .unwrap();
                    assert_eq!(called.as_i32(), Some(0));
                    assert!(event_loop.as_mut().await.is_none());
                }

                // Move forward 50 milliseconds.
                clock.forward(50);
                assert!(event_loop.as_mut().await.is_none());
                let called = ctx
                    .with(|access| global_object.get(js_str!("called"), access))
                    .unwrap();
                assert_eq!(called.as_i32(), Some(0));

                // Move forward 51 milliseconds.
                clock.forward(51);
                assert!(event_loop.as_mut().await.is_none());
                let called = ctx
                    .with(|access| global_object.get(js_str!("called"), access))
                    .unwrap();
                assert_eq!(called.as_i32(), Some(1));

                // Move forward 50 milliseconds.
                clock.forward(50);
                assert!(event_loop.as_mut().await.is_none());
                let called = ctx
                    .with(|access| global_object.get(js_str!("called"), access))
                    .unwrap();
                assert_eq!(called.as_i32(), Some(1));

                // Move forward 51 milliseconds.
                clock.forward(51);
                assert!(event_loop.as_mut().await.is_none());
                let called = ctx
                    .with(|access| global_object.get(js_str!("called"), access))
                    .unwrap();
                assert_eq!(called.as_i32(), Some(2));

                // Move forward 500 milliseconds, should only be called once.
                clock.forward(500);
                assert!(event_loop.as_mut().await.is_none());
                let called = ctx
                    .with(|access| global_object.get(js_str!("called"), access))
                    .unwrap();
                assert_eq!(called.as_i32(), Some(3));
            }),
            // Cancel
            TestAction::run("clearInterval(id);"),
            TestAction::inspect_context(move |ctx| {
                let clock = clock2;
                // Doesn't matter how long, this should not be called ever again.
                clock.forward(500);
                ctx.run_jobs().unwrap();
                let called = ctx
                    .global_object()
                    .get(js_str!("called"), ctx.access())
                    .unwrap();
                assert_eq!(called.as_i32(), Some(3));

                clock.forward(500);
                ctx.run_jobs().unwrap();
                let called = ctx
                    .global_object()
                    .get(js_str!("called"), ctx.access())
                    .unwrap();
                assert_eq!(called.as_i32(), Some(3));
            }),
        ],
        context,
    );
}

#[test]
fn clear_all_cancels_pending_timers_and_intervals() {
    let clock = Rc::new(FixedClock::default());
    let context = &mut create_context(clock.clone());

    run_test_actions_with(
        [
            // Arm one of each timer kind: a one-shot `setTimeout` and a
            // recurring `setInterval`, both due at t+100ms. `clear_all` must
            // cancel both, regardless of whether a timer fires once or repeats.
            TestAction::run(indoc! {r#"
                timeout_fired = false;
                interval_fired = false;
                setTimeout(() => { timeout_fired = true; }, 100);
                setInterval(() => { interval_fired = true; }, 100);
            "#}),
            TestAction::inspect_context_async(async move |ctx| {
                let job_executor = ctx.downcast_job_executor::<SimpleJobExecutor>().unwrap();
                let ctx = &ctx.host_access();

                // Drive the event loop once. With the clock still short of the
                // 100ms deadline, both timers stay pending and the loop parks
                // (`is_none`) rather than completing.
                let mut event_loop = pin!(poll_once(job_executor.run_jobs_async(ctx)));
                clock.forward(50);
                assert!(
                    event_loop.as_mut().await.is_none(),
                    "timers are not yet due; event loop must keep waiting",
                );

                // Cancel everything while the jobs are still queued. This is
                // the behaviour under test.
                ctx.with(|access| {
                    interval::clear_all(access);
                    Ok(())
                })
                .unwrap();

                // Advance well past both deadlines. Because the queue was
                // drained, no callback runs and the loop has no remaining work,
                // so `run_jobs_async` completes (`is_some`) on its next tick.
                clock.forward(100);
                assert!(
                    event_loop.as_mut().await.is_some(),
                    "after clear_all the queue is empty; event loop must exit",
                );

                // Neither callback should have produced an observable effect.
                let global = ctx.with(|access| Ok(access.global_object())).unwrap();
                let timeout_fired = ctx
                    .with(|access| global.get(js_str!("timeout_fired"), access))
                    .unwrap();
                let interval_fired = ctx
                    .with(|access| global.get(js_str!("interval_fired"), access))
                    .unwrap();
                assert_eq!(
                    timeout_fired.as_boolean(),
                    Some(false),
                    "cleared setTimeout callback must not run",
                );
                assert_eq!(
                    interval_fired.as_boolean(),
                    Some(false),
                    "cleared setInterval callback must not run",
                );
            }),
        ],
        context,
    );
}
