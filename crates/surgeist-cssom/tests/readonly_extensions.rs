#![forbid(unsafe_code)]
//! MQ5 WD 2026-02-19 §11: authored MediaList versus true/false, no evaluation.
//! Color5 WD 2026-09-08 §12.1: serialize associated names/descriptors;
//! absent descriptors return empty, independently of unavailable whole formats.
use surgeist_css::*;
use surgeist_css::{CssColorProfileRule, CssCustomMediaRule, CssRule};
use surgeist_cssom::*;

fn limits() -> CssSerializationLimits {
    CssSerializationLimits::default()
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn new_store() -> CssomStore {
    CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap()
}
fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "supplied".into(),
            revision: 1,
        },
        "supplied".into(),
        None,
    )
}
fn create(store: &mut CssomStore, source: &str) -> CssomSheetId {
    let mut edit = batch(store);
    let ticket = edit.create_parsed_sheet(source, inputs()).unwrap();
    store.commit(edit).unwrap().sheet(&ticket).unwrap().clone()
}
fn rules(snapshot: &CssomSnapshot, sheet: &CssomSheetId) -> Vec<CssomRuleId> {
    snapshot
        .rules(snapshot.sheet(sheet).unwrap().rules())
        .unwrap()
        .to_vec()
}
fn custom(
    store: &CssomStore,
    snapshot: &CssomSnapshot,
    rule: &CssomRuleId,
) -> Result<CssomCapture<surgeist_cssom::CssCustomMediaRule>, CssomError> {
    store.css_custom_media_rule(rule)?.capture(snapshot)
}
fn profile_capture(
    store: &CssomStore,
    snapshot: &CssomSnapshot,
    rule: &CssomRuleId,
) -> Result<CssomCapture<surgeist_cssom::CssColorProfileRule>, CssomError> {
    store.css_color_profile_rule(rule)?.capture(snapshot)
}
fn list_capture(
    store: &CssomStore,
    snapshot: &CssomSnapshot,
    id: &CssomMediaListId,
) -> Result<CssomCapture<MediaList>, CssomError> {
    store.media_list(id)?.capture(snapshot)
}
fn media(store: &CssomStore, snapshot: &CssomSnapshot, rule: &CssomRuleId) -> CssomMediaListId {
    match custom(store, snapshot, rule).unwrap().query().unwrap() {
        CapturedCustomMediaQuery::MediaList(value) => value.handle().id().clone(),
        other => panic!("MediaList branch: {other:?}"),
    }
}

