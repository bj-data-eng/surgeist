#![forbid(unsafe_code)]

use surgeist_css::*;

fn line_name(value: &str) -> CssGridLineName {
    CssGridLineName::try_new(CssIdent::try_new(value).unwrap()).unwrap()
}

fn exact_integer(value: &str) -> CssIntegerLiteral {
    CssIntegerLiteral::try_from_component(CssComponentValue::try_number(value).unwrap()).unwrap()
}

fn parsed_line(value: &str) -> CssGridLine {
    let source = format!("grid-row-start: {value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::GridRowStart(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("grid-row-start")
    };
    value.value().clone()
}

#[test]
fn parsed_exact_and_math_lines_keep_current_values_and_canonical_output() {
    for (input, output) in [
        ("main 2147483648", "2147483648 main"),
        ("-2147483649 main", "-2147483649 main"),
        ("main -3", "-3 main"),
        ("main 2 span", "span 2 main"),
        ("span main 2147483648", "span 2147483648 main"),
        ("calc(1 + 2) main", "calc(3) main"),
        ("span calc(-1) main", "span calc(-1) main"),
    ] {
        assert_eq!(
            parsed_line(input).serialize_specified().unwrap(),
            output,
            "{input}"
        );
    }
    let CssGridLine::Indexed(index) = parsed_line("2147483648") else {
        panic!("exact line index")
    };
    let CssIntegerValue::ExactLiteral(literal) = index.value() else {
        panic!("large ordinary number must stay exact")
    };
    assert_eq!(literal.numeric().representation(), "2147483648");
    assert!(matches!(literal.origin(), CssValueOrigin::Parsed(_)));
    assert!(matches!(parsed_line("calc(0)"), CssGridLine::Indexed(_)));
    assert!(matches!(parsed_line("span calc(-1)"), CssGridLine::Span(_)));
    assert_eq!(
        parsed_line("calc(2.5)").serialize_specified().unwrap(),
        "calc(2.5)"
    );
    assert!(!parse_style_attribute("grid-row-start: calc(1px)").is_clean());
}

#[test]
fn checked_construction_excludes_invalid_states_and_keeps_authored_variants() {
    for word in [
        "default",
        "DeFaUlT",
        "AUTO",
        "SpAn",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
    ] {
        assert!(
            CssGridLineName::try_new(CssIdent::try_new(word).unwrap()).is_none(),
            "{word}"
        );
    }
    assert!(CssGridLineName::try_new(CssIdent::try_new("none").unwrap()).is_some());
    assert!(CssGridLine::try_indexed(CssIntegerValue::Literal(0), None).is_none());
    assert!(
        CssGridLine::try_indexed(CssIntegerValue::ExactLiteral(exact_integer("-000")), None)
            .is_none()
    );
    assert!(CssGridLine::try_span(None, None).is_none());

    let small =
        CssGridLine::try_indexed(CssIntegerValue::ExactLiteral(exact_integer("+002")), None)
            .unwrap();
    assert_eq!(small.serialize_specified().unwrap(), "2");
    assert_ne!(
        small,
        CssGridLine::try_indexed(CssIntegerValue::Literal(2), None).unwrap(),
        "ordinary and exact authored variants remain distinct",
    );
    let large = CssGridLine::try_indexed(
        CssIntegerValue::ExactLiteral(exact_integer("2147483648")),
        None,
    )
    .unwrap();
    assert_eq!(large.serialize_specified().unwrap(), "2147483648");
    let CssGridLine::Indexed(index) = &large else {
        panic!("constructed index")
    };
    let CssIntegerValue::ExactLiteral(literal) = index.value() else {
        panic!("constructed exact token")
    };
    assert!(matches!(literal.origin(), CssValueOrigin::Programmatic));
    assert_eq!(large, parsed_line("2147483648"));
    let named =
        CssGridLine::try_indexed(CssIntegerValue::Literal(2), Some(line_name("main"))).unwrap();
    assert_eq!(named.serialize_specified().unwrap(), "2 main");
    let span = CssGridLine::try_span(
        Some(CssPositiveIntegerValue::ExactLiteral(
            CssPositiveIntegerLiteral::try_new(exact_integer("+002")).unwrap(),
        )),
        Some(line_name("main")),
    )
    .unwrap();
    assert_eq!(span.serialize_specified().unwrap(), "span 2 main");
    let math = CssGridLine::try_indexed(
        CssIntegerValue::Calculation(
            CssIntegerCalculation::try_from_components(parse_component_values("calc(0)").unwrap())
                .unwrap(),
        ),
        None,
    )
    .unwrap();
    assert_eq!(math.serialize_specified().unwrap(), "calc(0)");
    assert_eq!(math, parsed_line("calc(0)"));
}

