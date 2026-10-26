//! Functional evidence for new APIs, without preimplementation executable API RED.
//! Independent expectations follow pinned Shapes 1 §§3.1–3.3 and the adopted
//! authored serialization resource contract. No contextual geometry is inferred.
use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::normal_declaration;
use crate::css::support::property_admission::assert_component_rejected;
use crate::css::support::serialization_limits::assert_limits as budget;
use surgeist_css::{CssComponentValue, CssComponentValues, CssSerializationLimits as L, *};

fn lp(css: &str) -> CssNumeric<CssLengthPercentage> {
    CssNumeric::<CssLengthPercentage>::try_from_component(
        CssComponentValue::try_token(css).unwrap(),
    )
    .unwrap()
}
fn nn(css: &str) -> CssNumeric<CssLengthPercentage, CssNonNegative> {
    CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
        CssComponentValue::try_token(css).unwrap(),
    )
    .unwrap()
}
fn edge(css: &str) -> CssAutoOr<CssNumeric<CssLengthPercentage>> {
    CssAutoOr::Value(lp(css))
}
fn basic_shape(css: &str) -> CssBasicShape {
    let declaration = normal_declaration("clip-path", css);
    let CssKnownPropertyValueRef::ClipPath(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("clip path")
    };
    let CssClipPath::BasicShape(composition) = value.value() else {
        panic!("shape")
    };
    composition.shape().clone()
}
#[test]
fn checked_rect_composition_retains_each_named_edge_and_omitted_rounding() {
    let top = edge("-1px");
    let right = CssAutoOr::Auto;
    let bottom = edge("2%");
    let left = edge("-3em");
    let rect = CssRectShape::new(
        top.clone(),
        right.clone(),
        bottom.clone(),
        left.clone(),
        None,
    );
    assert_eq!(rect.top(), &top);
    assert_eq!(rect.right(), &right);
    assert_eq!(rect.bottom(), &bottom);
    assert_eq!(rect.left(), &left);
    assert!(rect.round().is_none());
    assert_eq!(
        rect.serialize_specified(Default::default()).unwrap(),
        "rect(-1px auto 2% -3em)"
    );
    assert_eq!(
        basic_shape("rect(-1px auto 2% -3em)"),
        CssBasicShape::Rect(rect.clone())
    );
    assert_ne!(rect, CssRectShape::new(left, right, bottom, top, None));
    for auto_index in 0..4 {
        let mut edges = [edge("1px"), edge("2%"), edge("3em"), edge("4px")];
        edges[auto_index] = CssAutoOr::Auto;
        let rect = CssRectShape::new(
            edges[0].clone(),
            edges[1].clone(),
            edges[2].clone(),
            edges[3].clone(),
            None,
        );
        for (index, actual) in [rect.top(), rect.right(), rect.bottom(), rect.left()]
            .into_iter()
            .enumerate()
        {
            assert_eq!(actual, &edges[index]);
        }
    }
}

