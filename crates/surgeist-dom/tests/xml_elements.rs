use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomElementDefinition, DomError,
    DomLimits, DomRegistryBinding, DomResource, HtmlCustomElementState, Namespace, NodeId,
};

fn dom() -> Dom {
    Dom::new(DomLimits::new(64, 4096, 128, 16, 65536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn setup(dom: &mut Dom) -> (DocumentId, NodeId) {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "Root");
    let committed = apply(dom, batch);
    (
        committed.document(&document).unwrap(),
        committed.created(&root).unwrap().unwrap(),
    )
}

#[test]
fn native_generic_elements_keep_exact_names_and_known_null_creation_state() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "MiXeD:input");
    batch.set_attribute(&root, "is", "x-control");
    let committed = apply(&mut dom, batch);
    let root = committed.created(&root).unwrap().unwrap();
    let capture = dom.snapshot();
    let node = capture.node(root).unwrap();
    let element = node.as_element().unwrap();
    assert_eq!(node.node_name(), DocumentString::from("MiXeD:input"));
    assert_eq!(node.tag_name(), Some(DocumentString::from("MiXeD:input")));
    assert_eq!(element.name().namespace(), &Namespace::none());
    assert_eq!(element.name().prefix(), None);
    assert_eq!(
        element.name().local_name(),
        &DocumentString::from("MiXeD:input")
    );
    assert_eq!(
        element.custom_element_state(),
        HtmlCustomElementState::Uncustomized
    );
    assert_eq!(element.custom_element_registry(), DomRegistryBinding::Null);
    assert_eq!(
        element.custom_element_definition(),
        DomElementDefinition::Null
    );
    assert_eq!(element.saved_is(), None);
    assert_eq!(
        element.attribute(&"is".into()),
        Some(&DocumentString::from("x-control"))
    );
    assert_eq!(node.element_index(), Some(0));
    assert_eq!(node.parent_element(), None);
    assert_eq!(
        capture
            .node(committed.document(&document).unwrap().node())
            .unwrap()
            .as_element(),
        None
    );
}

#[test]
fn element_names_use_the_relaxed_dom_branches_and_preserve_isolated_surrogates() {
    let mut dom = dom();
    let (document, _) = setup(&mut dom);
    for name in [
        "a=",
        "a<",
        "a\"",
        "a?",
        "a\u{b}",
        "_",
        ":a",
        "é",
        "\u{80}",
        "\u{ffff}",
        "\u{10ffff}",
    ] {
        let mut batch = DomBatch::new();
        let element = batch.create_element(document, name);
        let element = apply(&mut dom, batch).created(&element).unwrap().unwrap();
        assert_eq!(
            dom.snapshot().node(element).unwrap().node_name(),
            DocumentString::from(name)
        );
    }
    for units in [
        vec![0xd800],
        vec![0xdc00],
        vec![0xdbff, 0xdfff],
        vec![0x61, 0xd800, 0x3d],
    ] {
        let name = DocumentString::from_utf16(units);
        let mut batch = DomBatch::new();
        let element = batch.create_element(document, name.clone());
        let element = apply(&mut dom, batch).created(&element).unwrap().unwrap();
        assert_eq!(dom.snapshot().node(element).unwrap().node_name(), name);
    }
    for name in [
        "", "_=", "é=", "1", "-a", ".a", "\u{7f}", "a b", "a\0", "a/", "a>", "_\u{b}",
    ] {
        let before = dom.snapshot().stamp();
        let mut batch = DomBatch::new();
        batch.create_element(document, name);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::InvalidCharacter,
            "{name:?}"
        );
        assert_eq!(dom.snapshot().stamp(), before);
    }
    let mut batch = DomBatch::new();
    batch.create_element(document, DocumentString::from_utf16(vec![0xd800, 0x3d]));
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidCharacter
    );
}

#[test]
fn attributes_preserve_expanded_identity_prefix_order_and_first_qualified_match() {
    let mut dom = dom();
    let (_, root) = setup(&mut dom);
    let a = Namespace::from_uri("urn:a");
    let b = Namespace::from_uri("urn:b");
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(root, a.clone(), "p:x", "first");
    batch.set_attribute_ns(root, b.clone(), "p:x", "second");
    batch.set_attribute(root, "xml:lang", "literal");
    apply(&mut dom, batch);
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(root, a.clone(), "other:x", "updated");
    batch.set_attribute(root, "p:x", "first qualified match");
    let report = apply(&mut dom, batch);
    assert_eq!(report.changes().changes(), [DomChange::Attributes(root)]);
    let snapshot = dom.snapshot();
    let element = snapshot.node(root).unwrap().as_element().unwrap();
    assert_eq!(
        element.attribute_ns(&a, &"x".into()),
        Some(&DocumentString::from("first qualified match"))
    );
    assert_eq!(
        element.attribute_ns(&b, &"x".into()),
        Some(&DocumentString::from("second"))
    );
    assert_eq!(
        element.attribute_ns(&Namespace::none(), &"xml:lang".into()),
        Some(&DocumentString::from("literal"))
    );
    assert_eq!(
        element.attribute_ns(
            &Namespace::from_uri("http://www.w3.org/XML/1998/namespace"),
            &"lang".into()
        ),
        None
    );
    let attrs = element.attributes().as_slice();
    assert_eq!(attrs.len(), 3);
    assert_eq!(attrs[0].name().prefix(), Some(&DocumentString::from("p")));
    assert_eq!(attrs[1].name().namespace(), &b);
    assert_eq!(
        old.node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute_ns(&a, &"x".into()),
        Some(&DocumentString::from("first"))
    );
    let mut batch = DomBatch::new();
    batch.remove_attribute(root, "p:x");
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let element = snapshot.node(root).unwrap().as_element().unwrap();
    assert_eq!(element.attributes().len(), 2);
    assert_eq!(
        element.attribute(&"p:x".into()),
        Some(&DocumentString::from("second"))
    );
    let mut batch = DomBatch::new();
    batch.remove_attribute_ns(root, b, "x");
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot()
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .len(),
        1
    );
}

