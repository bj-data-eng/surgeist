use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability as Applicability,
    HtmlAttributeAuthorState as Author, HtmlAttributeConformanceError as Conformance,
    HtmlAttributeObservation as Observation, HtmlAttributeRecognition as Recognition,
    HtmlAutocapitalize, HtmlAutocapitalizeState, HtmlAutocorrect, HtmlBooleanAttribute,
    HtmlButtonType, HtmlButtonTypeState, HtmlClosedBy, HtmlClosedByState, HtmlContentEditable,
    HtmlContentEditableState, HtmlDecoding, HtmlDir, HtmlDirState, HtmlDraggable,
    HtmlDraggableState, HtmlEnterKeyHint, HtmlFormAutocomplete, HtmlFormEnctype, HtmlFormMethod,
    HtmlHidden, HtmlHiddenState, HtmlInputMode, HtmlInputType, HtmlLoading, HtmlOlType,
    HtmlPopover, HtmlPopoverState, HtmlPreload, HtmlPreloadState, HtmlShadowRootMode,
    HtmlSlotAssignment, HtmlSpellcheck, HtmlSpellcheckState, HtmlTrackKind, HtmlTranslate,
    HtmlTranslateState, HtmlValueInterpreter, HtmlWritingSuggestions, HtmlWritingSuggestionsState,
    KnownHtmlAttribute, KnownHtmlElement, Namespace, QualifiedName,
};
use surgeist_html::{
    HtmlCrossOrigin, HtmlCrossOriginState, HtmlFetchPriority, HtmlScriptAttributeContext,
};
use surgeist_html::{HtmlHyperlinkAttributeContext, HtmlReferrerPolicy};

fn element(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, local)
}
fn attr(local: &str, value: &str) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, local), value)
}

