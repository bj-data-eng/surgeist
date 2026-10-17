//! Started native Details dispatch retains historical targets after retirement.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]

use std::sync::atomic::AtomicBool;
use surgeist_css::CssUtf16InputLimits;
use surgeist_dom::*;
use surgeist_html::Namespace;
use surgeist_runtime::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

fn session() -> DomSession {
    let dom = Dom::new(
        DomLimits::new(128, 200_000, 256, 256, 256, 32, 1_000_000)
            .unwrap()
            .with_details_toggle_intentions(16)
            .unwrap(),
    )
    .unwrap();
    DomSession::new(
        dom,
        DomSessionLimits {
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 16,
                reports: 16,
                storage_bytes: 16_000_000,
                work: 1_000_000,
                css_input: CssUtf16InputLimits::default(),
            },
            details: DomDetailsRuntimeLimits {
                tasks: 16,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
                    16_000_000,
                    16_000_000,
                    1_000_000,
                ),
                tasks: 16,
                storage_bytes: 16_000_000,
                scratch_bytes: 16_000_000,
                work: 1_000_000,
            },
        },
    )
    .unwrap()
}

fn dispatch_limits() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 64,
        snapshot_listeners: 16,
        failure_records: 16,
        callbacks: 16,
        work: 100_000,
    }
}

struct Context {
    details: DomDetailsRuntime,
    target: NodeId,
    observations: Vec<(Option<DomEventTarget>, Option<NodeId>, DomEventPhase)>,
    edits: Vec<Result<DomCommit, DomError>>,
}

impl DomDetailsEventContext for Context {
    fn details(&mut self) -> &mut DomDetailsRuntime {
        &mut self.details
    }
}

#[test]
fn started_hosted_details_dispatch_delivers_later_listener_after_target_retirement() {
    let mut session = session();
    let mut hosted = session.hosted_execution();
    let top = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel()).unwrap();
        hosted.create_top_level(&mut dom, host, &cancel()).unwrap()
    };
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut details = session.details_execution();
    let target = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let target = batch.create_element_ns(top.document(), Namespace::html(), "details");
        batch.insert_before(&target, body, None);
        batch.set_attribute(&target, "open", "");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let target = commit.created(&target).unwrap().unwrap();
        assert_eq!(details.ingest(&mut dom, &cancel()).unwrap(), 1);
        target
    };
    let queued = details.pending_task(0).unwrap().unwrap();
    let association = queued.queue_association().unwrap();
    assert_eq!(association.document(), top.document());
    assert_eq!(association.global(), top.global());
    let mut context = Context {
        details,
        target,
        observations: Vec::new(),
        edits: Vec::new(),
    };
    let mut registry = DomEventDispatcher::<Context>::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap();
    {
        let dom = session.dom();
        registry
            .add_listener(
                &dom,
                target,
                DomEventType::Toggle,
                DomEventCallback::new(|dom, _, context: &mut Context, event| {
                    context.observations.push((
                        event.current_target(),
                        event.target(),
                        event.phase(),
                    ));
                    let mut batch = DomBatch::new();
                    batch.detach(context.target);
                    batch.destroy(context.target);
                    context.edits.push(dom.apply(batch, &cancel()));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        registry
            .add_listener(
                &dom,
                target,
                DomEventType::Toggle,
                DomEventCallback::new(|dom, _, context: &mut Context, event| {
                    context.observations.push((
                        event.current_target(),
                        event.target(),
                        event.phase(),
                    ));
                    let mut batch = DomBatch::new();
                    batch.set_attribute(context.target, "retired-write", "forbidden");
                    context.edits.push(dom.apply(batch, &cancel()));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }

    let progress = DomDetailsRuntime::run_next(
        &mut session.dom(),
        &mut registry,
        &mut context,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap();
    let DomDetailsProgress::Complete {
        task,
        receipt,
        displaced_owner: None,
    } = progress
    else {
        panic!("started dispatch must deliver its retained exact target and finish: {progress:?}")
    };
    assert_eq!(task, queued);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert_eq!(
        context.observations,
        [
            (Some(target.into()), Some(target), DomEventPhase::AtTarget),
            (Some(target.into()), Some(target), DomEventPhase::AtTarget),
        ]
    );
    assert_eq!(context.edits.len(), 2);
    assert!(context.edits[0].is_ok());
    assert!(matches!(context.edits[1], Err(DomError::StaleNode)));
    assert!(matches!(
        session.snapshot().node(target),
        Err(DomError::StaleNode)
    ));
    let settings = hosted
        .settings(&session.dom(), top.global(), &cancel())
        .unwrap();
    assert_eq!(settings.document().id(), top.document());
    assert!(settings.environment_ready());
    assert!(settings.document().is_fully_active().unwrap());
    assert_eq!(context.details.pending_tasks().unwrap(), 0);
    assert!(matches!(
        DomDetailsRuntime::run_next(
            &mut session.dom(),
            &mut registry,
            &mut context,
            dispatch_limits(),
            &cancel(),
        )
        .unwrap(),
        DomDetailsProgress::Empty
    ));
}
