//! Assembly of already accepted, paired native appearances. This layer never
//! schedules, measures, rounds, or obtains source facts from a host.

use super::output::{NodePublication, ResultIdentity};
use super::source::Inventory;
use crate::output::CompletedBatchEntries;
use crate::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct AppearanceIndex(pub(super) usize);

pub(super) struct PreparedAppearance<Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) frame: usize,
    pub(super) parent: Option<AppearanceIndex>,
    pub(super) payload: FragmentPayloadOf<Node, S>,
    pub(super) source_children: Vec<(SourceIndex, Node)>,
    pub(super) graphical: Option<FragmentationGraphicalCoverageOf<S>>,
}

pub(super) struct PreparedMetadata<Node, S: LayoutScalar> {
    pub(super) kind: NonBoxKind,
    pub(super) unrounded: LayoutOutputEntryOf<Node, S>,
    pub(super) final_entry: LayoutOutputEntryOf<Node, S>,
}

/// The exact-key driver validates source/constraint/placement binding before
/// constructing this input. Pairs already retain the shared rounding owner's
/// actual anchors and phase custody; assembly only validates and moves them.
/// `inventory` is the trusted canonical capture issued by `Inventory::new`
/// from the validated source graph. Its identity/topology is not raw assembly
/// input; this immutable borrow never reinterprets or rebuilds that snapshot.
pub(super) struct AssemblyInput<'a, Node, S: LayoutScalar> {
    pub(super) root: Node,
    pub(super) inventory: &'a Inventory<Node>,
    pub(super) appearances: Vec<PreparedAppearance<Node, S>>,
    pub(super) metadata: Vec<PreparedMetadata<Node, S>>,
    pub(super) frames: &'a [BlockFragmentainerOf<S>],
    pub(super) axes: FlowAxes,
    pub(super) kind: FragmentationKind,
    pub(super) target: LayoutTargetBasis,
    pub(super) invalidated: Vec<Node>,
}

fn failure<Node, S: LayoutScalar, M>(
    node: Node,
    kind: BlockFragmentationFailure,
) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RootLayout,
        LayoutErrorKindOf::BlockFragmentation(kind),
    )
}
fn allocation<Node, S: LayoutScalar, M>(node: Node) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RootLayout,
        LayoutErrorKindOf::Allocation,
    )
}
fn packet_geometry<S: LayoutScalar>(packet: NodeOutputOf<S>) -> bool {
    [
        packet.location.x,
        packet.location.y,
        packet.margin.left,
        packet.margin.right,
        packet.margin.top,
        packet.margin.bottom,
    ]
    .into_iter()
    .all(LayoutScalar::is_finite)
        && [
            packet.size.width,
            packet.size.height,
            packet.content_size.width,
            packet.content_size.height,
            packet.padding.left,
            packet.padding.right,
            packet.padding.top,
            packet.padding.bottom,
            packet.border.left,
            packet.border.right,
            packet.border.top,
            packet.border.bottom,
        ]
        .into_iter()
        .all(|value| value.is_finite() && value >= S::ZERO)
}
fn packet_pair<Node: Copy + Eq, S: LayoutScalar>(
    node: Node,
    source: SourceIndex,
    unrounded: LayoutOutputEntryOf<Node, S>,
    final_entry: LayoutOutputEntryOf<Node, S>,
    kind: Option<NonBoxKind>,
) -> Result<(), BlockFragmentationFailure> {
    use crate::output::{AtomicReadbackPhase, GeometryStateOf};
    if unrounded.node() != node
        || final_entry.node() != node
        || unrounded.output().source_index != source
        || final_entry.output().source_index != source
        || unrounded.non_box != kind
        || final_entry.non_box != kind
    {
        return Err(BlockFragmentationFailure::InvalidSource);
    }
    let paired_phase = match (unrounded.geometry_state(), final_entry.geometry_state()) {
        (GeometryStateOf::OrdinaryUnrounded { .. }, GeometryStateOf::OrdinaryCanonical { .. }) => {
            true
        }
        (
            GeometryStateOf::CanonicalAtomic {
                measured_normal: a,
                measured_padding_origin: pad_a,
                normal: b,
                normal_origin: c,
                visual_origin: d,
                readback: AtomicReadbackPhase::Pending,
            },
            GeometryStateOf::CanonicalAtomic {
                measured_normal: e,
                measured_padding_origin: pad_b,
                normal: f,
                normal_origin: g,
                visual_origin: h,
                readback: AtomicReadbackPhase::Finalized,
            },
        ) => {
            a == e
                && pad_a == pad_b
                && b == f
                && c == g
                && d == h
                && [c.x, c.y, d.x, d.y].into_iter().all(f64::is_finite)
        }
        _ => false,
    };
    if !paired_phase
        || !packet_geometry(unrounded.output())
        || !packet_geometry(final_entry.output())
    {
        return Err(BlockFragmentationFailure::InvalidGeometry);
    }
    Ok(())
}

