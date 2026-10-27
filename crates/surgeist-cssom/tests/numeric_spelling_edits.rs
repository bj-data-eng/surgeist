//! Public consumer expectations for edits whose new text differs from the old
//! text only in numeric spelling (`2px` / `2.0px` / `+2px` / `1e1px` / `10px`).
//!
//! Authority: CSSOM Editor's Draft (31 August 2026, `references/cssom-1--editor-capture-20261009--d42e145ec395.md`)
//! - §6.1.2 "synchronously replace the rules of a CSSStyleSheet": parse the text
//!   and set the sheet's CSS rules to the result, so every replacement yields
//!   new CSSRule objects regardless of text equality.
//! - §6.6 `cssText` setter: empty the declarations, reinsert the parsed ones,
//!   then "update style attribute for" the block (unconditionally).
//! - §6.6 `setProperty`: update the style attribute only when "updated" is true;
//!   "set a CSS declaration" must return true when the serialization changed.
//! - §6.7.2 "serialize a CSS component value": a `<number>` is serialized in the
//!   shortest base-ten form without scientific notation, so every spelling of
//!   one value reads back identically (`2.0px` and `1e1px` read as `2px` and
//!   `10px`).
//!
//! Numeric spelling is therefore never CSSOM-observable through text getters,
//! while object identity, revisions and owner updates follow the algorithms above.
use surgeist_css::CssParser;
use surgeist_css::component::CssValueOrigin;
use surgeist_css::declaration::CssSpecifiedDeclarationEntry;
use surgeist_css::property::CssPropertyGrammar;
use surgeist_css::rule::CssRule;
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_cssom::*;

