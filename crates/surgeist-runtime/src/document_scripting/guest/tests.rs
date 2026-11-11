use super::*;
use crate::DomEventRegistryLimits;
use surgeist_bindings_js::{ExceptionPhase, Primitive, PropertyKey};

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn host(cancel: Arc<AtomicBool>) -> DocumentScriptHost {
    let (session, target) = super::super::cross_realm_tests::hosted_session(&cancel, 0);
    PreparedDocumentScriptHost::prepare(session, 2_000_000, cancel)
        .unwrap()
        .enable(
            target,
            HtmlScriptingPolicy {
                user_allows: true,
                automation_allows: true,
            },
            DocumentScriptHooks {
                child_count: &units("readChildCount"),
                detach_source_node_if: &units("detachIf"),
            },
        )
        .unwrap()
}

fn in_realm_root(scope: &mut Scope<'_>, root: &Root) -> Result<Root, Error> {
    let input = scope.import(root)?;
    let holder = scope.object(ObjectPrototype::Default)?;
    let key = PropertyKey::String(&[b'x'.into()]);
    scope.set(&holder, key, &input)?;
    let read = scope.get(&holder, key)?;
    Ok(scope.retain(&read))
}

fn attach_registry(host: &mut DocumentScriptHost, permit: &DocumentScriptPermit) {
    let mut registry = Some(
        DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 8,
            pending_events: 4,
            nested_dispatches: 4,
            storage_bytes: 1_000_000,
            work: 100_000,
        })
        .unwrap(),
    );
    host.attach_guest_event_registry(
        permit,
        &mut registry,
        DocumentGuestEventLimits {
            storage_bytes: 32_000,
            work: 1000,
            dispatch: DomEventDispatchLimits {
                path_items: 1,
                snapshot_listeners: 8,
                failure_records: 8,
                callbacks: 8,
                work: 10_000,
            },
        },
    )
    .unwrap();
    assert!(registry.is_none());
}

pub(in crate::document_scripting) fn attached_registered_function_uses_the_exact_native_invocation()
{
    let cancel = Arc::new(AtomicBool::new(false));
    let mut host = host(cancel);
    let permit = host.permit().unwrap();
    attach_registry(&mut host, &permit);
    let target = host.create_guest_event_target(&permit).unwrap();
    let kind = [0xd800];
    let event = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let (first, second, thrown) = host
        .in_document(&permit, |scope| {
            let global = scope.global()?;
            let target = target.object(scope)?;
            let event = event.object(scope)?;
            scope.set(
                &global,
                PropertyKey::String(&units("expectedTarget")),
                &target,
            )?;
            scope.set(
                &global,
                PropertyKey::String(&units("expectedEvent")),
                &event,
            )?;
            let first = scope.evaluate(&units(
                "globalThis.marker = {}; globalThis.trace = []; (function(e) {
                if (this !== expectedTarget || e !== expectedEvent) throw 1;
                trace.push(1); throw marker;
            })",
            ))?;
            let second = scope.evaluate(&units(
                "(function(e) {
            if (this !== expectedTarget || e !== expectedEvent) throw 2;
            trace.push(2);
        })",
            ))?;
            let thrown = scope.get(&global, PropertyKey::String(&units("marker")))?;
            Ok((
                scope.retain(&first),
                scope.retain(&second),
                scope.retain(&thrown),
            ))
        })
        .unwrap();
    let (from_other_scope, unattached) = {
        let engine = host.engine.as_mut().unwrap();
        let mut execution = engine.process.execution();
        let other = execution.create_realm().unwrap();
        let from_other_scope = execution
            .in_realm(&other, |scope| in_realm_root(scope, &first))
            .unwrap();
        let unattached = execution
            .in_realm(&other, |scope| {
                let callback = scope.evaluate(&units("(function() { throw 3 })"))?;
                Ok(scope.retain(&callback))
            })
            .unwrap();
        let unattached = execution
            .in_realm(&engine.realm, |scope| in_realm_root(scope, &unattached))
            .unwrap();
        (from_other_scope, unattached)
    };
    let once = DomEventListenerOptions {
        once: true,
        ..Default::default()
    };
    assert!(matches!(
        host.add_guest_event_listener(&permit, &target, &kind, &unattached, once),
        Err(DocumentGuestEventError::CallbackRealmUnavailable)
    ));
    let first_id = host
        .add_guest_event_listener(&permit, &target, &kind, &from_other_scope, once)
        .unwrap();
    assert_eq!(
        host.add_guest_event_listener(
            &permit,
            &target,
            &kind,
            &first,
            DomEventListenerOptions::default()
        )
        .unwrap(),
        first_id
    );
    let second_id = host
        .add_guest_event_listener(
            &permit,
            &target,
            &kind,
            &second,
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let report = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(report.receipt().callbacks(), 2);
    assert_eq!(report.receipt().event_type().code_units(), kind);
    assert_eq!(report.receipt().failures().len(), 1);
    let [exception] = report.exceptions() else {
        panic!("exactly one ordinary exception")
    };
    assert_eq!(exception.registration, first_id);
    assert_eq!(exception.realm, permit.realm);
    host.in_document(&permit, |scope| {
        let original = scope.import(&thrown)?;
        let reported = scope.import(exception.exception.value())?;
        assert!(scope.strict_equals(&original, &reported)?);
        let trace = scope.evaluate(&units("trace.join(',') === '1,2'"))?;
        let yes = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&trace, &yes)?);
        Ok(())
    })
    .unwrap();
    assert!(!host.remove_guest_event_listener(&permit, first_id).unwrap());
    assert!(
        host.remove_guest_event_listener(&permit, second_id)
            .unwrap()
    );
    let report = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(report.receipt().callbacks(), 0);
    assert!(report.exceptions().is_empty());
    assert_eq!(event.native().current_target(), None);
    assert_eq!(event.native().composed_path().count(), 0);
}

