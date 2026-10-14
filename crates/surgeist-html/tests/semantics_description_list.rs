#[path = "common/builder.rs"]
mod shared_builder;
use shared_builder::builder;
#[path = "common/context.rs"]
mod shared_context;
use shared_context::context;
use surgeist_html::*;

// Independent literal group expectations from HTML 877d §4.4.9:
// references/html-language-elements-text-links--877d6146487f--8833a85fd242.md
// #the-dl-element and #process-dt-or-dd. Neither malformed groups nor div
// boundaries are normalized into a conforming-content model.

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: QualifiedName,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        name,
        OrderedAttributes::new([]).unwrap(),
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
    element(b, d, QualifiedName::new(Namespace::html(), None, name))
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    cs: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, cs).unwrap();
}

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 0, 0, 1000000, 100000)
}
fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::DescriptionListGroups)
}
fn session<'a>(
    c: &'a HtmlCapture,
    cx: &'a HtmlSemanticContext,
    l: HtmlSemanticLimits,
    cancellation: HtmlSemanticCancellation,
) -> HtmlSemanticSession<'a> {
    // The grouping algorithm consumes no attribute/text values, so zero is a
    // meaningful unrelated raw-input cap even for nodes with large contents.
    HtmlSemanticSession::new(c, cx, HtmlValueInterpreter::new(0), l, cancellation).unwrap()
}
fn report(c: &HtmlCapture, id: &HtmlCaptureSubjectId) -> HtmlSemanticReport {
    session(c, &context(c), limits(), HtmlSemanticCancellation::new())
        .query(&[request(id)])
        .unwrap()
}
fn value(r: &HtmlSemanticReport) -> &HtmlDescriptionListGroups {
    match r.results()[0].fact() {
        HtmlSemanticFact::DescriptionListGroups(HtmlSemanticValue::Known(v)) => v,
        other => panic!("{other:?}"),
    }
}
fn shape(
    r: &HtmlSemanticReport,
    ids: &[HtmlCaptureSubjectId],
    names: &[&[usize]],
    values: &[&[usize]],
) {
    let groups = value(r).groups();
    assert_eq!(groups.len(), names.len());
    assert_eq!(groups.len(), values.len());
    for (i, group) in groups.iter().enumerate() {
        let expected_names: Vec<_> = names[i].iter().map(|&i| ids[i].clone()).collect();
        let expected_values: Vec<_> = values[i].iter().map(|&i| ids[i].clone()).collect();
        assert_eq!(group.names(), expected_names);
        assert_eq!(group.values(), expected_values);
    }
}
fn sequence(names: &[&str]) -> (HtmlCapture, HtmlCaptureSubjectId, Vec<HtmlCaptureSubjectId>) {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "dl");
    let members: Vec<_> = names.iter().map(|n| html(&mut b, &d, n)).collect();
    children(&mut b, &dl, &members);
    // dl ancestry and all member children remain genuinely unavailable.
    let c = b.finish().unwrap();
    let ids = members.iter().map(|n| c.subject_id(n).unwrap()).collect();
    let id = c.subject_id(&dl).unwrap();
    (c, id, ids)
}