#[test]
fn namespace_validation_uses_first_colon_and_source_error_precedence() {
    let mut dom = dom();
    let (_, root) = setup(&mut dom);
    let xml = "http://www.w3.org/XML/1998/namespace";
    let xmlns = "http://www.w3.org/2000/xmlns/";
    for (namespace, name, error) in [
        ("", "p:x", DomError::Namespace),
        ("", "p:=", DomError::InvalidCharacter),
        ("urn:x", ":a", DomError::InvalidCharacter),
        ("urn:x", "xml:", DomError::InvalidCharacter),
        ("urn:x", "xml:a", DomError::Namespace),
        ("urn:x", "xmlns", DomError::Namespace),
        ("urn:x", "xmlns:a", DomError::Namespace),
        (xmlns, "p:a", DomError::Namespace),
        ("urn:x", "p :a", DomError::InvalidCharacter),
    ] {
        let before = dom.snapshot().stamp();
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(root, Namespace::from_uri(namespace), name, "v");
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            error,
            "{namespace:?} {name:?}"
        );
        assert_eq!(dom.snapshot().stamp(), before);
    }
    for (namespace, qualified, prefix, local) in [
        ("urn:x", "p:a:b", Some("p"), "a:b"),
        ("urn:x", "p::a", Some("p"), ":a"),
        ("urn:x", "1:a", Some("1"), "a"),
        ("urn:x", "p=:1", Some("p="), "1"),
        ("urn:x", "XML:a", Some("1"), "a"),
        (xml, "xml:lang", Some("xml"), "lang"),
        (xmlns, "xmlns", None, "xmlns"),
        (xmlns, "xmlns:a", Some("xmlns"), "a"),
        ("", "?", None, "?"),
    ] {
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(root, Namespace::from_uri(namespace), qualified, "v");
        apply(&mut dom, batch);
        let snapshot = dom.snapshot();
        let element = snapshot.node(root).unwrap().as_element().unwrap();
        let attr = element
            .attributes()
            .get(&Namespace::from_uri(namespace), &local.into())
            .unwrap();
        // Repeated expanded identity preserves the first installed prefix.
        assert_eq!(
            attr.name().prefix(),
            prefix.map(DocumentString::from).as_ref()
        );
        assert_eq!(attr.name().local_name(), &DocumentString::from(local));
    }
}

#[test]
fn ordinary_attribute_validation_and_missing_removals_do_not_become_qname_checks() {
    let mut dom = dom();
    let (_, root) = setup(&mut dom);
    for name in ["1", "?", "<", "xml:lang", ":", "a\u{b}"] {
        let mut batch = DomBatch::new();
        batch.set_attribute(root, name, "v");
        apply(&mut dom, batch);
        assert_eq!(
            dom.snapshot()
                .node(root)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&name.into()),
            Some(&DocumentString::from("v"))
        );
    }
    for name in ["", "=", "a b", "a/", "a>", "a\0"] {
        let mut batch = DomBatch::new();
        batch.set_attribute(root, name, "v");
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::InvalidCharacter
        );
    }
    let before = dom.snapshot().stamp();
    let mut batch = DomBatch::new();
    batch.remove_attribute(root, "");
    batch.remove_attribute_ns(root, Namespace::none(), "=");
    assert!(apply(&mut dom, batch).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before);
}

