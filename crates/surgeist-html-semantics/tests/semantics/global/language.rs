use crate::semantics::support::{
    compact_builder_without_registries as builder,
    small_semantic_limits_with_integer_digits as limits,
};
use surgeist_html::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureError,
    HtmlCaptureRegistryInput, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlControlCurrentValue, HtmlDirectionality, HtmlDocumentLanguageFallback, HtmlLanguage,
    HtmlLanguageDirectionError, HtmlLanguageDirectionFacts, HtmlLanguageDirectionLimits,
    HtmlLanguageOrigin, HtmlPragmaLanguage, HtmlProtocolLanguage, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticValue,
    HtmlSlotAssignedNodes,
};

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    let attributes = OrderedAttributes::new(attrs.iter().map(|(name, value)| {
        Attribute::new(QualifiedName::new(Namespace::none(), None, *name), *value)
    }))
    .unwrap();
    expanded(b, d, Namespace::html(), name, attributes)
}
fn expanded(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    name: &str,
    attributes: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, name),
        attributes,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn text(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    value: impl Into<DocumentString>,
) -> HtmlCaptureBuilderSubject {
    b.add_text(d, value.into()).unwrap()
}
fn supplement(
    c: &HtmlCapture,
    fallback: Vec<HtmlDocumentLanguageFallback>,
    values: Vec<HtmlControlCurrentValue>,
    assignments: Vec<HtmlSlotAssignedNodes>,
) -> HtmlLanguageDirectionFacts {
    HtmlLanguageDirectionFacts::new(
        c,
        c.host_facts_revision().clone(),
        fallback,
        values,
        assignments,
        HtmlLanguageDirectionLimits::new(100, 100, 10000, 1000000),
    )
    .unwrap()
}
fn context(c: &HtmlCapture, facts: Option<HtmlLanguageDirectionFacts>) -> HtmlSemanticContext {
    let mut input = HtmlSemanticContextInput::new(vec![]);
    if let Some(facts) = facts {
        input = input.with_language_direction(facts);
    }
    HtmlSemanticContext::new(c, c.host_facts_revision().clone(), input, 0).unwrap()
}
fn query(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    node: &HtmlCaptureBuilderSubject,
    kind: HtmlSemanticQuery,
) -> HtmlSemanticReport {
    HtmlSemanticSession::new(
        c,
        cx,
        HtmlValueInterpreter::new(10000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(c.subject_id(node).unwrap(), kind)])
    .unwrap()
}
fn language(report: &HtmlSemanticReport) -> &HtmlLanguage {
    match report.results()[0].fact() {
        HtmlSemanticFact::Language(HtmlSemanticValue::Known(value)) => value,
        other => panic!("expected language, got {other:?}"),
    }
}
fn direction(report: &HtmlSemanticReport) -> HtmlDirectionality {
    match report.results()[0].fact() {
        HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(value)) => *value,
        other => panic!("expected direction, got {other:?}"),
    }
}
fn shadow(
    b: &mut HtmlCaptureBuilder,
    document: &HtmlCaptureBuilderSubject,
    host: &HtmlCaptureBuilderSubject,
    mode: HtmlSlotAssignment,
) -> HtmlCaptureBuilderSubject {
    b.add_shadow_root(
        document,
        host,
        HtmlCapturedShadowProperties::new(
            HtmlShadowRootProperties::new(HtmlShadowRootMode::Closed, mode, false, false, false),
            false,
            false,
            false,
        ),
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}

#[test]
fn xml_expanded_name_wins_on_foreign_element_with_exact_tag_and_prefix() {
    let (mut b, d) = builder();
    let attrs = OrderedAttributes::new([
        Attribute::new(QualifiedName::new(Namespace::none(), None, "lang"), "en"),
        Attribute::new(
            QualifiedName::new(
                Namespace::from_uri("http://www.w3.org/XML/1998/namespace"),
                Some("unusual".into()),
                "lang",
            ),
            DocumentString::from_utf16(vec![32, 0xd800, 0, 90]),
        ),
    ])
    .unwrap();
    let e = expanded(&mut b, &d, Namespace::mathml(), "mi", attrs);
    let c = b.finish().unwrap();
    let report = query(&c, &context(&c, None), &e, HtmlSemanticQuery::Language);
    assert_eq!(language(&report).tag().code_units(), [32, 0xd800, 0, 90]);
    assert_eq!(
        language(&report).origin(),
        &HtmlLanguageOrigin::XmlAttribute(c.subject_id(&e).unwrap())
    );
    assert!(!language(&report).is_explicitly_unknown());
}

#[test]
fn svg_lang_applies_literal_xml_lang_does_not_and_empty_stops_inheritance() {
    let (mut b, d) = builder();
    let parent = element(&mut b, &d, "div", &[("lang", "fr")]);
    let svg = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "svg",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "lang"),
            "",
        )])
        .unwrap(),
    );
    let literal = element(&mut b, &d, "span", &[("xml:lang", "de")]);
    let foreign = expanded(
        &mut b,
        &d,
        Namespace::mathml(),
        "mi",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "lang"),
            "ar",
        )])
        .unwrap(),
    );
    b.establish_children(&parent, &[svg.clone(), literal.clone(), foreign.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    let report = query(&c, &cx, &svg, HtmlSemanticQuery::Language);
    assert!(language(&report).is_explicitly_unknown());
    assert_eq!(
        language(&report).origin(),
        &HtmlLanguageOrigin::LangAttribute(c.subject_id(&svg).unwrap())
    );
    for child in [&literal, &foreign] {
        assert_eq!(
            language(&query(&c, &cx, child, HtmlSemanticQuery::Language))
                .tag()
                .code_units(),
            [102, 114]
        );
    }
}

#[test]
fn language_shadow_inherits_host_for_text_and_preserves_original_source() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[("lang", "XYzzy")]);
    let root = shadow(&mut b, &d, &host, HtmlSlotAssignment::Named);
    let child = text(&mut b, &d, "text");
    b.establish_children(&root, std::slice::from_ref(&child))
        .unwrap();
    let c = b.finish().unwrap();
    let report = query(&c, &context(&c, None), &child, HtmlSemanticQuery::Language);
    assert_eq!(language(&report).tag().to_utf8().unwrap(), "XYzzy");
    assert_eq!(
        language(&report).origin(),
        &HtmlLanguageOrigin::LangAttribute(c.subject_id(&host).unwrap())
    );
}

