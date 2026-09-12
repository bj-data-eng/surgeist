#![forbid(unsafe_code)]

//! The selected Grid3 auto-repeat body is a general, nonrecursive track-size
//! sequence. Surrounding auto-track-list members retain their Grid2 fixed types.
//! https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#intrinsic-auto-repeat
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#typedef-auto-track-list
//! The explicit borrowed return type below is a public consumer contract.
//! Before that API change, compilation fails and none of these assertions runs.

use surgeist_css::{
    CssAuthoredGridAutoRepeat, CssAuthoredGridAutoRepeatKind, CssAuthoredGridAutoTrackComponent,
    CssAuthoredGridFixedRepeatComponent, CssAuthoredGridTrackBreadthKind as BreadthKind,
    CssAuthoredGridTrackList, CssAuthoredGridTrackRepeatComponent as Member,
    CssAuthoredGridTrackRepeatContent, CssAuthoredGridTrackSize, CssCalcLength,
    CssCalculationExpressionRef, CssCalculationProductOperator, CssCalculationType,
    CssCalculationValueRef, CssDeclaration, CssGridAutoFlowAxis, CssImportance,
    CssKnownProperty as Property, CssKnownPropertyValueRef, CssLength, CssPropertyNameRef,
    parse_component_values, parse_property_value, parse_style_attribute,
};

fn columns(declaration: &CssDeclaration) -> &surgeist_css::CssGridTemplateColumnsPropertyValue {
    let Some(CssKnownPropertyValueRef::GridTemplateColumns(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("an ordinary template-columns value");
    };
    value
}

fn general_body(repeat: &CssAuthoredGridAutoRepeat) -> &CssAuthoredGridTrackRepeatContent {
    // This annotation must compile for an external consumer after the migration.
    let content: &CssAuthoredGridTrackRepeatContent = repeat.content();
    content
}

fn single_repeat(list: &CssAuthoredGridTrackList) -> &CssAuthoredGridAutoRepeat {
    assert!(list.general_list().is_none());
    let [CssAuthoredGridAutoTrackComponent::AutoRepeat(value)] =
        list.auto_list().expect("an auto track list").components()
    else {
        panic!("one automatic repetition without surrounding tracks");
    };
    value
}

fn track(body: &CssAuthoredGridTrackRepeatContent, index: usize) -> &CssAuthoredGridTrackSize {
    let Member::TrackSize(size) = &body.components()[index] else {
        panic!("track-size at authored component {index}");
    };
    size
}

fn assert_name(body: &CssAuthoredGridTrackRepeatContent, index: usize, expected: &str) {
    let Member::LineNames(names) = &body.components()[index] else {
        panic!("line names at authored component {index}");
    };
    let [name] = names.names() else {
        panic!("one authored line name");
    };
    assert_eq!(name.as_str(), expected);
}

fn assert_keyword(size: &CssAuthoredGridTrackSize, expected: BreadthKind) {
    let breadth = size.breadth().expect("a keyword track breadth");
    assert_eq!(breadth.kind(), expected);
    assert!(breadth.length().is_none());
    assert!(breadth.fraction().is_none());
}

fn intrinsic_body_and_fixed_surroundings_remain_distinct() {
    let authored = concat!(
        "[outer] 10px repeat(auto-fill, [a] auto [b] min-content [c] max-content ",
        "[d] 1fr [e] minmax(auto, 1fr) [f] fit-content(20%) [g]) repeat(2, 5px) [end]",
    );
    let source = format!("grid-template-columns:{authored}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one complete Grid declaration");
    };
    assert_eq!(declaration.importance(), CssImportance::Important);
    let value = columns(declaration);
    assert_eq!(value.as_css(), authored);
    assert!(value.current().general_list().is_none());
    let [
        CssAuthoredGridAutoTrackComponent::LineNames(outer),
        CssAuthoredGridAutoTrackComponent::FixedSize(before),
        CssAuthoredGridAutoTrackComponent::AutoRepeat(repeat),
        CssAuthoredGridAutoTrackComponent::Repeat(after),
        CssAuthoredGridAutoTrackComponent::LineNames(end),
    ] = value.current().auto_list().unwrap().components()
    else {
        panic!("fixed surroundings and automatic body retain their distinct branches");
    };
    assert_eq!(outer.names().len(), 1);
    assert_eq!(outer.names()[0].as_str(), "outer");
    assert_eq!(end.names().len(), 1);
    assert_eq!(end.names()[0].as_str(), "end");
    assert_eq!(
        before.size().breadth().unwrap().length(),
        Some(&CssLength::try_px(10.0).unwrap())
    );
    assert_eq!(after.count().value(), 2);
    let [CssAuthoredGridFixedRepeatComponent::FixedSize(size)] = after.content().components()
    else {
        panic!("surrounding integer repetition retains fixed-size content");
    };
    assert_eq!(
        size.size().breadth().unwrap().length(),
        Some(&CssLength::try_px(5.0).unwrap())
    );
    assert_eq!(repeat.kind(), CssAuthoredGridAutoRepeatKind::AutoFill);
    let body = general_body(repeat);
    assert_eq!(body.components().len(), 13);
    for (index, name) in [
        (0, "a"),
        (2, "b"),
        (4, "c"),
        (6, "d"),
        (8, "e"),
        (10, "f"),
        (12, "g"),
    ] {
        assert_name(body, index, name);
    }
    assert_keyword(track(body, 1), BreadthKind::Auto);
    assert_keyword(track(body, 3), BreadthKind::MinContent);
    assert_keyword(track(body, 5), BreadthKind::MaxContent);
    let flex = track(body, 7).breadth().unwrap();
    assert_eq!(flex.kind(), BreadthKind::Fraction);
    assert_eq!(flex.fraction().unwrap().value(), 1.0);
    let (minimum, maximum) = track(body, 9).minmax().unwrap();
    assert_eq!(minimum.kind(), BreadthKind::Auto);
    assert_eq!(maximum.kind(), BreadthKind::Fraction);
    assert_eq!(maximum.fraction().unwrap().value(), 1.0);
    assert_eq!(
        track(body, 11).fit_content(),
        Some(&CssLength::try_percent(20.0).unwrap())
    );
    println!("general automatic body and fixed surroundings: ok");
}

