//! Native custom validity refusal preserves the control's stored error.

use super::*;
use crate::{
    document_scripting::guest::tests::{attach_public_nodes, expose_public_node_for_source_test},
    internal::dom_source::DomSourceDriveError,
};
use surgeist_html::values::DocumentString;

pub(super) fn custom_validity_preserves_original_source_refusal() {
    let cancellation = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancellation, 0);
    let mut batch = DomBatch::new();
    let subject = batch.create_element_ns(target.document(), Namespace::html(), "input");
    let original = DocumentString::from_utf16(vec![0x41, 0x0d, 0x0a, 0xd800, 0]);
    batch.set_custom_validity(&subject, original);
    let subject = session
        .dom()
        .apply(batch, &cancellation)
        .unwrap()
        .created(&subject)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancellation, 200_000, 0);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, subject, "customValidityProbe");
    let before = host.snapshot().unwrap();
    let native = before.node(subject).unwrap().as_input().unwrap().state();
    assert_eq!(
        native.custom_validity_message().code_units(),
        &[0x41, 0x0a, 0xd800, 0]
    );

    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "customValidityProbe.setCustomValidity('next')").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("expected the original Input Source refusal: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::SourceWorkLimit { limit: 0 }
        )))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(
        after.node(subject).unwrap().as_input().unwrap().state(),
        native
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
