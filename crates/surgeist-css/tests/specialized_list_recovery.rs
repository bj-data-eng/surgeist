use surgeist_css::{
    CssErrorCode, CssMediaConditionKind, CssMediaQuery, CssNamespaceConstraint, CssPseudoClass,
    CssRecoveryAction, CssRule, CssSelector, ErrorKind, parse_sheet,
};

fn style_rule(
    report: &surgeist_css::CssParseReport<surgeist_css::CssSheet>,
) -> &surgeist_css::CssStyleRule {
    report
        .syntax()
        .rules()
        .iter()
        .find_map(|rule| match rule {
            CssRule::Style(rule) => Some(rule),
            _ => None,
        })
        .expect("expected a retained style rule")
}

fn media_rule(
    report: &surgeist_css::CssParseReport<surgeist_css::CssSheet>,
) -> &surgeist_css::CssMediaRule {
    report
        .syntax()
        .rules()
        .iter()
        .find_map(|rule| match rule {
            CssRule::Media(rule) => Some(rule),
            _ => None,
        })
        .expect("expected a retained media rule")
}

fn forgiving_selectors(
    report: &surgeist_css::CssParseReport<surgeist_css::CssSheet>,
) -> &[CssSelector] {
    let pseudo = match style_rule(report).selector() {
        CssSelector::PseudoClass(pseudo) => pseudo,
        CssSelector::Compound(selector) => selector
            .pseudo_classes()
            .first()
            .expect("expected a selector-list pseudo-class"),
        _ => panic!("expected a selector-list pseudo-class"),
    };
    match pseudo {
        CssPseudoClass::Is(selectors) | CssPseudoClass::Where(selectors) => selectors.selectors(),
        _ => panic!("expected :is() or :where()"),
    }
}

fn selector_name(selector: &CssSelector) -> &str {
    match selector {
        CssSelector::Class(name) | CssSelector::Tag(name) | CssSelector::Key(name) => name,
        _ => panic!("expected a simple selector"),
    }
}

fn assert_specialized_diagnostic(
    source: &str,
    diagnostic: &surgeist_css::CssRecoveryDiagnostic,
    code: CssErrorCode,
    action: CssRecoveryAction,
    span_start: usize,
    span_end: usize,
    error_offset: usize,
) {
    assert_eq!(diagnostic.error().code(), code);
    assert_eq!(diagnostic.action(), action);
    assert_eq!(diagnostic.span().start().byte_offset().value(), span_start);
    assert_eq!(diagnostic.span().end().byte_offset().value(), span_end);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        error_offset
    );
    assert_eq!(
        &source[span_start..span_end],
        source
            .get(span_start..span_end)
            .expect("valid expected span")
    );
}

#[test]
fn specialized_list_forgiving_selector_members_drop_independently_in_authored_order() {
    let cases = [
        (":is(???,.a,.b) { color: red; }", 4, 7, vec!["a", "b"]),
        (":is(.a,???,.b) { color: red; }", 7, 10, vec!["a", "b"]),
        (":is(.a,.b,???) { color: red; }", 10, 13, vec!["a", "b"]),
        (":where(???) { color: red; }", 7, 10, vec![]),
    ];

    for (source, start, end, expected) in cases {
        let report = parse_sheet(source);
        let retained = forgiving_selectors(&report)
            .iter()
            .map(selector_name)
            .collect::<Vec<_>>();

        assert_eq!(retained, expected, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_specialized_diagnostic(
            source,
            &report.diagnostics()[0],
            CssErrorCode::InvalidSelector,
            CssRecoveryAction::DropSelectorListItem,
            start,
            end,
            start,
        );
        let ErrorKind::InvalidSelector(detail) = report.diagnostics()[0].error().kind() else {
            panic!("expected selector detail")
        };
        assert_eq!(
            detail.production().expect("selector production").as_str(),
            "baseline.selector.complex"
        );
        assert_eq!(detail.expectation().as_str(), "a supported selector");
    }
}

#[test]
fn forgiving_selector_lists_drop_only_undeclared_namespace_members() {
    let source = concat!(
        "@namespace svg \"urn:svg\";",
        ":is(missing|a,svg|a,.kept) { color: red; }",
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Namespace(_), CssRule::Style(_)]
    ));

    let [qualified, CssSelector::Class(kept)] = forgiving_selectors(&report) else {
        panic!("expected qualified and class members after forgiving recovery")
    };
    let CssSelector::Compound(qualified) = qualified else {
        panic!("expected qualified selector model")
    };
    assert!(matches!(
        qualified
            .type_selector()
            .expect("qualified type selector")
            .namespace(),
        CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
    ));
    assert_eq!(kept, "kept");

    let [diagnostic] = report.diagnostics() else {
        panic!("expected one dropped undeclared-prefix member")
    };
    let start = source.find("missing|a").unwrap();
    assert_specialized_diagnostic(
        source,
        diagnostic,
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropSelectorListItem,
        start,
        start + "missing|a".len(),
        start + "missing|".len(),
    );
}

