use surgeist_html::*;

fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(1000, 10000, 1000, 100000, 1000, 0, 0),
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
    ns: Namespace,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, local),
        OrderedAttributes::new(
            attrs
                .iter()
                .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
        )
        .unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
) -> HtmlCaptureBuilderSubject {
    element(b, d, Namespace::html(), local, &[])
}
fn children(
    b: &mut HtmlCaptureBuilder,
    parent: &HtmlCaptureBuilderSubject,
    items: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(parent, items).unwrap();
}
fn context(
    c: &HtmlCapture,
    selected: &[(HtmlCaptureSubjectId, Option<bool>)],
) -> HtmlSemanticContext {
    let facts = HtmlControlFacts::new(
        c,
        c.host_facts_revision().clone(),
        vec![],
        vec![],
        selected
            .iter()
            .map(|(id, v)| HtmlOptionControlState::new(id.clone(), *v, None))
            .collect(),
        vec![],
        HtmlControlFactsLimits::new(1000, 1000000),
    )
    .unwrap();
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_controls(facts),
        0,
    )
    .unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 1000, 0, 1000000, 1000)
}
fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    cancel: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(c, cx, HtmlValueInterpreter::new(0), l, cancel).unwrap()
}
fn request(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn query(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
) -> HtmlSemanticFact {
    session(c, cx, limits(), HtmlSemanticCancellation::new())
        .query(&[request(id, q)])
        .unwrap()
        .results()[0]
        .fact()
        .clone()
}
fn options(f: &HtmlSemanticFact) -> &[HtmlCaptureSubjectId] {
    match f {
        HtmlSemanticFact::SelectOptions(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::SelectedOptions(HtmlSemanticValue::Known(v)) => v.options(),
        other => panic!("expected complete collection, got {other:?}"),
    }
}
fn simple(count: usize) -> (HtmlCapture, HtmlCaptureSubjectId, Vec<HtmlCaptureSubjectId>) {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select");
    let os: Vec<_> = (0..count).map(|_| html(&mut b, &d, "option")).collect();
    children(&mut b, &s, &os);
    children(&mut b, &d, std::slice::from_ref(&s));
    let c = b.finish().unwrap();
    let id = c.subject_id(&s).unwrap();
    let ids = os.iter().map(|o| c.subject_id(o).unwrap()).collect();
    (c, id, ids)
}

#[test]
fn complete_source_order_prunes_all_html_barriers_and_nested_optgroups() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select");
    let wrapper = html(&mut b, &d, "div");
    let a = html(&mut b, &d, "option");
    let ob = element(
        &mut b,
        &d,
        Namespace::html(),
        "option",
        &[("disabled", ""), ("selected", "")],
    );
    let oc = html(&mut b, &d, "option");
    let hr = html(&mut b, &d, "hr");
    let od = html(&mut b, &d, "option");
    let dl = html(&mut b, &d, "datalist");
    let oe = html(&mut b, &d, "option");
    let group = html(&mut b, &d, "optgroup");
    let of = html(&mut b, &d, "option");
    let nested = html(&mut b, &d, "optgroup");
    let og = html(&mut b, &d, "option");
    let select = html(&mut b, &d, "select");
    let oh = html(&mut b, &d, "option");
    let foreign = element(&mut b, &d, Namespace::svg(), "select", &[]);
    let oi = html(&mut b, &d, "option");
    children(&mut b, &wrapper, std::slice::from_ref(&a));
    children(&mut b, &ob, std::slice::from_ref(&oc));
    children(&mut b, &hr, std::slice::from_ref(&od));
    children(&mut b, &dl, std::slice::from_ref(&oe));
    children(&mut b, &nested, std::slice::from_ref(&og));
    children(&mut b, &group, &[of.clone(), nested]);
    children(&mut b, &select, std::slice::from_ref(&oh));
    children(&mut b, &foreign, std::slice::from_ref(&oi));
    children(
        &mut b,
        &s,
        &[wrapper, ob.clone(), hr, dl, group, select, foreign],
    );
    children(&mut b, &d, std::slice::from_ref(&s));
    let c = b.finish().unwrap();
    let s = c.subject_id(&s).unwrap();
    let included = [a, ob, of, oi].map(|o| c.subject_id(&o).unwrap());
    let flags: Vec<_> = included
        .iter()
        .zip([false, true, true, false])
        .map(|(id, v)| (id.clone(), Some(v)))
        .collect();
    let cx = context(&c, &flags);
    assert_eq!(
        options(&query(&c, &cx, &s, HtmlSemanticQuery::SelectOptions)),
        included
    );
    assert_eq!(
        options(&query(&c, &cx, &s, HtmlSemanticQuery::SelectedOptions)),
        &included[1..3]
    );
    assert_eq!(
        query(&c, &cx, &s, HtmlSemanticQuery::SelectSelectedIndex),
        HtmlSemanticFact::SelectSelectedIndex(HtmlSemanticValue::Known(Some(1)))
    );
    for (index, id) in included.iter().enumerate() {
        assert_eq!(
            query(&c, &cx, id, HtmlSemanticQuery::OptionIndex),
            HtmlSemanticFact::OptionIndex(HtmlSemanticValue::Known(index))
        );
    }
    for o in [oc, od, oe, og, oh] {
        assert_eq!(
            query(
                &c,
                &cx,
                &c.subject_id(&o).unwrap(),
                HtmlSemanticQuery::OptionIndex
            ),
            HtmlSemanticFact::OptionIndex(HtmlSemanticValue::Known(0))
        );
    }
}

#[test]
fn empty_and_established_all_false_are_known_empty_and_minus_one() {
    for count in [0, 3] {
        let (c, s, os) = simple(count);
        let flags: Vec<_> = os.iter().map(|id| (id.clone(), Some(false))).collect();
        let cx = context(&c, &flags);
        assert!(options(&query(&c, &cx, &s, HtmlSemanticQuery::SelectedOptions)).is_empty());
        assert_eq!(
            query(&c, &cx, &s, HtmlSemanticQuery::SelectSelectedIndex),
            HtmlSemanticFact::SelectSelectedIndex(HtmlSemanticValue::Known(None))
        );
        assert_eq!(
            options(&query(&c, &cx, &s, HtmlSemanticQuery::SelectOptions)),
            os
        );
    }
}
#[test]
fn selectedness_is_current_state_and_does_not_use_authored_selected_or_defaults() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select");
    let a = element(&mut b, &d, Namespace::html(), "option", &[("selected", "")]);
    let ob = html(&mut b, &d, "option");
    children(&mut b, &s, &[a.clone(), ob.clone()]);
    let c = b.finish().unwrap();
    let s = c.subject_id(&s).unwrap();
    let a = c.subject_id(&a).unwrap();
    let ob = c.subject_id(&ob).unwrap();
    let cx = context(&c, &[(a, Some(false)), (ob.clone(), Some(true))]);
    assert_eq!(
        options(&query(&c, &cx, &s, HtmlSemanticQuery::SelectedOptions)),
        [ob]
    );
}
#[test]
fn unknown_before_selection_is_missing_and_unknown_after_is_irrelevant_only_to_first_index() {
    let (c, s, os) = simple(3);
    let cx = context(
        &c,
        &[
            (os[0].clone(), Some(false)),
            (os[1].clone(), Some(true)),
            (os[2].clone(), None),
        ],
    );
    assert_eq!(
        query(&c, &cx, &s, HtmlSemanticQuery::SelectSelectedIndex),
        HtmlSemanticFact::SelectSelectedIndex(HtmlSemanticValue::Known(Some(1)))
    );
    assert_eq!(
        query(&c, &cx, &s, HtmlSemanticQuery::SelectedOptions),
        HtmlSemanticFact::SelectedOptions(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::ControlState {
                subject: os[2].clone(),
                flag: HtmlControlFlag::Selectedness
            }
        ))
    );
    let cx = context(&c, &[(os[0].clone(), None), (os[1].clone(), Some(true))]);
    assert_eq!(
        query(&c, &cx, &s, HtmlSemanticQuery::SelectSelectedIndex),
        HtmlSemanticFact::SelectSelectedIndex(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::ControlState {
                subject: os[0].clone(),
                flag: HtmlControlFlag::Selectedness
            }
        ))
    );
    assert_eq!(
        options(&query(&c, &cx, &s, HtmlSemanticQuery::SelectOptions)),
        os
    );
}
#[test]
fn absent_state_record_is_missing_without_inventing_false() {
    let (c, s, os) = simple(1);
    let cx = context(&c, &[]);
    assert_eq!(
        query(&c, &cx, &s, HtmlSemanticQuery::SelectedOptions),
        HtmlSemanticFact::SelectedOptions(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::ControlState {
                subject: os[0].clone(),
                flag: HtmlControlFlag::Selectedness
            }
        ))
    );
}
#[test]
fn first_selection_and_target_index_do_not_demand_later_child_coverage() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select");
    let o = html(&mut b, &d, "option");
    let later = html(&mut b, &d, "div");
    children(&mut b, &s, &[o.clone(), later.clone()]);
    let c = b.finish().unwrap();
    let s = c.subject_id(&s).unwrap();
    let o = c.subject_id(&o).unwrap();
    let later = c.subject_id(&later).unwrap();
    let cx = context(&c, &[(o.clone(), Some(true))]);
    assert_eq!(
        query(&c, &cx, &s, HtmlSemanticQuery::SelectSelectedIndex),
        HtmlSemanticFact::SelectSelectedIndex(HtmlSemanticValue::Known(Some(0)))
    );
    assert_eq!(
        query(&c, &cx, &o, HtmlSemanticQuery::OptionIndex),
        HtmlSemanticFact::OptionIndex(HtmlSemanticValue::Known(0))
    );
    assert_eq!(
        query(&c, &cx, &s, HtmlSemanticQuery::SelectOptions),
        HtmlSemanticFact::SelectOptions(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(later)
        ))
    );
}
#[test]
fn required_earlier_child_order_is_missing_but_pruned_unknown_children_are_irrelevant() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select");
    let unknown = html(&mut b, &d, "div");
    let o = html(&mut b, &d, "option");
    children(&mut b, &s, &[unknown.clone(), o.clone()]);
    let c = b.finish().unwrap();
    let s = c.subject_id(&s).unwrap();
    let o = c.subject_id(&o).unwrap();
    let cx = context(&c, &[(o, Some(true))]);
    assert_eq!(
        query(&c, &cx, &s, HtmlSemanticQuery::SelectSelectedIndex),
        HtmlSemanticFact::SelectSelectedIndex(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(c.subject_id(&unknown).unwrap())
        ))
    );
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select");
    let nodes: Vec<_> = ["select", "hr", "datalist", "option"]
        .iter()
        .map(|n| html(&mut b, &d, n))
        .collect();
    children(&mut b, &s, &nodes);
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    let only = c.subject_id(&nodes[3]).unwrap();
    assert_eq!(
        options(&query(
            &c,
            &cx,
            &c.subject_id(&s).unwrap(),
            HtmlSemanticQuery::SelectOptions
        )),
        [only]
    );
}
#[test]
fn option_unknown_parent_differs_from_proven_no_owner() {
    let (mut b, d) = builder();
    let unknown = html(&mut b, &d, "option");
    let root = html(&mut b, &d, "option");
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    let unknown = c.subject_id(&unknown).unwrap();
    assert_eq!(
        query(&c, &cx, &unknown, HtmlSemanticQuery::OptionIndex),
        HtmlSemanticFact::OptionIndex(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(
            unknown
        )))
    );
    assert_eq!(
        query(
            &c,
            &cx,
            &c.subject_id(&root).unwrap(),
            HtmlSemanticQuery::OptionIndex
        ),
        HtmlSemanticFact::OptionIndex(HtmlSemanticValue::Known(0))
    );
}
#[test]
fn exact_expanded_names_control_applicability_and_foreign_wrappers_do_not_prune() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select");
    let upper = html(&mut b, &d, "OPTION");
    let foreign = element(&mut b, &d, Namespace::svg(), "option", &[]);
    let o = html(&mut b, &d, "option");
    children(&mut b, &upper, &[]);
    children(&mut b, &foreign, std::slice::from_ref(&o));
    children(&mut b, &s, &[upper.clone(), foreign.clone()]);
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    assert_eq!(
        options(&query(
            &c,
            &cx,
            &c.subject_id(&s).unwrap(),
            HtmlSemanticQuery::SelectOptions
        )),
        [c.subject_id(&o).unwrap()]
    );
    assert_eq!(
        query(
            &c,
            &cx,
            &c.subject_id(&upper).unwrap(),
            HtmlSemanticQuery::OptionIndex
        ),
        HtmlSemanticFact::OptionIndex(HtmlSemanticValue::Inapplicable)
    );
    assert_eq!(
        query(
            &c,
            &cx,
            &c.subject_id(&foreign).unwrap(),
            HtmlSemanticQuery::OptionIndex
        ),
        HtmlSemanticFact::OptionIndex(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
    assert_eq!(
        query(
            &c,
            &cx,
            &c.subject_id(&d).unwrap(),
            HtmlSemanticQuery::SelectOptions
        ),
        HtmlSemanticFact::SelectOptions(HtmlSemanticValue::Inapplicable)
    );
}
#[test]
fn state_only_context_changes_are_qualified_and_recomputed() {
    let (c, s, os) = simple(1);
    let a = context(&c, &[(os[0].clone(), Some(false))]);
    let b = context(&c, &[(os[0].clone(), Some(true))]);
    let req = request(&s, HtmlSemanticQuery::SelectSelectedIndex);
    let ar = session(&c, &a, limits(), HtmlSemanticCancellation::new())
        .query(std::slice::from_ref(&req))
        .unwrap();
    let br = session(&c, &b, limits(), HtmlSemanticCancellation::new())
        .query(std::slice::from_ref(&req))
        .unwrap();
    assert_eq!(
        ar.results()[0].fact(),
        &HtmlSemanticFact::SelectSelectedIndex(HtmlSemanticValue::Known(None))
    );
    assert_eq!(
        br.results()[0].fact(),
        &HtmlSemanticFact::SelectSelectedIndex(HtmlSemanticValue::Known(Some(0)))
    );
    assert_eq!(ar.capture(), br.capture());
    assert_ne!(ar.context(), br.context());
    assert_eq!(ar.host_revision(), c.host_facts_revision());
}

#[test]
fn all_actual_work_capacity_and_byte_limits_have_exact_and_one_short_boundaries() {
    let (c, s, _) = simple(3);
    let cx = context(&c, &[]);
    let req = request(&s, HtmlSemanticQuery::SelectOptions);
    let baseline = session(&c, &cx, limits(), HtmlSemanticCancellation::new())
        .query(std::slice::from_ref(&req))
        .unwrap()
        .usage();
    let dimensions = [
        (HtmlSemanticResource::Work, baseline.work()),
        (
            HtmlSemanticResource::OutputRecords,
            baseline.output_records(),
        ),
        (
            HtmlSemanticResource::ScratchRecords,
            baseline.peak_scratch_records(),
        ),
        (
            HtmlSemanticResource::NestedOutputRecords,
            baseline.nested_output_records(),
        ),
        (
            HtmlSemanticResource::AllocatedBytes,
            baseline.allocated_bytes(),
        ),
    ];
    for (resource, exact) in dimensions {
        assert!(exact > 0);
        for cap in [exact, exact - 1] {
            let l = HtmlSemanticLimits::new(
                if resource == HtmlSemanticResource::Work {
                    cap
                } else {
                    limits().work()
                },
                if resource == HtmlSemanticResource::OutputRecords {
                    cap
                } else {
                    100
                },
                if resource == HtmlSemanticResource::ScratchRecords {
                    cap
                } else {
                    1000
                },
                0,
                if resource == HtmlSemanticResource::AllocatedBytes {
                    cap
                } else {
                    1000000
                },
                if resource == HtmlSemanticResource::NestedOutputRecords {
                    cap
                } else {
                    1000
                },
            );
            let mut ss = session(&c, &cx, l, HtmlSemanticCancellation::new());
            let result = ss.query(std::slice::from_ref(&req));
            if cap == exact {
                assert!(result.is_ok(), "{resource:?}: {result:?}");
            } else {
                let failure = result.unwrap_err();
                assert!(
                    matches!(failure.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r==resource),
                    "{resource:?}: {failure:?}"
                );
                assert_eq!(ss.query(&[]).unwrap_err(), failure);
            }
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}
#[test]
fn known_empty_needs_no_scratch_nested_output_or_integer_budget() {
    let (c, s, _) = simple(0);
    let cx = context(&c, &[]);
    let report = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(1000000, 1, 0, 0, 100000, 0),
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&s, HtmlSemanticQuery::SelectedOptions)])
    .unwrap();
    assert!(options(report.results()[0].fact()).is_empty());
    assert_eq!(report.usage().scratch_records(), 0);
    assert_eq!(report.usage().nested_output_records(), 0);
}
#[test]
fn late_batch_limit_exhaustion_discards_prefix_and_retains_exact_failure_qualification() {
    let (c, s, _) = simple(2);
    let cx = context(&c, &[]);
    let req = request(&s, HtmlSemanticQuery::SelectOptions);
    let mut ss = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(10000000, 2, 100, 0, 1000000, 3),
        HtmlSemanticCancellation::new(),
    );
    let failure = ss.query(&[req.clone(), req]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(failure.qualification().unwrap().context(), cx.identity());
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
}
#[test]
fn cancellation_is_sticky_and_foreign_capture_ids_are_rejected() {
    let (c, s, _) = simple(1);
    let cx = context(&c, &[]);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), cancel);
    let failure = ss
        .query(&[request(&s, HtmlSemanticQuery::SelectOptions)])
        .unwrap_err();
    assert_eq!(ss.query(&[]).unwrap_err(), failure);
    let (_, other, _) = simple(1);
    let failure = session(&c, &cx, limits(), HtmlSemanticCancellation::new())
        .query(&[request(&other, HtmlSemanticQuery::OptionIndex)])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(_)
    ));
}