pub(super) fn assemble<Node: Copy + Eq, S: LayoutScalar, M>(
    input: AssemblyInput<'_, Node, S>,
) -> LayoutResultOf<Node, CompletedLayoutBatchOf<Node, S>, S, M> {
    let AssemblyInput {
        root,
        inventory,
        appearances,
        metadata,
        frames,
        axes,
        kind,
        target,
        invalidated,
    } = input;
    if inventory
        .nodes
        .first()
        .is_none_or(|source| source.node != root || source.parent.is_some())
    {
        return Err(failure(root, BlockFragmentationFailure::InvalidSource));
    }
    let allowed = if matches!(kind, FragmentationKind::Page { .. }) {
        FragmentationBreakKinds::SPREAD
    } else {
        kind.breaks()
    };
    if frames.is_empty()
        || frames.iter().enumerate().any(|(index, frame)| {
            !frame.origin.x.is_finite()
                || !frame.origin.y.is_finite()
                || ![frame.size.width, frame.size.height]
                    .into_iter()
                    .all(|value| value.is_finite() && value >= S::ZERO)
                || !allowed.contains(frame.breaks)
                || (index != 0 && frame.breaks == FragmentationBreakKinds::NONE)
        })
    {
        return Err(failure(root, BlockFragmentationFailure::InvalidGeometry));
    }
    for (index, &node) in invalidated.iter().enumerate() {
        if invalidated[..index].contains(&node)
            || !inventory.nodes.iter().any(|source| source.node == node)
        {
            return Err(failure(node, BlockFragmentationFailure::InvalidSource));
        }
    }
    let mut by_source: Vec<Vec<usize>> = Vec::new();
    by_source
        .try_reserve(inventory.nodes.len())
        .map_err(|_| allocation(root))?;
    by_source.resize_with(inventory.nodes.len(), Vec::new);
    let mut appearance_sources = Vec::new();
    appearance_sources
        .try_reserve(appearances.len())
        .map_err(|_| allocation(root))?;
    for (index, appearance) in appearances.iter().enumerate() {
        let source_index = inventory
            .nodes
            .iter()
            .position(|source| source.node == appearance.node)
            .ok_or_else(|| failure(appearance.node, BlockFragmentationFailure::InvalidSource))?;
        let source = &inventory.nodes[source_index];
        if source.hidden
            || appearance.frame >= frames.len()
            || by_source[source_index]
                .iter()
                .any(|&other| appearances[other].frame == appearance.frame)
        {
            return Err(failure(
                appearance.node,
                BlockFragmentationFailure::InvalidSource,
            ));
        }
        match &appearance.payload {
            FragmentPayloadOf::ActualBox(pair) => packet_pair(
                appearance.node,
                source.source_index,
                pair.unrounded,
                pair.final_entry,
                None,
            )
            .map_err(|error| failure(appearance.node, error))?,
            FragmentPayloadOf::OverflowOnly(pair) => {
                if [pair.unrounded.origin, pair.final_geometry.origin]
                    .into_iter()
                    .any(|origin| !origin.x.is_finite() || !origin.y.is_finite())
                {
                    return Err(failure(
                        appearance.node,
                        BlockFragmentationFailure::InvalidGeometry,
                    ));
                }
            }
        }
        if appearance.graphical.is_some_and(|coverage| {
            ![coverage.start, coverage.end, coverage.extent]
                .into_iter()
                .all(LayoutScalar::is_finite)
                || coverage.start < S::ZERO
                || coverage.end < coverage.start
                || coverage.extent < coverage.end
        }) {
            return Err(failure(
                appearance.node,
                BlockFragmentationFailure::InvalidGeometry,
            ));
        }
        by_source[source_index]
            .try_reserve(1)
            .map_err(|_| allocation(root))?;
        by_source[source_index].push(index);
        appearance_sources.push(source_index);
    }
    let mut metadata_sources = Vec::new();
    metadata_sources
        .try_reserve(metadata.len())
        .map_err(|_| allocation(root))?;
    let mut metadata_by_source: Vec<Option<usize>> = Vec::new();
    metadata_by_source
        .try_reserve(inventory.nodes.len())
        .map_err(|_| allocation(root))?;
    metadata_by_source.resize(inventory.nodes.len(), None);
    for (index, entry) in metadata.iter().enumerate() {
        let node = entry.unrounded.node();
        let source_index = inventory
            .nodes
            .iter()
            .position(|source| source.node == node)
            .ok_or_else(|| failure(node, BlockFragmentationFailure::InvalidSource))?;
        let source = &inventory.nodes[source_index];
        if !by_source[source_index].is_empty()
            || metadata_by_source[source_index].is_some()
            || (entry.kind == NonBoxKind::Hidden) != source.hidden
        {
            return Err(failure(node, BlockFragmentationFailure::InvalidSource));
        }
        packet_pair(
            node,
            source.source_index,
            entry.unrounded,
            entry.final_entry,
            Some(entry.kind),
        )
        .map_err(|error| failure(node, error))?;
        metadata_by_source[source_index] = Some(index);
        metadata_sources.push(source_index);
    }
    if by_source[0].is_empty() && metadata_by_source[0].is_none() {
        return Err(failure(root, BlockFragmentationFailure::InvalidSource));
    }
    for (index, appearance) in appearances.iter().enumerate() {
        let source = &inventory.nodes[appearance_sources[index]];
        match (source.parent, appearance.parent) {
            (None, None) => {}
            (Some(parent_source), Some(AppearanceIndex(parent))) => {
                let actual = appearances.get(parent).ok_or_else(|| {
                    failure(appearance.node, BlockFragmentationFailure::InvalidSource)
                })?;
                if parent == index
                    || actual.node != inventory.nodes[parent_source].node
                    || actual.frame != appearance.frame
                    || !actual
                        .source_children
                        .contains(&(source.source_index, appearance.node))
                {
                    return Err(failure(
                        appearance.node,
                        BlockFragmentationFailure::InvalidSource,
                    ));
                }
            }
            _ => {
                return Err(failure(
                    appearance.node,
                    BlockFragmentationFailure::InvalidSource,
                ));
            }
        }
        let mut previous = None;
        for &(order, child) in &appearance.source_children {
            let child_source = source.children.get(order.get()).copied().ok_or_else(|| {
                failure(appearance.node, BlockFragmentationFailure::InvalidSource)
            })?;
            if inventory.nodes[child_source].node != child
                || inventory.nodes[child_source].source_index != order
                || previous.is_some_and(|old| old >= order.get())
                || (metadata_by_source[child_source].is_none()
                    && !by_source[child_source].iter().any(|&child_index| {
                        appearances[child_index].frame == appearance.frame
                            && appearances[child_index].parent == Some(AppearanceIndex(index))
                    }))
            {
                return Err(failure(
                    appearance.node,
                    BlockFragmentationFailure::InvalidSource,
                ));
            }
            previous = Some(order.get());
        }
    }

    let mut appearance_orders = Vec::new();
    appearance_orders
        .try_reserve(appearances.len())
        .map_err(|_| allocation(root))?;
    appearance_orders.resize(appearances.len(), 0);
    for family in &mut by_source {
        family.sort_unstable_by_key(|&index| appearances[index].frame);
        for (order, &index) in family.iter().enumerate() {
            appearance_orders[index] = order;
        }
    }
    // Binding/phase checks precede construction. Canonical continuation order
    // follows logical frame indices; opaque IDs retain global appearance indices.
    let result = Arc::new(ResultIdentity);
    let id = |index| LayoutFragmentId {
        result: result.clone(),
        index,
    };
    let mut fragments: Vec<Vec<FragmentOutputOf<Node, S>>> = Vec::new();
    fragments
        .try_reserve(inventory.nodes.len())
        .map_err(|_| allocation(root))?;
    for indices in &by_source {
        let mut group = Vec::new();
        group
            .try_reserve(indices.len())
            .map_err(|_| allocation(root))?;
        fragments.push(group);
    }
    for (index, appearance) in appearances.into_iter().enumerate() {
        let source_index = appearance_sources[index];
        let source = &inventory.nodes[source_index];
        let order = appearance_orders[index];
        let indices = &by_source[source_index];
        fragments[source_index].push(FragmentOutputOf {
            id: id(index),
            node: appearance.node,
            role: source.role,
            frame: Some(LayoutFragmentainerId {
                result: result.clone(),
                index: appearance.frame,
            }),
            parent: Some(appearance.parent.map_or_else(
                || {
                    FragmentCoordinateSpace::Fragmentainer(LayoutFragmentainerId {
                        result: result.clone(),
                        index: appearance.frame,
                    })
                },
                |AppearanceIndex(parent)| FragmentCoordinateSpace::Fragment(id(parent)),
            )),
            order,
            previous: order.checked_sub(1).map(|previous| id(indices[previous])),
            next: indices.get(order + 1).map(|&next| id(next)),
            children: appearance.source_children,
            graphical: appearance.graphical,
            payload: appearance.payload,
        });
    }
    let mut owned_metadata: Vec<Option<PreparedMetadata<Node, S>>> = Vec::new();
    owned_metadata
        .try_reserve(inventory.nodes.len())
        .map_err(|_| allocation(root))?;
    owned_metadata.resize_with(inventory.nodes.len(), || None);
    for (entry, source) in metadata.into_iter().zip(metadata_sources) {
        owned_metadata[source] = Some(entry);
    }
    let mut nodes = Vec::new();
    nodes
        .try_reserve(inventory.nodes.len())
        .map_err(|_| allocation(root))?;
    for (index, source) in inventory.nodes.iter().enumerate() {
        let mut group = std::mem::take(&mut fragments[index]);
        group.sort_unstable_by_key(|fragment| fragment.order);
        match group.len() {
            0 => {
                if let Some(entry) = owned_metadata[index].take() {
                    nodes.push(NodePublication::NonBox {
                        kind: entry.kind,
                        unrounded: entry.unrounded,
                        final_entry: entry.final_entry,
                    });
                }
            }
            1 => nodes.push(NodePublication::One(
                group.pop().expect("the group has one accepted occurrence"),
            )),
            _ => nodes.push(NodePublication::Fragments {
                node: source.node,
                fragments: group,
            }),
        }
    }
    let mut fragmentainers = Vec::new();
    fragmentainers
        .try_reserve(frames.len())
        .map_err(|_| allocation(root))?;
    fragmentainers.extend(frames.iter().enumerate().map(|(index, &frame)| {
        LayoutFragmentainerOutputOf {
            id: LayoutFragmentainerId {
                result: result.clone(),
                index,
            },
            kind,
            axes,
            frame,
        }
    }));
    let mut clears = Vec::new();
    clears
        .try_reserve(invalidated.len())
        .map_err(|_| allocation(root))?;
    clears.extend(invalidated.iter().copied().map(LayoutCacheClearEntry::new));
    let mut batch = CompletedLayoutBatchOf::from_entries(
        root,
        CompletedBatchEntries {
            unrounded: Vec::new(),
            final_entries: Vec::new(),
            unrounded_inline: Vec::new(),
            final_inline: Vec::new(),
            stores: Vec::new(),
            clears,
            invalidated,
        },
    );
    batch.result = result;
    batch.nodes = nodes;
    batch.fragmentainers = fragmentainers;
    batch.target = Some(target);
    Ok(batch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::NormalGeometryOf;
    use crate::test_support::layout_tree::PublicLayoutTreeOf;
    use std::cell::Cell;

    fn n<S: LayoutScalar>(value: f64) -> S {
        S::from_f64(value)
    }
    fn point<S: LayoutScalar>(x: f64, y: f64) -> Point<S> {
        Point::new(n(x), n(y))
    }
    fn bounds<S: LayoutScalar>(x: f64, y: f64, w: f64, h: f64) -> ScrollRectOf<S> {
        ScrollRectOf::try_new(point(x, y), Size::new(n(w), n(h))).unwrap()
    }
    fn axes() -> FlowAxes {
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
    }
    fn captured<S: LayoutScalar>(children: bool) -> Inventory<u32> {
        let block = || NodeInputOf::<S> {
            display: Display::Block,
            ..Default::default()
        };
        let mut tree = PublicLayoutTreeOf::new().style(0, block());
        if children {
            tree = tree
                .style(
                    1,
                    NodeInputOf {
                        block_generation: BlockGeneration::Anonymous,
                        ..block()
                    },
                )
                .style(2, block())
                .style(
                    3,
                    NodeInputOf {
                        display: Display::None,
                        ..block()
                    },
                )
                .children(0, [1, 3])
                .children(1, [2]);
        }
        let (graph, _) =
            crate::engine::validation::validate_layout_request_with_text_graph(&tree, 0, &[], &[])
                .unwrap();
        Inventory::new(&tree, &graph, axes()).unwrap()
    }
    fn actual<S: LayoutScalar>(node: u32, width: f64, height: f64) -> BoxGeometryPairOf<u32, S> {
        let unrounded = NodeOutputOf {
            location: point(0.25, 0.75),
            size: Size::new(n(width), n(height)),
            content_size: Size::new(n(width.max(32.0)), n(height.max(27.0))),
            ..NodeOutputOf::new()
        };
        let normal = NormalGeometryOf::box_geometry(
            unrounded.location,
            unrounded.size,
            bounds(0.0, 0.0, width, height),
        );
        let final_output = NodeOutputOf {
            location: point(0.0, 1.0),
            ..unrounded
        };
        BoxGeometryPairOf {
            unrounded: LayoutOutputEntryOf::ordinary_unrounded(node, normal, unrounded),
            final_entry: LayoutOutputEntryOf::ordinary_canonical(node, normal, final_output),
        }
    }
    fn overflow<S: LayoutScalar>(x: f64, y: f64) -> FragmentPayloadOf<u32, S> {
        FragmentPayloadOf::OverflowOnly(OverflowGeometryPairOf {
            unrounded: OverflowFragmentGeometryOf {
                origin: point(x, y),
                content_bounds: bounds(-2.25, 1.5, 32.5, 27.25),
            },
            final_geometry: OverflowFragmentGeometryOf {
                origin: point(x.round(), y.round()),
                content_bounds: bounds(-2.0, 2.0, 32.0, 27.0),
            },
        })
    }
    fn frames<S: LayoutScalar>() -> Vec<BlockFragmentainerOf<S>> {
        [(10.0, 20.0, 100.0, 100.0), (200.5, 100.25, 60.0, 80.0)]
            .into_iter()
            .enumerate()
            .map(|(index, (x, y, w, h))| BlockFragmentainerOf {
                origin: point(x, y),
                size: Size::new(n(w), n(h)),
                breaks: if index == 0 {
                    FragmentationBreakKinds::NONE
                } else {
                    FragmentationBreakKinds::PAGE
                },
                page_side: None,
            })
            .collect()
    }
    fn target<S: LayoutScalar>(frames: &[BlockFragmentainerOf<S>]) -> LayoutTargetBasis {
        let sizing = SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::Normal,
            )),
            OrthogonalFallbackOf::new(
                Size::NONE,
                NearestScrollportOf::Unknown,
                Size::splat(OrthogonalFallbackLimitOf::Unknown),
            ),
        );
        LayoutBoundRequestOf::block_fragmentation(
            LayoutSourceBasis::fresh(),
            LayoutRootRequestOf::standalone(sizing, axes(), Size::NONE),
            FragmentationKind::Page {
                progression: crate::PageProgression::LeftToRight,
            },
            axes(),
            frames.to_vec(),
        )
        .unwrap()
        .target_basis()
    }
    struct Fixture<S: LayoutScalar> {
        inventory: Inventory<u32>,
        appearances: Vec<PreparedAppearance<u32, S>>,
        metadata: Vec<PreparedMetadata<u32, S>>,
        frames: Vec<BlockFragmentainerOf<S>>,
    }
    fn fixture<S: LayoutScalar>() -> Fixture<S> {
        let appearance = |node, frame, parent, payload, source_children| PreparedAppearance {
            node,
            frame,
            parent,
            payload,
            source_children,
            graphical: None,
        };
        let mut hidden = LayoutOutputEntryOf::hidden_unrounded(3, SourceIndex::new(1));
        hidden.non_box = Some(NonBoxKind::Hidden);
        let mut final_hidden = LayoutOutputEntryOf::ordinary_canonical(
            3,
            NormalGeometryOf {
                location: Point::ZERO,
                size: Size::ZERO,
                content: crate::output::NormalContentOf::Hidden,
            },
            hidden.output(),
        );
        final_hidden.non_box = Some(NonBoxKind::Hidden);
        Fixture {
            inventory: captured::<S>(true),
            frames: frames(),
            // Forward parent references are deliberate: IDs follow actual
            // appearance identity, without sorting coordinates or host nodes.
            appearances: vec![
                appearance(
                    2,
                    1,
                    Some(AppearanceIndex(4)),
                    FragmentPayloadOf::ActualBox(actual(2, 0.0, 0.0)),
                    Vec::new(),
                ),
                PreparedAppearance {
                    graphical: Some(FragmentationGraphicalCoverageOf {
                        start: S::ZERO,
                        end: n(50.0),
                        extent: n(50.0),
                    }),
                    ..appearance(
                        0,
                        0,
                        None,
                        FragmentPayloadOf::ActualBox(actual(0, 100.0, 50.0)),
                        vec![(SourceIndex::ZERO, 1), (SourceIndex::new(1), 3)],
                    )
                },
                appearance(
                    1,
                    0,
                    Some(AppearanceIndex(1)),
                    FragmentPayloadOf::ActualBox(actual(1, 100.0, 30.0)),
                    Vec::new(),
                ),
                appearance(
                    0,
                    1,
                    None,
                    overflow(7.25, 3.75),
                    vec![(SourceIndex::ZERO, 1)],
                ),
                appearance(
                    1,
                    1,
                    Some(AppearanceIndex(3)),
                    overflow(4.5, 6.25),
                    vec![(SourceIndex::ZERO, 2)],
                ),
            ],
            metadata: vec![PreparedMetadata {
                kind: NonBoxKind::Hidden,
                unrounded: hidden,
                final_entry: final_hidden,
            }],
        }
    }
    fn run<S: LayoutScalar>(
        fixture: Fixture<S>,
    ) -> LayoutResultOf<u32, CompletedLayoutBatchOf<u32, S>, S> {
        let invalidated = fixture
            .inventory
            .nodes
            .iter()
            .map(|source| source.node)
            .collect();
        assemble(AssemblyInput {
            root: 0,
            inventory: &fixture.inventory,
            appearances: fixture.appearances,
            metadata: fixture.metadata,
            frames: &fixture.frames,
            axes: axes(),
            kind: FragmentationKind::Page {
                progression: crate::PageProgression::LeftToRight,
            },
            target: target(&fixture.frames),
            invalidated,
        })
    }
    fn mixed_graph<S: LayoutScalar>() {
        let batch = run(fixture::<S>()).unwrap();
        assert!(batch.replaces_whole_root());
        assert_eq!(
            batch
                .node_layouts()
                .map(|(node, _)| node)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(batch.fragments().count(), 5);
        assert_eq!(
            batch
                .cache_clear_entries()
                .iter()
                .map(LayoutCacheClearEntry::node)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert!(batch.cache_store_entries().is_empty());
        let roots = batch.node_layout(0).unwrap().fragments().unwrap();
        assert_eq!(roots.len(), 2);
        assert!(batch.node_layout(0).unwrap().single_box().is_none());
        assert_eq!(roots[0].next(), Some(roots[1].id()));
        assert_eq!(roots[1].previous(), Some(roots[0].id()));
        assert_eq!(roots[1].continuation_order(), 1);
        assert_eq!(
            roots[0].graphical_coverage(),
            Some(FragmentationGraphicalCoverageOf {
                start: S::ZERO,
                end: n(50.0),
                extent: n(50.0)
            })
        );
        let carrier = &roots[1];
        assert!(carrier.principal_box().is_none());
        assert_eq!(
            carrier.coordinate_parent(),
            Some(&FragmentCoordinateSpace::Fragmentainer(
                batch.fragmentainers()[1].id().clone()
            ))
        );
        assert_eq!(
            batch
                .fragmentainer(carrier.fragmentainer().unwrap())
                .unwrap()
                .origin(),
            point(200.5, 100.25)
        );
        let overflow = carrier.overflow_only().unwrap();
        assert_eq!(overflow.unrounded().origin, point(7.25, 3.75));
        assert_eq!(overflow.final_geometry().origin, point(7.0, 4.0));
        assert_eq!(
            overflow.unrounded().content_bounds,
            bounds(-2.25, 1.5, 32.5, 27.25)
        );
        let owners: Vec<_> = batch.child_fragments(carrier.id()).unwrap().collect();
        assert_eq!(owners.len(), 1);
        assert_eq!(owners[0].node(), 1);
        assert_eq!(owners[0].generated_role(), BlockGeneration::Anonymous);
        assert!(owners[0].overflow_only().is_some());
        assert_eq!(owners[0].fragmentainer(), carrier.fragmentainer());
        let leaves: Vec<_> = batch.child_fragments(owners[0].id()).unwrap().collect();
        assert_eq!(leaves.len(), 1);
        assert_eq!(leaves[0].node(), 2);
        assert!(leaves[0].overflow_only().is_none());
        assert_eq!(
            leaves[0].principal_box().unwrap().unrounded().size,
            Size::ZERO
        );
        assert_eq!(
            leaves[0].principal_box().unwrap().final_geometry().size,
            Size::ZERO
        );
        assert_eq!(
            batch.node_layout(3).unwrap().non_box().unwrap().kind(),
            NonBoxKind::Hidden
        );
        assert_eq!(
            roots[0].source_children(),
            &[(SourceIndex::ZERO, 1), (SourceIndex::new(1), 3)]
        );
        let all: Vec<_> = batch.fragments().collect();
        for (index, fragment) in all.iter().enumerate() {
            assert!(!all[..index].iter().any(|other| other.id() == fragment.id()));
        }
        let another = run(fixture::<S>()).unwrap();
        assert!(matches!(
            another.fragment(carrier.id()),
            Err(LayoutHandleError::WrongResult)
        ));
    }
    fn unsorted_family_keeps_forward_parent_identity<S: LayoutScalar>() {
        let mut fixture = fixture::<S>();
        fixture.appearances.swap(1, 3);
        for appearance in &mut fixture.appearances {
            appearance.parent = appearance.parent.map(|AppearanceIndex(index)| {
                AppearanceIndex(match index {
                    1 => 3,
                    3 => 1,
                    other => other,
                })
            });
        }
        let batch = run(fixture).unwrap();
        let roots = batch.node_layout(0).unwrap().fragments().unwrap();
        assert_eq!(
            roots[0].fragmentainer(),
            Some(batch.fragmentainers()[0].id())
        );
        assert_eq!(
            roots[1].fragmentainer(),
            Some(batch.fragmentainers()[1].id())
        );
        assert_eq!(roots[0].continuation_order(), 0);
        assert_eq!(roots[0].previous(), None);
        assert_eq!(roots[0].next(), Some(roots[1].id()));
        assert_eq!(roots[1].previous(), Some(roots[0].id()));
        assert_eq!(roots[1].continuation_order(), 1);
        for (index, root) in roots.iter().enumerate() {
            let children: Vec<_> = batch.child_fragments(root.id()).unwrap().collect();
            assert_eq!(children.len(), 1);
            assert_eq!(children[0].node(), 1);
            assert_eq!(
                children[0].fragmentainer(),
                Some(batch.fragmentainers()[index].id())
            );
            let parent = match children[0].coordinate_parent().unwrap() {
                FragmentCoordinateSpace::Fragment(id) => id,
                _ => panic!("the accepted owner retains its actual root parent"),
            };
            assert_eq!(batch.fragment(parent).unwrap().id(), root.id());
        }
    }
    fn opaque_custody<S: LayoutScalar>() {
        let mut fixture = fixture::<S>();
        let packet = NodeOutputOf::<S>::new();
        let normal =
            NormalGeometryOf::box_geometry(Point::ZERO, Size::ZERO, bounds(0.0, 0.0, 0.0, 0.0));
        let unrounded = LayoutOutputEntryOf::canonical_atomic(
            2,
            normal,
            Point::ZERO,
            normal,
            Point::new(1_000_000_000.25, -1_000_000_000.75),
            Point::new(1_000_000_000.5, -1_000_000_000.25),
            packet,
        );
        let final_entry = unrounded.finalized_atomic(packet).unwrap();
        fixture.appearances[0].payload = FragmentPayloadOf::ActualBox(BoxGeometryPairOf {
            unrounded,
            final_entry,
        });
        let batch = run(fixture).unwrap();
        let single = batch.node_layout(2).unwrap().single_box().unwrap();
        assert_eq!(single.unrounded_entry(), unrounded);
        assert_eq!(single.final_entry(), final_entry);
    }
    fn invalid_graph<S: LayoutScalar>() {
        for case in 0..7 {
            let mut fixture = fixture::<S>();
            match case {
                0 => fixture.appearances[0].parent = Some(AppearanceIndex(99)),
                1 => fixture.appearances[0].parent = Some(AppearanceIndex(3)),
                2 => fixture.appearances[0].parent = Some(AppearanceIndex(2)),
                3 => fixture.appearances[3].source_children.clear(),
                4 => fixture.appearances[4].source_children = vec![(SourceIndex::new(1), 2)],
                5 => fixture.appearances.push(PreparedAppearance {
                    node: 0,
                    frame: 1,
                    parent: None,
                    payload: overflow(7.25, 3.75),
                    source_children: Vec::new(),
                    graphical: None,
                }),
                _ => fixture.appearances[1].parent = Some(AppearanceIndex(4)),
            }
            let error = run(fixture).unwrap_err();
            assert!(
                matches!(
                    error.kind(),
                    LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidSource)
                ),
                "case {case}: {error:?}"
            );
        }
    }
    fn invalid_phase<S: LayoutScalar>() {
        let mut fixture = fixture::<S>();
        if let FragmentPayloadOf::ActualBox(pair) = &mut fixture.appearances[0].payload {
            pair.final_entry = pair.unrounded;
        }
        assert!(matches!(
            run(fixture).unwrap_err().kind(),
            LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidGeometry)
        ));
        let mut fixture = self::fixture::<S>();
        if let FragmentPayloadOf::ActualBox(pair) = &mut fixture.appearances[0].payload {
            let normal =
                NormalGeometryOf::box_geometry(Point::ZERO, Size::ZERO, bounds(0.0, 0.0, 0.0, 0.0));
            pair.unrounded = LayoutOutputEntryOf::canonical_atomic(
                2,
                normal,
                Point::ZERO,
                normal,
                Point::new(0.25, 0.75),
                Point::new(0.25, 0.75),
                NodeOutputOf::new(),
            );
        }
        assert!(matches!(
            run(fixture).unwrap_err().kind(),
            LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidGeometry)
        ));
        let mut fixture = self::fixture::<S>();
        if let FragmentPayloadOf::ActualBox(pair) = &mut fixture.appearances[0].payload {
            pair.unrounded = pair.unrounded.with_source_index(SourceIndex::new(1));
        }
        assert!(matches!(
            run(fixture).unwrap_err().kind(),
            LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidSource)
        ));
        let mut fixture = self::fixture::<S>();
        if let FragmentPayloadOf::OverflowOnly(pair) = &mut fixture.appearances[3].payload {
            pair.final_geometry.origin.x = S::from_f64(f64::NAN);
        }
        assert!(matches!(
            run(fixture).unwrap_err().kind(),
            LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidGeometry)
        ));
    }

    struct Sink<S: LayoutScalar> {
        targets: LayoutPublicationTargets<u32>,
        visible: Vec<FragmentOutputOf<u32, S>>,
        reject: Cell<bool>,
        commits: usize,
    }
    impl<S: LayoutScalar> LayoutBatchSink<u32, S> for Sink<S> {
        type Error = &'static str;
        type Prepared = Vec<FragmentOutputOf<u32, S>>;
        fn publication_targets(&self) -> &LayoutPublicationTargets<u32> {
            &self.targets
        }
        fn publication_targets_mut(&mut self) -> &mut LayoutPublicationTargets<u32> {
            &mut self.targets
        }
        fn prepare_layout_batch(
            &self,
            preparation: LayoutBatchPreparation<'_, u32, S>,
        ) -> Result<Self::Prepared, Self::Error> {
            let mut next = self.visible.clone();
            next.retain(|fragment| !preparation.previous_members().contains(&fragment.node()));
            next.extend(preparation.batch().fragments().cloned());
            if self.reject.get() {
                Err("rejected")
            } else {
                Ok(next)
            }
        }
        fn commit_layout_batch(&mut self, commit: LayoutBatchCommit<'_, u32, S, Self::Prepared>) {
            self.visible = commit.into_prepared();
            self.commits += 1;
        }
    }
    fn whole_root_replacement<S: LayoutScalar>() {
        let initial = run(fixture::<S>()).unwrap();
        let old = initial.node_layout(0).unwrap().fragments().unwrap()[1]
            .id()
            .clone();
        let mut sink = Sink {
            targets: LayoutPublicationTargets::new(),
            visible: Vec::new(),
            reject: Cell::new(false),
            commits: 0,
        };
        sink.targets
            .install(0, initial.target_basis().unwrap().clone())
            .unwrap();
        initial.apply_to(&mut sink).unwrap();
        assert_eq!(
            sink.visible
                .iter()
                .filter(|fragment| fragment.overflow_only().is_some())
                .count(),
            2
        );
        let replacement = run(Fixture {
            inventory: captured::<S>(false),
            frames: frames(),
            metadata: Vec::new(),
            appearances: vec![PreparedAppearance {
                node: 0,
                frame: 0,
                parent: None,
                payload: FragmentPayloadOf::ActualBox(actual(0, 0.0, 0.0)),
                source_children: Vec::new(),
                graphical: None,
            }],
        })
        .unwrap();
        sink.targets
            .install(0, replacement.target_basis().unwrap().clone())
            .unwrap();
        sink.reject.set(true);
        assert_eq!(
            replacement.apply_to(&mut sink),
            Err(LayoutBatchApplyError::Preparation("rejected"))
        );
        assert!(sink.visible.iter().any(|fragment| fragment.id() == &old));
        assert_eq!(sink.commits, 1);
        sink.reject.set(false);
        replacement.apply_to(&mut sink).unwrap();
        assert_eq!(sink.visible.len(), 1);
        assert!(sink.visible[0].principal_box().is_some());
        assert_eq!(sink.targets.published_members(0), &[0]);
        assert_eq!(sink.commits, 2);
        assert!(matches!(
            replacement.fragment(&old),
            Err(LayoutHandleError::WrongResult)
        ));
    }

    #[test]
    fn mixed_accepted_graph_resolves_carriers_and_zero_boxes_f32() {
        mixed_graph::<f32>();
    }
    #[test]
    fn mixed_accepted_graph_resolves_carriers_and_zero_boxes_f64() {
        mixed_graph::<f64>();
    }
    #[test]
    fn canonical_atomic_phase_custody_is_moved_unchanged_f32() {
        opaque_custody::<f32>();
    }
    #[test]
    fn canonical_atomic_phase_custody_is_moved_unchanged_f64() {
        opaque_custody::<f64>();
    }
    #[test]
    fn malformed_parent_and_source_custody_is_refused_f32() {
        invalid_graph::<f32>();
    }
    #[test]
    fn malformed_parent_and_source_custody_is_refused_f64() {
        invalid_graph::<f64>();
    }
    #[test]
    fn malformed_phase_pair_or_overflow_origin_is_refused_f32() {
        invalid_phase::<f32>();
    }
    #[test]
    fn malformed_phase_pair_or_overflow_origin_is_refused_f64() {
        invalid_phase::<f64>();
    }
    #[test]
    fn assembled_whole_root_replacement_removes_old_carriers_atomically_f32() {
        whole_root_replacement::<f32>();
    }
    #[test]
    fn assembled_whole_root_replacement_removes_old_carriers_atomically_f64() {
        whole_root_replacement::<f64>();
    }
    #[test]
    fn unsorted_family_is_canonical_without_reassigning_parents_f32() {
        unsorted_family_keeps_forward_parent_identity::<f32>();
    }
    #[test]
    fn unsorted_family_is_canonical_without_reassigning_parents_f64() {
        unsorted_family_keeps_forward_parent_identity::<f64>();
    }
}
