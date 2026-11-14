//! Hosted admission through real guest effects and the original parent Ledger.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use surgeist_ledger::{
    Change, Limits, Operation, PhaseObservation, ProfileMode, SharedOperation, Tariff, Tariffs,
};

use super::{
    AdmissionError, BufferRequest, BufferRetention, CleanupPrimary, CleanupStage,
    ConversionAccount, Failure, Host, Hosted, LedgerError, Resource, ResourceTable, Work,
    WorkTable, declarations, types,
};
use crate::{Callback, Completion, EngineKind, Error, Process, PropertyKey, Realm, Root};

type TestResult = Result<(), Box<dyn std::error::Error>>;
type Ledger = SharedOperation<Work, Resource>;

fn units(source: &str) -> Vec<u16> {
    source.encode_utf16().collect()
}

struct Parent {
    ledger: Ledger,
    scope: surgeist_ledger::Scope,
    charges: RefCell<Vec<(Work, u64)>>,
    payloads: RefCell<Vec<u64>>,
    release_fault: Cell<bool>,
    phase_fault: Cell<bool>,
    releases: Cell<usize>,
}

impl Parent {
    fn new(work: u64, tariff: u64) -> Result<Rc<Self>, LedgerError> {
        let mut limits = WorkTable::splat(u64::MAX);
        limits.Value = work;
        let ledger = Operation::with_profile(
            "original-host",
            Limits::new(limits, ResourceTable::splat(u64::MAX)),
            2,
            ProfileMode::Summary { timing: false },
        )?
        .share();
        let scope = ledger.child(
            ledger.root(),
            "conversion",
            Limits::unlimited(),
            Tariffs::from_fn(
                |dimension| Tariff::new(dimension, tariff),
                |dimension| Tariff::new(dimension, 1),
            ),
        )?;
        Ok(Rc::new(Self {
            ledger,
            scope,
            charges: RefCell::new(Vec::new()),
            payloads: RefCell::new(Vec::new()),
            release_fault: Cell::new(false),
            phase_fault: Cell::new(false),
            releases: Cell::new(0),
        }))
    }

    fn work(&self) -> u64 {
        self.ledger
            .usage(self.ledger.root())
            .unwrap()
            .work(Work::Value)
    }
    fn live(&self) -> u64 {
        self.ledger
            .usage(self.scope)
            .unwrap()
            .live(Resource::BufferBytes)
    }

    fn underflow(&self) -> LedgerError {
        self.ledger
            .apply(self.scope, Change::Release(Resource::BufferBytes, u64::MAX))
            .unwrap_err()
    }
}

struct TestHost {
    parent: Rc<Parent>,
    _factory: Rc<()>,
}
struct Buffer {
    parent: Rc<Parent>,
    bytes: u64,
}

impl BufferRetention for Buffer {
    type Error = LedgerError;
    fn release(self) -> Result<(), LedgerError> {
        self.parent.ledger.apply(
            self.parent.scope,
            Change::Release(Resource::BufferBytes, self.bytes),
        )?;
        self.parent.releases.set(self.parent.releases.get() + 1);
        if self.parent.release_fault.replace(false) {
            Err(self.parent.underflow())
        } else {
            Ok(())
        }
    }
}

impl Host for TestHost {
    type CleanupError = LedgerError;
    type Buffer = Buffer;

    fn charge(&self, work: Work, units: u64) -> Result<(), Error> {
        self.parent.charges.borrow_mut().push((work, units));
        // The enclosing operation aggregates semantic attempts, with explicit
        // double-cost payload passes. This is host policy, not a producer cap.
        let amount = units
            .checked_mul(if work == Work::CodeUnit { 2 } else { 1 })
            .ok_or(Error::Allocation)?;
        self.parent
            .ledger
            .apply(self.parent.scope, Change::Work(Work::Value, amount))
            .map_err(Error::ConversionLedger)
    }

    fn retain_buffer(&self, request: BufferRequest) -> Result<Buffer, Error> {
        self.parent
            .payloads
            .borrow_mut()
            .push(request.payload_bytes());
        let bytes = u64::try_from(request.backing_bytes()).map_err(|_| Error::Allocation)?;
        self.parent
            .ledger
            .apply(
                self.parent.scope,
                Change::Acquire(Resource::BufferBytes, bytes),
            )
            .map_err(Error::ConversionLedger)?;
        Ok(Buffer {
            parent: self.parent.clone(),
            bytes,
        })
    }

