#![forbid(unsafe_code)]
//! Functional evidence for five new represented text-value providers.
//! New serializer APIs have no executable preimplementation RED. The separate
//! signed-thickness/unordered-indent admission regressions use existing APIs.
//! Orders follow selected Text 4 §9.1, Text Decoration 4 §§2.1/2.4/2.6 and
//! CSS2 vertical-align. Numeric spelling follows its existing specified policy.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

fn length(text: &str) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_component(CssComponentValue::try_token(text).unwrap())
        .unwrap()
}

fn calculation(text: &str) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_calculation(
        CssLengthPercentageCalculation::try_from_components(parse_component_values(text).unwrap())
            .unwrap(),
    )
    .unwrap()
}

fn declaration(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    value.clone()
}

macro_rules! parsed {
    ($source:expr, $variant:ident) => {{
        let declaration = declaration($source);
        let CssKnownPropertyValueRef::$variant(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("expected text property")
        };
        value.value().clone()
    }};
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
        (L::new(0, projection, expected.len()), K::InputNodeLimit),
        (L::new(input, 0, expected.len()), K::ProjectionNodeLimit),
        (L::new(input, projection, 0), K::ByteLimit),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), kind);
    }
}

macro_rules! check {
    ($value:expr, $input:expr, $projection:expr, $expected:expr) => {{
        let value = $value;
        let expected: &str = $expected;
        assert_eq!(value.serialize_specified().unwrap(), expected);
        budget(expected, $input, $projection, |limits| {
            value.serialize_specified_with_limits(limits)
        });
    }};
}

#[test]
fn indent_flags_emit_in_grammar_order_with_exact_cumulative_costs() {
    for (hanging, each_line, expected, nodes) in [
        (false, false, "-2.5%", 2),
        (true, false, "-2.5% hanging", 3),
        (false, true, "-2.5% each-line", 3),
        (true, true, "-2.5% hanging each-line", 4),
    ] {
        let value = CssTextIndent::new(length("-2.50%"), hanging, each_line);
        check!(&value, nodes, nodes, expected);
        assert_eq!(
            parsed!(&format!("text-indent:{expected}"), TextIndent),
            CssTextIndent::new(length("-2.5%"), hanging, each_line)
        );
        assert_eq!(value.length().origin(), &CssValueOrigin::Programmatic);
        let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) =
            value.length().literal_component().unwrap().view()
        else {
            panic!("retained authored percentage")
        };
        assert_eq!(number.representation(), "-2.50");
        assert_eq!(value.hanging(), hanging);
        assert_eq!(value.each_line(), each_line);
    }
}

#[test]
fn every_indent_permutation_reenters_the_same_semantic_value() {
    let expected = CssTextIndent::new(length("-1px"), true, true);
    for text in [
        "-1px hanging each-line",
        "-1px each-line hanging",
        "hanging -1px each-line",
        "hanging each-line -1px",
        "each-line -1px hanging",
        "each-line hanging -1px",
    ] {
        let value = parsed!(&format!("text-indent:{text}"), TextIndent);
        assert_eq!(value, expected);
        assert_eq!(
            value.serialize_specified().unwrap(),
            "-1px hanging each-line"
        );
    }
    for text in [
        "hanging",
        "each-line hanging",
        "1px 2px",
        "hanging 1px hanging",
        "each-line 1px each-line",
        "hanging each-line 1px 2%",
        "1px sideways",
    ] {
        let source = format!("text-indent:{text}; color:blue");
        let report = parse_style_attribute(&source);
        assert!(!report.is_clean(), "{source}");
        let [sibling] = report.syntax().as_slice() else {
            panic!("only valid sibling")
        };
        assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
        assert!(validate_style_attribute(&source).is_err());
    }
}

#[test]
fn vertical_alignment_keywords_and_signed_scalars_use_their_own_node_costs() {
    for (value, expected) in [
        (CssVerticalAlign::Baseline, "baseline"),
        (CssVerticalAlign::Sub, "sub"),
        (CssVerticalAlign::Super, "super"),
        (CssVerticalAlign::TextTop, "text-top"),
        (CssVerticalAlign::TextBottom, "text-bottom"),
        (CssVerticalAlign::Middle, "middle"),
        (CssVerticalAlign::Top, "top"),
        (CssVerticalAlign::Bottom, "bottom"),
        (CssVerticalAlign::Length(length("-3.25%")), "-3.25%"),
        (CssVerticalAlign::Length(length("-2px")), "-2px"),
    ] {
        check!(&value, 1, 1, expected);
        assert_eq!(
            parsed!(&format!("vertical-align:{expected}"), VerticalAlign),
            value
        );
    }
}