#[test]
fn checked_xywh_composition_retains_signed_offsets_and_nonnegative_sizes() {
    let x = lp("-1px");
    let y = lp("-2%");
    let width = nn("3em");
    let height = nn("0");
    let xywh = CssXywhShape::new(x.clone(), y.clone(), width.clone(), height.clone(), None);
    assert_eq!(xywh.x(), &x);
    assert_eq!(xywh.y(), &y);
    assert_eq!(xywh.width(), &width);
    assert_eq!(xywh.height(), &height);
    assert!(xywh.round().is_none());
    assert_eq!(
        xywh.serialize_specified(Default::default()).unwrap(),
        "xywh(-1px -2% 3em 0)"
    );
    assert_eq!(
        basic_shape("xywh(-1px -2% 3em 0)"),
        CssBasicShape::Xywh(xywh.clone())
    );
    assert_ne!(xywh, CssXywhShape::new(y, x, height, width, None));
    for (invalid, kind) in [
        ("-1px", NumericFailure::OutOfRange),
        ("-1%", NumericFailure::OutOfRange),
        ("-1e-999px", NumericFailure::OutOfRange),
        ("-1e-999%", NumericFailure::OutOfRange),
        ("auto", NumericFailure::RootDomainMismatch),
        ("1", NumericFailure::RootDomainMismatch),
        ("1deg", NumericFailure::RootDomainMismatch),
    ] {
        assert_component_rejected(
            CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component,
            CssComponentValue::try_token(invalid).unwrap(),
            kind,
        );
    }
    for (invalid, kind) in [
        ("auto", NumericFailure::RootDomainMismatch),
        ("1", NumericFailure::RootDomainMismatch),
        ("1deg", NumericFailure::RootDomainMismatch),
    ] {
        assert_component_rejected(
            CssNumeric::<CssLengthPercentage>::try_from_component,
            CssComponentValue::try_token(invalid).unwrap(),
            kind,
        );
    }
    assert_eq!(lp("-1e-999px").origin(), &CssValueOrigin::Programmatic);
}

#[test]
fn semantic_equality_preserves_separately_observable_numeric_origins() {
    for (text, constructed) in [
        (
            "rect(auto 1px 2% 0)",
            CssBasicShape::Rect(CssRectShape::new(
                CssAutoOr::Auto,
                edge("1px"),
                edge("2%"),
                edge("0"),
                None,
            )),
        ),
        (
            "xywh(-1px 2% 3em 4vh)",
            CssBasicShape::Xywh(CssXywhShape::new(
                lp("-1px"),
                lp("2%"),
                nn("3em"),
                nn("4vh"),
                None,
            )),
        ),
    ] {
        let value = basic_shape(text);
        assert_eq!(value, constructed);
        let (parsed_scalar, constructed_scalar) = match (&value, &constructed) {
            (CssBasicShape::Rect(a), CssBasicShape::Rect(b)) => {
                let (CssAutoOr::Value(a), CssAutoOr::Value(b)) = (a.right(), b.right()) else {
                    panic!("edge")
                };
                (a, b)
            }
            (CssBasicShape::Xywh(a), CssBasicShape::Xywh(b)) => (a.x(), b.x()),
            _ => panic!("rectangle"),
        };
        let origin = parsed_scalar.origin().clone();
        assert!(matches!(origin, CssValueOrigin::Parsed(_)));
        assert_eq!(constructed_scalar.origin(), &CssValueOrigin::Programmatic);
        value.serialize_specified(L::new(1, 100, 100)).unwrap_err();
        assert_eq!(parsed_scalar.origin(), &origin);
        value.serialize_specified(Default::default()).unwrap();
        assert_eq!(parsed_scalar.origin(), &origin);
    }
    // Structural equality ignores provenance only, retaining exact lexical structure.
    assert_eq!(
        basic_shape("rect(auto 1px 2% 0)"),
        basic_shape("rect(auto 1.00px 2% 0)")
    );
    assert_eq!(
        basic_shape("xywh(-1px 2% 3em 4vh)"),
        basic_shape("xywh(-1.00px 2% 3em 4vh)")
    );
}

