use super::*;
use surgeist_html::values::DocumentString;
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};

mod shadow_reuse;

fn report() -> Arc<HtmlParseReport> {
    report_for("<!doctype html><p>private</p>")
}
fn report_for(source: &str) -> Arc<HtmlParseReport> {
    Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), parser_limits())
            .parse()
            .unwrap(),
    )
}
fn parser_limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(4096)
        .buffered_utf16_units(4096)
        .token_attributes(256)
        .auxiliary_entries(100_000)
        .active_formatting_entries(100_000)
        .managed_nodes(256)
        .managed_edges(512)
        .managed_attributes(512)
        .managed_utf16_units(4096)
        .source_map_entries(4096)
        .source_map_payload_units(4096)
        .diagnostics(512)
        .diagnostic_payload_units(4096)
        .context_effects(256)
        .open_elements(64)
        .tree_depth(256)
        .managed_capacity_bytes(16_000_000)
        .peak_transient_bytes(16_000_000)
        .atomic_layout_bytes(16_000_000)
        .work_credits(10_000_000)
        .finish()
        .unwrap()
}

#[test]
fn scratch_growth_refuses_the_live_old_and_new_backings_before_changing_the_input() {
    let cancel = AtomicBool::new(false);
    let mut values = Vec::with_capacity(64);
    values.extend(0_u8..64);
    assert_eq!(values.capacity(), 64);
    let mut control = Control {
        limits: Limits::new(256, 10_000_000, 128, 100_000_000),
        work: 0,
        bytes: 64,
        peak_bytes: 64,
        cancel: &cancel,
    };
    // The existing 64-byte allocation is live. Even the smallest replacement
    // that can hold 65 bytes would make 129 simultaneous bytes, exceeding 128.
    assert!(matches!(
        control.reserve(&mut values, 1),
        Err(Error::ResourceLimit {
            resource: Resource::ScratchBytes,
            limit: 128
        })
    ));
    assert_eq!(values.capacity(), 64);
    assert_eq!(values, (0_u8..64).collect::<Vec<_>>());
    assert_eq!(control.bytes, 64);
    assert_eq!(control.work, 0);
}

#[test]
fn scratch_growth_exact_peak_releases_each_old_bank_and_keeps_the_true_highwater() {
    let cancel = AtomicBool::new(false);
    let mut values = Vec::with_capacity(64);
    values.extend(0_u8..64);
    let mut control = Control {
        limits: Limits::new(256, 192, 191, 100_000_000),
        work: 0,
        bytes: 64,
        peak_bytes: 64,
        cancel: &cancel,
    };
    assert!(matches!(
        control.reserve(&mut values, 1),
        Err(Error::ResourceLimit {
            resource: Resource::ScratchBytes,
            limit: 191
        })
    ));
    assert_eq!(control.bytes, 64);
    assert_eq!(control.peak_bytes, 64);
    assert_eq!(control.work, 0);
    control.limits.scratch_bytes = 192;
    control.reserve(&mut values, 1).unwrap();
    assert_eq!(values.capacity(), 128);
    assert_eq!(values, (0_u8..64).collect::<Vec<_>>());
    assert_eq!(control.bytes, 128);
    assert_eq!(control.peak_bytes, 192);
    assert_eq!(control.work, 64);
    values.extend(64_u8..128);
    control.limits.scratch_bytes = 384;
    control.reserve(&mut values, 1).unwrap();
    assert_eq!(values.capacity(), 256);
    assert_eq!(values, (0_u8..128).collect::<Vec<_>>());
    assert_eq!(control.bytes, 256);
    // The first 64-byte bank was released: this is 128+256, not 64+128+256.
    assert_eq!(control.peak_bytes, 384);
    assert_eq!(control.work, 192);
    control.limits.scratch_bytes = 400;
    control.retain(8).unwrap();
    assert_eq!(control.bytes, 264);
    assert_eq!(control.peak_bytes, 384);
}

#[test]
fn scratch_growth_stops_at_cancellation_copy_work_and_representation_before_mutation() {
    let cancel = AtomicBool::new(false);
    let mut values = Vec::with_capacity(64);
    values.extend(0_u8..64);
    let mut control = Control {
        limits: Limits::new(256, 63, 192, 100_000_000),
        work: 0,
        bytes: 64,
        peak_bytes: 64,
        cancel: &cancel,
    };
    cancel.store(true, Ordering::Release);
    assert!(matches!(
        control.reserve(&mut values, 1),
        Err(Error::Cancelled)
    ));
    assert_eq!(control.work, 0);
    cancel.store(false, Ordering::Release);
    assert!(matches!(
        control.reserve(&mut values, usize::MAX),
        Err(Error::RepresentationExhausted)
    ));
    assert_eq!(control.work, 0);
    assert!(matches!(
        control.reserve(&mut values, 1),
        Err(Error::ResourceLimit {
            resource: Resource::Work,
            limit: 63
        })
    ));
    assert_eq!(control.work, 64);
    assert_eq!(control.bytes, 64);
    assert_eq!(control.peak_bytes, 64);
    assert_eq!(values.capacity(), 64);
    assert_eq!(values, (0_u8..64).collect::<Vec<_>>());
}

#[test]
fn scratch_growth_peak_refuses_public_installation_atomically_and_exact_peak_succeeds() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let environment = environment(&mut dom);
    // The `a` element holds the record of its hyperlink state beside the records of the
    // Document and the nonces of `html` and `body`, whose rows are reserved at creation; an
    // element with the creation state and the nonce of its namespace holds no record.
    let report = report_for("<!doctype html><a>private</a>");
    // This literal has seven source subjects and four metadata records: the nonces of `html` and
    // `body`, the hyperlink state of `a` and the Document. The walk of the tree adds the seven
    // subjects before the records are read, so the selected geometric bank holds eight Rows, and
    // the cells of the row map are one for each of the seven nodes of the result, while metadata
    // grows from two to four. Both metadata allocations coexist during copying; the peak is their
    // sum, not a final-occupancy receipt from the allocator. The walk's own scratch is returned
    // before the records are read.
    // The original envelope also reserves the two actual mandatory shared
    // output Vec payload headers before collection. This is future-backing
    // reservation, not a claim that those allocations overlap this copy.
    let mut cursor = report.metadata();
    let mut records = 0;
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            HtmlParseMetadataStep::Record(_) => records += 1,
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(records, 4);
    let nodes = report.document().unwrap().usage().nodes();
    assert_eq!(nodes, 7);
    let peak = 8 * size_of::<Row<'_>>()
        + nodes * size_of::<u32>()
        + (2 + 4) * size_of::<HtmlParseMetadata<'_>>()
        + size_of::<Vec<DomParsedHtmlBinding>>()
        + size_of::<Vec<DomParsedHtmlCapturedBinding>>();
    let before = dom.snapshot();
    let usage = dom.usage();
    let result = dom.install_parsed_html_document(
        before.stamp(),
        report.clone(),
        environment.clone(),
        Limits::new(256, 10_000_000, peak - 1, 100_000_000),
        &cancel,
    );
    assert!(matches!(
        result,
        Err(Error::ResourceLimit { resource: Resource::ScratchBytes, limit }) if limit == (peak - 1) as u64
    ));
    assert_eq!(*dom.state, *before.state);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    let installed = dom
        .install_parsed_html_document(
            before.stamp(),
            report,
            environment,
            Limits::new(256, 10_000_000, peak, 100_000_000),
            &cancel,
        )
        .unwrap();
    assert_eq!(installed.bindings().len(), 7);
}

