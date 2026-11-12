//! Detached Attr creation and lifecycle through the canonical arena.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentString, Dom, DomAttributeAttachment, DomBatch, DomError, DomLimits, DomNameLimits,
    DomNodeKind, DomSourceEdit, DomSourceLimits, DomSourceOutput, DomSourceProgress, Namespace,
    NodeId,
};

fn owner() -> Dom {
    let names = DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap();
    Dom::new(
        DomLimits::new(64, 8192, 128, 0, 0, 64, 1_000_000, names)
            .unwrap()
            .with_ranges(8)
            .unwrap(),
    )
    .unwrap()
}

fn policy() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 4096, 4096, 8)
}

#[test]
fn standard_attribute_sink_requires_the_node_documents_global() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let element = setup.create_element_ns(&document, Namespace::html(), "div");
    let commit = dom.apply(setup, &cancel).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let before = dom.snapshot();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_attribute(element, "onclick", ""),
            policy(),
            &cancel,
        )
        .unwrap();
    let result = lease.advance(&cancel);
    assert!(
        matches!(result, Err(DomError::SourceAttributeGlobalUnavailable)),
        "a recognized handler sink needs the node document's actual Global: {result:?}"
    );
    let after = lease.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(
        after.node(element).unwrap().attribute(&"onclick".into()),
        None
    );
}

#[test]
fn attached_attr_text_content_preserves_the_attribute_identity() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let element = setup.create_element(&document, "div");
    let namespace = Namespace::from_uri("urn:attr-test");
    setup.set_attribute_ns(&element, namespace, "original:item", "old");
    let commit = dom.apply(setup, &cancel).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let name = "original:item".into();
    let before = dom.snapshot();
    let attr = before
        .node(element)
        .unwrap()
        .get_attribute_node(&name)
        .unwrap()
        .id();
    let value = DocumentString::from_utf16(vec![0xd800, 0x0041, 0]);

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_text_content(attr.node(), value.clone()),
            policy(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    let after = lease.snapshot();
    let current = after
        .node(element)
        .unwrap()
        .get_attribute_node(&name)
        .unwrap();
    assert_eq!(current.id(), attr);
    assert_eq!(current.value(), &value);
    assert_eq!(before.attr(attr.node()).unwrap().value(), &"old".into());
}

#[test]
fn attached_attr_text_content_verifies_against_its_owner_element() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let element = setup.create_element_ns(&document, Namespace::html(), "div");
    setup.set_attribute(&element, "onclick", "old");
    let detached = setup.create_attribute(&document, "onclick");
    let commit = dom.apply(setup, &cancel).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let detached = commit.created(&detached).unwrap().unwrap();
    let before = dom.snapshot();
    let attr = before
        .node(element)
        .unwrap()
        .get_attribute_node(&"onclick".into())
        .unwrap()
        .id();

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_text_content(attr.node(), "new"),
            policy(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel),
        Err(DomError::SourceAttributeGlobalUnavailable)
    ));
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert_eq!(
        lease.snapshot().attr(attr.node()).unwrap().value(),
        &"old".into()
    );
    drop(lease);

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_text_content(detached, "new"),
            policy(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    assert_eq!(
        lease.snapshot().attr(detached).unwrap().value(),
        &"new".into()
    );
}

