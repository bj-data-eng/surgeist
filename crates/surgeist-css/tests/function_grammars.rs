use surgeist_css::*;

fn parsed_transform_property(value: &str) -> CssTransformPropertyValue {
    let report = parse_style_attribute(&format!("transform: {value}"));
    assert!(
        report.is_clean(),
        "expected `{value}` to parse cleanly, got {:?}",
        report.diagnostics()
    );
    let declaration = report.syntax()[0]
        .known()
        .expect("known transform declaration");
    let CssKnownPropertyValueRef::Transform(value) = declaration
        .property_value()
        .expect("ordinary transform value")
    else {
        panic!("expected transform property value");
    };
    value.clone()
}

fn assert_transform_rejected(value: &str) {
    let report = parse_style_attribute(&format!("transform: {value}"));
    assert!(report.syntax().is_empty(), "retained invalid `{value}`");
    assert_eq!(
        report.diagnostics().len(),
        1,
        "expected one diagnostic for `{value}`"
    );
}

fn assert_easing_rejected(value: &str) {
    let source = format!("transition-timing-function: {value}; color: red");
    let report = parse_style_attribute(&source);
    assert_eq!(
        report.syntax().len(),
        1,
        "retained invalid easing `{value}`: {:?}",
        report.syntax()
    );
    assert_eq!(
        report.syntax()[0]
            .known()
            .expect("retained color sibling")
            .property(),
        CssKnownProperty::Color,
    );
    assert_eq!(
        report.diagnostics().len(),
        1,
        "expected one diagnostic for `{value}`"
    );
}

fn assert_filter_rejected(value: &str) {
    let source = format!("filter: {value}; color: red");
    let report = parse_style_attribute(&source);
    assert_eq!(
        report.syntax().len(),
        1,
        "retained invalid filter `{value}`: {:?}",
        report.syntax()
    );
    assert_eq!(
        report.syntax()[0]
            .known()
            .expect("retained color sibling")
            .property(),
        CssKnownProperty::Color,
    );
    assert_eq!(
        report.diagnostics().len(),
        1,
        "expected one diagnostic for `{value}`"
    );
}

fn parsed_filter_property(value: &str) -> CssFilterPropertyValue {
    let report = parse_style_attribute(&format!("filter: {value}"));
    assert!(
        report.is_clean(),
        "expected `{value}` to parse cleanly, got {:?}",
        report.diagnostics()
    );
    let declaration = report.syntax()[0]
        .known()
        .expect("known filter declaration");
    let CssKnownPropertyValueRef::Filter(value) =
        declaration.property_value().expect("ordinary filter value")
    else {
        panic!("expected filter property value");
    };
    value.clone()
}

fn assert_clip_path_rejected(value: &str) {
    let source = format!("clip-path: {value}; color: red");
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 1, "retained invalid `{value}`");
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color,
        "{value}",
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("`{value}` must produce one diagnostic");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
}

fn parsed_clip_path_property(value: &str) -> CssClipPathPropertyValue {
    let report = parse_style_attribute(&format!("clip-path: {value}"));
    assert!(report.is_clean(), "`{value}`: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::ClipPath(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected clip-path property value");
    };
    value.clone()
}

#[test]
fn clip_path_accepts_circle_percentage_radius() {
    let report = parse_style_attribute("clip-path: circle(25%); color: red");
    assert!(report.is_clean());
    assert_eq!(report.syntax().len(), 2);
    let CssKnownPropertyValueRef::ClipPath(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("circle clip path");
    };
    let CssClipPath::BasicShape(CssBasicShape::Circle(circle)) = value.value() else {
        panic!("typed circle");
    };
    assert!(
        matches!(circle.radius(), CssCircleRadius::LengthPercentage(radius)
        if exact_percentage(radius.literal_component(), "25"))
    );
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color
    );
}

#[test]
fn basic_shape_radius_arity_and_separator_mutations_are_rejected() {
    for value in [
        "circle(-1px)",
        "circle(10px at left left)",
        "ellipse(-1px 2px)",
        "ellipse(1px)",
        "ellipse(1px 2px 3px)",
        "inset(1px 2px 3px 4px 5px)",
        "inset(1px round / 2px)",
        "inset(1px round 2px /)",
        "inset(1px round 2px / 3px / 4px)",
        "polygon(, 0 0, 100% 0)",
        "polygon(evenodd 0 0, 100% 0)",
        "polygon(round -1px, 0 0)",
        "polygon(round 10%, 0 0)",
        "polygon(0 0, 100%)",
    ] {
        assert_clip_path_rejected(value);
    }
}

#[test]
fn circle_percentage_position_has_typed_radius_and_position() {
    let property = parsed_clip_path_property("circle(50% at center)");
    let CssClipPath::BasicShape(CssBasicShape::Circle(circle)) = property.value() else {
        panic!("typed circle");
    };
    assert!(
        matches!(circle.radius(), CssCircleRadius::LengthPercentage(radius)
        if exact_percentage(radius.literal_component(), "50"))
    );
    assert!(circle.position().is_some());
}

#[test]
fn every_radial_extent_keyword_has_a_typed_branch() {
    for (keyword, expected) in [
        ("closest-side", CssRadialExtent::ClosestSide),
        ("farthest-side", CssRadialExtent::FarthestSide),
        ("closest-corner", CssRadialExtent::ClosestCorner),
        ("farthest-corner", CssRadialExtent::FarthestCorner),
    ] {
        let circle = parsed_clip_path_property(&format!("circle({keyword})"));
        let CssClipPath::BasicShape(CssBasicShape::Circle(circle)) = circle.value() else {
            panic!("expected typed circle");
        };
        assert!(matches!(circle.radius(), CssCircleRadius::Extent(value) if *value == expected));

        let ellipse = parsed_clip_path_property(&format!("ellipse({keyword})"));
        let CssClipPath::BasicShape(CssBasicShape::Ellipse(ellipse)) = ellipse.value() else {
            panic!("expected typed ellipse");
        };
        assert!(matches!(ellipse.radius(), CssEllipseRadius::Extent(value) if *value == expected));
    }
}

