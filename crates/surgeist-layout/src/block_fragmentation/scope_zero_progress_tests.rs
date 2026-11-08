//! Real source acceptance can precede a force without consuming principal height.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;

fn number<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn block<S: LayoutScalar>(height: Option<f64>) -> NodeInputOf<S> {
    let mut input = NodeInputOf {
        display: Display::Block,
        ..Default::default()
    };
    if let Some(height) = height {
        input.size.height = PreferredSizeOf::px(number(height));
    }
    input
}

fn forced_transitions<S: LayoutScalar>(
    report: &FragmentationReportOf<SourceKey<u32>, S>,
    count: usize,
) {
    assert_eq!(report.breaks.len(), count);
    for (index, transition) in report.breaks.iter().enumerate() {
        assert_eq!(transition.flow, 0);
        assert_eq!(transition.context, 0);
        assert_eq!(transition.from_fragmentainer, index);
        assert_eq!(transition.to_fragmentainer, index + 1);
        assert_eq!(transition.crossed, FragmentationBreakKinds::PAGE);
        assert_eq!(transition.stage, FragmentationStage::Strict);
        assert_eq!(
            transition.reason,
            FragmentationBreakReason::Forced {
                required: FragmentationBreakKinds::PAGE,
                side: None,
            }
        );
    }
}

/// Replay an uncapped accepted frame through the actual Block producer. Its
/// scheduler report contains measured body extents; the accepted allocations
/// additionally carry the box fill required by Break3 section 5.3.
fn replay_uncapped<S: LayoutScalar>(
    environment: &NativeProvider<'_, '_, PublicLayoutTreeOf<S>>,
    scopes: &Scopes<u32>,
    resolved: &work::ResolvedScopes<u32, S>,
    completed: &Completed<u32, S>,
    frame: &AcceptedFrame<u32, S>,
) -> Produced<u32, S> {
    let provider = Provider {
        environment,
        scopes,
        scope: completed.scope,
        placement: completed.placement.as_ref(),
        progress: Rc::new(RefCell::new(Progress {
            minima: Vec::new(),
            active: None,
            completed: Vec::new(),
            tails: vec![None; scopes.scopes[completed.scope.0].units.len()],
            opened: frame.frame != 0,
        })),
        dispatch: Dispatch {
            scope: resolved.root,
            view: SourceViewId(0),
            target: environment.target.clone(),
            owners: Vec::<u32>::new().into(),
            paths: Vec::<u32>::new().into(),
            ready: completed.ready.clone(),
            requests: Rc::new(RefCell::new(Requests::new())),
        },
        query: Rc::new(Cell::new(None)),
        bindings: completed.bindings.clone(),
        records: resolved.records.clone(),
        discovery: Rc::new(RefCell::new(None)),
        rebind_generation: 0,
        trial_selection: Rc::new(RefCell::new(None)),
        views: Rc::new(RefCell::new(Views::new())),
        qualifications: Rc::new(RefCell::new(Vec::new())),
        qualification_uses: Rc::new(RefCell::new(Vec::new())),
        bodies: Rc::new(RefCell::new(Vec::new())),
    };
    provider
        .trial(
            &frame.selection,
            frame.frame,
            frame.allocations.clone(),
            S::ZERO,
        )
        .expect("accepted uncapped source window replays through Block")
}

