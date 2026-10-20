use crate::semantics::support::{builder, context, session};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlBooleanAuthorContextEvidence, HtmlBooleanAuthorContextReadback, HtmlCapture,
    HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureError, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlHyperlinkReferrerPolicyReadback, HtmlImageMapAncestorEvidence, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticUnsupported, HtmlSemanticValue,
};

fn attr(name: &str, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, name), value)
}
fn elem(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    namespace: Namespace,
    name: &str,
    attrs: Vec<Attribute>,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(namespace, None, name),
        OrderedAttributes::new(attrs).unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn leaf(name: &str, attrs: Vec<Attribute>) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let e = elem(&mut b, &d, Namespace::html(), name, attrs);
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    (c, id)
}

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 1000, 1000, 10000000, 10000)
}
fn request(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn report(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
) -> HtmlSemanticReport {
    session(c, cx, limits(), 100000, HtmlSemanticCancellation::new())
        .query(&[request(id, q)])
        .unwrap()
}
fn boolean(r: &HtmlSemanticReport) -> &HtmlBooleanAuthorContextReadback {
    match r.results()[0].fact() {
        HtmlSemanticFact::ImageMapAttribute(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::ImageControlsAttribute(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::StylesheetDisabledAttribute(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected captured Boolean report: {other:?}"),
    }
}
fn hyperlink(r: &HtmlSemanticReport) -> &HtmlHyperlinkReferrerPolicyReadback {
    match r.results()[0].fact() {
        HtmlSemanticFact::HyperlinkReferrerPolicyAttribute(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected hyperlink report: {other:?}"),
    }
}
fn conforming_ismap(
    raw: Option<DocumentString>,
) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let a = elem(&mut b, &d, Namespace::html(), "a", vec![attr("href", "")]);
    let img = elem(
        &mut b,
        &d,
        Namespace::html(),
        "img",
        raw.map(|v| attr("ismap", v)).into_iter().collect(),
    );
    b.establish_children(&a, std::slice::from_ref(&img))
        .unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&img).unwrap();
    let anchor = c.subject_id(&a).unwrap();
    (c, id, anchor)
}

#[test]
fn all_boolean_spellings_keep_content_independent_from_lexical_author_and_context() {
    for (name, attribute, q, valid_name) in [
        (
            "img",
            "ismap",
            HtmlSemanticQuery::ImageMapAttribute,
            "ISmAp",
        ),
        (
            "img",
            "controls",
            HtmlSemanticQuery::ImageControlsAttribute,
            "CONtrols",
        ),
        (
            "link",
            "disabled",
            HtmlSemanticQuery::StylesheetDisabledAttribute,
            "DisABled",
        ),
    ] {
        for raw in [
            None,
            Some(""),
            Some(attribute),
            Some(valid_name),
            Some("false"),
        ] {
            let (c, id) = if q == HtmlSemanticQuery::ImageMapAttribute {
                let (c, id, _) = conforming_ismap(raw.map(DocumentString::from));
                (c, id)
            } else {
                let mut attrs = vec![if name == "img" {
                    attr("alt", " ")
                } else {
                    attr("rel", "stylesheet")
                }];
                if let Some(raw) = raw {
                    attrs.push(attr(attribute, raw));
                }
                leaf(name, attrs)
            };
            let r = report(&c, &context(&c), &id, q);
            let v = boolean(&r);
            let a = v.attribute_report();
            assert_eq!(a.intrinsic(), Some(&raw.is_some()));
            if raw.is_none() {
                assert_eq!(a.author(), HtmlAttributeAuthorState::Missing);
                assert_eq!(
                    a.context_author(),
                    HtmlAttributeContextAuthorState::NotRequired
                );
                assert_eq!(v.evidence(), &HtmlBooleanAuthorContextEvidence::NotRequired);
            } else {
                assert_eq!(
                    a.context_author(),
                    HtmlAttributeContextAuthorState::Conforming
                );
                assert_eq!(
                    a.author(),
                    if raw == Some("false") {
                        HtmlAttributeAuthorState::Nonconforming(
                            HtmlAttributeConformanceError::BooleanSpelling,
                        )
                    } else {
                        HtmlAttributeAuthorState::Conforming
                    }
                );
            }
        }
    }
}

#[test]
fn ismap_missing_ancestry_is_a_missing_context_with_known_true_content() {
    let (c, id) = leaf("img", vec![attr("ismap", "false")]);
    let r = report(&c, &context(&c), &id, HtmlSemanticQuery::ImageMapAttribute);
    let v = boolean(&r);
    assert_eq!(v.attribute_report().intrinsic(), Some(&true));
    assert_eq!(
        v.attribute_report().context_author(),
        HtmlAttributeContextAuthorState::Unavailable
    );
    assert_eq!(
        v.attribute_report().image_map_context(),
        Some(HtmlImageMapAttributeContext::Unavailable)
    );
    assert_eq!(
        v.evidence(),
        &HtmlBooleanAuthorContextEvidence::ImageMap(HtmlImageMapAncestorEvidence::Missing(
            HtmlSemanticMissing::Parent(id)
        ))
    );
}

#[test]
fn ismap_empty_href_witness_stops_before_unavailable_higher_parent() {
    let (c, id, anchor) = conforming_ismap(Some("ismap".into()));
    let r = report(&c, &context(&c), &id, HtmlSemanticQuery::ImageMapAttribute);
    assert_eq!(
        boolean(&r).attribute_report().context_author(),
        HtmlAttributeContextAuthorState::Conforming
    );
    match boolean(&r).evidence() {
        HtmlBooleanAuthorContextEvidence::ImageMap(
            HtmlImageMapAncestorEvidence::AncestorAnchorWithHref { ancestor, href },
        ) => {
            assert_eq!(ancestor, &anchor);
            assert_eq!(href.value(), &DocumentString::from(""));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn ismap_complete_detached_no_witness_is_nonconforming_without_erasing_presence() {
    let (mut b, d) = builder();
    let parent = elem(&mut b, &d, Namespace::html(), "a", vec![]);
    let img = elem(
        &mut b,
        &d,
        Namespace::html(),
        "img",
        vec![attr("ismap", "")],
    );
    b.establish_root(&parent).unwrap();
    b.establish_children(&parent, std::slice::from_ref(&img))
        .unwrap();
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &context(&c),
        &c.subject_id(&img).unwrap(),
        HtmlSemanticQuery::ImageMapAttribute,
    );
    let v = boolean(&r);
    assert_eq!(v.attribute_report().intrinsic(), Some(&true));
    assert_eq!(
        v.attribute_report().context_author(),
        HtmlAttributeContextAuthorState::Nonconforming(
            HtmlAttributeContextConformanceError::ImageMapRequiresAncestorAnchorWithHref
        )
    );
    assert_eq!(
        v.evidence(),
        &HtmlBooleanAuthorContextEvidence::ImageMap(
            HtmlImageMapAncestorEvidence::NoAncestorAnchorWithHref
        )
    );
}

#[test]
fn ismap_traverses_intervening_foreign_nodes_and_ignores_namespace_and_case_lookalikes() {
    let (mut b, d) = builder();
    let a = elem(
        &mut b,
        &d,
        Namespace::html(),
        "a",
        vec![attr("href", "real")],
    );
    let foreign = elem(
        &mut b,
        &d,
        Namespace::svg(),
        "a",
        vec![attr("href", "ignored")],
    );
    let caps = elem(
        &mut b,
        &d,
        Namespace::html(),
        "A",
        vec![attr("href", "ignored")],
    );
    let namespaced = elem(
        &mut b,
        &d,
        Namespace::html(),
        "a",
        vec![
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "href"),
                "ignored",
            ),
            attr("HREF", "ignored"),
        ],
    );
    let img = elem(
        &mut b,
        &d,
        Namespace::html(),
        "img",
        vec![attr("ismap", "")],
    );
    for (parent, child) in [
        (&a, &foreign),
        (&foreign, &caps),
        (&caps, &namespaced),
        (&namespaced, &img),
    ] {
        b.establish_children(parent, std::slice::from_ref(child))
            .unwrap();
    }
    let c = b.finish().unwrap();
    let r = report(
        &c,
        &context(&c),
        &c.subject_id(&img).unwrap(),
        HtmlSemanticQuery::ImageMapAttribute,
    );
    assert!(
        matches!(boolean(&r).evidence(),HtmlBooleanAuthorContextEvidence::ImageMap(HtmlImageMapAncestorEvidence::AncestorAnchorWithHref{ancestor,..}) if ancestor==&c.subject_id(&a).unwrap())
    );
}

#[test]
fn ismap_chooses_first_qualifying_ancestor_and_uses_only_href_presence() {
    let (mut b, d) = builder();
    let outer = elem(
        &mut b,
        &d,
        Namespace::html(),
        "a",
        vec![attr("href", "outer")],
    );
    let inner = elem(
        &mut b,
        &d,
        Namespace::html(),
        "a",
        vec![attr("href", DocumentString::from_utf16(vec![0xd800; 100]))],
    );
    let img = elem(
        &mut b,
        &d,
        Namespace::html(),
        "img",
        vec![attr("ismap", "")],
    );
    b.establish_children(&outer, std::slice::from_ref(&inner))
        .unwrap();
    b.establish_children(&inner, std::slice::from_ref(&img))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(
            &c.subject_id(&img).unwrap(),
            HtmlSemanticQuery::ImageMapAttribute,
        )])
        .unwrap();
    assert!(
        matches!(boolean(&r).evidence(),HtmlBooleanAuthorContextEvidence::ImageMap(HtmlImageMapAncestorEvidence::AncestorAnchorWithHref{ancestor,href}) if ancestor==&c.subject_id(&inner).unwrap() && href.value().len()==100)
    );
}

#[test]
fn shadow_and_template_hosts_are_not_ordinary_ismap_ancestors() {
    for shadow in [true, false] {
        let (mut b, d) = builder();
        let a = elem(&mut b, &d, Namespace::html(), "a", vec![attr("href", "")]);
        let host = elem(
            &mut b,
            &d,
            Namespace::html(),
            if shadow { "x-host" } else { "template" },
            vec![],
        );
        b.establish_children(&a, std::slice::from_ref(&host))
            .unwrap();
        let fragment = if shadow {
            b.add_shadow_root(
                &d,
                &host,
                HtmlCapturedShadowProperties::new(
                    HtmlShadowRootProperties::new(
                        HtmlShadowRootMode::Closed,
                        HtmlSlotAssignment::Named,
                        false,
                        false,
                        false,
                    ),
                    false,
                    false,
                    false,
                ),
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap()
        } else {
            let f = b.add_fragment(&d).unwrap();
            b.establish_template_contents(&host, &f).unwrap();
            f
        };
        let img = elem(
            &mut b,
            &d,
            Namespace::html(),
            "img",
            vec![attr("ismap", "")],
        );
        b.establish_children(&fragment, std::slice::from_ref(&img))
            .unwrap();
        let c = b.finish().unwrap();
        let r = report(
            &c,
            &context(&c),
            &c.subject_id(&img).unwrap(),
            HtmlSemanticQuery::ImageMapAttribute,
        );
        assert_eq!(
            boolean(&r).evidence(),
            &HtmlBooleanAuthorContextEvidence::ImageMap(
                HtmlImageMapAncestorEvidence::NoAncestorAnchorWithHref
            )
        );
    }
}

#[test]
fn controls_alt_absent_empty_and_exact_nonempty_are_distinct_without_trimming() {
    for alt in [
        None,
        Some(DocumentString::from("")),
        Some(" \t\n".into()),
        Some("description".into()),
        Some(DocumentString::from_utf16(vec![0, 0xd800])),
    ] {
        let mut attrs = vec![attr("controls", "false")];
        if let Some(raw) = &alt {
            attrs.push(attr("alt", raw.clone()));
        }
        let (c, id) = leaf("img", attrs);
        let r = report(
            &c,
            &context(&c),
            &id,
            HtmlSemanticQuery::ImageControlsAttribute,
        );
        let v = boolean(&r);
        assert_eq!(v.attribute_report().intrinsic(), Some(&true));
        let valid = alt.as_ref().is_some_and(|s| !s.is_empty());
        assert_eq!(
            v.attribute_report().context_author(),
            if valid {
                HtmlAttributeContextAuthorState::Conforming
            } else {
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::ImageControlsRequireNonemptyAlt,
                )
            }
        );
        assert_eq!(
            v.attribute_report().image_controls_context(),
            Some(match &alt {
                None => HtmlImageControlsAttributeContext::AltMissing,
                Some(s) if s.is_empty() => HtmlImageControlsAttributeContext::AltEmpty,
                Some(_) => HtmlImageControlsAttributeContext::AltNonempty,
            })
        );
        match v.evidence() {
            HtmlBooleanAuthorContextEvidence::ImageControls { alt: actual } => {
                assert_eq!(actual.as_ref().map(Attribute::value), alt.as_ref())
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn controls_does_not_parse_or_admit_the_uninspected_alt_spelling() {
    let (c, id) = leaf(
        "img",
        vec![
            attr("controls", ""),
            attr("alt", DocumentString::from_utf16(vec![0xd800; 100])),
        ],
    );
    let cx = context(&c);
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(&id, HtmlSemanticQuery::ImageControlsAttribute)])
        .unwrap();
    assert_eq!(
        boolean(&r).attribute_report().context_author(),
        HtmlAttributeContextAuthorState::Conforming
    );
}

#[test]
fn stylesheet_membership_uses_only_ascii_whitespace_and_ascii_case() {
    for (rel, expected) in [
        (None, false),
        (Some(""), false),
        (Some("StyleSheet"), true),
        (Some("other\t\n\u{c}\r StyleSHEET stylesheet"), true),
        (Some("xstylesheet stylesheetx"), false),
        (Some("stylesheet\u{a0}next"), false),
        (Some("\u{a0}stylesheet"), false),
        (Some("style sheet"), false),
        (Some("\u{0} stylesheet"), true),
        (Some("styłeSheet"), false),
    ] {
        let mut attrs = vec![attr("disabled", "false")];
        if let Some(raw) = rel {
            attrs.push(attr("rel", raw));
        }
        let (c, id) = leaf("link", attrs);
        let r = report(
            &c,
            &context(&c),
            &id,
            HtmlSemanticQuery::StylesheetDisabledAttribute,
        );
        let v = boolean(&r);
        assert_eq!(v.attribute_report().intrinsic(), Some(&true));
        assert_eq!(
            v.attribute_report().context_author(),
            if expected {
                HtmlAttributeContextAuthorState::Conforming
            } else {
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::StylesheetDisabledRequiresStylesheetRel,
                )
            }
        );
        let context = v.attribute_report().stylesheet_link_context().unwrap();
        assert_eq!(context.contains_stylesheet(), expected);
        assert_eq!(context.rel(), rel.map(DocumentString::from).as_ref());
    }
}

#[test]
fn stylesheet_retains_original_rel_attribute_prefix_and_non_scalar_units() {
    let raw = DocumentString::from_utf16(vec![
        0xd800, 0x20, 0x53, 0x54, 0x59, 0x4c, 0x45, 0x53, 0x48, 0x45, 0x45, 0x54, 0x20, 0xdc00,
    ]);
    let rel = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "rel"),
        raw.clone(),
    );
    let (c, id) = leaf("link", vec![attr("disabled", ""), rel.clone()]);
    let r = report(
        &c,
        &context(&c),
        &id,
        HtmlSemanticQuery::StylesheetDisabledAttribute,
    );
    assert_eq!(
        boolean(&r).evidence(),
        &HtmlBooleanAuthorContextEvidence::StylesheetLink { rel: Some(rel) }
    );
    assert_eq!(
        boolean(&r)
            .attribute_report()
            .stylesheet_link_context()
            .unwrap()
            .rel(),
        Some(&raw)
    );
}

