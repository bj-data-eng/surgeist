use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use surgeist_bindings_js::conversion::{
    self, ConversionAccount, Converted, EnumAssignment, IntegerType, Resource, ResourceTable, Work,
    WorkTable, types,
};
use surgeist_bindings_js::declarations::ShadowRootMode;
use surgeist_bindings_js::{
    Callback, Completion, DomString, EngineKind, Error, IntegerKind, IntegerMode, IntegerValue,
    Primitive, Process, PropertyKey, Realm,
};
use surgeist_ledger::{EventFilter, Limits, Operation, ProfileMode, SharedOperation, Tariffs};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn new_ledger(
    mode: ProfileMode,
) -> Result<SharedOperation<Work, Resource>, conversion::LedgerError> {
    Operation::with_profile("host", Limits::unlimited(), 8, mode).map(Operation::share)
}

fn account(
    process: &mut Process,
    realm: &Realm,
    ledger: &SharedOperation<Work, Resource>,
    limits: Limits<Work, Resource>,
) -> Result<ConversionAccount, Error> {
    process.execution().in_realm(realm, |scope| {
        ConversionAccount::new(
            scope,
            ledger.clone(),
            ledger.root(),
            limits,
            Tariffs::identity(),
        )
    })
}

fn callback_and_owned_output(process: &mut Process) -> TestResult {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    drop(execution);
    let ledger = new_ledger(ProfileMode::Summary { timing: false })?;
    let account = account(process, &realm, &ledger, Limits::unlimited())?;
    let accepted = Rc::new(Cell::new(0));
    let retained: Rc<RefCell<Option<Converted<DomString>>>> = Rc::new(RefCell::new(None));
    let calls = accepted.clone();
    let output = retained.clone();
    let conversion = account.clone();
    let callback = Callback::new(move |scope, arguments| {
        let mode_input = arguments.get(0).ok_or(Error::BackendUnavailable)?;
        let text_input = arguments.get(1).ok_or(Error::BackendUnavailable)?;
        let (mode, text) = conversion.run(scope, |cx| {
            let mode = cx.value(
                conversion::declarations::shadow_root_mode().value_type(),
                mode_input,
            )?;
            let text = cx.value(&types::DOM_STRING, text_input)?;
            Ok((mode, text))
        })?;
        scope.check_active()?;
        if *mode.get() == ShadowRootMode::Open {
            calls.set(calls.get() + 1);
        }
        output.replace(Some(text));
        Ok(Completion::Undefined)
    });
    // Missing arguments are not part of this consumer's declared input. The
    // actual optional/default argument layer remains a subsequent contribution.
    let mut execution = process.execution();
    execution.in_realm(&realm, |scope| {
        let function = scope.function(&callback)?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("accept")), &function)?;
        let value = scope.evaluate(&units("globalThis.calls = 0; accept({[Symbol.toPrimitive](hint) { calls++; if (hint !== 'string') throw 1; return 'open' }}, '\\0\\uD800x'); calls"))?;
        assert_eq!(scope.to_utf16(&value)?, units("1"));
        Ok(())
    })?;
    assert_eq!(accepted.get(), 1);
    assert_eq!(
        retained
            .borrow()
            .as_ref()
            .expect("retained output")
            .get()
            .as_code_units(),
        [0, 0xd800, 0x78]
    );
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 6);
    retained.replace(None);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    assert_eq!(account.cleanup_status().failures(), Some(0));
    Ok(())
}

