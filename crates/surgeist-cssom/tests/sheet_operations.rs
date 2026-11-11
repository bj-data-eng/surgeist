//! Public consumer expectations from the selected CSSOM §6.1/§6.2 sheet algorithms.
use surgeist_css::component::CssValueOrigin;
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::rule::CssRule;
use surgeist_css::rule::CssSheet;
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::{CssParser, CssParserContext, CssParserMode};
use surgeist_cssom::*;

fn version(role: CssomInputRole, identity: &str, revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: identity.into(),
        revision,
    }
}
fn document() -> CssomConstructorDocument {
    CssomConstructorDocument {
        version: version(CssomInputRole::Document, "d", 1),
        identity: "d".into(),
        base_location: String::new(),
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
fn construct(store: &mut CssomStore) -> CssomSheetId {
    let mut edit = batch(store);
    let construction = edit
        .construct_sheet(document(), CssomSheetInit::default())
        .unwrap();
    store
        .commit(edit)
        .unwrap()
        .sheet(construction.ticket())
        .unwrap()
        .clone()
}
fn captured_sheet(
    store: &CssomStore,
    snapshot: &CssomSnapshot,
    id: &CssomSheetId,
) -> CssomCapture<CssStyleSheet> {
    store
        .css_style_sheet(id)
        .unwrap()
        .capture(snapshot)
        .unwrap()
}
fn captured_list(store: &CssomStore, snapshot: &CssomSnapshot) -> CssomCapture<StyleSheetList> {
    store
        .style_sheet_list(snapshot.sheet_list())
        .unwrap()
        .capture(snapshot)
        .unwrap()
}
fn rules(store: &CssomStore, sheet: &CssomSheetId) -> Vec<CssomRuleId> {
    let snapshot = store.snapshot();
    snapshot
        .rules(snapshot.sheet(sheet).unwrap().rules())
        .unwrap()
        .to_vec()
}
fn replace(store: &mut CssomStore, sheet: &CssomSheetId, text: &str) {
    let mut edit = batch(store);
    edit.replace_sheet_sync(sheet, text).unwrap();
    store.commit(edit).unwrap();
}

#[test]
fn constructor_defaults_fixed_location_and_explicit_ordered_host_membership() {
    let mut owner = store(CssomLimits::default());
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    let mut current = document();
    current.base_location = "https://host.example/current/base".into();
    let first = edit
        .construct_sheet(current, CssomSheetInit::default())
        .unwrap();
    let second = edit
        .construct_sheet(
            document(),
            CssomSheetInit {
                base_url: Some(std::sync::Arc::new(Vec::new())),
                disabled: true,
                ..Default::default()
            },
        )
        .unwrap();
    let committed = owner.commit(edit).unwrap();
    let first = committed.sheet(first.ticket()).unwrap().clone();
    let second = committed.sheet(second.ticket()).unwrap().clone();
    let captured = owner.snapshot();
    assert_eq!(
        captured_sheet(&owner, &captured, &first).r#type().unwrap(),
        "text/css"
    );
    assert_eq!(
        captured_sheet(&owner, &captured, &first).href().unwrap(),
        Some("https://host.example/current/base")
    );
    assert_eq!(
        captured_sheet(&owner, &captured, &first)
            .owner_node()
            .unwrap(),
        None
    );
    assert_eq!(
        captured_sheet(&owner, &captured, &first).title().unwrap(),
        None
    );
    assert!(
        captured_sheet(&owner, &captured, &first)
            .owner_rule()
            .unwrap()
            .is_none()
    );
    assert_eq!(captured.parent_sheet(&first).unwrap(), None);
    assert!(
        !captured_sheet(&owner, &captured, &first)
            .disabled()
            .unwrap()
    );
    assert!(
        captured_sheet(&owner, &captured, &second)
            .disabled()
            .unwrap()
    );
    let facts = captured.sheet(&first).unwrap().inputs();
    assert_eq!(facts.base_url, None);
    assert_eq!(
        captured.sheet(&second).unwrap().inputs().base_url,
        Some(std::sync::Arc::new(Vec::new()))
    );
    assert_eq!(facts.constructor_document.as_deref(), Some("d"));
    assert!(
        facts.origin_clean && facts.constructed && !facts.disallow_modification && !facts.alternate
    );
    assert_eq!(
        captured_sheet(&owner, &captured, &first)
            .css_rules()
            .map(|capture| capture.handle().id().clone())
            .unwrap(),
        captured_sheet(&owner, &captured, &first)
            .rules()
            .map(|capture| capture.handle().id().clone())
            .unwrap()
    );
    assert!(captured.sheet_rules(&first).unwrap().is_empty());
    assert!(
        captured
            .media(
                &captured_sheet(&owner, &captured, &first)
                    .media()
                    .map(|capture| capture.handle().id().clone())
                    .unwrap()
            )
            .unwrap()
            .queries()
            .is_empty()
    );
    assert_eq!(captured_list(&owner, &captured).length().unwrap(), 0);
    let mut edit = batch(&owner);
    assert!(
        edit.set_sheet_list_membership(
            captured.sheet_list(),
            &[
                CssomSheetListTarget::Existing(&second),
                CssomSheetListTarget::Existing(&first)
            ]
        )
        .unwrap()
    );
    owner.commit(edit).unwrap();
    let selected = owner.snapshot();
    assert_eq!(
        captured_list(&owner, &selected)
            .item(0)
            .map(|value| value.map(|capture| capture.handle().id().clone()))
            .unwrap(),
        Some(second.clone())
    );
    assert_eq!(
        captured_list(&owner, &selected)
            .item(1)
            .map(|value| value.map(|capture| capture.handle().id().clone()))
            .unwrap(),
        Some(first.clone())
    );
    assert_eq!(
        captured_list(&owner, &selected)
            .item(2)
            .map(|value| value.map(|capture| capture.handle().id().clone()))
            .unwrap(),
        None
    );
    assert_eq!(selected.sheet_list(), before.sheet_list());
    assert!(captured.sheets().is_empty() && before.sheets().is_empty());
    let revision = selected.revision().clone();
    let mut edit = batch(&owner);
    assert!(
        !edit
            .set_sheet_list_membership(
                selected.sheet_list(),
                &[
                    CssomSheetListTarget::Existing(&second),
                    CssomSheetListTarget::Existing(&first)
                ]
            )
            .unwrap()
    );
    assert!(
        matches!(owner.commit(edit).unwrap().publication(), CssomPublication::Unchanged { revision: value } if value == &revision)
    );
}

#[test]
fn constructor_media_list_copies_serialized_recovery_to_an_independent_identity() {
    let mut source = store(CssomLimits::default());
    let mut edit = batch(&source);
    let construction = edit
        .construct_sheet(
            document(),
            CssomSheetInit {
                media: CssomInitialMedia::Text("screen and, print".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!construction.media_diagnostics().is_empty());
    let source_sheet = source
        .commit(edit)
        .unwrap()
        .sheet(construction.ticket())
        .unwrap()
        .clone();
    let original = source.snapshot();
    let original_id = captured_sheet(&source, &original, &source_sheet)
        .media()
        .map(|capture| capture.handle().id().clone())
        .unwrap();
    assert_eq!(
        original
            .media(&original_id)
            .unwrap()
            .serialize_cssom(Default::default())
            .unwrap()
            .as_css(),
        "not all, print"
    );
    let mut owner = store(CssomLimits::default());
    let mut edit = batch(&owner);
    let copied = edit
        .construct_sheet(
            document(),
            CssomSheetInit {
                media: CssomInitialMedia::List {
                    snapshot: &original,
                    id: &original_id,
                },
                ..Default::default()
            },
        )
        .unwrap();
    assert!(copied.media_diagnostics().is_empty());
    let sheet = owner
        .commit(edit)
        .unwrap()
        .sheet(copied.ticket())
        .unwrap()
        .clone();
    let capture = owner.snapshot();
    let copied_id = captured_sheet(&owner, &capture, &sheet)
        .media()
        .map(|capture| capture.handle().id().clone())
        .unwrap();
    assert_ne!(copied_id, original_id);
    let copied = capture.media(&copied_id).unwrap();
    assert_eq!(
        copied.serialize_cssom(Default::default()).unwrap().as_css(),
        "not all, print"
    );
    assert!(!copied.queries()[0].is_guaranteed_false());
    assert!(
        matches!(copied.queries()[0].origin(), CssValueOrigin::Parsed(origin) if origin.source().as_str() == "not all, print")
    );
    drop(source);
    drop(original);
    drop(owner);
    assert_eq!(
        capture
            .media(&copied_id)
            .unwrap()
            .serialize_cssom(Default::default())
            .unwrap()
            .as_css(),
        "not all, print"
    );
}

#[test]
fn owner_attributes_and_disabled_do_not_inherit_css_rules_security_guards() {
    let mut owner = store(CssomLimits::default());
    let mut facts = CssomSheetInputs::external(version(CssomInputRole::Owner, "style", 1), false);
    facts.owner = Some("style".into());
    facts.location = Some("https://host.example/fixed".into());
    facts.title = std::sync::Arc::new("old".encode_utf16().collect());
    facts.disallow_modification = true;
    facts.media = CssMediaQueryList::parse(&CssParser::default(), "screen")
        .into_parts()
        .0;
    let mut edit = batch(&owner);
    let ticket = edit.create_parsed_sheet("a {}", facts).unwrap();
    let sheet = owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let before = owner.snapshot();
    let media = captured_sheet(&owner, &before, &sheet)
        .media()
        .map(|capture| capture.handle().id().clone())
        .unwrap()
        .clone();
    assert!(matches!(
        captured_sheet(&owner, &before, &sheet)
            .css_rules()
            .map(|capture| capture.handle().id().clone()),
        Err(CssomError::Source(CssomException::Security))
    ));
    assert!(matches!(
        captured_sheet(&owner, &before, &sheet)
            .rules()
            .map(|capture| capture.handle().id().clone()),
        Err(CssomError::Source(CssomException::Security))
    ));
    let mut edit = batch(&owner);
    assert!(edit.set_disabled(&sheet, true).unwrap());
    assert!(
        edit.update_sheet_owner_attributes(
            &sheet,
            CssomSheetOwnerAttributes {
                owner: "style",
                version: version(CssomInputRole::Owner, "style", 2),
                title: None,
                media: CssomOwnerMediaInput::Utf8(None)
            }
        )
        .unwrap()
        .changed()
    );
    owner.commit(edit).unwrap();
    let after = owner.snapshot();
    assert_eq!(
        captured_sheet(&owner, &after, &sheet).title().unwrap(),
        None
    );
    assert!(after.media(&media).unwrap().queries().is_empty());
    assert_eq!(
        captured_sheet(&owner, &after, &sheet)
            .media()
            .map(|capture| capture.handle().id().clone())
            .unwrap(),
        media
    );
    assert_eq!(
        captured_sheet(&owner, &after, &sheet).href().unwrap(),
        captured_sheet(&owner, &before, &sheet).href().unwrap()
    );
    assert_eq!(
        captured_sheet(&owner, &after, &sheet).owner_node().unwrap(),
        Some("style")
    );
    assert!(captured_sheet(&owner, &after, &sheet).disabled().unwrap());
    assert_eq!(
        captured_sheet(&owner, &before, &sheet).title().unwrap(),
        Some("old".encode_utf16().collect::<Vec<_>>().as_slice())
    );
    assert!(!captured_sheet(&owner, &before, &sheet).disabled().unwrap());
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.update_sheet_owner_attributes(
            &sheet,
            CssomSheetOwnerAttributes {
                owner: "other",
                version: version(CssomInputRole::Owner, "other", 1),
                title: Some(&[119, 114, 111, 110, 103]),
                media: CssomOwnerMediaInput::Utf8(None)
            }
        ),
        Err(CssomError::InvalidInput(_))
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), after.revision());
}

#[test]
fn synchronous_replacement_recovers_filters_imports_and_detaches_only_old_roots() {
    let mut owner = store(CssomLimits::default());
    let sheet = construct(&mut owner);
    replace(&mut owner, &sheet, "@media all { a {color:red;} }");
    let before = owner.snapshot();
    let old_root = before.sheet_rules(&sheet).unwrap()[0].clone();
    let child = before
        .rules(before.rule(&old_root).unwrap().data().children().unwrap())
        .unwrap()[0]
        .clone();
    let rule_list = captured_sheet(&owner, &before, &sheet)
        .css_rules()
        .map(|capture| capture.handle().id().clone())
        .unwrap()
        .clone();
    let media = captured_sheet(&owner, &before, &sheet)
        .media()
        .map(|capture| capture.handle().id().clone())
        .unwrap()
        .clone();
    let source = "@import 'discard.css'; @unknown; b {color:blue; invalid: ;} c { color: green; }";
    replace(&mut owner, &sheet, source);
    let after = owner.snapshot();
    assert_eq!(
        captured_sheet(&owner, &after, &sheet)
            .css_rules()
            .map(|capture| capture.handle().id().clone())
            .unwrap(),
        rule_list
    );
    assert_eq!(
        captured_sheet(&owner, &after, &sheet)
            .media()
            .map(|capture| capture.handle().id().clone())
            .unwrap(),
        media
    );
    assert_eq!(after.sheet_rules(&sheet).unwrap().len(), 2);
    assert!(!after.sheet(&sheet).unwrap().diagnostics().is_empty());
    assert_eq!(
        after.rule(&old_root).unwrap().parent(),
        &CssomParent::Detached
    );
    assert_eq!(
        after.rule(&child).unwrap().parent(),
        &CssomParent::Rule(old_root.clone())
    );
    assert_eq!(after.parent_style_sheet(&child).unwrap(), None);
    assert_eq!(before.parent_style_sheet(&child).unwrap(), Some(&sheet));
    let authored = after.sheet(&sheet).unwrap().authored();
    assert_eq!(authored.rules().len(), 3);
    assert!(
        matches!(&authored.rules()[0], CssRule::Import(rule) if matches!(rule.origin(), CssValueOrigin::Parsed(origin) if origin.source().as_str() == source))
    );
    let first_ids = rules(&owner, &sheet);
    replace(&mut owner, &sheet, source);
    assert_ne!(rules(&owner, &sheet), first_ids);
    replace(&mut owner, &sheet, "invalid-only;");
    assert!(rules(&owner, &sheet).is_empty());
}

#[test]
fn exclusive_async_replacement_keeps_stable_objects_and_exact_tokens() {
    let mut owner = store(CssomLimits::default());
    let sheet = construct(&mut owner);
    replace(&mut owner, &sheet, "a {color:red;}");
    let before = owner.snapshot();
    let list = captured_sheet(&owner, &before, &sheet)
        .css_rules()
        .map(|capture| capture.handle().id().clone())
        .unwrap()
        .clone();
    let media = captured_sheet(&owner, &before, &sheet)
        .media()
        .map(|capture| capture.handle().id().clone())
        .unwrap()
        .clone();
    let job = owner
        .begin_replace_sheet(guard(&owner), &sheet, "@import 'ignored'; b {color:blue;}")
        .unwrap();
    let token = job.token().clone();
    assert!(
        matches!(job.begin_publication(), CssomPublication::Changed(summary) if summary.categories().contains(&CssomChange::Modification))
    );
    assert!(
        owner
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disallow_modification
    );
    assert!(matches!(
        owner.begin_replace_sheet(guard(&owner), &sheet, "c {}"),
        Err(CssomError::Source(CssomException::NotAllowed))
    ));
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.replace_sheet_sync(&sheet, "c {}"),
        Err(CssomError::Source(CssomException::NotAllowed))
    ));
    let completion = owner
        .finish_replace_sheet(guard(&owner), job.parse())
        .unwrap();
    assert!(matches!(
        completion.outcome(),
        CssomReplaceOutcome::Replaced
    ));
    let after = owner.snapshot();
    assert!(!after.sheet(&sheet).unwrap().inputs().disallow_modification);
    assert!(owner.pending_sheet_replacement(&sheet).unwrap().is_none());
    assert_eq!(
        captured_sheet(&owner, &after, &sheet)
            .css_rules()
            .map(|capture| capture.handle().id().clone())
            .unwrap(),
        list
    );
    assert_eq!(
        captured_sheet(&owner, &after, &sheet)
            .media()
            .map(|capture| capture.handle().id().clone())
            .unwrap(),
        media
    );
    assert_eq!(after.sheet_rules(&sheet).unwrap().len(), 1);
    assert_eq!(before.sheet_rules(&sheet).unwrap().len(), 1);
    let newer = owner
        .begin_replace_sheet(guard(&owner), &sheet, "c {}")
        .unwrap();
    assert!(matches!(
        owner.cancel_replace_sheet(&token),
        Err(CssomError::StaleReplacement)
    ));
    assert_eq!(
        owner.pending_sheet_replacement(&sheet).unwrap(),
        Some(newer.token())
    );
    owner.cancel_replace_sheet(newer.token()).unwrap();
}