#[test]
fn target_absence_requires_no_context_lookup_or_raw_admission() {
    for (name, q, context_name) in [
        ("img", HtmlSemanticQuery::ImageMapAttribute, "href"),
        ("img", HtmlSemanticQuery::ImageControlsAttribute, "alt"),
        (
            "link",
            HtmlSemanticQuery::StylesheetDisabledAttribute,
            "rel",
        ),
        (
            "a",
            HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute,
            "href",
        ),
        (
            "area",
            HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute,
            "href",
        ),
    ] {
        let (c, id) = leaf(name, vec![attr(context_name, "unadmitted huge value")]);
        let cx = context(&c);
        let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
            .query(&[request(&id, q)])
            .unwrap();
        if q == HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute {
            let v = hyperlink(&r);
            assert_eq!(
                v.attribute_report().context_author(),
                HtmlAttributeContextAuthorState::NotRequired
            );
            assert_eq!(v.attribute_report().hyperlink_context(), None);
            assert_eq!(v.href_attribute(), None);
        } else {
            assert_eq!(
                boolean(&r).evidence(),
                &HtmlBooleanAuthorContextEvidence::NotRequired
            );
            assert_eq!(boolean(&r).attribute_report().intrinsic(), Some(&false));
        }
    }
}

#[test]
fn hyperlink_href_presence_is_independent_from_referrerpolicy_spelling_and_defaults() {
    for name in ["a", "area"] {
        for href in [None, Some(""), Some("not a parsed URL")] {
            for raw in [None, Some(""), Some("ORIGIN"), Some("not-a-policy")] {
                let mut attrs = vec![];
                if let Some(href) = href {
                    attrs.push(attr("href", href));
                }
                if let Some(raw) = raw {
                    attrs.push(attr("referrerpolicy", raw));
                }
                let (c, id) = leaf(name, attrs);
                let r = report(
                    &c,
                    &context(&c),
                    &id,
                    HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute,
                );
                let v = hyperlink(&r);
                let a = v.attribute_report();
                assert_eq!(
                    a.intrinsic(),
                    Some(&if raw == Some("ORIGIN") {
                        HtmlReferrerPolicy::Origin
                    } else {
                        HtmlReferrerPolicy::Empty
                    })
                );
                assert_eq!(
                    a.context_author(),
                    if raw.is_none() {
                        HtmlAttributeContextAuthorState::NotRequired
                    } else if href.is_some() {
                        HtmlAttributeContextAuthorState::Conforming
                    } else {
                        HtmlAttributeContextAuthorState::Nonconforming(
                            HtmlAttributeContextConformanceError::ReferrerPolicyRequiresHref,
                        )
                    }
                );
                assert_eq!(
                    v.href_attribute().map(Attribute::value),
                    if raw.is_some() {
                        href.map(DocumentString::from)
                    } else {
                        None
                    }
                    .as_ref()
                );
                if raw == Some("not-a-policy") {
                    assert!(matches!(
                        a.author(),
                        HtmlAttributeAuthorState::Nonconforming(_)
                    ));
                }
            }
        }
    }
}

