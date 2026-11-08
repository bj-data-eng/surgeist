//! A zero-height child accepts real source before a nonforced deferral.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;

fn scalar<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn block<S: LayoutScalar>(height: Option<f64>) -> NodeInputOf<S> {
    let mut input = NodeInputOf {
        display: Display::Block,
        ..Default::default()
    };
    if let Some(height) = height {
        input.size.height = PreferredSizeOf::px(scalar(height));
    }
    input
}

fn deferred_after_zero_extent<S: LayoutScalar>(capped: bool) {
    // Frame0 is exhausted by prefix100. A0 still has Atomic extent1 and
    // belongs there. B150 cannot fit there, so it begins in frame1 and
    // overflows its80 height as one indivisible box. Tail10 then starts at
    // frame2 y0. No break is forced and maximum200 is never exhausted.
    let mut owner = block::<S>(None);
    if capped {
        owner.max_size.height = MaxSizeOf::px(scalar(200.0));
    }
    let tree = PublicLayoutTreeOf::new()
        .style(0, block(None))
        .style(1, block(Some(100.0)))
        .style(2, owner)
        .style(3, block(Some(0.0)))
        .style(4, block(Some(150.0)))
        .style(5, block(Some(10.0)))
        .children(0, vec![1, 2, 5])
        .children(2, vec![3, 4]);
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
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            fallback,
        ),
        axes,
        Size::NONE,
    );
    let frames: Vec<_> = [(100.0, 100.0), (60.0, 80.0), (80.0, 60.0)]
        .into_iter()
        .enumerate()
        .map(|(index, (width, height))| BlockFragmentainerOf {
            origin: Point::ZERO,
            size: Size::new(scalar::<S>(width), scalar(height)),
            breaks: if index == 0 {
                FragmentationBreakKinds::NONE
            } else {
                FragmentationBreakKinds::PAGE
            },
            page_side: None,
        })
        .collect();
    let bound = LayoutBoundRequestOf::block_fragmentation(
        LayoutSourceBasis::fresh(),
        request,
        FragmentationKind::Page,
        axes,
        frames.clone(),
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
    let scopes = Scopes::new(
        Inventory::new(&tree, &capture.graph, axes).unwrap(),
        &tree,
        axes,
    )
    .unwrap();
    let environment = NativeProvider {
        capture: &capture,
        inventory: &scopes.source,
        frames: &frames,
        axes,
        kind: FragmentationKind::Page,
        target: bound.target_basis(),
        template: request,
        intrinsics: Rc::new(RefCell::new(Vec::new())),
        progress: Rc::new(RefCell::new(Default::default())),
    };
    let driver = Driver::new(&environment, &scopes).unwrap();
    let retained_records = driver.records.clone();
    let result = driver.run();
    {
        let completed_records = retained_records.borrow();
        if capped {
            let child = completed_records
                .iter()
                .filter_map(Option::as_ref)
                .find(|record| {
                    let scope = &scopes.scopes[record.scope.0];
                    scope.contents && scopes.source.nodes[scope.owner].node == 2
                })
                .unwrap();
            let deferred = &child.report.report().breaks[0];
            assert_eq!(deferred.reason, FragmentationBreakReason::Deferred);
            assert_eq!(deferred.stage, FragmentationStage::Strict);
            assert_eq!(
                (deferred.from_fragmentainer, deferred.to_fragmentainer),
                (0, 1)
            );
            assert_eq!(
                deferred.position,
                FragmentationPosition {
                    occurrence: 1,
                    offset: 0
                }
            );
            let accepted_a = child
                .report
                .report()
                .pieces
                .iter()
                .find(|piece| {
                    piece
                        .coverage
                        .iter()
                        .any(|coverage| coverage.key == SourceKey::Atomic(3))
                })
                .unwrap();
            assert_eq!(accepted_a.fragmentainer, 0);
            assert_eq!(accepted_a.block_size, S::ZERO);
            let coverage: Vec<_> = accepted_a
                .coverage
                .iter()
                .filter(|coverage| coverage.key == SourceKey::Atomic(3))
                .map(|coverage| (coverage.start, coverage.end))
                .collect();
            assert_eq!(coverage, vec![(0, 1)]);
            let timeline = child.timeline.as_ref().unwrap();
            assert_eq!(timeline.body_extent, scalar(150.0));
            assert_eq!(
                (timeline.spans[0].frame, timeline.spans[0].content),
                (0, S::ZERO)
            );
            assert_eq!(
                (timeline.spans[1].frame, timeline.spans[1].content),
                (1, scalar(150.0))
            );
        }
    }
    let resolved =
        result.unwrap_or_else(|error| panic!("Deferred after A0 capped={capped}: {error:?}"));
    let records = resolved.records.borrow();
    let root = records[resolved.root.0].as_ref().unwrap();
    assert_eq!(
        root.frames
            .iter()
            .map(|frame| frame.frame)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(root.report.report().breaks.len(), 2);
    for (index, transition) in root.report.report().breaks.iter().enumerate() {
        assert_eq!(
            (transition.from_fragmentainer, transition.to_fragmentainer),
            (index, index + 1)
        );
        assert_eq!(transition.stage, FragmentationStage::Strict);
        assert!(matches!(
            transition.reason,
            FragmentationBreakReason::Unforced | FragmentationBreakReason::Deferred
        ));
    }
    for (leaf, frame, height) in [(1, 0, 100.0), (3, 0, 0.0), (4, 1, 150.0), (5, 2, 10.0)] {
        let custody: Vec<_> = records
            .iter()
            .filter_map(Option::as_ref)
            .flat_map(|record| &record.report.report().pieces)
            .flat_map(|piece| {
                piece
                    .coverage
                    .iter()
                    .filter(move |coverage| coverage.key == SourceKey::Atomic(leaf))
                    .map(move |coverage| (piece, coverage))
            })
            .collect();
        assert_eq!(custody.len(), 1, "leaf{leaf} capped={capped}");
        assert_eq!(
            (
                custody[0].0.fragmentainer,
                custody[0].1.start,
                custody[0].1.end
            ),
            (frame, 0, 1)
        );
        // A and B are selected alone within the owning content execution.
        // In the uncapped frame0 packet prefix100 and A0 share one piece.
        if leaf == 4 || leaf == 5 {
            assert_eq!(custody[0].0.block_size, scalar(height));
        }
    }
    let tail = root
        .report
        .report()
        .pieces
        .iter()
        .find(|piece| {
            piece
                .coverage
                .iter()
                .any(|coverage| coverage.key == SourceKey::Atomic(5))
        })
        .unwrap();
    assert_eq!(tail.fragmentainer, 2);
    assert_eq!(tail.block_offset, S::ZERO);
    assert_eq!(tail.block_size, scalar(10.0));
}
#[test]
fn scope_deferred_after_zero_extent_uncapped_f32() {
    deferred_after_zero_extent::<f32>(false);
}
#[test]
fn scope_deferred_after_zero_extent_uncapped_f64() {
    deferred_after_zero_extent::<f64>(false);
}
#[test]
fn scope_deferred_after_zero_extent_capped_f32() {
    deferred_after_zero_extent::<f32>(true);
}
#[test]
fn scope_deferred_after_zero_extent_capped_f64() {
    deferred_after_zero_extent::<f64>(true);
}