#[test]
fn fallback_pragma_wins_without_protocol_knowledge_and_protocol_absence_is_unknown() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "p", &[]);
    b.establish_children(&d, std::slice::from_ref(&e)).unwrap();
    let c = b.finish().unwrap();
    let document = c.subject_id(&d).unwrap();
    for (pragma, protocol, expected, origin) in [
        (
            HtmlPragmaLanguage::Tag("EN-x-private".into()),
            HtmlProtocolLanguage::Unavailable,
            "EN-x-private",
            HtmlLanguageOrigin::Pragma(document.clone()),
        ),
        (
            HtmlPragmaLanguage::Absent,
            HtmlProtocolLanguage::Single("zz-invalid".into()),
            "zz-invalid",
            HtmlLanguageOrigin::Protocol(document.clone()),
        ),
        (
            HtmlPragmaLanguage::Absent,
            HtmlProtocolLanguage::Absent,
            "",
            HtmlLanguageOrigin::NoInformation(document.clone()),
        ),
        (
            HtmlPragmaLanguage::Absent,
            HtmlProtocolLanguage::Multiple,
            "",
            HtmlLanguageOrigin::NoInformation(document.clone()),
        ),
    ] {
        let cx = context(
            &c,
            Some(supplement(
                &c,
                vec![HtmlDocumentLanguageFallback::new(
                    document.clone(),
                    pragma,
                    protocol,
                )],
                vec![],
                vec![],
            )),
        );
        let report = query(&c, &cx, &e, HtmlSemanticQuery::Language);
        assert_eq!(language(&report).tag().to_utf8().unwrap(), expected);
        assert_eq!(language(&report).origin(), &origin);
    }
}

#[test]
fn unavailable_fallback_and_parent_remain_missing_not_unknown() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "p", &[]);
    let c = b.finish().unwrap();
    assert!(matches!(
        query(&c, &context(&c, None), &e, HtmlSemanticQuery::Language).results()[0].fact(),
        HtmlSemanticFact::Language(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(_)))
    ));
    assert!(matches!(
        query(&c, &context(&c, None), &d, HtmlSemanticQuery::Language).results()[0].fact(),
        HtmlSemanticFact::Language(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::LanguageFallback(_)
        ))
    ));
}

