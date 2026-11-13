use std::cell::Cell;
use std::num::TryFromIntError;
use std::rc::Rc;

use surgeist_bindings_js::{
    Callback, Completion, ConstructorCallback, EngineKind, Error, FunctionMetadata, HostFailure,
    HostFailureSlot, IntegerKind, IntegerMode, IntegerValue, ObjectPrototype, Primitive, Process,
    PropertyKey, Realm, RecordFailure,
};

use surgeist_bindings_js::conversion::{Resource, Work};
use surgeist_ledger::{Change, Limits, Operation, ProfileMode, SharedOperation};

use super::units;

pub(super) fn observed_realm_is_the_callback_creation_realm(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let a = execution.create_realm()?;
    let b = execution.create_realm()?;
    let authority = execution.in_realm(&a, |scope| scope.realm_authority())?;
    let a_id = authority.identity();
    let b_id = execution.in_realm(&b, |scope| Ok(scope.realm_authority()?.identity()))?;
    assert_ne!(a_id, b_id);
    let called = Rc::new(Cell::new(false));
    let observed = called.clone();
    let callback = Callback::new(move |scope, _| {
        assert_eq!(scope.realm_authority()?.identity(), a_id);
        observed.set(true);
        Ok(Completion::Undefined)
    });
    let function = execution.in_realm(&a, |scope| {
        let function = scope.function(&callback)?;
        Ok(scope.retain(&function))
    })?;
    drop(a);
    execution.memory_pressure();
    execution.in_realm(&b, |scope| {
        assert_eq!(scope.realm_authority()?.identity(), b_id);
        let function = scope.import(&function)?;
        let undefined = scope.primitive(Primitive::Undefined)?;
        scope.call(&function, &undefined, &[])?;
        Ok(())
    })?;
    assert!(called.get());
    authority.check_owner_active()
}

pub(super) fn completing_with_an_occupied_failure_slot_refuses_before_entry(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let slot = HostFailureSlot::new(&scope.realm_authority()?, ())?;
        let completed =
            scope.complete_after_host_failure(&slot, std::io::Error::other("original"), |_| {
                Ok::<_, Error>(17)
            })?;
        assert_eq!(completed, 17);
        let entered = Cell::new(false);
        let repeated =
            scope.complete_after_host_failure(&slot, std::io::Error::other("replacement"), |_| {
                entered.set(true);
                Ok::<_, Error>(())
            });
        assert!(matches!(repeated, Err(Error::RegistryConflict)));
        assert!(!entered.get());
        scope.check_active()
    })
}

#[derive(Debug)]
struct NativeTicketError {
    requested: u64,
    range: TryFromIntError,
    drops: Rc<Cell<u32>>,
}

