//! Release-mode cost of charging at different granularities on a wide schema.
//!
//! The pair has 20 work and 3 resource dimensions, the width of one schema
//! shared by a group of crates. The sections measure what a consumer pays for
//! each way of reaching the ledger:
//!
//! - one batch of one to eight work changes on an owned and on a shared
//!   operation, two and three scopes from the root, in three profile modes;
//! - one phase, detached and through the callback helper;
//! - building and dropping an operation, alone and with eight children;
//! - recording a gauge;
//! - one and four threads charging sibling scopes of one shared operation;
//! - a loop of events that keeps a local limit check and charges the ledger per
//!   event, per block of events, or once after the loop, beside the same loop
//!   with no ledger and with plain counters that apply the two-scope work rule.
//!
//! A batch of one work change with profiling off takes the journal-free path of
//! `Operation::apply_batch`; every other batch and mode takes the general path.
//!
//! Run with
//! `cargo bench --offline --locked -j 1 -p surgeist-ledger --bench granularity`.

use std::{
    hint::black_box,
    sync::{Arc, Barrier},
    thread,
    time::Instant,
};

use surgeist_ledger::{
    Change, Charge, Dimension, Limits, Operation, ProfileMode, Scope, SharedOperation, Tariffs,
};

surgeist_ledger::dimensions! {
    enum Work => WorkTable, WorkFields {
        W0, W1, W2, W3, W4, W5, W6, W7, W8, W9,
        W10, W11, W12, W13, W14, W15, W16, W17, W18, W19,
    }
}

surgeist_ledger::dimensions! {
    enum Resource => ResourceTable, ResourceFields { R0, R1, R2 }
}

type Batch = Change<Work, Resource>;

const ROUNDS: usize = 11;
const BATCHES: u64 = 400_000;
const PHASES: u64 = 400_000;
const SETUPS: u64 = 50_000;
const EVENTS: u64 = 1 << 20;
const THREADS: u64 = 4;
/// Scopes of the setup cases: a root and eight children.
const SETUP_SCOPES: usize = 9;

const MODES: [(&str, ProfileMode); 3] = [
    ("off", ProfileMode::Off),
    ("summary", ProfileMode::Summary { timing: false }),
    ("summary timed", ProfileMode::Summary { timing: true }),
];

/// The first `count` work dimensions, one unit each.
fn changes(count: usize) -> Vec<Batch> {
    Work::ALL
        .iter()
        .take(count)
        .map(|&dimension| Change::Work(dimension, 1))
        .collect()
}

/// An operation and its leaf, `chain` scopes from the root inclusive.
fn chain(mode: ProfileMode, chain: usize) -> (Operation<Work, Resource>, Scope) {
    let mut operation =
        Operation::with_reserved_profile("root", Limits::unlimited(), chain, mode).unwrap();
    let mut scope = operation.root();
    for _ in 1..chain {
        scope = operation
            .child(scope, "child", Limits::unlimited(), Tariffs::identity())
            .unwrap();
    }
    (operation, scope)
}

/// What the cases need of an owned and of a shared operation alike.
trait Holder {
    fn apply(&mut self, scope: Scope, batch: &[Batch]);
    fn detached_phase(&mut self, scope: Scope);
    fn callback_phase(&mut self, scope: Scope);
    fn record(&mut self, scope: Scope, value: u64);
    fn work(&self, scope: Scope, dimension: Work) -> u64;
    fn peak(&self, scope: Scope) -> u64;
}

impl Holder for Operation<Work, Resource> {
    fn apply(&mut self, scope: Scope, batch: &[Batch]) {
        self.apply_batch(scope, batch).unwrap();
    }
    fn detached_phase(&mut self, scope: Scope) {
        let observation = self.begin_phase(scope).unwrap();
        self.end_phase(black_box(observation)).unwrap();
    }
    fn callback_phase(&mut self, scope: Scope) {
        self.with_phase(scope, |phase| {
            black_box(phase);
        })
        .unwrap();
    }
    fn record(&mut self, scope: Scope, value: u64) {
        self.charge_at(scope)
            .record_largest(Resource::R0, value)
            .unwrap();
    }
    fn work(&self, scope: Scope, dimension: Work) -> u64 {
        self.usage(scope).unwrap().work(dimension)
    }
    fn peak(&self, scope: Scope) -> u64 {
        self.usage(scope).unwrap().peak(Resource::R0)
    }
}

