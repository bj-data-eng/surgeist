use super::*;

fn rect<S: LayoutScalar>(left: f64, top: f64, width: f64, height: f64) -> ScrollRectOf<S> {
    ScrollRectOf::try_new(
        crate::Point::new(S::from_f64(left), S::from_f64(top)),
        crate::Size::new(S::from_f64(width), S::from_f64(height)),
    )
    .unwrap()
}

type TestNode<S> = ScrollAnchorNodeOf<'static, u32, S>;

fn node<S: LayoutScalar>(
    parent: Option<u32>,
    children: &'static [u32],
    subject: ScrollAnchorSubjectOf<u32, S>,
) -> TestNode<S> {
    ScrollAnchorNodeOf {
        parent,
        children,
        positioned: &[],
        subject,
    }
}

fn element<S: LayoutScalar>(box_kind: ScrollAnchorBoxOf<S>) -> ScrollAnchorSubjectOf<u32, S> {
    ScrollAnchorSubjectOf::Element {
        box_kind,
        policy: ScrollAnchorPolicy::Auto,
        position: ScrollAnchorPosition::Ordinary,
    }
}

fn visible<S: LayoutScalar>() -> ScrollAnchorSubjectOf<u32, S> {
    element(ScrollAnchorBoxOf::Eligible(rect(5.0, 5.0, 20.0, 20.0)))
}

fn request<S: LayoutScalar>() -> ScrollAnchorRequestOf<u32, S> {
    ScrollAnchorRequestOf {
        scroller: 0,
        optimal_viewing_region: rect(0.0, 0.0, 100.0, 100.0),
        focused: ScrollAnchorPriority::Absent,
        find_match: ScrollAnchorPriority::Absent,
    }
}

fn select<S: LayoutScalar>(
    request: ScrollAnchorRequestOf<u32, S>,
    nodes: &[TestNode<S>],
) -> Result<Option<u32>, ScrollAnchorSelectionError<u32>> {
    select_scroll_anchor(request, |node| nodes.get(node as usize).copied())
}

