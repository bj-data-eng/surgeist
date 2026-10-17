//! A handed-off Select intention keeps its retirement outcome across Session adoption.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]

use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_runtime::*;

fn session_limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: DomSourceEventRuntimeLimits {
            jobs: 0,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 8,
            reports: 8,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: Default::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 8,
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
    }
}

#[test]
fn runtime_select_completion_preserves_a_real_handoff_retirement_skip() {
    let cancel = AtomicBool::new(false);
    let checkpoint = DomNativeCheckpointLimits::new(
        4_000_000,
        10_000_000,
        DomSlotQueryLimits::new(128, 512).unwrap(),
    )
    .unwrap();
    let mut dom = Dom::new(
        DomLimits::new(128, 32768, 512, 512, 128, 16, 1_000_000)
            .unwrap()
            .with_select_processing(32, 4_000_000, 10_000_000)
            .unwrap()
            .with_select_value_queries(
                surgeist_html_semantics::HtmlCaptureLimits::new(128, 512, 512, 32768, 128, 0, 0),
                surgeist_html_semantics::HtmlSemanticLimits::new(
                    10_000_000, 1, 512, 0, 4_000_000, 512,
                ),
            ),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let select = batch.create_element_ns(&document, Namespace::html(), "select");
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    batch.set_attribute(&option, "selected", "");
    batch.insert_before(&option, &select, None);
    batch.insert_before(&select, &root, None);
    let survivor = batch.create_element_ns(&document, Namespace::html(), "select");
    let survivor_option = batch.create_element_ns(&document, Namespace::html(), "option");
    batch.set_attribute(&survivor_option, "selected", "");
    batch.insert_before(&survivor_option, &survivor, None);
    batch.insert_before(&survivor, &root, None);
    batch.insert_before(&root, &document, None);
    let setup = dom.apply(batch, &cancel).unwrap();
    let document = setup.document(&document).unwrap();
    let root = setup.created(&root).unwrap().unwrap();
    let select = setup.created(&select).unwrap().unwrap();
    let option = setup.created(&option).unwrap().unwrap();
    let survivor = setup.created(&survivor).unwrap().unwrap();
    let survivor_option = setup.created(&survivor_option).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.detach(option);
    batch.detach(survivor_option);
    dom.apply(batch, &cancel).unwrap();
    assert_eq!(dom.pending_native_tasks(), 2);
    let DomNativeCheckpoint::Select(task) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("the first real Option removal must hand off its captured Select");
    };
    let operation = task.operation();
    assert_eq!(task.select(), select);
    let historical = task.source().clone();
    assert_eq!(historical.node(select).unwrap().parent(), Some(root));
    assert!(historical.node(option).unwrap().parent().is_none());

    // Move the ORIGINAL unbound owner, with its actual active authorization and
    // later FIFO intention, into the public durable Runtime composition.
    let mut session = DomSession::new(dom, session_limits()).unwrap();
    assert_eq!(session.snapshot().owner(), historical.owner());
    let mut batch = DomBatch::new();
    batch.detach(select);
    batch.destroy(select);
    let retired = session.dom().apply(batch, &cancel).unwrap();
    assert!(
        retired
            .changes()
            .changes()
            .contains(&DomChange::SelectTaskCancelled {
                operation,
                select,
                reason: DomSelectTaskCancellationReason::TargetDestroyed,
            })
    );
    let mut batch = DomBatch::new();
    let replacement = batch.create_element_ns(document, Namespace::html(), "select");
    batch.set_attribute(&replacement, "data-replacement", "unchanged");
    batch.insert_before(&replacement, root, None);
    let setup = session.dom().apply(batch, &cancel).unwrap();
    let replacement = setup.created(&replacement).unwrap().unwrap();
    assert_ne!(replacement, select);
    let before = session.snapshot();
    assert!(matches!(before.node(select), Err(DomError::StaleNode)));
    assert!(session.dom().pending_source_edit().is_none());
    let mut runtime = session.source_execution();
    let mut unexpected_policy = |_: &DomStyleSource| -> Option<DomStyleInlineCspObservation> {
        panic!("a retired Select skip must not request unrelated Style services")
    };
    let result = runtime.complete_select(
        &mut session.dom(),
        &task,
        checkpoint,
        DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 1),
        &mut DomSourceServices {
            execution_scope: None,
            style_policy: &mut unexpected_policy,
            url: None,
        },
        &cancel,
    );
    // #1239 requires the authorized executor to consume the historical skip
    // before live-only Source admission. The current Runtime return type has
    // no skip variant, so this first regression asserts success at that seam.
    assert!(
        result.is_ok(),
        "actual retired handoff must complete as a skip: {result:?}"
    );
    assert!(matches!(
        runtime.complete_select(
            &mut session.dom(),
            &task,
            checkpoint,
            DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 1),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut unexpected_policy,
                url: None,
            },
            &cancel,
        ),
        Err(DomSourceRuntimeError::Dom(DomError::InvalidNativeTask))
    ));
    let after = session.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(task.operation(), operation);
    assert_eq!(task.select(), select);
    assert_eq!(task.source().stamp(), historical.stamp());
    assert_eq!(historical.node(select).unwrap().parent(), Some(root));
    for id in [replacement, survivor, survivor_option] {
        assert_eq!(
            after.node(id).unwrap().parent(),
            before.node(id).unwrap().parent()
        );
        assert_eq!(after.node(id).unwrap().owner_document(), Some(document));
    }
    assert_eq!(
        after.node(root).unwrap().children(),
        &[survivor, replacement]
    );
    assert_eq!(
        after
            .node(replacement)
            .unwrap()
            .attribute(&"data-replacement".into()),
        Some(&"unchanged".into())
    );
    // The later live captured intention still belongs to its survivor. There is
    // no public Session checkpoint drain; retire it through the real owner and
    // require its distinct cancellation evidence rather than inventing a turn.
    let mut batch = DomBatch::new();
    batch.detach(survivor);
    batch.destroy(survivor);
    let cleanup = session.dom().apply(batch, &cancel).unwrap();
    let canceled: Vec<_> = cleanup
        .changes()
        .changes()
        .iter()
        .filter_map(|change| match change {
            DomChange::SelectTaskCancelled {
                operation,
                select,
                reason,
            } => {
                assert_eq!(*reason, DomSelectTaskCancellationReason::TargetDestroyed);
                Some((*operation, *select))
            }
            _ => None,
        })
        .collect();
    assert_eq!(canceled.len(), 1);
    assert_eq!(canceled[0].1, survivor);
    assert_ne!(canceled[0].0, operation);
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}
