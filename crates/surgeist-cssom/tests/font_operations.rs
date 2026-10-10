use surgeist_css::*;
use surgeist_cssom::*;

fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(text: &str) -> (CssomStore, CssomSheetId, Vec<CssomRuleId>) {
    let mut store = CssomStore::new(Default::default(), Default::default()).unwrap();
    let mut edit = batch(&store);
    let sheet = edit
        .create_parsed_sheet(
            text,
            CssomSheetInputs::constructed(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "doc".into(),
                    revision: 1,
                },
                "doc".into(),
                None,
            ),
        )
        .unwrap();
    let id = store.commit(edit).unwrap().sheet(&sheet).unwrap().clone();
    let snapshot = store.snapshot();
    let rules = snapshot
        .rules(snapshot.sheet(&id).unwrap().rules())
        .unwrap()
        .to_vec();
    (store, id, rules)
}
#[test]
fn seven_named_maps_scalar_empty_sequence_and_raw_keys_are_source_distinct() {
    let (mut store, _, rules) = create("@font-feature-values A {}");
    let before = store.snapshot();
    let view = before.font_feature_values(&rules[0]).unwrap();
    let maps = [
        view.annotation(),
        view.ornaments(),
        view.stylistic(),
        view.swash(),
        view.character_variant(),
        view.styleset(),
        view.historical_forms(),
    ];
    assert_eq!(
        maps.iter().collect::<std::collections::HashSet<_>>().len(),
        7
    );
    for id in maps {
        let mut edit = batch(&store);
        assert!(
            edit.set_feature(id, "", CssomFeatureValues::Scalar(u32::MAX).as_sequence())
                .unwrap()
        );
        assert!(
            edit.set_feature(
                id,
                "\\\\ A\0",
                CssomFeatureValues::Sequence(vec![]).as_sequence()
            )
            .unwrap()
        );
        store.commit(edit).unwrap();
        let after = store.snapshot();
        let map = after.feature_map(id).unwrap();
        assert_eq!(map.get("").unwrap(), Some([u32::MAX].as_slice()));
        assert_eq!(map.get("\\\\ A\0").unwrap(), Some([].as_slice()));
        assert_eq!(map.get("absent").unwrap(), None);
        assert!(map.has("\\\\ A\0"));
        assert_eq!(map.size(), 2);
        assert_eq!(map.keys().collect::<Vec<_>>(), ["", "\\\\ A\0"]);
        assert_eq!(
            map.values().collect::<Result<Vec<_>, _>>().unwrap(),
            [vec![u32::MAX], vec![]]
        );
        assert_eq!(
            after
                .font_feature_values(&rules[0])
                .unwrap()
                .map(map.kind()),
            id
        );
        assert!(!before.feature_map(id).unwrap().has(""));
    }
}
#[test]
fn independent_map_cardinalities_reject_without_changing_entry_or_publication() {
    let (mut store, _, rules) = create("@font-feature-values A {}");
    for (kind, maximum) in [
        (CssFontFeatureValueKind::Annotation, 1),
        (CssFontFeatureValueKind::Ornaments, 1),
        (CssFontFeatureValueKind::Stylistic, 1),
        (CssFontFeatureValueKind::Swash, 1),
        (CssFontFeatureValueKind::CharacterVariant, 2),
    ] {
        let before = store.snapshot();
        let id = before
            .font_feature_values(&rules[0])
            .unwrap()
            .map(kind)
            .clone();
        let mut edit = batch(&store);
        edit.set_feature(&id, "key", &vec![0; maximum]).unwrap();
        store.commit(edit).unwrap();
        let before = store.snapshot();
        let mut edit = batch(&store);
        edit.set_feature(&id, "unrelated", &[99]).unwrap();
        assert!(matches!(
            edit.set_feature(&id, "key", &vec![5; maximum + 1]),
            Err(CssomError::Source(CssomException::InvalidAccess))
        ));
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        let after = store.snapshot();
        assert_eq!(after.revision(), before.revision());
        assert_eq!(
            after.feature_map(&id).unwrap().get("key").unwrap(),
            Some(vec![0; maximum].as_slice())
        );
        assert!(!after.feature_map(&id).unwrap().has("unrelated"));
    }
    for kind in [
        CssFontFeatureValueKind::Styleset,
        CssFontFeatureValueKind::HistoricalForms,
    ] {
        let snapshot = store.snapshot();
        let id = snapshot
            .font_feature_values(&rules[0])
            .unwrap()
            .map(kind)
            .clone();
        let mut edit = batch(&store);
        edit.set_feature(&id, "long", &[0, 99, 100, u32::MAX, 5, 6])
            .unwrap();
        store.commit(edit).unwrap();
        assert_eq!(
            store
                .snapshot()
                .feature_map(&id)
                .unwrap()
                .get("long")
                .unwrap()
                .unwrap()
                .len(),
            6
        );
    }
}
#[test]
fn exact_unrepresentable_authored_value_stays_present_then_live_clear_retains_occurrences() {
    let (mut store, _, rules) =
        create("@font-feature-values A { @styleset { exact: 4294967296; } }");
    let before = store.snapshot();
    let id = before
        .font_feature_values(&rules[0])
        .unwrap()
        .styleset()
        .clone();
    let map = before.feature_map(&id).unwrap();
    assert!(map.has("exact"));
    assert!(matches!(
        map.get("exact"),
        Err(CssomError::AuthoredFeatureConversion)
    ));
    assert!(matches!(
        map.values().next().unwrap(),
        Err(CssomError::AuthoredFeatureConversion)
    ));
    let CssomFeatureValue::Authored(value) = &map.entries()[0].value else {
        panic!("exact authored value required")
    };
    let CssFontFeatureValueRef::Indexes(indexes) = value.view() else {
        panic!("exact indexes required")
    };
    assert_eq!(indexes[0].as_decimal_str(), "4294967296");
    assert_eq!(
        map.authored()[0].parsed_name().unwrap().source().as_str(),
        "@font-feature-values A { @styleset { exact: 4294967296; } }"
    );
    let mut edit = batch(&store);
    assert!(edit.clear_feature_map(&id).unwrap());
    assert!(!edit.clear_feature_map(&id).unwrap());
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(after.feature_map(&id).unwrap().size(), 0);
    assert_eq!(
        after.feature_map(&id).unwrap().authored(),
        before.feature_map(&id).unwrap().authored()
    );
    assert!(before.feature_map(&id).unwrap().has("exact"));
    let revision = after.revision().clone();
    let mut edit = batch(&store);
    assert!(!edit.clear_feature_map(&id).unwrap());
    store.commit(edit).unwrap();
    assert_eq!(store.snapshot().revision(), &revision);
}
#[test]
fn typed_family_replacement_preserves_maps_and_authored_rule_capture() {
    let (mut store, _, rules) = create("@font-feature-values A { @annotation { x: 1; } }");
    let before = store.snapshot();
    let map = before
        .font_feature_values(&rules[0])
        .unwrap()
        .annotation()
        .clone();
    let mut edit = batch(&store);
    assert!(
        edit.set_font_feature_families(
            &rules[0],
            CssFontFeatureValuesFamilyList::try_new(vec![
                CssFontFaceFamily::try_new("B C").unwrap()
            ])
            .unwrap()
        )
        .unwrap()
    );
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        after.font_feature_values(&rules[0]).unwrap().families()[0].as_str(),
        "B C"
    );
    assert_eq!(
        before.font_feature_values(&rules[0]).unwrap().families()[0].as_str(),
        "A"
    );
    assert_eq!(
        after.font_feature_values(&rules[0]).unwrap().annotation(),
        &map
    );
    assert_eq!(
        after.feature_map(&map).unwrap().get("x").unwrap(),
        Some([1].as_slice())
    );
    let revision = after.revision().clone();
    let mut edit = batch(&store);
    assert!(
        !edit
            .set_font_feature_families(
                &rules[0],
                CssFontFeatureValuesFamilyList::try_new(vec![
                    CssFontFaceFamily::try_new("B C").unwrap()
                ])
                .unwrap()
            )
            .unwrap()
    );
    store.commit(edit).unwrap();
    assert_eq!(store.snapshot().revision(), &revision);
}
#[test]
fn mutable_families_share_exact_utf8_quota_and_release_only_current_values() {
    // Six sheet host bytes plus two UTF-8 family bytes fill this allowance.
    let mut store = CssomStore::new(
        CssomLimits {
            max_string_bytes: 8,
            ..Default::default()
        },
        Default::default(),
    )
    .unwrap();
    let mut edit = batch(&store);
    let ticket = edit
        .create_parsed_sheet(
            "@font-feature-values \"é\" {}",
            CssomSheetInputs::constructed(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "doc".into(),
                    revision: 1,
                },
                "doc".into(),
                None,
            ),
        )
        .unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let before = store.snapshot();
    let rule = before.rules(before.sheet(&sheet).unwrap().rules()).unwrap()[0].clone();
    let map = before
        .font_feature_values(&rule)
        .unwrap()
        .styleset()
        .clone();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.set_font_feature_families(
            &rule,
            CssFontFeatureValuesFamilyList::try_new(vec![
                CssFontFaceFamily::try_new("AAA").unwrap()
            ])
            .unwrap()
        ),
        Err(CssomError::Limit {
            resource: "host and live string bytes",
            maximum: 8
        })
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(store.snapshot().revision(), before.revision());
    let mut edit = batch(&store);
    assert!(
        edit.set_font_feature_families(
            &rule,
            CssFontFeatureValuesFamilyList::try_new(vec![CssFontFaceFamily::try_new("A").unwrap()])
                .unwrap()
        )
        .unwrap()
    );
    assert!(edit.set_feature(&map, "x", &[]).unwrap());
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        after.font_feature_values(&rule).unwrap().families()[0].as_str(),
        "A"
    );
    assert_eq!(
        after.feature_map(&map).unwrap().get("x").unwrap(),
        Some([].as_slice())
    );
    // Independent immutable captures retain old values without being current live state.
    assert_eq!(
        before.font_feature_values(&rule).unwrap().families()[0].as_str(),
        "é"
    );
    assert!(!before.feature_map(&map).unwrap().has("x"));
}
#[test]
fn readonly_palette_attributes_use_effective_checked_descriptors_and_honest_empty_absence() {
    let (store, _, rules) = create(
        "@font-palette-values --Palette { font-family: A; base-palette:light; base-palette:dark; override-colors:0 red, 1 blue; } @font-palette-values --Empty {font-family:B;}",
    );
    let snapshot = store.snapshot();
    let view = snapshot.font_palette_values(&rules[0]).unwrap();
    assert_eq!(view.name(), "--Palette");
    assert_eq!(view.font_family(Default::default()).unwrap(), "A");
    assert_eq!(view.base_palette(Default::default()).unwrap(), "dark");
    assert_eq!(
        view.override_colors(Default::default()).unwrap(),
        "0 red, 1 blue"
    );
    assert!(matches!(
        view.base_palette(CssSpecifiedValueSerializationLimits::new(100, 100, 0)),
        Err(CssomError::Value(_))
    ));
    let empty = snapshot.font_palette_values(&rules[1]).unwrap();
    assert_eq!(
        empty
            .base_palette(CssSpecifiedValueSerializationLimits::new(0, 0, 0))
            .unwrap(),
        ""
    );
    assert_eq!(empty.override_colors(Default::default()).unwrap(), "");
    assert!(matches!(
        snapshot.font_feature_values(&rules[0]),
        Err(CssomError::WrongKind)
    ));
}

