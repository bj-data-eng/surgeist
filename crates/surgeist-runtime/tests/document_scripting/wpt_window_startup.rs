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
    let fixtures: Vec<_> = std::env::args_os()
        .skip(3)
        .map(|path| std::fs::read_to_string(path).expect("read the exact EventTarget fixture"))
        .collect();
    assert!(fixtures.is_empty() || fixtures.len() == 2);

    for expected_seconds in [10, 60] {
        let mut dom = native_dom();
        let recipients =
            surgeist_dom::event::DomEventRecipientLimits::new(32, 16_000_000, 16_000_000, 100_000)
                .unwrap();
        dom.set_limits(
            dom.limits()
                .with_event_recipients(recipients)
                .with_meta_processing_work(100_000),
        )
        .unwrap();
        let mut session = DomSession::new(dom, limits()).unwrap();
        let (_, top) = create(&mut session);
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
        if let Err(error) = startup {
            use std::error::Error as _;
            let mut cause = error.source();
            while let Some(error) = cause {
                eprintln!("{error:?}");
                cause = error.source();
            }
            panic!("Window harness startup failed: {error:?}");
        }
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
        println!("Runtime Window harness scheduled {expected_seconds} s without delivering load");
        exercise_live_meta_collection(&mut host, &permit, top.document(), expected_seconds == 60);
        if !fixtures.is_empty() {
            observe_event_target_subtests(&mut host, &permit, &fixtures);
        }
        assert!(host.stop().into_session().is_ok());
    }
}

fn exercise_live_meta_collection(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    document: DocumentId,
    populated: bool,
) {
    let base = usize::from(populated);
    host.in_document(permit, |scope| {
        let script = format!(
            r#"Number((() => {{
            const list = document.getElementsByTagName("MeTa");
            globalThis.startupMetaList = list;
            const length = Object.getOwnPropertyDescriptor(HTMLCollection.prototype, "length").get;
            try {{ length.call(Object.create(HTMLCollection.prototype)); return false; }}
            catch (error) {{ if (!(error instanceof TypeError)) return false; }}
            return list instanceof HTMLCollection && list.length === {base} &&
                list.item({base}) === null && list[{base}] === undefined;
        }})())"#
        );
        assert_script_number(scope, &script, 1.0)
    })
    .unwrap();

    let (head, first, second) = host
        .with_native(|dom, _, _| {
            let snapshot = dom.snapshot();
            let html = snapshot.node(document.node()).unwrap().children()[0];
            let head = snapshot.node(html).unwrap().children()[0];
            let mut edit = DomBatch::new();
            let first = edit.create_element_ns(document, Namespace::html(), "meta");
            let second = edit.create_element_ns(document, Namespace::html(), "meta");
            edit.set_attribute(&first, "id", "held-first");
            edit.set_attribute(&second, "id", "held-second");
            edit.set_attribute(&first, "name", "ordinary");
            edit.insert_before(&first, head, None);
            edit.insert_before(&second, head, None);
            let receipt = dom.apply(edit, &cancel()).unwrap();
            (
                head,
                receipt.created(&first).unwrap().unwrap(),
                receipt.created(&second).unwrap().unwrap(),
            )
        })
        .unwrap();
    host.in_document(permit, |scope| {
        let script = format!(
            r#"Number(startupMetaList.length === {} &&
            startupMetaList[{base}] === document.getElementById("held-first") &&
            startupMetaList.item({base}) === startupMetaList[{base}] &&
            startupMetaList[{base}].name === "ordinary" &&
            startupMetaList[{base}].content === "" &&
            startupMetaList[{}] === document.getElementById("held-second"))"#,
            base + 2,
            base + 1
        );
        assert_script_number(scope, &script, 1.0)
    })
    .unwrap();

    host.with_native(|dom, _, _| {
        let mut edit = DomBatch::new();
        edit.insert_before(
            second,
            head,
            Some(surgeist_dom::edit::NodeOperand::Existing(first)),
        );
        edit.set_attribute(second, "content", "updated");
        dom.apply(edit, &cancel()).unwrap();
    })
    .unwrap();
    host.in_document(permit, |scope| {
        let script = format!(
            r#"Number(startupMetaList[{base}].content === "updated" &&
            startupMetaList[{base}] === document.getElementById("held-second"))"#
        );
        assert_script_number(scope, &script, 1.0)
    })
    .unwrap();

    host.with_native(|dom, _, _| {
        let mut edit = DomBatch::new();
        edit.detach(first);
        edit.detach(second);
        dom.apply(edit, &cancel()).unwrap();
    })
    .unwrap();
    host.in_document(permit, |scope| {
        let script = format!(
            "Number(startupMetaList.length === {base} && startupMetaList[{base}] === undefined)"
        );
        assert_script_number(scope, &script, 1.0)
    })
    .unwrap();
}

fn observe_event_target_subtests(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    fixtures: &[String],
) {
    host.in_document(permit, |scope| {
        let install = r#"
            globalThis.startupSubtests = [];
            globalThis.startupComplete = false;
            add_result_callback(test => startupSubtests.push([test.name, test.status]));
            add_completion_callback(() => { startupComplete = true; });
        "#;
        scope.evaluate(&install.encode_utf16().collect::<Vec<_>>())?;
        for fixture in fixtures {
            scope.evaluate(&fixture.encode_utf16().collect::<Vec<_>>())?;
        }
        assert_script_number(scope, r#"Number(!startupComplete && startupSubtests.length === 2 &&
            startupSubtests[0][0] === "Adding a null event listener should succeed" &&
            startupSubtests[0][1] === 0 &&
            startupSubtests[1][0] === "Removing an event listener without explicit capture arg should succeed" &&
            startupSubtests[1][1] === 0)"#, 1.0)
    }).unwrap();
    println!(
        "Both upstream EventTarget assertions passed while harness completion remained pending"
    );
}

pub(super) fn refuses_unsupported_html_collection_index_assignment() {
    let (mut host, _) = prepare_event_window();
    let permit = host.permit().unwrap();
    let script = r#"(() => {
        const list = document.getElementsByTagName("meta");
        if (list.length !== 0 || list[0] !== undefined || Object.hasOwn(list, "0")) {
            return "expected an unsupported index in the original empty HTMLCollection";
        }
        const assigned = Reflect.set({}, "0", "unexpected", list);
        const ownsIndex = Object.hasOwn(list, "0");
        if (assigned || ownsIndex) {
            return "HTMLCollection must refuse own index creation through an alternate target"
                + "; Reflect.set returned " + assigned + ", own index " + ownsIndex;
        }
        return "ok";
    })()"#;
    let result = host
        .in_document(&permit, |scope| {
            let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            scope.to_utf16(&value)
        })
        .unwrap();
    assert_eq!(String::from_utf16(&result).unwrap(), "ok");
}
