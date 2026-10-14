#[path = "common/builder.rs"]
mod shared_builder;
use shared_builder::builder;
use surgeist_html::*;

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    name: &str,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, name),
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "value"),
            "irrelevant very long value attribute",
        )])
        .unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: &str,
) -> HtmlCaptureBuilderSubject {
    element(b, d, Namespace::html(), name)
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    c: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, c).unwrap();
}
fn text(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    v: impl Into<DocumentString>,
) -> HtmlCaptureBuilderSubject {
    b.add_text(d, v.into()).unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 100000)
}
fn fact_limits() -> HtmlOutputDefaultLimits {
    HtmlOutputDefaultLimits::new(100, 100000, 1000000)
}
fn context(c: &HtmlCapture, records: Option<Vec<HtmlOutputDefaultFact>>) -> HtmlSemanticContext {
    let input = if let Some(records) = records {
        let facts =
            HtmlOutputDefaultFacts::new(c, c.host_facts_revision().clone(), records, fact_limits())
                .unwrap();
        HtmlSemanticContextInput::new(vec![]).with_output_defaults(facts)
    } else {
        HtmlSemanticContextInput::new(vec![])
    };
    HtmlSemanticContext::new(c, c.host_facts_revision().clone(), input, 0).unwrap()
}
fn record(
    id: &HtmlCaptureSubjectId,
    value: HtmlOutputDefaultValueOverride,
) -> HtmlOutputDefaultFact {
    HtmlOutputDefaultFact::new(id.clone(), value)
}
fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(c, cx, HtmlValueInterpreter::new(0), l, cancel).unwrap()
}
fn req(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn report(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
) -> HtmlSemanticReport {
    session(c, cx, limits(), HtmlSemanticCancellation::new())
        .query(&[req(id, q)])
        .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlControlTextReadback {
    match r.results()[0].fact() {
        HtmlSemanticFact::TextareaDefaultValue(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::OutputValue(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::OutputDefaultValue(HtmlSemanticValue::Known(v)) => v,
        f => panic!("expected known text readback: {f:?}"),
    }
}
fn leaf(name: &str, v: &[DocumentString]) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let p = html(&mut b, &d, name);
    let ts: Vec<_> = v.iter().map(|v| text(&mut b, &d, v.clone())).collect();
    children(&mut b, &p, &ts);
    let c = b.finish().unwrap();
    let id = c.subject_id(&p).unwrap();
    (c, id)
}

#[test]
fn textarea_default_is_only_immediate_text_exact_utf16_and_ignores_unknown_descendants() {
    let (mut b, d) = builder();
    let ta = html(&mut b, &d, "textarea");
    let a = text(&mut b, &d, " A\r\n");
    let nested = html(&mut b, &d, "span");
    let ignored = text(&mut b, &d, "nested ignored");
    children(&mut b, &nested, &[ignored]);
    let gap = html(&mut b, &d, "div"); // Unavailable descendants have no influence.
    let exact = text(
        &mut b,
        &d,
        DocumentString::from_utf16(vec![0, 0xd800, 0xdc00]),
    );
    let comment = b.add_comment(&d, "ignored comment".into()).unwrap();
    children(&mut b, &ta, &[a, nested, gap, exact, comment]);
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    let r = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(limits().work(), 1, 0, 0, 100000, 100),
        HtmlSemanticCancellation::new(),
    )
    .query(&[req(
        &c.subject_id(&ta).unwrap(),
        HtmlSemanticQuery::TextareaDefaultValue,
    )])
    .unwrap();
    assert_eq!(
        value(&r).value().code_units(),
        [0x20, 0x41, 0x0d, 0x0a, 0, 0xd800, 0xdc00]
    );
    assert_eq!(value(&r).origin(), HtmlControlTextOrigin::ChildText);
    assert_eq!(r.usage().peak_scratch_records(), 0);
}

#[test]
fn output_value_includes_script_img_foreign_text_without_whitespace_or_separator_rules() {
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "output");
    let first = text(&mut b, &d, " A\r\n");
    let mut cs = vec![first];
    for (ns, name, v) in [
        (Namespace::html(), "script", "script"),
        (Namespace::html(), "img", "image"),
        (Namespace::from_uri("urn:x"), "foreign", "foreign"),
    ] {
        let p = element(&mut b, &d, ns, name);
        let t = text(&mut b, &d, v);
        children(&mut b, &p, &[t]);
        cs.push(p);
    }
    let br = html(&mut b, &d, "br");
    children(&mut b, &br, &[]);
    cs.push(br);
    cs.push(text(
        &mut b,
        &d,
        DocumentString::from_utf16(vec![0xa0, 0, 0xd800]),
    ));
    cs.push(b.add_comment(&d, "ignored".into()).unwrap());
    cs.push(
        b.add_processing_instruction(&d, "x".into(), "ignored".into())
            .unwrap(),
    );
    children(&mut b, &o, &cs);
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    let r = report(
        &c,
        &cx,
        &c.subject_id(&o).unwrap(),
        HtmlSemanticQuery::OutputValue,
    );
    let mut expected: Vec<_> = " A\r\nscriptimageforeign".encode_utf16().collect();
    expected.extend([0xa0, 0, 0xd800]);
    assert_eq!(value(&r).value().code_units(), expected);
    assert_eq!(value(&r).origin(), HtmlControlTextOrigin::DescendantText);
    assert_eq!(r.usage().scratch_records(), 0);
}

#[test]
fn missing_override_is_not_inferred_null_even_with_complete_text() {
    let (c, o) = leaf("output", &["text".into()]);
    for records in [
        None,
        Some(vec![]),
        Some(vec![record(
            &o,
            HtmlOutputDefaultValueOverride::Unavailable,
        )]),
    ] {
        let cx = context(&c, records);
        let r = report(&c, &cx, &o, HtmlSemanticQuery::OutputDefaultValue);
        assert_eq!(
            r.results()[0].fact(),
            &HtmlSemanticFact::OutputDefaultValue(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::OutputDefaultValueOverride(o.clone())
            ))
        );
        assert_eq!(r.usage().nested_output_records(), 0);
        assert_eq!(r.usage().peak_scratch_records(), 0);
        assert_eq!(
            value(&report(&c, &cx, &o, HtmlSemanticQuery::OutputValue)).value(),
            &DocumentString::from("text")
        );
    }
}

