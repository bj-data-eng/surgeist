//! Actual Source entry constructors and affine pre-advance binding.
use super::*;
use crate::*;
fn limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 2_000_000, 10_000_000, 128, 8192, 8192, 4)
}
fn fixture() -> (Dom, DomHostedSource, DomHostedHostId, DocumentId, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut dom =
        Dom::new(DomLimits::new(256, 1_000_000, 1024, 1024, 1024, 32, 4_000_000).unwrap()).unwrap();
    let source = dom
        .register_hosted_source(
            DomHostedLimits::new(
                DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
                16_000_000,
                16_000_000,
                4_000_000,
            ),
            &cancel,
        )
        .unwrap();
    let mut source = source;
    let host = source.issue_host(&cancel).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&root, &document, None);
    let done = dom.apply(batch, &cancel).unwrap();
    let document = done.document(&document).unwrap();
    let root = done.created(&root).unwrap().unwrap();
    (dom, source, host, document, root)
}
#[test]
fn original_edit_binding_is_once_only_and_first_advance_seals_the_native_entry() {
    let (mut dom, source, host, _, root) = fixture();
    let cancel = AtomicBool::new(false);
    let before = dom.snapshot();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_attribute(root, "data-entry", "value"),
            limits(),
            &cancel,
        )
        .unwrap();
    assert_eq!(
        lease.execution_entry_scope().unwrap(),
        DomSourceExecutionEntryScope::NativeEntry
    );
    cancel.store(true, Ordering::Relaxed);
    assert!(matches!(
        lease.bind_execution_host(&source, host, &cancel),
        Err(DomError::Cancelled)
    ));
    cancel.store(false, Ordering::Relaxed);
    assert_eq!(
        lease.execution_entry_scope().unwrap(),
        DomSourceExecutionEntryScope::NativeEntry
    );
    lease.bind_execution_host(&source, host, &cancel).unwrap();
    assert_eq!(
        lease.execution_entry_scope().unwrap(),
        DomSourceExecutionEntryScope::ExplicitHost(host)
    );
    let work = lease.status().spent_work();
    assert!(matches!(
        lease.bind_execution_host(&source, host, &cancel),
        Err(DomError::InvalidSourceOperation)
    ));
    assert!(lease.status().spent_work() > work);
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    assert!(matches!(
        lease.bind_execution_host(&source, host, &cancel),
        Err(DomError::InvalidSourceOperation)
    ));
    assert!(matches!(
        lease.execution_entry_scope(),
        Err(DomError::InvalidSourceOperation)
    ));
    assert_eq!(
        before.node(root).unwrap().attribute(&"data-entry".into()),
        None
    );
    assert_eq!(
        lease
            .snapshot()
            .node(root)
            .unwrap()
            .attribute(&"data-entry".into()),
        Some(&"value".into())
    );
}
#[test]
fn installation_binds_its_actual_target_parent_before_creations_or_publication() {
    let (mut dom, source, host, document, parent) = fixture();
    let cancel = AtomicBool::new(false);
    let mut builder = surgeist_html::HtmlFragment::builder(
        surgeist_html::HtmlConstructionLimits::new(8, 8, 8, 1024, 8),
    )
    .unwrap();
    builder
        .append_element(
            &builder.root(),
            QualifiedName::new(Namespace::html(), None, "span"),
            OrderedAttributes::new([]).unwrap(),
        )
        .unwrap();
    let fragment = builder.finish();
    let child = fragment.root().children().next().unwrap();
    let before = dom.snapshot();
    let target =
        DomNativeHtmlInstallTarget::new(before.document(document).unwrap().stamp(), parent, None);
    let mut lease = dom
        .begin_native_installation(
            target,
            &fragment,
            DomSourceNativeHtmlInstallLimits::new(
                DomNativeHtmlInstallLimits::new(16, 1_000_000, 16_000_000),
                limits(),
            ),
            &cancel,
        )
        .unwrap();
    assert_eq!(
        lease.execution_entry_scope().unwrap(),
        DomSourceExecutionEntryScope::NativeEntry
    );
    lease.bind_execution_host(&source, host, &cancel).unwrap();
    assert!(lease.snapshot().node(parent).unwrap().children().is_empty());
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("actual plain installation must complete")
    };
    let installed = done
        .native_installation()
        .unwrap()
        .node_for(&child)
        .unwrap();
    assert_eq!(
        lease.snapshot().node(installed).unwrap().parent(),
        Some(parent)
    );
    assert_eq!(before.node(parent).unwrap().children(), &[]);
}
#[test]
fn actual_select_completion_captures_its_select_and_reuses_the_original_authorization() {
    let (mut dom, source, host, document, root) = fixture();
    dom.set_limits(
        dom.limits
            .with_select_processing(16, 4_000_000, 10_000_000)
            .unwrap(),
    )
    .unwrap();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let select = batch.create_element_ns(document, Namespace::html(), "select");
    let option = batch.create_element_ns(document, Namespace::html(), "option");
    let other = batch.create_element_ns(document, Namespace::html(), "option");
    let content = batch.create_element_ns(document, Namespace::html(), "selectedcontent");
    batch.set_attribute(&option, "selected", "");
    batch.insert_before(&option, &select, None);
    batch.insert_before(&other, &select, None);
    batch.insert_before(&content, &select, None);
    batch.insert_before(&select, root, None);
    let created = dom.apply(batch, &cancel).unwrap();
    let select = created.created(&select).unwrap().unwrap();
    let option = created.created(&option).unwrap().unwrap();
    let mut change = DomBatch::new();
    change.detach(option);
    dom.apply(change, &cancel).unwrap();
    let checkpoint = DomNativeCheckpointLimits::new(
        4_000_000,
        10_000_000,
        DomSlotQueryLimits::new(512, 2048).unwrap(),
    )
    .unwrap();
    let DomNativeCheckpoint::Select(task) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("actual queued Select task required")
    };
    assert_eq!(task.select(), select);
    {
        let mut lease = dom
            .begin_select_completion(&task, checkpoint, limits(), &cancel)
            .unwrap();
        assert_eq!(
            lease.execution_entry_scope().unwrap(),
            DomSourceExecutionEntryScope::NativeEntry
        );
        lease.bind_execution_host(&source, host, &cancel).unwrap();
        assert_eq!(
            lease.execution_entry_scope().unwrap(),
            DomSourceExecutionEntryScope::ExplicitHost(host)
        );
    }
    // Dropping this unadvanced entry parks no fabricated completion and consumes
    // neither the original task nor its actual authorization.
    assert_eq!(dom.pending_native_tasks(), 1);
    let DomNativeCheckpoint::Select(same) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("original task remains")
    };
    assert_eq!(same.operation(), task.operation());
    assert_eq!(same.select(), select);
}
