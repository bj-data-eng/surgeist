#![forbid(unsafe_code)]
//! Public consumer expectations from selected CSSOM §4.4 MediaList algorithms.
//! The CSS writer's historical modern-value lexical policy remains selected.
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use surgeist_css::CssParser;
use surgeist_css::component::CssValueOrigin;
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::query::media::{CssMediaQuery, CssMediaQueryList};
use surgeist_css::serialization::{CssSerializationError, CssSerializationLimits};
use surgeist_css::source::{CssUtf16InputLimits, CssUtf16Resource};
use surgeist_cssom::*;

fn limits() -> CssSerializationLimits {
    CssSerializationLimits::default()
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
fn captured_media(
    store: &CssomStore,
    snapshot: &CssomSnapshot,
    id: &CssomMediaListId,
) -> CssomCapture<MediaList> {
    store.media_list(id).unwrap().capture(snapshot).unwrap()
}
fn text(store: &CssomStore, snapshot: &CssomSnapshot, id: &CssomMediaListId) -> String {
    captured_media(store, snapshot, id)
        .media_text(limits())
        .unwrap()
}
fn unchanged(commit: &CssomCommit) {
    assert!(matches!(
        commit.publication(),
        CssomPublication::Unchanged { .. }
    ));
}

fn source_free_media(store: &mut CssomStore) -> MediaList {
    let mut edit = batch(store);
    let ticket = edit
        .create_typed_sheet(surgeist_css::rule::CssSheet::new(), input())
        .unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    store.css_style_sheet(&sheet).unwrap().media(store).unwrap()
}

fn exact_media(suffix: &str) -> Vec<u16> {
    let mut units = Vec::with_capacity(64);
    units.extend("/*\r\n😀".encode_utf16());
    units.push(0xd800);
    units.extend("*/".encode_utf16());
    units.extend(suffix.encode_utf16());
    units
}

#[test]
fn utf16_setter_retains_original_units_and_changes_only_distinct_source_values() {
    let mut store = new_store();
    let media = source_free_media(&mut store);
    let proceed = AtomicBool::new(false);
    let raw = Arc::new(exact_media(" SCREEN, ???, print"));
    let changed = media
        .set_media_text(&mut store, CssomSheetText::Utf16(raw.clone()), &proceed)
        .unwrap();
    let first = store.snapshot();
    let list = first.media(media.id()).unwrap();
    let source = list.parse_source().unwrap().source();
    assert!(Arc::ptr_eq(source.original_utf16().unwrap(), &raw));
    assert_eq!(source.as_str(), "/*\n😀\u{fffd}*/ SCREEN, ???, print");
    assert_eq!(source.original_utf16_range(2..3), Some(2..4));
    assert_eq!(source.original_utf16_range(7..10), Some(6..7));
    assert_eq!(changed.diagnostics().len(), 1);
    let diagnostic = &changed.diagnostics()[0];
    assert!(diagnostic.source().unwrap().same_snapshot(source));
    let at = diagnostic.error().position().byte_offset().value();
    assert_eq!(source.original_utf16_range(at..at + 3), Some(18..21));
    assert_eq!(
        media.media_text(&store, limits()).unwrap(),
        "screen, not all, print"
    );

    let equal = Arc::new(raw.as_ref().clone());
    assert!(!Arc::ptr_eq(&equal, &raw));
    let same = media
        .set_media_text(&mut store, CssomSheetText::Utf16(equal), &proceed)
        .unwrap();
    assert!(matches!(
        same.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), first.revision());
    let mut distinct = raw.as_ref().clone();
    distinct[6] = 0;
    let distinct = Arc::new(distinct);
    let changed = media
        .set_media_text(
            &mut store,
            CssomSheetText::Utf16(distinct.clone()),
            &proceed,
        )
        .unwrap();
    assert!(matches!(
        changed.publication(),
        CssomPublication::Changed(_)
    ));
    let second = store.snapshot();
    let actual = second
        .media(media.id())
        .unwrap()
        .parse_source()
        .unwrap()
        .source();
    assert_eq!(actual.as_str(), source.as_str());
    assert!(Arc::ptr_eq(actual.original_utf16().unwrap(), &distinct));
    assert!(Arc::ptr_eq(source.original_utf16().unwrap(), &raw));

    let mut edit = batch(&store);
    assert!(
        edit.set_media_text(media.id(), CssomSheetText::Utf16(raw.clone()), &proceed)
            .unwrap()
            .changed()
    );
    let restored = Arc::new(distinct.as_ref().clone());
    assert!(
        edit.set_media_text(media.id(), CssomSheetText::Utf16(restored), &proceed)
            .unwrap()
            .changed()
    );
    unchanged(&store.commit(edit).unwrap());
    assert_eq!(store.snapshot().revision(), second.revision());

    let comment = Arc::new(exact_media(""));
    let _ = media
        .set_media_text(&mut store, CssomSheetText::Utf16(comment.clone()), &proceed)
        .unwrap();
    let current = store.snapshot();
    let empty = current.media(media.id()).unwrap();
    assert!(empty.queries().is_empty());
    assert!(Arc::ptr_eq(
        empty
            .parse_source()
            .unwrap()
            .source()
            .original_utf16()
            .unwrap(),
        &comment
    ));
    let cleared = media
        .set_media_text(
            &mut store,
            CssomSheetText::Utf16(Arc::new(Vec::new())),
            &proceed,
        )
        .unwrap();
    assert!(matches!(
        cleared.publication(),
        CssomPublication::Changed(_)
    ));
    assert!(
        store
            .snapshot()
            .media(media.id())
            .unwrap()
            .parse_source()
            .is_none()
    );
}

#[test]
fn utf16_single_medium_preserves_canonical_duplicate_recovery_and_removal() {
    let mut store = new_store();
    let media = source_free_media(&mut store);
    let proceed = AtomicBool::new(false);
    let valid = Arc::new(exact_media(" PRINT"));
    let _ = media
        .append_medium(&mut store, CssomSheetText::Utf16(valid.clone()), &proceed)
        .unwrap();
    let first = store.snapshot();
    let member = &first.media(media.id()).unwrap().queries()[0];
    let CssValueOrigin::Parsed(origin) = member.origin() else {
        panic!("appended exact query origin");
    };
    assert!(Arc::ptr_eq(
        origin.source().original_utf16().unwrap(),
        &valid
    ));
    assert_eq!(media.media_text(&store, limits()).unwrap(), "print");
    let duplicate = media
        .append_medium(
            &mut store,
            CssomSheetText::Utf16(Arc::new(exact_media(" print"))),
            &proceed,
        )
        .unwrap();
    assert!(matches!(
        duplicate.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), first.revision());

    let invalid = Arc::new(exact_media(" ???"));
    let recovered = media
        .append_medium(&mut store, CssomSheetText::Utf16(invalid.clone()), &proceed)
        .unwrap();
    assert_eq!(
        media.media_text(&store, limits()).unwrap(),
        "print, not all"
    );
    assert_eq!(recovered.diagnostics().len(), 1);
    assert!(Arc::ptr_eq(
        recovered.diagnostics()[0]
            .source()
            .unwrap()
            .original_utf16()
            .unwrap(),
        &invalid
    ));
    let before_delete = store.snapshot();
    let multiple = CssomSheetText::Utf16(Arc::new(exact_media("print, speech")));
    let skipped = media.delete_medium(&mut store, multiple, &proceed).unwrap();
    assert!(matches!(
        skipped.publication(),
        CssomPublication::Unchanged { .. }
    ));
    let absent = CssomSheetText::Utf16(Arc::new(exact_media("speech")));
    assert!(matches!(
        media.delete_medium(&mut store, absent, &proceed),
        Err(CssomError::Source(CssomException::NotFound))
    ));
    assert_eq!(store.snapshot().revision(), before_delete.revision());
    let removed = media
        .delete_medium(&mut store, CssomSheetText::Utf16(invalid), &proceed)
        .unwrap();
    assert_eq!(removed.diagnostics().len(), 1);
    assert_eq!(media.media_text(&store, limits()).unwrap(), "print");
}

#[test]
fn utf16_batch_replaces_exact_member_origin_even_when_normalization_is_equal() {
    let mut store = new_store();
    let media = source_free_media(&mut store);
    let proceed = AtomicBool::new(false);
    let input = |unit| {
        let mut units = vec![b'/' as u16, b'*' as u16, unit];
        units.extend("*/print".encode_utf16());
        Arc::new(units)
    };
    let original = input(0xd800);
    let _ = media
        .append_medium(
            &mut store,
            CssomSheetText::Utf16(original.clone()),
            &proceed,
        )
        .unwrap();
    let first = store.snapshot();
    let original_units = |snapshot: &CssomSnapshot| {
        let [member] = snapshot.media(media.id()).unwrap().queries() else {
            panic!("expected one print member");
        };
        let CssValueOrigin::Parsed(origin) = member.origin() else {
            panic!("expected an authored member origin");
        };
        assert_eq!(origin.source().as_str(), "/*\u{fffd}*/print");
        origin.source().original_utf16().unwrap().clone()
    };
    assert!(Arc::ptr_eq(&original_units(&first), &original));
    let replace = |store: &mut CssomStore, replacement| {
        let mut edit = batch(store);
        assert!(
            edit.delete_medium(media.id(), "print", &proceed)
                .unwrap()
                .changed()
        );
        assert!(
            edit.append_medium(media.id(), CssomSheetText::Utf16(replacement), &proceed)
                .unwrap()
                .changed()
        );
        store.commit(edit).unwrap()
    };

    let equal = input(0xd800);
    assert!(!Arc::ptr_eq(&equal, &original));
    unchanged(&replace(&mut store, equal));
    assert_eq!(store.snapshot().revision(), first.revision());
    assert!(Arc::ptr_eq(&original_units(&store.snapshot()), &original));

    let replacement = input(0);
    let commit = replace(&mut store, replacement.clone());
    assert!(
        matches!(commit.publication(), CssomPublication::Changed(_)),
        "different original units must publish a new member origin: {:?}",
        commit.publication()
    );
    let current = store.snapshot();
    assert_ne!(current.revision(), first.revision());
    assert_eq!(media.media_text(&store, limits()).unwrap(), "print");
    assert!(Arc::ptr_eq(&original_units(&current), &replacement));
    assert!(Arc::ptr_eq(&original_units(&first), &original));
}

#[test]
fn utf16_batch_member_restoration_spends_only_the_reached_comparison_work() {
    let original = Arc::new(exact_media("print"));
    let mut polls = 0;
    CssMediaQuery::parse_medium_utf16(
        &CssParser::default(),
        original.clone(),
        CssUtf16InputLimits::default(),
        || {
            polls += 1;
            Ok::<_, std::convert::Infallible>(())
        },
    )
    .unwrap();
    let parse_work = polls - 1; // The initial interruption observation costs no work.
    let exact_work = parse_work + 1 + original.len();
    let proceed = AtomicBool::new(false);
    for work in [exact_work - 1, exact_work] {
        let mut policy = CssomLimits::default();
        policy.sheet_utf16_input.max_work = work;
        let mut store = CssomStore::new(policy, Default::default()).unwrap();
        let media = source_free_media(&mut store);
        let _ = media
            .append_medium(
                &mut store,
                CssomSheetText::Utf16(original.clone()),
                &proceed,
            )
            .unwrap();
        let before = store.snapshot();
        let mut edit = batch(&store);
        assert!(
            edit.delete_medium(media.id(), "print", &proceed)
                .unwrap()
                .changed()
        );
        let equal = Arc::new(original.as_ref().clone());
        let restored = edit.append_medium(media.id(), CssomSheetText::Utf16(equal), &proceed);
        if work == exact_work {
            assert!(restored.unwrap().changed());
            unchanged(&store.commit(edit).unwrap());
        } else {
            assert!(matches!(restored, Err(CssomError::Utf16Input(error))
                if error.exhausted_limit() == Some(CssUtf16Resource::Work)));
            assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        }
        assert_eq!(store.snapshot().revision(), before.revision());
        assert_eq!(media.media_text(&store, limits()).unwrap(), "print");
    }
}

#[test]
fn utf16_batch_restores_member_origins_when_later_deletes_shift_appended_indices() {
    let mut store = new_store();
    let media = source_free_media(&mut store);
    let proceed = AtomicBool::new(false);
    for source in ["print", "screen"] {
        let _ = media
            .append_medium(
                &mut store,
                CssomSheetText::Utf16(Arc::new(exact_media(source))),
                &proceed,
            )
            .unwrap();
    }
    let before = store.snapshot();
    let mut edit = batch(&store);
    for source in ["print", "screen"] {
        assert!(
            edit.delete_medium(media.id(), source, &proceed)
                .unwrap()
                .changed()
        );
        assert!(
            edit.append_medium(
                media.id(),
                CssomSheetText::Utf16(Arc::new(exact_media(source))),
                &proceed,
            )
            .unwrap()
            .changed()
        );
    }
    unchanged(&store.commit(edit).unwrap());
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(media.media_text(&store, limits()).unwrap(), "print, screen");
}

#[test]
fn explicit_empty_text_keeps_the_list_source_cleared_after_a_member_is_restored() {
    let mut store = new_store();
    let media = source_free_media(&mut store);
    let proceed = AtomicBool::new(false);
    let exact = Arc::new(exact_media("print"));
    let _ = media
        .set_media_text(&mut store, CssomSheetText::Utf16(exact.clone()), &proceed)
        .unwrap();
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(
        edit.delete_medium(media.id(), "print", &proceed)
            .unwrap()
            .changed()
    );
    assert!(
        !edit
            .set_media_text(media.id(), "", &proceed)
            .unwrap()
            .changed()
    );
    assert!(
        edit.append_medium(media.id(), CssomSheetText::Utf16(exact), &proceed)
            .unwrap()
            .changed()
    );
    assert!(matches!(
        store.commit(edit).unwrap().publication(),
        CssomPublication::Changed(_)
    ));
    let after = store.snapshot();
    assert_ne!(after.revision(), before.revision());
    assert!(before.media(media.id()).unwrap().parse_source().is_some());
    assert!(after.media(media.id()).unwrap().parse_source().is_none());
    assert_eq!(media.media_text(&store, limits()).unwrap(), "print");
}

#[test]
fn explicit_exact_text_never_restores_the_old_source_with_equal_normalization() {
    let mut store = new_store();
    let media = source_free_media(&mut store);
    let proceed = AtomicBool::new(false);
    let units = exact_media("print");
    let mut replacement = units.clone();
    replacement[6] = 0;
    let replacement = Arc::new(replacement);
    let _ = media
        .set_media_text(&mut store, CssomSheetText::Utf16(Arc::new(units)), &proceed)
        .unwrap();
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(
        edit.set_media_text(
            media.id(),
            CssomSheetText::Utf16(replacement.clone()),
            &proceed
        )
        .unwrap()
        .changed()
    );
    assert!(
        edit.append_medium(media.id(), "screen", &proceed)
            .unwrap()
            .changed()
    );
    assert!(
        edit.delete_medium(media.id(), "screen", &proceed)
            .unwrap()
            .changed()
    );
    assert!(matches!(
        store.commit(edit).unwrap().publication(),
        CssomPublication::Changed(_)
    ));
    let after = store.snapshot();
    assert_ne!(after.revision(), before.revision());
    let list = after.media(media.id()).unwrap();
    if let Some(parsed) = list.parse_source() {
        assert!(Arc::ptr_eq(
            parsed.source().original_utf16().unwrap(),
            &replacement
        ));
    }
    let [member] = list.queries() else {
        panic!("expected one print member");
    };
    let CssValueOrigin::Parsed(origin) = member.origin() else {
        panic!("expected an authored print member");
    };
    assert!(Arc::ptr_eq(
        origin.source().original_utf16().unwrap(),
        &replacement
    ));
    assert_eq!(media.media_text(&store, limits()).unwrap(), "print");
}

#[test]
fn utf16_media_admits_exact_backing_and_cumulative_equal_input_work() {
    let raw = Arc::new(exact_media("screen"));
    let (parsed, parse_work) = CssMediaQueryList::parse_utf16_with_work(
        &CssParser::default(),
        raw.clone(),
        CssUtf16InputLimits::default(),
        || Ok::<(), std::convert::Infallible>(()),
    )
    .unwrap();
    let backing = parsed
        .syntax()
        .parse_source()
        .unwrap()
        .source()
        .storage_usage()
        .retained_bytes();
    let proceed = AtomicBool::new(false);
    let exact_work = parse_work + raw.len();
    for (maximum, work, backing_fits, comparison_fits) in [
        (backing - 1, exact_work, false, false),
        (backing, exact_work - 1, true, false),
        (backing, exact_work, true, true),
    ] {
        let mut limits = CssomLimits {
            max_retained_source_bytes: maximum,
            ..Default::default()
        };
        limits.sheet_utf16_input.max_input_units = raw.len();
        limits.sheet_utf16_input.max_work = work;
        let mut store = CssomStore::new(limits, Default::default()).unwrap();
        let media = source_free_media(&mut store);
        let before = store.snapshot();
        let first = media.set_media_text(&mut store, CssomSheetText::Utf16(raw.clone()), &proceed);
        if !backing_fits {
            assert!(
                matches!(first, Err(CssomError::Limit {
                resource: "retained parse source bytes", maximum: actual
            }) if actual == maximum),
                "{first:?}"
            );
            assert_eq!(store.snapshot().revision(), before.revision());
            continue;
        }
        let _ = first.unwrap();
        let before = store.snapshot();
        let mut equal = Vec::with_capacity(raw.capacity());
        equal.extend_from_slice(raw.as_slice());
        let second =
            media.set_media_text(&mut store, CssomSheetText::Utf16(Arc::new(equal)), &proceed);
        if comparison_fits {
            assert!(matches!(
                second.unwrap().publication(),
                CssomPublication::Unchanged { .. }
            ));
        } else {
            assert!(matches!(second, Err(CssomError::Utf16Input(error))
                if error.exhausted_limit() == Some(CssUtf16Resource::Work)));
        }
        assert_eq!(store.snapshot().revision(), before.revision());
    }
}

#[test]
fn utf16_media_refuses_original_input_limit_and_cancellation_without_publication() {
    let raw = Arc::new(exact_media("screen"));
    let mut limits = CssomLimits::default();
    limits.sheet_utf16_input.max_input_units = raw.len() - 1;
    let mut store = CssomStore::new(limits, Default::default()).unwrap();
    let media = source_free_media(&mut store);
    let before = store.snapshot();
    let original = || CssomSheetText::Utf16(raw.clone());
    let cancel = AtomicBool::new(true);
    assert!(matches!(
        media.set_media_text(&mut store, original(), &cancel),
        Err(CssomError::Cancelled)
    ));
    assert!(matches!(
        media.append_medium(&mut store, original(), &cancel),
        Err(CssomError::Cancelled)
    ));
    assert!(matches!(
        media.delete_medium(&mut store, original(), &cancel),
        Err(CssomError::Cancelled)
    ));
    let mut foreign = new_store();
    assert!(matches!(
        media.delete_medium(&mut foreign, original(), &cancel),
        Err(CssomError::ForeignOwner)
    ));
    let mut edit = batch(&store);
    assert!(matches!(
        edit.append_medium(media.id(), original(), &cancel),
        Err(CssomError::Cancelled)
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert!(
        matches!(media.append_medium(&mut store, original(), &AtomicBool::new(false)),
        Err(CssomError::Utf16Input(error)) if error.exhausted_limit() == Some(CssUtf16Resource::InputUnits))
    );
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(media.length(&store).unwrap(), 0);
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
        assert!(
            edit.set_media_text(id, value, &AtomicBool::new(false))
                .unwrap()
                .changed()
        );
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
    assert_eq!(text(&store, &after, &sheet_media), "speech");
    assert_eq!(text(&store, &after, &import_media), "print");
    assert_eq!(text(&store, &after, &group_media), "screen");
    assert_eq!(text(&store, &before, &sheet_media), "");
    assert_eq!(text(&store, &before, &import_media), "screen");
    assert_eq!(text(&store, &before, &group_media), "print");
}

#[test]
fn exact_empty_setter_clears_before_any_parser_or_serialization_work() {
    let mut store = CssomStore::new(
        CssomLimits {
            css: CssSerializationLimits::new(0, 0, 0),
            max_input_bytes: 0,
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let mut inputs = input();
    inputs.media = CssMediaQueryList::parse(&CssParser::default(), "screen")
        .into_parts()
        .0;
    let sheet = create(&mut store, "", inputs);
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let before = store.snapshot();
    let mut edit = batch(&store);
    let result = edit
        .set_media_text(&id, "", &AtomicBool::new(false))
        .unwrap();
    assert!(result.changed());
    assert!(result.diagnostics().is_empty());
    store.commit(edit).unwrap();
    assert_eq!(
        captured_media(&store, &store.snapshot(), &id)
            .length()
            .unwrap(),
        0
    );
    assert_eq!(captured_media(&store, &before, &id).length().unwrap(), 1);
    let mut edit = batch(&store);
    assert!(
        !edit
            .set_media_text(&id, "", &AtomicBool::new(false))
            .unwrap()
            .changed()
    );
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
    let result = edit
        .set_media_text(&id, source.as_str(), &AtomicBool::new(false))
        .unwrap();
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
    assert_eq!(text(&store, &after, &id), "screen, not all, print");
    assert_eq!(captured_media(&store, &after, &id).length().unwrap(), 3);
    for (index, expected) in ["screen", "not all", "print"].into_iter().enumerate() {
        assert_eq!(
            captured_media(&store, &after, &id)
                .item(index, limits())
                .unwrap()
                .as_deref(),
            Some(expected)
        );
    }
    assert_eq!(
        captured_media(&store, &after, &id)
            .item(3, CssSerializationLimits::new(0, 0, 0))
            .unwrap(),
        None
    );
    assert_eq!(
        captured_media(&store, &after, &id)
            .item(usize::MAX, limits())
            .unwrap(),
        None
    );
    let mut edit = batch(&store);
    assert!(
        edit.set_media_text(&id, "", &AtomicBool::new(false))
            .unwrap()
            .changed()
    );
    store.commit(edit).unwrap();
    assert_eq!(text(&store, &store.snapshot(), &id), "");
    assert_eq!(
        captured_media(&store, &store.snapshot(), &id)
            .length()
            .unwrap(),
        0
    );
    assert_eq!(captured_media(&store, &before, &id).length().unwrap(), 0);
    let retained_media = captured_media(&store, &after, &id);
    drop(store);
    assert_eq!(after.media(&id).unwrap().queries()[1].origin(), &origin);
    assert_eq!(
        retained_media.media_text(limits()).unwrap(),
        "screen, not all, print"
    );
}

#[test]
fn append_ignores_null_and_canonical_duplicates_but_accepts_single_recovered_never() {
    let mut store = new_store();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let mut edit = batch(&store);
    edit.set_media_text(&id, "screen", &AtomicBool::new(false))
        .unwrap();
    store.commit(edit).unwrap();
    let original = store.snapshot();
    for source in ["", "/* empty */", "screen, print", "SCREEN"] {
        let mut edit = batch(&store);
        assert!(
            !edit
                .append_medium(&id, source, &AtomicBool::new(false))
                .unwrap()
                .changed(),
            "{source}"
        );
        unchanged(&store.commit(edit).unwrap());
    }
    assert_eq!(
        store.snapshot().media(&id).unwrap(),
        original.media(&id).unwrap()
    );
    let mut edit = batch(&store);
    let result = edit
        .append_medium(&id, "???", &AtomicBool::new(false))
        .unwrap();
    assert!(result.changed());
    assert_eq!(result.diagnostics().len(), 1);
    store.commit(edit).unwrap();
    let with_recovered = store.snapshot();
    assert_eq!(text(&store, &with_recovered, &id), "screen, not all");
    let mut edit = batch(&store);
    assert!(
        !edit
            .append_medium(&id, "&different-malformed", &AtomicBool::new(false))
            .unwrap()
            .changed()
    );
    unchanged(&store.commit(edit).unwrap());
    assert_eq!(
        store.snapshot().media(&id).unwrap(),
        with_recovered.media(&id).unwrap()
    );
    assert_eq!(text(&store, &original, &id), "screen");
}

#[test]
fn append_uses_case_sensitive_serialized_values_without_query_evaluation() {
    let mut store = new_store();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let mut edit = batch(&store);
    edit.set_media_text(
        &id,
        "(future: AbC), (resolution: 96dpi)",
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(
        edit.append_medium(&id, "(future: abc)", &AtomicBool::new(false))
            .unwrap()
            .changed()
    );
    assert!(
        !edit
            .append_medium(&id, "(FUTURE: AbC)", &AtomicBool::new(false))
            .unwrap()
            .changed()
    );
    assert!(
        edit.append_medium(&id, "(resolution: 1dppx)", &AtomicBool::new(false))
            .unwrap()
            .changed()
    );
    store.commit(edit).unwrap();
    assert_eq!(
        text(&store, &store.snapshot(), &id),
        "(future: AbC), (resolution: 96dpi), (future: abc), (resolution: 1dppx)"
    );
}

#[test]
fn delete_removes_all_serialized_matches_and_not_found_aborts_then_retry_succeeds() {
    let mut store = new_store();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let mut edit = batch(&store);
    edit.set_media_text(
        &id,
        "screen, PRINT, SCREEN, ???, &bad",
        &AtomicBool::new(false),
    )
    .unwrap();
    store.commit(edit).unwrap();
    let before = store.snapshot();
    for source in ["", "screen, print"] {
        let mut edit = batch(&store);
        assert!(
            !edit
                .delete_medium(&id, source, &AtomicBool::new(false))
                .unwrap()
                .changed()
        );
        unchanged(&store.commit(edit).unwrap());
    }
    let mut edit = batch(&store);
    assert!(
        edit.delete_medium(&id, "SCREEN", &AtomicBool::new(false))
            .unwrap()
            .changed()
    );
    assert!(
        edit.delete_medium(&id, "???", &AtomicBool::new(false))
            .unwrap()
            .changed()
    );
    store.commit(edit).unwrap();
    assert_eq!(text(&store, &store.snapshot(), &id), "print");
    let mut edit = batch(&store);
    edit.append_medium(&id, "speech", &AtomicBool::new(false))
        .unwrap();
    assert!(matches!(
        edit.delete_medium(&id, "screen", &AtomicBool::new(false)),
        Err(CssomError::Source(CssomException::NotFound))
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(text(&store, &store.snapshot(), &id), "print");
    let mut retry = batch(&store);
    assert!(
        retry
            .delete_medium(&id, "PRINT", &AtomicBool::new(false))
            .unwrap()
            .changed()
    );
    store.commit(retry).unwrap();
    assert_eq!(text(&store, &store.snapshot(), &id), "");
    assert_eq!(
        text(&store, &before, &id),
        "screen, print, screen, not all, not all"
    );
}

#[test]
fn complete_scan_budget_fails_atomically_including_later_members_and_retries() {
    type Case = (CssSerializationLimits, fn(&CssSerializationError) -> bool);
    let cases: [Case; 3] = [
        (
            CssSerializationLimits::new(6, 7, 17),
            CssSerializationError::is_input_node_limit,
        ),
        (
            CssSerializationLimits::new(7, 6, 17),
            CssSerializationError::is_projection_node_limit,
        ),
        (
            CssSerializationLimits::new(7, 7, 16),
            CssSerializationError::is_byte_limit,
        ),
    ];
    for (css, expected) in cases {
        let mut store = CssomStore::new(
            CssomLimits {
                css,
                ..CssomLimits::default()
            },
            CssomContext::default(),
        )
        .unwrap();
        let mut inputs = input();
        inputs.media = CssMediaQueryList::parse(&CssParser::default(), "screen, print")
            .into_parts()
            .0;
        let sheet = create(&mut store, "", inputs);
        let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
        let before = store.snapshot();
        let mut edit = batch(&store);
        let Err(CssomError::Media(error)) =
            edit.append_medium(&id, "SCREEN", &AtomicBool::new(false))
        else {
            panic!("whole scan must exhaust {css:?}");
        };
        assert!(expected(&error), "{css:?}: {error:?}");
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(store.snapshot().revision(), before.revision());
        assert_eq!(text(&store, &store.snapshot(), &id), "screen, print");
        let mut retry = batch(&store);
        retry
            .set_media_text(&id, "print", &AtomicBool::new(false))
            .unwrap();
        assert!(
            retry
                .delete_medium(&id, "PRINT", &AtomicBool::new(false))
                .unwrap()
                .changed()
        );
        store.commit(retry).unwrap();
        assert_eq!(text(&store, &store.snapshot(), &id), "");
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
    edit.set_media_text(&id, "screen", &AtomicBool::new(false))
        .unwrap();
    store.commit(edit).unwrap();
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.set_media_text(&id, "screen, print", &AtomicBool::new(false)),
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
    edit.set_media_text(&id, "print", &AtomicBool::new(false))
        .unwrap();
    store.commit(edit).unwrap();
    let deep = format!("{}color{}", "(".repeat(2000), ")".repeat(2000));
    let mut edit = batch(&store);
    assert!(matches!(
        edit.set_media_text(&id, deep.as_str(), &AtomicBool::new(false)),
        Err(CssomError::ParseResource(_))
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(text(&store, &store.snapshot(), &id), "print");
    let mut retry = batch(&store);
    retry
        .append_medium(&id, "screen", &AtomicBool::new(false))
        .unwrap();
    store.commit(retry).unwrap();
    assert_eq!(text(&store, &store.snapshot(), &id), "print, screen");
}

#[test]
fn snapshot_output_errors_are_typed_and_do_not_change_the_owner() {
    let mut store = new_store();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let mut edit = batch(&store);
    edit.set_media_text(&id, "screen", &AtomicBool::new(false))
        .unwrap();
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    for result in [
        captured_media(&store, &snapshot, &id).media_text(CssSerializationLimits::new(2, 2, 5)),
        captured_media(&store, &snapshot, &id)
            .item(0, CssSerializationLimits::new(2, 2, 5))
            .map(|v| v.unwrap()),
    ] {
        assert!(matches!(result, Err(CssomError::Media(error)) if error.is_byte_limit()));
    }
    assert_eq!(text(&store, &snapshot, &id), "screen");
    assert_eq!(store.snapshot().revision(), snapshot.revision());
}

#[test]
fn replacing_a_list_with_an_equal_list_of_other_provenance_is_a_change() {
    let mut store = new_store();
    let sheet = create(&mut store, "", input());
    let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
    let parsed = CssMediaQueryList::parse(&CssParser::default(), "screen, print")
        .into_parts()
        .0;
    let mut edit = batch(&store);
    assert!(edit.set_media(&id, parsed.clone()).unwrap());
    store.commit(edit).unwrap();
    // The members rebuilt without their commas and parse source equal the parsed list,
    // yet the stored list came from other syntax, so storing it is a change.
    let rebuilt = CssMediaQueryList::new(parsed.queries().to_vec());
    assert_eq!(parsed, rebuilt);
    let mut edit = batch(&store);
    assert!(edit.set_media(&id, rebuilt.clone()).unwrap());
    assert!(matches!(
        store.commit(edit).unwrap().publication(),
        CssomPublication::Changed(_)
    ));
    let mut edit = batch(&store);
    assert!(!edit.set_media(&id, rebuilt).unwrap());
    unchanged(&store.commit(edit).unwrap());
}
