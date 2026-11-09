#![allow(unused_crate_dependencies)]

use std::{cell::RefCell, path::Path};

use surgeist_js::{
    Context, JsError, JsNativeError, JsObject, JsValue, Module, NativeFunction, Source,
    builtins::promise::PromiseState,
    error::{
        Diagnostic, DiagnosticEngine, DiagnosticFrame, DiagnosticKind, DiagnosticNative,
        DiagnosticNativeKind, DiagnosticStack, EngineError, EngineResourceError, ErasedEngineError,
        JobError, JsErasedError, PanicError, RuntimeLimitError,
    },
    execution::ExecutionOptions,
    job::JobRetirement,
    js_string,
    vm::SourcePath,
};
use surgeist_js_ast::scope::ScopeStateError;
use surgeist_js_parser::frontend::InternerLimitKind;

thread_local! {
    static CALLBACK_ERROR: RefCell<Option<JsError>> = const { RefCell::new(None) };
}

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .register_global_builtin_callable(js_string!("stopModule"), 0, callback())
        .unwrap();
    context
}

fn callback() -> NativeFunction {
    NativeFunction::from_fn_ptr(|_, _, _| {
        Err(CALLBACK_ERROR.with(|slot| {
            slot.borrow_mut()
                .take()
                .expect("one authored callback error")
        }))
    })
}

fn stage(error: JsError) {
    CALLBACK_ERROR.with(|slot| assert!(slot.replace(Some(error)).is_none()));
}

fn observe(module: &Module, context: &mut Context) -> JsError {
    let outcome = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            module.evaluate(access)
        })
        .unwrap();
    let error = outcome
        .result
        .expect_err("terminal evaluation refuses native reentry");
    assert_eq!(
        outcome.report.instructions().unwrap(),
        0,
        "observation must not replay or inherit old counters"
    );
    error
}

fn interrupt(original: JsError, context: &mut Context) -> (Module, JsError) {
    context
        .global_object()
        .set(js_string!("moduleEffects"), 0, true, context.access())
        .unwrap();
    let module = Module::parse(
        Source::from_bytes(
            "globalThis.moduleEffects += 1; stopModule(); globalThis.moduleEffects += 100;",
        )
        .with_path(Path::new("typed-module-stop.js")),
        None,
        context.access(),
    )
    .unwrap();
    let loaded = module.load(context.access()).unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    module.link(context.access()).unwrap();
    stage(original.clone());
    let first = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            let result = module.evaluate(access);
            assert_eq!(
                result.as_ref().unwrap_err(),
                &original,
                "the initiating entry preserves the actual authored error"
            );
            result
        })
        .unwrap();
    assert_eq!(first.result.unwrap_err(), original);
    assert!(first.report.instructions().unwrap() > 0);
    assert_eq!(effects(context), 1.into());
    let terminal = observe(&module, context);
    assert_eq!(effects(context), 1.into());
    (module, terminal)
}

fn effects(context: &mut Context) -> JsValue {
    context
        .global_object()
        .get(js_string!("moduleEffects"), context.access())
        .unwrap()
}

fn cause(error: &JsError) -> Diagnostic<'_> {
    let Some(EngineError::ModuleInterrupted(interruption)) = error.as_engine() else {
        panic!("later native observation requires the distinct interruption variant: {error:?}")
    };
    interruption.cause()
}

