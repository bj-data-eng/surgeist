//! Canonical descendant ID selection and bounded ordinary traversal.

use std::{
    cell::{Cell, RefCell},
    sync::atomic::AtomicBool,
};
use surgeist_dom::{
    Dom, DomBatch, DomCommit, DomError, DomSnapshot,
    edit::CreatedNode,
    identity::NodeId,
    limits::{DomLimits, DomNameLimits},
};
use surgeist_html::values::{DocumentString, Namespace};

fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            128,
            16_384,
            256,
            0,
            0,
            16,
            131_072,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

fn child(batch: &mut DomBatch, doc: &CreatedNode, parent: &CreatedNode) -> CreatedNode {
    let node = batch.create_element_ns(doc, Namespace::html(), "div");
    batch.insert_before(&node, parent, None);
    node
}

fn find(snapshot: &DomSnapshot, root: NodeId, id: &[u16]) -> Option<NodeId> {
    snapshot
        .node(root)
        .unwrap()
        .descendant_element_by_id_with(id, || Ok::<_, DomError>(()), |_| Ok(()))
        .unwrap()
}

#[test]
fn first_exact_descendant_id_tracks_current_order_without_searching_the_root() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let root = child(&mut batch, &doc, &doc);
    let id = [u16::from(b'A'), 0xd800, 0, 0xdfff];
    let exact = DocumentString::from_utf16(id);
    batch.set_attribute_ns(&root, Namespace::none(), "id", exact.clone());

    let wrong = child(&mut batch, &doc, &root);
    batch.set_attribute_ns(&wrong, Namespace::svg(), "s:id", exact.clone());
    batch.set_attribute_ns(&wrong, Namespace::none(), "ID", exact.clone());
    let empty = child(&mut batch, &doc, &root);
    batch.set_attribute_ns(&empty, Namespace::none(), "id", "");

    let first = batch.create_element_ns(&doc, Namespace::svg(), "svg");
    batch.insert_before(&first, &root, None);
    batch.set_attribute_ns(&first, Namespace::none(), "id", exact.clone());
    let second = child(&mut batch, &doc, &root);
    batch.set_attribute_ns(&second, Namespace::none(), "id", exact);

    let fragment = batch.create_fragment(&doc);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&doc).unwrap().node();
    let [root, first, second, fragment] =
        [root, first, second, fragment].map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let before = dom.snapshot();
    assert_eq!(find(&before, document, &id), Some(root));
    assert_eq!(find(&before, root, &id), Some(first));
    assert_eq!(find(&before, fragment, &id), None);
    assert_eq!(find(&before, root, &[]), None);
    assert_eq!(
        find(&before, root, &[u16::from(b'a'), 0xd800, 0, 0xdfff]),
        None
    );

    let mut batch = DomBatch::new();
    batch.insert_before(second, root, Some(first.into()));
    apply(&mut dom, batch);
    assert_eq!(find(&dom.snapshot(), root, &id), Some(second));

    let mut batch = DomBatch::new();
    batch.insert_before(second, fragment, None);
    batch.remove_attribute_ns(first, Namespace::none(), "id");
    apply(&mut dom, batch);
    let moved = dom.snapshot();
    assert_eq!(find(&moved, root, &id), None);
    assert_eq!(find(&moved, fragment, &id), Some(second));
    assert_eq!(find(&before, root, &id), Some(first));
}

#[derive(Debug, Eq, PartialEq)]
enum Refusal {
    Native(DomError),
    Work,
    Scratch,
}

impl From<DomError> for Refusal {
    fn from(error: DomError) -> Self {
        Self::Native(error)
    }
}

struct ScratchDebit<'a> {
    held: &'a Cell<usize>,
    bytes: usize,
}

impl Drop for ScratchDebit<'_> {
    fn drop(&mut self) {
        self.held.set(self.held.get() - self.bytes);
    }
}

