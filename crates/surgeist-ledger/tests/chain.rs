//! A chain against the operation it stands in for.
//!
//! The authority is [`Operation`]: a root with `BELOW` scopes below it, one
//! under the other, charged at its leaf, admits and refuses a batch by the
//! ledger's rules. A [`Chain`] with the same caps and tariffs must give the
//! same outcome for every batch and leave the same usage in every scope.

use surgeist_ledger::{
    Chain, Change, Charge, Dimension, Error, Limits, NoDimension, NoTable, Operation,
    RefusalReason, Scope, Tariff, Tariffs,
};

surgeist_ledger::dimensions! {
    enum NarrowWork => NarrowWorkTable, NarrowWorkFields { Step, Credit }
}

surgeist_ledger::dimensions! {
    enum NarrowResource => NarrowResourceTable, NarrowResourceFields { Bytes }
}

surgeist_ledger::dimensions! {
    enum WideWork => WideWorkTable, WideWorkFields { A, B, C, D, E, F, G, H, Total }
}

surgeist_ledger::dimensions! {
    enum WideResource => WideResourceTable, WideResourceFields { Nodes, Bytes, Slots, Handles, Frames }
}

/// A refusal with its scope named by depth below the root, so that a chain
/// and an operation, whose scope identities differ, can be compared.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Refused<W: Dimension, R: Dimension> {
    depth: usize,
    item: usize,
    change: Change<W, R>,
    current: u64,
    reason: RefusalReason,
}

fn outcome<W: Dimension, R: Dimension>(
    result: Result<(), Error<W, R>>,
    scopes: &[Scope],
) -> Result<(), Refused<W, R>> {
    match result {
        Ok(()) => Ok(()),
        Err(Error::Refused(refusal)) => Err(Refused {
            depth: scopes
                .iter()
                .position(|&scope| scope == refusal.scope)
                .expect("a refusal names a scope of the chain"),
            item: refusal.item,
            change: refusal.change,
            current: refusal.current,
            reason: refusal.reason,
        }),
        Err(other) => panic!("an error that is not a refusal: {other:?}"),
    }
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn pick<T: Copy>(&mut self, values: &[T]) -> Option<T> {
        if values.is_empty() {
            None
        } else {
            Some(values[self.below(values.len())])
        }
    }
}

fn cap(rng: &mut Rng) -> u64 {
    match rng.below(4) {
        0 => u64::MAX,
        1 => 1_000 + rng.below(4_000) as u64,
        2 => rng.below(30) as u64,
        _ => 40 + rng.below(200) as u64,
    }
}

fn scale(rng: &mut Rng) -> u64 {
    match rng.below(12) {
        0 => 0,
        1 => 2,
        2 => 3,
        3 => u64::MAX / 2,
        _ => 1,
    }
}

fn amount(rng: &mut Rng) -> u64 {
    match rng.below(40) {
        0 => u64::MAX,
        1 => u64::MAX - rng.below(4) as u64,
        2 => 0,
        _ => 1 + rng.below(9) as u64,
    }
}

fn limits<W: Dimension, R: Dimension>(rng: &mut Rng) -> Limits<W, R> {
    let work = W::from_fn(|_| cap(rng));
    let live = R::from_fn(|_| cap(rng));
    Limits::new(work, live)
}

fn tariffs<W: Dimension, R: Dimension>(rng: &mut Rng) -> Tariffs<W, R> {
    if rng.below(3) == 0 {
        return Tariffs::identity();
    }
    let work = W::from_fn(|dimension| {
        let target = rng.pick(W::ALL).unwrap_or(dimension);
        Tariff::new(target, scale(rng))
    });
    let resources = R::from_fn(|dimension| {
        let target = rng.pick(R::ALL).unwrap_or(dimension);
        Tariff::new(target, scale(rng))
    });
    Tariffs::new(work, resources)
}

