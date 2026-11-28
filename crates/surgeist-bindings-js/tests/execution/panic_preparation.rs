use super::*;
use std::sync::{Arc, Mutex};
use surgeist_bindings_js::preparation::{Admission, Controller, Request, Retention};

struct Payload {
    id: u8,
    dropped: Arc<Mutex<Vec<u8>>>,
    panic_on_drop: bool,
}

impl Drop for Payload {
    fn drop(&mut self) {
        self.dropped.lock().unwrap().push(self.id);
        if self.panic_on_drop {
            panic!("panic payload destructor");
        }
    }
}

struct Account {
    frames: RefCell<Vec<usize>>,
    limit: usize,
    live: Cell<usize>,
    releases: Cell<usize>,
    panic_frame: Option<usize>,
    dropped: Arc<Mutex<Vec<u8>>>,
}

impl Account {
    fn new(limit: usize, panic_frame: Option<usize>) -> Rc<Self> {
        Rc::new(Self {
            frames: RefCell::new(Vec::new()),
            limit,
            live: Cell::new(0),
            releases: Cell::new(0),
            panic_frame,
            dropped: Arc::new(Mutex::new(Vec::new())),
        })
    }

    fn admit(self: &Rc<Self>, request: Request) -> Result<Backing, Error> {
        let bytes = match request {
            Request::CallbackFrame { bytes } => {
                let ordinal = {
                    let mut frames = self.frames.borrow_mut();
                    frames.push(bytes);
                    frames.len()
                };
                if self.panic_frame == Some(ordinal) {
                    std::panic::panic_any(Payload {
                        id: 7,
                        dropped: self.dropped.clone(),
                        panic_on_drop: false,
                    });
                }
                let live = self
                    .live
                    .get()
                    .checked_add(bytes)
                    .ok_or(Error::Allocation)?;
                if live > self.limit {
                    return Err(Error::Allocation);
                }
                self.live.set(live);
                bytes
            }
            // This policy selects physical callback-frame custody. Other
            // preparations have their own focused admission tests.
            _ => 0,
        };
        Ok(Backing {
            account: self.clone(),
            bytes,
        })
    }
}

struct Backing {
    account: Rc<Account>,
    bytes: usize,
}

impl Drop for Backing {
    fn drop(&mut self) {
        if self.bytes != 0 {
            self.account.live.set(self.account.live.get() - self.bytes);
            self.account.releases.set(self.account.releases.get() + 1);
        }
    }
}

impl Retention for Backing {
    fn release(self: Box<Self>) -> Result<(), Error> {
        drop(self);
        Ok(())
    }
}

fn install_account(
    process: &mut Process,
    account: &Rc<Account>,
) -> Result<Controller<Backing>, Error> {
    let account = account.clone();
    Admission::new(move |request| account.admit(request), |_| Ok(())).install(process, ())
}

fn probe_limit(limit: usize, refusal: bool) -> Result<usize, Error> {
    let mut process = Process::bootstrap()?;
    let realm = process.execution().create_realm()?;
    let calls = Rc::new(Cell::new(0));
    let called = calls.clone();
    install(
        &mut process.execution(),
        &realm,
        "frame",
        &Callback::new(move |_, _| {
            called.set(called.get() + 1);
            Ok(Completion::Undefined)
        }),
    )?;
    let account = Account::new(limit, None);
    let controller = install_account(&mut process, &account)?;
    let result = process.execution().in_realm(&realm, |scope| {
        scope
            .evaluate(&units("try { frame() } catch (_) {}"))
            .map(drop)
    });
    if refusal {
        assert!(matches!(result, Err(Error::Allocation)));
        assert_eq!(calls.get(), 0);
        assert_eq!(account.releases.get(), 0);
    } else {
        result?;
        assert_eq!(calls.get(), 1);
        assert_eq!(account.releases.get(), 1);
    }
    assert_eq!(account.live.get(), 0);
    let bytes = match account.frames.borrow().as_slice() {
        [bytes] => *bytes,
        _ => return Err(Error::BackendUnavailable),
    };
    assert!(bytes > size_of::<Backing>());
    assert!(controller.observation().cleanup_error().is_none());
    Ok(bytes)
}