#[test]
fn established_null_uses_descendant_text_and_preserves_fallback_origin() {
    let (c, o) = leaf("output", &[" \r\ntext\t ".into()]);
    let cx = context(
        &c,
        Some(vec![record(&o, HtmlOutputDefaultValueOverride::Null)]),
    );
    let r = report(&c, &cx, &o, HtmlSemanticQuery::OutputDefaultValue);
    assert_eq!(value(&r).value(), &DocumentString::from(" \r\ntext\t "));
    assert_eq!(value(&r).origin(), HtmlControlTextOrigin::DescendantText);
}

#[test]
fn non_null_empty_and_exact_override_share_backing_without_unknown_children_or_ancestry() {
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "output");
    let c = b.finish().unwrap();
    let id = c.subject_id(&o).unwrap();
    for v in [
        DocumentString::from(""),
        DocumentString::from_utf16(vec![0x0d, 0x0a, 0, 0xd800]),
    ] {
        let cx = context(
            &c,
            Some(vec![record(
                &id,
                HtmlOutputDefaultValueOverride::Value(v.clone()),
            )]),
        );
        let l = HtmlSemanticLimits::new(limits().work(), 1, 0, 0, 100000, 0);
        let r = session(&c, &cx, l, HtmlSemanticCancellation::new())
            .query(&[req(&id, HtmlSemanticQuery::OutputDefaultValue)])
            .unwrap();
        assert_eq!(value(&r).value(), &v);
        assert_eq!(
            value(&r).origin(),
            HtmlControlTextOrigin::OutputDefaultOverride
        );
        assert_eq!(r.usage().nested_output_records(), 0);
        assert_eq!(r.usage().peak_scratch_records(), 0);
        assert_eq!(
            value(&r).value().code_units().as_ptr(),
            v.code_units().as_ptr()
        );
    }
}

