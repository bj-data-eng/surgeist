#![forbid(unsafe_code)]
//! Speech 1 voice-family / voice-stress through existing public CSS boundaries.
//! Independent authority: retained CRD-css-speech-1-20230214 §§11.1 and 11.5.
//! Reserved-token cases follow Speech's quotation rule and its borrowed
//! family-name/custom-ident grammar: retained Fonts 4 `font-family-name-syntax`
//! and Values 4 `identifier-value`. CSS-wide/default exclusions apply per token;
//! gender/preserve tokens cannot be claimed by an unquoted family name.
//! Authored names remain distinct from downstream computed string conversion;
//! no voice selection, language preservation or acoustic execution occurs here.
//! Raw component serialization preserves tokens, not canonical model output.

use surgeist_css::*;

const PROPERTIES: [&str; 2] = ["voice-family", "voice-stress"];

fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name)
        .unwrap_or_else(|| panic!("Speech 1 property `{name}` must be recognized"))
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one complete declaration: {source}")
    };
    assert_eq!(declaration.known().unwrap().property(), property(name));
    assert_eq!(declaration.importance(), CssImportance::Important);
    declaration.clone()
}

fn expanded(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("complete terminal expansion")
    };
    values
}

fn assert_admitted(name: &str, value: &str) {
    let source = declaration(name, value);
    let known = source.known().unwrap();
    assert!(known.property_value().is_some());
    assert!(known.global().is_none());
    assert!(known.substitution_dependent().is_none());
    let components = parse_component_values(value).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(property(name)),
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
        source.value_components().serialize().unwrap().as_css(),
        value
    );
    assert!(validate_style_attribute(&format!("{name}:{value}")).is_ok());
    assert!(validate_sheet(&format!(".speech{{{name}:{value}}}")).is_ok());
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
fn voice_stress_admits_five_distinct_ordinary_keywords() {
    let mut values = Vec::new();
    for text in ["normal", "strong", "moderate", "none", "reduced"] {
        assert_admitted("voice-stress", text);
        let expanded = expanded(&declaration("voice-stress", text));
        let value = expanded.items()[0].ordinary_value().unwrap().clone();
        assert!(!values.contains(&value), "distinct stress value: {text}");
        values.push(value);
    }
}

#[test]
fn voice_family_admits_named_priorities_and_lone_preserve_without_visual_generic_semantics() {
    for value in [
        "Mike",
        "john doe",
        "carlos2",
        "child",
        "young",
        "old",
        "young Smith",
        "serif",
        "sans-serif",
        "menu",
        "\"valley girl\"",
        "''",
        "\"Henry\\9 the-8th\"",
        "Mike, \"valley girl\", young female, neutral 2",
        "preserve",
        r"john\/doe",
        r"\31 stvoice",
        r"Mike\ Smith",
    ] {
        assert_admitted("voice-family", value);
    }
}

#[test]
fn generic_voice_grammar_preserves_ordered_age_gender_and_exact_positive_variant_indices() {
    for gender in ["male", "female", "neutral"] {
        for age in [None, Some("child"), Some("young"), Some("old")] {
            let prefix = age.map_or_else(|| gender.to_owned(), |age| format!("{age} {gender}"));
            assert_admitted("voice-family", &prefix);
            for variant in [
                "1",
                "+0002",
                "9007199254740993",
                "9999999999999999999999999999999999999999",
            ] {
                assert_admitted("voice-family", &format!("{prefix} {variant}"));
            }
        }
    }
    assert_admitted("voice-family", "child male 2, old neutral 1, female");
}

#[test]
fn generic_voice_integer_math_retains_authored_calculation_phase() {
    // The shared integer production admits integer-root math; this does not
    // select a voice or perform downstream computed variant range handling.
    for value in ["male calc(1 + 1)", "old female min(2, 3)"] {
        assert_admitted("voice-family", value);
    }
}

#[test]
fn quoted_voice_names_admit_gender_preserve_css_wide_and_reserved_default_words() {
    for name in [
        "male",
        "female",
        "neutral",
        "preserve",
        "default",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
    ] {
        assert_admitted("voice-family", &format!("\"{name}\""));
        assert_admitted("voice-family", &format!("Mike, '{name}'"));
    }
    assert_admitted(
        "voice-family",
        "\"young male\", \"male john\", \"default family\"",
    );
}

