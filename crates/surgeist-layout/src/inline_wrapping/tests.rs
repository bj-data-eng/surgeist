use super::*;

fn class(ancestry: &Arc<Ancestry>, scope: Option<usize>) -> AvoidClass {
    AvoidClass {
        ancestry: Arc::clone(ancestry),
        scope,
        edges: Vec::new(),
    }
}
#[test]
fn proper_ancestry_prefix_dominates_without_ranking_unrelated_depths() {
    // Scope0 contains Scope1; Scope2/3 are a separate, deeper sibling chain.
    let tree = Arc::new(Ancestry {
        intervals: vec![(1, 3), (2, 3), (3, 6), (4, 6), (5, 6)],
    });
    let mut frontier = class(&tree, None).frontier();
    frontier
        .consider(class(&tree, Some(1)), "inner first")
        .unwrap();
    frontier.consider(class(&tree, Some(0)), "outer").unwrap();
    frontier
        .consider(class(&tree, Some(4)), "unrelated deeper later")
        .unwrap();
    assert_eq!(frontier.into_preferred(), Some("unrelated deeper later"));
    let mut frontier = class(&tree, None).frontier();
    frontier.consider(class(&tree, Some(4)), "sibling").unwrap();
    frontier.consider(class(&tree, None), "outside").unwrap();
    frontier
        .consider(class(&tree, Some(0)), "inside later")
        .unwrap();
    assert_eq!(frontier.into_preferred(), Some("outside"));
}
#[test]
fn retained_incomparable_class_blocks_a_later_descendant_without_depth_scoring() {
    let tree = Arc::new(Ancestry {
        intervals: vec![(1, 3), (2, 3), (3, 5), (4, 5)],
    });
    let mut frontier = class(&tree, None).frontier();
    frontier.consider(class(&tree, Some(2)), 20).unwrap();
    frontier.consider(class(&tree, Some(1)), 30).unwrap();
    frontier.consider(class(&tree, Some(0)), 10).unwrap();
    frontier.consider(class(&tree, Some(3)), 40).unwrap();
    assert_eq!(frontier.into_preferred(), Some(10));
    let other = Arc::new(Ancestry {
        intervals: vec![(1, 2)],
    });
    let mut frontier = class(&tree, None).frontier();
    assert_eq!(
        frontier.consider(class(&other, Some(0)), 0),
        Err(WrappingScopeError::InconsistentIncidence)
    );
}

#[test]
fn native_scope_proof_checks_real_roles_order_and_non_crossing_pairs() {
    use crate::test_support::layout_tree::PublicLayoutTreeOf;
    use crate::{
        InlineBoundaryEdgeOf, InlineBoundaryInputOf, InlineBoundaryKind, InlineMetricsOf,
        LayoutInputOf, NodeInputOf,
    };
    let boundary = |kind| {
        LayoutInputOf::inline_boundary(
            InlineBoundaryInputOf::contribution(
                kind,
                InlineMetricsOf::from_line_height_and_baseline(
                    crate::InlineBaselineContext::Physical,
                    10.0f64,
                    10.0,
                )
                .unwrap(),
                InlineBoundaryEdgeOf::ZERO,
            )
            .unwrap(),
        )
    };
    let tree = PublicLayoutTreeOf::<f64>::new()
        .children(0, [1, 2, 3, 4])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .style(0, NodeInputOf::default())
        .input(1, boundary(InlineBoundaryKind::Start))
        .input(2, boundary(InlineBoundaryKind::Start))
        .input(3, boundary(InlineBoundaryKind::End))
        .input(4, boundary(InlineBoundaryKind::End));
    let (graph, _) =
        crate::engine::validation::validate_layout_request_with_text_graph(&tree, 0, &[], &[])
            .unwrap();
    let pair = |start, end| WrappingScopeInput::NativePair {
        metric_contribution: None,
        edges: Default::default(),
        start,
        end,
        wrap_inside: WrapInside::Avoid,
    };
    assert!(WrappingContext::new(&tree, &graph, &[pair(1, 4), pair(2, 3)], &[], &[]).is_ok());
    assert_eq!(
        WrappingContext::new(&tree, &graph, &[pair(1, 3), pair(2, 4)], &[], &[]).unwrap_err(),
        WrappingScopeError::CrossingScopes
    );
    assert_eq!(
        WrappingContext::new(&tree, &graph, &[pair(3, 4)], &[], &[]).unwrap_err(),
        WrappingScopeError::EndpointRole
    );
    assert_eq!(
        WrappingContext::new(&tree, &graph, &[pair(2, 3), pair(2, 3)], &[], &[]).unwrap_err(),
        WrappingScopeError::DuplicateScope
    );
    assert_eq!(
        WrappingContext::new(&tree, &graph, &[pair(1, 9)], &[], &[]).unwrap_err(),
        WrappingScopeError::NodeNotReachable
    );
}
