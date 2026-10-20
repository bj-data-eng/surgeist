use crate::html::support::{input_element as input, input_state_forbids_attribute as forbidden};
use surgeist_html::*;

fn raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Colorspace.qualified_name(), value)
}

#[test]
fn exact_keywords_and_ascii_case_select_states_without_claiming_color_context() {
    for (spelling, canonical, state) in [
        (
            "limited-srgb",
            "limited-srgb",
            HtmlInputColorSpace::LimitedSrgb,
        ),
        (
            "LIMITED-SRGB",
            "limited-srgb",
            HtmlInputColorSpace::LimitedSrgb,
        ),
        (
            "LiMiTeD-sRgB",
            "limited-srgb",
            HtmlInputColorSpace::LimitedSrgb,
        ),
        ("display-p3", "display-p3", HtmlInputColorSpace::DisplayP3),
        ("DISPLAY-P3", "display-p3", HtmlInputColorSpace::DisplayP3),
        ("dIsPlAy-P3", "display-p3", HtmlInputColorSpace::DisplayP3),
    ] {
        let attribute = raw(spelling);
        let report = HtmlValueInterpreter::new(attribute.value().len())
            .input_colorspace(&input(), Some(&attribute))
            .unwrap();
        assert_eq!(report.raw_attribute(), Some(&attribute));
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(
            report.recognition(),
            HtmlAttributeRecognition::Keyword {
                matched: canonical,
                canonical
            }
        );
        assert_eq!(report.intrinsic(), Some(&state));
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
        assert_eq!(report.input_state(), None);
    }
}

#[test]
fn absence_and_empty_share_limited_srgb_but_not_author_or_presence_states() {
    let interpreter = HtmlValueInterpreter::new(0);
    let missing = interpreter
        .input_colorspace(&input(), None::<&surgeist_html::Attribute>)
        .unwrap();
    assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(missing.recognition(), HtmlAttributeRecognition::Missing);
    assert_eq!(missing.author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(
        missing.context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(missing.intrinsic(), Some(&HtmlInputColorSpace::LimitedSrgb));
    assert_eq!(missing.raw(), None);
    let attribute = raw("");
    let empty = interpreter
        .input_colorspace(&input(), Some(&attribute))
        .unwrap();
    assert_eq!(empty.observation(), HtmlAttributeObservation::PresentEmpty);
    assert_eq!(empty.recognition(), HtmlAttributeRecognition::Empty);
    assert_eq!(
        empty.author(),
        HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::UnrecognizedKeyword)
    );
    assert_eq!(
        empty.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(empty.intrinsic(), Some(&HtmlInputColorSpace::LimitedSrgb));
    assert_eq!(empty.raw_attribute(), Some(&attribute));
}

#[test]
fn invalid_keywords_and_all_padding_select_fallback_without_lexical_salvage() {
    for spelling in [
        "srgb",
        "p3",
        "display-p3 limited-srgb",
        "limited-srgb,display-p3",
        "display-p3x",
        "display-p3\0",
        " display-p3",
        "display-p3 ",
        "\tdisplay-p3",
        "display-p3\n",
        "\rdisplay-p3",
        "display-p3\u{c}",
        "display‐p3",
        "diſplay-p3",
    ] {
        let attribute = raw(spelling);
        let interpreter = HtmlValueInterpreter::new(attribute.value().len());
        for state in [None, Some(HtmlInputType::Color), Some(HtmlInputType::Text)] {
            let report = match state {
                None => interpreter.input_colorspace(&input(), Some(&attribute)),
                Some(state) => {
                    interpreter.input_colorspace_for_input(&input(), Some(&attribute), state)
                }
            }
            .unwrap();
            assert_eq!(report.raw_attribute(), Some(&attribute));
            assert_eq!(report.recognition(), HtmlAttributeRecognition::Unrecognized);
            assert_eq!(
                report.author(),
                HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::UnrecognizedKeyword
                )
            );
            assert_eq!(report.intrinsic(), Some(&HtmlInputColorSpace::LimitedSrgb));
            assert_eq!(
                report.context_author(),
                match state {
                    None => HtmlAttributeContextAuthorState::Unchecked,
                    Some(HtmlInputType::Color) => HtmlAttributeContextAuthorState::Conforming,
                    Some(_) => forbidden(),
                }
            );
        }
    }
}

#[test]
fn isolated_surrogates_and_astral_suffixes_remain_exact_invalid_units() {
    for units in [
        vec![0xD800],
        vec![0xDC00],
        vec![100, 105, 115, 112, 108, 97, 121, 45, 112, 51, 0xD800],
        vec![
            100, 105, 115, 112, 108, 97, 121, 45, 112, 51, 0xD83D, 0xDE00,
        ],
    ] {
        let attribute = raw(DocumentString::from_utf16(units.clone()));
        let report = HtmlValueInterpreter::new(units.len())
            .input_colorspace_for_input(&input(), Some(&attribute), HtmlInputType::Color)
            .unwrap();
        assert_eq!(report.raw().unwrap().code_units(), units);
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Unrecognized);
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword
            )
        );
        assert_eq!(report.intrinsic(), Some(&HtmlInputColorSpace::LimitedSrgb));
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Conforming
        );
    }
}

