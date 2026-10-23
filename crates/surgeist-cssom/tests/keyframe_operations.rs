use surgeist_css::*;
use surgeist_cssom::*;
fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}
fn fixture(
    source: &str,
    limits: CssomLimits,
) -> (CssomStore, CssomSheetId, CssomRuleId, Vec<CssomRuleId>) {
    let mut store = CssomStore::new(limits, CssomContext::default()).unwrap();
    let inputs = CssomSheetInputs::constructed(
        CssomInputVersion {
            role: CssomInputRole::Document,
            identity: "document".into(),
            revision: 1,
        },
        "document".into(),
        None,
    );
    let mut edit = batch(&store);
    let ticket = edit.create_parsed_sheet(source, inputs).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    let snapshot = store.snapshot();
    let root = snapshot
        .rules(snapshot.sheet(&sheet).unwrap().rules())
        .unwrap()[0]
        .clone();
    let children = snapshot
        .rules(
            snapshot
                .rule(&root)
                .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                .unwrap(),
        )
        .unwrap()
        .to_vec();
    (store, sheet, root, children)
}

#[test]
fn ordered_full_sequence_last_match_is_independent_of_counts_duplicates_and_single_offsets() {
    let (mut store, _, root, children) = fixture(
        "@keyframes test { 0%, 100% {} from, to {} 100%, 0% {} 0% {} 0%, 100%, 100% {} }",
        CssomLimits::default(),
    );
    let before = store.snapshot();
    let limits = CssomLimits::default();
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.find_rule(" from , to ", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[1].clone())
    );
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.find_rule("to, from", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[2].clone())
    );
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.find_rule("from", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[3].clone())
    );
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.find_rule("from, to, to", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[4].clone())
    );
    let block = before
        .rule(&children[1])
        .and_then(|record| record.data().block().ok_or(CssomError::WrongKind))
        .unwrap()
        .clone();
    let list = before
        .rule(&root)
        .unwrap()
        .data()
        .children()
        .unwrap()
        .clone();
    let mut edit = batch(&store);
    assert_eq!(
        edit.delete_keyframe_rule(&root, "from,to").unwrap(),
        Some(children[1].clone())
    );
    let commit = store.commit(edit).unwrap();
    let CssomPublication::Changed(summary) = commit.publication() else {
        panic!("deletion publishes");
    };
    let current = store.snapshot();
    assert_eq!(
        current.rule(&root).unwrap().data().children().unwrap(),
        &list
    );
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&current))
            .and_then(|capture| capture.find_rule("0%,100%", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[0].clone())
    );
    assert_eq!(current.parent_rule(&children[1]).unwrap(), None);
    assert_eq!(current.parent_style_sheet(&children[1]).unwrap(), None);
    assert_eq!(
        current
            .rule(&children[1])
            .and_then(|record| record.data().block().ok_or(CssomError::WrongKind))
            .unwrap(),
        &block
    );
    assert!(summary.categories().contains(&CssomChange::RuleMembership));
    assert!(summary.categories().contains(&CssomChange::Order));
    assert!(summary.categories().contains(&CssomChange::Ancestry));
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.find_rule("from,to", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[1].clone())
    );
}

#[test]
fn current_key_text_and_specified_calculation_phase_control_matching_without_text_keys() {
    let (mut store, _, root, children) = fixture(
        "@keyframes test { 10% {} calc(10%) {} calc(5% + 5%) {} calc(25% * sign(1em + 1px)) {} calc(25% * sign(1px + 1em)) {} }",
        CssomLimits::default(),
    );
    let before = store.snapshot();
    let limits = CssomLimits::default();
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.find_rule("10%", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[0].clone())
    );
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.find_rule("calc( 10% )", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[2].clone())
    );
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.find_rule("calc(25% * sign(1em + 1px))", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[3].clone())
    );
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.find_rule("calc(25% * sign(1px + 1em))", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[4].clone())
    );
    let block = before
        .rule(&children[2])
        .and_then(|record| record.data().block().ok_or(CssomError::WrongKind))
        .unwrap()
        .clone();
    let mut edit = batch(&store);
    edit.set_keyframe_key_text(&children[2], "20%").unwrap();
    store.commit(edit).unwrap();
    let current = store.snapshot();
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&current))
            .and_then(|capture| capture.find_rule("calc(10%)", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[1].clone())
    );
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&current))
            .and_then(|capture| capture.find_rule("20%", &limits))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[2].clone())
    );
    assert_eq!(
        current
            .rule(&children[2])
            .and_then(|record| record.data().block().ok_or(CssomError::WrongKind))
            .unwrap(),
        &block
    );
}