#[test]
fn referrerpolicy_recognizes_the_pinned_vocabulary_including_conforming_empty() {
    // Actual index.src.html cc435b05ca4a L216–236, not HTTP header parsing.
    let cases = [
        ("", HtmlReferrerPolicy::Empty),
        ("no-referrer", HtmlReferrerPolicy::NoReferrer),
        (
            "no-referrer-when-downgrade",
            HtmlReferrerPolicy::NoReferrerWhenDowngrade,
        ),
        ("same-origin", HtmlReferrerPolicy::SameOrigin),
        ("origin", HtmlReferrerPolicy::Origin),
        ("strict-origin", HtmlReferrerPolicy::StrictOrigin),
        (
            "origin-when-cross-origin",
            HtmlReferrerPolicy::OriginWhenCrossOrigin,
        ),
        (
            "strict-origin-when-cross-origin",
            HtmlReferrerPolicy::StrictOriginWhenCrossOrigin,
        ),
        ("unsafe-url", HtmlReferrerPolicy::UnsafeUrl),
    ];
    let interpreter = HtmlValueInterpreter::new(100);
    for (canonical, state) in cases {
        assert_eq!(state.canonical_keyword(), canonical);
        assert_eq!(state.into_attribute(), attr("referrerpolicy", canonical));
        let mixed: String = canonical
            .chars()
            .enumerate()
            .map(|(index, c)| {
                if index % 2 == 0 {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
            .collect();
        for literal in [canonical.to_owned(), mixed] {
            let raw = attr("referrerpolicy", &literal);
            let report = interpreter
                .referrerpolicy(&element("img"), Some(&raw))
                .unwrap();
            assert_eq!(report.intrinsic(), Some(&state));
            assert_eq!(report.author(), Author::Conforming);
            assert_eq!(
                report.recognition(),
                Recognition::Keyword {
                    matched: canonical,
                    canonical
                }
            );
            assert_eq!(
                report.observation(),
                if canonical.is_empty() {
                    Observation::PresentEmpty
                } else {
                    Observation::PresentNonempty
                }
            );
            assert_eq!(report.raw_attribute(), Some(&raw));
        }
    }
}

#[test]
fn referrerpolicy_empty_default_stays_unresolved_and_raw_states_remain_distinct() {
    let interpreter = HtmlValueInterpreter::new(100);
    let context = element("iframe");
    let missing = interpreter.referrerpolicy(&context, None).unwrap();
    assert_eq!(missing.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
    assert_eq!(missing.observation(), Observation::Missing);
    assert_eq!(missing.recognition(), Recognition::Missing);
    assert_eq!(missing.author(), Author::Missing);
    let empty = attr("referrerpolicy", "");
    let empty_report = interpreter.referrerpolicy(&context, Some(&empty)).unwrap();
    assert_eq!(empty_report.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
    assert_eq!(
        empty_report.recognition(),
        Recognition::Keyword {
            matched: "",
            canonical: ""
        }
    );
    assert_eq!(empty_report.author(), Author::Conforming);
    for literal in [
        "unknown",
        "default",
        "never",
        " origin",
        "origin ",
        "\tno-referrer",
        "no-referrer\n",
        "no-referrer, origin",
        "origin, no-referrer",
        "no-referrer unsafe-url",
        "\u{ff2f}rigin",
    ] {
        let raw = attr("referrerpolicy", literal);
        let report = interpreter.referrerpolicy(&context, Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
        assert_eq!(report.observation(), Observation::PresentNonempty);
        assert_eq!(report.recognition(), Recognition::Unrecognized);
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::UnrecognizedKeyword)
        );
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn referrerpolicy_contexts_use_explicit_hyperlink_and_script_facts() {
    use HtmlHyperlinkAttributeContext::{WithHref, WithoutHref};
    use HtmlScriptAttributeContext::{
        DataBlock, ExternalClassic, ExternalModule, ImportMap, InlineClassic, InlineModule,
        SpeculationRules,
    };
    let interpreter = HtmlValueInterpreter::new(100);
    let raw = attr("referrerpolicy", "ORIGIN");
    for local in ["iframe", "img", "link"] {
        let report = interpreter
            .referrerpolicy(&element(local), Some(&raw))
            .unwrap();
        assert_eq!(report.applicability(), Applicability::Applicable);
        assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Origin));
        assert_eq!(report.inventory_rows(), &[126]);
        assert!(
            report
                .source_clause()
                .ends_with("#referrer-policy-attributes")
        );
        assert_eq!(
            report.vocabulary_source_clause(),
            Some(
                "https://github.com/w3c/webappsec-referrer-policy/blob/cc435b05ca4a94f7f1a139be5074b168d20014db/index.src.html#L216"
            )
        );
    }
    for (local, source) in [("a", "#the-a-element"), ("area", "#the-area-element")] {
        let context = element(local);
        let unspecified = interpreter.referrerpolicy(&context, Some(&raw)).unwrap();
        assert_eq!(
            unspecified.applicability(),
            Applicability::HyperlinkContextRequired
        );
        assert_eq!(unspecified.intrinsic(), None);
        assert_eq!(unspecified.author(), Author::Unapplied);
        assert_eq!(unspecified.hyperlink_context(), None);
        for facts in [WithHref, WithoutHref] {
            let report = interpreter
                .referrerpolicy_for_hyperlink(&context, Some(&raw), facts)
                .unwrap();
            assert_eq!(report.hyperlink_context(), Some(facts));
            assert_eq!(report.script_context(), None);
            assert_eq!(report.element(), &context);
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert!(report.context_source_clause().unwrap().ends_with(source));
            assert_eq!(
                report.applicability(),
                if facts == WithHref {
                    Applicability::Applicable
                } else {
                    Applicability::InapplicableHyperlinkContext
                }
            );
            assert_eq!(
                report.intrinsic(),
                if facts == WithHref {
                    Some(&HtmlReferrerPolicy::Origin)
                } else {
                    None
                }
            );
            assert_eq!(
                report.recognition(),
                if facts == WithHref {
                    Recognition::Keyword {
                        matched: "origin",
                        canonical: "origin",
                    }
                } else {
                    Recognition::Unapplied
                }
            );
            assert_eq!(
                report.author(),
                if facts == WithHref {
                    Author::Conforming
                } else {
                    Author::Unapplied
                }
            );
        }
    }
    assert_eq!(
        interpreter
            .referrerpolicy(&element("script"), Some(&raw))
            .unwrap()
            .applicability(),
        Applicability::ScriptContextRequired
    );
    for (facts, allowed) in [
        (ExternalClassic, true),
        (InlineClassic, true),
        (ExternalModule, true),
        (InlineModule, true),
        (ImportMap, false),
        (SpeculationRules, false),
        (DataBlock, false),
    ] {
        let report = interpreter
            .referrerpolicy_for_script(&element("script"), Some(&raw), facts)
            .unwrap();
        assert_eq!(report.script_context(), Some(facts));
        assert_eq!(report.hyperlink_context(), None);
        assert_eq!(
            report.applicability(),
            if allowed {
                Applicability::Applicable
            } else {
                Applicability::InapplicableScriptKind
            }
        );
        assert_eq!(
            report.intrinsic(),
            if allowed {
                Some(&HtmlReferrerPolicy::Origin)
            } else {
                None
            }
        );
        assert_eq!(
            report.author(),
            if allowed {
                Author::Conforming
            } else {
                Author::Unapplied
            }
        );
        assert!(
            report
                .context_source_clause()
                .unwrap()
                .ends_with("#the-script-element")
        );
    }
    assert_eq!(
        interpreter
            .referrerpolicy_for_hyperlink(&element("img"), Some(&raw), WithHref)
            .unwrap()
            .applicability(),
        Applicability::DifferentElement
    );
    assert_eq!(
        interpreter
            .referrerpolicy_for_hyperlink(&element("script"), Some(&raw), WithHref)
            .unwrap()
            .applicability(),
        Applicability::DifferentElement
    );
    assert_eq!(
        interpreter
            .referrerpolicy_for_script(&element("a"), Some(&raw), ExternalClassic)
            .unwrap()
            .applicability(),
        Applicability::DifferentElement
    );
    assert_eq!(
        interpreter
            .crossorigin(&element("img"), None)
            .unwrap()
            .vocabulary_source_clause(),
        None
    );
}

#[test]
fn referrerpolicy_uses_exact_expanded_names_and_admits_unapplied_raw_units_first() {
    use HtmlHyperlinkAttributeContext::WithoutHref;
    let interpreter = HtmlValueInterpreter::new(100);
    let prefixed = QualifiedName::new(Namespace::html(), Some("h".into()), "a");
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "referrerpolicy"),
        DocumentString::from_utf16(vec![0xd800, 0x0041]),
    );
    let report = interpreter
        .referrerpolicy_for_hyperlink(
            &prefixed,
            Some(&raw),
            HtmlHyperlinkAttributeContext::WithHref,
        )
        .unwrap();
    assert_eq!(report.element(), &prefixed);
    assert_eq!(report.raw_attribute(), Some(&raw));
    assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
    assert_eq!(report.recognition(), Recognition::Unrecognized);
    assert_eq!(report.input_code_units(), 2);
    let error = HtmlValueInterpreter::new(1)
        .referrerpolicy_for_hyperlink(&prefixed, Some(&raw), WithoutHref)
        .unwrap_err();
    assert_eq!(error.capacity(), 1);
    assert_eq!(error.required(), 2);
    for local in ["A", "AREA", "IMG", "audio", "video", "div", "x-link"] {
        assert_eq!(
            interpreter
                .referrerpolicy(&element(local), Some(&raw))
                .unwrap()
                .applicability(),
            Applicability::DifferentElement
        );
    }
    let foreign = QualifiedName::new(Namespace::svg(), None, "a");
    let report = interpreter
        .referrerpolicy_for_hyperlink(&foreign, Some(&raw), WithoutHref)
        .unwrap();
    assert_eq!(
        report.applicability(),
        Applicability::ForeignElementNamespace
    );
    assert_eq!(report.hyperlink_context(), Some(WithoutHref));
    assert_eq!(report.raw_attribute(), Some(&raw));
    for name in [
        QualifiedName::new(Namespace::none(), None, "REFERRERPOLICY"),
        QualifiedName::new(Namespace::html(), None, "referrerpolicy"),
    ] {
        let wrong = Attribute::new(name, "origin");
        let report = interpreter
            .referrerpolicy(&element("img"), Some(&wrong))
            .unwrap();
        assert_eq!(report.applicability(), Applicability::DifferentAttribute);
        assert_eq!(report.author(), Author::Unapplied);
        assert_eq!(report.intrinsic(), None);
    }
}

#[test]
fn cors_intrinsic_defaults_do_not_claim_a_request_mode() {
    let interpreter = HtmlValueInterpreter::new(100);
    for local in ["audio", "video", "img", "link"] {
        let context = element(local);
        let missing = interpreter.crossorigin(&context, None).unwrap();
        assert_eq!(missing.intrinsic(), Some(&HtmlCrossOriginState::NoCors));
        assert_eq!(missing.author(), Author::Missing);
        assert_eq!(missing.recognition(), Recognition::Missing);
        for (literal, state, author, recognition) in [
            (
                "",
                HtmlCrossOrigin::Anonymous,
                Author::Conforming,
                Recognition::Empty,
            ),
            (
                "ANONYMOUS",
                HtmlCrossOrigin::Anonymous,
                Author::Conforming,
                Recognition::Keyword {
                    matched: "anonymous",
                    canonical: "anonymous",
                },
            ),
            (
                "USE-CREDENTIALS",
                HtmlCrossOrigin::UseCredentials,
                Author::Conforming,
                Recognition::Keyword {
                    matched: "use-credentials",
                    canonical: "use-credentials",
                },
            ),
            (
                "false",
                HtmlCrossOrigin::Anonymous,
                Author::Nonconforming(Conformance::UnrecognizedKeyword),
                Recognition::Unrecognized,
            ),
            (
                " anonymous ",
                HtmlCrossOrigin::Anonymous,
                Author::Nonconforming(Conformance::UnrecognizedKeyword),
                Recognition::Unrecognized,
            ),
        ] {
            let raw = attr("crossorigin", literal);
            let report = interpreter.crossorigin(&context, Some(&raw)).unwrap();
            assert_eq!(
                report.intrinsic(),
                Some(&HtmlCrossOriginState::Keyword(state))
            );
            assert_eq!(report.author(), author);
            assert_eq!(report.recognition(), recognition);
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(
                report.observation(),
                if literal.is_empty() {
                    Observation::PresentEmpty
                } else {
                    Observation::PresentNonempty
                }
            );
            assert_eq!(report.inventory_rows(), &[34]);
            assert!(
                report
                    .source_clause()
                    .ends_with("#cors-settings-attributes")
            );
        }
    }
    assert_eq!(
        HtmlCrossOrigin::Anonymous.into_attribute(),
        attr("crossorigin", "anonymous")
    );
    assert_eq!(
        HtmlCrossOrigin::UseCredentials.into_attribute(),
        attr("crossorigin", "use-credentials")
    );
}

#[test]
fn fetchpriority_is_a_literal_intrinsic_hint_with_no_empty_keyword() {
    let interpreter = HtmlValueInterpreter::new(100);
    for local in ["img", "link"] {
        let context = element(local);
        assert_eq!(
            interpreter
                .fetchpriority(&context, None)
                .unwrap()
                .intrinsic(),
            Some(&HtmlFetchPriority::Auto)
        );
        for (literal, expected) in [
            ("HIGH", HtmlFetchPriority::High),
            ("LOW", HtmlFetchPriority::Low),
            ("AUTO", HtmlFetchPriority::Auto),
        ] {
            let raw = attr("fetchpriority", literal);
            let report = interpreter.fetchpriority(&context, Some(&raw)).unwrap();
            assert_eq!(report.intrinsic(), Some(&expected));
            assert_eq!(report.author(), Author::Conforming);
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(
                expected.into_attribute(),
                attr("fetchpriority", &literal.to_ascii_lowercase())
            );
            assert_eq!(report.inventory_rows(), &[51]);
            assert!(
                report
                    .source_clause()
                    .ends_with("#fetch-priority-attributes")
            );
        }
        for literal in [
            "",
            "unknown",
            " high",
            "high ",
            "\u{ff28}\u{ff29}\u{ff27}\u{ff28}",
        ] {
            let raw = attr("fetchpriority", literal);
            let report = interpreter.fetchpriority(&context, Some(&raw)).unwrap();
            assert_eq!(report.intrinsic(), Some(&HtmlFetchPriority::Auto));
            assert_eq!(
                report.author(),
                Author::Nonconforming(Conformance::UnrecognizedKeyword)
            );
            assert_eq!(
                report.recognition(),
                if literal.is_empty() {
                    Recognition::Empty
                } else {
                    Recognition::Unrecognized
                }
            );
        }
    }
}

#[test]
fn script_fetch_attributes_require_supplied_source_table_facts() {
    use HtmlScriptAttributeContext::{
        DataBlock, ExternalClassic, ExternalModule, ImportMap, InlineClassic, InlineModule,
        SpeculationRules,
    };
    let interpreter = HtmlValueInterpreter::new(100);
    let script = element("script");
    let cors = attr("crossorigin", "USE-CREDENTIALS");
    let priority = attr("fetchpriority", "HIGH");
    assert_eq!(
        interpreter
            .crossorigin(&script, Some(&cors))
            .unwrap()
            .applicability(),
        Applicability::ScriptContextRequired
    );
    assert_eq!(
        interpreter
            .fetchpriority(&script, Some(&priority))
            .unwrap()
            .applicability(),
        Applicability::ScriptContextRequired
    );
    // The independent expected booleans are the actual script attribute table.
    for (facts, cors_allowed, priority_allowed) in [
        (ExternalClassic, true, true),
        (InlineClassic, true, false),
        (ExternalModule, true, true),
        (InlineModule, true, false),
        (ImportMap, false, false),
        (SpeculationRules, false, false),
        (DataBlock, false, false),
    ] {
        let report = interpreter
            .crossorigin_for_script(&script, Some(&cors), facts)
            .unwrap();
        assert_eq!(report.script_context(), Some(facts));
        assert_eq!(report.raw_attribute(), Some(&cors));
        assert!(
            report
                .context_source_clause()
                .unwrap()
                .ends_with("#the-script-element")
        );
        assert_eq!(
            report.applicability(),
            if cors_allowed {
                Applicability::Applicable
            } else {
                Applicability::InapplicableScriptKind
            }
        );
        assert_eq!(
            report.intrinsic(),
            if cors_allowed {
                Some(&HtmlCrossOriginState::Keyword(
                    HtmlCrossOrigin::UseCredentials,
                ))
            } else {
                None
            }
        );
        assert_eq!(
            report.author(),
            if cors_allowed {
                Author::Conforming
            } else {
                Author::Unapplied
            }
        );
        let report = interpreter
            .fetchpriority_for_script(&script, Some(&priority), facts)
            .unwrap();
        assert_eq!(report.script_context(), Some(facts));
        assert_eq!(
            report.applicability(),
            if priority_allowed {
                Applicability::Applicable
            } else {
                Applicability::InapplicableScriptKind
            }
        );
        assert_eq!(
            report.intrinsic(),
            if priority_allowed {
                Some(&HtmlFetchPriority::High)
            } else {
                None
            }
        );
        assert_eq!(
            report.recognition(),
            if priority_allowed {
                Recognition::Keyword {
                    matched: "high",
                    canonical: "high",
                }
            } else {
                Recognition::Unapplied
            }
        );
    }
    let missing = interpreter
        .crossorigin_for_script(&script, None, ExternalModule)
        .unwrap();
    assert_eq!(missing.intrinsic(), Some(&HtmlCrossOriginState::NoCors));
    let inline = interpreter
        .fetchpriority_for_script(&script, Some(&priority), InlineModule)
        .unwrap();
    assert_eq!(inline.observation(), Observation::PresentNonempty);
    assert_eq!(inline.author(), Author::Unapplied);
    assert_eq!(
        interpreter
            .crossorigin_for_script(&element("img"), Some(&cors), ExternalClassic)
            .unwrap()
            .applicability(),
        Applicability::DifferentElement
    );
}

#[test]
fn fetch_attribute_names_raw_units_and_admission_remain_exact() {
    let interpreter = HtmlValueInterpreter::new(100);
    let prefixed = QualifiedName::new(Namespace::html(), Some("h".into()), "img");
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "crossorigin"),
        DocumentString::from_utf16(vec![0xd800, 0x0041]),
    );
    let report = interpreter.crossorigin(&prefixed, Some(&raw)).unwrap();
    assert_eq!(report.element(), &prefixed);
    assert_eq!(report.raw_attribute(), Some(&raw));
    assert_eq!(report.input_code_units(), 2);
    assert_eq!(report.recognition(), Recognition::Unrecognized);
    assert_eq!(
        report.intrinsic(),
        Some(&HtmlCrossOriginState::Keyword(HtmlCrossOrigin::Anonymous))
    );
    let error = HtmlValueInterpreter::new(1)
        .crossorigin_for_script(
            &element("div"),
            Some(&raw),
            HtmlScriptAttributeContext::DataBlock,
        )
        .unwrap_err();
    assert_eq!(error.capacity(), 1);
    assert_eq!(error.required(), 2);
    for local in ["IMG", "div", "x-img", "iframe"] {
        assert_eq!(
            interpreter
                .crossorigin(&element(local), Some(&raw))
                .unwrap()
                .applicability(),
            Applicability::DifferentElement
        );
    }
    for local in ["audio", "video", "iframe", "div"] {
        assert_eq!(
            interpreter
                .fetchpriority(&element(local), None)
                .unwrap()
                .applicability(),
            Applicability::DifferentElement
        );
    }
    let foreign = QualifiedName::new(Namespace::svg(), None, "img");
    assert_eq!(
        interpreter
            .crossorigin(&foreign, Some(&raw))
            .unwrap()
            .applicability(),
        Applicability::ForeignElementNamespace
    );
    let wrong = Attribute::new(
        QualifiedName::new(Namespace::html(), None, "crossorigin"),
        "anonymous",
    );
    assert_eq!(
        interpreter
            .crossorigin(&prefixed, Some(&wrong))
            .unwrap()
            .applicability(),
        Applicability::DifferentAttribute
    );
    let wrong = attr("FETCHPRIORITY", "high");
    assert_eq!(
        interpreter
            .fetchpriority(&prefixed, Some(&wrong))
            .unwrap()
            .applicability(),
        Applicability::DifferentAttribute
    );
}

