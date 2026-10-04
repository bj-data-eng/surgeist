#![forbid(unsafe_code)]
//! Existing public boundaries for Speech 1 §§6.1–6.2 and Values 4 Number math.
//! https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/#mixing-props
//! Authored values remain distinct from computed clamping and audio calibration.
//! New typed models and CSSOM canonical serialization are tested with their
//! functional implementation, rather than introducing unavailable APIs here.

use surgeist_css::*;

fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name)
        .unwrap_or_else(|| panic!("Speech mixing property must be admitted: {name}"))
}

fn declaration(name: &str, text: &str) -> CssDeclaration {
    let source = format!("{name}:{text}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one complete mixing declaration: {source}")
    };
    assert_eq!(value.known().unwrap().property(), property(name));
    assert_eq!(value.importance(), CssImportance::Important);
    value.clone()
}

fn expanded(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary/global mixing value completes")
    };
    values
}

fn ordinary(name: &str, text: &str) -> CssLonghandValue {
    let values = expanded(&declaration(name, text));
    let [value] = values.items() else {
        panic!("one terminal mixing contribution")
    };
    value.ordinary_value().unwrap().clone()
}

fn assert_admitted(name: &str, text: &str) {
    let source = declaration(name, text);
    let known = source.known().unwrap();
    assert!(known.property_value().is_some());
    assert!(known.global().is_none());
    assert!(known.substitution_dependent().is_none());
    let components = parse_component_values(text).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(property(name)),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{text}: {error:?}"));
    assert!(checked.known().unwrap().property_value().is_some());
    assert_eq!(checked.value_components(), &components);
    assert!(checked.position().is_none());
    assert_eq!(checked.importance(), CssImportance::Important);
    assert!(validate_style_attribute(&format!("{name}:{text}")).is_ok());
    assert!(validate_sheet(&format!(".speech{{{name}:{text}}}")).is_ok());
    let values = expanded(&source);
    let [value] = values.items() else {
        panic!("one completed mixing terminal")
    };
    assert_eq!(value.property(), property(name));
    assert!(value.ordinary_value().is_some());
    assert!(value.source().same_occurrence(&source));
    assert_eq!(value.source().importance(), CssImportance::Important);
}

#[test]
fn balance_admits_exact_unbounded_numbers_and_number_math_without_specified_clamping() {
    for text in [
        "-101",
        "101",
        "-9007199254740993",
        "9007199254740993",
        "1e400",
        "1e-400",
        "+0",
        "-0",
        ".125",
        "calc(101)",
        "calc(-100 - 20)",
        "min(101, 200)",
        "calc(1px / 1px)",
        "calc(infinity)",
        "calc(NaN)",
    ] {
        assert_admitted("voice-balance", text);
        assert_eq!(
            declaration("voice-balance", text)
                .value_components()
                .serialize()
                .unwrap()
                .as_css(),
            text
        );
    }
    for (left, right) in [
        ("101", "100"),
        ("-101", "-100"),
        ("9007199254740993", "9007199254740992"),
        ("calc(101)", "calc(100)"),
        ("left", "-100"),
        ("center", "0"),
        ("right", "100"),
    ] {
        assert_ne!(
            ordinary("voice-balance", left),
            ordinary("voice-balance", right)
        );
    }
}

#[test]
fn balance_keywords_and_all_six_volume_keywords_are_ordinary_authored_values() {
    for text in ["left", "center", "right", "leftwards", "rightwards"] {
        assert_admitted("voice-balance", text);
    }
    for text in ["silent", "x-soft", "soft", "medium", "loud", "x-loud"] {
        assert_admitted("voice-volume", text);
    }
}

#[test]
fn volume_admits_signed_exact_decibel_literals_and_both_level_offset_orders() {
    for offset in [
        "-6dB",
        "+6DB",
        "0db",
        "-0dB",
        "1e400dB",
        "1e-400dB",
        "9007199254740993dB",
    ] {
        assert_admitted("voice-volume", offset);
        for level in ["x-soft", "soft", "medium", "loud", "x-loud"] {
            assert_admitted("voice-volume", &format!("{level} {offset}"));
            assert_admitted("voice-volume", &format!("{offset} {level}"));
        }
    }
    // Omitted level and explicit medium have different inheritance meaning.
    assert_ne!(
        ordinary("voice-volume", "0db"),
        ordinary("voice-volume", "medium")
    );
    assert_ne!(
        ordinary("voice-volume", "-6db"),
        ordinary("voice-volume", "medium -6db")
    );
    // Canonical text may omit a zero companion, but the authored occurrence keeps it.
    assert_eq!(
        declaration("voice-volume", "medium 0db")
            .value_components()
            .serialize()
            .unwrap()
            .as_css(),
        "medium 0db"
    );
}

