use std::cell::RefCell;

use surgeist_ledger::{
    Change, Operation, ProfileMode, RefusalReason, SharedOperation, Tariff, Tariffs,
};

use super::*;
use crate::conversion::{AdmissionError, CleanupPrimary, LedgerError};

#[derive(Debug, PartialEq, Eq)]
enum HostFault {
    Original(u64),
    Ledger(LedgerError),
    PhaseOwner,
}

impl fmt::Display for HostFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "original enclosing-operation fault: {self:?}")
    }
}

impl std::error::Error for HostFault {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ledger(error) => Some(error),
            Self::Original(_) | Self::PhaseOwner => None,
        }
    }
}

/// A real enclosing Ledger. Its child retains conversion units and tariffs
/// project every work dimension into the root's Value aggregate. No engine or
/// native effects are simulated by this admission/custody fixture.
struct Enclosing {
    ledger: SharedOperation<Work, Resource>,
    child: surgeist_ledger::Scope,
    work_calls: Cell<usize>,
    buffer_calls: Cell<usize>,
    release_calls: Cell<usize>,
    phase_starts: Cell<usize>,
    phase_ends: Cell<usize>,
    last_buffer: Cell<Option<BufferRequest>>,
    during_work: RefCell<Option<Box<dyn FnOnce()>>>,
    during_buffer: RefCell<Option<Box<dyn FnOnce()>>>,
    during_release: RefCell<Option<Box<dyn FnOnce()>>>,
    reject_buffer: Cell<bool>,
    release_error: RefCell<Option<HostFault>>,
    phase_error: RefCell<Option<HostFault>>,
}

impl Enclosing {
    fn new(work: u64, bytes: u64, scale: u64, mode: ProfileMode) -> Rc<Self> {
        let ledger = Operation::with_reserved_profile(
            "original-host",
            Limits::from_fn(
                |dimension| if dimension == Work::Value { work } else { 0 },
                |_| bytes,
            ),
            2,
            mode,
        )
        .unwrap()
        .share();
        let child = ledger
            .child(
                ledger.root(),
                "conversion",
                Limits::unlimited(),
                Tariffs::from_fn(
                    |_| Tariff::new(Work::Value, scale),
                    |resource| Tariff::new(resource, 1),
                ),
            )
            .unwrap();
        Rc::new(Self {
            ledger,
            child,
            work_calls: Cell::new(0),
            buffer_calls: Cell::new(0),
            release_calls: Cell::new(0),
            phase_starts: Cell::new(0),
            phase_ends: Cell::new(0),
            last_buffer: Cell::new(None),
            during_work: RefCell::new(None),
            during_buffer: RefCell::new(None),
            during_release: RefCell::new(None),
            reject_buffer: Cell::new(false),
            release_error: RefCell::new(None),
            phase_error: RefCell::new(None),
        })
    }

    fn work(&self) -> u64 {
        self.ledger
            .usage(self.ledger.root())
            .unwrap()
            .work(Work::Value)
    }

    fn live(&self) -> u64 {
        self.ledger
            .usage(self.ledger.root())
            .unwrap()
            .live(Resource::BufferBytes)
    }
}

struct Adapter {
    enclosing: Rc<Enclosing>,
    retired: Rc<Cell<usize>>,
}

impl Drop for Adapter {
    fn drop(&mut self) {
        self.retired.set(self.retired.get() + 1);
    }
}

struct Buffer {
    enclosing: Rc<Enclosing>,
    bytes: u64,
    error: Option<HostFault>,
}

fn run_action(action: &RefCell<Option<Box<dyn FnOnce()>>>) {
    let action = action.borrow_mut().take();
    if let Some(action) = action {
        action();
    }
}

impl BufferRetention for Buffer {
    type Error = HostFault;

