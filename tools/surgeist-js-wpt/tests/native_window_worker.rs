//! Process-level qualification of the actual Runtime-owned Window worker.

use std::{collections::BTreeMap, env, path::PathBuf, sync::atomic::AtomicBool, time::Duration};
use surgeist_js_wpt::{
    CleanupStatus, FixtureId, GuestDiagnostic, HarnessStatus, MemoryPolicy, ProcessExit,
    ProcessStop, RelativePath, Report, Run, SourceRevision, SubtestStatus, WorkerFailure,
    WorkerLimits, WorkerResult, execution_capabilities, run,
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
    run
}

fn received(fixture: &FixtureId) -> WorkerResult {
    let run = invoke(fixture, Duration::from_secs(10));
    assert_eq!(run.process.stop, ProcessStop::Exited, "{run:?}");
    assert_eq!(run.process.exit, ProcessExit::Success, "{run:?}");
    match run.report {
        Report::Received(result) => result,
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
        fixture(NATIVE_WINDOW, "assertion-failure", ""),
        fixture(NATIVE_WINDOW, "native-window", "#different-variant"),
    ] {
        let result = received(&invalid);
        assert!(
            matches!(
                result,
                WorkerResult::Failure(WorkerFailure::UnavailableCapability { .. })
            ),
            "{result:?}"
        );
    }
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
    let result = received(&fixture(SOURCE_REFUSAL, "source-refusal", ""));
    assert!(
        matches!(
            &result,
            WorkerResult::Failure(WorkerFailure::NativeRefusal { owner, code, .. })
                if owner == "runtime" && code == "source-work-limit"
        ),
        "{result:?}"
    );
}
