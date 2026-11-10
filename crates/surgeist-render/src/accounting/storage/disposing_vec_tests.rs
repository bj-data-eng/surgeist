use super::*;
use crate::accounting::{
    RenderAccountingOptions, RenderClock, RenderCpuScope, RenderCpuSnapshot, RenderProfile,
    RenderWork, RendererCpu,
};
use std::{
    cell::Cell,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use surgeist_ledger::Dimension;

static TIME: AtomicU64 = AtomicU64::new(0);
static CLOCK_LOCK: Mutex<()> = Mutex::new(());
thread_local! {
    static CLEANUPS: Cell<usize> = const { Cell::new(0) };
}

fn clock() -> Duration {
    Duration::from_millis(TIME.load(Ordering::SeqCst))
}
fn forbidden_clock() -> Duration {
    panic!("off and untimed disposal must not read the clock")
}
fn options(profile: RenderProfile) -> RenderAccountingOptions {
    RenderAccountingOptions::new()
        .with_profile(profile)
        .with_clock(RenderClock::Host(
            if matches!(profile, RenderProfile::Summary { timing: true }) {
                clock
            } else {
                forbidden_clock
            },
        ))
}
fn profiles() -> [RenderProfile; 3] {
    [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ]
}

struct Payload {
    id: usize,
    bytes: Option<AccountedVec<u8>>,
    cpu: RendererCpu,
    outer_bytes: u64,
    destroyed: Arc<Mutex<Vec<usize>>>,
}
impl Drop for Payload {
    fn drop(&mut self) {
        assert_eq!(
            self.cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            self.outer_bytes
        );
        drop(self.bytes.take());
        // The real nested payload allocation is gone; original vector custody
        // must still cover this complete element destructor.
        assert_eq!(
            self.cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            self.outer_bytes
        );
        TIME.fetch_add(7, Ordering::SeqCst);
        self.destroyed.lock().unwrap().push(self.id);
    }
}

fn vector(
    cpu: &RendererCpu,
    count: usize,
    capacity: usize,
    destroyed: &Arc<Mutex<Vec<usize>>>,
) -> AccountedVec<Payload> {
    let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
    let mut values =
        AccountedVec::new_in(&storage, capacity, RenderResource::ScratchCpuBytes).unwrap();
    let outer_bytes = cpu
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::ScratchCpuBytes);
    for id in 0..count {
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::PreparedValues));
        let mut bytes =
            AccountedVec::new_in(&storage, 5, RenderResource::RetainedCpuBytes).unwrap();
        bytes.push(id as u8).unwrap();
        values
            .push(Payload {
                id,
                bytes: Some(bytes),
                cpu: cpu.clone(),
                outer_bytes,
                destroyed: destroyed.clone(),
            })
            .unwrap();
    }
    values
}

fn dispose<T>(values: AccountedVec<T>) {
    CLEANUPS.with(|calls| calls.set(calls.get() + 1));
    drop(values);
}

fn unchanged(before: &RenderCpuSnapshot, after: &RenderCpuSnapshot) {
    assert_eq!(after.identity(), before.identity());
    assert_eq!(after.usage(), before.usage());
    assert_eq!(after.fault(), before.fault());
    assert_eq!(
        after.events().collect::<Vec<_>>(),
        before.events().collect::<Vec<_>>()
    );
    assert_eq!(after.dropped_events(), before.dropped_events());
    for scope in RenderCpuScope::ALL {
        assert_eq!(after.scope_count(scope), before.scope_count(scope));
        assert_eq!(after.scope_elapsed(scope), before.scope_elapsed(scope));
    }
}