    fn release(self) -> Result<(), Self::Error> {
        self.enclosing
            .release_calls
            .set(self.enclosing.release_calls.get() + 1);
        self.enclosing
            .ledger
            .apply(
                self.enclosing.child,
                Change::Release(Resource::BufferBytes, self.bytes),
            )
            .map_err(HostFault::Ledger)?;
        run_action(&self.enclosing.during_release);
        match self.error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl Host for Adapter {
    type CleanupError = HostFault;
    type Buffer = Buffer;

    fn charge(&self, work: Work, units: u64) -> Result<(), Error> {
        self.enclosing
            .work_calls
            .set(self.enclosing.work_calls.get() + 1);
        run_action(&self.enclosing.during_work);
        self.enclosing
            .ledger
            .apply(self.enclosing.child, Change::Work(work, units))
            .map_err(|_| Error::TooManyArguments)
    }

    fn retain_buffer(&self, request: BufferRequest) -> Result<Self::Buffer, Error> {
        self.enclosing
            .buffer_calls
            .set(self.enclosing.buffer_calls.get() + 1);
        self.enclosing.last_buffer.set(Some(request));
        run_action(&self.enclosing.during_buffer);
        if self.enclosing.reject_buffer.get() {
            return Err(Error::Allocation);
        }
        let bytes = u64::try_from(request.backing_bytes()).unwrap();
        self.enclosing
            .ledger
            .apply(
                self.enclosing.child,
                Change::Acquire(Resource::BufferBytes, bytes),
            )
            .map_err(|_| Error::Allocation)?;
        Ok(Buffer {
            enclosing: self.enclosing.clone(),
            bytes,
            error: self.enclosing.release_error.borrow_mut().take(),
        })
    }

    fn begin_phase(&self) -> Result<Option<PhaseObservation>, Error> {
        self.enclosing
            .phase_starts
            .set(self.enclosing.phase_starts.get() + 1);
        self.enclosing
            .ledger
            .begin_phase(self.enclosing.child)
            .map(Some)
            .map_err(Error::ConversionLedger)
    }

    fn end_phase(&self, marker: PhaseObservation) -> Result<(), Self::CleanupError> {
        self.enclosing
            .phase_ends
            .set(self.enclosing.phase_ends.get() + 1);
        self.enclosing
            .ledger
            .end_phase(marker)
            .map_err(|_| HostFault::PhaseOwner)?;
        self.enclosing
            .ledger
            .usage(self.enclosing.child)
            .map_err(HostFault::Ledger)?;
        match self.enclosing.phase_error.borrow_mut().take() {
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

struct Fixture {
    owner: Rc<OwnerState>,
    target: Rc<Target<Adapter>>,
    enclosing: Rc<Enclosing>,
    factory_retired: Rc<Cell<usize>>,
    setup_retired: Rc<Cell<usize>>,
}

fn fixture(
    local_work: u64,
    local_bytes: u64,
    host_work: u64,
    host_bytes: u64,
    scale: u64,
    mode: ProfileMode,
) -> Fixture {
    let owner = Rc::new(OwnerState::new(7));
    let enclosing = Enclosing::new(host_work, host_bytes, scale, mode);
    let factory_retired = Rc::new(Cell::new(0));
    let setup_retired = Rc::new(Cell::new(0));
    // Supply already-owned metadata custody before constructing any participant
    // records. This fixture exercises the private owning boundary without SDK.
    let backing: Rc<dyn Any> = Rc::new(Setup(setup_retired.clone()));
    let record = Rc::new(Record {
        owner: owner.id(),
        first: OnceCell::new(),
        _backing: backing.clone(),
    });
    let erased: Rc<dyn CleanupRecord> = record.clone();
    let state = Rc::new(StorageState {
        local: LocalState::new(Limits::from_fn(|_| local_work, |_| local_bytes)),
        cleanup: Cell::new(CleanupStatus::default()),
        owner: Rc::downgrade(&owner),
        record: Rc::downgrade(&erased),
        _backing: backing,
    });
    let custody: Rc<dyn CleanupCustody> = state.clone();
    owner.register_conversion_cleanup(Rc::downgrade(&custody), erased);
    let target = Rc::new(Target {
        host: Adapter {
            enclosing: enclosing.clone(),
            retired: factory_retired.clone(),
        },
        state,
        record,
    });
    Fixture {
        owner,
        target,
        enclosing,
        factory_retired,
        setup_retired,
    }
}

fn unlimited(local_work: u64, local_bytes: u64) -> Fixture {
    fixture(
        local_work,
        local_bytes,
        u64::MAX,
        u64::MAX,
        1,
        ProfileMode::Off,
    )
}

#[test]
fn local_work_refusal_precedes_host_refusal_and_host_acceptance_commits_once() {
    let f = fixture(1, 0, 0, 0, 2, ProfileMode::Off);
    let refusal = match f.target.work(Work::Value, 2) {
        Err(Error::ConversionAdmission(AdmissionError::Refused(refusal))) => refusal,
        other => panic!("unexpected local refusal: {other:?}"),
    };
    assert_eq!(refusal.change(), Change::Work(Work::Value, 2));
    assert_eq!(refusal.current(), 0);
    assert_eq!(refusal.reason(), RefusalReason::Limit { limit: 1 });
    assert_eq!(f.enclosing.work_calls.get(), 0);
    assert!(matches!(
        f.target.work(Work::Value, 1),
        Err(Error::TooManyArguments)
    ));
    assert_eq!(f.target.state.local.usage().work(Work::Value), 0);
    assert_eq!(f.enclosing.work(), 0);
    f.enclosing
        .ledger
        .set_limits(f.enclosing.ledger.root(), Limits::from_fn(|_| 2, |_| 0))
        .unwrap();
    f.target.work(Work::Value, 1).unwrap();
    assert_eq!(f.target.state.local.usage().work(Work::Value), 1);
    assert_eq!(f.enclosing.work(), 2);
}

#[test]
fn host_work_refusal_preserves_prior_dimension_usage_and_tariffed_prefix() {
    let f = fixture(10, 0, 4, 0, 2, ProfileMode::Off);
    f.target.work(Work::Value, 2).unwrap();
    assert!(matches!(
        f.target.work(Work::CodeUnit, 1),
        Err(Error::TooManyArguments)
    ));
    assert_eq!(f.target.state.local.usage().work(Work::Value), 2);
    assert_eq!(f.target.state.local.usage().work(Work::CodeUnit), 0);
    assert_eq!(f.enclosing.work(), 4);
    f.enclosing
        .ledger
        .set_limits(f.enclosing.ledger.root(), Limits::from_fn(|_| 6, |_| 0))
        .unwrap();
    f.target.work(Work::CodeUnit, 1).unwrap();
    assert_eq!(f.target.state.local.usage().work(Work::CodeUnit), 1);
    assert_eq!(f.enclosing.work(), 6);
}

#[test]
fn buffer_caps_are_exact_and_host_backing_includes_its_concrete_release_record() {
    let bytes = 3 + size_of::<Retained<Buffer>>();
    for host_bytes in [bytes - 1, bytes] {
        let f = fixture(
            0,
            3,
            0,
            u64::try_from(host_bytes).unwrap(),
            1,
            ProfileMode::Off,
        );
        let result = f.target.reserve(3);
        let request = f.enclosing.last_buffer.get().unwrap();
        assert_eq!(request.payload_bytes(), 3);
        assert_eq!(request.backing_bytes(), bytes);
        if host_bytes == bytes {
            let buffer = result.unwrap();
            assert_eq!(f.target.state.local.usage().live(), 3);
            assert_eq!(f.enclosing.live(), u64::try_from(bytes).unwrap());
            drop(buffer);
            assert_eq!(f.enclosing.release_calls.get(), 1);
            assert_eq!(f.target.state.local.usage().peak(), 3);
        } else {
            assert!(matches!(result, Err(Error::Allocation)));
            assert_eq!(f.target.state.local.usage().peak(), 0);
            assert_eq!(f.enclosing.release_calls.get(), 0);
        }
        assert_eq!(f.target.state.local.usage().live(), 0);
        assert_eq!(f.enclosing.live(), 0);
    }
    let f = fixture(0, 2, 0, 0, 1, ProfileMode::Off);
    assert!(matches!(
        f.target.reserve(3),
        Err(Error::ConversionAdmission(AdmissionError::Refused(_)))
    ));
    assert_eq!(f.enclosing.buffer_calls.get(), 0);
    drop(f.target.reserve(0).unwrap());
    assert_eq!(f.enclosing.buffer_calls.get(), 0);
    assert_eq!(f.enclosing.release_calls.get(), 0);
}

#[test]
fn release_during_host_admission_preserves_actual_custody_and_conditional_peak() {
    for refused in [false, true] {
        let f = unlimited(0, 110);
        let earlier = f.target.reserve(40).unwrap();
        let released = Rc::new(RefCell::new(Some(f.target.reserve(20).unwrap())));
        let remaining = f.target.reserve(40).unwrap();
        drop(earlier);
        assert_eq!(f.target.state.local.usage().live(), 60);
        assert_eq!(f.target.state.local.usage().peak(), 100);
        let during = released.clone();
        *f.enclosing.during_buffer.borrow_mut() =
            Some(Box::new(move || drop(during.borrow_mut().take())));
        f.enclosing.reject_buffer.set(refused);
        let result = f.target.reserve(50);
        let accepted = if refused {
            assert!(matches!(result, Err(Error::Allocation)));
            assert_eq!(f.target.state.local.usage().live(), 40);
            assert_eq!(f.target.state.local.usage().peak(), 100);
            None
        } else {
            assert_eq!(f.target.state.local.usage().live(), 90);
            assert_eq!(f.target.state.local.usage().peak(), 110);
            Some(result.unwrap())
        };
        drop((remaining, accepted));
        assert_eq!(f.target.state.local.usage().live(), 0);
        assert_eq!(f.enclosing.live(), 0);
    }
}

#[test]
fn host_callbacks_cannot_reenter_work_buffer_or_phase_admission() {
    let f = unlimited(10, 10);
    let selected = Rc::downgrade(&f.target);
    let nested = Rc::new(RefCell::new(None));
    let observed = nested.clone();
    *f.enclosing.during_work.borrow_mut() = Some(Box::new(move || {
        let selected = selected.upgrade().unwrap();
        assert!(matches!(
            selected.begin_phase(),
            Err(Error::ConversionAdmission(AdmissionError::Reentrant))
        ));
        *observed.borrow_mut() = Some(selected.work(Work::CodeUnit, 1));
    }));
    f.target.work(Work::Value, 1).unwrap();
    assert!(matches!(
        nested.borrow().as_ref(),
        Some(Err(Error::ConversionAdmission(AdmissionError::Reentrant)))
    ));
    assert_eq!(f.enclosing.work_calls.get(), 1);
    assert_eq!(f.enclosing.phase_starts.get(), 0);
    assert_eq!(f.enclosing.work(), 1);
    assert_eq!(f.target.state.local.usage().work(Work::CodeUnit), 0);
    let selected = Rc::downgrade(&f.target);
    *f.enclosing.during_buffer.borrow_mut() = Some(Box::new(move || {
        assert!(matches!(
            selected.upgrade().unwrap().reserve(1),
            Err(Error::ConversionAdmission(AdmissionError::Reentrant))
        ));
    }));
    let buffer = f.target.reserve(3).unwrap();
    assert_eq!(f.enclosing.buffer_calls.get(), 1);
    let selected = Rc::downgrade(&f.target);
    let observed = nested.clone();
    *f.enclosing.during_release.borrow_mut() = Some(Box::new(move || {
        *observed.borrow_mut() = Some(selected.upgrade().unwrap().work(Work::CodeUnit, 1));
    }));
    drop(buffer);
    assert!(matches!(
        nested.borrow().as_ref(),
        Some(Err(Error::ConversionAdmission(AdmissionError::Reentrant)))
    ));
    assert_eq!(f.enclosing.work_calls.get(), 1);
    assert_eq!(f.target.state.local.usage().live(), 0);
    assert_eq!(f.enclosing.live(), 0);
    f.target.work(Work::CodeUnit, 1).unwrap();
    assert_eq!(f.enclosing.work(), 2);
}

#[test]
fn accepted_buffer_is_released_when_an_earlier_host_refusal_blocks_materialization() {
    let f = unlimited(0, 3);
    let owner = Rc::downgrade(&f.owner);
    *f.enclosing.during_buffer.borrow_mut() = Some(Box::new(move || {
        owner.upgrade().unwrap().fail(Error::IntegerOutOfRange);
    }));
    assert!(matches!(f.target.reserve(3), Err(Error::IntegerOutOfRange)));
    assert_eq!(f.enclosing.buffer_calls.get(), 1);
    assert_eq!(f.enclosing.release_calls.get(), 1);
    assert_eq!(f.enclosing.live(), 0);
    assert_eq!(f.target.state.local.usage().live(), 0);
    assert_eq!(f.target.state.local.usage().peak(), 3);
}

#[test]
fn late_buffer_release_keeps_typed_cleanup_without_retaining_its_factory() {
    let Fixture {
        owner,
        target,
        enclosing,
        factory_retired,
        setup_retired,
    } = unlimited(0, 3);
    *enclosing.release_error.borrow_mut() = Some(HostFault::Original(41));
    let buffer = target.reserve(3).unwrap();
    let observation = target.observation();
    let factory = Rc::downgrade(&target);
    drop(target);
    assert!(factory.upgrade().is_none());
    assert_eq!(factory_retired.get(), 1);
    drop(owner);
    assert_eq!(setup_retired.get(), 0);
    drop(buffer);
    assert_eq!(enclosing.live(), 0);
    assert_eq!(enclosing.release_calls.get(), 1);
    assert_eq!(observation.status().failures(), Some(1));
    assert_eq!(
        observation.status().first(),
        Some(CleanupFault::HostStorageRelease)
    );
    assert_eq!(
        observation
            .host_error()
            .unwrap()
            .downcast_ref::<HostFault>(),
        Some(&HostFault::Original(41))
    );
    assert_eq!(setup_retired.get(), 0);
    drop(observation);
    assert_eq!(setup_retired.get(), 1);
}

#[test]
fn owner_registration_preserves_a_release_error_after_the_factory_retires() {
    let Fixture {
        owner,
        target,
        enclosing,
        factory_retired,
        setup_retired,
    } = unlimited(0, 3);
    *enclosing.release_error.borrow_mut() = Some(HostFault::Original(42));
    let buffer = target.reserve(3).unwrap();
    let record = Rc::downgrade(&target.record);
    drop(target);
    assert_eq!(factory_retired.get(), 1);
    assert!(record.upgrade().is_some());
    drop(buffer);
    assert_eq!(enclosing.live(), 0);
    let reported = match owner.available() {
        Err(Error::ConversionCleanup(failure)) => failure,
        other => panic!("unexpected release result: {other:?}"),
    };
    assert_eq!(
        reported.host_error().unwrap().downcast_ref::<HostFault>(),
        Some(&HostFault::Original(42))
    );
    drop(reported);
    drop(owner);
    assert!(record.upgrade().is_none());
    assert_eq!(setup_retired.get(), 1);
}

#[test]
fn release_finishes_after_the_factory_owner_and_observers_are_gone() {
    let Fixture {
        owner,
        target,
        enclosing,
        factory_retired,
        setup_retired,
    } = unlimited(0, 3);
    *enclosing.release_error.borrow_mut() = Some(HostFault::Original(43));
    let buffer = target.reserve(3).unwrap();
    drop(target);
    drop(owner);
    assert_eq!(factory_retired.get(), 1);
    assert_eq!(setup_retired.get(), 0);
    drop(buffer);
    assert_eq!(enclosing.live(), 0);
    assert_eq!(enclosing.release_calls.get(), 1);
    assert_eq!(setup_retired.get(), 1);
}

#[test]
fn phase_error_remains_first_when_release_also_fails_and_callback_panic_is_selected() {
    let f = unlimited(0, 3);
    *f.enclosing.release_error.borrow_mut() = Some(HostFault::Original(2));
    let buffer = f.target.reserve(3).unwrap();
    let observation = f.target.observation();
    let boundary = crate::owner::Boundary::enter(f.owner.clone()).unwrap();
    let marker = f.target.begin_phase().unwrap().unwrap();
    *f.enclosing.phase_error.borrow_mut() = Some(HostFault::Original(1));
    f.target.end_phase(marker);
    drop(buffer);
    let failure = match boundary.finish::<()>(Err(Error::CallbackPanicked)) {
        Err(Error::ConversionCleanup(failure)) => failure,
        other => panic!("unexpected cleanup result: {other:?}"),
    };
    assert!(matches!(
        failure.primary(),
        Some(CleanupPrimary::CallbackPanicked)
    ));
    assert_eq!(
        failure.host_error().unwrap().downcast_ref::<HostFault>(),
        Some(&HostFault::Original(1))
    );
    assert_eq!(
        observation.status().first(),
        Some(CleanupFault::HostPhaseCompletion)
    );
    assert_eq!(observation.status().failures(), Some(2));
    assert_eq!(f.owner.cleanup_status().failures(), Some(2));
    assert_eq!(f.enclosing.phase_starts.get(), 1);
    assert_eq!(f.enclosing.phase_ends.get(), 1);
    assert_eq!(f.enclosing.live(), 0);
}

#[test]
fn profiling_changes_observations_without_changing_tariffs_limits_or_release() {
    for mode in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
        let f = fixture(2, 3, 6, u64::MAX, 2, mode);
        let marker = f.target.begin_phase().unwrap().unwrap();
        f.target.work(Work::Value, 2).unwrap();
        f.target.work(Work::CodeUnit, 1).unwrap();
        assert!(matches!(
            f.target.work(Work::Argument, 1),
            Err(Error::TooManyArguments)
        ));
        let buffer = f.target.reserve(3).unwrap();
        f.target.end_phase(marker);
        drop(buffer);
        assert_eq!(f.enclosing.work(), 6);
        assert_eq!(f.enclosing.live(), 0);
        assert_eq!(f.target.state.local.usage().work(Work::Argument), 0);
        assert_eq!(f.target.state.local.usage().peak(), 3);
        let report = f.enclosing.ledger.snapshot().unwrap();
        match mode {
            ProfileMode::Off => assert!(report.is_none()),
            _ => {
                let report = report.unwrap();
                let child = report
                    .scopes
                    .iter()
                    .find(|scope| scope.scope == f.enclosing.child)
                    .unwrap();
                assert_eq!(child.phases, 1);
                assert_eq!(child.refused_batches, 1);
                assert_eq!(child.usage.work(Work::Value), 2);
                assert_eq!(child.usage.work(Work::CodeUnit), 1);
            }
        }
    }
}
