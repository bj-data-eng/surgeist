//! Native XML DOMImplementation creator functional contracts.
//! Literal DOM3071 2584–2610 and Document defaults2110–2114.
//! New-front functional evidence only: no compiler/stub or historical-main RED.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;

fn limits() -> DomLimits {
    DomLimits::new(
        128,
        65536,
        256,
        0,
        0,
        32,
        1_000_000,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn setup(limits: DomLimits) -> (Dom, DocumentId, DocumentId) {
    let mut dom = Dom::new(limits).unwrap();
    let mut b = DomBatch::new();
    let associated = b.create_document();
    let unrelated = b.create_document();
    let c = apply(&mut dom, b);
    (
        dom,
        c.document(&associated).unwrap(),
        c.document(&unrelated).unwrap(),
    )
}
fn xml_defaults(snapshot: &DomSnapshot, doc: DocumentId, associated: DocumentId, mime: &str) {
    let metadata = snapshot.document(doc).unwrap().metadata();
    assert_eq!(metadata.class(), DomDocumentClass::Xml);
    assert_eq!(metadata.content_type(), &DocumentString::from(mime));
    assert_eq!(metadata.encoding(), &DocumentString::from("UTF-8"));
    assert_eq!(metadata.url(), &DocumentString::from("about:blank"));
    assert_eq!(metadata.mode(), HtmlDocumentMode::NoQuirks);
    assert_eq!(metadata.custom_element_registry(), DomRegistryBinding::Null);
    assert!(!metadata.allow_declarative_shadow_roots());
    assert_eq!(
        metadata.origin(),
        snapshot.document(associated).unwrap().metadata().origin()
    );
    let environment = metadata.document_environment();
    assert_eq!(
        environment.browsing_context(),
        surgeist_html::HtmlBrowsingContextPresence::Absent
    );
    assert!(!environment.has_ever_had_navigable());
    assert!(!snapshot.document(doc).unwrap().is_fully_active().unwrap());
}

#[test]
fn namespace_argument_selects_literal_mime_even_without_a_root() {
    for (namespace, mime, units) in [
        (Namespace::none(), "application/xml", 31),
        (Namespace::from_uri(""), "application/xml", 31),
        (Namespace::html(), "application/xhtml+xml", 37),
        (Namespace::svg(), "image/svg+xml", 29),
        (Namespace::from_uri("urn:custom"), "application/xml", 31),
        (
            Namespace::from_uri("HTTP://www.w3.org/1999/xhtml"),
            "application/xml",
            31,
        ),
    ] {
        for name in ["", "MiXeD"] {
            let (mut dom, associated, unrelated) = setup(limits());
            let usage = dom.usage();
            let mut b = DomBatch::new();
            let doc = b.create_xml_document(associated, namespace.clone(), name, None);
            let c = apply(&mut dom, b);
            let doc = c.document(&doc).unwrap();
            let snapshot = dom.snapshot();
            xml_defaults(&snapshot, doc, associated, mime);
            assert_ne!(
                snapshot.document(doc).unwrap().metadata().origin(),
                snapshot.document(unrelated).unwrap().metadata().origin()
            );
            let children = snapshot.node(doc.node()).unwrap().children();
            assert_eq!(children.len(), usize::from(!name.is_empty()));
            if name.is_empty() {
                assert_eq!(dom.usage().managed_nodes(), usage.managed_nodes() + 1);
                assert_eq!(
                    dom.usage().string_code_units(),
                    usage.string_code_units() + units
                );
            } else {
                let root = snapshot.node(children[0]).unwrap();
                assert_eq!(root.owner_document(), Some(doc));
                assert_eq!(root.parent(), Some(doc.node()));
                assert_eq!(root.node_name(), DocumentString::from("MiXeD"));
                let element = root.as_element().unwrap();
                assert_eq!(element.name().namespace_uri(), namespace.uri());
                assert_eq!(element.name().prefix(), None);
                assert_eq!(element.name().local_name(), &DocumentString::from("MiXeD"));
                assert!(element.attributes().is_empty());
                assert_eq!(dom.usage().managed_nodes(), usage.managed_nodes() + 2);
            }
        }
    }
}

#[test]
fn prefixed_case_and_xml_class_survive_xhtml_mime_and_following_ticket() {
    let (mut dom, source, _) = setup(limits());
    let mut b = DomBatch::new();
    let html_associated = b.create_html_document(source, None);
    let xml = b.create_xml_document(&html_associated, Namespace::html(), "p:MiXeD", None);
    let comment = b.create_comment(&xml, "after private root");
    let element = b.create_element(&xml, "NeXt");
    let c = apply(&mut dom, b);
    let associated = c.document(&html_associated).unwrap();
    let doc = c.document(&xml).unwrap();
    let snapshot = dom.snapshot();
    xml_defaults(&snapshot, doc, associated, "application/xhtml+xml");
    let root = snapshot
        .node(snapshot.node(doc.node()).unwrap().children()[0])
        .unwrap();
    assert_eq!(root.node_name(), DocumentString::from("p:MiXeD"));
    assert_eq!(root.tag_name(), Some(DocumentString::from("p:MiXeD")));
    assert_eq!(
        root.as_element().unwrap().name().prefix(),
        Some(&DocumentString::from("p"))
    );
    assert_eq!(
        root.as_element().unwrap().name().local_name(),
        &DocumentString::from("MiXeD")
    );
    let comment = snapshot
        .node(c.created(&comment).unwrap().unwrap())
        .unwrap();
    assert_eq!(
        comment.data(),
        Some(&DocumentString::from("after private root"))
    );
    assert_eq!(comment.owner_document(), Some(doc));
    assert_ne!(comment.id(), root.id());
    let element = snapshot
        .node(c.created(&element).unwrap().unwrap())
        .unwrap();
    assert_eq!(element.node_name(), DocumentString::from("NeXt"));
    assert_eq!(
        element.as_element().unwrap().name().namespace_uri(),
        Namespace::html().uri()
    );
    assert_eq!(element.as_element().unwrap().name().prefix(), None);
    assert_eq!(
        snapshot.document(associated).unwrap().metadata().class(),
        DomDocumentClass::Html
    );
}

#[test]
fn detached_and_attached_doctype_move_actual_identity_before_root_without_copying_origin() {
    for attached in [false, true] {
        let (mut dom, associated, old_doc) = setup(limits());
        let mut b = DomBatch::new();
        let public = DocumentString::from_utf16(vec![0xd800, 0, 65]);
        let system = DocumentString::from_utf16(vec![0xdc00, 13, 10]);
        let dt = b.create_document_type(old_doc, "QuIrKs/=", public.clone(), system.clone());
        if attached {
            b.insert_before(&dt, old_doc, None);
        }
        let c = apply(&mut dom, b);
        let dt = c.created(&dt).unwrap().unwrap();
        let old = dom.snapshot();
        let count = dom.usage().managed_nodes();
        let mut b = DomBatch::new();
        let doc = b.create_xml_document(associated, Namespace::svg(), "s:RoOt", Some(dt.into()));
        let c = apply(&mut dom, b);
        let doc = c.document(&doc).unwrap();
        let snapshot = dom.snapshot();
        xml_defaults(&snapshot, doc, associated, "image/svg+xml");
        let children = snapshot.node(doc.node()).unwrap().children();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0], dt);
        assert_eq!(
            snapshot.node(children[1]).unwrap().node_name(),
            DocumentString::from("s:RoOt")
        );
        assert_eq!(snapshot.node(dt).unwrap().parent(), Some(doc.node()));
        assert_eq!(snapshot.node(dt).unwrap().owner_document(), Some(doc));
        let DomNodeKind::DocumentType(value) = snapshot.node(dt).unwrap().kind() else {
            panic!("actual doctype");
        };
        assert_eq!(value.name(), &DocumentString::from("QuIrKs/="));
        assert_eq!(value.public_id(), &public);
        assert_eq!(value.system_id(), &system);
        assert_eq!(snapshot.node(old_doc.node()).unwrap().children(), []);
        assert_eq!(old.node(dt).unwrap().owner_document(), Some(old_doc));
        assert_eq!(
            old.node(dt).unwrap().parent(),
            attached.then_some(old_doc.node())
        );
        assert_eq!(dom.usage().managed_nodes(), count + 2);
        assert!(c.changes().changes().contains(&DomChange::NodeDocument(dt)));
    }
}

