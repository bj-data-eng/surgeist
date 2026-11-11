use super::*;
use crate::document_scripting::cross_realm_tests::hosted_session;
use std::sync::atomic::Ordering;

fn attach(cancel: Arc<AtomicBool>, storage_bytes: usize) -> DocumentScriptHost {
    let (session, window) = hosted_session(&cancel, 0);
    PreparedDocumentScriptHost::prepare(session, 1_000_000, cancel)
        .unwrap()
        .with_window_timer_limits(WindowTimerLimits::new(16, 8, storage_bytes))
        .enable(
            window,
            HtmlScriptingPolicy {
                user_allows: true,
                automation_allows: true,
            },
            DocumentScriptHooks {
                child_count: &b"readChildCount".map(u16::from),
                detach_source_node_if: &b"detachIf".map(u16::from),
            },
        )
        .unwrap()
}

fn clock(host: &mut DocumentScriptHost, offsets_ms: impl IntoIterator<Item = u64>) {
    host.with_native(|_, hosted, _| {
        hosted.script_window_timer_clock(offsets_ms.into_iter().map(Duration::from_millis));
    })
    .unwrap();
}

fn evaluate<T>(
    host: &mut DocumentScriptHost,
    script: &str,
    observe: impl for<'scope> FnOnce(&mut Scope<'scope>, Value<'scope>) -> Result<T, Error>,
) -> Result<T, DocumentScriptError> {
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let value = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
        observe(scope, value)
    })
}

fn run(host: &mut DocumentScriptHost, script: &str) {
    evaluate(host, script, |_, _| Ok(())).unwrap();
}

fn trace(host: &mut DocumentScriptHost, expected: &str) {
    let actual = evaluate(host, "trace.join(',')", |scope, value| {
        scope.to_utf16(&value)
    })
    .unwrap();
    assert_eq!(String::from_utf16(&actual).unwrap(), expected);
}

fn timer_backing(host: &DocumentScriptHost) -> Rc<Backing> {
    host.engine
        .as_ref()
        .unwrap()
        .window_timers
        .as_ref()
        .unwrap()
        .backing
        .clone()
}

// The V8 process must bootstrap on the same OS thread for all Runtime tests.
pub(in crate::document_scripting) fn run_timer_tests() {
    deadlines_reentrant_arguments_exceptions_and_task_checkpoints();
    nested_timers_are_clamped_only_after_the_sixth_timer_task();
    exact_registration_storage_clears_and_one_short_refuses();
    terminal_callback_releases_all_pending_roots_and_preserves_the_first_refusal();
    checkpoint_refusal_preserves_the_completed_callbacks_exception();
}

fn deadlines_reentrant_arguments_exceptions_and_task_checkpoints() {
    let mut host = attach(Arc::new(AtomicBool::new(false)), 4096);
    let permit = host.permit().unwrap();
    clock(&mut host, [0, 0, 0, 0, 0, 0, 0, 0, 3, 10]);
    run(
        &mut host,
        r#"
        const trace = [], marker = {};
        setTimeout(() => trace.push('late'), 4294967306);
        setTimeout(function (actual, absent) {
            trace.push(this === window && actual === marker && absent === undefined);
            setTimeout(() => trace.push('reentrant'), -8);
            Promise.resolve().then(() => {
                trace.push('microtask');
                setTimeout(() => trace.push('microtask timer'), 0);
            });
            throw marker;
        }, { valueOf() { trace.push('converted'); return 0.9; } }, marker, undefined);
        const clear = setTimeout(() => trace.push('cleared'), 5);
        clearTimeout(clear + 4294967296);
    "#,
    );
    trace(&mut host, "converted");
    let fired = host.service_window_timer(&permit).unwrap();
    let WindowTimerProgress::Fired {
        id: 2,
        exception: Some(exception),
    } = fired
    else {
        panic!("unexpected timer completion: {fired:?}");
    };
    evaluate(&mut host, "marker", |scope, marker| {
        let thrown = scope.import(exception.value())?;
        assert!(scope.strict_equals(&marker, &thrown)?);
        Ok(())
    })
    .unwrap();
    drop(exception);
    trace(&mut host, "converted,true,microtask");
    for id in [4, 5] {
        assert!(matches!(host.service_window_timer(&permit).unwrap(),
            WindowTimerProgress::Fired { id: found, exception: None } if found == id));
    }
    trace(
        &mut host,
        "converted,true,microtask,reentrant,microtask timer",
    );
    assert!(matches!(host.service_window_timer(&permit).unwrap(),
        WindowTimerProgress::Waiting(delay) if delay == Duration::from_millis(7)));
    assert!(matches!(
        host.service_window_timer(&permit).unwrap(),
        WindowTimerProgress::Fired {
            id: 1,
            exception: None
        }
    ));
    trace(
        &mut host,
        "converted,true,microtask,reentrant,microtask timer,late",
    );
    assert!(matches!(
        host.service_window_timer(&permit).unwrap(),
        WindowTimerProgress::Idle
    ));
    assert_eq!(timer_backing(&host).used.get(), 0);
}

