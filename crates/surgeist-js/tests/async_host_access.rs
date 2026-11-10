#![allow(unused_crate_dependencies)]
use std::{
    cell::{Cell, RefCell},
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js::{
    Context, ContextAccess, HostContext, JsError, JsNativeError, JsResult, JsValue, Module,
    NativeFunction, Source,
    builtins::promise::PromiseState,
    error::{EngineError, PanicError, RuntimeLimitError},
    execution::{ExecutionOptions, ExecutionReceipt, ExecutionTermination},
    job::{ContextTask, JobStep, NativeAsyncJob, SimpleJobExecutor},
    js_string,
    module::{ModuleLoader, ModuleRequest, Referrer},
    object::builtins::JsPromise,
};
fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
}
fn step(job: &mut NativeAsyncJob, host: &HostContext<'_>) -> JobStep {
    job.run_step(&mut TaskContext::from_waker(Waker::noop()), host)
}
fn prepared(job: &mut NativeAsyncJob, host: &HostContext<'_>) {
    assert!(matches!(step(job, host), JobStep::Prepared));
}
fn completed(job: &mut NativeAsyncJob, host: &HostContext<'_>) -> JsResult<JsValue> {
    match step(job, host) {
        JobStep::Complete(result) => result,
        _ => panic!("the authored task must complete in this step"),
    }
}
async fn yield_once() {
    let mut yielded = false;
    std::future::poll_fn(move |cx| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await;
}
#[test]
fn a_panicking_async_job_poll_restores_its_caller_realm() {
    let mut context = Context::builder().build().unwrap();
    let original = context.realm().clone();
    let selected = context.create_realm().unwrap();
    assert_ne!(selected, original);
    let expected = selected.clone();
    let mut job = NativeAsyncJob::with_realm(
        move |_| {
            Ok(ContextTask::from_poll(move |_, host| {
                host.with(|access| {
                    assert_eq!(access.realm(), &expected);
                    Ok(())
                })
                .unwrap();
                panic!("async job poll panic")
            }))
        },
        selected,
    );
    let panic = {
        let host = context.host_access();
        prepared(&mut job, &host);
        catch_unwind(AssertUnwindSafe(|| step(&mut job, &host)))
            .expect_err("the async callback panic must propagate to its host caller")
    };
    assert_eq!(panic.downcast_ref::<&str>(), Some(&"async job poll panic"));
    assert_eq!(
        context.eval(Source::from_bytes("40 + 2")).unwrap(),
        JsValue::new(42)
    );
    assert_eq!(context.realm(), &original);
}
#[test]
fn interleaved_async_jobs_restore_each_selected_realm_after_pending_and_ready() {
    let mut context = Context::builder().build().unwrap();
    let original = context.realm().clone();
    let first = context.create_realm().unwrap();
    let second = context.create_realm().unwrap();
    let make = |realm: surgeist_js::realm::Realm, value| {
        let expected = realm.clone();
        NativeAsyncJob::with_realm(
            move |_| {
                let mut first = true;
                Ok(ContextTask::from_poll(move |cx, host| {
                    host.with(|mut access| {
                        assert_eq!(access.realm(), &expected);
                        assert_eq!(access.eval(Source::from_bytes("40 + 2"))?, JsValue::new(42));
                        let busy = host.with(|_| Ok(())).unwrap_err();
                        assert_eq!(busy.as_engine(), Some(&EngineError::ContextAccessBusy));
                        Ok(())
                    })?;
                    if first {
                        first = false;
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    } else {
                        Poll::Ready(Ok(JsValue::new(value)))
                    }
                }))
            },
            realm,
        )
    };
    let host = context.host_access();
    let mut first = make(first, 20);
    let mut second = make(second, 22);
    let restored = || {
        host.with(|access| {
            assert_eq!(access.realm(), &original);
            Ok(())
        })
        .unwrap()
    };
    prepared(&mut first, &host);
    prepared(&mut second, &host);
    restored();
    assert!(matches!(step(&mut first, &host), JobStep::Pending));
    restored();
    assert!(matches!(step(&mut second, &host), JobStep::Pending));
    restored();
    assert_eq!(completed(&mut first, &host).unwrap(), JsValue::new(20));
    restored();
    assert_eq!(completed(&mut second, &host).unwrap(), JsValue::new(22));
    restored();
}
#[test]
fn selected_realm_async_polls_preserve_a_live_guest_callers_frames_and_return() {
    let mut context = Context::builder().build().unwrap();
    let selected = context.create_realm().unwrap();
    context
        .register_global_builtin_callable(
            js_string!("nestedJob"),
            0,
            NativeFunction::from_fn_with_captures(
                |_, _, selected, access| {
                    let caller = access.realm().clone();
                    let baseline = access
                        .stack_trace()
                        .map(|f| f.position())
                        .collect::<Vec<_>>();
                    assert!(!baseline.is_empty());
                    let expected = selected.clone();
                    let mut job = NativeAsyncJob::with_realm(
                        move |_| {
                            let mut first = true;
                            Ok(ContextTask::from_poll(move |cx, host| {
                                let value = host.with(|mut access| {
                                    assert_eq!(access.realm(), &expected);
                                    access.eval(Source::from_bytes(if first {
                                        "40"
                                    } else {
                                        "40 + 2"
                                    }))
                                })?;
                                if first {
                                    first = false;
                                    cx.waker().wake_by_ref();
                                    Poll::Pending
                                } else {
                                    Poll::Ready(Ok(value))
                                }
                            }))
                        },
                        selected.clone(),
                    );
                    let host = access.into_host_access();
                    let preserved = || {
                        host.with(|access| {
                            assert_eq!(access.realm(), &caller);
                            assert_eq!(
                                access
                                    .stack_trace()
                                    .map(|f| f.position())
                                    .collect::<Vec<_>>(),
                                baseline
                            );
                            Ok(())
                        })
                        .unwrap()
                    };
                    prepared(&mut job, &host);
                    preserved();
                    assert!(matches!(step(&mut job, &host), JobStep::Pending));
                    preserved();
                    assert_eq!(completed(&mut job, &host)?, JsValue::new(42));
                    preserved();
                    Ok(JsValue::new(42))
                },
                selected,
            ),
        )
        .unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "function liveCaller() { const answer = nestedJob(); return answer; } liveCaller()"
            ))
            .unwrap(),
        JsValue::new(42)
    );
    assert_eq!(context.stack_trace().count(), 0);
}
#[test]
fn selected_realm_poll_cleanup_preserves_transferred_guest_await_continuations() {
    let mut context = promise_context();
    let caller = context.realm().clone();
    let selected = context.create_realm().unwrap();
    let expected = selected.clone();
    let mut job = NativeAsyncJob::with_realm(
        move |_| {
            let mut promise = None;
            Ok(ContextTask::from_poll(move |cx, host| {
                host.with(|access| {
                    assert_eq!(access.realm(), &expected);
                    Ok(())
                })?;
                if promise.is_none() {
                    let value=host.with(|mut access|access.eval(Source::from_bytes("globalThis.acceptedValue = 0; (async function transferredContinuation() { const before = 40; const extra = { answer: 2 }; await Promise.resolve(0); globalThis.acceptedValue = before + extra.answer; return acceptedValue; })()")))?;
                    let accepted = JsPromise::from_object(value.as_object().unwrap())?;
                    assert_eq!(accepted.state().unwrap(), PromiseState::Pending);
                    promise = Some(accepted);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    let accepted = promise.take().unwrap();
                    assert_eq!(accepted.state().unwrap(), PromiseState::Pending);
                    Poll::Ready(Ok(accepted.into()))
                }
            }))
        },
        selected.clone(),
    );
    let promise = {
        let host = context.host_access();
        prepared(&mut job, &host);
        assert!(matches!(step(&mut job, &host), JobStep::Pending));
        host.with(|access| {
            assert_eq!(access.realm(), &caller);
            assert_eq!(access.stack_trace().count(), 0);
            Ok(())
        })
        .unwrap();
        let value = completed(&mut job, &host).unwrap();
        host.with(|access| {
            assert_eq!(access.realm(), &caller);
            assert_eq!(access.stack_trace().count(), 0);
            Ok(())
        })
        .unwrap();
        JsPromise::from_object(value.as_object().unwrap()).unwrap()
    };
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    context.run_jobs().unwrap();
    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(42))
    );
    assert_eq!(context.realm(), &caller);
    assert_eq!(context.stack_trace().count(), 0);
    context.enter_realm(selected).unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("acceptedValue")).unwrap(),
        JsValue::new(42)
    );
}
#[test]
fn an_async_job_without_selected_realm_preserves_deliberate_changes_across_polls() {
    let mut context = Context::builder().build().unwrap();
    let selected = context.create_realm().unwrap();
    let expected = selected.clone();
    let mut job = NativeAsyncJob::new(move |_| {
        let mut first = true;
        Ok(ContextTask::from_poll(move |cx, host| {
            if first {
                first = false;
                host.with(|mut access| access.enter_realm(selected.clone()))?;
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                host.with(|access| {
                    assert_eq!(access.realm(), &expected);
                    Ok(())
                })?;
                Poll::Ready(Ok(JsValue::new(42)))
            }
        }))
    });
    let host = context.host_access();
    prepared(&mut job, &host);
    assert!(matches!(step(&mut job, &host), JobStep::Pending));
    let changed = host.with(|access| Ok(access.realm().clone())).unwrap();
    assert_eq!(completed(&mut job, &host).unwrap(), JsValue::new(42));
    assert_eq!(
        host.with(|access| Ok(access.realm().clone())).unwrap(),
        changed
    );
}
#[test]
fn selected_realm_job_rejects_a_real_competing_lease_before_callback_entry() {
    let mut context = Context::builder().build().unwrap();
    let realm = context.realm().clone();
    let called = Rc::new(Cell::new(false));
    let c = called.clone();
    let mut job = NativeAsyncJob::with_realm(
        move |_| {
            c.set(true);
            Ok(ContextTask::from_future(async { Ok(JsValue::new(42)) }))
        },
        realm,
    );
    let host = context.host_access();
    host.with(|mut access| {
        let JobStep::Deferred(error) = step(&mut job, &host) else {
            panic!("a competing lease must defer the factory")
        };
        assert_eq!(error.as_engine(), Some(&EngineError::ContextAccessBusy));
        assert!(!called.get());
        assert_eq!(access.eval(Source::from_bytes("40 + 2"))?, JsValue::new(42));
        Ok(())
    })
    .unwrap();
    assert_eq!(
        host.with(|mut access| access.eval(Source::from_bytes("42")))
            .unwrap(),
        JsValue::new(42)
    );
}
#[test]
fn rebinding_the_callers_holder_reference_keeps_the_async_jobs_original_custody() {
    let mut context = Context::builder().build().unwrap();
    let mut peer = Context::builder().build().unwrap();
    let caller = context.realm().clone();
    let selected = context.create_realm().unwrap();
    let mut job = NativeAsyncJob::with_realm(
        |_| {
            let mut first = true;
            Ok(ContextTask::from_poll(move |cx, host| {
                if first {
                    first = false;
                    host.with(|mut access| {
                        access.eval(Source::from_bytes("globalThis.value = 40"))
                    })?;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    Poll::Ready(
                        host.with(|mut access| access.eval(Source::from_bytes("value + 2"))),
                    )
                }
            }))
        },
        selected.clone(),
    );
    {
        let host = context.host_access();
        let independent = peer.host_access();
        let mut local = &host;
        prepared(&mut job, local);
        local = &independent;
        local
            .with(|mut access| access.eval(Source::from_bytes("globalThis.value = 7")))
            .unwrap();
        assert!(matches!(step(&mut job, &host), JobStep::Pending));
        assert_eq!(
            local
                .with(|mut access| access.eval(Source::from_bytes("value")))
                .unwrap(),
            JsValue::new(7)
        );
        assert_eq!(
            host.with(|access| Ok(access.realm().clone())).unwrap(),
            caller
        );
        assert_eq!(completed(&mut job, &host).unwrap(), JsValue::new(42));
        assert_eq!(
            host.with(|access| Ok(access.realm().clone())).unwrap(),
            caller
        );
    }
    context.enter_realm(selected).unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("value")).unwrap(),
        JsValue::new(40)
    );
    assert_eq!(
        peer.eval(Source::from_bytes("value")).unwrap(),
        JsValue::new(7)
    );
}
fn native_plain(
    this: JsValue,
    args: Vec<JsValue>,
    mut access: ContextAccess<'_>,
) -> JsResult<ContextTask<JsValue>> {
    access.eval(Source::from_bytes("factories += 1"))?;
    let mut first = true;
    Ok(ContextTask::from_poll(move |cx, host| {
        host.with(|mut access| access.eval(Source::from_bytes("polls += 1")))?;
        if first {
            first = false;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(Ok(JsValue::new(
                this.as_number().unwrap() + args[0].as_number().unwrap(),
            )))
        }
    }))
}
fn native_captured(
    _: JsValue,
    args: Vec<JsValue>,
    capture: i32,
    mut access: ContextAccess<'_>,
) -> JsResult<ContextTask<JsValue>> {
    access.eval(Source::from_bytes("factories += 1"))?;
    let mut first = true;
    Ok(ContextTask::from_poll(move |cx, host| {
        host.with(|mut access| access.eval(Source::from_bytes("polls += 1")))?;
        if first {
            first = false;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(Ok(JsValue::new(
                f64::from(capture) + args[0].as_number().unwrap(),
            )))
        }
    }))
}

