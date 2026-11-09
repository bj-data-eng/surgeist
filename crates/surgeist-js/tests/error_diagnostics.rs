#![allow(unused_crate_dependencies)]

use std::{cell::RefCell, error::Error, path::Path};

use surgeist_js::{
    Context, JsError, JsNativeError, JsObject, JsValue, NativeFunction, Source,
    error::{
        Diagnostic, DiagnosticEngine, DiagnosticFrame, DiagnosticKind, DiagnosticNative,
        DiagnosticNativeKind, DiagnosticStack, EngineError, EngineResourceError, ErasedEngineError,
        JobError, JsErasedError, PanicDiagnostic, PanicError, RuntimeLimitError,
    },
    job::JobRetirement,
    js_string,
    value::Type,
    vm::SourcePath,
};
use surgeist_js_ast::scope::ScopeStateError;
use surgeist_js_parser::frontend::InternerLimitKind;

thread_local! {
    static CALLBACK_ERROR: RefCell<Option<JsError>> = const { RefCell::new(None) };
}

fn diagnostic_context() -> Context {
    #[cfg(feature = "fuzz")]
    {
        // Fuzz contexts require an explicit allowance for these bounded snippets.
        Context::builder()
            .instructions_remaining(100_000)
            .build()
            .unwrap()
    }
    #[cfg(not(feature = "fuzz"))]
    {
        Context::default()
    }
}

fn admit(error: JsError, receiving: &mut Context) -> JsError {
    CALLBACK_ERROR.with(|slot| assert!(slot.replace(Some(error)).is_none()));
    NativeFunction::from_fn_ptr(|_, _, _| {
        Err(CALLBACK_ERROR.with(|slot| slot.borrow_mut().take().unwrap()))
    })
    .call(&JsValue::undefined(), &[], receiving.access())
    .unwrap_err()
}

fn diagnostic(error: &JsError) -> Diagnostic<'_> {
    let Some(EngineError::Panic(panic)) = error.as_engine() else {
        panic!("expected the enclosing engine panic")
    };
    panic.diagnostic().expect("authored diagnostic source")
}

