use crate::html::support::html_element as element;
use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability as Applicability,
    HtmlAttributeAuthorState as Author, HtmlAttributeConformanceError as Conformance,
    HtmlAttributeContextAuthorState as ContextAuthor,
    HtmlAttributeContextConformanceError as ContextConformance,
    HtmlAttributeObservation as Observation, HtmlAttributeRecognition as Recognition,
    HtmlBooleanAttribute as Boolean, HtmlButtonSubmitterContext, HtmlCustomControlAttributeContext,
    HtmlHyperlinkAttributeContext as Hyperlink, HtmlImageControlsAttributeContext as ImageControls,
    HtmlImageMapAttributeContext as ImageMap, HtmlInputType as Input, HtmlReferrerPolicy,
    HtmlScriptAttributeContext as Script, HtmlValueInterpreter, KnownHtmlAttribute, Namespace,
    QualifiedName,
};

#[test]
fn image_map_presence_survives_a_missing_qualifying_ancestor_and_invalid_spelling() {
    // #attr-img-ismap makes presence Boolean independently of its author-only
    // ancestor a[href] requirement. An empty href still qualifies.
    let interpreter = HtmlValueInterpreter::new(100);
    let raw = Attribute::new(KnownHtmlAttribute::Ismap.qualified_name(), "false");
    for (context, expected) in [
        (ImageMap::AncestorAnchorWithHref, ContextAuthor::Conforming),
        (
            ImageMap::NoAncestorAnchorWithHref,
            ContextAuthor::Nonconforming(
                ContextConformance::ImageMapRequiresAncestorAnchorWithHref,
            ),
        ),
        (ImageMap::Unavailable, ContextAuthor::Unavailable),
    ] {
        let report = interpreter
            .ismap_with_context(&element("img"), Some(&raw), context)
            .unwrap();
        assert_eq!(report.applicability(), Applicability::Applicable);
        assert_eq!(report.intrinsic(), Some(&true));
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::BooleanSpelling)
        );
        assert_eq!(report.context_author(), expected);
        assert_eq!(report.image_map_context(), Some(context));
        assert_eq!(report.raw_attribute(), Some(&raw));
        assert_eq!(report.inventory_rows(), &[79]);
        assert!(report.source_clause().ends_with("#attr-img-ismap"));
        assert_eq!(report.context_source_clause(), Some(report.source_clause()));
        let omitted = interpreter
            .ismap_with_context(&element("img"), None::<&surgeist_html::Attribute>, context)
            .unwrap();
        assert_eq!(omitted.intrinsic(), Some(&false));
        assert_eq!(omitted.author(), Author::Missing);
        assert_eq!(omitted.context_author(), ContextAuthor::NotRequired);
        assert_eq!(omitted.image_map_context(), Some(context));
    }
}

#[test]
fn image_controls_presence_survives_missing_or_empty_alt() {
    // #attr-img-controls prohibits authored presence without nonempty alt;
    // it does not make alt an intrinsic presence or reflection gate.
    let interpreter = HtmlValueInterpreter::new(100);
    let raw = Attribute::new(KnownHtmlAttribute::Controls.qualified_name(), "CoNtRoLs");
    for (context, expected) in [
        (ImageControls::AltNonempty, ContextAuthor::Conforming),
        (
            ImageControls::AltMissing,
            ContextAuthor::Nonconforming(ContextConformance::ImageControlsRequireNonemptyAlt),
        ),
        (
            ImageControls::AltEmpty,
            ContextAuthor::Nonconforming(ContextConformance::ImageControlsRequireNonemptyAlt),
        ),
        (ImageControls::Unavailable, ContextAuthor::Unavailable),
    ] {
        let report = interpreter
            .image_controls_with_context(&element("img"), Some(&raw), context)
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&true));
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(report.recognition(), Recognition::BooleanName);
        assert_eq!(report.context_author(), expected);
        assert_eq!(report.image_controls_context(), Some(context));
        assert_eq!(report.raw_attribute(), Some(&raw));
        assert_eq!(report.inventory_rows(), &[32]);
        assert!(report.source_clause().ends_with("#attr-img-controls"));
        let missing = interpreter
            .image_controls_with_context(
                &element("img"),
                None::<&surgeist_html::Attribute>,
                context,
            )
            .unwrap();
        assert_eq!(missing.intrinsic(), Some(&false));
        assert_eq!(missing.context_author(), ContextAuthor::NotRequired);
    }
    let invalid = Attribute::new(KnownHtmlAttribute::Controls.qualified_name(), "false");
    let report = interpreter
        .image_controls_with_context(&element("img"), Some(&invalid), ImageControls::AltEmpty)
        .unwrap();
    assert_eq!(report.intrinsic(), Some(&true));
    assert_eq!(
        report.author(),
        Author::Nonconforming(Conformance::BooleanSpelling)
    );
    assert_eq!(
        report.context_author(),
        ContextAuthor::Nonconforming(ContextConformance::ImageControlsRequireNonemptyAlt)
    );
}

