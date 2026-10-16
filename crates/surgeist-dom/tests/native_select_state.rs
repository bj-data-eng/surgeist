//! Literal HTML877d Select-family outcomes on the real canonical owner.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomLimits {
    DomLimits::new(512, 32768, 2048, 2048, 256, 16, 1_000_000)
        .unwrap()
        .with_select_processing(128, 4_000_000, 10_000_000)
        .unwrap()
        .with_select_value_queries(
            surgeist_html_semantics::HtmlCaptureLimits::new(512, 2048, 2048, 32768, 512, 0, 0),
            surgeist_html_semantics::HtmlSemanticLimits::new(
                10_000_000, 1, 2048, 0, 4_000_000, 2048,
            ),
        )
}
fn checkpoint() -> DomNativeCheckpointLimits {
    DomNativeCheckpointLimits::new(
        4_000_000,
        10_000_000,
        DomSlotQueryLimits::new(512, 2048).unwrap(),
    )
    .unwrap()
}
fn apply(d: &mut Dom, b: DomBatch) -> DomCommit {
    d.apply(b, &cancel()).unwrap()
}
struct Fixture {
    dom: Dom,
    doc: DocumentId,
    select: NodeId,
    a: NodeId,
    b: NodeId,
    content: NodeId,
}
fn fixture(multiple: bool, connected: bool) -> Fixture {
    let mut dom = Dom::new(limits()).unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let s = b.create_element_ns(&d, Namespace::html(), "select");
    if multiple {
        b.set_attribute(&s, "multiple", "");
    }
    let a = b.create_element_ns(&d, Namespace::html(), "option");
    let at = b.create_text(&d, " A\t value ");
    b.insert_before(&at, &a, None);
    b.set_attribute(&a, "selected", "");
    let z = b.create_element_ns(&d, Namespace::html(), "option");
    let zt = b.create_text(&d, "B");
    b.insert_before(&zt, &z, None);
    b.set_attribute(&z, "value", "");
    let c = b.create_element_ns(&d, Namespace::html(), "selectedcontent");
    b.insert_before(&a, &s, None);
    b.insert_before(&z, &s, None);
    b.insert_before(&c, &s, None);
    if connected {
        b.insert_before(&s, &d, None);
    }
    let out = apply(&mut dom, b);
    Fixture {
        doc: out.document(&d).unwrap(),
        select: out.created(&s).unwrap().unwrap(),
        a: out.created(&a).unwrap().unwrap(),
        b: out.created(&z).unwrap().unwrap(),
        content: out.created(&c).unwrap().unwrap(),
        dom,
    }
}
fn selected(s: &DomSnapshot, id: NodeId) -> bool {
    s.node(id)
        .unwrap()
        .as_option()
        .unwrap()
        .state()
        .flags()
        .selectedness()
}
fn task(d: &mut Dom) -> DomSelectTask {
    match d.native_checkpoint_step(checkpoint(), &cancel()).unwrap() {
        DomNativeCheckpoint::Select(t) => t,
        x => panic!("expected Select work, got {x:?}"),
    }
}
fn text(s: &DomSnapshot, id: NodeId) -> DocumentString {
    let children = s.node(id).unwrap().children();
    assert_eq!(children.len(), 1);
    match s.node(children[0]).unwrap().kind() {
        DomNodeKind::Text(t) => t.clone(),
        _ => panic!("Text"),
    }
}