fn promise_context() -> Context {
    Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .build()
        .unwrap()
}

#[test]
fn native_async_pointer_and_capture_factories_and_polls_can_take_short_leases() {
    let mut context = promise_context();
    let caller = context.realm().clone();
    let selected = context.create_realm().unwrap();
    context.enter_realm(selected.clone()).unwrap();
    context
        .eval(Source::from_bytes(
            "globalThis.factories = 0; globalThis.polls = 0",
        ))
        .unwrap();
    let plain = NativeFunction::from_async_fn(native_plain);
    let captured = NativeFunction::from_async_fn_with_captures(native_captured, 35);
    let plain = JsPromise::from_object(
        plain
            .call(&JsValue::new(40), &[JsValue::new(2)], context.access())
            .unwrap()
            .as_object()
            .unwrap(),
    )
    .unwrap();
    let captured = JsPromise::from_object(
        captured
            .call(&JsValue::undefined(), &[JsValue::new(7)], context.access())
            .unwrap()
            .as_object()
            .unwrap(),
    )
    .unwrap();
    context.enter_realm(caller.clone()).unwrap();
    context.run_jobs().unwrap();
    assert_eq!(context.realm(), &caller);
    assert_eq!(
        plain.state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(42))
    );
    assert_eq!(
        captured.state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(42))
    );
    context.enter_realm(selected).unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("factories")).unwrap(),
        JsValue::new(2)
    );
    assert_eq!(
        context.eval(Source::from_bytes("polls")).unwrap(),
        JsValue::new(4)
    );
}

