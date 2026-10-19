use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeAuthorState,
    HtmlAttributeConformanceError, HtmlAttributeContextAuthorState, HtmlAttributeObservation,
    HtmlAttributeRecognition, HtmlAttributeReport, HtmlNavigableTarget,
    HtmlNavigableTargetAttribute as Role, HtmlNavigableTargetError as Error,
    HtmlNavigableTargetKeyword as Keyword, HtmlNavigableTargetName, HtmlValueInterpreter,
    KnownHtmlAttribute as Attr, KnownHtmlElement as Element, Namespace, QualifiedName,
};

// Expectations follow #navigable-target-names and the six actual defining clauses.
// Row105's non-normative summary does not admit keywords as iframe/object names.
const ROLES: [(Role, Element, Attr, u16, &str, bool); 6] = [
    (
        Role::HyperlinkTarget,
        Element::A,
        Attr::Target,
        157,
        "attr-hyperlink-target",
        true,
    ),
    (
        Role::BaseTarget,
        Element::Base,
        Attr::Target,
        158,
        "attr-base-target",
        true,
    ),
    (
        Role::FormTarget,
        Element::Form,
        Attr::Target,
        159,
        "attr-fs-target",
        false,
    ),
    (
        Role::SubmitterFormtarget,
        Element::Button,
        Attr::Formtarget,
        60,
        "attr-fs-formtarget",
        true,
    ),
    (
        Role::IframeName,
        Element::Iframe,
        Attr::Name,
        105,
        "attr-iframe-name",
        false,
    ),
    (
        Role::ObjectName,
        Element::Object,
        Attr::Name,
        105,
        "attr-object-name",
        false,
    ),
];

fn attribute(name: Attr, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(name.qualified_name(), value)
}

fn read(
    role: Role,
    element: Element,
    raw: Option<&Attribute>,
) -> HtmlAttributeReport<HtmlNavigableTarget> {
    HtmlValueInterpreter::new(8192)
        .navigable_target_attribute(role, &element.qualified_name(), raw)
        .unwrap()
}

fn expected_context(required: bool) -> HtmlAttributeContextAuthorState {
    if required {
        HtmlAttributeContextAuthorState::Unchecked
    } else {
        HtmlAttributeContextAuthorState::NotRequired
    }
}

fn assert_name(report: &HtmlAttributeReport<HtmlNavigableTarget>, value: &DocumentString) {
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
    assert_eq!(report.recognition(), HtmlAttributeRecognition::NonemptyText);
    assert_eq!(report.raw(), Some(value));
    match report.intrinsic().unwrap() {
        HtmlNavigableTarget::Name(name) => assert_eq!(name.value(), value),
        HtmlNavigableTarget::Keyword(_) => panic!("expected an exact ordinary name"),
    }
}

fn assert_invalid(report: &HtmlAttributeReport<HtmlNavigableTarget>, error: Error) {
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(report.intrinsic(), None);
    assert_eq!(
        report.author(),
        HtmlAttributeAuthorState::Nonconforming(
            HtmlAttributeConformanceError::InvalidNavigableTarget(error),
        )
    );
}

#[test]
fn missing_and_specified_empty_are_distinct_for_every_optional_role() {
    for (role, element, name, _, _, context) in ROLES {
        let missing = read(role, element, None);
        assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
        assert_eq!(missing.recognition(), HtmlAttributeRecognition::Missing);
        assert_eq!(missing.author(), HtmlAttributeAuthorState::Missing);
        assert_eq!(
            missing.context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(missing.raw_attribute(), None);
        assert_eq!(missing.intrinsic(), None);
        let empty = attribute(name, "");
        let report = read(role, element, Some(&empty));
        assert_invalid(&report, Error::Empty);
        assert_eq!(report.raw_attribute(), Some(&empty));
        assert_eq!(report.observation(), HtmlAttributeObservation::PresentEmpty);
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Empty);
        assert_eq!(report.context_author(), expected_context(context));
    }
    assert_eq!(HtmlNavigableTargetName::new(""), Err(Error::Empty));
}

#[test]
fn spaces_tabs_newlines_and_less_than_are_valid_when_the_forbidden_pair_is_absent() {
    for text in [
        "123", "a b", " ", "<", "<<", "\t", "\n", "\r", "a\t\nb\r", "\u{c}<", " < ", "a_b", "é",
        "e\u{301}", "\u{a0}<", " _blank", "x_blank",
    ] {
        let value = DocumentString::from(text);
        let native = HtmlNavigableTargetName::new(value.clone()).unwrap();
        assert_eq!(native.value(), &value);
        for (role, element, name, _, _, context) in ROLES {
            let raw = attribute(name, value.clone());
            let report = read(role, element, Some(&raw));
            assert_name(&report, &value);
            assert_eq!(report.context_author(), expected_context(context));
        }
    }
}

