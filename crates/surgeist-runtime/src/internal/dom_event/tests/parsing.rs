//! Literal event contracts from the pinned HTML stop-parsing and DOM dispatch
//! algorithms. The newly callable native firing path has no preexisting RED.

use super::*;

#[test]
fn trusted_window_load_uses_the_document_target_and_resumes_its_window_path() {
    let cancel = AtomicBool::new(false);
    let mut session = event_session(dom());
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel).unwrap();
    let top = hosted.create_top_level(&mut dom, host, &cancel).unwrap();
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let other = dom.apply(batch, &cancel).unwrap().document(&other).unwrap();
    let document = DomEventTarget::from(top.document().node());
    let window = DomEventTarget::from(top.global());
    let mut registry = registry();
    let kind: Vec<u16> = "load".encode_utf16().collect();
    let observer = DomEventCallback::new({
        let window = window.clone();
        let document = document.clone();
        move |_, _, observed: &mut Vec<NativeDomEvent>, event| {
            assert!(event.is_trusted());
            assert!(!event.bubbles());
            assert!(!event.cancelable());
            assert!(!event.composed());
            assert_eq!(event.target(), Some(document.clone()));
            assert_eq!(event.current_target(), Some(window.clone()));
            assert_eq!(event.phase(), DomEventPhase::AtTarget);
            assert_eq!(
                event.composed_path().collect::<Vec<_>>().as_slice(),
                std::slice::from_ref(&window)
            );
            event.prevent_default();
            assert!(!event.default_prevented());
            observed.push(event.clone());
            Ok(())
        }
    });
    let once = registry
        .add_listener(
            &mut dom,
            window.clone(),
            &kind,
            observer.clone(),
            DomEventListenerOptions {
                capture: true,
                once: true,
                ..Default::default()
            },
        )
        .unwrap();
    registry
        .add_listener(
            &mut dom,
            window.clone(),
            &kind,
            observer,
            Default::default(),
        )
        .unwrap();
    registry
        .add_listener(
            &mut dom,
            document.clone(),
            &kind,
            DomEventCallback::new(|_, _, _, _| {
                panic!("Document is outside the load invocation path")
            }),
            DomEventListenerOptions {
                capture: true,
                ..Default::default()
            },
        )
        .unwrap();

    let mut creation = Some(Err(DomEventTimestampUnavailable::NoAuthenticatedClock));
    assert!(matches!(
        registry.prepare_parsing_event(
            &mut dom,
            ParsingEvent::WindowLoad,
            (top.global(), other),
            &mut creation,
            &cancel,
        ),
        Err(DomEventDispatchError::Dom(DomError::WrongDocument))
    ));
    let mut firing = registry
        .prepare_parsing_event(
            &mut dom,
            ParsingEvent::WindowLoad,
            (top.global(), top.document()),
            &mut creation,
            &cancel,
        )
        .unwrap();
    let mut observed = Vec::new();
    assert!(matches!(
        firing.advance(
            &mut dom,
            &mut registry,
            &mut observed,
            DomEventDispatchLimits {
                path_items: 0,
                ..limits()
            },
            &cancel,
        ),
        Err(DomEventDispatchError::Admission(
            DomEventResource::PathItems
        ))
    ));
    assert!(observed.is_empty());
    assert!(matches!(
        firing
            .advance(
                &mut dom,
                &mut registry,
                &mut observed,
                DomEventDispatchLimits {
                    callbacks: 1,
                    ..limits()
                },
                &cancel,
            )
            .unwrap(),
        DomEventDispatchOutcome::Retained(DomEventResource::Callbacks)
    ));
    assert_eq!(observed.len(), 1);
    assert!(!registry.remove_listener(once));
    assert_eq!(observed[0].current_target(), None);

    let receipt = complete(
        firing
            .advance(&mut dom, &mut registry, &mut observed, limits(), &cancel)
            .unwrap(),
    );
    assert_eq!(receipt.original_target(), window);
    assert_eq!(receipt.event_type().code_units(), kind);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    let [first, second] = observed.as_slice() else {
        panic!("each original Window listener ran exactly once");
    };
    assert!(Rc::ptr_eq(&first.0, &second.0));
    assert_eq!(second.target(), Some(document));
    assert_eq!(second.current_target(), None);
    assert_eq!(second.phase(), DomEventPhase::None);
    assert_eq!(second.composed_path().count(), 0);
    assert!(matches!(
        firing.advance(&mut dom, &mut registry, &mut observed, limits(), &cancel),
        Err(DomEventDispatchError::AlreadyCompleted)
    ));
}

#[test]
fn trusted_document_readiness_events_follow_their_pinned_bubbling_rules() {
    let cancel = AtomicBool::new(false);
    let mut session = event_session(dom());
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel).unwrap();
    let top = hosted.create_top_level(&mut dom, host, &cancel).unwrap();
    let document = DomEventTarget::from(top.document().node());
    let window = DomEventTarget::from(top.global());
    let mut registry = registry();

    for (event, name, bubbles) in [
        (ParsingEvent::ReadyStateChange, "readystatechange", false),
        (ParsingEvent::DomContentLoaded, "DOMContentLoaded", true),
    ] {
        let kind: Vec<u16> = name.encode_utf16().collect();
        let observer = DomEventCallback::new({
            let document = document.clone();
            let window = window.clone();
            let kind = kind.clone();
            move |_, _, observed: &mut Vec<_>, event| {
                assert_eq!(event.event_type().code_units(), kind);
                assert_eq!(event.target(), Some(document.clone()));
                assert_eq!(
                    event.composed_path().collect::<Vec<_>>(),
                    [document.clone(), window.clone()]
                );
                assert!(event.is_trusted());
                assert_eq!(event.bubbles(), bubbles);
                assert!(!event.cancelable());
                assert!(!event.composed());
                observed.push((event.current_target(), event.phase()));
                Ok(())
            }
        });
        for (target, capture) in [
            (window.clone(), true),
            (document.clone(), false),
            (window.clone(), false),
        ] {
            registry
                .add_listener(
                    &mut dom,
                    target,
                    &kind,
                    observer.clone(),
                    DomEventListenerOptions {
                        capture,
                        ..Default::default()
                    },
                )
                .unwrap();
        }
        let mut firing = registry
            .prepare_parsing_event(
                &mut dom,
                event,
                (top.global(), top.document()),
                &mut Some(Err(DomEventTimestampUnavailable::NoAuthenticatedClock)),
                &cancel,
            )
            .unwrap();
        let mut observed = Vec::new();
        let receipt = complete(
            firing
                .advance(&mut dom, &mut registry, &mut observed, limits(), &cancel)
                .unwrap(),
        );
        assert!(receipt.failures().is_empty());
        assert_eq!(receipt.original_target(), document);
        let mut expected = vec![
            (Some(window.clone()), DomEventPhase::Capturing),
            (Some(document.clone()), DomEventPhase::AtTarget),
        ];
        if bubbles {
            expected.push((Some(window.clone()), DomEventPhase::Bubbling));
        }
        assert_eq!(observed, expected);
    }
}
