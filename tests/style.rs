use proptest::prelude::*;
use surgeist::{layout as l, retained as r, style as s};

#[test]
fn public_color_helper_and_error_surface_are_stable() {
    let color = s::color(0x11223344);
    let error = s::Error::new(s::ErrorCode::InvalidValue, "bad style value");

    assert_eq!(
        color,
        s::Color::rgba(
            0x11 as f32 / 255.0,
            0x22 as f32 / 255.0,
            0x33 as f32 / 255.0,
            0x44 as f32 / 255.0
        )
    );
    assert_eq!(error.code(), s::ErrorCode::InvalidValue);
    assert_eq!(error.message(), "bad style value");
    assert_eq!(error.to_string(), "InvalidValue: bad style value");
}

#[test]
fn primitive_style_values_default_and_validate_predictably() {
    assert_eq!(s::Color::default(), s::Color::TRANSPARENT);
    assert_eq!(s::Length::default(), s::Length::ZERO);
    assert_eq!(s::Edges::default(), s::Edges::all(s::Length::ZERO));
    assert_eq!(s::Corners::default(), s::Corners::all(s::Length::ZERO));
    assert_eq!(s::LineStyle::default(), s::LineStyle::Solid);
    assert_eq!(s::SideSet::default(), s::SideSet::all());
    assert_eq!(s::StrokeAlign::default(), s::StrokeAlign::Center);
    assert_eq!(s::Visibility::default(), s::Visibility::Visible);
    assert_eq!(s::Cursor::default(), s::Cursor::Default);
    assert_eq!(s::PointerEvents::default(), s::PointerEvents::Auto);

    assert!(s::Color::try_rgba(0.0, 0.25, 0.5, 1.0).is_ok());
    assert_eq!(s::Length::try_px(12.0).unwrap(), s::Length::px(12.0));
    assert_eq!(
        s::Length::try_percent(50.0).unwrap(),
        s::Length::percent(50.0)
    );
    assert_eq!(
        s::Color::try_rgba(f32::NAN, 0.0, 0.0, 1.0)
            .unwrap_err()
            .code(),
        s::ErrorCode::InvalidValue
    );
    assert_eq!(
        s::Length::try_px(f32::INFINITY).unwrap_err().code(),
        s::ErrorCode::InvalidValue
    );
    assert_eq!(
        s::Length::try_percent(f32::NEG_INFINITY)
            .unwrap_err()
            .code(),
        s::ErrorCode::InvalidValue
    );
}

proptest! {
    #[test]
    fn generated_colors_accept_any_finite_channels(
        r in finite_style_number(),
        g in finite_style_number(),
        b in finite_style_number(),
        a in finite_style_number(),
    ) {
        let color = s::Color::try_rgba(r, g, b, a).unwrap();

        prop_assert_eq!(color, s::Color::rgba(r, g, b, a));
        prop_assert!(s::Value::Color(color).validate().is_ok());
    }

    #[test]
    fn generated_declarations_replace_same_property_without_growing(
        first in generated_color(),
        second in generated_color(),
        opacity in 0.0f32..=1.0,
    ) {
        let declarations = s::Declarations::new()
            .bg(first)
            .opacity(opacity)
            .bg(second);

        prop_assert_eq!(declarations.len(), 2);
        prop_assert_eq!(declarations.background(), Some(second));
        prop_assert_eq!(declarations.opacity_number(), Some(opacity));
    }

    #[test]
    fn generated_property_domains_accept_and_reject_expected_ranges(
        non_negative in 0.0f32..1_000_000.0,
        negative in -1_000_000.0f32..0.0,
        opacity in 0.0f32..=1.0,
        high_opacity in 1.0001f32..1_000.0,
    ) {
        prop_assert!(s::Declaration::try_new(
            s::Property::Padding,
            s::Value::Edges(s::Edges::all(s::Length::px(non_negative))),
        ).is_ok());
        prop_assert_eq!(
            s::Declaration::try_new(
                s::Property::Padding,
                s::Value::Edges(s::Edges::all(s::Length::px(negative))),
            ).unwrap_err().code(),
            s::ErrorCode::InvalidValue,
        );
        prop_assert!(s::Declaration::try_new(
            s::Property::Opacity,
            s::Value::Number(opacity),
        ).is_ok());
        prop_assert_eq!(
            s::Declaration::try_new(
                s::Property::Opacity,
                s::Value::Number(high_opacity),
            ).unwrap_err().code(),
            s::ErrorCode::InvalidValue,
        );
    }
}

#[test]
fn viewport_container_queries_expose_dimensions_and_condition_kind() {
    let viewport = s::Viewport::new(800.0, 600.0);
    let container = s::Container::new(320.0, 200.0);
    let viewport_condition = s::Condition::viewport(s::Viewport::min_width(640.0));
    let container_condition = s::Condition::container(s::Container::max_width(480.0));

    assert_eq!(viewport.width(), Some(800.0));
    assert_eq!(viewport.height(), Some(600.0));
    assert_eq!(container.width(), Some(320.0));
    assert_eq!(container.height(), Some(200.0));
    assert_eq!(s::Viewport::query().width(), None);
    assert_eq!(s::Container::query().height(), None);
    assert!(viewport_condition.is_viewport());
    assert!(!viewport_condition.is_container());
    assert!(container_condition.is_container());
    assert!(!container_condition.is_viewport());
    assert!(viewport_condition.matches(viewport, None));
    assert!(container_condition.matches(viewport, Some(container)));
    assert!(!container_condition.matches(viewport, None));
    assert!(s::Condition::matches_all(
        &[viewport_condition, container_condition],
        viewport,
        Some(container)
    ));
}

#[test]
fn declarations_override_earlier_values_and_expose_typed_accessors() {
    let declarations = s::Declarations::new()
        .bg(s::Color::rgba(0.1, 0.2, 0.3, 1.0))
        .padding(s::Edges::all(s::Length::px(8.0)))
        .bg(s::Color::rgba(0.7, 0.6, 0.5, 1.0));

    assert_eq!(
        declarations.get(s::Property::Background),
        Some(&s::Value::Color(s::Color::rgba(0.7, 0.6, 0.5, 1.0)))
    );
    assert_eq!(
        declarations.background(),
        Some(s::Color::rgba(0.7, 0.6, 0.5, 1.0))
    );
    assert_eq!(
        declarations.padding_edges(),
        Some(s::Edges::all(s::Length::px(8.0)))
    );
    assert_eq!(declarations.iter().count(), 2);
}

#[test]
fn declarations_mutation_methods_preserve_unique_properties_and_report_lengths() {
    let mut declarations = s::Declarations::new();

    assert!(declarations.is_empty());
    assert_eq!(declarations.len(), 0);

    assert_eq!(
        declarations
            .insert(s::Property::Background, s::Value::Color(red()))
            .get(s::Property::Background),
        Some(&s::Value::Color(red()))
    );
    declarations.insert(s::Property::Background, s::Value::Color(blue()));
    declarations
        .try_insert(s::Property::Opacity, s::Value::Keyword(s::Keyword::Initial))
        .unwrap();

    assert!(!declarations.is_empty());
    assert_eq!(declarations.len(), 2);
    assert_eq!(
        declarations.background(),
        Some(blue()),
        "later insert should replace the property without growing the set"
    );
    assert_eq!(
        declarations.try_insert(s::Property::Opacity, s::Value::Length(s::Length::px(1.0))),
        Err(s::Error::new(
            s::ErrorCode::InvalidProperty,
            "Opacity does not accept length"
        ))
    );
}

#[test]
fn concise_style_helpers_and_rule_conditions_are_public_front_doors() {
    let margin = s::Edges::all(s::Length::px(4.0));
    let font_size = s::Length::px(15.0);
    let condition = s::Condition::viewport(s::Viewport::min_width(640.0));
    let declarations = s::Declarations::new()
        .margin(margin.clone())
        .opacity(0.65)
        .font_size(font_size.clone())
        .cursor(s::Cursor::Pointer)
        .pointer_events(s::PointerEvents::None);
    let rule = s::Rule::new(s::Selector::any(), declarations.clone()).when([condition]);

    assert_eq!(s::color(0x336699cc), s::Color::rgba(0.2, 0.4, 0.6, 0.8));
    assert_eq!(
        declarations.get(s::Property::Margin),
        Some(&s::Value::Edges(margin.clone()))
    );
    assert_eq!(declarations.margin_edges(), Some(margin));
    assert_eq!(declarations.opacity_number(), Some(0.65));
    assert_eq!(declarations.font_size_length(), Some(font_size));
    assert_eq!(declarations.cursor_kind(), Some(s::Cursor::Pointer));
    assert_eq!(
        declarations.pointer_events_kind(),
        Some(s::PointerEvents::None)
    );
    assert_eq!(rule.conditions(), &[condition]);
}

#[test]
fn declaration_fingerprint_changes_with_content() {
    let base = s::Declarations::new().text_color(s::Color::rgba(1.0, 1.0, 1.0, 1.0));
    let changed = s::Declarations::new().text_color(s::Color::rgba(0.0, 0.0, 0.0, 1.0));

    assert_eq!(base.fingerprint(), base.clone().fingerprint());
    assert_ne!(base.fingerprint(), changed.fingerprint());
}

#[test]
fn declaration_fingerprint_distinguishes_grid_flow_tolerance_from_box_sizing() {
    let base = s::Declarations::new()
        .set(
            s::Property::BoxSizing,
            s::Value::BoxSizing(s::BoxSizing::ContentBox),
        )
        .set(
            s::Property::GridFlowTolerance,
            s::Value::GridFlowTolerance(s::GridFlowTolerance::Normal),
        );
    let swapped = s::Declarations::new()
        .set(
            s::Property::BoxSizing,
            s::Value::GridFlowTolerance(s::GridFlowTolerance::Normal),
        )
        .set(
            s::Property::GridFlowTolerance,
            s::Value::BoxSizing(s::BoxSizing::ContentBox),
        );

    assert_ne!(base.fingerprint(), swapped.fingerprint());
}

#[test]
fn metadata_reports_defaults_inheritance_impact_and_animation() {
    let color = s::Property::Color.metadata();
    assert!(color.inherited);
    assert!(color.impact.text);
    assert!(color.animatable);
    assert_eq!(color.interpolation, s::Interpolation::Color);
    assert_eq!(
        color.default,
        s::Value::Color(s::Color::rgba(0.0, 0.0, 0.0, 1.0))
    );

    let padding = s::Property::Padding.metadata();
    assert!(!padding.inherited);
    assert!(padding.impact.layout);
    assert!(!padding.impact.text);
    assert_eq!(padding.interpolation, s::Interpolation::Edges);
    assert_eq!(
        padding.default,
        s::Value::Edges(s::Edges::all(s::Length::px(0.0)))
    );

    let visibility = s::Property::Visibility.metadata();
    assert_eq!(visibility.interpolation, s::Interpolation::Discrete);
}

#[test]
fn metadata_and_invalidation_builders_accumulate_flags() {
    let metadata = s::Metadata::new(s::Value::Number(1.0))
        .inherited(true)
        .impact(
            s::Impact::empty()
                .layout()
                .paint()
                .text()
                .effect()
                .animation(),
        )
        .animatable(true)
        .interpolation(s::Interpolation::Number);
    let mut scope = s::Scope::empty();
    let invalidation = s::Invalidation::from_properties([
        s::Property::Padding,
        s::Property::Opacity,
        s::Property::TransitionDuration,
    ]);

    scope.include_node();
    scope.include_siblings();
    scope.include_descendants();

    assert!(metadata.inherited);
    assert!(metadata.animatable);
    assert_eq!(metadata.interpolation, s::Interpolation::Number);
    assert!(metadata.impact.layout);
    assert!(metadata.impact.paint);
    assert!(metadata.impact.text);
    assert!(metadata.impact.effect);
    assert!(metadata.impact.animation);
    assert!(scope.node);
    assert!(scope.siblings);
    assert!(scope.descendants);
    assert!(invalidation.layout);
    assert!(invalidation.paint);
    assert!(invalidation.animation);
}

#[test]
fn malformed_numeric_values_are_rejected_by_fallible_apis() {
    assert_eq!(
        s::Color::try_rgba(0.0, 0.0, f32::NAN, 1.0)
            .unwrap_err()
            .code(),
        s::ErrorCode::InvalidValue
    );
    assert!(s::Length::try_px(f32::INFINITY).is_err());
    assert!(s::Length::try_percent(f32::NEG_INFINITY).is_err());

    let mut declarations = s::Declarations::new();
    assert!(
        declarations
            .try_insert(s::Property::Opacity, s::Value::Number(f32::NAN))
            .is_err()
    );
    assert!(declarations.is_empty());

    let declarations = s::Declarations::new()
        .try_set(s::Property::Opacity, s::Value::Number(0.5))
        .unwrap();
    assert_eq!(
        declarations.get(s::Property::Opacity),
        Some(&s::Value::Number(0.5))
    );
}