#[test]
fn complete_input_keyword_table_selects_source_states_and_authors_canonical_values() {
    // Independent table from attr-input-type-keywords in the pinned edition.
    let cases = [
        ("hidden", HtmlInputType::Hidden),
        ("text", HtmlInputType::Text),
        ("search", HtmlInputType::Search),
        ("tel", HtmlInputType::Telephone),
        ("url", HtmlInputType::Url),
        ("email", HtmlInputType::Email),
        ("password", HtmlInputType::Password),
        ("date", HtmlInputType::Date),
        ("month", HtmlInputType::Month),
        ("week", HtmlInputType::Week),
        ("time", HtmlInputType::Time),
        ("datetime-local", HtmlInputType::LocalDateTime),
        ("number", HtmlInputType::Number),
        ("range", HtmlInputType::Range),
        ("color", HtmlInputType::Color),
        ("checkbox", HtmlInputType::Checkbox),
        ("radio", HtmlInputType::Radio),
        ("file", HtmlInputType::File),
        ("submit", HtmlInputType::Submit),
        ("image", HtmlInputType::Image),
        ("reset", HtmlInputType::Reset),
        ("button", HtmlInputType::Button),
    ];
    let interpreter = HtmlValueInterpreter::new(100);
    for (canonical, state) in cases {
        assert_eq!(state.canonical_keyword(), canonical);
        assert_eq!(state.into_attribute(), attr("type", canonical));
        for literal in [canonical.to_owned(), canonical.to_ascii_uppercase()] {
            let raw = attr("type", &literal);
            let report = interpreter
                .input_type(&element("input"), Some(&raw))
                .unwrap();
            assert_eq!(report.intrinsic(), Some(&state));
            assert_eq!(report.author(), Author::Conforming);
            assert_eq!(
                report.recognition(),
                Recognition::Keyword {
                    matched: canonical,
                    canonical
                }
            );
            assert_eq!(report.raw_attribute(), Some(&raw));
        }
    }
}

#[test]
fn input_text_fallback_preserves_missing_empty_and_unrecognized_authorship() {
    let interpreter = HtmlValueInterpreter::new(100);
    let context = element("input");
    let omitted = interpreter.input_type(&context, None).unwrap();
    assert_eq!(omitted.intrinsic(), Some(&HtmlInputType::Text));
    assert_eq!(omitted.observation(), Observation::Missing);
    assert_eq!(omitted.recognition(), Recognition::Missing);
    assert_eq!(omitted.author(), Author::Missing);
    // Neither historic datetime nor English state names are admitted aliases.
    for literal in [
        "",
        "datetime",
        "telephone",
        "localdatetime",
        "auto",
        " text",
        "text ",
        "\u{ff34}\u{ff25}\u{ff38}\u{ff34}",
    ] {
        let raw = attr("type", literal);
        let report = interpreter.input_type(&context, Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlInputType::Text));
        assert_eq!(
            report.observation(),
            if literal.is_empty() {
                Observation::PresentEmpty
            } else {
                Observation::PresentNonempty
            }
        );
        assert_eq!(
            report.recognition(),
            if literal.is_empty() {
                Recognition::Empty
            } else {
                Recognition::Unrecognized
            }
        );
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::UnrecognizedKeyword)
        );
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn button_type_keywords_do_not_turn_auto_fallback_into_a_valid_keyword() {
    let interpreter = HtmlValueInterpreter::new(100);
    let context = element("button");
    for (canonical, native) in [
        ("submit", HtmlButtonType::Submit),
        ("reset", HtmlButtonType::Reset),
        ("button", HtmlButtonType::Button),
    ] {
        assert_eq!(native.canonical_keyword(), canonical);
        assert_eq!(native.into_attribute(), attr("type", canonical));
        let raw = attr("type", &canonical.to_ascii_uppercase());
        let report = interpreter.button_type(&context, Some(&raw)).unwrap();
        assert_eq!(
            report.intrinsic(),
            Some(&HtmlButtonTypeState::Keyword(native))
        );
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(
            report.recognition(),
            Recognition::Keyword {
                matched: canonical,
                canonical
            }
        );
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
    let missing = interpreter.button_type(&context, None).unwrap();
    assert_eq!(missing.intrinsic(), Some(&HtmlButtonTypeState::Auto));
    assert_eq!(missing.author(), Author::Missing);
    assert_eq!(missing.recognition(), Recognition::Missing);
    assert_eq!(missing.observation(), Observation::Missing);
    for literal in ["", "auto", "menu", "unknown", "submit "] {
        let raw = attr("type", literal);
        let report = interpreter.button_type(&context, Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlButtonTypeState::Auto));
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::UnrecognizedKeyword)
        );
        assert_eq!(
            report.recognition(),
            if literal.is_empty() {
                Recognition::Empty
            } else {
                Recognition::Unrecognized
            }
        );
        assert_eq!(
            report.observation(),
            if literal.is_empty() {
                Observation::PresentEmpty
            } else {
                Observation::PresentNonempty
            }
        );
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn input_and_button_type_have_separate_exact_contexts_and_defining_sources() {
    let interpreter = HtmlValueInterpreter::new(100);
    let raw = attr("type", "submit");
    let input = interpreter
        .input_type(&element("input"), Some(&raw))
        .unwrap();
    let button = interpreter
        .button_type(&element("button"), Some(&raw))
        .unwrap();
    assert_eq!(input.attribute(), KnownHtmlAttribute::Type);
    assert_eq!(input.inventory_rows(), &[169]);
    assert!(input.source_clause().ends_with("#attr-input-type"));
    assert_eq!(button.inventory_rows(), &[167]);
    assert!(button.source_clause().ends_with("#attr-button-type"));
    for local in ["div", "x-control", "INPUT", "BUTTON", "ol", "script"] {
        assert_eq!(
            interpreter
                .input_type(&element(local), Some(&raw))
                .unwrap()
                .applicability(),
            Applicability::DifferentElement
        );
        assert_eq!(
            interpreter
                .button_type(&element(local), Some(&raw))
                .unwrap()
                .applicability(),
            Applicability::DifferentElement
        );
    }
    assert_eq!(
        interpreter
            .input_type(&element("button"), Some(&raw))
            .unwrap()
            .intrinsic(),
        None
    );
    assert_eq!(
        interpreter
            .button_type(&element("input"), Some(&raw))
            .unwrap()
            .intrinsic(),
        None
    );
    for (namespace, local) in [(Namespace::svg(), "input"), (Namespace::mathml(), "button")] {
        let foreign = QualifiedName::new(namespace, None, local);
        assert_eq!(
            interpreter
                .input_type(&foreign, Some(&raw))
                .unwrap()
                .applicability(),
            Applicability::ForeignElementNamespace
        );
        assert_eq!(
            interpreter
                .button_type(&foreign, Some(&raw))
                .unwrap()
                .applicability(),
            Applicability::ForeignElementNamespace
        );
    }
    for name in [
        QualifiedName::new(Namespace::none(), None, "TYPE"),
        QualifiedName::new(Namespace::html(), None, "type"),
    ] {
        let raw = Attribute::new(name, "submit");
        assert_eq!(
            interpreter
                .input_type(&element("input"), Some(&raw))
                .unwrap()
                .applicability(),
            Applicability::DifferentAttribute
        );
        assert_eq!(
            interpreter
                .button_type(&element("button"), Some(&raw))
                .unwrap()
                .applicability(),
            Applicability::DifferentAttribute
        );
    }
}

