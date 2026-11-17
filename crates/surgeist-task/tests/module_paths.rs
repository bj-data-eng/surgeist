//! Every public item is reachable through its domain module, and the root holds
//! only the modules and the crate identity constant.

#![allow(unused_imports)]

use surgeist_task::CRATE_NAME;
use surgeist_task::cancellation::{
    CancelReason, CancelReasonKind, CancellationToken, CancellationView,
};
use surgeist_task::coordination::{
    ObservationChange, ObserverCountSnapshot, TaskCoordination, TaskRegistration,
    TaskRegistrationError, TaskRegistrationErrorKind,
};
use surgeist_task::event::{
    TaskDiagnosticEvent, TaskDiagnosticLevel, TaskEvent, TaskEventKind, TaskEventSink,
    TaskEventSinkError, TaskEventSinkErrorKind, TaskJobEvent, TaskLifecycleEvent, TaskOutputEvent,
};
use surgeist_task::executor::{
    ExecutorDrainReport, ExecutorError, ExecutorErrorKind, ExecutorTaskHandle, TaskExecutor,
    TaskHandle, TaskSpawnRequest, TaskSpawnRequestBuilder, TokioTaskExecutor,
};
use surgeist_task::identity::{
    CoalescingKey, CorrelationId, ObserverId, ResourceClassId, TaskAttemptId, TaskId, TaskIdError,
    TaskIdErrorKind, TaskKey, TaskName, TaskProvenance,
};
use surgeist_task::job::{
    AsyncTaskJob, BlockingTaskJob, TaskContext, TaskFailure, TaskFailureKind, TaskRunnable,
};
use surgeist_task::lifecycle::{
    TaskLifecycleError, TaskLifecycleErrorKind, TaskRecord, TaskSnapshot, TaskStatus,
    TaskTerminalStatus,
};
use surgeist_task::policy::{
    BlockingPolicy, RetryPolicy, TaskPolicy, TaskPriority, UnobservedPolicy,
};
use surgeist_task::progress::{
    PercentComplete, PercentCompleteError, PercentCompleteErrorKind, TaskProgressEvent,
    TaskProgressValue,
};
use surgeist_task::queue::{
    DrainBudget, DrainReport, QueueError, QueueErrorCode, QueuePolicy, QueueReport, TaskEventQueue,
};
use surgeist_task::scope::{TaskScope, TaskScopeError, TaskScopeErrorKind, TaskScopeSegment};

#[test]
fn items_from_each_module_work_together() {
    assert_eq!(CRATE_NAME, "surgeist-task");

    let task = TaskId::from_u64(1);
    let attempt = TaskAttemptId::from_u64(1);
    let key = TaskKey::try_new("import:orders").unwrap();
    let scope = TaskScope::try_workspace("orders").unwrap();
    let mut record = TaskRecord::queued(task, key, scope, TaskPolicy::continue_when_unobserved());
    record.start_attempt(attempt).unwrap();
    record.mark_running(attempt).unwrap();
    assert_eq!(record.status(), TaskStatus::Running);

    let percent = PercentComplete::try_from_basis_points(2_500).unwrap();
    let progress =
        TaskJobEvent::<()>::progress_percent(CoalescingKey::try_new("rows").unwrap(), percent);
    let mut queue = TaskEventQueue::new(QueuePolicy::bounded(4));
    queue
        .push(TaskEvent::from_job_event(task, attempt, progress))
        .unwrap();
    let drained = queue.drain(DrainBudget::new());
    assert_eq!(drained.events().len(), 1);
    assert_eq!(
        drained.events()[0].progress_value(),
        Some(&TaskProgressValue::percent(percent))
    );

    let token = CancellationToken::new();
    token.cancel(CancelReason::user_requested());
    assert_eq!(
        token.view().reason().map(|reason| reason.kind()),
        Some(CancelReasonKind::UserRequested)
    );

    let mut coordination = TaskCoordination::default();
    let observed = TaskKey::try_new("search").unwrap();
    coordination.observe(observed.clone(), ObserverId::try_new("panel").unwrap());
    assert!(coordination.is_observed(&observed));

    assert!(TokioTaskExecutor::new().is_ok());
}