#[test]
fn line_keywords_canonicalize_grammar_order_without_mutating_authored_order() {
    let authored = vec![
        CssTextDecorationLineComponent::Blink,
        CssTextDecorationLineComponent::LineThrough,
        CssTextDecorationLineComponent::Overline,
        CssTextDecorationLineComponent::Underline,
    ];
    let value = CssTextDecorationLine::try_new(authored.clone()).unwrap();
    check!(&value, 5, 5, "underline overline line-through blink");
    assert_eq!(value.components(), authored);
    let expected = [
        CssTextDecorationLineComponent::Underline,
        CssTextDecorationLineComponent::Overline,
        CssTextDecorationLineComponent::LineThrough,
        CssTextDecorationLineComponent::Blink,
    ];
    assert_eq!(
        parsed!(
            "text-decoration-line:underline overline line-through blink",
            TextDecorationLine
        )
        .components(),
        expected
    );
    for (component, expected) in [
        (CssTextDecorationLineComponent::Underline, "underline"),
        (CssTextDecorationLineComponent::Overline, "overline"),
        (CssTextDecorationLineComponent::LineThrough, "line-through"),
        (CssTextDecorationLineComponent::Blink, "blink"),
    ] {
        check!(
            CssTextDecorationLine::try_new(vec![component]).unwrap(),
            2,
            2,
            expected
        );
    }
    let none = parsed!("text-decoration-line:none", TextDecorationLine);
    check!(&none, 2, 2, "none");
    assert!(none.is_none());
    assert!(none.components().is_empty());
    assert!(CssTextDecorationLine::try_new(vec![]).is_none());
    assert!(
        CssTextDecorationLine::try_new(vec![CssTextDecorationLineComponent::Underline; 2])
            .is_none()
    );
}

#[test]
fn every_line_subset_keeps_its_members_in_canonical_grammar_order() {
    let components = [
        CssTextDecorationLineComponent::Underline,
        CssTextDecorationLineComponent::Overline,
        CssTextDecorationLineComponent::LineThrough,
        CssTextDecorationLineComponent::Blink,
    ];
    let keywords = ["underline", "overline", "line-through", "blink"];
    for mask in 1..16 {
        let expected = (0..4)
            .filter(|index| mask & (1 << index) != 0)
            .map(|index| keywords[index])
            .collect::<Vec<_>>()
            .join(" ");
        let canonical = (0..4)
            .filter(|index| mask & (1 << index) != 0)
            .map(|index| components[index])
            .collect::<Vec<_>>();
        let mut authored = canonical.clone();
        authored.reverse();
        let value = CssTextDecorationLine::try_new(authored.clone()).unwrap();
        check!(&value, canonical.len() + 1, canonical.len() + 1, &expected);
        assert_eq!(value.components(), authored);
        let reparsed = parsed!(
            &format!("text-decoration-line:{expected}"),
            TextDecorationLine
        );
        assert_eq!(reparsed.components(), canonical);
    }
}

#[test]
fn thickness_keeps_signed_numeric_values_and_intrinsic_keywords_without_flooring() {
    for (value, expected) in [
        (CssTextDecorationThickness::Auto, "auto"),
        (CssTextDecorationThickness::FromFont, "from-font"),
        (CssTextDecorationThickness::Length(length("-10px")), "-10px"),
        (CssTextDecorationThickness::Length(length("-27%")), "-27%"),
        (CssTextDecorationThickness::Length(length("0")), "0"),
    ] {
        check!(&value, 1, 1, expected);
        assert_eq!(
            parsed!(
                &format!("text-decoration-thickness:{expected}"),
                TextDecorationThickness
            ),
            value
        );
    }
}

#[test]
fn decoration_order_is_line_thickness_style_color_and_all_fields_stay_explicit() {
    let value = CssTextDecoration::try_new(
        Some(
            CssTextDecorationLine::try_new(vec![
                CssTextDecorationLineComponent::Overline,
                CssTextDecorationLineComponent::Underline,
            ])
            .unwrap(),
        ),
        Some(CssColor::current_color()),
        Some(CssTextDecorationStyle::Wavy),
        Some(CssTextDecorationThickness::Length(length("-1px"))),
    )
    .unwrap();
    check!(&value, 7, 7, "underline overline -1px wavy currentcolor");
    assert_eq!(
        value.line().unwrap().components(),
        &[
            CssTextDecorationLineComponent::Overline,
            CssTextDecorationLineComponent::Underline
        ]
    );
    let parsed = parsed!(
        "text-decoration:currentcolor wavy -1px overline underline",
        TextDecoration
    );
    assert_eq!(
        parsed.serialize_specified().unwrap(),
        "underline overline -1px wavy currentcolor"
    );
    let reparsed = parsed!(
        "text-decoration:underline overline -1px wavy currentcolor",
        TextDecoration
    );
    assert_eq!(reparsed.thickness(), value.thickness());
    assert_eq!(reparsed.color(), value.color());
    assert_eq!(reparsed.style(), value.style());
    let none = parsed!("text-decoration-line:none", TextDecorationLine);
    let explicit_initials = CssTextDecoration::try_new(
        Some(none),
        Some(CssColor::current_color()),
        Some(CssTextDecorationStyle::Solid),
        Some(CssTextDecorationThickness::Auto),
    )
    .unwrap();
    check!(&explicit_initials, 6, 6, "none auto solid currentcolor");
    assert_eq!(
        parsed!(
            "text-decoration:none auto solid currentcolor",
            TextDecoration
        ),
        explicit_initials
    );
}

