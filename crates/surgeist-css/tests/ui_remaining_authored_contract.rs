#![forbid(unsafe_code)]
//! Uncompiled preparation for UI4 WD20260120 remaining authored owners.
//! Normative clauses: outline §3.1–3.5; caret §5.2.2–5.2.4;
//! navigation §5.3.1; obsolete IME §5.3.2; interaction §6.3–6.4;
//! accent/appearance §7.1–7.2, including required legacy name alias.
//! Outline imports Border4 WD20251216 §2.1 and Images4 WD20250930 §4/8.
//! Expectations are explicit specialized stimuli, never production inventories.
//! Generic metadata/catalog/dispatch expectations belong in common records.rs.
//! New typed models/borrowed variants require functional implementation tests.
use surgeist_css::*;

const COLOR_GOLDENS: &[(&str, &str)] = &[
    ("red", "red"),
    ("currentcolor", "currentcolor"),
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
    ("color(--P 0% 70% 20% 0%)", "color(--P 0 0.7 0.2 0)"),
    ("rgb(from red r g b / alpha)", "rgb(from red r g b / alpha)"),
    ("lab(from red l a b)", "lab(from red l a b)"),
    (
        "color(from red --P Cyan / alpha)",
        "color(from red --P Cyan / alpha)",
    ),
    ("color-mix(in oklab, red, blue)", "color-mix(red, blue)"),
    (
        "color-mix(in oklch longer hue, red 25%, blue 75%)",
        "color-mix(in oklch longer hue, red 25%, blue 75%)",
    ),
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

fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name).unwrap_or_else(|| panic!("missing authored family: {name}"))
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
    assert_eq!(source.importance(), CssImportance::Important);
    source
}
fn fronts(name: &str, value: &str) -> Vec<CssDeclaration> {
    let css = format!("/*😀*/{}:{value}!important", name.to_ascii_uppercase());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert_eq!(&validate_style_attribute(&css).unwrap(), report.syntax());
    let [parsed] = report.syntax().as_slice() else {
        panic!("one authored occurrence")
    };
    assert_eq!(parsed.known().unwrap().property(), property(name));
    let start = parsed.position().unwrap();
    assert_eq!(start.byte_offset().value(), "/*😀*/".len());
    assert_eq!(
        start.column().value() as usize,
        "/*😀*/".encode_utf16().count()
    );
    assert_eq!(parsed.parsed_value().unwrap().source().as_str(), css);
    let mut result = vec![
        parsed.clone(),
        checked(name, value, false),
        checked(name, value, true),
    ];
    let p = property(name);
    for report in [
        parse_property_value_text(
            value,
            CssPropertyNameRef::Known(p),
            CssImportance::Important,
        ),
        parse_property_value_text_for_grammar(value, p.grammar(), CssImportance::Important),
    ] {
        assert!(
            report.is_clean(),
            "{name}:{value}: {:?}",
            report.diagnostics()
        );
        let source = report.syntax().as_ref().unwrap();
        assert_eq!(source.known().unwrap().grammar(), p.grammar());
        assert_eq!(source.parsed_value().unwrap().source().as_str(), value);
        assert!(source.position().is_none());
        assert!(source.parsed_name().is_none());
        result.push(source.clone());
    }
    result
}
fn canonical(name: &str, text: &str, expected: &str) {
    for source in fronts(name, text) {
        let before = source.clone();
        let expected = format!("{name}: {expected} !important;");
        assert_eq!(source.to_specified_css().unwrap(), expected);
        assert_eq!(source, before);
        let limits = CssSpecifiedValueSerializationLimits::new(65_536, 262_144, expected.len());
        assert_eq!(
            source.to_specified_css_with_limits(limits).unwrap(),
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
            source.to_specified_css_with_limits(limits).unwrap(),
            expected
        );
    }
    assert_eq!(
        checked(name, expected, true).to_specified_css().unwrap(),
        format!("{name}: {expected} !important;")
    );
}
fn invalid(name: &str, text: &str) {
    let p = property(name);
    let css = format!("color:red;{name}:{text};color:blue");
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "atomic rejection: {css}");
    assert!(
        report
            .syntax()
            .iter()
            .all(|d| d.known().unwrap().property() == CssKnownProperty::Color)
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one grammar failure: {css}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    match diagnostic.error().kind() {
        ErrorKind::InvalidPropertyValue(v) => assert_eq!(v.property(), p),
        ErrorKind::InvalidColorSyntax(_) if matches!(name, "outline-color" | "accent-color") => {}
        other => panic!("owning grammar diagnostic: {other:?}"),
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
fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary terminals")
    };
    for item in values.items() {
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), source.importance());
        assert_eq!(item.source().value_components(), source.value_components());
    }
    values
}
fn shape(values: &CssLonghandContributions, names: &[&str]) {
    let actual: Vec<_> = values
        .items()
        .iter()
        .map(|v| v.property().canonical_name())
        .collect();
    assert_eq!(actual, names);
}
fn reset_values(name: &str, text: &str, expected: &[(&str, &str)]) {
    for source in fronts(name, text) {
        let before = source.clone();
        let values = contributions(&source);
        shape(
            &values,
            &expected.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
        );
        for (item, (terminal, primitive)) in values.items().iter().zip(expected) {
            // Explicit source-derived primitive is parsed only to bridge the
            // existing generic value API. Exact new payload tests must accompany
            // implementation; this comparison alone cannot prove typed modeling.
            let control = checked(terminal, primitive, true);
            let controls = contributions(&control);
            assert_eq!(controls.items().len(), 1);
            assert_eq!(item.ordinary_value(), controls.items()[0].ordinary_value());
            assert!(item.replacement_components().is_none());
        }
        assert_eq!(source, before);
    }
}
fn globals_and_all(name: &str, names: &[&str]) {
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for source in fronts(name, text) {
            let values = contributions(&source);
            shape(&values, names);
            for item in values.items() {
                assert!(matches!(item.value(),CssContributionValueRef::Global(v) if v==keyword));
                assert!(item.ordinary_value().is_none());
            }
        }
    }
    let CssPropertyKindRef::UniversalReset(all) = property("all").metadata().unwrap().kind() else {
        panic!("all metadata")
    };
    assert!(!all.excludes(CssPropertyNameRef::Known(property(name))));
    for terminal in names {
        assert!(!all.excludes(CssPropertyNameRef::Known(property(terminal))));
    }
}
fn pending(name: &str, replacement: &str, names: &[&str]) {
    for text in ["var(--ui)", "env(ui)", "attr(data-ui *)"] {
        for source in fronts(name, text) {
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("one whole-value pending handle")
            };
            assert!(handle.source().same_occurrence(&source));
            for residual in [
                "var(--again)",
                "env(again)",
                "attr(data-again *)",
                "var(--again",
            ] {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(residual).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            }
            for rejected in [
                "bogus",
                "initial inherit",
                "auto!important",
                "auto;color:red",
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
            let components = parse_component_values(replacement).unwrap();
            let snapshot = components.clone();
            let CssContributions::Longhands(values) = handle.reenter(components.clone()).unwrap()
            else {
                panic!("resolved terminals")
            };
            shape(&values, names);
            let control = contributions(&checked(name, replacement, true));
            for (item, ordinary) in values.items().iter().zip(control.items()) {
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&components));
                assert_eq!(item.ordinary_value(), ordinary.ordinary_value());
            }
            let CssContributions::Longhands(global) = handle
                .reenter(parse_component_values("revert-layer").unwrap())
                .unwrap()
            else {
                panic!("global retry")
            };
            shape(&global, names);
            assert!(global.items().iter().all(|v| matches!(
                v.value(),
                CssContributionValueRef::Global(CssGlobalKeyword::RevertLayer)
            )));
            assert_eq!(components, snapshot);
            assert_eq!(source, before);
        }
    }
}
fn closure(name: &str, ordinary: &str) {
    for text in [
        format!("{ordinary}/*"),
        "initial/*".to_owned(),
        "var(--ui".to_owned(),
        "env(ui".to_owned(),
        "attr(data-ui *".to_owned(),
    ] {
        let components = parse_component_values(&text).unwrap();
        let snapshot = components.clone();
        let serialized = components.serialize().unwrap();
        let origin = (0..=serialized.as_css().len())
            .find_map(|n| match serialized.origin_at(n) {
                Some(CssSerializedOrigin::Token(v @ CssValueOrigin::ImplicitClosure { .. })) => {
                    Some(v.clone())
                }
                Some(CssSerializedOrigin::End(Some(
                    v @ CssValueOrigin::ImplicitClosure { .. },
                ))) => Some(v.clone()),
                _ => None,
            })
            .or_else(|| {
                components.items().iter().find_map(|v| match v.view() {
                    CssComponentValueRef::Function(f) => match f.closing_origin() {
                        v @ CssValueOrigin::ImplicitClosure { .. } => Some(v.clone()),
                        _ => None,
                    },
                    _ => None,
                })
            })
            .expect("original implicit origin");
        for grammar in [false, true] {
            let error = checked_components(name, components.clone(), grammar).unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ));
            assert_eq!(
                error.origin(),
                &CssSerializedOrigin::End(Some(origin.clone()))
            );
        }
        assert_eq!(components, snapshot);
    }
    let source = checked(name, "var(--ui)", true);
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending closure owner")
    };
    let replacement = parse_component_values(&format!("{ordinary}/*")).unwrap();
    let error = handle.reenter(replacement).unwrap_err();
    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
        panic!("strict original replacement closure")
    };
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert!(
        handle
            .reenter(parse_component_values(&format!("{ordinary}/**/")).unwrap())
            .is_ok()
    );
    checked(name, &format!("{ordinary}/**/"), true);
}

