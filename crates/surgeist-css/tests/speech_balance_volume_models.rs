#![forbid(unsafe_code)]
//! Functional Speech 1 §§6.1–6.2 authored mixing models. Computed clamping,
//! inherited adjustments, user calibration and synthesis stay downstream.
//! Existing-public RED is preserved in speech_balance_volume_lifecycle.rs.

use surgeist_css::*;

fn declaration(name: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{text}!important"));
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    report.syntax()[0].clone()
}
fn balance(source: &CssDeclaration) -> &CssVoiceBalance {
    let CssKnownPropertyValueRef::VoiceBalance(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("balance wrapper")
    };
    value.value()
}
fn volume(source: &CssDeclaration) -> &CssVoiceVolume {
    let CssKnownPropertyValueRef::VoiceVolume(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("volume wrapper")
    };
    value.value()
}
fn number(text: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_component(CssComponentValue::try_number(text).unwrap()).unwrap()
}
fn level(level: CssVoiceVolumeLevel, offset: Option<&str>) -> CssVoiceVolume {
    CssVoiceVolume::Level {
        level,
        decibel: offset.map(|value| CssDecibelLiteral::try_new(value).unwrap()),
    }
}

#[test]
fn balance_keywords_remain_authored_separately_from_numeric_equivalents() {
    for (keyword, text, numeric) in [
        (CssVoiceBalanceKeyword::Left, "left", Some("-100")),
        (CssVoiceBalanceKeyword::Center, "center", Some("0")),
        (CssVoiceBalanceKeyword::Right, "right", Some("100")),
        (CssVoiceBalanceKeyword::Leftwards, "leftwards", None),
        (CssVoiceBalanceKeyword::Rightwards, "rightwards", None),
    ] {
        let value = CssVoiceBalance::from_keyword(keyword);
        assert_eq!(value.keyword(), Some(keyword));
        assert!(value.number().is_none());
        assert_eq!(value.serialize_specified().unwrap(), text);
        assert_eq!(balance(&declaration("voice-balance", text)), &value);
        if let Some(numeric) = numeric {
            assert_ne!(value, CssVoiceBalance::try_number(number(numeric)).unwrap());
        }
    }
}

#[test]
fn balance_numbers_preserve_exact_unrestricted_coefficients_and_original_components() {
    for text in [
        "-101",
        "101",
        "-0",
        "+0",
        "9007199254740993",
        "1e400",
        "-1e-400",
    ] {
        let source = declaration("voice-balance", text);
        let value = balance(&source);
        assert!(value.keyword().is_none());
        let literal = value.number().unwrap().literal_component().unwrap();
        assert!(source.value_components().items().contains(literal));
        let CssComponentValueRef::Token(CssValueTokenRef::Number(numeric)) = literal.view() else {
            panic!("pure number token")
        };
        assert_eq!(numeric.representation(), text);
        assert_eq!(value.number().unwrap().origin(), literal.origin());
        let retained = value.clone();
        assert_eq!(
            value.serialize_specified().unwrap(),
            value.number().unwrap().serialize_specified().unwrap()
        );
        assert_eq!(value, &retained);
    }
    assert_eq!(
        CssVoiceBalance::try_number(number("101"))
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "101"
    );
    assert_eq!(
        CssVoiceBalance::try_number(number("-101"))
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "-101"
    );
}

#[test]
fn balance_math_uses_only_pure_number_typing_and_keeps_its_authored_tree() {
    for text in [
        "calc(101)",
        "calc(-100 - 20)",
        "min(101, 200)",
        "calc(1px / 1px)",
        "calc(infinity)",
        "calc(NaN)",
    ] {
        let source = declaration("voice-balance", text);
        let value = balance(&source);
        let numeric = value.number().unwrap();
        assert!(numeric.literal_component().is_none());
        let calculation = numeric.calculation().unwrap();
        assert_eq!(calculation.components().serialize().unwrap().as_css(), text);
        assert_eq!(
            value.serialize_specified().unwrap(),
            numeric.serialize_specified().unwrap()
        );
        let checked = CssVoiceBalance::try_number(numeric.clone()).unwrap();
        assert_eq!(&checked, value);
    }
    for text in ["calc(1%)", "calc(1 + 1%)", "calc(1px)", "calc(1db)"] {
        assert!(
            CssNumberCalculation::try_from_components(parse_component_values(text).unwrap())
                .is_err(),
            "{text}"
        );
        let report = parse_style_attribute(&format!("voice-balance:{text}"));
        assert!(report.syntax().is_empty());
        assert!(!report.is_clean());
    }
}

