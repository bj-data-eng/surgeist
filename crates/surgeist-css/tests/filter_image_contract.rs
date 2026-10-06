#![forbid(unsafe_code)]
//! Authored Filter Effects image grammar through existing checked declaration fronts.
//! Filter Effects 1 WD20181218 §§5–6,12 supplies the missing image grammar.
//! Preserve String separately from Image; downstream string interpretation is unresolved.
//! New typed variants, checked constructors, borrowed views and graph errors need
//! functional tests with implementation, described in filter-image-source-basis.md.
//! No missing symbols, stubs, browser execution or acceptance verdict is intended.

use surgeist_css::{
    CssKnownProperty as P, CssSpecifiedValueSerializationErrorKind as K,
    CssSpecifiedValueSerializationLimits as L, *,
};

const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("inherit", CssGlobalKeyword::Inherit),
    ("initial", CssGlobalKeyword::Initial),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn checked(
    p: P,
    values: CssComponentValues,
    grammar: bool,
) -> Result<CssDeclaration, CssPropertyValueParseError> {
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

fn fronts(p: P, text: &str) -> Vec<CssDeclaration> {
    let css = format!(
        "/*😀*/{}:{text}!important",
        p.canonical_name().to_ascii_uppercase()
    );
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert_eq!(validate_style_attribute(&css).unwrap(), *report.syntax());
    let [parsed] = report.syntax().as_slice() else {
        panic!("one occurrence")
    };
    assert_eq!(
        parsed.position().unwrap().byte_offset().value(),
        "/*😀*/".len()
    );
    assert_eq!(
        parsed.position().unwrap().column().value() as usize,
        "/*😀*/".encode_utf16().count()
    );
    assert_eq!(parsed.parsed_value().unwrap().source().as_str(), css);
    let values = parse_component_values(text).unwrap();
    let before = values.clone();
    let mut result = vec![parsed.clone()];
    for grammar in [false, true] {
        let source = checked(p, values.clone(), grammar).unwrap();
        assert_eq!(source.value_components(), &values);
        assert!(source.position().is_none());
        assert!(source.parsed_name().is_none());
        assert!(source.parsed_value().is_none());
        result.push(source);
    }
    for report in [
        parse_property_value_text(text, CssPropertyNameRef::Known(p), CssImportance::Important),
        parse_property_value_text_for_grammar(text, p.grammar(), CssImportance::Important),
    ] {
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        let source = report.syntax().as_ref().unwrap();
        assert_eq!(source.parsed_value().unwrap().source().as_str(), text);
        result.push(source.clone());
    }
    assert_eq!(values, before);
    for source in &result {
        assert_eq!(source.known().unwrap().property(), p);
        assert_eq!(source.known().unwrap().grammar(), p.grammar());
        assert_eq!(source.importance(), CssImportance::Important);
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            text
        );
    }
    result
}

fn canonical(p: P, text: &str, value: &str) {
    let expected = format!("{}: {value} !important;", p.canonical_name());
    for source in fronts(p, text) {
        let before = source.clone();
        for _ in 0..2 {
            assert_eq!(
                source
                    .to_specified_css_with_limits(L::new(65_536, 262_144, expected.len() - 1))
                    .unwrap_err()
                    .kind(),
                K::ByteLimit
            );
            assert_eq!(source, before);
            assert_eq!(
                source
                    .to_specified_css_with_limits(L::new(65_536, 262_144, expected.len()))
                    .unwrap(),
                expected
            );
        }
        let report = parse_style_attribute(&expected);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(report.syntax()[0].to_specified_css().unwrap(), expected);
    }
}

