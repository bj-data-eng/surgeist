//! Checked candidate selection: exact terminal identity, selected priority and
//! survivor occurrence order. Expectations come from the declaration winner
//! contract, independently of authored-list construction or serialization.

use surgeist_css::*;

fn entries(source: &str) -> Vec<CssSpecifiedDeclarationEntry> {
    let report = CssParserContext::new(CssParserMode::Standards)
        .with_svg_glyph_orientation_vertical()
        .parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax())
        .unwrap()
        .entries()
        .to_vec()
}

fn select(candidates: Vec<CssSpecifiedDeclarationEntry>) -> Vec<CssSpecifiedDeclarationEntry> {
    CssSpecifiedDeclarationBlock::try_select_winning_entries_with_limits(
        candidates,
        CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, 0),
    )
    .unwrap()
}

fn text(selected: &[CssSpecifiedDeclarationEntry]) -> String {
    CssSpecifiedDeclarationBlock::try_from_entries(selected)
        .unwrap()
        .serialize_cssom()
        .unwrap()
}

fn same_entry(actual: &CssSpecifiedDeclarationEntry, expected: &CssSpecifiedDeclarationEntry) {
    assert_eq!(actual.property_name(), expected.property_name());
    assert_eq!(actual.importance(), expected.importance());
    assert_eq!(actual.authored_ordinal(), expected.authored_ordinal());
    assert_eq!(actual.member_ordinal(), expected.member_ordinal());
    assert!(actual.source().same_occurrence(expected.source()));
    assert_eq!(actual.source().position(), expected.source().position());
}

fn resource(error: &CssDeclarationBlockError) -> CssSpecifiedValueSerializationErrorKind {
    let CssDeclarationBlockErrorKind::Serialization(cause) = error.kind() else {
        panic!("expected native resource cause: {error:?}");
    };
    cause.kind()
}

#[test]
fn all_priority_pairs_preserve_the_winning_position_among_unrelated_entries() {
    for (earlier, later, expected, positions) in [
        ("", "", "color: red; width: 2px;", [1, 2]),
        (
            "",
            "!important",
            "color: red; width: 2px !important;",
            [1, 2],
        ),
        (
            "!important",
            "",
            "width: 1px !important; color: red;",
            [0, 1],
        ),
        (
            "!important",
            "!important",
            "color: red; width: 2px !important;",
            [1, 2],
        ),
    ] {
        let mut candidates = entries(&format!("width:1px{earlier}"));
        candidates.extend(entries("color:red"));
        candidates.extend(entries(&format!("width:2px{later}")));
        let before = candidates.clone();
        let selected = select(candidates);
        assert_eq!(selected.len(), 2);
        for (actual, index) in selected.iter().zip(positions) {
            same_entry(actual, &before[index]);
        }
        assert_eq!(text(&selected), expected);
    }
}

#[test]
fn canonical_aliases_share_keys_and_svg_retains_its_independent_identity() {
    for (first, second, expected, property) in [
        (
            "page-break-before:always",
            "break-before:avoid",
            "color: red; break-before: avoid;",
            CssPropertyNameRef::Known(CssKnownProperty::BreakBefore),
        ),
        (
            "font-stretch:condensed",
            "font-width:expanded",
            "color: red; font-width: expanded;",
            CssPropertyNameRef::Known(CssKnownProperty::FontWidth),
        ),
        (
            "WIDTH:1px",
            "width:2px",
            "color: red; width: 2px;",
            CssPropertyNameRef::Known(CssKnownProperty::Width),
        ),
        (
            "glyph-orientation-vertical:180deg",
            "glyph-orientation-vertical:90deg",
            "color: red; glyph-orientation-vertical: 90deg;",
            CssPropertyNameRef::SvgGlyphOrientationVertical,
        ),
    ] {
        let mut candidates = entries(first);
        candidates.extend(entries("color:red"));
        candidates.extend(entries(second));
        let last = candidates.last().unwrap().clone();
        let selected = select(candidates);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[1].property_name(), property);
        same_entry(&selected[1], &last);
        assert_eq!(text(&selected), expected);
    }
}

#[test]
fn decoded_custom_keys_are_case_sensitive_and_keep_exact_token_payloads() {
    let mut candidates = entries("--Case:A/**/ B;--case:C");
    candidates.extend(entries("--C\\61 se:D/**/ E"));
    let before = candidates.clone();
    let selected = select(candidates);
    assert_eq!(selected.len(), 2);
    same_entry(&selected[0], &before[1]);
    same_entry(&selected[1], &before[2]);
    assert_eq!(text(&selected), "--case: C; --Case: D/**/ E;");
    assert!(matches!(
        selected[1].value(),
        CssSpecifiedDeclarationValueRef::Custom(_)
    ));
}