// Prior accepted Outline subset controls never call metadata/expansion.
#[test]
fn outline_accepted_provider_goldens_remain_callable_independently_of_lifecycle() {
    for (text, expected) in [
        ("red solid 2px", "2px solid red"),
        ("auto", "auto"),
        ("auto auto 2px", "2px auto"),
        ("auto solid", "solid auto"),
        ("2px red auto", "2px auto red"),
        ("thin", "thin"),
    ] {
        canonical("outline", text, expected);
    }
    for (name, text, expected) in [
        ("outline-color", "AUTO", "auto"),
        ("outline-color", "#abc", "rgb(170, 187, 204)"),
        (
            "outline-color",
            "light-dark(red,blue)",
            "light-dark(red, blue)",
        ),
        (
            "outline-color",
            "color-mix(in oklab, red, blue)",
            "color-mix(red, blue)",
        ),
        ("outline-width", "2.00PX", "2px"),
        ("outline-offset", "-2.50em", "-2.5em"),
        ("outline-offset", "calc(1px - 2em)", "calc(-2em + 1px)"),
    ] {
        canonical(name, text, expected);
    }
}
#[test]
fn outline_hidden_invert_duplicate_and_negative_width_controls_remain_rejected() {
    for text in [
        "hidden",
        "solid hidden",
        "solid dashed",
        "red blue",
        "1px 2px",
        "auto auto auto",
        "solid red auto",
    ] {
        invalid("outline", text);
    }
    invalid("outline-style", "hidden");
    invalid("outline-color", "invert");
    invalid("outline-width", "-1px");
    invalid("outline-width", "1%");
    invalid("outline-offset", "1%");
}
#[test]
fn outline_constructed_projection_and_omitted_second_auto_keep_accepted_prices() {
    for (value, expected, input, projection) in [
        (
            CssOutline::try_new(None, None, Some(CssOutlineColor::Auto)).unwrap(),
            "none auto",
            2,
            3,
        ),
        (
            CssOutline::try_new(
                None,
                Some(CssOutlineStyle::Auto),
                Some(CssOutlineColor::Auto),
            )
            .unwrap(),
            "auto",
            3,
            3,
        ),
    ] {
        let before = value.clone();
        let exact = CssSpecifiedValueSerializationLimits::new(input, projection, expected.len());
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(input - 1, projection, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(input, projection - 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(value, before);
    }
}
#[test]
fn outline_imported_image_1d_grammar_and_canonical_order() {
    for (text, expected) in [
        ("stripes(red,blue)", "stripes(red, blue)"),
        (
            "stripes(2px red, blue 25%, green .5fr)",
            "stripes(red 2px, blue 25%, green 0.5fr)",
        ),
        (
            "stripes(red 0%,blue 100%,green 0fr)",
            "stripes(red 0%, blue 100%, green 0fr)",
        ),
        ("stripes(red -1fr)", "stripes(red -1fr)"),
        ("stripes(red calc(1fr + 2fr))", "stripes(red calc(3fr))"),
        (
            "stripes(light-dark(red,blue) 2px)",
            "stripes(light-dark(red, blue) 2px)",
        ),
        (
            "stripes(red calc(1px + 1em),blue)",
            "stripes(red calc(1px + 1em), blue)",
        ),
    ] {
        canonical("outline-color", text, expected);
    }
    canonical(
        "outline",
        "stripes(red,blue) solid thin",
        "thin solid stripes(red, blue)",
    );
}
#[test]
fn outline_image_1d_rejects_missing_color_duplicates_and_explicit_ranges() {
    for text in [
        "stripes()",
        "stripes(1px)",
        "stripes(red blue)",
        "stripes(red 2px 3px)",
        "stripes(red,)",
        "stripes(red -1px)",
        "stripes(red -1%)",
        "stripes(red 101%)",
        "stripes(red 1s)",
        "stripes(red calc(1fr + 1px))",
        "url(a)",
        "linear-gradient(red,blue)",
    ] {
        invalid("outline-color", text);
    }
}
#[test]
fn outline_sets_exactly_three_members_and_never_resets_offset() {
    reset_values(
        "outline",
        "red",
        &[
            ("outline-width", "medium"),
            ("outline-style", "none"),
            ("outline-color", "red"),
        ],
    );
    reset_values(
        "outline",
        "auto",
        &[
            ("outline-width", "medium"),
            ("outline-style", "auto"),
            ("outline-color", "auto"),
        ],
    );
    reset_values(
        "outline",
        "thin",
        &[
            ("outline-width", "thin"),
            ("outline-style", "none"),
            ("outline-color", "auto"),
        ],
    );
    reset_values(
        "outline",
        "solid",
        &[
            ("outline-width", "medium"),
            ("outline-style", "solid"),
            ("outline-color", "auto"),
        ],
    );
    reset_values(
        "outline",
        "stripes(red,blue)",
        &[
            ("outline-width", "medium"),
            ("outline-style", "none"),
            ("outline-color", "stripes(red,blue)"),
        ],
    );
}

#[test]
fn navigation_preserves_case_sensitive_ids_and_strings_with_optional_target() {
    for name in ["nav-up", "nav-right", "nav-down", "nav-left"] {
        for (text, expected) in [
            ("AUTO", "auto"),
            ("#Next", "#Next"),
            ("#Next CURRENT", "#Next current"),
            ("#Next ROOT", "#Next root"),
            ("#Next 'Frame'", "#Next \"Frame\""),
            ("#Next ''", "#Next \"\""),
            (r"#\4e ext root", "#Next root"),
        ] {
            canonical(name, text, expected);
        }
    }
}
#[test]
fn navigation_rejects_non_id_selectors_and_extra_roles() {
    for name in ["nav-up", "nav-right", "nav-down", "nav-left"] {
        for text in [
            "#123",
            ".next",
            "next",
            "#next #other",
            "#next next",
            "auto root",
            "#next root current",
        ] {
            invalid(name, text);
        }
    }
}
#[test]
fn navigation_checked_authored_targets_exclude_underscore_without_assuming_legacy_execution() {
    // The normative legacy repair behavior is a separate symbolic recovery
    // contract; no browser-drop or frame-execution oracle is asserted here.
    for name in ["nav-up", "nav-right", "nav-down", "nav-left"] {
        for text in [
            r#"#next "_parent""#,
            r#"#next "_root""#,
            r#"#next "_Frame""#,
            r#"#next "\5f root""#,
        ] {
            let values = parse_component_values(text).unwrap();
            for grammar in [false, true] {
                assert!(matches!(
                    checked_components(name, values.clone(), grammar)
                        .unwrap_err()
                        .kind(),
                    CssPropertyValueErrorKind::Grammar(_)
                ));
            }
            assert!(validate_style_attribute(&format!("{name}:{text}")).is_err());
        }
    }
}

#[test]
fn browser_navigation_retains_nonconforming_legacy_target_with_diagnostics_and_original_occurrence()
{
    // Author/repair split selected from MUST NOT plus explicit legacy handling
    // in UI4 §5.3.1. Future exact typed target/error/action tests accompany APIs.
    for name in ["nav-up", "nav-right", "nav-down", "nav-left"] {
        for (text, expected) in [
            (r#"#Next "_parent""#, r#"#Next "_parent""#),
            (r#"#Next "_root""#, r#"#Next "_root""#),
            (r#"#Next "_Frame""#, r#"#Next "_Frame""#),
            (r#"#Next "_ROOT""#, r#"#Next "_ROOT""#),
            (r#"#Next "\5f root""#, r#"#Next "_root""#),
        ] {
            let css = format!(".a{{color:red;{name}:{text}!important;color:blue}}");
            let report = parse_sheet(&css);
            let before = report.clone();
            assert!(!report.is_clean());
            assert!(!report.diagnostics().is_empty());
            assert!(validate_sheet(&css).is_err());
            let normalized = normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 3, 3).unwrap(),
            )
            .unwrap();
            assert_eq!(normalized.diagnostics(), report.diagnostics());
            let items: Vec<_> = normalized
                .syntax()
                .items()
                .iter()
                .filter_map(|v| match v {
                    CssNormalizedItem::Declaration(v) => Some(v),
                    _ => None,
                })
                .collect();
            assert_eq!(items.len(), 3);
            let item = items[1];
            let source = item.source();
            assert_eq!(source.known().unwrap().property(), property(name));
            assert_eq!(source.importance(), CssImportance::Important);
            assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {expected} !important;")
            );
            let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
            else {
                panic!("recovered symbolic terminal")
            };
            shape(values, &[name]);
            assert!(values.items()[0].source().same_occurrence(source));
            assert_eq!(report, before);
            let p = property(name);
            for value_report in [
                parse_property_value_text(
                    text,
                    CssPropertyNameRef::Known(p),
                    CssImportance::Important,
                ),
                parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Important),
            ] {
                assert!(!value_report.is_clean());
                assert!(!value_report.diagnostics().is_empty());
                let source = value_report.syntax().as_ref().unwrap();
                assert_eq!(source.parsed_value().unwrap().source().as_str(), text);
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: {expected} !important;")
                );
            }
        }
    }
}

#[test]
fn obsolete_ime_has_no_invented_known_grammar_and_keeps_atomic_unknown_recovery() {
    assert!(CssKnownProperty::from_name("ime-mode").is_none());
    assert!(CssPropertyGrammar::from_name("ime-mode").is_none());
    for text in [
        "auto", "active", "inactive", "disabled", "normal", "inherit",
    ] {
        let css = format!("color:red;ime-mode:{text}!important;color:blue");
        let report = parse_style_attribute(&css);
        assert_eq!(report.syntax().len(), 2);
        let [diagnostic] = report.diagnostics() else {
            panic!("one obsolete unknown diagnostic")
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert!(validate_style_attribute(&css).is_err());
    }
}

#[test]
fn caret_keywords_and_unordered_noninitial_roles_have_explicit_grammar_order() {
    for text in ["auto", "manual"] {
        canonical("caret-animation", &text.to_ascii_uppercase(), text);
    }
    for text in ["auto", "bar", "block", "underscore"] {
        canonical("caret-shape", &text.to_ascii_uppercase(), text);
    }
    for (text, expected) in [
        ("block manual red", "red manual block"),
        ("manual red block", "red manual block"),
        ("red block manual", "red manual block"),
        ("block manual", "manual block"),
        ("red", "red"),
        ("manual", "manual"),
        ("block", "block"),
        ("auto", "auto"),
    ] {
        canonical("caret", text, expected);
    }
}
#[test]
fn caret_shared_auto_spellings_fill_at_most_three_roles_and_keep_original_tokens() {
    for text in [
        "auto auto",
        "auto auto auto",
        "red auto auto",
        "manual auto auto",
        "block auto auto",
        r"\61 uto manual block",
    ] {
        for source in fronts("caret", text) {
            assert!(source.known().unwrap().property_value().is_some());
            assert!(source.known().unwrap().global().is_none());
            assert!(source.known().unwrap().substitution_dependent().is_none());
        }
    }
    reset_values(
        "caret",
        "auto auto auto",
        &[
            ("caret-color", "auto"),
            ("caret-animation", "auto"),
            ("caret-shape", "auto"),
        ],
    );
    reset_values(
        "caret",
        "red auto auto",
        &[
            ("caret-color", "red"),
            ("caret-animation", "auto"),
            ("caret-shape", "auto"),
        ],
    );
}

#[test]
fn caret_rejects_duplicate_excess_and_cross_role_values() {
    for text in ["none", "blink", "manual manual", "bar"] {
        invalid("caret-animation", text);
    }
    for text in ["manual", "none", "block bar"] {
        invalid("caret-shape", text);
    }
    for text in [
        "red blue",
        "manual manual",
        "bar block",
        "red manual block auto",
        "auto auto auto auto",
        "none",
        "solid",
    ] {
        invalid("caret", text);
    }
}
#[test]
fn caret_resets_omitted_color_animation_and_shape_to_independent_initials() {
    reset_values(
        "caret",
        "manual",
        &[
            ("caret-color", "auto"),
            ("caret-animation", "manual"),
            ("caret-shape", "auto"),
        ],
    );
    reset_values(
        "caret",
        "block",
        &[
            ("caret-color", "auto"),
            ("caret-animation", "auto"),
            ("caret-shape", "block"),
        ],
    );
    reset_values(
        "caret",
        "red",
        &[
            ("caret-color", "red"),
            ("caret-animation", "auto"),
            ("caret-shape", "auto"),
        ],
    );
    reset_values(
        "caret",
        "auto",
        &[
            ("caret-color", "auto"),
            ("caret-animation", "auto"),
            ("caret-shape", "auto"),
        ],
    );
    reset_values(
        "caret",
        "red manual block",
        &[
            ("caret-color", "red"),
            ("caret-animation", "manual"),
            ("caret-shape", "block"),
        ],
    );
}
#[test]
fn interactivity_has_exact_authored_keyword_domain() {
    canonical("interactivity", "AUTO", "auto");
    canonical("interactivity", "INERT", "inert");
    for text in ["none", "active", "disabled", "inert auto"] {
        invalid("interactivity", text);
    }
}
#[test]
fn interest_delay_accepts_signed_time_and_normal_without_duration_range() {
    for name in ["interest-delay-start", "interest-delay-end"] {
        for (text, expected) in [
            ("NORMAL", "normal"),
            ("-1.250S", "-1.25s"),
            ("+2.000MS", "0.002s"),
            ("-0s", "0s"),
            ("calc(1s + 250ms)", "calc(1.25s)"),
            ("calc(-1s)", "calc(-1s)"),
        ] {
            canonical(name, text, expected);
        }
    }
    canonical("interest-delay", "-1s 2ms", "-1s 0.002s");
    canonical("interest-delay", "normal 2ms", "normal 0.002s");
    canonical("interest-delay", "-1s", "-1s");
}
#[test]
fn interest_delay_rejects_numbers_dimensions_percentages_and_excess_values() {
    for name in [
        "interest-delay-start",
        "interest-delay-end",
        "interest-delay",
    ] {
        for text in [
            "0",
            "-1",
            "auto",
            "1px",
            "1%",
            "calc(1px)",
            "normal normal normal",
            "1s,2s",
        ] {
            invalid(name, text);
        }
    }
    invalid("interest-delay-start", "1s 2s");
    invalid("interest-delay-end", "normal 1s");
}
#[test]
fn interest_delay_repetition_sets_both_members_and_two_values_stay_distinct() {
    reset_values(
        "interest-delay",
        "-1s",
        &[
            ("interest-delay-start", "-1s"),
            ("interest-delay-end", "-1s"),
        ],
    );
    reset_values(
        "interest-delay",
        "normal",
        &[
            ("interest-delay-start", "normal"),
            ("interest-delay-end", "normal"),
        ],
    );
    reset_values(
        "interest-delay",
        "normal 2ms",
        &[
            ("interest-delay-start", "normal"),
            ("interest-delay-end", "0.002s"),
        ],
    );
}
#[test]
fn accent_imports_selected_color_forms_and_keeps_auto_symbolic() {
    for (text, expected) in [
        ("AUTO", "auto"),
        ("Canvas", "canvas"),
        ("currentcolor", "currentcolor"),
        ("#abc", "rgb(170, 187, 204)"),
        ("light-dark(red,blue)", "light-dark(red, blue)"),
        ("color-mix(in oklab,red,blue)", "color-mix(red, blue)"),
        ("rgb(from red r g b)", "rgb(from red r g b)"),
        ("alpha(from red / 50%)", "alpha(from red / 0.5)"),
    ] {
        canonical("accent-color", text, expected);
    }
    for text in ["none", "auto red", "stripes(red,blue)", "url(a)"] {
        invalid("accent-color", text);
    }
}
#[test]
fn appearance_retains_every_selected_keyword_including_compatibility_identity() {
    for text in [
        "none",
        "auto",
        "base",
        "base-select",
        "searchfield",
        "textarea",
        "checkbox",
        "radio",
        "menulist",
        "listbox",
        "meter",
        "progress-bar",
        "button",
        "textfield",
        "menulist-button",
    ] {
        canonical("appearance", &text.to_ascii_uppercase(), text);
    }
    for text in [
        "push-button",
        "square-button",
        "listitem",
        "slider-horizontal",
        "caret",
        "button auto",
    ] {
        invalid("appearance", text);
    }
}
#[test]
fn standard_webkit_appearance_name_alias_preserves_original_name_and_canonical_output() {
    let p = property("appearance");
    assert_eq!(CssKnownProperty::from_name("-WEBKIT-APPEARANCE"), Some(p));
    assert_eq!(
        CssPropertyGrammar::from_name("-webkit-appearance"),
        Some(p.grammar())
    );
    let report = parse_style_attribute("/*😀*/-WEBKIT-APPEARANCE:TEXTFIELD!important");
    assert!(report.is_clean());
    let source = &report.syntax()[0];
    assert_eq!(
        source.parsed_name().unwrap().source().as_str(),
        "/*😀*/-WEBKIT-APPEARANCE:TEXTFIELD!important"
    );
    assert_eq!(
        source.to_specified_css().unwrap(),
        "appearance: textfield !important;"
    );
    let values = contributions(source);
    shape(&values, &["appearance"]);
}

// Specialized per-owner lifecycle stimuli; this does not replace common records.
macro_rules! lifecycle {
    ($global:ident,$pending:ident,$closure:ident,[$(($name:literal,$value:literal,[$($terminal:literal),+])),+ $(,)?]) => {
        #[test] fn $global(){ $(globals_and_all($name,&[$($terminal),+]));+ }
        #[test] fn $pending(){ $(pending($name,$value,&[$($terminal),+]));+ }
        #[test] fn $closure(){ $(closure($name,$value));+ }
    };
}
lifecycle!(
    outline_globals_and_all,
    outline_pending_and_strict_retry,
    outline_original_closure,
    [
        (
            "outline",
            "thin solid red",
            ["outline-width", "outline-style", "outline-color"]
        ),
        ("outline-color", "red", ["outline-color"]),
        ("outline-style", "solid", ["outline-style"]),
        ("outline-width", "thin", ["outline-width"]),
        ("outline-offset", "-2px", ["outline-offset"])
    ]
);
lifecycle!(
    navigation_globals_and_all,
    navigation_pending_and_strict_retry,
    navigation_original_closure,
    [
        ("nav-up", "#Next root", ["nav-up"]),
        ("nav-right", "#Next current", ["nav-right"]),
        ("nav-down", "#Next", ["nav-down"]),
        ("nav-left", "auto", ["nav-left"])
    ]
);
lifecycle!(
    caret_globals_and_all,
    caret_pending_and_strict_retry,
    caret_original_closure,
    [
        (
            "caret",
            "red manual block",
            ["caret-color", "caret-animation", "caret-shape"]
        ),
        ("caret-animation", "manual", ["caret-animation"]),
        ("caret-shape", "block", ["caret-shape"])
    ]
);
lifecycle!(
    interaction_globals_and_all,
    interaction_pending_and_strict_retry,
    interaction_original_closure,
    [
        ("interactivity", "inert", ["interactivity"]),
        (
            "interest-delay",
            "-1s 2ms",
            ["interest-delay-start", "interest-delay-end"]
        ),
        ("interest-delay-start", "-1s", ["interest-delay-start"]),
        ("interest-delay-end", "normal", ["interest-delay-end"])
    ]
);
lifecycle!(
    widgets_globals_and_all,
    widgets_pending_and_strict_retry,
    widgets_original_closure,
    [
        ("accent-color", "red", ["accent-color"]),
        ("appearance", "textfield", ["appearance"])
    ]
);

#[test]
fn composed_ui_normalization_keeps_order_alias_importance_and_exact_terminal_budget() {
    let css = ".a{outline:red!important;outline-offset:-2px;caret:block manual;nav-up:#Next root;nav-right:auto;nav-down:#Down;nav-left:#Left \"Frame\";interactivity:inert;interest-delay:-1s 2ms;accent-color:Canvas;-webkit-appearance:TEXTFIELD;caret-color:transparent}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(0, 1, 12, 17).unwrap();
    let normalized = normalize_report_with_limits(&report, exact).unwrap();
    let items: Vec<_> = normalized
        .syntax()
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 12);
    for (index, (item, (name, count))) in items
        .iter()
        .zip([
            ("outline", 3),
            ("outline-offset", 1),
            ("caret", 3),
            ("nav-up", 1),
            ("nav-right", 1),
            ("nav-down", 1),
            ("nav-left", 1),
            ("interactivity", 1),
            ("interest-delay", 2),
            ("accent-color", 1),
            ("appearance", 1),
            ("caret-color", 1),
        ])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().property(), property(name));
        assert_eq!(
            item.source().importance(),
            if index == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("resolved UI terminals")
        };
        assert_eq!(values.items().len(), count);
        assert!(
            values
                .items()
                .iter()
                .all(|v| v.source().same_occurrence(item.source()))
        );
        assert_eq!(item.source().parsed_value().unwrap().source().as_str(), css);
    }
    let error = normalize_report_with_limits(
        &report,
        CssNormalizationLimits::try_new(0, 1, 12, 16).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 16
        }
    );
    assert_eq!(error.declaration_order(), Some(11));
    assert_eq!(report, before);
    assert!(normalize_report_with_limits(&report, exact).is_ok());
}

