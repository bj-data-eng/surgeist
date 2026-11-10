use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    task::Poll,
};

use surgeist_js::{
    Context, ContextAccess, IntoJsModule, JsData, JsError, JsObject, JsResult, JsValue, Module,
    NativeFunction, Source,
    builtins::promise::PromiseState,
    error::{DiagnosticEngine, DiagnosticKind, EngineError, RootingError},
    execution::ExecutionOptions,
    job::{ContextTask, SimpleJobExecutor},
    js_string,
    module::MapModuleLoader,
    native_function::NativeCapture,
    object::builtins::{JsFunction, JsPromise},
};
use surgeist_js_gc::heap::{AccessError, EdgeVisitor, Rooted, RuntimeHeap, Trace};

const FINITE: u64 = 100_000;

fn context() -> Context {
    let builder = Context::builder().job_executor(Rc::new(SimpleJobExecutor::new()));
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(FINITE.try_into().unwrap());
    builder.build().unwrap()
}

fn scoped<T>(
    context: &mut Context,
    call: impl for<'call> FnOnce(ContextAccess<'call>) -> JsResult<T>,
) -> JsResult<T> {
    context
        .with_execution(ExecutionOptions::new(FINITE), call)
        .unwrap()
        .result
}

fn access_refusal(error: &JsError, access: AccessError) {
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::Rooting(RootingError::Access(access)))
    );
}

#[derive(Debug, JsData)]
struct Leaf {
    value: u32,
    drops: Rc<Cell<usize>>,
}

impl Trace for Leaf {
    fn trace(&self, _: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        Ok(())
    }
}

impl Drop for Leaf {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn held_heap_read_refuses_capture_root_acquisition_and_preserves_retry_identity() {
    let heap = RuntimeHeap::default();
    let drops = Rc::new(Cell::new(0));
    let original = heap
        .allocate(Leaf {
            value: 23,
            drops: drops.clone(),
        })
        .unwrap();
    let stored = original.store();
    let identity = stored.identity();
    heap.read(|_| {
        assert_eq!(
            Rooted::<Leaf>::restore(&stored).unwrap_err(),
            RootingError::Access(AccessError::HeapBorrowed)
        );
    })
    .unwrap();
    assert_eq!(stored.identity(), identity);
    let restored = Rooted::<Leaf>::restore(&stored).unwrap();
    assert_eq!(restored.handle().erase(), identity);
    assert_eq!(restored.value, 23);
    drop(original);
    assert_eq!(heap.collect().unwrap(), 0);
    assert_eq!(drops.get(), 0);
    drop(restored);
    assert_eq!(heap.collect().unwrap(), 1);
    assert_eq!(drops.get(), 1);
}

#[test]
fn stored_ordinary_object_refuses_native_brand_and_callable_restoration() {
    let context = context();
    let object = JsObject::with_null_proto(context.runtime()).unwrap();
    let identity = object.identity();
    let stored = object.store();
    assert_eq!(
        JsObject::<Leaf>::restore(&stored).unwrap_err(),
        RootingError::WrongNativeBrand
    );
    assert_eq!(
        JsFunction::restore(&stored).unwrap_err(),
        RootingError::NotCallable
    );
    let restored: JsObject = JsObject::restore(&stored).unwrap();
    assert_eq!(restored.identity(), identity);
    assert_eq!(restored, object);
}

#[test]
fn tuple_and_vector_expired_tail_release_partial_roots_without_changing_storage() {
    for vector in [false, true] {
        let heap = RuntimeHeap::default();
        let tail_heap = RuntimeHeap::default();
        let first_drops = Rc::new(Cell::new(0));
        let last_drops = Rc::new(Cell::new(0));
        let first = heap
            .allocate(Leaf {
                value: 31,
                drops: first_drops.clone(),
            })
            .unwrap();
        let last = tail_heap
            .allocate(Leaf {
                value: 37,
                drops: last_drops.clone(),
            })
            .unwrap();
        let stored = (first.store(), last.store());
        let identities = (stored.0.identity(), stored.1.identity());
        let stored_vector = vec![stored.0.clone(), stored.1.clone()];
        drop(last);
        assert_eq!(tail_heap.collect().unwrap(), 1);
        assert_eq!(last_drops.get(), 1);
        assert!(stored.1.upgrade().unwrap().is_none());
        drop(first);

        let restored = if vector {
            Vec::<Rooted<Leaf>>::restore(&stored_vector).map(|_| ())
        } else {
            <(Rooted<Leaf>, Rooted<Leaf>)>::restore(&stored).map(|_| ())
        };
        assert_eq!(restored, Err(RootingError::Expired));
        assert_eq!(first_drops.get(), 0);
        assert_eq!((stored.0.identity(), stored.1.identity()), identities);
        assert_eq!(stored_vector[0].identity(), identities.0);
        assert_eq!(stored_vector[1].identity(), identities.1);
        assert_eq!(heap.collect().unwrap(), 1, "failed prefix must own no root");
        assert_eq!(first_drops.get(), 1);
        assert!(stored.0.upgrade().unwrap().is_none());
        assert_eq!(last_drops.get(), 1);
    }
}

#[test]
fn restored_native_function_retains_actual_leaf_until_its_final_owner_drops() {
    let mut context = context();
    let drops = Rc::new(Cell::new(0));
    let object = JsObject::new_unique(
        context.runtime(),
        None,
        Leaf {
            value: 43,
            drops: drops.clone(),
        },
    )
    .unwrap()
    .upcast();
    let identity = object.identity();
    let original =
        NativeFunction::from_fn_with_captures(|_, _, object, _| Ok(object.clone().into()), object);
    let stored = original.store();
    let restored = NativeFunction::restore(&stored).unwrap();
    drop(original);
    context.runtime().collect().unwrap();
    assert_eq!(drops.get(), 0);
    let result = restored
        .call(&JsValue::undefined(), &[], context.access())
        .unwrap();
    assert_eq!(result.as_object().unwrap().identity(), identity);
    drop(result);
    drop(restored);
    context.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "stored function alone is not an external root"
    );
    assert_eq!(
        NativeFunction::restore(&stored).unwrap_err(),
        RootingError::Expired
    );
    drop(stored);
    context.runtime().collect().unwrap();
    assert_eq!(drops.get(), 1);
}

