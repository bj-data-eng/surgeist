use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlCustomControlClassification, HtmlSemanticCancellation, HtmlSemanticContext,
    HtmlSemanticContextInput, HtmlSemanticDependencies, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticUnsupported, HtmlSemanticValue,
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

fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::ConstraintValidationCandidate)
}
fn report(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    raw: usize,
) -> HtmlSemanticReport {
    session(c, cx, limits(), raw, HtmlSemanticCancellation::new())
        .query(&[request(id)])
        .unwrap()
}
fn fact(r: &HtmlSemanticReport) -> &HtmlSemanticValue<bool> {
    match r.results()[0].fact() {
        HtmlSemanticFact::ConstraintValidationCandidate(v) => v,
        other => panic!("{other:?}"),
    }
}
fn fixture(local: &str, attrs: &[(&str, &str)], root: bool) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let e = html(&mut b, &d, local, attrs);
    if root {
        b.establish_root(&e).unwrap();
    }
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    (c, id)
}
fn custom_context(c: &HtmlCapture, id: &HtmlCaptureSubjectId, value: bool) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![HtmlCustomControlClassification::new(
            id.clone(),
            value,
        )]),
        10000,
    )
    .unwrap()
}
#[test]
fn native_non_submittable_is_false_without_topology_or_raw_context() {
    for local in [
        "object", "output", "fieldset", "meter", "progress", "div", "form", "option", "img",
        "datalist",
    ] {
        let (c, id) = fixture(
            local,
            &[
                ("type", "huge unread"),
                ("readonly", "huge unread"),
                ("disabled", "huge unread"),
            ],
            false,
        );
        assert_eq!(
            fact(&report(&c, &context(&c), &id, 0)),
            &HtmlSemanticValue::Known(false),
            "{local}"
        );
    }
}
#[test]
fn all_input_types_and_missing_invalid_type_follow_actual_processing() {
    for (raw, expected) in [
        ("hidden", false),
        ("reset", false),
        ("button", false),
        ("text", true),
        ("search", true),
        ("tel", true),
        ("url", true),
        ("email", true),
        ("password", true),
        ("date", true),
        ("month", true),
        ("week", true),
        ("time", true),
        ("datetime-local", true),
        ("number", true),
        ("range", true),
        ("color", true),
        ("checkbox", true),
        ("radio", true),
        ("file", true),
        ("image", true),
        ("submit", true),
        ("", true),
        ("invalid", true),
        ("HIDDEN", false),
    ] {
        let (c, id) = fixture("input", &[("type", raw)], true);
        assert_eq!(
            fact(&report(&c, &context(&c), &id, 20)),
            &HtmlSemanticValue::Known(expected),
            "{raw}"
        );
    }
    let (c, id) = fixture("input", &[], true);
    assert_eq!(
        fact(&report(&c, &context(&c), &id, 0)),
        &HtmlSemanticValue::Known(true)
    );
}
#[test]
fn readonly_barrier_respects_every_input_applicability_state() {
    for raw in [
        "text",
        "search",
        "tel",
        "url",
        "email",
        "password",
        "date",
        "month",
        "week",
        "time",
        "datetime-local",
        "number",
    ] {
        let (c, id) = fixture(
            "input",
            &[("type", raw), ("readonly", "oversized author-invalid")],
            false,
        );
        assert_eq!(
            fact(&report(&c, &context(&c), &id, raw.len())),
            &HtmlSemanticValue::Known(false),
            "{raw}"
        );
    }
    for raw in [
        "checkbox", "radio", "file", "range", "color", "image", "submit",
    ] {
        let (c, id) = fixture(
            "input",
            &[("type", raw), ("readonly", "oversized author-invalid")],
            true,
        );
        assert_eq!(
            fact(&report(&c, &context(&c), &id, raw.len())),
            &HtmlSemanticValue::Known(true),
            "{raw}"
        );
    }
}
#[test]
fn textarea_readonly_presence_and_select_ignored_readonly_need_no_content_scan() {
    for raw in ["", "false", "readonly", " "] {
        let (c, id) = fixture("textarea", &[("readonly", raw)], false);
        assert_eq!(
            fact(&report(&c, &context(&c), &id, 0)),
            &HtmlSemanticValue::Known(false)
        );
        let (c, id) = fixture("select", &[("readonly", raw)], true);
        assert_eq!(
            fact(&report(&c, &context(&c), &id, 0)),
            &HtmlSemanticValue::Known(true)
        );
    }
    let (mut b, d) = builder();
    let e = element(
        &mut b,
        &d,
        Namespace::html(),
        "textarea",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "readonly"),
            DocumentString::from_utf16(vec![0xd800]),
        )])
        .unwrap(),
    );
    let c = b.finish().unwrap();
    assert_eq!(
        fact(&report(&c, &context(&c), &c.subject_id(&e).unwrap(), 0)),
        &HtmlSemanticValue::Known(false)
    );
}
#[test]
fn source_barred_input_types_skip_disabled_and_unknown_ancestry() {
    for raw in ["hidden", "reset", "button"] {
        let (c, id) = fixture(
            "input",
            &[("type", raw), ("disabled", "unread oversized")],
            false,
        );
        assert_eq!(
            fact(&report(&c, &context(&c), &id, raw.len())),
            &HtmlSemanticValue::Known(false)
        );
    }
}
#[test]
fn button_explicit_states_auto_commands_and_select_parent_use_shared_predicate() {
    for (raw, expected) in [
        ("submit", true),
        ("reset", false),
        ("button", false),
        ("", true),
        ("invalid", true),
    ] {
        let (c, id) = fixture("button", &[("type", raw)], true);
        assert_eq!(
            fact(&report(&c, &context(&c), &id, raw.len())),
            &HtmlSemanticValue::Known(expected)
        );
    }
    for command in ["command", "commandfor"] {
        let (c, id) = fixture("button", &[(command, "unread huge value")], false);
        assert_eq!(
            fact(&report(&c, &context(&c), &id, 0)),
            &HtmlSemanticValue::Known(false)
        );
    }
    for raw in [None, Some("submit")] {
        let (mut b, d) = builder();
        let select = html(&mut b, &d, "select", &[]);
        let button = html(
            &mut b,
            &d,
            "button",
            &raw.map(|v| vec![("type", v)]).unwrap_or_default(),
        );
        children(&mut b, &select, std::slice::from_ref(&button));
        b.establish_root(&select).unwrap();
        let c = b.finish().unwrap();
        assert_eq!(
            fact(&report(
                &c,
                &context(&c),
                &c.subject_id(&button).unwrap(),
                6
            )),
            &HtmlSemanticValue::Known(raw.is_some())
        );
    }
}
#[test]
fn auto_button_missing_parent_is_missing_but_own_disabled_proves_false() {
    let (c, id) = fixture("button", &[], false);
    assert!(matches!(
        fact(&report(&c, &context(&c), &id, 0)),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(_))
    ));
    let (c, id) = fixture("button", &[("disabled", "")], false);
    assert_eq!(
        fact(&report(&c, &context(&c), &id, 0)),
        &HtmlSemanticValue::Known(false)
    );
}
#[test]
fn effective_disabledness_preserves_first_legend_exception_and_missing_order() {
    let (mut b, d) = builder();
    let fieldset = html(&mut b, &d, "fieldset", &[("disabled", "")]);
    let legend = html(&mut b, &d, "legend", &[]);
    let protected = html(&mut b, &d, "input", &[]);
    let ordinary = html(&mut b, &d, "input", &[]);
    children(&mut b, &legend, std::slice::from_ref(&protected));
    children(&mut b, &fieldset, &[legend, ordinary.clone()]);
    b.establish_root(&fieldset).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    assert_eq!(
        fact(&report(&c, &cx, &c.subject_id(&protected).unwrap(), 0)),
        &HtmlSemanticValue::Known(true)
    );
    assert_eq!(
        fact(&report(&c, &cx, &c.subject_id(&ordinary).unwrap(), 0)),
        &HtmlSemanticValue::Known(false)
    );
    let (mut b, d) = builder();
    let fieldset = html(&mut b, &d, "fieldset", &[("disabled", "")]);
    let legend = html(&mut b, &d, "legend", &[]);
    let e = html(&mut b, &d, "input", &[]);
    b.establish_parent(&e, &legend).unwrap();
    b.establish_parent(&legend, &fieldset).unwrap();
    b.establish_root(&fieldset).unwrap();
    let c = b.finish().unwrap();
    assert!(matches!(
        fact(&report(&c, &context(&c), &c.subject_id(&e).unwrap(), 0)),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(_))
    ));
}
#[test]
fn known_datalist_ancestor_proves_false_after_disabled_legend_missing() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let f = html(&mut b, &d, "fieldset", &[("disabled", "")]);
    let legend = html(&mut b, &d, "legend", &[]);
    let e = html(&mut b, &d, "input", &[]);
    b.establish_parent(&e, &legend).unwrap();
    b.establish_parent(&legend, &f).unwrap();
    b.establish_parent(&f, &dl).unwrap();
    b.establish_root(&dl).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        fact(&report(&c, &context(&c), &c.subject_id(&e).unwrap(), 0)),
        &HtmlSemanticValue::Known(false)
    );
}
#[test]
fn custom_classification_true_false_unknown_and_independent_barriers() {
    let (c, id) = fixture("x-control", &[], true);
    assert_eq!(
        fact(&report(&c, &custom_context(&c, &id, true), &id, 0)),
        &HtmlSemanticValue::Known(true)
    );
    assert_eq!(
        fact(&report(&c, &custom_context(&c, &id, false), &id, 0)),
        &HtmlSemanticValue::Known(false)
    );
    assert!(matches!(
        fact(&report(&c, &context(&c), &id, 0)),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::CustomControlClassification(_))
    ));
    for attribute in ["readonly", "disabled"] {
        let (c, id) = fixture("x-control", &[(attribute, "")], false);
        assert_eq!(
            fact(&report(&c, &context(&c), &id, 0)),
            &HtmlSemanticValue::Known(false)
        );
    }
    let (c, id) = fixture(
        "x-control",
        &[
            ("disabled", "huge unconsulted"),
            ("type", "huge unconsulted"),
        ],
        false,
    );
    assert_eq!(
        fact(&report(&c, &custom_context(&c, &id, false), &id, 0)),
        &HtmlSemanticValue::Known(false)
    );
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let e = html(&mut b, &d, "x-control", &[]);
    children(&mut b, &dl, std::slice::from_ref(&e));
    b.establish_root(&dl).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        fact(&report(&c, &context(&c), &c.subject_id(&e).unwrap(), 0)),
        &HtmlSemanticValue::Known(false)
    );
}
#[test]
fn terminal_disabled_raw_admission_is_not_bypassed_by_later_datalist() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let e = html(&mut b, &d, "input", &[("disabled", "oversized raw")]);
    children(&mut b, &dl, std::slice::from_ref(&e));
    let c = b.finish().unwrap();
    let cx = context(&c);
    let id = c.subject_id(&e).unwrap();
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let f = ss.query(&[request(&id)]).unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
#[test]
fn unrelated_state_owner_and_validation_attributes_are_not_dependencies() {
    for local in ["input", "select", "textarea"] {
        let (c, id) = fixture(
            local,
            &[
                ("novalidate", "huge unread"),
                ("formnovalidate", "huge unread"),
                ("form", "huge unread"),
                ("required", "huge unread"),
                ("value", "huge unread"),
                ("min", "huge unread"),
                ("maxlength", "huge unread"),
            ],
            true,
        );
        assert_eq!(
            fact(&report(&c, &context(&c), &id, 0)),
            &HtmlSemanticValue::Known(true)
        );
    }
}
#[test]
fn namespace_case_attributes_and_non_elements_preserve_domain() {
    let (mut b, d) = builder();
    let foreign = element(
        &mut b,
        &d,
        Namespace::svg(),
        "input",
        OrderedAttributes::new([]).unwrap(),
    );
    let e = element(
        &mut b,
        &d,
        Namespace::html(),
        "textarea",
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "readonly"),
                "huge ignored",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "readOnly"),
                "huge ignored",
            ),
        ])
        .unwrap(),
    );
    b.establish_root(&e).unwrap();
    let t = b.add_text(&d, "ignored".into()).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    assert_eq!(
        fact(&report(&c, &cx, &c.subject_id(&e).unwrap(), 0)),
        &HtmlSemanticValue::Known(true)
    );
    assert!(matches!(
        fact(&report(&c, &cx, &c.subject_id(&foreign).unwrap(), 0)),
        HtmlSemanticValue::Unsupported(HtmlSemanticUnsupported::ForeignNamespace)
    ));
    for h in [&d, &t] {
        assert_eq!(
            fact(&report(&c, &cx, &c.subject_id(h).unwrap(), 0)),
            &HtmlSemanticValue::Inapplicable
        );
    }
    let (c, id) = fixture("INPUT", &[], true);
    assert!(matches!(
        fact(&report(&c, &context(&c), &id, 0)),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::CustomControlClassification(_))
    ));
}
#[test]
fn unknown_ancestry_is_missing_and_datalist_does_not_cross_shadow_or_template_hosts() {
    let (c, id) = fixture("input", &[], false);
    assert!(matches!(
        fact(&report(&c, &context(&c), &id, 0)),
        HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(_))
    ));
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let host = html(
            &mut b,
            &d,
            if shadow { "datalist" } else { "template" },
            &[],
        );
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
        let e = html(&mut b, &d, "input", &[]);
        children(&mut b, &f, std::slice::from_ref(&e));
        let c = b.finish().unwrap();
        assert_eq!(
            fact(&report(&c, &context(&c), &c.subject_id(&e).unwrap(), 0)),
            &HtmlSemanticValue::Known(true)
        );
    }
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
fn exact_one_short_work_output_bytes_and_type_raw_caps_are_qualified_sticky() {
    let (c, id) = fixture("input", &[("type", "text")], true);
    let cx = context(&c);
    let req = request(&id);
    let base = report(&c, &cx, &id, 4).usage();
    for (resource, cap) in [
        (HtmlSemanticResource::Work, base.work()),
        (HtmlSemanticResource::OutputRecords, 1),
        (HtmlSemanticResource::AllocatedBytes, base.allocated_bytes()),
    ] {
        session(
            &c,
            &cx,
            capped(resource, cap),
            4,
            HtmlSemanticCancellation::new(),
        )
        .query(std::slice::from_ref(&req))
        .unwrap();
        let mut ss = session(
            &c,
            &cx,
            capped(resource, cap - 1),
            4,
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
    let f = session(&c, &cx, limits(), 3, HtmlSemanticCancellation::new())
        .query(&[req])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
}
#[test]
fn duplicate_cumulative_work_cancel_and_late_failure_return_no_successful_prefix() {
    let (c, id) = fixture("select", &[], true);
    let cx = context(&c);
    let req = request(&id);
    let one = report(&c, &cx, &id, 0);
    let batch = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[req.clone(), req.clone()])
        .unwrap();
    assert!(batch.usage().work() > one.usage().work());
    assert_eq!(batch.usage().output_records(), 2);
    assert_eq!(batch.usage().nested_output_records(), 0);
    assert_eq!(batch.usage().peak_scratch_records(), 0);
    let mut ss = session(
        &c,
        &cx,
        capped(HtmlSemanticResource::Work, one.usage().work()),
        0,
        HtmlSemanticCancellation::new(),
    );
    let f = ss.query(&[req.clone(), req.clone()]).unwrap_err();
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), 0, cancel);
    let f = ss.query(&[req]).unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
#[test]
fn classification_only_changes_and_exact_capture_configuration_qualify_results() {
    let (c, id) = fixture("x-control", &[], true);
    let a = custom_context(&c, &id, true);
    let b = custom_context(&c, &id, false);
    let ar = report(&c, &a, &id, 0);
    let br = report(&c, &b, &id, 0);
    assert_eq!(fact(&ar), &HtmlSemanticValue::Known(true));
    assert_eq!(fact(&br), &HtmlSemanticValue::Known(false));
    assert_ne!(ar.context(), br.context());
    assert_eq!(ar.capture(), br.capture());
    assert_eq!(
        ar.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let changed = session(
        &c,
        &a,
        capped(HtmlSemanticResource::Work, ar.usage().work()),
        1,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&id)])
    .unwrap();
    assert_ne!(ar.qualification(), changed.qualification());
    let (other, otherid) = fixture("input", &[], true);
    assert_ne!(ar.capture(), other.identity());
    assert_ne!(ar.host_revision(), other.host_facts_revision());
    let f = session(&c, &a, limits(), 0, HtmlSemanticCancellation::new())
        .query(&[request(&otherid)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(
        HtmlSemanticQuery::ConstraintValidationCandidate.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#candidate-for-constraint-validation"
    );
}
