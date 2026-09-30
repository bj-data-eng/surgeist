#![forbid(unsafe_code)]

//! Functional authored-value construction and bounded specified serialization
//! for CSS Box Alignment 3, 2026-01-30, §§4–7.
//! https://www.w3.org/TR/2026/WD-css-align-3-20260130/

use surgeist_css::*;

fn normal(overflow: Option<CssOverflowPosition>) -> CssAlignmentValue {
    CssAlignmentValue::Normal { overflow }
}

fn position(
    overflow: Option<CssOverflowPosition>,
    position: CssAlignmentPosition,
) -> CssAlignmentValue {
    CssAlignmentValue::Position { overflow, position }
}

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

#[test]
fn content_constructors_reject_foreign_positions_and_justify_baseline() {
    for value in [
        normal(None),
        CssAlignmentValue::Stretch,
        CssAlignmentValue::Baseline(CssBaselinePosition::First),
        position(Some(CssOverflowPosition::Safe), CssAlignmentPosition::Start),
        CssAlignmentValue::SpaceEvenly,
    ] {
        assert_eq!(CssAlignContentValue::try_new(value).unwrap().value(), value);
    }
    for value in [
        CssAlignmentValue::Auto,
        normal(Some(CssOverflowPosition::Safe)),
        position(None, CssAlignmentPosition::Left),
        position(None, CssAlignmentPosition::SelfStart),
        CssAlignmentValue::Legacy(None),
    ] {
        assert!(CssAlignContentValue::try_new(value).is_none(), "{value:?}");
    }

    for value in [
        normal(None),
        CssAlignmentValue::Stretch,
        position(None, CssAlignmentPosition::Right),
        position(
            Some(CssOverflowPosition::Unsafe),
            CssAlignmentPosition::Left,
        ),
        CssAlignmentValue::SpaceBetween,
    ] {
        assert_eq!(
            CssJustifyContentValue::try_new(value).unwrap().value(),
            value
        );
    }
    for value in [
        CssAlignmentValue::Auto,
        CssAlignmentValue::Baseline(CssBaselinePosition::Baseline),
        normal(Some(CssOverflowPosition::Unsafe)),
        position(None, CssAlignmentPosition::SelfEnd),
        CssAlignmentValue::Legacy(None),
    ] {
        assert!(
            CssJustifyContentValue::try_new(value).is_none(),
            "{value:?}"
        );
    }
}

#[test]
fn items_and_self_constructors_keep_auto_normal_and_legacy_in_their_domains() {
    for value in [
        normal(None),
        CssAlignmentValue::Stretch,
        CssAlignmentValue::Baseline(CssBaselinePosition::Last),
        position(
            Some(CssOverflowPosition::Unsafe),
            CssAlignmentPosition::SelfStart,
        ),
    ] {
        assert_eq!(CssAlignItemsValue::try_new(value).unwrap().value(), value);
        assert_eq!(CssJustifyItemsValue::try_new(value).unwrap().value(), value);
    }
    for value in [
        CssAlignmentValue::Auto,
        normal(Some(CssOverflowPosition::Safe)),
        CssAlignmentValue::SpaceAround,
    ] {
        assert!(CssAlignItemsValue::try_new(value).is_none());
        assert!(CssJustifyItemsValue::try_new(value).is_none());
    }
    let left = position(None, CssAlignmentPosition::Left);
    assert!(CssAlignItemsValue::try_new(left).is_none());
    assert_eq!(CssJustifyItemsValue::try_new(left).unwrap().value(), left);
    for legacy in [
        CssAlignmentValue::Legacy(None),
        CssAlignmentValue::Legacy(Some(CssLegacyAlignment::Left)),
        CssAlignmentValue::Legacy(Some(CssLegacyAlignment::Right)),
        CssAlignmentValue::Legacy(Some(CssLegacyAlignment::Center)),
    ] {
        assert!(CssAlignItemsValue::try_new(legacy).is_none());
        assert_eq!(
            CssJustifyItemsValue::try_new(legacy).unwrap().value(),
            legacy
        );
    }

    for value in [
        CssAlignmentValue::Auto,
        normal(None),
        normal(Some(CssOverflowPosition::Unsafe)),
        CssAlignmentValue::Stretch,
        CssAlignmentValue::Baseline(CssBaselinePosition::Baseline),
        position(
            Some(CssOverflowPosition::Safe),
            CssAlignmentPosition::SelfEnd,
        ),
    ] {
        assert_eq!(CssAlignSelfValue::try_new(value).unwrap().value(), value);
        assert_eq!(CssJustifySelfValue::try_new(value).unwrap().value(), value);
    }
    for value in [
        CssAlignmentValue::Legacy(None),
        CssAlignmentValue::SpaceEvenly,
    ] {
        assert!(CssAlignSelfValue::try_new(value).is_none());
        assert!(CssJustifySelfValue::try_new(value).is_none());
    }
    assert!(CssAlignSelfValue::try_new(left).is_none());
    assert_eq!(CssJustifySelfValue::try_new(left).unwrap().value(), left);
}

