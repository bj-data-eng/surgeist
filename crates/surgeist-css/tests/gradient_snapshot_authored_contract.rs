#![forbid(unsafe_code)]
//! Independently authored Images 4 WD 2025-09-30 §§3.1, 3.2.1, 3.3.1, 3.5.1
//! contract, selected by Snapshot 2026 and owning issue #1023.
//! https://www.w3.org/TR/2025/WD-css-images-4-20250930/#linear-gradients
//! https://www.w3.org/TR/2025/WD-css-images-4-20250930/#radial-color-interpolation
//! https://www.w3.org/TR/2025/WD-css-images-4-20250930/#conic-gradient-syntax
//! https://www.w3.org/TR/2025/WD-css-images-4-20250930/#color-stop-syntax
//! Commas adjoining omitted optional productions are omitted (Values 4 §2.1):
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#comb-comma
//! The specific at-position links import Values 5 WD 2024-11-11 §4.2, despite
//! Images 4's general Values 3 prose. All three coordinate families remain
//! symbolic under the already adopted #935 position output contract.
//! https://www.w3.org/TR/2024/WD-css-values-5-20241111/#position
//! Required composed Color 4/5 interpolation productions reuse existing owners:
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#interpolation-space
//! https://www.w3.org/TR/2026/WD-css-color-5-20260908/#color-interpolation-method
//! No contextual geometry, stop fixup, color interpolation, or rendering oracle.
//! Repeating-conic and other unselected Images 4 siblings are outside this leaf.

use surgeist_css::{
    CssKnownProperty as P, CssSpecifiedValueSerializationErrorKind as K,
    CssSpecifiedValueSerializationLimits as L, *,
};

const LINEAR: [&str; 2] = ["linear-gradient", "repeating-linear-gradient"];
const RADIAL: [&str; 2] = ["radial-gradient", "repeating-radial-gradient"];
const SELECTED: [&str; 5] = [
    "linear-gradient",
    "repeating-linear-gradient",
    "radial-gradient",
    "repeating-radial-gradient",
    "conic-gradient",
];

fn parsed(property: P, value: &str) -> CssDeclaration {
    let source = format!("/*😀*/{}:{value}!important", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(validate_style_attribute(&source).unwrap(), *report.syntax());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(declaration.known().unwrap().property(), property);
    declaration.clone()
}

fn fronts(property: P, value: &str) -> Vec<CssDeclaration> {
    let parsed = parsed(property, value);
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    let mut declarations = vec![parsed];
    for grammar in [false, true] {
        let checked = if grammar {
            parse_property_value_for_grammar(
                property.grammar(),
                components.clone(),
                CssImportance::Important,
            )
        } else {
            parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                CssImportance::Important,
            )
        }
        .unwrap_or_else(|error| panic!("checked {value}: {error:?}"));
        assert_eq!(checked.value_components(), &before);
        assert_eq!(checked.importance(), CssImportance::Important);
        declarations.push(checked);
    }
    assert_eq!(components, before);
    declarations
}

fn image(declaration: &CssDeclaration) -> &CssImageValue {
    let CssKnownPropertyValueRef::BackgroundImage(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("background image")
    };
    let [image] = value.images().images() else {
        panic!("one image")
    };
    image
}

fn round_trip(value: &str) {
    for declaration in fronts(P::BackgroundImage, value) {
        let before = declaration.clone();
        assert!(matches!(image(&declaration), CssImageValue::Gradient(_)));
        let checked_image = CssImage::try_new(image(&declaration).clone()).unwrap();
        let specified = checked_image.serialize_specified().unwrap();
        let reparsed = parsed(P::BackgroundImage, &specified);
        // Intrinsic normalization may collapse standard aliases/default spellings;
        // its output must retain meaning and have a stable semantic reparse.
        assert_eq!(image(&reparsed).serialize_specified().unwrap(), specified);
        let output = declaration.to_specified_css().unwrap();
        let report = parse_style_attribute(&output);
        assert!(report.is_clean(), "{output}: {:?}", report.diagnostics());
        assert_eq!(report.syntax()[0].to_specified_css().unwrap(), output);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&declaration).unwrap()
        else {
            panic!("ordinary image expansion")
        };
        let [item] = values.items() else {
            panic!("one image terminal")
        };
        assert_eq!(item.property(), P::BackgroundImage);
        assert!(item.source().same_occurrence(&declaration));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.ordinary_value().is_some());
        assert!(item.replacement_components().is_none());
        assert_eq!(declaration, before);
    }
}

