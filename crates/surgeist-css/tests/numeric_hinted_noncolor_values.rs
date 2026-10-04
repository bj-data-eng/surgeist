#![forbid(unsafe_code)]
//! Functional checked-payload evidence for percentage-permitting numeric slots.
//! Source matching rules and selected grammars are cited in the unchanged
//! numeric_hinted_noncolor_contexts target. No percentage basis is supplied.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Error = CssSpecifiedValueSerializationError;
type Kind = CssSpecifiedValueSerializationErrorKind;
const SOURCE: &str = "calc((1px + 1%) / 1px)";
const CSS: &str = "calc((1% + 1px) / 1px)";

fn components(text: &str) -> CssComponentValues {
    parse_component_values(text).unwrap()
}
fn hinted(text: &str) -> CssHintedNumberCalculation {
    CssHintedNumberCalculation::try_from_components(components(text)).unwrap()
}
fn declaration(property: CssKnownProperty, text: &str) -> CssDeclaration {
    let source = format!("{}:{text}!important", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}
fn transform(source: &CssDeclaration) -> &CssTransformFunction {
    let CssKnownPropertyValueRef::Transform(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("transform")
    };
    let CssTransform::Functions(values) = value.value() else {
        panic!("functions")
    };
    &values.functions()[0]
}
fn filter(source: &CssDeclaration) -> &CssFilter {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Filter(value) => value.value(),
        CssKnownPropertyValueRef::BackdropFilter(value) => value.value(),
        _ => panic!("filter"),
    }
}
fn line_height(source: &CssDeclaration) -> &CssLineHeight {
    let CssKnownPropertyValueRef::LineHeight(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("line-height")
    };
    value.line_height()
}

#[test]
fn selected_frontdoors_expose_distinct_hinted_number_payloads_without_changing_authored_graphs() {
    let source = declaration(CssKnownProperty::Transform, &format!("scaleZ({SOURCE})"));
    let CssTransformFunction::ScaleZ(CssTransformScaleComponent::HintedNumberCalculation(value)) =
        transform(&source)
    else {
        panic!("hinted Z")
    };
    assert_eq!(value.components().serialize().unwrap().as_css(), SOURCE);
    assert_eq!(
        value.numeric_type().percent_hint(),
        Some(CssNumericDimension::Length)
    );
    assert!(value.position().is_some());
    assert_eq!(value.components().nesting_depth(), 2);
    let source = declaration(CssKnownProperty::Filter, &format!("contrast({SOURCE})"));
    let CssFilter::Functions(values) = filter(&source) else {
        panic!("functions")
    };
    let CssFilterFunction::Contrast(CssFilterAmount::HintedNumberCalculation(value)) =
        &values.functions()[0]
    else {
        panic!("hinted filter")
    };
    assert_eq!(value.components().serialize().unwrap().as_css(), SOURCE);
    let source = declaration(CssKnownProperty::BorderImageSlice, SOURCE);
    let CssKnownPropertyValueRef::BorderImageSlice(slice) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("slice")
    };
    for value in slice.slice().values() {
        let CssBorderImageSliceComponent::HintedNumberCalculation(value) = value else {
            panic!("hinted slice")
        };
        assert_eq!(value.components().serialize().unwrap().as_css(), SOURCE);
    }
    let source = declaration(CssKnownProperty::BorderImageWidth, SOURCE);
    let CssKnownPropertyValueRef::BorderImageWidth(width) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("width")
    };
    for value in width.widths().values() {
        let CssBorderImageWidthComponent::HintedNumberCalculation(value) = value else {
            panic!("hinted width")
        };
        assert_eq!(value.components().serialize().unwrap().as_css(), SOURCE);
    }
    let source = declaration(CssKnownProperty::LineHeight, SOURCE);
    let CssLineHeight::HintedNumberCalculation(value) = line_height(&source) else {
        panic!("hinted line-height")
    };
    assert_eq!(value.components().serialize().unwrap().as_css(), SOURCE);
    assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
}