#[test]
fn standard_attr_attachment_verifies_before_owner_and_identity_checks() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let first = setup.create_element_ns(&document, Namespace::html(), "div");
    let second = setup.create_element_ns(&document, Namespace::html(), "div");
    setup.set_attribute(&first, "onclick", "old");
    let ordinary = setup.create_attribute(&document, "data-x");
    let commit = dom.apply(setup, &cancel).unwrap();
    let first = commit.created(&first).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let ordinary = commit.created(&ordinary).unwrap().unwrap();
    let before = dom.snapshot();
    let ordinary = before.attr(ordinary).unwrap().id();
    let handler = before
        .node(first)
        .unwrap()
        .get_attribute_node(&"onclick".into())
        .unwrap()
        .id();

    for edit in [
        DomSourceEdit::attach_attribute(first, handler),
        DomSourceEdit::attach_attribute(second, handler),
        DomSourceEdit::set_attribute_value(handler, "replacement"),
    ] {
        let mut lease = dom.begin_source_edit(edit, policy(), &cancel).unwrap();
        assert!(matches!(
            lease.advance(&cancel),
            Err(DomError::SourceAttributeGlobalUnavailable)
        ));
        assert_eq!(lease.snapshot().stamp(), before.stamp());
    }

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::attach_attribute(second, ordinary),
            policy(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Complete(completion) = lease.advance(&cancel).unwrap() else {
        panic!("an ordinary Attr attaches synchronously");
    };
    assert_eq!(
        completion.output(),
        Some(DomSourceOutput::AttributeAttachment(
            DomAttributeAttachment::Appended(ordinary)
        ))
    );
    drop(lease);

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_attribute_value(ordinary, "new"),
            policy(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    assert_eq!(
        lease.snapshot().attr(ordinary.node()).unwrap().value(),
        &"new".into()
    );
}

#[test]
fn standard_source_setters_select_exact_attribute_sinks() {
    let cancel = AtomicBool::new(false);
    let xlink = "http://www.w3.org/1999/xlink";
    let cases = [
        (Namespace::html(), "div", None, "ONCLICK", true),
        (Namespace::html(), "div", Some(""), "ONCLICK", false),
        (Namespace::html(), "div", None, "onafterprint", true),
        (Namespace::html(), "div", None, "onopen", false),
        (Namespace::mathml(), "mrow", None, "onbegin", true),
        (
            Namespace::from_uri("urn:foreign"),
            "div",
            None,
            "onclick",
            false,
        ),
        (Namespace::svg(), "script", None, "href", true),
        (Namespace::svg(), "script", Some(xlink), "p:href", true),
        (Namespace::svg(), "g", Some(xlink), "p:href", false),
        (Namespace::html(), "script", None, "src", true),
        (Namespace::html(), "iframe", None, "srcdoc", true),
        (
            Namespace::html(),
            "div",
            Some("urn:foreign"),
            "p:onclick",
            false,
        ),
    ];
    for (namespace, tag, attr_namespace, name, sink) in cases {
        let mut dom = owner();
        let mut setup = DomBatch::new();
        let origin = setup.create_document();
        let document = setup.create_html_document(&origin, None);
        let element = setup.create_element_ns(&document, namespace, tag);
        let commit = dom.apply(setup, &cancel).unwrap();
        let element = commit.created(&element).unwrap().unwrap();
        let before = dom.snapshot().stamp();
        let edit = match attr_namespace {
            Some(namespace) => {
                DomSourceEdit::set_attribute_ns(element, Namespace::from_uri(namespace), name, "")
            }
            None => DomSourceEdit::set_attribute(element, name, ""),
        };
        let mut lease = dom.begin_source_edit(edit, policy(), &cancel).unwrap();
        let result = lease.advance(&cancel);
        if sink {
            assert!(
                matches!(result, Err(DomError::SourceAttributeGlobalUnavailable)),
                "{tag} {attr_namespace:?} {name}: {result:?}"
            );
            assert_eq!(lease.snapshot().stamp(), before);
        } else {
            assert!(
                matches!(result, Ok(DomSourceProgress::Complete(_))),
                "{tag} {attr_namespace:?} {name}: {result:?}"
            );
            assert_ne!(lease.snapshot().stamp(), before);
        }
    }
}

#[test]
fn attr_value_comparison_uses_the_operation_budget_without_spending_name_work() {
    let names = DomNameLimits::new(16_777_216, 16_384, 512, 1_048_576).unwrap();
    let mut dom =
        Dom::new(DomLimits::new(64, 8192, 128, 0, 0, 64, 1_000_000, names).unwrap()).unwrap();
    let cancel = AtomicBool::new(false);
    let mut units = vec![0xd800; 2048];
    let original = DocumentString::from_utf16(units.clone());
    units[2047] = 0xdc00;
    let replacement = DocumentString::from_utf16(units);
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let element = setup.create_element(&document, "div");
    setup.set_attribute(&element, "data-x", original.clone());
    let detached = setup.create_attribute(&document, "data-y");
    let commit = dom.apply(setup, &cancel).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let detached = commit.created(&detached).unwrap().unwrap();

    let mut edit = DomBatch::new();
    edit.set_attribute(element, "data-x", replacement.clone());
    dom.apply(edit, &cancel).unwrap();
    let after = dom.snapshot();
    let value = after.node(element).unwrap().attribute(&"data-x".into());
    assert_eq!(value, Some(&replacement));

    for value in [original, replacement.clone()] {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_text_content(detached, value),
                policy(),
                &cancel,
            )
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    assert_eq!(dom.snapshot().attr(detached).unwrap().value(), &replacement);
}

#[test]
fn attr_creation_uses_the_actual_document_and_exact_qualified_name() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let xml = batch.create_document();
    let html = batch.create_html_document(&xml, None);
    let xml_attr = batch.create_attribute(&xml, "DaTa-Mode");
    let html_attr = batch.create_attribute(&html, "DaTa-Mode");
    let commit = dom.apply(batch, &cancel).unwrap();
    let xml = commit.document(&xml).unwrap();
    let html = commit.document(&html).unwrap();
    let xml_attr = commit.created(&xml_attr).unwrap().unwrap();
    let html_attr = commit.created(&html_attr).unwrap().unwrap();
    let snapshot = dom.snapshot();
    for (id, document, spelling) in [(xml_attr, xml, "DaTa-Mode"), (html_attr, html, "data-mode")] {
        let attr = snapshot.attr(id).unwrap();
        assert_eq!(attr.id().node(), id);
        assert_eq!(attr.owner_document(), document);
        assert_eq!(attr.owner_element(), None);
        assert_eq!(attr.name().local_name(), &spelling.into());
        assert_eq!(attr.name().namespace_uri(), None);
        assert_eq!(attr.name().prefix(), None);
        assert_eq!(attr.value(), &DocumentString::from(""));
        let node = snapshot.node(id).unwrap();
        assert!(matches!(node.kind(), DomNodeKind::Attr(_)));
        assert_eq!(node.parent(), None);
        assert!(node.children().is_empty());
    }
    assert!(matches!(
        snapshot.attr(xml.node()),
        Err(DomError::WrongNodeKind)
    ));

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::create_attribute_ns(
                html,
                Namespace::from_uri("urn:attribute"),
                "p:Token",
            ),
            policy(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("a detached Attr has no external creation effect");
    };
    let id = done.published_node().unwrap();
    drop(lease);
    let snapshot = dom.snapshot();
    let attr = snapshot.attr(id).unwrap();
    assert_eq!(attr.name().local_name(), &"Token".into());
    assert_eq!(attr.name().prefix(), Some(&"p".into()));
    assert_eq!(attr.name().namespace_uri(), Some(&"urn:attribute".into()));
    assert_eq!(attr.owner_document(), html);
    assert_eq!(attr.owner_element(), None);
    assert_eq!(snapshot.node(id).unwrap().node_name(), "p:Token".into());
}

