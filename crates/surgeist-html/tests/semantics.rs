#[path = "common/context.rs"]
mod shared_context;
use shared_context::context;
use surgeist_html::*;

fn name(ns: Namespace, local: &str) -> QualifiedName {
    QualifiedName::new(ns, None, local)
}
fn attributes(values: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(
        values
            .iter()
            .map(|(n, v)| Attribute::new(name(Namespace::none(), n), *v)),
    )
    .unwrap()
}
fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 10000, 100, 0, 0),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (b, d)
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        name(Namespace::html(), local),
        attributes(attrs),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Unavailable,
    )
    .unwrap()
}

fn session<'a>(c: &'a HtmlCapture, cx: &'a HtmlSemanticContext) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(
        c,
        cx,
        HtmlValueInterpreter::new(10000),
        HtmlSemanticLimits::new(1000000, 100, 10000, 10000, 1000000, 100000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
}
fn request(
    c: &HtmlCapture,
    e: &HtmlCaptureBuilderSubject,
    query: HtmlSemanticQuery,
) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(c.subject_id(e).unwrap(), query)
}
fn disabled(value: bool) -> HtmlSemanticFact {
    HtmlSemanticFact::Disabledness(HtmlSemanticValue::Known(value))
}

#[test]
fn hidden_preserves_intrinsic_missing_invalid_empty_and_until_found_states() {
    let (mut b, d) = builder();
    let none = element(&mut b, &d, "div", &[]);
    let empty = element(&mut b, &d, "div", &[("hidden", "")]);
    let invalid = element(&mut b, &d, "div", &[("hidden", " false ")]);
    let found = element(&mut b, &d, "div", &[("hidden", "UnTiL-FoUnD")]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let requests =
        [&none, &empty, &invalid, &found].map(|e| request(&c, e, HtmlSemanticQuery::Hidden));
    let report = session(&c, &cx).query(&requests).unwrap();
    let expected = [
        HtmlHiddenState::NotHidden,
        HtmlHiddenState::Keyword(HtmlHidden::Hidden),
        HtmlHiddenState::Keyword(HtmlHidden::Hidden),
        HtmlHiddenState::Keyword(HtmlHidden::UntilFound),
    ];
    for (result, expected) in report.results().iter().zip(expected) {
        assert_eq!(
            result.fact(),
            &HtmlSemanticFact::Hidden(HtmlSemanticValue::Known(expected))
        );
    }
    assert_eq!(report.capture(), c.identity());
    assert_eq!(report.context(), cx.identity());
    assert_eq!(report.host_revision(), c.host_facts_revision());
    assert_eq!(
        report.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    assert_eq!(report.attribute_input_limit(), 10000);
}

#[test]
fn exact_names_and_namespaces_distinguish_foreign_and_inapplicable() {
    let (mut b, d) = builder();
    let div = element(&mut b, &d, "div", &[("disabled", "")]);
    let upper = element(&mut b, &d, "INPUT", &[("disabled", "")]);
    let foreign = b
        .add_element(
            &d,
            name(Namespace::svg(), "input"),
            attributes(&[("disabled", "")]),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let exact = element(&mut b, &d, "input", &[("DISABLED", "")]);
    b.establish_root(&exact).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let requests =
        [&div, &upper, &foreign, &exact].map(|e| request(&c, e, HtmlSemanticQuery::Disabledness));
    let r = session(&c, &cx).query(&requests).unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::Disabledness(HtmlSemanticValue::Inapplicable)
    );
    assert_eq!(
        r.results()[1].fact(),
        &HtmlSemanticFact::Disabledness(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::CustomControlClassification(c.subject_id(&upper).unwrap())
        ))
    );
    assert_eq!(
        r.results()[2].fact(),
        &HtmlSemanticFact::Disabledness(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
    assert_eq!(r.results()[3].fact(), &disabled(false));
}

#[test]
fn present_disabled_is_true_even_with_false_spelling_and_unknown_ancestors() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "button", &[("disabled", "false")]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    assert_eq!(
        session(&c, &cx)
            .query(&[request(&c, &e, HtmlSemanticQuery::Disabledness)])
            .unwrap()
            .results()[0]
            .fact(),
        &disabled(true)
    );
}

#[test]
fn known_false_requires_complete_ancestor_evidence() {
    let (mut b, d) = builder();
    let unknown = element(&mut b, &d, "input", &[]);
    let root = element(&mut b, &d, "input", &[]);
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = session(&c, &cx)
        .query(&[
            request(&c, &unknown, HtmlSemanticQuery::Disabledness),
            request(&c, &root, HtmlSemanticQuery::Disabledness),
        ])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::Disabledness(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(
            c.subject_id(&unknown).unwrap()
        )))
    );
    assert_eq!(r.results()[1].fact(), &disabled(false));
}

#[test]
fn first_legend_exception_uses_first_legend_element_not_first_child() {
    let (mut b, d) = builder();
    let f = element(&mut b, &d, "fieldset", &[("disabled", "")]);
    b.establish_root(&f).unwrap();
    let before = element(&mut b, &d, "div", &[]);
    let first = element(&mut b, &d, "legend", &[]);
    let second = element(&mut b, &d, "legend", &[]);
    let a = element(&mut b, &d, "input", &[]);
    let z = element(&mut b, &d, "input", &[]);
    let outside = element(&mut b, &d, "input", &[]);
    b.establish_children(
        &f,
        &[before, first.clone(), second.clone(), outside.clone()],
    )
    .unwrap();
    b.establish_children(&first, std::slice::from_ref(&a))
        .unwrap();
    b.establish_children(&second, std::slice::from_ref(&z))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = session(&c, &cx)
        .query(&[&a, &z, &outside].map(|e| request(&c, e, HtmlSemanticQuery::Disabledness)))
        .unwrap();
    assert_eq!(r.results()[0].fact(), &disabled(false));
    assert_eq!(r.results()[1].fact(), &disabled(true));
    assert_eq!(r.results()[2].fact(), &disabled(true));
}

#[test]
fn outer_fieldset_still_disables_control_exempt_from_inner_fieldset() {
    let (mut b, d) = builder();
    let outer = element(&mut b, &d, "fieldset", &[("disabled", "")]);
    let inner = element(&mut b, &d, "fieldset", &[("disabled", "")]);
    let legend = element(&mut b, &d, "legend", &[]);
    let e = element(&mut b, &d, "input", &[]);
    b.establish_root(&outer).unwrap();
    b.establish_children(&outer, std::slice::from_ref(&inner))
        .unwrap();
    b.establish_children(&inner, std::slice::from_ref(&legend))
        .unwrap();
    b.establish_children(&legend, std::slice::from_ref(&e))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    assert_eq!(
        session(&c, &cx)
            .query(&[request(&c, &e, HtmlSemanticQuery::Disabledness)])
            .unwrap()
            .results()[0]
            .fact(),
        &disabled(true)
    );
}

#[test]
fn unknown_fieldset_child_order_cannot_prove_legend_exclusion() {
    let (mut b, d) = builder();
    let f = element(&mut b, &d, "fieldset", &[("disabled", "")]);
    let e = element(&mut b, &d, "input", &[]);
    let legend = element(&mut b, &d, "legend", &[]);
    b.establish_root(&f).unwrap();
    b.establish_parent(&legend, &f).unwrap();
    b.establish_children(&legend, std::slice::from_ref(&e))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    assert_eq!(
        session(&c, &cx)
            .query(&[request(&c, &e, HtmlSemanticQuery::Disabledness)])
            .unwrap()
            .results()[0]
            .fact(),
        &HtmlSemanticFact::Disabledness(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(c.subject_id(&f).unwrap())
        ))
    );
}

#[test]
fn non_legend_branch_proves_disabled_without_unrelated_child_order() {
    let (mut b, d) = builder();
    let f = element(&mut b, &d, "fieldset", &[("disabled", "")]);
    let e = element(&mut b, &d, "input", &[]);
    b.establish_parent(&e, &f).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    assert_eq!(
        session(&c, &cx)
            .query(&[request(&c, &e, HtmlSemanticQuery::Disabledness)])
            .unwrap()
            .results()[0]
            .fact(),
        &disabled(true)
    );
}

#[test]
fn option_search_crosses_wrappers_but_stops_at_source_specified_barriers() {
    for barrier in ["div", "hr", "datalist", "option"] {
        let (mut b, d) = builder();
        let group = element(&mut b, &d, "optgroup", &[("disabled", "")]);
        let middle = element(&mut b, &d, barrier, &[]);
        let e = element(&mut b, &d, "option", &[]);
        b.establish_root(&group).unwrap();
        b.establish_children(&group, std::slice::from_ref(&middle))
            .unwrap();
        b.establish_children(&middle, std::slice::from_ref(&e))
            .unwrap();
        let c = b.finish().unwrap();
        let cx = context(&c);
        assert_eq!(
            session(&c, &cx)
                .query(&[request(&c, &e, HtmlSemanticQuery::Disabledness)])
                .unwrap()
                .results()[0]
                .fact(),
            &disabled(barrier == "div"),
            "{barrier}"
        );
    }
}

