#![forbid(unsafe_code)]
//! New typed Shapes APIs have functional tests alongside implementation, without
//! preimplementation missing-symbol RED. Independent expectations follow Shapes 1
//! CRD 2025-06-12 §§4, 6.1–6.3 and the selected authored omission/range contracts.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

fn parsed(property: CssKnownProperty, text: &str) -> CssDeclaration {
    let report = parse_property_value_text(
        text,
        CssPropertyNameRef::Known(property),
        CssImportance::Important,
    );
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    report.syntax().as_ref().unwrap().clone()
}

fn checked(property: CssKnownProperty, components: CssComponentValues) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        components,
        CssImportance::Important,
    )
    .unwrap()
}

fn outside(source: &CssDeclaration) -> &CssShapeOutside {
    let CssKnownPropertyValueRef::ShapeOutside(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("outside wrapper")
    };
    value.value()
}

fn threshold(source: &CssDeclaration) -> &CssOpacityValue {
    let CssKnownPropertyValueRef::ShapeImageThreshold(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("threshold wrapper")
    };
    value.value()
}

fn margin(source: &CssDeclaration) -> &CssSpecifiedNonNegativeLengthPercentage {
    let CssKnownPropertyValueRef::ShapeMargin(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("margin wrapper")
    };
    value.value()
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
        assert_eq!(serialize(limits).unwrap_err().kind(), kind);
    }
    assert_eq!(serialize(L::default()).unwrap(), expected);
}

#[test]
fn four_box_constructor_excludes_svg_boxes_and_emits_one_bounded_node() {
    for (keyword, value) in [
        ("content-box", CssShapeBox::ContentBox),
        ("padding-box", CssShapeBox::PaddingBox),
        ("border-box", CssShapeBox::BorderBox),
        ("margin-box", CssShapeBox::MarginBox),
    ] {
        assert_eq!(CssShapeBox::from_keyword(keyword), Some(value));
        assert_eq!(
            CssShapeBox::from_keyword(&keyword.to_ascii_uppercase()),
            Some(value)
        );
        assert_eq!(value.as_css_str(), keyword);
        budget(keyword, 1, 1, |limits| {
            value.serialize_specified_with_limits(limits)
        });
        let owner = CssShapeOutside::ShapeBox(value);
        budget(keyword, 1, 1, |limits| {
            owner.serialize_specified_with_limits(limits)
        });
        let input =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident(keyword).unwrap()])
                .unwrap();
        assert_eq!(
            outside(&checked(CssKnownProperty::ShapeOutside, input)),
            &owner
        );
    }
    for keyword in [
        "fill-box",
        "stroke-box",
        "view-box",
        "text",
        "no-clip",
        "none",
        "circle()",
        "border-box margin-box",
    ] {
        assert_eq!(CssShapeBox::from_keyword(keyword), None, "{keyword}");
    }
    budget("none", 1, 1, |limits| {
        CssShapeOutside::None.serialize_specified_with_limits(limits)
    });
}

#[test]
fn checked_shape_composition_retains_omission_and_each_explicit_box() {
    let shape = CssBasicShape::Circle(CssCircleShape::new(CssCircleRadius::Default, None));
    let omitted = CssShapeOutsideShape::new(shape.clone(), None);
    assert_eq!(omitted.shape(), &shape);
    assert_eq!(omitted.reference_box(), None);
    budget("circle()", 2, 2, |limits| {
        omitted.serialize_specified_with_limits(limits)
    });
    assert_eq!(
        outside(&parsed(CssKnownProperty::ShapeOutside, "circle()")),
        &CssShapeOutside::BasicShape(omitted.clone())
    );
    for (keyword, reference_box) in [
        ("content-box", CssShapeBox::ContentBox),
        ("padding-box", CssShapeBox::PaddingBox),
        ("border-box", CssShapeBox::BorderBox),
        ("margin-box", CssShapeBox::MarginBox),
    ] {
        let composed = CssShapeOutsideShape::new(shape.clone(), Some(reference_box));
        assert_eq!(composed.shape(), &shape);
        assert_eq!(composed.reference_box(), Some(reference_box));
        assert_ne!(composed, omitted);
        let expected = format!("circle() {keyword}");
        budget(&expected, 3, 3, |limits| {
            composed.serialize_specified_with_limits(limits)
        });
        let owner = CssShapeOutside::BasicShape(composed);
        budget(&expected, 3, 3, |limits| {
            owner.serialize_specified_with_limits(limits)
        });
        for text in [format!("{keyword} circle()"), expected] {
            for source in [
                parsed(CssKnownProperty::ShapeOutside, &text),
                checked(
                    CssKnownProperty::ShapeOutside,
                    parse_component_values(&text).unwrap(),
                ),
            ] {
                assert_eq!(outside(&source), &owner);
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("shape-outside: circle() {keyword} !important;")
                );
            }
        }
    }
}

