//! DOM3071 Document get-the-parent and #1221's native Global projection.
//! These are event-kernel tests, not Style scheduling/completion evidence:
//! the actual attached BODY target avoids introducing a CSS processing fixture.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
#![forbid(unsafe_code)]

use super::*;
use crate::{
    DomElementTaskRuntimeLimits, DomHostedRuntimeLimits, DomSession, DomSessionLimits,
    DomStyleRuntimeLimits,
};
use surgeist_dom::{DomHostedLimits, DomHostedRecordLimits};

#[derive(Debug, Eq, PartialEq)]
struct Recorded {
    current: Option<DomEventTarget>,
    target: Option<NodeId>,
    phase: DomEventPhase,
    capture_listener: bool,
    path: Vec<DomEventTarget>,
}

fn observer(capture_listener: bool) -> DomEventCallback<Vec<Recorded>> {
    DomEventCallback::new(move |_, _, seen: &mut Vec<Recorded>, event| {
        seen.push(Recorded {
            current: event.current_target(),
            target: event.target(),
            phase: event.phase(),
            capture_listener,
            path: event.composed_path().collect(),
        });
        Ok(())
    })
}

fn hosted_session() -> DomSession {
    DomSession::new(
        Dom::new(
            DomLimits::new(
                128,
                100_000,
                256,
                256,
                256,
                32,
                4_000_000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_event_recipients(
                surgeist_dom::DomEventRecipientLimits::new(64, 16_000_000, 16_000_000, 4_000_000)
                    .unwrap(),
            )
            .with_dispatch_custody(
                surgeist_dom::DomDispatchCustodyLimits::new(
                    64, 4096, 16_000_000, 16_000_000, 4_000_000,
                )
                .unwrap(),
            ),
        )
        .unwrap(),
        DomSessionLimits {
            source_events: crate::DomSourceEventRuntimeLimits {
                jobs: 0,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 32,
                reports: 32,
                storage_bytes: 16_000_000,
                work: 1_000_000,
                css_input: surgeist_css::source::CssUtf16InputLimits::default(),
            },
            element_tasks: DomElementTaskRuntimeLimits {
                tasks: 32,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                    16_000_000,
                    16_000_000,
                    1_000_000,
                ),
                tasks: 32,
                storage_bytes: 16_000_000,
                scratch_bytes: 16_000_000,
                work: 1_000_000,
            },
        },
    )
    .unwrap()
}

fn dispatch_hosted(kind: DomEventType) {
    assert!(matches!(kind, DomEventType::Load | DomEventType::Error));
    let cancel = AtomicBool::new(false);
    let mut session = hosted_session();
    let mut hosted = session.hosted_execution();
    let top = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, host, &cancel).unwrap()
    };
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let target = snapshot.node(html).unwrap().children()[1];
    assert_eq!(
        snapshot
            .node(target)
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .local_name(),
        &"body".into()
    );
    assert_eq!(snapshot.node(target).unwrap().parent(), Some(html));
    assert_eq!(
        snapshot.node(html).unwrap().parent(),
        Some(top.document().node())
    );
    assert_eq!(
        snapshot.node(target).unwrap().owner_document(),
        Some(top.document())
    );
    let global = DomEventTarget::Global(top.global());
    let document = DomEventTarget::Node(top.document().node());
    let ancestor = DomEventTarget::Node(html);
    let target_endpoint = DomEventTarget::Node(target);
    let mut listeners = registry::<Vec<Recorded>>();
    for endpoint in [target_endpoint, ancestor, document, global] {
        // Opposite registration order must not turn capture traversal into
        // registration order or give nonbubbling ancestors a bubble visit.
        for capture in [false, true] {
            listeners
                .add_listener(
                    &mut session.dom(),
                    endpoint,
                    kind,
                    observer(capture),
                    DomEventListenerOptions {
                        capture,
                        once: false,
                    },
                )
                .unwrap();
        }
    }
    let event = if kind == DomEventType::Error {
        NativeDomEvent::style_error(target)
    } else {
        NativeDomEvent::style_load(target)
    };
    assert!(!event.bubbles());
    let mut seen = Vec::new();
    let receipt = complete(
        listeners
            .dispatch(
                session.dom().raw_mut(),
                &mut seen,
                &event,
                limits(),
                &AtomicBool::new(false),
            )
            .unwrap(),
    );
    let mut path = vec![target_endpoint, ancestor, document];
    let mut expected = Vec::new();
    if kind == DomEventType::Error {
        path.push(global);
        expected.push((global, DomEventPhase::Capturing, true));
    }
    expected.extend([
        (document, DomEventPhase::Capturing, true),
        (ancestor, DomEventPhase::Capturing, true),
        (target_endpoint, DomEventPhase::AtTarget, true),
        (target_endpoint, DomEventPhase::AtTarget, false),
    ]);
    assert_eq!(
        seen,
        expected
            .into_iter()
            .map(|(current, phase, capture_listener)| Recorded {
                current: Some(current),
                target: Some(target),
                phase,
                capture_listener,
                path: path.clone(),
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        receipt.callbacks(),
        if kind == DomEventType::Error { 5 } else { 4 }
    );
    assert_eq!(receipt.original_target(), target);
    assert_eq!(receipt.event_type(), kind);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    assert!(
        seen.iter()
            .all(|entry| entry.phase != DomEventPhase::Bubbling)
    );
    assert!(
        !seen
            .iter()
            .any(|entry| entry.current == Some(global) && !entry.capture_listener)
    );
    assert_eq!(event.current_target(), None);
    assert_eq!(event.phase(), DomEventPhase::None);
    assert_eq!(event.target(), Some(target));
    assert_eq!(event.composed_path().count(), 0);
}

#[test]
fn hosted_style_error_kernel_captures_global_document_ancestor_then_target_without_bubbling() {
    dispatch_hosted(DomEventType::Error);
}

#[test]
fn hosted_style_load_kernel_excludes_global_but_captures_document_and_ancestor() {
    dispatch_hosted(DomEventType::Load);
}
