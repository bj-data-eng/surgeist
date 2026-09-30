#![forbid(unsafe_code)]
//! Percentage tokens retain exact authored numeric spelling in checked lengths.
//! Bounded scalar consumers retain their directly authored percentage magnitude.

use surgeist_css::*;

fn first_object_position_percent(source: &str) -> String {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::ObjectPosition(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected object-position");
    };
    let CssHorizontalPosition::Offset(offset) = value.position().horizontal() else {
        panic!("expected percentage offset");
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) =
        offset.literal_component().unwrap().view()
    else {
        panic!("expected percentage length");
    };
    number.representation().to_owned()
}

#[test]
fn parsed_gradient_stop_keeps_authored_thirty_percent() {
    let report = parse_style_attribute("background-image: linear-gradient(red 30%, blue)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BackgroundImage(images) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected background-image");
    };
    let CssImageValue::Gradient(CssGradient::Linear(gradient)) = &images.images().images()[0]
    else {
        panic!("expected linear gradient");
    };
    let CssColorStopListItem::Stop(stop) = &gradient.stops().items()[0] else {
        panic!("expected first color stop");
    };
    assert!(exact_percentage(
        stop.position().unwrap().literal_component(),
        "30"
    ));
}

#[test]
fn parsed_generic_position_keeps_authored_thirty_percent_after_trivia() {
    assert_eq!(first_object_position_percent("object-position: 30%"), "30");
    // The token begins after both trivia forms. The numeric spelling, not a
    // preceding comment or whitespace, determines the retained magnitude.
    assert_eq!(
        first_object_position_percent("object-position: /*lead*/ 30%"),
        "30"
    );
}

fn checked_object_position_percent(components: CssComponentValues) -> String {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ObjectPosition),
        components,
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::ObjectPosition(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected checked object-position");
    };
    let CssHorizontalPosition::Offset(offset) = value.position().horizontal() else {
        panic!("expected checked percentage offset");
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) =
        offset.literal_component().unwrap().view()
    else {
        panic!("expected checked percentage length");
    };
    number.representation().to_owned()
}

#[test]
fn checked_parsed_components_keep_authored_thirty_percent() {
    assert_eq!(
        checked_object_position_percent(parse_component_values("30%").unwrap()),
        "30"
    );
}

#[test]
fn checked_programmatic_components_keep_authored_thirty_percent() {
    let components =
        CssComponentValues::try_new(vec![CssComponentValue::try_token("30%").unwrap()]).unwrap();
    assert_eq!(checked_object_position_percent(components), "30");
}

#[test]
fn grid_track_keeps_authored_thirty_percent() {
    let report = parse_style_attribute("grid-template-columns: 30%");
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::GridTemplateColumns(grid) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected grid track");
    };
    let CssGridGeneralTrackComponent::TrackSize(size) =
        &grid.value().general_list().unwrap().components()[0]
    else {
        panic!("expected general track size");
    };
    assert_eq!(
        size.breadth()
            .unwrap()
            .length_percentage()
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "30%"
    );
}

#[test]
fn transform_scale_keeps_authored_thirty_percent() {
    let report = parse_style_attribute("transform: scale3d(1, 30%, 1)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Transform(transform) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected transform");
    };
    let CssTransform::Functions(functions) = transform.value() else {
        panic!("expected transform functions");
    };
    let CssTransformFunction::Scale3d(scale) = &functions.functions()[0] else {
        panic!("expected scale3d");
    };
    assert!(
        matches!(scale.y(), CssTransformScaleComponent::Percentage(number) if exact_percentage(number.literal_component(), "30"))
    );
}

#[test]
fn filter_amount_keeps_authored_thirty_percent() {
    let report = parse_style_attribute("filter: grayscale(30%)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Filter(filter) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected filter");
    };
    let CssFilter::Functions(functions) = filter.value() else {
        panic!("expected filter functions");
    };
    assert!(
        matches!(&functions.functions()[0], CssFilterFunction::Grayscale(CssFilterAmount::Percentage(number)) if exact_percentage(number.literal_component(), "30"))
    );
}

#[test]
fn border_image_slice_keeps_authored_thirty_percent() {
    let report = parse_style_attribute("border-image-slice: 30%");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderImageSlice(slice) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected border-image-slice");
    };
    assert!(
        matches!(&slice.slice().values()[0], CssBorderImageSliceComponent::Percentage(number) if exact_percentage(number.literal_component(), "30"))
    );
}

#[test]
fn keyframe_selector_keeps_authored_thirty_percent() {
    let sheet = parse_sheet("@keyframes fade { 30% { opacity: 1; } }");
    assert!(sheet.is_clean(), "{:?}", sheet.diagnostics());
    let CssRule::Keyframes(keyframes) = &sheet.syntax().rules()[0] else {
        panic!("expected keyframes");
    };
    assert!(
        matches!(keyframes.blocks()[0].selectors().selectors()[0], CssKeyframeSelector::Percent(number) if number.value().value() == 30.0)
    );
}

#[test]
fn already_exact_percentage_controls_remain_valid() {
    for (source, expected) in [
        ("object-position: 25%", "25"),
        ("object-position: .5%", ".5"),
    ] {
        assert_eq!(first_object_position_percent(source), expected, "{source}");
    }
}

#[test]
fn signed_and_exponent_spellings_keep_authored_thirty_percent() {
    for (source, expected) in [
        ("object-position: +30%", "+30"),
        ("object-position: -30%", "-30"),
        ("object-position: 3e1%", "3e1"),
    ] {
        assert_eq!(first_object_position_percent(source), expected, "{source}");
    }
}

#[test]
fn invalid_unitless_negative_and_range_controls_still_recover_at_the_token() {
    for source in [
        "object-position: 1e100; color: red",
        "grid-template-columns: -30%; color: red",
        "filter: grayscale(-30%); color: red",
        "border-image-slice: -30%; color: red",
    ] {
        let report = parse_style_attribute(source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        let error = report.diagnostics()[0].error();
        assert_eq!(error.code(), CssErrorCode::InvalidPropertyValue, "{source}");
        assert_eq!(
            error.position().byte_offset().value(),
            source
                .find(if source.contains("1e100") {
                    "1e100"
                } else {
                    "-30%"
                })
                .unwrap(),
            "{source}"
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
    }

    let source = "@keyframes fade { 101% { opacity: 1; } 25% { opacity: 0; } }";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    let CssRule::Keyframes(keyframes) = &report.syntax().rules()[0] else {
        panic!("expected retained keyframes");
    };
    assert_eq!(keyframes.blocks().len(), 1);
    assert!(
        matches!(keyframes.blocks()[0].selectors().selectors()[0], CssKeyframeSelector::Percent(number) if number.value().value() == 25.0)
    );
}

#[test]
fn huge_finite_decimal_percentage_is_an_exact_authored_position() {
    let source = "object-position: 1e100%";
    assert_eq!(first_object_position_percent(source), "1e100");
    let report = parse_style_attribute(source);
    let CssKnownPropertyValueRef::ObjectPosition(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("position")
    };
    let CssHorizontalPosition::Offset(offset) = wrapper.position().horizontal() else {
        panic!("percentage")
    };
    let CssValueOrigin::Parsed(origin) = offset.origin() else {
        panic!("parsed origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        &source[origin.span().start().byte_offset().value()
            ..origin.span().end().byte_offset().value()],
        "1e100%"
    );
}

fn exact_percentage(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Percentage(number))) if number.representation() == representation)
}