#[test]
fn fresh_flags_cache_and_form_owner_are_real_and_single() {
    let mut f = fixture(false, false);
    let s = f.dom.snapshot();
    assert!(selected(&s, f.a));
    assert!(!selected(&s, f.b));
    assert_eq!(
        s.node(f.a)
            .unwrap()
            .as_option()
            .unwrap()
            .cached_select()
            .unwrap(),
        Some(f.select)
    );
    assert!(
        !s.node(f.select)
            .unwrap()
            .as_select()
            .unwrap()
            .state()
            .flags()
            .user_validity()
    );
    assert!(
        !s.node(f.content)
            .unwrap()
            .as_selectedcontent()
            .unwrap()
            .disabled()
    );
    let mut b = DomBatch::new();
    let form = b.create_element_ns(f.doc, Namespace::html(), "form");
    b.insert_before(f.select, &form, None);
    let c = apply(&mut f.dom, b);
    assert_eq!(
        f.dom
            .snapshot()
            .node(f.select)
            .unwrap()
            .as_select()
            .unwrap()
            .form_owner()
            .unwrap(),
        c.created(&form).unwrap()
    );
}
#[test]
fn selected_attributes_dirty_setter_reset_and_multiple_removal_follow_distinct_orders() {
    let mut f = fixture(true, false);
    let mut b = DomBatch::new();
    b.set_attribute(f.b, "selected", "");
    apply(&mut f.dom, b);
    assert!(selected(&f.dom.snapshot(), f.a));
    assert!(selected(&f.dom.snapshot(), f.b));
    let mut b = DomBatch::new();
    b.remove_attribute(f.select, "multiple");
    apply(&mut f.dom, b);
    assert!(selected(&f.dom.snapshot(), f.a));
    assert!(!selected(&f.dom.snapshot(), f.b));
    let mut b = DomBatch::new();
    b.set_option_selected(f.b, true);
    b.remove_attribute(f.b, "selected");
    apply(&mut f.dom, b);
    assert!(!selected(&f.dom.snapshot(), f.a));
    assert!(selected(&f.dom.snapshot(), f.b));
    assert!(
        f.dom
            .snapshot()
            .node(f.b)
            .unwrap()
            .as_option()
            .unwrap()
            .state()
            .flags()
            .dirtiness()
    );
    let mut b = DomBatch::new();
    b.reset_select(f.select);
    apply(&mut f.dom, b);
    assert!(selected(&f.dom.snapshot(), f.a));
    assert!(!selected(&f.dom.snapshot(), f.b));
    assert!(
        !f.dom
            .snapshot()
            .node(f.b)
            .unwrap()
            .as_option()
            .unwrap()
            .state()
            .flags()
            .dirtiness()
    );
}
#[test]
fn value_uses_canonical_option_text_and_empty_attribute_then_nomatch_stays_unselected() {
    let mut f = fixture(false, false);
    let mut b = DomBatch::new();
    b.set_select_value(f.select, "A value");
    apply(&mut f.dom, b);
    assert!(selected(&f.dom.snapshot(), f.a));
    let mut b = DomBatch::new();
    b.set_select_value(f.select, "");
    apply(&mut f.dom, b);
    assert!(!selected(&f.dom.snapshot(), f.a));
    assert!(selected(&f.dom.snapshot(), f.b));
    let mut b = DomBatch::new();
    b.set_select_value(f.select, "missing");
    apply(&mut f.dom, b);
    assert!(!selected(&f.dom.snapshot(), f.a));
    assert!(!selected(&f.dom.snapshot(), f.b));
    let mut b = DomBatch::new();
    b.set_select_selected_index(f.select, 0);
    apply(&mut f.dom, b);
    assert!(selected(&f.dom.snapshot(), f.a));
    let mut b = DomBatch::new();
    b.set_select_selected_index(f.select, -1);
    apply(&mut f.dom, b);
    assert!(!selected(&f.dom.snapshot(), f.a));
    assert!(!selected(&f.dom.snapshot(), f.b));
}
#[test]
fn real_selectedcontent_clone_detaches_old_children_and_negative_connection_does_not_clone() {
    let mut f = fixture(false, false);
    assert!(
        f.dom
            .snapshot()
            .node(f.content)
            .unwrap()
            .children()
            .is_empty()
    );
    let mut b = DomBatch::new();
    b.insert_before(f.select, f.doc, None);
    apply(&mut f.dom, b);
    let old = f.dom.snapshot();
    let old_child = old.node(f.content).unwrap().children()[0];
    assert_eq!(text(&old, f.content), " A\t value ".into());
    assert_ne!(old_child, old.node(f.a).unwrap().children()[0]);
    let mut b = DomBatch::new();
    b.set_select_selected_index(f.select, 1);
    apply(&mut f.dom, b);
    let s = f.dom.snapshot();
    assert_eq!(text(&s, f.content), "B".into());
    assert_eq!(s.node(old_child).unwrap().parent(), None);
    assert_eq!(text(&old, f.content), " A\t value ".into());
    let mut b = DomBatch::new();
    b.set_select_selected_index(f.select, -1);
    apply(&mut f.dom, b);
    assert!(
        f.dom
            .snapshot()
            .node(f.content)
            .unwrap()
            .children()
            .is_empty()
    );
}
#[test]
fn removal_tasks_preserve_duplicates_and_authorization_rejects_replay_foreign_and_failure() {
    let mut f = fixture(false, true);
    let mut b = DomBatch::new();
    b.detach(f.a);
    b.insert_before(f.a, f.select, None);
    b.detach(f.a);
    apply(&mut f.dom, b);
    assert_eq!(f.dom.pending_native_tasks(), 2);
    let t = task(&mut f.dom);
    let duplicate = t.clone();
    let same = task(&mut f.dom);
    assert_eq!(t.operation(), same.operation());
    let before = f.dom.snapshot();
    let tiny =
        DomNativeCheckpointLimits::new(0, 0, DomSlotQueryLimits::new(0, 0).unwrap()).unwrap();
    assert!(f.dom.complete_select_task(&t, tiny, &cancel()).is_err());
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_eq!(f.dom.pending_native_tasks(), 2);
    let mut foreign = Dom::new(limits()).unwrap();
    assert_eq!(
        foreign
            .complete_select_task(&t, checkpoint(), &cancel())
            .unwrap_err(),
        DomError::ForeignOwner
    );
    assert_eq!(
        f.dom
            .complete_select_task(&t, checkpoint(), &cancel())
            .unwrap()
            .outcome(),
        DomSelectTaskOutcome::Updated
    );
    assert_eq!(
        f.dom
            .complete_select_task(&duplicate, checkpoint(), &cancel())
            .unwrap_err(),
        DomError::InvalidNativeTask
    );
    assert_eq!(f.dom.pending_native_tasks(), 1);
    let last = task(&mut f.dom);
    assert_ne!(last.operation(), t.operation());
    assert_eq!(text(&f.dom.snapshot(), f.content), "B".into());
}
#[test]
fn dropped_permissions_leave_head_pending_and_retirement_skips_historical_handle_once() {
    let mut f = fixture(false, true);
    let mut b = DomBatch::new();
    b.detach(f.a);
    apply(&mut f.dom, b);
    let t = task(&mut f.dom);
    let op = t.operation();
    drop(t);
    let t = task(&mut f.dom);
    assert_eq!(t.operation(), op);
    let mut b = DomBatch::new();
    b.detach(f.select);
    b.destroy(f.select);
    let c = apply(&mut f.dom, b);
    assert_eq!(f.dom.pending_native_tasks(), 0);
    assert!(
        c.changes()
            .changes()
            .contains(&DomChange::SelectTaskCancelled {
                operation: op,
                select: f.select,
                reason: DomSelectTaskCancellationReason::TargetDestroyed
            })
    );
    let mut b = DomBatch::new();
    let replacement = b.create_element_ns(f.doc, Namespace::html(), "select");
    apply(&mut f.dom, b);
    assert_eq!(
        f.dom
            .complete_select_task(&t, checkpoint(), &cancel())
            .unwrap()
            .outcome(),
        DomSelectTaskOutcome::SkippedTargetDestroyed
    );
    assert_eq!(
        f.dom
            .complete_select_task(&t, checkpoint(), &cancel())
            .unwrap_err(),
        DomError::InvalidNativeTask
    );
    assert!(t.source().node(f.select).is_ok());
    let _ = replacement;
}
#[test]
fn entirely_private_creation_tasks_and_retirement_are_a_canonical_noop() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    let s = b.create_element_ns(d, Namespace::html(), "select");
    let a = b.create_element_ns(d, Namespace::html(), "option");
    let z = b.create_element_ns(d, Namespace::html(), "option");
    let sc = b.create_element_ns(d, Namespace::html(), "selectedcontent");
    b.insert_before(&a, &s, None);
    b.insert_before(&z, &s, None);
    b.insert_before(&sc, &s, None);
    b.destroy(&s);
    let c = apply(&mut dom, b);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(c.changes().changes().is_empty());
    assert_eq!(dom.pending_native_tasks(), 0);
    assert_eq!(c.created(&s).unwrap(), None);
}