#[test]
fn selected_basic_shapes_expose_typed_authored_components() {
    let circle = parsed_clip_path_property("circle(10px at right 5% bottom 2px)");
    let CssClipPath::BasicShape(CssBasicShape::Circle(circle)) = circle.value() else {
        panic!("expected typed circle");
    };
    assert!(matches!(
        circle.radius(),
        CssCircleRadius::LengthPercentage(value)
            if exact_dimension(value.literal_component(), "10", "px")
    ));
    let position = circle.position().unwrap();
    assert!(matches!(
        position.horizontal(),
        CssHorizontalPosition::RightOffset(_)
    ));
    assert!(matches!(
        position.vertical(),
        CssVerticalPosition::BottomOffset(_)
    ));

    let ellipse = parsed_clip_path_property("ellipse(10px 25% at center)");
    let CssClipPath::BasicShape(CssBasicShape::Ellipse(ellipse)) = ellipse.value() else {
        panic!("expected typed ellipse");
    };
    let CssEllipseRadius::Radii(radii) = ellipse.radius() else {
        panic!("expected explicit ellipse radii");
    };
    assert!(exact_dimension(
        radii.horizontal().literal_component(),
        "10",
        "px"
    ));
    assert!(exact_percentage(radii.vertical().literal_component(), "25"));

    let inset = parsed_clip_path_property("inset(1px 2% 3px round 4px 5% / 6px 7%)");
    let CssClipPath::BasicShape(CssBasicShape::Inset(inset)) = inset.value() else {
        panic!("expected typed inset");
    };
    assert_eq!(inset.offsets().values().len(), 3);
    let radii = inset.round().unwrap();
    assert!(exact_dimension(
        radii.top_left().horizontal().literal_component(),
        "4",
        "px"
    ));
    assert!(exact_dimension(
        radii.top_left().vertical().literal_component(),
        "6",
        "px"
    ));

    let polygon =
        parsed_clip_path_property("polygon(evenodd round 2px, 0 0, 100% 0, calc(50% - 1px) 100%)");
    let CssClipPath::BasicShape(CssBasicShape::Polygon(polygon)) = polygon.value() else {
        panic!("expected typed polygon");
    };
    assert_eq!(polygon.fill_rule(), Some(CssPolygonFillRule::Evenodd));
    assert!(polygon.round().is_some_and(|value| exact_dimension(
        value.literal_component(),
        "2",
        "px"
    )));
    assert_eq!(polygon.points().points().len(), 3);
}

#[test]
fn omitted_shape_branches_are_explicit() {
    let circle = parsed_clip_path_property("circle()");
    let CssClipPath::BasicShape(CssBasicShape::Circle(circle)) = circle.value() else {
        panic!("expected default circle");
    };
    assert!(matches!(circle.radius(), CssCircleRadius::Default));
    assert!(circle.position().is_none());

    let ellipse = parsed_clip_path_property("ellipse(at left top)");
    let CssClipPath::BasicShape(CssBasicShape::Ellipse(ellipse)) = ellipse.value() else {
        panic!("expected default ellipse");
    };
    assert!(matches!(ellipse.radius(), CssEllipseRadius::Default));
    assert!(ellipse.position().is_some());

    for (value, count) in [
        ("inset(1px)", 1),
        ("inset(1px 2px)", 2),
        ("inset(1px 2px 3px)", 3),
        ("inset(1px 2px 3px 4px)", 4),
    ] {
        let inset = parsed_clip_path_property(value);
        let CssClipPath::BasicShape(CssBasicShape::Inset(inset)) = inset.value() else {
            panic!("expected typed `{value}`");
        };
        assert_eq!(inset.offsets().values().len(), count);
    }

    let polygon = parsed_clip_path_property("polygon(0 0)");
    let CssClipPath::BasicShape(CssBasicShape::Polygon(polygon)) = polygon.value() else {
        panic!("expected prefix-free polygon");
    };
    assert_eq!(polygon.fill_rule(), None);
    assert!(polygon.round().is_none());
    assert_eq!(polygon.points().points().len(), 1);

    let polygon = parsed_clip_path_property("polygon(round 1px nonzero, -1px -2%)");
    let CssClipPath::BasicShape(CssBasicShape::Polygon(polygon)) = polygon.value() else {
        panic!("expected round-first polygon prefix");
    };
    assert_eq!(polygon.fill_rule(), Some(CssPolygonFillRule::Nonzero));
    assert!(polygon.round().is_some());
}

#[test]
fn shape_checked_scalars_reject_invalid_public_construction() {
    assert!(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_token("10%").unwrap()
        )
        .is_err()
    );
    assert!(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension("-1", "px").unwrap()
        )
        .is_err()
    );
    assert!(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            CssComponentValue::try_token("-1%").unwrap()
        )
        .is_err()
    );
}

