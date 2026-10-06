#![forbid(unsafe_code)]
//! Authored Filter1 SVG property contracts through existing public fronts.
//! Source: Filter Effects 1 WD20181218 §§3,9.13.1–2,10,11.5;
//! selected Color4 and complete shared Color owner, Values4, Cascade5,
//! Variables1, Syntax3 and CSSOM WD20210826 §6.7.2.
use surgeist_css::*;

const FLOOD: &str = "flood-color";
const ALPHA: &str = "flood-opacity";
const LIGHT: &str = "lighting-color";
const INTERPOLATION: &str = "color-interpolation-filters";
const NAMES: [&str; 4] = [FLOOD, ALPHA, LIGHT, INTERPOLATION];
const COLOR_GOLDENS: &[(&str, &str)] = &[
    ("red", "red"),
    ("currentColor", "currentcolor"),
    ("transparent", "transparent"),
    ("Canvas", "canvas"),
    ("ActiveBorder", "activeborder"),
    ("#abc", "rgb(170, 187, 204)"),
    ("#abcdef", "rgb(171, 205, 239)"),
    ("#00000080", "rgba(0, 0, 0, 0.5)"),
    ("rgb(1 2 3 / .5)", "rgba(1, 2, 3, 0.5)"),
    ("rgba(1,2,3,.5)", "rgba(1, 2, 3, 0.5)"),
    ("hsl(0 100% 50%)", "rgb(255, 0, 0)"),
    ("hwb(0 0% 0%)", "rgb(255, 0, 0)"),
    ("lab(50% 20 -30 / .5)", "lab(50 20 -30 / 0.5)"),
    ("lch(50% 20 30deg)", "lch(50 20 30)"),
    ("oklab(.5 .1 -.1)", "oklab(0.5 0.1 -0.1)"),
    ("oklch(.5 .1 30)", "oklch(0.5 0.1 30)"),
    ("rgb(none 0 255)", "color(srgb none 0 1)"),
    ("lab(none 0 0)", "lab(none 0 0)"),
    ("color(srgb 100% 50% 0%)", "color(srgb 1 0.5 0)"),
    ("color(display-p3 1 0 0)", "color(display-p3 1 0 0)"),
    (
        "color(display-p3-linear 1 0 0)",
        "color(display-p3-linear 1 0 0)",
    ),
    ("color(--P 0% 70% 20% 0%)", "color(--P 0 0.7 0.2 0)"),
    ("rgb(from red r g b / alpha)", "rgb(from red r g b / alpha)"),
    (
        "color(from red --P Cyan / alpha)",
        "color(from red --P Cyan / alpha)",
    ),
    ("color-mix(in oklab, red, blue)", "color-mix(red, blue)"),
    ("light-dark(red,blue)", "light-dark(red, blue)"),
    (
        "contrast-color(hsl(0 100% 50%))",
        "contrast-color(rgb(255, 0, 0))",
    ),
    ("alpha(from red / 50%)", "alpha(from red / 0.5)"),
    (
        "device-cmyk(0% 81% 81% 30%)",
        "device-cmyk(0 0.81 0.81 0.3)",
    ),
];
const OPACITY_GOLDENS: &[(&str, &str)] = &[
    (".5", "0.5"),
    ("-2.5", "-2.5"),
    ("150%", "1.5"),
    ("-25%", "-0.25"),
    ("+000.1000e1", "1"),
    ("-0%", "0"),
    ("1e-47", "0"),
    ("1e-47%", "0"),
    ("calc(1 / 2)", "calc(0.5)"),
    ("calc(25% + 25%)", "calc(50%)"),
    ("calc(2 - 3)", "calc(-1)"),
    ("max(-1,2)", "calc(2)"),
    ("min(25%,50%)", "calc(25%)"),
    ("clamp(2,1,0)", "calc(2)"),
    ("calc(1in / 96px)", "calc(1)"),
    // Shared type-matching permits an unresolved Number result with a Length
    // percent hint; no external percentage basis is supplied or guessed.
    ("calc((1px + 1%) / 1px)", "calc((1% + 1px) / 1px)"),
];
fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name)
        .unwrap_or_else(|| panic!("missing authored Filter1 property: {name}"))
}
fn checked_components(
    name: &str,
    values: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
    let p = property(name);
    if grammar {
        parse_property_value_for_grammar(p.grammar(), values, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(p),
            values,
            CssImportance::Important,
        )
    }
}
fn checked(name: &str, text: &str, grammar: bool) -> CssDeclaration {
    let values = parse_component_values(text).unwrap();
    let before = values.clone();
    let source = checked_components(name, values.clone(), grammar).unwrap();
    assert_eq!(source.value_components(), &values);
    assert_eq!(values, before);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert!(source.parsed_value().is_none());
    source
}
fn fronts(name: &str, text: &str) -> Vec<CssDeclaration> {
    let css = format!("/*😀*/{}:{text}!important", name.to_ascii_uppercase());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert_eq!(&validate_style_attribute(&css).unwrap(), report.syntax());
    let [parsed] = report.syntax().as_slice() else {
        panic!("one complete authored occurrence")
    };
    let p = property(name);
    assert_eq!(parsed.known().unwrap().property(), p);
    assert_eq!(
        parsed.position().unwrap().byte_offset().value(),
        "/*😀*/".len()
    );
    assert_eq!(
        parsed.position().unwrap().column().value() as usize,
        "/*😀*/".encode_utf16().count()
    );
    assert_eq!(parsed.parsed_value().unwrap().source().as_str(), css);
    let mut sources = vec![
        parsed.clone(),
        checked(name, text, false),
        checked(name, text, true),
    ];
    for report in [
        parse_property_value_text(text, CssPropertyNameRef::Known(p), CssImportance::Important),
        parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Important),
    ] {
        assert!(
            report.is_clean(),
            "{name}:{text}: {:?}",
            report.diagnostics()
        );
        let source = report.syntax().as_ref().unwrap();
        assert_eq!(source.parsed_value().unwrap().source().as_str(), text);
        assert!(source.position().is_none());
        assert!(source.parsed_name().is_none());
        sources.push(source.clone());
    }
    for source in &sources {
        assert_eq!(source.importance(), CssImportance::Important);
        assert_eq!(source.known().unwrap().grammar(), p.grammar());
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            text
        );
    }
    sources
}
fn canonical(name: &str, text: &str, expected: &str) {
    for source in fronts(name, text) {
        let before = source.clone();
        let expected = format!("{name}: {expected} !important;");
        let exact = CssSpecifiedValueSerializationLimits::new(65_536, 262_144, expected.len());
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        for _ in 0..2 {
            let short =
                CssSpecifiedValueSerializationLimits::new(65_536, 262_144, expected.len() - 1);
            assert_eq!(
                source
                    .to_specified_css_with_limits(short)
                    .unwrap_err()
                    .kind(),
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            );
            assert_eq!(source, before);
        }
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        // Canonical text must be accepted again through the actual strict owner.
        let css = source.to_specified_css().unwrap();
        let report = parse_style_attribute(&css);
        assert!(report.is_clean());
        assert_eq!(report.syntax()[0].to_specified_css().unwrap(), expected);
    }
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one authored terminal")
    };
    let [item] = values.items() else {
        panic!("one longhand contribution")
    };
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().value_components(), source.value_components());
    values
}
fn invalid(name: &str, text: &str) {
    let p = property(name);
    let css = format!("color:red;{name}:{text};color:blue");
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "atomic grammar rejection: {css}");
    assert!(
        report
            .syntax()
            .iter()
            .all(|d| d.known().unwrap().property() == CssKnownProperty::Color)
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one atomic diagnostic: {css}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    match diagnostic.error().kind() {
        ErrorKind::InvalidPropertyValue(v) => assert_eq!(v.property(), p),
        ErrorKind::InvalidColorSyntax(_) if name == FLOOD || name == LIGHT => {}
        other => panic!("property-owned grammar diagnostic: {other:?}"),
    }
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let values = parse_component_values(text).unwrap();
    let before = values.clone();
    for grammar in [false, true] {
        assert!(matches!(
            checked_components(name, values.clone(), grammar)
                .unwrap_err()
                .kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
    }
    for report in [
        parse_property_value_text(text, CssPropertyNameRef::Known(p), CssImportance::Normal),
        parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Normal),
    ] {
        assert!(report.syntax().is_none());
        assert!(!report.is_clean());
    }
    assert_eq!(values, before);
}
#[test]
fn four_filter_longhands_are_recognized_case_insensitively_with_intrinsic_metadata() {
    for (name, inherited, initial, canonical_initial) in [
        (FLOOD, false, "black", "black"),
        (ALPHA, false, "1", "1"),
        (LIGHT, false, "white", "white"),
        (INTERPOLATION, true, "linearRGB", "linearrgb"),
    ] {
        let p = property(name);
        assert_eq!(
            CssKnownProperty::from_name(&name.to_ascii_uppercase()),
            Some(p)
        );
        assert_eq!(p.canonical_name(), name);
        let CssPropertyKindRef::Longhand(metadata) = p.metadata().unwrap().kind() else {
            panic!("terminal metadata")
        };
        assert_eq!(metadata.property().known_property(), p);
        assert_eq!(metadata.inherited_by_default(), inherited);
        let initial_value = metadata.initial_value();
        assert_eq!(initial_value.property().known_property(), p);
        let CssInitialValueRef::Value(value) = initial_value.view() else {
            panic!("intrinsic value")
        };
        // Existing generic API bridges independently sourced initial spelling;
        // new payload variants need direct typed functional assertions at GREEN.
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_token(initial).unwrap()])
                .unwrap();
        let control = checked_components(name, components, true).unwrap();
        canonical(name, initial, canonical_initial);
        let values = completed(&control);
        assert_eq!(value, values.items()[0].ordinary_value().unwrap());
    }
}
#[test]
fn flood_color_accepts_the_complete_existing_color_owner() {
    for &(input, expected) in COLOR_GOLDENS {
        canonical(FLOOD, input, expected);
    }
}
#[test]
fn lighting_color_accepts_the_complete_existing_color_owner() {
    for &(input, expected) in COLOR_GOLDENS {
        canonical(LIGHT, input, expected);
    }
}
#[test]
fn flood_opacity_retains_authored_ranges_and_uses_exact_numeric_projection() {
    for &(input, expected) in OPACITY_GOLDENS {
        canonical(ALPHA, input, expected);
    }
}
#[test]
fn interpolation_keywords_remain_distinct_and_emit_cssom_lowercase() {
    for (input, expected) in [
        ("AUTO", "auto"),
        ("sRGB", "srgb"),
        ("SrGb", "srgb"),
        ("linearRGB", "linearrgb"),
        ("LINEARRGB", "linearrgb"),
        ("s\\52 GB", "srgb"),
    ] {
        canonical(INTERPOLATION, input, expected);
    }
}
#[test]
fn color_grammar_rejects_noncolors_duplicates_and_malformed_color_children_atomically() {
    for name in [FLOOD, LIGHT] {
        for text in [
            "auto",
            "none",
            "red blue",
            "1px",
            "rgb(1 2)",
            "rgb(1 2 3 / none none)",
            "inherit red",
            "url(x)",
        ] {
            invalid(name, text);
        }
    }
}
#[test]
fn flood_opacity_rejects_non_alpha_values_and_incompatible_calculation_dimensions() {
    for text in [
        "none",
        "auto",
        "1px",
        "1 2",
        "infinity",
        "NaN",
        "calc(1px)",
        "calc(1 + 25%)",
        "calc(1px + 1s)",
        "inherit .5",
    ] {
        invalid(ALPHA, text);
    }
}
#[test]
fn interpolation_rejects_other_color_spaces_and_multi_keyword_values_atomically() {
    for text in [
        "normal",
        "srgb-linear",
        "linear-rgb",
        "display-p3",
        "auto srgb",
        "srgb linearrgb",
        "1",
        "rgb(0 0 0)",
    ] {
        invalid(INTERPOLATION, text);
    }
}
#[test]
fn large_and_tiny_flood_scalars_retain_original_tokens_and_parsed_origins() {
    for text in [
        "1e100", "-1e100", "1e-47", "-1e-47", "1e100%", "-1e100%", "1e-47%", "-1e-47%",
    ] {
        for source in fronts(ALPHA, text) {
            let [component] = source.value_components().items() else {
                panic!("one numeric token")
            };
            let number = match component.view() {
                CssComponentValueRef::Token(
                    CssValueTokenRef::Number(v) | CssValueTokenRef::Percentage(v),
                ) => v,
                _ => panic!("numeric kind preserved"),
            };
            assert_eq!(number.representation(), text.trim_end_matches('%'));
            if let CssValueOrigin::Parsed(origin) = component.origin() {
                assert!(
                    origin
                        .source()
                        .same_snapshot(source.parsed_value().unwrap().source())
                );
            }
            let values = completed(&source);
            assert!(matches!(
                values.items()[0].value(),
                CssContributionValueRef::Ordinary(_)
            ));
        }
    }
    canonical(ALPHA, "1e100%", &format!("1{}", "0".repeat(98)));
    canonical(ALPHA, "-1e100", &format!("-1{}", "0".repeat(100)));
}
#[test]
fn four_terminal_expansions_preserve_original_occurrence_and_both_importance_states() {
    for (name, text) in [
        (FLOOD, "red"),
        (ALPHA, "150%"),
        (LIGHT, "currentcolor"),
        (INTERPOLATION, "auto"),
    ] {
        for importance in [CssImportance::Normal, CssImportance::Important] {
            let source = parse_property_value(
                CssPropertyNameRef::Known(property(name)),
                parse_component_values(text).unwrap(),
                importance,
            )
            .unwrap();
            let before = source.clone();
            let values = completed(&source);
            assert!(matches!(
                values.items()[0].value(),
                CssContributionValueRef::Ordinary(_)
            ));
            assert!(values.items()[0].replacement_components().is_none());
            assert_eq!(source, before);
        }
    }
}
#[test]
fn css_wide_keywords_and_all_membership_cover_each_filter_terminal() {
    let CssPropertyKindRef::UniversalReset(all) = CssKnownProperty::All.metadata().unwrap().kind()
    else {
        panic!("all metadata")
    };
    for name in NAMES {
        let p = property(name);
        assert!(!all.excludes(CssPropertyNameRef::Known(p)));
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            for source in fronts(name, text) {
                let values = completed(&source);
                assert_eq!(
                    values.items()[0].value(),
                    CssContributionValueRef::Global(keyword)
                );
                assert!(values.items()[0].ordinary_value().is_none());
                assert!(values.items()[0].replacement_components().is_none());
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {text} !important;")
                );
            }
        }
    }
}
#[test]
fn whole_pending_substitutions_reenter_strictly_reusably_and_preserve_replacement_provenance() {
    for (name, replacement) in [
        (FLOOD, "rgb(1 2 3 / .5)"),
        (ALPHA, "-25%"),
        (LIGHT, "currentcolor"),
        (INTERPOLATION, "sRGB"),
    ] {
        for text in ["var(--filter)", "env(filter)", "attr(data-filter *)"] {
            for source in fronts(name, text) {
                let before = source.clone();
                let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                    panic!("pending whole value")
                };
                assert!(handle.source().same_occurrence(&source));
                for residual in ["var(--again)", "env(again)", "attr(data-again *)"] {
                    assert_eq!(
                        handle
                            .reenter(parse_component_values(residual).unwrap())
                            .unwrap_err()
                            .kind(),
                        &CssExpansionErrorKind::ResidualSubstitution
                    );
                }
                for rejected in [
                    "bogus",
                    "",
                    "initial inherit",
                    "initial!important",
                    "initial;color:red",
                ] {
                    for _ in 0..2 {
                        assert!(matches!(
                            handle
                                .reenter(parse_component_values(rejected).unwrap())
                                .unwrap_err()
                                .kind(),
                            CssExpansionErrorKind::InvalidReplacement(_)
                        ));
                    }
                }
                for valid in [replacement, "inherit"] {
                    let components = parse_component_values(valid).unwrap();
                    let snapshot = components.clone();
                    let CssContributions::Longhands(values) =
                        handle.reenter(components.clone()).unwrap()
                    else {
                        panic!("complete replacement")
                    };
                    let [item] = values.items() else {
                        panic!("one terminal")
                    };
                    assert_eq!(item.property(), property(name));
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert_eq!(item.replacement_components(), Some(&components));
                    if valid == "inherit" {
                        assert_eq!(
                            item.value(),
                            CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                        );
                    } else {
                        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
                    }
                    assert_eq!(components, snapshot);
                }
                assert_eq!(source, before);
            }
        }
    }
}
#[test]
fn strict_checked_admission_rejects_original_implicit_closures_before_serialized_recovery() {
    for (name, ordinary) in [
        (FLOOD, "red"),
        (ALPHA, ".5"),
        (LIGHT, "white"),
        (INTERPOLATION, "auto"),
    ] {
        for text in [
            format!("{ordinary}/*"),
            "initial/*".into(),
            "var(--filter".into(),
            "env(filter".into(),
            "attr(data-filter *".into(),
        ] {
            let components = parse_component_values(&text).unwrap();
            let before = components.clone();
            for grammar in [false, true] {
                let error = checked_components(name, components.clone(), grammar).unwrap_err();
                assert!(matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
                ));
                assert!(matches!(
                    error.origin(),
                    CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { .. }))
                ));
            }
            assert_eq!(components, before);
        }
        let source = checked(name, "var(--filter)", true);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        assert!(matches!(
            handle
                .reenter(parse_component_values(&format!("{ordinary}/*")).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        assert!(
            handle
                .reenter(parse_component_values(&format!("{ordinary}/**/")).unwrap())
                .is_ok()
        );
    }
}
#[test]
fn nested_filter_declarations_normalize_in_source_order_with_exact_cumulative_limits() {
    let css = ".outer{flood-color:red!important;flood-opacity:150%;.child{lighting-color:currentcolor}color-interpolation-filters:auto;flood-opacity:var(--alpha)}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 3, 5, 5).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 5);
    for (order, (value, name)) in values
        .iter()
        .zip([FLOOD, ALPHA, LIGHT, INTERPOLATION, ALPHA])
        .enumerate()
    {
        assert_eq!(value.order(), order);
        assert_eq!(value.source().known().unwrap().property(), property(name));
        assert_eq!(
            value.source().parsed_value().unwrap().source().as_str(),
            css
        );
        assert_eq!(
            value.source().importance(),
            if order == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
    }
    assert!(
        values[0]
            .selector_context()
            .same_context(values[1].selector_context())
    );
    assert!(
        values[0]
            .selector_context()
            .same_context(values[3].selector_context())
    );
    assert!(
        !values[0]
            .selector_context()
            .same_context(values[2].selector_context())
    );
    let CssExpansion::Pending(handle) = values[4].expansion() else {
        panic!("trailing pending")
    };
    assert!(handle.source().same_occurrence(values[4].source()));
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(1, 3, 4, 5).unwrap(),
            CssNormalizationResource::Declarations,
        ),
        (
            CssNormalizationLimits::try_new(1, 3, 5, 4).unwrap(),
            CssNormalizationResource::Contributions,
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit: 4 }
        );
        assert_eq!(error.declaration_order(), Some(4));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(values[4].source())
        );
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
#[test]
fn selected_filter_property_support_provenance_points_to_the_exact_filter1_clauses() {
    for (name, production) in [
        (FLOOD, "#FloodColorProperty"),
        (ALPHA, "#FloodOpacityProperty"),
        (LIGHT, "#LightingColorProperty"),
        (INTERPOLATION, "#ColorInterpolationFiltersProperty"),
    ] {
        let feature = property_support_metadata(name)
            .unwrap_or_else(|| panic!("missing Filter1 support: {name}"))
            .feature();
        assert_eq!(feature.source().id().as_str(), "I-FILTER1");
        assert_eq!(
            feature.source().url(),
            Some("https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/")
        );
        assert_eq!(feature.production(), production);
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        assert_eq!(feature.supported_subset(), None);
        assert_eq!(feature.unsupported_remainder(), None);
    }
}
#[test]
fn borrowed_keyword_color_and_numeric_providers_keep_three_node_declaration_prices() {
    for (name, text, output) in [
        (FLOOD, "red", "red"),
        (LIGHT, "currentcolor", "currentcolor"),
        (ALPHA, "150%", "1.5"),
        (INTERPOLATION, "linearRGB", "linearrgb"),
    ] {
        let source = checked(name, text, true);
        let before = source.clone();
        let expected = format!("{name}: {output} !important;");
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
            for _ in 0..2 {
                assert_eq!(
                    source
                        .to_specified_css_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(source, before);
            }
        }
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
    }
}
#[test]
fn sheet_siblings_share_final_bytes_and_fail_atomically_at_the_later_rule() {
    let report = parse_sheet(
        ".a{flood-color:red!important;flood-opacity:150%}.b{lighting-color:currentcolor;color-interpolation-filters:linearRGB}",
    );
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { flood-color: red !important; flood-opacity: 1.5; }\n.b { lighting-color: currentcolor; color-interpolation-filters: linearrgb; }";
    let exact = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len());
    let short =
        CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|v| v.to_specified_css_with_limits(short).is_ok())
    );
    for _ in 0..2 {
        let error = sheet.to_specified_css_with_limits(short).unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            )
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}
#[test]
fn huge_flood_exponents_report_atomic_output_limits_without_clamping_or_mutation() {
    let source = checked(ALPHA, "1e1000000000%", true);
    let before = source.clone();
    for _ in 0..2 {
        assert_eq!(
            source.to_specified_css().unwrap_err().kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(source, before);
    }
    canonical(
        ALPHA,
        "1e-999999999999999999999999999999999999999999999999999999",
        "0",
    );
}
#[test]
fn accepted_filter_function_controls_keep_existing_color_space_independent_grammar() {
    canonical(
        "filter",
        "blur() hue-rotate() opacity(150%)",
        "blur() hue-rotate() opacity(150%)",
    );
    canonical("backdrop-filter", "none", "none");
}
#[test]
fn existing_numeric_provider_controls_execute_all_opacity_goldens_before_new_registration() {
    for &(input, expected) in OPACITY_GOLDENS {
        canonical("opacity", input, expected);
    }
}
#[test]
fn existing_color_provider_controls_execute_all_color_goldens_before_new_registration() {
    for &(input, expected) in COLOR_GOLDENS {
        canonical("color", input, expected);
    }
}
#[test]
fn all_symbolic_reset_includes_each_filter_terminal_without_resolving_targets() {
    let source = checked("all", "revert-layer", false);
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("symbolic all")
    };
    assert_eq!(reset.keyword(), CssGlobalKeyword::RevertLayer);
    assert!(reset.source().same_occurrence(&source));
    for name in NAMES {
        assert!(!reset.excludes(CssPropertyNameRef::Known(property(name))));
    }
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::Direction)));
    assert!(reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::UnicodeBidi)));
}
#[test]
fn hinted_flood_calculation_uses_existing_seven_input_eight_projection_provider_price() {
    // Authored: calc root, product, sum, two sum leaves, inverse, divisor leaf.
    // Projection adds two merged scalar visits while removing the calc root.
    // Declaration and semantic name add two to each provider total.
    let source = checked(ALPHA, "calc((1px + 1%) / 1px)", true);
    let before = source.clone();
    let expected = "flood-opacity: calc((1% + 1px) / 1px) !important;";
    let exact = CssSpecifiedValueSerializationLimits::new(9, 10, expected.len());
    assert_eq!(
        source.to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(8, 10, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, 9, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
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
fn invalid_filter_declaration_diagnostics_keep_byte_and_utf16_coordinates() {
    for (name, value) in [
        (FLOOD, "auto"),
        (ALPHA, "1px"),
        (LIGHT, "auto"),
        (INTERPOLATION, "srgb-linear"),
    ] {
        let prefix = "/*😀*/ ";
        let target = format!("{name}:{value};");
        let css = format!("{prefix}{target}color:blue");
        let report = parse_style_attribute(&css);
        assert!(!report.is_clean());
        let [sibling] = report.syntax().as_slice() else {
            panic!("one surviving sibling")
        };
        assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
        let [diagnostic] = report.diagnostics() else {
            panic!("one value diagnostic")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let span = diagnostic.span();
        assert_eq!(span.start().byte_offset().value(), prefix.len());
        assert_eq!(
            span.end().byte_offset().value(),
            prefix.len() + target.len()
        );
        assert_eq!(
            span.start().column().value() as usize,
            prefix.encode_utf16().count()
        );
        assert_eq!(
            span.end().column().value() as usize,
            css[..prefix.len() + target.len()].encode_utf16().count()
        );
        let position = diagnostic.error().position();
        let byte = position.byte_offset().value();
        assert!(byte > prefix.len() + name.len() && byte < prefix.len() + target.len());
        assert_eq!(
            position.column().value() as usize,
            css[..byte].encode_utf16().count()
        );
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}