#[test]
fn optgroup_disabled_uses_the_shared_nearest_select_barriers() {
    for (barrier, nested_groups) in [("datalist", 0), ("hr", 0), ("option", 0), ("optgroup", 1)] {
        let mut f = fixture(false, false);
        let mut b = DomBatch::new();
        let wrapper = b.create_element_ns(f.doc, Namespace::html(), barrier);
        let group = b.create_element_ns(f.doc, Namespace::html(), "optgroup");
        if nested_groups == 1 {
            let inner = b.create_element_ns(f.doc, Namespace::html(), "optgroup");
            b.insert_before(&group, &inner, None);
            b.insert_before(&inner, &wrapper, None);
        } else {
            b.insert_before(&group, &wrapper, None);
        }
        b.insert_before(&wrapper, f.select, None);
        let c = apply(&mut f.dom, b);
        let group = c.created(&group).unwrap().unwrap();
        let mut b = DomBatch::new();
        b.set_select_selected_index(f.select, -1);
        apply(&mut f.dom, b);
        let mut b = DomBatch::new();
        b.set_attribute(group, "disabled", "");
        apply(&mut f.dom, b);
        assert!(!selected(&f.dom.snapshot(), f.a), "{barrier}");
        let mut b = DomBatch::new();
        b.remove_attribute(group, "disabled");
        apply(&mut f.dom, b);
        assert!(!selected(&f.dom.snapshot(), f.a), "{barrier}");
    }
    // A foreign element whose local name is a barrier does not establish one.
    let mut f = fixture(false, false);
    let mut b = DomBatch::new();
    let wrapper = b.create_element_ns(f.doc, Namespace::from_uri("urn:foreign"), "option");
    let group = b.create_element_ns(f.doc, Namespace::html(), "optgroup");
    b.insert_before(&group, &wrapper, None);
    b.insert_before(&wrapper, f.select, None);
    let c = apply(&mut f.dom, b);
    let mut b = DomBatch::new();
    b.set_select_selected_index(f.select, -1);
    b.set_attribute(c.created(&group).unwrap().unwrap(), "disabled", "");
    apply(&mut f.dom, b);
    assert!(selected(&f.dom.snapshot(), f.a));
}

