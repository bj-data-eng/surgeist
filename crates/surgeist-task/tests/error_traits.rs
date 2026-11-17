//! Every public error type implements `Display` with a semantic message and
//! `core::error::Error`, with a source only where it wraps another error.

use core::error::Error;

use surgeist_task::cancellation::CancelReason;
use surgeist_task::coordination::{TaskRegistration, TaskRegistrationError};
use surgeist_task::event::{TaskEvent, TaskEventSinkError};
use surgeist_task::executor::ExecutorError;
use surgeist_task::identity::{TaskAttemptId, TaskId, TaskIdError, TaskKey, TaskName};
use surgeist_task::job::TaskFailure;
use surgeist_task::lifecycle::{TaskLifecycleError, TaskRecord};
use surgeist_task::policy::TaskPolicy;
use surgeist_task::progress::{PercentComplete, PercentCompleteError};
use surgeist_task::queue::{QueueError, QueuePolicy, TaskEventQueue};
use surgeist_task::scope::{TaskScope, TaskScopeError, TaskScopeSegment};

fn assert_standard_error<E: Error + Send + Sync + 'static>() {}

#[test]
fn every_public_error_type_implements_the_standard_error_trait() {
    assert_standard_error::<TaskIdError>();
    assert_standard_error::<TaskScopeError>();
    assert_standard_error::<PercentCompleteError>();
    assert_standard_error::<QueueError>();
    assert_standard_error::<TaskEventSinkError>();
    assert_standard_error::<ExecutorError>();
    assert_standard_error::<TaskRegistrationError>();
    assert_standard_error::<TaskLifecycleError>();
    assert_standard_error::<TaskFailure>();
}

#[test]
fn task_id_errors_say_what_is_wrong_with_the_identifier() {
    let empty = TaskName::try_new("  ").unwrap_err();
    assert_eq!(
        empty.to_string(),
        "task identifier \"  \" is empty or only whitespace"
    );
    assert!(empty.source().is_none());

    let control = TaskKey::try_new("rows\ncount").unwrap_err();
    assert_eq!(
        control.to_string(),
        "task identifier \"rows\\ncount\" contains an ASCII control character"
    );
}

#[test]
fn task_scope_errors_say_which_segment_part_is_wrong() {
    let message = |namespace: &str, value: &str| {
        let error = TaskScopeSegment::try_new(namespace, value).unwrap_err();
        assert!(error.source().is_none());
        error.to_string()
    };

    assert_eq!(
        message(" ", "orders"),
        "task scope namespace \" \" is empty or only whitespace"
    );
    assert_eq!(
        message("workspace", ""),
        "task scope value \"\" for namespace \"workspace\" is empty or only whitespace"
    );
    assert_eq!(
        message("workspace", "a\u{7}b"),
        "task scope segment \"workspace\":\"a\\u{7}b\" contains an ASCII control character"
    );
    assert_eq!(
        message("ui", "panel"),
        "task scope namespace \"ui\" is reserved"
    );
}

#[test]
fn percent_complete_error_states_the_limit_in_basis_points() {
    let error = PercentComplete::try_from_basis_points(10_001).unwrap_err();

    assert_eq!(
        error.to_string(),
        "10001 basis points is more than 10000 basis points (100 percent)"
    );
    assert!(error.source().is_none());
}

#[test]
fn queue_overflow_names_the_full_capacity() {
    let mut queue = TaskEventQueue::<()>::new(QueuePolicy::bounded(1));
    let task = TaskId::from_u64(1);
    let attempt = TaskAttemptId::from_u64(1);
    queue.push(TaskEvent::started(task, attempt)).unwrap();

    let error = queue.push(TaskEvent::started(task, attempt)).unwrap_err();

    assert_eq!(error.to_string(), "task event queue capacity 1 is full");
    assert!(error.source().is_none());
}

#[test]
fn event_sink_errors_name_the_failure_and_keep_the_sink_message() {
    assert_eq!(
        TaskEventSinkError::closed("receiver dropped").to_string(),
        "task event sink is closed: receiver dropped"
    );
    assert_eq!(
        TaskEventSinkError::overflow("buffer full").to_string(),
        "task event sink overflowed: buffer full"
    );
    let error = TaskEventSinkError::serialization_failed("bad utf-8");
    assert_eq!(
        error.to_string(),
        "task event could not be serialized: bad utf-8"
    );
    assert!(error.source().is_none());
}

#[test]
fn executor_errors_name_the_failure_and_the_task_when_known() {
    assert_eq!(
        ExecutorError::invalid_request("missing key").to_string(),
        "invalid task spawn request: missing key"
    );
    assert_eq!(
        ExecutorError::spawn_failed("no threads")
            .with_task_id(TaskId::from_u64(7))
            .to_string(),
        "task could not be spawned: no threads (task 7)"
    );
    assert_eq!(
        ExecutorError::cancellation_failed("unknown handle")
            .with_attempt_id(TaskAttemptId::from_u64(2))
            .to_string(),
        "task could not be cancelled: unknown handle (attempt 2)"
    );
    assert_eq!(
        ExecutorError::join_failed("worker panicked")
            .with_task_attempt(TaskId::from_u64(7), TaskAttemptId::from_u64(2))
            .to_string(),
        "task could not be joined: worker panicked (task 7, attempt 2)"
    );
    let error = ExecutorError::sink_failed("closed");
    assert_eq!(error.to_string(), "task event sink failed: closed");
    assert!(error.source().is_none());
}

