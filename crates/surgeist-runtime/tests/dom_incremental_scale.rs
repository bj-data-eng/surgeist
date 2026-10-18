//! Explicit production Session -> Source qualification after #1262 integration.
//! Select this ignored test in release mode with --test-threads=1 --nocapture.
//! The local XML case requires no external service. The parked case uses the
//! actual beforetoggle dispatcher, nested Source edits and original receipt.
//! Missing later services remain unavailable; no successful receipt is faked.
#![forbid(unsafe_code)]

use std::hint::black_box;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use surgeist_dom::{
    DocumentString, Dom, DomBatch, DomChange, DomHostedLimits, DomHostedRecordLimits, DomLimits,
    DomSourceEdit, DomSourceLimits,
};
use surgeist_runtime::{
    DomElementTaskRuntimeLimits, DomHostedRuntimeLimits, DomSession, DomSessionLimits,
    DomSourceDriveProgress, DomSourceEventRuntimeLimits, DomSourceServices, DomStyleRuntimeLimits,
};

const CHUNK: usize = 8192;
const SAMPLES: usize = 3;
const WARMUPS: usize = 1;
const SOURCE_WORK: u64 = 20_000;

#[expect(
    clippy::assertions_on_constants,
    reason = "Keep ignored qualification cases compilable in debug while preserving their runtime release-only guard"
)]
fn require_release() {
    // These ignored cases must compile in normal debug test binaries and
    // refuse execution only when the caller explicitly selects them there.
    assert!(
        !cfg!(debug_assertions),
        "select this fixture in a release build"
    );
}

fn selected_sizes() -> Vec<usize> {
    let defaults = [10_000usize, 100_000, 1_000_000, 2_000_000];
    let Some(value) = std::env::var_os("SURGEIST_SCALE_SIZES") else {
        return defaults.to_vec();
    };
    let mut sizes = Vec::new();
    for item in value.to_str().expect("UTF-8 size selection").split(',') {
        let size: usize = item.parse().expect("comma-separated managed-node counts");
        assert!(
            size == 1000 || defaults.contains(&size),
            "unsupported size {size}"
        );
        assert!(!sizes.contains(&size), "duplicate size {size}");
        sizes.push(size);
    }
    assert!(!sizes.is_empty());
    eprintln!("runtime_scale explicit_selected_sizes={sizes:?} full_default={defaults:?}");
    sizes
}

// Match the finite DOM qualification policy for an actual two-million-node
// owner plus selected-operation headroom. This fixture capacity is not an
// expected usage or admission threshold; production still admits actual usage.
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(
        usize::try_from(32_000_000_000u64).expect("scale qualification requires a 64-bit host"),
        512_000_000,
        SOURCE_WORK,
        8,
        16,
        16,
        1,
    )
}

fn session_limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: DomSourceEventRuntimeLimits {
            jobs: 8,
            storage_bytes: 64_000_000,
            work: 4_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 8,
            completions: 32,
            reports: 32,
            storage_bytes: 64_000_000,
            work: 4_000_000,
            css_input: surgeist_css::CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 32,
            storage_bytes: 64_000_000,
            work: 4_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                64_000_000,
                64_000_000,
                4_000_000,
            ),
            tasks: 32,
            storage_bytes: 64_000_000,
            scratch_bytes: 64_000_000,
            work: 4_000_000,
        },
    }
}

fn report(label: &str, samples: &[Duration]) {
    assert_eq!(samples.len(), SAMPLES);
    let mut samples = samples.to_vec();
    samples.sort_unstable();
    // Report this finite sample directly; the maximum is not a tail percentile.
    eprintln!(
        "runtime_scale measurement={label} samples={} median_ns={} observed_maximum_of_three_ns={}",
        samples.len(),
        samples[samples.len() / 2].as_nanos(),
        samples.last().unwrap().as_nanos()
    );
}

