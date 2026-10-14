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

fn attrs(values: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(
        values
            .iter()
            .map(|(n, v)| Attribute::new(QualifiedName::new(Namespace::none(), None, *n), *v)),
    )
    .unwrap()
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    ns: Namespace,
    name: &str,
    a: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(ns, None, name),
        a,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    name: &str,
    a: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    element(b, d, Namespace::html(), name, attrs(a))
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    items: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, items).unwrap();
}

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 1000, 0, 1000000, 100000)
}

fn request(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn report(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
    raw: usize,
) -> HtmlSemanticReport {
    session(c, cx, limits(), raw, HtmlSemanticCancellation::new())
        .query(&[request(id, q)])
        .unwrap()
}
fn source(r: &HtmlSemanticReport) -> &Option<HtmlCaptureSubjectId> {
    match r.results()[0].fact() {
        HtmlSemanticFact::InputSuggestionsSource(HtmlSemanticValue::Known(v)) => v,
        f => panic!("expected known source, got {f:?}"),
    }
}
fn collection(r: &HtmlSemanticReport) -> &HtmlOptionCollection {
    match r.results()[0].fact() {
        HtmlSemanticFact::DatalistOptions(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::DatalistSuggestionOptions(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::SelectOptions(HtmlSemanticValue::Known(v)) => v,
        f => panic!("expected known collection, got {f:?}"),
    }
}
fn associated(
    state: Option<&str>,
    list: &str,
) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut b, d) = builder();
    let mut a = vec![("list", list)];
    if let Some(state) = state {
        a.push(("type", state));
    }
    let i = html(&mut b, &d, "input", &a);
    let dl = html(&mut b, &d, "datalist", &[("id", "target")]);
    children(&mut b, &i, &[]);
    children(&mut b, &dl, &[]);
    children(&mut b, &d, &[i.clone(), dl.clone()]);
    let c = b.finish().unwrap();
    let ids = (c.subject_id(&i).unwrap(), c.subject_id(&dl).unwrap());
    (c, ids.0, ids.1)
}

#[test]
fn all_applicable_states_and_intrinsic_text_defaults_resolve_source() {
    // The per-state defining clauses, rather than the non-normative summary.
    for state in [
        None,
        Some("invalid"),
        Some("text"),
        Some("SEARCH"),
        Some("tel"),
        Some("url"),
        Some("email"),
        Some("date"),
        Some("month"),
        Some("week"),
        Some("time"),
        Some("datetime-local"),
        Some("number"),
        Some("range"),
        Some("color"),
    ] {
        let (c, i, dl) = associated(state, "target");
        let cx = context(&c);
        assert_eq!(
            source(&report(
                &c,
                &cx,
                &i,
                HtmlSemanticQuery::InputSuggestionsSource,
                100
            )),
            &Some(dl),
            "{state:?}"
        );
    }
}

#[test]
fn every_excluded_state_establishes_null_without_reading_large_list_or_topology() {
    for state in [
        "hidden", "password", "checkbox", "radio", "file", "submit", "image", "reset", "button",
    ] {
        let (mut b, d) = builder();
        let i = html(
            &mut b,
            &d,
            "input",
            &[("type", state), ("list", &"x".repeat(100))],
        );
        let c = b.finish().unwrap();
        let cx = context(&c);
        let r = report(
            &c,
            &cx,
            &c.subject_id(&i).unwrap(),
            HtmlSemanticQuery::InputSuggestionsSource,
            state.len(),
        );
        assert_eq!(source(&r), &None, "{state}");
        assert_eq!(r.usage().peak_scratch_records(), 0);
    }
}

#[test]
fn absent_or_empty_list_skips_type_and_unknown_parent() {
    for empty in [false, true] {
        let (mut b, d) = builder();
        let huge = "x".repeat(100);
        let mut a = vec![("type", huge.as_str())];
        if empty {
            a.push(("list", ""));
        }
        let i = html(&mut b, &d, "input", &a);
        let c = b.finish().unwrap();
        let cx = context(&c);
        assert_eq!(
            source(&report(
                &c,
                &cx,
                &c.subject_id(&i).unwrap(),
                HtmlSemanticQuery::InputSuggestionsSource,
                0
            )),
            &None
        );
    }
}

#[test]
fn first_equal_id_blocks_later_datalist_without_later_coverage() {
    for (ns, name, wins) in [
        (Namespace::html(), "div", false),
        (Namespace::from_uri("urn:x"), "datalist", false),
        (Namespace::html(), "DATALIST", false),
        (Namespace::html(), "datalist", true),
    ] {
        let (mut b, d) = builder();
        let i = html(&mut b, &d, "input", &[("list", "target")]);
        let first = element(&mut b, &d, ns, name, attrs(&[("id", "target")]));
        let later = html(&mut b, &d, "datalist", &[("id", "target")]);
        children(&mut b, &i, &[]);
        // The first candidate and later subtree intentionally have unknown children.
        children(&mut b, &d, &[i.clone(), first.clone(), later]);
        let c = b.finish().unwrap();
        let cx = context(&c);
        assert_eq!(
            source(&report(
                &c,
                &cx,
                &c.subject_id(&i).unwrap(),
                HtmlSemanticQuery::InputSuggestionsSource,
                6
            )),
            &wins.then(|| c.subject_id(&first).unwrap())
        );
    }
}

#[test]
fn exact_utf16_ids_preserve_case_null_and_surrogates() {
    for units in [vec![0x41, 0], vec![0xd800, 0x20, 0xdc00]] {
        let (mut b, d) = builder();
        let key = DocumentString::from_utf16(units);
        let i = element(
            &mut b,
            &d,
            Namespace::html(),
            "input",
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "list"),
                key.clone(),
            )])
            .unwrap(),
        );
        let wrong = html(&mut b, &d, "datalist", &[("id", "a")]);
        let dl = element(
            &mut b,
            &d,
            Namespace::html(),
            "datalist",
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "id"),
                key.clone(),
            )])
            .unwrap(),
        );
        children(&mut b, &i, &[]);
        children(&mut b, &wrong, &[]);
        children(&mut b, &d, &[i.clone(), wrong, dl.clone()]);
        let c = b.finish().unwrap();
        let cx = context(&c);
        assert_eq!(
            source(&report(
                &c,
                &cx,
                &c.subject_id(&i).unwrap(),
                HtmlSemanticQuery::InputSuggestionsSource,
                key.len()
            )),
            &Some(c.subject_id(&dl).unwrap())
        );
    }
    let (c, i, _) = associated(None, "Target");
    let cx = context(&c);
    assert_eq!(
        source(&report(
            &c,
            &cx,
            &i,
            HtmlSemanticQuery::InputSuggestionsSource,
            6
        )),
        &None
    );
}

