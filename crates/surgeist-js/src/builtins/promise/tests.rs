use crate::{TestAction, run_test_actions};
use indoc::indoc;

#[test]
fn pending_reaction_cycle_is_reclaimed_when_its_promise_is_dropped() {
    use crate::{Context, JsObject, NativeFunction, object::builtins::JsPromise};

    let context = &mut Context::default();
    let (promise, resolvers) = JsPromise::new_pending(context.access());
    let handler = NativeFunction::from_fn_with_captures(
        |_, _, promise, _| Ok(promise.clone().into()),
        JsObject::from(promise.clone()),
    )
    .to_js_function(context.realm())
    .unwrap();
    let promise_edge = promise.edge();
    let handler_edge = handler.edge();
    let dependent_edge = promise
        .then(Some(handler), None, context.access())
        .unwrap()
        .edge();

    drop(resolvers);
    context.runtime().collect().unwrap();
    assert!(handler_edge.is_live().unwrap());
    assert!(dependent_edge.is_live().unwrap());

    drop(promise);
    context.runtime().collect().unwrap();
    assert!(!promise_edge.is_live().unwrap());
    assert!(!handler_edge.is_live().unwrap());
    assert!(!dependent_edge.is_live().unwrap());
}

#[test]
fn every_reaction_survives_collection_during_job_enqueue() {
    use crate::{
        Context, JsObject, JsResult, JsValue, NativeFunction,
        builtins::promise::PromiseState,
        job::{Job, JobAcceptance, JobExecutor, JobPublicationError, SimpleJobExecutor},
        object::builtins::JsPromise,
    };
    use std::rc::Rc;

    #[derive(Default)]
    struct CollectingExecutor(Rc<SimpleJobExecutor>);

    impl JobExecutor for CollectingExecutor {
        fn enqueue_job(
            self: Rc<Self>,
            job: Job,
            context: crate::ContextAccess<'_>,
        ) -> Result<JobAcceptance, JobPublicationError> {
            context.runtime().collect().unwrap();
            self.0.clone().enqueue_job(job, context)
        }

        fn run_jobs(self: Rc<Self>, context: crate::ContextAccess<'_>) -> JsResult<()> {
            self.0.clone().run_jobs(context)
        }
    }

    let context = &mut Context::builder()
        .job_executor(Rc::new(CollectingExecutor::default()))
        .build()
        .unwrap();
    let (promise, resolvers) = JsPromise::new_pending(context.access());
    let first = promise.then(None, None, context.access()).unwrap();
    let captured = JsObject::with_null_proto(context.runtime());
    let captured_edge = captured.edge();
    let handler =
        NativeFunction::from_fn_with_captures(|_, _, value, _| Ok(value.clone().into()), captured)
            .to_js_function(context.realm())
            .unwrap();
    let second = promise.then(Some(handler), None, context.access()).unwrap();

    let result = JsObject::with_null_proto(context.runtime());
    let result_edge = result.edge();
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[result.into()], context.access())
        .unwrap();
    drop(promise);
    drop(resolvers);
    context.runtime().collect().unwrap();
    context.run_jobs().unwrap();
    context.runtime().collect().unwrap();

    let expected_result = result_edge
        .upgrade()
        .unwrap()
        .expect("live fulfilled value");
    let expected_capture = captured_edge
        .upgrade()
        .unwrap()
        .expect("live captured value");
    assert_eq!(
        first.state().unwrap(),
        PromiseState::Fulfilled(expected_result.into())
    );
    assert_eq!(
        second.state().unwrap(),
        PromiseState::Fulfilled(expected_capture.into())
    );
}