fn native(view: Diagnostic<'_>) -> DiagnosticNative<'_> {
    let DiagnosticKind::Native(native) = view.kind() else {
        panic!("authored native topology was lost")
    };
    native
}

fn assert_same_identity(left: &JsError, right: &JsError) {
    let Some(EngineError::ModuleInterrupted(left)) = left.as_engine() else {
        panic!("left interruption")
    };
    let Some(EngineError::ModuleInterrupted(right)) = right.as_engine() else {
        panic!("right interruption")
    };
    assert_eq!(left.identity(), right.identity());
    assert_eq!(left, right);
}

fn assert_engine_cause(view: Diagnostic<'_>, expected: &EngineError) {
    let DiagnosticKind::Engine(actual) = view.kind() else {
        panic!("the retained cause stays engine-typed")
    };
    let expected = match expected {
        EngineError::Resource(value) => DiagnosticEngine::Resource(*value),
        EngineError::InvalidEntryPermit => DiagnosticEngine::InvalidEntryPermit,
        EngineError::ContextAccessBusy => DiagnosticEngine::ContextAccessBusy,
        EngineError::ObjectAccessBusy => DiagnosticEngine::ObjectAccessBusy,
        EngineError::Job(value) => DiagnosticEngine::Job(*value),
        EngineError::JobContextRequired { required } => {
            DiagnosticEngine::JobContextRequired { required }
        }
        EngineError::JobRetired(value) => DiagnosticEngine::JobRetired(*value),
        EngineError::InvocationIdExhausted => DiagnosticEngine::InvocationIdExhausted,
        EngineError::ScopeAnalysis(value) => DiagnosticEngine::ScopeAnalysis(*value),
        EngineError::RuntimeLimit(value) => DiagnosticEngine::RuntimeLimit(*value),
        #[cfg(feature = "fuzz")]
        EngineError::NoInstructionsRemain => DiagnosticEngine::NoInstructionsRemain,
        EngineError::Panic(_) | EngineError::ModuleInterrupted(_) => {
            panic!("topology cases have separate borrowed assertions")
        }
    };
    assert_eq!(actual, expected);
}

fn scalar_cases(context: &Context) -> Vec<EngineError> {
    let mut cases = vec![
        EngineError::Resource(EngineResourceError::SizeOverflow),
        EngineError::Resource(EngineResourceError::Allocation),
        EngineError::InvalidEntryPermit,
        EngineError::ContextAccessBusy,
        EngineError::ObjectAccessBusy,
        EngineError::Job(JobError::TaskCompleted),
        EngineError::Job(JobError::CancellationRevoked),
        EngineError::JobContextRequired {
            required: context.id(),
        },
        EngineError::InvocationIdExhausted,
    ];
    cases.extend(
        [
            JobRetirement::Cancelled,
            JobRetirement::OwnerDropped,
            JobRetirement::Interrupted,
            JobRetirement::Dropped,
        ]
        .map(EngineError::JobRetired),
    );
    cases.extend(
        [
            ScopeStateError::Conflict,
            ScopeStateError::IdentityExhausted,
            ScopeStateError::Allocation,
        ]
        .map(EngineError::ScopeAnalysis),
    );
    let mut limits = vec![
        RuntimeLimitError::CompilerWork { limit: 11 },
        RuntimeLimitError::CompilerDepth { limit: 3 },
        RuntimeLimitError::OptimizerWork { limit: 12 },
        RuntimeLimitError::ScopeWork { limit: 13 },
        RuntimeLimitError::SourceRead { limit: 14 },
        RuntimeLimitError::SourceConsumption { limit: 15 },
        RuntimeLimitError::SourceRetention { limit: 16 },
        RuntimeLimitError::LexerLookahead { limit: 17 },
        RuntimeLimitError::LexerDispatch { limit: 18 },
        RuntimeLimitError::TokenProbe { limit: 19 },
        RuntimeLimitError::TokenAdvance { limit: 20 },
        RuntimeLimitError::GrammarDispatch { limit: 21 },
        RuntimeLimitError::EarlyErrorVisit { limit: 22 },
        RuntimeLimitError::LabelLookup { limit: 23 },
        RuntimeLimitError::DirectEvalVisit { limit: 24 },
        RuntimeLimitError::NameVisit { limit: 25 },
        RuntimeLimitError::NameRecord { limit: 26 },
        RuntimeLimitError::NameComparison { limit: 27 },
        RuntimeLimitError::DiagnosticInput { limit: 28 },
        RuntimeLimitError::DiagnosticOutput { limit: 29 },
        RuntimeLimitError::TemplateInput { limit: 30 },
        RuntimeLimitError::TemplateOutput { limit: 31 },
        RuntimeLimitError::Instruction { limit: 32 },
        RuntimeLimitError::JobPreparation { limit: 33 },
        RuntimeLimitError::JobInvocation { limit: 34 },
        RuntimeLimitError::JobPoll { limit: 35 },
        RuntimeLimitError::LoopIteration,
        RuntimeLimitError::Recursion,
        RuntimeLimitError::StackSize,
    ];
    limits.extend(
        [
            InternerLimitKind::Bytes,
            InternerLimitKind::Utf16,
            InternerLimitKind::Index,
            InternerLimitKind::Capacity,
            InternerLimitKind::Relocation,
        ]
        .map(|kind| RuntimeLimitError::Interner { kind, limit: 36 }),
    );
    cases.extend(limits.into_iter().map(EngineError::RuntimeLimit));
    #[cfg(feature = "fuzz")]
    cases.push(EngineError::NoInstructionsRemain);
    cases
}

#[test]
fn actual_callback_stops_preserve_every_scalar_engine_payload_identity_and_erased_cause() {
    let owner = context();
    for engine in scalar_cases(&owner) {
        let mut context = context();
        let original = JsError::from(engine.clone());
        let (module, first) = interrupt(original.clone(), &mut context);
        assert_engine_cause(cause(&first), &engine);
        let repeated = observe(&module, &mut context);
        assert_same_identity(&first, &repeated);
        assert_same_identity(&first, &first.clone());
        assert_engine_cause(cause(&repeated), &engine);
        let Some(EngineError::ModuleInterrupted(handle)) = first.as_engine() else {
            unreachable!()
        };
        let expected = original.into_erased(context.access());
        let erased = first.clone().into_erased(context.access());
        let Some(ErasedEngineError::ModuleInterrupted { identity, cause }) = erased.as_engine()
        else {
            panic!("erased interruption stays typed")
        };
        assert_eq!(*identity, handle.identity());
        assert_eq!(cause.as_ref(), &expected);
        if let EngineError::ObjectAccessBusy = engine {
            assert_eq!(
                cause.as_engine(),
                Some(&ErasedEngineError::ObjectAccessBusy)
            );
        }
        if let EngineError::Job(error) = engine {
            assert_eq!(cause.as_engine(), Some(&ErasedEngineError::Job(error)));
        }
        assert_eq!(effects(&mut context), 1.into());
    }
    fn send_sync<T: Send + Sync>() {}
    send_sync::<JsErasedError>();
}

#[test]
fn equal_original_stops_from_distinct_module_instances_have_distinct_terminal_identities() {
    let mut context = context();
    let original = JsError::from(EngineError::InvalidEntryPermit);
    let (left_module, left) = interrupt(original.clone(), &mut context);
    let (right_module, right) = interrupt(original, &mut context);
    let (
        Some(EngineError::ModuleInterrupted(left_handle)),
        Some(EngineError::ModuleInterrupted(right_handle)),
    ) = (left.as_engine(), right.as_engine())
    else {
        unreachable!()
    };
    assert_ne!(left_handle.identity(), right_handle.identity());
    assert_ne!(left, right);
    assert_eq!(cause(&left), cause(&right));
    assert_same_identity(&left, &observe(&left_module, &mut context));
    assert_same_identity(&right, &observe(&right_module, &mut context));
}

#[test]
fn a_current_operations_first_stop_precedes_a_historical_module_cause() {
    let mut context = context();
    let original = EngineError::Resource(EngineResourceError::SizeOverflow);
    let (module, terminal) = interrupt(JsError::from(original.clone()), &mut context);
    let current = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            let prior = access.eval(Source::from_bytes("0")).unwrap_err();
            assert_eq!(
                prior.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            let refused = module.evaluate(access).unwrap_err();
            assert_eq!(refused, prior);
            Err::<JsValue, _>(refused)
        })
        .unwrap();
    assert_eq!(
        current.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(current.report.instructions().unwrap(), 0);
    let fresh = observe(&module, &mut context);
    assert_same_identity(&terminal, &fresh);
    assert_engine_cause(cause(&fresh), &original);
    assert_eq!(effects(&mut context), 1.into());
}

#[test]
fn an_incoming_interruption_reuses_its_original_cause_in_another_real_module() {
    let mut context = context();
    let original = EngineError::RuntimeLimit(RuntimeLimitError::JobPoll { limit: 47 });
    let (_first_module, first) = interrupt(JsError::from(original.clone()), &mut context);
    let (second_module, second) = interrupt(first.clone(), &mut context);
    assert_same_identity(&first, &second);
    assert_engine_cause(cause(&second), &original);
    assert_same_identity(&first, &observe(&second_module, &mut context));
}

#[test]
fn nested_panic_causes_keep_native_order_object_identity_and_borrowed_provenance() {
    let mut context = context();
    let object = JsObject::with_null_proto(context.runtime());
    let identity = object.identity();
    let captured = context
        .eval_with_budget(
            Source::from_bytes("function causeOrigin() { missingCauseName(); } causeOrigin();")
                .with_path(Path::new("original-cause.js")),
            ExecutionOptions::new(100_000),
        )
        .unwrap()
        .result
        .unwrap_err()
        .try_native(context.access())
        .unwrap();
    let aggregate =
        JsNativeError::aggregate(vec![JsError::from_opaque(object.into()), captured.into()])
            .with_message("authored members")
            .with_cause(JsError::from_opaque(
                js_string!("authored native cause").into(),
            ));
    let original: JsError = PanicError::new("original outer panic")
        .with_source(PanicError::new("original inner panic").with_source(aggregate))
        .into();
    let (module, terminal) = interrupt(original.clone(), &mut context);
    let root = cause(&terminal);
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        message,
        diagnostic: Some(inner),
    }) = root.kind()
    else {
        panic!("original outer Panic stays typed")
    };
    assert_eq!(message, "original outer panic");
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        message,
        diagnostic: Some(aggregate),
    }) = inner.kind()
    else {
        panic!("original nested Panic stays borrowed")
    };
    assert_eq!(message, "original inner panic");
    let aggregate = native(aggregate);
    assert_eq!(aggregate.kind(), DiagnosticNativeKind::Aggregate);
    assert_eq!(aggregate.message(), "authored members");
    let DiagnosticKind::Opaque(native_cause) = aggregate.cause().unwrap().kind() else {
        unreachable!()
    };
    assert_eq!(
        native_cause.value().unwrap(),
        js_string!("authored native cause").into()
    );
    let mut members = aggregate.members();
    assert_eq!(members.len(), 2);
    let DiagnosticKind::Opaque(object) = members.next().unwrap().kind() else {
        unreachable!()
    };
    assert_eq!(object.object_identity(), Some(identity));
    assert_eq!(
        object.value().unwrap().as_object().unwrap().identity(),
        identity
    );
    let captured = native(members.next_back().unwrap());
    assert_eq!(captured.kind(), DiagnosticNativeKind::Reference);
    let DiagnosticStack::Backtrace(trace) = captured.stack() else {
        panic!("the authored failure retains its actual captured stack")
    };
    assert!(trace.frames().any(|frame| matches!(frame, DiagnosticFrame::Bytecode { function_name, path: SourcePath::Path(path), .. } if function_name == &js_string!("causeOrigin") && path.as_ref() == Path::new("original-cause.js"))));
    assert!(members.next().is_none());
    assert!(root.backtrace().unwrap().frames().any(|frame| matches!(frame, DiagnosticFrame::Bytecode { path: SourcePath::Path(path), .. } if path.as_ref() == Path::new("typed-module-stop.js"))));
    assert_same_identity(&terminal, &observe(&module, &mut context));
    let expected = original.into_erased(context.access());
    let erased = terminal.clone().into_erased(context.access());
    let Some(ErasedEngineError::ModuleInterrupted { cause, .. }) = erased.as_engine() else {
        unreachable!()
    };
    assert_eq!(cause.as_ref(), &expected);
}