#[test]
fn fragment_parser_constructor_preserves_one_source_origin_and_selects_only_destination_platform() {
    use surgeist_html::elements::input::{
        HtmlInputEmptySelectionSupport, HtmlInputNoneDirectionSupport,
    };
    use surgeist_html_parser::HtmlFragmentParseRequest;
    use surgeist_html_semantics::{HtmlCaptureLimits, HtmlFragmentContext};
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let host = batch.create_element_ns(&doc, surgeist_html::values::Namespace::html(), "div");
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&doc).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let target_metadata = snapshot.document(document).unwrap().metadata().clone();
    let capture = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(256, 1024, 1024, 65536, 256, 0, 0, 16_777_216),
            crate::internal::semantic::DomHtmlCaptureLimits::new(256, 1_000_000),
            &cancel,
        )
        .unwrap();
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let parse = || {
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<style>a{}</style>"),
            context.clone(),
            parser_limits(),
        )
        .unwrap()
        .parse()
        .unwrap()
    };
    let source = parse();
    let descriptor = source.fragment_parser_document().unwrap().unwrap();
    let current = crate::internal::document::DomDocumentEnvironment::new(
        crate::internal::document::DomNativeInputPlatform::new(
            HtmlInputEmptySelectionSupport::Unsupported,
            HtmlInputNoneDirectionSupport::Unsupported,
        ),
    );
    let first = DomDocumentMetadata::from_fragment_parser(
        &descriptor,
        current.clone(),
        descriptor
            .document()
            .document_design_mode_enabled()
            .unwrap(),
    );
    let repeated = DomDocumentMetadata::from_fragment_parser(
        &descriptor,
        current.clone(),
        descriptor
            .document()
            .document_design_mode_enabled()
            .unwrap(),
    );
    let independent = parse();
    let separate = DomDocumentMetadata::from_fragment_parser(
        &independent.fragment_parser_document().unwrap().unwrap(),
        current,
        independent
            .fragment_parser_document()
            .unwrap()
            .unwrap()
            .document()
            .document_design_mode_enabled()
            .unwrap(),
    );
    assert_eq!(first.origin(), repeated.origin());
    assert_ne!(first.origin(), separate.origin());
    assert_ne!(first.origin(), target_metadata.origin());
    assert_eq!(
        first.class(),
        crate::internal::document::DomDocumentClass::Html
    );
    assert_eq!(
        first.content_type(),
        &DocumentString::from("application/xml")
    );
    assert_eq!(first.url(), &DocumentString::from("about:blank"));
    assert_eq!(first.encoding(), &DocumentString::from("UTF-8"));
    assert!(!first.allow_declarative_shadow_roots());
    assert_eq!(
        first.creation_cause(),
        &crate::internal::document::DomDocumentCreationCause::ParsedFragmentParser
    );
    assert_eq!(
        first
            .document_environment()
            .input_platform()
            .empty_selection(),
        HtmlInputEmptySelectionSupport::Unsupported
    );
    assert_eq!(
        first
            .document_environment()
            .input_platform()
            .none_direction(),
        HtmlInputNoneDirectionSupport::Unsupported
    );
    assert_eq!(
        snapshot.document(document).unwrap().metadata(),
        &target_metadata
    );
    assert_eq!(
        target_metadata
            .document_environment()
            .input_platform()
            .empty_selection(),
        HtmlInputEmptySelectionSupport::Supported
    );
}

