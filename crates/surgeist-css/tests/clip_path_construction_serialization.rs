#![forbid(unsafe_code)]
//! Functional evidence for new authored composition, ellipse component and serializer APIs.
//! These APIs have no executable preimplementation RED. Expectations are specified
//! independently by Masking 1 §5.1, the adopted Shapes 1 ellipse/polygon decisions,
//! and the authored resource contract. No used geometry or defaults are inferred.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

fn parsed(css: &str) -> CssClipPath {
    let report = parse_style_attribute(&format!("clip-path:{css}"));
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::ClipPath(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("clip path")
    };
    value.value().clone()
}

fn shape(css: &str) -> CssBasicShape {
    let CssClipPath::BasicShape(value) = parsed(css) else {
        panic!("shape")
    };
    value.shape().clone()
}

fn lp(css: &str) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_component(CssComponentValue::try_token(css).unwrap())
        .unwrap()
}

fn radius(css: &str) -> CssSpecifiedNonNegativeLengthPercentage {
    CssSpecifiedNonNegativeLengthPercentage::try_from_component(
        CssComponentValue::try_token(css).unwrap(),
    )
    .unwrap()
}

fn length(css: &str) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_component(CssComponentValue::try_token(css).unwrap()).unwrap()
}

fn budget(
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(
        serialize(L::new(input, projection, expected.len())).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            L::new(input - 1, projection, expected.len()),
            K::InputNodeLimit,
        ),
        (
            L::new(input, projection - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (L::new(input, projection, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), kind, "{expected}");
    }
    assert_eq!(serialize(L::default()).unwrap(), expected);
}

#[test]
fn shape_composition_keeps_box_identity_and_omission_separate_from_border_box() {
    let circle = CssBasicShape::Circle(CssCircleShape::new(CssCircleRadius::Default, None));
    let omitted = CssClipPathShape::new(circle.clone(), None);
    assert_eq!(omitted.shape(), &circle);
    assert_eq!(omitted.reference_box(), None);
    for (text, edge) in [
        ("content-box", CssBoxEdgeKeyword::ContentBox),
        ("padding-box", CssBoxEdgeKeyword::PaddingBox),
        ("border-box", CssBoxEdgeKeyword::BorderBox),
        ("margin-box", CssBoxEdgeKeyword::MarginBox),
        ("fill-box", CssBoxEdgeKeyword::FillBox),
        ("stroke-box", CssBoxEdgeKeyword::StrokeBox),
        ("view-box", CssBoxEdgeKeyword::ViewBox),
    ] {
        assert_eq!(parsed(text), CssClipPath::GeometryBox(edge));
        let composition = CssClipPathShape::new(circle.clone(), Some(edge));
        assert_eq!(composition.shape(), &circle);
        assert_eq!(composition.reference_box(), Some(edge));
        assert_ne!(composition, omitted);
        let expected = CssClipPath::BasicShape(composition.clone());
        assert_eq!(parsed(&format!("{text} circle()")), expected);
        assert_eq!(parsed(&format!("circle() {text}")), expected);
        budget(&format!("circle() {text}"), 3, 3, |limits| {
            composition.serialize_specified_with_limits(limits)
        });
        budget(&format!("circle() {text}"), 3, 3, |limits| {
            expected.serialize_specified_with_limits(limits)
        });
    }
    budget("circle()", 2, 2, |limits| {
        omitted.serialize_specified_with_limits(limits)
    });
    budget("circle()", 2, 2, |limits| {
        CssClipPath::BasicShape(omitted.clone()).serialize_specified_with_limits(limits)
    });
}

#[test]
fn ellipse_pairs_preserve_independent_components_order_and_omission() {
    let horizontal = CssEllipseRadius::LengthPercentage(radius("10px"));
    let vertical = CssEllipseRadius::Extent(CssRadialExtent::ClosestSide);
    let pair = CssEllipseRadii::new(horizontal.clone(), vertical.clone());
    assert_eq!(pair.horizontal(), &horizontal);
    assert_eq!(pair.vertical(), &vertical);
    let mixed = CssEllipseShape::new(Some(pair.clone()), None);
    assert_eq!(mixed.radii(), Some(&pair));
    assert!(mixed.position().is_none());
    assert_eq!(
        shape("ellipse(10px closest-side)"),
        CssBasicShape::Ellipse(mixed.clone())
    );
    assert_ne!(
        mixed,
        CssEllipseShape::new(
            Some(CssEllipseRadii::new(vertical.clone(), horizontal)),
            None
        )
    );
    let omitted = CssEllipseShape::new(None, None);
    assert!(omitted.radii().is_none());
    assert_ne!(
        omitted,
        CssEllipseShape::new(Some(CssEllipseRadii::new(vertical.clone(), vertical)), None)
    );
    for (first, second) in [
        (
            CssRadialExtent::ClosestSide,
            CssRadialExtent::FarthestCorner,
        ),
        (
            CssRadialExtent::FarthestSide,
            CssRadialExtent::ClosestCorner,
        ),
    ] {
        let pair = CssEllipseRadii::new(
            CssEllipseRadius::Extent(first),
            CssEllipseRadius::Extent(second),
        );
        assert_eq!(pair.horizontal(), &CssEllipseRadius::Extent(first));
        assert_eq!(pair.vertical(), &CssEllipseRadius::Extent(second));
    }
    budget("ellipse(10px closest-side)", 4, 4, |limits| {
        mixed.serialize_specified_with_limits(limits)
    });
    budget("ellipse()", 1, 1, |limits| {
        omitted.serialize_specified_with_limits(limits)
    });
    assert_ne!(
        CssEllipseRadius::LengthPercentage(radius("0")),
        CssEllipseRadius::LengthPercentage(radius("0px"))
    );
    // Existing scalar structural equality preserves numeric token representation.
    assert_ne!(
        CssEllipseRadius::LengthPercentage(radius("1.0px")),
        CssEllipseRadius::LengthPercentage(radius("1px"))
    );
}

#[test]
fn checked_shapes_retain_signed_rounding_offsets_point_lists_and_radius_arity() {
    let offsets = CssInsetShapeOffsets::try_new(vec![lp("-1px"), lp("2%"), lp("3em")]).unwrap();
    let round = CssBorderRadiusShorthand::try_new(
        vec![radius("4px"), radius("5%")],
        Some(vec![radius("6px")]),
    )
    .unwrap();
    let inset = CssInsetShape::new(offsets.clone(), Some(round.clone()));
    assert_eq!(inset.offsets(), &offsets);
    assert_eq!(inset.round(), Some(&round));
    assert_eq!(inset.offsets().values().len(), 3);
    assert_eq!(inset.round().unwrap().horizontal_values().len(), 2);
    assert_eq!(
        inset
            .round()
            .unwrap()
            .authored_vertical_values()
            .unwrap()
            .len(),
        1
    );
    budget("inset(-1px 2% 3em round 4px 5% / 6px)", 9, 9, |limits| {
        inset.serialize_specified_with_limits(limits)
    });
    assert!(CssInsetShapeOffsets::try_new(vec![]).is_none());
    assert!(CssInsetShapeOffsets::try_new(vec![lp("0"); 5]).is_none());
    assert!(CssPolygonPointList::try_new(vec![]).is_none());
    let point = CssPolygonPoint::new(lp("-1px"), lp("25%"));
    assert_eq!(point.x().serialize_specified().unwrap(), "-1px");
    assert_eq!(point.y().serialize_specified().unwrap(), "25%");
    let points = CssPolygonPointList::try_new(vec![point.clone()]).unwrap();
    let signed = length("-1px");
    let polygon = CssPolygonShape::new(
        Some(CssPolygonFillRule::Nonzero),
        Some(signed.clone()),
        points.clone(),
    );
    assert_eq!(polygon.fill_rule(), Some(CssPolygonFillRule::Nonzero));
    assert_eq!(polygon.round(), Some(&signed));
    assert_eq!(polygon.points().points(), &[point]);
    assert_ne!(
        polygon,
        CssPolygonShape::new(None, Some(signed), points.clone())
    );
    assert_ne!(
        CssPolygonShape::new(None, None, points.clone()),
        CssPolygonShape::new(None, Some(length("0")), points)
    );
    budget("polygon(nonzero round -1px, -1px 25%)", 7, 7, |limits| {
        polygon.serialize_specified_with_limits(limits)
    });
    for invalid in ["10%", "1"] {
        assert!(
            CssSpecifiedLength::try_from_component(CssComponentValue::try_token(invalid).unwrap())
                .is_err()
        );
    }
    assert!(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            CssComponentValue::try_token("-1e-999px").unwrap()
        )
        .is_err()
    );
}