#[test]
fn batch_local_doctype_is_the_same_returned_node_and_empty_name_adds_no_root() {
    let (mut dom, associated, old_doc) = setup(limits());
    let mut b = DomBatch::new();
    let dt = b.create_document_type(old_doc, "root", "public", "system");
    b.insert_before(&dt, old_doc, None);
    let doc = b.create_xml_document(associated, Namespace::html(), "", Some((&dt).into()));
    let c = apply(&mut dom, b);
    let doc = c.document(&doc).unwrap();
    let dt = c.created(&dt).unwrap().unwrap();
    let snapshot = dom.snapshot();
    xml_defaults(&snapshot, doc, associated, "application/xhtml+xml");
    assert_eq!(snapshot.node(doc.node()).unwrap().children(), &[dt]);
    assert_eq!(snapshot.node(dt).unwrap().owner_document(), Some(doc));
    assert_eq!(snapshot.node(old_doc.node()).unwrap().children(), []);
}

#[test]
fn root_name_error_precedes_doctype_effect_and_later_failure_rolls_back_the_whole_batch() {
    for (namespace, name, expected) in [
        (Namespace::none(), "bad name", DomError::InvalidCharacter),
        (Namespace::none(), "p:root", DomError::Namespace),
        (
            Namespace::from_uri("urn:wrong"),
            "xml:root",
            DomError::Namespace,
        ),
    ] {
        let (mut dom, associated, old_doc) = setup(limits());
        let mut b = DomBatch::new();
        let dt = b.create_document_type(old_doc, "root", "", "");
        b.insert_before(&dt, old_doc, None);
        let text = b.create_text(old_doc, "wrong doctype kind");
        let c = apply(&mut dom, b);
        let dt = c.created(&dt).unwrap().unwrap();
        let text = c.created(&text).unwrap().unwrap();
        for supplied in [dt, text] {
            let before = dom.snapshot();
            let usage = dom.usage();
            let mut b = DomBatch::new();
            b.create_comment(old_doc, "earlier unpublished");
            b.create_xml_document(associated, namespace.clone(), name, Some(supplied.into()));
            assert_eq!(dom.apply(b, &AtomicBool::new(false)).unwrap_err(), expected);
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
            assert_eq!(
                dom.snapshot().node(dt).unwrap().parent(),
                Some(old_doc.node())
            );
            assert_eq!(
                dom.snapshot().node(old_doc.node()).unwrap().children(),
                &[dt]
            );
        }
    }
}

