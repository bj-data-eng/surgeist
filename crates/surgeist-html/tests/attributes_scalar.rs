use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeAuthorState,
    HtmlAttributeContextAuthorState, HtmlAttributeContextConformanceError,
    HtmlAttributeObservation, HtmlAttributeRecognition, HtmlButtonSubmitterContext,
    HtmlFormEnctype, HtmlFormMethod, HtmlInputType, HtmlInteger, HtmlNonNegativeInteger,
    HtmlNumericAttributeAuthorState, HtmlNumericAttributeConformanceError,
    HtmlNumericAttributeReport, HtmlNumericAttributeValueOrigin, HtmlTabindexValue,
    HtmlValueAdmissionError, HtmlValueAuthorState, HtmlValueInterpreter, HtmlValueRecoveryState,
    KnownHtmlAttribute, KnownHtmlElement, Namespace, QualifiedName,
};

fn raw(attribute: KnownHtmlAttribute, value: &str) -> Attribute {
    Attribute::new(attribute.qualified_name(), value)
}

type NonnegativeOperation = fn(
    &HtmlValueInterpreter,
    &QualifiedName,
    Option<&Attribute>,
) -> Result<
    HtmlNumericAttributeReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
    HtmlValueAdmissionError,
>;

// Expectations are the individual defining clauses, not an inventory summary.
const NUMERIC_CASES: [(
    NonnegativeOperation,
    KnownHtmlElement,
    KnownHtmlAttribute,
    &str,
    u16,
); 4] = [
    (
        HtmlValueInterpreter::canvas_width,
        KnownHtmlElement::Canvas,
        KnownHtmlAttribute::Width,
        "300",
        178,
    ),
    (
        HtmlValueInterpreter::canvas_height,
        KnownHtmlElement::Canvas,
        KnownHtmlAttribute::Height,
        "150",
        64,
    ),
    (
        HtmlValueInterpreter::textarea_cols,
        KnownHtmlElement::Textarea,
        KnownHtmlAttribute::Cols,
        "20",
        26,
    ),
    (
        HtmlValueInterpreter::textarea_rows,
        KnownHtmlElement::Textarea,
        KnownHtmlAttribute::Rows,
        "2",
        131,
    ),
];

#[test]
fn missing_and_failed_numeric_content_use_each_defining_default() {
    let interpreter = HtmlValueInterpreter::new(64);
    for (operation, element, attribute, default, row) in NUMERIC_CASES {
        let element = element.qualified_name();
        let missing = operation(&interpreter, &element, None).unwrap();
        assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
        assert_eq!(missing.author(), HtmlNumericAttributeAuthorState::Missing);
        assert_eq!(missing.intrinsic().unwrap().decimal_digits(), default);
        assert_eq!(
            missing.origin(),
            Some(HtmlNumericAttributeValueOrigin::MissingDefault)
        );
        assert_eq!(missing.inventory_rows(), &[row]);
        assert!(matches!(
            missing.common().unwrap().recovery(),
            HtmlValueRecoveryState::Missing
        ));
        for value in ["", "-1", "\u{a0}12", "no"] {
            let attribute = raw(attribute, value);
            let report = operation(&interpreter, &element, Some(&attribute)).unwrap();
            assert_eq!(report.raw_attribute(), Some(&attribute));
            assert_eq!(
                report.intrinsic().unwrap().decimal_digits(),
                default,
                "{value:?}"
            );
            assert_eq!(
                report.origin(),
                Some(HtmlNumericAttributeValueOrigin::ParseFailureDefault)
            );
            assert!(matches!(
                report.common().unwrap().recovery(),
                HtmlValueRecoveryState::Failed(_)
            ));
            assert!(matches!(
                report.author(),
                HtmlNumericAttributeAuthorState::Nonconforming(
                    HtmlNumericAttributeConformanceError::CommonValue(_)
                )
            ));
            assert_eq!(
                report.observation(),
                if value.is_empty() {
                    HtmlAttributeObservation::PresentEmpty
                } else {
                    HtmlAttributeObservation::PresentNonempty
                }
            );
        }
    }
}

