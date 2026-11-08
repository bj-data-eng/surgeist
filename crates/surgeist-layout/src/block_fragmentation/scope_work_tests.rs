use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;

fn principal_and_content<S: LayoutScalar>() {
    let block = || NodeInputOf::<S> {
        display: Display::Block,
        ..Default::default()
    };
    let mut root = block();
    root.max_size.height = MaxSizeOf::px(S::from_f64(70.0));
    let tree = PublicLayoutTreeOf::new()
        .style(0, root)
        .style(1, block())
        .style(2, block())
        .measure(1, Size::new(S::from_f64(100.0), S::from_f64(30.0)))
        .measure(2, Size::new(S::from_f64(60.0), S::from_f64(30.0)))
        .children(0, [1, 2]);
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let fallback = OrthogonalFallbackOf::new(
        Size::NONE,
        NearestScrollportOf::Absent,
        Size::splat(OrthogonalFallbackLimitOf::Unbounded),
    );
    let request = LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(
                        NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
                    ),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            fallback,
        ),
        axes,
        Size::NONE,
    );
    let frames = [
        BlockFragmentainerOf {
            origin: Point::ZERO,
            size: Size::new(S::from_f64(100.0), S::from_f64(50.0)),
            breaks: FragmentationBreakKinds::NONE,
            page_side: None,
        },
        BlockFragmentainerOf {
            origin: Point::ZERO,
            size: Size::new(S::from_f64(60.0), S::from_f64(80.0)),
            breaks: FragmentationBreakKinds::PAGE,
            page_side: None,
        },
    ];
    let bound = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        request,
        FragmentationKind::Page {
            progression: crate::PageProgression::LeftToRight,
        },
        axes,
        frames.to_vec(),
    )
    .unwrap();
    let capture = LayoutFlowInput::try_new_bound(
        &tree,
        0,
        bound.clone(),
        Vec::new(),
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let inventory = Inventory::new(&tree, &capture.graph, axes).unwrap();
    let scopes = Scopes::new(inventory, &tree, axes).unwrap();
    let environment = NativeProvider {
        capture: &capture,
        inventory: &scopes.source,
        frames: &frames,
        axes,
        kind: FragmentationKind::Page {
            progression: crate::PageProgression::LeftToRight,
        },
        target: bound.target_basis(),
        template: request,
        intrinsics: Rc::new(RefCell::new(Vec::new())),
    };
    let resolved = Driver::new(&environment, &scopes).unwrap().run().unwrap();
    let records = resolved.records.borrow();
    let principal = records[resolved.root.0].as_ref().unwrap();
    assert_eq!(principal.report.report().pieces.len(), 2);
    assert_eq!(
        principal.report.report().pieces[0].block_size,
        S::from_f64(50.0)
    );
    assert_eq!(
        principal.report.report().pieces[1].block_size,
        S::from_f64(20.0)
    );
    assert_eq!(principal.frames.len(), 2);
    assert_eq!(principal.frames[0].receipts, [ReceiptId(1)]);
    assert_eq!(principal.frames[1].receipts, [ReceiptId(1)]);
    let content = records[1].as_ref().unwrap();
    assert_eq!(content.report.report().pieces.len(), 2);
    assert_eq!(
        content.report.report().pieces[0].block_size,
        S::from_f64(30.0)
    );
    assert_eq!(
        content.report.report().pieces[1].block_size,
        S::from_f64(30.0)
    );
    let timeline = content.timeline.as_ref().unwrap();
    assert_eq!(timeline.body_extent, S::from_f64(70.0));
    assert_eq!(timeline.spans[0].content, S::from_f64(50.0));
    assert_eq!(timeline.spans[1].content, S::from_f64(20.0));
    // Each actual leaf belongs only to the content inventory; the principal
    // execution consumes structural owner/end occurrences with empty ranges.
    assert!(
        principal
            .report
            .report()
            .pieces
            .iter()
            .flat_map(|piece| &piece.coverage)
            .all(|coverage| matches!(
                coverage.key,
                SourceKey::Owner(0) | SourceKey::PrincipalEnd(0)
            ) && coverage.start == 0
                && coverage.end == 0)
    );
    for leaf in [1, 2] {
        let coverage: Vec<_> = content
            .report
            .report()
            .pieces
            .iter()
            .flat_map(|piece| &piece.coverage)
            .filter(|coverage| coverage.key == SourceKey::Atomic(leaf))
            .collect();
        assert_eq!(coverage.len(), 1);
        assert_eq!((coverage[0].start, coverage[0].end), (0, 1));
    }
}
#[test]
fn scope_work_principal_and_content_progress_are_distinct_f32() {
    principal_and_content::<f32>();
}
#[test]
fn scope_work_principal_and_content_progress_are_distinct_f64() {
    principal_and_content::<f64>();
}

