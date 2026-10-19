use crate::semantics::support::{
    builder, collection_limits as capped, context, semantic_limits as limits, session,
};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlMetaAttributeAuthorStructureReport, HtmlMetaCharsetUniqueness,
    HtmlMetaContentPresenceRequirement, HtmlMetaDiscriminatorRequirement, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticMissing,
    HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource,
    HtmlSemanticUnsupported, HtmlSemanticValue,
};

// HTML 877d #the-meta-element, raw 11227–11260: exactly one discriminator,
// independent conditional content presence, theme-color media placement and
// specified-charset multiplicity. These are structural author observations.

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: QualifiedName,
    attributes: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        name,
        attributes,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn attributes(fields: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(fields.iter().map(|(name, value)| {
        Attribute::new(QualifiedName::new(Namespace::none(), None, *name), *value)
    }))
    .unwrap()
}
fn meta(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    fields: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    element(
        b,
        d,
        KnownHtmlElement::Meta.qualified_name(),
        attributes(fields),
    )
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    cs: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, cs).unwrap();
}

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::MetaAttributeAuthorStructure)
}

fn report(c: &HtmlCapture, id: &HtmlCaptureSubjectId, raw: usize) -> HtmlSemanticReport {
    session(
        c,
        &context(c),
        limits(),
        raw,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(id)])
    .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlMetaAttributeAuthorStructureReport {
    match r.results()[0].fact() {
        HtmlSemanticFact::MetaAttributeAuthorStructure(HtmlSemanticValue::Known(v)) => v,
        other => panic!("{other:?}"),
    }
}
fn single(fields: &[(&str, &str)]) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, fields);
    children(&mut b, &a, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    (c, id)
}
fn pair(
    a_fields: &[(&str, &str)],
    z_fields: &[(&str, &str)],
) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, a_fields);
    let z = meta(&mut b, &d, z_fields);
    children(&mut b, &a, &[]);
    children(&mut b, &z, &[]);
    children(&mut b, &d, &[a.clone(), z.clone()]);
    let c = b.finish().unwrap();
    let ai = c.subject_id(&a).unwrap();
    let zi = c.subject_id(&z).unwrap();
    (c, ai, zi)
}

#[test]
fn every_discriminator_subset_has_independent_literal_content_requirement() {
    use HtmlMetaDiscriminatorRequirement::{
        MultipleSpecified as M, NoneSpecified as N, Satisfied as S,
    };
    // bit order name/http-equiv/charset/itemprop, literal source-derived truth table.
    for (mask, exclusivity, content_required) in [
        (0, N, false),
        (1, S, true),
        (2, S, true),
        (3, M, true),
        (4, S, false),
        (5, M, true),
        (6, M, true),
        (7, M, true),
        (8, S, true),
        (9, M, true),
        (10, M, true),
        (11, M, true),
        (12, M, true),
        (13, M, true),
        (14, M, true),
        (15, M, true),
    ] {
        for present in [false, true] {
            let mut fields = Vec::new();
            for (bit, local) in [
                (1, "name"),
                (2, "http-equiv"),
                (4, "charset"),
                (8, "itemprop"),
            ] {
                if mask & bit != 0 {
                    fields.push((local, ""));
                }
            }
            if present {
                fields.push(("content", ""));
            }
            let (c, id) = single(&fields);
            let r = report(&c, &id, 0);
            let v = value(&r);
            assert_eq!(v.discriminator_requirement(), exclusivity, "mask {mask}");
            assert_eq!(
                v.content_presence_requirement(),
                match (content_required, present) {
                    (true, false) => HtmlMetaContentPresenceRequirement::MissingRequiredContent,
                    (false, true) => HtmlMetaContentPresenceRequirement::UnexpectedContent,
                    _ => HtmlMetaContentPresenceRequirement::Satisfied,
                },
                "mask {mask}, content {present}"
            );
            assert_eq!(
                v.charset_uniqueness(),
                &if mask & 4 == 0 {
                    HtmlMetaCharsetUniqueness::NotRequired
                } else {
                    HtmlMetaCharsetUniqueness::Unique
                }
            );
        }
    }
}

