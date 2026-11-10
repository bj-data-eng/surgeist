use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::{
    Context, JsObject, JsValue, Source,
    builtins::promise::PromiseState,
    error::{EngineError, RootingError},
    execution::ExecutionOptions,
    object::builtins::JsPromise,
};
use surgeist_js_gc::heap::{AccessError, RuntimeHeap};

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    builder.build().unwrap()
}

fn guest_object(context: &mut Context, source: &str) -> JsObject {
    context
        .with_execution(ExecutionOptions::new(100_000), |mut access| {
            access.eval(Source::from_bytes(source))
        })
        .unwrap()
        .result
        .unwrap()
        .as_object()
        .unwrap()
}

fn under_heap_read<T>(heap: &RuntimeHeap, operation: impl FnOnce() -> T) -> std::thread::Result<T> {
    heap.read(|_| {
        assert_eq!(
            heap.allocate(()).unwrap_err().reason,
            AccessError::HeapBorrowed
        );
        catch_unwind(AssertUnwindSafe(operation))
    })
    .unwrap()
}

#[test]
fn standard_prototype_projection_under_heap_read_does_not_unwind_and_retries_identity() {
    let mut context = context();
    let expected = guest_object(&mut context, "Promise.prototype");
    let heap = context.runtime().heap.clone();
    let attempt = under_heap_read(&heap, || {
        context.intrinsics().constructors().promise().prototype()
    });
    assert!(
        attempt.is_ok(),
        "intrinsic prototype acquisition must refuse without a Rust unwind"
    );
    assert_eq!(
        attempt.as_ref().unwrap().as_ref().err(),
        Some(&RootingError::Access(AccessError::HeapBorrowed))
    );
    drop(attempt);
    let retried = context
        .intrinsics()
        .constructors()
        .promise()
        .prototype()
        .unwrap();
    assert_eq!(retried.identity(), expected.identity());
    assert_eq!(retried, expected);
}

#[test]
fn standard_constructor_projection_under_heap_read_does_not_unwind_and_retries_identity() {
    let mut context = context();
    let expected = guest_object(&mut context, "Promise");
    let heap = context.runtime().heap.clone();
    let attempt = under_heap_read(&heap, || {
        context.intrinsics().constructors().promise().constructor()
    });
    assert!(
        attempt.is_ok(),
        "intrinsic constructor acquisition must refuse without a Rust unwind"
    );
    assert_eq!(
        attempt.as_ref().unwrap().as_ref().err(),
        Some(&RootingError::Access(AccessError::HeapBorrowed))
    );
    drop(attempt);
    let retried = context
        .intrinsics()
        .constructors()
        .promise()
        .constructor()
        .unwrap();
    assert_eq!(retried.identity(), expected.identity());
    assert_eq!(retried, expected);
}

#[test]
fn intrinsic_runtime_association_under_heap_read_does_not_root_or_unwind() {
    let context = context();
    let heap = context.runtime().heap.clone();
    let attempt = under_heap_read(&heap, || context.intrinsics().runtime());
    assert!(
        attempt.is_ok(),
        "runtime association must not acquire an intrinsic object root"
    );
    let associated = attempt.unwrap().unwrap();
    assert!(associated.same_heap(context.runtime()));
    let retried = context.intrinsics().runtime().unwrap();
    assert!(retried.same_heap(context.runtime()));
}

#[test]
fn pending_factory_under_heap_read_does_not_unwind_and_allows_fresh_resolution() {
    let mut context = context();
    let heap = context.runtime().heap.clone();
    let attempt = under_heap_read(&heap, || JsPromise::new_pending(context.access()));
    assert!(
        attempt.is_ok(),
        "actual heap access refusal must not become a Rust unwind"
    );
    assert_eq!(
        attempt
            .as_ref()
            .unwrap()
            .as_ref()
            .err()
            .unwrap()
            .as_engine(),
        Some(&EngineError::Rooting(RootingError::Access(
            AccessError::HeapBorrowed
        )))
    );
    drop(attempt);
    context.runtime().collect().unwrap();

    let (promise, functions) = JsPromise::new_pending(context.access()).unwrap();
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    functions
        .resolve
        .call(&JsValue::undefined(), &[71.into()], context.access())
        .unwrap();
    assert_eq!(promise.state().unwrap(), PromiseState::Fulfilled(71.into()));
}

#[test]
fn unborrowed_intrinsic_identity_and_pending_factory_preserve_actual_guest_objects() {
    let mut context = context();
    let expected_constructor = guest_object(&mut context, "Promise");
    let expected_prototype = guest_object(&mut context, "Promise.prototype");
    assert_eq!(
        context
            .intrinsics()
            .constructors()
            .promise()
            .constructor()
            .unwrap(),
        expected_constructor
    );
    assert_eq!(
        context
            .intrinsics()
            .constructors()
            .promise()
            .prototype()
            .unwrap(),
        expected_prototype
    );
    assert!(
        context
            .intrinsics()
            .runtime()
            .unwrap()
            .same_heap(context.runtime())
    );

    let value = JsValue::from(JsObject::with_null_proto(context.runtime()).unwrap());
    let (promise, functions) = JsPromise::new_pending(context.access()).unwrap();
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    functions
        .resolve
        .call(
            &JsValue::undefined(),
            std::slice::from_ref(&value),
            context.access(),
        )
        .unwrap();
    assert_eq!(promise.state().unwrap(), PromiseState::Fulfilled(value));
}

