use surgeist_css::{CssMediaQuery, parse_media_query, parse_media_query_list};

// MQ5 section 3 prefers media conditions/features before general-enclosed,
// whose optional any-value contents come from Syntax 3 section 8.2.
// https://www.w3.org/TR/2026/WD-mediaqueries-5-20260219/#mq-syntax
#[test]
fn arbitrary_enclosures_remain_clean_query_syntax() {
    for source in [
        "future()",
        "future(2px)",
        "()",
        "(2px)",
        "(width >= )",
        "future([x] {y:z} nested(a,b); !)",
    ] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(matches!(report.syntax(), CssMediaQuery::Condition(_)));
    }
}

#[test]
fn arbitrary_enclosures_compose_with_media_operators_and_groups() {
    for source in [
        "not future()",
        "future() and (color)",
        "future() or (color)",
        "(future())",
        "screen and future()",
        "((color) and (hover))",
    ] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(!matches!(report.syntax(), CssMediaQuery::Never(_)));
    }
}

#[test]
fn invalid_range_derivations_can_use_general_enclosed_grammar() {
    for source in [
        "(1px < width > 2px)",
        "(1px <= width >= 2px)",
        "(1px = width = 2px)",
        "(width< =10px)",
        "(width < /**/= 10px)",
        "(aspect-ratio: -1/2)",
    ] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(matches!(report.syntax(), CssMediaQuery::Condition(_)));
    }
}

#[test]
fn malformed_outer_queries_recover_each_list_member() {
    for source in [
        "screen and",
        "future() future()",
        "future() and (color) or (hover)",
    ] {
        let report = parse_media_query_list(&format!("{source}, print"));
        assert!(!report.is_clean(), "{source}");
        assert!(matches!(
            report.syntax().queries(),
            [CssMediaQuery::Never(_), CssMediaQuery::Typed(_)]
        ));
    }
}

#[test]
fn nested_commas_do_not_split_query_members() {
    let report = parse_media_query_list("future(a,b), print");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssMediaQuery::Condition(_), CssMediaQuery::Typed(last)] = report.syntax().queries()
    else {
        panic!("one enclosed member and print");
    };
    assert_eq!(last.media_type(), surgeist_css::CssMediaType::Print);
}

#[test]
fn invalid_any_value_tokens_recover_without_losing_the_next_member() {
    use surgeist_css::{CssComponentValueErrorKind, CssErrorCode, CssValueOrigin, ErrorKind};
    for (source, kind) in [
        (
            "future(]), print",
            CssComponentValueErrorKind::UnmatchedClosingDelimiter,
        ),
        (
            "future(}), print",
            CssComponentValueErrorKind::UnmatchedClosingDelimiter,
        ),
        (
            "future(url(a b)), print",
            CssComponentValueErrorKind::BadUrl,
        ),
        (
            "future(\"x\n), print",
            CssComponentValueErrorKind::BadString,
        ),
    ] {
        let report = parse_media_query_list(source);
        assert!(!report.is_clean(), "{source}");
        let [CssMediaQuery::Never(_), CssMediaQuery::Typed(last)] = report.syntax().queries()
        else {
            panic!("expected rejected enclosure then print: {source}");
        };
        assert_eq!(last.media_type(), surgeist_css::CssMediaType::Print);
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "one lexical diagnostic: {source}: {:?}",
                report.diagnostics()
            );
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidComponentValue
        );
        let ErrorKind::InvalidComponentValue(component) = diagnostic.error().kind() else {
            panic!("original component error");
        };
        assert_eq!(component.kind(), kind);
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("original lexical origin");
        };
        assert_eq!(origin.span().start().byte_offset().value(), 7);
        assert_eq!(diagnostic.error().position(), origin.span().start());
    }
}

#[test]
fn enclosing_a_mixed_operator_body_allows_the_general_enclosed_alternative() {
    for source in [
        "(future() and (color) or (hover))",
        "screen and (future() or (color))",
    ] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(!matches!(report.syntax(), CssMediaQuery::Never(_)));
    }
    let report = parse_media_query("screen and future() or (color)");
    assert!(!report.is_clean());
    assert!(matches!(report.syntax(), CssMediaQuery::Never(_)));
}