#[test]
fn disabled_select_contributes_actual_disabledness_but_nested_optgroups_stop_lookup() {
    for double in [false, true] {
        let (mut b, d) = builder();
        let select = element(&mut b, &d, "select", &[("disabled", "")]);
        let group = element(&mut b, &d, "optgroup", &[]);
        let inner = element(&mut b, &d, "optgroup", &[]);
        let e = element(&mut b, &d, "option", &[]);
        b.establish_root(&select).unwrap();
        b.establish_children(&select, std::slice::from_ref(&group))
            .unwrap();
        if double {
            b.establish_children(&group, std::slice::from_ref(&inner))
                .unwrap();
            b.establish_children(&inner, std::slice::from_ref(&e))
                .unwrap();
        } else {
            b.establish_children(&group, std::slice::from_ref(&e))
                .unwrap();
        }
        let c = b.finish().unwrap();
        let cx = context(&c);
        assert_eq!(
            session(&c, &cx)
                .query(&[request(&c, &e, HtmlSemanticQuery::Disabledness)])
                .unwrap()
                .results()[0]
                .fact(),
            &disabled(!double)
        );
        assert_eq!(
            session(&c, &cx)
                .query(&[request(&c, &group, HtmlSemanticQuery::Disabledness)])
                .unwrap()
                .results()[0]
                .fact(),
            &disabled(true)
        );
    }
}

#[test]
fn custom_control_participation_requires_explicit_matching_host_revision() {
    let (mut b, d) = builder();
    let yes = element(&mut b, &d, "x-control", &[("disabled", "")]);
    let no = element(&mut b, &d, "x-other", &[("disabled", "")]);
    let unknown = element(&mut b, &d, "x-unknown", &[("disabled", "")]);
    let c = b.finish().unwrap();
    let facts = vec![
        HtmlCustomControlClassification::new(c.subject_id(&yes).unwrap(), true),
        HtmlCustomControlClassification::new(c.subject_id(&no).unwrap(), false),
    ];
    assert!(matches!(
        HtmlSemanticContext::new(
            &c,
            HtmlHostFactsRevision::fresh(),
            HtmlSemanticContextInput::new(facts.clone()),
            2
        ),
        Err(HtmlSemanticContextError::HostRevisionMismatch)
    ));
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(facts),
        2,
    )
    .unwrap();
    let r = session(&c, &cx)
        .query(&[&yes, &no, &unknown].map(|e| request(&c, e, HtmlSemanticQuery::Disabledness)))
        .unwrap();
    assert_eq!(r.results()[0].fact(), &disabled(true));
    assert_eq!(
        r.results()[1].fact(),
        &HtmlSemanticFact::Disabledness(HtmlSemanticValue::Inapplicable)
    );
    assert_eq!(
        r.results()[2].fact(),
        &HtmlSemanticFact::Disabledness(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::CustomControlClassification(c.subject_id(&unknown).unwrap())
        ))
    );
}

#[test]
fn complete_batch_limits_and_first_terminal_failure_are_sticky() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "input", &[]);
    b.establish_root(&e).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let request = request(&c, &e, HtmlSemanticQuery::Disabledness);
    let measured = session(&c, &cx)
        .query(std::slice::from_ref(&request))
        .unwrap();
    let exact = measured.usage().work();
    let cancel = HtmlSemanticCancellation::new();
    let mut short = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(10000),
        HtmlSemanticLimits::new(exact - 1, 2, 10000, 10000, 1000000, 100000),
        cancel.clone(),
    )
    .unwrap();
    let failed = short.query(std::slice::from_ref(&request)).unwrap_err();
    assert!(matches!(
        failed.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    cancel.cancel();
    assert_eq!(short.query(&[]).unwrap_err(), failed);
    let mut enough = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(10000),
        HtmlSemanticLimits::new(exact, 1, 10000, 10000, 1000000, 100000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    assert_eq!(
        enough
            .query(std::slice::from_ref(&request))
            .unwrap()
            .results()[0]
            .fact(),
        &disabled(false)
    );
    assert!(matches!(
        enough.query(&[request]).unwrap_err().kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::OutputRecords,
            ..
        }
    ));
}

#[test]
fn cancellation_empty_batch_and_foreign_capture_fail_without_results() {
    let (b, _) = builder();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(0),
        HtmlSemanticLimits::new(0, 0, 10000, 10000, 1000000, 100000),
        cancel,
    )
    .unwrap();
    assert_eq!(
        s.query(&[]).unwrap_err().kind(),
        &HtmlSemanticFailureKind::Cancelled
    );
    let (mut b2, d2) = builder();
    let e = element(&mut b2, &d2, "div", &[]);
    let c2 = b2.finish().unwrap();
    assert!(matches!(
        HtmlSemanticSession::new(
            &c2,
            &cx,
            HtmlValueInterpreter::new(0),
            HtmlSemanticLimits::new(10, 1, 10000, 10000, 1000000, 100000),
            HtmlSemanticCancellation::new()
        ),
        Err(HtmlSemanticFailure { .. })
    ));
    assert_eq!(
        session(&c, &cx)
            .query(&[request(&c2, &e, HtmlSemanticQuery::Hidden)])
            .unwrap_err()
            .kind(),
        &HtmlSemanticFailureKind::Capture(HtmlCaptureError::ForeignCapture)
    );
    assert!(session(&c, &cx).query(&[]).unwrap().results().is_empty());
}

#[test]
fn raw_surrogate_values_stay_preserved_while_intrinsic_defaults_apply() {
    let (mut b, d) = builder();
    let raw = DocumentString::from_utf16(vec![0xd800]);
    let e = b
        .add_element(
            &d,
            name(Namespace::html(), "div"),
            OrderedAttributes::new([Attribute::new(
                name(Namespace::none(), "hidden"),
                raw.clone(),
            )])
            .unwrap(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    assert_eq!(
        session(&c, &cx)
            .query(&[request(&c, &e, HtmlSemanticQuery::Hidden)])
            .unwrap()
            .results()[0]
            .fact(),
        &HtmlSemanticFact::Hidden(HtmlSemanticValue::Known(HtmlHiddenState::Keyword(
            HtmlHidden::Hidden
        )))
    );
    let id = c.subject_id(&e).unwrap();
    let HtmlCapturedNodeKind::Element(element) = c.node(&id).unwrap().kind() else {
        panic!("element");
    };
    assert_eq!(element.attributes().as_slice()[0].value(), &raw);
}

#[test]
fn completed_results_survive_later_cancellation_and_qualify_context_changes() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "x-control", &[("disabled", "")]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let yes = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![HtmlCustomControlClassification::new(id.clone(), true)]),
        1,
    )
    .unwrap();
    let no = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![HtmlCustomControlClassification::new(id, false)]),
        1,
    )
    .unwrap();
    let cancel = HtmlSemanticCancellation::new();
    let mut s = HtmlSemanticSession::new(
        &c,
        &yes,
        HtmlValueInterpreter::new(10),
        HtmlSemanticLimits::new(100000, 1, 10000, 10000, 1000000, 100000),
        cancel.clone(),
    )
    .unwrap();
    let r = s
        .query(&[request(&c, &e, HtmlSemanticQuery::Disabledness)])
        .unwrap();
    cancel.cancel();
    assert_eq!(r.results()[0].fact(), &disabled(true));
    assert_eq!(
        s.query(&[]).unwrap_err().kind(),
        &HtmlSemanticFailureKind::Cancelled
    );
    let changed = session(&c, &no)
        .query(&[request(&c, &e, HtmlSemanticQuery::Disabledness)])
        .unwrap();
    assert_ne!(r.context(), changed.context());
    assert_eq!(r.capture(), changed.capture());
    assert_eq!(
        changed.results()[0].fact(),
        &HtmlSemanticFact::Disabledness(HtmlSemanticValue::Inapplicable)
    );
}

#[test]
fn late_batch_failure_publishes_no_prefix_and_intrinsic_admission_is_retained() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "div", &[("hidden", "until-found")]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let (mut other, od) = builder();
    let oe = element(&mut other, &od, "div", &[]);
    let oc = other.finish().unwrap();
    let mut s = session(&c, &cx);
    let error = s
        .query(&[
            request(&c, &e, HtmlSemanticQuery::Hidden),
            request(&oc, &oe, HtmlSemanticQuery::Hidden),
        ])
        .unwrap_err();
    assert_eq!(
        error.kind(),
        &HtmlSemanticFailureKind::Capture(HtmlCaptureError::ForeignCapture)
    );
    assert_eq!(s.failure(), Some(&error));
    assert_eq!(error.usage().output_records(), 2);
    let mut small = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(1),
        HtmlSemanticLimits::new(100000, 1, 10000, 10000, 1000000, 100000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    assert!(matches!(
        small
            .query(&[request(&c, &e, HtmlSemanticQuery::Hidden)])
            .unwrap_err()
            .kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
}

#[test]
fn producer_context_rejects_duplicate_foreign_and_over_capacity_records() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "x-control", &[]);
    let c = b.finish().unwrap();
    let fact = HtmlCustomControlClassification::new(c.subject_id(&e).unwrap(), true);
    assert!(matches!(
        HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![fact.clone()]),
            0
        ),
        Err(HtmlSemanticContextError::ClassificationLimit { .. })
    ));
    assert!(matches!(
        HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![fact.clone(), fact.clone()]),
            2
        ),
        Err(HtmlSemanticContextError::DuplicateSubject(_))
    ));
    let (other, _) = builder();
    let oc = other.finish().unwrap();
    assert_eq!(
        HtmlSemanticContext::new(
            &oc,
            oc.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![fact]),
            1
        )
        .unwrap_err(),
        HtmlSemanticContextError::Capture(HtmlCaptureError::ForeignCapture)
    );
}

fn cancelled_failure(
    capture: &HtmlCapture,
    context: &HtmlSemanticContext,
    limits: HtmlSemanticLimits,
    attribute_limit: usize,
) -> HtmlSemanticFailure {
    let cancellation = HtmlSemanticCancellation::new();
    cancellation.cancel();
    HtmlSemanticSession::new(
        capture,
        context,
        HtmlValueInterpreter::new(attribute_limit),
        limits,
        cancellation,
    )
    .unwrap()
    .query(&[])
    .unwrap_err()
}

