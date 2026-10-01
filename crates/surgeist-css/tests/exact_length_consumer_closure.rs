#![forbid(unsafe_code)]

//! The selected property grammars retain exact scalar owners and authored shape.
//! Ordinary signedness and dimensional requirements follow the pinned Values 4
//! numeric definitions and the existing CSS2, Images 3, Transforms 2, Filter
//! Effects 1 and Shapes property contracts in `specs/catalog.json`.
//! Symbolic math is admitted by its root domain, before contextual range checks.

use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained declaration: {source}");
    };
    declaration.clone()
}

macro_rules! value {
    ($source:expr, $variant:ident, $accessor:ident) => {{
        let declaration = declaration($source);
        let Some(CssKnownPropertyValueRef::$variant(wrapper)) =
            declaration.known().unwrap().property_value()
        else {
            panic!("expected checked property wrapper");
        };
        wrapper.$accessor().clone()
    }};
}

fn literal(component: &CssComponentValue, spelling: &str, unit: Option<&str>) {
    match (component.view(), unit) {
        (CssComponentValueRef::Token(CssValueTokenRef::Number(number)), None) => {
            assert_eq!(number.representation(), spelling);
        }
        (CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)), Some("%")) => {
            assert_eq!(number.representation(), spelling);
        }
        (
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }),
            Some(expected_unit),
        ) => {
            assert_eq!(number.representation(), spelling);
            assert_eq!(unit, expected_unit);
        }
        _ => panic!("expected exact ordinary numeric kind"),
    }
}

fn parsed_origin(origin: &CssValueOrigin, source: &str, token: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("expected original parsed scalar provenance");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        &source[origin.span().start().byte_offset().value()
            ..origin.span().end().byte_offset().value()],
        token,
    );
}

fn length(token: &str) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_component(CssComponentValue::try_token(token).unwrap()).unwrap()
}

fn nonnegative_length(token: &str) -> CssSpecifiedNonNegativeLength {
    CssSpecifiedNonNegativeLength::try_from_component(CssComponentValue::try_token(token).unwrap())
        .unwrap()
}

fn length_percentage(token: &str) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_component(CssComponentValue::try_token(token).unwrap())
        .unwrap()
}

fn nonnegative_length_percentage(token: &str) -> CssSpecifiedNonNegativeLengthPercentage {
    CssSpecifiedNonNegativeLengthPercentage::try_from_component(
        CssComponentValue::try_token(token).unwrap(),
    )
    .unwrap()
}

#[test]
fn signed_pure_length_properties_retain_extreme_literals_and_original_spans() {
    let source = "outline-offset:-1e999px";
    let offset: CssSpecifiedLength = value!(source, OutlineOffset, offset);
    literal(offset.literal_component().unwrap(), "-1e999", Some("px"));
    parsed_origin(offset.origin(), source, "-1e999px");

    let source = "clip:rect(-1e-999px, auto, 1e999em, -0.000e999px)";
    let CssClip::Rect(rect) = value!(source, Clip, clip) else {
        panic!("expected checked rectangle");
    };
    let CssClipEdge::Length(top) = rect.top() else {
        panic!("signed top")
    };
    literal(top.literal_component().unwrap(), "-1e-999", Some("px"));
    parsed_origin(top.origin(), source, "-1e-999px");
    assert_eq!(rect.right(), &CssClipEdge::Auto);
    let CssClipEdge::Length(bottom) = rect.bottom() else {
        panic!("signed bottom")
    };
    literal(bottom.literal_component().unwrap(), "1e999", Some("em"));
    let CssClipEdge::Length(left) = rect.left() else {
        panic!("signed left")
    };
    literal(left.literal_component().unwrap(), "-0.000e999", Some("px"));
}