#[test]
fn hyperlink_href_uninspected_utf16_backing_does_not_consume_target_raw_cap() {
    let (c, id) = leaf(
        "a",
        vec![
            attr("referrerpolicy", ""),
            attr("href", DocumentString::from_utf16(vec![0xd800; 100])),
        ],
    );
    let cx = context(&c);
    let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(
            &id,
            HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute,
        )])
        .unwrap();
    assert_eq!(
        hyperlink(&r).attribute_report().context_author(),
        HtmlAttributeContextAuthorState::Conforming
    );
    assert_eq!(hyperlink(&r).href_attribute().unwrap().value().len(), 100);
}

#[test]
fn exact_expanded_names_ignore_context_and_target_attribute_lookalikes() {
    for (name, q, target, context_name) in [
        (
            "img",
            HtmlSemanticQuery::ImageControlsAttribute,
            "controls",
            "alt",
        ),
        (
            "link",
            HtmlSemanticQuery::StylesheetDisabledAttribute,
            "disabled",
            "rel",
        ),
        (
            "a",
            HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute,
            "referrerpolicy",
            "href",
        ),
    ] {
        let foreign = Attribute::new(
            QualifiedName::new(Namespace::svg(), None, context_name),
            "stylesheet",
        );
        let (c, id) = leaf(
            name,
            vec![
                attr(target, ""),
                foreign,
                attr(&context_name.to_ascii_uppercase(), "ignored"),
            ],
        );
        let r = report(&c, &context(&c), &id, q);
        if q == HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute {
            assert_eq!(
                hyperlink(&r).attribute_report().hyperlink_context(),
                Some(HtmlHyperlinkAttributeContext::WithoutHref)
            );
        } else {
            assert!(matches!(
                boolean(&r).attribute_report().context_author(),
                HtmlAttributeContextAuthorState::Nonconforming(_)
            ));
        }
        let (c, id) = leaf(
            name,
            vec![
                Attribute::new(
                    QualifiedName::new(Namespace::svg(), None, target),
                    "oversized",
                ),
                attr(&target.to_ascii_uppercase(), "oversized"),
            ],
        );
        let cx = context(&c);
        let r = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
            .query(&[request(&id, q)])
            .unwrap();
        if q == HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute {
            assert_eq!(
                hyperlink(&r).attribute_report().context_author(),
                HtmlAttributeContextAuthorState::NotRequired
            );
        } else {
            assert_eq!(boolean(&r).attribute_report().intrinsic(), Some(&false));
        }
    }
}

