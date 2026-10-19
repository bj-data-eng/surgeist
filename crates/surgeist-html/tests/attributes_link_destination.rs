use surgeist_html::*;

fn link() -> QualifiedName {
    KnownHtmlElement::Link.qualified_name()
}
fn attr(name: KnownHtmlAttribute, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(name.qualified_name(), value)
}
fn attrs(values: impl IntoIterator<Item = Attribute>) -> OrderedAttributes {
    OrderedAttributes::new(values).unwrap()
}
fn interpreter() -> HtmlValueInterpreter {
    HtmlValueInterpreter::new(4096)
}
fn context_error(error: HtmlLinkDestinationContextError) -> HtmlLinkDestinationContextAuthorState {
    HtmlLinkDestinationContextAuthorState::Nonconforming(error)
}

// Independent literal domains: HTML877d #preload-destination and
// #module-preload-destination + Fetch e946 #request-destination-script-like.
const DESTINATIONS: &[(&str, HtmlLinkDestination, bool, bool)] = &[
    (
        "audioworklet",
        HtmlLinkDestination::AudioWorklet,
        false,
        true,
    ),
    ("fetch", HtmlLinkDestination::Fetch, true, false),
    ("font", HtmlLinkDestination::Font, true, false),
    ("image", HtmlLinkDestination::Image, true, false),
    ("json", HtmlLinkDestination::Json, false, true),
    (
        "paintworklet",
        HtmlLinkDestination::PaintWorklet,
        false,
        true,
    ),
    ("script", HtmlLinkDestination::Script, true, true),
    (
        "serviceworker",
        HtmlLinkDestination::ServiceWorker,
        false,
        true,
    ),
    (
        "sharedworker",
        HtmlLinkDestination::SharedWorker,
        false,
        true,
    ),
    ("style", HtmlLinkDestination::Style, true, true),
    ("text", HtmlLinkDestination::Text, false, true),
    ("track", HtmlLinkDestination::Track, true, false),
    ("worker", HtmlLinkDestination::Worker, false, true),
];

