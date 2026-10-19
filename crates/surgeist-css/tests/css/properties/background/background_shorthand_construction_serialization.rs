#![forbid(unsafe_code)]
//! Backgrounds 3 CRD20240311 §2.10 and CSSOM WD20210826 §6.7.2.
//! Canonical examples include the Backgrounds 3 subset of pinned WebKit WPT
//! background-shorthand-serialization.html; Backgrounds 4 text/border-area excluded.

use surgeist_css::*;

fn background(text: &str) -> CssBackground {
    let report = parse_style_attribute(&format!("background: {text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Background(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("ordinary background")
    };
    value.background().clone()
}

fn layer(color: Option<CssColor>) -> CssBackgroundLayer {
    CssBackgroundLayer::try_new(None, None, None, None, None, None, color).unwrap()
}

#[test]
fn construction_rejects_empty_layers_uncoupled_size_and_nonfinal_color() {
    assert_eq!(
        CssBackgroundLayer::try_new(None, None, None, None, None, None, None),
        Err(CssBackgroundConstructionError::EmptyLayer)
    );
    assert_eq!(
        CssBackgroundLayer::try_new(
            None,
            None,
            Some(CssBackgroundSize::Cover),
            None,
            None,
            None,
            None
        ),
        Err(CssBackgroundConstructionError::SizeWithoutPosition)
    );
    assert_eq!(
        CssBackground::try_new(vec![]),
        Err(CssBackgroundConstructionError::EmptyLayers)
    );
    let first = CssBackgroundLayer::try_new(
        Some(CssImageValue::None),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        CssBackground::try_new(vec![
            first.clone(),
            layer(Some(CssColor::transparent())),
            first
        ]),
        Err(CssBackgroundConstructionError::NonFinalColor { layer_index: 1 })
    );
    let value = CssBackground::try_new(vec![layer(Some(CssColor::current_color()))]).unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "currentcolor");
    assert_eq!(
        CssBackgroundConstructionError::SizeWithoutPosition.to_string(),
        "background size requires a position"
    );
    let error: &dyn std::error::Error = &CssBackgroundConstructionError::EmptyLayers;
    assert!(error.source().is_none());
}

#[test]
fn construction_matches_parsing_without_erasing_omissions_boxes_or_child_origins() {
    let parsed = background(
        "url(pic) left 1em top / cover no-repeat fixed content-box padding-box currentcolor",
    );
    let source = &parsed.layers()[0];
    let constructed = CssBackgroundLayer::try_new(
        source.image().cloned(),
        source.position().cloned(),
        source.size().cloned(),
        source.repeat(),
        source.attachment(),
        source.boxes(),
        source.color().cloned(),
    )
    .unwrap();
    let CssHorizontalPosition::LeftOffset(offset) = constructed.position().unwrap().horizontal()
    else {
        panic!("copied authored edge offset")
    };
    let CssValueOrigin::Parsed(origin) = offset.origin() else {
        panic!("copied child retains parsed provenance")
    };
    assert_eq!(
        origin.source().as_str(),
        "background: url(pic) left 1em top / cover no-repeat fixed content-box padding-box currentcolor"
    );
    let span = origin.span();
    assert_eq!(
        &origin.source().as_str()
            [span.start().byte_offset().value()..span.end().byte_offset().value()],
        "1em"
    );
    assert_eq!(constructed, *source);
    assert_eq!(CssBackground::try_new(vec![constructed]).unwrap(), parsed);
    let color_only = layer(Some(CssColor::transparent()));
    assert!(color_only.image().is_none());
    assert!(color_only.position().is_none());
    assert!(color_only.size().is_none());
    assert!(color_only.boxes().is_none());
    assert!(matches!(
        background("padding-box").layers()[0].boxes(),
        Some(CssBackgroundLayerBoxes::One(_))
    ));
    assert!(matches!(
        background("padding-box padding-box").layers()[0].boxes(),
        Some(CssBackgroundLayerBoxes::OriginAndClip { .. })
    ));
}