#[test]
fn construction_and_extraction_emit_no_admission_release_work_or_phase_events() {
    use crate::accounting::{RenderChangeKind, RenderCpuEvent, RenderPhaseFilter, RenderQuantity};
    let cpu = RendererCpu::new(
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Detailed {
                timing: false,
                event_limit: 128,
                filter: RenderPhaseFilter::All,
            })
            .with_clock(RenderClock::Host(forbidden_clock)),
    )
    .unwrap();
    let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
    let values = AccountedVec::<u8>::new_in(&storage, 8, RenderResource::ScratchCpuBytes).unwrap();
    let backing_bytes = values.capacity() as u64;
    let before = cpu.snapshot().unwrap();
    assert_eq!(before.dropped_events(), 0);
    assert!(before.events().any(|event| matches!(
        event,
        RenderCpuEvent::Change {
            quantity: RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
            kind: RenderChangeKind::Acquire,
            amount,
            ..
        } if amount == backing_bytes
    )));
    let owner = DisposingVec::new(values, dispose);
    unchanged(&before, &cpu.snapshot().unwrap());
    let values = owner.into_inner();
    unchanged(&before, &cpu.snapshot().unwrap());
    drop(values);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::ScratchCpuBytes),
        0
    );
}

#[derive(Clone, Copy, Debug)]
enum Exit {
    Drop,
    Error,
    Unwind,
    Extract,
}

fn return_error<T>(owner: DisposingVec<T>) -> Result<()> {
    let _owner = owner;
    Err(crate::Error::invalid_value(
        "disposal fixture",
        "refused",
        "preserve this original error",
    ))
}

#[test]
fn armed_and_extracted_vectors_keep_exact_backing_and_custody_through_real_payload_drop() {
    let _clock = CLOCK_LOCK.lock().unwrap();
    for profile in profiles() {
        for (count, capacity) in [(0, 0), (0, 3), (1, 3), (3, 3)] {
            for exit in [Exit::Drop, Exit::Error, Exit::Unwind, Exit::Extract] {
                TIME.store(0, Ordering::SeqCst);
                CLEANUPS.with(|calls| calls.set(0));
                let cpu = RendererCpu::new(options(profile)).unwrap();
                let opening = cpu.snapshot().unwrap();
                let destroyed = Arc::new(Mutex::new(Vec::new()));
                let values = vector(&cpu, count, capacity, &destroyed);
                let pointer = values.as_ptr();
                let capacity = values.capacity();
                let admitted = values.backing().unwrap().amount().unwrap();
                assert_eq!(admitted, (capacity * std::mem::size_of::<Payload>()) as u64);
                let payloads = values
                    .iter()
                    .map(|value| value.bytes.as_ref().unwrap().as_ptr())
                    .collect::<Vec<_>>();
                let before = cpu.snapshot().unwrap();
                let owner = DisposingVec::new(values, dispose);
                unchanged(&before, &cpu.snapshot().unwrap());
                assert_eq!(owner.as_slice().as_ptr(), pointer);
                assert_eq!(owner.len(), count);
                assert_eq!(owner.backing_for_test().capacity(), capacity);
                assert_eq!(owner.vector.backing().unwrap().amount().unwrap(), admitted);
                assert_eq!(
                    owner.vector.backing().unwrap().resource(),
                    RenderResource::ScratchCpuBytes
                );
                assert_eq!(
                    owner
                        .as_slice()
                        .iter()
                        .map(|value| value.bytes.as_ref().unwrap().as_ptr())
                        .collect::<Vec<_>>(),
                    payloads
                );
                assert_eq!(
                    owner
                        .as_slice()
                        .iter()
                        .map(|value| value.id)
                        .collect::<Vec<_>>(),
                    (0..count).collect::<Vec<_>>()
                );
                match exit {
                    Exit::Drop => drop(owner),
                    Exit::Error => {
                        let result = return_error(owner);
                        assert_eq!(result.unwrap_err().code(), crate::ErrorCode::InvalidInput);
                    }
                    Exit::Unwind => {
                        assert!(
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                                let _owner = owner;
                                panic!("external unwind around armed storage owner");
                            }))
                            .is_err()
                        );
                    }
                    Exit::Extract => {
                        let before_extract = cpu.snapshot().unwrap();
                        let values = owner.into_inner();
                        unchanged(&before_extract, &cpu.snapshot().unwrap());
                        assert_eq!(values.as_ptr(), pointer);
                        assert_eq!(values.capacity(), capacity);
                        assert_eq!(values.len(), count);
                        assert_eq!(values.backing().unwrap().amount().unwrap(), admitted);
                        assert_eq!(
                            values.backing().unwrap().resource(),
                            RenderResource::ScratchCpuBytes
                        );
                        assert_eq!(
                            values
                                .iter()
                                .map(|value| value.bytes.as_ref().unwrap().as_ptr())
                                .collect::<Vec<_>>(),
                            payloads
                        );
                        CLEANUPS.with(|calls| assert_eq!(calls.get(), 0));
                        drop(values);
                    }
                }
                CLEANUPS.with(|calls| {
                    assert_eq!(calls.get(), usize::from(!matches!(exit, Exit::Extract)))
                });
                let mut actual = destroyed.lock().unwrap().clone();
                actual.sort_unstable();
                assert_eq!(actual, (0..count).collect::<Vec<_>>());
                let after = cpu.snapshot().unwrap();
                assert_eq!(
                    after.usage().live(RenderResource::ScratchCpuBytes),
                    opening.usage().live(RenderResource::ScratchCpuBytes)
                );
                assert_eq!(
                    after.usage().live(RenderResource::RetainedCpuBytes),
                    opening.usage().live(RenderResource::RetainedCpuBytes)
                );
                assert_eq!(
                    after.usage().peak(RenderResource::ScratchCpuBytes),
                    before.usage().peak(RenderResource::ScratchCpuBytes)
                );
                for &work in RenderWork::ALL {
                    assert_eq!(after.usage().work(work), before.usage().work(work));
                }
                assert_eq!(after.fault(), None);
                assert_eq!(
                    after.scope_count(RenderCpuScope::FrameBacking)
                        - before.scope_count(RenderCpuScope::FrameBacking),
                    u64::from(!matches!(profile, RenderProfile::Off))
                );
                if matches!(profile, RenderProfile::Summary { timing: true }) {
                    assert_eq!(
                        after.scope_elapsed(RenderCpuScope::FrameBacking).unwrap()
                            - before.scope_elapsed(RenderCpuScope::FrameBacking).unwrap(),
                        Duration::from_millis(7 * count as u64)
                    );
                } else {
                    assert_eq!(after.scope_elapsed(RenderCpuScope::FrameBacking), None);
                }
            }
        }
    }
}