#[test]
fn thirteen_union_keywords_match_whole_ascii_case_and_emit_canonical_as() {
    for &(spelling, destination, _, _) in DESTINATIONS {
        let mixed: String = spelling
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i % 2 == 0 {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
            .collect();
        for value in [spelling.to_owned(), spelling.to_ascii_uppercase(), mixed] {
            let raw = attr(KnownHtmlAttribute::As, value);
            let report = interpreter().link_as(&link(), Some(&raw)).unwrap();
            assert_eq!(report.intrinsic(), Some(&destination));
            assert_eq!(report.author(), HtmlAttributeAuthorState::Conforming);
            assert_eq!(
                report.recognition(),
                HtmlAttributeRecognition::Keyword {
                    matched: spelling,
                    canonical: spelling,
                }
            );
            assert_eq!(
                report.context_author(),
                HtmlAttributeContextAuthorState::Unchecked
            );
            assert_eq!(report.raw_attribute(), Some(&raw));
        }
        assert_eq!(destination.canonical_keyword(), spelling);
        assert_eq!(
            destination.into_attribute(),
            attr(KnownHtmlAttribute::As, spelling)
        );
    }
}

#[test]
fn missing_empty_invalid_and_xslt_never_get_an_enum_default() {
    let missing = interpreter()
        .link_as(&link(), None::<&surgeist_html::Attribute>)
        .unwrap();
    assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(missing.recognition(), HtmlAttributeRecognition::Missing);
    assert_eq!(missing.author(), HtmlAttributeAuthorState::Missing);
    assert_eq!(missing.intrinsic(), None);
    assert_eq!(
        missing.context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    for value in [
        "",
        "xslt",
        "bogus",
        " script",
        "script ",
        "script\t",
        "script\0",
        "ſcript",
        "script\u{a0}",
    ] {
        let raw = attr(KnownHtmlAttribute::As, value);
        let report = interpreter().link_as(&link(), Some(&raw)).unwrap();
        assert_eq!(report.intrinsic(), None, "{value:?}");
        assert_eq!(
            report.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword
            )
        );
        assert_eq!(
            report.recognition(),
            if value.is_empty() {
                HtmlAttributeRecognition::Empty
            } else {
                HtmlAttributeRecognition::Unrecognized
            }
        );
        assert_eq!(report.raw_attribute(), Some(&raw));
    }
    let raw = attr(
        KnownHtmlAttribute::As,
        DocumentString::from_utf16(vec![0xd800, 115]),
    );
    let report = interpreter().link_as(&link(), Some(&raw)).unwrap();
    assert_eq!(report.intrinsic(), None);
    assert_eq!(report.raw().unwrap().code_units(), &[0xd800, 115]);
}

#[test]
fn both_rel_domains_are_checked_independently_for_every_keyword() {
    for &(spelling, destination, preload, module) in DESTINATIONS {
        let report = interpreter()
            .link_as_with_attributes(
                &link(),
                &attrs([
                    attr(KnownHtmlAttribute::As, spelling),
                    attr(KnownHtmlAttribute::Rel, "PRELOAD\tmodulepreload"),
                ]),
            )
            .unwrap();
        assert_eq!(report.attribute().intrinsic(), Some(&destination));
        assert_eq!(
            report.attribute().author(),
            HtmlAttributeAuthorState::Conforming
        );
        assert_eq!(
            report.preload_author(),
            if preload {
                HtmlLinkDestinationContextAuthorState::Conforming
            } else {
                context_error(HtmlLinkDestinationContextError::PreloadRequiresPreloadDestination)
            }
        );
        assert_eq!(
            report.modulepreload_author(),
            if module {
                HtmlLinkDestinationContextAuthorState::Conforming
            } else {
                context_error(
                    HtmlLinkDestinationContextError::ModulePreloadRequiresModuleDestination,
                )
            }
        );
        assert_eq!(
            report.other_rel_author(),
            HtmlLinkDestinationContextAuthorState::NotRequired
        );
        assert_eq!(
            report.preload_processing(),
            if preload {
                HtmlLinkDestinationProcessing::Destination(destination)
            } else {
                HtmlLinkDestinationProcessing::Nonmember(destination)
            }
        );
        assert_eq!(
            report.modulepreload_processing(),
            if module {
                HtmlLinkDestinationProcessing::Destination(destination)
            } else {
                HtmlLinkDestinationProcessing::Nonmember(destination)
            }
        );
    }
}

#[test]
fn preload_requires_presence_while_optional_modulepreload_recovers_no_state_to_script() {
    for rel in ["preload", "modulepreload", "preload modulepreload"] {
        let report = interpreter()
            .link_as_with_attributes(&link(), &attrs([attr(KnownHtmlAttribute::Rel, rel)]))
            .unwrap();
        assert_eq!(report.attribute().intrinsic(), None);
        assert_eq!(
            report.attribute().author(),
            HtmlAttributeAuthorState::Missing
        );
        assert_eq!(
            report.preload_author(),
            if rel == "modulepreload" {
                HtmlLinkDestinationContextAuthorState::NotRequired
            } else {
                context_error(HtmlLinkDestinationContextError::PreloadRequiresAs)
            }
        );
        assert_eq!(
            report.modulepreload_author(),
            HtmlLinkDestinationContextAuthorState::NotRequired
        );
        assert_eq!(
            report.preload_processing(),
            if rel == "modulepreload" {
                HtmlLinkDestinationProcessing::NotSelected
            } else {
                HtmlLinkDestinationProcessing::NoState
            }
        );
        assert_eq!(
            report.modulepreload_processing(),
            if rel == "preload" {
                HtmlLinkDestinationProcessing::NotSelected
            } else {
                HtmlLinkDestinationProcessing::ScriptForNoState
            }
        );
    }
}

#[test]
fn invalid_modulepreload_as_recovers_script_without_erasing_author_failures() {
    for value in ["", "unknown", "xslt", " script"] {
        let report = interpreter()
            .link_as_with_attributes(
                &link(),
                &attrs([
                    attr(KnownHtmlAttribute::Rel, "modulepreload"),
                    attr(KnownHtmlAttribute::As, value),
                ]),
            )
            .unwrap();
        assert_eq!(report.attribute().intrinsic(), None);
        assert_eq!(
            report.attribute().author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::UnrecognizedKeyword
            )
        );
        assert_eq!(
            report.modulepreload_author(),
            context_error(HtmlLinkDestinationContextError::ModulePreloadRequiresModuleDestination)
        );
        assert_eq!(
            report.modulepreload_processing(),
            HtmlLinkDestinationProcessing::ScriptForNoState
        );
    }
    let report = interpreter()
        .link_as_with_attributes(
            &link(),
            &attrs([
                attr(KnownHtmlAttribute::Rel, "modulepreload"),
                attr(KnownHtmlAttribute::As, "image"),
            ]),
        )
        .unwrap();
    assert_eq!(
        report.attribute().intrinsic(),
        Some(&HtmlLinkDestination::Image)
    );
    assert_eq!(
        report.attribute().author(),
        HtmlAttributeAuthorState::Conforming
    );
    assert_eq!(
        report.modulepreload_processing(),
        HtmlLinkDestinationProcessing::Nonmember(HtmlLinkDestination::Image)
    );
}

