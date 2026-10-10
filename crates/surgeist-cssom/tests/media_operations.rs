#![forbid(unsafe_code)]
//! Public consumer expectations from selected CSSOM §4.4 MediaList algorithms.
//! The CSS writer's historical modern-value lexical policy remains selected.
use surgeist_css::*;
use surgeist_cssom::*;

fn limits() -> CssSpecifiedValueSerializationLimits {
    CssSpecifiedValueSerializationLimits::default()
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn input() -> CssomSheetInputs {
    CssomSheetInputs::external(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "supplied document".into(),
            revision: 1,
        },
        false,
    )
}
fn create(store: &mut CssomStore, source: &str, inputs: CssomSheetInputs) -> CssomSheetId {
    let mut edit = batch(store);
    let ticket = edit.create_parsed_sheet(source, inputs).unwrap();
    store.commit(edit).unwrap().sheet(&ticket).unwrap().clone()
}
fn new_store() -> CssomStore {
    CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap()
}
fn text(snapshot: &CssomSnapshot, id: &CssomMediaListId) -> String {
    snapshot.media_text(id, limits()).unwrap()
}
fn unchanged(commit: &CssomCommit) {
    assert!(matches!(
        commit.publication(),
        CssomPublication::Unchanged { .. }
    ));
}

#[test]
fn current_associations_share_stable_media_ids_without_sheet_guards() {
    let mut store = new_store();
    let mut inputs = input();
    inputs.disallow_modification = true;
    let sheet = create(
        &mut store,
        "@import 'child.css' screen; @media print {}",
        inputs,
    );
    let before = store.snapshot();
    let sheet_media = before.sheet(&sheet).unwrap().media().clone();
    let rules = before.rules(before.sheet(&sheet).unwrap().rules()).unwrap();
    let import = rules[0].clone();
    let group = rules[1].clone();
    let import_media = match before.rule(&import).unwrap().data() {
        CssomRuleData::Import { media, .. } => media.clone(),
        other => panic!("import expected: {other:?}"),
    };
    let group_media = match before.rule(&group).unwrap().data() {
        CssomRuleData::Group {
            prelude: CssomGroupPrelude::Media(media),
            ..
        } => media.clone(),
        other => panic!("media group expected: {other:?}"),
    };
    let mut edit = batch(&store);
    for (id, value) in [
        (&sheet_media, "speech"),
        (&import_media, "print"),
        (&group_media, "screen"),
    ] {
        assert!(edit.set_media_text(id, value).unwrap().changed());
    }
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(after.sheet(&sheet).unwrap().media(), &sheet_media);
    assert_eq!(
        after.sheet(&sheet).unwrap().inputs().media,
        *after.media(&sheet_media).unwrap()
    );
    assert!(
        matches!(after.rule(&import).unwrap().data(), CssomRuleData::Import {media, child_sheet:None, ..} if media == &import_media)
    );
    assert!(
        matches!(after.rule(&group).unwrap().data(), CssomRuleData::Group {prelude:CssomGroupPrelude::Media(media), ..} if media == &group_media)
    );
    assert_eq!(text(&after, &sheet_media), "speech");
    assert_eq!(text(&after, &import_media), "print");
    assert_eq!(text(&after, &group_media), "screen");
    assert_eq!(text(&before, &sheet_media), "");
    assert_eq!(text(&before, &import_media), "screen");
    assert_eq!(text(&before, &group_media), "print");
}