fn invalid(p: P, text: &str) {
    let css = format!("color:red;{}:{text};color:blue", p.canonical_name());
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "atomic rejection: {css}");
    assert!(
        report
            .syntax()
            .iter()
            .all(|d| d.known().unwrap().property() == P::Color)
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one owned grammar failure: {css}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let values = parse_component_values(text).unwrap();
    let before = values.clone();
    for grammar in [false, true] {
        assert!(matches!(
            checked(p, values.clone(), grammar).unwrap_err().kind(),
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
fn image_operand_urls_keep_url_and_src_identity() {
    for (input, expected) in [
        ("filter(url(a),blur())", "filter(url(\"a\"), blur())"),
        (
            "FILTER(src(\"#a\"),HUE-ROTATE())",
            "filter(src(\"#a\"), hue-rotate())",
        ),
        (
            "filter(url(\"\"),url(#f))",
            "filter(url(\"\"), url(\"#f\"))",
        ),
        (
            "filter(url(\"a\" cors m(a b)),blur())",
            "filter(url(\"a\" cors m(a b)), blur())",
        ),
    ] {
        canonical(P::BackgroundImage, input, expected);
    }
}

#[test]
fn string_operand_stays_a_distinct_authored_string_without_url_conversion() {
    for (input, expected) in [
        ("filter('a.png',blur())", "filter(\"a.png\", blur())"),
        ("filter(\"#a\",url(#f))", "filter(\"#a\", url(\"#f\"))"),
        ("filter('',blur())", "filter(\"\", blur())"),
        (
            "filter('é😀',brightness())",
            "filter(\"é😀\", brightness())",
        ),
    ] {
        canonical(P::BackgroundImage, input, expected);
    }
}

#[test]
fn all_four_existing_gradient_forms_are_image_operands() {
    for function in [
        "linear-gradient",
        "repeating-linear-gradient",
        "radial-gradient",
        "repeating-radial-gradient",
    ] {
        canonical(
            P::BackgroundImage,
            &format!("filter({function}(red,blue),blur())"),
            &format!("filter({function}(red, blue), blur())"),
        );
    }
}

#[test]
fn light_dark_keeps_image_none_branches_and_filter_nesting() {
    for (input, expected) in [
        (
            "filter(light-dark(none,none),blur())",
            "filter(light-dark(none, none), blur())",
        ),
        (
            "light-dark(filter('a',blur()),filter(url(b),sepia()))",
            "light-dark(filter(\"a\", blur()), filter(url(\"b\"), sepia()))",
        ),
        (
            "filter(filter('a',blur()),hue-rotate())",
            "filter(filter(\"a\", blur()), hue-rotate())",
        ),
    ] {
        canonical(P::BackgroundImage, input, expected);
    }
}

#[test]
fn full_nonempty_filter_list_preserves_order_omission_color_and_url_children() {
    canonical(
        P::BackgroundImage,
        "filter('a',blur() brightness() contrast(150%) drop-shadow(1px 2px 3px red) grayscale(2) hue-rotate(450deg) invert(2) opacity(125%) saturate(3) sepia(200%) src('#f'))",
        "filter(\"a\", blur() brightness() contrast(150%) drop-shadow(red 1px 2px 3px) grayscale(2) hue-rotate(450deg) invert(2) opacity(125%) saturate(3) sepia(200%) src(\"#f\"))",
    );
    canonical(
        P::BackgroundImage,
        "filter(linear-gradient(light-dark(red,blue),currentColor),drop-shadow(color-mix(in srgb,currentColor,blue) calc(1px + 2em) -2px))",
        "filter(linear-gradient(light-dark(red, blue), currentcolor), drop-shadow(color-mix(in srgb, currentcolor, blue) calc(2em + 1px) -2px))",
    );
}

#[test]
fn nested_numeric_payloads_keep_authored_phase_and_existing_matching() {
    for (input, expected) in [
        (
            "filter('a',blur(calc(1px + 2em)))",
            "filter(\"a\", blur(calc(2em + 1px)))",
        ),
        (
            "filter('a',brightness(calc(2 - 3)))",
            "filter(\"a\", brightness(calc(-1)))",
        ),
        (
            "filter('a',opacity(calc((1px + 1%) / 1px)))",
            "filter(\"a\", opacity(calc((1% + 1px) / 1px)))",
        ),
        (
            "filter('a',hue-rotate(calc(30deg + 60deg)))",
            "filter(\"a\", hue-rotate(calc(90deg)))",
        ),
    ] {
        canonical(P::BackgroundImage, input, expected);
    }
}

#[test]
fn imported_longhand_image_consumers_share_one_grammar() {
    for p in [
        P::BackgroundImage,
        P::BorderImageSource,
        P::ListStyleImage,
        P::MaskImage,
        P::MaskBorderSource,
        P::Content,
    ] {
        canonical(p, "filter('a',blur())", "filter(\"a\", blur())");
        for source in fronts(p, "filter('a',blur())") {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("terminal")
            };
            let [item] = values.items() else {
                panic!("one longhand")
            };
            assert_eq!(item.property(), p);
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), source.importance());
        }
    }
}

#[test]
fn importing_shorthands_preserve_their_existing_expansion_footprints() {
    for (p, count, image_property) in [
        (P::Background, 9, P::BackgroundImage),
        (P::BorderImage, 5, P::BorderImageSource),
        (P::ListStyle, 3, P::ListStyleImage),
        (P::Mask, 14, P::MaskImage),
        (P::MaskBorder, 6, P::MaskBorderSource),
    ] {
        for source in fronts(p, "filter('a',blur())") {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("shorthand")
            };
            assert_eq!(values.items().len(), count);
            let image = values
                .items()
                .iter()
                .find(|item| item.property() == image_property)
                .unwrap();
            assert!(image.ordinary_value().is_some());
            for item in values.items() {
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), source.importance());
            }
            let report = parse_style_attribute(&source.to_specified_css().unwrap());
            assert!(report.is_clean());
        }
    }
}