#[test]
fn direct_typed_construction_retains_components_omissions_and_programmatic_origin() {
    let offset = CssSpecifiedLengthPercentage::try_from_component(
        CssComponentValue::try_dimension("42.50", "px").unwrap(),
    )
    .unwrap();
    let position = CssBackgroundPosition::try_new(
        CssHorizontalPosition::LeftOffset(offset),
        CssVerticalPosition::Top,
    )
    .unwrap();
    let boxes = CssBackgroundLayerBoxes::OriginAndClip {
        origin: CssBackgroundBox::ContentBox,
        clip: CssBackgroundBox::PaddingBox,
    };
    let repeat = CssBackgroundRepeat::Axes {
        x: CssBackgroundRepeatStyle::Space,
        y: CssBackgroundRepeatStyle::Round,
    };
    let first = CssBackgroundLayer::try_new(
        None,
        Some(position),
        Some(CssBackgroundSize::Cover),
        Some(repeat),
        Some(CssBackgroundAttachment::Local),
        Some(boxes),
        None,
    )
    .unwrap();
    let final_layer = CssBackgroundLayer::try_new(
        None,
        None,
        None,
        None,
        None,
        None,
        Some(CssColor::current_color()),
    )
    .unwrap();
    let value = CssBackground::try_new(vec![first, final_layer]).unwrap();
    assert_eq!(value.layers().len(), 2);
    let first = &value.layers()[0];
    assert!(first.image().is_none());
    assert!(first.color().is_none());
    assert_eq!(first.size(), Some(&CssBackgroundSize::Cover));
    assert_eq!(first.repeat(), Some(repeat));
    assert_eq!(first.attachment(), Some(CssBackgroundAttachment::Local));
    assert_eq!(first.boxes(), Some(boxes));
    let position = first.position().unwrap();
    assert_eq!(position.vertical(), &CssVerticalPosition::Top);
    let CssHorizontalPosition::LeftOffset(offset) = position.horizontal() else {
        panic!("checked edge offset retained")
    };
    assert_eq!(offset.origin(), &CssValueOrigin::Programmatic);
    let component = offset.literal_component().unwrap();
    assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
        component.view()
    else {
        panic!("exact checked dimension retained")
    };
    assert_eq!(number.representation(), "42.50");
    assert_eq!(unit, "px");
    let final_layer = &value.layers()[1];
    assert!(final_layer.color().unwrap().is_current_color());
    assert!(final_layer.image().is_none());
    assert!(final_layer.position().is_none());
    assert!(final_layer.size().is_none());
    assert!(final_layer.repeat().is_none());
    assert!(final_layer.attachment().is_none());
    assert!(final_layer.boxes().is_none());
}

