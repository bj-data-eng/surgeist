//! HTML877d §4.9.1 over the canonical DOM Source and ordinary child sequence.
use super::{Dom, DomBatch, DomSnapshot, Namespace, NodeId, apply, element, node, owner};
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DomError,
    edit::DomChildListObservationLimits,
    source::{
        DomSourceCompletion, DomSourceEdit, DomSourceLimits, DomSourceOutput, DomSourceProgress,
    },
    tree::DomHtmlTablePart::{self, Caption, Foot, Head},
};

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 8)
}

fn complete(dom: &mut Dom, edit: DomSourceEdit) -> DomSourceCompletion {
    let cancel = AtomicBool::new(false);
    let progress = dom
        .begin_source_edit(edit, source_limits(), &cancel)
        .unwrap()
        .advance(&cancel)
        .unwrap();
    match progress {
        DomSourceProgress::Complete(done) => done,
        other => panic!("expected an ordinary completed Table part edit, got {other:?}"),
    }
}

fn create(dom: &mut Dom, table: NodeId, part: DomHtmlTablePart) -> NodeId {
    let done = complete(dom, DomSourceEdit::create_html_table_part(table, part));
    let Some(DomSourceOutput::Node(id)) = done.output() else {
        panic!(
            "expected one native Table part identity, got {:?}",
            done.output()
        );
    };
    id
}

fn part(snapshot: &DomSnapshot, table: NodeId, part: DomHtmlTablePart) -> Option<NodeId> {
    snapshot
        .node(table)
        .unwrap()
        .html_table_part_with(part, || Ok::<_, DomError>(()))
        .unwrap()
}

#[derive(Debug, PartialEq)]
enum ReadError {
    Dom(DomError),
    Cap,
}

impl From<DomError> for ReadError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}

fn read_head_with_cap(
    snapshot: &DomSnapshot,
    table: NodeId,
    cap: usize,
) -> (Result<Option<NodeId>, ReadError>, usize) {
    let mut used = 0;
    let result = snapshot
        .node(table)
        .unwrap()
        .html_table_part_with(Head, || {
            if used == cap {
                return Err(ReadError::Cap);
            }
            used += 1;
            Ok(())
        });
    (result, used)
}

#[test]
fn detached_table_reads_first_exact_direct_child_and_stops_at_that_prefix() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let doc = b.create_document();
    let table = b.create_element_ns(&doc, Namespace::html(), "table");
    let foreign_table = b.create_element_ns(&doc, Namespace::svg(), "table");
    let foreign_head = b.create_element_ns(&doc, Namespace::svg(), "thead");
    b.insert_before(&foreign_head, &table, None);
    element(&mut b, &doc, &table, "THEAD", &[]);
    let wrapper = element(&mut b, &doc, &table, "div", &[]);
    element(&mut b, &doc, &wrapper, "thead", &[]);
    let comment = b.create_comment(&doc, "ordinary child");
    b.insert_before(&comment, &table, None);
    let first = element(&mut b, &doc, &table, "h:thead", &[]);
    let second = b.create_element_ns(&doc, Namespace::html(), "thead");
    let third = b.create_element_ns(&doc, Namespace::html(), "thead");
    let caption = element(&mut b, &doc, &table, "caption", &[]);
    let commit = apply(&mut dom, b);
    let [table, foreign_table, first, second, third, caption] =
        [table, foreign_table, first, second, third, caption].map(|id| node(&commit, &id));

    let original = dom.snapshot();
    assert!(!original.node(table).unwrap().is_connected());
    assert_eq!(part(&original, table, Caption), Some(caption));
    assert_eq!(part(&original, table, Foot), None);
    assert_eq!(
        original
            .node(foreign_table)
            .unwrap()
            .html_table_part_with(Head, || Ok::<_, DomError>(())),
        Err(DomError::WrongNodeKind)
    );
    let (answer, prefix) = read_head_with_cap(&original, table, usize::MAX);
    assert_eq!(answer, Ok(Some(first)));
    assert!(prefix > 0);

    let mut append = DomBatch::new();
    append.insert_before(second, table, None);
    append.insert_before(third, table, None);
    apply(&mut dom, append);
    let expanded = dom.snapshot();
    // An unchanged successful prefix has the same cost with additional matching
    // siblings after it. One short propagates the caller error unchanged.
    assert_eq!(
        read_head_with_cap(&expanded, table, prefix),
        (Ok(Some(first)), prefix)
    );
    assert_eq!(
        read_head_with_cap(&expanded, table, prefix - 1),
        (Err(ReadError::Cap), prefix - 1)
    );
    assert_eq!(create(&mut dom, table, Head), first);
    assert_eq!(
        dom.snapshot().node(table).unwrap().children(),
        expanded.node(table).unwrap().children()
    );
    complete(&mut dom, DomSourceEdit::delete_html_table_part(table, Head));
    let deleted = dom.snapshot();
    assert_eq!(part(&deleted, table, Head), Some(second));
    assert_eq!(deleted.node(first).unwrap().parent(), None);
    assert_eq!(deleted.node(third).unwrap().parent(), Some(table));
    assert_eq!(part(&original, table, Head), Some(first));
}