#[test]
fn stylesheet_link_context_uses_ascii_space_tokens_and_retains_exact_rel() {
    // #attr-link-disabled selects stylesheet token membership, not equality
    // with the whole rel value. Non-ASCII space and comma are not separators.
    let interpreter = HtmlValueInterpreter::new(100);
    for (text, stylesheet) in [
        ("stylesheet", true),
        ("alternate\tStYlEsHeEt\npreload", true),
        ("\r\u{c}stylesheet stylesheet ", true),
        ("", false),
        ("alternate", false),
        ("stylesheet,preload", false),
        ("stylesheet\u{a0}alternate", false),
        ("\u{17f}tylesheet", false),
    ] {
        let raw = DocumentString::from(text);
        let context = interpreter.stylesheet_link_context(Some(&raw)).unwrap();
        assert_eq!(context.rel(), Some(&raw));
        assert_eq!(context.contains_stylesheet(), stylesheet);
        let disabled = Attribute::new(KnownHtmlAttribute::Disabled.qualified_name(), "false");
        let report = interpreter
            .stylesheet_disabled(&element("link"), Some(&disabled), &context)
            .unwrap();
        assert_eq!(report.applicability(), Applicability::Applicable);
        assert_eq!(report.intrinsic(), Some(&true));
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::BooleanSpelling)
        );
        assert_eq!(
            report.context_author(),
            if stylesheet {
                ContextAuthor::Conforming
            } else {
                ContextAuthor::Nonconforming(
                    ContextConformance::StylesheetDisabledRequiresStylesheetRel,
                )
            }
        );
        assert_eq!(report.stylesheet_link_context(), Some(&context));
        assert_eq!(report.raw_attribute(), Some(&disabled));
        assert_eq!(report.inventory_rows(), &[46]);
        assert!(report.source_clause().ends_with("#attr-link-disabled"));
        let absent = interpreter
            .stylesheet_disabled(
                &element("link"),
                None::<&surgeist_html::Attribute>,
                &context,
            )
            .unwrap();
        assert_eq!(absent.intrinsic(), Some(&false));
        assert_eq!(absent.context_author(), ContextAuthor::NotRequired);
    }
    let missing = interpreter.stylesheet_link_context(None).unwrap();
    assert_eq!(missing.rel(), None);
    assert!(!missing.contains_stylesheet());
    let empty = interpreter
        .stylesheet_link_context(Some(&DocumentString::from("")))
        .unwrap();
    assert_ne!(missing, empty);
}

#[test]
fn content_only_image_and_link_rules_leave_present_author_context_unchecked() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (selected, local) in [
        (Boolean::Ismap, "img"),
        (Boolean::ImageControls, "img"),
        (Boolean::StylesheetDisabled, "link"),
    ] {
        let raw = selected.into_attribute(true).unwrap();
        let present = interpreter
            .boolean_attribute(selected, &element(local), Some(&raw))
            .unwrap();
        assert_eq!(present.intrinsic(), Some(&true));
        assert_eq!(present.author(), Author::Conforming);
        assert_eq!(present.context_author(), ContextAuthor::Unchecked);
        let absent = interpreter
            .boolean_attribute(selected, &element(local), None::<&surgeist_html::Attribute>)
            .unwrap();
        assert_eq!(absent.intrinsic(), Some(&false));
        assert_eq!(absent.context_author(), ContextAuthor::NotRequired);
        assert!(selected.into_attribute(false).is_none());
    }
}

#[test]
fn image_and_link_contexts_do_not_apply_to_other_roles_or_namespaces() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (selected, correct, wrong) in [
        (Boolean::Ismap, "img", "input"),
        (Boolean::ImageControls, "img", "video"),
        (Boolean::StylesheetDisabled, "link", "button"),
    ] {
        let raw = selected.into_attribute(true).unwrap();
        for name in [element(wrong), element(&correct.to_uppercase())] {
            let report = interpreter
                .boolean_attribute(selected, &name, Some(&raw))
                .unwrap();
            assert_eq!(report.applicability(), Applicability::DifferentElement);
            assert_eq!(report.context_author(), ContextAuthor::Unapplied);
            assert_eq!(report.intrinsic(), None);
            assert_eq!(report.raw_attribute(), Some(&raw));
        }
        for namespace in [Namespace::none(), Namespace::svg(), Namespace::mathml()] {
            let name = QualifiedName::new(namespace, None, correct);
            let report = interpreter
                .boolean_attribute(selected, &name, Some(&raw))
                .unwrap();
            assert_eq!(
                report.applicability(),
                Applicability::ForeignElementNamespace
            );
            assert_eq!(report.context_author(), ContextAuthor::Unapplied);
            assert_eq!(report.intrinsic(), None);
        }
        let mismatched = Attribute::new(
            QualifiedName::new(Namespace::svg(), None, selected.attribute().local_name()),
            "",
        );
        let report = interpreter
            .boolean_attribute(selected, &element(correct), Some(&mismatched))
            .unwrap();
        assert_eq!(report.applicability(), Applicability::DifferentAttribute);
        assert_eq!(report.intrinsic(), None);
        let wrong_facts = interpreter
            .boolean_attribute_for_input(selected, &element(correct), Some(&raw), Input::Text)
            .unwrap();
        assert_eq!(wrong_facts.applicability(), Applicability::DifferentContext);
        assert_eq!(wrong_facts.context_author(), ContextAuthor::Unapplied);
        assert_eq!(wrong_facts.intrinsic(), None);
    }
    // The existing media selector stays separate from image controls.
    let raw = Attribute::new(KnownHtmlAttribute::Controls.qualified_name(), "");
    assert_eq!(
        interpreter
            .boolean_attribute(Boolean::MediaControls, &element("img"), Some(&raw))
            .unwrap()
            .intrinsic(),
        None
    );
    assert_eq!(
        interpreter
            .image_controls_with_context(&element("audio"), Some(&raw), ImageControls::AltNonempty)
            .unwrap()
            .intrinsic(),
        None
    );
    assert_eq!(
        interpreter
            .boolean_attribute(
                Boolean::Disabled,
                &element("link"),
                None::<&surgeist_html::Attribute>
            )
            .unwrap()
            .intrinsic(),
        None
    );
}