fn automatic_calculations_and_names_remain_symbolic() {
    let report = parse_style_attribute(
        "grid-template-columns:repeat(auto-fit, [start] calc(1px * 2) [end])",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let value = columns(&report.syntax()[0]);
    assert!(value.i01_subset().is_none());
    let repeat = single_repeat(value.current());
    assert_eq!(repeat.kind(), CssAuthoredGridAutoRepeatKind::AutoFit);
    let body = general_body(repeat);
    assert_eq!(body.components().len(), 3);
    assert_name(body, 0, "start");
    assert_name(body, 2, "end");
    let Some(CssLength::Calc(CssCalcLength::Typed(calculation))) =
        track(body, 1).breadth().unwrap().length()
    else {
        panic!("the authored typed calculation is preserved");
    };
    assert_eq!(calculation.result_type(), CssCalculationType::Length);
    let CssCalculationExpressionRef::NestedCalc(root) = calculation.expression() else {
        panic!("expected calc root")
    };
    let CssCalculationExpressionRef::Product(product) = root.operand() else {
        panic!("the multiplication stays symbolic");
    };
    assert_eq!(product.len(), 2);
    let first = product.factor(0).unwrap();
    assert_eq!(first.operator(), None);
    let CssCalculationExpressionRef::Value(CssCalculationValueRef::Length(length)) =
        first.expression()
    else {
        panic!("the first operand is 1px");
    };
    assert_eq!(length.representation(), "1");
    assert_eq!(length.unit(), Some("px"));
    let second = product.factor(1).unwrap();
    assert_eq!(
        second.operator(),
        Some(CssCalculationProductOperator::Multiply)
    );
    assert!(matches!(
        second.expression(),
        CssCalculationExpressionRef::Value(CssCalculationValueRef::Integer(v)) if v.representation() == "2"
    ));
    println!("symbolic automatic calculation and line names: ok");
}

fn checked_components_preserve_general_body_values_and_origins() {
    let authored = "RePeAt(AUTO-FIT, [Start] MINMAX(MIN-CONTENT, 2fr) [End])";
    let components = parse_component_values(authored).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(Property::GridTemplateColumns),
        components.clone(),
        CssImportance::Important,
    )
    .expect("checked construction uses the selected auto-repeat grammar");
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    let original_tokens = components.serialize().unwrap();
    let retained_tokens = declaration.value_components().serialize().unwrap();
    assert_eq!(retained_tokens.as_css(), original_tokens.as_css());
    assert_eq!(retained_tokens.segments(), original_tokens.segments());
    assert_eq!(
        retained_tokens.origin_at(retained_tokens.as_css().len()),
        original_tokens.origin_at(original_tokens.as_css().len())
    );
    let surgeist_css::CssValueOrigin::Parsed(original) = components.items()[0].origin() else {
        panic!("the supplied first token has parsed provenance");
    };
    let surgeist_css::CssValueOrigin::Parsed(retained) =
        declaration.value_components().items()[0].origin()
    else {
        panic!("checked construction retains the first token's provenance");
    };
    assert!(retained.source().same_snapshot(original.source()));
    let value = columns(&declaration);
    assert_eq!(value.as_css(), authored);
    let repeat = single_repeat(value.current());
    assert_eq!(repeat.kind(), CssAuthoredGridAutoRepeatKind::AutoFit);
    let body = general_body(repeat);
    assert_eq!(body.components().len(), 3);
    assert_name(body, 0, "Start");
    assert_name(body, 2, "End");
    let (minimum, maximum) = track(body, 1).minmax().unwrap();
    assert_eq!(minimum.kind(), BreadthKind::MinContent);
    assert_eq!(maximum.kind(), BreadthKind::Fraction);
    assert_eq!(maximum.fraction().unwrap().value(), 2.0);
    println!("checked component grammar and preserved origins: ok");
}

