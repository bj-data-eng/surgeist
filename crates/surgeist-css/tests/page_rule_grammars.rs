use surgeist_css::{
    CssErrorCode, CssImportance, CssKnownProperty, CssLength, CssLengthUnit, CssPageSelector,
    CssRecoveryAction, CssRule, CssSupportStatus, ErrorKind, feature_metadata, parse_sheet,
};

#[test]
fn page_rules_and_pseudos_retain_valid_authored_structure() {
    let report = parse_sheet(concat!(
        "@import \"print.css\"; ",
        "@page { margin: 1cm; } ",
        "@page :left { margin-left: -12mm !important; } ",
        "@page :right { margin-right: 10%; } ",
        "@page :first { margin-top: auto; margin-bottom: 0; }",
    ));

    assert!(
        report.is_clean(),
        "valid page rules and pseudos should not recover: {:?}",
        report.diagnostics()
    );
    assert_eq!(report.syntax().rules().len(), 5);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.error().code() != CssErrorCode::UnsupportedAtRule)
    );

    let [
        CssRule::Import(_),
        CssRule::Page(default),
        CssRule::Page(left),
        CssRule::Page(right),
        CssRule::Page(first),
    ] = report.syntax().rules()
    else {
        panic!("expected the import and four page rules")
    };
    assert_eq!(default.selector(), None);
    assert_eq!(left.selector(), Some(CssPageSelector::Left));
    assert_eq!(right.selector(), Some(CssPageSelector::Right));
    assert_eq!(first.selector(), Some(CssPageSelector::First));
    assert_eq!(left.declarations().len(), 1);
    assert_eq!(
        left.declarations()[0].known().unwrap().property(),
        CssKnownProperty::MarginLeft
    );
    assert_eq!(
        left.declarations()[0].importance(),
        CssImportance::Important
    );
    assert_eq!(first.declarations().len(), 2);
    assert_eq!(default.position().byte_offset().value(), 21);
}

#[test]
fn page_margin_declarations_accept_only_the_css2_page_domain() {
    let report = parse_sheet(concat!(
        "@page { ",
        "margin: -1px 2% auto 3cm; ",
        "margin-top: -4mm; margin-right: 5in; ",
        "margin-bottom: 6pc; margin-left: 7pt; ",
        "}"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Page(rule)] = report.syntax().rules() else {
        panic!("expected one page rule")
    };
    assert_eq!(rule.declarations().len(), 5);

    let margin = rule.declarations()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap();
    let surgeist_css::CssKnownPropertyValueRef::Margin(margin) = margin else {
        panic!("expected typed margin")
    };
    let edges = margin.i01_subset().unwrap();
    assert!(matches!(edges.top, CssLength::Px(value) if value.value() == -1.0));
    assert!(matches!(edges.right, CssLength::Percent(value) if value.value() == 2.0));
    assert!(matches!(edges.bottom, CssLength::Auto));
    assert!(matches!(
        edges.left,
        CssLength::Dimension(value)
            if value.value() == 3.0 && value.unit() == CssLengthUnit::Cm
    ));
}

#[test]
fn page_context_rejects_relative_modern_and_symbolic_margin_values_and_keeps_siblings() {
    let source = concat!(
        "@page { ",
        "margin-top: 1em; margin-right: 2ex; margin-bottom: 3rem; ",
        "margin-left: 4q; margin: calc(1px + 2%); margin: inherit; ",
        "margin-top: var(--page-margin); margin-bottom: -5px; ",
        "}"
    );
    let report = parse_sheet(source);
    let [CssRule::Page(rule)] = report.syntax().rules() else {
        panic!("expected recovered page rule")
    };
    assert_eq!(rule.declarations().len(), 1);
    assert_eq!(
        rule.declarations()[0].known().unwrap().property(),
        CssKnownProperty::MarginBottom
    );
    assert_eq!(report.diagnostics().len(), 7);
    assert!(report.diagnostics().iter().all(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::InvalidPropertyValue
            && diagnostic.action() == CssRecoveryAction::DropDeclaration
    }));
}

#[test]
fn page_context_distinguishes_known_non_margin_unknown_and_invalid_margin_declarations() {
    let report = parse_sheet(concat!(
        "@page { ",
        "color: red; mystery: 1; margin-top: bogus; ",
        "margin-left: 1cm !important; margin-right: 2%; ",
        "}"
    ));
    let [CssRule::Page(rule)] = report.syntax().rules() else {
        panic!("expected recovered page rule")
    };
    assert_eq!(rule.declarations().len(), 2);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| (diagnostic.error().code(), diagnostic.action()))
            .collect::<Vec<_>>(),
        vec![
            (
                CssErrorCode::InvalidPropertyValue,
                CssRecoveryAction::DropDeclaration
            ),
            (
                CssErrorCode::UnknownProperty,
                CssRecoveryAction::DropDeclaration
            ),
            (
                CssErrorCode::InvalidPropertyValue,
                CssRecoveryAction::DropDeclaration
            ),
        ]
    );
    assert!(matches!(
        report.diagnostics()[0].error().kind(),
        ErrorKind::InvalidPropertyValue(detail) if detail.property() == CssKnownProperty::Color
    ));
    assert!(matches!(
        report.diagnostics()[1].error().kind(),
        ErrorKind::UnknownProperty(detail) if detail.name().as_str() == "mystery"
    ));
    assert!(matches!(
        report.diagnostics()[2].error().kind(),
        ErrorKind::InvalidPropertyValue(detail)
            if detail.property() == CssKnownProperty::MarginTop
    ));
}