fn native(view: Diagnostic<'_>) -> DiagnosticNative<'_> {
    let DiagnosticKind::Native(native) = view.kind() else {
        panic!("expected a native diagnostic")
    };
    native
}

fn panic_with(source: impl Into<JsError>) -> JsError {
    PanicError::new("outer failure").with_source(source).into()
}

#[test]
fn standard_source_exposes_the_diagnostic_wrapper_and_typed_nested_source() {
    let ordinary = JsError::from_native(JsNativeError::range().with_message("range detail"));
    let panic = PanicError::new("outer failure").with_source(ordinary.clone());
    let source = Error::source(&panic).unwrap();
    assert!(source.downcast_ref::<JsError>().is_none());
    let wrapper = source.downcast_ref::<PanicDiagnostic>().unwrap();
    assert!(Error::source(wrapper).is_none());
    assert_eq!(wrapper.to_string(), ordinary.to_string());
    assert_eq!(wrapper.view(), panic.diagnostic().unwrap());
    let view = native(wrapper.view());
    assert_eq!(view.kind(), DiagnosticNativeKind::Range);
    assert_eq!(view.message(), "range detail");
    assert!(view.cause().is_none());
    assert_eq!(view.members().len(), 0);
    assert_eq!(panic.message(), "outer failure");
    assert!(PanicError::new("without source").diagnostic().is_none());
}

#[test]
fn native_kinds_and_authored_cause_and_member_order_remain_typed() {
    for (error, kind) in [
        (JsNativeError::error(), DiagnosticNativeKind::Error),
        (JsNativeError::eval(), DiagnosticNativeKind::Eval),
        (JsNativeError::range(), DiagnosticNativeKind::Range),
        (JsNativeError::reference(), DiagnosticNativeKind::Reference),
        (JsNativeError::syntax(), DiagnosticNativeKind::Syntax),
        (JsNativeError::typ(), DiagnosticNativeKind::Type),
        (JsNativeError::uri(), DiagnosticNativeKind::Uri),
    ] {
        let panic = panic_with(error.with_message("detail"));
        let view = native(diagnostic(&panic));
        assert_eq!(view.kind(), kind);
        assert_eq!(view.message(), "detail");
        assert_eq!(view.members().len(), 0);
    }
    let panic = panic_with(
        JsNativeError::aggregate(vec![
            JsError::from_opaque(11.into()),
            JsNativeError::syntax().with_message("middle").into(),
            JsError::from_opaque(33.into()),
        ])
        .with_message("aggregate")
        .with_cause(JsError::from_opaque(js_string!("cause").into())),
    );
    let view = native(diagnostic(&panic));
    assert_eq!(view.kind(), DiagnosticNativeKind::Aggregate);
    let DiagnosticKind::Opaque(cause) = view.cause().unwrap().kind() else {
        panic!("the authored cause precedes the aggregate members")
    };
    assert_eq!(cause.value().unwrap(), JsValue::from(js_string!("cause")));
    let mut members = view.members();
    assert_eq!(members.len(), 3);
    let DiagnosticKind::Opaque(first) = members.next().unwrap().kind() else {
        panic!("first member must stay first")
    };
    assert_eq!(first.value().unwrap(), JsValue::new(11));
    let DiagnosticKind::Opaque(last) = members.next_back().unwrap().kind() else {
        panic!("last member must stay last")
    };
    assert_eq!(last.value().unwrap(), JsValue::new(33));
    assert_eq!(members.len(), 1);
    assert_eq!(native(members.next_back().unwrap()).message(), "middle");
    assert_eq!(members.len(), 0);
    assert!(members.next().is_none());
    assert!(members.next_back().is_none());
}

#[test]
fn checked_opaque_leaf_access_preserves_identity_without_executing_a_getter() {
    let mut context = diagnostic_context();
    let object = context
        .eval(Source::from_bytes(
            "globalThis.diagnosticReads = 0;
             ({ get value() { diagnosticReads += 1; throw new Error('getter ran'); } })",
        ))
        .unwrap();
    let identity = object.as_object().unwrap().identity();
    let panic = panic_with(JsError::from_opaque(object.clone()));
    let DiagnosticKind::Opaque(leaf) = diagnostic(&panic).kind() else {
        panic!("opaque values stay opaque")
    };
    assert_eq!(leaf.value_type(), Type::Object);
    assert_eq!(leaf.object_identity(), Some(identity));
    assert_eq!(leaf.value().unwrap(), object);
    assert_eq!(
        context.eval(Source::from_bytes("diagnosticReads")).unwrap(),
        0.into()
    );
    for value in [
        JsValue::undefined(),
        JsValue::null(),
        true.into(),
        7.into(),
        js_string!("text").into(),
    ] {
        let panic = panic_with(JsError::from_opaque(value.clone()));
        let DiagnosticKind::Opaque(leaf) = diagnostic(&panic).kind() else {
            unreachable!()
        };
        assert_eq!(leaf.value_type(), value.get_type());
        assert_eq!(leaf.object_identity(), None);
        assert_eq!(leaf.value().unwrap(), value);
    }
}

fn assert_engine_view(view: Diagnostic<'_>, expected: &EngineError) {
    let DiagnosticKind::Engine(actual) = view.kind() else {
        panic!("engine diagnostic lost its type")
    };
    let expected = match expected {
        EngineError::Resource(error) => DiagnosticEngine::Resource(*error),
        EngineError::InvalidEntryPermit => DiagnosticEngine::InvalidEntryPermit,
        EngineError::ContextAccessBusy => DiagnosticEngine::ContextAccessBusy,
        EngineError::ObjectAccessBusy => DiagnosticEngine::ObjectAccessBusy,
        EngineError::ForeignSelectedRealm(expected) => {
            let DiagnosticEngine::ForeignSelectedRealm(actual) = actual else {
                panic!("publication refusal lost its borrowed diagnostic variant")
            };
            assert_eq!(actual.selected_realm(), expected.selected_realm());
            assert_eq!(actual.receiving_realm(), expected.receiving_realm());
            assert_eq!(actual.diagnostic(), expected.diagnostic());
            return;
        }
        EngineError::JobDeadlineOverflow { now, delay } => DiagnosticEngine::JobDeadlineOverflow {
            now: *now,
            delay: *delay,
        },
        EngineError::Job(error) => DiagnosticEngine::Job(*error),
        EngineError::JobContextRequired { required } => {
            DiagnosticEngine::JobContextRequired { required }
        }
        EngineError::JobRetired(reason) => DiagnosticEngine::JobRetired(*reason),
        EngineError::InvocationIdExhausted => DiagnosticEngine::InvocationIdExhausted,
        EngineError::ScopeAnalysis(error) => DiagnosticEngine::ScopeAnalysis(*error),
        EngineError::RuntimeLimit(error) => DiagnosticEngine::RuntimeLimit(*error),
        EngineError::ModuleInterrupted(expected) => {
            let DiagnosticEngine::ModuleInterrupted(actual) = actual else {
                panic!("module interruption lost its borrowed diagnostic variant")
            };
            assert_eq!(actual.identity(), expected.identity());
            assert_eq!(actual.cause(), expected.cause());
            return;
        }
        #[cfg(feature = "fuzz")]
        EngineError::NoInstructionsRemain => DiagnosticEngine::NoInstructionsRemain,
        EngineError::Panic(_) => panic!("nested panic is covered with its own typed source"),
    };
    assert_eq!(actual, expected);
}

#[test]
fn every_engine_family_preserves_typed_payloads_and_erased_export() {
    let mut context = diagnostic_context();
    let required = context.id();
    let mut cases = vec![
        EngineError::Resource(EngineResourceError::SizeOverflow),
        EngineError::Resource(EngineResourceError::Allocation),
        EngineError::InvalidEntryPermit,
        EngineError::ContextAccessBusy,
        EngineError::ObjectAccessBusy,
        EngineError::Job(JobError::TaskCompleted),
        EngineError::Job(JobError::CancellationRevoked),
        EngineError::JobContextRequired {
            required: required.clone(),
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
    for engine in cases {
        let input = JsError::from(engine.clone());
        let panic = panic_with(input.clone());
        assert_engine_view(diagnostic(&panic), &engine);
        let direct_export = input.into_erased(context.access());
        let exported = panic.into_erased(context.access());
        let Some(ErasedEngineError::Panic(exported_panic)) = exported.as_engine() else {
            panic!("outer panic export lost its type")
        };
        let source = Error::source(exported_panic)
            .unwrap()
            .downcast_ref::<Box<JsErasedError>>()
            .unwrap();
        assert_eq!(source.as_ref(), &direct_export);
        if let EngineError::JobContextRequired { .. } = engine {
            assert_eq!(
                source.as_engine(),
                Some(&ErasedEngineError::JobContextRequired {
                    required: required.clone()
                })
            );
        }
        if let EngineError::ObjectAccessBusy = engine {
            assert_eq!(
                source.as_engine(),
                Some(&ErasedEngineError::ObjectAccessBusy)
            );
        }
        if let EngineError::Job(error) = engine {
            assert_eq!(source.as_engine(), Some(&ErasedEngineError::Job(error)));
        }
    }
    fn send_sync<T: Send + Sync>() {}
    send_sync::<JsErasedError>();
}

#[test]
fn nested_panic_views_preserve_their_own_message_and_borrowed_source() {
    let error =
        panic_with(PanicError::new("inner failure").with_source(JsError::from_opaque(42.into())));
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        message,
        diagnostic: Some(inner),
    }) = diagnostic(&error).kind()
    else {
        panic!("nested panic topology must remain typed")
    };
    assert_eq!(message, "inner failure");
    let DiagnosticKind::Opaque(value) = inner.kind() else {
        unreachable!()
    };
    assert_eq!(value.value().unwrap(), 42.into());
}

#[test]
fn clone_and_sequential_cross_runtime_admission_never_resurrect_hidden_objects() {
    let mut a = diagnostic_context();
    let mut b = diagnostic_context();
    let left = JsObject::with_null_proto(a.runtime());
    let right = JsObject::with_null_proto(b.runtime());
    let raw = panic_with(JsNativeError::aggregate(vec![
        JsError::from_opaque(left.clone().into()),
        JsError::from_opaque(right.clone().into()),
    ]));
    let first = admit(raw.clone(), &mut a);
    let members: Vec<_> = native(diagnostic(&raw)).members().collect();
    assert!(matches!(members[0].kind(), DiagnosticKind::Opaque(_)));
    assert!(matches!(members[1].kind(), DiagnosticKind::Opaque(_)));
    let members: Vec<_> = native(diagnostic(&first)).members().collect();
    let DiagnosticKind::Opaque(local) = members[0].kind() else {
        unreachable!()
    };
    assert_eq!(local.object_identity(), Some(left.identity()));
    assert_eq!(
        native(members[1]).message(),
        "object belongs to a different runtime"
    );
    let second = admit(first, &mut b);
    let third = admit(second.clone(), &mut a);
    assert_eq!(second, third);
    for member in native(diagnostic(&third)).members() {
        assert_eq!(native(member).kind(), DiagnosticNativeKind::Type);
        assert_eq!(
            native(member).message(),
            "object belongs to a different runtime"
        );
    }
    let nested = admit(panic_with(second), &mut a);
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        diagnostic: Some(inner),
        ..
    }) = diagnostic(&nested).kind()
    else {
        unreachable!()
    };
    for member in native(inner).members() {
        assert_eq!(native(member).kind(), DiagnosticNativeKind::Type);
    }
}

