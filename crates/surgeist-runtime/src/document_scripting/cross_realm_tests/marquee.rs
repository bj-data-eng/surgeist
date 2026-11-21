use super::*;
use crate::{
    document_scripting::guest::tests::{attach_public_nodes, expose_public_node_for_source_test},
    internal::dom_source::{DomSourceDriveError, DomSourceRuntimeError},
};

pub(super) fn stop_marquee_preserves_native_state_on_source_work_refusal() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let mut batch = DomBatch::new();
    let marquee = batch.create_element_ns(target.document(), Namespace::html(), "marquee");
    let marquee = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&marquee)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 0);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, marquee, "limitedMarquee");
    let before = host.snapshot().unwrap();
    let before_state = before
        .node(marquee)
        .unwrap()
        .as_element()
        .unwrap()
        .marquee_state()
        .unwrap();
    assert!(before_state.turned_on());
    assert_eq!(before_state.current_loop_index(), 0);

    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "limitedMarquee.stop()").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("Marquee stop Source work refusal: {result:?}");
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
        after
            .node(marquee)
            .unwrap()
            .as_element()
            .unwrap()
            .marquee_state(),
        Some(before_state)
    );
    host.with_native(|dom, _, _| assert!(dom.pending_source_edit().is_none()))
        .unwrap();
    host.stop().into_session().unwrap();
}