#[test]
fn canvas_zero_is_valid_but_textarea_zero_selects_positive_defaults() {
    let interpreter = HtmlValueInterpreter::new(8);
    for (operation, element, attribute, default, _) in NUMERIC_CASES {
        let zero = raw(attribute, "000");
        let report = operation(&interpreter, &element.qualified_name(), Some(&zero)).unwrap();
        assert!(
            matches!(report.common().unwrap().author(), HtmlValueAuthorState::Valid(value) if value.is_zero())
        );
        assert!(
            matches!(report.common().unwrap().recovery(), HtmlValueRecoveryState::Value(value) if value.is_zero())
        );
        if element == KnownHtmlElement::Canvas {
            assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
            assert_eq!(report.intrinsic().unwrap().decimal_digits(), "0");
            assert_eq!(
                report.origin(),
                Some(HtmlNumericAttributeValueOrigin::Recovered)
            );
        } else {
            assert_eq!(
                report.author(),
                HtmlNumericAttributeAuthorState::Nonconforming(
                    HtmlNumericAttributeConformanceError::ZeroMustBePositive
                )
            );
            assert_eq!(report.intrinsic().unwrap().decimal_digits(), default);
            assert_eq!(
                report.origin(),
                Some(HtmlNumericAttributeValueOrigin::ZeroDefault)
            );
        }
    }
}

#[test]
fn numeric_prefix_recovery_does_not_certify_authored_syntax() {
    let interpreter = HtmlValueInterpreter::new(64);
    for (operation, element, attribute, default, _) in NUMERIC_CASES {
        let element = element.qualified_name();
        for value in [" +0012tail", "12.5"] {
            let attribute = raw(attribute, value);
            let report = operation(&interpreter, &element, Some(&attribute)).unwrap();
            assert_eq!(report.intrinsic().unwrap().decimal_digits(), "12");
            assert_eq!(
                report.origin(),
                Some(HtmlNumericAttributeValueOrigin::Recovered)
            );
            assert!(matches!(
                report.common().unwrap().author(),
                HtmlValueAuthorState::Invalid(_)
            ));
            assert!(
                matches!(report.common().unwrap().recovery(), HtmlValueRecoveryState::Value(value) if value.decimal_digits() == "12")
            );
            assert!(matches!(
                report.author(),
                HtmlNumericAttributeAuthorState::Nonconforming(_)
            ));
        }
        let minus_zero = raw(attribute, "-0");
        let report = operation(&interpreter, &element, Some(&minus_zero)).unwrap();
        assert!(matches!(
            report.common().unwrap().author(),
            HtmlValueAuthorState::Invalid(_)
        ));
        assert!(
            matches!(report.common().unwrap().recovery(), HtmlValueRecoveryState::Value(value) if value.is_zero())
        );
        assert_eq!(
            report.intrinsic().unwrap().decimal_digits(),
            if element == KnownHtmlElement::Canvas.qualified_name() {
                "0"
            } else {
                default
            }
        );
    }
}

#[test]
fn numeric_content_preserves_exact_names_prefixes_and_surrogate_units() {
    let interpreter = HtmlValueInterpreter::new(8);
    let element = QualifiedName::new(Namespace::html(), Some("p".into()), "canvas");
    let value = DocumentString::from_utf16(vec![0x31, 0x32, 0xd800]);
    let attribute = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("q".into()), "width"),
        value.clone(),
    );
    let report = interpreter
        .canvas_width(&element, Some(&attribute))
        .unwrap();
    assert_eq!(report.element(), &element);
    assert_eq!(report.raw_attribute(), Some(&attribute));
    assert_eq!(report.raw(), Some(&value));
    assert_eq!(report.input_code_units(), 3);
    assert_eq!(report.intrinsic().unwrap().decimal_digits(), "12");
    assert_eq!(report.common().unwrap().raw(), Some(&value));
    assert!(matches!(
        report.author(),
        HtmlNumericAttributeAuthorState::Nonconforming(_)
    ));
}