#[test]
fn explicit_direction_is_ascii_insensitive_foreign_dir_inherits_and_tel_defaults_ltr() {
    let (mut b, d) = builder();
    let p = element(&mut b, &d, "div", &[("dir", "RtL")]);
    let foreign = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "g",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "dir"),
            "ltr",
        )])
        .unwrap(),
    );
    let tel = element(&mut b, &d, "input", &[("type", "TEL")]);
    let invalid = element(&mut b, &d, "span", &[("dir", " ltr ")]);
    b.establish_children(&p, &[foreign.clone(), tel.clone(), invalid.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    for child in [&p, &foreign, &invalid] {
        assert_eq!(
            direction(&query(&c, &cx, child, HtmlSemanticQuery::Directionality)),
            HtmlDirectionality::Rtl
        );
    }
    assert_eq!(
        direction(&query(&c, &cx, &tel, HtmlSemanticQuery::Directionality)),
        HtmlDirectionality::Ltr
    );
}

#[test]
fn auto_scans_first_strong_and_excludes_entire_special_and_directed_subtrees() {
    let (mut b, d) = builder();
    let p = element(&mut b, &d, "div", &[("dir", "auto")]);
    let mut children = vec![];
    for name in ["bdi", "script", "style", "textarea", "span"] {
        let e = element(
            &mut b,
            &d,
            name,
            if name == "span" {
                &[("dir", "rtl")]
            } else {
                &[]
            },
        );
        let rtl = text(&mut b, &d, "א");
        b.establish_children(&e, &[rtl]).unwrap();
        children.push(e);
    }
    let neutral = text(&mut b, &d, "123 \u{200e}"); // U+200E is L, despite no letter shape.
    children.push(neutral);
    let later = text(&mut b, &d, "ع");
    children.push(later);
    b.establish_children(&p, &children).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        direction(&query(
            &c,
            &context(&c, None),
            &p,
            HtmlSemanticQuery::Directionality
        )),
        HtmlDirectionality::Ltr
    );
}

#[test]
fn undefined_bdi_scans_own_content_and_neutral_auto_defaults_ltr() {
    let (mut b, d) = builder();
    let bdi = element(&mut b, &d, "bdi", &[]);
    let t = text(&mut b, &d, "  ع");
    b.establish_children(&bdi, &[t]).unwrap();
    let p = element(&mut b, &d, "div", &[("dir", "auto")]);
    let neutral = text(&mut b, &d, "123! ");
    b.establish_children(&p, &[neutral]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    assert_eq!(
        direction(&query(&c, &cx, &bdi, HtmlSemanticQuery::Directionality)),
        HtmlDirectionality::Rtl
    );
    assert_eq!(
        direction(&query(&c, &cx, &p, HtmlSemanticQuery::Directionality)),
        HtmlDirectionality::Ltr
    );
}

#[test]
fn current_value_overrides_markup_for_all_ten_input_states_and_textarea() {
    let (mut b, d) = builder();
    let mut controls = vec![];
    for state in [
        "hidden", "text", "search", "tel", "url", "email", "password", "submit", "reset", "button",
    ] {
        controls.push(element(
            &mut b,
            &d,
            "input",
            &[("dir", "auto"), ("type", state), ("value", "English")],
        ));
    }
    let textarea = element(&mut b, &d, "textarea", &[("dir", "auto")]);
    let default = text(&mut b, &d, "English");
    b.establish_children(&textarea, &[default]).unwrap();
    controls.push(textarea);
    let c = b.finish().unwrap();
    let values = controls
        .iter()
        .map(|e| HtmlControlCurrentValue::new(c.subject_id(e).unwrap(), Some("ع".into())))
        .collect();
    let cx = context(&c, Some(supplement(&c, vec![], values, vec![])));
    for e in controls {
        assert_eq!(
            direction(&query(&c, &cx, &e, HtmlSemanticQuery::Directionality)),
            HtmlDirectionality::Rtl
        );
    }
}

#[test]
fn control_empty_neutral_first_strong_and_missing_are_distinct() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "input", &[("dir", "auto")]);
    let c = b.finish().unwrap();
    for (value, expected) in [
        (Some(""), Some(HtmlDirectionality::Ltr)),
        (Some("123"), Some(HtmlDirectionality::Ltr)),
        (Some("aع"), Some(HtmlDirectionality::Ltr)),
        (Some("عa"), Some(HtmlDirectionality::Rtl)),
        (None, None),
    ] {
        let cx = context(
            &c,
            Some(supplement(
                &c,
                vec![],
                vec![HtmlControlCurrentValue::new(
                    c.subject_id(&e).unwrap(),
                    value.map(DocumentString::from),
                )],
                vec![],
            )),
        );
        let report = query(&c, &cx, &e, HtmlSemanticQuery::Directionality);
        if let Some(expected) = expected {
            assert_eq!(direction(&report), expected);
        } else {
            assert!(matches!(
                report.results()[0].fact(),
                HtmlSemanticFact::Directionality(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::CurrentValue(_)
                ))
            ));
        }
    }
}

