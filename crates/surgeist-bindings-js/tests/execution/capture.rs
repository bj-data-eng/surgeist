use super::*;
use surgeist_bindings_js::preparation::{
    Admission, BufferKind, Request, Retention, Work as PreparationWork,
};
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
            tracked: true,
        })
    }

    fn admit_incoming(self: &Rc<Self>, request: Request) -> Result<Backing, Error> {
        match request {
            Request::IncomingRoot { .. } | Request::IncomingBuffer { .. } => self.admit(request),
            _ => Ok(self.untracked()),
        }
    }

    fn admit_general_root(self: &Rc<Self>, request: Request) -> Result<Backing, Error> {
        match request {
            Request::RootRecord { .. } => self.admit(request),
            _ => Ok(self.untracked()),
        }
    }

    fn untracked(self: &Rc<Self>) -> Backing {
        Backing {
            account: self.clone(),
            bytes: 0,
            tracked: false,
        }
    }
}

struct Backing {
    account: Rc<Account>,
    bytes: u64,
    tracked: bool,
}

impl Backing {
    fn finish(self) -> Result<(), Error> {
        if !self.tracked {
            return Ok(());
        }
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

impl Retention for Backing {
    fn release(self: Box<Self>) -> Result<(), Error> {
        (*self).finish()
    }
}

fn install(
    process: &mut Process,
    account: Rc<Account>,
) -> Result<surgeist_bindings_js::preparation::Observation, Error> {
    assert!(!process.has_preparation_admission());
    // These capture regressions select only incoming physical records. General
    // preparation is exercised independently by the general-root cases below.
    let policy = Admission::new(move |request| account.admit_incoming(request), |_| Ok(()));
    assert!(policy.storage_bytes::<()>().is_some());
    let controller = policy.install(process, ())?;
    assert!(process.has_preparation_admission());
    assert!(matches!(
        Admission::new(|_| -> Result<Backing, Error> { unreachable!() }, |_| Ok(()))
            .install(process, ()),
        Err(Error::PreparationAlreadyAdmitted)
    ));
    Ok(controller.observation())
}

fn install_general_roots(process: &mut Process, account: Rc<Account>) -> Result<(), Error> {
    Admission::new(
        move |request| account.admit_general_root(request),
        |_| Ok(()),
    )
    .install(process, ())?;
    Ok(())
}

/// General value materialization owns a record, while importing or retaining
/// an existing identity shares that record without another admission.
pub(super) fn private_native_slots_admit_only_returned_value_records() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let (native, reference, token) = process.execution().in_realm(&realm, |scope| {
        let reference = scope.object(ObjectPrototype::Null)?;
        let native = scope.project_native_object_with_reference(
            ObjectPrototype::Null,
            7_u32,
            3,
            &reference,
            |_| Ok::<_, Error>(()),
        )?;
        let token = scope.native_object_token::<u32>(&native)?.unwrap();
        Ok((scope.retain(&native), scope.retain(&reference), token))
    })?;
    let bytes = process.managed_object_storage_bytes() + size_of::<Backing>();
    let account = Account::new(2, u64::try_from(2 * bytes).unwrap(), ProfileMode::Off);
    let admitted = account.clone();
    Admission::new(
        move |request| admitted.admit_general_root(request),
        |_| Ok(()),
    )
    .install(&mut process, ())?;
    process.execution().in_realm(&realm, |scope| {
        let original = scope.import(&native)?;
        let reference = scope.import(&reference)?;
        let opened = scope.open_native_object(&token)?.unwrap();
        assert!(scope.strict_equals(&original, &opened)?);
        let value = scope.native_object_reference::<u32>(&opened, 3)?.unwrap();
        assert!(scope.strict_equals(&reference, &value)?);
        assert_eq!(account.work(), 2);
        assert_eq!(account.live(), u64::try_from(2 * bytes).unwrap());
        scope.clear_native_object_reference(&token, 3)?;
        scope.clear_native_object_reference(&token, 3)?;
        assert!(scope.native_object_reference::<u32>(&opened, 3)?.is_none());
        scope.set_native_object_reference::<u32>(&opened, 4, Some(&reference))?;
        assert_eq!(scope.kind(&opened)?, ValueKind::Object);
        assert_eq!(account.work(), 2);
        Ok(())
    })?;
    assert_eq!(account.live(), 0);
    Ok(())
}

pub(super) fn native_slot_cleanup_preserves_terminal_refusal() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let (native, token) = process.execution().in_realm(&realm, |scope| {
        let reference = scope.object(ObjectPrototype::Null)?;
        let native = scope.project_native_object_with_reference(
            ObjectPrototype::Null,
            7_u32,
            3,
            &reference,
            |_| Ok::<_, Error>(()),
        )?;
        let token = scope.native_object_token::<u32>(&native)?.unwrap();
        Ok((scope.retain(&native), token))
    })?;
    let account = Account::new(0, 0, ProfileMode::Off);
    install_general_roots(&mut process, account.clone())?;
    let result = process.execution().in_realm(&realm, |scope| {
        assert!(matches!(
            scope.primitive(Primitive::Undefined),
            Err(Error::TooManyArguments)
        ));
        // Checking the selected failure also requests SDK termination. Cleanup
        // must still remove the private edge without reviving guest execution.
        assert!(matches!(scope.check_active(), Err(Error::TooManyArguments)));
        scope.clear_native_object_reference(&token, 3)?;
        scope.clear_native_object_reference(&token, 3)?;
        assert!(matches!(scope.check_active(), Err(Error::TooManyArguments)));
        assert_eq!(account.work(), 0);
        assert!(account.requests.borrow().is_empty());
        Ok(())
    });
    assert!(matches!(result, Err(Error::TooManyArguments)));
    drop((native, process));
    assert_eq!(account.live(), 0);