#[test]
fn optgroup_ancestor_bit_survives_intervening_ordinary_and_foreign_wrappers() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select");
    let group = html(&mut b, &d, "optgroup");
    let wrapper = element(&mut b, &d, Namespace::svg(), "optgroup", &[]);
    let nested = html(&mut b, &d, "optgroup");
    let excluded = html(&mut b, &d, "option");
    let included = html(&mut b, &d, "option");
    children(&mut b, &nested, std::slice::from_ref(&excluded));
    children(&mut b, &wrapper, &[nested, included.clone()]);
    children(&mut b, &group, std::slice::from_ref(&wrapper));
    children(&mut b, &s, std::slice::from_ref(&group));
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    assert_eq!(
        options(&query(
            &c,
            &cx,
            &c.subject_id(&s).unwrap(),
            HtmlSemanticQuery::SelectOptions
        )),
        [c.subject_id(&included).unwrap()]
    );
    assert_eq!(
        query(
            &c,
            &cx,
            &c.subject_id(&excluded).unwrap(),
            HtmlSemanticQuery::OptionIndex
        ),
        HtmlSemanticFact::OptionIndex(HtmlSemanticValue::Known(0))
    );
}
#[test]
fn borrowed_deep_traversal_is_iterative_and_scratch_is_released_on_missing_and_stop() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select");
    let wrappers: Vec<_> = (0..40).map(|_| html(&mut b, &d, "div")).collect();
    let o = html(&mut b, &d, "option");
    children(&mut b, &s, std::slice::from_ref(&wrappers[0]));
    for pair in wrappers.windows(2) {
        children(&mut b, &pair[0], std::slice::from_ref(&pair[1]));
    }
    children(&mut b, wrappers.last().unwrap(), std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    let s = c.subject_id(&s).unwrap();
    let mut ss = session(&c, &cx, limits(), HtmlSemanticCancellation::new());
    let report = ss
        .query(&[request(&s, HtmlSemanticQuery::SelectOptions)])
        .unwrap();
    assert_eq!(
        options(report.results()[0].fact()),
        [c.subject_id(&o).unwrap()]
    );
    assert!(report.usage().peak_scratch_records() >= 41);
    assert_eq!(ss.usage().scratch_records(), 0);
    let mut stopped = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(10000000, 1, 40, 0, 1000000, 1000),
        HtmlSemanticCancellation::new(),
    );
    assert!(
        stopped
            .query(&[request(&s, HtmlSemanticQuery::SelectOptions)])
            .is_err()
    );
    assert_eq!(stopped.usage().scratch_records(), 0);
    let mut missing = session(&c, &cx, limits(), HtmlSemanticCancellation::new());
    let report = missing
        .query(&[request(&s, HtmlSemanticQuery::SelectedOptions)])
        .unwrap();
    assert!(matches!(
        report.results()[0].fact(),
        HtmlSemanticFact::SelectedOptions(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::ControlState { .. }
        ))
    ));
    assert_eq!(missing.usage().scratch_records(), 0);
}
#[test]
fn scalar_index_and_all_false_filter_do_not_reserve_nested_collection_output() {
    let (c, s, os) = simple(2);
    let false_cx = context(
        &c,
        &os.iter()
            .map(|id| (id.clone(), Some(false)))
            .collect::<Vec<_>>(),
    );
    let l = HtmlSemanticLimits::new(10000000, 2, 100, 0, 1000000, 0);
    let report = session(&c, &false_cx, l, HtmlSemanticCancellation::new())
        .query(&[
            request(&s, HtmlSemanticQuery::SelectedOptions),
            request(&s, HtmlSemanticQuery::SelectSelectedIndex),
        ])
        .unwrap();
    assert!(options(report.results()[0].fact()).is_empty());
    assert_eq!(report.usage().nested_output_records(), 0);
    let true_cx = context(&c, &[(os[0].clone(), Some(true))]);
    assert_eq!(
        session(&c, &true_cx, l, HtmlSemanticCancellation::new())
            .query(&[request(&s, HtmlSemanticQuery::SelectSelectedIndex)])
            .unwrap()
            .results()[0]
            .fact(),
        &HtmlSemanticFact::SelectSelectedIndex(HtmlSemanticValue::Known(Some(0)))
    );
}
#[test]
fn text_is_not_an_option_and_plain_fragment_parent_does_not_cross_to_owner_document() {
    let (mut b, d) = builder();
    let s = html(&mut b, &d, "select");
    let text = b
        .add_text(&d, DocumentString::from_utf16(vec![0xd800, 0, 0x6f]))
        .unwrap();
    let o = html(&mut b, &d, "option");
    let fragment = b.add_fragment(&d).unwrap();
    let isolated = html(&mut b, &d, "option");
    children(&mut b, &fragment, std::slice::from_ref(&isolated));
    children(&mut b, &s, &[text.clone(), o.clone()]);
    children(&mut b, &d, std::slice::from_ref(&s));
    let c = b.finish().unwrap();
    let cx = context(&c, &[]);
    assert_eq!(
        options(&query(
            &c,
            &cx,
            &c.subject_id(&s).unwrap(),
            HtmlSemanticQuery::SelectOptions
        )),
        [c.subject_id(&o).unwrap()]
    );
    assert_eq!(
        query(
            &c,
            &cx,
            &c.subject_id(&isolated).unwrap(),
            HtmlSemanticQuery::OptionIndex
        ),
        HtmlSemanticFact::OptionIndex(HtmlSemanticValue::Known(0))
    );
    assert_eq!(
        query(
            &c,
            &cx,
            &c.subject_id(&text).unwrap(),
            HtmlSemanticQuery::OptionIndex
        ),
        HtmlSemanticFact::OptionIndex(HtmlSemanticValue::Inapplicable)
    );
}
