#![forbid(unsafe_code)]
//! Decoration 3 CRD20220505 and 4 WD20220504 authored lifecycle contracts.
//! Existing callable fronts only. No future typed variants or constructors.
//! Generic metadata/catalog/alias/dispatch facts belong solely in common records.rs.
//! Specialized grammar, expansion, closure, source and budget stimuli stay here.
//! Canonical oracles: accepted bf43 Decoration and shared Color/Number/Shadow
//! contracts; Values4 math simplification/unit ordering; new Emphasis default
//! omission is an explicitly documented CSSOM + Decoration4 inference.
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
        ErrorKind::InvalidColorSyntax(_)
            if matches!(name, "text-decoration-color" | "text-emphasis-color") => {}
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
    for text in [
        "var(--decoration)",
        "env(decoration)",
        "attr(data-decoration *)",
    ] {
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
        "var(--decoration".to_owned(),
        "env(decoration".to_owned(),
        "attr(data-decoration *".to_owned(),
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
    let source = checked(name, "var(--decoration)", true);
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

// These primitive witnesses are independent expected strings. They never query
// provider output to derive an oracle for a new consumer.
const SIGNED_LENGTH_PERCENTAGE_GOLDENS: &[(&str, &str)] = &[
    ("0", "0"),
    ("-10px", "-10px"),
    ("-27%", "-27%"),
    ("-1e-999%", "0%"),
    ("2.54cm", "2.54cm"),
    ("1Q", "1q"),
    ("1EM", "1em"),
    ("calc(1px + 2px)", "calc(3px)"),
    ("calc(1px + 1em)", "calc(1em + 1px)"),
    ("calc(1in + 2px)", "calc(98px)"),
    ("calc(-20px + 40%)", "calc(40% - 20px)"),
    ("min(1em, 2px)", "min(1em, 2px)"),
];

#[test]
fn accepted_decoration_controls_execute_without_metadata_or_new_roles() {
    for (name, input, expected) in [
        (
            "text-decoration-line",
            "blink overline underline line-through",
            "underline overline line-through blink",
        ),
        ("text-decoration-line", "none", "none"),
        ("text-decoration-style", "DOUBLE", "double"),
        ("text-decoration-thickness", "FROM-FONT", "from-font"),
        (
            "text-decoration",
            "red wavy -2px overline underline",
            "underline overline -2px wavy red",
        ),
        (
            "text-decoration",
            "none auto solid currentcolor",
            "none auto solid currentcolor",
        ),
        ("text-decoration", "auto", "auto"),
        ("text-decoration", "currentcolor", "currentcolor"),
    ] {
        canonical(name, input, expected);
    }
    for &(input, expected) in SIGNED_LENGTH_PERCENTAGE_GOLDENS {
        canonical("text-decoration-thickness", input, expected);
    }
}

#[test]
fn accepted_color_provider_controls_execute_every_pair_before_new_emphasis() {
    for &(input, expected) in COLOR_GOLDENS {
        canonical("text-decoration-color", input, expected);
    }
}

#[test]
fn accepted_shadow_provider_keeps_signed_box_spread_and_authored_omissions() {
    for (input, expected) in [
        ("none", "none"),
        ("1px 2px", "1px 2px"),
        ("inset red -1px 2px 3px -4px", "red -1px 2px 3px -4px inset"),
        (
            "1px 2px 0px 0px currentcolor",
            "currentcolor 1px 2px 0px 0px",
        ),
        ("calc(1px + 1em) 2px", "calc(1em + 1px) 2px"),
        (
            "red 1px 2px, blue 3px 4px inset",
            "red 1px 2px, blue 3px 4px inset",
        ),
    ] {
        canonical("box-shadow", input, expected);
    }
    for bad in [
        "1px",
        "1px 2px -1px",
        "1px 2px red 3px",
        "inset inset 1px 2px",
    ] {
        invalid("box-shadow", bad);
    }
}

#[test]
fn accepted_decoration_projection_and_cumulative_prices_are_independent_controls() {
    for (name, input, expected, input_nodes, projection_nodes) in [
        (
            "text-decoration",
            "underline overline -1px wavy currentcolor",
            "underline overline -1px wavy currentcolor",
            9,
            9,
        ),
        (
            "text-decoration-thickness",
            "calc(1px + 2px)",
            "calc(3px)",
            6,
            5,
        ),
        (
            "text-decoration",
            "none auto solid currentcolor",
            "none auto solid currentcolor",
            8,
            8,
        ),
    ] {
        let source = checked(name, input, true);
        let before = source.clone();
        let output = format!("{name}: {expected} !important;");
        let exact =
            CssSpecifiedValueSerializationLimits::new(input_nodes, projection_nodes, output.len());
        assert_eq!(source.to_specified_css_with_limits(exact).unwrap(), output);
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(
                    input_nodes - 1,
                    projection_nodes,
                    output.len(),
                ),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(
                    input_nodes,
                    projection_nodes - 1,
                    output.len(),
                ),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(
                    input_nodes,
                    projection_nodes,
                    output.len() - 1,
                ),
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
        assert_eq!(source.to_specified_css_with_limits(exact).unwrap(), output);
    }
}

#[test]
fn line_exclusive_error_alternatives_and_all_ordinary_subsets() {
    let words = ["underline", "overline", "line-through", "blink"];
    for mask in 1..16 {
        let expected = words
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, w)| *w)
            .collect::<Vec<_>>()
            .join(" ");
        let input = words
            .iter()
            .enumerate()
            .rev()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, w)| *w)
            .collect::<Vec<_>>()
            .join(" ");
        canonical("text-decoration-line", &input, &expected);
    }
    for line in ["spelling-error", "grammar-error"] {
        canonical("text-decoration-line", line, line);
        canonical(
            "text-decoration",
            &format!("red wavy -2px {line}"),
            &format!("{line} -2px wavy red"),
        );
    }
    for bad in [
        "none underline",
        "underline underline",
        "spelling-error grammar-error",
        "spelling-error underline",
        "none spelling-error",
        "grammar-error grammar-error",
        "bogus",
    ] {
        invalid("text-decoration-line", bad);
        invalid("text-decoration", bad);
    }
}

