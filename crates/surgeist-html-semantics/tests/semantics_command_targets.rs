use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlAssociatedElement, HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedChildIds, HtmlCapturedIsValue,
    HtmlCapturedShadowProperties, HtmlCommandValidity, HtmlControlFacts, HtmlControlFactsLimits,
    HtmlElementReferenceFact, HtmlElementReferenceFacts, HtmlElementReferenceFactsError,
    HtmlElementReferenceFactsLimits, HtmlElementReferenceKey, HtmlElementReferenceOrigin,
    HtmlExplicitElementReference, HtmlFormAssociation, HtmlFormAssociationState,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextError,
    HtmlSemanticContextInput, HtmlSemanticFact, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticResource, HtmlSemanticResult, HtmlSemanticSession, HtmlSemanticValue,
};
#[path = "common/builder.rs"]
mod shared_builder;
use shared_builder::builder;
#[path = "common/session.rs"]
mod shared_session;
use shared_session::session;
use surgeist_html::*;

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
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    element(b, d, Namespace::html(), local, attrs)
}
fn children(
    b: &mut HtmlCaptureBuilder,
    p: &HtmlCaptureBuilderSubject,
    cs: &[HtmlCaptureBuilderSubject],
) {
    b.establish_children(p, cs).unwrap();
}
fn facts(c: &HtmlCapture, refs: Vec<HtmlElementReferenceFact>) -> HtmlElementReferenceFacts {
    HtmlElementReferenceFacts::new(
        c,
        c.host_facts_revision().clone(),
        refs,
        HtmlElementReferenceFactsLimits::new(100, 1000000),
    )
    .unwrap()
}
fn reference(
    c: &HtmlCapture,
    source: &HtmlCaptureBuilderSubject,
    key: HtmlElementReferenceKey,
    state: HtmlExplicitElementReference,
) -> HtmlElementReferenceFact {
    HtmlElementReferenceFact::new(c.subject_id(source).unwrap(), key, state)
}
fn context(
    c: &HtmlCapture,
    refs: Option<HtmlElementReferenceFacts>,
    associations: Vec<HtmlFormAssociation>,
) -> HtmlSemanticContext {
    let mut input = HtmlSemanticContextInput::new(vec![]);
    if let Some(refs) = refs {
        input = input.with_element_references(refs);
    }
    if !associations.is_empty() {
        input = input.with_controls(
            HtmlControlFacts::new(
                c,
                c.host_facts_revision().clone(),
                associations,
                vec![],
                vec![],
                vec![],
                HtmlControlFactsLimits::new(100, 1000000),
            )
            .unwrap(),
        );
    }
    HtmlSemanticContext::new(c, c.host_facts_revision().clone(), input, 0).unwrap()
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 1000, 1000000, 1000000, 1000)
}

