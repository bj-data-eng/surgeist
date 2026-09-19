#![forbid(unsafe_code)]
//! Specified opacity transport preserves symbolic values and declaration identity.
//! Oracle: catalog-pinned Color 4 (2026-09-08), §§3.2–3.3; existing intrinsic
//! expansion/normalization contracts. Values below are specified, not computed.
//! Raw authored text assertions do not claim canonical serialization (§17).
use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    value.clone()
}

fn ordinary(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("opacity has intrinsic expansion")
    else {
        panic!("completed ordinary longhand")
    };
    let [value] = values.items() else {
        panic!("one terminal contribution")
    };
    assert_eq!(value.property(), CssKnownProperty::Opacity);
    assert!(matches!(
        value.value(),
        CssContributionValueRef::Ordinary(_)
    ));
    assert_eq!(
        value.ordinary_value().unwrap().property().known_property(),
        CssKnownProperty::Opacity
    );
    assert!(value.source().same_occurrence(source));
    values
}

fn authored_opacity(source: &CssDeclaration) -> &CssOpacityValue {
    let CssKnownPropertyValueRef::Opacity(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary opacity source")
    };
    value.value()
}

fn normalized_declarations(sheet: &CssNormalizedSheet) -> Vec<&CssNormalizedDeclaration> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect()
}

#[test]
fn opacity_has_noninherited_terminal_metadata_and_ordinary_initial() {
    let metadata = CssKnownProperty::Opacity
        .metadata()
        .expect("opacity metadata");
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("terminal property")
    };
    assert!(!longhand.inherited_by_default());
    let initial = longhand.initial_value();
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::Opacity
    );
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("fixed ordinary initial")
    };
    let literal_one = declaration("opacity:1");
    assert!(
        matches!(authored_opacity(&literal_one), CssOpacityValue::Literal(value) if value.value() == 1.0)
    );
    let values = ordinary(&literal_one);
    // Consistency evidence only: both outputs could share a wrong transport.
    // New-variant tests must independently assert the initial's numeric one.
    assert_eq!(initial, values.items()[0].ordinary_value().unwrap());

    let CssPropertyKindRef::Longhand(color) = CssKnownProperty::Color.metadata().unwrap().kind()
    else {
        panic!("color longhand")
    };
    assert!(color.inherited_by_default());
    let color_initial = color.initial_value();
    let CssInitialValueRef::Value(color_initial) = color_initial.view() else {
        panic!("symbolic color")
    };
    let CssLonghandValueRef::Color(color) = color_initial.view() else {
        panic!("color initial payload")
    };
    assert_eq!(color.system(), Some(CssAuthoredSystemColor::CanvasText));
}

#[test]
fn opacity_expansion_accepts_and_keeps_distinct_authored_numeric_branches() {
    let cases = [
        "0.5",
        "-0.5",
        "1.5",
        "-25%",
        "150%",
        "calc(1 / 2)",
        "calc(25% + 25%)",
    ];
    let mut payloads = Vec::new();
    for (index, value) in cases.iter().enumerate() {
        let input = format!("opacity:{value}!important");
        let source = declaration(&input);
        match (index, authored_opacity(&source)) {
            (0, CssOpacityValue::Literal(value)) => assert_eq!(value.value(), 0.5),
            (1, CssOpacityValue::Number(value)) => assert_eq!(value.value(), -0.5),
            (2, CssOpacityValue::Number(value)) => assert_eq!(value.value(), 1.5),
            (3, CssOpacityValue::Percentage(value)) => assert_eq!(value.value(), -25.0),
            (4, CssOpacityValue::Percentage(value)) => assert_eq!(value.value(), 150.0),
            (5, CssOpacityValue::Calculation(_))
            | (6, CssOpacityValue::PercentageCalculation(_)) => {}
            _ => panic!("independently specified authored numeric branch: {input}"),
        }
        let values = ordinary(&source);
        let contribution = &values.items()[0];
        assert_eq!(contribution.source().importance(), CssImportance::Important);
        assert!(contribution.replacement_components().is_none());
        assert_eq!(
            contribution
                .source()
                .value_components()
                .serialize()
                .unwrap()
                .as_css(),
            *value
        );
        let origin = contribution.source().parsed_value().unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), 8);
        assert_eq!(origin.span().end().byte_offset().value(), 8 + value.len());
        assert!(
            origin
                .source()
                .same_snapshot(source.parsed_value().unwrap().source())
        );
        let payload = contribution.ordinary_value().unwrap().clone();
        // Exact source branch retention above and distinct output payloads here
        // do not substitute for direct new-variant payload assertions later.
        assert!(payloads.iter().all(|earlier| earlier != &payload));
        payloads.push(payload);
    }
}

#[test]
fn opacity_css_wide_values_emit_one_symbolic_terminal_without_resolving_it() {
    for (text, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("opacity:{text}!important"));
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("one completed terminal set")
        };
        let [value] = values.items() else {
            panic!("one opacity contribution")
        };
        assert_eq!(value.property(), CssKnownProperty::Opacity);
        assert_eq!(value.value(), CssContributionValueRef::Global(expected));
        assert!(value.ordinary_value().is_none());
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert!(value.replacement_components().is_none());
    }
}

