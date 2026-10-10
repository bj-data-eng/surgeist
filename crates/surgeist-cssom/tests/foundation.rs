use surgeist_css::*;
use surgeist_cssom::*;

fn version(role: CssomInputRole, identity: &str, revision: u64) -> CssomInputVersion {
    CssomInputVersion {
        role,
        identity: identity.to_owned(),
        revision,
    }
}
fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::constructed(
        version(CssomInputRole::Document, "document", 1),
        "document".into(),
        None,
    )
}
fn new_store() -> CssomStore {
    CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap()
}
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn create(store: &mut CssomStore, text: &str) -> CssomSheetId {
    let mut edit = batch(store);
    let ticket = edit.create_parsed_sheet(text, inputs()).unwrap();
    store.commit(edit).unwrap().sheet(&ticket).unwrap().clone()
}
fn root(snapshot: &CssomSnapshot, sheet: &CssomSheetId) -> CssomRuleId {
    snapshot
        .rules(snapshot.sheet(sheet).unwrap().rules())
        .unwrap()[0]
        .clone()
}
fn block(snapshot: &CssomSnapshot, rule: &CssomRuleId) -> CssomBlockId {
    snapshot.rule(rule).unwrap().data().block().unwrap().clone()
}
fn children(snapshot: &CssomSnapshot, rule: &CssomRuleId) -> Vec<CssomRuleId> {
    snapshot
        .rules(snapshot.rule(rule).unwrap().data().children().unwrap())
        .unwrap()
        .to_vec()
}
fn selected<'a>(
    snapshot: &'a CssomSnapshot,
    id: &CssomBlockId,
) -> &'a CssSpecifiedDeclarationBlock {
    match snapshot.block(id).unwrap().data() {
        CssomBlockData::Properties {
            selected: CssomProjection::Available(v),
            ..
        } => v,
        other => panic!("expected available properties: {other:?}"),
    }
}

#[test]
fn owning_snapshot_retains_origin_and_selected_shorthand_survivors_after_owner_drop() {
    let mut store = new_store();
    let text = "a { margin: 1px 2px 3px 4px !important; --x: var(--input); }";
    let sheet = create(&mut store, text);
    let before = store.snapshot();
    let rule = root(&before, &sheet);
    let block = block(&before, &rule);
    let original = selected(&before, &block).entries()[0].source().clone();
    assert_eq!(original.parsed_value().unwrap().source().as_str(), text);
    let mut edit = batch(&store);
    assert!(
        edit.remove_selected_terminal(
            &block,
            CssPropertyNameRef::Known(CssKnownProperty::MarginTop)
        )
        .unwrap()
    );
    let commit = store.commit(edit).unwrap();
    let after = store.snapshot();
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("change expected")
    };
    assert!(summary.supports_incremental(&before, &after));
    assert_eq!(
        selected(&after, &block).entries()[..3]
            .iter()
            .map(|v| v.property_name())
            .collect::<Vec<_>>(),
        [
            CssPropertyNameRef::Known(CssKnownProperty::MarginRight),
            CssPropertyNameRef::Known(CssKnownProperty::MarginBottom),
            CssPropertyNameRef::Known(CssKnownProperty::MarginLeft)
        ]
    );
    assert_eq!(
        selected(&after, &block).entries()[3]
            .source()
            .custom()
            .unwrap()
            .name()
            .as_str(),
        "--x"
    );
    for entry in &selected(&after, &block).entries()[..3] {
        assert_eq!(entry.importance(), CssImportance::Important);
        assert!(entry.source().same_occurrence(&original));
    }
    assert_eq!(
        after
            .specified_property_css_text(&block, CssSpecifiedValueSerializationLimits::default())
            .unwrap(),
        "margin-right: 2px !important; margin-bottom: 3px !important; margin-left: 4px !important; --x: var(--input);"
    );
    drop(store);
    assert_eq!(selected(&before, &block).entries().len(), 5);
    assert_eq!(selected(&after, &block).entries().len(), 4);
    assert_eq!(after.parent_style_sheet(&rule).unwrap(), Some(&sheet));
}

#[test]
fn detach_and_replacement_preserve_collections_and_internal_parent_chains() {
    let mut store = new_store();
    let sheet = create(
        &mut store,
        "@media screen { @media print { a { color: red; } } }",
    );
    let before = store.snapshot();
    let outer = root(&before, &sheet);
    let inner = children(&before, &outer)[0].clone();
    let leaf = children(&before, &inner)[0].clone();
    let sheet_list = before.sheet(&sheet).unwrap().rules().clone();
    let media = before.sheet(&sheet).unwrap().media().clone();
    let leaf_block = block(&before, &leaf);
    let mut edit = batch(&store);
    assert_eq!(edit.delete_sheet_rule(&sheet, 0).unwrap(), outer);
    store.commit(edit).unwrap();
    let detached = store.snapshot();
    assert_eq!(
        detached.rule(&outer).unwrap().parent(),
        &CssomParent::Detached
    );
    assert_eq!(detached.parent_rule(&leaf).unwrap(), Some(&inner));
    assert_eq!(detached.parent_rule(&inner).unwrap(), Some(&outer));
    assert_eq!(detached.parent_style_sheet(&leaf).unwrap(), None);
    assert!(detached.block(&leaf_block).is_ok());
    let mut edit = batch(&store);
    edit.replace_sheet_text(&sheet, "a { color: red; }")
        .unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(after.sheet(&sheet).unwrap().rules(), &sheet_list);
    assert_eq!(after.sheet(&sheet).unwrap().media(), &media);
    assert_ne!(root(&after, &sheet), leaf);
    assert_eq!(after.parent_style_sheet(&leaf).unwrap(), None);
    assert_eq!(before.parent_style_sheet(&leaf).unwrap(), Some(&sheet));
}

