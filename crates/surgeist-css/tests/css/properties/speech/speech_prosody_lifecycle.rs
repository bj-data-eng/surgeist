//! Executable RED for Speech 1 CRD-20230214 §§11.2–11.4 and §12.1.
//! Expectations come from the retained property grammars and prose, not another
//! Surgeist serializer. Values 4 WD-20240312 §5.6 permits relative frequency/%
//! math; §10.9 gives no semitone math type; §10.12 defers calculation ranges.
//! Ordinary absolute frequency is strictly positive. Speech's negative absolute
//! prose conflicts with its explicit illegal-syntax example: authored negative
//! literals are rejected, while calculations retain their specified phase.
//! Raw component serialization is lexical; new typed canonical model APIs get
//! functional tests with implementation, rather than compile-failing stubs.
use crate::css::support::named_property::longhand_contributions as expanded;
use crate::css::support::{
    important_speech_declaration as declaration, known_property as property,
};
use surgeist_css::*;
const PROPERTIES: [&str; 4] = ["voice-duration", "voice-pitch", "voice-range", "voice-rate"];

fn assert_admitted(name: &str, value: &str) {
    let source = declaration(name, value);
    let known = source.known().unwrap();
    assert!(known.property_value().is_some());
    assert!(known.global().is_none());
    assert!(known.substitution_dependent().is_none());
    let components = CssComponentValues::parse(&CssParser::default(), value).unwrap();
    let checked = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(property(name))),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(checked.known().unwrap().property(), property(name));
    assert!(checked.known().unwrap().property_value().is_some());
    assert_eq!(checked.value_components(), &components);
    assert_eq!(checked.importance(), CssImportance::Important);
    assert!(checked.position().is_none());
    assert_eq!(
        source
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        value
    );
    assert!(
        CssParser::default()
            .parse_block_contents(format!("{name}:{value}"))
            .into_validation_result()
            .is_ok()
    );
    assert!(
        CssParser::default()
            .parse_sheet(format!(".speech{{{name}:{value}}}"))
            .into_validation_result()
            .is_ok()
    );
    let values = expanded(&source);
    let [item] = values.items() else {
        panic!("one terminal contribution")
    };
    assert_eq!(item.property(), property(name));
    assert!(item.ordinary_value().is_some());
    assert!(item.source().same_occurrence(&source));
    assert!(item.replacement_components().is_none());
}

#[test]
fn voice_duration_admits_auto_and_exact_nonnegative_ordinary_time() {
    // Speech §12.1 specifies absolute s/ms and a nonnegative ordinary range.
    for text in [
        "auto",
        "0s",
        "-0ms",
        "+3s",
        "250ms",
        "1e-999s",
        "9007199254740993ms",
        "1e999s",
    ] {
        assert_admitted("voice-duration", text);
    }
    let report = CssParser::default()
        .parse_sheet(".parent{voice-duration:1s}.child{voice-duration:2s;voice-rate:fast}");
    assert!(
        report.is_clean(),
        "duration subtree precedence belongs downstream: {:?}",
        report.diagnostics()
    );
    assert_eq!(report.syntax().rules().len(), 2);
}

#[test]
fn pitch_and_range_admit_absolute_frequency_in_both_orders_and_signed_relative_offsets() {
    // Speech §§11.3/11.4: && permits both absolute orders; || is a nonempty
    // composition of at most one level and at most one offset, in either order.
    for name in ["voice-pitch", "voice-range"] {
        for text in [
            "30Hz absolute",
            "absolute +2kHz",
            "1e-999Hz absolute",
            "9007199254740993Hz absolute",
            "x-low",
            "low",
            "medium",
            "high",
            "x-high",
            "250Hz",
            "+250Hz",
            "-20Hz",
            "0Hz",
            "-0kHz",
            "2st",
            "-3.5st",
            "+0st",
            "-0st",
            "1e999st",
            "-1e-999st",
            "25%",
            "+25%",
            "-50%",
            "-125%",
            "0%",
            "high -20Hz",
            "-20Hz high",
            "low 2st",
            "2st low",
            "x-high -50%",
            "-50% x-high",
        ] {
            assert_admitted(name, text);
        }
    }
}