#[test]
fn actual_img_generation_maps_once_and_the_next_native_update_does_not_restart_it() {
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let environment = environment(&mut dom);
    let source = report_for("<!doctype html><img src=x>");
    let mut cursor = source.metadata();
    let mut subject = None;
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
            HtmlParseMetadataStep::Record(row) => {
                if let Some(image) = row.image() {
                    assert!(subject.is_none());
                    // Actual creation/relevant-mutation prefix; no activation
                    // or queued continuation has occurred in this profile.
                    assert_eq!(image.latest_started(), 2);
                    subject = Some(image.subject().id());
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            source,
            environment,
            Limits::new(256, 10_000_000, 16_000_000, 100_000_000),
            &AtomicBool::new(false),
        )
        .unwrap();
    let id = installed.node_for(&subject.unwrap()).unwrap();
    let imported = installed.snapshot().node(id).unwrap().node.image().unwrap();
    assert_eq!(imported.high_water, 2);
    assert_eq!(imported.waiting.unwrap().invocation, 2);
    let mut batch = DomBatch::new();
    batch.set_attribute(id, "src", "x");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    let next = current.node(id).unwrap().node.image().unwrap();
    assert_eq!(next.high_water, 3);
    assert_eq!(next.waiting.unwrap().invocation, 3);
    assert_eq!(imported.high_water, 2);
    assert_eq!(imported.waiting.unwrap().invocation, 2);
}
fn environment(dom: &mut Dom) -> DomParsedDocumentEnvironment {
    use surgeist_html_semantics::{
        HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
        HtmlBaseUrlScheme,
    };
    let mut b = DomBatch::new();
    let source = b.create_document();
    let commit = dom.apply(b, &AtomicBool::new(false)).unwrap();
    let origin = dom
        .snapshot()
        .document(commit.document(&source).unwrap())
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let url = HtmlBaseUrlRecord::observed(
        revision,
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        true,
        None,
    )
    .unwrap();
    DomParsedDocumentEnvironment::headless(
        origin,
        url,
        HtmlBaseEncoding::fresh(),
        "UTF-8".into(),
        crate::internal::document::DomNativeInputPlatform::headless(),
    )
}

#[test]
fn complete_private_graph_and_final_ids_do_not_escape_cancelled_or_exhausted_publication() {
    for exhausted in [false, true] {
        let mut dom = Dom::new(
            DomLimits::new(
                256,
                65536,
                256,
                1024,
                1024,
                16,
                1_000_000,
                crate::internal::name_limits::DomNameLimits::new(
                    16_777_216, 16_384, 4_000_000, 1_048_576,
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let environment = environment(&mut dom);
        if exhausted {
            dom.revision = DomRevision(u64::MAX);
        }
        let before = dom.snapshot();
        let usage = dom.usage();
        let cancel = AtomicBool::new(false);
        let pending = dom
            .prepare_parsed_html_installation(
                before.stamp(),
                report(),
                environment,
                Limits::new(256, 10_000_000, 16_000_000, 100_000_000),
                &cancel,
            )
            .unwrap();
        assert!(!pending.bindings.is_empty());
        for binding in &pending.bindings {
            assert!(before.node(binding.node()).is_err());
        }
        assert!(before.document(pending.document).is_err());
        if !exhausted {
            cancel.store(true, Ordering::Release);
        }
        let error = dom
            .publish_parsed_html_installation(pending, &cancel)
            .unwrap_err();
        assert!(matches!(
            (exhausted, error),
            (false, Error::Cancelled) | (true, Error::Target(DomError::RevisionExhausted))
        ));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
}

#[test]
fn actual_staged_meta_history_is_not_published_when_final_cancellation_wins() {
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let environment = environment(&mut dom);
    let before = dom.snapshot();
    let usage = dom.usage();
    let cancel = AtomicBool::new(false);
    let pending = dom
        .prepare_parsed_html_installation(
            before.stamp(),
            report_for("<!doctype html><meta http-equiv=content-language content=ja><meta name=referrer content=no-referrer><p>X"),
            environment,
            Limits::new(256, 10_000_000, 16_000_000, 100_000_000),
            &cancel,
        )
        .unwrap();
    let staged = pending
        .prepared
        .state
        .node(dom.owner, pending.document.node())
        .unwrap();
    let DomNodeKind::Document(metadata) = staged.kind() else {
        panic!("real staged Document");
    };
    assert_eq!(
        metadata.meta_state().unwrap().pragma_language(),
        Some(&DocumentString::from("ja"))
    );
    assert_eq!(
        metadata.policy_container().referrer_policy(),
        Some(surgeist_html::attributes::HtmlReferrerPolicy::NoReferrer)
    );
    cancel.store(true, Ordering::Release);
    assert!(matches!(
        dom.publish_parsed_html_installation(pending, &cancel),
        Err(Error::Cancelled)
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(*dom.state, *before.state);
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn complete_adapter_work_and_real_subject_closure_admit_exact_and_refuse_one_short_atomically() {
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let environment = environment(&mut dom);
    let source = report();
    let before = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let full = dom
        .prepare_parsed_html_installation(
            before.stamp(),
            source.clone(),
            environment.clone(),
            Limits::new(256, 10_000_000, 16_000_000, 100_000_000),
            &cancel,
        )
        .unwrap();
    let work = full.admitted_work;
    let scratch = full.admitted_scratch_bytes;
    // Literal Document/doctype/html/head/body/p/Text, without an auxiliary or
    // native skeleton. Work comes from the actual private admission register,
    // not a copied formula or public instrumentation API.
    assert_eq!(full.bindings.len(), 7);
    drop(full);
    for (selected, resource, succeeds) in [
        (
            Limits::new(7, work, scratch, 100_000_000),
            Resource::Work,
            true,
        ),
        (
            Limits::new(7, work - 1, 16_000_000, 100_000_000),
            Resource::Work,
            false,
        ),
        (
            Limits::new(6, work, 16_000_000, 100_000_000),
            Resource::MappedNodes,
            false,
        ),
        (
            Limits::new(7, work, scratch - 1, 100_000_000),
            Resource::ScratchBytes,
            false,
        ),
    ] {
        let result = dom.prepare_parsed_html_installation(
            before.stamp(),
            source.clone(),
            environment.clone(),
            selected,
            &cancel,
        );
        if succeeds {
            assert_eq!(result.unwrap().bindings.len(), 7);
        } else {
            assert!(
                matches!(result, Err(Error::ResourceLimit { resource: actual, .. }) if actual == resource)
            );
        }
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
}

#[test]
fn contextual_prepared_effects_and_complete_admission_never_publish_on_final_stop() {
    use surgeist_html_parser::HtmlFragmentParseRequest;
    use surgeist_html_semantics::{HtmlCaptureLimits, HtmlFragmentContext};
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let host = batch.create_element_ns(&doc, surgeist_html::values::Namespace::html(), "div");
    batch.insert_before(&host, &doc, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&doc).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let capture = before
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(256, 1024, 1024, 65536, 256, 0, 0, 16_777_216),
            crate::internal::semantic::DomHtmlCaptureLimits::new(256, 1_000_000),
            &cancel,
        )
        .unwrap();
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let report = Arc::new(
        HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(
                "<template shadowrootmode=open><p>S</p></template><template><b>T</b></template>",
            ),
            context,
            parser_limits(),
            surgeist_html::parsing::HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    let full = Limits::new(256, 10_000_000, 16_000_000, 100_000_000);
    let prepared = dom
        .prepare_parsed_html(
            before.stamp(),
            report.clone(),
            None,
            Some(&capture),
            full,
            &cancel,
        )
        .unwrap();
    let work = prepared.admitted_work;
    let scratch = prepared.admitted_scratch_bytes;
    let mapped = prepared.bindings.len() + prepared.captured_bindings.len();
    assert!(
        prepared
            .prepared
            .state
            .node(dom.owner, host)
            .unwrap()
            .shadow_root()
            .is_some()
    );
    assert!(matches!(
        prepared
            .prepared
            .state
            .node(dom.owner, document.node())
            .unwrap()
            .template_document(),
        Some(crate::internal::template::TemplateDocument::Ordinary { inert: Some(_) })
    ));
    assert!(matches!(
        dom.publish_parsed_html_installation(prepared, &AtomicBool::new(true)),
        Err(Error::Cancelled)
    ));
    for (limits, expected) in [
        (
            Limits::new(mapped, work - 1, 16_000_000, 100_000_000),
            Resource::Work,
        ),
        (
            Limits::new(mapped - 1, work, 16_000_000, 100_000_000),
            Resource::MappedNodes,
        ),
        (
            Limits::new(mapped, work, scratch - 1, 100_000_000),
            Resource::ScratchBytes,
        ),
    ] {
        assert!(
            matches!(dom.prepare_parsed_html(before.stamp(),report.clone(),None,Some(&capture),limits,&cancel),Err(Error::ResourceLimit{resource,..}) if resource==expected)
        );
    }
    let exact = dom
        .prepare_parsed_html(
            before.stamp(),
            report,
            None,
            Some(&capture),
            Limits::new(mapped, work, scratch, 100_000_000),
            &cancel,
        )
        .unwrap();
    drop(exact);
    assert_eq!(*dom.state, *before.state);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
}

#[test]
fn captured_url_join_preserves_first_work_stop_at_every_read_and_exact_lookup_unit() {
    use surgeist_html_parser::HtmlFragmentParseRequest;
    use surgeist_html_semantics::*;
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap()
        .with_base_processing(256, 10_000_000)
        .unwrap()
        .with_encoding_url_processing(256, 10_000_000)
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let host = batch.create_element_ns(&doc, surgeist_html::values::Namespace::html(), "div");
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&doc).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let blank = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        true,
        None,
    )
    .unwrap();
    let encoding = HtmlBaseEncoding::fresh();
    let raw = format!("https://example.test/{}", "a".repeat(64));
    let url = HtmlBaseUrlRecord::observed(
        revision.clone(),
        raw.clone().into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(blank.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![],
        HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
    )
    .unwrap();
    let parsing = HtmlEncodingUrlParseFacts::new(
        blank,
        encoding,
        revision,
        vec![HtmlEncodingUrlParseObservation::new(
            raw.clone().into(),
            HtmlEncodingUrlParseOutcome::Parsed(url),
        )],
        HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let stamp = dom.snapshot().document(document).unwrap().stamp();
    batch.bind_base_facts(stamp, &facts);
    batch.bind_encoding_url_facts(stamp, &parsing);
    dom.apply(batch, &cancel).unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let capture = before
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(4096, 8192, 4096, 1_000_000, 4096, 0, 0, 16_777_216),
            crate::internal::semantic::DomHtmlCaptureLimits::new(4096, 10_000_000)
                .with_base_facts(4096, 65536, 10_000_000),
            &cancel,
        )
        .unwrap();
    let selected = capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(document.node()).unwrap(),
                HtmlSemanticQuery::DocumentEncodingUrlContext,
            )],
            surgeist_html::values::HtmlValueInterpreter::new(65536),
            HtmlSemanticLimits::new(10_000_000, 16, 4096, 0, 10_000_000, 0),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    let HtmlSemanticFact::DocumentEncodingUrlContext(HtmlSemanticValue::Known(selected)) =
        selected.results()[0].fact()
    else {
        panic!("actual canonical context")
    };
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let report = Arc::new(
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8(format!("<div><a href='{raw}'></a></div>")),
            context,
            parser_limits(),
        )
        .unwrap()
        .with_encoding_url_contexts(vec![selected.clone()])
        .with_encoding_url_facts(vec![parsing])
        .parse()
        .unwrap(),
    );
    let mut collection = Control {
        limits: Limits::new(256, 10_000_000, 16_000_000, 100_000_000),
        work: 0,
        bytes: 0,
        peak_bytes: 0,
        cancel: &cancel,
    };
    let source = collect_fragment(&report, &capture, &mut collection).unwrap();
    let draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
    let mut full = Control {
        limits: Limits::new(256, 10_000_000, 16_000_000, 100_000_000),
        work: 0,
        bytes: 0,
        peak_bytes: 0,
        cancel: &cancel,
    };
    qualify_captured_hyperlinks(&source, &draft, &mut full).unwrap();
    let exact = full.work;
    assert!(exact > raw.len() as u64);
    for maximum in 0..=exact {
        let mut control = Control {
            limits: Limits::new(256, maximum, 16_000_000, 100_000_000),
            work: 0,
            bytes: 0,
            peak_bytes: 0,
            cancel: &cancel,
        };
        let result = qualify_captured_hyperlinks(&source, &draft, &mut control);
        if maximum == exact {
            result.unwrap();
        } else {
            assert!(
                matches!(result,Err(Error::ResourceLimit{resource:Resource::Work,limit}) if limit==maximum),
                "cut {maximum}"
            );
        }
        assert_eq!(control.bytes, 0);
        assert_eq!(control.peak_bytes, 0);
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
    }
    cancel.store(true, Ordering::Release);
    let mut control = Control {
        limits: Limits::new(256, exact, 16_000_000, 100_000_000),
        work: 0,
        bytes: 0,
        peak_bytes: 0,
        cancel: &cancel,
    };
    assert!(matches!(
        qualify_captured_hyperlinks(&source, &draft, &mut control),
        Err(Error::Cancelled)
    ));
    assert_eq!(control.work, 0);
    assert_eq!(*dom.state, *before.state);
}

#[test]
fn parsed_dialog_owned_history_is_prepared_before_publication_and_late_stop_is_atomic() {
    use crate::internal::store::canonical::InteractiveView as DocumentInteractiveState;
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let environment = environment(&mut dom);
    let report = report_for(
        "<!doctype html><dialog open returnvalue=content></dialog><template><dialog open></dialog></template>",
    );
    let before = dom.snapshot();
    let usage = dom.usage();
    let full = Limits::new(256, 10_000_000, 16_000_000, 100_000_000);
    let pending = dom
        .prepare_parsed_html_installation(
            before.stamp(),
            report.clone(),
            environment.clone(),
            full,
            &cancel,
        )
        .unwrap();
    let mut documents = 0;
    let mut dialogs = 0;
    for binding in &pending.bindings {
        let node = pending
            .prepared
            .state
            .node(dom.owner, binding.node())
            .unwrap();
        if matches!(node.kind(), DomNodeKind::Document(_)) {
            documents += 1;
            let Some(DocumentInteractiveState::Known {
                open_dialogs,
                top_layer,
                pending_top_layer_removals,
            }) = node.interactive_document()
            else {
                panic!("each actual new root/inert Document has original qualified history");
            };
            assert!(open_dialogs.is_empty());
            assert!(top_layer.is_empty());
            assert!(pending_top_layer_removals.is_empty());
            // The original parser's Dialog-list disposition does not attest
            // its independently retained autofocus history.
            assert!(matches!(
                node.autofocus_document(),
                Some(crate::internal::store::canonical::AutofocusView::Unavailable)
            ));
        }
        if let DomNodeKind::Element(element) = node.kind()
            && let Some(dialog) = element.dialog_state()
        {
            dialogs += 1;
            assert_eq!(dialog.return_value().code_units(), []);
            assert!(!dialog.is_modal());
            assert!(!dialog.has_close_watcher());
        }
    }
    assert_eq!(documents, 2);
    assert_eq!(dialogs, 2);
    let work = pending.admitted_work;
    let scratch = pending.admitted_scratch_bytes;
    let mapped = pending.bindings.len();
    assert!(matches!(
        dom.publish_parsed_html_installation(pending, &AtomicBool::new(true)),
        Err(Error::Cancelled)
    ));
    assert_eq!(*dom.state, *before.state);
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    for (limits, resource) in [
        (
            Limits::new(mapped - 1, work, scratch, 100_000_000),
            Resource::MappedNodes,
        ),
        (
            Limits::new(mapped, work - 1, scratch, 100_000_000),
            Resource::Work,
        ),
        (
            Limits::new(mapped, work, scratch - 1, 100_000_000),
            Resource::ScratchBytes,
        ),
    ] {
        assert!(matches!(
            dom.prepare_parsed_html_installation(before.stamp(), report.clone(), environment.clone(), limits, &cancel),
            Err(Error::ResourceLimit { resource: actual, .. }) if actual == resource
        ));
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
    }
    let exact = dom
        .prepare_parsed_html_installation(
            before.stamp(),
            report,
            environment,
            Limits::new(mapped, work, scratch, 100_000_000),
            &cancel,
        )
        .unwrap();
    let installed = dom
        .publish_parsed_html_installation(exact, &cancel)
        .unwrap();
    assert_eq!(installed.bindings().len(), mapped);
    assert_ne!(installed.snapshot().stamp(), before.stamp());
    for binding in installed.bindings() {
        if let Ok(document) = installed.snapshot().document(DocumentId(binding.node())) {
            assert_eq!(
                document.autofocus().unwrap_err(),
                DomError::AutofocusDocumentStateUnavailable
            );
        }
    }
}

#[test]
fn csp_transfer_prices_real_graph_overlap_and_closing_without_retaining_atomic_scratch() {
    use surgeist_html::security::csp::{
        HtmlCspAssociation, HtmlCspAssociationCause, HtmlCspError, HtmlCspLayout, HtmlCspOwner,
    };
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut source = HtmlCspOwner::new_with(&mut allow).unwrap();
    let original = source.initial_document_with(&mut allow).unwrap();
    let mut destination = HtmlCspOwner::new_with(&mut allow).unwrap();
    let layout = original.retained_layout().unwrap();
    let graph = layout.storage().bytes();
    // The association protocol retains a new occurrence while the incoming
    // graph remains live, plus its documented independent atomic cleanup frame.
    let peak = graph * 2 + size_of::<HtmlCspAssociation>().max(layout.disposal().atomic_bytes());
    let cancel = AtomicBool::new(false);
    let mut control = Control {
        limits: Limits::new(256, 10_000_000, peak - 1, 100_000_000),
        work: 0,
        bytes: 0,
        peak_bytes: 0,
        cancel: &cancel,
    };
    assert!(matches!(
        destination.associate_document_with(
            &original,
            HtmlCspAssociationCause::Installation,
            &mut |action| control.csp(action)
        ),
        Err(HtmlCspError::Interrupted(Error::ResourceLimit {
            resource: Resource::ScratchBytes,
            ..
        }))
    ));
    assert_eq!(
        control.bytes, 0,
        "read/atomic frames are not retained graph custody"
    );
    control.limits.scratch_bytes = peak;
    let transferred = destination
        .associate_document_with(
            &original,
            HtmlCspAssociationCause::Installation,
            &mut |action| control.csp(action),
        )
        .unwrap();
    assert_eq!(control.bytes, graph);
    assert_eq!(control.peak_bytes, peak);
    assert_eq!(transferred.policy(), original.policy());
    let mut disposal = 0_u64;
    layout
        .disposal()
        .charge_with_calls(
            0,
            &mut crate::internal::store::html_credits::Credits(|credits| {
                disposal += credits;
                Ok::<(), crate::internal::store::DomError>(())
            }),
        )
        .unwrap();
    assert!(control.work >= 2 * (10 + 8 + disposal));
    let before = control.bytes;
    let cancelled = AtomicBool::new(true);
    control.cancel = &cancelled;
    assert!(matches!(
        destination.associate_document_with(
            &original,
            HtmlCspAssociationCause::Installation,
            &mut |action| control.csp(action)
        ),
        Err(HtmlCspError::Interrupted(Error::Cancelled))
    ));
    assert_eq!(control.bytes, before);
}

#[test]
fn csp_destination_authority_is_prepared_atomically_and_abandoned_ids_are_not_reissued() {
    use surgeist_html::security::csp::{
        HtmlCspError, HtmlCspLayout, HtmlCspProcessorAuthority, HtmlCspProcessorProfile,
        HtmlCspStorage, HtmlCspUnavailable, HtmlMetaCspOutcome, HtmlMetaCspProcessor,
        HtmlMetaCspRequest,
    };
    struct NoReachedMeta {
        profile: HtmlCspProcessorProfile,
    }
    impl HtmlMetaCspProcessor for NoReachedMeta {
        type Error = ();
        fn profile(&self) -> &HtmlCspProcessorProfile {
            &self.profile
        }
        fn prepare<E>(
            &mut self,
            _: &HtmlMetaCspRequest,
            _: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
            _: &mut impl FnMut(HtmlCspStorage),
        ) -> Result<HtmlMetaCspOutcome<()>, HtmlCspError<E>> {
            Ok(HtmlMetaCspOutcome::Unavailable(
                HtmlCspUnavailable::Processor,
            ))
        }
    }
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let processor_owner = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
    let mut service = NoReachedMeta {
        profile: processor_owner.profile_with(1, &mut allow).unwrap(),
    };
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><p>no reached Meta"),
            parser_limits(),
        )
        .parse_with_meta_csp(&mut service)
        .unwrap(),
    );
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let env = environment(&mut dom);
    let cancel = AtomicBool::new(false);
    let before = dom.snapshot();
    let usage = dom.usage();
    let full = Limits::new(256, 10_000_000, 16_000_000, 100_000_000);
    let pending = dom
        .prepare_parsed_html_installation(
            before.stamp(),
            report.clone(),
            env.clone(),
            full,
            &cancel,
        )
        .unwrap();
    let DomNodeKind::Document(metadata) = pending
        .prepared
        .state
        .node(dom.owner, pending.document.node())
        .unwrap()
        .kind()
    else {
        panic!("staged actual Document")
    };
    let crate::internal::meta::DomDocumentCspState::Associated(association) =
        metadata.policy_container().csp()
    else {
        panic!("formed destination authority")
    };
    let abandoned = association.document().clone();
    assert!(association.policy().profile().is_none());
    assert!(pending.admitted_work > 0);
    assert!(matches!(
        dom.publish_parsed_html_installation(pending, &AtomicBool::new(true)),
        Err(Error::Cancelled)
    ));
    assert_eq!(*dom.state, *before.state);
    assert_eq!(dom.usage(), usage);
    let installed = dom
        .install_parsed_html_document(before.stamp(), report, env, full, &cancel)
        .unwrap();
    let metadata = installed.snapshot().document(installed.document()).unwrap();
    let crate::internal::meta::DomDocumentCspState::Associated(association) =
        metadata.metadata().policy_container().csp()
    else {
        panic!("published destination")
    };
    assert_ne!(
        association.document(),
        &abandoned,
        "operational serials never roll back on refusal"
    );
    crate::internal::store::oracle_tests::assert_full_canonical(&dom);
}

fn text_parser_limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(65_536)
        .buffered_utf16_units(65_536)
        .token_attributes(256)
        .auxiliary_entries(1_000_000)
        .active_formatting_entries(100_000)
        .managed_nodes(4096)
        .managed_edges(4096)
        .managed_attributes(4096)
        .managed_utf16_units(65_536)
        .source_map_entries(65_536)
        .source_map_payload_units(65_536)
        .diagnostics(4096)
        .diagnostic_payload_units(65_536)
        .context_effects(256)
        .open_elements(64)
        .tree_depth(256)
        .managed_capacity_bytes(64_000_000)
        .peak_transient_bytes(64_000_000)
        .atomic_layout_bytes(64_000_000)
        .work_credits(1_000_000_000)
        .finish()
        .unwrap()
}

/// The custody units of a store that installed `html`, the text strings its Text and Comment nodes
/// hold, and the blocks the registry lists.
fn installed_text(html: &str) -> (usize, Vec<DocumentString>, usize) {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            4096,
            1_000_000,
            4096,
            4096,
            4096,
            32,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let environment = environment(&mut dom);
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(html), text_parser_limits())
            .parse()
            .unwrap(),
    );
    dom.install_parsed_html_document(
        dom.snapshot().stamp(),
        report,
        environment,
        crate::internal::parsed_installation::DomParsedHtmlInstallLimits::new(
            4096,
            10_000_000,
            16_000_000,
            100_000_000,
        ),
        &cancel,
    )
    .unwrap();
    let snapshot = dom.snapshot();
    let strings = snapshot
        .nodes()
        .filter_map(|id| snapshot.node(id).ok()?.data().cloned())
        .collect();
    let units = dom.custody_registry.ledger().total().units();
    (units, strings, dom.custody_registry.listed_blocks())
}