#[test]
fn raw_six_fields_preserve_null_isolated_surrogates_and_spelling() {
    let (mut b, d) = builder();
    let raw = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00, 0xa0, 13, 10]);
    let a = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        OrderedAttributes::new(
            [
                "name",
                "http-equiv",
                "charset",
                "itemprop",
                "content",
                "media",
            ]
            .map(|local| {
                Attribute::new(
                    QualifiedName::new(Namespace::none(), Some("p".into()), local),
                    raw.clone(),
                )
            }),
        )
        .unwrap(),
    );
    children(&mut b, &a, &[]);
    children(&mut b, &d, std::slice::from_ref(&a));
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&a).unwrap(), 6);
    let v = value(&r);
    for field in [
        v.name_attribute(),
        v.http_equiv_attribute(),
        v.charset_attribute(),
        v.itemprop_attribute(),
        v.content_attribute(),
        v.media_attribute(),
    ] {
        let a = field.unwrap();
        assert_eq!(a.value().code_units(), &[0xd800, 0, 0xdc00, 0xa0, 13, 10]);
        assert_eq!(a.name().prefix().unwrap().code_units(), &[112]);
    }
    assert_eq!(
        v.discriminator_requirement(),
        HtmlMetaDiscriminatorRequirement::MultipleSpecified
    );
    assert_eq!(
        v.content_presence_requirement(),
        HtmlMetaContentPresenceRequirement::Satisfied
    );
    assert_eq!(v.media_placement(), HtmlMetaMediaPlacement::Forbidden);
    assert_eq!(v.charset_uniqueness(), &HtmlMetaCharsetUniqueness::Unique);
    assert_eq!(r.qualification().capture(), c.identity());
}

