use super::*;
use surgeist_html::values::Namespace;

/// Limits that no document of the sizes below reaches.
fn roomy_limits() -> HtmlParseLimits {
    limits()
        .into_builder()
        .auxiliary_entries(10_000_000)
        .active_formatting_entries(10_000_000)
        .managed_nodes(10_000_000)
        .managed_edges(10_000_000)
        .tree_depth(100_000)
        .managed_capacity_bytes(4_000_000_000)
        .peak_transient_bytes(8_000_000_000)
        .atomic_layout_bytes(4_000_000_000)
        .work_credits(1_000_000_000_000)
        .finish()
        .unwrap()
}

/// The work of one insert, by category: every category that does not depend on whether the
/// insert had to grow a bank.
const MEASURED: [HtmlParseWorkCategory; 5] = [
    HtmlParseWorkCategory::InputUnitRead,
    HtmlParseWorkCategory::InspectByte,
    HtmlParseWorkCategory::CopyByte,
    HtmlParseWorkCategory::RecordVisit,
    HtmlParseWorkCategory::RecordWrite,
];

/// Inserts a fresh element as the last child of `parent` and returns the work the insert
/// charged, which must not include the growth of a bank.
fn measured_append(
    c: &mut ParseControl,
    session: &mut ParserArenaSession,
    parent: &ParserNode,
) -> [u64; 5] {
    let fresh = session
        .allocate_element(c, element(Namespace::none(), "fresh"))
        .unwrap();
    let reserved = c.usage().work(HtmlParseWorkCategory::ReserveByte);
    let before = MEASURED.map(|category| c.usage().work(category));
    session.insert(c, parent, &fresh, None, None).unwrap();
    let after = MEASURED.map(|category| c.usage().work(category));
    assert_eq!(
        c.usage().work(HtmlParseWorkCategory::ReserveByte),
        reserved,
        "the measured insert grew a bank"
    );
    std::array::from_fn(|i| after[i] - before[i])
}

#[test]
fn appending_one_node_charges_the_same_work_at_every_fanout() {
    // 1,000 to 8,000 children each leave spare room in a child order that doubles.
    let measure = |fanout: usize| {
        let mut c = ParseControl::new(roomy_limits(), HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let root = session.root(&mut c).unwrap();
        for _ in 0..fanout {
            let child = session
                .allocate_element(&mut c, element(Namespace::none(), "x"))
                .unwrap();
            session.insert(&mut c, &root, &child, None, None).unwrap();
        }
        let work = measured_append(&mut c, &mut session, &root);
        session.release(&mut c).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        work
    };
    let base = measure(1_000);
    for fanout in [2_000, 4_000, 8_000] {
        assert_eq!(
            measure(fanout),
            base,
            "an append under {fanout} children charged different work than under 1000"
        );
    }
}

#[test]
fn appending_one_node_charges_the_same_work_at_every_depth() {
    let measure = |depth: usize| {
        let mut c = ParseControl::new(roomy_limits(), HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let mut parent = session.root(&mut c).unwrap();
        for _ in 0..depth {
            let child = session
                .allocate_element(&mut c, element(Namespace::none(), "x"))
                .unwrap();
            session.insert(&mut c, &parent, &child, None, None).unwrap();
            parent = child;
        }
        // Three siblings leave room for a fourth in a child order that doubles.
        for _ in 0..3 {
            let child = session
                .allocate_element(&mut c, element(Namespace::none(), "y"))
                .unwrap();
            session.insert(&mut c, &parent, &child, None, None).unwrap();
        }
        let work = measured_append(&mut c, &mut session, &parent);
        session.release(&mut c).unwrap();
        assert_eq!(c.usage().actual_capacity_bytes, 0);
        work
    };
    let base = measure(1_000);
    for depth in [2_000, 4_000, 8_000] {
        assert_eq!(
            measure(depth),
            base,
            "an append at depth {depth} charged different work than at depth 1000"
        );
    }
}
