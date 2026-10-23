//! Fonts 4 §9.2 authored palette definitions, not font lookup or evaluation.
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-palette-values

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::rules::{ExpectedRejection, invalid_descriptor};
use surgeist_css::{
    CssComponentValue, CssComponentValueLimits, CssComponentValueRef, CssComponentValues,
    CssErrorCode, CssFontPaletteBase, CssFontPaletteDescriptor,
    CssFontPaletteDescriptorKind as Kind, CssFontPaletteDescriptorValue,
    CssFontPaletteDescriptorValueRef, CssFontPaletteIndex, CssFontPaletteName,
    CssFontPaletteValuesRule, CssIntegerValue, CssNamespaceContext, CssNormalizedItem,
    CssRecoveryAction, CssRule, CssRuleContextKindRef, CssSerializationLimits, CssValueOrigin,
    normalize_sheet, parse_component_values, parse_font_palette_descriptor_value, parse_rule,
    parse_sheet,
};

#[test]
fn checked_construction_rejects_missing_family_negative_index_and_tight_component_budget() {
    assert!(
        CssFontPaletteName::try_new("brand")
            .unwrap_err()
            .is_invalid_name()
    );
    assert!(
        CssFontPaletteIndex::try_new(CssIntegerValue::Literal(
            surgeist_css::CssIntegerLiteral::from_i32(-1)
        ))
        .unwrap_err()
        .is_negative_index()
    );
    assert!(
        CssFontPaletteValuesRule::try_new(CssFontPaletteName::try_new("--brand").unwrap(), vec![])
            .unwrap_err()
            .is_missing_font_family()
    );

    let components = parse_component_values("dark").unwrap();
    let err = CssFontPaletteDescriptorValue::try_new(
        Kind::BasePalette,
        components,
        CssComponentValueLimits::try_new(256, 10, 3).unwrap(),
    )
    .unwrap_err();
    assert!(
        err.component_error()
            .is_some_and(|error| error.is_byte_limit()),
        "{err:?}"
    );
    assert!(std::error::Error::source(&err).is_some());
}

