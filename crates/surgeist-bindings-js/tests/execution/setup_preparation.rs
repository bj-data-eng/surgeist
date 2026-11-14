use super::*;
use surgeist_bindings_js::preparation::{Admission, Request, Retention};

#[derive(Debug)]
enum SetupError {
    Binding(Error),
    Refused,
}

impl From<Error> for SetupError {
    fn from(error: Error) -> Self {
        Self::Binding(error)
    }
}

struct Backing {
    live: Rc<Cell<usize>>,
    bytes: usize,
}

impl Backing {
    fn new(live: &Rc<Cell<usize>>, bytes: usize) -> Self {
        live.set(live.get() + bytes);
        Self {
            live: live.clone(),
            bytes,
        }
    }
}

impl Drop for Backing {
    fn drop(&mut self) {
        self.live.set(self.live.get() - self.bytes);
    }
}

impl Retention for Backing {
    fn release(self: Box<Self>) -> Result<(), Error> {
        drop(self);
        Ok(())
    }
}

struct ProcessBacking {
    backing: Backing,
    disposed: Rc<Cell<bool>>,
}

impl Drop for ProcessBacking {
    fn drop(&mut self) {
        // A new owner can be acquired only after the original isolate exits.
        let next = Process::bootstrap(EngineKind::V8);
        self.disposed.set(next.is_ok());
        drop(next);
        assert!(self.backing.live.get() > 0);
    }
}

fn direct_setup_refusal_preserves_cause_and_releases_singleton() -> Result<(), Error> {
    let refused = Process::bootstrap_funded(EngineKind::V8, |request| {
        assert!(matches!(request, Request::Process { bytes } if bytes > 0));
        Err::<(), _>(SetupError::Refused)
    });
    assert!(matches!(refused, Err(SetupError::Refused)));

    let live = Rc::new(Cell::new(0));
    let disposed = Rc::new(Cell::new(false));
    let process = Process::bootstrap_funded(EngineKind::V8, |request| {
        Ok::<_, Error>(ProcessBacking {
            backing: Backing::new(&live, request.bytes()),
            disposed: disposed.clone(),
        })
    })?;
    assert!(live.get() > 0);
    assert!(!process.has_preparation_admission());
    let called = Cell::new(false);
    let duplicate = Process::bootstrap_funded(EngineKind::V8, |_| {
        called.set(true);
        Ok::<_, SetupError>(())
    });
    assert!(matches!(
        duplicate,
        Err(SetupError::Binding(Error::OwnerAlreadyActive))
    ));
    assert!(!called.get());
    drop(process);
    assert!(disposed.get());
    assert_eq!(live.get(), 0);

    let mut process = Process::bootstrap(EngineKind::V8)?;
    let refused = process.execution().create_realm_funded(|request| {
        assert!(matches!(request, Request::Realm { bytes } if bytes > 0));
        Err::<(), _>(SetupError::Refused)
    });
    assert!(matches!(refused, Err(SetupError::Refused)));
    let realm = process.execution().create_realm()?;
    assert_eq!(
        evaluate(&mut process.execution(), &realm, "40 + 2")?,
        units("42")
    );
    Ok(())
}

fn realm_backing_follows_surviving_tokens_and_guest_contexts() -> Result<(), Error> {
    let live = Rc::new(Cell::new(0));
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let bytes = process.realm_storage_bytes() + size_of::<Backing>();
    let realm = process.execution().create_realm_funded(|request| {
        assert_eq!(request, Request::Realm { bytes });
        Ok::<_, Error>(Backing::new(&live, request.bytes()))
    })?;
    assert_eq!(live.get(), bytes);
    drop(process);
    assert_eq!(live.get(), bytes);
    drop(realm);
    assert_eq!(live.get(), 0);

    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process
        .execution()
        .create_realm_funded(|request| Ok::<_, Error>(Backing::new(&live, request.bytes())))?;
    let other = process.execution().create_realm()?;
    let function = process.execution().in_realm(&realm, |scope| {
        let value = scope.evaluate(&units("() => 42"))?;
        Ok(scope.retain(&value))
    })?;
    drop(realm);
    process.execution().memory_pressure();
    assert!(live.get() > 0);
    process.execution().in_realm(&other, |scope| {
        let function = scope.import(&function)?;
        let receiver = scope.global()?;
        let value = scope.call(&function, &receiver, &[])?;
        assert_eq!(scope.to_utf16(&value)?, units("42"));
        Ok(())
    })?;
    drop(function);
    drop(process);
    assert_eq!(live.get(), 0);
    Ok(())
}

fn installed_policy_owns_later_realm_admission() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let requests = Rc::new(RefCell::new(Vec::new()));
    let live = Rc::new(Cell::new(0));
    let admitted_requests = requests.clone();
    let admitted_live = live.clone();
    let controller = Admission::new(
        move |request| {
            admitted_requests.borrow_mut().push(request);
            Ok(Backing::new(&admitted_live, request.bytes()))
        },
        |_| Ok(()),
    )
    .install(&mut process, ())?;
    let called = Cell::new(false);
    let refused = process.execution().create_realm_funded(|_| {
        called.set(true);
        Ok::<_, Error>(())
    });
    assert!(matches!(refused, Err(Error::PreparationAlreadyAdmitted)));
    assert!(!called.get());
    assert!(requests.borrow().is_empty());
    let realm = process.execution().create_realm()?;
    assert!(matches!(requests.borrow().as_slice(), [Request::Realm { bytes }] if *bytes > 0));
    assert!(live.get() > 0);
    drop(process);
    assert!(live.get() > 0);
    drop(realm);
    assert_eq!(live.get(), 0);
    assert!(controller.observation().cleanup_error().is_none());
    Ok(())
}

pub(super) fn setup_preparation_contracts() -> Result<(), Error> {
    direct_setup_refusal_preserves_cause_and_releases_singleton()?;
    realm_backing_follows_surviving_tokens_and_guest_contexts()?;
    installed_policy_owns_later_realm_admission()
}
