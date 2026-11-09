use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{
    Context, JsResult, JsValue, Source, TestAction,
    builtins::atomics::Atomics,
    job::{IdleJobExecutor, Job, JobAcceptance, JobExecutor, JobPublicationError, TimeoutJob},
    js_string,
    module::IdleModuleLoader,
    object::{JsInt32Array, JsPromise, JsSharedArrayBuffer},
    run_test_actions,
    value::TryFromJs,
};

#[test]
fn atomics_revalidate_resizable_buffers_after_argument_coercion() {
    run_test_actions([
        TestAction::run(
            r#"
            var backing = new ArrayBuffer(8, {maxByteLength: 8});
            var tracking = new Int32Array(backing);
            tracking[0] = 42;
            var loaded = Atomics.load(tracking, { valueOf() { backing.resize(4); return 0; } });

            backing.resize(8);
            var rangeError = false;
            try {
                Atomics.store(tracking, 1, { valueOf() { backing.resize(4); return 7; } });
            } catch (error) {
                rangeError = error instanceof RangeError;
            }

            backing.resize(8);
            var fixed = new Int32Array(backing, 0, 2);
            var typeError = false;
            try {
                Atomics.add(fixed, 0, { valueOf() { backing.resize(4); return 1; } });
            } catch (error) {
                typeError = error instanceof TypeError;
            }

            backing.resize(8);
            var conversions = [];
            var previous = Atomics.compareExchange(
                tracking,
                0,
                { valueOf() { conversions.push('expected'); return 42; } },
                { valueOf() { conversions.push('replacement'); backing.resize(4); return 9; } },
            );
            "#,
        ),
        TestAction::assert("loaded === 42 && rangeError && typeError"),
        TestAction::assert("previous === 42 && tracking[0] === 9"),
        TestAction::assert("conversions.join(',') === 'expected,replacement'"),
    ]);
}

#[test]
fn discarded_async_job_unregisters_its_waiter() {
    let mut context = Context::builder()
        .job_executor(Rc::new(IdleJobExecutor))
        .build()
        .unwrap();
    let notified = context
        .eval(Source::from_bytes(
            r#"
            var view = new Int32Array(new SharedArrayBuffer(4));
            var result = Atomics.waitAsync(view, 0, 0);
            if (!result.async) throw new Error('expected a queued wait');
            Atomics.notify(view, 0);
            "#,
        ))
        .unwrap();

    assert_eq!(notified.as_number(), Some(0.0));
}

#[test]
fn discarded_async_job_revokes_its_pending_timeout() {
    #[derive(Default)]
    struct KeepTimeout(RefCell<Option<TimeoutJob>>);

    impl JobExecutor for KeepTimeout {
        fn enqueue_job(
            self: Rc<Self>,
            job: Job,
            mut access: crate::ContextAccess<'_>,
        ) -> Result<JobAcceptance, JobPublicationError> {
            if let Job::TimeoutJob(timeout) = job {
                let job = Job::TimeoutJob(timeout);
                if let Err(cause) = job.check_publication(access.reborrow()) {
                    return Err(JobPublicationError::new(job, cause));
                }
                let Job::TimeoutJob(timeout) = job else {
                    unreachable!("the retained job is the authored timeout")
                };
                self.0.replace(Some(timeout));
                Ok(JobAcceptance::Accepted)
            } else {
                drop(job);
                Ok(JobAcceptance::Discarded)
            }
        }

        fn run_jobs(self: Rc<Self>, _: crate::ContextAccess<'_>) -> JsResult<()> {
            Ok(())
        }
    }

    let executor = Rc::new(KeepTimeout::default());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let notified = context
        .eval(Source::from_bytes(
            "var view = new Int32Array(new SharedArrayBuffer(4)); \
             Atomics.waitAsync(view, 0, 0, 1000); Atomics.notify(view, 0);",
        ))
        .unwrap();

    assert_eq!(notified.as_number(), Some(0.0));
    assert!(executor.0.borrow().as_ref().unwrap().cancelled());
}