#[test]
fn palette_rule_fragment_admits_only_a_complete_named_definition() {
    let valid = "@font-palette-values --brand { font-family: Demo; base-palette: light; }";
    let report = parse_rule(valid, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(matches!(
        report.syntax(),
        Some(CssRule::FontPaletteValues(_))
    ));
    // Fonts 4 §9.2: `@font-palette-values <dashed-ident> { <declaration-list> }`.
    for (source, expected) in [
        (
            "@font-palette-values brand { font-family: Demo; }",
            Some(
                ExpectedRejection::new(CssErrorCode::InvalidAtRulePrelude)
                    .in_rule("font-palette-values")
                    .at("brand"),
            ),
        ),
        // Fonts 4 states no diagnostic for a missing font-family; the
        // rejection itself is the project's completeness policy.
        (
            "@font-palette-values --brand { base-palette: light; }",
            None,
        ),
        (
            "@font-palette-values --brand;",
            Some(
                ExpectedRejection::new(CssErrorCode::InvalidAtRuleBody)
                    .in_rule("font-palette-values"),
            ),
        ),
    ] {
        let report = parse_rule(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}");
        assert!(!report.is_clean(), "{source}");
        if let Some(expected) = expected {
            expected.assert_first(&report);
        }
    }
}

#[test]
fn palette_payload_survives_deep_scoped_chunk_conversion() {
    let source = format!(
        "@scope (.root) {{ {} @font-palette-values --deep {{ font-family: Demo; base-palette: 7; }} {} }}",
        "@media all {".repeat(70),
        "}".repeat(70)
    );
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let palettes = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::FontPaletteValues(rule) => Some(rule),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(palettes.len(), 1);
    assert_eq!(palettes[0].name().as_str(), "--deep");
    assert_eq!(palettes[0].descriptors().len(), 2);
    assert_eq!(
        palettes[0].descriptors()[1]
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "7"
    );
    assert_eq!(
        palettes[0].position().unwrap().byte_offset().value(),
        source.find("@font-palette-values --deep").unwrap()
    );
}

#[test]
fn recovered_raw_closure_is_distinct_from_strict_reentry() {
    let parsed = parse_font_palette_descriptor_value("var(--base", Kind::BasePalette);
    assert_eq!(parsed.diagnostics().len(), 1);
    assert_eq!(
        parsed.diagnostics()[0].action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    let pending = parsed.syntax().as_ref().unwrap();
    assert!(matches!(
        pending.view(),
        CssFontPaletteDescriptorValueRef::Pending(_)
    ));
    let [component] = pending.components().items() else {
        panic!("one function component");
    };
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("function component");
    };
    assert!(matches!(
        function.closing_origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));
    assert_eq!(
        pending
            .components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        "var(--base)"
    );
    let error = pending
        .reparse_after_substitution(
            parse_component_values("var(--base").unwrap(),
            Default::default(),
        )
        .unwrap_err();
    assert!(error.is_residual_substitution(), "{error:?}");
}

#[test]
fn palette_prelude_recovery_keeps_following_rule_and_real_position() {
    let source = "/* 😀 */\n@font-palette-values theme { font-family: Demo; }\n\
                  @font-palette-values --Good { font-family: Demo; }\n\
                  after { color: red; }";
    let report = parse_sheet(source);
    let [bad] = report.diagnostics() else {
        panic!("one bad prelude: {:?}", report.diagnostics());
    };
    assert_eq!(bad.error().code(), CssErrorCode::InvalidAtRulePrelude);
    assert_eq!(bad.action(), CssRecoveryAction::DropAtRule);
    let [CssRule::FontPaletteValues(palette), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("valid palette and following style remain in order");
    };
    assert_eq!(palette.name().as_str(), "--Good");
    assert_eq!(
        palette.position().unwrap().byte_offset().value(),
        source.find("@font-palette-values --Good").unwrap()
    );
    assert_eq!(
        palette.descriptors()[0]
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        source.rfind("font-family:").unwrap()
    );
}

#[test]
fn palette_admission_preserves_group_order_but_rejects_style_ancestors() {
    let definition = "@font-palette-values --inside { font-family: Demo; }";
    for (prefix, suffix) in [
        ("", ""),
        ("@media all {", "}"),
        ("@scope (.host) {", "}"),
        ("@scope (.host) { @media all {", "} }"),
    ] {
        let source = format!("{prefix}before{{}} {definition} after{{}}{suffix}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let kinds = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Rule(context) => Some(context.kind()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let palette_index = kinds
            .iter()
            .position(|kind| matches!(kind, CssRuleContextKindRef::FontPaletteValues(_)))
            .unwrap();
        assert!(matches!(
            kinds[palette_index - 1],
            CssRuleContextKindRef::Style(_) | CssRuleContextKindRef::ScopedStyle(_)
        ));
        assert!(matches!(
            kinds[palette_index + 1],
            CssRuleContextKindRef::Style(_) | CssRuleContextKindRef::ScopedStyle(_)
        ));
    }

    for (prefix, suffix) in [
        ("host {", "}"),
        ("host { @media all {", "} }"),
        ("@scope (.root) { host { @scope {", "} } }"),
    ] {
        let source = format!("{prefix}before{{}} {definition} after{{}}{suffix}");
        let report = parse_sheet(&source);
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropAtRule),
            "{source}: {:?}",
            report.diagnostics()
        );
        let normalized = normalize_sheet(report.syntax()).unwrap();
        assert!(normalized.items().iter().all(|item| !matches!(item, CssNormalizedItem::Rule(context) if matches!(context.kind(), CssRuleContextKindRef::FontPaletteValues(_)))));
    }
}

#[test]
fn palette_name_keeps_decoded_case_and_accepts_bare_dashes() {
    for (source, name) in [
        ("@font-palette-values -- { font-family: Demo; }", "--"),
        (
            r"@font-palette-values --Th\65 me { font-family: Demo; }",
            "--Theme",
        ),
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
            panic!("expected authored palette for {source}");
        };
        assert_eq!(rule.name().as_str(), name);
        assert_eq!(rule.descriptors().len(), 1);
        assert_eq!(rule.position().unwrap().byte_offset().value(), 0);
    }
}

