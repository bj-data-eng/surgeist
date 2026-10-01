#![forbid(unsafe_code)]
//! Functional tests for Backgrounds 3 §2.6 and §2.6.1 and Values 4 §8.3.2.
//! https://www.w3.org/TR/2024/CRD-css-backgrounds-3-20240311/#bg-position-serialization

use surgeist_css::{
    CssHorizontalPosition as H, CssSpecifiedValueSerializationErrorKind as E,
    CssSpecifiedValueSerializationLimits as L, CssVerticalPosition as V, *,
};

fn declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    source.clone()
}

fn authored(source: &CssDeclaration) -> &CssBackgroundPositionList {
    let CssKnownPropertyValueRef::BackgroundPosition(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("background position")
    };
    value.positions()
}

fn offset(css: &str) -> CssSpecifiedLengthPercentage {
    if css.starts_with("calc(") {
        CssSpecifiedLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(
                parse_component_values(css).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        CssSpecifiedLengthPercentage::try_from_component(CssComponentValue::try_token(css).unwrap())
            .unwrap()
    }
}

fn position(horizontal: H, vertical: V) -> CssBackgroundPosition {
    CssBackgroundPosition::try_new(horizontal, vertical).unwrap()
}

fn contributed(item: &CssLonghandContribution) -> &CssBackgroundPositionList {
    let CssLonghandValueRef::BackgroundPosition(value) = item.ordinary_value().unwrap().view()
    else {
        panic!("typed position contribution")
    };
    value
}

#[test]
fn intrinsic_initial_is_one_pair_of_zero_percentages_without_authored_provenance() {
    let CssPropertyKindRef::Longhand(metadata) = CssKnownProperty::BackgroundPosition
        .metadata()
        .unwrap()
        .kind()
    else {
        panic!("longhand metadata")
    };
    let initial = metadata.initial_value();
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::BackgroundPosition
    );
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("intrinsic value")
    };
    let CssLonghandValueRef::BackgroundPosition(positions) = value.view() else {
        panic!("position initial")
    };
    let expected = CssBackgroundPositionList::try_new(vec![position(
        H::Offset(offset("0%")),
        V::Offset(offset("0%")),
    )])
    .unwrap();
    assert_eq!(positions, &expected);
    assert_eq!(positions.serialize_specified().unwrap(), "0% 0%");
    let [position] = positions.positions() else {
        panic!("one initial layer")
    };
    let (H::Offset(x), V::Offset(y)) = (position.horizontal(), position.vertical()) else {
        panic!("two free percentage offsets")
    };
    for axis in [x, y] {
        assert!(matches!(axis.origin(), CssValueOrigin::Programmatic));
        assert!(matches!(axis.literal_component().unwrap().view(),
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
            if number.representation() == "0"));
    }
}

#[test]
fn ordinary_contributions_preserve_exact_edge_axes_signed_values_and_layer_order() {
    let source = declaration("background-position:bottom -20% right -10px, left 5px top!important");
    let expected = CssBackgroundPositionList::try_new(vec![
        position(
            H::RightOffset(offset("-10px")),
            V::BottomOffset(offset("-20%")),
        ),
        position(H::LeftOffset(offset("5px")), V::Top),
    ])
    .unwrap();
    assert_eq!(authored(&source), &expected);
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("completed position")
    };
    let [item] = items.items() else {
        panic!("one terminal contribution")
    };
    assert_eq!(contributed(item), &expected);
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    let [first, _] = contributed(item).positions() else {
        panic!("two layers")
    };
    let H::RightOffset(x) = first.horizontal() else {
        panic!("right offset")
    };
    assert!(matches!(x.origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(
        contributed(item).serialize_specified().unwrap(),
        "right -10px bottom -20%, left 5px top"
    );
}

#[test]
fn reentry_preserves_checked_programmatic_axes_and_symbolic_calculation() {
    let source = declaration("background-position:var(--position)!important");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("pending position")
    };
    let replacement = CssComponentValues::try_new(vec![
        CssComponentValue::try_token("-5%").unwrap(),
        CssComponentValue::try_ident("top").unwrap(),
    ])
    .unwrap();
    let expected =
        CssBackgroundPositionList::try_new(vec![position(H::Offset(offset("-5%")), V::Top)])
            .unwrap();
    for _ in 0..2 {
        let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed replacement")
        };
        let [item] = items.items() else {
            panic!("one terminal")
        };
        assert_eq!(contributed(item), &expected);
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
        let H::Offset(x) = contributed(item).positions()[0].horizontal() else {
            panic!("free horizontal offset")
        };
        assert!(matches!(x.origin(), CssValueOrigin::Programmatic));
    }
    let replacement = parse_component_values("calc(2px + 3%) center").unwrap();
    let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("symbolic replacement")
    };
    let value = contributed(&items.items()[0]);
    let H::Offset(x) = value.positions()[0].horizontal() else {
        panic!("symbolic axis")
    };
    assert!(x.calculation().is_some());
    assert!(x.literal_component().is_none());
    assert!(matches!(value.positions()[0].vertical(), V::Center));
    assert_eq!(
        value.serialize_specified().unwrap(),
        "calc(3% + 2px) center"
    );
    assert_eq!(
        items.items()[0].replacement_components(),
        Some(&replacement)
    );
}

