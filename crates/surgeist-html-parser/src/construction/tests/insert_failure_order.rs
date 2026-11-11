use super::*;
use surgeist_html::values::Namespace;

/// Limits that no insert below reaches, except the one a test sets to zero.
fn limits_with(resource: HtmlParseResource) -> HtmlParseLimits {
    let builder = limits()
        .into_builder()
        .auxiliary_entries(10_000_000)
        .active_formatting_entries(10_000_000)
        .managed_nodes(10_000_000)
        .managed_capacity_bytes(4_000_000_000)
        .peak_transient_bytes(8_000_000_000)
        .atomic_layout_bytes(4_000_000_000)
        .work_credits(1_000_000_000_000);
    match resource {
        HtmlParseResource::TreeDepth => builder.tree_depth(0).managed_edges(10_000_000),
        _ => builder.tree_depth(100_000).managed_edges(0),
    }
    .finish()
    .unwrap()
}

/// Inserts a fresh element under the fragment root and returns the insert's result and the work
/// credits it charged by category: the bytes copied and the record writes.
fn insert_under(resource: HtmlParseResource) -> (Result<(), SessionError>, [u64; 2]) {
    let mut c = ParseControl::new(limits_with(resource), HtmlParseCancellation::new());
    let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
    let root = session.root(&mut c).unwrap();
    let child = session
        .allocate_element(&mut c, element(Namespace::none(), "x"))
        .unwrap();
    let categories = [
        HtmlParseWorkCategory::CopyByte,
        HtmlParseWorkCategory::RecordWrite,
    ];
    let before = categories.map(|category| c.usage().work(category));
    let result = session.insert(&mut c, &root, &child, None, None);
    let after = categories.map(|category| c.usage().work(category));
    session.release(&mut c).unwrap();
    (result, std::array::from_fn(|i| after[i] - before[i]))
}

#[test]
fn an_insert_over_the_depth_or_edge_limit_stops_on_that_limit_before_charging_its_writes() {
    for resource in [
        HtmlParseResource::TreeDepth,
        HtmlParseResource::ManagedEdges,
    ] {
        let (result, [copied, written]) = insert_under(resource);
        assert!(
            matches!(
                result,
                Err(SessionError::Stop(HtmlParseStop::LimitExceeded { resource: actual, .. })) if actual == resource
            ),
            "{resource:?}: {result:?}"
        );
        // The limit is checked before the bytes copied and the entries written by the insert are
        // charged, so an insert that stops on it has charged none of them: the one record write
        // is the parser's own, made before the insert.
        assert_eq!(copied, 0, "{resource:?}: bytes copied before the limit");
        assert!(
            written <= 1,
            "{resource:?}: {written} records written before the limit"
        );
    }
}