#[test]
fn declarations_reject_property_value_mismatches() {
    let mut declarations = s::Declarations::new();

    let error = declarations
        .try_insert(s::Property::Padding, s::Value::Color(red()))
        .unwrap_err();

    assert_eq!(error.code(), s::ErrorCode::InvalidProperty);
    assert!(declarations.is_empty());

    let error = s::Declaration::try_new(s::Property::Opacity, s::Value::Length(s::Length::px(1.0)))
        .unwrap_err();

    assert_eq!(error.code(), s::ErrorCode::InvalidProperty);
}

#[test]
fn declarations_reject_mismatches_across_property_groups() {
    for (property, value) in [
        (s::Property::Display, s::Value::Length(s::Length::px(1.0))),
        (s::Property::ZIndex, s::Value::StringList(vec!["1".into()])),
        (s::Property::AnimationName, s::Value::Number(1.0)),
        (
            s::Property::TransitionProperty,
            s::Value::Length(s::Length::px(1.0)),
        ),
        (
            s::Property::Cursor,
            s::Value::Visibility(s::Visibility::Hidden),
        ),
    ] {
        let error = s::Declaration::try_new(property, value).unwrap_err();
        assert_eq!(error.code(), s::ErrorCode::InvalidProperty, "{property:?}");
    }
}

#[test]
fn declarations_reject_property_specific_invalid_values() {
    for (property, value) in [
        (
            s::Property::Padding,
            s::Value::Edges(s::Edges::all(s::Length::px(0.0))),
        ),
        (s::Property::FlexGrow, s::Value::Number(0.0)),
        (s::Property::TransitionDuration, s::Value::Number(0.0)),
    ] {
        s::Declaration::try_new(property, value).unwrap();
    }

    for (property, value) in [
        (s::Property::FontSize, s::Value::Length(s::Length::px(-1.0))),
        (
            s::Property::LineHeight,
            s::Value::Length(s::Length::percent(-10.0)),
        ),
        (
            s::Property::Padding,
            s::Value::Edges(s::Edges::all(s::Length::px(-1.0))),
        ),
        (
            s::Property::BorderWidth,
            s::Value::Edges(s::Edges::all(s::Length::px(-1.0))),
        ),
        (
            s::Property::Radius,
            s::Value::Corners(s::Corners::all(s::Length::px(-1.0))),
        ),
        (s::Property::Opacity, s::Value::Number(1.5)),
        (s::Property::FlexGrow, s::Value::Number(-1.0)),
        (s::Property::TransitionDuration, s::Value::Number(-1.0)),
    ] {
        let error = s::Declaration::try_new(property, value).unwrap_err();
        assert_eq!(error.code(), s::ErrorCode::InvalidValue, "{property:?}");
    }
}

#[test]
fn normal_length_is_limited_to_gap_properties() {
    for property in [
        s::Property::Gap,
        s::Property::RowGap,
        s::Property::ColumnGap,
    ] {
        s::Declaration::try_new(property, s::Value::Length(s::Length::NORMAL)).unwrap();
    }

    for property in [
        s::Property::Width,
        s::Property::Height,
        s::Property::MinWidth,
        s::Property::MaxWidth,
        s::Property::FlexBasis,
        s::Property::FontSize,
        s::Property::LineHeight,
    ] {
        let error =
            s::Declaration::try_new(property, s::Value::Length(s::Length::NORMAL)).unwrap_err();
        assert_eq!(error.code(), s::ErrorCode::InvalidValue, "{property:?}");
    }
}

#[test]
fn grid_flow_tolerance_length_accepts_only_px_lengths() {
    for tolerance in [
        s::GridFlowTolerance::Length(s::Length::px(0.0)),
        s::GridFlowTolerance::Length(s::Length::px(12.0)),
    ] {
        s::Declaration::try_new(
            s::Property::GridFlowTolerance,
            s::Value::GridFlowTolerance(tolerance),
        )
        .unwrap();
    }

    for tolerance in [
        s::GridFlowTolerance::Length(s::Length::percent(25.0)),
        s::GridFlowTolerance::Length(s::Length::Auto),
        s::GridFlowTolerance::Length(s::Length::Fill),
        s::GridFlowTolerance::Length(s::Length::Fit),
        s::GridFlowTolerance::Length(s::Length::MinContent),
        s::GridFlowTolerance::Length(s::Length::MaxContent),
        s::GridFlowTolerance::Length(s::Length::px(-1.0)),
    ] {
        let error = s::Declaration::try_new(
            s::Property::GridFlowTolerance,
            s::Value::GridFlowTolerance(tolerance.clone()),
        )
        .unwrap_err();
        assert_eq!(error.code(), s::ErrorCode::InvalidValue, "{tolerance:?}");
    }

    s::Declaration::try_new(
        s::Property::GridFlowTolerance,
        s::Value::GridFlowTolerance(s::GridFlowTolerance::Percent(25.0)),
    )
    .unwrap();
}

#[test]
fn values_expose_interpolation_categories() {
    assert_eq!(
        s::Value::Color(red()).interpolation(),
        s::Interpolation::Color
    );
    assert_eq!(
        s::Value::Stroke(s::Stroke::new(s::Length::px(1.0), blue())).interpolation(),
        s::Interpolation::Stroke
    );
    assert_eq!(
        s::Value::StringList(vec!["Inter".to_owned()]).interpolation(),
        s::Interpolation::Discrete
    );
}

#[test]
fn composite_values_validate_nested_fields() {
    let valid_transform = s::Transform {
        operations: vec![
            s::TransformOp::Translate {
                x: s::Length::px(1.0),
                y: s::Length::percent(2.0),
            },
            s::TransformOp::Scale { x: 2.0, y: 0.5 },
            s::TransformOp::Rotate { radians: 1.5 },
        ],
    };

    assert!(
        s::Value::Size(s::Size::new(s::Length::Fill, s::Length::Fit))
            .validate()
            .is_ok()
    );
    assert!(
        s::Value::ShadowList(vec![s::Shadow::soft(0.25).inset(true)])
            .validate()
            .is_ok()
    );
    assert!(
        s::Value::PropertyList(vec![s::Property::Opacity])
            .validate()
            .is_ok()
    );
    assert!(s::Value::Transform(valid_transform).validate().is_ok());

    for value in [
        s::Value::Size(s::Size::new(s::Length::px(f32::NAN), s::Length::Fit)),
        s::Value::ShadowList(vec![s::Shadow::new(
            s::Length::px(0.0),
            s::Length::px(0.0),
            s::Length::px(f32::INFINITY),
            s::Length::px(0.0),
            red(),
        )]),
        s::Value::Transform(s::Transform {
            operations: vec![s::TransformOp::Scale {
                x: f32::NAN,
                y: 1.0,
            }],
        }),
        s::Value::Transform(s::Transform {
            operations: vec![s::TransformOp::Rotate {
                radians: f32::INFINITY,
            }],
        }),
    ] {
        assert_eq!(
            value.validate().unwrap_err().code(),
            s::ErrorCode::InvalidValue
        );
    }

    for value in [
        s::Value::StringList(vec![String::new()]),
        s::Value::StringList(vec!["\u{0000}".to_string()]),
    ] {
        assert_eq!(
            value.validate().unwrap_err().code(),
            s::ErrorCode::InvalidString
        );
    }
}

#[test]
fn grid_style_values_represent_templates_subgrid_placements_and_lanes() {
    let track_list = s::GridTrackList::new(vec![
        s::GridTrackComponent::line_names(["outer-start"]),
        s::GridTrackComponent::Track(s::TrackSizing::px(40.0)),
        s::GridTrackComponent::line_names(["outer-end"]),
        s::GridTrackComponent::Repeat(s::TrackRepeat::count(
            2,
            vec![
                s::GridTrackComponent::line_names(["inner"]),
                s::GridTrackComponent::Track(s::TrackSizing::fr(1.0)),
            ],
        )),
        s::GridTrackComponent::Subgrid(s::SubgridTrack::new(vec![vec![], vec!["b".to_string()]])),
    ]);
    let placement = s::GridPlacement::new(
        s::GridLine::NamedSpan {
            name: "bar".to_string(),
            index: 1,
        },
        s::GridLine::NamedLine {
            name: "foo".to_string(),
            index: 3,
        },
    );
    let areas = s::GridTemplateAreas::new([
        s::GridTemplateAreaRow::named(["header", "header"]),
        s::GridTemplateAreaRow::new([Some("rail"), Some("content")]),
    ]);

    assert!(
        s::Value::GridTrackList(track_list.clone())
            .validate()
            .is_ok()
    );
    assert!(
        s::Value::GridPlacement(placement.clone())
            .validate()
            .is_ok()
    );
    assert!(
        s::Value::GridTemplateAreas(areas.clone())
            .validate()
            .is_ok()
    );

    let declarations = s::Declarations::new()
        .display(s::Display::GridLanes)
        .grid_template_columns(track_list.clone())
        .grid_template_areas(areas.clone())
        .grid_row_start(s::GridLine::Line(1))
        .grid_column_end(s::GridLine::Span(2))
        .grid_column(placement.clone())
        .grid_auto_flow(s::GridAutoFlow::ColumnDense);

    assert_eq!(
        declarations.get(s::Property::Display),
        Some(&s::Value::Display(s::Display::GridLanes))
    );
    assert_eq!(
        declarations.get(s::Property::GridTemplateColumns),
        Some(&s::Value::GridTrackList(track_list))
    );
    assert_eq!(
        declarations.get(s::Property::GridTemplateAreas),
        Some(&s::Value::GridTemplateAreas(areas))
    );
    assert_eq!(
        declarations.get(s::Property::GridRowStart),
        Some(&s::Value::GridLine(s::GridLine::Line(1)))
    );
    assert_eq!(
        declarations.get(s::Property::GridColumnEnd),
        Some(&s::Value::GridLine(s::GridLine::NamedLine {
            name: "foo".to_string(),
            index: 3,
        }))
    );
    assert_eq!(declarations.get(s::Property::GridColumn), None);
    assert_eq!(
        declarations.get(s::Property::GridAutoFlow),
        Some(&s::Value::GridAutoFlow(s::GridAutoFlow::ColumnDense))
    );
}

#[test]
fn grid_style_properties_validate_domains_and_resolve_defaults() {
    let invalid_subgrid = s::GridTrackList::new(vec![s::GridTrackComponent::Subgrid(
        s::SubgridTrack::new(vec![vec!["".to_string()]]),
    )]);
    let invalid_areas = s::GridTemplateAreas::new([
        s::GridTemplateAreaRow::named(["header", "header"]),
        s::GridTemplateAreaRow::named(["content"]),
    ]);
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let rows = s::GridTrackList::new(vec![s::GridTrackComponent::Track(s::TrackSizing::minmax(
        s::MinTrackSizing::Auto,
        s::MaxTrackSizing::fr(1.0),
    ))]);
    let areas = s::GridTemplateAreas::new([s::GridTemplateAreaRow::named(["a", "b"])]);
    let row_start = s::GridLine::NamedLine {
        name: "a-start".to_string(),
        index: 1,
    };
    let row = s::GridPlacement::span_line(2, 4);
    let local = s::Declarations::new()
        .display(s::Display::InlineGridLanes)
        .grid_template_rows(rows.clone())
        .grid_template_areas(areas.clone())
        .grid_row_start(row_start.clone())
        .grid_row(row.clone());

    assert_eq!(
        s::Declaration::try_new(
            s::Property::GridTemplateColumns,
            s::Value::GridTrackList(invalid_subgrid)
        )
        .unwrap_err()
        .code(),
        s::ErrorCode::InvalidString
    );
    assert_eq!(
        s::Declaration::try_new(
            s::Property::GridTemplateAreas,
            s::Value::GridTemplateAreas(invalid_areas)
        )
        .unwrap_err()
        .code(),
        s::ErrorCode::InvalidValue
    );
    assert_eq!(
        s::Declaration::try_new(
            s::Property::GridColumn,
            s::Value::GridAutoFlow(s::GridAutoFlow::Row)
        )
        .unwrap_err()
        .code(),
        s::ErrorCode::InvalidProperty
    );

    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();

    assert_eq!(
        resolved.get(s::Property::GridTemplateColumns),
        &s::Value::GridTrackList(s::GridTrackList::default())
    );
    assert_eq!(resolved.display(), s::Display::InlineGridLanes);
    assert_eq!(
        resolved.get(s::Property::GridTemplateRows),
        &s::Value::GridTrackList(rows)
    );
    assert_eq!(
        resolved.get(s::Property::GridTemplateAreas),
        &s::Value::GridTemplateAreas(areas)
    );
    assert_eq!(
        resolved.get(s::Property::GridRowStart),
        &s::Value::GridLine(row.start)
    );
    assert_eq!(
        resolved.get(s::Property::GridRowEnd),
        &s::Value::GridLine(row.end)
    );
    assert!(s::Property::GridTemplateColumns.metadata().impact.layout);
    assert!(s::Property::GridTemplateAreas.metadata().impact.layout);
    assert!(s::Property::GridColumnEnd.metadata().impact.layout);
}

