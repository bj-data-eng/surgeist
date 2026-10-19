#![forbid(unsafe_code)]
//! Functional new-API witnesses for Speech 1 §§10.1/10.3 and CSSOM optional
//! component omission. Existing-public preimplementation RED is in
//! speech_cue_lifecycle.rs; resources stay authored and are never loaded.

use surgeist_css::*;

fn audio(target: &str, offset: Option<&str>) -> CssCue {
    CssCue::Audio(
        CssAudioCue::try_new(
            CssUrl::new(target),
            offset.map(|number| CssDecibelLiteral::try_new(number).unwrap()),
        )
        .unwrap(),
    )
}

fn declaration(name: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{text}!important"));
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    report.syntax()[0].clone()
}

fn cue(source: &CssDeclaration) -> &CssCue {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::CueBefore(value) => value.value(),
        CssKnownPropertyValueRef::CueAfter(value) => value.value(),
        _ => panic!("cue longhand wrapper"),
    }
}

fn pair(source: &CssDeclaration) -> &CssCuePair {
    let CssKnownPropertyValueRef::Cue(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("cue shorthand wrapper")
    };
    value.value()
}

#[test]
fn decibel_literal_preserves_exact_signed_coefficients_units_and_origins() {
    for text in [
        "-0dB",
        "+3DB",
        "-9007199254740993dB",
        "1e400dB",
        "-1e-400dB",
        r"-3d\42",
    ] {
        let values = parse_component_values(text).unwrap();
        let component = values.items()[0].clone();
        let value = CssDecibelLiteral::try_from_component(component.clone()).unwrap();
        assert_eq!(value.component(), &component);
        assert_eq!(value.origin(), component.origin());
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) =
            component.view()
        else {
            panic!("dimension")
        };
        assert_eq!(value.numeric().representation(), number.representation());
        assert_eq!(
            CssComponentValues::try_new(vec![value.component().clone()])
                .unwrap()
                .serialize()
                .unwrap()
                .as_css(),
            text
        );
    }
    for number in ["-3", "+3", "-0", "1e400", "-1e-400"] {
        let value = CssDecibelLiteral::try_new(number).unwrap();
        assert_eq!(value.numeric().representation(), number);
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    }
}

#[test]
fn decibel_literal_rejects_wrong_domains_and_invalid_coefficients_at_the_original_origin() {
    for text in ["3", "3%", "3px", "3s", "none", "calc(3dB)"] {
        let component = parse_component_values(text).unwrap().items()[0].clone();
        let error = CssDecibelLiteral::try_from_component(component.clone()).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
        assert_eq!(error.origin(), component.origin());
    }
    for text in ["NaN", "infinity", "3 4", "", "3dB"] {
        assert!(CssDecibelLiteral::try_new(text).is_err(), "{text}");
    }
}

#[test]
fn cue_models_retain_none_resources_and_authored_offset_omission() {
    assert_eq!(cue(&declaration("cue-before", "none")), &CssCue::None);
    for (text, function, target, number) in [
        ("url(a)", CssUrlFunction::Url, "a", None),
        ("url(\"\")", CssUrlFunction::Url, "", None),
        (
            r#"src("a\20 b") -3d\42"#,
            CssUrlFunction::Src,
            "a b",
            Some("-3"),
        ),
        ("url(a) -0dB", CssUrlFunction::Url, "a", Some("-0")),
        ("url(a) 1e400dB", CssUrlFunction::Url, "a", Some("1e400")),
    ] {
        for name in ["cue-before", "cue-after"] {
            let source = declaration(name, text);
            let CssCue::Audio(value) = cue(&source) else {
                panic!("audio branch")
            };
            assert_eq!(value.url().function(), function);
            assert_eq!(value.url().as_str(), target);
            assert_eq!(
                value
                    .decibel()
                    .map(|value| value.numeric().representation()),
                number
            );
            if let Some(offset) = value.decibel() {
                assert!(
                    source
                        .value_components()
                        .items()
                        .iter()
                        .any(|part| part == offset.component())
                );
            }
        }
    }
    assert_ne!(audio("a", None), audio("a", Some("0")));
}

