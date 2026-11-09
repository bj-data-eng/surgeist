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
        Operation, Scope, Tariff,
        in_place::{FORCE, Force},
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
    fn operation(seed: u64) -> (Operation<Wk, Rs>, Vec<Scope>) {
        let random = Random(std::cell::Cell::new(seed));
        let limit_set = |random: &Random| Limits::from_fn(|_| cap(random), |_| cap(random));
        let mut operation = Operation::new("root", limit_set(&random), 6).unwrap();
        let mut scopes = vec![operation.root()];
        for _ in 0..random.below(5) {
            let parent = scopes[random.below(scopes.len() as u64) as usize];
            let limits = limit_set(&random);
            let tariffs = Tariffs::from_fn(
                |_| Tariff::new(work(&random), scale(&random)),
                |_| Tariff::new(resource(&random), scale(&random)),
            );
            scopes.push(operation.child(parent, "child", limits, tariffs).unwrap());
        }
        (operation, scopes)
    }

    fn batch(random: &Random) -> Vec<Change<Wk, Rs>> {
        let amount = |random: &Random| match random.below(10) {
            0 => u64::MAX / 2,
            1 => 0,
            _ => random.below(9),
        };
        (0..random.below(11))
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

    /// Every batch is accepted or refused the same way by the in-place store
    /// and by the dense store, and leaves the same usage, peaks and direct
    /// custody, including batches too large for the journal.
    #[test]
    fn the_in_place_and_dense_stores_agree_on_random_batches() {
        let mut accepted = 0;
        let mut refused = 0;
        for seed in 1..=400_u64 {
            let (mut fast, scopes) = operation(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
            let (mut slow, slow_scopes) = operation(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
            let random = Random(std::cell::Cell::new(
                seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1,
            ));
            for _ in 0..40 {
                let position = random.below(scopes.len() as u64) as usize;
                let (scope, twin) = (scopes[position], slow_scopes[position]);
                let changes = batch(&random);
                assert_eq!(
                    anonymous(with(Force::InPlace, || fast.check_batch(scope, &changes))),
                    anonymous(with(Force::Dense, || slow.check_batch(twin, &changes))),
                    "seed {seed}: check"
                );
                let result = with(Force::InPlace, || fast.apply_batch(scope, &changes));
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
        }
        assert!(accepted > 1_000 && refused > 1_000, "{accepted} {refused}");
    }
}