#[test]
fn namespaced_and_uppercase_list_id_lookalikes_are_ignored() {
    let (mut b, d) = builder();
    let i = element(
        &mut b,
        &d,
        Namespace::html(),
        "input",
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::from_uri("urn:x"), None, "list"),
                "target",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "LIST"),
                "target",
            ),
        ])
        .unwrap(),
    );
    let j = html(&mut b, &d, "input", &[("list", "target")]);
    let dl = element(
        &mut b,
        &d,
        Namespace::html(),
        "datalist",
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::from_uri("urn:x"), None, "id"),
                "target",
            ),
            Attribute::new(QualifiedName::new(Namespace::none(), None, "ID"), "target"),
        ])
        .unwrap(),
    );
    for x in [&i, &j, &dl] {
        children(&mut b, x, &[]);
    }
    children(&mut b, &d, &[i.clone(), j.clone(), dl]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    for x in [i, j] {
        assert_eq!(
            source(&report(
                &c,
                &cx,
                &c.subject_id(&x).unwrap(),
                HtmlSemanticQuery::InputSuggestionsSource,
                6
            )),
            &None
        );
    }
}

#[test]
fn influencing_unknown_root_or_earlier_subtree_is_missing() {
    for earlier in [false, true] {
        let (mut b, d) = builder();
        let i = html(&mut b, &d, "input", &[("list", "target")]);
        let p = html(&mut b, &d, "div", &[]);
        let dl = html(&mut b, &d, "datalist", &[("id", "target")]);
        if earlier {
            children(&mut b, &i, &[]);
            children(&mut b, &d, &[i.clone(), p.clone(), dl]);
        }
        let c = b.finish().unwrap();
        let cx = context(&c);
        let r = report(
            &c,
            &cx,
            &c.subject_id(&i).unwrap(),
            HtmlSemanticQuery::InputSuggestionsSource,
            6,
        );
        let missing = if earlier {
            HtmlSemanticMissing::OrderedChildren(c.subject_id(&p).unwrap())
        } else {
            HtmlSemanticMissing::Parent(c.subject_id(&i).unwrap())
        };
        assert_eq!(
            r.results()[0].fact(),
            &HtmlSemanticFact::InputSuggestionsSource(HtmlSemanticValue::Missing(missing))
        );
        assert_eq!(r.usage().scratch_records(), 0);
    }
}