#[test]
fn retained_cancellation_identifies_exact_capture_context_host_and_configuration() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "x-control", &[]);
    let c = b.finish().unwrap();
    let subject = c.subject_id(&e).unwrap();
    let yes = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![HtmlCustomControlClassification::new(
            subject.clone(),
            true,
        )]),
        1,
    )
    .unwrap();
    let no = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![HtmlCustomControlClassification::new(subject, false)]),
        1,
    )
    .unwrap();
    let limits = HtmlSemanticLimits::new(1000, 2, 10000, 10000, 1000000, 100000);
    let first = cancelled_failure(&c, &yes, limits, 10);
    let changed_context = cancelled_failure(&c, &no, limits, 10);
    let changed_work = cancelled_failure(
        &c,
        &yes,
        HtmlSemanticLimits::new(1001, 2, 10000, 10000, 1000000, 100000),
        10,
    );
    let changed_output = cancelled_failure(
        &c,
        &yes,
        HtmlSemanticLimits::new(1000, 3, 10000, 10000, 1000000, 100000),
        10,
    );
    let changed_attribute_limit = cancelled_failure(&c, &yes, limits, 11);
    let key = first.qualification().unwrap();
    assert_eq!(key.capture(), c.identity());
    assert_eq!(key.context(), yes.identity());
    assert_eq!(key.host_revision(), c.host_facts_revision());
    assert_eq!(key.limits(), limits);
    assert_eq!(key.attribute_input_limit(), 10);
    assert_eq!(
        key.source_revision(),
        "877d6146487f15b15bc8dd0e3c625063a37b456a"
    );
    for other in [
        &changed_context,
        &changed_work,
        &changed_output,
        &changed_attribute_limit,
    ] {
        assert_eq!(other.kind(), first.kind());
        assert_eq!(other.usage(), first.usage());
        assert_ne!(other.qualification(), first.qualification());
        assert_eq!(other.qualification().unwrap().capture(), key.capture());
    }
    assert_eq!(
        changed_context.qualification().unwrap().context(),
        no.identity()
    );
    assert_eq!(changed_work.qualification().unwrap().limits().work(), 1001);
    assert_eq!(
        changed_output
            .qualification()
            .unwrap()
            .limits()
            .output_records(),
        3
    );
    assert_eq!(
        changed_attribute_limit
            .qualification()
            .unwrap()
            .attribute_input_limit(),
        11
    );

    // A distinct capture at the same producer host revision remains distinct.
    let mut same_host = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(1, 0, 0, 0, 0, 0, 0),
        c.host_facts_revision().clone(),
    );
    same_host
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let same_host = same_host.finish().unwrap();
    let same_context = context(&same_host);
    let changed_capture = cancelled_failure(&same_host, &same_context, limits, 10);
    assert_ne!(
        changed_capture.qualification().unwrap().capture(),
        key.capture()
    );
    assert_eq!(
        changed_capture.qualification().unwrap().host_revision(),
        key.host_revision()
    );
    let (other, _) = builder();
    let other = other.finish().unwrap();
    let other_context = context(&other);
    let changed_host = cancelled_failure(&other, &other_context, limits, 10);
    assert_ne!(
        changed_host.qualification().unwrap().host_revision(),
        key.host_revision()
    );
    assert_eq!(
        changed_host.qualification().unwrap().capture(),
        other.identity()
    );
}

#[test]
fn successful_batch_and_sticky_limit_failure_share_original_qualification() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "div", &[]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let limits = HtmlSemanticLimits::new(100000, 1, 10000, 10000, 1000000, 100000);
    let cancellation = HtmlSemanticCancellation::new();
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(3),
        limits,
        cancellation.clone(),
    )
    .unwrap();
    let query = request(&c, &e, HtmlSemanticQuery::Hidden);
    let completed = s.query(std::slice::from_ref(&query)).unwrap();
    let failure = s.query(std::slice::from_ref(&query)).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::OutputRecords,
            ..
        }
    ));
    assert_eq!(failure.qualification(), Some(completed.qualification()));
    cancellation.cancel();
    let (mut b2, d2) = builder();
    let e2 = element(&mut b2, &d2, "input", &[]);
    let c2 = b2.finish().unwrap();
    let repeated = s
        .query(&[request(&c2, &e2, HtmlSemanticQuery::Disabledness)])
        .unwrap_err();
    assert_eq!(repeated, failure);
    assert_eq!(s.failure(), Some(&failure));
    assert_eq!(repeated.qualification().unwrap().capture(), c.identity());
    assert_eq!(repeated.qualification().unwrap().context(), cx.identity());
    assert_eq!(repeated.qualification().unwrap().limits(), limits);
    assert_eq!(repeated.qualification().unwrap().attribute_input_limit(), 3);
    let capture_id = c.identity().clone();
    let context_id = cx.identity().clone();
    let host = c.host_facts_revision().clone();
    drop(s);
    drop(completed);
    drop(cx);
    drop(c);
    assert_eq!(failure.qualification().unwrap().capture(), &capture_id);
    assert_eq!(failure.qualification().unwrap().context(), &context_id);
    assert_eq!(failure.qualification().unwrap().host_revision(), &host);
}

#[test]
fn work_and_attribute_failures_are_qualified_but_constructor_rejection_is_not() {
    let (mut b, d) = builder();
    let e = element(&mut b, &d, "div", &[("hidden", "hidden")]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    for (limits, attribute_limit, expected) in [
        (
            HtmlSemanticLimits::new(0, 1, 10000, 10000, 1000000, 100000),
            10,
            HtmlSemanticResource::Work,
        ),
        (
            HtmlSemanticLimits::new(100000, 0, 10000, 10000, 1000000, 100000),
            10,
            HtmlSemanticResource::OutputRecords,
        ),
    ] {
        let mut s = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(attribute_limit),
            limits,
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let failure = s
            .query(&[request(&c, &e, HtmlSemanticQuery::Hidden)])
            .unwrap_err();
        assert!(
            matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded { resource,.. } if *resource==expected)
        );
        let key = failure.qualification().unwrap();
        assert_eq!(key.capture(), c.identity());
        assert_eq!(key.context(), cx.identity());
        assert_eq!(key.limits(), limits);
        assert_eq!(key.attribute_input_limit(), attribute_limit);
    }
    let limits = HtmlSemanticLimits::new(100000, 1, 10000, 10000, 1000000, 100000);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(0),
        limits,
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let failure = s
        .query(&[request(&c, &e, HtmlSemanticQuery::Hidden)])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(failure.qualification().unwrap().attribute_input_limit(), 0);
    assert_eq!(failure.qualification().unwrap().limits(), limits);
    let (other, _) = builder();
    let other = other.finish().unwrap();
    let rejected = match HtmlSemanticSession::new(
        &other,
        &cx,
        HtmlValueInterpreter::new(0),
        limits,
        HtmlSemanticCancellation::new(),
    ) {
        Err(error) => error,
        Ok(_) => panic!("foreign context admitted"),
    };
    assert_eq!(
        rejected.kind(),
        &HtmlSemanticFailureKind::ContextCaptureMismatch
    );
    assert!(rejected.qualification().is_none());
}

fn participation_revision() -> HtmlListParticipationRevision {
    HtmlListParticipationRevision::new(
        HtmlParticipationProducerId::fresh(),
        3,
        17,
        HtmlParticipationProducerId::fresh(),
        5,
        29,
    )
}
fn list_context(
    c: &HtmlCapture,
    revision: HtmlListParticipationRevision,
    facts: Vec<HtmlListParticipationFact>,
) -> HtmlSemanticContext {
    let cap = facts.capacity();
    let participation = HtmlListParticipation::new(c, revision.clone(), facts, cap).unwrap();
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![])
            .with_list_participation(revision, Some(participation)),
        0,
    )
    .unwrap()
}
fn list_fact(
    c: &HtmlCapture,
    e: &HtmlCaptureBuilderSubject,
    item: bool,
    rendered: bool,
    css: bool,
) -> HtmlListParticipationFact {
    HtmlListParticipationFact::new(
        c.subject_id(e).unwrap(),
        Some(item),
        Some(rendered),
        Some(css),
    )
}
fn list_fixture(
    local: &str,
    attrs: &[(&str, &str)],
    specs: &[(&str, Option<&str>)],
) -> (
    HtmlCapture,
    HtmlCaptureBuilderSubject,
    Vec<HtmlCaptureBuilderSubject>,
) {
    let (mut b, d) = builder();
    let owner = element(&mut b, &d, local, attrs);
    b.establish_root(&owner).unwrap();
    let mut items = Vec::new();
    for (local, value) in specs {
        let attrs = value
            .map(|value| vec![("value", value)])
            .unwrap_or_default();
        let item = element(&mut b, &d, local, &attrs);
        b.establish_children(&item, &[]).unwrap();
        items.push(item);
    }
    b.establish_children(&owner, &items).unwrap();
    (b.finish().unwrap(), owner, items)
}
fn complete_list_context(
    c: &HtmlCapture,
    owner: &HtmlCaptureBuilderSubject,
    items: &[HtmlCaptureBuilderSubject],
) -> HtmlSemanticContext {
    let facts = std::iter::once(list_fact(c, owner, false, true, true))
        .chain(items.iter().map(|e| list_fact(c, e, true, true, true)))
        .collect();
    list_context(c, participation_revision(), facts)
}
fn assert_ordinals(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    items: &[HtmlCaptureBuilderSubject],
    expected: &[&str],
) {
    let requests: Vec<_> = items
        .iter()
        .map(|e| request(c, e, HtmlSemanticQuery::ListOrdinal))
        .collect();
    let report = session(c, cx).query(&requests).unwrap();
    assert_eq!(report.results().len(), expected.len());
    for (result, expected) in report.results().iter().zip(expected) {
        let HtmlSemanticFact::ListOrdinal(HtmlSemanticValue::Known(Some(value))) = result.fact()
        else {
            panic!("unexpected ordinal: {:?}", result.fact());
        };
        assert_eq!(value.as_document_string(), DocumentString::from(*expected));
    }
}