#[test]
fn boolean_raw_utf16_is_preserved_without_changing_presence_or_context() {
    let raw = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    let (c, id) = leaf("img", vec![attr("controls", raw.clone()), attr("alt", " ")]);
    let r = report(
        &c,
        &context(&c),
        &id,
        HtmlSemanticQuery::ImageControlsAttribute,
    );
    let a = boolean(&r).attribute_report();
    assert_eq!(a.raw_attribute().unwrap().value(), &raw);
    assert_eq!(a.intrinsic(), Some(&true));
    assert_eq!(
        a.author(),
        HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::BooleanSpelling)
    );
    assert_eq!(
        a.context_author(),
        HtmlAttributeContextAuthorState::Conforming
    );
}

fn limited(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    let l = limits();
    HtmlSemanticLimits::new(
        if resource == HtmlSemanticResource::Work {
            cap
        } else {
            l.work()
        },
        if resource == HtmlSemanticResource::OutputRecords {
            cap
        } else {
            l.output_records()
        },
        if resource == HtmlSemanticResource::ScratchRecords {
            cap
        } else {
            l.scratch_records()
        },
        l.integer_digits(),
        if resource == HtmlSemanticResource::AllocatedBytes {
            cap
        } else {
            l.allocated_bytes()
        },
        if resource == HtmlSemanticResource::NestedOutputRecords {
            cap
        } else {
            l.nested_output_records()
        },
    )
}
fn cases() -> Vec<(HtmlCapture, HtmlCaptureSubjectId, HtmlSemanticQuery)> {
    let (c, id, _) = conforming_ismap(Some("false".into()));
    let (controls, cid) = leaf("img", vec![attr("controls", ""), attr("alt", " ")]);
    let (link, lid) = leaf(
        "link",
        vec![attr("disabled", "false"), attr("rel", "other StyleSheet")],
    );
    let (a, aid) = leaf(
        "area",
        vec![attr("href", ""), attr("referrerpolicy", "origin")],
    );
    vec![
        (c, id, HtmlSemanticQuery::ImageMapAttribute),
        (controls, cid, HtmlSemanticQuery::ImageControlsAttribute),
        (link, lid, HtmlSemanticQuery::StylesheetDisabledAttribute),
        (a, aid, HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute),
    ]
}