#[test]
fn shared_waiter_lists_follow_byte_offsets_across_element_widths() {
    run_test_actions([
        TestAction::run(
            r#"
            var buffer = new SharedArrayBuffer(8);
            var words = new Int32Array(buffer);
            var wide = new BigInt64Array(buffer);
            var first = Atomics.waitAsync(words, 0, 0);
            var second = Atomics.waitAsync(wide, 0, 0n);
            var wrongOffset = Atomics.notify(words, 1);
            var firstNotify = Atomics.notify(wide, 0, 1);
            var secondNotify = Atomics.notify(words, 0, 1);
            var empty = Atomics.notify(words, 0);
            "#,
        ),
        TestAction::assert("first.async && second.async"),
        TestAction::assert(
            "wrongOffset === 0 && firstNotify === 1 && secondNotify === 1 && empty === 0",
        ),
    ]);
}

#[test]
fn waiterlist_block_indexedposition_wake() {
    const NUMAGENT: i32 = 2;
    const RUNNING: i32 = 4;

    let context = &mut Context::builder()
        .can_block(true)
        .module_loader(Rc::new(IdleModuleLoader))
        .build()
        .unwrap();
    let buffer = JsSharedArrayBuffer::new(size_of::<i32>() * 5, context.access()).unwrap();
    let inner_buffer = buffer.inner();
    let i32a = JsInt32Array::from_shared_array_buffer(buffer.clone(), context.access()).unwrap();

    std::thread::scope(|s| {
        let mut threads = Vec::new();
        for idx in [2, 0] {
            let buffer = inner_buffer.clone();
            let handle = s.spawn(move || {
                let context = &mut Context::builder()
                    .can_block(true)
                    .module_loader(Rc::new(IdleModuleLoader))
                    .build()
                    .unwrap();
                let buffer = JsSharedArrayBuffer::from_buffer(buffer, context.access());
                let i32a =
                    JsInt32Array::from_shared_array_buffer(buffer, context.access()).unwrap();

                Atomics::add(
                    &JsValue::undefined(),
                    &[i32a.clone().into(), RUNNING.into(), 1.into()],
                    context,
                )
                .unwrap();

                let promise = JsPromise::try_from_js(
                    &Atomics::wait::<true>(
                        &JsValue::undefined(),
                        &[
                            i32a.clone().into(),
                            idx.into(),
                            0.into(),
                            f64::INFINITY.into(),
                        ],
                        context,
                    )
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .get(js_string!("value"), context.access())
                    .unwrap(),
                    context.access(),
                )
                .unwrap();

                assert_eq!(
                    promise
                        .await_blocking(context.access())
                        .unwrap()
                        .to_string(context.access())
                        .unwrap()
                        .to_std_string_lossy(),
                    "ok"
                );
            });
            threads.push(handle);
        }

        while Atomics::load(
            &JsValue::undefined(),
            &[i32a.clone().into(), RUNNING.into()],
            context,
        )
        .unwrap()
        .to_i32(context.access())
        .unwrap()
            != NUMAGENT
        {}

        std::thread::sleep(Duration::from_millis(100));

        assert_eq!(
            Atomics::load(
                &JsValue::undefined(),
                &[i32a.clone().into(), RUNNING.into()],
                context,
            )
            .unwrap()
            .to_i32(context.access())
            .unwrap(),
            NUMAGENT
        );

        for missing in [1, 3] {
            assert_eq!(
                Atomics::notify(
                    &JsValue::undefined(),
                    &[i32a.clone().into(), missing.into()],
                    context
                )
                .unwrap()
                .to_i32(context.access())
                .unwrap(),
                0
            );
        }

        for exists in [2, 0] {
            assert_eq!(
                Atomics::notify(
                    &JsValue::undefined(),
                    &[i32a.clone().into(), exists.into()],
                    context
                )
                .unwrap()
                .to_i32(context.access())
                .unwrap(),
                1
            );
        }
    });
}