fn version(role: CssomInputRole, identity: &str, revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: identity.into(),
        revision,
    }
}
fn store() -> CssomStore {
    CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap()
}
fn block_store() -> CssomStore {
    let profile = CssomDeclarationSupport::try_new(vec![(
        CssPropertyGrammar::from_name("width").unwrap(),
        CssomUsabilityRequirement::AllCheckedValues,
    )])
    .unwrap();
    CssomStore::new(
        CssomLimits::default(),
        CssomContext {
            inputs: vec![CssomInput {
                version: version(CssomInputRole::Support, "renderer", 1),
                data: CssomInputData::Support(profile),
            }],
            ..CssomContext::default()
        },
    )
    .unwrap()
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn constructed(store: &mut CssomStore, text: &str) -> CssomSheetId {
    let mut edit = batch(store);
    let construction = edit
        .construct_sheet(
            CssomConstructorDocument {
                version: version(CssomInputRole::Document, "d", 1),
                identity: "d".into(),
                base_location: String::new(),
            },
            CssomSheetInit::default(),
        )
        .unwrap();
    let sheet = store
        .commit(edit)
        .unwrap()
        .sheet(construction.ticket())
        .unwrap()
        .clone();
    if !text.is_empty() {
        let mut edit = batch(store);
        edit.replace_sheet_sync(&sheet, text).unwrap();
        store.commit(edit).unwrap();
    }
    sheet
}
fn roots(store: &CssomStore, sheet: &CssomSheetId) -> Vec<CssomRuleId> {
    let snapshot = store.snapshot();
    snapshot
        .rules(snapshot.sheet(sheet).unwrap().rules())
        .unwrap()
        .to_vec()
}
fn rule_text(store: &CssomStore, snapshot: &CssomSnapshot, id: &CssomRuleId) -> String {
    store
        .css_rule(id)
        .unwrap()
        .capture(snapshot)
        .unwrap()
        .css_text(&CssomRuleFormatLimits::default())
        .unwrap()
}
fn style_value(
    store: &CssomStore,
    snapshot: &CssomSnapshot,
    rule: &CssomRuleId,
    property: &str,
) -> String {
    store
        .css_style_rule(rule)
        .unwrap()
        .capture(snapshot)
        .unwrap()
        .style()
        .unwrap()
        .get_property_value(property, Default::default())
        .unwrap()
}
fn changed(commit: &CssomCommit) -> &CssomChangeSummary {
    match commit.publication() {
        CssomPublication::Changed(summary) => summary,
        CssomPublication::Unchanged { revision } => {
            panic!("expected a published change, got Unchanged at {revision:?}")
        }
    }
}

/// (old spelling, new spelling, canonical serialization of both)
const SPELLINGS: [(&str, &str, &str); 5] = [
    ("2px", "2.0px", "2px"),
    ("10px", "1e1px", "10px"),
    ("0.5px", ".5px", "0.5px"),
    ("2px", "+2px", "2px"),
    ("2px", "2px", "2px"),
];

#[test]
fn replace_sync_with_numeric_spelling_only_change_publishes_fresh_rules_with_canonical_text() {
    for (old, new, canonical) in SPELLINGS {
        let mut store = store();
        let sheet = constructed(&mut store, &format!("a {{ width: {old} }}"));
        let before = store.snapshot();
        let old_rule = roots(&store, &sheet)[0].clone();
        let expected = format!("a {{ width: {canonical}; }}");
        assert_eq!(rule_text(&store, &before, &old_rule), expected, "{old}");

        let mut edit = batch(&store);
        edit.replace_sheet_sync(&sheet, &format!("a {{ width: {new} }}"))
            .unwrap();
        let commit = store.commit(edit).unwrap();
        let summary = changed(&commit);
        let after = store.snapshot();
        assert!(
            after.revision().value() > before.revision().value(),
            "{old} -> {new}"
        );
        assert_eq!(summary.to(), after.revision());
        assert!(
            matches!(
                store.changes_since(&before),
                CssomChanges::FullRecompute(CssomRecomputeReason::ConservativeChange)
            ),
            "{old} -> {new}: {:?}",
            store.changes_since(&before)
        );

        // New CSSRule object; the replaced one leaves the sheet.
        let new_rules = roots(&store, &sheet);
        assert_eq!(new_rules.len(), 1);
        assert_ne!(new_rules[0], old_rule, "{old} -> {new}");
        assert_eq!(
            after.rule(&old_rule).unwrap().parent(),
            &CssomParent::Detached
        );
        assert!(
            store
                .css_rule(&old_rule)
                .unwrap()
                .capture(&after)
                .unwrap()
                .parent_style_sheet()
                .unwrap()
                .is_none()
        );

        // CSSOM serialization of the value is spelling independent.
        assert_eq!(
            rule_text(&store, &after, &new_rules[0]),
            expected,
            "{old} -> {new}"
        );
        assert_eq!(
            style_value(&store, &after, &new_rules[0], "width"),
            canonical
        );
        // The earlier capture is untouched.
        assert_eq!(rule_text(&store, &before, &old_rule), expected);
    }
}

#[test]
fn replace_sync_numeric_spelling_change_inside_a_group_replaces_the_whole_rule_tree() {
    let mut store = store();
    let sheet = constructed(&mut store, "@media (min-width: 10px) { a { width: 10px } }");
    let before = store.snapshot();
    let old_group = roots(&store, &sheet)[0].clone();
    let old_child = children(&before, &old_group)[0].clone();
    let expected = rule_text(&store, &before, &old_group);

    let mut edit = batch(&store);
    edit.replace_sheet_sync(&sheet, "@media (min-width: 10px) { a { width: 1e1px } }")
        .unwrap();
    changed(&store.commit(edit).unwrap());
    let after = store.snapshot();
    let new_group = roots(&store, &sheet)[0].clone();
    let new_child = children(&after, &new_group)[0].clone();
    assert_ne!(new_group, old_group);
    assert_ne!(new_child, old_child);
    assert_eq!(rule_text(&store, &after, &new_group), expected);
    assert_eq!(style_value(&store, &after, &new_child, "width"), "10px");
}

fn children(snapshot: &CssomSnapshot, id: &CssomRuleId) -> Vec<CssomRuleId> {
    snapshot
        .rules(
            snapshot
                .rule(id)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                .unwrap(),
        )
        .unwrap()
        .to_vec()
}
fn create_block(store: &mut CssomStore, text: &str) -> CssomBlockId {
    let mut edit = batch(store);
    let values = CssParser::default()
        .parse_block_contents(text)
        .into_parts()
        .0;
    let ticket = edit
        .create_declarations(
            values,
            false,
            false,
            Some(version(CssomInputRole::Owner, "element", 1)),
        )
        .unwrap();
    store.commit(edit).unwrap().block(&ticket).unwrap().clone()
}
fn block_view(
    store: &CssomStore,
    snapshot: &CssomSnapshot,
    id: &CssomBlockId,
) -> CssomCapture<CssStyleDeclaration> {
    store
        .css_style_declaration(id)
        .unwrap()
        .capture(snapshot)
        .unwrap()
}
fn selected<'a>(
    snapshot: &'a CssomSnapshot,
    id: &CssomBlockId,
) -> &'a [CssSpecifiedDeclarationEntry] {
    match snapshot.block(id).unwrap().data() {
        CssomBlockData::Properties {
            selected: CssomProjection::Available(v),
            ..
        } => v.entries(),
        other => panic!("properties: {other:?}"),
    }
}