#[test]
fn voice_rate_admits_six_keywords_and_nonnegative_exact_percentage_modifiers() {
    // Speech §11.2: percentages above 100 are valid; no machine float admission.
    for text in [
        "normal",
        "x-slow",
        "slow",
        "medium",
        "fast",
        "x-fast",
        "0%",
        "-0%",
        "+50%",
        "120%",
        "1e-999%",
        "1e999%",
        "9007199254740993%",
        "fast 120%",
        "120% fast",
        "normal 100%",
        "0% x-slow",
    ] {
        assert_admitted("voice-rate", text);
    }
}

#[test]
fn prosody_calculation_range_processing_stays_in_the_calculation_phase() {
    // Values §10.12: parse-time range checks do not occur inside math functions.
    for (name, values) in [
        (
            "voice-duration",
            &["calc(-1s)", "min(-1s, 2s)", "calc(0s)", "calc(1s + 250ms)"][..],
        ),
        (
            "voice-rate",
            &[
                "calc(-1%)",
                "min(-1%, 2%)",
                "fast calc(50% + 25%)",
                "calc(120%) fast",
            ][..],
        ),
        (
            "voice-pitch",
            &[
                "calc(-20Hz) absolute",
                "absolute min(-1Hz, 2Hz)",
                "absolute calc(0Hz)",
                "calc(-20Hz)",
                "high calc(-50%)",
            ][..],
        ),
        (
            "voice-range",
            &[
                "calc(-20Hz) absolute",
                "absolute min(-1Hz, 2Hz)",
                "absolute calc(0Hz)",
                "calc(-20Hz)",
                "high calc(-50%)",
            ][..],
        ),
    ] {
        for text in values {
            assert_admitted(name, text);
        }
    }
}

#[test]
fn relative_pitch_range_math_can_mix_frequency_and_inherited_frequency_percentages() {
    // Values §5.6 defines frequency-percentage as equivalent [frequency|%]
    // when % represents frequency. Speech % increments use inherited frequency.
    for name in ["voice-pitch", "voice-range"] {
        for text in [
            "calc(10Hz + 25%)",
            "high calc(10Hz + 25%)",
            "calc(25% - 10Hz) low",
            "min(10Hz, 25%)",
            "calc(-1kHz + 200%)",
        ] {
            assert_admitted(name, text);
        }
    }
}

#[test]
fn invalid_prosody_grammars_drop_one_declaration_and_keep_valid_siblings() {
    for (name, invalid) in [
        (
            "voice-duration",
            &[
                "",
                "normal",
                "-1ms",
                "-1e-999s",
                "0",
                "20%",
                "1Hz",
                "1st",
                "auto 1s",
                "1s 2s",
                "1s, 2s",
                "calc(1px)",
            ][..],
        ),
        (
            "voice-pitch",
            &[
                "",
                "normal",
                "absolute",
                "0Hz absolute",
                "absolute -0Hz",
                "-20Hz absolute",
                "absolute -1e-999Hz",
                "absolute 20%",
                "20% absolute",
                "2st absolute",
                "absolute 2st",
                "high absolute 20Hz",
                "20Hz absolute high",
                "high low",
                "1Hz 2Hz",
                "1Hz 2st",
                "20% 1Hz",
                "0",
                "1s",
                "1deg",
                "2px",
                "high, low",
                "calc(1st)",
                "calc(1st + 2st)",
                "calc(1Hz + 10%) absolute",
                "absolute calc(10%)",
                "calc(1px)",
            ][..],
        ),
        (
            "voice-range",
            &[
                "",
                "normal",
                "absolute",
                "0kHz absolute",
                "absolute -0Hz",
                "-20Hz absolute",
                "absolute -1e-999Hz",
                "absolute 20%",
                "2st absolute",
                "high absolute 20Hz",
                "high low",
                "1Hz 2st",
                "20% 1Hz",
                "0",
                "1s",
                "1deg",
                "high, low",
                "calc(1st)",
                "calc(1Hz + 10%) absolute",
                "absolute calc(10%)",
                "calc(1px)",
            ][..],
        ),
        (
            "voice-rate",
            &[
                "",
                "auto",
                "high",
                "-1%",
                "-1e-999%",
                "0",
                "1s",
                "1Hz",
                "1st",
                "fast slow",
                "50% 20%",
                "fast, 20%",
                "fast 20% normal",
                "calc(1)",
                "calc(1px)",
                "calc(1Hz + 20%)",
            ][..],
        ),
    ] {
        for value in invalid {
            let source = format!("color:red;{name}:{value};height:1px");
            let report = CssParser::default().parse_block_contents(&source);
            assert_eq!(report.syntax().len(), 2, "{source}");
            assert_eq!(
                report.syntax()[0].known().unwrap().property(),
                CssKnownProperty::Color
            );
            assert_eq!(
                report.syntax()[1].known().unwrap().property(),
                CssKnownProperty::Height
            );
            let [diagnostic] = report.diagnostics() else {
                panic!("one dropped declaration: {source}")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_ne!(
                diagnostic.error().code(),
                CssErrorCode::UnknownProperty,
                "{source}"
            );
            assert_ne!(
                diagnostic.error().code(),
                CssErrorCode::UnsupportedProperty,
                "{source}"
            );
            assert_eq!(
                CssParser::default()
                    .parse_block_contents(&source)
                    .into_validation_result()
                    .unwrap_err()
                    .diagnostics(),
                report.diagnostics()
            );
            assert!(
                CssDeclaration::try_from_components(
                    &CssParser::default(),
                    CssPropertyTarget::Property(CssPropertyNameRef::Known(property(name))),
                    CssComponentValues::parse(&CssParser::default(), value).unwrap(),
                    CssImportance::Normal
                )
                .is_err(),
                "checked {name}:{value}"
            );
        }
    }
}

#[test]
fn prosody_metadata_supplies_exact_inheritance_and_intrinsic_initial_terminals() {
    for (name, initial, inherited) in [
        ("voice-duration", "auto", false),
        ("voice-pitch", "medium", true),
        ("voice-range", "medium", true),
        ("voice-rate", "normal", true),
    ] {
        let known = property(name);
        let metadata = known.metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("terminal metadata")
        };
        assert_eq!(longhand.inherited_by_default(), inherited);
        assert_eq!(longhand.property().known_property(), known);
        let initial_value = longhand.initial_value();
        assert_eq!(initial_value.property().known_property(), known);
        let CssInitialValueRef::Value(value) = initial_value.view() else {
            panic!("intrinsic initial")
        };
        assert_eq!(
            value,
            expanded(&declaration(name, initial)).items()[0]
                .ordinary_value()
                .unwrap()
        );
    }
}

