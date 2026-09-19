#![forbid(unsafe_code)]
//! Conditional 5 section 5.4 admits general-enclosed query operands. MQ4
//! section 3 permits empty any-value contents; unknown syntax stays symbolic.
use surgeist_css::{
    CssComponentValue, CssComponentValueRef, CssComponentValues,
    CssContainerCondition as Condition, CssContainerConditionKind as Kind,
    CssContainerGeneralEnclosed, CssGeneralEnclosed, CssRecoveryAction, CssRule,
    CssSerializedOrigin, CssValueOrigin, parse_component_values, parse_sheet, validate_sheet,
};

fn condition(query: &str) -> Condition {
    let report = parse_sheet(&format!("@container {query} {{ .x {{ color:red }} }}"));
    assert!(report.is_clean(), "{query}: {:?}", report.diagnostics());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("container")
    };
    assert!(matches!(rule.rules(), [CssRule::Style(_)]));
    rule.prelude().entries()[0].query().unwrap().clone()
}

fn opaque(value: &Condition) -> &CssContainerGeneralEnclosed {
    let Kind::GeneralEnclosed(value) = value.kind() else {
        panic!("opaque query operand: {value:?}")
    };
    value
}

#[test]
fn nested_negated_unknown_operands_keep_boolean_structure_and_the_original_snapshot() {
    let source = "/*😀*/ @container not ((a) and (not (b))) { .x { color:red } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(validate_sheet(source).unwrap(), *report.syntax());
    let [CssRule::Container(rule)] = report.syntax().rules() else {
        panic!("container")
    };
    let Kind::Not(outer) = rule.prelude().entries()[0].query().unwrap().kind() else {
        panic!("outer not")
    };
    let Kind::Parenthesized(outer) = outer.kind() else {
        panic!("grouped outer condition")
    };
    let Kind::And(list) = outer.kind() else {
        panic!("and")
    };
    let [a, grouped_b] = list.conditions() else {
        panic!("two operands")
    };
    let Kind::Parenthesized(grouped_b) = grouped_b.kind() else {
        panic!("grouped negation")
    };
    let Kind::Not(b) = grouped_b.kind() else {
        panic!("negation")
    };
    let a = opaque(a);
    let b = opaque(b);
    for (value, authored) in [(a, "(a)"), (b, "(b)")] {
        assert_eq!(value.authored(), Some(authored));
        assert_eq!(value.serialize().unwrap().as_css(), authored);
        let CssValueOrigin::Parsed(origin) = value.origin() else {
            panic!("original opener")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            source.find(authored).unwrap()
        );
        let CssComponentValueRef::Block(block) = value.component().view() else {
            panic!("block")
        };
        let CssValueOrigin::Parsed(token) = block.values().items()[0].origin() else {
            panic!("original ident")
        };
        assert!(token.source().same_snapshot(origin.source()));
        assert_eq!(
            token.span().start().byte_offset().value(),
            source.find(authored).unwrap() + 1
        );
    }
    let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) = (a.origin(), b.origin()) else {
        panic!("parsed")
    };
    assert!(a.source().same_snapshot(b.source()));
}

#[test]
fn recognized_size_and_style_branches_take_precedence_over_opaque_fallback() {
    assert!(matches!(
        condition("(width > 1px)").kind(),
        Kind::Feature(_)
    ));
    assert!(matches!(condition("style(--theme)").kind(), Kind::Style(_)));
    let mixed = condition("(width > 1px) and Future() and style(--theme)");
    let Kind::And(list) = mixed.kind() else {
        panic!("conjunction")
    };
    let [feature, opaque_condition, style] = list.conditions() else {
        panic!("three operands")
    };
    assert!(matches!(feature.kind(), Kind::Feature(_)));
    assert!(matches!(opaque_condition.kind(), Kind::GeneralEnclosed(_)));
    assert!(matches!(style.kind(), Kind::Style(_)));
    for query in [
        "style(color:red)",
        "(unknown-size > 1px)",
        "(width: nonsense)",
    ] {
        assert_eq!(opaque(&condition(query)).authored(), Some(query));
    }
}

