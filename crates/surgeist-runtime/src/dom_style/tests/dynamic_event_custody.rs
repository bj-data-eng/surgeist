//! A completed Style notification leaves its actual escaped path funded.
use super::*;
use crate::dom_event::manipulation_observation_layout;

struct Observations {
    styles: DomStyleRuntime,
    event: Option<NativeDomEvent>,
    paths: Option<[crate::DomEventComposedPath; 2]>,
}

impl DomStyleEventContext for Observations {
    fn styles(&mut self) -> &mut DomStyleRuntime {
        &mut self.styles
    }
}

#[test]
fn escaped_style_path_retains_exact_storage_after_task_and_event_disposal() {
    let cancel = AtomicBool::new(false);
    let (mut session, document) = bound_dispatch_session();
    let target = append_style(&mut session, document, None);
    let parent = session.snapshot().node(target).unwrap().parent().unwrap();
    let mut context = Observations {
        styles: session.style_execution(),
        event: None,
        paths: None,
    };
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, context: &mut Observations, event| {
                context.event = Some(event.clone());
                context.paths = Some([event.composed_path(), event.composed_path()]);
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let DomStyleNetworkingProgress::Complete { receipt, .. } = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        dispatch_limits(),
        &cancel,
    )
    .unwrap() else {
        panic!("actual Style load completes")
    };
    assert_eq!(receipt.callbacks(), 1);
    assert_eq!(receipt.original_target(), target);
    assert_eq!(receipt.event_type(), DomEventType::Load);
    assert!(context.styles.pending_sheet(0).unwrap().is_none());
    assert_eq!(
        context
            .styles
            .snapshot(document)
            .unwrap()
            .unwrap()
            .sheets()
            .len(),
        1
    );
    drop(receipt);
    let event = context.event.take().unwrap();
    let [path, second] = context.paths.take().unwrap();
    let (_, bytes) = manipulation_observation_layout(&event, &path);
    assert!(bytes > 0);
    drop(event);
    drop(dispatcher);
    let state = context.styles.original().unwrap();
    let policy = state.borrow().limits;
    let held = state.borrow().storage(None, &cancel, policy.work).unwrap();
    drop(second);
    assert_eq!(
        state.borrow().storage(None, &cancel, policy.work).unwrap(),
        held
    );
    assert!(matches!(
        context.styles.set_limits(
            &session.dom(),
            DomStyleRuntimeLimits {
                storage_bytes: held - 1,
                ..policy
            },
            &cancel,
        ),
        Err(DomStyleRuntimeError::Admission),
    ));
    context
        .styles
        .set_limits(
            &session.dom(),
            DomStyleRuntimeLimits {
                storage_bytes: held,
                ..policy
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(
        path.collect::<Vec<_>>(),
        [target.into(), parent.into(), document.into()]
    );
    let remaining = state.borrow().storage(None, &cancel, policy.work).unwrap();
    assert_eq!(
        held - remaining,
        bytes,
        "the last iterator releases its actual path allocation"
    );
}
