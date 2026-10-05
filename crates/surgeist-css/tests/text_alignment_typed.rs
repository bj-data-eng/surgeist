#![forbid(unsafe_code)]
//! Functional current-value evidence for Text 4 (2026-08-14) and UAX29 r47.

use surgeist_css::*;

fn one(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one {name} declaration")
    };
    declaration.clone()
}

fn shorthand(value: &str) -> CssTextAlignValue {
    let source = one("text-align", value);
    let Some(CssKnownPropertyValueRef::TextAlign(value)) = source.known().unwrap().property_value()
    else {
        panic!("text-align current value")
    };
    value.value().clone()
}

fn all(value: &str) -> CssTextAlignAllValue {
    let source = one("text-align-all", value);
    let Some(CssKnownPropertyValueRef::TextAlignAll(value)) =
        source.known().unwrap().property_value()
    else {
        panic!("text-align-all current value")
    };
    value.value().clone()
}

fn last(value: &str) -> CssTextAlignLastValue {
    let source = one("text-align-last", value);
    let Some(CssKnownPropertyValueRef::TextAlignLast(value)) =
        source.known().unwrap().property_value()
    else {
        panic!("text-align-last current value")
    };
    *value.value()
}

fn assigned(value: &str) -> (CssTextAlignAllValue, CssTextAlignLastValue) {
    let source = one("text-align", value);
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("text-align has two terminal targets")
    };
    let [first, second] = values.items() else {
        panic!("all followed by last")
    };
    assert_eq!(first.property(), CssKnownProperty::TextAlignAll);
    assert_eq!(second.property(), CssKnownProperty::TextAlignLast);
    assert!(first.source().same_occurrence(&source));
    assert!(second.source().same_occurrence(&source));
    let Some(CssLonghandValueRef::TextAlignAll(all)) =
        first.ordinary_value().map(CssLonghandValue::view)
    else {
        panic!("exact all payload")
    };
    let Some(CssLonghandValueRef::TextAlignLast(last)) =
        second.ordinary_value().map(CssLonghandValue::view)
    else {
        panic!("exact last payload")
    };
    (all.clone(), *last)
}

#[test]
fn seven_shared_keywords_and_two_special_shorthand_effects_are_exact() {
    for (spelling, keyword) in [
        ("start", CssTextAlign::Start),
        ("end", CssTextAlign::End),
        ("left", CssTextAlign::Left),
        ("right", CssTextAlign::Right),
        ("center", CssTextAlign::Center),
        ("justify", CssTextAlign::Justify),
        ("match-parent", CssTextAlign::MatchParent),
    ] {
        let expected_all = CssTextAlignAllValue::Keyword(keyword);
        assert_eq!(all(spelling), expected_all);
        assert_eq!(
            shorthand(spelling),
            CssTextAlignValue::Alignment(expected_all.clone())
        );
        assert_eq!(last(spelling), CssTextAlignLastValue::Keyword(keyword));
        assert_eq!(
            assigned(spelling),
            (
                expected_all,
                if spelling == "match-parent" {
                    CssTextAlignLastValue::Keyword(CssTextAlign::MatchParent)
                } else {
                    CssTextAlignLastValue::Auto
                }
            )
        );
    }
    assert_eq!(last("auto"), CssTextAlignLastValue::Auto);
    assert_eq!(shorthand("justify-all"), CssTextAlignValue::JustifyAll);
    assert_eq!(
        assigned("justify-all"),
        (
            CssTextAlignAllValue::Keyword(CssTextAlign::Justify),
            CssTextAlignLastValue::Keyword(CssTextAlign::Justify)
        )
    );
    assert_eq!(
        CssTextAlignAllValue::initial(),
        CssTextAlignAllValue::Keyword(CssTextAlign::Start)
    );
    assert_eq!(
        CssTextAlignLastValue::initial(),
        CssTextAlignLastValue::Auto
    );
}

