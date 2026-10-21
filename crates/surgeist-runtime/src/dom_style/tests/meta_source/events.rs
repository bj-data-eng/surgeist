//! An actual observed event parent and the native child it authorizes.
use super::*;
use crate::{
    DomEventCallback, DomEventDispatcher, DomEventListenerOptions, DomEventRegistryLimits,
    DomEventType,
};

#[test]
fn observed_beforetoggle_child_meta_completes_and_preserves_the_canceled_parent_tail() {
    let (mut session, document) = bound_dispatch_session();
    let limits = session.dom().limits().with_meta_processing_work(1_000_000);
    session.dom().raw_mut().set_limits(limits).unwrap();
    session
        .source_execution()
        .raise_event_runtime_limits(
            &session.dom(),
            crate::DomSourceEventRuntimeLimits {
                jobs: 4,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            &cancel(),
        )
        .unwrap();
    let source_limits =
        surgeist_dom::DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 4);
    let parent = container(&session, document);
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, parent, None);
    let made = session.dom().apply(batch, &cancel()).unwrap();
    let dialog = made.created(&dialog).unwrap().unwrap();
    let meta = detached_meta(&mut session, document, "B".into());
    let progress = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_dialog(dialog),
            source_limits,
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(request),
        ..
    } = progress
    else {
        panic!("actual beforetoggle");
    };
    struct Context {
        source: crate::DomSourceRuntime,
        meta: NodeId,
        parent: NodeId,
        limits: surgeist_dom::DomSourceLimits,
        operation: surgeist_dom::DomSourceOperationId,
        outcome: Option<DomStyleMetaSourceOutcome>,
    }
    let mut context = Context {
        source: session.source_execution(),
        meta,
        parent,
        limits: source_limits,
        operation: request.operation(),
        outcome: None,
    };
    let mut listeners = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    listeners
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|dom, _, context: &mut Context, event| {
                event.prevent_default();
                let parent_marker = dom.snapshot().incomplete_source_operation();
                assert_eq!(
                    dom.pending_source_edit().unwrap().operation(),
                    context.operation
                );
                let child = context
                    .source
                    .apply(
                        dom,
                        DomSourceEdit::insert_before(context.meta, context.parent, None),
                        context.limits,
                        &mut DomSourceServices {
                            meta: Some(envelope()),
                            execution_scope: None,
                            style_policy: &mut |_| panic!("no child Style service"),
                            url: None,
                        },
                        &cancel(),
                    )
                    .unwrap();
                let DomSourceDriveProgress::MetaSource(outcome) = child else {
                    panic!("real Observed child Meta output");
                };
                assert!(matches!(outcome.source(), DomSourceProgress::Complete(_)));
                assert_eq!(dom.snapshot().incomplete_source_operation(), parent_marker);
                assert_eq!(
                    dom.pending_source_edit().unwrap().operation(),
                    context.operation
                );
                context.outcome = Some(outcome);
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let serviced = session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &request,
            &mut listeners,
            &mut context,
            crate::DomSourceEventServiceLimits {
                source: source_limits,
                dispatch: dispatch_limits(),
            },
            &cancel(),
        )
        .unwrap();
    let crate::DomSourceEventServiceProgress::Serviced { receipt } = serviced else {
        panic!("actual original receipt");
    };
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.default_prevented());
    let outcome = context.outcome.take().unwrap();
    assert_eq!(outcome.preferences().len(), 1);
    let done = session
        .source_execution()
        .resume(
            &mut session.dom(),
            request.operation(),
            source_limits,
            &mut DomSourceServices {
                meta: Some(envelope()),
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(done, DomSourceDriveProgress::Complete(_)));
    assert!(session.dom().pending_source_edit().is_none());
    assert!(
        session
            .snapshot()
            .node(dialog)
            .unwrap()
            .attribute(&"open".into())
            .is_none()
    );
    assert_eq!(
        observe(&mut session, document)
            .preferred_name()
            .code_units(),
        &[66]
    );
}