#[test]
fn issued_update_uses_current_state_after_detach_adopt_and_cancel_retry() {
    let mut f = fixture(false, true);
    let mut b = DomBatch::new();
    b.detach(f.a);
    apply(&mut f.dom, b);
    let t = task(&mut f.dom);
    let mut b = DomBatch::new();
    let other = b.create_document();
    b.adopt(f.select, &other);
    b.set_attribute(f.b, "value", "current");
    b.set_select_selected_index(f.select, -1);
    apply(&mut f.dom, b);
    assert!(
        f.dom
            .snapshot()
            .node(f.content)
            .unwrap()
            .children()
            .is_empty()
    );
    let before = f.dom.snapshot();
    assert_eq!(
        f.dom
            .complete_select_task(&t, checkpoint(), &AtomicBool::new(true))
            .unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_eq!(f.dom.pending_native_tasks(), 1);
    f.dom
        .complete_select_task(&t, checkpoint(), &cancel())
        .unwrap();
    assert!(
        f.dom
            .snapshot()
            .node(f.content)
            .unwrap()
            .children()
            .is_empty()
    );
    assert_eq!(f.dom.pending_native_tasks(), 0);
    assert_eq!(t.select(), f.select);
}

#[test]
fn retiring_select_cancels_all_its_occurrences_after_cleanup_and_preserves_survivors() {
    let mut f = fixture(false, false);
    let mut b = DomBatch::new();
    let t = b.create_element_ns(f.doc, Namespace::html(), "select");
    let a = b.create_element_ns(f.doc, Namespace::html(), "option");
    let z = b.create_element_ns(f.doc, Namespace::html(), "option");
    b.insert_before(&a, &t, None);
    b.insert_before(&z, &t, None);
    let c = apply(&mut f.dom, b);
    let t = c.created(&t).unwrap().unwrap();
    let a = c.created(&a).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.detach(f.a); // S
    b.detach(a); // T
    b.insert_before(f.a, f.select, None);
    b.detach(f.a); // S
    b.insert_before(a, t, None);
    b.detach(a); // T
    apply(&mut f.dom, b);
    let handed = task(&mut f.dom);
    let mut b = DomBatch::new();
    b.destroy(f.select); // cleanup removes selected B and creates another S task
    let c = apply(&mut f.dom, b);
    assert_eq!(f.dom.pending_native_tasks(), 2);
    let cancellations: Vec<_> = c
        .changes()
        .changes()
        .iter()
        .filter_map(|change| {
            if let DomChange::SelectTaskCancelled {
                operation,
                select,
                reason: DomSelectTaskCancellationReason::TargetDestroyed,
            } = change
            {
                Some((*operation, *select))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(cancellations.len(), 3);
    assert!(cancellations.iter().all(|(_, target)| *target == f.select));
    assert_eq!(cancellations[0].0, handed.operation());
    for _ in 0..2 {
        let survivor = task(&mut f.dom);
        assert_eq!(survivor.select(), t);
        f.dom
            .complete_select_task(&survivor, checkpoint(), &cancel())
            .unwrap();
    }
    let before = f.dom.snapshot();
    let tiny =
        DomNativeCheckpointLimits::new(0, 0, DomSlotQueryLimits::new(0, 0).unwrap()).unwrap();
    assert!(
        f.dom
            .complete_select_task(&handed, tiny, &cancel())
            .is_err()
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        f.dom
            .complete_select_task(&handed, checkpoint(), &cancel())
            .unwrap()
            .outcome(),
        DomSelectTaskOutcome::SkippedTargetDestroyed
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_eq!(f.dom.pending_native_tasks(), 0);
}

#[test]
fn option_and_selectedcontent_retirement_leave_surviving_select_task() {
    let mut f = fixture(false, false);
    let mut b = DomBatch::new();
    b.detach(f.a);
    apply(&mut f.dom, b);
    let t = task(&mut f.dom);
    let mut b = DomBatch::new();
    b.destroy(f.a);
    b.detach(f.content);
    b.destroy(f.content);
    apply(&mut f.dom, b);
    assert_eq!(f.dom.pending_native_tasks(), 1);
    assert_eq!(
        f.dom
            .complete_select_task(&t, checkpoint(), &cancel())
            .unwrap()
            .outcome(),
        DomSelectTaskOutcome::Updated
    );
}

#[test]
fn slot_marker_and_select_updates_share_fifo_and_snapshot_signals_at_execution() {
    use surgeist_html::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment};
    let mut f = fixture(false, false);
    let mut b = DomBatch::new();
    let host = b.create_element_ns(f.doc, Namespace::html(), "div");
    let root = b.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Manual,
            false,
            false,
            false,
        ),
    );
    let slot_a = b.create_element_ns(f.doc, Namespace::html(), "slot");
    let slot_b = b.create_element_ns(f.doc, Namespace::html(), "slot");
    b.insert_before(&slot_a, &root, None);
    b.insert_before(&slot_b, &root, None);
    let x = b.create_text(f.doc, "x");
    let y = b.create_text(f.doc, "y");
    b.insert_before(&x, &host, None);
    b.insert_before(&y, &host, None);
    let c = apply(&mut f.dom, b);
    let a = c.created(&slot_a).unwrap().unwrap();
    let bslot = c.created(&slot_b).unwrap().unwrap();
    let x = c.created(&x).unwrap().unwrap();
    let y = c.created(&y).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.detach(f.a); // Select S1
    b.assign_slot(a, vec![x.into()]); // one Slot marker
    b.insert_before(f.a, f.select, None);
    b.detach(f.a); // Select S2
    apply(&mut f.dom, b);
    assert_eq!(f.dom.pending_native_tasks(), 3);
    let first = task(&mut f.dom);
    let mut b = DomBatch::new();
    b.assign_slot(bslot, vec![y.into()]);
    b.assign_slot(a, vec![x.into()]); // no duplicate recipient/marker
    apply(&mut f.dom, b);
    assert_eq!(f.dom.pending_native_tasks(), 3);
    f.dom
        .complete_select_task(&first, checkpoint(), &cancel())
        .unwrap();
    let DomNativeCheckpoint::Slots(notifications) = f
        .dom
        .native_checkpoint_step(checkpoint(), &cancel())
        .unwrap()
    else {
        panic!("Slot marker precedes S2")
    };
    assert_eq!(notifications.targets(), &[a, bslot]);
    let second = task(&mut f.dom);
    assert_ne!(first.operation(), second.operation());
    let mut b = DomBatch::new();
    b.assign_slot(a, vec![]);
    apply(&mut f.dom, b);
    f.dom
        .complete_select_task(&second, checkpoint(), &cancel())
        .unwrap();
    let DomNativeCheckpoint::Slots(notifications) = f
        .dom
        .native_checkpoint_step(checkpoint(), &cancel())
        .unwrap()
    else {
        panic!("new Slot marker")
    };
    assert_eq!(notifications.targets(), &[a]);
}

#[test]
fn selectedcontent_deep_clone_runs_real_control_template_and_shadow_hooks() {
    use surgeist_html::{HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment};
    let mut f = fixture(false, true);
    let mut b = DomBatch::new();
    let input = b.create_element_ns(f.doc, Namespace::html(), "input");
    b.set_input_value(&input, "current input");
    b.set_custom_validity(&input, "source message");
    let textarea = b.create_element_ns(f.doc, Namespace::html(), "textarea");
    b.set_textarea_value(&textarea, "dirty textarea");
    let output = b.create_element_ns(f.doc, Namespace::html(), "output");
    b.set_output_value(&output, "output text");
    let template = b.create_element_ns(f.doc, Namespace::html(), "template");
    let host = b.create_element_ns(f.doc, Namespace::html(), "div");
    let shadow = b.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Named,
            false,
            true,
            false,
        ),
    );
    let shadow_text = b.create_text(f.doc, "shadow text");
    b.insert_before(&shadow_text, &shadow, None);
    for n in [&input, &textarea, &output, &template, &host] {
        b.insert_before(n, f.a, None);
    }
    let c = apply(&mut f.dom, b);
    let template = c.created(&template).unwrap().unwrap();
    let contents = f
        .dom
        .snapshot()
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let mut b = DomBatch::new();
    let template_text = b.create_text(f.doc, "template text");
    b.insert_before(&template_text, contents, None);
    b.set_select_selected_index(f.select, 0);
    apply(&mut f.dom, b);
    let s = f.dom.snapshot();
    let copies = s.node(f.content).unwrap().children();
    assert_eq!(copies.len(), 6); // original Option Text plus five real Elements
    assert_eq!(
        s.node(copies[1]).unwrap().as_input().unwrap().value(),
        "current input".encode_utf16().collect::<Vec<_>>()
    );
    assert!(
        s.node(copies[1])
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .custom_validity_message()
            .is_empty()
    );
    assert_eq!(
        s.node(copies[2])
            .unwrap()
            .as_textarea()
            .unwrap()
            .state()
            .raw_value(),
        &DocumentString::from("dirty textarea")
    );
    assert!(
        s.node(copies[2])
            .unwrap()
            .as_textarea()
            .unwrap()
            .state()
            .flags()
            .dirty_value()
    );
    assert_eq!(
        s.node(copies[3])
            .unwrap()
            .as_output()
            .unwrap()
            .state()
            .default_value_override(),
        None
    );
    assert_eq!(text(&s, copies[3]), "output text".into());
    let template_contents = s
        .node(copies[4])
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    assert_ne!(template_contents, contents);
    assert_eq!(text(&s, template_contents), "template text".into());
    let shadow = s.node(copies[5]).unwrap().hosted_shadow_root().unwrap();
    assert_eq!(text(&s, shadow), "shadow text".into());
}

