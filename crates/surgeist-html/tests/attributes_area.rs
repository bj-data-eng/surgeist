//! Literal oracles from HTML #attr-area-shape/#attr-area-coords and
//! #image-map-processing-model, plus exact authored number meaning in §2.3.4.3.
use surgeist_html::*;

fn attr(name: KnownHtmlAttribute, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(name.qualified_name(), value)
}
fn read(shape: Option<&str>, coords: Option<&str>) -> HtmlAreaCoordinatesReport {
    let shape = shape.map(|value| attr(KnownHtmlAttribute::Shape, value));
    let coords = coords.map(|value| attr(KnownHtmlAttribute::Coords, value));
    HtmlValueInterpreter::new(4096)
        .area_coordinates(
            &KnownHtmlElement::Area.qualified_name(),
            shape.as_ref(),
            coords.as_ref(),
        )
        .unwrap()
}
fn error(report: &HtmlAreaCoordinatesReport, expected: HtmlAreaCoordinatesAuthorError) {
    assert_eq!(
        report.author(),
        HtmlAreaCoordinatesAuthorState::Nonconforming(expected)
    );
}
fn point(point: HtmlAreaProcessedPoint) -> (f64, f64) {
    (point.x().value(), point.y().value())
}

#[test]
fn keywords_recognize_aliases_but_only_canonical_forms_conform() {
    for (raw, canonical, state, conforming) in [
        ("circle", "circle", HtmlAreaShape::Circle, true),
        ("CiRcLe", "circle", HtmlAreaShape::Circle, true),
        ("CIRC", "circle", HtmlAreaShape::Circle, false),
        ("default", "default", HtmlAreaShape::WholeImage, true),
        ("poly", "poly", HtmlAreaShape::Polygon, true),
        ("polygon", "poly", HtmlAreaShape::Polygon, false),
        ("rect", "rect", HtmlAreaShape::Rectangle, true),
        ("rectangle", "rect", HtmlAreaShape::Rectangle, false),
    ] {
        let report = read(Some(raw), None);
        assert_eq!(report.shape().intrinsic(), Some(&state));
        assert_eq!(
            report.shape().recognition(),
            HtmlAttributeRecognition::Keyword {
                matched: match raw {
                    "CiRcLe" => "circle",
                    "CIRC" => "circ",
                    _ => raw,
                },
                canonical
            }
        );
        assert_eq!(
            report.shape().author(),
            if conforming {
                HtmlAttributeAuthorState::Conforming
            } else {
                HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::NonconformingKeyword,
                )
            }
        );
        assert_eq!(
            state.into_attribute().value(),
            &DocumentString::from(canonical)
        );
    }
}

#[test]
fn absent_invalid_empty_and_space_shape_default_without_changing_recognition() {
    let report = read(None, None);
    assert_eq!(report.shape().author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(
        report.shape().recognition(),
        HtmlAttributeRecognition::Missing
    );
    error(&report, HtmlAreaCoordinatesAuthorError::MissingRequired);
    assert_eq!(
        report.processed_shape(),
        HtmlAreaProcessedShape::Empty(HtmlAreaEmptyReason::TooFewCoordinates {
            required: 4,
            actual: 0
        })
    );
    for raw in ["", "unknown", " circle", "circle ", "\tpoly"] {
        let report = read(Some(raw), Some("0,0,1,1"));
        assert_eq!(report.shape().intrinsic(), Some(&HtmlAreaShape::Rectangle));
        assert_eq!(
            report.shape().author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword
            )
        );
        assert_eq!(report.author(), HtmlAreaCoordinatesAuthorState::Conforming);
        assert!(matches!(
            report.processed_shape(),
            HtmlAreaProcessedShape::Rectangle(_)
        ));
    }
}