#[test]
fn authored_omissions_and_effective_members_remain_distinct() {
    let named = CssGridLine::Name(line_name("Hero"));
    let range = CssGridLineRange::new(named.clone(), None);
    assert!(range.authored_end().is_none());
    assert_eq!(range.effective_end(), named);
    assert_eq!(range.serialize_specified().unwrap(), "Hero");
    let explicit_auto = CssGridLineRange::new(named.clone(), Some(CssGridLine::Auto));
    assert_eq!(explicit_auto.authored_end(), Some(&CssGridLine::Auto));
    assert_eq!(explicit_auto.effective_end(), CssGridLine::Auto);
    assert_eq!(explicit_auto.serialize_specified().unwrap(), "Hero / auto");

    let indexed =
        CssGridLine::try_indexed(CssIntegerValue::Literal(2), Some(line_name("Hero"))).unwrap();
    let range = CssGridLineRange::new(indexed.clone(), None);
    assert_eq!(range.effective_end(), CssGridLine::Auto);
    assert_eq!(range.serialize_specified().unwrap(), "2 Hero");

    let area = CssGridArea::try_new(
        indexed,
        Some(CssGridLine::Name(line_name("Nav"))),
        None,
        None,
    )
    .unwrap();
    assert!(area.authored_row_end().is_none());
    assert!(area.authored_column_end().is_none());
    assert_eq!(area.effective_row_end(), CssGridLine::Auto);
    assert_eq!(
        area.effective_column_end(),
        CssGridLine::Name(line_name("Nav"))
    );
    assert_eq!(area.serialize_specified().unwrap(), "2 Hero / Nav");

    assert!(CssGridArea::try_new(CssGridLine::Auto, None, Some(CssGridLine::Auto), None).is_none());
    assert!(CssGridArea::try_new(CssGridLine::Auto, None, None, Some(CssGridLine::Auto)).is_none());
    assert!(
        CssGridArea::try_new(
            CssGridLine::Auto,
            Some(CssGridLine::Auto),
            None,
            Some(CssGridLine::Auto)
        )
        .is_none()
    );
}

#[test]
fn decoded_names_serialize_as_identifiers_and_reenter() {
    let line = CssGridLine::Name(line_name("A:B"));
    let css = line.serialize_specified().unwrap();
    assert_eq!(css, "A\\:B");
    assert_eq!(
        line.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 4))
            .unwrap(),
        "A\\:B"
    );
    assert_eq!(
        line.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 3))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(parsed_line(&css), line);
    assert_eq!(
        CssGridLine::Name(line_name("MiXeD"))
            .serialize_specified()
            .unwrap(),
        "MiXeD"
    );
}

#[test]
fn shorthand_serialization_charges_one_cumulative_budget() {
    let area = CssGridArea::try_new(
        CssGridLine::Name(line_name("a")),
        Some(CssGridLine::Name(line_name("b"))),
        Some(CssGridLine::Name(line_name("c"))),
        Some(CssGridLine::Name(line_name("d"))),
    )
    .unwrap();
    let expected = "a / b / c / d";
    assert_eq!(area.serialize_specified().unwrap(), expected);
    assert_eq!(
        area.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            4,
            4,
            expected.len()
        ))
        .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(3, 4, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 3, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 4, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            area.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}