#[test]
fn empty_comments_escaped_functions_and_nested_components_remain_lexical_operands() {
    for query in [
        "()",
        "(/**/)",
        "Future()",
        "Future(/**/)",
        r"F\75ture(1/**/e2)",
        "Future([a] {b:c} nested(1))",
        "(unknown [a] {b:c})",
    ] {
        let condition = condition(query);
        let value = opaque(&condition);
        assert_eq!(value.authored(), Some(query));
        assert_eq!(value.serialize().unwrap().as_css(), query);
    }
}

#[test]
fn programmatic_opaque_wrapper_preserves_parsed_children_and_required_token_boundaries() {
    let parsed = parse_component_values("/*😀*/1").unwrap();
    let child = parsed.items().last().unwrap().clone();
    let original = child.origin().clone();
    let wrapper = CssGeneralEnclosed::try_function(
        "Future",
        CssComponentValues::try_new(vec![child, CssComponentValue::try_ident("e2").unwrap()])
            .unwrap(),
    )
    .unwrap();
    let condition = Condition::try_from_enclosed(wrapper).unwrap();
    let value = opaque(&condition);
    assert_eq!(value.position(), None);
    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(value.authored(), None);
    let output = value.serialize().unwrap();
    assert_eq!(output.as_css(), "Future(1/**/e2)");
    let Some(CssSerializedOrigin::Token(CssValueOrigin::Parsed(actual))) = output.origin_at(7)
    else {
        panic!("original numeric token")
    };
    let CssValueOrigin::Parsed(expected) = original else {
        panic!("parsed")
    };
    assert!(actual.source().same_snapshot(expected.source()));
    assert_eq!(actual.span(), expected.span());
    let cloned = condition.clone();
    assert_eq!(cloned, condition);
    drop(condition);
    assert_eq!(
        opaque(&cloned).serialize().unwrap().as_css(),
        "Future(1/**/e2)"
    );
}

#[test]
fn a_recovered_lexical_operand_retains_its_eof_closure_origin() {
    // A missing container rule block is independently invalid. Exercise the
    // recovered lexical operand through its public checked enclosure owner.
    let components = parse_component_values("Future(").unwrap();
    let value = CssGeneralEnclosed::try_from_component(components.items()[0].clone()).unwrap();
    let condition = Condition::try_from_enclosed(value).unwrap();
    let value = opaque(&condition);
    assert_eq!(value.authored(), Some("Future("));
    let output = value.serialize().unwrap();
    assert_eq!(output.as_css(), "Future()");
    let Some(CssSerializedOrigin::Token(CssValueOrigin::ImplicitClosure { opening, at })) =
        output.origin_at(7)
    else {
        panic!("EOF closure")
    };
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start(), at.span().end());
    assert_eq!(at.span().start().byte_offset().value(), 7);
}

#[test]
fn lexical_errors_and_malformed_outer_boolean_grammar_are_not_hidden_by_fallback() {
    for query in [
        "Future(url(a b))",
        "Future(])",
        "Future() ]",
        "Future() junk",
        "Future() and",
        "not not Future()",
        "Future() and Other() or Third()",
    ] {
        let report = parse_sheet(&format!("@container {query} {{ .x {{ color:red }} }}"));
        assert!(!report.is_clean(), "{query}");
        assert!(report.syntax().rules().is_empty(), "{query}");
    }
}

#[test]
fn component_depth_limit_still_rejects_an_oversized_opaque_query() {
    let query = format!("{}x{}", "Future(".repeat(257), ")".repeat(257));
    let report = parse_sheet(&format!("@container {query} {{}}"));
    assert!(report.syntax().rules().is_empty());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::StopAtNestingLimit)
    );
}

#[test]
fn speculative_query_branches_preserve_terminal_bad_token_categories_and_original_coordinates() {
    use surgeist_css::{CssComponentValueErrorKind, CssErrorCode, ErrorKind};
    // Syntax tokenization attributes a bad string to its opening quote and a
    // bad URL to its url( opener, even when an enclosing query probe fails.
    for (query, offending, kind) in [
        (
            "Future(\"x\n)",
            "\"x",
            CssComponentValueErrorKind::BadString,
        ),
        (
            "not ((width > 1px) and (not Future(url(a b))))",
            "url(a b)",
            CssComponentValueErrorKind::BadUrl,
        ),
    ] {
        let source = format!("/*😀*/ @container {query} {{ .x {{ color:red }} }}");
        let offset = source.find(offending).unwrap();
        let report = parse_sheet(&source);
        assert!(report.syntax().rules().is_empty(), "{query}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one terminal lexical failure: {:?}", report.diagnostics())
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidComponentValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        let ErrorKind::InvalidComponentValue(component) = diagnostic.error().kind() else {
            panic!("original typed component failure")
        };
        assert_eq!(component.kind(), kind);
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("original bad token")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(origin.span().start().byte_offset().value(), offset);
        assert_eq!(origin.span().start().line().value(), 0);
        assert_eq!(
            origin.span().start().column().value(),
            u32::try_from(source[..offset].encode_utf16().count()).unwrap()
        );
        assert_eq!(diagnostic.error().position(), origin.span().start());
    }
}

