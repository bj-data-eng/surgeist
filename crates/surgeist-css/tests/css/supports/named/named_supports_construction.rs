//! Checked named-definition construction preserves extension names and applies
//! one aggregate resource budget to its rule shell and generic test body.

use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValue, CssComponentValueLimits, CssComponentValueRef, CssComponentValues,
    CssSerializedOrigin, CssValueOrigin, CssValueTokenRef,
};
use surgeist_css::declaration::CssImportance;
use surgeist_css::query::supports::{
    CssSupportsConditionName, CssSupportsTestBody, CssSupportsTestItem,
};
use surgeist_css::rule::{CssRule, CssSupportsConditionRule};

#[test]
fn extension_names_use_decoded_case_sensitive_identifiers_and_original_origin() {
    for name in ["--", "---", "--Theme", "--theme", "-- x", "--x;"] {
        assert_eq!(
            CssSupportsConditionName::try_new(name).unwrap().as_str(),
            name
        );
    }
    // `<extension-name>` is a dashed ident; the empty string is no ident.
    for rejected in ["x", "-x"] {
        assert!(
            CssSupportsConditionName::try_new(rejected)
                .as_ref()
                .is_err_and(|error| error.is_invalid_name()
                    && error.origin() == &CssValueOrigin::Programmatic),
            "{rejected:?}"
        );
    }
    let Err(outer) = CssSupportsConditionName::try_new("") else {
        panic!("empty name")
    };
    let Some(error) = outer.component_error() else {
        panic!("empty name")
    };
    assert!(error.is_invalid_identifier(), "{error:?}");
    assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
    let source = "@supports-condition --\\54 heme{}";
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::SupportsCondition(rule)] = report.syntax().rules() else {
        panic!("one rule");
    };
    assert_eq!(rule.name().as_str(), "--Theme");
    assert_eq!(
        surgeist_css::component::CssComponentValues::try_new(
            vec![rule.name().component().clone()],
            Default::default()
        )
        .unwrap()
        .serialize_tokens(Default::default())
        .unwrap()
        .as_css(),
        "--\\54 heme"
    );
    assert!(matches!(rule.name().origin(), CssValueOrigin::Parsed(_)));
}

#[test]
fn checked_body_rejects_invalid_grammar_without_property_support_filtering() {
    let unknown = CssComponentValues::parse(
        &CssParser::default(),
        "future:unknown(foo);@future test{other:value}",
    )
    .unwrap();
    let body = CssSupportsTestBody::try_from_components(unknown, Default::default()).unwrap();
    assert_eq!(body.items().len(), 2);
    let invalid = CssComponentValues::parse(&CssParser::default(), "missing-colon;").unwrap();
    assert!(
        CssSupportsTestBody::try_from_components(invalid, Default::default())
            .as_ref()
            .is_err_and(|error| error.is_invalid_body_grammar())
    );
    let recovered = CssComponentValues::parse(&CssParser::default(), "future:fn(").unwrap();
    assert!(
        CssSupportsTestBody::try_from_components(recovered, Default::default())
            .as_ref()
            .is_err_and(|error| error.is_recovered_input())
    );
}