#[test]
fn async_promise_host_errors_reach_the_driver_without_guest_rejection() {
    let busy: JsError = EngineError::ContextAccessBusy.into();
    let panic: JsError = PanicError::new("async host failure").into();
    let nested: JsError = JsNativeError::aggregate(vec![busy.clone()]).into();
    let mut foreign_context = Context::builder().build().unwrap();
    let foreign = JsError::from_opaque(
        foreign_context
            .eval(Source::from_bytes("new Error('foreign async completion')"))
            .unwrap(),
    )
    .unwrap();
    let mut cases = vec![
        ("direct Busy", busy.clone(), busy.clone()),
        ("direct PanicError", panic.clone(), panic),
        ("local nested Busy", nested, busy),
    ];
    cases.extend(mixed_foreign_busy_cases(foreign));
    for (name, error, expected) in cases {
        let mut context = promise_context();
        let returned = Rc::new(Cell::new(false));
        let callback_returned = returned.clone();
        let promise = JsPromise::from_task(
            move |_| {
                Ok(ContextTask::from_future(async move {
                    callback_returned.set(true);
                    Err(error)
                }))
            },
            context.access(),
        )
        .unwrap();
        context
            .global_object()
            .unwrap()
            .set(
                js_string!("promise"),
                promise.clone(),
                true,
                context.access(),
            )
            .unwrap();
        context
            .eval(Source::from_bytes(
                "globalThis.caught = false; promise.catch(() => { caught = true; })",
            ))
            .unwrap();
        assert_eq!(context.run_jobs().unwrap_err(), expected, "{name}");
        assert!(
            returned.get(),
            "{name}: the host callback must return its actual error"
        );
        assert_eq!(promise.state().unwrap(), PromiseState::Pending);
        assert_eq!(
            context.eval(Source::from_bytes("caught")).unwrap(),
            JsValue::new(false)
        );
        context.run_jobs().unwrap();
        assert_eq!(
            context.eval(Source::from_bytes("caught")).unwrap(),
            JsValue::new(false)
        );
    }
}

