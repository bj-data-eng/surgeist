use super::*;
use surgeist_bindings_js::capture::{Admission, Request, Retention};
use surgeist_ledger::{Change, Limits, Operation, ProfileMode, SharedOperation};

surgeist_ledger::dimensions! {
    enum Work => WorkTable { Capture }
}
surgeist_ledger::dimensions! {
    enum Resource => ResourceTable { Bytes }
}

struct Account {
    ledger: SharedOperation<Work, Resource>,
    requests: RefCell<Vec<Request>>,
    release_failure: Cell<bool>,
}

impl Account {
    fn new(work: u64, bytes: u64, mode: ProfileMode) -> Rc<Self> {
        Rc::new(Self {
            ledger: Operation::with_reserved_profile(
                "incoming",
                Limits::new(WorkTable::splat(work), ResourceTable::splat(bytes)),
                1,
                mode,
            )
            .unwrap()
            .share(),
            requests: RefCell::new(Vec::new()),
            release_failure: Cell::new(false),
        })
    }

    fn live(&self) -> u64 {
        self.ledger
            .usage(self.ledger.root())
            .unwrap()
            .live(Resource::Bytes)
    }

    fn work(&self) -> u64 {
        self.ledger
            .usage(self.ledger.root())
            .unwrap()
            .work(Work::Capture)
    }

    fn admit(self: &Rc<Self>, request: Request) -> Result<Backing, Error> {
        self.ledger
            .apply(self.ledger.root(), Change::Work(Work::Capture, 1))
            .map_err(|_| Error::TooManyArguments)?;
        let bytes = u64::try_from(request.bytes()).map_err(|_| Error::Allocation)?;
        self.ledger
            .apply(self.ledger.root(), Change::Acquire(Resource::Bytes, bytes))
            .map_err(|_| Error::Allocation)?;
        self.requests.borrow_mut().push(request);
        Ok(Backing {
            account: self.clone(),
            bytes,
        })
    }
}

struct Backing {
    account: Rc<Account>,
    bytes: u64,
}

impl Retention for Backing {
    fn release(self: Box<Self>) -> Result<(), Error> {
        self.account
            .ledger
            .apply(
                self.account.ledger.root(),
                Change::Release(Resource::Bytes, self.bytes),
            )
            .map_err(|_| Error::RegistryConflict)?;
        if self.account.release_failure.get() {
            Err(Error::BackendUnavailable)
        } else {
            Ok(())
        }
    }
}

fn install(
    process: &mut Process,
    account: Rc<Account>,
) -> Result<surgeist_bindings_js::capture::Observation, Error> {
    assert!(!process.has_callback_capture_admission());
    let policy = Admission::new(move |request| account.admit(request));
    assert!(policy.storage_bytes::<()>().is_some());
    let observation = policy.install(process, ())?;
    assert!(process.has_callback_capture_admission());
    assert!(matches!(
        Admission::new(|_| -> Result<Backing, Error> { unreachable!() }).install(process, ()),
        Err(Error::CallbackCaptureAlreadyAdmitted)
    ));
    Ok(observation)
}

