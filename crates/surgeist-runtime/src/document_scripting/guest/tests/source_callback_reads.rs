//! The original trusted-attribute Source allowance for live DOM reads.

use super::*;
use surgeist_dom::{DomBatch, edit::CreatedNode, identity::DocumentId, source::DomSourceEdit};
use surgeist_html::values::Namespace;

pub(super) fn numeric_queries_consume_the_enclosing_attribute_source() {
    queries_consume_the_enclosing_attribute_source(
        "sourceReadProbe.width === 7",
        4_000,
        |batch, document| {
            let pre = batch.create_element_ns(document, Namespace::html(), "pre");
            batch.set_attribute_ns(&pre, Namespace::none(), "width", "00000007");
            pre
        },
    );
}

pub(in crate::document_scripting) fn editable_ancestry_queries_consume_the_enclosing_attribute_source()
 {
    queries_consume_the_enclosing_attribute_source(
        "sourceReadProbe.isContentEditable === true",
        20_000,
        |batch, document| {
            let root = batch.create_element_ns(document, Namespace::html(), "section");
            batch.set_attribute_ns(&root, Namespace::none(), "contenteditable", "true");
            let mut parent = root;
            for _ in 0..6 {
                let child = batch.create_element_ns(document, Namespace::html(), "div");
                batch.insert_before(&child, &parent, None);
                parent = child;
            }
            parent
        },
    );
}

pub(in crate::document_scripting) fn translation_queries_consume_the_enclosing_attribute_source() {
    queries_consume_the_enclosing_attribute_source(
        "sourceReadProbe.translate === false",
        20_000,
        |batch, document| {
            let root = batch.create_element_ns(document, Namespace::html(), "section");
            batch.set_attribute_ns(&root, Namespace::none(), "translate", "no");
            let mut parent = root;
            for _ in 0..10 {
                let child = batch.create_element_ns(document, Namespace::html(), "div");
                batch.insert_before(&child, &parent, None);
                parent = child;
            }
            parent
        },
    );
}

pub(in crate::document_scripting) fn base_uri_reads_consume_the_enclosing_attribute_source() {
    queries_consume_the_enclosing_attribute_source(
        "sourceReadProbe.baseURI === 'about:blank'",
        4_000,
        |batch, document| batch.create_element_ns(document, Namespace::html(), "div"),
    );
}