#[test]
fn invalid_or_unmatched_key_queries_are_noops_in_a_mixed_batch() {
    let (mut store, sheet, root, children) =
        fixture("@keyframes test { from {} to {} }", CssomLimits::default());
    let before = store.snapshot();
    let limits = CssomLimits::default();
    for query in [")", "50% garbage", "from,", "50%"] {
        assert_eq!(
            store
                .css_keyframes_rule(&root)
                .and_then(|handle| handle.capture(&before))
                .and_then(|capture| capture.find_rule(query, &limits))
                .map(|found| found.map(|rule| rule.handle().id().clone()))
                .unwrap(),
            None
        );
    }
    let mut edit = batch(&store);
    edit.set_disabled(&sheet, true).unwrap();
    assert_eq!(edit.delete_keyframe_rule(&root, ")").unwrap(), None);
    assert_eq!(edit.delete_keyframe_rule(&root, "50%").unwrap(), None);
    store.commit(edit).unwrap();
    let current = store.snapshot();
    assert!(current.sheet(&sheet).unwrap().inputs().disabled);
    assert_eq!(
        current
            .rules(
                current
                    .rule(&root)
                    .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                    .unwrap()
            )
            .unwrap(),
        children
    );
    assert_eq!(
        before
            .rules(
                before
                    .rule(&root)
                    .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                    .unwrap()
            )
            .unwrap(),
        children
    );
}

#[test]
fn a_whole_reverse_search_shares_native_work_and_resource_failure_rolls_back_batch() {
    // The provider's public literal tariff admits query + two candidates at
    // (6,8), but not a third; resetting a pair budget would falsely succeed.
    let limits = CssomLimits {
        css: CssSerializationLimits::new(6, 8, 0),
        ..CssomLimits::default()
    };
    let (mut store, sheet, root, children) =
        fixture("@keyframes test { 10% {} 20% {} 30% {} }", limits.clone());
    let before = store.snapshot();
    let (query, _) = parse_keyframe_selector_list("10%").into_parts();
    let query = query.unwrap();
    // Every isolated public pair is admissible at this allowance. The complete
    // CSSOM search must still fail because it shares one native request across
    // all three candidates, rather than retrying those isolated pairs.
    for child in &children {
        let CssomRuleData::Keyframe { selectors, .. } = before.rule(child).unwrap().data() else {
            panic!("Keyframe");
        };
        assert!(
            query
                .selectors()
                .matches_normalized(selectors, limits.css)
                .is_ok()
        );
    }
    let Err(CssomError::KeyframeComparison(error)) = store
        .css_keyframes_rule(&root)
        .and_then(|handle| handle.capture(&before))
        .and_then(|capture| capture.find_rule("10%", &limits))
        .map(|found| found.map(|rule| rule.handle().id().clone()))
    else {
        panic!("whole search quota");
    };
    assert_eq!(
        error.cause().kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        error.stage(),
        CssKeyframeSelectorComparisonStage::RightNormalization
    );
    assert_eq!(
        store
            .css_keyframes_rule(&root)
            .and_then(|handle| handle.capture(&before))
            .and_then(|capture| capture.find_rule("10%", &CssomLimits::default()))
            .map(|found| found.map(|rule| rule.handle().id().clone()))
            .unwrap(),
        Some(children[0].clone())
    );
    let mut edit = batch(&store);
    edit.set_disabled(&sheet, true).unwrap();
    assert!(matches!(
        edit.delete_keyframe_rule(&root, "10%"),
        Err(CssomError::KeyframeComparison(_))
    ));
    assert!(matches!(store.commit(edit), Err(CssomError::BatchAborted)));
    let current = store.snapshot();
    assert_eq!(current.revision(), before.revision());
    assert!(!current.sheet(&sheet).unwrap().inputs().disabled);
    assert_eq!(
        current
            .rules(
                current
                    .rule(&root)
                    .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                    .unwrap()
            )
            .unwrap(),
        children
    );
    assert_eq!(current.parent_rule(&children[0]).unwrap(), Some(&root));
    let mut edit = batch(&store);
    assert_eq!(
        edit.delete_keyframe_rule(&root, "30%").unwrap(),
        Some(children[2].clone())
    );
    store.commit(edit).unwrap();
    let final_snapshot = store.snapshot();
    assert_eq!(
        final_snapshot
            .rules(
                final_snapshot
                    .rule(&root)
                    .and_then(|record| record.data().children().ok_or(CssomError::WrongKind))
                    .unwrap()
            )
            .unwrap(),
        &children[..2]
    );
}
