use surgeist_html::*;

const BASE: &str =
    "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#";

fn element(name: KnownHtmlElement) -> QualifiedName {
    name.qualified_name()
}
fn raw(name: KnownHtmlAttribute, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(name.qualified_name(), value)
}
fn input_forbidden() -> HtmlAttributeContextAuthorState {
    HtmlAttributeContextAuthorState::Nonconforming(
        HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
    )
}
fn source_forbidden() -> HtmlAttributeContextAuthorState {
    HtmlAttributeContextAuthorState::Nonconforming(
        HtmlAttributeContextConformanceError::SourceDimensionsRequirePictureParent,
    )
}
fn malformed() -> HtmlImageAttributeAuthorState {
    HtmlImageAttributeAuthorState::Nonconforming(HtmlImageAttributeConformanceError::CommonValue(
        HtmlValueFailure::Syntax(HtmlValueSyntaxError::InvalidNumericSyntax),
    ))
}

#[test]
fn actual_image_requires_both_strings_and_retains_missing_versus_empty() {
    let interpreter = HtmlValueInterpreter::new(0);
    let missing = interpreter
        .input_image_attributes_for_state(
            &element(KnownHtmlElement::Input),
            HtmlInputType::Image,
            None::<&surgeist_html::Attribute>,
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    let alt = raw(KnownHtmlAttribute::Alt, "");
    let src = raw(KnownHtmlAttribute::Src, "");
    let empty = interpreter
        .input_image_attributes_for_state(
            &element(KnownHtmlElement::Input),
            HtmlInputType::Image,
            Some(&alt),
            Some(&src),
        )
        .unwrap();
    for slot in [missing.alt(), missing.src()] {
        assert_eq!(
            slot.attribute().observation(),
            HtmlAttributeObservation::Missing
        );
        assert_eq!(
            slot.author(),
            HtmlImageAttributeAuthorState::Nonconforming(
                HtmlImageAttributeConformanceError::RequiredAttributeMissing
            )
        );
        assert_eq!(slot.attribute().input_state(), Some(HtmlInputType::Image));
        assert_eq!(
            slot.attribute().context_author(),
            HtmlAttributeContextAuthorState::Conforming
        );
    }
    assert_eq!(
        missing.alt().label_adequacy(),
        HtmlImageExternalConformance::NotRequired
    );
    assert_eq!(
        missing.src().url_conformance(),
        HtmlImageExternalConformance::NotRequired
    );
    assert_eq!(
        empty.alt().attribute().observation(),
        HtmlAttributeObservation::PresentEmpty
    );
    assert_eq!(
        empty.alt().author(),
        HtmlImageAttributeAuthorState::Nonconforming(HtmlImageAttributeConformanceError::EmptyAlt)
    );
    assert_eq!(
        empty.src().author(),
        HtmlImageAttributeAuthorState::Nonconforming(
            HtmlImageAttributeConformanceError::EmptySource
        )
    );
    assert_eq!(empty.src().source_reference(), Some(&[][..]));
    assert_eq!(empty.src().attribute().raw_attribute(), Some(&src));
}

#[test]
fn nonempty_alt_is_not_trimmed_and_source_strips_only_ascii_edges() {
    for alt_units in [
        vec![32],
        vec![0],
        vec![0xD800],
        vec![0xDC00],
        vec![0xD83D, 0xDE00],
        vec![9, 10, 13, 12, 32],
    ] {
        let alt = raw(
            KnownHtmlAttribute::Alt,
            DocumentString::from_utf16(alt_units.clone()),
        );
        let src = raw(KnownHtmlAttribute::Src, " \t../button.png\r\n\u{c}");
        let report = HtmlValueInterpreter::new(100)
            .input_image_attributes_for_state(
                &element(KnownHtmlElement::Input),
                HtmlInputType::Image,
                Some(&alt),
                Some(&src),
            )
            .unwrap();
        assert_eq!(
            report.alt().author(),
            HtmlImageAttributeAuthorState::Conforming
        );
        assert_eq!(
            report.alt().attribute().raw().unwrap().code_units(),
            alt_units
        );
        assert_eq!(
            report.src().author(),
            HtmlImageAttributeAuthorState::Conforming
        );
        assert_eq!(
            report.src().source_reference().unwrap(),
            "../button.png".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(report.src().attribute().raw_attribute(), Some(&src));
        assert_eq!(
            report.alt().label_adequacy(),
            HtmlImageExternalConformance::Unchecked
        );
        assert_eq!(
            report.src().url_conformance(),
            HtmlImageExternalConformance::Unchecked
        );
        assert_eq!(
            report.src().resource_conformance(),
            HtmlImageExternalConformance::Unchecked
        );
        assert_eq!(
            report.alt().url_conformance(),
            HtmlImageExternalConformance::Unapplied
        );
    }
}

#[test]
fn ascii_only_whitespace_source_fails_but_non_ascii_and_invalid_url_units_are_unchecked() {
    let alt = raw(KnownHtmlAttribute::Alt, "submit");
    for (units, expected) in [
        (vec![9, 10, 12, 13, 32], false),
        (vec![0xA0], true),
        (vec![0x0B], true),
        (vec![0], true),
        (vec![0xD800], true),
        (vec![0xDC00], true),
        (vec![32, 0xD800, 32], true),
    ] {
        let src = raw(
            KnownHtmlAttribute::Src,
            DocumentString::from_utf16(units.clone()),
        );
        let report = HtmlValueInterpreter::new(100)
            .input_image_attributes_for_state(
                &element(KnownHtmlElement::Input),
                HtmlInputType::Image,
                Some(&alt),
                Some(&src),
            )
            .unwrap();
        assert_eq!(
            report.src().author(),
            if expected {
                HtmlImageAttributeAuthorState::Conforming
            } else {
                HtmlImageAttributeAuthorState::Nonconforming(
                    HtmlImageAttributeConformanceError::EmptySource,
                )
            }
        );
        assert_eq!(report.src().attribute().raw().unwrap().code_units(), units);
        assert_eq!(
            report.src().url_conformance(),
            HtmlImageExternalConformance::Unchecked
        );
    }
}

#[test]
fn content_only_input_and_source_keep_unknown_context_distinct_from_absence() {
    let interpreter = HtmlValueInterpreter::new(20);
    let alt = raw(KnownHtmlAttribute::Alt, "");
    let src = raw(KnownHtmlAttribute::Src, "x");
    for (a, s) in [(None, None), (Some(&alt), Some(&src))] {
        let report = interpreter
            .input_image_attributes(&element(KnownHtmlElement::Input), a, s)
            .unwrap();
        for slot in [report.alt(), report.src()] {
            assert_eq!(slot.author(), HtmlImageAttributeAuthorState::Unchecked);
            assert_eq!(
                slot.attribute().context_author(),
                HtmlAttributeContextAuthorState::Unchecked
            );
            assert_eq!(slot.attribute().input_state(), None);
        }
        assert_eq!(report.src().source_reference(), None);
    }
    let width = raw(KnownHtmlAttribute::Width, "+2tail");
    for name in [KnownHtmlElement::Input, KnownHtmlElement::Source] {
        let report = interpreter
            .embedded_dimension_attributes(
                &element(name),
                Some(&width),
                None::<&surgeist_html::Attribute>,
            )
            .unwrap();
        for slot in [report.width(), report.height()] {
            assert_eq!(slot.author(), HtmlImageAttributeAuthorState::Unchecked);
            assert_eq!(
                slot.attribute().context_author(),
                HtmlAttributeContextAuthorState::Unchecked
            );
            assert!(slot.common().is_none());
            assert!(slot.recovered().is_none());
        }
    }
}

#[test]
fn normative_twenty_two_input_states_select_only_image_and_forbid_other_presence() {
    // Literal normative §4.10.5.1 bookkeeping, not its nonnormative summary.
    let states = [
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
        HtmlInputType::Color,
        HtmlInputType::Checkbox,
        HtmlInputType::Radio,
        HtmlInputType::File,
        HtmlInputType::Submit,
        HtmlInputType::Image,
        HtmlInputType::Reset,
        HtmlInputType::Button,
    ];
    let interpreter = HtmlValueInterpreter::new(20);
    let alt = raw(KnownHtmlAttribute::Alt, " ");
    let src = raw(KnownHtmlAttribute::Src, "x");
    let width = raw(KnownHtmlAttribute::Width, "0");
    let height = raw(KnownHtmlAttribute::Height, "002");
    for state in states {
        let strings = interpreter
            .input_image_attributes_for_state(
                &element(KnownHtmlElement::Input),
                state,
                Some(&alt),
                Some(&src),
            )
            .unwrap();
        let dimensions = interpreter
            .embedded_dimension_attributes_with_context(
                &element(KnownHtmlElement::Input),
                Some(&width),
                Some(&height),
                &HtmlEmbeddedDimensionContext::InputState(state),
            )
            .unwrap();
        let author = if state == HtmlInputType::Image {
            HtmlImageAttributeAuthorState::Conforming
        } else {
            HtmlImageAttributeAuthorState::Unapplied
        };
        let context = if state == HtmlInputType::Image {
            HtmlAttributeContextAuthorState::Conforming
        } else {
            input_forbidden()
        };
        for slot in [strings.alt(), strings.src()] {
            assert_eq!(slot.author(), author, "{state:?}");
            assert_eq!(slot.attribute().context_author(), context, "{state:?}");
            assert_eq!(slot.attribute().input_state(), Some(state));
        }
        for slot in [dimensions.width(), dimensions.height()] {
            assert_eq!(slot.author(), author, "{state:?}");
            assert_eq!(slot.attribute().context_author(), context, "{state:?}");
            assert_eq!(slot.common().is_some(), state == HtmlInputType::Image);
        }
        if state != HtmlInputType::Image {
            let absent = interpreter
                .input_image_attributes_for_state(
                    &element(KnownHtmlElement::Input),
                    state,
                    None::<&surgeist_html::Attribute>,
                    None::<&surgeist_html::Attribute>,
                )
                .unwrap();
            assert_eq!(
                absent.alt().attribute().context_author(),
                HtmlAttributeContextAuthorState::NotRequired
            );
            assert_eq!(
                absent.alt().author(),
                HtmlImageAttributeAuthorState::Unapplied
            );
        }
    }
}

#[test]
fn unconditional_embedded_roles_have_exact_zero_and_missing_without_canvas_defaults() {
    let interpreter = HtmlValueInterpreter::new(20);
    let width = raw(KnownHtmlAttribute::Width, "0");
    for name in [
        KnownHtmlElement::Img,
        KnownHtmlElement::Iframe,
        KnownHtmlElement::Embed,
        KnownHtmlElement::Object,
        KnownHtmlElement::Video,
    ] {
        let report = interpreter
            .embedded_dimension_attributes(
                &element(name),
                Some(&width),
                None::<&surgeist_html::Attribute>,
            )
            .unwrap();
        assert_eq!(
            report.width().author(),
            HtmlImageAttributeAuthorState::Conforming
        );
        assert_eq!(report.width().recovered().unwrap().decimal_digits(), "0");
        assert_eq!(
            report.height().author(),
            HtmlImageAttributeAuthorState::Missing
        );
        assert!(matches!(
            report.height().common().unwrap().author(),
            HtmlValueAuthorState::Missing
        ));
        assert!(matches!(
            report.height().common().unwrap().recovery(),
            HtmlValueRecoveryState::Missing
        ));
        assert!(report.height().recovered().is_none());
        assert_eq!(
            report.width().attribute().context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
    }
    let canvas = interpreter
        .embedded_dimension_attributes(
            &element(KnownHtmlElement::Canvas),
            Some(&width),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(
        canvas.width().attribute().applicability(),
        HtmlAttributeApplicability::DifferentElement
    );
    assert!(canvas.width().common().is_none());
    assert_eq!(
        interpreter
            .canvas_width(
                &element(KnownHtmlElement::Canvas),
                None::<&surgeist_html::Attribute>
            )
            .unwrap()
            .intrinsic()
            .unwrap()
            .decimal_digits(),
        "300"
    );
    assert_eq!(
        interpreter
            .canvas_height(
                &element(KnownHtmlElement::Canvas),
                None::<&surgeist_html::Attribute>
            )
            .unwrap()
            .intrinsic()
            .unwrap()
            .decimal_digits(),
        "150"
    );
}

#[test]
fn dimension_prefix_recovery_does_not_repair_author_syntax_or_default_zero() {
    for (text, valid, recovered) in [
        ("0002", true, Some("2")),
        ("+2tail", false, Some("2")),
        ("-0", false, Some("0")),
        ("-1", false, None),
        ("", false, None),
        (" 5.9", false, Some("5")),
        ("1\0x", false, Some("1")),
        ("\u{a0}2", false, None),
    ] {
        let width = raw(KnownHtmlAttribute::Width, text);
        let report = HtmlValueInterpreter::new(100)
            .embedded_dimension_attributes(
                &element(KnownHtmlElement::Img),
                Some(&width),
                None::<&surgeist_html::Attribute>,
            )
            .unwrap();
        assert_eq!(
            report.width().author(),
            if valid {
                HtmlImageAttributeAuthorState::Conforming
            } else {
                malformed()
            },
            "{text:?}"
        );
        assert_eq!(
            report
                .width()
                .recovered()
                .map(HtmlNonNegativeInteger::decimal_digits),
            recovered,
            "{text:?}"
        );
        assert_eq!(report.width().attribute().raw_attribute(), Some(&width));
    }
}

#[test]
fn arbitrary_dimension_magnitudes_and_surrogate_prefixes_remain_exact() {
    let huge = "3402823669209384634633746074317682114560000000000000000000";
    let width = raw(KnownHtmlAttribute::Width, huge);
    let height = raw(
        KnownHtmlAttribute::Height,
        DocumentString::from_utf16(vec![50, 0xD800, 51]),
    );
    let report = HtmlValueInterpreter::new(huge.len())
        .embedded_dimension_attributes(&element(KnownHtmlElement::Img), Some(&width), Some(&height))
        .unwrap();
    assert_eq!(
        report.width().author(),
        HtmlImageAttributeAuthorState::Conforming
    );
    assert_eq!(report.width().recovered().unwrap().decimal_digits(), huge);
    assert!(report.width().recovered().unwrap().to_u128().is_err());
    assert_eq!(report.height().author(), malformed());
    assert_eq!(report.height().recovered().unwrap().decimal_digits(), "2");
    assert_eq!(
        report.height().attribute().raw().unwrap().code_units(),
        &[50, 0xD800, 51]
    );
    // Existing exact native integer serialization suffices for width authoring.
    let native = HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(huge)).unwrap();
    let emitted = raw(KnownHtmlAttribute::Width, native.decimal_digits());
    assert_eq!(emitted, width);
}

#[test]
fn actual_picture_parent_is_distinct_from_known_other_no_parent_and_unavailable() {
    let width = raw(KnownHtmlAttribute::Width, "2");
    let interpreter = HtmlValueInterpreter::new(20);
    for (parent, expected, author, selected) in [
        (
            HtmlImageSourceParent::Parent(element(KnownHtmlElement::Picture)),
            HtmlAttributeContextAuthorState::Conforming,
            HtmlImageAttributeAuthorState::Conforming,
            true,
        ),
        (
            HtmlImageSourceParent::Parent(element(KnownHtmlElement::Video)),
            source_forbidden(),
            HtmlImageAttributeAuthorState::Unapplied,
            false,
        ),
        (
            HtmlImageSourceParent::NoParent,
            source_forbidden(),
            HtmlImageAttributeAuthorState::Unapplied,
            false,
        ),
        (
            HtmlImageSourceParent::Unavailable,
            HtmlAttributeContextAuthorState::Unavailable,
            HtmlImageAttributeAuthorState::Unchecked,
            false,
        ),
        (
            HtmlImageSourceParent::Parent(QualifiedName::new(Namespace::svg(), None, "picture")),
            source_forbidden(),
            HtmlImageAttributeAuthorState::Unapplied,
            false,
        ),
        (
            HtmlImageSourceParent::Parent(QualifiedName::new(Namespace::html(), None, "PICTURE")),
            source_forbidden(),
            HtmlImageAttributeAuthorState::Unapplied,
            false,
        ),
    ] {
        let context = HtmlEmbeddedDimensionContext::SourceParent(parent);
        let report = interpreter
            .embedded_dimension_attributes_with_context(
                &element(KnownHtmlElement::Source),
                Some(&width),
                None::<&surgeist_html::Attribute>,
                &context,
            )
            .unwrap();
        assert_eq!(report.context(), Some(&context));
        assert_eq!(report.width().author(), author);
        assert_eq!(report.width().attribute().context_author(), expected);
        assert_eq!(report.width().common().is_some(), selected);
    }
}

#[test]
fn wrong_fact_family_is_typed_and_does_not_run_dimension_grammar() {
    let width = raw(KnownHtmlAttribute::Width, "garbage");
    for (name, context) in [
        (
            KnownHtmlElement::Source,
            HtmlEmbeddedDimensionContext::InputState(HtmlInputType::Image),
        ),
        (
            KnownHtmlElement::Input,
            HtmlEmbeddedDimensionContext::SourceParent(HtmlImageSourceParent::NoParent),
        ),
        (
            KnownHtmlElement::Img,
            HtmlEmbeddedDimensionContext::InputState(HtmlInputType::Image),
        ),
    ] {
        let report = HtmlValueInterpreter::new(20)
            .embedded_dimension_attributes_with_context(
                &element(name),
                Some(&width),
                None::<&surgeist_html::Attribute>,
                &context,
            )
            .unwrap();
        assert_eq!(report.context(), Some(&context));
        assert_eq!(
            report.width().attribute().applicability(),
            HtmlAttributeApplicability::DifferentContext
        );
        assert_eq!(
            report.width().author(),
            HtmlImageAttributeAuthorState::Unapplied
        );
        assert!(report.width().common().is_none());
    }
}

#[test]
fn exact_expanded_roles_and_prefixes_survive_without_cross_slot_parsing() {
    let wrong = raw(KnownHtmlAttribute::Title, "garbage");
    let src = raw(KnownHtmlAttribute::Src, "x");
    let strings = HtmlValueInterpreter::new(20)
        .input_image_attributes_for_state(
            &element(KnownHtmlElement::Input),
            HtmlInputType::Image,
            Some(&wrong),
            Some(&src),
        )
        .unwrap();
    assert_eq!(
        strings.alt().attribute().applicability(),
        HtmlAttributeApplicability::DifferentAttribute
    );
    assert_eq!(strings.alt().attribute().raw_attribute(), Some(&wrong));
    assert_eq!(
        strings.alt().author(),
        HtmlImageAttributeAuthorState::Unapplied
    );
    assert_eq!(
        strings.src().author(),
        HtmlImageAttributeAuthorState::Conforming
    );
    let prefixed = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "width"),
        "02",
    );
    let report = HtmlValueInterpreter::new(20)
        .embedded_dimension_attributes(
            &element(KnownHtmlElement::Img),
            Some(&prefixed),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(report.width().attribute().raw_attribute(), Some(&prefixed));
    assert_eq!(report.width().recovered().unwrap().decimal_digits(), "2");
    for name in [
        QualifiedName::new(Namespace::none(), None, "WIDTH"),
        QualifiedName::new(Namespace::svg(), None, "width"),
    ] {
        let width = Attribute::new(name, "2");
        let report = HtmlValueInterpreter::new(20)
            .embedded_dimension_attributes(
                &element(KnownHtmlElement::Img),
                Some(&width),
                None::<&surgeist_html::Attribute>,
            )
            .unwrap();
        assert_eq!(
            report.width().attribute().applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert!(report.width().common().is_none());
    }
    for name in [
        QualifiedName::new(Namespace::svg(), None, "input"),
        QualifiedName::new(Namespace::html(), None, "INPUT"),
        element(KnownHtmlElement::Img),
    ] {
        let report = HtmlValueInterpreter::new(20)
            .input_image_attributes_for_state(
                &name,
                HtmlInputType::Image,
                None::<&surgeist_html::Attribute>,
                Some(&src),
            )
            .unwrap();
        assert_eq!(
            report.src().author(),
            HtmlImageAttributeAuthorState::Unapplied
        );
        assert_eq!(report.src().source_reference(), None);
    }
}

#[test]
fn every_original_slot_is_capped_before_roles_context_and_trimming() {
    let too_long = raw(KnownHtmlAttribute::Src, "   x   ");
    let alt = raw(KnownHtmlAttribute::Title, "x");
    let interpreter = HtmlValueInterpreter::new(6);
    let error = interpreter
        .input_image_attributes_for_state(
            &element(KnownHtmlElement::Div),
            HtmlInputType::Text,
            Some(&alt),
            Some(&too_long),
        )
        .unwrap_err();
    assert_eq!((error.capacity(), error.required()), (6, 7));
    let long_alt = raw(
        KnownHtmlAttribute::Alt,
        DocumentString::from_utf16(vec![0xD800; 7]),
    );
    assert!(
        interpreter
            .input_image_attributes(
                &element(KnownHtmlElement::Div),
                Some(&long_alt),
                None::<&surgeist_html::Attribute>
            )
            .is_err()
    );
    let height = raw(KnownHtmlAttribute::Height, "1234567");
    assert!(
        interpreter
            .embedded_dimension_attributes_with_context(
                &element(KnownHtmlElement::Source),
                None::<&surgeist_html::Attribute>,
                Some(&height),
                &HtmlEmbeddedDimensionContext::SourceParent(HtmlImageSourceParent::NoParent)
            )
            .is_err()
    );
    let width = raw(KnownHtmlAttribute::Width, "1234567");
    assert!(
        interpreter
            .embedded_dimension_attributes(
                &element(KnownHtmlElement::Div),
                Some(&width),
                None::<&surgeist_html::Attribute>
            )
            .is_err()
    );
    let exact = HtmlValueInterpreter::new(7)
        .input_image_attributes_for_state(
            &element(KnownHtmlElement::Input),
            HtmlInputType::Image,
            Some(&raw(KnownHtmlAttribute::Alt, "x")),
            Some(&too_long),
        )
        .unwrap();
    assert_eq!(
        exact.src().author(),
        HtmlImageAttributeAuthorState::Conforming
    );
    assert_eq!(exact.src().attribute().input_code_units(), 7);
}

#[test]
fn native_image_bundle_preserves_exact_units_and_emits_actual_image_type() {
    assert_eq!(
        HtmlInputImageAttributes::new("", "x"),
        Err(HtmlInputImageAttributeError::EmptyAlt)
    );
    assert_eq!(
        HtmlInputImageAttributes::new("label", " \t\r\n\u{c}"),
        Err(HtmlInputImageAttributeError::EmptySource)
    );
    let alt = DocumentString::from_utf16(vec![32, 0, 0xD800]);
    let src = DocumentString::from_utf16(vec![32, 46, 47, 0xDC00, 32]);
    let native = HtmlInputImageAttributes::new(alt.clone(), src.clone()).unwrap();
    assert_eq!(native.alt(), &alt);
    assert_eq!(native.src(), &src);
    let emitted = native.into_attributes();
    assert_eq!(emitted[0], raw(KnownHtmlAttribute::Type, "image"));
    assert_eq!(emitted[1], raw(KnownHtmlAttribute::Alt, alt));
    assert_eq!(emitted[2], raw(KnownHtmlAttribute::Src, src));
    let ordered = OrderedAttributes::new(emitted).unwrap();
    assert_eq!(ordered.len(), 3);
}

#[test]
fn reports_link_the_actual_defining_clauses_not_generic_canvas_or_img_alt_rules() {
    let interpreter = HtmlValueInterpreter::new(20);
    let strings = interpreter
        .input_image_attributes_for_state(
            &element(KnownHtmlElement::Input),
            HtmlInputType::Image,
            None::<&surgeist_html::Attribute>,
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(
        strings.alt().attribute().source_clause(),
        format!("{BASE}attr-input-alt")
    );
    assert_eq!(
        strings.src().attribute().source_clause(),
        format!("{BASE}attr-input-src")
    );
    assert_eq!(strings.alt().attribute().inventory_rows(), &[9]);
    assert_eq!(strings.src().attribute().inventory_rows(), &[149]);
    assert_eq!(
        strings.src().attribute().context_source_clause(),
        Some(format!("{BASE}image-button-state-(type=image)").as_str())
    );
    let forbidden = interpreter
        .input_image_attributes_for_state(
            &element(KnownHtmlElement::Input),
            HtmlInputType::Text,
            None::<&surgeist_html::Attribute>,
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(
        forbidden.alt().attribute().context_source_clause(),
        Some(format!("{BASE}text-(type=text)-state-and-search-state-(type=search)").as_str())
    );
    let dimensions = interpreter
        .embedded_dimension_attributes(
            &element(KnownHtmlElement::Img),
            None::<&surgeist_html::Attribute>,
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    for slot in [dimensions.width(), dimensions.height()] {
        assert_eq!(
            slot.attribute().source_clause(),
            format!("{BASE}dimension-attributes")
        );
        assert_eq!(
            slot.attribute().recognition(),
            HtmlAttributeRecognition::Unapplied
        );
        assert_eq!(
            slot.attribute().author(),
            HtmlAttributeAuthorState::Unapplied
        );
    }
    assert_eq!(dimensions.width().attribute().inventory_rows(), &[178]);
    assert_eq!(dimensions.height().attribute().inventory_rows(), &[64]);
    let source = interpreter
        .embedded_dimension_attributes(
            &element(KnownHtmlElement::Source),
            None::<&surgeist_html::Attribute>,
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(
        source.width().attribute().context_source_clause(),
        Some(format!("{BASE}the-source-element").as_str())
    );
}