#[test]
fn color_only_presence_is_independent_of_spelling_and_all_other_input_states() {
    // Literal list of all source-defined non-Color states, not discovered from a provider.
    for state in [
        HtmlInputType::Hidden,
        HtmlInputType::Text,
        HtmlInputType::Search,
        HtmlInputType::Telephone,
        HtmlInputType::Url,
        HtmlInputType::Email,
        HtmlInputType::Password,
        HtmlInputType::Date,
        HtmlInputType::Month,
        HtmlInputType::Week,
        HtmlInputType::Time,
        HtmlInputType::LocalDateTime,
        HtmlInputType::Number,
        HtmlInputType::Range,
        HtmlInputType::Checkbox,
        HtmlInputType::Radio,
        HtmlInputType::File,
        HtmlInputType::Submit,
        HtmlInputType::Image,
        HtmlInputType::Reset,
        HtmlInputType::Button,
    ] {
        let interpreter = HtmlValueInterpreter::new(64);
        let absent = interpreter
            .input_colorspace_for_input(&input(), None::<&surgeist_html::Attribute>, state)
            .unwrap();
        assert_eq!(absent.input_state(), Some(state));
        assert_eq!(absent.author(), HtmlAttributeAuthorState::Missing);
        assert_eq!(
            absent.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(absent.intrinsic(), Some(&HtmlInputColorSpace::LimitedSrgb));
        for (spelling, expected) in [
            ("display-p3", HtmlInputColorSpace::DisplayP3),
            ("limited-srgb", HtmlInputColorSpace::LimitedSrgb),
            ("bad", HtmlInputColorSpace::LimitedSrgb),
            ("", HtmlInputColorSpace::LimitedSrgb),
        ] {
            let attribute = raw(spelling);
            let report = interpreter
                .input_colorspace_for_input(&input(), Some(&attribute), state)
                .unwrap();
            assert_eq!(report.input_state(), Some(state));
            assert_eq!(
                report.applicability(),
                HtmlAttributeApplicability::Applicable
            );
            assert_eq!(report.intrinsic(), Some(&expected));
            assert_eq!(report.context_author(), forbidden());
            assert_eq!(
                report.author(),
                if spelling == "bad" || spelling.is_empty() {
                    HtmlAttributeAuthorState::Nonconforming(
                        HtmlAttributeConformanceError::UnrecognizedKeyword,
                    )
                } else {
                    HtmlAttributeAuthorState::Conforming
                }
            );
        }
    }
    let attribute = raw("");
    let color = HtmlValueInterpreter::new(0)
        .input_colorspace_for_input(&input(), Some(&attribute), HtmlInputType::Color)
        .unwrap();
    assert_eq!(
        color.context_author(),
        HtmlAttributeContextAuthorState::Conforming
    );
    assert_eq!(
        color.author(),
        HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::UnrecognizedKeyword)
    );
}

#[test]
fn native_canonical_keywords_emit_real_attributes_and_roundtrip_in_color_context() {
    for (state, spelling) in [
        (HtmlInputColorSpace::LimitedSrgb, "limited-srgb"),
        (HtmlInputColorSpace::DisplayP3, "display-p3"),
    ] {
        assert_eq!(state.canonical_keyword(), spelling);
        let attribute = state.into_attribute();
        assert_eq!(
            attribute.name(),
            &KnownHtmlAttribute::Colorspace.qualified_name()
        );
        assert_eq!(attribute.name().namespace(), &Namespace::none());
        assert_eq!(
            attribute.value().code_units(),
            spelling.encode_utf16().collect::<Vec<_>>()
        );
        let report = HtmlValueInterpreter::new(12)
            .input_colorspace_for_input(&input(), Some(&attribute), HtmlInputType::Color)
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&state));
        assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
        assert_eq!(
            report.context_author(),
            HtmlAttributeContextAuthorState::Conforming
        );
    }
}

#[test]
fn actual_input_type_reports_supply_color_and_source_defined_text_defaults() {
    let interpreter = HtmlValueInterpreter::new(64);
    let attribute = raw("display-p3");
    for (spelling, state, context) in [
        (
            Some("cOlOr"),
            HtmlInputType::Color,
            HtmlAttributeContextAuthorState::Conforming,
        ),
        (None, HtmlInputType::Text, forbidden()),
        (Some("unknown"), HtmlInputType::Text, forbidden()),
        (Some(""), HtmlInputType::Text, forbidden()),
    ] {
        let type_attribute =
            spelling.map(|s| Attribute::new(KnownHtmlAttribute::Type.qualified_name(), s));
        let type_report = interpreter
            .input_type(&input(), type_attribute.as_ref())
            .unwrap();
        assert_eq!(type_report.intrinsic(), Some(&state));
        let report = interpreter
            .input_colorspace_for_input(
                &input(),
                Some(&attribute),
                *type_report.intrinsic().unwrap(),
            )
            .unwrap();
        assert_eq!(report.input_state(), Some(state));
        assert_eq!(report.context_author(), context);
        assert_eq!(report.intrinsic(), Some(&HtmlInputColorSpace::DisplayP3));
    }
}

