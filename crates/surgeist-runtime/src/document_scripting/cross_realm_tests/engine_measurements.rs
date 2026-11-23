use std::{
    cell::Cell,
    rc::Rc,
    sync::{Arc, atomic::AtomicBool},
};

use surgeist_bindings_js::{
    Callback, Completion, EngineInstructionCount, Error, Primitive, PropertyKey, Scope,
};

use super::{
    DocumentScriptError, DocumentScriptHost, DocumentScriptNativeError, DocumentScriptPermit,
    DomHostedTaskProgress, enable, evaluate, hosted_session, live_script_storage,
};

fn instructions(host: &DocumentScriptHost, permit: &DocumentScriptPermit) -> u64 {
    match host.observe_engine_instructions(permit).unwrap() {
        EngineInstructionCount::Exact(count) => count,
        other => panic!("expected an exact instruction count, got {other:?}"),
    }
}

fn evaluate_answer(scope: &mut Scope<'_>) -> Result<(), Error> {
    let answer = evaluate(scope, "40 + 2")?;
    let expected = scope.primitive(Primitive::Number(42.0))?;
    assert!(scope.strict_equals(&answer, &expected)?);
    Ok(())
}

pub(super) fn attached_measurements_cover_guest_realms_and_the_collector() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = enable(session, target, cancel);
    let permit = host.permit().unwrap();
    assert_eq!(
        host.observe_engine_instructions(&permit).unwrap(),
        EngineInstructionCount::Disabled
    );
    let initial_heap = host.observe_engine_heap(&permit).unwrap();
    host.in_document(&permit, evaluate_answer).unwrap();
    host.enable_engine_instruction_counting(&permit).unwrap();
    assert_eq!(instructions(&host, &permit), 0);

    let conversion_work = Rc::new(Cell::new(0));
    let observed = conversion_work.clone();
    let _conversion_observer = host
        .engine
        .as_ref()
        .unwrap()
        .preparation
        .observe_conversion_admission(move |_| observed.set(observed.get() + 1));

    host.in_document(&permit, evaluate_answer).unwrap();
    let evaluation = instructions(&host, &permit);
    assert!(evaluation > 0);
    host.enable_engine_instruction_counting(&permit).unwrap();
    assert_eq!(instructions(&host, &permit), evaluation);

    let engine = host.engine.as_mut().unwrap();
    let other = engine.process.execution().create_realm().unwrap();
    let before_other = instructions(&host, &permit);
    host.engine
        .as_mut()
        .unwrap()
        .process
        .execution()
        .in_realm(&other, |scope| {
            assert_ne!(scope.realm_authority()?.identity(), permit.realm);
            evaluate_answer(scope)
        })
        .unwrap();
    assert_eq!(instructions(&host, &permit), before_other + evaluation);

    host.in_document(&permit, |scope| {
        let before_jobs = evaluate(
            scope,
            "globalThis.measuredValue = 0; Promise.resolve(42).then(value => { measuredValue = value; }); measuredValue;",
        )?;
        let zero = scope.primitive(Primitive::Number(0.0))?;
        assert!(scope.strict_equals(&before_jobs, &zero)?);
        Ok(())
    })
    .unwrap();
    let before_checkpoint = instructions(&host, &permit);
    host.checkpoint().unwrap();
    let after_checkpoint = instructions(&host, &permit);
    assert!(after_checkpoint > before_checkpoint);
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, "measuredValue")?;
        let expected = scope.primitive(Primitive::Number(42.0))?;
        assert!(scope.strict_equals(&result, &expected)?);
        Ok(())
    })
    .unwrap();

    let before_samples = instructions(&host, &permit);
    let retained = live_script_storage(&host);
    let heap = host.observe_engine_heap(&permit).unwrap();
    assert!(heap.used_bytes > 0);
    assert!(heap.cumulative_shallow_bytes.unwrap() >= u64::try_from(heap.used_bytes).unwrap());
    assert!(
        heap.cumulative_shallow_bytes.unwrap() > initial_heap.cumulative_shallow_bytes.unwrap()
    );
    assert_eq!(host.observe_engine_heap(&permit).unwrap(), heap);
    assert_eq!(instructions(&host, &permit), before_samples);
    assert_eq!(live_script_storage(&host), retained);
    assert_eq!(conversion_work.get(), 0);

    host.engine
        .as_mut()
        .unwrap()
        .process
        .execution()
        .memory_pressure();
    let after_collection = host.observe_engine_heap(&permit).unwrap();
    assert_eq!(
        after_collection.collections_entered,
        Some(heap.collections_entered.unwrap() + 1)
    );
    assert_eq!(
        after_collection.cumulative_shallow_bytes,
        heap.cumulative_shallow_bytes
    );
    assert_eq!(instructions(&host, &permit), before_samples);
    assert_eq!(conversion_work.get(), 0);
}