#[test]
fn map_order_replace_delete_reinsert_and_detached_retention_have_distinct_effects() {
    let (mut store, sheet, rules) = create("@font-feature-values A {}");
    let snapshot = store.snapshot();
    let id = snapshot
        .font_feature_values(&rules[0])
        .unwrap()
        .styleset()
        .clone();
    let mut edit = batch(&store);
    edit.set_feature(&id, "a", &[1]).unwrap();
    edit.set_feature(&id, "b", &[2]).unwrap();
    edit.set_feature(&id, "a", &[3]).unwrap();
    store.commit(edit).unwrap();
    let before = store.snapshot();
    assert_eq!(
        before.feature_map(&id).unwrap().keys().collect::<Vec<_>>(),
        ["a", "b"]
    );
    let mut edit = batch(&store);
    assert!(edit.delete_feature(&id, "a").unwrap());
    assert!(!edit.delete_feature(&id, "missing").unwrap());
    edit.set_feature(&id, "a", &[4]).unwrap();
    edit.delete_sheet_rule(&sheet, 0).unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        after.feature_map(&id).unwrap().keys().collect::<Vec<_>>(),
        ["b", "a"]
    );
    assert_eq!(
        after.font_feature_values(&rules[0]).unwrap().styleset(),
        &id
    );
    drop(store);
    assert_eq!(
        before.feature_map(&id).unwrap().get("a").unwrap(),
        Some([3].as_slice())
    );
    assert_eq!(
        after.feature_map(&id).unwrap().get("a").unwrap(),
        Some([4].as_slice())
    );
}
#[test]
fn font_views_and_clear_enforce_real_owner_identity() {
    let (mut a, _, arules) = create("@font-feature-values A {}");
    let (b, _, brules) = create("@font-feature-values B {}");
    let bsnapshot = b.snapshot();
    let foreign = bsnapshot
        .font_feature_values(&brules[0])
        .unwrap()
        .styleset();
    assert!(matches!(
        a.snapshot().font_feature_values(&brules[0]),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(
        a.snapshot().font_palette_values(&arules[0]),
        Err(CssomError::WrongKind)
    ));
    let before = a.snapshot();
    let mut edit = batch(&a);
    assert!(matches!(
        edit.clear_feature_map(foreign),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(a.commit(edit), Err(CssomError::BatchAborted)));
    assert_eq!(a.snapshot().revision(), before.revision());
}

#[test]
fn readonly_palette_pending_values_retain_literal_substitution_and_case() {
    let (store, _, rules) = create(
        "@font-palette-values --p {font-family:var(--Family);base-palette:var(--Base);override-colors:var(--Colors)}",
    );
    let snapshot = store.snapshot();
    let view = snapshot.font_palette_values(&rules[0]).unwrap();
    assert_eq!(
        view.font_family(Default::default()).unwrap(),
        "var(--Family)"
    );
    assert_eq!(
        view.base_palette(Default::default()).unwrap(),
        "var(--Base)"
    );
    assert_eq!(
        view.override_colors(Default::default()).unwrap(),
        "var(--Colors)"
    );
}

fn font_face_store(conditional: bool) -> (CssomStore, CssomRuleId) {
    let support = CssomDeclarationSupport::try_new(vec![])
        .unwrap()
        .with_font_face_descriptors(
            [
                CssFontFaceDescriptorKind::FontFamily,
                CssFontFaceDescriptorKind::FontWidth,
                CssFontFaceDescriptorKind::Src,
                CssFontFaceDescriptorKind::FontDisplay,
            ]
            .into_iter()
            .map(|kind| {
                (
                    kind,
                    if conditional && kind == CssFontFaceDescriptorKind::FontFamily {
                        CssomUsabilityRequirement::WholeDeclarationDecision
                    } else {
                        CssomUsabilityRequirement::AllCheckedValues
                    },
                )
            })
            .collect(),
        )
        .unwrap();
    let context = CssomContext {
        inputs: vec![CssomInput {
            version: CssomInputVersion {
                role: CssomInputRole::Support,
                identity: "fonts".into(),
                revision: 1,
            },
            data: CssomInputData::Support(support),
        }],
        ..Default::default()
    };
    let mut store = CssomStore::new(Default::default(), context).unwrap();
    let mut edit = batch(&store);
    let ticket = edit
        .create_parsed_sheet(
            "@font-face {font-family:A;font-width:80%;src:local(A);font-display:swap}",
            CssomSheetInputs::external(
                CssomInputVersion {
                    role: CssomInputRole::Document,
                    identity: "sheet".into(),
                    revision: 1,
                },
                true,
            ),
        )
        .unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    let rule = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap()[0]
        .clone();
    (store, rule)
}
#[test]
fn font_face_reflections_same_object_and_putforwards_share_selected_owner() {
    let (mut store, rule) = font_face_store(false);
    let before = store.snapshot();
    let view = before.font_face(&rule, Default::default()).unwrap();
    let style = view.style_id().clone();
    assert_eq!(view.style().parent_rule(), Some(&rule));
    assert_eq!(view.font_family().unwrap(), "A");
    assert_eq!(view.font_width().unwrap(), "80%");
    assert_eq!(view.font_stretch().unwrap(), "80%");
    assert_eq!(view.src().unwrap(), "local(\"A\")");
    assert_eq!(view.font_display().unwrap(), "swap");
    for absent in [
        view.font_style(),
        view.font_weight(),
        view.unicode_range(),
        view.font_feature_settings(),
        view.font_variation_settings(),
        view.font_named_instance(),
        view.font_language_override(),
        view.ascent_override(),
        view.descent_override(),
        view.line_gap_override(),
    ] {
        assert_eq!(absent.unwrap(), "");
    }
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_font_face_descriptor(
            &rule,
            CssFontFaceDescriptorKind::FontWidth,
            "90%",
            Default::default(),
        )
        .unwrap();
    assert_eq!(
        edit.apply_declaration(prepared, &[]).unwrap(),
        CssomDeclarationEditResult::Set { updated: true }
    );
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    let view = snapshot.font_face(&rule, Default::default()).unwrap();
    assert_eq!(view.style_id(), &style);
    assert_eq!(view.font_stretch().unwrap(), "90%");
    assert_eq!(
        before
            .font_face(&rule, Default::default())
            .unwrap()
            .font_width()
            .unwrap(),
        "80%"
    );
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_font_face_css_text(
            &rule,
            "font-family:B;font-display:block;src:local(B);invalid:lost",
            Default::default(),
        )
        .unwrap();
    assert!(!prepared.diagnostics().is_empty());
    edit.apply_declaration(prepared, &[]).unwrap();
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    let view = snapshot.font_face(&rule, Default::default()).unwrap();
    assert_eq!(view.style_id(), &style);
    assert_eq!(view.font_family().unwrap(), "B");
    assert_eq!(view.font_display().unwrap(), "block");
    assert_eq!(view.font_width().unwrap(), "");
    assert_eq!(
        view.style().css_text().unwrap(),
        "font-family: B; font-display: block; src: local(\"B\");"
    );
}
#[test]
fn font_face_partial_support_uses_actual_correlated_decision_and_source_invalid_noop() {
    let (mut store, rule) = font_face_store(true);
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_font_face_descriptor(
            &rule,
            CssFontFaceDescriptorKind::FontFamily,
            "B",
            Default::default(),
        )
        .unwrap();
    assert_eq!(prepared.candidates().len(), 1);
    assert!(matches!(
        prepared.candidates()[0].value(),
        CssomDeclarationValue::FontFaceDescriptor(_)
    ));
    let version = prepared.profile_version().unwrap().clone();
    let rejected = prepared.candidates()[0]
        .decide(version, CssomDeclarationUsability::Unusable)
        .unwrap();
    assert_eq!(
        edit.apply_declaration(prepared, &[rejected]).unwrap(),
        CssomDeclarationEditResult::Noop
    );
    let prepared = edit
        .prepare_font_face_descriptor(
            &rule,
            CssFontFaceDescriptorKind::FontWidth,
            "90%",
            Default::default(),
        )
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
    let invalid = edit
        .prepare_font_face_descriptor(
            &rule,
            CssFontFaceDescriptorKind::FontWidth,
            "]",
            Default::default(),
        )
        .unwrap();
    assert_eq!(
        edit.apply_declaration(invalid, &[]).unwrap(),
        CssomDeclarationEditResult::Noop
    );
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    let view = snapshot.font_face(&rule, Default::default()).unwrap();
    assert_eq!(view.font_family().unwrap(), "A");
    assert_eq!(view.font_stretch().unwrap(), "90%");
}

#[test]
fn font_face_pending_descriptor_mutation_keeps_current_style_id_and_owning_origin() {
    let (mut store, rule) = font_face_store(false);
    let before = store.snapshot();
    let style = before
        .font_face(&rule, Default::default())
        .unwrap()
        .style_id()
        .clone();
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_font_face_descriptor(
            &rule,
            CssFontFaceDescriptorKind::FontWidth,
            "env(width, 90%)",
            Default::default(),
        )
        .unwrap();
    edit.apply_declaration(prepared, &[]).unwrap();
    store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    let view = snapshot.font_face(&rule, Default::default()).unwrap();
    assert_eq!(view.style_id(), &style);
    assert_eq!(view.font_width().unwrap(), "env(width, 90%)");
    let selected = snapshot
        .block(&style)
        .unwrap()
        .selected_font_face()
        .unwrap();
    let CssomProjection::Available(selected) = selected else {
        panic!("checked selected font block")
    };
    let occurrence = selected
        .entries()
        .iter()
        .find(|entry| entry.value().kind() == CssFontFaceDescriptorKind::FontWidth)
        .unwrap();
    assert!(occurrence.position().is_none());
    let CssAuthoredFontFaceDescriptorValue::Pending(value) = occurrence.value() else {
        panic!("honest pending descriptor")
    };
    let CssValueOrigin::Parsed(origin) = value.components().items()[0].origin() else {
        panic!("actual parsed component origin")
    };
    assert_eq!(origin.source().as_str(), "env(width, 90%)");
    drop(store);
    assert_eq!(
        before
            .font_face(&rule, Default::default())
            .unwrap()
            .font_width()
            .unwrap(),
        "80%"
    );
    assert_eq!(
        snapshot
            .font_face(&rule, Default::default())
            .unwrap()
            .font_width()
            .unwrap(),
        "env(width, 90%)"
    );
}