fn zero_extent_force_case<S: LayoutScalar>(repeated: bool, capped: bool) {
    // Repeated: prefix100, W(A0, B0/Page, C0/Page), tail10. The prefix
    // exhausts frame0; W has real source there but no remaining height to fill.
    // Terminal: W(A30, B0/Page), tail10. The first force fills W to100;
    // its final zero-height source child still requires the next frame.
    // Break3 section 5.3 independently gives W0/80/0 and W100/0 respectively.
    // Both totals remain below maximum200, so these are actual principal boxes.
    {
        let mut owner = block::<S>(None);
        if capped {
            owner.max_size.height = MaxSizeOf::px(number(200.0));
        }
        let mut second = block::<S>(Some(0.0));
        second.break_before = FragmentationBreak::Page;
        let mut third = block::<S>(Some(0.0));
        third.break_before = FragmentationBreak::Page;
        let tree = PublicLayoutTreeOf::new()
            .style(0, block(None))
            .style(1, block(Some(100.0)))
            .style(2, owner)
            .style(3, block(Some(if repeated { 0.0 } else { 30.0 })))
            .style(4, second)
            .style(5, third)
            .style(6, block(Some(10.0)))
            .children(0, if repeated { vec![1, 2, 6] } else { vec![2, 6] })
            .children(2, if repeated { vec![3, 4, 5] } else { vec![3, 4] });
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
                            NonNegativeFiniteOf::new(number(100.0)).unwrap(),
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
        let dimensions: &[(f64, f64)] = if repeated {
            &[(100.0, 100.0), (60.0, 80.0), (80.0, 60.0)]
        } else {
            &[(100.0, 100.0), (60.0, 80.0)]
        };
        let frames: Vec<_> = dimensions
            .iter()
            .enumerate()
            .map(|(index, &(width, height))| BlockFragmentainerOf {
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
        let inventory = Inventory::new(&tree, &capture.graph, axes).unwrap();
        let scopes = Scopes::new(inventory, &tree, axes).unwrap();
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
        let resolved = Driver::new(&environment, &scopes)
            .unwrap()
            .run()
            .unwrap_or_else(|error| {
                panic!(
                    "real zero-height force case repeated={repeated}, capped={capped}: {error:?}"
                )
            });
        let records = resolved.records.borrow();
        let principal = records[resolved.root.0].as_ref().unwrap();
        let transitions = usize::from(repeated) + 1;
        forced_transitions(principal.report.report(), transitions);
        assert_eq!(
            principal
                .frames
                .iter()
                .map(|frame| frame.frame)
                .collect::<Vec<_>>(),
            (0..=transitions).collect::<Vec<_>>()
        );
        assert_eq!(principal.report.report().fragmentainers.len(), frames.len());

        let last = transitions;
        let leaves: Vec<(u32, usize, f64)> = if repeated {
            vec![
                (1, 0, 100.0),
                (3, 0, 0.0),
                (4, 1, 0.0),
                (5, 2, 0.0),
                (6, last, 10.0),
            ]
        } else {
            vec![(3, 0, 30.0), (4, 1, 0.0), (6, last, 10.0)]
        };
        for &(leaf, frame, _) in &leaves {
            let custody: Vec<_> = records
                .iter()
                .filter_map(Option::as_ref)
                .flat_map(|record| &record.report.report().pieces)
                .flat_map(|piece| {
                    piece
                        .coverage
                        .iter()
                        .map(move |coverage| (piece.fragmentainer, coverage))
                })
                .filter(|(_, coverage)| coverage.key == SourceKey::Atomic(leaf))
                .collect();
            assert_eq!(
                custody.len(),
                1,
                "source custody for node{leaf}, capped={capped}"
            );
            assert_eq!(
                (custody[0].0, custody[0].1.start, custody[0].1.end),
                (frame, 0, 1)
            );
        }
        let heights: &[f64] = if repeated {
            &[0.0, 80.0, 0.0]
        } else {
            &[100.0, 0.0]
        };
        if capped {
            let content = records
                .iter()
                .filter_map(Option::as_ref)
                .find(|record| {
                    let scope = &scopes.scopes[record.scope.0];
                    scope.contents && scopes.source.nodes[scope.owner].node == 2
                })
                .expect("capped owner retains its authenticated content execution");
            forced_transitions(content.report.report(), transitions);
            for (index, transition) in content.report.report().breaks.iter().enumerate() {
                assert_eq!(
                    transition.position,
                    FragmentationPosition {
                        occurrence: index + 1,
                        offset: 0
                    }
                );
            }
            assert!(
                principal
                    .report
                    .report()
                    .pieces
                    .iter()
                    .flat_map(|piece| &piece.coverage)
                    .all(|coverage| !matches!(coverage.key, SourceKey::Atomic(3 | 4 | 5)))
            );
            let timeline = content.timeline.as_ref().unwrap();
            assert_eq!(
                timeline.body_extent,
                number(if repeated { 80.0 } else { 100.0 })
            );
            assert_eq!(timeline.spans.len(), heights.len());
            let mut remaining = number::<S>(200.0);
            for (frame, &height) in heights.iter().enumerate() {
                remaining = remaining - number(height);
                let span = &timeline.spans[frame];
                assert_eq!(span.frame, frame);
                assert_eq!(span.opening, S::ZERO);
                assert_eq!(span.content, number(height));
                assert_eq!(span.remaining, remaining);
                assert!(remaining > S::ZERO);
                // Select the materialization of the accepted parent window;
                // exploratory principal-cache entries are not accepted boxes.
                let parent_selection = &principal.frames[frame].selection;
                let owner = scopes.scopes[principal.scope.0].units.iter().position(|unit|
                    matches!(unit, Unit::Owner { node, .. } if scopes.source.nodes[*node].node == 2))
                    .unwrap();
                let key = ProjectionKey {
                    frame,
                    body: parent_selection.range.start <= owner,
                    closing: frame + 1 == heights.len()
                        && parent_selection.range.end > owner + 1
                        && parent_selection.partial != Some(owner),
                };
                let cached = content.principals.borrow();
                let actual: Vec<_> = cached
                    .iter()
                    .filter(|principal| principal.key == key)
                    .collect();
                assert!(
                    !actual.is_empty(),
                    "actual principal in zero-content frame{frame}"
                );
                for principal in actual {
                    assert_eq!(
                        principal.output.size,
                        Size::new(number(dimensions[frame].0), number(height))
                    );
                }
            }
            let expected_root: &[f64] = if repeated {
                &[100.0, 80.0, 10.0]
            } else {
                &[100.0, 10.0]
            };
            for (frame, &height) in expected_root.iter().enumerate() {
                let total = principal
                    .report
                    .report()
                    .pieces
                    .iter()
                    .filter(|piece| piece.fragmentainer == frame)
                    .fold(S::ZERO, |total, piece| total + piece.block_size);
                assert_eq!(total, number(height), "principal report frame{frame}");
            }
        } else {
            for (frame, &height) in heights.iter().enumerate() {
                let accepted = &principal.frames[frame];
                let replay = replay_uncapped(&environment, &scopes, &resolved, principal, accepted);
                let owner = replay
                    .output(2)
                    .expect("real accepted source retains W's actual zero box");
                assert_eq!(
                    owner.size,
                    Size::new(number(dimensions[frame].0), number(height))
                );
                assert_eq!(
                    owner.location,
                    Point::new(
                        S::ZERO,
                        number(if repeated && frame == 0 { 100.0 } else { 0.0 })
                    )
                );
                for &(leaf, leaf_frame, leaf_height) in &leaves {
                    if leaf_frame != frame {
                        continue;
                    }
                    let output = replay
                        .output(leaf)
                        .expect("accepted source leaf has Block output");
                    assert_eq!(
                        output.size,
                        Size::new(number(dimensions[frame].0), number(leaf_height))
                    );
                    assert_eq!(output.location, Point::ZERO);
                }
            }
        }
    }
}

#[test]
fn scope_zero_progress_before_repeated_forces_preserves_actual_principals_uncapped_f32() {
    zero_extent_force_case::<f32>(true, false);
}
#[test]
fn scope_zero_progress_before_repeated_forces_preserves_actual_principals_uncapped_f64() {
    zero_extent_force_case::<f64>(true, false);
}
#[test]
fn scope_zero_progress_terminal_source_child_preserves_its_forced_frame_uncapped_f32() {
    zero_extent_force_case::<f32>(false, false);
}
#[test]
fn scope_zero_progress_terminal_source_child_preserves_its_forced_frame_uncapped_f64() {
    zero_extent_force_case::<f64>(false, false);
}

#[test]
fn scope_zero_progress_before_repeated_forces_preserves_actual_principals_capped_f32() {
    zero_extent_force_case::<f32>(true, true);
}
#[test]
fn scope_zero_progress_before_repeated_forces_preserves_actual_principals_capped_f64() {
    zero_extent_force_case::<f64>(true, true);
}
#[test]
fn scope_zero_progress_terminal_source_child_preserves_its_forced_frame_capped_f32() {
    zero_extent_force_case::<f32>(false, true);
}
#[test]
fn scope_zero_progress_terminal_source_child_preserves_its_forced_frame_capped_f64() {
    zero_extent_force_case::<f64>(false, true);
}
