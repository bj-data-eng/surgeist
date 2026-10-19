#![forbid(unsafe_code)]
//! CSS2 REC 2011-06-07 §9.9.1 supplies auto and exact signed integers,
//! initial auto, noninheritance. Values4 specified math and the shared bounded
//! writer are existing product contracts; computed integer rounding is downstream.
use surgeist_css::*;

fn declaration(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("z-index:{text}!important"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

fn value(source: &CssDeclaration) -> &CssZIndexValue {
    let CssKnownPropertyValueRef::ZIndex(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("z-index")
    };
    value.value()
}

fn expanded(source: &CssDeclaration) -> CssZIndexValue {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("longhand")
    };
    let [item] = values.items() else {
        panic!("one longhand")
    };
    let CssLonghandValueRef::ZIndex(value) = item.ordinary_value().unwrap().view() else {
        panic!("z-index payload")
    };
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    value.clone()
}

#[test]
fn initial_and_ordinary_longhand_payloads_expose_exact_z_index_values() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::ZIndex.metadata().unwrap().kind()
    else {
        panic!("longhand")
    };
    assert!(!metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary initial")
    };
    let CssLonghandValueRef::ZIndex(initial) = initial.view() else {
        panic!("z-index initial")
    };
    assert_eq!(initial, &CssZIndexValue::Auto);
    assert_eq!(initial.serialize_specified().unwrap(), "auto");
    for text in ["AUTO", "+0002147483648", "-2147483649", "calc(1.5)"] {
        let source = declaration(text);
        assert_eq!(&expanded(&source), value(&source));
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ZIndex),
            source.value_components().clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(expanded(&checked), expanded(&source));
        if let CssZIndexValue::Integer(CssIntegerValue::Literal(literal)) = expanded(&source) {
            assert_eq!(literal.component(), &source.value_components().items()[0]);
            assert_eq!(
                literal.origin(),
                source.value_components().items()[0].origin()
            );
        }
    }
}

#[test]
fn canonical_auto_exact_integers_and_unrounded_math_round_trip_without_mutation() {
    for (input, expected) in [
        ("AUTO", "auto"),
        (r"a\75 to", "auto"),
        ("+0000", "0"),
        ("-0000", "0"),
        ("+0002147483648", "2147483648"),
        ("-0002147483649", "-2147483649"),
        ("9007199254740993", "9007199254740993"),
        ("calc(1.5)", "calc(1.5)"),
        ("calc(-1.5)", "calc(-1.5)"),
        ("calc(1 + 2)", "calc(3)"),
        ("calc(2px / 1px)", "calc(2)"),
        ("calc(1em / 1em)", "calc(1em / 1em)"),
        ("sign(1em - 1px)", "sign(1em - 1px)"),
        ("calc(infinity)", "calc(infinity)"),
        ("calc(NaN)", "calc(NaN)"),
    ] {
        let source = declaration(input);
        let before = source.clone();
        let value = value(&source);
        assert_eq!(value.serialize_specified().unwrap(), expected, "{input}");
        assert_eq!(
            self::value(&declaration(expected))
                .serialize_specified()
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    65_536,
                    262_144,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    65_536,
                    262_144,
                    expected.len() - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert!(source.same_occurrence(&before));
        assert_eq!(source.value_components(), before.value_components());
        assert_eq!(self::value(&source), self::value(&before));
    }
}

#[test]
fn each_z_index_branch_obeys_atomic_input_projection_and_byte_limits() {
    for text in ["auto", "+0002", "calc(1 + 2)"] {
        let source = declaration(text);
        let value = value(&source);
        let before = value.clone();
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 262_144, 1_048_576),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(65_536, 0, 1_048_576),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(65_536, 262_144, 0),
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
            assert_eq!(value, &before);
        }
        assert!(value.serialize_specified().is_ok());
    }
    for (value, expected) in [
        (CssZIndexValue::Auto, "auto"),
        (
            CssZIndexValue::Integer(CssIntegerValue::Literal(CssIntegerLiteral::from_i32(-2))),
            "-2",
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
    }
}

#[test]
fn reentered_z_index_longhand_retains_parsed_and_programmatic_integer_origins() {
    let source = declaration("var(--level)");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending")
    };
    for replacement in [
        parse_component_values("+0002147483648").unwrap(),
        CssComponentValues::try_new(vec![
            CssComponentValue::try_number("+0002147483648").unwrap(),
        ])
        .unwrap(),
    ] {
        let before = replacement.clone();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("longhand")
        };
        let [item] = values.items() else {
            panic!("one terminal")
        };
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
        let CssLonghandValueRef::ZIndex(CssZIndexValue::Integer(CssIntegerValue::Literal(literal))) =
            item.ordinary_value().unwrap().view()
        else {
            panic!("exact integer payload")
        };
        assert_eq!(literal.component(), &replacement.items()[0]);
        assert_eq!(literal.origin(), replacement.items()[0].origin());
        let CssLonghandValueRef::ZIndex(value) = item.ordinary_value().unwrap().view() else {
            unreachable!()
        };
        assert_eq!(value.serialize_specified().unwrap(), "2147483648");
        for (limits, failure) in [
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, 10),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, 9),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                failure
            );
            assert_eq!(literal.component(), &before.items()[0]);
            assert_eq!(literal.origin(), before.items()[0].origin());
            assert_eq!(item.replacement_components(), Some(&before));
        }
        assert_eq!(item.replacement_components(), Some(&before));
    }
    let replacement = parse_component_values("calc(1.5)").unwrap();
    let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("longhand")
    };
    let CssLonghandValueRef::ZIndex(CssZIndexValue::Integer(CssIntegerValue::Calculation(math))) =
        values.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("unrounded math")
    };
    assert_eq!(math.components(), &replacement);
    assert_eq!(
        math.components().items()[0].origin(),
        replacement.items()[0].origin()
    );
    assert_eq!(
        values.items()[0].replacement_components(),
        Some(&replacement)
    );
}

#[test]
fn parsed_symbolic_math_keeps_origins_after_late_projection_and_byte_failure() {
    let source = declaration("sign(1em - 1px)");
    let value = value(&source);
    let before = value.clone();
    let CssZIndexValue::Integer(CssIntegerValue::Calculation(math)) = value else {
        panic!("symbolic Number math")
    };
    let original = math.components().clone();
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(65_536, 1, 1_048_576),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(65_536, 262_144, "sign(1em - 1px)".len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, &before);
        assert_eq!(math.components(), &original);
        for (component, retained) in original.items().iter().zip(math.components().items()) {
            assert_eq!(component.origin(), retained.origin());
        }
    }
    assert_eq!(value.serialize_specified().unwrap(), "sign(1em - 1px)");
}