#[test]
fn a_panic_borrows_a_nested_interruption_without_reconstructing_its_original_error() {
    let mut context = context();
    let original = EngineError::ContextAccessBusy;
    let (_module, terminal) = interrupt(JsError::from(original.clone()), &mut context);
    let outer = PanicError::new("enclosing panic").with_source(terminal.clone());
    let DiagnosticKind::Engine(DiagnosticEngine::ModuleInterrupted(view)) =
        outer.diagnostic().unwrap().kind()
    else {
        panic!("nested interruption has a borrowed discriminant view")
    };
    let Some(EngineError::ModuleInterrupted(handle)) = terminal.as_engine() else {
        unreachable!()
    };
    assert_eq!(view.identity(), handle.identity());
    assert_engine_cause(view.cause(), &original);
    let (_next_module, next) = interrupt(outer.into(), &mut context);
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        diagnostic: Some(nested),
        ..
    }) = cause(&next).kind()
    else {
        unreachable!()
    };
    let DiagnosticKind::Engine(DiagnosticEngine::ModuleInterrupted(nested)) = nested.kind() else {
        unreachable!()
    };
    assert_eq!(nested.identity(), handle.identity());
    assert_engine_cause(nested.cause(), &original);
}

fn admit(error: JsError, context: &mut Context) -> JsError {
    stage(error);
    callback()
        .call(&JsValue::undefined(), &[], context.access())
        .unwrap_err()
}

