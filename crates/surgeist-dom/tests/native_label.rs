//! Live Label targets and their stored Form owners through bounded native reads.

use std::{
    cell::{Cell, RefCell},
    sync::atomic::AtomicBool,
};
use surgeist_dom::{
    Dom, DomBatch, DomCommit, DomError, DomSnapshot,
    edit::CreatedNode,
    identity::NodeId,
    limits::{DomLimits, DomNameLimits, DomResource},
    tree::DomNodeList,
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

fn append(
    batch: &mut DomBatch,
    doc: &CreatedNode,
    parent: &CreatedNode,
    name: &str,
) -> CreatedNode {
    let child = batch.create_element_ns(doc, Namespace::html(), name);
    batch.insert_before(&child, parent, None);
    child
}

fn relation(snapshot: &DomSnapshot, label: NodeId) -> (Option<NodeId>, Option<NodeId>) {
    let label = snapshot.node(label).unwrap();
    let step = || Ok::<_, DomError>(());
    let reserve = |_| Ok(());
    let control = label.html_labeled_control_with(step, reserve).unwrap();
    let form = label
        .html_label_form_owner_with(step, reserve)
        .unwrap()
        .map(|owner| owner.id());
    (control, form)
}

fn assert_labels(dom: &Dom, list: &DomNodeList, expected: &[NodeId]) {
    let cancel = AtomicBool::new(false);
    let read = dom
        .read_node_list(list, 100_000, || Ok::<_, DomError>(()), &cancel)
        .unwrap();
    assert_eq!(read.length().unwrap(), expected.len());
    for (index, &id) in expected.iter().enumerate() {
        assert_eq!(read.item(index).unwrap(), Some(id));
    }
    assert_eq!(read.item(expected.len()).unwrap(), None);
}

#[test]
fn reverse_labels_follow_the_current_inclusive_ordinary_root_and_input_type() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "label");
    let duplicate = append(&mut batch, &document, &root, "div");
    let input = append(&mut batch, &document, &root, "input");
    let explicit = append(&mut batch, &document, &root, "s:label");
    let exact = DocumentString::from_utf16([0x0041, 0xd800, 0x0000]);
    batch.set_attribute(&duplicate, "id", exact.clone());
    batch.set_attribute(&input, "id", exact.clone());
    batch.set_attribute(&explicit, "for", exact);
    let created = apply(&mut dom, batch);
    let [root, duplicate, input, explicit] =
        [root, duplicate, input, explicit].map(|ticket| created.created(&ticket).unwrap().unwrap());
    let cancel = AtomicBool::new(false);
    let list = dom.control_labels(input, &cancel).unwrap().unwrap();
    assert_labels(&dom, &list, &[root]);

    let mut batch = DomBatch::new();
    batch.detach(duplicate);
    apply(&mut dom, batch);
    assert_labels(&dom, &list, &[root, explicit]);

    let mut batch = DomBatch::new();
    batch.set_attribute(input, "type", "HiDdEn");
    apply(&mut dom, batch);
    assert!(dom.control_labels(input, &cancel).unwrap().is_none());
    assert_labels(&dom, &list, &[]);

    let mut batch = DomBatch::new();
    batch.set_attribute(input, "type", "invalid type");
    apply(&mut dom, batch);
    assert_labels(&dom, &list, &[root, explicit]);

    let mut batch = DomBatch::new();
    batch.set_attribute(input, "id", "different");
    apply(&mut dom, batch);
    assert_labels(&dom, &list, &[root]);
}

#[test]
fn reverse_label_lists_follow_adoption_and_refuse_retired_generations() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let first = batch.create_document();
    let second = batch.create_document();
    let first_root = append(&mut batch, &first, &first, "div");
    let second_root = append(&mut batch, &second, &second, "div");
    let first_label = append(&mut batch, &first, &first_root, "label");
    let second_label = append(&mut batch, &second, &second_root, "label");
    let control = append(&mut batch, &first, &first_root, "button");
    batch.set_attribute(&control, "id", "key");
    batch.set_attribute(&first_label, "for", "key");
    batch.set_attribute(&second_label, "for", "key");
    let created = apply(&mut dom, batch);
    let second = created.document(&second).unwrap();
    let [control, first_label, second_label, second_root] =
        [control, first_label, second_label, second_root]
            .map(|ticket| created.created(&ticket).unwrap().unwrap());
    let cancel = AtomicBool::new(false);
    let list = dom.control_labels(control, &cancel).unwrap().unwrap();
    assert_labels(&dom, &list, &[first_label]);

    let mut batch = DomBatch::new();
    batch.adopt(control, second);
    batch.insert_before(control, second_root, None);
    apply(&mut dom, batch);
    assert_labels(&dom, &list, &[second_label]);

    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(control);
    batch.destroy(control);
    apply(&mut dom, batch);
    assert!(matches!(
        dom.read_node_list(&list, 100_000, || Ok::<_, DomError>(()), &cancel),
        Err(DomError::StaleNode)
    ));
    assert_eq!(relation(&old, second_label).0, Some(control));
}

