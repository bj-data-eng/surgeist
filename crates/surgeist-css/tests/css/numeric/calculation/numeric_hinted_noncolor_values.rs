//! Functional checked-payload evidence for percentage-permitting numeric slots.
//! Source matching rules and selected grammars are cited in the unchanged
//! numeric_hinted_noncolor_contexts target. No percentage basis is supplied.

use crate::css::support::component_values as components;
use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::important_declaration;
use crate::css::support::serialization_limits::LimitLadder;
use crate::css::support::value_assertions::filter_value as filter;
use surgeist_css::*;

const SOURCE: &str = "calc((1px + 1%) / 1px)";
const CSS: &str = "calc((1% + 1px) / 1px)";

fn hinted(text: &str) -> CssHintedNumberCalculation {
    CssHintedNumberCalculation::try_from_components(components(text), Default::default()).unwrap()
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
    let source = important_declaration(
        CssKnownProperty::Transform.canonical_name(),
        &format!("scaleZ({SOURCE})"),
    );
    let CssTransformFunction::ScaleZ(CssTransformScaleComponent::HintedNumberCalculation(value)) =
        transform(&source)
    else {
        panic!("hinted Z")
    };
    assert_eq!(
        value
            .components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        SOURCE
    );
    assert_eq!(
        value.numeric_type().percent_hint(),
        Some(CssNumericDimension::Length)
    );
    assert!(value.position().is_some());
    assert_eq!(value.components().nesting_depth(), 2);
    let source = important_declaration(
        CssKnownProperty::Filter.canonical_name(),
        &format!("contrast({SOURCE})"),
    );
    let CssFilter::Functions(values) = filter(&source) else {
        panic!("functions")
    };
    let CssFilterFunction::Contrast(CssFilterAmount::HintedNumberCalculation(value)) =
        &values.functions()[0]
    else {
        panic!("hinted filter")
    };
    assert_eq!(
        value
            .components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        SOURCE
    );
    let source = important_declaration(CssKnownProperty::BorderImageSlice.canonical_name(), SOURCE);
    let CssKnownPropertyValueRef::BorderImageSlice(slice) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("slice")
    };
    for value in slice.slice().values() {
        let CssBorderImageSliceComponent::HintedNumberCalculation(value) = value else {
            panic!("hinted slice")
        };
        assert_eq!(
            value
                .components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            SOURCE
        );
    }
    let source = important_declaration(CssKnownProperty::BorderImageWidth.canonical_name(), SOURCE);
    let CssKnownPropertyValueRef::BorderImageWidth(width) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("width")
    };
    for value in width.widths().values() {
        let CssBorderImageWidthComponent::HintedNumberCalculation(value) = value else {
            panic!("hinted width")
        };
        assert_eq!(
            value
                .components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            SOURCE
        );
    }
    let source = important_declaration(CssKnownProperty::LineHeight.canonical_name(), SOURCE);
    let CssLineHeight::HintedNumberCalculation(value) = line_height(&source) else {
        panic!("hinted line-height")
    };
    assert_eq!(
        value
            .components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        SOURCE
    );
    assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
}