#[test]
fn element_traversal_filters_character_data_and_preserves_captured_indices() {
    let mut dom = dom();
    let (document, root) = setup(&mut dom);
    let mut batch = DomBatch::new();
    let a = batch.create_element(document, "a");
    let b = batch.create_element(document, "b");
    let text = batch.create_text(document, "text");
    let comment = batch.create_comment(document, "comment");
    batch.insert_before(&a, root, None);
    batch.insert_before(&text, root, None);
    batch.insert_before(&comment, root, None);
    batch.insert_before(&b, root, None);
    let commit = apply(&mut dom, batch);
    let a = commit.created(&a).unwrap().unwrap();
    let b = commit.created(&b).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    let before = dom.snapshot();
    assert_eq!(
        before
            .node(root)
            .unwrap()
            .element_children()
            .collect::<Vec<_>>(),
        [a, b]
    );
    assert_eq!(
        before
            .node(root)
            .unwrap()
            .element_children()
            .rev()
            .collect::<Vec<_>>(),
        [b, a]
    );
    assert_eq!(before.node(a).unwrap().next_sibling(), Some(text));
    assert_eq!(before.node(b).unwrap().previous_sibling(), Some(comment));
    assert_eq!(before.node(a).unwrap().next_element_sibling(), Some(b));
    assert_eq!(before.node(b).unwrap().previous_element_sibling(), Some(a));
    assert_eq!(before.node(text).unwrap().element_index(), None);
    assert_eq!(before.node(b).unwrap().element_index(), Some(1));
    assert_eq!(before.node(a).unwrap().parent_element(), Some(root));
    let mut batch = DomBatch::new();
    batch.insert_before(b, root, Some(a.into()));
    apply(&mut dom, batch);
    assert_eq!(dom.snapshot().node(b).unwrap().element_index(), Some(0));
    assert_eq!(before.node(b).unwrap().element_index(), Some(1));
    let mut batch = DomBatch::new();
    batch.detach(b);
    batch.destroy(b);
    apply(&mut dom, batch);
    assert_eq!(dom.snapshot().node(b).unwrap_err(), DomError::StaleNode);
    assert_eq!(before.node(a).unwrap().next_element_sibling(), Some(b));
}

#[test]
fn exact_attribute_budget_counts_all_stored_name_parts_and_rolls_back_overflow() {
    // 31 Document + 1 Element + 5 URI + 1 prefix + 1 local + 2 value = 41.
    let mut dom = Dom::new(DomLimits::new(2, 41, 8, 8, 8192).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "R");
    batch.set_attribute_ns(&root, Namespace::from_uri("urn:a"), "p:x", "😀");
    let commit = apply(&mut dom, batch);
    let root = commit.created(&root).unwrap().unwrap();
    let before = dom.snapshot();
    assert_eq!(dom.usage().string_code_units(), 41);
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(root, Namespace::from_uri("urn:a"), "longer:x", "😀");
    assert!(apply(&mut dom, batch).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let mut batch = DomBatch::new();
    batch.set_attribute(root, "a", "");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: 41
        }
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage().string_code_units(), 41);
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(root, Namespace::from_uri("urn:a"), "ignored:x", "v");
    batch.set_attribute(root, "a", "");
    apply(&mut dom, batch);
    assert_eq!(dom.usage().string_code_units(), 41);
    let mut batch = DomBatch::new();
    batch.destroy(root);
    apply(&mut dom, batch);
    assert_eq!(dom.usage().string_code_units(), 31);
    assert_eq!(
        before
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"p:x".into()),
        Some(&DocumentString::from("😀"))
    );
}

#[test]
fn attribute_names_namespaces_and_values_preserve_exact_utf16_units() {
    let mut dom = dom();
    let (_, root) = setup(&mut dom);
    let literal = DocumentString::from_utf16(vec![0xd800, 0x3a, 0xdfff]);
    let namespace = Namespace::from_uri(DocumentString::from_utf16(vec![0xdc00]));
    let qualified = DocumentString::from_utf16(vec![0x70, 0x3a, 0xd800]);
    let local = DocumentString::from_utf16(vec![0xd800]);
    let value = DocumentString::from_utf16(vec![0xdfff, 0xd800, 0x0d, 0x0a]);
    let mut batch = DomBatch::new();
    batch.set_attribute(root, literal.clone(), value.clone());
    batch.set_attribute_ns(root, namespace.clone(), qualified, value.clone());
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let element = snapshot.node(root).unwrap().as_element().unwrap();
    assert_eq!(
        element.attribute_ns(&Namespace::none(), &literal),
        Some(&value)
    );
    assert_eq!(element.attribute_ns(&namespace, &local), Some(&value));
    assert_eq!(
        element.attributes().as_slice()[1].name().prefix(),
        Some(&DocumentString::from("p"))
    );
}

#[test]
fn final_attribute_and_unpublished_element_noops_do_not_advance_revision() {
    let mut dom = dom();
    let (document, root) = setup(&mut dom);
    let mut batch = DomBatch::new();
    batch.set_attribute(root, "kept", "old");
    apply(&mut dom, batch);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(root, "kept", "changed");
    batch.set_attribute(root, "kept", "old");
    batch.set_attribute(root, "new", "new");
    batch.remove_attribute(root, "new");
    let transient = batch.create_element(document, "transient");
    batch.set_attribute(&transient, "a", "value");
    batch.destroy(&transient);
    let report = apply(&mut dom, batch);
    assert_eq!(report.created(&transient).unwrap(), None);
    assert!(report.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
}
