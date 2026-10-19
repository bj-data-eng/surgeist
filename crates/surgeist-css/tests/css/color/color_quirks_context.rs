#![forbid(unsafe_code)]
//! Color4 CRD20260908 Appendix B, selected raw-representation disposition.
//! Functional new-API evidence with implementation; no missing-symbol RED.
use surgeist_css::*;
const QUIRKS: CssParserContext = CssParserContext::new(CssParserMode::Quirks);
const NAMES: [&str; 7] = [
    "color",
    "background-color",
    "border-color",
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
];
// Independent integer/padding/hexadecimal byte goldens, not captured production output.
const GOLDENS: &[(&str, &str, &str)] = &[
    ("ABC", "ABC", "rgb(170, 187, 204)"),
    ("a1b2c3", "a1b2c3", "rgb(161, 178, 195)"),
    ("123", "000123", "rgb(0, 1, 35)"),
    ("+000123", "000123", "rgb(0, 1, 35)"),
    ("000000123", "000123", "rgb(0, 1, 35)"),
    ("0", "000000", "rgb(0, 0, 0)"),
    ("-0", "000000", "rgb(0, 0, 0)"),
    ("999999", "999999", "rgb(153, 153, 153)"),
    ("12abc", "012abc", "rgb(1, 42, 188)"),
    ("0001FF", "0001FF", "rgb(0, 1, 255)"),
    ("123abc", "123abc", "rgb(18, 58, 188)"),
];
fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name).unwrap()
}
fn checked(context: CssParserContext, name: &str, text: &str) -> CssDeclaration {
    context
        .parse_property_value_for_grammar(
            property(name).grammar(),
            parse_component_values(text).unwrap(),
            CssImportance::Important,
        )
        .unwrap()
}
fn color(source: &CssDeclaration) -> &CssColor {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Color(v) => v.value(),
        CssKnownPropertyValueRef::BackgroundColor(v) => v.value(),
        CssKnownPropertyValueRef::BorderTopColor(v) => v.value(),
        CssKnownPropertyValueRef::BorderRightColor(v) => v.value(),
        CssKnownPropertyValueRef::BorderBottomColor(v) => v.value(),
        CssKnownPropertyValueRef::BorderLeftColor(v) => v.value(),
        CssKnownPropertyValueRef::BorderColor(v) => v.value().assigned_values()[0],
        _ => panic!("direct Color owner"),
    }
}
fn test_declaration(condition: &CssSupportsCondition) -> &CssSupportsDeclaration {
    let CssSupportsConditionKind::Declaration(v) = condition.kind() else {
        panic!("declaration test")
    };
    v
}
fn strict_error(error: &CssPropertyValueParseError, original: &str) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    let CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { opening, at })) =
        error.origin()
    else {
        panic!("original EOF")
    };
    assert_eq!(opening.source().as_str(), original);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), original.len());
    assert_eq!(at.span().start(), at.span().end());
}
#[test]
fn defaults_and_existing_functions_keep_standards_grammar() {
    assert_eq!(CssParserContext::default().mode(), CssParserMode::Standards);
    for name in NAMES {
        for text in ["ABC", "123", "12abc"] {
            let values = parse_component_values(text).unwrap();
            let p = property(name);
            assert!(
                parse_property_value(
                    CssPropertyNameRef::Known(p),
                    values.clone(),
                    CssImportance::Normal
                )
                .is_err()
            );
            assert!(
                CssParserContext::default()
                    .parse_property_value_for_grammar(p.grammar(), values, CssImportance::Normal)
                    .is_err()
            );
            assert!(
                CssParserContext::default()
                    .parse_property_value_text(
                        text,
                        CssPropertyNameRef::Known(p),
                        CssImportance::Normal
                    )
                    .syntax()
                    .is_none()
            );
            assert!(
                CssParserContext::default()
                    .parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Normal)
                    .syntax()
                    .is_none()
            );
            let css = format!("{name}:{text};color:red");
            assert_eq!(
                CssParserContext::default().parse_style_attribute(&css),
                parse_style_attribute(&css)
            );
        }
    }
    let standard = checked(CssParserContext::default(), "color", "var(--paint)");
    let quirky = checked(QUIRKS, "color", "var(--paint)");
    assert_ne!(standard, quirky);
    assert_eq!(quirky, quirky.clone());
    assert!(quirky.same_occurrence(&quirky.clone()));
    assert!(!quirky.same_occurrence(&standard));
}
#[test]
fn seven_properties_convert_quirky_tokens_across_all_declaration_fronts() {
    for name in NAMES {
        for &(text, hex, canonical) in GOLDENS {
            let css = format!("{name}:{text}!important");
            let report = QUIRKS.parse_style_attribute(&css);
            assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
            assert_eq!(
                &report.clone().into_validation_result().unwrap(),
                report.syntax()
            );
            let raw = parse_component_values(text).unwrap();
            let generated =
                CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()])
                    .unwrap();
            let p = property(name);
            let single = QUIRKS.parse_declaration(&css);
            assert!(single.is_clean());
            let name_text = QUIRKS.parse_property_value_text(
                text,
                CssPropertyNameRef::Known(p),
                CssImportance::Important,
            );
            let grammar_text = QUIRKS.parse_property_value_text_for_grammar(
                text,
                p.grammar(),
                CssImportance::Important,
            );
            assert!(name_text.is_clean() && grammar_text.is_clean());
            for source in [
                report.syntax()[0].clone(),
                single.syntax().as_ref().unwrap().clone(),
                QUIRKS
                    .parse_property_value(
                        CssPropertyNameRef::Known(p),
                        raw.clone(),
                        CssImportance::Important,
                    )
                    .unwrap(),
                QUIRKS
                    .parse_property_value_for_grammar(
                        p.grammar(),
                        generated.clone(),
                        CssImportance::Important,
                    )
                    .unwrap(),
                name_text.syntax().as_ref().unwrap().clone(),
                grammar_text.syntax().as_ref().unwrap().clone(),
            ] {
                assert_eq!(source.parser_context(), QUIRKS);
                assert_eq!(source.known().unwrap().grammar(), p.grammar());
                assert_eq!(source.importance(), CssImportance::Important);
                assert_eq!(
                    color(&source),
                    &CssColor::from_hex(CssHexColor::try_new(hex).unwrap())
                );
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {canonical} !important;")
                );
                assert_eq!(
                    source.value_components().serialize().unwrap().as_css(),
                    text
                );
                let emitted = source.to_specified_css().unwrap();
                let reparse = parse_style_attribute(&emitted);
                assert!(reparse.is_clean());
                assert_eq!(
                    color(&reparse.syntax()[0]).to_specified_css().unwrap(),
                    canonical
                );
            }
            assert_eq!(raw.serialize().unwrap().as_css(), text);
            assert_eq!(generated.items()[0].origin(), &CssValueOrigin::Programmatic);
        }
    }
}
#[test]
fn raw_quirky_escapes_fail_while_ordinary_escaped_named_colors_win() {
    for name in NAMES {
        for text in [r"\61 bc", r"12\61 bc", r"\61\62\63", r"12a\62 c"] {
            for values in [
                parse_component_values(text).unwrap(),
                CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()])
                    .unwrap(),
            ] {
                let before = values.clone();
                assert!(
                    QUIRKS
                        .parse_property_value_for_grammar(
                            property(name).grammar(),
                            values.clone(),
                            CssImportance::Normal
                        )
                        .is_err()
                );
                assert_eq!(values, before);
            }
        }
        assert_eq!(
            color(&checked(QUIRKS, name, r"\72 ed")),
            &CssColor::from_named(CssNamedColor::try_new("red").unwrap())
        );
        let values =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident("abc").unwrap()])
                .unwrap();
        let source = QUIRKS
            .parse_property_value_for_grammar(
                property(name).grammar(),
                values.clone(),
                CssImportance::Normal,
            )
            .unwrap();
        assert_eq!(source.value_components(), &values);
        assert_eq!(
            color(&source).to_specified_css().unwrap(),
            "rgb(170, 187, 204)"
        );
    }
}
#[test]
fn lexical_flag_length_range_and_unit_failures_keep_original_error_anchors() {
    for name in NAMES {
        for text in [
            "-1",
            "1000000",
            "1.0",
            "1e0",
            "1.0abc",
            "1e0abc",
            "abcd",
            "abcdefgh",
            "12abz",
            "1234abc",
            "123%",
            "\"abc\"",
            "calc(123)",
        ] {
            let values = parse_component_values(text).unwrap();
            let before = values.clone();
            let error = QUIRKS
                .parse_property_value_for_grammar(
                    property(name).grammar(),
                    values.clone(),
                    CssImportance::Normal,
                )
                .unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
            let anchor = match error.origin() {
                CssSerializedOrigin::Token(v) | CssSerializedOrigin::End(Some(v)) => v,
                _ => panic!("original token"),
            };
            assert_eq!(anchor, values.items()[0].origin());
            assert_eq!(values, before);
        }
    }
}
#[test]
fn rust_authored_number_and_dimension_flags_stay_faithful() {
    for (token, expected) in [
        (
            CssComponentValue::try_number("+000123").unwrap(),
            Some("rgb(0, 1, 35)"),
        ),
        (
            CssComponentValue::try_number("-0").unwrap(),
            Some("rgb(0, 0, 0)"),
        ),
        (CssComponentValue::try_number("1.0").unwrap(), None),
        (CssComponentValue::try_number("1e0").unwrap(), None),
        (
            CssComponentValue::try_dimension("12", "abc").unwrap(),
            Some("rgb(1, 42, 188)"),
        ),
        (
            CssComponentValue::try_dimension("1.0", "abc").unwrap(),
            None,
        ),
        (CssComponentValue::try_token(r"12\61 bc").unwrap(), None),
    ] {
        let values = CssComponentValues::try_new(vec![token]).unwrap();
        let before = values.clone();
        let result = QUIRKS.parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            values.clone(),
            CssImportance::Important,
        );
        match expected {
            Some(text) => {
                let source = result.unwrap();
                assert_eq!(color(&source).to_specified_css().unwrap(), text);
                assert_eq!(source.value_components(), &values);
                assert!(source.position().is_none());
            }
            None => assert!(result.is_err()),
        }
        assert_eq!(values, before);
    }
}
#[test]
fn referencing_shorthands_logical_owners_and_color_functions_do_not_acquire_quirks() {
    for (name, text) in [
        ("background", "ABC"),
        ("border", "solid ABC"),
        ("border-top", "solid ABC"),
        ("border-right", "solid ABC"),
        ("border-bottom", "solid ABC"),
        ("border-left", "solid ABC"),
        ("border-block-color", "ABC"),
        ("border-inline-color", "ABC"),
        ("border-block-start-color", "ABC"),
        ("border-inline-end-color", "ABC"),
        ("outline-color", "ABC"),
        ("text-decoration-color", "ABC"),
        ("flood-color", "ABC"),
        ("lighting-color", "ABC"),
        ("border-color", "logical ABC"),
    ] {
        assert!(
            QUIRKS
                .parse_property_value_for_grammar(
                    property(name).grammar(),
                    parse_component_values(text).unwrap(),
                    CssImportance::Normal
                )
                .is_err(),
            "{name}:{text}"
        );
    }
    for name in NAMES {
        for text in [
            "color-mix(in oklab, ABC, red)",
            "light-dark(ABC, red)",
            "contrast-color(ABC)",
            "alpha(from ABC / .5)",
            "rgb(from ABC r g b)",
        ] {
            assert!(
                QUIRKS
                    .parse_property_value_for_grammar(
                        property(name).grammar(),
                        parse_component_values(text).unwrap(),
                        CssImportance::Normal
                    )
                    .is_err(),
                "{name}:{text}"
            );
        }
    }
    assert!(
        QUIRKS
            .parse_style_attribute("border-color:logical red blue")
            .is_clean()
    );
}
#[test]
fn physical_border_color_assigns_one_through_four_colors_without_extra_resets() {
    for (text, expected) in [
        ("ABC", ["rgb(170, 187, 204)"; 4]),
        (
            "ABC 123",
            [
                "rgb(170, 187, 204)",
                "rgb(0, 1, 35)",
                "rgb(170, 187, 204)",
                "rgb(0, 1, 35)",
            ],
        ),
        (
            "ABC red 12abc",
            ["rgb(170, 187, 204)", "red", "rgb(1, 42, 188)", "red"],
        ),
        (
            "ABC red 12abc blue",
            ["rgb(170, 187, 204)", "red", "rgb(1, 42, 188)", "blue"],
        ),
    ] {
        let source = checked(QUIRKS, "border-color", text);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("physical members")
        };
        assert_eq!(values.items().len(), 4);
        for ((item, p), text) in values
            .items()
            .iter()
            .zip([
                CssKnownProperty::BorderTopColor,
                CssKnownProperty::BorderRightColor,
                CssKnownProperty::BorderBottomColor,
                CssKnownProperty::BorderLeftColor,
            ])
            .zip(expected)
        {
            assert_eq!(item.property(), p);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
            let color = match item.ordinary_value().unwrap().view() {
                CssLonghandValueRef::BorderTopColor(v)
                | CssLonghandValueRef::BorderRightColor(v)
                | CssLonghandValueRef::BorderBottomColor(v)
                | CssLonghandValueRef::BorderLeftColor(v) => v,
                _ => panic!("physical color"),
            };
            assert_eq!(color.to_specified_css().unwrap(), text);
        }
    }
    assert!(
        QUIRKS
            .parse_style_attribute("border-color:ABC red 123 blue 12abc")
            .syntax()
            .is_empty()
    );
}
#[test]
fn sheet_rule_and_block_fronts_keep_mode_through_nested_declaration_runs() {
    let report = QUIRKS
        .parse_sheet("@media all{.a{color:ABC;& .b{background-color:123}border-color:12abc}}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 3);
    for (order, v) in declarations.iter().enumerate() {
        assert_eq!(v.order(), order);
        assert_eq!(v.source().parser_context(), QUIRKS);
    }
    let namespaces = CssNamespaceContext::default();
    let report = QUIRKS.parse_rule(".a{color:123}", &namespaces);
    assert!(report.is_clean());
    let CssRule::Style(rule) = report.syntax().as_ref().unwrap() else {
        panic!("style rule")
    };
    assert_eq!(rule.declarations()[0].parser_context(), QUIRKS);
    let report = QUIRKS.parse_style_block("{color:ABC}", &namespaces);
    assert!(report.is_clean());
    assert_eq!(
        report.syntax().as_ref().unwrap().declarations()[0].parser_context(),
        QUIRKS
    );
}
#[test]
fn mode_survives_bounded_deep_sheet_recovery() {
    let mut css = "@media all{".repeat(140);
    css.push_str(".a{color:123}");
    css.push_str(&"}".repeat(140));
    let report = QUIRKS.parse_sheet(&css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0].source().parser_context(), QUIRKS);
    assert_eq!(
        color(declarations[0].source()).to_specified_css().unwrap(),
        "rgb(0, 1, 35)"
    );
}
#[test]
fn parsed_and_mixed_rust_authored_origins_are_preserved() {
    let css = "/*😀*/ COLOR:ABC!important";
    let report = QUIRKS.parse_style_attribute(css);
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    assert_eq!(
        source.position().unwrap().byte_offset().value(),
        "/*😀*/ ".len()
    );
    assert_eq!(
        source.position().unwrap().column().value() as usize,
        "/*😀*/ ".encode_utf16().count()
    );
    assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
    let parsed = parse_component_values("ABC").unwrap();
    let mut items = parsed.items().to_vec();
    items.push(CssComponentValue::try_token(" ").unwrap());
    items.push(CssComponentValue::try_number("123").unwrap());
    let values = CssComponentValues::try_new(items).unwrap();
    let before = values.clone();
    let source = QUIRKS
        .parse_property_value_for_grammar(
            CssKnownProperty::BorderColor.grammar(),
            values.clone(),
            CssImportance::Important,
        )
        .unwrap();
    assert_eq!(source.value_components(), &values);
    assert!(
        source.position().is_none()
            && source.parsed_name().is_none()
            && source.parsed_value().is_none()
    );
    assert_eq!(
        source.value_components().items()[0].origin(),
        parsed.items()[0].origin()
    );
    assert_eq!(
        source.value_components().items()[2].origin(),
        &CssValueOrigin::Programmatic
    );
    assert_eq!(values, before);
}
#[test]
fn pending_handles_keep_mode_importance_and_origins_across_failed_successful_retry() {
    for name in NAMES {
        for pending in ["var(--paint)", "env(paint)", "attr(data-paint *)"] {
            let source = checked(QUIRKS, name, pending);
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            for bad in ["1.0", "abcd", "inherit extra", "123!important"] {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(bad).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ));
            }
            assert_eq!(
                handle
                    .reenter(parse_component_values("var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            for replacement in [
                parse_component_values("ABC").unwrap(),
                CssComponentValues::try_new(vec![CssComponentValue::try_number("123").unwrap()])
                    .unwrap(),
                parse_component_values("unset").unwrap(),
            ] {
                for _ in 0..2 {
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("completed original grammar")
                    };
                    assert_eq!(
                        values.items().len(),
                        if name == "border-color" { 4 } else { 1 }
                    );
                    for item in values.items() {
                        assert!(item.source().same_occurrence(&source));
                        assert_eq!(item.source().importance(), CssImportance::Important);
                        assert_eq!(item.source().parser_context(), QUIRKS);
                        assert_eq!(item.replacement_components(), Some(&replacement));
                    }
                }
            }
            assert_eq!(source, before);
            let standard = checked(CssParserContext::default(), name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&standard).unwrap() else {
                panic!("standard pending")
            };
            assert!(
                handle
                    .reenter(parse_component_values("ABC").unwrap())
                    .is_err()
            );
        }
    }
}
#[test]
fn contextual_checked_and_reentry_failures_report_original_implicit_eof() {
    for original in ["ABC/*", "initial/*", "var(--paint"] {
        strict_error(
            &QUIRKS
                .parse_property_value_for_grammar(
                    CssKnownProperty::Color.grammar(),
                    parse_component_values(original).unwrap(),
                    CssImportance::Important,
                )
                .unwrap_err(),
            original,
        );
    }
    let source = checked(QUIRKS, "color", "var(--paint)");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    let error = handle
        .reenter(parse_component_values("ABC/*").unwrap())
        .unwrap_err();
    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
        panic!("strict original")
    };
    strict_error(error, "ABC/*");
    assert!(
        handle
            .reenter(parse_component_values("ABC/**/").unwrap())
            .is_ok()
    );
    let mut items = vec![CssComponentValue::try_ident("ABC").unwrap()];
    items.extend_from_slice(parse_component_values("/*").unwrap().items());
    strict_error(
        &QUIRKS
            .parse_property_value_for_grammar(
                CssKnownProperty::Color.grammar(),
                CssComponentValues::try_new(items).unwrap(),
                CssImportance::Normal,
            )
            .unwrap_err(),
        "/*",
    );
}
#[test]
fn supports_rules_retain_document_mode_while_both_method_overloads_force_standards() {
    let namespaces = CssNamespaceContext::default();
    for name in NAMES {
        let text = format!("({name}:123)");
        let rule = QUIRKS.parse_supports_condition(&text, &namespaces).unwrap();
        assert!(test_declaration(&rule).known().is_some());
        assert_eq!(rule.serialize().unwrap().as_css(), text);
        let method = parse_css_supports_condition(&text).unwrap();
        assert!(test_declaration(&method).known().is_none());
        assert_eq!(method.serialize().unwrap().as_css(), text);
        let method = parse_css_supports_declaration(name, "123").unwrap();
        assert!(method.known().is_none());
        let CssValueOrigin::Parsed(origin) = method.value_components()[0].origin() else {
            panic!("independent value argument source");
        };
        assert_eq!(origin.source().as_str(), "123");
        let css = format!("@supports {text} {{.a{{{name}:123}}}}");
        let report = QUIRKS.parse_sheet(&css);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let CssRule::Supports(rule) = &report.syntax().rules()[0] else {
            panic!("group")
        };
        assert!(test_declaration(rule.condition()).known().is_some());
        let report = parse_sheet(&format!("@supports {text} {{}}"));
        assert!(report.is_clean());
        let CssRule::Supports(rule) = &report.syntax().rules()[0] else {
            panic!("standard group")
        };
        assert!(test_declaration(rule.condition()).known().is_none());
    }
    for name in ["color:red", "color red", "", "#color"] {
        assert!(parse_css_supports_declaration(name, "red").is_err());
    }
    assert!(
        test_declaration(&parse_css_supports_condition("(color:red)").unwrap())
            .known()
            .is_some()
    );
    assert!(
        parse_css_supports_declaration("color", "red")
            .unwrap()
            .known()
            .is_some()
    );
    assert!(matches!(
        QUIRKS
            .parse_supports_condition("future(ABC)", &namespaces)
            .unwrap()
            .kind(),
        CssSupportsConditionKind::GeneralEnclosed(_)
    ));
}
#[test]
fn supports_resources_keep_original_component_errors_and_allow_retry() {
    let text = "(color:123)";
    let namespaces = CssNamespaceContext::default();
    let exact = CssComponentValueLimits::try_new(1, 4, text.len()).unwrap();
    assert!(
        test_declaration(
            &QUIRKS
                .parse_supports_condition_with_limits(text, &namespaces, exact)
                .unwrap()
        )
        .known()
        .is_some()
    );
    for (limits, kind) in [
        (
            CssComponentValueLimits::try_new(1, 3, text.len()).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 4, text.len() - 1).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
        (
            CssComponentValueLimits::try_new(0, 4, text.len()).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
    ] {
        let CssSupportsConstructionError::Component(error) = QUIRKS
            .parse_supports_condition_with_limits(text, &namespaces, limits)
            .unwrap_err()
        else {
            panic!("component resource")
        };
        assert_eq!(error.kind(), kind);
        if kind == CssComponentValueErrorKind::ByteLimit {
            // The byte preflight rejects before allocating a source snapshot.
            assert_eq!(
                error.origin(),
                &CssValueOrigin::UnretainedInput {
                    byte_length: text.len()
                }
            );
        } else {
            let CssValueOrigin::Parsed(origin) = error.origin() else {
                panic!("actual retained input origin");
            };
            assert_eq!(origin.source().as_str(), text);
        }
    }
    assert!(
        QUIRKS
            .parse_supports_condition_with_limits(text, &namespaces, exact)
            .is_ok()
    );
}
#[test]
fn ordinary_color_providers_and_css_wide_branches_are_unchanged() {
    for text in [
        "red",
        "#abcd",
        "#12345678",
        "rgb(1 2 3 / .5)",
        "hsl(0 100% 50%)",
        "lab(50% 20 -30)",
        "oklch(.5 .1 30)",
        "color(display-p3 1 0 0)",
        "color(--P 0% 70% 20% 0%)",
        "rgb(from red r g b / alpha)",
        "color-mix(in oklab, red, blue)",
        "light-dark(red, blue)",
        "contrast-color(red)",
        "alpha(from red / .5)",
        "device-cmyk(0% 81% 81% 30%)",
    ] {
        let standard = checked(CssParserContext::default(), "color", text);
        let quirky = checked(QUIRKS, "color", text);
        assert_eq!(color(&quirky), color(&standard));
        assert_eq!(
            quirky.to_specified_css().unwrap(),
            standard.to_specified_css().unwrap()
        );
    }
    for text in ["initial", "inherit", "unset", "revert", "revert-layer"] {
        for name in NAMES {
            let source = checked(QUIRKS, name, text);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {text} !important;")
            );
            assert_eq!(source.parser_context(), QUIRKS);
        }
    }
}
#[test]
fn conversion_shares_cumulative_normalization_and_sheet_byte_budgets() {
    let report = QUIRKS.parse_sheet(".a{color:ABC}.b{background-color:123}");
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 2, 2, 2).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 2);
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(1, 2, 2, 1).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 1
        }
    );
    assert_eq!(error.declaration_order(), Some(1));
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(declarations[1].source())
    );
    let expected = ".a { color: rgb(170, 187, 204); }\n.b { background-color: rgb(0, 1, 35); }";
    let exact = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len());
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    let short =
        CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len() - 1);
    for _ in 0..2 {
        let error = report
            .syntax()
            .to_specified_css_with_limits(short)
            .unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            )
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(report, before);
    }
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
}
#[test]
fn converted_hex_keeps_leaf_node_tariffs_and_atomic_writer_retry() {
    let source = checked(QUIRKS, "color", "ABC");
    let before = source.clone();
    let expected = "color: rgb(170, 187, 204) !important;";
    // Declaration + semantic property name + Hex Color leaf: 3 input/3 projection.
    // Hex's legacy RGB text adds bytes, not newly authored channel graph nodes.
    let exact = CssSpecifiedValueSerializationLimits::new(3, 3, expected.len());
    assert_eq!(
        source.to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            source
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(source, before);
    }
    assert_eq!(
        source.to_specified_css_with_limits(exact).unwrap(),
        expected
    );
}