#[test]
fn actual_work_and_physical_compact_output_have_exact_one_short_limits() {
    for (c, id, q) in cases() {
        let cx = context(&c);
        let usage = report(&c, &cx, &id, q).usage();
        assert_eq!(usage.nested_output_records(), 1);
        assert_eq!(usage.scratch_records(), 0);
        assert_eq!(usage.peak_scratch_records(), 0);
        for (resource, exact) in [
            (HtmlSemanticResource::Work, usage.work()),
            (HtmlSemanticResource::OutputRecords, usage.output_records()),
            (
                HtmlSemanticResource::AllocatedBytes,
                usage.allocated_bytes(),
            ),
            (
                HtmlSemanticResource::NestedOutputRecords,
                usage.nested_output_records(),
            ),
        ] {
            assert!(exact > 0);
            for cap in [exact, exact - 1] {
                let mut ss = session(
                    &c,
                    &cx,
                    limited(resource, cap),
                    100000,
                    HtmlSemanticCancellation::new(),
                );
                let result = ss.query(&[request(&id, q)]);
                if cap == exact {
                    assert!(result.is_ok(), "{q:?} {resource:?}: {result:?}");
                } else {
                    let failure = result.unwrap_err();
                    assert!(
                        matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                        "{q:?}: {failure:?}"
                    );
                    let qual = failure.qualification().unwrap();
                    assert_eq!(qual.capture(), c.identity());
                    assert_eq!(qual.context(), cx.identity());
                    assert_eq!(qual.host_revision(), c.host_facts_revision());
                    assert_eq!(qual.limits(), limited(resource, cap));
                    assert_eq!(ss.query(&[]).unwrap_err(), failure);
                }
                assert_eq!(ss.usage().scratch_records(), 0);
            }
        }
        // No query-owned traversal scratch or arbitrary integer is required.
        assert!(
            session(
                &c,
                &cx,
                HtmlSemanticLimits::new(10000000, 1, 0, 0, 10000000, 1),
                100000,
                HtmlSemanticCancellation::new()
            )
            .query(&[request(&id, q)])
            .is_ok()
        );
    }
}