#[test]
fn source_metadata_and_prefixed_expanded_names_preserve_independent_context_evidence() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "input");
    let attribute = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("a".into()), "colorspace"),
        "DISPLAY-P3",
    );
    let interpreter = HtmlValueInterpreter::new(64);
    let content = interpreter
        .input_colorspace(&element, Some(&attribute))
        .unwrap();
    assert_eq!(content.element(), &element);
    assert_eq!(content.raw_attribute(), Some(&attribute));
    assert_eq!(content.attribute(), KnownHtmlAttribute::Colorspace);
    assert_eq!(content.inventory_rows(), &[25]);
    assert_eq!(
        content.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-input-colorspace"
    );
    assert_eq!(
        content.context_source_clause(),
        Some(
            "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-input-element"
        )
    );
    assert_eq!(content.vocabulary_source_clause(), None);
    for (state, anchor) in [
        (HtmlInputType::Color, "color-state-(type=color)"),
        (
            HtmlInputType::Text,
            "text-(type=text)-state-and-search-state-(type=search)",
        ),
        (HtmlInputType::Number, "number-state-(type=number)"),
    ] {
        for raw in [None, Some(&attribute)] {
            let report = interpreter
                .input_colorspace_for_input(&element, raw, state)
                .unwrap();
            assert_eq!(report.context_source_clause(), Some(format!("https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#{anchor}").as_str()));
        }
    }
    let unrelated = interpreter
        .input_type(&element, None::<&surgeist_html::Attribute>)
        .unwrap();
    assert_eq!(unrelated.context_source_clause(), None);
}

#[test]
fn wrong_roles_keep_raw_and_supplied_state_but_no_intrinsic_or_conformance_claim() {
    let attribute = raw("display-p3");
    for (element, attribute, expected) in [
        (
            KnownHtmlElement::Div.qualified_name(),
            attribute.clone(),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "INPUT"),
            attribute.clone(),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::svg(), None, "input"),
            attribute.clone(),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            input(),
            Attribute::new(KnownHtmlAttribute::Id.qualified_name(), "display-p3"),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            input(),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "COLORSPACE"),
                "display-p3",
            ),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            input(),
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "colorspace"),
                "display-p3",
            ),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
    ] {
        let interpreter = HtmlValueInterpreter::new(64);
        for state in [None, Some(HtmlInputType::Color), Some(HtmlInputType::Text)] {
            let report = match state {
                None => interpreter.input_colorspace(&element, Some(&attribute)),
                Some(state) => {
                    interpreter.input_colorspace_for_input(&element, Some(&attribute), state)
                }
            }
            .unwrap();
            assert_eq!(report.element(), &element);
            assert_eq!(report.raw_attribute(), Some(&attribute));
            assert_eq!(report.input_state(), state);
            assert_eq!(report.applicability(), expected);
            assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
            assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::Unapplied
            );
            assert_eq!(report.intrinsic(), None);
        }
    }
}

#[test]
fn whole_original_utf16_cap_precedes_wrong_role_and_context_observation() {
    let attribute = raw("display-p3");
    let invalid = raw(format!("display-p3{}", " ".repeat(2048)));
    for attribute in [&attribute, &invalid] {
        let interpreter = HtmlValueInterpreter::new(attribute.value().len() - 1);
        for element in [input(), KnownHtmlElement::Div.qualified_name()] {
            let content_error = interpreter
                .input_colorspace(&element, Some(attribute))
                .unwrap_err();
            let state_error = interpreter
                .input_colorspace_for_input(&element, Some(attribute), HtmlInputType::Text)
                .unwrap_err();
            assert_eq!(content_error, state_error);
            assert_eq!(content_error.capacity(), attribute.value().len() - 1);
            assert_eq!(content_error.required(), attribute.value().len());
        }
    }
    let equal = HtmlValueInterpreter::new(attribute.value().len())
        .input_colorspace(&input(), Some(&attribute))
        .unwrap();
    assert_eq!(equal.intrinsic(), Some(&HtmlInputColorSpace::DisplayP3));
    let invalid_report = HtmlValueInterpreter::new(invalid.value().len())
        .input_colorspace(&input(), Some(&invalid))
        .unwrap();
    assert_eq!(
        invalid_report.recognition(),
        HtmlAttributeRecognition::Unrecognized
    );
    assert_eq!(
        invalid_report.intrinsic(),
        Some(&HtmlInputColorSpace::LimitedSrgb)
    );
}