fn shorthand_axes_keep_their_explicit_and_implicit_roles() {
    let report = parse_style_attribute(concat!(
        "grid-template:repeat(auto-fill, auto) / repeat(auto-fit, 1fr);",
        "grid:auto-flow dense 12px / repeat(auto-fit, fit-content(20%))",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    let Some(CssKnownPropertyValueRef::GridTemplate(value)) =
        report.syntax()[0].known().unwrap().property_value()
    else {
        panic!("the template shorthand");
    };
    let rows = single_repeat(value.current().rows().unwrap());
    assert_eq!(rows.kind(), CssAuthoredGridAutoRepeatKind::AutoFill);
    let rows_body = general_body(rows);
    assert_eq!(rows_body.components().len(), 1);
    assert_keyword(track(rows_body, 0), BreadthKind::Auto);
    let columns = single_repeat(value.current().columns().unwrap());
    assert_eq!(columns.kind(), CssAuthoredGridAutoRepeatKind::AutoFit);
    let columns_body = general_body(columns);
    assert_eq!(columns_body.components().len(), 1);
    assert_eq!(
        track(columns_body, 0)
            .breadth()
            .unwrap()
            .fraction()
            .unwrap()
            .value(),
        1.0
    );

    let Some(CssKnownPropertyValueRef::Grid(value)) =
        report.syntax()[1].known().unwrap().property_value()
    else {
        panic!("the auto-flow grid shorthand");
    };
    assert!(value.current().template_value().is_none());
    let flow = value.current().auto_flow().unwrap();
    assert_eq!(flow.axis(), CssGridAutoFlowAxis::Row);
    assert!(flow.dense());
    let [implicit] = value.current().auto_tracks().unwrap().sizes() else {
        panic!("one implicit track size without repetition");
    };
    assert_eq!(
        implicit.breadth().unwrap().length(),
        Some(&CssLength::try_px(12.0).unwrap())
    );
    let explicit = single_repeat(value.current().explicit_tracks().unwrap());
    assert_eq!(explicit.kind(), CssAuthoredGridAutoRepeatKind::AutoFit);
    let body = general_body(explicit);
    assert_eq!(body.components().len(), 1);
    assert_eq!(
        track(body, 0).fit_content(),
        Some(&CssLength::try_percent(20.0).unwrap())
    );
    println!("explicit axes and implicit track roles: ok");
}

fn main() {
    intrinsic_body_and_fixed_surroundings_remain_distinct();
    automatic_calculations_and_names_remain_symbolic();
    checked_components_preserve_general_body_values_and_origins();
    shorthand_axes_keep_their_explicit_and_implicit_roles();
}
