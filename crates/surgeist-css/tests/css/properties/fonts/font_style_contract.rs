//! Authored Fonts 4 property, shared angle, and shorthand contracts.

use crate::css::support::named_property::assert_programmatic_rejection;
use crate::css::support::named_property::normal_grammar_components;
use surgeist_css::{
    CssAngle, CssCalculation, CssComponentValue, CssComponentValueRef, CssComponentValues,
    CssContributions, CssExpansion, CssFontStyle, CssFontStyleKeyword, CssFontValue, CssImportance,
    CssInitialValueRef, CssKnownProperty, CssKnownPropertyValueRef, CssLonghandValueRef,
    CssNumeric, CssObliqueAngleRange, CssPropertyKindRef, CssRecoveryAction,
    CssSerializationLimits, CssSerializedOrigin, CssValueOrigin, CssValueTokenRef,
    expand_declaration, parse_component_values, parse_style_attribute,
};

fn angle(text: &str) -> CssNumeric<CssAngle, CssObliqueAngleRange> {
    CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_component(
        CssComponentValue::try_token(text).unwrap(),
    )
    .unwrap()
}

fn property(text: &str) -> CssFontStyle {
    let report = parse_style_attribute(&format!("font-style:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::FontStyle(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("font-style wrapper")
    };
    value.value().clone()
}

#[test]
fn keywords_and_optional_oblique_angle_preserve_authored_distinctions() {
    for (text, expected) in [
        ("normal", CssFontStyleKeyword::Normal),
        ("italic", CssFontStyleKeyword::Italic),
        ("left", CssFontStyleKeyword::Left),
        ("right", CssFontStyleKeyword::Right),
    ] {
        assert_eq!(property(text), CssFontStyle::Keyword(expected));
        assert_eq!(
            CssFontStyle::Keyword(expected)
                .serialize_specified(Default::default())
                .unwrap(),
            text
        );
    }
    assert_eq!(property("oblique"), CssFontStyle::Oblique { angle: None });
    assert_eq!(
        property("oblique")
            .serialize_specified(Default::default())
            .unwrap(),
        "oblique"
    );
    let explicit = property("oblique 14deg");
    assert_eq!(
        explicit,
        CssFontStyle::Oblique {
            angle: Some(angle("14deg"))
        }
    );
    assert_ne!(explicit, property("oblique"));
    // Fonts 4 (WD 2026-09-07) §2.4:"The lack of an <angle> represents 14deg".
    // CSSOM (editor capture 2026-10-09) "serialize a CSS value": omit component
    // values whose omission does not change the meaning. The retained model above
    // still distinguishes the explicit angle from its omission. Witness
    // divergence: Chromium 152 (2026-10-21) keeps `oblique 14deg`; the spec governs.
    for text in ["oblique 14deg", "oblique 14.0deg", "oblique +1.4e1DEG"] {
        assert_eq!(
            property(text)
                .serialize_specified(Default::default())
                .unwrap(),
            "oblique"
        );
    }
    for (text, expected) in [
        ("oblique 15deg", "oblique 15deg"),
        ("oblique -14deg", "oblique -14deg"),
        // 14deg is a nonterminating number of gradians or turns.
        ("oblique 15.5556grad", "oblique 15.5556grad"),
    ] {
        assert_eq!(
            property(text)
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
    }
}

#[test]
fn exact_lexical_bounds_and_radian_precision_retain_original_components() {
    for text in [
        "90deg",
        "-90deg",
        "+90deg",
        "100grad",
        "-100grad",
        ".25turn",
        "-.25turn",
        "1e-999deg",
        "-1e-999grad",
        "1e-999rad",
        "0e999999999999999999999999rad",
        "-0deg",
        "+0turn",
        "1.5707963267948966rad",
        "-1.5707963267948966rad",
    ] {
        let value = angle(text);
        assert!(
            value.literal().map(|literal| literal.component()).is_some(),
            "{text}"
        );
        assert!(value.calculation().is_none(), "{text}");
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic, "{text}");
        assert!(
            property(&format!("oblique {text}"))
                .serialize_specified(Default::default())
                .unwrap()
                .starts_with("oblique "),
            "{text}"
        );
    }
    for text in [
        "90.0000000000000000000000000000001deg",
        "-90.0000000000000000000000000000001deg",
        "100.0000000000000000000000000000001grad",
        ".2500000000000000000000000000001turn",
        "1e999deg",
        "1e999rad",
        "1.5707963267948968rad",
        "-1.5707963267948968rad",
    ] {
        let error = CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_component(
            CssComponentValue::try_token(text).unwrap(),
        )
        .unwrap_err();
        assert!(error.is_out_of_range(), "{text}");
    }
    let rad = angle("1.5707963267948966rad");
    let component = rad.literal().map(|literal| literal.component()).unwrap();
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) =
        component.view()
    else {
        panic!("retained dimension")
    };
    assert_eq!(number.representation(), "1.5707963267948966");
    assert_eq!(unit, "rad");
    assert_eq!(
        rad.serialize_specified(Default::default()).unwrap(),
        "1.570796rad"
    );
    let same_binary64 = angle("1.5707963267948965rad");
    assert_eq!(
        "1.5707963267948965".parse::<f64>().unwrap(),
        "1.5707963267948966".parse::<f64>().unwrap(),
    );
    assert_ne!(rad, same_binary64);
    assert_eq!(
        same_binary64
            .serialize_specified(Default::default())
            .unwrap(),
        "1.570796rad"
    );
    let long = angle("0.0000000000000000000000000000000000000001turn");
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) = long
        .literal()
        .map(|literal| literal.component())
        .unwrap()
        .view()
    else {
        panic!("retained tiny dimension")
    };
    assert_eq!(
        number.representation(),
        "0.0000000000000000000000000000000000000001"
    );
    assert_eq!(unit, "turn");
}