#[test]
fn checked_enclosure_classification_prefers_the_same_recognized_branches_as_parsing() {
    let feature =
        CssGeneralEnclosed::try_parenthesized(parse_component_values("width > 1px").unwrap())
            .unwrap();
    assert!(matches!(
        Condition::try_from_enclosed(feature).unwrap().kind(),
        Kind::Feature(_)
    ));
    assert!(matches!(
        condition("(width > 1px)").kind(),
        Kind::Feature(_)
    ));
    let style =
        CssGeneralEnclosed::try_function("style", parse_component_values("--theme").unwrap())
            .unwrap();
    assert!(matches!(
        Condition::try_from_enclosed(style).unwrap().kind(),
        Kind::Style(_)
    ));
    assert!(matches!(condition("style(--theme)").kind(), Kind::Style(_)));
    let grouped = CssGeneralEnclosed::try_parenthesized(
        parse_component_values("(width > 1px) and style(--theme)").unwrap(),
    )
    .unwrap();
    let checked = Condition::try_from_enclosed(grouped).unwrap();
    let parsed = condition("((width > 1px) and style(--theme))");
    for condition in [&checked, &parsed] {
        let Kind::Parenthesized(grouped) = condition.kind() else {
            panic!("explicit grouping")
        };
        let Kind::And(children) = grouped.kind() else {
            panic!("grouped conjunction")
        };
        let [feature, style] = children.conditions() else {
            panic!("two operands")
        };
        assert!(matches!(feature.kind(), Kind::Feature(_)));
        assert!(matches!(style.kind(), Kind::Style(_)));
    }
}

#[test]
fn checked_unknown_and_failed_recognized_enclosures_retain_the_supplied_lexical_value() {
    for (name, contents) in [
        ("Future", ""),
        ("style", "color:red"),
        ("Future", "1/**/e2"),
    ] {
        let enclosure =
            CssGeneralEnclosed::try_function(name, parse_component_values(contents).unwrap())
                .unwrap();
        let original = enclosure.clone();
        let checked = Condition::try_from_enclosed(enclosure).unwrap();
        assert_eq!(opaque(&checked).component(), original.component());
        assert_eq!(opaque(&checked).origin(), &CssValueOrigin::Programmatic);
        let parsed = condition(&format!("{name}({contents})"));
        assert_eq!(
            opaque(&parsed).serialize().unwrap().as_css(),
            format!("{name}({contents})")
        );
    }
    // Invalid recognized grammar still has a complete general-enclosed branch.
    let enclosure =
        CssGeneralEnclosed::try_parenthesized(parse_component_values("width: nonsense").unwrap())
            .unwrap();
    assert!(matches!(
        Condition::try_from_enclosed(enclosure).unwrap().kind(),
        Kind::GeneralEnclosed(_)
    ));
}

#[test]
fn wrong_outer_syntax_is_rejected_by_the_enclosure_owner_with_its_original_origin() {
    use surgeist_css::CssGeneralEnclosedError;
    for source in ["/*😀*/ident", "/*😀*/[x]", "/*😀*/{x}"] {
        let components = parse_component_values(source).unwrap();
        let component = components.items().last().unwrap().clone();
        let expected = component.origin().clone();
        let error = CssGeneralEnclosed::try_from_component(component).unwrap_err();
        assert!(matches!(
            error,
            CssGeneralEnclosedError::WrongOuterComponent { .. }
        ));
        let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) =
            (error.origin(), expected)
        else {
            panic!("original rejected opener")
        };
        assert!(actual.source().same_snapshot(expected.source()));
        assert_eq!(actual.span(), expected.span());
    }
}