#[test]
#[ignore = "explicit release million-node production Source qualification after #1262"]
fn production_session_local_source_scale() {
    require_release();
    eprintln!(
        "runtime_scale environment profile=release os={} arch={} revision={} toolchain={} machine={} timing=uninstrumented source_storage_configuration_bytes=32000000000 storage_configuration_not_oracle=true private_counters=not_connected warmups={WARMUPS} samples={SAMPLES}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::env::var("SURGEIST_SCALE_REVISION").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_MACHINE").unwrap_or_else(|_| "unknown".into())
    );
    let cancel = AtomicBool::new(false);
    for managed in selected_sizes() {
        let started = Instant::now();
        let limits =
            DomLimits::new(managed, managed * 128, CHUNK * 2, 0, 0, 32, 4_000_000).unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let target = batch.create_element(&document, "target");
        batch.set_attribute(&target, "data-local", "base");
        batch.insert_before(&target, &document, None);
        let other = batch.create_document();
        let commit = dom.apply(batch, &cancel).unwrap();
        let target = commit.created(&target).unwrap().unwrap();
        let other = commit.document(&other).unwrap();
        let mut remaining = managed - 3;
        while remaining > 0 {
            let count = remaining.min(CHUNK);
            let mut batch = DomBatch::new();
            for _ in 0..count {
                batch.create_comment(other, "u");
            }
            dom.apply(batch, &cancel).unwrap();
            remaining -= count;
        }
        assert_eq!(dom.usage().managed_nodes(), managed);
        let construction = started.elapsed();
        assert_eq!(dom.snapshot().nodes().count(), managed);
        let admitted = session_limits();
        let started = Instant::now();
        let mut session = DomSession::new(dom, admitted).unwrap();
        let adoption = started.elapsed();
        eprintln!(
            "runtime_scale fixture managed={managed} unrelated={} setup_ns={} session_admission_ns={} canonical_limits={limits:?} session_limits={admitted:?} canonical_usage={:?} source_work_limit={SOURCE_WORK}",
            managed - 3,
            construction.as_nanos(),
            adoption.as_nanos(),
            session.dom().usage()
        );
        eprintln!("runtime_scale source_limits={:?}", source_limits());
        let original = session.snapshot();
        let mut snapshot_samples = Vec::new();
        let mut snapshots = Vec::new();
        for sample in 0..WARMUPS + SAMPLES {
            let started = Instant::now();
            let snapshot = black_box(session.snapshot());
            let elapsed = started.elapsed();
            if sample >= WARMUPS {
                snapshot_samples.push(elapsed);
            }
            snapshots.push(snapshot);
        }
        report("snapshot_acquire", &snapshot_samples);
        let mut snapshot_drop_samples = Vec::new();
        for (sample, snapshot) in snapshots.into_iter().enumerate() {
            let started = Instant::now();
            drop(black_box(snapshot));
            let elapsed = started.elapsed();
            if sample >= WARMUPS {
                snapshot_drop_samples.push(elapsed);
            }
        }
        report("snapshot_drop_shared", &snapshot_drop_samples);

        // Exercise the actual bound Runtime Source driver. Plain XML has no
        // CSP, URL, event, hosted or resource request to acknowledge here.
        let mut mutation_samples = Vec::new();
        for sample in 0..WARMUPS + SAMPLES {
            let next = if sample % 2 == 0 { "next" } else { "base" };
            let before = session.snapshot();
            let edit = DomSourceEdit::set_attribute(target, "data-local", next);
            let limits = source_limits();
            let started = Instant::now();
            let progress = session
                .source_execution()
                .apply(
                    &mut session.dom(),
                    edit,
                    limits,
                    &mut DomSourceServices {
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &cancel,
                )
                .unwrap();
            let elapsed = started.elapsed();
            let DomSourceDriveProgress::Complete(done) = progress else {
                panic!("plain XML attribute operation has no unresolved external service");
            };
            assert_eq!(
                done.commit().changes().changes(),
                &[DomChange::Attributes(target)]
            );
            let now = session.snapshot();
            assert_eq!(now.revision().get(), before.revision().get() + 1);
            assert_eq!(
                now.node(target)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .attribute(&"data-local".into()),
                Some(&DocumentString::from(next))
            );
            assert_eq!(session.dom().usage().managed_nodes(), managed);
            assert!(session.dom().pending_source_edit().is_none());
            if sample >= WARMUPS {
                mutation_samples.push(elapsed);
            }
        }
        report("production_source_attribute", &mutation_samples);
        assert_eq!(
            original
                .node(target)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"data-local".into()),
            Some(&DocumentString::from("base"))
        );
        assert_eq!(original.nodes().count(), managed);
        eprintln!("runtime_scale final_usage={:?}", session.dom().usage());
        let started = Instant::now();
        drop(session);
        eprintln!(
            "runtime_scale session_drop_held_snapshot_ns={}",
            started.elapsed().as_nanos()
        );
        let started = Instant::now();
        drop(original);
        eprintln!(
            "runtime_scale snapshot_drop_last_ns={}",
            started.elapsed().as_nanos()
        );
    }
}