#[test]
fn unrelated_rel_forbids_presence_without_changing_the_as_state() {
    for rel in [
        None,
        Some(""),
        Some("stylesheet"),
        Some("bogus"),
        Some("preload,modulepreload"),
        Some("preload\u{a0}modulepreload"),
    ] {
        let mut values = vec![attr(KnownHtmlAttribute::As, "font")];
        if let Some(rel) = rel {
            values.push(attr(KnownHtmlAttribute::Rel, rel));
        }
        let report = interpreter()
            .link_as_with_attributes(&link(), &attrs(values))
            .unwrap();
        assert_eq!(
            report.attribute().intrinsic(),
            Some(&HtmlLinkDestination::Font)
        );
        assert_eq!(
            report.other_rel_author(),
            context_error(HtmlLinkDestinationContextError::AsRequiresPreloadOrModulePreloadRel)
        );
        assert_eq!(
            report.preload_author(),
            HtmlLinkDestinationContextAuthorState::NotRequired
        );
        assert_eq!(
            report.modulepreload_author(),
            HtmlLinkDestinationContextAuthorState::NotRequired
        );
        assert_eq!(
            report.preload_processing(),
            HtmlLinkDestinationProcessing::NotSelected
        );
        assert_eq!(
            report.modulepreload_processing(),
            HtmlLinkDestinationProcessing::NotSelected
        );
    }
    let absent = interpreter()
        .link_as_with_attributes(&link(), &attrs([]))
        .unwrap();
    assert_eq!(
        absent.other_rel_author(),
        HtmlLinkDestinationContextAuthorState::NotRequired
    );
}

#[test]
fn repeated_ascii_rel_membership_does_not_certify_whole_rel_authoring() {
    let rel = attr(
        KnownHtmlAttribute::Rel,
        "\tpreload\r\nPRELOAD\u{c} modulepreload ",
    );
    let report = interpreter()
        .link_as_with_attributes(
            &link(),
            &attrs([attr(KnownHtmlAttribute::As, "style"), rel.clone()]),
        )
        .unwrap();
    assert_eq!(report.rel_attribute(), Some(&rel));
    assert_eq!(
        report.preload_author(),
        HtmlLinkDestinationContextAuthorState::Conforming
    );
    assert_eq!(
        report.modulepreload_author(),
        HtmlLinkDestinationContextAuthorState::Conforming
    );
}