fn queries_consume_the_enclosing_attribute_source(
    predicate: &str,
    source_work: u64,
    arrange: impl FnOnce(&mut DomBatch, DocumentId) -> CreatedNode,
) {
    const QUERIES: usize = 128;
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) =
        crate::document_scripting::cross_realm_tests::hosted_session_with_dom_limits(
            &cancel,
            0,
            0,
            |limits| limits.with_meta_processing_work(1_000_000),
        );
    let mut batch = DomBatch::new();
    let probe = arrange(&mut batch, target.document());
    let element = batch.create_element_ns(target.document(), Namespace::html(), "span");
    batch.set_attribute(&element, "onclick", "original");
    let meta = batch.create_element_ns(target.document(), Namespace::html(), "meta");
    batch.set_attribute(&meta, "http-equiv", "content-security-policy");
    batch.set_attribute(&meta, "content", "require-trusted-types-for 'script'");
    let commit = session.dom().apply(batch, &cancel).unwrap();
    let probe = commit.created(&probe).unwrap().unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let meta = commit.created(&meta).unwrap().unwrap();
    session.enable_trusted_types_csp(&cancel).unwrap();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(target.document())
        .unwrap()
        .document_element()
        .unwrap();
    let head = snapshot.node(html).unwrap().children()[0];
    let full = public_node_source_limits(2_000_000, 64);
    let mut services = crate::internal::dom_source::DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut |_| None,
        url: None,
    };
    let mut source = session.source_execution();
    let crate::internal::dom_source::DomSourceDriveProgress::Pending {
        request: surgeist_dom::source::DomSourceRequest::MetaCsp(request),
        ..
    } = source
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(meta, head, None),
            full,
            &mut services,
            &cancel,
        )
        .unwrap()
    else {
        panic!("the real Meta CSP operation must reach its native processor");
    };
    session
        .service_trusted_types_meta_csp(request, full, &cancel)
        .unwrap();
    assert!(matches!(
        source
            .resume(
                &mut session.dom(),
                request.operation(),
                full,
                &mut services,
                &cancel
            )
            .unwrap(),
        crate::internal::dom_source::DomSourceDriveProgress::Complete(_)
    ));
    let mut host = attach_public_nodes(session, target, cancel.clone(), 200_000, source_work);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, probe, "sourceReadProbe");
    expose_public_node_for_source_test(&mut host, &permit, element, "probeElement");
    let entered = Rc::new(Cell::new(0usize));
    let accepted = Rc::new(Cell::new(0usize));
    host.in_document(&permit, |scope| {
        let global = scope.global()?;
        for (name, count) in [("beginReadVerification", entered.clone()), ("acceptedSourceQuery", accepted.clone())] {
            let callback = scope.function(&Callback::new(move |_, _| {
                count.set(count.get() + 1);
                Ok(Completion::Undefined)
            }))?;
            scope.set(&global, PropertyKey::String(&units(name)), &callback)?;
        }
        let script = format!(r#"
            trustedTypes.createPolicy('default', {{createScript(value) {{
                beginReadVerification();
                for (let index = 0; index < {QUERIES}; index++) {{
                    if (!({predicate})) throw new Error('incorrect native read in original Source callback');
                    acceptedSourceQuery();
                }}
                return value + '!';
            }}}});
        "#);
        scope.evaluate(&units(&script))?;
        Ok(())
    }).unwrap();
    let events = host.guest_events().unwrap();
    events.budget.set_work_remaining(1_000_000);
    let before = host.snapshot().unwrap();
    let result = host.in_document(&permit, |scope| {
        scope
            .evaluate(&units("probeElement.setAttribute('onclick', 'unreached')"))
            .map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("native read {predicate} must spend the enclosing Source allowance: {result:?}");
    };
    assert!(
        matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(
                    DocumentScriptNativeError::Dom(DomError::SourceWorkLimit { limit })
                    | DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Dom(
                        DomHostedError::Dom(DomError::SourceWorkLimit { limit })
                    ))
                )
            )) if *limit == source_work
        ),
        "original Source cause: {:?}",
        failure.cause::<DocumentGuestEventError>()
    );
    assert_eq!(entered.get(), 1);
    assert!(
        accepted.get() > 0 && accepted.get() < QUERIES,
        "accepted prefix for {predicate}={}",
        accepted.get()
    );
    assert!(events.budget.work_remaining().unwrap() > 0);
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
    assert_eq!(
        host.snapshot()
            .unwrap()
            .node(element)
            .unwrap()
            .attribute(&"onclick".into()),
        Some(&"original".into())
    );
    let pending = host
        .with_native(|dom, _, _| dom.pending_source_edit())
        .unwrap()
        .unwrap();
    assert!(pending.spent_work() <= source_work && pending.cancellation_requested());
    drop(failure);
    let completion = host
        .with_native(|dom, _, _| {
            let mut lease = dom
                .resume_source_edit(
                    pending.operation(),
                    public_node_source_limits(2_000_000, 64),
                    &cancel,
                )
                .unwrap();
            let surgeist_dom::source::DomSourceProgress::Complete(completion) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("the original cancelled verifier must settle its own native tail");
            };
            completion
        })
        .unwrap();
    assert_eq!(completion.operation(), pending.operation());
    assert!(completion.cancellation_requested());
    let budget = events.budget.clone();
    drop(events);
    let mut session = host.stop().into_session().unwrap();
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(
        session
            .snapshot()
            .node(element)
            .unwrap()
            .attribute(&"onclick".into()),
        Some(&"original".into())
    );
    assert_eq!(budget.used(), 0);
}
