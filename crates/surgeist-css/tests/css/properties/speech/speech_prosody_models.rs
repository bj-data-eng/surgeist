#![forbid(unsafe_code)]
//! Functional models accompany their new APIs after speech_prosody_lifecycle RED.
//! Independent authority: pinned Speech 1 §§11.2–11.4/12.1, Values 4 §§5.6,
//! 10.9/10.12, and CSSOM serialize-a-css-value/component-value. These tests
//! distinguish exact admission, specified projection and downstream voice use.
use surgeist_css::*;

fn declaration(name: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{text}"));
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    report.syntax()[0].clone()
}
fn pitch(source: &CssDeclaration) -> &CssVoicePitchRange {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::VoicePitch(value) => value.value(),
        CssKnownPropertyValueRef::VoiceRange(value) => value.value(),
        _ => panic!("pitch/range wrapper"),
    }
}
fn rate(source: &CssDeclaration) -> &CssVoiceRate {
    let CssKnownPropertyValueRef::VoiceRate(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("rate wrapper")
    };
    value.value()
}
fn frequency(number: &str, unit: CssFrequencyUnit) -> CssFrequencyValue {
    CssFrequencyValue::from_literal(CssFrequencyLiteral::try_new(number, unit).unwrap())
}
fn percentage(text: &str) -> CssSpecifiedPercentage {
    CssSpecifiedPercentage::try_from_component(
        CssComponentValue::try_token(&format!("{text}%")).unwrap(),
    )
    .unwrap()
}
fn nonnegative_percentage(text: &str) -> CssSpecifiedNonNegativePercentage {
    CssSpecifiedNonNegativePercentage::try_from_component(
        CssComponentValue::try_token(&format!("{text}%")).unwrap(),
    )
    .unwrap()
}
fn calculation(text: &str) -> CssFrequencyPercentageCalculation {
    CssFrequencyPercentageCalculation::try_from_components(parse_component_values(text).unwrap())
        .unwrap()
}

#[test]
fn duration_checked_construction_distinguishes_auto_nonnegative_time_and_retained_math() {
    assert_eq!(
        CssVoiceDuration::Auto.serialize_specified().unwrap(),
        "auto"
    );
    for (text, output) in [("-0", "0s"), ("+3", "3s"), ("0.25", "0.25s")] {
        let value = CssVoiceDuration::try_time(CssTimeValue::from_literal(
            CssTimeLiteral::try_new(text, CssTimeUnit::Seconds).unwrap(),
        ))
        .unwrap();
        assert_eq!(value.serialize_specified().unwrap(), output);
        let CssVoiceDuration::Time(duration) = value else {
            panic!("time")
        };
        assert_eq!(
            duration
                .time()
                .literal()
                .unwrap()
                .numeric()
                .representation(),
            text
        );
    }
    for text in ["-1", "-1e-999"] {
        let error = CssVoiceDuration::try_time(CssTimeValue::from_literal(
            CssTimeLiteral::try_new(text, CssTimeUnit::Seconds).unwrap(),
        ))
        .unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
    }
    let time = CssTimeValue::try_from_calculation(
        CssTimeCalculation::try_from_components(parse_component_values("calc(-1s)").unwrap())
            .unwrap(),
    )
    .unwrap();
    let value = CssVoiceDuration::try_time(time).unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "calc(-1s)");
    assert!(matches!(value, CssVoiceDuration::Time(value) if value.time().calculation().is_some()));
}

#[test]
fn absolute_pitch_range_requires_exact_positive_ordinary_frequency_and_normalizes_order() {
    for text in ["0", "-0", "-20", "-1e-999"] {
        let error =
            CssVoicePitchRange::try_absolute(frequency(text, CssFrequencyUnit::Hertz)).unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
    }
    for (text, unit, output) in [
        ("30", CssFrequencyUnit::Hertz, "30hz absolute"),
        ("+2", CssFrequencyUnit::Kilohertz, "2khz absolute"),
        (
            "9007199254740993",
            CssFrequencyUnit::Hertz,
            "9007199254740993hz absolute",
        ),
    ] {
        let value = CssVoicePitchRange::try_absolute(frequency(text, unit)).unwrap();
        assert_eq!(value.serialize_specified().unwrap(), output);
        assert_eq!(
            value
                .absolute_frequency()
                .unwrap()
                .literal()
                .unwrap()
                .numeric()
                .representation(),
            text
        );
        assert_eq!(value.level(), None);
        assert_eq!(value.offset(), None);
    }
    for name in ["voice-pitch", "voice-range"] {
        for text in ["absolute +2kHz", "+2kHz absolute"] {
            assert_eq!(
                pitch(&declaration(name, text))
                    .serialize_specified()
                    .unwrap(),
                "2khz absolute"
            );
        }
    }
}