#[test]
fn decoration_unordered_roles_reject_duplicates_and_preserve_signed_math() {
    for style in ["solid", "double", "dotted", "dashed", "wavy"] {
        canonical("text-decoration-style", style, style);
        canonical(
            "text-decoration",
            &format!("red {style} calc(1px + 1em) underline"),
            &format!("underline calc(1em + 1px) {style} red"),
        );
    }
    for &(input, expected) in SIGNED_LENGTH_PERCENTAGE_GOLDENS {
        canonical(
            "text-decoration",
            &format!("wavy {input} red"),
            &format!("{expected} wavy red"),
        );
    }
    for bad in [
        "solid wavy",
        "red blue",
        "auto from-font",
        "1px 2px",
        "none none",
        "underline bogus",
        "initial red",
        "calc(1s + 1s)",
        "calc(1px + 1)",
    ] {
        invalid("text-decoration", bad);
    }
    for bad in ["1", "1s", "1deg", "calc(1s)", "auto auto", "from-font 1px"] {
        invalid("text-decoration-thickness", bad);
    }
}

#[test]
fn underline_position_alternatives_and_unordered_sides_are_precise() {
    for (input, expected) in [
        ("auto", "auto"),
        ("under", "under"),
        ("from-font", "from-font"),
        ("left", "left"),
        ("right", "right"),
        ("left under", "under left"),
        ("right under", "under right"),
        ("left from-font", "from-font left"),
        ("right from-font", "from-font right"),
    ] {
        canonical("text-underline-position", input, expected);
    }
    for bad in [
        "auto left",
        "auto under",
        "under from-font",
        "left right",
        "under under",
        "left left",
        "over",
        "none",
    ] {
        invalid("text-underline-position", bad);
    }
}