#[test]
fn shadow_offsets_spread_and_blur_keep_distinct_checked_domains() {
    let source = "box-shadow:inset -1e-999px 1e999em 0px -2px red";
    let CssBoxShadow::Shadows(shadows) = value!(source, BoxShadow, value) else {
        panic!("shadow list");
    };
    let [shadow] = shadows.shadows() else {
        panic!("one shadow")
    };
    assert!(shadow.inset());
    literal(
        shadow.offset_x().literal_component().unwrap(),
        "-1e-999",
        Some("px"),
    );
    literal(
        shadow.offset_y().literal_component().unwrap(),
        "1e999",
        Some("em"),
    );
    literal(
        shadow.blur_radius().unwrap().literal_component().unwrap(),
        "0",
        Some("px"),
    );
    literal(
        shadow.spread_radius().unwrap().literal_component().unwrap(),
        "-2",
        Some("px"),
    );
    parsed_origin(shadow.offset_x().origin(), source, "-1e-999px");
    assert_eq!(shadow.color().unwrap().named().unwrap().name(), "red");

    assert!(
        CssShadow::try_new(
            false,
            length("1px"),
            length("2px"),
            None,
            Some(length("3px")),
            None
        )
        .is_none()
    );
    let bare = CssShadow::try_new(false, length("1px"), length("2px"), None, None, None).unwrap();
    assert!(bare.blur_radius().is_none());
    assert!(bare.spread_radius().is_none());
    assert!(bare.color().is_none());
}

#[test]
fn filter_blur_retains_authored_omission_and_effective_zero_pixels() {
    let CssFilter::Functions(functions) = value!(
        "filter:blur() blur(-0px) drop-shadow(-1e-999px 2px)",
        Filter,
        value
    ) else {
        panic!("filter functions");
    };
    let [
        CssFilterFunction::Blur(default),
        CssFilterFunction::Blur(explicit),
        CssFilterFunction::DropShadow(shadow),
    ] = functions.functions()
    else {
        panic!("ordered filter functions")
    };
    assert!(default.authored_length().is_none());
    literal(
        default.length().literal_component().unwrap(),
        "0",
        Some("px"),
    );
    assert_eq!(default.length().origin(), &CssValueOrigin::Programmatic);
    literal(
        explicit
            .authored_length()
            .unwrap()
            .literal_component()
            .unwrap(),
        "-0",
        Some("px"),
    );
    assert!(matches!(
        explicit.length().origin(),
        CssValueOrigin::Parsed(_)
    ));
    literal(
        shadow.offset_x().literal_component().unwrap(),
        "-1e-999",
        Some("px"),
    );
    assert!(shadow.standard_deviation().is_none());
    assert!(shadow.color().is_none());
}

#[test]
fn positions_and_background_sizes_use_their_signed_and_nonnegative_domains() {
    let source = "object-position:right -1e999px bottom -1e-999%";
    let position = value!(source, ObjectPosition, position);
    let CssHorizontalPosition::RightOffset(x) = position.horizontal() else {
        panic!("right offset")
    };
    let CssVerticalPosition::BottomOffset(y) = position.vertical() else {
        panic!("bottom offset")
    };
    literal(x.literal_component().unwrap(), "-1e999", Some("px"));
    literal(y.literal_component().unwrap(), "-1e-999", Some("%"));
    parsed_origin(y.origin(), source, "-1e-999%");

    let sizes = value!("background-size:1e999px auto", BackgroundSize, sizes);
    let [
        CssBackgroundSize::Explicit {
            width: CssBackgroundSizeComponent::Length(width),
            height: Some(CssBackgroundSizeComponent::Auto),
        },
    ] = sizes.sizes()
    else {
        panic!("ordered size pair")
    };
    literal(width.literal_component().unwrap(), "1e999", Some("px"));
}

#[test]
fn gradient_stops_and_radial_sizes_retain_exact_kind_and_order() {
    let images = value!(
        "background-image:linear-gradient(red -1e-999px, blue 1e999%), radial-gradient(ellipse 1e999px 25% at center, red, blue)",
        BackgroundImage,
        images
    );
    let [
        CssImageValue::Gradient(CssGradient::Linear(linear)),
        CssImageValue::Gradient(CssGradient::Radial(radial)),
    ] = images.images()
    else {
        panic!("ordered gradients")
    };
    let [
        CssColorStopListItem::Stop(red),
        CssColorStopListItem::Stop(blue),
    ] = linear.stops().items()
    else {
        panic!("two stops")
    };
    assert_eq!(red.color().named().unwrap().name(), "red");
    assert_eq!(blue.color().named().unwrap().name(), "blue");
    literal(
        red.position().unwrap().literal_component().unwrap(),
        "-1e-999",
        Some("px"),
    );
    literal(
        blue.position().unwrap().literal_component().unwrap(),
        "1e999",
        Some("%"),
    );
    let Some(CssRadialSize::Ellipse(size)) = radial.size() else {
        panic!("ellipse radii")
    };
    literal(
        size.horizontal().literal_component().unwrap(),
        "1e999",
        Some("px"),
    );
    literal(
        size.vertical().literal_component().unwrap(),
        "25",
        Some("%"),
    );
}