#[test]
fn output_value_remains_current_text_even_when_override_has_different_default() {
    let (c, o) = leaf("output", &["current".into()]);
    let cx = context(
        &c,
        Some(vec![record(
            &o,
            HtmlOutputDefaultValueOverride::Value("prior".into()),
        )]),
    );
    assert_eq!(
        value(&report(&c, &cx, &o, HtmlSemanticQuery::OutputValue)).value(),
        &DocumentString::from("current")
    );
    assert_eq!(
        value(&report(&c, &cx, &o, HtmlSemanticQuery::OutputDefaultValue)).value(),
        &DocumentString::from("prior")
    );
}

#[test]
fn required_child_or_descendant_coverage_is_missing_and_never_partial_text() {
    for name in ["textarea", "output"] {
        let (mut b, d) = builder();
        let p = html(&mut b, &d, name);
        let c = b.finish().unwrap();
        let cx = context(&c, None);
        let id = c.subject_id(&p).unwrap();
        let q = if name == "textarea" {
            HtmlSemanticQuery::TextareaDefaultValue
        } else {
            HtmlSemanticQuery::OutputValue
        };
        let r = report(&c, &cx, &id, q);
        let expected = HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(id));
        assert_eq!(
            r.results()[0].fact(),
            &if name == "textarea" {
                HtmlSemanticFact::TextareaDefaultValue(expected)
            } else {
                HtmlSemanticFact::OutputValue(expected)
            }
        );
    }
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "output");
    let t = text(&mut b, &d, "prefix");
    let gap = html(&mut b, &d, "div");
    children(&mut b, &o, &[t, gap.clone()]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&o).unwrap();
    let cx = context(
        &c,
        Some(vec![record(&id, HtmlOutputDefaultValueOverride::Null)]),
    );
    for q in [
        HtmlSemanticQuery::OutputValue,
        HtmlSemanticQuery::OutputDefaultValue,
    ] {
        let r = report(&c, &cx, &id, q);
        let expected = HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&gap).unwrap(),
        ));
        assert_eq!(
            r.results()[0].fact(),
            &if q == HtmlSemanticQuery::OutputValue {
                HtmlSemanticFact::OutputValue(expected)
            } else {
                HtmlSemanticFact::OutputDefaultValue(expected)
            }
        );
        assert!(r.usage().nested_output_records() >= 6);
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn empty_or_nontext_only_complete_children_prove_empty_without_output_buffers() {
    for name in ["textarea", "output"] {
        let (mut b, d) = builder();
        let p = html(&mut b, &d, name);
        let comment = b.add_comment(&d, "not text".into()).unwrap();
        children(&mut b, &p, &[comment]);
        let c = b.finish().unwrap();
        let cx = context(&c, None);
        let q = if name == "textarea" {
            HtmlSemanticQuery::TextareaDefaultValue
        } else {
            HtmlSemanticQuery::OutputValue
        };
        let r = session(
            &c,
            &cx,
            HtmlSemanticLimits::new(limits().work(), 1, 10, 0, 100000, 0),
            HtmlSemanticCancellation::new(),
        )
        .query(&[req(&c.subject_id(&p).unwrap(), q)])
        .unwrap();
        assert!(value(&r).value().is_empty());
        assert_eq!(r.usage().nested_output_records(), 0);
    }
    for name in ["textarea", "output"] {
        let (c, id) = leaf(name, &[]);
        let cx = context(&c, None);
        let q = if name == "textarea" {
            HtmlSemanticQuery::TextareaDefaultValue
        } else {
            HtmlSemanticQuery::OutputValue
        };
        assert!(value(&report(&c, &cx, &id, q)).value().is_empty());
    }
}