#[test]
fn checked_shape_aggregates_retain_circle_percentage_and_omission() {
    use surgeist_css::{
        CssCircleShape, CssEllipseRadii, CssEllipseShape, CssInsetShape, CssInsetShapeOffsets,
        CssPolygonPoint, CssPolygonPointList, CssPolygonShape,
    };

    let radius = nonnegative_length_percentage("25%");
    let circle = CssCircleShape::new(CssCircleRadius::LengthPercentage(radius), None);
    assert!(
        matches!(circle.radius(), CssCircleRadius::LengthPercentage(radius)
        if exact_percentage(radius.literal_component(), "25"))
    );
    assert!(circle.position().is_none());
    let omitted = CssCircleShape::new(CssCircleRadius::Default, None);
    assert!(matches!(omitted.radius(), CssCircleRadius::Default));

    let ellipse = CssEllipseShape::new(
        CssEllipseRadius::Radii(CssEllipseRadii::new(
            nonnegative_length_percentage("10px"),
            nonnegative_length_percentage("20%"),
        )),
        None,
    );
    assert!(matches!(ellipse.radius(), CssEllipseRadius::Radii(radii)
        if exact_percentage(radii.vertical().literal_component(), "20")));

    let inset = CssInsetShape::new(
        CssInsetShapeOffsets::try_new(vec![signed_length_percentage("5%")]).unwrap(),
        None,
    );
    assert!(
        matches!(inset.offsets().values(), [value] if exact_percentage(value.literal_component(), "5"))
    );

    let point = CssPolygonPoint::new(
        signed_length_percentage("0px"),
        signed_length_percentage("100%"),
    );
    let polygon = CssPolygonShape::new(
        None,
        None,
        CssPolygonPointList::try_new(vec![point]).unwrap(),
    );
    assert_eq!(polygon.points().points().len(), 1);
    assert!(exact_percentage(
        polygon.points().points()[0].y().literal_component(),
        "100"
    ));
    assert!(CssPolygonPointList::try_new(Vec::new()).is_none());
    assert!(CssInsetShapeOffsets::try_new(Vec::new()).is_none());
}

#[test]
fn circle_symbolic_length_percentage_radius_remains_typed() {
    let property = parsed_clip_path_property("circle(calc(10px + 20%))");
    let CssClipPath::BasicShape(CssBasicShape::Circle(circle)) = property.value() else {
        panic!("typed circle");
    };
    assert!(
        matches!(circle.radius(), CssCircleRadius::LengthPercentage(radius)
        if radius.calculation().is_some())
    );
}

#[test]
fn deferred_basic_shape_functions_remain_unrecognized() {
    for value in [
        "path('M 0 0 L 1 1')",
        "shape(from 0 0, line to 1px 1px)",
        "rect(0 1px 1px 0)",
        "xywh(0 0 1px 1px)",
    ] {
        assert_clip_path_rejected(value);
    }
}

#[test]
fn basic_shape_calculations_preserve_the_exact_depth_boundary() {
    let source = format!(
        "clip-path: circle({}1px{}); color: red",
        "calc(".repeat(255),
        ")".repeat(255),
    );
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "depth 255: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);

    for depth in [256_usize, 257] {
        let source = format!(
            "clip-path: circle({}1px{}); color: red",
            "calc(".repeat(depth),
            ")".repeat(depth),
        );
        let first_over_limit = source.match_indices("calc(").nth(255).unwrap().0;
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "depth {depth}");
        let [diagnostic] = report.diagnostics() else {
            panic!("depth {depth}: expected one diagnostic");
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            first_over_limit,
            "depth {depth}",
        );
    }
}

#[test]
fn drop_shadow_rejects_box_shadow_only_components_and_negative_filter_amounts() {
    for value in [
        "drop-shadow(inset 1px 2px)",
        "drop-shadow(1px 2px 3px 4px)",
        "brightness(-0.01)",
        "grayscale(-1%)",
    ] {
        assert_filter_rejected(value);
    }
}

#[test]
fn filter_function_list_preserves_typed_authored_order() {
    let property = parsed_filter_property(concat!(
        "url(\"filters.svg#rough\") blur(4px) brightness() contrast(25%) ",
        "drop-shadow(red -1px 2px calc(1px + 2px)) grayscale(.5) ",
        "hue-rotate(calc(1turn - 90deg)) invert(10%) opacity(.75) saturate(2) sepia(30%)"
    ));
    let CssFilter::Functions(functions) = property.value() else {
        panic!("expected current filter function list");
    };
    assert!(matches!(
        functions.functions(),
        [
            CssFilterFunction::Url(_),
            CssFilterFunction::Blur(_),
            CssFilterFunction::Brightness(_),
            CssFilterFunction::Contrast(_),
            CssFilterFunction::DropShadow(_),
            CssFilterFunction::Grayscale(_),
            CssFilterFunction::HueRotate(_),
            CssFilterFunction::Invert(_),
            CssFilterFunction::Opacity(_),
            CssFilterFunction::Saturate(_),
            CssFilterFunction::Sepia(_),
        ]
    ));
}

#[test]
fn every_filter_amount_function_has_exact_typed_domain() {
    let property = parsed_filter_property(concat!(
        "brightness() contrast(2) grayscale(25%) invert(calc(1 - .25)) ",
        "opacity(calc(50%)) saturate(3) sepia(75%)"
    ));
    let CssFilter::Functions(functions) = property.value() else {
        panic!("expected current filter function list");
    };
    assert!(matches!(
        functions.functions()[0],
        CssFilterFunction::Brightness(CssFilterAmount::Default)
    ));
    assert!(matches!(
        &functions.functions()[1],
        CssFilterFunction::Contrast(CssFilterAmount::Number(value)) if exact_number(value.literal_component(), "2")
    ));
    assert!(matches!(
        &functions.functions()[2],
        CssFilterFunction::Grayscale(CssFilterAmount::Percentage(value)) if exact_percentage(value.literal_component(), "25")
    ));
    assert!(matches!(
        &functions.functions()[3],
        CssFilterFunction::Invert(CssFilterAmount::Number(value)) if value.calculation().is_some()
    ));
    assert!(matches!(
        &functions.functions()[4],
        CssFilterFunction::Opacity(CssFilterAmount::Percentage(value)) if value.calculation().is_some()
    ));

    for value in [
        "brightness(-1)",
        "contrast(-0.1%)",
        "opacity(1 2)",
        "saturate(1, 2)",
        "sepia(auto)",
    ] {
        assert_filter_rejected(value);
    }
}

