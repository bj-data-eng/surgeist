#![forbid(unsafe_code)]

use surgeist_css::{
    CssKnownProperty, CssPropertyNameRef, CssSpecifiedDeclarationBlock, parse_declaration_list_text,
};

#[test]
fn complete_margin_getter_crosses_logical_interference_without_reordering_css_text() {
    let report = parse_declaration_list_text(
        "margin-top:1px; margin-inline-start:7px; margin-right:2px; margin-bottom:3px; margin-left:4px",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let block = CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap();
    assert_eq!(
        block.serialize_cssom().unwrap(),
        "margin-top: 1px; margin-inline-start: 7px; margin-right: 2px; margin-bottom: 3px; margin-left: 4px;",
    );
    assert_eq!(
        block
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::Margin))
            .unwrap(),
        Some("1px 2px 3px 4px".to_owned()),
    );
}

fn block(source: &str) -> CssSpecifiedDeclarationBlock {
    let report = parse_declaration_list_text(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax()).unwrap()
}

#[test]
fn selected_survivors_keep_order_priority_and_original_margin_occurrence() {
    use surgeist_css::{CssImportance, CssSpecifiedDeclarationEntry};
    let original = block("margin:1px 2px 3px 4px");
    let entries = original.entries();
    let selected: Vec<CssSpecifiedDeclarationEntry> = vec![
        entries[3].clone(),
        entries[1].with_importance(CssImportance::Important),
        entries[2].clone(),
    ];
    let edited = CssSpecifiedDeclarationBlock::try_from_entries(&selected).unwrap();
    assert_eq!(
        edited
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::MarginTop))
            .unwrap(),
        None
    );
    assert_eq!(
        edited
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::Margin))
            .unwrap(),
        None
    );
    assert_eq!(
        edited.serialize_cssom().unwrap(),
        "margin-left: 4px; margin-right: 2px !important; margin-bottom: 3px;"
    );
    for (entry, source_entry) in
        edited
            .entries()
            .iter()
            .zip([&entries[3], &entries[1], &entries[2]])
    {
        assert!(entry.source().same_occurrence(source_entry.source()));
        assert_eq!(entry.source().position(), source_entry.source().position());
        assert_eq!(entry.authored_ordinal(), source_entry.authored_ordinal());
        assert_eq!(entry.member_ordinal(), source_entry.member_ordinal());
    }
    assert_eq!(
        original.serialize_cssom().unwrap(),
        "margin: 1px 2px 3px 4px;"
    );
}

#[test]
fn selected_pending_members_retain_origin_and_reject_duplicates_atomically() {
    use surgeist_css::{CssDeclarationBlockErrorKind, CssImportance};
    let original = block("margin-inline:var(--m)");
    let mut selected = original.entries().to_vec();
    selected.reverse();
    let edited = CssSpecifiedDeclarationBlock::try_from_entries(&selected).unwrap();
    assert_eq!(
        edited
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::MarginInline))
            .unwrap()
            .as_deref(),
        Some("var(--m)")
    );
    selected[0] = selected[0].with_importance(CssImportance::Important);
    let mixed = CssSpecifiedDeclarationBlock::try_from_entries(&selected).unwrap();
    assert_eq!(
        mixed
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::MarginInline))
            .unwrap(),
        None
    );
    assert!(
        mixed.entries()[0]
            .source()
            .same_occurrence(original.entries()[0].source())
    );
    let other = block("margin-inline:var(--m)");
    selected[0] = other.entries()[0].clone();
    selected[1] = selected[0].clone();
    assert!(matches!(
        CssSpecifiedDeclarationBlock::try_from_entries(&selected)
            .unwrap_err()
            .kind(),
        CssDeclarationBlockErrorKind::DuplicateTerminal
    ));
    assert_eq!(
        original.serialize_cssom().unwrap(),
        "margin-inline: var(--m);"
    );
    let survivors = &original.entries()[1..];
    assert_eq!(
        CssSpecifiedDeclarationBlock::try_from_entries(survivors)
            .unwrap()
            .entries()
            .len(),
        1
    );
    let report = parse_declaration_list_text("inset:var(--i)");
    assert!(matches!(
        CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax())
            .unwrap_err()
            .kind(),
        CssDeclarationBlockErrorKind::PendingFootprintUndetermined { .. }
    ));
}