#[test]
fn foreign_admission_masks_retained_objects_without_resurrecting_them_or_changing_identity() {
    let mut owner = context();
    let mut foreign = context();
    let object = JsObject::with_null_proto(owner.runtime());
    let identity = object.identity();
    let original: JsError = PanicError::new("opaque cause")
        .with_source(JsError::from_opaque(object.into()))
        .into();
    let (_module, terminal) = interrupt(original, &mut owner);
    let masked = admit(terminal.clone(), &mut foreign);
    assert_same_identity(&terminal, &masked);
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        diagnostic: Some(visible),
        ..
    }) = cause(&terminal).kind()
    else {
        unreachable!()
    };
    let DiagnosticKind::Opaque(visible) = visible.kind() else {
        unreachable!()
    };
    assert_eq!(visible.object_identity(), Some(identity));
    for narrowed in [masked.clone(), admit(masked, &mut owner)] {
        assert_same_identity(&terminal, &narrowed);
        let DiagnosticKind::Engine(DiagnosticEngine::Panic {
            message,
            diagnostic: Some(replacement),
        }) = cause(&narrowed).kind()
        else {
            unreachable!()
        };
        assert_eq!(message, "opaque cause");
        let replacement = native(replacement);
        assert_eq!(replacement.kind(), DiagnosticNativeKind::Type);
        assert_eq!(
            replacement.message(),
            "object belongs to a different runtime"
        );
        assert_eq!(replacement.realm_identity(), None);
        assert!(replacement.cause().is_none());
        assert_eq!(replacement.members().len(), 0);
    }
    assert_eq!(
        visible.value().unwrap().as_object().unwrap().identity(),
        identity
    );
}