#[test]
fn ordinary_absolute_frequency_fails_closed_when_shared_precision_rounds_to_zero() {
    // Values §5 permits supported precision; CSSOM number output rounds to six
    // places, but its value algorithm must emit grammar-representative output.
    // Speech ordinary absolute frequency is strictly positive: retain admission,
    // fail atomically at emission without epsilon clamping or invented calc().
    for (unit, suffix) in [
        (CssFrequencyUnit::Hertz, "hz"),
        (CssFrequencyUnit::Kilohertz, "khz"),
    ] {
        for text in ["1e-999", "0.00000049"] {
            let value = CssVoicePitchRange::try_absolute(frequency(text, unit)).unwrap();
            let original = value.clone();
            let error = value.serialize_specified().unwrap_err();
            assert_eq!(
                error.kind(),
                CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
            );
            assert_eq!(value, original);
            let literal = value.absolute_frequency().unwrap().literal().unwrap();
            assert_eq!(literal.numeric().representation(), text);
            assert!(matches!(literal.origin(), CssValueOrigin::Programmatic));
            // The primitive's policy and legal relative zero emission remain intact.
            assert_eq!(literal.serialize_specified().unwrap(), format!("0{suffix}"));
            let relative = CssVoicePitchRange::try_relative(
                None,
                Some(CssVoiceOffset::Frequency(
                    value.absolute_frequency().unwrap().clone(),
                )),
            )
            .unwrap();
            assert_eq!(
                relative.serialize_specified().unwrap(),
                format!("0{suffix}")
            );
            for name in ["voice-pitch", "voice-range"] {
                let source = declaration(name, &format!("absolute {text}{suffix}"));
                let original = source.clone();
                let origin = pitch(&source)
                    .absolute_frequency()
                    .unwrap()
                    .origin()
                    .clone();
                assert_eq!(
                    pitch(&source).serialize_specified().unwrap_err().kind(),
                    CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
                );
                assert_eq!(source, original);
                assert_eq!(
                    pitch(&source).absolute_frequency().unwrap().origin(),
                    &origin
                );
                assert!(matches!(origin, CssValueOrigin::Parsed(_)));
            }
        }
        // CSSOM nearest rounding with ties away from zero preserves positivity.
        for text in ["0.0000005", "0.000001"] {
            let value = CssVoicePitchRange::try_absolute(frequency(text, unit)).unwrap();
            let css = value.serialize_specified().unwrap();
            assert_eq!(css, format!("0.000001{suffix} absolute"));
            assert!(validate_style_attribute(&format!("voice-pitch:{css}")).is_ok());
        }
    }
}

#[test]
fn absolute_precision_failure_respects_resources_encountered_before_value_detection() {
    for unit in [CssFrequencyUnit::Hertz, CssFrequencyUnit::Kilohertz] {
        let value = CssVoicePitchRange::try_absolute(frequency("1e-999", unit)).unwrap();
        let original = value.clone();
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, usize::MAX, usize::MAX),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, usize::MAX, usize::MAX),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(usize::MAX, 1, usize::MAX),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, 0),
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
            assert_eq!(value, original);
        }
        let literal_bytes = match unit {
            CssFrequencyUnit::Hertz => 3,
            CssFrequencyUnit::Kilohertz => 4,
            _ => unreachable!("selected units"),
        };
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    2,
                    2,
                    literal_bytes - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        // Once the formatted literal fits, fail before visiting/emitting absolute.
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    2,
                    2,
                    literal_bytes
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::UnrepresentableValue
        );
        assert_eq!(value, original);
    }
}

#[test]
fn absolute_frequency_calculations_defer_range_checks_and_reject_ordinary_root_conversion() {
    for text in ["calc(-20Hz)", "calc(0Hz)"] {
        let frequency = CssFrequencyValue::try_from_calculation(
            CssFrequencyCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap(),
        )
        .unwrap();
        let value = CssVoicePitchRange::try_absolute(frequency).unwrap();
        assert!(value.absolute_frequency().unwrap().calculation().is_some());
        let output = value.serialize_specified().unwrap();
        assert!(validate_style_attribute(&format!("voice-pitch:{output}")).is_ok());
    }
    let ordinary = CssFrequencyValue::try_from_calculation(
        CssFrequencyCalculation::try_literal("0", CssFrequencyUnit::Hertz).unwrap(),
    )
    .unwrap();
    assert!(ordinary.literal().is_some());
    assert_eq!(
        CssVoicePitchRange::try_absolute(ordinary)
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::OutOfRange
    );
}