#[test]
fn blur_hue_rotate_and_drop_shadow_expose_distinct_typed_payloads() {
    let property = parsed_filter_property(
        "blur(calc(1px + 2em)) hue-rotate(-.25turn) drop-shadow(1px -2px blue)",
    );
    let CssFilter::Functions(functions) = property.value() else {
        panic!("expected current filter function list");
    };
    assert!(matches!(
        &functions.functions()[0],
        CssFilterFunction::Blur(blur) if blur.length().calculation().is_some()
    ));
    assert!(matches!(
        functions.functions()[1],
        CssFilterFunction::HueRotate(CssAngleValue::Literal(value))
            if value.value() == -0.25
    ));
    let CssFilterFunction::DropShadow(shadow) = &functions.functions()[2] else {
        panic!("expected typed drop-shadow");
    };
    assert!(exact_dimension(
        shadow.offset_x().literal_component(),
        "1",
        "px"
    ));
    assert!(exact_dimension(
        shadow.offset_y().literal_component(),
        "-2",
        "px"
    ));
    assert!(shadow.blur_radius().is_none());
    assert!(shadow.color().is_some());
}

#[test]
fn box_shadow_accepts_component_orders_and_rejects_invalid_components() {
    let report = parse_style_attribute(concat!(
        "box-shadow: red inset -1px 2px 3px -4px, ",
        "5px 6px blue inset, inset 7px 8px; color: red"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BoxShadow(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected box-shadow");
    };
    let CssBoxShadow::Shadows(shadows) = value.value() else {
        panic!("expected shadow list");
    };
    assert_eq!(shadows.shadows().len(), 3);
    assert!(shadows.shadows()[0].inset());
    assert!(
        shadows.shadows()[0]
            .spread_radius()
            .is_some_and(|value| exact_dimension(value.literal_component(), "-4", "px"))
    );
    assert!(shadows.shadows()[1].color().is_some());

    for value in [
        "1px 2px -3px",
        "1px 2px red blue",
        "inset inset 1px 2px",
        "1px 2px,",
    ] {
        let report = parse_style_attribute(&format!("box-shadow: {value}; color: red"));
        assert_eq!(report.syntax().len(), 1, "retained `{value}`");
        assert_eq!(report.diagnostics().len(), 1, "{value}");
    }
}

// Backgrounds 3 §6.1 and Filter Effects 1 §6.1 require a contiguous
// length group; unordered color placement does not split that group.
#[test]
fn box_shadow_rejects_interleaved_color_between_offsets() {
    let source = "box-shadow: 1px red 2px; color: blue";
    let report = parse_style_attribute(source);
    assert!(!report.is_clean(), "interrupted length group was accepted");
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(report.diagnostics().len(), 1);
    assert!(validate_style_attribute(source).is_err());
}

#[test]
fn drop_shadow_rejects_interleaved_color_between_offsets() {
    let source = "filter: drop-shadow(1px red 2px); color: blue";
    let report = parse_style_attribute(source);
    assert!(!report.is_clean(), "interrupted length group was accepted");
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(report.diagnostics().len(), 1);
    assert!(validate_style_attribute(source).is_err());
}

#[test]
fn drop_shadow_retains_an_authored_color_mix_without_a_lossy_filter_projection() {
    let property = parsed_filter_property(concat!(
        "drop-shadow(1px 2px ",
        "color-mix(in srgb, lab(calc(50% + 10%) 20 30), blue))",
    ));
    let CssFilter::Functions(functions) = property.value() else {
        panic!("expected current filter function list");
    };
    let [CssFilterFunction::DropShadow(shadow)] = functions.functions() else {
        panic!("expected one typed drop-shadow");
    };
    assert_eq!(
        shadow
            .color()
            .unwrap()
            .color_mix_value()
            .unwrap()
            .components()
            .len(),
        2
    );
}

#[test]
fn filter_lists_reject_empty_unknown_repeated_and_trailing_mutations() {
    for value in [
        "none blur(1px)",
        "blur(,)",
        "hue-rotate()",
        "hue-rotate(1deg, 2deg)",
        "drop-shadow()",
        "drop-shadow(red red 1px 2px)",
        "unknown(1)",
        "blur(1px), opacity(1)",
        "blur(1px) trailing",
    ] {
        assert_filter_rejected(value);
    }
}

#[test]
fn filter_checked_scalars_and_lists_reject_unrepresentable_states() {
    assert!(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension("-1", "px").unwrap()
        )
        .is_err()
    );
    assert!(
        CssSpecifiedLength::try_from_component(CssComponentValue::try_token("1%").unwrap())
            .is_err()
    );
    assert!(CssFilterFunctionList::try_new(Vec::new()).is_none());
}

#[test]
fn filter_calculations_preserve_the_exact_depth_boundary() {
    let source = format!(
        "filter: brightness({}1{}); color: red",
        "calc(".repeat(255),
        ")".repeat(255),
    );
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "depth 255: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);

    for depth in [256_usize, 257] {
        let source = format!(
            "filter: brightness({}1{}); color: red",
            "calc(".repeat(depth),
            ")".repeat(depth),
        );
        let first_over_limit = source
            .match_indices("calc(")
            .nth(255)
            .expect("256th authored nested calculation")
            .0;
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "depth {depth}");
        let [diagnostic] = report.diagnostics() else {
            panic!("depth {depth}: expected one diagnostic");
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            first_over_limit,
            "depth {depth}"
        );
    }
}

