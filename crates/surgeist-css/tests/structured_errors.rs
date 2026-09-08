mod common;

use common::CssParseReportTestExt;
use surgeist_css::{
    CssDeclarationContextRef, CssErrorCode, CssKnownProperty, CssRecoveryAction, CssRule,
    CssTokenKind, ErrorKind, parse_sheet, parse_style_attribute,
};

#[test]
fn invalid_core_font_value_has_exact_property_diagnostic_and_retains_its_sibling() {
    let source = "font-size: -1px; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("negative font size must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 16);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected property value error");
    };
    assert_eq!(detail.property(), CssKnownProperty::FontSize);
    let encountered = detail.encountered().expect("responsible negative length");
    assert_eq!(encountered.kind(), CssTokenKind::Dimension);
    assert_eq!(encountered.authored(), "-1px");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color,
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects the recovered font size");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn counter_style_descriptor_value_and_combination_errors_preserve_typed_context() {
    let value_source = concat!(
        "@counter-style kept { system: additive; ",
        "additive-symbols: 10 X, 10 I; additive-symbols: 10 X, 1 I; }",
    );
    let value_report = parse_sheet(value_source);
    assert!(matches!(
        value_report.syntax().rules(),
        [CssRule::CounterStyle(_)]
    ));
    let [value_diagnostic] = value_report.diagnostics() else {
        panic!("expected one invalid additive ordering diagnostic")
    };
    assert_eq!(
        value_diagnostic.error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    assert_eq!(value_diagnostic.action(), CssRecoveryAction::DropDescriptor);
    assert_eq!(
        value_diagnostic.error().position().byte_offset().value(),
        value_source.match_indices("10 I").next().unwrap().0
    );
    let ErrorKind::InvalidDescriptorValue(value) = value_diagnostic.error().kind() else {
        panic!("expected typed descriptor-value error")
    };
    assert_eq!(value.at_rule().as_str(), "counter-style");
    assert_eq!(value.descriptor().as_str(), "additive-symbols");

    let combination_source =
        "@counter-style inherited { system: extends decimal; symbols: x; } .after {}";
    let combination_report = parse_sheet(combination_source);
    assert!(matches!(
        combination_report.syntax().rules(),
        [CssRule::Style(_)]
    ));
    let [combination_diagnostic] = combination_report.diagnostics() else {
        panic!("expected one invalid effective-combination diagnostic")
    };
    assert_eq!(
        combination_diagnostic.error().code(),
        CssErrorCode::InvalidDescriptorCombination
    );
    assert_eq!(
        combination_diagnostic.action(),
        CssRecoveryAction::DropAtRule
    );
    assert_eq!(
        combination_diagnostic
            .error()
            .position()
            .byte_offset()
            .value(),
        combination_source.find("system").unwrap()
    );
    let ErrorKind::InvalidDescriptorCombination(combination) =
        combination_diagnostic.error().kind()
    else {
        panic!("expected typed descriptor-combination error")
    };
    assert_eq!(combination.at_rule().as_str(), "counter-style");
    assert_eq!(combination.responsible().as_str(), "system");
    assert_eq!(combination.conflicting()[0].as_str(), "symbols");
}

#[test]
fn selectors3_invalid_language_and_pseudo_element_sequences_drop_exact_rules() {
    let failures = [
        ".empty:lang() { color: black; }",
        ".string:lang(\"en\") { color: black; }",
        ".many:lang(en fr) { color: black; }",
        ".later:marker { color: black; }",
        ".terminal::first-line:hover { color: black; }",
        ".sequence::first-letter::marker { color: black; }",
    ];
    let source = format!(
        ".before {{ color: red; }} {} .after {{ color: blue; }}",
        failures.join(" ")
    );
    let report = parse_sheet(&source);

    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Style(_), CssRule::Style(_)]
    ));
    assert_eq!(report.diagnostics().len(), failures.len());
    for (diagnostic, failure) in report.diagnostics().iter().zip(failures) {
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
        let start = source.find(failure).expect("failed rule start");
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + failure.len()
        );
        let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
            panic!("expected selector error for {failure}")
        };
        assert_eq!(
            detail.production().expect("selector production").as_str(),
            "baseline.selector.complex"
        );
        assert_eq!(detail.expectation().as_str(), "a supported selector");
    }
}

