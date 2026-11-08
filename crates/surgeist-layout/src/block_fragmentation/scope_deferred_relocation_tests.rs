//! Raw before-owner controls and interior avoidance retain their own sites.
use super::*;
fn relocation_control<S: LayoutScalar>(capped: bool, page_before: bool) {
    // Prefix100 consumes frame0. A break before W (Page), or a legal
    // earlier sibling break before an avoided W interior, moves W to frame1.
    // A0 accepts there; indivisible B150 overflows frame1's80 height.
    // Tail10 therefore begins at frame2 y0. No extra empty frame is needed.
    let mut owner = block::<S>(None);
    owner.break_before = if page_before {
        FragmentationBreak::Page
    } else {
        FragmentationBreak::Auto
    };
    owner.break_inside = if page_before {
        FragmentationBreakInside::Auto
    } else {
        FragmentationBreakInside::AvoidPage
    };
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
        FragmentationKind::Page {
            progression: crate::PageProgression::LeftToRight,
        },
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
        kind: FragmentationKind::Page {
            progression: crate::PageProgression::LeftToRight,
        },
        target: bound.target_basis(),
        template: request,
        intrinsics: Rc::new(RefCell::new(Vec::new())),
    };

    let resolved = Driver::new(&environment, &scopes)
        .unwrap()
        .run()
        .unwrap_or_else(|error| {
            panic!("owner relocation capped={capped} page_before={page_before}: {error:?}")
        });
    let records = resolved.records.borrow();
    let root = records[resolved.root.0].as_ref().unwrap();
    assert_eq!(
        root.frames
            .iter()
            .map(|frame| frame.frame)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    let first = &root.report.report().breaks[0];
    assert_eq!((first.from_fragmentainer, first.to_fragmentainer), (0, 1));
    assert_eq!(first.stage, FragmentationStage::Strict);
    assert_eq!(
        first.reason,
        if page_before {
            FragmentationBreakReason::Forced {
                required: FragmentationBreakKinds::PAGE,
                side: None,
            }
        } else {
            FragmentationBreakReason::Unforced
        }
    );
    for (leaf, frame) in [(1, 0), (3, 1), (4, 1), (5, 2)] {
        let custody = records
            .iter()
            .filter_map(Option::as_ref)
            .flat_map(|record| &record.report.report().pieces)
            .flat_map(|piece| {
                piece
                    .coverage
                    .iter()
                    .filter(move |coverage| coverage.key == SourceKey::Atomic(leaf))
                    .map(move |coverage| (piece.fragmentainer, coverage.start, coverage.end))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            custody,
            vec![(frame, 0, 1)],
            "leaf{leaf} capped={capped} page_before={page_before}"
        );
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
    assert_eq!(
        (tail.fragmentainer, tail.block_offset, tail.block_size),
        (2, S::ZERO, scalar(10.0))
    );
    if capped {
        let (receipt, child) = records
            .iter()
            .enumerate()
            .find_map(|(index, record)| {
                let record = record.as_ref()?;
                let scope = &scopes.scopes[record.scope.0];
                (scope.contents && scopes.source.nodes[scope.owner].node == 2)
                    .then_some((ReceiptId(index), record))
            })
            .unwrap();
        assert_eq!(
            child.placement.as_ref().unwrap().query.fragmentainer_index,
            1
        );
        assert_eq!(child.placement.as_ref().unwrap().origin.block, S::ZERO);
        assert_eq!(
            child
                .frames
                .iter()
                .map(|frame| frame.frame)
                .collect::<Vec<_>>(),
            vec![1]
        );
        assert!(!root.frames[0].receipts.contains(&receipt));
        assert_eq!(
            root.frames[1]
                .receipts
                .iter()
                .filter(|&&id| id == receipt)
                .count(),
            1
        );
        let timeline = child.timeline.as_ref().unwrap();
        assert_eq!(timeline.spans.len(), 1);
        assert_eq!(
            (timeline.spans[0].frame, timeline.spans[0].content),
            (1, scalar(150.0))
        );
    }
}
#[test]
fn scope_deferred_inside_avoid_uncapped_f32() {
    relocation_control::<f32>(false, false);
}
#[test]
fn scope_deferred_inside_avoid_uncapped_f64() {
    relocation_control::<f64>(false, false);
}
#[test]
fn scope_deferred_inside_avoid_capped_f32() {
    relocation_control::<f32>(true, false);
}
#[test]
fn scope_deferred_inside_avoid_capped_f64() {
    relocation_control::<f64>(true, false);
}
#[test]
fn scope_deferred_raw_before_page_uncapped_f32() {
    relocation_control::<f32>(false, true);
}
#[test]
fn scope_deferred_raw_before_page_uncapped_f64() {
    relocation_control::<f64>(false, true);
}
#[test]
fn scope_deferred_raw_before_page_capped_f32() {
    relocation_control::<f32>(true, true);
}
#[test]
fn scope_deferred_raw_before_page_capped_f64() {
    relocation_control::<f64>(true, true);
}