fn callable_case(
    work: u64,
    bytes: u64,
    mode: ProfileMode,
    selected: usize,
) -> Result<(u64, bool), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let root_bytes = process.managed_object_storage_bytes() + size_of::<Backing>();
    let buffer_bytes = selected * size_of::<Value<'_>>() + size_of::<Backing>();
    let account = Account::new(work, bytes, mode);
    let called = Rc::new(Cell::new(false));
    let inside = called.clone();
    let captured = account.clone();
    let callback = Callback::new(move |_, arguments| {
        inside.set(true);
        assert_eq!(arguments.len(), selected);
        assert_eq!(
            captured.live(),
            u64::try_from((selected + 1) * root_bytes + buffer_bytes).unwrap()
        );
        let (_, values) = arguments.into_parts();
        let mut values = values.into_iter();
        for _ in 0..selected {
            assert!(values.next().is_some());
        }
        assert!(values.next().is_none());
        assert_eq!(
            captured.live(),
            u64::try_from((selected + 1) * root_bytes + buffer_bytes).unwrap()
        );
        drop(values);
        assert_eq!(
            captured.live(),
            u64::try_from((selected + 1) * root_bytes).unwrap()
        );
        Ok(Completion::Undefined)
    });
    let realm = process.execution().create_realm()?;
    process.execution().in_realm(&realm, |scope| {
        let function = scope.bounded_function(
            &callback,
            FunctionMetadata {
                name: &units("take"),
                length: 0,
            },
            if selected == 3 { u32::MAX } else { 2 },
        )?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("take")), &function)
    })?;
    // Functions installed before the process policy must use it too.
    let observation = install(&mut process, account.clone())?;
    let result = process.execution().in_realm(&realm, |scope| {
        scope
            .evaluate(&units(
                "take(1, undefined, {toString(){throw 'ignored'}}); 'done'",
            ))
            .map(|_| ())
    });
    assert_eq!(account.live(), 0);
    assert!(observation.cleanup_error().is_none());
    if work >= u64::try_from(selected + 2).unwrap()
        && bytes >= u64::try_from((selected + 1) * root_bytes + buffer_bytes).unwrap()
    {
        result?;
        assert!(called.get());
        let requests = account.requests.borrow();
        let mut expected = vec![
            Request::RootRecord { bytes: root_bytes },
            Request::Buffer {
                bytes: buffer_bytes,
            },
        ];
        expected.extend(std::iter::repeat_n(
            Request::RootRecord { bytes: root_bytes },
            selected,
        ));
        assert_eq!(*requests, expected);
    } else {
        assert!(!called.get());
        assert!(matches!(
            result,
            Err(Error::Allocation | Error::TooManyArguments)
        ));
    }
    Ok((account.work(), called.get()))
}

pub(super) fn boundaries_and_retained_roots() -> Result<(), Error> {
    let process = Process::bootstrap(EngineKind::V8)?;
    let root_bytes = process.managed_object_storage_bytes() + size_of::<Backing>();
    let buffer_bytes = 2 * size_of::<Value<'_>>() + size_of::<Backing>();
    drop(process);
    let peak = u64::try_from(3 * root_bytes + buffer_bytes).unwrap();
    for mode in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 2,
            filter: surgeist_ledger::EventFilter::All,
        },
    ] {
        assert_eq!(callable_case(4, peak, mode, 2)?, (4, true));
        assert_eq!(callable_case(3, peak, mode, 2)?, (3, false));
        assert_eq!(callable_case(4, peak - 1, mode, 2)?, (4, false));
    }
    let full_peak =
        u64::try_from(4 * root_bytes + 3 * size_of::<Value<'_>>() + size_of::<Backing>()).unwrap();
    assert_eq!(callable_case(5, full_peak, ProfileMode::Off, 3)?, (5, true));
    assert_eq!(
        callable_case(4, full_peak, ProfileMode::Off, 3)?,
        (4, false)
    );
    assert_eq!(
        callable_case(5, full_peak - 1, ProfileMode::Off, 3)?,
        (5, false)
    );
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let account = Account::new(100, 1_000_000, ProfileMode::Off);
    install(&mut process, account.clone())?;
    let retained = Rc::new(RefCell::new(None));
    let output = retained.clone();
    let callback = Callback::new(move |scope, arguments| {
        output.replace(Some(scope.retain(arguments.get(0).unwrap())));
        Ok(Completion::Undefined)
    });
    let realm = process.execution().create_realm()?;
    process.execution().in_realm(&realm, |scope| {
        let function = scope.function(&callback)?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("keep")), &function)?;
        scope.evaluate(&units("keep({alive:true})"))?;
        Ok(())
    })?;
    assert_eq!(account.live(), u64::try_from(root_bytes).unwrap());
    retained.borrow_mut().take();
    process.execution().memory_pressure();
    assert_eq!(account.live(), 0);
    Ok(())
}