#[test]
fn mixing_wrong_domains_duplicates_and_silent_companions_drop_only_the_invalid_declaration() {
    for (name, invalid) in [
        (
            "voice-balance",
            &[
                "",
                "10%",
                "0db",
                "1px",
                "silent",
                "left right",
                "left 1",
                "\"left\"",
                "calc(1db)",
                "calc(1%)",
                "calc(1px)",
                "calc(1 + 1px)",
            ] as &[&str],
        ),
        (
            "voice-volume",
            &[
                "",
                "0",
                "6",
                "10%",
                "1px",
                "1Hz",
                "silent 0db",
                "0db silent",
                "silent medium",
                "medium silent",
                "silent silent",
                "soft loud",
                "soft soft",
                "1db 2db",
                "medium 1db 2db",
                "1db medium loud",
                "medium, 1db",
                "\"medium\"",
                "calc(1db)",
                "calc(1db + 2db)",
                "min(1db, 2db)",
                "calc(6)",
                "medium calc(1db)",
                "1 dB",
            ] as &[&str],
        ),
    ] {
        for text in invalid {
            let source = format!("color:red;{name}:{text};height:1px");
            let report = parse_style_attribute(&source);
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
                panic!("one dropped mixing declaration: {source}")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
            assert_ne!(diagnostic.error().code(), CssErrorCode::UnsupportedProperty);
            assert_eq!(
                validate_style_attribute(&source).unwrap_err().diagnostics(),
                report.diagnostics()
            );
            assert!(
                parse_property_value(
                    CssPropertyNameRef::Known(property(name)),
                    parse_component_values(text).unwrap(),
                    CssImportance::Normal,
                )
                .is_err(),
                "checked {name}:{text}"
            );
        }
    }
}

#[test]
fn mixing_metadata_initials_globals_and_expansion_preserve_inherited_terminal_identity() {
    for (name, initial) in [("voice-balance", "center"), ("voice-volume", "medium")] {
        let known = property(name);
        let CssPropertyKindRef::Longhand(metadata) = known.metadata().unwrap().kind() else {
            panic!("mixing property is a terminal longhand")
        };
        assert!(metadata.inherited_by_default());
        assert_eq!(metadata.property().known_property(), known);
        let intrinsic = metadata.initial_value();
        assert_eq!(intrinsic.property().known_property(), known);
        let CssInitialValueRef::Value(value) = intrinsic.view() else {
            panic!("specified initial has no host dependency")
        };
        assert_eq!(value, &ordinary(name, initial));
        let feature = feature_metadata(&format!("official.property.{name}")).unwrap();
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        assert_eq!(feature.source().id().as_str(), "S-SPEECH1");
        assert_eq!(
            feature.source().url(),
            Some("https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/")
        );
        assert_eq!(feature.production(), format!("#propdef-{name}"));
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text);
            assert_eq!(source.known().unwrap().global(), Some(keyword));
            let values = expanded(&source);
            let [value] = values.items() else {
                panic!("one global terminal")
            };
            assert_eq!(value.property(), known);
            assert_eq!(value.value(), CssContributionValueRef::Global(keyword));
            assert!(value.source().same_occurrence(&source));
            let checked = parse_property_value(
                CssPropertyNameRef::Known(known),
                parse_component_values(text).unwrap(),
                CssImportance::Important,
            )
            .unwrap();
            assert_eq!(checked.known().unwrap().global(), Some(keyword));
            assert!(
                parse_property_value(
                    CssPropertyNameRef::Known(known),
                    parse_component_values(&format!("{text} {initial}")).unwrap(),
                    CssImportance::Normal
                )
                .is_err()
            );
        }
    }
}

