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
    Context, HostContext, JsError, JsNativeError, JsResult, JsValue, Module, NativeFunction,
    Source,
    builtins::promise::PromiseState,
    error::{EngineError, PanicError, RuntimeLimitError},
    execution::{ExecutionOptions, ExecutionReceipt, ExecutionTermination},
    job::{BoxedFuture, NativeAsyncJob, SimpleJobExecutor},
    js_string,
    module::{ModuleLoader, ModuleRequest, Referrer},
    object::builtins::JsPromise,
};

fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
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
    let mut context = Context::default();
    let original_realm = context.realm().clone();
    let selected_realm = context.create_realm().unwrap();
    assert_ne!(selected_realm, original_realm);
    let expected_realm = selected_realm.clone();
    let job = NativeAsyncJob::with_realm(
        async move |context| {
            context
                .with(|access| {
                    assert_eq!(access.realm(), &expected_realm);
                    Ok(())
                })
                .unwrap();
            panic!("async job poll panic");
        },
        selected_realm,
    );

    let panic = {
        let holder = context.host_access();
        let mut future = job.call(&holder);
        catch_unwind(AssertUnwindSafe(|| poll(&mut future)))
            .expect_err("the async callback panic must propagate to its host caller")
    };
    assert_eq!(panic.downcast_ref::<&str>(), Some(&"async job poll panic"));
    assert_eq!(
        context.eval(Source::from_bytes("40 + 2")).unwrap(),
        JsValue::new(42)
    );
    assert_eq!(context.realm(), &original_realm);
}

#[test]
fn interleaved_async_jobs_restore_each_selected_realm_after_pending_and_ready() {
    let mut context = Context::default();
    let original = context.realm().clone();
    let first = context.create_realm().unwrap();
    let second = context.create_realm().unwrap();
    let make_job = |realm: surgeist_js::realm::Realm, value| {
        let expected = realm.clone();
        NativeAsyncJob::with_realm(
            async move |host| {
                host.with(|mut access| {
                    assert_eq!(access.realm(), &expected);
                    assert_eq!(access.eval(Source::from_bytes("40 + 2"))?, JsValue::new(42));
                    let busy = host.with(|_| Ok(())).unwrap_err();
                    assert_eq!(busy.as_engine(), Some(&EngineError::ContextAccessBusy));
                    Ok(())
                })?;
                yield_once().await;
                host.with(|access| {
                    assert_eq!(access.realm(), &expected);
                    Ok(JsValue::new(value))
                })
            },
            realm,
        )
    };
    let holder = context.host_access();
    let mut first = make_job(first, 20).call(&holder);
    let mut second = make_job(second, 22).call(&holder);
    // Creation and each actual poll leave the caller's realm in place.
    let assert_restored = || {
        holder
            .with(|access| {
                assert_eq!(access.realm(), &original);
                Ok(())
            })
            .unwrap();
    };
    assert_restored();
    assert!(poll(&mut first).is_pending());
    assert_restored();
    assert!(poll(&mut second).is_pending());
    assert_restored();
    assert_eq!(poll(&mut first), Poll::Ready(Ok(JsValue::new(20))));
    assert_restored();
    assert_eq!(poll(&mut second), Poll::Ready(Ok(JsValue::new(22))));
    assert_restored();
}

