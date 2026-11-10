//! Prototype for #2228 (scratch, not for merge): the exact `Meter` of the design
//! note, checked against the ledger it replaces on the hot path.
//!
//! The parse's accounting is two ledger scopes: a charging scope with unlimited
//! caps that converts each of the eight work categories to credits at rate one,
//! under a cap scope that carries every limit (`surgeist-html-parser`
//! `accounting::private`). `Meter` keeps the same state in plain counters.
//! `HostMeter` serves a shared host operation: work-only batches accumulate
//! locally and reach the ledger in blocks that a `check_batch` guard proved
//! admissible; every batch with a resource change reaches the ledger exactly.
//!
//! Property test: random limits and random batches (work, acquire, release;
//! all-or-nothing; provisional prefix; overflow; underflow) run on the ledger,
//! on `Meter` and on `HostMeter`; the outcome of every batch (accepted, or the
//! first refusal with item, change, current value, reason and refusing scope)
//! and the complete usage after every batch (work totals, credits, live and peak
//! of every resource) must be equal.
//!
//! Run: `cargo run --release --offline --locked -j 1 -p surgeist-html-parser --example model_meter`

use std::{hint::black_box, time::Instant};
use surgeist_html::accounting::{HtmlChange, HtmlLimits, HtmlResource, HtmlWork, charging_tariffs};
use surgeist_ledger::{
    Change, Dimension, Error, Operation, RefusalReason, Scope, SharedOperation,
};

const WORKS: usize = 8;
const RES: usize = HtmlResource::ALL.len();

fn work_dim(i: usize) -> HtmlWork {
    HtmlWork::ALL[i]
}

/// Refusal as the equality test sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Refused {
    item: usize,
    change: HtmlChange,
    current: u64,
    reason: RefusalReason,
    at_cap: bool,
}

/// The state of both scopes of a private parse operation.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Snapshot {
    work: [u64; WORKS],
    credits: u64,
    live: [u64; RES],
    peak: [u64; RES],
}

// ---- the ledger oracle ------------------------------------------------------

struct Oracle {
    op: Operation<HtmlWork, HtmlResource>,
    cap: Scope,
    charging: Scope,
}
impl Oracle {
    fn new(limits: HtmlLimits) -> Self {
        let mut op = Operation::new("parse", limits, 2).unwrap();
        let cap = op.root();
        let charging = op
            .child(cap, "charging", HtmlLimits::unlimited(), charging_tariffs())
            .unwrap();
        Self { op, cap, charging }
    }
    fn apply(&mut self, batch: &[HtmlChange]) -> Result<(), Refused> {
        match self.op.apply_batch(self.charging, batch) {
            Ok(()) => Ok(()),
            Err(Error::Refused(r)) => Err(Refused {
                item: r.item,
                change: r.change,
                current: r.current,
                reason: r.reason,
                at_cap: r.scope == self.cap,
            }),
            Err(other) => panic!("unexpected ledger error {other:?}"),
        }
    }
    fn snapshot(&self) -> Snapshot {
        let c = self.op.usage(self.charging).unwrap();
        let k = self.op.usage(self.cap).unwrap();
        let mut s = Snapshot {
            work: [0; WORKS],
            credits: k.work(HtmlWork::Credits),
            live: [0; RES],
            peak: [0; RES],
        };
        for i in 0..WORKS {
            s.work[i] = c.work(work_dim(i));
        }
        for (i, r) in HtmlResource::ALL.iter().enumerate() {
            s.live[i] = c.live(*r);
            s.peak[i] = c.peak(*r);
            assert_eq!(k.live(*r), c.live(*r));
            assert_eq!(k.peak(*r), c.peak(*r));
        }
        s
    }
}

// ---- Meter -------------------------------------------------------------------