#[test]
fn new_boolean_roles_preserve_prefixes_raw_units_and_literal_spelling() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (selected, local, canonical) in [
        (Boolean::Ismap, "img", "IsMaP"),
        (Boolean::ImageControls, "img", "CoNtRoLs"),
        (Boolean::StylesheetDisabled, "link", "DiSaBlEd"),
    ] {
        let name = QualifiedName::new(Namespace::html(), Some("h".into()), local);
        for (value, conforming) in [
            (DocumentString::from(""), true),
            (DocumentString::from(canonical), true),
            (DocumentString::from("true"), false),
            (DocumentString::from("false"), false),
            (DocumentString::from(format!(" {canonical} ")), false),
            (DocumentString::from_utf16(vec![0xd800]), false),
        ] {
            let raw = Attribute::new(
                QualifiedName::new(
                    Namespace::none(),
                    Some("p".into()),
                    selected.attribute().local_name(),
                ),
                value,
            );
            let report = interpreter
                .boolean_attribute(selected, &name, Some(&raw))
                .unwrap();
            assert_eq!(report.element(), &name);
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(report.intrinsic(), Some(&true));
            assert_eq!(
                report.author(),
                if conforming {
                    Author::Conforming
                } else {
                    Author::Nonconforming(Conformance::BooleanSpelling)
                }
            );
            assert_eq!(report.context_author(), ContextAuthor::Unchecked);
        }
    }
}

#[test]
fn stylesheet_context_is_readmitted_and_attribute_limits_precede_applicability() {
    let interpreter = HtmlValueInterpreter::new(100);
    let rel = DocumentString::from("stylesheet");
    let context = interpreter.stylesheet_link_context(Some(&rel)).unwrap();
    let short = HtmlValueInterpreter::new(1);
    let error = short.stylesheet_link_context(Some(&rel)).unwrap_err();
    assert_eq!(error.required(), 10);
    assert_eq!(error.capacity(), 1);
    let error = short
        .stylesheet_disabled(
            &element("link"),
            None::<&surgeist_html::Attribute>,
            &context,
        )
        .unwrap_err();
    assert_eq!(error.required(), 10);
    let oversized = Attribute::new(KnownHtmlAttribute::Controls.qualified_name(), "xx");
    assert!(
        short
            .image_controls_with_context(
                &element("input"),
                Some(&oversized),
                ImageControls::AltEmpty
            )
            .is_err()
    );
    let oversized = Attribute::new(KnownHtmlAttribute::Ismap.qualified_name(), "xx");
    assert!(
        short
            .ismap_with_context(
                &QualifiedName::new(Namespace::svg(), None, "img"),
                Some(&oversized),
                ImageMap::Unavailable
            )
            .is_err()
    );
    let unknown_rel = DocumentString::from_utf16(vec![0xd800]);
    let context = interpreter
        .stylesheet_link_context(Some(&unknown_rel))
        .unwrap();
    assert_eq!(context.rel(), Some(&unknown_rel));
    assert!(!context.contains_stylesheet());
}

#[test]
fn hyperlink_author_context_and_literal_policy_conformance_remain_independent() {
    // The a/area clauses require omission without href, while their
    // known-value-limited reflection is unconditional. The common policy
    // clause supplies Empty for invalid raw syntax, without host resolution.
    let interpreter = HtmlValueInterpreter::new(100);
    for (local, anchor) in [("a", "#the-a-element"), ("area", "#the-area-element")] {
        for (value, state, author) in [
            ("", HtmlReferrerPolicy::Empty, Author::Conforming),
            ("OrIgIn", HtmlReferrerPolicy::Origin, Author::Conforming),
            (
                " origin ",
                HtmlReferrerPolicy::Empty,
                Author::Nonconforming(Conformance::UnrecognizedKeyword),
            ),
        ] {
            let raw = Attribute::new(KnownHtmlAttribute::Referrerpolicy.qualified_name(), value);
            for context in [Hyperlink::WithHref, Hyperlink::WithoutHref] {
                let report = interpreter
                    .referrerpolicy_for_hyperlink(&element(local), Some(&raw), context)
                    .unwrap();
                assert_eq!(report.applicability(), Applicability::Applicable);
                assert_eq!(report.intrinsic(), Some(&state));
                assert_eq!(report.author(), author);
                assert_eq!(
                    report.context_author(),
                    if context == Hyperlink::WithHref {
                        ContextAuthor::Conforming
                    } else {
                        ContextAuthor::Nonconforming(ContextConformance::ReferrerPolicyRequiresHref)
                    }
                );
                assert_eq!(report.raw_attribute(), Some(&raw));
                assert_eq!(report.hyperlink_context(), Some(context));
                assert!(
                    report
                        .source_clause()
                        .ends_with("#referrer-policy-attributes")
                );
                assert!(report.context_source_clause().unwrap().ends_with(anchor));
            }
        }
        for context in [Hyperlink::WithHref, Hyperlink::WithoutHref] {
            let report = interpreter
                .referrerpolicy_for_hyperlink(
                    &element(local),
                    None::<&surgeist_html::Attribute>,
                    context,
                )
                .unwrap();
            assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
            assert_eq!(report.author(), Author::Missing);
            assert_eq!(report.context_author(), ContextAuthor::NotRequired);
            assert_eq!(report.hyperlink_context(), Some(context));
        }
    }
}