    fn begin_phase(&self) -> Result<Option<PhaseObservation>, Error> {
        self.parent
            .ledger
            .begin_phase(self.parent.scope)
            .map(Some)
            .map_err(Error::ConversionLedger)
    }

    fn end_phase(&self, marker: PhaseObservation) -> Result<(), LedgerError> {
        assert!(
            self.parent.ledger.end_phase(marker).is_ok(),
            "finish the original parent marker"
        );
        if self.parent.phase_fault.replace(false) {
            Err(self.parent.underflow())
        } else {
            Ok(())
        }
    }
}

struct Setup {
    parent: Rc<Parent>,
    bytes: u64,
    drops: Rc<Cell<usize>>,
}
impl Drop for Setup {
    fn drop(&mut self) {
        assert!(
            self.parent
                .ledger
                .apply(
                    self.parent.ledger.root(),
                    Change::Release(Resource::BufferBytes, self.bytes)
                )
                .is_ok()
        );
        self.drops.set(self.drops.get() + 1);
    }
}

struct Fixture {
    process: Process,
    realm: Realm,
    parent: Rc<Parent>,
    account: ConversionAccount,
    factory: std::rc::Weak<()>,
    setup_drops: Rc<Cell<usize>>,
}

fn fixture(
    local: Limits<Work, Resource>,
    parent_work: u64,
    tariff: u64,
) -> Result<Fixture, Box<dyn std::error::Error>> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let parent = Parent::new(parent_work, tariff)?;
    let factory = Rc::new(());
    let weak = Rc::downgrade(&factory);
    let setup_drops = Rc::new(Cell::new(0));
    let prepared = Hosted::new(TestHost {
        parent: parent.clone(),
        _factory: factory,
    });
    let setup_bytes = u64::try_from(prepared.storage_bytes::<Setup>().ok_or(Error::Allocation)?)?;
    parent.ledger.apply(
        parent.ledger.root(),
        Change::Acquire(Resource::BufferBytes, setup_bytes),
    )?;
    let account = process.execution().in_realm(&realm, |scope| {
        prepared.install(
            scope,
            local,
            Setup {
                parent: parent.clone(),
                bytes: setup_bytes,
                drops: setup_drops.clone(),
            },
        )
    })?;
    Ok(Fixture {
        process,
        realm,
        parent,
        account,
        factory: weak,
        setup_drops,
    })
}

fn expose(scope: &mut crate::Scope<'_>, name: &str, callback: &Callback) -> Result<(), Error> {
    let global = scope.global()?;
    let function = scope.function(callback)?;
    scope.set(&global, PropertyKey::String(&units(name)), &function)
}

fn local_refusal_precedes_each_guest_effect() -> TestResult {
    for (dimension, source, parent_prefix) in [
        (Work::Value, "({toString(){effect();return 'ab'}})", 0),
        (
            Work::DictionaryMember,
            "({get bubbles(){effect();return true}})",
            1,
        ),
        (
            Work::IteratorOperation,
            "({get [Symbol.iterator](){effect();return function*(){yield true}}})",
            1,
        ),
    ] {
        let mut limits = WorkTable::splat(u64::MAX);
        match dimension {
            Work::Value => limits.Value = 0,
            Work::DictionaryMember => limits.DictionaryMember = 0,
            Work::IteratorOperation => limits.IteratorOperation = 0,
            _ => unreachable!(),
        }
        let Fixture {
            mut process,
            realm,
            parent,
            account,
            ..
        } = fixture(
            Limits::new(limits, ResourceTable::splat(u64::MAX)),
            u64::MAX,
            1,
        )?;
        let effects = Rc::new(Cell::new(0));
        let reached = effects.clone();
        let callback = Callback::new(move |_, _| {
            reached.set(reached.get() + 1);
            Ok(Completion::Undefined)
        });
        let result = process.execution().in_realm(&realm, |scope| {
            expose(scope, "effect", &callback)?;
            let input = scope.evaluate(&units(source))?;
            account
                .run(scope, |cx| match dimension {
                    Work::Value => cx.value(&types::DOM_STRING, &input).map(|_| ()),
                    Work::DictionaryMember => {
                        cx.value(&declarations::event_init(), &input).map(|_| ())
                    }
                    Work::IteratorOperation => {
                        cx.value(&types::BOOLEAN.sequence(), &input).map(|_| ())
                    }
                    _ => unreachable!(),
                })
                .map_err(Error::from)
        });
        assert!(
            matches!(result, Err(Error::ConversionAdmission(AdmissionError::Refused(refused))) if refused.change() == Change::Work(dimension, 1))
        );
        assert_eq!(effects.get(), 0, "local cap preceded guest getter/coercion");
        assert_eq!(parent.work(), parent_prefix);
        assert!(
            !parent
                .charges
                .borrow()
                .iter()
                .any(|(work, _)| *work == dimension)
        );
    }
    Ok(())
}

