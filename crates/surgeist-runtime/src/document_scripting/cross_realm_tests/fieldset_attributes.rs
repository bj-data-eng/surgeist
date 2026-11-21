//! Exact refusal before a native Fieldset custom-message change publishes.

use super::*;
use crate::{
    document_scripting::guest::tests::{attach_public_nodes, expose_public_node_for_source_test},
    internal::dom_source::DomSourceDriveError,
};
use surgeist_html::values::DocumentString;

pub(super) fn custom_validity_preserves_original_refusal_and_native_message() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let subject = batch.create_element_ns(target.document(), Namespace::html(), "fieldset");
    batch.set_custom_validity(&subject, "prior\r\nmessage");
    let subject = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&subject)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, subject, "limitedFieldset");
    let before = host.snapshot().unwrap();
    let first = host.in_document(&permit, |scope| {
        evaluate(scope, "limitedFieldset.setCustomValidity('unpublished')").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = first else {
        panic!(
            "the canonical fieldset setter must return the first Source work refusal: {first:?}"
        );
    };
    assert!(matches!(
        failure.cause::<DomSourceDriveError>(),
        Some(DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::SourceWorkLimit { limit: 0 }
        )))
    ));
    let after = host.snapshot().unwrap();
    assert_eq!(after.stamp(), before.stamp());
    let fieldset = after.node(subject).unwrap().as_fieldset().unwrap();
    assert_eq!(
        fieldset.state().custom_validity_message(),
        &DocumentString::from("prior\nmessage")
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