#[test]
fn counter_style_and_symbols_image_slots_admit_filter_images() {
    canonical(
        P::ListStyleType,
        "symbols(filter('a',blur()))",
        "symbols(filter(\"a\", blur()))",
    );
    for (body, descriptor, expected) in [
        (
            "symbols:filter('a',blur());",
            CssCounterStyleDescriptorKind::Symbols,
            "filter(\"a\", blur())",
        ),
        (
            "symbols:a;prefix:filter('a',blur());",
            CssCounterStyleDescriptorKind::Prefix,
            "filter(\"a\", blur())",
        ),
        (
            "symbols:a;suffix:filter('a',blur());",
            CssCounterStyleDescriptorKind::Suffix,
            "filter(\"a\", blur())",
        ),
        (
            "symbols:a;negative:filter('a',blur()) ')';",
            CssCounterStyleDescriptorKind::Negative,
            "filter(\"a\", blur()) \")\"",
        ),
        (
            "symbols:a;pad:2 filter('a',blur());",
            CssCounterStyleDescriptorKind::Pad,
            "2 filter(\"a\", blur())",
        ),
        (
            "system:additive;additive-symbols:1 filter('a',blur());",
            CssCounterStyleDescriptorKind::AdditiveSymbols,
            "1 filter(\"a\", blur())",
        ),
    ] {
        let css = format!("@counter-style F {{{body}}}");
        let report = parse_sheet(&css);
        assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
        let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
            panic!("counter definition")
        };
        assert_eq!(rule.descriptor_specified_css(descriptor).unwrap(), expected);
        assert!(parse_sheet(&rule.to_specified_css().unwrap()).is_clean());
    }
}

#[test]
fn invalid_filter_image_grammar_drops_only_its_declaration() {
    for text in [
        "filter()",
        "filter(url(a))",
        "filter('a')",
        "filter(url(a) blur())",
        "filter(url(a),)",
        "filter('a',none)",
        "filter(none,blur())",
        "filter('a',none blur())",
        "filter('a',blur() none)",
        "filter('a',blur(),sepia())",
        "filter('a' 'b',blur())",
        "filter(a,blur())",
        "filter(1,blur())",
        "filter('a',unknown())",
        "filter('a',blur(1%))",
        "filter('a',blur(-1px))",
        "filter('a',brightness(-1e-999))",
        "filter('a',hue-rotate(1px))",
        "filter('a',drop-shadow(inset 0 0))",
        "filter('a',drop-shadow(0 0 1px 2px))",
        "filter('a',filter(url(a),blur()))",
        "filter('a',blur()) junk",
    ] {
        invalid(P::BackgroundImage, text);
    }
}

