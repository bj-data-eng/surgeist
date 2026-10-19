#![forbid(unsafe_code)]
//! Exact nonnegative authored scalars through existing public parsing boundaries.
//!
//! Exact decimal retention is Surgeist's selected authored-value contract.
//! CSS Values 4 permits implementation-defined supported precision and range;
//! these tests do not claim a universal CSS prohibition on finite execution domains.
//! Ordinary negatives are invalid; range checks inside actual math remain downstream.
//!
//! Selected dated publications (the repository's pinned cutoff remains unchanged):
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-range
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-image-slice
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-image-width
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-image-outset
//! https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/#supported-filter-functions
//! https://www.w3.org/TR/2023/WD-css-animations-1-20230302/#animation-iteration-count

use surgeist_css::{
    CssAnimationIterationCount, CssBorderImageRepeatKeyword, CssBorderImageSliceComponent,
    CssBorderImageWidthComponent, CssDeclaration, CssErrorCode, CssFilter, CssFilterAmount,
    CssFilterFunction, CssKnownProperty, CssKnownPropertyValueRef, CssRecoveryAction,
    CssRecoveryDiagnostic, CssRule, ErrorKind, parse_sheet, parse_style_attribute,
};

const AMOUNT_FUNCTIONS: [&str; 7] = [
    "brightness",
    "contrast",
    "grayscale",
    "invert",
    "opacity",
    "saturate",
    "sepia",
];

fn assert_typed_property(declaration: &CssDeclaration, expected: CssKnownProperty, source: &str) {
    let known = declaration.known().expect("retained known declaration");
    assert_eq!(known.property(), expected, "{source}");
    let value = known
        .property_value()
        .expect("ordinary typed property value");
    assert!(
        matches!(
            (expected, value),
            (
                CssKnownProperty::BorderImage,
                CssKnownPropertyValueRef::BorderImage(_)
            ) | (
                CssKnownProperty::BorderImageSlice,
                CssKnownPropertyValueRef::BorderImageSlice(_)
            ) | (
                CssKnownProperty::BorderImageWidth,
                CssKnownPropertyValueRef::BorderImageWidth(_)
            ) | (
                CssKnownProperty::BorderImageOutset,
                CssKnownPropertyValueRef::BorderImageOutset(_)
            ) | (
                CssKnownProperty::Filter,
                CssKnownPropertyValueRef::Filter(_)
            ) | (
                CssKnownProperty::BackdropFilter,
                CssKnownPropertyValueRef::BackdropFilter(_)
            ) | (
                CssKnownProperty::AnimationIterationCount,
                CssKnownPropertyValueRef::AnimationIterationCount(_)
            ) | (
                CssKnownProperty::Animation,
                CssKnownPropertyValueRef::Animation(_)
            ) | (CssKnownProperty::Width, CssKnownPropertyValueRef::Width(_))
                | (CssKnownProperty::Color, CssKnownPropertyValueRef::Color(_))
        ),
        "{source}: wrong ordinary property payload: {value:?}"
    );
}

fn assert_declarations(
    source: &str,
    declarations: &[CssDeclaration],
    diagnostics: &[CssRecoveryDiagnostic],
    expected: CssKnownProperty,
    admitted: bool,
) {
    if admitted {
        assert!(
            diagnostics.is_empty(),
            "{source}: expected clean admission, got {diagnostics:?}"
        );
        assert_eq!(
            declarations.len(),
            3,
            "{source}: expected numeric value and both siblings"
        );
        assert_typed_property(&declarations[0], CssKnownProperty::Width, source);
        assert_typed_property(&declarations[1], expected, source);
        assert_typed_property(&declarations[2], CssKnownProperty::Color, source);
    } else {
        let [diagnostic] = diagnostics else {
            panic!("{source}: expected one numeric rejection, got {diagnostics:?}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}"
        );
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropDeclaration,
            "{source}"
        );
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected a property-value diagnostic");
        };
        assert_eq!(
            detail.property(),
            expected,
            "{source}: rejected the wrong property"
        );
        assert_eq!(
            declarations.len(),
            2,
            "{source}: invalid declaration must be dropped alone"
        );
        assert_typed_property(&declarations[0], CssKnownProperty::Width, source);
        assert_typed_property(&declarations[1], CssKnownProperty::Color, source);
    }
}