#[test]
fn oblique_angles_compare_their_exact_decimal_with_the_quarter_turn_bound() {
    // Fonts 4 WD 2026-09-07 §3.3: `oblique <angle [-90deg,90deg]>?`. A quarter
    // turn is 90deg = 100grad = 0.25turn = π/2 rad, and
    // π/2 = 1.57079632679489661923132169163975144209858469968755291048747…
    // The first two radian rejections round to the same binary64 value as
    // accepted literals; the authored decimal, not binary64, decides.
    const HALF_PI_100: &str = "1.570796326794896619231321691639751442098584699687552910487472296153908203143104499314017412671058533";
    let accepted = [
        "90deg".to_owned(),
        "-90deg".to_owned(),
        "89.9999999999999999999deg".to_owned(),
        "100grad".to_owned(),
        "-100grad".to_owned(),
        "99.99999999999999999grad".to_owned(),
        "0.25turn".to_owned(),
        "-0.25turn".to_owned(),
        "0.2499999999999999999turn".to_owned(),
        "1.5707963267948966rad".to_owned(),
        "-1.5707963267948966rad".to_owned(),
        "1.57079632679489661923rad".to_owned(),
        "-1.57079632679489661923rad".to_owned(),
        "15707963267948966e-16rad".to_owned(),
        "0.0157079632679489661923e2rad".to_owned(),
        format!("{HALF_PI_100}rad"),
        format!("-{HALF_PI_100}rad"),
    ];
    for text in &accepted {
        assert!(
            angle(text)
                .literal()
                .map(|literal| literal.component())
                .is_some(),
            "{text}"
        );
        assert!(
            property(&format!("oblique {text}"))
                .serialize_specified(Default::default())
                .unwrap()
                .starts_with("oblique "),
            "{text}"
        );
    }
    let rejected = [
        "1.57079632679489665rad".to_owned(),
        "-1.57079632679489665rad".to_owned(),
        "1.57079632679489661924rad".to_owned(),
        "-1.57079632679489661924rad".to_owned(),
        "157079632679489662e-17rad".to_owned(),
        // One unit above π/2 truncated to 100 significant digits.
        format!("{}4rad", &HALF_PI_100[..HALF_PI_100.len() - 1]),
        format!("-{}4rad", &HALF_PI_100[..HALF_PI_100.len() - 1]),
        "90.0000000000001deg".to_owned(),
        "-90.0000000000001deg".to_owned(),
        "100.000000000001grad".to_owned(),
        "-100.000000000001grad".to_owned(),
        "0.2500000000001turn".to_owned(),
        "-0.2500000000001turn".to_owned(),
    ];
    for text in &rejected {
        let error = CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_component(
            CssComponentValue::try_token(text).unwrap(),
        )
        .unwrap_err();
        assert!(error.is_out_of_range(), "{text}");
        let report = parse_style_attribute(&format!("font-style:oblique {text}"));
        assert!(!report.is_clean(), "{text}");
    }
}

