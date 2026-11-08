//! A real start margin separates the raw owner boundary from its retained interior.
use super::*;

fn margin_window<S: LayoutScalar>(capped: bool) {
    margin_window_with_entry::<S>(capped, 90.0, 10.0);
}
fn margin_window_with_entry<S: LayoutScalar>(capped: bool, prefix: f64, margin: f64) {
    let without_prefix = prefix == 0.0;
    let opening = if without_prefix { 1.0 } else { 0.0 };
    let margin = margin - opening;
    let mut root_input = block::<S>(None);
    root_input.padding.top = LengthOf::px(scalar(opening));
    let mut owner = block::<S>(None);
    owner.margin.top = LengthAutoOf::px(scalar(margin));
    if capped {
        owner.max_size.height = MaxSizeOf::px(scalar(200.0));
    }
    let tree = PublicLayoutTreeOf::new()
        .style(0, root_input)
        .style(1, block(Some(prefix)))
        .style(2, owner)
        .style(3, block(Some(0.0)))
        .style(4, block(Some(150.0)))
        .style(5, block(Some(10.0)))
        .children(
            0,
            if without_prefix {
                vec![2, 5]
            } else {
                vec![1, 2, 5]
            },
        )
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
    };
    let driver = Driver::new(&environment, &scopes).unwrap();
    let work = &driver.stack[0];
    let replay = Provider {
        environment: &environment,
        scopes: &scopes,
        scope: work.scope,
        placement: None,
        progress: work.progress.clone(),
        dispatch: driver.dispatch(work),
        query: work.query.clone(),
        bindings: work.bindings.clone(),
        records: driver.records.clone(),
        discovery: work.discovery.clone(),
        rebind_generation: work.rebind_generation,
        trial_selection: work.trial_selection.clone(),
        views: driver.views.clone(),
        qualifications: driver.qualifications.clone(),
        qualification_uses: work.qualifications.clone(),
        productions: driver.productions.clone(),
    };
    let retained = driver.records.clone();
    let result = driver.run();
    let records = retained.borrow();
    if capped {
        let child = records
            .iter()
            .filter_map(Option::as_ref)
            .find(|record| {
                let scope = &scopes.scopes[record.scope.0];
                scope.contents && scopes.source.nodes[scope.owner].node == 2
            })
            .unwrap_or_else(|| {
                panic!(
                    "completed margin child missing: {:?}",
                    result.as_ref().err()
                )
            });
        let placement = child.placement.as_ref().unwrap();
        assert_eq!(placement.origin.block, scalar(100.0));
        // Outside is the occupied preceding prefix. Root opening belongs to
        // the owner's actual origin, not to preceding source occupancy.
        assert_eq!(placement.outside, scalar(prefix));
        let first = &child.report.report().breaks[0];
        assert_eq!(first.reason, FragmentationBreakReason::Unforced);
        assert_eq!(first.stage, FragmentationStage::Strict);
        assert_eq!(
            first.position,
            FragmentationPosition {
                occurrence: 1,
                offset: 0
            }
        );
        assert_eq!((first.from_fragmentainer, first.to_fragmentainer), (0, 1));
        assert_eq!(
            child.report.report().pieces[0].block_size,
            scalar(margin + opening)
        );
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
    let resolved =
        result.unwrap_or_else(|error| panic!("margin window capped={capped}: {error:?}"));
    let root = records[resolved.root.0].as_ref().unwrap();
    assert_eq!(
        root.frames
            .iter()
            .map(|frame| frame.frame)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    if capped && without_prefix {
        assert!(
            root.report
                .report()
                .pieces
                .iter()
                .all(|piece| piece.fragmentainer != 0)
        );
        assert_eq!(root.frames[0].selection.partial, Some(0));
        assert_eq!(root.frames[0].receipts.len(), 1);
    }
    // Replay the actual accepted parent selections/allocations through Block.
    // This observes real W boxes, independently of scheduler source reports.
    for (frame, height, y) in [(0, 0.0, 100.0), (1, 150.0, 0.0)] {
        let accepted = &root.frames[frame];
        let actual = replay
            .trial(
                &accepted.selection,
                frame,
                accepted.allocations.clone(),
                S::ZERO,
            )
            .expect("accepted margin window replays through Block");
        let owner = actual.output(2).expect("actual W principal retained");
        assert_eq!(owner.size.height, scalar(height));
        assert_eq!(owner.location.y, scalar(y));
    }
    for (leaf, expected_frame) in [(1, 0), (3, 0), (4, 1), (5, 2)] {
        if without_prefix && leaf == 1 {
            continue;
        }
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
            (expected_frame, 0, 1)
        );
        if leaf == 5 {
            assert_eq!(custody[0].0.block_offset, S::ZERO);
        }
    }
}
#[test]
fn scope_margin_zero_principal_window_uncapped_f32() {
    margin_window::<f32>(false);
}
#[test]
fn scope_margin_zero_principal_window_uncapped_f64() {
    margin_window::<f64>(false);
}
#[test]
fn scope_margin_zero_principal_window_capped_f32() {
    margin_window::<f32>(true);
}
#[test]
fn scope_margin_zero_principal_window_capped_f64() {
    margin_window::<f64>(true);
}

// Root padding1 prevents margin collapse. W margin99 places A0 at100; the
// current owner has no ordinary source prefix. Its actual accepted child window
// supplies source progress alongside that physical contribution.
#[test]
fn scope_margin_current_owner_uncapped_f32() {
    margin_window_with_entry::<f32>(false, 0.0, 100.0);
}
#[test]
fn scope_margin_current_owner_uncapped_f64() {
    margin_window_with_entry::<f64>(false, 0.0, 100.0);
}
#[test]
fn scope_margin_current_owner_capped_f32() {
    margin_window_with_entry::<f32>(true, 0.0, 100.0);
}
#[test]
fn scope_margin_current_owner_capped_f64() {
    margin_window_with_entry::<f64>(true, 0.0, 100.0);
}