#[test]
fn filter_ordinary_global_and_substitution_values_remain_distinct() {
    let report = parse_style_attribute(concat!(
        "filter: blur(1px); backdrop-filter: inherit; filter: var(--filters); ",
        "box-shadow: initial; box-shadow: var(--shadow)"
    ));
    assert!(report.is_clean());
    assert!(matches!(
        report.syntax()[0].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Property(CssKnownPropertyValueRef::Filter(_))
    ));
    assert!(matches!(
        report.syntax()[1].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    ));
    assert!(matches!(
        report.syntax()[2].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
    assert!(matches!(
        report.syntax()[3].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(_)
    ));
    assert!(matches!(
        report.syntax()[4].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(_)
    ));
}

fn parsed_easing_property(value: &str) -> CssTransitionTimingFunctionPropertyValue {
    let report = parse_style_attribute(&format!("transition-timing-function: {value}"));
    assert!(
        report.is_clean(),
        "expected `{value}` to parse cleanly, got {:?}",
        report.diagnostics()
    );
    let declaration = report.syntax()[0]
        .known()
        .expect("known transition-timing-function declaration");
    let CssKnownPropertyValueRef::TransitionTimingFunction(value) = declaration
        .property_value()
        .expect("ordinary transition-timing-function value")
    else {
        panic!("expected transition-timing-function property value");
    };
    value.clone()
}

#[test]
fn easing_functions_reject_out_of_range_x_and_invalid_jump_none_count() {
    for value in [
        "cubic-bezier(-0.01, 0, 0.5, 1)",
        "cubic-bezier(0.5, 0, 1.01, 1)",
        "steps(1, jump-none)",
    ] {
        assert_easing_rejected(value);
    }
}

#[test]
fn every_easing_keyword_and_alias_is_a_distinct_branch() {
    let property = parsed_easing_property(
        "ease, linear, ease-in, ease-out, ease-in-out, step-start, step-end",
    );
    let values = property.timing_functions().values();
    assert!(matches!(
        values,
        [
            CssEasing::Keyword(CssEasingKeyword::Ease),
            CssEasing::Keyword(CssEasingKeyword::Linear),
            CssEasing::Keyword(CssEasingKeyword::EaseIn),
            CssEasing::Keyword(CssEasingKeyword::EaseOut),
            CssEasing::Keyword(CssEasingKeyword::EaseInOut),
            CssEasing::Keyword(CssEasingKeyword::StepStart),
            CssEasing::Keyword(CssEasingKeyword::StepEnd),
        ]
    ));
}

#[test]
fn cubic_bezier_coordinates_are_typed_and_keep_symbolic_number_math() {
    let property = parsed_easing_property(concat!(
        "cubic-bezier(0, -20, 1, 30), ",
        "cubic-bezier(calc(0 + .25), calc(-1 - 2), calc(1 - .25), calc(2 * 3))"
    ));
    let [
        CssEasing::CubicBezier(literal),
        CssEasing::CubicBezier(symbolic),
    ] = property.timing_functions().values()
    else {
        panic!("expected two typed cubic-bezier values");
    };
    assert!(exact_number(
        (literal.x1().value()).literal_component(),
        "0"
    ));
    assert!(exact_number((literal.y1()).literal_component(), "-20"));
    assert!(exact_number(
        (literal.x2().value()).literal_component(),
        "1"
    ));
    assert!(exact_number((literal.y2()).literal_component(), "30"));
    assert!((symbolic.x1().value()).calculation().is_some());
    assert!((symbolic.y1()).calculation().is_some());
    assert!((symbolic.x2().value()).calculation().is_some());
    assert!((symbolic.y2()).calculation().is_some());

    let finite = checked_number("0.5");
    assert!(
        CssCubicBezier::try_new(finite.clone(), finite.clone(), finite.clone(), finite).is_some()
    );
    let out_of_range = checked_number("1.01");
    let zero = checked_number("0");
    assert!(CssCubicBezier::try_new(out_of_range, zero.clone(), zero.clone(), zero).is_none());
}

#[test]
fn every_steps_position_is_typed_and_jump_none_keeps_its_count_rule() {
    let property = parsed_easing_property(concat!(
        "steps(1), steps(1, jump-start), steps(1, jump-end), ",
        "steps(2, jump-none), steps(1, jump-both), steps(1, start), steps(1, end), ",
        "steps(calc(1 + 1), jump-none)"
    ));
    let values = property.timing_functions().values();
    let expected_positions = [
        None,
        Some(CssStepPosition::JumpStart),
        Some(CssStepPosition::JumpEnd),
        Some(CssStepPosition::JumpNone),
        Some(CssStepPosition::JumpBoth),
        Some(CssStepPosition::Start),
        Some(CssStepPosition::End),
    ];
    for (value, expected_position) in values[..7].iter().zip(expected_positions) {
        let CssEasing::Steps(steps) = value else {
            panic!("expected typed steps value");
        };
        assert_eq!(steps.position(), expected_position);
    }
    let CssEasing::Steps(symbolic) = &values[7] else {
        panic!("expected symbolic typed steps value");
    };
    assert!(matches!(
        symbolic.count(),
        CssPositiveIntegerValue::Calculation(_)
    ));

    let one = CssPositiveIntegerValue::Literal(
        CssPositiveIntegerLiteral::try_new(CssIntegerLiteral::from_i32(1))
            .expect("positive step count"),
    );
    assert!(CssSteps::try_new(one, Some(CssStepPosition::JumpNone)).is_none());
    assert!(CssPositiveIntegerLiteral::try_new(CssIntegerLiteral::from_i32(0)).is_none());
    assert!(CssPositiveIntegerLiteral::try_new(CssIntegerLiteral::from_i32(-1)).is_none());
    assert!(CssEasingList::try_new(Vec::new()).is_none());
}

#[test]
fn easing_functions_require_exact_separators_arities_and_domains() {
    for value in [
        "cubic-bezier(0 0 1 1)",
        "cubic-bezier(0, 0, 1)",
        "cubic-bezier(0, 0, 1, 1, 2)",
        "cubic-bezier(0%, 0, 1, 1)",
        "cubic-bezier(0, NaN, 1, 1)",
        "steps(0)",
        "steps(-1)",
        "steps(1.5)",
        "steps(1 start)",
        "steps(1, middle)",
        "steps(1, start, end)",
        "steps()",
        "steps(1),",
    ] {
        assert_easing_rejected(value);
    }
    let property = parsed_easing_property("cubic-bezier(0, 1e999, 1, 1)");
    let [CssEasing::CubicBezier(value)] = property.timing_functions().values() else {
        panic!("exact unrestricted Y coordinate")
    };
    assert!(exact_number(value.y1().literal_component(), "1e999"));
}

#[test]
fn repeated_easing_failures_recover_to_valid_timing_and_color_siblings() {
    let source = concat!(
        "transition-timing-function: cubic-bezier(-0.1, 0, 1, 1); ",
        "animation-timing-function: steps(1, jump-none); ",
        "transition-timing-function: steps(2, jump-none); color: red"
    );
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(report.diagnostics().len(), 2);
    assert_eq!(
        report.syntax()[0]
            .known()
            .expect("valid timing sibling")
            .property(),
        CssKnownProperty::TransitionTimingFunction,
    );
    assert_eq!(
        report.syntax()[1]
            .known()
            .expect("valid color sibling")
            .property(),
        CssKnownProperty::Color,
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
    );

    {
        let failure = surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects both recovered easing declarations");
        assert_eq!(failure.diagnostics(), report.diagnostics());
    }
}

#[test]
fn easing_symbolic_math_preserves_the_exact_depth_boundary() {
    let source = format!(
        "transition-timing-function: cubic-bezier(0, {}1{}, 1, 1); color: red",
        "calc(".repeat(255),
        ")".repeat(255),
    );
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "depth 255: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);

    for depth in [256_usize, 257] {
        let source = format!(
            "transition-timing-function: cubic-bezier(0, {}1{}, 1, 1); color: red",
            "calc(".repeat(depth),
            ")".repeat(depth),
        );
        let first_over_limit = source
            .match_indices("calc(")
            .nth(255)
            .expect("256th authored nested calculation")
            .0;
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "depth {depth}");
        let [diagnostic] = report.diagnostics() else {
            panic!("depth {depth}: over-limit easing must produce one diagnostic");
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            first_over_limit,
            "depth {depth}",
        );

        {
            let failure = surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects over-limit easing calculations");
            assert_eq!(failure.diagnostics(), report.diagnostics());
        }
    }
}

fn assert_function_sequence(value: &str, expected: &[CssTransformFunctionKind]) {
    let property = parsed_transform_property(value);
    let CssTransform::Functions(functions) = property.value() else {
        panic!("expected transform function list");
    };
    let actual = functions
        .functions()
        .iter()
        .map(CssTransformFunction::kind)
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn every_selected_two_dimensional_transform_function_preserves_authored_order() {
    assert_function_sequence(
        concat!(
            "matrix(1, 0, 0, 1, 10, 20) ",
            "translate(1px, 2%) translateX(calc(1px + 2%)) translateY(3em) ",
            "scale(1.5, calc(1 + 0.5)) scaleX(calc(1 + 0.5)) scaleY(.75) ",
            "rotate(calc(1turn - 90deg)) skew(10deg, 0) skewX(.25turn) skewY(0)"
        ),
        &[
            CssTransformFunctionKind::Matrix,
            CssTransformFunctionKind::Translate,
            CssTransformFunctionKind::TranslateX,
            CssTransformFunctionKind::TranslateY,
            CssTransformFunctionKind::Scale,
            CssTransformFunctionKind::ScaleX,
            CssTransformFunctionKind::ScaleY,
            CssTransformFunctionKind::Rotate,
            CssTransformFunctionKind::Skew,
            CssTransformFunctionKind::SkewX,
            CssTransformFunctionKind::SkewY,
        ],
    );
    let property = parsed_transform_property(
        "matrix(1, 0, 0, 1, 10, 20) translate(1px, 2%) scale(1.5, calc(1 + 0.5)) rotate(calc(1turn - 90deg))",
    );
    let CssTransform::Functions(functions) = property.value() else {
        panic!("expected ordered typed transform functions");
    };
    assert!(matches!(
        &functions.functions()[0],
        CssTransformFunction::Matrix(matrix)
            if exact_number((matrix.components()[4]).literal_component(), "10")
    ));
    assert!(matches!(
        &functions.functions()[1],
        CssTransformFunction::Translate(translation)
            if exact_dimension(translation.x().literal_component(), "1", "px")
                && exact_percentage(translation.y().unwrap().literal_component(), "2")
    ));
    assert!(matches!(
        &functions.functions()[2],
        CssTransformFunction::Scale(scale)
            if scale.y().is_some_and(|value| value.calculation().is_some())
    ));
    assert!(matches!(
        functions.functions()[3],
        CssTransformFunction::Rotate(CssAngleValue::Calculation(_))
    ));
}

#[test]
fn transform_matrix3d_exposes_sixteen_finite_components() {
    assert_function_sequence(
        "matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)",
        &[CssTransformFunctionKind::Matrix3d],
    );

    let property =
        parsed_transform_property("matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30, 1)");
    let CssTransform::Functions(functions) = property.value() else {
        panic!("expected transform function list");
    };
    let CssTransformFunction::Matrix3d(matrix) = &functions.functions()[0] else {
        panic!("expected typed matrix3d");
    };
    assert!(exact_number(
        (matrix.components()[12]).literal_component(),
        "10"
    ));
    assert!(exact_number(
        (matrix.components()[15]).literal_component(),
        "1"
    ));
}

#[test]
fn transform_perspective_accepts_none_and_zero_and_rejects_invalid_dimensions() {
    assert_function_sequence(
        "perspective(none) perspective(0) perspective(12px)",
        &[
            CssTransformFunctionKind::Perspective,
            CssTransformFunctionKind::Perspective,
            CssTransformFunctionKind::Perspective,
        ],
    );

    for value in [
        "perspective(-1px)",
        "perspective(10%)",
        "perspective(auto)",
        "perspective()",
        "perspective(1px, 2px)",
    ] {
        assert_transform_rejected(value);
    }

    let property = parsed_transform_property("perspective(none) perspective(0)");
    let CssTransform::Functions(functions) = property.value() else {
        panic!("expected transform function list");
    };
    assert!(matches!(
        functions.functions()[0],
        CssTransformFunction::Perspective(CssTransformPerspective::None)
    ));
    assert!(matches!(
        &functions.functions()[1],
        CssTransformFunction::Perspective(CssTransformPerspective::Length(length))
            if matches!(length.literal_component().map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Number(number))) if number.representation() == "0")
    ));
}