#[test]
fn specified_positions_follow_horizontal_first_order_and_keep_omitted_offsets() {
    for (css, expected) in [
        ("top", "center top"),
        ("left", "left center"),
        ("center", "center center"),
        ("25%", "25% center"),
        ("bottom right", "right bottom"),
        ("25% 75%", "25% 75%"),
        ("bottom 20% left", "left bottom 20%"),
        ("top right 10px", "right 10px top"),
        ("bottom 20% right 10px", "right 10px bottom 20%"),
        ("center bottom 0px", "center bottom 0px"),
        ("right 0px top", "right 0px top"),
        ("-10% calc(2px + 3%)", "-10% calc(3% + 2px)"),
    ] {
        let source = declaration(&format!("background-position:{css}"));
        let list = authored(&source);
        let [value] = list.positions() else {
            panic!("one position")
        };
        assert_eq!(value.serialize_specified().unwrap(), expected, "{css}");
        assert_eq!(list.serialize_specified().unwrap(), expected, "{css}");
        let roundtrip = declaration(&format!("background-position:{expected}"));
        assert_eq!(
            authored(&roundtrip).serialize_specified().unwrap(),
            expected,
            "{css}"
        );
        // Existing typed calculation equality retains authored expression structure;
        // canonical numeric output may reorder its terms without changing meaning.
        if !css.contains("calc(") {
            assert_eq!(authored(&roundtrip), list, "{css}");
        } else {
            assert!(matches!(
                authored(&roundtrip).positions()[0].horizontal(),
                H::Offset(_)
            ));
            assert!(
                matches!(authored(&roundtrip).positions()[0].vertical(), V::Offset(value)
                if value.calculation().is_some() && value.literal_component().is_none())
            );
        }
    }
}

#[test]
fn checked_list_serialization_retains_domain_distinctions_and_comma_order() {
    let three = position(H::RightOffset(offset("10px")), V::Top);
    let expected = CssBackgroundPositionList::try_new(vec![
        three.clone(),
        position(H::Left, V::BottomOffset(offset("-20%"))),
    ])
    .unwrap();
    assert_eq!(
        expected.serialize_specified().unwrap(),
        "right 10px top, left bottom -20%"
    );
    assert_eq!(three.serialize_specified().unwrap(), "right 10px top");
    assert!(CssPhysicalPosition::try_new(H::RightOffset(offset("10px")), V::Top).is_err());
    assert!(
        CssBackgroundPosition::try_new(H::RightOffset(offset("10px")), V::Offset(offset("2px")))
            .is_none()
    );
    assert!(CssBackgroundPositionList::try_new(Vec::new()).is_none());
}

