use surgeist_html::bounded::{
    HtmlFragmentNameTest, HtmlIntegrationPointComparison as Comparison,
    HtmlIntegrationPointRule as Rule, HtmlMathmlTextPointTest, fragment_initial_modes,
    html_integration_point_encoding_attribute, html_integration_point_rule,
    mathml_text_integration_point, stack_reset_insertion_mode,
};
use surgeist_html::{
    Attribute, DocumentString, HtmlDocumentMode, HtmlFragmentInsertionMode as Insertion,
    HtmlFragmentTokenizerState as Tokenizer, HtmlParserScriptingMode as Scripting, Namespace,
    OrderedAttributes, QualifiedName, fragment_insertion_mode, fragment_tokenizer_state,
    is_html_integration_point, is_mathml_text_integration_point,
};
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureError, HtmlCaptureFact, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCapturedIsValue, HtmlFragmentContext, HtmlScriptingStatus,
};

fn name(namespace: Namespace, local: &str) -> QualifiedName {
    QualifiedName::new(namespace, Some("retained-prefix".into()), local)
}

#[test]
fn ordinary_fragment_decisions_use_only_their_actual_inputs() {
    for (local, insertion) in [
        ("tr", Insertion::InRow),
        ("tbody", Insertion::InTableBody),
        ("caption", Insertion::InCaption),
        ("colgroup", Insertion::InColumnGroup),
        ("table", Insertion::InTable),
        ("template", Insertion::InTemplate),
        ("frameset", Insertion::InFrameset),
        ("html", Insertion::BeforeHead),
        ("noscript", Insertion::InBody),
    ] {
        assert_eq!(
            fragment_insertion_mode(&name(Namespace::html(), local)),
            insertion
        );
    }
    for (local, effective, tokenizer) in [
        ("title", Scripting::Disabled, Tokenizer::Rcdata),
        ("textarea", Scripting::Inert, Tokenizer::Rcdata),
        ("style", Scripting::Disabled, Tokenizer::Rawtext),
        ("noscript", Scripting::Disabled, Tokenizer::Data),
        ("noscript", Scripting::Inert, Tokenizer::Rawtext),
        ("script", Scripting::Fragment, Tokenizer::ScriptData),
        ("plaintext", Scripting::Normal, Tokenizer::Plaintext),
        ("TITLE", Scripting::Disabled, Tokenizer::Data),
    ] {
        assert_eq!(
            fragment_tokenizer_state(&name(Namespace::html(), local), effective),
            tokenizer
        );
    }
    let foreign = name(Namespace::svg(), "title");
    assert_eq!(
        fragment_tokenizer_state(&foreign, Scripting::Inert),
        Tokenizer::Data
    );
    assert_eq!(fragment_insertion_mode(&foreign), Insertion::InBody);
    let exact = QualifiedName::new(
        Namespace::html(),
        None,
        DocumentString::from_utf16(vec![b't' as u16, b'r' as u16, 0xd800]),
    );
    assert_eq!(fragment_insertion_mode(&exact), Insertion::InBody);
}