#[test]
fn rust_promise_handlers_must_belong_to_the_receiving_runtime() {
    use crate::{Context, JsValue, NativeFunction, Runtime, object::builtins::JsPromise};

    let context = &mut Context::builder()
        .runtime(Runtime::default())
        .build()
        .unwrap();
    let foreign = Context::builder()
        .runtime(Runtime::default())
        .build()
        .unwrap();
    let (promise, _) = JsPromise::new_pending(context.access());
    let handler = NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::undefined()))
        .to_js_function(foreign.realm())
        .unwrap();

    assert!(
        promise
            .then(Some(handler.clone()), None, context.access())
            .is_err()
    );
    assert!(promise.catch(handler.clone(), context.access()).is_err());
    assert!(promise.finally(handler, context.access()).is_err());
    context.runtime().collect().unwrap();
}

fn context_with_foreign_job_callback_data(invalid_at: usize) -> crate::Context {
    use crate::{
        Context, JsObject, Runtime, context::HostHooks, job::JobCallback, object::JsFunction,
    };
    use std::{cell::Cell, rc::Rc};

    struct Hooks {
        foreign: JsObject,
        call_count: Cell<usize>,
        invalid_at: usize,
    }

    impl HostHooks for Hooks {
        fn make_job_callback(
            &self,
            callback: JsFunction,
            _: crate::ContextAccess<'_>,
        ) -> JobCallback {
            let call = self.call_count.get();
            self.call_count.set(call + 1);
            if call == self.invalid_at {
                JobCallback::new(callback, self.foreign.clone())
            } else {
                JobCallback::new(callback, ())
            }
        }
    }

    let foreign = Runtime::default();
    Context::builder()
        .runtime(Runtime::default())
        .host_hooks(Rc::new(Hooks {
            foreign: JsObject::with_null_proto(&foreign),
            call_count: Cell::new(0),
            invalid_at,
        }))
        .build()
        .unwrap()
}

#[test]
fn pending_promise_rejects_foreign_job_callback_data_before_installing_either_reaction() {
    use crate::{JsValue, Source};

    let context = &mut context_with_foreign_job_callback_data(1);
    let error = context
        .eval(Source::from_bytes(
            "let calls = 0; let resolve;
            const promise = new Promise(r => resolve = r);
            promise.then(() => calls += 100, () => calls += 100);",
        ))
        .unwrap_err();
    assert!(error.try_native(context.access()).unwrap().is_type());
    context.runtime().collect().unwrap();

    context
        .eval(Source::from_bytes(
            "promise.then(() => ++calls); resolve();",
        ))
        .unwrap();
    context.run_jobs().unwrap();
    context.runtime().collect().unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("calls")).unwrap(),
        JsValue::from(1)
    );
}

#[test]
fn await_can_catch_job_callback_admission_error_and_use_its_frame() {
    use crate::{JsValue, Source};

    let context = &mut context_with_foreign_job_callback_data(1);
    context
        .eval(Source::from_bytes(
            "let caught = 0;
            const pending = new Promise(() => {});
            async function observe(value) {
                const local = value + 1;
                try {
                    await pending;
                } catch (error) {
                    caught = error instanceof TypeError ? local : -1;
                }
            }
            observe(41);",
        ))
        .unwrap();

    context.run_jobs().unwrap();
    context.runtime().collect().unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("caught")).unwrap(),
        JsValue::from(42)
    );
}

#[test]
fn thenable_resolution_rejects_foreign_job_callback_data() {
    use crate::{JsValue, Source};

    let context = &mut context_with_foreign_job_callback_data(0);
    context
        .eval(Source::from_bytes(
            "let ran = false; let reason;
            Promise.resolve({ then() { ran = true; } }).catch(error => reason = error);",
        ))
        .unwrap();

    context.run_jobs().unwrap();
    context.runtime().collect().unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes("!ran && reason instanceof TypeError"))
            .unwrap(),
        JsValue::from(true)
    );
}

#[test]
fn promise() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                    let count = 0;
                    const promise = new Promise((resolve, reject) => {
                        count += 1;
                        resolve(undefined);
                    }).then((_) => (count += 1));
                    count += 1;
                "#}),
        TestAction::assert_eq("count", 2),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("count", 3),
    ]);
}