fn rejects(value: &str) {
    let prefix = "/*😀*/color:red;\n";
    let unit = format!("background-image:{value};");
    let source = format!("{prefix}{unit}color:blue!important");
    let report = parse_style_attribute(&source);
    let [first, last] = report.syntax().as_slice() else {
        panic!("two neighbors: {source}")
    };
    assert_eq!(first.to_specified_css().unwrap(), "color: red;");
    assert_eq!(last.to_specified_css().unwrap(), "color: blue !important;");
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejection: {source}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        prefix.len()
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        prefix.len() + unit.len()
    );
    let responsible = diagnostic.error().position();
    assert!(responsible.byte_offset().value() >= prefix.len());
    assert!(responsible.byte_offset().value() < prefix.len() + unit.len());
    assert_eq!(responsible.line().value(), 1);
    assert_eq!(
        responsible.column().value() as usize,
        source[prefix.len()..responsible.byte_offset().value()]
            .encode_utf16()
            .count()
    );
    assert_eq!(
        validate_style_attribute(&source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    for grammar in [false, true] {
        let components = parse_component_values(value).unwrap();
        let result = if grammar {
            parse_property_value_for_grammar(
                P::BackgroundImage.grammar(),
                components,
                CssImportance::Normal,
            )
        } else {
            parse_property_value(
                CssPropertyNameRef::Known(P::BackgroundImage),
                components,
                CssImportance::Normal,
            )
        };
        assert!(result.is_err(), "checked accepted {value}");
    }
}

#[test]
fn linear_single_stop_is_admitted_without_a_trailing_comma() {
    for function in LINEAR {
        for stop in ["red", "red 10%", "red 10px 20%"] {
            round_trip(&format!("{function}({stop})"));
        }
    }
}
#[test]
fn radial_single_stop_is_admitted_without_a_trailing_comma() {
    for function in RADIAL {
        for stop in ["red", "red 10%", "red 10px 20%"] {
            round_trip(&format!("{function}({stop})"));
        }
    }
}
#[test]
fn conic_single_stop_preserves_zero_one_and_two_angular_positions() {
    for stop in ["red", "red 10%", "red 0 1turn", "red -90deg 150%"] {
        round_trip(&format!("conic-gradient({stop})"));
    }
}
#[test]
fn linear_two_position_stops_admit_mixed_lengths_and_percentages() {
    for function in LINEAR {
        round_trip(&format!("{function}(red -10px 20%, 40%, blue 80% 120px)"));
    }
}
#[test]
fn radial_two_position_stops_admit_mixed_lengths_and_percentages() {
    for function in RADIAL {
        round_trip(&format!("{function}(red -10px 20%, 40%, blue 80% 120px)"));
    }
}
#[test]
fn conic_angle_percentage_stops_and_intervening_hints_are_admitted() {
    for value in [
        "conic-gradient(red, blue)",
        "conic-gradient(red -90deg 25%, 0.5turn, blue 270deg 150%)",
        "conic-gradient(red 0, 0, blue 1turn)",
        "conic-gradient(red calc(90deg + 10%) calc(180deg + 20%), calc(200deg + 5%), blue)",
    ] {
        round_trip(value);
    }
}
#[test]
fn explicit_linear_interpolation_can_precede_or_follow_the_complete_direction() {
    for function in LINEAR {
        for prelude in [
            "in oklch longer hue",
            "to right top in oklch longer hue",
            "in oklch longer hue to top right",
            "0 in lab",
            "in lab 0",
            "calc(45deg + 45deg) in srgb",
        ] {
            round_trip(&format!("{function}({prelude}, red, blue)"));
        }
    }
}
#[test]
fn explicit_radial_interpolation_can_precede_or_follow_the_complete_shape_size_position_group() {
    for function in RADIAL {
        for group in [
            "",
            "circle",
            "closest-side",
            "circle 10px",
            "10px circle",
            "ellipse 20% 30%",
            "20% 30% ellipse",
            "at right bottom",
            "circle 10px at right bottom",
        ] {
            for prelude in [
                format!("in hsl increasing hue {group}"),
                format!("{group} in hsl increasing hue"),
            ] {
                round_trip(&format!("{function}({prelude}, red, blue)"));
            }
        }
    }
}
#[test]
fn conic_from_at_group_is_contiguous_on_either_side_of_interpolation() {
    for group in [
        "",
        "from 0",
        "from -0.25turn",
        "at right bottom",
        "from calc(45deg + 45deg) at left 10px top 20%",
    ] {
        for prelude in [
            format!("in oklch decreasing hue {group}"),
            format!("{group} in oklch decreasing hue"),
        ] {
            round_trip(&format!("conic-gradient({prelude}, red, blue)"));
        }
    }
}
#[test]
fn conic_from_admits_angle_or_literal_zero_and_rejects_other_numeric_dimensions() {
    for angle in [
        "0",
        "-0",
        "0deg",
        "1turn",
        "100grad",
        "1rad",
        "calc(90deg + 1turn)",
    ] {
        round_trip(&format!("conic-gradient(from {angle}, red, blue)"));
    }
    for angle in ["1", "10%", "1px", "calc(10% + 20%)"] {
        rejects(&format!("conic-gradient(from {angle}, red, blue)"));
    }
}
#[test]
fn every_selected_gradient_admits_all_rectangular_interpolation_spaces() {
    for function in SELECTED {
        for space in [
            "srgb",
            "srgb-linear",
            "display-p3",
            "display-p3-linear",
            "a98-rgb",
            "prophoto-rgb",
            "rec2020",
            "lab",
            "oklab",
            "xyz",
            "xyz-d50",
            "xyz-d65",
        ] {
            round_trip(&format!("{function}(in {space}, red, blue)"));
        }
    }
}
#[test]
fn every_selected_gradient_admits_all_polar_spaces_and_hue_strategies() {
    for function in SELECTED {
        for space in ["hsl", "hwb", "lch", "oklch"] {
            for suffix in [
                "",
                " shorter hue",
                " longer hue",
                " increasing hue",
                " decreasing hue",
            ] {
                round_trip(&format!("{function}(in {space}{suffix}, red, blue)"));
            }
        }
    }
}
#[test]
fn custom_interpolation_names_preserve_case_escapes_and_unresolved_profile_identity() {
    for function in SELECTED {
        for name in ["--Profile", "--profile", "--Pr\\6f file", "--a\\ b", "--"] {
            round_trip(&format!("{function}(in {name}, red, blue)"));
        }
        let upper = parsed(
            P::BackgroundImage,
            &format!("{function}(in --Profile, red, blue)"),
        );
        let lower = parsed(
            P::BackgroundImage,
            &format!("{function}(in --profile, red, blue)"),
        );
        assert_ne!(image(&upper), image(&lower));
    }
}
#[test]
fn interpolation_keywords_are_case_insensitive_and_escape_aware() {
    for function in SELECTED {
        round_trip(&format!("{function}(IN OKLCH LONGER HUE, RED, BLUE)"));
        round_trip(&format!(
            "{function}(i\\6e oklch l\\6f nger h\\75 e, red, blue)"
        ));
    }
}
#[test]
fn rectangular_and_custom_spaces_reject_hue_and_incomplete_methods() {
    for function in SELECTED {
        for prelude in [
            "in srgb longer hue",
            "in lab shorter hue",
            "in --Profile increasing hue",
            "in hsl longer",
            "in hsl hue",
            "in shorter hue hsl",
            "in hsl sideways hue",
            "in hsl longer hue shorter hue",
            "in",
            "in unknown",
            "hsl",
        ] {
            rejects(&format!("{function}({prelude}, red, blue)"));
        }
    }
}
#[test]
fn prelude_groups_reject_split_duplicate_and_reversed_constituents() {
    for function in LINEAR {
        for prelude in [
            "to right in lab top",
            "to in lab right",
            "in lab in srgb",
            "45deg to right in lab",
        ] {
            rejects(&format!("{function}({prelude}, red, blue)"));
        }
    }
    for function in RADIAL {
        for prelude in [
            "circle in lab at center",
            "circle in lab 10px",
            "at center circle in lab",
            "in lab at center 10px circle",
            "circle 10px at center in lab at left",
        ] {
            rejects(&format!("{function}({prelude}, red, blue)"));
        }
    }
    for prelude in [
        "from 45deg in lab at center",
        "at center from 45deg",
        "from in lab 45deg",
        "from 45deg from 90deg",
        "at center at left",
        "in lab in hsl",
    ] {
        rejects(&format!("conic-gradient({prelude}, red, blue)"));
    }
}
#[test]
fn selected_lists_reject_dangling_commas_missing_stops_and_bad_hint_order() {
    for function in SELECTED {
        for list in [
            "",
            "red,",
            ",red",
            "red,,blue",
            "20%,red,blue",
            "red,20%",
            "red,20%,30%,blue",
            "red 20% blue",
            "red 10% 20% 30%,blue",
        ] {
            rejects(&format!("{function}({list})"));
        }
    }
}
#[test]
fn linear_and_radial_lists_reject_angles_while_conic_lists_reject_lengths() {
    for function in LINEAR.into_iter().chain(RADIAL) {
        for list in [
            "red 90deg,blue",
            "red,90deg,blue",
            "red 10% 90deg,blue",
            "red calc(90deg + 10%),blue",
        ] {
            rejects(&format!("{function}({list})"));
        }
    }
    for list in [
        "red 1px,blue",
        "red,1px,blue",
        "red 10% 1px,blue",
        "red calc(1px + 10%),blue",
        "red 1,blue",
        "red,1,blue",
    ] {
        rejects(&format!("conic-gradient({list})"));
    }
}
#[test]
fn exact_at_import_admits_complete_one_two_and_four_component_position_families() {
    // Independent production branches: physical/axis-relative Cartesian,
    // named block/inline flow, and ordered start/end flow. No writing-mode input.
    for function in RADIAL.into_iter().chain(["conic-gradient"]) {
        for position in [
            "left",
            "center",
            "right",
            "top",
            "bottom",
            "x-start",
            "x-end",
            "y-start",
            "y-end",
            "block-start",
            "block-end",
            "inline-start",
            "inline-end",
            "25%",
            "left top",
            "top left",
            "y-end x-start",
            "x-end y-start",
            "left 25%",
            "25% top",
            "25% 75%",
            "block-start inline-end",
            "inline-start block-end",
            "center inline-start",
            "block-end center",
            "start end",
            "end start",
            "center start",
            "end center",
            "left -10px top 20%",
            "y-end 20% x-start -10px",
            "x-end 10% bottom calc(1px + 2%)",
            "block-start 10px inline-end 20%",
            "inline-start 20% block-end -10px",
            "start 10px end 20%",
            "end -10% start calc(1px + 2%)",
        ] {
            round_trip(&format!("{function}(at {position}, red, blue)"));
        }
    }
}
#[test]
fn exact_at_import_rejects_three_component_and_cross_family_position_mixtures() {
    for function in RADIAL.into_iter().chain(["conic-gradient"]) {
        for position in [
            "left right",
            "50% left",
            "left top 10px",
            "left 10px top",
            "top 10px",
            "center 10px top 20px",
            "block-start left",
            "x-start inline-end",
            "start inline-end",
            "start",
            "block-start 10px inline-end",
            "start 10px end",
            "left 10px right 20px",
        ] {
            rejects(&format!("{function}(at {position}, red, blue)"));
        }
    }
}
#[test]
fn checked_singleton_stop_list_and_image_construction_accept_one_real_stop() {
    let declaration = parsed(P::Color, "currentcolor");
    let CssKnownPropertyValueRef::Color(color) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    let stop = CssGradientColorStop::from_color(color.value().clone(), None);
    let list = CssColorStopList::try_new(vec![CssColorStopListItem::Stop(Box::new(stop))])
        .expect("Images4 stop-list contains one or more color stops");
    assert_eq!(list.items().len(), 1);
    let gradient = CssLinearGradient::new(None, list);
    let image = CssImage::try_new(CssImageValue::Gradient(CssGradient::Linear(gradient))).unwrap();
    assert_eq!(
        image.serialize_specified().unwrap(),
        "linear-gradient(currentcolor)"
    );
}
#[test]
fn checked_stop_list_still_rejects_empty_and_hint_only_or_endpoint_hints() {
    assert!(CssColorStopList::try_new(vec![]).is_none());
    let hint = CssSpecifiedLengthPercentage::try_from_component(
        CssComponentValue::try_token("50%").unwrap(),
    )
    .unwrap();
    assert!(CssColorStopList::try_new(vec![CssColorStopListItem::Hint(hint.clone())]).is_none());
    let declaration = parsed(P::BackgroundImage, "linear-gradient(red, blue)");
    let CssImageValue::Gradient(CssGradient::Linear(gradient)) = image(&declaration) else {
        panic!("linear")
    };
    for prefix in [true, false] {
        let mut list = gradient.stops().items().to_vec();
        if prefix {
            list.insert(0, CssColorStopListItem::Hint(hint.clone()));
        } else {
            list.push(CssColorStopListItem::Hint(hint.clone()));
        }
        assert!(CssColorStopList::try_new(list).is_none());
    }
}
#[test]
fn selected_gradients_compose_through_actual_image_consumers_and_nested_images() {
    // Specialized image consumers, not a second authored property inventory.
    for value in [
        "conic-gradient(red, blue)",
        "linear-gradient(in lab, red 10% 20%, blue)",
    ] {
        for property in [
            P::BackgroundImage,
            P::Background,
            P::ListStyleImage,
            P::ListStyle,
            P::BorderImageSource,
            P::BorderImage,
            P::MaskImage,
            P::Mask,
            P::MaskBorderSource,
            P::MaskBorder,
            P::ShapeOutside,
            P::Content,
        ] {
            for declaration in fronts(property, value) {
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    expand_declaration(&declaration).unwrap()
                else {
                    panic!("consumer expansion")
                };
                assert!(!values.items().is_empty());
                assert!(
                    values
                        .items()
                        .iter()
                        .all(|item| item.source().same_occurrence(&declaration))
                );
                let output = declaration.to_specified_css().unwrap();
                let reparsed = parse_style_attribute(&output);
                assert!(
                    reparsed.is_clean(),
                    "{output}: {:?}",
                    reparsed.diagnostics()
                );
                assert_eq!(reparsed.syntax()[0].to_specified_css().unwrap(), output);
            }
        }
        for nested in [
            format!("image-set({value} 1x)"),
            format!("cross-fade({value}, url(a.png))"),
            format!("light-dark({value}, none)"),
            format!("filter({value}, blur())"),
        ] {
            let declaration = parsed(P::BackgroundImage, &nested);
            assert!(CssImage::try_new(image(&declaration).clone()).is_ok());
            let output = declaration.to_specified_css().unwrap();
            assert_eq!(
                parse_style_attribute(&output).syntax()[0]
                    .to_specified_css()
                    .unwrap(),
                output
            );
        }
    }
}

#[test]
fn cursor_url_only_import_remains_narrow_when_gradient_image_support_expands() {
    // UI4 2026-01-20 §5.1.1 cursor-image selects URL/url-set, not <image>.
    for value in [
        "conic-gradient(red, blue)",
        "linear-gradient(in lab, red, blue)",
    ] {
        for authored in [
            format!("{value}, auto"),
            format!("image-set({value} 1x), auto"),
        ] {
            assert!(
                parse_property_value(
                    CssPropertyNameRef::Known(P::Cursor),
                    parse_component_values(&authored).unwrap(),
                    CssImportance::Normal
                )
                .is_err()
            );
        }
    }
}
#[test]
fn pending_substitution_is_whole_value_strict_reusable_and_preserves_source_identity() {
    for property in [
        P::BackgroundImage,
        P::MaskImage,
        P::BorderImageSource,
        P::ListStyleImage,
        P::ShapeOutside,
        P::Content,
    ] {
        let source = parsed(property, "bogus var(--gradient)");
        let before = source.clone();
        assert!(source.known().unwrap().substitution_dependent().is_some());
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for _ in 0..2 {
            for invalid in [
                "conic-gradient(red,)",
                "linear-gradient(in lab longer hue, red, blue)",
                "radial-gradient(red 0 1% 2%, blue)",
            ] {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(invalid).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ));
            }
            assert_eq!(
                handle
                    .reenter(parse_component_values("var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let replacement =
                parse_component_values("/*😀*/conic-gradient(from 0 in lab, red 0 50%, blue)")
                    .unwrap();
            let replacement_before = replacement.clone();
            let CssContributions::Longhands(values) = handle
                .reenter(replacement.clone())
                .expect("valid selected gradient replacement")
            else {
                panic!("completed")
            };
            for item in values.items() {
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
                assert!(item.ordinary_value().is_some());
            }
            assert_eq!(replacement, replacement_before);
            assert_eq!(source, before);
        }
    }
}
#[test]
fn css_wide_values_remain_distinct_from_pending_and_ordinary_gradients() {
    for property in [
        P::BackgroundImage,
        P::MaskImage,
        P::BorderImageSource,
        P::ListStyleImage,
    ] {
        for (text, keyword) in [
            ("inherit", CssGlobalKeyword::Inherit),
            ("initial", CssGlobalKeyword::Initial),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            for source in fronts(property, text) {
                assert_eq!(source.known().unwrap().global(), Some(keyword));
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    expand_declaration(&source).unwrap()
                else {
                    panic!("global expansion")
                };
                assert_eq!(
                    values.items()[0].value(),
                    CssContributionValueRef::Global(keyword)
                );
                assert!(values.items()[0].ordinary_value().is_none());
            }
        }
    }
}
#[test]
fn normalization_preserves_gradient_order_importance_pending_and_occurrence_provenance() {
    let report = parse_sheet(
        ".a{background-image:conic-gradient(red,blue)!important;background-image:linear-gradient(in lab,red 0 20%,blue);background-image:var(--g);background-image:unset}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    let CssRule::Style(style) = &report.syntax().rules()[0] else {
        panic!("style")
    };
    let limits = CssNormalizationLimits::try_new(0, 1, 4, 4).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), limits).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 4);
    for (order, (normalized, source)) in declarations
        .iter()
        .zip(style.declarations().iter())
        .enumerate()
    {
        assert_eq!(normalized.order(), order);
        assert!(normalized.source().same_occurrence(source));
        assert_eq!(normalized.source().importance(), source.importance());
        match normalized.expansion() {
            CssExpansion::Pending(_) => assert_eq!(order, 2),
            CssExpansion::Contributions(_) => assert_ne!(order, 2),
            _ => panic!("ordinary or pending gradient expansion"),
        }
    }
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 4, 3).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.declaration_order(), Some(3));
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 3
        }
    );
    assert_eq!(report, before);
    assert!(normalize_report_with_limits(&report, limits).is_ok());
}
#[test]
fn selected_gradient_serialization_limits_are_atomic_reusable_and_cumulative() {
    let source = parsed(
        P::BackgroundImage,
        "conic-gradient(from 45deg at left 10px top 20% in lab, red 0 25%, 50%, blue 75% 1turn)",
    );
    let before = source.clone();
    let specified = source.to_specified_css().unwrap();
    let unlimited = usize::MAX;
    assert_eq!(
        source
            .to_specified_css_with_limits(L::new(unlimited, unlimited, specified.len()))
            .unwrap(),
        specified
    );
    for (limit, kind) in [
        (L::new(0, unlimited, unlimited), K::InputNodeLimit),
        (L::new(unlimited, 0, unlimited), K::ProjectionNodeLimit),
        (
            L::new(unlimited, unlimited, specified.len() - 1),
            K::ByteLimit,
        ),
    ] {
        for _ in 0..2 {
            assert_eq!(
                source
                    .to_specified_css_with_limits(limit)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(source.to_specified_css().unwrap(), specified);
        assert_eq!(source, before);
    }
    let single = image(&source).serialize_specified().unwrap();
    let CssKnownPropertyValueRef::BackgroundImage(authored) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("image list")
    };
    let repeated = CssImageValueList::try_new(vec![
        authored.images().images()[0].clone(),
        authored.images().images()[0].clone(),
    ])
    .unwrap();
    assert_eq!(
        repeated.serialize_specified().unwrap(),
        format!("{single}, {single}")
    );
    // Each child's output fits separately; the composed list fails the total cap.
    assert!(
        image(&source)
            .serialize_specified_with_limits(L::new(unlimited, unlimited, single.len()))
            .is_ok()
    );
    assert_eq!(
        repeated
            .serialize_specified_with_limits(L::new(unlimited, unlimited, single.len()))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    assert_eq!(
        repeated.serialize_specified().unwrap(),
        format!("{single}, {single}")
    );
}
#[test]
fn unchanged_images3_two_stop_single_position_contract_still_round_trips() {
    for function in LINEAR.into_iter().chain(RADIAL) {
        round_trip(&format!("{function}(red -10%, 40%, blue 120%)"));
    }
}
#[test]
fn radial_shape_size_restrictions_survive_the_interpolation_extension() {
    for function in RADIAL {
        for prelude in [
            "circle 10%",
            "circle 10px 20px",
            "ellipse 10px",
            "circle -1px",
            "ellipse 10px -2px",
        ] {
            rejects(&format!("{function}({prelude} in lab, red, blue)"));
        }
    }
}

#[test]
fn checked_property_and_grammar_fronts_admit_the_selected_gradient_extensions() {
    // Exercise checks directly so a browser parser rejection cannot mask them.
    for text in [
        "conic-gradient(red, blue)",
        "linear-gradient(in lab, red, blue)",
        "radial-gradient(at block-start inline-end, red, blue)",
        "repeating-linear-gradient(red 10% 20%, blue)",
        "repeating-radial-gradient(red)",
    ] {
        let components = parse_component_values(text).unwrap();
        let before = components.clone();
        for grammar in [false, true] {
            let declaration = if grammar {
                parse_property_value_for_grammar(
                    P::BackgroundImage.grammar(),
                    components.clone(),
                    CssImportance::Important,
                )
            } else {
                parse_property_value(
                    CssPropertyNameRef::Known(P::BackgroundImage),
                    components.clone(),
                    CssImportance::Important,
                )
            }
            .unwrap_or_else(|error| panic!("selected checked gradient {text}: {error:?}"));
            assert_eq!(declaration.value_components(), &before);
            assert!(declaration.position().is_none());
            assert!(declaration.known().unwrap().property_value().is_some());
            assert!(CssImage::try_new(image(&declaration).clone()).is_ok());
        }
        assert_eq!(components, before);
    }
}

#[test]
fn checked_interpolation_admission_reuses_the_real_property_parser() {
    for function in LINEAR.into_iter().chain(RADIAL) {
        let text = format!("{function}(in lab, red, blue)");
        let checked = parse_property_value(
            CssPropertyNameRef::Known(P::BackgroundImage),
            parse_component_values(&text).unwrap(),
            CssImportance::Normal,
        )
        .expect("Images4 explicit interpolation is valid at checked boundary");
        assert_eq!(image(&checked).serialize_specified().unwrap(), text);
    }
}

#[test]
fn checked_radial_at_admission_uses_the_full_imported_position_owner() {
    for function in RADIAL {
        let text = format!("{function}(at block-start inline-end, red, blue)");
        let checked = parse_property_value_for_grammar(
            P::BackgroundImage.grammar(),
            parse_component_values(&text).unwrap(),
            CssImportance::Normal,
        )
        .expect("specific Values5 at-position import permits named flow axes");
        assert_eq!(image(&checked).serialize_specified().unwrap(), text);
    }
}

#[test]
fn specified_output_keeps_both_positions_and_nondefault_interpolation_semantics() {
    // Exact output oracles use non-default positions and lab (not the default
    // oklab), so loss of an authored field cannot pass a stable-reparse check.
    for function in LINEAR.into_iter().chain(RADIAL).chain(["conic-gradient"]) {
        let stops = if function == "conic-gradient" {
            "red -90deg 25%, 50%, blue 270deg 150%"
        } else {
            "red -10px 20%, 40%, blue 80% 120px"
        };
        let expected = format!("{function}(in lab, {stops})");
        for declaration in fronts(P::BackgroundImage, &expected) {
            assert_eq!(image(&declaration).serialize_specified().unwrap(), expected);
            assert_eq!(
                declaration.to_specified_css().unwrap(),
                format!("background-image: {expected} !important;")
            );
        }
    }
}

#[test]
fn named_and_relative_flow_positions_keep_the_adopted_symbolic_output() {
    for function in RADIAL.into_iter().chain(["conic-gradient"]) {
        for (input, canonical) in [
            ("inline-end block-start", "block-start inline-end"),
            (
                "inline-end 20% block-start 10px",
                "block-start 10px inline-end 20%",
            ),
            ("start end", "start end"),
            ("end start", "end start"),
            ("start 10px end 20%", "start 10px end 20%"),
        ] {
            let declaration = parsed(
                P::BackgroundImage,
                &format!("{function}(at {input}, red, blue)"),
            );
            assert_eq!(
                image(&declaration).serialize_specified().unwrap(),
                format!("{function}(at {canonical}, red, blue)")
            );
        }
    }
}

#[test]
fn existing_line_stop_calculations_remain_symbolic_without_geometry() {
    for function in LINEAR.into_iter().chain(RADIAL) {
        for declaration in fronts(
            P::BackgroundImage,
            &format!("{function}(red calc(10px + 5%) calc(20px + 10%), calc(30px + 15%), blue)"),
        ) {
            let output = image(&declaration).serialize_specified().unwrap();
            assert_eq!(
                output,
                format!("{function}(red calc(10px + 5%) calc(20px + 10%), calc(30px + 15%), blue)")
            );
        }
    }
}

#[test]
fn double_stop_payload_keeps_original_first_position_hint_and_color_occurrences() {
    let source =
        "/*😀*/background-image:linear-gradient(red -10px 20%, 40%, blue 80% 120px)!important";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = &report.syntax()[0];
    let CssImageValue::Gradient(CssGradient::Linear(gradient)) = image(declaration) else {
        panic!("linear")
    };
    let [
        CssColorStopListItem::Stop(first),
        CssColorStopListItem::Hint(hint),
        CssColorStopListItem::Stop(last),
    ] = gradient.stops().items()
    else {
        panic!("authored double-position stops must not be expanded or fixed up")
    };
    assert_eq!(first.color().to_specified_css().unwrap(), "red");
    assert_eq!(last.color().to_specified_css().unwrap(), "blue");
    for (value, text) in [
        (first.position().unwrap(), "-10px"),
        (hint, "40%"),
        (last.position().unwrap(), "80%"),
    ] {
        let CssValueOrigin::Parsed(origin) = value.literal_component().unwrap().origin() else {
            panic!("original parsed numeric occurrence")
        };
        assert_eq!(origin.source().as_str(), source);
        let start = source.find(text).unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), start);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            start + text.len()
        );
        assert_eq!(
            origin.span().start().column().value() as usize,
            source[..start].encode_utf16().count()
        );
    }
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(
        declaration.position().unwrap().byte_offset().value(),
        source.find("background-image").unwrap()
    );
}

