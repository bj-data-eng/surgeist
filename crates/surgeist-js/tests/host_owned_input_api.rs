use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
use surgeist_js::{
    Context, JsResult, JsValue, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{ExecutionOptions, ExecutionReport, ExecutionTermination},
};

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    builder.build().unwrap()
}
#[derive(Debug)]
struct Payload {
    message: String,
    value: Cell<u32>,
    drops: Rc<Cell<usize>>,
}
impl Drop for Payload {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
fn payload(drops: &Rc<Cell<usize>>) -> Rc<Payload> {
    Rc::new(Payload {
        message: String::from("original owned input"),
        value: Cell::new(7),
        drops: drops.clone(),
    })
}
fn no_actions(report: &ExecutionReport) {
    assert_eq!(report.instructions().unwrap(), 0);
    assert_eq!(report.job_preparations().unwrap(), 0);
    assert_eq!(report.job_invocations().unwrap(), 0);
    assert_eq!(report.job_polls().unwrap(), 0);
}
fn instruction_stop() -> EngineError {
    EngineError::RuntimeLimit(RuntimeLimitError::Instruction { limit: 0 })
}

#[test]
fn busy_acquisition_returns_the_same_owned_input_without_calling_or_retaining_it() {
    let mut owner = context();
    let owner_id = owner.id();
    let drops = Rc::new(Cell::new(0));
    let original = payload(&drops);
    let weak = Rc::downgrade(&original);
    let input = original.clone();
    let denied_calls = Cell::new(0);
    let denied = owner
        .with_execution(ExecutionOptions::new(0), |access| {
            let holder = access.into_host_access();
            holder
                .with_owned(input, |input, access| {
                    assert_eq!(access.id(), owner_id);
                    let (returned, error) = holder
                        .with_owned(input, |input, mut competing| -> JsResult<_> {
                            denied_calls.set(denied_calls.get() + 1);
                            competing.eval(Source::from_bytes("globalThis.deniedOwned = 1"))?;
                            Ok(input)
                        })
                        .unwrap_err();
                    assert_eq!(error.as_engine(), Some(&EngineError::ContextAccessBusy));
                    assert!(Rc::ptr_eq(&returned, &original));
                    assert_eq!(returned.message, "original owned input");
                    assert_eq!(returned.value.get(), 7);
                    assert_eq!(drops.get(), 0);
                    Ok((returned, error))
                })
                .unwrap()
        })
        .unwrap();
    let (returned, busy) = denied.result.unwrap();
    no_actions(&denied.report);
    assert!(matches!(
        denied.report.termination(),
        ExecutionTermination::Completed
    ));
    assert_eq!(denied_calls.get(), 0);
    assert!(Rc::ptr_eq(&returned, &original));
    assert_eq!(drops.get(), 0);
    assert_eq!(busy.clone().into_opaque(owner.access()), Err(busy));

    let accepted_calls = Cell::new(0);
    let accepted = owner
        .with_execution(ExecutionOptions::new(512), |access| {
            let holder = access.into_host_access();
            holder
                .with_owned(returned, |input, mut access| {
                    accepted_calls.set(accepted_calls.get() + 1);
                    assert_eq!(access.id(), owner_id);
                    assert!(Rc::ptr_eq(&input, &original));
                    input.value.set(42);
                    let answer = access.eval(Source::from_bytes("40 + 2"))?;
                    Ok((input, answer))
                })
                .unwrap()
        })
        .unwrap();
    let (returned, answer) = accepted.result.unwrap();
    assert_eq!(answer, JsValue::new(42));
    assert_eq!(accepted_calls.get(), 1);
    assert_eq!(denied_calls.get(), 0);
    assert!(Rc::ptr_eq(&returned, &original));
    assert_eq!(returned.value.get(), 42);
    drop(original);
    assert!(weak.upgrade().is_some());
    assert_eq!(drops.get(), 0);
    drop(returned);
    assert!(
        weak.upgrade().is_none(),
        "the holder must not retain returned input"
    );
    assert_eq!(drops.get(), 1);
}

#[test]
fn busy_owned_acquisition_preserves_input_and_the_operations_actual_first_stop() {
    let mut owner = context();
    let owner_id = owner.id();
    let drops = Rc::new(Cell::new(0));
    let input = payload(&drops);
    let weak = Rc::downgrade(&input);
    let actual_direct = RefCell::new(None);
    let denied_calls = Cell::new(0);
    let stopped = owner
        .with_execution(ExecutionOptions::new(0), |access| {
            let holder = access.into_host_access();
            let direct = holder
                .with_owned(input, |input, mut access| {
                    assert_eq!(access.id(), owner_id);
                    let first = access.eval(Source::from_bytes("0")).unwrap_err();
                    assert_eq!(first.as_engine(), Some(&instruction_stop()));
                    let (returned, error) = holder
                        .with_owned(input, |input, _| {
                            denied_calls.set(denied_calls.get() + 1);
                            input
                        })
                        .unwrap_err();
                    assert_eq!(error.as_engine(), Some(&EngineError::ContextAccessBusy));
                    assert!(Rc::ptr_eq(&returned, &weak.upgrade().unwrap()));
                    assert_eq!(returned.value.get(), 7);
                    (returned, error)
                })
                .unwrap();
            // Observe the direct acquire-or-return result separately from the
            // enclosing operation's mandatory first-stop result.
            *actual_direct.borrow_mut() = Some(direct);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        stopped.result.unwrap_err().as_engine(),
        Some(&instruction_stop())
    );
    no_actions(&stopped.report);
    assert_eq!(denied_calls.get(), 0);
    let (returned, direct_error) = actual_direct.into_inner().unwrap();
    assert_eq!(
        direct_error.as_engine(),
        Some(&EngineError::ContextAccessBusy)
    );
    assert_eq!(
        direct_error.clone().into_opaque(owner.access()),
        Err(direct_error)
    );
    assert!(Rc::ptr_eq(&returned, &weak.upgrade().unwrap()));
    assert_eq!(drops.get(), 0);
    let fresh = owner
        .with_execution(ExecutionOptions::new(512), |access| {
            access
                .into_host_access()
                .with_owned(returned, |input, mut access| {
                    assert_eq!(access.id(), owner_id);
                    assert_eq!(input.message, "original owned input");
                    access.eval(Source::from_bytes("40 + 2"))
                })
                .unwrap()
        })
        .unwrap();
    assert_eq!(fresh.result.unwrap(), JsValue::new(42));
    assert_eq!(denied_calls.get(), 0);
    assert!(weak.upgrade().is_none());
    assert_eq!(drops.get(), 1);
}

#[derive(Debug)]
struct CallbackUnwind;
#[test]
fn owned_callback_unwind_restores_the_original_context_for_another_owned_lease() {
    let mut owner = context();
    let owner_id = owner.id();
    let drops = Rc::new(Cell::new(0));
    let input = payload(&drops);
    let weak = Rc::downgrade(&input);
    let called = Cell::new(0);
    let outcome = owner
        .with_execution(ExecutionOptions::new(512), |access| {
            let holder = access.into_host_access();
            let unwind = catch_unwind(AssertUnwindSafe(|| {
                holder.with_owned(input, |input, mut access| -> JsResult<()> {
                    called.set(called.get() + 1);
                    assert_eq!(access.id(), owner_id);
                    assert!(Rc::ptr_eq(&input, &weak.upgrade().unwrap()));
                    assert_eq!(input.message, "original owned input");
                    access
                        .eval(Source::from_bytes("globalThis.ownedCommitted = 7"))
                        .unwrap();
                    std::panic::panic_any(CallbackUnwind);
                })
            }))
            .unwrap_err();
            assert!(unwind.is::<CallbackUnwind>());
            assert_eq!(called.get(), 1);
            assert!(weak.upgrade().is_none());
            assert_eq!(drops.get(), 1);
            holder
                .with_owned(35, |increment, mut access| {
                    assert_eq!(access.id(), owner_id);
                    assert_eq!(increment, 35);
                    access.eval(Source::from_bytes("ownedCommitted += 35"))
                })
                .unwrap()
        })
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    assert_eq!(called.get(), 1);
    assert_eq!(drops.get(), 1);
    let checked = owner
        .with_execution(ExecutionOptions::new(512), |access| {
            access
                .into_host_access()
                .with_owned((), |(), mut access| {
                    assert_eq!(access.id(), owner_id);
                    access.eval(Source::from_bytes("ownedCommitted"))
                })
                .unwrap()
        })
        .unwrap();
    assert_eq!(checked.result.unwrap(), JsValue::new(42));
}
