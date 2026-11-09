use super::*;
use crate::{
    JsError, JsValue,
    context::time::FixedClock,
    error::RuntimeLimitError,
    execution::{EventFilter, ExecutionOptions, ProfileMode},
    job::{ContextTask, JobStatus, NativeAsyncJob, PromiseJob, TimeoutJob},
};
use std::mem::size_of;

fn fixture() -> (Context, Rc<SimpleJobExecutor>, Rc<RuntimeJobQueues>) {
    let executor = Rc::new(SimpleJobExecutor::new());
    let builder = Context::builder()
        .job_executor(executor.clone())
        .clock(Rc::new(FixedClock::from_millis(0)));
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut owner = builder.build().unwrap();
    let queues = executor.attach_context(&mut owner).unwrap();
    (owner, executor, queues)
}
fn options(limit: u64) -> ExecutionOptions {
    ExecutionOptions::new(0).with_job_claim_work_limit(limit)
}
fn refusal(error: &JsError, limit: u64) {
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::JobClaimWork { limit }
        ))
    );
}
fn callback(id: usize, log: &Rc<RefCell<Vec<usize>>>, realm: crate::realm::Realm) -> GenericJob {
    let log = log.clone();
    GenericJob::new(
        move |_| {
            log.borrow_mut().push(id);
            Ok(JsValue::undefined())
        },
        realm,
    )
}
fn slots(queues: &RuntimeJobQueues, kind: Kind) -> Vec<Rc<Slot>> {
    let store = queues.store.borrow();
    let mut cursor = store.bucket(kind).unwrap().head;
    let mut slots = Vec::new();
    while let Some(index) = cursor {
        let slot = store.node(index).unwrap();
        slots.push(slot.clone());
        cursor = slot.next.get();
    }
    slots
}
fn assert_available(slots: &[Rc<Slot>]) {
    assert!(
        slots
            .iter()
            .all(|slot| slot.status.get() == Status::Available)
    );
    for slot in slots {
        assert_eq!(
            slot.observer.as_ref().unwrap().status().unwrap(),
            JobStatus::Pending
        );
    }
}

// The declared scratch payload is one Rc<Slot>. These positive events are
// derived from the adopted logical growth 0,1,2,4,..., not allocator capacity.
fn one_bucket_debits(candidates: usize) -> Vec<u64> {
    let unit = u64::try_from(size_of::<Rc<Slot>>()).unwrap();
    let mut debits = vec![1]; // selected bucket, even when empty
    let mut capacity = 0;
    for len in 0..candidates {
        debits.push(1); // linked node
        if len == capacity {
            let next = if capacity == 0 { 1 } else { capacity * 2 };
            debits.push(u64::try_from(next - capacity).unwrap() * unit);
            if len != 0 {
                debits.push(u64::try_from(len).unwrap() * unit);
            }
            capacity = next;
        }
        debits.push(1); // available Rc snapshot entry
    }
    if candidates != 0 {
        debits.push(u64::try_from(candidates).unwrap());
    }
    debits
}
fn total(candidates: usize) -> u64 {
    one_bucket_debits(candidates).into_iter().sum()
}

#[test]
fn ordinary_empty_buckets_price_one_head_but_an_empty_clock_range_prices_zero() {
    for selection in [
        ClaimSelection::Async,
        ClaimSelection::Promise,
        ClaimSelection::Generic,
    ] {
        for limit in [0, 1] {
            let (mut owner, _, queues) = fixture();
            let outcome = owner
                .with_execution(options(limit), |access| {
                    queues.claim(selection, &access.into_host_access())
                })
                .unwrap();
            if limit == 0 {
                let Err(error) = outcome.result else {
                    panic!("the selected empty head requires one credit")
                };
                refusal(&error, 0);
            } else {
                assert!(outcome.result.unwrap().slots.is_empty());
            }
            assert_eq!(outcome.report.job_claim_work().unwrap(), limit);
            assert!(!outcome.report.effects_started());
        }
    }
    let (mut owner, _, queues) = fixture();
    let outcome = owner
        .with_execution(options(0), |access| {
            queues.claim(
                ClaimSelection::ClocksBefore(JsInstant::new(1, 0)),
                &access.into_host_access(),
            )
        })
        .unwrap();
    assert!(outcome.result.unwrap().slots.is_empty());
    assert_eq!(outcome.report.job_claim_work().unwrap(), 0);
    assert!(!outcome.report.effects_started());
}

