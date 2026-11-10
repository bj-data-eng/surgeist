//! The meter against the ledger operation it replaces.
//!
//! The authority is `surgeist-ledger`: a parse's own operation of two scopes, a
//! charging scope without caps below a cap scope with the parse caps, admits a
//! batch and refuses it by the ledger's rules. The meter must give the same
//! outcome for every batch, and leave the same usage in both scopes.

use super::meter::{Meter, Reading};
use crate::accounting::{LedgerError, private};
use surgeist_html::accounting::{
    HtmlChange, HtmlLimits, HtmlResource, HtmlResourceTable, HtmlWork,
};
use surgeist_ledger::{Change, Dimension, Error, Operation, RefusalReason, Scope};

/// Where a refusal came from, in a form that two operations can share: their
/// scopes differ in identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum At {
    Cap,
    Charging,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Refused {
    at: At,
    item: usize,
    change: HtmlChange,
    current: u64,
    reason: RefusalReason,
}

fn outcome(result: Result<(), LedgerError>, cap: Scope, charging: Scope) -> Result<(), Refused> {
    match result {
        Ok(()) => Ok(()),
        Err(Error::Refused(refusal)) => Err(Refused {
            at: if refusal.scope == cap {
                At::Cap
            } else {
                assert_eq!(refusal.scope, charging, "a refusal names one of the scopes");
                At::Charging
            },
            item: refusal.item,
            change: refusal.change,
            current: refusal.current,
            reason: refusal.reason,
        }),
        Err(other) => panic!("a ledger error that is not a refusal: {other:?}"),
    }
}

/// The ledger operation and the meter of one set of caps.
struct Pair {
    operation: Operation<HtmlWork, HtmlResource>,
    cap: Scope,
    charging: Scope,
    meter: Meter,
}

impl Pair {
    fn new(limits: HtmlLimits) -> Self {
        let (operation, cap, charging) = private(limits).unwrap();
        let meter = Meter::with_limits(limits).unwrap();
        Self {
            operation,
            cap,
            charging,
            meter,
        }
    }

    fn apply(&mut self, batch: &[HtmlChange]) -> (Result<(), Refused>, Result<(), Refused>) {
        let ledger = self.operation.apply_batch(self.charging, batch);
        let meter = self.meter.apply(batch);
        (
            outcome(ledger, self.cap, self.charging),
            outcome(meter, self.meter.cap(), self.meter.charging()),
        )
    }

    fn check(&mut self, batch: &[HtmlChange]) -> (Result<(), Refused>, Result<(), Refused>) {
        let ledger = self.operation.check_batch(self.charging, batch);
        let meter = self.meter.check(batch);
        (
            outcome(ledger, self.cap, self.charging),
            outcome(meter, self.meter.cap(), self.meter.charging()),
        )
    }

    /// Asserts that both scopes hold the same work, live values and peaks in
    /// the operation and in the meter.
    fn assert_same_usage(&self, context: &str) {
        for (ledger, meter) in [
            (self.cap, self.meter.cap()),
            (self.charging, self.meter.charging()),
        ] {
            let usage = self.operation.usage(ledger).unwrap();
            let view = self.meter.view(meter).unwrap();
            for &dimension in HtmlWork::ALL {
                assert_eq!(
                    Reading::work(usage, dimension),
                    view.work(dimension),
                    "{context}: work {dimension:?}"
                );
            }
            for &dimension in HtmlResource::ALL {
                assert_eq!(
                    Reading::live(usage, dimension),
                    view.live(dimension),
                    "{context}: live {dimension:?}"
                );
                assert_eq!(
                    Reading::peak(usage, dimension),
                    view.peak(dimension),
                    "{context}: peak {dimension:?}"
                );
            }
        }
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(bound).unwrap()).unwrap()
    }
}

/// Amounts from nothing to nearly the range of a `u64`, mostly small.
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
const CREDIT_CAPS: [u64; 6] = [50, 500, 5_000, 100_000, 10_000_000, u64::MAX];

fn amount(rng: &mut Rng) -> u64 {
    AMOUNTS[rng.below(12).min(rng.below(12))]
}

