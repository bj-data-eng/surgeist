//! Native selection, exact replacement, canonical identity and refusal oracles.
use super::*;

fn tree(namespace: Namespace, root_name: &str) -> (Dom, DocumentId, NodeId) {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, namespace, root_name);
    batch.insert_before(&root, &document, None);
    let commit = apply(&mut dom, batch);
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&root).unwrap().unwrap(),
    )
}
fn set(dom: &mut Dom, document: DocumentId, value: impl Into<DocumentString>) {
    complete(
        dom,
        DomSourceEdit::set_document_title(document, value.into()),
    );
}
fn text(dom: &Dom, title: NodeId) -> DocumentString {
    let children = children(dom, title);
    assert_eq!(children.len(), 1);
    dom.snapshot()
        .node(children[0])
        .unwrap()
        .data()
        .unwrap()
        .clone()
}

#[test]
fn document_title_html_uses_first_ordinary_title_outside_head_and_original_range_replacement() {
    let (mut dom, document, root) = tree(Namespace::html(), "h:html");
    let mut batch = DomBatch::new();
    let body = batch.create_element_ns(document, Namespace::html(), "body");
    let first = batch.create_element_ns(document, Namespace::html(), "p:title");
    let old = batch.create_text(document, "old");
    let nested = batch.create_element_ns(document, Namespace::html(), "span");
    let second = batch.create_element_ns(document, Namespace::html(), "title");
    batch.insert_before(&body, root, None);
    batch.insert_before(&first, &body, None);
    batch.insert_before(&old, &first, None);
    batch.insert_before(&nested, &first, None);
    batch.insert_before(&second, root, None);
    let range = batch.create_range(document);
    batch.set_range_start(&range, &old, 2);
    batch.set_range_end(&range, &first, 2);
    let commit = apply(&mut dom, batch);
    let [first, old, nested, second] =
        [&first, &old, &nested, &second].map(|n| commit.created(n).unwrap().unwrap());
    let range = commit.created_range(&range).unwrap().unwrap();
    let before = dom.snapshot();
    let raw = DocumentString::from_utf16([0x20, 0xd800, 0, 0xdc00, 0x09]);
    set(&mut dom, document, raw.clone());
    assert_eq!(text(&dom, first), raw);
    assert!(children(&dom, second).is_empty());
    assert_eq!(dom.snapshot().node(old).unwrap().parent(), None);
    assert_eq!(dom.snapshot().node(nested).unwrap().parent(), None);
    assert_eq!(before.node(old).unwrap().parent(), Some(first));
    let snapshot = dom.snapshot();
    let range = snapshot.range(range).unwrap();
    assert_eq!(
        (range.start().container(), range.start().offset()),
        (first, 0)
    );
    assert_eq!((range.end().container(), range.end().offset()), (first, 0));
    set(&mut dom, document, "");
    assert!(children(&dom, first).is_empty());
}

#[test]
fn document_title_html_root_itself_can_match_and_any_html_root_can_select_existing_title() {
    for root_name in ["x:title", "odd-root"] {
        let (mut dom, document, root) = tree(Namespace::html(), root_name);
        let title = if root_name == "x:title" {
            root
        } else {
            let mut batch = DomBatch::new();
            let title = batch.create_element_ns(document, Namespace::html(), "title");
            batch.insert_before(&title, root, None);
            apply(&mut dom, batch).created(&title).unwrap().unwrap()
        };
        set(&mut dom, document, "selected");
        assert_eq!(text(&dom, title), DocumentString::from("selected"));
    }
}

#[test]
fn document_title_svg_only_selects_first_direct_exact_svg_title() {
    let (mut dom, document, root) = tree(Namespace::svg(), "s:svg");
    let mut batch = DomBatch::new();
    let group = batch.create_element_ns(document, Namespace::svg(), "g");
    let nested = batch.create_element_ns(document, Namespace::svg(), "title");
    let html = batch.create_element_ns(document, Namespace::html(), "title");
    let wrong_case = batch.create_element_ns(document, Namespace::svg(), "Title");
    let first = batch.create_element_ns(document, Namespace::svg(), "s:title");
    let second = batch.create_element_ns(document, Namespace::svg(), "title");
    batch.insert_before(&group, root, None);
    batch.insert_before(&nested, &group, None);
    for node in [&html, &wrong_case, &first, &second] {
        batch.insert_before(node, root, None);
    }
    let commit = apply(&mut dom, batch);
    let [nested, html, wrong_case, first, second] =
        [&nested, &html, &wrong_case, &first, &second].map(|n| commit.created(n).unwrap().unwrap());
    set(&mut dom, document, "svg");
    assert_eq!(text(&dom, first), DocumentString::from("svg"));
    for node in [nested, html, wrong_case, second] {
        assert!(children(&dom, node).is_empty());
    }
}

