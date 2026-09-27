#![forbid(unsafe_code)]
//! Conditional 5 §2 and §8 admit symbolic named references in supports tests.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#at-ruledef-supports-condition

use surgeist_css::{
    CssComponentValue, CssComponentValues, CssImportConstructionError, CssImportRule,
    CssNamespaceContext, CssRule, CssSerializedOrigin, CssSupportsCondition,
    CssSupportsConditionKind, CssSupportsConstructionError, CssValueOrigin, parse_component_values,
    parse_sheet,
};

fn checked_condition(source: &str) -> CssSupportsCondition {
    CssSupportsCondition::try_from_components(
        parse_component_values(source).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap()
}

fn checked_import(source: &str) -> CssImportRule {
    CssImportRule::try_from_components(
        parse_component_values(source).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap()
}

fn assert_same_parsed_origin(actual: &CssValueOrigin, expected: &CssValueOrigin) {
    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) = (actual, expected)
    else {
        panic!("expected original parsed origins");
    };
    assert!(actual.source().same_snapshot(expected.source()));
    assert_eq!(actual.span(), expected.span());
}

#[test]
fn named_operands_keep_decoded_case_escapes_and_unknown_identity() {
    for (source, decoded) in [
        ("(--)", "--"),
        ("(---)", "---"),
        ("(--Future)", "--Future"),
        (r"(--\46 oo)", "--Foo"),
    ] {
        let checked = checked_condition(source);
        let CssSupportsConditionKind::Named(name) = checked.kind() else {
            panic!("{source} must be a named reference");
        };
        assert_eq!(name.as_str(), decoded);
        assert_eq!(checked.serialize().unwrap().as_css(), source);
        assert_same_parsed_origin(name.component().origin(), name.origin());

        let report = parse_sheet(&format!("@supports {source} {{}}"));
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::Supports(rule)] = report.syntax().rules() else {
            panic!("expected parsed @supports");
        };
        let CssSupportsConditionKind::Named(parsed) = rule.condition().kind() else {
            panic!("parsed {source} must remain symbolic Named");
        };
        assert_eq!(parsed.as_str(), decoded);
        // Parsed rule preludes retain the authored spaces around the operand.
        assert_eq!(
            rule.condition().serialize().unwrap().as_css(),
            format!(" {source} ")
        );
        assert!(parsed.position().is_some());
    }
    assert_ne!(
        match checked_condition("(--Future)").kind() {
            CssSupportsConditionKind::Named(name) => name.as_str().to_owned(),
            _ => unreachable!(),
        },
        match checked_condition("(--future)").kind() {
            CssSupportsConditionKind::Named(name) => name.as_str().to_owned(),
            _ => unreachable!(),
        }
    );
}

#[test]
fn named_operands_compose_with_existing_boolean_supports_grammar() {
    let negated = checked_condition("not (--feature)");
    let CssSupportsConditionKind::Not(child) = negated.kind() else {
        panic!("expected negation");
    };
    assert!(
        matches!(child.kind(), CssSupportsConditionKind::Named(name) if name.as_str() == "--feature")
    );
    assert_eq!(negated.serialize().unwrap().as_css(), "not (--feature)");

    let conjunction = checked_condition("(--feature) and (display:grid)");
    let CssSupportsConditionKind::And(children) = conjunction.kind() else {
        panic!("expected conjunction");
    };
    assert!(
        matches!(children.conditions()[0].kind(), CssSupportsConditionKind::Named(name) if name.as_str() == "--feature")
    );
    assert!(matches!(
        children.conditions()[1].kind(),
        CssSupportsConditionKind::Declaration(_)
    ));
    assert_eq!(
        conjunction.serialize().unwrap().as_css(),
        "(--feature) and (display:grid)"
    );

    let disjunction = checked_condition("(--left) or (--right)");
    let CssSupportsConditionKind::Or(children) = disjunction.kind() else {
        panic!("expected disjunction");
    };
    let names: Vec<_> = children
        .conditions()
        .iter()
        .map(|child| match child.kind() {
            CssSupportsConditionKind::Named(name) => name.as_str(),
            other => panic!("expected named operand: {other:?}"),
        })
        .collect();
    assert_eq!(names, ["--left", "--right"]);

    let report = parse_sheet("@supports (--feature) and (display:grid) {}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("expected parsed @supports");
    };
    assert!(matches!(
        rule.condition().kind(),
        CssSupportsConditionKind::And(_)
    ));
}