#[test]
fn underline_offset_imports_signed_length_percentage_without_font_resolution() {
    canonical("text-underline-offset", "auto", "auto");
    for &(input, expected) in SIGNED_LENGTH_PERCENTAGE_GOLDENS {
        canonical("text-underline-offset", input, expected);
    }
    for bad in [
        "from-font",
        "none",
        "1",
        "1s",
        "1deg",
        "calc(1s)",
        "1px 2px",
        "calc(1px + 1)",
    ] {
        invalid("text-underline-offset", bad);
    }
}

#[test]
fn skip_current_keywords_and_nonempty_boundary_space_subset() {
    for (name, words) in [
        ("text-decoration-skip", &["none", "auto"][..]),
        ("text-decoration-skip-self", &["none", "objects"][..]),
        ("text-decoration-skip-box", &["none", "all"][..]),
        ("text-decoration-skip-inset", &["none", "auto"][..]),
        ("text-decoration-skip-ink", &["auto", "none", "all"][..]),
        (
            "text-decoration-skip-spaces",
            &["none", "all", "start", "end", "start end"][..],
        ),
    ] {
        for word in words {
            canonical(name, word, word);
        }
        for bad in ["bogus", "none auto", "inherit none"] {
            invalid(name, bad);
        }
    }
    canonical("text-decoration-skip-spaces", "end start", "start end");
    for bad in [
        "all start",
        "none end",
        "start start",
        "end end",
        "always",
        "objects",
    ] {
        invalid("text-decoration-skip-spaces", bad);
    }
    for (name, bad) in [
        ("text-decoration-skip", "objects"),
        ("text-decoration-skip", "ink"),
        ("text-decoration-skip", "auto none"),
        ("text-decoration-skip-self", "all"),
        ("text-decoration-skip-box", "objects"),
        ("text-decoration-skip-inset", "all"),
        ("text-decoration-skip-ink", "always"),
        ("text-decoration-skip-ink", "objects"),
    ] {
        invalid(name, bad);
    }
}

#[test]
fn emphasis_style_all_shapes_fills_and_multicharacter_strings_remain_authored() {
    for word in ["none", "filled", "open"] {
        canonical("text-emphasis-style", word, word);
    }
    for shape in ["dot", "circle", "double-circle", "triangle", "sesame"] {
        canonical("text-emphasis-style", shape, shape);
        canonical(
            "text-emphasis-style",
            &format!("{shape} open"),
            &format!("open {shape}"),
        );
        // CSSOM meaning-preserving default omission, NOT contextual shape synthesis.
        canonical("text-emphasis-style", &format!("filled {shape}"), shape);
    }
    for (input, expected) in [
        ("'x'", "\"x\""),
        ("'hello'", "\"hello\""),
        ("''", "\"\""),
        ("'😀x'", "\"😀x\""),
        ("'á'", "\"á\""),
    ] {
        canonical("text-emphasis-style", input, expected);
    }
    for bad in [
        "none filled",
        "filled open",
        "dot circle",
        "dot dot",
        "open open",
        "'a' 'b'",
        "'a' dot",
        "red",
    ] {
        invalid("text-emphasis-style", bad);
    }
}

#[test]
fn emphasis_color_and_shorthand_import_shared_color_without_contextual_resolution() {
    for &(input, expected) in COLOR_GOLDENS {
        canonical("text-emphasis-color", input, expected);
        canonical(
            "text-emphasis",
            &format!("open triangle {input}"),
            &format!("open triangle {expected}"),
        );
    }
    for (input, expected) in [
        ("red triangle open", "open triangle red"),
        ("'hello' blue", "\"hello\" blue"),
        ("red", "red"),
        ("open", "open"),
    ] {
        canonical("text-emphasis", input, expected);
    }
    for bad in [
        "red blue",
        "open filled",
        "none triangle",
        "red 'x' circle",
        "over red",
        "symbols red",
    ] {
        invalid("text-emphasis", bad);
    }
}

