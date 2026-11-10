//! Capture destruction must unwind at an owned Rust boundary, never a native ABI.
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::error::Error as StdError;
use std::io::{self, Write};
use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};
use std::process::{Command, Output};
use std::rc::{Rc, Weak};

use surgeist_bindings_js::{
    Callback, Completion, ConstructorCallback, ConstructorCompletion, EngineKind, Error,
    FunctionMetadata, ObjectPrototype, Primitive, Process, PropertyKey, Realm,
};

type TestResult = Result<(), Box<dyn StdError>>;
const DROP_PANIC: &str = "capture destructor panic";

#[derive(Clone, Copy)]
enum Registration {
    Callable,
    Constructor,
}

#[derive(Clone, Copy)]
enum Boundary {
    GarbageCollection,
    GarbageCollectionWithFirstRefusal,
    OwnerDrop,
}

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    registration: Registration,
    boundary: Boundary,
}

const CASES: [Case; 6] = [
    Case {
        name: "callable-gc",
        registration: Registration::Callable,
        boundary: Boundary::GarbageCollection,
    },
    Case {
        name: "constructor-gc",
        registration: Registration::Constructor,
        boundary: Boundary::GarbageCollection,
    },
    Case {
        name: "callable-gc-first-refusal",
        registration: Registration::Callable,
        boundary: Boundary::GarbageCollectionWithFirstRefusal,
    },
    Case {
        name: "constructor-gc-first-refusal",
        registration: Registration::Constructor,
        boundary: Boundary::GarbageCollectionWithFirstRefusal,
    },
    Case {
        name: "callable-owner-drop",
        registration: Registration::Callable,
        boundary: Boundary::OwnerDrop,
    },
    Case {
        name: "constructor-owner-drop",
        registration: Registration::Constructor,
        boundary: Boundary::OwnerDrop,
    },
];

#[derive(Default)]
struct Observations {
    drops: Cell<usize>,
    body_calls: Cell<usize>,
}

struct PanickingCapture {
    observations: Rc<Observations>,
    armed: Cell<bool>,
}

impl PanickingCapture {
    fn body_called(&self) {
        self.observations
            .body_calls
            .set(self.observations.body_calls.get() + 1);
    }
}

impl Drop for PanickingCapture {
    fn drop(&mut self) {
        self.observations
            .drops
            .set(self.observations.drops.get() + 1);
        eprintln!(
            "capture Drop: drops={}, body_calls={}, armed={}",
            self.observations.drops.get(),
            self.observations.body_calls.get(),
            self.armed.get()
        );
        // A registration/setup error must not masquerade as the expected panic.
        if self.armed.get() {
            panic_any(DROP_PANIC);
        }
    }
}

fn register_capture(
    process: &mut Process,
    registration: Registration,
    observations: &Rc<Observations>,
) -> Result<(Realm, Weak<PanickingCapture>), Error> {
    let capture = Rc::new(PanickingCapture {
        observations: observations.clone(),
        armed: Cell::new(false),
    });
    let weak = Rc::downgrade(&capture);
    let retained = capture.clone();
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    match registration {
        Registration::Callable => {
            let callback = Callback::new(move |_, _| {
                retained.body_called();
                Ok(Completion::Undefined)
            });
            execution.in_realm(&realm, |scope| {
                let function = scope.function(&callback)?;
                let global = scope.global()?;
                scope.set(
                    &global,
                    PropertyKey::String(&"heldCapture".encode_utf16().collect::<Vec<_>>()),
                    &function,
                )
            })??;
        }
        Registration::Constructor => {
            let callback = ConstructorCallback::new(
                |scope| scope.object(ObjectPrototype::Default),
                move |scope, _| {
                    retained.body_called();
                    Ok(ConstructorCompletion::Return(
                        scope.object(ObjectPrototype::Default)?,
                    ))
                },
            );
            execution.in_realm(&realm, |scope| {
                let function = scope.constructor(
                    &callback,
                    FunctionMetadata {
                        name: &[],
                        length: 0,
                    },
                )?;
                let global = scope.global()?;
                scope.set(
                    &global,
                    PropertyKey::String(&"heldCapture".encode_utf16().collect::<Vec<_>>()),
                    &function,
                )
            })??;
        }
    }
    capture.armed.set(true);
    drop(capture);
    assert!(
        weak.upgrade().is_some(),
        "the guest function must retain its capture"
    );
    assert_eq!(observations.drops.get(), 0);
    assert_eq!(observations.body_calls.get(), 0);
    Ok((realm, weak))
}

fn assert_expected_panic(result: std::thread::Result<()>) {
    let payload = result.expect_err("capture Drop must reach the Rust unwind catcher");
    assert_eq!(payload.downcast_ref::<&str>(), Some(&DROP_PANIC));
}

fn assert_released(observations: &Observations, weak: &Weak<PanickingCapture>) {
    assert_eq!(
        observations.body_calls.get(),
        0,
        "cleanup must never call native behavior"
    );
    assert_eq!(
        observations.drops.get(),
        1,
        "captured resource is destroyed exactly once"
    );
    assert!(weak.upgrade().is_none());
}

fn verify_usable_owner(process: &mut Process, realm: &Realm) -> Result<(), Error> {
    process.execution().in_realm(realm, |scope| {
        let value = scope.evaluate(&"40 + 2".encode_utf16().collect::<Vec<_>>())?;
        assert_eq!(
            scope.to_utf16(&value)?,
            "42".encode_utf16().collect::<Vec<_>>()
        );
        Ok::<_, Error>(())
    })?
}

