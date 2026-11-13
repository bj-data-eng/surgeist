use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
use surgeist_cssom::CssomStore;
use surgeist_dom::DomStyleCssomRoot;

pub(super) fn guest_insertion_publishes_a_sheet_under_the_known_empty_policy() {
    guest_insertion(None, true);
}

pub(super) fn guest_insertion_uses_the_effective_meta_style_directive() {
    guest_insertion(
        Some("default-src 'none'; style-src-elem 'unsafe-inline'"),
        true,
    );
    guest_insertion(
        Some("style-src 'unsafe-inline'; style-src-elem 'none'"),
        false,
    );
}

fn guest_insertion(policy: Option<&str>, expected_sheet: bool) {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session_with_dom_limits(&cancel, 0, 0, |limits| {
        limits.with_meta_processing_work(1_000_000)
    });
    let document = target.document();
    let mut styles = session.style_execution();
    styles
        .bind_root(
            &session.dom(),
            "guest-inline-style".into(),
            &mut Some(DomStyleCssomRoot::new(
                document.node(),
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            )),
            &cancel,
        )
        .unwrap();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(html).unwrap().children()[1];
    let head = snapshot.node(html).unwrap().children()[0];
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, "p { color: red }");
    batch.insert_before(&text, &style, None);
    let meta = policy.map(|policy| {
        let meta = batch.create_element_ns(document, Namespace::html(), "meta");
        batch.set_attribute(&meta, "http-equiv", "content-security-policy");
        batch.set_attribute(&meta, "content", policy);
        meta
    });
    let committed = session.dom().apply(batch, &cancel).unwrap();
    let style = committed.created(&style).unwrap().unwrap();
    let meta = meta.map(|meta| committed.created(&meta).unwrap().unwrap());
    if meta.is_some() {
        session.enable_trusted_types_csp(&cancel).unwrap();
    }
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
    if let Some(meta) = meta {
        expose_public_node_for_source_test(&mut host, &permit, head, "styleHead");
        expose_public_node_for_source_test(&mut host, &permit, meta, "styleMeta");
        host.in_document(&permit, |scope| {
            evaluate(scope, "styleHead.appendChild(styleMeta)").map(drop)
        })
        .unwrap();
    }
    expose_public_node_for_source_test(&mut host, &permit, body, "styleParent");
    expose_public_node_for_source_test(&mut host, &permit, style, "inlineStyle");
    let inserted = host.in_document(&permit, |scope| {
        let result = evaluate(
            scope,
            "styleParent.appendChild(inlineStyle) === inlineStyle",
        )?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&result, &expected)?);
        Ok(())
    });
    if inserted.is_err() {
        let pending = host
            .with_native(|dom, _, mut source| {
                dom.pending_source_edit().map(|status| {
                    let progress = source.resume(
                        dom,
                        status.operation(),
                        DomSourceLimits::new(16_000_000, 200_000, 60_000, 64, 64, 64, 2),
                        &mut DomSourceServices {
                            meta: None,
                            execution_scope: None,
                            style_policy: &mut |_| None,
                            url: None,
                        },
                        &AtomicBool::new(false),
                    );
                    match progress {
                        Ok(DomSourceDriveProgress::Pending { reason, .. }) => format!("{reason:?}"),
                        other => format!("{other:?}"),
                    }
                })
            })
            .unwrap();
        panic!("guest Style insertion refused: {inserted:?}; original Source: {pending:?}");
    }
    let snapshot = host.snapshot().unwrap();
    let node = snapshot.node(style).unwrap();
    assert_eq!(node.parent(), Some(body));
    let sheet = node.as_style().unwrap().sheet().unwrap();
    let cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert_eq!(sheet.is_some(), expected_sheet);
    if let Some(sheet) = sheet {
        assert_eq!(cssom.sheets(), &[sheet.id().clone()]);
        assert_eq!(cssom.sheet_rules(sheet.id()).unwrap().len(), 1);
    } else {
        assert!(cssom.sheets().is_empty());
    }
}