#[test]
fn font_feature_list_separator_error_has_exact_identity_span_and_recovery() {
    let source = r#"font-feature-settings: "kern",, "liga" on; color: red"#;
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("empty feature-list item must recover once");
    };
    let responsible = source.find(",,").expect("adjacent separators") + 1;
    let declaration_end = source.find(';').expect("declaration terminator") + 1;
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        declaration_end
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected property value error");
    };
    assert_eq!(detail.property(), CssKnownProperty::FontFeatureSettings);
    let encountered = detail.encountered().expect("responsible second comma");
    assert_eq!(encountered.kind(), CssTokenKind::Comma);
    assert_eq!(encountered.authored(), ",");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color,
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects the recovered empty list item");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn duplicate_font_synthesis_component_has_exact_identity_span_and_recovery() {
    let source = "font-synthesis: weight weight; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("duplicate synthesis component must recover once");
    };
    let responsible = source.match_indices("weight").nth(1).unwrap().0;
    let declaration_end = source.find(';').unwrap() + 1;
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        declaration_end
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected property value error");
    };
    assert_eq!(detail.property(), CssKnownProperty::FontSynthesis);
    let encountered = detail.encountered().expect("responsible duplicate keyword");
    assert_eq!(encountered.kind(), CssTokenKind::Ident);
    assert_eq!(encountered.authored(), "weight");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color,
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects duplicate synthesis components");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn conflicting_font_variant_group_has_exact_identity_span_and_recovery() {
    let source = "font-variant-numeric: oldstyle-nums lining-nums; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("conflicting numeric figure forms must recover once");
    };
    let responsible = source.find("lining-nums").unwrap();
    let declaration_end = source.find(';').unwrap() + 1;
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        declaration_end
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected property value error");
    };
    assert_eq!(detail.property(), CssKnownProperty::FontVariantNumeric);
    let encountered = detail
        .encountered()
        .expect("responsible conflicting keyword");
    assert_eq!(encountered.kind(), CssTokenKind::Ident);
    assert_eq!(encountered.authored(), "lining-nums");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color,
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects conflicting variant groups");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn color_domain_failure_reports_the_responsible_component_and_retains_its_sibling() {
    let source = "color: hsl(20px 30% 40%); opacity: 0.5";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("invalid hue unit must recover once");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidColorSyntax(detail) = diagnostic.error().kind() else {
        panic!("expected structured color detail");
    };
    assert_eq!(detail.component().map(|value| value.as_str()), Some("hue"));
    assert_eq!(
        detail.encountered().unwrap().kind(),
        CssTokenKind::Dimension
    );
    assert_eq!(detail.encountered().unwrap().authored(), "20px");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered hue unit error");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn predefined_color_space_failure_reports_the_responsible_ident_and_retains_its_sibling() {
    let source = "color: color(--custom 1 2 3); opacity: 0.5";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("custom color profile must recover once");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidColorSyntax(detail) = diagnostic.error().kind() else {
        panic!("expected structured color detail");
    };
    assert_eq!(
        detail.component().map(|value| value.as_str()),
        Some("color space")
    );
    let encountered = detail.encountered().expect("responsible profile name");
    assert_eq!(encountered.kind(), CssTokenKind::Ident);
    assert_eq!(encountered.authored(), "--custom");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects custom color profiles");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn relative_color_environment_failure_reports_the_foreign_channel_and_recovers_once() {
    let source = "color: hwb(from red h s b); opacity: 0.5";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("foreign HWB channel must recover once");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidColorSyntax(detail) = diagnostic.error().kind() else {
        panic!("expected structured relative-color detail");
    };
    assert_eq!(
        detail.component().map(|component| component.as_str()),
        Some("relative channel")
    );
    let encountered = detail.encountered().expect("responsible foreign channel");
    assert_eq!(encountered.kind(), CssTokenKind::Ident);
    assert_eq!(encountered.authored(), "s");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects a foreign relative channel");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn color_mix_hue_method_failure_reports_the_responsible_token_and_retains_its_sibling() {
    let source = "color: color-mix(in srgb longer hue, red, blue); opacity: 0.5";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("rectangular-space hue method must recover once");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidColorSyntax(detail) = diagnostic.error().kind() else {
        panic!("expected structured color-mix detail");
    };
    assert_eq!(
        detail.component().map(|component| component.as_str()),
        Some("hue interpolation"),
    );
    let encountered = detail.encountered().expect("responsible hue method");
    assert_eq!(encountered.kind(), CssTokenKind::Ident);
    assert_eq!(encountered.authored(), "longer");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity,
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects rectangular-space hue methods");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn repeated_color_mix_failures_make_progress_and_preserve_later_siblings() {
    let source = concat!(
        "color: color-mix(in srgb longer hue, red, blue); ",
        "background-color: color-mix(in lab shorter hue, red, blue); ",
        "opacity: 0.5",
    );
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(report.diagnostics().len(), 2);
    assert!(report.diagnostics().iter().all(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::InvalidColorSyntax
            && diagnostic.action() == CssRecoveryAction::DropDeclaration
    }));
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity,
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects repeated invalid color-mix declarations");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn repeated_relative_color_failures_make_progress_and_preserve_later_siblings() {
    let source = concat!(
        "color: rgb(from red bogus g b); ",
        "background-color: color(from red xyz r y z); ",
        "opacity: 0.5",
    );
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(report.diagnostics().len(), 2);
    assert!(report.diagnostics().iter().all(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::InvalidColorSyntax
            && diagnostic.action() == CssRecoveryAction::DropDeclaration
    }));
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects repeated relative-color failures");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn transform_separator_and_domain_failures_report_exact_tokens_and_retain_siblings() {
    for (value, responsible, token_kind) in [
        ("matrix(1 0 0 1 10 20)", "0", CssTokenKind::Number),
        ("perspective(10%)", "10%", CssTokenKind::Percentage),
        ("translate3d(1px, 2px, 3%)", "3%", CssTokenKind::Percentage),
    ] {
        let source = format!("transform: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected one transform diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}",
        );
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropDeclaration,
            "{source}",
        );
        let responsible_offset = if value.starts_with("matrix") {
            source.find(" 0").expect("first missing-comma operand") + 1
        } else {
            source.rfind(responsible).expect("responsible token")
        };
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible_offset,
            "{source}",
        );
        assert_eq!(diagnostic.error().position().line().value(), 0, "{source}");
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            responsible_offset,
            "{source}",
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            0,
            "{source}"
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            source.find(';').expect("declaration semicolon") + 1,
            "{source}",
        );
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected property-value root");
        };
        assert_eq!(detail.property(), CssKnownProperty::Transform, "{source}");
        let encountered = detail.encountered().expect("responsible transform token");
        assert_eq!(encountered.kind(), token_kind, "{source}");
        assert_eq!(encountered.authored(), responsible, "{source}");
        assert_eq!(
            report.syntax()[0]
                .known()
                .expect("retained color declaration")
                .property(),
            CssKnownProperty::Color,
            "{source}",
        );

        #[cfg(feature = "app-strict")]
        {
            let failure = surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects recovered transform mutation");
            assert_eq!(failure.diagnostics(), report.diagnostics(), "{source}");
        }
    }
}

