#![forbid(unsafe_code)]

use surgeist_css::*;

fn parsed_color(text: &str) -> CssColor {
    let source = format!("border-top-color:{text}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderTopColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("border-top-color has a checked color")
    };
    value.value().clone()
}

fn error_kind(
    result: Result<String, CssSpecifiedValueSerializationError>,
) -> CssSpecifiedValueSerializationErrorKind {
    result.unwrap_err().kind()
}

#[test]
fn logical_color_pairs_preserve_omission_exact_values_and_cumulative_limits() {
    let first = parsed_color("currentcolor");
    let second = parsed_color("transparent");
    let one = CssBorderColorPair::new(first.clone(), None);
    assert!(one.start().is_current_color());
    assert!(one.authored_end().is_none());
    assert_eq!(one.end(), one.start());
    assert_eq!(one.serialize_specified().unwrap(), "currentcolor");
    assert_ne!(
        one,
        CssBorderColorPair::new(first.clone(), Some(first.clone()))
    );

    let pair = CssBorderColorPair::new(first, Some(second.clone()));
    assert_eq!(pair.authored_end(), Some(&second));
    assert!(pair.end().is_transparent());
    assert_eq!(
        pair.serialize_specified().unwrap(),
        "currentcolor transparent"
    );
    let exact = CssSpecifiedValueSerializationLimits::new(3, 3, 24);
    assert_eq!(
        pair.serialize_specified_with_limits(exact).unwrap(),
        "currentcolor transparent"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, 24),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, 24),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, 23),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            error_kind(pair.serialize_specified_with_limits(limits)),
            kind
        );
    }
}

#[test]
fn four_side_colors_keep_authored_arity_and_independent_role_assignments() {
    let colors = ["red", "blue", "transparent", "currentcolor"].map(parsed_color);
    assert!(CssBorderColorShorthand::try_new(CssBoxSideKind::Physical, vec![]).is_none());
    assert!(
        CssBorderColorShorthand::try_new(CssBoxSideKind::Logical, vec![colors[0].clone(); 5])
            .is_none()
    );
    for (count, expected_roles, specified) in [
        (1, ["red", "red", "red", "red"], "red"),
        (2, ["red", "blue", "red", "blue"], "red blue"),
        (
            3,
            ["red", "blue", "transparent", "blue"],
            "red blue transparent",
        ),
        (
            4,
            ["red", "blue", "transparent", "currentcolor"],
            "red blue transparent currentcolor",
        ),
    ] {
        for kind in [CssBoxSideKind::Physical, CssBoxSideKind::Logical] {
            let shorthand =
                CssBorderColorShorthand::try_new(kind, colors[..count].to_vec()).unwrap();
            assert_eq!(shorthand.kind(), kind);
            assert_eq!(shorthand.authored_values(), &colors[..count]);
            assert_eq!(
                shorthand
                    .assigned_values()
                    .map(|color| color.to_specified_css().unwrap()),
                expected_roles.map(str::to_owned)
            );
            let expected = match kind {
                CssBoxSideKind::Physical => specified.to_owned(),
                CssBoxSideKind::Logical => format!("logical {specified}"),
            };
            assert_eq!(shorthand.serialize_specified().unwrap(), expected);
        }
    }
    let one = CssBorderColorShorthand::try_new(CssBoxSideKind::Physical, vec![colors[0].clone()])
        .unwrap();
    let repeated =
        CssBorderColorShorthand::try_new(CssBoxSideKind::Physical, vec![colors[0].clone(); 4])
            .unwrap();
    let logical =
        CssBorderColorShorthand::try_new(CssBoxSideKind::Logical, vec![colors[0].clone()]).unwrap();
    assert_ne!(one, repeated);
    assert_ne!(one, logical);
}

#[test]
fn logical_four_color_budget_charges_only_authored_colors_and_prefix_bytes() {
    let colors = ["red", "blue", "transparent", "currentcolor"].map(parsed_color);
    let shorthand =
        CssBorderColorShorthand::try_new(CssBoxSideKind::Logical, colors.to_vec()).unwrap();
    let expected = "logical red blue transparent currentcolor";
    let exact = CssSpecifiedValueSerializationLimits::new(5, 5, 41);
    assert_eq!(
        shorthand.serialize_specified_with_limits(exact).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(4, 5, 41),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 4, 41),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 5, 40),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            error_kind(shorthand.serialize_specified_with_limits(limits)),
            kind
        );
    }
    let one = CssBorderColorShorthand::try_new(
        CssBoxSideKind::Physical,
        vec![parsed_color("currentcolor")],
    )
    .unwrap();
    assert_eq!(
        one.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 12))
            .unwrap(),
        "currentcolor"
    );
}

