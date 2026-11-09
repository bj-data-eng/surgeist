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
