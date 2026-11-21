//! Actual Source entry constructors and affine pre-advance binding.
use super::*;
use crate::internal::batch::DomBatch;
use crate::internal::hosted::{
    DomHostedHostId, DomHostedLimits, DomHostedRecordLimits, DomHostedSource,
};
use crate::internal::identity::{DocumentId, NodeId};
use crate::internal::installation::{DomNativeHtmlInstallLimits, DomNativeHtmlInstallTarget};
use crate::internal::select::{DomNativeCheckpoint, DomNativeCheckpointLimits};
use crate::internal::slot::DomSlotQueryLimits;
use crate::internal::source::{
    DomSourceEdit, DomSourceExecutionEntryScope, DomSourceLimits, DomSourceNativeHtmlInstallLimits,
    DomSourceProgress,
};
use crate::internal::store::{Dom, DomError, DomLimits};
use surgeist_html::values::{Namespace, OrderedAttributes, QualifiedName};
fn limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 2_000_000, 10_000_000, 128, 8192, 8192, 4)
}
fn fixture() -> (Dom, DomHostedSource, DomHostedHostId, DocumentId, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            1_000_000,
            1024,
            1024,
            1024,
            32,
            4_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
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
            DomSourceEdit::set_native_attribute(root, "data-entry", "value"),
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
    let mut builder = surgeist_html::construction::HtmlFragment::builder(
        surgeist_html::construction::HtmlConstructionLimits::new(8, 8, 8, 1024, 8),
    )
    .unwrap();
    builder
        .append_element(
            &builder.root(),
            QualifiedName::new(Namespace::html(), None, "span"),
            OrderedAttributes::new([]).unwrap(),
        )
        .unwrap();
    let fragment = builder.finish().unwrap();
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
        let crate::internal::select::DomSelectCompletionStart::Source(mut lease) = dom
            .begin_select_completion(&task, checkpoint, limits(), &cancel)
            .unwrap()
        else {
            panic!("live Select must enter Source");
        };
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

#[test]
fn collection_names_keep_output_custody_and_reject_a_revoked_executor() {
    let (mut dom, _hosted, _, document, root) = fixture();
    let cancel = AtomicBool::new(false);
    let custody = crate::internal::dispatch::DomDispatchCustodyLimits::new(
        64, 4096, 16_000_000, 16_000_000, 4_000_000,
    )
    .unwrap();
    dom.set_limits(dom.limits().with_dispatch_custody(custody))
        .unwrap();
    let mut edit = DomBatch::new();
    edit.set_attribute(root, "id", "first");
    let dialog = edit.create_element_ns(document, Namespace::html(), "dialog");
    edit.set_attribute(&dialog, "id", "first");
    edit.insert_before(&dialog, root, None);
    let dialog = dom
        .apply(edit, &cancel)
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap();
    let collection = dom
        .get_elements_by_tag_name(document, "div".into(), 64, &cancel)
        .unwrap();
    let executor = dom.bind_source_event_executor(&cancel).unwrap();
    let event = {
        let mut source = dom
            .begin_source_edit(DomSourceEdit::show_dialog(dialog), limits(), &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            source.advance(&cancel).unwrap()
        else {
            panic!("the original dialog effect must reach its event");
        };
        event
    };
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
    let read = dom
        .read_html_collection(
            &collection,
            crate::tree::DomHtmlCollectionLimits::new(100_000, 8, 64),
            || Ok::<_, DomError>(()),
            &cancel,
        )
        .unwrap();

    let resolve_window = || {
        dom.window_named_elements(
            document,
            "first".into(),
            64,
            100_000,
            || Ok::<_, DomError>(()),
            &cancel,
        )
    };
    let window = match resolve_window().unwrap() {
        crate::tree::DomWindowNamedElements::Collection(value) => Some(value),
        _ => None,
    }
    .expect("two actual Window candidate Elements");
    let window = dom
        .read_html_collection(
            &window,
            crate::tree::DomHtmlCollectionLimits::new(100_000, 8, 64),
            || Ok::<_, DomError>(()),
            &cancel,
        )
        .unwrap();
    assert_eq!(window.length().unwrap(), 2);

    let ledger = dom.custody_registry.ledger();
    let baseline = ledger.total();
    let first = read.supported_names().unwrap();
    let first_backing = ledger.total();
    let second = read.supported_names().unwrap();
    let second_backing = ledger.total();
    assert_eq!(first.as_slice(), ["first".into()]);
    assert_eq!(second.as_slice(), first.as_slice());
    assert!(first_backing.bytes() > baseline.bytes());
    assert!(first_backing.units() > baseline.units());
    assert_eq!(
        second_backing.bytes() - first_backing.bytes(),
        first_backing.bytes() - baseline.bytes()
    );
    assert_eq!(
        second_backing.units() - first_backing.units(),
        first_backing.units() - baseline.units()
    );
    drop(first);
    assert_eq!(ledger.total(), first_backing);
    drop(second);
    assert_eq!(ledger.total(), baseline);

    drop(executor);
    assert_eq!(read.item(0), Err(DomError::InvalidSourceEvent));
    assert_eq!(window.item(0), Err(DomError::InvalidSourceEvent));
    assert!(matches!(
        resolve_window(),
        Err(DomError::InvalidSourceEvent)
    ));
    drop(scope);
}

#[test]
fn document_tree_child_read_borrows_only_the_exact_live_dispatch_scope() {
    use crate::internal::hosted::DomHostedError;

    let (mut dom, _hosted, _, document, root) = fixture();
    let cancel = AtomicBool::new(false);
    let custody = crate::internal::dispatch::DomDispatchCustodyLimits::new(
        64, 4096, 16_000_000, 16_000_000, 4_000_000,
    )
    .unwrap();
    dom.set_limits(dom.limits().with_dispatch_custody(custody))
        .unwrap();
    let mut edit = DomBatch::new();
    let dialog = edit.create_element_ns(document, Namespace::html(), "dialog");
    edit.insert_before(&dialog, root, None);
    let dialog = dom
        .apply(edit, &cancel)
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap();
    let executor = dom.bind_source_event_executor(&cancel).unwrap();
    let event = {
        let mut source = dom
            .begin_source_edit(DomSourceEdit::show_dialog(dialog), limits(), &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
            source.advance(&cancel).unwrap()
        else {
            panic!("the original dialog effect must reach its event");
        };
        event
    };
    let read = |dom: &Dom| {
        dom.visit_document_tree_child_navigables(
            document,
            10_000,
            || Ok::<_, DomHostedError>(()),
            &cancel,
            |_| Ok(()),
        )
    };
    assert_eq!(read(&dom), Err(DomError::IncompleteSourceOperation.into()));
    let permit = executor.begin_dispatch(&dom, &event, &cancel).unwrap();
    let mut scope = Some(executor.enter_dispatch(&dom, &permit, &cancel).unwrap());
    let before = dom.pending_source_edit().unwrap().spent_work();
    let mut admitted = 0;
    assert_eq!(
        dom.visit_document_tree_child_navigables(
            document,
            10_000,
            || {
                admitted += 1;
                Ok::<_, DomHostedError>(())
            },
            &cancel,
            |_| Ok(()),
        ),
        Ok(0)
    );
    let after = dom.pending_source_edit().unwrap().spent_work();
    assert!(admitted > 0);
    // The dispatch qualification consumes one step. Remaining steps each
    // charge the original Source exactly once alongside caller admission.
    assert_eq!(after - before, admitted + 1);
    let mut reached_admission = false;
    let result = dom.visit_document_tree_child_navigables(
        document,
        10_000,
        || {
            reached_admission = true;
            drop(scope.take());
            Ok::<_, DomHostedError>(())
        },
        &cancel,
        |_| Ok(()),
    );
    assert_eq!(result, Err(DomError::InvalidSourceEvent.into()));
    assert!(reached_admission);
    assert_eq!(read(&dom), Err(DomError::IncompleteSourceOperation.into()));
}
