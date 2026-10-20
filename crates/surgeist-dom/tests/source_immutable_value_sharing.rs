//! Optional sharing through real Source restoration, publication and reentry.
//! Public backing observations accompany literal state and protocol outcomes.
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, atomic::AtomicBool},
};

use surgeist_dom::*;
use surgeist_html::{HtmlDialogAction, HtmlStyleUpdateAction};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn raw(value: &str) -> DocumentString {
    DocumentString::from_utf16(value.encode_utf16().collect::<Vec<_>>())
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(128_000_000, 1_000_000, 20_000_000, 64, 4096, 4096, 4)
}
fn owner() -> (Dom, DocumentId) {
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            1_000_000,
            128,
            0,
            0,
            64,
            8_000_000,
            DomNameLimits::new(32_000_000, 16_384, 10_000_000, 4_000_000).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(32, 1024, 8_000_000, 8_000_000, 1_000_000).unwrap(),
        ),
    )
    .unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let commit = dom.apply(setup, &cancel()).unwrap();
    let document = commit.document(&document).unwrap();
    (dom, document)
}
fn bank(snapshot: &DomSnapshot, node: NodeId) -> &[DomAttribute] {
    snapshot
        .node(node)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes()
}
fn text(snapshot: &DomSnapshot, node: NodeId) -> &DocumentString {
    snapshot.node(node).unwrap().data().unwrap()
}
fn complete_local(dom: &mut Dom, edit: DomSourceEdit) {
    let mut lease = dom
        .begin_source_edit(edit, source_limits(), &cancel())
        .unwrap();
    let id = lease.operation();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
        panic!("local Source edit must complete without an external service")
    };
    assert_eq!(done.operation(), id);
}
fn blocked_style_source(dom: &mut Dom, edit: DomSourceEdit, style: NodeId) {
    let (id, first, spent) = {
        let mut lease = dom
            .begin_source_edit(edit, source_limits(), &cancel())
            .unwrap();
        let id = lease.operation();
        let first = match lease.advance(&cancel()).unwrap() {
            DomSourceProgress::Request(DomSourceRequest::Style(first)) => first,
            other => panic!(
                "connected eligible Style must reach its real inline-CSP request, got {other:?}"
            ),
        };
        assert_eq!(first.source().element(), style);
        assert_eq!(
            first.action(),
            DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
        );
        assert!(lease.status().has_published());
        (id, first, lease.status().spent_work())
    };
    let mut lease = dom
        .resume_source_edit(id, source_limits(), &cancel())
        .unwrap();
    assert!(lease.status().spent_work() >= spent);
    let DomSourceProgress::Request(DomSourceRequest::Style(reissued)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("the original pending request must survive permission drop")
    };
    assert!(Arc::ptr_eq(first.source().text(), reissued.source().text()));
    assert_eq!(first.source().ordinal(), reissued.source().ordinal());
    // This is a supplied host policy evaluation of the actual request, exactly
    // as in native_style_state. It blocks creation; no CSSOM reply is invented.
    let observation = DomStyleInlineCspObservation::evaluated(
        reissued.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    );
    lease
        .accept_inline_csp(&reissued, &observation, &cancel())
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
        panic!("the supplied blocking policy must finish the actual Style hook")
    };
    assert_eq!(done.operation(), id);
}

