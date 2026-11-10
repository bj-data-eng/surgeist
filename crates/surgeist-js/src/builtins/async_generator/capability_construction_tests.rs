use super::{AsyncGenerator, AsyncGeneratorState};
use crate::{
    Context, JsObject, JsResult, JsValue, Source,
    builtins::promise::PromiseState,
    error::{EngineError, ObjectAccessError},
    execution::ExecutionOptions,
    js_string,
    object::builtins::JsPromise,
};
use surgeist_js_gc::heap::AllocationCollection;

#[derive(Clone, Copy, Debug)]
enum Entry {
    Next,
    Return,
    Throw,
}

impl Entry {
    fn call(
        self,
        context: &mut Context,
        generator: &JsObject<AsyncGenerator>,
    ) -> JsResult<JsValue> {
        let this = generator.clone().upcast().into();
        let argument = JsValue::new(11);
        context
            .with_execution(ExecutionOptions::new(100_000), |access| {
                let context = access.into_owner();
                let args = std::slice::from_ref(&argument);
                match self {
                    Self::Next => AsyncGenerator::next(&this, args, context),
                    Self::Return => AsyncGenerator::r#return(&this, args, context),
                    Self::Throw => AsyncGenerator::throw(&this, args, context),
                }
            })
            .unwrap()
            .result
    }
}

fn fixture() -> (Context, JsObject<AsyncGenerator>) {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .runtime()
        .heap
        .set_allocation_collection(AllocationCollection::Manual)
        .unwrap();
    let value = context
        .with_execution(ExecutionOptions::new(100_000), |mut access| {
            access.eval(Source::from_bytes(
                "globalThis.capabilityBodyEntries = 0; \
                 (async function* () { capabilityBodyEntries++; yield 7; })()",
            ))
        })
        .unwrap()
        .result
        .unwrap();
    let generator = value
        .as_object()
        .unwrap()
        .downcast::<AsyncGenerator>()
        .unwrap();
    (context, generator)
}

fn body_entries(context: &mut Context) -> JsValue {
    context
        .global_object()
        .unwrap()
        .get(js_string!("capabilityBodyEntries"), context.access())
        .unwrap()
}

fn assert_untouched(context: &mut Context, generator: &JsObject<AsyncGenerator>) {
    {
        let generator = generator.borrow().unwrap();
        assert_eq!(generator.data().state, AsyncGeneratorState::SuspendedStart);
        assert!(generator.data().context.is_some());
        assert!(generator.data().queue.is_empty());
    }
    assert_eq!(body_entries(context), JsValue::new(0));
}

fn assert_guest_result(
    entry: Entry,
    context: &mut Context,
    generator: &JsObject<AsyncGenerator>,
    value: JsValue,
) {
    let promise = JsPromise::from_object(value.as_object().unwrap()).unwrap();
    context.run_jobs().unwrap();
    let state = promise.state().unwrap();
    match entry {
        Entry::Next | Entry::Return => {
            let PromiseState::Fulfilled(value) = state else {
                panic!("{entry:?} must fulfill with its iterator result")
            };
            let result = value.as_object().unwrap();
            let (value, done) = if matches!(entry, Entry::Next) {
                (7, false)
            } else {
                (11, true)
            };
            assert_eq!(
                result.get(js_string!("value"), context.access()).unwrap(),
                JsValue::new(value)
            );
            assert_eq!(
                result.get(js_string!("done"), context.access()).unwrap(),
                JsValue::new(done)
            );
        }
        Entry::Throw => assert_eq!(state, PromiseState::Rejected(JsValue::new(11))),
    }
    {
        let generator = generator.borrow().unwrap();
        assert!(generator.data().queue.is_empty());
        assert_eq!(
            generator.data().state,
            match entry {
                Entry::Next => AsyncGeneratorState::SuspendedYield,
                Entry::Return | Entry::Throw => AsyncGeneratorState::Completed,
            }
        );
    }
    assert_eq!(
        body_entries(context),
        JsValue::new(if matches!(entry, Entry::Next) { 1 } else { 0 })
    );
}

fn borrowed_constructor_refuses_before_request_and_retries(entry: Entry) {
    let (mut context, generator) = fixture();
    assert_untouched(&mut context, &generator);
    let constructor = context
        .intrinsics()
        .constructors()
        .promise()
        .constructor()
        .unwrap();
    let heap = context.runtime().heap.clone();
    let constructor_loan = constructor.borrow_mut().unwrap();

    // A payload loan permits rooting and allocation. Manual collection keeps
    // allocation from tracing this unrelated borrowed existing payload.
    assert_eq!(
        context
            .intrinsics()
            .constructors()
            .promise()
            .constructor()
            .unwrap()
            .identity(),
        constructor.identity()
    );
    let allocation = heap.allocate(()).unwrap();
    drop(allocation);

    let error = entry.call(&mut context, &generator).unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed)),
        "{entry:?} must preserve the native constructor's exact access refusal"
    );
    assert!(!error.is_catchable());
    assert_untouched(&mut context, &generator);

    drop(constructor_loan);
    let result = entry.call(&mut context, &generator).unwrap();
    assert_guest_result(entry, &mut context, &generator, result);
}

#[test]
fn next_capability_constructor_payload_refusal_keeps_request_for_retry() {
    borrowed_constructor_refuses_before_request_and_retries(Entry::Next);
}

#[test]
fn return_capability_constructor_payload_refusal_keeps_request_for_retry() {
    borrowed_constructor_refuses_before_request_and_retries(Entry::Return);
}

#[test]
fn throw_capability_constructor_payload_refusal_keeps_request_for_retry() {
    borrowed_constructor_refuses_before_request_and_retries(Entry::Throw);
}

#[test]
fn unborrowed_capability_constructor_preserves_async_generator_guest_results() {
    for entry in [Entry::Next, Entry::Return, Entry::Throw] {
        let (mut context, generator) = fixture();
        let result = entry.call(&mut context, &generator).unwrap();
        assert_guest_result(entry, &mut context, &generator, result);
    }
}