fn nested_timers_are_clamped_only_after_the_sixth_timer_task() {
    let mut host = attach(Arc::new(AtomicBool::new(false)), 4096);
    let permit = host.permit().unwrap();
    clock(&mut host, std::iter::repeat_n(0, 14).chain([4]));
    run(
        &mut host,
        r#"
        let calls = 0;
        function nested() {
            calls++;
            if (calls < 7) setTimeout(nested, -1);
        }
        setTimeout(nested);
    "#,
    );
    for id in 1..=6 {
        assert!(matches!(host.service_window_timer(&permit).unwrap(),
            WindowTimerProgress::Fired { id: found, exception: None } if found == id));
    }
    assert!(matches!(host.service_window_timer(&permit).unwrap(),
        WindowTimerProgress::Waiting(delay) if delay == Duration::from_millis(4)));
    assert!(matches!(
        host.service_window_timer(&permit).unwrap(),
        WindowTimerProgress::Fired {
            id: 7,
            exception: None
        }
    ));
}

fn exact_registration_storage_clears_and_one_short_refuses() {
    let cancellation = Arc::new(AtomicBool::new(false));
    let mut probe = attach(cancellation.clone(), 4096);
    clock(&mut probe, [0]);
    run(&mut probe, "setTimeout(() => {}, 10)");
    let exact = timer_backing(&probe).used.get();
    assert!(exact > size_of::<Registration>());
    drop(probe);

    let mut host = attach(cancellation.clone(), exact);
    clock(&mut host, [0, 0]);
    run(&mut host, "const id = setTimeout(() => {}, 10)");
    let backing = timer_backing(&host);
    assert_eq!(backing.used.get(), exact);
    run(&mut host, "clearTimeout(id); setTimeout(() => {}, 10)");
    assert_eq!(backing.used.get(), exact);
    host.stop().into_session().unwrap();
    assert_eq!(backing.used.get(), 0);

    let mut host = attach(cancellation, exact - 1);
    clock(&mut host, [0]);
    let refused = evaluate(&mut host, "setTimeout(() => {}, 10)", |_, _| Ok(()));
    assert!(matches!(refused,
        Err(DocumentScriptError::Script(Error::HostFailure(failure)))
            if matches!(failure.cause::<DocumentScriptNativeError>(),
                Some(DocumentScriptNativeError::StorageLimit))));
    assert_eq!(timer_backing(&host).used.get(), 0);
}

fn terminal_callback_releases_all_pending_roots_and_preserves_the_first_refusal() {
    let cancellation = Arc::new(AtomicBool::new(false));
    let mut host = attach(cancellation.clone(), 4096);
    let permit = host.permit().unwrap();
    clock(&mut host, [0; 3]);
    expose_cancellation(&mut host, &permit, cancellation.clone());
    run(
        &mut host,
        r#"
        setTimeout(() => {
            Promise.resolve().then(() => { throw new Error('unexpected microtask'); });
            cancel();
            try { readChildCount(); } catch (_) {}
        });
        setTimeout(() => { throw new Error('unexpected timer'); }, 10);
    "#,
    );
    let backing = timer_backing(&host);
    assert!(backing.used.get() > 0);
    let Err(DocumentScriptError::Script(Error::HostFailure(first))) =
        host.service_window_timer(&permit)
    else {
        panic!("expected the actual native cancellation");
    };
    let first = first.cause::<DocumentScriptNativeError>().unwrap();
    assert!(matches!(
        first,
        DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Cancelled)
    ));
    assert_eq!(backing.used.get(), 0);
    cancellation.store(false, Ordering::Relaxed);
    let Err(DocumentScriptError::Script(Error::HostFailure(again))) =
        host.service_window_timer(&permit)
    else {
        panic!("expected the original refusal");
    };
    assert!(std::ptr::eq(
        first,
        again.cause::<DocumentScriptNativeError>().unwrap()
    ));
    host.stop().into_session().unwrap();
}

fn expose_cancellation(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    cancel: Arc<AtomicBool>,
) {
    host.in_document(permit, |scope| {
        let global = scope.global()?;
        let trigger = scope.function(&Callback::new(move |_, _| {
            cancel.store(true, Ordering::Relaxed);
            Ok(Completion::Undefined)
        }))?;
        scope.set(
            &global,
            PropertyKey::String(&b"cancel".map(u16::from)),
            &trigger,
        )?;
        Ok(())
    })
    .unwrap();
}

fn checkpoint_refusal_preserves_the_completed_callbacks_exception() {
    let cancellation = Arc::new(AtomicBool::new(false));
    let mut host = attach(cancellation.clone(), 4096);
    let permit = host.permit().unwrap();
    clock(&mut host, [0, 0]);
    expose_cancellation(&mut host, &permit, cancellation);
    run(
        &mut host,
        r#"
        setTimeout(() => {
            Promise.resolve().then(() => { cancel(); readChildCount(); });
            throw 'ordinary callback exception';
        });
    "#,
    );
    let result = host.service_window_timer(&permit).unwrap();
    let WindowTimerProgress::CheckpointRefused {
        id: 1,
        exception: Some(exception),
        error: DocumentScriptError::Script(Error::HostFailure(failure)),
    } = result
    else {
        panic!("unexpected callback and checkpoint results: {result:?}");
    };
    assert_eq!(
        exception.phase(),
        surgeist_bindings_js::ExceptionPhase::Invoke
    );
    assert!(matches!(
        failure.cause::<DocumentScriptNativeError>(),
        Some(DocumentScriptNativeError::Hosted(
            DomHostedRuntimeError::Cancelled
        ))
    ));
}
