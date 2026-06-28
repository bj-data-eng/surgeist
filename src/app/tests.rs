use super::*;
use crate as surgeist;

#[test]
fn typed_ids_are_stable_and_debuggable() {
    assert_eq!(AppId::new("photo.lab").as_str(), "photo.lab");
    assert_eq!(SurfaceId::from_u64(7).as_u64(), 7);
    assert_eq!(TaskAttemptId::from_u64(3).as_u64(), 3);
    assert_eq!(CorrelationId::from_u64(11).as_u64(), 11);
    assert_eq!(
        format!("{:?}", ResourceId::new("thumbs:42")),
        "ResourceId(\"thumbs:42\")"
    );
}

#[test]
fn provenance_carries_causal_fields() {
    let parent = CorrelationId::from_u64(1);
    let child = InputProvenance::task(TaskId::from_u64(2), TaskAttemptId::from_u64(3))
        .with_surface(SurfaceId::from_u64(4))
        .with_correlation(CorrelationId::from_u64(5))
        .with_parent(parent);

    assert_eq!(child.source(), &InputSourceId::TASK);
    assert!(matches!(child.origin(), InputOrigin::Task(_)));
    assert_eq!(child.task_id(), Some(TaskId::from_u64(2)));
    assert_eq!(child.task_attempt_id(), Some(TaskAttemptId::from_u64(3)));
    assert_eq!(child.surface_id(), Some(SurfaceId::from_u64(4)));
    assert_eq!(child.correlation_id(), CorrelationId::from_u64(5));
    assert_eq!(child.parent_correlation_id(), Some(parent));
}

#[test]
fn diagnostics_keep_recent_entries_and_counters() {
    let mut log = DiagnosticLog::with_capacity(2);
    log.push(Diagnostic::warning(
        DiagnosticCode::UNKNOWN_RETAINED_COMMAND,
        "missing binding",
        InputProvenance::ui(SurfaceId::from_u64(1)),
    ));
    log.push(
        Diagnostic::error(
            DiagnosticCode::STALE_TASK_EVENT,
            "attempt mismatch",
            InputProvenance::task(TaskId::from_u64(2), TaskAttemptId::from_u64(1)),
        )
        .with_app(AppId::new("photo.lab"))
        .with_window(surgeist::window::Id::from_u64(9))
        .with_root(RootId::new("gallery"))
        .with_scope(AppScope::resource(ResourceId::new("thumbs")))
        .with_resource(ResourceId::new("thumbs"))
        .with_queue(QueueDiagnostic::new("task-events", 128).with_age_ms(17))
        .with_effect("request_redraw"),
    );
    log.push(Diagnostic::info(
        DiagnosticCode::QUEUE_COALESCED,
        "progress coalesced",
        InputProvenance::system(),
    ));

    let entries = log.entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(log.dropped_oldest(), 1);
    assert_eq!(log.count(&DiagnosticCode::UNKNOWN_RETAINED_COMMAND), 1);
    assert_eq!(log.count(&DiagnosticCode::QUEUE_COALESCED), 1);
    assert_eq!(entries[0].code(), &DiagnosticCode::STALE_TASK_EVENT);
    assert_eq!(entries[0].app_id(), Some(&AppId::new("photo.lab")));
    assert_eq!(
        entries[0].window_id(),
        Some(surgeist::window::Id::from_u64(9))
    );
    assert_eq!(entries[0].root_id(), Some(&RootId::new("gallery")));
    assert_eq!(entries[0].resource_id(), Some(&ResourceId::new("thumbs")));
    assert_eq!(entries[0].emitted_effects(), &["request_redraw"]);
    assert_eq!(entries[0].queue().unwrap().capacity(), 128);
    assert_eq!(entries[0].queue().unwrap().age_ms(), Some(17));
}

#[test]
fn zero_capacity_diagnostic_log_counts_without_retaining_entries() {
    let mut log = DiagnosticLog::with_capacity(0);
    log.push(Diagnostic::warning(
        DiagnosticCode::QUEUE_OVERFLOW,
        "queue disabled",
        InputProvenance::system(),
    ));

    assert!(log.entries().is_empty());
    assert_eq!(log.dropped_oldest(), 1);
    assert_eq!(log.count(&DiagnosticCode::QUEUE_OVERFLOW), 1);
}