fn continuation_limits() -> DomSourceLimits {
    let limits = source_limits();
    DomSourceLimits::new(
        limits.retained_bytes(),
        limits.retained_string_code_units(),
        100_000,
        64,
        4096,
        4096,
        4,
    )
}

/// This measured case uses the real Runtime dispatcher and its sealed receipt.
/// Its callback publishes three real nested Source results, including a new
/// public identity, then the original Show observes `open` and returns normally.
/// No unimplemented focus/host service is acknowledged.
#[test]
#[ignore = "explicit release parked Source event, actual Runtime receipt and reentry after public million-node setup"]
fn production_session_parked_source_reentry_scale() {
    use surgeist_dom::{
        DomDispatchCustodyLimits, DomSourceOutput, DomSourceRequest, Namespace, NodeId,
    };
    use surgeist_runtime::{
        DomEventCallback, DomEventDispatchLimits, DomEventDispatcher, DomEventListenerOptions,
        DomEventRegistryLimits, DomEventType, DomSourceEventServiceLimits,
        DomSourceEventServiceProgress, DomSourceRuntime, DomSourceUnavailable,
    };
    require_release();
    let cancel = AtomicBool::new(false);
    for managed in selected_sizes() {
        let setup = Instant::now();
        let limits = DomLimits::new(managed + 1, managed * 128, CHUNK * 4, 0, 0, 64, 4_000_000)
            .unwrap()
            .with_dispatch_custody(
                DomDispatchCustodyLimits::new(8, 128, 1_000_000, 1_000_000, 100_000).unwrap(),
            );
        let mut dom = Dom::new(limits).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let subject = batch.create_element_ns(&document, Namespace::html(), "dialog");
        batch.insert_before(&subject, &document, None);
        let other = batch.create_document();
        let target = batch.create_element(&other, "callback-target");
        batch.set_attribute(&target, "data-local", "base");
        let commit = dom.apply(batch, &cancel).unwrap();
        let subject = commit.created(&subject).unwrap().unwrap();
        let target = commit.created(&target).unwrap().unwrap();
        let other = commit.document(&other).unwrap();
        let mut remaining = managed - 4;
        while remaining > 0 {
            let mut batch = DomBatch::new();
            let count = remaining.min(CHUNK);
            for _ in 0..count {
                batch.create_comment(other, "u");
            }
            dom.apply(batch, &cancel).unwrap();
            remaining -= count;
        }
        assert_eq!(dom.usage().managed_nodes(), managed);
        assert_eq!(dom.snapshot().nodes().count(), managed);
        let construction = setup.elapsed();
        let mut session = DomSession::new(dom, session_limits()).unwrap();
        let mut listeners = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 1,
            pending_events: 1,
            nested_dispatches: 4,
        })
        .unwrap();
        struct Callback {
            source: DomSourceRuntime,
            next: &'static str,
            calls: usize,
            created: Option<NodeId>,
        }
        listeners
            .add_listener(
                &session.dom(),
                subject,
                DomEventType::BeforeToggle,
                DomEventCallback::new(move |dom, _, context: &mut Callback, _| {
                    context.calls += 1;
                    let mut services = DomSourceServices {
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    };
                    let DomSourceDriveProgress::Complete(changed) = context
                        .source
                        .apply(
                            dom,
                            DomSourceEdit::set_attribute(target, "data-local", context.next),
                            continuation_limits(),
                            &mut services,
                            &AtomicBool::new(false),
                        )
                        .unwrap()
                    else {
                        panic!("plain callback attribute needs no service");
                    };
                    assert_eq!(
                        changed.commit().changes().changes(),
                        &[DomChange::Attributes(target)]
                    );
                    let DomSourceDriveProgress::Complete(created) = context
                        .source
                        .apply(
                            dom,
                            DomSourceEdit::create_comment(other.node(), "callback-created"),
                            continuation_limits(),
                            &mut services,
                            &AtomicBool::new(false),
                        )
                        .unwrap()
                    else {
                        panic!("plain callback constructor needs no service");
                    };
                    let Some(DomSourceOutput::Node(id)) = created.output() else {
                        panic!("actual constructor output");
                    };
                    assert_eq!(created.published_node(), Some(id));
                    assert_eq!(
                        created.commit().changes().changes(),
                        &[DomChange::Created(id)]
                    );
                    context.created = Some(id);
                    let DomSourceDriveProgress::Complete(opened) = context
                        .source
                        .apply(
                            dom,
                            DomSourceEdit::set_attribute(subject, "open", ""),
                            continuation_limits(),
                            &mut services,
                            &AtomicBool::new(false),
                        )
                        .unwrap()
                    else {
                        panic!("real callback setter completes through its own Source");
                    };
                    assert_eq!(
                        opened.commit().changes().changes(),
                        &[DomChange::Attributes(subject)]
                    );
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let mut callback = Callback {
            source: session.source_execution(),
            next: "next",
            calls: 0,
            created: None,
        };
        let mut begin_samples = Vec::new();
        let mut service_samples = Vec::new();
        let mut resume_samples = Vec::new();
        let original = session.snapshot();
        eprintln!(
            "runtime_scale continuation profile=release os={} arch={} warmups={WARMUPS} samples={SAMPLES} managed={managed} setup_ns={} limits={limits:?} source_limits={:?} revision={} toolchain={} machine={} private_counters=DOM_owning_companion",
            std::env::consts::OS,
            std::env::consts::ARCH,
            construction.as_nanos(),
            continuation_limits(),
            std::env::var("SURGEIST_SCALE_REVISION").unwrap_or_else(|_| "unknown".into()),
            std::env::var("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|_| "unknown".into()),
            std::env::var("SURGEIST_SCALE_MACHINE").unwrap_or_else(|_| "unknown".into())
        );
        for sample in 0..WARMUPS + SAMPLES {
            callback.next = if sample % 2 == 0 { "next" } else { "base" };
            let before = session.snapshot();
            let start = Instant::now();
            let progress = session
                .source_execution()
                .apply(
                    &mut session.dom(),
                    DomSourceEdit::show_dialog(subject),
                    continuation_limits(),
                    &mut DomSourceServices {
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &cancel,
                )
                .unwrap();
            let begin = start.elapsed();
            let DomSourceDriveProgress::Pending {
                request: DomSourceRequest::Event(request),
                reason: DomSourceUnavailable::Event,
            } = progress
            else {
                panic!("the actual Show parks at beforetoggle");
            };
            let operation = request.operation();
            let parked = session.dom().pending_source_edit().unwrap();
            let parked_work = parked.spent_work();
            assert_eq!(parked.operation(), operation);
            assert_eq!(session.snapshot().stamp(), before.stamp());
            let start = Instant::now();
            let receipt = session
                .source_execution()
                .service_event(
                    &mut session.dom(),
                    &request,
                    &mut listeners,
                    &mut callback,
                    DomSourceEventServiceLimits {
                        source: continuation_limits(),
                        dispatch: DomEventDispatchLimits {
                            path_items: 16,
                            snapshot_listeners: 4,
                            failure_records: 4,
                            callbacks: 4,
                            work: 100_000,
                        },
                    },
                    &cancel,
                )
                .unwrap();
            let service = start.elapsed();
            let DomSourceEventServiceProgress::Serviced { receipt } = receipt else {
                panic!("real dispatcher completed its one callback");
            };
            assert_eq!(receipt.callbacks(), 1);
            assert!(receipt.failures().is_empty());
            assert!(!receipt.default_prevented());
            assert_eq!(receipt.original_target(), subject);
            assert_eq!(receipt.event_type(), DomEventType::BeforeToggle);
            assert_eq!(callback.calls, sample + 1);
            let id = callback.created.take().unwrap();
            let published = session.snapshot();
            assert_eq!(published.revision().get(), before.revision().get() + 3);
            assert_eq!(
                published.node(id).unwrap().data(),
                Some(&"callback-created".into())
            );
            assert_eq!(published.node(id).unwrap().owner_document(), Some(other));
            assert!(before.node(id).is_err());
            assert!(session.dom().pending_source_edit().unwrap().spent_work() > parked_work);
            let start = Instant::now();
            let progress = session
                .source_execution()
                .resume(
                    &mut session.dom(),
                    operation,
                    continuation_limits(),
                    &mut DomSourceServices {
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &cancel,
                )
                .unwrap();
            let resume = start.elapsed();
            let DomSourceDriveProgress::Complete(done) = progress else {
                panic!(
                    "original Show rechecks actual callback open without requesting a fake service"
                );
            };
            assert_eq!(done.operation(), operation);
            assert!(done.commit().changes().changes().is_empty());
            assert_eq!(session.snapshot().stamp(), published.stamp());
            assert_eq!(
                session
                    .snapshot()
                    .node(target)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .attribute(&"data-local".into()),
                Some(&DocumentString::from(callback.next))
            );
            assert_eq!(
                before
                    .node(subject)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .attribute(&"open".into()),
                None
            );
            assert!(session.dom().pending_source_edit().is_none());
            assert!(
                session
                    .source_execution()
                    .service_event(
                        &mut session.dom(),
                        &request,
                        &mut listeners,
                        &mut callback,
                        DomSourceEventServiceLimits {
                            source: continuation_limits(),
                            dispatch: DomEventDispatchLimits {
                                path_items: 16,
                                snapshot_listeners: 4,
                                failure_records: 4,
                                callbacks: 4,
                                work: 100_000
                            }
                        },
                        &cancel,
                    )
                    .is_err(),
                "acknowledged original receipt cannot replay"
            );
            assert_eq!(callback.calls, sample + 1);
            eprintln!(
                "runtime_scale continuation sample={sample} managed={managed} parked_work={parked_work} begin_ns={} actual_dispatch_and_reentry_ns={} original_resume_ns={} created={id:?}",
                begin.as_nanos(),
                service.as_nanos(),
                resume.as_nanos()
            );
            if sample >= WARMUPS {
                begin_samples.push(begin);
                service_samples.push(service);
                resume_samples.push(resume);
            }
            // Reset through the real owner outside the measured interval.
            let mut cleanup = DomBatch::new();
            cleanup.remove_attribute(subject, "open");
            cleanup.destroy(id);
            session.dom().apply(cleanup, &cancel).unwrap();
            assert_eq!(session.dom().usage().managed_nodes(), managed);
        }
        report("parked_source_begin", &begin_samples);
        report("actual_event_dispatch_nested_source", &service_samples);
        report("same_operation_resume", &resume_samples);
        assert_eq!(original.nodes().count(), managed);
        assert_eq!(
            original
                .node(target)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"data-local".into()),
            Some(&"base".into())
        );
        let start = Instant::now();
        drop(callback);
        drop(listeners);
        drop(session);
        eprintln!(
            "runtime_scale continuation_owner_drop_held_snapshot_ns={}",
            start.elapsed().as_nanos()
        );
        let start = Instant::now();
        drop(original);
        eprintln!(
            "runtime_scale continuation_snapshot_last_drop_ns={}",
            start.elapsed().as_nanos()
        );
    }
}