#[test]
fn target_raw_and_actual_rel_admission_are_whole_utf16_even_after_early_match() {
    for (name, q, target, raw, other) in [
        (
            "img",
            HtmlSemanticQuery::ImageMapAttribute,
            "ismap",
            "false",
            None,
        ),
        (
            "img",
            HtmlSemanticQuery::ImageControlsAttribute,
            "controls",
            "false",
            Some(attr("alt", "")),
        ),
        (
            "link",
            HtmlSemanticQuery::StylesheetDisabledAttribute,
            "disabled",
            "false",
            None,
        ),
        (
            "a",
            HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute,
            "referrerpolicy",
            "origin",
            None,
        ),
    ] {
        let mut attrs = vec![attr(target, raw)];
        if let Some(a) = other {
            attrs.push(a);
        }
        let (c, id) = leaf(name, attrs);
        let cx = context(&c);
        assert!(
            session(
                &c,
                &cx,
                limits(),
                raw.len(),
                HtmlSemanticCancellation::new()
            )
            .query(&[request(&id, q)])
            .is_ok()
        );
        let mut ss = session(
            &c,
            &cx,
            limits(),
            raw.len() - 1,
            HtmlSemanticCancellation::new(),
        );
        let failure = ss.query(&[request(&id, q)]).unwrap_err();
        assert!(
            matches!(failure.kind(),HtmlSemanticFailureKind::Attribute(e) if e.required()==raw.len() && e.capacity()==raw.len()-1)
        );
        assert_eq!(
            failure.qualification().unwrap().attribute_input_limit(),
            raw.len() - 1
        );
        assert_eq!(ss.query(&[]).unwrap_err(), failure);
    }
    let raw = DocumentString::from_utf16(
        "stylesheet ignored"
            .encode_utf16()
            .chain([0xd800])
            .collect::<Vec<_>>(),
    );
    let (c, id) = leaf("link", vec![attr("disabled", ""), attr("rel", raw.clone())]);
    let cx = context(&c);
    let r = session(
        &c,
        &cx,
        limits(),
        raw.len(),
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&id, HtmlSemanticQuery::StylesheetDisabledAttribute)])
    .unwrap();
    assert!(
        boolean(&r)
            .attribute_report()
            .stylesheet_link_context()
            .unwrap()
            .contains_stylesheet()
    );
    let mut ss = session(
        &c,
        &cx,
        limits(),
        raw.len() - 1,
        HtmlSemanticCancellation::new(),
    );
    let failure = ss
        .query(&[request(&id, HtmlSemanticQuery::StylesheetDisabledAttribute)])
        .unwrap_err();
    assert!(
        matches!(failure.kind(),HtmlSemanticFailureKind::Attribute(e) if e.required()==raw.len())
    );
    assert_eq!(ss.usage().nested_output_records(), 0);
}