#[test]
fn declaration_css_text_with_numeric_spelling_only_change_updates_owner_and_publishes() {
    for (old, new, canonical) in SPELLINGS {
        let mut store = block_store();
        let id = create_block(&mut store, &format!("width:{old}"));
        let before = store.snapshot();
        let mut edit = batch(&store);
        let prepared = edit
            .prepare_css_text(
                &id,
                &format!("width:{new}"),
                CssomDeclarationRequestLimits::default(),
            )
            .unwrap();
        edit.apply_declaration(prepared, &[]).unwrap();
        let commit = store.commit(edit).unwrap();
        changed(&commit);
        // The cssText setter updates the style attribute unconditionally.
        assert!(!commit.owner_effects().is_empty(), "{old} -> {new}");
        let after = store.snapshot();
        assert!(after.revision().value() > before.revision().value());
        let read = block_view(&store, &after, &id);
        assert_eq!(
            read.css_text(Default::default()).unwrap(),
            format!("width: {canonical};")
        );
        assert_eq!(
            read.get_property_value("width", Default::default())
                .unwrap(),
            canonical
        );
        assert!(
            !selected(&after, &id)[0]
                .source()
                .same_occurrence(selected(&before, &id)[0].source())
        );
    }
}

#[test]
fn set_property_with_numeric_spelling_only_change_is_not_updated_but_records_fresh_provenance() {
    for (old, new, canonical) in SPELLINGS {
        let mut store = block_store();
        let id = create_block(&mut store, &format!("width:{old}"));
        let before = store.snapshot();
        let mut edit = batch(&store);
        let prepared = edit
            .prepare_set_property(
                &id,
                "width",
                new,
                "",
                CssomDeclarationRequestLimits::default(),
            )
            .unwrap();
        // The serialization of the declarations is unchanged, so "set a CSS
        // declaration" may return false; this store does.
        assert_eq!(
            edit.apply_declaration(prepared, &[]).unwrap(),
            CssomDeclarationEditResult::Set { updated: false },
            "{old} -> {new}"
        );
        let commit = store.commit(edit).unwrap();
        assert!(commit.owner_effects().is_empty(), "{old} -> {new}");
        changed(&commit);
        let after = store.snapshot();
        assert_eq!(
            block_view(&store, &after, &id)
                .get_property_value("width", Default::default())
                .unwrap(),
            canonical
        );
        let source = selected(&after, &id)[0].source();
        assert!(!source.same_occurrence(selected(&before, &id)[0].source()));
        assert_eq!(source.parsed_value().unwrap().source().as_str(), new);
    }
}

