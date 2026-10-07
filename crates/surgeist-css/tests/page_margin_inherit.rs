#![forbid(unsafe_code)]
//! CSS2.1 Page §§13.2.1/13.4 imports the physical margin grammars, including
//! Box §8.3's authored inherit alternative. Inheritance remains symbolic.
use surgeist_css::*;

fn clean<T: Clone>(report: &CssParseReport<T>) {
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(report.clone().into_validation_result().is_ok());
}

fn recovered<T: Clone>(report: &CssParseReport<T>) {
    assert!(!report.is_clean());
    assert!(report.clone().into_validation_result().is_err());
}

fn inherit(
    declaration: &CssDeclaration,
    property: CssKnownProperty,
    importance: CssImportance,
    specified: &str,
) {
    let known = declaration.known().expect("known physical margin");
    assert_eq!(known.property(), property);
    assert!(matches!(
        known.declared_value(),
        CssKnownDeclaredValueRef::Global(CssGlobalKeyword::Inherit)
    ));
    assert_eq!(known.global(), Some(CssGlobalKeyword::Inherit));
    assert_eq!(declaration.importance(), importance);
    assert_eq!(declaration.to_specified_css().unwrap(), specified);
}

fn five_margins(list: &CssDeclarationList) {
    let [margin, top, right, bottom, left] = list.as_slice() else {
        panic!("five selected physical margin occurrences in authored order")
    };
    inherit(
        margin,
        CssKnownProperty::Margin,
        CssImportance::Normal,
        "margin: inherit;",
    );
    inherit(
        top,
        CssKnownProperty::MarginTop,
        CssImportance::Important,
        "margin-top: inherit !important;",
    );
    inherit(
        right,
        CssKnownProperty::MarginRight,
        CssImportance::Normal,
        "margin-right: inherit;",
    );
    inherit(
        bottom,
        CssKnownProperty::MarginBottom,
        CssImportance::Normal,
        "margin-bottom: inherit;",
    );
    inherit(
        left,
        CssKnownProperty::MarginLeft,
        CssImportance::Important,
        "margin-left: inherit !important;",
    );
}

fn span(origin: &CssParsedOrigin, source: &str, start: usize, end: usize) {
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), end);
}

#[test]
fn actual_page_retains_inherit_for_shorthand_and_all_physical_longhands_cleanly() {
    let report = parse_sheet(
        "@page{margin:inherit;margin-top:inherit!important;margin-right:inherit;margin-bottom:inherit;margin-left:inherit!important}",
    );
    clean(&report);
    let [CssRule::Page(page)] = report.syntax().rules() else {
        panic!("one actual page")
    };
    assert_eq!(page.selector(), None);
    assert_eq!(page.position().byte_offset().value(), 0);
    five_margins(page.declarations());
}

#[test]
fn exact_one_page_retains_decoded_keyword_spelling_duplicates_and_priority() {
    let source = r"@page :left{margin-top:INHERIT;margin-left:\69 nherit!important;margin-top:inherit!important}";
    let report = parse_rule(source, &CssNamespaceContext::default());
    clean(&report);
    let Some(CssRule::Page(page)) = report.syntax() else {
        panic!("actual isolated page")
    };
    assert_eq!(page.selector(), Some(CssPageSelector::Left));
    let [first, second, third] = page.declarations().as_slice() else {
        panic!("all occurrences retained")
    };
    inherit(
        first,
        CssKnownProperty::MarginTop,
        CssImportance::Normal,
        "margin-top: inherit;",
    );
    inherit(
        second,
        CssKnownProperty::MarginLeft,
        CssImportance::Important,
        "margin-left: inherit !important;",
    );
    inherit(
        third,
        CssKnownProperty::MarginTop,
        CssImportance::Important,
        "margin-top: inherit !important;",
    );
    for declaration in page.declarations().as_slice() {
        assert_eq!(
            declaration.parsed_value().unwrap().source().as_str(),
            source
        );
    }
}

