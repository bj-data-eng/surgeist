use super::*;
use crate::accounting::{
    RenderChangeKind, RenderClock, RenderCpuEvent, RenderPhaseFilter, RenderQuantity,
};
use std::{cell::RefCell, time::Duration};
use surgeist_ledger::Dimension;

#[derive(Clone, Copy, Debug)]
enum CancelAt {
    Open,
    Frozen,
    Ready,
    ClosedInterval,
}

#[test]
fn each_owned_cancellation_preserves_first_sources_and_its_real_capacity_policy() {
    for stage in [
        CancelAt::Open,
        CancelAt::Frozen,
        CancelAt::Ready,
        CancelAt::ClosedInterval,
    ] {
        let cpu = cpu();
        let history = ImageHistory::new(&cpu).unwrap();
        let bytes: Arc<[u8]> = Arc::from([1, 2, 3, 4, 5, 6, 7, 8]);
        let old_a = image(7, 20, Size::new(2.0, 1.0), Arc::clone(&bytes));
        let old_b = pixel(9, 40, [4, 3, 2, 1]);
        let new_a = image(80, 10, Size::new(1.0, 2.0), bytes);
        let new_b = pixel(90, 30, [9, 8, 7, 6]);
        publish(&history, &[&old_a, &old_b]);
        let original = canonical(&history);
        let original_capacity = canonical_capacity(&history);
        let counts = [
            Arc::strong_count(&old_a.bytes),
            Arc::strong_count(&old_b.bytes),
        ];
        let before = cpu.snapshot().unwrap();
        let draft = unlimited(|account| {
            let draft = history.begin(account)?;
            let observer = draft.observer();
            assert!(observer.observe(&new_a, account)?);
            assert!(observer.observe(&new_b, account)?);
            Ok(draft)
        });
        let candidate_capacity = candidate_capacity(&history);
        assert_eq!(candidate_capacity, 4);
        assert_eq!(
            canonical(&history).iter().map(|x| x.0).collect::<Vec<_>>(),
            [7, 9]
        );
        assert_eq!(history.identities_for_test().len(), 2);
        let observer = draft.observer();
        match stage {
            CancelAt::Open => drop(draft),
            CancelAt::Frozen => {
                let publication = draft.into_publication().unwrap();
                assert_stale(&observer, &new_b);
                assert_eq!(canonical(&history).len(), 2);
                drop(publication);
            }
            CancelAt::Ready => {
                let ready = draft.into_publication().unwrap().retire_old().unwrap();
                assert_stale(&observer, &new_b);
                assert_eq!(history.identities_for_test().len(), 2);
                drop(ready);
            }
            CancelAt::ClosedInterval => {
                let checkpoint = cpu.begin_interval().unwrap();
                let ready = draft.into_publication().unwrap().retire_old().unwrap();
                let interval = checkpoint.finish().unwrap();
                assert_eq!(interval.usage().work(RenderWork::ImageIdentityDrops), 2);
                drop(ready);
                assert_eq!(interval.usage().work(RenderWork::ImageIdentityDrops), 2);
            }
        }
        let restored = canonical(&history);
        assert_eq!(restored.iter().map(|x| x.0).collect::<Vec<_>>(), [7, 9]);
        for (entry, original) in restored.iter().zip(&original) {
            assert!(Arc::ptr_eq(&entry.1, &original.1));
        }
        assert_eq!(
            canonical_capacity(&history),
            if matches!(stage, CancelAt::Open) {
                original_capacity
            } else {
                candidate_capacity
            }
        );
        assert_eq!(Arc::strong_count(&old_a.bytes), counts[0] + 1);
        assert_eq!(Arc::strong_count(&old_b.bytes), counts[1] + 1);
        assert_eq!(Arc::strong_count(&new_b.bytes), 3); // caller bytes, identity and Peniko blob
        let after = cpu.snapshot().unwrap();
        assert_eq!(
            after.usage().work(RenderWork::ImageIdentityDrops)
                - before.usage().work(RenderWork::ImageIdentityDrops),
            4,
            "{stage:?}"
        );
        assert_eq!(
            after.usage().work(RenderWork::ImageIdentityCopies)
                - before.usage().work(RenderWork::ImageIdentityCopies),
            if matches!(stage, CancelAt::Open) {
                0
            } else {
                4
            },
            "{stage:?}"
        );
        let next = unlimited(|account| history.begin(account));
        assert_stale(&observer, &new_b);
        drop(next);
        cpu.ensure_healthy().unwrap();
    }
}

