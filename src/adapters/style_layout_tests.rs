use super::style_layout::{
    font_size_scalar, layout_aspect_ratio, layout_grid_line, layout_grid_span,
    lower_grid_placement, lower_track_repeat_with_session, map_track_repetition_error,
};
use super::{
    AdapterBoundary, AdapterErrorKind, lower_style_to_layout, lower_style_to_layout_with_store,
};
use crate::{layout, style};

#[test]
fn style_layout_calc_font_size_is_unsupported_for_normal_grid_flow_tolerance_lowering() {
    let error = font_size_scalar(style::Length::Calc(style::CalcLength::px(12.0))).unwrap_err();

    assert!(error.to_string().contains("calc font size"));
}

#[test]
fn style_layout_lower_node_converts_positive_aspect_ratio_to_layout_type() {
    let resolved = resolved_with(
        style::Declarations::new()
            .try_set(style::Property::AspectRatio, style::Value::Number(1.5))
            .unwrap(),
    );

    let lowered = lower_style_to_layout(&resolved).unwrap();

    assert_eq!(
        lowered.aspect_ratio.map(layout::AspectRatio::get),
        Some(1.5)
    );
}

#[test]
fn style_layout_default_zero_aspect_ratio_lowers_to_none() {
    let resolved = resolved_with(style::Declarations::new());

    let lowered = lower_style_to_layout(&resolved).unwrap();

    assert_eq!(lowered.aspect_ratio, None);
}

#[test]
fn style_layout_adapter_rejects_non_finite_positive_aspect_ratio() {
    let error = layout_aspect_ratio(f32::INFINITY).unwrap_err();

    assert_eq!(error.boundary(), AdapterBoundary::StyleToLayout);
    assert!(matches!(
        error.kind(),
        AdapterErrorKind::StyleLayoutValue { property, reason }
            if property == "aspect-ratio" && reason.contains("aspect ratio")
    ));
}

#[test]
fn style_layout_adapter_rejects_negative_aspect_ratio_if_validation_is_bypassed() {
    let error = layout_aspect_ratio(-1.0).unwrap_err();

    assert_eq!(error.boundary(), AdapterBoundary::StyleToLayout);
    assert!(matches!(
        error.kind(),
        AdapterErrorKind::StyleLayoutValue { property, reason }
            if property == "aspect-ratio" && reason.contains("aspect ratio")
    ));
}

#[test]
fn style_layout_lower_grid_placement_uses_layout_validated_line() {
    let placement =
        lower_grid_placement(style::GridLine::line(2).unwrap(), style::GridLine::Auto).unwrap();

    assert_eq!(placement.start().map(layout::GridLine::get), Some(2));
    assert_eq!(placement.end(), None);
    assert_eq!(placement.span(), None);
}

#[test]
fn style_layout_lower_grid_placement_uses_layout_validated_line_span() {
    let placement = lower_grid_placement(
        style::GridLine::line(2).unwrap(),
        style::GridLine::span(3).unwrap(),
    )
    .unwrap();

    assert_eq!(placement.start().map(layout::GridLine::get), Some(2));
    assert_eq!(placement.end(), None);
    assert_eq!(placement.span().map(layout::GridSpan::get), Some(3));
}

#[test]
fn style_layout_lower_grid_placement_rejects_invalid_zero_line_if_validation_is_bypassed() {
    let error = layout_grid_line(0).unwrap_err();

    assert_eq!(error.boundary(), AdapterBoundary::StyleToLayout);
    assert!(error.to_string().contains("grid line"));
}

#[test]
fn style_layout_lower_grid_placement_rejects_invalid_zero_span_if_validation_is_bypassed() {
    let error = layout_grid_span(0).unwrap_err();

    assert_eq!(error.boundary(), AdapterBoundary::StyleToLayout);
    assert!(error.to_string().contains("grid span"));
}