#[test]
fn import_bare_name_preserves_function_form_and_original_token_origin() {
    // Conditional 5 §8 shows a bare reference in an import supports() function.
    let source = "@import url(\"nested-styles.css\") supports(--nesting);";
    let parsed = parse_sheet(source);
    assert!(parsed.is_clean(), "{:?}", parsed.diagnostics());
    let [CssRule::Import(parsed)] = parsed.syntax().rules() else {
        panic!("expected parsed import");
    };
    let constructed = checked_import(source);
    for import in [parsed, &constructed] {
        assert!(import.media().is_none());
        let condition = import
            .supports()
            .expect("selected supports clause")
            .condition();
        let CssSupportsConditionKind::Named(name) = condition.kind() else {
            panic!("bare import name must remain symbolic");
        };
        assert_eq!(name.as_str(), "--nesting");
        assert_eq!(condition.components().len(), 1);
        let standalone = condition.serialize().unwrap();
        assert_eq!(standalone.as_css(), "(--nesting)");
        let Some(CssSerializedOrigin::Token(standalone_origin)) = standalone.origin_at(1) else {
            panic!("standalone name token origin");
        };
        assert_same_parsed_origin(standalone_origin, name.origin());

        let import_css = import.serialize().unwrap();
        assert_eq!(import_css.as_css(), source);
        let offset = source.find("--nesting").unwrap();
        let Some(CssSerializedOrigin::Token(import_origin)) = import_css.origin_at(offset) else {
            panic!("import name token origin");
        };
        assert_same_parsed_origin(import_origin, name.origin());
    }
}

#[test]
fn parenthesized_import_name_and_escaped_bare_name_reconstruct_distinct_forms() {
    for (source, standalone, decoded) in [
        (
            "@import 'x' supports((--Feature));",
            "(--Feature)",
            "--Feature",
        ),
        (
            r"@import 'x' supports(--\46 eature);",
            r"(--\46 eature)",
            "--Feature",
        ),
        ("@import 'x' supports(--);", "(--)", "--"),
    ] {
        let parsed = parse_sheet(source);
        assert!(parsed.is_clean(), "{source}: {:?}", parsed.diagnostics());
        let [CssRule::Import(parsed)] = parsed.syntax().rules() else {
            panic!("expected parsed import");
        };
        let constructed = checked_import(source);
        for import in [parsed, &constructed] {
            let condition = import.supports().expect("supports clause").condition();
            let CssSupportsConditionKind::Named(name) = condition.kind() else {
                panic!("{source} must select named operand");
            };
            assert_eq!(name.as_str(), decoded);
            assert_eq!(condition.serialize().unwrap().as_css(), standalone);
            assert_eq!(import.serialize().unwrap().as_css(), source);
            assert!(import.media().is_none());
        }
    }
}

#[test]
fn checked_named_references_reject_eof_recovery_before_grammar() {
    let error = CssSupportsCondition::try_from_components(
        parse_component_values("(--missing").unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        CssSupportsConstructionError::RecoveredInput { .. }
    ));
    assert!(matches!(
        error.origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));

    let recovered = parse_component_values("supports(--missing").unwrap();
    let mut components = vec![
        CssComponentValue::try_token("@import").unwrap(),
        CssComponentValue::try_token(" ").unwrap(),
        CssComponentValue::try_token("'x'").unwrap(),
        CssComponentValue::try_token(" ").unwrap(),
    ];
    components.extend_from_slice(recovered.items());
    components.push(CssComponentValue::try_token(";").unwrap());
    let error = CssImportRule::try_from_components(
        CssComponentValues::try_new(components).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        CssImportConstructionError::RecoveredInput { .. }
    ));
    assert!(matches!(
        error.origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));
}
