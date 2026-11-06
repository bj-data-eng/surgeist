use std::{hint::black_box, mem::size_of, time::Instant};

use surgeist_ledger::{
    Change, Event, EventFilter, Limits, Operation, ProfileMode, Resource, ScopeReport, Tariffs,
    Work,
};

const PHASES: usize = 1_000;
const BATCHES: usize = 32;
const ROUNDS: usize = 11;
const WORK: Work<1> = Work::new(0).unwrap();
const BYTES: Resource<1> = Resource::new(0).unwrap();

fn measure(mode: ProfileMode) -> (f64, usize, u64) {
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
        operation
            .with_phase(child, |operation| {
                for _ in 0..BATCHES {
                    operation.apply_batch(child, black_box(&changes)).unwrap();
                }
            })
            .unwrap();
    }
    let elapsed = start.elapsed();
    let usage = operation.usage(root).unwrap();
    assert_eq!(usage.work(WORK), (PHASES * BATCHES) as u64);
    assert_eq!(usage.live(BYTES), 0);
    assert_eq!(usage.peak(BYTES), 8);
    let (bytes, dropped) = operation.finish().map_or((0, 0), |report| {
        let bytes = report.scopes.capacity() * size_of::<ScopeReport<1, 1>>()
            + report.events.capacity() * size_of::<Event<1, 1>>();
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
    let mut samples = [[0.0; 6]; ROUNDS];
    let mut storage = [(0, 0); 6];
    for (round, sample) in samples.iter_mut().enumerate() {
        for offset in 0..cases.len() {
            let index = (offset + round) % cases.len();
            let (elapsed, bytes, dropped) = measure(cases[index].1);
            sample[index] = elapsed;
            storage[index] = (bytes, dropped);
        }
    }

    println!("mode\tmedian ns/batch\tmin ns/batch\tmax ns/batch\tobserver capacity bytes\tdropped");
    for (index, ((name, _), (bytes, dropped))) in cases.into_iter().zip(storage).enumerate() {
        let mut times: [f64; ROUNDS] = std::array::from_fn(|round| samples[round][index]);
        times.sort_by(f64::total_cmp);
        println!(
            "{name}\t{:.2}\t{:.2}\t{:.2}\t{bytes}\t{dropped}",
            times[ROUNDS / 2],
            times[0],
            times[ROUNDS - 1]
        );
    }
}
