#![forbid(unsafe_code)]
//! Counter symbols use the selected shared image grammar without resolving images.
//!
//! Authorities: Counter Styles 3 (2021-07-27) §§3, 3.2–3.8; Images 3
//! (2023-12-18) §§2–3; Color 5 (2026-09-08) §7 for light-dark images.
//! https://www.w3.org/TR/2021/CR-css-counter-styles-3-20210727/#counter-style-symbols
//! https://www.w3.org/TR/2023/CRD-css-images-3-20231218/#image-values
//! Whole-rule text follows the documented Surgeist descriptor-section order;
//! this is a project serialization policy, not a claimed CSSOM rule ordering.
//! No test depends on an Image counter-symbol variant or new provenance API.

#[path = "support/descriptor_phases.rs"]
mod descriptor_phases;
use descriptor_phases::*;

use surgeist_css::{
    CssCounterStyleDescriptorKind as Descriptor, CssCounterStyleRule, CssCounterStyleSpeakAs,
    CssCounterStyleSystem, CssCounterSymbol, CssErrorCode, CssNormalizedItem, CssRecoveryAction,
    CssRule, CssRuleContextKindRef, CssScopedRule,
    CssSpecifiedValueSerializationErrorKind as SerializationError,
    CssSpecifiedValueSerializationLimits as Limits, normalize_report, normalize_sheet, parse_sheet,
    validate_sheet,
};

fn clean_rule(body: &str) -> CssCounterStyleRule {
    let source = format!("@counter-style ImageCase {{ {body} }}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("expected one retained counter definition for {source}");
    };
    rule.clone()
}

fn assert_output(rule: &CssCounterStyleRule, descriptor: Descriptor, value: &str, body: &str) {
    let retained = rule.clone();
    let expected = format!("@counter-style ImageCase {{ {body} }}");
    assert_eq!(rule.descriptor_specified_css(descriptor).unwrap(), value);
    assert_eq!(rule.to_specified_css().unwrap(), expected);
    assert_eq!(
        rule.descriptor_specified_css_with_limits(
            descriptor,
            Limits::new(10_000, 10_000, value.len())
        )
        .unwrap(),
        value,
    );
    assert_eq!(
        rule.to_specified_css_with_limits(Limits::new(10_000, 10_000, expected.len()))
            .unwrap(),
        expected,
    );
    assert_eq!(
        rule.to_specified_css_with_limits(Limits::new(10_000, 10_000, expected.len() - 1))
            .unwrap_err()
            .kind(),
        SerializationError::ByteLimit,
    );
    assert_eq!(
        rule, &retained,
        "projections must not change authored values"
    );
    let reparsed = parse_sheet(&expected);
    assert!(reparsed.is_clean(), "{:?}", reparsed.diagnostics());
    let [CssRule::CounterStyle(reparsed)] = reparsed.syntax().rules() else {
        panic!("canonical text must remain a counter definition");
    };
    assert_eq!(reparsed.to_specified_css().unwrap(), expected);
}

// Every macro invocation below becomes a separate executable test. Expected
// descriptor values are fixed from grammar, not obtained from another serializer.
macro_rules! gradient_case {
    ($name:ident, symbols, $gradient:literal) => {
        #[test]
        fn $name() {
            let body = concat!("symbols: ", $gradient, ";");
            let rule = clean_rule(body);
            assert_output(&rule, Descriptor::Symbols, $gradient, body);
        }
    };
    ($name:ident, negative, $gradient:literal) => {
        #[test]
        fn $name() {
            let rule = clean_rule(concat!("symbols: a; negative: ", $gradient, " ')';"));
            assert_output(
                &rule,
                Descriptor::Negative,
                concat!($gradient, " \")\""),
                concat!("negative: ", $gradient, " \")\"; symbols: a;"),
            );
            assert!(matches!(rule.descriptors().negative().unwrap().ordinary_negative().suffix(),
                Some(CssCounterSymbol::String(value)) if value.as_str() == ")"));
        }
    };
    ($name:ident, prefix, $gradient:literal) => {
        #[test]
        fn $name() {
            let rule = clean_rule(concat!("symbols: a; prefix: ", $gradient, ";"));
            assert_output(&rule, Descriptor::Prefix, $gradient,
                concat!("prefix: ", $gradient, "; symbols: a;"));
        }
    };
    ($name:ident, suffix, $gradient:literal) => {
        #[test]
        fn $name() {
            let rule = clean_rule(concat!("symbols: a; suffix: ", $gradient, ";"));
            assert_output(&rule, Descriptor::Suffix, $gradient,
                concat!("suffix: ", $gradient, "; symbols: a;"));
        }
    };
    ($name:ident, pad, $gradient:literal) => {
        #[test]
        fn $name() {
            let rule = clean_rule(concat!("symbols: a; pad: 3 ", $gradient, ";"));
            assert_output(&rule, Descriptor::Pad, concat!("3 ", $gradient),
                concat!("pad: 3 ", $gradient, "; symbols: a;"));
            assert_eq!(rule.descriptors().pad().unwrap().ordinary_pad().minimum_length().literal().numeric().representation(), "3");
        }
    };
    ($name:ident, additive, $gradient:literal) => {
        #[test]
        fn $name() {
            let rule = clean_rule(concat!("system: additive; additive-symbols: ", $gradient, " 10, N 0;"));
            assert_output(&rule, Descriptor::AdditiveSymbols, concat!("10 ", $gradient, ", 0 N"),
                concat!("system: additive; additive-symbols: 10 ", $gradient, ", 0 N;"));
            assert_eq!(rule.descriptors().additive_symbols().unwrap().ordinary_additive_symbols().tuples()[0].weight().literal().numeric().representation(), "10");
        }
    };
}