#[test]
fn relative_pitch_range_is_nonempty_and_preserves_level_and_offset_omissions() {
    assert_eq!(
        CssVoicePitchRange::try_relative(None, None)
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::EmptyValue
    );
    for (level, keyword) in [
        (CssVoiceLevel::XLow, "x-low"),
        (CssVoiceLevel::Low, "low"),
        (CssVoiceLevel::Medium, "medium"),
        (CssVoiceLevel::High, "high"),
        (CssVoiceLevel::XHigh, "x-high"),
    ] {
        let value = CssVoicePitchRange::level_only(level);
        assert_eq!(value.level(), Some(level));
        assert!(value.offset().is_none());
        assert!(value.absolute_frequency().is_none());
        assert_eq!(value.serialize_specified().unwrap(), keyword);
    }
    let offset = CssVoiceOffset::Frequency(frequency("-20", CssFrequencyUnit::Hertz));
    let value = CssVoicePitchRange::try_relative(None, Some(offset)).unwrap();
    assert_eq!(value.level(), None);
    assert_eq!(value.serialize_specified().unwrap(), "-20hz");
    for name in ["voice-pitch", "voice-range"] {
        for text in ["-20Hz high", "high -20Hz"] {
            assert_eq!(
                pitch(&declaration(name, text))
                    .serialize_specified()
                    .unwrap(),
                "high -20hz"
            );
        }
        // Speech computes a keyword-plus-offset to fixed frequency; even zero
        // offsets differ from keyword-only inheritance on later voice changes.
        assert_eq!(
            pitch(&declaration(name, "high 0Hz"))
                .serialize_specified()
                .unwrap(),
            "high 0hz"
        );
        assert_eq!(
            pitch(&declaration(name, "high 0%"))
                .serialize_specified()
                .unwrap(),
            "high 0%"
        );
    }
}

#[test]
fn semitone_terminal_keeps_exact_signed_coefficients_and_rejects_other_units_and_math() {
    for (text, output) in [
        ("-3.5", "-3.5st"),
        ("+2", "2st"),
        ("-0", "0st"),
        ("1e-999", "0st"),
        ("9007199254740993", "9007199254740993st"),
    ] {
        let value = CssSemitoneLiteral::try_new(text).unwrap();
        assert_eq!(value.numeric().representation(), text);
        assert!(matches!(value.origin(), CssValueOrigin::Programmatic));
        assert_eq!(value.serialize_specified().unwrap(), output);
        let model = CssVoicePitchRange::try_relative(
            Some(CssVoiceLevel::Low),
            Some(CssVoiceOffset::Semitones(value)),
        )
        .unwrap();
        assert_eq!(
            model.serialize_specified().unwrap(),
            format!("low {output}")
        );
    }
    let escaped = parse_component_values(r"-3.5S\54").unwrap();
    let value = CssSemitoneLiteral::try_from_component(escaped.items()[0].clone()).unwrap();
    assert_eq!(value.component(), &escaped.items()[0]);
    assert_eq!(value.serialize_specified().unwrap(), "-3.5st");
    for text in ["2Hz", "2s", "2deg", "2%", "2", "calc(2st)"] {
        let components = parse_component_values(text).unwrap();
        assert_eq!(
            CssSemitoneLiteral::try_from_component(components.items()[0].clone())
                .unwrap_err()
                .kind(),
            CssComponentValueErrorKind::InvalidToken
        );
    }
}