#[test]
fn numeric_roles_are_exact_and_unapplied_names_do_not_receive_defaults() {
    let interpreter = HtmlValueInterpreter::new(32);
    for (operation, element, attribute, _, _) in NUMERIC_CASES {
        let valid = raw(attribute, "8");
        for wrong_element in [
            KnownHtmlElement::Div.qualified_name(),
            QualifiedName::new(Namespace::svg(), None, element.local_name()),
            QualifiedName::new(Namespace::html(), None, "CANVAS"),
        ] {
            let report = operation(&interpreter, &wrong_element, Some(&valid)).unwrap();
            assert_ne!(
                report.applicability(),
                HtmlAttributeApplicability::Applicable
            );
            assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Unapplied);
            assert_eq!(report.intrinsic(), None);
            assert_eq!(report.origin(), None);
            assert_eq!(report.common(), None);
            assert_eq!(report.raw_attribute(), Some(&valid));
        }
        for name in [
            QualifiedName::new(Namespace::html(), None, attribute.local_name()),
            KnownHtmlAttribute::Title.qualified_name(),
            QualifiedName::new(Namespace::none(), None, "WIDTH"),
        ] {
            let wrong = Attribute::new(name, "8");
            let report = operation(&interpreter, &element.qualified_name(), Some(&wrong)).unwrap();
            assert_eq!(
                report.applicability(),
                HtmlAttributeApplicability::DifferentAttribute
            );
            assert_eq!(report.raw_attribute(), Some(&wrong));
            assert_eq!(report.intrinsic(), None);
        }
    }
}

#[test]
fn admitted_large_scalar_values_stay_exact_without_webidl_clamping() {
    let digits = "1234567890".repeat(60);
    let interpreter = HtmlValueInterpreter::new(digits.len() + 1);
    for (operation, element, attribute, _, _) in NUMERIC_CASES {
        let attribute = raw(attribute, &digits);
        let report = operation(&interpreter, &element.qualified_name(), Some(&attribute)).unwrap();
        assert_eq!(report.intrinsic().unwrap().decimal_digits(), digits);
        assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
        assert!(report.intrinsic().unwrap().to_u128().is_err());
    }
    let attribute = raw(KnownHtmlAttribute::Tabindex, &format!("-{digits}"));
    let report = interpreter
        .tabindex(&KnownHtmlElement::Button.qualified_name(), Some(&attribute))
        .unwrap();
    let Some(HtmlTabindexValue::Integer(value)) = report.intrinsic() else {
        panic!("exact signed value required")
    };
    assert!(value.is_negative());
    assert_eq!(value.magnitude().decimal_digits(), digits);
    assert!(value.to_i128().is_err());
}

#[test]
fn tabindex_missing_and_failure_are_source_null_not_idl_element_defaults() {
    let interpreter = HtmlValueInterpreter::new(32);
    for element in [
        KnownHtmlElement::Button.qualified_name(),
        KnownHtmlElement::Div.qualified_name(),
        QualifiedName::new(Namespace::html(), None, "custom-name"),
    ] {
        let missing = interpreter.tabindex(&element, None).unwrap();
        assert_eq!(missing.intrinsic(), Some(&HtmlTabindexValue::Null));
        assert_eq!(missing.author(), HtmlNumericAttributeAuthorState::Missing);
        assert_eq!(
            missing.origin(),
            Some(HtmlNumericAttributeValueOrigin::MissingDefault)
        );
        for value in ["", "no", "\u{a0}1"] {
            let attribute = raw(KnownHtmlAttribute::Tabindex, value);
            let report = interpreter.tabindex(&element, Some(&attribute)).unwrap();
            assert_eq!(report.intrinsic(), Some(&HtmlTabindexValue::Null));
            assert_eq!(
                report.origin(),
                Some(HtmlNumericAttributeValueOrigin::ParseFailureDefault)
            );
            assert_eq!(report.inventory_rows(), &[156]);
            assert!(report.source_clause().ends_with("/#attr-tabindex"));
            assert!(
                report
                    .processing_source_clause()
                    .ends_with("/#tabindex-value")
            );
        }
    }
}