#[test]
fn hyperlink_context_does_not_admit_another_element_or_bypass_raw_limit() {
    let raw = Attribute::new(
        KnownHtmlAttribute::Referrerpolicy.qualified_name(),
        "origin",
    );
    let interpreter = HtmlValueInterpreter::new(100);
    for local in ["img", "iframe", "link", "script", "div", "A"] {
        let report = interpreter
            .referrerpolicy_for_hyperlink(&element(local), Some(&raw), Hyperlink::WithoutHref)
            .unwrap();
        assert_eq!(report.applicability(), Applicability::DifferentElement);
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.context_author(), ContextAuthor::Unapplied);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
    let error = HtmlValueInterpreter::new(1)
        .referrerpolicy_for_hyperlink(&element("img"), Some(&raw), Hyperlink::WithoutHref)
        .unwrap_err();
    assert_eq!(error.capacity(), 1);
    assert_eq!(error.required(), 6);
}

#[test]
fn external_classic_script_flags_preserve_literal_boolean_conformance() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (selected, spelling) in [
        (Boolean::Async, "AsYnC"),
        (Boolean::Defer, "DeFeR"),
        (Boolean::Nomodule, "NoMoDuLe"),
    ] {
        for (value, expected) in [
            (DocumentString::from(""), Author::Conforming),
            (DocumentString::from(spelling), Author::Conforming),
            (
                DocumentString::from("false"),
                Author::Nonconforming(Conformance::BooleanSpelling),
            ),
            (
                DocumentString::from(format!(" {spelling}")),
                Author::Nonconforming(Conformance::BooleanSpelling),
            ),
            (
                DocumentString::from_utf16(vec![0xd800]),
                Author::Nonconforming(Conformance::BooleanSpelling),
            ),
        ] {
            let raw = Attribute::new(selected.attribute().qualified_name(), value);
            let report = interpreter
                .boolean_attribute_for_script(
                    selected,
                    &element("script"),
                    Some(&raw),
                    Script::ExternalClassic,
                )
                .unwrap();
            assert_eq!(report.intrinsic(), Some(&true));
            assert_eq!(report.author(), expected);
            assert_eq!(report.context_author(), ContextAuthor::Conforming);
            assert_eq!(report.raw_attribute(), Some(&raw));
        }
    }
}

#[test]
fn unqualified_hyperlink_policies_retain_intrinsic_state_and_unchecked_author_context() {
    let interpreter = HtmlValueInterpreter::new(100);
    for local in ["a", "area"] {
        for (value, state, author) in [
            ("", HtmlReferrerPolicy::Empty, Author::Conforming),
            ("ORIGIN", HtmlReferrerPolicy::Origin, Author::Conforming),
            (
                " unknown ",
                HtmlReferrerPolicy::Empty,
                Author::Nonconforming(Conformance::UnrecognizedKeyword),
            ),
        ] {
            let raw = Attribute::new(KnownHtmlAttribute::Referrerpolicy.qualified_name(), value);
            let report = interpreter
                .referrerpolicy(&element(local), Some(&raw))
                .unwrap();
            assert_eq!(report.applicability(), Applicability::Applicable);
            assert_eq!(report.intrinsic(), Some(&state));
            assert_eq!(report.author(), author);
            assert_eq!(report.context_author(), ContextAuthor::Unchecked);
            assert_eq!(report.hyperlink_context(), None);
            assert_eq!(report.raw_attribute(), Some(&raw));
        }
        let omitted = interpreter
            .referrerpolicy(&element(local), None::<&surgeist_html::Attribute>)
            .unwrap();
        assert_eq!(omitted.intrinsic(), Some(&HtmlReferrerPolicy::Empty));
        assert_eq!(omitted.context_author(), ContextAuthor::NotRequired);
        assert_eq!(omitted.author(), Author::Missing);
    }
    for local in ["img", "link", "iframe"] {
        let raw = Attribute::new(
            KnownHtmlAttribute::Referrerpolicy.qualified_name(),
            "origin",
        );
        let report = interpreter
            .referrerpolicy(&element(local), Some(&raw))
            .unwrap();
        assert_eq!(report.context_author(), ContextAuthor::NotRequired);
        assert_eq!(report.intrinsic(), Some(&HtmlReferrerPolicy::Origin));
    }
}

#[test]
fn wrong_boolean_fact_families_report_actual_context_mismatch() {
    let interpreter = HtmlValueInterpreter::new(100);
    let checked = Boolean::Checked.into_attribute(true).unwrap();
    let async_flag = Boolean::Async.into_attribute(true).unwrap();
    let required = Boolean::Formnovalidate.into_attribute(true).unwrap();
    let readonly = Boolean::Readonly.into_attribute(true).unwrap();
    let reports = [
        interpreter
            .boolean_attribute_for_script(
                Boolean::Checked,
                &element("input"),
                Some(&checked),
                Script::ExternalClassic,
            )
            .unwrap(),
        interpreter
            .boolean_attribute_for_input(
                Boolean::Async,
                &element("script"),
                Some(&async_flag),
                Input::Text,
            )
            .unwrap(),
        interpreter
            .boolean_attribute_for_script(
                Boolean::Formnovalidate,
                &element("button"),
                Some(&required),
                Script::ExternalClassic,
            )
            .unwrap(),
        interpreter
            .boolean_attribute_for_input(
                Boolean::Formnovalidate,
                &element("button"),
                Some(&required),
                Input::Submit,
            )
            .unwrap(),
        interpreter
            .boolean_attribute_for_script(
                Boolean::Readonly,
                &element("x-control"),
                Some(&readonly),
                Script::ExternalClassic,
            )
            .unwrap(),
        interpreter
            .boolean_attribute_for_input(
                Boolean::Readonly,
                &element("x-control"),
                Some(&readonly),
                Input::Text,
            )
            .unwrap(),
        interpreter
            .formnovalidate_for_button(
                &element("input"),
                Some(&required),
                HtmlButtonSubmitterContext::SubmitButton,
            )
            .unwrap(),
    ];
    for report in reports {
        assert_eq!(report.applicability(), Applicability::DifferentContext);
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.author(), Author::Unapplied);
        assert_eq!(report.context_author(), ContextAuthor::Unapplied);
        assert_eq!(report.recognition(), Recognition::Unapplied);
        assert!(report.raw_attribute().is_some());
    }
}