#[test]
fn deleted_shorthand_member_stays_absent_and_selected_importance_controls_winners() {
    let bulk = entries("color:red;margin:1px 2px 3px 4px!important");
    // Remove top, demote only right, and supply a later normal replacement.
    let mut candidates = vec![
        bulk[4].clone(),
        bulk[2].with_importance(CssImportance::Normal),
        bulk[3].clone(),
    ];
    candidates.extend(entries("margin-right:9px"));
    let before = candidates.clone();
    let selected = select(candidates);
    for (actual, index) in selected.iter().zip([0, 2, 3]) {
        same_entry(actual, &before[index]);
    }
    assert_eq!(selected.len(), 3);
    assert_eq!(
        text(&selected),
        "margin-left: 4px !important; margin-bottom: 3px !important; margin-right: 9px;"
    );
    assert_eq!(selected[0].authored_ordinal(), 1);
    assert_eq!(selected[0].member_ordinal(), 3);
    assert_eq!(selected[0].source().importance(), CssImportance::Important);
    assert_eq!(
        CssSpecifiedDeclarationBlock::try_from_entries(&selected)
            .unwrap()
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::MarginTop))
            .unwrap(),
        None
    );
    // Promote a selected entry while its original source stays normal.
    let normal = entries("width:1px");
    let promoted = normal[0].with_importance(CssImportance::Important);
    let mut candidates = vec![promoted.clone()];
    candidates.extend(entries("width:2px"));
    let selected = select(candidates);
    same_entry(&selected[0], &promoted);
    assert_eq!(selected[0].source().importance(), CssImportance::Normal);
    assert_eq!(text(&selected), "width: 1px !important;");
}

#[test]
fn pending_bulk_members_reconstruct_only_when_the_original_occurrence_is_complete() {
    let bulk = entries("color:red;margin-inline:var(--m)");
    let selected = select(vec![bulk[2].clone(), bulk[1].clone()]);
    assert_eq!(text(&selected), "margin-inline: var(--m);");
    for (actual, expected) in selected.iter().zip([&bulk[2], &bulk[1]]) {
        same_entry(actual, expected);
        let CssSpecifiedDeclarationValueRef::PendingShorthand(source) = actual.value() else {
            panic!("pending alternative must survive");
        };
        assert!(source.same_occurrence(expected.source()));
    }
    let selected = select(vec![bulk[2].clone()]);
    let block = CssSpecifiedDeclarationBlock::try_from_entries(&selected).unwrap();
    assert_eq!(
        block
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::MarginInline))
            .unwrap(),
        None
    );
    same_entry(&selected[0], &bulk[2]);
}

#[test]
fn colliding_bulk_and_singleton_ordinals_do_not_merge_distinct_pending_occurrences() {
    let first = entries("margin-inline:var(--m);color:red");
    let second = entries("margin-inline:var(--m)");
    assert_eq!(first[0].authored_ordinal(), second[0].authored_ordinal());
    assert!(!first[0].source().same_occurrence(second[0].source()));
    let selected = select(vec![first[0].clone(), second[1].clone()]);
    same_entry(&selected[0], &first[0]);
    same_entry(&selected[1], &second[1]);
    let block = CssSpecifiedDeclarationBlock::try_from_entries(&selected).unwrap();
    assert_eq!(
        block
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::MarginInline))
            .unwrap(),
        None
    );
    let selected = select(vec![first[0].clone(), first[1].clone(), second[0].clone()]);
    same_entry(&selected[0], &first[1]);
    same_entry(&selected[1], &second[0]);
    assert_eq!(
        CssSpecifiedDeclarationBlock::try_from_entries(&selected)
            .unwrap()
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::MarginInline))
            .unwrap(),
        None
    );
}

#[test]
fn symbolic_longhands_and_selected_svg_priority_keep_their_original_alternatives() {
    let original = entries("width:var(--w);glyph-orientation-vertical:90deg");
    let promoted = original[1].with_importance(CssImportance::Important);
    let mut candidates = vec![original[0].clone(), promoted.clone()];
    candidates.extend(entries("glyph-orientation-vertical:auto"));
    let selected = select(candidates);
    assert_eq!(selected.len(), 2);
    same_entry(&selected[0], &original[0]);
    same_entry(&selected[1], &promoted);
    assert!(matches!(
        selected[0].value(),
        CssSpecifiedDeclarationValueRef::UnresolvedLonghand(_)
    ));
    assert!(matches!(
        selected[1].value(),
        CssSpecifiedDeclarationValueRef::SvgGlyphOrientationVertical(_)
    ));
    assert_eq!(selected[1].source().importance(), CssImportance::Normal);
    assert_eq!(
        text(&selected),
        "width: var(--w); glyph-orientation-vertical: 90deg !important;"
    );
}