#[test]
fn capture_insertion_does_not_require_unavailable_scripting() {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(2, 10, 0, 1000, 1, 0, 0, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let element = builder
        .add_element(
            &document,
            name(Namespace::html(), "table"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&element).unwrap();
    let capture = builder.finish().unwrap();
    let context =
        HtmlFragmentContext::element(&capture, &capture.subject_id(&element).unwrap()).unwrap();
    assert_eq!(context.initial_insertion_mode(), Insertion::InTable);
    assert!(matches!(
        context.initial_tokenizer_state(Scripting::Inert),
        Err(HtmlCaptureError::MissingFact {
            fact: HtmlCaptureFact::ScriptingStatus,
            ..
        })
    ));
}

#[test]
fn fragment_policy_refusal_stops_before_any_later_comparison() {
    let mut calls = 0;
    let result = fragment_initial_modes(Scripting::Inert, |test| {
        calls += 1;
        match test {
            HtmlFragmentNameTest::HtmlNamespace => Ok(true),
            HtmlFragmentNameTest::Local(choices) => {
                assert_eq!(choices, ["title", "textarea"]);
                Err("admission refused")
            }
        }
    });
    assert_eq!(result, Err("admission refused"));
    assert_eq!(calls, 2);

    let mut calls = 0;
    let modes = fragment_initial_modes(Scripting::Normal, |test| {
        calls += 1;
        assert!(matches!(test, HtmlFragmentNameTest::HtmlNamespace));
        Ok::<_, ()>(false)
    })
    .unwrap();
    assert_eq!(modes, (Tokenizer::Data, Insertion::InBody));
    assert_eq!(calls, 1);
}

#[test]
fn combined_fragment_policy_keeps_tokenizer_groups_and_fragment_mapping_order() {
    let mut groups = Vec::new();
    let result = fragment_initial_modes(Scripting::Inert, |test| match test {
        HtmlFragmentNameTest::HtmlNamespace => Ok::<_, ()>(true),
        HtmlFragmentNameTest::Local(choices) => {
            groups.push(choices.to_vec());
            Ok(choices.contains(&"noscript"))
        }
    })
    .unwrap();
    assert_eq!(result, (Tokenizer::Rawtext, Insertion::InBody));
    assert_eq!(
        &groups[..3],
        [
            vec!["title", "textarea"],
            vec!["style", "xmp", "iframe", "noembed", "noframes"],
            vec!["noscript"],
        ]
    );
    assert_eq!(
        &groups[3..],
        [
            vec!["tr"],
            vec!["tbody"],
            vec!["thead"],
            vec!["tfoot"],
            vec!["caption"],
            vec!["colgroup"],
            vec!["table"],
            vec!["template"],
            vec!["frameset"],
            vec!["html"],
        ]
    );
}

#[test]
fn fixed_stack_reset_excludes_state_sensitive_fragment_only_rows() {
    let expected = [
        "tr", "tbody", "thead", "tfoot", "caption", "colgroup", "table", "body",
    ];
    let mut seen = Vec::new();
    let mode = stack_reset_insertion_mode(|local| {
        seen.push(local);
        Ok::<_, ()>(local == "body")
    })
    .unwrap();
    assert_eq!(mode, Some(Insertion::InBody));
    assert_eq!(seen, expected);
    for refused in 0..expected.len() {
        let mut seen = Vec::new();
        assert_eq!(
            stack_reset_insertion_mode(|local| {
                seen.push(local);
                if seen.len() - 1 == refused {
                    Err(refused)
                } else {
                    Ok(false)
                }
            }),
            Err(refused)
        );
        assert_eq!(seen, expected[..=refused]);
    }
}

fn compare(value: &DocumentString, literal: &str, comparison: Comparison) -> bool {
    value
        .code_units()
        .iter()
        .copied()
        .map(|unit| match comparison {
            Comparison::AsciiCaseInsensitive if (65..=90).contains(&unit) => unit + 32,
            _ => unit,
        })
        .eq(literal.encode_utf16())
}

#[test]
fn html_integration_uses_expanded_names_and_exact_unpadded_mime_values() {
    let annotation = name(Namespace::mathml(), "annotation-xml");
    let attributes = OrderedAttributes::new([Attribute::new(
        QualifiedName::new(Namespace::none(), Some("raw-prefix".into()), "encoding"),
        "TEXT/HTML",
    )])
    .unwrap();
    assert!(is_html_integration_point(&annotation, &attributes));
    assert!(!is_html_integration_point(
        &annotation,
        &OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "encoding"),
            " text/html"
        ),])
        .unwrap()
    ));
    assert!(is_html_integration_point(
        &name(Namespace::svg(), "foreignObject"),
        &OrderedAttributes::default()
    ));
    assert!(!is_html_integration_point(
        &name(Namespace::svg(), "foreignobject"),
        &OrderedAttributes::default()
    ));

    assert_eq!(
        html_integration_point_rule(
            annotation.namespace().uri(),
            annotation.local_name(),
            |value, literal, comparison| Ok::<_, ()>(compare(value, literal, comparison)),
        ),
        Ok(Rule::EncodingAttribute)
    );
    let mut calls = 0;
    assert_eq!(
        html_integration_point_encoding_attribute(
            Namespace::html().uri(),
            &"encoding".into(),
            &"text/html".into(),
            |_, _, _| {
                calls += 1;
                Err::<bool, _>("unreached")
            },
        ),
        Ok(None)
    );
    assert_eq!(calls, 0);
}

#[test]
fn html_integration_callback_refusal_preserves_the_first_reached_error() {
    let namespace = DocumentString::from("http://www.w3.org/1998/Math/MathML");
    let local = DocumentString::from("annotation-xml");
    let mut calls = 0;
    assert_eq!(
        html_integration_point_rule(Some(&namespace), &local, |_, _, _| {
            calls += 1;
            if calls == 2 {
                Err("namespace refusal")
            } else {
                Ok(false)
            }
        }),
        Err("namespace refusal")
    );
    assert_eq!(calls, 2);

    let mut seen = Vec::new();
    assert_eq!(
        html_integration_point_encoding_attribute(
            None,
            &"encoding".into(),
            &"text/html".into(),
            |_, literal, comparison| {
                seen.push((literal, comparison));
                if literal == "encoding" {
                    Ok(true)
                } else {
                    Err("value refusal")
                }
            },
        ),
        Err("value refusal")
    );
    assert_eq!(
        seen,
        [
            ("encoding", Comparison::Exact),
            ("text/html", Comparison::AsciiCaseInsensitive)
        ]
    );
}

#[test]
fn mathml_text_points_use_one_ordered_local_group_after_namespace() {
    for local in ["mi", "mo", "mn", "ms", "mtext"] {
        assert!(is_mathml_text_integration_point(&name(
            Namespace::mathml(),
            local
        )));
    }
    assert!(!is_mathml_text_integration_point(&name(
        Namespace::mathml(),
        "mglyph"
    )));
    assert!(!is_mathml_text_integration_point(&name(
        Namespace::html(),
        "mtext"
    )));
    let mut calls = 0;
    assert_eq!(
        mathml_text_integration_point(|test| {
            calls += 1;
            match test {
                HtmlMathmlTextPointTest::Namespace(uri) => {
                    assert_eq!(uri, "http://www.w3.org/1998/Math/MathML");
                    Ok(true)
                }
                HtmlMathmlTextPointTest::Local(choices) => {
                    assert_eq!(choices, ["mi", "mo", "mn", "ms", "mtext"]);
                    Err("local refusal")
                }
            }
        }),
        Err("local refusal")
    );
    assert_eq!(calls, 2);
    let mut calls = 0;
    assert_eq!(
        mathml_text_integration_point(|test| {
            calls += 1;
            assert!(matches!(test, HtmlMathmlTextPointTest::Namespace(_)));
            Ok::<_, ()>(false)
        }),
        Ok(false)
    );
    assert_eq!(calls, 1);
}