#[test]
fn character_fallback_omission_is_authored_and_expansion_resets_last_to_auto() {
    let omitted = CssCharacterAlignment::try_new(".", None).unwrap();
    let explicit = CssCharacterAlignment::try_new(".", Some(CssTextAlignPosition::Right)).unwrap();
    assert_ne!(omitted, explicit);
    assert_eq!(omitted.authored_fallback(), None);
    assert_eq!(omitted.effective_fallback(), CssTextAlignPosition::Right);
    assert_eq!(
        explicit.authored_fallback(),
        Some(CssTextAlignPosition::Right)
    );
    assert_eq!(explicit.effective_fallback(), CssTextAlignPosition::Right);
    assert_eq!(omitted.decoded(), ".");
    assert!(matches!(
        omitted.component().origin(),
        CssValueOrigin::Programmatic
    ));
    assert_eq!(
        omitted.clone(),
        CssCharacterAlignment::try_new(".", None).unwrap()
    );

    for (source, fallback) in [
        ("\".\"", None),
        ("\".\" right", Some(CssTextAlignPosition::Right)),
        ("right \".\"", Some(CssTextAlignPosition::Right)),
        ("\".\" left", Some(CssTextAlignPosition::Left)),
        ("left \".\"", Some(CssTextAlignPosition::Left)),
    ] {
        let parsed = shorthand(source);
        let CssTextAlignValue::Alignment(CssTextAlignAllValue::Character(character)) = &parsed
        else {
            panic!("character shorthand")
        };
        assert_eq!(character.decoded(), ".");
        assert_eq!(character.authored_fallback(), fallback);
        assert_eq!(
            character.effective_fallback(),
            fallback.unwrap_or(CssTextAlignPosition::Right)
        );
        assert!(matches!(
            character.component().origin(),
            CssValueOrigin::Parsed(_)
        ));
        assert_eq!(
            parsed.serialize_specified().unwrap(),
            if let Some(fallback) = fallback {
                format!(
                    "\".\" {}",
                    match fallback {
                        CssTextAlignPosition::Start => "start",
                        CssTextAlignPosition::End => "end",
                        CssTextAlignPosition::Left => "left",
                        CssTextAlignPosition::Right => "right",
                        CssTextAlignPosition::Center => "center",
                        _ => panic!("selected positional fallback"),
                    }
                )
            } else {
                "\".\"".to_owned()
            }
        );
        let (all, last) = assigned(source);
        let CssTextAlignAllValue::Character(assigned_character) = all else {
            panic!("complete character assignment")
        };
        assert_eq!(assigned_character.decoded(), ".");
        assert_eq!(assigned_character.authored_fallback(), fallback);
        assert_eq!(last, CssTextAlignLastValue::Auto);
    }
    assert_eq!(
        all("\".\" right").serialize_specified().unwrap(),
        "\".\" right"
    );
}

#[test]
fn decoded_unicode_clusters_and_source_graphs_share_checked_construction() {
    for text in ["é", "한", "👩\u{200d}💻", "🇺🇸"] {
        let value = CssCharacterAlignment::try_new(text, None).unwrap();
        assert_eq!(value.decoded(), text);
        assert!(matches!(
            value.component().origin(),
            CssValueOrigin::Programmatic
        ));
        let css = format!("\"{text}\"");
        let CssTextAlignAllValue::Character(parsed) = all(&css) else {
            panic!("one decoded cluster")
        };
        assert_eq!(parsed.decoded(), text);
        assert_eq!(parsed.serialize_specified().unwrap(), css);
        assert!(matches!(
            parsed.component().origin(),
            CssValueOrigin::Parsed(_)
        ));
    }
    let escaped = parse_component_values("\"\\65\\301 \"").unwrap();
    let original = escaped.items()[0].clone();
    let value = CssCharacterAlignment::try_from_component(original.clone(), None).unwrap();
    assert_eq!(value.decoded(), "é");
    assert_eq!(value.component(), &original);
    assert!(matches!(
        value.component().origin(),
        CssValueOrigin::Parsed(_)
    ));
    let CssTextAlignAllValue::Character(parsed) = all("\"\\65\\301 \"") else {
        panic!("escaped source character")
    };
    assert_eq!(parsed.decoded(), "é");
    assert_eq!(parsed.serialize_specified().unwrap(), "\"é\"");
}