#[test]
fn typed_longhands_serialize_selected_keywords_in_canonical_order() {
    let align_content =
        CssAlignContentValue::try_new(CssAlignmentValue::Baseline(CssBaselinePosition::Last))
            .unwrap();
    assert_eq!(
        align_content.serialize_specified().unwrap(),
        "last baseline"
    );
    let first =
        CssAlignContentValue::try_new(CssAlignmentValue::Baseline(CssBaselinePosition::First))
            .unwrap();
    assert_eq!(first.serialize_specified().unwrap(), "baseline");
    let justify_content = CssJustifyContentValue::try_new(position(
        Some(CssOverflowPosition::Unsafe),
        CssAlignmentPosition::Right,
    ))
    .unwrap();
    assert_eq!(
        justify_content.serialize_specified().unwrap(),
        "unsafe right"
    );
    let align_items = CssAlignItemsValue::try_new(position(
        Some(CssOverflowPosition::Safe),
        CssAlignmentPosition::SelfStart,
    ))
    .unwrap();
    assert_eq!(
        align_items.serialize_specified().unwrap(),
        "safe self-start"
    );
    let justify_items =
        CssJustifyItemsValue::try_new(CssAlignmentValue::Legacy(Some(CssLegacyAlignment::Center)))
            .unwrap();
    assert_eq!(
        justify_items.serialize_specified().unwrap(),
        "legacy center"
    );
    let legacy_right =
        CssJustifyItemsValue::try_new(CssAlignmentValue::Legacy(Some(CssLegacyAlignment::Right)))
            .unwrap();
    assert_eq!(legacy_right.serialize_specified().unwrap(), "legacy right");
    let align_self = CssAlignSelfValue::try_new(normal(Some(CssOverflowPosition::Safe))).unwrap();
    assert_eq!(align_self.serialize_specified().unwrap(), "safe normal");
    let justify_self = CssJustifySelfValue::try_new(CssAlignmentValue::Auto).unwrap();
    assert_eq!(justify_self.serialize_specified().unwrap(), "auto");
}