impl Holder for SharedOperation<Work, Resource> {
    fn apply(&mut self, scope: Scope, batch: &[Batch]) {
        self.apply_batch(scope, batch).unwrap();
    }
    fn detached_phase(&mut self, scope: Scope) {
        let observation = self.begin_phase(scope).unwrap();
        self.end_phase(black_box(observation)).unwrap();
    }
    fn callback_phase(&mut self, scope: Scope) {
        self.with_phase(scope, || {
            black_box(());
        })
        .unwrap();
    }
    fn record(&mut self, scope: Scope, value: u64) {
        self.charge_at(scope)
            .record_largest(Resource::R0, value)
            .unwrap();
    }
    fn work(&self, scope: Scope, dimension: Work) -> u64 {
        self.usage(scope).unwrap().work(dimension)
    }
    fn peak(&self, scope: Scope) -> u64 {
        self.usage(scope).unwrap().peak(Resource::R0)
    }
}

/// Nanoseconds per unit of `count` for one run of `body`.
fn timed(count: u64, body: impl FnOnce()) -> f64 {
    let start = Instant::now();
    body();
    start.elapsed().as_secs_f64() * 1e9 / count as f64
}

fn admission(mut holder: impl Holder, scope: Scope, size: usize) -> f64 {
    let batch = changes(size);
    let time = timed(BATCHES, || {
        for _ in 0..BATCHES {
            holder.apply(scope, black_box(&batch));
        }
    });
    for &dimension in Work::ALL.iter().take(size) {
        assert_eq!(holder.work(scope, dimension), BATCHES);
    }
    time
}

fn phases(mut holder: impl Holder, scope: Scope, detached: bool) -> f64 {
    timed(PHASES, || {
        for _ in 0..PHASES {
            if detached {
                holder.detached_phase(scope);
            } else {
                holder.callback_phase(scope);
            }
        }
    })
}

fn gauge(mut holder: impl Holder, scope: Scope) -> f64 {
    let time = timed(BATCHES, || {
        for value in 0..BATCHES {
            holder.record(scope, black_box(value));
        }
    });
    assert_eq!(holder.peak(scope), BATCHES - 1);
    time
}

/// Builds and drops an operation with `children` scopes below its root.
fn setup(mode: ProfileMode, children: usize, reserved: bool, share: bool) -> f64 {
    timed(SETUPS, || {
        for _ in 0..SETUPS {
            let scopes = children.saturating_add(1);
            let mut operation = if reserved {
                Operation::<Work, Resource>::with_reserved_profile(
                    "root",
                    Limits::unlimited(),
                    scopes,
                    mode,
                )
            } else {
                Operation::with_profile("root", Limits::unlimited(), scopes, mode)
            }
            .unwrap();
            let root = operation.root();
            for _ in 0..children {
                black_box(
                    operation
                        .child(root, "child", Limits::unlimited(), Tariffs::identity())
                        .unwrap(),
                );
            }
            if share {
                black_box(operation.share());
            } else {
                black_box(operation);
            }
        }
    })
}

/// `threads` participants of one shared operation, each charging its own
/// child of the root. Reports the mean over the participants of the time one
/// takes per batch.
fn contended(threads: u64) -> f64 {
    let operation = Operation::<Work, Resource>::with_reserved_scopes(
        "root",
        Limits::unlimited(),
        SETUP_SCOPES,
    )
    .unwrap()
    .share();
    let root = operation.root();
    let barrier = Arc::new(Barrier::new(threads as usize));
    let workers: Vec<_> = (0..threads)
        .map(|_| {
            let scope = operation
                .child(root, "child", Limits::unlimited(), Tariffs::identity())
                .unwrap();
            let operation = operation.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let batch = changes(3);
                barrier.wait();
                timed(BATCHES, || {
                    for _ in 0..BATCHES {
                        operation.apply_batch(scope, black_box(&batch)).unwrap();
                    }
                })
            })
        })
        .collect();
    let times: Vec<f64> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert_eq!(
        operation.usage(root).unwrap().work(Work::W0),
        BATCHES * threads
    );
    times.iter().sum::<f64>() / times.len() as f64
}

/// Plain counters for an uncapped scope below a capped root with an identity
/// tariff, applying the ledger's rule for one work change: the scope is checked
/// for overflow, then the root against its cap, and nothing changes unless
/// both accept.
struct Counters {
    scope: WorkTable<u64>,
    root: WorkTable<u64>,
    caps: WorkTable<u64>,
}