#[test]
fn each_ordinary_selector_claims_only_its_real_job_kind_and_preserves_other_payloads() {
    let (mut owner, _, queues) = fixture();
    let log = Rc::new(RefCell::new(Vec::new()));
    let async_log = log.clone();
    owner
        .enqueue_job(
            NativeAsyncJob::new(move |_| {
                Ok(ContextTask::from_future(async move {
                    async_log.borrow_mut().push(0);
                    Ok(JsValue::undefined())
                }))
            })
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();
    let promise_log = log.clone();
    owner
        .enqueue_job(
            PromiseJob::with_realm(
                move |_| {
                    promise_log.borrow_mut().push(1);
                    Ok(JsValue::undefined())
                },
                owner.realm().clone(),
            )
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();
    owner
        .enqueue_job(callback(2, &log, owner.realm().clone()).into())
        .unwrap()
        .into_result()
        .unwrap();
    for (selection, selected_kind, other_kinds) in [
        (
            ClaimSelection::Async,
            Kind::Ordinary(Ordinary::Async),
            [
                Kind::Ordinary(Ordinary::Promise),
                Kind::Ordinary(Ordinary::Generic),
            ],
        ),
        (
            ClaimSelection::Promise,
            Kind::Ordinary(Ordinary::Promise),
            [
                Kind::Ordinary(Ordinary::Async),
                Kind::Ordinary(Ordinary::Generic),
            ],
        ),
        (
            ClaimSelection::Generic,
            Kind::Ordinary(Ordinary::Generic),
            [
                Kind::Ordinary(Ordinary::Async),
                Kind::Ordinary(Ordinary::Promise),
            ],
        ),
    ] {
        let outcome = owner
            .with_execution(options(total(1)), |access| {
                queues.claim(selection, &access.into_host_access())
            })
            .unwrap();
        assert_eq!(outcome.report.job_claim_work().unwrap(), total(1));
        assert!(!outcome.report.effects_started());
        let batch = outcome.result.unwrap();
        assert_eq!(batch.slots.len(), 1);
        assert!(
            slots(&queues, selected_kind)
                .iter()
                .all(|slot| slot.status.get() == Status::Claimed)
        );
        for kind in other_kinds {
            assert_available(&slots(&queues, kind));
        }
        assert!(log.borrow().is_empty());
        drop(batch);
    }
    let drained = owner
        .with_execution(options(100_000), |mut access| access.run_jobs())
        .unwrap();
    drained.result.unwrap();
    assert_eq!(drained.report.job_preparations().unwrap(), 1);
    assert_eq!(drained.report.job_polls().unwrap(), 1);
    assert_eq!(drained.report.job_invocations().unwrap(), 2);
    assert_eq!(*log.borrow(), [0, 1, 2]);
}

#[test]
fn each_positive_snapshot_prefix_and_atomic_final_mark_batch_refuse_without_claiming_any_slot() {
    for candidates in [1, 2, 3, 5] {
        let mut prefix = 0;
        for debit in one_bucket_debits(candidates) {
            let limit = prefix + debit - 1;
            let (mut owner, _, queues) = fixture();
            let log = Rc::new(RefCell::new(Vec::new()));
            for id in 0..candidates {
                owner
                    .enqueue_job(callback(id, &log, owner.realm().clone()).into())
                    .unwrap()
                    .into_result()
                    .unwrap();
            }
            let selected = slots(&queues, Kind::Ordinary(Ordinary::Generic));
            let mut direct = None;
            let outcome = owner
                .with_execution(options(limit), |access| {
                    let result = queues.claim(ClaimSelection::Generic, &access.into_host_access());
                    direct = result.as_ref().err().cloned();
                    result
                })
                .unwrap();
            refusal(direct.as_ref().unwrap(), limit);
            let Err(error) = outcome.result else {
                panic!("the targeted positive debit must refuse")
            };
            assert_eq!(error, direct.unwrap());
            assert_eq!(
                outcome.report.job_claim_work().unwrap(),
                prefix,
                "a refused byte or final-mark batch is atomic"
            );
            assert!(!outcome.report.effects_started());
            assert!(log.borrow().is_empty());
            assert_available(&selected);
            // Actual later execution proves these are callable retained jobs,
            // rather than an incidental slot count left after failed scanning.
            owner
                .with_execution(options(100_000), |mut access| access.run_jobs())
                .unwrap()
                .result
                .unwrap();
            assert_eq!(*log.borrow(), (0..candidates).collect::<Vec<_>>());
            prefix += debit;
        }
        assert_eq!(prefix, total(candidates));
    }
}

#[test]
fn exact_commit_marks_every_candidate_and_batch_drop_restores_without_more_admission() {
    for candidates in [1, 2, 3, 5] {
        let (mut owner, _, queues) = fixture();
        let log = Rc::new(RefCell::new(Vec::new()));
        for id in 0..candidates {
            owner
                .enqueue_job(callback(id, &log, owner.realm().clone()).into())
                .unwrap()
                .into_result()
                .unwrap();
        }
        let selected = slots(&queues, Kind::Ordinary(Ordinary::Generic));
        let exact = total(candidates);
        let outcome = owner
            .with_execution(
                options(exact).with_profile(ProfileMode::Summary { timing: false }),
                |access| {
                    let host = access.into_host_access();
                    let batch = queues.claim(ClaimSelection::Generic, &host)?;
                    assert_eq!(batch.slots.len(), candidates);
                    assert!(
                        selected
                            .iter()
                            .all(|slot| slot.status.get() == Status::Claimed)
                    );
                    // Force a real later refusal at the next selected bucket. Cleanup
                    // of the accepted batch must remain allocation/admission-free.
                    let error = queues.claim(ClaimSelection::Promise, &host).err().unwrap();
                    refusal(&error, exact);
                    drop(batch);
                    assert_available(&selected);
                    Ok(())
                },
            )
            .unwrap();
        refusal(&outcome.result.unwrap_err(), exact);
        assert_eq!(outcome.report.job_claim_work().unwrap(), exact);
        assert!(!outcome.report.effects_started());
        let profile = outcome.report.profile().unwrap().unwrap();
        let dimension = crate::execution::ExecutionWork::JobClaimWork;
        for (origin, expected) in [
            ("claim-bucket", 1),
            ("claim-node", candidates as u64),
            ("claim-snapshot-copy", candidates as u64),
            ("claim-commit", candidates as u64),
        ] {
            assert_eq!(
                profile
                    .scopes
                    .iter()
                    .find(|scope| scope.name == origin)
                    .unwrap()
                    .usage
                    .work(dimension),
                expected
            );
        }
        assert!(log.borrow().is_empty());
        owner
            .with_execution(options(100_000), |mut access| access.run_jobs())
            .unwrap()
            .result
            .unwrap();
        assert_eq!(*log.borrow(), (0..candidates).collect::<Vec<_>>());
    }
}

#[test]
fn every_profile_mode_preserves_exact_and_one_short_atomic_claim_outcomes() {
    fn clock() -> std::time::Duration {
        std::time::Duration::ZERO
    }
    for mode in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 0,
            filter: EventFilter::ScopeName("job-claim"),
        },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 8,
            filter: EventFilter::ScopeName("job-claim"),
        },
    ] {
        let exact = total(3);
        for limit in [exact - 1, exact] {
            let (mut owner, _, queues) = fixture();
            let log = Rc::new(RefCell::new(Vec::new()));
            for id in 0..3 {
                owner
                    .enqueue_job(callback(id, &log, owner.realm().clone()).into())
                    .unwrap()
                    .into_result()
                    .unwrap();
            }
            let selected = slots(&queues, Kind::Ordinary(Ordinary::Generic));
            let outcome = owner
                .with_execution(
                    options(limit).with_profile(mode).with_profile_clock(clock),
                    |access| queues.claim(ClaimSelection::Generic, &access.into_host_access()),
                )
                .unwrap();
            if limit == exact {
                let batch = outcome.result.unwrap();
                assert_eq!(outcome.report.job_claim_work().unwrap(), exact);
                assert!(
                    selected
                        .iter()
                        .all(|slot| slot.status.get() == Status::Claimed)
                );
                drop(batch);
            } else {
                let Err(error) = outcome.result else {
                    panic!("three marks require one atomic three-credit batch")
                };
                refusal(&error, limit);
                assert_eq!(outcome.report.job_claim_work().unwrap(), exact - 3);
            }
            assert_available(&selected);
            assert!(log.borrow().is_empty());
            assert!(!outcome.report.effects_started());
        }
    }
}