#[test]
fn prosody_substitution_reentry_checks_phase_and_grammar_atomically_with_origins() {
    for (name, valid, invalid) in [
        ("voice-duration", "calc(-1s)", "-1s"),
        ("voice-pitch", "high calc(10Hz + 25%)", "-1Hz absolute"),
        ("voice-range", "absolute calc(-1Hz)", "2st absolute"),
        ("voice-rate", "fast calc(-1%)", "-1% fast"),
    ] {
        let source = declaration(name, "var(--prosody)");
        assert_eq!(
            source
                .known()
                .unwrap()
                .substitution_dependent()
                .unwrap()
                .as_css(),
            "var(--prosody)"
        );
        let CssExpansion::Pending(handle) = source.expand().unwrap() else {
            panic!("pending prosody")
        };
        for text in [valid, "inherit"] {
            let replacement = CssComponentValues::parse(&CssParser::default(), text).unwrap();
            let checked = CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(property(name))),
                replacement.clone(),
                CssImportance::Important,
            )
            .unwrap();
            let expected = expanded(&checked);
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("one completed terminal")
            };
            let [item] = values.items() else {
                panic!("one terminal")
            };
            assert_eq!(item.property(), property(name));
            assert_eq!(item.value(), expected.items()[0].value());
            assert_eq!(item.replacement_components(), Some(&replacement));
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
        }
        for text in [
            invalid,
            "inherit extra",
            "normal!important",
            "normal;color:red",
        ] {
            let error = handle
                .reenter(CssComponentValues::parse(&CssParser::default(), text).unwrap())
                .unwrap_err();
            assert!(
                error.replacement_error().is_some(),
                "{name}:{text}: {error:?}"
            );
        }
        assert!(
            handle
                .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap())
                .unwrap_err()
                .is_residual_substitution()
        );
        assert!(
            handle
                .reenter(CssComponentValues::parse(&CssParser::default(), valid).unwrap())
                .is_ok()
        );
    }
}