#[test]
fn deep_selectedcontent_copy_is_iterative_and_reclaims_only_its_private_wrapper() {
    let mut f = fixture(false, true);
    let before = f.dom.usage().managed_nodes();
    let mut b = DomBatch::new();
    let first = b.create_element_ns(f.doc, Namespace::html(), "span");
    let mut previous = &first;
    let mut chain = Vec::new();
    for _ in 0..100 {
        chain.push(b.create_element_ns(f.doc, Namespace::html(), "span"));
    }
    for node in &chain {
        b.insert_before(node, previous, None);
        previous = node;
    }
    let leaf = b.create_text(f.doc, "leaf");
    b.insert_before(&leaf, previous, None);
    b.insert_before(&first, f.a, None);
    b.set_select_selected_index(f.select, 0);
    apply(&mut f.dom, b);
    // 102 source nodes, their 102 copies, and a copy of the original Option Text.
    // The replaced old child stays managed; the private emptied Fragment does not.
    assert_eq!(f.dom.usage().managed_nodes(), before + 205);
    let s = f.dom.snapshot();
    let mut node = s.node(f.content).unwrap().children()[1];
    for _ in 0..101 {
        node = s.node(node).unwrap().children()[0];
    }
    assert_eq!(
        s.node(node).unwrap().kind(),
        &DomNodeKind::Text("leaf".into())
    );
}