fn enum_errors_and_assignment(process: &mut Process) -> TestResult {
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let account = account(process, &realm, &ledger, Limits::unlimited())?;
    let assigned = Rc::new(Cell::new(0));
    let conversion = account.clone();
    let convert = Callback::new(move |scope, arguments| {
        let input = arguments.get(0).ok_or(Error::BackendUnavailable)?;
        conversion.run(scope, |cx| {
            cx.value(
                conversion::declarations::shadow_root_mode().value_type(),
                input,
            )
        })?;
        Ok(Completion::Undefined)
    });
    let conversion = account.clone();
    let effects = assigned.clone();
    let assign = Callback::new(move |scope, arguments| {
        let input = arguments.get(0).ok_or(Error::BackendUnavailable)?;
        let result = conversion.run(scope, |cx| {
            cx.enum_assignment(&conversion::declarations::shadow_root_mode(), input)
        })?;
        scope.check_active()?;
        if matches!(result, EnumAssignment::Member(_)) {
            effects.set(effects.get() + 1);
        }
        Ok(Completion::Undefined)
    });
    process.execution().in_realm(&realm, |scope| {
        let global = scope.global()?;
        for (name, callback) in [("convert", &convert), ("assign", &assign)] {
            let function = scope.function(callback)?;
            scope.set(&global, PropertyKey::String(&units(name)), &function)?;
        }
        let result = scope.evaluate(&units("globalThis.NativeTypeError = TypeError; globalThis.TypeError = null; globalThis.coercions = 0; let ok = false; try { convert({toString() {coercions++; return 'BAD'}}) } catch(e) {ok = Object.getPrototypeOf(e) === NativeTypeError.prototype} assign({toString() {coercions++; return 'BAD'}}); assign('closed'); globalThis.original = {toString(){throw 3}}; try { convert({[Symbol.toPrimitive](){throw original}}) } catch(e) {ok = ok && e === original} ok && coercions === 2"))?;
        assert!(scope.boolean(&result)?);
        Ok(())
    })?;
    assert_eq!(assigned.get(), 1);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    // A handled original guest failure leaves the owner usable.
    process.execution().in_realm(&realm, |scope| {
        let result = scope.evaluate(&units("40 + 2"))?;
        assert_eq!(scope.to_utf16(&result)?, units("42"));
        Ok(())
    })?;
    Ok(())
}

fn scalars_and_nested_reentry(process: &mut Process) -> TestResult {
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Summary { timing: false })?;
    let account = account(process, &realm, &ledger, Limits::unlimited())?;
    let nested = account.clone();
    let callback = Callback::new(move |scope, arguments| {
        let input = arguments.get(0).ok_or(Error::BackendUnavailable)?;
        let value = nested.run(scope, |cx| cx.value(&types::BOOLEAN, input))?;
        Ok(Completion::Return(
            scope.primitive(Primitive::Boolean(*value.get()))?,
        ))
    });
    process.execution().in_realm(&realm, |scope| {
        let global = scope.global()?;
        let function = scope.function(&callback)?;
        scope.set(&global, PropertyKey::String(&units("nested")), &function)?;
        let input = scope.evaluate(&units(
            "({toString() {if (!nested(true)) throw 1; return '\\uD800A\\uD83D\\uDE00'}})",
        ))?;
        let value = account.run(scope, |cx| cx.value(&types::USV_STRING, &input))?;
        assert_eq!(value.get().as_code_units(), [0xfffd, 65, 0xd83d, 0xde00]);
        drop(value);
        let input = scope.string(&[0, 255])?;
        let value = account.run(scope, |cx| cx.value(&types::BYTE_STRING, &input))?;
        assert_eq!(value.get().as_bytes(), [0, 255]);
        drop(value);
        let input = scope.string(&[256])?;
        let error = match account.run(scope, |cx| cx.value(&types::BYTE_STRING, &input)) {
            Err(error) => error,
            Ok(_) => panic!("ByteString must reject"),
        };
        assert_eq!(
            error.kind(),
            conversion::FailureKind::TypeError(conversion::Rule::ByteString)
        );
        let number = scope.primitive(Primitive::Number(-2.0))?;
        let integer = account.run(scope, |cx| {
            cx.integer(
                IntegerType::new(IntegerKind::UnsignedLongLong, IntegerMode::Wrap),
                &number,
            )
        })?;
        assert_eq!(*integer.get(), IntegerValue::UnsignedLongLong(u64::MAX - 1));
        let infinite = scope.primitive(Primitive::Number(f64::INFINITY))?;
        let failure = match account.run(scope, |cx| cx.value(&types::FLOAT, &infinite)) {
            Err(error) => error,
            Ok(_) => panic!("restricted float must reject"),
        };
        assert_eq!(
            failure.kind(),
            conversion::FailureKind::TypeError(conversion::Rule::FloatingRange)
        );
        let value = account.run(scope, |cx| cx.value(&types::UNRESTRICTED_DOUBLE, &infinite))?;
        assert_eq!(*value.get(), f64::INFINITY);
        let out_of_range = scope.primitive(Primitive::Number(256.0))?;
        let failure = match account.run(scope, |cx| {
            cx.integer(
                IntegerType::new(IntegerKind::Octet, IntegerMode::EnforceRange),
                &out_of_range,
            )
        }) {
            Err(error) => error,
            Ok(_) => panic!("octet must reject"),
        };
        assert_eq!(
            failure.kind(),
            conversion::FailureKind::TypeError(conversion::Rule::IntegerRange)
        );
        let input = scope.evaluate(&units(
            "globalThis.numberCalls = 0; ({valueOf(){numberCalls++;return 42}})",
        ))?;
        let value = account.run(scope, |cx| cx.value(&types::DOUBLE, &input))?;
        assert_eq!(*value.get(), 42.0);
        let calls = scope.evaluate(&units("numberCalls"))?;
        assert_eq!(scope.to_utf16(&calls)?, units("1"));
        let nan = scope.primitive(Primitive::Number(f64::NAN))?;
        let value = account.run(scope, |cx| cx.value(&types::UNRESTRICTED_FLOAT, &nan))?;
        assert_eq!(value.get().to_bits(), 0x7fc0_0000);
        let symbol = scope.evaluate(&units("Symbol()"))?;
        let failure = match account.run(scope, |cx| cx.value(&types::DOUBLE, &symbol)) {
            Err(error) => error,
            Ok(_) => panic!("ToNumber(Symbol) throws"),
        };
        assert_eq!(failure.kind(), conversion::FailureKind::GuestException);
        Ok(())
    })?;
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    assert!(
        ledger
            .snapshot()?
            .expect("profile")
            .scopes
            .iter()
            .any(|scope| scope.name == "js-conversion" && scope.phases >= 2)
    );
    Ok(())
}

