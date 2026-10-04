#![forbid(unsafe_code)]
//! Authored pause/rest lifecycle through existing public CSS boundaries.
//! Independent authority: references/css-speech-1--CRD-css-speech-1-20230214--bb7b59c92564.md,
//! §§8.1–8.2 and 9.1–9.2: nonnegative time or symbolic strength, `none` initial,
//! noninheritance, and before/after shorthand order. Strength durations, pause
//! collapse, additive rest execution and computed-time range handling are downstream.
//! Component serialization below preserves tokens; canonical typed specified
//! serialization will accompany the functional new model API.

use surgeist_css::*;

const LONGHANDS: [&str; 4] = ["pause-before", "pause-after", "rest-before", "rest-after"];
const ALL: [&str; 6] = [
    "pause-before",
    "pause-after",
    "pause",
    "rest-before",
    "rest-after",
    "rest",
];
const STRENGTHS: [&str; 6] = ["none", "x-weak", "weak", "medium", "strong", "x-strong"];

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
        panic!("ordinary/global declaration must complete")
    };
    values
}

fn ordinary(name: &str, text: &str) -> CssLonghandValue {
    let values = expanded(&declaration(name, text));
    let [value] = values.items() else {
        panic!("one terminal")
    };
    value.ordinary_value().unwrap().clone()
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
    let checked = parse_property_value(
        CssPropertyNameRef::Known(property(name)),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{text}: {error:?}"));
    assert!(checked.known().unwrap().property_value().is_some());
    assert_eq!(checked.known().unwrap().property(), property(name));
    assert_eq!(checked.value_components(), &components);
    assert_eq!(checked.importance(), CssImportance::Important);
    assert!(checked.position().is_none());
    assert!(validate_style_attribute(&format!("{name}:{text}")).is_ok());
    assert!(validate_sheet(&format!(".speech{{{name}:{text}}}")).is_ok());
}

#[test]
fn pause_rest_longhands_admit_strengths_and_exact_nonnegative_absolute_times() {
    for name in LONGHANDS {
        for value in STRENGTHS {
            assert_admitted(name, value);
        }
        for value in [
            "0s",
            "-0ms",
            "+3s",
            "250ms",
            ".125s",
            "1e-400s",
            "9007199254740993ms",
        ] {
            assert_admitted(name, value);
        }
    }
}

#[test]
fn pause_rest_time_calculations_remain_authored_without_computed_range_evaluation() {
    // Existing CSS time calculation policy preserves dimensionally valid
    // symbolic trees, including deferred negative results; no speech execution.
    for name in ALL {
        for value in ["calc(1s + 250ms)", "calc(-1s)", "min(1s, 250ms)"] {
            assert_admitted(name, value);
            let source = declaration(name, value);
            assert_eq!(
                source.value_components().serialize().unwrap().as_css(),
                value
            );
        }
    }
}

#[test]
fn pause_rest_shorthands_expand_one_value_twice_and_two_values_before_then_after() {
    for (name, before, after) in [
        ("pause", "pause-before", "pause-after"),
        ("rest", "rest-before", "rest-after"),
    ] {
        for (text, first, second) in [
            ("none", "none", "none"),
            ("strong", "strong", "strong"),
            ("250ms", "250ms", "250ms"),
            ("weak x-strong", "weak", "x-strong"),
            ("+3s none", "+3s", "none"),
            ("medium 250ms", "medium", "250ms"),
            ("calc(1s + 250ms) weak", "calc(1s + 250ms)", "weak"),
        ] {
            assert_admitted(name, text);
            let source = declaration(name, text);
            let values = expanded(&source);
            let [left, right] = values.items() else {
                panic!("exactly before and after: {name}:{text}")
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
                .collect();
            assert!((1..=2).contains(&parts.len()));
            for (item, terminal, expected, component) in [
                (left, before, first, parts[0]),
                (right, after, second, *parts.last().unwrap()),
            ] {
                // Reuse exact child origins: authored value equality includes
                // provenance, so independently parsed occurrences are unequal.
                let component = CssComponentValues::try_new(vec![component.clone()]).unwrap();
                assert_eq!(component.serialize().unwrap().as_css(), expected);
                let expected = expanded(&checked(terminal, component));
                assert_eq!(item.property(), property(terminal));
                assert_eq!(
                    item.ordinary_value().unwrap(),
                    expected.items()[0].ordinary_value().unwrap()
                );
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert!(item.replacement_components().is_none());
            }
        }
    }
}

#[test]
fn pause_rest_schema_has_noninherited_none_initials_and_ordered_shorthand_members() {
    for name in LONGHANDS {
        let known = property(name);
        let metadata = known.metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("terminal metadata: {name}")
        };
        assert!(!longhand.inherited_by_default());
        assert_eq!(longhand.property().known_property(), known);
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), known);
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("intrinsic none initial")
        };
        assert_eq!(initial, &ordinary(name, "none"));
    }
    for (name, before, after) in [
        ("pause", "pause-before", "pause-after"),
        ("rest", "rest-before", "rest-after"),
    ] {
        let metadata = property(name).metadata().unwrap();
        let CssPropertyKindRef::Shorthand(shorthand) = metadata.kind() else {
            panic!("pair shorthand metadata")
        };
        let expected = [property(before), property(after)];
        assert_eq!(
            shorthand
                .members()
                .iter()
                .map(|p| p.known_property())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            shorthand
                .settable_members()
                .iter()
                .map(|p| p.known_property())
                .collect::<Vec<_>>(),
            expected
        );
        assert!(shorthand.reset_only_members().is_empty());
        assert!(!shorthand.is_legacy());
    }
}

