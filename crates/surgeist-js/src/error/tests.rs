use std::path::Path;

use crate::{
    Context, JsError, JsNativeError, JsObject, Runtime, Source,
    builtins::promise::PromiseState,
    error::PanicError,
    js_string,
    module::Module,
    vm::{shadow_stack::ShadowEntry, source_info::SourcePath},
};
use indoc::indoc;

/// Helper to extract backtrace entries from a rejected module promise.
fn get_backtrace_from_rejection(
    context: &mut Context,
    js_code: &[u8],
    path: &str,
) -> Vec<ShadowEntry> {
    let source = Source::from_bytes(js_code).with_path(Path::new(path));
    let module = Module::parse(source, None, context.access()).unwrap();
    let promise = module.load_link_evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();

    match promise.state().unwrap() {
        PromiseState::Rejected(err) => {
            let js_error = JsError::from_opaque(err);
            js_error
                .backtrace
                .as_ref()
                .expect("error should have a backtrace")
                .iter()
                .cloned()
                .collect()
        }
        PromiseState::Fulfilled(_) => panic!("Module should have thrown an error"),
        PromiseState::Pending => panic!("Module evaluation should not be pending"),
    }
}

/// Assert that a `ShadowEntry::Bytecode` frame matches expected function name, path, line, and column.
#[track_caller]
fn assert_bytecode_frame(
    entry: &ShadowEntry,
    expected_fn: &str,
    expected_path: &Path,
    expected_line: u32,
    expected_col: u32,
) {
    match entry {
        ShadowEntry::Bytecode { pc, source_info } => {
            assert_eq!(
                source_info.function_name().to_std_string_escaped(),
                expected_fn,
                "function name mismatch"
            );
            assert_eq!(
                source_info.map().path(),
                &SourcePath::Path(expected_path.into()),
                "path mismatch"
            );
            let pos = source_info
                .map()
                .find(*pc)
                .expect("should have a source position");
            assert_eq!(pos.line_number(), expected_line, "line number mismatch");
            assert_eq!(pos.column_number(), expected_col, "column number mismatch");
        }
        ShadowEntry::Native { .. } => panic!("expected Bytecode frame, got Native"),
    }
}

/// Assert that a `ShadowEntry` is a `Native` frame.
#[track_caller]
fn assert_native_frame(entry: &ShadowEntry) {
    assert!(
        matches!(entry, ShadowEntry::Native { .. }),
        "expected Native frame, got Bytecode"
    );
}

