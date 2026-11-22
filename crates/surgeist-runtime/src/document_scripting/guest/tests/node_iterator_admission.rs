use super::*;
use crate::document_scripting::cross_realm_tests::hosted_session_with_dom_limits;
use surgeist_dom::DomBatch;
use surgeist_html::values::Namespace;

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_node_iterators(4));
    let mut batch = DomBatch::new();
    let root = batch.create_element_ns(target.document(), Namespace::html(), "div");
    let leaf = batch.create_text(target.document(), "native child");
    batch.insert_before(&leaf, &root, None);
    let root = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&root)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, root, "probeIteratorRoot");
    host.in_document(&permit, |scope| {
        // Predefine the result slot so the trial prices iterator traversal
        // independently of a named lookup during a new Window assignment.
        scope
            .evaluate(&units(
                r#"
                globalThis.probeIterator = document.createNodeIterator(probeIteratorRoot, NodeFilter.SHOW_TEXT);
                globalThis.probeIteratorResult = undefined;
                "#,
            ))
            .map(drop)
    })
    .unwrap();
    (host, permit)
}

fn advance(
    host: &mut DocumentScriptHost,
    permit: &DocumentScriptPermit,
) -> Result<(), DocumentScriptError> {
    host.in_document(permit, |scope| {
        scope
            .evaluate(&units(
                "globalThis.probeIteratorResult = probeIterator.nextNode()",
            ))
            .map(drop)
    })
}

pub(in crate::document_scripting) fn traversal_refusal_preserves_the_native_pointer() {
    let (mut host, permit) = fixture();
    let budget = host.guest_events().unwrap().budget.clone();
    let before = budget.work().unwrap();
    advance(&mut host, &permit).unwrap();
    let required = budget.work().unwrap() - before;
    assert!(required > 1);
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);

    for (available, succeeds) in [(required, true), (required - 1, false)] {
        let (mut host, permit) = fixture();
        let budget = host.guest_events().unwrap().budget.clone();
        let stamp = host.snapshot().unwrap().stamp();
        budget.set_work_remaining(available);
        let result = advance(&mut host, &permit);
        if succeeds {
            result.unwrap();
            assert_eq!(budget.work_remaining().unwrap(), 0);
            assert_ne!(host.snapshot().unwrap().stamp(), stamp);
            budget.set_work_remaining(10_000);
            host.in_document(&permit, |scope| {
                let correct = scope.evaluate(&units(
                    "probeIteratorResult === probeIteratorRoot.firstChild && probeIterator.referenceNode === probeIteratorResult",
                ))?;
                assert!(scope.boolean(&correct)?);
                Ok(())
            }).unwrap();
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short iterator traversal must refuse: {result:?}");
            };
            let cause = failure.cause::<DocumentGuestEventError>().unwrap();
            assert!(matches!(cause, DocumentGuestEventError::WorkLimit));
            assert_eq!(host.snapshot().unwrap().stamp(), stamp);
            let Err(DocumentScriptError::Script(Error::HostFailure(repeated))) =
                host.in_document(&permit, |_| Ok(()))
            else {
                panic!("iterator traversal refusal must remain terminal");
            };
            assert!(std::ptr::eq(
                cause,
                repeated.cause::<DocumentGuestEventError>().unwrap()
            ));
        }
        let mut session = host.stop().into_session().unwrap();
        assert_eq!(session.dom().usage().node_iterators(), 0);
        assert_eq!(budget.used(), 0);
    }
}

pub(in crate::document_scripting) fn an_unreachable_filter_cycle_releases_its_native_cursor() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_node_iterators(1));
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let budget = host.guest_events().unwrap().budget.clone();
    let before = budget.used();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                r#"
            (() => {
                const filter = () => cursor ? 1 : 3;
                const cursor = document.createNodeIterator(document, undefined, filter);
            })()
        "#,
            ))
            .map(drop)
    })
    .unwrap();
    for _ in 0..8 {
        host.checkpoint().unwrap();
        host.engine
            .as_mut()
            .unwrap()
            .process
            .execution()
            .memory_pressure();
        host.in_document(&permit, |_| Ok(())).unwrap();
        let live = host
            .with_native(|dom, _, _| dom.usage().node_iterators())
            .unwrap();
        if live == 0 {
            break;
        }
    }
    assert_eq!(
        host.with_native(|dom, _, _| dom.usage().node_iterators())
            .unwrap(),
        0
    );
    assert_eq!(budget.used(), before);
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "globalThis.nextCursor = document.createNodeIterator(document)",
            ))
            .map(drop)
    })
    .unwrap();
    assert_eq!(
        host.with_native(|dom, _, _| dom.usage().node_iterators())
            .unwrap(),
        1
    );
    host.stop().into_session().unwrap();
    assert_eq!(budget.used(), 0);
}
