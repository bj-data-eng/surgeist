#![forbid(unsafe_code)]
//! Public authored Color lifecycle and standards-mode admission contract.
//! Color4 CRD20260908 #propdef-color; full accepted Color/Color5/profile/device
//! owners; Syntax3 original EOF boundaries; Cascade5/Variables1; CSSOM.
//! Context-specific quirky hex APIs are intentionally not fabricated here.
use surgeist_css::*;

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
fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name).unwrap_or_else(|| panic!("missing known property: {name}"))
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

fn color(source: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("color payload")
    };
    value.value()
}
fn strict_error(error: &CssPropertyValueParseError, original: &str) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    let CssSerializedOrigin::End(Some(CssValueOrigin::ImplicitClosure { opening, at })) =
        error.origin()
    else {
        panic!("strict failure must retain the original EOF closure: {error:?}")
    };
    assert_eq!(opening.source().as_str(), original);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), original.len());
    assert_eq!(at.span().end().byte_offset().value(), original.len());
}
fn rejects_original(text: &str) {
    let values = parse_component_values(text).unwrap();
    let before = values.clone();
    for grammar in [false, true] {
        for _ in 0..2 {
            strict_error(
                &checked_components("color", values.clone(), grammar).unwrap_err(),
                text,
            );
            assert_eq!(values, before);
        }
    }
}
#[test]
fn color_is_an_inherited_terminal_with_intrinsic_canvastext_initial() {
    let p = property("COLOR");
    assert_eq!(p, CssKnownProperty::Color);
    assert_eq!(p.canonical_name(), "color");
    let CssPropertyKindRef::Longhand(metadata) = p.metadata().unwrap().kind() else {
        panic!("longhand")
    };
    assert!(metadata.inherited_by_default());
    assert_eq!(metadata.property().known_property(), p);
    let initial = metadata.initial_value();
    assert_eq!(initial.property().known_property(), p);
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("intrinsic initial")
    };
    let values =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("CanvasText").unwrap()])
            .unwrap();
    let source = checked_components("color", values, true).unwrap();
    assert_eq!(
        color(&source),
        &CssColor::from_system(CssSystemColor::CanvasText)
    );
    assert_eq!(
        value,
        completed(&source).items()[0].ordinary_value().unwrap()
    );
    canonical("color", "CanvasText", "canvastext");
}
#[test]
fn complete_accepted_color_families_use_all_five_declaration_fronts() {
    for &(input, expected) in COLOR_GOLDENS {
        canonical("color", input, expected);
    }
}
#[test]
fn color_terminal_contributions_keep_both_importance_states_and_original_occurrences() {
    for text in [
        "red",
        "currentcolor",
        "color(--P 0 0.7 0.2 0)",
        "rgb(from red r g b / alpha)",
    ] {
        for importance in [CssImportance::Normal, CssImportance::Important] {
            let source = parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Color),
                parse_component_values(text).unwrap(),
                importance,
            )
            .unwrap();
            let before = source.clone();
            let values = completed(&source);
            let item = &values.items()[0];
            assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
            assert_eq!(
                item.ordinary_value().unwrap().property().known_property(),
                CssKnownProperty::Color
            );
            assert!(item.replacement_components().is_none());
            assert_eq!(color(item.source()), color(&source));
            assert_eq!(source, before);
        }
    }
}
#[test]
fn global_keywords_and_universal_reset_include_color_without_contextual_resolution() {
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for source in fronts("color", text) {
            let values = completed(&source);
            assert_eq!(
                values.items()[0].value(),
                CssContributionValueRef::Global(keyword)
            );
            assert!(values.items()[0].ordinary_value().is_none());
            assert!(values.items()[0].replacement_components().is_none());
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("color: {text} !important;")
            );
        }
        let source = checked("all", text, true);
        let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("universal reset")
        };
        assert_eq!(reset.keyword(), keyword);
        assert!(reset.source().same_occurrence(&source));
        assert!(!reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::Color)));
    }
}
#[test]
fn exact_color_channels_and_intrinsic_alpha_remain_separate_from_specified_projection() {
    for source in fronts("color", "rgb(1e100 1e-47 0.1 / 150%)") {
        let rgb = color(&source).rgb_value().unwrap();
        for (channel, expected) in rgb.channels().iter().zip(["1e100", "1e-47", "0.1"]) {
            let CssColorComponent::Number(value) = channel else {
                panic!("exact number")
            };
            assert_eq!(value.numeric().representation(), expected);
            assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
        }
        let CssColorComponent::Percentage(alpha) = rgb.alpha().unwrap() else {
            panic!("authored alpha")
        };
        assert_eq!(alpha.numeric().representation(), "150");
        let CssParsedColorAlphaRef::Scalar(alpha) = rgb.parsed_alpha() else {
            panic!("intrinsic alpha")
        };
        assert!(alpha.is_one());
        assert!(!alpha.is_zero());
    }
}
#[test]
fn mixed_programmatic_and_parsed_channels_keep_their_actual_origins() {
    let mut children = vec![CssComponentValue::try_number("0.1").unwrap()];
    children.extend(
        parse_component_values("1e100 0")
            .unwrap()
            .items()
            .iter()
            .cloned(),
    );
    let values = CssComponentValues::try_new(vec![
        CssComponentValue::try_function("rgb", CssComponentValues::try_new(children).unwrap())
            .unwrap(),
    ])
    .unwrap();
    let before = values.clone();
    for grammar in [false, true] {
        let source = checked_components("color", values.clone(), grammar).unwrap();
        assert_eq!(source.value_components(), &values);
        assert!(source.position().is_none());
        assert!(source.parsed_value().is_none());
        let rgb = color(&source).rgb_value().unwrap();
        let CssColorComponent::Number(first) = &rgb.channels()[0] else {
            panic!("first number")
        };
        assert_eq!(first.origin(), &CssValueOrigin::Programmatic);
        let CssColorComponent::Number(second) = &rgb.channels()[1] else {
            panic!("second number")
        };
        let CssValueOrigin::Parsed(origin) = second.origin() else {
            panic!("original parsed origin")
        };
        assert_eq!(origin.source().as_str(), "1e100 0");
        assert_eq!(origin.span().start().byte_offset().value(), 0);
        assert_eq!(
            source.to_specified_css().unwrap(),
            "color: rgb(0.1, 255, 0) !important;"
        );
    }
    assert_eq!(values, before);
}
#[test]
fn invalid_color_grammar_drops_only_its_declaration_and_retains_original_coordinates() {
    for (text, code) in [
        ("red blue", CssErrorCode::InvalidPropertyValue),
        ("10px", CssErrorCode::InvalidColorSyntax),
        ("rgb(1 2)", CssErrorCode::InvalidColorSyntax),
    ] {
        let prefix = "/*😀*/ ";
        let target = format!("color:{text};");
        let css = format!("{prefix}{target}color:blue");
        let report = parse_style_attribute(&css);
        let [sibling] = report.syntax().as_slice() else {
            panic!("valid sibling")
        };
        assert_eq!(color(sibling).to_specified_css().unwrap(), "blue");
        let [diagnostic] = report.diagnostics() else {
            panic!("one diagnostic")
        };
        assert_eq!(diagnostic.error().code(), code);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            prefix.len()
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            prefix.len() + target.len()
        );
        assert_eq!(
            diagnostic.span().start().column().value() as usize,
            prefix.encode_utf16().count()
        );
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        let values = parse_component_values(text).unwrap();
        for grammar in [false, true] {
            assert!(matches!(
                checked_components("color", values.clone(), grammar)
                    .unwrap_err()
                    .kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ));
        }
        for report in [
            parse_property_value_text(
                text,
                CssPropertyNameRef::Known(CssKnownProperty::Color),
                CssImportance::Normal,
            ),
            parse_property_value_text_for_grammar(
                text,
                CssKnownProperty::Color.grammar(),
                CssImportance::Normal,
            ),
        ] {
            assert!(!report.is_clean());
            assert!(report.syntax().is_none());
        }
    }
}
#[test]
fn complete_pending_values_reenter_repeatedly_with_original_source_and_replacement_components() {
    for text in ["var(--paint)", "env(paint)", "attr(data-paint *)"] {
        for source in fronts("color", text) {
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            assert!(handle.source().same_occurrence(&source));
            for invalid in [
                "",
                "red blue",
                "10px",
                "rgb(1 2)",
                "initial!important",
                "initial;color:red",
            ] {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(invalid).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ));
            }
            for residual in ["var(--again)", "env(again)", "attr(data-again *)"] {
                assert_eq!(
                    handle
                        .reenter(parse_component_values(residual).unwrap())
                        .unwrap_err()
                        .kind(),
                    &CssExpansionErrorKind::ResidualSubstitution
                );
            }
            let programmatic = CssComponentValues::try_new(vec![
                CssComponentValue::try_ident("currentcolor").unwrap(),
            ])
            .unwrap();
            for replacement in [
                parse_component_values("red").unwrap(),
                programmatic,
                parse_component_values("unset").unwrap(),
            ] {
                for _ in 0..2 {
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("longhand replacement")
                    };
                    let [item] = values.items() else {
                        panic!("one contribution")
                    };
                    assert_eq!(item.property(), CssKnownProperty::Color);
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert_eq!(item.replacement_components(), Some(&replacement));
                    if replacement.serialize().unwrap().as_css() == "unset" {
                        assert_eq!(
                            item.value(),
                            CssContributionValueRef::Global(CssGlobalKeyword::Unset)
                        );
                    } else {
                        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
                    }
                }
            }
            assert_eq!(source, before);
        }
    }
}
// These existing-callable groups are expected to expose the published omission
// in requires_closed_components. They are not qualified RED until root executes.
#[test]
fn checked_ordinary_color_rejects_original_comment_and_function_eof_closures() {
    for text in [
        "red/*",
        "rgb(1 2 3",
        "light-dark(red,blue",
        "color-mix(in srgb,red,blue",
        "light-dark(red,rgb(1 2 3",
    ] {
        rejects_original(text);
    }
}
#[test]
fn checked_global_color_rejects_original_comment_eof_closure() {
    for text in [
        "initial/*",
        "inherit/*",
        "unset/*",
        "revert/*",
        "revert-layer/*",
    ] {
        rejects_original(text);
    }
}
#[test]
fn checked_pending_color_rejects_original_substitution_eof_closure() {
    for text in ["var(--paint", "env(paint", "attr(data-paint *"] {
        rejects_original(text);
    }
}
#[test]
fn pending_reentry_rejects_recovered_ordinary_replacement_then_accepts_complete_retry() {
    for original in ["red/*", "rgb(1 2 3", "light-dark(red,blue"] {
        let source = checked("color", "var(--paint)", true);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let replacement = parse_component_values(original).unwrap();
        let before = replacement.clone();
        let error = handle.reenter(replacement.clone()).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(detail) = error.kind() else {
            panic!("invalid replacement")
        };
        strict_error(detail, original);
        assert_eq!(replacement, before);
        assert!(
            handle
                .reenter(parse_component_values("red/**/").unwrap())
                .is_ok()
        );
        assert!(handle.source().same_occurrence(&source));
    }
}
#[test]
fn pending_reentry_rejects_recovered_global_replacement_with_original_eof_origin() {
    for original in [
        "initial/*",
        "inherit/*",
        "unset/*",
        "revert/*",
        "revert-layer/*",
    ] {
        let source = checked("color", "var(--paint)", true);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let error = handle
            .reenter(parse_component_values(original).unwrap())
            .unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(detail) = error.kind() else {
            panic!("invalid replacement")
        };
        strict_error(detail, original);
        assert!(
            handle
                .reenter(parse_component_values("unset/**/").unwrap())
                .is_ok()
        );
    }
}
#[test]
fn browser_recovery_retains_authored_color_occurrences_and_reports_unclean_input() {
    for text in ["red/*", "rgb(1 2 3", "initial/*", "var(--paint"] {
        let css = format!("color:{text}");
        let report = parse_style_attribute(&css);
        assert!(!report.is_clean());
        let [source] = report.syntax().as_slice() else {
            panic!("browser recovery retains one occurrence")
        };
        assert_eq!(source.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(source.parsed_value().unwrap().source().as_str(), css);
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}
#[test]
fn nested_color_occurrences_normalize_in_original_order_with_atomic_cumulative_limits() {
    let css = ".a{color:red!important;.b{color:currentcolor}color:CanvasText;color:var(--paint)}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    // Two style rules plus the ordered outer declaration segment after nesting.
    let exact = CssNormalizationLimits::try_new(1, 3, 4, 4).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 4);
    for (order, value) in values.iter().enumerate() {
        assert_eq!(value.order(), order);
        assert_eq!(
            value.source().known().unwrap().property(),
            CssKnownProperty::Color
        );
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
            .same_context(values[2].selector_context())
    );
    assert!(
        !values[0]
            .selector_context()
            .same_context(values[1].selector_context())
    );
    assert!(matches!(values[3].expansion(), CssExpansion::Pending(_)));
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(1, 3, 3, 4).unwrap(),
            CssNormalizationResource::Declarations,
        ),
        (
            CssNormalizationLimits::try_new(1, 3, 4, 3).unwrap(),
            CssNormalizationResource::Contributions,
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit: 3 }
        );
        assert_eq!(error.declaration_order(), Some(3));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(values[3].source())
        );
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
#[test]
fn named_color_declaration_has_exact_three_input_three_projection_prices_and_atomic_failures() {
    for text in ["red", "currentcolor", "canvastext"] {
        let source = checked("color", text, true);
        let before = source.clone();
        let expected = format!("color: {text} !important;");
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
fn sheet_siblings_share_final_color_bytes_and_fail_atomically_at_the_later_rule() {
    let report = parse_sheet(".a{color:red}.b{color:currentcolor}");
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { color: red; }\n.b { color: currentcolor; }";
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
fn standalone_color_provider_keeps_its_one_node_budget_and_capacity_failure() {
    let source = checked("color", "purple", true);
    let value = color(&source);
    let before = value.clone();
    assert_eq!(
        value
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap(),
        "purple"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 6),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 6),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 5),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, &before);
    }
    let source = checked(
        "color",
        "hsl(0 12345678901234567891e170141183460469231731687303715884105727% 50%)",
        true,
    );
    let before = source.clone();
    assert_eq!(
        color(&source).to_specified_css().unwrap_err().kind(),
        CssSpecifiedValueSerializationErrorKind::CapacityOverflow
    );
    assert_eq!(source, before);
}
#[test]
fn ordinary_mode_keeps_all_seven_quirky_color_properties_on_normal_color_grammar() {
    for name in [
        "color",
        "background-color",
        "border-color",
        "border-top-color",
        "border-right-color",
        "border-bottom-color",
        "border-left-color",
    ] {
        for text in ["ABC", "123", "12abc"] {
            let values = parse_component_values(text).unwrap();
            for grammar in [false, true] {
                assert!(checked_components(name, values.clone(), grammar).is_err());
            }
            let report = parse_style_attribute(&format!("{name}:{text};color:red"));
            assert!(!report.is_clean());
            assert_eq!(report.syntax().len(), 1);
            assert_eq!(
                color(&report.syntax()[0]).to_specified_css().unwrap(),
                "red"
            );
        }
    }
}
#[test]
fn ordinary_supports_preserves_unsupported_bare_quirky_color_tests_as_syntax() {
    for text in ["color:123", "background-color:ABC", "border-color:12abc"] {
        let values = parse_component_values(text).unwrap();
        let before = values.clone();
        let declaration = CssSupportsDeclaration::try_from_components(values.clone()).unwrap();
        assert!(declaration.known().is_none());
        assert_eq!(declaration.serialize().unwrap().as_css(), text);
        assert_eq!(values, before);
    }
}

#[test]
fn invalid_checked_color_reports_the_actual_original_token_not_generated_source() {
    let values = parse_component_values("10px").unwrap();
    let before = values.clone();
    let CssValueOrigin::Parsed(original) = values.items()[0].origin() else {
        panic!("parsed token")
    };
    for grammar in [false, true] {
        let error = checked_components("color", values.clone(), grammar).unwrap_err();
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidColorSyntax(_))
        ));
        let origin = match error.origin() {
            CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin))
            | CssSerializedOrigin::End(Some(CssValueOrigin::Parsed(origin))) => origin,
            _ => panic!("grammar error anchored to the actual supplied token"),
        };
        assert!(origin.source().same_snapshot(original.source()));
        assert_eq!(origin.span(), original.span());
        assert_eq!(values, before);
    }
}