#[test]
fn detached_ordinary_tree_resolves_external_sibling_and_root_candidate() {
    for root_candidate in [false, true] {
        let (mut b, d) = builder();
        let i = html(&mut b, &d, "input", &[("list", "target")]);
        let dl = html(&mut b, &d, "datalist", &[("id", "target")]);
        if root_candidate {
            children(&mut b, &dl, std::slice::from_ref(&i));
            b.establish_root(&dl).unwrap();
        } else {
            let f = b.add_fragment(&d).unwrap();
            children(&mut b, &i, &[]);
            children(&mut b, &f, &[dl.clone(), i.clone()]);
        }
        let c = b.finish().unwrap();
        let cx = context(&c);
        assert_eq!(
            source(&report(
                &c,
                &cx,
                &c.subject_id(&i).unwrap(),
                HtmlSemanticQuery::InputSuggestionsSource,
                6
            )),
            &Some(c.subject_id(&dl).unwrap())
        );
    }
}

#[test]
fn consulted_type_and_list_have_exact_original_raw_caps() {
    for (state, raw) in [("text", 4), ("datetime-local", 14)] {
        let list = "x".repeat(raw);
        let (c, i, _) = associated(Some(state), &list);
        let cx = context(&c);
        let req = request(&i, HtmlSemanticQuery::InputSuggestionsSource);
        assert!(
            session(&c, &cx, limits(), raw, HtmlSemanticCancellation::new())
                .query(std::slice::from_ref(&req))
                .is_ok()
        );
        let mut ss = session(&c, &cx, limits(), raw - 1, HtmlSemanticCancellation::new());
        let f = ss.query(std::slice::from_ref(&req)).unwrap_err();
        assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
        assert_eq!(f.qualification().unwrap().attribute_input_limit(), raw - 1);
        assert_eq!(ss.query(&[]).unwrap_err(), f);
    }
}

#[test]
fn collections_include_nested_barriers_and_foreign_descendants_in_source_order() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let mut all = Vec::new();
    let mut children_of_dl = Vec::new();
    for name in [
        "option", "select", "hr", "datalist", "optgroup", "script", "img",
    ] {
        let p = html(&mut b, &d, name, &[]);
        let o = html(&mut b, &d, "option", &[("value", "v")]);
        children(&mut b, &o, &[]);
        children(&mut b, &p, std::slice::from_ref(&o));
        if name == "option" {
            all.push(p.clone());
        }
        all.push(o);
        children_of_dl.push(p);
    }
    for (ns, name) in [
        (Namespace::from_uri("urn:x"), "option"),
        (Namespace::html(), "OPTION"),
    ] {
        let p = element(&mut b, &d, ns, name, attrs(&[]));
        let o = html(&mut b, &d, "option", &[("value", "v")]);
        children(&mut b, &o, &[]);
        children(&mut b, &p, std::slice::from_ref(&o));
        all.push(o);
        children_of_dl.push(p);
    }
    children(&mut b, &dl, &children_of_dl);
    b.establish_root(&dl).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = report(
        &c,
        &cx,
        &c.subject_id(&dl).unwrap(),
        HtmlSemanticQuery::DatalistOptions,
        0,
    );
    let expected: Vec<_> = all.iter().map(|o| c.subject_id(o).unwrap()).collect();
    assert_eq!(collection(&r).options(), expected);
    assert_eq!(r.usage().scratch_records(), 0);
}

