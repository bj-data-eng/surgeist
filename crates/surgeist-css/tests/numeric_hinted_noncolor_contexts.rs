#![forbid(unsafe_code)]
//! Existing percentage-permitting consumers must accept Number results with
//! unresolved hints without evaluating the percentage basis.
//! Transforms 2 WD 2021-11-09 §12.2 admits percentages in scale3d/scaleZ;
//! the selected Level 1 scale/scaleX/scaleY Number grammar remains separate.
//! Filter Effects 1 WD 2018-12-18 §6.1 admits number-percentage amounts.
//! Backgrounds 3 CRD 2024-03-11 §§5.2–5.3 and CSS2 §10.8 admit percentages.
//! https://www.w3.org/TR/2021/WD-css-transforms-2-20211109/#three-d-transform-functions
//! https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/#FilterProperty
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-image-slice
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#border-image-width
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/visudet.html#line-height
//! https://www.w3.org/TR/2024/WD-css-typed-om-1-20240321/#cssnumericvalue-match

use surgeist_css::*;

const HINTED: &str = "calc((1px + 1%) / 1px)";
const PURE: &str = "calc(1px / 1px)";
const AMOUNTS: [&str; 7] = [
    "brightness",
    "contrast",
    "grayscale",
    "invert",
    "opacity",
    "saturate",
    "sepia",
];

fn components(text: &str) -> CssComponentValues {
    parse_component_values(text).unwrap()
}

fn parsed(property: CssKnownProperty, text: &str) -> CssDeclaration {
    let source = format!(
        "/*😀*/\r\n{}:{text}!important;\r\nheight:1px",
        property.canonical_name()
    );
    let report = parse_style_attribute(&source);
    assert!(
        report.is_clean(),
        "{}:{text}: {:?}",
        property.canonical_name(),
        report.diagnostics()
    );
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        validate_style_attribute(&source),
        Ok(report.syntax().clone())
    );
    let value = report.syntax()[0].clone();
    assert_eq!(value.known().unwrap().property(), property);
    assert!(value.known().unwrap().property_value().is_some());
    assert_eq!(value.importance(), CssImportance::Important);
    // Admission retains the entire expression and its unresolved percentage;
    // no scalar value or image/font/geometry basis is supplied by this test.
    assert_eq!(value.value_components().serialize().unwrap().as_css(), text);
    let CssValueOrigin::Parsed(origin) = value.value_components().items()[0].origin() else {
        panic!("parsed target provenance")
    };
    assert_eq!(origin.source().as_str(), source);
    let start = source.find(':').unwrap() + 1;
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().start().line().value(), 1);
    assert_height_provenance(&report.syntax()[1], &source);
    value
}

fn assert_height_provenance(value: &CssDeclaration, source: &str) {
    assert_eq!(value.known().unwrap().property(), CssKnownProperty::Height);
    let CssValueOrigin::Parsed(origin) = value.value_components().items()[0].origin() else {
        panic!("parsed sibling provenance")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.rfind("1px").unwrap()
    );
    assert_eq!(origin.span().start().line().value(), 2);
}

