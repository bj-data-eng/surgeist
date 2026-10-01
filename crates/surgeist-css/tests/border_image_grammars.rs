use surgeist_css::*;

#[test]
fn c13_border_images_retain_typed_structure() {
    let report = parse_style_attribute(
        "border-image: url(frame.png) 10% 20 30% 40 fill / 1 auto 25% 4px / 0 2px 3 4px round space",
    );

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0]
            .known()
            .map(|known| known.property().canonical_name()),
        Some("border-image"),
    );

    let CssKnownPropertyValueRef::BorderImage(value) = report.syntax()[0]
        .known()
        .expect("known border-image")
        .property_value()
        .expect("ordinary border-image")
    else {
        panic!("expected border-image wrapper");
    };
    let value = value.border_image();
    assert!(matches!(
        value.source(),
        Some(CssImageValue::Url(url)) if url.as_str() == "frame.png"
    ));
    let slice = value.slice().expect("slice");
    assert!(slice.fill());
    assert!(
        matches!(
            slice.values(),
            [
                CssBorderImageSliceComponent::Percentage(top),
                CssBorderImageSliceComponent::Number(right),
                CssBorderImageSliceComponent::Percentage(bottom),
                CssBorderImageSliceComponent::Number(left),
            ] if exact_percentage(top.literal_component(), "10")
                && exact_number(right.literal_component(), "20")
                && exact_percentage(bottom.literal_component(), "30")
                && exact_number(left.literal_component(), "40")
        ),
        "{:?}",
        slice.values()
    );
    assert!(matches!(
        value.width().expect("width").values(),
        [
            CssBorderImageWidthComponent::Number(one),
            CssBorderImageWidthComponent::Auto,
            CssBorderImageWidthComponent::LengthPercentage(percent),
            CssBorderImageWidthComponent::LengthPercentage(px),
        ] if exact_number(one.literal_component(), "1")
            && exact_percentage(percent.literal_component(), "25")
            && exact_dimension(px.literal_component(), "4", "px")
    ));
    assert!(matches!(
        value.outset().expect("outset").values(),
        [
            CssBorderImageOutsetComponent::Number(zero),
            CssBorderImageOutsetComponent::Length(two),
            CssBorderImageOutsetComponent::Number(three),
            CssBorderImageOutsetComponent::Length(four),
        ] if exact_number(zero.literal_component(), "0")
            && exact_dimension(two.literal_component(), "2", "px")
            && exact_number(three.literal_component(), "3")
            && exact_dimension(four.literal_component(), "4", "px")
    ));
    let repeat = value.repeat().expect("repeat");
    assert_eq!(repeat.horizontal(), CssBorderImageRepeatKeyword::Round);
    assert_eq!(repeat.vertical(), CssBorderImageRepeatKeyword::Space);
}