fn fail_after_prefix(values: AccountedVec<Payload>) -> Result<()> {
    let mut values = values.into_iter();
    drop(values.next().unwrap());
    Err(crate::Error::invalid_value(
        "owning traversal",
        "refused",
        "release the unread suffix",
    ))
}
fn suffix_after_error(values: AccountedVec<Payload>) {
    CLEANUPS.with(|calls| calls.set(calls.get() + 1));
    let result = fail_after_prefix(values);
    assert_eq!(result.unwrap_err().code(), crate::ErrorCode::InvalidInput);
}
fn suffix_after_unwind(values: AccountedVec<Payload>) {
    CLEANUPS.with(|calls| calls.set(calls.get() + 1));
    let mut values = values.into_iter();
    drop(values.next().unwrap());
    panic!("cleanup callback unwind after one consumed value");
}

#[test]
fn callback_error_and_unwind_keep_unread_suffix_inside_original_backing_disposal() {
    let _clock = CLOCK_LOCK.lock().unwrap();
    for profile in profiles() {
        for unwind in [false, true] {
            TIME.store(0, Ordering::SeqCst);
            CLEANUPS.with(|calls| calls.set(0));
            let cpu = RendererCpu::new(options(profile)).unwrap();
            let opening = cpu.snapshot().unwrap();
            let destroyed = Arc::new(Mutex::new(Vec::new()));
            let values = vector(&cpu, 3, 3, &destroyed);
            let before = cpu.snapshot().unwrap();
            let owner = DisposingVec::new(
                values,
                if unwind {
                    suffix_after_unwind
                } else {
                    suffix_after_error
                },
            );
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(owner)));
            assert_eq!(result.is_err(), unwind);
            CLEANUPS.with(|calls| assert_eq!(calls.get(), 1));
            let mut actual = destroyed.lock().unwrap().clone();
            assert_eq!(actual.first(), Some(&0));
            actual.sort_unstable();
            assert_eq!(actual, [0, 1, 2]);
            let after = cpu.snapshot().unwrap();
            assert_eq!(
                after.usage().live(RenderResource::ScratchCpuBytes),
                opening.usage().live(RenderResource::ScratchCpuBytes)
            );
            assert_eq!(
                after.usage().live(RenderResource::RetainedCpuBytes),
                opening.usage().live(RenderResource::RetainedCpuBytes)
            );
            assert_eq!(after.fault(), None);
            for &work in RenderWork::ALL {
                assert_eq!(after.usage().work(work), before.usage().work(work));
            }
            assert_eq!(
                after.scope_count(RenderCpuScope::FrameBacking)
                    - before.scope_count(RenderCpuScope::FrameBacking),
                u64::from(!matches!(profile, RenderProfile::Off))
            );
            if matches!(profile, RenderProfile::Summary { timing: true }) {
                // First consumed value is outside the iterator's backing
                // disposal; the two unread values remain inside that interval.
                assert_eq!(
                    after.scope_elapsed(RenderCpuScope::FrameBacking).unwrap()
                        - before.scope_elapsed(RenderCpuScope::FrameBacking).unwrap(),
                    Duration::from_millis(14)
                );
            } else {
                assert_eq!(after.scope_elapsed(RenderCpuScope::FrameBacking), None);
            }
        }
    }
}

