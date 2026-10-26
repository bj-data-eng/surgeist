//! Functional authored Scrollbars 1 cases for checked models and explicit writers.
//! Expectations follow the pinned 2021-12-09 CR §§2–3 and the selected Color 4/5
//! specified serialization policy; platform scrollbar rendering remains downstream.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::sole_declaration as declaration;
use surgeist_css::*;

fn color(text: &str) -> CssColor {
    let source = declaration(&format!("color:{text}"));
    let CssKnownPropertyValueRef::Color(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed color")
    };
    value.value().clone()
}

fn scrollbar_color(source: &CssDeclaration) -> &CssScrollbarColor {
    let CssKnownPropertyValueRef::ScrollbarColor(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed scrollbar color")
    };
    value.value()
}

#[test]
fn checked_auto_and_exact_pair_keep_distinct_ordered_roles() {
    let auto = CssScrollbarColor::auto();
    assert_eq!(auto.thumb(), None);
    assert_eq!(auto.track(), None);
    assert_eq!(
        auto.serialize_specified(Default::default()).unwrap(),
        "auto"
    );

    let red = CssColor::from_named(CssNamedColor::try_new("RED").unwrap());
    let blue = CssColor::from_named(CssNamedColor::try_new("BLUE").unwrap());
    let pair = CssScrollbarColor::new(red.clone(), blue.clone());
    assert_eq!(pair.thumb(), Some(&red));
    assert_eq!(pair.track(), Some(&blue));
    assert_eq!(
        pair.serialize_specified(Default::default()).unwrap(),
        "red blue"
    );
    assert_ne!(pair, CssScrollbarColor::new(blue, red));
    assert_ne!(auto, pair);

    let same = CssScrollbarColor::new(color("red"), color("red"));
    assert_eq!(
        same.serialize_specified(Default::default()).unwrap(),
        "red red"
    );
}

#[test]
fn parsed_pair_roles_and_canonical_text_preserve_contextual_graphs_and_sources() {
    for (authored, expected) in [
        ("AUTO", "auto"),
        (r"r\65 d BLUE", "red blue"),
        ("currentcolor Canvas", "currentcolor canvas"),
        ("hsl(0 100% 50%) #0000ff", "rgb(255, 0, 0) rgb(0, 0, 255)"),
        (
            "light-dark(red, blue) contrast-color(currentcolor)",
            "light-dark(red, blue) contrast-color(currentcolor)",
        ),
        (
            "alpha(from RGB(300, 0, 0, 120%)) color(--P 0% 70% 20% 0%)",
            "alpha(from rgb(300 0 0 / 120%)) color(--P 0 0.7 0.2 0)",
        ),
    ] {
        let source = declaration(&format!("scrollbar-color:{authored}!important"));
        let original = source.value_components().clone();
        let value = scrollbar_color(&source);
        let snapshot = value.clone();
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected,
            "{authored}"
        );
        assert_eq!(value, &snapshot);
        assert_eq!(source.value_components(), &original);
        assert_eq!(source.importance(), CssImportance::Important);
        assert!(
            original
                .items()
                .iter()
                .all(|item| { !matches!(item.origin(), CssValueOrigin::Programmatic) })
        );
        let checked = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(
                CssKnownProperty::ScrollbarColor,
            )),
            original.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(scrollbar_color(&checked), value);
        assert_eq!(checked.value_components(), &original);
    }

    let source = declaration("scrollbar-color:currentcolor Canvas");
    let value = scrollbar_color(&source);
    assert!(value.thumb().unwrap().is_current_color());
    assert_eq!(
        value.track().unwrap().system(),
        Some(CssSystemColor::Canvas)
    );
    assert_eq!(value.thumb().unwrap().keyword_srgba8(), None);
    assert_eq!(value.track().unwrap().keyword_srgba8(), None);
}