fn mixed_foreign_busy_cases(foreign: JsError) -> [(&'static str, JsError, JsError); 3] {
    let busy: JsError = EngineError::ContextAccessBusy.into();
    [
        (
            "foreign aggregate member before Busy",
            JsNativeError::aggregate(vec![foreign.clone(), busy.clone()]).into(),
            busy.clone(),
        ),
        (
            "foreign aggregate member after Busy",
            JsNativeError::aggregate(vec![busy.clone(), foreign.clone()]).into(),
            busy.clone(),
        ),
        (
            "foreign cause and Busy aggregate member",
            JsNativeError::aggregate(vec![busy.clone()])
                .with_cause(foreign)
                .into(),
            busy,
        ),
    ]
}

#[test]
fn async_promise_success_and_catchable_error_use_the_ordinary_guest_paths() {
    let mut context = promise_context();
    let success = JsPromise::from_task(
        |_| Ok(ContextTask::from_future(async { Ok(JsValue::new(42)) })),
        context.access(),
    )
    .unwrap();
    let failure = JsPromise::from_task(
        |_| {
            Ok(ContextTask::from_future(async {
                Err(JsError::from_opaque(JsValue::new(7)).unwrap())
            }))
        },
        context.access(),
    )
    .unwrap();
    context
        .global_object()
        .unwrap()
        .set(
            js_string!("failure"),
            failure.clone(),
            true,
            context.access(),
        )
        .unwrap();
    context
        .eval(Source::from_bytes(
            "globalThis.caught = 0; failure.catch(value => { caught = value; })",
        ))
        .unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        success.state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(42))
    );
    assert_eq!(
        failure.state().unwrap(),
        PromiseState::Rejected(JsValue::new(7))
    );
    assert_eq!(
        context.eval(Source::from_bytes("caught")).unwrap(),
        JsValue::new(7)
    );
}