#[test]
fn wrong_domains_and_bare_math_roots_cannot_enter_the_angle_model() {
    for text in ["0", "10", "10%", "10px", "10frobs"] {
        let error = CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_component(
            CssComponentValue::try_token(text).unwrap(),
        )
        .unwrap_err();
        assert!(error.is_root_domain_mismatch(), "{text}");
    }
    for text in ["0deg", "91deg"] {
        let calculation = CssCalculation::<CssAngle>::try_from_components(
            parse_component_values(text).unwrap(),
            Default::default(),
        )
        .unwrap();
        let result =
            CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_calculation(calculation);
        assert_eq!(result.is_ok(), text == "0deg", "{text}");
    }
    for text in ["calc(91deg)", "calc(-100deg)"] {
        let calculation = CssCalculation::<CssAngle>::try_from_components(
            parse_component_values(text).unwrap(),
            Default::default(),
        )
        .unwrap();
        let value = CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_calculation(calculation)
            .unwrap();
        assert!(value.calculation().is_some(), "{text}");
        assert!(
            value.literal().map(|literal| literal.component()).is_none(),
            "{text}"
        );
        assert_eq!(value.serialize_specified(Default::default()).unwrap(), text);
        assert_eq!(
            property(&format!("oblique {text}")),
            CssFontStyle::Oblique { angle: Some(value) }
        );
    }
    for text in [
        "oblique 0",
        "oblique 1px",
        "oblique 1%",
        "oblique calc(10px)",
        "oblique 10deg 20deg",
    ] {
        let report = parse_style_attribute(&format!("font-style:{text}"));
        assert!(report.syntax().is_empty(), "{text}");
        assert_eq!(report.diagnostics().len(), 1, "{text}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration,
            "{text}"
        );
    }
}

#[test]
fn recovered_math_is_retained_by_parser_admission_and_rejected_by_typed_transfer() {
    let report = parse_style_attribute("font-style:oblique calc(10deg");
    assert_eq!(report.syntax().len(), 1);
    assert!(!report.is_clean());
    let CssKnownPropertyValueRef::FontStyle(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("retained style")
    };
    let CssFontStyle::Oblique {
        angle: Some(admitted),
    } = value.value()
    else {
        panic!("retained oblique math")
    };
    let calculation = admitted.calculation().expect("typed recovered calculation");
    let error = CssCalculation::<CssAngle>::try_from_components(
        calculation.components().clone(),
        Default::default(),
    )
    .unwrap_err();
    assert!(error.is_recovered_component(), "{error:?}");
    // The public checked constructor rejects the recovered math that only
    // parser admission retains.
    let error =
        CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_calculation(calculation.clone())
            .unwrap_err();
    assert!(error.is_recovered_component(), "{error:?}");
    assert_eq!(
        admitted.serialize_specified(Default::default()).unwrap(),
        "calc(10deg)"
    );
}