#[test]
fn border_image_longhands_preserve_domains_and_expanded_arity() {
    let report = parse_style_attribute(concat!(
        "border-image-source: linear-gradient(red, blue); ",
        "border-image-slice: fill 10 20% 30 40%; ",
        "border-image-width: auto 2 30% 4px; ",
        "border-image-outset: 1 2px 3 4em; ",
        "border-image-repeat: repeat round; ",
        "border-style: dotted solid double none; ",
        "border-width: thin 2px medium thick",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let values = report
        .syntax()
        .iter()
        .map(|declaration| {
            declaration
                .known()
                .expect("known declaration")
                .property_value()
                .expect("ordinary declaration")
        })
        .collect::<Vec<_>>();

    let CssKnownPropertyValueRef::BorderImageSource(source) = values[0] else {
        panic!("expected border-image-source");
    };
    assert!(matches!(source.source(), CssImageValue::Gradient(_)));

    let CssKnownPropertyValueRef::BorderImageSlice(slice) = values[1] else {
        panic!("expected border-image-slice");
    };
    assert!(slice.slice().fill());
    assert_eq!(slice.slice().values().len(), 4);

    let CssKnownPropertyValueRef::BorderImageWidth(width) = values[2] else {
        panic!("expected border-image-width");
    };
    assert_eq!(width.widths().values().len(), 4);

    let CssKnownPropertyValueRef::BorderImageOutset(outset) = values[3] else {
        panic!("expected border-image-outset");
    };
    assert_eq!(outset.outsets().values().len(), 4);

    let CssKnownPropertyValueRef::BorderImageRepeat(repeat) = values[4] else {
        panic!("expected border-image-repeat");
    };
    assert_eq!(
        repeat.repeat().horizontal(),
        CssBorderImageRepeatKeyword::Repeat
    );
    assert_eq!(
        repeat.repeat().vertical(),
        CssBorderImageRepeatKeyword::Round
    );

    assert!(matches!(
        values[5],
        CssKnownPropertyValueRef::BorderStyle(_)
    ));
    assert!(matches!(
        values[6],
        CssKnownPropertyValueRef::BorderWidth(_)
    ));
}

#[test]
fn border_image_one_to_four_arity_expands_by_css_edge_rules() {
    for (authored, expected) in [
        ("1", ["1", "1", "1", "1"]),
        ("1 2", ["1", "2", "1", "2"]),
        ("1 2 3", ["1", "2", "3", "2"]),
        ("1 2 3 4", ["1", "2", "3", "4"]),
    ] {
        for property in [
            "border-image-slice",
            "border-image-width",
            "border-image-outset",
        ] {
            let source = format!("{property}: {authored}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            let value = report.syntax()[0]
                .known()
                .unwrap()
                .property_value()
                .unwrap();
            let actual = match value {
                CssKnownPropertyValueRef::BorderImageSlice(value) => value
                    .slice()
                    .values()
                    .iter()
                    .map(|value| match value {
                        CssBorderImageSliceComponent::Number(value) => {
                            value.serialize_specified().unwrap()
                        }
                        _ => panic!("{source}: expected number"),
                    })
                    .collect::<Vec<_>>(),
                CssKnownPropertyValueRef::BorderImageWidth(value) => value
                    .widths()
                    .values()
                    .iter()
                    .map(|value| match value {
                        CssBorderImageWidthComponent::Number(value) => {
                            value.serialize_specified().unwrap()
                        }
                        _ => panic!("{source}: expected number"),
                    })
                    .collect::<Vec<_>>(),
                CssKnownPropertyValueRef::BorderImageOutset(value) => value
                    .outsets()
                    .values()
                    .iter()
                    .map(|value| match value {
                        CssBorderImageOutsetComponent::Number(value) => {
                            value.serialize_specified().unwrap()
                        }
                        _ => panic!("{source}: expected number"),
                    })
                    .collect::<Vec<_>>(),
                _ => panic!("{source}: expected border-image component list"),
            };
            assert_eq!(actual, expected, "{source}");
        }
    }
}

#[test]
fn border_image_source_distinguishes_none_url_and_image_values() {
    for (authored, expected) in [
        ("none", "none"),
        ("url(frame.png)", "url"),
        ("radial-gradient(red, blue)", "gradient"),
    ] {
        let source = format!("border-image-source: {authored}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::BorderImageSource(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("expected border-image-source");
        };
        let actual = match value.source() {
            CssImageValue::None => "none",
            CssImageValue::Url(_) => "url",
            CssImageValue::Gradient(_) => "gradient",
            _ => "future",
        };
        assert_eq!(actual, expected, "{source}");
    }
}

#[test]
fn image_orientation_rendering_and_object_fit_retain_typed_keywords() {
    let report = parse_style_attribute(concat!(
        "image-orientation: from-image; ",
        "image-orientation: 0deg; ",
        "image-orientation: 0.25turn flip; ",
        "image-orientation: flip; ",
        "image-rendering: crisp-edges; ",
        "image-rendering: pixelated; ",
        "object-fit: fill; ",
        "object-fit: contain; ",
        "object-fit: cover; ",
        "object-fit: none; ",
        "object-fit: scale-down",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let values = report
        .syntax()
        .iter()
        .map(|declaration| declaration.known().unwrap().property_value().unwrap())
        .collect::<Vec<_>>();

    for (index, expected) in [
        (0, CssImageOrientation::FromImage),
        (
            1,
            CssImageOrientation::Angle(CssAngleValue::from_literal(
                CssAngleLiteral::try_new("0", CssAngleUnit::Degrees).unwrap(),
            )),
        ),
        (3, CssImageOrientation::Flip(None)),
    ] {
        let CssKnownPropertyValueRef::ImageOrientation(value) = values[index] else {
            panic!("expected image-orientation");
        };
        assert_eq!(value.orientation(), &expected);
    }
    let CssKnownPropertyValueRef::ImageOrientation(flipped) = values[2] else {
        panic!("expected flipped image-orientation");
    };
    assert!(matches!(
        flipped.orientation(),
        CssImageOrientation::Flip(Some(value))
            if value.literal().is_some_and(|angle| angle.numeric().representation() == "0.25" && angle.unit() == CssAngleUnit::Turns)
    ));

    assert!(matches!(
        values[4],
        CssKnownPropertyValueRef::ImageRendering(value)
            if value.rendering() == &CssImageRendering::CrispEdges
    ));
    assert!(matches!(
        values[5],
        CssKnownPropertyValueRef::ImageRendering(value)
            if value.rendering() == &CssImageRendering::Pixelated
    ));
    for (index, expected) in [
        (6, CssObjectFit::Fill),
        (7, CssObjectFit::Contain),
        (8, CssObjectFit::Cover),
        (9, CssObjectFit::None),
        (10, CssObjectFit::ScaleDown),
    ] {
        let CssKnownPropertyValueRef::ObjectFit(value) = values[index] else {
            panic!("expected object-fit");
        };
        assert_eq!(value.fit(), &expected);
    }
}

#[test]
fn border_image_numeric_and_orientation_calculations_remain_symbolic() {
    let report = parse_style_attribute(concat!(
        "border-image-slice: calc(10% + 5%); ",
        "border-image-width: calc(1 + 1); ",
        "border-image-outset: calc(2 + 3); ",
        "image-orientation: calc(45deg + 0.25turn) flip",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let values = report
        .syntax()
        .iter()
        .map(|declaration| declaration.known().unwrap().property_value().unwrap())
        .collect::<Vec<_>>();
    assert!(matches!(
        values[0],
        CssKnownPropertyValueRef::BorderImageSlice(value)
            if matches!(
                &value.slice().values()[0],
                CssBorderImageSliceComponent::Percentage(number) if number.calculation().is_some()
            )
    ));
    assert!(matches!(
        values[1],
        CssKnownPropertyValueRef::BorderImageWidth(value)
            if matches!(
                &value.widths().values()[0],
                CssBorderImageWidthComponent::Number(number) if number.calculation().is_some()
            )
    ));
    assert!(matches!(
        values[2],
        CssKnownPropertyValueRef::BorderImageOutset(value)
            if matches!(
                &value.outsets().values()[0],
                CssBorderImageOutsetComponent::Number(number) if number.calculation().is_some()
            )
    ));
    assert!(matches!(
        values[3],
        CssKnownPropertyValueRef::ImageOrientation(value)
            if matches!(
                value.orientation(),
                CssImageOrientation::Flip(Some(angle)) if angle.calculation().is_some()
            )
    ));
}

#[test]
fn border_image_and_image_properties_keep_symbolic_branches_distinct() {
    for property in [
        "border-image",
        "border-image-outset",
        "border-image-repeat",
        "border-image-slice",
        "border-image-source",
        "border-image-width",
        "image-orientation",
        "image-rendering",
        "object-fit",
    ] {
        let source = format!("{property}: inherit; {property}: var(--value)");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [global, substitution] = report.syntax().as_slice() else {
            panic!("{source}: expected two declarations");
        };
        assert!(matches!(
            global.known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::Global(_)
        ));
        assert!(matches!(
            substitution.known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::SubstitutionDependent(_)
        ));
    }
}

#[test]
fn invalid_border_image_domains_drop_exact_declaration_and_keep_siblings() {
    let cases = [
        ("border-image", "url(frame.png) / 2", "/"),
        ("border-image", "10 // 1 2 3 4 5", "5"),
        ("border-image-slice", "-1", "-1"),
        ("border-image-slice", "1 2 3 4 5", "5"),
        ("border-image-slice", "1 fill fill", "fill"),
        ("border-image-width", "-1", "-1"),
        ("border-image-width", "1 2 3 4 5", "5"),
        ("border-image-outset", "10%", "10%"),
        ("border-image-outset", "-2px", "-2px"),
        ("border-image-repeat", "round space stretch", "stretch"),
        ("border-image-source", "none, url(frame.png)", ","),
        ("image-orientation", "90", "90"),
        ("image-orientation", "flip 90deg", "90deg"),
        ("image-rendering", "smooth", "smooth"),
        ("object-fit", "scale-up", "scale-up"),
    ];

    for (property, invalid, responsible_text) in cases {
        let declaration = format!("{property}: {invalid};");
        let source = format!("--😀: kept; {declaration} color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected one diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}",
        );
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropDeclaration,
            "{source}",
        );
        let declaration_start = source.find(&declaration).unwrap();
        let declaration_end = declaration_start + declaration.len();
        let value_start = declaration_start + property.len() + 2;
        let responsible = if responsible_text == "fill" {
            value_start + invalid.rfind(responsible_text).unwrap()
        } else {
            value_start + invalid.find(responsible_text).unwrap()
        };
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            declaration_start,
            "{source}",
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            declaration_end,
            "{source}",
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible,
            "{source}",
        );
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            source[..responsible].encode_utf16().count(),
            "{source}",
        );
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected property-specific diagnostic");
        };
        assert_eq!(detail.property().canonical_name(), property, "{source}");

        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects invalid image syntax")
                .diagnostics(),
            report.diagnostics(),
            "{source}",
        );
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

fn exact_number(component: Option<&CssComponentValue>, expected: &str) -> bool {
    matches!(component.map(CssComponentValue::view), Some(CssComponentValueRef::Token(CssValueTokenRef::Number(number))) if number.representation() == expected)
}