impl std::fmt::Display for NativeTicketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("native ticket out of range")
    }
}
impl std::error::Error for NativeTicketError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.range)
    }
}
impl Drop for NativeTicketError {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

fn native_ticket(requested: u64, drops: &Rc<Cell<u32>>) -> Result<u8, NativeTicketError> {
    u8::try_from(requested).map_err(|range| NativeTicketError {
        requested,
        range,
        drops: drops.clone(),
    })
}

fn original_failure(error: &Error) -> &HostFailure {
    match error {
        Error::HostFailure(failure) => failure,
        other => panic!("expected original native failure, got {other:?}"),
    }
}

fn assert_same_cause(left: &HostFailure, right: &HostFailure) {
    assert!(std::ptr::eq(
        left.cause::<NativeTicketError>().expect("left cause"),
        right.cause::<NativeTicketError>().expect("right cause"),
    ));
}

struct Debit {
    cause_drops: Rc<Cell<u32>>,
    observed: Rc<Cell<Option<u32>>>,
    ledger: SharedOperation<Work, Resource>,
    bytes: u64,
}
impl Drop for Debit {
    fn drop(&mut self) {
        self.observed.set(Some(self.cause_drops.get()));
        self.ledger
            .apply_batch(
                self.ledger.root(),
                &[Change::Release(Resource::BufferBytes, self.bytes)],
            )
            .expect("release the prepared slot debit");
    }
}

pub(super) fn refusal_preserves_actual_native_error_and_expiry() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let authority = process
        .execution()
        .in_realm(&realm, |scope| scope.realm_authority())?;
    let drops = Rc::new(Cell::new(0));
    let debit_release = Rc::new(Cell::new(None));
    let ledger = Operation::with_profile("native-ticket", Limits::unlimited(), 1, ProfileMode::Off)
        .map_err(Error::ConversionLedger)?
        .share();
    let bytes = u64::try_from(HostFailureSlot::<NativeTicketError, Debit>::storage_bytes())
        .expect("native slot extent");
    ledger
        .apply_batch(
            ledger.root(),
            &[Change::Acquire(Resource::BufferBytes, bytes)],
        )
        .map_err(Error::ConversionLedger)?;
    let slot = Rc::new(HostFailureSlot::new(
        &authority,
        Debit {
            cause_drops: drops.clone(),
            observed: debit_release.clone(),
            ledger: ledger.clone(),
            bytes,
        },
    )?);
    let count = Rc::new(Cell::new(0));
    let (prepared, released, effects) = (slot.clone(), drops.clone(), count.clone());
    let callback = Callback::new(move |scope, arguments| {
        let value = arguments.get(0).ok_or(Error::ExpectedObject)?;
        let IntegerValue::UnsignedLongLong(requested) =
            scope.integer(value, IntegerKind::UnsignedLongLong, IntegerMode::Wrap)?
        else {
            return Err(Error::BackendUnavailable);
        };
        scope.check_active()?;
        match native_ticket(requested, &released) {
            Ok(ticket) => {
                effects.set(effects.get() + 1);
                Ok(Completion::Return(
                    scope.primitive(Primitive::Number(f64::from(ticket)))?,
                ))
            }
            Err(cause) => {
                let failure = prepared.record(cause).expect("prepared unused error slot");
                // Even an adapter that handles the returned Rust error cannot
                // make the owner accept further SDK or native continuation.
                let error = scope.refuse_host(&failure);
                let still_first = scope.check_active().expect_err("terminal refusal");
                assert_same_cause(original_failure(&error), original_failure(&still_first));
                Ok(Completion::Undefined)
            }
        }
    });
    let error = process
        .execution()
        .in_realm(&realm, |scope| {
            let function = scope.function(&callback)?;
            let global = scope.global()?;
            scope.set(
                &global,
                surgeist_bindings_js::PropertyKey::String(&units("ticket")),
                &function,
            )?;
            let value = scope.evaluate(&units("ticket(255)"))?;
            assert_eq!(scope.to_utf16(&value)?, units("255"));
            scope.evaluate(&units("try { ticket(256); } catch (_) {} ticket(1);"))?;
            Ok(())
        })
        .expect_err("native refusal remains outside guest exception handling");
    let failure = original_failure(&error);
    assert_eq!(
        failure
            .cause::<NativeTicketError>()
            .expect("typed cause")
            .requested,
        256
    );
    assert_eq!(count.get(), 1);
    let later = process
        .execution()
        .in_realm(&realm, |_| Ok(()))
        .expect_err("terminal owner");
    assert_same_cause(failure, original_failure(&later));
    drop(later);
    drop(callback);
    drop(process);
    assert!(matches!(
        authority.check_owner_active(),
        Err(Error::ExpiredIdentity)
    ));
    assert_eq!(
        failure
            .cause::<NativeTicketError>()
            .expect("retained cause")
            .requested,
        256
    );
    assert_eq!(drops.get(), 0);
    let inactive_drops = Rc::new(Cell::new(0));
    let inactive = slot.record(native_ticket(257, &inactive_drops).expect_err("out of range"));
    let Err(RecordFailure::Inactive {
        error: Error::ExpiredIdentity,
        cause,
    }) = inactive
    else {
        panic!("expired slot must return attempted cause");
    };
    assert_eq!(cause.requested, 257);
    assert_eq!(inactive_drops.get(), 0);
    drop(cause);
    assert_eq!(inactive_drops.get(), 1);
    drop(slot);
    assert_eq!(debit_release.get(), None);
    assert_eq!(
        ledger
            .usage(ledger.root())
            .map_err(Error::ConversionLedger)?
            .live(Resource::BufferBytes),
        bytes
    );

    let mut next = Process::bootstrap(EngineKind::V8)?;
    let next_realm = next.execution().create_realm()?;
    let result = next.execution().in_realm(&next_realm, |scope| {
        assert_ne!(scope.realm_authority()?.identity(), authority.identity());
        Err::<(), _>(scope.refuse_host(failure))
    });
    assert!(matches!(result, Err(Error::ForeignOwner)));
    assert_eq!(
        failure
            .cause::<NativeTicketError>()
            .expect("retained cause")
            .requested,
        256
    );
    drop(error);
    assert_eq!(drops.get(), 1);
    assert_eq!(debit_release.get(), Some(1));
    assert_eq!(
        ledger
            .usage(ledger.root())
            .map_err(Error::ConversionLedger)?
            .live(Resource::BufferBytes),
        0
    );
    Ok(())
}