#[test]
fn dropped_job_foreign_work_and_reused_work_cannot_release_another_operation() {
    let mut owner = store(CssomLimits::default());
    let sheet = construct(&mut owner);
    let dropped = owner
        .begin_replace_sheet(guard(&owner), &sheet, "a {}")
        .unwrap();
    let token = dropped.token().clone();
    drop(dropped);
    assert_eq!(
        owner.pending_sheet_replacement(&sheet).unwrap(),
        Some(&token)
    );
    assert!(matches!(
        owner.cancel_replace_sheet(&token).unwrap().outcome(),
        CssomReplaceOutcome::Cancelled
    ));
    let job = owner
        .begin_replace_sheet(guard(&owner), &sheet, "b {}")
        .unwrap();
    let reused = job.clone().parse();
    let mut foreign = store(CssomLimits::default());
    let foreign_sheet = construct(&mut foreign);
    let foreign_job = foreign
        .begin_replace_sheet(guard(&foreign), &foreign_sheet, "foreign {}")
        .unwrap();
    assert!(matches!(
        owner.finish_replace_sheet(guard(&owner), foreign_job.parse()),
        Err(CssomError::ForeignOwner)
    ));
    assert_eq!(
        owner.pending_sheet_replacement(&sheet).unwrap(),
        Some(job.token())
    );
    owner
        .finish_replace_sheet(guard(&owner), job.parse())
        .unwrap();
    assert!(matches!(
        owner.finish_replace_sheet(guard(&owner), reused),
        Err(CssomError::StaleReplacement)
    ));
    let foreign_token = foreign
        .pending_sheet_replacement(&foreign_sheet)
        .unwrap()
        .unwrap()
        .clone();
    foreign.cancel_replace_sheet(&foreign_token).unwrap();
}