    // A later heap may reuse the native index. The original token still has
    // no authority to clear that heap's reference.
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let retired = process.execution().in_realm(&realm, |scope| {
        let reference = scope.object(ObjectPrototype::Null)?;
        let native = scope.project_native_object_with_reference(
            ObjectPrototype::Null,
            7_u32,
            3,
            &reference,
            |_| Ok::<_, Error>(()),
        )?;
        assert!(matches!(
            scope.clear_native_object_reference(&token, 3),
            Err(Error::ForeignOwner)
        ));
        let retained = scope.native_object_reference::<u32>(&native, 3)?.unwrap();
        assert!(scope.strict_equals(&retained, &reference)?);
        scope
            .native_object_token::<u32>(&native)?
            .ok_or(Error::ExpectedNativeObject)
    })?;
    process.execution().memory_pressure();
    process.execution().in_realm(&realm, |scope| {
        assert!(scope.open_native_object(&retired)?.is_none());
        scope.clear_native_object_reference(&retired, 3)?;
        scope.clear_native_object_reference(&retired, 3)?;
        Ok(())
    })?;
    Ok(())
}

pub(super) fn general_roots_keep_admission_until_reaping() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let bytes = process.managed_object_storage_bytes() + size_of::<Backing>();
    let account = Account::new(1, u64::try_from(bytes).unwrap(), ProfileMode::Off);
    install_general_roots(&mut process, account.clone())?;
    let root = process.execution().in_realm(&realm, |scope| {
        let value = scope.primitive(Primitive::Number(42.0))?;
        assert_eq!(*account.requests.borrow(), [Request::RootRecord { bytes }]);
        assert_eq!(account.live(), u64::try_from(bytes).unwrap());
        Ok(scope.retain(&value))
    })?;
    let clone = root.clone();
    process.execution().in_realm(&realm, |scope| {
        let imported = scope.import(&clone)?;
        assert_eq!(scope.kind(&imported)?, ValueKind::Number);
        let retained = scope.retain(&imported);
        drop(retained);
        assert_eq!(account.requests.borrow().len(), 1);
        assert_eq!(account.work(), 1);
        Ok(())
    })?;
    drop(root);
    process.execution().memory_pressure();
    assert_eq!(account.live(), u64::try_from(bytes).unwrap());
    drop(clone);
    process.execution().memory_pressure();
    assert_eq!(account.live(), 0);
    assert_eq!(account.work(), 1);

    drop(process);
    for (work, available, expected_work) in [(1, bytes - 1, 1), (0, bytes, 0)] {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let realm = process.execution().create_realm()?;
        let account = Account::new(work, u64::try_from(available).unwrap(), ProfileMode::Off);
        install_general_roots(&mut process, account.clone())?;
        let result = process.execution().in_realm(&realm, |scope| {
            scope.primitive(Primitive::Number(42.0)).map(|_| ())
        });
        if work == 0 {
            assert!(matches!(result, Err(Error::TooManyArguments)));
        } else {
            assert!(matches!(result, Err(Error::Allocation)));
        }
        assert!(account.requests.borrow().is_empty());
        assert_eq!(account.work(), expected_work);
        assert_eq!(account.live(), 0);
    }
    Ok(())
}

