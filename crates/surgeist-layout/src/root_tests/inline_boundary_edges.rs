//! Endpoint-only changes keep native preparation and transactional cache custody.
use super::fixtures::Fri06C02StatefulTextTree;
use super::*;
use crate::test_support::inline_endpoint::Fixture;
use crate::test_support::text::ahem_style;
use surgeist_text as text;

fn tree<S: LayoutScalar>(
    fixture: &Fixture,
    start: InlineBoundaryEdgeOf<S>,
) -> Fri06C02StatefulTextTree<S> {
    let mut tree = Fri06C02StatefulTextTree::new(text::Source::new("AB"), ahem_style(10.0, 10.0));
    tree.text = fixture.binding.clone();
    tree.inputs.clear();
    tree.children.clear();
    tree.node_inputs.clear();
    tree.inputs.insert(
        0,
        LayoutInputOf::box_input(NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(10.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        }),
    );
    tree.children.insert(0, fixture.binding.nodes.clone());
    for row in &fixture.binding.origins {
        tree.inputs.insert(
            row.node,
            LayoutInputOf::inline_text(TextParticipation::new(row.origin.clone())),
        );
    }
    for row in &fixture.binding.controls {
        let input = if row.node == 22 {
            LayoutInputOf::text_control(TextControlParticipation::new(row.occurrence.clone()))
        } else {
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::contribution(
                if row.node == 20 {
                    InlineBoundaryKind::Start
                } else {
                    InlineBoundaryKind::End
                },
                InlineMetricsOf::try_new(
                    crate::InlineBaselineContext::Physical,
                    S::from_f64(8.0),
                    S::from_f64(10.0),
                )
                .unwrap(),
                if row.node == 20 {
                    start
                } else {
                    InlineBoundaryEdgeOf::ZERO
                },
            ))
        };
        tree.inputs.insert(row.node, input);
    }
    for &node in &fixture.binding.nodes {
        tree.children.insert(node, vec![]);
        tree.node_inputs.insert(node, NodeInputOf::non_box());
    }
    tree
}

fn first(batch: &CompletedLayoutBatchOf<u32, impl LayoutScalar>) -> text::FinalizedLine {
    batch
        .unrounded_inline_fragments()
        .iter()
        .map(|entry| entry.fragment().finalized_fragment().finalized_line())
        .find(|line| line.selected_line().request().context.start == text::LineStart::Paragraph)
        .unwrap()
}