struct JobDrop {
    receipt: Rc<RefCell<Option<ExecutionReceipt>>>,
    dropped_before_publication: Rc<Cell<bool>>,
}

impl Drop for JobDrop {
    fn drop(&mut self) {
        self.dropped_before_publication
            .set(self.receipt.borrow().as_ref().unwrap().report().is_none());
    }
}

#[test]
fn async_job_polls_share_first_stop_and_dispose_inner_work_before_receipt() {
    let mut control = Context::builder().build().unwrap();
    let baseline = control
        .with_execution(ExecutionOptions::new(256), |mut access| {
            access.eval(Source::from_bytes("globalThis.committed = 20"))?;
            access.eval(Source::from_bytes("while (true) {}"))
        })
        .unwrap();
    assert_eq!(
        baseline.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 256
        }))
    );
    let mut context = Context::builder().build().unwrap();
    let receipt_slot = Rc::new(RefCell::new(None));
    let dropped_before_publication = Rc::new(Cell::new(false));
    let resumed_after_stop = Rc::new(Cell::new(false));
    let resumed_probe = resumed_after_stop.clone();
    let disposed_probe = dropped_before_publication.clone();
    let probe = JobDrop {
        receipt: receipt_slot.clone(),
        dropped_before_publication: dropped_before_publication.clone(),
    };
    let mut task = context
        .execution_task(ExecutionOptions::new(256), async move |access| {
            let holder = access.into_host_access();
            let mut stage = 0;
            let mut job = NativeAsyncJob::new(move |_| {
                Ok(ContextTask::from_poll(move |cx, host| {
                    let _keep = &probe;
                    if stage == 0 {
                        stage = 1;
                        host.with(|mut access| {
                            access.eval(Source::from_bytes("globalThis.committed = 20"))
                        })?;
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                    if stage == 1 {
                        stage = 2;
                        let stopped = host
                            .with(|mut access| access.eval(Source::from_bytes("while (true) {}")))
                            .unwrap_err();
                        assert_eq!(
                            stopped.as_engine(),
                            Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                                limit: 256
                            }))
                        );
                        return Poll::Pending;
                    }
                    resumed_probe.set(true);
                    Poll::Ready(Ok(JsValue::new(42)))
                }))
            });
            prepared(&mut job, &holder);
            assert!(matches!(step(&mut job, &holder), JobStep::Pending));
            yield_once().await;
            let JobStep::Complete(Err(stopped)) = step(&mut job, &holder) else {
                panic!("the stopping second task poll must complete with its original stop")
            };
            assert_eq!(
                stopped.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 256
                }))
            );
            assert!(
                disposed_probe.get(),
                "job disposal precedes outer completion"
            );
            // The outer host future may continue its own Rust code. A later
            // Busy and apparent success must not replace its retained stop.
            holder.with(|_| {
                assert_eq!(
                    holder.with(|_| Ok(())).unwrap_err().as_engine(),
                    Some(&EngineError::ContextAccessBusy)
                );
                Ok(())
            })?;
            Ok(JsValue::new(42))
        })
        .unwrap();
    let receipt = task.receipt();
    *receipt_slot.borrow_mut() = Some(receipt.clone());
    assert!(poll(&mut task).is_pending());
    assert!(receipt.report().is_none());
    let Poll::Ready(outcome) = poll(&mut task) else {
        panic!("the stopping second poll must dispose the job and finish the task")
    };
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 256
        }))
    );
    assert_eq!(outcome.report.instructions().unwrap(), 256);
    assert_eq!(
        outcome.report.scope_work().unwrap(),
        baseline.report.scope_work().unwrap()
    );
    assert!(matches!(
        outcome.report.termination(),
        ExecutionTermination::Completed
    ));
    assert!(!resumed_after_stop.get());
    assert!(dropped_before_publication.get());
    assert!(receipt.report().is_some());
    drop(task);
    assert_eq!(
        context.eval(Source::from_bytes("committed + 22")).unwrap(),
        JsValue::new(42)
    );
}