#[test]
fn required_named_family_accepts_quoted_generic_but_not_bare_generic() {
    let accepted = parse_sheet(
        "@font-palette-values --named { font-family: Demo, \"serif\"; base-palette: dark; }",
    );
    assert!(accepted.is_clean(), "{:?}", accepted.diagnostics());
    let [CssRule::FontPaletteValues(rule)] = accepted.syntax().rules() else {
        panic!("expected palette definition");
    };
    let CssFontPaletteDescriptorValueRef::FontFamily(families) =
        rule.descriptors()[0].value().view()
    else {
        panic!("expected named family list");
    };
    assert_eq!(
        families.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
        ["Demo", "serif"]
    );
    assert!(matches!(
        rule.descriptors()[1].value().view(),
        CssFontPaletteDescriptorValueRef::BasePalette(CssFontPaletteBase::Dark)
    ));

    let rejected = parse_sheet(
        "@font-palette-values --invalid { font-family: Demo, serif; base-palette: light; } \
         .after { color: red; }",
    );
    assert!(matches!(rejected.syntax().rules(), [CssRule::Style(_)]));
    assert!(
        rejected
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropDescriptor)
    );
    assert!(
        rejected
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::DropAtRule)
    );
}

#[test]
fn descriptors_keep_occurrences_and_repeated_override_pairs() {
    let report = parse_sheet(
        "@font-palette-values --dupes { font-family: First; font-family: Second; \
         override-colors: 0 red, 0 color(display-p3 1 0 0), 2 alpha(from blue / 50%); \
         base-palette: +0007; }",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
        panic!("expected authored palette");
    };
    assert_eq!(
        rule.descriptors()
            .iter()
            .map(|d| d.value().kind())
            .collect::<Vec<_>>(),
        [
            Kind::FontFamily,
            Kind::FontFamily,
            Kind::OverrideColors,
            Kind::BasePalette
        ]
    );
    let CssFontPaletteDescriptorValueRef::OverrideColors(overrides) =
        rule.descriptors()[2].value().view()
    else {
        panic!("expected override list");
    };
    assert_eq!(overrides.len(), 3);
    assert_eq!(
        overrides[0]
            .index()
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "0"
    );
    assert_eq!(
        overrides[1]
            .index()
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "0"
    );
    assert_eq!(overrides[2].color().kind_name(), "alpha");
    let CssFontPaletteDescriptorValueRef::BasePalette(CssFontPaletteBase::Index(index)) =
        rule.descriptors()[3].value().view()
    else {
        panic!("expected exact base index");
    };
    assert_eq!(
        index
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "7"
    );
}

#[test]
fn ordinary_invalid_descriptors_drop_locally_and_later_valid_one_survives() {
    let report = parse_sheet(
        "@font-palette-values --recover { font-family: Demo; \
         override-colors: 0 red, 1 currentColor; \
         mystery: 9; base-palette: -1; \
         override-colors: 2 color(display-p3 1 0 0); }",
    );
    let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
        panic!("valid sibling descriptors retain the rule");
    };
    assert_eq!(rule.descriptors().len(), 2);
    assert_eq!(rule.descriptors()[1].value().kind(), Kind::OverrideColors);
    assert_eq!(report.diagnostics().len(), 3);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.action() == CssRecoveryAction::DropDescriptor)
    );
}

#[test]
fn override_colors_accepts_selected_absolute_forms_and_rejects_contextual_descendants() {
    for color in [
        "red",
        "#f00",
        "color(display-p3 1 0 0)",
        "color(--Profile 0.2 0.3 0.4)",
        "rgb(from red r g b)",
        "color-mix(red, blue)",
        "alpha(from red / 50%)",
    ] {
        let source = format!(
            "@font-palette-values --absolute {{ font-family: Demo; override-colors: 0 {color}; }}"
        );
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{color}: {:?}", report.diagnostics());
        let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
            panic!("expected absolute override: {color}");
        };
        assert_eq!(rule.descriptors().len(), 2, "{color}");
    }
    for color in [
        "currentColor",
        "Canvas",
        "color-mix(red, currentColor)",
        "alpha(from currentColor / 50%)",
    ] {
        let source = format!(
            "@font-palette-values --contextual {{ font-family: Demo; override-colors: 0 {color}; base-palette: light; }}"
        );
        let report = parse_sheet(&source);
        let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
            panic!("valid siblings must retain the rule: {color}");
        };
        assert_eq!(rule.descriptors().len(), 2, "{color}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropDescriptor),
            "{color}"
        );
    }
}