#[test]
fn non_control_auto_scans_ordinary_content_and_never_requires_current_value() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "input", &[("dir", "auto"), ("type", "number")]);
    b.establish_children(&e, &[]).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        direction(&query(
            &c,
            &context(&c, None),
            &e,
            HtmlSemanticQuery::Directionality
        )),
        HtmlDirectionality::Ltr
    );
}

#[test]
fn utf16_surrogates_and_nonbmp_first_strong_use_selected_unicode_properties() {
    let (mut b, d) = builder();
    let mut nodes = vec![];
    for units in [
        vec![0xd800, 0x05d0],
        vec![0xdc00, 0x05d0],
        vec![0xd83a, 0xdd00, 0x61],
        vec![0x05d0, 0xd800],
    ] {
        // U+1E900 Adlam is R.
        let e = element(&mut b, &d, "p", &[("dir", "auto")]);
        let t = text(&mut b, &d, DocumentString::from_utf16(units));
        b.establish_children(&e, &[t]).unwrap();
        nodes.push(e);
    }
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    for (e, expected) in nodes.iter().zip([
        HtmlDirectionality::Ltr,
        HtmlDirectionality::Ltr,
        HtmlDirectionality::Rtl,
        HtmlDirectionality::Rtl,
    ]) {
        assert_eq!(
            direction(&query(&c, &cx, e, HtmlSemanticQuery::Directionality)),
            expected
        );
    }
}

#[test]
fn shadow_inheritance_and_contained_slot_return_host_directionality() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[("dir", "rtl")]);
    let root = shadow(&mut b, &d, &host, HtmlSlotAssignment::Named);
    let auto = element(&mut b, &d, "div", &[("dir", "auto")]);
    let slot = element(&mut b, &d, "slot", &[]);
    let later = text(&mut b, &d, "English");
    b.establish_children(&auto, &[slot, later]).unwrap();
    let inherited = element(&mut b, &d, "span", &[]);
    b.establish_children(&root, &[auto.clone(), inherited.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    for e in [&auto, &inherited] {
        assert_eq!(
            direction(&query(&c, &cx, e, HtmlSemanticQuery::Directionality)),
            HtmlDirectionality::Rtl
        );
    }
}

#[test]
fn manual_slot_assignment_order_is_preserved_and_assigned_root_exclusion_applies() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[]);
    let a = text(&mut b, &d, "a");
    let rtl = text(&mut b, &d, "א");
    let excluded = element(&mut b, &d, "bdi", &[]); // No children required for the excluded root.
    b.establish_children(&host, &[a.clone(), rtl.clone(), excluded.clone()])
        .unwrap();
    let root = shadow(&mut b, &d, &host, HtmlSlotAssignment::Manual);
    let slot = element(&mut b, &d, "slot", &[("dir", "auto")]);
    b.establish_children(&root, std::slice::from_ref(&slot))
        .unwrap();
    let c = b.finish().unwrap();
    let assigned = vec![
        c.subject_id(&excluded).unwrap(),
        c.subject_id(&rtl).unwrap(),
        c.subject_id(&a).unwrap(),
    ];
    let cx = context(
        &c,
        Some(supplement(
            &c,
            vec![],
            vec![],
            vec![HtmlSlotAssignedNodes::new(
                c.subject_id(&slot).unwrap(),
                Some(assigned),
            )],
        )),
    );
    assert_eq!(
        direction(&query(&c, &cx, &slot, HtmlSemanticQuery::Directionality)),
        HtmlDirectionality::Rtl
    );
}

#[test]
fn empty_assignment_uses_fallback_children_unavailable_is_missing() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[]);
    b.establish_children(&host, &[]).unwrap();
    let root = shadow(&mut b, &d, &host, HtmlSlotAssignment::Named);
    let slot = element(&mut b, &d, "slot", &[("dir", "auto")]);
    let t = text(&mut b, &d, "א");
    b.establish_children(&slot, &[t]).unwrap();
    b.establish_children(&root, std::slice::from_ref(&slot))
        .unwrap();
    let c = b.finish().unwrap();
    for nodes in [Some(vec![]), None] {
        let known = nodes.is_some();
        let cx = context(
            &c,
            Some(supplement(
                &c,
                vec![],
                vec![],
                vec![HtmlSlotAssignedNodes::new(
                    c.subject_id(&slot).unwrap(),
                    nodes,
                )],
            )),
        );
        let r = query(&c, &cx, &slot, HtmlSemanticQuery::Directionality);
        if known {
            assert_eq!(direction(&r), HtmlDirectionality::Rtl);
        } else {
            assert!(matches!(
                r.results()[0].fact(),
                HtmlSemanticFact::Directionality(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::SlotAssignment(_)
                ))
            ));
        }
    }
}