#[test]
fn empty_nested_child_remains_live_and_css_provider_filters_only_its_text() {
    let mut store = new_store();
    let sheet = create(
        &mut store,
        "@media screen { a { & b { color: red; } color: blue; & c { color: green; } } }",
    );
    let before = store.snapshot();
    let media = root(&before, &sheet);
    let style = children(&before, &media)[0].clone();
    let order = children(&before, &style);
    assert_eq!(order.len(), 3);
    let nested = order[1].clone();
    assert!(matches!(
        before.rule(&nested).unwrap().data(),
        CssomRuleData::NestedDeclarations { .. }
    ));
    let id = block(&before, &nested);
    let mut edit = batch(&store);
    edit.remove_selected_terminal(&id, CssPropertyNameRef::Known(CssKnownProperty::Color))
        .unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(children(&after, &style), order);
    assert_eq!(block(&after, &nested), id);
    assert!(selected(&after, &id).entries().is_empty());
    assert_eq!(after.parent_rule(&nested).unwrap(), Some(&style));
    assert_eq!(selected(&before, &id).entries().len(), 1);
    let views = order
        .iter()
        .map(|id| match after.rule(id).unwrap().data() {
            CssomRuleData::NestedDeclarations { block } => {
                CssEditedRuleView::try_nested_declarations(selected(&after, block)).unwrap()
            }
            _ => match after.rule(id).unwrap().authored() {
                CssomAuthoredRule::Ordinary(v) => CssEditedRuleView::from_rule(v),
                _ => panic!("ordinary child"),
            },
        })
        .collect::<Vec<_>>();
    let CssomRuleData::Style {
        selectors: CssomSelectors::Ordinary(selectors),
        block,
        ..
    } = after.rule(&style).unwrap().data()
    else {
        panic!("style")
    };
    let view = CssEditedRuleView::try_style(selectors, selected(&after, block), &views).unwrap();
    assert_eq!(
        view.serialize_cssom().unwrap(),
        "a {\n  & b { color: red; }\n  & c { color: green; }\n}"
    );
    let media_id = match after.rule(&media).unwrap().data() {
        CssomRuleData::Group {
            prelude: CssomGroupPrelude::Media(id),
            ..
        } => id,
        _ => panic!("media"),
    };
    let child_views = [view];
    let media_view = CssEditedRuleView::try_group(
        CssEditedGroupPreludeRef::Media(after.media(media_id).unwrap()),
        &child_views,
    )
    .unwrap();
    // Selected CSSOM §serialize-rule indents each serialized child item once.
    assert_eq!(
        media_view.serialize_cssom().unwrap(),
        "@media screen {\n  a {\n  & b { color: red; }\n  & c { color: green; }\n}\n}"
    );
}

#[test]
fn raw_counter_names_and_all_seven_maps_do_not_weaken_authored_grammar() {
    let mut store = new_store();
    let sheet = create(
        &mut store,
        "@counter-style custom { system: cyclic; symbols: 'x'; } @font-feature-values Demo { @styleset { huge: 4294967296; } }",
    );
    let before = store.snapshot();
    let rules = before.sheet_rules(&sheet).unwrap();
    assert_eq!(rules.len(), 2);
    let counter = rules[0].clone();
    let maps = match before.rule(&rules[1]).unwrap().data() {
        CssomRuleData::FontFeatureValues { maps, .. } => maps.clone(),
        _ => panic!("maps"),
    };
    assert_eq!(maps.len(), 7);
    let styleset = maps
        .iter()
        .find(|id| before.feature_map(id).unwrap().kind() == CssFontFeatureValueKind::Styleset)
        .unwrap();
    assert!(matches!(
        before.feature_map(styleset).unwrap().get("huge"),
        Err(CssomError::AuthoredFeatureConversion)
    ));
    let CssomFeatureValue::Authored(value) =
        &before.feature_map(styleset).unwrap().entries()[0].value
    else {
        panic!("exact authored")
    };
    let CssFontFeatureValueRef::Indexes(exact) = value.view() else {
        panic!("indexes")
    };
    assert_eq!(exact[0].as_decimal_str(), "4294967296");
    assert!(exact[0].origin().is_some());
    let mut edit = batch(&store);
    assert!(!edit.set_counter_name(&counter, "DeCiMaL").unwrap());
    assert!(edit.set_counter_name(&counter, "").unwrap());
    for id in &maps {
        assert!(edit.set_feature(id, "", &[]).unwrap());
        assert!(edit.set_feature(id, "A!?", &[0]).unwrap());
    }
    edit.set_feature(styleset, "huge", &[u32::MAX]).unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert!(
        matches!(after.rule(&counter).unwrap().data(),CssomRuleData::CounterStyle{name,..} if name.is_empty())
    );
    for id in &maps {
        let map = after.feature_map(id).unwrap();
        assert_eq!(map.get("").unwrap(), Some([].as_slice()));
        assert_eq!(map.get("absent").unwrap(), None);
        assert_eq!(map.get("A!?").unwrap(), Some([0].as_slice()));
    }
    assert_eq!(
        after.feature_map(styleset).unwrap().get("huge").unwrap(),
        Some([u32::MAX].as_slice())
    );
    assert_eq!(
        after.feature_map(styleset).unwrap().authored()[0].value(),
        value
    );
    let mut edit = batch(&store);
    edit.set_counter_name(&counter, "A!?\0").unwrap();
    store.commit(edit).unwrap();
    assert!(
        matches!(store.snapshot().rule(&counter).unwrap().data(),CssomRuleData::CounterStyle{name,..} if name=="A!?\0")
    );
    assert!(
        matches!(before.rule(&counter).unwrap().data(),CssomRuleData::CounterStyle{name,..} if name=="custom")
    );
}