#[test]
fn foreign_parent_masks_cause_members_and_provenance_in_every_public_observation() {
    let mut foreign = diagnostic_context();
    let mut receiver = diagnostic_context();
    let thrown = foreign
        .eval(
            Source::from_bytes(
                "throw new AggregateError(['hidden member'], 'hidden diagnostic payload',
                     { cause: 'hidden cause' });",
            )
            .with_path(Path::new("hidden-diagnostic-origin.js")),
        )
        .unwrap_err();
    let original = thrown.try_native(foreign.access()).unwrap();
    let raw = panic_with(original);
    assert!(native(diagnostic(&raw)).realm_identity().is_some());
    assert!(native(diagnostic(&raw)).cause().is_some());
    assert_eq!(native(diagnostic(&raw)).members().len(), 1);
    let admitted = admit(raw.clone(), &mut receiver);
    let view = diagnostic(&admitted);
    let replacement = native(view);
    assert_eq!(replacement.kind(), DiagnosticNativeKind::Type);
    assert_eq!(
        replacement.message(),
        "error realm belongs to a different runtime"
    );
    assert!(replacement.cause().is_none());
    assert_eq!(replacement.members().len(), 0);
    assert_eq!(replacement.realm_identity(), None);
    assert!(view.backtrace().is_none());
    let Some(EngineError::Panic(panic)) = admitted.as_engine() else {
        unreachable!()
    };
    let wrapper = Error::source(panic)
        .unwrap()
        .downcast_ref::<PanicDiagnostic>()
        .unwrap();
    assert_eq!(wrapper.view(), view);
    assert!(Error::source(wrapper).is_none());
    assert_eq!(
        admitted,
        panic_with(JsNativeError::typ().with_message("error realm belongs to a different runtime"))
    );
    let exported = admitted.clone().into_erased(receiver.access());
    let Some(ErasedEngineError::Panic(exported_panic)) = exported.as_engine() else {
        unreachable!()
    };
    let exported_source = Error::source(exported_panic)
        .unwrap()
        .downcast_ref::<Box<JsErasedError>>()
        .unwrap();
    assert_eq!(
        exported_source.to_string(),
        "TypeError: error realm belongs to a different runtime"
    );
    assert!(Error::source(exported_source.as_ref()).is_none());
    for text in [
        format!("{admitted:?}"),
        admitted.to_string(),
        format!("{view:?}"),
        view.to_string(),
        replacement.stack().to_string(),
        format!("{:?}", replacement.members()),
        format!("{wrapper:?}"),
        format!("{exported:?}"),
        exported_source.to_string(),
    ] {
        assert!(!text.contains("hidden diagnostic payload"), "{text}");
        assert!(!text.contains("hidden member"), "{text}");
        assert!(!text.contains("hidden cause"), "{text}");
        assert!(!text.contains("hidden-diagnostic-origin.js"), "{text}");
    }
    assert_eq!(
        native(diagnostic(&raw)).message(),
        "hidden diagnostic payload"
    );
}