#[test]
fn shadow_and_template_associated_text_is_excluded_but_ordinary_host_text_remains() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let o = html(&mut b, &d, "output");
        let host = html(&mut b, &d, if shadow { "div" } else { "template" });
        let f = if shadow {
            b.add_shadow_root(
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
        let excluded = text(&mut b, &d, "excluded");
        children(&mut b, &f, &[excluded]);
        let included = text(&mut b, &d, "included");
        children(&mut b, &host, &[included]);
        children(&mut b, &o, &[host]);
        let c = b.finish().unwrap();
        let cx = context(&c, None);
        assert_eq!(
            value(&report(
                &c,
                &cx,
                &c.subject_id(&o).unwrap(),
                HtmlSemanticQuery::OutputValue
            ))
            .value(),
            &DocumentString::from("included")
        );
    }
}

#[test]
fn wrong_roles_and_foreign_subjects_need_no_output_state_or_child_facts() {
    let (mut b, d) = builder();
    let div = html(&mut b, &d, "div");
    let upper = html(&mut b, &d, "OUTPUT");
    let custom = html(&mut b, &d, "x-output");
    let ta = html(&mut b, &d, "textarea");
    let o = html(&mut b, &d, "output");
    let foreign = element(&mut b, &d, Namespace::from_uri("urn:x"), "output");
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    for q in [
        HtmlSemanticQuery::TextareaDefaultValue,
        HtmlSemanticQuery::OutputValue,
        HtmlSemanticQuery::OutputDefaultValue,
    ] {
        for x in [
            &d,
            &div,
            &upper,
            &custom,
            if q == HtmlSemanticQuery::TextareaDefaultValue {
                &o
            } else {
                &ta
            },
        ] {
            let r = report(&c, &cx, &c.subject_id(x).unwrap(), q);
            assert!(matches!(
                r.results()[0].fact(),
                HtmlSemanticFact::TextareaDefaultValue(HtmlSemanticValue::Inapplicable)
                    | HtmlSemanticFact::OutputValue(HtmlSemanticValue::Inapplicable)
                    | HtmlSemanticFact::OutputDefaultValue(HtmlSemanticValue::Inapplicable)
            ));
            assert_eq!(r.usage().nested_output_records(), 0);
            assert_eq!(r.usage().peak_scratch_records(), 0);
        }
        let r = report(&c, &cx, &c.subject_id(&foreign).unwrap(), q);
        assert!(matches!(
            r.results()[0].fact(),
            HtmlSemanticFact::TextareaDefaultValue(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace
            )) | HtmlSemanticFact::OutputValue(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace
            )) | HtmlSemanticFact::OutputDefaultValue(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace
            ))
        ));
    }
}

#[test]
fn output_facts_reject_wrong_roles_namespaces_duplicates_foreign_ids_and_host_revisions() {
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "output");
    let wrong = html(&mut b, &d, "textarea");
    let upper = html(&mut b, &d, "OUTPUT");
    let foreign = element(&mut b, &d, Namespace::from_uri("urn:x"), "output");
    let c = b.finish().unwrap();
    let id = c.subject_id(&o).unwrap();
    for x in [&d, &wrong, &upper, &foreign] {
        let id = c.subject_id(x).unwrap();
        let e = HtmlOutputDefaultFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![record(&id, HtmlOutputDefaultValueOverride::Null)],
            fact_limits(),
        )
        .unwrap_err();
        assert_eq!(e, HtmlOutputDefaultError::InapplicableSubject(id));
    }
    let e = HtmlOutputDefaultFacts::new(
        &c,
        c.host_facts_revision().clone(),
        vec![
            record(&id, HtmlOutputDefaultValueOverride::Null),
            record(&id, HtmlOutputDefaultValueOverride::Unavailable),
        ],
        fact_limits(),
    )
    .unwrap_err();
    assert_eq!(e, HtmlOutputDefaultError::DuplicateSubject(id.clone()));
    let (_, other) = leaf("output", &[]);
    assert!(matches!(
        HtmlOutputDefaultFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![record(&other, HtmlOutputDefaultValueOverride::Null)],
            fact_limits()
        ),
        Err(HtmlOutputDefaultError::Capture(_))
    ));
    assert!(matches!(
        HtmlOutputDefaultFacts::new(&c, HtmlHostFactsRevision::fresh(), vec![], fact_limits()),
        Err(HtmlOutputDefaultError::HostRevisionMismatch)
    ));
    let facts = HtmlOutputDefaultFacts::new(
        &c,
        c.host_facts_revision().clone(),
        vec![record(&id, HtmlOutputDefaultValueOverride::Null)],
        fact_limits(),
    )
    .unwrap();
    let (other, _) = leaf("output", &[]);
    assert!(matches!(
        HtmlSemanticContext::new(
            &other,
            other.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_output_defaults(facts),
            0
        ),
        Err(HtmlSemanticContextError::OutputDefaultsCaptureMismatch)
    ));
}