#[test]
fn substitution_defers_the_whole_descriptor_before_ordinary_grammar() {
    let report = parse_sheet(
        "@font-palette-values --pending { font-family: env(family); \
         base-palette: env(var(--name)); \
         override-colors: 0 var(--color); }",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
        panic!("pending family satisfies structural presence");
    };
    assert_eq!(rule.descriptors().len(), 3);
    assert!(rule.descriptors().iter().all(|d| matches!(
        d.value().view(),
        CssFontPaletteDescriptorValueRef::Pending(_)
    )));

    let descriptor = parse_font_palette_descriptor_value("0 var(--color)", Kind::OverrideColors);
    assert!(descriptor.is_clean(), "{:?}", descriptor.diagnostics());
    assert!(matches!(
        descriptor.syntax().as_ref().unwrap().view(),
        CssFontPaletteDescriptorValueRef::Pending(_)
    ));
    for source in ["var(bad) env(index)", "env(123) var(--ok)"] {
        let report = parse_font_palette_descriptor_value(source, Kind::BasePalette);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(matches!(
            report.syntax().as_ref().unwrap().view(),
            CssFontPaletteDescriptorValueRef::Pending(_)
        ));
    }
    // var() needs a <custom-property-name> and env() a <custom-ident>, so
    // neither defers and base-palette's ordinary grammar rejects var(.
    let neither = parse_font_palette_descriptor_value("var(bad) env(123)", Kind::BasePalette);
    assert!(neither.syntax().is_none());
    invalid_descriptor("font-palette-values", "base-palette")
        .at("var(")
        .assert_first(&neither);
}

#[test]
fn descriptors_do_not_apply_property_importance_globals_or_custom_fallbacks() {
    let report = parse_sheet(
        "@font-palette-values --restricted { font-family: Demo; \
         base-palette: dark !important; override-colors: inherit; \
         --base-palette: 1; base-palette: light; }",
    );
    let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
        panic!("valid family and final base retain rule");
    };
    assert_eq!(rule.descriptors().len(), 2);
    assert_eq!(rule.descriptors()[1].value().kind(), Kind::BasePalette);
    assert_eq!(report.diagnostics().len(), 3);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.action() == CssRecoveryAction::DropDescriptor)
    );
}

#[test]
fn index_grammar_keeps_exact_literals_and_delays_integer_math_ranges() {
    for (source, expected) in [
        ("-0", "0"),
        ("+0000000000000000000000000000009", "9"),
        ("calc(1 / 2)", "calc(0.5)"),
    ] {
        let report = parse_font_palette_descriptor_value(source, Kind::BasePalette);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .serialize_specified(Default::default())
                .unwrap(),
            expected,
            "{source}"
        );
    }
    // Fonts 4 §9.2.2: base-palette is light | dark | <integer [0,∞]>, so
    // the non-integer or negative token is responsible.
    let rejected = || invalid_descriptor("font-palette-values", "base-palette");
    for source in ["-1", "1.5", "1e2"] {
        let report = parse_font_palette_descriptor_value(source, Kind::BasePalette);
        assert!(report.syntax().is_none(), "{source}");
        rejected().at(source).assert_first(&report);
    }
    // <declaration-value> needs at least one token, and CSS-wide keywords
    // are property-value forms.
    for (source, expected) in [
        ("", rejected().at_end()),
        ("inherit", rejected().at("inherit")),
    ] {
        let report = parse_font_palette_descriptor_value(source, Kind::BasePalette);
        assert!(report.syntax().is_none(), "{source}");
        expected.assert_first(&report);
    }
    let negative_math = parse_font_palette_descriptor_value("calc(-1)", Kind::BasePalette);
    assert!(
        negative_math.is_clean(),
        "{:?}",
        negative_math.diagnostics()
    );
}

#[test]
fn scoped_palette_normalizes_as_nonstyle_rule_at_its_parent() {
    let source =
        "@scope (.root) { @media all { @font-palette-values --scope { font-family: Demo; } } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let palettes = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::FontPaletteValues(rule) => Some((context, rule)),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(palettes.len(), 1);
    assert!(palettes[0].0.parent().is_some());
    assert_eq!(palettes[0].1.name().as_str(), "--scope");
    assert!(
        normalized
            .items()
            .iter()
            .all(|item| matches!(item, CssNormalizedItem::Rule(_)))
    );
}

