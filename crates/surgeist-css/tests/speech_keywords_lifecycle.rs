#![forbid(unsafe_code)]
//! Authored Speech 1 keyword properties through existing public boundaries.
//! Independent grammar authority: retained CRD-css-speech-1-20230214,
//! sections 3.1, 7.1 and 7.2. Speech rendering and contextual `auto` resolution
//! remain downstream. Counter Styles `speak-as` is a separate descriptor.

use surgeist_css::*;

fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name)
        .unwrap_or_else(|| panic!("Speech 1 property `{name}` must be admitted"))
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one complete Speech declaration: {source}");
    };
    assert_eq!(declaration.known().unwrap().property(), property(name));
    assert_eq!(declaration.importance(), CssImportance::Important);
    declaration.clone()
}

fn assert_ordinary_admission(name: &str, value: &str) {
    let parsed = declaration(name, value);
    let known = parsed.known().unwrap();
    assert!(known.property_value().is_some(), "{name}:{value}");
    assert!(known.global().is_none());
    assert!(known.substitution_dependent().is_none());
    let components = parse_component_values(value).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(property(name)),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"));
    assert_eq!(checked.known().unwrap().property(), property(name));
    assert!(checked.known().unwrap().property_value().is_some());
    assert_eq!(checked.value_components(), &components);
    assert_eq!(checked.importance(), CssImportance::Important);
    assert!(checked.position().is_none());
    assert!(validate_style_attribute(&format!("{name}:{value}")).is_ok());
    assert!(validate_sheet(&format!(".a{{{name}:{value}}}")).is_ok());
}

fn permutations(parts: &[&str]) -> Vec<String> {
    if parts.is_empty() {
        return vec![String::new()];
    }
    let mut values = Vec::new();
    for (index, first) in parts.iter().enumerate() {
        let rest: Vec<_> = parts
            .iter()
            .enumerate()
            .filter_map(|(other, part)| (other != index).then_some(*part))
            .collect();
        for suffix in permutations(&rest) {
            values.push(if suffix.is_empty() {
                (*first).to_owned()
            } else {
                format!("{first} {suffix}")
            });
        }
    }
    values
}

#[test]
fn speak_admits_exactly_the_three_speech_keywords_as_ordinary_values() {
    for value in ["auto", "never", "always"] {
        assert_ordinary_admission("speak", value);
    }
}

#[test]
fn speak_as_admits_normal_and_every_nonempty_unordered_modifier_combination() {
    assert_ordinary_admission("speak-as", "normal");
    for spell_out in [false, true] {
        for digits in [false, true] {
            for punctuation in [None, Some("literal-punctuation"), Some("no-punctuation")] {
                let mut parts = Vec::new();
                if spell_out {
                    parts.push("spell-out");
                }
                if digits {
                    parts.push("digits");
                }
                if let Some(punctuation) = punctuation {
                    parts.push(punctuation);
                }
                if !parts.is_empty() {
                    for value in permutations(&parts) {
                        assert_ordinary_admission("speak-as", &value);
                    }
                }
            }
        }
    }
}