#[test]
fn correct_token_stale_guard_failure_cleans_up_while_preserving_old_rules() {
    let mut owner = store(CssomLimits::default());
    let sheet = construct(&mut owner);
    replace(&mut owner, &sheet, "a {}");
    let original = rules(&owner, &sheet);
    let job = owner
        .begin_replace_sheet(guard(&owner), &sheet, "b {}")
        .unwrap();
    let stale = guard(&owner);
    let mut edit = batch(&owner);
    edit.set_disabled(&sheet, true).unwrap();
    owner.commit(edit).unwrap();
    let completion = owner.finish_replace_sheet(stale, job.parse()).unwrap();
    assert!(matches!(
        completion.outcome(),
        CssomReplaceOutcome::Failed(CssomError::StaleRevision)
    ));
    assert_eq!(rules(&owner, &sheet), original);
    assert!(
        !owner
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disallow_modification
    );
    assert!(
        captured_sheet(&owner, &owner.snapshot(), &sheet)
            .disabled()
            .unwrap()
    );
    assert!(owner.pending_sheet_replacement(&sheet).unwrap().is_none());
    let job = owner
        .begin_replace_sheet(guard(&owner), &sheet, "c {}")
        .unwrap();
    assert!(matches!(
        owner
            .finish_replace_sheet(guard(&owner), job.parse())
            .unwrap()
            .outcome(),
        CssomReplaceOutcome::Replaced
    ));
}