#[test]
fn checked_transform_construction_preserves_context_and_ignores_only_origins_in_equality() {
    let programmatic = CssComponentValues::try_new(vec![
        CssComponentValue::try_function("calc", components("(1px + 1%) / 1px")).unwrap(),
    ])
    .unwrap();
    let value = CssHintedNumberCalculation::try_from_components_with_limits(
        programmatic.clone(),
        CssComponentValueLimits::try_new(2, 11, SOURCE.len()).unwrap(),
    )
    .unwrap();
    assert!(matches!(value.origin(), CssValueOrigin::Programmatic));
    assert!(value.position().is_none());
    assert_eq!(value.components(), &programmatic);
    assert!(
        CssHintedNumberCalculation::try_from_components_with_limits(
            programmatic,
            CssComponentValueLimits::try_new(1, 11, SOURCE.len()).unwrap(),
        )
        .is_err()
    );
    let original = value.components().clone();
    let z = CssTransformFunction::ScaleZ(CssTransformScaleComponent::HintedNumberCalculation(
        value.clone(),
    ));
    let parsed = declaration(CssKnownProperty::Transform, &format!("scaleZ({SOURCE})"));
    assert_eq!(&z, transform(&parsed));
    let xyz = CssTransformFunction::Scale3d(CssTransformScale3d::new(
        CssTransformScaleComponent::HintedNumberCalculation(value.clone()),
        CssTransformScaleComponent::HintedNumberCalculation(value.clone()),
        CssTransformScaleComponent::HintedNumberCalculation(value.clone()),
    ));
    let parsed = declaration(
        CssKnownProperty::Transform,
        &format!("scale3d({SOURCE}, {SOURCE}, {SOURCE})"),
    );
    assert_eq!(&xyz, transform(&parsed));
    assert_ne!(
        z,
        CssTransformFunction::ScaleZ(CssTransformScaleComponent::HintedNumberCalculation(hinted(
            "calc((2px + 1%) / 1px)"
        )))
    );
    assert_eq!(value.components(), &original);
    assert_eq!(value.serialize().unwrap().as_css(), SOURCE);
}

#[test]
fn all_checked_filter_amount_functions_reuse_symbolic_projection_and_structural_equality() {
    let amount = CssFilterAmount::HintedNumberCalculation(hinted(SOURCE));
    for (name, function) in [
        ("brightness", CssFilterFunction::Brightness(amount.clone())),
        ("contrast", CssFilterFunction::Contrast(amount.clone())),
        ("grayscale", CssFilterFunction::Grayscale(amount.clone())),
        ("invert", CssFilterFunction::Invert(amount.clone())),
        ("opacity", CssFilterFunction::Opacity(amount.clone())),
        ("saturate", CssFilterFunction::Saturate(amount.clone())),
        ("sepia", CssFilterFunction::Sepia(amount.clone())),
    ] {
        let value =
            CssFilter::Functions(CssFilterFunctionList::try_new(vec![function.clone()]).unwrap());
        let before = value.clone();
        for property in [CssKnownProperty::Filter, CssKnownProperty::BackdropFilter] {
            let source = declaration(property, &format!("{name}({SOURCE})"));
            assert_eq!(&value, filter(&source));
        }
        assert_eq!(
            function.serialize_specified().unwrap(),
            format!("{name}({CSS})")
        );
        assert_eq!(
            value.serialize_specified().unwrap(),
            format!("{name}({CSS})")
        );
        assert_eq!(value, before);
    }
}

#[test]
fn border_image_checked_edges_compress_only_after_visiting_their_shared_numeric_children() {
    let slice = CssBorderImageSlice::try_new(
        vec![CssBorderImageSliceComponent::HintedNumberCalculation(
            hinted(SOURCE),
        )],
        false,
    )
    .unwrap();
    let width =
        CssBorderImageWidth::try_new(vec![CssBorderImageWidthComponent::HintedNumberCalculation(
            hinted(SOURCE),
        )])
        .unwrap();
    let source = declaration(CssKnownProperty::BorderImageSlice, SOURCE);
    let CssKnownPropertyValueRef::BorderImageSlice(parsed) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("slice")
    };
    assert_eq!(&slice, parsed.slice());
    let source = declaration(CssKnownProperty::BorderImageWidth, SOURCE);
    let CssKnownPropertyValueRef::BorderImageWidth(parsed) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("width")
    };
    assert_eq!(&width, parsed.widths());
    // Aggregate 1 + four edges, each edge aggregate 1 + seven input/eight
    // projection nodes. Compression retains all visits: 33 inputs/37 projections.
    limits(&slice, CSS, 33, 37, |value, limit| {
        value.serialize_specified_with_limits(limit)
    });
    limits(&width, CSS, 33, 37, |value, limit| {
        value.serialize_specified_with_limits(limit)
    });
    let mixed = CssBorderImageSlice::try_new(
        vec![
            CssBorderImageSliceComponent::HintedNumberCalculation(hinted(SOURCE)),
            CssBorderImageSliceComponent::HintedNumberCalculation(hinted("calc((1px + 2%) / 1px)")),
        ],
        true,
    )
    .unwrap();
    assert_eq!(
        mixed.serialize_specified().unwrap(),
        "calc((1% + 1px) / 1px) calc((2% + 1px) / 1px) fill"
    );
}

