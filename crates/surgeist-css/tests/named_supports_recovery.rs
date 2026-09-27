#![forbid(unsafe_code)]
//! Conditional Rules 5 §8 retains authored feature tests without rendering them.
//! The selected Syntax block consumer and the catalog's named-test reconciliation
//! require local recovery, with recovered input rejected by checked construction.

use surgeist_css::{
    CssNamedSupportsConstructionError, CssRecoveryAction, CssRule, CssSheet,
    CssSupportsConditionRule, CssSupportsTestBody, CssSupportsTestItem, parse_sheet,
    validate_sheet,
};

fn definition(sheet: &CssSheet) -> &CssSupportsConditionRule {
    sheet
        .rules()
        .iter()
        .find_map(|rule| match rule {
            CssRule::SupportsCondition(definition) => Some(definition),
            _ => None,
        })
        .expect("the named supports definition survives")
}

fn declaration_names(body: &CssSupportsTestBody) -> Vec<&str> {
    body.items()
        .iter()
        .flat_map(|item| match item {
            CssSupportsTestItem::Declarations(run) => run
                .declarations()
                .iter()
                .map(|declaration| declaration.property())
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        })
        .collect()
}

fn assert_recovered_definition(source: &str) {
    let report = parse_sheet(source);
    assert!(!report.is_clean(), "{source:?} must report recovery");
    let definition = definition(report.syntax());
    let origin = definition
        .body()
        .recovery_origin()
        .expect("retained body records its first recovery")
        .clone();
    let failure = validate_sheet(source).expect_err("recovered input is not clean validation");
    assert_eq!(failure.diagnostics(), report.diagnostics());
    assert!(matches!(
        CssSupportsConditionRule::try_new(definition.name().clone(), definition.body().clone()),
        Err(CssNamedSupportsConstructionError::RecoveredInput { origin: error_origin })
            if error_origin == origin
    ));
}

#[test]
fn missing_colon_item_recovers_locally_while_block_form_remains_a_qualified_test() {
    let source = concat!(
        "@supports-condition --feature {",
        " head: yes; broken; future x { child: yes; } tail: yes;",
        "} .after {}",
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::SupportsCondition(_), CssRule::Style(_)]
    ));
    let body = definition(report.syntax()).body();
    let [
        CssSupportsTestItem::Declarations(before),
        CssSupportsTestItem::QualifiedRule(qualified),
        CssSupportsTestItem::Declarations(after),
    ] = body.items()
    else {
        panic!("missing-colon item drops without losing ordered neighbors: {body:?}")
    };
    assert_eq!(before.declarations()[0].property(), "head");
    assert_eq!(after.declarations()[0].property(), "tail");
    assert_eq!(declaration_names(qualified.body()), ["child"]);
    assert_eq!(
        before.declarations()[0]
            .position()
            .expect("parsed head")
            .byte_offset()
            .value(),
        source.find("head: yes").unwrap()
    );
    assert_eq!(
        after.declarations()[0]
            .position()
            .expect("parsed tail")
            .byte_offset()
            .value(),
        source.find("tail: yes").unwrap()
    );
    assert_recovered_definition(source);

    let clean = "@supports-condition --feature { future x { child: yes; } tail: yes; }";
    let report = parse_sheet(clean);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(matches!(
        definition(report.syntax()).body().items(),
        [
            CssSupportsTestItem::QualifiedRule(_),
            CssSupportsTestItem::Declarations(_)
        ]
    ));
    assert!(validate_sheet(clean).is_ok());
}

#[test]
fn bad_string_url_and_nested_function_tokens_drop_only_their_declaration() {
    for source in [
        "@supports-condition --x { head: yes; bad:\"oops\n; tail: yes; } .after {}",
        "@supports-condition --x { head: yes; bad:url(a b); tail: yes; } .after {}",
        "@supports-condition --x { head: yes; bad: fn(url(a b)); tail: yes; } .after {}",
    ] {
        let report = parse_sheet(source);
        assert!(matches!(
            report.syntax().rules(),
            [CssRule::SupportsCondition(_), CssRule::Style(_)]
        ));
        let body = definition(report.syntax()).body();
        assert_eq!(declaration_names(body), ["head", "tail"], "{source:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
        );
        assert_recovered_definition(source);
    }

    for source in [
        "@supports-condition --x { head: yes; bad:\"okay\"; tail: yes; }",
        "@supports-condition --x { head: yes; bad:url(a); tail: yes; }",
        "@supports-condition --x { head: yes; bad: fn(url(a)); tail: yes; }",
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source:?}: {:?}", report.diagnostics());
        assert_eq!(
            declaration_names(definition(report.syntax()).body()),
            ["head", "bad", "tail"]
        );
        assert!(validate_sheet(source).is_ok());
    }
}

