#![forbid(unsafe_code)]
//! Checked named-definition construction preserves extension names and applies
//! one aggregate resource budget to its rule shell and generic test body.

use surgeist_css::{
    CssComponentValue, CssComponentValueErrorKind, CssComponentValueLimits, CssComponentValueRef,
    CssComponentValues, CssImportance, CssNamedSupportsConstructionError, CssRule,
    CssSerializedOrigin, CssSupportsConditionName, CssSupportsConditionRule, CssSupportsTestBody,
    CssSupportsTestItem, CssValueOrigin, CssValueTokenRef, parse_component_values, parse_sheet,
};

#[test]
fn extension_names_use_decoded_case_sensitive_identifiers_and_original_origin() {
    for name in ["--", "---", "--Theme", "--theme", "-- x", "--x;"] {
        assert_eq!(
            CssSupportsConditionName::try_new(name).unwrap().as_str(),
            name
        );
    }
    for rejected in ["x", "-x", ""] {
        assert!(
            CssSupportsConditionName::try_new(rejected).is_err(),
            "{rejected:?}"
        );
    }
    let source = "@supports-condition --\\54 heme{}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
        panic!("one rule");
    };
    assert_eq!(rule.name().as_str(), "--Theme");
    assert_eq!(
        surgeist_css::CssComponentValues::try_new(vec![rule.name().component().clone()])
            .unwrap()
            .serialize()
            .unwrap()
            .as_css(),
        "--\\54 heme"
    );
    assert!(matches!(rule.name().origin(), CssValueOrigin::Parsed(_)));
}

#[test]
fn checked_body_rejects_invalid_grammar_without_property_support_filtering() {
    let unknown = parse_component_values("future:unknown(foo);@future test{other:value}").unwrap();
    let body = CssSupportsTestBody::try_from_components(unknown).unwrap();
    assert_eq!(body.items().len(), 2);
    let invalid = parse_component_values("missing-colon;").unwrap();
    assert!(matches!(
        CssSupportsTestBody::try_from_components(invalid),
        Err(CssNamedSupportsConstructionError::InvalidBodyGrammar { .. })
    ));
    let recovered = parse_component_values("future:fn(").unwrap();
    assert!(matches!(
        CssSupportsTestBody::try_from_components(recovered),
        Err(CssNamedSupportsConstructionError::RecoveredInput { .. })
    ));
}

#[test]
fn strict_rule_budget_counts_name_shell_body_and_nested_structural_depth() {
    let body =
        CssSupportsTestBody::try_from_components(parse_component_values("x:y;").unwrap()).unwrap();
    let name = CssSupportsConditionName::try_new("--x").unwrap();
    let too_few = CssComponentValueLimits::try_new(256, 7, usize::MAX).unwrap();
    let error = CssSupportsConditionRule::try_new_with_limits(name.clone(), body.clone(), too_few)
        .unwrap_err();
    assert!(
        matches!(error, CssNamedSupportsConstructionError::Component(ref detail) if detail.kind() == CssComponentValueErrorKind::ComponentLimit)
    );
    assert!(
        matches!(error.origin(), CssValueOrigin::Parsed(origin) if origin.span().start().byte_offset().value() == 3)
    );
    let enough = CssComponentValueLimits::try_new(256, 8, usize::MAX).unwrap();
    CssSupportsConditionRule::try_new_with_limits(name.clone(), body, enough).unwrap();

    let whitespace_limit = CssComponentValueLimits::try_new(256, 1, usize::MAX).unwrap();
    let empty =
        CssSupportsTestBody::try_from_components(parse_component_values("").unwrap()).unwrap();
    let error =
        CssSupportsConditionRule::try_new_with_limits(name.clone(), empty, whitespace_limit)
            .unwrap_err();
    assert!(matches!(error.origin(), CssValueOrigin::Programmatic));

    let nested =
        CssSupportsTestBody::try_from_components(parse_component_values("a{b:c}").unwrap())
            .unwrap();
    let too_shallow = CssComponentValueLimits::try_new(1, usize::MAX, usize::MAX).unwrap();
    let error =
        CssSupportsConditionRule::try_new_with_limits(name, nested, too_shallow).unwrap_err();
    assert!(
        matches!(error, CssNamedSupportsConstructionError::Component(ref detail) if detail.kind() == CssComponentValueErrorKind::NestingLimit)
    );
}