#[test]
fn transform_three_dimensional_rotations_are_typed() {
    assert_function_sequence(
        "rotate3d(1, 0, -1, 45deg) rotateX(10deg) rotateY(0) rotateZ(calc(1turn / 2))",
        &[
            CssTransformFunctionKind::Rotate3d,
            CssTransformFunctionKind::RotateX,
            CssTransformFunctionKind::RotateY,
            CssTransformFunctionKind::RotateZ,
        ],
    );

    let property = parsed_transform_property("rotate3d(1, 0, -1, 45deg) rotateZ(calc(1turn / 2))");
    let CssTransform::Functions(functions) = property.value() else {
        panic!("expected transform function list");
    };
    let CssTransformFunction::Rotate3d(rotation) = &functions.functions()[0] else {
        panic!("expected typed rotate3d");
    };
    assert!(exact_number((rotation.z()).literal_component(), "-1"));
    assert!(matches!(rotation.angle(), CssAngleValue::Literal(value) if value.value() == 45.0));
    assert!(matches!(
        functions.functions()[1],
        CssTransformFunction::RotateZ(CssAngleValue::Calculation(_))
    ));
}

#[test]
fn transform_angles_reject_percentage_calculations_and_recover_siblings() {
    let mut wrongly_retained = Vec::new();
    for value in [
        "rotate(calc(10%))",
        "skew(calc(10%))",
        "rotate3d(1, 0, -1, calc(1deg + 10%))",
    ] {
        let source =
            format!("transform: {value}; transform: rotate(calc(1turn - 90deg)); color: red");
        let report = parse_style_attribute(&source);
        if report.syntax().len() != 2 || report.diagnostics().len() != 1 {
            wrongly_retained.push(value);
            continue;
        }
        let [diagnostic] = report.diagnostics() else {
            panic!("expected one diagnostic for `{value}`");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            report.syntax()[0]
                .known()
                .expect("retained valid symbolic-angle transform")
                .property(),
            CssKnownProperty::Transform,
        );
        assert_eq!(
            report.syntax()[1]
                .known()
                .expect("retained color sibling")
                .property(),
            CssKnownProperty::Color,
        );

        {
            let failure = surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects percentage-typed transform angles");
            assert_eq!(failure.diagnostics(), report.diagnostics());
        }
    }
    assert!(
        wrongly_retained.is_empty(),
        "retained invalid transform angles: {wrongly_retained:?}"
    );

    let property = parsed_transform_property("rotate(calc(1turn - 90deg))");
    let CssTransform::Functions(functions) = property.value() else {
        panic!("expected transform function list");
    };
    assert!(matches!(
        functions.functions()[0],
        CssTransformFunction::Rotate(CssAngleValue::Calculation(_))
    ));
}