fn cross_progression<S: LayoutScalar>(forced: bool) {
    for (cap, child) in [(20.0, 20.0), (1.0, 150.0)] {
        let block = |width: Option<f64>, mode| {
            let mut input = NodeInputOf::<S> {
                display: Display::Block,
                writing_mode: mode,
                ..Default::default()
            };
            if let Some(width) = width {
                input.size.width = PreferredSizeOf::px(S::from_f64(width));
            }
            input
        };
        let mut owner = block(None, WritingMode::VerticalRl);
        owner.max_size.width = MaxSizeOf::px(S::from_f64(cap));
        let mut final_child = block(Some(0.0), WritingMode::VerticalRl);
        if forced {
            final_child.break_before = FragmentationBreak::Page;
        }
        let tree = PublicLayoutTreeOf::new()
            .style(0, block(None, WritingMode::VerticalRl))
            .style(1, block(None, WritingMode::VerticalLr))
            .style(2, owner)
            .style(3, block(Some(child), WritingMode::VerticalRl))
            .style(4, block(Some(10.0), WritingMode::VerticalLr))
            .style(5, final_child)
            .children(0, [1])
            .children(1, [2, 4])
            .children(2, if forced { vec![3, 5] } else { vec![3] });
        let axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
        let fallback = OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        );
        let request = LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes.physical_size(crate::geometry::LogicalSizeOf::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(
                            NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
                        ),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                )),
                fallback,
            ),
            axes,
            Size::NONE,
        );
        let frames = [BlockFragmentainerOf {
            origin: Point::ZERO,
            size: Size::new(S::from_f64(200.0), S::from_f64(100.0)),
            breaks: FragmentationBreakKinds::NONE,
            page_side: None,
        }];
        let bound = LayoutBoundRequestOf::block_fragmentation(
            LayoutSourceBasis::fresh(),
            request,
            FragmentationKind::Page {
                progression: crate::PageProgression::LeftToRight,
            },
            axes,
            frames.to_vec(),
        )
        .unwrap();
        let capture = LayoutFlowInput::try_new_bound(
            &tree,
            0,
            bound.clone(),
            Vec::new(),
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let inventory = Inventory::new(&tree, &capture.graph, axes).unwrap();
        let scopes = Scopes::new(inventory, &tree, axes).unwrap();
        let environment = NativeProvider {
            capture: &capture,
            inventory: &scopes.source,
            frames: &frames,
            axes,
            kind: FragmentationKind::Page {
                progression: crate::PageProgression::LeftToRight,
            },
            target: bound.target_basis(),
            template: request,
            intrinsics: Rc::new(RefCell::new(Vec::new())),
        };
        let result = Driver::new(&environment, &scopes).unwrap().run();
        if forced {
            assert!(matches!(result, Err(error) if matches!(error.kind(),
                LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::BlockFragmentationProfile))));
        } else {
            let resolved = result.unwrap();
            let records = resolved.records.borrow();
            let content = records
                .iter()
                .flatten()
                .find(|record| scopes.source.nodes[scopes.scopes[record.scope.0].owner].node == 2)
                .unwrap();
            let placement = content.placement.as_ref().unwrap();
            // The opposite wrapper's completed width is owner + 10. Its
            // physical x=0 owner therefore starts 10 units into the RL flow.
            assert_eq!(placement.origin.block, S::from_f64(10.0));
            assert_eq!(placement.origin.inline, S::ZERO);
            assert_eq!(content.frames.len(), 1);
            let principal = content.principals.borrow();
            assert!(
                principal
                    .iter()
                    .any(|entry| entry.output.size.width == S::from_f64(cap.min(child)))
            );
        }
    }
}
#[test]
fn scope_work_whole_opposite_ancestor_uses_completed_physical_origin_f32() {
    cross_progression::<f32>(false);
}
#[test]
fn scope_work_whole_opposite_ancestor_uses_completed_physical_origin_f64() {
    cross_progression::<f64>(false);
}
#[test]
fn scope_work_whole_opposite_qualification_refuses_partitioned_descendant_force_f32() {
    cross_progression::<f32>(true);
}
#[test]
fn scope_work_whole_opposite_qualification_refuses_partitioned_descendant_force_f64() {
    cross_progression::<f64>(true);
}