#[test]
fn descendant_search_returns_first_refusal_and_releases_current_and_replaced_backing() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let fragment = batch.create_fragment(&doc);
    let branch = child(&mut batch, &doc, &fragment);
    child(&mut batch, &doc, &fragment);
    let nested = child(&mut batch, &doc, &branch);
    child(&mut batch, &doc, &branch);
    let target = child(&mut batch, &doc, &nested);
    batch.set_attribute_ns(&target, Namespace::none(), "id", "x");
    let commit = apply(&mut dom, batch);
    let doc = commit.document(&doc).unwrap();
    let [root, target] = [fragment, target].map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let before = dom.snapshot();

    let read = |snapshot: &DomSnapshot, work_cap: usize, scratch_cap: usize, id: &[u16]| {
        let work = Cell::new(0);
        let held = Cell::new(0);
        let requests = RefCell::new(Vec::new());
        let result = snapshot.node(root).unwrap().descendant_element_by_id_with(
            id,
            || {
                if work.get() == work_cap {
                    return Err(Refusal::Work);
                }
                work.set(work.get() + 1);
                Ok(())
            },
            |bytes| {
                let before = held.get();
                requests.borrow_mut().push((before, bytes));
                if before + bytes > scratch_cap {
                    return Err(Refusal::Scratch);
                }
                held.set(before + bytes);
                Ok(ScratchDebit { held: &held, bytes })
            },
        );
        assert_eq!(held.get(), 0);
        (result, work.get(), requests.into_inner())
    };

    // There are two deferred sibling slices before the first match. The
    // first frame's allocation stays admitted while the two-frame replacement is reserved.
    let frame = size_of::<&[NodeId]>();
    let scratch = 3 * frame;
    let expected_requests = [(0, frame), (frame, 2 * frame)];
    let key = [u16::from(b'x')];
    let (answer, reached, requests) = read(&before, usize::MAX, scratch, &key);
    assert_eq!(answer, Ok(Some(target)));
    assert_eq!(requests, expected_requests);
    assert!(reached > 0);

    let (answer, _, requests) = read(&before, usize::MAX, scratch - 1, &key);
    assert_eq!(answer, Err(Refusal::Scratch));
    assert_eq!(requests, expected_requests);
    let (answer, _, requests) = read(&before, usize::MAX, 0, &[]);
    assert_eq!(answer, Ok(None));
    assert!(requests.is_empty());

    let mut batch = DomBatch::new();
    let late = batch.create_element_ns(doc, Namespace::html(), "p");
    batch.set_attribute_ns(&late, Namespace::none(), "id", "x");
    batch.insert_before(&late, root, None);
    apply(&mut dom, batch);
    let after = dom.snapshot();
    let (answer, used, requests) = read(&after, reached, scratch, &key);
    assert_eq!((answer, used), (Ok(Some(target)), reached));
    assert_eq!(requests, expected_requests);
    let (answer, used, _) = read(&after, reached - 1, scratch, &key);
    assert_eq!((answer, used), (Err(Refusal::Work), reached - 1));
}

#[test]
fn input_suggestions_preserve_first_refusal_and_release_traversal_backing() {
    let mut dom = dom();
    dom.set_limits(dom.limits().with_input_processing(4096, 65_536).unwrap())
        .unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let root = batch.create_fragment(&doc);
    let branch = child(&mut batch, &doc, &root);
    let input = batch.create_element_ns(&doc, Namespace::html(), "input");
    batch.insert_before(&input, &root, None);
    batch.set_attribute(&input, "list", "suggestions");
    let target = batch.create_element_ns(&doc, Namespace::html(), "datalist");
    batch.insert_before(&target, &branch, None);
    batch.set_attribute(&target, "id", "suggestions");
    let commit = apply(&mut dom, batch);
    let [input, target] = [input, target].map(|ticket| commit.created(&ticket).unwrap().unwrap());

    let read = |snapshot: &DomSnapshot, work_cap, scratch_cap| {
        let attempted = Cell::new(0);
        let held = Cell::new(0);
        let requested = Cell::new(0);
        let result = snapshot
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .suggestions_source_with(
                || {
                    let previous = attempted.get();
                    attempted.set(previous + 1);
                    if previous == work_cap {
                        return Err(Refusal::Work);
                    }
                    Ok(())
                },
                |bytes| {
                    requested.set(bytes);
                    if bytes > scratch_cap {
                        return Err(Refusal::Scratch);
                    }
                    held.set(held.get() + bytes);
                    Ok(ScratchDebit { held: &held, bytes })
                },
            );
        assert_eq!(held.get(), 0);
        (result, attempted.get(), requested.get())
    };

    let before = dom.snapshot();
    let (found, work, scratch) = read(&before, usize::MAX, usize::MAX);
    assert_eq!(found, Ok(Some(target)));
    assert!(scratch > 0);
    assert_eq!(read(&before, work, scratch).0, Ok(Some(target)));
    assert_eq!(read(&before, work, scratch - 1).0, Err(Refusal::Scratch));
    assert_eq!(read(&before, 0, 0), (Err(Refusal::Work), 1, 0));
    for cap in 0..work {
        let (result, attempted, _) = read(&before, cap, scratch);
        assert_eq!(result, Err(Refusal::Work));
        assert_eq!(attempted, cap + 1);
    }

    let mut batch = DomBatch::new();
    batch.set_attribute(input, "type", "password");
    apply(&mut dom, batch);
    assert_eq!(read(&dom.snapshot(), usize::MAX, 0).0, Ok(None));

    let mut batch = DomBatch::new();
    batch.set_attribute(input, "type", "text");
    batch.set_attribute(input, "list", "");
    apply(&mut dom, batch);
    assert_eq!(read(&dom.snapshot(), usize::MAX, 0).0, Ok(None));
    assert_eq!(read(&before, work, scratch).0, Ok(Some(target)));
}