#[test]
fn existing_caret_color_provider_executes_all_imported_goldens_without_remaining_metadata() {
    for &(input, expected) in COLOR_GOLDENS {
        canonical("caret-color", input, expected);
    }
}
#[test]
fn remaining_outline_and_accent_import_every_adopted_color_form() {
    for &(input, expected) in COLOR_GOLDENS {
        canonical("outline-color", input, expected);
        canonical("accent-color", input, expected);
    }
}
#[test]
fn caret_shorthand_imports_complete_color_child_without_reordering_its_channels() {
    for &(input, expected) in COLOR_GOLDENS {
        canonical(
            "caret",
            &format!("block {input} manual"),
            &format!("{expected} manual block"),
        );
    }
}
#[test]
fn existing_signed_time_provider_controls_are_independent_of_interest_registration() {
    for (input, expected) in [
        ("-1.250S", "-1.25s"),
        ("+2.000MS", "0.002s"),
        ("-0s", "0s"),
        ("calc(1s + 250ms)", "calc(1.25s)"),
        ("calc(-1s)", "calc(-1s)"),
    ] {
        canonical("animation-delay", input, expected);
    }
    let time =
        CssTimeValue::from_literal(CssTimeLiteral::try_new("-1", CssTimeUnit::Seconds).unwrap());
    assert_eq!(time.literal().unwrap().numeric().representation(), "-1");
    assert!(CssDuration::try_new(time).is_err());
}
#[test]
fn scalar_keyword_declarations_share_existing_three_node_accounting() {
    for (name, word) in [
        ("caret-animation", "manual"),
        ("caret-shape", "block"),
        ("nav-up", "auto"),
        ("nav-right", "auto"),
        ("nav-down", "auto"),
        ("nav-left", "auto"),
        ("interactivity", "inert"),
        ("interest-delay-start", "normal"),
        ("interest-delay-end", "normal"),
        ("accent-color", "auto"),
        ("appearance", "textfield"),
    ] {
        let source = checked(name, word, true);
        let before = source.clone();
        let expected = format!("{name}: {word} !important;");
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
}

#[test]
fn programmatic_keyword_components_keep_supplied_origins_in_new_authored_roles() {
    for (name, text) in [
        ("caret-animation", "manual"),
        ("caret-shape", "block"),
        ("nav-up", "auto"),
        ("interactivity", "inert"),
        ("interest-delay-start", "normal"),
        ("interest-delay-end", "normal"),
        ("accent-color", "auto"),
        ("appearance", "textfield"),
    ] {
        let values =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident(text).unwrap()]).unwrap();
        let before = values.clone();
        for grammar in [false, true] {
            let source = checked_components(name, values.clone(), grammar).unwrap();
            assert_eq!(source.value_components(), &values);
            assert_eq!(
                source.value_components().items()[0].origin(),
                &CssValueOrigin::Programmatic
            );
            assert!(source.position().is_none());
            assert!(source.parsed_name().is_none());
            assert!(source.parsed_value().is_none());
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: {text} !important;")
            );
            let terminals = contributions(&source);
            shape(&terminals, &[name]);
            assert!(terminals.items()[0].source().same_occurrence(&source));
        }
        assert_eq!(values, before);
    }
}