fn request(id: &HtmlCaptureSubjectId, q: HtmlSemanticQuery) -> HtmlSemanticRequest {
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
        .query(&[request(id, q)])
        .unwrap()
}
fn associated(r: &HtmlSemanticReport) -> &HtmlAssociatedElement {
    match r.results()[0].fact() {
        HtmlSemanticFact::CommandForElement(HtmlSemanticValue::Known(v))
        | HtmlSemanticFact::PopoverTargetAssociatedElement(HtmlSemanticValue::Known(v)) => v,
        other => panic!("{other:?}"),
    }
}
fn validity(r: &HtmlSemanticReport) -> &HtmlCommandValidity {
    match r.results()[0].fact() {
        HtmlSemanticFact::CommandValidity(HtmlSemanticValue::Known(v)) => v,
        other => panic!("{other:?}"),
    }
}
fn popover(r: &HtmlSemanticReport) -> Option<&HtmlCaptureSubjectId> {
    match r.results()[0].fact() {
        HtmlSemanticFact::PopoverTargetElement(HtmlSemanticValue::Known(v)) => v.as_ref(),
        other => panic!("{other:?}"),
    }
}
fn missing(r: &HtmlSemanticReport) -> &HtmlSemanticMissing {
    match r.results()[0].fact() {
        HtmlSemanticFact::CommandForElement(HtmlSemanticValue::Missing(v))
        | HtmlSemanticFact::PopoverTargetAssociatedElement(HtmlSemanticValue::Missing(v))
        | HtmlSemanticFact::CommandValidity(HtmlSemanticValue::Missing(v))
        | HtmlSemanticFact::PopoverTargetElement(HtmlSemanticValue::Missing(v)) => v,
        other => panic!("{other:?}"),
    }
}
#[test]
fn no_record_and_explicit_unavailable_are_missing_even_when_content_absent() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[]);
    b.establish_root(&button).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&button).unwrap();
    for refs in [
        None,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Unavailable,
            )],
        )),
    ] {
        let cx = context(&c, refs, vec![]);
        assert_eq!(
            missing(&query(
                &c,
                &cx,
                &id,
                HtmlSemanticQuery::CommandForElement,
                0
            )),
            &HtmlSemanticMissing::ElementReference {
                subject: id.clone(),
                key: HtmlElementReferenceKey::CommandFor
            }
        );
    }
}
#[test]
fn null_pointer_missing_and_empty_content_prove_null_without_topology() {
    for raw in [vec![], vec![("commandfor", "")]] {
        let (mut b, d) = builder();
        let button = html(&mut b, &d, "button", &raw);
        let c = b.finish().unwrap();
        let id = c.subject_id(&button).unwrap();
        let cx = context(
            &c,
            Some(facts(
                &c,
                vec![reference(
                    &c,
                    &button,
                    HtmlElementReferenceKey::CommandFor,
                    HtmlExplicitElementReference::Null,
                )],
            )),
            vec![],
        );
        let r = query(&c, &cx, &id, HtmlSemanticQuery::CommandForElement, 0);
        assert_eq!(associated(&r).target(), None);
        assert_eq!(
            associated(&r).origin(),
            HtmlElementReferenceOrigin::ContentAttribute
        );
        assert_eq!(associated(&r).raw().is_some(), !raw.is_empty());
    }
}
#[test]
fn first_exact_id_includes_foreign_and_does_not_read_matched_or_later_children() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("commandfor", "Case")]);
    let foreign = element(&mut b, &d, Namespace::svg(), "rect", &[("id", "Case")]);
    let later = html(&mut b, &d, "dialog", &[("id", "Case")]);
    children(&mut b, &button, &[]);
    children(&mut b, &d, &[button.clone(), foreign.clone(), later]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    assert_eq!(
        associated(&query(
            &c,
            &cx,
            &c.subject_id(&button).unwrap(),
            HtmlSemanticQuery::CommandForElement,
            4
        ))
        .target(),
        Some(&c.subject_id(&foreign).unwrap())
    );
}
#[test]
fn id_lookup_has_no_case_whitespace_hash_or_author_id_normalization() {
    for (raw, id, found) in [
        ("Case", "case", false),
        (" target", "target", false),
        ("#target", "target", false),
        ("a b", "a b", true),
        ("", "", false),
    ] {
        let (mut b, d) = builder();
        let button = html(&mut b, &d, "button", &[("commandfor", raw)]);
        let target = html(&mut b, &d, "div", &[("id", id)]);
        children(&mut b, &button, &[]);
        children(&mut b, &target, &[]);
        children(&mut b, &d, &[button.clone(), target.clone()]);
        let c = b.finish().unwrap();
        let cx = context(
            &c,
            Some(facts(
                &c,
                vec![reference(
                    &c,
                    &button,
                    HtmlElementReferenceKey::CommandFor,
                    HtmlExplicitElementReference::Null,
                )],
            )),
            vec![],
        );
        assert_eq!(
            associated(&query(
                &c,
                &cx,
                &c.subject_id(&button).unwrap(),
                HtmlSemanticQuery::CommandForElement,
                100
            ))
            .target(),
            found.then(|| c.subject_id(&target).unwrap()).as_ref()
        );
    }
}
#[test]
fn isolated_utf16_id_is_compared_losslessly() {
    let (mut b, d) = builder();
    let value = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    let make = |b: &mut HtmlCaptureBuilder, local: &str, name: &str| {
        b.add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, local),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, name),
                value.clone(),
            )])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap()
    };
    let button = make(&mut b, "button", "commandfor");
    let target = make(&mut b, "div", "id");
    children(&mut b, &button, &[]);
    children(&mut b, &d, &[button.clone(), target.clone()]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    let r = query(
        &c,
        &cx,
        &c.subject_id(&button).unwrap(),
        HtmlSemanticQuery::CommandForElement,
        3,
    );
    assert_eq!(associated(&r).raw().unwrap().value(), &value);
    assert_eq!(
        associated(&r).target(),
        Some(&c.subject_id(&target).unwrap())
    );
}
#[test]
fn earlier_unavailable_children_influence_first_id_despite_later_witness() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("commandfor", "x")]);
    let gap = html(&mut b, &d, "div", &[]);
    let target = html(&mut b, &d, "div", &[("id", "x")]);
    children(&mut b, &button, &[]);
    children(&mut b, &d, &[button.clone(), gap.clone(), target]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    assert_eq!(
        missing(&query(
            &c,
            &cx,
            &c.subject_id(&button).unwrap(),
            HtmlSemanticQuery::CommandForElement,
            10
        )),
        &HtmlSemanticMissing::OrderedChildren(c.subject_id(&gap).unwrap())
    );
}
#[test]
fn detached_ordinary_tree_resolves_without_connectedness() {
    let (mut b, d) = builder();
    let root = html(&mut b, &d, "div", &[("id", "x")]);
    let button = html(&mut b, &d, "button", &[("commandfor", "x")]);
    children(&mut b, &root, std::slice::from_ref(&button));
    b.establish_root(&root).unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    assert_eq!(
        associated(&query(
            &c,
            &cx,
            &c.subject_id(&button).unwrap(),
            HtmlSemanticQuery::CommandForElement,
            1
        ))
        .target(),
        Some(&c.subject_id(&root).unwrap())
    );
}
#[test]
fn out_of_scope_explicit_pointer_returns_null_without_valid_content_fallback() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("commandfor", "x")]);
    let fallback = html(&mut b, &d, "dialog", &[("id", "x")]);
    let other = html(&mut b, &d, "div", &[]);
    b.establish_root(&other).unwrap();
    children(&mut b, &d, &[button.clone(), fallback]);
    let c = b.finish().unwrap();
    let pointer = c.subject_id(&other).unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Element(pointer.clone()),
            )],
        )),
        vec![],
    );
    let r = query(
        &c,
        &cx,
        &c.subject_id(&button).unwrap(),
        HtmlSemanticQuery::CommandForElement,
        1,
    );
    assert_eq!(associated(&r).target(), None);
    assert_eq!(
        associated(&r).origin(),
        HtmlElementReferenceOrigin::ExplicitPointer
    );
    assert_eq!(
        associated(&r).explicit_reference(),
        &HtmlExplicitElementReference::Element(pointer)
    );
    assert_eq!(r.usage().scratch_records(), 0);
}
#[test]
fn explicit_target_needs_no_content_id_or_children_when_scope_is_proved() {
    for attrs in [
        vec![],
        vec![("commandfor", "")],
        vec![("commandfor", "wrong")],
    ] {
        let (mut b, d) = builder();
        let button = html(&mut b, &d, "button", &attrs);
        let target = element(&mut b, &d, Namespace::mathml(), "mi", &[]);
        children(&mut b, &d, &[button.clone(), target.clone()]);
        let c = b.finish().unwrap();
        let cx = context(
            &c,
            Some(facts(
                &c,
                vec![reference(
                    &c,
                    &button,
                    HtmlElementReferenceKey::CommandFor,
                    HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap()),
                )],
            )),
            vec![],
        );
        assert_eq!(
            associated(&query(
                &c,
                &cx,
                &c.subject_id(&button).unwrap(),
                HtmlSemanticQuery::CommandForElement,
                10
            ))
            .target(),
            Some(&c.subject_id(&target).unwrap())
        );
    }
}
fn shadow(
    b: &mut HtmlCaptureBuilder,
    document: &HtmlCaptureBuilderSubject,
    host: &HtmlCaptureBuilderSubject,
) -> HtmlCaptureBuilderSubject {
    b.add_shadow_root(
        document,
        host,
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
}
#[test]
fn shadow_source_can_explicitly_reference_outer_tree_but_content_cannot() {
    let (mut b, d) = builder();
    let host = html(&mut b, &d, "div", &[]);
    let sr = shadow(&mut b, &d, &host);
    let button = html(&mut b, &d, "button", &[("commandfor", "x")]);
    let target = html(&mut b, &d, "dialog", &[("id", "x")]);
    children(&mut b, &button, &[]);
    children(&mut b, &sr, std::slice::from_ref(&button));
    children(&mut b, &d, &[host, target.clone()]);
    let c = b.finish().unwrap();
    for explicit in [false, true] {
        let state = if explicit {
            HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap())
        } else {
            HtmlExplicitElementReference::Null
        };
        let cx = context(
            &c,
            Some(facts(
                &c,
                vec![reference(
                    &c,
                    &button,
                    HtmlElementReferenceKey::CommandFor,
                    state,
                )],
            )),
            vec![],
        );
        assert_eq!(
            associated(&query(
                &c,
                &cx,
                &c.subject_id(&button).unwrap(),
                HtmlSemanticQuery::CommandForElement,
                1
            ))
            .target(),
            explicit.then(|| c.subject_id(&target).unwrap()).as_ref()
        );
    }
}
#[test]
fn unrelated_shadow_target_and_template_host_are_not_ordinary_scope_edges() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("commandfor", "x")]);
    let host = html(&mut b, &d, "div", &[]);
    let sr = shadow(&mut b, &d, &host);
    let target = html(&mut b, &d, "div", &[("id", "x")]);
    children(&mut b, &sr, std::slice::from_ref(&target));
    children(&mut b, &button, &[]);
    children(&mut b, &host, &[]);
    children(&mut b, &d, &[button.clone(), host]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap()),
            )],
        )),
        vec![],
    );
    assert_eq!(
        associated(&query(
            &c,
            &cx,
            &c.subject_id(&button).unwrap(),
            HtmlSemanticQuery::CommandForElement,
            1
        ))
        .target(),
        None
    );
    let (mut b, d) = builder();
    let template = html(&mut b, &d, "template", &[]);
    let frag = b.add_fragment(&d).unwrap();
    b.establish_template_contents(&template, &frag).unwrap();
    let button = html(&mut b, &d, "button", &[]);
    let target = html(&mut b, &d, "dialog", &[]);
    children(&mut b, &frag, std::slice::from_ref(&button));
    children(&mut b, &d, &[template, target.clone()]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap()),
            )],
        )),
        vec![],
    );
    assert_eq!(
        associated(&query(
            &c,
            &cx,
            &c.subject_id(&button).unwrap(),
            HtmlSemanticQuery::CommandForElement,
            0
        ))
        .target(),
        None
    );
}
#[test]
fn explicit_scope_unknowns_remain_missing_but_positive_intersection_needs_no_later_ancestors() {
    let (mut b, d) = builder();
    let parent = html(&mut b, &d, "div", &[]);
    let button = html(&mut b, &d, "button", &[]);
    let target = html(&mut b, &d, "div", &[]);
    children(&mut b, &parent, &[button.clone(), target.clone()]);
    let unknown = html(&mut b, &d, "div", &[]);
    let c = b.finish().unwrap();
    for (selected, expected) in [
        (&target, None),
        (
            &unknown,
            Some(HtmlSemanticMissing::Parent(c.subject_id(&unknown).unwrap())),
        ),
    ] {
        let cx = context(
            &c,
            Some(facts(
                &c,
                vec![reference(
                    &c,
                    &button,
                    HtmlElementReferenceKey::CommandFor,
                    HtmlExplicitElementReference::Element(c.subject_id(selected).unwrap()),
                )],
            )),
            vec![],
        );
        let r = query(
            &c,
            &cx,
            &c.subject_id(&button).unwrap(),
            HtmlSemanticQuery::CommandForElement,
            0,
        );
        if let Some(expected) = expected {
            assert_eq!(missing(&r), &expected)
        } else {
            assert_eq!(
                associated(&r).target(),
                Some(&c.subject_id(&target).unwrap())
            )
        }
    }
}
#[test]
fn source_root_with_no_strict_ancestors_proves_explicit_null_even_target_parent_unknown() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[]);
    b.establish_root(&button).unwrap();
    let target = html(&mut b, &d, "div", &[]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap()),
            )],
        )),
        vec![],
    );
    assert_eq!(
        associated(&query(
            &c,
            &cx,
            &c.subject_id(&button).unwrap(),
            HtmlSemanticQuery::CommandForElement,
            0
        ))
        .target(),
        None
    );
}
#[test]
fn command_support_is_processing_only_and_ignores_dialog_open_and_popover_state() {
    for (command, ns, local, expected) in [
        ("unknown", Namespace::html(), "dialog", false),
        ("-- ", Namespace::svg(), "rect", true),
        ("show-modal", Namespace::svg(), "dialog", false),
        ("close", Namespace::html(), "dialog", true),
        ("request-close", Namespace::html(), "dialog", true),
        ("show-modal", Namespace::html(), "dialog", true),
        ("close", Namespace::html(), "div", false),
        ("show-popover", Namespace::html(), "x-custom", true),
        ("hide-popover", Namespace::html(), "div", true),
        ("toggle-popover", Namespace::html(), "dialog", true),
    ] {
        for open in [false, true] {
            let (mut b, d) = builder();
            let button = html(
                &mut b,
                &d,
                "button",
                &[("command", command), ("disabled", "bad")],
            );
            let attrs = if open {
                vec![("open", "notboolean"), ("popover", "bad")]
            } else {
                vec![]
            };
            let target = element(&mut b, &d, ns.clone(), local, &attrs);
            children(&mut b, &d, &[button.clone(), target.clone()]);
            let c = b.finish().unwrap();
            let cx = context(
                &c,
                Some(facts(
                    &c,
                    vec![reference(
                        &c,
                        &button,
                        HtmlElementReferenceKey::CommandFor,
                        HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap()),
                    )],
                )),
                vec![],
            );
            let r = query(
                &c,
                &cx,
                &c.subject_id(&button).unwrap(),
                HtmlSemanticQuery::CommandValidity,
                100,
            );
            assert_eq!(validity(&r).valid(), Some(expected));
            if command == "-- " {
                assert!(matches!(
                    validity(&r).command().author(),
                    HtmlAttributeAuthorState::Nonconforming(_)
                ));
                assert_eq!(
                    validity(&r)
                        .command()
                        .intrinsic()
                        .unwrap()
                        .custom_value()
                        .unwrap()
                        .code_units(),
                    &[45, 45, 32]
                );
            }
        }
    }
}
#[test]
fn command_known_null_is_distinct_from_false_support_and_retains_missing_raw_command() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    let r = query(
        &c,
        &cx,
        &c.subject_id(&button).unwrap(),
        HtmlSemanticQuery::CommandValidity,
        0,
    );
    assert_eq!(validity(&r).valid(), None);
    assert_eq!(
        validity(&r).command().intrinsic().unwrap().state(),
        HtmlCommandState::Unknown
    );
}
#[test]
fn popover_nonbutton_and_disabled_short_circuit_missing_target_and_form_facts() {
    for (local, attrs) in [
        ("div", vec![("popovertarget", "too long")]),
        (
            "input",
            vec![("type", "text"), ("popovertarget", "too long")],
        ),
        (
            "button",
            vec![("disabled", ""), ("popovertarget", "too long")],
        ),
    ] {
        let (mut b, d) = builder();
        let subject = html(&mut b, &d, local, &attrs);
        let c = b.finish().unwrap();
        let cx = context(&c, None, vec![]);
        assert_eq!(
            popover(&query(
                &c,
                &cx,
                &c.subject_id(&subject).unwrap(),
                HtmlSemanticQuery::PopoverTargetElement,
                4
            )),
            None
        );
    }
}
#[test]
fn popover_form_owner_and_submit_guards_use_actual_history_and_type() {
    for (local, ty, has_owner, expected_target) in [
        ("button", "submit", true, false),
        ("button", "button", true, true),
        ("button", "reset", true, true),
        ("button", "submit", false, true),
        ("input", "submit", true, false),
        ("input", "image", true, false),
        ("input", "reset", true, true),
        ("input", "button", true, true),
    ] {
        let (mut b, d) = builder();
        let form = html(&mut b, &d, "form", &[]);
        let button = html(
            &mut b,
            &d,
            local,
            &[("type", ty), ("popovertarget", "target")],
        );
        let target = html(&mut b, &d, "div", &[("id", "target"), ("popover", "")]);
        children(&mut b, &form, &[]);
        children(&mut b, &button, &[]);
        children(&mut b, &d, &[form.clone(), button.clone(), target.clone()]);
        let c = b.finish().unwrap();
        let id = c.subject_id(&button).unwrap();
        let cx = context(
            &c,
            Some(facts(
                &c,
                vec![reference(
                    &c,
                    &button,
                    HtmlElementReferenceKey::PopoverTarget,
                    HtmlExplicitElementReference::Null,
                )],
            )),
            vec![HtmlFormAssociation::new(
                id.clone(),
                HtmlFormAssociationState::EstablishedCurrent(
                    has_owner.then(|| c.subject_id(&form).unwrap()),
                ),
            )],
        );
        assert_eq!(
            popover(&query(
                &c,
                &cx,
                &id,
                HtmlSemanticQuery::PopoverTargetElement,
                100
            )),
            expected_target
                .then(|| c.subject_id(&target).unwrap())
                .as_ref()
        );
    }
}
#[test]
fn non_submit_button_proves_guard_false_despite_missing_form_history() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("type", "button")]);
    let target = html(&mut b, &d, "div", &[("popover", "")]);
    children(&mut b, &d, &[button.clone(), target.clone()]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::PopoverTarget,
                HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap()),
            )],
        )),
        vec![],
    );
    assert_eq!(
        popover(&query(
            &c,
            &cx,
            &c.subject_id(&button).unwrap(),
            HtmlSemanticQuery::PopoverTargetElement,
            10
        )),
        Some(&c.subject_id(&target).unwrap())
    );
}
#[test]
fn submit_button_missing_form_history_is_not_reset_or_replaced_by_later_target() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("type", "submit")]);
    children(&mut b, &d, std::slice::from_ref(&button));
    let c = b.finish().unwrap();
    let id = c.subject_id(&button).unwrap();
    let cx = context(&c, None, vec![]);
    assert_eq!(
        missing(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::PopoverTargetElement,
            10
        )),
        &HtmlSemanticMissing::FormAssociation(id)
    );
}
#[test]
fn known_null_owner_allows_auto_without_inventing_a_form_owner() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[]);
    children(&mut b, &d, std::slice::from_ref(&button));
    let c = b.finish().unwrap();
    let id = c.subject_id(&button).unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::PopoverTarget,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![HtmlFormAssociation::new(
            id.clone(),
            HtmlFormAssociationState::EstablishedCurrent(None),
        )],
    );
    assert_eq!(
        popover(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::PopoverTargetElement,
            0
        )),
        None
    );
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("commandfor", "")]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&button).unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::PopoverTarget,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    // Own disabled=false still needs ancestry; establish root in separate fixture
    // to isolate Auto's presence classification, covered by the next test.
    assert!(matches!(
        missing(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::PopoverTargetElement,
            0
        )),
        HtmlSemanticMissing::Parent(_)
    ));
}
#[test]
fn auto_presence_keeps_non_submit_even_with_unrecognized_command_and_unavailable_owner() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("command", "nonsense")]);
    b.establish_root(&button).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&button).unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::PopoverTarget,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    assert_eq!(
        popover(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::PopoverTargetElement,
            0
        )),
        None
    );
}
#[test]
fn popover_no_state_null_and_empty_or_invalid_state_target_are_distinct() {
    for attrs in [
        vec![],
        vec![("popover", "")],
        vec![("popover", "INVALID")],
        vec![("popover", "manual")],
    ] {
        let (mut b, d) = builder();
        let button = html(&mut b, &d, "button", &[("type", "button")]);
        let target = html(&mut b, &d, "div", &attrs);
        children(&mut b, &d, &[button.clone(), target.clone()]);
        let c = b.finish().unwrap();
        let cx = context(
            &c,
            Some(facts(
                &c,
                vec![reference(
                    &c,
                    &button,
                    HtmlElementReferenceKey::PopoverTarget,
                    HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap()),
                )],
            )),
            vec![],
        );
        assert_eq!(
            popover(&query(
                &c,
                &cx,
                &c.subject_id(&button).unwrap(),
                HtmlSemanticQuery::PopoverTargetElement,
                10
            )),
            (!attrs.is_empty())
                .then(|| c.subject_id(&target).unwrap())
                .as_ref()
        );
    }
}
#[test]
fn foreign_first_association_is_retained_but_guarded_popover_is_null_without_raw_admission() {
    let (mut b, d) = builder();
    let button = html(
        &mut b,
        &d,
        "button",
        &[("type", "button"), ("popovertarget", "x")],
    );
    let foreign = element(
        &mut b,
        &d,
        Namespace::svg(),
        "rect",
        &[("id", "x"), ("popover", "unconsulted long raw")],
    );
    let later = html(&mut b, &d, "div", &[("id", "x"), ("popover", "")]);
    children(&mut b, &button, &[]);
    children(&mut b, &d, &[button.clone(), foreign.clone(), later]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::PopoverTarget,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    let id = c.subject_id(&button).unwrap();
    assert_eq!(
        associated(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::PopoverTargetAssociatedElement,
            6
        ))
        .target(),
        Some(&c.subject_id(&foreign).unwrap())
    );
    assert_eq!(
        popover(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::PopoverTargetElement,
            6
        )),
        None
    );
}
#[test]
fn both_reference_keys_are_independent_and_wrong_source_interfaces_are_inapplicable() {
    let (mut b, d) = builder();
    let input = html(&mut b, &d, "input", &[("type", "button")]);
    let div = html(&mut b, &d, "div", &[]);
    let foreign = element(&mut b, &d, Namespace::svg(), "button", &[]);
    let c = b.finish().unwrap();
    let cx = context(&c, None, vec![]);
    for subject in [&input, &div, &foreign] {
        assert!(matches!(
            query(
                &c,
                &cx,
                &c.subject_id(subject).unwrap(),
                HtmlSemanticQuery::CommandForElement,
                0
            )
            .results()[0]
                .fact(),
            HtmlSemanticFact::CommandForElement(HtmlSemanticValue::Inapplicable)
        ));
    }
    assert_eq!(
        missing(&query(
            &c,
            &cx,
            &c.subject_id(&input).unwrap(),
            HtmlSemanticQuery::PopoverTargetAssociatedElement,
            0
        )),
        &HtmlSemanticMissing::ElementReference {
            subject: c.subject_id(&input).unwrap(),
            key: HtmlElementReferenceKey::PopoverTarget
        }
    );
}
#[test]
fn checked_facts_reject_bad_subject_target_duplicates_capacity_and_validation_work() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[]);
    let input = html(&mut b, &d, "input", &[]);
    let text = b.add_text(&d, "text".into()).unwrap();
    let c = b.finish().unwrap();
    let base = reference(
        &c,
        &button,
        HtmlElementReferenceKey::CommandFor,
        HtmlExplicitElementReference::Null,
    );
    let make =
        |refs, l| HtmlElementReferenceFacts::new(&c, c.host_facts_revision().clone(), refs, l);
    assert!(matches!(
        make(
            vec![base.clone()],
            HtmlElementReferenceFactsLimits::new(0, 1000000)
        ),
        Err(HtmlElementReferenceFactsError::RecordCapacity { .. })
    ));
    assert!(matches!(
        make(
            vec![base.clone()],
            HtmlElementReferenceFactsLimits::new(10, 0)
        ),
        Err(HtmlElementReferenceFactsError::ValidationWork { .. })
    ));
    assert!(matches!(
        make(
            vec![base.clone(), base.clone()],
            HtmlElementReferenceFactsLimits::new(10, 1000000)
        ),
        Err(HtmlElementReferenceFactsError::DuplicateReference { .. })
    ));
    assert!(matches!(
        make(
            vec![reference(
                &c,
                &input,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Null
            )],
            HtmlElementReferenceFactsLimits::new(10, 1000000)
        ),
        Err(HtmlElementReferenceFactsError::InapplicableSubject(_))
    ));
    assert!(matches!(
        make(
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Element(c.subject_id(&text).unwrap())
            )],
            HtmlElementReferenceFactsLimits::new(10, 1000000)
        ),
        Err(HtmlElementReferenceFactsError::InvalidTarget(_))
    ));
    assert!(matches!(
        HtmlElementReferenceFacts::new(
            &c,
            HtmlHostFactsRevision::fresh(),
            vec![base.clone()],
            HtmlElementReferenceFactsLimits::new(10, 1000000)
        ),
        Err(HtmlElementReferenceFactsError::HostRevisionMismatch)
    ));
    let (mut ob, od) = builder();
    let foreign = html(&mut ob, &od, "div", &[]);
    let other = ob.finish().unwrap();
    assert!(matches!(
        make(
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Element(other.subject_id(&foreign).unwrap())
            )],
            HtmlElementReferenceFactsLimits::new(10, 1000000)
        ),
        Err(HtmlElementReferenceFactsError::Capture(_))
    ));
    let mismatched = facts(&c, vec![base]);
    assert!(matches!(
        HtmlSemanticContext::new(
            &other,
            other.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_element_references(mismatched),
            0
        ),
        Err(HtmlSemanticContextError::ElementReferencesCaptureMismatch)
    ));
}
#[test]
fn work_scratch_and_allocation_caps_latch_without_partial_results_and_release_frames() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("commandfor", "x")]);
    let target = html(&mut b, &d, "dialog", &[("id", "x")]);
    children(&mut b, &button, &[]);
    children(&mut b, &d, &[button.clone(), target.clone()]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    let id = c.subject_id(&button).unwrap();
    let req = request(&id, HtmlSemanticQuery::CommandForElement);
    let preserved = query(&c, &cx, &id, HtmlSemanticQuery::CommandForElement, 1);
    let work = preserved.usage().work();
    assert_eq!(preserved.usage().scratch_records(), 0);
    assert_eq!(preserved.usage().nested_output_records(), 0);
    let mut exact = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(work, 100, 1000, 1000000, 1000000, 1000),
        1,
        HtmlSemanticCancellation::new(),
    );
    assert_eq!(
        associated(&exact.query(std::slice::from_ref(&req)).unwrap()).target(),
        Some(&c.subject_id(&target).unwrap())
    );
    for (l, resource) in [
        (
            HtmlSemanticLimits::new(work - 1, 100, 1000, 1000000, 1000000, 1000),
            HtmlSemanticResource::Work,
        ),
        (
            HtmlSemanticLimits::new(10000000, 100, 0, 1000000, 1000000, 1000),
            HtmlSemanticResource::ScratchRecords,
        ),
        (
            HtmlSemanticLimits::new(10000000, 100, 1000, 1000000, 0, 1000),
            HtmlSemanticResource::AllocatedBytes,
        ),
        (
            HtmlSemanticLimits::new(10000000, 0, 1000, 1000000, 1000000, 1000),
            HtmlSemanticResource::OutputRecords,
        ),
    ] {
        let mut s = session(&c, &cx, l, 1, HtmlSemanticCancellation::new());
        let e = s.query(std::slice::from_ref(&req)).unwrap_err();
        assert!(
            matches!(e.kind(),HtmlSemanticFailureKind::LimitExceeded {resource:r,..}if *r==resource)
        );
        assert_eq!(s.usage().scratch_records(), 0);
        assert_eq!(s.query(&[]).unwrap_err(), e);
        assert_eq!(e.qualification().unwrap().context(), cx.identity());
    }
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &cx, limits(), 1, cancel.clone());
    cancel.cancel();
    let e = s.query(std::slice::from_ref(&req)).unwrap_err();
    assert_eq!(e.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(s.query(&[]).unwrap_err(), e);
    assert_eq!(
        associated(&preserved).target(),
        Some(&c.subject_id(&target).unwrap())
    );
}
#[test]
fn raw_admission_failure_is_sticky_and_later_request_discards_whole_batch() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("commandfor", "long")]);
    let c = b.finish().unwrap();
    let cx = context(&c, None, vec![]);
    let id = c.subject_id(&button).unwrap();
    let mut s = session(&c, &cx, limits(), 3, HtmlSemanticCancellation::new());
    let e = s
        .query(&[
            request(&id, HtmlSemanticQuery::PopoverTargetElement),
            request(&id, HtmlSemanticQuery::CommandForElement),
        ])
        .unwrap_err();
    // The first request yields a complete Missing fact for disabled ancestry.
    // The next oversized raw value discards that entire batch on terminal stop.
    assert!(matches!(e.kind(), HtmlSemanticFailureKind::Attribute(_)));
    assert_eq!(s.query(&[]).unwrap_err(), e);
}

