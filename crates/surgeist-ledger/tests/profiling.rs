use surgeist_ledger::{
    Change, Event, EventFilter, Limits, Operation, ProfileMode, Tariff, Tariffs,
};

surgeist_ledger::dimensions! {
    enum Work => WorkTable { Visit }
}

surgeist_ledger::dimensions! {
    enum Resource => ResourceTable { Bytes }
}

const WORK: Work = Work::Visit;
const BYTES: Resource = Resource::Bytes;

fn limits(work: u64, bytes: u64) -> Limits<Work, Resource> {
    Limits::new(WorkTable { Visit: work }, ResourceTable { Bytes: bytes })
}

fn tariffs(work_scale: u64, bytes_scale: u64) -> Tariffs<Work, Resource> {
    Tariffs::new(
        WorkTable {
            Visit: Tariff::new(WORK, work_scale),
        },
        ResourceTable {
            Bytes: Tariff::new(BYTES, bytes_scale),
        },
    )
}

#[test]
fn profiling_modes_keep_outcomes_and_accounting_identical() {
    fn run(mode: ProfileMode) -> (u64, u64, Option<surgeist_ledger::Report<Work, Resource>>) {
        let mut ledger = Operation::with_profile("html", limits(7, 8), 2, mode).unwrap();
        let root = ledger.root();
        let child = ledger
            .child(root, "css", limits(4, 4), tariffs(2, 2))
            .unwrap();
        ledger.apply(root, Change::Work(WORK, 1)).unwrap();
        ledger
            .with_phase(root, |ledger| {
                ledger
                    .with_phase(child, |ledger| {
                        ledger
                            .apply_batch(child, &[Change::Work(WORK, 3), Change::Acquire(BYTES, 4)])
                            .unwrap();
                        let result = ledger.apply_batch(
                            child,
                            &[Change::Release(BYTES, 2), Change::Work(WORK, 1)],
                        );
                        let Err(surgeist_ledger::Error::Refused(refusal)) = result else {
                            panic!("expected root refusal");
                        };
                        assert_eq!(refusal.scope, root);
                        assert_eq!(refusal.item, 1);
                        assert_eq!(ledger.usage(child).unwrap().live(BYTES), 4);
                        // Model a consumer stop: cleanup runs and no new source work is submitted.
                        ledger.apply(child, Change::Release(BYTES, 4)).unwrap();
                    })
                    .unwrap();
            })
            .unwrap();

        let usage = *ledger.usage(root).unwrap();
        assert_eq!(usage.live(BYTES), 0);
        (
            usage.work(WORK),
            usage.peak(BYTES),
            ledger.finish().unwrap(),
        )
    }

    let (work, peak, off) = run(ProfileMode::Off);
    assert_eq!((work, peak), (7, 8));
    assert!(off.is_none());

    for mode in [
        ProfileMode::Summary { timing: false },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 2,
            filter: EventFilter::All,
        },
    ] {
        let (work, peak, report) = run(mode);
        assert_eq!((work, peak), (7, 8));
        let report = report.unwrap();
        assert_eq!(report.elapsed, None);
        assert_eq!(report.scopes[0].usage.work(WORK), 7);
        assert_eq!(report.scopes[0].accepted_items, 1);
        assert_eq!(report.scopes[0].phases, 1);
        let child = &report.scopes[1];
        assert_eq!(child.name, "css");
        assert_eq!(child.parent, Some(report.scopes[0].scope));
        assert_eq!(child.usage.work(WORK), 3);
        assert_eq!(child.usage.live(BYTES), 0);
        assert_eq!(child.accepted_batches, 2);
        assert_eq!(child.accepted_items, 3);
        assert_eq!(child.refused_batches, 1);
        assert_eq!(child.first_refusal.unwrap().item, 1);
        assert_eq!(child.phase_elapsed, None);
        if matches!(mode, ProfileMode::Detailed { .. }) {
            assert_eq!(report.events.len(), 2);
            assert_eq!(report.dropped_events, 5);
        } else {
            assert!(report.events.is_empty());
            assert_eq!(report.dropped_events, 0);
        }
        let terminal = report.terminal().to_string();
        assert!(terminal.contains("css"));
        assert!(terminal.contains("refused=1"));
    }
}

#[test]
fn detailed_filter_keeps_selected_events_and_captures_refusal_context() {
    let mut ledger = Operation::with_profile(
        "root",
        limits(3, 0),
        2,
        ProfileMode::Detailed {
            timing: false,
            event_limit: 2,
            filter: EventFilter::ScopeName("css"),
        },
    )
    .unwrap();
    let root = ledger.root();
    let css = ledger
        .child(root, "css", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    ledger.apply(root, Change::Work(WORK, 2)).unwrap();
    ledger.apply(css, Change::Work(WORK, 1)).unwrap();
    assert!(
        ledger
            .apply_batch(css, &[Change::Work(WORK, 0), Change::Work(WORK, 1)])
            .is_err()
    );
    let report = ledger.finish().unwrap().unwrap();

    assert_eq!(report.dropped_events, 0);
    assert_eq!(report.events.len(), 2);
    assert!(
        matches!(report.events[0], Event::Accepted { scope, change: Change::Work(_, 1), elapsed: None } if scope == css)
    );
    assert!(
        matches!(report.events[1], Event::Refused { origin, refusal, elapsed: None } if origin == css && refusal.scope == root && refusal.item == 1)
    );
}