#[test]
fn creates_use_pinned_node_and_element_positions_and_return_existing_identity() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let doc = b.create_document();
    let table = b.create_element_ns(&doc, Namespace::html(), "table");
    let lead = b.create_comment(&doc, "leading");
    b.insert_before(&lead, &table, None);
    let group = element(&mut b, &doc, &table, "colgroup", &[]);
    let middle = b.create_comment(&doc, "before first eligible element");
    b.insert_before(&middle, &table, None);
    let foreign_group = b.create_element_ns(&doc, Namespace::svg(), "colgroup");
    b.insert_before(&foreign_group, &table, None);
    let body = element(&mut b, &doc, &table, "tbody", &[]);
    let trail = b.create_text(&doc, "trailing");
    b.insert_before(&trail, &table, None);
    let no_reference = b.create_element_ns(&doc, Namespace::html(), "table");
    let last_comment = b.create_comment(&doc, "do not insert before a comment");
    b.insert_before(&last_comment, &no_reference, None);
    let commit = apply(&mut dom, b);
    let document = commit.document(&doc).unwrap();
    let [
        table,
        lead,
        group,
        middle,
        foreign_group,
        body,
        trail,
        no_reference,
        last_comment,
    ] = [
        table,
        lead,
        group,
        middle,
        foreign_group,
        body,
        trail,
        no_reference,
        last_comment,
    ]
    .map(|id| node(&commit, &id));

    let caption = create(&mut dom, table, Caption);
    let head = create(&mut dom, table, Head);
    let foot = create(&mut dom, table, Foot);
    let appended_head = create(&mut dom, no_reference, Head);
    for (kind, id) in [(Caption, caption), (Head, head), (Foot, foot)] {
        assert_eq!(create(&mut dom, table, kind), id);
    }

    let fresh = dom.snapshot();
    assert_eq!(
        fresh.node(table).unwrap().children(),
        [
            caption,
            lead,
            group,
            middle,
            head,
            foreign_group,
            body,
            trail,
            foot
        ]
    );
    assert_eq!(
        fresh.node(no_reference).unwrap().children(),
        [last_comment, appended_head]
    );
    for (id, local) in [(caption, "caption"), (head, "thead"), (foot, "tfoot")] {
        let node = fresh.node(id).unwrap();
        let name = node.as_element().unwrap().name();
        assert_eq!(node.owner_document(), Some(document));
        assert_eq!(name.namespace_uri(), Namespace::html().uri());
        assert_eq!(name.local_name(), &local.into());
        assert_eq!(name.prefix(), None);
    }
}

#[test]
fn setters_move_the_original_part_and_adopt_the_existing_cross_document_subtree() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let first_doc = b.create_document();
    let second_doc = b.create_document();
    let table = b.create_element_ns(&first_doc, Namespace::html(), "table");
    let comment = b.create_comment(&first_doc, "before existing Caption");
    b.insert_before(&comment, &table, None);
    let caption = element(&mut b, &first_doc, &table, "caption", &[]);
    let head = element(&mut b, &first_doc, &table, "thead", &[]);
    let foot = element(&mut b, &first_doc, &table, "tfoot", &[]);
    let body = element(&mut b, &first_doc, &table, "tbody", &[]);
    let remote = b.create_element_ns(&second_doc, Namespace::html(), "table");
    let remote_head = element(&mut b, &second_doc, &remote, "thead", &[]);
    let remote_row = element(&mut b, &second_doc, &remote_head, "tr", &[]);
    let commit = apply(&mut dom, b);
    let origin = commit.document(&first_doc).unwrap();
    let remote_doc = commit.document(&second_doc).unwrap();
    let [
        table,
        comment,
        caption,
        head,
        foot,
        body,
        remote,
        remote_head,
        remote_row,
    ] = [
        table,
        comment,
        caption,
        head,
        foot,
        body,
        remote,
        remote_head,
        remote_row,
    ]
    .map(|id| node(&commit, &id));

    let before = dom.snapshot();
    assert_eq!(
        before.node(remote_row).unwrap().owner_document(),
        Some(remote_doc)
    );
    complete(
        &mut dom,
        DomSourceEdit::set_html_table_part(table, Caption, Some(caption)),
    );
    complete(
        &mut dom,
        DomSourceEdit::set_html_table_part(table, Foot, Some(foot)),
    );
    complete(
        &mut dom,
        DomSourceEdit::set_html_table_part(table, Head, Some(remote_head)),
    );
    let after = dom.snapshot();
    assert_eq!(
        after.node(table).unwrap().children(),
        [caption, comment, remote_head, body, foot]
    );
    assert_eq!(after.node(head).unwrap().parent(), None);
    assert!(after.node(remote).unwrap().children().is_empty());
    assert_eq!(after.node(remote_head).unwrap().children(), [remote_row]);
    for id in [remote_head, remote_row] {
        assert_eq!(after.node(id).unwrap().owner_document(), Some(origin));
    }

    complete(
        &mut dom,
        DomSourceEdit::set_html_table_part(table, Caption, None),
    );
    complete(
        &mut dom,
        DomSourceEdit::delete_html_table_part(table, Caption),
    );
    assert_eq!(
        dom.snapshot().node(table).unwrap().children(),
        [comment, remote_head, body, foot]
    );
}