#[test]
fn page_rules_enforce_top_level_body_phase_and_nested_placement() {
    let invalid_then_import = parse_sheet(
        "@page :unknown { margin: 1cm; } @import \"still-early.css\"; @page { margin: 2cm; }",
    );
    assert!(matches!(
        invalid_then_import.syntax().rules(),
        [CssRule::Import(_), CssRule::Page(_)]
    ));
    assert_eq!(invalid_then_import.diagnostics().len(), 1);
    assert_eq!(
        invalid_then_import.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );

    let report = parse_sheet(concat!(
        "@import \"valid.css\"; ",
        "@page :first { margin: 1cm; } ",
        "@import \"late.css\"; ",
        "@media print { @page :left { margin: 2cm; } .inside {} } ",
        ".host { @page :right { margin: 3cm; } color: red; }"
    ));
    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::Import(_),
            CssRule::Page(_),
            CssRule::Media(_),
            CssRule::Style(_)
        ]
    ));
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.error().code())
            .collect::<Vec<_>>(),
        vec![
            CssErrorCode::InvalidAtRulePlacement,
            CssErrorCode::InvalidAtRulePlacement,
            CssErrorCode::InvalidAtRulePlacement,
        ]
    );
}

#[test]
fn malformed_page_preludes_and_statement_forms_drop_only_each_rule() {
    let source = concat!(
        ".before {} ",
        "@page named { margin: 1cm; } ",
        "@page :unknown { margin: 1cm; } ",
        "@page :left:right { margin: 1cm; } ",
        "@page :left extra { margin: 1cm; } ",
        "@page :first; ",
        "@page :right { margin: 2cm; } ",
        ".after {}"
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Style(_), CssRule::Page(_), CssRule::Style(_)]
    ));
    assert_eq!(report.diagnostics().len(), 5);
    assert!(report.diagnostics()[..4].iter().all(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::InvalidAtRulePrelude
            && diagnostic.action() == CssRecoveryAction::DropAtRule
    }));
    assert_eq!(
        report.diagnostics()[4].error().code(),
        CssErrorCode::InvalidAtRuleBody
    );
}

#[test]
fn page_body_recovery_rejects_margin_boxes_nested_rules_and_repeated_failures() {
    let report = parse_sheet(concat!(
        "@page { ",
        "margin-top: bogus; ",
        "@top-left { content: \"title\"; } ",
        "@mystery { color: red; } ",
        ".nested { color: blue; } ",
        "margin-left: 1cm; mystery: 1; margin-right: 2cm; ",
        "} .after {}"
    ));
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Page(_), CssRule::Style(_)]
    ));
    let CssRule::Page(page) = &report.syntax().rules()[0] else {
        panic!("expected retained page")
    };
    assert_eq!(page.declarations().len(), 2);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| (diagnostic.error().code(), diagnostic.action()))
            .collect::<Vec<_>>(),
        vec![
            (
                CssErrorCode::InvalidPropertyValue,
                CssRecoveryAction::DropDeclaration
            ),
            (
                CssErrorCode::UnsupportedAtRule,
                CssRecoveryAction::DropAtRule
            ),
            (
                CssErrorCode::InvalidAtRuleBody,
                CssRecoveryAction::DropAtRule
            ),
            (
                CssErrorCode::InvalidAtRuleBody,
                CssRecoveryAction::DropAtRule
            ),
            (
                CssErrorCode::UnknownProperty,
                CssRecoveryAction::DropDeclaration
            ),
        ]
    );
}

#[test]
fn page_eof_recovery_retains_valid_authored_structure_and_later_complete_rules() {
    let unterminated = parse_sheet("@page :first { margin: -1cm");
    assert!(matches!(unterminated.syntax().rules(), [CssRule::Page(_)]));
    assert_eq!(unterminated.diagnostics().len(), 1);
    assert_eq!(
        unterminated.diagnostics()[0].error().code(),
        CssErrorCode::UnexpectedEnd
    );

    let malformed = parse_sheet("@page :left :right");
    assert!(malformed.syntax().rules().is_empty());
    assert_eq!(malformed.diagnostics().len(), 1);
    assert_eq!(
        malformed.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );
}

#[test]
fn page_rule_and_selector_named_metadata_match_retained_behavior() {
    for (id, kind, spelling, production) in [
        (
            "later.rule.page",
            surgeist_css::CssFeatureKind::Rule,
            "@page",
            "page.html#page-box",
        ),
        (
            "official.selector.page-pseudo",
            surgeist_css::CssFeatureKind::Selector,
            ":left|:right|:first",
            "page.html#page-selectors",
        ),
    ] {
        let metadata = feature_metadata(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(metadata.kind(), kind);
        assert_eq!(metadata.spelling(), spelling);
        assert_eq!(metadata.source().id().as_str(), "O-CSS2");
        assert_eq!(metadata.production(), production);
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
        assert_eq!(metadata.recognized_unsupported_code(), None);
    }
}

#[test]
fn page_rules_preserve_non_bmp_source_coordinates_for_rules_and_declarations() {
    let source = "/*😀*/\n@page :left { margin-left: -1cm !important; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Page(rule)] = report.syntax().rules() else {
        panic!("expected one page rule")
    };
    let rule_offset = source.find("@page").unwrap();
    assert_eq!(rule.position().byte_offset().value(), rule_offset);
    assert_eq!(rule.position().line().value(), 1);
    assert_eq!(rule.position().column().value(), 0);

    let declaration = &rule.declarations()[0];
    let declaration_offset = source.find("margin-left").unwrap();
    assert_eq!(
        declaration.position().byte_offset().value(),
        declaration_offset
    );
    assert_eq!(declaration.position().line().value(), 1);
    assert_eq!(
        declaration.position().column().value() as usize,
        source[source.rfind('\n').unwrap() + 1..declaration_offset]
            .encode_utf16()
            .count()
    );
}
