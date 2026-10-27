//! Public consumer evidence for exclusive private preparation and real publication.
use std::sync::{Arc, Weak};
use surgeist_css::property::CssPropertyGrammar;
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::rule::CssSheet;
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_css::{CssParser, CssParserContext};
use surgeist_cssom::*;

fn version(role: CssomInputRole, name: &str, revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: name.into(),
        revision,
    }
}
fn store(limits: CssomLimits) -> CssomStore {
    CssomStore::new(limits, CssomContext::default()).unwrap()
}
fn guard(store: &CssomStore) -> CssomEditGuard {
    CssomEditGuard::from_snapshot(&store.snapshot())
}
fn batch(store: &CssomStore) -> CssomBatch {
    store.batch(guard(store)).unwrap()
}
fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::constructed(
        version(CssomInputRole::Document, "document", 1),
        "document".into(),
        None,
    )
}
fn sheet(store: &mut CssomStore) -> CssomSheetId {
    let mut edit = batch(store);
    let ticket = edit.create_parsed_sheet("a {}", inputs()).unwrap();
    store.commit(edit).unwrap().sheet(&ticket).unwrap().clone()
}
fn assert_unchanged(store: &CssomStore, before: &CssomSnapshot) {
    let after = store.snapshot();
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.inputs(), before.inputs());
    assert_eq!(after.sheets(), before.sheets());
    assert!(matches!(
        store.changes_since(before),
        CssomChanges::Unchanged
    ));
}
fn changed(commit: &CssomCommit) -> &CssomChangeSummary {
    match commit.publication() {
        CssomPublication::Changed(summary) => summary,
        CssomPublication::Unchanged { .. } => panic!("expected real changed publication"),
    }
}

#[test]
fn prepared_report_publishes_real_tickets_diagnostics_source_and_summary() {
    let mut owner = store(CssomLimits::default());
    let before = owner.snapshot();
    let raw = Arc::new(
        "@unknown x;\r\na { color: red }"
            .encode_utf16()
            .collect::<Vec<_>>(),
    );
    let report = CssParser::new(CssParserContext::default(), Default::default())
        .parse_sheet_utf16(raw.clone(), CssUtf16InputLimits::default(), || {
            Ok::<_, ()>(())
        })
        .unwrap();
    let source = report.syntax().parse_source().unwrap().source().clone();
    let diagnostics = report.diagnostics().to_vec();
    assert!(!diagnostics.is_empty());
    let mut edit = batch(&owner);
    let ticket = edit.create_parsed_sheet_report(report, inputs()).unwrap();
    let prepared = owner.prepare_commit(edit).unwrap();
    assert!(
        before.sheets().is_empty(),
        "independent old captures remain readable"
    );
    let (commit, retirement) = prepared.publish();
    let id = commit.sheet(&ticket).unwrap();
    let after = owner.snapshot();
    assert_eq!(after.sheets(), std::slice::from_ref(id));
    let authored = after.sheet(id).unwrap();
    assert_eq!(authored.diagnostics(), diagnostics);
    assert!(
        authored
            .authored()
            .parse_source()
            .unwrap()
            .source()
            .same_snapshot(&source)
    );
    assert!(Arc::ptr_eq(
        authored
            .authored()
            .parse_source()
            .unwrap()
            .source()
            .original_utf16()
            .unwrap(),
        &raw
    ));
    let summary = changed(&commit);
    assert_eq!(summary.from(), before.revision());
    assert_eq!(summary.to(), after.revision());
    assert_eq!(summary.owner(), after.owner());
    assert_eq!(summary.before_inputs(), &before.inputs());
    assert_eq!(summary.after_inputs(), &after.inputs());
    assert!(summary.categories().contains(&CssomChange::Provenance));
    assert!(
        summary
            .affected()
            .contains(&CssomObjectId::Sheet(id.clone()))
    );
    assert!(commit.owner_effects().is_empty());
    drop(owner);
    drop(retirement);
    assert_eq!(after.sheet(id).unwrap().diagnostics(), diagnostics);
}

#[test]
fn dropping_preparation_keeps_state_history_and_tickets_private() {
    let mut owner = store(CssomLimits {
        summary_history: 2,
        ..Default::default()
    });
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    let abandoned_ticket = edit.create_parsed_sheet("abandoned {}", inputs()).unwrap();
    drop(owner.prepare_commit(edit).unwrap());
    assert_unchanged(&owner, &before);
    let mut retry = batch(&owner);
    let real_ticket = retry.create_parsed_sheet("real {}", inputs()).unwrap();
    let (commit, retirement) = owner.prepare_commit(retry).unwrap().publish();
    assert!(matches!(
        commit.sheet(&abandoned_ticket),
        Err(CssomError::ForeignTicket)
    ));
    assert_eq!(
        owner.snapshot().sheets(),
        std::slice::from_ref(commit.sheet(&real_ticket).unwrap())
    );
    assert_eq!(owner.snapshot().revision().value(), 1);
    drop(retirement);
}