#[test]
fn foreign_admission_hides_an_entire_native_parent_and_its_original_provenance() {
    let mut owner = context();
    let mut foreign = context();
    let captured = owner
        .eval_with_budget(
            Source::from_bytes(
                "throw new AggregateError(['hidden retained member'], 'hidden retained native',
                     { cause: 'hidden retained cause' });",
            )
            .with_path(Path::new("hidden-retained-origin.js")),
            ExecutionOptions::new(100_000),
        )
        .unwrap()
        .result
        .unwrap_err()
        .try_native(owner.access())
        .unwrap();
    let original: JsError = PanicError::new("visible panic")
        .with_source(captured)
        .into();
    let (_module, terminal) = interrupt(original, &mut owner);
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        diagnostic: Some(unrestricted),
        ..
    }) = cause(&terminal).kind()
    else {
        unreachable!()
    };
    assert_eq!(native(unrestricted).message(), "hidden retained native");
    assert_eq!(native(unrestricted).members().len(), 1);
    assert!(native(unrestricted).cause().is_some());
    assert!(native(unrestricted).realm_identity().is_some());

    let masked = admit(terminal.clone(), &mut foreign);
    let readmitted = admit(masked.clone(), &mut owner);
    for narrowed in [masked, readmitted] {
        assert_same_identity(&terminal, &narrowed);
        let DiagnosticKind::Engine(DiagnosticEngine::Panic {
            message,
            diagnostic: Some(replacement),
        }) = cause(&narrowed).kind()
        else {
            unreachable!()
        };
        assert_eq!(message, "visible panic");
        assert!(replacement.backtrace().is_none());
        let replacement = native(replacement);
        assert_eq!(replacement.kind(), DiagnosticNativeKind::Type);
        assert_eq!(
            replacement.message(),
            "error realm belongs to a different runtime"
        );
        assert_eq!(replacement.realm_identity(), None);
        assert!(replacement.cause().is_none());
        assert_eq!(replacement.members().len(), 0);
        let erased = narrowed.clone().into_erased(owner.access());
        let Some(ErasedEngineError::ModuleInterrupted {
            cause: erased_cause,
            ..
        }) = erased.as_engine()
        else {
            unreachable!()
        };
        let expected: JsError = PanicError::new("visible panic")
            .with_source(
                JsNativeError::typ().with_message("error realm belongs to a different runtime"),
            )
            .into();
        assert_eq!(erased_cause.as_ref(), &expected.into_erased(owner.access()));
        for text in [
            format!("{narrowed:?}"),
            narrowed.to_string(),
            format!("{:?}", cause(&narrowed)),
            format!("{erased:?}"),
        ] {
            for hidden in [
                "hidden retained native",
                "hidden retained member",
                "hidden retained cause",
                "hidden-retained-origin.js",
            ] {
                assert!(!text.contains(hidden), "{text}");
            }
        }
    }
    assert_eq!(native(unrestricted).message(), "hidden retained native");
}

#[test]
fn fresh_native_wait_reports_terminal_module_before_executing_a_guest_species_getter() {
    let executor = std::rc::Rc::new(surgeist_js::job::SimpleJobExecutor::new());
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .register_global_builtin_callable(js_string!("stopModule"), 0, callback())
        .unwrap();
    context
        .global_object()
        .set(js_string!("moduleEffects"), 0, true, context.access())
        .unwrap();
    let (gate, resolvers) = surgeist_js::object::builtins::JsPromise::new_pending(context.access());
    context
        .global_object()
        .set(js_string!("moduleGate"), gate, true, context.access())
        .unwrap();
    let module = Module::parse(
        Source::from_bytes(
            "await moduleGate; globalThis.moduleEffects += 1; stopModule(); globalThis.moduleEffects += 100;",
        ),
        None,
        context.access(),
    )
    .unwrap();
    assert_eq!(
        module.load(context.access()).unwrap().state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    module.link(context.access()).unwrap();
    let evaluation = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            module.evaluate(access)
        })
        .unwrap()
        .result
        .unwrap();
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    assert_eq!(effects(&mut context), 0.into());
    let original = JsError::from(EngineError::RuntimeLimit(RuntimeLimitError::JobPoll {
        limit: 73,
    }));
    stage(original.clone());
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[1.into()], context.access())
        .unwrap();
    let stopped = context
        .with_execution(ExecutionOptions::new(100_000), |mut access| {
            access.run_jobs()
        })
        .unwrap();
    assert_eq!(stopped.result.unwrap_err(), original);
    assert_eq!(effects(&mut context), 1.into());
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    let terminal = observe(&module, &mut context);
    context
        .global_object()
        .set(
            js_string!("terminalEvaluation"),
            evaluation.clone(),
            true,
            context.access(),
        )
        .unwrap();
    context
        .eval_with_budget(
            Source::from_bytes(
                "globalThis.speciesReads = 0;
             Object.defineProperty(terminalEvaluation, 'constructor', {
                 get() { speciesReads += 1; throw 'species getter must not run'; }
             });",
            ),
            ExecutionOptions::new(100_000),
        )
        .unwrap()
        .result
        .unwrap();
    let fresh_wait_error = evaluation
        .clone()
        .into_js_future(context.access())
        .unwrap_err();
    assert_same_identity(&terminal, &fresh_wait_error);
    assert_eq!(
        context
            .global_object()
            .get(js_string!("speciesReads"), context.access())
            .unwrap(),
        0.into()
    );
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    assert_eq!(effects(&mut context), 1.into());
}