#[derive(Default)]
struct Entries {
    sync: Cell<usize>,
    factories: Cell<usize>,
    polls: Cell<usize>,
}

struct GateCapture {
    state: Rc<RefCell<u32>>,
    entries: Rc<Entries>,
    snapshot: u32,
}

struct StoredGate {
    state: Rc<RefCell<u32>>,
    entries: Rc<Entries>,
}

impl Trace for StoredGate {
    fn trace(&self, _: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        // The state and counters contain Rust scalars and no JavaScript edges.
        Ok(())
    }
}

impl NativeCapture for GateCapture {
    type Stored = StoredGate;

    fn store(&self) -> Self::Stored {
        StoredGate {
            state: self.state.clone(),
            entries: self.entries.clone(),
        }
    }

    fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
        let snapshot = *stored
            .state
            .try_borrow()
            .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
        Ok(Self {
            state: stored.state.clone(),
            entries: stored.entries.clone(),
            snapshot,
        })
    }
}

fn capture(state: &Rc<RefCell<u32>>, entries: &Rc<Entries>) -> GateCapture {
    GateCapture {
        state: state.clone(),
        entries: entries.clone(),
        snapshot: *state.borrow(),
    }
}

fn sync_function(state: &Rc<RefCell<u32>>, entries: &Rc<Entries>) -> NativeFunction {
    NativeFunction::from_fn_with_captures(
        |_, _, capture, _| {
            capture.entries.sync.set(capture.entries.sync.get() + 1);
            Ok(capture.snapshot.into())
        },
        capture(state, entries),
    )
}

#[test]
fn real_capture_data_borrow_prevents_sync_entry_then_same_storage_retries() {
    let mut context = context();
    let state = Rc::new(RefCell::new(47));
    let entries = Rc::new(Entries::default());
    let original = sync_function(&state, &entries);
    let stored = original.store();
    let loan = state.borrow_mut();
    assert_eq!(
        NativeFunction::restore(&stored).unwrap_err(),
        RootingError::Access(AccessError::PayloadBorrowed)
    );
    let error = original
        .call(&JsValue::undefined(), &[], context.access())
        .unwrap_err();
    access_refusal(&error, AccessError::PayloadBorrowed);
    assert_eq!(entries.sync.get(), 0);
    drop(loan);
    let restored = NativeFunction::restore(&stored).unwrap();
    drop(original);
    assert_eq!(
        restored
            .call(&JsValue::undefined(), &[], context.access())
            .unwrap(),
        JsValue::new(47)
    );
    assert_eq!(entries.sync.get(), 1);
}