#[test]
fn a_parse_installs_its_text_by_length_and_its_one_block_once() {
    // The same elements, with and without the text, so that the custody the text adds stands
    // alone.
    let count = 200;
    let mut with = String::from("<!doctype html><body>");
    let mut without = String::from("<!doctype html><body>");
    for index in 0..count {
        with.push_str(&format!("<p>text {index}</p><!--note {index}-->"));
        without.push_str("<p></p>");
    }
    let (with_units, strings, listed) = installed_text(&with);
    let (without_units, _, none) = installed_text(&without);
    assert_eq!(strings.len(), 2 * count);
    // Every string is a range of the block of the parse, which the registry lists once.
    let block = strings[0]
        .borrowed_block()
        .expect("a text of the parse borrows its block");
    assert!(strings.iter().all(|string| {
        string
            .borrowed_block()
            .is_some_and(|other| other.shares_allocation_with(block))
    }));
    assert_eq!((listed, none), (1, 0));
    // Each text counts its own length, and the block counts its capacity once.
    let lengths: usize = strings.iter().map(DocumentString::len).sum();
    assert_eq!(with_units - without_units, lengths + block.capacity());
}

/// How many times installing `html` asks the report for the document of a tree.
fn tree_document_asks(html: &str) -> usize {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            4096,
            1_000_000,
            4096,
            4096,
            4096,
            32,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let environment = environment(&mut dom);
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(html), text_parser_limits())
            .parse()
            .unwrap(),
    );
    TREE_DOCUMENT_ASKS.with(|asks| asks.set(0));
    dom.install_parsed_html_document(
        dom.snapshot().stamp(),
        report,
        environment,
        crate::internal::parsed_installation::DomParsedHtmlInstallLimits::new(
            4096,
            100_000_000,
            16_000_000,
            100_000_000,
        ),
        &cancel,
    )
    .unwrap();
    TREE_DOCUMENT_ASKS.with(std::cell::Cell::get)
}