#[test]
fn checked_values_and_pending_reentry_preserve_real_origins_and_importance() {
    let parsed = normal_grammar_components(
        "font-style",
        parse_component_values("oblique 12.50deg ").unwrap(),
    )
    .unwrap();
    let CssKnownPropertyValueRef::FontStyle(value) =
        parsed.known().unwrap().property_value().unwrap()
    else {
        panic!("style")
    };
    let CssFontStyle::Oblique { angle: Some(value) } = value.value() else {
        panic!("angle")
    };
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("parsed token")
    };
    assert_eq!(origin.source().as_str(), "oblique 12.50deg ");
    assert_eq!(origin.span().start().byte_offset().value(), 8);
    assert_eq!(origin.span().end().byte_offset().value(), 16);

    let programmatic = normal_grammar_components(
        "font-style",
        CssComponentValues::try_new(
            vec![
                CssComponentValue::try_ident("oblique").unwrap(),
                CssComponentValue::try_token("12deg").unwrap(),
            ],
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap();
    let CssKnownPropertyValueRef::FontStyle(value) =
        programmatic.known().unwrap().property_value().unwrap()
    else {
        panic!("style")
    };
    let CssFontStyle::Oblique { angle: Some(value) } = value.value() else {
        panic!("angle")
    };
    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);

    let invalid = normal_grammar_components(
        "font-style",
        CssComponentValues::try_new(
            vec![
                CssComponentValue::try_ident("oblique").unwrap(),
                CssComponentValue::try_token("91deg").unwrap(),
            ],
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap_err();
    assert!(invalid.code().is_some());
    assert_eq!(
        invalid.origin(),
        CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
    let adjacent = CssComponentValues::try_new(
        vec![
            CssComponentValue::try_ident("oblique").unwrap(),
            CssComponentValue::try_number("1").unwrap(),
            CssComponentValue::try_token("0deg").unwrap(),
        ],
        Default::default(),
    )
    .unwrap();
    assert_programmatic_rejection(
        normal_grammar_components("font-style", adjacent),
        CssKnownProperty::FontStyle,
    );

    let report = parse_style_attribute("font-style:env(style)!important");
    let source = &report.syntax()[0];
    let CssExpansion::Pending(handle) = expand_declaration(source).unwrap() else {
        panic!("pending property")
    };
    let CssContributions::Longhands(values) = handle
        .reenter(parse_component_values("oblique -20grad").unwrap())
        .unwrap()
    else {
        panic!("reentry")
    };
    let [item] = values.items() else {
        panic!("one contribution")
    };
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    let CssLonghandValueRef::FontStyle(CssFontStyle::Oblique { angle: Some(value) }) =
        item.ordinary_value().unwrap().view()
    else {
        panic!("typed contribution")
    };
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "-20grad"
    );
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("parsed replacement angle")
    };
    assert_eq!(origin.source().as_str(), "oblique -20grad");
    assert_eq!(origin.span().start().byte_offset().value(), 8);
    assert_eq!(origin.span().end().byte_offset().value(), 15);
    let error = handle
        .reenter(
            CssComponentValues::try_new(
                vec![
                    CssComponentValue::try_ident("oblique").unwrap(),
                    CssComponentValue::try_token("91deg").unwrap(),
                ],
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap_err();
    let Some(mapped) = error.replacement_error() else {
        panic!("invalid angle replacement")
    };
    assert!(mapped.code().is_some());
    assert_eq!(
        mapped.origin(),
        CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
}

#[test]
fn initial_metadata_css_wide_and_shorthand_ambiguity_are_typed() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::FontStyle.metadata().unwrap().kind()
    else {
        panic!("longhand")
    };
    assert!(metadata.inherited_by_default());
    let initial_value = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("ordinary initial")
    };
    assert_eq!(
        initial.view(),
        CssLonghandValueRef::FontStyle(&CssFontStyle::Keyword(CssFontStyleKeyword::Normal))
    );
    for source in ["font-style:inherit", "font-style:unset!important"] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1);
    }
    for (source, expected_angle) in [
        ("font:oblique 10deg 16px serif", Some("10deg")),
        ("font:oblique calc(10deg) 16px serif", Some("calc(10deg)")),
        ("font:oblique calc(16px) serif", None),
        ("font:oblique calc(700) calc(16px) serif", None),
    ] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::Font(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("font")
        };
        let CssFontValue::Explicit(font) = value.font() else {
            panic!("explicit font")
        };
        let Some(CssFontStyle::Oblique { angle }) = font.style() else {
            panic!("oblique style")
        };
        assert_eq!(
            angle
                .as_ref()
                .map(|angle| angle.serialize_specified(Default::default()).unwrap())
                .as_deref(),
            expected_angle
        );
    }
    for source in [
        "font:italic oblique 16px serif",
        "font:oblique 10deg italic 16px serif",
    ] {
        assert!(!parse_style_attribute(source).is_clean(), "{source}");
    }
}

#[test]
fn scalar_and_property_serialization_charge_budgets_atomically() {
    let value = property("oblique 10deg");
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "oblique 10deg"
    );
    assert!(
        value
            .serialize_specified(CssSerializationLimits::new(8, 8, 4))
            .unwrap_err()
            .is_byte_limit()
    );
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "oblique 10deg"
    );
    let value = angle("-10grad");
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "-10grad"
    );
    assert!(
        value
            .serialize_specified(CssSerializationLimits::new(8, 8, 3))
            .unwrap_err()
            .is_byte_limit()
    );
}