fn assert_stale(observer: &HistoryObserver, image: &Image) {
    let result = attempt(RenderLimits::new(), |account| {
        observer.observe(image, account)
    })
    .0;
    assert_eq!(
        result.unwrap_err().accounting_diagnostic(),
        Some(RenderAccountingError::Invariant)
    );
}

#[test]
fn refused_baseline_clone_releases_its_prefix_without_exposing_or_advancing_a_draft() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(7, 10, [1, 2, 3, 4]);
    let second = pixel(9, 20, [4, 3, 2, 1]);
    publish(&history, &[&first, &second]);
    let original = canonical(&history);
    let capacity = canonical_capacity(&history);
    let live = retained(&cpu);
    let next_attempt = history.inner.lock().next_attempt;
    let counts = [
        Arc::strong_count(&first.bytes),
        Arc::strong_count(&second.bytes),
    ];
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::ImageIdentityCopies, 1)
        .unwrap();
    let (result, report) = attempt(limits, |account| history.begin(account));
    assert!(result.is_err());
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ImageIdentityCopies),
        1
    );
    assert_eq!(Arc::strong_count(&history.inner), 1);
    assert_eq!(history.inner.lock().next_attempt, next_attempt);
    assert_eq!(canonical_capacity(&history), capacity);
    assert_eq!(retained(&cpu), live);
    assert_eq!(Arc::strong_count(&first.bytes), counts[0]);
    assert_eq!(Arc::strong_count(&second.bytes), counts[1]);
    for (entry, original) in canonical(&history).iter().zip(&original) {
        assert!(Arc::ptr_eq(&entry.1, &original.1));
    }
    publish(&history, &[&first]);
    cpu.ensure_healthy().unwrap();
}

#[test]
fn actual_frame_finalization_failure_keeps_ready_history_rollback_armed_after_retirement() {
    use crate::accounting::FrameFixture;
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(7, 20, [1, 2, 3, 4]);
    let new = pixel(9, 10, [4, 3, 2, 1]);
    publish(&history, &[&first]);
    let original = canonical(&history);
    let mut fixture = FrameFixture::new(&cpu, options);
    let finished = fixture
        .run_sync(|work| {
            let draft = history.begin(work.account())?;
            assert!(draft.observer().observe(&new, work.account())?);
            let ready = draft.into_publication()?.retire_old()?;
            cpu.at(RenderCpuScope::Cleanup)
                .maintenance(RenderWork::ImageIdentityDrops, u64::MAX);
            Ok(ready)
        })
        .unwrap();
    let (result, started, report, fault) = finished.into_parts();
    assert!(started);
    assert!(fault.is_some());
    assert_eq!(fault, cpu.fault());
    assert_eq!(
        report.status(),
        crate::RenderAttemptStatus::Failed(crate::ErrorCode::Accounting)
    );
    assert_eq!(report.cleanup_fault(), fault);
    let ready = result.unwrap();
    assert!(matches!(
        history.inner.lock().custody,
        Custody::Transaction {
            baseline: BaselineCustody::DraftOnly,
            ..
        }
    ));
    drop(ready);
    let restored = canonical(&history);
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].0, 7);
    assert!(Arc::ptr_eq(&restored[0].1, &original[0].1));
    assert_eq!(Arc::strong_count(&new.bytes), 3);
    assert_eq!(cpu.fault(), fault);
}