#[test]
fn repeated_transform_failures_make_progress_to_a_valid_sibling() {
    let source = concat!(
        "transform: matrix(1 0 0 1 0 0); ",
        "transform: perspective(-1px); ",
        "transform: translateZ(10%); color: red",
    );
    let report = parse_style_attribute(source);
    assert_eq!(report.diagnostics().len(), 3);
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0]
            .known()
            .expect("retained color declaration")
            .property(),
        CssKnownProperty::Color,
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects repeated recovered transforms");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn repeated_filter_failures_make_progress_to_valid_filter_and_color_siblings() {
    let source = concat!(
        "filter: drop-shadow(inset 1px 2px); ",
        "filter: brightness(-1); ",
        "filter: blur(); ",
        "filter: opacity(50%); color: red",
    );
    let report = parse_style_attribute(source);
    assert_eq!(report.diagnostics().len(), 3);
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Filter
    );
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color
    );
    for diagnostic in report.diagnostics() {
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("expected filter property-value error");
        };
        assert_eq!(detail.property(), CssKnownProperty::Filter);
    }

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered filter mutations");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn repeated_opacity_domain_failures_make_progress_to_a_valid_sibling() {
    let source = concat!(
        "opacity: 1e999; ",
        "opacity: 1px; ",
        "opacity: calc(1% + 2); ",
        "color: red",
    );
    let report = parse_style_attribute(source);
    assert_eq!(report.diagnostics().len(), 3);
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0]
            .known()
            .expect("retained color declaration")
            .property(),
        CssKnownProperty::Color,
    );
    for diagnostic in report.diagnostics() {
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("expected opacity property-value error");
        };
        assert_eq!(detail.property(), CssKnownProperty::Opacity);
        assert!(detail.encountered().is_some());
    }

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered opacity mutations");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn basic_shape_failures_report_clip_path_and_retain_valid_siblings() {
    for (value, responsible, token_kind) in [
        ("circle(-1px)", "-1px", CssTokenKind::Dimension),
        ("ellipse(1px)", "1px", CssTokenKind::Dimension),
        ("polygon(round 10%, 0 0)", "10%", CssTokenKind::Percentage),
        ("polygon(, 0 0, 100%)", "100%", CssTokenKind::Percentage),
    ] {
        let source = format!("clip-path: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected one diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected property-value detail");
        };
        assert_eq!(detail.property(), CssKnownProperty::ClipPath, "{source}");
        let encountered = detail.encountered().expect("responsible shape token");
        assert_eq!(encountered.kind(), token_kind, "{source}");
        assert_eq!(encountered.authored(), responsible, "{source}");

        #[cfg(feature = "app-strict")]
        {
            let failure = surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects invalid basic shape");
            assert_eq!(failure.diagnostics(), report.diagnostics(), "{source}");
        }
    }
}

#[test]
fn unknown_at_rules_remain_distinct_from_supported_conditional_rules() {
    let unknown = parse_sheet("@not-a-css-rule;").expect_err("unknown at-rule must fail");
    assert_eq!(unknown.code(), CssErrorCode::UnknownAtRule);
    match unknown.kind() {
        ErrorKind::UnknownAtRule(detail) => {
            assert_eq!(detail.name().as_str(), "not-a-css-rule");
        }
        _ => panic!("unexpected error root"),
    }

    let supports = parse_sheet("@supports (display: grid) {}");
    assert!(supports.is_clean(), "{:?}", supports.diagnostics());
    assert!(matches!(supports.syntax().rules(), [CssRule::Supports(_)]));
}