fn proxy_in_scope(scope: &mut Scope<'_>, target: &Root) -> Result<Root, Error> {
    let target = scope.import(target)?;
    let create = scope.evaluate(&units(
        "globalThis.proxyReads = 0; globalThis.proxyCalls = 0;
        (target => new Proxy(target, {
            get(t, k, r) { proxyReads++; return Reflect.get(t, k, r); },
            getPrototypeOf(t) { proxyReads++; return Reflect.getPrototypeOf(t); },
            apply(t, r, args) { proxyCalls++; return Reflect.apply(t, r, args); }
        }))",
    ))?;
    let receiver = scope.primitive(Primitive::Undefined)?;
    let proxy = scope.call(&create, &receiver, &[target])?;
    Ok(scope.retain(&proxy))
}

fn assert_guest(scope: &mut Scope<'_>, expression: &str) -> Result<(), Error> {
    let observed = scope.evaluate(&units(expression))?;
    let yes = scope.primitive(Primitive::Boolean(true))?;
    assert!(scope.strict_equals(&observed, &yes)?, "{expression}");
    Ok(())
}

pub(in crate::document_scripting) fn attached_object_listeners_look_up_handle_event_when_invoked() {
    let mut host = host(Arc::new(AtomicBool::new(false)));
    let permit = host.permit().unwrap();
    attach_registry(&mut host, &permit);
    let target = host.create_guest_event_target(&permit).unwrap();
    let kind = [u16::from(b'x')];
    let event = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let callbacks = host
        .in_document(&permit, |scope| {
            let global = scope.global()?;
            let target = target.object(scope)?;
            let event = event.object(scope)?;
            scope.set(
                &global,
                PropertyKey::String(&units("expectedTarget")),
                &target,
            )?;
            scope.set(
                &global,
                PropertyKey::String(&units("expectedEvent")),
                &event,
            )?;
            scope.evaluate(&units(
                "globalThis.trace = [];
            globalThis.getterMarker = {};
            globalThis.callMarker = {};
            globalThis.attachedTypeError = TypeError;
            globalThis.TypeError = function ReplacementTypeError() {};
            globalThis.normalMethod = 7;
            globalThis.normalListener = {
                get handleEvent() {
                    if (this !== normalListener) throw 1;
                    trace.push('normal-get');
                    return normalMethod;
                }
            };
            globalThis.getterFailure = {
                get handleEvent() {
                    if (this !== getterFailure) throw 2;
                    trace.push('throw-get');
                    throw getterMarker;
                }
            };
            globalThis.callFailure = {
                get handleEvent() {
                    if (this !== callFailure) throw 3;
                    trace.push('call-get');
                    return function(e) {
                        if (this !== callFailure || e !== expectedEvent) throw 4;
                        trace.push('call-throw');
                        throw callMarker;
                    };
                }
            };
            globalThis.noncallable = {
                get handleEvent() { trace.push('number-get'); return 17; }
            };
            globalThis.later = function(e) {
                if (this !== expectedTarget || e !== expectedEvent) throw 5;
                trace.push('later');
            };",
            ))?;
            let mut root = |name| {
                let value = scope.get(&global, PropertyKey::String(&units(name)))?;
                Ok::<_, Error>(scope.retain(&value))
            };
            Ok([
                root("normalListener")?,
                root("getterFailure")?,
                root("callFailure")?,
                root("noncallable")?,
                root("later")?,
            ])
        })
        .unwrap();
    let attempted = callbacks.each_ref().map(|callback| {
        host.add_guest_event_listener(
            &permit,
            &target,
            &kind,
            callback,
            DomEventListenerOptions::default(),
        )
    });
    let duplicate = host
        .in_document(&permit, |scope| {
            assert_guest(scope, "trace.length === 0")?;
            let original = scope.evaluate(&units("normalListener"))?;
            Ok(scope.retain(&original))
        })
        .unwrap();
    let [normal, getter, caller, number, later] = attempted.map(|registration| {
        registration
            .unwrap_or_else(|error| panic!("attached EventListener registration failed: {error:?}"))
    });
    let replacement_options = DomEventListenerOptions {
        once: true,
        ..Default::default()
    };
    assert_eq!(
        host.add_guest_event_listener(&permit, &target, &kind, &duplicate, replacement_options)
            .unwrap(),
        normal
    );
    host.in_document(&permit, |scope| {
        assert_guest(scope, "trace.length === 0")?;
        scope.evaluate(&units(
            "normalMethod = function(e) {
                if (this !== normalListener || e !== expectedEvent) throw 6;
                trace.push('normal-call');
            };",
        ))?;
        Ok(())
    })
    .unwrap();

    let report = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(report.receipt().callbacks(), 5);
    assert_eq!(
        report
            .receipt()
            .failures()
            .iter()
            .map(|failure| failure.registration)
            .collect::<Vec<_>>(),
        [getter, caller, number]
    );
    let [getter_error, call_error, not_callable] = report.exceptions() else {
        panic!("expected exactly three ordinary listener exceptions");
    };
    assert_eq!(getter_error.registration, getter);
    assert_eq!(getter_error.exception.phase(), ExceptionPhase::PropertyRead);
    assert_eq!(call_error.registration, caller);
    assert_eq!(call_error.exception.phase(), ExceptionPhase::Invoke);
    assert_eq!(not_callable.registration, number);
    assert_eq!(not_callable.realm, permit.realm);
    host.in_document(&permit, |scope| {
        for (error, marker) in [(getter_error, "getterMarker"), (call_error, "callMarker")] {
            let original = scope.evaluate(&units(marker))?;
            let reported = scope.import(error.exception.value())?;
            assert!(scope.strict_equals(&original, &reported)?);
        }
        let type_error = scope.import(not_callable.exception.value())?;
        let check = scope.evaluate(&units(
            "value => value instanceof attachedTypeError && !(value instanceof TypeError)",
        ))?;
        let undefined = scope.primitive(Primitive::Undefined)?;
        let is_original_intrinsic = scope.call(&check, &undefined, &[type_error])?;
        let yes = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&is_original_intrinsic, &yes)?);
        assert_guest(scope,
            "trace.join(',') === 'normal-get,normal-call,throw-get,call-get,call-throw,number-get,later'",
        )
    }).unwrap();
    for registration in [normal, getter, caller, number, later] {
        assert!(
            host.remove_guest_event_listener(&permit, registration)
                .unwrap()
        );
    }
    assert!(!host.remove_guest_event_listener(&permit, normal).unwrap());
    let report = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(report.receipt().callbacks(), 0);
    assert!(report.exceptions().is_empty());
}