fn batch<W: Dimension, R: Dimension>(rng: &mut Rng, changes: &mut Vec<Change<W, R>>) {
    changes.clear();
    let length = match rng.below(10) {
        0 => 0,
        1..=4 => 1,
        5..=7 => 2 + rng.below(3),
        8 => 5 + rng.below(8),
        _ => 33 + rng.below(8),
    };
    for _ in 0..length {
        let work = rng.pick(W::ALL);
        let resource = rng.pick(R::ALL);
        let change = match (rng.below(5), work, resource) {
            (0 | 1, Some(work), _) | (_, Some(work), None) => Change::Work(work, amount(rng)),
            (2 | 3, _, Some(resource)) | (_, None, Some(resource)) => {
                Change::Acquire(resource, amount(rng))
            }
            (_, _, Some(resource)) => Change::Release(resource, amount(rng)),
            (_, None, None) => return,
        };
        changes.push(change);
    }
}

/// A chain and the operation of the same caps and tariffs.
struct Pair<W: Dimension, R: Dimension, const BELOW: usize> {
    operation: Operation<W, R>,
    operation_scopes: Vec<Scope>,
    chain: Chain<W, R, BELOW>,
    chain_scopes: Vec<Scope>,
}

impl<W: Dimension, R: Dimension, const BELOW: usize> Pair<W, R, BELOW> {
    fn new(root: Limits<W, R>, below: [(Limits<W, R>, Tariffs<W, R>); BELOW]) -> Self {
        let mut operation = Operation::new("root", root, BELOW + 1).unwrap();
        let mut operation_scopes = vec![operation.root()];
        for &(limits, tariffs) in &below {
            let parent = *operation_scopes.last().unwrap();
            operation_scopes.push(operation.child(parent, "below", limits, tariffs).unwrap());
        }
        let chain = Chain::new(root, below).unwrap();
        let chain_scopes: Vec<Scope> = (0..=BELOW)
            .map(|depth| chain.scope(depth).unwrap())
            .collect();
        assert_eq!(chain_scopes[0], chain.root());
        assert_eq!(chain_scopes[BELOW], chain.leaf());
        assert_eq!(chain.scope(BELOW + 1), None);
        Self {
            operation,
            operation_scopes,
            chain,
            chain_scopes,
        }
    }

    fn random(rng: &mut Rng) -> Self {
        let root = limits(rng);
        let below = [(); BELOW].map(|()| (limits(rng), tariffs(rng)));
        Self::new(root, below)
    }

    fn leaf(&self) -> Scope {
        self.operation_scopes[BELOW]
    }

    #[allow(clippy::type_complexity)]
    fn apply(
        &mut self,
        changes: &[Change<W, R>],
    ) -> (Result<(), Refused<W, R>>, Result<(), Refused<W, R>>) {
        let leaf = self.leaf();
        (
            outcome(
                self.operation.apply_batch(leaf, changes),
                &self.operation_scopes,
            ),
            outcome(self.chain.apply_batch(changes), &self.chain_scopes),
        )
    }

    #[allow(clippy::type_complexity)]
    fn check(
        &mut self,
        changes: &[Change<W, R>],
    ) -> (Result<(), Refused<W, R>>, Result<(), Refused<W, R>>) {
        let leaf = self.leaf();
        (
            outcome(
                self.operation.check_batch(leaf, changes),
                &self.operation_scopes,
            ),
            outcome(self.chain.check_batch(changes), &self.chain_scopes),
        )
    }

    #[allow(clippy::type_complexity)]
    fn record(
        &mut self,
        resource: R,
        value: u64,
    ) -> (Result<(), Refused<W, R>>, Result<(), Refused<W, R>>) {
        let leaf = self.leaf();
        (
            outcome(
                self.operation
                    .charge_at(leaf)
                    .record_largest(resource, value),
                &self.operation_scopes,
            ),
            outcome(
                self.chain.record_largest(resource, value),
                &self.chain_scopes,
            ),
        )
    }