#[test]
fn assignment_all_excluded_defaults_ltr_without_using_slot_fallback() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[]);
    let excluded = element(&mut b, &d, "span", &[("dir", "ltr")]);
    b.establish_children(&host, std::slice::from_ref(&excluded))
        .unwrap();
    let root = shadow(&mut b, &d, &host, HtmlSlotAssignment::Named);
    let slot = element(&mut b, &d, "slot", &[("dir", "auto")]);
    let t = text(&mut b, &d, "א");
    b.establish_children(&slot, &[t]).unwrap();
    b.establish_children(&root, std::slice::from_ref(&slot))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(supplement(
            &c,
            vec![],
            vec![],
            vec![HtmlSlotAssignedNodes::new(
                c.subject_id(&slot).unwrap(),
                Some(vec![c.subject_id(&excluded).unwrap()]),
            )],
        )),
    );
    assert_eq!(
        direction(&query(&c, &cx, &slot, HtmlSemanticQuery::Directionality)),
        HtmlDirectionality::Ltr
    );
}

#[test]
fn missing_child_after_first_strong_does_not_erase_proved_direction() {
    let (mut b, d) = builder();
    let auto = element(&mut b, &d, "div", &[("dir", "auto")]);
    let t = text(&mut b, &d, "א");
    let unknown = element(&mut b, &d, "section", &[]);
    b.establish_children(&auto, &[t, unknown]).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        direction(&query(
            &c,
            &context(&c, None),
            &auto,
            HtmlSemanticQuery::Directionality
        )),
        HtmlDirectionality::Rtl
    );
}

#[test]
fn language_facts_reject_foreign_ids_wrong_kinds_host_revision_and_empty_pragma() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "div", &[]);
    let c = b.finish().unwrap();
    let (other_b, other_d) = builder();
    let other = other_b.finish().unwrap();
    let caps = HtmlLanguageDirectionLimits::new(10, 10, 100, 100000);
    let foreign = HtmlDocumentLanguageFallback::new(
        other.subject_id(&other_d).unwrap(),
        HtmlPragmaLanguage::Absent,
        HtmlProtocolLanguage::Absent,
    );
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![foreign],
            vec![],
            vec![],
            caps
        ),
        Err(HtmlLanguageDirectionError::Capture(
            HtmlCaptureError::ForeignCapture
        ))
    ));
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &c,
            HtmlHostFactsRevision::fresh(),
            vec![],
            vec![],
            vec![],
            caps
        ),
        Err(HtmlLanguageDirectionError::HostRevisionMismatch)
    ));
    let wrong = HtmlDocumentLanguageFallback::new(
        c.subject_id(&e).unwrap(),
        HtmlPragmaLanguage::Absent,
        HtmlProtocolLanguage::Absent,
    );
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![wrong],
            vec![],
            vec![],
            caps
        ),
        Err(HtmlLanguageDirectionError::Capture(
            HtmlCaptureError::WrongKind { .. }
        ))
    ));
    let empty = HtmlDocumentLanguageFallback::new(
        c.subject_id(&d).unwrap(),
        HtmlPragmaLanguage::Tag("".into()),
        HtmlProtocolLanguage::Absent,
    );
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![empty],
            vec![],
            vec![],
            caps
        ),
        Err(HtmlLanguageDirectionError::EmptyPragma(_))
    ));
    let value = HtmlControlCurrentValue::new(c.subject_id(&e).unwrap(), Some("abc".into()));
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            vec![value],
            vec![],
            caps
        ),
        Err(HtmlLanguageDirectionError::InapplicableCurrentValue(_))
    ));
}

