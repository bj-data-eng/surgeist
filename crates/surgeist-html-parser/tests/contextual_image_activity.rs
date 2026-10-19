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

fn context(browsing_context: HtmlBrowsingContextPresence) -> HtmlFragmentContext {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_document_nonce_environment(
            &document,
            HtmlNonceDocumentEnvironment::new(
                browsing_context,
                HtmlHeaderDeliveredCsp::Unavailable,
            ),
        )
        .unwrap();
    builder
        .establish_document_image_support(&document, HtmlImageSupport::Disabled)
        .unwrap();
    builder
        .establish_parser_creation_image_support(&document, HtmlImageSupport::Disabled)
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

fn image_with_document(
    report: &HtmlParseReport,
) -> (HtmlParseImage<'_>, HtmlParseMetadataDocument<'_>) {
    let mut image = None;
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(found) = row.image() {
                    assert!(image.replace(found).is_none(), "one literal Img");
                }
            }
            HtmlParseMetadataStep::Skipped => (),
            HtmlParseMetadataStep::Complete => break,
        }
    }
    let image = image.expect("retained literal Img");
    let mut document = None;
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) if row.node().id() == image.subject().id() => {
                if let Some(found) = row.node_document() {
                    assert!(
                        document.replace(found).is_none(),
                        "one actual node-Document"
                    );
                }
            }
            HtmlParseMetadataStep::Record(_) | HtmlParseMetadataStep::Skipped => (),
            HtmlParseMetadataStep::Complete => break,
        }
    }
    (image, document.expect("actual Img node-Document"))
}

fn assert_unchanged_request(image: &HtmlParseImage<'_>) {
    let state = image.state();
    let request = state.current_request();
    assert_eq!(request.state(), HtmlImgFreshRequestState::Unavailable);
    assert!(request.current_url().is_empty());
    assert_eq!(request.current_pixel_density(), 1);
    assert!(request.preferred_dimensions_are_null());
    assert!(!request.image_data_is_available());
    assert!(state.pending_request_is_null());
    assert!(state.last_selected_source_is_null());
    assert!(state.last_auto_sizes_width_is_null());
    assert!(state.lazy_load_resumption_is_null());
    assert_eq!(
        image.dimension_attribute_source().id(),
        image.subject().id()
    );
}

#[test]
fn fragment_owned_inert_template_document_keeps_first_image_wait() {
    // HTML877d #appropriate-template-contents-owner-document creates an
    // independent inert Document (48440–48450). #fully-active (71922–71926)
    // cannot hold for it. #updating-the-image-data (22218–22227) waits BEFORE
    // inspecting disabled image support. Creation starts exactly one invocation
    // (22091–22093); this literal has no relevant image attribute mutation.
    let report = HtmlFragmentParseRequest::new(
        HtmlSource::utf8("<template><img></template>"),
        context(HtmlBrowsingContextPresence::Present),
        limits(),
    )
    .unwrap()
    .parse()
    .unwrap();
    let (image, document) = image_with_document(&report);
    let HtmlParseMetadataDocument::Owned(document) = document else {
        panic!("Img belongs to the source-created inert Document");
    };
    assert_ne!(
        document.id(),
        report.fragment_parser_document().unwrap().document().id()
    );
    let mut inert = false;
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) if row.node().id() == document.id() => {
                if let Some((class, is_inert, cache)) = row.document_facts() {
                    assert_eq!(class, HtmlDocumentClass::Html);
                    assert!(is_inert);
                    assert!(cache.is_none());
                    inert = true;
                }
            }
            HtmlParseMetadataStep::Record(_) | HtmlParseMetadataStep::Skipped => (),
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert!(inert, "actual retained inert constructor state");
    assert_unchanged_request(&image);
    assert!(image.cause().is_some());
    assert_eq!(image.latest_started(), 1);
    assert_eq!(
        image.update(),
        HtmlParseImageUpdate::Waiting {
            generation: 1,
            flags: HtmlImageUpdateFlags::new(false, false),
        }
    );
}

#[test]
fn fragment_captured_document_without_browsing_context_keeps_latest_image_wait() {
    // HTML877d #fully-active (71922–71926) and #nav-bc (70887–70891): an
    // actual absent browsing context rules out a fully-active node-Document.
    // Present/Unavailable alone do not prove activity. The independent disabled
    // image-support fact cannot bypass the earlier activity guard (22218–22227).
    for browsing_context in [
        HtmlBrowsingContextPresence::Present,
        HtmlBrowsingContextPresence::Unavailable,
        HtmlBrowsingContextPresence::Absent,
    ] {
        let context = context(browsing_context);
        let target = context.target_document().clone();
        let report = HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<img src=x srcset=y>"),
            context,
            limits(),
        )
        .unwrap()
        .parse()
        .unwrap();
        let (image, document) = image_with_document(&report);
        let HtmlParseMetadataDocument::Captured(document) = document else {
            panic!("Img retains the actual captured target Document");
        };
        assert_eq!(document.id(), *target.subject());
        let HtmlCapturedNodeKind::Document(facts) = document.kind() else {
            panic!("actual captured Document");
        };
        assert_eq!(
            facts.nonce_environment().browsing_context(),
            browsing_context
        );
        assert_eq!(facts.image_support(), HtmlImageSupport::Disabled);
        assert_eq!(
            facts.parser_creation_image_support(),
            HtmlImageSupport::Disabled
        );
        assert_unchanged_request(&image);
        assert!(image.cause().is_some());
        // Creation + src SET + srcset SET + adoption (22091–22093,
        // 22110–22116). Creation uses the implicit html's parser Document
        // (95977, 99276–99281); insertion into the returned fragment adopts into
        // the captured target Document (99282–99285). All flags are false/false;
        // only the latest inactive wait survives.
        assert_eq!(image.latest_started(), 4);
        let expected = if browsing_context == HtmlBrowsingContextPresence::Absent {
            HtmlParseImageUpdate::Waiting {
                generation: 4,
                flags: HtmlImageUpdateFlags::new(false, false),
            }
        } else {
            HtmlParseImageUpdate::UnavailableActivity { latest_started: 4 }
        };
        assert_eq!(image.update(), expected, "{browsing_context:?}");
    }
}
