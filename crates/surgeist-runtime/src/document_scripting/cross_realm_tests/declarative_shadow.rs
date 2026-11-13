//! Real parsed ShadowRoot attachment and wrapper identity in the attached Window.

use super::*;
use surgeist_dom::{
    DocumentId, DomError, DomHtmlCapture, DomHtmlCaptureLimits, DomParsedHtmlInstallLimits,
    DomSnapshot, HtmlShadowRootMode,
};
use surgeist_html::parsing::HtmlParserScriptingMode;
use surgeist_html_parser::{
    HtmlFragmentParseRequest, HtmlParseLimits, HtmlParseReport, HtmlSource,
};
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlFragmentContext};

fn parse_limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(100_000)
        .active_formatting_entries(100_000)
        .managed_nodes(10_000)
        .managed_edges(10_000)
        .managed_attributes(10_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(1_000)
        .managed_capacity_bytes(10_000_000)
        .peak_transient_bytes(10_000_000)
        .atomic_layout_bytes(10_000_000)
        .work_credits(10_000_000)
        .finish()
        .unwrap()
}

fn fragment(
    snapshot: &DomSnapshot,
    document: DocumentId,
    element: NodeId,
    markup: &str,
    cancel: &AtomicBool,
) -> (DomHtmlCapture, Arc<HtmlParseReport>) {
    let capture = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 4, 0, 16_777_216),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            cancel,
        )
        .unwrap();
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(element).unwrap())
            .unwrap();
    let report = Arc::new(
        HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(markup),
            context,
            parse_limits(),
            HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    (capture, report)
}

fn attach_window(
    mut session: DomSession,
    target: DomHostedTopLevel,
    cancel: Arc<AtomicBool>,
) -> DocumentScriptHost {
    let registry_limits = DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    };
    let mut registry = DomEventDispatcher::new(registry_limits).unwrap();
    registry.bind(&mut session.dom(), &cancel).unwrap();
    super::super::guest::tests::attach_public_nodes_with_registry(
        session,
        target,
        cancel,
        1_000,
        20_000,
        &mut Some(registry),
    )
}

pub(super) fn verify_declarative_shadow_reuse() {
    for (mode, markup, assertion) in [
        (
            HtmlShadowRootMode::Open,
            "<template shadowrootmode=open shadowrootdelegatesfocus><p>child</p></template>",
            "observed === original && observed === shadowHost.shadowRoot",
        ),
        (
            HtmlShadowRootMode::Closed,
            "<template shadowrootmode=closed shadowrootdelegatesfocus><p>child</p></template>",
            "original === null && shadowHost.shadowRoot === null && observed.mode === 'closed'",
        ),
    ] {
        let cancel = Arc::new(AtomicBool::new(false));
        let (mut session, target) = hosted_session(&cancel, 0);
        let (document, element) = {
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
            let element = batch.create_element_ns(&document, Namespace::html(), "div");
            batch.insert_before(&element, &document, None);
            let commit = dom.apply(batch, &cancel).unwrap();
            (
                commit.document(&document).unwrap(),
                commit.created(&element).unwrap().unwrap(),
            )
        };
        let (capture, report) = fragment(&session.snapshot(), document, element, markup, &cancel);
        let installed = session
            .install_parsed_html_fragment(
                report,
                &capture,
                DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
                &cancel,
            )
            .unwrap();
        let snapshot = installed.snapshot();
        let root = snapshot
            .node(element)
            .unwrap()
            .hosted_shadow_root()
            .unwrap();
        let child = snapshot.node(root).unwrap().children()[0];
        let mut host = attach_window(session, target, cancel.clone());
        let permit = host.permit().unwrap();
        for (node, name) in [(element, "shadowHost"), (child, "shadowChild")] {
            super::super::guest::tests::expose_public_node_for_source_test(
                &mut host, &permit, node, name,
            );
        }
        let (matching, mismatch) = match mode {
            HtmlShadowRootMode::Open => ("open", "closed"),
            HtmlShadowRootMode::Closed => ("closed", "open"),
        };
        let result = host.in_document(&permit, |scope| {
            let value = evaluate(
                scope,
                &format!(
                    r#"
                (() => {{
                    const original = shadowHost.shadowRoot;
                    try {{ shadowHost.attachShadow({{mode: "{mismatch}"}}); return false; }}
                    catch (error) {{
                        if (error.name !== "NotSupportedError" || shadowChild.parentNode === null)
                            return false;
                    }}
                    if (original !== null) {{
                        const calls = [];
                        customElements.define('x-shadow-reuse-reaction', class extends HTMLElement {{
                            disconnectedCallback() {{ calls.push(this); }}
                        }});
                        const custom = document.createElement('x-shadow-reuse-reaction');
                        original.appendChild(custom);
                        try {{ shadowHost.attachShadow({{mode: "{matching}"}}); return false; }}
                        catch (error) {{
                            if (error.name !== "NotSupportedError" || custom.parentNode !== original ||
                                shadowChild.parentNode !== original || calls.length !== 0) return false;
                        }}
                        original.removeChild(custom);
                        if (calls.length !== 1 || calls[0] !== custom) return false;
                    }}
                    const observed = shadowHost.attachShadow({{mode: "{matching}"}});
                    try {{ shadowHost.attachShadow({{mode: "{matching}"}}); return false; }}
                    catch (error) {{
                        if (error.name !== "NotSupportedError") return false;
                    }}
                    return ({assertion}) && observed.host === shadowHost && observed.delegatesFocus
                        && observed instanceof ShadowRoot && shadowChild.parentNode === null;
                }})()
            "#
                ),
            )?;
            let yes = scope.primitive(Primitive::Boolean(true))?;
            assert!(
                scope.strict_equals(&value, &yes)?,
                "{matching} root must preserve its canonical projection"
            );
            Ok(())
        });
        match result {
            Ok(()) => (),
            Err(DocumentScriptError::Script(Error::HostFailure(failure))) => panic!(
                "{matching} attachment refused: {:?}",
                failure.cause::<DocumentGuestEventError>()
            ),
            Err(error) => panic!("{matching} attachment failed: {error:?}"),
        }
        let after = host.snapshot().unwrap();
        assert_eq!(
            after.node(element).unwrap().hosted_shadow_root(),
            Some(root)
        );
        assert!(after.node(root).unwrap().children().is_empty());
        assert!(!after.shadow_root(root).unwrap().state().declarative());
        host.with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            batch.destroy(child);
            dom.apply(batch, &cancel).unwrap();
        })
        .unwrap();
        assert!(matches!(
            host.snapshot().unwrap().node(child),
            Err(DomError::StaleNode)
        ));
        let retired = host.in_document(&permit, |scope| {
            evaluate(scope, "shadowChild.parentNode").map(drop)
        });
        let Err(DocumentScriptError::Script(Error::HostFailure(failure))) = retired else {
            panic!("the detached child's original wrapper must observe retirement: {retired:?}");
        };
        assert!(matches!(
            failure.cause::<DocumentGuestEventError>(),
            Some(DocumentGuestEventError::Document(
                DocumentScriptError::Native(DocumentScriptNativeError::Dom(DomError::StaleNode))
            ))
        ));
        assert_eq!(
            host.snapshot().unwrap().shadow_root(root).unwrap().host(),
            element
        );
    }
    verify_mode_mismatch_precedes_realm_qualification();
}