#[test]
fn control_type_admission_and_reports_preserve_exact_utf16_and_prefixes() {
    let interpreter = HtmlValueInterpreter::new(2);
    let input = QualifiedName::new(Namespace::html(), Some("h".into()), "input");
    let button = QualifiedName::new(Namespace::html(), Some("h".into()), "button");
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "type"),
        DocumentString::from_utf16(vec![0xd800, 0x0054]),
    );
    let input_report = interpreter.input_type(&input, Some(&raw)).unwrap();
    assert_eq!(input_report.element(), &input);
    assert_eq!(input_report.raw_attribute(), Some(&raw));
    assert_eq!(input_report.input_code_units(), 2);
    assert_eq!(input_report.intrinsic(), Some(&HtmlInputType::Text));
    assert_eq!(input_report.recognition(), Recognition::Unrecognized);
    let button_report = interpreter.button_type(&button, Some(&raw)).unwrap();
    assert_eq!(button_report.element(), &button);
    assert_eq!(button_report.raw_attribute(), Some(&raw));
    assert_eq!(button_report.intrinsic(), Some(&HtmlButtonTypeState::Auto));
    let limited = HtmlValueInterpreter::new(1);
    for error in [
        limited.input_type(&element("div"), Some(&raw)).unwrap_err(),
        limited
            .button_type(&element("div"), Some(&raw))
            .unwrap_err(),
    ] {
        assert_eq!(error.capacity(), 1);
        assert_eq!(error.required(), 2);
    }
    let interpreter = HtmlValueInterpreter::new(100);
    let native = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "type"),
        "SUBMIT",
    );
    assert_eq!(
        interpreter
            .input_type(&input, Some(&native))
            .unwrap()
            .intrinsic(),
        Some(&HtmlInputType::Submit)
    );
    assert_eq!(
        interpreter
            .button_type(&button, Some(&native))
            .unwrap()
            .intrinsic(),
        Some(&HtmlButtonTypeState::Keyword(HtmlButtonType::Submit))
    );
}

#[test]
fn element_keyword_defaults_preserve_missing_empty_and_invalid() {
    let interpreter = HtmlValueInterpreter::new(100);
    // Independent expectations from each defining clause, rather than an
    // index value-summary category or generated rule metadata.
    macro_rules! defaults {
        ($method:ident, $element:literal, $name:literal, $missing:expr, $invalid:expr) => {
            let context = element($element);
            let omitted = interpreter.$method(&context, None).unwrap();
            assert_eq!(omitted.observation(), Observation::Missing);
            assert_eq!(omitted.author(), Author::Missing);
            assert_eq!(omitted.intrinsic(), Some(&$missing));
            for raw in [attr($name, ""), attr($name, "unknown")] {
                let report = interpreter.$method(&context, Some(&raw)).unwrap();
                assert_eq!(report.intrinsic(), Some(&$invalid));
                assert_eq!(report.raw_attribute(), Some(&raw));
                assert_eq!(
                    report.author(),
                    Author::Nonconforming(Conformance::UnrecognizedKeyword)
                );
                assert_eq!(
                    report.observation(),
                    if raw.value().is_empty() {
                        Observation::PresentEmpty
                    } else {
                        Observation::PresentNonempty
                    }
                );
                assert_eq!(
                    report.recognition(),
                    if raw.value().is_empty() {
                        Recognition::Empty
                    } else {
                        Recognition::Unrecognized
                    }
                );
            }
        };
    }
    defaults!(
        closedby,
        "dialog",
        "closedby",
        HtmlClosedByState::Auto,
        HtmlClosedByState::Auto
    );
    defaults!(
        form_autocomplete,
        "form",
        "autocomplete",
        HtmlFormAutocomplete::On,
        HtmlFormAutocomplete::On
    );
    defaults!(
        decoding,
        "img",
        "decoding",
        HtmlDecoding::Auto,
        HtmlDecoding::Auto
    );
    defaults!(
        loading,
        "iframe",
        "loading",
        HtmlLoading::Eager,
        HtmlLoading::Eager
    );
    defaults!(
        form_method,
        "form",
        "method",
        HtmlFormMethod::Get,
        HtmlFormMethod::Get
    );
    defaults!(
        form_enctype,
        "form",
        "enctype",
        HtmlFormEnctype::UrlEncoded,
        HtmlFormEnctype::UrlEncoded
    );
    defaults!(
        track_kind,
        "track",
        "kind",
        HtmlTrackKind::Subtitles,
        HtmlTrackKind::Metadata
    );
    defaults!(
        ol_type,
        "ol",
        "type",
        HtmlOlType::Decimal,
        HtmlOlType::Decimal
    );
}

#[test]
fn preload_has_no_guessed_missing_or_invalid_hint_and_empty_is_automatic() {
    // attr-media-preload does not normatively select the suggested Metadata.
    let interpreter = HtmlValueInterpreter::new(100);
    for local in ["audio", "video"] {
        let context = element(local);
        let missing = interpreter.preload(&context, None).unwrap();
        assert_eq!(
            missing.intrinsic(),
            Some(&HtmlPreloadState::ImplementationDefined)
        );
        assert_eq!(missing.author(), Author::Missing);
        let invalid = attr("preload", "invalid");
        let report = interpreter.preload(&context, Some(&invalid)).unwrap();
        assert_eq!(
            report.intrinsic(),
            Some(&HtmlPreloadState::ImplementationDefined)
        );
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::UnrecognizedKeyword)
        );
        let empty = attr("preload", "");
        let report = interpreter.preload(&context, Some(&empty)).unwrap();
        assert_eq!(
            report.intrinsic(),
            Some(&HtmlPreloadState::Keyword(HtmlPreload::Auto))
        );
        assert_eq!(report.observation(), Observation::PresentEmpty);
        assert_eq!(report.recognition(), Recognition::Empty);
        assert_eq!(report.author(), Author::Conforming);
        assert!(report.source_clause().ends_with("#attr-media-preload"));
    }
}