fn dictionary_exact_and_one_short_parent_budgets() -> TestResult {
    // Presence + dictionary value + 3 member visits + 3 booleans = 8.
    for (budget, tariff, local_values, accepted, attempts) in [
        (7, 1, u64::MAX, false, 8),
        (8, 1, u64::MAX, true, 8),
        (23, 3, u64::MAX, false, 8),
        (24, 3, u64::MAX, true, 8),
        (u64::MAX, 1, 3, false, 7),
        (u64::MAX, 1, 4, true, 8),
    ] {
        let mut local = WorkTable::splat(u64::MAX);
        local.Value = local_values;
        let Fixture {
            mut process,
            realm,
            parent,
            account,
            ..
        } = fixture(
            Limits::new(local, ResourceTable::splat(u64::MAX)),
            budget,
            tariff,
        )?;
        let markers = Rc::new(Cell::new(0));
        let marker = markers.clone();
        let callback = Callback::new(move |scope, arguments| {
            let descriptor = declarations::event_constructor_0_event_init_dict();
            let converted = account.run(scope, |cx| {
                cx.argument(&descriptor, arguments.get(descriptor.index()))
            })?;
            let super::ArgumentValue::Present(value) = converted.get() else {
                return Err(Error::BackendUnavailable);
            };
            assert!(value.bubbles && value.cancelable && value.composed);
            marker.set(marker.get() + 1);
            Ok(Completion::Undefined)
        });
        let result = process.execution().in_realm(&realm, |scope| {
            expose(scope, "convert", &callback)?;
            scope
                .evaluate(&units(
                    "convert('event', {bubbles:true,cancelable:true,composed:true})",
                ))
                .map(|_| ())
        });
        assert_eq!(result.is_ok(), accepted);
        assert_eq!(markers.get(), usize::from(accepted));
        assert_eq!(
            parent.work(),
            if accepted { 8 * tariff } else { 7 * tariff }
        );
        let expected = [
            (Work::Argument, 1),
            (Work::Value, 1),
            (Work::DictionaryMember, 1),
            (Work::Value, 1),
            (Work::DictionaryMember, 1),
            (Work::Value, 1),
            (Work::DictionaryMember, 1),
            (Work::Value, 1),
        ];
        assert_eq!(parent.charges.borrow().as_slice(), &expected[..attempts]);
        if !accepted {
            if local_values == 3 {
                assert!(
                    matches!(result, Err(Error::ConversionAdmission(AdmissionError::Refused(refused))) if refused.change() == Change::Work(Work::Value, 1) && refused.current() == 3)
                );
            } else {
                assert!(matches!(result, Err(Error::ConversionLedger(_))));
            }
        }
    }
    Ok(())
}

