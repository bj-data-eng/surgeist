//! Concrete extension operations preserve their native owner contracts.
use surgeist_css::{CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits};
use surgeist_cssom::*;

fn create(source: &str, limits: CssomLimits) -> (CssomStore, CssomSheetId, Vec<CssomRuleId>) {
    let mut store = CssomStore::new(limits, Default::default()).unwrap();
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    let ticket = batch
        .create_parsed_sheet(
            source,
            CssomSheetInputs::external(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "extensions".into(),
                    revision: 1,
                },
                true,
            ),
        )
        .unwrap();
    let sheet = store.commit(batch).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    let rules = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap()
        .to_vec();
    (store, sheet, rules)
}
fn changed<T>(outcome: &CssomOperationOutcome<T>) -> bool {
    matches!(outcome.publication(), CssomPublication::Changed(_))
}

#[test]
fn seven_maps_are_distinct_same_identity_children_with_scalar_sequence_and_cardinality() {
    let (mut store, _, ids) = create("@font-feature-values A {}", Default::default());
    let rule = store.css_font_feature_values_rule(&ids[0]).unwrap();
    let maps = [
        rule.annotation(&store).unwrap(),
        rule.ornaments(&store).unwrap(),
        rule.stylistic(&store).unwrap(),
        rule.swash(&store).unwrap(),
        rule.character_variant(&store).unwrap(),
        rule.styleset(&store).unwrap(),
        rule.historical_forms(&store).unwrap(),
    ];
    for (index, map) in maps.iter().enumerate() {
        for other in &maps[index + 1..] {
            assert_ne!(map, other);
        }
        assert_eq!(map.id().owner(), store.snapshot().owner());
        assert_eq!(map.size(&store).unwrap(), 0);
        let outcome = map
            .set(&mut store, "", CssomFeatureValues::Scalar(u32::MAX))
            .unwrap();
        assert_eq!(outcome.value(), &());
        assert!(changed(&outcome));
        assert_eq!(map.get(&store, "").unwrap(), Some(vec![u32::MAX]));
        let allowed = match index {
            4 => 2,
            5 | 6 => 3,
            _ => 1,
        };
        let _outcome = map
            .set(
                &mut store,
                "punctuation key!",
                CssomFeatureValues::Sequence(vec![0; allowed]),
            )
            .unwrap();
        let revision = store.snapshot().revision().clone();
        if index < 5 {
            assert!(matches!(
                map.set(
                    &mut store,
                    "punctuation key!",
                    CssomFeatureValues::Sequence(vec![1; allowed + 1])
                ),
                Err(CssomError::Source(CssomException::InvalidAccess))
            ));
            assert_eq!(store.snapshot().revision(), &revision);
        }
        let _outcome = map
            .set(&mut store, "empty", CssomFeatureValues::Sequence(vec![]))
            .unwrap();
        assert_eq!(map.get(&store, "empty").unwrap(), Some(vec![]));
        assert_eq!(map.get(&store, "absent").unwrap(), None);
        assert!(map.has(&store, "empty").unwrap());
    }
    assert_eq!(rule.annotation(&store).unwrap(), maps[0]);
    assert_eq!(rule.historical_forms(&store).unwrap(), maps[6]);
}

