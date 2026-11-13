//! Defects reached by corrupting private ledger state. No valid use of the
//! public API produces these states, so each test sets one up directly and
//! checks the typed error, that it is latched, and that `finish` returns it.

use crate::{
    Change, Defect, Dimension, Error, Limits, Operation, Parent, ProfileMode, ScopeEntry, Tariffs,
};

crate::dimensions! {
    enum Work => WorkTable { Visit }
}

crate::dimensions! {
    enum Resource => ResourceTable { Bytes }
}

#[test]
fn reserved_shared_backing_covers_exact_scope_capacity() {
    let mut operation =
        Operation::<Work, Resource>::with_reserved_scopes("root", Limits::unlimited(), 2).unwrap();
    assert_eq!(operation.scopes.capacity(), 2);
    let allocated = size_of::<std::sync::Mutex<Operation<Work, Resource>>>()
        + operation.scopes.capacity() * size_of::<ScopeEntry<Work, Resource>>();
    assert_eq!(
        crate::SharedOperation::<Work, Resource>::reserved_backing_bytes(2),
        Some(allocated)
    );
    let root = operation.root();
    operation
        .child(root, "child", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    assert_eq!(operation.scopes.capacity(), 2);
    assert_eq!(
        operation.child(root, "excess", Limits::unlimited(), Tariffs::identity()),
        Err(Error::ScopeLimit { limit: 2 })
    );
    assert_eq!(
        crate::SharedOperation::<Work, Resource>::reserved_backing_bytes(0),
        None
    );
}

#[test]
fn reserved_profile_backing_covers_the_scope_and_observer_extents() {
    use crate::{Event, EventFilter, ScopeReport, SharedOperation, profile::Profile};
    use std::sync::Mutex;

    for mode in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 3,
            filter: EventFilter::All,
        },
    ] {
        let mut operation = Operation::<Work, Resource>::with_reserved_profile(
            "root",
            Limits::unlimited(),
            2,
            mode,
        )
        .unwrap();
        let root = operation.root();
        operation
            .child(root, "child", Limits::unlimited(), Tariffs::identity())
            .unwrap();
        assert_eq!(operation.scopes.capacity(), 2);
        let observer = match mode {
            ProfileMode::Off => {
                assert_eq!(operation.profile.capacity(), 0);
                assert!(operation.snapshot().unwrap().is_none());
                0
            }
            mode => {
                assert_eq!(operation.profile.capacity(), 1);
                let report = operation.snapshot().unwrap().unwrap();
                assert_eq!(report.scopes.len(), 2);
                let events = match mode {
                    ProfileMode::Detailed { event_limit, .. } => event_limit,
                    _ => 0,
                };
                size_of::<Profile<Work, Resource>>()
                    + 2 * size_of::<ScopeReport<Work, Resource>>()
                    + events * size_of::<Event<Work, Resource>>()
            }
        };
        let base = size_of::<Mutex<Operation<Work, Resource>>>()
            + 2 * size_of::<ScopeEntry<Work, Resource>>();
        assert_eq!(
            SharedOperation::<Work, Resource>::profiled_backing_bytes(2, mode),
            Some(base + observer)
        );
    }
    assert_eq!(
        SharedOperation::<Work, Resource>::profiled_backing_bytes(
            1,
            ProfileMode::Detailed {
                timing: false,
                event_limit: usize::MAX,
                filter: EventFilter::All,
            }
        ),
        None
    );
}

fn plain(max_scopes: usize) -> Operation<Work, Resource> {
    Operation::new("root", Limits::unlimited(), max_scopes).unwrap()
}

fn profiled() -> Operation<Work, Resource> {
    Operation::with_profile(
        "root",
        Limits::unlimited(),
        2,
        ProfileMode::Summary { timing: false },
    )
    .unwrap()
}

/// Adds a scope entry that the profile has no record for and returns its
/// identity. The operation's scopes and its report records no longer match.
pub(crate) fn unrecorded_scope<W: Dimension, R: Dimension>(
    operation: &mut Operation<W, R>,
) -> crate::Scope {
    operation
        .scopes
        .push(ScopeEntry::new("unrecorded", None, Limits::unlimited()));
    operation.scope(operation.scopes.len() - 1)
}