#[test]
fn shorthand_construction_and_omission_default_values_work_without_parsing() {
    let baseline =
        CssAlignContentValue::try_new(CssAlignmentValue::Baseline(CssBaselinePosition::Baseline))
            .unwrap();
    let content = CssPlaceContentValue::from_align(baseline);
    assert_eq!(content.align(), baseline);
    assert_eq!(
        content.justify().value(),
        position(None, CssAlignmentPosition::Start)
    );
    assert_eq!(content.serialize_specified().unwrap(), "baseline start");
    let right =
        CssJustifyContentValue::try_new(position(None, CssAlignmentPosition::Right)).unwrap();
    let explicit = CssPlaceContentValue::new(baseline, right);
    assert_eq!(explicit.justify(), right);
    assert_eq!(explicit.serialize_specified().unwrap(), "baseline right");

    let stretch = CssAlignItemsValue::try_new(CssAlignmentValue::Stretch).unwrap();
    let items = CssPlaceItemsValue::from_align(stretch);
    assert_eq!(items.align(), stretch);
    assert_eq!(items.justify().value(), CssAlignmentValue::Stretch);
    assert_eq!(items.serialize_specified().unwrap(), "stretch stretch");
    let legacy = CssJustifyItemsValue::try_new(CssAlignmentValue::Legacy(None)).unwrap();
    let explicit = CssPlaceItemsValue::new(stretch, legacy);
    assert_eq!(explicit.justify(), legacy);
    assert_eq!(explicit.serialize_specified().unwrap(), "stretch legacy");

    let auto = CssAlignSelfValue::try_new(CssAlignmentValue::Auto).unwrap();
    let self_pair = CssPlaceSelfValue::from_align(auto);
    assert_eq!(self_pair.align(), auto);
    assert_eq!(self_pair.justify().value(), CssAlignmentValue::Auto);
    assert_eq!(self_pair.serialize_specified().unwrap(), "auto auto");
    let right = CssJustifySelfValue::try_new(position(None, CssAlignmentPosition::Right)).unwrap();
    let explicit = CssPlaceSelfValue::new(auto, right);
    assert_eq!(explicit.justify(), right);
    assert_eq!(explicit.serialize_specified().unwrap(), "auto right");
}

#[test]
fn parsed_wrappers_expose_authored_alignment_values() {
    let source = declaration("align-content: CENTER !important");
    let CssKnownPropertyValueRef::AlignContent(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("align-content wrapper")
    };
    assert_eq!(value.as_css(), "CENTER");
    assert_eq!(
        value.value().value(),
        position(None, CssAlignmentPosition::Center)
    );

    let source = declaration("justify-content: right");
    let CssKnownPropertyValueRef::JustifyContent(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("justify-content wrapper")
    };
    assert_eq!(
        value.value().value(),
        position(None, CssAlignmentPosition::Right)
    );

    let source = declaration("align-items: self-start");
    let CssKnownPropertyValueRef::AlignItems(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("align-items wrapper")
    };
    assert_eq!(
        value.value().value(),
        position(None, CssAlignmentPosition::SelfStart)
    );

    let source = declaration("align-self: auto");
    let CssKnownPropertyValueRef::AlignSelf(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("align-self wrapper")
    };
    assert_eq!(value.value().value(), CssAlignmentValue::Auto);

    let source = declaration("justify-items: legacy left");
    let CssKnownPropertyValueRef::JustifyItems(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("justify-items wrapper")
    };
    assert_eq!(
        value.value().value(),
        CssAlignmentValue::Legacy(Some(CssLegacyAlignment::Left))
    );

    let source = declaration("justify-self: unsafe normal");
    let CssKnownPropertyValueRef::JustifySelf(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("justify-self wrapper")
    };
    assert_eq!(
        value.value().value(),
        normal(Some(CssOverflowPosition::Unsafe))
    );
}

#[test]
fn parsed_shorthands_keep_authored_css_and_canonical_specified_pairs() {
    let source = declaration("place-content: BASELINE !important");
    let CssKnownPropertyValueRef::PlaceContent(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("place-content wrapper")
    };
    assert_eq!(value.as_css(), "BASELINE");
    assert_eq!(
        value.value().serialize_specified().unwrap(),
        "baseline start"
    );
    let canonical = value.value().serialize_specified().unwrap();
    let reparsed = declaration(&format!("place-content:{canonical}"));
    let CssKnownPropertyValueRef::PlaceContent(reparsed) =
        reparsed.known().unwrap().property_value().unwrap()
    else {
        panic!("reparsed place-content")
    };
    assert_eq!(reparsed.value(), value.value());

    let source = declaration("place-items: normal legacy");
    let CssKnownPropertyValueRef::PlaceItems(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("place-items wrapper")
    };
    assert_eq!(
        value.value().serialize_specified().unwrap(),
        "normal legacy"
    );

    let source = declaration("place-self: auto right");
    let CssKnownPropertyValueRef::PlaceSelf(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("place-self wrapper")
    };
    assert_eq!(value.value().serialize_specified().unwrap(), "auto right");

    let source = declaration("place-content: center end");
    let CssKnownPropertyValueRef::PlaceContent(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("place-content pair")
    };
    assert_eq!(
        value.value().align().value(),
        position(None, CssAlignmentPosition::Center)
    );
    assert_eq!(
        value.value().justify().value(),
        position(None, CssAlignmentPosition::End)
    );
}

