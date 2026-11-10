use surgeist_js::{
    Context, JsValue, NativeFunction, Source,
    error::{EngineError, EngineResourceError, ObjectAccessError},
    interop::{JsAll, TryFromJsArgument},
};

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    builder.build().unwrap()
}

#[test]
fn argument_list_conversion_preserves_host_refusal_instead_of_stopping_successfully() {
    let mut context = context();
    let this = JsValue::undefined();
    let value = context.eval(Source::from_bytes("[1, 2]")).unwrap();
    let object = value.as_object().unwrap();
    let held = object.borrow_mut().unwrap();
    let args = [value.clone()];
    let error =
        JsAll::<Vec<i32>>::try_from_js_argument(&this, &args, context.access()).unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
    );
    drop(held);

    let args = [value, JsValue::new(42)];
    let (accepted, remaining) =
        JsAll::<Vec<i32>>::try_from_js_argument(&this, &args, context.access()).unwrap();
    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted.iter().next(), Some(&vec![1, 2]));
    assert_eq!(remaining, &args[1..]);
}

#[test]
fn generator_resume_keeps_the_first_host_failure_when_post_body_access_refuses() {
    for abrupt in [false, true] {
        let mut context = context();
        context
            .register_global_callable(
                surgeist_js::js_string!("refuseAndRetainAlias"),
                1,
                NativeFunction::from_fn_ptr(|_, args, _| {
                    let object = args[0].as_object().unwrap();
                    // No root or allocation is leaked. The deliberately retained
                    // borrow lasts until the fixture's runtime destroys its object.
                    std::mem::forget(object.borrow().unwrap());
                    Err(EngineError::Resource(EngineResourceError::Allocation).into())
                }),
            )
            .unwrap();
        let source = if abrupt {
            "globalThis.iterator = (function* () { try { yield 0; } finally { refuseAndRetainAlias(iterator); } })(); iterator.next(); iterator.return();"
        } else {
            "globalThis.iterator = (function* () { refuseAndRetainAlias(iterator); })(); iterator.next();"
        };
        let error = context.eval(Source::from_bytes(source)).unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::Resource(EngineResourceError::Allocation))
        );
    }
}

use std::{cell::RefCell, ops::Deref, rc::Rc};

use surgeist_js::{
    ContextAccess, JsError, JsObject, JsResult,
    execution::ExecutionOptions,
    job::{Job, JobAcceptance, JobExecutor, JobPublicationError},
    js_string,
    object::builtins::{JsFunction, JsPromise},
};

fn finite_eval(context: &mut Context, source: &str) -> JsResult<JsValue> {
    context
        .with_execution(ExecutionOptions::new(100_000), |mut access| {
            access.eval(Source::from_bytes(source))
        })
        .unwrap()
        .result
}

fn borrowed(error: &JsError) {
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
    );
}

fn close_refusal(block_iterator: bool) {
    let mut context = context();
    context
        .register_global_callable(
            js_string!("mapperRetainsTargetAndThrows"),
            1,
            NativeFunction::from_fn_ptr(|_, _, mut access| {
                let target = access
                    .global_object()
                    .unwrap()
                    .get(js_string!("blockedTarget"), access.reborrow())?
                    .as_object()
                    .unwrap();
                std::mem::forget(target.borrow_mut().unwrap());
                Err(JsError::from_opaque(17.into())?)
            }),
        )
        .unwrap();
    finite_eval(
        &mut context,
        "globalThis.closed = 0; globalThis.guestCaught = false;
         globalThis.iterator = {
             [Symbol.iterator]() { return this; },
             next() { return { value: 1, done: false }; },
             return() { closed += 1; return {}; }
         };",
    )
    .unwrap();
    finite_eval(
        &mut context,
        if block_iterator {
            "globalThis.blockedTarget = iterator"
        } else {
            "globalThis.blockedTarget = iterator.return"
        },
    )
    .unwrap();
    let error = finite_eval(
        &mut context,
        "try { Array.from(iterator, mapperRetainsTargetAndThrows); }
         catch (error) { guestCaught = error; }",
    )
    .unwrap_err();
    borrowed(&error);
    assert_eq!(finite_eval(&mut context, "closed").unwrap(), 0.into());
    assert_eq!(
        finite_eval(&mut context, "guestCaught").unwrap(),
        false.into()
    );
}