    /// Asserts that every scope holds the same work, live values and peaks in
    /// the operation and in the chain, and has the same caps.
    fn assert_same(&self, context: &str) {
        for (depth, (&ledger, &chain)) in self
            .operation_scopes
            .iter()
            .zip(&self.chain_scopes)
            .enumerate()
        {
            assert_eq!(
                self.operation.usage(ledger).unwrap(),
                self.chain.usage(chain).unwrap(),
                "{context}: usage at depth {depth}"
            );
            assert_eq!(
                self.operation.limits(ledger).unwrap(),
                self.chain.limits(chain).unwrap(),
                "{context}: limits at depth {depth}"
            );
        }
    }
}

/// Runs random batches through chains and operations of random caps and
/// tariffs and returns how many batches were admitted and refused.
fn compare<W: Dimension, R: Dimension, const BELOW: usize>(seeds: u64) -> (u64, u64) {
    let mut admitted = 0_u64;
    let mut refusals = 0_u64;
    let mut changes = Vec::new();
    for seed in 1..=seeds {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let mut pair = Pair::<W, R, BELOW>::random(&mut rng);
        for step in 0..200 {
            let context = format!("seed {seed}, step {step}, {BELOW} below");
            let (ledger, chain) = match (rng.below(8), rng.pick(R::ALL)) {
                (0, Some(resource)) => pair.record(resource, amount(&mut rng)),
                (1 | 2, _) => {
                    batch(&mut rng, &mut changes);
                    pair.check(&changes)
                }
                _ => {
                    batch(&mut rng, &mut changes);
                    pair.apply(&changes)
                }
            };
            assert_eq!(ledger, chain, "{context}: {changes:?}");
            match ledger {
                Ok(()) => admitted += 1,
                Err(_) => refusals += 1,
            }
            pair.assert_same(&context);
        }
    }
    (admitted, refusals)
}

#[test]
fn a_chain_admits_and_refuses_like_an_operation_of_the_same_scopes() {
    let mut totals = (0_u64, 0_u64);
    let mut add = |(admitted, refusals): (u64, u64)| {
        // Each shape must see both outcomes, or the comparison proves little.
        assert!(admitted > 200, "{admitted} batches admitted");
        assert!(refusals > 200, "{refusals} batches refused");
        totals.0 += admitted;
        totals.1 += refusals;
    };
    add(compare::<NarrowWork, NarrowResource, 0>(60));
    add(compare::<NarrowWork, NarrowResource, 1>(60));
    add(compare::<NarrowWork, NarrowResource, 3>(60));
    add(compare::<WideWork, WideResource, 0>(60));
    add(compare::<WideWork, WideResource, 1>(120));
    add(compare::<WideWork, WideResource, 2>(60));
    add(compare::<WideWork, WideResource, 3>(60));
    add(compare::<WideWork, NoDimension, 1>(40));
    add(compare::<NoDimension, WideResource, 2>(40));
}

fn wide(work: u64, bytes: u64) -> Limits<WideWork, WideResource> {
    Limits::from_fn(
        |dimension| {
            if dimension == WideWork::Total {
                work
            } else {
                u64::MAX
            }
        },
        |dimension| {
            if dimension == WideResource::Bytes {
                bytes
            } else {
                u64::MAX
            }
        },
    )
}

/// Every work dimension counts as `Total` in the parent, at rate one.
fn totalling() -> Tariffs<WideWork, WideResource> {
    Tariffs::from_fn(
        |_| Tariff::new(WideWork::Total, 1),
        |dimension| Tariff::new(dimension, 1),
    )
}

#[test]
fn a_scope_below_a_capped_root_is_refused_by_the_root_in_the_roots_units() {
    let mut chain =
        Chain::<WideWork, WideResource, 1>::new(wide(10, 64), [(Limits::unlimited(), totalling())])
            .unwrap();
    chain.apply(Change::Work(WideWork::A, 6)).unwrap();
    chain.apply(Change::Work(WideWork::B, 4)).unwrap();
    let refusal = match chain.apply(Change::Work(WideWork::C, 1)) {
        Err(Error::Refused(refusal)) => refusal,
        other => panic!("the eleventh unit is refused: {other:?}"),
    };
    assert_eq!(refusal.scope, chain.root());
    assert_eq!(refusal.item, 0);
    assert_eq!(refusal.change, Change::Work(WideWork::Total, 1));
    assert_eq!(refusal.current, 10);
    assert_eq!(refusal.reason, RefusalReason::Limit { limit: 10 });
    let leaf = chain.usage(chain.leaf()).unwrap();
    assert_eq!(leaf.work(WideWork::A), 6);
    assert_eq!(leaf.work(WideWork::B), 4);
    assert_eq!(leaf.work(WideWork::C), 0);
    assert_eq!(chain.usage(chain.root()).unwrap().work(WideWork::Total), 10);
}