#[test]
fn invalid_decoded_clusters_components_and_literal_bad_newlines_are_rejected() {
    for text in ["", "ab", "🇺🇸🇨"] {
        assert_eq!(
            CssCharacterAlignment::try_new(text, None)
                .unwrap_err()
                .kind(),
            CssCharacterAlignmentErrorKind::InvalidCharacter
        );
    }
    let error = CssCharacterAlignment::try_new("\0", None).unwrap_err();
    assert_eq!(
        error.kind(),
        CssCharacterAlignmentErrorKind::Component(CssComponentValueErrorKind::InvalidString)
    );
    assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
    let identifier = CssComponentValue::try_ident("red").unwrap();
    assert_eq!(
        CssCharacterAlignment::try_from_component(identifier, None)
            .unwrap_err()
            .kind(),
        CssCharacterAlignmentErrorKind::Component(CssComponentValueErrorKind::InvalidString)
    );
    for name in ["text-align", "text-align-all"] {
        let report = parse_style_attribute(&format!("{name}:\"a\nb\""));
        assert!(!report.is_clean(), "literal newline in string is invalid");
        let report = parse_style_attribute(&format!("{name}:\".\\\n\""));
        assert!(
            report.is_clean(),
            "escaped newline preserves one decoded character"
        );
    }
}

#[test]
fn parsed_alignment_values_keep_keywords_and_character_domains() {
    for (text, expected) in [
        (
            "start",
            CssTextAlignValue::Alignment(CssTextAlignAllValue::Keyword(CssTextAlign::Start)),
        ),
        (
            "match-parent",
            CssTextAlignValue::Alignment(CssTextAlignAllValue::Keyword(CssTextAlign::MatchParent)),
        ),
        ("justify-all", CssTextAlignValue::JustifyAll),
    ] {
        let source = one("text-align", text);
        let CssKnownPropertyValueRef::TextAlign(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("alignment")
        };
        assert_eq!(value.value(), &expected);
    }
    let source = one("text-align", "\".\"");
    let CssKnownPropertyValueRef::TextAlign(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("alignment")
    };
    assert!(
        matches!(value.value(), CssTextAlignValue::Alignment(CssTextAlignAllValue::Character(character)) if character.decoded() == "." && character.authored_fallback().is_none())
    );
    for (text, expected) in [
        ("auto", CssTextAlignLastValue::Auto),
        (
            "justify",
            CssTextAlignLastValue::Keyword(CssTextAlign::Justify),
        ),
        (
            "match-parent",
            CssTextAlignLastValue::Keyword(CssTextAlign::MatchParent),
        ),
    ] {
        let source = one("text-align-last", text);
        let CssKnownPropertyValueRef::TextAlignLast(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("last alignment")
        };
        assert_eq!(value.value(), &expected);
    }
}

#[test]
fn specified_serialization_is_canonical_and_atomically_bounded() {
    for (decoded, expected) in [
        ("\"", r#""\"""#),
        ("\\", r#""\\""#),
        ("\u{0001}", r#""\1 ""#),
    ] {
        assert_eq!(
            CssCharacterAlignment::try_new(decoded, None)
                .unwrap()
                .serialize_specified()
                .unwrap(),
            expected
        );
    }
    for (value, expected, nodes) in [
        (shorthand("START"), "start", 1),
        (shorthand("justify-all"), "justify-all", 1),
        (shorthand("\".\""), "\".\"", 1),
        (shorthand("left \".\""), "\".\" left", 2),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    nodes,
                    nodes,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(nodes - 1, nodes, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes - 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes, expected.len() - 1),
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
        }
    }
    assert_eq!(
        last("MATCH-PARENT").serialize_specified().unwrap(),
        "match-parent"
    );
    assert_eq!(all("CENTER").serialize_specified().unwrap(), "center");

    let one_long_cluster = format!("e{}", "\u{301}".repeat(4096));
    let parsed = parse_component_values(&format!("\"{one_long_cluster}\"")).unwrap();
    let value = CssCharacterAlignment::try_from_component(parsed.items()[0].clone(), None).unwrap();
    assert_eq!(value.decoded(), one_long_cluster);
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 8))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(
        value.serialize_specified().unwrap(),
        format!("\"{one_long_cluster}\"")
    );
}

#[test]
fn shared_text4_source_matches_complete_text_wrap_authored_grammar() {
    let metadata = property_support_metadata("text-wrap").expect("existing Text 4 property");
    assert_eq!(metadata.feature().status(), CssSupportStatus::Complete);
    assert_eq!(metadata.feature().source().id().as_str(), "X-TEXT4");
    assert_eq!(
        metadata.feature().source().url(),
        Some("https://www.w3.org/TR/2026/WD-css-text-4-20260814/")
    );
    let report = parse_style_attribute("text-wrap:balance");
    assert!(report.is_clean());
    let report = parse_style_attribute("text-wrap:auto");
    assert!(report.is_clean());
    let report = parse_style_attribute("text-wrap:wrap nowrap");
    assert!(!report.is_clean());
}