#[test]
fn map_iteration_is_ordered_snapshot_projection_with_honest_conversion_errors() {
    let (mut store, _, ids) = create(
        "@font-feature-values A { @styleset { wide:4294967296; good:2; } }",
        Default::default(),
    );
    let rule = store.css_font_feature_values_rule(&ids[0]).unwrap();
    let map = rule.styleset(&store).unwrap();
    let captured = rule.capture(&store.snapshot()).unwrap().styleset().unwrap();
    assert_eq!(captured.handle(), &map);
    let mut entries = map.entries(&store).unwrap();
    let keys = map.keys(&store).unwrap();
    let mut values = map.values(&store).unwrap();
    assert!(matches!(
        map.get(&store, "wide"),
        Err(CssomError::AuthoredFeatureConversion)
    ));
    assert!(matches!(
        entries.next().unwrap(),
        Err(CssomError::AuthoredFeatureConversion)
    ));
    assert!(matches!(
        values.next().unwrap(),
        Err(CssomError::AuthoredFeatureConversion)
    ));
    let _outcome = map
        .set(&mut store, "wide", CssomFeatureValues::Scalar(9))
        .unwrap();
    let _outcome = map
        .set(&mut store, "later", CssomFeatureValues::Sequence(vec![]))
        .unwrap();
    assert_eq!(keys.collect::<Vec<_>>(), ["wide", "good"]);
    assert_eq!(entries.next().unwrap().unwrap(), ("good".into(), vec![2]));
    assert_eq!(values.next().unwrap().unwrap(), [2]);
    assert!(entries.next().is_none());
    assert!(matches!(
        captured.get("wide"),
        Err(CssomError::AuthoredFeatureConversion)
    ));
    assert_eq!(
        map.keys(&store).unwrap().collect::<Vec<_>>(),
        ["wide", "good", "later"]
    );
    assert!(*map.delete(&mut store, "wide").unwrap().value());
    assert!(!*map.delete(&mut store, "wide").unwrap().value());
    let _outcome = map
        .set(&mut store, "wide", CssomFeatureValues::Scalar(5))
        .unwrap();
    assert_eq!(
        map.keys(&store).unwrap().collect::<Vec<_>>(),
        ["good", "later", "wide"]
    );
    let snapshot = store.snapshot();
    assert_eq!(snapshot.feature_map(map.id()).unwrap().authored().len(), 2);
    let _outcome = map.clear(&mut store).unwrap();
    assert_eq!(map.size(&store).unwrap(), 0);
    assert!(!changed(&map.clear(&mut store).unwrap()));
    drop(store);
    assert_eq!(captured.size().unwrap(), 2);
    assert_eq!(
        captured
            .snapshot()
            .feature_map(map.id())
            .unwrap()
            .authored()
            .len(),
        2
    );
}

#[test]
fn family_setter_has_complete_native_grammar_diagnostics_provenance_and_noops() {
    let (mut store, _, ids) = create("@font-feature-values A {}", Default::default());
    let rule = store.css_font_feature_values_rule(&ids[0]).unwrap();
    let old = rule.capture(&store.snapshot()).unwrap();
    for invalid in ["", "serif", "A,", "A;B", "A{}", "A !important", "]"] {
        let result = rule.set_font_family(&mut store, invalid).unwrap();
        assert!(!changed(&result), "{invalid}");
        assert_eq!(rule.font_family(&store, Default::default()).unwrap(), "A");
    }
    let result = rule.set_font_family(&mut store, "'Recovered").unwrap();
    assert!(changed(&result));
    let snapshot = store.snapshot();
    let input = snapshot
        .inspect_font_feature_values(rule.id())
        .unwrap()
        .family_input()
        .unwrap();
    assert_eq!(result.diagnostics(), input.diagnostics());
    assert!(!result.diagnostics().is_empty());
    let _result = rule.set_font_family(&mut store, "'serif', A\\ B").unwrap();
    assert_eq!(
        rule.font_family(&store, Default::default()).unwrap(),
        "\"serif\", A B"
    );
    let snapshot = store.snapshot();
    let view = snapshot.inspect_font_feature_values(rule.id()).unwrap();
    let input = view.family_input().unwrap();
    assert_eq!(
        input.value().family_origin(1).unwrap().source().as_str(),
        "'serif', A\\ B"
    );
    assert_eq!(view.families().len(), 2);
    drop(store);
    assert_eq!(old.font_family(Default::default()).unwrap(), "A");
}

#[test]
fn family_output_and_input_budgets_remain_cumulative_and_failure_does_not_publish() {
    let (mut store, _, ids) = create(
        "@font-feature-values A {}",
        CssomLimits {
            max_input_bytes: 50,
            ..Default::default()
        },
    );
    let rule = store.css_font_feature_values_rule(&ids[0]).unwrap();
    let _result = rule.set_font_family(&mut store, "A, B").unwrap();
    assert_eq!(
        rule.font_family(&store, CssSpecifiedValueSerializationLimits::new(2, 2, 4))
            .unwrap(),
        "A, B"
    );
    for limits in [
        CssSpecifiedValueSerializationLimits::new(1, 2, 4),
        CssSpecifiedValueSerializationLimits::new(2, 1, 4),
        CssSpecifiedValueSerializationLimits::new(2, 2, 3),
    ] {
        assert!(matches!(
            rule.font_family(&store, limits),
            Err(CssomError::Value(_))
        ));
    }
    let revision = store.snapshot().revision().clone();
    assert!(matches!(
        rule.set_font_family(&mut store, &format!("{}C", " ".repeat(50))),
        Err(CssomError::ParseResource(_))
    ));
    assert_eq!(store.snapshot().revision(), &revision);
    assert_eq!(
        rule.font_family(&store, Default::default()).unwrap(),
        "A, B"
    );
}

