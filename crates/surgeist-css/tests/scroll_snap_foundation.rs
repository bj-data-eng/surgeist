#![forbid(unsafe_code)]

//! CSS Scroll Snap 1 (2021-03-11) property definitions and Appendix A, with
//! Logical 1 (2025-12-04) §4.7's authored `logical` four-side syntax.
//! Four-side shorthand reset membership is unsettled; these tests do not claim
//! that an ordinary four-side value has a completed intrinsic expansion.

use surgeist_css::*;

const TERMINALS: &[&str] = &[
    "scroll-snap-type",
    "scroll-snap-align",
    "scroll-snap-stop",
    "scroll-padding-top",
    "scroll-padding-right",
    "scroll-padding-bottom",
    "scroll-padding-left",
    "scroll-padding-block-start",
    "scroll-padding-block-end",
    "scroll-padding-inline-start",
    "scroll-padding-inline-end",
    "scroll-margin-top",
    "scroll-margin-right",
    "scroll-margin-bottom",
    "scroll-margin-left",
    "scroll-margin-block-start",
    "scroll-margin-block-end",
    "scroll-margin-inline-start",
    "scroll-margin-inline-end",
];

const AXIS_SHORTHANDS: &[(&str, [&str; 2])] = &[
    (
        "scroll-padding-block",
        ["scroll-padding-block-start", "scroll-padding-block-end"],
    ),
    (
        "scroll-padding-inline",
        ["scroll-padding-inline-start", "scroll-padding-inline-end"],
    ),
    (
        "scroll-margin-block",
        ["scroll-margin-block-start", "scroll-margin-block-end"],
    ),
    (
        "scroll-margin-inline",
        ["scroll-margin-inline-start", "scroll-margin-inline-end"],
    ),
];

const ALL: &[(&str, &str)] = &[
    ("scroll-snap-type", "x mandatory"),
    ("scroll-snap-align", "start end"),
    ("scroll-snap-stop", "always"),
    ("scroll-padding-top", "auto"),
    ("scroll-padding-right", "1px"),
    ("scroll-padding-bottom", "5%"),
    ("scroll-padding-left", "calc(1px + 2%)"),
    ("scroll-padding-block-start", "auto"),
    ("scroll-padding-block-end", "1em"),
    ("scroll-padding-inline-start", "2%"),
    ("scroll-padding-inline-end", "3px"),
    ("scroll-padding-block", "auto 2px"),
    ("scroll-padding-inline", "3% auto"),
    ("scroll-padding", "1px 2% 3px auto"),
    ("scroll-margin-top", "-1px"),
    ("scroll-margin-right", "2px"),
    ("scroll-margin-bottom", "0"),
    ("scroll-margin-left", "calc(1px + 2px)"),
    ("scroll-margin-block-start", "-1em"),
    ("scroll-margin-block-end", "2px"),
    ("scroll-margin-inline-start", "3px"),
    ("scroll-margin-inline-end", "4px"),
    ("scroll-margin-block", "-1px 2px"),
    ("scroll-margin-inline", "3px -4px"),
    ("scroll-margin", "1px 2px 3px 4px"),
];

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("selected grammar: {name}"))
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one selected declaration: {source}")
    };
    declaration.clone()
}

fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

#[test]
fn every_selected_scroll_snap_name_has_known_identity_and_accepts_its_ordinary_grammar() {
    assert_eq!(ALL.len(), 25);
    for &(name, value) in ALL {
        let identity = grammar(name);
        assert_eq!(identity.name(), name);
        assert_eq!(identity.target_property().canonical_name(), name);
        assert_eq!(
            identity.feature_id().as_str(),
            format!("official.property.{name}")
        );
        let support = property_support_metadata(name).unwrap();
        assert_eq!(support.property(), identity.target_property());
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(support.feature().source().id().as_str(), "R-SCROLLSNAP1");
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2021/CR-css-scroll-snap-1-20210311/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));

        let parsed = declaration(name, value);
        assert_eq!(
            parsed.known().unwrap().property(),
            identity.target_property()
        );
        assert_eq!(parsed.importance(), CssImportance::Important);
        assert_eq!(
            parsed.value_components().serialize().unwrap().as_css(),
            value
        );

        let constructed = checked(name, value);
        assert_eq!(
            constructed.known().unwrap().property(),
            identity.target_property()
        );
        assert_eq!(constructed.importance(), CssImportance::Important);
        assert!(constructed.parsed_value().is_none());
        assert_eq!(
            constructed.value_components().serialize().unwrap().as_css(),
            value
        );
    }
}