#[test]
fn invalid_only_and_trailing_bad_tokens_do_not_become_clean_test_candidates() {
    for (source, retained) in [
        ("@supports-condition --x { bad:url(a b); }", vec![]),
        (
            "@supports-condition --x { head: yes; bad:url(a b); }",
            vec!["head"],
        ),
    ] {
        let report = parse_sheet(source);
        assert_eq!(report.syntax().rules().len(), 1);
        assert_eq!(
            declaration_names(definition(report.syntax()).body()),
            retained
        );
        assert_recovered_definition(source);
    }
}

#[test]
fn a_curly_value_with_a_bad_token_drops_one_declaration_but_a_rule_keeps_good_children() {
    let declaration_source =
        "@supports-condition --x { future: { bad:url(a b); good:1 }; tail:yes; }";
    let declaration = parse_sheet(declaration_source);
    let body = definition(declaration.syntax()).body();
    assert_eq!(declaration_names(body), ["tail"]);
    assert!(
        body.items()
            .iter()
            .all(|item| matches!(item, CssSupportsTestItem::Declarations(_)))
    );
    assert!(
        declaration
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );
    assert_recovered_definition(declaration_source);

    let rule_source = "@supports-condition --x { a { bad:url(a b); good:1 } tail:yes; }";
    let rule = parse_sheet(rule_source);
    let body = definition(rule.syntax()).body();
    let [
        CssSupportsTestItem::QualifiedRule(qualified),
        CssSupportsTestItem::Declarations(tail),
    ] = body.items()
    else {
        panic!("qualified rule and trailing declaration survive: {body:?}")
    };
    assert_eq!(declaration_names(qualified.body()), ["good"]);
    assert_eq!(tail.declarations()[0].property(), "tail");
    assert!(
        rule.diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );
    assert_recovered_definition(rule_source);

    let clean_declaration = "@supports-condition --x { future: { good:1 }; tail:yes; }";
    let report = parse_sheet(clean_declaration);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        declaration_names(definition(report.syntax()).body()),
        ["future", "tail"]
    );
    assert!(matches!(
        definition(report.syntax()).body().items(),
        [CssSupportsTestItem::Declarations(_)]
    ));
}

#[test]
fn standalone_bad_tokens_do_not_resurrect_following_text_or_erase_separate_rules() {
    for (source, names) in [
        (
            "@supports-condition --x { url(a b) plausible:yes; tail:yes; }",
            vec!["tail"],
        ),
        (
            "@supports-condition --x { head:yes; url(a b) }",
            vec!["head"],
        ),
        ("@supports-condition --x { url(a b) }", vec![]),
    ] {
        let report = parse_sheet(source);
        assert_eq!(declaration_names(definition(report.syntax()).body()), names);
        assert_recovered_definition(source);
    }

    let separate = "@supports-condition --x { url(a b); @future { good:1; } tail:yes; }";
    let report = parse_sheet(separate);
    let body = definition(report.syntax()).body();
    let [
        CssSupportsTestItem::AtRule(at_rule),
        CssSupportsTestItem::Declarations(tail),
    ] = body.items()
    else {
        panic!("semicolon separates the bad token from a valid at-rule: {body:?}")
    };
    assert_eq!(at_rule.name(), "future");
    assert_eq!(declaration_names(at_rule.body().unwrap()), ["good"]);
    assert_eq!(tail.declarations()[0].property(), "tail");
    assert_recovered_definition(separate);

    // Without a semicolon, Syntax consumes the bad prefix and following
    // at-keyword as one failed candidate up to its first curly block.
    let combined = "@supports-condition --x { url(a b) @future { good:1; } tail:yes; }";
    let report = parse_sheet(combined);
    let body = definition(report.syntax()).body();
    assert_eq!(declaration_names(body), ["tail"]);
    assert!(
        body.items()
            .iter()
            .all(|item| matches!(item, CssSupportsTestItem::Declarations(_)))
    );
    assert_recovered_definition(combined);
}