fn verify_mode_mismatch_precedes_realm_qualification() {
    let cancel = Arc::new(AtomicBool::new(false));
    let (session, target, element) = hosted_element(&cancel, "div");
    let parent = session.snapshot().node(element).unwrap().parent().unwrap();
    let mut host = attach_window(session, target, cancel.clone());
    let permit = host.permit().unwrap();
    super::super::guest::tests::expose_public_node_for_source_test(
        &mut host,
        &permit,
        element,
        "shadowHost",
    );
    let root = host
        .with_native(|dom, _, _| {
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            batch.adopt(element, &document);
            batch.insert_before(element, &document, None);
            let document = dom
                .apply(batch, &cancel)
                .unwrap()
                .document(&document)
                .unwrap();
            let (capture, report) = fragment(
                &dom.snapshot(),
                document,
                element,
                "<template shadowrootmode=closed><p>original child</p></template>",
                &cancel,
            );
            let parsed = dom
                .raw_mut()
                .install_parsed_html_fragment(
                    report,
                    &capture,
                    DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
                    &cancel,
                )
                .unwrap();
            let root = parsed
                .snapshot()
                .node(element)
                .unwrap()
                .hosted_shadow_root()
                .unwrap();
            let mut return_to_window = DomBatch::new();
            return_to_window.adopt(element, target.document());
            return_to_window.insert_before(element, parent, None);
            dom.apply(return_to_window, &cancel).unwrap();
            root
        })
        .unwrap();
    let before = host.snapshot().unwrap();
    let retained = before.shadow_root(root).unwrap();
    assert_eq!(retained.node().relevant_global().unwrap(), None);
    assert!(retained.state().declarative());
    assert_eq!(retained.node().children().len(), 1);
    let result = host.in_document(&permit, |scope| {
        let actual = evaluate(
            scope,
            r#"(() => {
            try { shadowHost.attachShadow({mode: "open"}); }
            catch (error) { return error.name === "NotSupportedError"; }
            return false;
        })()"#,
        )?;
        let expected = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        Ok(())
    });
    match result {
        Ok(()) => (),
        Err(DocumentScriptError::Script(Error::HostFailure(failure))) => panic!(
            "mode mismatch must precede creation realm lookup: {:?}",
            failure.cause::<DocumentGuestEventError>()
        ),
        Err(error) => panic!("mode-mismatch observation failed: {error:?}"),
    }
    assert_eq!(host.snapshot().unwrap().stamp(), before.stamp());
}