#[test]
fn canonical_values_and_rule_keep_duplicates_without_evaluating_colors() {
    let light = parse_font_palette_descriptor_value("LIGHT", Kind::BasePalette);
    assert!(light.is_clean(), "{:?}", light.diagnostics());
    assert_eq!(
        light
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "light"
    );
    let index = parse_font_palette_descriptor_value("+0007", Kind::BasePalette);
    assert!(index.is_clean(), "{:?}", index.diagnostics());
    assert_eq!(
        index
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "7"
    );

    let report = parse_sheet(
        "@font-palette-values -- { font-family: Demo; font-family: \"serif\"; \
         override-colors: 0 red, 0 color(display-p3 1 0 0), 2 alpha(from blue / 50%); }",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::FontPaletteValues(rule)] = report.syntax().rules() else {
        panic!("expected palette definition");
    };
    let expected = "@font-palette-values -- { font-family: Demo; font-family: \"serif\"; \
                    override-colors: 0 red, 0 color(display-p3 1 0 0), 2 alpha(from blue / 0.5); }";
    assert_eq!(
        rule.serialize_specified(Default::default()).unwrap(),
        expected
    );
    let reparsed = parse_sheet(expected);
    assert!(reparsed.is_clean(), "{:?}", reparsed.diagnostics());
    let [CssRule::FontPaletteValues(reparsed)] = reparsed.syntax().rules() else {
        panic!("canonical CSS must retain the palette");
    };
    assert_eq!(reparsed.name().as_str(), "--");
    assert_eq!(reparsed.descriptors().len(), 3);
    assert_eq!(
        reparsed.serialize_specified(Default::default()).unwrap(),
        expected
    );
}

#[test]
fn quoted_family_with_reserved_member_stays_quoted_after_canonicalization() {
    for name in ["Demo initial", "serif Demo"] {
        let source = format!("@font-palette-values --family {{ font-family: \"{name}\"; }}");
        let parsed = parse_sheet(&source);
        assert!(parsed.is_clean(), "{source}: {:?}", parsed.diagnostics());
        let [CssRule::FontPaletteValues(rule)] = parsed.syntax().rules() else {
            panic!("expected quoted family");
        };
        let canonical = rule.serialize_specified(Default::default()).unwrap();
        assert!(
            canonical.contains(&format!("font-family: \"{name}\";")),
            "{canonical}"
        );
        let reparsed = parse_sheet(&canonical);
        assert!(
            reparsed.is_clean(),
            "{canonical}: {:?}",
            reparsed.diagnostics()
        );
    }
}

#[test]
fn strict_reentry_rejects_residual_functions_before_grammar() {
    let pending = CssFontPaletteDescriptorValue::try_new(
        Kind::BasePalette,
        parse_component_values("env(index)").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(matches!(
        pending.view(),
        CssFontPaletteDescriptorValueRef::Pending(_)
    ));
    let residual = pending
        .reparse_after_substitution(
            parse_component_values(r"c\61 lc(var(--x))").unwrap(),
            Default::default(),
        )
        .unwrap_err();
    assert!(residual.is_residual_substitution(), "{residual:?}");

    let checked = pending
        .reparse_after_substitution(parse_component_values("12").unwrap(), Default::default())
        .unwrap();
    assert_eq!(
        checked.serialize_specified(Default::default()).unwrap(),
        "12"
    );
    let round_trip = parse_font_palette_descriptor_value(
        &checked.serialize_specified(Default::default()).unwrap(),
        Kind::BasePalette,
    );
    assert!(round_trip.is_clean(), "{:?}", round_trip.diagnostics());
    let Some(reparsed) = round_trip.syntax() else {
        panic!("canonical replacement must reenter ordinary grammar");
    };
    let CssFontPaletteDescriptorValueRef::BasePalette(CssFontPaletteBase::Index(index)) =
        reparsed.view()
    else {
        panic!("canonical replacement must remain an index");
    };
    assert_eq!(
        index
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "12"
    );
    assert!(
        checked
            .reparse_after_substitution(parse_component_values("13").unwrap(), Default::default())
            .unwrap_err()
            .is_not_pending()
    );
}

#[test]
fn pending_canonical_text_preserves_fallback_whitespace() {
    let source = "var(--choice,  2  )";
    let value = parse_font_palette_descriptor_value(source, Kind::BasePalette);
    assert!(value.is_clean(), "{:?}", value.diagnostics());
    let value = value.syntax().as_ref().unwrap();
    assert!(matches!(
        value.view(),
        CssFontPaletteDescriptorValueRef::Pending(_)
    ));
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        source
    );
    let reparsed = parse_font_palette_descriptor_value(
        &value.serialize_specified(Default::default()).unwrap(),
        Kind::BasePalette,
    );
    assert!(reparsed.is_clean(), "{:?}", reparsed.diagnostics());
    assert!(matches!(
        reparsed.syntax().as_ref().unwrap().view(),
        CssFontPaletteDescriptorValueRef::Pending(_)
    ));
}