#[test]
fn fully_programmatic_known_enclosures_expose_expected_feature_and_custom_property_payloads() {
    use surgeist_css::{
        CssContainerFeatureQuery, CssContainerLengthRef, CssContainerStyleQuery, CssMediaRangeRef,
        CssQueryComparison,
    };
    let values = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("width").unwrap(),
        CssComponentValue::try_token(">").unwrap(),
        CssComponentValue::try_dimension("1", "px").unwrap(),
    ])
    .unwrap();
    let feature =
        Condition::try_from_enclosed(CssGeneralEnclosed::try_parenthesized(values).unwrap())
            .unwrap();
    let Kind::Feature(CssContainerFeatureQuery::Width(range)) = feature.kind() else {
        panic!("known width")
    };
    let CssMediaRangeRef::FeatureFirst { comparison, value } = range.view() else {
        panic!("range")
    };
    assert_eq!(comparison, CssQueryComparison::GreaterThan);
    let CssContainerLengthRef::Numeric(value) = value.view() else {
        panic!("length")
    };
    assert_eq!(value.components().serialize().unwrap().as_css(), "1px");
    let values =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("--Theme").unwrap()])
            .unwrap();
    let style =
        Condition::try_from_enclosed(CssGeneralEnclosed::try_function("style", values).unwrap())
            .unwrap();
    let Kind::Style(CssContainerStyleQuery::CustomPropertyPresence(name)) = style.kind() else {
        panic!("known custom property presence")
    };
    assert_eq!(name.as_str(), "--Theme");
}

#[test]
fn style_variable_grammar_preserves_valid_references_and_falls_back_for_invalid_references() {
    use surgeist_css::CssContainerStyleQuery;
    // Existing authored declaration grammar requires var's first argument to be
    // one custom-property name, with an optional comma and fallback value.
    for source in ["var(--tone)", "var(--tone, red)", "var(--tone,)"] {
        let query = format!("style(--theme:{source})");
        let parsed = condition(&query);
        let enclosure = CssGeneralEnclosed::try_function(
            "style",
            parse_component_values(&format!("--theme:{source}")).unwrap(),
        )
        .unwrap();
        let checked = Condition::try_from_enclosed(enclosure).unwrap();
        for condition in [parsed, checked] {
            let Kind::Style(CssContainerStyleQuery::CustomPropertyValue { name, value }) =
                condition.kind()
            else {
                panic!("valid variable remains a style value: {source}")
            };
            assert_eq!(name.as_str(), "--theme");
            assert_eq!(value.as_css(), source);
        }
    }
    for source in ["var()", "var(tone)", "var(--a --b)"] {
        let query = format!("style(--theme:{source})");
        let parsed = condition(&query);
        let enclosure = CssGeneralEnclosed::try_function(
            "style",
            parse_component_values(&format!("--theme:{source}")).unwrap(),
        )
        .unwrap();
        let checked = Condition::try_from_enclosed(enclosure).unwrap();
        assert_eq!(opaque(&parsed).serialize().unwrap().as_css(), query);
        assert_eq!(opaque(&checked).serialize().unwrap().as_css(), query);
    }
}

#[test]
fn constructed_style_value_serialization_keeps_adjacent_number_and_identifier_separate() {
    use surgeist_css::CssContainerStyleQuery;
    for first in [
        CssComponentValue::try_number("1").unwrap(),
        parse_component_values("/*😀*/1")
            .unwrap()
            .items()
            .last()
            .unwrap()
            .clone(),
    ] {
        let values = CssComponentValues::try_new(vec![
            CssComponentValue::try_ident("--theme").unwrap(),
            CssComponentValue::try_token(":").unwrap(),
            first,
            CssComponentValue::try_ident("e2").unwrap(),
        ])
        .unwrap();
        let condition = Condition::try_from_enclosed(
            CssGeneralEnclosed::try_function("style", values).unwrap(),
        )
        .unwrap();
        let Kind::Style(CssContainerStyleQuery::CustomPropertyValue { name, value }) =
            condition.kind()
        else {
            panic!("authored custom property value")
        };
        assert_eq!(name.as_str(), "--theme");
        assert_eq!(value.as_css(), "1/**/e2");
    }
}

fn parsed_and_checked_feature(body: &str) -> [Condition; 2] {
    let parsed = condition(&format!("({body})"));
    let enclosed =
        CssGeneralEnclosed::try_parenthesized(parse_component_values(body).unwrap()).unwrap();
    [parsed, Condition::try_from_enclosed(enclosed).unwrap()]
}