impl Counters {
    fn work(&mut self, dimension: Work, amount: u64) -> Result<(), u64> {
        let scope = Work::get(&self.scope, dimension)
            .checked_add(amount)
            .ok_or(u64::MAX)?;
        let root = Work::get(&self.root, dimension)
            .checked_add(amount)
            .ok_or(u64::MAX)?;
        let cap = *Work::get(&self.caps, dimension);
        if root > cap {
            return Err(cap);
        }
        *Work::get_mut(&mut self.scope, dimension) = scope;
        *Work::get_mut(&mut self.root, dimension) = root;
        Ok(())
    }
}

/// How a loop of events reaches the ledger.
#[derive(Clone, Copy)]
enum Granularity {
    /// The local limit check alone.
    Guard,
    /// The local check and plain two-scope counters per event.
    Counters,
    /// The local check and one ledger batch per `block` events, with the rest
    /// charged after the loop. A block of one is a charge per event.
    Block(u64),
    /// The local check and one ledger batch after the loop.
    Call,
}

/// Runs `EVENTS` events. Every event passes a local limit check, as a request
/// limit does; the ledger sees them at the given granularity.
fn events(mut holder: impl Holder, scope: Scope, granularity: Granularity) -> f64 {
    let limit = black_box(u64::MAX);
    let mut counters = Counters {
        scope: WorkTable::splat(0),
        root: WorkTable::splat(0),
        caps: WorkTable::splat(limit),
    };
    let mut local = 0_u64;
    let mut pending = 0_u64;
    let time = timed(EVENTS, || {
        for _ in 0..EVENTS {
            let amount = black_box(1_u64);
            local += amount;
            assert!(local <= limit, "local limit reached");
            match granularity {
                Granularity::Guard | Granularity::Call => {}
                Granularity::Counters => counters.work(Work::W0, amount).unwrap(),
                Granularity::Block(block) => {
                    pending += amount;
                    if pending == block {
                        holder.apply(scope, &[Change::Work(Work::W0, pending)]);
                        pending = 0;
                    }
                }
            }
        }
        match granularity {
            Granularity::Guard | Granularity::Counters => {}
            Granularity::Block(_) if pending == 0 => {}
            Granularity::Block(_) => holder.apply(scope, &[Change::Work(Work::W0, pending)]),
            Granularity::Call => holder.apply(scope, &[Change::Work(Work::W0, local)]),
        }
    });
    assert_eq!(local, EVENTS);
    match granularity {
        Granularity::Guard => {}
        Granularity::Counters => assert_eq!(counters.root[Work::W0], EVENTS),
        Granularity::Block(_) | Granularity::Call => {
            assert_eq!(holder.work(scope, Work::W0), EVENTS);
        }
    }
    black_box(counters.scope);
    time
}

struct Case {
    name: String,
    run: Box<dyn Fn() -> f64>,
}

fn case(name: impl Into<String>, run: impl Fn() -> f64 + 'static) -> Case {
    Case {
        name: name.into(),
        run: Box::new(run),
    }
}

/// Runs every case once per round, starting each round one case later, and
/// prints the median, minimum and maximum of each.
fn report(title: &str, unit: &str, cases: &[Case]) {
    let mut samples = vec![Vec::with_capacity(ROUNDS); cases.len()];
    for round in 0..ROUNDS {
        for offset in 0..cases.len() {
            let index = (offset + round) % cases.len();
            samples[index].push((cases[index].run)());
        }
    }
    println!("\n{title}");
    println!("case\tmedian {unit}\tmin {unit}\tmax {unit}");
    for (case, mut times) in cases.iter().zip(samples) {
        times.sort_by(f64::total_cmp);
        println!(
            "{}\t{:.2}\t{:.2}\t{:.2}",
            case.name,
            times[ROUNDS / 2],
            times[0],
            times[ROUNDS - 1]
        );
    }
}

fn admission_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for (mode_name, mode) in MODES {
        for scopes in [2, 3] {
            for size in [1, 2, 3, 5, 8] {
                cases.push(case(
                    format!("owned, {mode_name}, {scopes} scopes, {size} changes"),
                    move || {
                        let (operation, scope) = chain(mode, scopes);
                        admission(operation, scope, size)
                    },
                ));
                cases.push(case(
                    format!("shared, {mode_name}, {scopes} scopes, {size} changes"),
                    move || {
                        let (operation, scope) = chain(mode, scopes);
                        admission(operation.share(), scope, size)
                    },
                ));
            }
        }
    }
    cases
}