#[test]
fn transform_z_is_pure_length_and_translate_arity_is_retained() {
    let source = "transform-origin:25% top -1e999px";
    let origin = value!(source, TransformOrigin, origin);
    literal(
        origin.z().unwrap().literal_component().unwrap(),
        "-1e999",
        Some("px"),
    );
    parsed_origin(origin.z().unwrap().origin(), source, "-1e999px");

    let CssTranslate::Values(one) = value!("translate:25%", Translate, value) else {
        panic!("translate value")
    };
    literal(one.x().literal_component().unwrap(), "25", Some("%"));
    assert!(one.y().is_none());
    assert!(one.z().is_none());
    let CssTranslate::Values(three) = value!("translate:25% -1px 1e999px", Translate, value) else {
        panic!("translate triple")
    };
    literal(
        three.y().unwrap().literal_component().unwrap(),
        "-1",
        Some("px"),
    );
    literal(
        three.z().unwrap().literal_component().unwrap(),
        "1e999",
        Some("px"),
    );
    assert!(
        CssTranslateValues::try_new(length_percentage("25%"), None, Some(length("1px"))).is_none()
    );
}

#[test]
fn basic_shape_round_retains_radii_arity_vertical_omission_and_assignment() {
    let source = "clip-path:inset(-1e-999px 25% 2px round 1px 2% 3px)";
    let CssClipPath::BasicShape(clip_shape) = value!(source, ClipPath, value) else {
        panic!("inset shape")
    };
    let CssBasicShape::Inset(inset) = clip_shape.shape() else {
        panic!("inset shape")
    };
    assert_eq!(inset.offsets().values().len(), 3);
    literal(
        inset.offsets().values()[0].literal_component().unwrap(),
        "-1e-999",
        Some("px"),
    );
    parsed_origin(inset.offsets().values()[0].origin(), source, "-1e-999px");
    let round = inset.round().unwrap();
    assert_eq!(round.horizontal_values().len(), 3);
    assert!(round.authored_vertical_values().is_none());
    for corner in [
        round.top_left(),
        round.top_right(),
        round.bottom_right(),
        round.bottom_left(),
    ] {
        assert!(corner.authored_vertical().is_none());
        assert_eq!(corner.horizontal().origin(), corner.vertical().origin());
    }
    literal(
        round.top_left().horizontal().literal_component().unwrap(),
        "1",
        Some("px"),
    );
    literal(
        round.top_right().horizontal().literal_component().unwrap(),
        "2",
        Some("%"),
    );
    literal(
        round
            .bottom_right()
            .horizontal()
            .literal_component()
            .unwrap(),
        "3",
        Some("px"),
    );
    literal(
        round
            .bottom_left()
            .horizontal()
            .literal_component()
            .unwrap(),
        "2",
        Some("%"),
    );

    let CssClipPath::BasicShape(clip_shape) = value!(
        "clip-path:inset(0 round 1px 2px / 3% 4% 5%)",
        ClipPath,
        value
    ) else {
        panic!("slash radius shape")
    };
    let CssBasicShape::Inset(inset) = clip_shape.shape() else {
        panic!("slash radius shape")
    };
    let round = inset.round().unwrap();
    assert_eq!(round.horizontal_values().len(), 2);
    assert_eq!(round.authored_vertical_values().unwrap().len(), 3);
    literal(
        round
            .bottom_left()
            .authored_vertical()
            .unwrap()
            .literal_component()
            .unwrap(),
        "4",
        Some("%"),
    );
}

#[test]
fn nonnegative_consumers_reject_tiny_negative_literals_and_pure_context_percentages() {
    for source in [
        "outline-width:-1e-999px",
        "background-size:-1e-999%",
        "border-image-width:-1e-999px",
        "border-image-outset:-1e-999px",
        "filter:blur(-1e-999px)",
        "box-shadow:1px 2px -1e-999px",
        "filter:drop-shadow(1px 2px -1e-999px)",
        "text-decoration-thickness:-1e-999%",
        "clip-path:circle(-1e-999%)",
        "clip-path:inset(0 round -1e-999px)",
        "background-image:radial-gradient(circle 25%, red, blue)",
        "transform:perspective(-1e-999px)",
        "transform:translateZ(1%)",
        "outline-offset:1%",
        "translate:0 0 1%",
        "clip:rect(0, 1%, 0, 0)",
        "filter:blur(1e-999)",
    ] {
        let report = parse_style_attribute(source);
        assert!(!report.is_clean(), "invalid domain accepted: {source}");
        assert!(
            report.syntax().is_empty(),
            "invalid declaration retained: {source}"
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration)
        );
    }
}