#[test]
fn suggestion_disabledness_uses_option_rules_without_select_or_form_disabledness() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let own = html(&mut b, &d, "option", &[("disabled", ""), ("value", "x")]);
    let group = html(&mut b, &d, "optgroup", &[("disabled", "")]);
    let excluded = html(&mut b, &d, "option", &[("value", "x")]);
    let inner = html(&mut b, &d, "optgroup", &[]);
    let included = html(&mut b, &d, "option", &[("value", "x")]);
    let select = html(&mut b, &d, "select", &[("disabled", "")]);
    let in_select = html(&mut b, &d, "option", &[("value", "x")]);
    for o in [&own, &excluded, &included, &in_select] {
        children(&mut b, o, &[]);
    }
    children(&mut b, &inner, std::slice::from_ref(&included));
    children(&mut b, &group, &[excluded, inner]);
    children(&mut b, &select, std::slice::from_ref(&in_select));
    children(&mut b, &dl, &[own, group, select]);
    b.establish_root(&dl).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = report(
        &c,
        &cx,
        &c.subject_id(&dl).unwrap(),
        HtmlSemanticQuery::DatalistSuggestionOptions,
        1,
    );
    assert_eq!(
        collection(&r).options(),
        [
            c.subject_id(&included).unwrap(),
            c.subject_id(&in_select).unwrap()
        ]
    );
}

#[test]
fn excluded_outer_options_do_not_prune_eligible_nested_options() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let mut nested = Vec::new();
    let mut outer = Vec::new();
    for a in [
        vec![("disabled", ""), ("value", "ignored")],
        vec![("value", "")],
    ] {
        let p = html(&mut b, &d, "option", &a);
        let o = html(&mut b, &d, "option", &[("value", "x")]);
        children(&mut b, &o, &[]);
        children(&mut b, &p, std::slice::from_ref(&o));
        nested.push(o);
        outer.push(p);
    }
    children(&mut b, &dl, &outer);
    b.establish_root(&dl).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    assert_eq!(
        collection(&report(
            &c,
            &cx,
            &c.subject_id(&dl).unwrap(),
            HtmlSemanticQuery::DatalistSuggestionOptions,
            1
        ))
        .options(),
        nested
            .iter()
            .map(|o| c.subject_id(o).unwrap())
            .collect::<Vec<_>>()
    );
}

#[test]
fn eligibility_distinguishes_present_empty_whitespace_and_exact_text_values() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let mut os = Vec::new();
    let mut yes = Vec::new();
    for (attribute, text, eligible) in [
        (Some(DocumentString::from("")), None, false),
        (Some(DocumentString::from(" \t")), None, true),
        (None, Some(DocumentString::from(" \t\n\r\u{c}")), false),
        (
            None,
            Some(DocumentString::from_utf16(vec![0xa0, 0xd800])),
            true,
        ),
        (None, Some(DocumentString::from(" A  B ")), true),
    ] {
        let a = OrderedAttributes::new(
            attribute
                .into_iter()
                .map(|v| Attribute::new(QualifiedName::new(Namespace::none(), None, "value"), v)),
        )
        .unwrap();
        let o = element(&mut b, &d, Namespace::html(), "option", a);
        let ts: Vec<_> = text
            .into_iter()
            .map(|v| b.add_text(&d, v).unwrap())
            .collect();
        children(&mut b, &o, &ts);
        if eligible {
            yes.push(o.clone());
        }
        os.push(o);
    }
    children(&mut b, &dl, &os);
    b.establish_root(&dl).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = report(
        &c,
        &cx,
        &c.subject_id(&dl).unwrap(),
        HtmlSemanticQuery::DatalistSuggestionOptions,
        2,
    );
    assert_eq!(
        collection(&r).options(),
        yes.iter()
            .map(|o| c.subject_id(o).unwrap())
            .collect::<Vec<_>>()
    );
    assert!(r.usage().nested_output_records() > yes.len()); // discarded normalized values are admitted too
    assert_eq!(r.usage().scratch_records(), 0);
}

#[test]
fn own_disabled_and_all_collection_skip_irrelevant_large_values_and_labels() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let o = html(
        &mut b,
        &d,
        "option",
        &[
            ("disabled", ""),
            ("value", &"x".repeat(100)),
            ("label", &"y".repeat(100)),
        ],
    );
    children(&mut b, &o, &[]);
    children(&mut b, &dl, std::slice::from_ref(&o));
    b.establish_root(&dl).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let id = c.subject_id(&dl).unwrap();
    assert_eq!(
        collection(&report(&c, &cx, &id, HtmlSemanticQuery::DatalistOptions, 0)).options(),
        [c.subject_id(&o).unwrap()]
    );
    assert!(
        collection(&report(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::DatalistSuggestionOptions,
            0
        ))
        .options()
        .is_empty()
    );
}

