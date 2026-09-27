#![forbid(unsafe_code)]
//! Conditional 5 section 8 keeps structurally formed, unknown feature tests.
//! Syntax 3's declaration-first block partition governs ambiguous curly input.

use surgeist_css::{
    CssComponentValueErrorKind, CssImportance, CssRule, CssSupportsTestItem, CssValueTokenRef,
    ErrorKind, parse_sheet,
};

fn body(source: &str) -> surgeist_css::CssSupportsTestBody {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
        panic!("one named definition: {report:?}");
    };
    rule.body().clone()
}

#[test]
fn declaration_first_accepts_sole_curly_value_and_custom_mixture() {
    let body = body("@supports-condition --t{future:{a:b};--x:a{b:c}d;}");
    let [CssSupportsTestItem::Declarations(run)] = body.items() else {
        panic!("one declaration run: {body:?}");
    };
    assert_eq!(run.declarations().len(), 2);
    assert_eq!(run.declarations()[0].property(), "future");
    assert_eq!(run.declarations()[1].property(), "--x");
    assert_eq!(
        run.declarations()[0]
            .value_components()
            .iter()
            .filter(|value| !matches!(
                value.view(),
                surgeist_css::CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
            ))
            .count(),
        1
    );
}

#[test]
fn empty_and_whitespace_only_qualified_preludes_retain_nested_tests() {
    for authored in ["{future:value}", " \t{future:value} "] {
        let source = format!("@supports-condition --x {{{authored}}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
            panic!("one definition");
        };
        let constructed = surgeist_css::CssSupportsTestBody::try_from_components(
            surgeist_css::parse_component_values(authored).unwrap(),
        )
        .unwrap();
        for body in [rule.body(), &constructed] {
            let [CssSupportsTestItem::QualifiedRule(test)] = body.items() else {
                panic!("one generic qualified test: {body:?}");
            };
            assert!(test.prelude().items().iter().all(|value| matches!(
                value.view(),
                surgeist_css::CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                    | surgeist_css::CssComponentValueRef::Comment(_)
            )));
            let [CssSupportsTestItem::Declarations(run)] = test.body().items() else {
                panic!("nested declaration");
            };
            let [declaration] = run.declarations() else {
                panic!("one nested declaration");
            };
            assert_eq!(declaration.property(), "future");
        }
        let canonical = rule.to_specified_css().unwrap();
        assert_eq!(canonical, "@supports-condition --x { { future: value; } }");
        let reparsed = parse_sheet(&canonical);
        assert!(reparsed.is_clean(), "{:?}", reparsed.diagnostics());
        let [CssRule::SupportsCondition(again)] = reparsed.syntax().rules() else {
            panic!("one roundtrip definition");
        };
        assert_eq!(again.to_specified_css().unwrap(), canonical);
    }
}

#[test]
fn bare_double_hyphen_is_not_a_custom_property_for_mixed_curly_values() {
    let source = "@supports-condition --t{--:a{b:c}d;tail:yes}";
    let report = parse_sheet(source);
    assert!(!report.is_clean());
    let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
        panic!("definition retained");
    };
    let [CssSupportsTestItem::Declarations(run)] = rule.body().items() else {
        panic!("only tail retained");
    };
    let [declaration] = run.declarations() else {
        panic!("only tail declaration");
    };
    assert_eq!(declaration.property(), "tail");
}

#[test]
fn terminal_importance_is_removed_before_noncustom_curly_restriction() {
    for authored in [
        "future:{a:b} !important;",
        "future:{a:b} ! /*c*/ IMPORTANT;",
    ] {
        let source = format!("@supports-condition --t{{{authored}}}");
        let parsed = body(&source);
        let constructed = surgeist_css::CssSupportsTestBody::try_from_components(
            surgeist_css::parse_component_values(authored).unwrap(),
        )
        .unwrap();
        for body in [&parsed, &constructed] {
            let [CssSupportsTestItem::Declarations(run)] = body.items() else {
                panic!("one declaration run: {body:?}");
            };
            let [declaration] = run.declarations() else {
                panic!("one declaration");
            };
            assert_eq!(declaration.importance(), CssImportance::Important);
            assert_eq!(
                declaration
                    .value_components()
                    .iter()
                    .filter(|value| !matches!(
                        value.view(),
                        surgeist_css::CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                            | surgeist_css::CssComponentValueRef::Comment(_)
                    ))
                    .count(),
                1
            );
            assert!(matches!(
                declaration
                    .value_components()
                    .iter()
                    .find(|value| !matches!(
                        value.view(),
                        surgeist_css::CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                            | surgeist_css::CssComponentValueRef::Comment(_)
                    ))
                    .unwrap()
                    .view(),
                surgeist_css::CssComponentValueRef::Block(_)
            ));
        }
    }
}

