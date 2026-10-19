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
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
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

fn contextual_fixture(
    namespace: Namespace,
    local: &str,
) -> (Dom, DomHtmlCapture, DocumentId, NodeId) {
    let cancel = AtomicBool::new(false);
    let mut dom =
        Dom::new(DomLimits::new(1024, 1_000_000, 1024, 4096, 4096, 32, 1_000_000).unwrap())
            .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, namespace, local);
    batch.insert_before(&host, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let capture = dom
        .snapshot()
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            &cancel,
        )
        .unwrap();
    (dom, capture, document, host)
}

fn fragment_report(
    capture: &DomHtmlCapture,
    host: NodeId,
    source: &str,
) -> Arc<surgeist_html_parser::HtmlParseReport> {
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    Arc::new(
        HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(source),
            context,
            parse_limits(),
            HtmlParserScriptingMode::Inert,
            false,
        )
        .unwrap()
        .parse()
        .unwrap(),
    )
}

fn install_limits() -> DomParsedHtmlInstallLimits {
    DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000)
}

fn source_scripts(
    report: &surgeist_html_parser::HtmlParseReport,
) -> Vec<(surgeist_html::HtmlNodeId, HtmlParseSvgScriptOrigin)> {
    let mut result = Vec::new();
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(processed) = row.svg_script_processed() {
                    assert!(!processed, "detached source scripts have not processed");
                    result.push((row.node().id(), row.svg_script_origin().unwrap()));
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => return result,
        }
    }
}

#[test]
fn explicit_end_and_foreign_contexts_preserve_svg_namespace_and_parser_cause() {
    let cases = [
        (Namespace::html(), "div", "<svg><script></script></svg>"),
        (Namespace::svg(), "svg", "<script/>"),
        // SVG foreignObject is an HTML integration point; its nested SVG
        // reenters foreign parsing rather than becoming HTML Script.
        (
            Namespace::svg(),
            "foreignObject",
            "<svg><script></script></svg>",
        ),
    ];
    for (namespace, local, text) in cases {
        let (mut dom, capture, document, host) = contextual_fixture(namespace, local);
        let report = fragment_report(&capture, host, text);
        let scripts = source_scripts(&report);
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].1, HtmlParseSvgScriptOrigin::Parser);
        let installed = dom
            .install_parsed_html_fragment(
                report.clone(),
                &capture,
                install_limits(),
                &AtomicBool::new(false),
            )
            .unwrap();
        let script = installed.node_for(&scripts[0].0).unwrap();
        let node = installed.snapshot().node(script).unwrap();
        assert_eq!(
            node.as_element().unwrap().name().namespace(),
            &Namespace::svg()
        );
        assert_eq!(
            node.as_element().unwrap().name().local_name(),
            &DocumentString::from("script")
        );
        assert_eq!(node.owner_document(), Some(document));
        assert!(!node.as_svg_script().unwrap().flags().already_processed());
        assert_eq!(
            node.as_svg_script().unwrap().origin(),
            DomSvgScriptOrigin::Parser
        );
        assert!(
            installed
                .snapshot()
                .node(host)
                .unwrap()
                .children()
                .is_empty()
        );
        assert!(std::ptr::eq(installed.report(), report.as_ref()));
    }
}