#[test]
fn moved_capacity_logical_units_and_validation_work_have_exact_one_short_boundaries() {
    let (c, id) = leaf("output", &[]);
    let make = || {
        let mut v = Vec::with_capacity(8);
        v.push(record(
            &id,
            HtmlOutputDefaultValueOverride::Value(DocumentString::from_utf16(vec![
                0, 0xd800, 0xdc00,
            ])),
        ));
        v
    };
    let baseline =
        HtmlOutputDefaultFacts::new(&c, c.host_facts_revision().clone(), make(), fact_limits())
            .unwrap();
    let work = baseline.validation_work();
    assert_eq!(baseline.records()[0].subject(), &id);
    assert_eq!(baseline.capture(), c.identity());
    assert_eq!(baseline.host_revision(), c.host_facts_revision());
    for (caps, expected) in [
        (
            HtmlOutputDefaultLimits::new(7, 3, work),
            HtmlOutputDefaultError::RecordCapacity {
                limit: 7,
                required: 8,
            },
        ),
        (
            HtmlOutputDefaultLimits::new(8, 2, work),
            HtmlOutputDefaultError::CodeUnits {
                limit: 2,
                required: 3,
            },
        ),
        (
            HtmlOutputDefaultLimits::new(8, 3, work - 1),
            HtmlOutputDefaultError::ValidationWork {
                limit: work - 1,
                required: work,
            },
        ),
    ] {
        assert_eq!(
            HtmlOutputDefaultFacts::new(&c, c.host_facts_revision().clone(), make(), caps)
                .unwrap_err(),
            expected
        );
    }
    assert!(
        HtmlOutputDefaultFacts::new(
            &c,
            c.host_facts_revision().clone(),
            make(),
            HtmlOutputDefaultLimits::new(8, 3, work)
        )
        .is_ok()
    );
    let f = HtmlOutputDefaultFacts::new(
        &c,
        c.host_facts_revision().clone(),
        vec![record(
            &id,
            HtmlOutputDefaultValueOverride::Value("".into()),
        )],
        HtmlOutputDefaultLimits::new(1, 0, 100000),
    )
    .unwrap();
    assert_eq!(
        f.records()[0].default_value_override(),
        &HtmlOutputDefaultValueOverride::Value("".into())
    );
}