#[test]
fn scroll_anchor_selection_respects_priority_and_dom_walk() {
    fn lane<S: LayoutScalar>() {
        let nodes = [
            node(None, &[1, 4], ScrollAnchorSubjectOf::NonElement),
            node(
                Some(0),
                &[2, 3],
                element(ScrollAnchorBoxOf::Eligible(rect(0.0, -10.0, 30.0, 40.0))),
            ),
            node(Some(1), &[], element(ScrollAnchorBoxOf::Inline(None))),
            node(Some(1), &[], visible()),
            node(Some(0), &[], visible()),
            node(None, &[], visible()),
        ];
        let request = request::<S>();

        assert_eq!(select(request, &nodes), Ok(Some(3)));
        assert_eq!(
            select(
                ScrollAnchorRequestOf {
                    focused: ScrollAnchorPriority::Candidate(2),
                    find_match: ScrollAnchorPriority::Candidate(4),
                    ..request
                },
                &nodes,
            ),
            Ok(Some(1)),
        );
        assert_eq!(
            select(
                ScrollAnchorRequestOf {
                    focused: ScrollAnchorPriority::Candidate(5),
                    find_match: ScrollAnchorPriority::Candidate(4),
                    ..request
                },
                &nodes,
            ),
            Ok(Some(4)),
        );
        let mut fully_visible = nodes;
        fully_visible[1].subject = visible();
        assert_eq!(select(request, &fully_visible), Ok(Some(1)));
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn scroll_anchor_selection_uses_the_absolute_owner_after_dom_children() {
    fn lane<S: LayoutScalar>() {
        let absolute = |containing_block| ScrollAnchorSubjectOf::Element {
            box_kind: ScrollAnchorBoxOf::Eligible(rect(5.0, 5.0, 20.0, 20.0)),
            policy: ScrollAnchorPolicy::Auto,
            position: ScrollAnchorPosition::Absolute(containing_block),
        };
        let clipped = element(ScrollAnchorBoxOf::Eligible(rect(0.0, -40.0, 20.0, 10.0)));
        let mut nodes = [
            node(None, &[1, 5], ScrollAnchorSubjectOf::NonElement),
            ScrollAnchorNodeOf {
                positioned: &[3],
                ..node(
                    Some(0),
                    &[2, 4],
                    element(ScrollAnchorBoxOf::Eligible(rect(0.0, -10.0, 30.0, 40.0))),
                )
            },
            node(Some(1), &[], visible()),
            node(
                Some(4),
                &[],
                absolute(ScrollAnchorContainingBlock::Element(1)),
            ),
            node(Some(1), &[3], clipped),
            node(Some(0), &[], visible()),
        ];
        let request = request::<S>();

        assert_eq!(select(request, &nodes), Ok(Some(2)));
        nodes[2].subject = clipped;
        assert_eq!(select(request, &nodes), Ok(Some(3)));

        nodes[4].subject = ScrollAnchorSubjectOf::Element {
            box_kind: ScrollAnchorBoxOf::Eligible(rect(0.0, -40.0, 20.0, 10.0)),
            policy: ScrollAnchorPolicy::None,
            position: ScrollAnchorPosition::Ordinary,
        };
        assert_eq!(select(request, &nodes), Ok(Some(1)));
        assert_eq!(
            select(
                ScrollAnchorRequestOf {
                    focused: ScrollAnchorPriority::Candidate(3),
                    find_match: ScrollAnchorPriority::Candidate(5),
                    ..request
                },
                &nodes,
            ),
            Ok(Some(5)),
        );

        let above = [
            node(None, &[1, 2], ScrollAnchorSubjectOf::NonElement),
            node(
                Some(0),
                &[],
                absolute(ScrollAnchorContainingBlock::AboveScroller),
            ),
            node(Some(0), &[], visible()),
        ];
        assert_eq!(select(request, &above), Ok(Some(2)));
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn scroll_anchor_selection_restarts_eligibility_at_each_scroller() {
    let disabled = ScrollAnchorSubjectOf::Element {
        box_kind: ScrollAnchorBoxOf::Eligible(rect(0.0, 0.0, 100.0, 100.0)),
        policy: ScrollAnchorPolicy::None,
        position: ScrollAnchorPosition::Ordinary,
    };
    let nodes = [
        node(None, &[1, 3], ScrollAnchorSubjectOf::NonElement),
        node(Some(0), &[2], disabled),
        node(Some(1), &[4], visible()),
        node(Some(0), &[], visible()),
        node(Some(2), &[], visible()),
    ];
    let request = request::<f64>();

    assert_eq!(select(request, &nodes), Ok(Some(3)));
    assert_eq!(
        select(
            ScrollAnchorRequestOf {
                scroller: 1,
                focused: ScrollAnchorPriority::Unavailable,
                ..request
            },
            &nodes,
        ),
        Ok(None),
    );
    assert_eq!(
        select(
            ScrollAnchorRequestOf {
                scroller: 2,
                ..request
            },
            &nodes,
        ),
        Ok(Some(4)),
    );
}

#[test]
fn scroll_anchor_selection_skips_exclusions_and_edges_without_visible_area() {
    let view = request::<f64>();
    let ordinary = ScrollAnchorPosition::Ordinary;
    let subject = |box_kind, position| ScrollAnchorSubjectOf::Element {
        box_kind,
        policy: ScrollAnchorPolicy::Auto,
        position,
    };
    let mut nodes = [
        node(None, &[1, 3, 4, 5, 6], ScrollAnchorSubjectOf::NonElement),
        node(
            Some(0),
            &[2],
            subject(ScrollAnchorBoxOf::Suppressed, ordinary),
        ),
        node(Some(1), &[], visible()),
        node(
            Some(0),
            &[],
            subject(
                ScrollAnchorBoxOf::Eligible(rect(5.0, 5.0, 20.0, 20.0)),
                ScrollAnchorPosition::Fixed,
            ),
        ),
        node(
            Some(0),
            &[],
            element(ScrollAnchorBoxOf::Eligible(rect(100.0, 2.0, 20.0, 20.0))),
        ),
        node(
            Some(0),
            &[],
            element(ScrollAnchorBoxOf::Eligible(rect(6.0, 2.0, 0.0, 20.0))),
        ),
        node(Some(0), &[], visible()),
    ];
    assert_eq!(select(view, &nodes), Ok(Some(6)));
    nodes[0].children = &[1, 3, 4, 5];
    assert_eq!(select(view, &nodes), Ok(None));

    nodes[1].subject = element(ScrollAnchorBoxOf::None);
    assert_eq!(select(view, &nodes), Ok(Some(2)));
}

#[test]
fn scroll_anchor_selection_reports_required_missing_authority() {
    let request = request::<f32>();
    let nodes = [
        node(None, &[1], ScrollAnchorSubjectOf::NonElement),
        node(Some(0), &[], visible()),
    ];
    assert_eq!(
        select(
            ScrollAnchorRequestOf {
                focused: ScrollAnchorPriority::Unavailable,
                ..request
            },
            &nodes,
        ),
        Err(ScrollAnchorSelectionError::FocusUnavailable),
    );
    assert_eq!(
        select(
            ScrollAnchorRequestOf {
                find_match: ScrollAnchorPriority::Unavailable,
                ..request
            },
            &nodes,
        ),
        Err(ScrollAnchorSelectionError::FindMatchUnavailable),
    );
    assert_eq!(
        select(
            ScrollAnchorRequestOf {
                focused: ScrollAnchorPriority::Candidate(1),
                find_match: ScrollAnchorPriority::Unavailable,
                ..request
            },
            &nodes,
        ),
        Ok(Some(1)),
    );
    assert_eq!(
        select(request, &nodes[..1]),
        Err(ScrollAnchorSelectionError::MissingNode(1)),
    );
}

#[test]
fn scroll_anchor_selection_uses_canonical_layout_overflow_and_scroll_padding() {
    use crate::test_support::layout_tree::PublicLayoutTreeOf;
    use crate::{
        AvailableSpaceOf, AxisSizingRequestOf, BoxSizing, ComputedOverflow, Direction, Edges,
        FlowAxes, LayoutRootRequestOf, LengthOf, LengthPercentageOf, NearestScrollportOf,
        NodeInputOf, OrthogonalFallbackLimitOf, OrthogonalFallbackOf, Overflow, PreferredSizeOf,
        ScrollPaddingOf, ScrollPaddingValueOf, Size, SizingOperation, SizingRequestOf, WritingMode,
        compute_layout,
    };

    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        let root = NodeInputOf {
            display: crate::Display::Block,
            size: Size::splat(px(100.0)).map(PreferredSizeOf::px),
            overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap(),
            scroll_padding: ScrollPaddingOf::new(
                ScrollPaddingValueOf::value(LengthPercentageOf::px(px(20.0)).unwrap()),
                ScrollPaddingValueOf::AUTO,
                ScrollPaddingValueOf::AUTO,
                ScrollPaddingValueOf::AUTO,
            ),
            ..NodeInputOf::default()
        };
        let parent = NodeInputOf {
            display: crate::Display::Block,
            size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(20.0))),
            box_sizing: BoxSizing::BorderBox,
            padding: Edges {
                top: LengthOf::px(px(20.0)),
                ..Edges::all(LengthOf::ZERO)
            },
            ..NodeInputOf::default()
        };
        let leaf = NodeInputOf {
            display: crate::Display::Block,
            size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(10.0))),
            ..NodeInputOf::default()
        };
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 3])
            .children(1, [2])
            .style(0, root)
            .style(1, parent)
            .style(2, leaf.clone())
            .style(3, leaf);
        let layout_request = LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                Size::splat(AxisSizingRequestOf::new(
                    AvailableSpaceOf::Indefinite,
                    SizingOperation::Normal,
                )),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::NONE,
        );
        let batch = compute_layout(&tree, 0, layout_request).unwrap();
        let output = |node| {
            batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        };
        let region = output(0).scroll_geometry.unwrap().optimal_viewing_region();
        let offset =
            |a: crate::Point<S>, b: crate::Point<S>| crate::Point::new(a.x + b.x, a.y + b.y);
        let overflow = |node, location| {
            let bounds = output(node).scroll_geometry.unwrap().scrollable_overflow();
            ScrollRectOf::try_new(offset(bounds.origin(), location), bounds.size()).unwrap()
        };
        let parent_output = output(1);
        let parent_bounds = overflow(1, parent_output.location);
        let child_bounds = overflow(2, offset(output(2).location, parent_output.location));
        let sibling_bounds = overflow(3, output(3).location);
        assert_eq!(region, rect(0.0, 20.0, 100.0, 80.0));
        assert_eq!(parent_output.size.height, px(20.0));
        assert_eq!(parent_bounds, rect(0.0, 0.0, 100.0, 30.0));
        assert_eq!(child_bounds, rect(0.0, 20.0, 100.0, 10.0));
        let nodes = [
            node(None, &[1, 3], ScrollAnchorSubjectOf::NonElement),
            node(
                Some(0),
                &[2],
                element(ScrollAnchorBoxOf::Eligible(parent_bounds)),
            ),
            node(
                Some(1),
                &[],
                element(ScrollAnchorBoxOf::Eligible(child_bounds)),
            ),
            node(
                Some(0),
                &[],
                element(ScrollAnchorBoxOf::Eligible(sibling_bounds)),
            ),
        ];
        assert_eq!(
            select(
                ScrollAnchorRequestOf {
                    optimal_viewing_region: region,
                    ..request::<S>()
                },
                &nodes
            ),
            Ok(Some(2)),
        );
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn scroll_anchor_selection_refuses_inconsistent_reached_relations() {
    let request = request::<f32>();
    let inconsistent_dom = [
        node(None, &[1], ScrollAnchorSubjectOf::NonElement),
        node(None, &[], visible()),
    ];
    assert_eq!(
        select(request, &inconsistent_dom),
        Err(ScrollAnchorSelectionError::InconsistentDomChild {
            parent: 0,
            child: 1,
        }),
    );

    let inconsistent_positioned = [
        node(None, &[1, 2], ScrollAnchorSubjectOf::NonElement),
        ScrollAnchorNodeOf {
            positioned: &[2],
            ..node(
                Some(0),
                &[],
                element(ScrollAnchorBoxOf::Eligible(rect(0.0, -10.0, 30.0, 40.0))),
            )
        },
        node(Some(0), &[], visible()),
    ];
    assert_eq!(
        select(request, &inconsistent_positioned),
        Err(ScrollAnchorSelectionError::InconsistentPositionedChild { owner: 1, child: 2 }),
    );

    let cycle = [
        node(Some(1), &[1], ScrollAnchorSubjectOf::NonElement),
        node(Some(0), &[0], element(ScrollAnchorBoxOf::None)),
    ];
    assert_eq!(
        select(request, &cycle),
        Err(ScrollAnchorSelectionError::Cycle(0)),
    );
}
