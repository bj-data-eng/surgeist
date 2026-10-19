use super::*;
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};

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
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let environment = environment(&mut dom);
    let report = report();
    // This literal has seven source subjects and seventeen metadata records.
    // The selected geometric banks hold eight Rows while metadata grows from
    // sixteen to thirty-two. Both metadata allocations coexist during copying;
    // the peak is their sum, not a final-occupancy receipt from the allocator.
    let mut cursor = report.metadata();
    let mut records = 0;
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(_) => records += 1,
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(records, 17);
    let peak = 8 * size_of::<Row<'_>>() + (16 + 32) * size_of::<HtmlParseMetadata<'_>>();
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
    use surgeist_html::{HtmlInputEmptySelectionSupport, HtmlInputNoneDirectionSupport};
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
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let host = batch.create_element_ns(&doc, crate::Namespace::html(), "div");
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
            crate::DomHtmlCaptureLimits::new(256, 1_000_000),
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
    let descriptor = source.fragment_parser_document().unwrap();
    let current = crate::DomDocumentEnvironment::new(crate::DomNativeInputPlatform::new(
        HtmlInputEmptySelectionSupport::Unsupported,
        HtmlInputNoneDirectionSupport::Unsupported,
    ));
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
        &independent.fragment_parser_document().unwrap(),
        current,
        independent
            .fragment_parser_document()
            .unwrap()
            .document()
            .document_design_mode_enabled()
            .unwrap(),
    );
    assert_eq!(first.origin(), repeated.origin());
    assert_ne!(first.origin(), separate.origin());
    assert_ne!(first.origin(), target_metadata.origin());
    assert_eq!(first.class(), crate::DomDocumentClass::Html);
    assert_eq!(
        first.content_type(),
        &DocumentString::from("application/xml")
    );
    assert_eq!(first.url(), &DocumentString::from("about:blank"));
    assert_eq!(first.encoding(), &DocumentString::from("UTF-8"));
    assert!(!first.allow_declarative_shadow_roots());
    assert_eq!(
        first.creation_cause(),
        &crate::DomDocumentCreationCause::ParsedFragmentParser
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
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
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
        crate::DomNativeInputPlatform::headless(),
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
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
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
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
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
        Some(surgeist_html::HtmlReferrerPolicy::NoReferrer)
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
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
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
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let host = batch.create_element_ns(&doc, crate::Namespace::html(), "div");
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
            crate::DomHtmlCaptureLimits::new(256, 1_000_000),
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
            surgeist_html::HtmlParserScriptingMode::Inert,
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
        Some(crate::template::TemplateDocument::Ordinary { inert: Some(_) })
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
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
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
    let host = batch.create_element_ns(&doc, crate::Namespace::html(), "div");
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
            crate::DomHtmlCaptureLimits::new(4096, 10_000_000)
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
            surgeist_html::HtmlValueInterpreter::new(65536),
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
    use crate::store::canonical::InteractiveView as DocumentInteractiveState;
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
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
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
                Some(crate::store::canonical::AutofocusView::Unavailable)
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