#[test]
fn reverse_implicit_label_follows_the_first_labelable_descendant() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let label = batch.create_element_ns(&doc, Namespace::html(), "label");
    let earlier = append(&mut batch, &doc, &label, "input");
    let later = append(&mut batch, &doc, &label, "textarea");
    let created = apply(&mut dom, batch);
    let [label, earlier, later] =
        [label, earlier, later].map(|ticket| created.created(&ticket).unwrap().unwrap());
    let cancel = AtomicBool::new(false);
    let list = dom.control_labels(later, &cancel).unwrap().unwrap();
    assert_labels(&dom, &list, &[]);

    let mut batch = DomBatch::new();
    batch.set_attribute(earlier, "type", "hidden");
    apply(&mut dom, batch);
    assert_labels(&dom, &list, &[label]);
}

#[test]
fn reverse_label_reads_stop_at_the_requested_prefix_and_preserve_refusals() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let root = append(&mut batch, &doc, &doc, "div");
    let label = append(&mut batch, &doc, &root, "label");
    let control = append(&mut batch, &doc, &label, "meter");
    let other = append(&mut batch, &doc, &root, "label");
    for _ in 0..12 {
        append(&mut batch, &doc, &other, "span");
    }
    let created = apply(&mut dom, batch);
    let [label, control] =
        [label, control].map(|ticket| created.created(&ticket).unwrap().unwrap());
    let cancel = AtomicBool::new(false);
    assert!(matches!(
        dom.control_labels(label, &cancel),
        Err(DomError::WrongNodeKind)
    ));
    let list = dom.control_labels(control, &cancel).unwrap().unwrap();
    let calls = Cell::new(0_u64);
    let measured = dom
        .read_node_list(
            &list,
            100_000,
            || {
                calls.set(calls.get() + 1);
                Ok::<_, DomError>(())
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(measured.item(0).unwrap(), Some(label));
    let prefix = calls.get();
    assert_eq!(
        dom.read_node_list(&list, prefix, || Ok::<_, DomError>(()), &cancel)
            .unwrap()
            .item(0),
        Ok(Some(label))
    );
    assert_eq!(
        dom.read_node_list(&list, prefix - 1, || Ok::<_, DomError>(()), &cancel)
            .unwrap()
            .item(0),
        Err(DomError::WorkLimit {
            resource: DomResource::NodeListWork,
            limit: prefix - 1,
        })
    );
    assert_eq!(
        dom.read_node_list(&list, prefix, || Ok::<_, DomError>(()), &cancel)
            .unwrap()
            .length(),
        Err(DomError::WorkLimit {
            resource: DomResource::NodeListWork,
            limit: prefix,
        })
    );
    let calls = Cell::new(0_u64);
    let read = dom
        .read_node_list(
            &list,
            prefix,
            || {
                if calls.get() == prefix - 1 {
                    return Err(ReadError::Work);
                }
                calls.set(calls.get() + 1);
                Ok(())
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(read.item(0), Err(ReadError::Work));
    let read = dom
        .read_node_list(&list, 100_000, || Ok::<_, DomError>(()), &cancel)
        .unwrap();
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(read.item(0), Err(DomError::Cancelled));
}

#[test]
fn label_uses_the_first_exact_id_then_checks_labelability_and_current_form() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let root = append(&mut batch, &doc, &doc, "div");
    let label = append(&mut batch, &doc, &root, "h:label");
    let early = append(&mut batch, &doc, &label, "input");
    batch.set_attribute(&early, "type", "HiDdEn");
    let nested = append(&mut batch, &doc, &label, "span");
    let implicit = append(&mut batch, &doc, &nested, "button");
    batch.set_attribute(&implicit, "id", "");

    let exact = DocumentString::from_utf16([u16::from(b'A'), 0xd800, 0]);
    let foreign = batch.create_element_ns(&doc, Namespace::svg(), "input");
    batch.insert_before(&foreign, &root, None);
    batch.set_attribute_ns(&foreign, Namespace::svg(), "s:id", exact.clone());
    let nonlabelable = append(&mut batch, &doc, &root, "div");
    batch.set_attribute(&nonlabelable, "id", exact.clone());
    let hidden = append(&mut batch, &doc, &root, "input");
    batch.set_attribute(&hidden, "id", exact.clone());
    batch.set_attribute(&hidden, "type", "hidden");
    let first_form = append(&mut batch, &doc, &root, "form");
    let first = append(&mut batch, &doc, &first_form, "input");
    batch.set_attribute(&first, "id", exact.clone());
    let next_form = append(&mut batch, &doc, &root, "form");
    batch.set_attribute(&next_form, "id", "other-owner");
    let next = append(&mut batch, &doc, &next_form, "input");
    batch.set_attribute(&next, "id", exact.clone());

    batch.set_attribute_ns(&label, Namespace::svg(), "s:for", exact.clone());
    batch.set_attribute_ns(&label, Namespace::none(), "FOR", exact.clone());
    let commit = apply(&mut dom, batch);
    let [
        label,
        implicit,
        foreign,
        nonlabelable,
        hidden,
        first_form,
        first,
        next_form,
        next,
    ] = [
        label,
        implicit,
        foreign,
        nonlabelable,
        hidden,
        first_form,
        first,
        next_form,
        next,
    ]
    .map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let implicit_snapshot = dom.snapshot();
    assert_eq!(relation(&implicit_snapshot, label), (Some(implicit), None));

    let mut batch = DomBatch::new();
    batch.set_attribute_ns(label, Namespace::none(), "for", "");
    apply(&mut dom, batch);
    let empty = dom.snapshot();
    let label_node = empty.node(label).unwrap();
    let step = || Ok::<_, ReadError>(());
    let no_scratch = |_| Err::<(), _>(ReadError::Scratch);
    assert_eq!(
        label_node.html_labeled_control_with(step, no_scratch),
        Ok(None)
    );
    assert!(
        label_node
            .html_label_form_owner_with(step, no_scratch)
            .unwrap()
            .is_none()
    );

    let mut batch = DomBatch::new();
    batch.set_attribute(
        label,
        "for",
        DocumentString::from_utf16([u16::from(b'a'), 0xd800, 0]),
    );
    apply(&mut dom, batch);
    assert_eq!(relation(&dom.snapshot(), label), (None, None));

    let mut batch = DomBatch::new();
    batch.set_attribute(label, "for", exact.clone());
    apply(&mut dom, batch);
    assert_eq!(relation(&dom.snapshot(), label), (None, None));

    let mut batch = DomBatch::new();
    batch.detach(nonlabelable);
    apply(&mut dom, batch);
    assert_eq!(relation(&dom.snapshot(), label), (None, None));

    let mut batch = DomBatch::new();
    batch.set_attribute(hidden, "type", "text");
    apply(&mut dom, batch);
    assert_eq!(relation(&dom.snapshot(), label), (Some(hidden), None));

    let mut batch = DomBatch::new();
    batch.set_attribute_ns(foreign, Namespace::none(), "id", exact);
    apply(&mut dom, batch);
    assert_eq!(relation(&dom.snapshot(), label), (None, None));

    let mut batch = DomBatch::new();
    batch.detach(foreign);
    batch.detach(hidden);
    apply(&mut dom, batch);
    let original = dom.snapshot();
    assert_eq!(relation(&original, label), (Some(first), Some(first_form)));

    let mut batch = DomBatch::new();
    batch.set_attribute(first, "form", "other-owner");
    apply(&mut dom, batch);
    let reassociated = dom.snapshot();
    assert_eq!(
        relation(&reassociated, label),
        (Some(first), Some(next_form))
    );
    assert_eq!(relation(&original, label), (Some(first), Some(first_form)));

    let mut batch = DomBatch::new();
    batch.remove_attribute_ns(next_form, Namespace::none(), "id");
    apply(&mut dom, batch);
    assert_eq!(relation(&dom.snapshot(), label), (Some(first), None));

    let mut batch = DomBatch::new();
    batch.detach(first_form);
    apply(&mut dom, batch);
    assert_eq!(
        relation(&dom.snapshot(), label),
        (Some(next), Some(next_form))
    );
    assert_eq!(relation(&implicit_snapshot, label), (Some(implicit), None));
}

#[test]
fn explicit_for_can_resolve_the_inclusive_detached_root() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let root = batch.create_element_ns(&doc, Namespace::html(), "button");
    batch.set_attribute(&root, "id", "same");
    let label = append(&mut batch, &doc, &root, "label");
    let descendant = append(&mut batch, &doc, &label, "progress");
    batch.set_attribute(&descendant, "id", "same");
    batch.set_attribute(&label, "for", "same");
    let commit = apply(&mut dom, batch);
    let [root, label, descendant] =
        [root, label, descendant].map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let detached = dom.snapshot();
    assert!(detached.node(root).unwrap().parent().is_none());
    assert_eq!(relation(&detached, label), (Some(root), None));
    let cancel = AtomicBool::new(false);
    let root_labels = dom.control_labels(root, &cancel).unwrap().unwrap();
    let descendant_labels = dom.control_labels(descendant, &cancel).unwrap().unwrap();
    assert_labels(&dom, &root_labels, &[label]);
    assert_labels(&dom, &descendant_labels, &[]);

    let mut batch = DomBatch::new();
    batch.detach(label);
    apply(&mut dom, batch);
    assert_eq!(relation(&dom.snapshot(), label), (Some(descendant), None));
    assert_labels(&dom, &root_labels, &[]);
    assert_labels(&dom, &descendant_labels, &[label]);

    let mut batch = DomBatch::new();
    batch.remove_attribute_ns(descendant, Namespace::none(), "id");
    apply(&mut dom, batch);
    assert_eq!(relation(&dom.snapshot(), label), (None, None));
}

#[derive(Debug, PartialEq)]
enum ReadError {
    Native(DomError),
    Work,
    Scratch,
}

impl From<DomError> for ReadError {
    fn from(error: DomError) -> Self {
        Self::Native(error)
    }
}

struct Backing<'a> {
    bytes: usize,
    retained: &'a Cell<usize>,
}