#[test]
fn known_features_and_explicit_condition_groups_precede_fallback() {
    use surgeist_css::CssMediaConditionKind;
    for source in ["(width <= 10px)", "(width </**/= 10px)"] {
        let report = parse_media_query(source);
        assert!(report.is_clean());
        let CssMediaQuery::Condition(condition) = report.syntax() else {
            panic!("condition");
        };
        assert!(matches!(
            condition.kind(),
            CssMediaConditionKind::Feature(_)
        ));
    }
    let report = parse_media_query("((color) and (hover))");
    assert!(report.is_clean());
    let CssMediaQuery::Condition(condition) = report.syntax() else {
        panic!("condition");
    };
    let CssMediaConditionKind::Parenthesized(inner) = condition.kind() else {
        panic!("group");
    };
    let CssMediaConditionKind::And(operands) = inner.kind() else {
        panic!("conjunction");
    };
    assert_eq!(operands.conditions().len(), 2);
    assert!(
        operands
            .conditions()
            .iter()
            .all(|v| matches!(v.kind(), CssMediaConditionKind::Feature(_)))
    );
}

#[test]
fn general_enclosed_operands_keep_existing_operator_structure() {
    use surgeist_css::{CssMediaConditionKind, CssMediaType};
    for (source, expected) in [
        ("not future()", 0),
        ("future() and (color)", 1),
        ("future() or (color)", 2),
        ("(future())", 3),
    ] {
        let report = parse_media_query(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let CssMediaQuery::Condition(condition) = report.syntax() else {
            panic!("condition");
        };
        match (expected, condition.kind()) {
            (0, CssMediaConditionKind::Not(_)) | (3, CssMediaConditionKind::Parenthesized(_)) => {}
            (1, CssMediaConditionKind::And(operands))
            | (2, CssMediaConditionKind::Or(operands)) => {
                assert_eq!(operands.conditions().len(), 2);
                assert!(matches!(
                    operands.conditions()[1].kind(),
                    CssMediaConditionKind::Feature(_)
                ));
            }
            _ => panic!("wrong outer condition for {source}"),
        }
    }
    let report = parse_media_query("screen and future()");
    assert!(report.is_clean());
    let CssMediaQuery::Typed(query) = report.syntax() else {
        panic!("typed screen");
    };
    assert_eq!(query.media_type(), CssMediaType::Screen);
    assert!(query.condition().is_some());
}

#[test]
fn media_and_import_rules_share_member_local_admission_and_recovery() {
    use surgeist_css::{CssMediaType, CssRecoveryAction, CssRule, parse_sheet};
    for source in [
        "@media future(), screen and, print {} .after {color:red}",
        "@import \"x\" future(), screen and, print; .after {color:red}",
    ] {
        let report = parse_sheet(source);
        let [first, CssRule::Style(_)] = report.syntax().rules() else {
            panic!("retained rule and sibling: {source}");
        };
        let queries = match first {
            CssRule::Media(rule) => rule.query(),
            CssRule::Import(rule) => rule.media().expect("media tail"),
            _ => panic!("owning rule"),
        };
        let [
            CssMediaQuery::Condition(_),
            CssMediaQuery::Never(_),
            CssMediaQuery::Typed(last),
        ] = queries.queries()
        else {
            panic!("three ordered members: {source}");
        };
        assert_eq!(last.media_type(), CssMediaType::Print);
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "one member diagnostic: {source}: {:?}",
                report.diagnostics()
            );
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::ReplaceMediaQueryWithNever
        );
        let start = source.find("screen and").unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start - 1);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + "screen and".len()
        );
        assert!(diagnostic.error().position().byte_offset().value() >= start);
        assert!(diagnostic.error().position().byte_offset().value() <= start + "screen and".len());
    }
}

#[test]
fn eof_closure_is_retained_with_a_recovery_diagnostic() {
    let report = parse_media_query("future(");
    assert!(matches!(report.syntax(), CssMediaQuery::Condition(_)));
    assert!(!report.is_clean());
    let [diagnostic] = report.diagnostics() else {
        panic!("one retained closure: {:?}", report.diagnostics());
    };
    assert_eq!(
        diagnostic.action(),
        surgeist_css::CssRecoveryAction::RetainWithImplicitClosure
    );
    assert!(report.into_validation_result().is_err());
}
