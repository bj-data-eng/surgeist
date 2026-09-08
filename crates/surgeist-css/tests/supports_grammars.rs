use surgeist_css::{
    CssErrorCode, CssImportance, CssNamespaceConstraint, CssRecoveryAction, CssRule, CssScopedRule,
    CssSelector, CssSupportsConditionKind, parse_sheet,
};

#[test]
fn supports_conditions_and_group_rules_follow_conditional3() {
    let report = parse_sheet("@supports (display: grid) { .x { color: red; } }");

    assert!(report.is_clean(), "diagnostics: {:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 1);
    let CssRule::Supports(rule) = &report.syntax().rules()[0] else {
        panic!("expected supports rule");
    };
    let CssSupportsConditionKind::Declaration(declaration) = rule.condition().kind() else {
        panic!("expected declaration condition");
    };
    assert_eq!(declaration.authored(), "display: grid");
    assert_eq!(declaration.property(), "display");
    assert_eq!(declaration.importance(), CssImportance::Normal);
    assert!(declaration.known().is_some());
    assert_eq!(declaration.position().byte_offset().value(), 11);
    assert!(matches!(rule.rules(), [CssRule::Style(_)]));
    assert_eq!(rule.position().byte_offset().value(), 0);
}

#[test]
fn supports_declaration_tests_preserve_authored_false_syntax_without_diagnostics() {
    let source = concat!(
        "@supports (--theme: red !important) {}",
        "@supports (mystery: 1) {}",
        "@supports (display: definitely-not-a-display) {}",
        "@supports (display:) {}",
        "@supports ( display: grid ) {}",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let declarations = report
        .syntax()
        .rules()
        .iter()
        .map(|rule| {
            let CssRule::Supports(rule) = rule else {
                panic!("expected supports rule");
            };
            let CssSupportsConditionKind::Declaration(declaration) = rule.condition().kind() else {
                panic!("expected declaration condition");
            };
            declaration
        })
        .collect::<Vec<_>>();

    assert_eq!(declarations[0].authored(), "--theme: red !important");
    assert_eq!(declarations[0].property(), "--theme");
    assert_eq!(declarations[0].importance(), CssImportance::Important);
    assert!(declarations[0].known().is_none());
    assert_eq!(declarations[1].authored(), "mystery: 1");
    assert!(declarations[1].known().is_none());
    assert_eq!(
        declarations[2].authored(),
        "display: definitely-not-a-display"
    );
    assert!(declarations[2].known().is_none());
    assert_eq!(declarations[3].authored(), "display:");
    assert!(declarations[3].known().is_none());
    assert_eq!(declarations[4].authored(), " display: grid ");
    assert_eq!(declarations[4].property(), "display");
    assert!(declarations[4].known().is_some());
}

#[test]
fn supports_boolean_operators_require_grouping_when_they_mix() {
    let valid = parse_sheet(concat!(
        "@supports not (display: grid) {}",
        "@supports (display: grid) and (color: red) {}",
        "@supports ((display: grid) or (color: red)) and (width: 1px) {}",
    ));
    assert!(valid.is_clean(), "{:?}", valid.diagnostics());
    let [
        CssRule::Supports(not),
        CssRule::Supports(and),
        CssRule::Supports(grouped),
    ] = valid.syntax().rules()
    else {
        panic!("expected three supports rules");
    };
    assert!(matches!(
        not.condition().kind(),
        CssSupportsConditionKind::Not(_)
    ));
    assert!(matches!(
        and.condition().kind(),
        CssSupportsConditionKind::And(list) if list.conditions().len() == 2
    ));
    let CssSupportsConditionKind::And(list) = grouped.condition().kind() else {
        panic!("expected grouped and condition");
    };
    assert!(matches!(
        list.conditions()[0].kind(),
        CssSupportsConditionKind::Or(or) if or.conditions().len() == 2
    ));

    let malformed = parse_sheet(concat!(
        ".before { color: red; }",
        "@supports (display: grid) and (color: red) or (width: 1px) {}",
        ".after { color: blue; }",
    ));
    assert!(matches!(
        malformed.syntax().rules(),
        [CssRule::Style(_), CssRule::Style(_)]
    ));
    let [diagnostic] = malformed.diagnostics() else {
        panic!("expected one parent diagnostic");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
}

#[test]
fn supports_general_enclosed_and_selector_fallback_preserve_exact_units() {
    let report = parse_sheet(concat!(
        "@supports future( a, nested([x]) ) {}",
        "@supports (future stuff(\")\") [x]) {}",
        "@supports selector(.card > .item:hover) {}",
        "@supports selector(svg|a) {}",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let rules = report.syntax().rules();
    let CssRule::Supports(function) = &rules[0] else {
        panic!("expected function condition");
    };
    assert!(
        matches!(
            function.condition().kind(),
            CssSupportsConditionKind::GeneralEnclosed(value)
                if value.authored() == "future( a, nested([x]) )"
        ),
        "{:?}",
        function.condition().kind()
    );
    let CssRule::Supports(parenthesis) = &rules[1] else {
        panic!("expected parenthesis condition");
    };
    assert!(matches!(
        parenthesis.condition().kind(),
        CssSupportsConditionKind::GeneralEnclosed(value)
            if value.authored() == "(future stuff(\")\") [x])"
    ));
    let CssRule::Supports(selector) = &rules[2] else {
        panic!("expected selector condition");
    };
    assert!(matches!(
        selector.condition().kind(),
        CssSupportsConditionKind::Selector(_)
    ));
    let CssRule::Supports(fallback) = &rules[3] else {
        panic!("expected selector fallback");
    };
    assert!(matches!(
        fallback.condition().kind(),
        CssSupportsConditionKind::GeneralEnclosed(value)
            if value.authored() == "selector(svg|a)"
    ));
}

#[test]
fn supports_selector_uses_active_names_and_falls_back_for_balanced_remainders() {
    let report = parse_sheet(concat!(
        "@namespace svg \"urn:svg\";",
        "@supports selector(svg|a) {}",
        "@supports selector(undeclared|a) {}",
        "@supports selector(svg|a, svg|b) {}",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [
        CssRule::Namespace(_),
        CssRule::Supports(typed),
        CssRule::Supports(undeclared),
        CssRule::Supports(remainder),
    ] = report.syntax().rules()
    else {
        panic!("expected retained namespace and three supports conditions")
    };

    let CssSupportsConditionKind::Selector(CssSelector::Compound(selector)) =
        typed.condition().kind()
    else {
        panic!("expected typed namespace-qualified selector condition")
    };
    assert!(matches!(
        selector
            .type_selector()
            .expect("qualified type selector")
            .namespace(),
        CssNamespaceConstraint::Named(prefix) if prefix.as_str() == "svg"
    ));
    assert!(matches!(
        undeclared.condition().kind(),
        CssSupportsConditionKind::GeneralEnclosed(value)
            if value.authored() == "selector(undeclared|a)"
    ));
    assert!(matches!(
        remainder.condition().kind(),
        CssSupportsConditionKind::GeneralEnclosed(value)
            if value.authored() == "selector(svg|a, svg|b)"
    ));
}

#[test]
fn supports_rules_work_in_conditional_nested_style_and_scoped_contexts() {
    let report = parse_sheet(concat!(
        "@media screen { @supports (display: grid) { .media { color: red; } } }",
        ".host { @supports (display: grid) { color: blue; & .child { width: 1px; } } }",
        "@scope (.root) { @supports selector(.item) { .item { height: 2px; } } }",
        "@supports (color: red) { @container (width > 1px) { .x {} } @layer theme { .y {} } }",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssRule::Media(media) = &report.syntax().rules()[0] else {
        panic!("expected media");
    };
    assert!(matches!(media.rules(), [CssRule::Supports(_)]));
    assert!(matches!(report.syntax().rules()[1], CssRule::Supports(_)));
    let CssRule::Scope(scope) = &report.syntax().rules()[2] else {
        panic!("expected scope");
    };
    assert!(matches!(
        scope.rules().rules(),
        [CssScopedRule::Supports(_)]
    ));
    let CssRule::Supports(group) = &report.syntax().rules()[3] else {
        panic!("expected supports group");
    };
    assert!(matches!(
        group.rules(),
        [CssRule::Container(_), CssRule::LayerBlock(_)]
    ));
}

#[test]
fn invalid_supports_children_do_not_drop_the_valid_parent_or_later_sibling() {
    let report = parse_sheet(concat!(
        "@supports (display: grid) {",
        "@import 'invalid-here.css';",
        "@supports (color: red) { .kept { color: red; } }",
        "@unknown value;",
        "}",
        ".after { color: blue; }",
    ));
    let [CssRule::Supports(parent), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("valid parent and later sibling must survive");
    };
    assert!(matches!(parent.rules(), [CssRule::Supports(_)]));
    assert_eq!(report.diagnostics().len(), 2);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
    );
}

#[test]
fn supports_non_bmp_positions_and_repeated_failures_remain_ordered() {
    let source = concat!(
        "/*🦊*/\n@supports (display: grid) {}",
        "@supports (a:b) and (c:d) or (e:f) {}",
        "@supports (f:g) or (h:i) and (j:k) {}",
        "@supports (color: red) {}",
    );
    let report = parse_sheet(source);
    assert!(
        matches!(
            report.syntax().rules(),
            [CssRule::Supports(_), CssRule::Supports(_)]
        ),
        "rules={:?}; diagnostics={:?}",
        report.syntax().rules(),
        report.diagnostics()
    );
    assert_eq!(report.diagnostics().len(), 2);
    assert!(
        report.diagnostics()[0].span().start().byte_offset().value()
            < report.diagnostics()[1].span().start().byte_offset().value()
    );
    let CssRule::Supports(first) = &report.syntax().rules()[0] else {
        unreachable!()
    };
    assert_eq!(first.position().line().value(), 1);
    assert_eq!(first.position().column().value(), 0);
}