fn assert_case(declaration: &str, expected: CssKnownProperty, admitted: bool) {
    let source = format!("width: 2px; {declaration}; color: red");
    let report = parse_style_attribute(&source);
    assert_eq!(
        report.is_clean(),
        admitted,
        "style attribute: {source}: {:?}",
        report.diagnostics()
    );
    assert_declarations(
        &source,
        report.syntax().as_slice(),
        report.diagnostics(),
        expected,
        admitted,
    );

    let sheet_source = format!(".subject {{ {source} }}");
    let sheet = parse_sheet(&sheet_source);
    assert_eq!(
        sheet.is_clean(),
        admitted,
        "stylesheet: {sheet_source}: {:?}",
        sheet.diagnostics()
    );
    let [CssRule::Style(rule)] = sheet.syntax().rules() else {
        panic!("{sheet_source}: expected the retained containing style rule");
    };
    assert_declarations(
        &sheet_source,
        rule.declarations().as_slice(),
        sheet.diagnostics(),
        expected,
        admitted,
    );
}

fn assert_clean(declaration: &str, expected: CssKnownProperty) {
    assert_case(declaration, expected, true);
}

fn assert_rejected(declaration: &str, expected: CssKnownProperty) {
    assert_case(declaration, expected, false);
}

#[test]
fn border_image_longhands_admit_huge_ordinary_scalars_in_every_number_and_percentage_role() {
    for (declaration, property) in [
        (
            "border-image-slice: 1e999",
            CssKnownProperty::BorderImageSlice,
        ),
        (
            "border-image-slice: 1e999%",
            CssKnownProperty::BorderImageSlice,
        ),
        (
            "border-image-width: 1e999",
            CssKnownProperty::BorderImageWidth,
        ),
        (
            "border-image-outset: 1e999",
            CssKnownProperty::BorderImageOutset,
        ),
    ] {
        assert_clean(declaration, property);
    }
}

#[test]
fn border_image_shorthand_admits_huge_ordinary_scalars_in_every_scalar_role() {
    for value in [
        "1e999 / 1 / 0",
        "1e999% / 1 / 0",
        "10 / 1e999 / 0",
        "10 / 1 / 1e999",
    ] {
        assert_clean(
            &format!("border-image: {value}"),
            CssKnownProperty::BorderImage,
        );
    }
}

#[test]
fn every_filter_amount_function_admits_huge_numbers_and_percentages_in_both_properties() {
    for (name, property) in [
        ("filter", CssKnownProperty::Filter),
        ("backdrop-filter", CssKnownProperty::BackdropFilter),
    ] {
        for function in AMOUNT_FUNCTIONS {
            for amount in ["1e999", "1e999%"] {
                assert_clean(&format!("{name}: {function}({amount})"), property);
            }
        }
    }
}

#[test]
fn animation_iteration_longhand_and_shorthand_admit_huge_ordinary_numbers() {
    assert_clean(
        "animation-iteration-count: 1e999",
        CssKnownProperty::AnimationIterationCount,
    );
    assert_clean(
        "animation-iteration-count: infinite, 1e999, .5",
        CssKnownProperty::AnimationIterationCount,
    );
    assert_clean("animation: fade 1s 1e999", CssKnownProperty::Animation);
}

#[test]
fn border_image_longhands_reject_tiny_negative_nonzero_scalars_and_preserve_siblings() {
    for (declaration, property) in [
        (
            "border-image-slice: -1e-999",
            CssKnownProperty::BorderImageSlice,
        ),
        (
            "border-image-slice: -1e-999%",
            CssKnownProperty::BorderImageSlice,
        ),
        (
            "border-image-width: -1e-999",
            CssKnownProperty::BorderImageWidth,
        ),
        (
            "border-image-outset: -1e-999",
            CssKnownProperty::BorderImageOutset,
        ),
    ] {
        assert_rejected(declaration, property);
    }
}

#[test]
fn border_image_shorthand_rejects_tiny_negative_nonzero_scalars_in_every_scalar_role() {
    for value in [
        "-1e-999 / 1 / 0",
        "-1e-999% / 1 / 0",
        "10 / -1e-999 / 0",
        "10 / 1 / -1e-999",
    ] {
        assert_rejected(
            &format!("border-image: {value}"),
            CssKnownProperty::BorderImage,
        );
    }
}