#[test]
fn mapping_order_relation_uses_canonical_terminals_and_excludes_unrelated_groups() {
    use surgeist_css::{CssLonghandProperty, CssPropertyKindRef};
    fn terminal(property: CssKnownProperty) -> CssLonghandProperty {
        let CssPropertyKindRef::Longhand(meta) = property.metadata().unwrap().kind() else {
            panic!("longhand");
        };
        meta.property()
    }
    let left = terminal(CssKnownProperty::MarginLeft);
    let start = terminal(CssKnownProperty::MarginInlineStart);
    assert!(left.has_mapping_order_conflict(start));
    assert!(start.has_mapping_order_conflict(left));
    assert!(!left.has_mapping_order_conflict(terminal(CssKnownProperty::PaddingInlineStart)));
    assert!(!left.has_mapping_order_conflict(terminal(CssKnownProperty::MarginRight)));
    assert!(!start.has_mapping_order_conflict(start));
    assert!(!terminal(CssKnownProperty::Opacity).has_mapping_order_conflict(left));
    assert!(
        !terminal(CssKnownProperty::BorderImageSource)
            .has_mapping_order_conflict(terminal(CssKnownProperty::BorderInlineStartColor))
    );
}

#[test]
fn empty_live_nested_declarations_are_retained_and_filtered_from_style_and_media_text() {
    use surgeist_css::{CssEditedGroupPreludeRef, CssEditedRuleView, CssRule, parse_sheet};
    let report = parse_sheet(
        ".host { color:red; & > .child { opacity:0.5 } margin:1px } @media screen { .other { color:blue } }",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(style), CssRule::Media(media)] = report.syntax().rules() else {
        panic!("rules");
    };
    let leading =
        CssSpecifiedDeclarationBlock::try_from_declarations(style.declarations()).unwrap();
    let empty = CssSpecifiedDeclarationBlock::try_from_entries(&[]).unwrap();
    let empty_child = CssEditedRuleView::try_nested_declarations(&empty).unwrap();
    assert_eq!(empty_child.serialize_cssom().unwrap(), "");
    let child = CssEditedRuleView::from_rule(&style.rules()[0]);
    assert!(std::ptr::eq(
        child.parsed_rule().unwrap(),
        &style.rules()[0]
    ));
    let children = [empty_child, child, empty_child];
    let edited = CssEditedRuleView::try_style(style.selectors(), &leading, &children).unwrap();
    assert!(edited.parsed_rule().is_none());
    assert_eq!(
        edited.serialize_cssom().unwrap(),
        ".host {\n  color: red;\n  & > .child { opacity: 0.5; }\n}"
    );
    let children = [empty_child, edited, empty_child];
    let group =
        CssEditedRuleView::try_group(CssEditedGroupPreludeRef::Media(media.query()), &children)
            .unwrap();
    assert_eq!(
        group.serialize_cssom().unwrap(),
        "@media screen {\n  .host {\n  color: red;\n  & > .child { opacity: 0.5; }\n}\n}"
    );
    let only_empty = [empty_child];
    assert_eq!(
        CssEditedRuleView::try_style(style.selectors(), &empty, &only_empty)
            .unwrap()
            .serialize_cssom()
            .unwrap(),
        ".host {\n}"
    );
    assert_eq!(
        CssEditedRuleView::try_group(CssEditedGroupPreludeRef::Media(media.query()), &only_empty)
            .unwrap()
            .serialize_cssom()
            .unwrap(),
        "@media screen {\n\n}"
    );
    assert_eq!(
        CssEditedRuleView::try_style(style.selectors(), &empty, &only_empty)
            .unwrap()
            .to_specified_css()
            .unwrap(),
        ".host { }"
    );
    assert_eq!(
        CssEditedRuleView::try_group(CssEditedGroupPreludeRef::Media(media.query()), &only_empty)
            .unwrap()
            .to_specified_css()
            .unwrap(),
        "@media screen { }"
    );
    assert!(
        style.rules()[0]
            .serialize_cssom()
            .unwrap()
            .contains("opacity: 0.5")
    );
}

