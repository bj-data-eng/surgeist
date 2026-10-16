//! Independently derived standard object results from the retained CSSOM edition.
use surgeist_css::{
    CssParserContext, CssParserMode, CssSpecifiedValueSerializationLimits, CssValueOrigin,
};
use surgeist_cssom::*;
fn limits() -> CssSpecifiedValueSerializationLimits {
    CssSpecifiedValueSerializationLimits::default()
}
fn store() -> CssomStore {
    CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap()
}
fn version(role: CssomInputRole, name: &str, revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: name.into(),
        revision,
    }
}
fn document() -> CssomConstructorDocument {
    CssomConstructorDocument {
        version: version(CssomInputRole::Document, "doc", 1),
        identity: "doc".into(),
        base_location: "https://host.test/base".into(),
    }
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn construct(store: &mut CssomStore) -> CssStyleSheet {
    CssStyleSheet::new(store, document(), CssStyleSheetInit::default())
        .unwrap()
        .into_parts()
        .0
}
fn external(store: &mut CssomStore, text: &str, clean: bool, locked: bool) -> CssStyleSheet {
    let mut edit = batch(store);
    let mut inputs =
        CssomSheetInputs::external(version(CssomInputRole::Owner, "external", 1), clean);
    inputs.disallow_modification = locked;
    inputs.owner = Some("opaque-style-node".into());
    inputs.title = std::sync::Arc::new("Selected".encode_utf16().collect());
    let ticket = edit.create_parsed_sheet(text, inputs).unwrap();
    let id = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    store.css_style_sheet(&id).unwrap()
}
fn rule_text(rule: &CssRule, store: &CssomStore) -> String {
    rule.css_text(store, &CssomRuleFormatLimits::default())
        .unwrap()
}
fn assert_exception<T>(result: Result<T, CssomError>, expected: CssomException) {
    assert!(matches!(result, Err(CssomError::Source(actual)) if actual == expected));
}

#[test]
fn constructor_defaults_and_inherited_attributes_are_single_publication() {
    let mut owner = store();
    let before = owner.snapshot();
    let outcome = CssStyleSheet::new(&mut owner, document(), CssStyleSheetInit::default()).unwrap();
    let sheet = outcome.value();
    assert!(outcome.diagnostics().is_empty() && outcome.owner_effects().is_empty());
    assert_eq!(
        owner.snapshot().revision().value(),
        before.revision().value() + 1
    );
    assert_eq!(sheet.r#type(&owner).unwrap(), "text/css");
    assert_eq!(sheet.href(&owner).unwrap(), Some("https://host.test/base"));
    assert_eq!(sheet.owner_node(&owner).unwrap(), None);
    assert_eq!(sheet.title(&owner).unwrap(), None);
    assert!(sheet.parent_style_sheet(&owner).unwrap().is_none());
    assert!(sheet.owner_rule(&owner).unwrap().is_none());
    assert!(!sheet.disabled(&owner).unwrap());
    assert_eq!(sheet.as_style_sheet().r#type(&owner).unwrap(), "text/css");
    assert_eq!(
        sheet
            .media(&owner)
            .unwrap()
            .media_text(&owner, limits())
            .unwrap(),
        ""
    );
    assert_eq!(
        sheet.css_rules(&owner).unwrap(),
        sheet.rules(&owner).unwrap()
    );
    assert_eq!(sheet.css_rules(&owner).unwrap().length(&owner).unwrap(), 0);
    let list = owner
        .style_sheet_list(owner.snapshot().sheet_list())
        .unwrap();
    assert_eq!(list.length(&owner).unwrap(), 0);
    assert_eq!(
        owner
            .snapshot()
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .base_url,
        None
    );
    let sheet = outcome.into_parts().0;
    let old = sheet.capture(&owner.snapshot()).unwrap();
    let changed = sheet
        .as_style_sheet()
        .set_disabled(&mut owner, true)
        .unwrap();
    let _: &() = changed.value();
    assert!(sheet.disabled(&owner).unwrap());
    assert!(!old.disabled().unwrap());
    assert!(matches!(
        sheet.set_disabled(&mut owner, true).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    drop(owner);
    assert_eq!(
        old.as_style_sheet().href().unwrap(),
        Some("https://host.test/base")
    );
    assert_eq!(old.media().unwrap().media_text(limits()).unwrap(), "");
}

#[test]
fn captured_media_is_copied_through_cssom_text_with_new_origin_and_identity() {
    let mut source = store();
    let outcome = CssStyleSheet::new(
        &mut source,
        document(),
        CssStyleSheetInit {
            media: CssStyleSheetMedia::Text("screen and, print"),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!outcome.diagnostics().is_empty());
    let sheet = outcome.into_parts().0;
    let original = sheet
        .media(&source)
        .unwrap()
        .capture(&source.snapshot())
        .unwrap();
    assert_eq!(original.media_text(limits()).unwrap(), "not all, print");
    assert!(
        original
            .snapshot()
            .media(original.handle().id())
            .unwrap()
            .queries()[0]
            .is_guaranteed_false()
    );
    drop(source);
    let mut owner = store();
    let outcome = CssStyleSheet::new(
        &mut owner,
        document(),
        CssStyleSheetInit {
            base_url: Some(String::new()),
            media: CssStyleSheetMedia::List(&original),
            disabled: true,
        },
    )
    .unwrap();
    assert!(outcome.diagnostics().is_empty());
    let copied = outcome.value().media(&owner).unwrap();
    assert_ne!(copied.id(), original.handle().id());
    assert_eq!(
        copied.media_text(&owner, limits()).unwrap(),
        "not all, print"
    );
    let snapshot = owner.snapshot();
    let queries = snapshot.media(copied.id()).unwrap().queries();
    assert!(!queries[0].is_guaranteed_false());
    assert!(
        matches!(queries[0].origin(), CssValueOrigin::Parsed(origin) if origin.source().as_str() == "not all, print")
    );
    assert_eq!(
        snapshot
            .sheet(outcome.value().id())
            .unwrap()
            .inputs()
            .base_url,
        Some(String::new())
    );
    assert!(outcome.value().disabled(&owner).unwrap());
}

#[test]
fn live_and_captured_sheet_lists_follow_distinct_ordered_memberships() {
    let mut owner = store();
    let first = construct(&mut owner);
    let second = construct(&mut owner);
    let list = owner
        .style_sheet_list(owner.snapshot().sheet_list())
        .unwrap();
    let empty = list.capture(&owner.snapshot()).unwrap();
    let mut edit = batch(&owner);
    edit.set_sheet_list_membership(list.id(), &[second.id().clone(), first.id().clone()])
        .unwrap();
    owner.commit(edit).unwrap();
    let captured = list.capture(&owner.snapshot()).unwrap();
    assert_eq!(list.length(&owner).unwrap(), 2);
    assert_eq!(list.item(&owner, 0).unwrap(), Some(second.clone()));
    assert_eq!(list.item(&owner, 1).unwrap(), Some(first.clone()));
    assert!(list.item(&owner, usize::MAX).unwrap().is_none());
    assert_eq!(empty.length().unwrap(), 0);
    let mut edit = batch(&owner);
    edit.set_sheet_list_membership(list.id(), &[first.id().clone()])
        .unwrap();
    owner.commit(edit).unwrap();
    assert_eq!(list.length(&owner).unwrap(), 1);
    assert_eq!(captured.length().unwrap(), 2);
    assert_eq!(
        captured.item(0).unwrap().unwrap().snapshot().revision(),
        captured.snapshot().revision()
    );
    drop(owner);
    assert_eq!(captured.item(0).unwrap().unwrap().handle(), &second);
    assert!(captured.item(usize::MAX).unwrap().is_none());
}

#[test]
fn rule_list_same_object_and_old_capture_survive_edits_and_detachment() {
    let mut owner = store();
    let sheet = construct(&mut owner);
    assert_eq!(
        *sheet.insert_rule(&mut owner, "a {}", None).unwrap().value(),
        0
    );
    let list = sheet.css_rules(&owner).unwrap();
    let old = list.capture(&owner.snapshot()).unwrap();
    let first = list.item(&owner, 0).unwrap().unwrap();
    assert_eq!(
        *sheet.insert_rule(&mut owner, "b {}", None).unwrap().value(),
        0
    );
    assert_eq!(sheet.css_rules(&owner).unwrap(), list);
    assert_eq!(
        rule_text(&list.item(&owner, 0).unwrap().unwrap(), &owner),
        "b { }"
    );
    assert_eq!(old.length().unwrap(), 1);
    assert_eq!(old.item(0).unwrap().unwrap().handle(), &first);
    assert!(old.item(usize::MAX).unwrap().is_none());
    let _ = sheet.delete_rule(&mut owner, 1).unwrap();
    assert!(first.parent_style_sheet(&owner).unwrap().is_none());
    assert!(
        old.item(0)
            .unwrap()
            .unwrap()
            .parent_style_sheet()
            .unwrap()
            .is_some()
    );
    let old_revision = old.snapshot().revision().clone();
    drop(owner);
    assert_eq!(
        old.item(0).unwrap().unwrap().snapshot().revision(),
        &old_revision
    );
    assert_eq!(
        old.item(0)
            .unwrap()
            .unwrap()
            .css_text(&CssomRuleFormatLimits::default())
            .unwrap(),
        "a { }"
    );
}

#[test]
fn legacy_defaults_returns_and_optional_indices_use_native_algorithms() {
    let mut owner = store();
    let sheet = construct(&mut owner);
    assert_eq!(
        *sheet
            .add_rule(&mut owner, None, None, None)
            .unwrap()
            .value(),
        -1
    );
    assert_eq!(
        *sheet
            .add_rule(&mut owner, Some("last"), Some("color: red"), None)
            .unwrap()
            .value(),
        -1
    );
    let list = sheet.css_rules(&owner).unwrap();
    assert_eq!(
        rule_text(&list.item(&owner, 0).unwrap().unwrap(), &owner),
        "undefined { }"
    );
    assert_eq!(
        rule_text(&list.item(&owner, 1).unwrap().unwrap(), &owner),
        "last { color: red; }"
    );
    let _: &() = sheet.remove_rule(&mut owner, None).unwrap().value();
    assert_eq!(list.length(&owner).unwrap(), 1);
    assert_exception(
        sheet.delete_rule(&mut owner, usize::MAX),
        CssomException::IndexSize,
    );
    let result = sheet
        .replace_sync(&mut owner, "@import 'ignored.css'; next {}")
        .unwrap();
    let _: &() = result.value();
    assert_eq!(list.length(&owner).unwrap(), 1);
    assert_eq!(
        rule_text(&list.item(&owner, 0).unwrap().unwrap(), &owner),
        "next { }"
    );
}

#[test]
fn per_operation_guards_precede_invalid_syntax_and_untruncated_indices() {
    let mut owner = store();
    let opaque = external(&mut owner, "a {}", false, true);
    assert_eq!(
        opaque.owner_node(&owner).unwrap(),
        Some("opaque-style-node")
    );
    assert_eq!(
        opaque.title(&owner).unwrap(),
        Some("Selected".encode_utf16().collect::<Vec<_>>().as_slice())
    );
    assert_exception(opaque.css_rules(&owner), CssomException::Security);
    assert_exception(opaque.rules(&owner), CssomException::Security);
    assert_exception(
        opaque.insert_rule(&mut owner, "!", Some(usize::MAX)),
        CssomException::Security,
    );
    assert_exception(
        opaque.delete_rule(&mut owner, usize::MAX),
        CssomException::Security,
    );
    let locked = external(&mut owner, "", true, true);
    assert_exception(
        locked.insert_rule(&mut owner, "!", Some(usize::MAX)),
        CssomException::NotAllowed,
    );
    let sheet = construct(&mut owner);
    assert_exception(
        sheet.insert_rule(&mut owner, "!", Some(usize::MAX)),
        CssomException::Syntax,
    );
    assert_exception(
        sheet.insert_rule(&mut owner, "@import 'x';", Some(usize::MAX)),
        CssomException::Syntax,
    );
    assert_exception(
        sheet.insert_rule(&mut owner, "a {}", Some(usize::MAX)),
        CssomException::IndexSize,
    );
    assert_exception(
        opaque.replace_sync(&mut owner, "a {}"),
        CssomException::NotAllowed,
    );
    // Disabled/media setters do not inherit sheet security/modification guards.
    let _ = opaque.set_disabled(&mut owner, true).unwrap();
    let _ = opaque
        .media(&owner)
        .unwrap()
        .set_media_text(&mut owner, "screen")
        .unwrap();
}

#[test]
fn media_operations_return_unit_recovery_and_canonical_membership() {
    let mut owner = store();
    let sheet = construct(&mut owner);
    let media = sheet.media(&owner).unwrap();
    let empty = media.capture(&owner.snapshot()).unwrap();
    let result = media
        .set_media_text(&mut owner, "screen, ???, print")
        .unwrap();
    let _: &() = result.value();
    assert!(!result.diagnostics().is_empty());
    assert!(result.owner_effects().is_empty());
    assert_eq!(
        media.media_text(&owner, limits()).unwrap(),
        "screen, not all, print"
    );
    assert_eq!(media.length(&owner).unwrap(), 3);
    assert_eq!(
        media.item(&owner, 1, limits()).unwrap().as_deref(),
        Some("not all")
    );
    assert_eq!(
        media
            .item(
                &owner,
                usize::MAX,
                CssSpecifiedValueSerializationLimits::new(0, 0, 0)
            )
            .unwrap(),
        None
    );
    assert_eq!(sheet.media(&owner).unwrap(), media);
    let result = media.append_medium(&mut owner, "SCREEN").unwrap();
    assert!(matches!(
        result.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_exception(
        media.delete_medium(&mut owner, "speech"),
        CssomException::NotFound,
    );
    let revision = owner.snapshot().revision().clone();
    assert!(matches!(
        media
            .append_medium(&mut owner, "screen, speech")
            .unwrap()
            .publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(owner.snapshot().revision(), &revision);
    let _ = media.delete_medium(&mut owner, "SCREEN").unwrap();
    assert_eq!(
        media.media_text(&owner, limits()).unwrap(),
        "not all, print"
    );
    let _ = media.set_media_text(&mut owner, "").unwrap();
    assert_eq!(media.length(&owner).unwrap(), 0);
    assert_eq!(empty.length().unwrap(), 0);
}

#[test]
fn failed_output_constructor_and_media_mutation_preserve_publication() {
    let mut owner = CssomStore::new(
        CssomLimits {
            max_input_bytes: 8,
            ..Default::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let before = owner.snapshot();
    let invalid = CssStyleSheet::new(
        &mut owner,
        document(),
        CssStyleSheetInit {
            media: CssStyleSheetMedia::Text("screen, print"),
            ..Default::default()
        },
    );
    assert!(matches!(invalid, Err(CssomError::Limit { .. })));
    assert_eq!(owner.snapshot().revision(), before.revision());
    let sheet = construct(&mut owner);
    let media = sheet.media(&owner).unwrap();
    let _ = media.set_media_text(&mut owner, "screen").unwrap();
    let before = owner.snapshot();
    assert!(matches!(
        media.set_media_text(&mut owner, "screen, print"),
        Err(CssomError::Limit { .. })
    ));
    assert_eq!(owner.snapshot().revision(), before.revision());
    assert!(matches!(
        media.media_text(&owner, CssSpecifiedValueSerializationLimits::new(9, 9, 5)),
        Err(CssomError::Media(_))
    ));
    assert_eq!(media.media_text(&owner, limits()).unwrap(), "screen");
    let mut bad_document = document();
    bad_document.version.role = CssomInputRole::Owner;
    assert!(matches!(
        CssStyleSheet::new(&mut owner, bad_document, CssStyleSheetInit::default()),
        Err(CssomError::InvalidInput(_))
    ));
    assert_eq!(owner.snapshot().revision(), before.revision());
}

#[test]
fn replacement_detached_work_preserves_old_capture_and_exact_two_publications() {
    fn is_send<T: Send>() {}
    is_send::<CssomStylesheetReplacementWork>();
    let mut owner = store();
    let sheet = construct(&mut owner);
    let _ = sheet.replace_sync(&mut owner, "old {}").unwrap();
    let list = sheet.css_rules(&owner).unwrap();
    let old = list.capture(&owner.snapshot()).unwrap();
    let previous = owner.snapshot().revision().value();
    let request = sheet
        .replace(&mut owner, "@import 'filtered.css'; new {}")
        .unwrap();
    assert_eq!(request.sheet(), &sheet);
    assert_eq!(request.token().sheet(), sheet.id());
    assert!(matches!(
        request.begin_publication(),
        CssomPublication::Changed(_)
    ));
    assert_eq!(owner.snapshot().revision().value(), previous + 1);
    assert_eq!(list.length(&owner).unwrap(), 1);
    assert_exception(
        sheet.insert_rule(&mut owner, "blocked {}", None),
        CssomException::NotAllowed,
    );
    let work = std::thread::spawn(move || request.parse()).join().unwrap();
    let result = work.finish(&mut owner).unwrap();
    assert_eq!(result.value().sheet(), &sheet);
    assert!(matches!(
        result.value().disposition(),
        CssomReplaceOutcome::Replaced
    ));
    assert_eq!(owner.snapshot().revision().value(), previous + 2);
    assert!(result.owner_effects().is_empty());
    assert_eq!(sheet.css_rules(&owner).unwrap(), list);
    assert_eq!(
        rule_text(&list.item(&owner, 0).unwrap().unwrap(), &owner),
        "new { }"
    );
    drop(owner);
    assert_eq!(
        old.item(0)
            .unwrap()
            .unwrap()
            .css_text(&CssomRuleFormatLimits::default())
            .unwrap(),
        "old { }"
    );
}

#[test]
fn stale_replacement_returns_failed_disposition_and_cleanup_publication() {
    let mut owner = store();
    let sheet = construct(&mut owner);
    let _ = sheet.replace_sync(&mut owner, "old {}").unwrap();
    let list = sheet.css_rules(&owner).unwrap();
    let original = list.item(&owner, 0).unwrap().unwrap();
    let work = sheet.replace(&mut owner, "new {}").unwrap().parse();
    let mut edit = batch(&owner);
    edit.update_context(CssomContext {
        inputs: vec![CssomInput {
            version: version(CssomInputRole::ParserMode, "mode", 1),
            data: CssomInputData::ParserMode(CssParserContext::new(CssParserMode::Quirks)),
        }],
        linked: Vec::new(),
    })
    .unwrap();
    owner.commit(edit).unwrap();
    let before = owner.snapshot().revision().value();
    let result = work.finish(&mut owner).unwrap();
    assert!(matches!(
        result.value().disposition(),
        CssomReplaceOutcome::Failed(CssomError::StaleContext)
    ));
    assert_eq!(result.value().sheet(), &sheet);
    assert!(matches!(result.publication(), CssomPublication::Changed(_)));
    assert_eq!(owner.snapshot().revision().value(), before + 1);
    assert_eq!(list.item(&owner, 0).unwrap(), Some(original));
    assert!(
        owner
            .pending_sheet_replacement(sheet.id())
            .unwrap()
            .is_none()
    );
    let _ = sheet.insert_rule(&mut owner, "unlocked {}", None).unwrap();
}

#[test]
fn parse_resource_failure_and_terminal_reservations_retain_cleanup() {
    let mut owner = CssomStore::new(
        CssomLimits {
            max_revision: 3,
            summary_history: 1,
            ..Default::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let sheet = construct(&mut owner);
    let source = format!("{}a {{}}{}", "@media all {".repeat(1025), "}".repeat(1025));
    let work = sheet.replace(&mut owner, &source).unwrap().parse();
    assert!(matches!(
        sheet.set_disabled(&mut owner, false).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert!(matches!(
        sheet.set_disabled(&mut owner, true),
        Err(CssomError::RevisionExhausted)
    ));
    let result = work.finish(&mut owner).unwrap();
    assert!(matches!(
        result.value().disposition(),
        CssomReplaceOutcome::Failed(CssomError::ParseResource(_))
    ));
    assert!(matches!(result.publication(), CssomPublication::Changed(_)));
    assert_eq!(owner.snapshot().revision().value(), 3);
    assert!(
        !owner
            .snapshot()
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .disallow_modification
    );
    assert!(
        owner
            .pending_sheet_replacement(sheet.id())
            .unwrap()
            .is_none()
    );
}

#[test]
fn foreign_work_does_not_clean_other_owner_and_native_cancel_remains_available() {
    let mut owner = store();
    let sheet = construct(&mut owner);
    let request = sheet.replace(&mut owner, "a {}").unwrap();
    let token = request.token().clone();
    let mut foreign = store();
    assert!(matches!(
        request.parse().finish(&mut foreign),
        Err(CssomError::ForeignOwner)
    ));
    assert_eq!(
        owner.pending_sheet_replacement(sheet.id()).unwrap(),
        Some(&token)
    );
    owner.cancel_replace_sheet(&token).unwrap();
    assert!(matches!(
        owner.cancel_replace_sheet(&token),
        Err(CssomError::StaleReplacement)
    ));
    let request = sheet.replace(&mut owner, "b {}").unwrap();
    let result = request.cancel(&mut owner).unwrap();
    assert!(matches!(
        result.value().disposition(),
        CssomReplaceOutcome::Cancelled
    ));
    assert_eq!(result.value().sheet(), &sheet);
    let work = sheet.replace(&mut owner, "c {}").unwrap().parse();
    let result = work.cancel(&mut owner).unwrap();
    assert!(matches!(
        result.value().disposition(),
        CssomReplaceOutcome::Cancelled
    ));
    assert_eq!(sheet.css_rules(&owner).unwrap().length(&owner).unwrap(), 0);
}

#[test]
fn imported_sheet_parent_and_owner_rule_remain_typed_captured_relationships() {
    let mut owner = store();
    let parent = external(&mut owner, "@import 'child.css';", true, false);
    let child = external(&mut owner, "inside {}", true, false);
    let import = parent
        .css_rules(&owner)
        .unwrap()
        .item(&owner, 0)
        .unwrap()
        .unwrap();
    let mut edit = batch(&owner);
    edit.associate_import(
        import.id(),
        child.id(),
        version(CssomInputRole::Import, "child", 1),
        Some("https://host.test/child.css".into()),
    )
    .unwrap();
    owner.commit(edit).unwrap();
    assert_eq!(child.owner_rule(&owner).unwrap(), Some(import.clone()));
    assert_eq!(
        child.as_style_sheet().parent_style_sheet(&owner).unwrap(),
        Some(parent.clone())
    );
    let captured = child.capture(&owner.snapshot()).unwrap();
    let revision = captured.snapshot().revision().clone();
    assert_eq!(
        captured.parent_style_sheet().unwrap().unwrap().handle(),
        &parent
    );
    assert_eq!(
        captured
            .owner_rule()
            .unwrap()
            .unwrap()
            .snapshot()
            .revision(),
        &revision
    );
    let _ = parent.delete_rule(&mut owner, 0).unwrap();
    assert!(child.parent_style_sheet(&owner).unwrap().is_none());
    drop(owner);
    assert_eq!(captured.owner_rule().unwrap().unwrap().handle(), &import);
    assert_eq!(
        captured
            .as_style_sheet()
            .parent_style_sheet()
            .unwrap()
            .unwrap()
            .handle(),
        &parent
    );
}

#[test]
fn namespace_and_hierarchy_failures_keep_typed_list_membership_atomic() {
    let mut owner = store();
    let sheet = external(&mut owner, "@layer seed; @import 'x';", true, false);
    let list = sheet.css_rules(&owner).unwrap();
    let revision = owner.snapshot().revision().clone();
    assert_exception(
        sheet.insert_rule(&mut owner, "@namespace p 'urn:p';", Some(2)),
        CssomException::InvalidState,
    );
    assert_eq!(list.length(&owner).unwrap(), 2);
    assert_eq!(owner.snapshot().revision(), &revision);
    assert_eq!(
        *sheet
            .insert_rule(&mut owner, "b {}", Some(2))
            .unwrap()
            .value(),
        2
    );
    let revision = owner.snapshot().revision().clone();
    assert_exception(
        sheet.insert_rule(&mut owner, "@import 'late';", Some(3)),
        CssomException::HierarchyRequest,
    );
    assert_exception(
        sheet.insert_rule(&mut owner, "@namespace p 'urn:p';", Some(3)),
        CssomException::HierarchyRequest,
    );
    assert_eq!(list.length(&owner).unwrap(), 3);
    assert_eq!(owner.snapshot().revision(), &revision);
}

#[test]
fn terminal_reports_move_into_outcomes_on_success_and_failed_adoption() {
    use surgeist_css::CssRecoveryAction;
    let text = "a { invalid: ; color:red; }";
    for max_objects in [5, 100] {
        let mut owner = CssomStore::new(
            CssomLimits {
                max_objects,
                ..Default::default()
            },
            CssomContext::default(),
        )
        .unwrap();
        let sheet = construct(&mut owner);
        let work = sheet.replace(&mut owner, text).unwrap().parse();
        assert_eq!(work.diagnostics().len(), 1);
        assert_eq!(
            work.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        let before = owner.snapshot().revision().value();
        let result = work.finish(&mut owner).unwrap();
        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(
            result.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        assert_eq!(owner.snapshot().revision().value(), before + 1);
        assert!(matches!(result.publication(), CssomPublication::Changed(_)));
        assert!(
            owner
                .pending_sheet_replacement(sheet.id())
                .unwrap()
                .is_none()
        );
        let snapshot = owner.snapshot();
        if max_objects == 5 {
            // Global sheet list + sheet + media + rule list and pending slot fit;
            // the two additional retained rule/block objects do not.
            assert!(matches!(
                result.value().disposition(),
                CssomReplaceOutcome::Failed(CssomError::Limit {
                    resource: "objects",
                    maximum: 5
                })
            ));
            assert!(snapshot.sheet(sheet.id()).unwrap().diagnostics().is_empty());
            assert_eq!(sheet.css_rules(&owner).unwrap().length(&owner).unwrap(), 0);
        } else {
            assert!(matches!(
                result.value().disposition(),
                CssomReplaceOutcome::Replaced
            ));
            assert_eq!(snapshot.sheet(sheet.id()).unwrap().diagnostics().len(), 1);
            assert_eq!(sheet.css_rules(&owner).unwrap().length(&owner).unwrap(), 1);
        }
        assert!(
            !snapshot
                .sheet(sheet.id())
                .unwrap()
                .inputs()
                .disallow_modification
        );
    }
}

#[test]
fn sheet_list_items_expose_css_stylesheet_interface_and_retained_rule_chain() {
    // CSSOM StyleSheetList.item returns CSSStyleSheet?, including in a capture.
    let mut owner = store();
    let sheet = construct(&mut owner);
    let _ = sheet.insert_rule(&mut owner, "old {}", None).unwrap();
    let list = owner
        .style_sheet_list(owner.snapshot().sheet_list())
        .unwrap();
    let mut edit = batch(&owner);
    edit.set_sheet_list_membership(list.id(), &[sheet.id().clone()])
        .unwrap();
    owner.commit(edit).unwrap();
    let live_item: CssStyleSheet = list.item(&owner, 0).unwrap().unwrap();
    let live_rules: CssRuleList = live_item.css_rules(&owner).unwrap();
    assert_eq!(live_item, sheet);
    assert_eq!(live_rules.length(&owner).unwrap(), 1);
    let captured_list = list.capture(&owner.snapshot()).unwrap();
    let captured_item: CssomCapture<CssStyleSheet> = captured_list.item(0).unwrap().unwrap();
    let captured_rules: CssomCapture<CssRuleList> = captured_item.css_rules().unwrap();
    let old_rule = captured_rules.item(0).unwrap().unwrap();
    assert_eq!(captured_item.handle(), &sheet);
    assert_eq!(
        old_rule.snapshot().revision(),
        captured_list.snapshot().revision()
    );
    let _ = live_item.replace_sync(&mut owner, "new {}").unwrap();
    assert_eq!(
        rule_text(&live_rules.item(&owner, 0).unwrap().unwrap(), &owner),
        "new { }"
    );
    assert_eq!(
        old_rule
            .css_text(&CssomRuleFormatLimits::default())
            .unwrap(),
        "old { }"
    );
    drop(owner);
    assert_eq!(
        captured_item
            .css_rules()
            .unwrap()
            .item(0)
            .unwrap()
            .unwrap()
            .handle(),
        old_rule.handle()
    );
    assert_eq!(
        captured_rules
            .item(0)
            .unwrap()
            .unwrap()
            .css_text(&CssomRuleFormatLimits::default())
            .unwrap(),
        "old { }"
    );
}
