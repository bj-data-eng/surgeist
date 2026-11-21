//! Scalar TableRow reads over ordinary canonical relationships.
use super::{DomBatch, DomSnapshot, Namespace, NodeId, apply, element, node, owner};
use surgeist_dom::DomError;
use surgeist_html_semantics::table_rows::HtmlTableRowIndexKind::{self, Section, Table};

fn indexes(snapshot: &DomSnapshot, id: NodeId) -> (Option<usize>, Option<usize>) {
    let row = snapshot.node(id).unwrap();
    let index = |kind| {
        row.html_table_row_index_with(kind, || Ok::<_, DomError>(()))
            .unwrap()
    };
    (index(Table), index(Section))
}

#[test]
fn native_indexes_follow_ordinary_table_order_and_current_topology() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let doc = b.create_document();
    let root = element(&mut b, &doc, &doc, "div", &[]);
    let table = element(&mut b, &doc, &root, "table", &[]);
    let foot = element(&mut b, &doc, &table, "tfoot", &[]);
    let f = element(&mut b, &doc, &foot, "tr", &[]);
    let d0 = element(&mut b, &doc, &table, "tr", &[]);
    let body = element(&mut b, &doc, &table, "tbody", &[]);
    let b0 = element(&mut b, &doc, &body, "tr", &[]);
    let b1 = element(&mut b, &doc, &body, "tr", &[]);
    let head0 = element(&mut b, &doc, &table, "thead", &[]);
    let h0 = element(&mut b, &doc, &head0, "h:tr", &[]);
    let d1 = element(&mut b, &doc, &table, "tr", &[]);
    let head1 = element(&mut b, &doc, &table, "thead", &[]);
    let h1 = element(&mut b, &doc, &head1, "tr", &[]);

    let comment = b.create_comment(&doc, "ordinary child");
    b.insert_before(&comment, &head0, Some((&h0).into()));
    let foreign_row = b.create_element_ns(&doc, Namespace::svg(), "tr");
    b.insert_before(&foreign_row, &head0, Some((&h0).into()));
    let foreign_section = b.create_element_ns(&doc, Namespace::svg(), "thead");
    b.insert_before(&foreign_section, &table, Some((&head0).into()));
    let foreign_section_row = element(&mut b, &doc, &foreign_section, "tr", &[]);
    let wrapper = element(&mut b, &doc, &table, "div", &[]);
    let wrapped_row = element(&mut b, &doc, &wrapper, "tr", &[]);
    let wrapped_body = element(&mut b, &doc, &wrapper, "tbody", &[]);
    let wrapped_section_row = element(&mut b, &doc, &wrapped_body, "tr", &[]);
    let detached_body = b.create_element_ns(&doc, Namespace::html(), "tbody");
    let detached_row = element(&mut b, &doc, &detached_body, "tr", &[]);
    let initial = apply(&mut dom, b);
    let [
        table,
        body,
        d0,
        d1,
        b0,
        b1,
        h0,
        h1,
        f,
        detached_body,
        detached_row,
    ] = [
        table,
        body,
        d0,
        d1,
        b0,
        b1,
        h0,
        h1,
        f,
        detached_body,
        detached_row,
    ]
    .map(|ticket| node(&initial, &ticket));
    let first = dom.snapshot();
    for (ordinal, (subject, section_index)) in
        [(h0, 0), (h1, 0), (d0, 2), (b0, 0), (b1, 1), (d1, 5), (f, 0)]
            .into_iter()
            .enumerate()
    {
        assert_eq!(
            indexes(&first, subject),
            (Some(ordinal), Some(section_index))
        );
    }
    assert_eq!(indexes(&first, detached_row), (None, Some(0)));
    assert_eq!(
        indexes(&first, node(&initial, &wrapped_section_row)),
        (None, Some(0))
    );
    for ticket in [foreign_section_row, wrapped_row] {
        assert_eq!(indexes(&first, node(&initial, &ticket)), (None, None));
    }

    let foreign_row = first.node(node(&initial, &foreign_row)).unwrap();
    for kind in [Table, Section] {
        assert_eq!(
            foreign_row.html_table_row_index_with(kind, || Ok::<_, DomError>(())),
            Err(DomError::WrongNodeKind)
        );
    }

    let mut reorder = DomBatch::new();
    reorder.insert_before(d1, table, Some(body.into()));
    reorder.insert_before(b1, detached_body, None);
    reorder.detach(table);
    apply(&mut dom, reorder);
    let reordered = dom.snapshot();
    assert_eq!(indexes(&reordered, d1), (Some(3), Some(3)));
    assert_eq!(indexes(&reordered, b0), (Some(4), Some(0)));
    assert_eq!(indexes(&reordered, b1), (None, Some(1)));
    assert_eq!(indexes(&first, d1), (Some(5), Some(5)));
    assert_eq!(indexes(&first, b1), (Some(4), Some(1)));

    let mut retire = DomBatch::new();
    retire.detach(b1);
    apply(&mut dom, retire);
    assert_eq!(indexes(&dom.snapshot(), b1), (None, None));
    let mut retire = DomBatch::new();
    retire.destroy(b1);
    apply(&mut dom, retire);
    assert!(matches!(dom.snapshot().node(b1), Err(DomError::StaleNode)));
    assert_eq!(indexes(&reordered, b1), (None, Some(1)));
}

