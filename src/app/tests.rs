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
