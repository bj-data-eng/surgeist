use std::sync::Arc;

use super::*;
use crate::document_scripting::{
    cross_realm_tests::hosted_session,
    guest::tests::{attach_public_nodes, projected_node_id},
};

pub(in crate::document_scripting) fn missing_pins_refuse_without_native_publication() {
    for (initial, reexpose) in [
        ("document", "document"),
        (
            "globalThis.retainedChild = new Document().createTextNode('x'); retainedChild.ownerDocument",
            "retainedChild.ownerDocument",
        ),
        ("document.documentElement", "document.documentElement"),
    ] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (session, target) = hosted_session(&cancel, 0);
        let mut host = attach_public_nodes(session, target, cancel, 5_000, 20_000);
        let permit = host.permit().unwrap();
        let evaluate = |scope: &mut Scope<'_>, script: &str| {
            let object = scope.evaluate(&script.encode_utf16().collect::<Vec<_>>())?;
            projected_node_id(scope, &object)
        };
        let node = host
            .in_document(&permit, |scope| evaluate(scope, initial))
            .unwrap();
        let events = host.guest_events().unwrap();
        let bindings = &events.public.as_ref().unwrap().document;
        let holder = if node == bindings.attached_id.node() {
            bindings.attached.clone()
        } else if let Some(pin) = bindings.documents.borrow().as_deref()
            && pin.document.node() == node
        {
            pin.holder.clone()
        } else {
            bindings
                .ordinary
                .pins
                .borrow()
                .slots
                .iter()
                .find_map(|slot| match slot {
                    Slot::Retained(pin) if pin.live.node == node => Some(pin.holder.clone()),
                    _ => None,
                })
                .unwrap()
        };
        host.engine
            .as_mut()
            .unwrap()
            .process
            .release_callback_object(holder)
            .unwrap();
        let before = host.snapshot().unwrap().stamp();
        let bytes = events.budget.used.get();
        for _ in 0..2 {
            let error = host
                .in_document(&permit, |scope| evaluate(scope, reexpose))
                .unwrap_err();
            let DocumentScriptError::Script(Error::HostFailure(failure)) = error else {
                panic!("missing managed pin for {reexpose} must report custody failure: {error:?}");
            };
            assert!(
                matches!(
                    failure.cause::<DocumentGuestEventError>(),
                    Some(DocumentGuestEventError::Document(
                        DocumentScriptError::Native(DocumentScriptNativeError::ProjectionCustody)
                    ))
                ),
                "missing managed pin for {reexpose}: {:?}",
                failure.cause::<DocumentGuestEventError>()
            );
            let snapshot = host.snapshot().unwrap();
            assert_eq!(snapshot.stamp(), before);
            assert!(snapshot.node(node).is_ok());
            assert_eq!(events.budget.used.get(), bytes);
        }
    }
}