#[test]
fn null_non_ascii_whitespace_and_surrogates_preserve_exact_code_units() {
    for units in [
        vec![0],
        vec![0xD800],
        vec![0xDC00],
        vec![0xD83D, 0xDE00],
        vec![0xD800, 0xA0, 0, 0xDC00, 0x3C],
        vec![0x2003, 0x3C],
    ] {
        let value = DocumentString::from_utf16(units.clone());
        let native = HtmlNavigableTargetName::new(value.clone()).unwrap();
        assert_eq!(native.value().code_units(), units);
        for (role, element, name, _, _, _) in ROLES {
            let raw = attribute(name, value.clone());
            let report = read(role, element, Some(&raw));
            assert_name(&report, &value);
            assert_eq!(report.raw().unwrap().code_units(), units);
        }
    }
}

#[test]
fn tab_lf_and_cr_with_less_than_are_invalid_in_either_order_with_exact_first_offsets() {
    for tab_or_newline in [0x09, 0x0A, 0x0D] {
        for (units, tab_offset, less_offset) in [
            (vec![tab_or_newline, 0x3C], 0, 1),
            (vec![0x3C, tab_or_newline], 1, 0),
            (vec![0xD83D, 0xDE00, tab_or_newline, 0x78, 0x3C], 2, 4),
            (
                vec![0xD800, 0x3C, 0x3C, 0x78, tab_or_newline, tab_or_newline],
                4,
                1,
            ),
            (
                vec![0xDC00, tab_or_newline, tab_or_newline, 0x78, 0x3C, 0x3C],
                1,
                4,
            ),
        ] {
            let value = DocumentString::from_utf16(units);
            let error = Error::TabOrNewlineWithLessThan {
                tab_or_newline_code_unit_offset: tab_offset,
                less_than_code_unit_offset: less_offset,
            };
            assert_eq!(HtmlNavigableTargetName::new(value.clone()), Err(error));
            for (role, element, name, _, _, context) in ROLES {
                let raw = attribute(name, value.clone());
                let report = read(role, element, Some(&raw));
                assert_invalid(&report, error);
                assert_eq!(report.raw_attribute(), Some(&raw));
                assert_eq!(report.recognition(), HtmlAttributeRecognition::Unrecognized);
                assert_eq!(report.context_author(), expected_context(context));
            }
        }
    }
}

#[test]
fn initial_underscore_is_reserved_and_its_error_precedes_other_name_violations() {
    for text in ["_", "_other", "_unfencedTop", "_blank ", "_SELF\t", "_\t<"] {
        assert_eq!(
            HtmlNavigableTargetName::new(text),
            Err(Error::ReservedLeadingUnderscore)
        );
        for (role, element, name, _, _, _) in ROLES {
            let raw = attribute(name, text);
            assert_invalid(
                &read(role, element, Some(&raw)),
                Error::ReservedLeadingUnderscore,
            );
        }
    }
}

#[test]
fn four_ascii_insensitive_keywords_apply_only_to_target_and_formtarget_roles() {
    for (text, canonical, keyword) in [
        ("_blank", "_blank", Keyword::Blank),
        ("_bLaNk", "_blank", Keyword::Blank),
        ("_SELF", "_self", Keyword::SelfTarget),
        ("_sElF", "_self", Keyword::SelfTarget),
        ("_PARENT", "_parent", Keyword::Parent),
        ("_pArEnT", "_parent", Keyword::Parent),
        ("_TOP", "_top", Keyword::Top),
        ("_ToP", "_top", Keyword::Top),
    ] {
        assert_eq!(
            HtmlNavigableTargetName::new(text),
            Err(Error::ReservedLeadingUnderscore)
        );
        for (role, element, name, _, _, context) in ROLES {
            let raw = attribute(name, text);
            let report = read(role, element, Some(&raw));
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(report.raw(), Some(&DocumentString::from(text)));
            assert_eq!(report.context_author(), expected_context(context));
            if matches!(role, Role::IframeName | Role::ObjectName) {
                assert_invalid(&report, Error::ReservedLeadingUnderscore);
            } else {
                assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
                assert_eq!(
                    report.recognition(),
                    HtmlAttributeRecognition::Keyword {
                        matched: canonical,
                        canonical,
                    }
                );
                assert_eq!(
                    report.intrinsic(),
                    Some(&HtmlNavigableTarget::Keyword(keyword))
                );
            }
        }
    }
}

#[test]
fn keyword_matching_never_trims_or_folds_non_ascii_units() {
    for text in ["_ſelf", "_ＳＥＬＦ", "_blank\u{a0}", "_blank\n", "_top "] {
        let raw = attribute(Attr::Target, text);
        assert_invalid(
            &read(Role::FormTarget, Element::Form, Some(&raw)),
            Error::ReservedLeadingUnderscore,
        );
    }
    for text in [" _blank", "\t_self", "\u{a0}_top", "\n_parent"] {
        let raw = attribute(Attr::Target, text);
        assert_name(
            &read(Role::FormTarget, Element::Form, Some(&raw)),
            &DocumentString::from(text),
        );
    }
}