/// Every node of a tree has the document of its root, so the installation asks the report for it
/// once for the root and not once for each node, however many nodes the tree holds and however
/// deep it is. The report is asked by the collection of the rows and by the allocation of the
/// nodes, once each for a root, and a document that has only the Document for a root is asked
/// twice.
#[test]
fn an_installation_asks_the_report_for_a_document_once_for_each_root() {
    let wide = |items: usize| format!("<!doctype html><ul>{}</ul>", "<li>x</li>".repeat(items));
    let deep = |depth: usize| {
        format!(
            "<!doctype html>{}x{}",
            "<div>".repeat(depth),
            "</div>".repeat(depth)
        )
    };
    for html in [wide(1), wide(8), wide(64), deep(8), deep(32)] {
        assert_eq!(tree_document_asks(&html), 2, "{html}");
    }
    // A template adds the roots of its contents and of the inert Document that owns them, so three
    // roots are asked twice each however many nodes the contents hold.
    for items in [1, 64] {
        let html = format!(
            "<!doctype html><template>{}</template>",
            "<i>x</i>".repeat(items)
        );
        assert_eq!(tree_document_asks(&html), 6, "{items} items");
    }
}

/// The work an installation of `html` admits, as the installation counts it.
fn admitted_work(html: &str) -> u64 {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            4096,
            1_000_000,
            4096,
            4096,
            4096,
            32,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let environment = environment(&mut dom);
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(html), text_parser_limits())
            .parse()
            .unwrap(),
    );
    let before = dom.snapshot();
    dom.prepare_parsed_html_installation(
        before.stamp(),
        report,
        environment,
        crate::internal::parsed_installation::DomParsedHtmlInstallLimits::new(
            4096,
            100_000_000,
            16_000_000,
            100_000_000,
        ),
        &cancel,
    )
    .unwrap()
    .admitted_work
}