impl Drop for Backing<'_> {
    fn drop(&mut self) {
        self.retained.set(self.retained.get() - self.bytes);
    }
}

#[derive(Default)]
struct Admission {
    work: Cell<usize>,
    retained: Cell<usize>,
    requested: RefCell<Vec<(usize, usize)>>,
}

impl Admission {
    fn find(
        &self,
        snapshot: &DomSnapshot,
        label: NodeId,
        (work_cap, scratch_cap): (usize, usize),
    ) -> Result<Option<NodeId>, ReadError> {
        let result = snapshot.node(label).unwrap().html_labeled_control_with(
            || {
                if self.work.get() == work_cap {
                    return Err(ReadError::Work);
                }
                self.work.set(self.work.get() + 1);
                Ok(())
            },
            |bytes| {
                let before = self.retained.get();
                self.requested.borrow_mut().push((before, bytes));
                if before + bytes > scratch_cap {
                    return Err(ReadError::Scratch);
                }
                self.retained.set(before + bytes);
                Ok(Backing {
                    bytes,
                    retained: &self.retained,
                })
            },
        );
        assert_eq!(self.retained.get(), 0);
        result
    }
}

#[test]
fn first_labelable_descendant_stops_before_suffix_and_releases_admitted_scratch() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let label = batch.create_element_ns(&doc, Namespace::html(), "label");
    let outer = append(&mut batch, &doc, &label, "div");
    append(&mut batch, &doc, &label, "div");
    let inner = append(&mut batch, &doc, &outer, "div");
    append(&mut batch, &doc, &outer, "div");
    let target = append(&mut batch, &doc, &inner, "button");
    let commit = apply(&mut dom, batch);
    let doc = commit.document(&doc).unwrap();
    let [label, target] = [label, target].map(|ticket| commit.created(&ticket).unwrap().unwrap());
    let before = dom.snapshot();

    // Two pending sibling slices. Growth retains the first frame's allocation
    // while admitting the doubled replacement allocation.
    let frame = size_of::<&[NodeId]>();
    let expected_requests = [(0, frame), (frame, 2 * frame)];
    let scratch = 3 * frame;
    let original = Admission::default();
    assert_eq!(
        original.find(&before, label, (usize::MAX, scratch)),
        Ok(Some(target))
    );
    assert_eq!(*original.requested.borrow(), expected_requests);
    let reached = original.work.get();
    assert!(reached > 0);

    let denied = Admission::default();
    assert_eq!(
        denied.find(&before, label, (usize::MAX, scratch - 1)),
        Err(ReadError::Scratch)
    );
    assert_eq!(*denied.requested.borrow(), expected_requests);

    let mut batch = DomBatch::new();
    let suffix = batch.create_element_ns(doc, Namespace::html(), "div");
    batch.insert_before(&suffix, label, None);
    let late = batch.create_element_ns(doc, Namespace::html(), "input");
    batch.insert_before(&late, &suffix, None);
    apply(&mut dom, batch);

    let after = dom.snapshot();
    let exact = Admission::default();
    assert_eq!(
        exact.find(&after, label, (reached, scratch)),
        Ok(Some(target))
    );
    assert_eq!(exact.work.get(), reached);
    assert_eq!(*exact.requested.borrow(), expected_requests);

    let one_short = Admission::default();
    assert_eq!(
        one_short.find(&after, label, (reached - 1, scratch)),
        Err(ReadError::Work)
    );
    assert_eq!(one_short.work.get(), reached - 1);
}