#[test]
fn source_list_ordinals_include_value_reset_for_general_owners() {
    let (c, owner, items) =
        list_fixture("ol", &[], &[("li", None), ("li", Some("3")), ("li", None)]);
    assert_ordinals(
        &c,
        &complete_list_context(&c, &owner, &items),
        &items,
        &["1", "3", "4"],
    );
    // The selected ordinal algorithm's value-reset step applies to HTML li
    // even when the source owner is not ol.
    let (c, owner, items) = list_fixture(
        "div",
        &[],
        &[("li", None), ("li", Some("-2")), ("div", None)],
    );
    assert_ordinals(
        &c,
        &complete_list_context(&c, &owner, &items),
        &items,
        &["1", "-2", "-1"],
    );
}

#[test]
fn list_integer_recovery_uses_exact_prefixes_signs_and_arbitrary_magnitudes() {
    let (c, owner, items) = list_fixture(
        "ol",
        &[("start", " \t+9999999999999999999999999999999999999999tail")],
        &[
            ("li", None),
            ("li", None),
            ("li", Some(" \n-2suffix")),
            ("li", Some("nope")),
            ("LI", Some("80")),
        ],
    );
    assert_ordinals(
        &c,
        &complete_list_context(&c, &owner, &items),
        &items,
        &[
            "9999999999999999999999999999999999999999",
            "10000000000000000000000000000000000000000",
            "-2",
            "-1",
            "0",
        ],
    );
    let (c, owner, items) = list_fixture(
        "ol",
        &[
            ("start", "-9999999999999999999999999999999999999999"),
            ("reversed", "false"),
        ],
        &[("li", None), ("li", None)],
    );
    assert_ordinals(
        &c,
        &complete_list_context(&c, &owner, &items),
        &items,
        &[
            "-9999999999999999999999999999999999999999",
            "-10000000000000000000000000000000000000000",
        ],
    );
}

#[test]
fn reversed_start_counts_owned_html_li_but_numbers_all_owned_items() {
    let (c, owner, items) = list_fixture(
        "ol",
        &[("reversed", "")],
        &[("li", None), ("div", None), ("li", None)],
    );
    let cx = complete_list_context(&c, &owner, &items);
    assert_ordinals(&c, &cx, &items, &["2", "1", "0"]);
    let report = session(&c, &cx)
        .query(&[request(&c, &owner, HtmlSemanticQuery::ListStartingValue)])
        .unwrap();
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::ListStartingValue(HtmlSemanticValue::Known(HtmlInteger::from(2i128)))
    );
    let (c, owner, items) = list_fixture("ol", &[("reversed", "")], &[("div", None)]);
    assert_ordinals(
        &c,
        &complete_list_context(&c, &owner, &items),
        &items,
        &["0"],
    );
}

#[test]
fn known_non_items_and_unrendered_items_are_excluded_without_inferred_hidden_state() {
    let (c, owner, items) = list_fixture(
        "ol",
        &[("reversed", "")],
        &[("li", None), ("li", None), ("div", None), ("li", None)],
    );
    let facts = vec![
        list_fact(&c, &owner, false, true, true),
        list_fact(&c, &items[0], true, true, true),
        HtmlListParticipationFact::new(c.subject_id(&items[1]).unwrap(), Some(false), None, None),
        list_fact(&c, &items[2], true, false, false),
        list_fact(&c, &items[3], true, true, true),
    ];
    let cx = list_context(&c, participation_revision(), facts);
    let requests: Vec<_> = items
        .iter()
        .map(|e| request(&c, e, HtmlSemanticQuery::ListOrdinal))
        .collect();
    let report = session(&c, &cx).query(&requests).unwrap();
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::ListOrdinal(HtmlSemanticValue::Known(Some(HtmlInteger::from(2i128))))
    );
    assert_eq!(
        report.results()[1].fact(),
        &HtmlSemanticFact::ListOrdinal(HtmlSemanticValue::Inapplicable)
    );
    assert_eq!(
        report.results()[2].fact(),
        &HtmlSemanticFact::ListOrdinal(HtmlSemanticValue::Known(None))
    );
    assert_eq!(
        report.results()[3].fact(),
        &HtmlSemanticFact::ListOrdinal(HtmlSemanticValue::Known(Some(HtmlInteger::from(1i128))))
    );
}

#[test]
fn explicit_or_ascending_start_uses_only_source_required_facts() {
    let (mut b, d) = builder();
    let explicit = element(&mut b, &d, "ol", &[("start", " +7tail"), ("reversed", "")]);
    let ascending = element(&mut b, &d, "ol", &[("start", "bad")]);
    let reversed = element(&mut b, &d, "ol", &[("start", ""), ("reversed", "")]);
    let other = element(&mut b, &d, "ul", &[("start", "5")]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let report = session(&c, &cx)
        .query(
            &[&explicit, &ascending, &reversed, &other]
                .map(|e| request(&c, e, HtmlSemanticQuery::ListStartingValue)),
        )
        .unwrap();
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::ListStartingValue(HtmlSemanticValue::Known(HtmlInteger::from(7i128)))
    );
    assert_eq!(
        report.results()[1].fact(),
        &HtmlSemanticFact::ListStartingValue(HtmlSemanticValue::Known(HtmlInteger::from(1i128)))
    );
    assert_eq!(
        report.results()[2].fact(),
        &HtmlSemanticFact::ListStartingValue(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(c.subject_id(&reversed).unwrap())
        ))
    );
    assert_eq!(
        report.results()[3].fact(),
        &HtmlSemanticFact::ListStartingValue(HtmlSemanticValue::Inapplicable)
    );
}

#[test]
fn nearest_list_ancestor_then_css_box_controls_boxless_and_nested_ownership() {
    for inner_box in [false, true] {
        let (mut b, d) = builder();
        let outer = element(&mut b, &d, "ol", &[("reversed", "")]);
        let first = element(&mut b, &d, "li", &[]);
        let inner = element(&mut b, &d, "ul", &[]);
        let wrapper = element(&mut b, &d, "div", &[]);
        let second = element(&mut b, &d, "li", &[]);
        let non_li = element(&mut b, &d, "div", &[]);
        b.establish_root(&outer).unwrap();
        b.establish_children(&outer, &[first.clone(), inner.clone(), non_li.clone()])
            .unwrap();
        b.establish_children(&inner, std::slice::from_ref(&wrapper))
            .unwrap();
        b.establish_children(&wrapper, std::slice::from_ref(&second))
            .unwrap();
        for leaf in [&first, &second, &non_li] {
            b.establish_children(leaf, &[]).unwrap();
        }
        let c = b.finish().unwrap();
        let cx = list_context(
            &c,
            participation_revision(),
            vec![
                list_fact(&c, &outer, false, true, true),
                list_fact(&c, &first, true, true, true),
                list_fact(&c, &inner, false, inner_box, inner_box),
                list_fact(&c, &wrapper, false, true, true),
                list_fact(&c, &second, true, true, true),
                list_fact(&c, &non_li, true, true, true),
            ],
        );
        let owner = session(&c, &cx)
            .query(&[request(&c, &second, HtmlSemanticQuery::ListOwner)])
            .unwrap();
        assert_eq!(
            owner.results()[0].fact(),
            &HtmlSemanticFact::ListOwner(HtmlSemanticValue::Known(Some(
                c.subject_id(if inner_box { &inner } else { &outer })
                    .unwrap()
            )))
        );
        let all = vec![first, second, non_li];
        assert_ordinals(
            &c,
            &cx,
            &all,
            if inner_box {
                &["1", "1", "0"]
            } else {
                &["2", "1", "0"]
            },
        );
    }
}