/// The work of an installation is linear in the nodes it installs: each node is charged the same
/// work whatever the size of the document, so a document of twice the items costs less than 2.2
/// times as much, the factor the fixed work of the document and the doubling of a reserved
/// vector leave above 2. A search of the rows or the records of the report for each node makes the
/// work quadratic, and the factor approaches 4.
#[test]
fn installation_work_grows_linearly_with_the_nodes() {
    let wide = |items: usize| format!("<!doctype html><ul>{}</ul>", "<li>x</li>".repeat(items));
    let mut previous = admitted_work(&wide(32));
    for items in [64, 128, 256, 512] {
        let work = admitted_work(&wide(items));
        assert!(
            work * 10 <= previous * 22,
            "{items} items cost {work}, {} items cost {previous}",
            items / 2
        );
        previous = work;
    }
}

/// The nodes a walk of `report` reaches: the Document or fragment root, the children, the
/// contents of every template and the contents of every declarative shadow root, by the record
/// position of each id.
fn reachable_positions(report: &HtmlParseReport) -> Vec<usize> {
    let root = match (report.document(), report.fragment()) {
        (Some(document), _) => document.root(),
        (None, Some(fragment)) => fragment.root(),
        (None, None) => panic!("a report holds a tree"),
    };
    let node = |id: &surgeist_html::construction::HtmlNodeId| match report.document() {
        Some(document) => document.node(id).unwrap(),
        None => report.fragment().unwrap().node(id).unwrap(),
    };
    let mut positions = Vec::new();
    let mut pending = vec![root.id()];
    while let Some(id) = pending.pop() {
        positions.push(id.position());
        let current = node(&id);
        pending.extend(current.children());
        pending.extend(current.template_contents());
        pending.extend(
            current
                .declarative_shadow_root()
                .map(|shadow| shadow.contents()),
        );
    }
    positions
}