#[test]
fn iterator_close_getter_host_refusal_cannot_be_masked_by_the_mapper_guest_throw() {
    close_refusal(true);
}

#[test]
fn iterator_close_call_host_refusal_cannot_be_masked_by_the_mapper_guest_throw() {
    close_refusal(false);
}

#[test]
fn iterator_close_guest_throw_preserves_the_earlier_mapper_guest_throw() {
    let mut context = context();
    finite_eval(
        &mut context,
        "globalThis.closed = 0; globalThis.guestCaught = undefined;
         const iterator = {
             [Symbol.iterator]() { return this; },
             next() { return { value: 1, done: false }; },
             return() { closed += 1; throw 18; }
         };
         try { Array.from(iterator, () => { throw 17; }); }
         catch (error) { guestCaught = error; }",
    )
    .unwrap();
    assert_eq!(finite_eval(&mut context, "closed").unwrap(), 1.into());
    assert_eq!(finite_eval(&mut context, "guestCaught").unwrap(), 17.into());
}

#[derive(Default)]
struct RecordingExecutor {
    jobs: RefCell<Vec<Job>>,
}

impl JobExecutor for RecordingExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        self.jobs.borrow_mut().push(job);
        Ok(JobAcceptance::Accepted)
    }

    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the fixture records publication without dispatching callbacks")
    }
}

fn recording_context(executor: Rc<RecordingExecutor>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    builder.build().unwrap()
}

fn ordinary_handler(context: &mut Context) -> JsFunction {
    finite_eval(context, "globalThis.callbackBody = 0").unwrap();
    JsFunction::from_object(
        finite_eval(context, "(function () { callbackBody += 1; })")
            .unwrap()
            .as_object()
            .unwrap(),
    )
    .unwrap()
}

fn clear_records(executor: &RecordingExecutor) {
    let jobs = std::mem::take(&mut *executor.jobs.borrow_mut());
    drop(jobs);
}

#[test]
fn reaction_realm_access_refusal_cannot_publish_a_fallback_job() {
    let executor = Rc::new(RecordingExecutor::default());
    let mut context = recording_context(executor.clone());
    let callback = ordinary_handler(&mut context);
    let source = JsPromise::resolve(1, context.access()).unwrap();
    assert!(executor.jobs.borrow().is_empty());
    let alias = Deref::deref(&callback).clone();
    let held = alias.borrow_mut().unwrap();
    let error = source
        .then(Some(callback.clone()), None, context.access())
        .unwrap_err();
    borrowed(&error);
    assert_eq!(executor.jobs.borrow().len(), 0);
    drop(held);
    assert_eq!(finite_eval(&mut context, "callbackBody").unwrap(), 0.into());

    let accepted = source.then(Some(callback), None, context.access()).unwrap();
    assert_eq!(executor.jobs.borrow().len(), 1);
    assert_eq!(finite_eval(&mut context, "callbackBody").unwrap(), 0.into());
    clear_records(&executor);
    drop(accepted);
    assert!(executor.jobs.borrow().is_empty());
}

#[test]
fn thenable_realm_access_refusal_cannot_publish_a_fallback_job() {
    let executor = Rc::new(RecordingExecutor::default());
    let mut context = recording_context(executor.clone());
    let callback = ordinary_handler(&mut context);
    let thenable = JsObject::with_null_proto(context.runtime()).unwrap();
    thenable
        .set(js_string!("then"), callback.clone(), true, context.access())
        .unwrap();
    context
        .register_global_property(
            js_string!("heldThenable"),
            thenable,
            surgeist_js::property::Attribute::all(),
        )
        .unwrap();
    assert!(executor.jobs.borrow().is_empty());
    let alias = Deref::deref(&callback).clone();
    let held = alias.borrow_mut().unwrap();
    let error = finite_eval(&mut context, "Promise.resolve(heldThenable)").unwrap_err();
    borrowed(&error);
    assert_eq!(executor.jobs.borrow().len(), 0);
    drop(held);
    assert_eq!(finite_eval(&mut context, "callbackBody").unwrap(), 0.into());

    let accepted = finite_eval(&mut context, "Promise.resolve(heldThenable)").unwrap();
    assert_eq!(executor.jobs.borrow().len(), 1);
    assert_eq!(finite_eval(&mut context, "callbackBody").unwrap(), 0.into());
    clear_records(&executor);
    drop(accepted);
    assert!(executor.jobs.borrow().is_empty());
}
