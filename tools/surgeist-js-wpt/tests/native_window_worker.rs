//! Process-level qualification of the actual Runtime-owned Window worker.

use serde::Deserialize;
use std::{collections::BTreeMap, env, path::PathBuf, sync::atomic::AtomicBool, time::Duration};
use surgeist_fuzz::worker::read_frame;
use surgeist_js_wpt::{
    CleanupStatus, EngineResourceEvidence, FixtureId, GuestDiagnostic, HarnessStatus, MemoryPolicy,
    NativeResourceEvidence, ProcessExit, ProcessStop, RelativePath, Report, Run, SourceRevision,
    SubtestStatus, WorkerFailure, WorkerLimits, WorkerReport, WorkerResult, execution_capabilities,
    run,
};

const NATIVE_WINDOW: &str = "e1e9b737e7e543f42be6afdd19b498d66525c99e";
const ASSERTION_FAILURE: &str = "e2eb1e7e64f154690affccae198879cf9d115bd3";
const COMPILE_FAILURE: &str = "21a6e1beba956341802e5891b1e368dcdf619b9c";
const EVALUATION_FAILURE: &str = "3d358f915b9ce79f8a6e2336212aa5adbde48747";
const DIAGNOSTIC_REFUSAL: &str = "be2f3e69279e4e0f54cd939861f40799d9320d76";
const SOURCE_REFUSAL: &str = "297090ef71123af0e5523e43fbc67d9132c7fdfc";
const DEADLINE: &str = "3199f087184d10118a79454c39aca5a12cd87ae9";

fn fixture(revision: &str, name: &str, variant: &str) -> FixtureId {
    FixtureId::new(
        SourceRevision::new(revision).unwrap(),
        RelativePath::new(format!("qualification/{name}.js")).unwrap(),
        format!("/qualification/{name}.html{variant}"),
    )
    .unwrap()
}