#[test]
fn map_cardinality_failure_poisoning_and_order_are_atomic() {
    let mut store = new_store();
    let sheet = create(&mut store, "@font-feature-values Demo {}");
    let snapshot = store.snapshot();
    let maps = match snapshot.rule(&root(&snapshot, &sheet)).unwrap().data() {
        CssomRuleData::FontFeatureValues { maps, .. } => maps.clone(),
        _ => panic!("maps"),
    };
    for id in &maps {
        let kind = snapshot.feature_map(id).unwrap().kind();
        let maximum = match kind {
            CssFontFeatureValueKind::Styleset | CssFontFeatureValueKind::HistoricalForms => None,
            CssFontFeatureValueKind::CharacterVariant => Some(2),
            _ => Some(1),
        };
        let mut edit = batch(&store);
        edit.set_feature(id, "x", &vec![0; maximum.unwrap_or(4)])
            .unwrap();
        store.commit(edit).unwrap();
        if let Some(max) = maximum {
            let before = store.snapshot();
            let mut edit = batch(&store);
            edit.set_disabled(&sheet, true).unwrap();
            assert!(matches!(
                edit.set_feature(id, "bad", &vec![0; max + 1]),
                Err(CssomError::Source(CssomException::InvalidAccess))
            ));
            assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
            assert_eq!(store.snapshot().revision(), before.revision());
            assert!(!store.snapshot().sheet(&sheet).unwrap().inputs().disabled);
        }
    }
    let id = &maps[0];
    let mut edit = batch(&store);
    edit.set_feature(id, "a", &[]).unwrap();
    edit.set_feature(id, "x", &[4]).unwrap();
    edit.delete_feature(id, "x").unwrap();
    edit.set_feature(id, "x", &[5]).unwrap();
    store.commit(edit).unwrap();
    assert_eq!(
        store
            .snapshot()
            .feature_map(id)
            .unwrap()
            .entries()
            .iter()
            .map(|v| v.key.as_str())
            .collect::<Vec<_>>(),
        ["a", "x"]
    );
}

#[test]
fn computed_readonly_owner_and_null_base_are_independent_explicit_inputs() {
    let mut store = new_store();
    let declarations = parse_declaration_block_contents("color: red;")
        .into_parts()
        .0;
    for computed in [false, true] {
        for readonly in [false, true] {
            let mut edit = batch(&store);
            let ticket = edit
                .create_declarations(
                    declarations.clone(),
                    computed,
                    readonly,
                    Some(version(CssomInputRole::Owner, "attribute", 3)),
                )
                .unwrap();
            let id = store.commit(edit).unwrap().block(&ticket).unwrap().clone();
            let before = store.snapshot();
            assert_eq!(before.block(&id).unwrap().flags().computed, computed);
            assert_eq!(before.block(&id).unwrap().flags().readonly, readonly);
            assert!(!before.block(&id).unwrap().flags().updating);
            assert_eq!(before.block(&id).unwrap().owner().unwrap().revision, 3);
            assert_eq!(
                before
                    .specified_property_css_text(
                        &id,
                        CssSpecifiedValueSerializationLimits::default()
                    )
                    .unwrap(),
                if computed { "" } else { "color: red;" }
            );
            let mut edit = batch(&store);
            let result = edit
                .remove_selected_terminal(&id, CssPropertyNameRef::Known(CssKnownProperty::Color));
            if readonly {
                assert!(matches!(
                    result,
                    Err(CssomError::Source(CssomException::NoModificationAllowed))
                ));
                assert!(store.commit(edit).is_err());
                assert_eq!(store.snapshot().revision(), before.revision());
            } else {
                assert!(result.unwrap());
                store.commit(edit).unwrap();
            }
        }
    }
    let mut edit = batch(&store);
    let a = edit.create_typed_sheet(CssSheet::new(), inputs()).unwrap();
    let mut empty = inputs();
    empty.base_url = Some(String::new());
    let b = edit.create_typed_sheet(CssSheet::new(), empty).unwrap();
    let commit = store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    assert_eq!(
        snapshot
            .sheet(commit.sheet(&a).unwrap())
            .unwrap()
            .inputs()
            .base_url,
        None
    );
    assert_eq!(
        snapshot
            .sheet(commit.sheet(&b).unwrap())
            .unwrap()
            .inputs()
            .base_url,
        Some(String::new())
    );
}

#[test]
fn typed_and_recovered_creation_share_checked_boundary_without_fabricated_origins() {
    let mut store = new_store();
    let namespace = CssNamespaceRule::new(None, CssNamespaceName::new("urn:typed"));
    let typed = CssSheet::try_from_rules(vec![CssRule::Namespace(namespace)]).unwrap();
    let mut edit = batch(&store);
    let ticket = edit.create_typed_sheet(typed, inputs()).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    match snapshot.rule(&root(&snapshot, &sheet)).unwrap().authored() {
        CssomAuthoredRule::Ordinary(CssRule::Namespace(v)) => assert!(v.position().is_none()),
        _ => panic!("namespace"),
    }
    let mut edit = batch(&store);
    edit.replace_sheet_text(
        &sheet,
        "@import 'ignored'; invalid { bad: nope; } b { color: red; }",
    )
    .unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(after.sheet_rules(&sheet).unwrap().len(), 2);
    assert!(!after.sheet(&sheet).unwrap().diagnostics().is_empty());
    assert!(
        after
            .sheet_rules(&sheet)
            .unwrap()
            .iter()
            .all(|id| !matches!(after.rule(id).unwrap().data(), CssomRuleData::Import { .. }))
    );
}