#[test]
fn tabindex_signed_recovery_and_long_out_of_range_content_stay_exact() {
    let interpreter = HtmlValueInterpreter::new(32);
    let element = KnownHtmlElement::Summary.qualified_name();
    for (text, expected, conforming) in [
        ("-1", -1, true),
        ("0", 0, true),
        ("2147483648", 2147483648, true),
        ("-2147483649", -2147483649, true),
        ("+1tail", 1, false),
        (" -002tail", -2, false),
    ] {
        let attribute = raw(KnownHtmlAttribute::Tabindex, text);
        let report = interpreter.tabindex(&element, Some(&attribute)).unwrap();
        assert_eq!(
            report.intrinsic(),
            Some(&HtmlTabindexValue::Integer(HtmlInteger::from(expected)))
        );
        assert_eq!(
            report.origin(),
            Some(HtmlNumericAttributeValueOrigin::Recovered)
        );
        assert_eq!(
            report.author() == HtmlNumericAttributeAuthorState::Conforming,
            conforming
        );
        assert_eq!(report.raw_attribute(), Some(&attribute));
    }
}

#[test]
fn tabindex_retains_exact_prefixes_and_units_and_rejects_other_expanded_names() {
    let interpreter = HtmlValueInterpreter::new(3);
    let element = QualifiedName::new(Namespace::html(), Some("p".into()), "div");
    let attribute = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("q".into()), "tabindex"),
        DocumentString::from_utf16(vec![0x2d, 0x31, 0xd800]),
    );
    let report = interpreter.tabindex(&element, Some(&attribute)).unwrap();
    assert_eq!(report.element(), &element);
    assert_eq!(report.raw_attribute(), Some(&attribute));
    assert_eq!(report.input_code_units(), 3);
    assert_eq!(
        report.intrinsic(),
        Some(&HtmlTabindexValue::Integer((-1).into()))
    );
    assert!(matches!(
        report.author(),
        HtmlNumericAttributeAuthorState::Nonconforming(_)
    ));
    assert_eq!(report.common().unwrap().raw(), Some(attribute.value()));
    for name in [
        QualifiedName::new(Namespace::html(), None, "tabindex"),
        QualifiedName::new(Namespace::none(), None, "TABINDEX"),
        KnownHtmlAttribute::Title.qualified_name(),
    ] {
        let wrong = Attribute::new(name, "0");
        let report = interpreter.tabindex(&element, Some(&wrong)).unwrap();
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.common(), None);
        assert_eq!(report.raw_attribute(), Some(&wrong));
    }
}

#[test]
fn scalar_admission_precedes_even_unapplied_names_and_empty_is_admitted() {
    let zero = HtmlValueInterpreter::new(0);
    let wrong_element = KnownHtmlElement::Div.qualified_name();
    for (operation, element, attribute, default, _) in NUMERIC_CASES {
        let missing = operation(&zero, &element.qualified_name(), None).unwrap();
        assert_eq!(missing.intrinsic().unwrap().decimal_digits(), default);
        let empty = raw(attribute, "");
        assert!(operation(&zero, &element.qualified_name(), Some(&empty)).is_ok());
        let over = raw(KnownHtmlAttribute::Title, "1");
        let error = operation(&zero, &wrong_element, Some(&over)).unwrap_err();
        assert_eq!(error.capacity(), 0);
        assert_eq!(error.required(), 1);
    }
    let over = raw(KnownHtmlAttribute::Title, "1");
    assert_eq!(
        zero.tabindex(&wrong_element, Some(&over))
            .unwrap_err()
            .required(),
        1
    );
    let foreign = QualifiedName::new(Namespace::svg(), None, "a");
    let attribute = raw(KnownHtmlAttribute::Tabindex, "0");
    let report = HtmlValueInterpreter::new(1)
        .tabindex(&foreign, Some(&attribute))
        .unwrap();
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::ForeignElementNamespace
    );
    assert_eq!(report.intrinsic(), None);
}

