//! Real execution/account cleanup fault qualification. Corrupting a private
//! child balance below is deliberate fault injection through actual Ledger;
//! no fake allocator, platform brand or production failure hook is introduced.

use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};
use std::rc::Rc;

use surgeist_ledger::{Change, Limits, Operation, ProfileMode, SharedOperation, Tariffs};

use super::{
    CleanupPrimary, CleanupStage, ConversionAccount, Converted, Failure, Resource, ResourceTable,
    Work, WorkTable, types,
};
use crate::{Callback, Completion, EngineKind, Error, Process, Realm, Root};

type TestResult = Result<(), Box<dyn std::error::Error>>;
type Ledger = SharedOperation<Work, Resource>;

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn setup() -> Result<(Process, Realm, Ledger, ConversionAccount), Box<dyn std::error::Error>> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let ledger = Operation::with_profile(
        "host",
        Limits::unlimited(),
        4,
        ProfileMode::Summary { timing: false },
    )?
    .share();
    let account = process.execution().in_realm(&realm, |scope| {
        ConversionAccount::new(
            scope,
            ledger.clone(),
            ledger.root(),
            Limits::unlimited(),
            Tariffs::identity(),
        )
    })?;
    Ok((process, realm, ledger, account))
}

fn sabotage_release(account: &ConversionAccount) {
    // Remove the existing charge while the actual payload is still owned.
    // Its later ordinary guard release must report Underflow, never disappear.
    let live = account
        .0
        .ledger
        .usage(account.0.scope)
        .expect("owned child")
        .live(Resource::BufferBytes);
    assert!(live > 0);
    account
        .0
        .ledger
        .apply_batch(
            account.0.scope,
            &[Change::Release(Resource::BufferBytes, live)],
        )
        .expect("deliberate balance corruption");
}

fn assert_cleanup(account: &ConversionAccount) {
    let status = account.cleanup_status();
    assert_eq!(status.failures(), Some(1));
    let fault = status.first().expect("cleanup fault retained");
    assert_eq!(fault.stage(), CleanupStage::StorageRelease);
    assert!(
        matches!(fault.ledger_error(), Some(surgeist_ledger::Error::Refused(refusal))
        if refusal.reason == surgeist_ledger::RefusalReason::Underflow)
    );
}

fn save_original(
    scope: &mut crate::Scope<'_>,
    failure: &Failure,
    saved: &RefCell<Option<Root>>,
) -> Result<(), Error> {
    let exception = failure.exception().expect("original guest failure");
    assert_eq!(exception.conversion_rule(), None);
    let value = scope.import(exception.value())?;
    let expected = scope.evaluate(&units("primary"))?;
    assert!(scope.strict_equals(&value, &expected)?);
    saved.replace(Some(exception.value().clone()));
    Ok(())
}

fn assert_original(error: Error, saved: &RefCell<Option<Root>>) {
    let Error::ConversionCleanup(failure) = error else {
        panic!("expected compound failure: {error:?}")
    };
    let Some(CleanupPrimary::Exception(exception)) = failure.primary() else {
        panic!("guest primary must survive: {failure:?}")
    };
    let saved = saved.borrow();
    assert!(Rc::ptr_eq(
        &exception.value().0,
        &saved.as_ref().expect("saved original").0
    ));
    assert_eq!(exception.conversion_rule(), None);
}

