//! Authored fill, rule, and span from Multicol 1 CR (2024-05-16) §§4, 6.1, 7.1:
//! https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/

use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::property_admission::assert_component_rejected;
use crate::css::support::property_admission::assert_declaration_rejected as rejected;
use crate::css::support::{
    checked_grammar_declaration as checked, important_declaration as declaration,
    selected_grammar as grammar,
};
use surgeist_css::CssParser;
use surgeist_css::color::CssColor;
use surgeist_css::component::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssValueOrigin, CssValueTokenRef,
};
use surgeist_css::conformance::{CssSupportStatus, property_support_metadata};
use surgeist_css::declaration::{CssDeclaration, CssGlobalKeyword, CssImportance};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::numeric::{CssCalculation, CssLength, CssNonNegative, CssNumeric};
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLonghandContribution, CssLonghandValue, CssLonghandValueRef,
    CssPropertyKindRef, CssPropertyNameRef,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::border::{CssLineStyle, CssLineWidth};
use surgeist_css::values::layout::{CssColumnFill, CssColumnRule, CssColumnSpan};

const NAMES: [&str; 6] = [
    "column-fill",
    "column-rule",
    "column-rule-color",
    "column-rule-style",
    "column-rule-width",
    "column-span",
];
const RULE_MEMBERS: [&str; 3] = [
    "column-rule-width",
    "column-rule-style",
    "column-rule-color",
];

fn accepted(name: &str, value: &str) {
    for source in [declaration(name, value), checked(name, value)] {
        assert_eq!(
            source.known().unwrap().property(),
            grammar(name).target_property()
        );
        assert_eq!(source.importance(), CssImportance::Important);
    }
}

fn expanded(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
    else {
        panic!(
            "terminal contributions for {:?}",
            source.known().unwrap().property()
        )
    };
    values.items().to_vec()
}

fn one_value(name: &str, value: &str) -> CssLonghandValue {
    let source = declaration(name, value);
    let items = expanded(&source);
    let [item] = items.as_slice() else {
        panic!("one terminal for {name}")
    };
    assert_eq!(item.property(), grammar(name).target_property());
    item.ordinary_value().unwrap().clone()
}

fn authored_color(value: &str) -> CssColor {
    let source = declaration("column-rule-color", value);
    let CssKnownPropertyValueRef::ColumnRuleColor(color) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed color")
    };
    color.value().clone()
}

fn programmatic_width(number: &str) -> CssLineWidth {
    CssLineWidth::Length(
        CssNumeric::<CssLength, CssNonNegative>::try_from_component(
            CssComponentValue::try_dimension(number, "px").unwrap(),
        )
        .unwrap(),
    )
}

#[test]
fn six_properties_have_dated_provenance_and_intrinsic_longhand_or_shorthand_shapes() {
    let mut seen = Vec::new();
    for name in NAMES {
        let property = grammar(name).target_property();
        assert_eq!(property.canonical_name(), name);
        assert!(
            !seen.contains(&property),
            "duplicate property identity: {name}"
        );
        seen.push(property);
        let support = property_support_metadata(name).unwrap();
        assert_eq!(support.property(), property);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(support.feature().source().id().as_str(), "O-MULTICOL1");
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2024/CR-css-multicol-1-20240516/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
        if name == "column-rule" {
            let CssPropertyKindRef::Shorthand(shorthand) = grammar(name).metadata().unwrap().kind()
            else {
                panic!("column-rule is a shorthand")
            };
            assert!(!shorthand.is_legacy());
            assert!(shorthand.reset_only_members().is_empty());
            assert_eq!(
                shorthand
                    .settable_members()
                    .iter()
                    .map(|member| member.known_property().canonical_name())
                    .collect::<Vec<_>>(),
                RULE_MEMBERS
            );
        } else {
            let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
            else {
                panic!("{name} is a longhand")
            };
            assert_eq!(longhand.property().known_property(), property);
            assert!(!longhand.inherited_by_default(), "{name} is non-inherited");
            let initial = longhand.initial_value();
            let CssInitialValueRef::Value(initial) = initial.view() else {
                panic!("{name} has a fixed initial value")
            };
            let initial_text = match name {
                "column-fill" => "balance",
                "column-rule-color" => "currentcolor",
                "column-rule-style" => "none",
                "column-rule-width" => "medium",
                "column-span" => "none",
                _ => unreachable!(),
            };
            assert_eq!(initial, &one_value(name, initial_text));
        }
    }
}