#[test]
fn demanded_off_selection_endpoint_changes_warm_requests_without_allocating_outgoing_flow() {
    fn lane<S: LayoutScalar>() {
        let fixture = Fixture::at_soft_seam();
        let request = LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(10.0)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        let mut actual = tree::<S>(&fixture, InlineBoundaryEdgeOf::ZERO);
        let cold = actual.compute(request).unwrap();
        let zero = first(&cold);
        let zero_request = zero.selected_line().request().clone();
        assert_eq!(zero.feedback().advance, 10.0);
        assert!(!zero_request.boundary_context.is_empty());
        cold.apply_to(&mut actual).unwrap();
        let warm = actual.compute(request).unwrap();
        assert_eq!(warm.unrounded_entries(), cold.unrounded_entries());
        assert_eq!(first(&warm).selected_line().request(), &zero_request);
        warm.apply_to(&mut actual).unwrap();
        let edge = InlineBoundaryEdgeOf::try_new(S::from_f64(2.0), S::from_f64(3.0)).unwrap();
        actual.inputs.insert(
            20,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::contribution(
                InlineBoundaryKind::Start,
                InlineMetricsOf::try_new(
                    crate::InlineBaselineContext::Physical,
                    S::from_f64(8.0),
                    S::from_f64(10.0),
                )
                .unwrap(),
                edge,
            )),
        );
        let before = actual.retained.clone();
        let changed = actual.compute_invalidated(request, &[20]).unwrap();
        assert_eq!(actual.retained, before);
        let changed_first = first(&changed);
        assert_eq!(changed_first.feedback().advance, 10.0);
        let selected = changed_first.selected_line();
        assert_eq!(selected.request().participants, zero_request.participants);
        assert_eq!(selected.request().controls, zero_request.controls);
        assert_eq!(selected.request().boundary_context.len(), 1);
        assert_eq!(
            selected.request().boundary_context[0].boundary.advance(),
            5.0
        );
        assert_eq!(
            selected.request().boundary_context[0].occurrence,
            fixture
                .binding
                .controls
                .iter()
                .find(|row| row.node == 20)
                .unwrap()
                .occurrence
        );
        let fresh = tree::<S>(&fixture, edge).compute(request).unwrap();
        assert_eq!(changed.unrounded_entries(), fresh.unrounded_entries());
        assert_eq!(selected.request(), first(&fresh).selected_line().request());
        assert_eq!(zero.selected_line().request(), &zero_request);
        assert_eq!(zero.feedback().advance, 10.0);
        actual.reject_preparation = true;
        assert_eq!(
            changed.apply_to(&mut actual),
            Err("C02 retained-state preparation rejected")
        );
        assert_eq!(actual.retained, before);
        actual.reject_preparation = false;
        changed.apply_to(&mut actual).unwrap();
        let repeated = actual.compute(request).unwrap();
        assert_eq!(repeated.unrounded_entries(), fresh.unrounded_entries());
        let largest = if S::from_f64(f64::MAX).is_finite() {
            f64::MAX
        } else {
            f64::from(f32::MAX)
        };
        let overflow = InlineBoundaryEdgeOf::try_new(S::from_f64(largest), S::ZERO).unwrap();
        actual.inputs.insert(
            20,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::contribution(
                InlineBoundaryKind::Start,
                InlineMetricsOf::try_new(
                    crate::InlineBaselineContext::Physical,
                    S::from_f64(8.0),
                    S::from_f64(10.0),
                )
                .unwrap(),
                overflow,
            )),
        );
        actual.inputs.insert(
            21,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::contribution(
                InlineBoundaryKind::End,
                InlineMetricsOf::try_new(
                    crate::InlineBaselineContext::Physical,
                    S::from_f64(8.0),
                    S::from_f64(10.0),
                )
                .unwrap(),
                overflow,
            )),
        );
        let retained = actual.retained.clone();
        let error = actual.compute_invalidated(request, &[20, 21]).unwrap_err();
        let LayoutErrorKindOf::Text(error) = error.kind() else {
            panic!("actual endpoint arithmetic must retain Text's numeric refusal: {error:?}")
        };
        assert_eq!(error.code, text::ErrorCode::InvalidTextNumeric);
        assert!(matches!(
            error.detail(),
            Some(text::ErrorDetail::InvalidTextNumeric {
                phase: text::TextNumericPhase::Finalization,
                category: text::TextNumericError::NonFinite | text::TextNumericError::NonZeroLost,
                ..
            })
        ));
        assert_eq!(actual.retained, retained);
        actual.inputs.insert(
            20,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::contribution(
                InlineBoundaryKind::Start,
                InlineMetricsOf::try_new(
                    crate::InlineBaselineContext::Physical,
                    S::from_f64(8.0),
                    S::from_f64(10.0),
                )
                .unwrap(),
                edge,
            )),
        );
        actual.inputs.insert(
            21,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::contribution(
                InlineBoundaryKind::End,
                InlineMetricsOf::try_new(
                    crate::InlineBaselineContext::Physical,
                    S::from_f64(8.0),
                    S::from_f64(10.0),
                )
                .unwrap(),
                InlineBoundaryEdgeOf::ZERO,
            )),
        );
        let retry = actual.compute_invalidated(request, &[20, 21]).unwrap();
        assert_eq!(retry.unrounded_entries(), fresh.unrounded_entries());
        retry.apply_to(&mut actual).unwrap();
        actual.inputs.insert(
            20,
            LayoutInputOf::inline_boundary(InlineBoundaryInputOf::contribution(
                InlineBoundaryKind::Start,
                InlineMetricsOf::try_new(
                    crate::InlineBaselineContext::Physical,
                    S::from_f64(8.0),
                    S::from_f64(10.0),
                )
                .unwrap(),
                InlineBoundaryEdgeOf::ZERO,
            )),
        );
        let restored = actual.compute_invalidated(request, &[20]).unwrap();
        assert_eq!(restored.unrounded_entries(), cold.unrounded_entries());
        assert_eq!(first(&restored).selected_line().request(), &zero_request);
    }
    lane::<f32>();
    lane::<f64>();
}