#[test]
fn place_items_component_boundary_keeps_legacy_on_justify_axis() {
    let center = CssAlignItemsValue::try_new(position(None, CssAlignmentPosition::Center)).unwrap();
    let legacy_center =
        CssJustifyItemsValue::try_new(CssAlignmentValue::Legacy(Some(CssLegacyAlignment::Center)))
            .unwrap();
    let constructed = CssPlaceItemsValue::new(center, legacy_center);
    assert_eq!(constructed.align(), center);
    assert_eq!(constructed.justify(), legacy_center);
    assert_eq!(
        constructed.serialize_specified().unwrap(),
        "center legacy center"
    );

    for (authored, expected_justify, canonical) in [
        (
            "center legacy",
            CssAlignmentValue::Legacy(None),
            "center legacy",
        ),
        (
            "center legacy left",
            CssAlignmentValue::Legacy(Some(CssLegacyAlignment::Left)),
            "center legacy left",
        ),
    ] {
        let source = declaration(&format!("place-items:{authored}"));
        let CssKnownPropertyValueRef::PlaceItems(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("place-items: {authored}")
        };
        assert_eq!(
            value.value().align().value(),
            position(None, CssAlignmentPosition::Center)
        );
        assert_eq!(value.value().justify().value(), expected_justify);
        assert_eq!(value.value().serialize_specified().unwrap(), canonical);
    }
    let source = declaration("justify-items:center legacy");
    let CssKnownPropertyValueRef::JustifyItems(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("standalone unordered legacy")
    };
    assert_eq!(
        value.value().value(),
        CssAlignmentValue::Legacy(Some(CssLegacyAlignment::Center))
    );
    assert_eq!(
        value.value().serialize_specified().unwrap(),
        "legacy center"
    );
}

#[test]
fn first_baseline_keeps_its_authored_value_despite_canonical_baseline() {
    let source = declaration("align-content:first baseline");
    let CssKnownPropertyValueRef::AlignContent(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("first-baseline wrapper")
    };
    assert_eq!(
        value.value().value(),
        CssAlignmentValue::Baseline(CssBaselinePosition::First)
    );
    assert_eq!(value.value().serialize_specified().unwrap(), "baseline");
}