#[test]
fn specialized_list_empty_forgiving_member_uses_its_delimiting_comma_span() {
    let source = ":is(.a,,.b) { color: red; }";
    let report = parse_sheet(source);

    assert_eq!(
        forgiving_selectors(&report)
            .iter()
            .map(selector_name)
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert_eq!(report.diagnostics().len(), 1);
    assert_specialized_diagnostic(
        source,
        &report.diagnostics()[0],
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropSelectorListItem,
        7,
        8,
        7,
    );
}

#[test]
fn specialized_list_forgiving_recovery_stops_at_balanced_nested_commas() {
    let source = ":is(???(a,b),.ok) { color: red; }";
    let report = parse_sheet(source);

    assert_eq!(
        forgiving_selectors(&report)
            .iter()
            .map(selector_name)
            .collect::<Vec<_>>(),
        ["ok"]
    );
    assert_eq!(report.diagnostics().len(), 1);
    assert_specialized_diagnostic(
        source,
        &report.diagnostics()[0],
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropSelectorListItem,
        4,
        12,
        4,
    );
}

#[test]
fn specialized_list_not_has_nth_and_ordinary_selector_lists_remain_unforgiving() {
    let sources = [
        ":not(.a,???,.b) { color: red; }",
        ":has(.a,???,.b) { color: red; }",
        ":nth-child(2n of .a,???,.b) { color: red; }",
        ".a,???,.b { color: red; }",
    ];

    for source in sources {
        let report = parse_sheet(source);
        assert!(report.syntax().rules().is_empty(), "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropQualifiedRule,
            "{source}"
        );
    }
}

#[test]
fn selectors3_pseudos_preserve_forgiving_and_unforgiving_list_recovery() {
    let source = ":is(:target,:lang(en),:marker,:visited) { color: red; }";
    let report = parse_sheet(source);
    assert!(matches!(
        forgiving_selectors(&report),
        [
            CssSelector::PseudoClass(CssPseudoClass::Target),
            CssSelector::PseudoClass(CssPseudoClass::Lang(range)),
            CssSelector::PseudoClass(CssPseudoClass::Visited),
        ] if range.as_str() == "en"
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one dropped double-colon-only pseudo member")
    };
    let start = source.find(":marker").unwrap();
    assert_specialized_diagnostic(
        source,
        diagnostic,
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropSelectorListItem,
        start,
        start + ":marker".len(),
        start + ":marker".len(),
    );

    let unforgiving = parse_sheet(":not(:target,:marker,:visited) { color: red; }");
    assert!(unforgiving.syntax().rules().is_empty());
    let [diagnostic] = unforgiving.diagnostics() else {
        panic!("expected whole-rule recovery for invalid :not() member")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
}

#[test]
fn specialized_list_media_members_become_never_in_authored_order() {
    let cases = [
        (
            "@media ???,screen,print { .x { color: red; } }",
            6,
            10,
            7,
            0,
        ),
        (
            "@media screen,???,print { .x { color: red; } }",
            14,
            17,
            14,
            1,
        ),
        (
            "@media screen,print,??? { .x { color: red; } }",
            20,
            24,
            20,
            2,
        ),
    ];

    for (source, span_start, span_end, responsible, never_index) in cases {
        let report = parse_sheet(source);
        let queries = media_rule(&report).query().queries();

        assert_eq!(queries.len(), 3, "{source}");
        assert!(queries[never_index].is_guaranteed_false(), "{source}");
        assert!(matches!(queries[never_index], CssMediaQuery::Never(_)));
        assert_eq!(
            queries[never_index].position().byte_offset().value(),
            responsible,
            "{source}"
        );
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_specialized_diagnostic(
            source,
            &report.diagnostics()[0],
            CssErrorCode::InvalidMediaQuery,
            CssRecoveryAction::ReplaceMediaQueryWithNever,
            span_start,
            span_end,
            responsible,
        );
        let ErrorKind::InvalidMediaQuery(detail) = report.diagnostics()[0].error().kind() else {
            panic!("expected media-query detail")
        };
        assert!(detail.feature().is_none());
        assert_eq!(detail.expectation().as_str(), "a supported media query");
        let encountered = detail.encountered().expect("responsible media token");
        assert_eq!(encountered.kind(), surgeist_css::CssTokenKind::Delim);
        assert_eq!(encountered.authored(), "?");
    }
}

#[test]
fn specialized_list_repeated_failures_emit_one_ordered_action_per_member() {
    let selector_source = ":is(???,.ok,???) { color: red; }";
    let selector_report = parse_sheet(selector_source);
    assert_eq!(
        forgiving_selectors(&selector_report)
            .iter()
            .map(selector_name)
            .collect::<Vec<_>>(),
        ["ok"]
    );
    assert_eq!(selector_report.diagnostics().len(), 2);
    for (diagnostic, (start, end)) in selector_report.diagnostics().iter().zip([(4, 7), (12, 15)]) {
        assert_specialized_diagnostic(
            selector_source,
            diagnostic,
            CssErrorCode::InvalidSelector,
            CssRecoveryAction::DropSelectorListItem,
            start,
            end,
            start,
        );
    }

    let media_source = "@media ???,screen,,print,??? { .x { color: red; } }";
    let media_report = parse_sheet(media_source);
    let queries = media_rule(&media_report).query().queries();
    assert_eq!(queries.len(), 5);
    assert!(matches!(queries[0], CssMediaQuery::Never(_)));
    assert!(matches!(queries[1], CssMediaQuery::Typed(_)));
    assert!(matches!(queries[2], CssMediaQuery::Never(_)));
    assert!(matches!(queries[3], CssMediaQuery::Typed(_)));
    assert!(matches!(queries[4], CssMediaQuery::Never(_)));
    assert_eq!(media_report.diagnostics().len(), 3);
    for (diagnostic, (span_start, span_end, responsible)) in media_report
        .diagnostics()
        .iter()
        .zip([(6, 10, 7), (18, 19, 18), (25, 29, 25)])
    {
        assert_specialized_diagnostic(
            media_source,
            diagnostic,
            CssErrorCode::InvalidMediaQuery,
            CssRecoveryAction::ReplaceMediaQueryWithNever,
            span_start,
            span_end,
            responsible,
        );
    }
}

#[test]
fn specialized_list_defined_false_and_repeated_malformed_members_recover_locally() {
    let source = "@media (unknown: yes),???,(width: calc(1px)),,print { .x { color: red; } }";
    let report = parse_sheet(source);
    let queries = media_rule(&report).query().queries();
    assert!(matches!(
        queries,
        [
            CssMediaQuery::Condition(unknown),
            CssMediaQuery::Never(_),
            CssMediaQuery::Condition(value),
            CssMediaQuery::Never(_),
            CssMediaQuery::Typed(_),
        ] if matches!(unknown.kind(), CssMediaConditionKind::DefinedFalse(_))
            && matches!(value.kind(), CssMediaConditionKind::DefinedFalse(_))
    ));
    assert_eq!(report.diagnostics().len(), 2);
    assert!(report.diagnostics().iter().all(|diagnostic| {
        diagnostic.action() == CssRecoveryAction::ReplaceMediaQueryWithNever
    }));
}

#[test]
fn specialized_list_empty_media_member_uses_delimiter_and_end_position() {
    let source = "@media screen,,print { .x { color: red; } }";
    let report = parse_sheet(source);
    let queries = media_rule(&report).query().queries();

    assert_eq!(queries.len(), 3);
    assert!(matches!(queries[1], CssMediaQuery::Never(_)));
    assert_eq!(queries[1].position().byte_offset().value(), 14);
    assert_eq!(report.diagnostics().len(), 1);
    assert_specialized_diagnostic(
        source,
        &report.diagnostics()[0],
        CssErrorCode::InvalidMediaQuery,
        CssRecoveryAction::ReplaceMediaQueryWithNever,
        14,
        15,
        14,
    );
}

#[test]
fn specialized_list_media_recovery_stops_at_balanced_nested_commas() {
    let source = "@media ???(a,b),screen { .x { color: red; } }";
    let report = parse_sheet(source);
    let queries = media_rule(&report).query().queries();

    assert_eq!(queries.len(), 2);
    assert!(matches!(queries[0], CssMediaQuery::Never(_)));
    assert!(matches!(queries[1], CssMediaQuery::Typed(_)));
    assert_eq!(report.diagnostics().len(), 1);
    assert_specialized_diagnostic(
        source,
        &report.diagnostics()[0],
        CssErrorCode::InvalidMediaQuery,
        CssRecoveryAction::ReplaceMediaQueryWithNever,
        6,
        15,
        7,
    );
}

#[test]
fn specialized_list_defined_false_member_keeps_exact_text_and_position_without_recovery() {
    let source = "@media screen,(unknown: yes),print { .x { color: red; } }";
    let report = parse_sheet(source);
    let queries = media_rule(&report).query().queries();
    let member_start = source.find('(').expect("member start");
    let CssMediaQuery::Condition(condition) = &queries[1] else {
        panic!("expected defined-false condition")
    };
    let CssMediaConditionKind::DefinedFalse(defined_false) = condition.kind() else {
        panic!("expected defined-false condition details")
    };
    assert_eq!(condition.position().byte_offset().value(), member_start);
    assert_eq!(defined_false.position(), condition.position());
    assert_eq!(defined_false.as_css(), "(unknown: yes)");
    assert!(report.is_clean());
}

#[test]
fn specialized_list_recovery_propagates_from_import_nested_and_scoped_contexts() {
    let import = parse_sheet("@import \"theme.css\" ???,screen;");
    let [CssRule::Import(import_rule)] = import.syntax().rules() else {
        panic!("expected retained import rule")
    };
    let queries = import_rule.media().expect("import media list").queries();
    assert!(matches!(
        queries,
        [CssMediaQuery::Never(_), CssMediaQuery::Typed(_)]
    ));
    assert_eq!(import.diagnostics().len(), 1);
    assert_eq!(
        import.diagnostics()[0].action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );

    let nested = parse_sheet(".parent { &:is(.ok,???) { color: red; } }");
    assert_eq!(nested.syntax().rules().len(), 1);
    assert_eq!(nested.diagnostics().len(), 1);
    assert_eq!(
        nested.diagnostics()[0].action(),
        CssRecoveryAction::DropSelectorListItem
    );

    let scoped = parse_sheet(
        "@scope (:is(.root,???)) { :where(.kept,???) { color: red; } @media ???,screen {} }",
    );
    assert!(matches!(scoped.syntax().rules(), [CssRule::Scope(_)]));
    assert_eq!(scoped.diagnostics().len(), 3);
    assert_eq!(
        scoped
            .diagnostics()
            .iter()
            .map(surgeist_css::CssRecoveryDiagnostic::action)
            .collect::<Vec<_>>(),
        [
            CssRecoveryAction::DropSelectorListItem,
            CssRecoveryAction::DropSelectorListItem,
            CssRecoveryAction::ReplaceMediaQueryWithNever,
        ]
    );
}

#[test]
fn specialized_list_media_positions_use_utf8_bytes_and_utf16_columns() {
    let source = "@media screen, /*😀*/ ???, print { .x { color: red; } }";
    let report = parse_sheet(source);
    let queries = media_rule(&report).query().queries();
    let member_start = source.find(" /*😀*/").expect("member start");
    let responsible = source.find("???").expect("responsible token");
    let member_end = source[responsible..].find(',').expect("next comma") + responsible;

    assert_eq!(
        queries[0].position().byte_offset().value(),
        source.find("screen").unwrap()
    );
    assert_eq!(queries[1].position().byte_offset().value(), responsible);
    assert_eq!(queries[1].position().line().value(), 0);
    assert_eq!(queries[1].position().column().value(), 22);
    assert_eq!(
        queries[2].position().byte_offset().value(),
        source.find("print").unwrap()
    );
    assert_eq!(report.diagnostics().len(), 1);
    assert_specialized_diagnostic(
        source,
        &report.diagnostics()[0],
        CssErrorCode::InvalidMediaQuery,
        CssRecoveryAction::ReplaceMediaQueryWithNever,
        member_start,
        member_end,
        responsible,
    );
    assert_eq!(report.diagnostics()[0].span().start().column().value(), 14);
    assert_eq!(report.diagnostics()[0].span().end().column().value(), 25);
}

#[test]
fn specialized_list_clean_media_queries_are_positioned_and_never_false_sentinels() {
    let source = "@media screen,(width: 1px) { .x { color: red; } }";
    let report = parse_sheet(source);
    let queries = media_rule(&report).query().queries();

    assert!(report.is_clean());
    assert_eq!(queries.len(), 2);
    assert!(matches!(queries[0], CssMediaQuery::Typed(_)));
    assert!(matches!(queries[1], CssMediaQuery::Condition(_)));
    assert!(!queries[0].is_guaranteed_false());
    assert!(!queries[1].is_guaranteed_false());
    assert_eq!(
        queries[0].position().byte_offset().value(),
        source.find("screen").unwrap()
    );
    assert_eq!(
        queries[1].position().byte_offset().value(),
        source.find('(').unwrap()
    );
}

#[test]
fn specialized_list_media_position_delegation_covers_condition_typed_and_never() {
    let source = "@media /*😀*/ not (width: 1px), only screen, ??? { .x { color: red; } }";
    let report = parse_sheet(source);
    let queries = media_rule(&report).query().queries();
    let condition_offset = source.find("not").expect("condition start");
    let typed_offset = source.find("only").expect("typed-query start");
    let never_offset = source.find("???").expect("malformed-query start");

    let [
        CssMediaQuery::Condition(condition),
        CssMediaQuery::Typed(typed),
        CssMediaQuery::Never(never),
    ] = queries
    else {
        panic!("expected condition, typed, and Never queries in authored order");
    };
    assert!(matches!(condition.kind(), CssMediaConditionKind::Not(_)));
    assert_eq!(condition.position(), queries[0].position());
    assert_eq!(condition.position().byte_offset().value(), condition_offset);
    assert_eq!(condition.position().line().value(), 0);
    assert_eq!(condition.position().column().value(), 14);
    assert_eq!(typed.position(), queries[1].position());
    assert_eq!(typed.position().byte_offset().value(), typed_offset);
    assert_eq!(typed.position().column().value(), 32);
    assert_eq!(never.position(), queries[2].position());
    assert_eq!(never.position().byte_offset().value(), never_offset);
    assert_eq!(never.position().column().value(), 45);
    assert!(!queries[0].is_guaranteed_false());
    assert!(!queries[1].is_guaranteed_false());
    assert!(queries[2].is_guaranteed_false());

    assert_eq!(report.diagnostics().len(), 1);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(diagnostic.error().position(), never.position());
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::ReplaceMediaQueryWithNever
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        never_offset - 1
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        never_offset + 4
    );
}