#[test]
fn checked_programmatic_owners_preserve_origins_and_composite_resource_limits() {
    let spacing = CssBorderSpacing::new(nonnegative_length("1e999px"), nonnegative_length("2px"));
    literal(
        spacing.horizontal().literal_component().unwrap(),
        "1e999",
        Some("px"),
    );
    assert_eq!(spacing.horizontal().origin(), &CssValueOrigin::Programmatic);
    assert_eq!(
        spacing
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 1008))
            .unwrap(),
        format!("1{}px 2px", "0".repeat(999))
    );
    for limits in [
        CssSpecifiedValueSerializationLimits::new(2, 3, 1008),
        CssSpecifiedValueSerializationLimits::new(3, 2, 1008),
        CssSpecifiedValueSerializationLimits::new(3, 3, 1005),
    ] {
        assert!(spacing.serialize_specified_with_limits(limits).is_err());
    }
    literal(
        spacing.horizontal().literal_component().unwrap(),
        "1e999",
        Some("px"),
    );
    let position = CssPosition::try_new(
        CssHorizontalPosition::Offset(length_percentage("1e-999px")),
        CssVerticalPosition::Offset(length_percentage("25%")),
    )
    .unwrap();
    let projected = position
        .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(5, 5, 1020))
        .unwrap();
    assert!(projected.ends_with("px 25%"));
    assert!(
        position
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(4, 5, 1020))
            .is_err()
    );
}

#[test]
fn typography_and_flow_keep_signed_percentages_and_keyword_branches() {
    let source = "text-indent:-1e-999% each-line hanging";
    let indent = value!(source, TextIndent, value);
    assert!(indent.hanging());
    assert!(indent.each_line());
    literal(
        indent.length().literal_component().unwrap(),
        "-1e-999",
        Some("%"),
    );
    parsed_origin(indent.length().origin(), source, "-1e-999%");
    let constructed = CssTextIndent::new(length_percentage("-1e-999%"), false, true);
    assert!(!constructed.hanging());
    assert!(constructed.each_line());
    assert_eq!(constructed.length().origin(), &CssValueOrigin::Programmatic);

    let CssVerticalAlign::Length(align) = value!("vertical-align:-1e999%", VerticalAlign, value)
    else {
        panic!("signed percentage alignment")
    };
    literal(align.literal_component().unwrap(), "-1e999", Some("%"));
    assert_eq!(
        value!("vertical-align:baseline", VerticalAlign, value),
        CssVerticalAlign::Baseline
    );
    let CssTextDecorationThickness::Length(thickness) = value!(
        "text-decoration-thickness:-0.000e999%",
        TextDecorationThickness,
        value
    ) else {
        panic!("nonnegative lexical zero thickness")
    };
    literal(
        thickness.literal_component().unwrap(),
        "-0.000e999",
        Some("%"),
    );
    assert_eq!(
        value!(
            "text-decoration-thickness:from-font",
            TextDecorationThickness,
            value
        ),
        CssTextDecorationThickness::FromFont
    );

    let source = "flow-tolerance:-1e999px";
    let flow = value!(source, FlowTolerance, value);
    let CssFlowToleranceRef::LengthPercentage(tolerance) = flow.as_ref() else {
        panic!("signed flow length")
    };
    literal(tolerance.literal_component().unwrap(), "-1e999", Some("px"));
    parsed_origin(tolerance.origin(), source, "-1e999px");
    assert_eq!(
        value!("flow-tolerance:infinite", FlowTolerance, value).as_ref(),
        CssFlowToleranceRef::Infinite
    );
}