fn terminal_text(value: CssLonghandValueRef<'_>) -> String {
    match value {
        CssLonghandValueRef::BackgroundImage(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundPosition(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundSize(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundRepeat(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundAttachment(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BackgroundOrigin(v) | CssLonghandValueRef::BackgroundClip(v) => {
            v.serialize_specified().unwrap()
        }
        CssLonghandValueRef::BackgroundColor(v) => v.to_specified_css().unwrap(),
        CssLonghandValueRef::BackgroundBlendMode(v) => {
            assert_eq!(v.modes(), &[CssBlendMode::Normal]);
            v.serialize_specified().unwrap()
        }
        _ => panic!("background terminal"),
    }
}

fn expanded(text: &str) -> Vec<(CssKnownProperty, String)> {
    let report = parse_style_attribute(&format!("background: {text}"));
    assert!(report.is_clean());
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&report.syntax()[0]).unwrap()
    else {
        panic!("completed background")
    };
    values
        .items()
        .iter()
        .map(|v| {
            (
                v.property(),
                terminal_text(v.ordinary_value().unwrap().view()),
            )
        })
        .collect()
}

#[test]
fn canonical_grammar_omits_initials_preserves_context_and_round_trips_expansion() {
    // Expectations are explicit grammar and selected WPT examples, not serializer output.
    for (authored, expected) in [
        ("yellow", "yellow"),
        (
            "no-repeat url(/favicon.ico)",
            "url(\"/favicon.ico\") no-repeat",
        ),
        (
            "url(pic) no-repeat, url(pic) no-repeat",
            "url(\"pic\") no-repeat, url(\"pic\") no-repeat",
        ),
        ("url(pic) 0% 0% / 10rem", "url(\"pic\") 0% 0% / 10rem auto"),
        (
            "url(pic) top left no-repeat, url(pic) center / 100% 100% no-repeat, url(pic) white",
            "url(\"pic\") left top no-repeat, url(\"pic\") center center / 100% 100% no-repeat, url(\"pic\") white",
        ),
        ("padding-box border-box", "none"),
        (
            "none 0% 0% / auto auto repeat repeat scroll padding-box border-box transparent",
            "none",
        ),
        ("padding-box padding-box", "padding-box"),
        ("content-box border-box", "content-box border-box"),
        ("none, transparent", "none, none"),
        ("left top", "left top"),
        (
            "0% 0% / cover fixed content-box padding-box currentcolor",
            "0% 0% / cover fixed content-box padding-box currentcolor",
        ),
        (
            "calc(1em + 2px) 0% / calc(2em + 3px) no-repeat currentcolor",
            "calc(1em + 2px) 0% / calc(2em + 3px) auto no-repeat currentcolor",
        ),
        (
            "linear-gradient(currentcolor, red) local",
            "linear-gradient(currentcolor, red) local",
        ),
    ] {
        let original = background(authored);
        let unchanged = original.clone();
        let actual = original.serialize_specified().unwrap();
        assert_eq!(actual, expected, "{authored}");
        assert_eq!(original, unchanged);
        assert_eq!(expanded(authored), expanded(expected), "{authored}");
        assert_eq!(
            background(expected).serialize_specified().unwrap(),
            expected
        );
    }
}

fn assert_limits(text: &str, input: usize, projection: usize, expected: &str) {
    let value = background(text);
    let original = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                input,
                projection,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(input - 1, projection, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(input, projection - 1, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(input, projection, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind,
            "{text}"
        );
        assert_eq!(value, original);
    }
}

#[test]
fn omitted_nodes_consume_visits_but_only_generated_none_consumes_four_bytes() {
    assert_limits("none", 3, 4, "none");
    assert_limits("transparent", 3, 4, "none");
    assert_limits("padding-box border-box transparent", 5, 6, "none");
    assert_limits(
        "none 0% 0% / auto auto repeat repeat scroll padding-box border-box transparent",
        18,
        19,
        "none",
    );
    // The tiny nonzero percentage must never be omitted through a machine-float zero.
    assert_eq!(
        background("1e-400% 0%")
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(100, 100, 4))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    // Exact signed/unit-bearing zero is an initial; discarded text need not fit four bytes.
    assert_limits("-0.0000000000000000000000em 0%", 7, 8, "none");
}

#[test]
fn implicit_size_height_is_projection_only_and_all_layers_share_separator_bytes() {
    assert_limits("0% 0% / 10px", 10, 11, "0% 0% / 10px auto");
    assert_limits(
        "none, padding-box border-box transparent",
        7,
        9,
        "none, none",
    );
    assert_limits("padding-box padding-box", 4, 4, "padding-box");
    assert_limits(
        "scroll no-repeat, fixed currentcolor",
        9,
        9,
        "no-repeat, fixed currentcolor",
    );
    assert_limits(
        "none, none 0% 0% / auto repeat scroll padding-box border-box currentcolor",
        19,
        20,
        "none, currentcolor",
    );
}