#[test]
fn input_constructor_bounds_moved_capacity_units_work_and_duplicates() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "textarea", &[]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let mut records = Vec::with_capacity(8);
    records.push(HtmlControlCurrentValue::new(id.clone(), Some("abc".into())));
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            records,
            vec![],
            HtmlLanguageDirectionLimits::new(7, 0, 100, 100000)
        ),
        Err(HtmlLanguageDirectionError::RecordCapacity {
            limit: 7,
            required: 8
        })
    ));
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            vec![HtmlControlCurrentValue::new(id.clone(), Some("abc".into()))],
            vec![],
            HtmlLanguageDirectionLimits::new(1, 0, 2, 100000)
        ),
        Err(HtmlLanguageDirectionError::CodeUnits {
            limit: 2,
            required: 3
        })
    ));
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            vec![HtmlControlCurrentValue::new(id.clone(), None)],
            vec![],
            HtmlLanguageDirectionLimits::new(1, 0, 0, 0)
        ),
        Err(HtmlLanguageDirectionError::ValidationWork { limit: 0, .. })
    ));
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            vec![
                HtmlControlCurrentValue::new(id.clone(), None),
                HtmlControlCurrentValue::new(id, None)
            ],
            vec![],
            HtmlLanguageDirectionLimits::new(2, 0, 0, 100000)
        ),
        Err(HtmlLanguageDirectionError::DuplicateSubject(_))
    ));
}

#[test]
fn named_assignment_rejects_reordered_duplicate_foreign_and_wrong_parent_nodes() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[]);
    let a = text(&mut b, &d, "a");
    let rtl = text(&mut b, &d, "א");
    b.establish_children(&host, &[a.clone(), rtl.clone()])
        .unwrap();
    let outside = element(&mut b, &d, "section", &[]);
    let outsider = text(&mut b, &d, "foreign parent");
    b.establish_children(&outside, std::slice::from_ref(&outsider))
        .unwrap();
    let root = shadow(&mut b, &d, &host, HtmlSlotAssignment::Named);
    let slot = element(&mut b, &d, "slot", &[]);
    let second = element(&mut b, &d, "slot", &[]);
    b.establish_children(&root, &[slot.clone(), second.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let sid = c.subject_id(&slot).unwrap();
    let aid = c.subject_id(&a).unwrap();
    let rid = c.subject_id(&rtl).unwrap();
    let caps = HtmlLanguageDirectionLimits::new(10, 10, 0, 100000);
    let check = |nodes| {
        HtmlLanguageDirectionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            vec![],
            vec![HtmlSlotAssignedNodes::new(sid.clone(), Some(nodes))],
            caps,
        )
    };
    assert!(matches!(
        check(vec![rid, aid.clone()]),
        Err(HtmlLanguageDirectionError::AssignmentOrder(_))
    ));
    assert!(matches!(
        check(vec![aid.clone(), aid.clone()]),
        Err(HtmlLanguageDirectionError::DuplicateAssignment(_))
    ));
    assert!(matches!(
        check(vec![c.subject_id(&outsider).unwrap()]),
        Err(HtmlLanguageDirectionError::InvalidAssignedNode(_))
    ));
    let assignments = vec![
        HtmlSlotAssignedNodes::new(sid, Some(vec![aid.clone()])),
        HtmlSlotAssignedNodes::new(c.subject_id(&second).unwrap(), Some(vec![aid])),
    ];
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            vec![],
            assignments,
            caps
        ),
        Err(HtmlLanguageDirectionError::DuplicateAssignment(_))
    ));
}

#[test]
fn known_empty_assignment_does_not_require_uninspected_host_children() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[]);
    let root = shadow(&mut b, &d, &host, HtmlSlotAssignment::Named);
    let slot = element(&mut b, &d, "slot", &[("dir", "auto")]);
    b.establish_children(&slot, &[]).unwrap();
    b.establish_children(&root, std::slice::from_ref(&slot))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(supplement(
            &c,
            vec![],
            vec![],
            vec![HtmlSlotAssignedNodes::new(
                c.subject_id(&slot).unwrap(),
                Some(vec![]),
            )],
        )),
    );
    assert_eq!(
        direction(&query(&c, &cx, &slot, HtmlSemanticQuery::Directionality)),
        HtmlDirectionality::Ltr
    );
}

#[test]
fn assigned_capacity_cap_applies_before_node_inspection() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[]);
    let root = shadow(&mut b, &d, &host, HtmlSlotAssignment::Manual);
    let slot = element(&mut b, &d, "slot", &[]);
    b.establish_children(&root, std::slice::from_ref(&slot))
        .unwrap();
    let c = b.finish().unwrap();
    let nodes = Vec::with_capacity(4);
    assert!(matches!(
        HtmlLanguageDirectionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            vec![],
            vec![HtmlSlotAssignedNodes::new(
                c.subject_id(&slot).unwrap(),
                Some(nodes)
            )],
            HtmlLanguageDirectionLimits::new(1, 3, 0, 100000)
        ),
        Err(HtmlLanguageDirectionError::AssignedNodeCapacity {
            limit: 3,
            required: 4
        })
    ));
}