pub(super) fn evaluation_prepares_its_outcome_before_guest_entry() -> Result<(), Error> {
    for work in [1, 2] {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let realm = process.execution().create_realm()?;
        let called = Rc::new(Cell::new(false));
        let marker = called.clone();
        let callback = Callback::new(move |_, _| {
            marker.set(true);
            Ok(Completion::Undefined)
        });
        let original = process.execution().in_realm(&realm, |scope| {
            let function = scope.function(&callback)?;
            let global = scope.global()?;
            scope.set(&global, PropertyKey::String(&units("reached")), &function)?;
            let original = scope.object(ObjectPrototype::Default)?;
            scope.set(&global, PropertyKey::String(&units("original")), &original)?;
            Ok(scope.retain(&original))
        })?;
        let bytes = process.managed_object_storage_bytes() + size_of::<Backing>();
        // Compilation and execution each need an original-exception slot.
        // Successful compilation releases its unused slot before execution.
        let account = Account::new(work, u64::try_from(bytes).unwrap(), ProfileMode::Off);
        install_general_roots(&mut process, account.clone())?;
        let result = process.execution().in_realm(&realm, |scope| {
            scope
                .evaluate(&units("reached(); throw original"))
                .map(|_| ())
        });
        assert_eq!(account.work(), work);
        if work == 1 {
            assert!(matches!(result, Err(Error::TooManyArguments)));
            assert!(!called.get(), "refused execution must not enter guest code");
            assert_eq!(account.live(), 0);
            assert_eq!(*account.requests.borrow(), [Request::RootRecord { bytes }]);
            continue;
        }
        let Err(Error::Exception(exception)) = result else {
            panic!("expected the original guest exception, got {result:?}")
        };
        assert!(called.get());
        assert_eq!(exception.phase(), ExceptionPhase::Evaluate);
        assert_eq!(account.live(), u64::try_from(bytes).unwrap());
        assert_eq!(
            *account.requests.borrow(),
            [Request::RootRecord { bytes }, Request::RootRecord { bytes }]
        );
        // No work remains. Observing exact identity must share existing roots,
        // and throwing must have filled the admitted slot without new funding.
        process.execution().in_realm(&realm, |scope| {
            let original = scope.import(&original)?;
            let thrown = scope.import(exception.value())?;
            assert!(scope.strict_equals(&original, &thrown)?);
            Ok(())
        })?;
        assert_eq!(account.work(), 2);
        assert_eq!(account.requests.borrow().len(), 2);
        drop(exception);
        process.execution().memory_pressure();
        assert_eq!(account.live(), 0);
    }
    Ok(())
}

