//! Literal DOM3071 createHTMLDocument and HTML-dependent API case oracles.
//! Native ownership/admission/rollback follows #1074; HTML877d supplies native
//! family initialization and independent absent-browsing-context semantics.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomCommit, DomDocumentClass, DomError,
    DomHtmlCaptureLimits, DomLimits, DomNodeKind, DomRegistryBinding, DomResource,
    DomShadowRootInit, DomSnapshot, HtmlDocumentMode, HtmlShadowRootMode, Namespace, NodeId,
};
const HTML: &str = "http://www.w3.org/1999/xhtml";
const SVG: &str = "http://www.w3.org/2000/svg";
fn new_dom() -> Dom {
    Dom::new(DomLimits::new(256, 65536, 256, 0, 0, 32, 65536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn documents(dom: &mut Dom) -> (DocumentId, DocumentId) {
    let mut batch = DomBatch::new();
    let xml = batch.create_document();
    let html = batch.create_html_document(&xml, None);
    let commit = apply(dom, batch);
    (
        commit.document(&xml).unwrap(),
        commit.document(&html).unwrap(),
    )
}
fn skeleton(snapshot: &DomSnapshot, document: DocumentId) -> (NodeId, NodeId, NodeId) {
    let children = snapshot.node(document.node()).unwrap().children();
    assert_eq!(children.len(), 2);
    let DomNodeKind::DocumentType(doctype) = snapshot.node(children[0]).unwrap().kind() else {
        panic!("html doctype first")
    };
    assert_eq!(doctype.name(), &"html".into());
    assert_eq!(doctype.public_id(), &"".into());
    assert_eq!(doctype.system_id(), &"".into());
    let html = children[1];
    let children = snapshot.node(html).unwrap().children();
    assert_eq!(children.len(), 2);
    let (head, body) = (children[0], children[1]);
    for (id, local, tag) in [
        (html, "html", "HTML"),
        (head, "head", "HEAD"),
        (body, "body", "BODY"),
    ] {
        let node = snapshot.node(id).unwrap();
        let element = node.as_element().unwrap();
        assert_eq!(element.name().namespace(), &Namespace::from_uri(HTML));
        assert_eq!(element.name().prefix(), None);
        assert_eq!(element.name().local_name(), &DocumentString::from(local));
        assert_eq!(node.tag_name(), Some(tag.into()));
        assert_eq!(node.node_name(), DocumentString::from(tag));
        assert_eq!(node.owner_document(), Some(document));
        assert!(node.is_connected());
        assert!(element.attributes().is_empty());
        assert_eq!(element.nonce(), Some(&"".into()));
    }
    assert!(snapshot.node(body).unwrap().children().is_empty());
    (html, head, body)
}
#[test]
fn complete_skeleton_defaults_and_real_origin_are_published_with_one_document_ticket() {
    let mut dom = new_dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let unrelated = batch.create_document();
    let html = batch.create_html_document(&source, None);
    let next = batch.create_comment(&html, "after private skeleton");
    let chained = batch.create_html_document(&html, None);
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let html = commit.document(&html).unwrap();
    let chained = commit.document(&chained).unwrap();
    let snapshot = dom.snapshot();
    let (_, head, _) = skeleton(&snapshot, html);
    assert!(snapshot.node(head).unwrap().children().is_empty());
    let metadata = snapshot.document(html).unwrap().metadata();
    assert_eq!(metadata.class(), DomDocumentClass::Html);
    assert_eq!(metadata.content_type(), &"text/html".into());
    assert_eq!(metadata.encoding(), &"UTF-8".into());
    assert_eq!(metadata.url(), &"about:blank".into());
    assert_eq!(metadata.mode(), HtmlDocumentMode::NoQuirks);
    assert!(!metadata.allow_declarative_shadow_roots());
    assert_eq!(metadata.custom_element_registry(), DomRegistryBinding::Null);
    assert_eq!(
        metadata.origin(),
        snapshot.document(source).unwrap().metadata().origin()
    );
    assert_eq!(
        metadata.origin(),
        snapshot.document(chained).unwrap().metadata().origin()
    );
    assert_ne!(
        metadata.origin(),
        snapshot
            .document(commit.document(&unrelated).unwrap())
            .unwrap()
            .metadata()
            .origin()
    );
    let source_metadata = snapshot.document(source).unwrap().metadata();
    assert_eq!(source_metadata.class(), DomDocumentClass::Xml);
    assert_eq!(source_metadata.content_type(), &"application/xml".into());
    let environment = metadata.native_environment();
    assert_eq!(
        environment.browsing_context(),
        surgeist_html::HtmlBrowsingContextPresence::Absent
    );
    assert!(!environment.has_ever_had_navigable());
    assert!(!environment.is_fully_active());
    assert_eq!(
        snapshot
            .node(commit.created(&next).unwrap().unwrap())
            .unwrap()
            .data(),
        Some(&"after private skeleton".into())
    );
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::Created(html.node()))
    );
    assert_eq!(dom.usage().managed_nodes(), 13); // two sources + two five-node trees + comment
}
#[test]
fn omitted_empty_and_raw_utf16_titles_have_distinct_required_trees() {
    for title in [
        None,
        Some(DocumentString::from("")),
        Some(DocumentString::from_utf16(vec![
            0x20, 0xd800, 0x41, 0x0a, 0xdc00,
        ])),
    ] {
        let mut dom = new_dom();
        let mut batch = DomBatch::new();
        let source = batch.create_document();
        let document = batch.create_html_document(&source, title.clone());
        let commit = apply(&mut dom, batch);
        let document = commit.document(&document).unwrap();
        let snapshot = dom.snapshot();
        let (html, head, body) = skeleton(&snapshot, document);
        let mut expected = vec![
            document.node(),
            snapshot.node(document.node()).unwrap().children()[0],
            html,
            head,
        ];
        if let Some(title) = &title {
            let children = snapshot.node(head).unwrap().children();
            assert_eq!(children.len(), 1);
            let element = snapshot.node(children[0]).unwrap();
            assert_eq!(element.node_name(), DocumentString::from("TITLE"));
            assert_eq!(element.children().len(), 1);
            let text = element.children()[0];
            assert_eq!(snapshot.node(text).unwrap().data(), Some(title));
            expected.extend([element.id(), text]);
            assert_eq!(dom.usage().managed_nodes(), 8);
            assert_eq!(dom.usage().string_code_units(), 31 + 158 + title.len());
        } else {
            assert!(snapshot.node(head).unwrap().children().is_empty());
            assert_eq!(dom.usage().managed_nodes(), 6);
            assert_eq!(dom.usage().string_code_units(), 31 + 125);
        }
        expected.push(body);
        assert_eq!(
            snapshot
                .node(document.node())
                .unwrap()
                .tree_order()
                .collect::<Vec<_>>(),
            expected
        );
    }
}
#[test]
fn whole_local_name_creation_folds_ascii_without_qname_extraction_or_unicode_repair() {
    let mut dom = new_dom();
    let (xml, html) = documents(&mut dom);
    let mut batch = DomBatch::new();
    let div = batch.create_element(html, "dIV");
    let colon = batch.create_element(html, "a:B");
    let raw = DocumentString::from_utf16(vec![0x41, 0x00c9, 0xd800, 0x5a]);
    let unusual = batch.create_element(html, raw);
    let xml_element = batch.create_element(xml, "dIV");
    let ns = batch.create_element_ns(html, Namespace::from_uri(HTML), "p:MiXeD");
    let null_ns = batch.create_element_ns(html, Namespace::none(), "MiXeD");
    let foreign = batch.create_element_ns(html, Namespace::from_uri(SVG), "p:MiXeD");
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let id = |ticket| commit.created(ticket).unwrap().unwrap();
    let div = snapshot.node(id(&div)).unwrap();
    assert_eq!(div.as_element().unwrap().name().local_name(), &"div".into());
    assert_eq!(
        div.as_element().unwrap().name().namespace(),
        &Namespace::from_uri(HTML)
    );
    assert_eq!(div.node_name(), "DIV".into());
    let colon = snapshot.node(id(&colon)).unwrap();
    let name = colon.as_element().unwrap().name();
    assert_eq!(name.prefix(), None);
    assert_eq!(name.local_name(), &"a:b".into());
    assert_eq!(colon.node_name(), "A:B".into());
    let unusual = snapshot.node(id(&unusual)).unwrap();
    assert_eq!(
        unusual
            .as_element()
            .unwrap()
            .name()
            .local_name()
            .code_units(),
        &[0x61, 0x00c9, 0xd800, 0x7a]
    );
    assert_eq!(
        unusual.node_name().code_units(),
        &[0x41, 0x00c9, 0xd800, 0x5a]
    );
    let xml_element = snapshot.node(id(&xml_element)).unwrap();
    assert_eq!(xml_element.node_name(), "dIV".into());
    assert_eq!(
        xml_element.as_element().unwrap().name().namespace(),
        &Namespace::none()
    );
    let ns = snapshot.node(id(&ns)).unwrap();
    assert_eq!(ns.node_name(), "P:MIXED".into());
    assert_eq!(ns.as_element().unwrap().name().prefix(), Some(&"p".into()));
    assert_eq!(
        ns.as_element().unwrap().name().local_name(),
        &"MiXeD".into()
    );
    assert_eq!(
        snapshot.node(id(&null_ns)).unwrap().node_name(),
        "MiXeD".into()
    );
    assert_eq!(
        snapshot.node(id(&foreign)).unwrap().node_name(),
        "p:MiXeD".into()
    );
}
#[test]
fn folded_builtin_names_require_actual_family_initialization_and_validation_still_precedes_folding()
{
    let mut dom = new_dom();
    let (_, html) = documents(&mut dom);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.create_element(html, "INPUT");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::NativeInitializationRequired(
            surgeist_html::HtmlNativeInitializationRequirement::NativeForm(
                surgeist_html::HtmlNativeFormAssociationKind::Input
            )
        )
    );
    let mut batch = DomBatch::new();
    batch.create_element(html, "A B");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidCharacter
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    // NS preserves INPUT and its actual unknown/common interface selection.
    let mut batch = DomBatch::new();
    let input = batch.create_element_ns(html, Namespace::from_uri(HTML), "INPUT");
    let commit = apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot()
            .node(commit.created(&input).unwrap().unwrap())
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .local_name(),
        &"INPUT".into()
    );
}
#[test]
fn html_non_ns_attributes_fold_only_input_while_ns_attributes_and_intrinsic_reads_stay_exact() {
    let mut dom = new_dom();
    let (_, html) = documents(&mut dom);
    let mut batch = DomBatch::new();
    let element = batch.create_element(html, "div");
    batch.set_attribute_ns(&element, Namespace::none(), "ID", "upper");
    batch.set_attribute(&element, "ÉTAT", "accent");
    let commit = apply(&mut dom, batch);
    let element = commit.created(&element).unwrap().unwrap();
    let before = dom.snapshot();
    let node = before.node(element).unwrap();
    assert_eq!(node.attribute(&"ID".into()), None);
    assert_eq!(
        node.attribute_ns(&Namespace::none(), &"ID".into()),
        Some(&"upper".into())
    );
    assert_eq!(
        node.as_element().unwrap().attribute(&"ID".into()),
        Some(&"upper".into())
    );
    assert_eq!(node.attribute(&"ÉTAT".into()), Some(&"accent".into()));
    assert_eq!(node.attribute(&"état".into()), None);
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "ID", "lower");
    batch.set_attribute(element, "NONCE", "content");
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let node = snapshot.node(element).unwrap();
    assert_eq!(node.attribute(&"ID".into()), Some(&"lower".into()));
    assert_eq!(
        node.attribute_ns(&Namespace::none(), &"ID".into()),
        Some(&"upper".into())
    );
    assert_eq!(
        node.attribute_ns(&Namespace::none(), &"id".into()),
        Some(&"lower".into())
    );
    assert_eq!(node.as_element().unwrap().nonce(), Some(&"content".into()));
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::Attributes(element))
    );
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::ElementState(element))
    );
    let stamp = snapshot.stamp();
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "Id", "lower");
    assert!(apply(&mut dom, batch).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), stamp);
    let mut batch = DomBatch::new();
    batch.remove_attribute(element, "ID");
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let node = snapshot.node(element).unwrap();
    assert_eq!(node.attribute(&"ID".into()), None);
    assert_eq!(
        node.attribute_ns(&Namespace::none(), &"ID".into()),
        Some(&"upper".into())
    );
    assert_eq!(
        before
            .node(element)
            .unwrap()
            .attribute_ns(&Namespace::none(), &"ID".into()),
        Some(&"upper".into())
    );
}
#[test]
fn qualified_non_ns_operations_select_first_matching_attribute_preserving_its_namespace() {
    let mut dom = new_dom();
    let (_, html) = documents(&mut dom);
    let mut batch = DomBatch::new();
    let element = batch.create_element(html, "div");
    batch.set_attribute_ns(&element, Namespace::from_uri("urn:a"), "p:key", "first");
    batch.set_attribute_ns(&element, Namespace::from_uri("urn:b"), "p:key", "second");
    batch.set_attribute_ns(&element, Namespace::from_uri("urn:a"), "p:KEY", "upper");
    batch.set_attribute(&element, "P:KEY", "changed");
    let commit = apply(&mut dom, batch);
    let element = commit.created(&element).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let node = snapshot.node(element).unwrap();
    assert_eq!(node.as_element().unwrap().attributes().len(), 3);
    assert_eq!(node.attribute(&"P:KEY".into()), Some(&"changed".into()));
    assert_eq!(
        node.attribute_ns(&Namespace::from_uri("urn:a"), &"key".into()),
        Some(&"changed".into())
    );
    assert_eq!(
        node.attribute_ns(&Namespace::from_uri("urn:a"), &"KEY".into()),
        Some(&"upper".into())
    );
    let mut batch = DomBatch::new();
    batch.remove_attribute(element, "P:KEY");
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let node = snapshot.node(element).unwrap();
    assert_eq!(node.attribute(&"p:key".into()), Some(&"second".into()));
    assert_eq!(
        node.attribute_ns(&Namespace::from_uri("urn:a"), &"KEY".into()),
        Some(&"upper".into())
    );
}
#[test]
fn html_namespace_and_document_class_are_both_required_for_non_ns_attribute_casing() {
    let mut dom = new_dom();
    let (xml, html) = documents(&mut dom);
    let mut batch = DomBatch::new();
    let html_in_xml = batch.create_element_ns(xml, Namespace::from_uri(HTML), "div");
    let svg_in_html = batch.create_element_ns(html, Namespace::from_uri(SVG), "g");
    let generic_in_html = batch.create_element_ns(html, Namespace::none(), "div");
    for element in [&html_in_xml, &svg_in_html, &generic_in_html] {
        batch.set_attribute(element, "Nonce", "raw");
    }
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    for element in [&html_in_xml, &svg_in_html, &generic_in_html] {
        let node = snapshot
            .node(commit.created(element).unwrap().unwrap())
            .unwrap();
        assert_eq!(node.attribute(&"Nonce".into()), Some(&"raw".into()));
        assert_eq!(node.attribute(&"nonce".into()), None);
        if let Some(nonce) = node.as_element().unwrap().nonce() {
            assert!(nonce.is_empty());
        }
    }
}
#[test]
fn adoption_changes_contextual_getters_and_next_write_without_rewriting_raw_names_or_old_snapshots()
{
    let mut dom = new_dom();
    let (xml, html) = documents(&mut dom);
    let mut batch = DomBatch::new();
    let element = batch.create_element_ns(xml, Namespace::from_uri(HTML), "p:MiXeD");
    batch.set_attribute(&element, "ID", "upper");
    batch.set_attribute(&element, "id", "lower");
    let commit = apply(&mut dom, batch);
    let element = commit.created(&element).unwrap().unwrap();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.adopt(element, html);
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let node = snapshot.node(element).unwrap();
    assert_eq!(node.node_name(), "P:MIXED".into());
    assert_eq!(node.attribute(&"ID".into()), Some(&"lower".into()));
    assert_eq!(
        node.as_element().unwrap().name(),
        old.node(element).unwrap().as_element().unwrap().name()
    );
    assert_eq!(
        node.as_element().unwrap().attributes(),
        old.node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
    );
    assert_eq!(
        commit.changes().changes(),
        &[DomChange::NodeDocument(element)]
    );
    assert_eq!(old.node(element).unwrap().node_name(), "p:MiXeD".into());
    assert_eq!(
        old.node(element).unwrap().attribute(&"ID".into()),
        Some(&"upper".into())
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "ID", "new lower");
    batch.adopt(element, xml);
    batch.set_attribute(element, "ID", "new upper");
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let node = snapshot.node(element).unwrap();
    assert_eq!(node.node_name(), "p:MiXeD".into());
    assert_eq!(node.attribute(&"ID".into()), Some(&"new upper".into()));
    assert_eq!(node.attribute(&"id".into()), Some(&"new lower".into()));
}
#[test]
fn import_preserves_actual_qnames_and_attribute_case_while_target_context_changes_presentation() {
    let mut source = new_dom();
    let (xml, html) = documents(&mut source);
    let mut batch = DomBatch::new();
    let a = batch.create_element_ns(xml, Namespace::from_uri(HTML), "p:MiXeD");
    let b = batch.create_element_ns(html, Namespace::from_uri(HTML), "p:MiXeD");
    for element in [&a, &b] {
        batch.set_attribute_ns(element, Namespace::none(), "ID", "raw upper");
        batch.set_attribute_ns(element, Namespace::none(), "id", "raw lower");
    }
    let commit = apply(&mut source, batch);
    let a = commit.created(&a).unwrap().unwrap();
    let b = commit.created(&b).unwrap().unwrap();
    let captured = source.snapshot();
    let source_document = xml;
    let mut target = new_dom();
    let (xml, html) = documents(&mut target);
    let mut batch = DomBatch::new();
    let to_html = batch.import_node(&captured, a, html, true);
    let to_xml = batch.import_node(&captured, b, xml, true);
    let after = batch.create_element(xml, "after");
    let commit = apply(&mut target, batch);
    let snapshot = target.snapshot();
    for (ticket, source_id, display, lookup) in [
        (&to_html, a, "P:MIXED", "raw lower"),
        (&to_xml, b, "p:MiXeD", "raw upper"),
    ] {
        let node = snapshot
            .node(commit.created(ticket).unwrap().unwrap())
            .unwrap();
        assert_ne!(node.id(), source_id);
        assert_eq!(node.node_name(), display.into());
        assert_eq!(node.attribute(&"ID".into()), Some(&lookup.into()));
        assert_eq!(
            node.as_element().unwrap().name(),
            captured
                .node(source_id)
                .unwrap()
                .as_element()
                .unwrap()
                .name()
        );
        assert_eq!(
            node.as_element().unwrap().attributes(),
            captured
                .node(source_id)
                .unwrap()
                .as_element()
                .unwrap()
                .attributes()
        );
    }
    assert_eq!(
        snapshot
            .node(commit.created(&after).unwrap().unwrap())
            .unwrap()
            .node_name(),
        "after".into()
    );
    let mut batch = DomBatch::new();
    batch.import_node(&captured, source_document.node(), html, true);
    assert_eq!(
        target.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::NotSupported
    );
}
#[test]
fn shadow_host_class_is_not_used_for_descendants_with_a_different_actual_document() {
    let mut dom = new_dom();
    let (xml, html) = documents(&mut dom);
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(xml, Namespace::from_uri(HTML), "div");
    let root = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let target = batch.create_element(html, "div");
    let old = batch.create_comment(html, "old");
    batch.insert_before(&old, &target, None);
    let commit = apply(&mut dom, batch);
    let host = commit.created(&host).unwrap().unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let old = commit.created(&old).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.replace_child(root, target, old);
    let child = batch.create_element_ns(html, Namespace::from_uri(HTML), "MiXeD");
    batch.insert_before(&child, root, None);
    batch.set_attribute(&child, "ID", "value");
    let commit = apply(&mut dom, batch);
    let child = commit.created(&child).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(root).unwrap().owner_document(), Some(html));
    assert_eq!(snapshot.node(host).unwrap().owner_document(), Some(xml));
    assert_eq!(snapshot.node(host).unwrap().node_name(), "div".into());
    assert_eq!(snapshot.node(child).unwrap().node_name(), "MIXED".into());
    assert_eq!(
        snapshot
            .node(child)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"id".into()),
        Some(&"value".into())
    );
}
#[test]
fn exact_skeleton_capacity_and_one_short_limits_apply_to_all_private_allocations() {
    for (title, total_nodes, total_units) in [
        (None, 6, 156),
        (Some(DocumentString::from("")), 8, 189),
        (Some(DocumentString::from_utf16(vec![0xd800, 0x41])), 8, 191),
    ] {
        for (nodes, units, error) in [
            (
                total_nodes - 1,
                total_units,
                Some(DomError::ResourceLimit {
                    resource: DomResource::ManagedNodes,
                    limit: total_nodes - 1,
                }),
            ),
            (
                total_nodes,
                total_units - 1,
                Some(DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: total_units - 1,
                }),
            ),
            (total_nodes, total_units, None),
        ] {
            let mut dom =
                Dom::new(DomLimits::new(nodes, units, 1, 0, 0, 8, 8192).unwrap()).unwrap();
            let mut batch = DomBatch::new();
            let source = batch.create_document();
            let source = apply(&mut dom, batch).document(&source).unwrap();
            let before = dom.snapshot();
            let usage = dom.usage();
            let mut batch = DomBatch::new();
            let document = batch.create_html_document(source, title.clone());
            match error {
                Some(error) => {
                    assert_eq!(
                        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
                        error
                    );
                    assert_eq!(dom.snapshot().stamp(), before.stamp());
                    assert_eq!(dom.usage(), usage);
                    assert!(
                        dom.changes_since(before.stamp())
                            .unwrap()
                            .changes()
                            .is_empty()
                    );
                }
                None => {
                    let commit = apply(&mut dom, batch);
                    skeleton(&dom.snapshot(), commit.document(&document).unwrap());
                    assert_eq!(dom.usage().managed_nodes(), total_nodes);
                    assert_eq!(dom.usage().string_code_units(), total_units);
                }
            }
        }
    }
}
#[test]
fn invalid_origin_source_late_failure_and_cancelled_creation_publish_no_document_or_history() {
    let mut dom = new_dom();
    let (source, _) = documents(&mut dom);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    let rejected = batch.create_html_document(source, Some("title".into()));
    batch.insert_before(&rejected, source, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let mut batch = DomBatch::new();
    batch.create_html_document(source, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let mut batch = DomBatch::new();
    let element = batch.create_element(source, "wrong");
    batch.create_html_document(&element, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::WrongNodeKind
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let mut foreign = new_dom();
    let (other, _) = documents(&mut foreign);
    let mut batch = DomBatch::new();
    batch.create_html_document(other, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ForeignOwner
    );
    let mut first = DomBatch::new();
    let ticket = first.create_document();
    let mut second = DomBatch::new();
    second.create_html_document(&ticket, None);
    assert_eq!(
        dom.apply(second, &AtomicBool::new(false)).unwrap_err(),
        DomError::ForeignBatch
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    let mut batch = DomBatch::new();
    let accepted = batch.create_html_document(source, None);
    let commit = apply(&mut dom, batch);
    assert_eq!(
        commit.created(&rejected).unwrap_err(),
        DomError::ForeignBatch
    );
    assert!(commit.document(&accepted).is_ok());
}
#[test]
fn semantic_capture_preserves_html_class_and_independent_native_environment_proof() {
    use surgeist_html_semantics::{
        HtmlCaptureLimits, HtmlCapturedNodeKind, HtmlDocumentClass, HtmlScriptingStatus,
    };
    let mut dom = new_dom();
    let (_, html) = documents(&mut dom);
    let snapshot = dom.snapshot();
    let capture = snapshot
        .html_capture(
            html,
            None,
            HtmlCaptureLimits::new(32, 256, 64, 8192, 32, 0, 0),
            DomHtmlCaptureLimits::new(32, 100000),
            &AtomicBool::new(false),
        )
        .unwrap();
    let subject = capture.subject(html.node()).unwrap();
    let HtmlCapturedNodeKind::Document(facts) = capture.capture().node(&subject).unwrap().kind()
    else {
        panic!("Document facts")
    };
    assert_eq!(facts.document_class(), HtmlDocumentClass::Html);
    assert_eq!(facts.scripting(), HtmlScriptingStatus::Disabled);
    assert_eq!(facts.mode(), HtmlDocumentMode::NoQuirks);
    assert_eq!(
        facts.nonce_environment().browsing_context(),
        surgeist_html::HtmlBrowsingContextPresence::Absent
    );
    assert_eq!(capture.basis(), snapshot.document(html).unwrap().stamp());
}