struct Meter {
    work: [u64; WORKS],
    credits: u64,
    credit_cap: u64,
    live: [u64; RES],
    peak: [u64; RES],
    cap: [u64; RES],
    undo: Vec<Undo>,
}
#[derive(Clone, Copy)]
enum Undo {
    Work(usize, u64, u64),
    Live(usize, u64, u64),
}
impl Meter {
    fn new(limits: &HtmlLimits) -> Self {
        let mut cap = [0; RES];
        for (i, r) in HtmlResource::ALL.iter().enumerate() {
            cap[i] = limits.live(*r);
        }
        Self {
            work: [0; WORKS],
            credits: 0,
            credit_cap: limits.work(HtmlWork::Credits),
            live: [0; RES],
            peak: [0; RES],
            cap,
            undo: Vec::with_capacity(16),
        }
    }

    #[inline]
    fn one(&mut self, item: usize, change: HtmlChange) -> Result<(), Refused> {
        match change {
            Change::Work(d, n) => {
                let i = d as usize;
                debug_assert!(i < WORKS);
                let held = self.work[i];
                let next = held.checked_add(n).ok_or(Refused {
                    item,
                    change,
                    current: held,
                    reason: RefusalReason::Overflow,
                    at_cap: false,
                })?;
                let held_c = self.credits;
                let credit_change = Change::Work(HtmlWork::Credits, n);
                let next_c = held_c.checked_add(n).ok_or(Refused {
                    item,
                    change: credit_change,
                    current: held_c,
                    reason: RefusalReason::Overflow,
                    at_cap: true,
                })?;
                if next_c > self.credit_cap {
                    return Err(Refused {
                        item,
                        change: credit_change,
                        current: held_c,
                        reason: RefusalReason::Limit {
                            limit: self.credit_cap,
                        },
                        at_cap: true,
                    });
                }
                self.undo.push(Undo::Work(i, held, held_c));
                self.work[i] = next;
                self.credits = next_c;
            }
            Change::Acquire(r, n) => {
                let i = r as usize;
                let held = self.live[i];
                let live = held.checked_add(n).ok_or(Refused {
                    item,
                    change,
                    current: held,
                    reason: RefusalReason::Overflow,
                    at_cap: false,
                })?;
                if live > self.cap[i] {
                    return Err(Refused {
                        item,
                        change,
                        current: held,
                        reason: RefusalReason::Limit { limit: self.cap[i] },
                        at_cap: true,
                    });
                }
                self.undo.push(Undo::Live(i, held, self.peak[i]));
                self.live[i] = live;
                self.peak[i] = self.peak[i].max(live);
            }
            Change::Release(r, n) => {
                let i = r as usize;
                let held = self.live[i];
                let remaining = held.checked_sub(n).ok_or(Refused {
                    item,
                    change,
                    current: held,
                    reason: RefusalReason::Underflow,
                    at_cap: false,
                })?;
                self.undo.push(Undo::Live(i, held, self.peak[i]));
                self.live[i] = remaining;
            }
        }
        Ok(())
    }

    fn apply(&mut self, batch: &[HtmlChange]) -> Result<(), Refused> {
        self.undo.clear();
        for (item, &change) in batch.iter().enumerate() {
            if let Err(refused) = self.one(item, change) {
                while let Some(u) = self.undo.pop() {
                    match u {
                        Undo::Work(i, w, c) => {
                            self.work[i] = w;
                            self.credits = c;
                        }
                        Undo::Live(i, l, p) => {
                            self.live[i] = l;
                            self.peak[i] = p;
                        }
                    }
                }
                return Err(refused);
            }
        }
        Ok(())
    }
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            work: self.work,
            credits: self.credits,
            live: self.live,
            peak: self.peak,
        }
    }
}

// ---- HostMeter -----------------------------------------------------------------

const BLOCK: u64 = 1 << 16;

