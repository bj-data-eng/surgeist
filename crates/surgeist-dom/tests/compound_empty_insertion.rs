//! DOM3071 insert step 3 returns for zero nodes, before children-changed hooks.
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomLimits, DomNameLimits, DomNodeKind,
    DomNodeOrString, DomSourceCompletion, DomSourceEdit, DomSourceLimits, DomSourceProgress,
    DomSourceRequest, DomStyleAction, DomStyleInlineCspObservation, DomStyleRequest, Namespace,
    NodeId,
};
use surgeist_html::elements::style::HtmlStyleUpdateAction;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 4096, 4096, 8)
}

fn deny(request: &DomStyleRequest) -> DomStyleInlineCspObservation {
    // The caller supplies an actual complete policy evaluation for this request.
    DomStyleInlineCspObservation::evaluated(
        request.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    )
}

fn finish_style_update(dom: &mut Dom, edit: DomSourceEdit, style: NodeId) -> DomSourceCompletion {
    let mut lease = dom
        .begin_source_edit(edit, source_limits(), &cancel())
        .unwrap();
    let request = match lease.advance(&cancel()).unwrap() {
        DomSourceProgress::Request(DomSourceRequest::Style(request)) => request,
        other => panic!("the real Style mutation must request inline CSP: {other:?}"),
    };
    assert_eq!(request.source().element(), style);
    assert_eq!(
        request.action(),
        DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
    );
    lease
        .accept_inline_csp(&request, &deny(&request), &cancel())
        .unwrap();
    match lease.advance(&cancel()).unwrap() {
        DomSourceProgress::Complete(done) => done,
        other => panic!("the denied Style update must finish its actual tail: {other:?}"),
    }
}

struct Fixture {
    dom: Dom,
    document: DocumentId,
    foreign_document: DocumentId,
    style: NodeId,
    marker: NodeId,
    fragment: NodeId,
    foreign_fragment: NodeId,
}

