use super::*;
use crate::document_scripting::guest::tests::{
    attach_public_nodes, expose_public_node_for_source_test, projected_node_id,
};

pub(super) fn template_content_is_the_live_owned_inert_fragment() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target) = hosted_session(&cancel, 0);
    let mut host = attach_public_nodes(session, target, cancel, 8_000, 80_000);
    let permit = host.permit().unwrap();
    host.in_document(&permit, |scope| {
        let result = evaluate(scope, r#"
            (() => {
                const getter = Object.getOwnPropertyDescriptor(HTMLTemplateElement.prototype, 'content')?.get;
                if (typeof getter !== 'function') return 'missing content';
                const first = document.createElement('template');
                const second = document.createElement('template');
                const fragment = first.content;
                if (!(first instanceof HTMLTemplateElement) ||
                    !(fragment instanceof DocumentFragment) || fragment !== first.content ||
                    fragment === second.content || fragment.ownerDocument === document ||
                    second.content.ownerDocument !== fragment.ownerDocument || fragment.parentNode !== null)
                    return 'canonical inert identity';
                for (const receiver of [Object.create(first), document.createElement('div'), new Proxy(first, {})]) {
                    try { getter.call(receiver); return 'invalid receiver'; }
                    catch (error) { if (!(error instanceof TypeError)) return error.name; }
                }
                const text = document.createTextNode('a\ud800');
                fragment.appendChild(text);
                const nested = fragment.appendChild(document.createElement('template'));
                nested.content.appendChild(document.createTextNode('nested'));
                if (first.firstChild !== null || fragment.firstChild !== text ||
                    text.ownerDocument !== fragment.ownerDocument ||
                    nested.content.ownerDocument !== fragment.ownerDocument)
                    return 'live contents';
                const shallow = first.cloneNode();
                const deep = first.cloneNode(true);
                if (shallow.content.firstChild !== null || deep.content === fragment ||
                    deep.content.firstChild.data !== 'a\ud800' ||
                    deep.content.lastChild.content.firstChild.data !== 'nested')
                    return 'clone contents';
                const destination = new Document();
                const imported = destination.importNode(first, true);
                if (imported.content === fragment || imported.content.ownerDocument === fragment.ownerDocument ||
                    imported.content.firstChild.data !== 'a\ud800') return 'import contents';
                const oldOwner = fragment.ownerDocument;
                if (destination.adoptNode(first) !== first || first.content !== fragment ||
                    fragment.ownerDocument === oldOwner || fragment.ownerDocument !== imported.content.ownerDocument ||
                    text.ownerDocument !== fragment.ownerDocument || nested.content.firstChild.data !== 'nested')
                    return 'adopt contents';
                return 'ok';
            })()
        "#)?;
        assert_eq!(String::from_utf16_lossy(&scope.to_utf16(&result)?), "ok");
        Ok(())
    }).unwrap();
}

pub(super) fn template_content_reaches_parsed_and_retired_native_contents() {
    use surgeist_dom::{DomParsedHtmlInstallLimits, Namespace};

    let cancel = Arc::new(AtomicBool::new(false));
    let (mut session, target) = hosted_session(&cancel, 0);
    let (document, root) = {
        let hosted = session.hosted_execution();
        let mut dom = session.dom();
        let context = hosted
            .with_current_top_level(&mut dom, target, &cancel, |source, dom| {
                source
                    .document_constructor_context(dom.raw(), target.global(), &cancel)
                    .map_err(DomHostedRuntimeError::Dom)
            })
            .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.construct_document(context);
        let root = batch.create_element_ns(&document, Namespace::html(), "main");
        batch.insert_before(&root, &document, None);
        let commit = dom.apply(batch, &cancel).unwrap();
        (
            commit.document(&document).unwrap(),
            commit.created(&root).unwrap().unwrap(),
        )
    };
    let (capture, report) = super::declarative_shadow::fragment(
        &session.snapshot(),
        document,
        root,
        "<template><span>parsed</span></template>",
        &cancel,
    );
    let parsed = session
        .install_parsed_html_fragment(
            report,
            &capture,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &cancel,
        )
        .unwrap();
    let snapshot = parsed.snapshot();
    let template = snapshot.node(parsed.root()).unwrap().children()[0];
    let contents = snapshot
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    assert_eq!(
        snapshot.node(contents).unwrap().relevant_global().unwrap(),
        Some(target.global())
    );
    let mut batch = DomBatch::new();
    let detached = batch.create_element_ns(document, Namespace::html(), "template");
    let detached = session
        .dom()
        .apply(batch, &cancel)
        .unwrap()
        .created(&detached)
        .unwrap()
        .unwrap();
    let mut host = attach_public_nodes(session, target, cancel.clone(), 8_000, 80_000);
    let permit = host.permit().unwrap();
    expose_public_node_for_source_test(&mut host, &permit, template, "parsedTemplate");
    expose_public_node_for_source_test(&mut host, &permit, detached, "retiredTemplate");
    let projected = host.in_document(&permit, |scope| {
        let result = evaluate(scope, "parsedTemplate.content instanceof DocumentFragment && parsedTemplate.content.firstChild.firstChild.data === 'parsed'")?;
        assert!(scope.boolean(&result)?);
        let result = evaluate(scope, "parsedTemplate.content")?;
        projected_node_id(scope, &result)
    }).unwrap();
    assert_eq!(projected, contents);
    host.with_native(|dom, _, _| {
        let mut batch = DomBatch::new();
        batch.destroy(detached);
        dom.apply(batch, &cancel).unwrap();
    })
    .unwrap();
    let result = host.in_document(&permit, |scope| {
        evaluate(scope, "retiredTemplate.content").map(drop)
    });
    let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = result else {
        panic!("the retired native template must refuse: {result:?}");
    };
    assert!(matches!(
        failure.cause::<DocumentGuestEventError>(),
        Some(DocumentGuestEventError::Document(
            DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
        ))
    ));
}
