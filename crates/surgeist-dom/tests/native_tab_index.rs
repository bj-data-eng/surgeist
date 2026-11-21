//! Bounded static tabindex reads through a selected canonical node view.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    Dom, DomBatch, DomError, DomSnapshot,
    identity::NodeId,
    limits::{DomLimits, DomNameLimits},
};
use surgeist_html::values::{DocumentString, Namespace};

fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            256,
            8192,
            128,
            0,
            0,
            32,
            1_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn read(snapshot: &DomSnapshot, node: NodeId, steps: usize) -> Result<(i32, usize), DomError> {
    let mut remaining = steps;
    let answer = snapshot.node(node)?.tab_index_with(|| {
        remaining = remaining.checked_sub(1).ok_or(DomError::Cancelled)?;
        Ok::<_, DomError>(())
    })?;
    Ok((answer, steps - remaining))
}

fn lookup_work(snapshot: &DomSnapshot, nodes: impl IntoIterator<Item = NodeId>) -> usize {
    let mut work = 0;
    for node in nodes {
        snapshot
            .node_with(node, || {
                work += 1;
                Ok::<_, DomError>(())
            })
            .unwrap();
    }
    work
}

#[test]
fn summary_default_follows_only_the_reached_ordinary_prefix() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let details = batch.create_element_ns(&document, Namespace::html(), "details");
    let before = batch.create_text(&document, "ordinary text");
    let other_namespace = batch.create_element_ns(&document, Namespace::svg(), "summary");
    let first = batch.create_element_ns(&document, Namespace::html(), "summary");
    let after = batch.create_element_ns(&document, Namespace::html(), "div");
    let second = batch.create_element_ns(&document, Namespace::html(), "summary");
    for child in [&before, &other_namespace, &first, &after, &second] {
        batch.insert_before(child, &details, None);
    }
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let details = commit.created(&details).unwrap().unwrap();
    let text = commit.created(&before).unwrap().unwrap();
    let foreign = commit.created(&other_namespace).unwrap().unwrap();
    let first = commit.created(&first).unwrap().unwrap();
    let after = commit.created(&after).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let original = dom.snapshot();
    let (value, exact) = read(&original, second, 10_000).unwrap();
    assert_eq!(value, -1);
    assert_eq!(read(&original, first, exact), Ok((0, exact)));
    assert_eq!(read(&original, first, exact - 1), Err(DomError::Cancelled));
    let prefix = |snapshot: &DomSnapshot, selected| {
        lookup_work(snapshot, [details, text, foreign, selected])
    };
    let comparisons_and_edges = exact - prefix(&original, first);

    let mut batch = DomBatch::new();
    for _ in 0..40 {
        let tail = batch.create_element_ns(document, Namespace::html(), "summary");
        batch.insert_before(&tail, after, None);
    }
    dom.apply(batch, &cancel).unwrap();
    let large_tail = dom.snapshot();
    // The unrelated tail may change the bank depth for the same reached IDs.
    // Only the cost of the original four canonical lookups may vary here.
    let large_exact = comparisons_and_edges + prefix(&large_tail, first);
    assert_eq!(read(&large_tail, first, large_exact), Ok((0, large_exact)));
    assert_eq!(
        read(&large_tail, second, large_exact),
        Ok((-1, large_exact))
    );

    let mut batch = DomBatch::new();
    batch.insert_before(second, details, Some(first.into()));
    dom.apply(batch, &cancel).unwrap();
    let current = dom.snapshot();
    let current_exact = comparisons_and_edges + prefix(&current, second);
    assert_eq!(
        read(&current, first, current_exact),
        Ok((-1, current_exact))
    );
    assert_eq!(
        read(&current, second, current_exact),
        Ok((0, current_exact))
    );
    assert_eq!(read(&original, first, exact), Ok((0, exact)));
}

#[test]
fn recovered_integer_and_foreign_namespace_are_distinct_from_static_defaults() {
    let mut dom = dom();
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let div = batch.create_element_ns(&document, Namespace::html(), "div");
    let button = batch.create_element_ns(&document, Namespace::html(), "button");
    let svg = batch.create_element_ns(&document, Namespace::svg(), "p:a");
    let other = batch.create_element_ns(&document, Namespace::none(), "a");
    batch.set_attribute(
        &div,
        "tabindex",
        DocumentString::from_utf16(vec![u16::from(b'+'), u16::from(b'7'), 0xd800]),
    );
    batch.set_attribute(&button, "tabindex", "2147483648");
    batch.set_attribute_ns(&svg, Namespace::html(), "p:tabindex", "8");
    let commit = dom.apply(batch, &cancel).unwrap();
    let snapshot = dom.snapshot();
    let div_id = commit.created(&div).unwrap().unwrap();
    let attr = snapshot
        .node(div_id)
        .unwrap()
        .attributes()
        .unwrap()
        .item(0)
        .unwrap();
    let lookup = lookup_work(&snapshot, [attr.id().node()]);
    // One kind read, the HTML namespace header and its 28 code units,
    // the original Attr bank lookup, eight local-name code units, and
    // the HTML integer parser's atomic envelope for three raw units.
    let exact = 1 + (1 + 28) + lookup + 8 + (8 * 3 + 16);
    assert_eq!(read(&snapshot, div_id, exact), Ok((7, exact)));
    assert_eq!(read(&snapshot, div_id, exact - 1), Err(DomError::Cancelled));
    for (ticket, expected) in [(&button, 0), (&svg, 0)] {
        let node = commit.created(ticket).unwrap().unwrap();
        assert_eq!(read(&snapshot, node, 10_000).unwrap().0, expected);
    }
    let node = commit.created(&other).unwrap().unwrap();
    assert_eq!(read(&snapshot, node, 10_000), Err(DomError::WrongNodeKind));
}