#[test]
fn fill_span_and_rule_longhands_accept_their_authored_grammars() {
    for value in ["auto", "balance", "balance-all"] {
        accepted("column-fill", value);
    }
    for value in ["none", "all"] {
        accepted("column-span", value);
    }
    for value in [
        "none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset",
        "outset",
    ] {
        accepted("column-rule-style", value);
    }
    for value in [
        "currentcolor",
        "red",
        "#123456",
        "rgb(10 20 30)",
        "lab(50 0 0)",
    ] {
        accepted("column-rule-color", value);
    }
    for value in [
        "thin",
        "medium",
        "thick",
        "0",
        "-0px",
        "1px",
        "1e100px",
        "1e-100px",
        "calc(1px + 2em)",
        "calc((-1px + 2px) * 3)",
    ] {
        accepted("column-rule-width", value);
    }
    for (name, value, token) in [
        ("column-fill", "justify", "justify"),
        ("column-span", "2", "2"),
        ("column-rule-style", "solid dashed", "dashed"),
        ("column-rule-color", "none", "none"),
        ("column-rule-width", "-1px", "-1px"),
        ("column-rule-width", "1%", "1%"),
        ("column-rule-width", "calc(1px + 2%)", "calc("),
    ] {
        rejected(name, value, Some(token));
    }
    assert_ne!(
        one_value("column-rule-width", "1e100px"),
        one_value("column-rule-width", "1e101px"),
        "distinct large finite authored widths must not round into one legacy float"
    );
}

#[test]
fn unordered_rule_components_expand_with_omissions_reset_to_initials() {
    for (authored, width, style, color) in [
        ("2px solid red", "2px", "solid", "red"),
        ("red solid 2px", "2px", "solid", "red"),
        ("dashed", "medium", "dashed", "currentcolor"),
        ("thin blue", "thin", "none", "blue"),
        ("currentcolor", "medium", "none", "currentcolor"),
    ] {
        let source = declaration("column-rule", authored);
        let items = expanded(&source);
        assert_eq!(items.len(), 3);
        for (item, (name, value)) in items.iter().zip([
            ("column-rule-width", width),
            ("column-rule-style", style),
            ("column-rule-color", color),
        ]) {
            assert_eq!(item.property(), grammar(name).target_property());
            assert_eq!(item.ordinary_value(), Some(&one_value(name, value)));
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
    for (value, token) in [
        ("", None),
        ("solid solid", Some("solid")),
        ("1px 2px", Some("2px")),
        ("red blue", Some("blue")),
        ("-1px solid", Some("-1px")),
        ("1% solid", Some("1%")),
    ] {
        rejected("column-rule", value, token);
    }
}

#[test]
fn globals_and_pending_values_expand_all_six_properties_with_strict_reentry() {
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        declaration("all", "initial").expand().unwrap()
    else {
        panic!("symbolic universal reset")
    };
    for name in NAMES {
        assert!(!reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())));
        let expected: &[&str] = if name == "column-rule" {
            &RULE_MEMBERS
        } else {
            &[name]
        };
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text);
            checked(name, text);
            let items = expanded(&source);
            assert_eq!(items.len(), expected.len());
            for (item, member) in items.iter().zip(expected.iter()) {
                assert_eq!(item.property(), grammar(member).target_property());
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
        for pending in ["var(--multicol)", "env(--multicol)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = source.expand().unwrap() else {
                panic!("{name} remains pending until substitution")
            };
            assert!(handle.source().same_occurrence(&source));
            assert!(
                handle
                    .reenter(
                        CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap()
                    )
                    .unwrap_err()
                    .is_residual_substitution()
            );
            let (valid, invalid_text) = match name {
                "column-fill" => ("balance-all", "justify"),
                "column-rule" => ("red dashed 2px", "red blue"),
                "column-rule-color" => ("lab(50 0 0)", "none"),
                "column-rule-style" => ("double", "solid dashed"),
                "column-rule-width" => ("1e100px", "1%"),
                "column-span" => ("all", "2"),
                _ => unreachable!(),
            };
            assert!(
                handle
                    .reenter(
                        CssComponentValues::parse(&CssParser::default(), invalid_text).unwrap()
                    )
                    .unwrap_err()
                    .replacement_error()
                    .is_some()
            );
            let replacement = CssComponentValues::parse(&CssParser::default(), valid).unwrap();
            for _ in 0..2 {
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("{name} reenters to terminal longhands")
                };
                assert_eq!(values.items().len(), expected.len());
                for (item, member) in values.items().iter().zip(expected.iter()) {
                    assert_eq!(item.property(), grammar(member).target_property());
                    assert!(item.ordinary_value().is_some());
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
            }
            let CssContributions::Longhands(values) = handle
                .reenter(CssComponentValues::parse(&CssParser::default(), "inherit").unwrap())
                .unwrap()
            else {
                panic!("{name} reenters a global keyword")
            };
            assert_eq!(values.items().len(), expected.len());
            for item in values.items() {
                assert_eq!(
                    item.value(),
                    CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                );
            }
        }
        rejected(name, "initial red", Some("red"));
    }
}