#[test]
fn width_keywords_and_auto_have_exact_root_resource_boundaries() {
    let exact = CssSerializationLimits::new(1, 1, 4);
    for (authored, expected, keyword) in [
        ("AUTO", "auto", CssScrollbarWidth::Auto),
        (r"th\69 n", "thin", CssScrollbarWidth::Thin),
        ("NONE", "none", CssScrollbarWidth::None),
    ] {
        let source = declaration(&format!("scrollbar-width:{authored}"));
        let CssKnownPropertyValueRef::ScrollbarWidth(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed scrollbar width")
        };
        assert_eq!(value.value(), &keyword);
        assert_eq!(
            keyword.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert_eq!(keyword.serialize_specified(exact).unwrap(), expected);
        for (limits, kind) in [
            (
                CssSerializationLimits::new(0, 1, 4),
                SerializationFailure::InputNodeLimit,
            ),
            (
                CssSerializationLimits::new(1, 0, 4),
                SerializationFailure::ProjectionNodeLimit,
            ),
            (
                CssSerializationLimits::new(1, 1, 3),
                SerializationFailure::ByteLimit,
            ),
        ] {
            assert_eq!(
                serialization_failure(&keyword.serialize_specified(limits).unwrap_err()),
                kind
            );
            assert_eq!(
                serialization_failure(
                    &CssScrollbarColor::auto()
                        .serialize_specified(limits)
                        .unwrap_err()
                ),
                kind
            );
        }
    }
    assert_eq!(
        CssScrollbarColor::auto()
            .serialize_specified(exact)
            .unwrap(),
        "auto"
    );
}

#[test]
fn pair_serialization_limits_accumulate_through_the_track_and_fail_atomically() {
    let source = declaration("scrollbar-color:RED blue!important");
    let original = source.value_components().clone();
    let pair = scrollbar_color(&source);
    let snapshot = pair.clone();
    let exact = CssSerializationLimits::new(3, 3, 8);
    assert_eq!(pair.serialize_specified(exact).unwrap(), "red blue");
    for (limits, kind) in [
        (
            CssSerializationLimits::new(2, 3, 8),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 8),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 3, 7),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&pair.serialize_specified(limits).unwrap_err()),
            kind
        );
        assert_eq!(pair, &snapshot);
        assert_eq!(source.value_components(), &original);
        assert_eq!(pair.serialize_specified(exact).unwrap(), "red blue");
    }
}

#[test]
fn nested_track_colors_share_the_whole_pair_node_budget() {
    let pair = CssScrollbarColor::new(
        color("light-dark(red, blue)"),
        color("light-dark(red, blue)"),
    );
    let expected = "light-dark(red, blue) light-dark(red, blue)";
    let exact = CssSerializationLimits::new(7, 7, 43);
    assert_eq!(pair.serialize_specified(exact).unwrap(), expected);
    for (limits, kind) in [
        (
            CssSerializationLimits::new(6, 7, 43),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(7, 6, 43),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(7, 7, 42),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&pair.serialize_specified(limits).unwrap_err()),
            kind
        );
        assert_eq!(
            pair.serialize_specified(Default::default()).unwrap(),
            expected
        );
    }
}

#[test]
fn checked_invalid_track_reports_its_original_mixed_source_token() {
    let thumb = CssComponentValues::parse(&CssParser::default(), "red").unwrap();
    let track = CssComponentValues::parse(&CssParser::default(), "not-a-color").unwrap();
    let original_track = track.items()[0].origin().clone();
    let components = CssComponentValues::try_new(
        vec![thumb.items()[0].clone(), track.items()[0].clone()],
        Default::default(),
    )
    .unwrap();
    let snapshot = components.clone();
    let error = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::ScrollbarColor)),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap_err();
    assert!(error.code().is_some());
    assert_eq!(error.origin(), CssSerializedOrigin::Token(original_track));
    assert_eq!(components, snapshot);
    let valid = CssComponentValues::try_new(
        vec![
            thumb.items()[0].clone(),
            CssComponentValue::try_ident("blue").unwrap(),
        ],
        Default::default(),
    )
    .unwrap();
    let checked = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::ScrollbarColor)),
        valid.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(checked.value_components(), &valid);
    assert_eq!(
        scrollbar_color(&checked)
            .serialize_specified(Default::default())
            .unwrap(),
        "red blue"
    );
}

#[test]
fn selected_scrollbars_properties_expose_complete_dated_support() {
    for (property, id) in [
        (
            CssKnownProperty::ScrollbarWidth,
            "baseline.property.scrollbar-width",
        ),
        (
            CssKnownProperty::ScrollbarColor,
            "official.property.scrollbar-color",
        ),
    ] {
        let support = property_support_metadata(property.canonical_name()).unwrap();
        let feature = support.feature();
        assert_eq!(support.property(), property);
        assert_eq!(feature.id().as_str(), id);
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        assert_eq!(feature.supported_subset(), None);
        assert_eq!(feature.unsupported_remainder(), None);
        assert_eq!(feature.source().id().as_str(), "R-SCROLLBARS1");
        assert_eq!(
            feature.source().url(),
            Some("https://www.w3.org/TR/2021/CR-css-scrollbars-1-20211209/")
        );
        assert!(support.aliases().is_empty());
    }
}