#[test]
fn transform_three_dimensional_scales_preserve_number_and_percentage_operands() {
    assert_function_sequence(
        "scale3d(1, 50%, calc(1 + .5)) scaleZ(2) scaleZ(125%)",
        &[
            CssTransformFunctionKind::Scale3d,
            CssTransformFunctionKind::ScaleZ,
            CssTransformFunctionKind::ScaleZ,
        ],
    );

    let property =
        parsed_transform_property("scale3d(1, 50%, calc(1 + .5)) scaleZ(2) scaleZ(125%)");
    let CssTransform::Functions(functions) = property.value() else {
        panic!("expected transform function list");
    };
    let CssTransformFunction::Scale3d(scale) = &functions.functions()[0] else {
        panic!("expected typed scale3d");
    };
    assert!(matches!(
        scale.x(),
        CssTransformScaleComponent::Number(value) if exact_number(value.literal_component(), "1")
    ));
    assert!(matches!(
        scale.y(),
        CssTransformScaleComponent::Percentage(value) if exact_percentage(value.literal_component(), "50")
    ));
    assert!(matches!(
        scale.z(),
        CssTransformScaleComponent::Number(value) if value.calculation().is_some()
    ));
    assert!(matches!(
        &functions.functions()[1],
        CssTransformFunction::ScaleZ(CssTransformScaleComponent::Number(value)) if exact_number(value.literal_component(), "2")
    ));
    assert!(matches!(
        &functions.functions()[2],
        CssTransformFunction::ScaleZ(CssTransformScaleComponent::Percentage(value)) if exact_percentage(value.literal_component(), "125")
    ));
}