#[test]
fn selected_realm_async_polls_preserve_a_live_guest_callers_frames_and_return() {
    let mut context = Context::default();
    let selected = context.create_realm().unwrap();
    context
        .register_global_builtin_callable(
            js_string!("nestedJob"),
            0,
            NativeFunction::from_fn_with_captures(
                |_, _, selected, access| {
                    let caller_realm = access.realm().clone();
                    let baseline = access
                        .stack_trace()
                        .map(|frame| frame.position())
                        .collect::<Vec<_>>();
                    assert!(
                        !baseline.is_empty(),
                        "the native caller must have live guest frames"
                    );
                    let expected_realm = selected.clone();
                    let job = NativeAsyncJob::with_realm(
                        async move |host| {
                            host.with(|mut access| {
                                assert_eq!(access.realm(), &expected_realm);
                                access.eval(Source::from_bytes("40"))
                            })?;
                            yield_once().await;
                            host.with(|mut access| {
                                assert_eq!(access.realm(), &expected_realm);
                                access.eval(Source::from_bytes("40 + 2"))
                            })
                        },
                        selected.clone(),
                    );
                    let holder = access.into_host_access();
                    let mut future = job.call(&holder);
                    let assert_caller_preserved = || {
                        holder
                            .with(|access| {
                                assert_eq!(access.realm(), &caller_realm);
                                assert_eq!(
                                    access
                                        .stack_trace()
                                        .map(|frame| frame.position())
                                        .collect::<Vec<_>>(),
                                    baseline
                                );
                                Ok(())
                            })
                            .unwrap();
                    };
                    assert_caller_preserved();
                    assert!(poll(&mut future).is_pending());
                    assert_caller_preserved();
                    let Poll::Ready(result) = poll(&mut future) else {
                        panic!("the second controlled poll must complete the nested job")
                    };
                    assert_eq!(result?, JsValue::new(42));
                    assert_caller_preserved();
                    Ok(JsValue::new(42))
                },
                selected,
            ),
        )
        .unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "function liveCaller() { const answer = nestedJob(); return answer; } liveCaller()",
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
    let expected_realm = selected.clone();
    let job = NativeAsyncJob::with_realm(
        async move |host| {
            let value = host.with(|mut access| {
                assert_eq!(access.realm(), &expected_realm);
                access.eval(Source::from_bytes(
                    "globalThis.acceptedValue = 0;
                     (async function transferredContinuation() {
                         const before = 40;
                         const extra = { answer: 2 };
                         await Promise.resolve(0);
                         globalThis.acceptedValue = before + extra.answer;
                         return acceptedValue;
                     })()",
                ))
            })?;
            let promise = JsPromise::from_object(value.as_object().unwrap())?;
            assert_eq!(promise.state(), PromiseState::Pending);
            yield_once().await;
            host.with(|access| {
                assert_eq!(access.realm(), &expected_realm);
                Ok(())
            })?;
            assert_eq!(promise.state(), PromiseState::Pending);
            Ok(promise.into())
        },
        selected.clone(),
    );
    let promise = {
        let holder = context.host_access();
        let mut future = job.call(&holder);
        assert!(poll(&mut future).is_pending());
        holder
            .with(|access| {
                assert_eq!(access.realm(), &caller);
                assert_eq!(access.stack_trace().count(), 0);
                Ok(())
            })
            .unwrap();
        let Poll::Ready(result) = poll(&mut future) else {
            panic!("the second controlled poll must transfer the accepted promise")
        };
        holder
            .with(|access| {
                assert_eq!(access.realm(), &caller);
                assert_eq!(access.stack_trace().count(), 0);
                Ok(())
            })
            .unwrap();
        JsPromise::from_object(result.unwrap().as_object().unwrap()).unwrap()
    };
    assert_eq!(promise.state(), PromiseState::Pending);
    context.run_jobs().unwrap();
    assert_eq!(promise.state(), PromiseState::Fulfilled(JsValue::new(42)));
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
    let mut context = Context::default();
    let selected = context.create_realm().unwrap();
    let expected = selected.clone();
    let job = NativeAsyncJob::new(async move |host| {
        host.with(|mut access| access.enter_realm(selected))?;
        yield_once().await;
        host.with(|access| {
            assert_eq!(access.realm(), &expected);
            Ok(JsValue::new(42))
        })
    });
    let holder = context.host_access();
    let mut future = job.call(&holder);
    assert!(poll(&mut future).is_pending());
    let changed = holder.with(|access| Ok(access.realm().clone())).unwrap();
    assert_eq!(poll(&mut future), Poll::Ready(Ok(JsValue::new(42))));
    assert_eq!(
        holder.with(|access| Ok(access.realm().clone())).unwrap(),
        changed
    );
}

