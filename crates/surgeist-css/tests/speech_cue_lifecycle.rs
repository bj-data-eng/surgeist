#![forbid(unsafe_code)]
//! Existing-public-boundary RED witnesses for CSS Speech 1 cue authoring.
//! Authority: references/css-speech-1--CRD-css-speech-1-20230214--bb7b59c92564.md,
//! §§10.1 and 10.3: shared URL resource plus optional signed ordinary dB, or none;
//! none initial, noninheritance, and one/two cue shorthand expansion.
//! Omitted dB stays omitted: its implied computed 0dB is downstream. Loading,
//! volume resolution and dB math are not authored CSS responsibilities here.
//! Component serialization witnesses authored spelling. Canonical typed payload
//! serialization requires the new cue API and belongs in its functional tests.

use surgeist_css::*;

const LONGHANDS: [&str; 2] = ["cue-before", "cue-after"];
const ALL: [&str; 3] = ["cue-before", "cue-after", "cue"];

fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name)
        .unwrap_or_else(|| panic!("Speech 1 property `{name}` must be recognized"))
}

fn declaration(name: &str, text: &str) -> CssDeclaration {
    let source = format!("{name}:{text}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one complete declaration: {source}")
    };
    assert_eq!(value.known().unwrap().property(), property(name));
    assert_eq!(value.importance(), CssImportance::Important);
    value.clone()
}

fn expanded(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary/global cue must complete")
    };
    values
}

fn checked(name: &str, components: CssComponentValues) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property(name)),
        components,
        CssImportance::Important,
    )
    .unwrap()
}

fn assert_admitted(name: &str, text: &str) {
    let source = declaration(name, text);
    assert!(source.known().unwrap().property_value().is_some());
    assert!(source.known().unwrap().global().is_none());
    assert!(source.known().unwrap().substitution_dependent().is_none());
    let components = parse_component_values(text).unwrap();
    let constructed = checked(name, components.clone());
    assert!(constructed.known().unwrap().property_value().is_some());
    assert_eq!(constructed.value_components(), &components);
    assert_eq!(constructed.importance(), CssImportance::Important);
    assert!(constructed.position().is_none());
    assert!(validate_style_attribute(&format!("{name}:{text}")).is_ok());
    assert!(validate_sheet(&format!(".speech{{{name}:{text}}}")).is_ok());
}

#[test]
fn cue_longhands_admit_shared_urls_and_exact_signed_ordinary_decibels() {
    for name in LONGHANDS {
        for text in [
            "none",
            "url(a)",
            "url(\"\")",
            "url(\"a b\")",
            "src(\"a\")",
            "url(\"a\" symbolic modifier(x))",
            "url(a) 0dB",
            "url(a) -0dB",
            "url(a) +3dB",
            "url(a) -3.5dB",
            "url(a) .125dB",
            "url(a) 1e-400dB",
            "url(a) -9007199254740993dB",
            "url(a) 1e400dB",
            r"url(a) -3d\42",
            "URL(a) +3DB",
        ] {
            assert_admitted(name, text);
        }
    }
}

#[test]
fn cue_expansion_groups_url_and_optional_decibel_before_then_after() {
    for (text, before_count, after_count) in [
        ("none", 1, 0),
        ("url(a)", 1, 0),
        ("url(a) -3dB", 2, 0),
        ("none url(a)", 1, 1),
        ("url(a) -3dB none", 2, 1),
        ("url(a) url(b) +6dB", 1, 2),
        ("url(a) -3dB src(\"b\") +6dB", 2, 2),
        ("url(a)/**/-3dB/**/none", 2, 1),
    ] {
        assert_admitted("cue", text);
        let source = declaration("cue", text);
        let values = expanded(&source);
        let [before, after] = values.items() else {
            panic!("exactly before and after")
        };
        let parts: Vec<_> = source
            .value_components()
            .items()
            .iter()
            .filter(|part| {
                !matches!(
                    part.view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                        | CssComponentValueRef::Comment(_)
                )
            })
            .cloned()
            .collect();
        assert_eq!(parts.len(), before_count + after_count);
        let left = &parts[..before_count];
        let right = if after_count == 0 {
            left
        } else {
            &parts[before_count..]
        };
        for (item, name, group) in [(before, "cue-before", left), (after, "cue-after", right)] {
            // Reuse the original roots; payload equality includes URL/number origins.
            let expected = expanded(&checked(
                name,
                CssComponentValues::try_new(group.to_vec()).unwrap(),
            ));
            assert_eq!(item.property(), property(name));
            assert_eq!(item.ordinary_value(), expected.items()[0].ordinary_value());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn cue_schema_has_noninherited_none_initials_and_ordered_pair_members() {
    for name in LONGHANDS {
        let known = property(name);
        let metadata = known.metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("longhand")
        };
        assert!(!longhand.inherited_by_default());
        assert_eq!(longhand.property().known_property(), known);
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), known);
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("none initial")
        };
        let expected = expanded(&declaration(name, "none"));
        assert_eq!(initial, expected.items()[0].ordinary_value().unwrap());
    }
    let metadata = property("cue").metadata().unwrap();
    let CssPropertyKindRef::Shorthand(pair) = metadata.kind() else {
        panic!("shorthand")
    };
    let expected = [property("cue-before"), property("cue-after")];
    assert_eq!(
        pair.members()
            .iter()
            .map(|p| p.known_property())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        pair.settable_members()
            .iter()
            .map(|p| p.known_property())
            .collect::<Vec<_>>(),
        expected
    );
    assert!(pair.reset_only_members().is_empty());
    assert!(!pair.is_legacy());
}