fn admission_is_exact_and_terminal(storage_short: bool) -> TestResult {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let limits = Limits::new(
        WorkTable {
            Value: 1,
            CodeUnit: if storage_short { 3 } else { 2 },
            EnumerationLookup: 0,
        },
        ResourceTable {
            BufferBytes: if storage_short { 5 } else { 6 },
        },
    );
    let account = account(&mut process, &realm, &ledger, limits)?;
    let effects = Rc::new(Cell::new(0));
    let count = effects.clone();
    let callback = Callback::new(move |scope, arguments| {
        let input = arguments.get(0).ok_or(Error::BackendUnavailable)?;
        account.run(scope, |cx| cx.value(&types::DOM_STRING, input))?;
        scope.check_active()?;
        count.set(count.get() + 1);
        Ok(Completion::Undefined)
    });
    let result = process.execution().in_realm(&realm, |scope| {
        let function = scope.function(&callback)?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("convert")), &function)?;
        let input = scope.evaluate(&units("try { convert('abc') } catch (_) {}"));
        assert!(matches!(input, Err(Error::ConversionLedger(_))));
        Ok(()) // Catching the Rust result cannot turn terminal failure into success.
    });
    assert!(matches!(result, Err(Error::ConversionLedger(_))));
    assert_eq!(effects.get(), 0);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    assert_eq!(
        ledger.usage(ledger.root())?.peak(Resource::BufferBytes),
        if storage_short { 0 } else { 6 }
    );
    assert!(matches!(
        process.execution().create_realm(),
        Err(Error::ConversionLedger(_))
    ));
    Ok(())
}

fn exact_admission_and_cross_realm_account(process: &mut Process) -> TestResult {
    let a = process.execution().create_realm()?;
    let b = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let limits = Limits::new(
        WorkTable {
            Value: 1,
            CodeUnit: 3,
            EnumerationLookup: 0,
        },
        ResourceTable { BufferBytes: 6 },
    );
    let exact = account(process, &a, &ledger, limits)?;
    let retained = process.execution().in_realm(&a, |scope| {
        let input = scope.string(&units("abc"))?;
        exact
            .run(scope, |cx| cx.value(&types::DOM_STRING, &input))
            .map_err(Error::from)
    })?;
    assert_eq!(ledger.usage(ledger.root())?.work(Work::Value), 1);
    assert_eq!(ledger.usage(ledger.root())?.work(Work::CodeUnit), 3);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 6);
    drop(retained);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);

    let other = new_ledger(ProfileMode::Off)?;
    let shared = account(process, &a, &other, Limits::unlimited())?;
    let callback = Callback::new(move |scope, arguments| {
        let input = arguments.get(0).ok_or(Error::BackendUnavailable)?;
        let member = shared.run(scope, |cx| {
            cx.value(
                conversion::declarations::shadow_root_mode().value_type(),
                input,
            )
        })?;
        Ok(Completion::Return(scope.primitive(Primitive::Boolean(
            *member.get() == ShadowRootMode::Open,
        ))?))
    });
    let function = process.execution().in_realm(&a, |scope| {
        let function = scope.function(&callback)?;
        Ok(scope.retain(&function))
    })?;
    drop(callback);
    drop(a);
    process.execution().memory_pressure();
    process.execution().in_realm(&b, |scope| {
        let function = scope.import(&function)?;
        let receiver = scope.primitive(Primitive::Undefined)?;
        let input = scope.string(&units("open"))?;
        let value = scope.call(&function, &receiver, &[input])?;
        assert!(scope.boolean(&value)?);
        Ok(())
    })?;
    assert_eq!(other.usage(other.root())?.live(Resource::BufferBytes), 0);
    Ok(())
}