#[test]
fn cue_models_keep_pair_authored_presence_and_effective_after() {
    let single = declaration("cue", "url(a) -3dB");
    let single = pair(&single);
    assert!(single.authored_after().is_none());
    assert_eq!(single.after(), single.before());
    let explicit = declaration("cue", "url(a) -3dB url(a) -3dB");
    let explicit = pair(&explicit);
    assert!(explicit.authored_after().is_some());
    assert_eq!(explicit.after(), explicit.authored_after().unwrap());
    let mixed = CssCuePair::try_new(audio("a", Some("-3")), Some(CssCue::None)).unwrap();
    assert_eq!(mixed.after(), &CssCue::None);
    assert_eq!(mixed.serialize_specified().unwrap(), "url(\"a\") -3db none");
}

#[test]
fn cue_serialization_uses_shared_url_escaping_and_exact_canonical_decibel_payloads() {
    for (number, expected) in [
        ("-0", "0db"),
        ("+03.500", "3.5db"),
        ("-9007199254740993", "-9007199254740993db"),
        ("-1e-400", "0db"),
    ] {
        let value = CssDecibelLiteral::try_new(number).unwrap();
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
    // The accepted shared exact formatter uses its fixed-decimal form here;
    // authored coefficient spelling stays 1e400 independently of output.
    let huge = CssDecibelLiteral::try_new("1e400").unwrap();
    assert_eq!(
        huge.serialize_specified().unwrap(),
        format!("1{}db", "0".repeat(400))
    );
    assert_eq!(huge.numeric().representation(), "1e400");
    // Shared specified precision rounds this output to zero; exact authored
    // zero detection and equivalence must still see the nonzero coefficient.
    let tiny = CssDecibelLiteral::try_new("-1e-400").unwrap();
    assert_eq!(tiny.serialize_specified().unwrap(), "0db");
    assert_eq!(tiny.numeric().representation(), "-1e-400");
    let tiny_audio = audio("a", Some("-1e-400"));
    assert_eq!(tiny_audio.serialize_specified().unwrap(), "url(\"a\") 0db");
    let CssCue::Audio(retained_tiny) = &tiny_audio else {
        panic!("audio")
    };
    assert_eq!(
        retained_tiny.decibel().unwrap().numeric().representation(),
        "-1e-400"
    );
    let distinct = CssCuePair::try_new(tiny_audio, Some(audio("a", None))).unwrap();
    assert_eq!(
        distinct.serialize_specified().unwrap(),
        "url(\"a\") 0db url(\"a\")"
    );
    for target in ["", "a b", "#clip", "a\" ); height: 1px; /*", "a\n\\b"] {
        let value = audio(target, Some("-3.500"));
        let retained = value.clone();
        let css = value.serialize_specified().unwrap();
        let parsed = declaration("cue-before", &css);
        let CssCue::Audio(parsed) = cue(&parsed) else {
            panic!("serialized audio")
        };
        assert_eq!(parsed.url().as_str(), target);
        assert_eq!(parsed.decibel().unwrap().numeric().representation(), "-3.5");
        assert_eq!(value, retained);
    }
    let source = declaration("cue-before", "src(\"a\" symbolic modifier(x)) +3DB");
    assert_eq!(
        cue(&source).serialize_specified().unwrap(),
        "src(\"a\" symbolic modifier(x)) 3db"
    );
}

#[test]
fn cue_serialization_omits_implied_zero_and_equivalent_after_without_discarding_authorship() {
    for zero in ["0", "-0", "+0.00e999999999999999999999999999999999999999"] {
        let value = audio("a", Some(zero));
        assert_eq!(value.serialize_specified().unwrap(), "url(\"a\")");
        let CssCue::Audio(value) = &value else {
            panic!("audio")
        };
        assert_eq!(value.decibel().unwrap().numeric().representation(), zero);
        for (before, after) in [
            (audio("a", None), CssCue::Audio(value.clone())),
            (CssCue::Audio(value.clone()), audio("a", None)),
        ] {
            let model = CssCuePair::try_new(before, Some(after)).unwrap();
            let retained = model.clone();
            assert_eq!(model.serialize_specified().unwrap(), "url(\"a\")");
            assert!(model.authored_after().is_some());
            assert_eq!(model, retained);
        }
    }
    for (left, right) in [
        ("+3.00", "3"),
        ("-1e-400", "-10e-401"),
        ("9007199254740993", "90071992547409930e-1"),
    ] {
        let model =
            CssCuePair::try_new(audio("a", Some(left)), Some(audio("a", Some(right)))).unwrap();
        assert_eq!(
            model.serialize_specified().unwrap(),
            audio("a", Some(left)).serialize_specified().unwrap()
        );
    }
    let different = CssCuePair::try_new(
        audio("a", Some("9007199254740992")),
        Some(audio("a", Some("9007199254740993"))),
    )
    .unwrap();
    assert_eq!(
        different.serialize_specified().unwrap(),
        "url(\"a\") 9007199254740992db url(\"a\") 9007199254740993db"
    );
}

#[test]
fn cue_budgets_are_cumulative_and_atomic_even_for_omitted_authored_children() {
    // Pair + two audio aggregates + two URL/target pairs + two dB terminals.
    let model =
        CssCuePair::try_new(audio("a", Some("-3")), Some(audio("a", Some("-3.0")))).unwrap();
    let retained = model.clone();
    let expected = "url(\"a\") -3db";
    assert_eq!(
        model
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                9,
                9,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(8, 9, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, 8, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(9, 9, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            model
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(model, retained);
        assert_eq!(model.serialize_specified().unwrap(), expected);
    }
    // No duplicate traversal for a second component that was never authored.
    let single = CssCuePair::try_new(audio("a", Some("-3")), None).unwrap();
    assert_eq!(
        single
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                5,
                5,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    // Explicit zero remains input work even when its output is omitted.
    let zero = audio("a", Some("0"));
    assert_eq!(
        zero.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(4, 4, 8))
            .unwrap(),
        "url(\"a\")"
    );
    assert_eq!(
        zero.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 4, 8))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
}

#[test]
fn cue_url_modifier_components_share_cumulative_budgets_and_safe_boundaries() {
    let argument =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("x").unwrap()]).unwrap();
    let modifier = CssUrlModifier::Function(
        CssUrlModifierFunction::try_new(CssIdent::try_new("mod").unwrap(), argument).unwrap(),
    );
    let url = CssUrl::from_parts(CssUrlFunction::Src, "a", vec![modifier]);
    let value = CssAudioCue::try_new(url, Some(CssDecibelLiteral::try_new("3").unwrap())).unwrap();
    let retained = value.clone();
    let expected = "src(\"a\" mod(x)) 3db";
    // Audio aggregate + URL + target + modifier + argument + dB.
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                6,
                6,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(5, 6, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 5, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 6, expected.len() - 1),
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
    assert_eq!(
        cue(&declaration("cue-after", expected))
            .serialize_specified()
            .unwrap(),
        expected
    );
}