#[test]
fn source_attribute_banks_share_and_text_survives_preflight_restore_and_replay() {
    let (mut dom, document) = owner();
    let mut setup = DomBatch::new();
    let seed = setup.create_element_ns(document, Namespace::html(), "div");
    setup.set_attribute(&seed, "type", raw("text/css"));
    setup.set_attribute(&seed, "data-shared", raw("kept"));
    let style = setup.create_element_ns(document, Namespace::html(), "style");
    setup.set_attribute(&style, "type", raw("text/plain"));
    setup.set_attribute(&style, "data-shared", raw("kept"));
    let child = setup.create_text(document, raw("a{}"));
    setup.insert_before(&child, &style, None);
    setup.insert_before(&style, document, None);
    let seed_text = setup.create_text(document, raw("b{}"));
    let done = dom.apply(setup, &cancel()).unwrap();
    let seed = done.created(&seed).unwrap().unwrap();
    let style = done.created(&style).unwrap().unwrap();
    let child = done.created(&child).unwrap().unwrap();
    let seed_text = done.created(&seed_text).unwrap().unwrap();
    let before = dom.snapshot();

    // Changing type changes the canonical bank but does not itself trigger
    // Style processing. The subsequent actual Text edit owns that update.
    complete_local(
        &mut dom,
        DomSourceEdit::set_attribute(style, "type", raw("text/css")),
    );
    let after_attribute = dom.snapshot();
    assert_eq!(bank(&after_attribute, style).len(), 2);
    assert_eq!(
        bank(&after_attribute, style)[0].name().local_name(),
        &raw("type")
    );
    assert_eq!(bank(&after_attribute, style)[0].value(), &raw("text/css"));
    assert_eq!(bank(&after_attribute, style)[1].value(), &raw("kept"));
    assert_eq!(bank(&before, style)[0].value(), &raw("text/plain"));
    assert!(std::ptr::eq(
        bank(&after_attribute, style),
        bank(&after_attribute, seed)
    ));
    assert!(
        after_attribute
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert!(
        after_attribute
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .parser_document()
            .is_none()
    );

    // The real ChildrenChanged Style stop restores the command, executes
    // headroom under Restore, then replays before exposing the policy request.
    blocked_style_source(
        &mut dom,
        DomSourceEdit::replace_data(child, 0, usize::MAX, raw("b{}")),
        style,
    );
    let after_text = dom.snapshot();
    assert_eq!(text(&after_text, child).code_units(), [98, 123, 125]);
    assert_eq!(text(&before, child).code_units(), [97, 123, 125]);
    assert_eq!(text(&after_attribute, child).code_units(), [97, 123, 125]);
    assert!(Arc::ptr_eq(
        text(&after_text, child).shared_code_units(),
        text(&after_text, seed_text).shared_code_units()
    ));
    assert!(std::ptr::eq(
        bank(&after_attribute, style),
        bank(&after_text, style)
    ));
    assert!(
        after_text
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert!(dom.pending_source_edit().is_none());
    assert!(!dom.has_style_cssom_roots());
}

#[test]
fn prepared_close_publication_drop_and_unwind_preserve_original_tail_and_shared_bank() {
    for unwind in [false, true] {
        let (mut dom, document) = owner();
        let executor = dom.bind_source_event_executor(&cancel()).unwrap();
        let task_source = dom.bind_element_task_source(&cancel()).unwrap();
        let task_retention = task_source
            .retain_payload(&mut dom, size_of::<[Option<DomDialogTask>; 2]>(), &cancel())
            .unwrap();
        let mut tasks: [Option<DomDialogTask>; 2] = [None, None];
        let mut setup = DomBatch::new();
        let seed = setup.create_element_ns(document, Namespace::html(), "div");
        setup.set_attribute(&seed, "data-shared", raw("kept"));
        let dialog = setup.create_element_ns(document, Namespace::html(), "dialog");
        setup.set_attribute(&dialog, "data-shared", raw("kept"));
        setup.set_attribute(&dialog, "open", "");
        setup.insert_before(&dialog, document, None);
        let done = dom.apply(setup, &cancel()).unwrap();
        let seed = done.created(&seed).unwrap().unwrap();
        let dialog = done.created(&dialog).unwrap().unwrap();
        let before = dom.snapshot();
        let event = {
            let mut lease = dom
                .begin_source_edit(
                    DomSourceEdit::close_dialog(dialog, None),
                    source_limits(),
                    &cancel(),
                )
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
                lease.advance(&cancel()).unwrap()
            else {
                panic!("actual beforetoggle")
            };
            assert!(!event.cancelable());
            event
        };
        let id = event.operation();
        let permit = executor.begin_dispatch(&dom, &event, &cancel()).unwrap();
        let scope = executor.enter_dispatch(&dom, &permit, &cancel()).unwrap();
        drop(scope);
        let mut permit = Some(permit);
        let mut event_receipt = Some(
            executor
                .seal_completed(
                    &dom,
                    &mut permit,
                    DomSourceEventOutcome::NotCanceled,
                    &cancel(),
                )
                .unwrap(),
        );
        assert!(permit.is_none());
        {
            let mut lease = dom
                .resume_source_edit(id, source_limits(), &cancel())
                .unwrap();
            lease
                .accept_source_event(&event, &mut event_receipt, &cancel())
                .unwrap();
            assert!(event_receipt.is_none());
            let DomSourceProgress::Request(DomSourceRequest::Dialog(toggle)) =
                lease.advance(&cancel()).unwrap()
            else {
                panic!("actual QueueToggle")
            };
            assert!(matches!(
                toggle.action(),
                HtmlDialogAction::QueueToggle { .. }
            ));
            let plan = lease
                .dialog_task_queue_plan(&toggle, &task_source, 1, &cancel())
                .unwrap();
            let prepared = lease
                .prepare_dialog_task_queue(&toggle, &task_source, plan, &cancel())
                .unwrap();
            let (task, publication) = prepared.publish();
            tasks[0] = Some(task);
            let mut publication = Some(publication);
            lease
                .accept_dialog_task_queue(&toggle, &mut publication, &cancel())
                .unwrap();
            assert!(publication.is_none());
            let DomSourceProgress::Request(DomSourceRequest::Dialog(close)) =
                lease.advance(&cancel()).unwrap()
            else {
                panic!("actual QueueClose after RemoveOpen")
            };
            assert_eq!(close.operation(), id);
            assert_eq!(close.action(), HtmlDialogAction::QueueClose);
            let stamp = lease.snapshot().stamp();
            let phases = lease.status().published_phases();
            let spent = lease.status().spent_work();
            let plan = lease
                .dialog_task_queue_plan(&close, &task_source, 2, &cancel())
                .unwrap();
            assert_eq!(plan.replacing(), None);
            if unwind {
                let entered = Cell::new(false);
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let _prepared = lease
                        .prepare_dialog_task_queue(&close, &task_source, plan, &cancel())
                        .unwrap();
                    entered.set(true);
                    panic!("caller drops prepared QueueClose");
                }));
                assert!(
                    entered.get(),
                    "the real prepared guard must exist before unwinding"
                );
                let error = result.expect_err("intentional caller unwind");
                assert_eq!(
                    error.downcast_ref::<&str>(),
                    Some(&"caller drops prepared QueueClose")
                );
            } else {
                drop(
                    lease
                        .prepare_dialog_task_queue(&close, &task_source, plan, &cancel())
                        .unwrap(),
                );
            }
            assert_eq!(lease.operation(), id);
            assert_eq!(lease.snapshot().stamp(), stamp);
            assert_eq!(lease.status().published_phases(), phases);
            assert!(lease.status().spent_work() > spent);
            assert_eq!(
                lease
                    .snapshot()
                    .node(dialog)
                    .unwrap()
                    .attribute(&raw("open")),
                Some(&raw(""))
            );
            assert!(tasks[1].is_none());
            let prepared = lease
                .prepare_dialog_task_queue(&close, &task_source, plan, &cancel())
                .unwrap();
            let (task, publication) = prepared.publish();
            tasks[1] = Some(task);
            let mut publication = Some(publication);
            lease
                .accept_dialog_task_queue(&close, &mut publication, &cancel())
                .unwrap();
            assert!(publication.is_none());
            let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
                panic!("original Close completes")
            };
            assert_eq!(done.operation(), id);
        }
        let after = dom.snapshot();
        assert!(dom.pending_source_edit().is_none());
        assert_eq!(after.node(dialog).unwrap().attribute(&raw("open")), None);
        assert_eq!(
            before.node(dialog).unwrap().attribute(&raw("open")),
            Some(&raw(""))
        );
        assert_eq!(bank(&after, dialog).len(), 1);
        assert_eq!(bank(&after, dialog)[0].value(), &raw("kept"));
        assert!(std::ptr::eq(bank(&after, dialog), bank(&after, seed)));
        assert_eq!(tasks[0].as_ref().unwrap().info().id().target(), dialog);
        assert_eq!(tasks[1].as_ref().unwrap().info().id().target(), dialog);
        drop(tasks);
        drop(task_retention);
    }
}