fn random_limits(rng: &mut Rng) -> HtmlLimits {
    let credits = CREDIT_CAPS[rng.below(6)];
    let semantic = CREDIT_CAPS[rng.below(6)];
    let mut live = HtmlResourceTable::splat(0);
    for &resource in HtmlResource::ALL {
        *HtmlResource::get_mut(&mut live, resource) = CAPS[rng.below(10).max(rng.below(10))];
    }
    HtmlLimits::from_fn(
        |work| match work {
            HtmlWork::Credits => credits,
            HtmlWork::SemanticStep => semantic,
            _ => u64::MAX,
        },
        |resource| *HtmlResource::get(&live, resource),
    )
}

fn random_batch(rng: &mut Rng, resources: &[HtmlResource; 6], batch: &mut Vec<HtmlChange>) {
    batch.clear();
    let length = if rng.below(50) == 0 {
        10 + rng.below(20)
    } else {
        1 + rng.below(5)
    };
    for _ in 0..length {
        let resource = resources[rng.below(6)];
        let amount = amount(rng);
        batch.push(match rng.below(10) {
            0..=4 => {
                let work = HtmlWork::ALL[rng.below(HtmlWork::ALL.len())];
                let bounded = if rng.below(8) == 0 {
                    u64::MAX
                } else {
                    amount.min(5_000)
                };
                Change::Work(work, bounded)
            }
            5..=7 => Change::Acquire(resource, amount),
            _ => Change::Release(resource, amount),
        });
    }
}

/// Which kinds of outcome the random batches reached.
#[derive(Default)]
struct Coverage {
    accepted: u64,
    accepted_long: u64,
    work_limit: u64,
    resource_limit: u64,
    overflow_charging: u64,
    overflow_cap: u64,
    underflow: u64,
    refused_after_prefix: u64,
    peak_inside_batch: u64,
    checks_refused: u64,
}

#[test]
fn the_meter_admits_and_refuses_like_the_ledger_operation() {
    let mut coverage = Coverage::default();
    let mut batch = Vec::new();
    for seed in 1..=1_000_u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let limits = random_limits(&mut rng);
        let mut resources = [HtmlResource::ALL[0]; 6];
        for slot in &mut resources {
            *slot = HtmlResource::ALL[rng.below(HtmlResource::ALL.len())];
        }
        let mut pair = Pair::new(limits);
        for step in 0..300 {
            random_batch(&mut rng, &resources, &mut batch);
            let context = format!("seed {seed} step {step} batch {batch:?}");
            if rng.below(3) == 0 {
                let (ledger, meter) = pair.check(&batch);
                assert_eq!(ledger, meter, "check differs: {context}");
                coverage.checks_refused += u64::from(ledger.is_err());
                pair.assert_same_usage(&format!("after a check: {context}"));
            }
            let peaks: Vec<u64> = HtmlResource::ALL
                .iter()
                .map(|&resource| {
                    pair.meter
                        .view(pair.meter.charging())
                        .unwrap()
                        .peak(resource)
                })
                .collect();
            let (ledger, meter) = pair.apply(&batch);
            assert_eq!(ledger, meter, "outcome differs: {context}");
            pair.assert_same_usage(&context);
            match ledger {
                Ok(()) => {
                    coverage.accepted += 1;
                    coverage.accepted_long += u64::from(batch.len() > 24);
                    let view = pair.meter.view(pair.meter.charging()).unwrap();
                    for (&resource, &before) in HtmlResource::ALL.iter().zip(&peaks) {
                        coverage.peak_inside_batch += u64::from(
                            view.peak(resource) > before
                                && view.peak(resource) > view.live(resource),
                        );
                    }
                }
                Err(refused) => {
                    coverage.refused_after_prefix += u64::from(refused.item > 0);
                    match (refused.reason, refused.change, refused.at) {
                        (RefusalReason::Limit { .. }, Change::Work(..), _) => {
                            coverage.work_limit += 1;
                        }
                        (RefusalReason::Limit { .. }, _, _) => coverage.resource_limit += 1,
                        (RefusalReason::Overflow, _, At::Charging) => {
                            coverage.overflow_charging += 1;
                        }
                        (RefusalReason::Overflow, _, At::Cap) => coverage.overflow_cap += 1,
                        (RefusalReason::Underflow, _, _) => coverage.underflow += 1,
                    }
                }
            }
        }
    }
    // The random batches reach every kind of outcome, so that the equality above
    // covers each.
    assert!(coverage.accepted > 10_000, "accepted batches");
    assert!(
        coverage.accepted_long > 0,
        "accepted batches longer than 24"
    );
    assert!(coverage.work_limit > 100, "work limit refusals");
    assert!(coverage.resource_limit > 100, "resource limit refusals");
    assert!(
        coverage.overflow_charging > 100,
        "overflow in the charging scope"
    );
    assert!(coverage.overflow_cap > 100, "overflow in the cap scope");
    assert!(coverage.underflow > 100, "underflow");
    assert!(
        coverage.refused_after_prefix > 100,
        "refusals after accepted items"
    );
    assert!(
        coverage.peak_inside_batch > 100,
        "peaks reached inside a batch"
    );
    assert!(coverage.checks_refused > 100, "refused checks");
}