#[test]
fn zero_and_negative_radii_have_distinct_author_constraints_and_empty_processing() {
    for radius in ["0", "-0", "0e999", "-0.000"] {
        let report = read(Some("circle"), Some(&format!("1,2,{radius}")));
        assert_eq!(report.author(), HtmlAreaCoordinatesAuthorState::Conforming);
        assert_eq!(
            report.processed_shape(),
            HtmlAreaProcessedShape::Empty(HtmlAreaEmptyReason::NonpositiveRadius)
        );
    }
    let report = read(Some("circle"), Some("1,2,-1"));
    error(&report, HtmlAreaCoordinatesAuthorError::NegativeRadius);
    assert_eq!(
        report.processed_shape(),
        HtmlAreaProcessedShape::Empty(HtmlAreaEmptyReason::NonpositiveRadius)
    );
}

#[test]
fn integer_constraint_uses_exact_decimal_meaning_not_rounded_binary64() {
    for raw in ["1.0,2e0,3", "10e-1,2,300e-2"] {
        assert_eq!(
            read(Some("circle"), Some(raw)).author(),
            HtmlAreaCoordinatesAuthorState::Conforming
        );
    }
    for raw in ["1,2,1.00000000000000001", "1,2,1e-999"] {
        let report = read(Some("circle"), Some(raw));
        error(
            &report,
            HtmlAreaCoordinatesAuthorError::NonintegerItem { index: 2 },
        );
    }
    let report = read(Some("circle"), Some("1,2,1.00000000000000001"));
    let HtmlAreaProcessedShape::Circle(circle) = report.processed_shape() else {
        panic!("circle")
    };
    assert_eq!(circle.radius().value(), 1.0);
    let report = read(Some("circle"), Some("1,2,1e999"));
    assert_eq!(report.author(), HtmlAreaCoordinatesAuthorState::Conforming);
    assert!(matches!(
        report.common().unwrap().author(),
        HtmlValueAuthorState::Unrepresentable(_)
    ));
    assert_eq!(
        report.processed_shape(),
        HtmlAreaProcessedShape::Empty(HtmlAreaEmptyReason::NonpositiveRadius)
    );
}

#[test]
fn rectangle_author_order_is_exact_even_when_processed_edges_round_equal() {
    let report = read(Some("rect"), Some("9007199254740992,0,9007199254740993,1"));
    assert_eq!(report.author(), HtmlAreaCoordinatesAuthorState::Conforming);
    let HtmlAreaProcessedShape::Rectangle(rect) = report.processed_shape() else {
        panic!("rectangle")
    };
    assert_eq!(point(rect.top_left()), (9007199254740992.0, 0.0));
    assert_eq!(point(rect.bottom_right()), (9007199254740992.0, 1.0));
    let report = read(Some("rect"), Some("9,8,1,2"));
    error(&report, HtmlAreaCoordinatesAuthorError::NonincreasingX);
    let HtmlAreaProcessedShape::Rectangle(rect) = report.processed_shape() else {
        panic!("rectangle")
    };
    assert_eq!(point(rect.top_left()), (1.0, 2.0));
    assert_eq!(point(rect.bottom_right()), (9.0, 8.0));
    error(
        &read(Some("rect"), Some("0,9,1,2")),
        HtmlAreaCoordinatesAuthorError::NonincreasingY,
    );
    let report = read(Some("rect"), Some("0,0,0,1"));
    error(&report, HtmlAreaCoordinatesAuthorError::NonincreasingX);
    assert!(matches!(
        report.processed_shape(),
        HtmlAreaProcessedShape::Rectangle(_)
    ));
}

