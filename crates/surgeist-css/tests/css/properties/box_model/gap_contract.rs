//! Public specified-value contracts from CSS Box Alignment 3 WD 2026-01-30
//! §§8.1, 8.2, and 8.4.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::property_admission::assert_declaration_rejected as rejected;
use crate::css::support::{
    checked_grammar_declaration as checked, important_declaration as parsed,
};
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValueRef, CssComponentValues, CssValueTokenRef};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::numeric::{CssCalculation, CssLengthPercentage, CssNonNegative, CssNumeric};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssKnownProperty, CssKnownPropertyValueRef, CssPropertyGrammar,
    CssPropertyKindRef, CssPropertyNameRef,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::common::CssSidePair;
use surgeist_css::values::layout::CssGapValue;

fn scalar(text: &str) -> CssNumeric<CssLengthPercentage, CssNonNegative> {
    let components =
        CssComponentValues::parse(&CssParser::default(), text).expect("valid component syntax");
    if text.starts_with("calc(") {
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(
            CssCalculation::<CssLengthPercentage>::try_from_components(
                components,
                Default::default(),
            )
            .expect("length-percentage math"),
        )
        .expect("checked deferred math")
    } else {
        let [component] = components.items() else {
            panic!("one scalar: {text}")
        };
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(component.clone())
            .expect("checked nonnegative scalar")
    }
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

fn gap_value(declaration: &CssDeclaration) -> CssSidePair<CssGapValue> {
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
    assert_eq!(
        CssGapValue::Normal
            .serialize_specified(Default::default())
            .unwrap(),
        "normal"
    );
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
            assert_eq!(
                direct.serialize_specified(Default::default()).unwrap(),
                canonical,
                "{text}"
            );
            if text == "1.0000000000000000000001px" {
                let CssGapValue::LengthPercentage(scalar) = &direct else {
                    panic!("numeric gap")
                };
                assert!(matches!(
                    scalar.literal().map(|literal| literal.component()).unwrap().view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
                        if number.representation() == "1.0000000000000000000001" && unit == "px"
                ));
            }
            for source in [parsed(name, text), checked(name, text)] {
                let actual = extract(&source);
                assert_eq!(actual, direct, "{name}:{text}");
                assert_eq!(
                    actual.serialize_specified(Default::default()).unwrap(),
                    canonical
                );
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
            assert_eq!(
                actual
                    .first()
                    .serialize_specified(Default::default())
                    .unwrap(),
                row
            );
            assert_eq!(
                actual
                    .second()
                    .serialize_specified(Default::default())
                    .unwrap(),
                column
            );
            assert_eq!(actual.authored_second().is_some(), has_authored_column);
            assert_eq!(
                actual.serialize_specified(Default::default()).unwrap(),
                text
            );
        }
    }

    let omitted = CssSidePair::new(CssGapValue::LengthPercentage(scalar("1px")), None);
    let explicit = CssSidePair::new(
        CssGapValue::LengthPercentage(scalar("1px")),
        Some(CssGapValue::LengthPercentage(scalar("1px"))),
    );
    assert_eq!(omitted.first(), omitted.second());
    assert!(omitted.authored_second().is_none());
    assert_eq!(
        omitted.serialize_specified(Default::default()).unwrap(),
        "1px"
    );
    assert_eq!(
        explicit.serialize_specified(Default::default()).unwrap(),
        "1px"
    );
    assert_ne!(omitted, explicit);
    assert_ne!(omitted, CssSidePair::new(CssGapValue::Normal, None));
}

#[test]
fn negative_literals_and_non_gap_tokens_are_invalid_but_math_remains_symbolic() {
    for name in ["row-gap", "column-gap", "gap", "grid-gap"] {
        // A gap shorthand admits two components; each longhand admits one.
        let shorthand = matches!(name, "gap" | "grid-gap");
        for (value, longhand_token, shorthand_token) in [
            ("-1px", "-1px", "-1px"),
            ("-1e-100%", "-1e-100%", "-1e-100%"),
            ("1fr", "1fr", "1fr"),
            ("1", "1", "1"),
            ("auto", "auto", "auto"),
            ("1px 2px 3px", "2px", "3px"),
        ] {
            let token = if shorthand {
                shorthand_token
            } else {
                longhand_token
            };
            rejected(name, value, Some(token));
        }
    }
    for name in ["row-gap", "column-gap", "gap"] {
        for value in ["calc(1px + 2%)", "calc(1px - 2px)"] {
            assert!(
                CssParser::default()
                    .parse_block_contents(format!("{name}:{value}"))
                    .is_clean(),
                "{name}:{value}"
            );
            checked(name, value);
        }
    }
    let deferred = CssGapValue::LengthPercentage(scalar("calc(1px - 2px)"));
    assert!(deferred.serialize_specified(Default::default()).is_ok());
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
        source.expand().expect("all expands symbolically")
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
            .serialize_specified(CssSerializationLimits::new(1, 1, 6))
            .unwrap(),
        "normal"
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(0, 1, 6),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 0, 6),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 1, 5),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&normal.serialize_specified(limits).unwrap_err()),
            kind
        );
    }

    let one = CssSidePair::new(CssGapValue::Normal, None);
    assert_eq!(
        one.serialize_specified(Default::default()).unwrap(),
        "normal"
    );
    let two = CssSidePair::new(
        CssGapValue::LengthPercentage(scalar("1px")),
        Some(CssGapValue::LengthPercentage(scalar("2%"))),
    );
    assert_eq!(
        two.serialize_specified(Default::default()).unwrap(),
        "1px 2%"
    );
    assert_eq!(
        two.serialize_specified(CssSerializationLimits::new(3, 3, 6))
            .unwrap(),
        "1px 2%"
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(2, 3, 6),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 6),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 3, 5),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&two.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
}