#[test]
fn influencing_later_descendant_or_value_text_gap_never_returns_subset() {
    for text_gap in [false, true] {
        let (mut b, d) = builder();
        let dl = html(&mut b, &d, "datalist", &[]);
        let good = html(&mut b, &d, "option", &[("value", "x")]);
        children(&mut b, &good, &[]);
        let gap = html(&mut b, &d, if text_gap { "option" } else { "div" }, &[]);
        children(&mut b, &dl, &[good, gap.clone()]);
        b.establish_root(&dl).unwrap();
        let c = b.finish().unwrap();
        let cx = context(&c);
        let r = report(
            &c,
            &cx,
            &c.subject_id(&dl).unwrap(),
            HtmlSemanticQuery::DatalistSuggestionOptions,
            1,
        );
        assert_eq!(
            r.results()[0].fact(),
            &HtmlSemanticFact::DatalistSuggestionOptions(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::OrderedChildren(c.subject_id(&gap).unwrap())
            ))
        );
        assert_eq!(r.usage().scratch_records(), 0);
        assert!(r.usage().nested_output_records() > 0);
    }
}

#[test]
fn established_empty_needs_no_scratch_nested_output_or_unrelated_host_facts() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    children(&mut b, &dl, &[]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    for q in [
        HtmlSemanticQuery::DatalistOptions,
        HtmlSemanticQuery::DatalistSuggestionOptions,
    ] {
        let r = session(
            &c,
            &cx,
            HtmlSemanticLimits::new(limits().work(), 1, 0, 0, 100000, 0),
            0,
            HtmlSemanticCancellation::new(),
        )
        .query(&[request(&c.subject_id(&dl).unwrap(), q)])
        .unwrap();
        assert!(collection(&r).options().is_empty());
        assert_eq!(r.usage().peak_scratch_records(), 0);
        assert_eq!(r.usage().nested_output_records(), 0);
    }
}

#[test]
fn select_and_datalist_share_the_public_collection_without_sharing_pruning() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let s = html(&mut b, &d, "select", &[]);
    let p = html(&mut b, &d, "option", &[]);
    let nested = html(&mut b, &d, "option", &[]);
    children(&mut b, &nested, &[]);
    children(&mut b, &p, std::slice::from_ref(&nested));
    children(&mut b, &s, std::slice::from_ref(&p));
    children(&mut b, &dl, std::slice::from_ref(&s));
    let c = b.finish().unwrap();
    let cx = context(&c);
    let a = report(
        &c,
        &cx,
        &c.subject_id(&dl).unwrap(),
        HtmlSemanticQuery::DatalistOptions,
        0,
    );
    let b = report(
        &c,
        &cx,
        &c.subject_id(&s).unwrap(),
        HtmlSemanticQuery::SelectOptions,
        0,
    );
    let all: &HtmlOptionCollection = collection(&a);
    let selected_source: &HtmlOptionCollection = collection(&b);
    assert_eq!(
        all.options(),
        [c.subject_id(&p).unwrap(), c.subject_id(&nested).unwrap()]
    );
    assert_eq!(selected_source.options(), [c.subject_id(&p).unwrap()]);
}

#[test]
fn shadow_and_template_sources_are_local_and_associated_subtrees_are_not_collected() {
    for shadow in [false, true] {
        let (mut b, d) = builder();
        let host = html(&mut b, &d, if shadow { "div" } else { "template" }, &[]);
        let outside = html(&mut b, &d, "datalist", &[("id", "target")]);
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
        let i = html(&mut b, &d, "input", &[("list", "target")]);
        let local = html(&mut b, &d, "datalist", &[("id", "target")]);
        let o = html(&mut b, &d, "option", &[("value", "x")]);
        children(&mut b, &o, &[]);
        children(&mut b, &local, std::slice::from_ref(&o));
        children(&mut b, &i, &[]);
        children(&mut b, &f, &[i.clone(), local.clone()]);
        children(&mut b, &host, &[]);
        children(&mut b, &outside, std::slice::from_ref(&host));
        children(&mut b, &d, std::slice::from_ref(&outside));
        let c = b.finish().unwrap();
        let cx = context(&c);
        assert_eq!(
            source(&report(
                &c,
                &cx,
                &c.subject_id(&i).unwrap(),
                HtmlSemanticQuery::InputSuggestionsSource,
                6
            )),
            &Some(c.subject_id(&local).unwrap())
        );
        assert!(
            collection(&report(
                &c,
                &cx,
                &c.subject_id(&outside).unwrap(),
                HtmlSemanticQuery::DatalistOptions,
                0
            ))
            .options()
            .is_empty()
        );
    }
}