#[test]
fn reached_node_and_pending_task_limits_refuse_without_changing_canonical_state() {
    let mut f = fixture(false, true);
    let configured = limits();
    f.dom
        .set_limits(
            configured
                .with_select_processing(0, 4_000_000, 10_000_000)
                .unwrap(),
        )
        .unwrap();
    let before = f.dom.snapshot();
    let usage = f.dom.usage();
    let mut b = DomBatch::new();
    b.detach(f.a);
    assert_eq!(
        f.dom.apply(b, &cancel()).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::SelectTasks,
            limit: 0
        }
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_eq!(f.dom.usage(), usage);
    let count = f.dom.usage().managed_nodes();
    f.dom
        .set_limits(
            DomLimits::new(count, 32768, 2048, 2048, 256, 16, 1_000_000)
                .unwrap()
                .with_select_processing(128, 4_000_000, 10_000_000)
                .unwrap(),
        )
        .unwrap();
    let mut b = DomBatch::new();
    b.set_select_selected_index(f.select, 1);
    assert_eq!(
        f.dom.apply(b, &cancel()).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            limit: count
        }
    );
    assert_eq!(f.dom.snapshot().stamp(), before.stamp());
    assert_eq!(f.dom.usage(), usage);
    assert!(selected(&f.dom.snapshot(), f.a));
    assert_eq!(text(&f.dom.snapshot(), f.content), " A\t value ".into());
}