#[test]
fn document_title_creates_svg_first_child_even_before_comment_and_empty_value_creates_title() {
    for value in ["", "exact"] {
        let (mut dom, document, root) = tree(Namespace::svg(), "svg");
        let mut batch = DomBatch::new();
        let comment = batch.create_comment(document, "first");
        batch.insert_before(&comment, root, None);
        let comment = apply(&mut dom, batch).created(&comment).unwrap().unwrap();
        set(&mut dom, document, value);
        let order = children(&dom, root);
        assert_eq!(order.len(), 2);
        assert_eq!(order[1], comment);
        let snapshot = dom.snapshot();
        let title = snapshot.node(order[0]).unwrap().as_element().unwrap();
        assert_eq!(title.name().namespace_uri(), Namespace::svg().uri());
        assert_eq!(title.name().local_name(), &DocumentString::from("title"));
        if value.is_empty() {
            assert!(children(&dom, order[0]).is_empty());
        } else {
            assert_eq!(text(&dom, order[0]), DocumentString::from(value));
        }
    }
}

#[test]
fn document_title_creation_uses_first_direct_html_head_without_creating_one_or_crossing_namespaces()
{
    let (mut dom, document, root) = tree(Namespace::html(), "h:html");
    let mut batch = DomBatch::new();
    let wrong = batch.create_element_ns(document, Namespace::svg(), "head");
    let upper = batch.create_element_ns(document, Namespace::html(), "Head");
    let first = batch.create_element_ns(document, Namespace::html(), "h:head");
    let second = batch.create_element_ns(document, Namespace::html(), "head");
    for node in [&wrong, &upper, &first, &second] {
        batch.insert_before(node, root, None);
    }
    let comment = batch.create_comment(document, "keep");
    batch.insert_before(&comment, &first, None);
    let commit = apply(&mut dom, batch);
    let [wrong, upper, first, second, comment] =
        [&wrong, &upper, &first, &second, &comment].map(|n| commit.created(n).unwrap().unwrap());
    set(&mut dom, document, "");
    let order = children(&dom, first);
    assert_eq!(order.len(), 2);
    assert_eq!(order[0], comment);
    assert!(children(&dom, order[1]).is_empty());
    for node in [wrong, upper, second] {
        assert!(children(&dom, node).is_empty());
    }
    let snapshot = dom.snapshot();
    let title = snapshot.node(order[1]).unwrap().as_element().unwrap();
    assert_eq!(title.name().namespace_uri(), Namespace::html().uri());
    assert_eq!(title.name().local_name(), &DocumentString::from("title"));
}

#[test]
fn document_title_missing_head_wrong_root_and_other_namespaces_are_canonical_noops() {
    for (namespace, name) in [
        (Namespace::html(), "html"),
        (Namespace::html(), "HTML"),
        (Namespace::html(), "other"),
        (Namespace::svg(), "SVG"),
        (Namespace::none(), "html"),
        (Namespace::from_uri("urn:foreign"), "html"),
    ] {
        let (mut dom, document, root) = tree(namespace, name);
        let mut batch = DomBatch::new();
        let container = batch.create_element_ns(document, Namespace::html(), "div");
        let head = batch.create_element_ns(document, Namespace::html(), "head");
        batch.insert_before(&container, root, None);
        batch.insert_before(&head, &container, None);
        apply(&mut dom, batch);
        let before = dom.snapshot().stamp();
        let usage = dom.usage();
        set(&mut dom, document, "ignored");
        assert_eq!(dom.snapshot().stamp(), before);
        assert_eq!(dom.usage(), usage);
    }
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let document = apply(&mut dom, batch).document(&document).unwrap();
    let before = dom.snapshot().stamp();
    set(&mut dom, document, "ignored");
    assert_eq!(dom.snapshot().stamp(), before);
}