#[test]
fn image_only_composition_rejects_bare_none_but_keeps_two_none_children() {
    assert_eq!(
        CssImage::try_new(CssImageValue::None).unwrap_err(),
        CssImageConstructionError::NotImage
    );
    let branches = CssLightDarkImage::try_new(CssImageValue::None, CssImageValue::None).unwrap();
    let image = CssImage::try_new(CssImageValue::LightDark(Box::new(branches))).unwrap();
    let owner = CssShapeOutside::Image(image.clone());
    let before = owner.clone();
    budget("light-dark(none, none)", 3, 3, |limits| {
        owner.serialize_specified_with_limits(limits)
    });
    assert_eq!(owner, before);
    let CssImageValue::LightDark(branches) = image.value() else {
        panic!("symbolic branches")
    };
    assert_eq!(branches.light(), &CssImageValue::None);
    assert_eq!(branches.dark(), &CssImageValue::None);
    for source in [
        parsed(CssKnownProperty::ShapeOutside, "light-dark(none, none)"),
        checked(
            CssKnownProperty::ShapeOutside,
            parse_component_values("light-dark(none, none)").unwrap(),
        ),
    ] {
        assert_eq!(outside(&source), &owner);
    }
    assert_ne!(owner, CssShapeOutside::None);
    let source = parsed(CssKnownProperty::ShapeOutside, "url(\"asset.png\")");
    let CssShapeOutside::Image(image) = outside(&source) else {
        panic!("image variant")
    };
    assert!(matches!(image.value(), CssImageValue::Url(_)));
    assert_eq!(image.serialize_specified().unwrap(), "url(\"asset.png\")");
}

#[test]
fn three_terminal_initials_have_independent_typed_payloads_and_programmatic_origins() {
    for property in [
        CssKnownProperty::ShapeOutside,
        CssKnownProperty::ShapeImageThreshold,
        CssKnownProperty::ShapeMargin,
    ] {
        let metadata = property.metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("terminal")
        };
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("fixed initial")
        };
        assert_eq!(value.property().known_property(), property);
        match value.view() {
            CssLonghandValueRef::ShapeOutside(value) => assert_eq!(value, &CssShapeOutside::None),
            CssLonghandValueRef::ShapeImageThreshold(CssOpacityValue::Scalar(value)) => {
                assert_eq!(value.kind(), CssOpacityScalarKind::Number);
                assert_eq!(value.numeric().representation(), "0");
                assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
            }
            CssLonghandValueRef::ShapeMargin(value) => {
                assert!(value.calculation().is_none());
                assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
                assert!(
                    matches!(value.literal_component().unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Number(token)) if token.representation() == "0")
                );
            }
            other => panic!("initial for {property:?}: {other:?}"),
        }
    }
}

#[test]
fn threshold_wrappers_borrow_exact_scalars_and_all_three_calculation_roots() {
    for text in ["-1e-999", "1e999", "150%", "-25%"] {
        let components = parse_component_values(text).unwrap();
        let original = components.items()[0].clone();
        let source = checked(CssKnownProperty::ShapeImageThreshold, components.clone());
        let CssOpacityValue::Scalar(value) = threshold(&source) else {
            panic!("exact scalar")
        };
        assert_eq!(value.component(), &original);
        assert_eq!(value.origin(), original.origin());
        assert_eq!(value.numeric().representation(), text.trim_end_matches('%'));
        assert_eq!(
            value.kind(),
            if text.ends_with('%') {
                CssOpacityScalarKind::Percentage
            } else {
                CssOpacityScalarKind::Number
            }
        );
        assert_eq!(source.value_components(), &components);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("one terminal")
        };
        let CssLonghandValueRef::ShapeImageThreshold(actual) =
            values.items()[0].ordinary_value().unwrap().view()
        else {
            panic!("threshold terminal")
        };
        assert_eq!(actual, threshold(&source));
    }
    for (text, root) in [
        ("calc(2 - 3)", 0),
        ("calc(25% + 25%)", 1),
        ("calc((1px + 1%) / 1px)", 2),
    ] {
        let input = parse_component_values(text).unwrap();
        let source = checked(CssKnownProperty::ShapeImageThreshold, input.clone());
        match (root, threshold(&source)) {
            (0, CssOpacityValue::NumberCalculation(value)) => {
                assert_eq!(value.components(), &input)
            }
            (1, CssOpacityValue::PercentageCalculation(value)) => {
                assert_eq!(value.components(), &input)
            }
            (2, CssOpacityValue::HintedNumberCalculation(value)) => {
                assert_eq!(value.components(), &input);
                assert_eq!(
                    value.numeric_type().percent_hint(),
                    Some(CssNumericDimension::Length)
                );
                assert_eq!(value.origin(), input.items()[0].origin());
            }
            (_, value) => panic!("root {root}: {value:?}"),
        }
    }
}