#[test]
fn remove_attribute_node_returns_the_exact_detached_identity_after_atomic_publication() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let element = setup.create_element(&document, "target");
    let other = setup.create_element(&document, "other");
    let raw = DocumentString::from_utf16(vec![0xd800, 0x61, 0]);
    setup.set_attribute(&element, "value", raw.clone());
    let commit = dom.apply(setup, &cancel).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let other = commit.created(&other).unwrap().unwrap();
    let original = dom.snapshot();
    let attr = original
        .node(element)
        .unwrap()
        .get_attribute_node(&"value".into())
        .unwrap()
        .id();

    let mut wrong = DomBatch::new();
    wrong.set_attribute(other, "temporary", "value");
    let foreign_ticket = wrong.remove_attribute_node(other, attr);
    assert!(matches!(dom.apply(wrong, &cancel), Err(DomError::NotFound)));
    assert_eq!(dom.snapshot().stamp(), original.stamp());
    assert!(
        dom.snapshot()
            .node(other)
            .unwrap()
            .attributes()
            .unwrap()
            .is_empty()
    );

    let mut edit = DomBatch::new();
    let removed = edit.remove_attribute_node(element, attr);
    let commit = dom.apply(edit, &cancel).unwrap();
    assert_eq!(commit.removed_attribute(&removed), Ok(attr));
    assert_eq!(
        commit.removed_attribute(&foreign_ticket),
        Err(DomError::ForeignBatch)
    );
    assert_eq!(
        commit.changes().changes(),
        [
            surgeist_dom::DomChange::Attributes(element),
            surgeist_dom::DomChange::AttrOwner(attr),
        ]
    );
    let now = dom.snapshot();
    assert_eq!(now.attr(attr.node()).unwrap().owner_element(), None);
    assert_eq!(now.attr(attr.node()).unwrap().value(), &raw);
    assert_eq!(
        original.attr(attr.node()).unwrap().owner_element(),
        Some(element)
    );

    let mut recreate = DomBatch::new();
    recreate.set_attribute(element, "value", "new");
    dom.apply(recreate, &cancel).unwrap();
    let current = dom
        .snapshot()
        .node(element)
        .unwrap()
        .get_attribute_node(&"value".into())
        .unwrap()
        .id();
    assert_ne!(current, attr);

    let mut retire = DomBatch::new();
    let removed = retire.remove_attribute_node(element, current);
    retire.destroy(current.node());
    let commit = dom.apply(retire, &cancel).unwrap();
    assert_eq!(commit.removed_attribute(&removed), Ok(current));
    assert!(matches!(
        dom.snapshot().attr(current.node()),
        Err(DomError::StaleNode)
    ));

    let mut stale = DomBatch::new();
    stale.remove_attribute_node(element, current);
    assert!(matches!(
        dom.apply(stale, &cancel),
        Err(DomError::StaleNode)
    ));
}