fn fixture() -> Fixture {
    let mut dom = Dom::new(
        DomLimits::new(
            128,
            32768,
            512,
            0,
            0,
            16,
            1_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let foreign_document = batch.create_document();
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    let marker = batch.create_comment(&document, "marker");
    let fragment = batch.create_fragment(&document);
    let foreign_fragment = batch.create_fragment(&foreign_document);
    batch.insert_before(&marker, &style, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    let document = commit.document(&document).unwrap();
    let foreign_document = commit.document(&foreign_document).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let marker = commit.created(&marker).unwrap().unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let foreign_fragment = commit.created(&foreign_fragment).unwrap().unwrap();
    finish_style_update(
        &mut dom,
        DomSourceEdit::insert_before(style, document.node(), None),
        style,
    );
    let snapshot = dom.snapshot();
    assert!(snapshot.node(style).unwrap().is_connected());
    assert!(
        snapshot
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert!(dom.pending_source_edit().is_none());
    Fixture {
        dom,
        document,
        foreign_document,
        style,
        marker,
        fragment,
        foreign_fragment,
    }
}

#[derive(Clone, Copy)]
enum Method {
    Append,
    Prepend,
    Before,
    After,
}

fn edit(f: &Fixture, method: Method, nodes: Vec<DomNodeOrString>) -> DomSourceEdit {
    match method {
        Method::Append => DomSourceEdit::append(f.style, nodes),
        Method::Prepend => DomSourceEdit::prepend(f.style, nodes),
        Method::Before => DomSourceEdit::before(f.marker, nodes),
        Method::After => DomSourceEdit::after(f.marker, nodes),
    }
}

fn assert_noop(f: &mut Fixture, edit: DomSourceEdit) {
    let before = f.dom.snapshot();
    let usage = f.dom.usage();
    let mut lease = f
        .dom
        .begin_source_edit(edit, source_limits(), &cancel())
        .unwrap();
    let completion = match lease.advance(&cancel()).unwrap() {
        DomSourceProgress::Complete(done) => done,
        other => panic!(
            "zero inserted nodes must finish without a service request or publication; \
             stamp {:?} -> {:?}, got {other:?}",
            before.stamp(),
            lease.snapshot().stamp(),
        ),
    };
    assert_eq!(completion.commit().changes().from(), before.stamp());
    assert_eq!(completion.commit().changes().to(), before.stamp());
    assert!(completion.commit().changes().changes().is_empty());
    assert!(!completion.cancellation_requested());
    drop(lease);
    let after = f.dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(f.dom.usage(), usage);
    assert!(f.dom.pending_source_edit().is_none());
    assert!(
        f.dom
            .changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    assert_eq!(after.node(f.style).unwrap().children(), [f.marker]);
    assert_eq!(after.node(f.marker).unwrap().parent(), Some(f.style));
    let old_style = before.node(f.style).unwrap().as_style().unwrap();
    let new_style = after.node(f.style).unwrap().as_style().unwrap();
    assert_eq!(new_style.state(), old_style.state());
    assert_eq!(new_style.sheet().unwrap(), old_style.sheet().unwrap());
    assert_eq!(
        new_style.is_render_blocking().unwrap(),
        old_style.is_render_blocking().unwrap()
    );
    for (fragment, document) in [
        (f.fragment, f.document),
        (f.foreign_fragment, f.foreign_document),
    ] {
        let node = after.node(fragment).unwrap();
        assert_eq!(node.owner_document(), Some(document));
        assert_eq!(node.parent(), None);
        assert!(node.children().is_empty());
    }
}

fn empty_arguments(method: Method) {
    let mut f = fixture();
    let invocation = edit(&f, method, vec![]);
    assert_noop(&mut f, invocation);
}

fn empty_fragment(method: Method) {
    let mut f = fixture();
    let invocation = edit(&f, method, vec![f.fragment.into()]);
    assert_noop(&mut f, invocation);
}

#[test]
fn empty_append_does_not_update_connected_style() {
    empty_arguments(Method::Append);
}

#[test]
fn empty_prepend_does_not_update_connected_style() {
    empty_arguments(Method::Prepend);
}

#[test]
fn empty_before_does_not_update_connected_style() {
    empty_arguments(Method::Before);
}

#[test]
fn empty_after_does_not_update_connected_style() {
    empty_arguments(Method::After);
}

#[test]
fn append_empty_fragment_does_not_update_connected_style() {
    empty_fragment(Method::Append);
}

#[test]
fn prepend_empty_fragment_does_not_update_connected_style() {
    empty_fragment(Method::Prepend);
}

#[test]
fn before_empty_fragment_does_not_update_connected_style() {
    empty_fragment(Method::Before);
}

#[test]
fn after_empty_fragment_does_not_update_connected_style() {
    empty_fragment(Method::After);
}

#[test]
fn insert_before_empty_fragment_does_not_update_connected_style() {
    let mut f = fixture();
    let invocation = DomSourceEdit::insert_before(f.fragment, f.style, Some(f.marker));
    assert_noop(&mut f, invocation);
}

#[test]
fn empty_foreign_fragment_keeps_its_document_without_updating_connected_style() {
    // DOM3071 adopts transferred children, never the fragment wrapper itself.
    let mut f = fixture();
    let invocation = DomSourceEdit::insert_before(f.foreign_fragment, f.style, Some(f.marker));
    assert_noop(&mut f, invocation);
}

#[test]
fn empty_string_inserts_one_text_node_and_updates_connected_style() {
    let mut f = fixture();
    let before = f.dom.snapshot();
    let nodes = f.dom.usage().managed_nodes();
    let done = finish_style_update(
        &mut f.dom,
        DomSourceEdit::append(f.style, vec![DocumentString::from("").into()]),
        f.style,
    );
    let after = f.dom.snapshot();
    assert_ne!(after.stamp(), before.stamp());
    let children = after.node(f.style).unwrap().children();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], f.marker);
    let text = after.node(children[1]).unwrap();
    assert!(matches!(text.kind(), DomNodeKind::Text(_)));
    assert_eq!(text.data(), Some(&DocumentString::from("")));
    assert_eq!(text.parent(), Some(f.style));
    assert_eq!(text.owner_document(), Some(f.document));
    assert_eq!(f.dom.usage().managed_nodes(), nodes + 1);
    assert_eq!(done.commit().changes().to(), after.stamp());
    assert!(
        f.dom
            .changes_since(before.stamp())
            .unwrap()
            .changes()
            .contains(&DomChange::Children(f.style))
    );
    assert!(
        after
            .node(f.style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert!(f.dom.pending_source_edit().is_none());
}

#[test]
fn replace_children_with_no_arguments_removes_existing_child_and_updates_style() {
    let mut f = fixture();
    let before = f.dom.snapshot();
    let done = finish_style_update(
        &mut f.dom,
        DomSourceEdit::replace_children(f.style, vec![]),
        f.style,
    );
    let after = f.dom.snapshot();
    assert_ne!(after.stamp(), before.stamp());
    assert!(after.node(f.style).unwrap().children().is_empty());
    assert_eq!(after.node(f.marker).unwrap().parent(), None);
    assert_eq!(done.commit().changes().to(), after.stamp());
    assert!(
        f.dom
            .changes_since(before.stamp())
            .unwrap()
            .changes()
            .contains(&DomChange::Children(f.style))
    );
    assert!(
        after
            .node(f.style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert!(f.dom.pending_source_edit().is_none());
}