#[test]
fn element_enumerations_recognize_literal_keywords_without_whitespace_folding() {
    let interpreter = HtmlValueInterpreter::new(100);
    macro_rules! keywords {
        ($method:ident, $element:literal, $name:literal, [$($raw:literal => ($canonical:literal, $expected:expr)),+]) => {
            let context = element($element);
            $(
                let raw = attr($name, $raw);
                let report = interpreter.$method(&context, Some(&raw)).unwrap();
                assert_eq!(report.intrinsic(), Some(&$expected));
                assert_eq!(report.author(), Author::Conforming);
                assert_eq!(report.recognition(), Recognition::Keyword { matched: $canonical, canonical: $canonical });
                assert_eq!(report.raw_attribute(), Some(&raw));
                let spaced = attr($name, concat!(" ", $raw, " "));
                assert_eq!(interpreter.$method(&context, Some(&spaced)).unwrap().recognition(), Recognition::Unrecognized);
            )+
        };
    }
    keywords!(closedby, "dialog", "closedby", ["ANY" => ("any", HtmlClosedByState::Keyword(HtmlClosedBy::Any)), "CLOSEREQUEST" => ("closerequest", HtmlClosedByState::Keyword(HtmlClosedBy::CloseRequest)), "NONE" => ("none", HtmlClosedByState::Keyword(HtmlClosedBy::None))]);
    keywords!(form_autocomplete, "form", "autocomplete", ["ON" => ("on", HtmlFormAutocomplete::On), "OFF" => ("off", HtmlFormAutocomplete::Off)]);
    keywords!(decoding, "img", "decoding", ["SYNC" => ("sync", HtmlDecoding::Sync), "ASYNC" => ("async", HtmlDecoding::Async), "AUTO" => ("auto", HtmlDecoding::Auto)]);
    keywords!(loading, "video", "loading", ["LAZY" => ("lazy", HtmlLoading::Lazy), "EAGER" => ("eager", HtmlLoading::Eager)]);
    keywords!(preload, "audio", "preload", ["AUTO" => ("auto", HtmlPreloadState::Keyword(HtmlPreload::Auto)), "NONE" => ("none", HtmlPreloadState::Keyword(HtmlPreload::None)), "METADATA" => ("metadata", HtmlPreloadState::Keyword(HtmlPreload::Metadata))]);
    keywords!(form_method, "form", "method", ["GET" => ("get", HtmlFormMethod::Get), "POST" => ("post", HtmlFormMethod::Post), "DIALOG" => ("dialog", HtmlFormMethod::Dialog)]);
    keywords!(form_enctype, "form", "enctype", ["APPLICATION/X-WWW-FORM-URLENCODED" => ("application/x-www-form-urlencoded", HtmlFormEnctype::UrlEncoded), "MULTIPART/FORM-DATA" => ("multipart/form-data", HtmlFormEnctype::MultipartFormData), "TEXT/PLAIN" => ("text/plain", HtmlFormEnctype::PlainText)]);
    keywords!(track_kind, "track", "kind", ["SUBTITLES" => ("subtitles", HtmlTrackKind::Subtitles), "CAPTIONS" => ("captions", HtmlTrackKind::Captions), "DESCRIPTIONS" => ("descriptions", HtmlTrackKind::Descriptions), "CHAPTERS" => ("chapters", HtmlTrackKind::Chapters), "METADATA" => ("metadata", HtmlTrackKind::Metadata)]);
}

#[test]
fn ordered_list_marker_characters_are_case_sensitive_and_preserved() {
    let interpreter = HtmlValueInterpreter::new(100);
    let context = element("ol");
    for (literal, expected) in [
        ("1", HtmlOlType::Decimal),
        ("a", HtmlOlType::LowerAlpha),
        ("A", HtmlOlType::UpperAlpha),
        ("i", HtmlOlType::LowerRoman),
        ("I", HtmlOlType::UpperRoman),
    ] {
        let raw = attr("type", literal);
        let report = interpreter.ol_type(&context, Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), Some(&expected));
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(report.raw_attribute(), Some(&raw));
        assert_eq!(expected.into_attribute(), raw);
    }
    for literal in ["01", " a", "A ", "ii", "\u{ff21}", "\u{0131}"] {
        let raw = attr("type", literal);
        let report = interpreter.ol_type(&context, Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), Some(&HtmlOlType::Decimal));
        assert_eq!(report.recognition(), Recognition::Unrecognized);
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::UnrecognizedKeyword)
        );
    }
}

#[test]
fn intrinsic_element_rules_require_exact_html_element_and_attribute_identity() {
    let interpreter = HtmlValueInterpreter::new(100);
    macro_rules! contexts {
        ($method:ident, $name:literal, $raw:literal, [$($local:literal),+], $row:literal, $source:literal) => {
            let raw = attr($name, $raw);
            $(
                let report = interpreter.$method(&element($local), Some(&raw)).unwrap();
                assert_eq!(report.applicability(), Applicability::Applicable);
                assert_eq!(report.inventory_rows(), &[$row]);
                assert!(report.source_clause().ends_with(concat!("#", $source)));
                let upper = interpreter.$method(&element(&$local.to_ascii_uppercase()), Some(&raw)).unwrap();
                assert_eq!(upper.applicability(), Applicability::DifferentElement);
                assert_eq!(upper.intrinsic(), None);
                let foreign = QualifiedName::new(Namespace::svg(), None, $local);
                let report = interpreter.$method(&foreign, Some(&raw)).unwrap();
                assert_eq!(report.applicability(), Applicability::ForeignElementNamespace);
                assert_eq!(report.raw_attribute(), Some(&raw));
            )+
            let unrelated = interpreter.$method(&element("x-thing"), Some(&raw)).unwrap();
            assert_eq!(unrelated.applicability(), Applicability::DifferentElement);
            assert_eq!(unrelated.author(), Author::Unapplied);
            assert_eq!(unrelated.recognition(), Recognition::Unapplied);
            let wrong = attr("unknown", $raw);
            assert_eq!(interpreter.$method(&element("x-thing"), Some(&wrong)).unwrap().applicability(), Applicability::DifferentAttribute);
        };
    }
    contexts!(
        closedby,
        "closedby",
        "any",
        ["dialog"],
        23,
        "attr-dialog-closedby"
    );
    contexts!(
        form_autocomplete,
        "autocomplete",
        "off",
        ["form"],
        13,
        "attr-form-autocomplete"
    );
    contexts!(
        decoding,
        "decoding",
        "sync",
        ["img"],
        38,
        "attr-img-decoding"
    );
    contexts!(
        loading,
        "loading",
        "lazy",
        ["img", "iframe", "audio", "video"],
        89,
        "lazy-loading-attributes"
    );
    contexts!(
        preload,
        "preload",
        "none",
        ["audio", "video"],
        123,
        "attr-media-preload"
    );
    contexts!(
        form_method,
        "method",
        "post",
        ["form"],
        96,
        "attr-fs-method"
    );
    contexts!(
        form_enctype,
        "enctype",
        "text/plain",
        ["form"],
        49,
        "attr-fs-enctype"
    );
    contexts!(
        track_kind,
        "kind",
        "captions",
        ["track"],
        85,
        "attr-track-kind"
    );
    contexts!(ol_type, "type", "A", ["ol"], 170, "attr-ol-type");
    // These similarly named attributes have different owning grammars.
    let control = attr("autocomplete", "off");
    assert_eq!(
        interpreter
            .form_autocomplete(&element("input"), Some(&control))
            .unwrap()
            .applicability(),
        Applicability::DifferentElement
    );
    let script = attr("type", "A");
    assert_eq!(
        interpreter
            .ol_type(&element("script"), Some(&script))
            .unwrap()
            .applicability(),
        Applicability::DifferentElement
    );
}

