//! Before-owner and interior forces are distinct even at one public anchor.
use super::*;
fn raw_before_and_inside_force<S: LayoutScalar>(capped: bool) {
    // Prefix100 ends frame0. Page-before-W moves A0 to frame1; the separate
    // Page-before-B0 then moves B0 to frame2. Break3 frame fill makes W80/0,
    // with no W principal at all in frame0. Tail10 begins frame2 at0.
    let page_before = true;
    let mut inside_force = block::<S>(Some(0.0));
    inside_force.break_before = FragmentationBreak::Page;
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
        owner.max_size.height = MaxSizeOf::px(number(200.0));
    }
    let tree = PublicLayoutTreeOf::new()
        .style(0, block(None))
        .style(1, block(Some(100.0)))
        .style(2, owner)
        .style(3, block(Some(0.0)))
        .style(4, inside_force)
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
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(number(100.0)).unwrap()),
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
            size: Size::new(number::<S>(width), number(height)),
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
        .unwrap_or_else(|error| panic!("raw before W and inside force capped={capped}: {error:?}"));
    let records = resolved.records.borrow();
    let root = records[resolved.root.0].as_ref().unwrap();
    forced_transitions(root.report.report(), 2);
    assert_eq!(
        root.frames
            .iter()
            .map(|frame| frame.frame)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    for (leaf, frame) in [(1, 0), (3, 1), (4, 2), (5, 2)] {
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
        assert_eq!(custody, vec![(frame, 0, 1)], "leaf{leaf} capped={capped}");
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
    assert_eq!((tail.fragmentainer, tail.block_offset), (2, S::ZERO));
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
        let timeline = child.timeline.as_ref().unwrap();
        assert_eq!(timeline.spans.len(), 2);
        assert_eq!(
            (timeline.spans[0].frame, timeline.spans[0].content),
            (1, number(80.0))
        );
        assert_eq!(
            (timeline.spans[1].frame, timeline.spans[1].content),
            (2, S::ZERO)
        );
        assert_eq!(child.report.report().breaks.len(), 1);
        assert_eq!(
            (
                child.report.report().breaks[0].from_fragmentainer,
                child.report.report().breaks[0].to_fragmentainer
            ),
            (1, 2)
        );
        assert!(
            root.frames[0]
                .projections
                .iter()
                .all(|(node, _)| *node != 2)
        );
        assert!(!root.frames[0].receipts.contains(&receipt));
        let adopted = root.frames[2]
            .projections
            .iter()
            .filter(|(node, _)| *node == 2)
            .map(|(_, id)| *id)
            .collect::<Vec<_>>();
        assert_eq!(adopted.len(), 1);
        let cache = child.principals.borrow();
        let actual = cache
            .iter()
            .find(|entry| entry.production == adopted[0])
            .unwrap();
        assert_eq!(actual.key.window, child.window_id(2));
        let arena = resolved.productions.borrow();
        let b = arena[adopted[0].0]
            .packet
            .bodies
            .iter()
            .filter(|body| body.node == 4)
            .collect::<Vec<_>>();
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].output.size.height, S::ZERO);
    } else {
        for frame in &root.frames {
            let produced = replay_uncapped(&environment, &scopes, &resolved, root, frame);
            match frame.frame {
                0 => assert!(produced.output(2).is_none()),
                1 => assert_eq!(produced.output(2).unwrap().size.height, number(80.0)),
                2 => assert_eq!(produced.output(2).unwrap().size.height, S::ZERO),
                _ => unreachable!(),
            }
        }
    }
}
#[test]
fn scope_zero_force_sites_raw_before_and_inside_uncapped_f32() {
    raw_before_and_inside_force::<f32>(false);
}
#[test]
fn scope_zero_force_sites_raw_before_and_inside_uncapped_f64() {
    raw_before_and_inside_force::<f64>(false);
}
#[test]
fn scope_zero_force_sites_raw_before_and_inside_capped_f32() {
    raw_before_and_inside_force::<f32>(true);
}
#[test]
fn scope_zero_force_sites_raw_before_and_inside_capped_f64() {
    raw_before_and_inside_force::<f64>(true);
}