#[test]
fn every_selected_scroll_snap_name_accepts_whole_value_css_wide_keywords() {
    for &(name, _) in ALL {
        for keyword in ["initial", "inherit", "unset", "revert", "revert-layer"] {
            let parsed = declaration(name, keyword);
            let constructed = checked(name, keyword);
            assert_eq!(
                parsed.known().unwrap().property(),
                grammar(name).target_property()
            );
            assert_eq!(
                constructed.known().unwrap().property(),
                grammar(name).target_property()
            );
        }
    }
}

#[test]
fn snap_type_requires_an_axis_before_optional_strictness_and_align_has_two_roles() {
    for value in [
        "none",
        "x",
        "y",
        "block",
        "inline",
        "both",
        "x mandatory",
        "y proximity",
        "block mandatory",
        "inline proximity",
        "both mandatory",
    ] {
        declaration("scroll-snap-type", value);
        checked("scroll-snap-type", value);
    }
    for value in ["none", "start", "end", "center", "none center", "start end"] {
        declaration("scroll-snap-align", value);
        checked("scroll-snap-align", value);
    }
    for value in ["normal", "always"] {
        declaration("scroll-snap-stop", value);
        checked("scroll-snap-stop", value);
    }
    for (name, value) in [
        ("scroll-snap-type", "mandatory"),
        ("scroll-snap-type", "proximity"),
        ("scroll-snap-type", "mandatory x"),
        ("scroll-snap-type", "x mandatory proximity"),
        ("scroll-snap-type", "x x"),
        ("scroll-snap-type", "none mandatory"),
        ("scroll-snap-align", "start end center"),
        ("scroll-snap-align", "mandatory"),
        ("scroll-snap-stop", "none"),
    ] {
        assert_invalid(name, value);
    }
}

#[test]
fn scroll_insets_keep_length_and_length_percentage_domains_distinct() {
    for (name, value) in [
        ("scroll-margin-top", "-1e999px"),
        ("scroll-margin-inline-end", "1e-999px"),
        ("scroll-margin-left", "calc(2px + 3px)"),
        ("scroll-padding-top", "1e999px"),
        ("scroll-padding-inline-end", "1e-999px"),
        ("scroll-padding-bottom", "1e-999%"),
        ("scroll-padding-left", "calc(2px + 3%)"),
    ] {
        declaration(name, value);
        checked(name, value);
    }
    for (name, value) in [
        ("scroll-margin-top", "5%"),
        ("scroll-margin-inline-end", "auto"),
        ("scroll-margin-bottom", "calc(2px + 3%)"),
        ("scroll-padding-top", "-1px"),
        ("scroll-padding-block-end", "-1e-999px"),
        ("scroll-padding-inline-start", "-1e-999%"),
        ("scroll-padding-bottom", "2fr"),
        ("scroll-padding-left", "calc(2 + 3)"),
    ] {
        assert_invalid(name, value);
    }
}