#[test]
fn session_work_scratch_bytes_cancellation_and_failed_batches_are_qualified_and_sticky() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "div", &[("dir", "auto")]);
    let t = text(&mut b, &d, "123א");
    b.establish_children(&e, &[t]).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    let request =
        HtmlSemanticRequest::new(c.subject_id(&e).unwrap(), HtmlSemanticQuery::Directionality);
    let make = |l, cancel| {
        HtmlSemanticSession::new(&c, &cx, HtmlValueInterpreter::new(100), l, cancel).unwrap()
    };
    let report = make(limits(), HtmlSemanticCancellation::new())
        .query(std::slice::from_ref(&request))
        .unwrap();
    let work = report.usage().work();
    assert_eq!(report.usage().scratch_records(), 0);
    assert!(report.usage().peak_scratch_records() > 0);
    let mut short = make(
        HtmlSemanticLimits::new(work - 1, 100, 1000, 1000, 1000000, 1000),
        HtmlSemanticCancellation::new(),
    );
    let failure = short.query(std::slice::from_ref(&request)).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert!(failure.qualification().is_some());
    assert_eq!(short.usage().scratch_records(), 0);
    assert_eq!(short.query(&[]).unwrap_err(), failure);
    let exact = make(
        HtmlSemanticLimits::new(work, 100, 1000, 1000, 1000000, 1000),
        HtmlSemanticCancellation::new(),
    )
    .query(std::slice::from_ref(&request))
    .unwrap();
    assert_eq!(direction(&exact), HtmlDirectionality::Rtl);
    for (l, resource) in [
        (
            HtmlSemanticLimits::new(1000000, 100, 0, 1000, 1000000, 1000),
            HtmlSemanticResource::ScratchRecords,
        ),
        (
            HtmlSemanticLimits::new(1000000, 100, 1000, 1000, 0, 1000),
            HtmlSemanticResource::AllocatedBytes,
        ),
    ] {
        let mut limited = make(l, HtmlSemanticCancellation::new());
        let failure = limited.query(std::slice::from_ref(&request)).unwrap_err();
        assert!(
            matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource)
        );
        assert_eq!(limited.usage().scratch_records(), 0);
    }
    let cancel = HtmlSemanticCancellation::new();
    let mut session = make(limits(), cancel.clone());
    let preserved = session.query(std::slice::from_ref(&request)).unwrap();
    cancel.cancel();
    let failure = session.query(&[request.clone(), request]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(direction(&preserved), HtmlDirectionality::Rtl);
    assert_eq!(session.query(&[]).unwrap_err(), failure);
}

#[test]
fn selected_tag_admission_and_foreign_query_failure_do_not_return_partial_batch() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "div", &[("lang", "unmodified")]);
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    let request = HtmlSemanticRequest::new(c.subject_id(&e).unwrap(), HtmlSemanticQuery::Language);
    let mut limited = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(9),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    assert!(matches!(
        limited
            .query(std::slice::from_ref(&request))
            .unwrap_err()
            .kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    let (b2, d2) = builder();
    let other = b2.finish().unwrap();
    let foreign =
        HtmlSemanticRequest::new(other.subject_id(&d2).unwrap(), HtmlSemanticQuery::Language);
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(10),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let failure = session.query(&[request, foreign]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(HtmlCaptureError::ForeignCapture)
    ));
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    assert_eq!(session.query(&[]).unwrap_err(), failure);
}

#[test]
fn direction_on_text_is_inapplicable_but_language_still_selects_ancestor() {
    let (mut b, d) = builder();
    let parent = element(&mut b, &d, "div", &[("lang", "en")]);
    let t = text(&mut b, &d, "א");
    b.establish_children(&parent, std::slice::from_ref(&t))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    assert!(matches!(
        query(&c, &cx, &t, HtmlSemanticQuery::Directionality).results()[0].fact(),
        HtmlSemanticFact::Directionality(HtmlSemanticValue::Inapplicable)
    ));
    assert_eq!(
        language(&query(&c, &cx, &t, HtmlSemanticQuery::Language))
            .tag()
            .to_utf8()
            .unwrap(),
        "en"
    );
}