#[test]
fn checked_transform_construction_preserves_context_and_ignores_only_origins_in_equality() {
    let programmatic = CssComponentValues::try_new(
        vec![CssComponentValue::try_function("calc", components("(1px + 1%) / 1px")).unwrap()],
        Default::default(),
    )
    .unwrap();
    let value = CssHintedNumberCalculation::try_from_components(
        programmatic.clone(),
        CssComponentValueLimits::try_new(2, 11, SOURCE.len()).unwrap(),
    )
    .unwrap();
    assert!(matches!(value.origin(), CssValueOrigin::Programmatic));
    assert!(value.position().is_none());
    assert_eq!(value.components(), &programmatic);
    crate::css::support::value_assertions::assert_numeric_resource(
        CssHintedNumberCalculation::try_from_components(
            programmatic,
            CssComponentValueLimits::try_new(1, 11, SOURCE.len()).unwrap(),
        ),
        ComponentFailure::NestingLimit,
    );
    let original = value.components().clone();
    let z = CssTransformFunction::ScaleZ(CssTransformScaleComponent::HintedNumberCalculation(
        value.clone(),
    ));
    let parsed = important_declaration(
        CssKnownProperty::Transform.canonical_name(),
        &format!("scaleZ({SOURCE})"),
    );
    assert_eq!(&z, transform(&parsed));
    let xyz = CssTransformFunction::Scale3d(CssTransformScale3d::new(
        CssTransformScaleComponent::HintedNumberCalculation(value.clone()),
        CssTransformScaleComponent::HintedNumberCalculation(value.clone()),
        CssTransformScaleComponent::HintedNumberCalculation(value.clone()),
    ));
    let parsed = important_declaration(
        CssKnownProperty::Transform.canonical_name(),
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
    assert_eq!(
        value.serialize_tokens(Default::default()).unwrap().as_css(),
        SOURCE
    );
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
            let source =
                important_declaration(property.canonical_name(), &format!("{name}({SOURCE})"));
            assert_eq!(&value, filter(&source));
        }
        assert_eq!(
            function.serialize_specified(Default::default()).unwrap(),
            format!("{name}({CSS})")
        );
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
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
    let source = important_declaration(CssKnownProperty::BorderImageSlice.canonical_name(), SOURCE);
    let CssKnownPropertyValueRef::BorderImageSlice(parsed) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("slice")
    };
    assert_eq!(&slice, parsed.slice());
    let source = important_declaration(CssKnownProperty::BorderImageWidth.canonical_name(), SOURCE);
    let CssKnownPropertyValueRef::BorderImageWidth(parsed) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("width")
    };
    assert_eq!(&width, parsed.widths());
    // Aggregate 1 + four edges, each edge aggregate 1 + seven input/eight
    // projection nodes. Compression retains all visits: 33 inputs/37 projections.
    LimitLadder::new(CSS, 33, 37)
        .assert_unchanged(&slice, |value, limit| value.serialize_specified(limit));
    LimitLadder::new(CSS, 33, 37)
        .assert_unchanged(&width, |value, limit| value.serialize_specified(limit));
    let mixed = CssBorderImageSlice::try_new(
        vec![
            CssBorderImageSliceComponent::HintedNumberCalculation(hinted(SOURCE)),
            CssBorderImageSliceComponent::HintedNumberCalculation(hinted("calc((1px + 2%) / 1px)")),
        ],
        true,
    )
    .unwrap();
    assert_eq!(
        mixed.serialize_specified(Default::default()).unwrap(),
        "calc((1% + 1px) / 1px) calc((2% + 1px) / 1px) fill"
    );
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
    LimitLadder::new(&css, 17, 19)
        .assert_unchanged(&value, |value, limit| value.serialize_specified(limit));
}

#[test]
fn contextual_line_height_has_checked_equality_and_exact_atomic_projection_limits() {
    let value = CssLineHeight::HintedNumberCalculation(hinted(SOURCE));
    let source = important_declaration(CssKnownProperty::LineHeight.canonical_name(), SOURCE);
    assert_eq!(&value, line_height(&source));
    // Wrapper, Product, Group, Sum and three leaves: seven inputs. Three leaves,
    // two replacement scalars, Sum, inverse and Product: eight projections.
    LimitLadder::new(CSS, 7, 8)
        .assert_unchanged(&value, |value, limit| value.serialize_specified(limit));
}

#[test]
fn negative_math_remains_authored_while_negative_ordinary_values_fail_admission() {
    let source = "calc((0px - 1px + 1%) / 1px)";
    let css = "calc((1% - 1px) / 1px)";
    let value = CssLineHeight::HintedNumberCalculation(hinted(source));
    assert_eq!(value.serialize_specified(Default::default()).unwrap(), css);
    assert_eq!(
        line_height(&important_declaration(
            CssKnownProperty::LineHeight.canonical_name(),
            source
        )),
        &value
    );
    assert_eq!(
        CssFilterFunction::Contrast(CssFilterAmount::HintedNumberCalculation(hinted(source)))
            .serialize_specified(Default::default())
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
fn shorthand_and_backdrop_reentry_retains_checked_contextual_payloads_and_occurrence() {
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
        assert!(
            pending
                .reenter(components("calc(1 + 1%)"))
                .unwrap_err()
                .replacement_error()
                .is_some()
        );
        assert!(pending.reenter(replacement).is_ok());
    }
}
