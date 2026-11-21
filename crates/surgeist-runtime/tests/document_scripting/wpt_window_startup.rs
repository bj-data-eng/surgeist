//! Startup of the pinned, unmodified Window harness on an attached Runtime host.

use super::*;
use std::time::{Duration, Instant};
use surgeist_runtime::document_scripting::{WindowTimerLimits, WindowTimerProgress};

pub(super) fn starts_unmodified_harness_with_native_timers() {
    let path = std::env::args_os()
        .nth(2)
        .expect("supply the pinned testharness.js path");
    let harness = std::fs::read_to_string(path).expect("read the pinned harness");
    let script: Vec<_> = harness.encode_utf16().collect();

    for expected_seconds in [10, 60] {
        let (mut session, top) = setup();
        if expected_seconds == 60 {
            let snapshot = session.snapshot();
            let html = snapshot.node(top.document().node()).unwrap().children()[0];
            let head = snapshot.node(html).unwrap().children()[0];
            let mut edit = DomBatch::new();
            let meta = edit.create_element_ns(top.document(), Namespace::html(), "meta");
            edit.set_attribute(&meta, "name", "timeout");
            edit.set_attribute(&meta, "content", "long");
            edit.insert_before(&meta, head, None);
            session.dom().apply(edit, &cancel()).unwrap();
        }
        let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 32,
            pending_events: 4,
            nested_dispatches: 4,
            storage_bytes: 128_000,
            work: 100_000,
        })
        .unwrap();
        registry.bind(&mut session.dom(), &cancel()).unwrap();
        let mut registry = Some(registry);
        let mut host = prepare(session, Arc::new(cancel()))
            .with_window_timer_limits(WindowTimerLimits::new(16, 8, 4096))
            .enable_with_guest_event_registry(
                top,
                ALLOW,
                None,
                &mut registry,
                DocumentGuestEventLimits {
                    storage_bytes: 640 * 1024,
                    work: 100_000,
                    pending_exception_reports: 16,
                    node_retirement: DomNodeRetirementLimits::new(16_000_000, 1000),
                    source_edits: DomSourceLimits::new(16_000_000, 200_000, 20_000, 64, 64, 64, 2),
                    dispatch: DomEventDispatchLimits {
                        path_items: 4,
                        snapshot_listeners: 32,
                        failure_records: 8,
                        callbacks: 32,
                        work: 10_000,
                    },
                },
            )
            .unwrap();
        assert!(registry.is_none());
        let permit = host.permit().unwrap();
        let start = Instant::now();
        let startup = host.in_document(&permit, |scope| scope.evaluate(&script).map(|_| ()));
        if let Err(DocumentScriptError::Script(Error::Exception(exception))) = startup {
            let message = host
                .in_document(&permit, |scope| {
                    let thrown = scope.import(exception.value())?;
                    let message = b"message".map(u16::from);
                    let message = scope.get(&thrown, PropertyKey::String(&message))?;
                    scope.to_utf16(&message)
                })
                .unwrap();
            panic!(
                "Window harness startup threw: {}",
                String::from_utf16(&message).unwrap()
            );
        }
        startup.unwrap();
        let progress = host.service_window_timer(&permit).unwrap();
        let WindowTimerProgress::Waiting(remaining) = progress else {
            panic!("expected the original harness timeout to be scheduled: {progress:?}");
        };
        let expected = Duration::from_secs(expected_seconds);
        let elapsed = start.elapsed();
        let tolerance = Duration::from_millis(1);
        assert!(
            remaining <= expected + tolerance && remaining + elapsed + tolerance >= expected,
            "expected {expected:?}, observed {remaining:?} after {elapsed:?}"
        );
        assert!(host.stop().into_session().is_ok());
    }
}