#[test]
fn every_filter_amount_function_rejects_tiny_negative_nonzero_amounts_and_preserves_siblings() {
    for (name, property) in [
        ("filter", CssKnownProperty::Filter),
        ("backdrop-filter", CssKnownProperty::BackdropFilter),
    ] {
        for function in AMOUNT_FUNCTIONS {
            for amount in ["-1e-999", "-1e-999%"] {
                assert_rejected(&format!("{name}: {function}({amount})"), property);
            }
        }
    }
}

#[test]
fn animation_iteration_longhand_and_shorthand_reject_tiny_negative_nonzero_counts() {
    assert_rejected(
        "animation-iteration-count: -1e-999",
        CssKnownProperty::AnimationIterationCount,
    );
    assert_rejected(
        "animation-iteration-count: 2, -1e-999, infinite",
        CssKnownProperty::AnimationIterationCount,
    );
    assert_rejected("animation: fade 1s -1e-999", CssKnownProperty::Animation);
}

#[test]
fn genuine_signed_zero_with_enormous_exponents_is_admitted_in_every_scalar_role() {
    // These exponents exceed i128. A zero coefficient still denotes exact zero.
    for zero in [
        "-0e9999999999999999999999999999999999999999",
        "+0e9999999999999999999999999999999999999999",
        "-0e-9999999999999999999999999999999999999999",
    ] {
        for (name, property, suffix) in [
            ("border-image-slice", CssKnownProperty::BorderImageSlice, ""),
            (
                "border-image-slice",
                CssKnownProperty::BorderImageSlice,
                "%",
            ),
            ("border-image-width", CssKnownProperty::BorderImageWidth, ""),
            (
                "border-image-outset",
                CssKnownProperty::BorderImageOutset,
                "",
            ),
            (
                "animation-iteration-count",
                CssKnownProperty::AnimationIterationCount,
                "",
            ),
        ] {
            assert_clean(&format!("{name}: {zero}{suffix}"), property);
        }
        assert_clean(
            &format!("border-image: {zero}% / {zero} / {zero}"),
            CssKnownProperty::BorderImage,
        );
        assert_clean(
            &format!("animation: fade 1s {zero}"),
            CssKnownProperty::Animation,
        );
        for (name, property) in [
            ("filter", CssKnownProperty::Filter),
            ("backdrop-filter", CssKnownProperty::BackdropFilter),
        ] {
            for function in AMOUNT_FUNCTIONS {
                for suffix in ["", "%"] {
                    assert_clean(&format!("{name}: {function}({zero}{suffix})"), property);
                }
            }
        }
    }
}

#[test]
fn values_above_one_or_one_hundred_percent_remain_authored_in_filter_and_slice_domains() {
    for (name, property) in [
        ("filter", CssKnownProperty::Filter),
        ("backdrop-filter", CssKnownProperty::BackdropFilter),
    ] {
        for function in AMOUNT_FUNCTIONS {
            for amount in ["2.5", "250%"] {
                assert_clean(&format!("{name}: {function}({amount})"), property);
            }
        }
    }
    assert_clean(
        "border-image-slice: 250% fill",
        CssKnownProperty::BorderImageSlice,
    );
}

#[test]
fn ordinary_border_image_arity_fill_and_auto_keep_their_typed_aggregate_roles() {
    for value in ["1", "1 2", "1 2 3", "1 2 3 4"] {
        for (name, property) in [
            ("border-image-slice", CssKnownProperty::BorderImageSlice),
            ("border-image-width", CssKnownProperty::BorderImageWidth),
            ("border-image-outset", CssKnownProperty::BorderImageOutset),
        ] {
            assert_clean(&format!("{name}: {value}"), property);
        }
    }
    let report =
        parse_style_attribute("border-image-slice: fill 10 20%; border-image-width: auto 2");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderImageSlice(slice) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed slice");
    };
    assert!(slice.slice().fill());
    let [top, right, bottom, left] = slice.slice().values();
    assert!(matches!(top, CssBorderImageSliceComponent::Number(_)));
    assert!(matches!(right, CssBorderImageSliceComponent::Percentage(_)));
    assert_eq!(top, bottom);
    assert_eq!(right, left);
    let CssKnownPropertyValueRef::BorderImageWidth(width) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed width");
    };
    let [top, right, bottom, left] = width.widths().values();
    assert!(matches!(top, CssBorderImageWidthComponent::Auto));
    assert!(matches!(right, CssBorderImageWidthComponent::Number(_)));
    assert_eq!(top, bottom);
    assert_eq!(right, left);
}