#[test]
fn foreign_computed_items_and_svg_rendering_use_supplied_css_box_ancestors() {
    let (mut b, d) = builder();
    let owner = element(&mut b, &d, "menu", &[]);
    let wrapper = b
        .add_element(
            &d,
            name(Namespace::svg(), "ol"),
            attributes(&[]),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Unavailable,
        )
        .unwrap();
    let item = b
        .add_element(
            &d,
            name(Namespace::svg(), "li"),
            attributes(&[("value", "99"), ("hidden", "")]),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Unavailable,
        )
        .unwrap();
    b.establish_root(&owner).unwrap();
    b.establish_children(&owner, std::slice::from_ref(&wrapper))
        .unwrap();
    b.establish_children(&wrapper, std::slice::from_ref(&item))
        .unwrap();
    b.establish_children(&item, &[]).unwrap();
    let c = b.finish().unwrap();
    let cx = list_context(
        &c,
        participation_revision(),
        vec![
            list_fact(&c, &owner, false, true, true),
            list_fact(&c, &wrapper, false, true, false),
            list_fact(&c, &item, true, true, false),
        ],
    );
    assert_ordinals(&c, &cx, std::slice::from_ref(&item), &["1"]);
    let r = session(&c, &cx)
        .query(&[request(&c, &item, HtmlSemanticQuery::ListOwner)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::ListOwner(HtmlSemanticValue::Known(Some(
            c.subject_id(&owner).unwrap()
        )))
    );
}

#[test]
fn missing_participation_and_null_owner_are_distinct_and_late_coverage_is_not_partial() {
    let (c, owner, items) = list_fixture("div", &[], &[("li", None), ("li", None)]);
    let first = c.subject_id(&items[0]).unwrap();
    let second = c.subject_id(&items[1]).unwrap();
    let no_input = context(&c);
    let r = session(&c, &no_input)
        .query(&[request(&c, &items[0], HtmlSemanticQuery::ListOwner)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::ListOwner(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::ComputedListItem(first.clone())
        ))
    );
    let cx = list_context(
        &c,
        participation_revision(),
        vec![
            list_fact(&c, &owner, false, true, true),
            list_fact(&c, &items[0], true, true, true),
            HtmlListParticipationFact::new(second.clone(), Some(true), None, None),
        ],
    );
    let r = session(&c, &cx)
        .query(&[request(&c, &items[0], HtmlSemanticQuery::ListOrdinal)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::ListOrdinal(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::BeingRendered(second)
        ))
    );
    let cx = list_context(
        &c,
        participation_revision(),
        vec![
            list_fact(&c, &owner, false, true, false),
            list_fact(&c, &items[0], true, true, false),
        ],
    );
    let r = session(&c, &cx)
        .query(&[request(&c, &items[0], HtmlSemanticQuery::ListOwner)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::ListOwner(HtmlSemanticValue::Known(None))
    );
}

#[test]
fn participation_input_rejects_foreign_duplicate_stale_and_contradictory_records() {
    let (c, owner, items) = list_fixture("ol", &[], &[("li", None)]);
    let revision = participation_revision();
    let fact = list_fact(&c, &items[0], true, true, true);
    assert!(matches!(
        HtmlListParticipation::new(&c, revision.clone(), vec![fact.clone()], 0),
        Err(HtmlSemanticContextError::ParticipationLimit { .. })
    ));
    assert_eq!(
        HtmlListParticipation::new(&c, revision.clone(), vec![fact.clone(), fact.clone()], 2)
            .unwrap_err(),
        HtmlSemanticContextError::DuplicateSubject(c.subject_id(&items[0]).unwrap())
    );
    assert!(matches!(
        HtmlListParticipation::new(
            &c,
            revision.clone(),
            vec![list_fact(&c, &owner, false, false, true)],
            1
        ),
        Err(HtmlSemanticContextError::ContradictoryParticipation(_))
    ));
    let (other, _, _) = list_fixture("ol", &[], &[]);
    assert!(matches!(
        HtmlListParticipation::new(&other, revision.clone(), vec![fact], 1),
        Err(HtmlSemanticContextError::Capture(
            HtmlCaptureError::ForeignCapture
        ))
    ));
    let input = HtmlListParticipation::new(&c, revision.clone(), vec![], 0).unwrap();
    assert_eq!(
        HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![])
                .with_list_participation(participation_revision(), Some(input)),
            0
        )
        .unwrap_err(),
        HtmlSemanticContextError::ParticipationRevisionMismatch
    );
    let input = HtmlListParticipation::new(&other, revision.clone(), vec![], 0).unwrap();
    assert_eq!(
        HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_list_participation(revision, Some(input)),
            0
        )
        .unwrap_err(),
        HtmlSemanticContextError::ParticipationCaptureMismatch
    );
}

#[test]
fn participation_keys_qualify_results_and_sticky_failures_across_producer_changes() {
    let (c, owner, items) = list_fixture("ol", &[], &[("li", None)]);
    let base = participation_revision();
    let changed = [
        HtmlListParticipationRevision::new(
            HtmlParticipationProducerId::fresh(),
            3,
            17,
            base.box_producer().clone(),
            5,
            29,
        ),
        HtmlListParticipationRevision::new(
            base.style_producer().clone(),
            4,
            17,
            base.box_producer().clone(),
            5,
            29,
        ),
        HtmlListParticipationRevision::new(
            base.style_producer().clone(),
            3,
            18,
            base.box_producer().clone(),
            5,
            29,
        ),
        HtmlListParticipationRevision::new(
            base.style_producer().clone(),
            3,
            17,
            HtmlParticipationProducerId::fresh(),
            5,
            29,
        ),
        HtmlListParticipationRevision::new(
            base.style_producer().clone(),
            3,
            17,
            base.box_producer().clone(),
            6,
            29,
        ),
        HtmlListParticipationRevision::new(
            base.style_producer().clone(),
            3,
            17,
            base.box_producer().clone(),
            5,
            30,
        ),
    ];
    let cx = list_context(
        &c,
        base.clone(),
        vec![
            list_fact(&c, &owner, false, true, true),
            list_fact(&c, &items[0], true, true, true),
        ],
    );
    let requested = request(&c, &items[0], HtmlSemanticQuery::ListOrdinal);
    let report = session(&c, &cx)
        .query(std::slice::from_ref(&requested))
        .unwrap();
    assert_eq!(report.qualification().participation_revision(), Some(&base));
    for key in changed {
        let cx2 = list_context(
            &c,
            key.clone(),
            vec![
                list_fact(&c, &owner, false, true, true),
                list_fact(&c, &items[0], true, true, true),
            ],
        );
        let cancel = HtmlSemanticCancellation::new();
        cancel.cancel();
        let mut query = HtmlSemanticSession::new(
            &c,
            &cx2,
            HtmlValueInterpreter::new(10000),
            HtmlSemanticLimits::new(1000000, 100, 10000, 10000, 1000000, 100000),
            cancel,
        )
        .unwrap();
        let failure = query.query(std::slice::from_ref(&requested)).unwrap_err();
        assert_eq!(
            failure.qualification().unwrap().participation_revision(),
            Some(&key)
        );
        assert_ne!(failure.qualification().unwrap(), report.qualification());
        assert_eq!(query.query(&[]).unwrap_err(), failure);
    }
}

#[test]
fn exact_list_resource_admission_and_late_failure_leave_no_scratch_or_result_prefix() {
    let (c, owner, items) = list_fixture("ol", &[("start", "99")], &[("li", None), ("li", None)]);
    let cx = complete_list_context(&c, &owner, &items);
    let requests: Vec<_> = items
        .iter()
        .map(|e| request(&c, e, HtmlSemanticQuery::ListOrdinal))
        .collect();
    let baseline = session(&c, &cx).query(&requests).unwrap();
    let usage = baseline.usage();
    assert_eq!(usage.scratch_records(), 0);
    assert!(usage.peak_scratch_records() >= 3);
    assert!(usage.allocated_bytes() > 0);
    let exact = HtmlSemanticLimits::new(
        usage.work(),
        usage.output_records(),
        usage.peak_scratch_records(),
        3,
        usage.allocated_bytes(),
        0,
    );
    let mut query = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(10000),
        exact,
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let report = query.query(&requests).unwrap();
    assert_eq!(report.usage(), usage);
    for (limits, resource) in [
        (
            HtmlSemanticLimits::new(
                usage.work() - 1,
                usage.output_records(),
                usage.peak_scratch_records(),
                3,
                usage.allocated_bytes(),
                0,
            ),
            HtmlSemanticResource::Work,
        ),
        (
            HtmlSemanticLimits::new(
                usage.work(),
                usage.output_records(),
                usage.peak_scratch_records() - 1,
                3,
                usage.allocated_bytes(),
                0,
            ),
            HtmlSemanticResource::ScratchRecords,
        ),
        (
            HtmlSemanticLimits::new(
                usage.work(),
                usage.output_records(),
                usage.peak_scratch_records(),
                2,
                usage.allocated_bytes(),
                0,
            ),
            HtmlSemanticResource::IntegerDigits,
        ),
        (
            HtmlSemanticLimits::new(
                usage.work(),
                usage.output_records(),
                usage.peak_scratch_records(),
                3,
                usage.allocated_bytes() - 1,
                0,
            ),
            HtmlSemanticResource::AllocatedBytes,
        ),
    ] {
        let cancellation = HtmlSemanticCancellation::new();
        let mut query = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(10000),
            limits,
            cancellation.clone(),
        )
        .unwrap();
        let failure = query.query(&requests).unwrap_err();
        assert!(
            matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: actual, .. } if actual == &resource)
        );
        assert_eq!(query.usage().scratch_records(), 0);
        assert_eq!(
            failure.qualification().unwrap().participation_revision(),
            cx.participation_revision()
        );
        cancellation.cancel();
        assert_eq!(query.query(&[]).unwrap_err(), failure);
    }
}

#[test]
fn missing_parent_and_css_box_coverage_are_required_for_owner_proof() {
    let (mut b, d) = builder();
    let wrapper = element(&mut b, &d, "div", &[]);
    let item = element(&mut b, &d, "li", &[]);
    b.establish_children(&wrapper, std::slice::from_ref(&item))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = list_context(
        &c,
        participation_revision(),
        vec![
            list_fact(&c, &item, true, true, true),
            list_fact(&c, &wrapper, false, true, true),
        ],
    );
    let r = session(&c, &cx)
        .query(&[request(&c, &item, HtmlSemanticQuery::ListOwner)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::ListOwner(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(
            c.subject_id(&wrapper).unwrap()
        )))
    );
    let (c, owner, items) = list_fixture("ol", &[], &[("li", None)]);
    let cx = list_context(
        &c,
        participation_revision(),
        vec![list_fact(&c, &items[0], true, true, true)],
    );
    let r = session(&c, &cx)
        .query(&[request(&c, &items[0], HtmlSemanticQuery::ListOwner)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::ListOwner(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::ProducesCssBox(c.subject_id(&owner).unwrap())
        ))
    );
}

