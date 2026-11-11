use super::*;
use crate::DomEventRegistryLimits;
use surgeist_bindings_js::{Primitive, PropertyKey};

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

pub(in crate::document_scripting) fn attached_registered_function_uses_the_exact_native_invocation()
{
    let cancel = Arc::new(AtomicBool::new(false));
    let mut host = host(cancel);
    let permit = host.permit().unwrap();
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
        &permit,
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