#[test]
fn unrelated_resident_jobs_do_not_add_snapshot_capacity_or_relocation_to_the_selected_bucket() {
    for unrelated in [0, 64] {
        let (mut owner, _, queues) = fixture();
        let factories = Rc::new(Cell::new(0));
        for _ in 0..unrelated {
            let factories = factories.clone();
            owner
                .enqueue_job(
                    NativeAsyncJob::new(move |_| {
                        factories.set(factories.get() + 1);
                        Ok(ContextTask::from_future(async { Ok(JsValue::undefined()) }))
                    })
                    .into(),
                )
                .unwrap()
                .into_result()
                .unwrap();
        }
        let log = Rc::new(RefCell::new(Vec::new()));
        for id in 0..2 {
            owner
                .enqueue_job(callback(id, &log, owner.realm().clone()).into())
                .unwrap()
                .into_result()
                .unwrap();
        }
        let outcome = owner
            .with_execution(
                options(total(2)).with_profile(ProfileMode::Summary { timing: false }),
                |access| queues.claim(ClaimSelection::Generic, &access.into_host_access()),
            )
            .unwrap();
        let batch = outcome.result.unwrap();
        assert_eq!(batch.slots.len(), 2);
        assert_eq!(outcome.report.job_claim_work().unwrap(), total(2));
        assert_eq!(factories.get(), 0);
        assert!(log.borrow().is_empty());
        let unit = size_of::<Rc<Slot>>() as u64;
        let dimension = crate::execution::ExecutionWork::JobClaimWork;
        let profile = outcome.report.profile().unwrap().unwrap();
        assert_eq!(
            profile
                .scopes
                .iter()
                .find(|scope| scope.name == "claim-capacity")
                .unwrap()
                .usage
                .work(dimension),
            2 * unit
        );
        assert_eq!(
            profile
                .scopes
                .iter()
                .find(|scope| scope.name == "claim-relocate")
                .unwrap()
                .usage
                .work(dimension),
            unit
        );
        drop(batch);
        owner
            .with_execution(options(100_000), |mut access| access.run_jobs())
            .unwrap()
            .result
            .unwrap();
        assert_eq!(factories.get(), unrelated);
        assert_eq!(*log.borrow(), [0, 1]);
    }
}