fn checked(property: CssKnownProperty, text: &str) -> CssDeclaration {
    let original = components(text);
    let result = parse_property_value(
        CssPropertyNameRef::Known(property),
        original.clone(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {}:{text}: {error:?}", property.canonical_name()));
    assert_eq!(result.value_components(), &original);
    assert!(result.position().is_none());
    result
}

fn admits(property: CssKnownProperty, text: &str) {
    let parsed = parsed(property, text);
    let checked = checked(property, text);
    if !has_expansion(property) {
        assert_unsupported_expansion(property, &parsed);
        assert_unsupported_expansion(property, &checked);
        assert_unsupported_expansion(property, &checked_pending(property));
        return;
    }
    let CssExpansion::Contributions(CssContributions::Longhands(expected)) =
        expand_declaration(&checked).unwrap()
    else {
        panic!("checked longhand")
    };
    let [expected] = expected.items() else {
        panic!("one checked contribution")
    };
    let CssExpansion::Contributions(CssContributions::Longhands(parsed_items)) =
        expand_declaration(&parsed).unwrap()
    else {
        panic!("parsed longhand")
    };
    assert_eq!(parsed_items.items().len(), 1);
    assert_eq!(parsed_items.items()[0].property(), property);
    assert!(parsed_items.items()[0].source().same_occurrence(&parsed));

    let pending_source = checked_pending(property);
    let CssExpansion::Pending(pending) = expand_declaration(&pending_source).unwrap() else {
        panic!("pending")
    };
    let replacement = components(text);
    let before = replacement.clone();
    let CssContributions::Longhands(actual) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("replacement longhand")
    };
    let [actual] = actual.items() else {
        panic!("one replacement contribution")
    };
    assert_eq!(actual.property(), property);
    assert_eq!(actual.value(), expected.value());
    assert!(actual.source().same_occurrence(&pending_source));
    assert_eq!(actual.source().importance(), CssImportance::Important);
    assert_eq!(actual.replacement_components(), Some(&replacement));
    assert_eq!(replacement, before);
}

fn checked_pending(property: CssKnownProperty) -> CssDeclaration {
    checked(property, "var(--numeric)")
}

fn has_expansion(property: CssKnownProperty) -> bool {
    // Independent oracle from properties.rs expansion annotations, consumed by
    // expansion.rs: Transform and Scale are parsed but not selected for expansion.
    match property {
        CssKnownProperty::Transform | CssKnownProperty::Scale => false,
        CssKnownProperty::Filter
        | CssKnownProperty::BorderImageSlice
        | CssKnownProperty::BorderImageWidth
        | CssKnownProperty::LineHeight
        | CssKnownProperty::VoiceBalance => true,
        _ => panic!("property outside this target's explicit expansion boundary"),
    }
}

fn assert_unsupported_expansion(property: CssKnownProperty, source: &CssDeclaration) {
    assert_eq!(
        expand_declaration(source).unwrap_err().kind(),
        &CssExpansionErrorKind::UnsupportedProperty(property)
    );
}

#[test]
fn transform_scale3d_and_scalez_accept_hinted_numbers() {
    for text in [
        format!("scale3d({HINTED}, 1, 1)"),
        format!("scaleZ({HINTED})"),
    ] {
        admits(CssKnownProperty::Transform, &text);
    }
}

#[test]
fn filter_amount_functions_accept_hinted_numbers() {
    for function in AMOUNTS {
        admits(CssKnownProperty::Filter, &format!("{function}({HINTED})"));
    }
}

#[test]
fn border_image_slice_accepts_hinted_number_offsets() {
    admits(CssKnownProperty::BorderImageSlice, HINTED);
    admits(
        CssKnownProperty::BorderImageSlice,
        &format!("{HINTED} 20% fill"),
    );
}

#[test]
fn border_image_width_accepts_hinted_number_multipliers() {
    admits(CssKnownProperty::BorderImageWidth, HINTED);
    admits(
        CssKnownProperty::BorderImageWidth,
        &format!("{HINTED} auto 2px 20%"),
    );
}

#[test]
fn line_height_accepts_hinted_number_without_resolving_font_or_percentage_basis() {
    admits(CssKnownProperty::LineHeight, HINTED);
}

#[test]
fn unhinted_dimension_cancellation_remains_valid_in_all_selected_consumers() {
    for text in [format!("scale3d({PURE}, 1, 1)"), format!("scaleZ({PURE})")] {
        admits(CssKnownProperty::Transform, &text);
    }
    for function in AMOUNTS {
        admits(CssKnownProperty::Filter, &format!("{function}({PURE})"));
    }
    for property in [
        CssKnownProperty::BorderImageSlice,
        CssKnownProperty::BorderImageWidth,
        CssKnownProperty::LineHeight,
    ] {
        admits(property, PURE);
    }
}

#[test]
fn ordinary_percentages_demonstrate_the_selected_consumers_percentage_permission() {
    for text in ["scale3d(50%, 1, 1)", "scaleZ(50%)"] {
        admits(CssKnownProperty::Transform, text);
    }
    for function in AMOUNTS {
        admits(CssKnownProperty::Filter, &format!("{function}(50%)"));
    }
    for property in [
        CssKnownProperty::BorderImageSlice,
        CssKnownProperty::BorderImageWidth,
        CssKnownProperty::LineHeight,
    ] {
        admits(property, "50%");
    }
}

fn rejects_atomically(property: CssKnownProperty, text: &str) {
    let source = format!(
        "/*😀*/\r\n{}:{text}!important;\r\nheight:1px",
        property.canonical_name()
    );
    let report = parse_style_attribute(&source);
    assert_eq!(
        report.syntax().len(),
        1,
        "{}:{text}",
        property.canonical_name()
    );
    assert_height_provenance(&report.syntax()[0], &source);
    assert_eq!(report.diagnostics().len(), 1, "{text}");
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    assert_eq!(
        validate_style_attribute(&source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components(text),
            CssImportance::Normal
        )
        .is_err()
    );
    let pending_source = checked_pending(property);
    if !has_expansion(property) {
        assert_unsupported_expansion(property, &pending_source);
        return;
    }
    let CssExpansion::Pending(pending) = expand_declaration(&pending_source).unwrap() else {
        panic!("pending")
    };
    assert!(matches!(
        pending.reenter(components(text)).unwrap_err().kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    assert!(pending.source().same_occurrence(&pending_source));
}

#[test]
fn pure_number_consumers_and_transform_functions_keep_hint_rejection() {
    rejects_atomically(CssKnownProperty::VoiceBalance, HINTED);
    admits(CssKnownProperty::VoiceBalance, PURE);
    for function in ["scale", "scaleX", "scaleY"] {
        rejects_atomically(
            CssKnownProperty::Transform,
            &format!("{function}({HINTED})"),
        );
        admits(CssKnownProperty::Transform, &format!("{function}({PURE})"));
    }
    // Independent scale remains outside the selected supported property boundary.
    rejects_atomically(CssKnownProperty::Scale, HINTED);
}

#[test]
fn incompatible_amounts_drop_whole_declarations_without_losing_the_sibling() {
    for expression in ["calc(1 + 1%)", "calc(1px + 1s)"] {
        rejects_atomically(
            CssKnownProperty::Transform,
            &format!("scale3d({expression}, 1, 1)"),
        );
        rejects_atomically(
            CssKnownProperty::Filter,
            &format!("brightness(1) contrast({expression})"),
        );
        for property in [
            CssKnownProperty::BorderImageSlice,
            CssKnownProperty::BorderImageWidth,
        ] {
            rejects_atomically(property, &format!("1 {expression}"));
        }
        rejects_atomically(CssKnownProperty::LineHeight, expression);
    }
}