#[test]
fn adjacent_baseline_components_serialize_without_swapping_their_axes() {
    for source in [
        "place-items:baseline last baseline",
        "place-self:baseline last baseline",
    ] {
        let parsed = declaration(source);
        match parsed.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::PlaceItems(value) => {
                assert_eq!(
                    value.value().align().value(),
                    CssAlignmentValue::Baseline(CssBaselinePosition::Baseline)
                );
                assert_eq!(
                    value.value().justify().value(),
                    CssAlignmentValue::Baseline(CssBaselinePosition::Last)
                );
            }
            CssKnownPropertyValueRef::PlaceSelf(value) => {
                assert_eq!(
                    value.value().align().value(),
                    CssAlignmentValue::Baseline(CssBaselinePosition::Baseline)
                );
                assert_eq!(
                    value.value().justify().value(),
                    CssAlignmentValue::Baseline(CssBaselinePosition::Last)
                );
            }
            _ => panic!("one of the two self-alignment shorthands"),
        }
    }

    let source = declaration("place-items:baseline last");
    let CssKnownPropertyValueRef::PlaceItems(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("place-items omitted second baseline")
    };
    assert_eq!(
        value.value().align().value(),
        CssAlignmentValue::Baseline(CssBaselinePosition::Last)
    );
    assert_eq!(
        value.value().justify().value(),
        CssAlignmentValue::Baseline(CssBaselinePosition::Last)
    );
    let source = declaration("place-self:baseline last");
    let CssKnownPropertyValueRef::PlaceSelf(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("place-self omitted second baseline")
    };
    assert_eq!(
        value.value().align().value(),
        CssAlignmentValue::Baseline(CssBaselinePosition::Last)
    );
    assert_eq!(
        value.value().justify().value(),
        CssAlignmentValue::Baseline(CssBaselinePosition::Last)
    );
    let source = declaration("place-content:baseline last");
    let CssKnownPropertyValueRef::PlaceContent(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("place-content omitted justify baseline")
    };
    assert_eq!(
        value.value().align().value(),
        CssAlignmentValue::Baseline(CssBaselinePosition::Last)
    );
    assert_eq!(
        value.value().justify().value(),
        position(None, CssAlignmentPosition::Start)
    );

    for first in [CssBaselinePosition::Baseline, CssBaselinePosition::First] {
        let align_items = CssAlignItemsValue::try_new(CssAlignmentValue::Baseline(first)).unwrap();
        let justify_items =
            CssJustifyItemsValue::try_new(CssAlignmentValue::Baseline(CssBaselinePosition::Last))
                .unwrap();
        let pair = CssPlaceItemsValue::new(align_items, justify_items);
        assert_eq!(pair.align().value(), CssAlignmentValue::Baseline(first));
        assert_eq!(
            pair.justify().value(),
            CssAlignmentValue::Baseline(CssBaselinePosition::Last)
        );
        let serialized = pair.serialize_specified().unwrap();
        assert_eq!(serialized, "baseline last baseline");
        let source = declaration(&format!("place-items:{serialized}"));
        let CssKnownPropertyValueRef::PlaceItems(parsed) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("reparsed place-items baseline pair")
        };
        assert_eq!(
            parsed.value().align().value(),
            CssAlignmentValue::Baseline(CssBaselinePosition::Baseline)
        );
        assert_eq!(
            parsed.value().justify().value(),
            CssAlignmentValue::Baseline(CssBaselinePosition::Last)
        );

        let align_self = CssAlignSelfValue::try_new(CssAlignmentValue::Baseline(first)).unwrap();
        let justify_self =
            CssJustifySelfValue::try_new(CssAlignmentValue::Baseline(CssBaselinePosition::Last))
                .unwrap();
        let pair = CssPlaceSelfValue::new(align_self, justify_self);
        assert_eq!(pair.align().value(), CssAlignmentValue::Baseline(first));
        assert_eq!(
            pair.justify().value(),
            CssAlignmentValue::Baseline(CssBaselinePosition::Last)
        );
        let serialized = pair.serialize_specified().unwrap();
        assert_eq!(serialized, "baseline last baseline");
        let source = declaration(&format!("place-self:{serialized}"));
        let CssKnownPropertyValueRef::PlaceSelf(parsed) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("reparsed place-self baseline pair")
        };
        assert_eq!(
            parsed.value().align().value(),
            CssAlignmentValue::Baseline(CssBaselinePosition::Baseline)
        );
        assert_eq!(
            parsed.value().justify().value(),
            CssAlignmentValue::Baseline(CssBaselinePosition::Last)
        );
    }
}

#[test]
fn serializers_enforce_exact_keyword_and_cumulative_pair_limits() {
    let single = CssAlignItemsValue::try_new(position(
        Some(CssOverflowPosition::Safe),
        CssAlignmentPosition::Center,
    ))
    .unwrap();
    assert_eq!(
        single
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 11))
            .unwrap(),
        "safe center"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(1, 2, 11),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(2, 1, 11),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(2, 2, 10),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            single
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }

    let align = CssAlignContentValue::try_new(position(
        Some(CssOverflowPosition::Safe),
        CssAlignmentPosition::Center,
    ))
    .unwrap();
    let justify = CssJustifyContentValue::try_new(position(
        Some(CssOverflowPosition::Unsafe),
        CssAlignmentPosition::Right,
    ))
    .unwrap();
    let pair = CssPlaceContentValue::new(align, justify);
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(4, 4, 24))
            .unwrap(),
        "safe center unsafe right"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(3, 4, 24),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 3, 24),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 4, 23),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            pair.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}