#[test]
fn mixing_substitution_reentry_rechecks_grammar_and_retains_original_and_replacement_origins() {
    for (name, valid, invalid) in [
        ("voice-balance", "calc(101)", "1db"),
        ("voice-volume", "-6db soft", "silent 1db"),
    ] {
        let source = declaration(name, "var(--mixing)");
        assert_eq!(
            source
                .known()
                .unwrap()
                .substitution_dependent()
                .unwrap()
                .as_css(),
            "var(--mixing)"
        );
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending mixing substitution")
        };
        for text in [valid, "inherit"] {
            let replacement = parse_component_values(text).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed mixing replacement")
            };
            let [value] = values.items() else {
                panic!("one replacement terminal")
            };
            assert_eq!(value.property(), property(name));
            assert!(value.source().same_occurrence(&source));
            assert_eq!(value.source().importance(), CssImportance::Important);
            assert_eq!(value.replacement_components(), Some(&replacement));
            if text == "inherit" {
                assert_eq!(
                    value.value(),
                    CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                );
            } else {
                assert!(value.ordinary_value().is_some());
            }
        }
        for text in [
            invalid,
            "inherit extra",
            "medium!important",
            "medium;color:red",
        ] {
            let error = handle
                .reenter(parse_component_values(text).unwrap())
                .unwrap_err();
            assert!(
                matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(_)),
                "{name}:{text}: {error:?}"
            );
        }
        assert_eq!(
            handle
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        assert!(
            handle
                .reenter(parse_component_values(valid).unwrap())
                .is_ok()
        );
    }
}

#[test]
fn mixing_case_escapes_comments_exact_coefficients_and_raw_byte_limits_preserve_authored_tokens() {
    for (name, canonical, text) in [
        ("VOICE-BALANCE", "voice-balance", "RIGHTWARDS"),
        (r"voice\2d balance", "voice-balance", r"l\65 ft"),
        ("voice-balance", "voice-balance", "9007199254740993"),
        ("VOICE-VOLUME", "voice-volume", "LOUD/**/+6DB"),
        (r"voice\2d volume", "voice-volume", r"-6d\42 /**/s\6f ft"),
        ("voice-volume", "voice-volume", "1e-400dB"),
    ] {
        let report = parse_style_attribute(&format!("{name}:{text}!important"));
        assert!(
            report.is_clean(),
            "{name}:{text}: {:?}",
            report.diagnostics()
        );
        let [source] = report.syntax().as_slice() else {
            panic!("one escaped mixing declaration")
        };
        assert_eq!(source.known().unwrap().property(), property(canonical));
        let components = source.value_components();
        assert_eq!(
            components
                .serialize_with_limit(text.len())
                .unwrap()
                .as_css(),
            text
        );
        assert!(components.serialize_with_limit(text.len() - 1).is_err());
        assert_eq!(components.serialize().unwrap().as_css(), text);
        let checked = parse_property_value(
            CssPropertyNameRef::Known(property(canonical)),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(checked.value_components(), components);
    }
}

#[test]
fn silent_companion_diagnostics_preserve_crlf_non_bmp_positions_and_original_component_origin() {
    let prefix = "/*😀*/\r\ncolor:red;";
    let invalid = "voice-volume:silent 6db;";
    let source = format!("{prefix}{invalid}height:1px");
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("one silent companion diagnostic")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("6db").unwrap();
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
    assert_eq!(diagnostic.error().position().line().value(), 1);
    assert_eq!(
        diagnostic.error().position().column().value(),
        u32::try_from("color:red;voice-volume:silent ".len()).unwrap()
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        prefix.len()
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        prefix.len() + invalid.len()
    );
    assert_eq!(
        validate_style_attribute(&source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let components = parse_component_values("silent 6db").unwrap();
    let token = components
        .items()
        .iter()
        .find(|value| {
            matches!(
                value.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Dimension { .. })
            )
        })
        .unwrap();
    let error = parse_property_value(
        CssPropertyNameRef::Known(property("voice-volume")),
        components.clone(),
        CssImportance::Normal,
    )
    .unwrap_err();
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(token.origin().clone())
    );
    let sheet_source = format!(".a{{{source}}}");
    let sheet = parse_sheet(&sheet_source);
    let [CssRule::Style(rule)] = sheet.syntax().rules() else {
        panic!("retained style rule")
    };
    assert_eq!(rule.declarations().len(), 2);
    assert_eq!(sheet.diagnostics().len(), 1);
    assert_eq!(
        sheet.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        responsible + 3
    );
    assert_eq!(
        validate_sheet(&sheet_source).unwrap_err().diagnostics(),
        sheet.diagnostics()
    );
}