#[test]
fn consecutive_names_and_alternative_values_preserve_each_order() {
    let (c, dl, ids) = sequence(&["dt", "dt", "dd", "dd", "dt", "dd"]);
    shape(&report(&c, &dl), &ids, &[&[0, 1], &[4]], &[&[2, 3], &[5]]);
}
#[test]
fn leading_values_and_trailing_names_remain_source_groups_with_an_empty_side() {
    let (c, dl, ids) = sequence(&["dd", "dd", "dt", "dt"]);
    shape(&report(&c, &dl), &ids, &[&[], &[2, 3]], &[&[0, 1], &[]]);
}
#[test]
fn only_names_or_only_values_each_form_one_nonempty_group() {
    for (ns, names, values) in [
        (["dt", "dt"], vec![vec![0, 1]], vec![vec![]]),
        (["dd", "dd"], vec![vec![]], vec![vec![0, 1]]),
    ] {
        let (c, dl, ids) = sequence(&ns);
        let names: Vec<_> = names.iter().map(Vec::as_slice).collect();
        let values: Vec<_> = values.iter().map(Vec::as_slice).collect();
        shape(&report(&c, &dl), &ids, &names, &values);
    }
}
#[test]
fn empty_list_and_ignored_nodes_do_not_create_empty_groups() {
    for ns in [vec![], vec!["script", "template", "p"]] {
        let (c, dl, ids) = sequence(&ns);
        let r = report(&c, &dl);
        shape(&r, &ids, &[], &[]);
        assert_eq!(r.usage().nested_output_records(), 0);
        assert_eq!(
            r.usage().allocated_bytes(),
            r.usage().output_records() * std::mem::size_of::<HtmlSemanticResult>()
        );
    }
}
#[test]
fn group_state_spans_div_boundaries_and_empty_divs() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "dl");
    let v1 = html(&mut b, &d, "div");
    let empty = html(&mut b, &d, "div");
    let v2 = html(&mut b, &d, "div");
    let a = html(&mut b, &d, "dt");
    let x = html(&mut b, &d, "dd");
    let z = html(&mut b, &d, "dt");
    children(&mut b, &v1, std::slice::from_ref(&a));
    children(&mut b, &empty, &[]);
    children(&mut b, &v2, std::slice::from_ref(&x));
    children(&mut b, &dl, &[v1, empty, v2, z.clone()]);
    let c = b.finish().unwrap();
    let ids = [a, x, z]
        .iter()
        .map(|n| c.subject_id(n).unwrap())
        .collect::<Vec<_>>();
    shape(
        &report(&c, &c.subject_id(&dl).unwrap()),
        &ids,
        &[&[0], &[2]],
        &[&[1], &[]],
    );
}
#[test]
fn direct_and_div_members_are_one_source_stream_in_both_directions() {
    for wrapped_name in [false, true] {
        let (mut b, d) = builder();
        let dl = html(&mut b, &d, "dl");
        let div = html(&mut b, &d, "div");
        let a = html(&mut b, &d, "dt");
        let x = html(&mut b, &d, "dd");
        if wrapped_name {
            children(&mut b, &div, std::slice::from_ref(&a));
            children(&mut b, &dl, &[div, x.clone()]);
        } else {
            children(&mut b, &div, std::slice::from_ref(&x));
            children(&mut b, &dl, &[a.clone(), div]);
        }
        let c = b.finish().unwrap();
        let ids = [a, x]
            .iter()
            .map(|n| c.subject_id(n).unwrap())
            .collect::<Vec<_>>();
        shape(
            &report(&c, &c.subject_id(&dl).unwrap()),
            &ids,
            &[&[0]],
            &[&[1]],
        );
    }
}
#[test]
fn nested_wrappers_and_non_div_children_do_not_expose_their_descendants() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "dl");
    let div = html(&mut b, &d, "div");
    let nested = html(&mut b, &d, "div");
    let p = html(&mut b, &d, "p");
    let hidden_name = html(&mut b, &d, "dt");
    let hidden_value = html(&mut b, &d, "dd");
    let real_name = html(&mut b, &d, "dt");
    let real_value = html(&mut b, &d, "dd");
    children(&mut b, &nested, std::slice::from_ref(&hidden_name));
    children(&mut b, &p, std::slice::from_ref(&hidden_value));
    children(&mut b, &div, &[nested, real_value.clone()]);
    children(&mut b, &dl, &[real_name.clone(), p, div]);
    let c = b.finish().unwrap();
    let ids = [real_name, real_value]
        .iter()
        .map(|n| c.subject_id(n).unwrap())
        .collect::<Vec<_>>();
    shape(
        &report(&c, &c.subject_id(&dl).unwrap()),
        &ids,
        &[&[0]],
        &[&[1]],
    );
}
#[test]
fn foreign_and_differently_cased_members_and_divs_are_ignored() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "dl");
    let a = html(&mut b, &d, "dt");
    let x = html(&mut b, &d, "dd");
    let mut cs = vec![a.clone()];
    for local in ["dt", "dd", "div"] {
        cs.push(element(
            &mut b,
            &d,
            QualifiedName::new(Namespace::svg(), None, local),
        ));
    }
    for local in ["DT", "DD", "DIV"] {
        cs.push(html(&mut b, &d, local));
    }
    cs.push(x.clone());
    children(&mut b, &dl, &cs);
    let c = b.finish().unwrap();
    let ids = [a, x]
        .iter()
        .map(|n| c.subject_id(n).unwrap())
        .collect::<Vec<_>>();
    shape(
        &report(&c, &c.subject_id(&dl).unwrap()),
        &ids,
        &[&[0]],
        &[&[1]],
    );
}
#[test]
fn arbitrary_prefixes_do_not_change_expanded_html_member_identity() {
    let (mut b, d) = builder();
    let dl = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("x".into()), "dl"),
    );
    let div = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("a".into()), "div"),
    );
    let a = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("z".into()), "dt"),
    );
    let x = element(
        &mut b,
        &d,
        QualifiedName::new(Namespace::html(), Some("y".into()), "dd"),
    );
    children(&mut b, &div, &[a.clone(), x.clone()]);
    children(&mut b, &dl, &[div]);
    let c = b.finish().unwrap();
    let ids = [a, x]
        .iter()
        .map(|n| c.subject_id(n).unwrap())
        .collect::<Vec<_>>();
    shape(
        &report(&c, &c.subject_id(&dl).unwrap()),
        &ids,
        &[&[0]],
        &[&[1]],
    );
}
#[test]
fn text_comments_and_script_contents_neither_split_groups_nor_supply_members() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "dl");
    let a = html(&mut b, &d, "dt");
    let x = html(&mut b, &d, "dd");
    let text = b
        .add_text(&d, DocumentString::from_utf16(vec![0, 0xd800]))
        .unwrap();
    let comment = b.add_comment(&d, "ignored".into()).unwrap();
    let script = html(&mut b, &d, "script");
    let hidden = html(&mut b, &d, "dt");
    children(&mut b, &script, &[hidden]);
    children(&mut b, &dl, &[a.clone(), text, comment, script, x.clone()]);
    let c = b.finish().unwrap();
    let ids = [a, x]
        .iter()
        .map(|n| c.subject_id(n).unwrap())
        .collect::<Vec<_>>();
    shape(
        &report(&c, &c.subject_id(&dl).unwrap()),
        &ids,
        &[&[0]],
        &[&[1]],
    );
}
#[test]
fn member_text_and_attributes_are_not_read_or_normalized_into_names() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "dl");
    let exact = DocumentString::from_utf16(vec![32, 0, 0xd800, 32]);
    let a = b
        .add_element(
            &d,
            KnownHtmlElement::Dt.qualified_name(),
            OrderedAttributes::new([Attribute::new(
                KnownHtmlAttribute::Title.qualified_name(),
                exact.clone(),
            )])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let text = b.add_text(&d, exact.clone()).unwrap();
    let x = html(&mut b, &d, "dd");
    children(&mut b, &a, std::slice::from_ref(&text));
    children(&mut b, &dl, &[a.clone(), x.clone()]);
    let c = b.finish().unwrap();
    let ids = [a, x]
        .iter()
        .map(|n| c.subject_id(n).unwrap())
        .collect::<Vec<_>>();
    let r = report(&c, &c.subject_id(&dl).unwrap());
    shape(&r, &ids, &[&[0]], &[&[1]]);
    assert_eq!(r.attribute_input_limit(), 0);
    let HtmlCapturedNodeKind::Text(value) = c.node(&c.subject_id(&text).unwrap()).unwrap().kind()
    else {
        panic!()
    };
    assert_eq!(value, &exact);
}
#[test]
fn separate_template_and_shadow_contents_are_not_grouping_children() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let dl = html(&mut b, &d, "dl");
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
        let hidden = html(&mut b, &d, "dt");
        children(&mut b, &f, &[hidden]);
        children(&mut b, &host, &[]);
        children(&mut b, &dl, &[host]);
        let c = b.finish().unwrap();
        assert!(
            value(&report(&c, &c.subject_id(&dl).unwrap()))
                .groups()
                .is_empty()
        );
    }
}
#[test]
fn unknown_ancestry_and_member_children_are_irrelevant_to_complete_groups() {
    let (c, dl, ids) = sequence(&["dt", "dd"]);
    assert_eq!(
        c.node(&dl).unwrap().parent(),
        HtmlCapturedParent::Unavailable
    );
    assert!(matches!(
        c.node(&ids[0]).unwrap().children(),
        HtmlCapturedChildren::Unavailable
    ));
    shape(&report(&c, &dl), &ids, &[&[0]], &[&[1]]);
}
#[test]
fn unavailable_dl_children_remain_missing_and_do_not_become_empty() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "dl");
    let c = b.finish().unwrap();
    let dl = c.subject_id(&dl).unwrap();
    assert_eq!(
        report(&c, &dl).results()[0].fact(),
        &HtmlSemanticFact::DescriptionListGroups(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(dl.clone())
        ))
    );
}
#[test]
fn unavailable_direct_div_children_discard_prior_partial_membership() {
    let (c, dl, ids) = sequence(&["dt", "dd", "div", "dt"]);
    assert_eq!(
        report(&c, &dl).results()[0].fact(),
        &HtmlSemanticFact::DescriptionListGroups(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(ids[2].clone())
        ))
    );
}
#[test]
fn unavailable_children_of_ignored_nodes_do_not_influence_the_result() {
    let (c, dl, ids) = sequence(&["dt", "p", "template", "dd"]);
    shape(&report(&c, &dl), &ids, &[&[0]], &[&[3]]);
}