#[test]
fn document_title_nonempty_creation_refusal_never_leaves_an_empty_title() {
    for namespace in [Namespace::html(), Namespace::svg()] {
        let svg = namespace == Namespace::svg();
        let (mut dom, document, root) = tree(namespace, if svg { "svg" } else { "html" });
        if !svg {
            let mut batch = DomBatch::new();
            let head = batch.create_element_ns(document, Namespace::html(), "head");
            batch.insert_before(&head, root, None);
            apply(&mut dom, batch);
        }
        let usage = dom.usage();
        let mut bounds = limits();
        bounds.nodes = usage.managed_nodes() + 1;
        dom.set_limits(bounds).unwrap();
        let before = dom.snapshot().stamp();
        {
            let mut lease = dom
                .begin_source_edit(
                    DomSourceEdit::set_document_title(document, "nonempty".into()),
                    source_limits(),
                    &cancel(),
                )
                .unwrap();
            assert!(matches!(
                lease.advance(&cancel()),
                Err(DomError::ResourceLimit {
                    resource: DomResource::ManagedNodes,
                    ..
                })
            ));
            assert_eq!(lease.snapshot().stamp(), before);
        }
        assert_eq!(dom.snapshot().stamp(), before);
        assert_eq!(dom.usage(), usage);
        assert!(dom.pending_source_edit().is_none());
    }
}

#[test]
fn document_title_search_never_enters_shadow_or_template_contents() {
    let (mut dom, document, root) = tree(Namespace::html(), "html");
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let hidden = batch.create_element_ns(document, Namespace::html(), "title");
    batch.insert_before(&hidden, &shadow, None);
    let template = batch.create_element_ns(document, Namespace::html(), "template");
    let head = batch.create_element_ns(document, Namespace::html(), "head");
    for node in [&host, &template, &head] {
        batch.insert_before(node, root, None);
    }
    let commit = apply(&mut dom, batch);
    let [hidden, template, head] =
        [&hidden, &template, &head].map(|n| commit.created(n).unwrap().unwrap());
    let snapshot = dom.snapshot();
    let contents = snapshot
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let owner = snapshot.node(contents).unwrap().owner_document().unwrap();
    let mut batch = DomBatch::new();
    let inert = batch.create_element_ns(owner, Namespace::html(), "title");
    batch.insert_before(&inert, contents, None);
    let inert = apply(&mut dom, batch).created(&inert).unwrap().unwrap();
    set(&mut dom, document, "ordinary");
    for node in [hidden, inert] {
        assert!(children(&dom, node).is_empty());
    }
    let title = children(&dom, head)[0];
    assert_eq!(text(&dom, title), DocumentString::from("ordinary"));
}

#[test]
fn document_title_native_html_document_uses_existing_head_and_reuses_created_title_identity() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let associated = batch.create_document();
    let associated = apply(&mut dom, batch).document(&associated).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_html_document(associated, None);
    let document = apply(&mut dom, batch).document(&document).unwrap();
    let root = dom
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let head = children(&dom, root)[0];
    set(&mut dom, document, "");
    let title = children(&dom, head)[0];
    assert!(children(&dom, title).is_empty());
    set(&mut dom, document, "stable");
    assert_eq!(children(&dom, head), [title]);
    assert_eq!(text(&dom, title), DocumentString::from("stable"));
}

#[test]
fn document_title_non_html_and_non_svg_roots_do_not_update_an_existing_html_title() {
    for (namespace, local) in [
        (Namespace::none(), "html"),
        (Namespace::svg(), "SVG"),
        (Namespace::svg(), "g"),
        (Namespace::from_uri("urn:foreign"), "html"),
    ] {
        let (mut dom, document, root) = tree(namespace, local);
        let mut batch = DomBatch::new();
        let title = batch.create_element_ns(document, Namespace::html(), "title");
        batch.insert_before(&title, root, None);
        let title = apply(&mut dom, batch).created(&title).unwrap().unwrap();
        let before = dom.snapshot().stamp();
        set(&mut dom, document, "ignored");
        assert_eq!(dom.snapshot().stamp(), before);
        assert!(children(&dom, title).is_empty());
    }
}

#[test]
fn document_title_admits_original_utf16_capacity_even_for_a_noop_root() {
    let (mut dom, document, _) = tree(Namespace::none(), "foreign");
    let before = dom.snapshot().stamp();
    let policy = DomSourceLimits::new(1_048_576, 2_000_000, 20_000_000, 128, 4096, 4096, 8);
    let mut value = Vec::with_capacity(1_048_576);
    value.push(0xd800);
    assert!(matches!(
        dom.begin_source_edit(
            DomSourceEdit::set_document_title(document, DocumentString::from_utf16(value)),
            policy,
            &cancel()
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    assert!(dom.pending_source_edit().is_none());
}