#[test]
fn emphasis_position_requires_vertical_and_defaults_only_horizontal_right() {
    for (input, expected) in [
        ("over", "over"),
        ("under", "under"),
        ("left over", "over left"),
        ("left under", "under left"),
        ("right over", "over"),
        ("under right", "under"),
    ] {
        canonical("text-emphasis-position", input, expected);
    }
    for bad in [
        "right",
        "left",
        "over under",
        "right left over",
        "over over",
        "auto",
        "none",
    ] {
        invalid("text-emphasis-position", bad);
    }
}

#[test]
fn emphasis_skip_nonempty_unique_subsets_follow_grammar_order() {
    let words = ["spaces", "punctuation", "symbols", "narrow"];
    for mask in 1..16 {
        let expected = words
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, w)| *w)
            .collect::<Vec<_>>()
            .join(" ");
        let input = words
            .iter()
            .enumerate()
            .rev()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, w)| *w)
            .collect::<Vec<_>>()
            .join(" ");
        canonical("text-emphasis-skip", &input, &expected);
    }
    for bad in [
        "none",
        "auto",
        "spaces spaces",
        "punctuation punctuation",
        "narrow narrow",
        "symbols symbols",
        "all",
        "spaces bogus",
    ] {
        invalid("text-emphasis-skip", bad);
    }
}

#[test]
fn level4_text_shadow_has_inset_and_nonnegative_spread_without_computation() {
    for (input, expected) in [
        ("none", "none"),
        ("1px 2px", "1px 2px"),
        ("red -1px 2px", "red -1px 2px"),
        ("1px 2px currentcolor", "currentcolor 1px 2px"),
        ("inset blue 1px 2px 3px 4px", "blue 1px 2px 3px 4px inset"),
        (
            "1px 2px 0px 0px currentcolor inset",
            "currentcolor 1px 2px 0px 0px inset",
        ),
        ("red calc(1px + 1em) 2px", "red calc(1em + 1px) 2px"),
        (
            "1px 2px 3px 4px, inset blue -1px -2px",
            "1px 2px 3px 4px, blue -1px -2px inset",
        ),
        (
            "light-dark(red,blue) 1px 2px",
            "light-dark(red, blue) 1px 2px",
        ),
        (
            "1px 2px calc(1px + 2px) calc(2px + 3px)",
            "1px 2px calc(3px) calc(5px)",
        ),
    ] {
        canonical("text-shadow", input, expected);
    }
    for &(input, expected) in COLOR_GOLDENS {
        canonical(
            "text-shadow",
            &format!("{input} 1px 2px"),
            &format!("{expected} 1px 2px"),
        );
    }
    // Values math range checks are deferred; ordinary exact negatives above
    // have a different admission policy from a retained function-root value.
    canonical(
        "text-shadow",
        "1px 2px calc(-1px) calc(-2px)",
        "1px 2px calc(-1px) calc(-2px)",
    );
}

#[test]
fn text_shadow_rejects_negative_ordinary_blur_spread_and_whole_invalid_lists() {
    for bad in [
        "red",
        "1px",
        "none, 1px 2px",
        "1px 2px,",
        ",1px 2px",
        "1px 2px -1px",
        "1px 2px 0px -1px",
        "1px 2px 0px -1e-999px",
        "1px 2px -1e-999px",
        "1px 2px 3px 4px 5px",
        "1px 2px red 3px",
        "red blue 1px 2px",
        "inset inset 1px 2px",
        "1% 2px",
        "1px 2s",
        "calc(1s) 2px",
        "red 1px 2px, blue 1px",
        "initial 1px 2px",
    ] {
        invalid("text-shadow", bad);
    }
}