#[test]
fn exact_utf16_invalid_integer_recovery_and_present_empty_start_follow_fallback() {
    let (mut b, d) = builder();
    let owner = b
        .add_element(
            &d,
            name(Namespace::html(), "ol"),
            OrderedAttributes::new([
                Attribute::new(
                    name(Namespace::none(), "start"),
                    DocumentString::from_utf16(vec![0xd800]),
                ),
                Attribute::new(name(Namespace::none(), "reversed"), ""),
            ])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Unavailable,
        )
        .unwrap();
    let item = element(&mut b, &d, "li", &[("value", "")]);
    b.establish_root(&owner).unwrap();
    b.establish_children(&owner, std::slice::from_ref(&item))
        .unwrap();
    b.establish_children(&item, &[]).unwrap();
    let c = b.finish().unwrap();
    let cx = complete_list_context(&c, &owner, std::slice::from_ref(&item));
    assert_ordinals(&c, &cx, std::slice::from_ref(&item), &["1"]);
    let HtmlCapturedNodeKind::Element(element) =
        c.node(&c.subject_id(&owner).unwrap()).unwrap().kind()
    else {
        panic!("element expected")
    };
    assert_eq!(
        element
            .attributes()
            .iter()
            .next()
            .unwrap()
            .value()
            .code_units(),
        &[0xd800]
    );
}

#[test]
fn reversed_start_does_not_require_non_li_classification_or_nested_box_subtree() {
    let (mut b, d) = builder();
    let owner = element(&mut b, &d, "ol", &[("reversed", "")]);
    let li = element(&mut b, &d, "li", &[]);
    let other = element(&mut b, &d, "div", &[]);
    let nested = element(&mut b, &d, "ul", &[]);
    b.establish_root(&owner).unwrap();
    b.establish_children(&owner, &[li.clone(), other.clone(), nested.clone()])
        .unwrap();
    b.establish_children(&li, &[]).unwrap();
    b.establish_children(&other, &[]).unwrap();
    // The nested list's ordered children are unavailable. Its established CSS
    // box still proves that its descendants cannot be owned by the outer ol.
    let c = b.finish().unwrap();
    let cx = list_context(
        &c,
        participation_revision(),
        vec![
            list_fact(&c, &owner, false, true, true),
            list_fact(&c, &li, true, true, true),
            HtmlListParticipationFact::new(
                c.subject_id(&nested).unwrap(),
                None,
                Some(true),
                Some(true),
            ),
        ],
    );
    let r = session(&c, &cx)
        .query(&[request(&c, &owner, HtmlSemanticQuery::ListStartingValue)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::ListStartingValue(HtmlSemanticValue::Known(HtmlInteger::from(1i128)))
    );
    // Ordinal enumeration needs possible non-li contributors as well.
    let r = session(&c, &cx)
        .query(&[request(&c, &li, HtmlSemanticQuery::ListOrdinal)])
        .unwrap();
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::ListOrdinal(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::ComputedListItem(c.subject_id(&other).unwrap())
        ))
    );
}

#[test]
fn html_hidden_and_offscreen_do_not_replace_supplied_rendering_participation() {
    let (mut b, d) = builder();
    let owner = element(&mut b, &d, "ol", &[]);
    let item = element(&mut b, &d, "li", &[("hidden", "")]);
    b.establish_root(&owner).unwrap();
    b.establish_children(&owner, std::slice::from_ref(&item))
        .unwrap();
    b.establish_children(&item, &[]).unwrap();
    let c = b.finish().unwrap();
    // A producer can establish boxes even when hidden is authored (stylesheet
    // override), or while the box is offscreen. Neither is an exclusion input.
    let cx = complete_list_context(&c, &owner, std::slice::from_ref(&item));
    let report = session(&c, &cx)
        .query(&[
            request(&c, &item, HtmlSemanticQuery::Hidden),
            request(&c, &item, HtmlSemanticQuery::ListOrdinal),
        ])
        .unwrap();
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::Hidden(HtmlSemanticValue::Known(HtmlHiddenState::Keyword(
            HtmlHidden::Hidden
        )))
    );
    assert_eq!(
        report.results()[1].fact(),
        &HtmlSemanticFact::ListOrdinal(HtmlSemanticValue::Known(Some(HtmlInteger::from(1i128))))
    );
}

fn fallback_context(
    c: &HtmlCapture,
    document: &HtmlCaptureBuilderSubject,
    protocol: HtmlProtocolLanguage,
) -> HtmlSemanticContext {
    let facts = HtmlLanguageDirectionFacts::new(
        c,
        c.host_facts_revision().clone(),
        vec![HtmlDocumentLanguageFallback::new(
            c.subject_id(document).unwrap(),
            HtmlPragmaLanguage::Absent,
            protocol,
        )],
        vec![],
        vec![],
        HtmlLanguageDirectionLimits::new(1, 0, 100, 10000),
    )
    .unwrap();
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_language_direction(facts),
        0,
    )
    .unwrap()
}

#[test]
fn aggregate_context_qualifies_changed_language_facts_and_unicode_direction_source() {
    let (mut b, d) = builder();
    let owner = element(&mut b, &d, "div", &[("dir", "auto")]);
    let text = b.add_text(&d, "\u{0627}".into()).unwrap();
    b.establish_children(&d, std::slice::from_ref(&owner))
        .unwrap();
    b.establish_children(&owner, std::slice::from_ref(&text))
        .unwrap();
    let c = b.finish().unwrap();
    let english = fallback_context(
        &c,
        &d,
        HtmlProtocolLanguage::Single(DocumentString::from("en")),
    );
    let french = fallback_context(
        &c,
        &d,
        HtmlProtocolLanguage::Single(DocumentString::from("fr")),
    );
    let requests = [
        request(&c, &owner, HtmlSemanticQuery::Language),
        request(&c, &owner, HtmlSemanticQuery::Directionality),
    ];
    let report = session(&c, &english).query(&requests).unwrap();
    let other = session(&c, &french).query(&requests).unwrap();
    for (report, tag) in [(&report, "en"), (&other, "fr")] {
        let HtmlSemanticFact::Language(HtmlSemanticValue::Known(language)) =
            report.results()[0].fact()
        else {
            panic!("language expected")
        };
        assert_eq!(language.tag(), &DocumentString::from(tag));
        assert_eq!(
            language.origin(),
            &HtmlLanguageOrigin::Protocol(c.subject_id(&d).unwrap())
        );
        assert_eq!(
            report.results()[1].fact(),
            &HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(HtmlDirectionality::Rtl))
        );
        let source = report.qualification().directionality_source();
        assert_eq!(source.unicode_version(), "18.0.0");
        assert_eq!(source.uax9_revision(), 52);
        assert_eq!(
            source.derived_bidi_class_sha256(),
            "d9e23222522551348ea1ccfbb4f62efbf98982afb95840f8959c08ed992c5607"
        );
    }
    assert_eq!(report.capture(), other.capture());
    assert_eq!(report.host_revision(), other.host_revision());
    assert_ne!(report.qualification(), other.qualification());
    assert_eq!(report.usage().scratch_records(), 0);
}

#[test]
fn aggregate_context_rejects_foreign_supplement_before_minting_identity() {
    let (b, _) = builder();
    let c = b.finish().unwrap();
    let (other, _) = builder();
    let other = other.finish().unwrap();
    let facts = HtmlLanguageDirectionFacts::new(
        &other,
        other.host_facts_revision().clone(),
        vec![],
        vec![],
        vec![],
        HtmlLanguageDirectionLimits::new(0, 0, 0, 0),
    )
    .unwrap();
    assert_eq!(
        HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_language_direction(facts),
            0
        )
        .unwrap_err(),
        HtmlSemanticContextError::LanguageDirectionCaptureMismatch
    );
}

#[test]
fn language_supplement_changes_do_not_join_retained_terminal_failures() {
    let (b, d) = builder();
    let c = b.finish().unwrap();
    let absent = fallback_context(&c, &d, HtmlProtocolLanguage::Absent);
    let unavailable = fallback_context(&c, &d, HtmlProtocolLanguage::Unavailable);
    let limits = HtmlSemanticLimits::new(1000000, 100, 10000, 10000, 1000000, 100000);
    let failure = cancelled_failure(&c, &absent, limits, 10000);
    let changed = cancelled_failure(&c, &unavailable, limits, 10000);
    assert_ne!(failure.qualification(), changed.qualification());
    assert_eq!(
        failure.qualification().unwrap().directionality_source(),
        changed.qualification().unwrap().directionality_source()
    );
    let absent_report = session(&c, &absent)
        .query(&[request(&c, &d, HtmlSemanticQuery::Language)])
        .unwrap();
    let HtmlSemanticFact::Language(HtmlSemanticValue::Known(language)) =
        absent_report.results()[0].fact()
    else {
        panic!("known empty language expected")
    };
    assert!(language.is_explicitly_unknown());
    assert_eq!(
        language.origin(),
        &HtmlLanguageOrigin::NoInformation(c.subject_id(&d).unwrap())
    );
    let unavailable_report = session(&c, &unavailable)
        .query(&[request(&c, &d, HtmlSemanticQuery::Language)])
        .unwrap();
    assert_eq!(
        unavailable_report.results()[0].fact(),
        &HtmlSemanticFact::Language(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::LanguageFallback(c.subject_id(&d).unwrap())
        ))
    );
}