#[test]
fn balance_recovered_math_is_retained_with_diagnostics_but_checked_construction_and_reentry_reject_it()
 {
    let source = "voice-balance:calc(101";
    let report = parse_style_attribute(source);
    let [declaration] = report.syntax().as_slice() else {
        panic!("retained recovered Number math")
    };
    assert!(!report.is_clean());
    assert_eq!(
        validate_style_attribute(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let value = balance(declaration);
    let numeric = value.number().unwrap();
    let error = CssVoiceBalance::try_number(numeric.clone()).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::RecoveredComponent
    );
    assert!(matches!(
        error.origin(),
        Some(CssValueOrigin::ImplicitClosure { .. })
    ));
    let components = declaration.value_components().clone();
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::VoiceBalance),
            components.clone(),
            CssImportance::Normal
        )
        .is_err()
    );
    let pending = self::declaration("voice-balance", "var(--balance)");
    let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
        panic!("pending")
    };
    assert!(matches!(
        handle.reenter(components).unwrap_err().kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    assert!(
        handle
            .reenter(parse_component_values("101").unwrap())
            .is_ok()
    );
}

#[test]
fn volume_models_encode_silent_level_and_standalone_offset_without_inserting_defaults() {
    assert_eq!(
        volume(&declaration("voice-volume", "silent")),
        &CssVoiceVolume::Silent
    );
    for (keyword, text) in [
        (CssVoiceVolumeLevel::XSoft, "x-soft"),
        (CssVoiceVolumeLevel::Soft, "soft"),
        (CssVoiceVolumeLevel::Medium, "medium"),
        (CssVoiceVolumeLevel::Loud, "loud"),
        (CssVoiceVolumeLevel::XLoud, "x-loud"),
    ] {
        let plain = level(keyword, None);
        assert_eq!(volume(&declaration("voice-volume", text)), &plain);
        assert_eq!(plain.serialize_specified().unwrap(), text);
        for authored in [format!("{text} -6dB"), format!("-6dB {text}")] {
            let source = declaration("voice-volume", &authored);
            let CssVoiceVolume::Level { level, decibel } = volume(&source) else {
                panic!("level+offset")
            };
            assert_eq!(*level, keyword);
            let offset = decibel.as_ref().unwrap();
            assert_eq!(offset.numeric().representation(), "-6");
            assert!(
                source
                    .value_components()
                    .items()
                    .contains(offset.component())
            );
            assert_eq!(
                volume(&source).serialize_specified().unwrap(),
                format!("{text} -6db")
            );
        }
    }
    let bare = declaration("voice-volume", "0db");
    assert!(matches!(volume(&bare), CssVoiceVolume::Offset(_)));
    assert_ne!(volume(&bare), &level(CssVoiceVolumeLevel::Medium, None));
    assert_eq!(volume(&bare).serialize_specified().unwrap(), "0db");
}

#[test]
fn volume_offsets_reuse_exact_decibel_storage_and_shared_specified_precision() {
    for text in [
        "-6dB",
        "+6DB",
        "-0db",
        "9007199254740993dB",
        "1e400dB",
        "-1e-400dB",
        r"-3d\42",
    ] {
        let source = declaration("voice-volume", text);
        let CssVoiceVolume::Offset(offset) = volume(&source) else {
            panic!("bare offset")
        };
        assert!(
            source
                .value_components()
                .items()
                .contains(offset.component())
        );
        assert_eq!(
            volume(&source).serialize_specified().unwrap(),
            offset.serialize_specified().unwrap()
        );
        assert_eq!(offset.component().origin(), offset.origin());
    }
    let tiny = CssDecibelLiteral::try_new("-1e-400").unwrap();
    let value = CssVoiceVolume::Offset(tiny.clone());
    assert_eq!(value.serialize_specified().unwrap(), "0db");
    assert_eq!(tiny.numeric().representation(), "-1e-400");
    let with_level = CssVoiceVolume::Level {
        level: CssVoiceVolumeLevel::Soft,
        decibel: Some(tiny),
    };
    assert_eq!(with_level.serialize_specified().unwrap(), "soft 0db");
}

#[test]
fn volume_omits_only_exact_zero_companions_without_losing_authored_fields() {
    for text in ["0", "-0", "+0.00e9999999999999999999999999999999999999"] {
        let value = level(CssVoiceVolumeLevel::Loud, Some(text));
        let retained = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), "loud");
        let CssVoiceVolume::Level { decibel, .. } = &value else {
            panic!("level")
        };
        assert_eq!(decibel.as_ref().unwrap().numeric().representation(), text);
        assert_ne!(value, level(CssVoiceVolumeLevel::Loud, None));
        assert_eq!(value, retained);
        assert_eq!(
            CssVoiceVolume::Offset(decibel.clone().unwrap())
                .serialize_specified()
                .unwrap(),
            "0db"
        );
    }
}

#[test]
fn mixing_specified_limits_charge_omitted_children_and_fail_atomically() {
    let zero = level(CssVoiceVolumeLevel::Medium, Some("-0"));
    let retained = zero.clone();
    assert_eq!(
        zero.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 6))
            .unwrap(),
        "medium"
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
            zero.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(zero, retained);
        assert_eq!(zero.serialize_specified().unwrap(), "medium");
    }
    let plain = level(CssVoiceVolumeLevel::Medium, None);
    assert_eq!(
        plain
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 6))
            .unwrap(),
        "medium"
    );
    let offset = CssVoiceVolume::Offset(CssDecibelLiteral::try_new("-6").unwrap());
    assert_eq!(
        offset
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 4))
            .unwrap(),
        "-6db"
    );
    for value in [
        CssVoiceBalance::from_keyword(CssVoiceBalanceKeyword::Rightwards),
        CssVoiceBalance::try_number(number("101")).unwrap(),
    ] {
        let retained = value.clone();
        let expected = value.serialize_specified().unwrap();
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
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, expected.len() - 1),
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
            assert_eq!(value, retained);
        }
    }
}

#[test]
fn balance_math_resource_limits_use_the_number_owner_and_preserve_authored_graphs() {
    let source = declaration("voice-balance", "calc(101 + 20)");
    let value = balance(&source);
    let retained = value.clone();
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 100, 100),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(100, 0, 100),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(100, 100, 0),
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
        assert_eq!(value, &retained);
    }
    assert_eq!(
        value
            .number()
            .unwrap()
            .calculation()
            .unwrap()
            .components()
            .serialize()
            .unwrap()
            .as_css(),
        "calc(101 + 20)"
    );
}