fn selected_guest_and_late_cleanup(callback: bool, inside_run: bool) -> TestResult {
    let (mut process, realm, _ledger, account) = setup()?;
    let saved = Rc::new(RefCell::new(None));
    let operation = account.clone();
    let recorded = saved.clone();
    let body = move |scope: &mut crate::Scope<'_>| {
        let good = scope.string(&units("abc"))?;
        let bad = scope.evaluate(&units(
            "globalThis.primary = {toString(){throw 7}}; ({[Symbol.toPrimitive](){throw primary}})",
        ))?;
        if inside_run {
            let result: Result<(), Failure> = operation.run(scope, |cx| {
                let _retained = cx.value(&types::DOM_STRING, &good)?;
                let failure = match cx.value(&types::DOM_STRING, &bad) {
                    Err(failure) => failure,
                    Ok(_) => panic!("expected throw"),
                };
                save_original(cx.scope, &failure, &recorded).map_err(Failure)?;
                sabotage_release(&operation);
                Err(failure)
            });
            result?;
        } else {
            let retained = operation.run(scope, |cx| cx.value(&types::DOM_STRING, &good))?;
            let failure = match operation.run(scope, |cx| cx.value(&types::DOM_STRING, &bad)) {
                Err(failure) => failure,
                Ok(_) => panic!("expected throw"),
            };
            save_original(scope, &failure, &recorded)?;
            sabotage_release(&operation);
            drop(retained);
            return Err(Error::from(failure));
        }
        Ok(())
    };
    let result = if callback {
        let callback = Callback::new(move |scope, _| {
            body(scope)?;
            Ok(Completion::Undefined)
        });
        process.execution().in_realm(&realm, |scope| {
            let function = scope.function(&callback)?;
            let receiver = scope.global()?;
            scope.call(&function, &receiver, &[]).map(|_| ())
        })
    } else {
        // Use the same real conversion work at the public host exit. Public
        // Arguments are intentionally unconstructable; this invokes its body
        // through a private ordinary closure without inventing guest handles.
        let operation = account.clone();
        let recorded = saved.clone();
        process.execution().in_realm(&realm, |scope| {
            let good = scope.string(&units("abc"))?;
            let bad = scope.evaluate(&units(
                "globalThis.primary = {}; ({[Symbol.toPrimitive](){throw primary}})",
            ))?;
            if inside_run {
                operation
                    .run(scope, |cx| {
                        let _retained = cx.value(&types::DOM_STRING, &good)?;
                        let failure = match cx.value(&types::DOM_STRING, &bad) {
                            Err(failure) => failure,
                            Ok(_) => panic!("expected throw"),
                        };
                        save_original(cx.scope, &failure, &recorded).map_err(Failure)?;
                        sabotage_release(&operation);
                        Err::<(), _>(failure)
                    })
                    .map_err(Error::from)
            } else {
                let retained = operation.run(scope, |cx| cx.value(&types::DOM_STRING, &good))?;
                let failure = match operation.run(scope, |cx| cx.value(&types::DOM_STRING, &bad)) {
                    Err(failure) => failure,
                    Ok(_) => panic!("expected throw"),
                };
                save_original(scope, &failure, &recorded)?;
                sabotage_release(&operation);
                drop(retained);
                Err(Error::from(failure))
            }
        })
    };
    assert_original(result.expect_err("fatal cleanup"), &saved);
    assert_cleanup(&account);
    assert!(matches!(
        process.execution().create_realm(),
        Err(Error::ConversionCleanup(_))
    ));
    Ok(())
}

fn earlier_host_stays_first() -> TestResult {
    let (mut process, realm, _ledger, account) = setup()?;
    let result = process.execution().in_realm(&realm, |scope| {
        let good = scope.string(&units("abc"))?;
        let retained = account.run(scope, |cx| cx.value(&types::DOM_STRING, &good))?;
        account
            .0
            .ledger
            .set_limits(
                account.0.scope,
                Limits::new(
                    WorkTable {
                        Value: 1,
                        CodeUnit: u64::MAX,
                        EnumerationLookup: u64::MAX,
                        Argument: u64::MAX,
                        DictionaryMember: u64::MAX,
                    },
                    ResourceTable {
                        BufferBytes: u64::MAX,
                    },
                ),
            )
            .expect("change future admission");
        let error = match account.run(scope, |cx| cx.value(&types::BOOLEAN, &good)) {
            Err(error) => error,
            Ok(_) => panic!("expected host refusal"),
        };
        sabotage_release(&account);
        drop(retained);
        Err::<(), _>(Error::from(error))
    });
    assert!(matches!(result, Err(Error::ConversionLedger(_))));
    assert_cleanup(&account);
    assert!(matches!(
        process.execution().create_realm(),
        Err(Error::ConversionLedger(_))
    ));
    drop(account);
    // The selected host error stays immutable; its secondary cleanup remains
    // observable even if the caller no longer retains the account itself.
    assert_eq!(process.conversion_cleanup_status().failures(), Some(1));
    assert!(process.conversion_cleanup_status().first().is_some());
    Ok(())
}