#[test]
fn each_function_and_transparent_basic_shape_branch_has_the_declared_literal_budget() {
    for (css, expected, nodes) in [
        ("circle()", "circle()", 1),
        ("ellipse()", "ellipse()", 1),
        ("circle(10%)", "circle(10%)", 2),
        ("circle(at top left)", "circle(at left top)", 4),
        ("circle(closest-corner)", "circle(closest-corner)", 2),
        (
            "ellipse(10px closest-side)",
            "ellipse(10px closest-side)",
            4,
        ),
        (
            "ellipse(closest-corner farthest-side at left top)",
            "ellipse(closest-corner farthest-side at left top)",
            7,
        ),
        ("inset(1px)", "inset(1px)", 3),
        ("inset(1px round 2px)", "inset(1px round 2px)", 5),
        ("polygon(0 0)", "polygon(0 0)", 5),
        (
            "polygon(nonzero round -1px, 0 0)",
            "polygon(nonzero round -1px, 0 0)",
            7,
        ),
        (
            "polygon(evenodd round 0, 0 0, 100% 0)",
            "polygon(evenodd round 0, 0 0, 100% 0)",
            10,
        ),
    ] {
        let value = shape(css);
        let before = value.clone();
        budget(expected, nodes, nodes, |limits| {
            value.serialize_specified_with_limits(limits)
        });
        match &value {
            CssBasicShape::Circle(value) => {
                budget(expected, nodes, nodes, |limits| {
                    value.serialize_specified_with_limits(limits)
                });
                assert_eq!(value.serialize_specified().unwrap(), expected);
            }
            CssBasicShape::Ellipse(value) => {
                budget(expected, nodes, nodes, |limits| {
                    value.serialize_specified_with_limits(limits)
                });
                assert_eq!(value.serialize_specified().unwrap(), expected);
            }
            CssBasicShape::Inset(value) => {
                budget(expected, nodes, nodes, |limits| {
                    value.serialize_specified_with_limits(limits)
                });
                assert_eq!(value.serialize_specified().unwrap(), expected);
            }
            CssBasicShape::Polygon(value) => {
                budget(expected, nodes, nodes, |limits| {
                    value.serialize_specified_with_limits(limits)
                });
                assert_eq!(value.serialize_specified().unwrap(), expected);
            }
            _ => panic!("selected shape"),
        }
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value, before);
    }
}