#[test]
fn margin_wrappers_keep_exact_checked_literals_and_deferred_negative_math() {
    for text in ["-0px", "-0%", "1e-999px", "25%"] {
        let components = parse_component_values(text).unwrap();
        let source = checked(CssKnownProperty::ShapeMargin, components.clone());
        let value = margin(&source);
        assert_eq!(value.literal_component(), Some(&components.items()[0]));
        assert_eq!(value.origin(), components.items()[0].origin());
        assert!(value.calculation().is_none());
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("one terminal")
        };
        let CssLonghandValueRef::ShapeMargin(actual) =
            values.items()[0].ordinary_value().unwrap().view()
        else {
            panic!("margin terminal")
        };
        assert_eq!(actual, value);
    }
    let input = parse_component_values("calc(1px - 2px)").unwrap();
    let source = checked(CssKnownProperty::ShapeMargin, input.clone());
    assert!(margin(&source).literal_component().is_none());
    assert_eq!(margin(&source).calculation().unwrap().components(), &input);
    assert_eq!(margin(&source).origin(), input.items()[0].origin());
    assert_eq!(margin(&source).serialize_specified().unwrap(), "calc(-1px)");
    for text in ["-1e-999px", "-1e-999%"] {
        let components = parse_component_values(text).unwrap();
        let numeric_error = CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            components.items()[0].clone(),
        )
        .unwrap_err();
        assert_eq!(
            numeric_error.kind(),
            &CssNumericConstructionErrorKind::OutOfRange
        );
        assert_eq!(numeric_error.origin(), Some(components.items()[0].origin()));
        let error = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ShapeMargin),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap_err();
        assert_eq!(
            error.origin(),
            &CssSerializedOrigin::Token(components.items()[0].origin().clone())
        );
    }
}

#[test]
fn checked_numeric_construction_retains_programmatic_zero_and_rechecks_bare_negative_roots() {
    for (property, text) in [
        (CssKnownProperty::ShapeImageThreshold, "150%"),
        (CssKnownProperty::ShapeMargin, "-0px"),
    ] {
        let component = CssComponentValue::try_token(text).unwrap();
        let source = checked(
            property,
            CssComponentValues::try_new(vec![component.clone()]).unwrap(),
        );
        if property == CssKnownProperty::ShapeImageThreshold {
            let CssOpacityValue::Scalar(value) = threshold(&source) else {
                panic!("scalar")
            };
            assert_eq!(value.component(), &component);
            assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
            assert_eq!(threshold(&source).serialize_specified().unwrap(), "1.5");
        } else {
            assert_eq!(margin(&source).literal_component(), Some(&component));
            assert_eq!(margin(&source).origin(), &CssValueOrigin::Programmatic);
        }
    }
    for text in ["-1e-999px", "-1e-999%"] {
        let input = parse_component_values(text).unwrap();
        let calculation =
            CssLengthPercentageCalculation::try_from_components(input.clone()).unwrap();
        let error =
            CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(calculation).unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
        assert_eq!(error.origin(), Some(input.items()[0].origin()));
    }
}

#[test]
fn replacement_terminals_borrow_new_typed_children_and_retain_the_replacement_snapshot() {
    for (property, text) in [
        (CssKnownProperty::ShapeOutside, "circle(25%) margin-box"),
        (CssKnownProperty::ShapeImageThreshold, "150%"),
        (CssKnownProperty::ShapeMargin, "calc(1px - 2px)"),
    ] {
        let source = parsed(property, "var(--shape)");
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        let replacement = parse_component_values(text).unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("one replacement terminal")
        };
        let [value] = values.items() else {
            panic!("one contribution")
        };
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert_eq!(value.replacement_components(), Some(&replacement));
        match value.ordinary_value().unwrap().view() {
            CssLonghandValueRef::ShapeOutside(CssShapeOutside::BasicShape(shape)) => {
                assert_eq!(shape.reference_box(), Some(CssShapeBox::MarginBox));
                let CssBasicShape::Circle(circle) = shape.shape() else {
                    panic!("circle")
                };
                let CssCircleRadius::LengthPercentage(radius) = circle.radius() else {
                    panic!("radius")
                };
                let CssComponentValueRef::Function(function) = replacement.items()[0].view() else {
                    panic!("function")
                };
                assert_eq!(radius.origin(), function.values().items()[0].origin());
            }
            CssLonghandValueRef::ShapeImageThreshold(CssOpacityValue::Scalar(scalar)) => {
                assert_eq!(scalar.component(), &replacement.items()[0])
            }
            CssLonghandValueRef::ShapeMargin(margin) => {
                assert_eq!(margin.calculation().unwrap().components(), &replacement)
            }
            other => panic!("replacement for {property:?}: {other:?}"),
        }
    }
}