fn cleanup_never_returns_success(callback: bool) -> TestResult {
    let (mut process, realm, _ledger, account) = setup()?;
    let operation = account.clone();
    let body = move |scope: &mut crate::Scope<'_>| {
        let good = scope.string(&units("abc"))?;
        let retained = operation.run(scope, |cx| cx.value(&types::DOM_STRING, &good))?;
        sabotage_release(&operation);
        drop(retained);
        Ok::<(), Error>(())
    };
    let result = if callback {
        let callback = Callback::new(move |scope, _| {
            body(scope)?;
            Ok(Completion::Undefined)
        });
        process.execution().in_realm(&realm, |scope| {
            let function = scope.function(&callback)?;
            let receiver = scope.global()?;
            scope.call(&function, &receiver, &[]).map(|_| ())
        })
    } else {
        process.execution().in_realm(&realm, body)
    };
    assert!(
        matches!(result, Err(Error::ConversionCleanup(ref failure)) if failure.primary().is_none())
    );
    assert_cleanup(&account);
    assert!(matches!(
        process.execution().create_realm(),
        Err(Error::ConversionCleanup(_))
    ));
    Ok(())
}

fn expired_release_fault_does_not_touch_the_next_owner() -> TestResult {
    let (retained, account) = {
        let (mut process, realm, _ledger, account) = setup()?;
        let retained = process.execution().in_realm(&realm, |scope| {
            let good = scope.string(&units("abc"))?;
            account
                .run(scope, |cx| cx.value(&types::DOM_STRING, &good))
                .map_err(Error::from)
        })?;
        (retained, account)
    };
    assert!(account.0.owner.upgrade().is_none());
    let mut next = Process::bootstrap(EngineKind::V8)?;
    let realm = next.execution().create_realm()?;
    sabotage_release(&account);
    drop(retained);
    assert_cleanup(&account);
    assert_eq!(next.conversion_cleanup_status().failures(), Some(0));
    next.execution().in_realm(&realm, |scope| {
        let value = scope.evaluate(&units("42"))?;
        assert_eq!(scope.to_utf16(&value)?, units("42"));
        Ok(())
    })?;
    Ok(())
}

fn selected_guest_survives_post_sdk_reaping() -> TestResult {
    let (mut process, realm, _ledger, account) = setup()?;
    let retained = process.execution().in_realm(&realm, |scope| {
        let good = scope.string(&units("abc"))?;
        account
            .run(scope, |cx| cx.value(&types::DOM_STRING, &good))
            .map_err(Error::from)
    })?;
    let calls = Rc::new(Cell::new(0));
    let bodies = calls.clone();
    let callback = Callback::new(move |_, _| {
        let _keep = retained.get();
        bodies.set(bodies.get() + 1);
        Ok(Completion::Undefined)
    });
    process.execution().in_realm(&realm, |scope| {
        let global = scope.global()?;
        let function = scope.function(&callback)?;
        scope.set(
            &global,
            crate::PropertyKey::String(&units("held")),
            &function,
        )
    })?;
    drop(callback);
    let saved = RefCell::new(None);
    let result = process.execution().in_realm(&realm, |scope| {
        let bad = scope.evaluate(&units(
            "globalThis.primary = {}; ({[Symbol.toPrimitive](){throw primary}})",
        ))?;
        let failure = match account.run(scope, |cx| cx.value(&types::DOM_STRING, &bad)) {
            Err(failure) => failure,
            Ok(_) => panic!("expected throw"),
        };
        save_original(scope, &failure, &saved)?;
        sabotage_release(&account);
        scope.evaluate(&units("held = undefined; gc()"))?;
        // SDK finalization only retires the lease. Captured storage must not
        // be destroyed until the returning entry reaps outside native frames.
        assert_eq!(account.cleanup_status().failures(), Some(0));
        Err::<(), _>(Error::from(failure))
    });
    assert_original(
        result.expect_err("reaping must not mask the selected guest"),
        &saved,
    );
    assert_cleanup(&account);
    assert_eq!(calls.get(), 0);
    Ok(())
}