#[test]
fn exact_empty_setter_clears_before_any_parser_or_serialization_work() {
    let mut store = CssomStore::new(
        CssomLimits {
            css: CssSpecifiedValueSerializationLimits::new(0, 0, 0),
            max_input_bytes: 0,
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let mut inputs = input();
    inputs.media = parse_media_query_list("screen").into_parts().0;
    let sheet = create(&mut store, "", inputs);
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let before = store.snapshot();
    let mut edit = batch(&store);
    let result = edit.set_media_text(&id, "").unwrap();
    assert!(result.changed());
    assert!(result.diagnostics().is_empty());
    store.commit(edit).unwrap();
    assert_eq!(store.snapshot().media_length(&id).unwrap(), 0);
    assert_eq!(before.media_length(&id).unwrap(), 1);
    let mut edit = batch(&store);
    assert!(!edit.set_media_text(&id, "").unwrap().changed());
    unchanged(&store.commit(edit).unwrap());
}

#[test]
fn recovered_setter_keeps_owned_origins_and_item_null_then_empty_clears() {
    let mut store = new_store();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let before = store.snapshot();
    let source = String::from("/* 😀 */ SCREEN, ???, print");
    let mut edit = batch(&store);
    let result = edit.set_media_text(&id, &source).unwrap();
    assert!(result.changed());
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    assert_eq!(
        result.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        source.find("???").unwrap()
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    let origin = after.media(&id).unwrap().queries()[1].origin().clone();
    assert!(matches!(origin, CssValueOrigin::Parsed(_)));
    drop(source);
    assert_eq!(text(&after, &id), "screen, not all, print");
    assert_eq!(after.media_length(&id).unwrap(), 3);
    for (index, expected) in ["screen", "not all", "print"].into_iter().enumerate() {
        assert_eq!(
            after.media_item(&id, index, limits()).unwrap().as_deref(),
            Some(expected)
        );
    }
    assert_eq!(
        after
            .media_item(&id, 3, CssSpecifiedValueSerializationLimits::new(0, 0, 0))
            .unwrap(),
        None
    );
    assert_eq!(after.media_item(&id, usize::MAX, limits()).unwrap(), None);
    let mut edit = batch(&store);
    assert!(edit.set_media_text(&id, "").unwrap().changed());
    store.commit(edit).unwrap();
    assert_eq!(text(&store.snapshot(), &id), "");
    assert_eq!(store.snapshot().media_length(&id).unwrap(), 0);
    assert_eq!(before.media_length(&id).unwrap(), 0);
    drop(store);
    assert_eq!(after.media(&id).unwrap().queries()[1].origin(), &origin);
    assert_eq!(text(&after, &id), "screen, not all, print");
}

#[test]
fn append_ignores_null_and_canonical_duplicates_but_accepts_single_recovered_never() {
    let mut store = new_store();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let mut edit = batch(&store);
    edit.set_media_text(&id, "screen").unwrap();
    store.commit(edit).unwrap();
    let original = store.snapshot();
    for source in ["", "/* empty */", "screen, print", "SCREEN"] {
        let mut edit = batch(&store);
        assert!(
            !edit.append_medium(&id, source).unwrap().changed(),
            "{source}"
        );
        unchanged(&store.commit(edit).unwrap());
    }
    assert_eq!(
        store.snapshot().media(&id).unwrap(),
        original.media(&id).unwrap()
    );
    let mut edit = batch(&store);
    let result = edit.append_medium(&id, "???").unwrap();
    assert!(result.changed());
    assert_eq!(result.diagnostics().len(), 1);
    store.commit(edit).unwrap();
    let with_recovered = store.snapshot();
    assert_eq!(text(&with_recovered, &id), "screen, not all");
    let mut edit = batch(&store);
    assert!(
        !edit
            .append_medium(&id, "&different-malformed")
            .unwrap()
            .changed()
    );
    unchanged(&store.commit(edit).unwrap());
    assert_eq!(
        store.snapshot().media(&id).unwrap(),
        with_recovered.media(&id).unwrap()
    );
    assert_eq!(text(&original, &id), "screen");
}

#[test]
fn append_uses_case_sensitive_serialized_values_without_query_evaluation() {
    let mut store = new_store();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let mut edit = batch(&store);
    edit.set_media_text(&id, "(future: AbC), (resolution: 96dpi)")
        .unwrap();
    assert!(edit.append_medium(&id, "(future: abc)").unwrap().changed());
    assert!(!edit.append_medium(&id, "(FUTURE: AbC)").unwrap().changed());
    assert!(
        edit.append_medium(&id, "(resolution: 1dppx)")
            .unwrap()
            .changed()
    );
    store.commit(edit).unwrap();
    assert_eq!(
        text(&store.snapshot(), &id),
        "(future: AbC), (resolution: 96dpi), (future: abc), (resolution: 1dppx)"
    );
}

#[test]
fn delete_removes_all_serialized_matches_and_not_found_aborts_then_retry_succeeds() {
    let mut store = new_store();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let mut edit = batch(&store);
    edit.set_media_text(&id, "screen, PRINT, SCREEN, ???, &bad")
        .unwrap();
    store.commit(edit).unwrap();
    let before = store.snapshot();
    for source in ["", "screen, print"] {
        let mut edit = batch(&store);
        assert!(!edit.delete_medium(&id, source).unwrap().changed());
        unchanged(&store.commit(edit).unwrap());
    }
    let mut edit = batch(&store);
    assert!(edit.delete_medium(&id, "SCREEN").unwrap().changed());
    assert!(edit.delete_medium(&id, "???").unwrap().changed());
    store.commit(edit).unwrap();
    assert_eq!(text(&store.snapshot(), &id), "print");
    let mut edit = batch(&store);
    edit.append_medium(&id, "speech").unwrap();
    assert!(matches!(
        edit.delete_medium(&id, "screen"),
        Err(CssomError::Source(CssomException::NotFound))
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(text(&store.snapshot(), &id), "print");
    let mut retry = batch(&store);
    assert!(retry.delete_medium(&id, "PRINT").unwrap().changed());
    store.commit(retry).unwrap();
    assert_eq!(text(&store.snapshot(), &id), "");
    assert_eq!(
        text(&before, &id),
        "screen, print, screen, not all, not all"
    );
}

#[test]
fn complete_scan_budget_fails_atomically_including_later_members_and_retries() {
    for (css, expected) in [
        (
            CssSpecifiedValueSerializationLimits::new(6, 7, 17),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(7, 6, 17),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(7, 7, 16),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        let mut store = CssomStore::new(
            CssomLimits {
                css,
                ..CssomLimits::default()
            },
            CssomContext::default(),
        )
        .unwrap();
        let mut inputs = input();
        inputs.media = parse_media_query_list("screen, print").into_parts().0;
        let sheet = create(&mut store, "", inputs);
        let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
        let before = store.snapshot();
        let mut edit = batch(&store);
        let Err(CssomError::Media(CssMediaCssomSerializationError::Resource { error, .. })) =
            edit.append_medium(&id, "SCREEN")
        else {
            panic!("whole scan must exhaust {expected:?}");
        };
        assert_eq!(error.kind(), expected);
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert_eq!(text(&store.snapshot(), &id), "screen, print");
        let mut retry = batch(&store);
        retry.set_media_text(&id, "print").unwrap();
        assert!(retry.delete_medium(&id, "PRINT").unwrap().changed());
        store.commit(retry).unwrap();
        assert_eq!(text(&store.snapshot(), &id), "");
    }
}

#[test]
fn parser_and_input_resource_failures_never_publish_recovered_empty_state() {
    let mut store = CssomStore::new(
        CssomLimits {
            max_input_bytes: 6,
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let mut edit = batch(&store);
    edit.set_media_text(&id, "screen").unwrap();
    store.commit(edit).unwrap();
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.set_media_text(&id, "screen, print"),
        Err(CssomError::Limit {
            resource: "parse input bytes",
            maximum: 6
        })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());

    let mut store = new_store();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let mut edit = batch(&store);
    edit.set_media_text(&id, "print").unwrap();
    store.commit(edit).unwrap();
    let deep = format!("{}color{}", "(".repeat(2000), ")".repeat(2000));
    let mut edit = batch(&store);
    assert!(matches!(
        edit.set_media_text(&id, &deep),
        Err(CssomError::ParseResource(_))
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(text(&store.snapshot(), &id), "print");
    let mut retry = batch(&store);
    retry.append_medium(&id, "screen").unwrap();
    store.commit(retry).unwrap();
    assert_eq!(text(&store.snapshot(), &id), "print, screen");
}

#[test]
fn snapshot_output_errors_are_typed_and_do_not_change_the_owner() {
    let mut store = new_store();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let mut edit = batch(&store);
    edit.set_media_text(&id, "screen").unwrap();
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    for result in [
        snapshot.media_text(&id, CssSpecifiedValueSerializationLimits::new(2, 2, 5)),
        snapshot
            .media_item(&id, 0, CssSpecifiedValueSerializationLimits::new(2, 2, 5))
            .map(|v| v.unwrap()),
    ] {
        assert!(
            matches!(result, Err(CssomError::Media(CssMediaCssomSerializationError::Resource { error, .. })) if error.kind() == CssSpecifiedValueSerializationErrorKind::ByteLimit)
        );
    }
    assert_eq!(text(&snapshot, &id), "screen");
    assert_eq!(store.snapshot().revision(), snapshot.revision());
}
