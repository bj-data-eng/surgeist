use super::*;
use crate::internal::dom_source::{DomSourceDriveProgress, DomSourceServices};
use surgeist_dom::{DomBatch, source::DomSourceEdit};
use surgeist_html::values::Namespace;

fn fixture() -> (DocumentScriptHost, DocumentScriptPermit) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    (host, permit)
}

pub(in crate::document_scripting) fn child_document_can_expose_its_window_first(retiring: bool) {
    let (mut host, permit) = fixture();
    let frame = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let detached = batch.create_html_document(permit.document(), None);
            let frame = batch.create_element_ns(&detached, Namespace::html(), "iframe");
            dom.apply(batch, &AtomicBool::new(false))
                .unwrap()
                .created(&frame)
                .unwrap()
                .unwrap()
        })
        .unwrap();
    let snapshot = host.snapshot().unwrap();
    let root = snapshot
        .document(permit.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(root).unwrap().children()[1];
    edit(&mut host, DomSourceEdit::insert_before(frame, body, None));
    let snapshot = host.snapshot().unwrap();
    let child = snapshot
        .node(frame)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    let record = snapshot.hosted_navigable(child).unwrap();
    let document = record.active_document().unwrap();
    let top = record.parent().unwrap();
    if retiring {
        edit(&mut host, DomSourceEdit::detach(frame));
        let removal = host.snapshot().unwrap();
        assert!(removal.hosted_navigable(child).unwrap().is_retiring());
        assert_eq!(removal.document(document).unwrap().navigable(), Some(child));
    }
    expose_public_node_for_source_test(&mut host, &permit, document.node(), "childDocument");
    let expected = if retiring {
        "first.closed && window.length === 0 && first === childDocument.defaultView"
    } else {
        "!first.closed && first === window[0] && first.document === childDocument"
    };
    host.in_document(&permit, |scope| {
        assert_guest(
            scope,
            &format!(
                r"(() => {{
                const first = globalThis.retainedView = childDocument.defaultView;
                return childDocument !== document && first !== window && ({expected});
            }})()",
            ),
        )
    })
    .unwrap();
    if retiring {
        let events = host.guest_events().unwrap();
        let associated_backing = events.budget.used();
        host.with_native(|dom, mut hosted, _| {
            let cancel = AtomicBool::new(false);
            for _ in 0..128 {
                if dom
                    .snapshot()
                    .document(document)
                    .unwrap()
                    .navigable()
                    .is_none()
                {
                    return;
                }
                hosted.run_next(dom, top, &cancel).unwrap();
            }
            panic!("the native lifecycle did not clear the child association");
        })
        .unwrap();
        host.in_document(&permit, |scope| {
            assert_guest(
                scope,
                "retainedView.closed && childDocument.defaultView === null",
            )
        })
        .unwrap();
        assert!(events.budget.used() < associated_backing);
    }
    host.stop().into_session().unwrap();
}

fn edit(host: &mut DocumentScriptHost, edit: DomSourceEdit) {
    host.with_native(|dom, _, mut source| {
        let mut services = DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        };
        let result = source
            .apply(
                dom,
                edit,
                public_node_source_limits(2_000_000, 64),
                &mut services,
                &AtomicBool::new(false),
            )
            .unwrap();
        assert!(
            matches!(result, DomSourceDriveProgress::Complete(_)),
            "{result:?}"
        );
    })
    .unwrap();
}

pub(in crate::document_scripting) fn projection_only_document_has_no_local_window_producer() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session(&cancel, 0);
    let origin = {
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let owner = hosted.create_headless_host(&mut dom, &cancel).unwrap();
        hosted.create_top_level(&mut dom, owner, &cancel).unwrap()
    };
    let mut host = attach_public_nodes(session, target, cancel, 200_000, 2_000_000);
    let permit = host.permit().unwrap();
    host.register_node_creation_realm(&permit, origin).unwrap();
    expose_public_node_for_source_test(
        &mut host,
        &permit,
        origin.document().node(),
        "foreignDocument",
    );
    let before = host.snapshot().unwrap();
    assert_eq!(
        before.document(origin.document()).unwrap().navigable(),
        Some(origin.navigable())
    );
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("foreignDocument.defaultView"))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("the present association without a local WindowProxy must refuse: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::CreationGlobalUnavailable)
        ))
    ));
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    host.stop().into_session().unwrap();
}

pub(in crate::document_scripting) fn default_view_admits_exact_native_read_work() {
    let observe = |scope: &mut Scope<'_>| assert_guest(scope, "document.defaultView === window");
    let (mut host, permit) = fixture();
    let events = host.guest_events().unwrap();
    let before_work = events.budget.work().unwrap();
    host.in_document(&permit, observe).unwrap();
    let required = events.budget.work().unwrap() - before_work;
    assert!(required > 0);
    drop(events);
    host.stop().into_session().unwrap();

    for available in [required, required - 1] {
        let (mut host, permit) = fixture();
        let events = host.guest_events().unwrap();
        let stamp = host.snapshot().unwrap().stamp();
        let bytes = events.budget.used();
        events.budget.set_work_remaining(available);
        let result = host.in_document(&permit, observe);
        assert_eq!(host.snapshot().unwrap().stamp(), stamp);
        assert_eq!(events.budget.used(), bytes);
        assert_eq!(events.budget.work_remaining().unwrap(), 0);
        if available == required {
            result.unwrap();
        } else {
            let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
                panic!("one-short native read must refuse: {result:?}");
            };
            assert!(matches!(
                failure.cause::<DocumentGuestEventError>(),
                Some(DocumentGuestEventError::WorkLimit)
            ));
        }
        let budget = events.budget.clone();
        drop(events);
        host.stop().into_session().unwrap();
        assert_eq!(budget.used(), 0);
    }
}
