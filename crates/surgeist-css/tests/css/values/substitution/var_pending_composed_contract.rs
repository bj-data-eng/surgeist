#![forbid(unsafe_code)]
//! Variables 1 §3 retains everything after the first var fallback comma.
//! It permits var only in property-value contexts and defers whole valid values.
//! https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#using-variables
//! The public pending transition preserves original occurrence, priority and origins.

use surgeist_css::*;

#[test]
fn later_var_commas_are_retained_fallback_tokens_in_known_and_custom_values() {
    for (text, arguments, comma_count) in [
        ("var(--Theme, red,/**/ BLUE,)", "--Theme, red,/**/ BLUE,", 3),
        (
            "var(--Theme, nested(red, blue), var(--other,))",
            "--Theme, nested(red, blue), var(--other,)",
            2,
        ),
    ] {
        let source = format!("display:{text}!important");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let parsed = &report.syntax()[0];
        let components = parse_component_values(text).unwrap();
        let original = components.clone();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Display),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        for declaration in [parsed, &checked] {
            assert_eq!(declaration.importance(), CssImportance::Important);
            assert_eq!(
                declaration
                    .known()
                    .unwrap()
                    .substitution_dependent()
                    .unwrap()
                    .as_css(),
                text,
            );
            let [function] = declaration.value_components().items() else {
                panic!("one complete var function")
            };
            let CssComponentValueRef::Function(function) = function.view() else {
                panic!("retained var function")
            };
            assert_eq!(function.name(), "var");
            assert_eq!(function.values().serialize().unwrap().as_css(), arguments);
            assert_eq!(
                function
                    .values()
                    .items()
                    .iter()
                    .filter(|value| matches!(
                        value.view(),
                        CssComponentValueRef::Token(CssValueTokenRef::Comma)
                    ))
                    .count(),
                comma_count,
            );
            assert!(matches!(
                expand_declaration(declaration),
                Ok(CssExpansion::Pending(_))
            ));
        }
        let name = CssCustomPropertyName::try_new("--kept").unwrap();
        let custom = parse_property_value(
            CssPropertyNameRef::Custom(&name),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(
            custom.custom().unwrap().value().value().unwrap().as_css(),
            text
        );
        assert_eq!(custom.value_components(), &original);
        assert!(matches!(
            expand_declaration(&custom),
            Ok(CssExpansion::Contributions(_))
        ));
        assert_eq!(components, original);
    }
}

#[test]
fn valid_var_calls_cannot_form_property_names_or_selectors() {
    for source in ["var(--side):20px", "var(--side, margin-top):20px"] {
        let report = parse_declaration(source);
        assert!(!report.is_clean(), "{source}");
        assert!(
            report.syntax().is_none(),
            "variable property name: {source}"
        );
    }
    let context = CssNamespaceContext::default();
    for selector in ["var(--selector)", ".item var(--selector, .chosen)"] {
        let report = parse_selector(selector, &context);
        assert!(!report.is_clean(), "{selector}");
        assert!(report.syntax().is_none(), "variable selector: {selector}");
        let rule = parse_rule(&format!("{selector}{{color:red}}"), &context);
        assert!(!rule.is_clean(), "{selector}");
        assert!(
            rule.syntax().is_none(),
            "variable rule selector: {selector}"
        );
    }
    let control = parse_declaration("display:var(--mode)");
    assert!(control.is_clean());
    assert!(
        control
            .syntax()
            .as_ref()
            .unwrap()
            .known()
            .unwrap()
            .substitution_dependent()
            .is_some()
    );
    assert!(parse_selector(".item", &context).is_clean());
}

#[test]
fn strict_pending_reentry_retains_original_priority_source_after_failed_replacements() {
    const SOURCE: &str = "/*😀*/display:bogus VAR(--layout, red, blue)!/**/ImPoRtAnT";
    const VALUE: &str = "bogus VAR(--layout, red, blue)";
    let report = parse_style_attribute(SOURCE);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one complete pending declaration")
    };
    assert_eq!(source.importance(), CssImportance::Important);
    assert_eq!(
        source
            .known()
            .unwrap()
            .substitution_dependent()
            .unwrap()
            .as_css(),
        VALUE
    );
    assert_eq!(
        source.value_components().serialize().unwrap().as_css(),
        VALUE
    );
    let origin = source.parsed_value().unwrap();
    assert_eq!(origin.source().as_str(), SOURCE);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        "/*😀*/display:".len()
    );
    assert_eq!(
        origin.span().end().byte_offset().value(),
        SOURCE.find('!').unwrap()
    );
    let CssExpansion::Pending(pending) = expand_declaration(source).unwrap() else {
        panic!("whole value awaits replacement")
    };
    for invalid in ["block!important", "block;inline"] {
        let error = pending
            .reenter(parse_component_values(invalid).unwrap())
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        assert!(pending.source().same_occurrence(source));
        assert_eq!(
            pending.source().parsed_value().unwrap().source().as_str(),
            SOURCE
        );
        assert_eq!(pending.source().importance(), CssImportance::Important);
    }
    assert_eq!(
        pending
            .reenter(parse_component_values("var(x)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution,
    );
    let replacement = parse_component_values("block").unwrap();
    let original = replacement.clone();
    for _ in 0..2 {
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("one completed display contribution")
        };
        let [value] = values.items() else {
            panic!("one terminal display")
        };
        assert_eq!(value.property(), CssKnownProperty::Display);
        let CssLonghandValueRef::Display(display) = value.ordinary_value().unwrap().view() else {
            panic!("actual replacement display")
        };
        assert_eq!(display.serialize_specified().unwrap(), "block");
        assert!(value.source().same_occurrence(source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert_eq!(
            value.source().parsed_value().unwrap().source().as_str(),
            SOURCE
        );
        assert_eq!(
            value
                .source()
                .value_components()
                .serialize()
                .unwrap()
                .as_css(),
            VALUE
        );
        assert_eq!(value.replacement_components(), Some(&replacement));
        let CssValueOrigin::Parsed(retained) =
            value.replacement_components().unwrap().items()[0].origin()
        else {
            panic!("original caller replacement token")
        };
        assert_eq!(retained.source().as_str(), "block");
        assert_eq!(retained.span().start().byte_offset().value(), 0);
        assert_eq!(retained.span().end().byte_offset().value(), 5);
    }
    assert_eq!(replacement, original);
}