#[test]
fn parsed_current_agrees_with_checked_aggregates_without_losing_color_graphs_or_source() {
    let start = "rgb(1 2 3 / none)";
    let end = "lab(calc(50% + 10%) 20 -30 / 120%)";
    let source = format!("border-block-color:{start} {end}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = &report.syntax()[0];
    let CssKnownPropertyValueRef::BorderBlockColor(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed border-block-color")
    };
    let rgb = value.value().start().rgb_value().unwrap();
    assert!(matches!(rgb.alpha(), Some(CssColorComponent::None)));
    let lab = value.value().end().lab_value().unwrap();
    assert!(matches!(
        lab.lightness(),
        CssColorComponent::PercentageCalculation(_)
    ));
    assert_eq!(declaration.importance(), CssImportance::Important);
    let CssValueOrigin::Parsed(origin) = declaration.value_components().items()[0].origin() else {
        panic!("parsed first color origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        "border-block-color:".len()
    );

    let report = parse_style_attribute("border-inline-color:red #12abef");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderInlineColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed border-inline-color")
    };
    assert_eq!(
        value.value(),
        &CssBorderColorPair::new(parsed_color("red"), Some(parsed_color("#12abef")))
    );

    let source = "border-color:logical red blue transparent currentcolor";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed border-color")
    };
    let colors = ["red", "blue", "transparent", "currentcolor"].map(parsed_color);
    assert_eq!(
        value.value(),
        &CssBorderColorShorthand::try_new(CssBoxSideKind::Logical, colors.to_vec()).unwrap()
    );
}

#[test]
fn constructed_aggregates_retain_one_checked_symbolic_graph_and_its_source() {
    let source = "alpha(from red / none)";
    let color = parsed_color(source);
    assert!(color.alpha_value().is_some());
    let pair = CssBorderColorPair::new(color.clone(), None);
    assert_eq!(pair.start(), &color);
    assert_eq!(pair.end(), &color);
    assert_eq!(pair.serialize_specified().unwrap(), source);

    let shorthand = CssBorderColorShorthand::try_new(
        CssBoxSideKind::Logical,
        vec![color.clone(), parsed_color("blue")],
    )
    .unwrap();
    assert_eq!(shorthand.authored_values()[0], color);
    let assigned = shorthand.assigned_values();
    assert_eq!(assigned[0], &color);
    assert_eq!(assigned[2], &color);
    assert_eq!(
        shorthand.serialize_specified().unwrap(),
        "logical alpha(from red / none) blue"
    );
}

#[test]
fn all_eight_color_longhand_initials_are_directly_symbolic_currentcolor() {
    for name in [
        "border-top-color",
        "border-right-color",
        "border-bottom-color",
        "border-left-color",
        "border-block-start-color",
        "border-block-end-color",
        "border-inline-start-color",
        "border-inline-end-color",
    ] {
        let grammar = CssPropertyGrammar::from_name(name).unwrap();
        let CssPropertyKindRef::Longhand(metadata) = grammar.metadata().unwrap().kind() else {
            panic!("{name} is a longhand")
        };
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("{name} has symbolic initial")
        };
        let color = match value.view() {
            CssLonghandValueRef::BorderTopColor(v)
            | CssLonghandValueRef::BorderRightColor(v)
            | CssLonghandValueRef::BorderBottomColor(v)
            | CssLonghandValueRef::BorderLeftColor(v)
            | CssLonghandValueRef::BorderBlockStartColor(v)
            | CssLonghandValueRef::BorderBlockEndColor(v)
            | CssLonghandValueRef::BorderInlineStartColor(v)
            | CssLonghandValueRef::BorderInlineEndColor(v) => v,
            _ => panic!("{name} has color initial"),
        };
        assert!(color.is_current_color(), "{name}");
    }
}