#[test]
fn parser_input_drift_fails_but_unrelated_facts_and_other_sheet_jobs_compose() {
    let mut owner = store(CssomLimits::default());
    let first = construct(&mut owner);
    let second = construct(&mut owner);
    let first_job = owner
        .begin_replace_sheet(guard(&owner), &first, "a {}")
        .unwrap();
    let second_job = owner
        .begin_replace_sheet(guard(&owner), &second, "b {}")
        .unwrap();
    let mut edit = batch(&owner);
    edit.update_context(CssomContext {
        inputs: vec![CssomInput {
            version: version(CssomInputRole::Support, "support", 1),
            data: CssomInputData::Support(CssomDeclarationSupport::default()),
        }],
        linked: Vec::new(),
    })
    .unwrap();
    owner.commit(edit).unwrap();
    assert!(matches!(
        owner
            .finish_replace_sheet(guard(&owner), second_job.parse())
            .unwrap()
            .outcome(),
        CssomReplaceOutcome::Replaced
    ));
    assert!(matches!(
        owner
            .finish_replace_sheet(guard(&owner), first_job.parse())
            .unwrap()
            .outcome(),
        CssomReplaceOutcome::Replaced
    ));
    let original = rules(&owner, &first);
    let job = owner
        .begin_replace_sheet(guard(&owner), &first, "a { width:10; }")
        .unwrap();
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
    assert!(matches!(
        owner
            .finish_replace_sheet(guard(&owner), job.parse())
            .unwrap()
            .outcome(),
        CssomReplaceOutcome::Failed(CssomError::StaleContext)
    ));
    assert_eq!(rules(&owner, &first), original);
    assert!(
        !owner
            .snapshot()
            .sheet(&first)
            .unwrap()
            .inputs()
            .disallow_modification
    );
}

