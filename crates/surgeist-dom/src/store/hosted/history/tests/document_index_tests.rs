//! Fixed selected Document membership against unrelated genuine issuer rows.
use super::*;

fn document_fixture(
    unrelated: usize,
) -> (
    Fixture,
    Arc<Graph>,
    DomHostedCreation,
    DomHostedCreation,
    Option<DomHostedCreation>,
    Arc<Work>,
) {
    let cancel = AtomicBool::new(false);
    let mut fixture = Fixture::new();
    let mut graph = None;
    let mut unrelated_last = None;
    for _ in 0..unrelated {
        let (creation, _, work) = fixture.issue(2);
        graph = Some(
            Graph::initial(
                graph.as_deref(),
                fixture.input(creation, None),
                &Context::new(&fixture.source, &work, &cancel, 0),
            )
            .unwrap(),
        );
        unrelated_last = Some(creation);
    }
    let (first, _, work) = fixture.issue(0);
    graph = Some(
        Graph::initial(
            graph.as_deref(),
            fixture.input(first, None),
            &Context::new(&fixture.source, &work, &cancel, 0),
        )
        .unwrap(),
    );
    drop(work);
    let (second, _, work) = fixture.issue(0);
    let graph = Graph::initial(
        graph.as_deref(),
        fixture.input(second, None),
        &Context::new(&fixture.source, &work, &cancel, 0),
    )
    .unwrap();
    assert_eq!(first.document, second.document);
    (fixture, graph, first, second, unrelated_last, work)
}

#[test]
fn document_outer_discovery_window_selection_is_bounded_and_keeps_first_ordinal() {
    fn measure(unrelated: usize) -> u64 {
        let (fixture, graph, first, second, other, work) = document_fixture(unrelated);
        let before = work.spent.load(Ordering::Relaxed);
        let selected = graph
            .active_navigable_for_document_with(first.document, || {
                work.step(&AtomicBool::new(false))
            })
            .unwrap();
        let spent = work.spent.load(Ordering::Relaxed) - before;
        assert_eq!(selected, Some(first.navigable));
        assert_ne!(first.navigable, second.navigable);
        assert_eq!(
            graph.entry(first.entry).unwrap().document,
            Some(first.document)
        );
        assert_eq!(
            graph.entry(second.entry).unwrap().document,
            Some(first.document)
        );
        if let Some(other) = other {
            assert_ne!(other.document, first.document);
            assert_eq!(
                graph.entry(other.entry).unwrap().document,
                Some(other.document)
            );
        }
        assert_eq!(fixture.documents[0], first.document);
        spent
    }
    let small = measure(0);
    let large = measure(48);
    assert!(
        large <= small + 64,
        "selected Document lookup spent {large} steps with 48 unrelated rows, versus {small} without them"
    );
}

#[test]
fn document_outer_discovery_clear_is_bounded_and_retires_only_matching_rows() {
    fn measure(unrelated: usize) -> u64 {
        let (fixture, graph, first, second, other, work) = document_fixture(unrelated);
        let captured = graph.clone();
        let before = work.spent.load(Ordering::Relaxed);
        let changed = graph
            .clear_document(
                first.document,
                &Context::new(&fixture.source, &work, &AtomicBool::new(false), 0),
            )
            .unwrap();
        let spent = work.spent.load(Ordering::Relaxed) - before;
        for creation in [first, second] {
            assert_eq!(changed.entry(creation.entry).unwrap().document, None);
            assert!(changed.navigable(creation.navigable).unwrap().retired);
            assert!(changed.navigable(creation.navigable).unwrap().retiring);
            assert_eq!(
                captured.entry(creation.entry).unwrap().document,
                Some(first.document)
            );
            assert!(!captured.navigable(creation.navigable).unwrap().retired);
            assert!(!captured.navigable(creation.navigable).unwrap().retiring);
        }
        if let Some(other) = other {
            assert_eq!(
                changed.entry(other.entry).unwrap().document,
                Some(other.document)
            );
            assert!(!changed.navigable(other.navigable).unwrap().retired);
            assert!(!changed.navigable(other.navigable).unwrap().retiring);
        }
        assert_eq!(
            changed
                .active_navigable_for_document_with(first.document, || Ok(()))
                .unwrap(),
            None
        );
        assert_eq!(
            captured
                .active_navigable_for_document_with(first.document, || Ok(()))
                .unwrap(),
            Some(first.navigable)
        );
        spent
    }
    let small = measure(0);
    let large = measure(48);
    assert!(
        large <= small + 128,
        "selected Document cleanup spent {large} steps with 48 unrelated rows, versus {small} without them"
    );
}
