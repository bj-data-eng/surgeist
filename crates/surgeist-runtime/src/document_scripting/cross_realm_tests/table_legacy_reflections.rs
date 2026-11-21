//! Table reflection keeps the original native Source refusal.

use super::*;
use crate::{
    document_scripting::guest::tests::{attach_public_nodes, expose_public_node_for_source_test},
    internal::dom_source::DomSourceDriveError,
};

pub(super) fn table_legacy_reflections_preserve_original_source_refusal() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let row = batch.create_element_ns(target.document(), Namespace::html(), "tr");
    batch.set_attribute_ns(&row, Namespace::none(), "bgcolor", "original");
    let row = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&row)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, row, "tableRowProbe");

    let before = host.snapshot().unwrap();
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "tableRowProbe.bgColor = null").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("expected the original Row Source refusal: {result:?}");
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
        after.node(row).unwrap().attribute(&"bgcolor".into()),
        Some(&"original".into()),
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