#[test]
fn semantic_equality_ignores_capture_storage_but_distinguishes_messages_and_identity() {
    let mut context = diagnostic_context();
    let first = context
        .eval(Source::from_bytes("throw new RangeError('same')").with_path(Path::new("first.js")))
        .unwrap_err()
        .try_native(context.access())
        .unwrap();
    let second = context
        .eval(
            Source::from_bytes("\nthrow new RangeError('same')").with_path(Path::new("second.js")),
        )
        .unwrap_err()
        .try_native(context.access())
        .unwrap();
    let left = panic_with(first);
    let right = panic_with(second);
    assert_eq!(left, right);
    assert_eq!(diagnostic(&left), diagnostic(&right));
    assert_ne!(
        panic_with(JsNativeError::range().with_message("changed")),
        panic_with(JsNativeError::range().with_message("other"))
    );
    let one = JsObject::with_null_proto(context.runtime());
    let two = JsObject::with_null_proto(context.runtime());
    assert_eq!(
        panic_with(JsError::from_opaque(one.clone().into())),
        panic_with(JsError::from_opaque(one.into()))
    );
    assert_ne!(
        panic_with(JsError::from_opaque(two.into())),
        panic_with(JsError::from_opaque(
            JsObject::with_null_proto(context.runtime()).into()
        ))
    );
}