#[test]
fn prosody_case_escapes_comments_and_exact_coefficients_survive_bounded_raw_serialization() {
    for (name, canonical, value) in [
        ("VOICE-DURATION", "voice-duration", "AUTO"),
        (r"voice\2d pitch", "voice-pitch", r"ABSOLUTE/**/+2k\48 z"),
        ("VOICE-RANGE", "voice-range", r"H\49 GH/**/-3.5S\54"),
        (r"voice\2d rate", "voice-rate", "9007199254740993%/**/FAST"),
    ] {
        let report = CssParser::default().parse_block_contents(format!("{name}:{value}!important"));
        assert!(
            report.is_clean(),
            "{name}:{value}: {:?}",
            report.diagnostics()
        );
        let [source] = report.syntax().as_slice() else {
            panic!("one declaration")
        };
        assert_eq!(source.known().unwrap().property(), property(canonical));
        assert_eq!(source.importance(), CssImportance::Important);
        let components = source.value_components();
        assert_eq!(
            components
                .serialize_tokens(crate::css::support::byte_limits(value.len()))
                .unwrap()
                .as_css(),
            value
        );
        assert!(
            components
                .serialize_tokens(crate::css::support::byte_limits(value.len() - 1))
                .unwrap_err()
                .is_byte_limit()
        );
        assert_eq!(
            components
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            value
        );
        let checked = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property(canonical))),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(checked.value_components(), components);
    }
}

#[test]
fn ordinary_range_failure_keeps_the_exact_token_position_and_complete_recovery_span() {
    for (name, value, rejected) in [
        ("voice-duration", "-1e-999s", "-1e-999s"),
        ("voice-pitch", "absolute -1e-999Hz", "-1e-999Hz"),
        ("voice-range", "-20Hz absolute", "-20Hz"),
        ("voice-rate", "fast -1e-999%", "-1e-999%"),
    ] {
        let prefix = "color:red;/*😀*/";
        let unit = format!("{name}:{value};");
        let source = format!("{prefix}{unit}height:1px");
        let report = CssParser::default().parse_block_contents(&source);
        assert_eq!(report.syntax().len(), 2);
        let [diagnostic] = report.diagnostics() else {
            panic!("one range diagnostic")
        };
        assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find(rejected).unwrap()
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            prefix.len()
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            prefix.len() + unit.len()
        );
        let sheet_source = format!(".a{{{source}}}");
        let sheet = CssParser::default().parse_sheet(&sheet_source);
        let [CssRule::Style(rule)] = sheet.syntax().rules() else {
            panic!("retained rule")
        };
        assert_eq!(rule.declarations().len(), 2);
        assert_eq!(
            sheet.diagnostics()[0]
                .error()
                .position()
                .byte_offset()
                .value(),
            source.find(rejected).unwrap() + 3
        );
        assert_eq!(
            CssParser::default()
                .parse_sheet(&sheet_source)
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            sheet.diagnostics()
        );
    }
}

#[test]
fn recovered_prosody_calculations_remain_diagnosed_and_checked_reentry_rejects_original_closures() {
    for (name, value) in [
        ("voice-duration", "calc(-1s"),
        ("voice-pitch", "calc(-1Hz"),
        ("voice-range", "high calc(-20%"),
        ("voice-rate", "fast calc(-1%"),
    ] {
        let source = format!("{name}:{value}");
        let report = CssParser::default().parse_block_contents(&source);
        assert!(!report.is_clean());
        let [declaration] = report.syntax().as_slice() else {
            panic!("retained recovered math: {source}")
        };
        assert_eq!(declaration.known().unwrap().property(), property(name));
        assert!(declaration.known().unwrap().property_value().is_some());
        assert!(!report.diagnostics().is_empty());
        assert_eq!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            report.diagnostics()
        );
        let components = declaration.value_components();
        let closed_text = components.serialize_tokens(Default::default()).unwrap();
        assert!(closed_text.as_css().ends_with(')'));
        // Serializing a recovery graph cannot launder its original implicit closure.
        assert!(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(property(name))),
                components.clone(),
                CssImportance::Normal
            )
            .is_err()
        );
        let pending = self::declaration(name, "var(--prosody)");
        let CssExpansion::Pending(handle) = pending.expand().unwrap() else {
            panic!("pending")
        };
        assert!(
            handle
                .reenter(components.clone())
                .unwrap_err()
                .replacement_error()
                .is_some()
        );
    }
}

#[test]
fn prosody_support_metadata_identifies_all_four_pinned_speech_productions() {
    for name in PROPERTIES {
        let feature = feature_metadata(&format!("official.property.{name}")).unwrap();
        assert_eq!(feature.source().id().as_str(), "S-SPEECH1");
        assert_eq!(
            feature.source().url(),
            Some("https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/")
        );
        assert_eq!(feature.production(), format!("#propdef-{name}"));
        assert_eq!(feature.status(), CssSupportStatus::Complete);
    }
}