#[test]
fn native_scalar_construction_checks_positive_authoring_without_machine_limits() {
    let zero = HtmlNonNegativeInteger::from(0);
    assert_eq!(
        zero.clone().into_canvas_width_attribute(),
        raw(KnownHtmlAttribute::Width, "0")
    );
    assert_eq!(
        zero.clone().into_canvas_height_attribute(),
        raw(KnownHtmlAttribute::Height, "0")
    );
    assert_eq!(
        zero.clone().into_textarea_cols_attribute(),
        Err(HtmlNumericAttributeConformanceError::ZeroMustBePositive)
    );
    assert_eq!(
        zero.into_textarea_rows_attribute(),
        Err(HtmlNumericAttributeConformanceError::ZeroMustBePositive)
    );
    let digits = "9999999999999999999999999999999999999999999999999";
    let value = HtmlNonNegativeInteger::from_decimal_digits(&digits.into()).unwrap();
    assert_eq!(
        value.clone().into_textarea_rows_attribute().unwrap(),
        raw(KnownHtmlAttribute::Rows, digits)
    );
    assert_eq!(
        value.clone().into_textarea_cols_attribute().unwrap(),
        raw(KnownHtmlAttribute::Cols, digits)
    );
    let signed = HtmlInteger::from_magnitude(value, true);
    assert_eq!(
        signed.into_tabindex_attribute(),
        raw(KnownHtmlAttribute::Tabindex, &format!("-{digits}"))
    );
}

#[test]
fn submitter_override_keywords_reuse_literal_grammars_and_missing_has_no_state() {
    let interpreter = HtmlValueInterpreter::new(64);
    for element in [KnownHtmlElement::Input, KnownHtmlElement::Button] {
        let element = element.qualified_name();
        let missing_method = interpreter.formmethod(&element, None).unwrap();
        let missing_enctype = interpreter.formenctype(&element, None).unwrap();
        assert_eq!(missing_method.intrinsic(), None);
        assert_eq!(missing_enctype.intrinsic(), None);
        assert_eq!(
            missing_method.recognition(),
            HtmlAttributeRecognition::Missing
        );
        assert_eq!(missing_method.author(), HtmlAttributeAuthorState::Missing);
        assert_eq!(
            missing_method.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        for (text, expected) in [
            ("get", HtmlFormMethod::Get),
            ("PoSt", HtmlFormMethod::Post),
            ("dialog", HtmlFormMethod::Dialog),
        ] {
            let attribute = raw(KnownHtmlAttribute::Formmethod, text);
            let report = interpreter.formmethod(&element, Some(&attribute)).unwrap();
            assert_eq!(report.intrinsic(), Some(&expected));
            assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::Unchecked
            );
            assert_eq!(report.inventory_rows(), &[58]);
            assert!(report.source_clause().ends_with("/#attr-fs-formmethod"));
        }
        for (text, expected) in [
            (
                "application/x-www-form-urlencoded",
                HtmlFormEnctype::UrlEncoded,
            ),
            ("MULTIPART/form-data", HtmlFormEnctype::MultipartFormData),
            ("text/plain", HtmlFormEnctype::PlainText),
        ] {
            let attribute = raw(KnownHtmlAttribute::Formenctype, text);
            let report = interpreter.formenctype(&element, Some(&attribute)).unwrap();
            assert_eq!(report.intrinsic(), Some(&expected));
            assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::Unchecked
            );
            assert_eq!(report.inventory_rows(), &[57]);
            assert!(report.source_clause().ends_with("/#attr-fs-formenctype"));
        }
    }
}

#[test]
fn present_empty_invalid_or_surrounded_override_values_select_invalid_defaults() {
    let interpreter = HtmlValueInterpreter::new(64);
    let element = KnownHtmlElement::Button.qualified_name();
    for text in ["", "unknown", " post", "post ", "get,post"] {
        let attribute = raw(KnownHtmlAttribute::Formmethod, text);
        let report = interpreter.formmethod(&element, Some(&attribute)).unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlFormMethod::Get));
        assert!(matches!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(_)
        ));
        assert_eq!(report.raw_attribute(), Some(&attribute));
        assert_eq!(
            report.recognition(),
            if text.is_empty() {
                HtmlAttributeRecognition::Empty
            } else {
                HtmlAttributeRecognition::Unrecognized
            }
        );
    }
    for text in [
        "",
        "unknown",
        " text/plain",
        "text/plain ",
        "text/plain; charset=UTF-8",
    ] {
        let attribute = raw(KnownHtmlAttribute::Formenctype, text);
        let report = interpreter.formenctype(&element, Some(&attribute)).unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlFormEnctype::UrlEncoded));
        assert!(matches!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(_)
        ));
    }
}

