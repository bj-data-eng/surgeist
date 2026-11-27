//! The work a commit's checks do depends on what the commit changed, not on
//! how many rules the store holds.
//!
//! One sheet of `rules` style rules is committed to a store; a batch then
//! changes it and commits. The objects the census examines during that batch
//! and its commit are read from the census probe, which counts at the point a
//! record is examined.
use super::probe;
use crate::*;
use std::sync::atomic::AtomicBool;

/// The rule counts the work is compared at; each doubles the previous one.
const SIZES: [usize; 3] = [100, 200, 400];

fn version(revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: "edit-work".into(),
        revision,
    }
}

fn guard(store: &CssomStore) -> CssomEditGuard {
    CssomEditGuard::from_snapshot(&store.snapshot())
}

/// A store holding one external sheet, owned by the element `"edit-work"`, of
/// `rules` style rules, and that sheet.
fn store_with_rules(rules: usize) -> (CssomStore, CssomSheetId) {
    let text = (0..rules)
        .map(|index| format!(".r{index} {{ color: red; margin: 1px 0 }}\n"))
        .collect::<String>();
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = store.batch(guard(&store)).unwrap();
    let mut inputs = CssomSheetInputs::external(version(1), true);
    inputs.owner = Some("edit-work".into());
    let ticket = edit.create_parsed_sheet(&text, inputs).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    (store, sheet)
}

/// The objects the census examines while `edit` changes the sheet of a store of
/// `rules` rules in a batch and the batch commits.
fn examined(
    rules: usize,
    edit: impl FnOnce(&mut CssomBatch, &CssomSheetId) -> Result<(), CssomError>,
) -> usize {
    let (mut store, sheet) = store_with_rules(rules);
    probe::reset();
    let mut batch = store.batch(guard(&store)).unwrap();
    edit(&mut batch, &sheet).unwrap();
    let commit = store.commit(batch).unwrap();
    let work = probe::read();
    assert!(matches!(commit.publication(), CssomPublication::Changed(_)));
    work.examined
}

fn owner_attributes(
    batch: &mut CssomBatch,
    sheet: &CssomSheetId,
    revision: u64,
) -> Result<(), CssomError> {
    batch
        .update_sheet_owner_attributes(
            sheet,
            CssomSheetOwnerAttributes {
                owner: "edit-work",
                version: version(revision),
                title: None,
                media: CssomOwnerMediaInput::Utf8(None),
            },
        )
        .map(drop)
}

/// One sheet record changes, so a bounded number of objects is examined however
/// many rules the store holds.
const ONE_RECORD: usize = 8;

#[test]
fn disabling_a_sheet_examines_the_same_few_objects_whatever_the_rule_count() {
    let counts = SIZES.map(|rules| {
        examined(rules, |batch, sheet| {
            assert!(batch.set_disabled(sheet, true)?);
            Ok(())
        })
    });
    assert!(
        counts.iter().all(|count| *count == counts[0]),
        "objects examined by one set_disabled commit at {SIZES:?} rules: {counts:?}"
    );
    assert!(
        counts[0] <= ONE_RECORD,
        "objects examined by one set_disabled commit at {SIZES:?} rules: {counts:?}"
    );
}

/// The operations of a batch of eight: the sheet's flag and its owner
/// attributes alternate, and one rule is inserted.
fn eight_operations(batch: &mut CssomBatch, sheet: &CssomSheetId) -> Result<(), CssomError> {
    batch.set_disabled(sheet, true)?;
    owner_attributes(batch, sheet, 2)?;
    batch.set_disabled(sheet, false)?;
    owner_attributes(batch, sheet, 3)?;
    batch.insert_sheet_rule(sheet, ".added { color: blue }", 0, &AtomicBool::new(false))?;
    batch.set_disabled(sheet, true)?;
    owner_attributes(batch, sheet, 4)?;
    batch.set_disabled(sheet, false)?;
    Ok(())
}

/// Each operation of a batch examines the records it changes: the sheet, and for
/// an inserted rule the rule, its block and its lists.
const PER_OPERATION: usize = 12;

#[test]
fn a_batch_of_eight_operations_examines_objects_in_proportion_to_its_operations() {
    let counts = [100, 400].map(|rules| examined(rules, eight_operations));
    assert_eq!(
        counts[0], counts[1],
        "objects examined by eight operations at 100 and 400 rules: {counts:?}"
    );
    assert!(
        counts[0] <= 8 * PER_OPERATION,
        "objects examined by eight operations at 100 and 400 rules: {counts:?}"
    );
}

/// Opens a batch on a store of `rules` rules, runs `edit` and commits it.
fn committed(
    rules: usize,
    edit: impl FnOnce(&mut CssomBatch, &CssomSheetId) -> Result<(), CssomError>,
) -> (CssomStore, CssomSheetId, CssomCommit) {
    let (mut store, sheet) = store_with_rules(rules);
    let mut batch = store.batch(guard(&store)).unwrap();
    probe::reset();
    edit(&mut batch, &sheet).unwrap();
    let commit = store.commit(batch).unwrap();
    (store, sheet, commit)
}

#[test]
fn a_commit_builds_no_input_manifest_from_the_whole_state() {
    for rules in [100, 400] {
        let (..) = committed(rules, |batch, sheet| {
            assert!(batch.set_disabled(sheet, true)?);
            Ok(())
        });
        assert_eq!(
            probe::manifests(),
            0,
            "input manifests built from every record by one set_disabled commit at {rules} rules"
        );
    }
    let (..) = committed(100, eight_operations);
    assert_eq!(
        probe::manifests(),
        0,
        "input manifests built from every record by a batch of eight operations"
    );
}

#[test]
fn a_commit_that_changes_no_input_row_publishes_one_manifest_as_before_and_after() {
    let (store, _, commit) = committed(100, |batch, sheet| {
        assert!(batch.set_disabled(sheet, true)?);
        Ok(())
    });
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("a changed commit")
    };
    // The flag is no input: the manifest the commit leaves is the one it started from,
    // so the summary holds one value for both.
    assert!(std::ptr::eq(
        summary.before_inputs(),
        summary.after_inputs()
    ));
    assert_eq!(summary.after_inputs(), &store.snapshot().inputs());
}
