//! Real issuer rows exercise the existing activity predicate and original Work.
//! Five independent top-level rows put the selected top last in both banks.
use super::*;

fn last_retained_top() -> (Fixture, Arc<Graph>, DomHostedCreation, Arc<Work>) {
    let mut fixture = Fixture::new();
    let cancel = AtomicBool::new(false);
    let mut graph = None;
    let mut last = None;
    for document in 0..5 {
        // The preceding issuer operation was completed by Fixture::issue.
        // Release its finished preparation lane before issuing the next top;
        // the historical Graph retains its real rows independently of Work.
        drop(last.take());
        let (creation, _, work) = fixture.issue(document);
        graph = Some(
            Graph::initial(
                graph.as_deref(),
                fixture.input(creation, None),
                &Context::new(&fixture.source, &work, &cancel, 0),
            )
            .unwrap(),
        );
        last = Some((creation, work));
    }
    let (creation, work) = last.unwrap();
    (fixture, graph.unwrap(), creation, work)
}

#[test]
fn last_retained_top_activity_charges_both_complete_bank_scans() {
    let (mut fixture, graph, creation, work) = last_retained_top();
    let before = work.spent.load(Ordering::Relaxed);
    let exact = before + 10;
    fixture.limits(16_000_000, 16_000_000, exact);
    assert!(
        captured_fully_active(&graph, creation.document, creation.navigable, || {
            work.step(&AtomicBool::new(false))
        })
        .unwrap()
    );
    assert_eq!(
        work.spent.load(Ordering::Relaxed),
        exact,
        "selecting the fifth top inspects five original navigables and five entries"
    );
}

#[test]
fn last_retained_top_activity_refuses_one_short_of_actual_bank_scan_work() {
    let (mut fixture, graph, creation, work) = last_retained_top();
    let before = work.spent.load(Ordering::Relaxed);
    let one_short = before + 9;
    fixture.limits(16_000_000, 16_000_000, one_short);
    let original_graph = graph.clone();
    let original_revision = fixture.source.revision().unwrap();
    let original_bytes = fixture.used();
    let original_work = work.clone();
    let result = captured_fully_active(&graph, creation.document, creation.navigable, || {
        work.step(&AtomicBool::new(false))
    });
    assert!(
        matches!(result, Err(DomHostedError::WorkLimit { limit }) if limit == one_short),
        "a fifth-row activity query must not succeed with only nine scan admissions: {result:?}"
    );
    assert_eq!(work.spent.load(Ordering::Relaxed), one_short);
    assert_eq!(*graph, *original_graph);
    assert_eq!(fixture.source.revision().unwrap(), original_revision);
    assert_eq!(fixture.used(), original_bytes);
    assert_eq!(
        graph.entry(creation.entry).unwrap().document,
        Some(creation.document)
    );
    assert!(!graph.navigable(creation.navigable).unwrap().retired);

    // Raising this same original issuer limit preserves every spent credit.
    // The retry must pay for both complete scans again, without new Work.
    let retry_exact = one_short + 10;
    fixture.limits(16_000_000, 16_000_000, retry_exact);
    assert!(Arc::ptr_eq(&work, &original_work));
    assert_eq!(work.spent.load(Ordering::Relaxed), one_short);
    assert!(
        captured_fully_active(&graph, creation.document, creation.navigable, || {
            work.step(&AtomicBool::new(false))
        })
        .unwrap()
    );
    assert_eq!(work.spent.load(Ordering::Relaxed), retry_exact);
    assert_eq!(*graph, *original_graph);
    assert_eq!(fixture.source.revision().unwrap(), original_revision);
    assert_eq!(fixture.used(), original_bytes);
}

#[test]
fn last_retained_top_activity_observes_cancellation_during_bank_scan() {
    let (fixture, graph, creation, work) = last_retained_top();
    let before = work.spent.load(Ordering::Relaxed);
    let original_graph = graph.clone();
    let original_revision = fixture.source.revision().unwrap();
    let original_bytes = fixture.used();
    let cancel = AtomicBool::new(false);
    let mut reached = 0;
    let result = captured_fully_active(&graph, creation.document, creation.navigable, || {
        reached += 1;
        if reached == 3 {
            cancel.store(true, Ordering::Relaxed);
        }
        work.step(&cancel)
    });
    assert!(
        matches!(result, Err(DomHostedError::Dom(DomError::Cancelled))),
        "the third reached scan admission must observe actual cancellation: {result:?}"
    );
    assert_eq!(reached, 3);
    assert_eq!(work.spent.load(Ordering::Relaxed), before + 2);
    assert_eq!(*graph, *original_graph);
    assert_eq!(fixture.source.revision().unwrap(), original_revision);
    assert_eq!(fixture.used(), original_bytes);
}