#[test]
fn grid_style_values_reject_invalid_track_and_line_domains() {
    let invalid_repeat =
        s::GridTrackList::new(vec![s::GridTrackComponent::Repeat(s::TrackRepeat::count(
            0,
            vec![s::GridTrackComponent::Track(s::TrackSizing::px(8.0))],
        ))]);
    let invalid_track =
        s::GridTrackList::new(vec![s::GridTrackComponent::Track(s::TrackSizing::fr(-1.0))]);
    let invalid_line = s::GridPlacement::line(0);
    let invalid_area = s::GridTemplateAreas::new([s::GridTemplateAreaRow::new([Some("")])]);

    for value in [
        s::Value::GridTrackList(invalid_repeat),
        s::Value::GridTrackList(invalid_track),
        s::Value::GridPlacement(invalid_line),
    ] {
        assert_eq!(
            value.validate().unwrap_err().code(),
            s::ErrorCode::InvalidValue
        );
    }
    assert_eq!(
        s::Value::GridTemplateAreas(invalid_area)
            .validate()
            .unwrap_err()
            .code(),
        s::ErrorCode::InvalidString
    );
}

#[test]
fn named_grid_style_values_reject_reserved_grid_line_names() {
    for name in ["auto", "span"] {
        let values = [
            s::Value::GridLine(s::GridLine::BareIdent(name.to_string())),
            s::Value::GridLine(s::GridLine::NamedLine {
                name: name.to_string(),
                index: 1,
            }),
            s::Value::GridLine(s::GridLine::NamedSpan {
                name: name.to_string(),
                index: 1,
            }),
        ];

        for value in values {
            assert_eq!(
                value.validate().unwrap_err().code(),
                s::ErrorCode::InvalidString
            );
        }
    }
}

#[test]
fn named_grid_style_values_reject_zero_line_occurrences_and_spans() {
    let values = [
        s::Value::GridLine(s::GridLine::Span(0)),
        s::Value::GridLine(s::GridLine::NamedLine {
            name: "content".to_string(),
            index: 0,
        }),
        s::Value::GridLine(s::GridLine::NamedSpan {
            name: "content".to_string(),
            index: 0,
        }),
    ];

    for value in values {
        assert_eq!(
            value.validate().unwrap_err().code(),
            s::ErrorCode::InvalidValue
        );
    }
}

#[test]
fn named_grid_track_line_name_components_reject_reserved_names() {
    for name in ["auto", "span"] {
        let track_list = s::GridTrackList::new(vec![s::GridTrackComponent::line_names([name])]);
        let repeated_track_list = s::GridTrackList::new(vec![s::GridTrackComponent::Repeat(
            s::TrackRepeat::count(1, vec![s::GridTrackComponent::line_names([name])]),
        )]);
        let subgrid = s::GridTrackList::new(vec![s::GridTrackComponent::Subgrid(
            s::SubgridTrack::new(vec![vec![name.to_string()]]),
        )]);
        let repeated_subgrid = s::GridTrackList::new(vec![s::GridTrackComponent::Subgrid(
            s::SubgridTrack::from_components(vec![s::SubgridLineNameComponent::Repeat {
                count: s::SubgridLineNameRepeatCount::Count(1),
                line_name_sets: vec![vec![name.to_string()]],
            }]),
        )]);

        for value in [
            s::Value::GridTrackList(track_list),
            s::Value::GridTrackList(repeated_track_list),
            s::Value::GridTrackList(subgrid),
            s::Value::GridTrackList(repeated_subgrid),
        ] {
            assert_eq!(
                value.validate().unwrap_err().code(),
                s::ErrorCode::InvalidString
            );
        }
    }
}

#[test]
fn named_grid_subgrid_line_name_repeats_expand_fixed_and_auto_fill_components() {
    let subgrid = s::SubgridTrack::from_components(vec![
        s::SubgridLineNameComponent::LineNames(vec!["start".to_string()]),
        s::SubgridLineNameComponent::Repeat {
            count: s::SubgridLineNameRepeatCount::Count(2),
            line_name_sets: vec![
                vec!["a".to_string()],
                vec!["b".to_string(), "c".to_string()],
            ],
        },
        s::SubgridLineNameComponent::Repeat {
            count: s::SubgridLineNameRepeatCount::AutoFill,
            line_name_sets: vec![vec!["fill".to_string()]],
        },
        s::SubgridLineNameComponent::LineNames(vec!["tail".to_string()]),
    ]);

    assert!(subgrid.validate().is_ok());
    assert_eq!(
        subgrid.line_names(),
        vec![
            vec!["start".to_string()],
            vec!["a".to_string()],
            vec!["b".to_string(), "c".to_string()],
            vec!["a".to_string()],
            vec!["b".to_string(), "c".to_string()],
            vec!["fill".to_string()],
            vec!["tail".to_string()],
        ]
    );
}

#[test]
fn named_grid_subgrid_line_name_repeats_reject_invalid_repeat_forms() {
    let zero_repeat = s::SubgridTrack::from_components(vec![s::SubgridLineNameComponent::Repeat {
        count: s::SubgridLineNameRepeatCount::Count(0),
        line_name_sets: vec![vec!["a".to_string()]],
    }]);
    let multiple_auto_fill = s::SubgridTrack::from_components(vec![
        s::SubgridLineNameComponent::Repeat {
            count: s::SubgridLineNameRepeatCount::AutoFill,
            line_name_sets: vec![vec!["a".to_string()]],
        },
        s::SubgridLineNameComponent::Repeat {
            count: s::SubgridLineNameRepeatCount::AutoFill,
            line_name_sets: vec![vec!["b".to_string()]],
        },
    ]);

    assert_eq!(
        zero_repeat.validate().unwrap_err().code(),
        s::ErrorCode::InvalidValue
    );
    assert_eq!(
        multiple_auto_fill.validate().unwrap_err().code(),
        s::ErrorCode::InvalidValue
    );
}

#[test]
fn shorthand_declarations_are_canonicalized_at_intake() {
    let columns =
        s::GridTrackList::new(vec![s::GridTrackComponent::Track(s::TrackSizing::fr(1.0))]);
    let rows = s::GridTrackList::new(vec![s::GridTrackComponent::Track(s::TrackSizing::px(32.0))]);
    let areas = s::GridTemplateAreas::new([s::GridTemplateAreaRow::named(["main"])]);
    let template = s::GridTemplate::new(rows.clone(), columns.clone(), areas.clone());
    let definition = s::GridDefinition::new(template.clone())
        .auto_rows(s::GridTrackList::new(vec![s::GridTrackComponent::Track(
            s::TrackSizing::AUTO,
        )]))
        .auto_flow(s::GridAutoFlow::RowDense);
    let area = s::GridAreaPlacement::new(
        s::GridLine::NamedLine {
            name: "main-start".to_string(),
            index: 1,
        },
        s::GridLine::Line(1),
        s::GridLine::NamedLine {
            name: "main-end".to_string(),
            index: 1,
        },
        s::GridLine::Span(2),
    );

    assert!(s::Value::GridTemplate(template.clone()).validate().is_ok());
    assert!(
        s::Value::GridDefinition(definition.clone())
            .validate()
            .is_ok()
    );
    assert!(s::Value::GridAreaPlacement(area.clone()).validate().is_ok());

    let declarations = s::Declarations::new()
        .grid_template(template.clone())
        .grid(definition.clone())
        .grid_area(area.clone())
        .set(
            s::Property::Overflow,
            s::Value::OverflowAxes(s::OverflowAxes::new(
                s::Overflow::Hidden,
                s::Overflow::Scroll,
            )),
        )
        .set(s::Property::Gap, s::Value::Length(s::Length::px(8.0)));

    assert_eq!(declarations.get(s::Property::GridTemplate), None);
    assert_eq!(declarations.get(s::Property::Grid), None);
    assert_eq!(declarations.get(s::Property::GridArea), None);
    assert_eq!(declarations.get(s::Property::Overflow), None);
    assert_eq!(declarations.get(s::Property::Gap), None);
    assert_eq!(
        declarations.get(s::Property::GridTemplateRows),
        Some(&s::Value::GridTrackList(rows))
    );
    assert_eq!(
        declarations.get(s::Property::GridTemplateColumns),
        Some(&s::Value::GridTrackList(columns))
    );
    assert_eq!(
        declarations.get(s::Property::GridTemplateAreas),
        Some(&s::Value::GridTemplateAreas(areas))
    );
    assert_eq!(
        declarations.get(s::Property::GridAutoRows),
        Some(&s::Value::GridTrackList(definition.auto_rows.clone()))
    );
    assert_eq!(
        declarations.get(s::Property::GridAutoFlow),
        Some(&s::Value::GridAutoFlow(s::GridAutoFlow::RowDense))
    );
    assert_eq!(
        declarations.get(s::Property::GridRowStart),
        Some(&s::Value::GridLine(area.row_start))
    );
    assert_eq!(
        declarations.get(s::Property::GridColumnEnd),
        Some(&s::Value::GridLine(area.column_end))
    );
    assert_eq!(
        declarations.get(s::Property::OverflowX),
        Some(&s::Value::Overflow(s::Overflow::Hidden))
    );
    assert_eq!(
        declarations.get(s::Property::OverflowY),
        Some(&s::Value::Overflow(s::Overflow::Scroll))
    );
    assert_eq!(
        declarations.get(s::Property::RowGap),
        Some(&s::Value::Length(s::Length::px(8.0)))
    );
    assert_eq!(
        declarations.get(s::Property::ColumnGap),
        Some(&s::Value::Length(s::Length::px(8.0)))
    );
}

#[test]
fn grid_column_shorthand_repeats_bare_ident_on_omitted_end_side() {
    let declarations = s::Declarations::new().grid_column(s::GridPlacement::new(
        s::GridLine::BareIdent("main".to_string()),
        s::GridLine::Auto,
    ));

    assert_eq!(
        declarations.get(s::Property::GridColumnStart),
        Some(&s::Value::GridLine(s::GridLine::BareIdent(
            "main".to_string()
        )))
    );
    assert_eq!(
        declarations.get(s::Property::GridColumnEnd),
        Some(&s::Value::GridLine(s::GridLine::BareIdent(
            "main".to_string()
        )))
    );
}

#[test]
fn grid_column_shorthand_omits_non_ident_end_to_auto() {
    let declarations = s::Declarations::new().grid_column(s::GridPlacement::new(
        s::GridLine::Line(2),
        s::GridLine::Auto,
    ));

    assert_eq!(
        declarations.get(s::Property::GridColumnStart),
        Some(&s::Value::GridLine(s::GridLine::Line(2)))
    );
    assert_eq!(
        declarations.get(s::Property::GridColumnEnd),
        Some(&s::Value::GridLine(s::GridLine::Auto))
    );
}

#[test]
fn grid_area_one_bare_ident_expands_to_all_four_sides() {
    let declarations = s::Declarations::new().grid_area(s::GridAreaPlacement::new(
        s::GridLine::BareIdent("main".to_string()),
        s::GridLine::Auto,
        s::GridLine::Auto,
        s::GridLine::Auto,
    ));

    assert_eq!(
        declarations.get(s::Property::GridRowStart),
        Some(&s::Value::GridLine(s::GridLine::BareIdent(
            "main".to_string()
        )))
    );
    assert_eq!(
        declarations.get(s::Property::GridColumnStart),
        Some(&s::Value::GridLine(s::GridLine::BareIdent(
            "main".to_string()
        )))
    );
    assert_eq!(
        declarations.get(s::Property::GridRowEnd),
        Some(&s::Value::GridLine(s::GridLine::BareIdent(
            "main".to_string()
        )))
    );
    assert_eq!(
        declarations.get(s::Property::GridColumnEnd),
        Some(&s::Value::GridLine(s::GridLine::BareIdent(
            "main".to_string()
        )))
    );
}