#[derive(Debug, PartialEq)]
enum ReadError {
    Native(DomError),
    Cap,
}

impl From<DomError> for ReadError {
    fn from(error: DomError) -> Self {
        Self::Native(error)
    }
}

fn lookup_cost(snapshot: &DomSnapshot, id: NodeId) -> usize {
    let mut count = 0;
    snapshot
        .node_with(id, || {
            count += 1;
            Ok::<_, DomError>(())
        })
        .unwrap();
    count
}

#[test]
fn native_index_admits_the_exact_prefix_and_preserves_the_first_refusal() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let doc = b.create_document();
    let table = b.create_element_ns(&doc, Namespace::html(), "table");
    let head = element(&mut b, &doc, &table, "thead", &[]);
    let comment = b.create_comment(&doc, "skip");
    b.insert_before(&comment, &head, None);
    let previous = element(&mut b, &doc, &head, "tr", &[]);
    let target = element(&mut b, &doc, &head, "tr", &[]);
    element(&mut b, &doc, &head, "tr", &[]);
    let late = element(&mut b, &doc, &table, "thead", &[]);
    element(&mut b, &doc, &late, "tr", &[]);
    let commit = apply(&mut dom, b);
    let [table, head, comment, previous, target] =
        [table, head, comment, previous, target].map(|ticket| node(&commit, &ticket));

    let snapshot = dom.snapshot();
    let cost = |id| lookup_cost(&snapshot, id);
    // Subject classification, parent edge and head lookup/classification,
    // then, for Table only, the head's parent edge and table lookup/classification.
    let direct_parent = 1 + 1 + cost(head) + 1;
    let table_phase = (1 + cost(table) + 1) + 1 + (cost(table) + 1) + (1 + cost(head) + 1 + 1);
    // Open head children, consume/classify a comment, then consume/classify
    // two rows, each followed by a row policy and a projection debit.
    let section_prefix = (cost(head) + 1)
        + (1 + cost(comment) + 1)
        + (1 + cost(previous) + 1 + 1 + 1)
        + (1 + cost(target) + 1 + 1 + 1);

    let row = snapshot.node(target).unwrap();
    for (kind, exact) in [
        (Table, direct_parent + table_phase + section_prefix),
        (Section, direct_parent + section_prefix),
    ] {
        let invoke = |kind: HtmlTableRowIndexKind, cap: usize| {
            let mut used = 0;
            let answer = row.html_table_row_index_with(kind, || {
                if used == cap {
                    return Err(ReadError::Cap);
                }
                used += 1;
                Ok(())
            });
            (answer, used)
        };
        assert_eq!(invoke(kind, exact), (Ok(Some(1)), exact));
        assert_eq!(invoke(kind, exact - 1), (Err(ReadError::Cap), exact - 1));
    }
}