#[test]
fn consuming_phases_move_one_history_handle_and_weak_observers_retain_no_payload() {
    let cpu = cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(7, 10, [1, 2, 3, 4]);
    publish(&history, &[&first]);
    assert_eq!(Arc::strong_count(&first.bytes), 5); // three caller handles, history identity, registry payload
    assert_eq!(Arc::strong_count(&history.inner), 1);
    let draft = unlimited(|account| history.begin(account));
    let observer = draft.observer();
    assert_eq!(Arc::strong_count(&history.inner), 2);
    assert_eq!(Arc::weak_count(&history.inner), 1);
    assert_eq!(Arc::strong_count(&first.bytes), 6);
    let publication = draft.into_publication().unwrap();
    assert_eq!(Arc::strong_count(&history.inner), 2);
    assert_eq!(Arc::strong_count(&first.bytes), 6);
    let ready = publication.retire_old().unwrap();
    assert_eq!(Arc::strong_count(&history.inner), 2);
    assert_eq!(Arc::strong_count(&first.bytes), 5);
    ready.commit();
    assert_eq!(Arc::strong_count(&history.inner), 1);
    assert_eq!(Arc::strong_count(&first.bytes), 5);
    let source = Arc::downgrade(&first.bytes);
    drop(first);
    drop(history);
    assert_eq!(observer.inner.strong_count(), 0);
    assert_eq!(source.strong_count(), 0);
    assert!(observer.inner.upgrade().is_none());
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn history_payload_admission_refuses_the_old_native_budget_and_one_under_new_size() {
    let sample = cpu();
    let opening = retained(&sample);
    // These budgets are the actual production compiler measurements for this target.
    for (budget, accepted) in [(344, false), (375, false), (376, true)] {
        let cpu = RendererCpu::new(
            RenderAccountingOptions::new()
                .with_profile(RenderProfile::Summary { timing: false })
                .with_limits(RenderLimits::new().with_retained_bytes(opening + budget)),
        )
        .unwrap();
        let result = ImageHistory::new(&cpu);
        assert_eq!(result.is_ok(), accepted);
        if let Err(error) = &result {
            assert!(
                matches!(error.accounting_diagnostic(), Some(RenderAccountingError::CpuRefused {
                    scope: RenderCpuScope::Root,
                quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
                current,
                amount: 376,
                ..
            }) if current == opening)
            );
        } else {
            assert_eq!(retained(&cpu), opening + 376);
        }
        drop(result);
        assert_eq!(retained(&cpu), opening);
        cpu.ensure_healthy().unwrap();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SeenCustody {
    Canonical,
    Retained,
    DraftOnly,
    Locked,
    Gone,
}
#[derive(Clone, Debug)]
struct ClockRead {
    custody: SeenCustody,
    ids: Vec<u64>,
    source_owners: usize,
}
#[derive(Default)]
struct ClockState {
    ticks: u64,
    history: Option<Weak<HistoryInner>>,
    source: Option<Weak<[u8]>>,
    reads: Vec<ClockRead>,
}
thread_local! {
    static CLOCK: RefCell<ClockState> = RefCell::new(ClockState::default());
}
fn clock() -> Duration {
    CLOCK.with_borrow_mut(|clock| {
        clock.ticks += 1;
        let (custody, ids) = match clock.history.as_ref().and_then(Weak::upgrade) {
            Some(inner) => match inner.contents.state.try_lock() {
                Ok(state) => {
                    let custody = match state.custody {
                        Custody::Canonical => SeenCustody::Canonical,
                        Custody::Transaction {
                            baseline: BaselineCustody::Retained { .. },
                            ..
                        } => SeenCustody::Retained,
                        Custody::Transaction {
                            baseline: BaselineCustody::DraftOnly,
                            ..
                        } => SeenCustody::DraftOnly,
                    };
                    (
                        custody,
                        state
                            .records
                            .iter()
                            .map(|r| r.identity.fingerprint().get())
                            .collect(),
                    )
                }
                Err(std::sync::TryLockError::WouldBlock) => (SeenCustody::Locked, vec![]),
                Err(std::sync::TryLockError::Poisoned(_)) => panic!("history mutex poisoned"),
            },
            None => (SeenCustody::Gone, vec![]),
        };
        clock.reads.push(ClockRead {
            custody,
            ids,
            source_owners: clock.source.as_ref().map_or(0, Weak::strong_count),
        });
        Duration::from_millis(clock.ticks)
    })
}
fn reset_clock() {
    CLOCK.with_borrow_mut(|clock| *clock = ClockState::default());
}
fn timed_cpu() -> RendererCpu {
    reset_clock();
    RendererCpu::new(
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Detailed {
                timing: true,
                event_limit: 4096,
                filter: RenderPhaseFilter::All,
            })
            .with_clock(RenderClock::Host(clock)),
    )
    .unwrap()
}

#[test]
fn late_restore_releases_history_lock_and_ends_cleanup_before_canonical_availability() {
    let cpu = timed_cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(7, 20, [1, 2, 3, 4]);
    let new = pixel(9, 10, [4, 3, 2, 1]);
    publish(&history, &[&first]);
    let draft = unlimited(|account| {
        let draft = history.begin(account)?;
        draft.observer().observe(&new, account)?;
        Ok(draft)
    });
    let ready = draft.into_publication().unwrap().retire_old().unwrap();
    CLOCK.with_borrow_mut(|clock| {
        clock.history = Some(Arc::downgrade(&history.inner));
        clock.reads.clear();
    });
    drop(ready);
    let reads = CLOCK.with_borrow(|clock| clock.reads.clone());
    assert_eq!(reads.first().unwrap().custody, SeenCustody::DraftOnly);
    assert_eq!(reads.first().unwrap().ids, [9, 7]);
    assert!(reads.iter().any(|read| read.custody == SeenCustody::Locked));
    assert_eq!(reads.last().unwrap().custody, SeenCustody::DraftOnly);
    assert_eq!(reads.last().unwrap().ids, [7]);
    assert!(matches!(history.inner.lock().custody, Custody::Canonical));
    let snapshot = cpu.snapshot().unwrap();
    assert!(matches!(
        snapshot.events().last(),
        Some(RenderCpuEvent::Phase {
            scope: RenderCpuScope::Cleanup,
            ..
        })
    ));
    let next = unlimited(|account| history.begin(account));
    drop(next);
    cpu.ensure_healthy().unwrap();
}

#[test]
fn actual_record_import_and_backing_release_precede_cleanup_end_and_history_hold_release() {
    let cpu = timed_cpu();
    let opening = retained(&cpu);
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(7, 20, [1, 2, 3, 4]);
    publish(&history, &[&first]);
    let metadata = (canonical_capacity(&history) * size_of::<ContentRecord>()) as u64;
    let source = Arc::downgrade(&first.bytes);
    drop(first);
    let before = cpu.snapshot().unwrap().events().count();
    CLOCK.with_borrow_mut(|clock| {
        clock.history = Some(Arc::downgrade(&history.inner));
        clock.source = Some(source.clone());
        clock.reads.clear();
    });
    drop(history);
    let reads = CLOCK.with_borrow(|clock| clock.reads.clone());
    assert!(reads.first().unwrap().source_owners > 0);
    assert_eq!(reads.last().unwrap().source_owners, 0);
    assert_eq!(source.strong_count(), 0);
    let snapshot = cpu.snapshot().unwrap();
    assert_eq!(snapshot.fault(), None);
    let events: Vec<_> = snapshot.events().skip(before).collect();
    let release = |scope, amount| {
        events.iter().position(|event| matches!(event,
        RenderCpuEvent::Change { scope: actual, quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes), amount: actual_amount, kind: RenderChangeKind::Release, .. }
        if *actual == scope && *actual_amount == amount)).unwrap()
    };
    let payload = release(RenderCpuScope::ImagePayloads, 4);
    let backing = release(RenderCpuScope::ImageHistory, metadata);
    let cleanup_end = events
        .iter()
        .rposition(|event| {
            matches!(
                event,
                RenderCpuEvent::Phase {
                    scope: RenderCpuScope::Cleanup,
                    ..
                }
            )
        })
        .unwrap();
    let hold = release(
        RenderCpuScope::ImageHistory,
        size_of::<HistoryInner>() as u64,
    );
    assert!(payload < backing && backing < cleanup_end && cleanup_end < hold);
    assert!(snapshot.scope_elapsed(RenderCpuScope::Cleanup).unwrap() > Duration::ZERO);
    assert_eq!(
        snapshot
            .scope_usage(RenderCpuScope::ImageHistory)
            .unwrap()
            .live(RenderResource::RetainedCpuBytes),
        0
    );
    // Registry capacity remains owned by CPU, independently of its now-empty payloads.
    assert!(retained(&cpu) >= opening);
}

#[test]
fn final_commit_changes_only_canonical_authority_without_clock_or_ledger_events() {
    let cpu = timed_cpu();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(7, 20, [1, 2, 3, 4]);
    let draft = unlimited(|account| {
        let draft = history.begin(account)?;
        draft.observer().observe(&first, account)?;
        Ok(draft)
    });
    let ready = draft.into_publication().unwrap().retire_old().unwrap();
    let pointer = history.inner.lock().records[0]
        .identity
        .source_bytes()
        .as_ptr();
    let capacity = candidate_capacity(&history);
    let before = cpu.snapshot().unwrap();
    let reads = CLOCK.with_borrow(|clock| clock.ticks);
    ready.commit();
    assert_eq!(CLOCK.with_borrow(|clock| clock.ticks), reads);
    let after = cpu.snapshot().unwrap();
    assert_eq!(before.usage(), after.usage());
    assert_eq!(
        before.events().collect::<Vec<_>>(),
        after.events().collect::<Vec<_>>()
    );
    assert_eq!(
        before.scope_count(RenderCpuScope::Cleanup),
        after.scope_count(RenderCpuScope::Cleanup)
    );
    assert_eq!(canonical_capacity(&history), capacity);
    assert_eq!(
        history.inner.lock().records[0]
            .identity
            .source_bytes()
            .as_ptr(),
        pointer
    );
    assert_eq!(canonical(&history)[0].0, 7);
}

#[test]
fn profiles_filter_and_lost_detail_preserve_rollback_work_identity_and_first_fault() {
    let mut expected_work = None;
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 256,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 256,
            filter: RenderPhaseFilter::RendererCpuScope(RenderCpuScope::ImageHistory),
        },
    ] {
        reset_clock();
        let cpu = RendererCpu::new(
            RenderAccountingOptions::new()
                .with_profile(profile)
                .with_clock(RenderClock::Host(clock)),
        )
        .unwrap();
        let history = ImageHistory::new(&cpu).unwrap();
        let first = pixel(7, 20, [1, 2, 3, 4]);
        let equal = pixel(70, 20, [1, 2, 3, 4]);
        let new = pixel(9, 10, [4, 3, 2, 1]);
        publish(&history, &[&first]);
        let draft = unlimited(|account| {
            let draft = history.begin(account)?;
            assert!(draft.observer().observe(&new, account)?);
            assert!(!draft.observer().observe(&equal, account)?);
            Ok(draft)
        });
        let ready = draft.into_publication().unwrap().retire_old().unwrap();
        cpu.at(RenderCpuScope::Cleanup)
            .maintenance(RenderWork::ImageIdentityDrops, u64::MAX);
        let first_fault = cpu.fault();
        assert!(first_fault.is_some());
        drop(ready);
        let snapshot = cpu.snapshot().unwrap();
        assert_eq!(snapshot.fault(), first_fault);
        assert_eq!(canonical(&history)[0].0, 7);
        assert!(Arc::ptr_eq(&canonical(&history)[0].1, &first.bytes));
        assert_eq!(snapshot.usage().work(RenderWork::ImageIdentityCopies), 2);
        let work: Vec<_> = RenderWork::ALL
            .iter()
            .map(|unit| snapshot.usage().work(*unit))
            .collect();
        if let Some(expected) = &expected_work {
            assert_eq!(&work, expected);
        } else {
            expected_work = Some(work);
        }
        if matches!(profile, RenderProfile::Detailed { event_limit: 0, .. }) {
            assert_eq!(snapshot.events().count(), 0);
            assert!(snapshot.dropped_events() > 0);
        }
        drop(history);
        let timed = matches!(
            profile,
            RenderProfile::Summary { timing: true } | RenderProfile::Detailed { timing: true, .. }
        );
        assert_eq!(CLOCK.with_borrow(|clock| clock.ticks) == 0, !timed);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn observer_upgraded_before_freeze_cannot_mutate_the_frozen_or_next_attempt() {
    use std::sync::mpsc::{self, Receiver, SyncSender};
    thread_local! {
        static PAUSE: RefCell<Option<(SyncSender<()>, Receiver<()>)>> = const { RefCell::new(None) };
    }
    fn paused_clock() -> Duration {
        if let Some((entered, resume)) = PAUSE.with_borrow_mut(Option::take) {
            entered.send(()).unwrap();
            resume
                .recv_timeout(Duration::from_secs(10))
                .expect("resume upgraded observer");
        }
        clock()
    }
    let cpu = RendererCpu::new(
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Summary { timing: true })
            .with_clock(RenderClock::Host(paused_clock)),
    )
    .unwrap();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(7, 20, [1, 2, 3, 4]);
    let draft = unlimited(|account| history.begin(account));
    let observer = draft.observer();
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (resume_tx, resume_rx) = mpsc::sync_channel(1);
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    std::thread::scope(|scope| {
        let worker_observer = observer.clone();
        let worker_first = &first;
        let worker = scope.spawn(move || {
            PAUSE.with_borrow_mut(|pause| *pause = Some((entered_tx, resume_rx)));
            result_tx
                .send(
                    attempt(RenderLimits::new(), |account| {
                        worker_observer.observe(worker_first, account)
                    })
                    .0,
                )
                .unwrap();
        });
        entered_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("observer upgraded and entered CPU phase");
        assert_eq!(Arc::strong_count(&history.inner), 3);
        let publication = draft.into_publication().unwrap();
        resume_tx.send(()).unwrap();
        let result = result_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("stale observer result");
        assert_eq!(
            result.unwrap_err().accounting_diagnostic(),
            Some(RenderAccountingError::Invariant)
        );
        worker.join().unwrap();
        assert_eq!(history.inner.lock().records.len(), 0);
        drop(publication);
    });
    let next = unlimited(|account| history.begin(account));
    assert_stale(&observer, &first);
    drop(next);
    drop(history);
    assert_eq!(observer.inner.strong_count(), 0);
    cpu.ensure_healthy().unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn concurrent_upgraded_observer_cannot_deadlock_or_mutate_late_rollback() {
    use std::sync::mpsc::{self, Receiver, SyncSender};
    use std::time::Instant;
    thread_local! {
        static PAUSE: RefCell<Option<(SyncSender<()>, Receiver<()>)>> = const { RefCell::new(None) };
    }
    fn paused_clock() -> Duration {
        let time = clock();
        let filtering =
            CLOCK.with_borrow(|clock| clock.reads.last().unwrap().custody == SeenCustody::Locked);
        if filtering && let Some((entered, resume)) = PAUSE.with_borrow_mut(Option::take) {
            entered.send(()).unwrap();
            resume
                .recv_timeout(Duration::from_secs(10))
                .expect("resume physical rollback");
        }
        time
    }
    let cpu = RendererCpu::new(
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Summary { timing: true })
            .with_clock(RenderClock::Host(paused_clock)),
    )
    .unwrap();
    let history = ImageHistory::new(&cpu).unwrap();
    let first = pixel(7, 20, [1, 2, 3, 4]);
    let new = pixel(9, 10, [4, 3, 2, 1]);
    publish(&history, &[&first]);
    let draft = unlimited(|account| {
        let draft = history.begin(account)?;
        draft.observer().observe(&new, account)?;
        Ok(draft)
    });
    let observer = draft.observer();
    let ready = draft.into_publication().unwrap().retire_old().unwrap();
    let weak = Arc::downgrade(&history.inner);
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (resume_tx, resume_rx) = mpsc::sync_channel(1);
    let (result_tx, result_rx) = mpsc::sync_channel(1);
    std::thread::scope(|scope| {
        let rollback = scope.spawn(move || {
            CLOCK.with_borrow_mut(|clock| clock.history = Some(weak));
            PAUSE.with_borrow_mut(|pause| *pause = Some((entered_tx, resume_rx)));
            drop(ready);
        });
        entered_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("filtering under history lock");
        let observer = &observer;
        let first = &first;
        let observation = scope.spawn(move || {
            result_tx
                .send(
                    attempt(RenderLimits::new(), |account| {
                        observer.observe(first, account)
                    })
                    .0,
                )
                .unwrap();
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        while Arc::strong_count(&history.inner) != 3 {
            assert!(
                Instant::now() < deadline,
                "observer did not upgrade while rollback was paused"
            );
            std::thread::yield_now();
        }
        resume_tx.send(()).unwrap();
        let result = result_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("concurrent stale observer completion");
        assert_eq!(
            result.unwrap_err().accounting_diagnostic(),
            Some(RenderAccountingError::Invariant)
        );
        observation.join().unwrap();
        rollback.join().unwrap();
    });
    assert_eq!(
        canonical(&history).iter().map(|x| x.0).collect::<Vec<_>>(),
        [7]
    );
    assert_eq!(Arc::strong_count(&new.bytes), 3);
    let next = unlimited(|account| history.begin(account));
    drop(next);
    cpu.ensure_healthy().unwrap();
}
