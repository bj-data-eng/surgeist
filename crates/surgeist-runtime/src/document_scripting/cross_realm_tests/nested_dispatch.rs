//! Nested authored delivery inside the original selected native Source listener.

use super::*;
use crate::DomEventInit;

pub(super) fn verify_nested_authored_dispatch() {
    attached_source_listener_dispatches_an_authored_event();
    inner_terminal_refusal_clears_both_original_cursors();
}

fn attached_source_listener_dispatches_an_authored_event() {
    let cancel = Arc::new(AtomicBool::new(false));
    let AttachedCancel {
        mut host,
        dialog,
        observations,
    } = attached_cancel_host_with_reports(cancel.clone(), 8);
    let permit = host.permit().unwrap();
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host, &permit, dialog, "dialog",
    );
    host.in_document(&permit, |scope| {
        evaluate(
            scope,
            r#"
            const inner = new Event("inner", {cancelable: true});
            globalThis.innerSentinel = {};
            globalThis.order = [];
            globalThis.innerChecks = false;
            globalThis.outerRestored = false;
            dialog.addEventListener("inner", function (event) {
                order.push("inner-first");
                innerChecks = event === inner && event === globalThis.event && this === dialog
                    && event.target === dialog && event.currentTarget === dialog
                    && !event.isTrusted;
                event.preventDefault();
                throw innerSentinel;
            }, {once: true});
            dialog.addEventListener("inner", event => {
                order.push("inner-next");
                innerChecks &&= event === inner && event.defaultPrevented
                    && event === globalThis.event;
            }, {once: true});
            dialog.addEventListener("cancel", function (event) {
                globalThis.originalOuter = event;
                order.push("outer-start");
                const canceled = !dialog.dispatchEvent(inner);
                let rejectsActiveEvent = false;
                try {
                    dialog.dispatchEvent(event);
                } catch (error) {
                    rejectsActiveEvent = error.name === "InvalidStateError";
                }
                outerRestored = canceled && rejectsActiveEvent && this === dialog
                    && event === globalThis.event && event.currentTarget === dialog
                    && inner.currentTarget === null && event.isTrusted;
                event.preventDefault();
                order.push("outer-end");
            }, {once: true});
        "#,
        )
        .map(drop)
    })
    .unwrap();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let request = start_attached_cancel(&mut host, dialog, &mut services, &cancel);
    let mut outer_limits = EVENT;
    outer_limits.dispatch.callbacks = 2;
    let progress = host
        .service_attached_source_event(&permit, &request, outer_limits)
        .unwrap_or_else(|error| {
            if let DocumentScriptEventError::Access(DocumentScriptError::Script(
                Error::HostFailure(failure),
            )) = &error
            {
                panic!(
                    "nested authored dispatch refused: {:?}",
                    failure.cause::<DocumentGuestEventError>()
                );
            }
            panic!("nested authored dispatch refused: {error:?}");
        });
    let DomSourceEventServiceProgress::Serviced { receipt } = progress else {
        panic!("nested delivery must complete the original outer callback: {progress:?}");
    };
    assert_eq!(observations.borrow().len(), 1);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert!(receipt.default_prevented());
    host.with_native(|dom, _, mut source| {
        assert!(matches!(
            source.resume(dom, request.operation(), SOURCE, &mut services, &cancel).unwrap(),
            DomSourceDriveProgress::Complete(done) if done.operation() == request.operation()
        ));
    })
    .unwrap();
    let exception = host
        .take_guest_listener_exception(&permit)
        .unwrap()
        .unwrap();
    host.in_document(&permit, |scope| {
        let actual = scope.import(exception.exception.value())?;
        let expected = evaluate(scope, "innerSentinel")?;
        assert!(scope.strict_equals(&actual, &expected)?);
        let actual = evaluate(scope, "innerChecks && outerRestored && order.join(',') === 'outer-start,inner-first,inner-next,outer-end' && originalOuter.currentTarget === null && globalThis.event === undefined")?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    }).unwrap();
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );
}

fn inner_terminal_refusal_clears_both_original_cursors() {
    let cancel = Arc::new(AtomicBool::new(false));
    let AttachedCancel {
        mut host,
        dialog,
        observations,
    } = attached_cancel_host_with_reports(cancel.clone(), 8);
    let permit = host.permit().unwrap();
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host, &permit, dialog, "dialog",
    );
    let name = "inner".encode_utf16().collect::<Vec<_>>();
    let inner = host
        .create_guest_event(&permit, &name, DomEventInit::default())
        .unwrap();
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        let event = inner.object(scope)?;
        scope.set(&global, PropertyKey::String(&name), &event)?;
        evaluate(
            scope,
            r#"
            dialog.addEventListener("inner", event => {
                if (event === inner && event === globalThis.event) throw { ordinary: true };
            }, {once: true});
            dialog.addEventListener("inner", event => {
                if (event === inner && event === globalThis.event) detachIf();
            }, {once: true});
            dialog.addEventListener("inner", () => { throw { replayed: true }; });
            dialog.addEventListener("cancel", event => {
                if (event !== globalThis.event) throw { wrongOuter: true };
                try {
                    dialog.dispatchEvent(inner);
                } catch (_) {
                    throw { wrongCause: true };
                }
                throw { unexpectedReturn: true };
            }, {once: true});
        "#,
        )
        .map(drop)
    })
    .unwrap();
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let request = start_attached_cancel(&mut host, dialog, &mut services, &cancel);
    let window = host.engine.as_ref().unwrap().window_event.clone();
    let Err(DocumentScriptEventError::Access(DocumentScriptError::Script(Error::HostFailure(
        first,
    )))) = host.service_attached_source_event(&permit, &request, EVENT)
    else {
        panic!("the actual inner listener must return its terminal host refusal");
    };
    let cause = first.cause::<DocumentScriptNativeError>().unwrap();
    assert!(matches!(cause, DocumentScriptNativeError::SourceAccess));
    assert!(window.is_idle());
    assert!(host.native.attached_event_request().is_none());
    assert!(inner.native().current_target().is_none());
    assert_eq!(inner.native().composed_path().count(), 0);
    {
        let observed = observations.borrow();
        let [outer] = observed.as_slice() else {
            panic!("the outer Rust listener must run exactly once");
        };
        assert!(outer.current_target().is_none());
        assert_eq!(outer.composed_path().count(), 0);
    }
    let Err(DocumentScriptEventError::Access(DocumentScriptError::Script(Error::HostFailure(
        retry,
    )))) = host.service_attached_source_event(&permit, &request, EVENT)
    else {
        panic!("retry must retain the original nested refusal");
    };
    assert!(std::ptr::eq(
        cause,
        retry.cause::<DocumentScriptNativeError>().unwrap()
    ));
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_some()
    );
    assert!(
        host.take_guest_listener_exception(&permit)
            .unwrap()
            .is_none()
    );
    let mut session = host.stop().into_session().unwrap();
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        request.operation()
    );
}
