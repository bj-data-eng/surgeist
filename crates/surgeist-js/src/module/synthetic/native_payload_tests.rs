// Intended as a child of the synthetic module owner. Construction is observed
// separately from the ordinary later consumer of the stored export payload.
use super::*;
use crate::{IntoJsModule, builtins::promise::PromiseState, execution::ExecutionOptions};

#[derive(Clone)]
struct Capture {
    id: u32,
    value: JsObject,
    restored: Rc<RefCell<Vec<u32>>>,
}
#[derive(Trace)]
struct StoredCapture {
    id: u32,
    value: <JsObject as NativeCapture>::Stored,
    #[heap_trace(skip)]
    restored: Rc<RefCell<Vec<u32>>>,
}
impl NativeCapture for Capture {
    type Stored = StoredCapture;
    fn store(&self) -> StoredCapture {
        StoredCapture {
            id: self.id,
            value: self.value.store(),
            restored: self.restored.clone(),
        }
    }
    fn restore(stored: &StoredCapture) -> Result<Self, crate::error::RootingError> {
        // The scalar log is outside every owning engine record. Conversion
        // neither enters the engine nor mutates the retained export payload.
        stored.restored.borrow_mut().push(stored.id);
        Ok(Self {
            id: stored.id,
            value: JsObject::restore(&stored.value)?,
            restored: stored.restored.clone(),
        })
    }
}
fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}
fn function(id: u32, value: &JsObject, restored: &Rc<RefCell<Vec<u32>>>) -> NativeFunction {
    NativeFunction::from_fn_with_captures(
        |_, _, capture, _| Ok(capture.value.clone().into()),
        Capture {
            id,
            value: value.clone(),
            restored: restored.clone(),
        },
    )
}

#[test]
fn stored_native_exports_preserve_duplicate_request_order_and_final_function_identity() {
    let mut context = context();
    let first = JsObject::with_null_proto(context.runtime()).unwrap();
    let second = JsObject::with_null_proto(context.runtime()).unwrap();
    let third = JsObject::with_null_proto(context.runtime()).unwrap();
    let restored = Rc::new(RefCell::new(Vec::new()));
    let exports = vec![
        (js_string!("answer"), function(1, &first, &restored)),
        (js_string!("answer"), function(2, &second, &restored)),
        (js_string!("other"), function(3, &third, &restored)),
    ];
    let constructed = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            exports.into_js_module(access)
        })
        .unwrap();
    let module = constructed.result.unwrap();
    assert!(!constructed.report.effects_started());
    assert_eq!(constructed.report.instructions().unwrap(), 0);
    assert!(
        restored.borrow().is_empty(),
        "construction stores the payload without restoring its functions"
    );

    // Ordinary evaluation consumes the stored payload after construction has
    // ended. No work from this operation is part of the constructor tariff.
    let initialized = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            let owner = access.into_owner();
            let promise = module.load_link_evaluate(owner.access())?;
            owner.run_jobs()?;
            Ok(promise)
        })
        .unwrap();
    assert_eq!(
        initialized.result.unwrap().state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(
        *restored.borrow(),
        [1, 2, 3],
        "all authored entries restore in order, including the overwritten duplicate"
    );
    let answer = module
        .get_value(js_string!("answer"), context.access())
        .unwrap()
        .as_object()
        .unwrap();
    let again = module
        .get_value(js_string!("answer"), context.access())
        .unwrap()
        .as_object()
        .unwrap();
    let other = module
        .get_value(js_string!("other"), context.access())
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(answer, again);
    assert_ne!(answer, other);
    assert_eq!(
        answer
            .call(&JsValue::undefined(), &[], context.access())
            .unwrap()
            .as_object()
            .unwrap(),
        second
    );
    assert_eq!(
        other
            .call(&JsValue::undefined(), &[], context.access())
            .unwrap()
            .as_object()
            .unwrap(),
        third
    );
    assert_ne!(first, second);
}

#[derive(Trace)]
struct FailureCounter(#[heap_trace(skip)] Rc<std::cell::Cell<usize>>);

#[derive(Trace)]
struct PromiseAllocationCanary(#[heap_trace(skip)] Rc<std::cell::Cell<usize>>);

impl Drop for PromiseAllocationCanary {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn initializer_host_refusal_precedes_later_promise_allocation() {
    use crate::error::{EngineError, RootingError};
    use surgeist_js_gc::heap::{AccessError, AllocationCollection};

    let mut context = context();
    let runtime = context.runtime().clone();
    runtime
        .heap
        .set_allocation_collection(AllocationCollection::Manual)
        .unwrap();
    let drops = Rc::new(std::cell::Cell::new(0));
    let counter = runtime
        .heap
        .allocate(FailureCounter(drops.clone()))
        .unwrap();
    let steps = SyntheticModuleInitializer::from_fn_with_captures(
        |module, counter, mut access| {
            module.set_export(&js_string!("accepted"), JsValue::new(42))?;
            let runtime = access.runtime().clone();
            let canary = runtime
                .heap
                .allocate(PromiseAllocationCanary(counter.0.clone()))
                .unwrap();
            drop(canary);
            // Any later GC allocation will collect the real unreachable canary.
            runtime
                .heap
                .set_allocation_collection(AllocationCollection::BeforeAllocation)
                .unwrap();
            let refused = runtime
                .heap
                .read(|_| JsPromise::new_pending(access.reborrow()))
                .unwrap();
            let error = match refused {
                Err(error) => error,
                Ok(_) => panic!("a live heap read must refuse promise construction"),
            };
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::Rooting(RootingError::Access(
                    AccessError::HeapBorrowed
                )))
            );
            Err(error)
        },
        counter,
    );
    let module = Module::synthetic(
        &[js_string!("accepted")],
        steps,
        None,
        None,
        context.access(),
    )
    .unwrap();
    module.link(context.access()).unwrap();
    let invocation = context.vm.frame().invocation();
    let error = module.evaluate(context.access()).unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::Rooting(RootingError::Access(
            AccessError::HeapBorrowed
        )))
    );
    assert_eq!(
        context.vm.frame().invocation(),
        invocation,
        "the initializer frame is restored before returning its error"
    );
    assert_eq!(
        drops.get(),
        0,
        "the admitted initializer failure prevents a later promise allocation"
    );
    let crate::module::ModuleKind::Synthetic(source) = module.kind() else {
        unreachable!()
    };
    assert!(matches!(
        &*source.state.borrow(),
        ModuleStatus::Linked { .. }
    ));
    runtime
        .heap
        .set_allocation_collection(AllocationCollection::Manual)
        .unwrap();
    assert_eq!(
        module
            .get_value(js_string!("accepted"), context.access())
            .unwrap(),
        JsValue::new(42)
    );
    runtime.collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "collection drops the canary without a later allocation"
    );
}