#[test]
fn reentrant_child_values_survive_parent_observation_discard_and_canceled_resume() {
    let (mut dom, document) = owner();
    let executor = dom.bind_source_event_executor(&cancel()).unwrap();
    let mut setup = DomBatch::new();
    let dialog = setup.create_element_ns(document, Namespace::html(), "dialog");
    setup.insert_before(&dialog, document, None);
    let seed = setup.create_element_ns(document, Namespace::html(), "div");
    setup.set_attribute(&seed, "data-shared", raw("child"));
    let target = setup.create_element_ns(document, Namespace::html(), "div");
    let target_text = setup.create_text(document, raw("before"));
    let seed_text = setup.create_text(document, raw("child-text"));
    let done = dom.apply(setup, &cancel()).unwrap();
    let dialog = done.created(&dialog).unwrap().unwrap();
    let seed = done.created(&seed).unwrap().unwrap();
    let target = done.created(&target).unwrap().unwrap();
    let target_text = done.created(&target_text).unwrap().unwrap();
    let seed_text = done.created(&seed_text).unwrap().unwrap();
    let before = dom.snapshot();
    let event = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::show_dialog(dialog),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("real parent observation")
        };
        assert!(event.cancelable());
        event
    };
    let parent = event.operation();
    let work_before = dom.pending_source_edit().unwrap().spent_work();
    let permit = executor.begin_dispatch(&dom, &event, &cancel()).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel()).unwrap();
    complete_local(
        &mut dom,
        DomSourceEdit::set_attribute(target, "data-shared", raw("child")),
    );
    complete_local(
        &mut dom,
        DomSourceEdit::replace_data(target_text, 0, usize::MAX, raw("child-text")),
    );
    let child = dom.snapshot();
    assert_eq!(dom.pending_source_edit().unwrap().operation(), parent);
    assert!(dom.pending_source_edit().unwrap().spent_work() > work_before);
    assert!(std::ptr::eq(bank(&child, target), bank(&child, seed)));
    assert!(Arc::ptr_eq(
        text(&child, target_text).shared_code_units(),
        text(&child, seed_text).shared_code_units()
    ));
    drop(scope);
    assert_eq!(dom.source_dispatch_blocker(), Some(parent));
    let mut permit = Some(permit);
    let mut receipt = Some(
        executor
            .seal_completed(
                &dom,
                &mut permit,
                DomSourceEventOutcome::Canceled,
                &cancel(),
            )
            .unwrap(),
    );
    {
        let mut lease = dom
            .resume_source_edit(parent, source_limits(), &cancel())
            .unwrap();
        lease
            .accept_source_event(&event, &mut receipt, &cancel())
            .unwrap();
        assert!(receipt.is_none());
        let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
            panic!("canceled parent returns without a queue effect")
        };
        assert_eq!(done.operation(), parent);
    }
    let mut later = DomBatch::new();
    let later_element = later.create_element_ns(document, Namespace::html(), "div");
    later.set_attribute(&later_element, "data-shared", raw("child"));
    let later_text = later.create_text(document, raw("child-text"));
    let done = dom.apply(later, &cancel()).unwrap();
    let later_element = done.created(&later_element).unwrap().unwrap();
    let later_text = done.created(&later_text).unwrap().unwrap();
    let after = dom.snapshot();
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(after.node(dialog).unwrap().attribute(&raw("open")), None);
    assert_eq!(text(&before, target_text), &raw("before"));
    assert!(bank(&before, target).is_empty());
    assert_eq!(bank(&after, target)[0].value(), &raw("child"));
    assert_eq!(text(&after, target_text), &raw("child-text"));
    assert!(std::ptr::eq(bank(&child, target), bank(&after, target)));
    assert!(std::ptr::eq(
        bank(&after, target),
        bank(&after, later_element)
    ));
    assert!(Arc::ptr_eq(
        text(&child, target_text).shared_code_units(),
        text(&after, later_text).shared_code_units()
    ));
}