#[test]
fn equivalent_after_with_url_modifiers_consumes_nodes_without_extra_output_bytes() {
    let before = declaration("cue-before", "src(\"a\" mod(x)) -3dB");
    let after = declaration("cue-after", "src(\"a\" mod(x)) -3.0dB");
    let value = CssCuePair::try_new(cue(&before).clone(), Some(cue(&after).clone())).unwrap();
    let retained = value.clone();
    let expected = "src(\"a\" mod(x)) -3db";
    // Pair plus two complete six-node audio cue providers, including hidden after.
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                13,
                13,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(12, 13, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(13, 12, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
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

#[test]
fn checked_cue_models_reject_retained_url_modifier_argument_recovery() {
    let report = parse_style_attribute("cue-before:url(\"a\" mod([");
    assert!(!report.is_clean());
    let [source] = report.syntax().as_slice() else {
        panic!("retained recovered cue")
    };
    let CssCue::Audio(value) = cue(source) else {
        panic!("audio")
    };
    let error = CssAudioCue::try_new(value.url().clone(), value.decibel().cloned()).unwrap_err();
    assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidFunction);
    assert!(matches!(
        error.origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));
    let error = CssCuePair::try_new(cue(source).clone(), None).unwrap_err();
    assert!(matches!(
        error.origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));
    assert!(CssCuePair::try_new(CssCue::None, Some(cue(source).clone())).is_err());
}