#[test]
fn each_shorthand_sets_only_selected_members_and_resets_omissions() {
    reset_values(
        "text-decoration",
        "red",
        &[
            ("text-decoration-line", "none"),
            ("text-decoration-thickness", "auto"),
            ("text-decoration-style", "solid"),
            ("text-decoration-color", "red"),
        ],
    );
    reset_values(
        "text-decoration",
        "spelling-error -2px",
        &[
            ("text-decoration-line", "spelling-error"),
            ("text-decoration-thickness", "-2px"),
            ("text-decoration-style", "solid"),
            ("text-decoration-color", "currentcolor"),
        ],
    );
    reset_values(
        "text-decoration-skip",
        "auto",
        &[
            ("text-decoration-skip-self", "objects"),
            ("text-decoration-skip-box", "none"),
            ("text-decoration-skip-inset", "none"),
            ("text-decoration-skip-spaces", "start end"),
            ("text-decoration-skip-ink", "auto"),
        ],
    );
    reset_values(
        "text-decoration-skip",
        "none",
        &[
            ("text-decoration-skip-self", "none"),
            ("text-decoration-skip-box", "none"),
            ("text-decoration-skip-inset", "none"),
            ("text-decoration-skip-spaces", "none"),
            ("text-decoration-skip-ink", "none"),
        ],
    );
    reset_values(
        "text-emphasis",
        "red",
        &[
            ("text-emphasis-style", "none"),
            ("text-emphasis-color", "red"),
        ],
    );
    reset_values(
        "text-emphasis",
        "open",
        &[
            ("text-emphasis-style", "open"),
            ("text-emphasis-color", "currentcolor"),
        ],
    );
}

macro_rules! lifecycle {
    ($global:ident,$pending:ident,$closure:ident,[$(($name:literal,$value:literal,[$($terminal:literal),+])),+ $(,)?]) => {
        #[test] fn $global(){ $(globals_and_all($name,&[$($terminal),+]));+ }
        #[test] fn $pending(){ $(pending($name,$value,&[$($terminal),+]));+ }
        #[test] fn $closure(){ $(closure($name,$value));+ }
    };
}
lifecycle!(
    decoration_globals,
    decoration_pending_retry,
    decoration_original_closure,
    [
        (
            "text-decoration",
            "underline -2px wavy red",
            [
                "text-decoration-line",
                "text-decoration-thickness",
                "text-decoration-style",
                "text-decoration-color"
            ]
        ),
        (
            "text-decoration-line",
            "spelling-error",
            ["text-decoration-line"]
        ),
        (
            "text-decoration-color",
            "light-dark(red,blue)",
            ["text-decoration-color"]
        ),
        ("text-decoration-style", "wavy", ["text-decoration-style"]),
        (
            "text-decoration-thickness",
            "calc(-20px + 40%)",
            ["text-decoration-thickness"]
        ),
    ]
);
lifecycle!(
    underline_globals,
    underline_pending_retry,
    underline_original_closure,
    [
        (
            "text-underline-position",
            "under left",
            ["text-underline-position"]
        ),
        (
            "text-underline-offset",
            "calc(-20px + 40%)",
            ["text-underline-offset"]
        ),
    ]
);
lifecycle!(
    skip_globals,
    skip_pending_retry,
    skip_original_closure,
    [
        (
            "text-decoration-skip",
            "auto",
            [
                "text-decoration-skip-self",
                "text-decoration-skip-box",
                "text-decoration-skip-inset",
                "text-decoration-skip-spaces",
                "text-decoration-skip-ink"
            ]
        ),
        (
            "text-decoration-skip-self",
            "objects",
            ["text-decoration-skip-self"]
        ),
        (
            "text-decoration-skip-box",
            "all",
            ["text-decoration-skip-box"]
        ),
        (
            "text-decoration-skip-inset",
            "auto",
            ["text-decoration-skip-inset"]
        ),
        (
            "text-decoration-skip-spaces",
            "end start",
            ["text-decoration-skip-spaces"]
        ),
        (
            "text-decoration-skip-ink",
            "all",
            ["text-decoration-skip-ink"]
        ),
    ]
);
lifecycle!(
    emphasis_globals,
    emphasis_pending_retry,
    emphasis_original_closure,
    [
        (
            "text-emphasis",
            "open triangle red",
            ["text-emphasis-style", "text-emphasis-color"]
        ),
        ("text-emphasis-style", "'hello'", ["text-emphasis-style"]),
        (
            "text-emphasis-color",
            "light-dark(red,blue)",
            ["text-emphasis-color"]
        ),
        (
            "text-emphasis-position",
            "under left",
            ["text-emphasis-position"]
        ),
        (
            "text-emphasis-skip",
            "narrow symbols spaces",
            ["text-emphasis-skip"]
        ),
    ]
);
lifecycle!(
    shadow_globals,
    shadow_pending_retry,
    shadow_original_closure,
    [(
        "text-shadow",
        "red 1px 2px 3px 4px inset, blue -1px -2px",
        ["text-shadow"]
    ),]
);