struct LeasingLoader {
    failure: Option<JsError>,
}

impl ModuleLoader for LeasingLoader {
    fn load_imported_module(
        self: Rc<Self>,
        _: Referrer,
        request: ModuleRequest,
    ) -> ContextTask<Module> {
        assert_eq!(request.specifier(), &js_string!("dependency"));
        let mut first = true;
        ContextTask::from_poll(move |cx, host| {
            if first {
                first = false;
                host.with(|mut access| access.eval(Source::from_bytes("loaderLog += 'f'")))?;
                host.with(|mut access| access.eval(Source::from_bytes("loaderLog += 'p'")))?;
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                host.with(|mut access| access.eval(Source::from_bytes("loaderLog += 'r'")))?;
                if let Some(error) = &self.failure {
                    return Poll::Ready(Err(error.clone()));
                }
                Poll::Ready(host.with(|access| {
                    Module::parse(Source::from_bytes("export const answer = 42"), None, access)
                }))
            }
        })
    }
}

fn loader_context(failure: Option<JsError>) -> Context {
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .module_loader(Rc::new(LeasingLoader { failure }))
        .build()
        .unwrap();
    context
        .eval(Source::from_bytes(
            "globalThis.loaderLog = ''; globalThis.caught = false; globalThis.answer = 0",
        ))
        .unwrap();
    context
}