#[test]
fn submitter_facts_observe_authorship_without_erasing_override_keywords() {
    let interpreter = HtmlValueInterpreter::new(64);
    let input = KnownHtmlElement::Input.qualified_name();
    let button = KnownHtmlElement::Button.qualified_name();
    let method = raw(KnownHtmlAttribute::Formmethod, "post");
    let enctype = raw(KnownHtmlAttribute::Formenctype, "multipart/form-data");
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
        HtmlInputType::Color,
        HtmlInputType::Checkbox,
        HtmlInputType::Radio,
        HtmlInputType::File,
        HtmlInputType::Submit,
        HtmlInputType::Image,
        HtmlInputType::Reset,
        HtmlInputType::Button,
    ] {
        let expected = if matches!(state, HtmlInputType::Submit | HtmlInputType::Image) {
            HtmlAttributeContextAuthorState::Conforming
        } else {
            HtmlAttributeContextAuthorState::Nonconforming(
                HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
            )
        };
        let report = interpreter
            .formmethod_for_input(&input, Some(&method), state)
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlFormMethod::Post));
        assert_eq!(report.context_author(), expected);
        assert_eq!(report.input_state(), Some(state));
        assert!(report.context_source_clause().is_some());
        let report = interpreter
            .formenctype_for_input(&input, Some(&enctype), state)
            .unwrap();
        assert_eq!(
            report.intrinsic(),
            Some(&HtmlFormEnctype::MultipartFormData)
        );
        assert_eq!(report.context_author(), expected);
        assert_eq!(
            interpreter
                .formmethod_for_input(&input, None, state)
                .unwrap()
                .context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
    }
    for context in [
        HtmlButtonSubmitterContext::SubmitButton,
        HtmlButtonSubmitterContext::NotSubmitButton,
    ] {
        let expected = if context == HtmlButtonSubmitterContext::SubmitButton {
            HtmlAttributeContextAuthorState::Conforming
        } else {
            HtmlAttributeContextAuthorState::Nonconforming(
                HtmlAttributeContextConformanceError::FormOverrideRequiresSubmitButton,
            )
        };
        let report = interpreter
            .formmethod_for_button(&button, Some(&method), context)
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlFormMethod::Post));
        assert_eq!(report.context_author(), expected);
        assert_eq!(report.button_submitter_context(), Some(context));
        assert!(
            report
                .context_source_clause()
                .unwrap()
                .ends_with("/#the-button-element")
        );
        let report = interpreter
            .formenctype_for_button(&button, Some(&enctype), context)
            .unwrap();
        assert_eq!(
            report.intrinsic(),
            Some(&HtmlFormEnctype::MultipartFormData)
        );
        assert_eq!(report.context_author(), expected);
        assert_eq!(
            interpreter
                .formenctype_for_button(&button, None, context)
                .unwrap()
                .intrinsic(),
            None
        );
    }
}