#[test]
fn native_checked_values_compose_through_real_generic_attribute_factory() {
    let units = vec![0xD800, 0, 0xA0, 0x3C, 0xDC00];
    let exact = DocumentString::from_utf16(units.clone());
    let checked = HtmlNavigableTargetName::new(exact.clone()).unwrap();
    let target = HtmlNavigableTarget::Name(checked.clone());
    assert_eq!(target.as_document_string(), exact);
    for (role, element, name, _, _, _) in ROLES {
        let emitted = attribute(name, target.as_document_string());
        assert_eq!(emitted.name(), &name.qualified_name());
        assert_eq!(emitted.name().namespace(), &Namespace::none());
        assert_eq!(emitted.name().prefix(), None);
        assert_eq!(emitted.value().code_units(), units);
        assert_name(&read(role, element, Some(&emitted)), &exact);
    }
    for (keyword, canonical) in [
        (Keyword::Blank, "_blank"),
        (Keyword::SelfTarget, "_self"),
        (Keyword::Parent, "_parent"),
        (Keyword::Top, "_top"),
    ] {
        assert_eq!(
            keyword.as_document_string(),
            DocumentString::from(canonical)
        );
        let native = HtmlNavigableTarget::Keyword(keyword);
        assert_eq!(native.as_document_string(), DocumentString::from(canonical));
        let emitted = attribute(Attr::Target, native.as_document_string());
        let report = read(Role::FormTarget, Element::Form, Some(&emitted));
        assert_eq!(report.intrinsic(), Some(&native));
    }
}

#[test]
fn each_defining_role_has_exact_attribute_rows_and_normative_anchor() {
    for (role, element, name, row, anchor, context) in ROLES {
        let element = QualifiedName::new(Namespace::html(), Some("h".into()), element.local_name());
        let raw = Attribute::new(
            QualifiedName::new(Namespace::none(), Some("p".into()), name.local_name()),
            "opaque",
        );
        let report = HtmlValueInterpreter::new(64)
            .navigable_target_attribute(role, &element, Some(&raw))
            .unwrap();
        assert_name(&report, &DocumentString::from("opaque"));
        assert_eq!(report.element(), &element);
        assert_eq!(report.raw_attribute(), Some(&raw));
        assert_eq!(report.attribute(), name);
        assert_eq!(report.inventory_rows(), &[row]);
        assert_eq!(
            report.source_clause(),
            format!(
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#{anchor}"
            )
        );
        assert_eq!(report.vocabulary_source_clause(), None);
        assert_eq!(report.context_author(), expected_context(context));
        assert_eq!(report.script_context(), None);
        assert_eq!(report.hyperlink_context(), None);
        assert_eq!(report.input_state(), None);
        assert_eq!(report.button_submitter_context(), None);
    }
}

#[test]
fn area_hyperlink_and_input_override_share_the_correct_two_element_roles() {
    for (role, element, name) in [
        (Role::HyperlinkTarget, Element::Area, Attr::Target),
        (Role::SubmitterFormtarget, Element::Input, Attr::Formtarget),
    ] {
        for text in ["ordinary", "_ToP"] {
            let raw = attribute(name, text);
            let report = read(role, element, Some(&raw));
            assert_eq!(
                report.applicability(),
                HtmlAttributeApplicability::Applicable
            );
            assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::Unchecked
            );
        }
    }
}

#[test]
fn wrong_elements_or_expanded_attribute_names_are_unapplied_without_grammar() {
    for (role, _, name, _, _, _) in ROLES {
        let raw = attribute(name, "_\t<");
        for (element, applicability) in [
            (
                Element::Div.qualified_name(),
                HtmlAttributeApplicability::DifferentElement,
            ),
            (
                QualifiedName::new(Namespace::html(), None, "x-widget"),
                HtmlAttributeApplicability::DifferentElement,
            ),
            (
                QualifiedName::new(Namespace::html(), None, "FORM"),
                HtmlAttributeApplicability::DifferentElement,
            ),
            (
                QualifiedName::new(Namespace::svg(), None, "form"),
                HtmlAttributeApplicability::ForeignElementNamespace,
            ),
            (
                QualifiedName::new(Namespace::mathml(), None, "form"),
                HtmlAttributeApplicability::ForeignElementNamespace,
            ),
            (
                QualifiedName::new(Namespace::none(), None, "form"),
                HtmlAttributeApplicability::ForeignElementNamespace,
            ),
        ] {
            let report = HtmlValueInterpreter::new(64)
                .navigable_target_attribute(role, &element, Some(&raw))
                .unwrap();
            assert_eq!(report.applicability(), applicability);
            assert_eq!(report.element(), &element);
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(report.intrinsic(), None);
            assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
            assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::Unapplied
            );
        }
    }
    for (role, element, _, _, _, _) in ROLES {
        for name in [
            Attr::Id.qualified_name(),
            QualifiedName::new(Namespace::none(), None, "TARGET"),
            QualifiedName::new(Namespace::svg(), Some("s".into()), "target"),
        ] {
            let raw = Attribute::new(name, "_\t<");
            let report = read(role, element, Some(&raw));
            assert_eq!(
                report.applicability(),
                HtmlAttributeApplicability::DifferentAttribute
            );
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(report.author(), HtmlAttributeAuthorState::Unapplied);
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::Unapplied
            );
            assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
        }
    }
}

