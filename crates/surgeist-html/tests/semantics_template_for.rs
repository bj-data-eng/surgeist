use surgeist_html::*;

fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(1000, 100000, 1000, 1000000, 1000, 10, 10),
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
fn attr(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, "for"), value)
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: Vec<Attribute>,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(Namespace::html(), None, local),
        OrderedAttributes::new(attrs).unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn pi(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    target: &str,
    data: impl Into<DocumentString>,
) -> HtmlCaptureBuilderSubject {
    b.add_processing_instruction(d, target.into(), data.into())
        .unwrap()
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    list: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, list).unwrap();
}
fn context(c: &HtmlCapture) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 1000, 0, 1000000, 10000)
}
fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::TemplateForReference)
}
fn run(c: &HtmlCapture, id: &HtmlCaptureSubjectId) -> HtmlSemanticReport {
    HtmlSemanticSession::new(
        c,
        &context(c),
        HtmlValueInterpreter::new(100000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[request(id)])
    .unwrap()
}
fn known(r: &HtmlSemanticReport) -> &HtmlTemplateForReport {
    match r.results()[0].fact() {
        HtmlSemanticFact::TemplateForReference(HtmlSemanticValue::Known(r)) => r,
        other => panic!("{other:?}"),
    }
}
fn one(
    value: impl Into<DocumentString>,
    target: &str,
    data: impl Into<DocumentString>,
) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let p = pi(&mut b, &d, target, data);
    let t = element(&mut b, &d, "template", vec![attr(value)]);
    children(&mut b, &t, &[]);
    children(&mut b, &d, &[p.clone(), t.clone()]);
    let c = b.finish().unwrap();
    let tid = c.subject_id(&t).unwrap();
    let pid = c.subject_id(&p).unwrap();
    (c, tid, pid)
}
#[test]
fn literal_marker_selects_itself_and_preserves_raw_without_trimming() {
    let (c, t, p) = one(" X ", "marker", "name=' X '");
    let r = run(&c, &t);
    assert_eq!(
        known(&r).raw_attribute().unwrap().value().code_units(),
        [32, 88, 32]
    );
    assert_eq!(
        known(&r).name_relation(),
        &HtmlTemplateForNameRelation::Match(p.clone())
    );
    assert_eq!(known(&r).selection(), &HtmlTemplateForSelection::Marker(p));
    assert_eq!(r.usage().scratch_records(), 0);
}
#[test]
fn full_xml_map_and_references_have_independent_literal_matches() {
    for (value, data) in [
        ("X", "name = 'X'"),
        ("X", " \t\r\nname=\"&#88;\" x='ok' \r\n"),
        ("X", "x='ok' name='&#x58;'"),
        ("&<>\"'", "name='&amp;&lt;&gt;&quot;&apos;'"),
        ("𝄞", "name='&#x1D11E;'"),
        ("X", "name='&#0000000000000000000000000000000000000088;'"),
        ("X", "\u{10000}='v' name='X'"),
        ("X", "a='A' A='B' name='X'"),
        ("\u{10ffff}", "name='&#x10FFFF;'"),
        ("X", "\u{effff}='v' name='X'"),
        ("a'b", "name=\"a'b\""),
    ] {
        let (c, t, p) = one(value, "marker", data);
        let r = run(&c, &t);
        assert_eq!(
            known(&r).name_relation(),
            &HtmlTemplateForNameRelation::Match(p.clone()),
            "{data}"
        );
        assert_eq!(
            known(&r).selection(),
            &HtmlTemplateForSelection::Marker(p),
            "{data}"
        );
    }
}
#[test]
fn any_invalid_suffix_or_duplicate_invalidates_the_whole_map() {
    for data in [
        "name='X' x='a' x='b'",
        "name='X' garbage",
        "name='X'x='a'",
        "name='X'\u{c}x='a'",
        "name='X' x='&unknown;'",
        "name='X' x='&#0;'",
        "name='X' x='&#xD800;'",
        "name='X' x='&#1114112;'",
        "name='X' x='&#999999999999999999999999;'",
        "name='X' x='&#xFFFFFFFFFFFFFFFF;'",
        "name='X' x='<'",
        "name='X' x='unterminated",
        "name='X' 0x='a'",
    ] {
        let (c, t, _) = one("X", "marker", data);
        let r = run(&c, &t);
        assert_eq!(
            known(&r).name_relation(),
            &HtmlTemplateForNameRelation::NoMatch,
            "{data}"
        );
        assert_eq!(
            known(&r).selection(),
            &HtmlTemplateForSelection::NoMatch,
            "{data}"
        );
    }
}
#[test]
fn invalid_original_utf16_and_name_case_do_not_match() {
    for data in [
        DocumentString::from_utf16(vec![110, 97, 109, 101, 61, 39, 88, 39, 32, 0xd800]),
        "NAME='X'".into(),
        "name='x'".into(),
    ] {
        let (c, t, _) = one("X", "marker", data);
        assert_eq!(
            known(&run(&c, &t)).selection(),
            &HtmlTemplateForSelection::NoMatch
        );
    }
}
#[test]
fn first_marker_wins_over_later_start_and_later_missing_subtree() {
    let (mut b, d) = builder();
    let a = pi(&mut b, &d, "marker", "name='X'");
    let start = pi(&mut b, &d, "start", "name='X'");
    let later = element(&mut b, &d, "div", vec![]);
    let t = element(&mut b, &d, "template", vec![attr("X")]);
    children(&mut b, &d, &[a.clone(), start, later, t.clone()]);
    let c = b.finish().unwrap();
    let r = run(&c, &c.subject_id(&t).unwrap());
    assert_eq!(
        known(&r).selection(),
        &HtmlTemplateForSelection::Marker(c.subject_id(&a).unwrap())
    );
}
#[test]
fn start_counts_every_sibling_target_ignoring_names_and_descendants() {
    let (mut b, d) = builder();
    let start = pi(&mut b, &d, "start", "name='X'");
    let nested = pi(&mut b, &d, "start", "name='Y'");
    let wrapper = element(&mut b, &d, "div", vec![]);
    let descendant = pi(&mut b, &d, "end", "");
    children(&mut b, &wrapper, &[descendant]);
    let nested_end = pi(&mut b, &d, "end", "malformed");
    let end = pi(&mut b, &d, "end", "name='other'");
    let t = element(&mut b, &d, "template", vec![attr("X")]);
    children(
        &mut b,
        &d,
        &[
            start.clone(),
            nested,
            wrapper,
            nested_end,
            end.clone(),
            t.clone(),
        ],
    );
    let c = b.finish().unwrap();
    let r = run(&c, &c.subject_id(&t).unwrap());
    assert_eq!(
        known(&r).selection(),
        &HtmlTemplateForSelection::Start {
            start: c.subject_id(&start).unwrap(),
            end: Some(c.subject_id(&end).unwrap())
        }
    );
}
#[test]
fn complete_sibling_exhaustion_keeps_start_with_no_end() {
    let (c, t, p) = one("X", "start", "name='X'");
    assert_eq!(
        known(&run(&c, &t)).selection(),
        &HtmlTemplateForSelection::Start {
            start: p,
            end: None
        }
    );
}
#[test]
fn target_case_and_unrelated_target_are_not_markers() {
    for target in ["Marker", "START", "other", "end"] {
        let (c, t, _) = one("X", target, "name='X'");
        assert_eq!(
            known(&run(&c, &t)).selection(),
            &HtmlTemplateForSelection::NoMatch
        );
    }
}
#[test]
fn earlier_unknown_descendants_preserve_later_author_witness_only() {
    for target in ["marker", "start"] {
        let (mut b, d) = builder();
        let a = element(&mut b, &d, "div", vec![]);
        let p = pi(&mut b, &d, target, "name='X'");
        let t = element(&mut b, &d, "template", vec![attr("X")]);
        children(&mut b, &d, &[a.clone(), p.clone(), t.clone()]);
        let c = b.finish().unwrap();
        let r = run(&c, &c.subject_id(&t).unwrap());
        assert_eq!(
            known(&r).name_relation(),
            &HtmlTemplateForNameRelation::Match(c.subject_id(&p).unwrap())
        );
        assert_eq!(
            known(&r).selection(),
            &HtmlTemplateForSelection::Missing(HtmlSemanticMissing::OrderedChildren(
                c.subject_id(&a).unwrap()
            ))
        );
        assert_eq!(r.usage().scratch_records(), 0);
    }
}
#[test]
fn incomplete_coverage_without_known_witness_is_missing_not_absence() {
    let (mut b, d) = builder();
    let a = element(&mut b, &d, "div", vec![]);
    let p = pi(&mut b, &d, "marker", "name='Y'");
    let t = element(&mut b, &d, "template", vec![attr("X")]);
    children(&mut b, &d, &[a.clone(), p, t.clone()]);
    let c = b.finish().unwrap();
    let r = run(&c, &c.subject_id(&t).unwrap());
    let missing = HtmlSemanticMissing::OrderedChildren(c.subject_id(&a).unwrap());
    assert_eq!(
        known(&r).name_relation(),
        &HtmlTemplateForNameRelation::Missing(missing.clone())
    );
    assert_eq!(
        known(&r).selection(),
        &HtmlTemplateForSelection::Missing(missing)
    );
}

