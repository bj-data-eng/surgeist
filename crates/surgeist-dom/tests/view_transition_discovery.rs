#![forbid(unsafe_code)]

use std::sync::atomic::AtomicBool;

use surgeist_css::{CssCustomIdent, CssViewTransitionIdent};
use surgeist_cssom::{
    CssomContext, CssomEditGuard, CssomInputRole, CssomInputVersion, CssomLimits, CssomSheetInputs,
    CssomStore,
};
use surgeist_dom::{
    DocumentId, Dom, DomBatch, DomChange, DomCommit, DomDocumentStamp, DomError, DomLimits,
    DomPreparedViewTransitionSheet, DomSourceCompletion, DomSourceEdit, DomSourceLimits,
    DomSourceOutput, DomSourceProgress, DomViewTransitionBranches, DomViewTransitionCaptureLimits,
    DomViewTransitionId, DomViewTransitionRecordId, NodeId,
};

const LOCAL_WORK: u64 = 20_000;
const UNRELATED: [usize; 3] = [0, 64, 2048];

fn name(value: &str) -> CssViewTransitionIdent {
    CssViewTransitionIdent::try_new(CssCustomIdent::try_new(value).unwrap()).unwrap()
}

fn basis(dom: &Dom, document: DocumentId) -> DomDocumentStamp {
    dom.snapshot().document(document).unwrap().stamp()
}

fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, LOCAL_WORK, 8, 64, 64, 1)
}

#[derive(Debug)]
enum Entry {
    Begin,
    Advance,
}

fn source(dom: &mut Dom, edit: DomSourceEdit) -> Result<DomSourceCompletion, (Entry, DomError)> {
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(edit, source_limits(), &cancel)
        .map_err(|error| (Entry::Begin, error))?;
    match lease
        .advance(&cancel)
        .map_err(|error| (Entry::Advance, error))?
    {
        DomSourceProgress::Complete(done) => Ok(done),
        DomSourceProgress::Request(request) => panic!("unexpected VT service: {request:?}"),
    }
}

struct Fixture {
    dom: Dom,
    document: DocumentId,
    transition: DomViewTransitionId,
    target: NodeId,
    spare: NodeId,
}