#[test]
fn physical_and_logical_shorthands_admit_their_defined_arities_and_marker() {
    for prefix in ["scroll-padding", "scroll-margin"] {
        let values = if prefix == "scroll-padding" {
            ["auto", "1px auto", "1px 2% 3px", "1px 2px 3px 4px"]
        } else {
            ["0", "1px -2px", "1px 2px 3px", "1px 2px 3px 4px"]
        };
        for value in values {
            declaration(prefix, value);
            checked(prefix, value);
            let logical = format!("logical {value}");
            declaration(prefix, &logical);
            checked(prefix, &logical);
        }
        for axis in ["block", "inline"] {
            let name = format!("{prefix}-{axis}");
            for value in if prefix == "scroll-padding" {
                ["auto", "1px 2%"]
            } else {
                ["0", "-1px 2px"]
            } {
                declaration(&name, value);
                checked(&name, value);
            }
        }
    }
    for (name, value) in [
        ("scroll-padding", "1px 2px 3px 4px 5px"),
        ("scroll-padding", "logical 1px 2px 3px 4px 5px"),
        ("scroll-padding", "-1px"),
        ("scroll-padding", "logical -1px"),
        ("scroll-margin", "1px 2px 3px 4px 5px"),
        ("scroll-margin", "logical 1px 2px 3px 4px 5px"),
        ("scroll-margin", "1%"),
        ("scroll-padding-block", "1px 2px 3px"),
        ("scroll-margin-inline", "1px 2px 3px"),
    ] {
        assert_invalid(name, value);
    }
}

fn assert_invalid(name: &str, value: &str) {
    let source = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid value diagnostic: {source}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(report.syntax().len(), 2, "{source}");
    assert!(validate_style_attribute(&source).is_err());
    assert!(
        parse_property_value_for_grammar(
            grammar(name),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .is_err(),
        "checked construction accepted {name}:{value}"
    );
}

#[test]
fn stable_terminals_have_noninherited_fixed_initials_and_one_contribution() {
    assert_eq!(TERMINALS.len(), 19);
    for &name in TERMINALS {
        let property = grammar(name).target_property();
        assert_eq!(
            property_support_metadata(name).unwrap().feature().status(),
            CssSupportStatus::Complete
        );
        let metadata = grammar(name).metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("{name} is terminal")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        assert!(matches!(initial.view(), CssInitialValueRef::Value(_)));

        let value = ALL
            .iter()
            .find(|(candidate, _)| *candidate == name)
            .unwrap()
            .1;
        let source = declaration(name, value);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("one terminal expansion for {name}")
        };
        let [item] = values.items() else {
            panic!("one contribution for {name}")
        };
        assert_eq!(item.property(), property);
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            property
        );
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
}

#[test]
fn axis_shorthands_expose_exact_logical_member_order_and_expand_to_two() {
    assert_eq!(AXIS_SHORTHANDS.len(), 4);
    for &(name, members) in AXIS_SHORTHANDS {
        assert_eq!(
            property_support_metadata(name).unwrap().feature().status(),
            CssSupportStatus::Complete
        );
        let metadata = grammar(name).metadata().unwrap();
        let CssPropertyKindRef::Shorthand(shorthand) = metadata.kind() else {
            panic!("{name} is an axis shorthand")
        };
        assert!(!shorthand.is_legacy());
        assert!(shorthand.reset_only_members().is_empty());
        assert_eq!(
            shorthand
                .settable_members()
                .iter()
                .map(|member| member.known_property().canonical_name())
                .collect::<Vec<_>>(),
            members
        );
        let value = ALL
            .iter()
            .find(|(candidate, _)| *candidate == name)
            .unwrap()
            .1;
        let source = declaration(name, value);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("two axis contributions for {name}")
        };
        assert_eq!(
            values
                .items()
                .iter()
                .map(|item| item.property().canonical_name())
                .collect::<Vec<_>>(),
            members
        );
        for item in values.items() {
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
        }
    }
}

#[test]
fn stable_globals_and_pending_reentry_keep_original_occurrence_and_strict_grammar() {
    for (name, replacement) in [
        ("scroll-snap-type", "both mandatory"),
        ("scroll-padding-top", "auto"),
        ("scroll-padding-block", "1px auto"),
        ("scroll-margin-left", "-2px"),
        ("scroll-margin-inline", "1px -2px"),
    ] {
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("symbolic CSS-wide value for {name}")
            };
            assert!(!values.items().is_empty());
            for item in values.items() {
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
            }
        }
        let source = declaration(name, "var(--snap)");
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending substitution for {name}")
        };
        assert!(handle.source().same_occurrence(&source));
        let invalid = if name.contains("margin") {
            "1%"
        } else {
            "auto auto auto"
        };
        assert!(matches!(
            handle
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        assert_eq!(
            handle
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        let components = parse_component_values(replacement).unwrap();
        let CssContributions::Longhands(values) = handle.reenter(components.clone()).unwrap()
        else {
            panic!("completed replacement for {name}")
        };
        for item in values.items() {
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.replacement_components(), Some(&components));
        }
    }
}