#[test]
fn ordinary_border_image_shorthand_preserves_explicit_members_and_omissions() {
    let report =
        parse_style_attribute("border-image: 10 fill; border-image: 10 / auto / 2 round space");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderImage(first) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("first shorthand");
    };
    let first = first.border_image();
    assert!(first.slice().unwrap().fill());
    assert!(first.source().is_none());
    assert!(first.width().is_none());
    assert!(first.outset().is_none());
    assert!(first.repeat().is_none());
    let CssKnownPropertyValueRef::BorderImage(second) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("second shorthand");
    };
    let second = second.border_image();
    assert!(second.source().is_none());
    assert!(second.slice().is_some());
    assert!(matches!(
        second.width().unwrap().values()[0],
        CssBorderImageWidthComponent::Auto
    ));
    assert!(second.outset().is_some());
    assert_eq!(
        second.repeat().unwrap().horizontal(),
        CssBorderImageRepeatKeyword::Round
    );
    assert_eq!(
        second.repeat().unwrap().vertical(),
        CssBorderImageRepeatKeyword::Space
    );
}

#[test]
fn omitted_filter_amounts_preserve_all_seven_function_identities_and_order() {
    let functions = AMOUNT_FUNCTIONS.map(|name| format!("{name}()")).join(" ");
    for name in ["filter", "backdrop-filter"] {
        let source = format!("{name}: {functions}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let value = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap();
        let filter = match value {
            CssKnownPropertyValueRef::Filter(value) => value.value(),
            CssKnownPropertyValueRef::BackdropFilter(value) => value.value(),
            _ => panic!("{source}: typed filter"),
        };
        let CssFilter::Functions(functions) = filter else {
            panic!("{source}: function list");
        };
        assert!(
            matches!(
                functions.functions(),
                [
                    CssFilterFunction::Brightness(CssFilterAmount::Default),
                    CssFilterFunction::Contrast(CssFilterAmount::Default),
                    CssFilterFunction::Grayscale(CssFilterAmount::Default),
                    CssFilterFunction::Invert(CssFilterAmount::Default),
                    CssFilterFunction::Opacity(CssFilterAmount::Default),
                    CssFilterFunction::Saturate(CssFilterAmount::Default),
                    CssFilterFunction::Sepia(CssFilterAmount::Default),
                ]
            ),
            "{source}"
        );
    }
}

#[test]
fn ordinary_animation_fractional_zero_infinite_counts_and_omission_keep_list_order() {
    assert_clean(
        "animation-iteration-count: 2.5, 0, infinite",
        CssKnownProperty::AnimationIterationCount,
    );
    assert_clean(
        "animation: fade 1s 2.5, other 2s infinite",
        CssKnownProperty::Animation,
    );
    let report =
        parse_style_attribute("animation-iteration-count: 2.5, 0, infinite; animation: fade 1s");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::AnimationIterationCount(counts) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed counts");
    };
    assert!(matches!(
        counts.iteration_counts().values(),
        [
            CssAnimationIterationCount::Number(_),
            CssAnimationIterationCount::Number(_),
            CssAnimationIterationCount::Infinite
        ]
    ));
    let CssKnownPropertyValueRef::Animation(animation) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed animation");
    };
    assert_eq!(animation.animations().values().len(), 1);
    assert!(
        animation.animations().values()[0]
            .iteration_count()
            .is_none()
    );
}