#[test]
fn authored_boolean_branches_and_decoded_names_do_not_evaluate_or_default() {
    let mut store = new_store();
    let sheet = create(
        &mut store,
        r"@custom-media --A\ B true; @custom-media --Never false; @custom-media --Unknown screen; @custom-media --Empty;",
    );
    let snapshot = store.snapshot();
    let rules = rules(&snapshot, &sheet);
    assert_eq!(rules.len(), 4); // An authored empty query list is not a boolean fallback.
    assert_eq!(
        custom(&store, &snapshot, &rules[0])
            .and_then(|value| value.name().map(str::to_owned))
            .unwrap(),
        "--A B"
    );
    assert!(matches!(
        custom(&store, &snapshot, &rules[0])
            .and_then(|value| value.query())
            .unwrap(),
        CapturedCustomMediaQuery::Boolean(true)
    ));
    assert!(matches!(
        custom(&store, &snapshot, &rules[1])
            .and_then(|value| value.query())
            .unwrap(),
        CapturedCustomMediaQuery::Boolean(false)
    ));
    let id = media(&store, &snapshot, &rules[2]);
    assert_eq!(
        list_capture(&store, &snapshot, &id)
            .unwrap()
            .media_text(limits())
            .unwrap(),
        "screen"
    );
    assert_eq!(id.owner(), snapshot.owner());
    let empty = media(&store, &snapshot, &rules[3]);
    assert_ne!(id, empty);
    assert_eq!(
        list_capture(&store, &snapshot, &empty)
            .unwrap()
            .length()
            .unwrap(),
        0
    );
    assert_eq!(
        list_capture(&store, &snapshot, &empty)
            .unwrap()
            .media_text(limits())
            .unwrap(),
        ""
    );

    let mut edit = batch(&store);
    edit.set_rule_css_text(&rules[0], "@custom-media --Changed false;")
        .unwrap();
    assert!(matches!(
        store.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(
        store
            .css_custom_media_rule(&rules[0])
            .unwrap()
            .name(&store)
            .unwrap(),
        "--A B"
    );
}

#[test]
fn query_collection_keeps_identity_current_recovery_and_historical_provenance() {
    let mut store = new_store();
    let source = String::from("/*😀*/ @custom-media --Alias print;");
    let sheet = create(&mut store, &source);
    let before = store.snapshot();
    let rule = rules(&before, &sheet)[0].clone();
    let id = media(&store, &before, &rule);
    let mut edit = batch(&store);
    assert!(
        edit.set_media_text(&id, "screen,,speech")
            .unwrap()
            .changed()
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    let current_list = list_capture(&store, &after, &id).unwrap();
    assert_eq!(media(&store, &after, &rule), id);
    assert_eq!(
        current_list.media_text(limits()).unwrap(),
        "screen, not all, speech"
    );
    assert_eq!(
        list_capture(&store, &before, &id)
            .unwrap()
            .media_text(limits())
            .unwrap(),
        "print"
    );
    let CssValueOrigin::Parsed(current_origin) = after.media(&id).unwrap().queries()[0].origin()
    else {
        panic!("current genuine media input")
    };
    assert_eq!(current_origin.source().as_str(), "screen,,speech");
    let CssomAuthoredRule::Ordinary(native) = after.rule(&rule).unwrap().authored() else {
        panic!("ordinary")
    };
    assert_eq!(
        native
            .serialize_cssom(Default::default())
            .unwrap_err()
            .unavailable_format(),
        Some(CssRuleCssomFormat::CustomMedia)
    );

    let CssomAuthoredRule::Ordinary(CssRule::CustomMedia(original)) =
        after.rule(&rule).unwrap().authored()
    else {
        panic!("genuine original")
    };
    let CssCustomMediaBody::Media(original_query) = original.body() else {
        panic!("authored original list")
    };
    assert_eq!(
        original_query
            .serialize_cssom(Default::default())
            .unwrap()
            .as_css(),
        "print"
    );
    let CssValueOrigin::Parsed(origin) = original.name().origin() else {
        panic!("original source occurrence")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("--Alias").unwrap()
    );
    let mut edit = batch(&store);
    edit.delete_sheet_rule(&sheet, 0).unwrap();
    store.commit(edit).unwrap();
    let detached = store.snapshot();
    assert_eq!(detached.parent_style_sheet(&rule).unwrap(), None);
    assert_eq!(media(&store, &detached, &rule), id);
    let mut edit = batch(&store);
    assert!(edit.set_media_text(&id, "").unwrap().changed());
    store.commit(edit).unwrap();
    let empty = store.snapshot();
    assert_eq!(media(&store, &empty, &rule), id);
    assert_eq!(
        list_capture(&store, &empty, &id).unwrap().length().unwrap(),
        0
    );
    assert_eq!(
        list_capture(&store, &empty, &id)
            .unwrap()
            .item(0, CssSerializationLimits::new(0, 0, 0))
            .unwrap(),
        None
    );
    let historical_custom = custom(&store, &before, &rule).unwrap();
    let empty_custom = custom(&store, &empty, &rule).unwrap();
    drop(source);
    drop(store);
    assert_eq!(
        current_list.media_text(limits()).unwrap(),
        "screen, not all, speech"
    );
    assert_eq!(historical_custom.name().unwrap(), "--Alias");
    assert!(matches!(
        empty_custom.query().unwrap(),
        CapturedCustomMediaQuery::MediaList(value) if value.handle().id() == &id
    ));
}

#[test]
fn profile_getters_serialize_effective_values_and_distinguish_absence_from_empty_url() {
    let mut store = new_store();
    let sheet = create(
        &mut store,
        r"@color-profile --A\ B {src:url(first);src:url(); rendering-intent: PERCEPTUAL; rendering-intent: SATURATION; components:A, a\+b, 日本, A; src:broken;} @color-profile DEVICE-CMYK {} @color-profile --Pending {components:env(channels);}",
    );
    let snapshot = store.snapshot();
    let ids = rules(&snapshot, &sheet);
    assert_eq!(ids.len(), 3);
    assert_eq!(
        profile_capture(&store, &snapshot, &ids[0])
            .and_then(|value| value.name(limits()))
            .unwrap(),
        r"--A\ B"
    );
    assert_eq!(
        profile_capture(&store, &snapshot, &ids[0])
            .and_then(|value| value.src(limits()))
            .unwrap(),
        "url(\"\")"
    );
    assert_eq!(
        profile_capture(&store, &snapshot, &ids[0])
            .and_then(|value| value.rendering_intent(limits()))
            .unwrap(),
        "saturation"
    );
    assert_eq!(
        profile_capture(&store, &snapshot, &ids[0])
            .and_then(|value| value.components(limits()))
            .unwrap(),
        r"A, a\+b, 日本, A"
    );
    assert_eq!(
        profile_capture(&store, &snapshot, &ids[1])
            .and_then(|value| value.name(limits()))
            .unwrap(),
        "device-cmyk"
    );
    for getter in [
        CssomCapture::<surgeist_cssom::CssColorProfileRule>::src,
        CssomCapture::<surgeist_cssom::CssColorProfileRule>::rendering_intent,
        CssomCapture::<surgeist_cssom::CssColorProfileRule>::components,
    ] {
        assert_eq!(
            getter(
                &profile_capture(&store, &snapshot, &ids[1]).unwrap(),
                CssSerializationLimits::new(0, 0, 0)
            )
            .unwrap(),
            ""
        );
    }
    assert_eq!(
        profile_capture(&store, &snapshot, &ids[2])
            .and_then(|value| value.components(limits()))
            .unwrap(),
        "env(channels)"
    );
    let CssomAuthoredRule::Ordinary(original) = snapshot.rule(&ids[0]).unwrap().authored() else {
        panic!("ordinary source")
    };
    assert_eq!(
        original
            .serialize_cssom(Default::default())
            .unwrap_err()
            .unavailable_format(),
        Some(CssRuleCssomFormat::ColorProfile)
    );
}

#[test]
fn profile_replacement_keeps_detached_old_objects_and_original_value_coordinates() {
    let mut store = new_store();
    let source = String::from("/*😀*/\n@color-profile --Old {src:url(relative.icc);}");
    let sheet = create(&mut store, &source);
    let before = store.snapshot();
    let old = rules(&before, &sheet)[0].clone();
    let mut edit = batch(&store);
    edit.replace_sheet_sync(
        &sheet,
        "@color-profile --New {rendering-intent:absolute-colorimetric;}",
    )
    .unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    let new = rules(&after, &sheet)[0].clone();
    assert_ne!(old, new);
    assert_eq!(after.parent_style_sheet(&old).unwrap(), None);
    assert_eq!(
        profile_capture(&store, &after, &new)
            .and_then(|value| value.name(limits()))
            .unwrap(),
        "--New"
    );
    assert_eq!(
        profile_capture(&store, &after, &new)
            .and_then(|value| value.src(limits()))
            .unwrap(),
        ""
    );
    assert_eq!(
        profile_capture(&store, &after, &old)
            .and_then(|value| value.src(limits()))
            .unwrap(),
        "url(\"relative.icc\")"
    );
    let CssomRuleData::Leaf {
        current: CssomAuthoredRule::Ordinary(CssRule::ColorProfile(profile)),
    } = after.rule(&old).unwrap().data()
    else {
        panic!("checked profile")
    };
    let value = profile
        .effective(CssColorProfileDescriptorKind::Src)
        .unwrap();
    let CssValueOrigin::Parsed(origin) = value.components().items()[0].origin() else {
        panic!("original value")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("url(").unwrap()
    );
    let historical_profile = profile_capture(&store, &before, &old).unwrap();
    let current_profile = profile_capture(&store, &after, &new).unwrap();
    drop(source);
    drop(store);
    assert_eq!(historical_profile.name(limits()).unwrap(), "--Old");
    assert_eq!(
        current_profile.rendering_intent(limits()).unwrap(),
        "absolute-colorimetric"
    );
}

#[test]
fn scoped_and_programmatic_checked_payloads_use_the_same_facets() {
    let mut store = new_store();
    let sheet = create(
        &mut store,
        "@scope { @custom-media --Inner print; @color-profile --Scoped {components:A;} }",
    );
    let snapshot = store.snapshot();
    let scope = rules(&snapshot, &sheet)[0].clone();
    let children = snapshot
        .rules(snapshot.rule(&scope).unwrap().data().children().unwrap())
        .unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(
        custom(&store, &snapshot, &children[0])
            .and_then(|value| value.name().map(str::to_owned))
            .unwrap(),
        "--Inner"
    );
    assert_eq!(
        list_capture(&store, &snapshot, &media(&store, &snapshot, &children[0]))
            .unwrap()
            .media_text(limits())
            .unwrap(),
        "print"
    );
    assert_eq!(
        profile_capture(&store, &snapshot, &children[1])
            .and_then(|value| value.components(limits()))
            .unwrap(),
        "A"
    );
    let typed = CssSheet::try_from_rules(vec![
        CssRule::CustomMedia(
            CssCustomMediaRule::try_new(
                CssCustomMediaName::try_new("--Made").unwrap(),
                CssCustomMediaBody::True,
                Default::default(),
            )
            .unwrap(),
        ),
        CssRule::ColorProfile(CssColorProfileRule::new(
            CssColorProfileRuleName::Custom(
                CssColorProfileName::try_new("--Made profile").unwrap(),
            ),
            vec![],
        )),
    ])
    .unwrap();
    let mut edit = batch(&store);
    let ticket = edit.create_typed_sheet(typed, inputs()).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    let ids = rules(&snapshot, &sheet);
    assert!(matches!(
        custom(&store, &snapshot, &ids[0])
            .and_then(|value| value.query())
            .unwrap(),
        CapturedCustomMediaQuery::Boolean(true)
    ));
    assert_eq!(
        profile_capture(&store, &snapshot, &ids[1])
            .and_then(|value| value.name(limits()))
            .unwrap(),
        r"--Made\ profile"
    );
    let CssomRuleData::CustomMedia { name, .. } = snapshot.rule(&ids[0]).unwrap().data() else {
        panic!("live model")
    };
    assert_eq!(name.origin(), &CssValueOrigin::Programmatic);
}

#[test]
fn owner_kind_missing_capture_and_output_limits_return_typed_errors_without_mutation() {
    let mut store = new_store();
    let before_creation = store.snapshot();
    let sheet = create(
        &mut store,
        r"@custom-media --m screen; @color-profile --A\ B {components:A, a\+b;} a{}",
    );
    let snapshot = store.snapshot();
    let ids = rules(&snapshot, &sheet);
    assert!(matches!(
        custom(&store, &snapshot, &ids[2]).and_then(|value| value.name().map(str::to_owned)),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(
        custom(&store, &snapshot, &ids[1]).and_then(|value| value.query()),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(
        profile_capture(&store, &snapshot, &ids[0]).and_then(|value| value.src(limits())),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(
        custom(&store, &before_creation, &ids[0]).and_then(|value| value.query()),
        Err(CssomError::MissingObject)
    ));
    let foreign = new_store();
    assert!(matches!(
        custom(&store, &foreign.snapshot(), &ids[0])
            .and_then(|value| value.name().map(str::to_owned)),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(
        profile_capture(&store, &foreign.snapshot(), &ids[1])
            .and_then(|value| value.name(limits())),
        Err(CssomError::ForeignOwner)
    ));
    let expected = r"A, a\+b";
    let exact = CssSerializationLimits::new(2, 2, expected.len());
    assert_eq!(
        profile_capture(&store, &snapshot, &ids[1])
            .and_then(|value| value.components(exact))
            .unwrap(),
        expected
    );
    type Case = (CssSerializationLimits, fn(&CssSerializationError) -> bool);
    let cases: [Case; 3] = [
        (
            CssSerializationLimits::new(1, 2, 99),
            CssSerializationError::is_input_node_limit,
        ),
        (
            CssSerializationLimits::new(2, 1, 99),
            CssSerializationError::is_projection_node_limit,
        ),
        (
            CssSerializationLimits::new(2, 2, expected.len() - 1),
            CssSerializationError::is_byte_limit,
        ),
    ];
    for (policy, kind) in cases {
        assert!(
            matches!(profile_capture(&store, &snapshot, &ids[1]).and_then(|value| value.components(policy)), Err(CssomError::Value(error)) if kind(&error))
        );
    }
    assert!(
        matches!(profile_capture(&store, &snapshot, &ids[1]).and_then(|value| value.name(CssSerializationLimits::new(0,1,99))), Err(CssomError::Value(error)) if error.is_input_node_limit())
    );
    let media = media(&store, &snapshot, &ids[0]);
    assert!(matches!(
        list_capture(&store, &snapshot, &media)
            .unwrap()
            .media_text(CssSerializationLimits::new(99, 99, 0)),
        Err(CssomError::Media(_))
    ));
    assert_eq!(
        profile_capture(&store, &snapshot, &ids[1])
            .and_then(|value| value.components(exact))
            .unwrap(),
        expected
    );
    assert_eq!(snapshot.revision(), store.snapshot().revision());
}
#[test]
fn query_identity_allocation_obeys_store_limits_and_failed_adoption_publishes_nothing() {
    // Store root, sheet, rule list, sheet media and one boolean rule are five objects.
    // A query-list rule additionally requires its genuine associated MediaList.
    let policy = CssomLimits {
        max_objects: 5,
        ..CssomLimits::default()
    };
    let mut owner = CssomStore::new(policy.clone(), CssomContext::default()).unwrap();
    let before = owner.snapshot();
    let mut edit = batch(&owner);
    assert!(matches!(
        edit.create_parsed_sheet("@custom-media --m screen;", inputs()),
        Err(CssomError::Limit {
            resource: "objects",
            maximum: 5
        })
    ));
    assert!(matches!(owner.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(owner.snapshot().revision(), before.revision());
    assert!(owner.snapshot().sheets().is_empty());
    let sheet = create(&mut owner, "@custom-media --m false;");
    let snapshot = owner.snapshot();
    assert!(matches!(
        custom(&owner, &snapshot, &rules(&snapshot, &sheet)[0])
            .unwrap()
            .query()
            .unwrap(),
        CapturedCustomMediaQuery::Boolean(false)
    ));
    let mut owner = CssomStore::new(
        CssomLimits {
            max_objects: 6,
            ..policy
        },
        CssomContext::default(),
    )
    .unwrap();
    let sheet = create(&mut owner, "@custom-media --m screen;");
    let snapshot = owner.snapshot();
    assert_eq!(
        list_capture(
            &owner,
            &snapshot,
            &media(&owner, &snapshot, &rules(&snapshot, &sheet)[0])
        )
        .unwrap()
        .media_text(limits())
        .unwrap(),
        "screen"
    );
}