#[test]
fn wrong_section_precedes_removal_but_ancestor_insertion_publishes_the_old_removal() {
    let mut dom = owner();
    let observer_limits = DomChildListObservationLimits::new(1, 1, 4, 24).unwrap();
    dom.set_limits(dom.limits().with_child_list_observation(observer_limits))
        .unwrap();
    let mut b = DomBatch::new();
    let doc = b.create_document();
    let ancestor = b.create_element_ns(&doc, Namespace::html(), "caption");
    let table = element(&mut b, &doc, &ancestor, "table", &[]);
    let caption = element(&mut b, &doc, &table, "caption", &[]);
    let head = element(&mut b, &doc, &table, "thead", &[]);
    let foot = element(&mut b, &doc, &table, "tfoot", &[]);
    let body = element(&mut b, &doc, &table, "tbody", &[]);
    let commit = apply(&mut dom, b);
    let [ancestor, table, caption, head, foot, body] =
        [ancestor, table, caption, head, foot, body].map(|id| node(&commit, &id));

    let cancel = AtomicBool::new(false);
    let observer = dom.create_child_list_observer(&cancel).unwrap();
    dom.observe_child_list(observer, table, false, &cancel)
        .unwrap();
    for (kind, invalid) in [(Head, foot), (Foot, body)] {
        let before = dom.snapshot();
        {
            let mut lease = dom
                .begin_source_edit(
                    DomSourceEdit::set_html_table_part(table, kind, Some(invalid)),
                    source_limits(),
                    &cancel,
                )
                .unwrap();
            let error = match lease.advance(&cancel) {
                Err(error) => error,
                Ok(DomSourceProgress::Failed(failure)) => failure.error().clone(),
                other => panic!("expected a wrong-section hierarchy error, got {other:?}"),
            };
            assert_eq!(error, DomError::HierarchyRequest);
            assert_eq!(
                lease.snapshot().node(table).unwrap().children(),
                before.node(table).unwrap().children()
            );
        }
        assert!(dom.pending_source_edit().is_none());
    }
    assert!(
        dom.take_child_list_records(observer, &cancel)
            .unwrap()
            .is_empty()
    );

    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_html_table_part(table, Caption, Some(ancestor)),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let original = lease.operation();
        assert!(matches!(
            lease.advance(&AtomicBool::new(true)),
            Err(DomError::Cancelled)
        ));
        assert_eq!(
            lease.snapshot().node(table).unwrap().children(),
            [caption, head, foot, body]
        );
        let DomSourceProgress::Failed(failure) = lease.advance(&cancel).unwrap() else {
            panic!(
                "the original Source must report the hierarchy error with its published removal"
            );
        };
        assert_eq!(failure.operation(), original);
        assert_eq!(failure.error(), &DomError::HierarchyRequest);
    }
    assert!(dom.pending_source_edit().is_none());
    let after = dom.snapshot();
    assert_eq!(after.node(ancestor).unwrap().children(), [table]);
    assert_eq!(after.node(table).unwrap().children(), [head, foot, body]);
    assert_eq!(after.node(caption).unwrap().parent(), None);
    let records = dom.take_child_list_records(observer, &cancel).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].target(), table);
    assert_eq!(records[0].removed_nodes(), [caption]);
    assert!(records[0].added_nodes().is_empty());
}