#[test]
fn revision_reservation_preserves_noops_and_terminal_cleanup_at_the_last_slot() {
    let mut owner = store(CssomLimits {
        max_revision: 3,
        summary_history: 1,
        ..Default::default()
    });
    let sheet = construct(&mut owner);
    let job = owner
        .begin_replace_sheet(guard(&owner), &sheet, "a {}")
        .unwrap();
    let pending = owner.snapshot();
    let mut edit = batch(&owner);
    assert!(!edit.set_disabled(&sheet, false).unwrap());
    assert!(matches!(
        owner.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(owner.snapshot().revision(), pending.revision());
    let mut edit = batch(&owner);
    edit.set_disabled(&sheet, true).unwrap();
    assert!(matches!(
        owner.commit(edit),
        Err(CssomError::RevisionExhausted)
    ));
    assert_eq!(
        owner.pending_sheet_replacement(&sheet).unwrap(),
        Some(job.token())
    );
    assert!(matches!(
        owner.cancel_replace_sheet(job.token()).unwrap().outcome(),
        CssomReplaceOutcome::Cancelled
    ));
    assert_eq!(owner.snapshot().revision().value(), 3);
    assert!(
        !owner
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disallow_modification
    );
    assert!(matches!(
        owner.begin_replace_sheet(guard(&owner), &sheet, ""),
        Err(CssomError::RevisionExhausted)
    ));
    let mut insufficient = store(CssomLimits {
        max_revision: 2,
        ..Default::default()
    });
    let sheet = construct(&mut insufficient);
    let before = insufficient.snapshot();
    assert!(matches!(
        insufficient.begin_replace_sheet(guard(&insufficient), &sheet, ""),
        Err(CssomError::RevisionExhausted)
    ));
    assert_eq!(insufficient.snapshot().revision(), before.revision());
    assert!(
        !insufficient
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disallow_modification
    );
}

#[test]
fn pending_record_quota_blocks_unrelated_admission_without_stranding_cleanup() {
    let mut owner = store(CssomLimits {
        max_objects: 7,
        ..Default::default()
    });
    let sheet = construct(&mut owner);
    let job = owner
        .begin_replace_sheet(guard(&owner), &sheet, "")
        .unwrap();
    let mut edit = batch(&owner);
    edit.construct_sheet(document(), CssomSheetInit::default())
        .unwrap();
    assert!(matches!(
        owner.commit(edit),
        Err(CssomError::Limit {
            resource: "objects",
            ..
        })
    ));
    owner.cancel_replace_sheet(job.token()).unwrap();
    assert!(
        !owner
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disallow_modification
    );
    construct(&mut owner);
    let mut bytes = store(CssomLimits {
        max_string_bytes: 8,
        ..Default::default()
    });
    let sheet = construct(&mut bytes);
    let job = bytes
        .begin_replace_sheet(guard(&bytes), &sheet, "")
        .unwrap();
    let mut edit = batch(&bytes);
    edit.update_context(CssomContext {
        inputs: vec![CssomInput {
            version: version(CssomInputRole::Support, "xxxxxs", 1),
            data: CssomInputData::Support(CssomDeclarationSupport::default()),
        }],
        linked: Vec::new(),
    })
    .unwrap();
    assert!(matches!(bytes.commit(edit), Err(CssomError::Limit { .. })));
    bytes.cancel_replace_sheet(job.token()).unwrap();
    assert!(
        !bytes
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disallow_modification
    );
}

#[test]
fn adoption_and_parser_resource_failure_unlock_without_partial_or_empty_publication() {
    for limits in [
        CssomLimits {
            max_identity: 5,
            ..Default::default()
        },
        CssomLimits {
            max_objects: 5,
            ..Default::default()
        },
        CssomLimits {
            max_entries: 0,
            ..Default::default()
        },
    ] {
        let mut owner = store(limits);
        let sheet = construct(&mut owner);
        let job = owner
            .begin_replace_sheet(guard(&owner), &sheet, "a {color:red;}")
            .unwrap();
        let completion = owner
            .finish_replace_sheet(guard(&owner), job.parse())
            .unwrap();
        assert!(matches!(
            completion.outcome(),
            CssomReplaceOutcome::Failed(CssomError::IdentityExhausted | CssomError::Limit { .. })
        ));
        assert!(rules(&owner, &sheet).is_empty());
        assert!(
            !owner
                .snapshot()
                .sheet(&sheet)
                .unwrap()
                .inputs()
                .disallow_modification
        );
        assert!(owner.pending_sheet_replacement(&sheet).unwrap().is_none());
    }
    let mut owner = store(CssomLimits::default());
    let sheet = construct(&mut owner);
    replace(&mut owner, &sheet, "a {}");
    let old = rules(&owner, &sheet);
    let source = format!("{}a {{}}{}", "@media all {".repeat(1025), "}".repeat(1025));
    let job = owner
        .begin_replace_sheet(guard(&owner), &sheet, &source)
        .unwrap();
    assert!(matches!(
        owner
            .finish_replace_sheet(guard(&owner), job.parse())
            .unwrap()
            .outcome(),
        CssomReplaceOutcome::Failed(CssomError::ParseResource(_))
    ));
    assert_eq!(rules(&owner, &sheet), old);
    assert!(
        !owner
            .snapshot()
            .sheet(&sheet)
            .unwrap()
            .inputs()
            .disallow_modification
    );
    let job = owner
        .begin_replace_sheet(guard(&owner), &sheet, "invalid-only;")
        .unwrap();
    assert!(matches!(
        owner
            .finish_replace_sheet(guard(&owner), job.parse())
            .unwrap()
            .outcome(),
        CssomReplaceOutcome::Replaced
    ));
    assert!(rules(&owner, &sheet).is_empty());
}

#[test]
fn replacement_source_guards_precede_parsing_without_an_origin_clean_guard() {
    let mut owner = store(CssomLimits::default());
    let mut edit = batch(&owner);
    let external = edit
        .create_typed_sheet(
            CssSheet::new(),
            CssomSheetInputs::external(version(CssomInputRole::Owner, "external", 1), false),
        )
        .unwrap();
    let mut facts = CssomSheetInputs::constructed(
        version(CssomInputRole::Document, "constructed", 1),
        "d".into(),
        None,
    );
    facts.origin_clean = false;
    let constructed = edit.create_typed_sheet(CssSheet::new(), facts).unwrap();
    let committed = owner.commit(edit).unwrap();
    let external = committed.sheet(&external).unwrap().clone();
    let constructed = committed.sheet(&constructed).unwrap().clone();
    assert!(matches!(
        owner.begin_replace_sheet(guard(&owner), &external, "a {}"),
        Err(CssomError::Source(CssomException::NotAllowed))
    ));
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.replace_sheet_sync(&external, "a {}"),
        Err(CssomError::Source(CssomException::NotAllowed))
    ));
    replace(&mut owner, &constructed, "a {}");
    assert_eq!(rules(&owner, &constructed).len(), 1);
    let job = owner
        .begin_replace_sheet(guard(&owner), &constructed, "b {}")
        .unwrap();
    assert!(matches!(
        owner
            .finish_replace_sheet(guard(&owner), job.parse())
            .unwrap()
            .outcome(),
        CssomReplaceOutcome::Replaced
    ));
    assert!(matches!(
        captured_sheet(&owner, &owner.snapshot(), &constructed)
            .css_rules()
            .map(|capture| capture.handle().id().clone()),
        Err(CssomError::Source(CssomException::Security))
    ));
    let mut bounded = store(CssomLimits {
        max_input_bytes: 0,
        ..Default::default()
    });
    let sheet = construct(&mut bounded);
    let before = bounded.snapshot();
    assert!(matches!(
        bounded.begin_replace_sheet(guard(&bounded), &sheet, "a {}"),
        Err(CssomError::Limit {
            resource: "parse input bytes",
            maximum: 0
        })
    ));
    assert_eq!(bounded.snapshot().revision(), before.revision());
    assert!(bounded.pending_sheet_replacement(&sheet).unwrap().is_none());
    let job = bounded
        .begin_replace_sheet(guard(&bounded), &sheet, "")
        .unwrap();
    assert!(matches!(
        bounded.begin_replace_sheet(guard(&bounded), &sheet, "a {}"),
        Err(CssomError::Source(CssomException::NotAllowed))
    ));
    assert!(matches!(
        bounded
            .finish_replace_sheet(guard(&bounded), job.parse())
            .unwrap()
            .outcome(),
        CssomReplaceOutcome::Replaced
    ));
}

#[test]
fn sheet_insertion_preliminary_and_constructed_import_guards_precede_shared_index() {
    let mut owner = store(CssomLimits::default());
    let sheet = construct(&mut owner);
    for (text, index, expected) in [
        ("a {} b {}", usize::MAX, CssomException::Syntax),
        ("@unknown {}", usize::MAX, CssomException::IndexSize),
        ("@unknown {}", 0, CssomException::Syntax),
        ("@\\69mport;", usize::MAX, CssomException::Syntax),
        ("a {}", 1, CssomException::IndexSize),
    ] {
        let before = owner.snapshot();
        let mut edit = batch(&owner);
        assert!(
            matches!(edit.insert_sheet_rule(&sheet, text, index), Err(CssomError::Source(error)) if error == expected)
        );
        assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
        assert_eq!(owner.snapshot().revision(), before.revision());
    }
    for (origin_clean, expected) in [
        (false, CssomException::Security),
        (true, CssomException::NotAllowed),
    ] {
        let mut facts =
            CssomSheetInputs::external(version(CssomInputRole::Owner, "external", 1), origin_clean);
        facts.disallow_modification = true;
        let mut edit = batch(&owner);
        let ticket = edit.create_typed_sheet(CssSheet::new(), facts).unwrap();
        let external = owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
        let mut edit = batch(&owner);
        assert!(
            matches!(edit.insert_sheet_rule(&external, "a {} b {}", usize::MAX), Err(CssomError::Source(error)) if error == expected)
        );
        assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    }
}