#[test]
fn named_node_map_removal_resolves_each_current_qualified_or_expanded_name() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut setup = DomBatch::new();
    let xml = setup.create_document();
    let html = setup.create_html_document(&xml, None);
    let element = setup.create_element_ns(&html, Namespace::html(), "div");
    let namespace = Namespace::from_uri("urn:named");
    setup.set_attribute_ns(&element, namespace.clone(), "p:item", "namespaced");
    setup.set_attribute(&element, "item", "ordinary");
    let commit = dom.apply(setup, &cancel).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let old = dom.snapshot();
    let map = old.node(element).unwrap().attributes().unwrap();
    let namespaced = map.item(0).unwrap().id();
    let ordinary = map.item(1).unwrap().id();

    let mut missing = DomBatch::new();
    missing.remove_named_item(element, "ITEM");
    missing.remove_named_item_ns(element, namespace.clone(), "ITEM");
    assert!(matches!(
        dom.apply(missing, &cancel),
        Err(DomError::NotFound)
    ));
    assert_eq!(dom.snapshot().stamp(), old.stamp());

    let mut edit = DomBatch::new();
    let removed = edit.remove_named_item(element, "ITEM");
    let commit = dom.apply(edit, &cancel).unwrap();
    assert_eq!(commit.removed_attribute(&removed), Ok(ordinary));
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::remove_named_item_ns(element, namespace, "item"),
            policy(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("a plain Attr removal has no external service");
    };
    assert_eq!(
        done.output(),
        Some(surgeist_dom::DomSourceOutput::RemovedAttribute(namespaced))
    );
    drop(lease);
    let now = dom.snapshot();
    assert!(now.node(element).unwrap().attributes().unwrap().is_empty());
    for (id, value) in [(ordinary, "ordinary"), (namespaced, "namespaced")] {
        let detached = now.attr(id.node()).unwrap();
        assert_eq!(detached.owner_element(), None);
        assert_eq!(detached.value(), &value.into());
        assert_eq!(old.attr(id.node()).unwrap().owner_element(), Some(element));
    }

    let document = now.node(element).unwrap().owner_document().unwrap();
    let mut mixed = DomBatch::new();
    let comment = mixed.create_comment(document, "first");
    mixed.set_attribute(element, "item", "new");
    let removed = mixed.remove_named_item(element, "item");
    let text = mixed.create_text(document, "last");
    let commit = dom.apply(mixed, &cancel).unwrap();
    let now = dom.snapshot();
    assert_eq!(
        now.node(commit.created(&comment).unwrap().unwrap())
            .unwrap()
            .data(),
        Some(&"first".into())
    );
    let removed = commit.removed_attribute(&removed).unwrap();
    assert_ne!(removed, ordinary);
    assert_eq!(now.attr(removed.node()).unwrap().value(), &"new".into());
    assert_eq!(now.attr(removed.node()).unwrap().owner_element(), None);
    assert_eq!(
        now.node(commit.created(&text).unwrap().unwrap())
            .unwrap()
            .data(),
        Some(&"last".into())
    );
}