gradient_case!(
    linear_symbols_are_images,
    symbols,
    "linear-gradient(red, blue)"
);
gradient_case!(
    linear_negative_is_an_image,
    negative,
    "linear-gradient(red, blue)"
);
gradient_case!(
    linear_prefix_is_an_image,
    prefix,
    "linear-gradient(red, blue)"
);
gradient_case!(
    linear_suffix_is_an_image,
    suffix,
    "linear-gradient(red, blue)"
);
gradient_case!(linear_pad_is_an_image, pad, "linear-gradient(red, blue)");
gradient_case!(
    linear_additive_symbol_is_an_image,
    additive,
    "linear-gradient(red, blue)"
);
gradient_case!(
    repeating_linear_symbols_are_images,
    symbols,
    "repeating-linear-gradient(red, blue)"
);
gradient_case!(
    repeating_linear_negative_is_an_image,
    negative,
    "repeating-linear-gradient(red, blue)"
);
gradient_case!(
    repeating_linear_prefix_is_an_image,
    prefix,
    "repeating-linear-gradient(red, blue)"
);
gradient_case!(
    repeating_linear_suffix_is_an_image,
    suffix,
    "repeating-linear-gradient(red, blue)"
);
gradient_case!(
    repeating_linear_pad_is_an_image,
    pad,
    "repeating-linear-gradient(red, blue)"
);
gradient_case!(
    repeating_linear_additive_symbol_is_an_image,
    additive,
    "repeating-linear-gradient(red, blue)"
);
gradient_case!(
    radial_symbols_are_images,
    symbols,
    "radial-gradient(red, blue)"
);
gradient_case!(
    radial_negative_is_an_image,
    negative,
    "radial-gradient(red, blue)"
);
gradient_case!(
    radial_prefix_is_an_image,
    prefix,
    "radial-gradient(red, blue)"
);
gradient_case!(
    radial_suffix_is_an_image,
    suffix,
    "radial-gradient(red, blue)"
);
gradient_case!(radial_pad_is_an_image, pad, "radial-gradient(red, blue)");
gradient_case!(
    radial_additive_symbol_is_an_image,
    additive,
    "radial-gradient(red, blue)"
);
gradient_case!(
    repeating_radial_symbols_are_images,
    symbols,
    "repeating-radial-gradient(red, blue)"
);
gradient_case!(
    repeating_radial_negative_is_an_image,
    negative,
    "repeating-radial-gradient(red, blue)"
);
gradient_case!(
    repeating_radial_prefix_is_an_image,
    prefix,
    "repeating-radial-gradient(red, blue)"
);
gradient_case!(
    repeating_radial_suffix_is_an_image,
    suffix,
    "repeating-radial-gradient(red, blue)"
);
gradient_case!(
    repeating_radial_pad_is_an_image,
    pad,
    "repeating-radial-gradient(red, blue)"
);
gradient_case!(
    repeating_radial_additive_symbol_is_an_image,
    additive,
    "repeating-radial-gradient(red, blue)"
);