#[test]
fn one_complete_batch_composes_old_queries_and_fixed_scalar_range_facts() {
    let (mut b, d) = builder();
    let meter = element(&mut b, &d, "meter", &[]);
    let progress = element(&mut b, &d, "progress", &[("max", "4"), ("value", "1")]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let report = session(&c, &cx)
        .query(&[
            request(&c, &meter, HtmlSemanticQuery::Hidden),
            request(&c, &meter, HtmlSemanticQuery::Meter),
            request(&c, &progress, HtmlSemanticQuery::Progress),
        ])
        .unwrap();
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::Hidden(HtmlSemanticValue::Known(HtmlHiddenState::NotHidden))
    );
    let HtmlSemanticFact::Meter(HtmlSemanticValue::Known(meter)) = report.results()[1].fact()
    else {
        panic!("meter expected")
    };
    assert_eq!(meter.value().value(), 0.0);
    assert_eq!(meter.maximum().value(), 1.0);
    let HtmlSemanticFact::Progress(HtmlSemanticValue::Known(HtmlProgressState::Determinate(
        progress,
    ))) = report.results()[2].fact()
    else {
        panic!("determinate progress expected")
    };
    assert_eq!(progress.position().value(), 0.25);
    assert_eq!(report.usage().scratch_records(), 0);
    assert!(cx.language_direction().is_none());
}

fn empty_inert_facts(c: &HtmlCapture, revision: HtmlFlatTreeRevision) -> HtmlInertFacts {
    HtmlInertFacts::new(
        c,
        c.host_facts_revision().clone(),
        revision,
        vec![],
        vec![],
        HtmlInertFactLimits::new(0, 0),
    )
    .unwrap()
}

fn empty_control_facts(c: &HtmlCapture) -> HtmlControlFacts {
    HtmlControlFacts::new(
        c,
        c.host_facts_revision().clone(),
        vec![],
        vec![],
        vec![],
        vec![],
        HtmlControlFactsLimits::new(0, 0),
    )
    .unwrap()
}

#[test]
fn aggregate_rejects_foreign_inert_and_control_inputs_and_wrong_flat_revision() {
    let (b, _) = builder();
    let c = b.finish().unwrap();
    let (b, _) = builder();
    let other = b.finish().unwrap();
    let key = HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 2, 3);
    let wrong = HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 2, 3);
    for (input, expected) in [
        (
            HtmlSemanticContextInput::new(vec![])
                .with_inert(key.clone(), Some(empty_inert_facts(&other, key.clone()))),
            HtmlSemanticContextError::InertCaptureMismatch,
        ),
        (
            HtmlSemanticContextInput::new(vec![]).with_controls(empty_control_facts(&other)),
            HtmlSemanticContextError::ControlsCaptureMismatch,
        ),
        (
            HtmlSemanticContextInput::new(vec![])
                .with_inert(wrong, Some(empty_inert_facts(&c, key))),
            HtmlSemanticContextError::InertRevisionMismatch,
        ),
    ] {
        assert_eq!(
            HtmlSemanticContext::new(&c, c.host_facts_revision().clone(), input, 0).unwrap_err(),
            expected
        );
    }
}

#[test]
fn context_admission_preserves_host_supplement_and_classification_error_order() {
    let (b, _) = builder();
    let capture = b.finish().unwrap();
    let (mut b, document) = builder();
    let custom = element(&mut b, &document, "x-control", &[]);
    let foreign = b.finish().unwrap();
    let classification =
        HtmlCustomControlClassification::new(foreign.subject_id(&custom).unwrap(), true);
    let flat_revision = HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 2, 3);
    let participation_revision = participation_revision();
    let classifications = || HtmlSemanticContextInput::new(vec![classification.clone()]);
    let participation = || {
        classifications().with_list_participation(
            participation_revision.clone(),
            Some(
                HtmlListParticipation::new(&foreign, participation_revision.clone(), vec![], 0)
                    .unwrap(),
            ),
        )
    };
    let controls = || participation().with_controls(empty_control_facts(&foreign));
    let inert = || {
        controls().with_inert(
            flat_revision.clone(),
            Some(empty_inert_facts(&foreign, flat_revision.clone())),
        )
    };

    // Every earlier invalid input dominates the still-invalid later inputs.
    assert_eq!(
        HtmlSemanticContext::new(&capture, HtmlHostFactsRevision::fresh(), inert(), 0).unwrap_err(),
        HtmlSemanticContextError::HostRevisionMismatch
    );
    for (input, cap, expected) in [
        (inert(), 0, HtmlSemanticContextError::InertCaptureMismatch),
        (
            controls(),
            0,
            HtmlSemanticContextError::ControlsCaptureMismatch,
        ),
        (
            participation(),
            0,
            HtmlSemanticContextError::ParticipationCaptureMismatch,
        ),
        (
            classifications(),
            0,
            HtmlSemanticContextError::ClassificationLimit {
                limit: 0,
                retained_capacity: 1,
            },
        ),
        (
            classifications(),
            1,
            HtmlSemanticContextError::Capture(HtmlCaptureError::ForeignCapture),
        ),
    ] {
        assert_eq!(
            HtmlSemanticContext::new(&capture, capture.host_facts_revision().clone(), input, cap,)
                .unwrap_err(),
            expected
        );
    }
}

#[test]
fn flat_producer_configuration_and_revision_qualify_reports_and_sticky_failures() {
    let (mut b, d) = builder();
    let own = element(&mut b, &d, "div", &[("inert", "false")]);
    let c = b.finish().unwrap();
    let producer = HtmlFlatTreeProducerId::fresh();
    let keys = [
        HtmlFlatTreeRevision::new(producer.clone(), 2, 3),
        HtmlFlatTreeRevision::new(producer.clone(), 4, 3),
        HtmlFlatTreeRevision::new(producer, 2, 4),
        HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 2, 3),
    ];
    let limits = HtmlSemanticLimits::new(1000000, 100, 10000, 10000, 1000000, 100000);
    let contexts: Vec<_> = keys
        .iter()
        .map(|key| {
            HtmlSemanticContext::new(
                &c,
                c.host_facts_revision().clone(),
                HtmlSemanticContextInput::new(vec![]).with_inert(key.clone(), None),
                0,
            )
            .unwrap()
        })
        .collect();
    let mut failures = Vec::new();
    for (cx, key) in contexts.iter().zip(&keys) {
        // Own inertness needs no flat relation or modal payload.
        let report = session(&c, cx)
            .query(&[request(&c, &own, HtmlSemanticQuery::Inertness)])
            .unwrap();
        assert_eq!(
            report.results()[0].fact(),
            &HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
        );
        assert_eq!(report.qualification().inert_revision(), Some(key));
        let failure = cancelled_failure(&c, cx, limits, 10000);
        assert_eq!(failure.qualification().unwrap().inert_revision(), Some(key));
        assert_eq!(failure.qualification(), Some(report.qualification()));
        failures.push(failure);
    }
    for changed in &failures[1..] {
        assert_ne!(failures[0].qualification(), changed.qualification());
    }
    let cx = &contexts[0];
    let cancel = HtmlSemanticCancellation::new();
    let mut live = HtmlSemanticSession::new(
        &c,
        cx,
        HtmlValueInterpreter::new(10000),
        limits,
        cancel.clone(),
    )
    .unwrap();
    cancel.cancel();
    let first = live.query(&[]).unwrap_err();
    let repeated = live
        .query(&[request(&c, &own, HtmlSemanticQuery::Inertness)])
        .unwrap_err();
    assert_eq!(first, repeated);
    assert_eq!(
        first.qualification().unwrap().inert_revision(),
        Some(&keys[0])
    );
}

#[test]
fn supplied_control_values_change_context_and_failure_identity() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[("checked", "")]);
    let c = b.finish().unwrap();
    let mut reports = Vec::new();
    let mut failures = Vec::new();
    for current in [false, true] {
        let facts = HtmlControlFacts::new(
            &c,
            c.host_facts_revision().clone(),
            vec![],
            vec![HtmlInputControlState::new(
                c.subject_id(&input).unwrap(),
                Some(current),
                None,
                None,
                None,
            )],
            vec![],
            vec![],
            HtmlControlFactsLimits::new(1, 10000),
        )
        .unwrap();
        let cx = HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_controls(facts),
            0,
        )
        .unwrap();
        let report = session(&c, &cx)
            .query(&[request(&c, &input, HtmlSemanticQuery::CurrentCheckedness)])
            .unwrap();
        assert_eq!(
            report.results()[0].fact(),
            &HtmlSemanticFact::CurrentCheckedness(HtmlSemanticValue::Known(current))
        );
        assert!(report.qualification().inert_revision().is_none());
        let failure = cancelled_failure(
            &c,
            &cx,
            HtmlSemanticLimits::new(1000000, 100, 10000, 10000, 1000000, 100000),
            10000,
        );
        assert_eq!(failure.qualification(), Some(report.qualification()));
        reports.push(report);
        failures.push(failure);
    }
    assert_ne!(reports[0].context(), reports[1].context());
    assert_ne!(failures[0].qualification(), failures[1].qualification());
    assert_eq!(reports[0].capture(), reports[1].capture());
    assert_eq!(reports[0].host_revision(), reports[1].host_revision());
}