fn length(text: &str) -> CssSpecifiedLength {
    let components = parse_component_values(text).unwrap();
    let [component] = components.items() else {
        panic!("one length: {text}")
    };
    CssSpecifiedLength::try_from_component(component.clone()).unwrap()
}

fn padding(text: &str) -> CssScrollPaddingValue {
    if text == "auto" {
        CssScrollPaddingValue::Auto
    } else {
        let components = parse_component_values(text).unwrap();
        let [component] = components.items() else {
            panic!("one padding value: {text}")
        };
        CssScrollPaddingValue::LengthPercentage(
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(component.clone()).unwrap(),
        )
    }
}

#[test]
fn snap_models_retain_optional_components_and_serialize_selected_keywords() {
    let source = declaration("scroll-snap-type", "BlOcK PrOxImItY");
    let CssKnownPropertyValueRef::ScrollSnapType(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed snap type")
    };
    assert_eq!(
        *value.value(),
        CssScrollSnapType::axis(
            CssScrollSnapAxis::Block,
            Some(CssScrollSnapStrictness::Proximity)
        )
    );
    assert_eq!(value.as_css(), "BlOcK PrOxImItY");
    assert_eq!(value.value().serialize_specified().unwrap(), "block");
    assert_eq!(
        CssScrollSnapType::axis(CssScrollSnapAxis::X, None)
            .serialize_specified()
            .unwrap(),
        "x"
    );
    assert_eq!(
        CssScrollSnapType::axis(
            CssScrollSnapAxis::Y,
            Some(CssScrollSnapStrictness::Mandatory)
        )
        .serialize_specified()
        .unwrap(),
        "y mandatory"
    );
    assert_eq!(
        CssScrollSnapType::None.serialize_specified().unwrap(),
        "none"
    );

    let source = declaration("scroll-snap-align", "start start");
    let CssKnownPropertyValueRef::ScrollSnapAlign(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed snap alignment")
    };
    assert_eq!(value.value().block(), CssScrollSnapAlignment::Start);
    assert_eq!(
        value.value().authored_inline(),
        Some(CssScrollSnapAlignment::Start)
    );
    assert_eq!(value.value().inline(), CssScrollSnapAlignment::Start);
    assert_eq!(value.value().serialize_specified().unwrap(), "start");
    let one = CssScrollSnapAlign::new(CssScrollSnapAlignment::End, None);
    assert_eq!(one.authored_inline(), None);
    assert_eq!(one.inline(), CssScrollSnapAlignment::End);
    assert_eq!(one.serialize_specified().unwrap(), "end");
    assert_eq!(
        CssScrollSnapStop::Always.serialize_specified().unwrap(),
        "always"
    );

    let components =
        CssComponentValues::try_new(vec![CssComponentValue::try_token("-2px").unwrap()]).unwrap();
    assert!(matches!(
        components.items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    let constructed = parse_property_value_for_grammar(
        grammar("scroll-margin-top"),
        components.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    assert!(constructed.parsed_value().is_none());
    assert_eq!(constructed.value_components(), &components);
    let CssKnownPropertyValueRef::ScrollMarginTop(value) =
        constructed.known().unwrap().property_value().unwrap()
    else {
        panic!("typed scroll margin top")
    };
    assert!(matches!(
        value.value().origin(),
        CssValueOrigin::Programmatic
    ));
    assert_eq!(value.value().serialize_specified().unwrap(), "-2px");
}

#[test]
fn pair_and_four_side_models_keep_authored_cardinality_and_role_order() {
    let pair = CssScrollPaddingPair::new(padding("auto"), None);
    assert!(pair.authored_end().is_none());
    assert_eq!(pair.end(), pair.start());
    assert_eq!(pair.serialize_specified().unwrap(), "auto");
    let margin = CssScrollMarginPair::new(length("1px"), Some(length("2px")));
    assert_eq!(margin.start().serialize_specified().unwrap(), "1px");
    assert_eq!(
        margin
            .authored_end()
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "2px"
    );
    assert_eq!(margin.end().serialize_specified().unwrap(), "2px");

    let two = CssScrollMarginShorthand::try_new(
        CssScrollSideKind::Logical,
        vec![length("1px"), length("2px")],
    )
    .unwrap();
    assert_eq!(two.kind(), CssScrollSideKind::Logical);
    assert_eq!(two.authored_values().len(), 2);
    for (role, expected) in ["1px", "2px", "1px", "2px"].into_iter().enumerate() {
        assert_eq!(
            two.role(role).unwrap().serialize_specified().unwrap(),
            expected
        );
    }
    assert!(two.role(4).is_none());
    let three = CssScrollPaddingShorthand::try_new(
        CssScrollSideKind::Physical,
        vec![padding("1px"), padding("2px"), padding("3px")],
    )
    .unwrap();
    assert_eq!(three.authored_values().len(), 3);
    for (role, expected) in ["1px", "2px", "3px", "2px"].into_iter().enumerate() {
        assert_eq!(
            three.role(role).unwrap().serialize_specified().unwrap(),
            expected
        );
    }
    assert!(CssScrollMarginShorthand::try_new(CssScrollSideKind::Physical, vec![]).is_none());
    assert!(
        CssScrollPaddingShorthand::try_new(CssScrollSideKind::Logical, vec![padding("auto"); 5])
            .is_none()
    );
}

#[test]
fn canonical_pair_and_quad_compression_compares_values_across_distinct_origins() {
    let parsed = length("1px");
    let programmatic =
        CssSpecifiedLength::try_from_component(CssComponentValue::try_token("1px").unwrap())
            .unwrap();
    assert_ne!(parsed, programmatic, "origins are retained in owned values");
    let pair = CssScrollMarginPair::new(parsed.clone(), Some(programmatic.clone()));
    assert_eq!(pair.authored_end(), Some(&programmatic));
    assert_eq!(pair.serialize_specified().unwrap(), "1px");

    for (values, expected) in [
        (vec!["1px", "1px", "1px", "1px"], "1px"),
        (vec!["1px", "2px", "1px", "2px"], "1px 2px"),
        (vec!["1px", "2px", "3px", "2px"], "1px 2px 3px"),
        (vec!["1px", "2px", "3px", "4px"], "1px 2px 3px 4px"),
    ] {
        let authored = values.into_iter().map(length).collect();
        let quad =
            CssScrollMarginShorthand::try_new(CssScrollSideKind::Physical, authored).unwrap();
        assert_eq!(quad.serialize_specified().unwrap(), expected);
    }
    let logical = CssScrollMarginShorthand::try_new(
        CssScrollSideKind::Logical,
        vec![
            parsed.clone(),
            length("2px"),
            programmatic.clone(),
            length("2px"),
        ],
    )
    .unwrap();
    assert_eq!(logical.serialize_specified().unwrap(), "logical 1px 2px");
    let padding = CssScrollPaddingShorthand::try_new(
        CssScrollSideKind::Physical,
        vec![
            padding("auto"),
            padding("5%"),
            padding("auto"),
            padding("5%"),
        ],
    )
    .unwrap();
    assert_eq!(padding.serialize_specified().unwrap(), "auto 5%");
}

#[test]
fn pair_and_quad_serialization_charge_each_authored_child_under_one_budget() {
    let pair = CssScrollMarginPair::new(length("1px"), Some(length("2px")));
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 2, 7))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 1, 7))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 6))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 7))
            .unwrap(),
        "1px 2px"
    );
    let compressed_pair = CssScrollMarginPair::new(length("1px"), Some(length("1px")));
    assert_eq!(
        compressed_pair
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 3))
            .unwrap(),
        "1px"
    );
    let quad = CssScrollMarginShorthand::try_new(
        CssScrollSideKind::Logical,
        vec![length("1px"), length("1px"), length("1px"), length("1px")],
    )
    .unwrap();
    assert_eq!(quad.serialize_specified().unwrap(), "logical 1px");
    assert_eq!(
        quad.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(4, 5, 64))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        quad.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(5, 4, 64))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        quad.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(5, 5, 11))
            .unwrap(),
        "logical 1px"
    );
    assert_eq!(
        quad.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(5, 5, 10))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}