#[test]
fn mixed_symbols_preserve_strings_escaped_identifiers_urls_and_gradients() {
    let rule = clean_rule(r#"symbols: 'text' \31 Name url(mark.svg) linear-gradient(red, blue);"#);
    assert_output(
        &rule,
        Descriptor::Symbols,
        r#""text" \31 Name url("mark.svg") linear-gradient(red, blue)"#,
        r#"symbols: "text" \31 Name url("mark.svg") linear-gradient(red, blue);"#,
    );
    let values = rule
        .descriptors()
        .symbols()
        .unwrap()
        .ordinary_symbols()
        .symbols();
    assert!(matches!(&values[0], CssCounterSymbol::String(value) if value.as_str() == "text"));
    assert!(matches!(&values[1], CssCounterSymbol::Ident(value) if value.as_str() == "1Name"));
}

#[test]
fn gradient_pad_accepts_both_component_orders_without_changing_the_integer() {
    for body in [
        "symbols: a; pad: +0003 linear-gradient(red, blue);",
        "symbols: a; pad: linear-gradient(red, blue) +0003;",
    ] {
        let rule = clean_rule(body);
        assert_output(
            &rule,
            Descriptor::Pad,
            "3 linear-gradient(red, blue)",
            "pad: 3 linear-gradient(red, blue); symbols: a;",
        );
        assert_eq!(
            rule.descriptors()
                .pad()
                .unwrap()
                .ordinary_pad()
                .minimum_length()
                .literal()
                .numeric()
                .representation(),
            "+0003"
        );
    }
}

#[test]
fn additive_gradient_tuples_accept_both_orders_without_reordering_weights() {
    for body in [
        "system: additive; additive-symbols: +0010 linear-gradient(red, blue), 0 N;",
        "system: additive; additive-symbols: linear-gradient(red, blue) +0010, N 0;",
    ] {
        let rule = clean_rule(body);
        assert_output(
            &rule,
            Descriptor::AdditiveSymbols,
            "10 linear-gradient(red, blue), 0 N",
            "system: additive; additive-symbols: 10 linear-gradient(red, blue), 0 N;",
        );
        assert_eq!(
            rule.descriptors()
                .additive_symbols()
                .unwrap()
                .ordinary_additive_symbols()
                .tuples()[0]
                .weight()
                .literal()
                .numeric()
                .representation(),
            "+0010"
        );
    }
}

#[test]
fn light_dark_image_symbols_keep_both_branches_and_identifier_none() {
    let rule = clean_rule(
        "symbols: none; prefix: light-dark(url(light.svg), linear-gradient(red, blue));",
    );
    assert_output(
        &rule,
        Descriptor::Prefix,
        "light-dark(url(\"light.svg\"), linear-gradient(red, blue))",
        "prefix: light-dark(url(\"light.svg\"), linear-gradient(red, blue)); symbols: none;",
    );
    assert!(
        matches!(rule.descriptors().symbols().unwrap().ordinary_symbols().symbols(),
        [CssCounterSymbol::Ident(value)] if value.as_str() == "none")
    );
}

#[test]
fn image_default_omission_keeps_the_counter_occurrence_unchanged() {
    let rule = clean_rule("symbols: a; prefix: Linear-Gradient(to bottom, red 0%, blue 100%);");
    assert_output(
        &rule,
        Descriptor::Prefix,
        "linear-gradient(red, blue)",
        "prefix: linear-gradient(red, blue); symbols: a;",
    );
}

#[test]
fn image_symbols_keep_currentcolor_and_percentage_dependent_math_unresolved() {
    let rule = clean_rule("symbols:linear-gradient(currentcolor calc(1px + 5%), blue);");
    assert_output(
        &rule,
        Descriptor::Symbols,
        "linear-gradient(currentcolor calc(5% + 1px), blue)",
        "symbols: linear-gradient(currentcolor calc(5% + 1px), blue);",
    );
}

#[test]
fn malformed_image_duplicate_keeps_the_previous_image_and_following_siblings() {
    let source = "/*😀*/\n.before{} @counter-style ImageCase { symbols:a; prefix:linear-gradient(red, blue); prefix:linear-gradient(red,); suffix:'.'; } .after{}";
    let report = parse_sheet(source);
    let retained = report.syntax().clone();
    let [
        CssRule::Style(_),
        CssRule::CounterStyle(rule),
        CssRule::Style(_),
    ] = report.syntax().rules()
    else {
        panic!("one invalid image descriptor must preserve the rule and siblings");
    };
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "only the malformed later image descriptor is invalid: {:?}",
            report.diagnostics()
        );
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    let start = source.find("prefix:linear-gradient(red,);").unwrap();
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + "prefix:linear-gradient(red,);".len()
    );
    assert_output(
        rule,
        Descriptor::Prefix,
        "linear-gradient(red, blue)",
        "prefix: linear-gradient(red, blue); suffix: \".\"; symbols: a;",
    );
    let normalized = normalize_report(&report).unwrap();
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    let failure = validate_sheet(source).expect_err("strict validation must reject recovery");
    assert_eq!(failure.diagnostics(), report.diagnostics());
    assert_eq!(report.syntax(), &retained);
}