fn worker(case: Case) -> TestResult {
    // Test-only GC flags precede the production owner's platform bootstrap.
    v8::V8::set_flags_from_string("--expose-gc");
    let observations = Rc::new(Observations::default());
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let (realm, weak) = register_capture(&mut process, case.registration, &observations)?;
    match case.boundary {
        Boundary::GarbageCollection => {
            let mut execution = process.execution();
            assert_eq!(observations.body_calls.get(), 0);
            assert_eq!(observations.drops.get(), 0);
            // Include unrooting: an opportunistic collection during that call
            // may retire the same capture before the explicit pressure request.
            // Every allowed destruction point must still be a Rust boundary.
            let result = catch_unwind(AssertUnwindSafe(|| {
                execution
                    .in_realm(&realm, |scope| {
                        let global = scope.global()?;
                        let undefined = scope.primitive(Primitive::Undefined)?;
                        scope.set(
                            &global,
                            PropertyKey::String(&"heldCapture".encode_utf16().collect::<Vec<_>>()),
                            &undefined,
                        )
                    })
                    .expect("live realm")
                    .expect("global capture release");
                execution.memory_pressure();
            }));
            assert_expected_panic(result);
            assert_released(&observations, &weak);
            assert!(
                catch_unwind(AssertUnwindSafe(|| execution.memory_pressure())).is_ok(),
                "retired capture must not be destroyed again"
            );
            drop(execution);
            // Destructor panics are not callback-body failures: no terminal
            // CallbackPanicked/RegistryConflict refusal may have been latched.
            verify_usable_owner(&mut process, &realm)?;
            drop(process);
            assert_released(&observations, &weak);
        }
        Boundary::GarbageCollectionWithFirstRefusal => {
            let mut execution = process.execution();
            let fail = Callback::new(|_, _| Err(Error::IntegerOutOfRange));
            execution.in_realm(&realm, |scope| {
                let function = scope.function(&fail)?;
                let global = scope.global()?;
                scope.set(
                    &global,
                    PropertyKey::String(&"failHost".encode_utf16().collect::<Vec<_>>()),
                    &function,
                )
            })??;
            drop(fail);
            let result = catch_unwind(AssertUnwindSafe(|| {
                let result = execution.in_realm(&realm, |scope| {
                    scope
                        .evaluate(
                            &"heldCapture = undefined; gc(); try { failHost() } catch (_) {}"
                                .encode_utf16()
                                .collect::<Vec<_>>(),
                        )
                        .map(|_| ())
                });
                assert!(matches!(result, Err(Error::IntegerOutOfRange)));
                // Reaping may occur at the returning entry boundary or here;
                // neither location may destroy captures inside an SDK frame.
                execution.memory_pressure();
            }));
            assert_expected_panic(result);
            assert_released(&observations, &weak);
            assert!(matches!(
                execution.create_realm(),
                Err(Error::IntegerOutOfRange)
            ));
            assert!(matches!(
                execution.checkpoint(),
                Err(Error::IntegerOutOfRange)
            ));
            assert!(catch_unwind(AssertUnwindSafe(|| execution.memory_pressure())).is_ok());
            drop(execution);
            drop(process);
            assert_released(&observations, &weak);
        }
        Boundary::OwnerDrop => {
            assert_expected_panic(catch_unwind(AssertUnwindSafe(|| drop(process))));
            assert_released(&observations, &weak);
            // Ordinary Rust owner destruction must still release the native heap
            // and singleton even when a user's capture destructor unwinds.
            let mut next = Process::bootstrap(EngineKind::V8)?;
            let realm = next.execution().create_realm()?;
            verify_usable_owner(&mut next, &realm)?;
            drop(next);
            assert_released(&observations, &weak);
        }
    }
    println!(
        "{}: caught ordinary Rust Drop panic, one capture released, no body call, boundary state preserved",
        case.name
    );
    Ok(())
}

fn isolated_worker(case: Case) -> io::Result<Output> {
    Command::new(std::env::current_exe()?)
        .args(["--worker", case.name])
        .output()
}

fn main() -> TestResult {
    let mut arguments = std::env::args().skip(1);
    let mode = arguments.next();
    let selected = arguments.next();
    if mode.as_deref() == Some("--worker") {
        let case = CASES
            .iter()
            .copied()
            .find(|case| Some(case.name) == selected.as_deref())
            .ok_or_else(|| io::Error::other("unknown cleanup worker"))?;
        return worker(case);
    }
    if mode.as_deref() == Some("--case")
        && selected.as_deref() != Some("owner-drop")
        && !CASES
            .iter()
            .any(|case| Some(case.name) == selected.as_deref())
    {
        return Err(io::Error::other("unknown cleanup case").into());
    }
    let mut failures = Vec::new();
    for case in CASES {
        if mode.as_deref() == Some("--case")
            && selected.as_deref() != Some(case.name)
            && !(selected.as_deref() == Some("owner-drop")
                && matches!(case.boundary, Boundary::OwnerDrop))
        {
            continue;
        }
        let output = isolated_worker(case)?;
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
        eprintln!("{} worker status: {}", case.name, output.status);
        if !output.status.success() {
            failures.push(format!("{}: {}", case.name, output.status));
        }
    }
    if !failures.is_empty() {
        return Err(io::Error::other(failures.join("; ")).into());
    }
    Ok(())
}