#[test]
fn border_image_width_and_outset_keep_distinct_domains_and_edge_assignment() {
    let source = "border-image-width:1e999px 25% auto";
    let widths = value!(source, BorderImageWidth, widths);
    let [
        CssBorderImageWidthComponent::LengthPercentage(top),
        CssBorderImageWidthComponent::LengthPercentage(right),
        CssBorderImageWidthComponent::Auto,
        CssBorderImageWidthComponent::LengthPercentage(left),
    ] = widths.values()
    else {
        panic!("three-value edge assignment")
    };
    literal(top.literal_component().unwrap(), "1e999", Some("px"));
    literal(right.literal_component().unwrap(), "25", Some("%"));
    literal(left.literal_component().unwrap(), "25", Some("%"));
    assert_eq!(right.origin(), left.origin());
    parsed_origin(top.origin(), source, "1e999px");

    let outsets = value!(
        "border-image-outset:-0.000e999px 2px",
        BorderImageOutset,
        outsets
    );
    let [
        CssBorderImageOutsetComponent::Length(top),
        CssBorderImageOutsetComponent::Length(right),
        CssBorderImageOutsetComponent::Length(bottom),
        CssBorderImageOutsetComponent::Length(left),
    ] = outsets.values()
    else {
        panic!("two-value edge assignment")
    };
    literal(top.literal_component().unwrap(), "-0.000e999", Some("px"));
    literal(
        bottom.literal_component().unwrap(),
        "-0.000e999",
        Some("px"),
    );
    literal(right.literal_component().unwrap(), "2", Some("px"));
    literal(left.literal_component().unwrap(), "2", Some("px"));
    assert_eq!(top.origin(), bottom.origin());
    assert_eq!(right.origin(), left.origin());
}