pub(super) fn utf16_output_keeps_its_buffer_admitted_after_process_teardown() -> Result<(), Error> {
    const ORIGINAL: [u16; 3] = [0, 0xd800, 0x61];
    let bytes = ORIGINAL.len() * size_of::<u16>() + size_of::<Backing>();
    // The selected tariff is one buffer allocation plus one unit per copied
    // code unit. General roots and other preparation have an explicit zero tariff.
    let exact_work = 1 + u64::try_from(ORIGINAL.len()).unwrap();
    for (work, available) in [
        (exact_work, bytes),
        (exact_work - 1, bytes),
        (exact_work, bytes - 1),
    ] {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let realm = process.execution().create_realm()?;
        let original = process.execution().in_realm(&realm, |scope| {
            let value = scope.string(&ORIGINAL)?;
            Ok(scope.retain(&value))
        })?;
        let account = Account::new(work, u64::try_from(available).unwrap(), ProfileMode::Off);
        let storage = account.clone();
        let copied = account.clone();
        Admission::new(
            move |request| match request {
                Request::Buffer {
                    kind: BufferKind::Utf16,
                    ..
                } => storage.admit(request),
                _ => Ok(storage.untracked()),
            },
            move |work| {
                let PreparationWork::CodeUnits { units } = work else {
                    return Ok(());
                };
                let units = u64::try_from(units).map_err(|_| Error::Allocation)?;
                copied
                    .ledger
                    .apply(copied.ledger.root(), Change::Work(Work::Capture, units))
                    .map_err(|_| Error::TooManyArguments)
            },
        )
        .install(&mut process, ())?;
        let result = process.execution().in_realm(&realm, |scope| {
            let value = scope.import(&original)?;
            scope.to_utf16(&value)
        });
        if available < bytes {
            assert!(matches!(result, Err(Error::Allocation)));
            assert_eq!(account.work(), 1);
            assert!(account.requests.borrow().is_empty());
            assert_eq!(account.live(), 0);
            continue;
        }
        assert_eq!(
            *account.requests.borrow(),
            [Request::Buffer {
                kind: BufferKind::Utf16,
                bytes,
                elements: ORIGINAL.len(),
            }]
        );
        if work < exact_work {
            assert!(matches!(result, Err(Error::TooManyArguments)));
            assert_eq!(account.work(), 1);
            assert_eq!(account.live(), 0);
            continue;
        }
        let output = result?;
        assert_eq!(output, ORIGINAL);
        assert_eq!(account.work(), exact_work);
        assert_eq!(account.live(), u64::try_from(bytes).unwrap());
        drop(process);
        assert_eq!(output, ORIGINAL);
        assert_eq!(account.live(), u64::try_from(bytes).unwrap());
        drop(output);
        assert_eq!(account.live(), 0);
    }
    Ok(())
}

struct RegistrationBacking {
    backing: Backing,
    callback_drops: Rc<Cell<usize>>,
}

impl Retention for RegistrationBacking {
    fn release(self: Box<Self>) -> Result<(), Error> {
        let Self {
            backing,
            callback_drops,
        } = *self;
        if backing.tracked {
            assert_eq!(
                callback_drops.get(),
                1,
                "release the callback before its backing"
            );
        }
        backing.finish()
    }
}