#[test]
fn excess_circle_and_rectangle_items_fail_author_cardinality_but_are_dropped() {
    let report = read(Some("circle"), Some("1,2,3,99"));
    error(&report, HtmlAreaCoordinatesAuthorError::Cardinality);
    let HtmlAreaProcessedShape::Circle(circle) = report.processed_shape() else {
        panic!("circle")
    };
    assert_eq!(point(circle.center()), (1.0, 2.0));
    assert_eq!(circle.radius().value(), 3.0);
    assert_eq!(recovered(&report).len(), 4);
    let report = read(Some("rect"), Some("9,8,1,2,99"));
    error(&report, HtmlAreaCoordinatesAuthorError::Cardinality);
    let HtmlAreaProcessedShape::Rectangle(rect) = report.processed_shape() else {
        panic!("rectangle")
    };
    assert_eq!(point(rect.top_left()), (1.0, 2.0));
    assert_eq!(point(rect.bottom_right()), (9.0, 8.0));
}
fn recovered(report: &HtmlAreaCoordinatesReport) -> &[HtmlFloatingPointNumber] {
    let HtmlValueRecoveryState::Value(list) = report.common().unwrap().recovery() else {
        panic!("recovery")
    };
    list.values()
}

#[test]
fn polygon_minimum_precedes_odd_truncation_and_vertices_borrow_common_order() {
    let report = read(Some("poly"), Some("0,0,1,0,0"));
    error(&report, HtmlAreaCoordinatesAuthorError::Cardinality);
    assert_eq!(
        report.processed_shape(),
        HtmlAreaProcessedShape::Empty(HtmlAreaEmptyReason::TooFewCoordinates {
            required: 6,
            actual: 5
        })
    );
    let report = read(Some("poly"), Some("0,0,1,0,0,1,99"));
    error(&report, HtmlAreaCoordinatesAuthorError::Cardinality);
    let HtmlAreaProcessedShape::Polygon(poly) = report.processed_shape() else {
        panic!("polygon")
    };
    assert_eq!(
        poly.vertices().map(point).collect::<Vec<_>>(),
        [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)]
    );
    assert_eq!(poly.coordinates().len(), 6);
    assert_eq!(poly.coordinates().as_ptr(), recovered(&report).as_ptr());
    assert_eq!(recovered(&report).len(), 7);
    // Repeated and collinear vertices are not prohibited by this author clause.
    assert_eq!(
        read(Some("poly"), Some("0,0,0,0,0,0")).author(),
        HtmlAreaCoordinatesAuthorState::Conforming
    );
}

#[test]
fn strict_list_and_recovery_remain_common_owner_outputs() {
    for raw in ["1;2;3", " 1,2,3 ", "garbage1,+2,3", "1,2,3junk"] {
        let report = read(Some("circle"), Some(raw));
        error(&report, HtmlAreaCoordinatesAuthorError::InvalidListSyntax);
        let common = HtmlValueInterpreter::new(4096)
            .floating_point_list(Some(&DocumentString::from(raw)))
            .unwrap();
        assert_eq!(report.common(), Some(&common));
        assert!(matches!(
            report.processed_shape(),
            HtmlAreaProcessedShape::Circle(_)
        ));
    }
    for (shape, required) in [("circle", 3), ("rect", 4), ("poly", 6)] {
        let report = read(Some(shape), Some(""));
        error(&report, HtmlAreaCoordinatesAuthorError::Cardinality);
        assert!(matches!(
            report.common().unwrap().author(),
            HtmlValueAuthorState::Valid(_)
        ));
        assert_eq!(
            report.processed_shape(),
            HtmlAreaProcessedShape::Empty(HtmlAreaEmptyReason::TooFewCoordinates {
                required,
                actual: 0
            })
        );
    }
}

#[test]
fn default_is_symbolic_whole_image_and_forbids_any_coords_presence() {
    let report = read(Some("default"), None);
    assert_eq!(report.author(), HtmlAreaCoordinatesAuthorState::Conforming);
    assert_eq!(report.processed_shape(), HtmlAreaProcessedShape::WholeImage);
    for raw in ["", "garbage", "1,2,3"] {
        let report = read(Some("default"), Some(raw));
        error(&report, HtmlAreaCoordinatesAuthorError::ForbiddenPresence);
        assert_eq!(report.processed_shape(), HtmlAreaProcessedShape::WholeImage);
        assert!(report.common().is_some());
        assert_eq!(report.coords().raw(), Some(&DocumentString::from(raw)));
    }
}

