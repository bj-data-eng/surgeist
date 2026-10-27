//! Exact authored font-width contracts from CSS Fonts 4 WD 2026-09-07 §§2.3, 2.7.

use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::numeric_failure;
use crate::css::support::failure_classes::serialization_failure;
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValueRef, CssComponentValues, CssValueOrigin, CssValueTokenRef,
};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::numeric::{CssCalculation, CssNonNegative, CssNumeric, CssPercentage};
use surgeist_css::property::{CssKnownPropertyValueRef, CssPropertyGrammar, CssPropertyTarget};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::font::{CssFontWidth, CssFontWidthKeyword};

fn scalar(text: &str) -> CssNumeric<CssPercentage, CssNonNegative> {
    let values = CssComponentValues::parse(&CssParser::default(), text).unwrap();
    let [component] = values.items() else {
        panic!("one percentage: {text}")
    };
    CssNumeric::<CssPercentage, CssNonNegative>::try_from_component(component.clone()).unwrap()
}

fn ordinary_percentage_spelling(value: &CssFontWidth) -> &str {
    let CssFontWidth::Percentage(value) = value else {
        panic!("percentage width")
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) = value
        .literal()
        .map(|literal| literal.component())
        .expect("ordinary percentage")
        .view()
    else {
        panic!("percentage token")
    };
    number.representation()
}

fn width(name: &str, value: &str, checked: bool) -> CssFontWidth {
    let declaration = if checked {
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(CssPropertyGrammar::from_name(name).unwrap()),
            CssComponentValues::parse(&CssParser::default(), value).unwrap(),
            CssImportance::Normal,
        )
        .unwrap()
    } else {
        let report = CssParser::default().parse_block_contents(format!("{name}:{value}"));
        assert!(
            report.is_clean(),
            "{name}:{value}: {:?}",
            report.diagnostics()
        );
        report.syntax()[0].clone()
    };
    let CssKnownPropertyValueRef::FontWidth(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("font-width typed wrapper")
    };
    wrapper.value().clone()
}

#[test]
fn nine_keywords_have_the_fonts_four_exact_percentage_mapping() {
    use CssFontWidthKeyword::*;
    for (keyword, spelling, percentage) in [
        (UltraCondensed, "ultra-condensed", "50%"),
        (ExtraCondensed, "extra-condensed", "62.5%"),
        (Condensed, "condensed", "75%"),
        (SemiCondensed, "semi-condensed", "87.5%"),
        (Normal, "normal", "100%"),
        (SemiExpanded, "semi-expanded", "112.5%"),
        (Expanded, "expanded", "125%"),
        (ExtraExpanded, "extra-expanded", "150%"),
        (UltraExpanded, "ultra-expanded", "200%"),
    ] {
        assert_eq!(
            keyword
                .percentage()
                .serialize_specified(Default::default())
                .unwrap(),
            percentage
        );
        let direct = CssFontWidth::Keyword(keyword);
        assert_eq!(
            direct.serialize_specified(Default::default()).unwrap(),
            spelling
        );
        for name in ["font-width", "font-stretch"] {
            for checked in [false, true] {
                assert_eq!(width(name, spelling, checked), direct);
            }
        }
        assert_ne!(direct, CssFontWidth::Percentage(scalar(percentage)));
    }
}

#[test]
fn percentage_constructor_preserves_exact_literals_and_rejects_invalid_roots() {
    for (text, canonical) in [
        ("0%", "0%"),
        ("-0%", "0%"),
        ("62.500000000000000000001%", "62.5%"),
        ("999999999999999999999999%", "999999999999999999999999%"),
    ] {
        let direct = CssFontWidth::Percentage(scalar(text));
        assert_eq!(
            direct.serialize_specified(Default::default()).unwrap(),
            canonical
        );
        for name in ["font-width", "font-stretch"] {
            for checked in [false, true] {
                let actual = width(name, text, checked);
                assert_eq!(actual, direct, "{name}:{text}");
                assert_eq!(
                    ordinary_percentage_spelling(&actual),
                    text.trim_end_matches('%')
                );
                assert_eq!(
                    actual.serialize_specified(Default::default()).unwrap(),
                    canonical,
                    "{name}:{text}"
                );
            }
        }
    }
    assert_ne!(scalar("62.500000000000000000001%"), scalar("62.5%"));
    assert_ne!(
        scalar("999999999999999999999999%"),
        scalar("999999999999999999999998%")
    );
    // A number or length is outside the <percentage> root; a negative
    // percentage is outside [0,inf].
    for (text, kind) in [
        ("0", NumericFailure::RootDomainMismatch),
        ("1", NumericFailure::RootDomainMismatch),
        ("1px", NumericFailure::RootDomainMismatch),
        ("-1%", NumericFailure::OutOfRange),
        ("-1e-999%", NumericFailure::OutOfRange),
    ] {
        let values = CssComponentValues::parse(&CssParser::default(), text).unwrap();
        let [component] = values.items() else {
            panic!("one invalid scalar")
        };
        let error =
            CssNumeric::<CssPercentage, CssNonNegative>::try_from_component(component.clone())
                .unwrap_err();
        assert_eq!(numeric_failure(&error), kind, "{text}");
        assert_eq!(error.origin(), Some(component.origin()), "{text}");
    }
    for text in ["1e999%", "1e-999%"] {
        let direct = CssFontWidth::Percentage(scalar(text));
        assert_eq!(width("font-width", text, false), direct);
        assert_eq!(width("font-stretch", text, true), direct);
        assert_eq!(
            ordinary_percentage_spelling(&width("font-width", text, false)),
            text.trim_end_matches('%')
        );
        assert_eq!(
            ordinary_percentage_spelling(&width("font-stretch", text, true)),
            text.trim_end_matches('%')
        );
    }
}

