//! Source-created HTML table structure with literal topology expectations.

use super::{Dom, DomBatch, Namespace, NodeId, apply, element, node, owner};
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DomError,
    source::{DomSourceEdit, DomSourceLimits, DomSourceOutput, DomSourceProgress},
};

fn policy(identities: u64) -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 64, 4096, identities, 4)
}

fn run(dom: &mut Dom, edit: DomSourceEdit, identities: u64) -> Result<Option<NodeId>, DomError> {
    let cancel = AtomicBool::new(false);
    let mut lease = dom.begin_source_edit(edit, policy(identities), &cancel)?;
    match lease.advance(&cancel)? {
        DomSourceProgress::Complete(done) => match done.output() {
            Some(DomSourceOutput::Node(node)) => Ok(Some(node)),
            None => Ok(None),
            other => panic!("unexpected Table Source output: {other:?}"),
        },
        other => panic!("unexpected Table Source progress: {other:?}"),
    }
}

#[test]
fn create_tbody_has_fresh_identity_after_the_last_direct_html_body() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let table = b.create_element_ns(&d, Namespace::html(), "table");
    let earlier = element(&mut b, &d, &table, "tbody", &[]);
    let outer = element(&mut b, &d, &table, "div", &[]);
    element(&mut b, &d, &outer, "tbody", &[]);
    let last = element(&mut b, &d, &table, "h:tbody", &[]);
    let text = b.create_text(&d, "raw next sibling");
    b.insert_before(&text, &table, None);
    let foreign = b.create_element_ns(&d, Namespace::svg(), "tbody");
    b.insert_before(&foreign, &table, None);
    let committed = apply(&mut dom, b);
    let [table, earlier, outer, last, text, foreign] =
        [table, earlier, outer, last, text, foreign].map(|ticket| node(&committed, &ticket));
    let first = run(&mut dom, DomSourceEdit::create_html_table_body(table), 16)
        .unwrap()
        .unwrap();
    let second = run(&mut dom, DomSourceEdit::create_html_table_body(table), 16)
        .unwrap()
        .unwrap();
    assert_ne!(first, second);
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot.node(table).unwrap().children(),
        [earlier, outer, last, first, second, text, foreign]
    );
    for part in [first, second] {
        let part = snapshot.node(part).unwrap();
        assert_eq!(part.parent(), Some(table));
        assert_eq!(
            part.owner_document(),
            snapshot.node(table).unwrap().owner_document()
        );
        assert_eq!(
            part.as_element().unwrap().name().local_name(),
            &"tbody".into()
        );
    }
}

#[test]
fn empty_row_creation_admits_both_identities_and_retries_without_prefix() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let table = b.create_element_ns(&d, Namespace::html(), "table");
    let comment = b.create_comment(&d, "keep");
    b.insert_before(&comment, &table, None);
    let committed = apply(&mut dom, b);
    let [table, comment] = [table, comment].map(|ticket| node(&committed, &ticket));
    let stamp = dom.snapshot().stamp();
    let usage = dom.usage();
    assert!(matches!(
        run(&mut dom, DomSourceEdit::insert_html_table_row(table, -1), 1),
        Err(DomError::SourceIdentityLimit)
    ));
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.usage(), usage);
    assert_eq!(dom.snapshot().node(table).unwrap().children(), [comment]);
    let row = run(&mut dom, DomSourceEdit::insert_html_table_row(table, -1), 2)
        .unwrap()
        .unwrap();
    let snapshot = dom.snapshot();
    let implicit = snapshot.node(row).unwrap().parent().unwrap();
    assert_eq!(
        snapshot.node(table).unwrap().children(),
        [comment, implicit]
    );
    assert_eq!(snapshot.node(implicit).unwrap().children(), [row]);
    assert_eq!(
        snapshot
            .node(implicit)
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .local_name(),
        &"tbody".into()
    );
    assert_eq!(
        snapshot
            .node(row)
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .local_name(),
        &"tr".into()
    );
}

#[test]
fn section_and_row_edits_count_only_direct_matching_children() {
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let section = b.create_element_ns(&d, Namespace::html(), "thead");
    let lead = element(&mut b, &d, &section, "a", &[]);
    element(&mut b, &d, &lead, "tr", &[]);
    let foreign_row = b.create_element_ns(&d, Namespace::svg(), "tr");
    b.insert_before(&foreign_row, &section, None);
    let original = element(&mut b, &d, &section, "tr", &[]);
    let header = element(&mut b, &d, &original, "th", &[]);
    let spacer = b.create_comment(&d, "between cells");
    b.insert_before(&spacer, &original, None);
    let foreign_cell = b.create_element_ns(&d, Namespace::svg(), "td");
    b.insert_before(&foreign_cell, &original, None);
    let committed = apply(&mut dom, b);
    let [
        section,
        lead,
        foreign_row,
        original,
        header,
        spacer,
        foreign_cell,
    ] = [
        section,
        lead,
        foreign_row,
        original,
        header,
        spacer,
        foreign_cell,
    ]
    .map(|ticket| node(&committed, &ticket));

    let new = run(
        &mut dom,
        DomSourceEdit::insert_html_table_row(section, 0),
        16,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        dom.snapshot().node(section).unwrap().children(),
        [lead, foreign_row, new, original]
    );
    let cell = run(
        &mut dom,
        DomSourceEdit::insert_html_table_cell(original, 0),
        16,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        dom.snapshot().node(original).unwrap().children(),
        [cell, header, spacer, foreign_cell]
    );
    assert_eq!(
        run(
            &mut dom,
            DomSourceEdit::delete_html_table_cell(original, -1),
            16
        )
        .unwrap(),
        None
    );
    assert_eq!(dom.snapshot().node(header).unwrap().parent(), None);
    assert_eq!(
        dom.snapshot().node(original).unwrap().children(),
        [cell, spacer, foreign_cell]
    );
    let before = dom.snapshot().stamp();
    assert!(matches!(
        run(
            &mut dom,
            DomSourceEdit::delete_html_table_cell(original, 1),
            16
        ),
        Err(DomError::IndexSize)
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        run(
            &mut dom,
            DomSourceEdit::delete_html_table_row(section, 0),
            16
        )
        .unwrap(),
        None
    );
    assert_eq!(dom.snapshot().node(new).unwrap().parent(), None);
    assert_eq!(
        dom.snapshot().node(section).unwrap().children(),
        [lead, foreign_row, original]
    );
}