fn byte_string_overlap_obeys_payload_cap_before_parent_acquisition() -> TestResult {
    // Two UTF-16 code units occupy 4 bytes; the ByteString copy adds 2 while
    // UTF-16 remains live. Only the 6-byte cap can admit this overlap.
    for cap in [5, 6] {
        let Fixture {
            mut process,
            realm,
            parent,
            account,
            ..
        } = fixture(
            Limits::new(WorkTable::splat(u64::MAX), ResourceTable::splat(cap)),
            u64::MAX,
            1,
        )?;
        let result = process.execution().in_realm(&realm, |scope| {
            let input = scope.string(&units("ab"))?;
            account
                .run(scope, |cx| cx.value(&types::BYTE_STRING, &input))
                .map_err(Error::from)
        });
        if cap == 5 {
            assert!(
                matches!(result, Err(Error::ConversionAdmission(AdmissionError::Refused(refused))) if refused.change() == Change::Acquire(Resource::BufferBytes, 2) && refused.current() == 4)
            );
            assert_eq!(*parent.payloads.borrow(), [4]);
            assert_eq!(parent.work(), 9); // value + copy and inspection at tariff 2
        } else {
            let converted = result?;
            assert_eq!(converted.get().as_bytes(), b"ab");
            assert_eq!(*parent.payloads.borrow(), [4, 2]);
            assert!(parent.live() >= 2);
            drop(converted);
            // Value + UTF-16 copy + byte validation + byte copy: 1 + 4 + 4 + 4.
            assert_eq!(parent.work(), 13);
        }
        assert_eq!(parent.live(), 0);
    }
    Ok(())
}

fn same_account_guest_reentry_begins_after_host_admission_returns() -> TestResult {
    let Fixture {
        mut process,
        realm,
        parent,
        account,
        ..
    } = fixture(Limits::unlimited(), u64::MAX, 1)?;
    let nested = Rc::new(Cell::new(0));
    let reached = nested.clone();
    let inner = account.clone();
    let callback = Callback::new(move |scope, arguments| {
        let input = arguments.get(0).ok_or(Error::BackendUnavailable)?;
        let result = inner.run(scope, |cx| cx.value(&types::BOOLEAN, input))?;
        assert!(*result.get());
        reached.set(reached.get() + 1);
        Ok(Completion::Undefined)
    });
    process.execution().in_realm(&realm, |scope| {
        expose(scope, "nested", &callback)?;
        let input = scope.evaluate(&units("({toString(){nested(true);return 'ab'}})"))?;
        let output = account.run(scope, |cx| cx.value(&types::DOM_STRING, &input))?;
        assert_eq!(output.get().as_code_units(), [97, 98]);
        Ok(())
    })?;
    assert_eq!(nested.get(), 1);
    assert_eq!(parent.work(), 6); // outer value + inner value + 2 doubled units
    assert_eq!(parent.live(), 0);
    Ok(())
}

fn result_storage_outlives_the_factory_and_process() -> TestResult {
    let Fixture {
        mut process,
        realm,
        parent,
        account,
        factory,
        setup_drops,
    } = fixture(Limits::unlimited(), u64::MAX, 1)?;
    let output = process.execution().in_realm(&realm, |scope| {
        let input = scope.string(&units("abc"))?;
        account
            .run(scope, |cx| cx.value(&types::DOM_STRING, &input))
            .map_err(Error::from)
    })?;
    assert_eq!(*parent.payloads.borrow(), [6]);
    drop(account);
    assert!(
        factory.upgrade().is_none(),
        "converted storage retained the admission factory"
    );
    drop(process);
    assert_eq!(setup_drops.get(), 0);
    assert!(parent.live() >= 6);
    assert_eq!(output.get().as_code_units(), [97, 98, 99]);
    drop(output);
    assert_eq!(parent.live(), 0);
    assert_eq!(parent.releases.get(), 1);
    assert_eq!(setup_drops.get(), 1);
    Ok(())
}