#[test]
fn absent_targets_spend_less_work_than_required_context_and_skip_unknown_ancestry() {
    for (c, id, q) in cases() {
        let with = report(&c, &context(&c), &id, q).usage().work();
        let name = match q {
            HtmlSemanticQuery::ImageMapAttribute | HtmlSemanticQuery::ImageControlsAttribute => {
                "img"
            }
            HtmlSemanticQuery::StylesheetDisabledAttribute => "link",
            _ => "area",
        };
        let (empty, eid) = leaf(name, vec![]);
        let cx = context(&empty);
        let without = report(&empty, &cx, &eid, q).usage().work();
        assert!(without < with);
        assert!(
            session(
                &empty,
                &cx,
                limited(HtmlSemanticResource::Work, without),
                0,
                HtmlSemanticCancellation::new()
            )
            .query(&[request(&eid, q)])
            .is_ok()
        );
    }
}

#[test]
fn wrong_roles_exact_case_foreign_namespace_and_non_elements_have_no_context_allocation() {
    for q in [
        HtmlSemanticQuery::ImageMapAttribute,
        HtmlSemanticQuery::ImageControlsAttribute,
        HtmlSemanticQuery::StylesheetDisabledAttribute,
        HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute,
    ] {
        let (mut b, d) = builder();
        let div = elem(
            &mut b,
            &d,
            Namespace::html(),
            "div",
            vec![
                attr("ismap", "huge"),
                attr("controls", "huge"),
                attr("disabled", "huge"),
                attr("referrerpolicy", "huge"),
            ],
        );
        let upper = elem(&mut b, &d, Namespace::html(), "IMG", vec![]);
        let foreign = elem(&mut b, &d, Namespace::svg(), "img", vec![]);
        let text = b.add_text(&d, "ignored".into()).unwrap();
        let c = b.finish().unwrap();
        let cx = context(&c);
        for (e, is_foreign) in [
            (&div, false),
            (&upper, false),
            (&foreign, true),
            (&text, false),
            (&d, false),
        ] {
            let r = session(
                &c,
                &cx,
                HtmlSemanticLimits::new(10000000, 1, 0, 0, 10000000, 0),
                0,
                HtmlSemanticCancellation::new(),
            )
            .query(&[request(&c.subject_id(e).unwrap(), q)])
            .unwrap();
            let expected = if is_foreign {
                HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
            } else {
                HtmlSemanticValue::Inapplicable
            };
            match r.results()[0].fact() {
                HtmlSemanticFact::ImageMapAttribute(v)
                | HtmlSemanticFact::ImageControlsAttribute(v)
                | HtmlSemanticFact::StylesheetDisabledAttribute(v) => assert_eq!(v, &expected),
                HtmlSemanticFact::HyperlinkReferrerPolicyAttribute(v) => assert_eq!(
                    v,
                    &if is_foreign {
                        HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
                    } else {
                        HtmlSemanticValue::Inapplicable
                    }
                ),
                other => panic!("{other:?}"),
            }
            assert_eq!(r.usage().nested_output_records(), 0);
        }
    }
}

