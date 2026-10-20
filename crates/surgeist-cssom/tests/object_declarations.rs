use std::cell::Cell;
use surgeist_css::*;
use surgeist_cssom::*;

fn version(id: &str, revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Support,
        identity: id.into(),
        revision,
    }
}
fn context(conditional: bool) -> CssomContext {
    let requirement = |name: &str| {
        if conditional && name == "background" {
            CssomUsabilityRequirement::WholeDeclarationDecision
        } else {
            CssomUsabilityRequirement::AllCheckedValues
        }
    };
    let profile = CssomDeclarationSupport::try_new(
        [
            "width",
            "height",
            "color",
            "margin",
            "margin-top",
            "margin-right",
            "margin-bottom",
            "margin-left",
            "float",
            "opacity",
            "background",
        ]
        .map(|name| {
            (
                CssPropertyGrammar::from_name(name).unwrap(),
                requirement(name),
            )
        })
        .to_vec(),
    )
    .unwrap()
    .with_page_descriptors(
        [
            CssPageDescriptorKind::Size,
            CssPageDescriptorKind::PageOrientation,
            CssPageDescriptorKind::Marks,
            CssPageDescriptorKind::Bleed,
        ]
        .map(|kind| (kind, CssomUsabilityRequirement::AllCheckedValues))
        .to_vec(),
    )
    .unwrap()
    .with_font_face_descriptors(
        [
            CssFontFaceDescriptorKind::Src,
            CssFontFaceDescriptorKind::FontFamily,
            CssFontFaceDescriptorKind::FontStyle,
            CssFontFaceDescriptorKind::FontWeight,
            CssFontFaceDescriptorKind::FontWidth,
            CssFontFaceDescriptorKind::UnicodeRange,
            CssFontFaceDescriptorKind::FontFeatureSettings,
            CssFontFaceDescriptorKind::FontVariationSettings,
            CssFontFaceDescriptorKind::FontNamedInstance,
            CssFontFaceDescriptorKind::FontDisplay,
            CssFontFaceDescriptorKind::FontLanguageOverride,
            CssFontFaceDescriptorKind::AscentOverride,
            CssFontFaceDescriptorKind::DescentOverride,
            CssFontFaceDescriptorKind::LineGapOverride,
        ]
        .map(|kind| (kind, CssomUsabilityRequirement::AllCheckedValues))
        .to_vec(),
    )
    .unwrap();
    CssomContext {
        inputs: vec![CssomInput {
            version: version("renderer", 1),
            data: CssomInputData::Support(profile),
        }],
        ..Default::default()
    }
}
fn store() -> CssomStore {
    CssomStore::new(Default::default(), context(false)).unwrap()
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(
    store: &mut CssomStore,
    text: &str,
    computed: bool,
    readonly: bool,
    owner: bool,
) -> CssStyleProperties {
    let mut edit = batch(store);
    let ticket = edit
        .create_declarations(
            parse_declaration_block_contents(text).into_parts().0,
            computed,
            readonly,
            owner.then(|| CssomInputVersion {
                role: CssomInputRole::Owner,
                identity: "element".into(),
                revision: 1,
            }),
        )
        .unwrap();
    let publication = store.commit(edit).unwrap();
    store
        .css_style_properties(publication.block(&ticket).unwrap())
        .unwrap()
}
fn sheet(store: &mut CssomStore, text: &str) -> Vec<CssomRuleId> {
    let mut edit = batch(store);
    let ticket = edit
        .create_parsed_sheet(
            text,
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
    let publication = store.commit(edit).unwrap();
    store
        .snapshot()
        .sheet_rules(publication.sheet(&ticket).unwrap())
        .unwrap()
        .to_vec()
}
fn unused(_: &CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError> {
    panic!("no supplied decision required")
}
fn decide(
    request: &CssomAdmissionRequest<'_>,
    usable: CssomDeclarationUsability,
    stale: bool,
) -> CssomAdmission {
    let decision = version("gradient", 7);
    let decisions = request
        .candidates()
        .iter()
        .map(|candidate| candidate.decide(decision.clone(), usable).unwrap())
        .collect();
    let mut current = request
        .context()
        .inputs
        .iter()
        .map(|input| input.version.clone())
        .collect::<Vec<_>>();
    current.push(version("gradient", if stale { 8 } else { 7 }));
    CssomAdmission::new(decisions, current)
}

#[test]
fn property_common_reads_reflections_priority_and_immutable_capture() {
    let mut store = store();
    let object = create(
        &mut store,
        "float:left!important;width:1px;--Case:old",
        false,
        false,
        false,
    );
    let before = object.capture(&store.snapshot()).unwrap();
    assert_eq!(object.length(&store, Default::default()).unwrap(), 3);
    assert_eq!(object.item(&store, 1, Default::default()).unwrap(), "width");
    assert_eq!(object.item(&store, 99, Default::default()).unwrap(), "");
    assert_eq!(
        object.css_float(&store, Default::default()).unwrap(),
        "left"
    );
    assert_eq!(
        object
            .get_property_priority(&store, "FLOAT", Default::default())
            .unwrap(),
        "important"
    );
    assert!(object.parent_rule(&store).unwrap().is_none());
    let outcome = object
        .set_css_float(&mut store, "right", Default::default(), unused)
        .unwrap();
    assert_eq!(*outcome.value(), ());
    assert!(outcome.owner_effects().is_empty());
    assert_eq!(
        object
            .get_property_priority(&store, "float", Default::default())
            .unwrap(),
        ""
    );
    let removal = object
        .remove_property(&mut store, "width", Default::default())
        .unwrap();
    assert_eq!(removal.value(), "1px");
    let current = object.capture(&store.snapshot()).unwrap();
    drop(store);
    assert_eq!(before.css_float(Default::default()).unwrap(), "left");
    assert_eq!(before.item(2, Default::default()).unwrap(), "--Case");
    assert_eq!(
        before
            .get_property_priority("float", Default::default())
            .unwrap(),
        "important"
    );
    assert_eq!(
        current.css_text(Default::default()).unwrap(),
        "float: right; --Case: old;"
    );
    assert!(current.parent_rule().unwrap().is_none());
}

#[test]
fn page_nine_reflections_are_common_operations_and_keep_same_parent_capture() {
    let mut store = store();
    let rules = sheet(&mut store, "@page { @top-left {} }");
    let page_rule = store.css_page_rule(&rules[0]).unwrap();
    let page = page_rule.style(&store).unwrap();
    let old = page.capture(&store.snapshot()).unwrap();
    macro_rules! field {
        ($set:ident,$get:ident,$value:literal,$expected:literal) => {{
            let result = page
                .$set(&mut store, $value, Default::default(), unused)
                .unwrap();
            assert_eq!(*result.value(), ());
            assert_eq!(page.$get(&store, Default::default()).unwrap(), $expected);
            assert_eq!(old.$get(Default::default()).unwrap(), "");
        }};
    }
    field!(set_margin, margin, "1px 2px", "1px 2px");
    field!(set_margin_top, margin_top, "3px", "3px");
    field!(set_margin_right, margin_right, "4px", "4px");
    field!(set_margin_bottom, margin_bottom, "5px", "5px");
    field!(set_margin_left, margin_left, "6px", "6px");
    field!(set_size, size, "A4", "a4");
    field!(
        set_page_orientation,
        page_orientation,
        "rotate-left",
        "rotate-left"
    );
    field!(set_marks, marks, "crop", "crop");
    field!(set_bleed, bleed, "2px", "2px");
    let result = page
        .set_property(&mut store, "color", "red", None, Default::default(), unused)
        .unwrap();
    assert_eq!(*result.value(), ());
    assert_eq!(
        page.get_property_value(&store, "color", Default::default())
            .unwrap(),
        "red"
    );
    let current = page.capture(&store.snapshot()).unwrap();
    let parent = old.parent_rule().unwrap().unwrap();
    assert_eq!(parent.handle().id(), &rules[0]);
    assert_eq!(parent.snapshot().revision(), old.snapshot().revision());
    assert_eq!(
        parent
            .as_page_rule()
            .unwrap()
            .unwrap()
            .style()
            .unwrap()
            .handle()
            .id(),
        old.handle().id()
    );
    assert_eq!(
        current
            .parent_rule()
            .unwrap()
            .unwrap()
            .snapshot()
            .revision(),
        current.snapshot().revision()
    );
    assert!(old.snapshot().revision().value() < current.snapshot().revision().value());
}

#[test]
fn font_face_fifteen_reflections_aliases_empty_absence_and_common_priority_rules() {
    let mut store = store();
    let rules = sheet(&mut store, "@font-face {}");
    let font = store
        .css_font_face_rule(&rules[0])
        .unwrap()
        .style(&store)
        .unwrap();
    let old = font.capture(&store.snapshot()).unwrap();
    macro_rules! field {
        ($set:ident,$get:ident,$value:literal,$expected:literal) => {{
            let outcome = font
                .$set(&mut store, $value, Default::default(), unused)
                .unwrap();
            assert_eq!(*outcome.value(), ());
            assert_eq!(font.$get(&store, Default::default()).unwrap(), $expected);
            assert_eq!(old.$get(Default::default()).unwrap(), "");
        }};
    }
    field!(set_src, src, "local(A)", "local(\"A\")");
    field!(set_font_family, font_family, "B", "B");
    field!(set_font_style, font_style, "italic", "italic");
    field!(set_font_weight, font_weight, "700", "700");
    field!(set_font_stretch, font_stretch, "80%", "80%");
    field!(set_font_width, font_width, "90%", "90%");
    field!(set_unicode_range, unicode_range, "U+0-7F", "U+0-7F");
    field!(
        set_font_feature_settings,
        font_feature_settings,
        "'kern' 1",
        "\"kern\" 1"
    );
    field!(
        set_font_variation_settings,
        font_variation_settings,
        "'wght' 500",
        "\"wght\" 500"
    );
    field!(
        set_font_named_instance,
        font_named_instance,
        "'Bold'",
        "\"Bold\""
    );
    field!(set_font_display, font_display, "swap", "swap");
    field!(
        set_font_language_override,
        font_language_override,
        "'TRK'",
        "\"TRK\""
    );
    field!(set_ascent_override, ascent_override, "80%", "80%");
    field!(set_descent_override, descent_override, "20%", "20%");
    field!(set_line_gap_override, line_gap_override, "0%", "0%");
    assert_eq!(
        font.font_stretch(&store, Default::default()).unwrap(),
        "90%"
    );
    let before = store.snapshot();
    let no_op = font
        .set_property(
            &mut store,
            "font-width",
            "95%",
            Some("important"),
            Default::default(),
            unused,
        )
        .unwrap();
    assert!(matches!(
        no_op.publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(
        font.get_property_priority(&store, "font-width", Default::default())
            .unwrap(),
        ""
    );
    let old_value = font
        .remove_property(&mut store, "FONT-STRETCH", Default::default())
        .unwrap();
    assert_eq!(old_value.value(), "90%");
    assert_eq!(font.font_width(&store, Default::default()).unwrap(), "");
}

#[test]
fn margin_and_keyframe_use_common_fallback_without_ordinary_domain_admission() {
    let mut store = store();
    let rules = sheet(&mut store, "@page{@top-left{}} @keyframes fade{from{}}");
    let snapshot = store.snapshot();
    let margin_id = snapshot
        .rules(snapshot.rule(&rules[0]).unwrap().data().children().unwrap())
        .unwrap()[0]
        .clone();
    let frame_id = snapshot
        .rules(snapshot.rule(&rules[1]).unwrap().data().children().unwrap())
        .unwrap()[0]
        .clone();
    let margin = store
        .css_margin_rule(&margin_id)
        .unwrap()
        .style(&store)
        .unwrap();
    let frame: CssStyleDeclaration = store
        .css_keyframe_rule(&frame_id)
        .unwrap()
        .style(&store)
        .unwrap();
    for object in [margin.as_style_declaration(), frame] {
        let outcome = object
            .set_css_text(
                &mut store,
                "width:2px;opacity:.5;animation-name:bad;--X:yes",
                Default::default(),
                unused,
            )
            .unwrap();
        assert_eq!(*outcome.value(), ());
        assert_eq!(
            object
                .get_property_value(&store, "width", Default::default())
                .unwrap(),
            "2px"
        );
        assert_eq!(
            object
                .get_property_value(&store, "animation-name", Default::default())
                .unwrap(),
            ""
        );
        assert_eq!(
            object
                .get_property_value(&store, "--X", Default::default())
                .unwrap(),
            "yes"
        );
    }
    assert_eq!(
        margin
            .get_property_value(&store, "opacity", Default::default())
            .unwrap(),
        ""
    );
    assert_eq!(
        margin.parent_rule(&store).unwrap().unwrap().id(),
        &margin_id
    );
}

#[test]
fn css_text_recovery_uses_actual_input_and_correlation_then_capture_survives_store() {
    let mut store = CssomStore::new(Default::default(), context(true)).unwrap();
    let object = create(&mut store, "width:1px", false, false, false);
    let called = Cell::new(0);
    let outcome = object
        .set_css_text(
            &mut store,
            "width:2px;bad:lost;background:linear-gradient(red,blue);color:red;}",
            Default::default(),
            |request| {
                called.set(called.get() + 1);
                assert_eq!(request.candidates().len(), 1);
                assert!(matches!(
                    request.candidates()[0]
                        .declaration()
                        .unwrap()
                        .property_name(),
                    CssPropertyNameRef::Known(CssKnownProperty::Background)
                ));
                assert_eq!(request.context().inputs[0].version, version("renderer", 1));
                Ok(decide(request, CssomDeclarationUsability::Usable, false))
            },
        )
        .unwrap();
    assert_eq!(called.get(), 1);
    assert!(!outcome.diagnostics().is_empty());
    let captured = object.capture(&store.snapshot()).unwrap();
    drop(store);
    assert_eq!(
        captured.css_text(Default::default()).unwrap(),
        "width: 2px; background: linear-gradient(red, blue); color: red;"
    );
    let input = captured
        .snapshot()
        .block(captured.handle().id())
        .unwrap()
        .replacement_input()
        .unwrap();
    assert_eq!(
        input.source(),
        "width:2px;bad:lost;background:linear-gradient(red,blue);color:red;}"
    );
    assert_eq!(input.diagnostics(), outcome.diagnostics());
}

#[test]
fn stale_or_unusable_css_text_decisions_do_not_publish_unapproved_compound() {
    let mut store = CssomStore::new(Default::default(), context(true)).unwrap();
    let object = create(&mut store, "width:1px", false, false, false);
    let before = store.snapshot();
    assert!(matches!(
        object.set_css_text(
            &mut store,
            "background:linear-gradient(red,blue)",
            Default::default(),
            |request| Ok(decide(request, CssomDeclarationUsability::Usable, true))
        ),
        Err(CssomError::StaleDeclarationDecision)
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert_eq!(
        object
            .get_property_value(&store, "width", Default::default())
            .unwrap(),
        "1px"
    );
    let result = object
        .set_css_text(
            &mut store,
            "background:linear-gradient(red,blue);height:4px",
            Default::default(),
            |request| Ok(decide(request, CssomDeclarationUsability::Unusable, false)),
        )
        .unwrap();
    assert_eq!(*result.value(), ());
    assert_eq!(
        object.css_text(&store, Default::default()).unwrap(),
        "height: 4px;"
    );
}

#[test]
fn computed_and_readonly_flags_keep_independent_precedence_on_object_css_text() {
    for computed in [false, true] {
        for readonly in [false, true] {
            let mut store = store();
            let object = create(&mut store, "width:1px", computed, readonly, false);
            let before = store.snapshot();
            assert_eq!(
                object.css_text(&store, Default::default()).unwrap(),
                if computed { "" } else { "width: 1px;" }
            );
            assert_eq!(
                object
                    .get_property_value(&store, "width", Default::default())
                    .unwrap(),
                "1px"
            );
            let result = object.set_css_text(&mut store, "width:2px", Default::default(), unused);
            if readonly {
                assert!(matches!(
                    result,
                    Err(CssomError::Source(CssomException::NoModificationAllowed))
                ));
            } else if computed {
                assert!(matches!(
                    result,
                    Err(CssomError::ComputedStyleUpdatePrecondition)
                ));
            } else {
                assert_eq!(*result.unwrap().value(), ());
            }
            if computed || readonly {
                assert_eq!(store.snapshot().revision(), before.revision());
            }
        }
    }
}

#[test]
fn bounded_reads_and_css_text_failure_are_typed_and_fresh_retry_succeeds() {
    let mut store = store();
    let object = create(&mut store, "width:1px", false, false, false);
    let before = object.capture(&store.snapshot()).unwrap();
    let limits = CssomDeclarationRequestLimits {
        max_entries: 0,
        ..Default::default()
    };
    assert!(matches!(
        object.length(&store, limits),
        Err(CssomError::Limit { .. })
    ));
    assert!(matches!(
        before.item(0, limits),
        Err(CssomError::Limit { .. })
    ));
    let revision = store.snapshot().revision().clone();
    assert!(matches!(
        object.set_css_text(
            &mut store,
            "width:2px",
            CssomDeclarationRequestLimits {
                max_input_components: 0,
                ..Default::default()
            },
            unused
        ),
        Err(CssomError::ParseResource(_))
    ));
    assert_eq!(store.snapshot().revision(), &revision);
    let result = object
        .set_css_text(&mut store, "width:2px", Default::default(), unused)
        .unwrap();
    assert_eq!(*result.value(), ());
    assert_eq!(
        object
            .get_property_value(&store, "width", Default::default())
            .unwrap(),
        "2px"
    );
    assert_eq!(
        before
            .get_property_value("width", Default::default())
            .unwrap(),
        "1px"
    );
}

#[test]
fn explicit_editor_stages_css_text_without_publishing_and_host_failure_is_post_publication() {
    let mut store = store();
    let object = create(&mut store, "width:1px", false, false, true);
    let before = store.snapshot();
    let mut edit = batch(&store);
    let provisional = object
        .as_style_declaration()
        .edit(&mut edit)
        .unwrap()
        .set_css_text("width:2px", Default::default(), unused)
        .unwrap();
    assert_eq!(*provisional.value(), ());
    assert_eq!(store.snapshot().revision(), before.revision());
    let mut commit = store.commit(edit).unwrap();
    let mut effects = commit.take_owner_effects();
    assert_eq!(effects.len(), 1);
    let published = store.snapshot();
    assert_eq!(
        object
            .get_property_value(&store, "width", Default::default())
            .unwrap(),
        "2px"
    );
    store
        .begin_owner_application(&mut effects[0])
        .unwrap()
        .finish_host_failure();
    assert_eq!(effects[0].status(), CssomOwnerEffectStatus::HostFailed);
    assert!(!store.declaration_flags(object.id()).unwrap().updating);
    assert_eq!(store.snapshot().revision(), published.revision());
}

#[test]
fn factories_validate_owner_and_captures_validate_the_supplied_snapshot() {
    let mut store = store();
    let old = store.snapshot();
    let object = create(&mut store, "width:1px", false, false, false);
    assert!(matches!(
        object.capture(&old),
        Err(CssomError::MissingObject)
    ));
    let other = self::store();
    assert!(matches!(
        object.length(&other, Default::default()),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(
        object.capture(&other.snapshot()),
        Err(CssomError::ForeignOwner)
    ));
}

#[test]
fn pending_groups_keep_selected_survivors_and_unavailable_footprint_is_an_error() {
    let mut store = store();
    let object = create(&mut store, "overflow:var(--M);--M:1px", false, false, false);
    let before = object.capture(&store.snapshot()).unwrap();
    assert_eq!(
        object
            .get_property_value(&store, "overflow", Default::default())
            .unwrap(),
        "var(--M)"
    );
    let removal = object
        .remove_property(&mut store, "overflow-y", Default::default())
        .unwrap();
    assert_eq!(removal.value(), "");
    assert_eq!(
        object
            .get_property_value(&store, "overflow", Default::default())
            .unwrap(),
        ""
    );
    assert_eq!(object.length(&store, Default::default()).unwrap(), 2);
    assert_eq!(
        before
            .get_property_value("overflow", Default::default())
            .unwrap(),
        "var(--M)"
    );
    let unresolved = create(&mut store, "inset:var(--I)", false, false, false);
    assert!(
        matches!(unresolved.length(&store, Default::default()), Err(CssomError::Declaration(ref error)) if matches!(error.kind(), CssDeclarationBlockErrorKind::PendingFootprintUndetermined {..}))
    );
}