#[test]
fn browser_recovery_retains_original_closures_and_complete_shapes_with_diagnostics() {
    for (name, value, count) in [
        ("outline", "red", 3),
        ("outline-offset", "-2px", 1),
        ("nav-up", "#Next root", 1),
        ("caret", "manual block", 3),
        ("caret-animation", "manual", 1),
        ("caret-shape", "block", 1),
        ("interactivity", "inert", 1),
        ("interest-delay", "-1s", 2),
        ("interest-delay-start", "-1s", 1),
        ("interest-delay-end", "normal", 1),
        ("accent-color", "red", 1),
        ("appearance", "textfield", 1),
    ] {
        for text in [format!("{value}/*"), "var(--ui".to_owned()] {
            let css = format!(".a{{{name}:{text}");
            let report = parse_sheet(&css);
            let before = report.clone();
            assert!(!report.is_clean());
            assert!(validate_sheet(&css).is_err());
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
            );
            let normalized = normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 1, count).unwrap(),
            )
            .unwrap();
            assert_eq!(normalized.diagnostics(), report.diagnostics());
            let items: Vec<_> = normalized
                .syntax()
                .items()
                .iter()
                .filter_map(|v| match v {
                    CssNormalizedItem::Declaration(v) => Some(v),
                    _ => None,
                })
                .collect();
            assert_eq!(items.len(), 1);
            let item = items[0];
            assert_eq!(item.source().known().unwrap().property(), property(name));
            assert_eq!(item.source().parsed_value().unwrap().source().as_str(), css);
            match item.expansion() {
                CssExpansion::Pending(handle) => {
                    assert!(handle.source().same_occurrence(item.source()))
                }
                CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                    assert_eq!(values.items().len(), count);
                    assert!(
                        values
                            .items()
                            .iter()
                            .all(|v| v.source().same_occurrence(item.source()))
                    );
                }
                _ => panic!("retained known lifecycle"),
            }
            assert_eq!(report, before);
        }
    }
}