#[test]
fn all_actual_input_states_select_author_conformance_without_erasing_boolean_content() {
    // The six columns are independently authored from each normative state
    // section: alpha, checked, multiple, readonly, required, formnovalidate.
    let expectations = [
        (
            Input::Hidden,
            [false, false, false, false, false, false],
            "hidden-state-(type=hidden)",
        ),
        (
            Input::Text,
            [false, false, false, true, true, false],
            "text-(type=text)-state-and-search-state-(type=search)",
        ),
        (
            Input::Search,
            [false, false, false, true, true, false],
            "text-(type=text)-state-and-search-state-(type=search)",
        ),
        (
            Input::Telephone,
            [false, false, false, true, true, false],
            "telephone-state-(type=tel)",
        ),
        (
            Input::Url,
            [false, false, false, true, true, false],
            "url-state-(type=url)",
        ),
        (
            Input::Email,
            [false, false, true, true, true, false],
            "email-state-(type=email)",
        ),
        (
            Input::Password,
            [false, false, false, true, true, false],
            "password-state-(type=password)",
        ),
        (
            Input::Date,
            [false, false, false, true, true, false],
            "date-state-(type=date)",
        ),
        (
            Input::Month,
            [false, false, false, true, true, false],
            "month-state-(type=month)",
        ),
        (
            Input::Week,
            [false, false, false, true, true, false],
            "week-state-(type=week)",
        ),
        (
            Input::Time,
            [false, false, false, true, true, false],
            "time-state-(type=time)",
        ),
        (
            Input::LocalDateTime,
            [false, false, false, true, true, false],
            "local-date-and-time-state-(type=datetime-local)",
        ),
        (
            Input::Number,
            [false, false, false, true, true, false],
            "number-state-(type=number)",
        ),
        (
            Input::Range,
            [false, false, false, false, false, false],
            "range-state-(type=range)",
        ),
        (
            Input::Color,
            [true, false, false, false, false, false],
            "color-state-(type=color)",
        ),
        (
            Input::Checkbox,
            [false, true, false, false, true, false],
            "checkbox-state-(type=checkbox)",
        ),
        (
            Input::Radio,
            [false, true, false, false, true, false],
            "radio-button-state-(type=radio)",
        ),
        (
            Input::File,
            [false, false, true, false, true, false],
            "file-upload-state-(type=file)",
        ),
        (
            Input::Submit,
            [false, false, false, false, false, true],
            "submit-button-state-(type=submit)",
        ),
        (
            Input::Image,
            [false, false, false, false, false, true],
            "image-button-state-(type=image)",
        ),
        (
            Input::Reset,
            [false, false, false, false, false, false],
            "reset-button-state-(type=reset)",
        ),
        (
            Input::Button,
            [false, false, false, false, false, false],
            "button-state-(type=button)",
        ),
    ];
    let rules = [
        Boolean::Alpha,
        Boolean::Checked,
        Boolean::Multiple,
        Boolean::Readonly,
        Boolean::Required,
        Boolean::Formnovalidate,
    ];
    let interpreter = HtmlValueInterpreter::new(100);
    for (state, columns, anchor) in expectations {
        for (rule, expected) in rules.into_iter().zip(columns) {
            let raw = rule.into_attribute(true).unwrap();
            let report = interpreter
                .boolean_attribute_for_input(rule, &element("input"), Some(&raw), state)
                .unwrap();
            assert_eq!(report.input_state(), Some(state));
            assert!(report.context_source_clause().unwrap().ends_with(anchor));
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(report.observation(), Observation::PresentEmpty);
            assert_eq!(report.applicability(), Applicability::Applicable);
            assert_eq!(report.intrinsic(), Some(&true));
            assert_eq!(report.recognition(), Recognition::Empty);
            assert_eq!(report.author(), Author::Conforming);
            assert_eq!(
                report.context_author(),
                if expected {
                    ContextAuthor::Conforming
                } else {
                    ContextAuthor::Nonconforming(ContextConformance::InputStateForbidsAttribute)
                },
                "{state:?}/{rule:?}"
            );
            let absent = interpreter
                .boolean_attribute_for_input(
                    rule,
                    &element("input"),
                    None::<&surgeist_html::Attribute>,
                    state,
                )
                .unwrap();
            assert_eq!(absent.intrinsic(), Some(&false));
            assert_eq!(absent.context_author(), ContextAuthor::NotRequired);
        }
    }
}