/// Test that errors caught by internal handlers (e.g. async module evaluation)
/// preserve their backtrace through promise rejection (`JsError` -> `JsValue` -> `JsError`).
#[test]
fn backtrace_preserved_through_promise_rejection() {
    let mut context = Context::default();
    let entries = get_backtrace_from_rejection(
        &mut context,
        indoc! {br#"
            let x = undefined;
            x()
        "#},
        "test.js",
    );

    let path = Path::new("test.js");

    // Backtrace stored bottom-up: [Native (call site), Bytecode (<main>)]
    assert_eq!(entries.len(), 2, "expected 2 backtrace entries");
    assert_native_frame(&entries[0]);
    assert_bytecode_frame(&entries[1], "<main>", path, 2, 2);
}

/// Test that nested call frames produce a full backtrace through the
/// promise rejection round-trip.
#[test]
fn nested_backtrace_preserved_through_promise_rejection() {
    let mut context = Context::default();
    let entries = get_backtrace_from_rejection(
        &mut context,
        indoc! {br#"
            function foo() {
                function baz() {
                    import.meta.non_existent()
                }
                baz()
            }

            foo()
        "#},
        "test.js",
    );

    let path = Path::new("test.js");

    // Backtrace stored bottom-up: [Native, <main>, foo, baz]
    assert_eq!(entries.len(), 4, "expected 4 backtrace entries");
    assert_native_frame(&entries[0]);
    assert_bytecode_frame(&entries[1], "<main>", path, 8, 4);
    assert_bytecode_frame(&entries[2], "foo", path, 5, 8);
    assert_bytecode_frame(&entries[3], "baz", path, 3, 33);
}

/// Test that an explicit `throw new Error(...)` inside a module also preserves
/// the backtrace through the promise rejection round-trip.
#[test]
fn explicit_throw_backtrace_preserved_through_promise_rejection() {
    let mut context = Context::default();
    let entries = get_backtrace_from_rejection(
        &mut context,
        indoc! {br#"
            function foo() {
                throw new Error("test")
            }
            foo()
        "#},
        "test.js",
    );

    let path = Path::new("test.js");

    // Backtrace stored bottom-up: [Native, <main>, foo]
    assert_eq!(entries.len(), 3, "expected 3 backtrace entries");
    assert_native_frame(&entries[0]);
    assert_bytecode_frame(&entries[1], "<main>", path, 4, 4);
    assert_bytecode_frame(&entries[2], "foo", path, 2, 11);
}

/// Sanity check: `context.eval()` errors include a backtrace (relates to
/// <https://github.com/boa-dev/boa/discussions/4475>).
#[test]
fn eval_error_has_backtrace() {
    let mut context = Context::default();
    let code = indoc! {br#"
        const a = 0;
        iWillCauseAnError
        const b = a + 1;
    "#};
    let source = Source::from_reader(code.as_slice(), Some(Path::new("test.js")));
    match context.eval(source) {
        Ok(_) => panic!("Should have thrown a ReferenceError"),
        Err(e) => {
            assert!(e.backtrace.is_some(), "eval error should have a backtrace");
            let entries: Vec<_> = e.backtrace.as_ref().unwrap().iter().collect();
            assert!(
                !entries.is_empty(),
                "backtrace should have at least one entry"
            );
        }
    }
}

#[test]
fn materialization_refuses_a_foreign_opaque_error() {
    let context = &mut Context::default();
    let foreign = JsObject::with_null_proto(&Runtime::default());
    let error = JsError::from_opaque(foreign.into());

    let refusal = error.into_opaque(context.access()).unwrap_err();
    assert!(refusal.as_native().is_some_and(JsNativeError::is_type));
}

#[test]
fn materialization_refuses_a_foreign_cause_inside_an_aggregate() {
    let context = &mut Context::default();
    let foreign = JsObject::with_null_proto(&Runtime::default());
    let cause = JsError::from_opaque(foreign.into());
    let member = JsNativeError::range().with_cause(cause).into();
    let error = JsError::from_native(JsNativeError::aggregate(vec![member]));

    let refusal = error.into_opaque(context.access()).unwrap_err();
    assert!(refusal.as_native().is_some_and(JsNativeError::is_type));
}

#[test]
fn materialization_preserves_the_original_engine_error() {
    let context = &mut Context::default();
    let foreign = JsObject::with_null_proto(&Runtime::default());
    let cause = JsError::from_opaque(foreign.into());
    let error = JsError::from(PanicError::new("engine failure").with_source(cause));

    assert_eq!(error.clone().into_opaque(context.access()), Err(error));
}

#[test]
fn materialization_preserves_a_local_aggregate_members_cause() {
    let context = &mut Context::default();
    let cause = JsObject::with_null_proto(context.runtime());
    let member = JsNativeError::range()
        .with_cause(JsError::from_opaque(cause.clone().into()))
        .into();
    let error = JsError::from_native(JsNativeError::aggregate(vec![member]));

    let error = error
        .into_opaque(context.access())
        .unwrap()
        .as_object()
        .unwrap();
    let members = error.get(js_string!("errors"), context.access()).unwrap();
    let member = members
        .as_object()
        .unwrap()
        .get(0, context.access())
        .unwrap();
    let observed = member
        .as_object()
        .unwrap()
        .get(js_string!("cause"), context.access());
    assert_eq!(observed.unwrap(), cause.into());
}

#[test]
fn native_error_materialization_refuses_a_foreign_nested_cause() {
    let context = &mut Context::default();
    let foreign = JsObject::with_null_proto(&Runtime::default());
    let cause = JsError::from_opaque(foreign.into());
    let inner = JsNativeError::range().with_cause(cause);
    let error = JsNativeError::error().with_cause(inner);

    let refusal = error.into_opaque(context.access()).unwrap_err();
    assert!(refusal.as_native().is_some_and(JsNativeError::is_type));
}

#[test]
fn native_error_materialization_propagates_an_engine_error_in_an_aggregate() {
    let context = &mut Context::default();
    let engine_error = JsError::from(PanicError::new("engine failure"));
    let aggregate = JsNativeError::aggregate(vec![engine_error.clone()]);

    assert_eq!(
        aggregate.into_opaque(context.access()).unwrap_err(),
        engine_error
    );
}

#[test]
fn scope_state_failure_stays_a_host_error_across_storage_and_erasure() {
    use crate::error::{EngineError, ErasedEngineError};
    use surgeist_js_ast::scope::ScopeStateError;

    for failure in [
        ScopeStateError::Conflict,
        ScopeStateError::IdentityExhausted,
        ScopeStateError::Allocation,
    ] {
        let error = JsError::from(surgeist_js_parser::Error::ScopeState(failure));
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::ScopeAnalysis(failure))
        );
        assert!(error.as_native().is_none());
        let restored = error.clone().into_stored().to_error().unwrap();
        assert_eq!(restored, error);
        let context = &mut Context::default();
        assert_eq!(restored.clone().into_opaque(context.access()), Err(error));
        assert_eq!(
            restored.into_erased(context.access()).as_engine(),
            Some(&ErasedEngineError::ScopeAnalysis(failure))
        );
    }
}

#[test]
fn prepared_scope_errors_preserve_their_failure_category() {
    use crate::error::{EngineError, RuntimeLimitError};
    use surgeist_js_ast::scope::{ScopeAnalysisError, ScopeStateError};
    use surgeist_js_parser::frontend::FrontendLimit;

    let syntax = JsError::from(ScopeAnalysisError::<FrontendLimit>::Syntax(
        "duplicate lexical declaration".into(),
    ));
    assert!(syntax.as_native().is_some_and(JsNativeError::is_syntax));
    assert!(
        syntax
            .as_native()
            .unwrap()
            .message()
            .contains("duplicate lexical declaration")
    );

    let refused = JsError::from(ScopeAnalysisError::Refused(FrontendLimit::Scope(17)));
    assert_eq!(
        refused.as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::ScopeWork {
            limit: 17
        }))
    );

    let changed = JsError::from(ScopeAnalysisError::<FrontendLimit>::State(
        ScopeStateError::Conflict,
    ));
    assert_eq!(
        changed.as_engine(),
        Some(&EngineError::ScopeAnalysis(ScopeStateError::Conflict))
    );
}

