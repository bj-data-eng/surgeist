fn map_capture(
    store: &CssomStore,
    snapshot: &CssomSnapshot,
    id: &CssomFeatureMapId,
) -> CssomCapture<CssFontFeatureValuesMap> {
    store
        .css_font_feature_values_map(id)
        .unwrap()
        .capture(snapshot)
        .unwrap()
}
fn font_style(
    store: &CssomStore,
    snapshot: &CssomSnapshot,
    rule: &CssomRuleId,
) -> CssomCapture<CssFontFaceDescriptors> {
    store
        .css_font_face_rule(rule)
        .unwrap()
        .style(store)
        .unwrap()
        .capture(snapshot)
        .unwrap()
}
use surgeist_css::component::CssValueOrigin;
use surgeist_css::rule::font_face::{
    CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptorKind, CssFontFaceFamily,
};
use surgeist_css::rule::font_feature_values::{
    CssFontFeatureValueKind, CssFontFeatureValueRef, CssFontFeatureValuesFamilyList,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_cssom::CssFontFaceDescriptors;
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
    let view = before.inspect_font_feature_values(&rules[0]).unwrap();
    let maps = [
        view.map(CssFontFeatureValueKind::Annotation),
        view.map(CssFontFeatureValueKind::Ornaments),
        view.map(CssFontFeatureValueKind::Stylistic),
        view.map(CssFontFeatureValueKind::Swash),
        view.map(CssFontFeatureValueKind::CharacterVariant),
        view.map(CssFontFeatureValueKind::Styleset),
        view.map(CssFontFeatureValueKind::HistoricalForms),
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
        let map = map_capture(&store, &after, id);
        assert_eq!(map.get("").unwrap(), Some(vec![u32::MAX]));
        assert_eq!(map.get("\\\\ A\0").unwrap(), Some(vec![]));
        assert_eq!(map.get("absent").unwrap(), None);
        assert!(map.has("\\\\ A\0").unwrap());
        assert_eq!(map.size().unwrap(), 2);
        assert_eq!(map.keys().unwrap().collect::<Vec<_>>(), ["", "\\\\ A\0"]);
        assert_eq!(
            map.values()
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
            [vec![u32::MAX], vec![]]
        );
        assert_eq!(
            after
                .inspect_font_feature_values(&rules[0])
                .unwrap()
                .map(after.feature_map(id).unwrap().kind()),
            id
        );
        assert!(!map_capture(&store, &before, id).has("").unwrap());
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
            .inspect_font_feature_values(&rules[0])
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
            map_capture(&store, &after, &id).get("key").unwrap(),
            Some(vec![0; maximum])
        );
        assert!(!map_capture(&store, &after, &id).has("unrelated").unwrap());
    }
    for kind in [
        CssFontFeatureValueKind::Styleset,
        CssFontFeatureValueKind::HistoricalForms,
    ] {
        let snapshot = store.snapshot();
        let id = snapshot
            .inspect_font_feature_values(&rules[0])
            .unwrap()
            .map(kind)
            .clone();
        let mut edit = batch(&store);
        edit.set_feature(&id, "long", &[0, 99, 100, u32::MAX, 5, 6])
            .unwrap();
        store.commit(edit).unwrap();
        assert_eq!(
            map_capture(&store, &store.snapshot(), &id)
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
        .inspect_font_feature_values(&rules[0])
        .unwrap()
        .map(CssFontFeatureValueKind::Styleset)
        .clone();
    let map = before.feature_map(&id).unwrap();
    let projected = map_capture(&store, &before, &id);
    assert!(projected.has("exact").unwrap());
    assert!(matches!(
        projected.get("exact"),
        Err(CssomError::AuthoredFeatureConversion)
    ));
    assert!(matches!(
        projected.values().unwrap().next().unwrap(),
        Err(CssomError::AuthoredFeatureConversion)
    ));
    let CssomFeatureValue::Authored(value) = &map.raw_entries()[0].value else {
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
    assert_eq!(map_capture(&store, &after, &id).size().unwrap(), 0);
    assert_eq!(
        after.feature_map(&id).unwrap().authored(),
        before.feature_map(&id).unwrap().authored()
    );
    assert!(map_capture(&store, &before, &id).has("exact").unwrap());
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
        .inspect_font_feature_values(&rules[0])
        .unwrap()
        .map(CssFontFeatureValueKind::Annotation)
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
        after
            .inspect_font_feature_values(&rules[0])
            .unwrap()
            .families()[0]
            .as_str(),
        "B C"
    );
    assert_eq!(
        before
            .inspect_font_feature_values(&rules[0])
            .unwrap()
            .families()[0]
            .as_str(),
        "A"
    );
    assert_eq!(
        after
            .inspect_font_feature_values(&rules[0])
            .unwrap()
            .map(CssFontFeatureValueKind::Annotation),
        &map
    );
    assert_eq!(
        map_capture(&store, &after, &map).get("x").unwrap(),
        Some(vec![1])
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
        .inspect_font_feature_values(&rule)
        .unwrap()
        .map(CssFontFeatureValueKind::Styleset)
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
        after.inspect_font_feature_values(&rule).unwrap().families()[0].as_str(),
        "A"
    );
    assert_eq!(
        map_capture(&store, &after, &map).get("x").unwrap(),
        Some(vec![])
    );
    // Independent immutable captures retain old values without being current live state.
    assert_eq!(
        before
            .inspect_font_feature_values(&rule)
            .unwrap()
            .families()[0]
            .as_str(),
        "é"
    );
    assert!(!map_capture(&store, &before, &map).has("x").unwrap());
}
#[test]
fn readonly_palette_attributes_use_effective_checked_descriptors_and_honest_empty_absence() {
    let (store, _, rules) = create(
        "@font-palette-values --Palette { font-family: A; base-palette:light; base-palette:dark; override-colors:0 red, 1 blue; } @font-palette-values --Empty {font-family:B;}",
    );
    let snapshot = store.snapshot();
    let view = store
        .css_font_palette_values_rule(&rules[0])
        .unwrap()
        .capture(&snapshot)
        .unwrap();
    assert_eq!(view.name().unwrap(), "--Palette");
    assert_eq!(view.font_family(Default::default()).unwrap(), "A");
    assert_eq!(view.base_palette(Default::default()).unwrap(), "dark");
    assert_eq!(
        view.override_colors(Default::default()).unwrap(),
        "0 red, 1 blue"
    );
    assert!(matches!(
        view.base_palette(CssSerializationLimits::new(100, 100, 0)),
        Err(CssomError::Value(_))
    ));
    let empty = store
        .css_font_palette_values_rule(&rules[1])
        .unwrap()
        .capture(&snapshot)
        .unwrap();
    assert_eq!(
        empty
            .base_palette(CssSerializationLimits::new(0, 0, 0))
            .unwrap(),
        ""
    );
    assert_eq!(empty.override_colors(Default::default()).unwrap(), "");
    assert!(matches!(
        snapshot.inspect_font_feature_values(&rules[0]),
        Err(CssomError::WrongKind)
    ));
}

#[test]
fn map_order_replace_delete_reinsert_and_detached_retention_have_distinct_effects() {
    let (mut store, sheet, rules) = create("@font-feature-values A {}");
    let snapshot = store.snapshot();
    let id = snapshot
        .inspect_font_feature_values(&rules[0])
        .unwrap()
        .map(CssFontFeatureValueKind::Styleset)
        .clone();
    let mut edit = batch(&store);
    edit.set_feature(&id, "a", &[1]).unwrap();
    edit.set_feature(&id, "b", &[2]).unwrap();
    edit.set_feature(&id, "a", &[3]).unwrap();
    store.commit(edit).unwrap();
    let before = store.snapshot();
    assert_eq!(
        map_capture(&store, &before, &id)
            .keys()
            .unwrap()
            .collect::<Vec<_>>(),
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
        map_capture(&store, &after, &id)
            .keys()
            .unwrap()
            .collect::<Vec<_>>(),
        ["b", "a"]
    );
    assert_eq!(
        after
            .inspect_font_feature_values(&rules[0])
            .unwrap()
            .map(CssFontFeatureValueKind::Styleset),
        &id
    );
    let before_map = map_capture(&store, &before, &id);
    let after_map = map_capture(&store, &after, &id);
    drop(store);
    assert_eq!(before_map.get("a").unwrap(), Some(vec![3]));
    assert_eq!(after_map.get("a").unwrap(), Some(vec![4]));
}
#[test]
fn font_views_and_clear_enforce_real_owner_identity() {
    let (mut a, _, arules) = create("@font-feature-values A {}");
    let (b, _, brules) = create("@font-feature-values B {}");
    let bsnapshot = b.snapshot();
    let foreign = bsnapshot
        .inspect_font_feature_values(&brules[0])
        .unwrap()
        .map(CssFontFeatureValueKind::Styleset);
    assert!(matches!(
        a.snapshot().inspect_font_feature_values(&brules[0]),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(
        a.css_font_palette_values_rule(&arules[0]),
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
    let view = store
        .css_font_palette_values_rule(&rules[0])
        .unwrap()
        .capture(&snapshot)
        .unwrap();
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
    let view = font_style(&store, &before, &rule);
    let style = view.handle().id().clone();
    assert_eq!(view.parent_rule().unwrap().unwrap().handle().id(), &rule);
    assert_eq!(view.font_family(Default::default()).unwrap(), "A");
    assert_eq!(view.font_width(Default::default()).unwrap(), "80%");
    assert_eq!(view.font_stretch(Default::default()).unwrap(), "80%");
    assert_eq!(view.src(Default::default()).unwrap(), "local(\"A\")");
    assert_eq!(view.font_display(Default::default()).unwrap(), "swap");
    for absent in [
        view.font_style(Default::default()),
        view.font_weight(Default::default()),
        view.unicode_range(Default::default()),
        view.font_feature_settings(Default::default()),
        view.font_variation_settings(Default::default()),
        view.font_named_instance(Default::default()),
        view.font_language_override(Default::default()),
        view.ascent_override(Default::default()),
        view.descent_override(Default::default()),
        view.line_gap_override(Default::default()),
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
    let view = font_style(&store, &snapshot, &rule);
    assert_eq!(view.handle().id(), &style);
    assert_eq!(view.font_stretch(Default::default()).unwrap(), "90%");
    assert_eq!(
        font_style(&store, &before, &rule)
            .font_width(Default::default())
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
    let view = font_style(&store, &snapshot, &rule);
    assert_eq!(view.handle().id(), &style);
    assert_eq!(view.font_family(Default::default()).unwrap(), "B");
    assert_eq!(view.font_display(Default::default()).unwrap(), "block");
    assert_eq!(view.font_width(Default::default()).unwrap(), "");
    assert_eq!(
        view.css_text(Default::default()).unwrap(),
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
    let view = font_style(&store, &snapshot, &rule);
    assert_eq!(view.font_family(Default::default()).unwrap(), "A");
    assert_eq!(view.font_stretch(Default::default()).unwrap(), "90%");
}

#[test]
fn font_face_pending_descriptor_mutation_keeps_current_style_id_and_owning_origin() {
    let (mut store, rule) = font_face_store(false);
    let before = store.snapshot();
    let style = font_style(&store, &before, &rule).handle().id().clone();
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
    let view = font_style(&store, &snapshot, &rule);
    assert_eq!(view.handle().id(), &style);
    assert_eq!(
        view.font_width(Default::default()).unwrap(),
        "env(width, 90%)"
    );
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
    let old_style = font_style(&store, &before, &rule);
    let new_style = font_style(&store, &snapshot, &rule);
    drop(store);
    assert_eq!(old_style.font_width(Default::default()).unwrap(), "80%");
    assert_eq!(
        new_style.font_width(Default::default()).unwrap(),
        "env(width, 90%)"
    );
}