#[test]
fn a_peak_reached_inside_a_batch_stays_after_the_batch_releases_it() {
    let mut pair = Pair::new(HtmlLimits::unlimited());
    let batch = [
        Change::Acquire(HtmlResource::ManagedNodes, 5),
        Change::Release(HtmlResource::ManagedNodes, 5),
    ];
    let (ledger, meter) = pair.apply(&batch);
    assert_eq!((ledger, meter), (Ok(()), Ok(())));
    pair.assert_same_usage("after the batch");
    let view = pair.meter.view(pair.meter.charging()).unwrap();
    // The authority is the ledger's rule that intermediate live usage counts
    // toward the peak, written as the expected figures.
    assert_eq!(view.live(HtmlResource::ManagedNodes), 0);
    assert_eq!(view.peak(HtmlResource::ManagedNodes), 5);
}

#[test]
fn a_refused_batch_leaves_the_meter_as_it_was() {
    let limits = HtmlLimits::from_fn(
        |work| {
            if work == HtmlWork::Credits {
                10
            } else {
                u64::MAX
            }
        },
        |resource| {
            if resource == HtmlResource::ManagedEdges {
                3
            } else {
                u64::MAX
            }
        },
    );
    let mut pair = Pair::new(limits);
    assert_eq!(
        pair.apply(&[Change::Acquire(HtmlResource::ManagedEdges, 2)]),
        (Ok(()), Ok(()))
    );
    // The second item reaches the cap after the first one is applied.
    let batch = [
        Change::Work(HtmlWork::CopyByte, 4),
        Change::Acquire(HtmlResource::ManagedEdges, 2),
    ];
    let (ledger, meter) = pair.apply(&batch);
    assert_eq!(ledger, meter);
    assert_eq!(
        ledger,
        Err(Refused {
            at: At::Cap,
            item: 1,
            change: Change::Acquire(HtmlResource::ManagedEdges, 2),
            current: 2,
            reason: RefusalReason::Limit { limit: 3 },
        })
    );
    pair.assert_same_usage("after the refusal");
    let view = pair.meter.view(pair.meter.cap()).unwrap();
    assert_eq!(view.work(HtmlWork::Credits), 0);
    assert_eq!(view.live(HtmlResource::ManagedEdges), 2);
}

#[test]
fn a_scope_of_another_operation_is_foreign_to_the_meter() {
    let meter = Meter::with_limits(HtmlLimits::unlimited()).unwrap();
    let (_, other, _) = private(HtmlLimits::unlimited()).unwrap();
    assert!(matches!(meter.view(other), Err(Error::ForeignScope)));
    assert!(matches!(meter.limits(other), Err(Error::ForeignScope)));
}

#[test]
fn the_caps_of_the_scopes_are_those_of_the_ledger() {
    let limits = HtmlLimits::from_fn(
        |work| {
            if work == HtmlWork::Credits {
                77
            } else {
                u64::MAX
            }
        },
        |resource| {
            if resource == HtmlResource::TreeDepth {
                9
            } else {
                u64::MAX
            }
        },
    );
    let pair = Pair::new(limits);
    assert_eq!(
        pair.meter.limits(pair.meter.cap()).unwrap(),
        *pair.operation.limits(pair.cap).unwrap()
    );
    assert_eq!(
        pair.meter.limits(pair.meter.charging()).unwrap(),
        *pair.operation.limits(pair.charging).unwrap()
    );
}