fn fixture(unrelated: usize) -> Fixture {
    let limits = DomLimits::new(
        16,
        1_000_000,
        4096,
        0,
        0,
        16,
        64_000_000,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_view_transitions(4, 4096, 16_384)
    .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let target = batch.create_element(&document, "target");
    let spare = batch.create_element(&document, "spare");
    batch.insert_before(&root, &document, None);
    batch.insert_before(&target, &root, None);
    batch.insert_before(&spare, &root, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let spare = commit.created(&spare).unwrap().unwrap();

    let mut css = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut css_batch = css
        .batch(CssomEditGuard::from_snapshot(&css.snapshot()))
        .unwrap();
    let sheet = css_batch
        .create_parsed_sheet(
            "",
            CssomSheetInputs::constructed(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "VT discovery fixture".into(),
                    revision: 1,
                },
                "VT discovery fixture".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = css
        .commit(css_batch)
        .unwrap()
        .sheet(&sheet)
        .unwrap()
        .clone();
    let prepared = DomPreparedViewTransitionSheet::capture(
        &css.snapshot(),
        &sheet,
        DomViewTransitionCaptureLimits::new(16_384, 1_000_000),
        &AtomicBool::new(false),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    batch.initialize_view_transitions(basis(&dom, document), prepared);
    apply(&mut dom, batch);

    let mut batch = DomBatch::new();
    let stamp = basis(&dom, document);
    let transition = batch.begin_view_transition(stamp);
    for index in 0..unrelated {
        // Every unrelated name and every measured incoming name has eight
        // UTF-8 bytes; increasing records does not increase edited payload.
        batch.append_view_transition_record(stamp, &transition, name(&format!("r{index:07}")));
    }
    let transition = apply(&mut dom, batch)
        .view_transition(&transition)
        .unwrap()
        .unwrap();
    Fixture {
        dom,
        document,
        transition,
        target,
        spare,
    }
}

fn append(
    f: &mut Fixture,
    value: &str,
    branches: DomViewTransitionBranches,
) -> DomViewTransitionRecordId {
    let mut batch = DomBatch::new();
    let stamp = basis(&f.dom, f.document);
    let record = batch.append_view_transition_record(stamp, f.transition, name(value));
    batch.set_view_transition_branches(stamp, &record, branches);
    apply(&mut f.dom, batch)
        .view_transition_record(&record)
        .unwrap()
        .unwrap()
}

#[test]
fn absent_name_source_append_fits_fixed_work_as_unrelated_records_grow() {
    for unrelated in UNRELATED {
        let mut f = fixture(unrelated);
        let before = f.dom.snapshot();
        let units = f.dom.usage().string_code_units();
        let done = source(
            &mut f.dom,
            DomSourceEdit::append_view_transition_record(
                before.document(f.document).unwrap().stamp(),
                f.transition,
                name("fresh___"),
            ),
        )
        .unwrap_or_else(|error| {
            panic!("unrelated={unrelated}, fixed work={LOCAL_WORK}: {error:?}")
        });
        let Some(DomSourceOutput::ViewTransitionRecord(record)) = done.output() else {
            panic!("actual append must publish its record: {done:?}")
        };
        assert_eq!(record.transition(), f.transition);
        assert_eq!(
            done.commit().changes().changes(),
            &[
                DomChange::ViewTransition(f.transition),
                DomChange::ViewTransitionRecord(record)
            ]
        );
        let after = f.dom.snapshot();
        let value = after.view_transition_record(record).unwrap();
        assert_eq!(value.name().as_str(), "fresh___");
        assert_eq!(value.branches(), DomViewTransitionBranches::Pending);
        assert!(value.rules().is_none());
        assert_eq!(
            after
                .view_transition(f.transition)
                .unwrap()
                .records()
                .count(),
            unrelated + 1
        );
        assert_eq!(
            before
                .view_transition(f.transition)
                .unwrap()
                .records()
                .count(),
            unrelated
        );
        for (index, record) in before
            .view_transition(f.transition)
            .unwrap()
            .records()
            .enumerate()
        {
            assert_eq!(record.name().as_str(), format!("r{index:07}"));
            assert_eq!(record.branches(), DomViewTransitionBranches::Pending);
        }
        assert_eq!(f.dom.usage().string_code_units(), units + 8);
        assert!(f.dom.pending_source_edit().is_none());
    }
}

#[test]
fn exact_duplicate_name_rejects_with_fixed_work_after_unrelated_records() {
    for unrelated in UNRELATED {
        let mut f = fixture(unrelated);
        let needle = append(&mut f, "needle__", DomViewTransitionBranches::Old);
        let before = f.dom.snapshot();
        let usage = f.dom.usage();
        let error = source(
            &mut f.dom,
            DomSourceEdit::append_view_transition_record(
                before.document(f.document).unwrap().stamp(),
                f.transition,
                name("needle__"),
            ),
        )
        .unwrap_err();
        assert_eq!(
            error.1,
            DomError::DuplicateViewTransitionName,
            "unrelated={unrelated}, entry={:?}",
            error.0
        );
        assert_eq!(f.dom.snapshot().stamp(), before.stamp());
        assert_eq!(f.dom.usage(), usage);
        assert!(f.dom.pending_source_edit().is_none());
        assert_eq!(
            before
                .view_transition_record(needle)
                .unwrap()
                .name()
                .as_str(),
            "needle__"
        );
        assert_eq!(
            f.dom
                .snapshot()
                .view_transition(f.transition)
                .unwrap()
                .records()
                .count(),
            unrelated + 1
        );
        assert!(
            f.dom
                .changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
}

#[test]
fn css_names_keep_case_prefix_and_unicode_identity() {
    let mut f = fixture(64);
    for value in ["one", "One", "x", "xy", "é", "e\u{301}", "𐐀x"] {
        append(&mut f, value, DomViewTransitionBranches::Old);
    }
    for value in ["one", "One", "x", "xy", "é", "e\u{301}", "𐐀x"] {
        let before = f.dom.snapshot();
        let error = source(
            &mut f.dom,
            DomSourceEdit::append_view_transition_record(
                before.document(f.document).unwrap().stamp(),
                f.transition,
                name(value),
            ),
        )
        .unwrap_err();
        assert_eq!(
            error.1,
            DomError::DuplicateViewTransitionName,
            "{value:?}: {error:?}"
        );
        assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    }
}

#[test]
fn new_element_claims_preserve_duplicate_self_noop_and_reassignment_semantics() {
    for unrelated in UNRELATED {
        let mut f = fixture(unrelated);
        let chosen = append(&mut f, "chosen__", DomViewTransitionBranches::Old);
        let target = f.target;
        let claimed = append(&mut f, "claimed_", DomViewTransitionBranches::New(target));
        let before = f.dom.snapshot();
        let edit = DomSourceEdit::set_view_transition_branches(
            basis(&f.dom, f.document),
            chosen,
            DomViewTransitionBranches::New(f.spare),
        );
        let done = source(&mut f.dom, edit).unwrap();
        assert_eq!(done.output(), None);
        assert_eq!(
            done.commit().changes().changes(),
            &[
                DomChange::ViewTransition(f.transition),
                DomChange::ViewTransitionRecord(chosen)
            ]
        );
        assert_eq!(
            before.view_transition_record(chosen).unwrap().branches(),
            DomViewTransitionBranches::Old
        );
        let unchanged = f.dom.snapshot();
        let usage = f.dom.usage();
        let edit = DomSourceEdit::set_view_transition_branches(
            basis(&f.dom, f.document),
            chosen,
            DomViewTransitionBranches::New(f.spare),
        );
        let done = source(&mut f.dom, edit).unwrap();
        assert!(done.commit().changes().changes().is_empty());
        assert_eq!(f.dom.snapshot().stamp(), unchanged.stamp());
        assert_eq!(f.dom.usage(), usage);
        let edit = DomSourceEdit::set_view_transition_branches(
            basis(&f.dom, f.document),
            chosen,
            DomViewTransitionBranches::OldAndNew(target),
        );
        let error = source(&mut f.dom, edit).unwrap_err();
        assert_eq!(error.1, DomError::InvalidViewTransitionState);
        assert_eq!(f.dom.snapshot().stamp(), unchanged.stamp());
        assert_eq!(f.dom.usage(), usage);
        let mut batch = DomBatch::new();
        let stamp = basis(&f.dom, f.document);
        batch.set_view_transition_branches(stamp, chosen, DomViewTransitionBranches::Old);
        batch.set_view_transition_branches(stamp, claimed, DomViewTransitionBranches::Old);
        batch.set_view_transition_branches(stamp, chosen, DomViewTransitionBranches::New(target));
        apply(&mut f.dom, batch);
        assert_eq!(
            f.dom
                .snapshot()
                .view_transition_record(chosen)
                .unwrap()
                .branches(),
            DomViewTransitionBranches::New(target)
        );
        assert_eq!(
            unchanged
                .view_transition_record(claimed)
                .unwrap()
                .branches(),
            DomViewTransitionBranches::New(target)
        );
        assert!(f.dom.pending_source_edit().is_none());
    }
}

#[test]
fn historical_element_claim_does_not_claim_a_replacement_identity() {
    let mut f = fixture(64);
    let target = f.target;
    let historical = append(&mut f, "oldclaim", DomViewTransitionBranches::New(target));
    let chosen = append(&mut f, "chosen__", DomViewTransitionBranches::Old);
    let before = f.dom.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(target);
    batch.destroy(target);
    let replacement = batch.create_element(f.document, "replacement");
    let replacement = apply(&mut f.dom, batch)
        .created(&replacement)
        .unwrap()
        .unwrap();
    assert_ne!(replacement, target);
    let edit = DomSourceEdit::set_view_transition_branches(
        basis(&f.dom, f.document),
        chosen,
        DomViewTransitionBranches::New(replacement),
    );
    source(&mut f.dom, edit).unwrap();
    let after = f.dom.snapshot();
    assert_eq!(
        after.view_transition_record(historical).unwrap().branches(),
        DomViewTransitionBranches::New(target)
    );
    assert_eq!(
        after
            .view_transition_record(historical)
            .unwrap()
            .resolve_new_element()
            .unwrap_err(),
        DomError::StaleNode
    );
    assert_eq!(
        after.view_transition_record(chosen).unwrap().branches(),
        DomViewTransitionBranches::New(replacement)
    );
    assert_eq!(
        before
            .view_transition_record(historical)
            .unwrap()
            .resolve_new_element()
            .unwrap()
            .unwrap()
            .id(),
        target
    );
}