#[test]
fn a_peak_reached_inside_a_batch_stays_after_the_batch_releases_it() {
    let mut chain =
        Chain::<WideWork, WideResource, 1>::new(wide(10, 64), [(Limits::unlimited(), totalling())])
            .unwrap();
    chain
        .apply_batch(&[
            Change::Acquire(WideResource::Bytes, 48),
            Change::Release(WideResource::Bytes, 40),
        ])
        .unwrap();
    for scope in [chain.leaf(), chain.root()] {
        let usage = chain.usage(scope).unwrap();
        assert_eq!(usage.live(WideResource::Bytes), 8);
        assert_eq!(usage.peak(WideResource::Bytes), 48);
    }
}

#[test]
fn a_refused_batch_reports_the_value_its_earlier_items_reached_and_changes_nothing() {
    let mut chain =
        Chain::<WideWork, WideResource, 1>::new(wide(10, 64), [(Limits::unlimited(), totalling())])
            .unwrap();
    chain
        .apply(Change::Acquire(WideResource::Bytes, 16))
        .unwrap();
    let before = (
        *chain.usage(chain.root()).unwrap(),
        *chain.usage(chain.leaf()).unwrap(),
    );
    let refusal = match chain.apply_batch(&[
        Change::Work(WideWork::A, 3),
        Change::Acquire(WideResource::Bytes, 40),
        Change::Acquire(WideResource::Bytes, 9),
    ]) {
        Err(Error::Refused(refusal)) => refusal,
        other => panic!("the third item passes the cap: {other:?}"),
    };
    assert_eq!(refusal.scope, chain.root());
    assert_eq!(refusal.item, 2);
    assert_eq!(refusal.change, Change::Acquire(WideResource::Bytes, 9));
    assert_eq!(refusal.current, 56);
    assert_eq!(refusal.reason, RefusalReason::Limit { limit: 64 });
    assert_eq!(
        (
            *chain.usage(chain.root()).unwrap(),
            *chain.usage(chain.leaf()).unwrap()
        ),
        before
    );
}

#[test]
fn a_checked_batch_changes_nothing() {
    let mut chain = Chain::<WideWork, WideResource, 2>::new(
        wide(10, 64),
        [
            (Limits::unlimited(), Tariffs::identity()),
            (Limits::unlimited(), totalling()),
        ],
    )
    .unwrap();
    chain
        .check_batch(&[
            Change::Work(WideWork::A, 3),
            Change::Acquire(WideResource::Bytes, 40),
        ])
        .unwrap();
    chain.check_batch(&[Change::Work(WideWork::A, 3)]).unwrap();
    assert!(chain.check_batch(&[Change::Work(WideWork::A, 11)]).is_err());
    for depth in 0..=2 {
        let usage = chain.usage(chain.scope(depth).unwrap()).unwrap();
        assert_eq!(usage.work(WideWork::A), 0);
        assert_eq!(usage.work(WideWork::Total), 0);
        assert_eq!(usage.live(WideResource::Bytes), 0);
        assert_eq!(usage.peak(WideResource::Bytes), 0);
    }
}