fn selected_validation_survives_cleanup() -> TestResult {
    let (mut process, realm, _ledger, account) = setup()?;
    let saved = RefCell::new(None);
    let result = process.execution().in_realm(&realm, |scope| {
        let good = scope.string(&units("abc"))?;
        let bad = scope.string(&units("BAD"))?;
        let retained = account.run(scope, |cx| cx.value(&types::DOM_STRING, &good))?;
        let failure = match account.run(scope, |cx| {
            cx.value(super::declarations::shadow_root_mode().value_type(), &bad)
        }) {
            Err(failure) => failure,
            Ok(_) => panic!("expected enum TypeError"),
        };
        let exception = failure.exception().expect("real TypeError");
        assert_eq!(
            exception.conversion_rule(),
            Some(super::Rule::Enumeration("ShadowRootMode"))
        );
        let value = scope.import(exception.value())?;
        let validate = scope.evaluate(&units(
            "e => Object.getPrototypeOf(e) === TypeError.prototype",
        ))?;
        let receiver = scope.global()?;
        let valid = scope.call(&validate, &receiver, &[value])?;
        assert!(scope.boolean(&valid)?);
        saved.replace(Some(exception.value().clone()));
        sabotage_release(&account);
        drop(retained);
        Err::<(), _>(Error::from(failure))
    });
    let Error::ConversionCleanup(failure) = result.expect_err("fatal cleanup") else {
        panic!("expected aggregate")
    };
    let Some(CleanupPrimary::Exception(exception)) = failure.primary() else {
        panic!("validation primary")
    };
    assert_eq!(
        exception.conversion_rule(),
        Some(super::Rule::Enumeration("ShadowRootMode"))
    );
    assert!(Rc::ptr_eq(
        &exception.value().0,
        &saved.borrow().as_ref().expect("saved TypeError").0
    ));
    assert_cleanup(&account);
    Ok(())
}

fn selected_resolver_failure_survives_cleanup() -> TestResult {
    let (mut process, realm, _ledger, account) = setup()?;
    let saved = Rc::new(RefCell::new(None));
    let recorded = saved.clone();
    let operation = account.clone();
    let actions = Rc::new(Cell::new(0));
    let count = actions.clone();
    let callback = crate::ConstructorCallback::new(
        move |scope| {
            let good = scope.string(&units("abc"))?;
            let retained = operation.run(scope, |cx| cx.value(&types::DOM_STRING, &good))?;
            let bad = scope.evaluate(&units(
                "globalThis.primary = {}; ({[Symbol.toPrimitive](){throw primary}})",
            ))?;
            let failure = match operation.run(scope, |cx| cx.value(&types::DOM_STRING, &bad)) {
                Err(failure) => failure,
                Ok(_) => panic!("expected resolver conversion throw"),
            };
            save_original(scope, &failure, &recorded)?;
            sabotage_release(&operation);
            drop(retained);
            Err(Error::from(failure))
        },
        move |scope, arguments| {
            let _invocation = arguments.select_prototype(scope)?;
            scope.check_active()?;
            count.set(count.get() + 1);
            Ok(crate::ConstructorCompletion::Return(
                scope.object(crate::ObjectPrototype::Default)?,
            ))
        },
    );
    let result = process.execution().in_realm(&realm, |scope| {
        let constructor = scope.constructor(
            &callback,
            crate::FunctionMetadata {
                name: &units("C"),
                length: 0,
            },
        )?;
        let global = scope.global()?;
        scope.set(
            &global,
            crate::PropertyKey::String(&units("C")),
            &constructor,
        )?;
        scope
            .evaluate(&units(
                "function Other(){}; Other.prototype = null; Reflect.construct(C, [], Other)",
            ))
            .map(|_| ())
    });
    assert_original(result.expect_err("resolver cleanup is terminal"), &saved);
    assert_cleanup(&account);
    assert_eq!(actions.get(), 0);
    Ok(())
}