#[test]
fn push_uses_original_checked_growth_and_preserves_backing_on_refusal() {
    let cpu = RendererCpu::new(
        RenderAccountingOptions::new()
            .with_limits(crate::RenderLimits::new().with_scratch_bytes(1)),
    )
    .unwrap();
    let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
    let values = AccountedVec::new_in(&storage, 1, RenderResource::ScratchCpuBytes).unwrap();
    let mut owner = DisposingVec::new(values, dispose);
    owner.push(7_u8).unwrap();
    let pointer = owner.as_slice().as_ptr();
    let before = cpu.snapshot().unwrap();
    let error = owner.push(9).unwrap_err();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(crate::RenderAccountingError::CpuRefused {
            quantity: crate::RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
            reason: crate::RenderRefusalReason::Limit { limit: 1 },
            ..
        })
    ));
    assert_eq!(owner.as_slice(), [7]);
    assert_eq!(owner.as_slice().as_ptr(), pointer);
    assert_eq!(cpu.snapshot().unwrap().usage(), before.usage());
    drop(owner.into_inner());
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::ScratchCpuBytes),
        0
    );

    let cpu = RendererCpu::new(RenderAccountingOptions::new()).unwrap();
    let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
    let values = AccountedVec::new_in(&storage, 1, RenderResource::ScratchCpuBytes).unwrap();
    let mut owner = DisposingVec::new(values, dispose);
    owner.push(7_u8).unwrap();
    owner.push(9).unwrap();
    assert_eq!(owner.as_slice(), [7, 9]);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .work(RenderWork::StorageRelocations),
        1
    );
    drop(owner.into_inner());
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::ScratchCpuBytes),
        0
    );
}

#[test]
fn zero_sized_values_keep_logical_contents_without_fabricating_backing_bytes() {
    let cpu = RendererCpu::new(RenderAccountingOptions::new()).unwrap();
    let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
    for extract in [false, true] {
        CLEANUPS.with(|calls| calls.set(0));
        let values = AccountedVec::new_in(&storage, 0, RenderResource::ScratchCpuBytes).unwrap();
        let mut owner = DisposingVec::new(values, dispose);
        for _ in 0..3 {
            owner.push(()).unwrap();
        }
        assert_eq!(owner.as_slice(), [(), (), ()]);
        assert_eq!(owner.vector.backing().unwrap().amount().unwrap(), 0);
        if extract {
            let values = owner.into_inner();
            assert_eq!(values.len(), 3);
            assert_eq!(values.capacity(), Vec::<()>::new().capacity());
            drop(values);
        } else {
            drop(owner);
        }
        CLEANUPS.with(|calls| assert_eq!(calls.get(), usize::from(!extract)));
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
        assert_eq!(cpu.snapshot().unwrap().fault(), None);
    }
}