#[test]
fn normalization_keeps_mixed_order_and_fails_atomically_at_contribution_limit() {
    let report = CssParser::default().parse_sheet(concat!(
        ".a{column-fill:balance;column-rule:red solid 2px;",
        "column-span:all;column-rule-width:1%}",
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid width diagnostic: {:?}", report.diagnostics())
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let normalized = report.syntax().normalize(Default::default()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 3);
    for (index, (name, members)) in [("column-fill", 1), ("column-rule", 3), ("column-span", 1)]
        .into_iter()
        .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
        assert_eq!(
            declarations[index].source().known().unwrap().property(),
            grammar(name).target_property()
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declarations[index].expansion()
        else {
            panic!("normalized terminal for {name}")
        };
        assert_eq!(values.items().len(), members);
        for item in values.items() {
            assert!(item.source().same_occurrence(declarations[index].source()));
        }
    }
    let limit = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 3).unwrap();
    let error = report.syntax().normalize(limit).unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some((CssNormalizationResource::Contributions, 3))
    );
    assert_eq!(error.declaration_order(), Some(1));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        CssKnownProperty::ColumnRule
    );
}

#[test]
fn direct_rule_construction_keeps_exact_width_and_modern_color_without_legacy_projection() {
    assert!(CssColumnRule::try_new(None, None, None).is_none());
    assert_component_rejected(
        CssNumeric::<CssLength, CssNonNegative>::try_from_component,
        CssComponentValue::try_dimension("-1e-100", "px").unwrap(),
        NumericFailure::OutOfRange,
    );
    let width = programmatic_width("1e100");
    let color = authored_color("color-mix(in srgb, red, green, blue)");
    let rule = CssColumnRule::try_new(
        Some(width.clone()),
        Some(CssLineStyle::Dashed),
        Some(color.clone()),
    )
    .unwrap();
    assert_eq!(rule.width(), Some(&width));
    assert_eq!(rule.style(), Some(CssLineStyle::Dashed));
    assert_eq!(rule.color(), Some(&color));
    assert!(matches!(
        rule.width().unwrap().origin(),
        Some(CssValueOrigin::Programmatic)
    ));
    let CssLineWidth::Length(length) = rule.width().unwrap() else {
        panic!("exact width")
    };
    assert!(matches!(
        length.literal().map(|literal| literal.component()).map(CssComponentValue::view),
        Some(CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }))
            if number.representation() == "1e100" && unit == "px"
    ));
    assert!(length.calculation().is_none());
    let source = declaration(
        "column-rule",
        "1e100px dashed color-mix(in srgb, red, green, blue)",
    );
    let CssKnownPropertyValueRef::ColumnRule(parsed) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed parsed column rule")
    };
    assert_eq!(parsed.value(), &rule);
    assert!(matches!(
        parsed.value().width().unwrap().origin(),
        Some(CssValueOrigin::Parsed(_))
    ));
    let CssValueOrigin::Parsed(origin) = parsed.value().width().unwrap().origin().unwrap() else {
        unreachable!()
    };
    assert_eq!(origin.span().start().byte_offset().value(), 12);
    assert_eq!(origin.span().end().byte_offset().value(), 19);
    let modern_source = declaration("column-rule-color", "color-mix(in srgb, red, green, blue)");
    let CssKnownPropertyValueRef::ColumnRuleColor(modern) =
        modern_source.known().unwrap().property_value().unwrap()
    else {
        panic!("modern color wrapper")
    };
    assert_eq!(
        modern.value().color_mix_value().unwrap().components().len(),
        3
    );
    let legacy_source = declaration("column-rule-color", "red");
    let CssKnownPropertyValueRef::ColumnRuleColor(legacy) =
        legacy_source.known().unwrap().property_value().unwrap()
    else {
        panic!("named color wrapper")
    };
    assert_eq!(legacy.value().named().unwrap().name(), "red");
}

