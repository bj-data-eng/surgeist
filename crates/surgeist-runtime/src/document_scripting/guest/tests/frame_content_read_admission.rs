//! Exact work admission for direct access to a detached native Iframe.

use super::*;
use surgeist_bindings_js::ValueKind;

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit, Root, Root, Root) {
    let mut host = public_node_host(200_000, 200_000);
    let permit = host.permit().unwrap();
    let (id, receiver, getter, control) = host
        .in_document(&permit, |scope| {
            let getter = scope.evaluate(&units(
                "Object.getOwnPropertyDescriptor(HTMLIFrameElement.prototype, 'contentDocument')?.get",
            ))?;
            assert!(
                scope.is_callable(&getter)?,
                "missing HTMLIFrameElement.prototype.contentDocument getter"
            );
            let receiver = scope.evaluate(&units(
                "document.implementation.createHTMLDocument().createElement('iframe')",
            ))?;
            let id = projected_node_id(scope, &receiver)?;
            let control = scope.evaluate(&units("() => null"))?;
            Ok((
                id,
                scope.retain(&receiver),
                scope.retain(&getter),
                scope.retain(&control),
            ))
        })
        .unwrap();
    let snapshot = host.snapshot().unwrap();
    let node = snapshot.node(id).unwrap();
    let owner = node.owner_document().unwrap();
    assert_ne!(owner, permit.document());
    assert!(
        !snapshot
            .document(owner)
            .unwrap()
            .metadata()
            .document_environment()
            .has_ever_had_navigable()
    );
    assert_eq!(node.parent(), None);
    assert_eq!(node.as_element().unwrap().child_navigable(), Some(None));
    fund_guest_operation_after_setup(&host);
    (host, permit, receiver, getter, control)
}

fn read_null(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
    receiver: &Root,
    getter: &Root,
) -> Result<(), DocumentScriptError> {
    host.in_document(permit, |scope| {
        let receiver = scope.import(receiver)?;
        let getter = scope.import(getter)?;
        let value = scope.call(&getter, &receiver, &[])?;
        assert_eq!(scope.kind(&value)?, ValueKind::Null);
        Ok(())
    })
}

pub(in crate::document_scripting) fn detached_iframe_content_document_admits_each_native_read() {
    let (mut host, permit, receiver, getter, control) = fixture();
    let events = host.guest_events().unwrap();
    let before = events.budget.work().unwrap();
    read_null(&mut host, &permit, &receiver, &control).unwrap();
    let after_control = events.budget.work().unwrap();
    read_null(&mut host, &permit, &receiver, &getter).unwrap();
    let required = events.budget.work().unwrap() - after_control;
    assert!(
        required > after_control - before,
        "reading a null native edge adds work beyond the control call"
    );
    let budget = events.budget.clone();
    drop((receiver, getter, control, events));
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);

    for available in [required, required - 1] {
        let (mut host, permit, receiver, getter, control) = fixture();
        read_null(&mut host, &permit, &receiver, &control).unwrap();
        let events = host.guest_events().unwrap();
        let stamp = host.snapshot().unwrap().stamp();
        let retained = events.budget.used() - events.budget.capture_used();
        events.budget.set_work_remaining(available);
        let observed = read_null(&mut host, &permit, &receiver, &getter);
        if available == required {
            observed.unwrap();
            assert_eq!(events.budget.work_remaining().unwrap(), 0);
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = observed else {
                panic!("one-short detached contentDocument must refuse: {observed:?}");
            };
            let cause = failure.cause::<DocumentGuestEventError>().unwrap();
            assert!(matches!(cause, DocumentGuestEventError::WorkLimit));
            let Err(DocumentScriptError::Script(Error::HostFailure(retry))) = host
                .in_document::<()>(&permit, |_| {
                    panic!("one-short contentDocument refusal reentered guest code")
                })
            else {
                panic!("expected the original terminal contentDocument refusal");
            };
            assert!(std::ptr::eq(
                cause,
                retry.cause::<DocumentGuestEventError>().unwrap()
            ));
        }
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        assert_eq!(
            events.budget.used() - events.budget.capture_used(),
            retained
        );
        let budget = events.budget.clone();
        drop((receiver, getter, control, events));
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}
