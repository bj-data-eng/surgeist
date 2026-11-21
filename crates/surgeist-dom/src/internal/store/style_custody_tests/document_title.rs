//! Real CSSOM release interrupts the original string-replace-all continuation.
use super::*;

#[test]
fn document_title_stylesheet_release_survives_drop_and_cancellation_and_finishes_original_tail() {
    use surgeist_cssom::CssomEditGuard;
    use surgeist_html::elements::style::HtmlStyleUpdateAction;
    let cancel = AtomicBool::new(false);
    let Fixture {
        mut dom,
        document,
        element,
        style,
        ..
    } = fixture(0, false);
    let mut batch = DomBatch::new();
    let title =
        batch.create_element_ns(document, surgeist_html::values::Namespace::html(), "title");
    let later = batch.create_comment(document, "remove after Style");
    batch.insert_before(element, document.node(), None);
    batch.insert_before(&title, element, None);
    batch.insert_before(&later, &title, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let title = commit.created(&title).unwrap().unwrap();
    let later = commit.created(&later).unwrap().unwrap();
    let owner = cssom_root(document.node());
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(style, title, Some(later)),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let request = creation_request(&mut lease, &cancel);
        lease.bind_cssom_root(&request, &owner, &cancel).unwrap();
        complete_sheet(&mut lease, &request, &owner, &cancel);
    }
    let before = dom.snapshot();
    let attached = owner.snapshot().unwrap();
    let sheet = attached.sheets()[0].clone();
    let raw = DocumentString::from_utf16([0xd800, 0, 0x20, 0xdc00]);
    let (operation, request) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_document_title(document, raw.clone()),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("title's actual Style descendant must release its associated sheet");
        };
        assert_eq!(
            request.action(),
            DomStyleAction::Update(HtmlStyleUpdateAction::RemoveAssociatedSheet)
        );
        assert_eq!(request.source().element(), style);
        assert_eq!(request.sheet().unwrap().id(), &sheet);
        let snapshot = lease.snapshot();
        // The first removal has reached its native hook, but the remaining
        // child and original replacement have not been processed early.
        assert_eq!(snapshot.node(style).unwrap().parent(), None);
        assert_eq!(snapshot.node(title).unwrap().children(), &[later]);
        (lease.operation(), request)
    };
    assert!(dom.pending_source_edit().unwrap().has_published());
    assert_eq!(dom.snapshot().node(title).unwrap().children(), &[later]);
    let mut lease = dom
        .resume_source_edit(operation, source_limits(), &cancel)
        .unwrap();
    lease.request_cancel();
    let DomSourceProgress::Request(DomSourceRequest::Style(repeated)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the unaccepted original release must remain pending");
    };
    assert_eq!(repeated.source().ordinal(), request.source().ordinal());
    let request = lease
        .qualify_cssom(&request, owner.root(), &attached)
        .unwrap();
    let mut batch = owner
        .batch(CssomEditGuard::from_snapshot(&attached))
        .unwrap();
    batch.remove_sheet(&sheet).unwrap();
    let publication = owner.commit(&mut Some(batch)).unwrap();
    assert!(
        lease
            .accept_cssom_publication(&request, publication, owner.snapshot().unwrap(), None)
            .unwrap()
            .is_none()
    );
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("the retained replacement must finish despite cancellation");
    };
    assert_eq!(done.operation(), operation);
    assert!(done.cancellation_requested());
    let snapshot = lease.snapshot();
    let children = snapshot.node(title).unwrap().children();
    assert_eq!(children.len(), 1);
    assert_eq!(snapshot.node(children[0]).unwrap().data(), Some(&raw));
    assert_eq!(snapshot.node(later).unwrap().parent(), None);
    assert_eq!(
        snapshot
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap(),
        None
    );
    assert_eq!(before.node(title).unwrap().children(), &[style, later]);
    drop(lease);
    assert!(dom.pending_source_edit().is_none());
    assert!(owner.snapshot().unwrap().sheets().is_empty());
}