#[test]
fn security_modification_index_namespace_and_product_guards_keep_precedence() {
    let mut store = new_store();
    let mut edit = batch(&store);
    let mut facts =
        CssomSheetInputs::external(version(CssomInputRole::Origin, "foreign", 1), false);
    facts.disallow_modification = true;
    let ticket = edit.create_parsed_sheet("a {}", facts).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    assert!(snapshot.sheet(&sheet).is_ok());
    assert!(matches!(
        snapshot.sheet_rules(&sheet),
        Err(CssomError::Source(CssomException::Security))
    ));
    let mut edit = batch(&store);
    assert!(matches!(
        edit.delete_sheet_rule(&sheet, 999),
        Err(CssomError::Source(CssomException::Security))
    ));
    assert!(store.commit(edit).is_err());
    let namespace = create(&mut store, "@namespace 'urn:x'; a {}");
    let mut edit = batch(&store);
    assert!(matches!(
        edit.delete_sheet_rule(&namespace, 999),
        Err(CssomError::Source(CssomException::IndexSize))
    ));
    let mut edit = batch(&store);
    assert!(matches!(
        edit.delete_sheet_rule(&namespace, 0),
        Err(CssomError::Source(CssomException::InvalidState))
    ));
    let mut other = new_store();
    let foreign = create(&mut other, "b {}");
    let mut edit = batch(&store);
    assert!(matches!(
        edit.delete_sheet_rule(&foreign, 999),
        Err(CssomError::ForeignOwner)
    ));
}