#[test]
fn standalone_none_geometry_and_url_delegate_without_synthetic_nodes() {
    for (value, expected, nodes) in [
        (CssClipPath::None, "none", 1),
        (
            CssClipPath::GeometryBox(CssBoxEdgeKeyword::ViewBox),
            "view-box",
            1,
        ),
        (CssClipPath::Url(CssUrl::new("#🦀")), "url(\"#🦀\")", 2),
    ] {
        let before = value.clone();
        budget(expected, nodes, nodes, |limits| {
            value.serialize_specified_with_limits(limits)
        });
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value, before);
    }
}

#[test]
fn parsed_and_programmatic_literals_retain_units_exact_magnitudes_and_origins() {
    for (css, expected) in [
        ("circle(1.50e2%)", "circle(150%)"),
        (
            "circle(123456789012345678901234567890px)",
            "circle(123456789012345678901234567890px)",
        ),
        ("ellipse(10em 20vh)", "ellipse(10em 20vh)"),
    ] {
        assert_eq!(shape(css).serialize_specified().unwrap(), expected);
    }
    // Exact canonical decimal expansion of 10^-999 retains its magnitude.
    let tiny = format!("0.{}1px", "0".repeat(998));
    for (css, expected) in [
        ("circle(1e-999px)", format!("circle({tiny})")),
        (
            "polygon(round -1e-999px, 0 0)",
            format!("polygon(round -{tiny}, 0 0)"),
        ),
        ("inset(-1e-999px)", format!("inset(-{tiny})")),
    ] {
        assert_eq!(shape(css).serialize_specified().unwrap(), expected);
    }
    let parsed = shape("ellipse(10px closest-side)");
    let CssBasicShape::Ellipse(ellipse) = &parsed else {
        panic!("ellipse")
    };
    let CssEllipseRadius::LengthPercentage(value) = ellipse.radii().unwrap().horizontal() else {
        panic!("numeric component")
    };
    let origin = value.origin().clone();
    assert!(matches!(origin, CssValueOrigin::Parsed(_)));
    let constructed = CssBasicShape::Ellipse(CssEllipseShape::new(
        Some(CssEllipseRadii::new(
            CssEllipseRadius::LengthPercentage(radius("10px")),
            CssEllipseRadius::Extent(CssRadialExtent::ClosestSide),
        )),
        None,
    ));
    assert_eq!(parsed, constructed);
    let CssBasicShape::Ellipse(ellipse) = &constructed else {
        panic!("constructed ellipse")
    };
    let CssEllipseRadius::LengthPercentage(value) = ellipse.radii().unwrap().horizontal() else {
        panic!("numeric component")
    };
    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    budget("ellipse(10px closest-side)", 4, 4, |limits| {
        parsed.serialize_specified_with_limits(limits)
    });
    let CssBasicShape::Ellipse(ellipse) = &parsed else {
        panic!("ellipse")
    };
    let CssEllipseRadius::LengthPercentage(value) = ellipse.radii().unwrap().horizontal() else {
        panic!("numeric component")
    };
    assert_eq!(value.origin(), &origin);
}