pub(super) fn attached_instruction_count_keeps_the_prefix_after_host_refusal() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = enable(session, target, cancel);
    let permit = host.permit().unwrap();
    let visits = Rc::new(Cell::new(0));
    let reached = visits.clone();
    let mark = "markMeasurementProgress".encode_utf16().collect::<Vec<_>>();
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        let callback = scope.function(&Callback::new(move |_, _| {
            reached.set(reached.get() + 1);
            Ok(Completion::Undefined)
        }))?;
        scope.set(&global, PropertyKey::String(&mark), &callback)
    })
    .unwrap();
    host.enable_engine_instruction_counting(&permit).unwrap();
    let result = host.in_document(&permit, |scope| {
        // The real attached mutation hook requires an active native Source operation.
        evaluate(
            scope,
            "markMeasurementProgress(); try { detachIf(); } catch (_) {} markMeasurementProgress();",
        )
        .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(first))) = result else {
        panic!("the attached Source hook must refuse outside a Source operation");
    };
    let cause = first.cause::<DocumentScriptNativeError>().unwrap();
    assert!(matches!(cause, DocumentScriptNativeError::SourceAccess));
    assert_eq!(visits.get(), 1);
    let prefix = instructions(&host, &permit);
    assert!(prefix > 0);

    let Err(DocumentScriptError::Script(Error::HostFailure(again))) =
        host.in_document::<()>(&permit, |_| panic!("terminal owner reentered guest work"))
    else {
        panic!("retry must report the original typed refusal");
    };
    assert!(std::ptr::eq(
        cause,
        again.cause::<DocumentScriptNativeError>().unwrap()
    ));
    assert_eq!(instructions(&host, &permit), prefix);
}

pub(super) fn attached_measurements_require_the_original_live_permit() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let old_host = enable(session, target, cancel.clone());
    let foreign = old_host.permit().unwrap();
    drop(old_host.stop());

    let (session, target) = hosted_session(&cancel, 0);
    let mut host = enable(session, target, cancel.clone());
    let permit = host.permit().unwrap();
    assert!(matches!(
        host.observe_engine_instructions(&foreign),
        Err(DocumentScriptNativeError::ForeignPermit)
    ));
    assert!(matches!(
        host.enable_engine_instruction_counting(&foreign),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::ForeignPermit
        ))
    ));
    assert!(matches!(
        host.observe_engine_heap(&foreign),
        Err(DocumentScriptError::Native(
            DocumentScriptNativeError::ForeignPermit
        ))
    ));
    assert_eq!(
        host.observe_engine_instructions(&permit).unwrap(),
        EngineInstructionCount::Disabled
    );

    host.enable_engine_instruction_counting(&permit).unwrap();
    host.with_native(|dom, mut hosted, _| {
        let retirement = hosted
            .destroy_top_level(dom, target.navigable(), &cancel)
            .unwrap();
        assert!((0..256).any(|_| matches!(
            hosted.run_next(dom, retirement.traversable(), &cancel).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == retirement.operation()
        )));
    })
    .unwrap();
    assert!(matches!(
        host.observe_engine_instructions(&permit),
        Err(DocumentScriptNativeError::Revoked)
    ));
}