#[test]
fn shared_insertion_retains_the_actual_occurrence_and_recovered_declaration_neighbors() {
    let mut owner = store(CssomLimits::default());
    let sheet = construct(&mut owner);
    let before = owner.snapshot();
    let list = captured_sheet(&owner, &before, &sheet)
        .css_rules()
        .map(|capture| capture.handle().id().clone())
        .unwrap()
        .clone();
    let source = String::from("a {color:red; invalid: ; width:2px;}");
    let mut edit = batch(&owner);
    let (index, ticket) = edit.insert_sheet_rule(&sheet, source.as_str(), 0).unwrap();
    assert_eq!(index, 0);
    let committed = owner.commit(edit).unwrap();
    let rule = committed.rule(&ticket).unwrap().clone();
    drop(source);
    let after = owner.snapshot();
    assert_eq!(
        captured_sheet(&owner, &after, &sheet)
            .css_rules()
            .map(|capture| capture.handle().id().clone())
            .unwrap(),
        list
    );
    assert_eq!(
        after.rule(&rule).unwrap().parent(),
        &CssomParent::Sheet(sheet.clone())
    );
    let block = after.rule(&rule).unwrap().data().block().unwrap();
    assert_eq!(
        after
            .specified_property_css_text(block, CssSerializationLimits::default())
            .unwrap(),
        "color: red; width: 2px;"
    );
    let CssomBlockData::Properties {
        selected: CssomProjection::Available(selected),
        ..
    } = after.block(block).unwrap().data()
    else {
        panic!("checked properties");
    };
    assert_eq!(
        selected.entries()[0]
            .source()
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        "a {color:red; invalid: ; width:2px;}"
    );
    assert!(before.rules(&list).unwrap().is_empty());
    let mut edit = batch(&owner);
    assert_eq!(edit.remove_sheet_rule(&sheet, None).unwrap(), rule);
    owner.commit(edit).unwrap();
    assert_eq!(
        owner.snapshot().rule(&rule).unwrap().parent(),
        &CssomParent::Detached
    );
    assert_eq!(after.parent_style_sheet(&rule).unwrap(), Some(&sheet));
}

#[test]
fn legacy_add_rule_defaults_end_index_exact_input_and_minus_one_result() {
    let mut owner = store(CssomLimits::default());
    let sheet = construct(&mut owner);
    let mut edit = batch(&owner);
    assert_eq!(edit.add_sheet_rule(&sheet, None, None, None).unwrap(), -1);
    owner.commit(edit).unwrap();
    let mut edit = batch(&owner);
    assert_eq!(
        edit.add_sheet_rule(&sheet, Some("a"), Some("color:red;"), None)
            .unwrap(),
        -1
    );
    assert_eq!(
        edit.add_sheet_rule(&sheet, Some("b"), Some(""), Some(0))
            .unwrap(),
        -1
    );
    owner.commit(edit).unwrap();
    let after = owner.snapshot();
    let ids = after.sheet_rules(&sheet).unwrap();
    let texts = ids
        .iter()
        .map(|id| match after.rule(id).unwrap().authored() {
            CssomAuthoredRule::Rule(rule) => rule.serialize_cssom(Default::default()).unwrap(),
            _ => panic!("ordinary source rule"),
        })
        .collect::<Vec<_>>();
    assert_eq!(texts, ["b { }", "undefined { }", "a { color: red; }"]);
    let block = after.rule(&ids[2]).unwrap().data().block().unwrap();
    let CssomBlockData::Properties {
        selected: CssomProjection::Available(selected),
        ..
    } = after.block(block).unwrap().data()
    else {
        panic!("checked properties");
    };
    assert_eq!(
        selected.entries()[0]
            .source()
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        "a { color:red; }"
    );
    let mut edit = batch(&owner);
    assert_eq!(edit.remove_sheet_rule(&sheet, None).unwrap(), ids[0]);
    owner.commit(edit).unwrap();
    assert_eq!(rules(&owner, &sheet), ids[1..]);
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.add_sheet_rule(&sheet, Some("@import 'x'; a"), Some(""), Some(usize::MAX)),
        Err(CssomError::Source(CssomException::Syntax))
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    let mut limited = store(CssomLimits {
        max_input_bytes: 3,
        ..Default::default()
    });
    let sheet = construct(&mut limited);
    let before = limited.snapshot();
    let mut edit = batch(&limited);
    assert!(matches!(
        edit.add_sheet_rule(&sheet, Some(""), Some(""), None),
        Err(CssomError::Limit {
            resource: "legacy rule input bytes",
            maximum: 3
        })
    ));
    assert!(matches!(
        limited.commit(edit),
        Err(CssomError::BatchAborted)
    ));
    assert_eq!(limited.snapshot().revision(), before.revision());
}

#[test]
fn whole_list_namespace_guards_follow_hierarchy_and_keep_membership_atomic() {
    let mut owner = store(CssomLimits::default());
    let sheet = construct(&mut owner);
    let mut edit = batch(&owner);
    let (_, namespace) = edit
        .insert_sheet_rule(&sheet, "@namespace p 'urn:p';", 0)
        .unwrap();
    let (_, body) = edit.insert_sheet_rule(&sheet, "p|a {}", 1).unwrap();
    let committed = owner.commit(edit).unwrap();
    let namespace = committed.rule(&namespace).unwrap().clone();
    let body = committed.rule(&body).unwrap().clone();
    let before = owner.snapshot();
    for (index, expected) in [
        (1, CssomException::InvalidState),
        (2, CssomException::HierarchyRequest),
    ] {
        let mut edit = batch(&owner);
        assert!(
            matches!(edit.insert_sheet_rule(&sheet, "@namespace q 'urn:q';", index), Err(CssomError::Source(error)) if error == expected)
        );
        assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    }
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.remove_sheet_rule(&sheet, None),
        Err(CssomError::Source(CssomException::InvalidState))
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(rules(&owner, &sheet), [namespace, body]);
    assert_eq!(owner.snapshot().revision(), before.revision());
}