fn release_cleanup_keeps_the_original_guest_throw() -> TestResult {
    let Fixture {
        mut process,
        realm,
        parent,
        account,
        ..
    } = fixture(Limits::unlimited(), u64::MAX, 1)?;
    let saved: RefCell<Option<Root>> = RefCell::new(None);
    let result = process.execution().in_realm(&realm, |scope| {
        let good = scope.string(&units("abc"))?;
        let bad = scope.evaluate(&units(
            "globalThis.primary={}; ({toString(){throw primary}})",
        ))?;
        account
            .run(scope, |cx| {
                let retained = cx.value(&types::DOM_STRING, &good)?;
                let failure = match cx.value(&types::DOM_STRING, &bad) {
                    Err(failure) => failure,
                    Ok(_) => panic!("guest conversion must throw"),
                };
                saved.replace(Some(failure.exception().unwrap().value().clone()));
                parent.release_fault.set(true);
                drop(retained);
                Err::<(), Failure>(failure)
            })
            .map_err(Error::from)
    });
    let Error::ConversionCleanup(failure) = result.unwrap_err() else {
        panic!("cleanup must be terminal")
    };
    assert_eq!(failure.cleanup().stage(), CleanupStage::StorageRelease);
    let Some(CleanupPrimary::Exception(exception)) = failure.primary() else {
        panic!("original guest throw lost")
    };
    assert!(Rc::ptr_eq(
        &exception.value().0,
        &saved.borrow().as_ref().unwrap().0
    ));
    let source = failure
        .host_error()
        .unwrap()
        .downcast_ref::<LedgerError>()
        .unwrap();
    assert!(
        matches!(source, surgeist_ledger::Error::Refused(refusal) if refusal.reason == surgeist_ledger::RefusalReason::Underflow)
    );
    assert_eq!(parent.live(), 0);
    Ok(())
}

fn observation_retains_a_typed_late_release_after_factory_and_process_retire() -> TestResult {
    let Fixture {
        mut process,
        realm,
        parent,
        account,
        factory,
        setup_drops,
    } = fixture(Limits::unlimited(), u64::MAX, 1)?;
    let observation = account.cleanup_observation();
    let output = process.execution().in_realm(&realm, |scope| {
        let input = scope.string(&units("abc"))?;
        account
            .run(scope, |cx| cx.value(&types::DOM_STRING, &input))
            .map_err(Error::from)
    })?;
    drop(account);
    assert!(factory.upgrade().is_none());
    drop(process);
    parent.release_fault.set(true);
    drop(output);
    assert_eq!(parent.live(), 0);
    assert_eq!(observation.status().failures(), Some(1));
    let source = observation
        .host_error()
        .unwrap()
        .downcast_ref::<LedgerError>()
        .unwrap();
    assert!(
        matches!(source, surgeist_ledger::Error::Refused(refusal) if refusal.reason == surgeist_ledger::RefusalReason::Underflow)
    );
    assert_eq!(setup_drops.get(), 0);
    drop(observation);
    assert_eq!(setup_drops.get(), 1);
    Ok(())
}

fn phase_cleanup_keeps_callback_panic_classification() -> TestResult {
    let Fixture {
        mut process,
        realm,
        parent,
        account,
        ..
    } = fixture(Limits::unlimited(), u64::MAX, 1)?;
    let operation = account.clone();
    let phase = parent.clone();
    let callback = Callback::new(move |scope, _| {
        operation.run::<()>(scope, |_cx| {
            phase.phase_fault.set(true);
            std::panic::panic_any(17_u32);
        })?;
        Ok(Completion::Undefined)
    });
    let result = process.execution().in_realm(&realm, |scope| {
        let function = scope.function(&callback)?;
        let receiver = scope.global()?;
        scope.call(&function, &receiver, &[]).map(|_| ())
    });
    let Error::ConversionCleanup(failure) = result.unwrap_err() else {
        panic!("cleanup must be terminal")
    };
    assert_eq!(failure.cleanup().stage(), CleanupStage::PhaseCompletion);
    assert!(matches!(
        failure.primary(),
        Some(CleanupPrimary::CallbackPanicked)
    ));
    assert!(failure.host_error().unwrap().is::<LedgerError>());
    Ok(())
}

pub(super) fn run() -> TestResult {
    local_refusal_precedes_each_guest_effect()?;
    dictionary_exact_and_one_short_parent_budgets()?;
    byte_string_overlap_obeys_payload_cap_before_parent_acquisition()?;
    same_account_guest_reentry_begins_after_host_admission_returns()?;
    result_storage_outlives_the_factory_and_process()?;
    observation_retains_a_typed_late_release_after_factory_and_process_retire()?;
    release_cleanup_keeps_the_original_guest_throw()?;
    phase_cleanup_keeps_callback_panic_classification()
}