#[test]
fn captured_frames_preserve_authored_origin_and_bidirectional_iteration() {
    let mut context = diagnostic_context();
    let error = context
        .eval(
            Source::from_bytes(
                "function authoredDiagnostic() { missingDiagnosticName(); }\n\
                 function diagnosticCaller() { authoredDiagnostic(); return 1; }\n\
                 diagnosticCaller();",
            )
            .with_path(Path::new("diagnostic-origin.js")),
        )
        .unwrap_err();
    let panic = panic_with(error);
    let capture = diagnostic(&panic)
        .backtrace()
        .expect("actual VM failure captures provenance");
    let mut frames = capture.frames();
    let total = frames.len();
    assert!(total > 0);
    frames.next();
    assert_eq!(frames.len(), total - 1);
    while frames.next_back().is_some() {}
    assert_eq!(frames.len(), 0);
    let mut authored = false;
    let mut caller_seen = false;
    for frame in capture.frames() {
        if let DiagnosticFrame::Bytecode {
            function_name,
            path,
            position,
            source_text,
            ..
        } = frame
        {
            let SourcePath::Path(path) = path else {
                panic!("authored bytecode keeps its source path")
            };
            assert_eq!(path.as_ref(), Path::new("diagnostic-origin.js"));
            if function_name == &js_string!("diagnosticCaller") {
                assert_eq!(position.unwrap().line_number(), 2);
                caller_seen = true;
            }
            if function_name == &js_string!("authoredDiagnostic") {
                assert!(
                    caller_seen,
                    "captures iterate oldest caller before its callee"
                );
                assert_eq!(position.unwrap().line_number(), 1);
                assert!(!source_text.is_empty());
                authored = true;
            }
        }
    }
    assert!(authored);
}