#[test]
fn task_failures_name_the_outcome_and_keep_the_job_message() {
    assert_eq!(
        TaskFailure::failed("disk full").to_string(),
        "task failed: disk full"
    );
    assert_eq!(
        TaskFailure::cancelled("stopped early").to_string(),
        "task cancelled: stopped early"
    );
    assert_eq!(
        TaskFailure::panicked("index out of range").to_string(),
        "task panicked: index out of range"
    );
    assert_eq!(
        TaskFailure::invalid_input("no rows").to_string(),
        "task input is invalid: no rows"
    );
    let failure = TaskFailure::sink_failed("closed");
    assert_eq!(failure.to_string(), "task event emission failed: closed");
    assert!(failure.source().is_none());
}

#[test]
fn registration_errors_wrap_the_identity_or_scope_error_that_caused_them() {
    let name = TaskRegistration::<()>::try_new("").map(|_| ()).unwrap_err();
    assert_eq!(name.to_string(), "invalid task name");
    let cause = name.source().expect("name error has a source");
    assert_eq!(
        cause.to_string(),
        "task identifier \"\" is empty or only whitespace"
    );

    let key_cause = TaskKey::try_new("").unwrap_err();
    let key = TaskRegistrationError::invalid_key(key_cause.clone());
    assert_eq!(key.to_string(), "invalid task key");
    assert_eq!(key.source().unwrap().to_string(), key_cause.to_string());

    let scope_cause = TaskScopeSegment::try_new("render", "x").unwrap_err();
    let scope = TaskRegistrationError::invalid_scope(scope_cause.clone());
    assert_eq!(scope.to_string(), "invalid task scope");
    assert_eq!(scope.source().unwrap().to_string(), scope_cause.to_string());
}

#[test]
fn registration_error_without_a_cause_reports_what_is_not_configured() {
    let registration = TaskRegistration::<()>::try_new("import").unwrap();

    let error = registration.key_for(&()).unwrap_err();

    assert_eq!(
        error.to_string(),
        "task key derivation has not been configured"
    );
    assert!(error.source().is_none());
}

#[test]
fn registration_errors_from_equal_causes_compare_equal() {
    let cause = TaskKey::try_new(" ").unwrap_err();
    let other = TaskKey::try_new("\t").unwrap_err();

    assert_eq!(
        TaskRegistrationError::invalid_key(cause.clone()),
        TaskRegistrationError::invalid_key(cause.clone())
    );
    assert_ne!(
        TaskRegistrationError::invalid_key(cause),
        TaskRegistrationError::invalid_key(other)
    );
}

fn running_record() -> (TaskRecord, TaskAttemptId) {
    let mut record = TaskRecord::queued(
        TaskId::from_u64(9),
        TaskKey::try_new("import:orders").unwrap(),
        TaskScope::app(),
        TaskPolicy::continue_when_unobserved(),
    );
    let attempt = TaskAttemptId::from_u64(1);
    record.start_attempt(attempt).unwrap();
    (record, attempt)
}

#[test]
fn lifecycle_errors_name_task_attempt_status_and_the_violated_rule() {
    let (mut stale, attempt) = running_record();
    stale.mark_running(attempt).unwrap();
    let error = stale.mark_running(TaskAttemptId::from_u64(2)).unwrap_err();
    assert_eq!(
        error.to_string(),
        "attempt 2 is not the active attempt of task 9 (status: running)"
    );
    assert!(error.source().is_none());

    let (mut terminal, attempt) = running_record();
    terminal.mark_running(attempt).unwrap();
    terminal.complete(attempt).unwrap();
    assert_eq!(
        terminal.mark_running(attempt).unwrap_err().to_string(),
        "task 9 has reached terminal status completed; attempt 1 cannot change it"
    );

    let (mut never_started, _) = running_record();
    assert_eq!(
        never_started
            .complete(TaskAttemptId::from_u64(1))
            .unwrap_err()
            .to_string(),
        "attempt 1 of task 9 is not running (status: queued)"
    );

    let (mut running, attempt) = running_record();
    running.mark_running(attempt).unwrap();
    assert_eq!(
        running
            .finish_after_cancel(attempt)
            .unwrap_err()
            .to_string(),
        "task 9 is not cancelling, so attempt 1 cannot finish after cancellation (status: running)"
    );

    let (mut repeated, attempt) = running_record();
    assert_eq!(
        repeated.start_attempt(attempt).unwrap_err().to_string(),
        "attempt 1 does not advance past the current attempt of task 9"
    );
}

#[test]
fn terminal_status_labels_read_as_words() {
    let (mut cancelled, attempt) = running_record();
    cancelled.mark_running(attempt).unwrap();
    cancelled.request_cancel(CancelReason::user_requested());
    cancelled.finish_after_cancel(attempt).unwrap();

    assert_eq!(
        cancelled.mark_running(attempt).unwrap_err().to_string(),
        "task 9 has reached terminal status finished after cancel; attempt 1 cannot change it"
    );
}
