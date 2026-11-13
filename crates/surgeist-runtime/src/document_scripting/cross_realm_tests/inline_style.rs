use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test,
};
use surgeist_cssom::CssomStore;
use surgeist_dom::DomStyleCssomRoot;

#[test]
fn guest_insertion_publishes_a_sheet_under_the_known_empty_policy() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
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
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, "p { color: red }");
    batch.insert_before(&text, &style, None);
    let style = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&style)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel, 5_000, 60_000);
    let permit = host.permit().unwrap();
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
                    source.resume(
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
                    )
                })
            })
            .unwrap();
        panic!("guest Style insertion refused: {inserted:?}; original Source: {pending:?}");
    }
    let snapshot = host.snapshot().unwrap();
    let sheet = snapshot
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap();
    assert!(
        styles
            .snapshot(document.node())
            .unwrap()
            .unwrap()
            .sheets()
            .contains(sheet.id())
    );
}