#[test]
fn cue_globals_apply_at_the_whole_property_boundary() {
    for name in ALL {
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
            assert_eq!(values.items().len(), if name == "cue" { 2 } else { 1 });
            for item in values.items() {
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
            }
            assert_eq!(
                checked(name, parse_component_values(text).unwrap())
                    .known()
                    .unwrap()
                    .global(),
                Some(keyword)
            );
        }
    }
}

#[test]
fn cue_invalid_grammar_drops_only_its_declaration_and_checked_construction_rejects_it() {
    for name in ALL {
        let mut invalid = vec![
            "",
            "-3dB",
            "0",
            "10%",
            "1px",
            "auto",
            "normal",
            "silent",
            "\"a\"",
            "none 3dB",
            "3dB url(a)",
            "url(a) 0",
            "url(a) 2%",
            "url(a) 1px",
            "url(a) 1dB 2dB",
            "url(a) calc(1dB)",
            "url(a) calc(1 + 2)",
            "inherit url(a)",
            "url(a), none",
            "none none none",
            "url(a) url(b) url(c)",
            "url(a) -3dB url(b) +6dB none",
        ];
        if name != "cue" {
            invalid.extend(["none none", "none url(a)", "url(a) none", "url(a) url(b)"]);
        }
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
                panic!("one dropped cue: {source}")
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
                    CssImportance::Normal
                )
                .is_err(),
                "{name}:{text}"
            );
        }
    }
}

#[test]
fn cue_reentry_is_atomic_and_keeps_source_and_replacement_occurrences() {
    for name in ALL {
        let source = declaration(name, "var(--cue)");
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending cue")
        };
        let valid = if name == "cue" {
            "url(a) -3dB none"
        } else {
            "url(a) -3dB"
        };
        for text in [valid, "inherit"] {
            let replacement = parse_component_values(text).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed cue")
            };
            let expected = expanded(&checked(name, replacement.clone()));
            assert_eq!(values.items().len(), expected.items().len());
            for (item, expected) in values.items().iter().zip(expected.items()) {
                assert_eq!(item.property(), expected.property());
                assert_eq!(item.value(), expected.value());
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
        }
        for text in [
            "none 3dB",
            "none none none",
            "inherit extra",
            "url(a)!important",
            "url(a);color:red",
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
fn cue_authored_spelling_and_exact_coefficients_survive_bounded_component_serialization() {
    for (name, canonical, text) in [
        ("CUE-BEFORE", "cue-before", "NONE"),
        (
            r"c\75 e-after",
            "cue-after",
            r#"URL("a") -9007199254740993d\42"#,
        ),
        ("cue", "cue", "url(a)/**/+3DB/**/NONE"),
        ("cue-before", "cue-before", "url(a) -1e-400dB"),
    ] {
        let report = parse_style_attribute(&format!("{name}:{text}!important"));
        assert!(
            report.is_clean(),
            "{name}:{text}: {:?}",
            report.diagnostics()
        );
        let [source] = report.syntax().as_slice() else {
            panic!("one authored cue")
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
    }
}

#[test]
fn cue_url_eof_recovery_retains_shared_closure_diagnostics_but_is_not_checked_clean() {
    for name in ALL {
        let source = format!("{name}:url(a");
        let report = parse_style_attribute(&source);
        let [declaration] = report.syntax().as_slice() else {
            panic!("recovered cue URL")
        };
        assert_eq!(declaration.known().unwrap().property(), property(name));
        assert!(declaration.known().unwrap().property_value().is_some());
        let [diagnostic] = report.diagnostics() else {
            panic!("one shared URL closure")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        for position in [
            diagnostic.error().position(),
            diagnostic.span().start(),
            diagnostic.span().end(),
        ] {
            assert_eq!(position.byte_offset().value(), source.len());
        }
        assert_eq!(
            validate_style_attribute(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(property(name)),
                declaration.value_components().clone(),
                CssImportance::Normal
            )
            .is_err()
        );
    }
}

#[test]
fn cue_invalid_decibel_diagnostics_keep_token_position_and_balanced_declaration_span() {
    for name in ALL {
        let prefix = "color:red;/*😀*/";
        let unit = format!("{name}:url(a) 1px;");
        let source = format!("{prefix}{unit}height:1px");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid dB diagnostic")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_ne!(diagnostic.error().code(), CssErrorCode::UnsupportedProperty);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find("1px").unwrap()
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
            panic!("retained style rule")
        };
        assert_eq!(rule.declarations().len(), 2);
        assert_eq!(
            validate_sheet(&sheet_source).unwrap_err().diagnostics(),
            sheet.diagnostics()
        );
    }
}

#[test]
fn cue_support_metadata_identifies_the_selected_speech_production() {
    for name in ALL {
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
