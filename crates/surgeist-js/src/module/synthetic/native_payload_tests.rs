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
    fn restore(stored: &StoredCapture) -> Self {
        // The scalar log is outside every owning engine record. Conversion
        // neither enters the engine nor mutates the retained export payload.
        stored.restored.borrow_mut().push(stored.id);
        Self {
            id: stored.id,
            value: JsObject::restore(&stored.value),
            restored: stored.restored.clone(),
        }
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
    let first = JsObject::with_null_proto(context.runtime());
    let second = JsObject::with_null_proto(context.runtime());
    let third = JsObject::with_null_proto(context.runtime());
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
