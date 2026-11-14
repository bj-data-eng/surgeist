use super::*;
use surgeist_bindings_js::Root;
use surgeist_bindings_js::preparation::{Admission, Request, Retention};

#[derive(Default)]
struct Accounting {
    live: Cell<usize>,
    released: Cell<usize>,
    requests: RefCell<Vec<Request>>,
    next_failure: RefCell<Option<Error>>,
}

impl Accounting {
    fn admit(self: &Rc<Self>, request: Request) -> Result<Backing, Error> {
        let root = matches!(
            request,
            Request::RootRecord { .. } | Request::IncomingRoot { .. }
        );
        let bytes = if root { request.bytes() } else { 0 };
        self.live.set(self.live.get() + bytes);
        self.requests.borrow_mut().push(request);
        Ok(Backing {
            account: self.clone(),
            bytes,
            failure: if root {
                self.next_failure.borrow_mut().take()
            } else {
                None
            },
        })
    }
}

struct Backing {
    account: Rc<Accounting>,
    bytes: usize,
    failure: Option<Error>,
}

impl Retention for Backing {
    fn release(self: Box<Self>) -> Result<(), Error> {
        self.account.live.set(self.account.live.get() - self.bytes);
        if self.bytes != 0 {
            self.account.released.set(self.account.released.get() + 1);
        }
        match self.failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

struct Setup(Rc<Cell<usize>>);

impl Drop for Setup {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn metadata_survives_process_for_each_admission_path() -> Result<(), Error> {
    for path in 0..3 {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let realm = process.execution().create_realm()?;
        let captured = Rc::new(RefCell::new(None));
        if path == 1 {
            let target = captured.clone();
            let callback = Callback::new(move |scope, arguments| {
                let (_, mut values) = arguments.into_parts();
                *target.borrow_mut() = Some(scope.retain(&values.pop().unwrap()));
                Ok(Completion::Undefined)
            });
            process.execution().in_realm(&realm, |scope| {
                let function = scope.function(&callback)?;
                let global = scope.global()?;
                scope.set(&global, PropertyKey::String(&units("capture")), &function)
            })?;
        }
        let account = Rc::new(Accounting::default());
        let policy = account.clone();
        let controller = Admission::new(move |request| policy.admit(request), |_| Ok(()))
            .install(&mut process, ())?;
        let bytes = process.managed_object_storage_bytes() + size_of::<Backing>();
        let root = process.execution().in_realm(&realm, |scope| {
            if path == 1 {
                scope.evaluate(&units("capture(42)"))?;
                return Ok(captured.borrow_mut().take().unwrap());
            }
            let value = if path == 2 {
                scope.primitive_funded(Primitive::Number(42.0), &controller, |request| {
                    account.admit(request)
                })?
            } else {
                scope.primitive(Primitive::Number(42.0))?
            };
            Ok(scope.retain(&value))
        })?;
        assert_eq!(account.live.get(), bytes);
        let admissions = account.requests.borrow().len();
        let clone = root.clone();
        process.execution().in_realm(&realm, |scope| {
            let value = scope.import(&clone)?;
            assert_eq!(scope.kind(&value)?, ValueKind::Number);
            drop(scope.retain(&value));
            Ok(())
        })?;
        assert_eq!(account.requests.borrow().len(), admissions);
        drop(controller);
        drop(process);
        assert_eq!(account.live.get(), bytes);
        let released = account.released.get();
        drop(root);
        assert_eq!(account.live.get(), bytes);
        drop(clone);
        assert_eq!(account.live.get(), 0);
        assert_eq!(account.released.get(), released + 1);
    }
    Ok(())
}

fn policy_captured_roots_do_not_retain_the_policy() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let account = Rc::new(Accounting::default());
    let captured: Rc<RefCell<Option<Root>>> = Rc::new(RefCell::new(None));
    let weak = Rc::downgrade(&captured);
    let policy_capture = captured.clone();
    let policy_account = account.clone();
    let drops = Rc::new(Cell::new(0));
    let controller = Admission::new(
        move |request| {
            let _occupied = policy_capture.borrow().is_some();
            policy_account.admit(request)
        },
        |_| Ok(()),
    )
    .install(&mut process, Setup(drops.clone()))?;
    let root = process.execution().in_realm(&realm, |scope| {
        let value = scope.primitive(Primitive::Undefined)?;
        Ok(scope.retain(&value))
    })?;
    *captured.borrow_mut() = Some(root.clone());
    drop(captured);
    drop(controller);
    drop(process);
    assert!(
        weak.upgrade().is_none(),
        "a surviving root retained its admission closure"
    );
    assert_eq!(drops.get(), 1);
    assert!(account.live.get() > 0);
    drop(root);
    assert_eq!(account.live.get(), 0);
    Ok(())
}

fn late_root_bearing_failure_keeps_the_original_error_without_a_cycle() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let account = Rc::new(Accounting::default());
    let policy = account.clone();
    let drops = Rc::new(Cell::new(0));
    let controller = Admission::new(move |request| policy.admit(request), |_| Ok(()))
        .install(&mut process, Setup(drops.clone()))?;
    let observation = controller.observation();
    let exception = process.execution().in_realm(&realm, |scope| {
        let value = scope.primitive(Primitive::Number(17.0))?;
        Ok(scope.invocation_exception(&value))
    })?;
    *account.next_failure.borrow_mut() = Some(Error::Exception(exception));
    let root = process.execution().in_realm(&realm, |scope| {
        let value = scope.primitive(Primitive::Undefined)?;
        Ok(scope.retain(&value))
    })?;
    drop(controller);
    drop(process);
    assert_eq!(drops.get(), 0);
    assert!(observation.cleanup_error().is_none());
    drop(root);
    assert!(
        matches!(observation.cleanup_error(), Some(Error::Exception(error)) if error.phase() == ExceptionPhase::Invoke)
    );
    assert!(
        account.live.get() > 0,
        "the stored original exception lost its root backing"
    );
    drop(observation);
    assert_eq!(account.live.get(), 0);
    assert_eq!(drops.get(), 1);
    Ok(())
}

fn late_release_finishes_without_an_observer() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let account = Rc::new(Accounting::default());
    *account.next_failure.borrow_mut() = Some(Error::Allocation);
    let policy = account.clone();
    let controller = Admission::new(move |request| policy.admit(request), |_| Ok(()))
        .install(&mut process, ())?;
    let root = process.execution().in_realm(&realm, |scope| {
        let value = scope.primitive(Primitive::Undefined)?;
        Ok(scope.retain(&value))
    })?;
    drop(controller);
    drop(process);
    assert_eq!(account.released.get(), 0);
    drop(root);
    assert_eq!(account.live.get(), 0);
    assert_eq!(account.released.get(), 1);
    Ok(())
}

pub(super) fn root_custody_contracts() -> Result<(), Error> {
    metadata_survives_process_for_each_admission_path()?;
    policy_captured_roots_do_not_retain_the_policy()?;
    late_root_bearing_failure_keeps_the_original_error_without_a_cycle()?;
    late_release_finishes_without_an_observer()
}
