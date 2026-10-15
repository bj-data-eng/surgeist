use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlCapturedNodeKind,
    HtmlCapturedShadowProperties, HtmlComputedWritingSuggestions, HtmlControlFacts,
    HtmlControlFactsLimits, HtmlDesignMode, HtmlDocumentDesignModeFact, HtmlFormAssociation,
    HtmlFormAssociationState, HtmlFormOwnerOrigin, HtmlGlobalInheritanceFacts,
    HtmlGlobalInheritanceLimits, HtmlGlobalInheritanceProducerId, HtmlGlobalInheritanceRevision,
    HtmlOwnAutocapitalizationHint, HtmlScriptingStatus, HtmlSemanticCancellation,
    HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticUsage, HtmlSemanticValue,
    HtmlUsedAutocapitalizationBranch, HtmlUsedAutocapitalizationHint, HtmlUsedAutocorrectionState,
};
#[path = "common/session.rs"]
mod shared_session;
use shared_session::session;
use surgeist_html::*;

// Literal expectations follow HTML 877d's four distinct algorithms and the
// selected execCommand 85de9ff4 editing-host-of definition, not browser helpers.
fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(2000, 100000, 2000, 1000000, 2000, 10, 10),
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
fn expanded(
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
    expanded(
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
fn input_context(c: &HtmlCapture, input: HtmlSemanticContextInput) -> HtmlSemanticContext {
    HtmlSemanticContext::new(c, c.host_facts_revision().clone(), input, 100).unwrap()
}
fn context(c: &HtmlCapture) -> HtmlSemanticContext {
    input_context(c, HtmlSemanticContextInput::new(vec![]))
}
fn with_owners(
    c: &HtmlCapture,
    owners: Vec<(HtmlCaptureSubjectId, HtmlFormAssociationState)>,
) -> HtmlSemanticContext {
    let f = HtmlControlFacts::new(
        c,
        c.host_facts_revision().clone(),
        owners
            .into_iter()
            .map(|(id, state)| HtmlFormAssociation::new(id, state))
            .collect(),
        vec![],
        vec![],
        vec![],
        HtmlControlFactsLimits::new(100, 1000000),
    )
    .unwrap();
    input_context(c, HtmlSemanticContextInput::new(vec![]).with_controls(f))
}
fn design(
    c: &HtmlCapture,
    d: &HtmlCaptureBuilderSubject,
    state: HtmlDesignMode,
) -> HtmlSemanticContext {
    let rev = HtmlGlobalInheritanceRevision::new(HtmlGlobalInheritanceProducerId::fresh(), 31, 47);
    let facts = HtmlGlobalInheritanceFacts::new(
        c,
        c.host_facts_revision().clone(),
        rev.clone(),
        vec![HtmlDocumentDesignModeFact::new(
            c.subject_id(d).unwrap(),
            state,
        )],
        vec![],
        HtmlGlobalInheritanceLimits::new(100, 1000000),
    )
    .unwrap();
    input_context(
        c,
        HtmlSemanticContextInput::new(vec![]).with_global_inheritance(rev, Some(facts)),
    )
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(20000000, 100, 10000, 1000000, 10000000, 1000000)
}

fn req(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), q)
}
fn query(
    c: &HtmlCapture,
    cx: &HtmlSemanticContext,
    id: &HtmlCaptureSubjectId,
    q: HtmlSemanticQuery,
    raw: usize,
) -> HtmlSemanticReport {
    session(c, cx, limits(), raw, HtmlSemanticCancellation::new())
        .query(&[req(id, q)])
        .unwrap()
}
fn own(r: &HtmlSemanticReport) -> &HtmlOwnAutocapitalizationHint {
    let HtmlSemanticFact::OwnAutocapitalizationHint(HtmlSemanticValue::Known(v)) =
        r.results()[0].fact()
    else {
        panic!("{:?}", r.results()[0].fact());
    };
    v
}
fn used(r: &HtmlSemanticReport) -> &HtmlUsedAutocapitalizationHint {
    let HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Known(v)) =
        r.results()[0].fact()
    else {
        panic!("{:?}", r.results()[0].fact());
    };
    v
}
fn correct(r: &HtmlSemanticReport) -> &HtmlUsedAutocorrectionState {
    let HtmlSemanticFact::UsedAutocorrectionState(HtmlSemanticValue::Known(v)) =
        r.results()[0].fact()
    else {
        panic!("{:?}", r.results()[0].fact());
    };
    v
}
fn writing(r: &HtmlSemanticReport) -> &HtmlComputedWritingSuggestions {
    let HtmlSemanticFact::ComputedWritingSuggestions(HtmlSemanticValue::Known(v)) =
        r.results()[0].fact()
    else {
        panic!("{:?}", r.results()[0].fact());
    };
    v
}