#[test]
fn voice_properties_decode_keyword_case_escapes_and_preserve_authored_tokens() {
    for (name, canonical, value) in [
        ("VOICE-STRESS", "voice-stress", "MODERATE"),
        (r"voice\2d stress", "voice-stress", r"r\65 duced"),
        ("VOICE-FAMILY", "voice-family", "PRESERVE"),
        (r"voice\2d family", "voice-family", "YOUNG/**/FEMALE +0002"),
        (
            "voice-family",
            "voice-family",
            "Mike/**/Smith, 'preserve', NEUTRAL",
        ),
    ] {
        let report = parse_style_attribute(&format!("{name}:{value}!important"));
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
                .serialize_with_limit(value.len())
                .unwrap()
                .as_css(),
            value
        );
        assert!(components.serialize_with_limit(value.len() - 1).is_err());
        assert_eq!(components.serialize().unwrap().as_css(), value);
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
fn voice_property_metadata_is_inherited_with_intrinsic_normal_or_user_agent_voice_initial() {
    for name in PROPERTIES {
        let known = property(name);
        let metadata = known.metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("terminal metadata")
        };
        assert!(longhand.inherited_by_default());
        assert_eq!(longhand.property().known_property(), known);
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), known);
        if name == "voice-family" {
            let CssInitialValueRef::UserAgent(context) = initial.view() else {
                panic!("implementation-dependent voice initial")
            };
            assert_eq!(context.property().known_property(), known);
        } else {
            let CssInitialValueRef::Value(value) = initial.view() else {
                panic!("normal stress initial")
            };
            assert_eq!(
                value,
                expanded(&declaration(name, "normal")).items()[0]
                    .ordinary_value()
                    .unwrap()
            );
        }
    }
}

#[test]
fn voice_globals_are_whole_value_keywords_with_one_inherited_terminal_contribution() {
    for name in PROPERTIES {
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text);
            assert_eq!(source.known().unwrap().global(), Some(keyword));
            assert!(source.known().unwrap().property_value().is_none());
            let values = expanded(&source);
            let [item] = values.items() else {
                panic!("one global terminal")
            };
            assert_eq!(item.property(), property(name));
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.source().same_occurrence(&source));
            let checked = parse_property_value(
                CssPropertyNameRef::Known(property(name)),
                parse_component_values(text).unwrap(),
                CssImportance::Important,
            )
            .unwrap();
            assert_eq!(checked.known().unwrap().global(), Some(keyword));
            assert!(!parse_style_attribute(&format!("{name}:{text} extra")).is_clean());
        }
    }
}

#[test]
fn invalid_voice_grammars_drop_one_complete_declaration_without_losing_valid_siblings() {
    for (name, invalid) in [
        (
            "voice-family",
            &[
                "",
                ", Mike",
                "Mike,",
                "Mike,,female",
                "preserve, Mike",
                "Mike, preserve",
                "preserve Mike",
                "Mike preserve",
                "default",
                "default Mike",
                "Mike DEFAULT",
                "inherit Mike",
                "Mike initial",
                "Mike, inherit",
                "Mike, default",
                "male Mike",
                "Mike male",
                "female young",
                "neutral female",
                "young male old",
                "male 0",
                "male -0",
                "male -1",
                "male 1.0",
                "male 1e0",
                "male 10%",
                "male 1px",
                "1 male",
                "male 1 2",
                "child 2",
                "john/doe",
                "john \"doe\"",
                "#john",
                "john@doe",
                "young male calc(1px)",
            ] as &[&str],
        ),
        (
            "voice-stress",
            &[
                "",
                "auto",
                "medium",
                "weak",
                "preserve",
                "normal strong",
                "normal, strong",
                "1",
                "50%",
                "\"normal\"",
                "strong()",
            ] as &[&str],
        ),
    ] {
        for value in invalid {
            let source = format!("color:red;{name}:{value};height:1px");
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
                panic!("one dropped voice declaration: {source}")
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
                validate_style_attribute(&source).unwrap_err().diagnostics(),
                report.diagnostics()
            );
            assert!(
                parse_property_value(
                    CssPropertyNameRef::Known(property(name)),
                    parse_component_values(value).unwrap(),
                    CssImportance::Normal
                )
                .is_err(),
                "checked {name}:{value}"
            );
        }
    }
}

#[test]
fn voice_substitution_reentry_checks_the_original_grammar_atomically_and_keeps_origins() {
    for (name, valid, invalid) in [
        (
            "voice-family",
            "\"preserve\", child male +0002, serif",
            "male 0",
        ),
        ("voice-stress", "reduced", "normal strong"),
    ] {
        let source = declaration(name, "var(--voice)");
        assert_eq!(
            source
                .known()
                .unwrap()
                .substitution_dependent()
                .unwrap()
                .as_css(),
            "var(--voice)"
        );
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("symbolic voice property")
        };
        for text in [valid, "inherit"] {
            let replacement = parse_component_values(text).unwrap();
            let checked = parse_property_value(
                CssPropertyNameRef::Known(property(name)),
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
fn invalid_voice_variant_diagnostic_keeps_exact_token_position_and_recovery_span() {
    let prefix = "color:red;/*😀*/";
    let unit = "voice-family:young male 0;";
    let source = format!("{prefix}{unit}height:1px");
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("one nonpositive variant diagnostic")
    };
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find("0;").unwrap()
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
    let sheet = parse_sheet(&sheet_source);
    let [CssRule::Style(rule)] = sheet.syntax().rules() else {
        panic!("retained rule")
    };
    assert_eq!(rule.declarations().len(), 2);
    assert_eq!(sheet.diagnostics().len(), 1);
    assert_eq!(
        sheet.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        source.find("0;").unwrap() + 3
    );
    assert_eq!(
        validate_sheet(&sheet_source).unwrap_err().diagnostics(),
        sheet.diagnostics()
    );
}

#[test]
fn voice_property_support_metadata_identifies_the_pinned_speech_productions() {
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