#[test]
fn frequency_percentage_root_reuses_shared_hint_algebra_and_excludes_other_dimensions() {
    for text in ["calc(10Hz + 25%)", "min(10Hz, 25%)", "calc(25%)"] {
        let value = calculation(text);
        assert_eq!(value.result_type(), CssCalculationType::FrequencyPercentage);
        assert_eq!(
            value.numeric_type().percent_hint(),
            Some(CssNumericDimension::Frequency)
        );
        assert_eq!(
            value
                .numeric_type()
                .exponent(CssNumericDimension::Frequency),
            1
        );
        assert_eq!(
            value
                .numeric_type()
                .exponent(CssNumericDimension::Percentage),
            0
        );
        let model = CssVoicePitchRange::try_relative(
            None,
            Some(CssVoiceOffset::try_from_calculation(value).unwrap()),
        )
        .unwrap();
        assert!(model.absolute_frequency().is_none());
        assert!(matches!(
            model.offset(),
            Some(CssVoiceOffset::Calculation(_))
        ));
    }
    assert_eq!(
        calculation("calc(10Hz)").result_type(),
        CssCalculationType::Frequency
    );
    for text in [
        "calc(1st)",
        "calc(1px)",
        "calc(1s)",
        "calc(1)",
        "calc(1Hz * 1Hz)",
    ] {
        assert!(
            CssFrequencyPercentageCalculation::try_from_components(
                parse_component_values(text).unwrap()
            )
            .is_err(),
            "{text}"
        );
    }
    assert!(
        CssFrequencyCalculation::try_from_components(
            parse_component_values("calc(10Hz + 25%)").unwrap()
        )
        .is_err()
    );
}

#[test]
fn mixed_offsets_preserve_unresolved_frequency_basis_during_shared_specified_projection() {
    // CSSOM numeric ordering plus Values simplification: percentages sort before
    // frequency, scalar operations distribute without merging unlike terms.
    for (text, output) in [
        ("calc((1Hz + 2%) * 3)", "calc(6% + 3hz)"),
        ("calc((1kHz + 2%) / 2)", "calc(1% + 500hz)"),
        ("calc(1% + 2%)", "calc(3%)"),
        ("min(1Hz, 2%)", "min(1hz, 2%)"),
    ] {
        let value = CssVoicePitchRange::try_relative(
            Some(CssVoiceLevel::High),
            Some(CssVoiceOffset::try_from_calculation(calculation(text)).unwrap()),
        )
        .unwrap();
        assert_eq!(
            value.serialize_specified().unwrap(),
            format!("high {output}")
        );
        let before = value.clone();
        assert!(validate_style_attribute(&format!("voice-range:high {output}")).is_ok());
        assert_eq!(value, before);
    }
    let ordinary = CssVoiceOffset::try_from_calculation(calculation("25%")).unwrap();
    assert!(matches!(ordinary, CssVoiceOffset::Percentage(_)));
    let ordinary = CssVoiceOffset::try_from_calculation(calculation("10Hz")).unwrap();
    assert!(matches!(ordinary, CssVoiceOffset::Frequency(_)));
}

#[test]
fn voice_rate_models_preserve_omitted_keyword_and_modifier_independently() {
    assert_eq!(
        CssVoiceRate::try_new(None, None).unwrap_err().kind(),
        &CssNumericConstructionErrorKind::EmptyValue
    );
    for (keyword, output) in [
        (CssVoiceRateKeyword::Normal, "normal"),
        (CssVoiceRateKeyword::XSlow, "x-slow"),
        (CssVoiceRateKeyword::Slow, "slow"),
        (CssVoiceRateKeyword::Medium, "medium"),
        (CssVoiceRateKeyword::Fast, "fast"),
        (CssVoiceRateKeyword::XFast, "x-fast"),
    ] {
        let value = CssVoiceRate::keyword_only(keyword);
        assert_eq!(value.keyword(), Some(keyword));
        assert!(value.percentage().is_none());
        assert_eq!(value.serialize_specified().unwrap(), output);
    }
    let value = CssVoiceRate::try_new(None, Some(nonnegative_percentage("120"))).unwrap();
    assert_eq!(value.keyword(), None);
    assert_eq!(value.serialize_specified().unwrap(), "120%");
    assert_ne!(
        value,
        CssVoiceRate::try_new(
            Some(CssVoiceRateKeyword::Normal),
            Some(nonnegative_percentage("120"))
        )
        .unwrap()
    );
    assert_eq!(
        rate(&declaration("voice-rate", "120% fast"))
            .serialize_specified()
            .unwrap(),
        "fast 120%"
    );
    assert_eq!(
        rate(&declaration("voice-rate", "fast calc(-1%)"))
            .serialize_specified()
            .unwrap(),
        "fast calc(-1%)"
    );
}