#[test]
fn mixed_inert_and_control_batch_preserves_known_missing_and_inapplicable_facts() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[("inert", ""), ("disabled", "")]);
    let c = b.finish().unwrap();
    let cx = context(&c); // Old queries need none of the new supplements.
    let report = session(&c, &cx)
        .query(&[
            request(&c, &input, HtmlSemanticQuery::Hidden),
            request(&c, &input, HtmlSemanticQuery::Disabledness),
            request(&c, &input, HtmlSemanticQuery::Inertness),
            request(&c, &d, HtmlSemanticQuery::Inertness),
            request(&c, &input, HtmlSemanticQuery::CurrentCheckedness),
            request(&c, &input, HtmlSemanticQuery::FormOwner),
            request(&c, &d, HtmlSemanticQuery::CurrentCheckedness),
            request(&c, &input, HtmlSemanticQuery::LabeledControl),
        ])
        .unwrap();
    let id = c.subject_id(&input).unwrap();
    let expected = [
        HtmlSemanticFact::Hidden(HtmlSemanticValue::Known(HtmlHiddenState::NotHidden)),
        disabled(true),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true)),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::ModalBlocking(c.subject_id(&d).unwrap()),
        )),
        HtmlSemanticFact::CurrentCheckedness(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::ControlState {
                subject: id.clone(),
                flag: HtmlControlFlag::Checkedness,
            },
        )),
        HtmlSemanticFact::FormOwner(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::FormAssociation(id),
        )),
        HtmlSemanticFact::CurrentCheckedness(HtmlSemanticValue::Inapplicable),
        HtmlSemanticFact::LabeledControl(HtmlSemanticValue::Inapplicable),
    ];
    for (result, expected) in report.results().iter().zip(&expected) {
        assert_eq!(result.fact(), expected);
    }
    assert_eq!(report.results().len(), 8);
    assert_eq!(report.usage().scratch_records(), 0);
}

#[test]
fn control_or_inert_limit_stop_discards_whole_mixed_batch_and_retains_key() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[("inert", "")]);
    let c = b.finish().unwrap();
    let key = HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 1, 2);
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![])
            .with_inert(key.clone(), None)
            .with_controls(empty_control_facts(&c)),
        0,
    )
    .unwrap();
    let limits = HtmlSemanticLimits::new(1000000, 1, 10000, 10000, 1000000, 100000);
    let mut live = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(10000),
        limits,
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let completed = live
        .query(&[request(&c, &input, HtmlSemanticQuery::Inertness)])
        .unwrap();
    let failed = live
        .query(&[request(&c, &input, HtmlSemanticQuery::CurrentCheckedness)])
        .unwrap_err();
    assert_eq!(failed.qualification(), Some(completed.qualification()));
    assert_eq!(failed.qualification().unwrap().inert_revision(), Some(&key));
    assert_eq!(failed.usage().scratch_records(), 0);
    let repeated = live
        .query(&[request(&c, &input, HtmlSemanticQuery::FormOwner)])
        .unwrap_err();
    assert_eq!(repeated, failed);
    assert_eq!(
        completed.results()[0].fact(),
        &HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
}

#[test]
fn late_inert_attribute_stop_publishes_no_completed_control_prefix() {
    let (mut b, d) = builder();
    let input = element(&mut b, &d, "input", &[("inert", "false")]);
    let c = b.finish().unwrap();
    let facts = HtmlControlFacts::new(
        &c,
        c.host_facts_revision().clone(),
        vec![],
        vec![HtmlInputControlState::new(
            c.subject_id(&input).unwrap(),
            Some(false),
            None,
            None,
            None,
        )],
        vec![],
        vec![],
        HtmlControlFactsLimits::new(1, 10000),
    )
    .unwrap();
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_controls(facts),
        0,
    )
    .unwrap();
    let mut live = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(0),
        HtmlSemanticLimits::new(1000000, 100, 10000, 10000, 1000000, 100000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let failure = live
        .query(&[
            request(&c, &input, HtmlSemanticQuery::CurrentCheckedness),
            request(&c, &input, HtmlSemanticQuery::Inertness),
        ])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(failure.usage().output_records(), 2);
    assert_eq!(failure.usage().scratch_records(), 0);
    assert_eq!(failure.qualification().unwrap().context(), cx.identity());
    assert_eq!(failure.qualification().unwrap().attribute_input_limit(), 0);
    assert_eq!(live.query(&[]).unwrap_err(), failure);
}

fn global_input(
    c: &HtmlCapture,
    revision: HtmlGlobalInheritanceRevision,
) -> HtmlGlobalInheritanceFacts {
    HtmlGlobalInheritanceFacts::new(
        c,
        c.host_facts_revision().clone(),
        revision,
        vec![],
        vec![],
        HtmlGlobalInheritanceLimits::new(0, 0),
    )
    .unwrap()
}

#[test]
fn aggregate_global_context_rejects_foreign_capture_and_stale_producer_key() {
    let (b, _) = builder();
    let c = b.finish().unwrap();
    let (b, _) = builder();
    let other = b.finish().unwrap();
    let producer = HtmlGlobalInheritanceProducerId::fresh();
    let expected = HtmlGlobalInheritanceRevision::new(producer.clone(), 4, 5);
    let stale = HtmlGlobalInheritanceRevision::new(producer, 4, 4);
    for (input, error) in [
        (
            HtmlSemanticContextInput::new(vec![]).with_global_inheritance(
                expected.clone(),
                Some(global_input(&other, expected.clone())),
            ),
            HtmlSemanticContextError::GlobalInheritanceCaptureMismatch,
        ),
        (
            HtmlSemanticContextInput::new(vec![])
                .with_global_inheritance(expected, Some(global_input(&c, stale))),
            HtmlSemanticContextError::GlobalInheritanceRevisionMismatch,
        ),
    ] {
        assert_eq!(
            HtmlSemanticContext::new(&c, c.host_facts_revision().clone(), input, 0).unwrap_err(),
            error
        );
    }
}

#[test]
fn global_producer_keys_and_actual_document_payloads_qualify_complete_and_terminal_results() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[("contenteditable", "false")]);
    b.establish_parent(&html, &d).unwrap();
    let c = b.finish().unwrap();
    let producer = HtmlGlobalInheritanceProducerId::fresh();
    let keys = [
        HtmlGlobalInheritanceRevision::new(producer.clone(), 3, 4),
        HtmlGlobalInheritanceRevision::new(producer.clone(), 9, 4),
        HtmlGlobalInheritanceRevision::new(producer, 3, 5),
        HtmlGlobalInheritanceRevision::new(HtmlGlobalInheritanceProducerId::fresh(), 3, 4),
    ];
    let limits = HtmlSemanticLimits::new(1000000, 100, 10000, 10000, 1000000, 100000);
    let mut failures = Vec::new();
    for (key, state, want) in keys
        .iter()
        .map(|k| (k, HtmlDesignMode::Enabled, true))
        .chain([(&keys[0], HtmlDesignMode::Disabled, false)])
    {
        let facts = HtmlGlobalInheritanceFacts::new(
            &c,
            c.host_facts_revision().clone(),
            key.clone(),
            vec![HtmlDocumentDesignModeFact::new(
                c.subject_id(&d).unwrap(),
                state,
            )],
            vec![],
            HtmlGlobalInheritanceLimits::new(1, 2),
        )
        .unwrap();
        let cx = HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_global_inheritance(key.clone(), Some(facts)),
            0,
        )
        .unwrap();
        let report = session(&c, &cx)
            .query(&[
                request(&c, &html, HtmlSemanticQuery::IsContentEditable),
                request(&c, &html, HtmlSemanticQuery::Hidden),
            ])
            .unwrap();
        assert_eq!(
            report.results()[0].fact(),
            &HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Known(want))
        );
        assert_eq!(
            report.results()[1].fact(),
            &HtmlSemanticFact::Hidden(HtmlSemanticValue::Known(HtmlHiddenState::NotHidden))
        );
        assert_eq!(
            report.qualification().global_inheritance_revision(),
            Some(key)
        );
        let failure = cancelled_failure(&c, &cx, limits, 10000);
        assert_eq!(failure.qualification(), Some(report.qualification()));
        assert_eq!(
            report.qualification().editability_source().revision(),
            "85de9ff4a9816139d06fc2dc39d5dacdc456fddc"
        );
        failures.push(failure);
    }
    for changed in &failures[1..] {
        assert_ne!(changed.qualification(), failures[0].qualification());
    }
    assert_eq!(
        failures[4]
            .qualification()
            .unwrap()
            .global_inheritance_revision(),
        failures[0]
            .qualification()
            .unwrap()
            .global_inheritance_revision()
    );
    assert_ne!(
        failures[4].qualification().unwrap().context(),
        failures[0].qualification().unwrap().context()
    );
}

#[test]
fn expected_global_key_without_payload_keeps_old_and_explicit_queries_independent() {
    let (mut b, d) = builder();
    let e = element(
        &mut b,
        &d,
        "div",
        &[
            ("contenteditable", ""),
            ("spellcheck", "true"),
            ("translate", "no"),
            ("disabled", ""),
        ],
    );
    let c = b.finish().unwrap();
    let expected =
        HtmlGlobalInheritanceRevision::new(HtmlGlobalInheritanceProducerId::fresh(), 1, 1);
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_global_inheritance(expected.clone(), None),
        0,
    )
    .unwrap();
    let report = session(&c, &cx)
        .query(&[
            request(&c, &e, HtmlSemanticQuery::Hidden),
            request(&c, &e, HtmlSemanticQuery::IsContentEditable),
            request(&c, &e, HtmlSemanticQuery::Spellcheck),
            request(&c, &e, HtmlSemanticQuery::TranslationMode),
        ])
        .unwrap();
    let expected_facts = [
        HtmlSemanticFact::Hidden(HtmlSemanticValue::Known(HtmlHiddenState::NotHidden)),
        HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Known(true)),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(true)),
        HtmlSemanticFact::TranslationMode(HtmlSemanticValue::Known(HtmlTranslationMode::Disabled)),
    ];
    for (result, expected) in report.results().iter().zip(&expected_facts) {
        assert_eq!(result.fact(), expected);
    }
    assert_eq!(
        report.qualification().global_inheritance_revision(),
        Some(&expected)
    );
}