#[test]
fn input_name_alone_preserves_content_and_leaves_state_conformance_unchecked() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (rule, row, anchor) in [
        (Boolean::Alpha, 8, "attr-input-alpha"),
        (Boolean::Checked, 20, "attr-input-checked"),
        (Boolean::Multiple, 100, "attr-input-multiple"),
        (Boolean::Readonly, 124, "attr-input-readonly"),
        (Boolean::Required, 129, "attr-input-required"),
        (Boolean::Formnovalidate, 59, "attr-fs-formnovalidate"),
    ] {
        let raw = Attribute::new(rule.attribute().qualified_name(), "false");
        for supplied in [None, Some(&raw)] {
            let report = interpreter
                .boolean_attribute(rule, &element("input"), supplied)
                .unwrap();
            assert_eq!(report.applicability(), Applicability::Applicable);
            assert_eq!(report.intrinsic(), Some(&supplied.is_some()));
            assert_eq!(
                report.recognition(),
                if supplied.is_some() {
                    Recognition::Unrecognized
                } else {
                    Recognition::Missing
                }
            );
            assert_eq!(
                report.author(),
                if supplied.is_some() {
                    Author::Nonconforming(Conformance::BooleanSpelling)
                } else {
                    Author::Missing
                }
            );
            assert_eq!(
                report.context_author(),
                if supplied.is_some() {
                    ContextAuthor::Unchecked
                } else {
                    ContextAuthor::NotRequired
                }
            );
            assert_eq!(report.raw_attribute(), supplied);
            assert_eq!(report.input_state(), None);
            assert_eq!(report.inventory_rows(), &[row]);
            assert!(report.source_clause().ends_with(anchor));
        }
    }
    // The real type producer can supply the missing-value default explicitly.
    let state = *interpreter
        .input_type(&element("input"), None::<&surgeist_html::Attribute>)
        .unwrap()
        .intrinsic()
        .unwrap();
    assert_eq!(state, Input::Text);
    let missing = interpreter
        .boolean_attribute_for_input(
            Boolean::Required,
            &element("input"),
            None::<&surgeist_html::Attribute>,
            state,
        )
        .unwrap();
    assert_eq!(missing.intrinsic(), Some(&false));
    assert_eq!(missing.observation(), Observation::Missing);
    assert_eq!(missing.author(), Author::Missing);
}

#[test]
fn forbidden_input_states_and_invalid_boolean_spelling_produce_both_errors() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (selected, state) in [
        (Boolean::Alpha, Input::Text),
        (Boolean::Checked, Input::Text),
        (Boolean::Multiple, Input::Text),
        (Boolean::Readonly, Input::Checkbox),
        (Boolean::Required, Input::Hidden),
        (Boolean::Formnovalidate, Input::Button),
    ] {
        let raw = Attribute::new(selected.attribute().qualified_name(), "false");
        let report = interpreter
            .boolean_attribute_for_input(selected, &element("input"), Some(&raw), state)
            .unwrap();
        assert_eq!(report.applicability(), Applicability::Applicable);
        assert_eq!(report.intrinsic(), Some(&true));
        assert_eq!(report.recognition(), Recognition::Unrecognized);
        assert_eq!(
            report.author(),
            Author::Nonconforming(Conformance::BooleanSpelling)
        );
        assert_eq!(
            report.context_author(),
            ContextAuthor::Nonconforming(ContextConformance::InputStateForbidsAttribute)
        );
        assert_eq!(report.input_state(), Some(state));
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn non_submit_button_and_invalid_boolean_spelling_produce_both_errors() {
    let raw = Attribute::new(
        Boolean::Formnovalidate.attribute().qualified_name(),
        "false",
    );
    let report = HtmlValueInterpreter::new(100)
        .formnovalidate_for_button(
            &element("button"),
            Some(&raw),
            HtmlButtonSubmitterContext::NotSubmitButton,
        )
        .unwrap();
    assert_eq!(report.applicability(), Applicability::Applicable);
    assert_eq!(report.intrinsic(), Some(&true));
    assert_eq!(
        report.author(),
        Author::Nonconforming(Conformance::BooleanSpelling)
    );
    assert_eq!(
        report.context_author(),
        ContextAuthor::Nonconforming(ContextConformance::FormnovalidateRequiresSubmitButton)
    );
    assert_eq!(report.recognition(), Recognition::Unrecognized);
    assert_eq!(report.raw_attribute(), Some(&raw));
}

#[test]
fn select_and_textarea_use_their_own_clauses_without_input_facts() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (rule, local, anchor, row) in [
        (Boolean::Multiple, "select", "attr-select-multiple", 100),
        (Boolean::Readonly, "textarea", "attr-textarea-readonly", 124),
        (Boolean::Required, "select", "attr-select-required", 129),
        (Boolean::Required, "textarea", "attr-textarea-required", 129),
    ] {
        let raw = rule.into_attribute(true).unwrap();
        let present = interpreter
            .boolean_attribute(rule, &element(local), Some(&raw))
            .unwrap();
        assert_eq!(present.applicability(), Applicability::Applicable);
        assert_eq!(present.intrinsic(), Some(&true));
        assert!(present.source_clause().ends_with(anchor));
        assert_eq!(present.inventory_rows(), &[row]);
        assert_eq!(present.input_state(), None);
        let missing = interpreter
            .boolean_attribute(rule, &element(local), None::<&surgeist_html::Attribute>)
            .unwrap();
        assert_eq!(missing.intrinsic(), Some(&false));
        assert_eq!(missing.recognition(), Recognition::Missing);
    }
    let wrong = interpreter
        .boolean_attribute(
            Boolean::Multiple,
            &element("textarea"),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(wrong.applicability(), Applicability::DifferentElement);
    assert_eq!(wrong.intrinsic(), None);
}

#[test]
fn script_flags_follow_all_seven_supplied_script_contexts() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (context, expected) in [
        (Script::ExternalClassic, [true, true, true]),
        (Script::InlineClassic, [false, false, true]),
        (Script::ExternalModule, [true, false, false]),
        (Script::InlineModule, [true, false, false]),
        (Script::ImportMap, [false, false, false]),
        (Script::SpeculationRules, [false, false, false]),
        (Script::DataBlock, [false, false, false]),
    ] {
        for ((rule, anchor, row), allowed) in [
            (Boolean::Async, "attr-script-async", 11),
            (Boolean::Defer, "attr-script-defer", 40),
            (Boolean::Nomodule, "attr-script-nomodule", 109),
        ]
        .into_iter()
        .zip(expected)
        {
            let raw = Attribute::new(rule.attribute().qualified_name(), "false");
            let report = interpreter
                .boolean_attribute_for_script(rule, &element("script"), Some(&raw), context)
                .unwrap();
            assert_eq!(report.script_context(), Some(context));
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(report.inventory_rows(), &[row]);
            assert!(report.source_clause().ends_with(anchor));
            assert!(
                report
                    .context_source_clause()
                    .unwrap()
                    .ends_with("the-script-element")
            );
            assert_eq!(report.applicability(), Applicability::Applicable);
            assert_eq!(report.intrinsic(), Some(&true));
            assert_eq!(
                report.author(),
                Author::Nonconforming(Conformance::BooleanSpelling)
            );
            assert_eq!(report.recognition(), Recognition::Unrecognized);
            assert_eq!(
                report.context_author(),
                if allowed {
                    ContextAuthor::Conforming
                } else {
                    ContextAuthor::Nonconforming(ContextConformance::ScriptContextForbidsAttribute)
                }
            );
            let missing = interpreter
                .boolean_attribute_for_script(
                    rule,
                    &element("script"),
                    None::<&surgeist_html::Attribute>,
                    context,
                )
                .unwrap();
            assert_eq!(missing.intrinsic(), Some(&false));
            assert_eq!(missing.context_author(), ContextAuthor::NotRequired);
        }
    }
}