fn registration_case(
    capacity: u64,
    constructor: bool,
    collect: bool,
) -> Result<Option<usize>, Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let account = Account::new(1, capacity, ProfileMode::Off);
    let callback_drops = Rc::new(Cell::new(0));
    let drops = callback_drops.clone();
    let admitted = account.clone();
    Admission::new(
        move |request| {
            let backing = match request {
                Request::CallableRegistration { .. } => admitted.admit(request)?,
                _ => admitted.untracked(),
            };
            Ok(RegistrationBacking {
                backing,
                callback_drops: drops.clone(),
            })
        },
        |_| Ok(()),
    )
    .install(&mut process, ())?;
    let calls = Rc::new(Cell::new(0));
    let called = calls.clone();
    let receipt = DropReceipt(callback_drops.clone());
    let result = process.execution().in_realm(&realm, |scope| {
        let function = if constructor {
            let callback = ConstructorCallback::bounded(
                0,
                |scope| scope.object(ObjectPrototype::Default),
                move |scope, _| {
                    let _capture = &receipt;
                    called.set(called.get() + 1);
                    Ok(ConstructorCompletion::Return(
                        scope.object(ObjectPrototype::Default)?,
                    ))
                },
            );
            scope.constructor(
                &callback,
                FunctionMetadata {
                    name: &units("held"),
                    length: 0,
                },
            )?
        } else {
            let callback = Callback::new(move |_, _| {
                let _capture = &receipt;
                called.set(called.get() + 1);
                Ok(Completion::Undefined)
            });
            scope.function(&callback)?
        };
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("held")), &function)
    });
    assert_eq!(account.work(), 1);
    if matches!(result, Err(Error::Allocation)) {
        assert!(account.requests.borrow().is_empty());
        assert_eq!(account.live(), 0);
        assert_eq!(callback_drops.get(), 1);
        assert_eq!(calls.get(), 0);
        return Ok(None);
    }
    result?;
    let bytes = match account.requests.borrow().as_slice() {
        [Request::CallableRegistration { bytes }] => *bytes,
        requests => panic!("expected one admitted registration, got {requests:?}"),
    };
    // Scoped roots have ended, but the global guest reference keeps the exact
    // registration and its callback capture alive across repeated invocation.
    process.execution().memory_pressure();
    assert_eq!(account.live(), u64::try_from(bytes).unwrap());
    assert_eq!(callback_drops.get(), 0);
    process.execution().in_realm(&realm, |scope| {
        let source = if constructor {
            "new held(); new held()"
        } else {
            "held(); held()"
        };
        scope.evaluate(&units(source))?;
        Ok(())
    })?;
    assert_eq!(calls.get(), 2);
    assert_eq!(account.requests.borrow().len(), 1);
    assert_eq!(account.work(), 1);
    assert_eq!(account.live(), u64::try_from(bytes).unwrap());
    if collect {
        process.execution().in_realm(&realm, |scope| {
            scope.evaluate(&units("held = null; gc()"))?;
            Ok(())
        })?;
        process.execution().memory_pressure();
        assert_eq!(account.live(), 0);
        assert_eq!(callback_drops.get(), 1);
    }
    drop(process);
    assert_eq!(account.live(), 0);
    assert_eq!(callback_drops.get(), 1);
    Ok(Some(bytes))
}

pub(super) fn callable_registration_follows_guest_reachability_and_teardown() -> Result<(), Error> {
    for constructor in [false, true] {
        // The body is intentionally backend-private. Probe its public request,
        // then independently qualify exact and one-short admission boundaries.
        let bytes = registration_case(u64::MAX, constructor, true)?.unwrap();
        assert!(bytes > size_of::<RegistrationBacking>());
        assert_eq!(
            registration_case(u64::try_from(bytes).unwrap(), constructor, false)?,
            Some(bytes)
        );
        assert_eq!(
            registration_case(u64::try_from(bytes - 1).unwrap(), constructor, false)?,
            None
        );
    }
    Ok(())
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
            Request::IncomingRoot { bytes: root_bytes },
            Request::IncomingBuffer {
                bytes: buffer_bytes,
            },
        ];
        expected.extend(std::iter::repeat_n(
            Request::IncomingRoot { bytes: root_bytes },
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
            expected.push(Request::IncomingBuffer {
                bytes: selected * size_of::<Value<'_>>() + size_of::<Backing>(),
            });
        }
        expected.extend(std::iter::repeat_n(
            Request::IncomingRoot { bytes: root_bytes },
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
    let Err(Error::PreparationCleanup(failure)) = result else {
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
    // The selected exception retains its admitted identity metadata through
    // the last clone, alongside any backend record that still owns the value.
    assert!(account.live() > 0);
    drop(process);
    assert!(account.live() > 0);
    drop(failure);
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
    let controller = Admission::new(move |request| account.admit_incoming(request), |_| Ok(()))
        .install(&mut process, Setup(drops.clone()))?;
    let observation = controller.observation();
    drop(controller);
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
    let account = Account::new(100, 1_000_000, ProfileMode::Off);
    Admission::new(
        move |request| match request {
            Request::IncomingRoot { .. } | Request::IncomingBuffer { .. } => Err(original.clone()),
            _ => Ok(account.untracked()),
        },
        |_| Ok(()),
    )
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