#[test]
fn selection_and_edited_graph_limits_are_cumulative_and_retry_preserves_sources() {
    use surgeist_css::{
        CssDeclarationBlockErrorKind, CssEditedGroupPreludeRef, CssEditedRuleView, CssRule,
        CssRuleCssomSerializationErrorKind, CssSpecifiedValueSerializationErrorKind as Kind,
        CssSpecifiedValueSerializationLimits as Limits, parse_sheet,
    };
    let input = block("margin-left:1px; padding-left:2px");
    // Input aggregate + each occurrence/name/component aggregate/dimension = 9.
    let exact = CssSpecifiedDeclarationBlock::try_from_entries_with_limits(
        input.entries(),
        Limits::new(9, 100, 0),
    )
    .unwrap();
    let failure = CssSpecifiedDeclarationBlock::try_from_entries_with_limits(
        input.entries(),
        Limits::new(8, 100, 0),
    )
    .unwrap_err();
    assert!(
        matches!(failure.kind(), CssDeclarationBlockErrorKind::Serialization(error) if error.kind()==Kind::InputNodeLimit)
    );
    assert_eq!(failure.authored_ordinal(), Some(1));
    assert!(
        failure
            .declaration()
            .unwrap()
            .same_occurrence(input.entries()[1].source())
    );
    let projection_failure = CssSpecifiedDeclarationBlock::try_from_entries_with_limits(
        input.entries(),
        Limits::new(100, 2, 0),
    )
    .unwrap_err();
    assert!(
        matches!(projection_failure.kind(), CssDeclarationBlockErrorKind::Serialization(error) if error.kind()==Kind::ProjectionNodeLimit)
    );
    assert_eq!(
        exact.serialize_cssom().unwrap(),
        "margin-left: 1px; padding-left: 2px;"
    );
    let report = parse_sheet("@media screen { .a {} }");
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("media");
    };
    let empty = CssSpecifiedDeclarationBlock::try_from_entries(&[]).unwrap();
    let empty_child = CssEditedRuleView::try_nested_declarations(&empty).unwrap();
    let parsed = CssEditedRuleView::from_rule(&media.rules()[0]);
    let children = [empty_child, parsed, empty_child];
    let group =
        CssEditedRuleView::try_group(CssEditedGroupPreludeRef::Media(media.query()), &children)
            .unwrap();
    let skipped_failure = group
        .serialize_cssom_with_limits(Limits::new(5, 100, 100))
        .unwrap_err();
    assert_eq!(
        skipped_failure.kind(),
        CssRuleCssomSerializationErrorKind::Resource(Kind::InputNodeLimit)
    );
    assert_eq!(skipped_failure.rule_path(), &[0]);
    let expected = "@media screen {\n  .a { }\n}";
    assert_eq!(
        group
            .serialize_cssom_with_limits(Limits::new(100, 100, expected.len()))
            .unwrap(),
        expected
    );
    let failure = group
        .serialize_cssom_with_limits(Limits::new(100, 100, expected.len() - 1))
        .unwrap_err();
    assert_eq!(
        failure.kind(),
        CssRuleCssomSerializationErrorKind::Resource(Kind::ByteLimit)
    );
    assert_eq!(group.serialize_cssom().unwrap(), expected);
    assert_eq!(
        CssEditedRuleView::try_group_with_limits(
            CssEditedGroupPreludeRef::Media(media.query()),
            &children,
            Limits::new(1, 100, 100)
        )
        .unwrap_err()
        .kind(),
        CssRuleCssomSerializationErrorKind::Resource(Kind::InputNodeLimit)
    );
}

#[test]
fn getters_reject_incomplete_mixed_reset_and_incompatible_pending_members() {
    use surgeist_css::{
        CssDeclarationBlockErrorKind, CssSpecifiedValueSerializationErrorKind as Kind,
        CssSpecifiedValueSerializationLimits as Limits,
    };
    let name = CssPropertyNameRef::Known(CssKnownProperty::Margin);
    for source in [
        "margin-top:1px",
        "margin:1px; margin-right:2px!important",
        "border:1px solid red; border-image-source:url(x)",
    ] {
        let input = block(source);
        let property = if source.starts_with("border") {
            CssKnownProperty::Border
        } else {
            CssKnownProperty::Margin
        };
        assert_eq!(
            input
                .property_value(CssPropertyNameRef::Known(property))
                .unwrap(),
            None
        );
    }
    let first = block("margin-inline:var(--m)");
    let second = block("margin-inline:var(--m)");
    let mut selected = first.entries().to_vec();
    selected[1] = second.entries()[1].clone();
    assert_eq!(
        CssSpecifiedDeclarationBlock::try_from_entries(&selected)
            .unwrap()
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::MarginInline))
            .unwrap(),
        None
    );
    let input = block(
        "margin-top:1px; margin-inline-start:7px; margin-right:2px; margin-bottom:3px; margin-left:4px",
    );
    assert!(
        matches!(input.property_value_with_limits(name, Limits::new(100, 1, 100)).unwrap_err().kind(), CssDeclarationBlockErrorKind::Serialization(error) if error.kind()==Kind::ProjectionNodeLimit)
    );
    assert_eq!(
        input
            .property_value_with_limits(name, Limits::new(10000, 10000, 15))
            .unwrap(),
        Some("1px 2px 3px 4px".into())
    );
    assert!(
        matches!(input.property_value_with_limits(name, Limits::new(10000, 10000, 14)).unwrap_err().kind(), CssDeclarationBlockErrorKind::Serialization(error) if error.kind()==Kind::ByteLimit)
    );
}