#[test]
fn font_face_style_is_a_same_identity_descriptor_facet_without_rule_scalars() {
    let (store, _, ids) = create(
        "@font-face {font-family:A;src:local(A)}",
        Default::default(),
    );
    let face = store.css_font_face_rule(&ids[0]).unwrap();
    let style = face.style(&store).unwrap();
    assert_eq!(face.style(&store).unwrap(), style);
    let captured = face.capture(&store.snapshot()).unwrap();
    assert_eq!(captured.style().unwrap().handle(), &style);
    assert_eq!(
        captured.style().unwrap().snapshot().revision(),
        captured.snapshot().revision()
    );
    assert_eq!(
        face.as_rule().r#type(&store).unwrap(),
        CssRule::FONT_FACE_RULE
    );
    drop(store);
    assert_eq!(captured.style().unwrap().handle(), &style);
}

#[test]
fn palette_and_profile_getters_remain_available_independently_of_wrapper_capability() {
    let (store, _, ids) = create(
        "@font-palette-values --P {font-family:A; base-palette:1; override-colors:0 red;} @font-palette-values --Empty {font-family:B;} @color-profile --C {src:url();rendering-intent:saturation;components:A,a\\+b;} @color-profile device-cmyk {}",
        Default::default(),
    );
    let palette = store.css_font_palette_values_rule(&ids[0]).unwrap();
    assert_eq!(palette.name(&store).unwrap(), "--P");
    assert_eq!(
        palette.font_family(&store, Default::default()).unwrap(),
        "A"
    );
    assert_eq!(
        palette.base_palette(&store, Default::default()).unwrap(),
        "1"
    );
    assert_eq!(
        palette.override_colors(&store, Default::default()).unwrap(),
        "0 red"
    );
    let empty = store
        .css_font_palette_values_rule(&ids[1])
        .unwrap()
        .capture(&store.snapshot())
        .unwrap();
    assert_eq!(empty.font_family(Default::default()).unwrap(), "B");
    assert_eq!(empty.base_palette(Default::default()).unwrap(), "");
    assert_eq!(empty.override_colors(Default::default()).unwrap(), "");
    let profile = store.css_color_profile_rule(&ids[2]).unwrap();
    assert_eq!(profile.name(&store, Default::default()).unwrap(), "--C");
    assert_eq!(
        profile.src(&store, Default::default()).unwrap(),
        "url(\"\")"
    );
    assert_eq!(
        profile
            .rendering_intent(&store, Default::default())
            .unwrap(),
        "saturation"
    );
    assert_eq!(
        profile.components(&store, Default::default()).unwrap(),
        "A, a\\+b"
    );
    assert!(matches!(
        profile.css_text(&store, &Default::default()),
        Err(CssomError::Format(_))
    ));
    let capture = profile.capture(&store.snapshot()).unwrap();
    let empty = store
        .css_color_profile_rule(&ids[3])
        .unwrap()
        .capture(&store.snapshot())
        .unwrap();
    for getter in [
        CssomCapture::<CssColorProfileRule>::src,
        CssomCapture::<CssColorProfileRule>::rendering_intent,
        CssomCapture::<CssColorProfileRule>::components,
    ] {
        assert_eq!(
            getter(&empty, CssSpecifiedValueSerializationLimits::new(0, 0, 0)).unwrap(),
            ""
        );
    }
    assert_eq!(empty.name(Default::default()).unwrap(), "device-cmyk");
    assert!(
        matches!(capture.components(CssSpecifiedValueSerializationLimits::new(2,2,6)), Err(CssomError::Value(error)) if error.kind()==CssSpecifiedValueSerializationErrorKind::ByteLimit)
    );
    drop(store);
    assert_eq!(capture.src(Default::default()).unwrap(), "url(\"\")");
}