#[test]
fn reentrant_claims_inspect_claimed_nodes_without_allocating_or_stealing_them() {
    let (mut owner, _, queues) = fixture();
    let log = Rc::new(RefCell::new(Vec::new()));
    for id in 0..2 {
        owner
            .enqueue_job(callback(id, &log, owner.realm().clone()).into())
            .unwrap()
            .into_result()
            .unwrap();
    }
    let held = owner
        .with_execution(options(total(2)), |access| {
            queues.claim(ClaimSelection::Generic, &access.into_host_access())
        })
        .unwrap()
        .result
        .unwrap();
    let skipped = owner
        .with_execution(options(3), |access| {
            queues.claim(ClaimSelection::Generic, &access.into_host_access())
        })
        .unwrap();
    assert!(skipped.result.unwrap().slots.is_empty());
    assert_eq!(skipped.report.job_claim_work().unwrap(), 3); // head + two claimed nodes
    assert!(!skipped.report.effects_started());
    owner
        .enqueue_job(callback(2, &log, owner.realm().clone()).into())
        .unwrap()
        .into_result()
        .unwrap();
    let unit = size_of::<Rc<Slot>>() as u64;
    let extra = owner
        .with_execution(options(unit + 6), |access| {
            queues.claim(ClaimSelection::Generic, &access.into_host_access())
        })
        .unwrap();
    let added = extra.result.unwrap();
    // Head1 + three linked nodes + first scratch capacity + copy1 + commit1.
    assert_eq!(extra.report.job_claim_work().unwrap(), unit + 6);
    assert_eq!(added.slots.len(), 1);
    assert!(
        held.slots
            .iter()
            .all(|slot| slot.status.get() == Status::Claimed)
    );
    assert!(
        added
            .slots
            .iter()
            .all(|slot| slot.status.get() == Status::Claimed)
    );
    drop(added);
    drop(held);
    assert_available(&slots(&queues, Kind::Ordinary(Ordinary::Generic)));
    owner
        .with_execution(options(100_000), |mut access| access.run_jobs())
        .unwrap()
        .result
        .unwrap();
    assert_eq!(*log.borrow(), [0, 1, 2]);
}