#[test]
fn element_boolean_rules_are_presence_only_in_their_selected_context() {
    let interpreter = HtmlValueInterpreter::new(100);
    let rules = [
        (
            HtmlBooleanAttribute::AllowFullscreen,
            "allowfullscreen",
            &["iframe"][..],
            7,
            "attr-iframe-allowfullscreen",
        ),
        (
            HtmlBooleanAttribute::Autoplay,
            "autoplay",
            &["audio", "video"][..],
            17,
            "attr-media-autoplay",
        ),
        (
            HtmlBooleanAttribute::MediaControls,
            "controls",
            &["audio", "video"][..],
            32,
            "attr-media-controls",
        ),
        (
            HtmlBooleanAttribute::TrackDefault,
            "default",
            &["track"][..],
            39,
            "attr-track-default",
        ),
        (
            HtmlBooleanAttribute::Loop,
            "loop",
            &["audio", "video"][..],
            90,
            "attr-media-loop",
        ),
        (
            HtmlBooleanAttribute::Muted,
            "muted",
            &["audio", "video"][..],
            101,
            "attr-media-muted",
        ),
        (
            HtmlBooleanAttribute::Novalidate,
            "novalidate",
            &["form"][..],
            111,
            "attr-fs-novalidate",
        ),
        (
            HtmlBooleanAttribute::DetailsOpen,
            "open",
            &["details"][..],
            112,
            "attr-details-open",
        ),
        (
            HtmlBooleanAttribute::DialogOpen,
            "open",
            &["dialog"][..],
            113,
            "attr-dialog-open",
        ),
        (
            HtmlBooleanAttribute::PlaysInline,
            "playsinline",
            &["video"][..],
            118,
            "attr-video-playsinline",
        ),
        (
            HtmlBooleanAttribute::Reversed,
            "reversed",
            &["ol"][..],
            130,
            "attr-ol-reversed",
        ),
        (
            HtmlBooleanAttribute::Selected,
            "selected",
            &["option"][..],
            135,
            "attr-option-selected",
        ),
    ];
    for (selected, name, elements, row, source) in rules {
        assert_eq!(selected.into_attribute(true), Some(attr(name, "")));
        assert_eq!(selected.into_attribute(false), None);
        for local in elements {
            let context = element(local);
            let omitted = interpreter
                .boolean_attribute(selected, &context, None)
                .unwrap();
            assert_eq!(omitted.intrinsic(), Some(&false));
            assert_eq!(omitted.author(), Author::Missing);
            for (literal, recognition, author) in [
                ("", Recognition::Empty, Author::Conforming),
                (
                    "false",
                    Recognition::Unrecognized,
                    Author::Nonconforming(Conformance::BooleanSpelling),
                ),
            ] {
                let raw = attr(name, literal);
                let report = interpreter
                    .boolean_attribute(selected, &context, Some(&raw))
                    .unwrap();
                assert_eq!(report.intrinsic(), Some(&true));
                assert_eq!(report.author(), author);
                assert_eq!(report.recognition(), recognition);
                assert_eq!(report.raw_attribute(), Some(&raw));
                assert_eq!(report.inventory_rows(), &[row]);
                assert!(report.source_clause().ends_with(&format!("#{source}")));
            }
            let uppercase = attr(name, &name.to_ascii_uppercase());
            assert_eq!(
                interpreter
                    .boolean_attribute(selected, &context, Some(&uppercase))
                    .unwrap()
                    .author(),
                Author::Conforming
            );
            assert_eq!(
                interpreter
                    .boolean_attribute(
                        selected,
                        &element(&local.to_ascii_uppercase()),
                        Some(&uppercase)
                    )
                    .unwrap()
                    .applicability(),
                Applicability::DifferentElement
            );
        }
        assert_eq!(
            interpreter
                .boolean_attribute(selected, &element("div"), None)
                .unwrap()
                .intrinsic(),
            None
        );
    }
    // The selected media rule does not guess img's required nonempty alt.
    let raw = attr("controls", "");
    assert_eq!(
        interpreter
            .boolean_attribute(
                HtmlBooleanAttribute::MediaControls,
                &element("img"),
                Some(&raw)
            )
            .unwrap()
            .applicability(),
        Applicability::DifferentElement
    );
    assert_eq!(
        interpreter
            .boolean_attribute(HtmlBooleanAttribute::DialogOpen, &element("details"), None)
            .unwrap()
            .applicability(),
        Applicability::DifferentElement
    );
}

#[test]
fn element_native_keywords_author_only_real_spelling_and_not_fallback_states() {
    for (native, expected) in [
        (HtmlClosedBy::Any, "any"),
        (HtmlClosedBy::CloseRequest, "closerequest"),
        (HtmlClosedBy::None, "none"),
    ] {
        assert_eq!(native.into_attribute(), attr("closedby", expected));
    }
    for (native, expected) in [
        (HtmlPreload::Auto, "auto"),
        (HtmlPreload::None, "none"),
        (HtmlPreload::Metadata, "metadata"),
    ] {
        assert_eq!(native.into_attribute(), attr("preload", expected));
    }
    assert_eq!(
        HtmlFormAutocomplete::Off.into_attribute(),
        attr("autocomplete", "off")
    );
    assert_eq!(
        HtmlDecoding::Async.into_attribute(),
        attr("decoding", "async")
    );
    assert_eq!(HtmlLoading::Lazy.into_attribute(), attr("loading", "lazy"));
    assert_eq!(
        HtmlFormMethod::Dialog.into_attribute(),
        attr("method", "dialog")
    );
    assert_eq!(
        HtmlFormEnctype::MultipartFormData.into_attribute(),
        attr("enctype", "multipart/form-data")
    );
    assert_eq!(
        HtmlTrackKind::Descriptions.into_attribute(),
        attr("kind", "descriptions")
    );
}

#[test]
fn element_rules_admit_units_before_context_checks_and_preserve_raw_prefixes() {
    let context = QualifiedName::new(Namespace::html(), Some("h".into()), "ol");
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "type"),
        DocumentString::from_utf16(vec![0xd800, 0x0041]),
    );
    let interpreter = HtmlValueInterpreter::new(2);
    let report = interpreter.ol_type(&context, Some(&raw)).unwrap();
    assert_eq!(report.element(), &context);
    assert_eq!(report.raw_attribute(), Some(&raw));
    assert_eq!(report.input_code_units(), 2);
    assert_eq!(report.intrinsic(), Some(&HtmlOlType::Decimal));
    assert_eq!(
        report.author(),
        Author::Nonconforming(Conformance::UnrecognizedKeyword)
    );
    let small = HtmlValueInterpreter::new(1);
    let wrong_context = element("div");
    let error = small.ol_type(&wrong_context, Some(&raw)).unwrap_err();
    assert_eq!(error.capacity(), 1);
    assert_eq!(error.required(), 2);
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "type"),
        "A",
    );
    assert_eq!(
        interpreter
            .ol_type(&context, Some(&raw))
            .unwrap()
            .intrinsic(),
        Some(&HtmlOlType::UpperAlpha)
    );
}

#[test]
fn boolean_false_text_is_nonconforming_but_presence_is_true() {
    // HTML boolean-attributes: authored false is invalid, not Boolean false.
    let interpreter = HtmlValueInterpreter::new(100);
    let element = element("div");
    let false_text = attr("inert", "false");
    let result = interpreter
        .boolean_attribute(HtmlBooleanAttribute::Inert, &element, Some(&false_text))
        .unwrap();
    assert_eq!(result.intrinsic(), Some(&true));
    assert_eq!(
        result.author(),
        Author::Nonconforming(Conformance::BooleanSpelling)
    );
    assert_eq!(result.observation(), Observation::PresentNonempty);
    assert_eq!(result.raw_attribute(), Some(&false_text));
    let omitted = interpreter
        .boolean_attribute(HtmlBooleanAttribute::Inert, &element, None)
        .unwrap();
    assert_eq!(omitted.intrinsic(), Some(&false));
    assert_eq!(omitted.observation(), Observation::Missing);
    let empty = attr("inert", "");
    let result = interpreter
        .boolean_attribute(HtmlBooleanAttribute::Inert, &element, Some(&empty))
        .unwrap();
    assert_eq!(result.intrinsic(), Some(&true));
    assert_eq!(result.author(), Author::Conforming);
    assert_eq!(result.observation(), Observation::PresentEmpty);
    let canonical_name = attr("inert", "INERT");
    assert_eq!(
        interpreter
            .boolean_attribute(HtmlBooleanAttribute::Inert, &element, Some(&canonical_name))
            .unwrap()
            .author(),
        Author::Conforming
    );
    let spaced = attr("inert", " inert ");
    assert_eq!(
        interpreter
            .boolean_attribute(HtmlBooleanAttribute::Inert, &element, Some(&spaced))
            .unwrap()
            .author(),
        Author::Nonconforming(Conformance::BooleanSpelling)
    );
}

#[test]
fn global_boolean_flags_apply_to_unknown_html_elements_and_template_flags_do_not() {
    let interpreter = HtmlValueInterpreter::new(100);
    let custom = element("x-thing");
    for (selected, name) in [
        (HtmlBooleanAttribute::Autofocus, "autofocus"),
        (HtmlBooleanAttribute::Inert, "inert"),
        (HtmlBooleanAttribute::Itemscope, "itemscope"),
        (HtmlBooleanAttribute::HeadingReset, "headingreset"),
    ] {
        let raw = attr(name, "");
        let report = interpreter
            .boolean_attribute(selected, &custom, Some(&raw))
            .unwrap();
        assert_eq!(report.applicability(), Applicability::Applicable);
        assert_eq!(report.intrinsic(), Some(&true));
        assert_eq!(selected.into_attribute(true), Some(raw));
        assert_eq!(selected.into_attribute(false), None);
    }
    for (selected, name) in [
        (
            HtmlBooleanAttribute::ShadowRootClonable,
            "shadowrootclonable",
        ),
        (
            HtmlBooleanAttribute::ShadowRootCustomElementRegistry,
            "shadowrootcustomelementregistry",
        ),
        (
            HtmlBooleanAttribute::ShadowRootDelegatesFocus,
            "shadowrootdelegatesfocus",
        ),
        (
            HtmlBooleanAttribute::ShadowRootSerializable,
            "shadowrootserializable",
        ),
    ] {
        let raw = attr(name, "false");
        let report = interpreter
            .boolean_attribute(selected, &element("template"), Some(&raw))
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&true));
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::BooleanSpelling)
        );
        let report = interpreter
            .boolean_attribute(selected, &custom, Some(&raw))
            .unwrap();
        assert_eq!(report.applicability(), Applicability::DifferentElement);
        assert_eq!(report.author(), Author::Unapplied);
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn enum_empty_defaults_are_specific_and_do_not_erase_empty_observation() {
    let i = HtmlValueInterpreter::new(100);
    let e = element("div");
    macro_rules! empty {
        ($method:ident, $name:literal, $state:expr, $author:expr) => {{
            let raw = attr($name, "");
            let report = i.$method(&e, Some(&raw)).unwrap();
            assert_eq!(report.intrinsic(), Some(&$state));
            assert_eq!(report.author(), $author);
            assert_eq!(report.observation(), Observation::PresentEmpty);
            assert_eq!(report.recognition(), Recognition::Empty);
            assert_eq!(report.raw_attribute(), Some(&raw));
        }};
    }
    empty!(
        dir,
        "dir",
        HtmlDirState::Undefined,
        Author::Nonconforming(Conformance::UnrecognizedKeyword)
    );
    empty!(
        draggable,
        "draggable",
        HtmlDraggableState::Auto,
        Author::Nonconforming(Conformance::UnrecognizedKeyword)
    );
    empty!(
        contenteditable,
        "contenteditable",
        HtmlContentEditableState::Keyword(HtmlContentEditable::True),
        Author::Conforming
    );
    empty!(
        hidden,
        "hidden",
        HtmlHiddenState::Keyword(HtmlHidden::Hidden),
        Author::Conforming
    );
    empty!(
        popover,
        "popover",
        HtmlPopoverState::Keyword(HtmlPopover::Auto),
        Author::Conforming
    );
    empty!(
        translate,
        "translate",
        HtmlTranslateState::Keyword(HtmlTranslate::Yes),
        Author::Conforming
    );
    empty!(
        spellcheck,
        "spellcheck",
        HtmlSpellcheckState::Keyword(HtmlSpellcheck::True),
        Author::Conforming
    );
    empty!(
        autocorrect,
        "autocorrect",
        HtmlAutocorrect::On,
        Author::Conforming
    );
    empty!(
        writingsuggestions,
        "writingsuggestions",
        HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::True),
        Author::Conforming
    );
    empty!(
        autocapitalize,
        "autocapitalize",
        HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Sentences),
        Author::Nonconforming(Conformance::UnrecognizedKeyword)
    );
}