#[test]
fn image_extension_does_not_replace_unrelated_url_and_filter_grammars() {
    for p in [P::Filter, P::BackdropFilter, P::CueBefore] {
        invalid(p, "filter('a',blur())");
    }
    // Cursor's published UI4 owner remains a separately scoped URL/image-set lane.
    // Positive cursor replacement requires its #328 source/ownership disposition.
    canonical(P::Filter, "none", "none");
    canonical(P::BackdropFilter, "none", "none");
}

#[test]
fn every_css_wide_value_stays_whole_and_symbolic() {
    for p in [
        P::BackgroundImage,
        P::BorderImageSource,
        P::ListStyleImage,
        P::MaskImage,
        P::MaskBorderSource,
        P::Content,
    ] {
        for (input, keyword) in GLOBALS {
            for source in fronts(p, input) {
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    expand_declaration(&source).unwrap()
                else {
                    panic!("global terminal")
                };
                assert_eq!(
                    values.items()[0].value(),
                    CssContributionValueRef::Global(keyword)
                );
                assert!(values.items()[0].source().same_occurrence(&source));
            }
            invalid(p, &format!("{input} filter('a',blur())"));
        }
    }
}

#[test]
fn pending_reentry_is_strict_reusable_and_keeps_occurrence_and_replacement() {
    for text in [
        "bogus var(--image)",
        "bogus env(image)",
        "bogus attr(data-image)",
    ] {
        for source in fronts(P::BackgroundImage, text) {
            let original = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("whole pending value")
            };
            for _ in 0..2 {
                for bad in ["filter('a',none)", "inherit bogus"] {
                    let replacement = parse_component_values(bad).unwrap();
                    let direct =
                        checked(P::BackgroundImage, replacement.clone(), false).unwrap_err();
                    let error = handle.reenter(replacement).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
                        panic!("strict replacement")
                    };
                    assert_eq!(actual.kind(), direct.kind());
                    assert_eq!(actual.origin(), direct.origin());
                }
                assert_eq!(
                    handle
                        .reenter(parse_component_values("filter(var(--again),blur())").unwrap())
                        .unwrap_err()
                        .kind(),
                    &CssExpansionErrorKind::ResidualSubstitution
                );
                for good in ["/*😀*/filter('a',blur())", "inherit"] {
                    let replacement = parse_component_values(good).unwrap();
                    let before = replacement.clone();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("resolved terminal")
                    };
                    assert_eq!(values.items().len(), 1);
                    assert_eq!(
                        values.items()[0].replacement_components(),
                        Some(&replacement)
                    );
                    assert!(values.items()[0].source().same_occurrence(&source));
                    assert_eq!(
                        values.items()[0].source().importance(),
                        CssImportance::Important
                    );
                    assert_eq!(replacement, before);
                }
            }
            assert_eq!(source, original);
            assert!(handle.source().same_occurrence(&source));
        }
    }
}

fn implicit_origin(values: &CssComponentValues) -> CssValueOrigin {
    let serialized = values.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("fixture retains implicit closure")
}

#[test]
fn checked_original_image_and_pending_closures_are_not_repaired() {
    for text in [
        "/*😀*/filter('a',blur()",
        "filter('a',blur())/*unfinished",
        "inherit/*unfinished",
        "var(--image",
        "var(--image)/*unfinished",
    ] {
        let values = parse_component_values(text).unwrap();
        let before = values.clone();
        let origin = implicit_origin(&values);
        for grammar in [false, true] {
            let error = checked(P::BackgroundImage, values.clone(), grammar).unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ));
            assert_eq!(
                error.origin(),
                &CssSerializedOrigin::End(Some(origin.clone()))
            );
        }
        assert_eq!(values, before);
    }
}