#[test]
fn initial_and_slot_capture_keep_actual_option_flags_and_select_form_association() {
    use surgeist_html_semantics::*;
    fn capture(s: &DomSnapshot, d: DocumentId) -> DomHtmlCapture {
        s.html_capture(
            d,
            None,
            HtmlCaptureLimits::new(512, 2048, 2048, 32768, 512, 0, 0),
            DomHtmlCaptureLimits::new(512, 1_000_000),
            &cancel(),
        )
        .unwrap()
    }
    fn fact(c: &DomHtmlCapture, n: NodeId, q: HtmlSemanticQuery) -> HtmlSemanticFact {
        c.query(
            &[HtmlSemanticRequest::new(c.subject(n).unwrap(), q)],
            surgeist_html::HtmlValueInterpreter::new(32768),
            HtmlSemanticLimits::new(1_000_000, 16, 2048, 512, 4_000_000, 2048),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .results()[0]
            .fact()
            .clone()
    }
    let mut f = fixture(false, true);
    let old_snapshot = f.dom.snapshot();
    let old = capture(&old_snapshot, f.doc);
    let mut b = DomBatch::new();
    let form = b.create_element_ns(f.doc, Namespace::html(), "form");
    b.insert_before(f.select, &form, None);
    b.insert_before(&form, f.doc, None);
    b.set_custom_validity(f.select, "message\r\nline");
    b.set_option_selected(f.b, true);
    let c = apply(&mut f.dom, b);
    let form = c.created(&form).unwrap().unwrap();
    let s = f.dom.snapshot();
    assert_eq!(
        s.node(f.select)
            .unwrap()
            .as_select()
            .unwrap()
            .state()
            .custom_validity_message(),
        &DocumentString::from("message\nline")
    );
    assert!(
        old_snapshot
            .node(f.select)
            .unwrap()
            .as_select()
            .unwrap()
            .state()
            .custom_validity_message()
            .is_empty()
    );
    assert_eq!(
        fact(&old, f.a, HtmlSemanticQuery::CurrentSelectedness),
        HtmlSemanticFact::CurrentSelectedness(HtmlSemanticValue::Known(true))
    );
    for supplement in [false, true] {
        let c = capture(&s, f.doc);
        let c = if supplement {
            c.with_slot_semantics(
                DomSlotSemanticLimits::new(
                    DomSlotQueryLimits::new(512, 2048).unwrap(),
                    DomHtmlCaptureLimits::new(512, 1_000_000),
                    HtmlLanguageDirectionLimits::new(1024, 32768, 0, 1_000_000),
                    HtmlInertFactLimits::new(1024, 1_000_000),
                ),
                &cancel(),
            )
            .unwrap()
        } else {
            c
        };
        assert_eq!(
            fact(&c, f.a, HtmlSemanticQuery::CurrentSelectedness),
            HtmlSemanticFact::CurrentSelectedness(HtmlSemanticValue::Known(false))
        );
        assert_eq!(
            fact(&c, f.b, HtmlSemanticQuery::CurrentSelectedness),
            HtmlSemanticFact::CurrentSelectedness(HtmlSemanticValue::Known(true))
        );
        assert_eq!(
            fact(&c, f.b, HtmlSemanticQuery::OptionDirtiness),
            HtmlSemanticFact::OptionDirtiness(HtmlSemanticValue::Known(true))
        );
        let HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(owner)) =
            fact(&c, f.select, HtmlSemanticQuery::FormOwner)
        else {
            panic!("actual Select owner")
        };
        assert_eq!(owner.owner(), Some(&c.subject(form).unwrap()));
    }
}