#[test]
fn calculated_rule_width_preserves_expression_and_source_origin() {
    let source = declaration("column-rule", "solid calc(1px * 2)");
    let CssKnownPropertyValueRef::ColumnRule(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed calculated rule")
    };
    assert_eq!(value.value().style(), Some(CssLineStyle::Solid));
    let programmatic_calc = CssComponentValue::try_function(
        "calc",
        CssComponentValues::try_new(
            vec![
                CssComponentValue::try_dimension("1", "px").unwrap(),
                CssComponentValue::try_token(" ").unwrap(),
                CssComponentValue::try_token("*").unwrap(),
                CssComponentValue::try_token(" ").unwrap(),
                CssComponentValue::try_token("2").unwrap(),
            ],
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap();
    let calculation = CssCalculation::<CssLength>::try_from_components(
        CssComponentValues::try_new(vec![programmatic_calc], Default::default()).unwrap(),
        Default::default(),
    )
    .unwrap();
    let constructed = CssColumnRule::try_new(
        Some(CssLineWidth::Length(
            CssNumeric::<CssLength, CssNonNegative>::try_from_calculation(calculation).unwrap(),
        )),
        Some(CssLineStyle::Solid),
        None,
    )
    .unwrap();
    for (rule, parsed_origin) in [(value.value(), true), (&constructed, false)] {
        let Some(CssLineWidth::Length(length)) = rule.width() else {
            panic!("exact calculated width")
        };
        assert!(
            length
                .literal()
                .map(|literal| literal.component())
                .is_none()
        );
        assert!(length.calculation().is_some());
        assert!(
            matches!(
                length.origin(),
                CssValueOrigin::Parsed(_) if parsed_origin
            ) || matches!(
                length.origin(),
                CssValueOrigin::Programmatic if !parsed_origin
            )
        );
        if let CssValueOrigin::Parsed(origin) = length.origin() {
            assert_eq!(origin.span().start().byte_offset().value(), 18);
            assert_eq!(origin.span().end().byte_offset().value(), 23);
        }
    }
}

#[test]
fn typed_initials_and_shorthand_contributions_are_independent_of_parsed_reference_values() {
    for name in NAMES.into_iter().filter(|name| *name != "column-rule") {
        let CssPropertyKindRef::Longhand(metadata) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} longhand")
        };
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("{name} fixed initial")
        };
        match (name, value.view()) {
            ("column-fill", CssLonghandValueRef::ColumnFill(CssColumnFill::Balance))
            | ("column-rule-style", CssLonghandValueRef::ColumnRuleStyle(CssLineStyle::None))
            | ("column-rule-width", CssLonghandValueRef::ColumnRuleWidth(CssLineWidth::Medium))
            | ("column-span", CssLonghandValueRef::ColumnSpan(CssColumnSpan::None)) => {}
            ("column-rule-color", CssLonghandValueRef::ColumnRuleColor(color))
                if color.is_current_color() => {}
            _ => panic!("wrong intrinsic initial for {name}"),
        }
    }
    let source = declaration("column-rule", "dashed");
    let items = expanded(&source);
    let [width, style, color] = items.as_slice() else {
        panic!("three rule members")
    };
    assert!(matches!(
        width.value(),
        CssContributionValueRef::Ordinary(CssLonghandValueRef::ColumnRuleWidth(
            CssLineWidth::Medium
        ))
    ));
    assert!(matches!(
        style.value(),
        CssContributionValueRef::Ordinary(CssLonghandValueRef::ColumnRuleStyle(
            CssLineStyle::Dashed
        ))
    ));
    assert!(matches!(
        color.value(),
        CssContributionValueRef::Ordinary(CssLonghandValueRef::ColumnRuleColor(value))
            if value.is_current_color()
    ));
}