#[test]
fn newly_runnable_tla_ancestor_preserves_its_original_host_panic_in_the_initiating_drain() {
    let executor = std::rc::Rc::new(surgeist_js::job::SimpleJobExecutor::new());
    let loader = std::rc::Rc::new(surgeist_js::module::MapModuleLoader::new());
    let builder = Context::builder()
        .job_executor(executor)
        .module_loader(loader.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .register_global_builtin_callable(js_string!("stopModule"), 0, callback())
        .unwrap();
    for name in ["moduleEffects", "childEffects", "parentRejections"] {
        context
            .global_object()
            .set(js_string!(name), 0, true, context.access())
            .unwrap();
    }
    let (gate, resolvers) = surgeist_js::object::builtins::JsPromise::new_pending(context.access());
    context
        .global_object()
        .set(
            js_string!("ancestorGate"),
            gate.clone(),
            true,
            context.access(),
        )
        .unwrap();
    let child = Module::parse(
        Source::from_bytes("await ancestorGate; globalThis.childEffects += 1;")
            .with_path(Path::new("ancestor-child")),
        None,
        context.access(),
    )
    .unwrap();
    loader.insert("ancestor-child", child.clone());
    let parent = Module::parse(
        Source::from_bytes(
            "import 'ancestor-child'; globalThis.moduleEffects += 1; stopModule(); await 0;",
        )
        .with_path(Path::new("ancestor-parent")),
        None,
        context.access(),
    )
    .unwrap();
    let loaded = parent.load(context.access()).unwrap();
    context
        .with_execution(ExecutionOptions::new(100_000), |mut access| {
            access.run_jobs()
        })
        .unwrap()
        .result
        .unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    parent.link(context.access()).unwrap();
    let evaluation = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            parent.evaluate(access)
        })
        .unwrap()
        .result
        .unwrap();
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    assert_eq!(gate.state().unwrap(), PromiseState::Pending);
    assert_eq!(effects(&mut context), 0.into());
    context
        .global_object()
        .set(
            js_string!("ancestorEvaluation"),
            evaluation.clone(),
            true,
            context.access(),
        )
        .unwrap();
    context
        .eval_with_budget(
            Source::from_bytes("ancestorEvaluation.catch(() => { parentRejections += 1; });"),
            ExecutionOptions::new(100_000),
        )
        .unwrap()
        .result
        .unwrap();

    // An authored Panic is not a ledger budget refusal. Operation completion
    // cannot substitute a retained budget stop for an incorrectly wrapped
    // direct job-drain result.
    let original: JsError = PanicError::new("authored ancestor host panic")
        .with_source(JsNativeError::range().with_message("authored ancestor native cause"))
        .into();
    stage(original.clone());
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[1.into()], context.access())
        .unwrap();
    let mut direct = None;
    let outcome = context
        .with_execution(
            ExecutionOptions::new(100_000)
                .with_job_preparation_limit(64)
                .with_job_invocation_limit(64)
                .with_job_poll_limit(64),
            |mut access| {
                let result = access.run_jobs();
                direct = Some(result.clone());
                result
            },
        )
        .unwrap();
    assert_eq!(direct.unwrap().unwrap_err(), original);
    assert_eq!(outcome.result.unwrap_err(), original);
    assert!(outcome.report.instructions().unwrap() > 0);
    assert!(outcome.report.instructions().unwrap() < 100_000);
    assert_eq!(effects(&mut context), 1.into());
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    assert_eq!(
        context
            .global_object()
            .get(js_string!("parentRejections"), context.access())
            .unwrap(),
        0.into()
    );
    assert_eq!(
        context
            .global_object()
            .get(js_string!("childEffects"), context.access())
            .unwrap(),
        1.into()
    );
    let child_completion = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            child.evaluate(access)
        })
        .unwrap()
        .result
        .unwrap();
    assert_eq!(
        child_completion.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(
        context
            .global_object()
            .get(js_string!("childEffects"), context.access())
            .unwrap(),
        1.into()
    );
    let terminal = observe(&parent, &mut context);
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        message,
        diagnostic: Some(source),
    }) = cause(&terminal).kind()
    else {
        panic!("the ancestor retains its authored Panic rather than another interruption wrapper")
    };
    assert_eq!(message, "authored ancestor host panic");
    let source = native(source);
    assert_eq!(source.kind(), DiagnosticNativeKind::Range);
    assert_eq!(source.message(), "authored ancestor native cause");
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
}