#[test]
fn shorthand_canonicalization_preserves_later_longhand_overrides() {
    let shorthand_row = s::GridPlacement::new(s::GridLine::Line(1), s::GridLine::Line(3));
    let override_row_end = s::GridLine::Line(5);
    let declarations = s::Declarations::new()
        .grid_row(shorthand_row.clone())
        .grid_row_end(override_row_end.clone())
        .set(s::Property::Gap, s::Value::Length(s::Length::px(4.0)))
        .set(
            s::Property::ColumnGap,
            s::Value::Length(s::Length::px(12.0)),
        );

    assert_eq!(declarations.get(s::Property::GridRow), None);
    assert_eq!(
        declarations.get(s::Property::GridRowStart),
        Some(&s::Value::GridLine(shorthand_row.start))
    );
    assert_eq!(
        declarations.get(s::Property::GridRowEnd),
        Some(&s::Value::GridLine(override_row_end))
    );
    assert_eq!(
        declarations.get(s::Property::RowGap),
        Some(&s::Value::Length(s::Length::px(4.0)))
    );
    assert_eq!(
        declarations.get(s::Property::ColumnGap),
        Some(&s::Value::Length(s::Length::px(12.0)))
    );
}

#[test]
fn resolved_styles_do_not_store_transient_shorthand_properties() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let local = s::Declarations::new()
        .grid_column(s::GridPlacement::new(
            s::GridLine::Line(2),
            s::GridLine::Span(3),
        ))
        .set(s::Property::Gap, s::Value::Length(s::Length::px(6.0)));

    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();

    assert!(
        !resolved
            .iter()
            .any(|(property, _)| *property == s::Property::GridColumn)
    );
    assert!(
        !resolved
            .iter()
            .any(|(property, _)| *property == s::Property::Gap)
    );
    assert_eq!(
        resolved.get(s::Property::GridColumnStart),
        &s::Value::GridLine(s::GridLine::Line(2))
    );
    assert_eq!(
        resolved.get(s::Property::GridColumnEnd),
        &s::Value::GridLine(s::GridLine::Span(3))
    );
    assert_eq!(
        resolved.get(s::Property::RowGap),
        &s::Value::Length(s::Length::px(6.0))
    );
    assert_eq!(
        resolved.get(s::Property::ColumnGap),
        &s::Value::Length(s::Length::px(6.0))
    );
}

#[test]
fn layout_adapter_preserves_gap_normal_and_explicit_zero() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());

    let resolved_default = resolver.resolve(s::Context::new(&tree, 0)).unwrap();
    let default_layout = s::adapters::layout::lower(&resolved_default).unwrap();
    assert_eq!(
        resolved_default.get(s::Property::RowGap),
        &s::Value::Length(s::Length::NORMAL)
    );
    assert_eq!(default_layout.gap.width, l::Length::NORMAL);
    assert_eq!(default_layout.gap.height, l::Length::NORMAL);

    let explicit_zero =
        s::Declarations::new().set(s::Property::Gap, s::Value::Length(s::Length::ZERO));
    let resolved_zero = resolver
        .resolve(s::Context::new(&tree, 0).local(&explicit_zero))
        .unwrap();
    let zero_layout = s::adapters::layout::lower(&resolved_zero).unwrap();
    assert_eq!(zero_layout.gap.width, l::Length::ZERO);
    assert_eq!(zero_layout.gap.height, l::Length::ZERO);
}

#[test]
fn layout_adapter_lowers_canonical_layout_properties() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let columns = s::GridTrackList::new(vec![
        s::GridTrackComponent::Track(s::TrackSizing::px(40.0)),
        s::GridTrackComponent::Repeat(s::TrackRepeat::count(
            2,
            vec![s::GridTrackComponent::Track(s::TrackSizing::fr(1.0))],
        )),
    ]);
    let rows = s::GridTrackList::new(vec![s::GridTrackComponent::Track(s::TrackSizing::minmax(
        s::MinTrackSizing::Auto,
        s::MaxTrackSizing::FitContent(s::Length::percent(25.0)),
    ))]);
    let local = s::Declarations::new()
        .display(s::Display::Grid)
        .set(
            s::Property::BoxSizing,
            s::Value::BoxSizing(s::BoxSizing::ContentBox),
        )
        .set(
            s::Property::Position,
            s::Value::Position(s::LayoutPosition::Absolute),
        )
        .set(
            s::Property::Direction,
            s::Value::Direction(s::Direction::Rtl),
        )
        .set(
            s::Property::WritingMode,
            s::Value::WritingMode(s::WritingMode::VerticalRl),
        )
        .set(
            s::Property::TextAlign,
            s::Value::TextAlign(s::StyleTextAlign::LegacyCenter),
        )
        .set(
            s::Property::Overflow,
            s::Value::Overflow(s::Overflow::Scroll),
        )
        .set(s::Property::Float, s::Value::Float(s::Float::Left))
        .set(s::Property::Clear, s::Value::Clear(s::Clear::Both))
        .set(s::Property::ScrollbarWidth, s::Value::Number(7.0))
        .width(s::Length::px(120.0))
        .height(s::Length::percent(50.0))
        .set(s::Property::MinSize, s::Value::Length(s::Length::px(20.0)))
        .set(
            s::Property::MaxWidth,
            s::Value::Length(s::Length::px(400.0)),
        )
        .set(
            s::Property::MaxHeight,
            s::Value::Length(s::Length::percent(75.0)),
        )
        .set(s::Property::AspectRatio, s::Value::Number(1.5))
        .set(
            s::Property::Inset,
            s::Value::Edges(s::Edges::new(
                s::Length::px(3.0),
                s::Length::Auto,
                s::Length::percent(10.0),
                s::Length::px(5.0),
            )),
        )
        .margin(s::Edges::new(
            s::Length::px(1.0),
            s::Length::percent(2.0),
            s::Length::Auto,
            s::Length::px(4.0),
        ))
        .padding(s::Edges::all(s::Length::px(8.0)))
        .border_width(s::Edges::all(s::Length::px(1.0)))
        .set(
            s::Property::AlignItems,
            s::Value::AlignItems(s::AlignItems::Center),
        )
        .set(
            s::Property::AlignSelf,
            s::Value::AlignItems(s::AlignItems::SafeEnd),
        )
        .set(
            s::Property::JustifyItems,
            s::Value::AlignItems(s::AlignItems::Stretch),
        )
        .set(
            s::Property::JustifySelf,
            s::Value::AlignItems(s::AlignItems::FlexEnd),
        )
        .set(
            s::Property::AlignContent,
            s::Value::AlignContent(s::AlignContent::SpaceBetween),
        )
        .set(
            s::Property::JustifyContent,
            s::Value::AlignContent(s::AlignContent::SafeCenter),
        )
        .set(
            s::Property::FlexDirection,
            s::Value::FlexDirection(s::FlexDirection::ColumnReverse),
        )
        .set(s::Property::FlexWrap, s::Value::FlexWrap(s::FlexWrap::Wrap))
        .set(s::Property::FlexGrow, s::Value::Number(2.0))
        .set(s::Property::Gap, s::Value::Length(s::Length::px(6.0)))
        .grid_template_columns(columns)
        .grid_template_rows(rows)
        .grid_auto_columns(s::GridTrackList::new(vec![s::GridTrackComponent::Track(
            s::TrackSizing::percent(10.0),
        )]))
        .grid_auto_flow(s::GridAutoFlow::ColumnDense)
        .grid_column(s::GridPlacement::new(
            s::GridLine::Line(2),
            s::GridLine::Span(3),
        ))
        .grid_row(s::GridPlacement::span_line(2, 4));

    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();
    let layout = s::adapters::layout::lower(&resolved).unwrap();

    assert_eq!(layout.display, l::Display::Grid);
    assert_eq!(layout.box_sizing, l::BoxSizing::ContentBox);
    assert_eq!(layout.position, l::Position::Absolute);
    assert_eq!(layout.direction, l::Direction::Rtl);
    assert_eq!(layout.writing_mode, l::WritingMode::VerticalRl);
    assert_eq!(layout.text_align, l::TextAlign::LegacyCenter);
    assert_eq!(layout.overflow.x, l::Overflow::Scroll);
    assert_eq!(layout.overflow.y, l::Overflow::Scroll);
    assert_eq!(layout.float, l::Float::Left);
    assert_eq!(layout.clear, l::Clear::Both);
    assert_eq!(layout.scrollbar_width, 7.0);
    assert_eq!(layout.size.width, l::Dimension::px(120.0));
    assert_eq!(layout.size.height, l::Dimension::percent(0.5));
    assert_eq!(layout.min_size.width, l::Dimension::px(20.0));
    assert_eq!(layout.min_size.height, l::Dimension::px(20.0));
    assert_eq!(layout.max_size.width, l::Dimension::px(400.0));
    assert_eq!(layout.max_size.height, l::Dimension::percent(0.75));
    assert_eq!(layout.aspect_ratio, Some(1.5));
    assert_eq!(layout.inset.top, l::LengthAuto::px(3.0));
    assert_eq!(layout.inset.right, l::LengthAuto::AUTO);
    assert_eq!(layout.inset.bottom, l::LengthAuto::percent(0.1));
    assert_eq!(layout.margin.top, l::LengthAuto::px(1.0));
    assert_eq!(layout.margin.right, l::LengthAuto::percent(0.02));
    assert_eq!(layout.margin.bottom, l::LengthAuto::AUTO);
    assert_eq!(layout.padding.left, l::Length::px(8.0));
    assert_eq!(layout.border.right, l::Length::px(1.0));
    assert_eq!(layout.gap.width, l::Length::px(6.0));
    assert_eq!(layout.gap.height, l::Length::px(6.0));
    assert_eq!(layout.align_items, Some(l::AlignItems::Center));
    assert_eq!(layout.align_self, Some(l::AlignItems::SafeEnd));
    assert_eq!(layout.justify_items, Some(l::AlignItems::Stretch));
    assert_eq!(layout.justify_self, Some(l::AlignItems::FlexEnd));
    assert_eq!(layout.align_content, Some(l::AlignContent::SpaceBetween));
    assert_eq!(layout.justify_content, Some(l::AlignContent::SafeCenter));
    assert_eq!(layout.flex_direction, l::FlexDirection::ColumnReverse);
    assert_eq!(layout.flex_wrap, l::FlexWrap::Wrap);
    assert_eq!(layout.flex_grow, 2.0);
    assert_eq!(
        layout.grid_template_columns,
        vec![
            l::TrackComponent::px(40.0),
            l::TrackComponent::Repeat(l::TrackRepetition::count(2, vec![l::TrackSizing::fr(1.0)])),
        ]
    );
    assert_eq!(
        layout.grid_template_rows,
        vec![l::TrackComponent::minmax(
            l::MinTrackSizing::AUTO,
            l::MaxTrackSizing::fit_content(l::Length::percent(0.25)),
        )]
    );
    assert_eq!(
        layout.grid_auto_columns,
        vec![l::TrackComponent::percent(0.1)]
    );
    assert_eq!(layout.grid_auto_flow, l::GridAutoFlow::ColumnDense);
    assert_eq!(layout.grid_column, l::GridPlacement::line_span(2, 3));
    assert_eq!(layout.grid_row, l::GridPlacement::span_line(2, 4));
}

#[test]
fn layout_adapter_preserves_named_grid_line_placement() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let resolved = resolver
        .resolve(
            s::Context::new(&tree, 0).local(
                &s::Declarations::new()
                    .display(s::Display::Grid)
                    .grid_column(s::GridPlacement::new(
                        s::GridLine::NamedLine {
                            name: "content".to_string(),
                            index: 2,
                        },
                        s::GridLine::NamedSpan {
                            name: "content".to_string(),
                            index: 3,
                        },
                    )),
            ),
        )
        .unwrap();

    let input = s::adapters::layout::lower(&resolved).unwrap();
    assert_eq!(
        input.raw_grid_column,
        l::RawGridPlacement::new(
            l::RawGridLine::NamedLine {
                name: "content".to_string(),
                index: 2,
            },
            l::RawGridLine::NamedSpan {
                name: "content".to_string(),
                index: 3,
            },
        )
    );
}

#[test]
fn layout_adapter_preserves_bare_grid_line_ident() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let resolved = resolver
        .resolve(
            s::Context::new(&tree, 0).local(&s::Declarations::new().grid_column(
                s::GridPlacement::new(
                    s::GridLine::BareIdent("main".to_string()),
                    s::GridLine::NamedLine {
                        name: "main".to_string(),
                        index: 1,
                    },
                ),
            )),
        )
        .unwrap();

    let input = s::adapters::layout::lower(&resolved).unwrap();
    assert_eq!(
        input.raw_grid_column,
        l::RawGridPlacement::new(
            l::RawGridLine::BareIdent("main".to_string()),
            l::RawGridLine::NamedLine {
                name: "main".to_string(),
                index: 1,
            },
        )
    );
}

