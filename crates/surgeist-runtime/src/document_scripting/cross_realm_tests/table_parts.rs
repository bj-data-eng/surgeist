//! The same Table setter resumes across a real descendant Dialog watcher request.
use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
use surgeist_dom::{
    DomError,
    edit::DomChildListObservationLimits,
    source::{DomSourcePhase, DomSourceProgress},
    tree::DomHtmlTablePart::Caption,
};
use surgeist_html::elements::dialog::HtmlDialogAction::DestroyCloseWatcher;

pub(super) fn table_part_setter_resumes_after_modal_descendant_watcher() {
    let cancel = Arc::new(AtomicBool::new(false));
    let observer_limits = DomChildListObservationLimits::new(1, 1, 4, 24).unwrap();
    let limits = scripting_dom_limits(200_000, 0).with_child_list_observation(observer_limits);
    let mut session = scripting_session(Dom::new(limits).unwrap(), 4);
    let mut hosted = session.hosted_execution();
    let owner = hosted
        .create_headless_host(&mut session.dom(), &cancel)
        .unwrap();
    let top = hosted
        .create_top_level(&mut session.dom(), owner, &cancel)
        .unwrap();
    let table = insert_hosted_element(&mut session, top, "table", &cancel);
    let mut setup = DomBatch::new();
    let old = setup.create_element_ns(top.document(), Namespace::html(), "caption");
    let replacement = setup.create_element_ns(top.document(), Namespace::html(), "caption");
    let dialog = setup.create_element_ns(top.document(), Namespace::html(), "dialog");
    setup.insert_before(&old, table, None);
    setup.insert_before(&dialog, &old, None);
    setup.set_attribute(&dialog, "inert", "");
    let commit = session.dom().apply(setup, &cancel).unwrap();
    let [old, replacement, dialog] =
        [old, replacement, dialog].map(|ticket| commit.created(&ticket).unwrap().unwrap());

    let mut host = attach_public_nodes(session, top, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, dialog, "childDialog");
    host.in_document(&permit, |scope| {
        evaluate(scope, "childDialog.showModal()").map(drop)
    })
    .unwrap();

    let shown = host.snapshot().unwrap();
    let before = shown.node(dialog).unwrap().as_dialog().unwrap();
    let watcher = before.close_watcher().unwrap();
    assert!(before.is_modal());
    assert_eq!(watcher.subject(), dialog);
    assert_eq!(
        shown
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        [dialog]
    );

    host.with_native(|dom, _, mut source| {
        let observer = dom.raw_mut().create_child_list_observer(&cancel).unwrap();
        dom.raw_mut()
            .observe_child_list(observer, table, false, &cancel)
            .unwrap();
        let (operation, request) = {
            let mut lease = dom
                .begin_source_edit(
                    DomSourceEdit::set_html_table_part(table, Caption, Some(replacement)),
                    SOURCE,
                    &cancel,
                )
                .unwrap();
            let operation = lease.operation();
            let DomSourceProgress::Request(DomSourceRequest::Dialog(request)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("Caption removal must reach its own descendant watcher");
            };
            assert_eq!(request.operation(), operation);
            assert_eq!(request.subject(), dialog);
            assert_eq!(request.action(), DestroyCloseWatcher);
            (operation, request)
        };
        let prefix = dom.snapshot();
        assert!(prefix.node(table).unwrap().children().is_empty());
        assert_eq!(prefix.node(old).unwrap().children(), [dialog]);
        assert_eq!(prefix.node(replacement).unwrap().parent(), None);
        assert_eq!(
            prefix
                .node(dialog)
                .unwrap()
                .as_dialog()
                .unwrap()
                .close_watcher(),
            Some(watcher)
        );
        let paused = dom.pending_source_edit().unwrap();
        assert_eq!(paused.operation(), operation);
        assert_eq!(paused.phase(), DomSourcePhase::DialogService);
        assert!(paused.has_published());

        assert!(matches!(
            dom.resume_source_edit(operation, SOURCE, &AtomicBool::new(true)),
            Err(DomError::Cancelled)
        ));
        assert_eq!(dom.pending_source_edit().unwrap(), paused);
        assert_eq!(dom.snapshot().stamp(), prefix.stamp());
        {
            let mut lease = dom.resume_source_edit(operation, SOURCE, &cancel).unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Dialog(reissued)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("an unserviced retry must reach the original watcher occurrence");
            };
            assert_eq!(reissued, request);
            assert_eq!(lease.snapshot().stamp(), prefix.stamp());
        }

        let mut services = DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        };
        let progress = source
            .resume(dom, operation, SOURCE, &mut services, &cancel)
            .unwrap();
        let DomSourceDriveProgress::Complete(done) = progress else {
            panic!("the original native watcher service must complete the setter: {progress:?}");
        };
        assert_eq!(done.operation(), operation);
        assert!(dom.pending_source_edit().is_none());
        let records = dom
            .raw_mut()
            .take_child_list_records(observer, &cancel)
            .unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].target(), table);
        assert_eq!(records[0].removed_nodes(), [old]);
        assert!(records[0].added_nodes().is_empty());
        assert_eq!(records[1].target(), table);
        assert_eq!(records[1].added_nodes(), [replacement]);
        assert!(records[1].removed_nodes().is_empty());
    })
    .unwrap();

    let settled = host.snapshot().unwrap();
    assert_eq!(settled.node(table).unwrap().children(), [replacement]);
    assert_eq!(settled.node(old).unwrap().parent(), None);
    let node = settled.node(dialog).unwrap();
    assert_eq!(node.parent(), Some(old));
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    let dialog = node.as_dialog().unwrap();
    assert!(!dialog.is_modal());
    assert!(!dialog.has_close_watcher());
    let layer = settled
        .document(top.document())
        .unwrap()
        .top_layer()
        .unwrap();
    assert!(layer.raw_members().is_empty());
    assert!(layer.pending_removals().is_empty());
}
