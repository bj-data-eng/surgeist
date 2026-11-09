//! Release-mode admission cost for dimension sets of different widths.
//!
//! Each case applies one small batch to the root alone, to a scope below a
//! capped root, or to the leaf of a four-scope chain that has the shape of the
//! HTML topology (host, cap, charging and attribution scopes). The narrow set has the
//! width of a parser's accounting, the wide set the width of a combined
//! schema. Run with
//! `cargo bench --offline --locked -j 1 -p surgeist-ledger --bench admission`.

use std::{hint::black_box, time::Instant};

use surgeist_ledger::{Change, Limits, Operation, Tariff, Tariffs};

surgeist_ledger::dimensions! {
    enum NarrowWork => NarrowWorkTable {
        Read, Inspect, Copy, Step, Visit, Write, Reserve, Call, Total,
    }
}

surgeist_ledger::dimensions! {
    enum NarrowResource => NarrowResourceTable {
        R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18,
    }
}

surgeist_ledger::dimensions! {
    enum WideWork => WideWorkTable {
        Read, Inspect, Copy, Step, Visit, Write, Reserve, Call, Total, A, B, C,
    }
}

surgeist_ledger::dimensions! {
    enum WideResource => WideResourceTable {
        R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16, R17, R18,
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18,
        S19, S20, S21, S22, S23, S24, S25, S26, S27, S28,
    }
}

const BATCHES: u64 = 2_000_000;
const ROUNDS: usize = 11;

macro_rules! case {
    ($results:ident, $name:literal, $work:ident, $resource:ident, children: $children:expr, batch: $batch:expr) => {{
        let mut times = [0.0_f64; ROUNDS];
        for time in &mut times {
            let mut operation =
                Operation::<$work, $resource>::new("root", Limits::unlimited(), 4).unwrap();
            let root = operation.root();
            let mut scope = root;
            for () in std::iter::repeat_n((), $children) {
                scope = operation
                    .child(
                        scope,
                        "child",
                        Limits::unlimited(),
                        Tariffs::from_fn(
                            |_| Tariff::new($work::Total, 1),
                            |dimension| Tariff::new(dimension, 1),
                        ),
                    )
                    .unwrap();
            }
            let batch = $batch;
            let start = Instant::now();
            for _ in 0..BATCHES {
                operation.apply_batch(scope, black_box(&batch)).unwrap();
            }
            *time = start.elapsed().as_secs_f64() * 1e9 / BATCHES as f64;
            black_box(operation.usage(root).unwrap());
        }
        times.sort_by(f64::total_cmp);
        $results.push(($name, times[ROUNDS / 2], times[0], times[ROUNDS - 1]));
    }};
}

fn main() {
    let mut results: Vec<(&str, f64, f64, f64)> = Vec::new();
    case!(results, "narrow, one work change, root", NarrowWork, NarrowResource,
        children: 0, batch: [Change::Work(NarrowWork::Inspect, 2)]);
    case!(results, "narrow, one work change, child and root", NarrowWork, NarrowResource,
        children: 1, batch: [Change::Work(NarrowWork::Inspect, 2)]);
    case!(results, "narrow, acquire and release, child and root", NarrowWork, NarrowResource,
        children: 1, batch: [Change::Acquire(NarrowResource::R4, 8), Change::Release(NarrowResource::R4, 8)]);
    case!(results, "narrow, five work changes, child and root", NarrowWork, NarrowResource,
    children: 1, batch: [
        Change::Work(NarrowWork::Visit, 1),
        Change::Work(NarrowWork::Inspect, 2),
        Change::Work(NarrowWork::Copy, 3),
        Change::Work(NarrowWork::Call, 1),
        Change::Work(NarrowWork::Reserve, 8),
    ]);
    case!(results, "wide, one work change, root", WideWork, WideResource,
        children: 0, batch: [Change::Work(WideWork::Inspect, 2)]);
    case!(results, "wide, one work change, child and root", WideWork, WideResource,
        children: 1, batch: [Change::Work(WideWork::Inspect, 2)]);
    case!(results, "wide, acquire and release, child and root", WideWork, WideResource,
        children: 1, batch: [Change::Acquire(WideResource::R4, 8), Change::Release(WideResource::R4, 8)]);
    case!(results, "wide, five work changes, child and root", WideWork, WideResource,
    children: 1, batch: [
        Change::Work(WideWork::Visit, 1),
        Change::Work(WideWork::Inspect, 2),
        Change::Work(WideWork::Copy, 3),
        Change::Work(WideWork::Call, 1),
        Change::Work(WideWork::Reserve, 8),
    ]);
    case!(results, "narrow, five work changes, four scopes", NarrowWork, NarrowResource,
    children: 3, batch: [
        Change::Work(NarrowWork::Visit, 1),
        Change::Work(NarrowWork::Inspect, 2),
        Change::Work(NarrowWork::Copy, 3),
        Change::Work(NarrowWork::Call, 1),
        Change::Work(NarrowWork::Reserve, 8),
    ]);
    case!(results, "wide, one work change, four scopes", WideWork, WideResource,
        children: 3, batch: [Change::Work(WideWork::Inspect, 2)]);
    case!(results, "wide, five work changes, four scopes", WideWork, WideResource,
    children: 3, batch: [
        Change::Work(WideWork::Visit, 1),
        Change::Work(WideWork::Inspect, 2),
        Change::Work(WideWork::Copy, 3),
        Change::Work(WideWork::Call, 1),
        Change::Work(WideWork::Reserve, 8),
    ]);
    case!(results, "wide, ten work changes, four scopes", WideWork, WideResource,
    children: 3, batch: [
        Change::Work(WideWork::Read, 1),
        Change::Work(WideWork::Inspect, 2),
        Change::Work(WideWork::Copy, 3),
        Change::Work(WideWork::Step, 1),
        Change::Work(WideWork::Visit, 1),
        Change::Work(WideWork::Write, 1),
        Change::Work(WideWork::Reserve, 8),
        Change::Work(WideWork::Call, 1),
        Change::Work(WideWork::Inspect, 2),
        Change::Work(WideWork::Copy, 3),
    ]);
    case!(results, "wide, mixed five changes, four scopes", WideWork, WideResource,
    children: 3, batch: [
        Change::Work(WideWork::Visit, 1),
        Change::Acquire(WideResource::R4, 8),
        Change::Work(WideWork::Copy, 3),
        Change::Release(WideResource::R4, 8),
        Change::Work(WideWork::Reserve, 8),
    ]);
    println!("{BATCHES} batches per round, {ROUNDS} rounds");
    println!("case\tmedian ns/batch\tmin ns/batch\tmax ns/batch");
    for (name, median, min, max) in results {
        println!("{name}\t{median:.2}\t{min:.2}\t{max:.2}");
    }
}