#[test]
fn layout_adapter_preserves_template_areas_and_track_line_names() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let areas = s::GridTemplateAreas::new([
        s::GridTemplateAreaRow::named(["head", "head"]),
        s::GridTemplateAreaRow::new([Some("nav"), Some("main")]),
    ]);
    let resolved = resolver
        .resolve(
            s::Context::new(&tree, 0).local(
                &s::Declarations::new()
                    .grid_template_areas(areas.clone())
                    .grid_template_columns(s::GridTrackList::new(vec![
                        s::GridTrackComponent::line_names(["page-start"]),
                        s::GridTrackComponent::Track(s::TrackSizing::px(12.0)),
                        s::GridTrackComponent::Repeat(s::TrackRepeat::count(
                            2,
                            vec![
                                s::GridTrackComponent::line_names(["inner"]),
                                s::GridTrackComponent::Track(s::TrackSizing::fr(1.0)),
                            ],
                        )),
                        s::GridTrackComponent::line_names(["page-end"]),
                    ])),
            ),
        )
        .unwrap();

    let input = s::adapters::layout::lower(&resolved).unwrap();
    assert_eq!(
        input.grid_template_areas,
        l::GridTemplateAreas {
            rows: areas
                .rows
                .iter()
                .map(|row| l::GridTemplateAreaRow {
                    cells: row.cells.clone(),
                })
                .collect(),
        }
    );
    assert_eq!(
        input.grid_template_columns,
        vec![
            l::TrackComponent::line_names(["page-start"]),
            l::TrackComponent::px(12.0),
            l::TrackComponent::Repeat(l::TrackRepetition::count_components(
                2,
                vec![
                    l::TrackComponent::line_names(["inner"]),
                    l::TrackComponent::fr(1.0),
                ],
            )),
            l::TrackComponent::line_names(["page-end"]),
        ]
    );
}

#[test]
fn layout_adapter_lowers_grid_lanes_displays_to_layout_grid_lanes() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());

    let local = s::Declarations::new().display(s::Display::GridLanes);
    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();

    let layout = s::adapters::layout::lower(&resolved).unwrap();

    assert_eq!(layout.display, l::Display::GridLanes);
}

#[test]
fn lowers_inline_grid_displays_to_layout_inline_variants() {
    for (style_display, layout_display) in [
        (s::Display::InlineGrid, l::Display::InlineGrid),
        (s::Display::InlineGridLanes, l::Display::InlineGridLanes),
    ] {
        let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
        let mut resolver = s::Resolver::new(s::Sheet::new());
        let resolved = resolver
            .resolve(
                s::Context::new(&tree, 0).local(&s::Declarations::new().display(style_display)),
            )
            .unwrap();

        let layout = s::adapters::layout::lower(&resolved).unwrap();

        assert_eq!(layout.display, layout_display);
    }
}

#[test]
fn lowers_inline_block_to_layout_inline_block() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let resolved = resolver
        .resolve(
            s::Context::new(&tree, 0)
                .local(&s::Declarations::new().display(s::Display::InlineBlock)),
        )
        .unwrap();
    let layout = s::adapters::layout::lower(&resolved).unwrap();

    assert_eq!(layout.display, l::Display::InlineBlock);
}

#[test]
fn lowers_last_baseline_alignment_to_layout() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let local = s::Declarations::new().set(
        s::Property::AlignSelf,
        s::Value::AlignItems(s::AlignItems::LastBaseline),
    );
    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();

    let layout = s::adapters::layout::lower(&resolved).expect("style lowers");

    assert_eq!(layout.align_self, Some(l::AlignItems::LastBaseline));
}

#[test]
fn layout_adapter_preserves_subgrid_track_components() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let local = s::Declarations::new().grid_template_columns(s::GridTrackList::new(vec![
        s::GridTrackComponent::Subgrid(s::SubgridTrack::new(vec![vec!["main".to_string()]])),
    ]));
    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();

    let layout = s::adapters::layout::lower(&resolved).unwrap();

    assert_eq!(
        layout.grid_template_columns,
        vec![l::TrackComponent::Subgrid(l::SubgridTrack {
            name_components: vec![l::SubgridLineNameComponent::LineNames(vec![
                "main".to_string(),
            ])],
        })]
    );
}

#[test]
fn subgrid_tracks_are_valid_only_for_grid_template_axes() {
    let subgrid = s::GridTrackList::new(vec![s::GridTrackComponent::Subgrid(
        s::SubgridTrack::new(vec![vec!["main".to_string()]]),
    )]);
    let template = s::GridTemplate::new(
        s::GridTrackList::default(),
        s::GridTrackList::default(),
        s::GridTemplateAreas::default(),
    );

    assert!(
        s::Declaration::try_new(
            s::Property::GridTemplateColumns,
            s::Value::GridTrackList(subgrid.clone()),
        )
        .is_ok()
    );
    assert_eq!(
        s::Declaration::try_new(
            s::Property::GridAutoColumns,
            s::Value::GridTrackList(subgrid.clone()),
        )
        .unwrap_err()
        .code(),
        s::ErrorCode::InvalidValue
    );
    assert_eq!(
        s::Declaration::try_new(
            s::Property::GridAutoRows,
            s::Value::GridTrackList(subgrid.clone()),
        )
        .unwrap_err()
        .code(),
        s::ErrorCode::InvalidValue
    );
    assert_eq!(
        s::Declaration::try_new(
            s::Property::Grid,
            s::Value::GridDefinition(s::GridDefinition::new(template).auto_rows(subgrid)),
        )
        .unwrap_err()
        .code(),
        s::ErrorCode::InvalidValue
    );
}

#[test]
fn layout_adapter_lowers_grid_flow_tolerance_with_resolved_font_size() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());

    let local = s::Declarations::new()
        .font_size(s::Length::px(18.0))
        .grid_flow_tolerance(s::GridFlowTolerance::Normal);
    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();
    let layout = s::adapters::layout::lower(&resolved).unwrap();
    assert_eq!(
        layout.grid_flow_tolerance,
        l::GridFlowTolerance::Normal { font_size: 18.0 }
    );

    let local = s::Declarations::new()
        .grid_flow_tolerance(s::GridFlowTolerance::Length(s::Length::px(12.0)));
    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();
    let layout = s::adapters::layout::lower(&resolved).unwrap();
    assert_eq!(
        layout.grid_flow_tolerance,
        l::GridFlowTolerance::Length(l::Length::px(12.0))
    );

    let local = s::Declarations::new().grid_flow_tolerance(s::GridFlowTolerance::Percent(25.0));
    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();
    let layout = s::adapters::layout::lower(&resolved).unwrap();
    assert_eq!(
        layout.grid_flow_tolerance,
        l::GridFlowTolerance::Percent(0.25)
    );

    let local = s::Declarations::new().grid_flow_tolerance(s::GridFlowTolerance::Infinite);
    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();
    let layout = s::adapters::layout::lower(&resolved).unwrap();
    assert_eq!(layout.grid_flow_tolerance, l::GridFlowTolerance::Infinite);
}

#[test]
fn property_vocabulary_covers_first_pass_style_groups() {
    let required = [
        s::Property::Display,
        s::Property::Position,
        s::Property::Direction,
        s::Property::FlexDirection,
        s::Property::FlexWrap,
        s::Property::FlexGrow,
        s::Property::FlexShrink,
        s::Property::FlexBasis,
        s::Property::Align,
        s::Property::Justify,
        s::Property::Gap,
        s::Property::RowGap,
        s::Property::ColumnGap,
        s::Property::GridTemplateRows,
        s::Property::GridTemplateColumns,
        s::Property::GridTemplateAreas,
        s::Property::GridTemplate,
        s::Property::GridAutoRows,
        s::Property::GridAutoColumns,
        s::Property::GridAutoFlow,
        s::Property::GridRowStart,
        s::Property::GridRowEnd,
        s::Property::GridColumnStart,
        s::Property::GridColumnEnd,
        s::Property::GridRow,
        s::Property::GridColumn,
        s::Property::GridArea,
        s::Property::Grid,
        s::Property::GridFlowTolerance,
        s::Property::Background,
        s::Property::Foreground,
        s::Property::BorderColor,
        s::Property::BorderWidth,
        s::Property::BorderStyle,
        s::Property::Radius,
        s::Property::Shadow,
        s::Property::Opacity,
        s::Property::Visibility,
        s::Property::FontFamily,
        s::Property::FontSize,
        s::Property::FontWeight,
        s::Property::FontStyle,
        s::Property::LineHeight,
        s::Property::Color,
        s::Property::TextAlign,
        s::Property::TextWrap,
        s::Property::WhiteSpace,
        s::Property::WordBreak,
        s::Property::OverflowWrap,
        s::Property::TextOverflow,
        s::Property::TextDecoration,
        s::Property::SelectionColor,
        s::Property::Cursor,
        s::Property::PointerEvents,
        s::Property::FocusOutline,
        s::Property::SelectionPaint,
        s::Property::Transform,
        s::Property::TransformOrigin,
        s::Property::Filter,
        s::Property::TransitionProperty,
        s::Property::TransitionDuration,
        s::Property::TransitionDelay,
        s::Property::TransitionTiming,
        s::Property::AnimationName,
    ];

    for property in required {
        assert!(s::Property::ALL.contains(&property), "{property:?}");
        let metadata = property.metadata();
        metadata.default.validate().unwrap();
    }

    assert!(s::Property::Gap.metadata().impact.layout);
    assert!(s::Property::FontWeight.metadata().impact.text);
    assert!(s::Property::Cursor.metadata().impact.paint);
    assert!(s::Property::TransitionDuration.metadata().impact.animation);
}

#[test]
fn sheet_extend_preserves_order_after_existing_rules() {
    let mut sheet = s::Sheet::new().rule(
        s::Selector::class("base").unwrap(),
        s::Declarations::new().bg(red()),
    );
    let incoming = s::Sheet::new()
        .rule(
            s::Selector::class("second").unwrap(),
            s::Declarations::new().bg(blue()),
        )
        .rule(
            s::Selector::class("third").unwrap(),
            s::Declarations::new().bg(green()),
        );
    let version = sheet.version();

    sheet.extend(incoming.rules().iter().cloned());

    assert_ne!(sheet.version(), version);
    assert_eq!(sheet.rule_count(), 3);
    assert_eq!(
        sheet.rules().iter().map(s::Rule::order).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(sheet.rules_for_class(&class("third")).count(), 1);
}

#[test]
fn sheet_and_rule_accessors_report_public_state() {
    let first = s::Rule::new(
        s::Selector::class("primary").unwrap(),
        s::Declarations::new().bg(red()),
    )
    .when([s::Condition::viewport(s::Viewport::max_height(720.0))]);
    let second_selector = s::Selector::key("save").unwrap();
    let second = s::Rule::new(second_selector.clone(), s::Declarations::new().opacity(0.5));
    let mut sheet = s::Sheet::new();
    let initial_version = sheet.version();

    sheet.extend([first.clone(), second]);

    assert_eq!(first.order(), 0);
    assert!(matches!(first.selector(), s::Selector::Class(_)));
    assert_eq!(
        first.declarations().background(),
        Some(red()),
        "rule declarations are exposed by reference"
    );
    assert_eq!(sheet.rule_count(), 2);
    assert_ne!(sheet.version(), initial_version);
    assert_eq!(sheet.rules().len(), 2);
    assert_eq!(sheet.rules()[0].order(), 0);
    assert_eq!(sheet.rules()[1].order(), 1);
    assert_eq!(sheet.rules_for_selector(&second_selector).count(), 1);
    assert_eq!(sheet.conditional_rules().count(), 1);
    assert_eq!(sheet.unconditional_rules().count(), 1);
    assert_eq!(
        sheet
            .rules_for_key(&key("save"))
            .next()
            .unwrap()
            .declarations()
            .opacity_number(),
        Some(0.5)
    );
    assert_eq!(sheet.rules_for_tag(&tag("button")).count(), 0);
}

#[test]
fn sheet_reports_invalidation_for_viewport_and_container_condition_changes() {
    let sheet = s::Sheet::new()
        .conditional_rule(
            s::Selector::class("wide").unwrap(),
            s::Declarations::new().padding(s::Edges::all(s::Length::px(12.0))),
            [s::Condition::viewport(s::Viewport::min_width(720.0))],
        )
        .conditional_rule(
            s::Selector::class("compact").unwrap(),
            s::Declarations::new().text_color(red()),
            [s::Condition::container(s::Container::max_width(320.0))],
        );

    let viewport = sheet.viewport_change();
    let container = sheet.container_change();

    assert!(viewport.rematch);
    assert!(viewport.scope.node);
    assert!(viewport.scope.descendants);
    assert!(viewport.invalidation.layout);
    assert!(!viewport.invalidation.text);
    assert!(container.rematch);
    assert!(container.scope.node);
    assert!(container.scope.descendants);
    assert!(container.invalidation.text);
    assert!(container.invalidation.paint);
}

#[test]
fn text_values_are_complete_and_validate_owned_strings_and_numbers() {
    let text = s::TextValue::new()
        .family("Inter")
        .family("Arial")
        .size(s::Length::px(15.0))
        .weight(s::TextWeight::Bold)
        .style(s::TextSlant::Italic)
        .line_height(s::Length::percent(120.0))
        .color(red())
        .align(s::TextAlign::Center)
        .wrap(s::TextWrap::Word)
        .white_space(s::WhiteSpace::Preserve)
        .word_break(s::WordBreak::Normal)
        .overflow_wrap(s::OverflowWrap::Anywhere)
        .underline(s::Decoration::solid(None))
        .selection_color(blue());

    assert_eq!(text.font_family, vec!["Inter", "Arial"]);
    assert_eq!(text.font_size, s::Length::px(15.0));
    assert_eq!(text.font_weight, s::TextWeight::Bold);
    assert_eq!(text.font_style, s::TextSlant::Italic);
    text.validate().unwrap();

    assert!(
        s::TextValue::new()
            .family("bad\0family")
            .validate()
            .is_err()
    );
    assert!(
        s::TextValue::new()
            .size(s::Length::px(f32::NAN))
            .validate()
            .is_err()
    );
}

#[test]
fn resolver_rejects_invalid_values_even_when_declarations_were_built_infallibly() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut local = s::Declarations::new();
    local.insert(s::Property::Opacity, s::Value::Number(f32::NAN));
    let mut resolver = s::Resolver::new(s::Sheet::new());

    let error = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap_err();

    assert_eq!(error.code(), s::ErrorCode::InvalidValue);
}