#[test]
fn runtime_association_keeps_the_heap_without_rooting_intrinsic_objects_after_context_drop() {
    let context = context();
    let weak_constructor = context
        .intrinsics()
        .constructors()
        .promise()
        .constructor()
        .unwrap()
        .edge();
    let weak_prototype = context
        .intrinsics()
        .constructors()
        .promise()
        .prototype()
        .unwrap()
        .edge();
    let runtime = context.intrinsics().runtime().unwrap();
    assert!(runtime.same_heap(context.runtime()));
    drop(context);

    runtime.collect().unwrap();
    assert!(!weak_constructor.is_live().unwrap());
    assert!(!weak_prototype.is_live().unwrap());
}

use std::{cell::Cell, rc::Rc};

use super::StandardConstructor;
use crate::{NativeFunction, object::FunctionObjectBuilder};
use surgeist_js_gc::heap::{AllocationCollection, ObjectId};

#[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
struct ProjectionLeaf(#[heap_trace(skip)] Rc<Cell<usize>>);

impl Drop for ProjectionLeaf {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn stored_pair(
    context: &Context,
    capture_drops: &Rc<Cell<usize>>,
    prototype_drops: &Rc<Cell<usize>>,
) -> (
    StandardConstructor,
    crate::object::JsFunction,
    JsObject,
    ObjectId,
) {
    context
        .runtime()
        .heap
        .set_allocation_collection(AllocationCollection::Manual)
        .unwrap();
    let captured = JsObject::new_unique(
        context.runtime(),
        None,
        ProjectionLeaf(capture_drops.clone()),
    )
    .unwrap()
    .upcast();
    let capture_id = captured.identity();
    let constructor = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_with_captures(
            |_, _, captured, _| Ok(captured.clone().into()),
            captured,
        ),
    )
    .constructor(true)
    .build()
    .unwrap();
    let prototype = JsObject::new_unique(
        context.runtime(),
        None,
        ProjectionLeaf(prototype_drops.clone()),
    )
    .unwrap()
    .upcast();
    let stored = StandardConstructor::new(constructor.clone(), prototype.clone());
    (stored, constructor, prototype, capture_id)
}

#[test]
fn actually_expired_standard_constructor_roots_return_expired_without_replacement() {
    let context = context();
    let capture_drops = Rc::new(Cell::new(0));
    let prototype_drops = Rc::new(Cell::new(0));
    let (stored, constructor, prototype, _) =
        stored_pair(&context, &capture_drops, &prototype_drops);
    let weak_constructor = constructor.edge();
    let weak_prototype = prototype.edge();
    drop(constructor);
    drop(prototype);
    context.runtime().collect().unwrap();
    assert_eq!(capture_drops.get(), 1);
    assert_eq!(prototype_drops.get(), 1);
    assert!(!weak_constructor.is_live().unwrap());
    assert!(!weak_prototype.is_live().unwrap());
    assert_eq!(stored.constructor(), Err(RootingError::Expired));
    assert_eq!(stored.prototype(), Err(RootingError::Expired));
    assert_eq!(capture_drops.get(), 1);
    assert_eq!(prototype_drops.get(), 1);
}

#[test]
fn projected_standard_objects_own_independent_roots_after_original_aliases_drop() {
    let mut context = context();
    let capture_drops = Rc::new(Cell::new(0));
    let prototype_drops = Rc::new(Cell::new(0));
    let (stored, constructor, prototype, capture_id) =
        stored_pair(&context, &capture_drops, &prototype_drops);
    let constructor_id = constructor.identity();
    let prototype_id = prototype.identity();
    let weak_constructor = constructor.edge();
    let weak_prototype = prototype.edge();
    let projected_constructor = stored.constructor().unwrap();
    let projected_prototype = stored.prototype().unwrap();
    drop(constructor);
    drop(prototype);
    context.runtime().collect().unwrap();
    assert_eq!(capture_drops.get(), 0);
    assert_eq!(prototype_drops.get(), 0);
    assert_eq!(projected_constructor.identity(), constructor_id);
    assert_eq!(projected_prototype.identity(), prototype_id);
    {
        let returned = projected_constructor
            .call(&JsValue::undefined(), &[], context.access())
            .unwrap();
        assert_eq!(returned.as_object().unwrap().identity(), capture_id);
    }
    drop(projected_constructor);
    context.runtime().collect().unwrap();
    assert!(!weak_constructor.is_live().unwrap());
    assert_eq!(capture_drops.get(), 1);
    assert_eq!(prototype_drops.get(), 0);
    drop(projected_prototype);
    context.runtime().collect().unwrap();
    assert!(!weak_prototype.is_live().unwrap());
    assert_eq!(prototype_drops.get(), 1);
}

#[test]
fn global_object_and_this_projections_preserve_heap_refusal_and_fresh_identity() {
    let mut context = context();
    let expected = guest_object(&mut context, "globalThis");
    let realm = context.realm().clone();
    let heap = context.runtime().heap.clone();
    heap.read(|_| {
        assert_eq!(
            heap.allocate(()).unwrap_err().reason,
            AccessError::HeapBorrowed
        );
        assert_eq!(
            context.global_object().unwrap_err(),
            RootingError::Access(AccessError::HeapBorrowed)
        );
        assert_eq!(
            context.access().global_object().unwrap_err(),
            RootingError::Access(AccessError::HeapBorrowed)
        );
        assert_eq!(
            realm.global_object().unwrap_err(),
            RootingError::Access(AccessError::HeapBorrowed)
        );
        assert_eq!(
            realm.global_this().unwrap_err(),
            RootingError::Access(AccessError::HeapBorrowed)
        );
    })
    .unwrap();
    assert_eq!(context.global_object().unwrap(), expected);
    assert_eq!(context.access().global_object().unwrap(), expected);
    assert_eq!(realm.global_object().unwrap(), expected);
    assert_eq!(realm.global_this().unwrap(), expected);
}