#[test]
fn applicability_uses_expanded_names_preserves_exact_raw_and_admits_first() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "area");
    let shape = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("s".into()), "shape"),
        "CiRc",
    );
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("c".into()), "coords"),
        DocumentString::from_utf16(vec![49, 44, 50, 44, 0xD800]),
    );
    let report = HtmlValueInterpreter::new(5)
        .area_coordinates(&element, Some(&shape), Some(&raw))
        .unwrap();
    assert_eq!(report.shape().element(), &element);
    assert_eq!(report.shape().raw_attribute(), Some(&shape));
    assert_eq!(report.coords().raw_attribute(), Some(&raw));
    assert_eq!(report.coords().inventory_rows(), &[33]);
    assert_eq!(report.shape().inventory_rows(), &[142]);
    assert!(
        report
            .processing_source_clause()
            .ends_with("/#image-map-processing-model")
    );
    error(&report, HtmlAreaCoordinatesAuthorError::InvalidListSyntax);
    let wrong = attr(KnownHtmlAttribute::Cite, "bad");
    let report = HtmlValueInterpreter::new(5)
        .area_coordinates(&element, Some(&shape), Some(&wrong))
        .unwrap();
    assert!(report.common().is_none());
    assert_eq!(report.author(), HtmlAreaCoordinatesAuthorState::Unapplied);
    assert_eq!(
        report.processed_shape(),
        HtmlAreaProcessedShape::Unapplied(HtmlAttributeApplicability::DifferentAttribute)
    );
    let huge_wrong = attr(KnownHtmlAttribute::Cite, "123456");
    let admission = HtmlValueInterpreter::new(5)
        .area_coordinates(
            &KnownHtmlElement::Div.qualified_name(),
            None,
            Some(&huge_wrong),
        )
        .unwrap_err();
    assert_eq!((admission.capacity(), admission.required()), (5, 6));
    for element in [
        KnownHtmlElement::Div.qualified_name(),
        QualifiedName::new(Namespace::svg(), None, "area"),
        QualifiedName::new(Namespace::html(), None, "AREA"),
    ] {
        let report = HtmlValueInterpreter::new(5)
            .area_coordinates(&element, Some(&shape), Some(&raw))
            .unwrap();
        assert_eq!(report.author(), HtmlAreaCoordinatesAuthorState::Unapplied);
        assert!(report.common().is_none());
        assert!(matches!(
            report.processed_shape(),
            HtmlAreaProcessedShape::Unapplied(_)
        ));
    }
}

fn exact(digits: &str, negative: bool) -> HtmlInteger {
    HtmlInteger::from_magnitude(
        HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(digits)).unwrap(),
        negative,
    )
}
fn native_point(x: i128, y: i128) -> HtmlAreaIntegerPoint {
    HtmlAreaIntegerPoint::new(x.into(), y.into())
}
#[test]
fn native_constructors_enforce_only_exact_intrinsic_geometry_and_serialize() {
    let circle = HtmlAuthoredArea::Circle(HtmlAreaCircle::new(native_point(-1, 2), 0.into()));
    let (shape, coords) = circle.into_attributes();
    assert_eq!(shape.value(), &DocumentString::from("circle"));
    assert_eq!(
        coords.as_ref().unwrap().value(),
        &DocumentString::from("-1,2,0")
    );
    let report = HtmlValueInterpreter::new(100)
        .area_coordinates(
            &KnownHtmlElement::Area.qualified_name(),
            Some(&shape),
            coords.as_ref(),
        )
        .unwrap();
    assert_eq!(report.author(), HtmlAreaCoordinatesAuthorState::Conforming);
    assert_eq!(
        report.processed_shape(),
        HtmlAreaProcessedShape::Empty(HtmlAreaEmptyReason::NonpositiveRadius)
    );
    assert_eq!(
        HtmlAreaRectangle::new(native_point(0, 0), native_point(0, 1)),
        Err(HtmlAreaNativeError::NonincreasingX)
    );
    assert_eq!(
        HtmlAreaRectangle::new(native_point(0, 1), native_point(1, 0)),
        Err(HtmlAreaNativeError::NonincreasingY)
    );
    assert_eq!(
        HtmlAreaPolygon::new(vec![native_point(0, 0), native_point(1, 1)]),
        Err(HtmlAreaNativeError::TooFewVertices)
    );
    let poly = HtmlAreaPolygon::new(vec![
        native_point(0, 0),
        native_point(0, 0),
        native_point(0, 0),
    ])
    .unwrap();
    assert_eq!(
        HtmlAuthoredArea::Polygon(poly)
            .into_attributes()
            .1
            .unwrap()
            .value(),
        &DocumentString::from("0,0,0,0,0,0")
    );
    let (shape, coords) = HtmlAuthoredArea::WholeImage.into_attributes();
    assert_eq!(shape.value(), &DocumentString::from("default"));
    assert!(coords.is_none());
}