#[test]
fn real_capture_data_borrow_prevents_async_factory_and_task_then_retries() {
    let mut context = context();
    let state = Rc::new(RefCell::new(53));
    let entries = Rc::new(Entries::default());
    let original = NativeFunction::from_async_fn_with_captures(
        |_, _, capture, _| {
            capture
                .entries
                .factories
                .set(capture.entries.factories.get() + 1);
            Ok(ContextTask::from_poll(move |_, _| {
                capture.entries.polls.set(capture.entries.polls.get() + 1);
                Poll::Ready(Ok(capture.snapshot.into()))
            }))
        },
        capture(&state, &entries),
    );
    let stored = original.store();
    let loan = state.borrow_mut();
    let error = original
        .call(&JsValue::undefined(), &[], context.access())
        .unwrap_err();
    access_refusal(&error, AccessError::PayloadBorrowed);
    assert_eq!(entries.factories.get(), 0);
    assert_eq!(entries.polls.get(), 0);
    scoped(&mut context, |mut access| access.run_jobs()).unwrap();
    assert_eq!(entries.factories.get(), 0);
    assert_eq!(entries.polls.get(), 0);
    drop(loan);

    let restored = NativeFunction::restore(&stored).unwrap();
    drop(original);
    let value = restored
        .call(&JsValue::undefined(), &[], context.access())
        .unwrap();
    let promise = JsPromise::from_object(value.as_object().unwrap()).unwrap();
    drop(value);
    drop(restored);
    drop(stored);
    scoped(&mut context, |mut access| access.run_jobs()).unwrap();
    assert_eq!(entries.factories.get(), 1);
    assert_eq!(entries.polls.get(), 1);
    assert_eq!(promise.state().unwrap(), PromiseState::Fulfilled(53.into()));
}

#[test]
fn later_native_export_restore_refusal_publishes_no_prefix_and_preserves_retry_work() {
    let loader = Rc::new(MapModuleLoader::default());
    let builder = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .module_loader(loader.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(FINITE.try_into().unwrap());
    let mut context = builder.build().unwrap();
    context
        .eval(Source::from_bytes("globalThis.importerEffects = 0"))
        .unwrap();
    let first_state = Rc::new(RefCell::new(59));
    let last_state = Rc::new(RefCell::new(61));
    let entries = Rc::new(Entries::default());
    let exports = vec![
        (js_string!("earlier"), sync_function(&first_state, &entries)),
        (js_string!("later"), sync_function(&last_state, &entries)),
    ];
    let native = exports.into_js_module(context.access()).unwrap();
    assert!(loader.insert("native", native.clone()).is_none());
    let importer = Module::parse(
        Source::from_bytes(
            "import { earlier, later } from 'native'; globalThis.importerEffects += 1;",
        ),
        None,
        context.access(),
    )
    .unwrap();
    let loaded = importer.load(context.access()).unwrap();
    scoped(&mut context, |mut access| access.run_jobs()).unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    importer.link(context.access()).unwrap();
    for name in [js_string!("earlier"), js_string!("later")] {
        assert_eq!(
            native.get_value(name, context.access()).unwrap(),
            JsValue::undefined()
        );
    }
    let loan = last_state.borrow_mut();
    let first = scoped(&mut context, |access| importer.evaluate(access)).unwrap_err();
    access_refusal(&first, AccessError::PayloadBorrowed);
    for name in [js_string!("earlier"), js_string!("later")] {
        assert_eq!(
            native.get_value(name, context.access()).unwrap(),
            JsValue::undefined()
        );
    }
    assert_eq!(entries.sync.get(), 0);
    assert_eq!(
        context.eval(Source::from_bytes("importerEffects")).unwrap(),
        0.into()
    );
    drop(loan);

    let later = scoped(&mut context, |access| importer.evaluate(access)).unwrap_err();
    let Some(EngineError::ModuleInterrupted(interruption)) = later.as_engine() else {
        panic!("the importer must retain its actual restoration failure")
    };
    assert_eq!(
        interruption.cause().kind(),
        DiagnosticKind::Engine(DiagnosticEngine::Rooting(RootingError::Access(
            AccessError::PayloadBorrowed
        )))
    );
    let retried = scoped(&mut context, |access| native.evaluate(access)).unwrap();
    assert_eq!(
        retried.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    for (name, value) in [(js_string!("earlier"), 59), (js_string!("later"), 61)] {
        let exported = native.get_value(name.clone(), context.access()).unwrap();
        assert_eq!(native.get_value(name, context.access()).unwrap(), exported);
        assert_eq!(
            exported
                .as_object()
                .unwrap()
                .call(&JsValue::undefined(), &[], context.access())
                .unwrap(),
            JsValue::new(value)
        );
    }
    assert_eq!(entries.sync.get(), 2);
    let repeated = scoped(&mut context, |access| importer.evaluate(access)).unwrap_err();
    assert_eq!(repeated.as_engine(), later.as_engine());
    assert_eq!(
        context.eval(Source::from_bytes("importerEffects")).unwrap(),
        0.into()
    );
}