#[test]
fn a_locally_adopted_foreign_cause_identity_does_not_admit_an_unrelated_foreign_object() {
    let mut origin = context();
    let mut receiving = context();
    let original = EngineError::Resource(EngineResourceError::SizeOverflow);
    let (_origin_module, origin_error) = interrupt(JsError::from(original.clone()), &mut origin);
    let masked = admit(origin_error.clone(), &mut receiving);
    assert_same_identity(&origin_error, &masked);
    let (_receiving_module, adopted) = interrupt(masked, &mut receiving);
    assert_same_identity(&origin_error, &adopted);
    assert_engine_cause(cause(&adopted), &original);

    let foreign = JsObject::with_null_proto(origin.runtime());
    let foreign_identity = foreign.identity();
    // The adopted interruption is the sole receiving-runtime root here. Its
    // immutable cause identity still belongs to the origin runtime; there is
    // deliberately no receiving object or realm leaf to influence admission.
    let outer: JsError = PanicError::new("composed runtime admission")
        .with_source(JsNativeError::aggregate(vec![
            adopted,
            JsError::from_opaque(foreign.into()),
            JsError::from_opaque(42.into()),
        ]))
        .into();
    let Some(EngineError::Panic(unrestricted)) = outer.as_engine() else {
        unreachable!()
    };
    let DiagnosticKind::Opaque(authored_foreign) = native(unrestricted.diagnostic().unwrap())
        .members()
        .nth(1)
        .unwrap()
        .kind()
    else {
        unreachable!()
    };
    assert_eq!(authored_foreign.object_identity(), Some(foreign_identity));
    let admitted = admit(outer, &mut receiving);
    let Some(EngineError::Panic(panic)) = admitted.as_engine() else {
        unreachable!()
    };
    let aggregate = native(panic.diagnostic().unwrap());
    let mut members = aggregate.members();
    assert_eq!(members.len(), 3);
    let DiagnosticKind::Engine(DiagnosticEngine::ModuleInterrupted(retained)) =
        members.next().unwrap().kind()
    else {
        panic!("the locally adopted interruption remains typed")
    };
    let Some(EngineError::ModuleInterrupted(origin_handle)) = origin_error.as_engine() else {
        unreachable!()
    };
    assert_eq!(retained.identity(), origin_handle.identity());
    assert_engine_cause(retained.cause(), &original);
    let replacement = native(members.next().unwrap());
    assert_eq!(replacement.kind(), DiagnosticNativeKind::Type);
    assert_eq!(
        replacement.message(),
        "object belongs to a different runtime"
    );
    assert_eq!(replacement.realm_identity(), None);
    assert!(replacement.cause().is_none());
    assert_eq!(replacement.members().len(), 0);
    let DiagnosticKind::Opaque(primitive) = members.next().unwrap().kind() else {
        unreachable!()
    };
    assert_eq!(primitive.value().unwrap(), 42.into());
    assert_eq!(primitive.object_identity(), None);
    assert!(members.next().is_none());
}

mod final_settlement {
    use std::cell::Cell;

    use surgeist_js::{
        Context, JsError, JsObject, JsValue, Module, NativeFunction, Source,
        builtins::promise::PromiseState,
        error::{DiagnosticEngine, DiagnosticKind, EngineError, RuntimeLimitError},
        execution::ExecutionOptions,
        js_string,
    };

    thread_local! {
        static ARM_CALLS: Cell<usize> = const { Cell::new(0) };
    }