#[test]
fn error_at_rule_placement_exposes_expected_context() {
    let placement =
        parse_sheet(".x { width: 1px; } @import 'x.css';").expect_err("late import must fail");
    assert_eq!(placement.code(), CssErrorCode::InvalidAtRulePlacement);
    match placement.kind() {
        ErrorKind::InvalidAtRulePlacement(detail) => {
            assert_eq!(detail.name().as_str(), "import");
            assert_eq!(
                detail.expected_context().as_str(),
                "before every non-import top-level rule"
            );
        }
        _ => panic!("unexpected error root"),
    }
}

#[test]
fn error_malformed_at_rule_prelude_reports_responsible_token_position() {
    let error =
        parse_sheet("@font-face nope {}").expect_err("non-empty font-face prelude must fail");
    assert_eq!(error.code(), CssErrorCode::InvalidAtRulePrelude);
    match error.kind() {
        ErrorKind::InvalidAtRulePrelude(detail) => {
            assert_eq!(detail.name().as_str(), "font-face");
            assert_eq!(detail.production().as_str(), "baseline.rule.font-face");
            assert_eq!(detail.expectation().as_str(), "an empty @font-face prelude");
            assert_eq!(detail.encountered().unwrap().authored(), "nope");
        }
        _ => panic!("unexpected error root"),
    }
    let position = error.position();
    assert_eq!(position.byte_offset().value(), 11);
    assert_eq!(position.line().value(), 0);
    assert_eq!(position.column().value(), 11);
}

#[test]
fn namespace_prelude_errors_expose_exact_payload_span_action_and_sibling_recovery() {
    let failed = "@namespace svg ident;";
    let source = format!("{failed} .after {{}}");
    let report = parse_sheet(&source);
    assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one malformed namespace diagnostic")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePrelude
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find("ident").unwrap()
    );
    let failed_start = source.find(failed).unwrap();
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        failed_start
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        failed_start + failed.len()
    );
    let ErrorKind::InvalidAtRulePrelude(detail) = diagnostic.error().kind() else {
        panic!("expected namespace prelude payload")
    };
    assert_eq!(detail.name().as_str(), "namespace");
    assert_eq!(detail.production().as_str(), "later.rule.namespace");
    assert_eq!(
        detail.expectation().as_str(),
        "an optional prefix followed by one string or URL namespace name"
    );
    assert_eq!(detail.encountered().unwrap().authored(), "ident");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_sheet(&source)
            .expect_err("strict validation rejects malformed namespace rules");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn undeclared_selector_namespace_exposes_exact_payload_span_action_and_siblings() {
    let failed = "SVG|a { color: red; }";
    let source = format!("@namespace svg \"urn:svg\"; .before {{}} {failed} .after {{}}");
    let report = parse_sheet(&source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::Namespace(_), CssRule::Style(_), CssRule::Style(_)]
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("expected one undeclared-prefix diagnostic")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    let failed_start = source.find(failed).unwrap();
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        failed_start + "SVG|".len()
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        failed_start
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        failed_start + failed.len()
    );
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("expected selector payload")
    };
    assert_eq!(
        detail.production().expect("selector production").as_str(),
        "baseline.selector.complex"
    );
    assert_eq!(detail.expectation().as_str(), "a supported selector");
    assert_eq!(
        detail.encountered().expect("local-name token").authored(),
        "a"
    );
}

#[test]
fn error_missing_at_rule_body_reports_missing_token_position() {
    let error = parse_sheet("@media screen;").expect_err("media requires a block");
    assert_eq!(error.code(), CssErrorCode::InvalidAtRuleBody);
    match error.kind() {
        ErrorKind::InvalidAtRuleBody(detail) => {
            assert_eq!(detail.name().as_str(), "media");
            assert_eq!(detail.production().as_str(), "baseline.rule.media");
            assert!(detail.encountered().is_none());
        }
        _ => panic!("unexpected error root"),
    }
    let position = error.position();
    assert_eq!(position.byte_offset().value(), 14);
    assert_eq!(position.line().value(), 0);
    assert_eq!(position.column().value(), 14);
}

#[test]
fn error_empty_font_face_body_reports_missing_descriptor_at_body_end() {
    let error = parse_sheet("@font-face {\n}")
        .expect_err("font-face requires font-family and src descriptors");
    assert_eq!(error.code(), CssErrorCode::InvalidAtRuleBody);
    match error.kind() {
        ErrorKind::InvalidAtRuleBody(detail) => {
            assert_eq!(detail.name().as_str(), "font-face");
            assert_eq!(detail.production().as_str(), "baseline.rule.font-face");
            assert_eq!(
                detail.expectation().as_str(),
                "font-family and src descriptors"
            );
            assert!(detail.encountered().is_none());
        }
        _ => panic!("unexpected error root"),
    }
    let position = error.position();
    assert_eq!(position.byte_offset().value(), 13);
    assert_eq!(position.line().value(), 1);
    assert_eq!(position.column().value(), 0);
}