fn limits<T: Clone + PartialEq + std::fmt::Debug>(
    value: &T,
    css: &str,
    inputs: usize,
    projections: usize,
    serialize: impl Fn(&T, Limits) -> Result<String, Error>,
) {
    let before = value.clone();
    for (limit, kind) in [
        (
            Limits::new(inputs - 1, projections, css.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(inputs, projections - 1, css.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(inputs, projections, css.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        assert_eq!(serialize(value, limit).unwrap_err().kind(), kind);
        assert_eq!(value, &before);
    }
    assert_eq!(
        serialize(value, Limits::new(inputs, projections, css.len())).unwrap(),
        css
    );
    assert_eq!(value, &before);
}

#[test]
fn filter_siblings_charge_one_cumulative_budget_without_resolving_the_percentage_basis() {
    let amount = CssFilterAmount::HintedNumberCalculation(hinted(SOURCE));
    let value = CssFilter::Functions(
        CssFilterFunctionList::try_new(vec![
            CssFilterFunction::Contrast(amount.clone()),
            CssFilterFunction::Sepia(amount),
        ])
        .unwrap(),
    );
    // List 1 + two function aggregates + two calculations of 7 inputs/8 projections.
    let css = format!("contrast({CSS}) sepia({CSS})");
    limits(&value, &css, 17, 19, |value, limit| {
        value.serialize_specified_with_limits(limit)
    });
}

#[test]
fn contextual_line_height_has_checked_equality_and_exact_atomic_projection_limits() {
    let value = CssLineHeight::HintedNumberCalculation(hinted(SOURCE));
    let source = declaration(CssKnownProperty::LineHeight, SOURCE);
    assert_eq!(&value, line_height(&source));
    // Wrapper, Product, Group, Sum and three leaves: seven inputs. Three leaves,
    // two replacement scalars, Sum, inverse and Product: eight projections.
    limits(&value, CSS, 7, 8, |value, limit| {
        value.serialize_specified_with_limits(limit)
    });
}

#[test]
fn negative_math_remains_authored_while_negative_ordinary_values_still_fail_admission() {
    let source = "calc((0px - 1px + 1%) / 1px)";
    let css = "calc((1% - 1px) / 1px)";
    let value = CssLineHeight::HintedNumberCalculation(hinted(source));
    assert_eq!(value.serialize_specified().unwrap(), css);
    assert_eq!(
        line_height(&declaration(CssKnownProperty::LineHeight, source)),
        &value
    );
    assert_eq!(
        CssFilterFunction::Contrast(CssFilterAmount::HintedNumberCalculation(hinted(source)))
            .serialize_specified()
            .unwrap(),
        format!("contrast({css})")
    );
    for (property, text) in [
        (CssKnownProperty::LineHeight, "-1"),
        (CssKnownProperty::BorderImageSlice, "-1"),
        (CssKnownProperty::BorderImageWidth, "-1"),
        (CssKnownProperty::Filter, "contrast(-1)"),
    ] {
        let report =
            parse_style_attribute(&format!("{}:{text};height:1px", property.canonical_name()));
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
    }
}

#[test]
fn existing_shorthand_and_backdrop_reentry_retains_checked_contextual_payloads_and_occurrence() {
    for (property, text) in [
        (CssKnownProperty::Font, format!("16px / {SOURCE} serif")),
        (
            CssKnownProperty::BorderImage,
            format!("none {SOURCE} / {SOURCE}"),
        ),
        (
            CssKnownProperty::BackdropFilter,
            format!("contrast({SOURCE})"),
        ),
    ] {
        let source = parse_property_value(
            CssPropertyNameRef::Known(property),
            components("var(--numeric)"),
            CssImportance::Important,
        )
        .unwrap();
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending consumer")
        };
        let replacement = components(&text);
        let checked = parse_property_value(
            CssPropertyNameRef::Known(property),
            replacement.clone(),
            CssImportance::Important,
        )
        .unwrap();
        let CssExpansion::Contributions(CssContributions::Longhands(expected)) =
            expand_declaration(&checked).unwrap()
        else {
            panic!("checked contributions")
        };
        let CssContributions::Longhands(actual) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("replacement contributions")
        };
        assert_eq!(actual.items().len(), expected.items().len());
        for (actual, expected) in actual.items().iter().zip(expected.items()) {
            assert_eq!(actual.property(), expected.property());
            assert_eq!(actual.value(), expected.value());
            assert!(actual.source().same_occurrence(&source));
            assert_eq!(actual.replacement_components(), Some(&replacement));
            assert_eq!(actual.source().importance(), CssImportance::Important);
        }
        assert!(matches!(
            pending
                .reenter(components("calc(1 + 1%)"))
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        assert!(pending.reenter(replacement).is_ok());
    }
}
