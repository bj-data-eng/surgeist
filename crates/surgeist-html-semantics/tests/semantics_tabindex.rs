use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlSemanticCancellation, HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticUnsupported, HtmlSemanticValue,
    HtmlTabIndexOrigin, HtmlTabIndexReadback,
};
#[path = "common/builder.rs"]
mod shared_builder;
use shared_builder::builder;
#[path = "common/context.rs"]
mod shared_context;
use shared_context::context;
#[path = "common/session.rs"]
mod shared_session;
use shared_session::session;
use surgeist_html::*;

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    local: &str,
    attrs: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, local),
        attrs,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    element(
        b,
        d,
        Namespace::html(),
        local,
        OrderedAttributes::new(
            attrs
                .iter()
                .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
        )
        .unwrap(),
    )
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    cs: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, cs).unwrap();
}

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 1000000, 1000000, 100000)
}

fn request(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn query(
    c: &HtmlCapture,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
    raw: usize,
) -> HtmlSemanticReport {
    let cx = context(c);
    session(c, &cx, limits(), raw, HtmlSemanticCancellation::new())
        .query(&[request(id, q)])
        .unwrap()
}
fn readback(r: &HtmlSemanticReport) -> &HtmlTabIndexReadback {
    match r.results()[0].fact() {
        HtmlSemanticFact::TabIndex(HtmlSemanticValue::Known(v)) => v,
        other => panic!("expected known getter, got {other:?}"),
    }
}
fn fixture(
    ns: Namespace,
    local: &str,
    raw: Option<DocumentString>,
) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let attrs =
        OrderedAttributes::new(raw.into_iter().map(|raw| {
            Attribute::new(QualifiedName::new(Namespace::none(), None, "tabindex"), raw)
        }))
        .unwrap();
    let e = element(&mut b, &d, ns, local, attrs);
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    (c, id)
}
fn get(c: &HtmlCapture, id: &HtmlCaptureSubjectId, raw: usize) -> HtmlSemanticReport {
    query(c, id, HtmlSemanticQuery::TabIndex, raw)
}
#[test]
fn exact_signed_long_boundaries_recovery_and_original_raw() {
    for (raw, value) in [
        ("2147483647", i32::MAX),
        ("-2147483648", i32::MIN),
        ("+12", 12),
        (" \t\n\u{000c}\r-42tail", -42),
        ("00000000000000000000000007", 7),
        ("-0", 0),
        ("1.8", 1),
    ] {
        let (c, id) = fixture(Namespace::html(), "summary", Some(raw.into()));
        let r = get(&c, &id, raw.len());
        let v = readback(&r);
        assert_eq!(v.value(), value, "{raw:?}");
        assert_eq!(v.origin(), HtmlTabIndexOrigin::Recovered);
        assert_eq!(
            v.attribute().unwrap().value().code_units(),
            raw.encode_utf16().collect::<Vec<_>>()
        );
    }
}
#[test]
fn fallback_origins_are_distinct_without_clamping_or_wrapping() {
    for local in ["div", "input"] {
        for (raw, origin) in [
            (None, HtmlTabIndexOrigin::MissingDefault),
            (Some(""), HtmlTabIndexOrigin::ParseFailureDefault),
            (Some("+"), HtmlTabIndexOrigin::ParseFailureDefault),
            (Some("\u{00a0}1"), HtmlTabIndexOrigin::ParseFailureDefault),
            (Some("2147483648"), HtmlTabIndexOrigin::OutOfRangeDefault),
            (Some("-2147483649"), HtmlTabIndexOrigin::OutOfRangeDefault),
            (
                Some("99999999999999999999999999"),
                HtmlTabIndexOrigin::OutOfRangeDefault,
            ),
        ] {
            let (c, id) = fixture(Namespace::html(), local, raw.map(Into::into));
            let r = get(&c, &id, 100);
            let v = readback(&r);
            assert_eq!(v.value(), if local == "input" { 0 } else { -1 });
            assert_eq!(v.origin(), origin);
            assert_eq!(v.attribute().is_some(), raw.is_some());
        }
    }
}
#[test]
fn all_source_html_defaults_include_obsolete_frame_and_ignore_unrelated_content() {
    for local in [
        "a", "area", "button", "frame", "iframe", "input", "object", "select", "textarea",
    ] {
        let (mut b, d) = builder();
        let e = html(
            &mut b,
            &d,
            local,
            &[
                ("href", "huge unconsulted"),
                ("type", "hidden"),
                ("disabled", "bad"),
                ("inert", "bad"),
            ],
        );
        let c = b.finish().unwrap();
        let id = c.subject_id(&e).unwrap();
        assert_eq!(readback(&get(&c, &id, 0)).value(), 0, "{local}");
    }
    for local in ["div", "x-custom", "FRAME", "INPUT", "summary"] {
        let (mut b, d) = builder();
        let e = html(&mut b, &d, local, &[]);
        b.establish_root(&e).unwrap();
        let c = b.finish().unwrap();
        let id = c.subject_id(&e).unwrap();
        assert_eq!(readback(&get(&c, &id, 0)).value(), -1, "{local}");
    }
}
#[test]
fn svg_and_mathml_interfaces_have_exact_a_default_and_explicit_values() {
    for ns in [Namespace::svg(), Namespace::mathml()] {
        for (local, expected) in [
            ("a", 0),
            ("A", -1),
            ("input", -1),
            ("summary", -1),
            ("frame", -1),
        ] {
            let (c, id) = fixture(ns.clone(), local, None);
            assert_eq!(readback(&get(&c, &id, 0)).value(), expected);
            let (c, id) = fixture(ns.clone(), local, Some("-9".into()));
            assert_eq!(readback(&get(&c, &id, 2)).value(), -9);
        }
    }
}
#[test]
fn unsupported_interfaces_non_elements_and_exact_attribute_names() {
    for ns in [Namespace::none(), Namespace::from_uri("urn:other")] {
        let (c, id) = fixture(ns, "a", Some("1".into()));
        assert!(matches!(
            get(&c, &id, 0).results()[0].fact(),
            HtmlSemanticFact::TabIndex(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace
            ))
        ));
    }
    let (mut b, d) = builder();
    let e = element(
        &mut b,
        &d,
        Namespace::html(),
        "a",
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "tabindex"),
                "999999",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "tabIndex"),
                "999999",
            ),
        ])
        .unwrap(),
    );
    let t = b.add_text(&d, "huge irrelevant text".into()).unwrap();
    let c = b.finish().unwrap();
    let r = get(&c, &c.subject_id(&e).unwrap(), 0);
    assert_eq!(readback(&r).origin(), HtmlTabIndexOrigin::MissingDefault);
    for h in [&d, &t] {
        assert!(matches!(
            get(&c, &c.subject_id(h).unwrap(), 0).results()[0].fact(),
            HtmlSemanticFact::TabIndex(HtmlSemanticValue::Inapplicable)
        ));
    }
}
#[test]
fn summary_defaults_use_first_direct_summary_and_skip_later_unknown_subtrees() {
    let (mut b, d) = builder();
    let p = html(&mut b, &d, "details", &[]);
    let div = html(&mut b, &d, "div", &[]);
    let wrapped = html(&mut b, &d, "summary", &[]);
    children(&mut b, &div, std::slice::from_ref(&wrapped));
    let first = html(&mut b, &d, "summary", &[]);
    let second = html(&mut b, &d, "summary", &[("tabindex", "bad")]);
    children(&mut b, &p, &[div, first.clone(), second.clone()]);
    let c = b.finish().unwrap();
    for (h, v) in [(&first, 0), (&second, -1), (&wrapped, -1)] {
        assert_eq!(readback(&get(&c, &c.subject_id(h).unwrap(), 3)).value(), v);
    }
}
#[test]
fn summary_missing_ancestry_and_order_are_influencing_only_when_defaulting() {
    let (c, id) = fixture(Namespace::html(), "summary", None);
    assert!(matches!(
        get(&c, &id, 0).results()[0].fact(),
        HtmlSemanticFact::TabIndex(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(_)))
    ));
    for raw in ["bad", "2147483648"] {
        let (c, id) = fixture(Namespace::html(), "summary", Some(raw.into()));
        assert!(matches!(
            get(&c, &id, 100).results()[0].fact(),
            HtmlSemanticFact::TabIndex(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(_)))
        ));
    }
    let (mut b, d) = builder();
    let p = html(&mut b, &d, "details", &[]);
    let s = html(&mut b, &d, "summary", &[]);
    b.establish_parent(&s, &p).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&s).unwrap();
    assert!(matches!(
        get(&c, &id, 0).results()[0].fact(),
        HtmlSemanticFact::TabIndex(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(_)
        ))
    ));
}
#[test]
fn exact_utf16_prefix_recovery_retains_surrogates_and_admits_ignored_suffix() {
    let raw = DocumentString::from_utf16(vec![49, 0xd800, 0xdc00]);
    let (c, id) = fixture(Namespace::html(), "summary", Some(raw.clone()));
    let r = get(&c, &id, 3);
    assert_eq!(readback(&r).value(), 1);
    assert_eq!(readback(&r).attribute().unwrap().value(), &raw);
    let cx = context(&c);
    let mut ss = session(&c, &cx, limits(), 2, HtmlSemanticCancellation::new());
    assert!(matches!(
        ss.query(&[request(&id, HtmlSemanticQuery::TabIndex)])
            .unwrap_err()
            .kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    let (c, id) = fixture(
        Namespace::html(),
        "div",
        Some(DocumentString::from_utf16(vec![0xd800, 49])),
    );
    assert_eq!(
        readback(&get(&c, &id, 2)).origin(),
        HtmlTabIndexOrigin::ParseFailureDefault
    );
}
fn capped(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
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
        0,
        0,
        if resource == HtmlSemanticResource::AllocatedBytes {
            cap
        } else {
            l.allocated_bytes()
        },
        0,
    )
}
#[test]
fn fixed_numeric_projection_ignores_digit_and_scratch_caps_including_huge_inputs() {
    for raw in [format!("{}7", "0".repeat(10000)), "9".repeat(10000)] {
        let (c, id) = fixture(Namespace::html(), "input", Some(raw.as_str().into()));
        let cx = context(&c);
        let r = session(
            &c,
            &cx,
            capped(HtmlSemanticResource::Work, 20000000),
            raw.len(),
            HtmlSemanticCancellation::new(),
        )
        .query(&[request(&id, HtmlSemanticQuery::TabIndex)])
        .unwrap();
        assert_eq!(readback(&r).value(), if raw.ends_with('7') { 7 } else { 0 });
        assert_eq!(r.usage().scratch_records(), 0);
        assert_eq!(r.usage().peak_scratch_records(), 0);
        assert_eq!(r.usage().nested_output_records(), 0);
    }
}
#[test]
fn exact_and_one_short_work_output_and_observed_bytes_retain_qualified_stop() {
    let (c, id) = fixture(Namespace::html(), "input", Some("+7tail".into()));
    let cx = context(&c);
    let req = request(&id, HtmlSemanticQuery::TabIndex);
    let baseline = get(&c, &id, 6).usage();
    for (resource, cap) in [
        (HtmlSemanticResource::Work, baseline.work()),
        (HtmlSemanticResource::OutputRecords, 1),
        (
            HtmlSemanticResource::AllocatedBytes,
            baseline.allocated_bytes(),
        ),
    ] {
        session(
            &c,
            &cx,
            capped(resource, cap),
            6,
            HtmlSemanticCancellation::new(),
        )
        .query(std::slice::from_ref(&req))
        .unwrap();
        let mut ss = session(
            &c,
            &cx,
            capped(resource, cap - 1),
            6,
            HtmlSemanticCancellation::new(),
        );
        let f = ss.query(std::slice::from_ref(&req)).unwrap_err();
        assert!(
            matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..}if *r==resource)
        );
        assert_eq!(f.qualification().unwrap().capture(), c.identity());
        assert_eq!(ss.query(&[]).unwrap_err(), f);
        assert_eq!(ss.usage().scratch_records(), 0);
    }
}
#[test]
fn cancellation_and_late_batch_failure_are_atomic_and_sticky() {
    let (c, id) = fixture(Namespace::html(), "input", Some("7".into()));
    let cx = context(&c);
    let req = request(&id, HtmlSemanticQuery::TabIndex);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), 1, cancel);
    let f = ss.query(std::slice::from_ref(&req)).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    let one = get(&c, &id, 1).usage().work();
    let mut ss = session(
        &c,
        &cx,
        capped(HtmlSemanticResource::Work, one),
        1,
        HtmlSemanticCancellation::new(),
    );
    let f = ss.query(&[req.clone(), req]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
#[test]
fn duplicate_queries_charge_again_and_results_are_exactly_qualified() {
    let (c, id) = fixture(Namespace::html(), "input", Some("7".into()));
    let cx = context(&c);
    let req = request(&id, HtmlSemanticQuery::TabIndex);
    let r = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new())
        .query(&[req.clone(), req])
        .unwrap();
    let one = get(&c, &id, 1);
    assert!(r.usage().work() > one.usage().work());
    assert_eq!(r.usage().output_records(), 2);
    assert_eq!(r.capture(), c.identity());
    assert_eq!(r.host_revision(), c.host_facts_revision());
    assert_eq!(r.context(), cx.identity());
    assert_eq!(
        r.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    assert_eq!(
        HtmlSemanticQuery::TabIndex.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-tabindex"
    );
    let (c2, id2) = fixture(Namespace::html(), "input", Some("8".into()));
    let other = get(&c2, &id2, 1);
    assert_ne!(one.capture(), other.capture());
    assert_eq!(readback(&other).value(), 8);
    let othercx = context(&c);
    let changed = session(&c, &othercx, limits(), 1, HtmlSemanticCancellation::new())
        .query(&[request(&id, HtmlSemanticQuery::TabIndex)])
        .unwrap();
    assert_ne!(one.context(), changed.context());
    let f = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new())
        .query(&[request(&id2, HtmlSemanticQuery::TabIndex)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
}
#[test]
fn prefixes_do_not_change_expanded_names_and_raw_admission_precedes_defaulting() {
    let (mut b, d) = builder();
    let e = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), Some("h".into()), "frame"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), Some("p".into()), "tabindex"),
                "2147483648ignored",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let r = get(&c, &id, 17);
    assert_eq!(readback(&r).value(), 0);
    assert_eq!(readback(&r).origin(), HtmlTabIndexOrigin::OutOfRangeDefault);
    let cx = context(&c);
    let f = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new())
        .query(&[request(&id, HtmlSemanticQuery::TabIndex)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
}
#[test]
fn summary_default_does_not_cross_template_or_shadow_host_associations() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let host = html(&mut b, &d, if shadow { "details" } else { "template" }, &[]);
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
        let s = html(&mut b, &d, "summary", &[]);
        children(&mut b, &f, std::slice::from_ref(&s));
        let c = b.finish().unwrap();
        let id = c.subject_id(&s).unwrap();
        assert_eq!(readback(&get(&c, &id, 0)).value(), -1);
    }
}
#[test]
fn configurations_and_missing_topology_recompute_without_cached_defaults() {
    let (c, id) = fixture(Namespace::html(), "input", Some("1".into()));
    let cx = context(&c);
    let req = request(&id, HtmlSemanticQuery::TabIndex);
    let a = get(&c, &id, 1);
    let b = session(
        &c,
        &cx,
        capped(HtmlSemanticResource::Work, a.usage().work()),
        2,
        HtmlSemanticCancellation::new(),
    )
    .query(std::slice::from_ref(&req))
    .unwrap();
    assert_eq!(readback(&b).value(), 1);
    assert_ne!(a.qualification(), b.qualification());
    assert_eq!(b.qualification().attribute_input_limit(), 2);
    let (mut builder, d) = builder();
    let parent = html(&mut builder, &d, "div", &[]);
    let s = html(&mut builder, &d, "summary", &[]);
    children(&mut builder, &parent, std::slice::from_ref(&s));
    let c = builder.finish().unwrap();
    assert_eq!(
        readback(&get(&c, &c.subject_id(&s).unwrap(), 0)).value(),
        -1
    );
}