#[test]
fn missing_and_unknown_keywords_keep_different_default_and_author_states() {
    let i = HtmlValueInterpreter::new(100);
    let e = element("div");
    macro_rules! defaults {
        ($method:ident, $name:literal, $missing:expr, $invalid:expr) => {{
            let report = i.$method(&e, None).unwrap();
            assert_eq!(report.intrinsic(), Some(&$missing));
            assert_eq!(report.author(), Author::Missing);
            assert_eq!(report.recognition(), Recognition::Missing);
            let raw = attr($name, "unknown");
            let report = i.$method(&e, Some(&raw)).unwrap();
            assert_eq!(report.intrinsic(), Some(&$invalid));
            assert_eq!(
                report.author(),
                Author::Nonconforming(Conformance::UnrecognizedKeyword)
            );
            assert_eq!(report.recognition(), Recognition::Unrecognized);
            assert_eq!(report.raw_attribute(), Some(&raw));
        }};
    }
    defaults!(dir, "dir", HtmlDirState::Undefined, HtmlDirState::Undefined);
    defaults!(
        contenteditable,
        "contenteditable",
        HtmlContentEditableState::Inherit,
        HtmlContentEditableState::Inherit
    );
    defaults!(
        draggable,
        "draggable",
        HtmlDraggableState::Auto,
        HtmlDraggableState::Auto
    );
    defaults!(
        hidden,
        "hidden",
        HtmlHiddenState::NotHidden,
        HtmlHiddenState::Keyword(HtmlHidden::Hidden)
    );
    defaults!(
        popover,
        "popover",
        HtmlPopoverState::NoPopover,
        HtmlPopoverState::Keyword(HtmlPopover::Manual)
    );
    defaults!(
        translate,
        "translate",
        HtmlTranslateState::Inherit,
        HtmlTranslateState::Inherit
    );
    defaults!(
        spellcheck,
        "spellcheck",
        HtmlSpellcheckState::Default,
        HtmlSpellcheckState::Default
    );
    defaults!(
        autocapitalize,
        "autocapitalize",
        HtmlAutocapitalizeState::Default,
        HtmlAutocapitalizeState::Keyword(HtmlAutocapitalize::Sentences)
    );
    defaults!(
        autocorrect,
        "autocorrect",
        HtmlAutocorrect::On,
        HtmlAutocorrect::On
    );
    defaults!(
        writingsuggestions,
        "writingsuggestions",
        HtmlWritingSuggestionsState::Default,
        HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::True)
    );
}

#[test]
fn autocapitalize_aliases_are_recognized_and_canonical_authoring_is_explicit() {
    // attr-autocapitalize getter explicitly returns none/sentences for aliases.
    let i = HtmlValueInterpreter::new(100);
    for (raw_value, matched, canonical, state) in [
        ("OFF", "off", "none", HtmlAutocapitalize::None),
        ("none", "none", "none", HtmlAutocapitalize::None),
        ("ON", "on", "sentences", HtmlAutocapitalize::Sentences),
        (
            "sentences",
            "sentences",
            "sentences",
            HtmlAutocapitalize::Sentences,
        ),
        ("words", "words", "words", HtmlAutocapitalize::Words),
        (
            "characters",
            "characters",
            "characters",
            HtmlAutocapitalize::Characters,
        ),
    ] {
        let raw = attr("autocapitalize", raw_value);
        let report = i.autocapitalize(&element("input"), Some(&raw)).unwrap();
        assert_eq!(
            report.intrinsic(),
            Some(&HtmlAutocapitalizeState::Keyword(state))
        );
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(
            report.recognition(),
            Recognition::Keyword { matched, canonical }
        );
        assert_eq!(state.into_attribute(), attr("autocapitalize", canonical));
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn bdo_required_direction_conformance_is_independent_of_intrinsic_state() {
    let i = HtmlValueInterpreter::new(100);
    let bdo = element("bdo");
    let missing = i.dir(&bdo, None).unwrap();
    assert_eq!(missing.intrinsic(), Some(&HtmlDirState::Undefined));
    assert_eq!(
        missing.author(),
        Author::Nonconforming(Conformance::BdoDirectionRequired)
    );
    assert_eq!(
        missing.context_source_clause(),
        Some(
            "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-bdo-element"
        )
    );
    let auto = attr("dir", "auto");
    let report = i.dir(&bdo, Some(&auto)).unwrap();
    assert_eq!(
        report.intrinsic(),
        Some(&HtmlDirState::Keyword(HtmlDir::Auto))
    );
    assert_eq!(
        report.recognition(),
        Recognition::Keyword {
            matched: "auto",
            canonical: "auto"
        }
    );
    assert_eq!(
        report.author(),
        Author::Nonconforming(Conformance::BdoDirectionMustBeLtrOrRtl)
    );
    for value in ["ltr", "RTL"] {
        let raw = attr("dir", value);
        assert_eq!(
            i.dir(&bdo, Some(&raw)).unwrap().author(),
            Author::Conforming
        );
    }
    assert_eq!(
        i.dir(&element("div"), Some(&auto)).unwrap().author(),
        Author::Conforming
    );
    // Exact uppercase local name is an unknown HTML name, not the bdo identity.
    assert_eq!(
        i.dir(&element("BDO"), None).unwrap().author(),
        Author::Missing
    );
}

#[test]
fn keyboard_hints_have_no_guessed_missing_invalid_or_empty_default() {
    let i = HtmlValueInterpreter::new(100);
    let e = element("input");
    assert_eq!(i.inputmode(&e, None).unwrap().intrinsic(), None);
    assert_eq!(i.enterkeyhint(&e, None).unwrap().intrinsic(), None);
    for value in ["", "unknown", " text ", "ｔｅｘｔ"] {
        let raw = attr("inputmode", value);
        let report = i.inputmode(&e, Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), None);
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::UnrecognizedKeyword)
        );
    }
    for value in ["", "unknown", " enter "] {
        let raw = attr("enterkeyhint", value);
        assert_eq!(i.enterkeyhint(&e, Some(&raw)).unwrap().intrinsic(), None);
    }
    assert_eq!(
        i.inputmode(&e, Some(&attr("inputmode", "DECIMAL")))
            .unwrap()
            .intrinsic(),
        Some(&HtmlInputMode::Decimal)
    );
    assert_eq!(
        i.enterkeyhint(&e, Some(&attr("enterkeyhint", "NEXT")))
            .unwrap()
            .intrinsic(),
        Some(&HtmlEnterKeyHint::Next)
    );
}

#[test]
fn keyboard_keyword_tables_select_their_own_concrete_hints() {
    let i = HtmlValueInterpreter::new(100);
    let e = element("input");
    for (value, expected) in [
        ("none", HtmlInputMode::None),
        ("text", HtmlInputMode::Text),
        ("tel", HtmlInputMode::Tel),
        ("url", HtmlInputMode::Url),
        ("email", HtmlInputMode::Email),
        ("numeric", HtmlInputMode::Numeric),
        ("decimal", HtmlInputMode::Decimal),
        ("search", HtmlInputMode::Search),
    ] {
        let raw = attr("inputmode", value);
        let report = i.inputmode(&e, Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), Some(&expected));
        assert_eq!(report.author(), Author::Conforming);
    }
    for (value, expected) in [
        ("enter", HtmlEnterKeyHint::Enter),
        ("done", HtmlEnterKeyHint::Done),
        ("go", HtmlEnterKeyHint::Go),
        ("next", HtmlEnterKeyHint::Next),
        ("previous", HtmlEnterKeyHint::Previous),
        ("search", HtmlEnterKeyHint::Search),
        ("send", HtmlEnterKeyHint::Send),
    ] {
        let raw = attr("enterkeyhint", value);
        let report = i.enterkeyhint(&e, Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), Some(&expected));
        assert_eq!(report.author(), Author::Conforming);
    }
}