static CLOCK: AtomicU64 = AtomicU64::new(0);
fn clock() -> Duration {
    Duration::from_nanos(CLOCK.fetch_add(1, Ordering::Relaxed))
}

fn timing_is_ledger_owned(process: &mut Process) -> TestResult {
    let realm = process.execution().create_realm()?;
    for mode in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 16,
            filter: EventFilter::ScopeName("excluded"),
        },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 16,
            filter: EventFilter::ScopeName("js-conversion"),
        },
    ] {
        CLOCK.store(0, Ordering::Relaxed);
        let ledger =
            Operation::with_profile_clock("host", Limits::unlimited(), 4, mode, clock)?.share();
        let account = account(process, &realm, &ledger, Limits::unlimited())?;
        process.execution().in_realm(&realm, |scope| {
            let input = scope.string(&units("abc"))?;
            let value = account.run(scope, |cx| cx.value(&types::DOM_STRING, &input))?;
            assert_eq!(value.get().as_code_units(), units("abc"));
            Ok(())
        })?;
        let usage = ledger.usage(ledger.root())?;
        assert_eq!(usage.work(Work::Value), 1);
        assert_eq!(usage.work(Work::CodeUnit), 3);
        assert_eq!(usage.peak(Resource::BufferBytes), 6);
        assert_eq!(usage.live(Resource::BufferBytes), 0);
        match mode {
            ProfileMode::Summary { timing: true } | ProfileMode::Detailed { timing: true, .. } => {
                assert!(CLOCK.load(Ordering::Relaxed) > 0)
            }
            _ => assert_eq!(CLOCK.load(Ordering::Relaxed), 0),
        }
        if let Some(report) = ledger.snapshot()? {
            assert_eq!(
                report
                    .scopes
                    .iter()
                    .find(|scope| scope.name == "js-conversion")
                    .expect("conversion scope")
                    .phases,
                1
            );
            if matches!(
                mode,
                ProfileMode::Detailed {
                    filter: EventFilter::ScopeName("excluded"),
                    ..
                }
            ) {
                assert!(report.events.is_empty());
            }
        }
    }
    Ok(())
}

fn retained_output_does_not_own_the_heap() -> TestResult {
    let ledger = new_ledger(ProfileMode::Off)?;
    let (retained, account) = {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let realm = process.execution().create_realm()?;
        let account = account(&mut process, &realm, &ledger, Limits::unlimited())?;
        let retained = process.execution().in_realm(&realm, |scope| {
            let input = scope.string(&units("abc"))?;
            account
                .run(scope, |cx| cx.value(&types::DOM_STRING, &input))
                .map_err(Error::from)
        })?;
        (retained, account)
    };
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 6);
    let mut next = Process::bootstrap(EngineKind::V8)?;
    let realm = next.execution().create_realm()?;
    let work = ledger.usage(ledger.root())?.work(Work::Value);
    let result = next.execution().in_realm(&realm, |scope| {
        account.run(scope, |_| Ok(())).map_err(Error::from)
    });
    assert!(matches!(result, Err(Error::ForeignOwner)));
    assert_eq!(ledger.usage(ledger.root())?.work(Work::Value), work);
    drop(retained);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    assert_eq!(account.cleanup_status().failures(), Some(0));
    Ok(())
}

fn main() -> TestResult {
    {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        callback_and_owned_output(&mut process)?;
        enum_errors_and_assignment(&mut process)?;
        scalars_and_nested_reentry(&mut process)?;
        timing_is_ledger_owned(&mut process)?;
        exact_admission_and_cross_realm_account(&mut process)?;
    }
    admission_is_exact_and_terminal(false)?;
    admission_is_exact_and_terminal(true)?;
    retained_output_does_not_own_the_heap()?;
    Ok(())
}