#[test]
fn own_nonempty_aliases_and_invalid_content_have_literal_canonical_getters() {
    for (raw, expected) in [
        ("off", "none"),
        ("NoNe", "none"),
        ("on", "sentences"),
        ("SENTENCES", "sentences"),
        ("words", "words"),
        ("characters", "characters"),
        (" words ", "sentences"),
        (" ", "sentences"),
        ("invalid", "sentences"),
    ] {
        let (c, id) = fixture("input", &[("autocapitalize", raw)], false);
        let cx = context(&c);
        let r = query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::OwnAutocapitalizationHint,
            raw.len(),
        );
        assert_eq!(own(&r).idl_value(), expected);
        assert!(own(&r).owner().is_none());
        assert_eq!(
            own(&r).own_report().unwrap().raw().unwrap().code_units(),
            raw.encode_utf16().collect::<Vec<_>>()
        );
    }
}
#[test]
fn own_empty_is_preserved_without_applying_local_invalid_default() {
    for raw in [None, Some("")] {
        let (c, id) = fixture(
            "div",
            &raw.map(|v| vec![("autocapitalize", v)]).unwrap_or_default(),
            false,
        );
        let r = query(
            &c,
            &context(&c),
            &id,
            HtmlSemanticQuery::OwnAutocapitalizationHint,
            0,
        );
        assert_eq!(own(&r).state(), HtmlAutocapitalizeState::Default);
        assert_eq!(own(&r).idl_value(), "");
        assert!(own(&r).own_report().is_none());
        assert_eq!(
            own(&r).own_attribute().map(|a| a.value().is_empty()),
            raw.map(|_| true)
        );
    }
}
#[test]
fn all_six_category_elements_inherit_actual_nonancestor_form_and_keep_empty() {
    let (mut b, d) = builder();
    let form = html(
        &mut b,
        &d,
        "form",
        &[("autocapitalize", "WORDS"), ("autocorrect", "off")],
    );
    let mut nodes = Vec::new();
    for local in [
        "button", "fieldset", "input", "output", "select", "textarea",
    ] {
        nodes.push(html(&mut b, &d, local, &[("autocapitalize", "")]));
    }
    let mut cs = vec![form.clone()];
    cs.extend(nodes.iter().cloned());
    b.establish_children(&d, &cs).unwrap();
    let c = b.finish().unwrap();
    let fid = c.subject_id(&form).unwrap();
    let cx = with_owners(
        &c,
        nodes
            .iter()
            .map(|n| {
                (
                    c.subject_id(n).unwrap(),
                    HtmlFormAssociationState::EstablishedCurrent(Some(fid.clone())),
                )
            })
            .collect(),
    );
    for n in nodes {
        let id = c.subject_id(&n).unwrap();
        let r = query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::OwnAutocapitalizationHint,
            5,
        );
        let v = own(&r);
        assert_eq!(v.idl_value(), "words");
        assert!(v.own_attribute().unwrap().value().is_empty());
        assert!(v.own_report().is_none());
        assert_eq!(v.owner().unwrap().owner(), Some(&fid));
        assert_eq!(
            v.owner().unwrap().origin(),
            HtmlFormOwnerOrigin::EstablishedCurrent
        );
        let r = query(&c, &cx, &id, HtmlSemanticQuery::UsedAutocorrectionState, 5);
        assert!(!correct(&r).idl_value());
        assert_eq!(correct(&r).owner().unwrap().owner(), Some(&fid));
    }
}
#[test]
fn noncategory_elements_do_not_consult_custom_or_association_facts() {
    for local in [
        "object",
        "x-control",
        "form",
        "div",
        "img",
        "meter",
        "progress",
    ] {
        let (c, id) = fixture(local, &[], false);
        let cx = context(&c);
        assert_eq!(
            own(&query(
                &c,
                &cx,
                &id,
                HtmlSemanticQuery::OwnAutocapitalizationHint,
                0
            ))
            .idl_value(),
            ""
        );
        assert!(
            correct(&query(
                &c,
                &cx,
                &id,
                HtmlSemanticQuery::UsedAutocorrectionState,
                0
            ))
            .idl_value()
        );
    }
}
#[test]
fn actual_null_defaults_differ_from_unavailable_association() {
    let (c, id) = fixture("input", &[], false);
    let absent = context(&c);
    for q in [
        HtmlSemanticQuery::OwnAutocapitalizationHint,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        HtmlSemanticQuery::UsedAutocorrectionState,
    ] {
        let r = query(&c, &absent, &id, q, 0);
        assert!(matches!(
            r.results()[0].fact(),
            HtmlSemanticFact::OwnAutocapitalizationHint(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::FormAssociation(_)
            )) | HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::FormAssociation(_)
            )) | HtmlSemanticFact::UsedAutocorrectionState(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::FormAssociation(_)
            ))
        ));
    }
    let cx = with_owners(
        &c,
        vec![(
            id.clone(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
    );
    let r = query(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::OwnAutocapitalizationHint,
        0,
    );
    assert_eq!(own(&r).idl_value(), "");
    assert_eq!(own(&r).owner().unwrap().owner(), None);
    assert_eq!(
        own(&r).owner().unwrap().origin(),
        HtmlFormOwnerOrigin::EstablishedCurrent
    );
    assert!(
        correct(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::UsedAutocorrectionState,
            0
        ))
        .idl_value()
    );
}
#[test]
fn reset_owner_uses_first_id_and_form_content_not_nearest_form() {
    let (mut b, d) = builder();
    let form = html(
        &mut b,
        &d,
        "form",
        &[
            ("id", "outside"),
            ("autocapitalize", "characters"),
            ("autocorrect", "OFF"),
        ],
    );
    let inner = html(
        &mut b,
        &d,
        "form",
        &[("autocapitalize", "none"), ("autocorrect", "on")],
    );
    let e = html(&mut b, &d, "textarea", &[("form", "outside")]);
    b.establish_children(&inner, std::slice::from_ref(&e))
        .unwrap();
    b.establish_children(&form, &[]).unwrap();
    b.establish_children(&e, &[]).unwrap();
    b.establish_children(&d, &[form.clone(), inner]).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let fid = c.subject_id(&form).unwrap();
    let cx = with_owners(
        &c,
        vec![(id.clone(), HtmlFormAssociationState::ResetAgainstCapture)],
    );
    let r = query(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::OwnAutocapitalizationHint,
        20,
    );
    assert_eq!(own(&r).idl_value(), "characters");
    assert_eq!(own(&r).owner().unwrap().owner(), Some(&fid));
    assert_eq!(
        own(&r).owner().unwrap().origin(),
        HtmlFormOwnerOrigin::ResetAgainstCapture
    );
    let r = query(&c, &cx, &id, HtmlSemanticQuery::UsedAutocorrectionState, 20);
    assert!(!correct(&r).idl_value());
    assert_eq!(r.usage().scratch_records(), 0);
    assert!(r.usage().peak_scratch_records() > 0);
}
#[test]
fn selected_form_absent_empty_invalid_defaults_are_phase_specific() {
    for (raw, cap, corr) in [
        (None, "", true),
        (Some(""), "", true),
        (Some("off"), "none", false),
        (Some("invalid"), "sentences", true),
    ] {
        let (mut b, d) = builder();
        let attrs = raw
            .map(|v| vec![("autocapitalize", v), ("autocorrect", v)])
            .unwrap_or_default();
        let f = html(&mut b, &d, "form", &attrs);
        let e = html(&mut b, &d, "select", &[]);
        b.establish_children(&d, &[f.clone(), e.clone()]).unwrap();
        let c = b.finish().unwrap();
        let id = c.subject_id(&e).unwrap();
        let cx = with_owners(
            &c,
            vec![(
                id.clone(),
                HtmlFormAssociationState::EstablishedCurrent(Some(c.subject_id(&f).unwrap())),
            )],
        );
        let r = query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::OwnAutocapitalizationHint,
            20,
        );
        assert_eq!(own(&r).idl_value(), cap);
        assert_eq!(
            own(&r).form_attribute().map(|a| a.value().is_empty()),
            raw.map(str::is_empty)
        );
        assert_eq!(
            own(&r).form_report().is_some(),
            raw.is_some_and(|v| !v.is_empty())
        );
        assert_eq!(
            correct(&query(
                &c,
                &cx,
                &id,
                HtmlSemanticQuery::UsedAutocorrectionState,
                20
            ))
            .idl_value(),
            corr
        );
    }
}
#[test]
fn three_input_types_keep_own_hint_but_suppress_used_hint_and_autocorrect() {
    for t in ["url", "EMAIL", "password"] {
        let (c, id) = fixture(
            "input",
            &[
                ("type", t),
                ("autocapitalize", "words"),
                ("autocorrect", "on"),
            ],
            false,
        );
        let cx = context(&c);
        assert_eq!(
            own(&query(
                &c,
                &cx,
                &id,
                HtmlSemanticQuery::OwnAutocapitalizationHint,
                8
            ))
            .idl_value(),
            "words"
        );
        let r = query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::UsedAutocapitalizationHint,
            8,
        );
        let v = used(&r);
        assert_eq!(v.state(), HtmlAutocapitalizeState::Default);
        assert_eq!(
            v.branch(),
            HtmlUsedAutocapitalizationBranch::SuppressedInput
        );
        assert!(v.own_hint().is_none());
        assert!(v.selected_subject().is_none());
        assert_eq!(
            v.input_type().unwrap().raw().unwrap().code_units(),
            t.encode_utf16().collect::<Vec<_>>()
        );
        let r = query(&c, &cx, &id, HtmlSemanticQuery::UsedAutocorrectionState, 8);
        assert!(!correct(&r).idl_value());
        assert!(correct(&r).own_report().is_none());
        assert!(correct(&r).owner().is_none());
    }
}
#[test]
fn suppression_does_not_inspect_oversized_own_values_or_unknown_host() {
    for t in ["url", "email", "password"] {
        let (c, id) = fixture(
            "input",
            &[
                ("type", t),
                ("autocapitalize", "an uninspected very large value"),
                ("autocorrect", "an uninspected very large value"),
                ("contenteditable", "an uninspected very large value"),
            ],
            false,
        );
        let cx = context(&c);
        used(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::UsedAutocapitalizationHint,
            t.len(),
        ));
        assert!(
            !correct(&query(
                &c,
                &cx,
                &id,
                HtmlSemanticQuery::UsedAutocorrectionState,
                t.len()
            ))
            .idl_value()
        );
    }
}
#[test]
fn other_input_states_and_textarea_use_own_not_editing_facts() {
    for t in [
        None,
        Some(""),
        Some("bogus"),
        Some("text"),
        Some("date"),
        Some("hidden"),
        Some("file"),
    ] {
        let mut attrs = vec![
            ("autocapitalize", "characters"),
            ("autocorrect", "off"),
            ("readonly", "uninspected"),
            ("disabled", "uninspected"),
        ];
        if let Some(t) = t {
            attrs.push(("type", t));
        }
        let (c, id) = fixture("input", &attrs, false);
        let cx = context(&c);
        let r = query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::UsedAutocapitalizationHint,
            10,
        );
        let v = used(&r);
        assert_eq!(v.branch(), HtmlUsedAutocapitalizationBranch::Control);
        assert_eq!(v.selected_subject(), Some(&id));
        assert_eq!(v.own_hint().unwrap().idl_value(), "characters");
        assert!(
            !correct(&query(
                &c,
                &cx,
                &id,
                HtmlSemanticQuery::UsedAutocorrectionState,
                10
            ))
            .idl_value()
        );
    }
    let (c, id) = fixture("textarea", &[("autocapitalize", "words")], false);
    let r = query(
        &c,
        &context(&c),
        &id,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        5,
    );
    assert!(used(&r).input_type().is_none());
    assert_eq!(used(&r).own_hint().unwrap().idl_value(), "words");
}
#[test]
fn autocorrect_present_empty_invalid_and_off_bypass_unknown_owner() {
    for (raw, expected) in [
        ("", true),
        ("nope", true),
        (" ", true),
        ("ON", true),
        ("off", false),
    ] {
        let (c, id) = fixture("input", &[("autocorrect", raw)], false);
        let r = query(
            &c,
            &context(&c),
            &id,
            HtmlSemanticQuery::UsedAutocorrectionState,
            raw.len(),
        );
        assert_eq!(correct(&r).idl_value(), expected);
        assert!(correct(&r).owner().is_none());
        assert_eq!(
            correct(&r)
                .own_report()
                .unwrap()
                .raw()
                .unwrap()
                .code_units(),
            raw.encode_utf16().collect::<Vec<_>>()
        );
    }
}
#[test]
fn nearest_editing_host_selects_used_hint_while_own_getter_stays_local() {
    let (mut b, d) = builder();
    let outer = html(
        &mut b,
        &d,
        "div",
        &[
            ("contenteditable", "true"),
            ("autocapitalize", "words"),
            ("autocorrect", "off"),
        ],
    );
    let inner = html(
        &mut b,
        &d,
        "section",
        &[
            ("contenteditable", "plaintext-only"),
            ("autocapitalize", "characters"),
        ],
    );
    let child = html(&mut b, &d, "span", &[("autocapitalize", "none")]);
    b.establish_parent(&child, &inner).unwrap();
    b.establish_parent(&inner, &outer).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&child).unwrap();
    let hid = c.subject_id(&inner).unwrap();
    let cx = context(&c);
    assert_eq!(
        own(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::OwnAutocapitalizationHint,
            20
        ))
        .idl_value(),
        "none"
    );
    let r = query(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        20,
    );
    assert_eq!(
        used(&r).branch(),
        HtmlUsedAutocapitalizationBranch::EditingHost
    );
    assert_eq!(used(&r).selected_subject(), Some(&hid));
    assert_eq!(used(&r).own_hint().unwrap().idl_value(), "characters");
    assert!(
        correct(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::UsedAutocorrectionState,
            20
        ))
        .idl_value()
    );
    let r = query(
        &c,
        &cx,
        &hid,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        20,
    );
    assert_eq!(used(&r).selected_subject(), Some(&hid));
}
#[test]
fn false_barriers_and_known_detached_nonhosts_are_inapplicable() {
    for local in ["div", "x-control"] {
        let (c, id) = fixture(local, &[], true);
        let r = query(
            &c,
            &context(&c),
            &id,
            HtmlSemanticQuery::UsedAutocapitalizationHint,
            0,
        );
        assert!(matches!(
            r.results()[0].fact(),
            HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Inapplicable)
        ));
    }
    let (mut b, d) = builder();
    let host = html(&mut b, &d, "div", &[("contenteditable", "true")]);
    let e = html(&mut b, &d, "span", &[("contenteditable", "false")]);
    b.establish_parent(&e, &host).unwrap();
    let c = b.finish().unwrap();
    let r = query(
        &c,
        &context(&c),
        &c.subject_id(&e).unwrap(),
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        5,
    );
    assert!(matches!(
        r.results()[0].fact(),
        HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Inapplicable)
    ));
}
#[test]
fn enabled_document_child_is_host_even_with_false_contenteditable() {
    let (mut b, d) = builder();
    let e = html(
        &mut b,
        &d,
        "div",
        &[("contenteditable", "false"), ("autocapitalize", "words")],
    );
    b.establish_parent(&e, &d).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let enabled = design(&c, &d, HtmlDesignMode::Enabled);
    let r = query(
        &c,
        &enabled,
        &id,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        5,
    );
    assert_eq!(used(&r).selected_subject(), Some(&id));
    assert_eq!(used(&r).own_hint().unwrap().idl_value(), "words");
    let disabled = design(&c, &d, HtmlDesignMode::Disabled);
    let r = query(
        &c,
        &disabled,
        &id,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        5,
    );
    assert!(matches!(
        r.results()[0].fact(),
        HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Inapplicable)
    ));
    let unknown = design(&c, &d, HtmlDesignMode::Unavailable);
    let r = query(
        &c,
        &unknown,
        &id,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        5,
    );
    assert!(matches!(
        r.results()[0].fact(),
        HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::DocumentDesignMode(_)
        ))
    ));
    assert_ne!(enabled.identity(), disabled.identity());
}
#[test]
fn unknown_self_host_cannot_be_bypassed_by_false_editable_or_local_hint() {
    let (mut b, d) = builder();
    let e = html(
        &mut b,
        &d,
        "div",
        &[("contenteditable", "false"), ("autocapitalize", "words")],
    );
    b.establish_parent(&e, &d).unwrap();
    let c = b.finish().unwrap();
    let r = query(
        &c,
        &context(&c),
        &c.subject_id(&e).unwrap(),
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        5,
    );
    assert!(matches!(
        r.results()[0].fact(),
        HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::DocumentDesignMode(_)
        ))
    ));
    let (c, id) = fixture("div", &[("autocapitalize", "words")], false);
    let r = query(
        &c,
        &context(&c),
        &id,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        5,
    );
    assert!(matches!(
        r.results()[0].fact(),
        HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::Parent(_)
        ))
    ));
}
#[test]
fn svg_and_math_entry_editability_consumes_actual_html_host() {
    for (ns, local) in [(Namespace::svg(), "svg"), (Namespace::mathml(), "math")] {
        let (mut b, d) = builder();
        let h = html(
            &mut b,
            &d,
            "div",
            &[("contenteditable", "true"), ("autocapitalize", "words")],
        );
        let e = expanded(&mut b, &d, ns, local, OrderedAttributes::new([]).unwrap());
        b.establish_parent(&e, &h).unwrap();
        let c = b.finish().unwrap();
        let r = query(
            &c,
            &context(&c),
            &c.subject_id(&e).unwrap(),
            HtmlSemanticQuery::UsedAutocapitalizationHint,
            5,
        );
        assert_eq!(
            used(&r).selected_subject(),
            Some(&c.subject_id(&h).unwrap())
        );
        assert_eq!(used(&r).own_hint().unwrap().idl_value(), "words");
    }
}
#[test]
fn writing_local_decisions_and_roots_have_source_literal_idl_values() {
    for (raw, expected) in [
        (None, "true"),
        (Some("false"), "false"),
        (Some("FaLsE"), "false"),
        (Some("true"), "true"),
        (Some(""), "true"),
        (Some("invalid"), "true"),
        (Some(" false "), "true"),
    ] {
        let (c, id) = fixture(
            "div",
            &raw.map(|v| vec![("writingsuggestions", v)])
                .unwrap_or_default(),
            raw.is_none(),
        );
        let r = query(
            &c,
            &context(&c),
            &id,
            HtmlSemanticQuery::ComputedWritingSuggestions,
            20,
        );
        let v = writing(&r);
        assert_eq!(v.idl_value(), expected);
        assert_eq!(v.selecting_subject(), &id);
        assert_eq!(
            v.selecting_attribute().map(|a| a.value().is_empty()),
            raw.map(str::is_empty)
        );
    }
}
#[test]
fn writing_parent_false_inherits_through_absent_foreign_and_html_elements() {
    let (mut b, d) = builder();
    let top = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "g",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "writingsuggestions"),
            "false",
        )])
        .unwrap(),
    );
    let foreign = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "foreignObject",
        OrderedAttributes::new([]).unwrap(),
    );
    let middle = html(&mut b, &d, "div", &[]);
    let e = html(&mut b, &d, "span", &[]);
    b.establish_parent(&e, &middle).unwrap();
    b.establish_parent(&middle, &foreign).unwrap();
    b.establish_parent(&foreign, &top).unwrap();
    let c = b.finish().unwrap();
    let r = query(
        &c,
        &context(&c),
        &c.subject_id(&e).unwrap(),
        HtmlSemanticQuery::ComputedWritingSuggestions,
        5,
    );
    assert_eq!(writing(&r).idl_value(), "false");
    assert_eq!(
        writing(&r).selecting_subject(),
        &c.subject_id(&top).unwrap()
    );
    assert_eq!(
        writing(&r).recognition(),
        HtmlAttributeRecognition::Keyword {
            matched: "false",
            canonical: "false"
        }
    );
    let captured = c.node(&c.subject_id(&top).unwrap()).unwrap();
    let HtmlCapturedNodeKind::Element(direct) = captured.kind() else {
        panic!("element")
    };
    let intrinsic = HtmlValueInterpreter::new(5)
        .writingsuggestions(direct.name(), direct.attributes().iter().next())
        .unwrap();
    assert_eq!(
        intrinsic.applicability(),
        HtmlAttributeApplicability::ForeignElementNamespace
    );
    assert!(intrinsic.intrinsic().is_none());
}
#[test]
fn writing_explicit_true_empty_invalid_and_false_skip_unavailable_parents() {
    for (raw, expected) in [
        ("true", "true"),
        ("", "true"),
        ("invalid", "true"),
        ("false", "false"),
    ] {
        let (c, id) = fixture("div", &[("writingsuggestions", raw)], false);
        assert_eq!(
            writing(&query(
                &c,
                &context(&c),
                &id,
                HtmlSemanticQuery::ComputedWritingSuggestions,
                raw.len()
            ))
            .idl_value(),
            expected
        );
    }
    let (c, id) = fixture("div", &[], false);
    let r = query(
        &c,
        &context(&c),
        &id,
        HtmlSemanticQuery::ComputedWritingSuggestions,
        0,
    );
    assert!(matches!(
        r.results()[0].fact(),
        HtmlSemanticFact::ComputedWritingSuggestions(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::Parent(_)
        ))
    ));
}
#[test]
fn writing_non_element_parent_terminates_without_designmode_or_host_edges() {
    let (mut b, d) = builder();
    let host = html(
        &mut b,
        &d,
        "template",
        &[
            ("writingsuggestions", "false"),
            ("contenteditable", "true"),
            ("autocapitalize", "words"),
        ],
    );
    let fragment = b.add_fragment(&d).unwrap();
    b.establish_template_contents(&host, &fragment).unwrap();
    let e = html(&mut b, &d, "div", &[]);
    b.establish_parent(&e, &fragment).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = context(&c);
    assert_eq!(
        writing(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::ComputedWritingSuggestions,
            0
        ))
        .idl_value(),
        "true"
    );
    assert!(matches!(
        query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::UsedAutocapitalizationHint,
            0
        )
        .results()[0]
            .fact(),
        HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Inapplicable)
    ));
}
#[test]
fn exact_attribute_names_ignore_namespace_and_case_lookalikes() {
    let (mut b, d) = builder();
    let e = expanded(
        &mut b,
        &d,
        Namespace::html(),
        "div",
        OrderedAttributes::new([
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "autocapitalize"),
                "oversized ignored",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "AUTOCORRECT"),
                "oversized ignored",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "WRITINGSUGGESTIONS"),
                "oversized ignored",
            ),
        ])
        .unwrap(),
    );
    b.establish_root(&e).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = context(&c);
    assert_eq!(
        own(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::OwnAutocapitalizationHint,
            0
        ))
        .idl_value(),
        ""
    );
    assert!(
        correct(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::UsedAutocorrectionState,
            0
        ))
        .idl_value()
    );
    assert_eq!(
        writing(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::ComputedWritingSuggestions,
            0
        ))
        .idl_value(),
        "true"
    );
}
#[test]
fn surrogate_content_is_retained_exactly_with_independent_invalid_defaults() {
    let (mut b, d) = builder();
    let units = vec![0xd800, 0x20, 0xdc00];
    let attrs = OrderedAttributes::new(
        ["autocapitalize", "autocorrect", "writingsuggestions"].map(|n| {
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, n),
                DocumentString::from_utf16(units.clone()),
            )
        }),
    )
    .unwrap();
    let e = expanded(&mut b, &d, Namespace::html(), "div", attrs);
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = context(&c);
    let r = query(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::OwnAutocapitalizationHint,
        3,
    );
    assert_eq!(own(&r).idl_value(), "sentences");
    assert_eq!(own(&r).own_attribute().unwrap().value().code_units(), units);
    assert_eq!(
        own(&r).own_report().unwrap().recognition(),
        HtmlAttributeRecognition::Unrecognized
    );
    assert!(
        correct(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::UsedAutocorrectionState,
            3
        ))
        .idl_value()
    );
    let r = query(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::ComputedWritingSuggestions,
        3,
    );
    assert_eq!(writing(&r).idl_value(), "true");
    assert_eq!(
        writing(&r)
            .selecting_attribute()
            .unwrap()
            .value()
            .code_units(),
        units
    );
}
#[test]
fn irrelevant_values_are_unread_but_consulted_values_obey_whole_raw_caps() {
    let (c, id) = fixture(
        "div",
        &[
            ("autocapitalize", "words"),
            ("autocorrect", "off"),
            ("writingsuggestions", "true"),
            ("contenteditable", "uninspected content"),
            ("spellcheck", "uninspected content"),
            ("value", "uninspected content"),
        ],
        false,
    );
    let cx = context(&c);
    for (q, cap) in [
        (HtmlSemanticQuery::OwnAutocapitalizationHint, 5),
        (HtmlSemanticQuery::UsedAutocorrectionState, 3),
        (HtmlSemanticQuery::ComputedWritingSuggestions, 4),
    ] {
        query(&c, &cx, &id, q, cap);
        let f = session(&c, &cx, limits(), cap - 1, HtmlSemanticCancellation::new())
            .query(&[req(&id, q)])
            .unwrap_err();
        assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    }
}
#[test]
fn foreign_and_non_element_getter_targets_are_inapplicable() {
    let (mut b, d) = builder();
    let e = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "div",
        OrderedAttributes::new([]).unwrap(),
    );
    b.establish_root(&e).unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c);
    for id in [c.subject_id(&e).unwrap(), c.subject_id(&d).unwrap()] {
        for q in [
            HtmlSemanticQuery::OwnAutocapitalizationHint,
            HtmlSemanticQuery::UsedAutocorrectionState,
            HtmlSemanticQuery::ComputedWritingSuggestions,
            HtmlSemanticQuery::UsedAutocapitalizationHint,
        ] {
            let r = query(&c, &cx, &id, q, 0);
            assert!(matches!(
                r.results()[0].fact(),
                HtmlSemanticFact::OwnAutocapitalizationHint(HtmlSemanticValue::Inapplicable)
                    | HtmlSemanticFact::UsedAutocorrectionState(HtmlSemanticValue::Inapplicable)
                    | HtmlSemanticFact::ComputedWritingSuggestions(HtmlSemanticValue::Inapplicable)
                    | HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Inapplicable)
            ));
            assert_eq!(r.usage().nested_output_records(), 0);
        }
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
        if resource == HtmlSemanticResource::ScratchRecords {
            cap
        } else {
            l.scratch_records()
        },
        l.integer_digits(),
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
fn resource_value(usage: HtmlSemanticUsage, resource: HtmlSemanticResource) -> usize {
    match resource {
        HtmlSemanticResource::Work => usage.work(),
        HtmlSemanticResource::OutputRecords => usage.output_records(),
        HtmlSemanticResource::ScratchRecords => usage.peak_scratch_records(),
        HtmlSemanticResource::AllocatedBytes => usage.allocated_bytes(),
        HtmlSemanticResource::NestedOutputRecords => usage.nested_output_records(),
        _ => panic!("not a selected resource"),
    }
}
#[test]
fn exact_and_one_short_physical_caps_cover_each_real_readback() {
    let (c, id) = fixture(
        "input",
        &[
            ("type", "text"),
            ("autocapitalize", "words"),
            ("autocorrect", "off"),
            ("writingsuggestions", "false"),
        ],
        false,
    );
    let cx = context(&c);
    for q in [
        HtmlSemanticQuery::OwnAutocapitalizationHint,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        HtmlSemanticQuery::UsedAutocorrectionState,
        HtmlSemanticQuery::ComputedWritingSuggestions,
    ] {
        let r = query(&c, &cx, &id, q, 5);
        for res in [
            HtmlSemanticResource::Work,
            HtmlSemanticResource::OutputRecords,
            HtmlSemanticResource::NestedOutputRecords,
            HtmlSemanticResource::AllocatedBytes,
        ] {
            let n = resource_value(r.usage(), res);
            assert!(n > 0);
            session(&c, &cx, capped(res, n), 5, HtmlSemanticCancellation::new())
                .query(&[req(&id, q)])
                .unwrap();
            let mut ss = session(
                &c,
                &cx,
                capped(res, n - 1),
                5,
                HtmlSemanticCancellation::new(),
            );
            let f = ss.query(&[req(&id, q)]).unwrap_err();
            assert!(
                matches!(f.kind(),HtmlSemanticFailureKind::LimitExceeded{resource,..}if *resource==res),
                "{q:?} {res:?} {f:?}"
            );
            assert_eq!(ss.query(&[]).unwrap_err(), f);
            assert_eq!(ss.usage().scratch_records(), 0);
            assert_eq!(f.qualification().unwrap().capture(), c.identity());
        }
    }
}
#[test]
fn used_hint_counts_both_nested_records_and_late_outer_failure_retains_cost() {
    let (c, id) = fixture("textarea", &[("autocapitalize", "words")], false);
    let cx = context(&c);
    let o = query(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::OwnAutocapitalizationHint,
        5,
    );
    let u = query(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        5,
    );
    assert_eq!(o.usage().nested_output_records(), 1);
    assert_eq!(u.usage().nested_output_records(), 2);
    assert!(u.usage().allocated_bytes() > o.usage().allocated_bytes());
    let mut ss = session(
        &c,
        &cx,
        capped(HtmlSemanticResource::NestedOutputRecords, 1),
        5,
        HtmlSemanticCancellation::new(),
    );
    let f = ss
        .query(&[req(&id, HtmlSemanticQuery::UsedAutocapitalizationHint)])
        .unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(ss.usage().nested_output_records(), 1);
    assert!(ss.usage().allocated_bytes() > 0);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
#[test]
fn reset_owner_scratch_is_finite_and_released_on_work_stop() {
    let (mut b, d) = builder();
    let f = html(
        &mut b,
        &d,
        "form",
        &[("id", "target"), ("autocapitalize", "words")],
    );
    let e = html(&mut b, &d, "input", &[("form", "target")]);
    b.establish_children(&f, &[]).unwrap();
    b.establish_children(&e, &[]).unwrap();
    b.establish_children(&d, &[f, e.clone()]).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = with_owners(
        &c,
        vec![(id.clone(), HtmlFormAssociationState::ResetAgainstCapture)],
    );
    let r = query(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        6,
    );
    let peak = r.usage().peak_scratch_records();
    assert!(peak > 0);
    session(
        &c,
        &cx,
        capped(HtmlSemanticResource::ScratchRecords, peak),
        6,
        HtmlSemanticCancellation::new(),
    )
    .query(&[req(&id, HtmlSemanticQuery::UsedAutocapitalizationHint)])
    .unwrap();
    let mut ss = session(
        &c,
        &cx,
        capped(HtmlSemanticResource::ScratchRecords, peak - 1),
        6,
        HtmlSemanticCancellation::new(),
    );
    let f = ss
        .query(&[req(&id, HtmlSemanticQuery::UsedAutocapitalizationHint)])
        .unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::ScratchRecords,
            ..
        }
    ));
    assert_eq!(ss.usage().scratch_records(), 0);
    for work in [1, 200, 500, 1000] {
        let mut ss = session(
            &c,
            &cx,
            capped(HtmlSemanticResource::Work, work),
            6,
            HtmlSemanticCancellation::new(),
        );
        if ss
            .query(&[req(&id, HtmlSemanticQuery::UsedAutocapitalizationHint)])
            .is_err()
        {
            assert_eq!(ss.usage().scratch_records(), 0);
        }
    }
}
#[test]
fn duplicate_and_late_batch_failure_have_cumulative_usage_and_no_prefix() {
    let (c, id) = fixture(
        "div",
        &[("autocapitalize", "words"), ("writingsuggestions", "false")],
        false,
    );
    let cx = context(&c);
    let request = req(&id, HtmlSemanticQuery::OwnAutocapitalizationHint);
    let one = query(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::OwnAutocapitalizationHint,
        5,
    );
    let two = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new())
        .query(&[request.clone(), request.clone()])
        .unwrap();
    assert_eq!(two.usage().nested_output_records(), 2);
    assert!(two.usage().work() > one.usage().work());
    assert!(two.usage().allocated_bytes() > one.usage().allocated_bytes());
    let mut ss = session(
        &c,
        &cx,
        capped(HtmlSemanticResource::NestedOutputRecords, 1),
        5,
        HtmlSemanticCancellation::new(),
    );
    let f = ss.query(&[request.clone(), request]).unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::NestedOutputRecords,
            ..
        }
    ));
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    assert_eq!(ss.usage().scratch_records(), 0);
}
#[test]
fn cancellation_and_first_stop_retain_exact_session_qualification() {
    let (c, id) = fixture("div", &[("writingsuggestions", "false")], false);
    let cx = context(&c);
    let cancel = HtmlSemanticCancellation::new();
    let mut ss = session(&c, &cx, limits(), 5, cancel.clone());
    ss.query(&[req(&id, HtmlSemanticQuery::ComputedWritingSuggestions)])
        .unwrap();
    cancel.cancel();
    let f = ss
        .query(&[req(&id, HtmlSemanticQuery::ComputedWritingSuggestions)])
        .unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(f.qualification().unwrap().capture(), c.identity());
    assert_eq!(f.qualification().unwrap().context(), cx.identity());
    assert_eq!(
        f.qualification().unwrap().host_revision(),
        c.host_facts_revision()
    );
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    let cancel = HtmlSemanticCancellation::new();
    let mut ss = session(
        &c,
        &cx,
        capped(HtmlSemanticResource::Work, 0),
        5,
        cancel.clone(),
    );
    let first = ss
        .query(&[req(&id, HtmlSemanticQuery::ComputedWritingSuggestions)])
        .unwrap_err();
    cancel.cancel();
    assert_eq!(ss.query(&[]).unwrap_err(), first);
}
#[test]
fn changes_to_actual_owner_and_host_configuration_qualify_recomputation() {
    let (mut b, d) = builder();
    let f = html(&mut b, &d, "form", &[("autocapitalize", "words")]);
    let e = html(&mut b, &d, "input", &[]);
    b.establish_children(&d, &[f.clone(), e.clone()]).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let fid = c.subject_id(&f).unwrap();
    let owner = with_owners(
        &c,
        vec![(
            id.clone(),
            HtmlFormAssociationState::EstablishedCurrent(Some(fid)),
        )],
    );
    let null = with_owners(
        &c,
        vec![(
            id.clone(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
    );
    let a = query(
        &c,
        &owner,
        &id,
        HtmlSemanticQuery::OwnAutocapitalizationHint,
        5,
    );
    let b = query(
        &c,
        &null,
        &id,
        HtmlSemanticQuery::OwnAutocapitalizationHint,
        5,
    );
    assert_eq!(own(&a).idl_value(), "words");
    assert_eq!(own(&b).idl_value(), "");
    assert_ne!(a.qualification().context(), b.qualification().context());
    assert_eq!(
        a.qualification().editability_source().revision(),
        "85de9ff4a9816139d06fc2dc39d5dacdc456fddc"
    );
    let (other, other_id) = fixture("div", &[("writingsuggestions", "true")], true);
    let other_cx = context(&other);
    let r = query(
        &other,
        &other_cx,
        &other_id,
        HtmlSemanticQuery::ComputedWritingSuggestions,
        4,
    );
    assert_ne!(r.qualification().capture(), a.qualification().capture());
    let f = session(&c, &owner, limits(), 5, HtmlSemanticCancellation::new())
        .query(&[req(&other_id, HtmlSemanticQuery::OwnAutocapitalizationHint)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Capture(_)));
}

#[test]
fn own_nonempty_overrides_form_and_owner_value_caps_are_conditional() {
    let (mut b, d) = builder();
    let f = html(
        &mut b,
        &d,
        "form",
        &[
            ("autocapitalize", "a very large form value"),
            ("autocorrect", "a very large form value"),
        ],
    );
    let a = html(
        &mut b,
        &d,
        "input",
        &[("autocapitalize", "words"), ("autocorrect", "off")],
    );
    let inherited = html(&mut b, &d, "textarea", &[("autocapitalize", "")]);
    b.establish_children(&d, &[f.clone(), a.clone(), inherited.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let fid = c.subject_id(&f).unwrap();
    let aid = c.subject_id(&a).unwrap();
    let iid = c.subject_id(&inherited).unwrap();
    let cx = with_owners(
        &c,
        vec![
            (
                aid.clone(),
                HtmlFormAssociationState::EstablishedCurrent(Some(fid.clone())),
            ),
            (
                iid.clone(),
                HtmlFormAssociationState::EstablishedCurrent(Some(fid)),
            ),
        ],
    );
    assert_eq!(
        own(&query(
            &c,
            &cx,
            &aid,
            HtmlSemanticQuery::OwnAutocapitalizationHint,
            5
        ))
        .idl_value(),
        "words"
    );
    assert!(
        !correct(&query(
            &c,
            &cx,
            &aid,
            HtmlSemanticQuery::UsedAutocorrectionState,
            5
        ))
        .idl_value()
    );
    for q in [
        HtmlSemanticQuery::OwnAutocapitalizationHint,
        HtmlSemanticQuery::UsedAutocorrectionState,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
    ] {
        let f = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new())
            .query(&[req(&iid, q)])
            .unwrap_err();
        assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    }
}
#[test]
fn source_type_and_editability_producers_keep_actual_raw_admission() {
    let (c, id) = fixture(
        "input",
        &[
            ("type", "password"),
            ("autocapitalize", "words"),
            ("autocorrect", "off"),
        ],
        false,
    );
    let cx = context(&c);
    for q in [
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        HtmlSemanticQuery::UsedAutocorrectionState,
    ] {
        let f = session(&c, &cx, limits(), 7, HtmlSemanticCancellation::new())
            .query(&[req(&id, q)])
            .unwrap_err();
        assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    }
    let (c, id) = fixture(
        "div",
        &[
            ("contenteditable", "an oversized invalid contenteditable"),
            ("autocapitalize", "words"),
        ],
        true,
    );
    let cx = context(&c);
    assert_eq!(
        own(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::OwnAutocapitalizationHint,
            5
        ))
        .idl_value(),
        "words"
    );
    let f = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new())
        .query(&[req(&id, HtmlSemanticQuery::UsedAutocapitalizationHint)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
}
#[test]
fn own_empty_input_can_inherit_while_autocorrect_empty_forces_local_on() {
    let (mut b, d) = builder();
    let f = html(
        &mut b,
        &d,
        "form",
        &[("autocapitalize", "words"), ("autocorrect", "off")],
    );
    let e = html(
        &mut b,
        &d,
        "input",
        &[("autocapitalize", ""), ("autocorrect", "")],
    );
    b.establish_children(&d, &[f.clone(), e.clone()]).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = with_owners(
        &c,
        vec![(
            id.clone(),
            HtmlFormAssociationState::EstablishedCurrent(Some(c.subject_id(&f).unwrap())),
        )],
    );
    assert_eq!(
        own(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::OwnAutocapitalizationHint,
            5
        ))
        .idl_value(),
        "words"
    );
    let r = query(&c, &cx, &id, HtmlSemanticQuery::UsedAutocorrectionState, 0);
    assert!(correct(&r).idl_value());
    assert!(correct(&r).owner().is_none());
    assert_eq!(
        correct(&r).own_report().unwrap().recognition(),
        HtmlAttributeRecognition::Empty
    );
}
#[test]
fn writing_local_true_states_override_false_parent_and_skip_its_cap() {
    for local in ["true", "", "invalid", " false "] {
        let (mut b, d) = builder();
        let parent = html(&mut b, &d, "div", &[("writingsuggestions", "false")]);
        let e = html(&mut b, &d, "span", &[("writingsuggestions", local)]);
        b.establish_parent(&e, &parent).unwrap();
        let c = b.finish().unwrap();
        let id = c.subject_id(&e).unwrap();
        let r = query(
            &c,
            &context(&c),
            &id,
            HtmlSemanticQuery::ComputedWritingSuggestions,
            local.len(),
        );
        assert_eq!(writing(&r).idl_value(), "true");
        assert_eq!(writing(&r).selecting_subject(), &id);
    }
    let (mut b, d) = builder();
    let parent = html(
        &mut b,
        &d,
        "div",
        &[("writingsuggestions", "a large unread parent value")],
    );
    let e = html(&mut b, &d, "span", &[("writingsuggestions", "false")]);
    b.establish_parent(&e, &parent).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        writing(&query(
            &c,
            &context(&c),
            &c.subject_id(&e).unwrap(),
            HtmlSemanticQuery::ComputedWritingSuggestions,
            5
        ))
        .idl_value(),
        "false"
    );
}
#[test]
fn consulted_foreign_parent_content_obeys_full_raw_cap() {
    let (mut b, d) = builder();
    let p = expanded(
        &mut b,
        &d,
        Namespace::svg(),
        "foreignObject",
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::none(), None, "writingsuggestions"),
            "FaLsE",
        )])
        .unwrap(),
    );
    let e = html(&mut b, &d, "div", &[]);
    b.establish_parent(&e, &p).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = context(&c);
    assert_eq!(
        writing(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::ComputedWritingSuggestions,
            5
        ))
        .idl_value(),
        "false"
    );
    let f = session(&c, &cx, limits(), 4, HtmlSemanticCancellation::new())
        .query(&[req(&id, HtmlSemanticQuery::ComputedWritingSuggestions)])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
}
#[test]
fn nearer_unknown_relation_cannot_select_an_unrelated_farther_host() {
    let (mut b, d) = builder();
    let _outer = html(
        &mut b,
        &d,
        "div",
        &[("contenteditable", "true"), ("autocapitalize", "words")],
    );
    let middle = html(&mut b, &d, "section", &[]);
    let e = html(&mut b, &d, "span", &[]);
    b.establish_parent(&e, &middle).unwrap();
    let c = b.finish().unwrap();
    let mid = c.subject_id(&middle).unwrap();
    let r = query(
        &c,
        &context(&c),
        &c.subject_id(&e).unwrap(),
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        5,
    );
    assert!(
        matches!(r.results()[0].fact(),HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(id))) if *id==mid)
    );
}
#[test]
fn shadow_host_attributes_are_not_ordinary_inheritance() {
    let (mut b, d) = builder();
    let host = html(
        &mut b,
        &d,
        "div",
        &[
            ("contenteditable", "true"),
            ("autocapitalize", "words"),
            ("writingsuggestions", "false"),
        ],
    );
    let shadow = b
        .add_shadow_root(
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
        .unwrap();
    let e = html(&mut b, &d, "span", &[]);
    b.establish_parent(&e, &shadow).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = context(&c);
    assert_eq!(
        writing(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::ComputedWritingSuggestions,
            0
        ))
        .idl_value(),
        "true"
    );
    assert!(matches!(
        query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::UsedAutocapitalizationHint,
            0
        )
        .results()[0]
            .fact(),
        HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Inapplicable)
    ));
}
#[test]
fn document_parent_terminates_writing_without_requiring_designmode() {
    let (mut b, d) = builder();
    let e = html(&mut b, &d, "div", &[]);
    b.establish_parent(&e, &d).unwrap();
    let c = b.finish().unwrap();
    let r = query(
        &c,
        &context(&c),
        &c.subject_id(&e).unwrap(),
        HtmlSemanticQuery::ComputedWritingSuggestions,
        0,
    );
    assert_eq!(writing(&r).idl_value(), "true");
    assert_eq!(
        writing(&r).local_state(),
        HtmlWritingSuggestionsState::Default
    );
    assert_eq!(writing(&r).recognition(), HtmlAttributeRecognition::Missing);
}
#[test]
fn uppercase_input_name_does_not_take_native_type_suppression_path() {
    let (c, id) = fixture(
        "INPUT",
        &[
            ("type", "a large ignored attribute"),
            ("autocapitalize", "words"),
        ],
        true,
    );
    let cx = context(&c);
    assert_eq!(
        own(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::OwnAutocapitalizationHint,
            5
        ))
        .idl_value(),
        "words"
    );
    assert!(matches!(
        query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::UsedAutocapitalizationHint,
            0
        )
        .results()[0]
            .fact(),
        HtmlSemanticFact::UsedAutocapitalizationHint(HtmlSemanticValue::Inapplicable)
    ));
}
#[test]
fn later_raw_failure_does_not_publish_successful_prefix_and_keeps_allocated_work() {
    let (mut b, d) = builder();
    let a = html(&mut b, &d, "div", &[("writingsuggestions", "false")]);
    let bad = html(
        &mut b,
        &d,
        "div",
        &[("writingsuggestions", "huge later content")],
    );
    let c = b.finish().unwrap();
    let cx = context(&c);
    let aid = c.subject_id(&a).unwrap();
    let bid = c.subject_id(&bad).unwrap();
    let mut ss = session(&c, &cx, limits(), 5, HtmlSemanticCancellation::new());
    let f = ss
        .query(&[
            req(&aid, HtmlSemanticQuery::ComputedWritingSuggestions),
            req(&bid, HtmlSemanticQuery::ComputedWritingSuggestions),
        ])
        .unwrap_err();
    assert!(matches!(f.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(ss.usage().nested_output_records(), 1);
    assert!(ss.usage().allocated_bytes() > 0);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}

#[test]
fn long_default_walk_is_bounded_and_sticky_cancelled_before_entry() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", &[("writingsuggestions", "false")]);
    let mut current = root.clone();
    for _ in 0..40 {
        let child = html(&mut b, &d, "span", &[]);
        b.establish_parent(&child, &current).unwrap();
        current = child;
    }
    let c = b.finish().unwrap();
    let id = c.subject_id(&current).unwrap();
    let cx = context(&c);
    let r = query(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::ComputedWritingSuggestions,
        5,
    );
    assert_eq!(writing(&r).idl_value(), "false");
    assert_eq!(
        writing(&r).selecting_subject(),
        &c.subject_id(&root).unwrap()
    );
    assert_eq!(r.usage().peak_scratch_records(), 0);
    assert_eq!(r.usage().nested_output_records(), 1);
    let mut ss = session(
        &c,
        &cx,
        capped(HtmlSemanticResource::Work, r.usage().work() - 1),
        5,
        HtmlSemanticCancellation::new(),
    );
    let f = ss
        .query(&[req(&id, HtmlSemanticQuery::ComputedWritingSuggestions)])
        .unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(ss.query(&[]).unwrap_err(), f);
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let mut ss = session(&c, &cx, limits(), 5, cancel);
    let f = ss
        .query(&[req(&id, HtmlSemanticQuery::ComputedWritingSuggestions)])
        .unwrap_err();
    assert_eq!(f.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(ss.usage().nested_output_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
#[test]
fn changed_global_revision_and_input_cap_remain_in_result_and_failure_keys() {
    let (mut b, d) = builder();
    let e = html(
        &mut b,
        &d,
        "div",
        &[("contenteditable", "false"), ("autocapitalize", "words")],
    );
    b.establish_parent(&e, &d).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let yes = design(&c, &d, HtmlDesignMode::Enabled);
    let no = design(&c, &d, HtmlDesignMode::Disabled);
    let a = query(
        &c,
        &yes,
        &id,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        5,
    );
    let b = query(
        &c,
        &no,
        &id,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        5,
    );
    assert_ne!(
        a.qualification().global_inheritance_revision(),
        b.qualification().global_inheritance_revision()
    );
    assert_eq!(a.qualification().attribute_input_limit(), 5);
    let mut ss = session(&c, &yes, limits(), 4, HtmlSemanticCancellation::new());
    let f = ss
        .query(&[req(&id, HtmlSemanticQuery::UsedAutocapitalizationHint)])
        .unwrap_err();
    assert_eq!(
        f.qualification().unwrap().global_inheritance_revision(),
        yes.global_inheritance_revision()
    );
    assert_eq!(f.qualification().unwrap().attribute_input_limit(), 4);
    assert_eq!(f.qualification().unwrap().context(), yes.identity());
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
#[test]
fn work_failure_after_real_reset_traversal_releases_all_scratch() {
    let (mut b, d) = builder();
    let f = html(
        &mut b,
        &d,
        "form",
        &[("id", "target"), ("autocapitalize", "words")],
    );
    let e = html(&mut b, &d, "textarea", &[("form", "target")]);
    b.establish_children(&f, &[]).unwrap();
    b.establish_children(&e, &[]).unwrap();
    b.establish_children(&d, &[f, e.clone()]).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    let cx = with_owners(
        &c,
        vec![(id.clone(), HtmlFormAssociationState::ResetAgainstCapture)],
    );
    let full = query(
        &c,
        &cx,
        &id,
        HtmlSemanticQuery::UsedAutocapitalizationHint,
        6,
    );
    let mut ss = session(
        &c,
        &cx,
        capped(HtmlSemanticResource::Work, full.usage().work() - 1),
        6,
        HtmlSemanticCancellation::new(),
    );
    let f = ss
        .query(&[req(&id, HtmlSemanticQuery::UsedAutocapitalizationHint)])
        .unwrap_err();
    assert!(matches!(
        f.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert!(ss.usage().peak_scratch_records() > 0);
    assert_eq!(ss.usage().scratch_records(), 0);
    assert_eq!(ss.query(&[]).unwrap_err(), f);
}