#[test]
fn no_same_tree_candidate_establishes_null_despite_matching_host_id() {
    let (mut b, d) = builder();
    let host = html(&mut b, &d, "template", &[("id", "target")]);
    let f = b.add_fragment(&d).unwrap();
    b.establish_template_contents(&host, &f).unwrap();
    let i = html(&mut b, &d, "input", &[("list", "target")]);
    children(&mut b, &i, &[]);
    children(&mut b, &f, std::slice::from_ref(&i));
    let c = b.finish().unwrap();
    let cx = context(&c);
    assert_eq!(
        source(&report(
            &c,
            &cx,
            &c.subject_id(&i).unwrap(),
            HtmlSemanticQuery::InputSuggestionsSource,
            6
        )),
        &None
    );
}

#[test]
fn actual_html_roles_are_distinct_from_foreign_unknown_and_non_element() {
    let (mut b, d) = builder();
    let div = html(&mut b, &d, "div", &[]);
    let custom = html(&mut b, &d, "x-control", &[]);
    let upper = html(&mut b, &d, "DATALIST", &[]);
    let foreign = element(
        &mut b,
        &d,
        Namespace::from_uri("urn:x"),
        "datalist",
        attrs(&[]),
    );
    let c = b.finish().unwrap();
    let cx = context(&c);
    for q in [
        HtmlSemanticQuery::InputSuggestionsSource,
        HtmlSemanticQuery::DatalistOptions,
        HtmlSemanticQuery::DatalistSuggestionOptions,
    ] {
        for x in [&d, &div, &custom, &upper] {
            let r = report(&c, &cx, &c.subject_id(x).unwrap(), q, 0);
            match r.results()[0].fact() {
                HtmlSemanticFact::InputSuggestionsSource(HtmlSemanticValue::Inapplicable)
                | HtmlSemanticFact::DatalistOptions(HtmlSemanticValue::Inapplicable)
                | HtmlSemanticFact::DatalistSuggestionOptions(HtmlSemanticValue::Inapplicable) => {}
                f => panic!("expected inapplicable {q:?}: {f:?}"),
            }
        }
        let r = report(&c, &cx, &c.subject_id(&foreign).unwrap(), q, 0);
        match r.results()[0].fact() {
            HtmlSemanticFact::InputSuggestionsSource(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ))
            | HtmlSemanticFact::DatalistOptions(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ))
            | HtmlSemanticFact::DatalistSuggestionOptions(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            )) => {}
            f => panic!("expected unsupported: {f:?}"),
        }
    }
}

