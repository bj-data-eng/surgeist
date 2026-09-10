#![forbid(unsafe_code)]

//! Grid2 permits optional line names before and after repeated track sizes.
//! A length-valued calculation is a fixed breadth, so these cases already fit
//! both integer and automatic repetition without the Grid3 grammar extension.
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#repeat-syntax
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#typedef-fixed-breadth
//! Selected Grid2 source SHA256:
//! 05aa64853c4428146973943b35caf121e44c1076bdf5b8c29f8896dba9b778e2
//! Authored spelling, semantic inspection, importance, and exact optional I01
//! projection are Surgeist public contracts. No calculation is evaluated here.

use surgeist_css::{
    CssAuthoredGridAutoRepeatKind, CssAuthoredGridAutoTrackComponent,
    CssAuthoredGridFixedRepeatComponent, CssAuthoredGridGeneralTrackComponent,
    CssAuthoredGridTrackList, CssAuthoredGridTrackRepeatComponent, CssAuthoredGridTrackSize,
    CssCalcLength, CssCalculationExpressionRef, CssCalculationProductOperator, CssCalculationType,
    CssCalculationValueRef, CssCustomIdent, CssGridLineNames, CssGridRepeat, CssGridRepeatCount,
    CssGridTrackBreadth, CssGridTrackComponent, CssGridTrackList, CssGridTrackSize, CssImportance,
    CssKnownProperty, CssKnownPropertyValueRef, CssLength, CssLengthUnit, parse_style_attribute,
    validate_style_attribute,
};

#[derive(Clone, Copy)]
enum RepeatMode {
    Integer,
    AutoFill,
    AutoFit,
}

#[derive(Clone, Copy)]
enum ContentCase {
    LiteralTrailing,
    TypedLeading,
    TypedTrailing,
}

#[test]
fn integer_repeat_preserves_trailing_line_names_after_a_typed_calculation() {
    assert_repeat(
        "RePeAt(2, 2px [end])",
        RepeatMode::Integer,
        ContentCase::LiteralTrailing,
    );
    assert_repeat(
        "RePeAt(2, [start] calc(1px * 2))",
        RepeatMode::Integer,
        ContentCase::TypedLeading,
    );
    assert_repeat(
        "RePeAt(2, calc(1px * 2) [end])",
        RepeatMode::Integer,
        ContentCase::TypedTrailing,
    );
}

#[test]
fn auto_fill_repeat_preserves_trailing_line_names_after_a_typed_calculation() {
    assert_repeat(
        "RePeAt(AUTO-FILL, 2px [end])",
        RepeatMode::AutoFill,
        ContentCase::LiteralTrailing,
    );
    assert_repeat(
        "RePeAt(AUTO-FILL, [start] calc(1px * 2))",
        RepeatMode::AutoFill,
        ContentCase::TypedLeading,
    );
    assert_repeat(
        "RePeAt(AUTO-FILL, calc(1px * 2) [end])",
        RepeatMode::AutoFill,
        ContentCase::TypedTrailing,
    );
}

#[test]
fn auto_fit_repeat_preserves_trailing_line_names_after_a_typed_calculation() {
    assert_repeat(
        "RePeAt(AUTO-FIT, 2px [end])",
        RepeatMode::AutoFit,
        ContentCase::LiteralTrailing,
    );
    assert_repeat(
        "RePeAt(AUTO-FIT, [start] calc(1px * 2))",
        RepeatMode::AutoFit,
        ContentCase::TypedLeading,
    );
    assert_repeat(
        "RePeAt(AUTO-FIT, calc(1px * 2) [end])",
        RepeatMode::AutoFit,
        ContentCase::TypedTrailing,
    );
}

fn assert_repeat(authored: &str, mode: RepeatMode, case: ContentCase) {
    let source = format!("GRID-TEMPLATE-COLUMNS:  {authored} !IMPORTANT; color: red");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration, sibling] = report.syntax().as_slice() else {
        panic!("{source}: the Grid declaration and following color must both remain");
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    let name = declaration.parsed_name().expect("authored property name");
    assert_eq!(name.source().as_str(), source);
    assert_eq!(name.span().start().byte_offset().value(), 0);
    assert_eq!(
        name.span().end().byte_offset().value(),
        "GRID-TEMPLATE-COLUMNS".len()
    );
    let known = declaration.known().expect("known Grid declaration");
    assert_eq!(known.property(), CssKnownProperty::GridTemplateColumns);
    let CssKnownPropertyValueRef::GridTemplateColumns(value) =
        known.property_value().expect("typed Grid value")
    else {
        panic!("{source}: expected the template-columns value");
    };
    assert_eq!(value.as_css(), authored);

    let (names, size) = ordered_repeat_content(value.current(), mode, case);
    let [name] = names.names() else {
        panic!("{source}: exactly one authored line name must remain");
    };
    assert_eq!(
        name.as_str(),
        match case {
            ContentCase::TypedLeading => "start",
            ContentCase::LiteralTrailing | ContentCase::TypedTrailing => "end",
        }
    );
    let breadth = size.breadth().expect("a direct length track breadth");
    let length = breadth.length().expect("a length-valued track");
    match case {
        ContentCase::LiteralTrailing => {
            assert_eq!(length, &CssLength::try_px(2.0).unwrap());
            assert_eq!(value.i01_subset(), Some(&literal_projection(mode)));
        }
        ContentCase::TypedLeading | ContentCase::TypedTrailing => {
            assert_one_pixel_times_two(length);
            assert!(value.i01_subset().is_none());
        }
    }

    assert_eq!(sibling.importance(), CssImportance::Normal);
    let sibling = sibling.known().expect("retained color declaration");
    assert_eq!(sibling.property(), CssKnownProperty::Color);
    let CssKnownPropertyValueRef::Color(color) =
        sibling.property_value().expect("typed sibling color")
    else {
        panic!("{source}: expected the unchanged color sibling");
    };
    assert_eq!(color.as_css(), "red");
    assert!(validate_style_attribute(&source).is_ok(), "{source}");
}