#[test]
fn projection_is_a_destination_decision_not_a_successful_href_or_fetch_branch() {
    for href in [None, Some(""), Some("bad url\0")] {
        let mut values = vec![attr(KnownHtmlAttribute::Rel, "modulepreload")];
        if let Some(href) = href {
            values.push(attr(KnownHtmlAttribute::Href, href));
        }
        let report = interpreter()
            .link_as_with_attributes(&link(), &attrs(values))
            .unwrap();
        assert_eq!(
            report.modulepreload_processing(),
            HtmlLinkDestinationProcessing::ScriptForNoState
        );
        assert_eq!(report.attribute().intrinsic(), None);
    }
}

#[test]
fn exact_expanded_names_select_slots_and_preserve_prefixes() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "link");
    let actual_as = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "as"),
        "ImAgE",
    );
    let actual_rel = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("r".into()), "rel"),
        "preload",
    );
    let report = interpreter()
        .link_as_with_attributes(
            &element,
            &attrs([
                Attribute::new(
                    QualifiedName::new(Namespace::from_uri("urn:x"), None, "as"),
                    "font",
                ),
                Attribute::new(QualifiedName::new(Namespace::none(), None, "AS"), "worker"),
                actual_as.clone(),
                actual_rel.clone(),
            ]),
        )
        .unwrap();
    assert_eq!(report.attribute().element(), &element);
    assert_eq!(report.attribute().raw_attribute(), Some(&actual_as));
    assert_eq!(report.rel_attribute(), Some(&actual_rel));
    assert_eq!(
        report.preload_author(),
        HtmlLinkDestinationContextAuthorState::Conforming
    );
    let foreign_rel = attrs([
        attr(KnownHtmlAttribute::As, "image"),
        Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:x"), None, "rel"),
            "preload",
        ),
    ]);
    let report = interpreter()
        .link_as_with_attributes(&link(), &foreign_rel)
        .unwrap();
    assert_eq!(report.rel_attribute(), None);
    assert_eq!(
        report.other_rel_author(),
        context_error(HtmlLinkDestinationContextError::AsRequiresPreloadOrModulePreloadRel)
    );
}

#[test]
fn wrong_element_and_wrong_attribute_roles_are_unapplied_without_defaulting() {
    let raw = attr(KnownHtmlAttribute::As, "script");
    for (element, applicability) in [
        (
            KnownHtmlElement::A.qualified_name(),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "LINK"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::svg(), None, "link"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
    ] {
        let report = interpreter()
            .link_as_with_attributes(
                &element,
                &attrs([raw.clone(), attr(KnownHtmlAttribute::Rel, "preload")]),
            )
            .unwrap();
        assert_eq!(report.attribute().applicability(), applicability);
        assert_eq!(report.attribute().intrinsic(), None);
        assert_eq!(
            report.attribute().author(),
            HtmlAttributeAuthorState::Unapplied
        );
        assert_eq!(
            report.preload_author(),
            HtmlLinkDestinationContextAuthorState::Unapplied
        );
        assert_eq!(
            report.modulepreload_processing(),
            HtmlLinkDestinationProcessing::Unapplied
        );
    }
    for name in [
        KnownHtmlAttribute::Rel.qualified_name(),
        QualifiedName::new(Namespace::from_uri("urn:x"), None, "as"),
    ] {
        let wrong = Attribute::new(name, "script");
        let report = interpreter().link_as(&link(), Some(&wrong)).unwrap();
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(report.recognition(), HtmlAttributeRecognition::Unapplied);
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.raw_attribute(), Some(&wrong));
    }
}