/// Checks that every later fallible call on `operation` returns `defect`, and
/// that `finish` returns it instead of a report.
pub(crate) fn assert_latched<W: Dimension, R: Dimension>(
    mut operation: Operation<W, R>,
    defect: Defect,
) {
    let latched = Err(Error::Defect(defect));
    let root = operation.root();
    assert_eq!(operation.apply_batch(root, &[]), latched);
    assert_eq!(operation.check_batch(root, &[]), latched);
    assert_eq!(
        operation
            .child(root, "late", Limits::unlimited(), Tariffs::identity())
            .map(|_| ()),
        latched
    );
    assert_eq!(operation.usage(root).map(|_| ()), latched);
    assert_eq!(operation.limits(root).map(|_| ()), latched);
    assert_eq!(operation.scope_name(root).map(|_| ()), latched);
    assert_eq!(operation.begin_phase(root).map(|_| ()), latched);
    assert_eq!(operation.with_phase(root, |_| ()), latched);
    assert_eq!(operation.snapshot().map(|_| ()), latched);
    assert_eq!(operation.begin_checkpoint().map(|_| ()), latched);
    assert_eq!(operation.finish().map(|_| ()), latched);
}

#[test]
fn scope_identity_without_an_entry_is_a_defect_on_first_use() {
    let mut operation = plain(2);
    let root = operation.root();
    let child = operation
        .child(root, "child", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    operation.scopes.pop();
    assert_eq!(
        operation.apply(child, Change::Work(Work::Visit, 1)),
        Err(Error::Defect(Defect::ScopeEntry))
    );
}

#[test]
fn scope_identity_without_an_entry_is_a_latched_defect() {
    let mut operation = plain(2);
    let root = operation.root();
    let child = operation
        .child(root, "child", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    operation.scopes.pop();
    assert_eq!(
        operation.usage(child).map(|_| ()),
        Err(Error::Defect(Defect::ScopeEntry))
    );
    assert_latched(operation, Defect::ScopeEntry);
}

#[test]
fn parent_naming_no_scope_is_a_latched_defect() {
    let mut operation = plain(2);
    let root = operation.root();
    let child = operation
        .child(root, "child", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    operation.scopes[1].parent = Some(Parent {
        index: 7,
        tariffs: Tariffs::identity(),
    });
    assert_eq!(
        operation.apply(child, Change::Work(Work::Visit, 1)),
        Err(Error::Defect(Defect::ScopeEntry))
    );
    assert_latched(operation, Defect::ScopeEntry);
}

#[test]
fn direct_custody_above_accepted_custody_is_a_latched_defect() {
    let mut operation = plain(1);
    let root = operation.root();
    // Direct holdings are a subset of the admitted live total, which is zero.
    operation.scopes[0].direct.Bytes = u64::MAX;
    assert_eq!(
        operation.apply(root, Change::Acquire(Resource::Bytes, 1)),
        Err(Error::Defect(Defect::DirectCustody))
    );
    assert_latched(operation, Defect::DirectCustody);
}

#[test]
fn admission_to_a_scope_without_a_report_record_is_a_latched_defect() {
    let mut operation = profiled();
    let scope = unrecorded_scope(&mut operation);
    assert_eq!(
        operation.apply(scope, Change::Work(Work::Visit, 1)),
        Err(Error::Defect(Defect::ReportScope))
    );
    assert_latched(operation, Defect::ReportScope);
}

#[test]
fn borrowed_phase_completion_on_drop_latches_a_report_defect() {
    let mut operation = profiled();
    let scope = unrecorded_scope(&mut operation);
    // Completion has no error return: the call succeeds and the defect is
    // returned by the next fallible call.
    assert_eq!(operation.with_phase(scope, |_| 7), Ok(7));
    assert_latched(operation, Defect::ReportScope);
}

#[test]
fn detached_phase_completion_latches_a_report_defect() {
    let mut operation = profiled();
    let scope = unrecorded_scope(&mut operation);
    let observation = operation.begin_phase(scope).unwrap();
    assert!(operation.end_phase(observation).is_ok());
    assert_latched(operation, Defect::ReportScope);
}

#[test]
fn shared_phase_completion_on_drop_latches_a_report_defect() {
    let mut operation = profiled();
    let scope = unrecorded_scope(&mut operation);
    let shared = operation.share();
    assert_eq!(shared.with_phase(scope, || 7), Ok(7));
    let root = shared.root();
    let latched = Err(Error::Defect(Defect::ReportScope));
    assert_eq!(shared.apply_batch(root, &[]), latched);
    assert_eq!(shared.snapshot().map(|_| ()), latched);
    assert_eq!(shared.begin_checkpoint().map(|_| ()), latched);
}

#[test]
fn report_records_not_matching_the_scopes_are_a_defect() {
    let mut operation = profiled();
    unrecorded_scope(&mut operation);
    assert_eq!(
        operation.snapshot().map(|_| ()),
        Err(Error::Defect(Defect::ReportScope))
    );
    assert_eq!(
        operation.finish().map(|_| ()),
        Err(Error::Defect(Defect::ReportScope))
    );
}

#[test]
fn snapshot_finding_mismatched_report_records_latches_the_defect() {
    let mut operation = profiled();
    unrecorded_scope(&mut operation);
    assert_eq!(
        operation.snapshot().map(|_| ()),
        Err(Error::Defect(Defect::ReportScope))
    );
    assert_latched(operation, Defect::ReportScope);
}

mod staging_stores {
    use super::*;
    use crate::{
        Operation, ProfileMode, Scope, Tariff,
        in_place::{FORCE, Force, ONE_WORK, ONE_WORK_DECLINED},
    };

    crate::dimensions! {
        enum Wk => WkTable { A, B, C, D }
    }

    crate::dimensions! {
        enum Rs => RsTable { X, Y, Z }
    }

    /// A deterministic generator, so a failing case can be replayed.
    struct Random(std::cell::Cell<u64>);

    impl Random {
        fn next(&self) -> u64 {
            let mut state = self.0.get();
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            self.0.set(state);
            state
        }

        fn below(&self, bound: u64) -> u64 {
            self.next() % bound
        }
    }

    fn cap(random: &Random) -> u64 {
        match random.below(4) {
            0 => u64::MAX,
            1 => random.below(6),
            _ => random.below(60),
        }
    }

    fn work(random: &Random) -> Wk {
        Wk::ALL[random.below(Wk::ALL.len() as u64) as usize]
    }

    fn resource(random: &Random) -> Rs {
        Rs::ALL[random.below(Rs::ALL.len() as u64) as usize]
    }

    fn scale(random: &Random) -> u64 {
        match random.below(12) {
            0 => 0,
            1 => u64::MAX / 3,
            2 | 3 => 2,
            4 => 3,
            _ => 1,
        }
    }

    /// Builds the same random scope tree from `seed` every time.
    ///
    /// `variant` 0 is a plain operation, 1 profiles it, and 2 profiles it and
    /// opens an interval, which records the batch peak of every resource.
    /// `chain` makes every scope the child of the previous one, with up to five
    /// children, so the chain from the last scope can be longer than the hops
    /// the journal-free pass holds; otherwise parents are random.
    fn operation(seed: u64, variant: u64, chain: bool) -> (Operation<Wk, Rs>, Vec<Scope>) {
        let random = Random(std::cell::Cell::new(seed));
        let limit_set = |random: &Random| Limits::from_fn(|_| cap(random), |_| cap(random));
        let mut operation = if variant == 0 {
            Operation::new("root", limit_set(&random), 6).unwrap()
        } else {
            Operation::with_profile(
                "root",
                limit_set(&random),
                6,
                ProfileMode::Summary { timing: false },
            )
            .unwrap()
        };
        if variant == 2 {
            operation = operation.with_checkpoint_limit(1).unwrap();
        }
        let mut scopes = vec![operation.root()];
        for _ in 0..random.below(if chain { 6 } else { 5 }) {
            let parent = if chain {
                scopes[scopes.len() - 1]
            } else {
                scopes[random.below(scopes.len() as u64) as usize]
            };
            let limits = limit_set(&random);
            let tariffs = Tariffs::from_fn(
                |_| Tariff::new(work(&random), scale(&random)),
                |_| Tariff::new(resource(&random), scale(&random)),
            );
            scopes.push(operation.child(parent, "child", limits, tariffs).unwrap());
        }
        (operation, scopes)
    }

    /// A batch of fewer than `longest` changes.
    fn batch(random: &Random, longest: u64) -> Vec<Change<Wk, Rs>> {
        let amount = |random: &Random| match random.below(10) {
            0 => u64::MAX / 2,
            1 => 0,
            _ => random.below(9),
        };
        (0..random.below(longest))
            .map(|_| match random.below(3) {
                0 => Change::Work(work(random), amount(random)),
                1 => Change::Acquire(resource(random), amount(random)),
                _ => Change::Release(resource(random), amount(random)),
            })
            .collect()
    }

    /// The result with the operation identity removed from a refused scope, so
    /// results of two operations compare.
    fn anonymous(result: Result<(), Error<Wk, Rs>>) -> Result<(), Error<Wk, Rs>> {
        match result {
            Err(Error::Refused(mut refusal)) => {
                refusal.scope.operation = 0;
                Err(Error::Refused(refusal))
            }
            other => other,
        }
    }

    fn with<T>(store: Force, run: impl FnOnce() -> T) -> T {
        FORCE.set(store);
        let result = run();
        FORCE.set(Force::Auto);
        result
    }

    /// Runs random batches through `store` and through the dense store on twin
    /// operations. Every batch is accepted or refused the same way, with the same
    /// refusal, and leaves the same usage, peaks, direct custody, profile and
    /// interval report. Returns the number of accepted and refused batches.
    fn agree(
        seeds: std::ops::RangeInclusive<u64>,
        chain: bool,
        longest: u64,
        store: Force,
    ) -> (usize, usize) {
        let mut accepted = 0;
        let mut refused = 0;
        for seed in seeds {
            let variant = seed % 3;
            let (mut fast, scopes) =
                operation(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15), variant, chain);
            let (mut slow, slow_scopes) =
                operation(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15), variant, chain);
            let intervals = (variant == 2).then(|| {
                (
                    fast.begin_checkpoint().unwrap(),
                    slow.begin_checkpoint().unwrap(),
                )
            });
            let random = Random(std::cell::Cell::new(
                seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1,
            ));
            for _ in 0..40 {
                let position = random.below(scopes.len() as u64) as usize;
                let (scope, twin) = (scopes[position], slow_scopes[position]);
                let changes = batch(&random, longest);
                assert_eq!(
                    anonymous(with(store, || fast.check_batch(scope, &changes))),
                    anonymous(with(Force::Dense, || slow.check_batch(twin, &changes))),
                    "seed {seed}: check"
                );
                let result = with(store, || fast.apply_batch(scope, &changes));
                assert_eq!(
                    anonymous(result),
                    anonymous(with(Force::Dense, || slow.apply_batch(twin, &changes))),
                    "seed {seed}: apply"
                );
                if result.is_ok() {
                    accepted += 1;
                } else {
                    refused += 1;
                }
                for (fast, slow) in fast.scopes.iter().zip(&slow.scopes) {
                    assert_eq!(fast.usage, slow.usage, "seed {seed}: usage");
                    assert_eq!(fast.direct, slow.direct, "seed {seed}: direct");
                }
            }
            if let Some((fast_token, slow_token)) = intervals {
                let fast_report = fast.end_checkpoint(&fast_token).unwrap();
                let slow_report = slow.end_checkpoint(&slow_token).unwrap();
                let usage = |report: &crate::IntervalReport<Wk, Rs>| {
                    report
                        .scopes
                        .iter()
                        .map(|scope| scope.usage)
                        .collect::<Vec<_>>()
                };
                assert_eq!(
                    usage(&fast_report),
                    usage(&slow_report),
                    "seed {seed}: interval"
                );
            }
            if variant != 0 {
                let observed = |operation: &Operation<Wk, Rs>| {
                    operation
                        .snapshot()
                        .unwrap()
                        .unwrap()
                        .scopes
                        .into_iter()
                        .map(|scope| {
                            (
                                scope.usage,
                                scope.accepted_batches,
                                scope.accepted_items,
                                scope.refused_batches,
                                scope.first_refusal.map(|first| {
                                    (first.item, first.change, first.current, first.reason)
                                }),
                            )
                        })
                        .collect::<Vec<_>>()
                };
                assert_eq!(observed(&fast), observed(&slow), "seed {seed}: profile");
            }
        }
        (accepted, refused)
    }

    /// Every batch is accepted or refused the same way by the in-place store
    /// and by the dense store, including batches too large for the journal.
    #[test]
    fn the_in_place_and_dense_stores_agree_on_random_batches() {
        let (accepted, refused) = agree(1..=600, false, 11, Force::InPlace);
        assert!(accepted > 1_000 && refused > 1_000, "{accepted} {refused}");
    }

    /// Batches of at most one change, which the journal-free pass commits when
    /// every scope on a chain of up to four accepts and the operation has no
    /// profile and no checkpoint limit, agree with the dense store on random
    /// trees and on chains longer than the pass holds. The pass commits changes
    /// and declines others to the general path, and the dense store never tries
    /// it.
    #[test]
    fn the_journal_free_pass_agrees_with_the_dense_store_on_single_changes() {
        for store in [Force::Auto, Force::InPlace] {
            for chain in [false, true] {
                let (committed, declined) = (ONE_WORK.get(), ONE_WORK_DECLINED.get());
                let (accepted, refused) = agree(1..=900, chain, 2, store);
                assert!(accepted > 300 && refused > 300, "{accepted} {refused}");
                let committed = ONE_WORK.get().saturating_sub(committed);
                let declined = ONE_WORK_DECLINED.get().saturating_sub(declined);
                assert!(committed > 150, "the pass committed {committed}");
                assert!(declined > 150, "the pass declined {declined}");
            }
        }
    }

    /// Batches of up to four changes use the mid-sized journal; they agree with
    /// the dense store on random trees and on chains.
    #[test]
    fn the_mid_journal_agrees_with_the_dense_store_on_short_batches() {
        for chain in [false, true] {
            let (accepted, refused) = agree(1..=900, chain, 5, Force::InPlace);
            assert!(accepted > 300 && refused > 300, "{accepted} {refused}");
        }
    }

    /// An open interval records the batch peak of every resource, including a
    /// resource acquired and released within one batch. The in-place store
    /// tracks no batch peak, so a wide pair with an open interval must use the
    /// dense store whatever the selection rule prefers.
    #[test]
    fn an_open_interval_records_the_peak_of_an_acquire_and_release_in_one_batch() {
        let build = || {
            let mut operation = Operation::<Wk, Rs>::new("root", Limits::unlimited(), 2)
                .unwrap()
                .with_checkpoint_limit(1)
                .unwrap();
            let root = operation.root();
            let child = operation
                .child(root, "child", Limits::unlimited(), Tariffs::identity())
                .unwrap();
            let token = operation.begin_checkpoint().unwrap();
            (operation, child, token)
        };
        let batch = [
            Change::Acquire(Rs::X, 7),
            Change::Work(Wk::A, 1),
            Change::Release(Rs::X, 7),
        ];
        let (mut preferred, child, token) = build();
        with(Force::InPlace, || preferred.apply_batch(child, &batch)).unwrap();
        let preferred = preferred.end_checkpoint(&token).unwrap();
        let (mut dense, child, token) = build();
        with(Force::Dense, || dense.apply_batch(child, &batch)).unwrap();
        let dense = dense.end_checkpoint(&token).unwrap();
        for report in [&preferred, &dense] {
            for scope in &report.scopes {
                assert_eq!(scope.usage.peak(Rs::X), 7);
                assert_eq!(scope.usage.live(Rs::X), 0);
            }
        }
        let usage = |report: &crate::IntervalReport<Wk, Rs>| {
            report
                .scopes
                .iter()
                .map(|scope| scope.usage)
                .collect::<Vec<_>>()
        };
        assert_eq!(usage(&preferred), usage(&dense));
    }

    /// The selection rule: a pair of fewer than eight usage words never uses
    /// the in-place store, a wide pair uses it for a batch of up to words/8
    /// changes that the journal holds, and the compile-time width test agrees.
    #[test]
    fn the_in_place_store_is_selected_by_width_batch_size_and_chain() {
        use crate::in_place::{preferred, wide};
        crate::dimensions! {
            enum OneWork => OneWorkTable { Only }
        }
        crate::dimensions! {
            enum OneResource => OneResourceTable { Only }
        }
        let work = |count: usize| vec![Change::<Wk, Rs>::Work(Wk::A, 1); count];
        assert!(!wide::<OneWork, OneResource>());
        assert!(!preferred(
            &[Change::<OneWork, OneResource>::Work(OneWork::Only, 1)],
            1
        ));
        // Four work words and three resources of two words each: ten words.
        assert!(wide::<Wk, Rs>());
        assert!(preferred(&work(1), 2));
        assert!(!preferred(&work(2), 2));
        // A pair wide enough for five changes: 2 work and 24 resources.
        crate::dimensions! {
            enum BigWork => BigWorkTable { A, B }
        }
        crate::dimensions! {
            enum BigResource => BigResourceTable {
                R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15,
                R16, R17, R18, R19, R20, R21, R22, R23,
            }
        }
        let big = |count: usize| vec![Change::<BigWork, BigResource>::Work(BigWork::A, 1); count];
        assert!(preferred(&big(5), 2));
        // Five work changes over four scopes journal 20 values; seven journal 28,
        // more than the 24 the journal holds.
        assert!(preferred(&big(5), 4));
        assert!(!preferred(&big(7), 4));
        // An acquisition or release also journals the origin's direct custody.
        let acquires = vec![Change::<BigWork, BigResource>::Acquire(BigResource::R0, 1); 5];
        assert!(!preferred(&acquires, 5));
        assert!(preferred(&acquires, 3));
    }

    /// A chain of `depth` scopes below the root. Every child's tariff scales
    /// work `A` into work `B` by `scale` and passes the other dimensions on.
    fn linked(
        depth: usize,
        scale: u64,
        caps: impl Fn(usize) -> Limits<Wk, Rs>,
    ) -> (Operation<Wk, Rs>, Vec<Scope>) {
        let mut operation =
            Operation::<Wk, Rs>::new("root", caps(0), depth.saturating_add(1)).unwrap();
        let mut scopes = vec![operation.root()];
        for level in 1..=depth {
            let tariffs = Tariffs::from_fn(
                |dimension| match dimension {
                    Wk::A => Tariff::new(Wk::B, scale),
                    other => Tariff::new(other, 1),
                },
                |dimension| Tariff::new(dimension, scale),
            );
            let parent = scopes[level - 1];
            scopes.push(
                operation
                    .child(parent, "child", caps(level), tariffs)
                    .unwrap(),
            );
        }
        (operation, scopes)
    }

    fn uncapped(_: usize) -> Limits<Wk, Rs> {
        Limits::unlimited()
    }

    fn work_usage(operation: &Operation<Wk, Rs>, scopes: &[Scope], dimension: Wk) -> Vec<u64> {
        scopes
            .iter()
            .map(|&scope| operation.usage(scope).unwrap().work(dimension))
            .collect()
    }

    fn refusal(
        scope: Scope,
        change: Change<Wk, Rs>,
        current: u64,
        reason: crate::RefusalReason,
    ) -> Result<(), Error<Wk, Rs>> {
        Err(Error::Refused(crate::Refusal {
            scope,
            item: 0,
            change,
            current,
            reason,
        }))
    }

    /// Applies `change` and checks that the journal-free pass was tried once,
    /// and that it committed the change exactly when `committed` is set; a
    /// declined change is admitted or refused by the general path.
    fn through_the_pass(
        operation: &mut Operation<Wk, Rs>,
        scope: Scope,
        change: Change<Wk, Rs>,
        committed: bool,
    ) -> Result<(), Error<Wk, Rs>> {
        let (taken, declined) = (ONE_WORK.get(), ONE_WORK_DECLINED.get());
        let result = operation.apply(scope, change);
        let (took, declines) = (
            ONE_WORK.get().saturating_sub(taken),
            ONE_WORK_DECLINED.get().saturating_sub(declined),
        );
        assert_eq!((took, declines), if committed { (1, 0) } else { (0, 1) });
        result
    }

    /// A one-change work batch below a three-scope chain is refused at the
    /// first scope along the path, from the leaf towards the root, whose cap
    /// the change exceeds, with the amount that scope sees.
    #[test]
    fn the_journal_free_pass_refuses_at_the_origin_and_at_each_ancestor() {
        // Leaf A cap 5; middle B cap 10 after a factor of 3; root B cap 12.
        let caps = |level: usize| match level {
            0 => Limits::from_fn(|_| 12, |_| u64::MAX),
            1 => Limits::from_fn(|_| 10, |_| u64::MAX),
            _ => Limits::from_fn(|_| 5, |_| u64::MAX),
        };
        let (mut operation, scopes) = linked(2, 3, caps);
        let leaf = scopes[2];
        // 6 exceeds the leaf's cap of 5, before any tariff applies.
        assert_eq!(
            through_the_pass(&mut operation, leaf, Change::Work(Wk::A, 6), false),
            refusal(
                leaf,
                Change::Work(Wk::A, 6),
                0,
                crate::RefusalReason::Limit { limit: 5 }
            )
        );
        assert_eq!(work_usage(&operation, &scopes, Wk::A), [0, 0, 0]);
        assert_eq!(work_usage(&operation, &scopes, Wk::B), [0, 0, 0]);
        // 2 passes the leaf (2 <= 5), reaches the middle as 6 of B (6 <= 10)
        // and the root as 6 of B (6 <= 12).
        through_the_pass(&mut operation, leaf, Change::Work(Wk::A, 2), true).unwrap();
        assert_eq!(work_usage(&operation, &scopes, Wk::A), [0, 0, 2]);
        assert_eq!(work_usage(&operation, &scopes, Wk::B), [6, 6, 0]);
        // 2 more reaches the middle as 6 on 6 held: 12 exceeds its cap of 10.
        assert_eq!(
            through_the_pass(&mut operation, leaf, Change::Work(Wk::A, 2), false),
            refusal(
                scopes[1],
                Change::Work(Wk::B, 6),
                6,
                crate::RefusalReason::Limit { limit: 10 }
            )
        );
        assert_eq!(work_usage(&operation, &scopes, Wk::A), [0, 0, 2]);
        assert_eq!(work_usage(&operation, &scopes, Wk::B), [6, 6, 0]);
        // Work B directly at the middle: 6 held and 4 more reach 10, the cap;
        // the root then holds 10 and a change of 3 more reaches 13 > 12.
        through_the_pass(&mut operation, scopes[1], Change::Work(Wk::B, 4), true).unwrap();
        assert_eq!(work_usage(&operation, &scopes, Wk::B), [10, 10, 0]);
        assert_eq!(
            through_the_pass(&mut operation, scopes[1], Change::Work(Wk::B, 0), true),
            Ok(())
        );
        assert_eq!(
            through_the_pass(&mut operation, scopes[0], Change::Work(Wk::B, 3), false),
            refusal(
                scopes[0],
                Change::Work(Wk::B, 3),
                10,
                crate::RefusalReason::Limit { limit: 12 }
            )
        );
        assert_eq!(work_usage(&operation, &scopes, Wk::B), [10, 10, 0]);
    }

    /// An addition past the largest count, and a tariff product past it, are
    /// typed refusals that leave every scope as it was.
    #[test]
    fn the_journal_free_pass_refuses_overflow_of_the_sum_and_of_the_tariff() {
        let (mut operation, scopes) = linked(2, 1, uncapped);
        let leaf = scopes[2];
        through_the_pass(
            &mut operation,
            leaf,
            Change::Work(Wk::C, u64::MAX - 1),
            true,
        )
        .unwrap();
        assert_eq!(
            through_the_pass(&mut operation, leaf, Change::Work(Wk::C, 2), false),
            refusal(
                leaf,
                Change::Work(Wk::C, 2),
                u64::MAX - 1,
                crate::RefusalReason::Overflow
            )
        );
        assert_eq!(
            work_usage(&operation, &scopes, Wk::C),
            [u64::MAX - 1, u64::MAX - 1, u64::MAX - 1]
        );
        // A factor of u64::MAX / 3 on 4 overflows when the leaf projects the
        // change into its parent; the leaf's own addition fits and is not kept.
        let (mut operation, scopes) = linked(2, u64::MAX / 3, uncapped);
        let leaf = scopes[2];
        assert_eq!(
            through_the_pass(&mut operation, leaf, Change::Work(Wk::A, 4), false),
            refusal(
                leaf,
                Change::Work(Wk::A, 4),
                0,
                crate::RefusalReason::Overflow
            )
        );
        assert_eq!(work_usage(&operation, &scopes, Wk::A), [0, 0, 0]);
        assert_eq!(work_usage(&operation, &scopes, Wk::B), [0, 0, 0]);
    }

    /// A tariff of scale zero passes a zero amount on, so ancestors keep their
    /// usage and the check against their caps still runs.
    #[test]
    fn the_journal_free_pass_projects_through_a_tariff_of_scale_zero() {
        let caps = |level: usize| match level {
            0 => Limits::from_fn(|_| 0, |_| u64::MAX),
            _ => Limits::unlimited(),
        };
        let (mut operation, scopes) = linked(2, 0, caps);
        through_the_pass(&mut operation, scopes[2], Change::Work(Wk::A, 9), true).unwrap();
        assert_eq!(work_usage(&operation, &scopes, Wk::A), [0, 0, 9]);
        assert_eq!(work_usage(&operation, &scopes, Wk::B), [0, 0, 0]);
    }

    /// A chain of more than four scopes is admitted by the journal, with the
    /// same result; so is a batch of one change when the operation is
    /// profiled, which must observe the admission.
    #[test]
    fn the_journal_free_pass_leaves_long_chains_and_profiled_operations_to_the_journal() {
        // Only the root is capped, at 6 of B: a leaf change of 3 reaches it as 6.
        let caps = |level: usize| match level {
            0 => Limits::from_fn(|_| 6, |_| u64::MAX),
            _ => Limits::unlimited(),
        };
        let (mut operation, scopes) = linked(5, 2, caps);
        let leaf = scopes[5];
        let (before, declined) = (ONE_WORK.get(), ONE_WORK_DECLINED.get());
        operation.apply(leaf, Change::Work(Wk::A, 3)).unwrap();
        // 1 more reaches the root as 2 on 6 held, the sixth hop of the chain.
        assert_eq!(
            operation.apply(leaf, Change::Work(Wk::A, 1)),
            refusal(
                scopes[0],
                Change::Work(Wk::B, 2),
                6,
                crate::RefusalReason::Limit { limit: 6 }
            )
        );
        assert_eq!(
            ONE_WORK.get(),
            before,
            "a chain of six scopes took the pass"
        );
        assert_eq!(ONE_WORK_DECLINED.get(), declined.saturating_add(2));
        assert_eq!(work_usage(&operation, &scopes, Wk::A), [0, 0, 0, 0, 0, 3]);
        assert_eq!(work_usage(&operation, &scopes, Wk::B), [6, 6, 6, 6, 6, 0]);

        let mut profiled = Operation::<Wk, Rs>::with_profile(
            "root",
            Limits::unlimited(),
            1,
            ProfileMode::Summary { timing: false },
        )
        .unwrap();
        let root = profiled.root();
        profiled.apply(root, Change::Work(Wk::A, 1)).unwrap();
        assert_eq!(ONE_WORK.get(), before);
        assert_eq!(ONE_WORK_DECLINED.get(), declined.saturating_add(2));
        let report = profiled.snapshot().unwrap().unwrap();
        assert_eq!(report.scopes[0].accepted_batches, 1);
    }

    /// A parent that names no scope is a latched defect. The pass declines
    /// and leaves usage as it was.
    #[test]
    fn the_journal_free_pass_declines_a_missing_ancestor() {
        let (mut operation, scopes) = linked(2, 1, uncapped);
        operation.scopes[2].parent = Some(Parent {
            index: 9,
            tariffs: Tariffs::identity(),
        });
        let leaf = scopes[2];
        assert_eq!(
            through_the_pass(&mut operation, leaf, Change::Work(Wk::A, 1), false),
            Err(Error::Defect(Defect::ScopeEntry))
        );
        assert_eq!(operation.scopes[2].usage.work(Wk::A), 0);
        assert_latched(operation, Defect::ScopeEntry);
    }

    /// A change that a scope refuses, below an ancestor that names no scope, is
    /// the latched defect the dense admission reports, not the refusal at the
    /// first scope: the pass declines every refusal and the general path reads
    /// the whole chain first.
    #[test]
    fn the_journal_free_pass_declines_a_refusal_below_a_missing_ancestor() {
        let build = || {
            let caps = |level: usize| match level {
                2 => Limits::from_fn(|_| 5, |_| u64::MAX),
                _ => Limits::unlimited(),
            };
            let (mut operation, scopes) = linked(2, 1, caps);
            operation.scopes[2].parent = Some(Parent {
                index: 9,
                tariffs: Tariffs::identity(),
            });
            (operation, scopes[2])
        };
        let change = Change::Work(Wk::A, 6);
        let (mut passing, leaf) = build();
        let result = through_the_pass(&mut passing, leaf, change, false);
        assert_eq!(result, Err(Error::Defect(Defect::ScopeEntry)));
        let (mut dense, twin) = build();
        assert_eq!(
            anonymous(with(Force::Dense, || dense.apply(twin, change))),
            anonymous(result)
        );
        assert_eq!(passing.scopes[2].usage, dense.scopes[2].usage);
        assert_latched(passing, Defect::ScopeEntry);
    }

    /// A chain of four scopes is the longest the pass holds: it commits on the
    /// root's accept and declines a refusal at the root; a chain of five is
    /// declined whatever the outcome, with the same results.
    #[test]
    fn the_journal_free_pass_holds_four_scopes_and_declines_five() {
        let caps = |level: usize| match level {
            0 => Limits::from_fn(|_| 6, |_| u64::MAX),
            _ => Limits::unlimited(),
        };
        for (depth, committed) in [(3, true), (4, false)] {
            let (mut operation, scopes) = linked(depth, 2, caps);
            let leaf = scopes[depth];
            // 3 reaches every ancestor as 6 of B, the root's cap.
            through_the_pass(&mut operation, leaf, Change::Work(Wk::A, 3), committed).unwrap();
            let mut expected = vec![6; depth];
            expected.insert(depth, 0);
            assert_eq!(work_usage(&operation, &scopes, Wk::B), expected);
            // 1 more reaches the root as 2 on 6 held.
            assert_eq!(
                through_the_pass(&mut operation, leaf, Change::Work(Wk::A, 1), false),
                refusal(
                    scopes[0],
                    Change::Work(Wk::B, 2),
                    6,
                    crate::RefusalReason::Limit { limit: 6 }
                )
            );
            assert_eq!(work_usage(&operation, &scopes, Wk::B), expected);
            assert_eq!(
                work_usage(&operation, &scopes, Wk::A)[depth],
                3,
                "depth {depth}"
            );
        }
    }

    /// Single acquisitions and releases are admitted with the short journal:
    /// peaks include the amount reached, and each refusal reason carries the
    /// value the refused scope held.
    #[test]
    fn a_single_resource_change_refuses_with_each_reason_and_keeps_peaks() {
        let caps = |level: usize| match level {
            0 => Limits::from_fn(|_| u64::MAX, |_| 20),
            _ => Limits::unlimited(),
        };
        let (mut operation, scopes) = linked(2, 2, caps);
        let leaf = scopes[2];
        // A factor of 2 doubles the amount at each level: 3, 6 and 12.
        operation.apply(leaf, Change::Acquire(Rs::X, 3)).unwrap();
        let live: Vec<_> = scopes
            .iter()
            .map(|&scope| operation.usage(scope).unwrap().live(Rs::X))
            .collect();
        assert_eq!(live, [12, 6, 3]);
        // 3 more reach the root as 12 on 12 held: 24 exceeds the cap of 20.
        assert_eq!(
            operation.apply(leaf, Change::Acquire(Rs::X, 3)),
            refusal(
                scopes[0],
                Change::Acquire(Rs::X, 12),
                12,
                crate::RefusalReason::Limit { limit: 20 }
            )
        );
        // Releasing more than the leaf holds underflows at the origin's direct
        // custody, which holds 3.
        assert_eq!(
            operation.apply(leaf, Change::Release(Rs::X, 4)),
            refusal(
                leaf,
                Change::Release(Rs::X, 4),
                3,
                crate::RefusalReason::Underflow
            )
        );
        operation.apply(leaf, Change::Release(Rs::X, 3)).unwrap();
        for (scope, peak) in scopes.iter().zip([12, 6, 3]) {
            let usage = operation.usage(*scope).unwrap();
            assert_eq!((usage.live(Rs::X), usage.peak(Rs::X)), (0, peak));
        }
    }
}