#[test]
fn replacement_original_closure_is_checked_after_residual_substitution() {
    let source = fronts(P::BackgroundImage, "var(--image)").remove(0);
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    let values = parse_component_values("filter('a',blur()").unwrap();
    let origin = implicit_origin(&values);
    let error = handle.reenter(values).unwrap_err();
    let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
        panic!("original closure")
    };
    assert_eq!(actual.origin(), &CssSerializedOrigin::End(Some(origin)));
    assert!(matches!(
        actual.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert_eq!(
        handle
            .reenter(parse_component_values("filter(var(--again),blur()").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    assert!(
        handle
            .reenter(parse_component_values("filter('a',blur())").unwrap())
            .is_ok()
    );
}

#[test]
fn normalization_keeps_duplicate_order_context_importance_and_pending_values() {
    let css = ".a{background-image:filter('a',blur())!important;background-image:var(--image);list-style-image:filter(url(b),sepia());background-image:unset}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssRule::Style(style) = &report.syntax().rules()[0] else {
        panic!("style rule")
    };
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(0, 1, 4, 4).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(item) => Some(item),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 4);
    for (order, (item, source)) in declarations
        .iter()
        .zip(style.declarations().iter())
        .enumerate()
    {
        assert_eq!(item.order(), order);
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), source.importance());
        match item.expansion() {
            CssExpansion::Pending(handle) if order == 1 => {
                assert!(handle.source().same_occurrence(source))
            }
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(values.items().len(), 1)
            }
            other => panic!("one authored expansion: {other:?}"),
        }
    }
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(0, 1, 3, 4).unwrap(),
            CssNormalizationResource::Declarations,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 4, 3).unwrap(),
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
                .same_occurrence(&style.declarations()[3])
        );
        assert_eq!(report, before);
    }
    assert!(normalize_report_with_limits(&report, exact).is_ok());
}

fn budget(text: &str, expected_value: &str, input: usize, projection: usize) {
    let expected = format!("background-image: {expected_value} !important;");
    for source in fronts(P::BackgroundImage, text) {
        let before = source.clone();
        let exact = L::new(input, projection, expected.len());
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        for _ in 0..2 {
            for (limits, kind) in [
                (
                    L::new(input - 1, projection, expected.len()),
                    K::InputNodeLimit,
                ),
                (
                    L::new(input, projection - 1, expected.len()),
                    K::ProjectionNodeLimit,
                ),
                (L::new(input, projection, expected.len() - 1), K::ByteLimit),
                (L::new(0, projection, expected.len()), K::InputNodeLimit),
                (L::new(input, 0, expected.len()), K::ProjectionNodeLimit),
                (L::new(input, projection, 0), K::ByteLimit),
            ] {
                assert_eq!(
                    source
                        .to_specified_css_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind,
                    "{text}"
                );
                assert_eq!(source, before);
            }
            assert_eq!(
                source.to_specified_css_with_limits(exact).unwrap(),
                expected
            );
        }
    }
}

#[test]
fn filter_image_aggregates_and_distinct_operands_have_exact_cumulative_tariffs() {
    // Each declaration/name costs 2; image list costs 1; filter image costs 1;
    // String costs 1 or URL costs 2; filter list costs 1; omitted blur costs 1.
    budget("filter('a',blur())", "filter(\"a\", blur())", 7, 7);
    budget("filter(url(a),blur())", "filter(url(\"a\"), blur())", 8, 8);
    budget(
        "filter(filter('a',blur()),hue-rotate())",
        "filter(filter(\"a\", blur()), hue-rotate())",
        10,
        10,
    );
    budget(
        "filter(light-dark(none,none),blur())",
        "filter(light-dark(none, none), blur())",
        9,
        9,
    );
    budget(
        "filter('a',blur()),filter(url(b),hue-rotate())",
        "filter(\"a\", blur()), filter(url(\"b\"), hue-rotate())",
        12,
        12,
    );
}