#[test]
fn speech_keywords_use_decoded_case_insensitive_identifiers_and_preserve_tokens() {
    for (name, value, canonical) in [
        ("SPEAK", "ALWAYS", "speak"),
        (r"sp\65 ak", r"n\65 ver", "speak"),
        ("SPEAK-AS", "DIGITS SPELL-OUT NO-PUNCTUATION", "speak-as"),
        (
            r"speak\2d as",
            r"d\69 gits l\69 teral-punctuation",
            "speak-as",
        ),
        ("speak-as", "digits/**/spell-out", "speak-as"),
    ] {
        let source = format!("{name}:{value}!important");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [parsed] = report.syntax().as_slice() else {
            panic!("one declaration")
        };
        assert_eq!(parsed.known().unwrap().property(), property(canonical));
        assert_eq!(parsed.importance(), CssImportance::Important);
        let serialized = parsed.value_components().serialize().unwrap();
        assert_eq!(serialized.as_css(), value);
        let checked = parse_property_value(
            CssPropertyNameRef::Known(property(canonical)),
            parsed.value_components().clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(checked.value_components(), parsed.value_components());
    }
}

#[test]
fn foreign_keywords_duplicates_and_exclusive_speech_modes_are_rejected_atomically() {
    for (name, values) in [
        (
            "speak",
            &[
                "",
                "none",
                "normal",
                "auto always",
                "never never",
                "words",
                "default",
                "\"always\"",
                "1",
                "always()",
            ] as &[&str],
        ),
        (
            "speak-as",
            &[
                "",
                "none",
                "auto",
                "always",
                "words",
                "numbers",
                "bullets",
                "default",
                "normal digits",
                "normal normal",
                "digits digits",
                "spell-out spell-out",
                "literal-punctuation no-punctuation",
                "no-punctuation literal-punctuation",
                "digits literal-punctuation literal-punctuation",
                "digits, spell-out",
                "\"digits\"",
                "d\\69 gits DIGITS",
            ] as &[&str],
        ),
    ] {
        for value in values {
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
                panic!("one dropped Speech declaration: {source}")
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
            let failure = validate_style_attribute(&source).unwrap_err();
            assert_eq!(failure.diagnostics(), report.diagnostics());
            let checked = parse_property_value(
                CssPropertyNameRef::Known(property(name)),
                parse_component_values(value).unwrap(),
                CssImportance::Normal,
            );
            assert!(checked.is_err(), "checked {name}:{value}");
        }
    }
}

#[test]
fn speech_global_keywords_keep_inherited_terminal_metadata_and_source_identity() {
    for name in ["speak", "speak-as"] {
        let known = property(name);
        let metadata = known.metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("Speech keyword property is a terminal longhand")
        };
        assert!(longhand.inherited_by_default());
        assert_eq!(longhand.property().known_property(), known);
        assert_eq!(longhand.initial_value().property().known_property(), known);
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
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("completed terminal")
            };
            let [value] = values.items() else {
                panic!("one terminal")
            };
            assert_eq!(value.property(), known);
            assert_eq!(value.value(), CssContributionValueRef::Global(keyword));
            assert!(value.source().same_occurrence(&source));
            assert_eq!(value.source().importance(), CssImportance::Important);
            let checked = parse_property_value(
                CssPropertyNameRef::Known(known),
                parse_component_values(text).unwrap(),
                CssImportance::Important,
            )
            .unwrap();
            assert_eq!(checked.known().unwrap().global(), Some(keyword));
            let mixed = format!("{text} digits");
            let report = parse_style_attribute(&format!("{name}:{mixed};color:red"));
            assert_eq!(report.syntax().len(), 1, "{name}:{mixed}");
            assert_eq!(report.diagnostics().len(), 1, "{name}:{mixed}");
            assert!(
                parse_property_value(
                    CssPropertyNameRef::Known(known),
                    parse_component_values(&mixed).unwrap(),
                    CssImportance::Normal,
                )
                .is_err(),
                "{name}:{mixed}"
            );
        }
    }
}

#[test]
fn speech_substitution_reentry_checks_the_original_grammar_and_retains_replacement_origins() {
    for (name, valid, invalid) in [
        ("speak", "always", "normal"),
        (
            "speak-as",
            "no-punctuation digits spell-out",
            "normal digits",
        ),
    ] {
        let source = declaration(name, "var(--speech)");
        assert_eq!(
            source
                .known()
                .unwrap()
                .substitution_dependent()
                .unwrap()
                .as_css(),
            "var(--speech)"
        );
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("symbolic Speech declaration")
        };
        for text in [valid, "inherit"] {
            let replacement = parse_component_values(text).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("one replacement terminal")
            };
            let [value] = values.items() else {
                panic!("one terminal")
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
            "always!important",
            "always;color:red",
        ] {
            let error = handle
                .reenter(parse_component_values(text).unwrap())
                .unwrap_err();
            assert!(
                matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(_)),
                "{name}:{text}: {error:?}"
            );
        }
        let error = handle
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err();
        assert_eq!(error.kind(), &CssExpansionErrorKind::ResidualSubstitution);
        // Failure does not consume the source or prevent a subsequent valid reentry.
        assert!(
            handle
                .reenter(parse_component_values(valid).unwrap())
                .is_ok()
        );
    }
}

#[test]
fn conflicting_speech_keyword_diagnostics_identify_the_authored_token_and_whole_declaration() {
    let prefix = "color:red;/*😀*/";
    let unit = "speak-as:normal digits;";
    let source = format!("{prefix}{unit}height:1px");
    let report = parse_style_attribute(&source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("one conflict diagnostic")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let responsible = source.find("digits").unwrap();
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        prefix.len()
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        prefix.len() + unit.len()
    );
    let failure = validate_style_attribute(&source).unwrap_err();
    assert_eq!(failure.diagnostics(), report.diagnostics());
    let components = parse_component_values("normal digits").unwrap();
    let token = components
        .items()
        .iter()
        .find(|value| {
            matches!(
                value.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Ident("digits"))
            )
        })
        .unwrap();
    let error = parse_property_value(
        CssPropertyNameRef::Known(property("speak-as")),
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
    let failure = validate_sheet(&sheet_source).unwrap_err();
    assert_eq!(failure.diagnostics(), sheet.diagnostics());
    let [CssRule::Style(rule)] = sheet.syntax().rules() else {
        panic!("style rule")
    };
    assert_eq!(rule.declarations().len(), 2);
    assert_eq!(
        sheet.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        responsible + 3
    );
}