#[test]
fn noops_stale_foreign_failed_and_created_then_detached_batches_publish_correctly() {
    let mut store = new_store();
    let sheet = create(&mut store, "a {}");
    let before = store.snapshot();
    let rule = root(&before, &sheet);
    let mut noop = batch(&store);
    noop.set_disabled(&sheet, true).unwrap();
    noop.set_disabled(&sheet, false).unwrap();
    noop.set_rule_css_text(&rule, "invalid").unwrap();
    assert!(matches!(
        store.commit(noop).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    let mut stale = batch(&store);
    stale.set_disabled(&sheet, true).unwrap();
    let mut fresh = batch(&store);
    fresh.set_disabled(&sheet, true).unwrap();
    store.commit(fresh).unwrap();
    assert!(matches!(
        store.commit(stale),
        Err(CssomError::StaleRevision)
    ));
    let mut other = new_store();
    assert!(matches!(
        other.commit(batch(&store)),
        Err(CssomError::ForeignOwner)
    ));
    let mut edit = batch(&store);
    let ticket = edit
        .create_parsed_sheet("b { color: red; }", inputs())
        .unwrap();
    let removed = edit.delete_created_sheet_rule(&ticket, 0).unwrap();
    let commit = store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    assert!(
        snapshot
            .sheet_rules(commit.sheet(&ticket).unwrap())
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        snapshot
            .rule(commit.rule(&removed).unwrap())
            .unwrap()
            .parent(),
        &CssomParent::Detached
    );
    assert!(commit.sheet(&CssomSheetTicket::clone(&ticket)).is_ok());
}

#[test]
fn real_limits_exhaustion_and_history_gaps_do_not_publish_partial_state() {
    let limits = CssomLimits {
        max_revision: 1,
        summary_history: 1,
        ..CssomLimits::default()
    };
    let mut store = CssomStore::new(limits, CssomContext::default()).unwrap();
    let empty = store.snapshot();
    let sheet = create(&mut store, "a {}");
    let before = store.snapshot();
    let mut edit = batch(&store);
    assert!(!edit.set_disabled(&sheet, false).unwrap());
    assert!(matches!(
        store.commit(edit).unwrap().publication(),
        CssomPublication::Unchanged { .. }
    ));
    let mut edit = batch(&store);
    edit.set_disabled(&sheet, true).unwrap();
    assert!(matches!(
        store.commit(edit),
        Err(CssomError::RevisionExhausted)
    ));
    assert_eq!(store.snapshot().revision(), before.revision());
    assert!(!store.snapshot().sheet(&sheet).unwrap().inputs().disabled);
    assert!(matches!(
        store.changes_since(&empty),
        CssomChanges::FullRecompute(CssomRecomputeReason::ConservativeChange)
    ));
    let mut limited = CssomStore::new(
        CssomLimits {
            max_identity: 1,
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let mut edit = batch(&limited);
    assert!(matches!(
        edit.create_typed_sheet(CssSheet::new(), inputs()),
        Err(CssomError::IdentityExhausted)
    ));
    assert!(limited.commit(edit).is_err());
    assert!(limited.snapshot().sheets().is_empty());
    let mut bounded = CssomStore::new(
        CssomLimits {
            max_objects: 4,
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let mut edit = batch(&bounded);
    edit.create_typed_sheet(CssSheet::new(), inputs()).unwrap();
    assert!(matches!(
        edit.create_typed_sheet(CssSheet::new(), inputs()),
        Err(CssomError::Limit { .. })
    ));
    assert!(bounded.commit(edit).is_err());
    assert_eq!(bounded.snapshot().revision().value(), 0);
    let mut history = CssomStore::new(
        CssomLimits {
            summary_history: 1,
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let sheet = create(&mut history, "a {}");
    let old = history.snapshot();
    for disabled in [true, false] {
        let mut edit = batch(&history);
        edit.set_disabled(&sheet, disabled).unwrap();
        history.commit(edit).unwrap();
    }
    assert!(matches!(
        history.changes_since(&old),
        CssomChanges::FullRecompute(CssomRecomputeReason::CoverageGap)
    ));
}

#[test]
fn summaries_coalesce_categories_and_capture_linked_context_revisions() {
    let mut linked = new_store();
    let linked_sheet = create(&mut linked, "b {}");
    let linked_before = linked.snapshot();
    let mut store = new_store();
    let sheet = create(&mut store, "a { color: red; }");
    let before = store.snapshot();
    let mut edit = batch(&store);
    edit.set_disabled(&sheet, true).unwrap();
    store.commit(edit).unwrap();
    let mut edit = batch(&store);
    edit.remove_selected_terminal(
        &block(&before, &root(&before, &sheet)),
        CssPropertyNameRef::Known(CssKnownProperty::Color),
    )
    .unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    let CssomChanges::Incremental(summary) = store.changes_since(&before) else {
        panic!("contiguous coverage")
    };
    assert!(summary.categories().contains(&CssomChange::Disabled));
    assert!(summary.categories().contains(&CssomChange::Declarations));
    assert!(summary.supports_incremental(&before, &after));
    assert!(!summary.supports_incremental(&before, &before));
    let context = CssomContext {
        inputs: vec![CssomInput {
            version: version(CssomInputRole::Layer, "layer", 7),
            data: CssomInputData::Layer {
                path: vec!["theme".into(), "components".into()],
                ordinal: 3,
            },
        }],
        linked: vec![CssomLinkedInput {
            version: version(CssomInputRole::Import, "import", 3),
            snapshot: linked_before.clone(),
            sheet: linked_sheet.clone(),
        }],
    };
    let mut edit = batch(&store);
    assert!(edit.update_context(context).unwrap());
    store.commit(edit).unwrap();
    let capture = store.snapshot();
    let mut edit = batch(&linked);
    edit.set_disabled(&linked_sheet, true).unwrap();
    linked.commit(edit).unwrap();
    drop(linked);
    assert_eq!(
        capture.context().linked[0].snapshot.revision(),
        linked_before.revision()
    );
    assert!(
        !capture.context().linked[0]
            .snapshot
            .sheet(&linked_sheet)
            .unwrap()
            .inputs()
            .disabled
    );
    assert_eq!(capture.context().inputs[0].version.revision, 7);
    assert!(
        matches!(&capture.context().inputs[0].data,CssomInputData::Layer{path,ordinal:3} if path==&["theme","components"])
    );
    assert!(matches!(
        store.changes_since(&after),
        CssomChanges::FullRecompute(CssomRecomputeReason::ConservativeChange)
    ));
}

#[test]
fn current_import_facets_and_nullable_child_capture_are_owned_independently() {
    let mut store = new_store();
    let mut edit = batch(&store);
    let ticket = edit
        .create_parsed_sheet(
            "@import url('child.css') layer(theme) supports(display: grid) screen;",
            CssomSheetInputs::external(version(CssomInputRole::Document, "external", 1), true),
        )
        .unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let before = store.snapshot();
    let import = root(&before, &sheet);
    let media = match before.rule(&import).unwrap().data() {
        CssomRuleData::Import {
            target: _,
            layer,
            supports,
            media,
            child_sheet,
            resolved_location,
            input,
            ..
        } => {
            assert!(layer.is_some());
            assert!(supports.is_some());
            assert!(child_sheet.is_none());
            assert!(resolved_location.is_none());
            assert!(input.is_none());
            media.clone()
        }
        _ => panic!("import"),
    };
    assert_eq!(
        before
            .media(&media)
            .unwrap()
            .serialize_cssom()
            .unwrap()
            .as_css(),
        "screen"
    );
    let child = create(&mut store, "b { color: red; }");
    let mut edit = batch(&store);
    edit.associate_import(
        &import,
        &child,
        version(CssomInputRole::Import, "fetch", 2),
        Some("https://example.test/child.css".into()),
    )
    .unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(after.sheet(&child).unwrap().owner_rule(), Some(&import));
    assert_eq!(after.parent_sheet(&child).unwrap(), Some(&sheet));
    assert!(
        matches!(after.rule(&import).unwrap().data(),CssomRuleData::Import{media:id,child_sheet:Some(id2),input:Some(v),..} if id==&media && id2==&child && v.revision==2)
    );
    assert!(matches!(
        before.rule(&import).unwrap().data(),
        CssomRuleData::Import {
            child_sheet: None,
            ..
        }
    ));
    let child2 = create(&mut store, "");
    let before_failed = store.snapshot();
    let mut edit = batch(&store);
    assert!(matches!(
        edit.associate_import(
            &import,
            &child2,
            version(CssomInputRole::Import, "fetch", 3),
            None
        ),
        Err(CssomError::Source(CssomException::InvalidState))
    ));
    assert!(store.commit(edit).is_err());
    assert_eq!(store.snapshot().revision(), before_failed.revision());
}

#[test]
fn domain_payloads_and_font_display_are_current_checked_capture_facts() {
    let mut store = new_store();
    let sheet = create(
        &mut store,
        "@page { size: a4; margin: 1px; @top-left { content: 'x'; } } @keyframes demo { from { opacity: 0; } } @font-face { font-family: Demo; src: url('demo.woff'); } @font-feature-values Demo { font-display: swap; @historical-forms { old: 1 2; } }",
    );
    let snapshot = store.snapshot();
    let roots = snapshot.sheet_rules(&sheet).unwrap();
    assert_eq!(roots.len(), 4);
    assert!(matches!(
        snapshot.block(&block(&snapshot, &roots[0])).unwrap().data(),
        CssomBlockData::Page {
            selected: CssomProjection::Available(_),
            ..
        }
    ));
    let margin = children(&snapshot, &roots[0])[0].clone();
    assert!(matches!(
        snapshot.block(&block(&snapshot, &margin)).unwrap().data(),
        CssomBlockData::Properties {
            domain: CssomPropertyDomain::Margin,
            ..
        }
    ));
    let frame = children(&snapshot, &roots[1])[0].clone();
    assert!(matches!(
        snapshot.block(&block(&snapshot, &frame)).unwrap().data(),
        CssomBlockData::Properties {
            domain: CssomPropertyDomain::Keyframe,
            ..
        }
    ));
    assert!(matches!(
        snapshot.block(&block(&snapshot, &roots[2])).unwrap().data(),
        CssomBlockData::FontFace(_)
    ));
    let CssomRuleData::FontFeatureValues {
        font_display, maps, ..
    } = snapshot.rule(&roots[3]).unwrap().data()
    else {
        panic!("feature values")
    };
    assert_eq!(font_display.len(), 1);
    assert!(font_display[0].parsed_name().is_some());
    let historical = maps
        .iter()
        .find(|id| {
            snapshot.feature_map(id).unwrap().kind() == CssFontFeatureValueKind::HistoricalForms
        })
        .unwrap();
    assert_eq!(
        snapshot
            .feature_map(historical)
            .unwrap()
            .get("old")
            .unwrap(),
        Some([1, 2].as_slice())
    );
}

#[test]
fn unavailable_symbolic_projection_is_retained_but_resource_failures_abort() {
    let mut store = new_store();
    let sheet = create(&mut store, "a { inset: var(--inset); opacity: 1; }");
    let snapshot = store.snapshot();
    let id = block(&snapshot, &root(&snapshot, &sheet));
    match snapshot.block(&id).unwrap().data() {
        CssomBlockData::Properties {
            authored: CssomPropertyOccurrences::Ordinary(raw),
            selected: CssomProjection::Unavailable(error),
            ..
        } => {
            assert_eq!(raw.len(), 2);
            assert!(matches!(
                error.kind(),
                CssDeclarationBlockErrorKind::PendingFootprintUndetermined { .. }
            ));
            assert!(raw[0].parsed_value().is_some());
        }
        _ => panic!("unavailable projection retained"),
    };
    let mut edit = batch(&store);
    assert!(matches!(
        edit.remove_selected_terminal(&id, CssPropertyNameRef::Known(CssKnownProperty::Opacity)),
        Err(CssomError::Declaration(_))
    ));
    assert!(store.commit(edit).is_err());
    assert_eq!(store.snapshot().revision(), snapshot.revision());
    let mut bounded = CssomStore::new(
        CssomLimits {
            css: CssSpecifiedValueSerializationLimits::new(0, 0, 0),
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let mut edit = batch(&bounded);
    assert!(matches!(
        edit.create_parsed_sheet("a { color: red; }", inputs()),
        Err(CssomError::Declaration(_))
    ));
    assert!(bounded.commit(edit).is_err());
    assert!(bounded.snapshot().sheets().is_empty());
}

#[test]
fn typed_unavailable_values_cannot_bypass_input_and_component_quotas() {
    let raw = parse_declaration_block_contents("inset:var(--inset);")
        .into_parts()
        .0;
    let mut nodes = CssomStore::new(
        CssomLimits {
            max_entries: 1,
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let mut edit = batch(&nodes);
    assert!(matches!(
        edit.create_declarations(raw.clone(), false, false, None),
        Err(CssomError::Limit {
            resource: "authored component entries",
            ..
        })
    ));
    assert!(nodes.commit(edit).is_err());
    assert_eq!(nodes.snapshot().revision().value(), 0);
    let mut bytes = CssomStore::new(
        CssomLimits {
            max_input_bytes: 5,
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let mut edit = batch(&bytes);
    assert!(matches!(
        edit.create_declarations(raw, false, false, None),
        Err(CssomError::Limit {
            resource: "retained parse input bytes",
            ..
        })
    ));
    assert!(bytes.commit(edit).is_err());
    assert_eq!(bytes.snapshot().revision().value(), 0);
}

#[test]
fn media_with_only_empty_nested_child_keeps_literal_two_newline_frame() {
    let mut store = new_store();
    let sheet = create(&mut store, "a { @media screen { color: red; } }");
    let before = store.snapshot();
    let style = root(&before, &sheet);
    let media = children(&before, &style)[0].clone();
    let nested = children(&before, &media)[0].clone();
    let block = block(&before, &nested);
    let mut edit = batch(&store);
    edit.remove_selected_terminal(&block, CssPropertyNameRef::Known(CssKnownProperty::Color))
        .unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(
        children(&after, &media).as_slice(),
        std::slice::from_ref(&nested)
    );
    let nested_view = CssEditedRuleView::try_nested_declarations(selected(&after, &block)).unwrap();
    assert_eq!(nested_view.serialize_cssom().unwrap(), "");
    let children = [nested_view];
    let CssomRuleData::Group {
        prelude: CssomGroupPrelude::Media(id),
        ..
    } = after.rule(&media).unwrap().data()
    else {
        panic!("media")
    };
    let media = CssEditedRuleView::try_group(
        CssEditedGroupPreludeRef::Media(after.media(id).unwrap()),
        &children,
    )
    .unwrap();
    assert_eq!(media.serialize_cssom().unwrap(), "@media screen {\n\n}");
}

#[test]
fn constructed_creation_filters_imports_and_repeated_checked_ingress_gets_fresh_ids() {
    let mut store = new_store();
    let source = parse_sheet("@import 'child.css'; a {color:red;}")
        .into_parts()
        .0;
    let mut edit = batch(&store);
    let a = edit.create_typed_sheet(source.clone(), inputs()).unwrap();
    let b = edit.create_typed_sheet(source, inputs()).unwrap();
    let result = store.commit(edit).unwrap();
    let first = result.sheet(&a).unwrap();
    let second = result.sheet(&b).unwrap();
    let snapshot = store.snapshot();
    assert_ne!(first, second);
    assert_eq!(snapshot.sheet_rules(first).unwrap().len(), 1);
    assert_eq!(snapshot.sheet_rules(second).unwrap().len(), 1);
    assert_ne!(root(&snapshot, first), root(&snapshot, second));
    assert_eq!(snapshot.sheet(first).unwrap().authored().rules().len(), 2);
    let original_a =
        selected(&snapshot, &block(&snapshot, &root(&snapshot, first))).entries()[0].source();
    let original_b =
        selected(&snapshot, &block(&snapshot, &root(&snapshot, second))).entries()[0].source();
    assert!(original_a.same_occurrence(original_b));
    let sheet = create(&mut store, "@import 'another.css'; b {}");
    assert_eq!(store.snapshot().sheet_rules(&sheet).unwrap().len(), 1);
    let mut edit = batch(&store);
    let typed = edit
        .create_typed_sheet(parse_sheet("@import 'only.css';").into_parts().0, inputs())
        .unwrap();
    let parsed = edit
        .create_parsed_sheet("@import 'only.css';", inputs())
        .unwrap();
    let result = store.commit(edit).unwrap();
    let snapshot = store.snapshot();
    for id in [
        result.sheet(&typed).unwrap(),
        result.sheet(&parsed).unwrap(),
    ] {
        assert!(snapshot.sheet_rules(id).unwrap().is_empty());
        assert_eq!(snapshot.sheet(id).unwrap().authored().rules().len(), 1);
        assert_eq!(
            snapshot
                .sheet(id)
                .unwrap()
                .inputs()
                .constructor_document
                .as_deref(),
            Some("document")
        );
    }
    let mut limited = CssomStore::new(
        CssomLimits {
            max_entries: 1,
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let mut edit = batch(&limited);
    assert!(matches!(
        edit.create_parsed_sheet("@import 'only.css';", inputs()),
        Err(CssomError::Limit {
            resource: "entries",
            ..
        })
    ));
    assert!(limited.commit(edit).is_err());
    assert!(limited.snapshot().sheets().is_empty());
}

#[test]
fn independent_context_guards_and_bounded_facts_reject_without_publication() {
    let mut store = new_store();
    let before = store.snapshot();
    let context = CssomContext {
        inputs: vec![CssomInput {
            version: version(CssomInputRole::Origin, "origin", 2),
            data: CssomInputData::Origin {
                origin: "https://example.test".into(),
                profile: "author".into(),
            },
        }],
        linked: Vec::new(),
    };
    assert!(matches!(
        store.batch(CssomEditGuard::for_inputs(&before, context.clone())),
        Err(CssomError::StaleContext)
    ));
    let mut edit = batch(&store);
    edit.update_context(context.clone()).unwrap();
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(after.context(), &context);
    assert_ne!(before.revision(), after.revision());
    let mut bad = context.clone();
    bad.inputs.push(context.inputs[0].clone());
    let mut edit = batch(&store);
    assert!(matches!(
        edit.update_context(bad),
        Err(CssomError::InvalidInput(_))
    ));
    assert!(store.commit(edit).is_err());
    assert_eq!(store.snapshot().revision(), after.revision());
    let mut bounded = CssomStore::new(
        CssomLimits {
            max_string_bytes: 2,
            max_input_bytes: 2,
            ..CssomLimits::default()
        },
        CssomContext::default(),
    )
    .unwrap();
    let mut edit = batch(&bounded);
    assert!(matches!(
        edit.update_context(context),
        Err(CssomError::Limit { .. })
    ));
    assert!(bounded.commit(edit).is_err());
    assert_eq!(bounded.snapshot().revision().value(), 0);
    let mut edit = batch(&bounded);
    assert!(matches!(
        edit.create_parsed_sheet("a {}", inputs()),
        Err(CssomError::Limit {
            resource: "parse input bytes",
            ..
        })
    ));
    assert!(bounded.commit(edit).is_err());
    assert!(bounded.snapshot().sheets().is_empty());
}

#[test]
fn parsed_ingress_uses_current_parser_facts_and_typed_ingress_keeps_its_own() {
    let quirks = CssParserContext::new(CssParserMode::Quirks);
    let mut store = new_store();
    let standard = create(&mut store, "a { width: 10; }");
    let before = store.snapshot();
    assert!(
        selected(&before, &block(&before, &root(&before, &standard)))
            .entries()
            .is_empty()
    );
    let context = CssomContext {
        inputs: vec![CssomInput {
            version: version(CssomInputRole::ParserMode, "mode", 1),
            data: CssomInputData::ParserMode(quirks),
        }],
        linked: Vec::new(),
    };
    let mut edit = batch(&store);
    edit.update_context(context.clone()).unwrap();
    let parsed = edit
        .create_parsed_sheet("a { width: 10; }", inputs())
        .unwrap();
    let typed = edit
        .create_typed_sheet(parse_sheet("b { height: 10; }").syntax().clone(), inputs())
        .unwrap();
    let receipt = store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(after.context(), &context);
    assert_eq!(before.context(), &CssomContext::default());
    let parsed = receipt.sheet(&parsed).unwrap();
    let typed = receipt.sheet(&typed).unwrap();
    let parsed_block = block(&after, &root(&after, parsed));
    assert_eq!(
        after
            .specified_property_css_text(
                &parsed_block,
                CssSpecifiedValueSerializationLimits::default()
            )
            .unwrap(),
        "width: 10px;"
    );
    assert_eq!(
        selected(&after, &parsed_block).entries()[0]
            .source()
            .parser_context(),
        quirks
    );
    assert!(
        selected(&after, &block(&after, &root(&after, typed)))
            .entries()
            .is_empty()
    );
    assert!(
        selected(&after, &block(&after, &root(&after, &standard)))
            .entries()
            .is_empty()
    );
    let mut edit = batch(&store);
    edit.replace_sheet_text(&standard, "a { height: 11; }")
        .unwrap();
    store.commit(edit).unwrap();
    let latest = store.snapshot();
    assert_eq!(
        latest
            .specified_property_css_text(
                &block(&latest, &root(&latest, &standard)),
                CssSpecifiedValueSerializationLimits::default()
            )
            .unwrap(),
        "height: 11px;"
    );
    assert!(
        selected(&before, &block(&before, &root(&before, &standard)))
            .entries()
            .is_empty()
    );
    let mut ambiguous = context;
    ambiguous.inputs.push(CssomInput {
        version: version(CssomInputRole::ParserMode, "other", 2),
        data: CssomInputData::ParserMode(CssParserContext::default()),
    });
    let mut edit = batch(&store);
    assert!(matches!(
        edit.update_context(ambiguous),
        Err(CssomError::InvalidInput("ambiguous parser mode input"))
    ));
    assert!(store.commit(edit).is_err());
    assert_eq!(store.snapshot().revision(), latest.revision());
}

#[test]
fn indexed_group_deletion_rejects_keyframes_atomically_and_detaches_grouping_children() {
    let mut store = new_store();
    let sheet = create(
        &mut store,
        "@keyframes demo { from { opacity: 0; } to { opacity: 1; } } a { & b { color: red; } } @media screen { c { color: blue; } } @page { @top-left { content: 'x'; } }",
    );
    let before = store.snapshot();
    let roots = before.sheet_rules(&sheet).unwrap().to_vec();
    assert_eq!(roots.len(), 4);
    let keyframes = &roots[0];
    assert!(matches!(
        before.rule(keyframes).unwrap().data(),
        CssomRuleData::Keyframes { .. }
    ));
    let frames = children(&before, keyframes);
    assert_eq!(frames.len(), 2);
    let frame_blocks = frames
        .iter()
        .map(|frame| block(&before, frame))
        .collect::<Vec<_>>();
    let groups = &roots[1..];
    assert!(matches!(
        before.rule(&groups[0]).unwrap().data(),
        CssomRuleData::Style { .. }
    ));
    assert!(matches!(
        before.rule(&groups[1]).unwrap().data(),
        CssomRuleData::Group { .. }
    ));
    assert!(matches!(
        before.rule(&groups[2]).unwrap().data(),
        CssomRuleData::Page { .. }
    ));
    let grouping_children = groups
        .iter()
        .map(|group| {
            let children = children(&before, group);
            assert_eq!(children.len(), 1);
            children[0].clone()
        })
        .collect::<Vec<_>>();
    for index in [0, usize::MAX] {
        let mut edit = batch(&store);
        assert!(edit.set_disabled(&sheet, true).unwrap());
        assert_eq!(
            edit.delete_group_rule(&groups[0], 0).unwrap(),
            grouping_children[0]
        );
        assert!(matches!(
            edit.delete_group_rule(keyframes, index),
            Err(CssomError::WrongKind)
        ));
        assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
        let unchanged = store.snapshot();
        assert_eq!(unchanged.revision(), before.revision());
        assert!(matches!(
            store.changes_since(&before),
            CssomChanges::Unchanged
        ));
        assert!(!unchanged.sheet(&sheet).unwrap().inputs().disabled);
        assert_eq!(unchanged.sheet_rules(&sheet).unwrap(), roots);
        assert_eq!(children(&unchanged, keyframes), frames);
        for (frame, expected_block) in frames.iter().zip(&frame_blocks) {
            assert_eq!(&block(&unchanged, frame), expected_block);
            assert_eq!(unchanged.parent_rule(frame).unwrap(), Some(keyframes));
            assert_eq!(unchanged.parent_style_sheet(frame).unwrap(), Some(&sheet));
            assert_eq!(selected(&unchanged, expected_block).entries().len(), 1);
        }
        for (group, child) in groups.iter().zip(&grouping_children) {
            assert_eq!(children(&unchanged, group), vec![child.clone()]);
            assert_eq!(unchanged.parent_rule(child).unwrap(), Some(group));
        }
    }
    let mut edit = batch(&store);
    for (group, child) in groups.iter().zip(&grouping_children) {
        assert_eq!(&edit.delete_group_rule(group, 0).unwrap(), child);
    }
    store.commit(edit).unwrap();
    let after = store.snapshot();
    assert_eq!(after.revision().value(), before.revision().value() + 1);
    for (group, child) in groups.iter().zip(&grouping_children) {
        assert!(children(&after, group).is_empty());
        assert_eq!(after.rule(child).unwrap().parent(), &CssomParent::Detached);
        assert_eq!(after.parent_rule(child).unwrap(), None);
        assert_eq!(after.parent_style_sheet(child).unwrap(), None);
        assert_eq!(before.parent_rule(child).unwrap(), Some(group));
    }
    assert_eq!(children(&after, keyframes), frames);
}