#[test]
fn resolver_rejects_property_value_mismatches_built_infallibly() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut local = s::Declarations::new();
    local.insert(s::Property::Padding, s::Value::Color(red()));
    let mut resolver = s::Resolver::new(s::Sheet::new());

    let error = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap_err();

    assert_eq!(error.code(), s::ErrorCode::InvalidProperty);
}

#[test]
fn resolver_resolves_style_keywords_against_parent_and_defaults() {
    let mut child = FixtureNode::new("button");
    child.parent = Some(0);
    let tree = FixtureTree::new(vec![FixtureNode::new("root"), child]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let parent_local = s::Declarations::new()
        .text_color(red())
        .padding(s::Edges::all(s::Length::px(9.0)));
    let parent = resolver
        .resolve(s::Context::new(&tree, 0).local(&parent_local))
        .unwrap();

    let inherit = s::Declarations::new()
        .set(s::Property::Color, s::Value::Keyword(s::Keyword::Inherit))
        .set(s::Property::Padding, s::Value::Keyword(s::Keyword::Inherit));
    let initial = s::Declarations::new()
        .set(s::Property::Color, s::Value::Keyword(s::Keyword::Initial))
        .set(s::Property::Padding, s::Value::Keyword(s::Keyword::Initial));
    let unset = s::Declarations::new()
        .set(s::Property::Color, s::Value::Keyword(s::Keyword::Unset))
        .set(s::Property::Padding, s::Value::Keyword(s::Keyword::Unset));

    let inherited = resolver
        .resolve(s::Context::new(&tree, 1).parent(&parent).local(&inherit))
        .unwrap();
    let initialed = resolver
        .resolve(s::Context::new(&tree, 1).parent(&parent).local(&initial))
        .unwrap();
    let unset = resolver
        .resolve(s::Context::new(&tree, 1).parent(&parent).local(&unset))
        .unwrap();

    assert_eq!(inherited.text_color(), red());
    assert_eq!(inherited.padding_edges(), s::Edges::all(s::Length::px(9.0)));
    assert_eq!(initialed.text_color(), s::Color::BLACK);
    assert_eq!(initialed.padding_edges(), s::Edges::default());
    assert_eq!(unset.text_color(), red());
    assert_eq!(unset.padding_edges(), s::Edges::default());
}

#[test]
fn stroke_values_carry_dash_sides_alignment_and_validate_dash_facts() {
    let stroke = s::Stroke::new(s::Length::px(2.0), red())
        .sides(s::SideSet::horizontal())
        .style(s::LineStyle::Dashed)
        .dash(s::Dash::dashed().density(1.5).phase(2.0))
        .align(s::StrokeAlign::Inside);

    assert_eq!(stroke.sides, s::SideSet::horizontal());
    assert_eq!(stroke.align, s::StrokeAlign::Inside);
    stroke.validate().unwrap();

    let invalid = s::Stroke::new(s::Length::px(1.0), blue()).dash(s::Dash::new(0.0));
    assert!(invalid.validate().is_err());
}

#[test]
fn resolved_exposes_typed_accessors_for_expanded_common_values() {
    let resolved = s::Resolved::new();

    assert_eq!(resolved.margin_edges(), s::Edges::default());
    assert_eq!(resolved.radius_corners(), s::Corners::default());
    assert_eq!(resolved.opacity(), 1.0);
    assert_eq!(resolved.font_size(), s::Length::px(16.0));
    assert_eq!(resolved.cursor(), s::Cursor::Default);
    assert_eq!(resolved.pointer_events(), s::PointerEvents::Auto);
}

#[test]
fn declarations_and_resolved_expose_typed_accessors_for_property_groups() {
    let border = s::Edges::all(s::Length::px(2.0));
    let transform_origin = s::Size::new(s::Length::percent(25.0), s::Length::percent(75.0));
    let transform = s::Transform {
        operations: vec![s::TransformOp::Scale { x: 2.0, y: 3.0 }],
    };
    let transition_properties = vec![s::Property::Opacity, s::Property::Transform];
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let local = s::Declarations::new()
        .width(s::Length::px(80.0))
        .height(s::Length::percent(50.0))
        .border_width(border.clone())
        .border_color(red())
        .visibility(s::Visibility::Hidden)
        .transform(transform.clone())
        .transform_origin(transform_origin.clone())
        .transition_properties(transition_properties.clone())
        .transition_duration(125.0)
        .transition_delay(25.0);

    assert_eq!(local.width_length(), Some(s::Length::px(80.0)));
    assert_eq!(local.height_length(), Some(s::Length::percent(50.0)));
    assert_eq!(local.border_width_edges(), Some(border.clone()));
    assert_eq!(
        local.get(s::Property::BorderColor),
        Some(&s::Value::Color(red()))
    );
    assert_eq!(local.visibility_state(), Some(s::Visibility::Hidden));
    assert_eq!(local.transform_value(), Some(&transform));
    assert_eq!(
        local.transform_origin_size(),
        Some(transform_origin.clone())
    );
    assert_eq!(
        local.transition_property_list(),
        Some(transition_properties.as_slice())
    );
    assert_eq!(local.transition_duration_number(), Some(125.0));
    assert_eq!(local.transition_delay_number(), Some(25.0));

    let resolved = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();

    assert_eq!(resolved.width(), s::Length::px(80.0));
    assert_eq!(resolved.height(), s::Length::percent(50.0));
    assert_eq!(resolved.border_width_edges(), border);
    assert_eq!(resolved.border_color(), red());
    assert_eq!(resolved.visibility(), s::Visibility::Hidden);
    assert_eq!(resolved.transform(), &transform);
    assert_eq!(resolved.transform_origin(), transform_origin);
    assert_eq!(
        resolved.transition_properties(),
        transition_properties.as_slice()
    );
    assert_eq!(resolved.transition_duration(), 125.0);
    assert_eq!(resolved.transition_delay(), 25.0);
}

#[test]
fn selectors_match_tags_classes_keys_states_attributes_and_positions() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root"), {
        let mut node = FixtureNode::new("button");
        node.parent = Some(0);
        node.key = Some(key("submit"));
        node.classes = vec![class("primary"), class("large")];
        node.attributes = vec![r::Attribute::new(attr("data-kind"), value("hero"))];
        node.state.hovered = true;
        node
    }]);
    let id = 1;

    assert!(
        s::Selector::tag("button")
            .unwrap()
            .matches(&tree, id, s::Traversal::Canonical)
            .unwrap()
    );
    assert!(
        s::Selector::class("primary")
            .unwrap()
            .matches(&tree, id, s::Traversal::Canonical)
            .unwrap()
    );
    assert!(
        s::Selector::key("submit")
            .unwrap()
            .matches(&tree, id, s::Traversal::Canonical)
            .unwrap()
    );
    assert!(
        s::Selector::state(s::StateFlag::Hovered)
            .matches(&tree, id, s::Traversal::Canonical)
            .unwrap()
    );
    assert!(
        s::Selector::attribute_exists("data-kind")
            .unwrap()
            .matches(&tree, id, s::Traversal::Canonical)
            .unwrap()
    );
    assert!(
        s::Selector::attribute_equals("data-kind", "hero")
            .unwrap()
            .matches(&tree, id, s::Traversal::Canonical)
            .unwrap()
    );
    assert!(
        s::Selector::position(s::PositionSelector::First)
            .matches(&tree, id, s::Traversal::Canonical)
            .unwrap()
    );
    assert!(
        s::Selector::position(s::PositionSelector::Last)
            .matches(&tree, id, s::Traversal::Canonical)
            .unwrap()
    );
    assert!(
        s::Selector::position(s::PositionSelector::Nth(s::Nth::new(1, 0)))
            .matches(&tree, id, s::Traversal::Canonical)
            .unwrap()
    );
}

#[test]
fn selector_constructors_validate_retained_strings() {
    assert!(s::Selector::attribute_equals("data-value", "").is_ok());
    assert!(
        s::Compound::new()
            .attribute_equals("data-value", "")
            .is_ok()
    );

    for error in [
        s::Selector::tag("").unwrap_err(),
        s::Selector::class("").unwrap_err(),
        s::Selector::key("").unwrap_err(),
        s::Selector::attribute_exists("").unwrap_err(),
        s::Selector::attribute_equals("data-value", "\u{0000}").unwrap_err(),
        s::Compound::new().tag("").unwrap_err(),
        s::Compound::new().key("").unwrap_err(),
        s::Compound::new().class("").unwrap_err(),
        s::Compound::new().attribute_exists("").unwrap_err(),
        s::Compound::new()
            .attribute_equals("data-value", "\u{0000}")
            .unwrap_err(),
    ] {
        assert_eq!(error.code(), s::ErrorCode::InvalidSelector);
    }
}

#[test]
fn positions_and_nth_selectors_cover_index_math() {
    let first = s::Position::new(0, 3);
    let middle = s::Position::new(1, 3);
    let last = s::Position::new(2, 3);

    assert!(first.is_first());
    assert!(!first.is_last());
    assert!(!middle.is_first());
    assert!(!middle.is_last());
    assert!(last.is_last());
    assert!(s::Nth::odd().matches(1));
    assert!(s::Nth::odd().matches(3));
    assert!(!s::Nth::odd().matches(2));
    assert!(s::Nth::even().matches(2));
    assert!(!s::Nth::even().matches(3));
    assert!(s::Nth::new(0, 3).matches(3));
    assert!(!s::Nth::new(0, 3).matches(4));
    assert!(middle.matches(s::Nth::new(3, 2)));
    assert!(!s::Nth::new(3, 2).matches(0));
}

#[test]
fn complex_selectors_match_descendant_child_adjacent_and_sibling_paths() {
    let tree = FixtureTree::new(vec![
        FixtureNode::new("root"),
        FixtureNode {
            parent: Some(0),
            classes: vec![class("panel")],
            ..FixtureNode::new("section")
        },
        FixtureNode {
            parent: Some(1),
            classes: vec![class("label")],
            ..FixtureNode::new("span")
        },
        FixtureNode {
            parent: Some(1),
            classes: vec![class("control")],
            ..FixtureNode::new("button")
        },
    ]);

    let panel = s::Compound::new().class("panel").unwrap();
    let label = s::Compound::new().class("label").unwrap();
    let control = s::Compound::new().class("control").unwrap();

    assert!(
        s::Selector::complex([
            s::Part::root(panel.clone()),
            s::Part::related(s::Combinator::Descendant, control.clone()),
        ])
        .unwrap()
        .matches(&tree, 3, s::Traversal::Canonical)
        .unwrap()
    );
    assert!(
        s::Selector::complex([
            s::Part::root(panel),
            s::Part::related(s::Combinator::Child, control.clone()),
        ])
        .unwrap()
        .matches(&tree, 3, s::Traversal::Canonical)
        .unwrap()
    );
    assert!(
        s::Selector::complex([
            s::Part::root(label.clone()),
            s::Part::related(s::Combinator::Adjacent, control.clone()),
        ])
        .unwrap()
        .matches(&tree, 3, s::Traversal::Canonical)
        .unwrap()
    );
    assert!(
        s::Selector::complex([
            s::Part::root(label),
            s::Part::related(s::Combinator::Sibling, control),
        ])
        .unwrap()
        .matches(&tree, 3, s::Traversal::Canonical)
        .unwrap()
    );
}