#[test]
fn error_multi_name_layer_block_reports_offending_block_token() {
    let error = parse_sheet("@layer 😀, theme {}")
        .expect_err("a layer block accepts at most one layer name");
    assert_eq!(error.code(), CssErrorCode::InvalidAtRuleBody);
    match error.kind() {
        ErrorKind::InvalidAtRuleBody(detail) => {
            assert_eq!(detail.name().as_str(), "layer");
            assert_eq!(detail.production().as_str(), "baseline.rule.layer-block");
            assert_eq!(
                detail.expectation().as_str(),
                "at most one layer name before a block"
            );
            let token = detail.encountered().expect("offending block token");
            assert_eq!(token.kind(), CssTokenKind::CurlyBracketBlock);
            assert_eq!(token.authored(), "{");
        }
        _ => panic!("unexpected error root"),
    }
    let position = error.position();
    assert_eq!(position.byte_offset().value(), 19);
    assert_eq!(position.line().value(), 0);
    assert_eq!(position.column().value(), 17);
}

#[test]
fn error_selector_media_and_qualified_rule_failures_keep_production_context() {
    let selector = parse_sheet("??? { width: 1px; }").expect_err("selector must fail");
    match selector.kind() {
        ErrorKind::InvalidSelector(detail) => {
            assert_eq!(
                detail.production().unwrap().as_str(),
                "baseline.selector.complex"
            );
            assert_eq!(detail.expectation().as_str(), "a supported selector");
            assert_eq!(detail.encountered().unwrap().authored(), "?");
        }
        _ => panic!("unexpected error root"),
    }

    let media = parse_sheet("@media (scripting: enabled) { .x { width: 1px; } }")
        .expect_err("recognized deferred scripting must retain its frozen failure");
    assert_eq!(media.code(), CssErrorCode::InvalidMediaQuery);
    match media.kind() {
        ErrorKind::InvalidMediaQuery(detail) => {
            assert_eq!(detail.feature().unwrap().as_str(), "scripting");
            assert_eq!(detail.expectation().as_str(), "a supported media query");
            assert_eq!(detail.encountered().unwrap().authored(), "scripting");
        }
        _ => panic!("unexpected error root"),
    }

    let qualified = parse_sheet("x;").expect_err("qualified rule without a block must fail");
    assert_eq!(qualified.code(), CssErrorCode::InvalidQualifiedRule);
    match qualified.kind() {
        ErrorKind::InvalidQualifiedRule(detail) => {
            assert_eq!(detail.production().as_str(), "css.qualified-rule");
            assert_eq!(detail.expectation().as_str(), "valid CSS syntax");
            assert!(detail.encountered().is_none());
        }
        _ => panic!("unexpected error root"),
    }
}

#[test]
fn error_property_descriptor_color_and_annotation_roots_expose_all_fields() {
    let property = parse_sheet(".x { WIDHT: 1px; }").expect_err("unknown property must fail");
    assert_eq!(property.code(), CssErrorCode::UnknownProperty);
    match property.kind() {
        ErrorKind::UnknownProperty(detail) => assert_eq!(detail.name().as_str(), "WIDHT"),
        _ => panic!("unexpected error root"),
    }

    let descriptor =
        parse_sheet("@font-face { mystery: x; font-family: Test; src: url(test.woff2); }")
            .expect_err("unknown descriptor must fail");
    assert_eq!(descriptor.code(), CssErrorCode::UnknownDescriptor);
    match descriptor.kind() {
        ErrorKind::UnknownDescriptor(detail) => {
            assert_eq!(detail.at_rule().as_str(), "font-face");
            assert_eq!(detail.descriptor().as_str(), "mystery");
        }
        _ => panic!("unexpected error root"),
    }

    let descriptor_value = parse_sheet(
        "@font-face { font-family: Test; src: url(test.woff2); font-display: mystery; }",
    )
    .expect_err("invalid descriptor value must fail");
    assert_eq!(
        descriptor_value.code(),
        CssErrorCode::InvalidDescriptorValue
    );
    match descriptor_value.kind() {
        ErrorKind::InvalidDescriptorValue(detail) => {
            assert_eq!(detail.at_rule().as_str(), "font-face");
            assert_eq!(detail.descriptor().as_str(), "font-display");
            assert_eq!(
                detail.expectation().as_str(),
                "a value accepted by the descriptor grammar"
            );
            assert_eq!(detail.encountered().unwrap().authored(), "mystery");
        }
        _ => panic!("unexpected error root"),
    }

    let color = parse_sheet(".x { color: #ggg; }").expect_err("invalid color must fail");
    assert_eq!(color.code(), CssErrorCode::InvalidColorSyntax);
    match color.kind() {
        ErrorKind::InvalidColorSyntax(detail) => {
            assert!(detail.component().is_none());
            assert_eq!(detail.expectation().as_str(), "valid color syntax");
            assert_eq!(detail.encountered().unwrap().authored(), "#ggg");
        }
        _ => panic!("unexpected error root"),
    }

    let annotation = parse_sheet(".x { width: 1px !oops; }").expect_err("bad annotation must fail");
    assert_eq!(
        annotation.code(),
        CssErrorCode::InvalidDeclarationAnnotation
    );
    match annotation.kind() {
        ErrorKind::InvalidDeclarationAnnotation(detail) => {
            match detail.context() {
                CssDeclarationContextRef::KnownProperty(property) => {
                    assert_eq!(property, CssKnownProperty::Width);
                }
                _ => panic!("unexpected declaration context"),
            }
            assert_eq!(detail.encountered().authored(), "!");
        }
        _ => panic!("unexpected error root"),
    }
}