#[test]
fn selected_keyframe_domain_rejects_priority_and_animation_terminals() {
    use surgeist_css::{
        CssDeclarationBlockErrorKind, CssEditedRuleView, CssImportance,
        CssInvalidKeyframeSourceReason,
    };
    let ordinary = block("opacity:0.5");
    let keyframe =
        CssSpecifiedDeclarationBlock::try_from_keyframe_entries(ordinary.entries()).unwrap();
    assert!(keyframe.is_keyframe());
    assert!(!ordinary.is_keyframe());
    assert_eq!(keyframe.serialize_cssom().unwrap(), "opacity: 0.5;");
    assert!(CssEditedRuleView::try_nested_declarations(&keyframe).is_err());
    let entries = [ordinary.entries()[0].with_importance(CssImportance::Important)];
    assert!(matches!(
        CssSpecifiedDeclarationBlock::try_from_keyframe_entries(&entries)
            .unwrap_err()
            .kind(),
        CssDeclarationBlockErrorKind::InvalidKeyframeSource {
            reason: CssInvalidKeyframeSourceReason::Importance
        }
    ));
    let animation = block("animation-name:a");
    assert!(matches!(
        CssSpecifiedDeclarationBlock::try_from_keyframe_entries(animation.entries())
            .unwrap_err()
            .kind(),
        CssDeclarationBlockErrorKind::InvalidKeyframeSource {
            reason: CssInvalidKeyframeSourceReason::Property
        }
    ));
}

#[test]
fn selected_completed_replacements_and_programmatic_sources_keep_their_own_payloads() {
    use surgeist_css::{
        CssComponentValue, CssComponentValues, CssDeclarationList, CssExpansion, CssImportance,
        expand_declaration, parse_component_values, parse_property_value,
    };
    let original = parse_declaration_list_text("padding-inline:var(--p)");
    let CssExpansion::Pending(pending) = expand_declaration(&original.syntax()[0]).unwrap() else {
        panic!("pending");
    };
    let expansion = CssExpansion::Contributions(
        pending
            .reenter(parse_component_values("9px 10px").unwrap())
            .unwrap(),
    );
    let projected = CssSpecifiedDeclarationBlock::try_from_expansions(&[expansion]).unwrap();
    let survivor =
        CssSpecifiedDeclarationBlock::try_from_entries(&projected.entries()[1..]).unwrap();
    assert_eq!(
        survivor.serialize_cssom().unwrap(),
        "padding-inline-end: 10px;"
    );
    assert!(
        survivor.entries()[0]
            .source()
            .same_occurrence(&original.syntax()[0])
    );
    let supplied = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("red").unwrap()]).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(supplied.position(), None);
    assert!(supplied.parsed_value().is_none());
    let programmatic = CssSpecifiedDeclarationBlock::try_from_declarations(
        &CssDeclarationList::try_new(vec![supplied.clone()]).unwrap(),
    )
    .unwrap();
    let selected = CssSpecifiedDeclarationBlock::try_from_entries(programmatic.entries()).unwrap();
    assert_eq!(selected.serialize_cssom().unwrap(), "color: red;");
    assert!(selected.entries()[0].source().same_occurrence(&supplied));
    assert!(selected.entries()[0].source().position().is_none());
    let empty = CssSpecifiedDeclarationBlock::try_from_entries(&[]).unwrap();
    assert_eq!(empty.serialize_cssom().unwrap(), "");
    assert_eq!(
        empty
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::Margin))
            .unwrap(),
        None
    );
}

