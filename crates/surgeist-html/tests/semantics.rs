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
fn context(c: &HtmlCapture) -> HtmlSemanticContext {
    HtmlSemanticContext::new(c, c.host_facts_revision().clone(), vec![], 0).unwrap()
}
fn session<'a>(c: &'a HtmlCapture, cx: &'a HtmlSemanticContext) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(
        c,
        cx,
        HtmlValueInterpreter::new(10000),
        HtmlSemanticLimits::new(1000000, 100),
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
        HtmlSemanticContext::new(&c, HtmlHostFactsRevision::fresh(), facts.clone(), 2),
        Err(HtmlSemanticContextError::HostRevisionMismatch)
    ));
    let cx = HtmlSemanticContext::new(&c, c.host_facts_revision().clone(), facts, 2).unwrap();
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
        HtmlSemanticLimits::new(exact - 1, 2),
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
        HtmlSemanticLimits::new(exact, 1),
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
        HtmlSemanticLimits::new(0, 0),
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
            HtmlSemanticLimits::new(10, 1),
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
        vec![HtmlCustomControlClassification::new(id.clone(), true)],
        1,
    )
    .unwrap();
    let no = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        vec![HtmlCustomControlClassification::new(id, false)],
        1,
    )
    .unwrap();
    let cancel = HtmlSemanticCancellation::new();
    let mut s = HtmlSemanticSession::new(
        &c,
        &yes,
        HtmlValueInterpreter::new(10),
        HtmlSemanticLimits::new(100000, 1),
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
        HtmlSemanticLimits::new(100000, 1),
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
        HtmlSemanticContext::new(&c, c.host_facts_revision().clone(), vec![fact.clone()], 0),
        Err(HtmlSemanticContextError::ClassificationLimit { .. })
    ));
    assert!(matches!(
        HtmlSemanticContext::new(
            &c,
            c.host_facts_revision().clone(),
            vec![fact.clone(), fact.clone()],
            2
        ),
        Err(HtmlSemanticContextError::DuplicateSubject(_))
    ));
    let (other, _) = builder();
    let oc = other.finish().unwrap();
    assert_eq!(
        HtmlSemanticContext::new(&oc, oc.host_facts_revision().clone(), vec![fact], 1).unwrap_err(),
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
        vec![HtmlCustomControlClassification::new(subject.clone(), true)],
        1,
    )
    .unwrap();
    let no = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        vec![HtmlCustomControlClassification::new(subject, false)],
        1,
    )
    .unwrap();
    let limits = HtmlSemanticLimits::new(1000, 2);
    let first = cancelled_failure(&c, &yes, limits, 10);
    let changed_context = cancelled_failure(&c, &no, limits, 10);
    let changed_work = cancelled_failure(&c, &yes, HtmlSemanticLimits::new(1001, 2), 10);
    let changed_output = cancelled_failure(&c, &yes, HtmlSemanticLimits::new(1000, 3), 10);
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
    let limits = HtmlSemanticLimits::new(100000, 1);
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
            HtmlSemanticLimits::new(0, 1),
            10,
            HtmlSemanticResource::Work,
        ),
        (
            HtmlSemanticLimits::new(100000, 0),
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
    let limits = HtmlSemanticLimits::new(100000, 1);
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