#[test]
fn style_layout_lower_grid_placement_keeps_named_numeric_placement_auto() {
    let placement = lower_grid_placement(
        style::GridLine::named_line("content", 1).unwrap(),
        style::GridLine::Auto,
    )
    .unwrap();

    assert!(placement.is_auto());
}

#[test]
fn style_layout_lower_track_repeat_count_uses_layout_fallible_constructor() {
    let repeat = style::TrackRepeat::count(2, vec![simple_track_component()]).unwrap();
    let mut session = super::LayoutLoweringSession::new();

    let lowered = lower_track_repeat_with_session(&repeat, &mut session).unwrap();

    assert_eq!(
        lowered.repeat(),
        layout::TrackRepeat::Count(layout::TrackRepeatCount::new(2).unwrap())
    );
    assert_eq!(lowered.components().len(), 1);
}

#[test]
fn style_layout_lower_track_repeat_rejects_zero_count_if_validation_is_bypassed() {
    let error = map_track_repetition_error(layout::TrackRepetitionError::ZeroCount);

    assert_eq!(error.boundary(), AdapterBoundary::StyleToLayout);
    assert!(error.to_string().contains("track repeat"));
}

#[test]
fn style_layout_lower_track_repeat_rejects_empty_components_if_validation_is_bypassed() {
    let error = map_track_repetition_error(layout::TrackRepetitionError::EmptyComponents);

    assert_eq!(error.boundary(), AdapterBoundary::StyleToLayout);
    assert!(error.to_string().contains("track repeat"));
}

#[test]
fn style_layout_lowers_calc_dimension_into_layout_calc_store() {
    let calc = style::CalcLength::sum(
        style::CalcLengthTerm::add(style::CalcLength::px(20.0)),
        [style::CalcLengthTerm::add(style::CalcLength::percent(10.0))],
    );
    let resolved = resolved_with(
        style::Declarations::new()
            .try_set(
                style::Property::Width,
                style::Value::Length(style::Length::Calc(calc)),
            )
            .unwrap(),
    );

    let lowered = lower_style_to_layout_with_store(&resolved).unwrap();

    let layout::Dimension::Calc(id) = lowered.node.size.width else {
        panic!("expected calc width, got {:?}", lowered.node.size.width);
    };
    let expression = lowered.calc_store.get(id).unwrap();
    assert!((expression.percent_fraction() - 0.10).abs() < f32::EPSILON);
    assert_eq!(expression.resolve(Some(200.0)).value, Some(40.0));
    assert_eq!(lowered.calc_store.len(), 1);
}

#[test]
fn style_layout_rejects_calc_values_without_calc_store() {
    let resolved = resolved_with(
        style::Declarations::new()
            .try_set(
                style::Property::Width,
                style::Value::Length(style::Length::Calc(style::CalcLength::px(12.0))),
            )
            .unwrap(),
    );

    let error = lower_style_to_layout(&resolved).unwrap_err();

    assert_eq!(error.boundary(), AdapterBoundary::StyleToLayout);
    assert!(matches!(
        error.kind(),
        AdapterErrorKind::StyleLayoutValue { property, reason }
            if property == "width" && reason.contains("calc values require")
    ));
}

fn simple_track_component() -> style::GridTrackComponent {
    style::GridTrackComponent::Track(style::TrackSizing::AUTO)
}

fn resolved_with(declarations: style::Declarations) -> style::Resolved {
    use crate::retained::{Element, Model, Patch, Tag};

    let mut model = Model::empty();
    let root = model.root();
    let panel = model
        .apply(Patch::Insert {
            parent: root,
            index: 0,
            element: Element::tagged(Tag::new("panel").unwrap()),
        })
        .unwrap()
        .changes()
        .inserted()[0];
    let tree = model.snapshot();
    style::Resolver::new(style::Sheet::new())
        .resolve(style::Context::new(&tree, panel).local(&declarations))
        .unwrap()
}