#[test]
fn round_arity_slash_and_omission_remain_authored_distinctions() {
    for horizontal_count in 1..=4 {
        for vertical_count in 0..=4 {
            let horizontal: Vec<_> = ["1px", "2%", "3em", "4px"][..horizontal_count]
                .iter()
                .map(|s| nn(s))
                .collect();
            let vertical = (vertical_count != 0).then(|| {
                ["5px", "6%", "7em", "8px"][..vertical_count]
                    .iter()
                    .map(|s| nn(s))
                    .collect()
            });
            let round = CssBorderRadiusShorthand::try_new(horizontal, vertical).unwrap();
            let mut suffix = ["1px", "2%", "3em", "4px"][..horizontal_count].join(" ");
            if vertical_count != 0 {
                suffix.push_str(" / ");
                suffix.push_str(&["5px", "6%", "7em", "8px"][..vertical_count].join(" "));
            }
            let rect = CssRectShape::new(
                edge("0"),
                edge("1px"),
                edge("2%"),
                edge("3px"),
                Some(round.clone()),
            );
            let xywh =
                CssXywhShape::new(lp("0"), lp("1px"), nn("2%"), nn("3px"), Some(round.clone()));
            assert_eq!(rect.round(), Some(&round));
            assert_eq!(xywh.round(), Some(&round));
            assert_eq!(
                rect.round().unwrap().horizontal_values().len(),
                horizontal_count
            );
            assert_eq!(
                rect.round()
                    .unwrap()
                    .authored_vertical_values()
                    .map(<[_]>::len),
                (vertical_count != 0).then_some(vertical_count)
            );
            let nodes = 6 + horizontal_count + vertical_count;
            budget(
                &format!("rect(0 1px 2% 3px round {suffix})"),
                nodes,
                nodes,
                |l| rect.serialize_specified(l),
            );
            budget(
                &format!("xywh(0 1px 2% 3px round {suffix})"),
                nodes,
                nodes,
                |l| xywh.serialize_specified(l),
            );
        }
    }
    let omitted = basic_shape("rect(0 0 1px 2%)");
    assert_ne!(omitted, basic_shape("rect(0 0 1px 2% round 0)"));
    assert_ne!(
        basic_shape("xywh(0 0 1px 2% round 0)"),
        basic_shape("xywh(0 0 1px 2% round 0 / 0)")
    );
}

#[test]
fn literal_budgets_are_five_standalone_six_composed_seven_boxed() {
    let rect = CssRectShape::new(CssAutoOr::Auto, edge("1px"), edge("2%"), edge("0"), None);
    budget("rect(auto 1px 2% 0)", 5, 5, |l| rect.serialize_specified(l));
    let xywh = CssXywhShape::new(lp("0"), lp("0"), nn("1px"), nn("2%"), None);
    budget("xywh(0 0 1px 2%)", 5, 5, |l| xywh.serialize_specified(l));
    for (shape, expected) in [
        (CssBasicShape::Rect(rect), "rect(auto 1px 2% 0)"),
        (CssBasicShape::Xywh(xywh), "xywh(0 0 1px 2%)"),
    ] {
        budget(expected, 5, 5, |l| shape.serialize_specified(l));
        let composition = CssClipPathShape::new(shape.clone(), None);
        budget(expected, 6, 6, |l| composition.serialize_specified(l));
        budget(expected, 6, 6, |l| {
            CssClipPath::BasicShape(composition.clone()).serialize_specified(l)
        });
        for box_edge in [CssBoxEdgeKeyword::BorderBox, CssBoxEdgeKeyword::ViewBox] {
            let composition = CssClipPathShape::new(shape.clone(), Some(box_edge));
            let text = format!("{expected} {}", box_edge.as_str());
            budget(&text, 7, 7, |l| composition.serialize_specified(l));
            budget(&text, 7, 7, |l| {
                CssClipPath::BasicShape(composition.clone()).serialize_specified(l)
            });
        }
    }
}

#[test]
fn four_math_children_and_rounding_share_one_cumulative_provider_budget() {
    // Each mixed LP calculation costs four input/five projection nodes.
    // The shape adds one; round adds one and its two children add eight/ten.
    for name in ["rect", "xywh"] {
        let value = basic_shape(&format!(
            "{name}(calc(1px + 2%) calc(3em + 4%) calc(5px + 6%) calc(7px + 8%) round calc(9px + 10%) / calc(11px + 12%))"
        ));
        let before = value.clone();
        let expected = format!(
            "{name}(calc(2% + 1px) calc(4% + 3em) calc(6% + 5px) calc(8% + 7px) round calc(10% + 9px) / calc(12% + 11px))"
        );
        budget(&expected, 26, 32, |l| value.serialize_specified(l));
        match &value {
            CssBasicShape::Rect(v) => budget(&expected, 26, 32, |l| v.serialize_specified(l)),
            CssBasicShape::Xywh(v) => budget(&expected, 26, 32, |l| v.serialize_specified(l)),
            _ => panic!("rectangle"),
        }
        let composition = CssClipPathShape::new(value.clone(), Some(CssBoxEdgeKeyword::BorderBox));
        budget(&format!("{expected} border-box"), 28, 34, |l| {
            composition.serialize_specified(l)
        });
        assert_eq!(value, before);
    }
}