#[test]
fn native_stack_position_retains_optional_native_origin_and_erased_opaque_behavior() {
    let mut context = diagnostic_context();
    let error = panic_with(JsNativeError::range().with_message("native"));
    let DiagnosticStack::Position(DiagnosticFrame::Native { source_info, .. }) =
        native(diagnostic(&error)).stack()
    else {
        panic!("a fresh native error has its authored native position")
    };
    assert_eq!(
        source_info.as_location().is_some(),
        cfg!(feature = "native-backtrace")
    );
    let source = JsError::from_opaque(js_string!("opaque detail").into());
    let expected = source.to_string();
    let exported = panic_with(source).into_erased(context.access());
    let Some(ErasedEngineError::Panic(panic)) = exported.as_engine() else {
        unreachable!()
    };
    assert_eq!(panic.message(), "outer failure");
    let source = Error::source(panic)
        .unwrap()
        .downcast_ref::<Box<JsErasedError>>()
        .unwrap();
    assert_eq!(source.as_opaque(), Some(expected.as_str()));
}

#[test]
fn native_backtrace_stack_preserves_guest_origin_and_distinguishes_an_empty_capture() {
    let mut context = diagnostic_context();
    let thrown = context
        .eval(
            Source::from_bytes(
                "function stackOrigin() { throw new RangeError('captured native stack'); }\n\
                 function stackCaller() { stackOrigin(); return 1; }\n\
                 stackCaller();",
            )
            .with_path(Path::new("native-stack-origin.js")),
        )
        .unwrap_err();
    let error = panic_with(thrown.try_native(context.access()).unwrap());
    let DiagnosticStack::Backtrace(capture) = native(diagnostic(&error)).stack() else {
        panic!("the guest Error object's native diagnostic retains its captured stack")
    };
    let mut caller_seen = false;
    let mut origin_seen = false;
    let mut frames = capture.frames();
    let count = frames.len();
    assert!(count >= 2);
    frames.next_back().unwrap();
    assert_eq!(frames.len(), count - 1);
    for frame in capture.frames() {
        if let DiagnosticFrame::Bytecode {
            function_name,
            path,
            position,
            source_text,
            ..
        } = frame
        {
            let SourcePath::Path(path) = path else {
                panic!("captured bytecode retains its authored path")
            };
            assert_eq!(path.as_ref(), Path::new("native-stack-origin.js"));
            if function_name == &js_string!("stackCaller") {
                assert_eq!(position.unwrap().line_number(), 2);
                assert_eq!(
                    String::from_utf16(source_text.to_code_points().unwrap()).unwrap(),
                    "function stackCaller() { stackOrigin(); return 1; }"
                );
                caller_seen = true;
            }
            if function_name == &js_string!("stackOrigin") {
                assert!(caller_seen, "native stacks iterate oldest caller first");
                assert_eq!(position.unwrap().line_number(), 1);
                assert_eq!(
                    String::from_utf16(source_text.to_code_points().unwrap()).unwrap(),
                    "function stackOrigin() { throw new RangeError('captured native stack'); }"
                );
                origin_seen = true;
            }
        }
    }
    assert!(origin_seen);

    context.runtime_limits_mut().set_backtrace_limit(0);
    let thrown = context
        .eval(Source::from_bytes("throw new RangeError('empty capture');"))
        .unwrap_err();
    let opaque_error = panic_with(thrown.clone());
    assert_eq!(
        diagnostic(&opaque_error)
            .backtrace()
            .unwrap()
            .frames()
            .len(),
        0
    );
    let native_error = panic_with(thrown.try_native(context.access()).unwrap());
    let DiagnosticStack::Backtrace(empty) = native(diagnostic(&native_error)).stack() else {
        panic!("an empty capture stays a Backtrace rather than becoming a Position")
    };
    let mut empty_frames = empty.frames();
    assert_eq!(empty_frames.len(), 0);
    assert!(empty_frames.next().is_none());
    assert!(empty_frames.next_back().is_none());
    assert!(
        diagnostic(&panic_with(JsNativeError::range()))
            .backtrace()
            .is_none()
    );
}