#[test]
fn malformed_generic_rule_preludes_drop_the_parent_not_promote_its_children() {
    for (source, action) in [
        (
            "@supports-condition --x { a url(a b) { child:yes; } tail:yes; }",
            CssRecoveryAction::DropQualifiedRule,
        ),
        (
            "@supports-condition --x { @future url(a b) { child:yes; } tail:yes; }",
            CssRecoveryAction::DropAtRule,
        ),
        (
            "@supports-condition --x { @future url(a b); tail:yes; }",
            CssRecoveryAction::DropAtRule,
        ),
    ] {
        let report = parse_sheet(source);
        let body = definition(report.syntax()).body();
        assert_eq!(declaration_names(body), ["tail"], "{source:?}");
        assert!(
            body.items()
                .iter()
                .all(|item| matches!(item, CssSupportsTestItem::Declarations(_)))
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == action)
        );
        assert_recovered_definition(source);
    }
}

#[test]
fn malformed_candidates_in_qualified_and_at_rule_tests_preserve_parents_and_siblings() {
    let source = concat!(
        "@supports-condition --x { before: yes;",
        " & { first: yes; broken:url(a b); last: yes; }",
        " @future { first: yes; broken:url(a b); last: yes; }",
        " after: yes; } .outside {}",
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::SupportsCondition(_), CssRule::Style(_)]
    ));
    let body = definition(report.syntax()).body();
    let [
        CssSupportsTestItem::Declarations(before),
        CssSupportsTestItem::QualifiedRule(qualified),
        CssSupportsTestItem::AtRule(at_rule),
        CssSupportsTestItem::Declarations(after),
    ] = body.items()
    else {
        panic!("both generic parent kinds remain ordered: {body:?}")
    };
    assert_eq!(before.declarations()[0].property(), "before");
    assert_eq!(after.declarations()[0].property(), "after");
    assert_eq!(declaration_names(qualified.body()), ["first", "last"]);
    assert_eq!(at_rule.name(), "future");
    let at_body = at_rule.body().expect("future block remains a block");
    assert_eq!(declaration_names(at_body), ["first", "last"]);
    assert!(qualified.body().recovery_origin().is_some());
    assert!(at_body.recovery_origin().is_some());
    assert_recovered_definition(source);

    let clean = "@supports-condition --x { & { first: yes; } @future { last: yes; } }";
    let report = parse_sheet(clean);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(matches!(
        definition(report.syntax()).body().items(),
        [
            CssSupportsTestItem::QualifiedRule(_),
            CssSupportsTestItem::AtRule(_)
        ]
    ));
}

#[test]
fn unmatched_closer_drops_the_affected_candidate_without_laundering_the_report() {
    let source = "@supports-condition --x { before: yes; bad: ); after: yes; }";
    let report = parse_sheet(source);
    assert_eq!(
        declaration_names(definition(report.syntax()).body()),
        ["before", "after"]
    );
    assert_recovered_definition(source);
}

#[test]
fn eof_rule_block_function_and_comment_recovery_retains_provenance_but_fails_strict_reuse() {
    let rule_eof = "@supports-condition --x { final: yes";
    let report = parse_sheet(rule_eof);
    assert_eq!(
        declaration_names(definition(report.syntax()).body()),
        ["final"]
    );
    assert!(
        report.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        })
    );
    assert_recovered_definition(rule_eof);

    let nested_eof = "@supports-condition --x { & { child: fn(a";
    let report = parse_sheet(nested_eof);
    let [CssSupportsTestItem::QualifiedRule(qualified)] =
        definition(report.syntax()).body().items()
    else {
        panic!("EOF-closed generic block survives")
    };
    assert_eq!(declaration_names(qualified.body()), ["child"]);
    assert!(qualified.body().recovery_origin().is_some());
    assert!(
        report.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        })
    );
    assert_recovered_definition(nested_eof);

    let comment_eof = "@supports-condition --x { final: yes; /*";
    let report = parse_sheet(comment_eof);
    assert_eq!(
        declaration_names(definition(report.syntax()).body()),
        ["final"]
    );
    assert!(
        report.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::IgnoreUnterminatedComment
        })
    );
    assert!(
        report.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        })
    );
    assert_recovered_definition(comment_eof);
}