fn exact_frame_budget_admits_and_one_short_refuses_before_handler() -> Result<(), Error> {
    let bytes = probe_limit(usize::MAX, false)?;
    assert_eq!(probe_limit(bytes, false)?, bytes);
    assert_eq!(probe_limit(bytes - 1, true)?, bytes);
    Ok(())
}

fn completed_frames_wait_until_the_owned_boundary() -> Result<(), Error> {
    let mut process = Process::bootstrap()?;
    let realm = process.execution().create_realm()?;
    let account = Account::new(usize::MAX, None);
    install(
        &mut process.execution(),
        &realm,
        "inner",
        &Callback::new(|_, _| Ok(Completion::Undefined)),
    )?;
    let observed = account.clone();
    install(
        &mut process.execution(),
        &realm,
        "outer",
        &Callback::new(move |scope, _| {
            scope.evaluate(&units("inner()"))?;
            assert_eq!(observed.frames.borrow().len(), 2);
            assert_eq!(observed.releases.get(), 0);
            assert!(observed.live.get() > 0);
            Ok(Completion::Undefined)
        }),
    )?;
    let _controller = install_account(&mut process, &account)?;
    process
        .execution()
        .in_realm(&realm, |scope| scope.evaluate(&units("outer()")).map(drop))?;
    assert_eq!(account.releases.get(), 2);
    assert_eq!(account.live.get(), 0);
    Ok(())
}

fn nested_handler_panics_keep_both_payloads_until_teardown(
    panic_on_drop: bool,
) -> Result<(), Error> {
    let mut process = Process::bootstrap()?;
    let realm = process.execution().create_realm()?;
    let account = Account::new(usize::MAX, None);
    let dropped = account.dropped.clone();
    install(
        &mut process.execution(),
        &realm,
        "inner",
        &Callback::new(move |_, _| {
            std::panic::panic_any(Payload {
                id: 1,
                dropped: dropped.clone(),
                panic_on_drop: false,
            })
        }),
    )?;
    let dropped = account.dropped.clone();
    install(
        &mut process.execution(),
        &realm,
        "outer",
        &Callback::new(move |scope, _| {
            assert!(matches!(
                scope.evaluate(&units("inner()")),
                Err(Error::CallbackPanicked)
            ));
            assert!(dropped.lock().unwrap().is_empty());
            std::panic::panic_any(Payload {
                id: 2,
                dropped: dropped.clone(),
                panic_on_drop,
            })
        }),
    )?;
    let _controller = install_account(&mut process, &account)?;
    assert!(matches!(
        process.execution().in_realm(&realm, |scope| {
            scope.evaluate(&units("outer()")).map(drop)
        }),
        Err(Error::CallbackPanicked)
    ));
    assert_eq!(account.frames.borrow().len(), 2);
    assert_eq!(account.releases.get(), 0);
    assert!(account.live.get() > 0);
    assert!(account.dropped.lock().unwrap().is_empty());
    let teardown = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(process)));
    assert_eq!(teardown.is_err(), panic_on_drop);
    assert_eq!(account.live.get(), 0);
    assert_eq!(account.releases.get(), 2);
    let mut dropped = account.dropped.lock().unwrap();
    dropped.sort_unstable();
    assert_eq!(*dropped, [1, 2]);
    Ok(())
}

struct SetupReceipt {
    dropped: Arc<Mutex<Vec<u8>>>,
    released: Rc<Cell<bool>>,
}

impl Drop for SetupReceipt {
    fn drop(&mut self) {
        // Process-funded emergency custody is disposed before setup backing.
        assert_eq!(self.dropped.lock().unwrap().len(), 2);
        self.released.set(true);
    }
}

