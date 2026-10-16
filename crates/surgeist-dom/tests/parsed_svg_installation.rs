//! Completed contextual SVG state through the actual public parse/install owners.
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_dom::*;
use surgeist_html::{HtmlBrowsingContextPresence, HtmlParserScriptingMode};
use surgeist_html_parser::{
    HtmlFragmentParseRequest, HtmlParseCompletion, HtmlParseLimits, HtmlParseMetadataDocument,
    HtmlParseMetadataStep, HtmlParseReadiness, HtmlParseSvgScriptOrigin, HtmlSource,
};
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlCapturedRegistry, HtmlFragmentContext};

fn parse_limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}

#[test]
fn completed_disabled_contextual_svg_script_keeps_processed_parser_state_and_target_document() {
    let cancel = AtomicBool::new(false);
    let mut dom =
        Dom::new(DomLimits::new(1024, 1_000_000, 1024, 4096, 4096, 32, 1_000_000).unwrap())
            .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&host, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let before = dom.snapshot();
    let metadata = before.document(document).unwrap().metadata();
    assert_eq!(metadata.custom_element_registry(), DomRegistryBinding::Null);
    assert_eq!(
        metadata.document_environment().browsing_context(),
        HtmlBrowsingContextPresence::Absent
    );
    assert!(!metadata.document_environment().has_ever_had_navigable());
    assert!(metadata.meta_state().is_some());
    assert!(metadata.policy_container().referrer_policy().is_some());

    let capture = before
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            &cancel,
        )
        .unwrap();
    let document_subject = capture.subject(document.node()).unwrap();
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    assert!(matches!(
        context.root_registry(),
        HtmlCapturedRegistry::Null
    ));
    let report = Arc::new(
        HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8("<svg><script/></svg>"),
            context,
            parse_limits(),
            HtmlParserScriptingMode::Inert,
            false,
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    assert_eq!(
        report.effective_scripting_mode(),
        HtmlParserScriptingMode::Disabled
    );
    assert!(report.document().is_none());
    let descriptor = report.fragment_parser_document().unwrap();
    let completion = descriptor.completion();
    assert_eq!(completion.completion(), HtmlParseCompletion::Complete);
    assert_eq!(completion.readiness(), HtmlParseReadiness::Complete);
    assert!(completion.ready_for_post_load_tasks());

    // Tiny processing step 1 returns for this detached source fragment.
    // Its original controller reaches SVG-end without setting processed.
    let mut source_script = None;
    let mut source_svg = None;
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(processed) = row.svg_script_processed() {
                    assert!(!processed);
                    assert_eq!(
                        row.svg_script_origin(),
                        Some(HtmlParseSvgScriptOrigin::Parser)
                    );
                    assert!(source_script.replace(row.node().id()).is_none());
                    source_svg = row.node().parent();
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    let source_script = source_script.unwrap();
    let source_svg = source_svg.unwrap();
    let mut saw_current_document = false;
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) if row.node().id() == source_script => {
                if let Some(current) = row.node_document() {
                    let HtmlParseMetadataDocument::Captured(current) = current else {
                        panic!("returned Script belongs to the captured target Document");
                    };
                    assert_eq!(current.id(), document_subject);
                    saw_current_document = true;
                }
            }
            HtmlParseMetadataStep::Complete => break,
            _ => {}
        }
    }
    assert!(saw_current_document);

    // Install the retained producer state after qualifying its actual profile,
    // processed flag, creation cause and captured target-document reference.
    let installed = dom
        .install_parsed_html_fragment(
            report.clone(),
            &capture,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &cancel,
        )
        .unwrap();
    assert!(std::ptr::eq(installed.report(), report.as_ref()));
    assert_eq!(installed.document(), document);
    assert_eq!(installed.context_basis(), Some(capture.basis()));
    assert_eq!(
        installed.captured_node_for(&document_subject),
        Some(document.node())
    );
    let snapshot = installed.snapshot();
    let root = snapshot.node(installed.root()).unwrap();
    assert!(matches!(root.kind(), DomNodeKind::DocumentFragment));
    assert_eq!(root.parent(), None);
    assert_eq!(root.owner_document(), Some(document));
    let svg = installed.node_for(&source_svg).unwrap();
    let script = installed.node_for(&source_script).unwrap();
    assert_eq!(root.children(), &[svg]);
    assert_eq!(snapshot.node(svg).unwrap().parent(), Some(installed.root()));
    assert_eq!(snapshot.node(svg).unwrap().children(), &[script]);
    let node = snapshot.node(script).unwrap();
    assert_eq!(node.owner_document(), Some(document));
    assert_eq!(node.parent(), Some(svg));
    let state = node.as_svg_script().unwrap();
    assert!(!state.flags().already_processed());
    assert_eq!(state.origin(), DomSvgScriptOrigin::Parser);
    assert!(snapshot.node(host).unwrap().children().is_empty());
    assert!(before.node(host).unwrap().children().is_empty());
    assert_eq!(snapshot.revision().get(), before.revision().get() + 1);

    // Connection is a later explicit canonical operation, not import replay.
    let mut insertion = DomBatch::new();
    insertion.insert_before(installed.root(), host, None);
    dom.apply(insertion, &cancel).unwrap();
    let current = dom.snapshot();
    let connected = current.node(script).unwrap().as_svg_script().unwrap();
    assert!(connected.flags().already_processed());
    assert_eq!(connected.origin(), DomSvgScriptOrigin::Parser);
    assert_eq!(
        current.node(script).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(current.node(svg).unwrap().parent(), Some(host));
    assert!(!state.flags().already_processed());
    assert_eq!(installed.node_for(&source_script), Some(script));
    assert_eq!(
        installed.captured_node_for(&document_subject),
        Some(document.node())
    );
}