#[test]
fn normalized_conditional_and_scoped_rules_keep_their_image_payloads() {
    let source = "@media print { @counter-style ImageCase { symbols:linear-gradient(red, blue); } } @scope { @counter-style ImageCase { symbols:a; prefix:radial-gradient(red, blue); } .inside{} }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let retained = report.syntax().clone();
    let [CssRule::Media(media), CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("conditional and scope rule lists");
    };
    let [CssRule::CounterStyle(conditional)] = media.rules() else {
        panic!("conditional counter definition");
    };
    let [CssScopedRule::CounterStyle(scoped), CssScopedRule::Style(_)] = scope.rules().rules()
    else {
        panic!("scoped counter definition and sibling");
    };
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let counters: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::CounterStyle(rule) => Some((context, rule)),
                _ => None,
            },
            _ => None,
        })
        .collect();
    let [
        (conditional_context, normalized_conditional),
        (scoped_context, normalized_scoped),
    ] = counters.as_slice()
    else {
        panic!("both complete counter payloads must survive normalization");
    };
    assert_eq!(*normalized_conditional, conditional);
    assert_eq!(*normalized_scoped, scoped);
    assert!(matches!(
        conditional_context.parent().unwrap().kind(),
        CssRuleContextKindRef::Media(_)
    ));
    assert!(matches!(
        scoped_context.parent().unwrap().kind(),
        CssRuleContextKindRef::Scope { .. }
    ));
    assert_output(
        normalized_conditional,
        Descriptor::Symbols,
        "linear-gradient(red, blue)",
        "symbols: linear-gradient(red, blue);",
    );
    assert_output(
        normalized_scoped,
        Descriptor::Prefix,
        "radial-gradient(red, blue)",
        "prefix: radial-gradient(red, blue); symbols: a;",
    );
    assert_eq!(report.syntax(), &retained);
}

#[test]
fn image_descriptors_do_not_resolve_extended_fallback_or_spoken_names() {
    let rule = clean_rule(
        "system:extends Missing; prefix:linear-gradient(red, blue); fallback:Unknown; speak-as:Voice;",
    );
    assert!(
        matches!(rule.descriptors().system().unwrap().ordinary_system(),
        CssCounterStyleSystem::Extends(name) if name.as_str() == "Missing")
    );
    assert_eq!(
        rule.descriptors()
            .fallback()
            .unwrap()
            .ordinary_fallback()
            .as_str(),
        "Unknown"
    );
    assert!(
        matches!(rule.descriptors().speak_as().unwrap().ordinary_speak_as(),
        CssCounterStyleSpeakAs::CounterStyle(name) if name.as_str() == "Voice")
    );
    assert!(rule.descriptors().range().is_none());
    assert_output(
        &rule,
        Descriptor::Prefix,
        "linear-gradient(red, blue)",
        "system: extends Missing; prefix: linear-gradient(red, blue); fallback: Unknown; speak-as: Voice;",
    );
}

#[test]
fn later_valid_image_duplicate_selects_the_last_value_without_mutation() {
    let rule = clean_rule(
        "symbols:a; prefix:linear-gradient(red, blue); prefix:radial-gradient(red, blue);",
    );
    assert_output(
        &rule,
        Descriptor::Prefix,
        "radial-gradient(red, blue)",
        "prefix: radial-gradient(red, blue); symbols: a;",
    );
    let first = rule
        .descriptors()
        .occurrences()
        .find_map(|occurrence| match occurrence {
            surgeist_css::CssCounterStyleDescriptorRef::Prefix(value) => Some(value),
            _ => None,
        })
        .unwrap();
    assert!(
        first.position().byte_offset().value()
            < rule
                .descriptors()
                .prefix()
                .unwrap()
                .position()
                .byte_offset()
                .value()
    );
}