fn admission_panic_uses_emergency_and_outer_frame_keeps_its_own_payload() -> Result<(), Error> {
    let account = Account::new(usize::MAX, Some(2));
    let released = Rc::new(Cell::new(false));
    let mut process = Process::bootstrap_funded(|request| {
        assert!(matches!(request, Request::Process { bytes } if bytes > 0));
        Ok::<_, Error>(SetupReceipt {
            dropped: account.dropped.clone(),
            released: released.clone(),
        })
    })?;
    let realm = process.execution().create_realm()?;
    let inner_calls = Rc::new(Cell::new(0));
    let called = inner_calls.clone();
    install(
        &mut process.execution(),
        &realm,
        "inner",
        &Callback::new(move |_, _| {
            called.set(called.get() + 1);
            Ok(Completion::Undefined)
        }),
    )?;
    let dropped = account.dropped.clone();
    install(
        &mut process.execution(),
        &realm,
        "outer",
        &Callback::new(move |scope, _| {
            assert!(matches!(
                scope.evaluate(&units("inner()")),
                Err(Error::CallbackPanicked)
            ));
            assert!(dropped.lock().unwrap().is_empty());
            std::panic::panic_any(Payload {
                id: 8,
                dropped: dropped.clone(),
                panic_on_drop: false,
            })
        }),
    )?;
    let _controller = install_account(&mut process, &account)?;
    assert!(matches!(
        process.execution().in_realm(&realm, |scope| {
            scope.evaluate(&units("outer()")).map(drop)
        }),
        Err(Error::CallbackPanicked)
    ));
    assert_eq!(inner_calls.get(), 0);
    assert_eq!(account.frames.borrow().len(), 2);
    assert!(account.dropped.lock().unwrap().is_empty());
    assert!(matches!(
        process.execution().in_realm(&realm, |_| Ok(())),
        Err(Error::CallbackPanicked)
    ));
    assert_eq!(account.frames.borrow().len(), 2);
    drop(process);
    assert!(released.get());
    assert_eq!(account.live.get(), 0);
    assert_eq!(account.releases.get(), 1);
    let mut dropped = account.dropped.lock().unwrap();
    dropped.sort_unstable();
    assert_eq!(*dropped, [7, 8]);
    Ok(())
}

fn later_panic_preserves_the_original_host_failure() -> Result<(), Error> {
    let mut process = Process::bootstrap()?;
    let realm = process.execution().create_realm()?;
    let account = Account::new(usize::MAX, None);
    install(
        &mut process.execution(),
        &realm,
        "inner",
        &Callback::new(|_, _| Err(Error::IdentityExhausted)),
    )?;
    let dropped = account.dropped.clone();
    install(
        &mut process.execution(),
        &realm,
        "outer",
        &Callback::new(move |scope, _| {
            assert!(matches!(
                scope.evaluate(&units("inner()")),
                Err(Error::IdentityExhausted)
            ));
            std::panic::panic_any(Payload {
                id: 3,
                dropped: dropped.clone(),
                panic_on_drop: false,
            })
        }),
    )?;
    let _controller = install_account(&mut process, &account)?;
    assert!(matches!(
        process.execution().in_realm(&realm, |scope| {
            scope.evaluate(&units("outer()")).map(drop)
        }),
        Err(Error::IdentityExhausted)
    ));
    assert!(account.dropped.lock().unwrap().is_empty());
    drop(process);
    assert_eq!(*account.dropped.lock().unwrap(), [3]);
    assert_eq!(account.live.get(), 0);
    Ok(())
}