#[test]
fn counter_all_ten_descriptor_pairs_reuse_native_prospective_definition() {
    let (mut store, _, ids) = create(
        "@counter-style c {system:fixed; symbols:'A';} @counter-style additive {system:additive; additive-symbols:1 'A';}",
        Default::default(),
    );
    let counter = store.css_counter_style_rule(&ids[0]).unwrap();
    let old = counter.capture(&store.snapshot()).unwrap();
    let _result = counter
        .set_system(&mut store, "fixed 4", Default::default())
        .unwrap();
    let _result = counter
        .set_symbols(&mut store, "'B'", Default::default())
        .unwrap();
    let _result = counter
        .set_negative(&mut store, "'-' '+'", Default::default())
        .unwrap();
    let _result = counter
        .set_prefix(&mut store, "'['", Default::default())
        .unwrap();
    let _result = counter
        .set_suffix(&mut store, "']'", Default::default())
        .unwrap();
    let _result = counter
        .set_range(&mut store, "1 9", Default::default())
        .unwrap();
    let _result = counter
        .set_pad(&mut store, "2 '0'", Default::default())
        .unwrap();
    let _result = counter
        .set_speak_as(&mut store, "words", Default::default())
        .unwrap();
    let _result = counter
        .set_fallback(&mut store, "decimal", Default::default())
        .unwrap();
    assert_eq!(
        counter.system(&store, Default::default()).unwrap(),
        "fixed 4"
    );
    assert_eq!(
        counter.symbols(&store, Default::default()).unwrap(),
        "\"B\""
    );
    assert_eq!(
        counter.negative(&store, Default::default()).unwrap(),
        "\"-\" \"+\""
    );
    assert_eq!(counter.prefix(&store, Default::default()).unwrap(), "\"[\"");
    assert_eq!(counter.suffix(&store, Default::default()).unwrap(), "\"]\"");
    assert_eq!(counter.range(&store, Default::default()).unwrap(), "1 9");
    assert_eq!(counter.pad(&store, Default::default()).unwrap(), "2 \"0\"");
    assert_eq!(
        counter.speak_as(&store, Default::default()).unwrap(),
        "words"
    );
    assert_eq!(
        counter.fallback(&store, Default::default()).unwrap(),
        "decimal"
    );
    assert_eq!(
        counter
            .additive_symbols(&store, Default::default())
            .unwrap(),
        ""
    );
    let additive = store.css_counter_style_rule(&ids[1]).unwrap();
    let _result = additive
        .set_additive_symbols(&mut store, "2 'B', 1 'A'", Default::default())
        .unwrap();
    assert_eq!(
        additive
            .additive_symbols(&store, Default::default())
            .unwrap(),
        "2 \"B\", 1 \"A\""
    );
    assert!(!changed(
        &counter
            .set_system(&mut store, "numeric", Default::default())
            .unwrap()
    ));
    assert!(!changed(
        &counter
            .set_symbols(&mut store, "", Default::default())
            .unwrap()
    ));
    let _result = counter.set_name(&mut store, "A B").unwrap();
    assert_eq!(counter.name(&store, Default::default()).unwrap(), r"A\ B");
    for reserved in [
        "NoNe",
        "DECIMAL",
        "disc",
        "Circle",
        "square",
        "disclosure-open",
        "disclosure-closed",
    ] {
        assert!(!changed(&counter.set_name(&mut store, reserved).unwrap()));
    }
    drop(store);
    assert_eq!(old.name(Default::default()).unwrap(), "c");
    assert_eq!(old.prefix(Default::default()).unwrap(), "");
}

