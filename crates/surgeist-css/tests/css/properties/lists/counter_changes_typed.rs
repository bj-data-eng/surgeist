#![forbid(unsafe_code)]

//! Functional Lists 3 counter values. The pinned CSSOM 2021 §6.7.2 preserves
//! compatible value spelling; the selected WebKit parser inserts omitted
//! reset/set `0` and increment `1`. Values 4 defines exact integer tokens and
//! symbolic integer-root math; expected text is sourced from those contracts.

use surgeist_css::*;

fn name(value: &str) -> CssContentCounterName {
    CssContentCounterName::try_new(CssIdent::try_new(value).unwrap()).unwrap()
}

fn entry(value: &str, number: Option<CssIntegerValue>) -> CssCounterChangeValue {
    CssCounterChangeValue::new(name(value), number)
}

fn parsed(property: CssKnownProperty, value: &str) -> CssDeclaration {
    let source = format!("{}:{value}", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

fn current(declaration: &CssDeclaration) -> &CssCounterChangesValue {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::CounterReset(value) => value.value(),
        CssKnownPropertyValueRef::CounterIncrement(value) => value.value(),
        CssKnownPropertyValueRef::CounterSet(value) => value.value(),
        _ => panic!("counter longhand"),
    }
}

#[test]
fn checked_entries_keep_omission_order_duplicates_and_none_distinct() {
    assert!(CssCounterChangesValue::try_changes(Vec::new()).is_none());
    assert!(CssCounterChangesValue::none().changes().is_none());
    for forbidden in ["none", "default", "INITIAL", "inherit", "revert-layer"] {
        assert!(CssContentCounterName::try_new(CssIdent::try_new(forbidden).unwrap()).is_none());
    }
    for accepted in ["auto", "span", "1chapter", "chapter name", "MyCounter"] {
        assert_eq!(name(accepted).as_str(), accepted);
    }
    let value = CssCounterChangesValue::try_changes(vec![
        entry("Chapter", None),
        entry(
            "Chapter",
            Some(CssIntegerValue::Literal(CssIntegerLiteral::from_i32(0))),
        ),
        entry(
            "chapter",
            Some(CssIntegerValue::Literal(CssIntegerLiteral::from_i32(-2))),
        ),
    ])
    .unwrap();
    let [first, second, third] = value.changes().unwrap() else {
        panic!("three ordered entries")
    };
    assert_eq!(first.name().as_str(), "Chapter");
    assert!(first.value().is_none());
    assert_eq!(second.name().as_str(), "Chapter");
    assert!(
        matches!(second.value(), Some(CssIntegerValue::Literal(value)) if value.numeric().representation() == "0")
    );
    assert_eq!(third.name().as_str(), "chapter");
    assert!(
        matches!(third.value(), Some(CssIntegerValue::Literal(value)) if value.numeric().representation() == "-2")
    );
}

#[test]
fn one_checked_value_uses_explicit_property_context_for_missing_operands() {
    let value = CssCounterChangesValue::try_changes(vec![
        entry("chapter", None),
        entry(
            "section",
            Some(CssIntegerValue::Literal(CssIntegerLiteral::from_i32(-2))),
        ),
    ])
    .unwrap();
    for (property, expected) in [
        (CssCounterProperty::Reset, "chapter 0 section -2"),
        (CssCounterProperty::Increment, "chapter 1 section -2"),
        (CssCounterProperty::Set, "chapter 0 section -2"),
    ] {
        assert_eq!(value.serialize_specified(property).unwrap(), expected);
    }
    assert!(value.changes().unwrap()[0].value().is_none());
    for property in [
        CssCounterProperty::Reset,
        CssCounterProperty::Increment,
        CssCounterProperty::Set,
    ] {
        assert_eq!(
            CssCounterChangesValue::none()
                .serialize_specified(property)
                .unwrap(),
            "none"
        );
    }
}

#[test]
fn decoded_names_are_escaped_and_exact_large_integers_stay_exact() {
    let huge = "1234567890123456789012345678901234567890";
    let literal =
        CssIntegerLiteral::try_from_component(CssComponentValue::try_token(huge).unwrap()).unwrap();
    assert!(matches!(literal.origin(), CssValueOrigin::Programmatic));
    let value = CssCounterChangesValue::try_changes(vec![
        entry("1chapter", Some(CssIntegerValue::Literal(literal))),
        entry("chapter name", None),
    ])
    .unwrap();
    assert_eq!(
        value
            .serialize_specified(CssCounterProperty::Reset)
            .unwrap(),
        format!(r"\31 chapter {huge} chapter\ name 0")
    );
    assert_eq!(
        value
            .serialize_specified(CssCounterProperty::Increment)
            .unwrap(),
        format!(r"\31 chapter {huge} chapter\ name 1")
    );
}

#[test]
fn small_counter_tokens_preserve_lexemes_and_escaped_names() {
    let padded =
        CssIntegerLiteral::try_from_component(CssComponentValue::try_token("-0002").unwrap())
            .unwrap();
    let representable = CssCounterChangesValue::try_changes(vec![entry(
        "chapter",
        Some(CssIntegerValue::Literal(padded)),
    )])
    .unwrap();
    let old_view = {
        let declaration = parsed(CssKnownProperty::CounterReset, "chapter -0002");
        let CssKnownPropertyValueRef::CounterReset(wrapper) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("reset wrapper")
        };
        wrapper.value().clone()
    };
    let old = old_view.changes().unwrap();
    let Some(CssIntegerValue::Literal(literal)) = old[0].value() else {
        panic!("checked integer")
    };
    assert_eq!(literal.numeric().representation(), "-0002");
    assert!(matches!(literal.origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(
        representable
            .serialize_specified(CssCounterProperty::Reset)
            .unwrap(),
        "chapter -2"
    );

    let escaped = parsed(CssKnownProperty::CounterReset, r"chapter\ name 2");
    let CssKnownPropertyValueRef::CounterReset(wrapper) =
        escaped.known().unwrap().property_value().unwrap()
    else {
        panic!("reset wrapper")
    };
    assert_eq!(
        wrapper.value().changes().unwrap()[0].name().as_str(),
        "chapter name"
    );
    assert_eq!(
        wrapper
            .value()
            .serialize_specified(CssCounterProperty::Reset)
            .unwrap(),
        r"chapter\ name 2"
    );
}

#[test]
fn parsed_values_preserve_full_source_origins() {
    for property in [
        CssKnownProperty::CounterReset,
        CssKnownProperty::CounterIncrement,
        CssKnownProperty::CounterSet,
    ] {
        let ordinary = parsed(property, "chapter section -2 chapter 0");
        let [first, second, third] = current(&ordinary).changes().unwrap() else {
            panic!("ordered current values")
        };
        assert_eq!(first.name().as_str(), "chapter");
        assert!(first.value().is_none());
        assert_eq!(second.name().as_str(), "section");
        assert!(
            matches!(second.value(), Some(CssIntegerValue::Literal(value)) if value.numeric().representation() == "-2")
        );
        assert_eq!(third.name().as_str(), "chapter");
        assert!(
            matches!(third.value(), Some(CssIntegerValue::Literal(value)) if value.numeric().representation() == "0")
        );
        assert_eq!(
            current(&ordinary)
                .serialize_specified(match property {
                    CssKnownProperty::CounterReset => CssCounterProperty::Reset,
                    CssKnownProperty::CounterIncrement => CssCounterProperty::Increment,
                    CssKnownProperty::CounterSet => CssCounterProperty::Set,
                    _ => unreachable!(),
                })
                .unwrap(),
            if property == CssKnownProperty::CounterIncrement {
                "chapter 1 section -2 chapter 0"
            } else {
                "chapter 0 section -2 chapter 0"
            }
        );

        let huge_source = format!("{}:chapter 2147483648", property.canonical_name());
        let huge = parsed(property, "chapter 2147483648");
        let [change] = current(&huge).changes().unwrap() else {
            panic!("one huge integer")
        };
        let Some(CssIntegerValue::Literal(literal)) = change.value() else {
            panic!("exact out-of-i32 integer")
        };
        assert_eq!(literal.numeric().representation(), "2147483648");
        let CssValueOrigin::Parsed(origin) = literal.origin() else {
            panic!("original parsed integer origin")
        };
        assert_eq!(origin.source().as_str(), huge_source);
        let span = origin.span();
        assert_eq!(
            &huge_source[span.start().byte_offset().value()..span.end().byte_offset().value()],
            "2147483648"
        );
        let math_source = format!("{}:chapter calc(2.5)", property.canonical_name());
        let math = parsed(property, "chapter calc(2.5)");
        let [change] = current(&math).changes().unwrap() else {
            panic!("one symbolic calculation")
        };
        let Some(CssIntegerValue::Calculation(calc)) = change.value() else {
            panic!("integer-root math")
        };
        let [component] = calc.components().items() else {
            panic!("one calculation component")
        };
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("original calculation origin")
        };
        assert_eq!(origin.source().as_str(), math_source);
        let span = origin.span();
        assert_eq!(
            &math_source[span.start().byte_offset().value()..span.end().byte_offset().value()],
            "calc("
        );
        let canonical = current(&math)
            .serialize_specified(CssCounterProperty::Reset)
            .unwrap();
        assert_eq!(canonical, "chapter calc(2.5)");
    }
}

#[test]
fn omitted_defaults_and_sibling_entries_share_exact_node_and_byte_budgets() {
    let one = CssCounterChangesValue::try_changes(vec![entry("chapter", None)]).unwrap();
    assert_eq!(
        one.serialize_specified_with_limits(
            CssCounterProperty::Increment,
            CssSpecifiedValueSerializationLimits::new(3, 4, 9),
        )
        .unwrap(),
        "chapter 1"
    );
    let two =
        CssCounterChangesValue::try_changes(vec![entry("chapter", None), entry("section", None)])
            .unwrap();
    let expected = "chapter 1 section 1";
    assert_eq!(
        two.serialize_specified_with_limits(
            CssCounterProperty::Increment,
            CssSpecifiedValueSerializationLimits::new(5, 7, expected.len()),
        )
        .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 7, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 7, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 0, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 6, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 7, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            two.serialize_specified_with_limits(CssCounterProperty::Increment, limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(
        two.serialize_specified(CssCounterProperty::Increment)
            .unwrap(),
        expected
    );
}

#[test]
fn two_symbolic_integers_share_the_css_byte_budget_without_resolution() {
    let calculation = || {
        CssIntegerValue::Calculation(
            CssIntegerCalculation::try_from_components(
                parse_component_values("calc(2.5)").unwrap(),
            )
            .unwrap(),
        )
    };
    let one = CssCounterChangesValue::try_changes(vec![entry("a", Some(calculation()))]).unwrap();
    let two = CssCounterChangesValue::try_changes(vec![
        entry("a", Some(calculation())),
        entry("b", Some(calculation())),
    ])
    .unwrap();
    let single = "a calc(2.5)";
    let both = "a calc(2.5) b calc(2.5)";
    assert_eq!(
        one.serialize_specified(CssCounterProperty::Increment)
            .unwrap(),
        single
    );
    assert_eq!(
        two.serialize_specified(CssCounterProperty::Increment)
            .unwrap(),
        both
    );
    assert_eq!(
        one.serialize_specified_with_limits(
            CssCounterProperty::Increment,
            CssSpecifiedValueSerializationLimits::new(100, 100, single.len()),
        )
        .unwrap(),
        single
    );
    // List + entry + name cost three nodes. The authored calc function and
    // leaf cost two input nodes; numeric projection unwraps calc to one scalar
    // node. The second sibling adds four input and three projection nodes.
    assert_eq!(
        one.serialize_specified_with_limits(
            CssCounterProperty::Increment,
            CssSpecifiedValueSerializationLimits::new(5, 4, single.len()),
        )
        .unwrap(),
        single
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(8, 7, both.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, 6, both.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(
            two.serialize_specified_with_limits(CssCounterProperty::Increment, limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(
        two.serialize_specified_with_limits(
            CssCounterProperty::Increment,
            CssSpecifiedValueSerializationLimits::new(9, 7, both.len()),
        )
        .unwrap(),
        both
    );
    assert_eq!(
        two.serialize_specified_with_limits(
            CssCounterProperty::Increment,
            CssSpecifiedValueSerializationLimits::new(100, 100, single.len()),
        )
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(
        two.serialize_specified_with_limits(
            CssCounterProperty::Increment,
            CssSpecifiedValueSerializationLimits::new(100, 100, both.len()),
        )
        .unwrap(),
        both
    );
}