#[test]
fn every_decoration_field_subset_preserves_omissions_and_reparses() {
    for mask in 1..16 {
        let line = (mask & 1 != 0).then(|| {
            CssTextDecorationLine::try_new(vec![CssTextDecorationLineComponent::Underline]).unwrap()
        });
        let thickness = (mask & 2 != 0).then_some(CssTextDecorationThickness::Auto);
        let style = (mask & 4 != 0).then_some(CssTextDecorationStyle::Solid);
        let color = (mask & 8 != 0).then(CssColor::current_color);
        let value = CssTextDecoration::try_new(line, color, style, thickness).unwrap();
        let expected = [
            (1, "underline"),
            (2, "auto"),
            (4, "solid"),
            (8, "currentcolor"),
        ]
        .into_iter()
        .filter(|(bit, _)| mask & bit != 0)
        .map(|(_, text)| text)
        .collect::<Vec<_>>()
        .join(" ");
        let nodes = 1
            + (mask & 1 != 0) as usize * 2
            + (mask & 2 != 0) as usize
            + (mask & 4 != 0) as usize
            + (mask & 8 != 0) as usize;
        check!(&value, nodes, nodes, &expected);
        assert_eq!(value.line().is_some(), mask & 1 != 0);
        assert_eq!(value.thickness().is_some(), mask & 2 != 0);
        assert_eq!(value.style().is_some(), mask & 4 != 0);
        assert_eq!(value.color().is_some(), mask & 8 != 0);
        assert_eq!(
            parsed!(&format!("text-decoration:{expected}"), TextDecoration),
            value
        );
    }
    assert!(CssTextDecoration::try_new(None, None, None, None).is_none());
}

#[test]
fn shared_numeric_projection_preserves_symbolic_origins_and_exact_budgets() {
    let thickness = CssTextDecorationThickness::Length(calculation("calc(-1px + -2px)"));
    check!(&thickness, 4, 3, "calc(-3px)");
    let align = CssVerticalAlign::Length(calculation("calc(1px + 2px)"));
    check!(&align, 4, 3, "calc(3px)");
    let indent = CssTextIndent::new(calculation("calc(1px + 2px)"), true, true);
    check!(&indent, 7, 6, "calc(3px) hanging each-line");
    let CssTextDecorationThickness::Length(value) = &thickness else {
        unreachable!()
    };
    let before = value.clone();
    let root = value.calculation().unwrap();
    let CssValueOrigin::Parsed(origin) = root.origin() else {
        panic!("original calculation origin")
    };
    assert_eq!(origin.source().as_str(), "calc(-1px + -2px)");
    assert_eq!(origin.span().end().byte_offset().value(), "calc(".len());
    assert_eq!(value, &before);
    let mixed = parsed!(
        "text-decoration-thickness:calc(40% - 20px)",
        TextDecorationThickness
    );
    assert_eq!(mixed.serialize_specified().unwrap(), "calc(40% - 20px)");
    assert_eq!(
        parsed!(
            "text-decoration-thickness:calc(-20px + 40%)",
            TextDecorationThickness
        )
        .serialize_specified()
        .unwrap(),
        "calc(40% - 20px)"
    );
}

#[test]
fn rounding_and_failed_serialization_keep_exact_coefficients_units_and_origins() {
    let source = "/* 🦀 */ text-decoration-thickness:-1e-999%";
    let value = parsed!(source, TextDecorationThickness);
    let before = value.clone();
    check!(&value, 1, 1, "0%");
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(1, 1, 1))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    assert_eq!(value, before);
    let CssTextDecorationThickness::Length(retained) = value else {
        panic!("numeric thickness")
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Percentage(token)) =
        retained.literal_component().unwrap().view()
    else {
        panic!("percentage")
    };
    assert_eq!(token.representation(), "-1e-999");
    let CssValueOrigin::Parsed(origin) = retained.origin() else {
        panic!("parsed origin")
    };
    let start = source.find("-1e-999%").unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), source.len());
    assert_eq!(
        origin.span().start().column().value() as usize,
        source[..start].encode_utf16().count()
    );
    let huge = CssTextDecorationThickness::Length(length("1e999px"));
    assert_eq!(
        huge.serialize_specified_with_limits(L::new(1, 1, 16))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    let CssTextDecorationThickness::Length(huge) = huge else {
        unreachable!()
    };
    assert_eq!(huge.origin(), &CssValueOrigin::Programmatic);
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
        huge.literal_component().unwrap().view()
    else {
        panic!("dimension")
    };
    assert_eq!(number.representation(), "1e999");
    assert_eq!(unit, "px");
}

#[test]
fn a_late_aggregate_failure_returns_no_partial_css_and_does_not_mutate_children() {
    let value = parsed!(
        "text-decoration:blink underline -10px solid currentcolor",
        TextDecoration
    );
    let before = value.clone();
    let expected = "underline blink -10px solid currentcolor";
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(7, 7, expected.len() - 1))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    assert_eq!(value, before);
    assert_eq!(
        value.line().unwrap().components(),
        &[
            CssTextDecorationLineComponent::Blink,
            CssTextDecorationLineComponent::Underline
        ]
    );
    assert_eq!(value.serialize_specified().unwrap(), expected);
}