#[test]
fn selection_does_not_certify_domains_or_relax_strict_duplicate_validation() {
    let source = entries("opacity:.5!important;animation-name:k");
    let duplicate = vec![source[0].clone(), source[0].clone()];
    assert!(matches!(
        CssSpecifiedDeclarationBlock::try_from_entries(&duplicate)
            .unwrap_err()
            .kind(),
        CssDeclarationBlockErrorKind::DuplicateTerminal
    ));
    let selected = select(duplicate);
    assert_eq!(selected.len(), 1);
    assert!(matches!(
        CssSpecifiedDeclarationBlock::try_from_keyframe_entries(&selected)
            .unwrap_err()
            .kind(),
        CssDeclarationBlockErrorKind::InvalidKeyframeSource {
            reason: CssInvalidKeyframeSourceReason::Importance
        }
    ));
    let selected = select(vec![source[1].clone()]);
    assert!(matches!(
        CssSpecifiedDeclarationBlock::try_from_keyframe_entries(&selected)
            .unwrap_err()
            .kind(),
        CssDeclarationBlockErrorKind::InvalidKeyframeSource {
            reason: CssInvalidKeyframeSourceReason::Property
        }
    ));
    for constructor in [
        CssSpecifiedDeclarationBlock::try_from_page_entries,
        CssSpecifiedDeclarationBlock::try_from_margin_entries,
    ] {
        assert!(matches!(
            constructor(&selected).unwrap_err().kind(),
            CssDeclarationBlockErrorKind::InvalidPageSource
        ));
    }
    same_entry(&selected[0], &source[1]);
}

#[test]
fn empty_selection_admits_only_aggregate_work_and_emits_no_bytes() {
    let exact = CssSpecifiedValueSerializationLimits::new(1, 1, 0);
    assert!(
        CssSpecifiedDeclarationBlock::try_select_winning_entries_with_limits(vec![], exact)
            .unwrap()
            .is_empty()
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 0),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 0),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        let error =
            CssSpecifiedDeclarationBlock::try_select_winning_entries_with_limits(vec![], limits)
                .unwrap_err();
        assert_eq!(resource(&error), kind);
        assert!(error.declaration().is_none());
        assert_eq!(error.authored_ordinal(), None);
        assert_eq!(error.member_ordinal(), None);
    }
}

#[test]
fn losing_candidates_consume_the_whole_budget_with_real_entry_attribution_and_retry() {
    let bulk = entries("color:red;margin:1px 2px 3px 4px");
    let winner = bulk[4].with_importance(CssImportance::Important);
    let loser = bulk[4].clone();
    // One aggregate, nine entry/key pairs and nine candidate visits: I19/P10.
    let mut candidates = vec![winner.clone()];
    candidates.extend(std::iter::repeat_n(loser.clone(), 8));
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(18, 10, 0),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(19, 9, 0),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        let error = CssSpecifiedDeclarationBlock::try_select_winning_entries_with_limits(
            candidates.clone(),
            limits,
        )
        .unwrap_err();
        assert_eq!(resource(&error), kind);
        assert!(error.declaration().unwrap().same_occurrence(loser.source()));
        assert_eq!(error.authored_ordinal(), Some(1));
        assert_eq!(error.member_ordinal(), Some(3));
    }
    let selected = CssSpecifiedDeclarationBlock::try_select_winning_entries_with_limits(
        candidates,
        CssSpecifiedValueSerializationLimits::new(19, 10, 0),
    )
    .unwrap();
    assert_eq!(selected.len(), 1);
    same_entry(&selected[0], &winner);
    assert_eq!(text(&selected), "margin-left: 4px !important;");
    assert_eq!(text(&bulk), "color: red; margin: 1px 2px 3px 4px;");
}

#[test]
fn custom_name_character_count_does_not_change_structural_selection_allowance() {
    for name in ["--x".to_owned(), format!("--{}", "x".repeat(2048))] {
        let candidates = entries(&format!("{name}:var(--v)/**/ retained"));
        let original = candidates[0].clone();
        let selected = CssSpecifiedDeclarationBlock::try_select_winning_entries_with_limits(
            candidates,
            CssSpecifiedValueSerializationLimits::new(3, 2, 0),
        )
        .unwrap();
        same_entry(&selected[0], &original);
    }
}