#[test]
fn full_original_utf16_cap_precedes_wrong_role_and_local_failure() {
    for (role, element, name, _, _, _) in ROLES {
        for units in [vec![0xD800], vec![0xD83D, 0xDE00], vec![0xD800, 0x09, 0x3C]] {
            let value = DocumentString::from_utf16(units.clone());
            for (element, attribute_name) in [
                (element.qualified_name(), name.qualified_name()),
                (element.qualified_name(), Attr::Id.qualified_name()),
                (
                    QualifiedName::new(Namespace::svg(), None, "foreign"),
                    name.qualified_name(),
                ),
            ] {
                let raw = Attribute::new(attribute_name, value.clone());
                let error = HtmlValueInterpreter::new(units.len() - 1)
                    .navigable_target_attribute(role, &element, Some(&raw))
                    .unwrap_err();
                assert_eq!(error.capacity(), units.len() - 1);
                assert_eq!(error.required(), units.len());
                let report = HtmlValueInterpreter::new(units.len())
                    .navigable_target_attribute(role, &element, Some(&raw))
                    .unwrap();
                assert_eq!(report.raw_attribute(), Some(&raw));
                assert_eq!(report.raw().unwrap().code_units(), units);
                assert_eq!(report.input_code_units(), units.len());
            }
        }
        let zero = HtmlValueInterpreter::new(0);
        assert_eq!(
            zero.navigable_target_attribute(
                role,
                &element.qualified_name(),
                None::<&surgeist_html::Attribute>
            )
            .unwrap()
            .author(),
            HtmlAttributeAuthorState::Missing
        );
        let empty = attribute(name, "");
        assert_invalid(
            &zero
                .navigable_target_attribute(role, &element.qualified_name(), Some(&empty))
                .unwrap(),
            Error::Empty,
        );
        let nonempty = attribute(name, " ");
        let error = zero
            .navigable_target_attribute(role, &element.qualified_name(), Some(&nonempty))
            .unwrap_err();
        assert_eq!(error.capacity(), 0);
        assert_eq!(error.required(), 1);
    }
}

#[test]
fn long_exact_name_and_retained_report_survive_later_operations_and_raw_drop() {
    let value = DocumentString::from("Opaque1!".repeat(1024));
    let raw = attribute(Attr::Target, value.clone());
    let report = read(Role::FormTarget, Element::Form, Some(&raw));
    assert_name(&report, &value);
    let retained = report.clone();
    let error = HtmlValueInterpreter::new(8191)
        .navigable_target_attribute(
            Role::FormTarget,
            &Element::Form.qualified_name(),
            Some(&raw),
        )
        .unwrap_err();
    assert_eq!(error.required(), 8192);
    assert_eq!(error.capacity(), 8191);
    drop(raw);
    let later = attribute(Attr::Target, "\t<");
    assert_invalid(
        &read(Role::FormTarget, Element::Form, Some(&later)),
        Error::TabOrNewlineWithLessThan {
            tab_or_newline_code_unit_offset: 0,
            less_than_code_unit_offset: 1,
        },
    );
    assert_eq!(report, retained);
    assert_name(&report, &value);
    assert_eq!(report.intrinsic().unwrap().as_document_string(), value);
}

#[test]
fn local_checked_values_never_establish_hyperlink_order_or_submitter_context() {
    for (role, element, name, _, _, context) in ROLES {
        for text in ["same", "_invalid", "", "<\t"] {
            let raw = attribute(name, text);
            let report = read(role, element, Some(&raw));
            assert_eq!(report.context_author(), expected_context(context));
        }
    }
    let raw = attribute(Attr::Target, "same");
    let a = read(Role::HyperlinkTarget, Element::A, Some(&raw));
    let base = read(Role::BaseTarget, Element::Base, Some(&raw));
    assert_eq!(a.intrinsic(), base.intrinsic());
    assert_eq!(
        a.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(
        base.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
}