#[test]
fn mixed_noncustom_curly_falls_back_at_first_block_and_leaves_sibling() {
    let body = body("@supports-condition --t{future:x{a:b}tail{next:1}}");
    let [
        CssSupportsTestItem::QualifiedRule(first),
        CssSupportsTestItem::QualifiedRule(second),
    ] = body.items()
    else {
        panic!("two qualified tests: {body:?}");
    };
    assert_eq!(first.body().items().len(), 1);
    assert_eq!(second.body().items().len(), 1);
}

#[test]
fn custom_property_lookalike_without_colon_is_a_qualified_test() {
    let source = "@supports-condition --t{--x no-colon{a:b};good:1}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
        panic!("definition retained");
    };
    let [
        CssSupportsTestItem::QualifiedRule(rule_test),
        CssSupportsTestItem::Declarations(run),
    ] = rule.body().items()
    else {
        panic!("qualified test and declaration retained");
    };
    assert_eq!(rule_test.body().items().len(), 1);
    assert_eq!(run.declarations().len(), 1);
    assert_eq!(run.declarations()[0].property(), "good");
}

#[test]
fn terminal_only_importance_preserves_earlier_bangs_and_unknown_values() {
    let body = body(
        "@supports-condition --t{future:a !b;future:a !important !important;future:thing() !IMPORTANT;}",
    );
    let [CssSupportsTestItem::Declarations(run)] = body.items() else {
        panic!("one run");
    };
    let declarations = run.declarations();
    assert_eq!(declarations.len(), 3);
    assert_eq!(declarations[0].importance(), CssImportance::Normal);
    assert_eq!(declarations[1].importance(), CssImportance::Important);
    assert_eq!(declarations[2].importance(), CssImportance::Important);
    assert!(
        declarations[1]
            .value_components()
            .iter()
            .any(|value| matches!(
                value.view(),
                surgeist_css::CssComponentValueRef::Token(CssValueTokenRef::Delim('!'))
            ))
    );
    assert!(declarations[1].importance_components().is_some());
}

#[test]
fn declaration_runs_flush_around_unknown_at_and_qualified_rules() {
    let body = body("@supports-condition --t{a:1;@future test{x:2}b:3;&{c:4}d:5}");
    assert!(matches!(
        body.items(),
        [
            CssSupportsTestItem::Declarations(_),
            CssSupportsTestItem::AtRule(_),
            CssSupportsTestItem::Declarations(_),
            CssSupportsTestItem::QualifiedRule(_),
            CssSupportsTestItem::Declarations(_),
        ]
    ));
}

#[test]
fn ambient_group_depth_counts_toward_named_test_limit_even_after_bad_token() {
    let nested = |depth: usize, prefix: &str| {
        let body = format!(
            "{prefix}{}leaf:yes;{}",
            "&{".repeat(depth),
            "}".repeat(depth)
        );
        format!("@media all{{@supports-condition --nested{{{body}}}}}")
    };
    let admitted = nested(254, "");
    let report = parse_sheet(&admitted);
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    for source in [nested(255, ""), nested(255, ")")] {
        let report = parse_sheet(&source);
        assert!(!report.is_clean());
        assert!(report.diagnostics().iter().any(|diagnostic| {
            matches!(diagnostic.error().kind(), ErrorKind::InvalidComponentValue(detail) if detail.kind() == CssComponentValueErrorKind::NestingLimit)
                || matches!(diagnostic.error().kind(), ErrorKind::NestingLimit(_))
        }), "expected typed depth failure: {:?}", report.diagnostics());
    }
}