#[test]
fn source_graph_and_dynamic_import_loader_owned_stages_can_lease() {
    for dynamic in [false, true] {
        let mut context = loader_context(None);
        let promise = if dynamic {
            JsPromise::from_object(context.eval(Source::from_bytes("import('dependency').then(ns => { answer = ns.answer; }, () => { caught = true; })")).unwrap().as_object().unwrap()).unwrap()
        } else {
            let module = Module::parse(
                Source::from_bytes(
                    "import { answer as value } from 'dependency'; globalThis.answer = value",
                ),
                None,
                context.access(),
            )
            .unwrap();
            module.load_link_evaluate(context.access()).unwrap()
        };
        context.run_jobs().unwrap();
        assert!(matches!(
            promise.state().unwrap(),
            PromiseState::Fulfilled(_)
        ));
        assert_eq!(
            context.eval(Source::from_bytes("answer")).unwrap(),
            JsValue::new(42)
        );
        assert_eq!(
            context.eval(Source::from_bytes("loaderLog")).unwrap(),
            JsValue::new(js_string!("fpr"))
        );
        assert_eq!(
            context.eval(Source::from_bytes("caught")).unwrap(),
            JsValue::new(false)
        );
    }
}

#[test]
fn source_graph_and_dynamic_import_loader_host_errors_do_not_reject_in_guest() {
    let busy: JsError = EngineError::ContextAccessBusy.into();
    let nested: JsError = JsNativeError::aggregate(vec![busy.clone()]).into();
    let mut foreign_context = Context::builder().build().unwrap();
    let foreign = JsError::from_opaque(
        foreign_context
            .eval(Source::from_bytes("new Error('foreign loader completion')"))
            .unwrap(),
    )
    .unwrap();
    for dynamic in [false, true] {
        let mut cases = vec![
            ("direct Busy", busy.clone(), busy.clone()),
            ("local nested Busy", nested.clone(), busy.clone()),
        ];
        cases.extend(mixed_foreign_busy_cases(foreign.clone()));
        for (name, error, expected) in cases {
            let mut context = loader_context(Some(error.clone()));
            let promise = if dynamic {
                JsPromise::from_object(
                    context
                        .eval(Source::from_bytes("import('dependency')"))
                        .unwrap()
                        .as_object()
                        .unwrap(),
                )
                .unwrap()
            } else {
                let module = Module::parse(
                    Source::from_bytes("import 'dependency'"),
                    None,
                    context.access(),
                )
                .unwrap();
                module.load(context.access()).unwrap()
            };
            context
                .global_object()
                .unwrap()
                .set(
                    js_string!("loaded"),
                    promise.clone(),
                    true,
                    context.access(),
                )
                .unwrap();
            context
                .eval(Source::from_bytes("loaded.catch(() => { caught = true; })"))
                .unwrap();
            assert_eq!(
                context.run_jobs().unwrap_err(),
                expected,
                "{name}; dynamic={dynamic}"
            );
            assert_eq!(
                context.eval(Source::from_bytes("loaderLog")).unwrap(),
                JsValue::new(js_string!("fpr")),
                "{name}: the actual loader factory and resumed future must run"
            );
            assert_eq!(promise.state().unwrap(), PromiseState::Pending);
            assert_eq!(
                context.eval(Source::from_bytes("caught")).unwrap(),
                JsValue::new(false)
            );
            context.run_jobs().unwrap();
            assert_eq!(
                context.eval(Source::from_bytes("caught")).unwrap(),
                JsValue::new(false)
            );
        }
    }
}
