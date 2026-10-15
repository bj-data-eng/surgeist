#![forbid(unsafe_code)]

use std::sync::atomic::AtomicBool;

use surgeist::dom::{Dom, DomBatch, DomLimits, DomSnapshot, Namespace, NodeId};
use surgeist::template::{
    AttributeKind, AttributeRule, ComponentRegistry, NativeElementRegistry, append_static_html,
    parse_template, validate_template,
};

fn attributes(snapshot: &DomSnapshot, node: NodeId) -> Vec<(String, String)> {
    snapshot
        .node(node)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes()
        .iter()
        .map(|attribute| {
            (
                attribute.name().local_name().to_utf8().unwrap(),
                attribute.value().to_utf8().unwrap(),
            )
        })
        .collect()
}

#[test]
fn native_and_template_edits_share_canonical_identity_and_preserve_old_snapshots() {
    let mut dom = Dom::new(DomLimits::new(64, 16384, 128, 8, 65536).unwrap()).unwrap();
    let mut initial = DomBatch::new();
    let document = initial.create_document();
    let root = initial.create_element_ns(&document, Namespace::html(), "div");
    initial.set_attribute(&root, "id", "root");
    initial.set_attribute(&root, "class", "one");
    let first = initial.create_text(&document, "A😀");
    let last = initial.create_text(&document, "tail");
    initial.insert_before(&first, &root, None);
    initial.insert_before(&last, &root, None);
    initial.insert_before(&root, &document, None);
    let committed = dom.apply(initial, &AtomicBool::new(false)).unwrap();
    let document = committed.document(&document).unwrap();
    let root = committed.created(&root).unwrap().unwrap();
    let first = committed.created(&first).unwrap().unwrap();
    let last = committed.created(&last).unwrap().unwrap();
    let old = dom.snapshot();
    assert_eq!(
        old.document(document).unwrap().document_element(),
        Some(root)
    );
    assert!(old.node(root).unwrap().is_connected());
    assert_eq!(old.node(root).unwrap().children(), &[first, last]);
    assert_eq!(
        old.node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .namespace(),
        &Namespace::html()
    );
    assert_eq!(
        old.node(first).unwrap().data().unwrap().code_units(),
        &[65, 0xd83d, 0xde00]
    );
    assert_eq!(
        attributes(&old, root),
        [("id".into(), "root".into()), ("class".into(), "one".into())]
    );

    let authored = parse_template(r#"<span title="authored">new</span>"#).unwrap();
    let native =
        NativeElementRegistry::from_html_metadata(AttributeRule::one(AttributeKind::Static))
            .unwrap();
    let template = validate_template(&authored, &native, &ComponentRegistry::default()).unwrap();
    let mut edit = DomBatch::new();
    edit.set_attribute(root, "class", "two");
    // CharacterData offsets/counts are UTF-16 code units, preserving the A.
    edit.replace_data(first, 1, 2, "Ω");
    let authored_roots =
        append_static_html(&template, &mut edit, document.into(), root.into()).unwrap();
    assert_eq!(authored_roots.len(), 1);
    edit.set_attribute(&authored_roots[0], "id", "joined");
    edit.insert_before(&authored_roots[0], root, Some(last.into()));
    let committed = dom.apply(edit, &AtomicBool::new(false)).unwrap();
    let span = committed.created(&authored_roots[0]).unwrap().unwrap();
    let current = dom.snapshot();
    assert_ne!(current.stamp(), old.stamp());
    assert_eq!(current.owner(), old.owner());
    assert_eq!(current.node(root).unwrap().children(), &[first, span, last]);
    assert_eq!(
        attributes(&current, root),
        [("id".into(), "root".into()), ("class".into(), "two".into())]
    );
    let span_node = current.node(span).unwrap();
    assert_eq!(span_node.owner_document(), Some(document));
    assert!(span_node.is_connected());
    assert_eq!(span_node.node_name().to_utf8().unwrap(), "span");
    assert_eq!(
        span_node.as_element().unwrap().name().namespace(),
        &Namespace::html()
    );
    assert_eq!(
        attributes(&current, span),
        [
            ("title".into(), "authored".into()),
            ("id".into(), "joined".into())
        ]
    );
    assert_eq!(span_node.children().len(), 1);
    assert_eq!(
        current
            .node(span_node.children()[0])
            .unwrap()
            .data()
            .unwrap()
            .to_utf8()
            .unwrap(),
        "new"
    );
    assert_eq!(
        current.node(first).unwrap().data().unwrap().code_units(),
        &[65, 0x03a9]
    );
    assert_eq!(
        current
            .node(last)
            .unwrap()
            .data()
            .unwrap()
            .to_utf8()
            .unwrap(),
        "tail"
    );
    assert_eq!(old.node(root).unwrap().children(), &[first, last]);
    assert_eq!(
        attributes(&old, root),
        [("id".into(), "root".into()), ("class".into(), "one".into())]
    );
    assert_eq!(
        old.node(first).unwrap().data().unwrap().to_utf8().unwrap(),
        "A😀"
    );
    assert!(old.node(span).is_err());
}