#[test]
fn promise_all_resolves_values() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var values = [];
            var p = Promise.all([Promise.resolve(1), Promise.resolve(2)]);
            p.then(v => { values = v; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("values.length", 2),
        TestAction::assert_eq("values[0]", 1),
        TestAction::assert_eq("values[1]", 2),
    ]);
}

#[test]
fn promise_all_rejects() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var err = null;
            var p = Promise.all([Promise.resolve(1), Promise.reject(2)]);
            p.catch(e => { err = e; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("err", 2),
    ]);
}

#[test]
fn promise_any_resolves_first_success() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var val = null;
            var p = Promise.any([Promise.reject(1), Promise.resolve(2)]);
            p.then(v => { val = v; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("val", 2),
    ]);
}

#[test]
fn promise_all_settled_resolves_results() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var values = [];
            Promise.allSettled([
                Promise.resolve(1),
                Promise.reject(2)
            ]).then(results => {
                values = [
                    results[0].status,
                    results[0].value,
                    results[1].status,
                    results[1].reason
                ];
            });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("values[0]", crate::js_string!("fulfilled")),
        TestAction::assert_eq("values[1]", 1),
        TestAction::assert_eq("values[2]", crate::js_string!("rejected")),
        TestAction::assert_eq("values[3]", 2),
    ]);
}

#[test]
fn promise_race_resolves_first() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            var val = null;
            Promise.race([
                Promise.resolve(10),
                Promise.resolve(20)
            ]).then(v => { val = v; });
        "#}),
        TestAction::inspect_context(|ctx| ctx.run_jobs().unwrap()),
        TestAction::assert_eq("val", 10),
    ]);
}

#[test]
fn foreign_async_outcomes_reject_instead_of_leaving_a_pending_promise() {
    use crate::{Context, JsObject, Runtime, object::JsPromise};

    let mut context = Context::default();
    let promise = JsPromise::from_task(
        |_| {
            Ok(crate::job::ContextTask::from_future(async {
                Ok(JsObject::with_null_proto(&Runtime::default()).into())
            }))
        },
        context.access(),
    )
    .unwrap();
    context.run_jobs().unwrap();
    assert!(promise.state().unwrap().as_rejected().is_some());
}

#[test]
fn promise_executor_error_materialization_refuses_a_foreign_nested_cause() {
    use crate::{Context, JsError, JsNativeError, JsObject, Runtime, object::JsPromise};

    let context = &mut Context::default();
    let foreign = JsObject::with_null_proto(&Runtime::default());
    let inner = JsNativeError::range().with_cause(JsError::from_opaque(foreign.into()).unwrap());
    let error = JsNativeError::error().with_cause(inner).into();

    let refusal = JsPromise::new(|_, _| Err(error), context.access()).unwrap_err();
    assert!(refusal.as_native().is_some_and(JsNativeError::is_type));
}

#[test]
fn promise_reject_error_materialization_refuses_a_foreign_nested_cause() {
    use crate::{Context, JsError, JsNativeError, JsObject, Runtime, object::JsPromise};

    let context = &mut Context::default();
    let foreign = JsObject::with_null_proto(&Runtime::default());
    let inner = JsNativeError::range().with_cause(JsError::from_opaque(foreign.into()).unwrap());
    let error = JsNativeError::error().with_cause(inner);

    let refusal = JsPromise::reject(error, context.access()).unwrap_err();
    assert!(refusal.as_native().is_some_and(JsNativeError::is_type));
}

#[test]
fn promise_reject_error_materialization_preserves_the_original_engine_error() {
    use crate::{Context, JsError, error::PanicError, object::JsPromise};

    let context = &mut Context::default();
    let error = JsError::from(PanicError::new("engine failure"));

    assert_eq!(
        JsPromise::reject(error.clone(), context.access()).unwrap_err(),
        error
    );
}