#[test]
fn language_slot_uses_shadow_host_without_consulting_assignment_or_control_facts() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[("lang", "host-tag")]);
    let assigned = element(&mut b, &d, "span", &[("lang", "assigned-tag")]);
    b.establish_children(&host, std::slice::from_ref(&assigned))
        .unwrap();
    let root = shadow(&mut b, &d, &host, HtmlSlotAssignment::Named);
    let slot = element(&mut b, &d, "slot", &[]);
    b.establish_children(&root, std::slice::from_ref(&slot))
        .unwrap();
    let c = b.finish().unwrap();
    let report = query(&c, &context(&c, None), &slot, HtmlSemanticQuery::Language);
    assert_eq!(language(&report).tag().to_utf8().unwrap(), "host-tag");
    assert_eq!(
        language(&report).origin(),
        &HtmlLanguageOrigin::LangAttribute(c.subject_id(&host).unwrap())
    );
}

#[test]
fn unavailable_pragma_does_not_skip_to_a_known_protocol_and_missing_parent_is_not_ltr() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "div", &[]);
    let c = b.finish().unwrap();
    let facts = supplement(
        &c,
        vec![HtmlDocumentLanguageFallback::new(
            c.subject_id(&d).unwrap(),
            HtmlPragmaLanguage::Unavailable,
            HtmlProtocolLanguage::Single("en".into()),
        )],
        vec![],
        vec![],
    );
    let cx = context(&c, Some(facts));
    assert!(matches!(
        query(&c, &cx, &d, HtmlSemanticQuery::Language).results()[0].fact(),
        HtmlSemanticFact::Language(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::LanguageFallback(_)
        ))
    ));
    assert!(matches!(
        query(&c, &cx, &e, HtmlSemanticQuery::Directionality).results()[0].fact(),
        HtmlSemanticFact::Directionality(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(
            _
        )))
    ));
}

#[test]
fn assigned_neutral_element_continues_to_later_text_without_ltr_substitution() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[]);
    let neutral = element(&mut b, &d, "span", &[]);
    let neutral_text = text(&mut b, &d, "123!");
    b.establish_children(&neutral, &[neutral_text]).unwrap();
    let rtl = text(&mut b, &d, "א");
    b.establish_children(&host, &[neutral.clone(), rtl.clone()])
        .unwrap();
    let root = shadow(&mut b, &d, &host, HtmlSlotAssignment::Named);
    let slot = element(&mut b, &d, "slot", &[("dir", "auto")]);
    b.establish_children(&root, std::slice::from_ref(&slot))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(supplement(
            &c,
            vec![],
            vec![],
            vec![HtmlSlotAssignedNodes::new(
                c.subject_id(&slot).unwrap(),
                Some(vec![
                    c.subject_id(&neutral).unwrap(),
                    c.subject_id(&rtl).unwrap(),
                ]),
            )],
        )),
    );
    assert_eq!(
        direction(&query(&c, &cx, &slot, HtmlSemanticQuery::Directionality)),
        HtmlDirectionality::Rtl
    );
}

#[test]
fn contained_shadow_slot_can_evaluate_an_auto_host_iteratively() {
    let (mut b, d) = builder();
    let host = element(&mut b, &d, "div", &[("dir", "auto")]);
    let rtl = text(&mut b, &d, "א");
    b.establish_children(&host, &[rtl]).unwrap();
    let root = shadow(&mut b, &d, &host, HtmlSlotAssignment::Named);
    let outer = element(&mut b, &d, "div", &[("dir", "auto")]);
    let slot = element(&mut b, &d, "slot", &[]);
    let later = text(&mut b, &d, "a");
    b.establish_children(&outer, &[slot, later]).unwrap();
    b.establish_children(&root, std::slice::from_ref(&outer))
        .unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        direction(&query(
            &c,
            &context(&c, None),
            &outer,
            HtmlSemanticQuery::Directionality
        )),
        HtmlDirectionality::Rtl
    );
}

#[test]
fn direction_source_and_distinct_context_content_are_preserved_in_qualification() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "input", &[("dir", "auto")]);
    let c = b.finish().unwrap();
    let mut qualifications = vec![];
    for value in ["a", "א"] {
        let cx = context(
            &c,
            Some(supplement(
                &c,
                vec![],
                vec![HtmlControlCurrentValue::new(
                    c.subject_id(&e).unwrap(),
                    Some(value.into()),
                )],
                vec![],
            )),
        );
        let report = query(&c, &cx, &e, HtmlSemanticQuery::Directionality);
        let source = report.qualification().directionality_source();
        assert_eq!(source.unicode_version(), "18.0.0");
        assert_eq!(source.uax9_revision(), 52);
        assert_eq!(source.derived_bidi_class_sha256().len(), 64);
        qualifications.push(report.qualification().clone());
    }
    assert_ne!(qualifications[0], qualifications[1]);
}