#[test]
fn scalar_serialization_charges_axes_numeric_children_and_output_atomically() {
    for (value, expected, nodes) in [
        (position(H::Left, V::Top), "left top", 3),
        (
            position(H::RightOffset(offset("10px")), V::Top),
            "right 10px top",
            4,
        ),
        (
            position(H::Offset(offset("0%")), V::Offset(offset("0%"))),
            "0% 0%",
            5,
        ),
    ] {
        let original = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(nodes, nodes, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, error) in [
            (L::new(nodes - 1, nodes, expected.len()), E::InputNodeLimit),
            (
                L::new(nodes, nodes - 1, expected.len()),
                E::ProjectionNodeLimit,
            ),
            (L::new(nodes, nodes, expected.len() - 1), E::ByteLimit),
            (L::new(0, nodes, expected.len()), E::InputNodeLimit),
            (L::new(nodes, 0, expected.len()), E::ProjectionNodeLimit),
            (L::new(nodes, nodes, 0), E::ByteLimit),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                error
            );
            assert_eq!(value, original);
        }
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn list_serialization_uses_one_budget_for_all_layers_and_separators() {
    let list = CssBackgroundPositionList::try_new(vec![
        position(H::Left, V::Top),
        position(H::Right, V::Bottom),
    ])
    .unwrap();
    let expected = "left top, right bottom";
    // One list node plus two positions, each with its two axis nodes.
    assert_eq!(
        list.serialize_specified_with_limits(L::new(7, 7, expected.len()))
            .unwrap(),
        expected
    );
    let original = list.clone();
    for (limits, error) in [
        (L::new(6, 7, expected.len()), E::InputNodeLimit),
        (L::new(7, 6, expected.len()), E::ProjectionNodeLimit),
        (L::new(7, 7, expected.len() - 1), E::ByteLimit),
        (L::new(4, 4, expected.len()), E::InputNodeLimit),
    ] {
        assert_eq!(
            list.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            error
        );
        assert_eq!(list, original);
    }
    assert_eq!(list.serialize_specified().unwrap(), expected);
}

#[test]
fn calculation_projection_work_is_cumulative_across_axes_and_layers() {
    let math = || offset("calc(1px + 2em)");
    let scalar = position(H::Offset(math()), V::Offset(math()));
    let scalar_css = "calc(2em + 1px) calc(2em + 1px)";
    assert_eq!(
        scalar
            .serialize_specified_with_limits(L::new(11, 13, scalar_css.len()))
            .unwrap(),
        scalar_css
    );
    for (limits, error) in [
        (L::new(10, 13, scalar_css.len()), E::InputNodeLimit),
        (L::new(11, 12, scalar_css.len()), E::ProjectionNodeLimit),
        (L::new(11, 13, scalar_css.len() - 1), E::ByteLimit),
    ] {
        assert_eq!(
            scalar
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            error
        );
    }
    let list = CssBackgroundPositionList::try_new(vec![
        position(H::Offset(math()), V::Center),
        position(H::Offset(math()), V::Top),
    ])
    .unwrap();
    let expected = "calc(2em + 1px) center, calc(2em + 1px) top";
    assert_eq!(
        list.serialize_specified_with_limits(L::new(15, 17, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, error) in [
        (L::new(14, 17, expected.len()), E::InputNodeLimit),
        (L::new(15, 16, expected.len()), E::ProjectionNodeLimit),
        (L::new(15, 17, expected.len() - 1), E::ByteLimit),
    ] {
        assert_eq!(
            list.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            error
        );
    }
    assert_eq!(list.serialize_specified().unwrap(), expected);
}

#[test]
fn universal_reset_keeps_background_position_with_the_existing_explicit_exclusions() {
    for (css, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("all:{css}!important"));
        let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("one symbolic universal reset")
        };
        assert_eq!(reset.keyword(), keyword);
        assert!(reset.source().same_occurrence(&source));
        assert_eq!(reset.source().importance(), CssImportance::Important);
        assert!(reset.replacement_components().is_none());
        for property in [
            CssKnownProperty::BackgroundPosition,
            CssKnownProperty::BackgroundImage,
            CssKnownProperty::Color,
        ] {
            assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
        }
        for property in [CssKnownProperty::Direction, CssKnownProperty::UnicodeBidi] {
            assert!(reset.excludes(CssPropertyNameRef::Known(property)));
        }
        let name = CssCustomPropertyName::try_new("--position").unwrap();
        assert!(reset.excludes(CssPropertyNameRef::Custom(&name)));
    }
}