#[test]
fn sufficient_author_witness_after_unknown_stops_before_later_unrelated_raw_input() {
    let (mut b, d) = builder();
    let unknown = element(&mut b, &d, "div", vec![]);
    let witness = pi(&mut b, &d, "marker", "name='X'");
    let later = pi(&mut b, &d, "marker", "z".repeat(256));
    let t = element(&mut b, &d, "template", vec![attr("X")]);
    children(
        &mut b,
        &d,
        &[unknown.clone(), witness.clone(), later, t.clone()],
    );
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(16),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let r = s.query(&[request(&c.subject_id(&t).unwrap())]).unwrap();
    assert_eq!(
        known(&r).name_relation(),
        &HtmlTemplateForNameRelation::Match(c.subject_id(&witness).unwrap())
    );
    assert_eq!(
        known(&r).selection(),
        &HtmlTemplateForSelection::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&unknown).unwrap()
        ))
    );
    assert_eq!(s.usage().scratch_records(), 0);
    assert!(s.failure().is_none());
}
#[test]
fn empty_for_is_ineligible_but_can_have_an_author_witness() {
    let (c, t, p) = one("", "marker", "name=''");
    let r = run(&c, &t);
    assert_eq!(
        known(&r).name_relation(),
        &HtmlTemplateForNameRelation::Match(p)
    );
    assert_eq!(known(&r).selection(), &HtmlTemplateForSelection::EmptyName);
}
#[test]
fn empty_for_has_missing_author_coverage_without_losing_ineligibility() {
    let (mut b, d) = builder();
    let t = element(&mut b, &d, "template", vec![attr("")]);
    let c = b.finish().unwrap();
    let r = run(&c, &c.subject_id(&t).unwrap());
    assert!(matches!(
        known(&r).name_relation(),
        HtmlTemplateForNameRelation::Missing(_)
    ));
    assert_eq!(known(&r).selection(), &HtmlTemplateForSelection::EmptyName);
}
#[test]
fn absent_for_skips_unavailable_parent_and_children() {
    let (mut b, d) = builder();
    let t = element(&mut b, &d, "template", vec![]);
    let c = b.finish().unwrap();
    let r = run(&c, &c.subject_id(&t).unwrap());
    assert_eq!(
        known(&r).name_relation(),
        &HtmlTemplateForNameRelation::NotSpecified
    );
    assert_eq!(
        known(&r).selection(),
        &HtmlTemplateForSelection::NotSpecified
    );
    assert_eq!(r.usage().peak_scratch_records(), 0);
}
#[test]
fn established_shared_ancestor_proves_author_despite_unavailable_ultimate_parent() {
    let (mut b, d) = builder();
    let ancestor = element(&mut b, &d, "div", vec![]);
    let p = pi(&mut b, &d, "marker", "name='X'");
    let t = element(&mut b, &d, "template", vec![attr("X")]);
    children(&mut b, &ancestor, &[p.clone(), t.clone()]);
    let c = b.finish().unwrap();
    let r = run(&c, &c.subject_id(&t).unwrap());
    assert_eq!(
        known(&r).name_relation(),
        &HtmlTemplateForNameRelation::Match(c.subject_id(&p).unwrap())
    );
    assert_eq!(
        known(&r).selection(),
        &HtmlTemplateForSelection::Missing(HtmlSemanticMissing::Parent(
            c.subject_id(&ancestor).unwrap()
        ))
    );
}
#[test]
fn unestablished_connection_does_not_turn_capture_membership_into_same_tree() {
    let (mut b, d) = builder();
    let _p = pi(&mut b, &d, "marker", "name='X'");
    let t = element(&mut b, &d, "template", vec![attr("X")]);
    children(&mut b, &t, &[]);
    let c = b.finish().unwrap();
    let r = run(&c, &c.subject_id(&t).unwrap());
    assert!(matches!(
        known(&r).name_relation(),
        HtmlTemplateForNameRelation::Missing(_)
    ));
}
#[test]
fn separate_fragment_marker_does_not_match_document_template() {
    let (mut b, d) = builder();
    let f = b.add_fragment(&d).unwrap();
    let p = pi(&mut b, &d, "marker", "name='X'");
    children(&mut b, &f, &[p]);
    let t = element(&mut b, &d, "template", vec![attr("X")]);
    children(&mut b, &t, &[]);
    children(&mut b, &d, std::slice::from_ref(&t));
    let c = b.finish().unwrap();
    assert_eq!(
        known(&run(&c, &c.subject_id(&t).unwrap())).selection(),
        &HtmlTemplateForSelection::NoMatch
    );
}
#[test]
fn raw_input_cap_precedes_wrong_role_and_foreign_subjects_are_distinct() {
    let (mut b, d) = builder();
    let div = element(&mut b, &d, "div", vec![attr("long")]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(3),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let first = s
        .query(&[request(&c.subject_id(&div).unwrap())])
        .unwrap_err();
    assert!(matches!(
        first.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    let (mut b, d) = builder();
    let foreign = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::from_uri("urn:x"), None, "template"),
            OrderedAttributes::new(vec![attr("X")]).unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let c = b.finish().unwrap();
    let r = run(&c, &c.subject_id(&foreign).unwrap());
    assert!(matches!(
        r.results()[0].fact(),
        HtmlSemanticFact::TemplateForReference(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    ));
}
#[test]
fn exact_expanded_for_name_ignores_namespaced_and_uppercase_attributes() {
    let (mut b, d) = builder();
    let attrs = vec![
        Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:x"), None, "for"),
            "X",
        ),
        Attribute::new(QualifiedName::new(Namespace::none(), None, "FOR"), "X"),
    ];
    let t = element(&mut b, &d, "template", attrs);
    let c = b.finish().unwrap();
    assert_eq!(
        known(&run(&c, &c.subject_id(&t).unwrap())).selection(),
        &HtmlTemplateForSelection::NotSpecified
    );
}
#[test]
fn each_original_pi_slot_is_admitted_before_grammar_or_target_selection() {
    for (target, data) in [("marker", "name='X'"), ("muchlongertarget", "")] {
        let (c, t, _) = one("X", target, data);
        let cx = context(&c);
        let mut s = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(5),
            limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        assert!(matches!(
            s.query(&[request(&t)]).unwrap_err().kind(),
            HtmlSemanticFailureKind::Attribute(_)
        ));
        assert_eq!(s.usage().scratch_records(), 0);
    }
}
#[test]
fn exact_actual_scratch_output_and_work_limits_keep_first_stop_and_release() {
    let (c, t, _) = one("X", "marker", "name='X' x='Y'");
    let r = run(&c, &t);
    let u = r.usage();
    let exact = HtmlSemanticLimits::new(
        u.work(),
        u.output_records(),
        u.peak_scratch_records(),
        0,
        u.allocated_bytes(),
        10000,
    );
    let cx = context(&c);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        exact,
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    assert!(s.query(&[request(&t)]).is_ok());
    // Capacity characterization uses real completed usage; semantic results in
    // the other tests use independently declared source literals.
    for (l, resource) in [
        HtmlSemanticLimits::new(u.work() - 1, 100, 1000, 0, 1000000, 10000),
        HtmlSemanticLimits::new(10000000, u.output_records() - 1, 1000, 0, 1000000, 10000),
        HtmlSemanticLimits::new(
            10000000,
            100,
            u.peak_scratch_records() - 1,
            0,
            1000000,
            10000,
        ),
        HtmlSemanticLimits::new(10000000, 100, 1000, 0, u.allocated_bytes() - 1, 10000),
    ]
    .into_iter()
    .zip([
        HtmlSemanticResource::Work,
        HtmlSemanticResource::OutputRecords,
        HtmlSemanticResource::ScratchRecords,
        HtmlSemanticResource::AllocatedBytes,
    ]) {
        let mut s = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(100),
            l,
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let first = s.query(&[request(&t)]).unwrap_err();
        assert!(
            matches!(first.kind(),HtmlSemanticFailureKind::LimitExceeded{resource:actual,..} if *actual==resource)
        );
        let after = s.usage();
        assert_eq!(s.query(&[request(&t)]).unwrap_err(), first);
        assert_eq!(s.usage(), after);
        assert_eq!(s.usage().scratch_records(), 0);
    }
}
#[test]
fn cancellation_after_success_is_sticky_and_original_reports_remain_valid() {
    let (c, t, p) = one("X", "marker", "name='X'");
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        limits(),
        cancel.clone(),
    )
    .unwrap();
    let r = s.query(&[request(&t)]).unwrap();
    cancel.cancel();
    let first = s.query(&[request(&t)]).unwrap_err();
    assert_eq!(first.kind(), &HtmlSemanticFailureKind::Cancelled);
    let used = s.usage();
    assert_eq!(s.query(&[request(&t)]).unwrap_err(), first);
    assert_eq!(s.usage(), used);
    assert_eq!(known(&r).selection(), &HtmlTemplateForSelection::Marker(p));
}
#[test]
fn batch_failure_returns_no_prefix_and_foreign_subject_is_terminal() {
    let (c, t, _) = one("X", "marker", "name='X'");
    let (other, foreign, _) = one("X", "marker", "name='X'");
    let cx = context(&c);
    let mut s = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let first = s.query(&[request(&t), request(&foreign)]).unwrap_err();
    assert!(matches!(first.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.query(&[request(&t)]).unwrap_err(), first);
    drop(other);
}

#[test]
fn xml_name_and_character_upper_boundaries_are_literal_not_html_recovery() {
    for data in [
        "\u{f0000}='v' name='X'",
        "\u{b7}='v' name='X'",
        "name='&#xFFFF;'",
        "name='&#X58;'",
        "name='&#;'",
        "name='&#x;'",
        "name='&#-1;'",
        "name='&#88'",
        "",
    ] {
        let (c, t, _) = one("X", "marker", data);
        let r = run(&c, &t);
        assert_eq!(
            known(&r).name_relation(),
            &HtmlTemplateForNameRelation::NoMatch,
            "{data}"
        );
        assert_eq!(
            known(&r).selection(),
            &HtmlTemplateForSelection::NoMatch,
            "{data}"
        );
    }
}

#[test]
fn template_contents_and_shadow_associations_do_not_join_ordinary_marker_trees() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let host = element(&mut b, &d, if shadow { "div" } else { "template" }, vec![]);
        let f = if shadow {
            b.add_shadow_root(
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
                    true,
                ),
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap()
        } else {
            let f = b.add_fragment(&d).unwrap();
            b.establish_template_contents(&host, &f).unwrap();
            f
        };
        let p = pi(&mut b, &d, "marker", "name='X'");
        children(&mut b, &f, &[p]);
        children(&mut b, &host, &[]);
        let t = element(&mut b, &d, "template", vec![attr("X")]);
        children(&mut b, &t, &[]);
        children(&mut b, &d, &[host, t.clone()]);
        let c = b.finish().unwrap();
        let r = run(&c, &c.subject_id(&t).unwrap());
        assert_eq!(
            known(&r).name_relation(),
            &HtmlTemplateForNameRelation::NoMatch
        );
        assert_eq!(known(&r).selection(), &HtmlTemplateForSelection::NoMatch);
    }
}

#[test]
fn non_elements_and_wrong_html_roles_are_inapplicable_and_reports_are_capture_qualified() {
    let (mut b, d) = builder();
    let p = pi(&mut b, &d, "marker", "name='X'");
    let e = element(&mut b, &d, "div", vec![attr("X")]);
    children(&mut b, &e, &[]);
    children(&mut b, &d, &[p.clone(), e.clone()]);
    let c = b.finish().unwrap();
    for id in [&d, &p, &e] {
        let r = run(&c, &c.subject_id(id).unwrap());
        assert_eq!(r.qualification().capture(), c.identity());
        assert!(matches!(
            r.results()[0].fact(),
            HtmlSemanticFact::TemplateForReference(HtmlSemanticValue::Inapplicable)
        ));
    }
}