#[test]
fn singleton_gradient_lists_share_exact_graph_work_caps_across_siblings() {
    // Existing public accounting contract: gradient 1 + stop-list 1 + stop 1
    // + color 1; enclosing image-list 1. Omitted optional prelude has no node.
    let source = parsed(P::BackgroundImage, "linear-gradient(red)");
    let single = image(&source).clone();
    let list = CssImageValueList::try_new(vec![single.clone(), single]).unwrap();
    let expected = "linear-gradient(red), linear-gradient(red)";
    assert_eq!(
        list.serialize_specified_with_limits(L::new(9, 9, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(8, 9, expected.len()), K::InputNodeLimit),
        (L::new(9, 8, expected.len()), K::ProjectionNodeLimit),
        (L::new(9, 9, expected.len() - 1), K::ByteLimit),
    ] {
        for _ in 0..2 {
            assert_eq!(
                list.serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        assert_eq!(list.serialize_specified().unwrap(), expected);
    }
    assert!(
        image(&source)
            .serialize_specified_with_limits(L::new(4, 4, "linear-gradient(red)".len()))
            .is_ok()
    );
    assert_eq!(
        list.serialize_specified_with_limits(L::new(4, 9, expected.len()))
            .unwrap_err()
            .kind(),
        K::InputNodeLimit
    );
}
