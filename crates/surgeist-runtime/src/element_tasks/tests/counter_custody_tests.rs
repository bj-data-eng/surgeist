//! An original Details counter may survive its disposed task owner.
use super::*;
use surgeist_dom::{DomResource, DomSourceEdit, DomSourceEventRequest, DomSourceLimits};

// Source setup retains the original materializations and their registry paths.
// Give that setup headroom before measuring the counter's exact released body.
const SOURCE_BYTES: usize = 1_048_576;

fn available_source_payload(dom: &Dom, request: &DomSourceEventRequest) -> usize {
    // Place the cut on this same original Source observation. The expected
    // difference below comes from the actual allocation, not this probe.
    let mut lower = 0;
    let mut upper = SOURCE_BYTES + 1;
    while upper - lower > 1 {
        let bytes = lower + (upper - lower) / 2;
        match dom.retain_source_event_payload(request, bytes, 0, &cancel()) {
            Ok(probe) => {
                drop(probe);
                lower = bytes;
            }
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: SOURCE_BYTES,
            }) => upper = bytes,
            other => panic!("the original Source storage policy decides the cut: {other:?}"),
        }
    }
    lower
}

#[test]
fn disposed_details_owner_counter_backing_remains_in_runtime_and_source_custody() {
    struct Observations {
        tasks: DomElementTaskRuntime,
        event: Option<crate::NativeDomEvent>,
    }
    impl DomElementTaskEventContext for Observations {
        fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
            &mut self.tasks
        }
    }

    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65_536,
            1024,
            1024,
            512,
            16,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            surgeist_dom::DomDispatchCustodyLimits::new(64, 4096, 1_000_000, 1_000_000, 100_000)
                .unwrap(),
        )
        .with_details_toggle_intentions(8)
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let details = batch.create_element_ns(&document, Namespace::html(), "details");
    let dialog = batch.create_element_ns(&document, Namespace::html(), "dialog");
    batch.insert_before(&details, &root, None);
    batch.insert_before(&root, &document, None);
    let created = dom.apply(batch, &cancel()).unwrap();
    let details = created.created(&details).unwrap().unwrap();
    let dialog = created.created(&dialog).unwrap().unwrap();

    // Use the actual constituent owners and their normal bindings, with
    // distinct Rust lifetimes so the native DOM outlives the task owner.
    let owner = ElementTaskOwner::new(dom.owner(), limits()).unwrap();
    let event_storage = crate::dom_event::EventStorage::session(
        &mut DomEventDom::borrow(&mut dom),
        crate::DomEventStorageLimits {
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        &cancel(),
    )
    .unwrap();
    let hosted = crate::dom_lifecycle::HostedOwner::new(
        &mut dom,
        DomHostedRuntimeLimits {
            canonical: surgeist_dom::DomHostedLimits::new(
                surgeist_dom::DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 32,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
        event_storage,
    )
    .unwrap();
    owner.bind_hosted(hosted.execution());
    hosted.bind_element_tasks(owner.execution());
    owner.bind_source(&mut dom);
    let mut context = Observations {
        tasks: owner.execution(),
        event: None,
    };
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    dispatcher
        .add_listener(
            &mut DomEventDom::borrow(&mut dom),
            details,
            DomEventType::TOGGLE,
            DomEventCallback::new(|_, _, context: &mut Observations, event| {
                context.event = Some(event.clone());
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(details, "open", "");
    dom.apply(batch, &cancel()).unwrap();
    assert_eq!(
        context
            .tasks
            .ingest(&mut DomEventDom::borrow(&mut dom), &cancel())
            .unwrap(),
        1
    );
    let source_limits = DomSourceLimits::new(SOURCE_BYTES, 1_000_000, 1_000_000, 16, 1024, 1024, 4);
    let DomElementTaskProgress::Complete { task, receipt, .. } = DomElementTaskRuntime::run_next(
        &mut DomEventDom::borrow(&mut dom),
        DomElementTaskSourceKind::DomManipulation,
        &mut dispatcher,
        &mut context,
        crate::DomElementTaskRunLimits {
            dispatch: dispatch_limits(),
            source: source_limits,
        },
        &cancel(),
    )
    .unwrap() else {
        panic!("actual Details delivery completes")
    };
    assert_eq!(task.id().target(), details);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    drop(receipt);
    assert_eq!(context.tasks.pending_total_tasks().unwrap(), 0);
    let event = context.event.take().unwrap();
    let toggle = event.toggle_event().unwrap();
    assert_eq!(
        (toggle.old_state(), toggle.new_state()),
        (
            "closed".encode_utf16().collect::<Vec<_>>().as_slice(),
            "open".encode_utf16().collect::<Vec<_>>().as_slice()
        )
    );

    // Hold one distinct admitted allocation so the actual Binding's bytes
    // stay constant across the probe. This resource receipt grants no task
    // execution permission and the source capability itself is not kept.
    let (counter, source) = {
        let state = owner.state.borrow();
        (
            state.payload_bytes.clone(),
            state.source.as_ref().unwrap().clone(),
        )
    };
    let anchor = source.retain_payload(&mut dom, 37, &cancel()).unwrap();
    let anchor_backing = vec![0_u8; 37];
    drop(source);
    drop(owner);
    assert!(matches!(
        context.tasks.pending_total_tasks(),
        Err(DomElementTaskRuntimeError::OwnerDisposed)
    ));
    assert_eq!(event.current_target(), None);
    assert_eq!(event.phase(), DomEventPhase::None);
    assert!(
        counter.get() > 0,
        "the real Event still retains its original resource bytes"
    );
    drop(event);
    drop(dispatcher);

    let executor = dom.bind_source_event_executor(&cancel()).unwrap();
    let request = {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::show_dialog(dialog), source_limits, &cancel())
            .unwrap();
        let surgeist_dom::DomSourceProgress::Request(surgeist_dom::DomSourceRequest::Event(
            request,
        )) = lease.advance(&cancel()).unwrap()
        else {
            panic!("actual Show parks at its original event")
        };
        request
    };
    let permit = executor.begin_dispatch(&dom, &request, &cancel()).unwrap();
    let guard = executor.enter_dispatch(&dom, &permit, &cancel()).unwrap();
    let stamp = dom.snapshot().stamp();
    let body = std::mem::size_of_val(counter.as_ref());
    let runtime_bytes = counter.get();
    let with_counter = available_source_payload(&dom, &request);
    drop(counter);
    let without_counter = available_source_payload(&dom, &request);
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        request.operation()
    );
    assert_eq!(
        (
            runtime_bytes,
            without_counter.checked_sub(with_counter).unwrap()
        ),
        (body, body),
        "the surviving counter target is charged by both original owners until its final Rc drops"
    );
    drop(guard);
    drop(anchor_backing);
    drop(anchor);
}