#[test]
fn genuine_page_body_and_both_context_modes_retain_the_same_five_symbolic_margins() {
    let source = "{margin:inherit;margin-top:inherit!important;margin-right:inherit;margin-bottom:inherit;margin-left:inherit!important}";
    let free = parse_page_block(source);
    clean(&free);
    let fragment = free.syntax().as_ref().unwrap();
    span(fragment.origin(), source, 0, source.len());
    five_margins(fragment.body());
    for mode in [CssParserMode::Standards, CssParserMode::Quirks] {
        let context = CssParserContext::new(mode);
        let report = context.parse_page_block(source);
        clean(&report);
        five_margins(report.syntax().as_ref().unwrap().body());
        for declaration in report.syntax().as_ref().unwrap().body().as_slice() {
            assert_eq!(declaration.parser_context(), context);
            assert!(
                declaration
                    .parsed_value()
                    .unwrap()
                    .source()
                    .same_snapshot(report.syntax().as_ref().unwrap().origin().source())
            );
        }
        if mode == CssParserMode::Standards {
            assert_eq!(report, free);
        }
    }
}

#[test]
fn inherit_occurrences_and_real_body_envelopes_share_original_unicode_source_coordinates() {
    let body_source = " /*é*/\r\n{margin-left:inherit} /*💡*/ ";
    let body_report = parse_page_block(body_source);
    clean(&body_report);
    let fragment = body_report.syntax().as_ref().unwrap();
    span(fragment.origin(), body_source, 9, 30);
    let [declaration] = fragment.body().as_slice() else {
        panic!("one actual inherit")
    };
    inherit(
        declaration,
        CssKnownProperty::MarginLeft,
        CssImportance::Normal,
        "margin-left: inherit;",
    );
    let name = declaration.parsed_name().unwrap();
    let value = declaration.parsed_value().unwrap();
    span(name, body_source, 10, 21);
    span(value, body_source, 22, 29);
    assert_eq!(
        (
            name.span().start().line().value(),
            name.span().start().column().value()
        ),
        (1, 1)
    );
    assert_eq!(
        (
            value.span().start().line().value(),
            value.span().start().column().value()
        ),
        (1, 13)
    );
    assert!(name.source().same_snapshot(fragment.origin().source()));
    assert!(value.source().same_snapshot(fragment.origin().source()));
    let [component] = declaration.value_components().items() else {
        panic!("one authored keyword token")
    };
    assert!(matches!(
        component.view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("inherit"))
    ));
    let CssValueOrigin::Parsed(token) = component.origin() else {
        panic!("actual keyword token origin")
    };
    span(token, body_source, 22, 29);
    assert!(token.source().same_snapshot(fragment.origin().source()));

    let rule_source = " /*é*/\r\n@page{margin-left:inherit} /*💡*/ ";
    let report = parse_rule(rule_source, &CssNamespaceContext::default());
    clean(&report);
    let Some(CssRule::Page(page)) = report.syntax() else {
        panic!("real at-keyword")
    };
    assert_eq!(page.position().byte_offset().value(), 9);
    assert_eq!(
        (
            page.position().line().value(),
            page.position().column().value()
        ),
        (1, 0)
    );
    let [declaration] = page.declarations().as_slice() else {
        panic!("actual page margin")
    };
    let name = declaration.parsed_name().unwrap();
    let value = declaration.parsed_value().unwrap();
    span(name, rule_source, 15, 26);
    span(value, rule_source, 27, 34);
    assert_eq!(
        (
            name.span().start().line().value(),
            name.span().start().column().value()
        ),
        (1, 6)
    );
    assert_eq!(
        (
            value.span().start().line().value(),
            value.span().start().column().value()
        ),
        (1, 18)
    );
    assert!(name.source().same_snapshot(value.source()));
}