#[test]
fn complex_selectors_backtrack_descendant_candidates() {
    let tree = FixtureTree::new(vec![
        FixtureNode::new("root"),
        FixtureNode {
            parent: Some(0),
            classes: vec![class("anchor")],
            ..FixtureNode::new("section")
        },
        FixtureNode {
            parent: Some(1),
            classes: vec![class("item")],
            ..FixtureNode::new("div")
        },
        FixtureNode {
            parent: Some(2),
            ..FixtureNode::new("div")
        },
        FixtureNode {
            parent: Some(3),
            classes: vec![class("item")],
            ..FixtureNode::new("div")
        },
        FixtureNode {
            parent: Some(4),
            classes: vec![class("target")],
            ..FixtureNode::new("button")
        },
    ]);

    let anchor = s::Compound::new().class("anchor").unwrap();
    let item = s::Compound::new().class("item").unwrap();
    let target = s::Compound::new().class("target").unwrap();

    assert!(
        s::Selector::complex([
            s::Part::root(anchor),
            s::Part::related(s::Combinator::Child, item),
            s::Part::related(s::Combinator::Descendant, target),
        ])
        .unwrap()
        .matches(&tree, 5, s::Traversal::Canonical)
        .unwrap()
    );
}

#[test]
fn complex_selectors_backtrack_sibling_candidates() {
    let tree = FixtureTree::new(vec![
        FixtureNode::new("root"),
        FixtureNode {
            parent: Some(0),
            classes: vec![class("anchor")],
            ..FixtureNode::new("div")
        },
        FixtureNode {
            parent: Some(0),
            classes: vec![class("item")],
            ..FixtureNode::new("div")
        },
        FixtureNode {
            parent: Some(0),
            ..FixtureNode::new("div")
        },
        FixtureNode {
            parent: Some(0),
            classes: vec![class("item")],
            ..FixtureNode::new("div")
        },
        FixtureNode {
            parent: Some(0),
            classes: vec![class("target")],
            ..FixtureNode::new("button")
        },
    ]);

    let anchor = s::Compound::new().class("anchor").unwrap();
    let item = s::Compound::new().class("item").unwrap();
    let target = s::Compound::new().class("target").unwrap();

    assert!(
        s::Selector::complex([
            s::Part::root(anchor),
            s::Part::related(s::Combinator::Adjacent, item),
            s::Part::related(s::Combinator::Sibling, target),
        ])
        .unwrap()
        .matches(&tree, 5, s::Traversal::Canonical)
        .unwrap()
    );
}

#[test]
fn complex_selector_validation_rejects_malformed_shapes() {
    let selector = s::Compound::new().class("control").unwrap();

    let error = s::Selector::try_complex(Vec::<s::Part>::new()).unwrap_err();
    assert_eq!(error.code(), s::ErrorCode::InvalidSelector);
    let error = s::Selector::complex(Vec::<s::Part>::new()).unwrap_err();
    assert_eq!(error.code(), s::ErrorCode::InvalidSelector);

    let error =
        s::Selector::try_complex([s::Part::related(s::Combinator::Child, selector.clone())])
            .unwrap_err();
    assert_eq!(error.code(), s::ErrorCode::InvalidSelector);

    let error =
        s::Selector::try_complex([s::Part::root(selector.clone()), s::Part::root(selector)])
            .unwrap_err();
    assert_eq!(error.code(), s::ErrorCode::InvalidSelector);
}

#[test]
fn public_complex_selector_variant_reports_malformed_shapes() {
    let tree = FixtureTree::new(vec![FixtureNode::new("button")]);
    let selector = s::Compound::new().class("control").unwrap();
    let malformed = s::Selector::Complex(vec![s::Part::related(s::Combinator::Child, selector)]);

    let error = malformed
        .matches(&tree, 0, s::Traversal::Canonical)
        .unwrap_err();

    assert_eq!(error.code(), s::ErrorCode::InvalidSelector);
}

#[test]
fn conditions_require_all_viewport_and_container_queries_to_match() {
    let conditions = [
        s::Condition::viewport(s::Viewport::min_width(640.0)),
        s::Condition::viewport(s::Viewport::max_height(900.0)),
        s::Condition::container(s::Container::max_width(420.0)),
    ];

    assert!(s::Condition::matches_all(
        &conditions,
        s::Viewport::new(800.0, 600.0),
        Some(s::Container::new(320.0, 240.0))
    ));
    assert!(!s::Condition::matches_all(
        &conditions,
        s::Viewport::new(500.0, 600.0),
        Some(s::Container::new(320.0, 240.0))
    ));
    assert!(!s::Condition::matches_all(
        &conditions,
        s::Viewport::new(800.0, 600.0),
        None
    ));
}

#[test]
fn sheet_indexes_reduce_candidate_rules() {
    let mut node = FixtureNode::new("button");
    node.parent = Some(0);
    node.key = Some(key("submit"));
    node.classes = vec![class("primary")];
    let tree = FixtureTree::new(vec![FixtureNode::new("root"), node]);

    let sheet = s::Sheet::new()
        .rule(
            s::Selector::any(),
            s::Declarations::new().text_color(white()),
        )
        .rule(
            s::Selector::class("primary").unwrap(),
            s::Declarations::new().bg(blue()),
        )
        .rule(
            s::Selector::class("secondary").unwrap(),
            s::Declarations::new().bg(red()),
        )
        .rule(
            s::Selector::tag("button").unwrap(),
            s::Declarations::new().padding(s::Edges::all(s::Length::px(6.0))),
        )
        .rule(
            s::Selector::key("submit").unwrap(),
            s::Declarations::new().radius(s::Corners::all(s::Length::px(4.0))),
        );

    assert_eq!(sheet.rule_count(), 5);
    assert_eq!(sheet.candidate_rule_count(&tree, 1).unwrap(), 4);
}

#[test]
fn resolver_applies_defaults_inheritance_rules_local_and_animated_overlays() {
    let mut child = FixtureNode::new("button");
    child.parent = Some(0);
    child.classes = vec![class("primary")];
    let tree = FixtureTree::new(vec![FixtureNode::new("root"), child]);

    let sheet = s::Sheet::new()
        .rule(
            s::Selector::tag("root").unwrap(),
            s::Declarations::new().text_color(red()),
        )
        .rule(
            s::Selector::class("primary").unwrap(),
            s::Declarations::new().bg(blue()),
        );
    let mut resolver = s::Resolver::new(sheet);
    let viewport = s::Viewport::new(800.0, 600.0);
    let root = resolver
        .resolve(s::Context::new(&tree, 0).viewport(viewport))
        .unwrap();
    let local = s::Declarations::new().padding(s::Edges::all(s::Length::px(8.0)));
    let animated = s::Declarations::new().bg(green());

    let resolved = resolver
        .resolve(
            s::Context::new(&tree, 1)
                .viewport(viewport)
                .parent(&root)
                .local(&local)
                .animated(&animated),
        )
        .unwrap();

    assert_eq!(resolved.text_color(), red());
    assert_eq!(resolved.background(), green());
    assert_eq!(resolved.padding_edges(), s::Edges::all(s::Length::px(8.0)));
}

#[test]
fn resolver_cache_reuses_entries_only_with_matching_identity() {
    let mut child = FixtureNode::new("button");
    child.parent = Some(0);
    child.classes = vec![class("primary")];
    let tree = FixtureTree::new(vec![FixtureNode::new("root"), child]);
    let sheet = s::Sheet::new().rule(
        s::Selector::class("primary").unwrap(),
        s::Declarations::new().bg(blue()),
    );
    let mut resolver = s::Resolver::new(sheet);
    let viewport = s::Viewport::new(800.0, 600.0);

    let first = resolver
        .resolve(s::Context::new(&tree, 1).viewport(viewport))
        .unwrap();
    let second = resolver
        .resolve(s::Context::new(&tree, 1).viewport(viewport))
        .unwrap();
    let canonical = resolver
        .resolve(
            s::Context::new(&tree, 1)
                .viewport(viewport)
                .traversal(s::Traversal::Canonical),
        )
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(first, canonical);
    assert_eq!(resolver.cache_hits(), 1);
}