struct HostMeter {
    ledger: SharedOperation<HtmlWork, HtmlResource>,
    root: Scope,
    charging: Scope,
    pending: [u64; WORKS],
    pending_sum: u64,
    /// Credits that a guard has proved admissible beyond what the ledger holds.
    safe: u64,
    ledger_calls: u64,
}
impl HostMeter {
    fn new(limits: HtmlLimits) -> Self {
        let mut op = Operation::new("host", limits, 3).unwrap();
        let cap = op.root();
        let charging = op
            .child(cap, "charging", HtmlLimits::unlimited(), charging_tariffs())
            .unwrap();
        Self {
            ledger: op.share(),
            root: cap,
            charging,
            pending: [0; WORKS],
            pending_sum: 0,
            safe: 0,
            ledger_calls: 0,
        }
    }
    fn flush(&mut self) -> Result<(), Error<HtmlWork, HtmlResource>> {
        if self.pending_sum == 0 {
            return Ok(());
        }
        let mut batch = [Change::Work(work_dim(0), 0); WORKS];
        let mut n = 0;
        for i in 0..WORKS {
            if self.pending[i] != 0 {
                batch[n] = Change::Work(work_dim(i), self.pending[i]);
                n += 1;
            }
        }
        self.pending = [0; WORKS];
        self.safe = self.safe.saturating_sub(self.pending_sum);
        self.pending_sum = 0;
        self.ledger_calls += 1;
        self.ledger.apply_batch(self.charging, &batch[..n])
    }
    fn exact(&mut self, batch: &[HtmlChange]) -> Result<(), Refused> {
        self.flush().expect("a flush admitted by the guard is accepted");
        self.safe = 0;
        self.ledger_calls += 1;
        match self.ledger.apply_batch(self.charging, batch) {
            Ok(()) => Ok(()),
            Err(Error::Refused(r)) => Err(Refused {
                item: r.item,
                change: r.change,
                current: r.current,
                reason: r.reason,
                at_cap: r.scope != self.charging,
            }),
            Err(other) => panic!("unexpected ledger error {other:?}"),
        }
    }
    fn apply(&mut self, batch: &[HtmlChange]) -> Result<(), Refused> {
        // Work that a guard has proved admissible cannot be refused, and a resource
        // change does not read the credits, so the work part of a mixed batch can
        // wait while the resource part reaches the ledger exactly; the first
        // refusal is then the first refused resource item, as in the whole batch.
        let mut sum = 0_u64;
        let mut local = true;
        let mut resources = [Change::Work(work_dim(0), 0); 24];
        let mut index = [0_usize; 24];
        let mut count = 0;
        for (item, c) in batch.iter().enumerate() {
            match c {
                Change::Work(d, n) if (*d as usize) < WORKS && *n < (1 << 40) => sum += n,
                Change::Work(..) => local = false,
                _ => {
                    if count == 24 {
                        local = false;
                    } else {
                        resources[count] = *c;
                        index[count] = item;
                        count += 1;
                    }
                }
            }
        }
        if !local {
            return self.exact(batch);
        }
        if sum > 0 && self.pending_sum + sum > self.safe {
            // Re-prove headroom for one block after the pending amount is in the ledger.
            self.flush().expect("a flush admitted by the guard is accepted");
            self.ledger_calls += 1;
            let guard = [Change::Work(work_dim(0), BLOCK + sum)];
            self.safe = if self.ledger.check_batch(self.charging, &guard).is_ok() {
                BLOCK + sum
            } else {
                0
            };
            if sum > self.safe {
                return self.exact(batch);
            }
        }
        if count > 0 {
            self.ledger_calls += 1;
            match self.ledger.apply_batch(self.charging, &resources[..count]) {
                Ok(()) => {}
                Err(Error::Refused(r)) => {
                    return Err(Refused {
                        item: index[r.item],
                        change: r.change,
                        current: r.current,
                        reason: r.reason,
                        at_cap: r.scope != self.charging,
                    });
                }
                Err(other) => panic!("unexpected ledger error {other:?}"),
            }
        }
        for c in batch {
            if let Change::Work(d, n) = *c {
                self.pending[d as usize] += n;
            }
        }
        self.pending_sum += sum;
        Ok(())
    }
    fn settle(&mut self) {
        self.flush().expect("settle is admitted");
    }
    fn snapshot(&mut self) -> Snapshot {
        self.settle();
        let u = self.ledger.usage(self.charging).unwrap();
        let root = self.ledger.usage(self.root).unwrap();
        let mut s = Snapshot {
            work: [0; WORKS],
            credits: root.work(HtmlWork::Credits),
            live: [0; RES],
            peak: [0; RES],
        };
        for i in 0..WORKS {
            s.work[i] = u.work(work_dim(i));
        }
        for (i, r) in HtmlResource::ALL.iter().enumerate() {
            s.live[i] = u.live(*r);
            s.peak[i] = u.peak(*r);
        }
        s
    }
}