#[test]
fn disabled_fieldset_witness_precedes_form_and_target_but_first_legend_is_exempt() {
    for inside_legend in [false, true] {
        let (mut b, d) = builder();
        let fieldset = html(&mut b, &d, "fieldset", &[("disabled", "")]);
        let legend = html(&mut b, &d, "legend", &[]);
        let button = html(
            &mut b,
            &d,
            "button",
            &[("type", "submit"), ("popovertarget", "unconsulted")],
        );
        if inside_legend {
            children(&mut b, &legend, std::slice::from_ref(&button));
            children(&mut b, &fieldset, std::slice::from_ref(&legend));
        } else {
            children(&mut b, &legend, &[]);
            children(&mut b, &fieldset, &[legend, button.clone()]);
        }
        children(&mut b, &d, std::slice::from_ref(&fieldset));
        let c = b.finish().unwrap();
        let id = c.subject_id(&button).unwrap();
        let cx = context(&c, None, vec![]);
        let r = query(&c, &cx, &id, HtmlSemanticQuery::PopoverTargetElement, 6);
        if inside_legend {
            assert_eq!(missing(&r), &HtmlSemanticMissing::FormAssociation(id));
        } else {
            assert_eq!(popover(&r), None);
        }
    }
}
#[test]
fn explicit_foreign_popover_pointer_is_generic_target_but_guarded_null() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("type", "button")]);
    let target = element(
        &mut b,
        &d,
        Namespace::svg(),
        "g",
        &[("popover", "unconsulted raw")],
    );
    children(&mut b, &d, &[button.clone(), target.clone()]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::PopoverTarget,
                HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap()),
            )],
        )),
        vec![],
    );
    let id = c.subject_id(&button).unwrap();
    assert_eq!(
        associated(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::PopoverTargetAssociatedElement,
            6
        ))
        .target(),
        Some(&c.subject_id(&target).unwrap())
    );
    assert_eq!(
        popover(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::PopoverTargetElement,
            6
        )),
        None
    );
}
#[test]
fn null_pointer_resolution_reports_missing_parent_before_id_traversal() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[("commandfor", "x")]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&button).unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    assert_eq!(
        missing(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::CommandForElement,
            1
        )),
        &HtmlSemanticMissing::Parent(id)
    );
}
#[test]
fn reference_keys_can_coexist_and_raw_selected_attribute_uses_exact_expanded_name() {
    let (mut b, d) = builder();
    let target = html(&mut b, &d, "div", &[]);
    let button = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "button"),
            OrderedAttributes::new([
                Attribute::new(
                    QualifiedName::new(Namespace::svg(), None, "commandfor"),
                    "unconsulted",
                ),
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "COMMANDfor"),
                    "unconsulted",
                ),
            ])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    children(&mut b, &d, &[button.clone(), target.clone()]);
    let c = b.finish().unwrap();
    let records = vec![
        reference(
            &c,
            &button,
            HtmlElementReferenceKey::CommandFor,
            HtmlExplicitElementReference::Null,
        ),
        reference(
            &c,
            &button,
            HtmlElementReferenceKey::PopoverTarget,
            HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap()),
        ),
    ];
    let cx = context(&c, Some(facts(&c, records)), vec![]);
    let id = c.subject_id(&button).unwrap();
    let command = query(&c, &cx, &id, HtmlSemanticQuery::CommandForElement, 0);
    assert!(associated(&command).raw().is_none());
    assert!(associated(&command).target().is_none());
    assert_eq!(
        associated(&query(
            &c,
            &cx,
            &id,
            HtmlSemanticQuery::PopoverTargetAssociatedElement,
            0
        ))
        .target(),
        Some(&c.subject_id(&target).unwrap())
    );
}
#[test]
fn queried_raw_command_and_reference_caps_are_independent_and_complete() {
    for (attrs, query_kind, raw_limit) in [
        (
            vec![("commandfor", "long")],
            HtmlSemanticQuery::CommandForElement,
            3,
        ),
        (
            vec![("command", "--long")],
            HtmlSemanticQuery::CommandValidity,
            5,
        ),
    ] {
        let (mut b, d) = builder();
        let button = html(&mut b, &d, "button", &attrs);
        let target = html(&mut b, &d, "div", &[]);
        children(&mut b, &d, &[button.clone(), target.clone()]);
        let c = b.finish().unwrap();
        let cx = context(
            &c,
            Some(facts(
                &c,
                vec![reference(
                    &c,
                    &button,
                    HtmlElementReferenceKey::CommandFor,
                    HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap()),
                )],
            )),
            vec![],
        );
        let mut s = session(
            &c,
            &cx,
            limits(),
            raw_limit,
            HtmlSemanticCancellation::new(),
        );
        let e = s
            .query(&[request(&c.subject_id(&button).unwrap(), query_kind)])
            .unwrap_err();
        assert!(matches!(e.kind(), HtmlSemanticFailureKind::Attribute(_)));
        assert_eq!(s.query(&[]).unwrap_err(), e);
    }
}
#[test]
fn retained_results_keep_exact_context_host_and_capture_keys_across_replay_and_cancel() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[]);
    let c = b.finish().unwrap();
    let id = c.subject_id(&button).unwrap();
    let make = || {
        context(
            &c,
            Some(facts(
                &c,
                vec![reference(
                    &c,
                    &button,
                    HtmlElementReferenceKey::CommandFor,
                    HtmlExplicitElementReference::Null,
                )],
            )),
            vec![],
        )
    };
    let first = make();
    let second = make();
    assert_ne!(first.identity(), second.identity());
    let r = query(&c, &first, &id, HtmlSemanticQuery::CommandForElement, 0);
    assert_eq!(r.capture(), c.identity());
    assert_eq!(r.context(), first.identity());
    assert_eq!(r.host_revision(), c.host_facts_revision());
    let r2 = query(&c, &second, &id, HtmlSemanticQuery::CommandForElement, 0);
    assert_eq!(associated(&r), associated(&r2));
    assert_ne!(r.context(), r2.context());
    let cancel = HtmlSemanticCancellation::new();
    let mut s = session(&c, &first, limits(), 0, cancel.clone());
    let retained = s
        .query(&[request(&id, HtmlSemanticQuery::CommandForElement)])
        .unwrap();
    let work = s.usage().work();
    cancel.cancel();
    assert_eq!(
        s.query(&[request(&id, HtmlSemanticQuery::CommandForElement)])
            .unwrap_err()
            .kind(),
        &HtmlSemanticFailureKind::Cancelled
    );
    assert_eq!(s.usage().work(), work);
    assert_eq!(associated(&retained).target(), None);
    assert!(matches!(
        HtmlSemanticContext::new(
            &c,
            HtmlHostFactsRevision::fresh(),
            HtmlSemanticContextInput::new(vec![]),
            0
        ),
        Err(HtmlSemanticContextError::HostRevisionMismatch)
    ));
}
#[test]
fn later_sticky_cap_drops_complete_batch_and_keeps_scratch_released() {
    let (mut b, d) = builder();
    let early = html(&mut b, &d, "button", &[]);
    let late = html(&mut b, &d, "button", &[("commandfor", "x")]);
    let target = html(&mut b, &d, "div", &[("id", "x")]);
    children(&mut b, &early, &[]);
    children(&mut b, &late, &[]);
    children(&mut b, &d, &[early.clone(), late.clone(), target]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![
                reference(
                    &c,
                    &early,
                    HtmlElementReferenceKey::CommandFor,
                    HtmlExplicitElementReference::Null,
                ),
                reference(
                    &c,
                    &late,
                    HtmlElementReferenceKey::CommandFor,
                    HtmlExplicitElementReference::Null,
                ),
            ],
        )),
        vec![],
    );
    let requests = [
        request(
            &c.subject_id(&early).unwrap(),
            HtmlSemanticQuery::CommandForElement,
        ),
        request(
            &c.subject_id(&late).unwrap(),
            HtmlSemanticQuery::CommandForElement,
        ),
    ];
    let good = session(&c, &cx, limits(), 1, HtmlSemanticCancellation::new())
        .query(&requests)
        .unwrap();
    let mut limited = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(good.usage().work() - 1, 100, 1000, 1000000, 1000000, 1000),
        1,
        HtmlSemanticCancellation::new(),
    );
    let failure = limited.query(&requests).unwrap_err();
    assert_eq!(limited.usage().scratch_records(), 0);
    assert!(limited.usage().work() > 0);
    assert_eq!(limited.query(&requests).unwrap_err(), failure);
}