#[test]
fn suppressed_gradient_children_share_the_complete_rule_node_budget() {
    let rule = clean_rule("symbols:linear-gradient(red, blue); symbols:a;");
    let retained = rule.clone();
    let expected = "@counter-style ImageCase { symbols: a; }";
    // Existing project accounting: rule/name 2; two occurrences 2; two
    // symbol lists 2; gradient/stops/two stop-color pairs 6; identifier 1.
    // Suppression changes final bytes, not input or projection node costs.
    assert_eq!(
        rule.to_specified_css_with_limits(Limits::new(13, 13, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            Limits::new(12, 13, expected.len()),
            SerializationError::InputNodeLimit,
        ),
        (
            Limits::new(13, 12, expected.len()),
            SerializationError::ProjectionNodeLimit,
        ),
        (
            Limits::new(13, 13, expected.len() - 1),
            SerializationError::ByteLimit,
        ),
    ] {
        assert_eq!(
            rule.to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(
        rule.to_specified_css_with_limits(Limits::new(13, 13, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(rule, retained);
}

// Passing controls protect the distinctions that image admission must preserve.
#[test]
fn url_src_and_modifier_symbols_keep_function_and_resource_identity() {
    let rule = clean_rule(r#"symbols: url(mark.svg) src("other.svg" Hint mode(a));"#);
    assert_output(
        &rule,
        Descriptor::Symbols,
        r#"url("mark.svg") src("other.svg" Hint mode(a))"#,
        r#"symbols: url("mark.svg") src("other.svg" Hint mode(a));"#,
    );
}

#[test]
fn none_auto_span_and_custom_case_remain_identifier_symbols() {
    let rule = clean_rule(r#"symbols: none AuTo SpAn MiXeD \6e one;"#);
    assert_output(
        &rule,
        Descriptor::Symbols,
        "none AuTo SpAn MiXeD none",
        "symbols: none AuTo SpAn MiXeD none;",
    );
    let decoded: Vec<_> = rule
        .descriptors()
        .symbols()
        .unwrap()
        .ordinary_symbols()
        .symbols()
        .iter()
        .map(|value| {
            let CssCounterSymbol::Ident(value) = value else {
                panic!("identifier symbol");
            };
            value.as_str()
        })
        .collect();
    assert_eq!(decoded, ["none", "AuTo", "SpAn", "MiXeD", "none"]);
}

#[test]
fn escaped_identifier_and_string_symbols_preserve_decoded_content_and_case() {
    let rule = clean_rule(r#"symbols: "a\a b" \31 Name \4d iXeD;"#);
    assert_output(
        &rule,
        Descriptor::Symbols,
        r#""a\a b" \31 Name MiXeD"#,
        r#"symbols: "a\a b" \31 Name MiXeD;"#,
    );
    assert!(
        matches!(rule.descriptors().symbols().unwrap().ordinary_symbols().symbols(),
        [CssCounterSymbol::String(text), CssCounterSymbol::Ident(name), CssCounterSymbol::Ident(case)]
            if text.as_str() == "a\nb" && name.as_str() == "1Name" && case.as_str() == "MiXeD")
    );
}

#[test]
fn reserved_identifier_symbols_reject_the_whole_descriptor() {
    for symbol in [
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
        "default",
        r"\69 nherit",
    ] {
        let source = format!(
            "@counter-style ImageCase {{ symbols:a; prefix:{symbol}; suffix:'.'; }} .after{{}}"
        );
        let report = parse_sheet(&source);
        let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("recovered counter and sibling");
        };
        assert!(rule.descriptors().prefix().is_none(), "{symbol}");
        assert_output(rule, Descriptor::Prefix, "", "suffix: \".\"; symbols: a;");
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejected descriptor");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    }
}

#[test]
fn unselected_image_functions_are_not_reinterpreted_as_identifiers() {
    for function in [
        "image(transparent)",
        "image-set(url(a.svg) 1x)",
        "cross-fade(url(a.svg), url(b.svg), 50%)",
        "element(#marker)",
        "repeating-conic-gradient(red, blue)",
        "future-image(red)",
    ] {
        let source = format!(
            "/*😀*/\n@counter-style ImageCase {{ symbols:a; prefix:{function}; suffix:'.'; }} .after{{}}"
        );
        let report = parse_sheet(&source);
        let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("recovered counter and sibling");
        };
        assert!(rule.descriptors().prefix().is_none(), "{function}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejected whole image descriptor");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        let start = source.find("prefix:").unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + "prefix:".len() + function.len() + 1
        );
        let responsible = source.find(function).unwrap();
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible
        );
        assert_eq!(diagnostic.error().position().line().value(), 1);
        let line_start = source.find('\n').unwrap() + 1;
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            source[line_start..responsible].encode_utf16().count()
        );
        assert_output(rule, Descriptor::Prefix, "", "suffix: \".\"; symbols: a;");
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn malformed_images_drop_the_whole_value_instead_of_keeping_a_partial_list() {
    for value in [
        "url(good.svg) linear-gradient(red,)",
        "url(good.svg) future-image(red)",
    ] {
        let source = format!(
            "@counter-style ImageCase {{ symbols:a; symbols:{value}; suffix:'.'; }} .after{{}}"
        );
        let report = parse_sheet(&source);
        let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("earlier valid symbols and sibling");
        };
        assert_output(rule, Descriptor::Symbols, "a", "suffix: \".\"; symbols: a;");
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid complete descriptor");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    }
}

#[test]
fn too_many_negative_or_prefix_symbols_are_rejected_as_complete_values() {
    for descriptor in [
        "negative:url(a.svg) url(b.svg) url(c.svg);",
        "prefix:url(a.svg) url(b.svg);",
    ] {
        let source = format!(
            "@counter-style ImageCase {{ symbols:a; {descriptor} suffix:'.'; }} .after{{}}"
        );
        let report = parse_sheet(&source);
        let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("recovered counter and sibling");
        };
        assert!(rule.descriptors().negative().is_none());
        assert!(rule.descriptors().prefix().is_none());
        assert_output(rule, Descriptor::Symbols, "a", "suffix: \".\"; symbols: a;");
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid cardinality");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    }
}

#[test]
fn negative_pad_or_non_descending_additive_weights_reject_the_entire_descriptor() {
    for descriptor in [
        "pad:-1 url(a.svg);",
        "additive-symbols:10 url(a.svg), 10 url(b.svg);",
        "additive-symbols:1 url(a.svg), 2 url(b.svg);",
    ] {
        let source = format!(
            "@counter-style ImageCase {{ symbols:a; {descriptor} suffix:'.'; }} .after{{}}"
        );
        let report = parse_sheet(&source);
        let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("recovered counter and sibling");
        };
        assert!(rule.descriptors().pad().is_none());
        assert!(rule.descriptors().additive_symbols().is_none());
        assert_output(rule, Descriptor::Symbols, "a", "suffix: \".\"; symbols: a;");
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid integer domain or order");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    }
}

#[test]
fn omitted_descriptors_do_not_materialize_defaults_on_extended_styles() {
    let rule = clean_rule("system:extends External;");
    for descriptor in [
        Descriptor::Negative,
        Descriptor::Prefix,
        Descriptor::Suffix,
        Descriptor::Range,
        Descriptor::Pad,
        Descriptor::Fallback,
        Descriptor::Symbols,
        Descriptor::AdditiveSymbols,
        Descriptor::SpeakAs,
    ] {
        assert_eq!(rule.descriptor_specified_css(descriptor).unwrap(), "");
    }
    assert_output(
        &rule,
        Descriptor::System,
        "extends External",
        "system: extends External;",
    );
}

#[test]
fn image_nesting_limit_keeps_the_typed_stop_action_and_following_siblings() {
    let mut value = String::from("url(deep.svg)");
    for _ in 0..256 {
        value = format!("light-dark({value}, none)");
    }
    let source = format!(
        "@counter-style ImageCase {{ symbols:a; prefix:url(kept.svg); prefix:{value}; suffix:'.'; }} .after{{}}"
    );
    let report = parse_sheet(&source);
    let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("counter and sibling survive the resource failure");
    };
    assert_output(
        rule,
        Descriptor::Prefix,
        "url(\"kept.svg\")",
        "prefix: url(\"kept.svg\"); suffix: \".\"; symbols: a;",
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one resource stop: {:?}", report.diagnostics());
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    let start = source.find("prefix:light-dark(").unwrap();
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + "prefix:".len() + value.len() + 1
    );
    assert_eq!(
        validate_sheet(&source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
}