fn text_options() -> (HtmlCapture, HtmlCaptureSubjectId, [HtmlCaptureSubjectId; 2]) {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let mut os = Vec::new();
    for v in [" A B ", " C D "] {
        let o = html(&mut b, &d, "option", &[]);
        let span = html(&mut b, &d, "span", &[]);
        let t = b.add_text(&d, DocumentString::from(v)).unwrap();
        children(&mut b, &span, &[t]);
        children(&mut b, &o, std::slice::from_ref(&span));
        os.push(o);
    }
    children(&mut b, &dl, &os);
    b.establish_root(&dl).unwrap();
    let c = b.finish().unwrap();
    let ids = [c.subject_id(&os[0]).unwrap(), c.subject_id(&os[1]).unwrap()];
    let id = c.subject_id(&dl).unwrap();
    (c, id, ids)
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
    raw: usize,
) {
    let base = report(c, cx, id, q, raw);
    let u = base.usage();
    let req = request(id, q);
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
                raw,
                HtmlSemanticCancellation::new(),
            );
            let outcome = ss.query(std::slice::from_ref(&req));
            if cap == exact {
                assert!(outcome.is_ok(), "{q:?} {resource:?}: {outcome:?}");
            } else {
                let f = outcome.unwrap_err();
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
fn nested_text_temporaries_and_retained_id_capacity_have_exact_one_short_limits() {
    let (c, dl, os) = text_options();
    let cx = context(&c);
    let r = report(
        &c,
        &cx,
        &dl,
        HtmlSemanticQuery::DatalistSuggestionOptions,
        0,
    );
    assert_eq!(collection(&r).options(), os);
    // Each normalized value has three units, even though only identities escape.
    assert!(r.usage().nested_output_records() >= 8);
    assert!(r.usage().peak_scratch_records() >= 3);
    assert_eq!(r.usage().scratch_records(), 0);
    for q in [
        HtmlSemanticQuery::DatalistOptions,
        HtmlSemanticQuery::DatalistSuggestionOptions,
    ] {
        check_caps(&c, &cx, &dl, q, 0);
    }
    let (c, i, _) = associated(None, "target");
    let cx = context(&c);
    check_caps(&c, &cx, &i, HtmlSemanticQuery::InputSuggestionsSource, 6);
}

#[test]
fn rejected_second_value_keeps_cumulative_capacity_and_no_batch_prefix() {
    let (c, dl, _) = text_options();
    let cx = context(&c);
    let req = request(&dl, HtmlSemanticQuery::DatalistSuggestionOptions);
    let baseline = report(
        &c,
        &cx,
        &dl,
        HtmlSemanticQuery::DatalistSuggestionOptions,
        0,
    )
    .usage();
    let mut ss = session(
        &c,
        &cx,
        limited(
            HtmlSemanticResource::NestedOutputRecords,
            baseline.nested_output_records(),
        ),
        0,
        HtmlSemanticCancellation::new(),
    );
    let f = ss.query(&[req.clone(), req]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert!(f.usage().nested_output_records() >= baseline.nested_output_records());
    assert_eq!(f.qualification().unwrap().context(), cx.identity());
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}

#[test]
fn cancellation_after_a_success_and_foreign_capture_failures_retain_qualification() {
    let (c, dl, _) = text_options();
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut ss = session(&c, &cx, limits(), 0, cancel.clone());
    ss.query(&[request(&dl, HtmlSemanticQuery::DatalistOptions)])
        .unwrap();
    cancel.cancel();
    let f = ss
        .query(&[request(&dl, HtmlSemanticQuery::DatalistSuggestionOptions)])
        .unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap().capture(), c.identity());
    assert_eq!(
        f.qualification().unwrap().host_revision(),
        c.host_facts_revision()
    );
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    let (_, foreign, _) = text_options();
    let mut ss = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let f = ss
        .query(&[
            request(&dl, HtmlSemanticQuery::DatalistOptions),
            request(&foreign, HtmlSemanticQuery::DatalistOptions),
        ])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}

#[test]
fn changed_captures_and_irrelevant_selectedness_contexts_remain_qualified() {
    let (c, dl, os) = text_options();
    let a = context(&c);
    let flags = HtmlControlFacts::new(
        &c,
        c.host_facts_revision().clone(),
        vec![],
        vec![],
        os.iter()
            .map(|id| HtmlOptionControlState::new(id.clone(), Some(false), None))
            .collect(),
        vec![],
        HtmlControlFactsLimits::new(100, 100000),
    )
    .unwrap();
    let b = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_controls(flags),
        0,
    )
    .unwrap();
    let ar = report(&c, &a, &dl, HtmlSemanticQuery::DatalistSuggestionOptions, 0);
    let br = report(&c, &b, &dl, HtmlSemanticQuery::DatalistSuggestionOptions, 0);
    assert_eq!(collection(&ar), collection(&br));
    assert_ne!(ar.context(), br.context());
    assert_eq!(ar.capture(), br.capture());
    assert_eq!(
        ar.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    let (changed, i, _) = associated(None, "absent");
    let cx = context(&changed);
    let cr = report(
        &changed,
        &cx,
        &i,
        HtmlSemanticQuery::InputSuggestionsSource,
        6,
    );
    assert_eq!(source(&cr), &None);
    assert_ne!(ar.capture(), cr.capture());
    assert_ne!(ar.host_revision(), cr.host_revision());
}

#[test]
fn eligible_value_keeps_its_raw_cap_and_late_batch_stop_is_atomic() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let o = html(&mut b, &d, "option", &[("value", "long")]);
    children(&mut b, &o, &[]);
    children(&mut b, &dl, &[o]);
    let c = b.finish().unwrap();
    let cx = context(&c);
    let id = c.subject_id(&dl).unwrap();
    assert_eq!(
        collection(&report(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::DatalistSuggestionOptions,
            4
        ))
        .options()
        .len(),
        1
    );
    let mut ss = session(&c, &cx, limits(), 3, HtmlSemanticCancellation::new());
    let f = ss
        .query(&[
            request(&id, HtmlSemanticQuery::DatalistOptions),
            request(&id, HtmlSemanticQuery::DatalistSuggestionOptions),
        ])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(f.qualification().unwrap().attribute_input_limit(), 3);
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}