#[test]
fn composed_decoration_normalization_keeps_occurrences_importance_and_atomic_budget() {
    let css = ".a{text-decoration:red!important;text-decoration-line:spelling-error;text-decoration-color:light-dark(red,blue);text-decoration-style:wavy;text-decoration-thickness:-27%;text-underline-position:under left;text-underline-offset:-2px;text-decoration-skip:auto;text-decoration-skip-self:objects;text-decoration-skip-box:all;text-decoration-skip-inset:auto;text-decoration-skip-spaces:end start;text-decoration-skip-ink:all;text-emphasis:open triangle blue;text-emphasis-style:'hello';text-emphasis-color:currentcolor;text-emphasis-position:under left;text-emphasis-skip:narrow spaces;text-shadow:red 1px 2px 3px 4px inset}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(0, 1, 19, 27).unwrap();
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
    let counts = [4, 1, 1, 1, 1, 1, 1, 5, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1];
    assert_eq!(items.len(), counts.len());
    for (index, (item, count)) in items.iter().zip(counts).enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().importance(),
            if index == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert_eq!(item.source().parsed_value().unwrap().source().as_str(), css);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("ordinary complete terminals")
        };
        assert_eq!(values.items().len(), count);
        assert!(
            values
                .items()
                .iter()
                .all(|v| v.source().same_occurrence(item.source()))
        );
    }
    let error = normalize_report_with_limits(
        &report,
        CssNormalizationLimits::try_new(0, 1, 19, 26).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 26
        }
    );
    assert_eq!(error.declaration_order(), Some(18));
    assert_eq!(report, before);
    assert!(normalize_report_with_limits(&report, exact).is_ok());
}

#[test]
fn recovered_original_closures_stay_retained_and_unclean_after_normalization() {
    for (name, ordinary) in [
        ("text-decoration", "red"),
        ("text-decoration-line", "spelling-error"),
        ("text-decoration-color", "red"),
        ("text-decoration-style", "wavy"),
        ("text-decoration-thickness", "-2px"),
        ("text-underline-position", "under left"),
        ("text-underline-offset", "-27%"),
        ("text-decoration-skip", "auto"),
        ("text-decoration-skip-self", "objects"),
        ("text-decoration-skip-box", "all"),
        ("text-decoration-skip-inset", "auto"),
        ("text-decoration-skip-spaces", "end start"),
        ("text-decoration-skip-ink", "all"),
        ("text-emphasis", "open red"),
        ("text-emphasis-style", "'hello'"),
        ("text-emphasis-color", "red"),
        ("text-emphasis-position", "under left"),
        ("text-emphasis-skip", "symbols"),
        ("text-shadow", "red 1px 2px 3px 4px inset"),
    ] {
        let css = format!("{name}:{ordinary}/*");
        let report = parse_style_attribute(&css);
        assert!(!report.is_clean());
        assert_eq!(report.syntax().len(), 1, "recovered {css}");
        let source = &report.syntax()[0];
        assert_eq!(source.known().unwrap().property(), property(name));
        assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        let sheet_source = format!(".a{{{name}:{ordinary}/*");
        let sheet_report = parse_sheet(&sheet_source);
        assert!(!sheet_report.is_clean());
        assert_eq!(
            validate_sheet(&sheet_source).unwrap_err().diagnostics(),
            sheet_report.diagnostics()
        );
        let normalized = normalize_report_with_limits(
            &sheet_report,
            CssNormalizationLimits::try_new(0, 1, 1, 5).unwrap(),
        )
        .unwrap();
        assert_eq!(normalized.diagnostics(), sheet_report.diagnostics());
    }
}