#[test]
fn aggregate_equality_ignores_only_origins_and_retains_exact_structure() {
    let parsed_spacing = value!("border-spacing:0", BorderSpacing, spacing);
    let programmatic_spacing = CssBorderSpacing::new(
        CssSpecifiedNonNegativeLength::zero(),
        CssSpecifiedNonNegativeLength::zero(),
    );
    assert_eq!(parsed_spacing, programmatic_spacing);
    assert_ne!(
        parsed_spacing.horizontal(),
        programmatic_spacing.horizontal()
    );
    assert_ne!(
        parsed_spacing,
        CssBorderSpacing::new(nonnegative_length("0px"), nonnegative_length("0px"))
    );
    let parsed = value!("flow-tolerance:25%", FlowTolerance, value);
    let programmatic = CssFlowTolerance::length_percentage(length_percentage("25%"));
    assert_eq!(parsed, programmatic);
    let CssFlowToleranceRef::LengthPercentage(parsed_scalar) = parsed.as_ref() else {
        panic!("percentage")
    };
    let CssFlowToleranceRef::LengthPercentage(programmatic_scalar) = programmatic.as_ref() else {
        panic!("percentage")
    };
    assert_ne!(parsed_scalar, programmatic_scalar);
    assert!(matches!(parsed_scalar.origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(programmatic_scalar.origin(), &CssValueOrigin::Programmatic);
    assert_ne!(
        parsed,
        CssFlowTolerance::length_percentage(length_percentage("25.0%"))
    );

    let math = value!("flow-tolerance:calc(1px + 2%)", FlowTolerance, value);
    let other_origin = value!(" flow-tolerance:calc(1px + 2%)", FlowTolerance, value);
    assert_eq!(math, other_origin);
    assert_ne!(
        math,
        value!("flow-tolerance:calc(1px - 2%)", FlowTolerance, value)
    );
    assert_ne!(
        math,
        value!("flow-tolerance:calc(2% + 1px)", FlowTolerance, value)
    );

    let position = value!("object-position:1px top", ObjectPosition, position);
    let constructed = CssPosition::try_new(
        CssHorizontalPosition::Offset(length_percentage("1px")),
        CssVerticalPosition::Top,
    )
    .unwrap();
    assert_eq!(position, constructed);
    assert_ne!(
        position,
        CssPosition::try_new(
            CssHorizontalPosition::Offset(length_percentage("1.0px")),
            CssVerticalPosition::Top
        )
        .unwrap()
    );

    let CssBoxShadow::Shadows(shadows) = value!("box-shadow:1px 2px", BoxShadow, value) else {
        panic!("shadow")
    };
    let [parsed] = shadows.shadows() else {
        panic!("one shadow")
    };
    assert_eq!(
        parsed,
        &CssShadow::try_new(false, length("1px"), length("2px"), None, None, None).unwrap()
    );
    assert_ne!(
        parsed,
        &CssShadow::try_new(
            false,
            length("1px"),
            length("2px"),
            Some(nonnegative_length("0px")),
            None,
            None
        )
        .unwrap()
    );
}

#[test]
fn clip_and_transform_aggregate_equality_keeps_axes_and_omission() {
    let clip = value!("clip:rect(1px, auto, 2px, 0)", Clip, clip);
    let expected = CssClip::Rect(CssClipRect::new(
        CssClipEdge::Length(length("1px")),
        CssClipEdge::Auto,
        CssClipEdge::Length(length("2px")),
        CssClipEdge::Length(length("0")),
    ));
    assert_eq!(clip, expected);
    let CssClip::Rect(rect) = &clip else {
        panic!("rectangle")
    };
    let CssClipEdge::Length(top) = rect.top() else {
        panic!("top")
    };
    assert!(matches!(top.origin(), CssValueOrigin::Parsed(_)));
    assert_ne!(rect.top(), &CssClipEdge::Length(length("1.0px")));

    let planar = || {
        CssPosition::try_new(
            CssHorizontalPosition::Offset(length_percentage("25%")),
            CssVerticalPosition::Top,
        )
        .unwrap()
    };
    let origin = value!("transform-origin:25% top 1px", TransformOrigin, origin);
    assert_eq!(
        origin,
        CssTransformOrigin::try_new(planar(), Some(length("1px"))).unwrap()
    );
    assert_ne!(origin, CssTransformOrigin::try_new(planar(), None).unwrap());
    let triple = value!("translate:25% 2px 3px", Translate, value);
    assert_eq!(
        triple,
        CssTranslate::Values(
            CssTranslateValues::try_new(
                length_percentage("25%"),
                Some(length_percentage("2px")),
                Some(length("3px"))
            )
            .unwrap()
        )
    );
    assert_ne!(
        triple,
        CssTranslate::Values(
            CssTranslateValues::try_new(
                length_percentage("25%"),
                Some(length_percentage("2px")),
                None
            )
            .unwrap()
        )
    );
    let CssTransform::Functions(functions) = value!(
        "transform:translateZ(3px) perspective(4px)",
        Transform,
        value
    ) else {
        panic!("transform list")
    };
    assert_eq!(
        functions.functions(),
        &[
            CssTransformFunction::TranslateZ(length("3px")),
            CssTransformFunction::Perspective(CssTransformPerspective::Length(nonnegative_length(
                "4px"
            )))
        ]
    );
    assert_ne!(
        functions.functions()[0],
        CssTransformFunction::TranslateX(length_percentage("3px"))
    );
}

#[test]
fn filter_and_shape_aggregate_equality_keeps_omission_and_authored_arity() {
    let CssFilter::Functions(functions) =
        value!("filter:blur(0px) drop-shadow(1px 2px)", Filter, value)
    else {
        panic!("filter list")
    };
    let [
        CssFilterFunction::Blur(blur),
        CssFilterFunction::DropShadow(shadow),
    ] = functions.functions()
    else {
        panic!("ordered filters")
    };
    assert_eq!(blur, &CssFilterBlur::new(nonnegative_length("0px")));
    assert_ne!(blur, &CssFilterBlur::omitted());
    assert_eq!(
        shadow,
        &CssDropShadow::new(length("1px"), length("2px"), None, None)
    );
    assert_ne!(
        shadow,
        &CssDropShadow::new(
            length("1px"),
            length("2px"),
            Some(nonnegative_length("0px")),
            None
        )
    );
    assert!(matches!(blur.length().origin(), CssValueOrigin::Parsed(_)));

    let circle = value!("clip-path:circle(25%)", ClipPath, value);
    assert_eq!(
        circle,
        CssClipPath::BasicShape(CssClipPathShape::new(
            CssBasicShape::Circle(CssCircleShape::new(
                CssCircleRadius::LengthPercentage(nonnegative_length_percentage("25%")),
                None
            )),
            None
        ))
    );
    assert_ne!(
        circle,
        CssClipPath::BasicShape(CssClipPathShape::new(
            CssBasicShape::Circle(CssCircleShape::new(CssCircleRadius::Default, None)),
            None
        ))
    );
    let CssClipPath::BasicShape(clip_shape) = value!("clip-path:inset(1px 2%)", ClipPath, value)
    else {
        panic!("inset shape")
    };
    let CssBasicShape::Inset(inset) = clip_shape.shape() else {
        panic!("inset shape")
    };
    assert_eq!(
        inset.offsets(),
        &CssInsetShapeOffsets::try_new(vec![length_percentage("1px"), length_percentage("2%")])
            .unwrap()
    );
    assert_ne!(
        inset.offsets(),
        &CssInsetShapeOffsets::try_new(vec![
            length_percentage("1px"),
            length_percentage("2%"),
            length_percentage("1px"),
            length_percentage("2%")
        ])
        .unwrap()
    );
    let CssClipPath::BasicShape(clip_shape) =
        value!("clip-path:polygon(1px 2%, 3px 4%, 5px 6%)", ClipPath, value)
    else {
        panic!("polygon")
    };
    let CssBasicShape::Polygon(polygon) = clip_shape.shape() else {
        panic!("polygon")
    };
    assert_eq!(polygon.points().points().len(), 3);
    assert_eq!(
        polygon.points().points()[0],
        CssPolygonPoint::new(length_percentage("1px"), length_percentage("2%"))
    );
    assert!(matches!(
        polygon.points().points()[0].x().origin(),
        CssValueOrigin::Parsed(_)
    ));
}

#[test]
fn image_and_border_image_aggregate_equality_retains_exact_branches() {
    let images = value!(
        "background-image:linear-gradient(red 1px, blue 90%), radial-gradient(circle 4px, red, blue)",
        BackgroundImage,
        images
    );
    let [
        CssImageValue::Gradient(CssGradient::Linear(linear)),
        CssImageValue::Gradient(CssGradient::Radial(radial)),
    ] = images.images()
    else {
        panic!("two gradients")
    };
    let CssColorStopListItem::Stop(red) = &linear.stops().items()[0] else {
        panic!("red stop")
    };
    assert_eq!(
        **red,
        CssGradientColorStop::from_color(red.color().clone(), Some(length_percentage("1px")))
    );
    assert_ne!(
        **red,
        CssGradientColorStop::from_color(red.color().clone(), None)
    );
    assert_eq!(
        radial.size(),
        Some(&CssRadialSize::Circle(nonnegative_length("4px")))
    );
    assert_ne!(
        radial.size(),
        Some(&CssRadialSize::Ellipse(CssRadialEllipseSize::new(
            nonnegative_length_percentage("4px"),
            nonnegative_length_percentage("4px")
        )))
    );
    assert!(matches!(
        red.position().unwrap().origin(),
        CssValueOrigin::Parsed(_)
    ));

    let widths = value!("border-image-width:3% auto", BorderImageWidth, widths);
    assert_eq!(
        widths,
        CssBorderImageWidth::try_new(vec![
            CssBorderImageWidthComponent::LengthPercentage(nonnegative_length_percentage("3%")),
            CssBorderImageWidthComponent::Auto
        ])
        .unwrap()
    );
    assert_ne!(
        widths,
        CssBorderImageWidth::try_new(vec![
            CssBorderImageWidthComponent::LengthPercentage(nonnegative_length_percentage("3.0%")),
            CssBorderImageWidthComponent::Auto
        ])
        .unwrap()
    );
    let outsets = value!("border-image-outset:4px", BorderImageOutset, outsets);
    assert_eq!(
        outsets,
        CssBorderImageOutset::try_new(vec![CssBorderImageOutsetComponent::Length(
            nonnegative_length("4px")
        )])
        .unwrap()
    );
}

#[test]
fn gradient_serialization_omits_only_exact_zero_hundred_and_center_values() {
    let images = value!(
        "background-image:linear-gradient(red 1e-999px, blue 100.000001%), radial-gradient(at 50.000001% 50%, red, blue)",
        BackgroundImage,
        images
    );
    assert_eq!(
        images.images()[0].serialize_specified().unwrap(),
        format!(
            "linear-gradient(red 0.{}1px, blue 100.000001%)",
            "0".repeat(998)
        )
    );
    assert_eq!(
        images.images()[1].serialize_specified().unwrap(),
        "radial-gradient(at 50.000001% 50%, red, blue)"
    );
    let defaults = value!(
        "background-image:linear-gradient(red -0px, blue 100%), radial-gradient(at 50% 50%, red, blue)",
        BackgroundImage,
        images
    );
    assert_eq!(
        defaults.images()[0].serialize_specified().unwrap(),
        "linear-gradient(red, blue)"
    );
    assert_eq!(
        defaults.images()[1].serialize_specified().unwrap(),
        "radial-gradient(red, blue)"
    );
}