#[test]
fn suggestion_values_consume_text_owner_pruning_but_collection_still_sees_nested_options() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let o = html(&mut b, &d, "option", &[]);
    let mut excluded = Vec::new();
    let mut nested = Vec::new();
    for name in ["script", "img"] {
        let p = html(&mut b, &d, name, &[("alt", "must not supply value")]);
        let n = html(&mut b, &d, "option", &[("value", "x")]);
        let t = b.add_text(&d, DocumentString::from("ignored")).unwrap();
        children(&mut b, &n, &[]);
        children(&mut b, &p, &[t, n.clone()]);
        excluded.push(p);
        nested.push(n);
    }
    children(&mut b, &o, &excluded);
    children(&mut b, &dl, std::slice::from_ref(&o));
    b.establish_root(&dl).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    let r = report(
        &c,
        &cx,
        &c.subject_id(&dl).unwrap(),
        HtmlSemanticQuery::DatalistSuggestionOptions,
        1,
    );
    assert_eq!(
        collection(&r).options(),
        nested
            .iter()
            .map(|o| c.subject_id(o).unwrap())
            .collect::<Vec<_>>()
    );
}

#[test]
fn descendant_value_and_topology_changes_produce_new_capture_qualified_membership() {
    let mut observations = Vec::new();
    for (explicit, text_value, included) in [
        (Some(""), "text", true),
        (Some("x"), "", true),
        (None, "  ", true),
        (None, "x", false),
    ] {
        let (mut b, d) = builder();
        let dl = html(&mut b, &d, "datalist", &[]);
        let a = explicit.map(|v| vec![("value", v)]).unwrap_or_default();
        let o = html(&mut b, &d, "option", &a);
        let t = b.add_text(&d, DocumentString::from(text_value)).unwrap();
        children(&mut b, &o, &[t]);
        if included {
            children(&mut b, &dl, std::slice::from_ref(&o));
        } else {
            children(&mut b, &dl, &[]);
        }
        let c = b.finish().unwrap();
        let cx = context(&c);
        let r = report(
            &c,
            &cx,
            &c.subject_id(&dl).unwrap(),
            HtmlSemanticQuery::DatalistSuggestionOptions,
            1,
        );
        assert_eq!(
            collection(&r).options().len(),
            usize::from(explicit == Some("x"))
        );
        observations.push(r);
    }
    for pair in observations.windows(2) {
        assert_ne!(pair[0].capture(), pair[1].capture());
        assert_ne!(pair[0].context(), pair[1].context());
    }
}

#[test]
fn deep_borrowed_descendant_walk_has_finite_scratch_and_releases_after_stop() {
    let (mut b, d) = builder();
    let dl = html(&mut b, &d, "datalist", &[]);
    let mut p = dl.clone();
    for _ in 0..80 {
        let child = html(&mut b, &d, "div", &[]);
        children(&mut b, &p, std::slice::from_ref(&child));
        p = child;
    }
    let o = html(&mut b, &d, "option", &[("value", "x")]);
    children(&mut b, &o, &[]);
    children(&mut b, &p, std::slice::from_ref(&o));
    let c = b.finish().unwrap();
    let cx = context(&c);
    let id = c.subject_id(&dl).unwrap();
    let r = report(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::DatalistSuggestionOptions,
        1,
    );
    assert_eq!(collection(&r).options(), [c.subject_id(&o).unwrap()]);
    assert!(r.usage().peak_scratch_records() >= 81);
    assert_eq!(r.usage().scratch_records(), 0);
    let mut ss = session(
        &c,
        &cx,
        limited(HtmlSemanticResource::ScratchRecords, 40),
        1,
        HtmlSemanticCancellation::new(),
    );
    let f = ss
        .query(&[request(&id, HtmlSemanticQuery::DatalistOptions)])
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