#[test]
fn native_rectangle_has_no_machine_integer_ceiling() {
    let n = "340282366920938463463374607431768211456";
    let next = "340282366920938463463374607431768211457";
    let rect = HtmlAreaRectangle::new(
        HtmlAreaIntegerPoint::new(exact(n, true), exact(n, true)),
        HtmlAreaIntegerPoint::new(exact(next, false), exact(next, false)),
    )
    .unwrap();
    assert_eq!(rect.top_left().x().magnitude().decimal_digits(), n);
    let (_, coords) = HtmlAuthoredArea::Rectangle(rect).into_attributes();
    assert_eq!(
        coords.unwrap().value(),
        &DocumentString::from(format!("-{n},-{n},{next},{next}"))
    );
}

#[test]
fn each_raw_slot_has_independent_admission_and_wrong_names_never_parse() {
    let element = KnownHtmlElement::Area.qualified_name();
    let shape = attr(KnownHtmlAttribute::Shape, "rect");
    let coords = attr(KnownHtmlAttribute::Coords, "0,0,1,1");
    let report = HtmlValueInterpreter::new(7)
        .area_coordinates(&element, Some(&shape), Some(&coords))
        .unwrap();
    assert_eq!(report.coords().input_code_units(), 7);
    let failure = HtmlValueInterpreter::new(6)
        .area_coordinates(&element, Some(&shape), Some(&coords))
        .unwrap_err();
    assert_eq!((failure.capacity(), failure.required()), (6, 7));
    let too_long_shape = attr(KnownHtmlAttribute::Cite, "12345678");
    let failure = HtmlValueInterpreter::new(7)
        .area_coordinates(&element, Some(&too_long_shape), Some(&coords))
        .unwrap_err();
    assert_eq!((failure.capacity(), failure.required()), (7, 8));
    for raw in [
        Attribute::new(QualifiedName::new(Namespace::html(), None, "coords"), "bad"),
        Attribute::new(QualifiedName::new(Namespace::none(), None, "COORDS"), "bad"),
    ] {
        let report = HtmlValueInterpreter::new(20)
            .area_coordinates(&element, Some(&shape), Some(&raw))
            .unwrap();
        assert_eq!(report.coords().raw_attribute(), Some(&raw));
        assert_eq!(
            report.coords().applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert!(report.common().is_none());
    }
    let wrong_shape = attr(KnownHtmlAttribute::Coords, "circle");
    let report = HtmlValueInterpreter::new(20)
        .area_coordinates(&element, Some(&wrong_shape), Some(&coords))
        .unwrap();
    assert_eq!(
        report.shape().applicability(),
        HtmlAttributeApplicability::DifferentAttribute
    );
    assert!(report.common().is_some());
    assert_eq!(report.author(), HtmlAreaCoordinatesAuthorState::Unapplied);
    assert_eq!(
        report.processed_shape(),
        HtmlAreaProcessedShape::Unapplied(HtmlAttributeApplicability::DifferentAttribute)
    );
}

#[test]
fn huge_exponents_and_signs_order_exactly_without_expanding_coordinates() {
    let report = read(
        Some("rect"),
        Some("1e999999999999999999999999,0,1e1000000000000000000000000,1"),
    );
    assert_eq!(report.author(), HtmlAreaCoordinatesAuthorState::Conforming);
    assert!(matches!(
        report.common().unwrap().author(),
        HtmlValueAuthorState::Unrepresentable(_)
    ));
    let HtmlAreaProcessedShape::Rectangle(rect) = report.processed_shape() else {
        panic!("rectangle")
    };
    assert_eq!(point(rect.top_left()), (0.0, 0.0));
    assert_eq!(point(rect.bottom_right()), (0.0, 1.0));
    assert_eq!(
        read(Some("rect"), Some("-999999999999999999999999999,0,-1,1")).author(),
        HtmlAreaCoordinatesAuthorState::Conforming
    );
    error(
        &read(Some("rect"), Some("1,0,1.0,1")),
        HtmlAreaCoordinatesAuthorError::NonincreasingX,
    );
    error(
        &read(Some("poly"), Some("0,0,1,0,0,1.00000000000000001")),
        HtmlAreaCoordinatesAuthorError::NonintegerItem { index: 5 },
    );
}

#[test]
fn minimums_missing_and_diagnostic_precedence_keep_common_evidence() {
    for (shape, required, raw) in [
        ("circle", 3, "1,2"),
        ("rect", 4, "1,2,3"),
        ("poly", 6, "1,2,3,4"),
    ] {
        let missing = read(Some(shape), None);
        error(&missing, HtmlAreaCoordinatesAuthorError::MissingRequired);
        assert_eq!(
            missing.common().unwrap().author(),
            &HtmlValueAuthorState::Missing
        );
        let report = read(Some(shape), Some(raw));
        error(&report, HtmlAreaCoordinatesAuthorError::Cardinality);
        assert_eq!(
            report.processed_shape(),
            HtmlAreaProcessedShape::Empty(HtmlAreaEmptyReason::TooFewCoordinates {
                required,
                actual: required - if shape == "poly" { 2 } else { 1 }
            })
        );
    }
    let report = read(Some("circle"), Some("1.5,2,3,4"));
    error(&report, HtmlAreaCoordinatesAuthorError::Cardinality);
    let report = read(Some("circle"), Some("1e999,nope,3"));
    error(&report, HtmlAreaCoordinatesAuthorError::InvalidListSyntax);
    assert!(matches!(
        report.common().unwrap().author(),
        HtmlValueAuthorState::Invalid(_)
    ));
    let old = read(Some("circle"), Some("1,2,3"));
    let later = read(Some("circle"), Some("4,5,-6"));
    assert_eq!(old.coords().raw(), Some(&DocumentString::from("1,2,3")));
    assert!(matches!(
        old.processed_shape(),
        HtmlAreaProcessedShape::Circle(_)
    ));
    assert_eq!(
        later.processed_shape(),
        HtmlAreaProcessedShape::Empty(HtmlAreaEmptyReason::NonpositiveRadius)
    );
}

#[test]
fn known_whole_image_needs_no_coordinate_role_but_keeps_unapplied_evidence() {
    let shape = attr(KnownHtmlAttribute::Shape, "default");
    let wrong = attr(KnownHtmlAttribute::Cite, "not coordinates");
    let report = HtmlValueInterpreter::new(20)
        .area_coordinates(
            &KnownHtmlElement::Area.qualified_name(),
            Some(&shape),
            Some(&wrong),
        )
        .unwrap();
    assert_eq!(report.coords().raw_attribute(), Some(&wrong));
    assert_eq!(
        report.coords().applicability(),
        HtmlAttributeApplicability::DifferentAttribute
    );
    assert_eq!(report.author(), HtmlAreaCoordinatesAuthorState::Unapplied);
    assert!(report.common().is_none());
    assert_eq!(report.processed_shape(), HtmlAreaProcessedShape::WholeImage);
}