#[test]
fn checked_body_counts_generated_semicolon_and_boundary_comment() {
    let input = parse_component_values("x:y").unwrap();
    let too_few = CssComponentValueLimits::try_new(256, 3, usize::MAX).unwrap();
    let error =
        CssSupportsTestBody::try_from_components_with_limits(input.clone(), too_few).unwrap_err();
    assert!(
        matches!(error, CssNamedSupportsConstructionError::Component(ref detail) if detail.kind() == CssComponentValueErrorKind::ComponentLimit)
    );
    assert!(matches!(error.origin(), CssValueOrigin::Programmatic));
    let enough = CssComponentValueLimits::try_new(256, 4, usize::MAX).unwrap();
    CssSupportsTestBody::try_from_components_with_limits(input, enough).unwrap();

    let mut adjacent = parse_component_values("x:").unwrap().items().to_vec();
    adjacent.push(CssComponentValue::try_number("1").unwrap());
    adjacent.push(CssComponentValue::try_ident("e3").unwrap());
    let adjacent = CssComponentValues::try_new(adjacent).unwrap();
    let body = CssSupportsTestBody::try_from_components(adjacent.clone()).unwrap();
    let output = body.serialize().unwrap();
    assert_eq!(output.as_css(), "x:1/**/e3;");
    assert!(
        output
            .segments()
            .iter()
            .any(|segment| matches!(segment.origin(), CssSerializedOrigin::Separator { .. }))
    );
    let too_few = CssComponentValueLimits::try_new(256, 5, usize::MAX).unwrap();
    assert!(matches!(
        CssSupportsTestBody::try_from_components_with_limits(adjacent.clone(), too_few),
        Err(CssNamedSupportsConstructionError::Component(ref detail)) if detail.kind() == CssComponentValueErrorKind::ComponentLimit
    ));
    let enough = CssComponentValueLimits::try_new(256, 6, usize::MAX).unwrap();
    CssSupportsTestBody::try_from_components_with_limits(adjacent, enough).unwrap();
}

#[test]
fn checked_declarations_keep_nonterminal_bangs_in_the_semantic_value() {
    let body = CssSupportsTestBody::try_from_components(
        parse_component_values("future:a !b;future:a !important !important;").unwrap(),
    )
    .unwrap();
    let [CssSupportsTestItem::Declarations(run)] = body.items() else {
        panic!("one run");
    };
    let [first, second] = run.declarations() else {
        panic!("two declarations");
    };
    assert_eq!(first.importance(), CssImportance::Normal);
    assert_eq!(second.importance(), CssImportance::Important);
    for declaration in [first, second] {
        assert!(declaration.value_components().iter().any(|value| matches!(
            value.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Delim('!'))
        )));
    }
    assert!(second.value_components().iter().any(|value| matches!(
        value.view(), CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) if name.eq_ignore_ascii_case("important")
    )));
}

#[test]
fn lexical_serialization_maps_supplied_tokens_and_generated_delimiters() {
    let body =
        CssSupportsTestBody::try_from_components(parse_component_values("x:y").unwrap()).unwrap();
    let rule =
        CssSupportsConditionRule::try_new(CssSupportsConditionName::try_new("--x").unwrap(), body)
            .unwrap();
    let serialized = rule.serialize().unwrap();
    assert_eq!(serialized.as_css(), "@supports-condition --x{x:y;}");
    for delimiter in ["{", "}"] {
        assert!(
            serialized.segments().iter().any(|segment| {
                &serialized.as_css()[segment.byte_range()] == delimiter
                    && matches!(
                        segment.origin(),
                        CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
                    )
            }),
            "generated {delimiter} origin"
        );
    }
    let names = parse_component_values("--x").unwrap();
    let [name] = names.items() else {
        unreachable!()
    };
    assert_eq!(
        CssSupportsConditionName::try_from_component(name.clone())
            .unwrap()
            .as_str(),
        "--x"
    );
    assert!(
        CssSupportsConditionName::try_from_component(
            CssComponentValue::try_ident("plain").unwrap()
        )
        .is_err()
    );
}