#[test]
fn refused_source_candidates_cancel_cleanly_and_semantic_noops_preserve_backing() {
    for attribute in [false, true] {
        let (mut dom, document) = owner();
        let mut setup = DomBatch::new();
        let element = setup.create_element_ns(document, Namespace::html(), "div");
        setup.set_attribute(&element, "data-shared", raw("old"));
        let node = setup.create_text(document, raw("old"));
        let done = dom.apply(setup, &cancel()).unwrap();
        let element = done.created(&element).unwrap().unwrap();
        let node = done.created(&node).unwrap().unwrap();
        let before = dom.snapshot();
        let value = raw("discarded-candidate");
        let weak = Arc::downgrade(value.shared_code_units());
        let edit = if attribute {
            DomSourceEdit::set_attribute(element, "data-shared", value)
        } else {
            DomSourceEdit::replace_data(node, 0, usize::MAX, value)
        };
        let bounded = DomSourceLimits::new(128_000_000, 1_000_000, 20_000_000, 0, 4096, 4096, 4);
        {
            let mut lease = dom.begin_source_edit(edit, bounded, &cancel()).unwrap();
            assert_eq!(
                lease.advance(&cancel()).unwrap_err(),
                DomError::SourcePublicationLimit
            );
            assert!(lease.status().spent_work() > 0);
            assert!(!lease.status().has_published());
            assert_eq!(lease.snapshot().stamp(), before.stamp());
            let canceled = AtomicBool::new(true);
            assert_eq!(lease.advance(&canceled).unwrap_err(), DomError::Cancelled);
            drop(lease); // cleanup cannot ask the already canceled caller for work
        }
        assert!(dom.pending_source_edit().is_none());
        assert!(
            weak.upgrade().is_none(),
            "optional weak membership must not retain the discarded strong input"
        );
        let after_refusal = dom.snapshot();
        assert_eq!(after_refusal.stamp(), before.stamp());
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
        assert!(std::ptr::eq(
            bank(&before, element),
            bank(&after_refusal, element)
        ));
        assert!(Arc::ptr_eq(
            text(&before, node).shared_code_units(),
            text(&after_refusal, node).shared_code_units()
        ));
        let edit = if attribute {
            DomSourceEdit::set_attribute(element, "data-shared", raw("kept"))
        } else {
            DomSourceEdit::replace_data(node, 0, usize::MAX, raw("kept"))
        };
        complete_local(&mut dom, edit);
        let published = dom.snapshot();
        if attribute {
            assert_eq!(bank(&published, element)[0].value(), &raw("kept"));
        } else {
            assert_eq!(text(&published, node), &raw("kept"));
        }
        let mut spare = Vec::with_capacity(64);
        spare.extend("kept".encode_utf16());
        let equal_with_spare = DocumentString::from_utf16(spare);
        let current = if attribute {
            bank(&published, element)[0].value()
        } else {
            text(&published, node)
        };
        assert!(
            equal_with_spare.shared_code_units().capacity()
                > current.shared_code_units().capacity()
        );
        let unchanged = if attribute {
            DomSourceEdit::set_attribute(element, "data-shared", equal_with_spare)
        } else {
            DomSourceEdit::replace_data(node, 0, usize::MAX, equal_with_spare)
        };
        complete_local(&mut dom, unchanged);
        let after_noop = dom.snapshot();
        assert_eq!(after_noop.stamp(), published.stamp());
        assert!(
            dom.changes_since(published.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
        assert!(std::ptr::eq(
            bank(&published, element),
            bank(&after_noop, element)
        ));
        assert!(Arc::ptr_eq(
            text(&published, node).shared_code_units(),
            text(&after_noop, node).shared_code_units()
        ));
    }
}