#[test]
fn unavailable_nested_div_children_are_not_an_influencing_second_level() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "dl");
    let direct = html(&mut b, &d, "div");
    let nested = html(&mut b, &d, "div");
    let a = html(&mut b, &d, "dt");
    let x = html(&mut b, &d, "dd");
    children(&mut b, &direct, &[a.clone(), nested, x.clone()]);
    children(&mut b, &dl, &[direct]);
    let c = b.finish().unwrap();
    let ids = [a, x]
        .iter()
        .map(|n| c.subject_id(n).unwrap())
        .collect::<Vec<_>>();
    shape(
        &report(&c, &c.subject_id(&dl).unwrap()),
        &ids,
        &[&[0]],
        &[&[1]],
    );
}
#[test]
fn a_dl_in_a_plain_fragment_does_not_need_document_membership() {
    let (mut b, d) = builder();
    let f = b.add_fragment(&d).unwrap();
    let dl = html(&mut b, &d, "dl");
    let a = html(&mut b, &d, "dt");
    children(&mut b, &dl, std::slice::from_ref(&a));
    children(&mut b, &f, std::slice::from_ref(&dl));
    let c = b.finish().unwrap();
    shape(
        &report(&c, &c.subject_id(&dl).unwrap()),
        &[c.subject_id(&a).unwrap()],
        &[&[0]],
        &[&[]],
    );
}
#[test]
fn only_exact_html_dl_is_applicable_without_irrelevant_topology() {
    let (mut b, d) = builder();
    let p = html(&mut b, &d, "p");
    let uppercase = html(&mut b, &d, "DL");
    let foreign = element(&mut b, &d, QualifiedName::new(Namespace::svg(), None, "dl"));
    let text = b.add_text(&d, "x".into()).unwrap();
    let f = b.add_fragment(&d).unwrap();
    let c = b.finish().unwrap();
    for n in [d, p, uppercase, text, f] {
        assert_eq!(
            report(&c, &c.subject_id(&n).unwrap()).results()[0].fact(),
            &HtmlSemanticFact::DescriptionListGroups(HtmlSemanticValue::Inapplicable)
        );
    }
    assert_eq!(
        report(&c, &c.subject_id(&foreign).unwrap()).results()[0].fact(),
        &HtmlSemanticFact::DescriptionListGroups(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    );
}
fn capped(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(
        if resource == HtmlSemanticResource::Work {
            cap
        } else {
            20000000
        },
        if resource == HtmlSemanticResource::OutputRecords {
            cap
        } else {
            100
        },
        0,
        0,
        if resource == HtmlSemanticResource::AllocatedBytes {
            cap
        } else {
            1000000
        },
        if resource == HtmlSemanticResource::NestedOutputRecords {
            cap
        } else {
            100000
        },
    )
}
#[test]
fn exact_and_one_short_work_and_actual_vector_caps_are_atomic_and_sticky() {
    let (c, dl, ids) = sequence(&["dd", "dt", "dd", "dt"]);
    let cx = context(&c);
    let observed = report(&c, &dl).usage();
    assert_eq!(observed.scratch_records(), 0);
    assert_eq!(observed.peak_scratch_records(), 0);
    for (resource, exact) in [
        (HtmlSemanticResource::Work, observed.work()),
        (
            HtmlSemanticResource::OutputRecords,
            observed.output_records(),
        ),
        (
            HtmlSemanticResource::NestedOutputRecords,
            observed.nested_output_records(),
        ),
        (
            HtmlSemanticResource::AllocatedBytes,
            observed.allocated_bytes(),
        ),
    ] {
        assert!(exact > 0);
        for cap in [exact, exact - 1] {
            let mut s = session(
                &c,
                &cx,
                capped(resource, cap),
                HtmlSemanticCancellation::new(),
            );
            let result = s.query(&[request(&dl)]);
            if cap == exact {
                shape(
                    &result.unwrap(),
                    &ids,
                    &[&[], &[1], &[3]],
                    &[&[0], &[2], &[]],
                );
            } else {
                let failure = result.unwrap_err();
                assert!(
                    matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded{resource:r,..} if *r == resource),
                    "{failure:?}"
                );
                assert_eq!(s.query(&[]).unwrap_err(), failure);
                assert_eq!(s.usage(), failure.usage());
                assert_eq!(s.usage().scratch_records(), 0);
                assert_eq!(failure.qualification().unwrap().capture(), c.identity());
            }
        }
    }
}
#[test]
fn cancellation_before_entry_or_after_an_earlier_query_preserves_the_first_stop() {
    let (c, dl, _) = sequence(&["dt", "dd"]);
    let cx = context(&c);
    for before in [false, true] {
        let cancel = HtmlSemanticCancellation::new();
        let mut s = session(&c, &cx, limits(), cancel.clone());
        if !before {
            s.query(&[request(&dl)]).unwrap();
        }
        let previous = s.usage();
        cancel.cancel();
        let fail = s.query(&[request(&dl)]).unwrap_err();
        assert_eq!(fail.kind(), &HtmlSemanticFailureKind::Cancelled);
        assert_eq!(fail.usage(), previous);
        assert_eq!(s.query(&[]).unwrap_err(), fail);
    }
}