fn phase_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for (mode_name, mode) in MODES {
        for (style, detached) in [("detached", true), ("callback", false)] {
            cases.push(case(format!("owned, {mode_name}, {style}"), move || {
                let (operation, scope) = chain(mode, 2);
                phases(operation, scope, detached)
            }));
            cases.push(case(format!("shared, {mode_name}, {style}"), move || {
                let (operation, scope) = chain(mode, 2);
                phases(operation.share(), scope, detached)
            }));
        }
    }
    cases
}

fn setup_cases() -> Vec<Case> {
    let timed_summary = ProfileMode::Summary { timing: true };
    let children = SETUP_SCOPES - 1;
    vec![
        case("root alone, off", || {
            setup(ProfileMode::Off, 0, false, false)
        }),
        case("root alone, off, shared", || {
            setup(ProfileMode::Off, 0, false, true)
        }),
        case("root alone, summary timed", move || {
            setup(timed_summary, 0, false, false)
        }),
        case("eight children, off, grown", move || {
            setup(ProfileMode::Off, children, false, false)
        }),
        case("eight children, off, reserved", move || {
            setup(ProfileMode::Off, children, true, false)
        }),
        case("eight children, off, reserved, shared", move || {
            setup(ProfileMode::Off, children, true, true)
        }),
        case(
            "eight children, summary timed, reserved, shared",
            move || setup(timed_summary, children, true, true),
        ),
    ]
}

fn gauge_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for (mode_name, mode) in MODES {
        cases.push(case(format!("owned, {mode_name}"), move || {
            let (operation, scope) = chain(mode, 2);
            gauge(operation, scope)
        }));
        cases.push(case(format!("shared, {mode_name}"), move || {
            let (operation, scope) = chain(mode, 2);
            gauge(operation.share(), scope)
        }));
    }
    cases
}

fn event_cases() -> Vec<Case> {
    let off = ProfileMode::Off;
    let summary = ProfileMode::Summary { timing: false };
    let mut cases = vec![
        case("local check only", move || {
            let (operation, scope) = chain(off, 2);
            events(operation, scope, Granularity::Guard)
        }),
        case("plain two-scope counters per event", move || {
            let (operation, scope) = chain(off, 2);
            events(operation, scope, Granularity::Counters)
        }),
    ];
    for (mode_name, mode) in [("off", off), ("summary", summary)] {
        for (name, granularity) in [
            ("per event", Granularity::Block(1)),
            ("per 64 events", Granularity::Block(64)),
            ("per 4096 events", Granularity::Block(4096)),
            ("once after the loop", Granularity::Call),
        ] {
            cases.push(case(format!("owned, {mode_name}, {name}"), move || {
                let (operation, scope) = chain(mode, 2);
                events(operation, scope, granularity)
            }));
            cases.push(case(format!("shared, {mode_name}, {name}"), move || {
                let (operation, scope) = chain(mode, 2);
                events(operation.share(), scope, granularity)
            }));
        }
    }
    cases
}

fn main() {
    println!(
        "{} work and {} resource dimensions, {ROUNDS} rounds",
        Work::ALL.len(),
        Resource::ALL.len()
    );
    println!(
        "reserved backing of a shared operation with {SETUP_SCOPES} scopes: {} bytes",
        SharedOperation::<Work, Resource>::reserved_backing_bytes(SETUP_SCOPES).unwrap()
    );
    println!(
        "the same with a timed summary: {} bytes",
        SharedOperation::<Work, Resource>::profiled_backing_bytes(
            SETUP_SCOPES,
            ProfileMode::Summary { timing: true }
        )
        .unwrap()
    );
    report(
        &format!("admission, {BATCHES} batches per round"),
        "ns/batch",
        &admission_cases(),
    );
    report(
        &format!("phases, {PHASES} phases per round"),
        "ns/phase",
        &phase_cases(),
    );
    report(
        &format!("setup, {SETUPS} operations per round"),
        "ns/operation",
        &setup_cases(),
    );
    report(
        &format!("gauge, {BATCHES} records per round"),
        "ns/record",
        &gauge_cases(),
    );
    report(
        &format!("shared participants, three changes, {BATCHES} batches each"),
        "ns/batch",
        &[
            case("one thread", || contended(1)),
            case(format!("{THREADS} threads"), || contended(THREADS)),
        ],
    );
    report(
        &format!("event loop, {EVENTS} events per round"),
        "ns/event",
        &event_cases(),
    );
}