#[test]
fn error_invalid_font_source_format_reports_descriptor_context_and_recovers_sibling() {
    let source = concat!(
        "@font-face{font-family:Demo;src:url(face) format(woff3)}",
        ".after{color:red}",
    );
    let report = parse_sheet(source);
    assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    assert_eq!(report.diagnostics().len(), 2);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    let ErrorKind::InvalidDescriptorValue(detail) = diagnostic.error().kind() else {
        panic!("expected typed descriptor error");
    };
    assert_eq!(detail.at_rule().as_str(), "font-face");
    assert_eq!(detail.descriptor().as_str(), "src");
    assert_eq!(detail.encountered().unwrap().authored(), "woff3");
    assert_eq!(
        report.diagnostics()[1].error().code(),
        CssErrorCode::InvalidAtRuleBody
    );
    assert_eq!(
        report.diagnostics()[1].action(),
        CssRecoveryAction::DropAtRule
    );
}

#[test]
fn error_invalid_property_value_retains_canonical_property_and_authored_token() {
    let error = parse_sheet(".panel { WIDTH: n\\6f pe; }")
        .expect_err("invalid known-property value must fail");

    assert_eq!(error.code(), CssErrorCode::InvalidPropertyValue);
    match error.kind() {
        ErrorKind::InvalidPropertyValue(detail) => {
            assert_eq!(detail.property(), CssKnownProperty::Width);
            assert_eq!(
                detail.expectation().as_str(),
                "a value accepted by the property's grammar"
            );
            let token = detail.encountered().expect("non-EOF token");
            assert_eq!(token.kind(), CssTokenKind::Ident);
            assert_eq!(token.authored(), "n\\6f pe");
        }
        _ => panic!("unexpected error root"),
    }
}

#[test]
fn error_property_value_at_bounded_end_uses_absent_encountered_token() {
    let error = parse_sheet(".panel { width:").expect_err("missing value must fail");

    assert_eq!(error.code(), CssErrorCode::InvalidPropertyValue);
    match error.kind() {
        ErrorKind::InvalidPropertyValue(detail) => {
            assert_eq!(detail.property(), CssKnownProperty::Width);
            assert!(detail.encountered().is_none());
        }
        _ => panic!("unexpected error root"),
    }
}

#[test]
fn error_malformed_declaration_exposes_unexpected_authored_token() {
    let error =
        parse_sheet(".panel { width 1px; }").expect_err("missing declaration colon must fail");

    assert_eq!(error.code(), CssErrorCode::UnexpectedToken);
    match error.kind() {
        ErrorKind::UnexpectedToken(detail) => {
            assert_eq!(detail.expectation().as_str(), "valid CSS syntax");
            assert_eq!(detail.encountered().kind(), CssTokenKind::Dimension);
            assert_eq!(detail.encountered().authored(), "1px");
        }
        _ => panic!("unexpected error root"),
    }
}

#[test]
fn error_position_uses_byte_line_and_utf16_coordinates_and_display_is_one_based() {
    let source = ".ok { width: 1px; }\n.emoji-😀 { width: nope; }";
    let error = parse_sheet(source).expect_err("invalid value must fail");
    let position = error.position();

    assert_eq!(position.byte_offset().value(), source.find("nope").unwrap());
    assert_eq!(position.line().value(), 1);
    assert_eq!(position.column().value(), 19);
    assert!(error.to_string().contains("2:20"));
}

#[test]
fn error_public_non_exhaustive_kinds_are_matched_with_wildcards() {
    let error = parse_sheet("??? { width: 1px; }").expect_err("invalid selector must fail");

    let expectation = match error.kind() {
        ErrorKind::InvalidSelector(detail) => detail.expectation().as_str(),
        _ => "different extensible root",
    };
    assert_eq!(expectation, "a supported selector");
}