#[test]
fn every_work_cut_through_div_iteration_and_group_transfer_is_terminal() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "dl");
    let div = html(&mut b, &d, "div");
    let x = html(&mut b, &d, "dd");
    let a = html(&mut b, &d, "dt");
    let z = html(&mut b, &d, "dd");
    children(&mut b, &div, &[x, a, z]);
    children(&mut b, &dl, &[div]);
    let c = b.finish().unwrap();
    let dl = c.subject_id(&dl).unwrap();
    let cx = context(&c);
    let work = report(&c, &dl).usage().work();
    for cut in 0..work {
        let mut s = session(
            &c,
            &cx,
            capped(HtmlSemanticResource::Work, cut),
            HtmlSemanticCancellation::new(),
        );
        let failure = s.query(&[request(&dl)]).unwrap_err();
        assert!(matches!(
            failure.kind(),
            HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                ..
            }
        ));
        assert_eq!(s.query(&[]).unwrap_err(), failure);
        assert_eq!(s.usage(), failure.usage());
        assert_eq!(s.usage().scratch_records(), 0);
    }
}
#[test]
fn late_foreign_subject_discards_the_entire_batch_and_retains_qualification() {
    let (c, dl, _) = sequence(&["dt", "dd"]);
    let (other, foreign, _) = sequence(&[]);
    assert_ne!(c.identity(), other.identity());
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), HtmlSemanticCancellation::new());
    let failure = s.query(&[request(&dl), request(&foreign)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(_)
    ));
    assert!(failure.usage().nested_output_records() > 0);
    assert_eq!(failure.qualification().unwrap().capture(), c.identity());
    assert_eq!(s.query(&[request(&dl)]).unwrap_err(), failure);
}
#[test]
fn repeated_requests_retain_order_and_old_results_survive_later_missing_queries() {
    let (mut b, d) = builder();
    let good = html(&mut b, &d, "dl");
    let missing = html(&mut b, &d, "dl");
    let a = html(&mut b, &d, "dt");
    let x = html(&mut b, &d, "dd");
    children(&mut b, &good, &[a.clone(), x.clone()]);
    let c = b.finish().unwrap();
    let good = c.subject_id(&good).unwrap();
    let missing = c.subject_id(&missing).unwrap();
    let ids = [a, x]
        .iter()
        .map(|n| c.subject_id(n).unwrap())
        .collect::<Vec<_>>();
    let cx = context(&c);
    let mut s = session(&c, &cx, limits(), HtmlSemanticCancellation::new());
    let old = s.query(&[request(&good)]).unwrap();
    let r = s
        .query(&[request(&missing), request(&good), request(&missing)])
        .unwrap();
    assert_eq!(r.results().len(), 3);
    assert_eq!(r.results()[0].request(), &request(&missing));
    assert_eq!(r.results()[1].request(), &request(&good));
    assert_eq!(r.results()[2].fact(), r.results()[0].fact());
    assert_eq!(
        r.results()[0].fact(),
        &HtmlSemanticFact::DescriptionListGroups(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(missing.clone())
        ))
    );
    assert_eq!(r.results()[1].fact(), old.results()[0].fact());
    assert!(s.failure().is_none());
    s.query(&[request(&missing)]).unwrap();
    shape(&old, &ids, &[&[0]], &[&[1]]);
}
#[test]
fn source_clause_and_result_qualification_identify_the_actual_pinned_algorithm() {
    let (c, dl, _) = sequence(&[]);
    let r = report(&c, &dl);
    assert_eq!(
        r.source_revision(),
        "877d6146487f15b15bc8dd0e3c625063a37b456a"
    );
    assert_eq!(r.capture(), c.identity());
    assert_eq!(r.host_revision(), c.host_facts_revision());
    assert_eq!(r.attribute_input_limit(), 0);
    assert_eq!(
        HtmlSemanticQuery::DescriptionListGroups.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-dl-element"
    );
}