#[test]
fn selected_realm_job_rejects_a_real_competing_lease_before_callback_entry() {
    let mut context = Context::default();
    let realm = context.realm().clone();
    let called = Rc::new(Cell::new(false));
    let callback_called = called.clone();
    let job = NativeAsyncJob::with_realm(
        async move |_| {
            callback_called.set(true);
            Ok(JsValue::new(42))
        },
        realm,
    );
    let holder = context.host_access();
    holder
        .with(|mut access| {
            let mut future = job.call(&holder);
            let Poll::Ready(Err(error)) = poll(&mut future) else {
                panic!("a real competing lease must reject the job");
            };
            assert_eq!(error.as_engine(), Some(&EngineError::ContextAccessBusy));
            assert!(!called.get());
            assert_eq!(access.eval(Source::from_bytes("40 + 2"))?, JsValue::new(42));
            Ok(())
        })
        .unwrap();
    assert_eq!(
        holder
            .with(|mut access| access.eval(Source::from_bytes("42")))
            .unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn rebinding_the_callers_holder_reference_keeps_the_async_jobs_original_custody() {
    let mut context = Context::default();
    let mut independent = Context::default();
    let caller = context.realm().clone();
    let selected = context.create_realm().unwrap();
    let job = NativeAsyncJob::with_realm(
        async move |host| {
            host.with(|mut access| access.eval(Source::from_bytes("globalThis.value = 40")))?;
            yield_once().await;
            host.with(|mut access| access.eval(Source::from_bytes("value + 2")))
        },
        selected.clone(),
    );
    {
        let holder = context.host_access();
        let peer = independent.host_access();
        let mut local = &holder;
        let mut future = job.call(local);
        local = &peer;
        local
            .with(|mut access| access.eval(Source::from_bytes("globalThis.value = 7")))
            .unwrap();
        assert!(poll(&mut future).is_pending());
        assert_eq!(
            local
                .with(|mut access| access.eval(Source::from_bytes("value")))
                .unwrap(),
            JsValue::new(7)
        );
        assert_eq!(
            holder.with(|access| Ok(access.realm().clone())).unwrap(),
            caller
        );
        assert_eq!(poll(&mut future), Poll::Ready(Ok(JsValue::new(42))));
        assert_eq!(
            holder.with(|access| Ok(access.realm().clone())).unwrap(),
            caller
        );
    }
    context.enter_realm(selected).unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("value")).unwrap(),
        JsValue::new(40)
    );
    assert_eq!(
        independent.eval(Source::from_bytes("value")).unwrap(),
        JsValue::new(7)
    );
}

fn native_plain<'a>(
    this: &'a JsValue,
    args: &'a [JsValue],
    host: &'a HostContext<'_>,
) -> BoxedFuture<'a> {
    host.with(|mut access| access.eval(Source::from_bytes("factories += 1")))
        .unwrap();
    Box::pin(async move {
        host.with(|mut access| access.eval(Source::from_bytes("polls += 1")))?;
        yield_once().await;
        host.with(|mut access| access.eval(Source::from_bytes("polls += 1")))?;
        Ok(JsValue::new(
            this.as_number().unwrap() + args[0].as_number().unwrap(),
        ))
    })
}