#[test]
fn native_attr_attachment_distinguishes_replace_unchanged_in_use_and_append() {
    use surgeist_dom::{DomAttributeAttachment, DomSourceOutput};

    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let namespace = Namespace::from_uri("urn:attachment");
    let mut setup = DomBatch::new();
    let source = setup.create_document();
    let destination = setup.create_document();
    let target = setup.create_element(&destination, "target");
    let other = setup.create_element(&source, "other");
    setup.set_attribute(&target, "first", "a");
    setup.set_attribute_ns(&target, namespace.clone(), "old:item", "old");
    setup.set_attribute(&target, "last", "z");
    let incoming = setup.create_attribute_ns(&source, namespace.clone(), "new:item");
    let commit = dom.apply(setup, &cancel).unwrap();
    let source = commit.document(&source).unwrap();
    let destination = commit.document(&destination).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let other = commit.created(&other).unwrap().unwrap();
    let incoming = commit.created(&incoming).unwrap().unwrap();
    let raw = DocumentString::from_utf16(vec![0xd800, 0x41, 0]);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_text_content(incoming, raw.clone()),
            policy(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);

    let before = dom.snapshot();
    let incoming = before.attr(incoming).unwrap().id();
    let previous = before
        .node(target)
        .unwrap()
        .attributes()
        .unwrap()
        .item(1)
        .unwrap()
        .id();
    let mut adopt = DomBatch::new();
    adopt.adopt(previous.node(), source);
    dom.apply(adopt, &cancel).unwrap();
    let before = dom.snapshot();
    let map = before.node(target).unwrap().attributes().unwrap();
    assert_eq!(map.item(1).unwrap().owner_document(), source);
    let mut attach = DomBatch::new();
    let replacement = attach.attach_native_attribute(target, incoming);
    let commit = dom.apply(attach, &cancel).unwrap();
    assert_eq!(
        commit.attribute_attachment(&replacement),
        Ok(DomAttributeAttachment::Replaced(previous))
    );
    let after = dom.snapshot();
    let attached = after.node(target).unwrap().attributes().unwrap();
    assert_eq!(
        attached.iter().map(|attr| attr.id()).collect::<Vec<_>>(),
        [
            map.item(0).unwrap().id(),
            incoming,
            map.item(2).unwrap().id()
        ]
    );
    let current = attached
        .get_named_item_ns(&namespace, &"item".into())
        .unwrap();
    assert_eq!(current.name().prefix(), Some(&"new".into()));
    assert_eq!(current.value(), &raw);
    assert_eq!(current.owner_document(), destination);
    assert_eq!(current.owner_element(), Some(target));
    assert_eq!(after.attr(previous.node()).unwrap().owner_element(), None);
    assert_eq!(
        after.attr(previous.node()).unwrap().owner_document(),
        source
    );
    assert_eq!(
        before.attr(incoming.node()).unwrap().owner_document(),
        source
    );

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::attach_native_attribute(target, incoming),
            policy(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("attaching the same native Attr is complete");
    };
    assert_eq!(
        done.output(),
        Some(DomSourceOutput::AttributeAttachment(
            DomAttributeAttachment::Unchanged(incoming)
        ))
    );
    assert!(done.commit().changes().changes().is_empty());
    drop(lease);
    assert_eq!(dom.snapshot().stamp(), after.stamp());

    let mut in_use = DomBatch::new();
    in_use.attach_native_attribute(other, incoming);
    assert!(matches!(
        dom.apply(in_use, &cancel),
        Err(DomError::InUseAttribute)
    ));
    assert_eq!(dom.snapshot().stamp(), after.stamp());

    let mut append = DomBatch::new();
    append.remove_attribute_node(target, incoming);
    let result = append.attach_native_attribute(other, incoming);
    let commit = dom.apply(append, &cancel).unwrap();
    assert_eq!(
        commit.attribute_attachment(&result),
        Ok(DomAttributeAttachment::Appended(incoming))
    );
    let now = dom.snapshot();
    assert_eq!(now.attr(incoming.node()).unwrap().owner_document(), source);
    assert_eq!(
        now.attr(incoming.node()).unwrap().owner_element(),
        Some(other)
    );
    assert_eq!(now.attr(incoming.node()).unwrap().value(), &raw);
    assert_eq!(now.node(target).unwrap().attributes().unwrap().len(), 2);
}