#[test]
fn scratch_byte_failure_after_output_and_one_frame_keeps_actual_cumulative_usage() {
    let (mut b, d) = builder();
    let wrapper = html(&mut b, &d, "div", &[]);
    let button = html(&mut b, &d, "button", &[("commandfor", "x")]);
    let target = html(&mut b, &d, "div", &[("id", "x")]);
    children(&mut b, &wrapper, &[button.clone(), target]);
    children(&mut b, &button, &[]);
    children(&mut b, &d, std::slice::from_ref(&wrapper));
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    let bytes =
        std::mem::size_of::<HtmlSemanticResult>() + std::mem::size_of::<HtmlCapturedChildIds<'_>>();
    let mut s = session(
        &c,
        &cx,
        HtmlSemanticLimits::new(10000000, 100, 1000, 1000000, bytes, 1000),
        1,
        HtmlSemanticCancellation::new(),
    );
    let failure = s
        .query(&[request(
            &c.subject_id(&button).unwrap(),
            HtmlSemanticQuery::CommandForElement,
        )])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::AllocatedBytes,
            ..
        }
    ));
    assert_eq!(s.usage().scratch_records(), 0);
    assert_eq!(s.usage().peak_scratch_records(), 1);
    assert_eq!(s.usage().allocated_bytes(), bytes);
    assert_eq!(s.query(&[]).unwrap_err(), failure);
}
#[test]
fn current_owner_and_reset_against_capture_both_supply_real_submit_guard() {
    let (mut b, d) = builder();
    let form = html(&mut b, &d, "form", &[]);
    let button = html(&mut b, &d, "button", &[("type", "submit")]);
    children(&mut b, &form, std::slice::from_ref(&button));
    children(&mut b, &d, std::slice::from_ref(&form));
    let c = b.finish().unwrap();
    let id = c.subject_id(&button).unwrap();
    for state in [
        HtmlFormAssociationState::EstablishedCurrent(Some(c.subject_id(&form).unwrap())),
        HtmlFormAssociationState::ResetAgainstCapture,
    ] {
        let cx = context(&c, None, vec![HtmlFormAssociation::new(id.clone(), state)]);
        assert_eq!(
            popover(&query(
                &c,
                &cx,
                &id,
                HtmlSemanticQuery::PopoverTargetElement,
                6
            )),
            None
        );
    }
}
#[test]
fn detached_target_no_ancestors_proves_out_of_scope_even_source_parent_unknown() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[]);
    let target = html(&mut b, &d, "div", &[]);
    b.establish_root(&target).unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Element(c.subject_id(&target).unwrap()),
            )],
        )),
        vec![],
    );
    assert_eq!(
        associated(&query(
            &c,
            &cx,
            &c.subject_id(&button).unwrap(),
            HtmlSemanticQuery::CommandForElement,
            0
        ))
        .target(),
        None
    );
}
#[test]
fn source_itself_is_eligible_through_a_strict_common_ancestor_but_not_as_detached_root() {
    for detached in [false, true] {
        let (mut b, d) = builder();
        let button = html(&mut b, &d, "button", &[]);
        if detached {
            b.establish_root(&button).unwrap();
        } else {
            children(&mut b, &d, std::slice::from_ref(&button));
        }
        let c = b.finish().unwrap();
        let id = c.subject_id(&button).unwrap();
        let cx = context(
            &c,
            Some(facts(
                &c,
                vec![reference(
                    &c,
                    &button,
                    HtmlElementReferenceKey::CommandFor,
                    HtmlExplicitElementReference::Element(id.clone()),
                )],
            )),
            vec![],
        );
        assert_eq!(
            associated(&query(
                &c,
                &cx,
                &id,
                HtmlSemanticQuery::CommandForElement,
                0
            ))
            .target(),
            (!detached).then_some(&id)
        );
    }
}
#[test]
fn complete_context_cannot_be_reused_with_another_capture_or_foreign_request() {
    let (mut b, d) = builder();
    let button = html(&mut b, &d, "button", &[]);
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(facts(
            &c,
            vec![reference(
                &c,
                &button,
                HtmlElementReferenceKey::CommandFor,
                HtmlExplicitElementReference::Null,
            )],
        )),
        vec![],
    );
    let (ob, od) = builder();
    let other = ob.finish().unwrap();
    assert!(matches!(
        HtmlSemanticSession::new(
            &other,
            &cx,
            HtmlValueInterpreter::new(0),
            limits(),
            HtmlSemanticCancellation::new()
        )
        .err()
        .unwrap()
        .kind(),
        HtmlSemanticFailureKind::ContextCaptureMismatch
    ));
    let mut s = session(&c, &cx, limits(), 0, HtmlSemanticCancellation::new());
    let failure = s
        .query(&[request(
            &other.subject_id(&od).unwrap(),
            HtmlSemanticQuery::PopoverTargetElement,
        )])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(_)
    ));
    assert_eq!(s.query(&[]).unwrap_err(), failure);
}