#[test]
fn clocks_select_only_existing_heads_strictly_before_the_cutoff_in_deadline_order() {
    let (mut owner, _, queues) = fixture();
    let log = Rc::new(RefCell::new(Vec::new()));
    for (id, secs) in [(3, 3), (1, 1), (2, 2)] {
        let timeout_log = log.clone();
        let job = TimeoutJob::with_realm(
            move |_| {
                timeout_log.borrow_mut().push(id);
                Ok(JsValue::undefined())
            },
            owner.realm().clone(),
            std::time::Duration::from_secs(secs),
        );
        owner
            .enqueue_job(job.into())
            .unwrap()
            .into_result()
            .unwrap();
    }
    let none = owner
        .with_execution(options(0), |access| {
            queues.claim(
                ClaimSelection::ClocksBefore(JsInstant::new(1, 0)),
                &access.into_host_access(),
            )
        })
        .unwrap();
    assert!(none.result.unwrap().slots.is_empty());
    assert_eq!(none.report.job_claim_work().unwrap(), 0);
    let before_two = owner
        .with_execution(options(total(1)), |access| {
            queues.claim(
                ClaimSelection::ClocksBefore(JsInstant::new(2, 0)),
                &access.into_host_access(),
            )
        })
        .unwrap();
    let first = before_two.result.unwrap();
    assert_eq!(first.slots.len(), 1);
    assert!(
        slots(&queues, Kind::Clock(JsInstant::new(1, 0)))
            .iter()
            .all(|slot| slot.status.get() == Status::Claimed)
    );
    assert_available(&slots(&queues, Kind::Clock(JsInstant::new(2, 0))));
    assert_available(&slots(&queues, Kind::Clock(JsInstant::new(3, 0))));
    drop(first);
    let unit = size_of::<Rc<Slot>>() as u64;
    // Two bucket heads + two nodes/copies + capacities 1+1 entries,
    // relocation of one initialized entry + atomic two-mark commit.
    let exact = 8 + 3 * unit;
    let before_three = owner
        .with_execution(options(exact), |access| {
            queues.claim(
                ClaimSelection::ClocksBefore(JsInstant::new(3, 0)),
                &access.into_host_access(),
            )
        })
        .unwrap();
    assert_eq!(before_three.report.job_claim_work().unwrap(), exact);
    let selected = before_three.result.unwrap();
    assert_eq!(selected.slots.len(), 2);
    // Consume real selected wrappers; their order is the deadline contract,
    // not insertion order or private slot numbering.
    for slot in &selected.slots {
        let mut payload = Payload::take(slot).unwrap().unwrap();
        let Some(Resident::Timeout(job)) = payload.command_slot().unwrap().take() else {
            panic!("authored timeout")
        };
        match job.call(owner.access()) {
            JobCall::Complete(result) => {
                result.unwrap();
                queues.finish(slot).unwrap();
            }
            JobCall::Deferred { .. } => panic!("the ordinary consumer has no selected refusal"),
        }
    }
    assert_eq!(*log.borrow(), [1, 2]);
    assert_available(&slots(&queues, Kind::Clock(JsInstant::new(3, 0))));
    drop(selected);
}

#[test]
fn a_linked_retired_node_costs_inspection_but_no_candidate_storage_or_claim_mark() {
    let (mut owner, _, queues) = fixture();
    let log = Rc::new(RefCell::new(Vec::new()));
    owner
        .enqueue_job(callback(0, &log, owner.realm().clone()).into())
        .unwrap()
        .into_result()
        .unwrap();
    let selected = slots(&queues, Kind::Ordinary(Ordinary::Generic));
    selected[0].retire(JobRetirement::Cancelled);
    let outcome = owner
        .with_execution(options(2), |access| {
            queues.claim(ClaimSelection::Generic, &access.into_host_access())
        })
        .unwrap();
    assert!(outcome.result.unwrap().slots.is_empty());
    assert_eq!(outcome.report.job_claim_work().unwrap(), 2);
    assert!(!outcome.report.effects_started());
    assert!(log.borrow().is_empty());
    assert_eq!(
        selected[0].observer.as_ref().unwrap().status().unwrap(),
        JobStatus::Retired(JobRetirement::Cancelled)
    );
    queues.finish(&selected[0]).unwrap();
}