// ---- property test ----------------------------------------------------------------

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

const AMOUNTS: [u64; 12] = [
    0,
    1,
    1,
    2,
    3,
    7,
    20,
    100,
    1_000,
    70_000,
    1 << 41,
    u64::MAX - 5,
];
const CAPS: [u64; 10] = [0, 1, 2, 5, 30, 100, 1_000, 20_000, 1 << 30, u64::MAX];

fn random_limits(rng: &mut Rng) -> HtmlLimits {
    let mut caps = [0_u64; RES];
    for c in &mut caps {
        *c = CAPS[(rng.below(10) as usize).max(rng.below(10) as usize)];
    }
    let credit_cap = [50, 500, 5_000, 100_000, 10_000_000, u64::MAX][rng.below(6) as usize];
    HtmlLimits::from_fn(
        |w| if w == HtmlWork::Credits { credit_cap } else { u64::MAX },
        |r| caps[r as usize],
    )
}

fn random_batch(rng: &mut Rng, subset: &[usize; 6], out: &mut Vec<HtmlChange>) {
    out.clear();
    let len = if rng.below(50) == 0 { 10 + rng.below(20) } else { 1 + rng.below(5) };
    for _ in 0..len {
        let amount = AMOUNTS[(rng.below(12) as usize).min(rng.below(12) as usize)];
        let r = HtmlResource::ALL[subset[rng.below(6) as usize]];
        out.push(match rng.below(10) {
            0..=4 => Change::Work(work_dim(rng.below(WORKS as u64) as usize), amount.min(if rng.below(8) == 0 { u64::MAX } else { 5_000 })),
            5..=7 => Change::Acquire(r, amount),
            _ => Change::Release(r, amount),
        });
    }
}

#[derive(Default)]
struct Coverage {
    accepted: u64,
    limit_cap: u64,
    overflow_charging: u64,
    overflow_cap: u64,
    underflow: u64,
}

fn property_test() {
    let mut cov = Coverage::default();
    let mut total = 0_u64;
    let mut host_calls = 0_u64;
    let mut host_batches = 0_u64;
    let mut batch = Vec::new();
    for seed in 1..=3_000_u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let limits = random_limits(&mut rng);
        let mut subset = [0_usize; 6];
        for s in &mut subset {
            *s = rng.below(RES as u64) as usize;
        }
        let mut oracle = Oracle::new(limits);
        let mut meter = Meter::new(&limits);
        let mut host = HostMeter::new(limits);
        for step in 0..400 {
            random_batch(&mut rng, &subset, &mut batch);
            let want = oracle.apply(&batch);
            let got = meter.apply(&batch);
            let hosted = host.apply(&batch);
            total += 1;
            host_batches += 1;
            match &want {
                Ok(()) => cov.accepted += 1,
                Err(r) => match (r.reason, r.at_cap) {
                    (RefusalReason::Limit { .. }, _) => cov.limit_cap += 1,
                    (RefusalReason::Overflow, false) => cov.overflow_charging += 1,
                    (RefusalReason::Overflow, true) => cov.overflow_cap += 1,
                    (RefusalReason::Underflow, _) => cov.underflow += 1,
                },
            }
            assert_eq!(want, got, "meter outcome differs: seed {seed} step {step} batch {batch:?}");
            assert_eq!(want, hosted, "host outcome differs: seed {seed} step {step} batch {batch:?}");
            assert_eq!(oracle.snapshot(), meter.snapshot(), "meter usage differs: seed {seed} step {step}");
            if step % 13 == 12 {
                assert_eq!(oracle.snapshot(), host.snapshot(), "host usage differs: seed {seed} step {step}");
            }
        }
        assert_eq!(oracle.snapshot(), host.snapshot(), "host usage differs at the end: seed {seed}");
        host_calls += host.ledger_calls;
    }
    println!(
        "property test: {total} batches on ledger, Meter and HostMeter: all outcomes and usages equal"
    );
    println!(
        "  coverage: accepted {}, cap limit {}, overflow at charging scope {}, overflow at cap scope {}, underflow {}",
        cov.accepted, cov.limit_cap, cov.overflow_charging, cov.overflow_cap, cov.underflow
    );
    println!(
        "  host meter ledger calls: {host_calls} for {host_batches} batches (random mix; includes the 13-step settles)"
    );
}