#[test]
fn inert_template_and_auxiliary_parser_documents_remain_distinct_through_adoption_and_clone() {
    let cancel = AtomicBool::new(false);
    let (mut dom, capture, document, host) = contextual_fixture(Namespace::html(), "div");
    let report = fragment_report(
        &capture,
        host,
        "<style>a{}</style><template><svg><script/></svg></template>",
    );
    let parser_document = report.fragment_parser_document().unwrap().document().id();
    let scripts = source_scripts(&report);
    assert_eq!(scripts.len(), 1);
    let installed = dom
        .install_parsed_html_fragment(report, &capture, install_limits(), &cancel)
        .unwrap();
    let script = installed.node_for(&scripts[0].0).unwrap();
    let inert = installed
        .snapshot()
        .node(script)
        .unwrap()
        .owner_document()
        .unwrap();
    // Resolve the real retained auxiliary through the public Style relation,
    // never from report-local numeric positions.
    let style = installed
        .bindings()
        .iter()
        .find_map(|b| installed.snapshot().node(b.node()).unwrap().as_style())
        .unwrap();
    let auxiliary = style.parser_document().unwrap();
    assert_eq!(Some(auxiliary.node()), installed.node_for(&parser_document));
    assert_ne!(auxiliary, document);
    assert_ne!(auxiliary, inert);
    assert_ne!(inert, document);
    assert_eq!(
        installed
            .snapshot()
            .document(inert)
            .unwrap()
            .template_role(),
        DomTemplateDocumentRole::InertTemplate
    );
    assert!(matches!(
        installed
            .snapshot()
            .document(auxiliary)
            .unwrap()
            .metadata()
            .creation_cause(),
        DomDocumentCreationCause::ParsedFragmentParser
    ));
    assert!(
        !installed
            .snapshot()
            .node(script)
            .unwrap()
            .as_svg_script()
            .unwrap()
            .flags()
            .already_processed()
    );

    let mut create = DomBatch::new();
    let other = create.create_document();
    let parent = create.create_element_ns(&other, Namespace::html(), "div");
    create.insert_before(&parent, &other, None);
    let commit = dom.apply(create, &cancel).unwrap();
    let other = commit.document(&other).unwrap();
    let parent = commit.created(&parent).unwrap().unwrap();
    let mut adopt = DomBatch::new();
    adopt.adopt(script, other);
    let copied = adopt.import_node(installed.snapshot(), script, other, false);
    let commit = dom.apply(adopt, &cancel).unwrap();
    let copied = commit.created(&copied).unwrap().unwrap();
    let adopted = dom.snapshot();
    for (id, cause) in [
        (script, DomSvgScriptOrigin::Parser),
        (copied, DomSvgScriptOrigin::Clone),
    ] {
        let node = adopted.node(id).unwrap();
        assert_eq!(node.owner_document(), Some(other));
        assert_eq!(node.parent(), None);
        assert_eq!(node.as_svg_script().unwrap().origin(), cause);
        assert!(!node.as_svg_script().unwrap().flags().already_processed());
    }
    let mut connect = DomBatch::new();
    connect.insert_before(script, parent, None);
    connect.insert_before(copied, parent, None);
    dom.apply(connect, &cancel).unwrap();
    let connected = dom.snapshot();
    for (id, cause) in [
        (script, DomSvgScriptOrigin::Parser),
        (copied, DomSvgScriptOrigin::Clone),
    ] {
        let node = connected.node(id).unwrap();
        assert_eq!(node.as_svg_script().unwrap().origin(), cause);
        assert!(node.as_svg_script().unwrap().flags().already_processed());
    }
    let mut clone_started = DomBatch::new();
    let started_copy = clone_started.import_node(&connected, script, document, false);
    let commit = dom.apply(clone_started, &cancel).unwrap();
    let started_copy = commit.created(&started_copy).unwrap().unwrap();
    let latest = dom.snapshot();
    let copied = latest.node(started_copy).unwrap().as_svg_script().unwrap();
    assert_eq!(copied.origin(), DomSvgScriptOrigin::Clone);
    assert!(copied.flags().already_processed());
    assert!(
        !adopted
            .node(script)
            .unwrap()
            .as_svg_script()
            .unwrap()
            .flags()
            .already_processed()
    );
    assert_eq!(
        installed.snapshot().node(script).unwrap().owner_document(),
        Some(inert)
    );
    assert_eq!(installed.node_for(&scripts[0].0), Some(script));
}

#[test]
fn contextual_general_clone_keeps_original_clone_cause_instead_of_parser_default() {
    let (mut dom, capture, _, host) = contextual_fixture(Namespace::html(), "div");
    let report = fragment_report(
        &capture,
        host,
        "<select><button><selectedcontent></selectedcontent></button><option selected><svg><script/></svg></option></select>",
    );
    let scripts = source_scripts(&report);
    assert_eq!(scripts.len(), 2);
    assert_eq!(
        scripts
            .iter()
            .filter(|(_, cause)| *cause == HtmlParseSvgScriptOrigin::Parser)
            .count(),
        1
    );
    assert_eq!(
        scripts
            .iter()
            .filter(|(_, cause)| *cause == HtmlParseSvgScriptOrigin::Clone)
            .count(),
        1
    );
    let result = dom
        .install_parsed_html_fragment(report, &capture, install_limits(), &AtomicBool::new(false))
        .unwrap();
    let mut canonical = Vec::new();
    for (source, cause) in scripts {
        let id = result.node_for(&source).unwrap();
        let script = result.snapshot().node(id).unwrap().as_svg_script().unwrap();
        assert!(!script.flags().already_processed());
        assert_eq!(
            script.origin(),
            match cause {
                HtmlParseSvgScriptOrigin::Parser => DomSvgScriptOrigin::Parser,
                HtmlParseSvgScriptOrigin::Clone => DomSvgScriptOrigin::Clone,
            }
        );
        canonical.push(id);
    }
    assert_ne!(canonical[0], canonical[1]);
}