fn ordered_repeat_content(
    value: &CssAuthoredGridTrackList,
    mode: RepeatMode,
    case: ContentCase,
) -> (&CssGridLineNames, &CssAuthoredGridTrackSize) {
    match mode {
        RepeatMode::Integer => {
            assert!(value.auto_list().is_none());
            let list = value
                .general_list()
                .expect("integer repeat in a general list");
            let [CssAuthoredGridGeneralTrackComponent::Repeat(repeat)] = list.components() else {
                panic!("exactly one integer repeat must remain");
            };
            assert_eq!(repeat.count().value(), 2);
            match (case, repeat.content().components()) {
                (
                    ContentCase::TypedLeading,
                    [
                        CssAuthoredGridTrackRepeatComponent::LineNames(names),
                        CssAuthoredGridTrackRepeatComponent::TrackSize(size),
                    ],
                )
                | (
                    ContentCase::LiteralTrailing | ContentCase::TypedTrailing,
                    [
                        CssAuthoredGridTrackRepeatComponent::TrackSize(size),
                        CssAuthoredGridTrackRepeatComponent::LineNames(names),
                    ],
                ) => (names, size),
                _ => panic!("integer repetition must preserve the exact two-component order"),
            }
        }
        RepeatMode::AutoFill | RepeatMode::AutoFit => {
            assert!(value.general_list().is_none());
            let list = value
                .auto_list()
                .expect("automatic repeat in an auto track list");
            let [CssAuthoredGridAutoTrackComponent::AutoRepeat(repeat)] = list.components() else {
                panic!("exactly one automatic repeat must remain");
            };
            assert_eq!(
                repeat.kind(),
                match mode {
                    RepeatMode::AutoFill => CssAuthoredGridAutoRepeatKind::AutoFill,
                    RepeatMode::AutoFit => CssAuthoredGridAutoRepeatKind::AutoFit,
                    RepeatMode::Integer => unreachable!("integer branch handled above"),
                }
            );
            match (case, repeat.content().components()) {
                (
                    ContentCase::TypedLeading,
                    [
                        CssAuthoredGridFixedRepeatComponent::LineNames(names),
                        CssAuthoredGridFixedRepeatComponent::FixedSize(size),
                    ],
                )
                | (
                    ContentCase::LiteralTrailing | ContentCase::TypedTrailing,
                    [
                        CssAuthoredGridFixedRepeatComponent::FixedSize(size),
                        CssAuthoredGridFixedRepeatComponent::LineNames(names),
                    ],
                ) => (names, size.size()),
                _ => panic!("automatic repetition must preserve the exact two-component order"),
            }
        }
    }
}

fn assert_one_pixel_times_two(length: &CssLength) {
    let CssLength::Calc(CssCalcLength::Typed(calculation)) = length else {
        panic!("retain the typed calculation rather than evaluating it to a literal");
    };
    assert_eq!(calculation.result_type(), CssCalculationType::Length);
    let CssCalculationExpressionRef::Product(product) = calculation.expression() else {
        panic!("retain the authored multiplication");
    };
    assert_eq!(product.len(), 2);
    let first = product.factor(0).expect("the authored 1px factor");
    assert_eq!(first.operator(), None);
    let CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(value)) =
        first.expression()
    else {
        panic!("the first factor is an authored length");
    };
    assert_eq!(value.value(), 1.0);
    assert_eq!(value.unit(), CssLengthUnit::Px);
    let second = product.factor(1).expect("the authored 2 factor");
    assert_eq!(
        second.operator(),
        Some(CssCalculationProductOperator::Multiply)
    );
    assert!(matches!(
        second.expression(),
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Integer(2))
    ));
}

fn literal_projection(mode: RepeatMode) -> CssGridTrackList {
    let count = match mode {
        RepeatMode::Integer => CssGridRepeatCount::try_integer(2).unwrap(),
        RepeatMode::AutoFill => CssGridRepeatCount::AutoFill,
        RepeatMode::AutoFit => CssGridRepeatCount::AutoFit,
    };
    let tracks = CssGridTrackList::try_new(vec![
        CssGridTrackComponent::TrackSize(CssGridTrackSize::Breadth(CssGridTrackBreadth::Length(
            CssLength::try_px(2.0).unwrap(),
        ))),
        CssGridTrackComponent::LineNames(
            CssGridLineNames::try_new(vec![CssCustomIdent::try_new("end").unwrap()]).unwrap(),
        ),
    ])
    .unwrap();
    CssGridTrackList::try_new(vec![CssGridTrackComponent::Repeat(
        CssGridRepeat::try_new(count, tracks).unwrap(),
    )])
    .unwrap()
}