#[test]
fn invalid_overrides_preserve_lexical_and_context_author_errors_together() {
    let interpreter = HtmlValueInterpreter::new(8);
    let button = KnownHtmlElement::Button.qualified_name();
    let attribute = raw(KnownHtmlAttribute::Formmethod, "no");
    let report = interpreter
        .formmethod_for_button(
            &button,
            Some(&attribute),
            HtmlButtonSubmitterContext::NotSubmitButton,
        )
        .unwrap();
    assert!(matches!(
        report.author(),
        HtmlAttributeAuthorState::Nonconforming(_)
    ));
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Nonconforming(
            HtmlAttributeContextConformanceError::FormOverrideRequiresSubmitButton
        )
    );
    assert_eq!(report.intrinsic(), Some(&HtmlFormMethod::Get));
    let input = KnownHtmlElement::Input.qualified_name();
    let attribute = raw(KnownHtmlAttribute::Formenctype, "no");
    let report = interpreter
        .formenctype_for_input(&input, Some(&attribute), HtmlInputType::Checkbox)
        .unwrap();
    assert!(matches!(
        report.author(),
        HtmlAttributeAuthorState::Nonconforming(_)
    ));
    assert_eq!(
        report.context_author(),
        HtmlAttributeContextAuthorState::Nonconforming(
            HtmlAttributeContextConformanceError::InputStateForbidsAttribute
        )
    );
    assert_eq!(report.intrinsic(), Some(&HtmlFormEnctype::UrlEncoded));
}

#[test]
fn override_contexts_names_and_admission_preserve_exact_unapplied_evidence() {
    let interpreter = HtmlValueInterpreter::new(64);
    let method = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "formmethod"),
        DocumentString::from_utf16(vec![0xd800]),
    );
    let input = KnownHtmlElement::Input.qualified_name();
    let button = KnownHtmlElement::Button.qualified_name();
    let wrong_context = interpreter
        .formmethod_for_button(
            &input,
            Some(&method),
            HtmlButtonSubmitterContext::SubmitButton,
        )
        .unwrap();
    assert_eq!(
        wrong_context.applicability(),
        HtmlAttributeApplicability::DifferentContext
    );
    assert_eq!(wrong_context.raw_attribute(), Some(&method));
    assert_eq!(
        wrong_context.button_submitter_context(),
        Some(HtmlButtonSubmitterContext::SubmitButton)
    );
    assert_eq!(wrong_context.intrinsic(), None);
    let enctype = raw(KnownHtmlAttribute::Formenctype, "text/plain");
    let wrong_context = interpreter
        .formenctype_for_input(&button, Some(&enctype), HtmlInputType::Submit)
        .unwrap();
    assert_eq!(
        wrong_context.applicability(),
        HtmlAttributeApplicability::DifferentContext
    );
    for element in [
        KnownHtmlElement::Form.qualified_name(),
        QualifiedName::new(Namespace::svg(), None, "input"),
        QualifiedName::new(Namespace::html(), None, "INPUT"),
    ] {
        let report = interpreter.formmethod(&element, Some(&method)).unwrap();
        assert_ne!(
            report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.raw_attribute(), Some(&method));
    }
    let wrong = raw(KnownHtmlAttribute::Method, "post");
    assert_eq!(
        interpreter
            .formmethod(&input, Some(&wrong))
            .unwrap()
            .applicability(),
        HtmlAttributeApplicability::DifferentAttribute
    );
    let limit = HtmlValueInterpreter::new(0);
    assert_eq!(
        limit
            .formmethod_for_button(
                &input,
                Some(&method),
                HtmlButtonSubmitterContext::NotSubmitButton
            )
            .unwrap_err()
            .required(),
        1
    );
    assert_eq!(
        limit
            .formenctype(&button, Some(&enctype))
            .unwrap_err()
            .required(),
        10
    );
}

#[test]
fn native_submitter_keywords_construct_override_identity_not_form_identity() {
    for (value, expected) in [
        (HtmlFormMethod::Get, "get"),
        (HtmlFormMethod::Post, "post"),
        (HtmlFormMethod::Dialog, "dialog"),
    ] {
        assert_eq!(
            value.into_formmethod_attribute(),
            raw(KnownHtmlAttribute::Formmethod, expected)
        );
    }
    for (value, expected) in [
        (
            HtmlFormEnctype::UrlEncoded,
            "application/x-www-form-urlencoded",
        ),
        (HtmlFormEnctype::MultipartFormData, "multipart/form-data"),
        (HtmlFormEnctype::PlainText, "text/plain"),
    ] {
        assert_eq!(
            value.into_formenctype_attribute(),
            raw(KnownHtmlAttribute::Formenctype, expected)
        );
    }
}
