use super::*;
use crate::document_scripting::cross_realm_tests::hosted_session_with_dom_limits;
use surgeist_dom::{DomBatch, tree::DomTreeWalkerFilter};
use surgeist_html::values::Namespace;

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_tree_walkers(1));
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
    expose_public_node_for_source_test(&mut host, &permit, root, "walkerRoot");
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "globalThis.walker = document.createTreeWalker(walkerRoot, NodeFilter.SHOW_TEXT)",
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
            .evaluate(&units("globalThis.walkerResult = walker.firstChild()"))
            .map(drop)
    })
}

pub(in crate::document_scripting) fn one_short_refusal_preserves_current_node() {
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
        let before = host.snapshot().unwrap().stamp();
        budget.set_work_remaining(available);
        let result = advance(&mut host, &permit);
        if succeeds {
            result.unwrap();
            assert_eq!(budget.work_remaining().unwrap(), 0);
            assert_ne!(host.snapshot().unwrap().stamp(), before);
            budget.set_work_remaining(10_000);
            host.in_document(&permit, |scope| {
                let correct = scope.evaluate(&units(
                    "walkerResult === walkerRoot.firstChild && walker.currentNode === walkerResult",
                ))?;
                assert!(scope.boolean(&correct)?);
                Ok(())
            })
            .unwrap();
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short walker traversal must refuse: {result:?}");
            };
            let cause = failure.cause::<DocumentGuestEventError>().unwrap();
            assert!(matches!(cause, DocumentGuestEventError::WorkLimit));
            assert_eq!(host.snapshot().unwrap().stamp(), before);
            let Err(DocumentScriptError::Script(Error::HostFailure(repeated))) =
                host.in_document(&permit, |_| Ok(()))
            else {
                panic!("the traversal refusal must remain terminal");
            };
            assert!(std::ptr::eq(
                cause,
                repeated.cause::<DocumentGuestEventError>().unwrap()
            ));
        }
        let mut session = host.stop().into_session().unwrap();
        assert_eq!(session.dom().usage().tree_walkers(), 0);
        assert_eq!(budget.used(), 0);
    }
}

pub(in crate::document_scripting) fn collected_filter_cycles_release_the_native_slot() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_tree_walkers(2));
    let independent = session
        .dom()
        .raw_mut()
        .create_tree_walker(
            target.document().node(),
            u32::MAX,
            DomTreeWalkerFilter::None,
            &cancel,
        )
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let budget = host.guest_events().unwrap().budget.clone();
    let before = budget.used();
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                r#"
            (() => {
                const filter = () => walker ? 1 : 3;
                const walker = document.createTreeWalker(document, undefined, filter);
            })()
        "#,
            ))
            .map(drop)
    })
    .unwrap();
    assert_eq!(
        host.with_native(|dom, _, _| dom.usage().tree_walkers())
            .unwrap(),
        2
    );
    for _ in 0..8 {
        host.checkpoint().unwrap();
        host.engine
            .as_mut()
            .unwrap()
            .process
            .execution()
            .memory_pressure();
        host.in_document(&permit, |_| Ok(())).unwrap();
        if host
            .with_native(|dom, _, _| dom.usage().tree_walkers())
            .unwrap()
            == 1
        {
            break;
        }
    }
    assert_eq!(
        host.with_native(|dom, _, _| dom.usage().tree_walkers())
            .unwrap(),
        1
    );
    assert_eq!(budget.used(), before);
    host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "globalThis.replacement = document.createTreeWalker(document)",
            ))
            .map(drop)
    })
    .unwrap();
    assert_eq!(
        host.with_native(|dom, _, _| dom.usage().tree_walkers())
            .unwrap(),
        2
    );
    let mut session = host.stop().into_session().unwrap();
    let dom = session.dom();
    assert_eq!(dom.usage().tree_walkers(), 1);
    assert_eq!(
        dom.raw().tree_walker(independent).unwrap().root(),
        target.document().node()
    );
    assert_eq!(budget.used(), 0);
}

pub(in crate::document_scripting) fn stale_cursor_and_failed_creation_keep_exact_native_causes() {
    let (mut host, permit) = fixture();
    let budget = host.guest_events().unwrap().budget.clone();
    let initial = host.snapshot().unwrap().stamp();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units(
                "document.createTreeWalker(walkerRoot, undefined, () => 1)",
            ))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("a full walker bank must refuse creation: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(
                surgeist_dom::DomError::ResourceLimit {
                    resource: surgeist_dom::limits::DomResource::TreeWalkers,
                    limit: 1
                }
            ))
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), initial);
    assert_eq!(
        host.with_native(|dom, _, _| dom.usage().tree_walkers())
            .unwrap(),
        1
    );
    let mut session = host.stop().into_session().unwrap();
    assert_eq!(session.dom().usage().tree_walkers(), 0);
    assert!(budget.used() > 0);
    drop(failure);
    assert_eq!(budget.used(), 0);

    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        hosted_session_with_dom_limits(&cancel, 0, 0, |limits| limits.with_tree_walkers(1));
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    let node = host
        .in_document(&permit, |scope| {
            let root = scope.evaluate(&units(
                r#"
            const retiredWalkerRoot = document.createElement('div');
            globalThis.retiredWalker = document.createTreeWalker(retiredWalkerRoot);
            retiredWalkerRoot;
        "#,
            ))?;
            projected_node_id(scope, &root)
        })
        .unwrap();
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(node);
        dom.apply(batch, &cancel).unwrap();
        assert_eq!(dom.usage().tree_walkers(), 0);
    })
    .unwrap();
    let before = host.snapshot().unwrap().stamp();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("retiredWalker.currentNode"))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("a retired walker getter must refuse: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(
                surgeist_dom::DomError::StaleTreeWalker
            ))
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before);
    host.stop().into_session().unwrap();
}