#[test]
fn theme_color_uses_whole_ascii_metadata_name_and_not_css_validation() {
    for (name, allowed) in [
        ("theme-color", true),
        ("ThEmE-CoLoR", true),
        (" theme-color", false),
        ("theme-color ", false),
        ("theme-coloR\u{a0}", false),
        ("theme-coloſ", false),
        ("", false),
        ("description", false),
    ] {
        let (c, id) = single(&[
            ("name", name),
            ("media", "not a CSS media query"),
            ("content", ""),
        ]);
        let r = report(&c, &id, 100);
        assert_eq!(
            value(&r).media_placement(),
            if allowed {
                HtmlMetaMediaPlacement::AllowedForThemeColor
            } else {
                HtmlMetaMediaPlacement::Forbidden
            }
        );
    }
}
#[test]
fn media_without_name_is_forbidden_even_with_another_discriminator() {
    for fields in [
        vec![("media", "")],
        vec![("charset", "utf-8"), ("media", "")],
        vec![("http-equiv", "theme-color"), ("media", "")],
        vec![("itemprop", "theme-color"), ("media", "")],
    ] {
        let (c, id) = single(&fields);
        assert_eq!(
            value(&report(&c, &id, 11)).media_placement(),
            HtmlMetaMediaPlacement::Forbidden
        );
    }
}
#[test]
fn absent_media_does_not_require_theme_color_or_ancestry() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, &[("name", "not-a-standard-name")]);
    let c = b.finish().unwrap();
    let r = report(&c, &c.subject_id(&a).unwrap(), 30);
    let v = value(&r);
    assert_eq!(v.media_placement(), HtmlMetaMediaPlacement::NotSpecified);
    assert_eq!(
        v.charset_uniqueness(),
        &HtmlMetaCharsetUniqueness::NotRequired
    );
    assert_eq!(
        v.content_presence_requirement(),
        HtmlMetaContentPresenceRequirement::MissingRequiredContent
    );
}
#[test]
fn namespaced_and_case_changed_attribute_names_do_not_become_discriminators() {
    for local in [
        "name",
        "http-equiv",
        "charset",
        "itemprop",
        "content",
        "media",
    ] {
        for name in [
            QualifiedName::new(Namespace::from_uri("urn:other"), Some("x".into()), local),
            QualifiedName::new(Namespace::none(), None, local.to_ascii_uppercase()),
        ] {
            let (mut b, d) = builder();
            let a = element(
                &mut b,
                &d,
                KnownHtmlElement::Meta.qualified_name(),
                OrderedAttributes::new([Attribute::new(name, "huge irrelevant raw value")])
                    .unwrap(),
            );
            let c = b.finish().unwrap();
            let r = report(&c, &c.subject_id(&a).unwrap(), 0);
            let v = value(&r);
            assert_eq!(
                v.discriminator_requirement(),
                HtmlMetaDiscriminatorRequirement::NoneSpecified
            );
            assert_eq!(
                v.content_presence_requirement(),
                HtmlMetaContentPresenceRequirement::Satisfied
            );
            assert_eq!(v.media_placement(), HtmlMetaMediaPlacement::NotSpecified);
            assert_eq!(
                v.charset_uniqueness(),
                &HtmlMetaCharsetUniqueness::NotRequired
            );
        }
    }
}
#[test]
fn namespace_and_exact_meta_element_name_define_applicability() {
    for (ns, local, expected_foreign) in [
        (Namespace::html(), "meta", false),
        (Namespace::html(), "META", false),
        (Namespace::html(), "div", false),
        (Namespace::svg(), "meta", true),
    ] {
        let (mut b, d) = builder();
        let a = element(
            &mut b,
            &d,
            QualifiedName::new(ns, Some("x".into()), local),
            attributes(&[("charset", "")]),
        );
        children(&mut b, &a, &[]);
        children(&mut b, &d, std::slice::from_ref(&a));
        let c = b.finish().unwrap();
        let r = report(&c, &c.subject_id(&a).unwrap(), 0);
        match r.results()[0].fact() {
            HtmlSemanticFact::MetaAttributeAuthorStructure(HtmlSemanticValue::Known(_)) => {
                assert_eq!(local, "meta")
            }
            HtmlSemanticFact::MetaAttributeAuthorStructure(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            )) => assert!(expected_foreign),
            HtmlSemanticFact::MetaAttributeAuthorStructure(HtmlSemanticValue::Inapplicable) => {
                assert!(local == "META" || local == "div")
            }
            x => panic!("{x:?}"),
        }
    }
}
#[test]
fn invalid_and_empty_charset_competitors_still_count_by_presence() {
    for competitor in ["", "utf-8", "not-an-encoding", "  UTF-8 "] {
        let (c, a, z) = pair(&[("charset", "utf-8")], &[("charset", competitor)]);
        assert_eq!(
            value(&report(&c, &a, 5)).charset_uniqueness(),
            &HtmlMetaCharsetUniqueness::Duplicate(z)
        );
    }
}
#[test]
fn competitor_raw_values_and_other_author_violations_are_irrelevant() {
    let (c, a, z) = pair(
        &[("charset", "")],
        &[
            ("charset", "1000000000000000000000"),
            ("name", "too long"),
            ("content", "long"),
            ("http-equiv", "long"),
            ("itemprop", "long"),
            ("media", "long"),
        ],
    );
    assert_eq!(
        value(&report(&c, &a, 0)).charset_uniqueness(),
        &HtmlMetaCharsetUniqueness::Duplicate(z)
    );
}
#[test]
fn empty_and_invalid_own_charset_do_not_claim_utf8_validity() {
    for raw in ["", "invalid"] {
        let (c, id) = single(&[("charset", raw)]);
        let r = report(&c, &id, 7);
        assert_eq!(
            value(&r).charset_attribute().unwrap().value(),
            &DocumentString::from(raw)
        );
        assert_eq!(
            value(&r).charset_uniqueness(),
            &HtmlMetaCharsetUniqueness::Unique
        );
        let raw_attribute = Attribute::new(KnownHtmlAttribute::Charset.qualified_name(), raw);
        let native = HtmlValueInterpreter::new(7)
            .meta_charset(
                &KnownHtmlElement::Meta.qualified_name(),
                Some(&raw_attribute),
            )
            .unwrap();
        // The existing local spelling failure is independent of population proof.
        assert_eq!(
            native.author(),
            HtmlAttributeAuthorState::Nonconforming(
                HtmlAttributeConformanceError::InvalidUtf8EncodingLabel(HtmlUtf8EncodingLabelError)
            )
        );
        assert_eq!(
            native.context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
    }
}
#[test]
fn known_document_duplicate_in_body_counts_without_head_only_restriction() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, &[("charset", "")]);
    let body = element(
        &mut b,
        &d,
        KnownHtmlElement::Body.qualified_name(),
        attributes(&[]),
    );
    let z = meta(&mut b, &d, &[("charset", "")]);
    for n in [&a, &z] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &body, std::slice::from_ref(&z));
    children(&mut b, &d, &[a.clone(), body]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&a).unwrap(), 0)).charset_uniqueness(),
        &HtmlMetaCharsetUniqueness::Duplicate(c.subject_id(&z).unwrap())
    );
}
#[test]
fn foreign_cased_and_namespaced_charset_lookalikes_do_not_compete() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, &[("charset", "")]);
    let foreign = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::svg(), None, "meta"),
        attributes(&[("charset", "")]),
    );
    let upper = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), None, "META"),
        attributes(&[("charset", "")]),
    );
    let ns = element(
        &mut b,
        &d,
        KnownHtmlElement::Meta.qualified_name(),
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:x"), None, "charset"),
            "",
        )])
        .unwrap(),
    );
    let cap = meta(&mut b, &d, &[("CHARSET", "")]);
    let http = meta(
        &mut b,
        &d,
        &[
            ("http-equiv", "content-type"),
            ("content", "text/html;charset=utf-8"),
        ],
    );
    for n in [&a, &foreign, &upper, &ns, &cap, &http] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &d, &[a.clone(), foreign, upper, ns, cap, http]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&a).unwrap(), 0)).charset_uniqueness(),
        &HtmlMetaCharsetUniqueness::Unique
    );
}
#[test]
fn two_documents_in_one_capture_do_not_share_charset_population() {
    let (mut b, d) = builder();
    let e = b
        .add_document(
            HtmlDocumentMode::Quirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let a = meta(&mut b, &d, &[("charset", "")]);
    let z = meta(&mut b, &e, &[("charset", "")]);
    for n in [&a, &z] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &d, std::slice::from_ref(&a));
    children(&mut b, &e, std::slice::from_ref(&z));
    let c = b.finish().unwrap();
    for n in [&a, &z] {
        assert_eq!(
            value(&report(&c, &c.subject_id(n).unwrap(), 0)).charset_uniqueness(),
            &HtmlMetaCharsetUniqueness::Unique
        );
    }
}
#[test]
fn detached_element_and_fragment_are_not_document_metadata_scopes() {
    for fragment in [false, true] {
        let (mut b, d) = builder();
        let root = if fragment {
            b.add_fragment(&d).unwrap()
        } else {
            element(
                &mut b,
                &d,
                KnownHtmlElement::Div.qualified_name(),
                attributes(&[]),
            )
        };
        let a = meta(&mut b, &d, &[("charset", "")]);
        let z = meta(&mut b, &d, &[("charset", "")]);
        for n in [&a, &z] {
            children(&mut b, n, &[]);
        }
        children(&mut b, &root, &[a.clone(), z]);
        if !fragment {
            b.establish_root(&root).unwrap();
        }
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap(), 0)).charset_uniqueness(),
            &HtmlMetaCharsetUniqueness::NoDocumentMetadataScope
        );
    }
}
#[test]
fn unknown_ancestry_cannot_be_rescued_by_duplicate_inside_known_component() {
    let (mut b, d) = builder();
    let p = element(
        &mut b,
        &d,
        KnownHtmlElement::Div.qualified_name(),
        attributes(&[]),
    );
    let a = meta(&mut b, &d, &[("charset", "")]);
    let z = meta(&mut b, &d, &[("charset", "")]);
    for n in [&a, &z] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &p, &[a.clone(), z]);
    let c = b.finish().unwrap();
    assert_eq!(
        value(&report(&c, &c.subject_id(&a).unwrap(), 0)).charset_uniqueness(),
        &HtmlMetaCharsetUniqueness::Missing(HtmlSemanticMissing::Parent(c.subject_id(&p).unwrap()))
    );
}
#[test]
fn local_violations_do_not_erase_influencing_missing_charset_context() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, &[("charset", ""), ("name", ""), ("media", "")]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&a).unwrap();
    let r = report(&c, &id, 0);
    let v = value(&r);
    assert_eq!(
        v.discriminator_requirement(),
        HtmlMetaDiscriminatorRequirement::MultipleSpecified
    );
    assert_eq!(
        v.content_presence_requirement(),
        HtmlMetaContentPresenceRequirement::MissingRequiredContent
    );
    assert_eq!(v.media_placement(), HtmlMetaMediaPlacement::Forbidden);
    assert_eq!(
        v.charset_uniqueness(),
        &HtmlMetaCharsetUniqueness::Missing(HtmlSemanticMissing::Parent(id))
    );
}
#[test]
fn proved_document_duplicate_resolves_unrelated_children_gap_but_absence_does_not() {
    for duplicate in [false, true] {
        let (mut b, d) = builder();
        let a = meta(&mut b, &d, &[("charset", "")]);
        let gap = element(
            &mut b,
            &d,
            KnownHtmlElement::Div.qualified_name(),
            attributes(&[]),
        );
        let z = meta(&mut b, &d, if duplicate { &[("charset", "")] } else { &[] });
        for n in [&a, &z] {
            children(&mut b, n, &[]);
        }
        children(&mut b, &d, &[a.clone(), gap.clone(), z.clone()]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap(), 0)).charset_uniqueness(),
            &if duplicate {
                HtmlMetaCharsetUniqueness::Duplicate(c.subject_id(&z).unwrap())
            } else {
                HtmlMetaCharsetUniqueness::Missing(HtmlSemanticMissing::OrderedChildren(
                    c.subject_id(&gap).unwrap(),
                ))
            }
        );
    }
}
#[test]
fn template_and_shadow_associations_do_not_join_charset_population() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let a = meta(&mut b, &d, &[("charset", "")]);
        let host = element(
            &mut b,
            &d,
            if shadow {
                KnownHtmlElement::Div.qualified_name()
            } else {
                KnownHtmlElement::Template.qualified_name()
            },
            attributes(&[]),
        );
        let f = if shadow {
            b.add_shadow_root(
                &d,
                &host,
                HtmlCapturedShadowProperties::new(
                    HtmlShadowRootProperties::new(
                        HtmlShadowRootMode::Open,
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
        let z = meta(&mut b, &d, &[("charset", "")]);
        for n in [&a, &host, &z] {
            children(&mut b, n, &[]);
        }
        children(&mut b, &f, std::slice::from_ref(&z));
        children(&mut b, &d, &[a.clone(), host]);
        let c = b.finish().unwrap();
        assert_eq!(
            value(&report(&c, &c.subject_id(&a).unwrap(), 0)).charset_uniqueness(),
            &HtmlMetaCharsetUniqueness::Unique
        );
        assert_eq!(
            value(&report(&c, &c.subject_id(&z).unwrap(), 0)).charset_uniqueness(),
            &HtmlMetaCharsetUniqueness::NoDocumentMetadataScope
        );
    }
}
#[test]
fn each_original_own_retained_raw_field_obeys_cap_before_delivery() {
    for field in [
        "name",
        "http-equiv",
        "charset",
        "itemprop",
        "content",
        "media",
    ] {
        let (c, id) = single(&[(field, "0123456789")]);
        let cx = context(&c);
        let mut s = session(&c, &cx, limits(), 9, HtmlSemanticCancellation::new());
        let fail = s.query(&[request(&id)]).unwrap_err();
        assert!(
            matches!(fail.kind(), HtmlSemanticFailureKind::Attribute(_)),
            "{field}"
        );
        assert_eq!(s.query(&[]).unwrap_err(), fail);
        assert_eq!(s.usage().scratch_records(), 0);
        assert!(
            session(&c, &cx, limits(), 10, HtmlSemanticCancellation::new())
                .query(&[request(&id)])
                .is_ok()
        );
    }
}

#[test]
fn actual_work_and_capacity_limits_accept_exact_and_reject_one_short() {
    let (c, id) = single(&[("charset", "utf-8"), ("content", "")]);
    let cx = context(&c);
    let usage = report(&c, &id, 5).usage();
    assert_eq!(usage.nested_output_records(), 1);
    assert_eq!(usage.scratch_records(), 0);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, usage.work()),
        (HtmlSemanticResource::OutputRecords, usage.output_records()),
        (
            HtmlSemanticResource::ScratchRecords,
            usage.peak_scratch_records(),
        ),
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
            let mut s = session(
                &c,
                &cx,
                capped(resource, cap),
                5,
                HtmlSemanticCancellation::new(),
            );
            let r = s.query(&[request(&id)]);
            if cap == exact {
                assert_eq!(
                    value(&r.unwrap()).charset_uniqueness(),
                    &HtmlMetaCharsetUniqueness::Unique
                );
            } else {
                let fail = r.unwrap_err();
                assert!(
                    matches!(fail.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..}if *r==resource),
                    "{fail:?}"
                );
                assert_eq!(s.query(&[]).unwrap_err(), fail);
            }
            assert_eq!(s.usage().scratch_records(), 0);
        }
    }
}
#[test]
fn terminal_work_cuts_cannot_be_rescued_by_a_later_duplicate() {
    let (mut b, d) = builder();
    let a = meta(&mut b, &d, &[("charset", "")]);
    let gap = element(
        &mut b,
        &d,
        KnownHtmlElement::Div.qualified_name(),
        attributes(&[]),
    );
    let z = meta(&mut b, &d, &[("charset", "")]);
    for n in [&a, &z] {
        children(&mut b, n, &[]);
    }
    children(&mut b, &d, &[a.clone(), gap, z.clone()]);
    let c = b.finish().unwrap();
    let ai = c.subject_id(&a).unwrap();
    let cx = context(&c);
    let total = report(&c, &ai, 0).usage().work();
    for cap in [0, 1, total / 2, total - 1] {
        let mut s = session(
            &c,
            &cx,
            capped(HtmlSemanticResource::Work, cap),
            0,
            HtmlSemanticCancellation::new(),
        );
        let fail = s.query(&[request(&ai)]).unwrap_err();
        assert!(matches!(
            fail.kind(),
            HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                ..
            }
        ));
        assert_eq!(s.query(&[]).unwrap_err(), fail);
        assert_eq!(s.usage().scratch_records(), 0);
    }
}
#[test]
fn cancellation_is_sticky_with_exact_capture_and_context_qualification() {
    let (c, id) = single(&[("charset", "")]);
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = session(&c, &cx, limits(), 0, cancel);
    let fail = s.query(&[request(&id)]).unwrap_err();
    assert_eq!(fail.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(fail.qualification().unwrap().capture(), c.identity());
    assert_eq!(fail.qualification().unwrap().context(), cx.identity());
    assert_eq!(s.query(&[]).unwrap_err(), fail);
    assert_eq!(s.usage().scratch_records(), 0);
}
#[test]
fn late_raw_and_nested_capacity_stops_fail_the_entire_requested_batch() {
    let (c, a, z) = pair(&[("name", "")], &[("name", ""), ("content", "123456")]);
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new());
    let fail = s.query(&[request(&a), request(&z)]).unwrap_err();
    assert!(matches!(fail.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.query(&[]).unwrap_err(), fail);
    for resource in [
        HtmlSemanticResource::OutputRecords,
        HtmlSemanticResource::NestedOutputRecords,
    ] {
        let mut s = session(
            &c,
            &cx,
            capped(resource, 1),
            6,
            HtmlSemanticCancellation::new(),
        );
        let fail = s.query(&[request(&a), request(&z)]).unwrap_err();
        assert!(
            matches!(fail.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..}if *r==resource)
        );
        assert_eq!(s.query(&[]).unwrap_err(), fail);
        assert_eq!(s.usage().scratch_records(), 0);
    }
}
#[test]
fn retained_report_survives_another_query_without_reinterpreting_capture() {
    let (c, a, z) = pair(&[("charset", "bad")], &[("name", "description")]);
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 11, HtmlSemanticCancellation::new());
    let first = s.query(&[request(&a)]).unwrap();
    let second = s.query(&[request(&z)]).unwrap();
    assert_eq!(
        value(&first).charset_attribute().unwrap().value(),
        &DocumentString::from("bad")
    );
    assert_eq!(
        value(&first).charset_uniqueness(),
        &HtmlMetaCharsetUniqueness::Unique
    );
    assert_eq!(
        value(&second).charset_uniqueness(),
        &HtmlMetaCharsetUniqueness::NotRequired
    );
    assert_eq!(second.usage().nested_output_records(), 2);
}
#[test]
fn non_element_and_foreign_capture_requests_remain_typed() {
    let (c, id) = single(&[]);
    let (other, foreign) = single(&[]);
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    assert!(s.query(&[request(&foreign)]).is_err());
    assert_ne!(c.identity(), other.identity());
    let (mut b, d) = builder();
    let t = b.add_text(&d, "x".into()).unwrap();
    children(&mut b, &d, std::slice::from_ref(&t));
    let capture = b.finish().unwrap();
    for n in [&d, &t] {
        let r = report(&capture, &capture.subject_id(n).unwrap(), 0);
        assert!(matches!(
            r.results()[0].fact(),
            HtmlSemanticFact::MetaAttributeAuthorStructure(HtmlSemanticValue::Inapplicable)
        ));
    }
    assert_eq!(
        value(&report(&c, &id, 0)).discriminator_requirement(),
        HtmlMetaDiscriminatorRequirement::NoneSpecified
    );
}