#[test]
fn ordinary_and_scoped_ordinary_group_callers_keep_actual_nested_page_inherit() {
    let ns = CssNamespaceContext::default();
    let source = "@media print{@page :right{margin:inherit!important}}";
    let report = parse_sheet(source);
    clean(&report);
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("actual media")
    };
    let [CssRule::Page(page)] = media.rules() else {
        panic!("actual nested page")
    };
    assert_eq!(page.selector(), Some(CssPageSelector::Right));
    assert_eq!(page.position().byte_offset().value(), 13);
    let [declaration] = page.declarations().as_slice() else {
        panic!("nested inherit")
    };
    inherit(
        declaration,
        CssKnownProperty::Margin,
        CssImportance::Important,
        "margin: inherit !important;",
    );
    assert_eq!(
        declaration.parsed_value().unwrap().source().as_str(),
        source
    );

    let source = "@scope (.root){@media print{@page{margin-left:inherit}}}";
    let report = parse_sheet(source);
    clean(&report);
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("actual scope")
    };
    let [CssScopedRule::Media(media)] = scope.rules().rules() else {
        panic!("ordinary scoped media")
    };
    let [CssScopedRule::Page(page)] = media.rules().rules() else {
        panic!("page in scoped ordinary group")
    };
    let [declaration] = page.declarations().as_slice() else {
        panic!("scoped page inherit")
    };
    inherit(
        declaration,
        CssKnownProperty::MarginLeft,
        CssImportance::Normal,
        "margin-left: inherit;",
    );
    assert_eq!(
        declaration.parsed_value().unwrap().source().as_str(),
        source
    );

    let source = "{@page{margin-right:inherit}}";
    let ordinary = parse_group_block(source, &ns);
    clean(&ordinary);
    let [CssRule::Page(page)] = ordinary.syntax().as_ref().unwrap().body().rules() else {
        panic!("direct ordinary group page")
    };
    inherit(
        &page.declarations()[0],
        CssKnownProperty::MarginRight,
        CssImportance::Normal,
        "margin-right: inherit;",
    );
    let scoped = parse_scoped_group_block(source, &ns, CssStyleAncestor::Absent);
    clean(&scoped);
    let [CssScopedRule::Page(page)] = scoped.syntax().as_ref().unwrap().body().rules() else {
        panic!("direct scoped ordinary group page")
    };
    inherit(
        &page.declarations()[0],
        CssKnownProperty::MarginRight,
        CssImportance::Normal,
        "margin-right: inherit;",
    );
}

#[test]
fn inherit_admission_does_not_lift_scope_body_or_style_ancestor_page_placement_restrictions() {
    let ns = CssNamespaceContext::default();
    let source = "{@page{margin:inherit}.after{color:red}}";
    for report in [
        parse_scope_block(source, &ns, CssStyleAncestor::Absent),
        parse_scoped_group_block(source, &ns, CssStyleAncestor::Present),
    ] {
        let [CssScopedRule::Style(style)] = report.syntax().as_ref().unwrap().body().rules() else {
            panic!("excluded page leaves the actual style sibling")
        };
        assert_eq!(
            style.declarations()[0].to_specified_css().unwrap(),
            "color: red;"
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropAtRule)
        );
        recovered(&report);
    }
}

#[test]
fn page_local_recovery_retains_inherit_and_literal_siblings_without_widening_units_or_children() {
    let source = "{margin-top:inherit;margin-right:1em;color:red;@top-left{content:'x'}margin-bottom:-2px;margin-left:inherit!important}";
    let report = parse_page_block(source);
    let [top, bottom, left] = report.syntax().as_ref().unwrap().body().as_slice() else {
        panic!("inherit, literal, inherit around three invalid units")
    };
    inherit(
        top,
        CssKnownProperty::MarginTop,
        CssImportance::Normal,
        "margin-top: inherit;",
    );
    assert_eq!(
        bottom.known().unwrap().property(),
        CssKnownProperty::MarginBottom
    );
    assert_eq!(bottom.to_specified_css().unwrap(), "margin-bottom: -2px;");
    inherit(
        left,
        CssKnownProperty::MarginLeft,
        CssImportance::Important,
        "margin-left: inherit !important;",
    );
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .filter(|d| d.action() == CssRecoveryAction::DropDeclaration)
            .count(),
        2
    );
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .filter(|d| d.action() == CssRecoveryAction::DropAtRule)
            .count(),
        1
    );
    recovered(&report);

    for mode in [CssParserMode::Standards, CssParserMode::Quirks] {
        let report = CssParserContext::new(mode)
            .parse_page_block("{margin:inherit;margin-left:7;margin-top:0}");
        let [margin, top] = report.syntax().as_ref().unwrap().body().as_slice() else {
            panic!("inherit and literal zero survive forbidden bare nonzero length")
        };
        inherit(
            margin,
            CssKnownProperty::Margin,
            CssImportance::Normal,
            "margin: inherit;",
        );
        assert_eq!(top.to_specified_css().unwrap(), "margin-top: 0;");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropDeclaration
                    && matches!(d.error().kind(), ErrorKind::InvalidPropertyValue(detail)
                if detail.property() == CssKnownProperty::MarginLeft))
        );
        recovered(&report);
    }
}

