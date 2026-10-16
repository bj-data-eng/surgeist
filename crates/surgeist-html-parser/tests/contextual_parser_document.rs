#![forbid(unsafe_code)]
use surgeist_html::*;
use surgeist_html_parser::*;
use surgeist_html_semantics::*;

fn limits() -> HtmlParseLimits {
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

fn fragment_context(mode: HtmlDocumentMode) -> HtmlFragmentContext {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            mode,
            HtmlScriptingStatus::Enabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    let div = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&div).unwrap();
    builder.establish_no_shadow_root(&div).unwrap();
    let capture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap()
}

// HTML877d §13.4 creates a new HTML-class Document and copies mode only.
// DOM constructor defaults remain application/xml, UTF-8, about:blank, fresh
// opaque origin, Null registry and false Document allow-DSD. Parser permission
// and captured target/context identities are separate inputs.
#[test]
fn fragment_descriptor_keeps_actual_constructor_environment_and_completion() {
    for mode in [
        HtmlDocumentMode::NoQuirks,
        HtmlDocumentMode::Quirks,
        HtmlDocumentMode::LimitedQuirks,
    ] {
        let request = HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(
                "<script></script><style>p{}</style><template><style>x{}</style></template>",
            ),
            fragment_context(mode),
            limits(),
            HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap();
        let source = request.source().identity().clone();
        let context = request.context().capture().identity().clone();
        let report = request.parse().unwrap();
        assert_eq!(
            report.effective_scripting_mode(),
            HtmlParserScriptingMode::Inert
        );
        assert!(report.document().is_none());
        assert!(report.document_state().is_none());
        assert!(report.allow_declarative_shadow_roots());
        let descriptor = report
            .fragment_parser_document()
            .expect("actual completed parser Document");
        assert_eq!(descriptor.source().identity(), &source);
        assert_eq!(descriptor.context().capture().identity(), &context);
        assert_eq!(descriptor.mode(), mode);
        assert_eq!(
            descriptor.completion().completion(),
            HtmlParseCompletion::Complete
        );
        assert_eq!(
            descriptor.completion().readiness(),
            HtmlParseReadiness::Complete
        );
        assert!(descriptor.completion().ready_for_post_load_tasks());
        let document = descriptor.document();
        assert!(matches!(document.kind(), HtmlNodeKind::Document(m) if *m == mode));
        let children: Vec<_> = document.children().collect();
        assert_eq!(children.len(), 1);
        let html = report.fragment().unwrap().node(&children[0]).unwrap();
        assert!(
            matches!(html.kind(), HtmlNodeKind::Element(e) if e.name().local_name().code_units() == [104,116,109,108])
        );
        assert_eq!(html.children().len(), 0);
        let mut scripts = 0;
        let mut styles = 0;
        let mut meta_root = false;
        let mut common_root = false;
        let mut cursor = report.metadata();
        loop {
            match cursor.advance() {
                HtmlParseMetadataStep::Record(row) => {
                    if row.node().id() == document.id() {
                        assert!(row.common_state().is_none());
                    }
                    if row.creation().is_some() && row.node().id() == html.id() {
                        let common = row
                            .common_state()
                            .expect("actual implicit html common state");
                        assert_eq!(common.scope(), HtmlCommonScope::Html);
                        common_root = true;
                    }
                    if let Some((Some(HtmlParseMetadataDocument::Owned(parser)), _)) =
                        row.script_documents()
                    {
                        assert_eq!(parser.id(), document.id());
                        scripts += 1;
                    }
                    if let Some(style) = row.style() {
                        let Some(HtmlParseMetadataDocument::Owned(parser)) =
                            style.creating_parser_document()
                        else {
                            panic!("actual parser Document");
                        };
                        assert_eq!(parser.id(), document.id());
                        assert!(style.state().created_by_document_parser());
                        assert!(!style.state().on_parser_stack());
                        styles += 1;
                    }
                    if let Some(meta) = row.document_meta()
                        && meta.document().id() == document.id()
                    {
                        assert!(meta.pragma_language().is_none());
                        assert_eq!(
                            meta.referrer_policy(),
                            HtmlReferrerPolicy::StrictOriginWhenCrossOrigin
                        );
                        assert!(matches!(meta.refresh(), HtmlParseMetaRefresh::Known(false)));
                        assert!(meta.first_missing().is_none());
                        meta_root = true;
                    }
                }
                HtmlParseMetadataStep::Skipped => (),
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert_eq!(
            (scripts, styles, meta_root, common_root),
            (1, 2, true, true)
        );
    }
}

#[test]
fn fragment_origin_is_once_created_and_independent_of_equal_source_or_capture() {
    let context = fragment_context(HtmlDocumentMode::NoQuirks);
    let source = HtmlSource::utf8("<style>x{}</style><style>y{}</style>");
    let parse = || {
        HtmlFragmentParseRequest::with_options(
            source.clone(),
            context.clone(),
            limits(),
            HtmlParserScriptingMode::Inert,
            false,
        )
        .unwrap()
        .parse()
        .unwrap()
    };
    let first = parse();
    let second = parse();
    let origin = first.fragment_parser_document().unwrap().origin().clone();
    assert_eq!(&origin, first.fragment_parser_document().unwrap().origin());
    assert_ne!(&origin, second.fragment_parser_document().unwrap().origin());
    let retained = std::sync::Arc::new(first);
    let alias = retained.clone();
    assert_eq!(
        retained.fragment_parser_document().unwrap().origin(),
        alias.fragment_parser_document().unwrap().origin()
    );
    drop(retained);
    drop(alias);
    assert_eq!(origin, origin.clone());
    assert_ne!(&origin, second.fragment_parser_document().unwrap().origin());
    let ordinary = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>x"), limits())
        .parse()
        .unwrap();
    assert!(ordinary.fragment_parser_document().is_none());
}

#[test]
fn fragment_constructor_and_output_share_one_finite_node_admission_and_cleanup() {
    // The actual §13.4 closure is parser Document + implicit html + returned
    // fragment + p + Text. No head/body/doctype is synthesized by this entry.
    let context = fragment_context(HtmlDocumentMode::NoQuirks);
    let source = HtmlSource::utf8("<p>X</p>");
    let request = |cap| {
        HtmlFragmentParseRequest::with_options(
            source.clone(),
            context.clone(),
            limits().into_builder().managed_nodes(cap).finish().unwrap(),
            HtmlParserScriptingMode::Inert,
            false,
        )
        .unwrap()
    };
    let report = request(5).parse().unwrap();
    assert_eq!(
        report
            .usage()
            .resource(HtmlParseResource::ManagedNodes)
            .current(),
        5
    );
    assert!(report.fragment_parser_document().is_some());
    let failure = request(4).parse().unwrap_err();
    let stop = HtmlParseStop::LimitExceeded {
        resource: HtmlParseResource::ManagedNodes,
        limit: 4,
        current: 4,
        requested_delta: 1,
    };
    assert_eq!(failure.reason(), HtmlParseFailureReason::Stop(stop));
    assert_eq!(failure.first_stop(), Some(stop));
    assert_eq!(failure.source().identity(), source.identity());
    assert_eq!(
        failure.context().unwrap().capture().identity(),
        context.capture().identity()
    );
    assert!(failure.cleanup_error().is_none());
    for resource in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::ManagedEdges,
        HtmlParseResource::ManagedAttributes,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::ContextEffects,
        HtmlParseResource::OpenElements,
    ] {
        assert_eq!(
            failure
                .after_cleanup()
                .unwrap()
                .resource(resource)
                .current(),
            0,
            "{resource:?}"
        );
    }
}