pub(super) fn constructors_and_nested_callbacks_share_admission() -> Result<(), Error> {
    for (maximum, shortage) in [0, 2, u32::MAX]
        .into_iter()
        .flat_map(|maximum| (0..3).map(move |shortage| (maximum, shortage)))
    {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let root_bytes = process.managed_object_storage_bytes() + size_of::<Backing>();
        let selected = usize::try_from(maximum.min(3)).unwrap();
        let work = u64::try_from(selected + 1 + usize::from(selected != 0)).unwrap();
        let buffer = if selected == 0 {
            0
        } else {
            selected * size_of::<Value<'_>>() + size_of::<Backing>()
        };
        let bytes = u64::try_from((selected + 1) * root_bytes + buffer).unwrap();
        let account = Account::new(
            work - u64::from(shortage == 1),
            bytes - u64::from(shortage == 2),
            ProfileMode::Off,
        );
        install(&mut process, account.clone())?;
        let called = Rc::new(Cell::new(false));
        let marker = called.clone();
        let retained = Rc::new(RefCell::new(None));
        let output = retained.clone();
        let constructor = ConstructorCallback::bounded(
            maximum,
            |scope| scope.object(ObjectPrototype::Default),
            move |scope, arguments| {
                marker.set(true);
                assert_eq!(arguments.len(), selected);
                output.replace(Some(scope.retain(arguments.new_target())));
                Ok(ConstructorCompletion::Return(
                    scope.object(ObjectPrototype::Default)?,
                ))
            },
        );
        let realm = process.execution().create_realm()?;
        process.execution().in_realm(&realm, |scope| {
            let constructor = scope.constructor(
                &constructor,
                FunctionMetadata {
                    name: &units("C"),
                    length: 0,
                },
            )?;
            let global = scope.global()?;
            scope.set(&global, PropertyKey::String(&units("C")), &constructor)?;
            let result = scope.evaluate(&units(
                "try { C(1,2,3) } catch(e) { e instanceof TypeError }",
            ))?;
            let yes = scope.primitive(Primitive::Boolean(true))?;
            assert!(scope.strict_equals(&result, &yes)?);
            Ok(())
        })?;
        assert!(!called.get());
        assert_eq!(account.work(), 0);
        let result = process.execution().in_realm(&realm, |scope| {
            scope.evaluate(&units("new C(1,2,3)"))?;
            Ok(())
        });
        if shortage != 0 {
            assert!(matches!(
                result,
                Err(Error::Allocation | Error::TooManyArguments)
            ));
            assert!(!called.get());
            assert_eq!(account.work(), work - u64::from(shortage == 1));
            assert_eq!(account.live(), 0);
            continue;
        }
        result?;
        assert!(called.get());
        let requests = account.requests.borrow();
        let mut expected = Vec::new();
        if selected != 0 {
            expected.push(Request::Buffer {
                bytes: selected * size_of::<Value<'_>>() + size_of::<Backing>(),
            });
        }
        expected.extend(std::iter::repeat_n(
            Request::RootRecord { bytes: root_bytes },
            selected + 1,
        ));
        assert_eq!(*requests, expected);
        drop(requests);
        assert_eq!(account.live(), u64::try_from(root_bytes).unwrap());
        retained.borrow_mut().take();
        process.execution().memory_pressure();
        assert_eq!(account.live(), 0);
    }
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let root_bytes = process.managed_object_storage_bytes() + size_of::<Backing>();
    let buffer_bytes = size_of::<Value<'_>>() + size_of::<Backing>();
    let account = Account::new(
        6,
        u64::try_from(4 * root_bytes + 2 * buffer_bytes).unwrap(),
        ProfileMode::Off,
    );
    install(&mut process, account.clone())?;
    let nested = Rc::new(Cell::new(false));
    let inside = nested.clone();
    let captured = account.clone();
    let callback = Callback::new(move |scope, arguments| {
        assert_eq!(arguments.len(), 1);
        if inside.replace(true) {
            assert_eq!(
                captured.live(),
                u64::try_from(4 * root_bytes + 2 * buffer_bytes).unwrap()
            );
        } else {
            scope.evaluate(&units("nested(undefined)"))?;
        }
        Ok(Completion::Undefined)
    });
    let realm = process.execution().create_realm()?;
    process.execution().in_realm(&realm, |scope| {
        let function = scope.function(&callback)?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("nested")), &function)?;
        scope.evaluate(&units("nested(1)"))?;
        Ok(())
    })?;
    assert!(nested.get());
    assert_eq!(account.work(), 6);
    assert_eq!(account.live(), 0);
    Ok(())
}

