use std::collections::HashSet;
use surgeist_cssom::*;

fn store() -> CssomStore {
    CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap()
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(store: &mut CssomStore, source: &str) -> (CssomSheetId, Vec<CssomRuleId>) {
    let mut edit = batch(store);
    let sheet = edit
        .create_parsed_sheet(
            source,
            CssomSheetInputs::external(
                CssomInputVersion {
                    role: CssomInputRole::Owner,
                    identity: "sheet".into(),
                    revision: 1,
                },
                true,
            ),
        )
        .unwrap();
    let id = store.commit(edit).unwrap().sheet(&sheet).unwrap().clone();
    let snapshot = store.snapshot();
    let rules = snapshot
        .rules(snapshot.sheet(&id).unwrap().rules())
        .unwrap()
        .to_vec();
    (id, rules)
}

#[test]
fn live_and_captured_parent_chains_retain_identity_but_observe_distinct_states() {
    let mut owner = store();
    let (sheet, rules) = create(
        &mut owner,
        ".parent { color: red; & .child { color: blue; } }",
    );
    let original = owner.snapshot();
    let child = original
        .rules(original.child_rules(&rules[0]).unwrap())
        .unwrap()[0]
        .clone();
    let parent = owner.css_style_rule(&rules[0]).unwrap();
    let child = owner.css_style_rule(&child).unwrap();
    let captured = child.capture(&original).unwrap();
    let original_parent = captured
        .parent_rule()
        .unwrap()
        .unwrap()
        .as_style_rule()
        .unwrap()
        .unwrap();
    let before = original_parent
        .css_text(&CssomRuleFormatLimits::default())
        .unwrap();
    assert_eq!(original_parent.handle(), &parent);
    assert_eq!(
        child
            .parent_rule(&owner)
            .unwrap()
            .unwrap()
            .as_style_rule(&owner)
            .unwrap()
            .unwrap(),
        parent
    );
    let mut identities = HashSet::new();
    identities.insert(parent.clone());
    let mut edit = batch(&owner);
    assert!(edit.set_style_selector_text(&rules[0], ".changed").unwrap());
    owner.commit(edit).unwrap();
    assert_eq!(owner.css_style_rule(&rules[0]).unwrap(), parent);
    assert!(identities.contains(&owner.css_style_rule(&rules[0]).unwrap()));
    assert_ne!(
        parent
            .css_text(&owner, &CssomRuleFormatLimits::default())
            .unwrap(),
        before
    );
    assert_eq!(
        captured
            .parent_rule()
            .unwrap()
            .unwrap()
            .css_text(&CssomRuleFormatLimits::default())
            .unwrap(),
        before
    );
    assert_eq!(
        captured.parent_style_sheet().unwrap().unwrap().handle(),
        &owner.css_style_sheet(&sheet).unwrap()
    );
    let mut edit = batch(&owner);
    edit.delete_sheet_rule(&sheet, 0).unwrap();
    owner.commit(edit).unwrap();
    assert!(parent.parent_rule(&owner).unwrap().is_none());
    assert!(parent.parent_style_sheet(&owner).unwrap().is_none());
    assert_eq!(
        child.parent_rule(&owner).unwrap().unwrap(),
        parent.as_rule()
    );
    assert!(child.parent_style_sheet(&owner).unwrap().is_none());
    assert_eq!(captured.snapshot().revision(), original.revision());
    drop(owner);
    let parent = captured
        .parent_rule()
        .unwrap()
        .unwrap()
        .as_style_rule()
        .unwrap()
        .unwrap();
    assert_eq!(
        parent.css_text(&CssomRuleFormatLimits::default()).unwrap(),
        before
    );
    assert_eq!(parent.snapshot().revision(), original.revision());
    assert!(parent.parent_style_sheet().unwrap().is_some());
    assert_eq!(
        parent.as_grouping_rule().as_rule().handle(),
        parent.as_rule().handle()
    );
}

#[test]
fn foreign_missing_and_wrong_kind_are_distinct_checked_results() {
    let mut first = store();
    let old = first.snapshot();
    let (_, rules) = create(&mut first, ".x {} @media print {}");
    let rule = first.css_rule(&rules[0]).unwrap();
    assert!(rule.as_media_rule(&first).unwrap().is_none());
    assert!(matches!(
        first.css_media_rule(&rules[0]),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(rule.capture(&old), Err(CssomError::MissingObject)));
    let mut other = store();
    let (_, other_rules) = create(&mut other, ".x {}");
    assert_ne!(rule, other.css_rule(&other_rules[0]).unwrap());
    assert!(matches!(
        other.css_rule(&rules[0]),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(
        rule.as_style_rule(&other),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(
        rule.capture(&other.snapshot()),
        Err(CssomError::ForeignOwner)
    ));
    assert!(matches!(rule.r#type(&other), Err(CssomError::ForeignOwner)));
    let captured = rule.capture(&first.snapshot()).unwrap();
    assert!(captured.as_media_rule().unwrap().is_none());
    assert_eq!(
        captured
            .as_style_rule()
            .unwrap()
            .unwrap()
            .as_rule()
            .handle(),
        &rule
    );
}

#[test]
fn grouping_is_capability_not_numeric_type_or_list_presence() {
    let mut owner = store();
    let (_, rules) = create(
        &mut owner,
        "@keyframes spin { from { opacity: 0; } } @container box (width > 1px) {} @supports-condition --test {} @when media(print) {}",
    );
    assert_eq!(rules.len(), 4);
    let keyframes = owner.css_keyframes_rule(&rules[0]).unwrap();
    assert_eq!(keyframes.r#type(&owner).unwrap(), CssRule::KEYFRAMES_RULE);
    assert!(
        keyframes
            .as_rule()
            .as_grouping_rule(&owner)
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        owner.css_grouping_rule(&rules[0]),
        Err(CssomError::WrongKind)
    ));
    let snapshot = owner.snapshot();
    let child = &snapshot
        .rules(snapshot.child_rules(&rules[0]).unwrap())
        .unwrap()[0];
    assert_eq!(
        owner
            .css_keyframe_rule(child)
            .unwrap()
            .r#type(&owner)
            .unwrap(),
        CssRule::KEYFRAME_RULE
    );
    let container = owner.css_container_rule(&rules[1]).unwrap();
    assert_eq!(container.r#type(&owner).unwrap(), 0);
    assert_eq!(
        container.as_condition_rule().as_grouping_rule().as_rule(),
        container.as_rule()
    );
    let named = owner.css_supports_condition_rule(&rules[2]).unwrap();
    assert_eq!(named.r#type(&owner).unwrap(), 0);
    assert!(named.as_rule().as_grouping_rule(&owner).unwrap().is_some());
    assert!(named.as_rule().as_condition_rule(&owner).unwrap().is_none());
    assert!(owner.css_grouping_rule(&rules[3]).is_ok());
}

#[test]
fn declaration_factories_validate_actual_domains_without_counter_style_alias() {
    let mut owner = store();
    let (_, rules) = create(
        &mut owner,
        ".x { color: red; } @page { size: a4; @top-left { color: blue; } } @font-face { font-family: Example; src: url(font.woff); } @keyframes k { to { color: red; } } @counter-style c { system: cyclic; symbols: x; }",
    );
    assert_eq!(rules.len(), 5);
    let snapshot = owner.snapshot();
    let block = |rule: &CssomRuleId| snapshot.rule_style(rule).unwrap();
    assert!(owner.css_style_rule(&rules[0]).is_ok());
    assert!(owner.css_page_rule(&rules[1]).is_ok());
    assert!(owner.css_font_face_rule(&rules[2]).is_ok());
    let style = owner.css_style_properties(block(&rules[0])).unwrap();
    assert_eq!(
        style
            .as_style_declaration()
            .as_style_properties(&owner)
            .unwrap()
            .unwrap(),
        style
    );
    assert!(matches!(
        owner.css_page_descriptors(block(&rules[0])),
        Err(CssomError::WrongKind)
    ));
    let page = owner.css_page_descriptors(block(&rules[1])).unwrap();
    assert!(
        page.as_style_declaration()
            .as_style_properties(&owner)
            .unwrap()
            .is_none()
    );
    let margin = &snapshot
        .rules(snapshot.child_rules(&rules[1]).unwrap())
        .unwrap()[0];
    assert!(owner.css_margin_rule(margin).is_ok());
    let margin = owner.css_margin_descriptors(block(margin)).unwrap();
    assert!(
        margin
            .as_style_declaration()
            .as_margin_descriptors(&owner)
            .unwrap()
            .is_some()
    );
    let face = owner.css_font_face_descriptors(block(&rules[2])).unwrap();
    assert_eq!(
        face.capture(&snapshot)
            .unwrap()
            .as_style_declaration()
            .as_font_face_descriptors()
            .unwrap()
            .unwrap()
            .handle(),
        &face
    );
    let keyframe = &snapshot
        .rules(snapshot.child_rules(&rules[3]).unwrap())
        .unwrap()[0];
    assert!(owner.css_style_declaration(block(keyframe)).is_ok());
    assert!(matches!(
        owner.css_style_properties(block(keyframe)),
        Err(CssomError::WrongKind)
    ));
    assert!(matches!(
        owner.css_style_declaration(block(&rules[4])),
        Err(CssomError::WrongKind)
    ));
    assert_eq!(
        owner
            .css_counter_style_rule(&rules[4])
            .unwrap()
            .r#type(&owner)
            .unwrap(),
        CssRule::COUNTER_STYLE_RULE
    );
}

#[test]
fn collection_and_sheet_shells_check_existing_identity_and_share_captures() {
    let mut owner = store();
    let original = owner.snapshot();
    let sheet_list = owner.style_sheet_list(original.sheet_list()).unwrap();
    assert!(sheet_list.capture(&original).is_ok());
    let (sheet, _) = create(&mut owner, ".x {}");
    let snapshot = owner.snapshot();
    let typed = owner.css_style_sheet(&sheet).unwrap();
    assert_eq!(typed.as_style_sheet(), owner.style_sheet(&sheet).unwrap());
    assert_eq!(
        typed
            .capture(&snapshot)
            .unwrap()
            .as_style_sheet()
            .as_css_style_sheet()
            .unwrap()
            .unwrap()
            .handle(),
        &typed
    );
    let list = owner
        .css_rule_list(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap();
    assert!(matches!(
        list.capture(&original),
        Err(CssomError::MissingObject)
    ));
    let media = owner
        .media_list(snapshot.sheet(&sheet).unwrap().media())
        .unwrap();
    assert!(media.capture(&snapshot).is_ok());
    assert!(matches!(
        media.capture(&original),
        Err(CssomError::MissingObject)
    ));
    assert_eq!(
        owner.style_sheet_list(snapshot.sheet_list()).unwrap(),
        sheet_list
    );
    let other = store();
    assert!(matches!(
        sheet_list.capture(&other.snapshot()),
        Err(CssomError::ForeignOwner)
    ));
}

#[test]
fn css_namespace_forwards_native_unicode_replacement_and_typed_limits() {
    use surgeist_css::{
        CssSpecifiedValueSerializationLimits, serialize_css_identifier_with_limits,
    };
    assert_eq!(css::escape("0a\0é").unwrap(), "\\30 a�é");
    assert_eq!(css::escape("").unwrap(), "");
    let limits = CssSpecifiedValueSerializationLimits::new(0, 10, 10);
    assert_eq!(
        css::escape_with_limits("é", limits),
        serialize_css_identifier_with_limits("é", limits)
    );
    assert!(css::escape_with_limits("é", limits).is_err());
    assert_eq!(
        css::escape_with_limits("é", CssSpecifiedValueSerializationLimits::default()).unwrap(),
        "é"
    );
}

#[test]
fn historical_constants_and_actual_fields_support_send_sync() {
    assert_eq!(
        [
            CssRule::STYLE_RULE,
            CssRule::CHARSET_RULE,
            CssRule::IMPORT_RULE,
            CssRule::MEDIA_RULE,
            CssRule::FONT_FACE_RULE,
            CssRule::PAGE_RULE,
            CssRule::KEYFRAMES_RULE,
            CssRule::KEYFRAME_RULE,
            CssRule::MARGIN_RULE,
            CssRule::NAMESPACE_RULE,
            CssRule::COUNTER_STYLE_RULE,
            CssRule::SUPPORTS_RULE,
            CssRule::FONT_FEATURE_VALUES_RULE
        ],
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14]
    );
    assert_eq!(CssStyleRule::CHARSET_RULE, CssRule::CHARSET_RULE);
    fn send_sync<T: Send + Sync>() {}
    send_sync::<CssStyleRule>();
    send_sync::<CssomCapture<CssStyleRule>>();
    send_sync::<CssStyleSheet>();
    send_sync::<CssomCapture<CssFontFaceDescriptors>>();
}

#[test]
fn every_specialized_rule_shell_checks_the_current_concrete_payload() {
    let mut owner = store();
    let (_, rules) = create(
        &mut owner,
        "@import url(sheet.css); @namespace n url(urn:n); @layer a,b; @media print {} @supports (display:grid) {} @layer a {} @scope {} @font-feature-values Example { @annotation { example:1; } } @font-palette-values --p { font-family:Example; } @custom-media --m print; @color-profile --p { components:A; }",
    );
    assert_eq!(rules.len(), 11);
    let snapshot = owner.snapshot();
    macro_rules! check {
        ($index:expr, $factory:ident, $downcast:ident) => {{
            let specific = owner.$factory(&rules[$index]).unwrap();
            let base = specific.as_rule();
            assert_eq!(base.id(), &rules[$index]);
            assert_eq!(base.$downcast(&owner).unwrap().unwrap(), specific);
            let capture = base
                .capture(&snapshot)
                .unwrap()
                .$downcast()
                .unwrap()
                .unwrap();
            assert_eq!(capture.handle(), &specific);
            assert_eq!(capture.snapshot().revision(), snapshot.revision());
            assert!(matches!(
                owner.$factory(&rules[($index + 1) % rules.len()]),
                Err(CssomError::WrongKind)
            ));
        }};
    }
    check!(0, css_import_rule, as_import_rule);
    check!(1, css_namespace_rule, as_namespace_rule);
    check!(2, css_layer_statement_rule, as_layer_statement_rule);
    check!(3, css_media_rule, as_media_rule);
    check!(4, css_supports_rule, as_supports_rule);
    check!(5, css_layer_block_rule, as_layer_block_rule);
    check!(6, css_scope_rule, as_scope_rule);
    check!(7, css_font_feature_values_rule, as_font_feature_values_rule);
    check!(8, css_font_palette_values_rule, as_font_palette_values_rule);
    check!(9, css_custom_media_rule, as_custom_media_rule);
    check!(10, css_color_profile_rule, as_color_profile_rule);
    let CssomRuleData::FontFeatureValues { maps, .. } = snapshot.rule(&rules[7]).unwrap().data()
    else {
        panic!()
    };
    assert!(!maps.is_empty());
    let map = owner.css_font_feature_values_map(&maps[0]).unwrap();
    assert_eq!(map.id(), &maps[0]);
    assert!(map.capture(&snapshot).is_ok());
    assert!(matches!(
        map.capture(&store().snapshot()),
        Err(CssomError::ForeignOwner)
    ));
}

#[test]
fn scoped_leaf_and_nested_declaration_shells_preserve_their_actual_kinds() {
    let mut owner = store();
    let (_, roots) = create(
        &mut owner,
        "@scope { @layer scoped; @font-palette-values --p {font-family:A;} @color-profile --c {components:A;} } .x { & .child {} color:red; }",
    );
    let snapshot = owner.snapshot();
    let scoped = snapshot
        .rules(snapshot.child_rules(&roots[0]).unwrap())
        .unwrap();
    assert_eq!(scoped.len(), 3);
    assert!(owner.css_layer_statement_rule(&scoped[0]).is_ok());
    assert!(owner.css_font_palette_values_rule(&scoped[1]).is_ok());
    assert!(owner.css_color_profile_rule(&scoped[2]).is_ok());
    let children = snapshot
        .rules(snapshot.child_rules(&roots[1]).unwrap())
        .unwrap();
    assert_eq!(children.len(), 2);
    let nested = owner.css_nested_declarations(&children[1]).unwrap();
    assert_eq!(nested.r#type(&owner).unwrap(), 0);
    assert!(nested.as_rule().as_grouping_rule(&owner).unwrap().is_none());
    let block = snapshot.rule_style(nested.id()).unwrap();
    assert!(owner.css_style_properties(block).is_ok());
    assert_eq!(nested.parent_rule(&owner).unwrap().unwrap().id(), &roots[1]);
}