#[test]
fn native_attr_source_replacement_runs_the_original_nonce_hook() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let target = setup.create_element_ns(&document, Namespace::html(), "div");
    setup.set_attribute(&target, "nonce", "original");
    let incoming = setup.create_attribute(&document, "nonce");
    let commit = dom.apply(setup, &cancel).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let incoming = commit.created(&incoming).unwrap().unwrap();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_text_content(incoming, "new-value!"),
            policy(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    let before = dom.snapshot();
    let target_before = before.node(target).unwrap();
    let previous = target_before
        .get_attribute_node(&"nonce".into())
        .unwrap()
        .id();
    let incoming = before.attr(incoming).unwrap().id();
    assert_eq!(
        target_before.as_element().unwrap().nonce(),
        Some(&"original".into())
    );

    // Document and Element use 62 units. The attached Attr and native slot
    // use 21 units, and the detached incoming Attr uses 15.
    const EXACT_BEFORE: usize = 98;
    assert_eq!(dom.usage().string_code_units(), EXACT_BEFORE);
    let ordinary_limits = dom.limits();
    dom.set_limits(
        DomLimits::new(
            64,
            EXACT_BEFORE,
            128,
            0,
            0,
            64,
            1_000_000,
            ordinary_limits.names(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut refused = DomBatch::new();
    refused.attach_native_attribute(target, incoming);
    assert!(matches!(
        dom.apply(refused, &cancel),
        Err(DomError::ResourceLimit {
            resource: surgeist_dom::DomResource::StringCodeUnits,
            limit: EXACT_BEFORE,
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        dom.snapshot()
            .attr(previous.node())
            .unwrap()
            .owner_element(),
        Some(target)
    );
    assert_eq!(
        dom.snapshot()
            .attr(incoming.node())
            .unwrap()
            .owner_element(),
        None
    );
    dom.set_limits(ordinary_limits).unwrap();

    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::attach_native_attribute(target, incoming),
            policy(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("native nonce replacement has no external service");
    };
    assert_eq!(
        done.output(),
        Some(surgeist_dom::DomSourceOutput::AttributeAttachment(
            surgeist_dom::DomAttributeAttachment::Replaced(previous)
        ))
    );
    drop(lease);
    let now = dom.snapshot();
    let node = now.node(target).unwrap();
    assert_eq!(
        node.get_attribute_node(&"nonce".into()).unwrap().id(),
        incoming
    );
    assert_eq!(
        node.as_element().unwrap().nonce(),
        Some(&"new-value!".into())
    );
    assert_eq!(
        now.attr(previous.node()).unwrap().value(),
        &"original".into()
    );
    assert_eq!(now.attr(previous.node()).unwrap().owner_element(), None);
}

#[test]
fn detached_attr_value_clone_adoption_and_retirement_use_exact_node_identity() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let destination = batch.create_document();
    let element = batch.create_element(&document, "root");
    let attr = batch.create_attribute(&document, "value");
    batch.insert_before(&element, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let destination = commit.document(&destination).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let attr = commit.created(&attr).unwrap().unwrap();
    let value = DocumentString::from_utf16(vec![0xd800, 0x0041, 0]);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::set_text_content(attr, value.clone()),
            policy(),
            &cancel,
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    let original = dom.snapshot();
    assert_eq!(original.attr(attr).unwrap().value(), &value);

    let mut batch = DomBatch::new();
    let range = batch.create_range(document);
    batch.set_range_start(&range, attr, 0);
    batch.set_range_end(&range, attr, 0);
    let commit = dom.apply(batch, &cancel).unwrap();
    let range = commit.created_range(&range).unwrap().unwrap();
    let mut bad_offset = DomBatch::new();
    bad_offset.set_range_start(range, attr, 1);
    assert!(matches!(
        dom.apply(bad_offset, &cancel),
        Err(DomError::IndexSize)
    ));
    let mut bad_insert = DomBatch::new();
    bad_insert.insert_before(attr, element, None);
    assert!(matches!(
        dom.apply(bad_insert, &cancel),
        Err(DomError::HierarchyRequest)
    ));

    let mut transfer = DomBatch::new();
    let cloned = transfer.clone_node(attr, true);
    let imported = transfer.import_node(&original, attr, destination, false);
    transfer.adopt(attr, destination);
    let commit = dom.apply(transfer, &cancel).unwrap();
    let cloned = commit.created(&cloned).unwrap().unwrap();
    let imported = commit.created(&imported).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_ne!(attr, cloned);
    assert_ne!(attr, imported);
    assert_ne!(cloned, imported);
    for (node, owner) in [
        (attr, destination),
        (cloned, document),
        (imported, destination),
    ] {
        let observed = snapshot.attr(node).unwrap();
        assert_eq!(observed.id().node(), node);
        assert_eq!(observed.owner_document(), owner);
        assert_eq!(observed.owner_element(), None);
        assert_eq!(observed.value(), &value);
    }
    assert_eq!(original.attr(attr).unwrap().owner_document(), document);

    let mut foreign = owner();
    let mut transfer = DomBatch::new();
    let receiving_document = transfer.create_document();
    let imported = transfer.import_node(&original, attr, &receiving_document, false);
    let commit = foreign.apply(transfer, &cancel).unwrap();
    let foreign_snapshot = foreign.snapshot();
    let imported = foreign_snapshot
        .attr(commit.created(&imported).unwrap().unwrap())
        .unwrap();
    assert_eq!(
        imported.owner_document(),
        commit.document(&receiving_document).unwrap()
    );
    assert_eq!(imported.value(), &value);
    assert_eq!(imported.name(), original.attr(attr).unwrap().name());
    assert_ne!(
        imported.name().owner(),
        original.attr(attr).unwrap().name().owner()
    );
    assert!(matches!(
        foreign_snapshot.attr(attr),
        Err(DomError::ForeignOwner)
    ));

    let mut retire = DomBatch::new();
    retire.retire_range(range);
    retire.destroy(attr);
    dom.apply(retire, &cancel).unwrap();
    assert!(matches!(
        dom.snapshot().attr(attr),
        Err(DomError::StaleNode)
    ));
    assert_eq!(original.attr(attr).unwrap().value(), &value);
    let mut replacement = DomBatch::new();
    let created = replacement.create_attribute(destination, "value");
    let replacement: NodeId = dom
        .apply(replacement, &cancel)
        .unwrap()
        .created(&created)
        .unwrap()
        .unwrap();
    assert_ne!(attr, replacement);
    assert_eq!(
        dom.snapshot().attr(replacement).unwrap().value(),
        &"".into()
    );
}

#[test]
fn string_attribute_edits_keep_identity_prefix_and_order_until_removal() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let namespace = Namespace::from_uri("urn:ordered");
    let local = DocumentString::from("token");
    let mut initial = DomBatch::new();
    let document = initial.create_document();
    let element = initial.create_element(&document, "root");
    initial.set_attribute(&element, "first", "a");
    initial.set_attribute_ns(&element, namespace.clone(), "p:token", "old");
    initial.set_attribute(&element, "last", "z");
    let commit = dom.apply(initial, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let element = commit.created(&element).unwrap().unwrap();

    let before = dom.snapshot();
    let map = before.node(element).unwrap().attributes().unwrap();
    assert_eq!(map.len(), 3);
    assert!(map.item(3).is_none());
    assert_eq!(map.item(0).unwrap().name().local_name(), &"first".into());
    assert_eq!(map.item(2).unwrap().name().local_name(), &"last".into());
    let old = map.get_named_item_ns(&namespace, &local).unwrap();
    let id = old.id();
    assert_eq!(map.item(1).unwrap().id(), id);
    assert_eq!(map.get_named_item(&"p:token".into()).unwrap().id(), id);
    assert_eq!(old.owner_element(), Some(element));
    assert_eq!(old.owner_document(), document);
    assert_eq!(before.node(id.node()).unwrap().parent(), None);

    let value = DocumentString::from_utf16(vec![0, 0xd800, 0x61]);
    let mut update = DomBatch::new();
    update.set_attribute_ns(element, namespace.clone(), "q:token", value.clone());
    let commit = dom.apply(update, &cancel).unwrap();
    assert!(
        commit
            .changes()
            .changes()
            .contains(&surgeist_dom::DomChange::AttrValue(id))
    );
    assert!(
        commit
            .changes()
            .changes()
            .contains(&surgeist_dom::DomChange::Attributes(element))
    );
    let edited = dom.snapshot();
    let map = edited.node(element).unwrap().attributes().unwrap();
    assert_eq!(map.item(1).unwrap().id(), id);
    assert_eq!(map.item(1).unwrap().value(), &value);
    assert_eq!(map.item(1).unwrap().name().prefix(), Some(&"p".into()));
    assert!(map.get_named_item(&"q:token".into()).is_none());
    assert_eq!(old.value(), &"old".into());

    let mut remove = DomBatch::new();
    remove.remove_attribute_ns(element, namespace.clone(), local.clone());
    dom.apply(remove, &cancel).unwrap();
    let removed = dom.snapshot();
    let detached = removed.attr(id.node()).unwrap();
    assert_eq!(detached.owner_element(), None);
    assert_eq!(detached.owner_document(), document);
    assert_eq!(detached.value(), &value);
    assert_eq!(
        edited.attr(id.node()).unwrap().owner_element(),
        Some(element)
    );

    let mut recreate = DomBatch::new();
    recreate.set_attribute_ns(element, namespace, "q:token", "fresh");
    dom.apply(recreate, &cancel).unwrap();
    let final_snapshot = dom.snapshot();
    let map = final_snapshot.node(element).unwrap().attributes().unwrap();
    assert_eq!(map.item(1).unwrap().name().local_name(), &"last".into());
    assert_eq!(map.item(2).unwrap().name().prefix(), Some(&"q".into()));
    assert_ne!(map.item(2).unwrap().id(), id);
    assert_eq!(final_snapshot.attr(id.node()).unwrap().value(), &value);
}

#[test]
fn attr_document_and_element_custody_are_independent() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut initial = DomBatch::new();
    let source = initial.create_document();
    let second = initial.create_document();
    let third = initial.create_document();
    let element = initial.create_element(&source, "root");
    initial.set_attribute(&element, "owned", "value");
    let commit = dom.apply(initial, &cancel).unwrap();
    let source = commit.document(&source).unwrap();
    let second = commit.document(&second).unwrap();
    let third = commit.document(&third).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let captured = dom.snapshot();
    let attr = captured
        .node(element)
        .unwrap()
        .attributes()
        .unwrap()
        .item(0)
        .unwrap()
        .id();

    let mut invalid = DomBatch::new();
    invalid.destroy(attr.node());
    assert!(matches!(
        dom.apply(invalid, &cancel),
        Err(DomError::InvalidDestruction)
    ));

    let mut adopt = DomBatch::new();
    adopt.adopt(attr.node(), second);
    adopt.adopt(element, source);
    dom.apply(adopt, &cancel).unwrap();
    let divergent = dom.snapshot();
    let view = divergent.attr(attr.node()).unwrap();
    assert_eq!(view.owner_element(), Some(element));
    assert_eq!(view.owner_document(), second);

    let mut adopt = DomBatch::new();
    adopt.adopt(element, third);
    dom.apply(adopt, &cancel).unwrap();
    let reconciled = dom.snapshot();
    let view = reconciled.attr(attr.node()).unwrap();
    assert_eq!(view.owner_element(), Some(element));
    assert_eq!(view.owner_document(), third);

    let mut destroy = DomBatch::new();
    destroy.destroy(element);
    dom.apply(destroy, &cancel).unwrap();
    assert!(matches!(
        dom.snapshot().attr(attr.node()),
        Err(DomError::StaleNode)
    ));
    assert_eq!(captured.attr(attr.node()).unwrap().owner_document(), source);
    assert_eq!(
        reconciled.attr(attr.node()).unwrap().value(),
        &"value".into()
    );
}

#[test]
fn shallow_element_clone_and_foreign_import_create_new_attached_attributes() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let namespace = Namespace::from_uri("urn:copy");
    let value = DocumentString::from_utf16(vec![0xd800, 0, 0x51]);
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let element = setup.create_element(&document, "source");
    setup.set_attribute(&element, "first", "");
    setup.set_attribute_ns(&element, namespace.clone(), "p:token", value.clone());
    let created = dom.apply(setup, &cancel).unwrap();
    let document = created.document(&document).unwrap();
    let element = created.created(&element).unwrap().unwrap();
    let before = dom.snapshot();
    let original = before.node(element).unwrap().attributes().unwrap();
    let original_ids: Vec<_> = original.iter().map(|attr| attr.id()).collect();

    let mut batch = DomBatch::new();
    let clone = batch.clone_node(element, false);
    let created = dom.apply(batch, &cancel).unwrap();
    let clone = created.created(&clone).unwrap().unwrap();
    let after = dom.snapshot();
    let cloned = after.node(clone).unwrap().attributes().unwrap();

    let mut foreign = owner();
    let mut batch = DomBatch::new();
    let target = batch.create_document();
    let import = batch.import_node(&before, element, &target, false);
    let created = foreign.apply(batch, &cancel).unwrap();
    let target = created.document(&target).unwrap();
    let import = created.created(&import).unwrap().unwrap();
    let imported_snapshot = foreign.snapshot();
    let imported = imported_snapshot
        .node(import)
        .unwrap()
        .attributes()
        .unwrap();

    for (map, owner, document) in [(cloned, clone, document), (imported, import, target)] {
        assert_eq!(map.len(), 2);
        for (index, attr) in map.iter().enumerate() {
            assert_ne!(attr.id(), original_ids[index]);
            assert_eq!(attr.name(), original.item(index).unwrap().name());
            assert_eq!(attr.owner_element(), Some(owner));
            assert_eq!(attr.owner_document(), document);
        }
        assert_eq!(map.item(1).unwrap().value(), &value);
    }
    assert_ne!(
        original.item(1).unwrap().name().owner(),
        imported.item(1).unwrap().name().owner()
    );
}