#[test]
fn numeric_filter_siblings_share_the_image_context_and_differing_projection_cost() {
    // Existing filter list costs 11 input/13 projection for two blur calculations.
    // Add image aggregate+String (2), image list (1), declaration/name (2).
    budget(
        "filter('a',blur(calc(1px + 2em)) blur(calc(1px + 2em)))",
        "filter(\"a\", blur(calc(2em + 1px)) blur(calc(2em + 1px)))",
        16,
        18,
    );
    // URL modifiers keep their own seven-node tariff; no secondary writer reset.
    budget(
        "filter(url(\"x\" cors m(a b)),hue-rotate())",
        "filter(url(\"x\" cors m(a b)), hue-rotate())",
        13,
        13,
    );
}

#[test]
fn suppressed_gradient_defaults_are_still_visited_inside_filter_images() {
    // Gradient 10 (direction/bottom and endpoints visited even when omitted);
    // filter image 1 + filter list 1 + blur 1 + image list 1 + declaration/name 2.
    budget(
        "filter(linear-gradient(to bottom,red 0%,blue 100%),blur())",
        "filter(linear-gradient(red, blue), blur())",
        16,
        16,
    );
    // Existing fully explicit radial omitted-default tariff is 14; enclosing 6.
    budget(
        "filter(radial-gradient(ellipse farthest-corner at 50% 50%,red,blue),blur())",
        "filter(radial-gradient(red, blue), blur())",
        20,
        20,
    );
}

#[test]
fn nested_filter_parser_boundary_counts_actual_retained_function_depth() {
    let nested = |count: usize| {
        let mut text = "'a'".to_owned();
        for _ in 0..count {
            text = format!("filter({text},blur())");
        }
        text
    };
    // 255 filter functions + deepest blur = the common 256 component ceiling.
    let valid = nested(255);
    let report = parse_property_value_text(
        &valid,
        CssPropertyNameRef::Known(P::BackgroundImage),
        CssImportance::Normal,
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let invalid = nested(256);
    let report = parse_property_value_text(
        &invalid,
        CssPropertyNameRef::Known(P::BackgroundImage),
        CssImportance::Normal,
    );
    assert!(!report.is_clean());
    assert!(report.syntax().is_none());
    assert!(parse_component_values(&invalid).is_err());
}

#[test]
fn existing_filter_provider_control_has_the_full_ordered_list_and_omissions() {
    for p in [P::Filter, P::BackdropFilter] {
        canonical(
            p,
            "blur() hue-rotate() drop-shadow(1px 2px)",
            "blur() hue-rotate() drop-shadow(1px 2px)",
        );
        canonical(
            p,
            "drop-shadow(1px 2px 3px red) contrast(150%)",
            "drop-shadow(red 1px 2px 3px) contrast(150%)",
        );
    }
}

#[test]
fn existing_image_url_gradient_control_has_independent_canonical_text() {
    canonical(
        P::BackgroundImage,
        "url(a),src('#b'),linear-gradient(red,blue),light-dark(none,url(c))",
        "url(\"a\"), src(\"#b\"), linear-gradient(red, blue), light-dark(none, url(\"c\"))",
    );
}

#[test]
fn existing_numeric_and_color_provider_control_keeps_the_complete_symbolic_children() {
    canonical(
        P::Filter,
        "blur(calc(1px + 2em)) opacity(calc((1px + 1%) / 1px))",
        "blur(calc(2em + 1px)) opacity(calc((1% + 1px) / 1px))",
    );
    canonical(P::Color, "light-dark(red,blue)", "light-dark(red, blue)");
    canonical(
        P::Color,
        "color-mix(in srgb,currentColor,blue)",
        "color-mix(in srgb, currentcolor, blue)",
    );
}