#[test]
fn sibling_math_arenas_share_cumulative_input_projection_and_byte_limits() {
    // Each mixed LP calculation visits four input nodes and projects five nodes.
    // The ellipse function and authored pair add two nodes in each budget.
    let value = shape("ellipse(calc(1px + 2%) calc(3em + 4%))");
    let before = value.clone();
    let expected = "ellipse(calc(2% + 1px) calc(4% + 3em))";
    budget(expected, 10, 12, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    // Inset function + offsets list + round shorthand are three aggregates;
    // three independent mixed LP calculations contribute twelve/fifteen nodes.
    let inset = shape("inset(calc(1px + 2%) round calc(3em + 4%) / calc(5px + 6%))");
    let inset_before = inset.clone();
    budget(
        "inset(calc(2% + 1px) round calc(4% + 3em) / calc(6% + 5px))",
        15,
        18,
        |limits| inset.serialize_specified_with_limits(limits),
    );
    assert_eq!(value, before);
    assert_eq!(inset, inset_before);
    let CssBasicShape::Ellipse(ellipse) = &value else {
        panic!("ellipse")
    };
    for component in [
        ellipse.radii().unwrap().horizontal(),
        ellipse.radii().unwrap().vertical(),
    ] {
        let CssEllipseRadius::LengthPercentage(value) = component else {
            panic!("symbolic LP")
        };
        assert!(value.calculation().is_some());
        assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
    }
    let first = shape("circle(calc(1px + 2%))");
    let second = shape("circle(/**/calc(1px + 2%))");
    // Scalar structural equality ignores source identity while borrowed origins remain distinct.
    assert_eq!(first, second);
    let CssBasicShape::Circle(first) = first else {
        panic!("circle")
    };
    let CssBasicShape::Circle(second) = second else {
        panic!("circle")
    };
    let CssCircleRadius::LengthPercentage(first) = first.radius() else {
        panic!("math")
    };
    let CssCircleRadius::LengthPercentage(second) = second.radius() else {
        panic!("math")
    };
    assert_ne!(first.origin(), second.origin());
}

#[test]
fn shared_position_and_border_radius_standalone_costs_are_preserved() {
    let position = CssPosition::from_cartesian(
        CssCartesianPosition::try_new(
            CssHorizontalPosition::RightOffset(lp("5%")),
            CssVerticalPosition::BottomOffset(lp("2px")),
        )
        .unwrap(),
    );
    budget("right 5% bottom 2px", 5, 5, |limits| {
        position.serialize_specified_with_limits(limits)
    });
    let circle = CssCircleShape::new(CssCircleRadius::Default, Some(position.clone()));
    assert_eq!(circle.position(), Some(&position));
    budget("circle(at right 5% bottom 2px)", 6, 6, |limits| {
        circle.serialize_specified_with_limits(limits)
    });
    let round = CssBorderRadiusShorthand::try_new(
        vec![radius("1px"), radius("2px")],
        Some(vec![radius("3%")]),
    )
    .unwrap();
    budget("1px 2px / 3%", 4, 4, |limits| {
        round.serialize_specified_with_limits(limits)
    });
}