#[test]
fn a_release_of_more_than_the_leaf_holds_is_an_underflow_at_the_leaf() {
    let mut chain =
        Chain::<WideWork, WideResource, 1>::new(wide(10, 64), [(Limits::unlimited(), totalling())])
            .unwrap();
    chain
        .apply(Change::Acquire(WideResource::Bytes, 5))
        .unwrap();
    let refusal = match chain.apply(Change::Release(WideResource::Bytes, 6)) {
        Err(Error::Refused(refusal)) => refusal,
        other => panic!("six of five cannot be released: {other:?}"),
    };
    assert_eq!(refusal.scope, chain.leaf());
    assert_eq!(refusal.current, 5);
    assert_eq!(refusal.reason, RefusalReason::Underflow);
}

#[test]
fn an_amount_a_tariff_scales_past_the_range_is_an_overflow_at_the_scope_below() {
    let doubling = Tariffs::from_fn(
        |dimension| Tariff::new(dimension, 2),
        |dimension: WideResource| Tariff::new(dimension, 1),
    );
    let mut chain = Chain::<WideWork, WideResource, 1>::new(
        Limits::unlimited(),
        [(Limits::unlimited(), doubling)],
    )
    .unwrap();
    let amount = u64::MAX / 2 + 1;
    let refusal = match chain.apply(Change::Work(WideWork::A, amount)) {
        Err(Error::Refused(refusal)) => refusal,
        other => panic!("twice the amount is out of range: {other:?}"),
    };
    assert_eq!(refusal.scope, chain.leaf());
    assert_eq!(refusal.change, Change::Work(WideWork::A, amount));
    assert_eq!(refusal.current, 0);
    assert_eq!(refusal.reason, RefusalReason::Overflow);
    assert_eq!(chain.usage(chain.leaf()).unwrap().work(WideWork::A), 0);
}

#[test]
fn a_scope_of_an_operation_or_of_another_chain_is_foreign() {
    let chain = Chain::<WideWork, WideResource, 1>::new(
        Limits::unlimited(),
        [(Limits::unlimited(), Tariffs::identity())],
    )
    .unwrap();
    let other = Chain::<WideWork, WideResource, 1>::new(
        Limits::unlimited(),
        [(Limits::unlimited(), Tariffs::identity())],
    )
    .unwrap();
    let operation =
        Operation::<WideWork, WideResource>::new("root", Limits::unlimited(), 1).unwrap();
    assert_eq!(chain.usage(other.leaf()), Err(Error::ForeignScope));
    assert_eq!(chain.limits(other.root()), Err(Error::ForeignScope));
    assert_eq!(chain.usage(operation.root()), Err(Error::ForeignScope));
    assert_eq!(
        operation.usage(chain.root()).err(),
        Some(Error::ForeignScope)
    );
}

#[test]
fn a_chain_with_no_scope_below_charges_its_root() {
    let mut chain = Chain::<WideWork, NoDimension, 0>::new(
        Limits::from_fn(|_| 3, |dimension: NoDimension| match dimension {}),
        [],
    )
    .unwrap();
    assert_eq!(chain.root(), chain.leaf());
    chain.apply(Change::Work(WideWork::A, 3)).unwrap();
    assert!(chain.apply(Change::Work(WideWork::A, 1)).is_err());
    assert_eq!(chain.usage(chain.root()).unwrap().work(WideWork::A), 3);
    let _ = NoTable::<u64>::EMPTY;
}

#[test]
fn a_gauge_keeps_the_largest_value_recorded_through_charge() {
    let mut chain =
        Chain::<WideWork, WideResource, 1>::new(wide(10, 64), [(Limits::unlimited(), totalling())])
            .unwrap();
    chain.record_largest(WideResource::Bytes, 40).unwrap();
    chain.record_largest(WideResource::Bytes, 12).unwrap();
    assert!(chain.record_largest(WideResource::Bytes, 65).is_err());
    let usage = chain.usage(chain.leaf()).unwrap();
    assert_eq!(usage.live(WideResource::Bytes), 12);
    assert_eq!(usage.peak(WideResource::Bytes), 40);
    chain.charge(&[Change::Work(WideWork::A, 1)]).unwrap();
    assert_eq!(chain.usage(chain.root()).unwrap().work(WideWork::Total), 1);
}