#[test]
fn column_rule_canonical_output_omits_initials_and_charges_one_cumulative_budget() {
    let source = declaration("column-rule", "red solid 2px");
    let CssKnownPropertyValueRef::ColumnRule(reordered) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed reordered rule")
    };
    assert_eq!(
        reordered
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "2px solid red"
    );
    let current = authored_color("currentcolor");
    let all_initial = CssColumnRule::try_new(
        Some(CssLineWidth::Medium),
        Some(CssLineStyle::None),
        Some(current.clone()),
    )
    .unwrap();
    // The `<line-width> || <line-style> || <color>` family keeps `none`, its
    // shortest all-initial representation (CSSOM "serialize a CSS value"
    // step 2), as `border` and `outline` do. Each retained member still costs
    // one input and one projection node.
    assert_eq!(
        all_initial.serialize_specified(Default::default()).unwrap(),
        "none"
    );
    assert_eq!(
        all_initial
            .serialize_specified(CssSerializationLimits::new(4, 4, 4))
            .unwrap(),
        "none"
    );
    for (limits, expected) in [
        (
            CssSerializationLimits::new(3, 4, 4),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(4, 3, 4),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(4, 4, 3),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&all_initial.serialize_specified(limits).unwrap_err()),
            expected
        );
    }
    // An authored initial style is the retained member: aggregate and style
    // each cost one input and one projection node, with nothing synthesized.
    let style_only = CssColumnRule::try_new(None, Some(CssLineStyle::None), None).unwrap();
    assert_eq!(
        style_only.serialize_specified(Default::default()).unwrap(),
        "none"
    );
    assert_eq!(
        style_only
            .serialize_specified(CssSerializationLimits::new(2, 2, 4))
            .unwrap(),
        "none"
    );
    assert!(
        style_only
            .serialize_specified(CssSerializationLimits::new(2, 1, 4))
            .unwrap_err()
            .is_projection_node_limit()
    );
    // Without an authored style the `none` token is synthesized: it charges
    // one projection node and no input node.
    let synthetic = CssColumnRule::try_new(Some(CssLineWidth::Medium), None, None).unwrap();
    assert_eq!(
        synthetic.serialize_specified(Default::default()).unwrap(),
        "none"
    );
    assert_eq!(
        synthetic
            .serialize_specified(CssSerializationLimits::new(2, 3, 4))
            .unwrap(),
        "none"
    );
    for (limits, expected) in [
        (
            CssSerializationLimits::new(1, 3, 4),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(2, 2, 4),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(2, 3, 3),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&synthetic.serialize_specified(limits).unwrap_err()),
            expected
        );
    }
    let rule = CssColumnRule::try_new(
        Some(programmatic_width("2")),
        Some(CssLineStyle::None),
        Some(authored_color("red")),
    )
    .unwrap();
    assert_eq!(
        rule.serialize_specified(Default::default()).unwrap(),
        "2px red"
    );
    assert_eq!(
        rule.serialize_specified(CssSerializationLimits::new(4, 4, 7))
            .unwrap(),
        "2px red"
    );
}