#[derive(Default)]
struct CounterState {
    value: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CounterInput {
    Increment,
    Save,
}

struct CounterReducer;

impl Reducer<CounterState, CounterInput> for CounterReducer {
    fn reduce(&mut self, state: &mut CounterState, input: AppInput<CounterInput>) -> ReducerResult {
        match input.payload() {
            CounterInput::Increment => {
                state.value += 1;
                ReducerResult::changed().with_effect(AppEffect::request_redraw(
                    RedrawTarget::surface(SurfaceId::from_u64(1)),
                ))
            }
            CounterInput::Save => ReducerResult::unchanged()
                .with_effect(AppEffect::persist("counter", AppScope::app())),
        }
    }
}

#[test]
fn reducer_returns_effects_without_executing_them() {
    let mut reducer = CounterReducer;
    let mut state = CounterState::default();
    let result = reducer.reduce(
        &mut state,
        AppInput::new(CounterInput::Increment, InputProvenance::system()),
    );

    assert_eq!(state.value, 1);
    assert!(result.is_changed());
    assert_eq!(result.effects().len(), 1);
    assert_eq!(result.effects()[0].kind(), &EffectKindId::REQUEST_REDRAW);

    let result = reducer.reduce(
        &mut state,
        AppInput::new(CounterInput::Save, InputProvenance::system()),
    );

    assert_eq!(state.value, 1);
    assert!(!result.is_changed());
    assert_eq!(result.effects().len(), 1);
    assert_eq!(result.effects()[0].kind(), &EffectKindId::PERSIST);
}

#[test]
fn effect_batches_preserve_order() {
    let effects = EffectBatch::new()
        .push(AppEffect::diagnostic(Diagnostic::info(
            DiagnosticCode::QUEUE_COALESCED,
            "coalesced",
            InputProvenance::system(),
        )))
        .push(AppEffect::request_redraw(RedrawTarget::all()));

    assert_eq!(effects.effects().len(), 2);
    assert_eq!(effects.effects()[0].kind(), &EffectKindId::EMIT_DIAGNOSTIC);
    assert_eq!(effects.effects()[1].kind(), &EffectKindId::REQUEST_REDRAW);
}

#[test]
fn resource_effects_expose_typed_payloads_and_kinds() {
    let load = AppEffect::load_resource(ResourceId::new("thumb:1"), AppScope::app());
    assert_eq!(load.kind(), &EffectKindId::LOAD_RESOURCE);
    assert!(matches!(
        load.payload(),
        AppEffectPayload::LoadResource(effect)
            if effect.id() == &ResourceId::new("thumb:1") && effect.scope() == &AppScope::app()
    ));

    let invalidate = AppEffect::invalidate_resource(ResourceId::new("thumb:1"), "source changed");
    assert_eq!(invalidate.kind(), &EffectKindId::INVALIDATE_RESOURCE);
    assert!(matches!(
        invalidate.payload(),
        AppEffectPayload::InvalidateResource(effect)
            if effect.id() == &ResourceId::new("thumb:1") && effect.reason() == "source changed"
    ));
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SearchInput {
    query: String,
}

#[test]
fn task_registry_records_identity_scope_key_and_policy() {
    let registration = TaskRegistration::<SearchInput>::new("search")
        .scope(|_| AppScope::resource(ResourceId::new("search-results")))
        .key(|input| TaskKey::new(format!("search:{}", input.query)))
        .with_policy(TaskPolicy::continue_when_unobserved().dedupe_by_key());

    let input = SearchInput {
        query: "rust".into(),
    };
    assert_eq!(registration.id().as_str(), "search");
    assert_eq!(
        registration.scope_for(&input),
        AppScope::resource(ResourceId::new("search-results"))
    );
    assert_eq!(registration.key_for(&input), TaskKey::new("search:rust"));
    assert!(registration.policy().dedupes_by_key());
    assert_eq!(
        registration.policy().unobserved(),
        UnobservedPolicy::Continue
    );
}

#[test]
fn task_record_rejects_events_from_stale_attempts() {
    let mut record = TaskRecord::queued(
        TaskId::from_u64(1),
        TaskKey::new("search:rust"),
        AppScope::app(),
        TaskPolicy::cancel_when_unobserved(),
    );

    let first = record.start_attempt(TaskAttemptId::from_u64(1));
    assert_eq!(first, TaskAttemptId::from_u64(1));
    record.mark_running();
    record.start_attempt(TaskAttemptId::from_u64(2));

    assert!(record.accepts_attempt(TaskAttemptId::from_u64(2)));
    assert!(!record.accepts_attempt(TaskAttemptId::from_u64(1)));
    assert_eq!(
        record.reject_stale(TaskAttemptId::from_u64(1)).code(),
        &DiagnosticCode::STALE_TASK_EVENT
    );
}

#[test]
fn cancellation_status_is_honest_until_terminal_event_arrives() {
    let mut record = TaskRecord::queued(
        TaskId::from_u64(2),
        TaskKey::new("media:import"),
        AppScope::app(),
        TaskPolicy::continue_when_unobserved(),
    );

    record.start_attempt(TaskAttemptId::from_u64(1));
    record.mark_running();
    let token = record.request_cancel();

    assert!(token.is_cancelled());
    assert_eq!(record.status(), TaskStatus::Cancelling);

    record.mark_finished_after_cancel();
    assert_eq!(record.status(), TaskStatus::FinishedAfterCancel);
}

#[test]
fn resource_state_tracks_freshness_and_refreshing_independently() {
    let mut resource = ResourceState::<u32, String>::idle(ResourceId::new("thumb:1"));

    resource.starting();
    assert_eq!(resource.status(), ResourceStatus::Starting);
    assert!(!resource.is_renderable());

    resource.ready(7, Freshness::Fresh);
    assert_eq!(resource.status(), ResourceStatus::Ready);
    assert_eq!(resource.value(), Some(&7));
    assert!(resource.is_renderable());
    assert_eq!(resource.freshness(), Freshness::Fresh);

    resource.refreshing();
    assert_eq!(resource.status(), ResourceStatus::Refreshing);
    assert_eq!(resource.value(), Some(&7));
    assert!(resource.is_renderable());

    resource.mark_stale("source changed");
    assert_eq!(resource.freshness(), Freshness::Stale);
    assert_eq!(resource.stale_reason(), Some("source changed"));
}

#[test]
fn resource_failure_preserves_renderable_stale_value() {
    let mut resource =
        ResourceState::<u32, String>::ready(ResourceId::new("query:1"), 10, Freshness::Fresh);

    resource.refreshing();
    resource.failed("timeout".to_string(), FailureVisibility::KeepStaleValue);

    assert_eq!(resource.status(), ResourceStatus::Failed);
    assert_eq!(resource.value(), Some(&10));
    assert_eq!(resource.error(), Some(&"timeout".to_string()));
    assert!(resource.is_renderable());
}
