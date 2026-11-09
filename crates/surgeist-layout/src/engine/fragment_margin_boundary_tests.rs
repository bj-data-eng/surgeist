//! A real zero-height line separates flat fragment-margin contributor groups.
use super::*;
use crate::block::fragment_margins::{Contributor, MarginOwner};

fn with_packet<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    run: impl FnOnce(crate::engine::block_scope::Packet<u32, S>),
) {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let sizing = SizingRequestOf::new(
        Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    );
    let request = LayoutRootRequestOf::standalone(sizing, axes, Size::NONE);
    let bound = LayoutBoundRequestOf::continuous(LayoutSourceBasis::fresh(), request);
    let capture = LayoutFlowInput::try_new_bound(
        tree,
        0,
        bound.clone(),
        Vec::new(),
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let mut session = ComputeSession::with_flow(&capture, Vec::new());
    // The continuous native Block producer supports floats and numeric runs.
    // Retain its actual margin provenance in the same unpublished packet that
    // scope production uses, without entering the fragmentation source profile.
    session.bind_fragment_frame(bound.target_basis(), Vec::new());
    crate::engine::compute_root(&mut session, 0, request).unwrap();
    run(session.block_packet().unwrap());
}

fn box_style<S: LayoutScalar>(height: Option<f64>) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(100.0)),
            height.map_or(PreferredSizeOf::AUTO, |height| {
                PreferredSizeOf::px(S::from_f64(height))
            }),
        ),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    }
}

fn margins<S: LayoutScalar>(top: f64, bottom: f64) -> Edges<LengthAutoOf<S>> {
    Edges {
        top: LengthAutoOf::px(S::from_f64(top)),
        bottom: LengthAutoOf::px(S::from_f64(bottom)),
        ..Edges::all(LengthAutoOf::ZERO)
    }
}

fn owners(contributors: impl IntoIterator<Item = Contributor<u32>>) -> Vec<MarginOwner<u32>> {
    let mut result = Vec::new();
    for contributor in contributors {
        let owner = match contributor {
            Contributor::Edge(owner) | Contributor::Projection(_, owner) => owner,
        };
        if !result.contains(&owner) {
            result.push(owner);
        }
    }
    result
}

fn expected_edges(pairs: &[(u32, PhysicalSide)]) -> Vec<MarginOwner<u32>> {
    pairs
        .iter()
        .map(|&(node, side)| MarginOwner { node, side })
        .collect()
}

fn assert_owners(
    actual: impl IntoIterator<Item = Contributor<u32>>,
    expected: &[(u32, PhysicalSide)],
) {
    let actual = owners(actual);
    let expected = expected_edges(expected);
    assert_eq!(actual.len(), expected.len(), "distinct contributor owners");
    for owner in expected {
        assert!(actual.contains(&owner), "missing {owner:?} in {actual:?}");
    }
}

fn cleared_group_then_run<S: LayoutScalar>(forced: bool) {
    let zero_metrics =
        InlineMetricsOf::try_new(InlineBaselineContext::Physical, S::ZERO, S::ZERO).unwrap();
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2, 3, 4])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .style(0, box_style::<S>(None))
        .style(
            1,
            NodeInputOf {
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::px(S::from_f64(15.0)),
                ),
                ..box_style(None)
            },
        )
        .style(
            2,
            NodeInputOf {
                clear: Clear::Left,
                margin: margins(8.0, 12.0),
                ..box_style(None)
            },
        )
        .input(
            3,
            if forced {
                LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(zero_metrics))
            } else {
                // A real zero-valued unpaired contribution is admitted by the
                // numeric inline protocol and does not request box geometry.
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        InlineBoundaryKind::Start,
                        zero_metrics,
                        InlineBoundaryEdgeOf::ZERO,
                    )
                    .unwrap(),
                )
            },
        )
        .style(
            4,
            NodeInputOf {
                margin: margins(10.0, 0.0),
                ..box_style(None)
            },
        );
    with_packet(&tree, |packet| {
        // Execute the ordinary native Block producer, including clearance and
        // the actual inline run. The native packet owns the real Join graph;
        // this test neither assembles groups nor observes an arena identifier.
        let entry = packet.margins.entry(4).unwrap();
        let trailing = packet.margins.boundary(0, false).unwrap();
        if forced {
            // E's C7 leaves its own 8/12 through-group ending at19. A real
            // Forced line ends that group even with used height0. Z therefore
            // consumes only its own top10; its authored zero bottom still has
            // an identity in the parent's trailing contributor group.
            assert_owners(entry, &[(4, PhysicalSide::Top)]);
            assert_owners(
                trailing,
                &[(4, PhysicalSide::Top), (4, PhysicalSide::Bottom)],
            );
        } else {
            // The neutral run supplies no separating CSS line. E's margins
            // remain adjoining to Z, so the same contributor identities survive.
            assert_owners(
                entry,
                &[
                    (2, PhysicalSide::Top),
                    (2, PhysicalSide::Bottom),
                    (4, PhysicalSide::Top),
                ],
            );
            assert_owners(
                trailing,
                &[
                    (2, PhysicalSide::Top),
                    (2, PhysicalSide::Bottom),
                    (4, PhysicalSide::Top),
                    (4, PhysicalSide::Bottom),
                ],
            );
        }
    });
}

#[test]
fn forced_zero_ends_cleared_fragment_margin_group_f32() {
    cleared_group_then_run::<f32>(true);
}
#[test]
fn forced_zero_ends_cleared_fragment_margin_group_f64() {
    cleared_group_then_run::<f64>(true);
}
#[test]
fn neutral_zero_run_retains_cleared_fragment_margin_group_f32() {
    cleared_group_then_run::<f32>(false);
}
#[test]
fn neutral_zero_run_retains_cleared_fragment_margin_group_f64() {
    cleared_group_then_run::<f64>(false);
}