#[test]
fn foreign_missing_and_detached_extension_identities_remain_checked() {
    let (mut store, sheet, ids) = create(
        "@font-feature-values A {} @counter-style c {system:fixed;symbols:'A';}",
        Default::default(),
    );
    let feature = store.css_font_feature_values_rule(&ids[0]).unwrap();
    let map = feature.annotation(&store).unwrap();
    let counter = store.css_counter_style_rule(&ids[1]).unwrap();
    let foreign = CssomStore::new(Default::default(), Default::default()).unwrap();
    assert!(matches!(
        feature.font_family(&foreign, Default::default()),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(map.size(&foreign), Err(CssomError::ForeignOwner)));
    let (other, _, _) = create("", Default::default());
    assert!(matches!(
        counter.capture(&other.snapshot()),
        Err(CssomError::ForeignOwner)
    ));
    let before = store.snapshot();
    let mut batch = store.batch(CssomEditGuard::from_snapshot(&before)).unwrap();
    batch.delete_sheet_rule(&sheet, 0).unwrap();
    batch.delete_sheet_rule(&sheet, 0).unwrap();
    store.commit(batch).unwrap();
    assert_eq!(counter.parent_style_sheet(&store).unwrap(), None);
    let _outcome = counter
        .set_prefix(&mut store, "'detached'", Default::default())
        .unwrap();
    let _outcome = map
        .set(&mut store, "retained", CssomFeatureValues::Scalar(1))
        .unwrap();
    assert_eq!(map.get(&store, "retained").unwrap(), Some(vec![1]));
    assert_eq!(
        counter.prefix(&store, Default::default()).unwrap(),
        "\"detached\""
    );
}

#[test]
fn custom_media_query_is_typed_live_or_same_snapshot_without_evaluation() {
    let (mut store, sheet, ids) = create(
        "@custom-media --True true; @custom-media --False false; @custom-media --List print; @custom-media --Empty;",
        Default::default(),
    );
    let true_rule = store.css_custom_media_rule(&ids[0]).unwrap();
    let false_rule = store.css_custom_media_rule(&ids[1]).unwrap();
    assert_eq!(
        true_rule.query(&store).unwrap(),
        CustomMediaQuery::Boolean(true)
    );
    assert_eq!(
        false_rule.query(&store).unwrap(),
        CustomMediaQuery::Boolean(false)
    );
    let rule = store.css_custom_media_rule(&ids[2]).unwrap();
    assert_eq!(rule.name(&store).unwrap(), "--List");
    let CustomMediaQuery::MediaList(list) = rule.query(&store).unwrap() else {
        panic!("typed MediaList")
    };
    let old = rule.capture(&store.snapshot()).unwrap();
    let CapturedCustomMediaQuery::MediaList(old_list) = old.query().unwrap() else {
        panic!("captured list")
    };
    assert_eq!(old_list.handle(), &list);
    assert_eq!(old_list.snapshot().revision(), old.snapshot().revision());
    let _result = list.set_media_text(&mut store, "screen,,speech").unwrap();
    assert_eq!(
        list.media_text(&store, Default::default()).unwrap(),
        "screen, not all, speech"
    );
    assert_eq!(
        rule.query(&store).unwrap(),
        CustomMediaQuery::MediaList(list.clone())
    );
    let empty_rule = store.css_custom_media_rule(&ids[3]).unwrap();
    let CustomMediaQuery::MediaList(empty) = empty_rule.query(&store).unwrap() else {
        panic!("empty is not Boolean")
    };
    assert_eq!(empty.length(&store).unwrap(), 0);
    assert_ne!(empty, list);
    let mut batch = store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap();
    batch.delete_sheet_rule(&sheet, 2).unwrap();
    store.commit(batch).unwrap();
    assert_eq!(rule.parent_style_sheet(&store).unwrap(), None);
    let _result = list.set_media_text(&mut store, "").unwrap();
    assert_eq!(
        rule.query(&store).unwrap(),
        CustomMediaQuery::MediaList(list.clone())
    );
    drop(store);
    assert_eq!(old.name().unwrap(), "--List");
    assert_eq!(old_list.media_text(Default::default()).unwrap(), "print");
}

#[test]
fn extension_resource_and_context_failures_preserve_revision_and_selected_payloads() {
    let (mut store, _, ids) = create(
        "@counter-style pending {system:fixed;symbols:'A';} @counter-style fixed {system:fixed;symbols:'A';}",
        Default::default(),
    );
    let pending = store.css_counter_style_rule(&ids[0]).unwrap();
    let fixed = store.css_counter_style_rule(&ids[1]).unwrap();
    let revision = store.snapshot().revision().clone();
    assert!(matches!(
        pending.set_symbols(&mut store, "env(counter-symbols)", Default::default()),
        Err(CssomError::CounterStylePreparationUnavailable(_))
    ));
    assert_eq!(store.snapshot().revision(), &revision);
    assert_eq!(pending.prefix(&store, Default::default()).unwrap(), "");
    let limits = CssomDeclarationRequestLimits {
        max_input_bytes: 0,
        ..Default::default()
    };
    assert!(matches!(
        fixed.set_prefix(&mut store, "'P'", limits),
        Err(CssomError::Limit { .. })
    ));
    assert_eq!(store.snapshot().revision(), &revision);
    assert_eq!(fixed.prefix(&store, Default::default()).unwrap(), "");
}