#[test]
fn pause_rest_global_keywords_expand_at_the_whole_property_boundary() {
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
            let expected = match name {
                "pause" => vec![property("pause-before"), property("pause-after")],
                "rest" => vec![property("rest-before"), property("rest-after")],
                _ => vec![property(name)],
            };
            let values = expanded(&source);
            assert_eq!(
                values
                    .items()
                    .iter()
                    .map(|item| item.property())
                    .collect::<Vec<_>>(),
                expected
            );
            for item in values.items() {
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
            }
            let checked = parse_property_value(
                CssPropertyNameRef::Known(property(name)),
                parse_component_values(text).unwrap(),
                CssImportance::Important,
            )
            .unwrap();
            assert_eq!(checked.known().unwrap().global(), Some(keyword));
            assert!(!parse_style_attribute(&format!("{name}:{text} weak")).is_clean());
        }
    }
}

#[test]
fn invalid_pause_rest_values_drop_only_the_declaration_and_remain_grammar_errors() {
    for name in ALL {
        let mut invalid = vec![
            "",
            "-1s",
            "-1e-400s",
            "0",
            "10%",
            "1px",
            "auto",
            "normal",
            "\"strong\"",
            "none, weak",
            "strong()",
            "calc(1s + 1px)",
            "inherit weak",
            "none weak strong",
        ];
        if LONGHANDS.contains(&name) {
            invalid.extend(["none weak", "1s 2s"]);
        } else {
            invalid.extend(["weak -1ms", "-1ms weak"]);
        }
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
                validate_style_attribute(&source).unwrap_err().diagnostics(),
                report.diagnostics()
            );
            assert!(
                parse_property_value(
                    CssPropertyNameRef::Known(property(name)),
                    parse_component_values(value).unwrap(),
                    CssImportance::Normal,
                )
                .is_err(),
                "checked {name}:{value}"
            );
        }
    }
}

#[test]
fn pause_rest_substitution_reentry_is_atomic_ordered_and_retains_replacement_origins() {
    for name in ALL {
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
            panic!("pending authored substitution")
        };
        let valid = if LONGHANDS.contains(&name) {
            "250ms"
        } else {
            "weak 250ms"
        };
        for text in [valid, "inherit"] {
            let replacement = parse_component_values(text).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed replacement")
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
            "-1ms",
            "none weak strong",
            "inherit extra",
            "weak!important",
            "weak;color:red",
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
        assert!(
            handle
                .reenter(parse_component_values(valid).unwrap())
                .is_ok()
        );
    }
}

#[test]
fn pause_rest_authored_case_escapes_comments_and_exact_numbers_survive_bounded_serialization() {
    for (name, canonical, text) in [
        ("PAUSE-BEFORE", "pause-before", "X-STRONG"),
        (r"p\61 use-after", "pause-after", r"w\65 ak"),
        ("pause", "pause", "+3s/**/NONE"),
        ("REST-BEFORE", "rest-before", "9007199254740993ms"),
        ("rest-after", "rest-after", "1e-400s"),
        (r"r\65 st", "rest", "MEDIUM/**/250ms"),
    ] {
        let report = parse_style_attribute(&format!("{name}:{text}!important"));
        assert!(
            report.is_clean(),
            "{name}:{text}: {:?}",
            report.diagnostics()
        );
        let [source] = report.syntax().as_slice() else {
            panic!("one complete escaped/case-insensitive declaration")
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
fn negative_pause_rest_time_diagnostics_preserve_token_location_and_balanced_declaration_span() {
    for name in ALL {
        let prefix = "color:red;/*😀*/";
        let unit = format!("{name}:-1ms;");
        let source = format!("{prefix}{unit}height:1px");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2);
        let [diagnostic] = report.diagnostics() else {
            panic!("one negative time diagnostic")
        };
        assert_ne!(diagnostic.error().code(), CssErrorCode::UnsupportedProperty);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.find("-1ms").unwrap()
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
        assert_eq!(sheet.diagnostics().len(), 1);
        assert_eq!(
            validate_sheet(&sheet_source).unwrap_err().diagnostics(),
            sheet.diagnostics()
        );
    }
}

#[test]
fn pause_rest_support_metadata_identifies_the_pinned_speech_production() {
    for name in ALL {
        let id = format!("official.property.{name}");
        let feature = feature_metadata(&id).unwrap();
        assert_eq!(feature.source().id().as_str(), "S-SPEECH1");
        assert_eq!(
            feature.source().url(),
            Some("https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/")
        );
        assert_eq!(feature.production(), format!("#propdef-{name}"));
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        assert_admitted(name, "none");
    }
}