#[test]
fn checked_programmatic_inputs_and_zero_resource_failures_leave_supplied_graphs_unchanged() {
    for (name, words) in [
        ("text-decoration-line", &["spelling-error"][..]),
        ("text-underline-position", &["under", "left"][..]),
        ("text-decoration-skip-spaces", &["end", "start"][..]),
        ("text-emphasis-style", &["open", "triangle"][..]),
        ("text-emphasis-position", &["under", "left"][..]),
        ("text-emphasis-skip", &["narrow", "symbols"][..]),
    ] {
        let components = CssComponentValues::try_new(
            words
                .iter()
                .map(|v| CssComponentValue::try_token(v).unwrap())
                .collect(),
        )
        .unwrap();
        let before = components.clone();
        assert!(
            components
                .items()
                .iter()
                .all(|v| v.origin() == &CssValueOrigin::Programmatic)
        );
        for grammar in [false, true] {
            let source = checked_components(name, components.clone(), grammar).unwrap();
            assert_eq!(source.value_components(), &components);
            let snapshot = source.clone();
            for (limits, kind) in [
                (
                    CssSpecifiedValueSerializationLimits::new(0, 262_144, 1_048_576),
                    CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
                ),
                (
                    CssSpecifiedValueSerializationLimits::new(65_536, 0, 1_048_576),
                    CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
                ),
                (
                    CssSpecifiedValueSerializationLimits::new(65_536, 262_144, 0),
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
                assert_eq!(source, snapshot);
            }
        }
        assert_eq!(components, before);
    }
}

#[test]
fn empty_ordinary_values_reject_atomically_through_all_existing_fronts() {
    // Complete-family grammar stimuli, not metadata/catalog/wrapper inventory.
    for name in [
        "text-decoration",
        "text-decoration-line",
        "text-decoration-color",
        "text-decoration-style",
        "text-decoration-thickness",
        "text-underline-position",
        "text-underline-offset",
        "text-decoration-skip",
        "text-decoration-skip-self",
        "text-decoration-skip-box",
        "text-decoration-skip-inset",
        "text-decoration-skip-spaces",
        "text-decoration-skip-ink",
        "text-emphasis",
        "text-emphasis-style",
        "text-emphasis-color",
        "text-emphasis-position",
        "text-emphasis-skip",
        "text-shadow",
    ] {
        let p = property(name);
        let components = parse_component_values("").unwrap();
        let before = components.clone();
        for grammar in [false, true] {
            assert!(matches!(
                checked_components(name, components.clone(), grammar)
                    .unwrap_err()
                    .kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
        }
        for report in [
            parse_property_value_text("", CssPropertyNameRef::Known(p), CssImportance::Normal),
            parse_property_value_text_for_grammar("", p.grammar(), CssImportance::Normal),
        ] {
            assert!(report.syntax().is_none());
            assert!(!report.is_clean());
        }
        let css = format!("color:red;{name}:;color:blue");
        let report = parse_style_attribute(&css);
        assert_eq!(report.syntax().len(), 2);
        assert!(
            report
                .syntax()
                .iter()
                .all(|d| d.known().unwrap().property() == CssKnownProperty::Color)
        );
        assert!(!report.is_clean());
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        assert_eq!(components, before);
    }
}
