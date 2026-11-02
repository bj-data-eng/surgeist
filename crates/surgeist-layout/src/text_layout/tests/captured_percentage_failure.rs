//! Final relative-percentage overflow cannot publish stale visual geometry.
use super::*;

fn captured_final_percent_failure_preserves_publication<S: LayoutScalar>(maximum: S) {
    assert!(maximum.to_f64().is_finite() && maximum > S::ZERO);
    let paragraph = custody_paragraph(false);
    let initial = custody_n::<S>(0.06) * custody_n::<S>(12.0);
    let mut host = CustodyHost::<S>::new(custody_restorable_fixture(
        &paragraph,
        false,
        initial.to_f64(),
    ));
    let binding = paragraph.binding(0);
    assert_eq!(binding.nodes, vec![1, 2]);
    assert_eq!(binding.atomics.len(), 1);
    assert_eq!(binding.atomics[0].node, 2);
    assert_eq!(
        binding.atomics[0].occurrence,
        paragraph.prepared.atomic_views().next().unwrap().handle()
    );
    assert!(host.node_input(2).atomic_inline_participation.is_none());
    assert_eq!(host.node_input(2).size, custody_atom::<S>(false, 0.0).size);

    // This is a genuine successful constant-control publication, independent
    // of whether final-auto percentage settlement has yet been implemented.
    let baseline = custody_compute(&host, vec![binding.clone()], &[]);
    custody_assert_simple(&baseline, false, 0.72);
    captured_percent_source(
        &baseline,
        &paragraph,
        false,
        50.0,
        12.0,
        &paragraph.binding(0).nodes,
    );
    let original_fragments = baseline
        .final_inline_fragments()
        .iter()
        .map(|row| row.fragment().finalized_fragment().clone())
        .collect::<Vec<_>>();
    let original_pending = custody_entry(&baseline, 2, false);
    baseline.apply_to(&mut host).unwrap();
    assert!(host.inner.caches.contains_key(&2));
    let prior_caches = host.inner.caches.clone();
    let prior = host.inner.published.as_ref().unwrap().clone();
    assert_eq!(custody_entry(&prior, 2, false), original_pending);

    // The coefficient itself is finite and valid. Initial absent height is
    // MissingBasis, not invalid arithmetic. Once actual normal contentH12 is
    // available, the documented S product is positive infinity in each lane.
    let coefficient = LengthPercentageOf::from_percent_fraction(maximum).unwrap();
    assert_eq!(coefficient.percent_fraction(), maximum);
    let overflow_top = LengthAutoOf::value(coefficient);
    assert_eq!(
        overflow_top.resolve_with_status(None).status(),
        crate::LengthResolutionStatus::MissingBasis
    );
    let expected_product = maximum * custody_n::<S>(12.0);
    assert_eq!(expected_product.to_f64(), f64::INFINITY);
    let mut overflowing = custody_atom(false, 0.0);
    overflowing.inset.top = overflow_top;
    host.inner
        .tree
        .insert_input(2, LayoutInputOf::box_input(overflowing));

    // Admit the real captured input, then observe the owning fallible compute
    // directly. The success-only custody_compute helper must not hide errors.
    let outcome = {
        let flow = LayoutFlowInput::try_new(
            &host,
            0,
            vec![binding.clone()],
            crate::WrappingInput {
                native_inline_boxes: Vec::new(),
                gaps: Vec::new(),
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        compute_layout_with_flow_invalidated(
            &flow,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(custody_n::<S>(50.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
            &[2],
        )
    };
    let Err(error) = outcome else {
        panic!("finite TOP coefficient must fail when final normalH12 makes its product infinite");
    };
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(2));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    match error.kind() {
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value }) => {
            assert_eq!(*value, expected_product);
            assert_eq!(value.to_f64(), f64::INFINITY);
        }
        other => panic!("expected owning final value-resolution arithmetic error, got {other:?}"),
    }

    assert_eq!(host.inner.caches, prior_caches);
    custody_same_scene(host.inner.published.as_ref().unwrap(), &prior);
    assert_eq!(
        custody_entry(host.inner.published.as_ref().unwrap(), 2, false),
        original_pending
    );
    custody_assert_simple(&baseline, false, 0.72);
    captured_percent_source(
        &baseline,
        &paragraph,
        false,
        50.0,
        12.0,
        &paragraph.binding(0).nodes,
    );
    assert_eq!(
        baseline
            .final_inline_fragments()
            .iter()
            .map(|row| row.fragment().finalized_fragment().clone())
            .collect::<Vec<_>>(),
        original_fragments
    );

    // Restore the real finite6% coefficient, not a stale successful output.
    // NormalH12/reference10/FIT20 remain unchanged; .72 yields the same checked
    // visual0..11 border and1..10 content as the independent constant control.
    host.inner
        .tree
        .insert_input(2, LayoutInputOf::box_input(captured_percent_atom(false)));
    let recovered = custody_compute(&host, vec![binding], &[2]);
    captured_percent_source(
        &recovered,
        &paragraph,
        false,
        50.0,
        12.0,
        &paragraph.binding(0).nodes,
    );
    custody_assert_simple(&recovered, false, 0.72);
    custody_same_scene(&recovered, &baseline);
    // Successful staging still leaves the old accepted publication untouched.
    custody_same_scene(host.inner.published.as_ref().unwrap(), &prior);
    assert_eq!(host.inner.caches, prior_caches);
    recovered.apply_to(&mut host).unwrap();
    custody_same_scene(host.inner.published.as_ref().unwrap(), &recovered);
    assert_eq!(
        baseline
            .final_inline_fragments()
            .iter()
            .map(|row| row.fragment().finalized_fragment().clone())
            .collect::<Vec<_>>(),
        original_fragments
    );
    custody_assert_simple(&baseline, false, 0.72);
}

#[test]
fn captured_final_percent_overflow_is_typed_and_recoverable_f32() {
    captured_final_percent_failure_preserves_publication::<f32>(f32::MAX);
}
#[test]
fn captured_final_percent_overflow_is_typed_and_recoverable_f64() {
    captured_final_percent_failure_preserves_publication::<f64>(f64::MAX);
}