#[test]
fn script_name_without_kind_and_src_facts_preserves_content_and_unchecked_conformance() {
    for rule in [Boolean::Async, Boolean::Defer, Boolean::Nomodule] {
        let raw = rule.into_attribute(true).unwrap();
        let report = HtmlValueInterpreter::new(100)
            .boolean_attribute(rule, &element("script"), Some(&raw))
            .unwrap();
        assert_eq!(report.applicability(), Applicability::Applicable);
        assert_eq!(report.script_context(), None);
        assert_eq!(report.intrinsic(), Some(&true));
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(report.context_author(), ContextAuthor::Unchecked);
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
}

#[test]
fn button_formnovalidate_retains_content_and_observes_submitter_author_conformance() {
    let interpreter = HtmlValueInterpreter::new(100);
    let raw = Boolean::Formnovalidate.into_attribute(true).unwrap();
    let missing = interpreter
        .boolean_attribute(Boolean::Formnovalidate, &element("button"), Some(&raw))
        .unwrap();
    assert_eq!(missing.applicability(), Applicability::Applicable);
    assert_eq!(missing.intrinsic(), Some(&true));
    assert_eq!(missing.context_author(), ContextAuthor::Unchecked);
    for context in [
        HtmlButtonSubmitterContext::SubmitButton,
        HtmlButtonSubmitterContext::NotSubmitButton,
    ] {
        let report = interpreter
            .formnovalidate_for_button(&element("button"), Some(&raw), context)
            .unwrap();
        assert_eq!(report.button_submitter_context(), Some(context));
        assert_eq!(report.raw_attribute(), Some(&raw));
        assert!(report.source_clause().ends_with("attr-fs-formnovalidate"));
        assert!(
            report
                .context_source_clause()
                .unwrap()
                .ends_with("attr-button-type")
        );
        assert_eq!(report.applicability(), Applicability::Applicable);
        assert_eq!(report.intrinsic(), Some(&true));
        assert_eq!(report.author(), Author::Conforming);
        assert_eq!(
            report.context_author(),
            if context == HtmlButtonSubmitterContext::SubmitButton {
                ContextAuthor::Conforming
            } else {
                ContextAuthor::Nonconforming(ContextConformance::FormnovalidateRequiresSubmitButton)
            }
        );
        let absent = interpreter
            .formnovalidate_for_button(
                &element("button"),
                None::<&surgeist_html::Attribute>,
                context,
            )
            .unwrap();
        assert_eq!(absent.intrinsic(), Some(&false));
        assert_eq!(absent.context_author(), ContextAuthor::NotRequired);
    }
}

#[test]
fn custom_readonly_keeps_explicit_classification_separate_from_editability() {
    let interpreter = HtmlValueInterpreter::new(100);
    let subject = element("custom-control");
    let raw = Attribute::new(KnownHtmlAttribute::Readonly.qualified_name(), "false");
    let required = interpreter
        .boolean_attribute(Boolean::Readonly, &subject, Some(&raw))
        .unwrap();
    assert_eq!(
        required.applicability(),
        Applicability::CustomControlContextRequired
    );
    assert_eq!(required.intrinsic(), None);
    for context in [
        HtmlCustomControlAttributeContext::FormAssociated,
        HtmlCustomControlAttributeContext::NotFormAssociated,
    ] {
        let report = interpreter
            .readonly_for_custom_control(&subject, Some(&raw), context)
            .unwrap();
        assert_eq!(report.custom_control_context(), Some(context));
        assert!(report.source_clause().ends_with("attr-face-readonly"));
        assert_eq!(report.inventory_rows(), &[125]);
        assert_eq!(report.raw_attribute(), Some(&raw));
        match context {
            HtmlCustomControlAttributeContext::FormAssociated => {
                assert_eq!(report.intrinsic(), Some(&true));
                assert_eq!(
                    report.author(),
                    Author::Nonconforming(Conformance::BooleanSpelling)
                );
            }
            HtmlCustomControlAttributeContext::NotFormAssociated => {
                assert_eq!(
                    report.applicability(),
                    Applicability::InapplicableCustomControl
                );
                assert_eq!(report.intrinsic(), None);
            }
        }
    }
}

#[test]
fn new_boolean_rules_distinguish_missing_empty_name_and_invalid_spelling() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (rule, state) in [
        (Boolean::Alpha, Input::Color),
        (Boolean::Checked, Input::Checkbox),
        (Boolean::Multiple, Input::Email),
        (Boolean::Readonly, Input::Text),
        (Boolean::Required, Input::Number),
        (Boolean::Formnovalidate, Input::Submit),
    ] {
        let missing = interpreter
            .boolean_attribute_for_input(
                rule,
                &element("input"),
                None::<&surgeist_html::Attribute>,
                state,
            )
            .unwrap();
        assert_eq!(missing.intrinsic(), Some(&false));
        assert_eq!(missing.observation(), Observation::Missing);
        for value in [
            DocumentString::from(""),
            rule.attribute().local_name().to_ascii_uppercase().into(),
            "false".into(),
            "true".into(),
            " ".into(),
            DocumentString::from_utf16(vec![0xD800]),
        ] {
            let raw = Attribute::new(rule.attribute().qualified_name(), value.clone());
            let report = interpreter
                .boolean_attribute_for_input(rule, &element("input"), Some(&raw), state)
                .unwrap();
            assert_eq!(report.intrinsic(), Some(&true));
            assert_eq!(report.raw(), Some(&value));
            if value.is_empty() {
                assert_eq!(report.observation(), Observation::PresentEmpty);
                assert_eq!(report.recognition(), Recognition::Empty);
                assert_eq!(report.author(), Author::Conforming);
            } else if value == rule.attribute().local_name().to_ascii_uppercase().into() {
                assert_eq!(report.recognition(), Recognition::BooleanName);
                assert_eq!(report.author(), Author::Conforming);
            } else {
                assert_eq!(report.observation(), Observation::PresentNonempty);
                assert_eq!(report.recognition(), Recognition::Unrecognized);
                assert_eq!(
                    report.author(),
                    Author::Nonconforming(Conformance::BooleanSpelling)
                );
            }
        }
        assert_eq!(rule.into_attribute(false), None);
        assert_eq!(
            rule.into_attribute(true).unwrap().value(),
            &DocumentString::from("")
        );
    }
}