#[test]
fn set_property_with_changed_numeric_value_reports_updated_and_updates_owner() {
    let mut store = block_store();
    let id = create_block(&mut store, "width:2px");
    let mut edit = batch(&store);
    let prepared = edit
        .prepare_set_property(
            &id,
            "width",
            "3.0px",
            "",
            CssomDeclarationRequestLimits::default(),
        )
        .unwrap();
    assert_eq!(
        edit.apply_declaration(prepared, &[]).unwrap(),
        CssomDeclarationEditResult::Set { updated: true }
    );
    let commit = store.commit(edit).unwrap();
    changed(&commit);
    assert!(!commit.owner_effects().is_empty());
    assert_eq!(
        block_view(&store, &store.snapshot(), &id)
            .get_property_value("width", Default::default())
            .unwrap(),
        "3px"
    );
}

// §4.2.1 leaves the width serialization as "..." in the pinned edition, so the
// text of a media feature value is CSS-owned policy and is not asserted here;
// only that the setter does not suppress the edit is.
#[test]
fn media_text_setter_with_numeric_spelling_only_change_is_still_published() {
    for (old, new) in [("2px", "2.0px"), ("10px", "1e1px")] {
        let mut store = store();
        let sheet = constructed(&mut store, "");
        let id = store.snapshot().sheet(&sheet).unwrap().media().clone();
        let mut edit = batch(&store);
        edit.set_media_text(&id, &format!("(min-width: {old})"))
            .unwrap();
        store.commit(edit).unwrap();
        let before = store.snapshot();
        let mut edit = batch(&store);
        let result = edit
            .set_media_text(&id, &format!("(min-width: {new})"))
            .unwrap();
        assert!(result.changed(), "{old} -> {new}");
        let commit = store.commit(edit).unwrap();
        changed(&commit);
        assert!(store.snapshot().revision().value() > before.revision().value());
    }
}

#[test]
fn replace_sync_of_import_only_text_removes_imports_and_retains_current_source_provenance() {
    // §6.1.2 step 3 removes @import rules, so the rule list is empty before and
    // after; the retained authored sheet is the only place the spelling lives.
    let old = "@import 'a.css' (min-width: 2px);";
    let new = "@import 'a.css' (min-width: 2.0px);";
    let mut store = store();
    let sheet = constructed(&mut store, old);
    let before = store.snapshot();
    assert!(roots(&store, &sheet).is_empty());

    // Identical text changes nothing observable and publishes nothing.
    let mut edit = batch(&store);
    edit.replace_sheet_sync(&sheet, old).unwrap();
    assert!(matches!(
        store.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), before.revision());

    // Spelling-only change: still no rules, but the retained source is current.
    let mut edit = batch(&store);
    edit.replace_sheet_sync(&sheet, new).unwrap();
    changed(&store.commit(edit).unwrap());
    let after = store.snapshot();
    assert!(roots(&store, &sheet).is_empty());
    let authored = after.sheet(&sheet).unwrap().authored();
    assert!(matches!(&authored.rules()[0], CssRule::Import(rule)
            if matches!(rule.origin(), CssValueOrigin::Parsed(origin) if origin.source().as_str() == new)));
    let previous = before.sheet(&sheet).unwrap().authored();
    assert!(matches!(&previous.rules()[0], CssRule::Import(rule)
            if matches!(rule.origin(), CssValueOrigin::Parsed(origin) if origin.source().as_str() == old)));
}

#[test]
fn keyframe_key_text_with_numeric_spelling_only_change_is_published_and_reads_back_canonically() {
    let mut store = store();
    let sheet = constructed(&mut store, "@keyframes k { 50% { width: 1px } }");
    let group = roots(&store, &sheet)[0].clone();
    let frame = children(&store.snapshot(), &group)[0].clone();
    let mut edit = batch(&store);
    edit.set_keyframe_key_text(&frame, "50.0%").unwrap();
    let commit = store.commit(edit).unwrap();
    changed(&commit);
    let key = store
        .css_keyframe_rule(&frame)
        .unwrap()
        .capture(&store.snapshot())
        .unwrap()
        .key_text(CssSerializationLimits::default())
        .unwrap();
    assert_eq!(key, "50%");
}