#[test]
fn exact_query_lengths_preserve_underflow_spelling_and_signed_zero() {
    use surgeist_css::{CssContainerFeatureQuery, CssContainerLengthRef, CssMediaRangeRef};
    for literal in ["-1e-999px", "1e-999px", "-0px", "+0px", "-0"] {
        let programmatic = CssGeneralEnclosed::try_parenthesized(
            CssComponentValues::try_new(vec![
                CssComponentValue::try_ident("width").unwrap(),
                CssComponentValue::try_token(":").unwrap(),
                CssComponentValue::try_token(literal).unwrap(),
            ])
            .unwrap(),
        )
        .unwrap();
        let [parsed, checked] = parsed_and_checked_feature(&format!("width:{literal}"));
        for condition in [
            parsed,
            checked,
            Condition::try_from_enclosed(programmatic).unwrap(),
        ] {
            let Kind::Feature(CssContainerFeatureQuery::Width(range)) = condition.kind() else {
                panic!("exact length {literal}")
            };
            let CssMediaRangeRef::Plain { value } = range.view() else {
                panic!("plain")
            };
            let CssContainerLengthRef::Numeric(value) = value.view() else {
                panic!("numeric")
            };
            assert_eq!(value.components().serialize().unwrap().as_css(), literal);
        }
    }
    // Neither tiny nonzero unitless number becomes zero through f32 underflow.
    for body in ["width:-1e-999", "width:1e-999"] {
        for condition in parsed_and_checked_feature(body) {
            assert_eq!(
                opaque(&condition).serialize().unwrap().as_css(),
                format!("({body})")
            );
        }
    }
}

#[test]
fn exact_query_numbers_do_not_overflow_through_tokenizer_floats() {
    for body in [
        "width:1e999px",
        "width:-1e999px",
        "width:0e999px",
        "width:0e999",
        "aspect-ratio:1e999 / 1",
        "aspect-ratio:1 / 1e999",
    ] {
        for (route, condition) in parsed_and_checked_feature(body).into_iter().enumerate() {
            assert!(matches!(condition.kind(), Kind::Feature(_)), "{body}");
            // The stylesheet helper contributes one trailing prelude space;
            // direct enclosure construction has no surrounding trivia.
            let trailing = if route == 0 { " " } else { "" };
            assert_eq!(
                condition.serialize().unwrap().as_css(),
                format!("({body}){trailing}")
            );
        }
    }
}

#[test]
fn exact_ratios_admit_degenerate_zero_and_reject_negative_nonzero_literals() {
    use surgeist_css::{CssContainerFeatureQuery, CssContainerRatioRef, CssMediaRangeRef};
    for (body, numerator, denominator) in [
        ("aspect-ratio:2 / 3", "2", "3"),
        ("aspect-ratio:-0 / 1", "-0", "1"),
        ("aspect-ratio:1 / 0", "1", "0"),
        ("aspect-ratio:1 / -0", "1", "-0"),
        ("aspect-ratio:1 / 1e-999", "1", "1e-999"),
    ] {
        for condition in parsed_and_checked_feature(body) {
            let Kind::Feature(CssContainerFeatureQuery::AspectRatio(range)) = condition.kind()
            else {
                panic!("exact ratio {body}")
            };
            let CssMediaRangeRef::Plain { value } = range.view() else {
                panic!("plain")
            };
            let CssContainerRatioRef::Numeric(value) = value.view() else {
                panic!("numeric")
            };
            assert_eq!(
                value
                    .numerator()
                    .components()
                    .serialize()
                    .unwrap()
                    .as_css()
                    .trim(),
                numerator
            );
            assert_eq!(
                value
                    .denominator()
                    .components()
                    .serialize()
                    .unwrap()
                    .as_css()
                    .trim(),
                denominator
            );
        }
    }
    for body in [
        "aspect-ratio:-1e-999 / 1",
        "aspect-ratio:1 / -1e-999",
        "aspect-ratio:-1 / 2",
    ] {
        for condition in parsed_and_checked_feature(body) {
            assert_eq!(
                opaque(&condition).serialize().unwrap().as_css(),
                format!("({body})")
            );
        }
    }
}
