//! HTML877d native Form scalars, canonical owner relations and real lifecycle.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlValueInterpreter;
use surgeist_html_semantics::*;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn dom() -> Dom {
    Dom::new(DomLimits::new(128, 16384, 256, 256, 128, 16, 65536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, b: DomBatch) -> DomCommit {
    dom.apply(b, &cancel()).unwrap()
}
fn owner(s: &DomSnapshot, n: NodeId) -> Option<NodeId> {
    let n = s.node(n).unwrap();
    n.as_button()
        .map(|b| b.form_owner().unwrap())
        .unwrap_or_else(|| n.as_fieldset().unwrap().form_owner().unwrap())
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
fn assert_owner(c: &DomHtmlCapture, n: NodeId, expected: Option<NodeId>) {
    let HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(value)) =
        fact(c, n, HtmlSemanticQuery::FormOwner)
    else {
        panic!("known current owner")
    };
    assert_eq!(value.origin(), HtmlFormOwnerOrigin::EstablishedCurrent);
    assert_eq!(value.owner().map(|id| c.node(id).unwrap()), expected);
}
#[test]
fn same_batch_form_and_controls_have_actual_relations_and_fresh_scalars() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let f = b.create_element_ns(&d, Namespace::html(), "form");
    let field = b.create_element_ns(&d, Namespace::html(), "fieldset");
    let button = b.create_element_ns(&d, Namespace::html(), "button");
    let custom = b.create_element_ns(&d, Namespace::html(), "x-widget");
    b.insert_before(&f, &d, None);
    b.insert_before(&field, &f, None);
    b.insert_before(&button, &field, None);
    b.insert_before(&custom, &field, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let f = c.created(&f).unwrap().unwrap();
    let field = c.created(&field).unwrap().unwrap();
    let button = c.created(&button).unwrap().unwrap();
    let s = dom.snapshot();
    let form = s.node(f).unwrap().as_form().unwrap();
    assert_eq!(form.id(), f);
    assert_eq!(form.past_names().len(), 0);
    assert!(!form.state().locked_for_reset());
    assert!(!form.state().constructing_entry_list());
    assert!(!form.state().firing_submission_events());
    assert_eq!(owner(&s, field), Some(f));
    assert_eq!(owner(&s, button), Some(f));
    let c = capture(&s, d);
    for c in [
        c,
        capture(&s, d)
            .with_slot_semantics(
                DomSlotSemanticLimits::new(
                    DomSlotQueryLimits::new(256, 512).unwrap(),
                    DomHtmlCaptureLimits::new(256, 1000000),
                    HtmlLanguageDirectionLimits::new(512, 512, 32768, 1000000),
                    HtmlInertFactLimits::new(512, 1000000),
                ),
                &cancel(),
            )
            .unwrap(),
    ] {
        assert_owner(&c, field, Some(f));
        assert_owner(&c, button, Some(f));
        let HtmlSemanticFact::FormControls(HtmlSemanticValue::Known(value)) =
            fact(&c, f, HtmlSemanticQuery::FormControls)
        else {
            panic!("known complete collection")
        };
        assert_eq!(
            value
                .controls()
                .iter()
                .map(|id| c.node(id).unwrap())
                .collect::<Vec<_>>(),
            vec![field, button]
        );
        assert!(matches!(
            fact(&c, f, HtmlSemanticQuery::FormLength),
            HtmlSemanticFact::FormLength(HtmlSemanticValue::Known(2))
        ));
        let HtmlSemanticFact::FieldsetControls(HtmlSemanticValue::Known(value)) =
            fact(&c, field, HtmlSemanticQuery::FieldsetControls)
        else {
            panic!("known fieldset collection")
        };
        assert_eq!(
            value
                .controls()
                .iter()
                .map(|id| c.node(id).unwrap())
                .collect::<Vec<_>>(),
            vec![button]
        );
    }
}
#[test]
fn connected_first_id_blocks_ancestor_fallback_and_detached_explicit_uses_nearest() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let top = b.create_element_ns(&d, Namespace::html(), "div");
    let collision = b.create_element(&d, "form");
    let f = b.create_element_ns(&d, Namespace::html(), "form");
    let button = b.create_element_ns(&d, Namespace::html(), "button");
    b.set_attribute(&collision, "id", "chosen");
    b.set_attribute(&f, "id", "chosen");
    b.set_attribute(&button, "form", "chosen");
    b.insert_before(&button, &f, None); // detached: explicit form still uses nearest.
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let top = c.created(&top).unwrap().unwrap();
    let collision = c.created(&collision).unwrap().unwrap();
    let f = c.created(&f).unwrap().unwrap();
    let button = c.created(&button).unwrap().unwrap();
    assert_eq!(owner(&dom.snapshot(), button), Some(f));
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.insert_before(top, d, None);
    b.insert_before(collision, top, None);
    b.insert_before(f, top, None);
    apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), button), None);
    assert_eq!(owner(&old, button), Some(f));
    let mut b = DomBatch::new();
    b.detach(collision);
    apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), button), Some(f));
    for value in ["", "missing"] {
        let mut b = DomBatch::new();
        b.set_attribute(button, "form", value);
        apply(&mut dom, b);
        assert_eq!(owner(&dom.snapshot(), button), None);
    }
    let mut b = DomBatch::new();
    b.remove_attribute(button, "form");
    apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), button), Some(f));
    let mut b = DomBatch::new();
    b.set_attribute_ns(
        button,
        Namespace::from_uri("urn:other"),
        "p:form",
        "missing",
    );
    b.set_attribute(button, "FORM", "missing");
    apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), button), Some(f));
}
#[test]
fn external_control_nested_forms_move_and_id_events_use_actual_live_tree() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let top = b.create_element_ns(&d, Namespace::html(), "div");
    let a = b.create_element_ns(&d, Namespace::html(), "form");
    let inner = b.create_element_ns(&d, Namespace::html(), "form");
    let field = b.create_element_ns(&d, Namespace::html(), "fieldset");
    let button = b.create_element_ns(&d, Namespace::html(), "button");
    b.set_attribute(&a, "id", "a");
    b.set_attribute(&inner, "id", "inner");
    b.insert_before(&top, &d, None);
    b.insert_before(&a, &top, None);
    b.insert_before(&inner, &a, None);
    b.insert_before(&field, &inner, None);
    b.set_attribute(&button, "form", "a");
    b.insert_before(&button, &top, None);
    let c = apply(&mut dom, b);
    let a = c.created(&a).unwrap().unwrap();
    let inner = c.created(&inner).unwrap().unwrap();
    let field = c.created(&field).unwrap().unwrap();
    let button = c.created(&button).unwrap().unwrap();
    assert_eq!(owner(&dom.snapshot(), field), Some(inner));
    assert_eq!(owner(&dom.snapshot(), button), Some(a));
    let mut b = DomBatch::new();
    b.set_attribute(a, "id", "renamed");
    apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), button), None);
    let mut b = DomBatch::new();
    b.set_attribute(inner, "id", "a");
    apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), button), Some(inner));
    let mut b = DomBatch::new();
    b.insert_before(field, a, None);
    apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), field), Some(a));
    let mut b = DomBatch::new();
    b.detach(field);
    apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), field), None);
}
#[test]
fn deep_import_maps_private_created_form_owners_and_native_clone_starts_fresh() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let f = b.create_element_ns(&d, Namespace::html(), "form");
    let field = b.create_element_ns(&d, Namespace::html(), "fieldset");
    let button = b.create_element_ns(&d, Namespace::html(), "button");
    b.insert_before(&field, &f, None);
    b.insert_before(&button, &field, None);
    b.set_custom_validity(&button, "old\r\nmessage");
    let c = apply(&mut dom, b);
    let f = c.created(&f).unwrap().unwrap();
    let source = dom.snapshot();
    let mut b = DomBatch::new();
    let target = b.create_document();
    let copy = b.import_node(&source, f, &target, true);
    let extra = b.create_comment(&target, "ticket after private children");
    let c = apply(&mut dom, b);
    let copy = c.created(&copy).unwrap().unwrap();
    let extra = c.created(&extra).unwrap().unwrap();
    let s = dom.snapshot();
    let field = s.node(copy).unwrap().children()[0];
    let button = s.node(field).unwrap().children()[0];
    assert_eq!(owner(&s, field), Some(copy));
    assert_eq!(owner(&s, button), Some(copy));
    assert!(s.node(copy).unwrap().as_form().unwrap().past_names().len() == 0);
    assert_eq!(
        s.node(button)
            .unwrap()
            .as_button()
            .unwrap()
            .state()
            .custom_validity_message(),
        &DocumentString::from("")
    );
    assert!(matches!(
        s.node(extra).unwrap().kind(),
        DomNodeKind::Comment(_)
    ));
}
#[test]
fn whole_adoption_preserves_relations_and_split_adoption_clears_them_atomically() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let target = b.create_document();
    let f = b.create_element_ns(&d, Namespace::html(), "form");
    let field = b.create_element_ns(&d, Namespace::html(), "fieldset");
    b.insert_before(&field, &f, None);
    b.set_custom_validity(&field, "message");
    let c = apply(&mut dom, b);
    let f = c.created(&f).unwrap().unwrap();
    let field = c.created(&field).unwrap().unwrap();
    let target = c.document(&target).unwrap();
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.adopt(f, target);
    let c = apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), field), Some(f));
    assert!(
        c.changes()
            .changes()
            .contains(&DomChange::NodeDocument(field))
    );
    let mut b = DomBatch::new();
    b.adopt(field, old.node(f).unwrap().owner_document().unwrap());
    apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), field), None);
    assert_eq!(owner(&old, field), Some(f));
    assert_eq!(
        dom.snapshot()
            .node(field)
            .unwrap()
            .as_fieldset()
            .unwrap()
            .state()
            .custom_validity_message(),
        &DocumentString::from("message")
    );
}
#[test]
fn ownership_changes_history_noops_and_late_rejection_are_atomic() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let f = b.create_element_ns(&d, Namespace::html(), "form");
    let button = b.create_element_ns(&d, Namespace::html(), "button");
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let f = c.created(&f).unwrap().unwrap();
    let button = c.created(&button).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.insert_before(button, f, None);
    b.insert_before(d, button, None);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(owner(&dom.snapshot(), button), None);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let mut b = DomBatch::new();
    b.insert_before(button, f, None);
    let c = apply(&mut dom, b);
    assert!(
        c.changes()
            .changes()
            .contains(&DomChange::ElementState(button))
    );
    assert_eq!(owner(&dom.snapshot(), button), Some(f));
    let stamp = dom.snapshot().stamp();
    let mut b = DomBatch::new();
    b.set_attribute(button, "form", "missing");
    apply(&mut dom, b); // detached nearest owner retained even explicit.
    let mut b = DomBatch::new();
    b.set_attribute(button, "form", "missing");
    assert!(apply(&mut dom, b).changes().changes().is_empty());
    assert_eq!(owner(&dom.snapshot(), button), Some(f));
    assert!(!dom.changes_since(stamp).unwrap().changes().is_empty());
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.detach(button);
    assert_eq!(
        dom.apply(b, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(owner(&dom.snapshot(), button), Some(f));
}
#[test]
fn form_default_button_and_implicit_condition_use_actual_current_owned_controls() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let form = b.create_element_ns(&d, Namespace::html(), "form");
    let first = b.create_element_ns(&d, Namespace::html(), "button");
    let second = b.create_element_ns(&d, Namespace::html(), "button");
    b.insert_before(&form, &d, None);
    b.insert_before(&first, &form, None);
    b.insert_before(&second, &form, None);
    b.set_attribute(&first, "disabled", "");
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let form = c.created(&form).unwrap().unwrap();
    let first = c.created(&first).unwrap().unwrap();
    let second = c.created(&second).unwrap().unwrap();
    let c = capture(&dom.snapshot(), d);
    let HtmlSemanticFact::FormDefaultButton(HtmlSemanticValue::Known(Some(value))) =
        fact(&c, form, HtmlSemanticQuery::FormDefaultButton)
    else {
        panic!("actual default Button")
    };
    assert_eq!(c.node(value.subject()).unwrap(), first);
    assert_eq!(
        value.form_owner().origin(),
        HtmlFormOwnerOrigin::EstablishedCurrent
    );
    let HtmlSemanticFact::FormImplicitSubmissionCondition(HtmlSemanticValue::Known(
        HtmlImplicitSubmissionCondition::DefaultButton(value),
    )) = fact(&c, form, HtmlSemanticQuery::FormImplicitSubmissionCondition)
    else {
        panic!("default condition")
    };
    assert_eq!(c.node(value.subject()).unwrap(), first);
    let mut b = DomBatch::new();
    b.set_attribute(first, "type", "button");
    apply(&mut dom, b);
    let c = capture(&dom.snapshot(), d);
    let HtmlSemanticFact::FormDefaultButton(HtmlSemanticValue::Known(Some(value))) =
        fact(&c, form, HtmlSemanticQuery::FormDefaultButton)
    else {
        panic!("second submit Button")
    };
    assert_eq!(c.node(value.subject()).unwrap(), second);
}
#[test]
fn actual_html_namespace_and_exact_local_name_select_form_owners() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let generic = b.create_element(&d, "form");
    let uppercase = b.create_element_ns(&d, Namespace::html(), "Form");
    let prefixed = b.create_element_ns(&d, Namespace::html(), "p:form");
    let a = b.create_element_ns(&d, Namespace::html(), "button");
    let z = b.create_element_ns(&d, Namespace::html(), "fieldset");
    b.insert_before(&a, &generic, None);
    b.insert_before(&z, &uppercase, None);
    let c = apply(&mut dom, b);
    let generic = c.created(&generic).unwrap().unwrap();
    let uppercase = c.created(&uppercase).unwrap().unwrap();
    let prefixed = c.created(&prefixed).unwrap().unwrap();
    let a = c.created(&a).unwrap().unwrap();
    let z = c.created(&z).unwrap().unwrap();
    let s = dom.snapshot();
    assert!(s.node(generic).unwrap().as_form().is_none());
    assert!(s.node(uppercase).unwrap().as_form().is_none());
    assert!(s.node(prefixed).unwrap().as_form().is_some());
    assert_eq!(owner(&s, a), None);
    assert_eq!(owner(&s, z), None);
    let mut b = DomBatch::new();
    b.insert_before(a, prefixed, None);
    b.insert_before(z, prefixed, None);
    apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), a), Some(prefixed));
    assert_eq!(owner(&dom.snapshot(), z), Some(prefixed));
}
#[test]
fn fragment_transfer_replacement_and_retirement_keep_relations_generation_exact() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let a = b.create_element_ns(&d, Namespace::html(), "form");
    let z = b.create_element_ns(&d, Namespace::html(), "form");
    let fragment = b.create_fragment(&d);
    let button = b.create_element_ns(&d, Namespace::html(), "button");
    let field = b.create_element_ns(&d, Namespace::html(), "fieldset");
    b.insert_before(&button, &fragment, None);
    b.insert_before(&field, &a, None);
    b.insert_before(&fragment, &z, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let a = c.created(&a).unwrap().unwrap();
    let z = c.created(&z).unwrap().unwrap();
    let button = c.created(&button).unwrap().unwrap();
    let field = c.created(&field).unwrap().unwrap();
    assert_eq!(owner(&dom.snapshot(), button), Some(z));
    let mut b = DomBatch::new();
    b.replace_child(button, a, field);
    apply(&mut dom, b);
    assert_eq!(owner(&dom.snapshot(), button), Some(a));
    assert_eq!(owner(&dom.snapshot(), field), None);
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    b.destroy(a);
    apply(&mut dom, b);
    assert_eq!(
        dom.snapshot().node(button).unwrap_err(),
        DomError::StaleNode
    );
    assert_eq!(owner(&old, button), Some(a));
    let mut b = DomBatch::new();
    let replacement = b.create_element_ns(d, Namespace::html(), "form");
    let c = apply(&mut dom, b);
    let replacement = c.created(&replacement).unwrap().unwrap();
    assert_ne!(replacement, a);
    assert_eq!(owner(&dom.snapshot(), field), None);
    assert_eq!(
        dom.snapshot()
            .node(replacement)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .len(),
        0
    );
}
#[test]
fn native_form_exact_string_and_node_limits_refuse_atomically_and_retirement_releases_units() {
    for cap in [62, 63] {
        let mut dom = Dom::new(DomLimits::new(2, cap, 8, 0, 0, 4, 4096).unwrap()).unwrap();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let f = b.create_element_ns(&d, Namespace::html(), "form");
        let result = dom.apply(b, &cancel());
        if cap == 62 {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 62
                }
            );
            assert_eq!(dom.usage().managed_nodes(), 0);
        } else {
            let c = result.unwrap();
            let f = c.created(&f).unwrap().unwrap();
            assert_eq!(dom.usage().string_code_units(), 63);
            let before = dom.snapshot();
            let mut b = DomBatch::new();
            b.set_attribute(f, "name", "x");
            assert_eq!(
                dom.apply(b, &cancel()).unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 63
                }
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            let mut b = DomBatch::new();
            b.destroy(f);
            apply(&mut dom, b);
            assert_eq!(dom.usage().string_code_units(), 31);
        }
    }
    let mut dom = Dom::new(DomLimits::new(1, 1024, 8, 0, 0, 4, 4096).unwrap()).unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    b.create_element_ns(&d, Namespace::html(), "form");
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            limit: 1
        }
    );
    assert_eq!(dom.usage().managed_nodes(), 0);
}
#[test]
fn form_capture_with_real_slot_supplement_keeps_owner_classification_and_directionality() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let form = b.create_element_ns(&d, Namespace::html(), "form");
    let host = b.create_element_ns(&d, Namespace::html(), "div");
    let button = b.create_element_ns(&d, Namespace::html(), "button");
    let text = b.create_text(&d, "\u{05d0}");
    b.insert_before(&form, &d, None);
    b.insert_before(&host, &form, None);
    b.insert_before(&button, &host, None);
    b.insert_before(&text, &host, None);
    let root = b.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let slot = b.create_element_ns(&d, Namespace::html(), "slot");
    b.set_attribute(&slot, "dir", "auto");
    b.insert_before(&slot, &root, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let form = c.created(&form).unwrap().unwrap();
    let button = c.created(&button).unwrap().unwrap();
    let slot = c.created(&slot).unwrap().unwrap();
    let c = capture(&dom.snapshot(), d)
        .with_slot_semantics(
            DomSlotSemanticLimits::new(
                DomSlotQueryLimits::new(256, 512).unwrap(),
                DomHtmlCaptureLimits::new(256, 1000000),
                HtmlLanguageDirectionLimits::new(512, 512, 32768, 1000000),
                HtmlInertFactLimits::new(512, 1000000),
            ),
            &cancel(),
        )
        .unwrap();
    assert_owner(&c, button, Some(form));
    assert_eq!(
        fact(&c, slot, HtmlSemanticQuery::Directionality),
        HtmlSemanticFact::Directionality(HtmlSemanticValue::Known(HtmlDirectionality::Rtl))
    );
}