#[test]
fn foreign_stale_wrong_kind_and_foreign_batch_operands_do_not_publish_any_constructor_prefix() {
    let (mut dom, associated, old_doc) = setup(limits());
    let mut b = DomBatch::new();
    let dt = b.create_document_type(old_doc, "root", "", "");
    let text = b.create_text(old_doc, "not a DocumentType");
    let c = apply(&mut dom, b);
    let stale = c.created(&dt).unwrap().unwrap();
    let text = c.created(&text).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.destroy(stale);
    apply(&mut dom, b);
    let (mut foreign, foreign_doc, _) = setup(limits());
    let mut b = DomBatch::new();
    let foreign_dt = b.create_document_type(foreign_doc, "root", "", "");
    let c = apply(&mut foreign, b);
    let foreign_dt = c.created(&foreign_dt).unwrap().unwrap();
    let mut other_batch = DomBatch::new();
    let pending = other_batch.create_document_type(old_doc, "root", "", "");
    for (doctype, expected) in [
        (stale.into(), DomError::StaleNode),
        (text.into(), DomError::WrongNodeKind),
        (foreign_dt.into(), DomError::ForeignOwner),
        ((&pending).into(), DomError::ForeignBatch),
    ] {
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.create_comment(old_doc, "partial");
        b.create_xml_document(associated, Namespace::none(), "Root", Some(doctype));
        assert_eq!(dom.apply(b, &AtomicBool::new(false)).unwrap_err(), expected);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
    let mut b = DomBatch::new();
    let retired = b.create_text(old_doc, "retired receiver");
    let c = apply(&mut dom, b);
    let retired = c.created(&retired).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.destroy(retired);
    apply(&mut dom, b);
    for (receiver, expected) in [
        (foreign_doc.node(), DomError::ForeignOwner),
        (text, DomError::WrongNodeKind),
        (retired, DomError::StaleNode),
    ] {
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.create_xml_document(receiver, Namespace::none(), "Root", None);
        assert_eq!(dom.apply(b, &AtomicBool::new(false)).unwrap_err(), expected);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
}

#[test]
fn node_and_string_exact_boundaries_and_cancellation_preserve_old_doctype_topology() {
    // Associated metadata31 + new XML metadata31 + one-unit root=63.
    for (nodes, units, success, resource) in [
        (3, 63, true, DomResource::ManagedNodes),
        (2, 63, false, DomResource::ManagedNodes),
        (3, 62, false, DomResource::StringCodeUnits),
    ] {
        let mut dom = Dom::new(
            DomLimits::new(
                nodes,
                units,
                32,
                0,
                0,
                8,
                65536,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut b = DomBatch::new();
        let associated = b.create_document();
        let c = apply(&mut dom, b);
        let associated = c.document(&associated).unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        let doc = b.create_xml_document(associated, Namespace::none(), "R", None);
        let result = dom.apply(b, &AtomicBool::new(false));
        if success {
            let c = result.unwrap();
            assert_eq!(dom.usage().managed_nodes(), 3);
            assert_eq!(dom.usage().string_code_units(), 63);
            assert_eq!(
                dom.snapshot()
                    .node(c.document(&doc).unwrap().node())
                    .unwrap()
                    .children()
                    .len(),
                1
            );
        } else {
            assert!(
                matches!(result, Err(DomError::ResourceLimit { resource: actual, .. }) if actual == resource)
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
        }
    }
    let (mut dom, associated, old_doc) = setup(limits());
    let mut b = DomBatch::new();
    let dt = b.create_document_type(old_doc, "root", "", "");
    b.insert_before(&dt, old_doc, None);
    let c = apply(&mut dom, b);
    let dt = c.created(&dt).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.create_xml_document(associated, Namespace::none(), "R", Some(dt.into()));
    assert_eq!(
        dom.apply(b, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.snapshot().node(dt).unwrap().parent(),
        Some(old_doc.node())
    );
}

#[test]
fn smaller_svg_mime_does_not_erase_required_initial_xml_metadata_admission() {
    for (cap, success) in [(60, false), (62, true)] {
        let mut dom = Dom::new(
            DomLimits::new(
                8,
                cap,
                16,
                0,
                0,
                8,
                65536,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut b = DomBatch::new();
        let associated = b.create_document();
        let c = apply(&mut dom, b);
        let associated = c.document(&associated).unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        let doc = b.create_xml_document(associated, Namespace::svg(), "", None);
        let result = dom.apply(b, &AtomicBool::new(false));
        if success {
            let c = result.unwrap();
            xml_defaults(
                &dom.snapshot(),
                c.document(&doc).unwrap(),
                associated,
                "image/svg+xml",
            );
            assert_eq!(dom.usage().string_code_units(), 60);
        } else {
            assert!(matches!(
                result,
                Err(DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    ..
                })
            ));
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
        }
    }
}

#[test]
fn atomic_style_root_refuses_actual_composition_without_moving_supplied_doctype() {
    let (mut dom, associated, old_doc) = setup(limits());
    let mut b = DomBatch::new();
    let dt = b.create_document_type(old_doc, "html", "", "");
    b.insert_before(&dt, old_doc, None);
    let c = apply(&mut dom, b);
    let dt = c.created(&dt).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.create_xml_document(associated, Namespace::html(), "style", Some(dt.into()));
    assert!(matches!(
        dom.apply(b, &AtomicBool::new(false)),
        Err(DomError::StyleCompositionRequired(
            surgeist_html::HtmlStyleUpdateAction::CheckInlineCsp
        ))
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.snapshot().node(dt).unwrap().parent(),
        Some(old_doc.node())
    );
    assert_eq!(
        dom.snapshot().node(dt).unwrap().owner_document(),
        Some(old_doc)
    );
}

#[test]
fn later_invalid_command_restores_doctype_after_private_constructor_adoption() {
    let (mut dom, associated, old_doc) = setup(limits());
    let mut b = DomBatch::new();
    let dt = b.create_document_type(old_doc, "root", "", "");
    b.insert_before(&dt, old_doc, None);
    let c = apply(&mut dom, b);
    let dt = c.created(&dt).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    let doc = b.create_xml_document(associated, Namespace::none(), "Root", Some(dt.into()));
    b.create_processing_instruction(&doc, "1invalid", "");
    assert_eq!(
        dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidCharacter
    );
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(after.node(old_doc.node()).unwrap().children(), &[dt]);
    assert_eq!(after.node(dt).unwrap().parent(), Some(old_doc.node()));
    assert_eq!(after.node(dt).unwrap().owner_document(), Some(old_doc));
    assert_eq!(
        after.node(dt).unwrap().node_name(),
        DocumentString::from("root")
    );
}

#[test]
fn associated_explicit_input_policy_is_reestablished_in_the_new_closed_environment() {
    use std::sync::Arc;
    use surgeist_html::{
        HtmlInputEmptySelectionSupport as Empty, HtmlInputNoneDirectionSupport as Direction,
    };
    use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};
    use surgeist_html_semantics::{
        HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
        HtmlBaseUrlScheme,
    };
    let policy = DomNativeInputPlatform::new(Empty::Unsupported, Direction::Unsupported);
    let (mut dom, origin_source, _) = setup(limits());
    let snapshot = dom.snapshot();
    let origin = snapshot
        .document(origin_source)
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let parse_limits = HtmlParseLimits::builder()
        .source_units(100)
        .buffered_utf16_units(100)
        .token_attributes(100)
        .auxiliary_entries(100_000)
        .managed_nodes(100)
        .managed_edges(100)
        .managed_attributes(100)
        .managed_utf16_units(1000)
        .source_map_entries(1000)
        .source_map_payload_units(1000)
        .diagnostics(100)
        .diagnostic_payload_units(1000)
        .context_effects(100)
        .open_elements(100)
        .tree_depth(100)
        .managed_capacity_bytes(1_000_000)
        .peak_transient_bytes(1_000_000)
        .atomic_layout_bytes(1_000_000)
        .work_credits(10_000_000)
        .finish()
        .unwrap();
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>source</p>"), parse_limits)
            .parse()
            .unwrap(),
    );
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1);
    let url = HtmlBaseUrlRecord::observed(
        revision,
        "https://example.test/source/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    // Actual existing installation supplies a per-Document policy distinct
    // from this DOM's native headless default; no unchecked metadata setter.
    let installed = dom
        .install_parsed_html_document(
            snapshot.stamp(),
            report,
            DomParsedDocumentEnvironment::headless(
                origin,
                url,
                HtmlBaseEncoding::fresh(),
                "windows-1252".into(),
                policy,
            ),
            DomParsedHtmlInstallLimits::new(128, 10_000_000, 10_000_000, 10_000_000),
            &AtomicBool::new(false),
        )
        .unwrap();
    let associated = installed.document();
    let mut b = DomBatch::new();
    let doc = b.create_xml_document(associated, Namespace::html(), "", None);
    let sibling = b.create_document();
    let c = apply(&mut dom, b);
    let doc = c.document(&doc).unwrap();
    let sibling = c.document(&sibling).unwrap();
    let snapshot = dom.snapshot();
    let source = snapshot.document(associated).unwrap().metadata();
    assert_eq!(source.class(), DomDocumentClass::Html);
    assert_eq!(source.encoding(), &DocumentString::from("windows-1252"));
    assert_eq!(
        source.url(),
        &DocumentString::from("https://example.test/source/")
    );
    assert_eq!(source.mode(), HtmlDocumentMode::Quirks);
    xml_defaults(&snapshot, doc, associated, "application/xhtml+xml");
    assert_eq!(
        snapshot
            .document(doc)
            .unwrap()
            .metadata()
            .document_environment()
            .input_platform(),
        policy
    );
    assert_eq!(
        snapshot
            .document(sibling)
            .unwrap()
            .metadata()
            .document_environment()
            .input_platform(),
        DomNativeInputPlatform::headless()
    );
}

#[test]
fn actual_style_publication_parks_original_constructor_then_finishes_origin_mime_and_tickets_once()
{
    use std::sync::{Arc, atomic::Ordering};
    use surgeist_css::{CssParserContext, CssUtf16InputLimits};
    use surgeist_cssom::{
        CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs, CssomStore,
    };
    use surgeist_html::HtmlStyleUpdateAction;
    let cancel = AtomicBool::new(false);
    let (mut dom, associated, old_doc) = setup(limits());
    let mut b = DomBatch::new();
    let dt = b.create_document_type(old_doc, "root", "", "");
    b.insert_before(&dt, old_doc, None);
    let c = apply(&mut dom, b);
    let dt = c.created(&dt).unwrap().unwrap();
    let before = dom.snapshot();
    let old_nodes = dom.usage().managed_nodes();
    let associated_origin = before
        .document(associated)
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let edit =
        DomSourceEdit::create_xml_document(associated.node(), Namespace::html(), "style", Some(dt));
    let bounds = DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 64, 4096, 4096, 1);
    let mut lease = dom.begin_source_edit(edit, bounds, &cancel).unwrap();
    let operation = lease.operation();
    let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("real inline policy request");
    };
    assert_eq!(
        policy.action(),
        DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
    );
    let document = policy.source().document();
    let style = policy.source().element();
    // DOM3071 copies origin and chooses MIME AFTER actual root insertion.
    // The original source capture and published unfinished snapshot therefore
    // retain default XML metadata, not a prematurely finalized constructor.
    assert_eq!(
        policy.source().document_metadata().content_type(),
        &DocumentString::from("application/xml")
    );
    assert_ne!(
        policy.source().document_metadata().origin(),
        &associated_origin
    );
    let pending = lease.snapshot();
    let metadata = pending.document(document).unwrap().metadata();
    assert_eq!(metadata.class(), DomDocumentClass::Xml);
    assert_eq!(
        metadata.content_type(),
        &DocumentString::from("application/xml")
    );
    assert_ne!(metadata.origin(), &associated_origin);
    assert_eq!(
        pending.node(document.node()).unwrap().children(),
        &[dt, style]
    );
    assert_eq!(pending.node(dt).unwrap().owner_document(), Some(document));
    assert_eq!(pending.node(old_doc.node()).unwrap().children(), []);
    assert_eq!(pending.nodes().count(), old_nodes + 2);
    assert_eq!(before.node(dt).unwrap().parent(), Some(old_doc.node()));
    lease
        .accept_inline_csp(
            &policy,
            &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("real sheet creation");
    };
    assert_eq!(
        create.action(),
        DomStyleAction::Update(HtmlStyleUpdateAction::CreateSheet)
    );
    assert_eq!(create.source().document(), document);
    assert_eq!(create.source().element(), style);
    let owner = DomStyleCssomRoot::new(
        create.cssom_root().unwrap(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    lease.bind_cssom_root(&create, &owner, &cancel).unwrap();
    let create = lease
        .qualify_cssom(&create, owner.root(), &owner.snapshot().unwrap())
        .unwrap();
    let version = CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: "xml-style-owner".into(),
        revision: 1,
    };
    lease
        .bind_cssom_owner(
            &create,
            DomStyleCssomOwnerBinding::new(&create, version.clone(), "inline-owner".into())
                .unwrap(),
            &cancel,
        )
        .unwrap();
    let parser = CssParserContext::default();
    let body = parser
        .parse_sheet_utf16_with_limits(
            create.source().text().clone(),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let media = parser
        .parse_media_query_list_utf16_with_limits(
            Arc::new(Vec::new()),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let mut inputs = CssomSheetInputs::external(version, true);
    inputs.owner = Some("inline-owner".into());
    let mut edit = owner
        .batch(CssomEditGuard::from_snapshot(&owner.snapshot().unwrap()))
        .unwrap();
    let construction = edit
        .create_parsed_sheet_with_media_reports(body, media, inputs)
        .unwrap();
    let commit = owner.commit(&mut Some(edit)).unwrap();
    let sheet = commit.sheet(construction.ticket()).unwrap().clone();
    let cssom_snapshot = owner.snapshot().unwrap();
    let revision = cssom_snapshot.revision().clone();
    let mut notification = lease
        .accept_cssom_publication(&create, commit, cssom_snapshot, Some(construction.ticket()))
        .unwrap()
        .unwrap();
    cancel.store(true, Ordering::Relaxed);
    assert_eq!(lease.advance(&cancel).unwrap_err(), DomError::Cancelled);
    drop(lease);
    assert!(dom.pending_source_edit().unwrap().has_published());
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .metadata()
            .content_type(),
        &DocumentString::from("application/xml")
    );
    assert_ne!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .metadata()
            .origin(),
        &associated_origin
    );
    drop(owner);
    cancel.store(false, Ordering::Relaxed);
    let mut lease = dom.resume_source_edit(operation, bounds, &cancel).unwrap();
    let owner = lease.cssom_root(&create).unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("acknowledged creation cannot replay");
    };
    assert_eq!(done.operation(), operation);
    assert_eq!(done.published_node(), Some(document.node()));
    assert_eq!(done.output(), Some(DomSourceOutput::Document(document)));
    drop(lease);
    // The next requested operation uses the completed Document identity.
    let mut next = DomBatch::new();
    let following = next.create_comment(document, "once after finalization");
    let following = apply(&mut dom, next).created(&following).unwrap().unwrap();
    assert_ne!(following, style);
    let final_snapshot = dom.snapshot();
    xml_defaults(
        &final_snapshot,
        document,
        associated,
        "application/xhtml+xml",
    );
    assert_eq!(
        final_snapshot.node(document.node()).unwrap().children(),
        &[dt, style]
    );
    assert_eq!(
        final_snapshot.node(following).unwrap().data(),
        Some(&DocumentString::from("once after finalization"))
    );
    assert_eq!(
        final_snapshot.node(following).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(
        final_snapshot
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap()
            .id(),
        &sheet
    );
    assert_eq!(dom.usage().managed_nodes(), old_nodes + 3);
    assert_eq!(owner.snapshot().unwrap().sheets().len(), 1);
    assert_eq!(owner.snapshot().unwrap().revision(), &revision);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(before.node(dt).unwrap().owner_document(), Some(old_doc));
    assert_eq!(
        pending
            .document(document)
            .unwrap()
            .metadata()
            .content_type(),
        &DocumentString::from("application/xml")
    );
    assert_eq!(
        policy.source().document_metadata().content_type(),
        &DocumentString::from("application/xml")
    );
    assert!(matches!(
        dom.finish_style_notification(&mut notification, 1024, &cancel)
            .unwrap(),
        DomStyleNotificationCompletion::Complete(_)
    ));
    assert_eq!(
        dom.finish_style_notification(&mut notification, 1024, &cancel)
            .unwrap_err(),
        DomError::StyleProtocol
    );
}