#[test]
fn boolean_completion_preserves_exact_names_and_rejects_expanded_identity_mismatches() {
    let interpreter = HtmlValueInterpreter::new(100);
    let subject = QualifiedName::new(Namespace::html(), Some("h".into()), "input");
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("a".into()), "checked"),
        "ChEcKeD",
    );
    let report = interpreter
        .boolean_attribute_for_input(Boolean::Checked, &subject, Some(&raw), Input::Checkbox)
        .unwrap();
    assert_eq!(report.element(), &subject);
    assert_eq!(report.raw_attribute(), Some(&raw));
    assert_eq!(report.recognition(), Recognition::BooleanName);
    for namespace in [Namespace::none(), Namespace::svg(), Namespace::mathml()] {
        let foreign = QualifiedName::new(namespace, None, "input");
        let report = interpreter
            .boolean_attribute_for_input(Boolean::Checked, &foreign, Some(&raw), Input::Checkbox)
            .unwrap();
        assert_eq!(
            report.applicability(),
            Applicability::ForeignElementNamespace
        );
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.input_state(), Some(Input::Checkbox));
    }
    for name in [
        QualifiedName::new(Namespace::none(), None, "CHECKED"),
        QualifiedName::new(Namespace::html(), None, "checked"),
    ] {
        let wrong = Attribute::new(name, "");
        let report = interpreter
            .boolean_attribute_for_input(Boolean::Checked, &subject, Some(&wrong), Input::Checkbox)
            .unwrap();
        assert_eq!(report.applicability(), Applicability::DifferentAttribute);
        assert_eq!(report.raw_attribute(), Some(&wrong));
        assert_eq!(report.intrinsic(), None);
    }
}

#[test]
fn finite_admission_precedes_even_missing_wrong_or_inapplicable_contexts() {
    let raw = Attribute::new(
        KnownHtmlAttribute::Checked.qualified_name(),
        DocumentString::from_utf16(vec![0xD800, 0x41]),
    );
    let interpreter = HtmlValueInterpreter::new(1);
    assert!(
        interpreter
            .boolean_attribute(Boolean::Checked, &element("input"), Some(&raw))
            .is_err()
    );
    assert!(
        interpreter
            .boolean_attribute_for_input(
                Boolean::Checked,
                &element("input"),
                Some(&raw),
                Input::Text
            )
            .is_err()
    );
    assert!(
        interpreter
            .boolean_attribute_for_input(
                Boolean::Checked,
                &element("div"),
                Some(&raw),
                Input::Checkbox
            )
            .is_err()
    );
    assert!(
        interpreter
            .boolean_attribute_for_input(
                Boolean::Required,
                &element("input"),
                Some(&raw),
                Input::Checkbox
            )
            .is_err()
    );
    assert!(
        interpreter
            .boolean_attribute_for_input(
                Boolean::Checked,
                &QualifiedName::new(Namespace::svg(), None, "input"),
                Some(&raw),
                Input::Checkbox
            )
            .is_err()
    );
}