#[test]
fn foreign_and_mismatched_expanded_names_are_unapplied_and_exactly_retained() {
    let i = HtmlValueInterpreter::new(100);
    let foreign_element = QualifiedName::new(Namespace::svg(), Some("s".into()), "div");
    let raw = attr("hidden", "until-found");
    let report = i.hidden(&foreign_element, Some(&raw)).unwrap();
    assert_eq!(
        report.applicability(),
        Applicability::ForeignElementNamespace
    );
    assert_eq!(report.author(), Author::Unapplied);
    assert_eq!(report.recognition(), Recognition::Unapplied);
    assert_eq!(report.intrinsic(), None);
    assert_eq!(report.element(), &foreign_element);
    assert_eq!(report.raw_attribute(), Some(&raw));
    for name in [
        QualifiedName::new(Namespace::none(), None, "HIDDEN"),
        QualifiedName::new(Namespace::svg(), Some("s".into()), "hidden"),
        QualifiedName::new(
            Namespace::none(),
            None,
            DocumentString::from_utf16(vec![0xd800]),
        ),
    ] {
        let raw = Attribute::new(name, "false");
        let report = i.hidden(&element("div"), Some(&raw)).unwrap();
        assert_eq!(report.applicability(), Applicability::DifferentAttribute);
        assert_eq!(report.author(), Author::Unapplied);
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn prefixes_do_not_change_expanded_identity_and_raw_surrogates_are_not_folded() {
    let i = HtmlValueInterpreter::new(100);
    let e = QualifiedName::new(
        Namespace::html(),
        Some(DocumentString::from_utf16(vec![0xd800])),
        "x-widget",
    );
    let name = QualifiedName::new(Namespace::none(), Some("p".into()), "popover");
    let raw = Attribute::new(name, DocumentString::from_utf16(vec![0xd800, 0x20, 0xdc00]));
    let report = i.popover(&e, Some(&raw)).unwrap();
    assert_eq!(report.applicability(), Applicability::Applicable);
    assert_eq!(
        report.intrinsic(),
        Some(&HtmlPopoverState::Keyword(HtmlPopover::Manual))
    );
    assert_eq!(report.recognition(), Recognition::Unrecognized);
    assert_eq!(report.raw_attribute(), Some(&raw));
    assert_eq!(report.element(), &e);
    assert_eq!(report.raw().unwrap().code_units(), &[0xd800, 0x20, 0xdc00]);
}

#[test]
fn raw_value_admission_precedes_even_unapplied_name_processing() {
    let i = HtmlValueInterpreter::new(0);
    let foreign = QualifiedName::new(Namespace::svg(), None, "thing");
    let raw = attr("other", "x");
    let error = i.hidden(&foreign, Some(&raw)).unwrap_err();
    assert_eq!(error.required(), 1);
    assert_eq!(error.capacity(), 0);
    assert_eq!(
        i.hidden(&element("div"), None).unwrap().intrinsic(),
        Some(&HtmlHiddenState::NotHidden)
    );
    let raw = attr("hidden", "");
    assert_eq!(
        i.hidden(&element("div"), Some(&raw)).unwrap().intrinsic(),
        Some(&HtmlHiddenState::Keyword(HtmlHidden::Hidden))
    );
    let raw = Attribute::new(
        KnownHtmlAttribute::Hidden.qualified_name(),
        DocumentString::from_utf16(vec![0xd83d, 0xde00]),
    );
    assert_eq!(
        HtmlValueInterpreter::new(1)
            .hidden(&element("div"), Some(&raw))
            .unwrap_err()
            .required(),
        2
    );
}

#[test]
fn declarative_modes_are_template_only_and_do_not_claim_root_creation() {
    let i = HtmlValueInterpreter::new(100);
    let template = KnownHtmlElement::Template.qualified_name();
    assert_eq!(i.shadowrootmode(&template, None).unwrap().intrinsic(), None);
    let bad = attr("shadowrootmode", "");
    let report = i.shadowrootmode(&template, Some(&bad)).unwrap();
    assert_eq!(report.intrinsic(), None);
    assert_eq!(
        report.author(),
        Author::Nonconforming(Conformance::UnrecognizedKeyword)
    );
    for (value, expected) in [
        ("OPEN", HtmlShadowRootMode::Open),
        ("closed", HtmlShadowRootMode::Closed),
    ] {
        let raw = attr("shadowrootmode", value);
        assert_eq!(
            i.shadowrootmode(&template, Some(&raw)).unwrap().intrinsic(),
            Some(&expected)
        );
        let report = i.shadowrootmode(&element("div"), Some(&raw)).unwrap();
        assert_eq!(report.applicability(), Applicability::DifferentElement);
        assert_eq!(report.intrinsic(), None);
    }
    assert_eq!(
        HtmlShadowRootMode::Open.into_shadowrootmode_attribute(),
        attr("shadowrootmode", "open")
    );
    assert_eq!(
        i.shadowrootslotassignment(&template, None)
            .unwrap()
            .intrinsic(),
        Some(&HtmlSlotAssignment::Named)
    );
    let invalid = attr("shadowrootslotassignment", "wrong");
    assert_eq!(
        i.shadowrootslotassignment(&template, Some(&invalid))
            .unwrap()
            .intrinsic(),
        Some(&HtmlSlotAssignment::Named)
    );
    let manual = HtmlSlotAssignment::Manual.into_shadowrootslotassignment_attribute();
    assert_eq!(manual, attr("shadowrootslotassignment", "manual"));
    assert_eq!(
        i.shadowrootslotassignment(&template, Some(&manual))
            .unwrap()
            .intrinsic(),
        Some(&HtmlSlotAssignment::Manual)
    );
}

#[test]
fn native_global_keywords_construct_real_attributes_and_typed_states() {
    let i = HtmlValueInterpreter::new(100);
    let e = element("div");
    macro_rules! native {
        ($method:ident, $name:literal, $keyword:literal, $native:expr, $state:expr) => {{
            let raw = $native.into_attribute();
            assert_eq!(raw, attr($name, $keyword));
            let report = i.$method(&e, Some(&raw)).unwrap();
            assert_eq!(report.intrinsic(), Some(&$state));
            assert_eq!(report.author(), Author::Conforming);
        }};
    }
    native!(
        dir,
        "dir",
        "rtl",
        HtmlDir::Rtl,
        HtmlDirState::Keyword(HtmlDir::Rtl)
    );
    native!(
        contenteditable,
        "contenteditable",
        "plaintext-only",
        HtmlContentEditable::PlaintextOnly,
        HtmlContentEditableState::Keyword(HtmlContentEditable::PlaintextOnly)
    );
    native!(
        draggable,
        "draggable",
        "false",
        HtmlDraggable::False,
        HtmlDraggableState::Keyword(HtmlDraggable::False)
    );
    native!(
        hidden,
        "hidden",
        "until-found",
        HtmlHidden::UntilFound,
        HtmlHiddenState::Keyword(HtmlHidden::UntilFound)
    );
    native!(
        popover,
        "popover",
        "hint",
        HtmlPopover::Hint,
        HtmlPopoverState::Keyword(HtmlPopover::Hint)
    );
    native!(
        translate,
        "translate",
        "no",
        HtmlTranslate::No,
        HtmlTranslateState::Keyword(HtmlTranslate::No)
    );
    native!(
        spellcheck,
        "spellcheck",
        "false",
        HtmlSpellcheck::False,
        HtmlSpellcheckState::Keyword(HtmlSpellcheck::False)
    );
    native!(
        autocorrect,
        "autocorrect",
        "off",
        HtmlAutocorrect::Off,
        HtmlAutocorrect::Off
    );
    native!(
        writingsuggestions,
        "writingsuggestions",
        "false",
        HtmlWritingSuggestions::False,
        HtmlWritingSuggestionsState::Keyword(HtmlWritingSuggestions::False)
    );
    native!(
        inputmode,
        "inputmode",
        "numeric",
        HtmlInputMode::Numeric,
        HtmlInputMode::Numeric
    );
    native!(
        enterkeyhint,
        "enterkeyhint",
        "send",
        HtmlEnterKeyHint::Send,
        HtmlEnterKeyHint::Send
    );
}

#[test]
fn metadata_selects_actual_defining_headingreset_clause_and_existing_rows() {
    // The retained non-normative index incorrectly links headingreset to tabindex.
    let report = HtmlValueInterpreter::new(0)
        .boolean_attribute(HtmlBooleanAttribute::HeadingReset, &element("h1"), None)
        .unwrap();
    assert_eq!(report.attribute(), KnownHtmlAttribute::Headingreset);
    assert_eq!(report.inventory_rows(), &[63]);
    assert_eq!(
        report.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-headingreset"
    );
    assert_eq!(report.intrinsic(), Some(&false));
}