#[test]
fn late_raw_failure_discards_mixed_batch_and_keeps_original_qualified_stop() {
    let (mut b, d) = builder();
    let img = elem(
        &mut b,
        &d,
        Namespace::html(),
        "img",
        vec![attr("controls", ""), attr("alt", " ")],
    );
    let link = elem(
        &mut b,
        &d,
        Namespace::html(),
        "link",
        vec![attr("disabled", ""), attr("rel", "stylesheet")],
    );
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let failure = ss
        .query(&[
            request(
                &c.subject_id(&img).unwrap(),
                HtmlSemanticQuery::ImageControlsAttribute,
            ),
            request(
                &c.subject_id(&link).unwrap(),
                HtmlSemanticQuery::StylesheetDisabledAttribute,
            ),
        ])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(ss.usage().nested_output_records(), 1);
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(failure.qualification().unwrap().context(), cx.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
}

#[test]
fn cancellation_and_foreign_capture_failures_are_sticky_after_completed_results() {
    let (c, id, _) = conforming_ismap(Some("".into()));
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut ss = session(&c, &cx, limits(), 100000, cancel.clone());
    let old = ss
        .query(&[request(&id, HtmlSemanticQuery::ImageMapAttribute)])
        .unwrap();
    cancel.cancel();
    let failure = ss.query(&[]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(
        ss.query(&[request(&id, HtmlSemanticQuery::ImageMapAttribute)])
            .unwrap_err(),
        failure
    );
    assert_eq!(boolean(&old).attribute_report().intrinsic(), Some(&true));
    let (foreign, fid) = leaf("img", vec![]);
    let foreign_context = context(&foreign);
    let mut ss = session(&c, &cx, limits(), 100000, HtmlSemanticCancellation::new());
    let failure = ss
        .query(&[
            request(&id, HtmlSemanticQuery::ImageMapAttribute),
            request(&fid, HtmlSemanticQuery::ImageMapAttribute),
        ])
        .unwrap_err();
    assert_eq!(
        failure.kind(),
        &HtmlSemanticFailureKind::Capture(HtmlCaptureError::ForeignCapture)
    );
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    assert_ne!(
        failure.qualification().unwrap().context(),
        foreign_context.identity()
    );
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
    assert_eq!(ss.usage().scratch_records(), 0);
}

#[test]
fn changed_captured_dependencies_and_contexts_are_qualified_without_mutating_old_reports() {
    for (name, q, target, context_name, first, second) in [
        (
            "img",
            HtmlSemanticQuery::ImageControlsAttribute,
            "controls",
            "alt",
            "",
            " ",
        ),
        (
            "link",
            HtmlSemanticQuery::StylesheetDisabledAttribute,
            "disabled",
            "rel",
            "alternate",
            "StyleSheet",
        ),
        (
            "a",
            HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute,
            "referrerpolicy",
            "href",
            "",
            "x",
        ),
    ] {
        let (c, id) = leaf(name, vec![attr(target, ""), attr(context_name, first)]);
        let cx = context(&c);
        let old = report(&c, &cx, &id, q);
        let (changed, cid) = leaf(name, vec![attr(target, ""), attr(context_name, second)]);
        let changed_context = context(&changed);
        let new = report(&changed, &changed_context, &cid, q);
        assert_ne!(old.capture(), new.capture());
        assert_ne!(old.context(), new.context());
        assert_ne!(
            old.qualification().host_revision(),
            new.qualification().host_revision()
        );
        if q != HtmlSemanticQuery::HyperlinkReferrerPolicyAttribute {
            assert_ne!(
                boolean(&old).attribute_report().context_author(),
                boolean(&new).attribute_report().context_author()
            );
        } else {
            assert_ne!(
                hyperlink(&old).href_attribute(),
                hyperlink(&new).href_attribute()
            );
        }
        let fresh = context(&c);
        let recomputed = report(&c, &fresh, &id, q);
        assert_ne!(old.context(), recomputed.context());
        assert_eq!(old.results()[0].fact(), recomputed.results()[0].fact());
    }
    let (unavailable, id) = leaf("img", vec![attr("ismap", "")]);
    let old = report(
        &unavailable,
        &context(&unavailable),
        &id,
        HtmlSemanticQuery::ImageMapAttribute,
    );
    let (established, id, _) = conforming_ismap(Some("".into()));
    let new = report(
        &established,
        &context(&established),
        &id,
        HtmlSemanticQuery::ImageMapAttribute,
    );
    assert_eq!(
        boolean(&old).attribute_report().context_author(),
        HtmlAttributeContextAuthorState::Unavailable
    );
    assert_eq!(
        boolean(&new).attribute_report().context_author(),
        HtmlAttributeContextAuthorState::Conforming
    );
}