#[test]
fn math_pair_captures_both_authored_trees_with_one_input_budget() {
    // Each authored calc() has a function, sum, and two numeric leaves, so
    // each tree costs four input nodes even if it simplifies to one length.
    let math = |text| {
        CssSpecifiedLength::try_from_calculation(
            CssLengthCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap(),
        )
        .unwrap()
    };
    let first = math("calc(1px + 2px)");
    let second = math("calc(3px + 4px)");
    let ample_projection_and_bytes = CssSpecifiedValueSerializationLimits::new(4, 64, 64);
    first
        .serialize_specified_with_limits(ample_projection_and_bytes)
        .unwrap();
    second
        .serialize_specified_with_limits(ample_projection_and_bytes)
        .unwrap();
    let pair = CssScrollMarginPair::new(first, Some(second));
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(7, 64, 64))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(8, 64, 64))
            .is_ok()
    );
}

#[test]
fn four_side_authored_values_report_exact_unresolved_expansion_footprint() {
    for (name, valid, invalid) in [
        ("scroll-padding", "logical auto 2px", "-1px"),
        ("scroll-margin", "logical 1px -2px", "1%"),
    ] {
        let grammar = grammar(name);
        assert_eq!(
            property_support_metadata(name).unwrap().feature().status(),
            CssSupportStatus::Complete
        );
        assert_eq!(
            grammar.metadata().unwrap_err(),
            CssPropertyMetadataError::UnresolvedStandard {
                grammar,
                reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
            }
        );
        for value in [valid, "initial", "inherit", "unset"] {
            let source = declaration(name, value);
            assert_eq!(
                expand_declaration(&source).unwrap_err().kind(),
                &CssExpansionErrorKind::UnresolvedStandard {
                    property: grammar.target_property(),
                    reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
                }
            );
        }
        let source = declaration(name, "var(--inset)");
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("four-side substitution remains pending")
        };
        assert!(handle.source().same_occurrence(&source));
        assert!(matches!(
            handle
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        assert_eq!(
            handle
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        let replacement = parse_component_values(valid).unwrap();
        assert_eq!(
            handle.reenter(replacement.clone()).unwrap_err().kind(),
            &CssExpansionErrorKind::UnresolvedStandard {
                property: grammar.target_property(),
                reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
            }
        );

        let pending_report = parse_sheet(&format!(".a{{{name}:var(--inset)}}"));
        assert!(pending_report.is_clean());
        let pending_normalized = normalize_sheet(pending_report.syntax()).unwrap();
        let pending_item = pending_normalized
            .items()
            .iter()
            .find_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .expect("pending four-side declaration retained");
        assert!(matches!(pending_item.expansion(), CssExpansion::Pending(_)));

        let report = parse_sheet(&format!(".a{{color:red;{name}:{valid};color:blue}}"));
        assert!(report.is_clean(), "{name}: {:?}", report.diagnostics());
        let syntax = report.syntax().clone();
        let error = normalize_sheet(report.syntax()).unwrap_err();
        assert!(
            matches!(error.kind(), CssNormalizationErrorKind::UnsupportedDeclaration(expansion)
                if expansion.kind() == &CssExpansionErrorKind::UnresolvedStandard {
                    property: grammar.target_property(),
                    reason: CssUnresolvedStandard::LogicalShorthandResetMembership,
                }
            )
        );
        assert_eq!(
            error.declaration().unwrap().known().unwrap().property(),
            grammar.target_property()
        );
        assert_eq!(error.declaration_order(), Some(1));
        assert_eq!(
            error
                .declaration()
                .unwrap()
                .value_components()
                .serialize()
                .unwrap()
                .as_css(),
            valid
        );
        assert_eq!(
            report.syntax(),
            &syntax,
            "normalization preserves authored sheet"
        );
    }
}