/// Every node an installation reads has a record position below the node count of its result,
/// which is the length of the row map, so no id of the result can name a cell the map lacks: the
/// nodes the parser removed or moved, a template's contents, the nodes of a table the parser
/// foster-parented and of an adoption agency run keep the records they were created with.
#[test]
fn every_node_of_a_document_has_a_cell_in_the_row_map() {
    for html in [
        "<!doctype html><p>a<p>b",
        "<!doctype html><b>1<p>2</b>3</p><a name=x>4<a>5",
        "<!doctype html><table>text<tr><td>x</table><b><table><b>y</table>",
        "<!doctype html><template><div><template><i>n</i></template></div></template>",
        "<!doctype html><svg><g><title>t</title></g></svg><math><mi>m</mi></math>",
        "<!doctype html><select><option>a<option>b</select>",
    ] {
        let report = report_for(html);
        let nodes = report.document().unwrap().usage().nodes();
        let positions = reachable_positions(&report);
        let mut seen = vec![false; nodes];
        for position in &positions {
            assert!(*position < nodes, "{html}: position {position} of {nodes}");
            assert!(!seen[*position], "{html}: two nodes at {position}");
            seen[*position] = true;
        }
        // The installation maps every reachable node of the result.
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                4096,
                1_000_000,
                4096,
                4096,
                4096,
                32,
                1_000_000,
                crate::internal::name_limits::DomNameLimits::new(
                    16_777_216, 16_384, 4_000_000, 1_048_576,
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let environment = environment(&mut dom);
        let before = dom.snapshot();
        let prepared = dom
            .prepare_parsed_html_installation(
                before.stamp(),
                report.clone(),
                environment.clone(),
                Limits::new(4096, 100_000_000, 16_000_000, 100_000_000),
                &cancel,
            )
            .unwrap();
        // The inert Document of a template is mapped beside the nodes the walk reaches.
        let mapped = prepared.bindings.len();
        assert!(mapped >= positions.len(), "{html}");
        drop(prepared);
        dom.prepare_parsed_html_installation(
            before.stamp(),
            report.clone(),
            environment.clone(),
            Limits::new(mapped, 100_000_000, 16_000_000, 100_000_000),
            &cancel,
        )
        .unwrap();
        assert!(matches!(
            dom.prepare_parsed_html_installation(
                before.stamp(),
                report,
                environment,
                Limits::new(mapped - 1, 100_000_000, 16_000_000, 100_000_000),
                &cancel,
            ),
            Err(Error::ResourceLimit {
                resource: Resource::MappedNodes,
                ..
            })
        ));
    }
}

/// The same holds for the trees of a contextual parse: the implicit root, the nodes of the
/// output fragment, the contents of a template and of a declarative shadow root.
#[test]
fn every_node_of_a_fragment_has_a_cell_in_the_row_map() {
    use surgeist_html_parser::HtmlFragmentParseRequest;
    use surgeist_html_semantics::{HtmlCaptureLimits, HtmlFragmentContext};
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65536,
            256,
            1024,
            1024,
            16,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let host = batch.create_element_ns(&doc, surgeist_html::values::Namespace::html(), "div");
    batch.insert_before(&host, &doc, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&doc).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let before = dom.snapshot();
    let capture = before
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(256, 1024, 1024, 65536, 256, 0, 0, 16_777_216),
            crate::internal::semantic::DomHtmlCaptureLimits::new(256, 1_000_000),
            &cancel,
        )
        .unwrap();
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let report = Arc::new(
        HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(
                "<template shadowrootmode=open><p>S</p></template><template><b>T</b></template>\
                 <i>x</i>",
            ),
            context,
            parser_limits(),
            surgeist_html::parsing::HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    let nodes = report.fragment().unwrap().usage().nodes();
    let positions = reachable_positions(&report);
    assert!(positions.iter().all(|&position| position < nodes));
    let mut sorted = positions.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), positions.len(), "two nodes share a position");
    let prepared = dom
        .prepare_parsed_html(
            before.stamp(),
            report,
            None,
            Some(&capture),
            Limits::new(256, 10_000_000, 16_000_000, 100_000_000),
            &cancel,
        )
        .unwrap();
    // The nodes of the fragment are mapped by their cells, with the nodes of the parser's own
    // Document and the captured nodes the context names, so the walk reaches no more than the
    // installation maps.
    assert!(prepared.bindings.len() >= positions.len());
    drop(prepared);
    // The rows hold each node the walk reaches once. The elements are in creation order, except
    // the template and the host that the effects of the parse add first, and the contents of the
    // shadow root and of the template follow the nodes of their own trees.
    let mut control = Control {
        limits: Limits::new(256, 100_000_000, 16_000_000, 100_000_000),
        work: 0,
        bytes: 0,
        peak_bytes: 0,
        cancel: &cancel,
    };
    let parsed = Arc::new(
        surgeist_html_parser::HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(
                "<template shadowrootmode=open><p>S</p><b>x</b></template><i>x</i><u>y</u>",
            ),
            HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap())
                .unwrap(),
            parser_limits(),
            surgeist_html::parsing::HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    let source = collect_fragment(&parsed, &capture, &mut control).unwrap();
    let rows: Vec<(usize, bool)> = source
        .rows
        .iter()
        .map(|row| {
            let element = matches!(row.source.kind(), HtmlNodeKind::Element(_));
            (row.source.id().position(), element)
        })
        .collect();
    let mut once: Vec<usize> = rows.iter().map(|row| row.0).collect();
    once.sort_unstable();
    once.dedup();
    assert_eq!(once.len(), rows.len(), "a node has two rows");
    for position in reachable_positions(&parsed) {
        assert!(rows.iter().any(|row| row.0 == position));
    }
    // The implicit `html` root of the parse is an element of the parser's own Document, which
    // no tree of the fragment reaches.
    let elements: Vec<usize> = source
        .rows
        .iter()
        .filter(|row| match row.source.kind() {
            HtmlNodeKind::Element(element) => element.name().local_name() != &"html".into(),
            _ => false,
        })
        .map(|row| row.source.id().position())
        .collect();
    // The template comes with the effects, and every other element follows in creation order.
    assert!(elements.len() >= 5, "{elements:?}");
    assert!(
        elements[1..].windows(2).all(|pair| pair[0] < pair[1]),
        "{elements:?}"
    );
}