#[test]
fn original_slot_caps_precede_role_and_early_rel_success() {
    let limited = HtmlValueInterpreter::new(6);
    let short = attr(KnownHtmlAttribute::As, "script");
    assert!(limited.link_as(&link(), Some(&short)).is_ok());
    let long = attr(KnownHtmlAttribute::As, "script ");
    assert!(
        limited
            .link_as(&KnownHtmlElement::Div.qualified_name(), Some(&long))
            .is_err()
    );
    for element in [link(), KnownHtmlElement::Div.qualified_name()] {
        let long_rel = attrs([short.clone(), attr(KnownHtmlAttribute::Rel, "preload ")]);
        assert!(
            limited
                .link_as_with_attributes(&element, &long_rel)
                .is_err()
        );
        let long_as = attrs([long.clone(), attr(KnownHtmlAttribute::Rel, "")]);
        assert!(limited.link_as_with_attributes(&element, &long_as).is_err());
    }
    let report = limited
        .link_as_with_attributes(
            &link(),
            &attrs([
                attr(KnownHtmlAttribute::As, "image"),
                attr(KnownHtmlAttribute::Rel, ""),
            ]),
        )
        .unwrap();
    assert_eq!(report.rel_attribute().unwrap().value().len(), 0);
}

#[test]
fn source_provenance_names_the_html_rule_and_the_selected_fetch_subset() {
    let report = interpreter()
        .link_as_with_attributes(&link(), &attrs([]))
        .unwrap();
    let base =
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#";
    assert_eq!(
        report.attribute().source_clause(),
        format!("{base}attr-link-as")
    );
    assert_eq!(report.attribute().inventory_rows(), &[10]);
    assert_eq!(
        report.attribute().vocabulary_source_clause(),
        Some(
            "https://fetch.spec.whatwg.org/commit-snapshots/e9460d1b22ead1c6a72eeac37fb2ec0dc01aeb1d/#request-destination-script-like"
        )
    );
    assert_eq!(report.rel_source_clause(), format!("{base}attr-link-rel"));
    assert_eq!(
        report.preload_source_clause(),
        format!("{base}preload-destination")
    );
    assert_eq!(
        report.modulepreload_source_clause(),
        format!("{base}module-preload-destination")
    );
    assert_eq!(
        report.modulepreload_processing_source_clause(),
        format!("{base}link-type-modulepreload")
    );
    assert_eq!(
        report.preload_processing_source_clause(),
        format!("{base}link-type-preload")
    );
}

#[test]
fn invalid_dual_rel_retains_both_context_failures_and_distinct_processing() {
    let report = interpreter()
        .link_as_with_attributes(
            &link(),
            &attrs([
                attr(KnownHtmlAttribute::As, "xslt"),
                attr(KnownHtmlAttribute::Rel, "preload modulepreload"),
            ]),
        )
        .unwrap();
    assert_eq!(
        report.attribute().author(),
        HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::UnrecognizedKeyword)
    );
    assert_eq!(
        report.preload_author(),
        context_error(HtmlLinkDestinationContextError::PreloadRequiresPreloadDestination)
    );
    assert_eq!(
        report.modulepreload_author(),
        context_error(HtmlLinkDestinationContextError::ModulePreloadRequiresModuleDestination)
    );
    assert_eq!(
        report.preload_processing(),
        HtmlLinkDestinationProcessing::NoState
    );
    assert_eq!(
        report.modulepreload_processing(),
        HtmlLinkDestinationProcessing::ScriptForNoState
    );
}

#[test]
fn exact_rel_cap_passes_and_unselected_foreign_slots_are_not_admitted_as_real_slots() {
    let limited = HtmlValueInterpreter::new(7);
    let values = attrs([
        attr(KnownHtmlAttribute::As, "image"),
        attr(KnownHtmlAttribute::Rel, "preload"),
        Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:x"), None, "as"),
            "long foreign as",
        ),
        Attribute::new(
            QualifiedName::new(Namespace::none(), None, "REL"),
            "long uppercase rel",
        ),
    ]);
    let report = limited.link_as_with_attributes(&link(), &values).unwrap();
    assert_eq!(
        report.preload_author(),
        HtmlLinkDestinationContextAuthorState::Conforming
    );
    for element in [link(), KnownHtmlElement::Div.qualified_name()] {
        assert!(
            limited
                .link_as_with_attributes(
                    &element,
                    &attrs([
                        attr(KnownHtmlAttribute::As, "image"),
                        attr(KnownHtmlAttribute::Rel, "preload "),
                    ])
                )
                .is_err()
        );
    }
}
