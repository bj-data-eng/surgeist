#![forbid(unsafe_code)]

//! Public specified-value contracts from CSS Box Alignment 3 WD 2026-01-30
//! §§8.1, 8.2, and 8.4.

use surgeist_css::*;

fn scalar(text: &str) -> CssSpecifiedNonNegativeLengthPercentage {
    let components = parse_component_values(text).expect("valid component syntax");
    if text.starts_with("calc(") {
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(components)
                .expect("length-percentage math"),
        )
        .expect("checked deferred math")
    } else {
        let [component] = components.items() else {
            panic!("one scalar: {text}")
        };
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(component.clone())
            .expect("checked nonnegative scalar")
    }
}

fn parsed(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        CssPropertyGrammar::from_name(name).expect("gap grammar"),
        parse_component_values(value).expect("valid component syntax"),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

fn row_value(declaration: &CssDeclaration) -> CssGapValue {
    let CssKnownPropertyValueRef::RowGap(wrapper) = declaration
        .known()
        .expect("known row-gap")
        .property_value()
        .expect("ordinary row-gap")
    else {
        panic!("row-gap wrapper")
    };
    wrapper.value().clone()
}

fn column_value(declaration: &CssDeclaration) -> CssGapValue {
    let CssKnownPropertyValueRef::ColumnGap(wrapper) = declaration
        .known()
        .expect("known column-gap")
        .property_value()
        .expect("ordinary column-gap")
    else {
        panic!("column-gap wrapper")
    };
    wrapper.value().clone()
}

fn gap_value(declaration: &CssDeclaration) -> CssGapShorthand {
    let CssKnownPropertyValueRef::Gap(wrapper) = declaration
        .known()
        .expect("known gap")
        .property_value()
        .expect("ordinary gap")
    else {
        panic!("gap wrapper")
    };
    wrapper.value().clone()
}

#[test]
fn typed_longhands_preserve_normal_and_exact_nonnegative_numeric_values() {
    assert_eq!(CssGapValue::Normal.serialize_specified().unwrap(), "normal");
    for (name, extract) in [
        ("row-gap", row_value as fn(&CssDeclaration) -> CssGapValue),
        (
            "column-gap",
            column_value as fn(&CssDeclaration) -> CssGapValue,
        ),
    ] {
        assert_eq!(extract(&parsed(name, "normal")), CssGapValue::Normal);
        assert_eq!(extract(&checked(name, "normal")), CssGapValue::Normal);
        for (text, canonical) in [
            ("0", "0"),
            ("2px", "2px"),
            ("3.25%", "3.25%"),
            ("1.0000000000000000000001px", "1px"),
            ("999999999999999999999999px", "999999999999999999999999px"),
        ] {
            let direct = CssGapValue::LengthPercentage(scalar(text));
            assert_eq!(direct.serialize_specified().unwrap(), canonical, "{text}");
            if text == "1.0000000000000000000001px" {
                let CssGapValue::LengthPercentage(scalar) = &direct else {
                    panic!("numeric gap")
                };
                assert!(matches!(
                    scalar.literal_component().unwrap().view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
                        if number.representation() == "1.0000000000000000000001" && unit == "px"
                ));
            }
            for source in [parsed(name, text), checked(name, text)] {
                let actual = extract(&source);
                assert_eq!(actual, direct, "{name}:{text}");
                assert_eq!(actual.serialize_specified().unwrap(), canonical);
            }
        }
    }
    assert_ne!(
        CssGapValue::LengthPercentage(scalar("1.0000000000000000000001px")),
        CssGapValue::LengthPercentage(scalar("1px"))
    );
    assert_ne!(
        CssGapValue::LengthPercentage(scalar("999999999999999999999999px")),
        CssGapValue::LengthPercentage(scalar("999999999999999999999998px"))
    );
    for text in ["1e100px", "1e-100%"] {
        let direct = CssGapValue::LengthPercentage(scalar(text));
        assert_eq!(row_value(&parsed("row-gap", text)), direct);
        assert_eq!(column_value(&checked("column-gap", text)), direct);
    }
}

#[test]
fn typed_shorthand_preserves_authored_arity_and_independent_row_column_values() {
    for (text, row, column, has_authored_column) in [
        ("normal", "normal", "normal", false),
        ("2px", "2px", "2px", false),
        ("normal 3%", "normal", "3%", true),
        ("1.25px normal", "1.25px", "normal", true),
        ("1px 2%", "1px", "2%", true),
    ] {
        for source in [parsed("gap", text), checked("gap", text)] {
            let actual = gap_value(&source);
            assert_eq!(actual.row().serialize_specified().unwrap(), row);
            assert_eq!(actual.column().serialize_specified().unwrap(), column);
            assert_eq!(actual.authored_column().is_some(), has_authored_column);
            assert_eq!(actual.serialize_specified().unwrap(), text);
        }
    }

    let omitted = CssGapShorthand::new(CssGapValue::LengthPercentage(scalar("1px")), None);
    let explicit = CssGapShorthand::new(
        CssGapValue::LengthPercentage(scalar("1px")),
        Some(CssGapValue::LengthPercentage(scalar("1px"))),
    );
    assert_eq!(omitted.row(), omitted.column());
    assert!(omitted.authored_column().is_none());
    assert_eq!(omitted.serialize_specified().unwrap(), "1px");
    assert_eq!(explicit.serialize_specified().unwrap(), "1px 1px");
    assert_ne!(omitted, explicit);
    assert_ne!(omitted, CssGapShorthand::new(CssGapValue::Normal, None));
}

#[test]
fn negative_literals_and_non_gap_tokens_are_invalid_but_math_remains_symbolic() {
    for name in ["row-gap", "column-gap", "gap", "grid-gap"] {
        for value in ["-1px", "-1e-100%", "1fr", "1", "auto", "1px 2px 3px"] {
            let report = parse_style_attribute(&format!("color:red;{name}:{value};color:blue"));
            assert_eq!(report.syntax().len(), 2, "{name}:{value}");
            assert_eq!(report.diagnostics().len(), 1, "{name}:{value}");
            assert_eq!(
                report.diagnostics()[0].action(),
                CssRecoveryAction::DropDeclaration
            );
            assert!(
                parse_property_value_for_grammar(
                    CssPropertyGrammar::from_name(name).unwrap(),
                    parse_component_values(value).unwrap(),
                    CssImportance::Normal,
                )
                .is_err(),
                "checked {name}:{value}"
            );
        }
    }
    for name in ["row-gap", "column-gap", "gap"] {
        for value in ["calc(1px + 2%)", "calc(1px - 2px)"] {
            assert!(
                parse_style_attribute(&format!("{name}:{value}")).is_clean(),
                "{name}:{value}"
            );
            checked(name, value);
        }
    }
    let deferred = CssGapValue::LengthPercentage(scalar("calc(1px - 2px)"));
    assert!(deferred.serialize_specified().is_ok());
    let CssGapValue::LengthPercentage(number) = deferred else {
        unreachable!()
    };
    assert!(number.calculation().is_some());
}

#[test]
fn parsed_aliases_keep_exact_authored_name_and_typed_canonical_identity() {
    for (alias, canonical, property) in [
        ("GrId-RoW-GaP", "row-gap", CssKnownProperty::RowGap),
        ("GrId-CoLuMn-GaP", "column-gap", CssKnownProperty::ColumnGap),
        ("GrId-GaP", "gap", CssKnownProperty::Gap),
    ] {
        let value = if property == CssKnownProperty::Gap {
            "1px 2%"
        } else {
            "1px"
        };
        let source = parsed(alias, value);
        assert_eq!(source.known().unwrap().property(), property);
        let origin = source.parsed_name().expect("parsed name origin");
        let span = origin.span();
        assert_eq!(
            &origin.source().as_str()
                [span.start().byte_offset().value()..span.end().byte_offset().value()],
            alias
        );
        assert_eq!(checked(alias, value).known().unwrap().property(), property);
        assert_eq!(
            CssPropertyGrammar::from_name(alias).unwrap().name(),
            canonical
        );
        assert_eq!(property.aliases().len(), 1);
        assert_eq!(property.aliases()[0], alias.to_ascii_lowercase());
    }
    for (name, expected) in [
        ("row-gap", CssKnownProperty::RowGap),
        ("column-gap", CssKnownProperty::ColumnGap),
        ("gap", CssKnownProperty::Gap),
    ] {
        assert_eq!(
            CssKnownProperty::all()
                .iter()
                .filter(|&&item| item == expected)
                .count(),
            1,
            "{name}"
        );
    }
    for legacy in ["grid-row-gap", "grid-column-gap", "grid-gap"] {
        assert_eq!(
            CssKnownProperty::all()
                .iter()
                .filter(|property| property.canonical_name() == legacy)
                .count(),
            0,
            "{legacy} is only a name alias"
        );
    }

    let CssPropertyKindRef::UniversalReset(metadata) = CssKnownProperty::All
        .metadata()
        .expect("all metadata")
        .kind()
    else {
        panic!("all has universal reset metadata")
    };
    let source = parsed("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&source).expect("all expands symbolically")
    else {
        panic!("all has a universal reset")
    };
    for (alias, canonical) in [
        ("grid-row-gap", CssKnownProperty::RowGap),
        ("grid-column-gap", CssKnownProperty::ColumnGap),
    ] {
        let resolved = CssKnownProperty::from_name(alias).unwrap();
        assert_eq!(resolved, canonical);
        let name = CssPropertyNameRef::Known(resolved);
        assert!(!metadata.excludes(name));
        assert!(!reset.excludes(name));
    }
    assert!(reset.source().same_occurrence(&source));
    assert_eq!(reset.source().importance(), CssImportance::Important);
}

#[test]
fn one_value_and_two_value_serializers_charge_actual_authored_structure() {
    let normal = CssGapValue::Normal;
    assert_eq!(
        normal
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap(),
        "normal"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 6),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 6),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 5),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            normal
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }

    let one = CssGapShorthand::new(CssGapValue::Normal, None);
    assert_eq!(one.serialize_specified().unwrap(), "normal");
    let two = CssGapShorthand::new(
        CssGapValue::LengthPercentage(scalar("1px")),
        Some(CssGapValue::LengthPercentage(scalar("2%"))),
    );
    assert_eq!(two.serialize_specified().unwrap(), "1px 2%");
    assert_eq!(
        two.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 6))
            .unwrap(),
        "1px 2%"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, 6),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, 6),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, 5),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            two.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}
