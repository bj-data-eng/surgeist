#![forbid(unsafe_code)]
//! Passing composed-contract controls for the selected Speech 1 CRD20230214.
//! Independent expectations come from §3.1, §§6–12 and its 19-name Property
//! Index (references/css-speech-1--CRD-css-speech-1-20230214--bb7b59c92564.md).
//! CSSOM value serialization supplies grammar order and optional-component
//! omission; Values 4 supplies shared numeric/unit serialization. These tests
//! do not select voices, resolve inheritance, load audio or execute synthesis.

use surgeist_css::*;

struct Case {
    name: &'static str,
    authored: &'static str,
    canonical: &'static str,
    invalid: &'static str,
    invalid_offset: usize,
    inherited: bool,
    initial: Option<&'static str>,
}

// Enumerated independently from the selected source's Property Index, rather
// than inferred from production schema membership or implementation output.
const CASES: [Case; 19] = [
    Case {
        name: "cue",
        authored: "url(\"cue.wav\") 0dB none",
        canonical: "url(\"cue.wav\") none",
        invalid: "none 3db",
        invalid_offset: 5,
        inherited: false,
        initial: None,
    },
    Case {
        name: "cue-after",
        authored: "NONE",
        canonical: "none",
        invalid: "none 3db",
        invalid_offset: 5,
        inherited: false,
        initial: Some("none"),
    },
    Case {
        name: "cue-before",
        authored: "url(\"before.wav\") -3DB",
        canonical: "url(\"before.wav\") -3db",
        invalid: "none 3db",
        invalid_offset: 5,
        inherited: false,
        initial: Some("none"),
    },
    Case {
        name: "pause",
        authored: "weak strong",
        canonical: "weak strong",
        invalid: "weak strong medium",
        invalid_offset: 12,
        inherited: false,
        initial: None,
    },
    Case {
        name: "pause-after",
        authored: "WEAK",
        canonical: "weak",
        invalid: "-1ms",
        invalid_offset: 0,
        inherited: false,
        initial: Some("none"),
    },
    Case {
        name: "pause-before",
        authored: "250ms",
        canonical: "0.25s",
        invalid: "-1ms",
        invalid_offset: 0,
        inherited: false,
        initial: Some("none"),
    },
    Case {
        name: "rest",
        authored: "none x-strong",
        canonical: "none x-strong",
        invalid: "none none none",
        invalid_offset: 10,
        inherited: false,
        initial: None,
    },
    Case {
        name: "rest-after",
        authored: "0s",
        canonical: "0s",
        invalid: "-1ms",
        invalid_offset: 0,
        inherited: false,
        initial: Some("none"),
    },
    Case {
        name: "rest-before",
        authored: "none",
        canonical: "none",
        invalid: "-1ms",
        invalid_offset: 0,
        inherited: false,
        initial: Some("none"),
    },
    Case {
        name: "speak",
        authored: "ALWAYS",
        canonical: "always",
        invalid: "sometimes",
        invalid_offset: 0,
        inherited: true,
        initial: Some("auto"),
    },
    Case {
        name: "speak-as",
        authored: "digits spell-out",
        canonical: "spell-out digits",
        invalid: "normal digits",
        invalid_offset: 7,
        inherited: true,
        initial: Some("normal"),
    },
    Case {
        name: "voice-balance",
        authored: "101",
        canonical: "101",
        invalid: "10%",
        invalid_offset: 0,
        inherited: true,
        initial: Some("center"),
    },
    Case {
        name: "voice-duration",
        authored: "auto",
        canonical: "auto",
        invalid: "-1s",
        invalid_offset: 0,
        inherited: false,
        initial: Some("auto"),
    },
    Case {
        name: "voice-family",
        authored: "\"Alice\", child female 2",
        canonical: "\"Alice\", child female 2",
        invalid: "male female",
        invalid_offset: 5,
        inherited: true,
        initial: None,
    },
    Case {
        name: "voice-pitch",
        authored: "high -2st",
        canonical: "high -2st",
        invalid: "0Hz absolute",
        invalid_offset: 0,
        inherited: true,
        initial: Some("medium"),
    },
    Case {
        name: "voice-range",
        authored: "220Hz absolute",
        canonical: "220hz absolute",
        invalid: "0Hz absolute",
        invalid_offset: 0,
        inherited: true,
        initial: Some("medium"),
    },
    Case {
        name: "voice-rate",
        authored: "100% fast",
        canonical: "fast",
        invalid: "-1%",
        invalid_offset: 0,
        inherited: true,
        initial: Some("normal"),
    },
    Case {
        name: "voice-stress",
        authored: "STRONG",
        canonical: "strong",
        invalid: "weak",
        invalid_offset: 0,
        inherited: true,
        initial: Some("normal"),
    },
    Case {
        name: "voice-volume",
        authored: "-0dB loud",
        canonical: "loud",
        invalid: "silent 3db",
        invalid_offset: 7,
        inherited: true,
        initial: Some("medium"),
    },
];