fn invoke(fixture: &FixtureId, deadline: Duration) -> Run {
    let worker = PathBuf::from(env!("CARGO_BIN_EXE_surgeist-js-wpt-worker"));
    let limits = WorkerLimits::new(
        4_096,
        32_768,
        4_096,
        deadline,
        Duration::from_millis(100),
        Duration::from_secs(3),
        MemoryPolicy::Unbounded,
    )
    .unwrap();
    let run = run(
        fixture,
        worker,
        env::temp_dir(),
        Vec::new(),
        BTreeMap::new(),
        limits,
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(&run.fixture, fixture);
    assert_eq!(run.process.cleanup, CleanupStatus::Complete, "{run:?}");
    assert!(run.process.stdout.eof && run.process.stderr.eof, "{run:?}");
    assert_eq!(run.process_memory_policy, MemoryPolicy::Unbounded);
    assert!(!run.process_capabilities.hard_memory_limit);
    assert!(!run.process_capabilities.sampled_rss);
    assert_eq!(run.process.sampled_peak_rss_bytes, None);
    run
}

fn received(fixture: &FixtureId) -> WorkerResult {
    received_report(fixture).result
}

fn received_report(fixture: &FixtureId) -> WorkerReport {
    let run = invoke(fixture, Duration::from_secs(10));
    assert_eq!(run.process.stop, ProcessStop::Exited, "{run:?}");
    assert_eq!(run.process.exit, ProcessExit::Success, "{run:?}");
    match run.report {
        Report::Received(report) => report,
        _ => panic!("no qualified result: {run:?}"),
    }
}

fn assert_completion(fixture: &FixtureId, status: SubtestStatus) {
    let result = received(fixture);
    let WorkerResult::Harness(report) = result else {
        panic!("expected an assertion report: {result:?}");
    };
    assert_eq!(report.status, HarnessStatus::Ok);
    assert_eq!(report.subtests.len(), 1);
    assert_eq!(report.subtests[0].index, 0);
    assert_eq!(report.subtests[0].status, status);
}

#[test]
fn native_assertions_and_isolation() {
    if !execution_capabilities().process_groups {
        return;
    }
    let native = fixture(NATIVE_WINDOW, "native-window", "");
    assert_completion(&native, SubtestStatus::Pass);
    assert_completion(
        &fixture(ASSERTION_FAILURE, "assertion-failure", ""),
        SubtestStatus::Fail,
    );
    assert_completion(&native, SubtestStatus::Pass);
}

#[test]
fn guest_failures_preserve_phase_and_code_units() {
    if !execution_capabilities().process_groups {
        return;
    }
    let compile = received(&fixture(COMPILE_FAILURE, "compile-failure", ""));
    assert!(
        matches!(
            compile,
            WorkerResult::Failure(WorkerFailure::GuestSyntax(_))
        ),
        "{compile:?}"
    );

    let evaluation = received(&fixture(EVALUATION_FAILURE, "evaluation-failure", ""));
    assert_eq!(
        evaluation,
        WorkerResult::Failure(WorkerFailure::GuestRuntime(GuestDiagnostic {
            message: Some(vec![0xd800, 103, 117, 101, 115, 116, 0xdc00]),
            stack: None,
        }))
    );

    let oversized = received(&fixture(DIAGNOSTIC_REFUSAL, "diagnostic-refusal", ""));
    assert_eq!(
        oversized,
        WorkerResult::Failure(WorkerFailure::GuestRuntime(GuestDiagnostic {
            message: None,
            stack: None,
        }))
    );
}

#[test]
fn checks_the_whole_fixture_identity() {
    if !execution_capabilities().process_groups {
        return;
    }
    for invalid in [
        fixture(ASSERTION_FAILURE, "native-window", ""),
        FixtureId::new(
            SourceRevision::new(NATIVE_WINDOW).unwrap(),
            RelativePath::new("qualification/assertion-failure.js").unwrap(),
            "/qualification/native-window.html".into(),
        )
        .unwrap(),
        fixture(NATIVE_WINDOW, "native-window", "#different-variant"),
    ] {
        let report = received_report(&invalid);
        assert!(
            matches!(
                report.result,
                WorkerResult::Failure(WorkerFailure::UnavailableCapability { .. })
            ),
            "{report:?}"
        );
        assert_eq!(
            report.resources.native,
            NativeResourceEvidence::NotAttached {}
        );
        assert_eq!(
            report.resources.engine,
            EngineResourceEvidence::Unavailable {}
        );
    }
}

#[test]
fn pinned_window_variants_report_the_missing_native_environment() {
    if !execution_capabilities().process_groups {
        return;
    }
    let revision = "69f2bba61ce7e9469ccbce53de4644a89f6b7dff";
    let listener = "dom/events/EventTarget-addEventListener.any.js";
    let listener_window = "/dom/events/EventTarget-addEventListener.any.html";
    for (revision, path, url, expected) in [
        (
            revision,
            listener,
            listener_window,
            "native Window WPT environment",
        ),
        (
            revision,
            "dom/events/EventTarget-add-remove-listener.any.js",
            "/dom/events/EventTarget-add-remove-listener.any.html",
            "native Window WPT environment",
        ),
        (
            "e22da4aff850a2f0042c5ecda4c844cc1852f2ec",
            listener,
            listener_window,
            "fixture identity absent from worker inventory",
        ),
        (
            revision,
            "dom/events/EventTarget-add-remove-listener.any.js",
            listener_window,
            "fixture identity absent from worker inventory",
        ),
        (
            revision,
            listener,
            "/dom/events/EventTarget-addEventListener.any.worker.html",
            "fixture identity absent from worker inventory",
        ),
        (
            revision,
            listener,
            "/dom/events/EventTarget-addEventListener.any.html?other",
            "fixture identity absent from worker inventory",
        ),
        (
            revision,
            listener,
            "/dom/events/EventTarget-addEventListener.any.html#other",
            "fixture identity absent from worker inventory",
        ),
    ] {
        let fixture = FixtureId::new(
            SourceRevision::new(revision).unwrap(),
            RelativePath::new(path).unwrap(),
            url.into(),
        )
        .unwrap();
        let report = received_report(&fixture);
        assert_eq!(
            report.result,
            WorkerResult::Failure(WorkerFailure::UnavailableCapability {
                capability: expected.into(),
                diagnostic: GuestDiagnostic {
                    message: None,
                    stack: None
                },
            }),
            "{fixture:?}"
        );
        assert_eq!(
            report.resources.native,
            NativeResourceEvidence::NotAttached {}
        );
        assert_eq!(
            report.resources.engine,
            EngineResourceEvidence::Unavailable {}
        );
    }
}

#[test]
fn pinned_html_collection_fixture_completes_with_both_subtests_passing() {
    if !execution_capabilities().process_groups {
        return;
    }
    let fixture = FixtureId::new(
        SourceRevision::new("69f2bba61ce7e9469ccbce53de4644a89f6b7dff").unwrap(),
        RelativePath::new("dom/collections/HTMLCollection-as-prototype.html").unwrap(),
        "/dom/collections/HTMLCollection-as-prototype.html".into(),
    )
    .unwrap();

    let result = received(&fixture);
    let WorkerResult::Harness(completion) = result else {
        panic!("expected the pinned HTML fixture to complete: {result:?}");
    };
    assert_eq!(completion.status, HarnessStatus::Ok);
    assert_eq!(
        completion.diagnostic,
        GuestDiagnostic {
            message: None,
            stack: None,
        }
    );
    let subtests: Vec<_> = completion
        .subtests
        .iter()
        .map(|test| {
            (
                test.index,
                String::from_utf16(&test.name).unwrap(),
                test.status,
            )
        })
        .collect();
    assert_eq!(
        subtests,
        [
            (
                0,
                "HTMLCollection as a prototype should not allow getting .length on the base object"
                    .into(),
                SubtestStatus::Pass,
            ),
            (
                1,
                "HTMLCollection as a prototype and setting own properties".into(),
                SubtestStatus::Pass,
            ),
        ]
    );
}

#[test]
fn policy_refusal_and_worker_deadline_have_distinct_outcomes() {
    if !execution_capabilities().process_groups {
        return;
    }
    let refusal = received(&fixture(
        NATIVE_WINDOW,
        "native-window",
        "?scripting=disabled",
    ));
    assert!(
        matches!(
            &refusal,
            WorkerResult::Failure(WorkerFailure::NativeRefusal { owner, code, .. })
                if owner == "runtime" && code == "scripting-disabled"
        ),
        "{refusal:?}"
    );
    let run = invoke(&fixture(DEADLINE, "deadline", ""), Duration::from_secs(2));
    assert_eq!(run.process.stop, ProcessStop::Deadline, "{run:?}");
    assert!(matches!(run.report, Report::TransportFailed), "{run:?}");
}

#[test]
fn canonical_source_refusal_keeps_a_qualified_native_result() {
    if !execution_capabilities().process_groups {
        return;
    }
    let report = received_report(&fixture(SOURCE_REFUSAL, "source-refusal", ""));
    assert!(
        matches!(
            &report.result,
            WorkerResult::Failure(WorkerFailure::NativeRefusal { owner, code, .. })
                if owner == "runtime" && code == "source-work-limit"
        ),
        "{report:?}"
    );
    assert_eq!(
        report.resources.engine,
        EngineResourceEvidence::BoaCollector { sample: None }
    );
    let NativeResourceEvidence::Measured(usage) = &report.resources.native else {
        panic!("expected native accounting after refusal: {report:?}");
    };
    assert!(usage.adapter_work() > 0);
    assert_eq!(usage.live_backing_bytes(), 0);
    assert!(usage.peak_backing_bytes() > 0);
}

#[derive(Debug, Deserialize)]
struct ResourceFrame {
    resources: ResourceEvidence,
}

#[derive(Debug, Deserialize)]
struct ResourceEvidence {
    native: NativeEvidence,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
enum NativeEvidence {
    NotAttached,
    Measured {
        adapter_work: u64,
        live_backing_bytes: u64,
        peak_backing_bytes: u64,
    },
}

fn resource_evidence(fixture: &FixtureId) -> (NativeEvidence, EngineResourceEvidence) {
    let run = invoke(fixture, Duration::from_secs(10));
    assert_eq!(run.process.stop, ProcessStop::Exited, "{run:?}");
    assert_eq!(run.process.exit, ProcessExit::Success, "{run:?}");
    let frame: ResourceFrame = read_frame(&mut run.process.stdout.bytes.as_slice(), 32_764)
        .expect("the actual terminal frame carries native resource evidence")
        .expect("the worker wrote a terminal frame");
    let Report::Received(report) = run.report else {
        panic!("no qualified result: {run:?}");
    };
    (frame.resources.native, report.resources.engine)
}

#[test]
fn native_resource_evidence_reports_actual_work_and_teardown() {
    if !execution_capabilities().process_groups {
        return;
    }
    let (completed, engine) = resource_evidence(&fixture(NATIVE_WINDOW, "native-window", ""));
    let NativeEvidence::Measured {
        adapter_work,
        live_backing_bytes,
        peak_backing_bytes,
    } = completed
    else {
        panic!("expected native accounting after execution: {completed:?}");
    };
    assert!(adapter_work > 0);
    assert!(peak_backing_bytes > 0);
    assert_eq!(live_backing_bytes, 0);
    assert!(
        matches!(&engine, EngineResourceEvidence::BoaCollector { sample: Some(sample) }
            if sample.used_bytes > 0),
        "{engine:?}"
    );

    let (refused, engine) = resource_evidence(&fixture(
        NATIVE_WINDOW,
        "native-window",
        "?scripting=disabled",
    ));
    assert!(
        matches!(refused, NativeEvidence::NotAttached),
        "{refused:?}"
    );
    assert_eq!(
        engine,
        EngineResourceEvidence::BoaCollector { sample: None }
    );
}