/// An id that names no row of a source is not found, whether it names a cell no row holds, a row
/// of another id, or a position past the cells, and a node whose position is past the cells is
/// refused when it is added: nothing is written.
#[test]
fn an_id_without_a_row_is_not_found_and_a_position_past_the_cells_is_refused() {
    let report = report_for("<!doctype html><p>a</p><p>b</p>");
    let other = report_for("<!doctype html><p>a</p><p>b</p>");
    let cancel = AtomicBool::new(false);
    let mut control = Control {
        limits: Limits::new(64, 10_000_000, 16_000_000, 100_000_000),
        work: 0,
        bytes: 0,
        peak_bytes: 0,
        cancel: &cancel,
    };
    let mut source = collect(&report, &mut control).unwrap();
    let document = report.document().unwrap();
    let ids: Vec<_> = {
        let html = document
            .node(&document.root().children().nth(1).unwrap())
            .unwrap();
        let body = document.node(&html.children().nth(1).unwrap()).unwrap();
        body.children().collect()
    };
    let first = source.index(&ids[0], &mut control).unwrap().unwrap();
    assert_eq!(source.rows[first].source.id(), ids[0]);
    // An id of another result names no row of this one, though its position holds a cell.
    let foreign = other.document().unwrap().root().id();
    let before = control.work;
    assert_eq!(source.index(&foreign, &mut control).unwrap(), None);
    assert_eq!(control.work - before, 1, "one step for the cell");
    // A cell that names a row of another node is not that node's row.
    let moved = source.row_of[ids[0].position()];
    source.row_of[ids[1].position()] = moved;
    assert_eq!(source.index(&ids[1], &mut control).unwrap(), None);
    source.row_of[ids[1].position()] = 0;
    assert_eq!(source.index(&ids[1], &mut control).unwrap(), None);
    // The cells cover fewer nodes than the result holds: the node is refused and no row is made.
    source.row_of.truncate(ids[1].position());
    let rows = source.rows.len();
    assert_eq!(source.index(&ids[1], &mut control).unwrap(), None);
    assert!(matches!(
        source.add(ids[1], &mut control),
        Err(Error::RepresentationExhausted)
    ));
    assert_eq!(source.rows.len(), rows);
}

/// The rows of a source hold every node of the trees of the report once, and the nodes of a tree
/// in the order the parser created them, whatever records the report holds, so that elements with
/// no record of their own are keyed with those that have one. Creation order is not tree order: the
/// adoption agency places a node under a parent created after it.
#[test]
fn the_rows_of_a_source_hold_every_node_once_in_creation_order() {
    let cancel = AtomicBool::new(false);
    for html in [
        "<!doctype html><p>a<p>b",
        "<!doctype html><form id=f><input type=radio name=g><input type=radio name=g checked></form>",
        "<!doctype html><b>1<p>2</b>3</p><table>text<tr><td nonce=n>x</table>",
        "<!doctype html><template><div><template><i>n</i></template></div></template><p>t",
        "<!doctype html><html nonce=a><body><html nonce=b><span></span><svg><g nonce=c></g></svg>",
    ] {
        let report = report_for(html);
        let mut control = Control {
            limits: Limits::new(256, 100_000_000, 16_000_000, 100_000_000),
            work: 0,
            bytes: 0,
            peak_bytes: 0,
            cancel: &cancel,
        };
        let source = collect(&report, &mut control).unwrap();
        let reached = reachable_positions(&report);
        let rows: Vec<usize> = source
            .rows
            .iter()
            .map(|row| row.source.id().position())
            .collect();
        let mut once = rows.clone();
        once.sort_unstable();
        once.dedup();
        assert_eq!(once.len(), rows.len(), "{html}: a node has two rows");
        let mut expected = reached.clone();
        expected.sort_unstable();
        // The nodes the walk reaches come first, in ascending order of position, and the rows
        // that follow are the inert Documents of templates, which no tree reaches.
        assert_eq!(rows[..expected.len()], expected[..], "{html}");
        for row in &rows[expected.len()..] {
            assert!(!reached.contains(row), "{html}: node {row} is added twice");
        }
    }
}

/// A node no tree of the report reaches is not installed. The second option of the select makes
/// its content the content of the selectedcontent element, which detaches the clones of the first
/// option's content that the first update made: the report keeps their records, and the
/// installation maps only the nodes a walk of the trees reaches.
#[test]
fn a_node_no_root_reaches_is_not_installed() {
    let report = report_for(
        "<!doctype html><select><button><selectedcontent></selectedcontent></button>\
         <option><b>x</b><option selected>y</select>",
    );
    let reached = reachable_positions(&report);
    let nodes = report.document().unwrap().usage().nodes();
    assert!(
        nodes > reached.len(),
        "the report holds {nodes} records and a walk reaches {}",
        reached.len()
    );
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            4096,
            1_000_000,
            4096,
            4096,
            4096,
            32,
            1_000_000,
            crate::internal::name_limits::DomNameLimits::new(
                16_777_216, 16_384, 4_000_000, 1_048_576,
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let environment = environment(&mut dom);
    let before = dom.snapshot();
    let prepared = dom
        .prepare_parsed_html_installation(
            before.stamp(),
            report,
            environment,
            Limits::new(4096, 100_000_000, 16_000_000, 100_000_000),
            &cancel,
        )
        .unwrap();
    assert_eq!(prepared.bindings.len(), reached.len());
}