#[test]
fn logical_override_units_are_cumulative_even_when_backing_is_shared() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "output");
    let bnode = html(&mut b, &d, "output");
    let c = b.finish().unwrap();
    let shared = DocumentString::from("ab");
    let records = vec![
        record(
            &c.subject_id(&a).unwrap(),
            HtmlOutputDefaultValueOverride::Value(shared.clone()),
        ),
        record(
            &c.subject_id(&bnode).unwrap(),
            HtmlOutputDefaultValueOverride::Value(shared),
        ),
    ];
    assert!(matches!(
        HtmlOutputDefaultFacts::new(
            &c,
            c.host_facts_revision().clone(),
            records,
            HtmlOutputDefaultLimits::new(2, 3, 100000)
        ),
        Err(HtmlOutputDefaultError::CodeUnits {
            limit: 3,
            required: 4
        })
    ));
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
        0,
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
fn check_caps(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
) {
    let u = report(c, cx, id, q).usage();
    let request = req(id, q);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, u.work()),
        (HtmlSemanticResource::OutputRecords, u.output_records()),
        (
            HtmlSemanticResource::ScratchRecords,
            u.peak_scratch_records(),
        ),
        (
            HtmlSemanticResource::NestedOutputRecords,
            u.nested_output_records(),
        ),
        (HtmlSemanticResource::AllocatedBytes, u.allocated_bytes()),
    ] {
        if exact == 0 {
            continue;
        }
        for cap in [exact, exact - 1] {
            let mut ss = session(
                c,
                cx,
                limited(resource, cap),
                HtmlSemanticCancellation::new(),
            );
            let r = ss.query(std::slice::from_ref(&request));
            if cap == exact {
                assert!(r.is_ok(), "{q:?} {resource:?}: {r:?}");
            } else {
                let f = r.unwrap_err();
                assert!(
                    matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                    "{q:?} {resource:?}: {f:?}"
                );
                assert_eq!(f.qualification().unwrap().capture(), c.identity());
                assert_eq!(f.qualification().unwrap().context(), cx.identity());
                assert_eq!(f.qualification().unwrap().limits(), limited(resource, cap));
                assert_eq!(ss.query(&[]).unwrap_err(), f);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}

#[test]
fn actual_text_and_override_work_output_scratch_capacity_bytes_caps_are_exact() {
    for name in ["textarea", "output"] {
        let (c, id) = leaf(name, &[" \r\nAB\t ".into()]);
        let cx = context(
            &c,
            if name == "output" {
                Some(vec![record(&id, HtmlOutputDefaultValueOverride::Null)])
            } else {
                None
            },
        );
        let q = if name == "textarea" {
            HtmlSemanticQuery::TextareaDefaultValue
        } else {
            HtmlSemanticQuery::OutputValue
        };
        check_caps(&c, &cx, &id, q);
        if name == "output" {
            check_caps(&c, &cx, &id, HtmlSemanticQuery::OutputDefaultValue);
            let cx = context(
                &c,
                Some(vec![record(
                    &id,
                    HtmlOutputDefaultValueOverride::Value("override".into()),
                )]),
            );
            check_caps(&c, &cx, &id, HtmlSemanticQuery::OutputDefaultValue);
        }
    }
}

#[test]
fn late_mixed_batch_failure_returns_no_prefix_and_releases_scratch() {
    let (c, id) = leaf("output", &["ABC".into()]);
    let cx = context(&c, None);
    let request = req(&id, HtmlSemanticQuery::OutputValue);
    let mut ss = session(
        &c,
        &cx,
        limited(HtmlSemanticResource::NestedOutputRecords, 3),
        HtmlSemanticCancellation::new(),
    );
    let f = ss
        .query(&[
            req(&id, HtmlSemanticQuery::Hidden),
            request.clone(),
            request,
        ])
        .unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(f.usage().nested_output_records(), 3);
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}

#[test]
fn cancellation_and_foreign_requests_retain_original_qualification_and_stop() {
    let (c, id) = leaf("output", &["A".into()]);
    let cx = context(&c, None);
    let cancel = HtmlSemanticCancellation::new();
    let mut ss = session(&c, &cx, limits(), cancel.clone());
    ss.query(&[req(&id, HtmlSemanticQuery::OutputValue)])
        .unwrap();
    cancel.cancel();
    let f = ss
        .query(&[req(&id, HtmlSemanticQuery::OutputDefaultValue)])
        .unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap().context(), cx.identity());
    assert_eq!(
        f.qualification().unwrap().host_revision(),
        c.host_facts_revision()
    );
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    let (_, foreign) = leaf("output", &[]);
    let mut ss = session(&c, &cx, limits(), HtmlSemanticCancellation::new());
    let f = ss
        .query(&[
            req(&id, HtmlSemanticQuery::OutputValue),
            req(&foreign, HtmlSemanticQuery::OutputValue),
        ])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}

#[test]
fn state_only_override_context_changes_and_retained_stops_cannot_be_misjoined() {
    let (c, id) = leaf("output", &["captured".into()]);
    let a = context(
        &c,
        Some(vec![record(&id, HtmlOutputDefaultValueOverride::Null)]),
    );
    let b = context(
        &c,
        Some(vec![record(
            &id,
            HtmlOutputDefaultValueOverride::Value("historical".into()),
        )]),
    );
    let ar = report(&c, &a, &id, HtmlSemanticQuery::OutputDefaultValue);
    let br = report(&c, &b, &id, HtmlSemanticQuery::OutputDefaultValue);
    assert_eq!(value(&ar).value(), &DocumentString::from("captured"));
    assert_eq!(value(&br).value(), &DocumentString::from("historical"));
    assert_eq!(ar.capture(), br.capture());
    assert_ne!(ar.context(), br.context());
    assert_eq!(
        ar.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let mut ss = session(
        &c,
        &a,
        limited(HtmlSemanticResource::Work, 0),
        HtmlSemanticCancellation::new(),
    );
    let f = ss
        .query(&[req(&id, HtmlSemanticQuery::OutputDefaultValue)])
        .unwrap_err();
    assert_eq!(f.qualification().unwrap().context(), a.identity());
    assert_ne!(f.qualification().unwrap().context(), b.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}

#[test]
fn changed_text_topology_and_capture_keys_recompute_exact_outputs() {
    let mut reports = Vec::new();
    for (nested, v) in [(false, "A"), (true, "A"), (false, "B")] {
        let (mut b, d) = builder();
        let p = html(&mut b, &d, "textarea");
        let t = text(&mut b, &d, v);
        if nested {
            let span = html(&mut b, &d, "span");
            children(&mut b, &span, &[t]);
            children(&mut b, &p, &[span]);
        } else {
            children(&mut b, &p, &[t]);
        }
        let c = b.finish().unwrap();
        let cx = context(&c, None);
        let r = report(
            &c,
            &cx,
            &c.subject_id(&p).unwrap(),
            HtmlSemanticQuery::TextareaDefaultValue,
        );
        assert_eq!(
            value(&r).value(),
            &DocumentString::from(if nested { "" } else { v })
        );
        reports.push(r);
    }
    for pair in reports.windows(2) {
        assert_ne!(pair[0].capture(), pair[1].capture());
        assert_ne!(pair[0].host_revision(), pair[1].host_revision());
    }
}

#[test]
fn deep_borrowed_output_walk_is_iterative_and_cleans_up_on_limit() {
    let (mut b, d) = builder();
    let o = html(&mut b, &d, "output");
    let mut p = o.clone();
    for _ in 0..80 {
        let next = html(&mut b, &d, "div");
        children(&mut b, &p, std::slice::from_ref(&next));
        p = next;
    }
    let t = text(&mut b, &d, "end");
    children(&mut b, &p, &[t]);
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    let id = c.subject_id(&o).unwrap();
    let r = report(&c, &cx, &id, HtmlSemanticQuery::OutputValue);
    assert_eq!(value(&r).value(), &DocumentString::from("end"));
    assert!(r.usage().peak_scratch_records() >= 81);
    assert_eq!(r.usage().scratch_records(), 0);
    let mut ss = session(
        &c,
        &cx,
        limited(HtmlSemanticResource::ScratchRecords, 40),
        HtmlSemanticCancellation::new(),
    );
    let f = ss
        .query(&[req(&id, HtmlSemanticQuery::OutputValue)])
        .unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::ScratchRecords,
            ..
        }
    ));
    assert_eq!(ss.usage().scratch_records(), 0);
}