#[test]
fn strict_rule_budget_counts_name_shell_body_and_nested_structural_depth() {
    let body = CssSupportsTestBody::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "x:y;").unwrap(),
        Default::default(),
    )
    .unwrap();
    let name = CssSupportsConditionName::try_new("--x").unwrap();
    let too_few = CssComponentValueLimits::try_new(256, 7, usize::MAX).unwrap();
    let error = CssSupportsConditionRule::try_new(name.clone(), body.clone(), too_few).unwrap_err();
    assert!(
        error
            .component_error()
            .is_some_and(|detail| detail.is_component_limit())
    );
    assert!(
        matches!(error.origin(), CssValueOrigin::Parsed(origin) if origin.span().start().byte_offset().value() == 3)
    );
    let enough = CssComponentValueLimits::try_new(256, 8, usize::MAX).unwrap();
    CssSupportsConditionRule::try_new(name.clone(), body, enough).unwrap();

    let whitespace_limit = CssComponentValueLimits::try_new(256, 1, usize::MAX).unwrap();
    let empty = CssSupportsTestBody::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "").unwrap(),
        Default::default(),
    )
    .unwrap();
    let error =
        CssSupportsConditionRule::try_new(name.clone(), empty, whitespace_limit).unwrap_err();
    assert!(matches!(error.origin(), CssValueOrigin::Programmatic));

    let nested = CssSupportsTestBody::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "a{b:c}").unwrap(),
        Default::default(),
    )
    .unwrap();
    let too_shallow = CssComponentValueLimits::try_new(1, usize::MAX, usize::MAX).unwrap();
    let error = CssSupportsConditionRule::try_new(name, nested, too_shallow).unwrap_err();
    assert!(
        error
            .component_error()
            .is_some_and(|detail| detail.is_nesting_limit())
    );
}

#[test]
fn checked_body_counts_generated_semicolon_and_boundary_comment() {
    let input = CssComponentValues::parse(&CssParser::default(), "x:y").unwrap();
    let too_few = CssComponentValueLimits::try_new(256, 3, usize::MAX).unwrap();
    let error = CssSupportsTestBody::try_from_components(input.clone(), too_few).unwrap_err();
    assert!(
        error
            .component_error()
            .is_some_and(|detail| detail.is_component_limit())
    );
    assert!(matches!(error.origin(), CssValueOrigin::Programmatic));
    let enough = CssComponentValueLimits::try_new(256, 4, usize::MAX).unwrap();
    CssSupportsTestBody::try_from_components(input, enough).unwrap();

    let mut adjacent = CssComponentValues::parse(&CssParser::default(), "x:")
        .unwrap()
        .items()
        .to_vec();
    adjacent.push(CssComponentValue::try_number("1").unwrap());
    adjacent.push(CssComponentValue::try_ident("e3").unwrap());
    let adjacent = CssComponentValues::try_new(adjacent, Default::default()).unwrap();
    let body =
        CssSupportsTestBody::try_from_components(adjacent.clone(), Default::default()).unwrap();
    let output = body.serialize_tokens(Default::default()).unwrap();
    assert_eq!(output.as_css(), "x:1/**/e3;");
    assert!(
        output
            .segments()
            .iter()
            .any(|segment| matches!(segment.origin(), CssSerializedOrigin::Separator { .. }))
    );
    let too_few = CssComponentValueLimits::try_new(256, 5, usize::MAX).unwrap();
    assert!(
        CssSupportsTestBody::try_from_components(adjacent.clone(), too_few)
            .as_ref()
            .is_err_and(|error| error
                .component_error()
                .is_some_and(|detail| detail.is_component_limit()))
    );
    let enough = CssComponentValueLimits::try_new(256, 6, usize::MAX).unwrap();
    CssSupportsTestBody::try_from_components(adjacent, enough).unwrap();
}

#[test]
fn checked_declarations_keep_nonterminal_bangs_in_the_semantic_value() {
    let body = CssSupportsTestBody::try_from_components(
        CssComponentValues::parse(
            &CssParser::default(),
            "future:a !b;future:a !important !important;",
        )
        .unwrap(),
        Default::default(),
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
    let body = CssSupportsTestBody::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "x:y").unwrap(),
        Default::default(),
    )
    .unwrap();
    let rule = CssSupportsConditionRule::try_new(
        CssSupportsConditionName::try_new("--x").unwrap(),
        body,
        Default::default(),
    )
    .unwrap();
    let serialized = rule.serialize_tokens(Default::default()).unwrap();
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
    let names = CssComponentValues::parse(&CssParser::default(), "--x").unwrap();
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
        .as_ref()
        .is_err_and(
            |error| error.is_invalid_name() && error.origin() == &CssValueOrigin::Programmatic
        )
    );
}