pub(super) fn host_microtask_panic_payload_waits_for_retirement() -> Result<(), Error> {
    let mut process = Process::bootstrap()?;
    let realm = process.execution().create_realm()?;
    let dropped = Arc::new(Mutex::new(Vec::new()));
    let observed = dropped.clone();
    process.execution().in_realm(&realm, |scope| {
        scope.enqueue_host_microtask(move |_| {
            std::panic::panic_any(Payload {
                id: 9,
                dropped: observed.clone(),
                panic_on_drop: true,
            })
        })
    })?;
    assert!(matches!(
        process.execution().checkpoint_microtasks(),
        Err(Error::CallbackPanicked)
    ));
    assert!(dropped.lock().unwrap().is_empty());
    assert!(matches!(
        process.execution().in_realm(&realm, |_| Ok(())),
        Err(Error::CallbackPanicked)
    ));
    let retired = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(process)));
    assert!(retired.is_err());
    assert_eq!(*dropped.lock().unwrap(), [9]);
    Ok(())
}

enum HostCleanup {
    Release,
    Unwind(&'static str),
    Refuse,
}

impl Retention for HostCleanup {
    fn release(self: Box<Self>) -> Result<(), Error> {
        match *self {
            Self::Release => Ok(()),
            Self::Unwind(message) => std::panic::panic_any(message),
            Self::Refuse => Err(Error::IdentityExhausted),
        }
    }
}

pub(super) fn host_microtask_cleanup_unwind_refuses_further_dispatch() -> Result<(), Error> {
    use surgeist_bindings_js::MicrotaskProgress;

    for progress in [MicrotaskProgress::Complete, MicrotaskProgress::Suspended] {
        let mut process = Process::bootstrap()?;
        let realm = process.execution().create_realm()?;
        let unwind = Cell::new(true);
        let account = Admission::new(
            move |request| {
                Ok(match request {
                    Request::CallbackFrame { .. } if unwind.replace(false) => {
                        HostCleanup::Unwind("returned host microtask frame release")
                    }
                    Request::HostMicrotask { .. } if progress == MicrotaskProgress::Suspended => {
                        HostCleanup::Refuse
                    }
                    _ => HostCleanup::Release,
                })
            },
            |_| Ok(()),
        )
        .install(&mut process, ())?;
        process.enable_instruction_counting()?;
        let native_runs = Rc::new(Cell::new(0));
        let runs = native_runs.clone();
        process.execution().in_realm(&realm, |scope| {
            scope.enqueue_host_microtask(move |_| {
                runs.set(runs.get() + 1);
                Ok(progress)
            })?;
            scope.evaluate(&units(
                "Promise.resolve().then(() => globalThis.suffixEntered = true)",
            ))?;
            Ok(())
        })?;
        let before = process.instruction_count();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            process.execution().checkpoint_microtasks()
        }));
        let payload = result.expect_err("Rust frame cleanup must unwind normally");
        assert_eq!(
            payload.downcast_ref::<&str>().copied(),
            Some("returned host microtask frame release")
        );
        assert!(matches!(
            process.execution().checkpoint_microtasks(),
            Err(Error::MicrotaskCheckpointUnwound)
        ));
        assert!(matches!(
            process.execution().checkpoint(),
            Err(Error::MicrotaskCheckpointUnwound)
        ));
        assert_eq!(process.instruction_count(), before, "the guest suffix ran");
        assert_eq!(native_runs.get(), 1);
        let observation = account.observation();
        assert!(observation.cleanup_error().is_none());
        drop(process);
        if progress == MicrotaskProgress::Suspended {
            assert!(matches!(
                observation.cleanup_error(),
                Some(Error::IdentityExhausted)
            ));
        }
    }
    Ok(())
}

pub(super) fn panic_preparation_contracts() -> Result<(), Error> {
    exact_frame_budget_admits_and_one_short_refuses_before_handler()?;
    completed_frames_wait_until_the_owned_boundary()?;
    nested_handler_panics_keep_both_payloads_until_teardown(false)?;
    nested_handler_panics_keep_both_payloads_until_teardown(true)?;
    admission_panic_uses_emergency_and_outer_frame_keeps_its_own_payload()?;
    later_panic_preserves_the_original_host_failure()?;
    host_microtask_panic_payload_waits_for_retirement()?;
    host_microtask_cleanup_unwind_refuses_further_dispatch()
}