#[test]
fn invocation_authority_failures_remain_uncatchable_host_errors() {
    use crate::error::{EngineError, ErasedEngineError};
    for (failure, erased) in [
        (
            EngineError::InvocationIdExhausted,
            ErasedEngineError::InvocationIdExhausted,
        ),
        (
            EngineError::InvalidEntryPermit,
            ErasedEngineError::InvalidEntryPermit,
        ),
    ] {
        let error = JsError::from(failure);
        assert!(!error.is_catchable());
        assert_eq!(error.clone().into_stored().to_error().unwrap(), error);
        let context = &mut Context::default();
        assert_eq!(
            error.clone().into_opaque(context.access()),
            Err(error.clone())
        );
        assert_eq!(
            error.into_erased(context.access()).as_engine(),
            Some(&erased)
        );
    }
}

#[test]
fn context_access_busy_remains_uncatchable_through_storage_and_erasure() {
    use crate::error::{EngineError, ErasedEngineError};

    let error = JsError::from(EngineError::ContextAccessBusy);
    assert!(!error.is_catchable());
    assert_eq!(error.clone().into_stored().to_error().unwrap(), error);
    let context = &mut Context::default();
    assert_eq!(
        error.clone().into_opaque(context.access()),
        Err(error.clone())
    );
    assert_eq!(
        error.into_erased(context.access()).as_engine(),
        Some(&ErasedEngineError::ContextAccessBusy)
    );
}