#[test]
fn equal_restored_and_empty_batches_remain_unchanged_without_history_rows() {
    let mut owner = store(CssomLimits {
        summary_history: 1,
        ..Default::default()
    });
    let id = sheet(&mut owner);
    let before = owner.snapshot();
    for restore in [false, true] {
        let mut edit = batch(&owner);
        if restore {
            assert!(edit.set_disabled(&id, true).unwrap());
            assert!(edit.set_disabled(&id, false).unwrap());
        }
        let (commit, retirement) = owner.prepare_commit(edit).unwrap().publish();
        assert!(
            matches!(commit.publication(), CssomPublication::Unchanged { revision } if revision == before.revision())
        );
        assert!(commit.owner_effects().is_empty());
        drop(retirement);
        assert_unchanged(&owner, &before);
    }
}

#[test]
fn later_owner_failure_abandons_earlier_preparation_without_publishing_either() {
    let mut first = store(CssomLimits::default());
    let mut second = store(CssomLimits {
        max_revision: 1,
        ..Default::default()
    });
    let a = sheet(&mut first);
    let b = sheet(&mut second);
    let before_a = first.snapshot();
    let before_b = second.snapshot();
    let mut edit_a = batch(&first);
    edit_a.set_disabled(&a, true).unwrap();
    let mut edit_b = batch(&second);
    edit_b.set_disabled(&b, true).unwrap();
    let prepared_a = first.prepare_commit(edit_a).unwrap();
    assert!(matches!(
        second.prepare_commit(edit_b),
        Err(CssomError::RevisionExhausted)
    ));
    drop(prepared_a);
    assert_unchanged(&first, &before_a);
    assert_unchanged(&second, &before_b);
    assert!(!first.snapshot().sheet(&a).unwrap().inputs().disabled);
    assert!(!second.snapshot().sheet(&b).unwrap().inputs().disabled);
    let mut retry = batch(&first);
    retry.set_disabled(&a, true).unwrap();
    let (commit, retirement) = first.prepare_commit(retry).unwrap().publish();
    assert!(changed(&commit).supports_incremental(&before_a, &first.snapshot()));
    drop(retirement);
}

#[test]
fn history_off_on_and_bounded_eviction_preserve_contiguous_coverage() {
    for capacity in [0, 1, 2] {
        let mut owner = store(CssomLimits {
            summary_history: capacity,
            ..Default::default()
        });
        let id = sheet(&mut owner);
        let before = owner.snapshot();
        let mut edit = batch(&owner);
        edit.set_disabled(&id, true).unwrap();
        let (first, retired_first) = owner.prepare_commit(edit).unwrap().publish();
        let middle = owner.snapshot();
        assert!(changed(&first).supports_incremental(&before, &middle));
        let mut edit = batch(&owner);
        edit.set_disabled(&id, false).unwrap();
        let (second, retired_second) = owner.prepare_commit(edit).unwrap().publish();
        let after = owner.snapshot();
        assert!(changed(&second).supports_incremental(&middle, &after));
        match owner.changes_since(&before) {
            CssomChanges::Incremental(summary) if capacity == 2 => {
                assert!(summary.supports_incremental(&before, &after))
            }
            CssomChanges::FullRecompute(CssomRecomputeReason::CoverageGap) if capacity < 2 => {}
            _ => panic!("history capacity {capacity} returned incorrect coverage"),
        }
        if capacity != 0 {
            assert!(matches!(
                owner.changes_since(&middle),
                CssomChanges::Incremental(_)
            ));
        }
        drop((retired_first, retired_second));
    }
}

fn linked_source() -> (CssomContext, Weak<Vec<u16>>) {
    let original = Arc::new(
        "/* retained original input */"
            .encode_utf16()
            .collect::<Vec<_>>(),
    );
    let weak = Arc::downgrade(&original);
    let report = CssParser::new(CssParserContext::default(), Default::default())
        .parse_sheet_utf16(original, CssUtf16InputLimits::default(), || Ok::<_, ()>(()))
        .unwrap();
    let mut source = store(CssomLimits {
        summary_history: 0,
        ..Default::default()
    });
    let mut edit = batch(&source);
    let ticket = edit.create_parsed_sheet_report(report, inputs()).unwrap();
    let id = source.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let context = CssomContext {
        inputs: Vec::new(),
        linked: vec![CssomLinkedInput {
            version: version(CssomInputRole::Import, "linked", 1),
            snapshot: source.snapshot(),
            sheet: id,
        }],
    };
    drop(source);
    (context, weak)
}