#[test]
fn exact_magnitudes_retain_identity_while_ordinary_text_rounds_and_math_is_preserved() {
    let rect = CssRectShape::new(
        edge("-1e-999px"),
        edge("123456789012345678901234567890px"),
        edge("1.50e2%"),
        edge("0"),
        None,
    );
    assert_eq!(
        rect.serialize_specified(Default::default()).unwrap(),
        "rect(0px 123456789012345678901234567890px 150% 0)"
    );
    let xywh = CssXywhShape::new(
        lp("-1e-999px"),
        lp("1.50e2%"),
        nn("1e-999px"),
        nn("123456789012345678901234567890px"),
        None,
    );
    let before = xywh.clone();
    assert_eq!(
        xywh.serialize_specified(Default::default()).unwrap(),
        "xywh(0px 150% 0px 123456789012345678901234567890px)"
    );
    assert_eq!(xywh, before);
    assert_eq!(
        CssComponentValues::try_new(
            vec![
                xywh.x()
                    .literal()
                    .map(|literal| literal.component())
                    .unwrap()
                    .clone()
            ],
            Default::default()
        )
        .unwrap()
        .serialize_tokens(Default::default())
        .unwrap()
        .as_css(),
        "-1e-999px",
    );
    assert_eq!(
        CssComponentValues::try_new(
            vec![
                xywh.width()
                    .literal()
                    .map(|literal| literal.component())
                    .unwrap()
                    .clone()
            ],
            Default::default()
        )
        .unwrap()
        .serialize_tokens(Default::default())
        .unwrap()
        .as_css(),
        "1e-999px",
    );
    assert_eq!(
        basic_shape("xywh(0 0 calc(1px - 2px) calc(-1%))")
            .serialize_specified(Default::default())
            .unwrap(),
        "xywh(0 0 calc(-1px) calc(-1%))"
    );
}

#[test]
fn shape_front_doors_preserve_construction_and_budgets() {
    let circle = CssCircleShape::new(CssCircleRadius::Default, None);
    budget("circle()", 1, 1, |l| circle.serialize_specified(l));
    let radii = CssEllipseRadii::new(
        CssEllipseRadius::LengthPercentage(nn("1px")),
        CssEllipseRadius::Extent(CssRadialExtent::ClosestSide),
    );
    let ellipse = CssEllipseShape::new(Some(radii.clone()), None);
    assert_eq!(ellipse.radii(), Some(&radii));
    budget("ellipse(1px closest-side)", 4, 4, |l| {
        ellipse.serialize_specified(l)
    });
    let inset = CssInsetShape::new(
        CssInsetShapeOffsets::try_new(vec![lp("1px")]).unwrap(),
        None,
    );
    budget("inset(1px)", 3, 3, |l| inset.serialize_specified(l));
    let point = CssPolygonPoint::new(lp("0"), lp("1%"));
    assert_eq!(
        point.x().serialize_specified(Default::default()).unwrap(),
        "0"
    );
    let polygon = CssPolygonShape::new(
        None,
        None,
        CssPolygonPointList::try_new(vec![point]).unwrap(),
    );
    budget("polygon(0 1%)", 5, 5, |l| polygon.serialize_specified(l));
    assert_eq!(
        basic_shape("inset(1px)"),
        CssBasicShape::Inset(Box::new(inset))
    );
}