#[test]
fn contextual_svg_cancelled_stale_foreign_and_zero_resource_requests_are_atomic() {
    let cancel = AtomicBool::new(false);
    for resource in [
        DomParsedHtmlInstallResource::MappedNodes,
        DomParsedHtmlInstallResource::Work,
        DomParsedHtmlInstallResource::ScratchBytes,
        DomParsedHtmlInstallResource::HistoricalBytes,
    ] {
        let (mut dom, capture, _, host) = contextual_fixture(Namespace::html(), "div");
        let report = fragment_report(&capture, host, "<svg><script/></svg>");
        let before = dom.snapshot();
        let usage = dom.usage();
        let limits = match resource {
            DomParsedHtmlInstallResource::MappedNodes => {
                DomParsedHtmlInstallLimits::new(0, 10_000_000, 16_000_000, 100_000_000)
            }
            DomParsedHtmlInstallResource::Work => {
                DomParsedHtmlInstallLimits::new(1024, 0, 16_000_000, 100_000_000)
            }
            DomParsedHtmlInstallResource::ScratchBytes => {
                DomParsedHtmlInstallLimits::new(1024, 10_000_000, 0, 100_000_000)
            }
            DomParsedHtmlInstallResource::HistoricalBytes => {
                DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 0)
            }
        };
        assert!(
            matches!(dom.install_parsed_html_fragment(report.clone(), &capture, limits, &cancel), Err(DomParsedHtmlInstallError::ResourceLimit { resource: actual, .. }) if actual == resource)
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(dom.snapshot().node(host).unwrap().children().is_empty());
        // No partial map/tree/IDs block a retry on exactly the same basis.
        dom.install_parsed_html_fragment(report, &capture, install_limits(), &cancel)
            .unwrap();
    }
    let (mut dom, capture, document, host) = contextual_fixture(Namespace::html(), "div");
    let report = fragment_report(&capture, host, "<svg><script/></svg>");
    let stamp = dom.snapshot().stamp();
    let usage = dom.usage();
    assert!(matches!(
        dom.install_parsed_html_fragment(
            report.clone(),
            &capture,
            install_limits(),
            &AtomicBool::new(true)
        ),
        Err(DomParsedHtmlInstallError::Cancelled)
    ));
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.usage(), usage);
    let (mut foreign, _, _, _) = contextual_fixture(Namespace::html(), "div");
    let foreign_stamp = foreign.snapshot().stamp();
    assert!(matches!(
        foreign.install_parsed_html_fragment(report.clone(), &capture, install_limits(), &cancel),
        Err(DomParsedHtmlInstallError::Target(DomError::ForeignOwner))
    ));
    assert_eq!(foreign.snapshot().stamp(), foreign_stamp);
    let mut update = DomBatch::new();
    let text = update.create_text(document, "changed");
    update.insert_before(&text, host, None);
    dom.apply(update, &cancel).unwrap();
    let changed = dom.snapshot().stamp();
    let usage = dom.usage();
    assert!(matches!(
        dom.install_parsed_html_fragment(report, &capture, install_limits(), &cancel),
        Err(DomParsedHtmlInstallError::StaleDestination { .. })
    ));
    assert_eq!(dom.snapshot().stamp(), changed);
    assert_eq!(dom.usage(), usage);
}