#[test]
fn transform_three_dimensional_translations_keep_z_length_only() {
    assert_function_sequence(
        "translate3d(10%, calc(2px + 3%), 4em) translateZ(calc(1px + 2em))",
        &[
            CssTransformFunctionKind::Translate3d,
            CssTransformFunctionKind::TranslateZ,
        ],
    );

    for value in ["translate3d(1px, 2px, 3%)", "translateZ(10%)"] {
        assert_transform_rejected(value);
    }

    let property = parsed_transform_property("translate3d(10%, calc(2px + 3%), 4em)");
    let CssTransform::Functions(functions) = property.value() else {
        panic!("expected transform function list");
    };
    let CssTransformFunction::Translate3d(translation) = &functions.functions()[0] else {
        panic!("expected typed translate3d");
    };
    assert!(exact_percentage(translation.x().literal_component(), "10"));
    assert!(translation.y().calculation().is_some());
    assert!(exact_dimension(
        translation.z().literal_component(),
        "4",
        "em"
    ));
}

#[test]
fn transform_functions_require_exact_commas_and_arities() {
    for value in [
        "matrix(1 0 0 1 10 20)",
        "matrix(1, 0, 0, 1, 10)",
        "matrix(1, 0, 0, 1, 10, 20, 30)",
        "translate(1px 2px)",
        "translateX(1px, 2px)",
        "scale(1 2)",
        "scale(1, 50%)",
        "scaleX()",
        "scaleY(50%)",
        "skew(10deg 20deg)",
        "rotate(10deg, 20deg)",
        "matrix3d(1 0 0 0 0 1 0 0 0 0 1 0 10 20 30 1)",
        "rotate3d(1 0 0 45deg)",
        "rotate3d(1, 0, 45deg)",
        "scale3d(1 2 3)",
        "scale3d(1, 2)",
        "translate3d(1px 2px 3px)",
        "translate3d(1px, 2px)",
    ] {
        assert_transform_rejected(value);
    }
}

#[test]
fn transform_function_lists_reject_empty_unknown_and_trailing_mutations() {
    for value in [
        "matrix()",
        "unknown(1)",
        "translateX(1px) trailing",
        "translateX(1px), rotate(1deg)",
    ] {
        assert_transform_rejected(value);
    }

    assert_function_sequence(
        "translateX(1px) rotate(2deg) scaleY(3)",
        &[
            CssTransformFunctionKind::TranslateX,
            CssTransformFunctionKind::Rotate,
            CssTransformFunctionKind::ScaleY,
        ],
    );
}

#[test]
fn transform_checked_scalars_and_lists_reject_unrepresentable_states() {
    assert!(
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_token("auto").unwrap()
        )
        .is_err()
    );
    assert!(
        CssSpecifiedLength::try_from_component(CssComponentValue::try_token("10%").unwrap())
            .is_err()
    );
    assert!(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension("-1", "px").unwrap()
        )
        .is_err()
    );
    assert!(CssTransformFunctionList::try_new(Vec::new()).is_none());
}

#[test]
fn transform_calculations_preserve_the_exact_depth_boundary() {
    let source = format!(
        "transform: rotate({}1deg{}); color: red",
        "calc(".repeat(255),
        ")".repeat(255),
    );
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "depth 255: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);

    for depth in [256_usize, 257] {
        let source = format!(
            "transform: rotate({}1deg{}); color: red",
            "calc(".repeat(depth),
            ")".repeat(depth),
        );
        let first_over_limit = source
            .match_indices("calc(")
            .nth(255)
            .expect("256th authored nested calculation")
            .0;
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "depth {depth}");
        let [diagnostic] = report.diagnostics() else {
            panic!("depth {depth}: over-limit transform must produce one diagnostic");
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            first_over_limit,
            "depth {depth}",
        );

        {
            let failure = surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects over-limit transform calculations");
            assert_eq!(failure.diagnostics(), report.diagnostics());
        }
    }
}

fn exact_dimension(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit })) if number.representation() == representation && unit == expected_unit)
}
fn exact_percentage(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Percentage(number))) if number.representation() == representation)
}

fn signed_length_percentage(css: &str) -> surgeist_css::CssSpecifiedLengthPercentage {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedLengthPercentage::try_from_calculation(
            surgeist_css::CssLengthPercentageCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedLengthPercentage::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}
fn nonnegative_length_percentage(
    css: &str,
) -> surgeist_css::CssSpecifiedNonNegativeLengthPercentage {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
            surgeist_css::CssLengthPercentageCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}

fn exact_number(component: Option<&surgeist_css::CssComponentValue>, representation: &str) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Number(number))) if number.representation() == representation)
}
fn checked_number(representation: &str) -> surgeist_css::CssSpecifiedNumber {
    surgeist_css::CssSpecifiedNumber::try_from_component(
        surgeist_css::CssComponentValue::try_number(representation).unwrap(),
    )
    .unwrap()
}