#[test]
fn fill_and_span_keyword_output_is_bounded() {
    for (fill, text) in [
        (CssColumnFill::Auto, "auto"),
        (CssColumnFill::Balance, "balance"),
        (CssColumnFill::BalanceAll, "balance-all"),
    ] {
        assert_eq!(fill.serialize_specified(Default::default()).unwrap(), text);
        assert_eq!(
            fill.serialize_specified(CssSerializationLimits::new(1, 1, text.len(),))
                .unwrap(),
            text
        );
        assert!(
            fill.serialize_specified(CssSerializationLimits::new(1, 1, text.len() - 1,))
                .unwrap_err()
                .is_byte_limit()
        );
    }
    for (span, text) in [(CssColumnSpan::None, "none"), (CssColumnSpan::All, "all")] {
        assert_eq!(span.serialize_specified(Default::default()).unwrap(), text);
        assert_eq!(
            span.serialize_specified(CssSerializationLimits::new(1, 1, text.len(),))
                .unwrap(),
            text
        );
        assert!(
            span.serialize_specified(CssSerializationLimits::new(1, 1, text.len() - 1))
                .unwrap_err()
                .is_byte_limit()
        );
    }
    for (input_limit, projection_limit, error) in [
        (0, 1, SerializationFailure::InputNodeLimit),
        (1, 0, SerializationFailure::ProjectionNodeLimit),
    ] {
        let limits = CssSerializationLimits::new(input_limit, projection_limit, 32);
        for fill in [
            CssColumnFill::Auto,
            CssColumnFill::Balance,
            CssColumnFill::BalanceAll,
        ] {
            assert_eq!(
                serialization_failure(&fill.serialize_specified(limits).unwrap_err()),
                error
            );
        }
        for span in [CssColumnSpan::None, CssColumnSpan::All] {
            assert_eq!(
                serialization_failure(&span.serialize_specified(limits).unwrap_err()),
                error
            );
        }
    }
}

#[test]
fn nested_math_and_modern_color_share_the_rule_budget() {
    let source = declaration("column-rule-width", "calc(1px * 2)");
    let CssKnownPropertyValueRef::ColumnRuleWidth(width) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("calculated exact width")
    };
    let color = authored_color("color-mix(in srgb, red, green, blue)");
    let width_only = CssColumnRule::try_new(Some(width.value().clone()), None, None).unwrap();
    let color_only = CssColumnRule::try_new(None, None, Some(color.clone())).unwrap();
    let combined = CssColumnRule::try_new(Some(width.value().clone()), None, Some(color)).unwrap();
    let minimum_nodes = |rule: &CssColumnRule, input: bool| {
        (1..=256)
            .find(|count| {
                let limits = if input {
                    CssSerializationLimits::new(*count, usize::MAX, usize::MAX)
                } else {
                    CssSerializationLimits::new(usize::MAX, *count, usize::MAX)
                };
                rule.serialize_specified(limits).is_ok()
            })
            .unwrap()
    };
    for input in [true, false] {
        let width_nodes = minimum_nodes(&width_only, input);
        let color_nodes = minimum_nodes(&color_only, input);
        let combined_nodes = minimum_nodes(&combined, input);
        assert!(width_nodes > 2, "calculation has nested nodes");
        assert!(color_nodes > 2, "modern color has nested nodes");
        assert_eq!(combined_nodes, width_nodes + color_nodes - 1);
        let limits = if input {
            CssSerializationLimits::new(combined_nodes - 1, usize::MAX, usize::MAX)
        } else {
            CssSerializationLimits::new(usize::MAX, combined_nodes - 1, usize::MAX)
        };
        assert_eq!(
            serialization_failure(&combined.serialize_specified(limits).unwrap_err()),
            if input {
                SerializationFailure::InputNodeLimit
            } else {
                SerializationFailure::ProjectionNodeLimit
            }
        );
    }
    let expected = "calc(2px) color-mix(in srgb, red, green, blue)";
    assert_eq!(
        combined.serialize_specified(Default::default()).unwrap(),
        expected
    );
    assert_eq!(
        combined
            .serialize_specified(CssSerializationLimits::new(
                usize::MAX,
                usize::MAX,
                expected.len(),
            ))
            .unwrap(),
        expected
    );
    assert!(
        combined
            .serialize_specified(CssSerializationLimits::new(
                usize::MAX,
                usize::MAX,
                expected.len() - 1,
            ))
            .unwrap_err()
            .is_byte_limit()
    );
}