#[test]
fn retirement_keeps_old_canonical_linked_backing_outside_publication_tail() {
    let (context, source) = linked_source();
    let mut owner = CssomStore::new(
        CssomLimits {
            summary_history: 0,
            ..Default::default()
        },
        context,
    )
    .unwrap();
    let mut edit = batch(&owner);
    edit.update_context(CssomContext::default()).unwrap();
    let (commit, retirement) = owner.prepare_commit(edit).unwrap().publish();
    assert!(owner.snapshot().context().linked.is_empty());
    drop(commit); // the actual before-input manifest also retains the old source
    assert!(
        source.upgrade().is_some(),
        "retired State still owns its linked snapshot"
    );
    drop(retirement);
    assert!(source.upgrade().is_none());
}

#[test]
fn retirement_keeps_evicted_history_manifest_backing_until_caller_disposal() {
    let (context, source) = linked_source();
    let mut owner = CssomStore::new(
        CssomLimits {
            summary_history: 1,
            ..Default::default()
        },
        context,
    )
    .unwrap();
    let mut edit = batch(&owner);
    edit.update_context(CssomContext::default()).unwrap();
    drop(owner.commit(edit).unwrap());
    assert!(
        source.upgrade().is_some(),
        "one actual history row retains before-input provenance"
    );
    let mut edit = batch(&owner);
    edit.update_context(CssomContext {
        inputs: vec![CssomInput {
            version: version(CssomInputRole::Origin, "next", 2),
            data: CssomInputData::Origin {
                origin: "next".into(),
                profile: "author".into(),
            },
        }],
        linked: Vec::new(),
    })
    .unwrap();
    let (commit, retirement) = owner.prepare_commit(edit).unwrap().publish();
    drop(commit);
    assert!(
        source.upgrade().is_some(),
        "evicted row moved to retirement, not destroyed inside publish"
    );
    drop(retirement);
    assert!(source.upgrade().is_none());
}

#[test]
fn pending_replacement_survives_prepare_drop_and_terminal_revision_credits() {
    let mut owner = store(CssomLimits {
        max_revision: 6,
        summary_history: 1,
        ..Default::default()
    });
    let a = sheet(&mut owner);
    let b = sheet(&mut owner);
    let job_a = owner
        .begin_replace_sheet(guard(&owner), &a, "a {color:red}")
        .unwrap();
    let pending_a = owner.snapshot();
    let mut edit = batch(&owner);
    edit.set_disabled(&a, true).unwrap();
    drop(owner.prepare_commit(edit).unwrap());
    assert_unchanged(&owner, &pending_a);
    assert_eq!(
        owner.pending_sheet_replacement(&a).unwrap(),
        Some(job_a.token())
    );
    let job_b = owner
        .begin_replace_sheet(guard(&owner), &b, "b {}")
        .unwrap();
    let pending = owner.snapshot();
    let noop = batch(&owner);
    let (commit, retirement) = owner.prepare_commit(noop).unwrap().publish();
    assert!(matches!(
        commit.publication(),
        CssomPublication::Unchanged { .. }
    ));
    drop(retirement);
    let mut edit = batch(&owner);
    edit.set_disabled(&a, true).unwrap();
    assert!(matches!(
        owner.prepare_commit(edit),
        Err(CssomError::RevisionExhausted)
    ));
    assert_unchanged(&owner, &pending);
    assert_eq!(
        owner.pending_sheet_replacement(&a).unwrap(),
        Some(job_a.token())
    );
    assert_eq!(
        owner.pending_sheet_replacement(&b).unwrap(),
        Some(job_b.token())
    );
    assert!(matches!(
        owner.cancel_replace_sheet(job_a.token()).unwrap().outcome(),
        CssomReplaceOutcome::Cancelled
    ));
    assert!(owner.pending_sheet_replacement(&a).unwrap().is_none());
    let completion = owner
        .finish_replace_sheet(guard(&owner), job_b.parse())
        .unwrap();
    assert!(matches!(
        completion.outcome(),
        CssomReplaceOutcome::Replaced
    ));
    assert!(owner.pending_sheet_replacement(&b).unwrap().is_none());
    assert_eq!(owner.snapshot().revision().value(), 6);
    assert!(
        !owner
            .snapshot()
            .sheet(&a)
            .unwrap()
            .inputs()
            .disallow_modification
    );
    assert!(
        !owner
            .snapshot()
            .sheet(&b)
            .unwrap()
            .inputs()
            .disallow_modification
    );
}

#[path = "prepared_commit/effects.rs"]
mod effects;

#[path = "prepared_commit/created_sheet.rs"]
mod created_sheet;