#[test]
fn contextual_svg_five_subject_closure_is_exact_and_one_short_publishes_nothing() {
    let cancel = AtomicBool::new(false);
    let (mut dom, capture, document, host) = contextual_fixture(Namespace::html(), "div");
    let report = fragment_report(&capture, host, "<svg><script/></svg>");
    let before = dom.snapshot();
    let usage = dom.usage();
    // Three returned owned subjects (Fragment, SVG, Script) plus the actual
    // captured context Element and target Document; bootstrap is not a subject.
    let short = DomParsedHtmlInstallLimits::new(4, 10_000_000, 16_000_000, 100_000_000);
    assert!(matches!(
        dom.install_parsed_html_fragment(report.clone(), &capture, short, &cancel),
        Err(DomParsedHtmlInstallError::ResourceLimit {
            resource: DomParsedHtmlInstallResource::MappedNodes,
            limit: 4,
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let exact = DomParsedHtmlInstallLimits::new(5, 10_000_000, 16_000_000, 100_000_000);
    let installed = dom
        .install_parsed_html_fragment(report, &capture, exact, &cancel)
        .unwrap();
    assert_eq!(installed.bindings().len(), 3);
    assert_eq!(installed.captured_bindings().len(), 2);
    assert_eq!(
        installed.captured_node_for(&capture.subject(host).unwrap()),
        Some(host)
    );
    assert_eq!(
        installed.captured_node_for(&capture.subject(document.node()).unwrap()),
        Some(document.node())
    );
    assert!(before.node(host).unwrap().children().is_empty());
    assert!(
        installed
            .snapshot()
            .node(host)
            .unwrap()
            .children()
            .is_empty()
    );
}

#[test]
fn declarative_shadow_svg_preserves_captured_and_owned_effect_state_until_real_connection() {
    use surgeist_html_parser::{HtmlParseEffect, HtmlParseEffectHost};

    let cancel = AtomicBool::new(false);
    for captured_host in [true, false] {
        let (mut dom, capture, document, context_host) =
            contextual_fixture(Namespace::html(), "div");
        let text = if captured_host {
            "<template shadowrootmode=open shadowrootclonable><svg><script/></svg></template>"
        } else {
            "<section><template shadowrootmode=closed shadowrootclonable><svg><script></script></svg></template></section>"
        };
        let context = HtmlFragmentContext::element(
            capture.capture(),
            &capture.subject(context_host).unwrap(),
        )
        .unwrap();
        let report = Arc::new(
            HtmlFragmentParseRequest::with_options(
                HtmlSource::utf8(text),
                context,
                parse_limits(),
                HtmlParserScriptingMode::Inert,
                true,
            )
            .unwrap()
            .parse()
            .unwrap(),
        );
        assert_eq!(
            report.effective_scripting_mode(),
            HtmlParserScriptingMode::Disabled
        );
        assert!(report.allow_declarative_shadow_roots());
        let scripts = source_scripts(&report);
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].1, HtmlParseSvgScriptOrigin::Parser);
        let source_script = scripts[0].0.clone();
        let document_subject = capture.subject(document.node()).unwrap();
        let context_subject = capture.subject(context_host).unwrap();
        let mut source_root = None;
        let mut source_template = None;
        let mut source_host = None;
        let mut attachment_count = 0;
        let effects = report.effects().unwrap();
        for index in 0..effects.len() {
            if let HtmlParseEffect::AttachShadow {
                host,
                template,
                contents,
                properties,
                declarative,
                available_to_internals,
                ..
            } = effects.effect(index).unwrap()
            {
                attachment_count += 1;
                assert!(declarative);
                assert!(available_to_internals);
                assert!(properties.clonable());
                assert_eq!(
                    properties.mode(),
                    if captured_host {
                        HtmlShadowRootMode::Open
                    } else {
                        HtmlShadowRootMode::Closed
                    }
                );
                match host {
                    HtmlParseEffectHost::Captured(host) => {
                        assert!(captured_host);
                        assert_eq!(host.subject(), &context_subject);
                    }
                    HtmlParseEffectHost::Owned(host) => {
                        assert!(!captured_host);
                        source_host = Some(host.id());
                    }
                }
                source_root = Some(contents.id());
                source_template = Some(template.id());
            }
        }
        assert_eq!(attachment_count, 1);
        let source_root = source_root.unwrap();
        let source_template = source_template.unwrap();
        let mut source_svg = None;
        let mut current_document_seen = false;
        let mut cursor = report.metadata();
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(row) if row.node().id() == source_script => {
                    source_svg = row.node().parent();
                    if let Some(current) = row.node_document() {
                        let HtmlParseMetadataDocument::Captured(current) = current else {
                            panic!("DSD Script keeps the actual captured target nodeDocument");
                        };
                        assert_eq!(current.id(), document_subject);
                        current_document_seen = true;
                    }
                }
                HtmlParseMetadataStep::Complete => break,
                _ => {}
            }
        }
        assert!(current_document_seen);
        let source_svg = source_svg.unwrap();
        let before = dom.snapshot();
        assert_eq!(
            before.node(context_host).unwrap().hosted_shadow_root(),
            None
        );
        let installed = dom
            .install_parsed_html_fragment(report.clone(), &capture, install_limits(), &cancel)
            .unwrap();
        let snapshot = installed.snapshot();
        let host = if captured_host {
            context_host
        } else {
            installed.node_for(&source_host.unwrap()).unwrap()
        };
        let shadow = installed.node_for(&source_root).unwrap();
        let template = installed.node_for(&source_template).unwrap();
        let script = installed.node_for(&source_script).unwrap();
        let svg = installed.node_for(&source_svg).unwrap();
        assert!(std::ptr::eq(installed.report(), report.as_ref()));
        assert_eq!(
            installed.captured_node_for(&document_subject),
            Some(document.node())
        );
        assert_eq!(
            installed.captured_node_for(&context_subject),
            Some(context_host)
        );
        assert_eq!(
            snapshot.node(host).unwrap().hosted_shadow_root(),
            Some(shadow)
        );
        let root = snapshot.shadow_root(shadow).unwrap();
        assert_eq!(root.host(), host);
        assert!(root.state().declarative());
        assert!(root.state().available_to_element_internals());
        assert!(root.state().clonable());
        assert_eq!(
            root.state().mode(),
            if captured_host {
                HtmlShadowRootMode::Open
            } else {
                HtmlShadowRootMode::Closed
            }
        );
        assert_eq!(root.node().shadow_host(), Some(host));
        assert_eq!(root.node().parent(), None);
        assert_eq!(root.node().owner_document(), Some(document));
        assert_eq!(root.node().children(), &[svg]);
        assert_eq!(snapshot.node(svg).unwrap().parent(), Some(shadow));
        assert_eq!(snapshot.node(svg).unwrap().children(), &[script]);
        assert_eq!(
            snapshot.node(script).unwrap().owner_document(),
            Some(document)
        );
        assert_eq!(
            snapshot
                .node(template)
                .unwrap()
                .as_template()
                .unwrap()
                .contents()
                .unwrap(),
            shadow
        );
        assert_eq!(snapshot.node(script).unwrap().is_connected(), captured_host);
        let original = snapshot.node(script).unwrap().as_svg_script().unwrap();
        // Retained attachment may establish canonical shadow connectivity, but
        // materialization does not replay the source's connection hooks.
        assert!(!original.flags().already_processed());
        assert_eq!(original.origin(), DomSvgScriptOrigin::Parser);
        assert_eq!(
            before.node(context_host).unwrap().hosted_shadow_root(),
            None
        );
        assert!(before.node(context_host).unwrap().children().is_empty());
        if captured_host {
            assert!(
                snapshot
                    .node(installed.root())
                    .unwrap()
                    .children()
                    .is_empty()
            );
            let mut detach = DomBatch::new();
            detach.detach(svg);
            dom.apply(detach, &cancel).unwrap();
            let detached = dom.snapshot();
            assert!(!detached.node(script).unwrap().is_connected());
            assert!(
                !detached
                    .node(script)
                    .unwrap()
                    .as_svg_script()
                    .unwrap()
                    .flags()
                    .already_processed()
            );
            let mut insert = DomBatch::new();
            insert.insert_before(svg, shadow, None);
            dom.apply(insert, &cancel).unwrap();
        } else {
            assert_eq!(
                snapshot.node(host).unwrap().parent(),
                Some(installed.root())
            );
            assert_eq!(snapshot.node(installed.root()).unwrap().children(), &[host]);
            let mut insert = DomBatch::new();
            insert.insert_before(host, context_host, None);
            dom.apply(insert, &cancel).unwrap();
        }
        let connected = dom.snapshot();
        let current = connected.node(script).unwrap().as_svg_script().unwrap();
        assert!(connected.node(script).unwrap().is_connected());
        assert!(current.flags().already_processed());
        assert_eq!(current.origin(), DomSvgScriptOrigin::Parser);
        assert_eq!(
            connected.node(script).unwrap().owner_document(),
            Some(document)
        );
        assert!(!original.flags().already_processed());
        assert_eq!(installed.node_for(&source_script), Some(script));
        assert_eq!(installed.node_for(&source_root), Some(shadow));
        // A second actual remove/insert preserves the established true bit.
        let mut repeat = DomBatch::new();
        repeat.detach(svg);
        repeat.insert_before(svg, shadow, None);
        dom.apply(repeat, &cancel).unwrap();
        let latest = dom.snapshot();
        let repeated = latest.node(script).unwrap().as_svg_script().unwrap();
        assert!(repeated.flags().already_processed());
        assert_eq!(repeated.origin(), DomSvgScriptOrigin::Parser);
        assert!(!original.flags().already_processed());
    }
}