#[test]
fn exact_base_url_backing_is_shared_by_sheets_and_pending_replacement() {
    use std::sync::Arc;

    fn make_base() -> Arc<Vec<u16>> {
        let mut units = Vec::with_capacity(32);
        units.extend([0xd800, 0, 0xdc00]);
        Arc::new(units)
    }

    let base = make_base();
    let base_bytes = base.capacity() * size_of::<u16>();
    let document = document();
    let fixed =
        document.version.identity.len() + document.identity.len() + document.base_location.len();
    let construct = |owner: &mut CssomStore, base| {
        CssStyleSheet::new(
            owner,
            document.clone(),
            CssStyleSheetInit {
                base_url: Some(base),
                ..Default::default()
            },
        )
    };

    let exact = 2 * fixed + base_bytes;
    let mut owner = store(CssomLimits {
        max_string_bytes: exact,
        ..Default::default()
    });
    let first = construct(&mut owner, base.clone()).unwrap().into_parts().0;
    let second = construct(&mut owner, base.clone()).unwrap().into_parts().0;
    let snapshot = owner.snapshot();
    for sheet in [first.id(), second.id()] {
        let inputs = snapshot.sheet(sheet).unwrap().inputs();
        assert!(Arc::ptr_eq(inputs.base_url.as_ref().unwrap(), &base));
        assert_eq!(
            inputs.base_url.as_deref().unwrap().as_slice(),
            &[0xd800, 0, 0xdc00]
        );
        assert_eq!(inputs.location.as_deref(), Some(""));
    }

    let mut short = store(CssomLimits {
        max_string_bytes: exact - 1,
        ..Default::default()
    });
    construct(&mut short, base.clone()).unwrap();
    let before = short.snapshot();
    assert!(matches!(
        construct(&mut short, base.clone()),
        Err(CssomError::Limit { resource: "host and live string bytes", maximum }) if maximum == exact - 1
    ));
    assert_eq!(short.snapshot().revision(), before.revision());

    let mut distinct = store(CssomLimits {
        max_string_bytes: exact,
        ..Default::default()
    });
    construct(&mut distinct, base.clone()).unwrap();
    assert!(matches!(
        construct(&mut distinct, make_base()),
        Err(CssomError::Limit {
            resource: "host and live string bytes",
            ..
        })
    ));

    for maximum in [exact + fixed, exact + fixed - 1] {
        let mut owner = store(CssomLimits {
            max_string_bytes: maximum,
            ..Default::default()
        });
        let first = construct(&mut owner, base.clone()).unwrap().into_parts().0;
        construct(&mut owner, base.clone()).unwrap();
        let before = owner.snapshot();
        let job = owner.begin_replace_sheet(guard(&owner), first.id(), "");
        if maximum == exact + fixed {
            let completion = owner
                .finish_replace_sheet(guard(&owner), job.unwrap().parse())
                .unwrap();
            assert!(matches!(
                completion.outcome(),
                CssomReplaceOutcome::Replaced
            ));
            assert!(Arc::ptr_eq(
                owner
                    .snapshot()
                    .sheet(first.id())
                    .unwrap()
                    .inputs()
                    .base_url
                    .as_ref()
                    .unwrap(),
                &base,
            ));
        } else {
            assert!(matches!(
                job,
                Err(CssomError::Limit {
                    resource: "host and live string bytes",
                    ..
                })
            ));
            assert_eq!(owner.snapshot().revision(), before.revision());
        }
        assert!(
            owner
                .pending_sheet_replacement(first.id())
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn sheet_constructor_keeps_exact_media_input_and_copies_a_list_through_cssom_text() {
    use std::sync::Arc;

    let mut units = "screen and\r\n, print /*😀"
        .encode_utf16()
        .collect::<Vec<_>>();
    units.extend([0, 0xd800]);
    units.extend("*/".encode_utf16());
    let original = Arc::new(units);
    let mut owner = store(CssomLimits::default());
    let outcome = CssStyleSheet::new(
        &mut owner,
        document(),
        CssStyleSheetInit {
            media: CssStyleSheetMedia::Text(CssomSheetText::Utf16(original.clone())),
            ..Default::default()
        },
    )
    .unwrap();
    let live = outcome.value().media(&owner).unwrap();
    let snapshot = owner.snapshot();
    let source = snapshot
        .media(live.id())
        .unwrap()
        .parse_source()
        .unwrap()
        .source();
    assert!(Arc::ptr_eq(source.original_utf16().unwrap(), &original));
    assert_eq!(
        source.as_str(),
        "screen and\n, print /*😀\u{fffd}\u{fffd}*/"
    );
    assert_eq!(source.original_utf16_range(10..11), Some(10..12));
    let scalar = source.as_str().find('😀').unwrap();
    let units = original.iter().position(|unit| *unit == 0xd83d).unwrap();
    assert_eq!(
        source.original_utf16_range(scalar..scalar + 4),
        Some(units..units + 2)
    );
    assert_eq!(
        source.original_utf16_range(scalar + 4..scalar + 7),
        Some(units + 2..units + 3)
    );
    assert_eq!(
        source.original_utf16_range(scalar + 7..scalar + 10),
        Some(units + 3..units + 4)
    );
    assert!(!outcome.diagnostics().is_empty());
    assert!(outcome.diagnostics().iter().all(|diagnostic| {
        diagnostic
            .source()
            .is_some_and(|actual| actual.same_snapshot(source))
    }));
    let original = live.capture(&snapshot).unwrap();
    assert_eq!(
        original
            .media_text(CssSerializationLimits::default())
            .unwrap(),
        "not all, print"
    );

    let copied = CssStyleSheet::new(
        &mut owner,
        document(),
        CssStyleSheetInit {
            media: CssStyleSheetMedia::List(&original),
            ..Default::default()
        },
    )
    .unwrap()
    .into_parts()
    .0
    .media(&owner)
    .unwrap();
    assert_ne!(live.id(), copied.id());
    let after = owner.snapshot();
    let copied_source = after
        .media(copied.id())
        .unwrap()
        .parse_source()
        .unwrap()
        .source();
    assert_eq!(copied_source.as_str(), "not all, print");
    assert!(copied_source.original_utf16().is_none());
    assert!(!copied_source.same_snapshot(source));
    live.set_media_text(&mut owner, "speech").unwrap();
    assert_eq!(
        copied
            .media_text(&owner, CssSerializationLimits::default())
            .unwrap(),
        "not all, print"
    );
}

#[test]
fn sheet_insert_rule_retains_utf16_rule_and_diagnostic_sources_and_guard_order() {
    use std::sync::Arc;
    use surgeist_css::source::CssUtf16Resource;

    let mut original = "/*😀\r\n*/ a { color: red; invalid: ; width: 2px; --raw: \""
        .encode_utf16()
        .collect::<Vec<_>>();
    original.extend([0, 0xdc00]);
    original.extend("\"; }".encode_utf16());
    let original = Arc::new(original);
    let mut limits = CssomLimits::default();
    limits.sheet_utf16_input.max_input_units = original.len();

    let mut owner = store(limits.clone());
    let sheet = CssStyleSheet::new(&mut owner, document(), Default::default())
        .unwrap()
        .into_parts()
        .0;
    let before = owner.snapshot();
    let outcome = sheet
        .insert_rule(&mut owner, CssomSheetText::Utf16(original.clone()), None)
        .unwrap();
    assert_eq!(*outcome.value(), 0);
    let after = owner.snapshot();
    assert!(before.sheet_rules(sheet.id()).unwrap().is_empty());
    let native = after
        .rule(&after.sheet_rules(sheet.id()).unwrap()[0])
        .unwrap();
    let insertion = native.insertion_origin().unwrap();
    let insertion_source = insertion.source();
    assert!(Arc::ptr_eq(
        insertion_source.original_utf16().unwrap(),
        &original
    ));
    assert_eq!(
        insertion.span().start().byte_offset().value() as usize,
        insertion_source.as_str().find("a {").unwrap(),
    );
    let block = native.data().block().unwrap();
    let CssomBlockData::Properties {
        selected: CssomProjection::Available(selected),
        ..
    } = after.block(block).unwrap().data()
    else {
        panic!("native stylesheet declaration projection");
    };
    let source = selected.entries()[0]
        .source()
        .parsed_value()
        .unwrap()
        .source();
    assert!(source.same_snapshot(insertion_source));
    assert_eq!(
        source.as_str(),
        "/*😀\n*/ a { color: red; invalid: ; width: 2px; --raw: \"\u{fffd}\u{fffd}\"; }"
    );
    assert_eq!(source.original_utf16_range(2..6), Some(2..4));
    assert_eq!(source.original_utf16_range(6..7), Some(4..6));
    assert!(!outcome.diagnostics().is_empty());
    assert!(outcome.diagnostics().iter().all(|diagnostic| {
        diagnostic
            .source()
            .is_some_and(|actual| actual.same_snapshot(source))
    }));
    let declaration_start = source.as_str().find("invalid").unwrap();
    let authored_start = original
        .iter()
        .position(|unit| *unit == u16::from(b'i'))
        .unwrap();
    assert_eq!(
        source.original_utf16_range(declaration_start..declaration_start + "invalid".len()),
        Some(authored_start..authored_start + "invalid".len()),
    );

    for (raw, exception) in [
        ("!", CssomException::Syntax),
        ("@import 'blocked.css';", CssomException::Syntax),
        ("a {}", CssomException::IndexSize),
    ] {
        let raw = CssomSheetText::Utf16(Arc::new(raw.encode_utf16().collect()));
        assert!(matches!(
            sheet.insert_rule(&mut owner, raw, Some(usize::MAX)),
            Err(CssomError::Source(actual)) if actual == exception
        ));
        assert_eq!(owner.snapshot().revision(), after.revision());
    }

    limits.sheet_utf16_input.max_input_units -= 1;
    let mut short = store(limits);
    let sheet = construct(&mut short);
    let before = short.snapshot();
    let mut edit = batch(&short);
    assert!(matches!(
        edit.insert_sheet_rule(&sheet, CssomSheetText::Utf16(original.clone()), 0),
        Err(CssomError::Utf16Input(error)) if error.exhausted_limit() == Some(CssUtf16Resource::InputUnits)
    ));
    assert!(matches!(short.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(short.snapshot().revision(), before.revision());

    for (origin_clean, locked, expected) in [
        (false, true, CssomException::Security),
        (true, true, CssomException::NotAllowed),
    ] {
        let mut edit = batch(&short);
        let mut inputs =
            CssomSheetInputs::external(version(CssomInputRole::Document, "guard", 1), origin_clean);
        inputs.disallow_modification = locked;
        let ticket = edit.create_typed_sheet(CssSheet::new(), inputs).unwrap();
        let id = short.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
        let sheet = short.css_style_sheet(&id).unwrap();
        let before = short.snapshot();
        assert!(matches!(
            sheet.insert_rule(&mut short, CssomSheetText::Utf16(original.clone()), Some(usize::MAX)),
            Err(CssomError::Source(actual)) if actual == expected
        ));
        assert_eq!(short.snapshot().revision(), before.revision());
    }
}

#[test]
fn utf16_rule_reaches_the_exact_canonical_source_backing_limit() {
    use std::sync::Arc;
    use surgeist_css::rule::CssRuleSyntax;
    use surgeist_css::source::CssUtf16InputLimits;

    let mut units = Vec::with_capacity(64);
    units.extend("a {} /*".encode_utf16());
    units.push(0xd800);
    units.extend("*/".encode_utf16());
    let original = Arc::new(units);
    let parse = CssRuleSyntax::classify_utf16(
        &CssParser::default(),
        original.clone(),
        CssUtf16InputLimits::default(),
        || Ok::<(), std::convert::Infallible>(()),
    )
    .unwrap();
    let source = parse.syntax().as_ref().unwrap().origin().source();
    let exact = source.storage_usage().retained_bytes();
    assert!(exact > original.capacity() * size_of::<u16>());

    for maximum in [exact, exact - 1] {
        let mut owner = store(CssomLimits {
            max_retained_source_bytes: maximum,
            ..Default::default()
        });
        // A native typed sheet starts without a parse source or a parsed
        // MediaList, so the inserted occurrence is its entire source backing.
        let mut edit = batch(&owner);
        let ticket = edit
            .create_typed_sheet(
                CssSheet::new(),
                CssomSheetInputs::constructed(
                    version(CssomInputRole::Document, "d", 1),
                    "d".into(),
                    None,
                ),
            )
            .unwrap();
        let id = owner.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
        let sheet = owner.css_style_sheet(&id).unwrap();
        let before = owner.snapshot();
        let result = sheet.insert_rule(&mut owner, CssomSheetText::Utf16(original.clone()), None);
        if maximum == exact {
            assert_eq!(*result.unwrap().value(), 0);
            assert_eq!(owner.snapshot().sheet_rules(&id).unwrap().len(), 1);
        } else {
            assert!(
                matches!(
                    result,
                    Err(CssomError::Limit { resource: "retained parse source bytes", maximum: actual }) if actual == maximum
                ),
                "{result:?}"
            );
            assert_eq!(owner.snapshot().revision(), before.revision());
            assert!(owner.snapshot().sheet_rules(&id).unwrap().is_empty());
        }
    }
}