#[test]
fn inherit_is_a_complete_global_alternative_not_a_mixed_margin_component() {
    for invalid in [
        "inherit 1px",
        "1px inherit",
        "inherit inherit",
        "inherit extra",
    ] {
        let source = format!("{{margin-left:inherit;margin:{invalid};margin-bottom:2px}}");
        let report = parse_page_block(&source);
        let [left, bottom] = report.syntax().as_ref().unwrap().body().as_slice() else {
            panic!("valid surrounding siblings")
        };
        inherit(
            left,
            CssKnownProperty::MarginLeft,
            CssImportance::Normal,
            "margin-left: inherit;",
        );
        assert_eq!(bottom.to_specified_css().unwrap(), "margin-bottom: 2px;");
        assert_eq!(report.diagnostics().len(), 1);
        assert!(
            matches!(report.diagnostics()[0].error().kind(), ErrorKind::InvalidPropertyValue(detail)
            if detail.property() == CssKnownProperty::Margin)
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        recovered(&report);
    }
}

#[test]
fn valid_inherit_cannot_escape_failed_genuine_body_or_exact_one_rule_exhaustion() {
    for source in [
        "",
        "margin:inherit",
        "[]",
        "{margin:inherit}{}",
        "{margin:inherit}x",
        "{margin:inherit};",
    ] {
        let report = parse_page_block(source);
        assert!(report.syntax().is_none());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
        recovered(&report);
    }
    for source in [
        "@page{margin:inherit}@page{margin-left:inherit}",
        "@page{margin:inherit}x",
        "@page{margin:inherit};",
    ] {
        let report = parse_rule(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RejectInput)
        );
        recovered(&report);
    }
}

#[test]
fn retained_page_shorthand_expands_symbolically_and_normalization_preserves_the_page_payload() {
    let report = parse_sheet("@page :first{margin:inherit!important}");
    clean(&report);
    let [CssRule::Page(page)] = report.syntax().rules() else {
        panic!("actual selected page")
    };
    let [declaration] = page.declarations().as_slice() else {
        panic!("one shorthand occurrence")
    };
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(declaration).unwrap()
    else {
        panic!("four symbolic physical margin contributions")
    };
    assert_eq!(
        values
            .items()
            .iter()
            .map(|item| item.property().canonical_name())
            .collect::<Vec<_>>(),
        ["margin-top", "margin-right", "margin-bottom", "margin-left"]
    );
    for item in values.items() {
        assert_eq!(
            item.value(),
            CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
        );
        assert!(item.ordinary_value().is_none());
        assert!(item.source().same_occurrence(declaration));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let [CssNormalizedItem::Rule(context)] = normalized.items() else {
        panic!("one normalized page context")
    };
    let CssRuleContextKindRef::Page(page) = context.kind() else {
        panic!("page payload retained")
    };
    assert_eq!(page.selector(), Some(CssPageSelector::First));
    inherit(
        &page.declarations()[0],
        CssKnownProperty::Margin,
        CssImportance::Important,
        "margin: inherit !important;",
    );
}