#[test]
fn checked_components_preserve_mixed_origins_without_fabricating_occurrence_positions() {
    let mut components = parse_component_values("0 ").unwrap().items().to_vec();
    components.push(CssComponentValue::try_ident("red").unwrap());
    let value = CssFontPaletteDescriptorValue::try_new(
        Kind::OverrideColors,
        CssComponentValues::try_new(components, Default::default()).unwrap(),
        Default::default(),
    )
    .unwrap();
    let items = value.components().items();
    assert!(matches!(items[0].origin(), CssValueOrigin::Parsed(_)));
    assert!(matches!(
        items.last().unwrap().origin(),
        CssValueOrigin::Programmatic
    ));
    let occurrence = CssFontPaletteDescriptor::new(value);
    assert_eq!(occurrence.position(), None);
    let CssFontPaletteDescriptorValueRef::OverrideColors(pairs) = occurrence.value().view() else {
        panic!("expected one checked override pair");
    };
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].color().kind_name(), "named");
}

#[test]
fn whole_rule_and_sheet_share_limits_with_a_style_sibling() {
    let source = "@font-palette-values --one { font-family: Demo; override-colors: 0 red, 1 blue; }\n\
                  @font-palette-values --two { font-family: Other; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [
        CssRule::FontPaletteValues(first),
        CssRule::FontPaletteValues(second),
    ] = report.syntax().rules()
    else {
        panic!("expected two authored palettes");
    };
    let first_css = first.serialize_specified(Default::default()).unwrap();
    let second_css = second.serialize_specified(Default::default()).unwrap();
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        format!("{first_css}\n{second_css}")
    );

    let whole = format!("{first_css}\n{second_css}");
    let err = report
        .syntax()
        .serialize_specified(CssSerializationLimits::new(1000, 1000, whole.len() - 1))
        .unwrap_err();
    assert!(err.is_byte_limit(), "{err:?}");
    assert_eq!(err.rule_index(), Some(1));

    let simple = parse_sheet(
        "@font-palette-values --a { font-family: Demo; }\n\
         @font-palette-values --b { font-family: Other; }",
    );
    assert!(simple.is_clean());
    let [first_simple, second_simple] = simple.syntax().rules() else {
        panic!("expected two simple palette rules");
    };
    for (limits, expected) in [
        (
            CssSerializationLimits::new(4, 100, 1000),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(100, 4, 1000),
            SerializationFailure::ProjectionNodeLimit,
        ),
    ] {
        assert!(first_simple.serialize_specified(limits).is_ok());
        assert!(second_simple.serialize_specified(limits).is_ok());
        let err = simple.syntax().serialize_specified(limits).unwrap_err();
        assert_eq!(serialization_failure(&err), expected);
        assert_eq!(err.rule_index(), Some(1));
    }

    let mixed = parse_sheet(&format!("{source}\n.after {{ color: red; }}"));
    assert!(mixed.is_clean(), "{:?}", mixed.diagnostics());
    let before = mixed.clone();
    assert_eq!(
        mixed
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        "@font-palette-values --one { font-family: Demo; override-colors: 0 red, 1 blue; }\n@font-palette-values --two { font-family: Other; }\n.after { color: red; }"
    );
    assert_eq!(mixed, before);
}