#[test]
fn resolver_cache_key_tracks_inherited_parent_style() {
    let mut child = FixtureNode::new("button");
    child.parent = Some(0);
    let tree = FixtureTree::new(vec![FixtureNode::new("root"), child]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let viewport = s::Viewport::new(800.0, 600.0);

    let red_parent = resolver
        .resolve(
            s::Context::new(&tree, 0)
                .viewport(viewport)
                .local(&s::Declarations::new().text_color(red())),
        )
        .unwrap();
    let blue_parent = resolver
        .resolve(
            s::Context::new(&tree, 0)
                .viewport(viewport)
                .local(&s::Declarations::new().text_color(blue())),
        )
        .unwrap();

    let red_child = resolver
        .resolve(
            s::Context::new(&tree, 1)
                .viewport(viewport)
                .parent(&red_parent),
        )
        .unwrap();
    let blue_child = resolver
        .resolve(
            s::Context::new(&tree, 1)
                .viewport(viewport)
                .parent(&blue_parent),
        )
        .unwrap();

    assert_eq!(red_child.text_color(), red());
    assert_eq!(blue_child.text_color(), blue());
}

#[test]
fn retained_snapshot_adapts_canonical_and_default_projected_traversal() {
    let root = r::Element::root().with_child(
        r::Element::tagged(tag("button"))
            .with_key(key("submit"))
            .with_class(class("primary")),
    );
    let model = r::Model::new(root).unwrap();
    let snapshot = model.snapshot();
    let child = snapshot.children(snapshot.root()).unwrap().next().unwrap();

    assert!(
        s::Selector::class("primary")
            .unwrap()
            .matches(&snapshot, child, s::Traversal::Canonical)
            .unwrap()
    );
    assert!(
        s::Selector::key("submit")
            .unwrap()
            .matches(&snapshot, child, s::Traversal::Projected)
            .unwrap()
    );
    assert_eq!(
        snapshot
            .children(snapshot.root())
            .unwrap()
            .collect::<Vec<_>>(),
        s::Tree::children(&snapshot, snapshot.root(), s::Traversal::Projected)
            .unwrap()
            .collect::<Vec<_>>()
    );
}

#[test]
fn nodes_report_each_state_flag_explicitly() {
    let mut state = r::State::default();
    state.hovered = true;
    state.active = true;
    state.focused = true;
    state.focus_within = true;
    state.pointer_captured = true;
    state.disabled = true;
    state.selected = true;
    state.pressed = true;
    state.checked = Some(true);
    state.expanded = Some(true);
    let node = s::Node {
        id: 0usize,
        tag: None,
        key: None,
        classes: &[],
        attributes: &[],
        role: r::Role::Generic,
        state: &state,
        text: false,
    };
    let mut unchecked = r::State::default();
    unchecked.checked = Some(false);
    unchecked.expanded = Some(false);
    let unchecked_node = s::Node {
        state: &unchecked,
        ..node.clone()
    };

    for flag in [
        s::StateFlag::Hovered,
        s::StateFlag::Active,
        s::StateFlag::Focused,
        s::StateFlag::FocusWithin,
        s::StateFlag::PointerCaptured,
        s::StateFlag::Disabled,
        s::StateFlag::Selected,
        s::StateFlag::Pressed,
        s::StateFlag::Checked,
        s::StateFlag::Expanded,
    ] {
        assert!(node.has_state(flag), "{flag:?}");
    }
    assert!(!unchecked_node.has_state(s::StateFlag::Checked));
    assert!(!unchecked_node.has_state(s::StateFlag::Expanded));
}

#[test]
fn invalidation_classifies_changed_properties_and_retained_changes() {
    let property_invalidation =
        s::Invalidation::from_properties([s::Property::Padding, s::Property::Color]);
    assert!(property_invalidation.layout);
    assert!(property_invalidation.text);
    assert!(property_invalidation.paint);
    assert!(!property_invalidation.effect);

    let style_change = s::Change::from_retained_flags(r::ChangeFlags::empty().classes().state());
    assert!(style_change.rematch);
    assert!(!style_change.invalidation.layout);

    let text_change = s::Change::from_retained_flags(r::ChangeFlags::empty().text());
    assert!(!text_change.rematch);
    assert!(text_change.invalidation.text);
}

#[test]
fn changes_report_scope_for_structure_projection_and_inheritance() {
    let structure = s::Change::from_retained_flags(r::ChangeFlags::empty().structure());
    assert!(structure.rematch);
    assert!(structure.scope.node);
    assert!(structure.scope.siblings);
    assert!(structure.scope.descendants);

    let projection = s::Change::from_retained_flags(r::ChangeFlags::empty().projection());
    assert!(projection.rematch);
    assert!(projection.scope.node);
    assert!(projection.scope.siblings);
    assert!(projection.scope.descendants);

    let inherited = s::Change::from_properties([s::Property::Color]);
    assert!(inherited.invalidation.text);
    assert!(inherited.scope.node);
    assert!(inherited.scope.descendants);

    let local = s::Change::from_properties([s::Property::Padding]);
    assert!(local.invalidation.layout);
    assert!(local.scope.node);
    assert!(!local.scope.descendants);
}

#[test]
fn large_tree_resolution_uses_indexes_and_cache() {
    let mut nodes = Vec::with_capacity(10_001);
    nodes.push(FixtureNode::new("root"));
    for _ in 0..10_000 {
        nodes.push(FixtureNode {
            parent: Some(0),
            classes: vec![class("row")],
            ..FixtureNode::new("div")
        });
    }
    let tree = FixtureTree::new(nodes);
    let sheet = s::Sheet::new()
        .rule(
            s::Selector::class("row").unwrap(),
            s::Declarations::new().bg(blue()),
        )
        .rule(
            s::Selector::class("other").unwrap(),
            s::Declarations::new().bg(red()),
        );
    let mut resolver = s::Resolver::new(sheet);
    let viewport = s::Viewport::new(1024.0, 768.0);

    assert_eq!(
        resolver
            .sheet()
            .candidate_rule_count(&tree, 10_000)
            .unwrap(),
        1
    );
    let first = resolver
        .resolve(s::Context::new(&tree, 10_000).viewport(viewport))
        .unwrap();
    let second = resolver
        .resolve(s::Context::new(&tree, 10_000).viewport(viewport))
        .unwrap();

    assert_eq!(first.background(), blue());
    assert_eq!(first, second);
    assert_eq!(resolver.cache_hits(), 1);
}

#[test]
fn sheet_queries_and_versions_are_deterministic() {
    let primary = s::Selector::class("primary").unwrap();
    let secondary = s::Selector::class("secondary").unwrap();
    let condition = s::Condition::viewport(s::Viewport::min_width(640.0));
    let sheet = s::Sheet::new()
        .rule(primary.clone(), s::Declarations::new().bg(blue()))
        .conditional_rule(secondary, s::Declarations::new().bg(red()), [condition]);

    assert_eq!(sheet.rules_for_selector(&primary).count(), 1);
    assert_eq!(sheet.rules_for_class(&class("primary")).count(), 1);
    assert_eq!(sheet.conditional_rules().count(), 1);
    assert_eq!(sheet.unconditional_rules().count(), 1);

    let cloned = sheet.clone();
    assert_eq!(sheet.version(), cloned.version());
    let changed = cloned.rule(
        s::Selector::any(),
        s::Declarations::new().text_color(white()),
    );
    assert_ne!(sheet.version(), changed.version());
}

#[test]
fn resolver_cache_can_be_cleared_and_skips_unversioned_trees() {
    let sheet = s::Sheet::new().rule(
        s::Selector::class("primary").unwrap(),
        s::Declarations::new().bg(blue()),
    );
    let mut node = FixtureNode::new("button");
    node.parent = Some(0);
    node.classes = vec![class("primary")];
    let tree = FixtureTree::new(vec![FixtureNode::new("root"), node]);
    let mut resolver = s::Resolver::new(sheet.clone());
    let viewport = s::Viewport::new(800.0, 600.0);

    resolver
        .resolve(s::Context::new(&tree, 1).viewport(viewport))
        .unwrap();
    resolver
        .resolve(s::Context::new(&tree, 1).viewport(viewport))
        .unwrap();
    assert_eq!(resolver.cache_hits(), 1);

    resolver.clear_cache_for_sheet(sheet.version());
    resolver
        .resolve(s::Context::new(&tree, 1).viewport(viewport))
        .unwrap();
    resolver
        .resolve(s::Context::new(&tree, 1).viewport(viewport))
        .unwrap();
    assert_eq!(resolver.cache_hits(), 1);

    let root = r::Element::root()
        .with_child(r::Element::tagged(tag("button")).with_class(class("primary")));
    let model = r::Model::new(root).unwrap();
    let snapshot = model.snapshot();
    let child = snapshot.children(snapshot.root()).unwrap().next().unwrap();
    let mut retained = s::Resolver::new(sheet.clone());

    retained
        .resolve(s::Context::new(&snapshot, child).viewport(viewport))
        .unwrap();
    retained
        .resolve(s::Context::new(&snapshot, child).viewport(viewport))
        .unwrap();
    assert_eq!(retained.cache_hits(), 1);

    let mut unversioned = s::Resolver::new(sheet);
    let unversioned_tree = UnversionedTree(&tree);
    unversioned
        .resolve(s::Context::new(&unversioned_tree, 1).viewport(viewport))
        .unwrap();
    unversioned
        .resolve(s::Context::new(&unversioned_tree, 1).viewport(viewport))
        .unwrap();
    assert_eq!(unversioned.cache_hits(), 0);
}

#[test]
fn resolver_cache_can_clear_one_node_without_dropping_unrelated_entries() {
    let tree = FixtureTree::new(vec![
        FixtureNode::new("root"),
        FixtureNode {
            parent: Some(0),
            classes: vec![class("row")],
            ..FixtureNode::new("div")
        },
        FixtureNode {
            parent: Some(0),
            classes: vec![class("row")],
            ..FixtureNode::new("div")
        },
    ]);
    let sheet = s::Sheet::new().rule(
        s::Selector::class("row").unwrap(),
        s::Declarations::new().bg(blue()),
    );
    let mut resolver = s::Resolver::new(sheet);
    let viewport = s::Viewport::new(800.0, 600.0);

    resolver
        .resolve(s::Context::new(&tree, 1).viewport(viewport))
        .unwrap();
    resolver
        .resolve(s::Context::new(&tree, 2).viewport(viewport))
        .unwrap();
    resolver
        .resolve(s::Context::new(&tree, 1).viewport(viewport))
        .unwrap();
    resolver
        .resolve(s::Context::new(&tree, 2).viewport(viewport))
        .unwrap();
    assert_eq!(resolver.cache_hits(), 2);

    resolver.clear_cache_for_node(1usize);
    resolver
        .resolve(s::Context::new(&tree, 2).viewport(viewport))
        .unwrap();
    assert_eq!(resolver.cache_hits(), 3);

    resolver
        .resolve(s::Context::new(&tree, 1).viewport(viewport))
        .unwrap();
    assert_eq!(resolver.cache_hits(), 3);
}

#[test]
fn resolved_comparison_classifies_downstream_invalidation() {
    let tree = FixtureTree::new(vec![FixtureNode::new("root")]);
    let mut resolver = s::Resolver::new(s::Sheet::new());
    let before = s::Resolved::new();
    let local = s::Declarations::new()
        .padding(s::Edges::all(s::Length::px(8.0)))
        .text_color(red());
    let after = resolver
        .resolve(s::Context::new(&tree, 0).local(&local))
        .unwrap();

    let invalidation = s::Invalidation::between(&before, &after);

    assert!(invalidation.layout);
    assert!(invalidation.paint);
    assert!(invalidation.text);
    assert!(!invalidation.effect);
}

struct FixtureTree {
    nodes: Vec<FixtureNode>,
}

impl FixtureTree {
    fn new(mut nodes: Vec<FixtureNode>) -> Self {
        let parent_pairs: Vec<_> = nodes
            .iter()
            .enumerate()
            .filter_map(|(id, node)| node.parent.map(|parent| (id, parent)))
            .collect();
        for (id, parent) in parent_pairs {
            nodes[parent].children.push(id);
            nodes[parent].projected_children.push(id);
        }
        Self { nodes }
    }
}

impl s::Tree for FixtureTree {
    type Id = usize;

    fn version_hint(&self) -> Option<u64> {
        Some(1)
    }

    fn node(&self, id: Self::Id) -> s::Result<s::Node<'_, Self::Id>> {
        let node = self.nodes.get(id).ok_or_else(|| {
            s::Error::new(
                s::ErrorCode::MissingNode,
                format!("missing fixture node {id}"),
            )
        })?;
        Ok(s::Node {
            id,
            tag: Some(&node.tag),
            key: node.key.as_ref(),
            classes: &node.classes,
            attributes: &node.attributes,
            role: node.role.clone(),
            state: &node.state,
            text: node.text,
        })
    }

    fn parent(&self, id: Self::Id, _traversal: s::Traversal) -> s::Result<Option<Self::Id>> {
        Ok(self.nodes.get(id).and_then(|node| node.parent))
    }

    fn children(
        &self,
        id: Self::Id,
        traversal: s::Traversal,
    ) -> s::Result<impl Iterator<Item = Self::Id> + '_> {
        let node = self.nodes.get(id).ok_or_else(|| {
            s::Error::new(
                s::ErrorCode::MissingNode,
                format!("missing fixture node {id}"),
            )
        })?;
        let children = match traversal {
            s::Traversal::Canonical => &node.children,
            s::Traversal::Projected => &node.projected_children,
        };
        Ok(children.iter().copied())
    }

    fn previous_sibling(
        &self,
        id: Self::Id,
        traversal: s::Traversal,
    ) -> s::Result<Option<Self::Id>> {
        let Some(parent) = self.parent(id, traversal)? else {
            return Ok(None);
        };
        let siblings: Vec<_> = self.children(parent, traversal)?.collect();
        Ok(siblings
            .iter()
            .position(|sibling| *sibling == id)
            .and_then(|index| index.checked_sub(1))
            .map(|index| siblings[index]))
    }
}

struct UnversionedTree<'a>(&'a FixtureTree);

impl s::Tree for UnversionedTree<'_> {
    type Id = usize;

    fn version_hint(&self) -> Option<u64> {
        None
    }

    fn node(&self, id: Self::Id) -> s::Result<s::Node<'_, Self::Id>> {
        self.0.node(id)
    }

    fn parent(&self, id: Self::Id, traversal: s::Traversal) -> s::Result<Option<Self::Id>> {
        self.0.parent(id, traversal)
    }

    fn children(
        &self,
        id: Self::Id,
        traversal: s::Traversal,
    ) -> s::Result<impl Iterator<Item = Self::Id> + '_> {
        self.0.children(id, traversal)
    }

    fn previous_sibling(
        &self,
        id: Self::Id,
        traversal: s::Traversal,
    ) -> s::Result<Option<Self::Id>> {
        self.0.previous_sibling(id, traversal)
    }
}

struct FixtureNode {
    parent: Option<usize>,
    children: Vec<usize>,
    projected_children: Vec<usize>,
    tag: r::Tag,
    key: Option<r::Key>,
    classes: Vec<r::Class>,
    attributes: Vec<r::Attribute>,
    role: r::Role,
    state: r::State,
    text: bool,
}

impl FixtureNode {
    fn new(tag: &str) -> Self {
        Self {
            parent: None,
            children: Vec::new(),
            projected_children: Vec::new(),
            tag: r::Tag::new(tag).unwrap(),
            key: None,
            classes: Vec::new(),
            attributes: Vec::new(),
            role: r::Role::Generic,
            state: r::State::default(),
            text: false,
        }
    }
}

fn key(value: &str) -> r::Key {
    r::Key::new(value).unwrap()
}

fn class(value: &str) -> r::Class {
    r::Class::new(value).unwrap()
}

fn attr(value: &str) -> r::AttributeName {
    r::AttributeName::new(value).unwrap()
}

fn tag(value: &str) -> r::Tag {
    r::Tag::new(value).unwrap()
}

fn value(value: &str) -> r::Value {
    r::Value::new(value).unwrap()
}

fn finite_style_number() -> impl Strategy<Value = f32> {
    -1_000_000.0f32..1_000_000.0
}

fn generated_color() -> impl Strategy<Value = s::Color> {
    (
        finite_style_number(),
        finite_style_number(),
        finite_style_number(),
        finite_style_number(),
    )
        .prop_map(|(r, g, b, a)| s::Color::rgba(r, g, b, a))
}

fn red() -> s::Color {
    s::Color::rgba(1.0, 0.0, 0.0, 1.0)
}

fn green() -> s::Color {
    s::Color::rgba(0.0, 1.0, 0.0, 1.0)
}

fn blue() -> s::Color {
    s::Color::rgba(0.0, 0.0, 1.0, 1.0)
}

fn white() -> s::Color {
    s::Color::rgba(1.0, 1.0, 1.0, 1.0)
}