#[test]
fn exact_neutral_rate_modifier_is_omitted_beside_keyword_but_authored_identity_and_visits_remain() {
    // CSSOM omission without meaning change; Speech's multiplicative 100% is
    // neutral beside explicit keyword. Standalone % may refer to inherited rate.
    for text in ["100", "+0100.00", "1e2"] {
        let value = CssVoiceRate::try_new(
            Some(CssVoiceRateKeyword::Fast),
            Some(nonnegative_percentage(text)),
        )
        .unwrap();
        assert!(value.percentage().is_some());
        assert_ne!(value, CssVoiceRate::keyword_only(CssVoiceRateKeyword::Fast));
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 4))
                .unwrap(),
            "fast"
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 3, 4))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 2, 4))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
    }
    assert_eq!(
        CssVoiceRate::try_new(None, Some(nonnegative_percentage("100")))
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "100%"
    );
    // Exact comparison, not rounded text: this modifier prints 100% but remains
    // nonneutral in the authored model and must not be suppressed.
    let value = CssVoiceRate::try_new(
        Some(CssVoiceRateKeyword::Fast),
        Some(nonnegative_percentage("100.00000001")),
    )
    .unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "fast 100%");
    assert_eq!(
        rate(&declaration("voice-rate", "fast calc(100%)"))
            .serialize_specified()
            .unwrap(),
        "fast calc(100%)"
    );
}

#[test]
fn checked_composition_rejects_original_recovered_numeric_children() {
    for (name, text) in [
        ("voice-duration", "calc(-1s"),
        ("voice-pitch", "absolute calc(-1Hz"),
        ("voice-range", "high calc(25%"),
        ("voice-rate", "fast calc(-1%"),
    ] {
        let report = parse_style_attribute(&format!("{name}:{text}"));
        assert!(!report.is_clean());
        let source = &report.syntax()[0];
        let error = match source.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::VoiceDuration(value) => {
                let CssVoiceDuration::Time(duration) = value.value() else {
                    panic!("time")
                };
                CssVoiceDuration::try_time(duration.time().clone()).unwrap_err()
            }
            CssKnownPropertyValueRef::VoicePitch(value) => CssVoicePitchRange::try_absolute(
                value.value().absolute_frequency().unwrap().clone(),
            )
            .unwrap_err(),
            CssKnownPropertyValueRef::VoiceRange(value) => {
                let CssVoiceOffset::Calculation(calculation) = value.value().offset().unwrap()
                else {
                    panic!("calculation")
                };
                let error = CssVoiceOffset::try_from_calculation(calculation.clone()).unwrap_err();
                assert_eq!(
                    error.kind(),
                    &CssNumericConstructionErrorKind::RecoveredComponent
                );
                CssVoicePitchRange::try_relative(
                    value.value().level(),
                    value.value().offset().cloned(),
                )
                .unwrap_err()
            }
            CssKnownPropertyValueRef::VoiceRate(value) => {
                CssVoiceRate::try_new(value.value().keyword(), value.value().percentage().cloned())
                    .unwrap_err()
            }
            _ => panic!("prosody wrapper"),
        };
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RecoveredComponent
        );
        assert!(matches!(
            error.origin(),
            Some(CssValueOrigin::ImplicitClosure { .. })
        ));
    }
}

#[test]
fn mixed_pitch_and_rate_serializers_share_atomic_cumulative_limits() {
    let pitch = pitch(&declaration("voice-pitch", "high calc(10Hz + 25%)")).clone();
    let rate = CssVoiceRate::try_new(
        Some(CssVoiceRateKeyword::Fast),
        Some(nonnegative_percentage("120")),
    )
    .unwrap();
    let offset = CssVoicePitchRange::try_relative(
        Some(CssVoiceLevel::Low),
        Some(CssVoiceOffset::Percentage(percentage("-50"))),
    )
    .unwrap();
    for value in [pitch, offset] {
        let original = value.clone();
        let css = value.serialize_specified().unwrap();
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(1, usize::MAX, usize::MAX),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(usize::MAX, 1, usize::MAX),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, css.len() - 1),
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
            assert_eq!(value, original);
        }
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    usize::MAX,
                    usize::MAX,
                    css.len()
                ))
                .unwrap(),
            css
        );
    }
    assert_eq!(
        rate.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 9))
            .unwrap(),
        "fast 120%"
    );
    assert_eq!(
        rate.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 3, 9))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    let literal = CssSemitoneLiteral::try_new("-3.5").unwrap();
    assert_eq!(
        literal
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap(),
        "-3.5st"
    );
    assert_eq!(
        literal
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 5))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}