pub(super) fn prepared_slot_cannot_replace_an_earlier_failure() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let authority = process
        .execution()
        .in_realm(&realm, |scope| scope.realm_authority())?;
    let slot = HostFailureSlot::new(&authority, ())?;
    let drops = Rc::new(Cell::new(0));
    let prepared = slot
        .record(native_ticket(300, &drops).expect_err("out of range"))
        .expect("unused slot");
    let Err(RecordFailure::Occupied { first, cause }) =
        slot.record(native_ticket(301, &drops).expect_err("out of range"))
    else {
        panic!("slot may be written once");
    };
    assert_same_cause(&first, &prepared);
    assert_eq!(
        first
            .cause::<NativeTicketError>()
            .expect("first cause")
            .requested,
        300
    );
    assert_eq!(cause.requested, 301);
    assert_eq!(drops.get(), 0);
    drop(cause);
    assert_eq!(drops.get(), 1);
    let refuse = Callback::new(|_, _| Err(Error::ExpectedFunction));
    let result = process.execution().in_realm(&realm, |scope| {
        let callback = scope.function(&refuse)?;
        let receiver = scope.primitive(Primitive::Undefined)?;
        assert!(matches!(
            scope.call(&callback, &receiver, &[]),
            Err(Error::ExpectedFunction)
        ));
        assert!(matches!(
            scope.refuse_host(&prepared),
            Error::ExpectedFunction
        ));
        Ok(())
    });
    assert!(matches!(result, Err(Error::ExpectedFunction)));
    Ok(())
}

#[derive(Clone, Copy)]
enum DirectBoundary {
    Callable,
    Constructor,
    Host,
}

fn return_failure(
    process: &mut Process,
    realm: &Realm,
    boundary: DirectBoundary,
    error: Error,
) -> Result<(), Error> {
    process.execution().in_realm(realm, |scope| match boundary {
        DirectBoundary::Callable => {
            let error = Cell::new(Some(error));
            let callback =
                Callback::new(move |_, _| Err(error.take().ok_or(Error::BackendUnavailable)?));
            let function = scope.function(&callback)?;
            let receiver = scope.primitive(Primitive::Undefined)?;
            scope.call(&function, &receiver, &[]).map(|_| ())
        }
        DirectBoundary::Constructor => {
            let error = Cell::new(Some(error));
            let callback = ConstructorCallback::new(
                |scope| scope.object(ObjectPrototype::Default),
                move |_, _| Err(error.take().ok_or(Error::BackendUnavailable)?),
            );
            let function = scope.constructor(
                &callback,
                FunctionMetadata {
                    name: &units("Native"),
                    length: 0,
                },
            )?;
            let global = scope.global()?;
            scope.set(&global, PropertyKey::String(&units("Native")), &function)?;
            scope.evaluate(&units("new Native()")).map(|_| ())
        }
        DirectBoundary::Host => Err(error),
    })
}

fn old_failure<Storage: 'static>(storage: Storage) -> Result<Error, Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let authority = process
        .execution()
        .in_realm(&realm, |scope| scope.realm_authority())?;
    let slot = HostFailureSlot::new(&authority, storage)?;
    let cause = u8::try_from(256_u16).expect_err("native range refusal");
    let failure = slot.record(cause).expect("unused slot");
    Ok(process
        .execution()
        .in_realm(&realm, |scope| Err::<(), _>(scope.refuse_host(&failure)))
        .expect_err("original process refusal"))
}

pub(super) fn direct_failure_returns_validate_owner() -> Result<(), Error> {
    for boundary in [
        DirectBoundary::Callable,
        DirectBoundary::Constructor,
        DirectBoundary::Host,
    ] {
        let old = old_failure(())?;
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let realm = process.execution().create_realm()?;
        let result = return_failure(&mut process, &realm, boundary, old);
        assert!(matches!(result, Err(Error::ForeignOwner)), "{result:?}");
        let next = process.execution().in_realm(&realm, |_| Ok(()));
        match boundary {
            DirectBoundary::Callable | DirectBoundary::Constructor => {
                assert!(matches!(next, Err(Error::ForeignOwner)));
            }
            DirectBoundary::Host => next?,
        }
    }

    for boundary in [DirectBoundary::Callable, DirectBoundary::Constructor] {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let realm = process.execution().create_realm()?;
        let authority = process
            .execution()
            .in_realm(&realm, |scope| scope.realm_authority())?;
        let slot = HostFailureSlot::new(&authority, ())?;
        let cause = native_ticket(300, &Rc::new(Cell::new(0))).expect_err("native range refusal");
        let failure = slot.record(cause).expect("unused slot");
        let result = return_failure(
            &mut process,
            &realm,
            boundary,
            Error::HostFailure(failure.clone()),
        );
        assert_same_cause(
            &failure,
            original_failure(&result.expect_err("direct refusal")),
        );
        let next = process
            .execution()
            .in_realm(&realm, |_| Ok(()))
            .expect_err("sticky refusal");
        assert_same_cause(&failure, original_failure(&next));
    }
    Ok(())
}

pub(super) fn rejected_failure_drop_stays_in_callback_guard() -> Result<(), Error> {
    struct PanickingDebit(Rc<Cell<u32>>);
    impl Drop for PanickingDebit {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
            panic!("foreign cause debit drop");
        }
    }

    let dropped = Rc::new(Cell::new(0));
    let old = old_failure(PanickingDebit(dropped.clone()))?;
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let result = return_failure(&mut process, &realm, DirectBoundary::Callable, old);
    assert!(matches!(result, Err(Error::CallbackPanicked)));
    assert_eq!(dropped.get(), 1);
    Ok(())
}