fn native_captured<'a>(
    _: &'a JsValue,
    args: &'a [JsValue],
    capture: &'a i32,
    host: &'a HostContext<'_>,
) -> BoxedFuture<'a> {
    host.with(|mut access| access.eval(Source::from_bytes("factories += 1")))
        .unwrap();
    Box::pin(async move {
        host.with(|mut access| access.eval(Source::from_bytes("polls += 1")))?;
        yield_once().await;
        host.with(|mut access| access.eval(Source::from_bytes("polls += 1")))?;
        Ok(JsValue::new(
            f64::from(*capture) + args[0].as_number().unwrap(),
        ))
    })
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
    assert_eq!(plain.state(), PromiseState::Fulfilled(JsValue::new(42)));
    assert_eq!(captured.state(), PromiseState::Fulfilled(JsValue::new(42)));
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
    for (error, expected) in [
        (busy.clone(), busy.clone()),
        (panic.clone(), panic),
        (nested, busy),
    ] {
        let mut context = promise_context();
        let promise = JsPromise::from_async_fn(async move |_| Err(error), context.access());
        context
            .global_object()
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
        assert_eq!(context.run_jobs().unwrap_err(), expected);
        assert_eq!(promise.state(), PromiseState::Pending);
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

#[test]
fn async_promise_success_and_catchable_error_use_the_ordinary_guest_paths() {
    let mut context = promise_context();
    let success = JsPromise::from_async_fn(async |_| Ok(JsValue::new(42)), context.access());
    let failure = JsPromise::from_async_fn(
        async |_| Err(JsError::from_opaque(JsValue::new(7))),
        context.access(),
    );
    context
        .global_object()
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
    assert_eq!(success.state(), PromiseState::Fulfilled(JsValue::new(42)));
    assert_eq!(failure.state(), PromiseState::Rejected(JsValue::new(7)));
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
    let mut control = Context::default();
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
    let mut context = Context::default();
    let receipt_slot = Rc::new(RefCell::new(None));
    let dropped_before_publication = Rc::new(Cell::new(false));
    let probe = JobDrop {
        receipt: receipt_slot.clone(),
        dropped_before_publication: dropped_before_publication.clone(),
    };
    let mut task = context
        .execution_task(ExecutionOptions::new(256), async move |access| {
            let holder = access.into_host_access();
            let job = NativeAsyncJob::new(async move |host| {
                let _probe = probe;
                host.with(|mut access| {
                    access.eval(Source::from_bytes("globalThis.committed = 20"))
                })?;
                yield_once().await;
                let stopped = host
                    .with(|mut access| access.eval(Source::from_bytes("while (true) {}")))
                    .unwrap_err();
                assert_eq!(
                    stopped.as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                        limit: 256
                    }))
                );
                yield_once().await;
                // Busy and an apparent success must not replace the retained stop.
                host.with(|_| {
                    assert_eq!(
                        host.with(|_| Ok(())).unwrap_err().as_engine(),
                        Some(&EngineError::ContextAccessBusy)
                    );
                    Ok(())
                })?;
                Ok(JsValue::new(42))
            });
            job.call(&holder).await
        })
        .unwrap();
    let receipt = task.receipt();
    *receipt_slot.borrow_mut() = Some(receipt.clone());
    assert!(poll(&mut task).is_pending());
    assert!(receipt.report().is_none());
    assert!(poll(&mut task).is_pending());
    assert!(receipt.report().is_none());
    let Poll::Ready(outcome) = poll(&mut task) else {
        panic!("three controlled polls must finish the job")
    };
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 256
        }))
    );
    assert_eq!(outcome.report.instructions(), 256);
    assert_eq!(outcome.report.scope_work(), baseline.report.scope_work());
    assert!(matches!(
        outcome.report.termination(),
        ExecutionTermination::Completed
    ));
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
        host: &HostContext<'_>,
    ) -> impl Future<Output = JsResult<Module>> {
        assert_eq!(request.specifier(), &js_string!("dependency"));
        // This runs at future creation, before its first poll.
        let factory = host.with(|mut access| access.eval(Source::from_bytes("loaderLog += 'f'")));
        async move {
            factory?;
            host.with(|mut access| access.eval(Source::from_bytes("loaderLog += 'p'")))?;
            yield_once().await;
            host.with(|mut access| access.eval(Source::from_bytes("loaderLog += 'r'")))?;
            if let Some(error) = &self.failure {
                return Err(error.clone());
            }
            host.with(|access| {
                Module::parse(Source::from_bytes("export const answer = 42"), None, access)
            })
        }
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
fn source_graph_and_dynamic_import_loader_factories_and_polls_can_lease() {
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
            module.load_link_evaluate(context.access())
        };
        context.run_jobs().unwrap();
        assert!(matches!(promise.state(), PromiseState::Fulfilled(_)));
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
    for dynamic in [false, true] {
        for (error, expected) in [(busy.clone(), busy.clone()), (nested.clone(), busy.clone())] {
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
                module.load(context.access())
            };
            context
                .global_object()
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
            assert_eq!(context.run_jobs().unwrap_err(), expected);
            assert_eq!(promise.state(), PromiseState::Pending);
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