    fn assert_recursion_refusal(error: &JsError) {
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::Recursion))
        );
    }

    fn observe(module: &Module, context: &mut Context) -> JsError {
        let observed = context
            .with_execution(ExecutionOptions::new(100_000), |access| {
                module.evaluate(access)
            })
            .unwrap();
        assert_eq!(observed.report.instructions().unwrap(), 0);
        let error = observed
            .result
            .expect_err("failed final publication makes this instance terminal");
        let Some(EngineError::ModuleInterrupted(interruption)) = error.as_engine() else {
            panic!("a fresh request needs the distinct typed interruption: {error:?}")
        };
        assert_eq!(
            interruption.cause().kind(),
            DiagnosticKind::Engine(DiagnosticEngine::RuntimeLimit(RuntimeLimitError::Recursion))
        );
        error
    }

    fn assert_final_publication_is_terminal(guest_throw: bool) {
        ARM_CALLS.with(|calls| calls.set(0));
        let builder = Context::builder();
        #[cfg(feature = "fuzz")]
        let builder = builder.instructions_remaining(1_000_000);
        let mut context = builder.build().unwrap();
        let saved_limits = context.runtime_limits();
        context
            .register_global_builtin_callable(
                js_string!("armFinalPublicationRefusal"),
                0,
                NativeFunction::from_fn_ptr(|_, _, mut access| {
                    ARM_CALLS.with(|calls| calls.set(calls.get() + 1));
                    let mut limits = access.runtime_limits();
                    limits.set_recursion_limit(0);
                    access.set_runtime_limits(limits);
                    Ok(JsValue::undefined())
                }),
            )
            .unwrap();
        context
            .global_object()
            .set(js_string!("publicationEffects"), 0, true, context.access())
            .unwrap();
        context
            .global_object()
            .set(
                js_string!("publicationBodyCompleted"),
                false,
                true,
                context.access(),
            )
            .unwrap();
        let reason = JsObject::with_null_proto(context.runtime());
        reason
            .set(
                js_string!("tag"),
                js_string!("authored guest reason"),
                true,
                context.access(),
            )
            .unwrap();
        context
            .global_object()
            .set(
                js_string!("publicationGuestReason"),
                reason.clone(),
                true,
                context.access(),
            )
            .unwrap();
        let source = if guest_throw {
            "globalThis.publicationEffects += 1;
             armFinalPublicationRefusal();
             globalThis.publicationBodyCompleted = true;
             throw publicationGuestReason;"
        } else {
            "globalThis.publicationEffects += 1;
             armFinalPublicationRefusal();
             globalThis.publicationBodyCompleted = true;"
        };
        // The setter is the body's last callable operation. The following data
        // property write proves it returned and the guest body continued, with
        // no additional guest/host call before final resolve/reject publication.
        let module = Module::parse(Source::from_bytes(source), None, context.access()).unwrap();
        let loaded = module.load(context.access()).unwrap();
        assert_eq!(
            loaded.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        module.link(context.access()).unwrap();
        let mut direct = None;
        let outcome = context
            .with_execution(ExecutionOptions::new(100_000), |access| {
                let result = module.evaluate(access);
                direct = Some(result.clone());
                result
            })
            .unwrap();
        // Restore through the host API before any observation that could need a
        // new runtime entry. This must not be a retry with the armed zero limit.
        let armed_limit = context.runtime_limits().recursion_limit();
        context.set_runtime_limits(saved_limits);
        assert_eq!(armed_limit, 0);
        ARM_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        assert_eq!(
            context
                .global_object()
                .get(js_string!("publicationBodyCompleted"), context.access())
                .unwrap(),
            true.into(),
            "the stimulus must reach the end of the body rather than fail during the native setter"
        );
        assert_eq!(
            context
                .global_object()
                .get(js_string!("publicationEffects"), context.access())
                .unwrap(),
            1.into()
        );
        assert_recursion_refusal(
            &direct
                .unwrap()
                .expect_err("final intrinsic publication must refuse the armed recursion limit"),
        );
        assert_recursion_refusal(&outcome.result.unwrap_err());
        assert!(outcome.report.instructions().unwrap() > 0);
        assert!(outcome.report.instructions().unwrap() < 100_000);
        assert_eq!(
            reason.get(js_string!("tag"), context.access()).unwrap(),
            js_string!("authored guest reason").into()
        );
        assert_eq!(
            context
                .global_object()
                .get(js_string!("publicationGuestReason"), context.access())
                .unwrap()
                .as_object()
                .unwrap()
                .identity(),
            reason.identity()
        );

        let first = observe(&module, &mut context);
        let repeated = observe(&module, &mut context);
        let (
            Some(EngineError::ModuleInterrupted(first)),
            Some(EngineError::ModuleInterrupted(repeated)),
        ) = (first.as_engine(), repeated.as_engine())
        else {
            unreachable!()
        };
        assert_eq!(first.identity(), repeated.identity());
        assert_eq!(first.cause(), repeated.cause());
        assert_eq!(
            context
                .global_object()
                .get(js_string!("publicationEffects"), context.access())
                .unwrap(),
            1.into(),
            "terminal observation cannot replay the accepted body effect"
        );
        ARM_CALLS.with(|calls| assert_eq!(calls.get(), 1));
    }

    #[test]
    fn successful_body_final_resolve_refusal_is_retained_as_module_interruption() {
        assert_final_publication_is_terminal(false);
    }

    #[test]
    fn guest_throw_final_reject_refusal_is_retained_as_the_original_host_stop() {
        assert_final_publication_is_terminal(true);
    }
}