#[test]
fn bare_percentage_calculation_roots_reenter_exact_range_admission() {
    for text in ["-1%", "-1e-999%"] {
        let typed = CssCalculation::<CssPercentage>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), text).unwrap(),
            Default::default(),
        )
        .expect("negative percentage is in the percentage calculation domain");
        assert!(
            CssNumeric::<CssPercentage, CssNonNegative>::try_from_calculation(typed)
                .unwrap_err()
                .is_out_of_range(),
            "{text}"
        );
    }
    let positive = CssNumeric::<CssPercentage, CssNonNegative>::try_from_calculation(
        CssCalculation::<CssPercentage>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), "125%").unwrap(),
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        positive
            .literal()
            .map(|literal| literal.component())
            .is_some()
    );
    assert!(positive.calculation().is_none());
    assert_eq!(
        positive.serialize_specified(Default::default()).unwrap(),
        "125%"
    );
}

#[test]
fn parsed_percentages_keep_source_origin_and_symbolic_math_unresolved() {
    let report =
        CssParser::default().parse_block_contents("FoNt-StReTcH:62.500000000000000000001%");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::FontWidth(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("font-width wrapper for legacy name")
    };
    let CssFontWidth::Percentage(parsed) = wrapper.value() else {
        panic!("exact parsed percentage")
    };
    assert!(matches!(parsed.origin(), CssValueOrigin::Parsed(_)));
    assert!(matches!(
        scalar("62.500000000000000000001%").origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        parsed.serialize_specified(Default::default()).unwrap(),
        "62.5%"
    );

    assert!(matches!(
        parsed.literal().map(|literal| literal.component()).unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
            if number.representation() == "62.500000000000000000001"
    ));
    for text in ["calc(25% + 50%)", "calc(25% - 50%)"] {
        let calculation = CssCalculation::<CssPercentage>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), text).unwrap(),
            Default::default(),
        )
        .unwrap();
        let exact =
            CssNumeric::<CssPercentage, CssNonNegative>::try_from_calculation(calculation).unwrap();
        assert!(exact.calculation().is_some());
        assert!(exact.literal().map(|literal| literal.component()).is_none());
        let value = CssFontWidth::Percentage(exact);
        assert!(value.serialize_specified(Default::default()).is_ok());
        for checked in [false, true] {
            let parsed = width("font-width", text, checked);
            assert_eq!(parsed, value);
            let CssFontWidth::Percentage(parsed) = parsed else {
                panic!("symbolic percentage width")
            };
            assert!(parsed.calculation().is_some());
            assert_eq!(
                parsed.serialize_specified(Default::default()).unwrap(),
                value.serialize_specified(Default::default()).unwrap()
            );
        }
    }
}

#[test]
fn scalar_and_width_serialization_enforce_input_projection_and_byte_budgets() {
    let value = CssFontWidth::Percentage(scalar("75%"));
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "75%"
    );
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(1, 1, 3))
            .unwrap(),
        "75%"
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(0, 1, 3),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 0, 3),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 1, 2),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
    let scalar = scalar("75%");
    assert_eq!(
        scalar
            .serialize_specified(CssSerializationLimits::new(1, 1, 3))
            .unwrap(),
        "75%"
    );
    assert!(
        scalar
            .serialize_specified(CssSerializationLimits::new(1, 1, 2))
            .unwrap_err()
            .is_byte_limit()
    );
}