const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name).unwrap()
}

fn declaration(name: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{text}!important"));
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    let [value] = report.syntax().as_slice() else {
        panic!("one Speech declaration")
    };
    value.clone()
}

fn checked(name: &str, values: CssComponentValues) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property(name)),
        values,
        CssImportance::Important,
    )
    .unwrap()
}

fn expanded(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed Speech contribution")
    };
    values
}

fn targets(name: &str) -> Vec<&str> {
    match name {
        "cue" => vec!["cue-before", "cue-after"],
        "pause" => vec!["pause-before", "pause-after"],
        "rest" => vec!["rest-before", "rest-after"],
        _ => vec![name],
    }
}

// This adapter only dispatches to existing owning public serializers. Expected
// bytes above are source-derived, never captured from the implementation.
fn specified(
    source: &CssDeclaration,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String, CssSpecifiedValueSerializationError> {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Cue(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::CueAfter(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::CueBefore(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::Pause(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::PauseAfter(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::PauseBefore(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::Rest(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::RestAfter(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::RestBefore(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::Speak(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::SpeakAs(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::VoiceBalance(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::VoiceDuration(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::VoiceFamily(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::VoicePitch(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::VoiceRange(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::VoiceRate(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::VoiceStress(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::VoiceVolume(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        _ => panic!("fixture dispatch covers exactly the source's Speech properties"),
    }
}

fn terminal_specified(value: &CssLonghandValue) -> String {
    match value.view() {
        CssLonghandValueRef::CueBefore(value) | CssLonghandValueRef::CueAfter(value) => {
            value.serialize_specified().unwrap()
        }
        CssLonghandValueRef::PauseBefore(value)
        | CssLonghandValueRef::PauseAfter(value)
        | CssLonghandValueRef::RestBefore(value)
        | CssLonghandValueRef::RestAfter(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::Speak(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::SpeakAs(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::VoiceBalance(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::VoiceDuration(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::VoiceFamily(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::VoicePitch(value) | CssLonghandValueRef::VoiceRange(value) => {
            value.serialize_specified().unwrap()
        }
        CssLonghandValueRef::VoiceRate(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::VoiceStress(value) => value.serialize_specified().unwrap(),
        CssLonghandValueRef::VoiceVolume(value) => value.serialize_specified().unwrap(),
        _ => panic!("Speech terminal"),
    }
}

fn ordinary_block() -> String {
    CASES
        .iter()
        .map(|case| format!("{}:{}!important;", case.name, case.authored))
        .collect()
}

#[test]
fn all_nineteen_ordinary_speech_properties_compose_in_attributes_and_stylesheets() {
    let text = ordinary_block();
    let report = parse_style_attribute(&text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 19);
    assert!(validate_style_attribute(&text).is_ok());
    let sheet_text = format!(".speech{{{text}}}");
    let sheet = parse_sheet(&sheet_text);
    assert!(sheet.is_clean(), "{:?}", sheet.diagnostics());
    assert!(validate_sheet(&sheet_text).is_ok());
    let [CssRule::Style(rule)] = sheet.syntax().rules() else {
        panic!("one composed Speech rule")
    };
    assert_eq!(rule.declarations().len(), 19);
    let mut terminal_count = 0;
    for ((source, sheet_source), case) in report
        .syntax()
        .iter()
        .zip(rule.declarations().iter())
        .zip(&CASES)
    {
        for source in [source, sheet_source] {
            let known = source.known().unwrap();
            assert_eq!(known.property(), property(case.name));
            assert!(known.property_value().is_some());
            assert!(known.global().is_none());
            assert!(known.substitution_dependent().is_none());
            assert_eq!(source.importance(), CssImportance::Important);
            assert_eq!(
                source.value_components().serialize().unwrap().as_css(),
                case.authored
            );
            assert_eq!(
                specified(source, CssSpecifiedValueSerializationLimits::default()).unwrap(),
                case.canonical
            );
        }
        let values = expanded(source);
        let names = targets(case.name);
        assert_eq!(values.items().len(), names.len());
        terminal_count += values.items().len();
        for (item, name) in values.items().iter().zip(names) {
            assert_eq!(item.property(), property(name));
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
    assert_eq!(terminal_count, 22); // 16 longhands plus three before/after pairs.
}

#[test]
fn every_speech_property_crosses_strict_construction_and_owning_bounded_canonical_serialization() {
    for case in &CASES {
        let source = declaration(case.name, case.authored);
        let components = source.value_components().clone();
        let constructed = checked(case.name, components.clone());
        assert!(constructed.position().is_none());
        assert_eq!(constructed.importance(), CssImportance::Important);
        assert_eq!(constructed.value_components(), &components);
        assert_eq!(
            constructed.known().unwrap().property_value(),
            source.known().unwrap().property_value()
        );
        let limits = CssSpecifiedValueSerializationLimits::default();
        let exact = CssSpecifiedValueSerializationLimits::new(
            limits.max_input_nodes(),
            limits.max_projection_nodes(),
            case.canonical.len(),
        );
        assert_eq!(specified(&constructed, exact).unwrap(), case.canonical);
        let too_small = CssSpecifiedValueSerializationLimits::new(
            limits.max_input_nodes(),
            limits.max_projection_nodes(),
            case.canonical.len() - 1,
        );
        assert_eq!(
            specified(&constructed, too_small).unwrap_err().kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(constructed.value_components(), &components);
        assert_eq!(specified(&constructed, exact).unwrap(), case.canonical);
        let canonical = declaration(case.name, case.canonical);
        assert_eq!(specified(&canonical, exact).unwrap(), case.canonical);
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(property(case.name)),
                parse_component_values(case.invalid).unwrap(),
                CssImportance::Normal
            )
            .is_err()
        );
    }
}

#[test]
fn all_five_css_wide_keywords_compose_across_all_nineteen_properties_and_their_terminals() {
    for (text, keyword) in GLOBALS {
        let block: String = CASES
            .iter()
            .map(|case| format!("{}:{text}!important;", case.name))
            .collect();
        let report = parse_style_attribute(&block);
        assert!(report.is_clean());
        assert_eq!(report.syntax().len(), 19);
        assert!(validate_style_attribute(&block).is_ok());
        assert!(validate_sheet(&format!(".speech{{{block}}}")).is_ok());
        let mut terminals = 0;
        for (source, case) in report.syntax().iter().zip(&CASES) {
            let known = source.known().unwrap();
            assert_eq!(known.property(), property(case.name));
            assert_eq!(known.global(), Some(keyword));
            assert!(known.property_value().is_none());
            assert_eq!(
                source.value_components().serialize().unwrap().as_css(),
                text
            );
            let constructed = checked(case.name, source.value_components().clone());
            assert_eq!(constructed.known().unwrap().global(), Some(keyword));
            let values = expanded(source);
            let names = targets(case.name);
            assert_eq!(values.items().len(), names.len());
            terminals += names.len();
            for (item, name) in values.items().iter().zip(names) {
                assert_eq!(item.property(), property(name));
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
            assert!(
                parse_property_value(
                    CssPropertyNameRef::Known(property(case.name)),
                    parse_component_values(&format!("{text} extra")).unwrap(),
                    CssImportance::Normal
                )
                .is_err()
            );
        }
        assert_eq!(terminals, 22);
    }
}

#[test]
fn the_three_real_shorthands_expand_ordered_and_shared_authored_components() {
    for (name, text, before_css, after_css) in [
        (
            "cue",
            "url(\"a\") -3db",
            "url(\"a\") -3db",
            "url(\"a\") -3db",
        ),
        ("cue", "url(\"a\") -3db none", "url(\"a\") -3db", "none"),
        ("pause", "250ms", "0.25s", "0.25s"),
        ("pause", "250ms strong", "0.25s", "strong"),
        ("rest", "weak", "weak", "weak"),
        ("rest", "weak 1s", "weak", "1s"),
    ] {
        let source = declaration(name, text);
        let values = expanded(&source);
        let [before, after] = values.items() else {
            panic!("two ordered terminals")
        };
        let names = targets(name);
        for (item, target, expected) in
            [(before, names[0], before_css), (after, names[1], after_css)]
        {
            assert_eq!(item.property(), property(target));
            assert_eq!(terminal_specified(item.ordinary_value().unwrap()), expected);
            assert!(item.source().same_occurrence(&source));
        }
        // Full child payloads retain original origins. Supplying the same
        // authored roots through strict shorthand construction reproduces them.
        let constructed = checked(name, source.value_components().clone());
        let expected = expanded(&constructed);
        for (item, expected) in values.items().iter().zip(expected.items()) {
            assert_eq!(item.value(), expected.value());
        }
    }
}

#[test]
fn all_properties_reenter_ordinary_and_global_values_atomically_with_both_occurrence_origins() {
    let block: String = CASES
        .iter()
        .map(|case| format!("{}:var(--speech)!important;", case.name))
        .collect();
    let report = parse_style_attribute(&block);
    assert!(report.is_clean());
    assert_eq!(report.syntax().len(), 19);
    for (source, case) in report.syntax().iter().zip(&CASES) {
        let CssExpansion::Pending(handle) = expand_declaration(source).unwrap() else {
            panic!("pending Speech property")
        };
        let replacement = parse_component_values(case.authored).unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed ordinary Speech")
        };
        let constructed = checked(case.name, replacement.clone());
        assert_eq!(
            specified(
                &constructed,
                CssSpecifiedValueSerializationLimits::default()
            )
            .unwrap(),
            case.canonical
        );
        let expected = expanded(&constructed);
        assert_eq!(values.items().len(), expected.items().len());
        for (item, expected) in values.items().iter().zip(expected.items()) {
            assert_eq!(item.property(), expected.property());
            assert_eq!(item.value(), expected.value());
            assert!(item.source().same_occurrence(source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
        for (text, keyword) in GLOBALS {
            let replacement = parse_component_values(text).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed global Speech")
            };
            let names = targets(case.name);
            assert_eq!(values.items().len(), names.len());
            for (item, name) in values.items().iter().zip(names) {
                assert_eq!(item.property(), property(name));
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(source));
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
        }
        for invalid in [
            case.invalid.to_owned(),
            "inherit extra".to_owned(),
            format!("{};color:red", case.authored),
        ] {
            assert!(
                matches!(
                    handle
                        .reenter(parse_component_values(&invalid).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ),
                "{}:{invalid}",
                case.name
            );
        }
        assert_eq!(
            handle
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        assert!(handle.reenter(replacement).is_ok());
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            "var(--speech)"
        );
    }
}

#[test]
fn source_metadata_covers_the_complete_property_index_and_exact_initial_kinds() {
    for case in &CASES {
        let known = property(case.name);
        let feature = feature_metadata(&format!("official.property.{}", case.name)).unwrap();
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        assert_eq!(feature.source().id().as_str(), "S-SPEECH1");
        assert_eq!(
            feature.source().url(),
            Some("https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/")
        );
        assert_eq!(feature.production(), format!("#propdef-{}", case.name));
        match known.metadata().unwrap().kind() {
            CssPropertyKindRef::Longhand(metadata) => {
                assert_eq!(metadata.inherited_by_default(), case.inherited);
                assert_eq!(metadata.property().known_property(), known);
                let initial = metadata.initial_value();
                assert_eq!(initial.property().known_property(), known);
                match initial.view() {
                    CssInitialValueRef::Value(value) => {
                        let expected = expanded(&declaration(case.name, case.initial.unwrap()));
                        assert_eq!(value, expected.items()[0].ordinary_value().unwrap());
                    }
                    CssInitialValueRef::UserAgent(value) => {
                        assert_eq!(case.name, "voice-family");
                        assert_eq!(value, CssUserAgentInitial::VoiceFamily);
                    }
                    _ => panic!("source initial kind"),
                }
            }
            CssPropertyKindRef::Shorthand(metadata) => {
                let expected: Vec<_> = targets(case.name).into_iter().map(property).collect();
                assert_eq!(
                    metadata
                        .members()
                        .iter()
                        .map(|value| value.known_property())
                        .collect::<Vec<_>>(),
                    expected
                );
                assert_eq!(
                    metadata
                        .settable_members()
                        .iter()
                        .map(|value| value.known_property())
                        .collect::<Vec<_>>(),
                    expected
                );
                assert!(metadata.reset_only_members().is_empty());
                assert!(!metadata.is_legacy());
            }
            _ => panic!("Speech property is a terminal or ordered shorthand"),
        }
    }
}

#[test]
fn composed_invalid_occurrences_drop_atomically_with_precise_utf16_crlf_provenance() {
    let mut body = "/*😀*/\r\n/*😀*/color:red;".to_owned();
    let mut spans = Vec::new();
    for case in &CASES {
        let start = body.len();
        let value_start = start + case.name.len() + 1;
        body.push_str(&format!("{}:{};", case.name, case.invalid));
        spans.push((start, body.len(), value_start + case.invalid_offset));
        body.push_str(&format!("{}:{}!important;", case.name, case.authored));
    }
    body.push_str("height:1px");
    let report = parse_style_attribute(&body);
    assert_eq!(report.syntax().len(), 21);
    assert_eq!(report.diagnostics().len(), 19);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        report.syntax()[20].known().unwrap().property(),
        CssKnownProperty::Height
    );
    for ((source, case), diagnostic) in report.syntax()[1..20]
        .iter()
        .zip(&CASES)
        .zip(report.diagnostics())
    {
        assert_eq!(source.known().unwrap().property(), property(case.name));
        assert_eq!(
            specified(source, CssSpecifiedValueSerializationLimits::default()).unwrap(),
            case.canonical
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
    }
    for (diagnostic, (start, end, responsible)) in report.diagnostics().iter().zip(&spans) {
        assert_eq!(diagnostic.span().start().byte_offset().value(), *start);
        assert_eq!(diagnostic.span().end().byte_offset().value(), *end);
        let position = diagnostic.error().position();
        assert_eq!(position.byte_offset().value(), *responsible);
        let prefix = &body[..*responsible];
        assert_eq!(
            position.line().value(),
            u32::try_from(prefix.bytes().filter(|byte| *byte == b'\n').count()).unwrap()
        );
        assert_eq!(
            position.column().value(),
            u32::try_from(prefix.rsplit('\n').next().unwrap().encode_utf16().count()).unwrap()
        );
    }
    assert_eq!(
        validate_style_attribute(&body).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let sheet_text = format!(".speech{{{body}}}");
    let sheet = parse_sheet(&sheet_text);
    let [CssRule::Style(rule)] = sheet.syntax().rules() else {
        panic!("retained composed style rule")
    };
    assert_eq!(rule.declarations().len(), 21);
    assert_eq!(sheet.diagnostics().len(), 19);
    for (diagnostic, (start, end, responsible)) in sheet.diagnostics().iter().zip(spans) {
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            start + ".speech{".len()
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            end + ".speech{".len()
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible + ".speech{".len()
        );
    }
    assert_eq!(
        validate_sheet(&sheet_text).unwrap_err().diagnostics(),
        sheet.diagnostics()
    );
}