#[test]
fn border_image_dimension_arity_separator_and_shorthand_coupling_errors_drop_only_the_declaration()
{
    for (declaration, property) in [
        (
            "border-image-slice: 1px",
            CssKnownProperty::BorderImageSlice,
        ),
        (
            "border-image-slice: 1, 2",
            CssKnownProperty::BorderImageSlice,
        ),
        (
            "border-image-slice: 1 2 3 4 5",
            CssKnownProperty::BorderImageSlice,
        ),
        (
            "border-image-slice: 1 fill fill",
            CssKnownProperty::BorderImageSlice,
        ),
        ("border-image-width: 1s", CssKnownProperty::BorderImageWidth),
        (
            "border-image-width: 1, 2",
            CssKnownProperty::BorderImageWidth,
        ),
        (
            "border-image-width: 1 2 3 4 5",
            CssKnownProperty::BorderImageWidth,
        ),
        (
            "border-image-outset: 1%",
            CssKnownProperty::BorderImageOutset,
        ),
        (
            "border-image-outset: auto",
            CssKnownProperty::BorderImageOutset,
        ),
        (
            "border-image-outset: 1 2 3 4 5",
            CssKnownProperty::BorderImageOutset,
        ),
        (
            "border-image: url(frame.png) / 2",
            CssKnownProperty::BorderImage,
        ),
        (
            "border-image: 10 / 1 / 2 / 3",
            CssKnownProperty::BorderImage,
        ),
    ] {
        assert_rejected(declaration, property);
    }
}

#[test]
fn every_filter_amount_function_rejects_dimensions_and_extra_or_comma_separated_operands() {
    for (name, property) in [
        ("filter", CssKnownProperty::Filter),
        ("backdrop-filter", CssKnownProperty::BackdropFilter),
    ] {
        for function in AMOUNT_FUNCTIONS {
            for amount in ["1px", "1 2", "1, 2"] {
                assert_rejected(&format!("{name}: {function}({amount})"), property);
            }
        }
    }
}

#[test]
fn animation_iteration_domains_and_comma_list_grammar_reject_invalid_values() {
    for value in ["1%", "1px", "1 2", "2,", "2,,3", "-1"] {
        assert_rejected(
            &format!("animation-iteration-count: {value}"),
            CssKnownProperty::AnimationIterationCount,
        );
    }
    for value in ["fade 1s 2%", "fade 1s 2 3", "fade 1s,"] {
        assert_rejected(&format!("animation: {value}"), CssKnownProperty::Animation);
    }
}

#[test]
fn actual_negative_number_and_percentage_math_is_admitted_without_authored_range_evaluation() {
    for (declaration, property) in [
        (
            "border-image-slice: calc(-1)",
            CssKnownProperty::BorderImageSlice,
        ),
        (
            "border-image-slice: calc(-1%)",
            CssKnownProperty::BorderImageSlice,
        ),
        (
            "border-image-width: calc(-1)",
            CssKnownProperty::BorderImageWidth,
        ),
        (
            "border-image-outset: calc(-1)",
            CssKnownProperty::BorderImageOutset,
        ),
        (
            "border-image: calc(-1%) / calc(-1) / calc(-1)",
            CssKnownProperty::BorderImage,
        ),
        (
            "animation-iteration-count: calc(-1)",
            CssKnownProperty::AnimationIterationCount,
        ),
        ("animation: fade 1s calc(-1)", CssKnownProperty::Animation),
    ] {
        assert_clean(declaration, property);
    }
    for (name, property) in [
        ("filter", CssKnownProperty::Filter),
        ("backdrop-filter", CssKnownProperty::BackdropFilter),
    ] {
        for function in AMOUNT_FUNCTIONS {
            for amount in ["calc(-1)", "calc(-1%)"] {
                assert_clean(&format!("{name}: {function}({amount})"), property);
            }
        }
    }
}

#[test]
fn mathematical_roots_from_other_numeric_domains_are_rejected_with_sibling_recovery() {
    for (declaration, property) in [
        (
            "border-image-slice: calc(1px)",
            CssKnownProperty::BorderImageSlice,
        ),
        (
            "border-image-width: calc(1s)",
            CssKnownProperty::BorderImageWidth,
        ),
        (
            "border-image-outset: calc(1%)",
            CssKnownProperty::BorderImageOutset,
        ),
        (
            "animation-iteration-count: calc(1%)",
            CssKnownProperty::AnimationIterationCount,
        ),
        ("animation: fade 1s calc(1%)", CssKnownProperty::Animation),
    ] {
        assert_rejected(declaration, property);
    }
    for (name, property) in [
        ("filter", CssKnownProperty::Filter),
        ("backdrop-filter", CssKnownProperty::BackdropFilter),
    ] {
        for function in AMOUNT_FUNCTIONS {
            assert_rejected(&format!("{name}: {function}(calc(1px))"), property);
        }
    }
}