fn panic_keeps_its_primary(callback: bool, prior_host: bool) -> TestResult {
    const PANIC: &str = "conversion user panic";
    let (mut process, realm, ledger, account) = setup()?;
    if callback {
        let operation = account.clone();
        let callback = Callback::new(move |scope, _| {
            let good = scope.string(&units("abc"))?;
            operation.run::<()>(scope, |cx| {
                let _retained = cx.value(&types::DOM_STRING, &good)?;
                sabotage_release(&operation);
                if prior_host {
                    cx.scope.operations.refuse(Error::IntegerOutOfRange);
                }
                panic_any(PANIC)
            })?;
            Ok(Completion::Undefined)
        });
        let result = process.execution().in_realm(&realm, |scope| {
            let function = scope.function(&callback)?;
            let receiver = scope.global()?;
            scope.call(&function, &receiver, &[]).map(|_| ())
        });
        if prior_host {
            assert!(matches!(result, Err(Error::IntegerOutOfRange)));
        } else {
            assert!(
                matches!(result, Err(Error::ConversionCleanup(ref failure)) if matches!(failure.primary(), Some(CleanupPrimary::CallbackPanicked)))
            );
        }
    } else {
        let result = catch_unwind(AssertUnwindSafe(|| {
            process.execution().in_realm(&realm, |scope| {
                let good = scope.string(&units("abc"))?;
                account
                    .run::<()>(scope, |cx| {
                        let _retained = cx.value(&types::DOM_STRING, &good)?;
                        sabotage_release(&account);
                        if prior_host {
                            cx.scope.operations.refuse(Error::IntegerOutOfRange);
                        }
                        panic_any(PANIC)
                    })
                    .map_err(Error::from)
            })
        }));
        let payload = result.expect_err("ordinary Rust unwind");
        assert_eq!(payload.downcast_ref::<&str>(), Some(&PANIC));
        if prior_host {
            assert!(matches!(
                process.execution().create_realm(),
                Err(Error::IntegerOutOfRange)
            ));
        } else {
            assert!(
                matches!(process.execution().create_realm(), Err(Error::ConversionCleanup(ref failure)) if failure.primary().is_none())
            );
        }
    }
    assert_cleanup(&account);
    let report = ledger.snapshot()?.expect("phase profile");
    assert_eq!(
        report
            .scopes
            .iter()
            .find(|scope| scope.name == "js-conversion")
            .expect("conversion scope")
            .phases,
        1
    );
    Ok(())
}

fn value_drops_before_its_charge() -> TestResult {
    struct Value {
        ledger: Ledger,
        observed: Rc<Cell<u64>>,
    }
    impl Drop for Value {
        fn drop(&mut self) {
            self.observed.set(
                self.ledger
                    .usage(self.ledger.root())
                    .expect("live account")
                    .live(Resource::BufferBytes),
            );
        }
    }
    let (mut process, realm, ledger, account) = setup()?;
    let observed = Rc::new(Cell::new(0));
    let value = process.execution().in_realm(&realm, |_| {
        ledger
            .apply_batch(
                account.0.scope,
                &[Change::Acquire(Resource::BufferBytes, 6)],
            )
            .expect("fixture storage admission");
        Ok(Converted {
            value: Value {
                ledger: ledger.clone(),
                observed: observed.clone(),
            },
            storage: super::account::Storage {
                account: account.0.clone(),
                bytes: 6,
            },
        })
    })?;
    drop(value);
    assert_eq!(observed.get(), 6);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    Ok(())
}

pub(crate) fn run() -> TestResult {
    for callback in [false, true] {
        for inside_run in [false, true] {
            selected_guest_and_late_cleanup(callback, inside_run)?;
        }
        for prior in [false, true] {
            panic_keeps_its_primary(callback, prior)?;
        }
        cleanup_never_returns_success(callback)?;
    }
    earlier_host_stays_first()?;
    selected_guest_survives_post_sdk_reaping()?;
    selected_validation_survives_cleanup()?;
    selected_resolver_failure_survives_cleanup()?;
    value_drops_before_its_charge()?;
    expired_release_fault_does_not_touch_the_next_owner()?;
    Ok(())
}
