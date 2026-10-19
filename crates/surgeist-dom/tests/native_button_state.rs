//! HTML877d native Button association, explicit weak reflection and query oracles.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{HtmlFormMethod, HtmlNativeFormAssociationKind, HtmlValueInterpreter};
use surgeist_html_semantics::*;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            128,
            16384,
            256,
            256,
            128,
            16,
            65536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, b: DomBatch) -> DomCommit {
    dom.apply(b, &cancel()).unwrap()
}
fn setup(dom: &mut Dom) -> (DocumentId, NodeId) {
    let mut b = DomBatch::new();
    let d = b.create_document();
    let button = b.create_element_ns(&d, Namespace::html(), "button");
    let c = apply(dom, b);
    (
        c.document(&d).unwrap(),
        c.created(&button).unwrap().unwrap(),
    )
}
fn button(s: &DomSnapshot, n: NodeId) -> DomButton<'_> {
    s.node(n).unwrap().as_button().unwrap()
}
fn capture(s: &DomSnapshot, d: DocumentId) -> DomHtmlCapture {
    s.html_capture(
        d,
        None,
        HtmlCaptureLimits::new(256, 4096, 256, 32768, 256, 0, 0, 16_777_216),
        DomHtmlCaptureLimits::new(256, 1000000),
        &cancel(),
    )
    .unwrap()
}
fn supplement(c: DomHtmlCapture) -> DomHtmlCapture {
    c.with_slot_semantics(
        DomSlotSemanticLimits::new(
            DomSlotQueryLimits::new(256, 512).unwrap(),
            DomHtmlCaptureLimits::new(256, 1000000),
            HtmlLanguageDirectionLimits::new(512, 512, 32768, 1000000),
            HtmlInertFactLimits::new(512, 1000000),
        ),
        &cancel(),
    )
    .unwrap()
}
fn fact(c: &DomHtmlCapture, n: NodeId, q: HtmlSemanticQuery) -> HtmlSemanticFact {
    c.query(
        &[HtmlSemanticRequest::new(c.subject(n).unwrap(), q)],
        HtmlValueInterpreter::new(32768),
        HtmlSemanticLimits::new(1000000, 16, 1024, 512, 1000000, 1024),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .results()[0]
        .fact()
        .clone()
}
fn associated(c: &DomHtmlCapture, n: NodeId, popover: bool) -> HtmlAssociatedElement {
    match fact(
        c,
        n,
        if popover {
            HtmlSemanticQuery::PopoverTargetAssociatedElement
        } else {
            HtmlSemanticQuery::CommandForElement
        },
    ) {
        HtmlSemanticFact::CommandForElement(HtmlSemanticValue::Known(a))
        | HtmlSemanticFact::PopoverTargetAssociatedElement(HtmlSemanticValue::Known(a)) => a,
        other => panic!("actual nullable reflected state required: {other:?}"),
    }
}
fn target(c: &DomHtmlCapture, n: NodeId, popover: bool) -> Option<NodeId> {
    associated(c, n, popover)
        .target()
        .map(|id| c.node(id).unwrap())
}
#[test]
fn initial_button_has_independent_null_slots_and_actual_native_control_state() {
    let mut dom = owner();
    let (d, n) = setup(&mut dom);
    let s = dom.snapshot();
    let b = button(&s, n);
    assert_eq!(b.id(), n);
    assert_eq!(b.state().kind(), HtmlNativeFormAssociationKind::Button);
    assert_eq!(b.form_owner().unwrap(), None);
    assert!(!b.state().parser_inserted());
    assert_eq!(
        b.state().custom_validity_message(),
        &DocumentString::from("")
    );
    assert_eq!(b.explicit_command_for(), None);
    assert_eq!(b.explicit_popover_target(), None);
    assert!(
        s.node(n)
            .unwrap()
            .as_element()
            .unwrap()
            .fieldset_state()
            .is_none()
    );
    assert_eq!(dom.usage().string_code_units(), 65); // metadata31 + HTML URI28 + button6.
    let c = capture(&s, d);
    for popover in [false, true] {
        let a = associated(&c, n, popover);
        assert_eq!(a.explicit_reference(), &HtmlExplicitElementReference::Null);
        assert_eq!(a.target(), None);
        assert_eq!(a.origin(), HtmlElementReferenceOrigin::ContentAttribute);
    }
    let HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(form)) =
        fact(&c, n, HtmlSemanticQuery::FormOwner)
    else {
        panic!("actual owner")
    };
    assert_eq!(form.owner(), None);
    assert_eq!(form.origin(), HtmlFormOwnerOrigin::EstablishedCurrent);
}
#[test]
fn same_batch_targets_resolve_to_real_ids_and_reflection_slots_stay_independent() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let top = b.create_element_ns(&d, Namespace::html(), "div");
    let n = b.create_element_ns(&d, Namespace::html(), "button");
    let one = b.create_element_ns(&d, Namespace::html(), "span");
    let two = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&top, &d, None);
    for child in [&n, &one, &two] {
        b.insert_before(child, &top, None);
    }
    b.set_command_for_element(&n, Some((&one).into()));
    b.set_popover_target_element(&n, Some((&two).into()));
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let id = |t: &CreatedNode| c.created(t).unwrap().unwrap();
    let (n, one, two) = (id(&n), id(&one), id(&two));
    let s = dom.snapshot();
    assert_eq!(button(&s, n).explicit_command_for(), Some(one));
    assert_eq!(button(&s, n).explicit_popover_target(), Some(two));
    for attr in ["commandfor", "popovertarget"] {
        assert_eq!(
            s.node(n)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&attr.into()),
            Some(&DocumentString::from(""))
        );
    }
    let c = supplement(capture(&s, d));
    assert_eq!(target(&c, n, false), Some(one));
    assert_eq!(target(&c, n, true), Some(two));
    assert_eq!(
        associated(&c, n, false).origin(),
        HtmlElementReferenceOrigin::ExplicitPointer
    );
    let mut b = DomBatch::new();
    b.set_attribute(n, "commandfor", "");
    let c = apply(&mut dom, b);
    assert!(c.changes().changes().contains(&DomChange::ElementState(n)));
    assert!(!c.changes().changes().contains(&DomChange::Attributes(n)));
    let current = dom.snapshot();
    assert_eq!(button(&current, n).explicit_command_for(), None);
    assert_eq!(button(&current, n).explicit_popover_target(), Some(two));
    assert_eq!(button(&s, n).explicit_command_for(), Some(one));
    let mut b = DomBatch::new();
    b.set_popover_target_element(n, None);
    apply(&mut dom, b);
    let current = dom.snapshot();
    assert_eq!(button(&current, n).explicit_popover_target(), None);
    assert_eq!(
        current
            .node(n)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"popovertarget".into()),
        None
    );
}
#[test]
fn exact_namespace_hooks_and_redundant_clears_do_not_confuse_the_two_targets() {
    let mut dom = owner();
    let (_, n) = setup(&mut dom);
    let mut b = DomBatch::new();
    b.set_command_for_element(n, Some(n.into()));
    b.set_popover_target_element(n, Some(n.into()));
    apply(&mut dom, b);
    let mut b = DomBatch::new();
    b.set_attribute_ns(n, Namespace::from_uri("urn:foreign"), "p:commandfor", "x");
    b.set_attribute(n, "COMMANDFOR", "x");
    b.remove_attribute(n, "absent");
    apply(&mut dom, b);
    assert_eq!(button(&dom.snapshot(), n).explicit_command_for(), Some(n));
    assert_eq!(
        button(&dom.snapshot(), n).explicit_popover_target(),
        Some(n)
    );
    let mut b = DomBatch::new();
    b.set_attribute_ns(n, Namespace::none(), "commandfor", "");
    b.remove_attribute(n, "popovertarget");
    apply(&mut dom, b);
    assert_eq!(button(&dom.snapshot(), n).explicit_command_for(), None);
    assert_eq!(button(&dom.snapshot(), n).explicit_popover_target(), None);
}
#[test]
fn explicit_storage_accepts_other_document_detached_self_and_shadow_targets_without_getter_scope_veto()
 {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let a = b.create_document();
    let d = b.create_document();
    let n = b.create_element_ns(&a, Namespace::html(), "button");
    let outer = b.create_element_ns(&d, Namespace::html(), "div");
    let child = b.create_element_ns(&d, Namespace::html(), "span");
    b.insert_before(&outer, &d, None);
    b.insert_before(&child, &outer, None);
    b.set_command_for_element(&n, Some((&child).into()));
    b.set_popover_target_element(&n, Some((&n).into()));
    let c = apply(&mut dom, b);
    let (a, d) = (c.document(&a).unwrap(), c.document(&d).unwrap());
    let id = |t: &CreatedNode| c.created(t).unwrap().unwrap();
    let (n, outer, child) = (id(&n), id(&outer), id(&child));
    let old = dom.snapshot();
    let c = capture(&old, a);
    assert_eq!(c.node(&c.subject(child).unwrap()).unwrap(), child);
    assert_eq!(target(&c, n, false), None);
    assert_eq!(target(&c, n, true), None);
    assert_eq!(button(&old, n).explicit_command_for(), Some(child));
    assert_eq!(button(&old, n).explicit_popover_target(), Some(n));
    let mut b = DomBatch::new();
    b.adopt(n, d);
    b.insert_before(n, outer, None);
    apply(&mut dom, b);
    let s = dom.snapshot();
    let c = supplement(capture(&s, d));
    assert_eq!(target(&c, n, false), Some(child));
    assert_eq!(target(&c, n, true), Some(n));
    assert_eq!(button(&s, n).explicit_command_for(), Some(child));
    assert_eq!(target(&capture(&old, a), n, false), None);
    let mut b = DomBatch::new();
    b.detach(child);
    apply(&mut dom, b);
    assert_eq!(
        button(&dom.snapshot(), n).explicit_command_for(),
        Some(child)
    );
    assert_eq!(target(&capture(&dom.snapshot(), d), n, false), None);
    let mut b = DomBatch::new();
    let host = b.create_element_ns(d, Namespace::html(), "div");
    b.insert_before(&host, outer, None);
    let r = b.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    b.insert_before(n, &r, None);
    b.insert_before(child, outer, None);
    apply(&mut dom, b);
    assert_eq!(target(&capture(&dom.snapshot(), d), n, false), Some(child));
}
#[test]
fn null_explicit_state_uses_first_exact_tree_id_and_old_captures_keep_their_candidate() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let top = b.create_element_ns(&d, Namespace::html(), "div");
    let n = b.create_element_ns(&d, Namespace::html(), "button");
    let first = b.create_element(&d, "first");
    let second = b.create_element_ns(&d, Namespace::html(), "span");
    for t in [&first, &second] {
        b.set_attribute(t, "id", "chosen");
        b.insert_before(t, &top, None);
    }
    b.insert_before(&n, &top, None);
    b.insert_before(&top, &d, None);
    b.set_attribute(&n, "commandfor", "chosen");
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let id = |t: &CreatedNode| c.created(t).unwrap().unwrap();
    let (n, top, first, second) = (id(&n), id(&top), id(&first), id(&second));
    let old = capture(&dom.snapshot(), d);
    assert_eq!(target(&old, n, false), Some(first));
    let mut b = DomBatch::new();
    b.insert_before(second, top, Some(first.into()));
    apply(&mut dom, b);
    let c = capture(&dom.snapshot(), d);
    assert_eq!(target(&c, n, false), Some(second));
    assert_eq!(target(&old, n, false), Some(first));
    assert_eq!(
        associated(&c, n, false).origin(),
        HtmlElementReferenceOrigin::ContentAttribute
    );
    let mut b = DomBatch::new();
    b.set_attribute(n, "commandfor", "");
    apply(&mut dom, b);
    assert_eq!(target(&capture(&dom.snapshot(), d), n, false), None);
}
#[test]
fn weak_retirement_clears_both_slots_without_following_generation_reuse() {
    let mut dom = owner();
    let (d, n) = setup(&mut dom);
    let mut b = DomBatch::new();
    let t = b.create_element(d, "target");
    b.set_command_for_element(n, Some((&t).into()));
    b.set_popover_target_element(n, Some((&t).into()));
    let c = apply(&mut dom, b);
    let t = c.created(&t).unwrap().unwrap();
    let old = dom.snapshot();
    let before = old.stamp();
    let mut b = DomBatch::new();
    b.destroy(t);
    let c = apply(&mut dom, b);
    assert!(c.changes().changes().contains(&DomChange::ElementState(n)));
    let mut b = DomBatch::new();
    let replacement = b.create_element(d, "replacement");
    let c = apply(&mut dom, b);
    let replacement = c.created(&replacement).unwrap().unwrap();
    assert_ne!(replacement, t);
    assert_eq!(button(&dom.snapshot(), n).explicit_command_for(), None);
    assert_eq!(button(&dom.snapshot(), n).explicit_popover_target(), None);
    assert_eq!(button(&old, n).explicit_command_for(), Some(t));
    assert!(old.node(t).is_ok());
    assert!(matches!(dom.snapshot().node(t), Err(DomError::StaleNode)));
    assert!(
        dom.changes_since(before)
            .unwrap()
            .changes()
            .contains(&DomChange::ElementState(n))
    );
    let mut b = DomBatch::new();
    let temporary = b.create_element(d, "temporary");
    b.set_command_for_element(n, Some((&temporary).into()));
    b.destroy(&temporary);
    apply(&mut dom, b);
    assert_eq!(button(&dom.snapshot(), n).explicit_command_for(), None);
}
#[test]
fn message_and_adoption_preserve_state_while_native_import_starts_fresh() {
    let mut dom = owner();
    let (a, n) = setup(&mut dom);
    let mut b = DomBatch::new();
    let d = b.create_document();
    b.set_custom_validity(
        n,
        DocumentString::from_utf16(vec![13, 10, 0xd800, 13, 0xdc00]),
    );
    b.set_command_for_element(n, Some(n.into()));
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    let copy = b.import_node(&old, n, d, true);
    b.adopt(n, d);
    let c = apply(&mut dom, b);
    let copy = c.created(&copy).unwrap().unwrap();
    let s = dom.snapshot();
    assert_eq!(
        button(&s, n).state().custom_validity_message().code_units(),
        &[10, 0xd800, 10, 0xdc00]
    );
    assert_eq!(button(&s, n).explicit_command_for(), Some(n));
    assert_eq!(
        button(&s, copy).state().custom_validity_message(),
        &DocumentString::from("")
    );
    assert_eq!(button(&s, copy).explicit_command_for(), None);
    assert_eq!(
        s.node(copy)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"commandfor".into()),
        Some(&DocumentString::from(""))
    );
    assert_eq!(old.node(n).unwrap().owner_document(), Some(a));
    assert_eq!(s.node(n).unwrap().owner_document(), Some(d));
}
#[test]
fn target_limits_late_failure_foreign_identity_and_cancel_preserve_state_and_history() {
    let mut dom = owner();
    let (d, n) = setup(&mut dom);
    let before = dom.snapshot();
    let usage = dom.usage();
    dom.set_limits(
        DomLimits::new(
            128,
            74,
            256,
            256,
            128,
            16,
            65536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut b = DomBatch::new();
    b.set_command_for_element(n, Some(n.into()));
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: 74
        }
    );
    dom.set_limits(
        DomLimits::new(
            128,
            75,
            256,
            256,
            128,
            16,
            65536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut b = DomBatch::new();
    b.set_command_for_element(n, Some(n.into()));
    b.insert_before(d, n, None);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::HierarchyRequest
    );
    let mut b = DomBatch::new();
    b.set_command_for_element(n, Some(n.into()));
    assert_eq!(
        dom.apply(b, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    let mut other = owner();
    let (_, foreign) = setup(&mut other);
    let mut b = DomBatch::new();
    b.set_command_for_element(n, Some(foreign.into()));
    assert_eq!(dom.apply(b, &cancel()).unwrap_err(), DomError::ForeignOwner);
    let mut b = DomBatch::new();
    b.set_command_for_element(d, Some(n.into()));
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::ButtonInapplicable
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    let mut b = DomBatch::new();
    b.set_command_for_element(n, Some(n.into()));
    apply(&mut dom, b);
    assert_eq!(dom.usage().string_code_units(), 75);
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.set_command_for_element(n, Some(n.into()));
    assert!(apply(&mut dom, b).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), old.stamp());
}
#[test]
fn button_submit_and_candidacy_queries_consume_actual_attrs_parent_and_association() {
    let mut dom = owner();
    let (d, n) = setup(&mut dom);
    let mut b = DomBatch::new();
    b.set_attribute(n, "formmethod", "post");
    apply(&mut dom, b);
    let c = supplement(capture(&dom.snapshot(), d));
    assert_eq!(
        fact(&c, n, HtmlSemanticQuery::ConstraintValidationCandidate),
        HtmlSemanticFact::ConstraintValidationCandidate(HtmlSemanticValue::Known(true))
    );
    let HtmlSemanticFact::SubmitterFormMethod(HtmlSemanticValue::Known(method)) =
        fact(&c, n, HtmlSemanticQuery::SubmitterFormMethod)
    else {
        panic!("submit context")
    };
    assert!(matches!(
        method.selection(),
        HtmlFormMethodSelection::Override(HtmlFormMethod::Post)
    ));
    let mut b = DomBatch::new();
    b.set_attribute(n, "commandfor", "");
    apply(&mut dom, b);
    let c = supplement(capture(&dom.snapshot(), d));
    assert_eq!(
        fact(&c, n, HtmlSemanticQuery::ConstraintValidationCandidate),
        HtmlSemanticFact::ConstraintValidationCandidate(HtmlSemanticValue::Known(false))
    );
    let mut b = DomBatch::new();
    b.set_attribute(n, "type", "submit");
    apply(&mut dom, b);
    let c = capture(&dom.snapshot(), d);
    assert_eq!(
        fact(&c, n, HtmlSemanticQuery::ConstraintValidationCandidate),
        HtmlSemanticFact::ConstraintValidationCandidate(HtmlSemanticValue::Known(true))
    );
    let mut b = DomBatch::new();
    b.set_attribute(n, "disabled", "");
    apply(&mut dom, b);
    let c = capture(&dom.snapshot(), d);
    assert_eq!(
        fact(&c, n, HtmlSemanticQuery::ConstraintValidationCandidate),
        HtmlSemanticFact::ConstraintValidationCandidate(HtmlSemanticValue::Known(false))
    );
}

#[test]
fn weak_target_capture_closure_is_bounded_and_refusal_preserves_the_canonical_owner() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let a = b.create_document();
    let d = b.create_document();
    let n = b.create_element_ns(&a, Namespace::html(), "button");
    let target = b.create_element(&d, "target");
    b.set_command_for_element(&n, Some((&target).into()));
    let c = apply(&mut dom, b);
    let a = c.document(&a).unwrap();
    let n = c.created(&n).unwrap().unwrap();
    let target = c.created(&target).unwrap().unwrap();
    let s = dom.snapshot();
    let usage = dom.usage();
    assert!(matches!(
        s.html_capture(
            a,
            None,
            HtmlCaptureLimits::new(256, 4096, 256, 32768, 256, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(2, 1000000),
            &cancel()
        ),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Subjects,
            limit: 2
        })
    ));
    assert!(matches!(
        s.html_capture(
            a,
            None,
            HtmlCaptureLimits::new(256, 4096, 256, 32768, 256, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(256, 1),
            &cancel()
        ),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            limit: 1
        })
    ));
    assert_eq!(dom.snapshot().stamp(), s.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(button(&s, n).explicit_command_for(), Some(target));
    let c = supplement(capture(&s, a));
    assert_eq!(c.node(&c.subject(target).unwrap()).unwrap(), target);
    assert_eq!(associated(&c, n, false).target(), None);
}