#[test]
fn typed_calculation_type_error_has_exact_non_bmp_coordinates_span_and_recovery() {
    let source = "--😀: 1; opacity: calc(1px + 2px); color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("the invalid typed calculation must recover exactly once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 25);
    assert_eq!(diagnostic.error().position().line().value(), 0);
    assert_eq!(diagnostic.error().position().column().value(), 23);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 11);
    assert_eq!(diagnostic.span().start().column().value(), 9);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 36);
    assert_eq!(diagnostic.span().end().column().value(), 34);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected structured property-value error");
    };
    assert_eq!(detail.property(), CssKnownProperty::Opacity);
    let encountered = detail.encountered().expect("responsible typed leaf");
    assert_eq!(encountered.kind(), CssTokenKind::Dimension);
    assert_eq!(encountered.authored(), "1px");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation must reject recovered typed calculation input");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn typed_calculation_operator_and_divisor_errors_retain_later_siblings() {
    for (source, property, authored, kind) in [
        (
            "width: calc(1px * 2px); color: red",
            CssKnownProperty::Width,
            "*",
            CssTokenKind::Delim,
        ),
        (
            "order: calc(1 / 0); color: red",
            CssKnownProperty::Order,
            "/",
            CssTokenKind::Delim,
        ),
        (
            "width: calc(1px / 1px); color: red",
            CssKnownProperty::Width,
            "/",
            CssTokenKind::Delim,
        ),
    ] {
        let report = parse_style_attribute(source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected one diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected property-value detail");
        };
        assert_eq!(detail.property(), property, "{source}");
        let encountered = detail.encountered().expect("responsible operator");
        assert_eq!(encountered.authored(), authored, "{source}");
        assert_eq!(encountered.kind(), kind, "{source}");
    }
}

#[test]
fn easing_bound_error_has_exact_payload_span_action_and_sibling_recovery() {
    let source = "transition-timing-function: cubic-bezier(1.1, 0, 0.5, 1); color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("out-of-range cubic-bezier x must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("1.1").expect("responsible x coordinate");
    let declaration_end = source.find(';').expect("timing declaration end") + 1;
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        declaration_end
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected structured property-value error");
    };
    assert_eq!(
        detail.property(),
        CssKnownProperty::TransitionTimingFunction
    );
    let encountered = detail
        .encountered()
        .expect("responsible cubic-bezier x coordinate");
    assert_eq!(encountered.kind(), CssTokenKind::Number);
    assert_eq!(encountered.authored(), "1.1");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered easing input");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn timing_first_duration_failure_has_exact_payload_span_action_and_sibling_recovery() {
    let source = "transition: opacity -1s 2s; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("negative first shorthand time must recover once");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 20);
    assert_eq!(diagnostic.error().position().line().value(), 0);
    assert_eq!(diagnostic.error().position().column().value(), 20);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().start().column().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 27);
    assert_eq!(diagnostic.span().end().column().value(), 27);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("expected structured property-value error");
    };
    assert_eq!(detail.property(), CssKnownProperty::Transition);
    let encountered = detail.encountered().expect("responsible negative duration");
    assert_eq!(encountered.kind(), CssTokenKind::Dimension);
    assert_eq!(encountered.authored(), "-1s");

    #[cfg(feature = "app-strict")]
    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered transition input");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn generic_position_mutations_report_the_responsible_token_and_retain_the_sibling() {
    for (value, responsible, token_kind) in [
        ("left right", "right", CssTokenKind::Ident),
        ("50% left", "left", CssTokenKind::Ident),
        ("left top 10px", "10px", CssTokenKind::Dimension),
        ("left 10px top", "top", CssTokenKind::Ident),
        ("center 10px top 20px", "top", CssTokenKind::Ident),
        ("left 10px right 20px", "right", CssTokenKind::Ident),
        ("top 10px", "10px", CssTokenKind::Dimension),
    ] {
        let source = format!("mask-position: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected one generic-position diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}",
        );
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropDeclaration,
            "{source}",
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find(responsible).expect("responsible token"),
            "{source}",
        );
        assert_eq!(diagnostic.error().position().line().value(), 0, "{source}");
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            source.find(responsible).expect("responsible column"),
            "{source}",
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            0,
            "{source}"
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            source.find(';').expect("declaration semicolon") + 1,
            "{source}",
        );
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected property-value root");
        };
        assert_eq!(
            detail.property(),
            CssKnownProperty::MaskPosition,
            "{source}"
        );
        let encountered = detail.encountered().expect("responsible position token");
        assert_eq!(encountered.kind(), token_kind, "{source}");
        assert_eq!(encountered.authored(), responsible, "{source}");
        assert_eq!(
            report.syntax()[0]
                .known()
                .expect("retained color declaration")
                .property(),
            CssKnownProperty::Color,
            "{source}",
        );

        #[cfg(feature = "app-strict")]
        {
            let failure = surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects the recovered mutation");
            assert_eq!(failure.diagnostics(), report.diagnostics(), "{source}");
        }
    }
}