// ---- cost ---------------------------------------------------------------------------

/// The batches of one table-heavy parse under the proposed tariff: per start tag,
/// text run and end tag a token batch (work only), and per created node an
/// action batch (resources and work).
fn stream() -> Vec<Vec<HtmlChange>> {
    use HtmlResource as R;
    use HtmlWork as W;
    let mut v = Vec::new();
    let token = |n: u64| {
        vec![
            Change::Work(W::InputUnitRead, n),
            Change::Work(W::InspectByte, 2 * n),
            Change::Work(W::StateStep, 1),
        ]
    };
    let node = |units: u64, attrs: u64| {
        let mut b = vec![
            Change::Acquire(R::ManagedNodes, 1),
            Change::Acquire(R::ManagedEdges, 1),
            Change::Acquire(R::ManagedUtf16Units, units),
        ];
        if attrs > 0 {
            b.push(Change::Acquire(R::ManagedAttributes, attrs));
        }
        b.push(Change::Work(W::CopyByte, 32 + 2 * units));
        b.push(Change::Work(W::RecordWrite, 1 + attrs));
        b
    };
    for _ in 0..16_658 {
        v.push(token(12));
        v.push(node(32, 0));
        v.push(token(6)); // end tag
    }
    for _ in 0..15_632 {
        v.push(token(4));
        v.push(node(4, 0));
    }
    v
}

fn bench() {
    let batches = stream();
    let limits = HtmlLimits::from_fn(|_| u64::MAX, |_| u64::MAX);
    let rounds = 41;
    let run = |f: &mut dyn FnMut()| {
        let mut t = Vec::new();
        for _ in 0..rounds {
            let s = Instant::now();
            f();
            t.push(s.elapsed().as_secs_f64() * 1e3);
        }
        t.sort_by(f64::total_cmp);
        t[rounds / 2]
    };
    let ledger = run(&mut || {
        let mut o = Oracle::new(limits);
        for b in black_box(&batches) {
            o.apply(b).unwrap();
        }
        black_box(o.snapshot());
    });
    let meter = run(&mut || {
        let mut m = Meter::new(&limits);
        for b in black_box(&batches) {
            m.apply(b).unwrap();
        }
        black_box(m.snapshot());
    });
    let mut calls = 0;
    let host = run(&mut || {
        let mut h = HostMeter::new(limits);
        for b in black_box(&batches) {
            h.apply(b).unwrap();
        }
        h.settle();
        calls = h.ledger_calls;
        black_box(h.snapshot());
    });
    println!();
    println!(
        "cost of one table-heavy parse's accounting ({} batches: 16,658 elements with start-tag, node and end-tag batches, 15,632 text runs with token and node batches), median of {rounds}:",
        batches.len()
    );
    println!("  {:<34} {:>9} {:>10}", "model", "ms", "ns/batch");
    for (name, ms) in [
        ("ledger apply_batch (main)", ledger),
        ("Meter (owned)", meter),
        ("HostMeter (shared operation)", host),
    ] {
        println!(
            "  {:<34} {:>9.3} {:>10.1}",
            name,
            ms,
            ms * 1e6 / batches.len() as f64
        );
    }
    println!("  HostMeter ledger calls per parse: {calls}");
}

fn main() {
    property_test();
    bench();
}