pub(in crate::document_scripting) fn attached_proxy_uses_its_own_realm() {
    let mut host = host(Arc::new(AtomicBool::new(false)));
    let permit = host.permit().unwrap();
    attach_registry(&mut host, &permit);
    let target = host.create_guest_event_target(&permit).unwrap();
    let kind = [u16::from(b'x')];
    let event = host
        .create_guest_event(&permit, &kind, DomEventInit::default())
        .unwrap();
    let in_attached = host
        .in_document(&permit, |scope| {
            let callable = scope.evaluate(&units(
                "globalThis.targetCalls = 0; (() => { targetCalls++; })",
            ))?;
            Ok(scope.retain(&callable))
        })
        .unwrap();
    let (other, allowed, denied) = {
        let engine = host.engine.as_mut().unwrap();
        let mut execution = engine.process.execution();
        let other = execution.create_realm().unwrap();
        let in_other = execution
            .in_realm(&other, |scope| {
                let global = scope.global()?;
                let target = target.object(scope)?;
                let event = event.object(scope)?;
                scope.set(
                    &global,
                    PropertyKey::String(&units("expectedTarget")),
                    &target,
                )?;
                scope.set(
                    &global,
                    PropertyKey::String(&units("expectedEvent")),
                    &event,
                )?;
                let callable = scope.evaluate(&units(
                    "globalThis.targetCalls = 0; (function(e) {
                    if (this !== expectedTarget || e !== expectedEvent) throw 1;
                    targetCalls++;
                })",
                ))?;
                Ok(scope.retain(&callable))
            })
            .unwrap();
        let allowed = execution
            .in_realm(&engine.realm, |scope| proxy_in_scope(scope, &in_other))
            .unwrap();
        let denied = execution
            .in_realm(&other, |scope| proxy_in_scope(scope, &in_attached))
            .unwrap();
        let allowed = execution
            .in_realm(&other, |scope| in_realm_root(scope, &allowed))
            .unwrap();
        let denied = execution
            .in_realm(&engine.realm, |scope| in_realm_root(scope, &denied))
            .unwrap();
        (other, allowed, denied)
    };
    let allow = host.add_guest_event_listener(
        &permit,
        &target,
        &kind,
        &allowed,
        DomEventListenerOptions::default(),
    );
    let deny = host.add_guest_event_listener(
        &permit,
        &target,
        &kind,
        &denied,
        DomEventListenerOptions::default(),
    );
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "proxyReads === 0 && proxyCalls === 0 && targetCalls === 0",
        )
    })
    .unwrap();
    {
        let engine = host.engine.as_mut().unwrap();
        engine
            .process
            .execution()
            .in_realm(&other, |scope| {
                assert_guest(
                    scope,
                    "proxyReads === 0 && proxyCalls === 0 && targetCalls === 0",
                )
            })
            .unwrap();
    }
    assert!(
        allow.is_ok() && matches!(deny, Err(DocumentGuestEventError::CallbackRealmUnavailable)),
        "accept the attached Proxy and refuse the unattached one: attached {allow:?}, unattached {deny:?}"
    );
    let report = host.dispatch_guest_event(&permit, &target, &event).unwrap();
    assert_eq!(report.receipt().callbacks(), 1);
    assert!(report.receipt().failures().is_empty());
    assert!(report.exceptions().is_empty());
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            "proxyReads === 0 && proxyCalls === 1 && targetCalls === 0",
        )
    })
    .unwrap();
    let engine = host.engine.as_mut().unwrap();
    engine
        .process
        .execution()
        .in_realm(&other, |scope| {
            assert_guest(
                scope,
                "proxyReads === 0 && proxyCalls === 0 && targetCalls === 1",
            )
        })
        .unwrap();
}