#[test]
fn long_zero_prefixed_integer_is_exact_and_nonrepresentable_magnitudes_and_units_reject() {
    let zeros = format!("{}123", "0".repeat(20000));
    let source = checked(QUIRKS, "color", &zeros);
    assert_eq!(color(&source).to_specified_css().unwrap(), "rgb(0, 1, 35)");
    assert_eq!(
        source.value_components().serialize().unwrap().as_css(),
        zeros
    );
    for text in [
        format!("1{}", "0".repeat(2000)),
        format!("12{}", "a".repeat(2000)),
    ] {
        let values = parse_component_values(&text).unwrap();
        let before = values.clone();
        assert!(
            QUIRKS
                .parse_property_value_for_grammar(
                    CssKnownProperty::Color.grammar(),
                    values.clone(),
                    CssImportance::Normal
                )
                .is_err()
        );
        assert_eq!(values, before);
    }
}
#[test]
fn contextual_browser_recovery_retains_unclean_occurrences_with_their_original_mode() {
    for text in ["ABC/*", "initial/*", "var(--paint"] {
        let css = format!("color:{text}");
        let report = QUIRKS.parse_style_attribute(&css);
        assert!(!report.is_clean());
        let [source] = report.syntax().as_slice() else {
            panic!("retained recovered occurrence")
        };
        assert_eq!(source.parser_context(), QUIRKS);
        assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
        assert_eq!(
            report
                .clone()
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            report.diagnostics()
        );
    }
}