#[test]
fn edited_group_views_reuse_preludes_and_retain_typed_literal_unavailability() {
    use surgeist_css::{
        CssEditedGroupPreludeRef as Prelude, CssEditedRuleView as View, CssRule,
        CssRuleCssomFormat, CssRuleCssomSerializationErrorKind as ErrorKind, parse_sheet,
    };
    let report =
        parse_sheet("@supports (display:grid) {} @container (width>1px) {} @layer theme {}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [
        CssRule::Supports(supports),
        CssRule::Container(container),
        CssRule::LayerBlock(layer),
    ] = report.syntax().rules()
    else {
        panic!("groups");
    };
    let supports_view = View::try_group(Prelude::Supports(supports.condition()), &[]).unwrap();
    assert_eq!(
        supports_view.to_specified_css().unwrap(),
        "@supports (display:grid) { }"
    );
    assert_eq!(
        supports_view.serialize_cssom().unwrap(),
        "@supports (display:grid) {\n}"
    );
    for (prelude, expected, format) in [
        (
            Prelude::Container(container.prelude()),
            "@container (width>1px) { }",
            CssRuleCssomFormat::Container,
        ),
        (
            Prelude::Layer(layer.name()),
            "@layer theme { }",
            CssRuleCssomFormat::LayerBlock,
        ),
    ] {
        let view = View::try_group(prelude, &[]).unwrap();
        assert_eq!(view.to_specified_css().unwrap(), expected);
        assert_eq!(
            view.serialize_cssom().unwrap_err().kind(),
            ErrorKind::FormatUnavailable(format)
        );
        assert_eq!(view.to_specified_css().unwrap(), expected);
    }
}

#[test]
fn edited_style_accepts_detached_checked_selector_list_without_rebuilding_a_rule() {
    use surgeist_css::{CssEditedRuleView, CssNamespaceContext, parse_selector_list};
    let report = parse_selector_list("*.edited, #next::before", &CssNamespaceContext::default());
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let selectors = report.syntax().as_ref().unwrap();
    let original = block("margin:1px 2px 3px 4px");
    let edited = CssSpecifiedDeclarationBlock::try_from_entries(&original.entries()[1..]).unwrap();
    let view = CssEditedRuleView::try_style(selectors, &edited, &[]).unwrap();
    assert_eq!(
        view.serialize_cssom().unwrap(),
        ".edited, #next::before { margin-right: 2px; margin-bottom: 3px; margin-left: 4px; }"
    );
    assert!(view.parsed_rule().is_none());
    assert!(view.serialize_cssom().is_ok());
}

#[test]
fn selected_page_supported_property_terminals_keep_page_sources_and_survivors() {
    use surgeist_css::{CssRule, parse_sheet};
    let report = parse_sheet("@page { margin:1px 2px 3px 4px; --note:paper }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Page(page)] = report.syntax().rules() else {
        panic!("Page");
    };
    let original =
        CssSpecifiedDeclarationBlock::try_from_declarations(page.declarations().properties())
            .unwrap();
    let selected =
        CssSpecifiedDeclarationBlock::try_from_page_entries(&original.entries()[1..]).unwrap();
    assert_eq!(
        selected.serialize_cssom().unwrap(),
        "margin-right: 2px; margin-bottom: 3px; margin-left: 4px; --note: paper;"
    );
    assert!(
        selected.entries()[0]
            .source()
            .same_occurrence(&page.declarations().properties()[0])
    );
    assert_eq!(
        selected
            .property_value(CssPropertyNameRef::Known(CssKnownProperty::MarginTop))
            .unwrap(),
        None
    );
}

#[test]
fn edited_scoped_styles_borrow_scope_context_and_preserve_scoped_descendant_origins() {
    use surgeist_css::{CssEditedRuleView as View, CssRule, CssScopedRule, parse_sheet};
    let report =
        parse_sheet("@scope (.host) { & .child { color:red; & > .nested { opacity:0.5 } } }");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("scope");
    };
    let [CssScopedRule::Style(style)] = scope.rules().rules() else {
        panic!("scoped style");
    };
    let leading =
        CssSpecifiedDeclarationBlock::try_from_declarations(style.declarations()).unwrap();
    let empty = CssSpecifiedDeclarationBlock::try_from_entries(&[]).unwrap();
    let empty_child = View::try_nested_declarations(&empty).unwrap();
    let children = [empty_child, View::from_rule(&style.rules()[0]), empty_child];
    let edited = View::try_scoped_style(style.selectors(), &leading, &children).unwrap();
    assert_eq!(
        edited.serialize_cssom().unwrap(),
        "& .child {\n  color: red;\n  & > .nested { opacity: 0.5; }\n}"
    );
    assert!(edited.parsed_scoped_rule().is_none());
    let borrowed = View::from_scoped_rule(&scope.rules().rules()[0]);
    assert!(std::ptr::eq(
        borrowed.parsed_scoped_rule().unwrap(),
        &scope.rules().rules()[0]
    ));
    assert_eq!(
        borrowed.serialize_cssom().unwrap(),
        edited.serialize_cssom().unwrap()
    );
    assert!(borrowed.parsed_rule().is_none());
}