pub(super) fn failed_release_keeps_the_original_guest_exception() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let account = Account::new(100, 1_000_000, ProfileMode::Off);
    let observation = install(&mut process, account.clone())?;
    let failing = account.clone();
    let callback = Callback::new(move |_, arguments| {
        failing.release_failure.set(true);
        let (_, mut arguments) = arguments.into_parts();
        Ok(Completion::Throw(arguments.pop().unwrap()))
    });
    let realm = process.execution().create_realm()?;
    let result = process.execution().in_realm(&realm, |scope| {
        let function = scope.function(&callback)?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("fail")), &function)?;
        scope.evaluate(&units("globalThis.original={}; fail(original)"))?;
        Ok(())
    });
    let Err(Error::CallbackCaptureCleanup(failure)) = result else {
        panic!("unexpected {result:?}")
    };
    let Some(surgeist_bindings_js::conversion::CleanupPrimary::Exception(exception)) =
        failure.primary()
    else {
        panic!("lost primary {failure:?}")
    };
    assert_eq!(exception.phase(), ExceptionPhase::Invoke);
    assert!(matches!(
        failure.cleanup_error(),
        Some(Error::BackendUnavailable)
    ));
    assert!(matches!(
        observation.cleanup_error(),
        Some(Error::BackendUnavailable)
    ));
    // The selected exception intentionally retains its captured root until
    // process teardown, even though ordinary capture records have been reaped.
    assert!(account.live() > 0);
    drop(process);
    assert_eq!(account.live(), 0);
    Ok(())
}

pub(super) fn setup_custody_follows_surviving_observations() -> Result<(), Error> {
    struct Setup(Rc<Cell<usize>>);
    impl Drop for Setup {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let drops = Rc::new(Cell::new(0));
    let account = Account::new(100, 1_000_000, ProfileMode::Off);
    let observation = Admission::new(move |request| account.admit(request))
        .install(&mut process, Setup(drops.clone()))?;
    drop(process);
    assert_eq!(drops.get(), 0);
    assert!(observation.cleanup_error().is_none());
    drop(observation);
    assert_eq!(drops.get(), 1);
    Ok(())
}

pub(super) fn admission_refusals_and_prior_host_errors_are_terminal() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let original = process
        .execution()
        .in_realm(&realm, |scope| {
            scope.evaluate(&units("throw {original: true}")).map(|_| ())
        })
        .unwrap_err();
    assert!(matches!(original, Error::Exception(_)));
    Admission::new(move |_| -> Result<Backing, Error> { Err(original.clone()) })
        .install(&mut process, ())?;
    let calls = Rc::new(Cell::new(0));
    let marker = calls.clone();
    let callback = Callback::new(move |_, _| {
        marker.set(marker.get() + 1);
        Ok(Completion::Undefined)
    });
    let result = process.execution().in_realm(&realm, |scope| {
        let function = scope.function(&callback)?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("refuse")), &function)?;
        scope.evaluate(&units("try { refuse() } catch(e) { 42 }; 99"))?;
        Ok(())
    });
    assert!(matches!(result, Err(Error::Exception(_))));
    assert_eq!(calls.get(), 0);
    assert!(matches!(
        process.execution().in_realm(&realm, |_| Ok(())),
        Err(Error::Exception(_))
    ));
    drop(process);

    let mut process = Process::bootstrap(EngineKind::V8)?;
    let account = Account::new(100, 1_000_000, ProfileMode::Off);
    let observation = install(&mut process, account.clone())?;
    let failing = account.clone();
    let callback = Callback::new(move |_, _| {
        failing.release_failure.set(true);
        Err(Error::IdentityExhausted)
    });
    let realm = process.execution().create_realm()?;
    let result = process.execution().in_realm(&realm, |scope| {
        let function = scope.function(&callback)?;
        let receiver = scope.global()?;
        scope.call(&function, &receiver, &[])?;
        Ok(())
    });
    assert!(matches!(result, Err(Error::IdentityExhausted)));
    assert!(matches!(
        observation.cleanup_error(),
        Some(Error::BackendUnavailable)
    ));
    assert_eq!(account.live(), 0);
    assert!(matches!(
        process.execution().in_realm(&realm, |_| Ok(())),
        Err(Error::IdentityExhausted)
    ));
    Ok(())
}
