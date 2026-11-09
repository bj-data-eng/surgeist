use std::{hint::black_box, mem::size_of, time::Instant};

use surgeist_ledger::{
    Change, Event, EventFilter, Limits, Operation, ProfileMode, ScopeReport, Tariffs,
};

surgeist_ledger::dimensions! {
    enum Work => WorkTable { Visit }
}

surgeist_ledger::dimensions! {
    enum Resource => ResourceTable { Bytes }
}

const PHASES: usize = 1_000;
const BATCHES: usize = 32;
const ROUNDS: usize = 11;
const WORK: Work = Work::Visit;
const BYTES: Resource = Resource::Bytes;

#[derive(Clone, Copy)]
enum Style {
    Borrowed,
    Detached,
}

impl Style {
    const fn name(self) -> &'static str {
        match self {
            Self::Borrowed => "borrowed",
            Self::Detached => "detached",
        }
    }
}

fn measure(style: Style, mode: ProfileMode) -> (f64, usize, u64) {
    let mut operation = Operation::with_profile("root", Limits::unlimited(), 2, mode).unwrap();
    let root = operation.root();
    let child = operation
        .child(root, "child", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    let changes = [
        Change::Work(WORK, 1),
        Change::Acquire(BYTES, 8),
        Change::Release(BYTES, 8),
    ];
    let start = Instant::now();
    for _ in 0..PHASES {
        match style {
            Style::Borrowed => operation
                .with_phase(child, |operation| {
                    for _ in 0..BATCHES {
                        operation.apply_batch(child, black_box(&changes)).unwrap();
                    }
                })
                .unwrap(),
            Style::Detached => {
                let observation = operation.begin_phase(child).unwrap();
                for _ in 0..BATCHES {
                    operation.apply_batch(child, black_box(&changes)).unwrap();
                }
                operation.end_phase(observation).unwrap();
            }
        }
    }
    let elapsed = start.elapsed();
    let usage = operation.usage(root).unwrap();
    assert_eq!(usage.work(WORK), (PHASES * BATCHES) as u64);
    assert_eq!(usage.live(BYTES), 0);
    assert_eq!(usage.peak(BYTES), 8);
    let (bytes, dropped) = operation.finish().unwrap().map_or((0, 0), |report| {
        assert_eq!(report.scopes[1].phases, PHASES as u64);
        assert_eq!(report.scopes[1].accepted_batches, (PHASES * BATCHES) as u64);
        assert_eq!(
            report.scopes[1].accepted_items,
            (PHASES * BATCHES * 3) as u64
        );
        assert_eq!(report.scopes[1].refused_batches, 0);
        if let ProfileMode::Detailed { event_limit, .. } = mode {
            let events = PHASES * (BATCHES * 3 + 1);
            assert_eq!(report.events.len(), events.min(event_limit));
            assert_eq!(
                report.dropped_events,
                events.saturating_sub(event_limit) as u64
            );
        } else {
            assert!(report.events.is_empty());
            assert_eq!(report.dropped_events, 0);
        }
        let bytes = report.scopes.capacity() * size_of::<ScopeReport<Work, Resource>>()
            + report.events.capacity() * size_of::<Event<Work, Resource>>();
        (bytes, report.dropped_events)
    });
    (
        elapsed.as_secs_f64() * 1e9 / (PHASES * BATCHES) as f64,
        bytes,
        dropped,
    )
}

fn main() {
    let cases = [
        ("off", ProfileMode::Off),
        ("summary", ProfileMode::Summary { timing: false }),
        ("summary timed", ProfileMode::Summary { timing: true }),
        (
            "detail",
            ProfileMode::Detailed {
                timing: false,
                event_limit: PHASES * (BATCHES * 3 + 1),
                filter: EventFilter::All,
            },
        ),
        (
            "detail timed",
            ProfileMode::Detailed {
                timing: true,
                event_limit: PHASES * (BATCHES * 3 + 1),
                filter: EventFilter::All,
            },
        ),
        (
            "detail capped",
            ProfileMode::Detailed {
                timing: true,
                event_limit: 1024,
                filter: EventFilter::All,
            },
        ),
    ];
    let styles = [Style::Borrowed, Style::Detached];
    let mut samples = [[0.0; 12]; ROUNDS];
    let mut storage = [(0, 0); 12];
    for (round, sample) in samples.iter_mut().enumerate() {
        // Rotate all matched style/mode cases, including which style runs first.
        for offset in 0..cases.len() * styles.len() {
            let index = (offset + round) % (cases.len() * styles.len());
            let (elapsed, bytes, dropped) =
                measure(styles[index % styles.len()], cases[index / styles.len()].1);
            sample[index] = elapsed;
            storage[index] = (bytes, dropped);
        }
    }

    println!(
        "style\tmode\tmedian ns/batch\tmin ns/batch\tmax ns/batch\tobserver capacity bytes\tdropped"
    );
    for (index, (bytes, dropped)) in storage.into_iter().enumerate() {
        let style = styles[index % styles.len()].name();
        let name = cases[index / styles.len()].0;
        let mut times: [f64; ROUNDS] = std::array::from_fn(|round| samples[round][index]);
        times.sort_by(f64::total_cmp);
        println!(
            "{style}\t{name}\t{:.2}\t{:.2}\t{:.2}\t{bytes}\t{dropped}",
            times[ROUNDS / 2],
            times[0],
            times[ROUNDS - 1]
        );
    }
}