#[test]
fn opacity_pending_reentry_is_strict_atomic_and_reusable() {
    let source = declaration("opacity:var(--fade)!important");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending opacity")
    };
    assert!(handle.source().same_occurrence(&source));
    for input in ["var(--again)", "calc(var(--again) + 1)"] {
        let error = handle
            .reenter(parse_component_values(input).unwrap())
            .unwrap_err();
        assert_eq!(error.kind(), &CssExpansionErrorKind::ResidualSubstitution);
    }
    for input in ["1px", "0.5 0.6", "inherit extra", "0.5!important"] {
        let error = handle
            .reenter(parse_component_values(input).unwrap())
            .unwrap_err();
        assert!(
            matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(_)),
            "{input}: {error:?}"
        );
    }
    let replacement = parse_component_values("150%").unwrap();
    let expected = ordinary(&declaration("opacity:150%"));
    for _ in 0..2 {
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed replacement")
        };
        let [value] = values.items() else {
            panic!("one replacement contribution")
        };
        assert_eq!(value.property(), CssKnownProperty::Opacity);
        assert_eq!(value.ordinary_value(), expected.items()[0].ordinary_value());
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        let actual = value.replacement_components().unwrap();
        assert_eq!(actual, &replacement);
        let CssValueOrigin::Parsed(actual_origin) = actual.items()[0].origin() else {
            panic!("replacement origin")
        };
        let CssValueOrigin::Parsed(expected_origin) = replacement.items()[0].origin() else {
            panic!("input origin")
        };
        assert!(
            actual_origin
                .source()
                .same_snapshot(expected_origin.source())
        );
    }
    let CssContributions::Longhands(values) = handle
        .reenter(parse_component_values("inherit").unwrap())
        .unwrap()
    else {
        panic!("global reentry")
    };
    assert_eq!(values.items().len(), 1);
    assert_eq!(
        values.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
    );
    assert!(handle.source().same_occurrence(&source));
}

#[test]
fn recovered_child_fixture_normalizes_original_opacity_with_order_and_context() {
    // Preserve the exact original R53/structured_rules fixture.
    let source = ".card { color: red; .bad, { color: blue; } color: green; .child { opacity: 1; } color: black; }";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropQualifiedRule
    );
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent style")
    };
    let [
        CssRule::NestedDeclarations(green),
        CssRule::Style(child),
        CssRule::NestedDeclarations(black),
    ] = parent.rules()
    else {
        panic!("retained green, child, black")
    };
    let originals = [
        &parent.declarations()[0],
        &green.declarations()[0],
        &child.declarations()[0],
        &black.declarations()[0],
    ];
    let normalized = normalize_report(&report).expect("color and opacity both normalize");
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    let values = normalized_declarations(normalized.syntax());
    assert_eq!(values.len(), 4);
    for (index, ((property, text), original)) in [
        (CssKnownProperty::Color, "red"),
        (CssKnownProperty::Color, "green"),
        (CssKnownProperty::Opacity, "1"),
        (CssKnownProperty::Color, "black"),
    ]
    .into_iter()
    .zip(originals)
    .enumerate()
    {
        let value = values[index];
        assert_eq!(value.order(), index);
        assert!(value.source().same_occurrence(original));
        assert_eq!(value.source().known().unwrap().property(), property);
        assert_eq!(
            value
                .source()
                .value_components()
                .serialize()
                .unwrap()
                .as_css(),
            format!(" {text}")
        );
        assert_eq!(
            value.source().position().unwrap().byte_offset().value(),
            source
                .find(&format!("{}: {text}", property.canonical_name()))
                .unwrap()
        );
        let CssExpansion::Contributions(CssContributions::Longhands(items)) = value.expansion()
        else {
            panic!("completed longhand")
        };
        assert_eq!(items.items().len(), 1);
        assert_eq!(items.items()[0].property(), property);
        assert!(items.items()[0].source().same_occurrence(original));
    }
    for index in [1, 3] {
        assert!(matches!(
            values[index].rule_context().kind(),
            CssRuleContextKindRef::NestedDeclarations(_)
        ));
        assert!(
            values[index]
                .rule_context()
                .parent()
                .unwrap()
                .same_context(values[0].rule_context())
        );
        assert!(
            values[index]
                .selector_context()
                .same_context(values[0].selector_context())
        );
    }
    assert!(
        !values[1]
            .rule_context()
            .same_context(values[3].rule_context())
    );
    assert!(matches!(
        values[2].rule_context().kind(),
        CssRuleContextKindRef::Style(_)
    ));
    assert!(
        values[2]
            .rule_context()
            .parent()
            .unwrap()
            .same_context(values[0].rule_context())
    );
    assert!(
        values[2]
            .selector_context()
            .parent()
            .unwrap()
            .same_context(values[0].selector_context())
    );
    assert!(
        !values[2]
            .selector_context()
            .same_context(values[0].selector_context())
    );
}

#[test]
fn opacity_normalization_consumes_one_declaration_and_one_contribution() {
    let source = ".p{opacity:150%!important}";
    let report = parse_sheet(source);
    assert!(report.is_clean());
    for (limits, resource, limit, offset) in [
        (
            CssNormalizationLimits::try_new(0, 0, 1, 1).unwrap(),
            CssNormalizationResource::Rules,
            0,
            0,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 0, 1).unwrap(),
            CssNormalizationResource::Declarations,
            0,
            3,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 1, 0).unwrap(),
            CssNormalizationResource::Contributions,
            0,
            3,
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit }
        );
        assert_eq!(error.position().unwrap().byte_offset().value(), offset);
    }
    let normalized = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap(),
    )
    .unwrap();
    let values = normalized_declarations(&normalized);
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].order(), 0);
    assert_eq!(values[0].source().importance(), CssImportance::Important);
    assert!(
        matches!(authored_opacity(values[0].source()), CssOpacityValue::Percentage(value) if value.value() == 150.0)
    );
    let pending = parse_sheet(".p{opacity:var(--fade)}");
    assert!(pending.is_clean());
    let normalized = normalize_sheet_with_limits(
        pending.syntax(),
        CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap(),
    )
    .unwrap();
    let values = normalized_declarations(&normalized);
    assert_eq!(values.len(), 1);
    assert!(matches!(values[0].expansion(), CssExpansion::Pending(_)));
}