#[test]
fn layered_position_mutations_report_exact_property_token_span_and_recovery() {
    for (property, known_property, value, responsible, token_kind) in [
        (
            "background-position",
            CssKnownProperty::BackgroundPosition,
            "left top 10px 20px",
            "20px",
            CssTokenKind::Dimension,
        ),
        (
            "background-position",
            CssKnownProperty::BackgroundPosition,
            "left, right left, top",
            "left",
            CssTokenKind::Ident,
        ),
        (
            "mask-position",
            CssKnownProperty::MaskPosition,
            "left 10px top",
            "top",
            CssTokenKind::Ident,
        ),
        (
            "mask-position",
            CssKnownProperty::MaskPosition,
            "center, top 10px",
            "10px",
            CssTokenKind::Dimension,
        ),
    ] {
        let source = format!("{property}: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected one diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let responsible_offset = if value.contains(", right left") {
            source.find("right left").expect("invalid layer") + "right ".len()
        } else {
            source.rfind(responsible).expect("responsible token")
        };
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible_offset,
            "{source}",
        );
        assert_eq!(diagnostic.error().position().line().value(), 0, "{source}");
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            responsible_offset,
            "{source}",
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            0,
            "{source}"
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            source.find(';').expect("declaration semicolon") + 1,
            "{source}",
        );
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected property-value root");
        };
        assert_eq!(detail.property(), known_property, "{source}");
        let encountered = detail.encountered().expect("responsible position token");
        assert_eq!(encountered.kind(), token_kind, "{source}");
        assert_eq!(encountered.authored(), responsible, "{source}");

        #[cfg(feature = "app-strict")]
        {
            let failure = surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects layered position mutation");
            assert_eq!(failure.diagnostics(), report.diagnostics(), "{source}");
        }
    }
}

#[test]
fn property_specific_origin_mutations_report_exact_payload_span_and_recovery() {
    for (property, known_property, value, responsible, token_kind) in [
        (
            "object-position",
            CssKnownProperty::ObjectPosition,
            "left top 10px",
            "10px",
            CssTokenKind::Dimension,
        ),
        (
            "transform-origin",
            CssKnownProperty::TransformOrigin,
            "left top 50%",
            "50%",
            CssTokenKind::Percentage,
        ),
        (
            "transform-origin",
            CssKnownProperty::TransformOrigin,
            "left top calc(10%)",
            "calc(",
            CssTokenKind::Function,
        ),
        (
            "transform-origin",
            CssKnownProperty::TransformOrigin,
            "left top calc(1px + 10%)",
            "calc(",
            CssTokenKind::Function,
        ),
        (
            "transform-origin",
            CssKnownProperty::TransformOrigin,
            "top 10px 20px",
            "20px",
            CssTokenKind::Dimension,
        ),
        (
            "transform-origin",
            CssKnownProperty::TransformOrigin,
            "top calc(10%)",
            "calc(",
            CssTokenKind::Function,
        ),
        (
            "transform-origin",
            CssKnownProperty::TransformOrigin,
            "top calc(1px + 10%)",
            "calc(",
            CssTokenKind::Function,
        ),
        (
            "transform-origin",
            CssKnownProperty::TransformOrigin,
            "left top 10px 20px",
            "20px",
            CssTokenKind::Dimension,
        ),
        (
            "transform-origin",
            CssKnownProperty::TransformOrigin,
            "left top bottom",
            "bottom",
            CssTokenKind::Ident,
        ),
    ] {
        let source = format!("{property}: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected one origin diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let responsible_offset = source.rfind(responsible).expect("responsible token");
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible_offset,
            "{source}",
        );
        assert_eq!(diagnostic.error().position().line().value(), 0, "{source}");
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            responsible_offset,
            "{source}",
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            0,
            "{source}"
        );
        assert_eq!(diagnostic.span().start().line().value(), 0, "{source}");
        assert_eq!(diagnostic.span().start().column().value(), 0, "{source}");
        let declaration_end = source.find(';').expect("declaration semicolon") + 1;
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            declaration_end,
            "{source}",
        );
        assert_eq!(diagnostic.span().end().line().value(), 0, "{source}");
        assert_eq!(
            diagnostic.span().end().column().value() as usize,
            declaration_end,
            "{source}",
        );
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected property-value root");
        };
        assert_eq!(detail.property(), known_property, "{source}");
        let encountered = detail.encountered().expect("responsible origin token");
        assert_eq!(encountered.kind(), token_kind, "{source}");
        assert_eq!(encountered.authored(), responsible, "{source}");
        assert_eq!(
            report.syntax()[0]
                .known()
                .expect("retained color declaration")
                .property(),
            CssKnownProperty::Color,
            "{source}",
        );

        #[cfg(feature = "app-strict")]
        {
            let failure = surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects the recovered origin mutation");
            assert_eq!(failure.diagnostics(), report.diagnostics(), "{source}");
        }
    }
}
