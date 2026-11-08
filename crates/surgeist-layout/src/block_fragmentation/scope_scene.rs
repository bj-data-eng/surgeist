//! One-time geometry composition of accepted scope packets.
use super::*;
use crate::block_fragmentation::scope_assembly::{
    AppearanceIndex, PreparedAppearance, PreparedMetadata,
};
use std::collections::{BTreeMap, VecDeque};

pub(super) struct ScopePacket<Node, S: LayoutScalar> {
    pub(super) frame: usize,
    pub(super) packet: Packet<Node, S>,
    pub(super) graphical: Vec<(Node, FragmentationGraphicalCoverageOf<S>)>,
}
pub(super) struct SceneOutput<Node, S: LayoutScalar> {
    pub(super) appearances: Vec<PreparedAppearance<Node, S>>,
    pub(super) metadata: Vec<PreparedMetadata<Node, S>>,
    pub(super) receipts: Vec<ReceiptId>,
    pub(super) clears: Vec<LayoutCacheClearEntry<Node>>,
}
#[derive(Clone, Copy)]
enum Geometry<S: LayoutScalar> {
    Entry { packet: usize, entry: usize },
    Overflow(OverflowFragmentGeometryOf<S>),
}
struct Node<S: LayoutScalar> {
    source: usize,
    frame: usize,
    geometry: Geometry<S>,
    parent: Option<usize>,
    children: Vec<usize>,
    anchor: Option<crate::engine::FragmentAnchor<S>>,
    graphical: Option<FragmentationGraphicalCoverageOf<S>>,
}
#[derive(Clone, Copy)]
enum RoundJob {
    Packet(usize),
    Carrier(usize),
}

/// Accepted visual packets and Block-issued parent-local carrier origins are
/// consumed as issued. Sizing, placement and relative-inset completion belong
/// to their producers. The immutable inventory is a validated source capture.
pub(super) struct Scene<'a, 'capture, 'tree, Tree: LayoutTree> {
    environment: &'a NativeProvider<'capture, 'tree, Tree>,
    nodes: Vec<Node<Tree::Scalar>>,
    lookup: BTreeMap<(usize, usize), usize>,
    packets: Vec<ScopePacket<Tree::Node, Tree::Scalar>>,
}
impl<'a, 'capture, 'tree, Tree: LayoutTree> Scene<'a, 'capture, 'tree, Tree> {
    fn failure(&self) -> LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError> {
        self.environment
            .failure(BlockFragmentationFailure::InvalidGeometry)
    }
    fn allocation(&self) -> LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError> {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(self.environment.capture.root),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::Allocation,
        )
    }
    pub(super) fn new(
        environment: &'a NativeProvider<'capture, 'tree, Tree>,
        mut packets: Vec<ScopePacket<Tree::Node, Tree::Scalar>>,
        mut carrier_origin: impl FnMut(usize, usize) -> NativeResult<Tree, Point<Tree::Scalar>>,
    ) -> NativeResult<Tree, Self> {
        let mut nodes = Vec::new();
        let mut lookup = BTreeMap::new();
        for (packet_index, packet) in packets.iter_mut().enumerate() {
            if packet.packet.target != environment.target
                || environment.frames.get(packet.frame).is_none()
            {
                return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
            }
            let frame = environment.frames[packet.frame];
            if !finite_point(frame.origin) || !finite_size(frame.size) {
                return Err(environment.failure(BlockFragmentationFailure::InvalidGeometry));
            }
            if packet.packet.clears.iter().any(|clear| {
                !environment
                    .inventory
                    .nodes
                    .iter()
                    .any(|source| source.node == clear.node())
            }) {
                return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
            }
            for (entry_index, entry) in packet.packet.entries.iter_mut().enumerate() {
                let source = environment
                    .inventory
                    .nodes
                    .iter()
                    .position(|source| source.node == entry.node())
                    .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
                *entry = entry.with_source_index(environment.inventory.nodes[source].source_index);
                let captured = &environment.inventory.nodes[source];
                let measured_hidden = match entry.geometry_state() {
                    crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } => {
                        measured_normal.is_hidden()
                    }
                    crate::output::GeometryStateOf::CanonicalAtomic {
                        normal_origin,
                        visual_origin,
                        readback: crate::output::AtomicReadbackPhase::Pending,
                        ..
                    } if finite_point(normal_origin) && finite_point(visual_origin) => false,
                    _ => {
                        return Err(environment.failure(BlockFragmentationFailure::InvalidGeometry));
                    }
                };
                if captured.hidden != measured_hidden
                    || entry.non_box.is_some() && entry.non_box != Some(NonBoxKind::Hidden)
                    || !captured.hidden
                        && (entry.non_box.is_some() || entry.output().scroll_geometry.is_none())
                {
                    return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
                }
                if !finite_point(entry.output().location)
                    || !finite_size(entry.output().size)
                    || !finite_size(entry.output().content_size)
                {
                    return Err(environment.failure(BlockFragmentationFailure::InvalidGeometry));
                }
                if lookup.contains_key(&(packet.frame, source)) {
                    return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
                }
                if captured.hidden
                    && nodes
                        .iter()
                        .any(|node: &Node<Tree::Scalar>| node.source == source)
                {
                    return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
                }
                let index = nodes.len();
                nodes.try_reserve(1).map_err(|_| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(entry.node()),
                        LayoutOperation::RootLayout,
                        LayoutErrorKindOf::Allocation,
                    )
                })?;
                nodes.push(Node {
                    source,
                    frame: packet.frame,
                    geometry: Geometry::Entry {
                        packet: packet_index,
                        entry: entry_index,
                    },
                    parent: None,
                    children: Vec::new(),
                    anchor: None,
                    graphical: None,
                });
                lookup.insert((packet.frame, source), index);
            }
            for &(node, coverage) in &packet.graphical {
                let source = environment
                    .inventory
                    .nodes
                    .iter()
                    .position(|source| source.node == node)
                    .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
                let index = lookup.get(&(packet.frame, source)).copied()
                    .filter(|&index| matches!(nodes[index].geometry, Geometry::Entry { packet, .. } if packet == packet_index))
                    .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
                if environment.inventory.nodes[source].hidden
                    || nodes[index].graphical.is_some()
                    || !coverage.start.is_finite()
                    || !coverage.end.is_finite()
                    || !coverage.extent.is_finite()
                    || coverage.start < Tree::Scalar::ZERO
                    || coverage.start >= coverage.end
                    || coverage.end > coverage.extent
                {
                    return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
                }
                nodes[index].graphical = Some(coverage);
            }
        }
        let mut scene = Self {
            environment,
            nodes,
            lookup,
            packets,
        };
        for node in &scene.nodes {
            let source = &environment.inventory.nodes[node.source];
            if !source.hidden
                && let Some(owner) = source.atomic_owner
                && owner != node.source
            {
                let Some(&owner_index) = scene.lookup.get(&(node.frame, owner)) else {
                    return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
                };
                if !matches!((node.geometry, scene.nodes[owner_index].geometry),
                    (Geometry::Entry { packet, .. }, Geometry::Entry { packet: owner_packet, .. }) if packet == owner_packet)
                {
                    return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
                }
            }
        }
        let mut cursor = 0;
        while cursor < scene.nodes.len() {
            let frame = scene.nodes[cursor].frame;
            let source = scene.nodes[cursor].source;
            if let Some(parent_source) = environment.inventory.nodes[source].parent {
                let parent = match scene.lookup.get(&(frame, parent_source)) {
                    Some(&parent) => parent,
                    None if environment.inventory.nodes[source].hidden => {
                        cursor += 1;
                        continue;
                    }
                    None => {
                        if environment.inventory.nodes[parent_source].hidden
                            || environment.inventory.nodes[parent_source]
                                .atomic_owner
                                .is_some()
                        {
                            return Err(
                                environment.failure(BlockFragmentationFailure::InvalidSource)
                            );
                        }
                        let origin = carrier_origin(parent_source, frame)?;
                        if !finite_point(origin) {
                            return Err(scene.failure());
                        }
                        let empty = ScrollRectOf::try_new(Point::ZERO, Size::ZERO)
                            .map_err(|_| scene.failure())?;
                        let parent = scene.nodes.len();
                        scene.nodes.try_reserve(1).map_err(|_| scene.allocation())?;
                        scene.nodes.push(Node {
                            source: parent_source,
                            frame,
                            parent: None,
                            children: Vec::new(),
                            anchor: None,
                            graphical: None,
                            geometry: Geometry::Overflow(OverflowFragmentGeometryOf {
                                origin,
                                content_bounds: empty,
                            }),
                        });
                        scene.lookup.insert((frame, parent_source), parent);
                        parent
                    }
                };
                scene.nodes[parent]
                    .children
                    .try_reserve(1)
                    .map_err(|_| scene.allocation())?;
                scene.nodes[parent].children.push(cursor);
                scene.nodes[cursor].parent = Some(parent);
            }
            cursor += 1;
        }
        scene.merge_bounds()?;
        Ok(scene)
    }
    fn merge_bounds(&mut self) -> NativeResult<Tree, ()> {
        let mut order = Vec::new();
        order
            .try_reserve(self.nodes.len())
            .map_err(|_| self.allocation())?;
        order.extend(0..self.nodes.len());
        order.sort_unstable_by_key(|&index| self.nodes[index].source);
        for index in order.into_iter().rev() {
            let source = &self.environment.inventory.nodes[self.nodes[index].source];
            if source.hidden || source.atomic_owner.is_some() {
                continue;
            }
            // descendant_bounds excludes this API's container seed. Only
            // actual child contributions become an overflow carrier's bounds.
            let empty =
                ScrollRectOf::try_new(Point::ZERO, Size::ZERO).map_err(|_| self.failure())?;
            let mut contributions = crate::scroll::ScrollContributionAccumulatorOf::new(empty);
            for &child in &self.nodes[index].children {
                if self.environment.inventory.nodes[self.nodes[child].source].hidden {
                    continue;
                }
                match self.nodes[child].geometry {
                    Geometry::Entry { packet, entry } => {
                        let entry = self.packets[packet].packet.entries[entry];
                        if entry.non_box.is_some() {
                            continue;
                        }
                        let output = entry.output();
                        let geometry = output.scroll_geometry.ok_or_else(|| self.failure())?;
                        contributions
                            .include_in_flow_geometry(output.location, output.margin, geometry)
                            .map_err(|_| self.failure())?;
                    }
                    Geometry::Overflow(geometry) => {
                        let bounds = ScrollRectOf::try_new(
                            Point::new(
                                geometry.origin.x + geometry.content_bounds.origin().x,
                                geometry.origin.y + geometry.content_bounds.origin().y,
                            ),
                            geometry.content_bounds.size(),
                        )
                        .map_err(|_| self.failure())?;
                        contributions.include_descendant_bounds(bounds);
                    }
                }
            }
            let Some(bounds) = contributions
                .descendant_bounds()
                .map_err(|_| self.failure())?
            else {
                continue;
            };
            match self.nodes[index].geometry {
                Geometry::Entry { packet, entry } => {
                    let old = self.packets[packet].packet.entries[entry];
                    let crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } =
                        old.geometry_state()
                    else {
                        return Err(self.failure());
                    };
                    let mut output = old.output();
                    let (geometry, content_size) = crate::scroll::include_scope_overflow(
                        output.scroll_geometry.ok_or_else(|| self.failure())?,
                        bounds,
                    )
                    .map_err(|_| self.failure())?;
                    output.content_size = content_size;
                    output = output.with_scroll_geometry(Some(geometry));
                    self.packets[packet].packet.entries[entry] =
                        LayoutOutputEntryOf::ordinary_unrounded(
                            old.node(),
                            measured_normal,
                            output,
                        );
                }
                Geometry::Overflow(mut geometry) => {
                    geometry.content_bounds = bounds;
                    self.nodes[index].geometry = Geometry::Overflow(geometry);
                }
            }
        }
        Ok(())
    }
    pub(super) fn round(self) -> NativeResult<Tree, SceneOutput<Tree::Node, Tree::Scalar>> {
        let Self {
            environment,
            mut nodes,
            lookup,
            packets,
        } = self;
        let failure = || environment.failure(BlockFragmentationFailure::InvalidGeometry);
        let allocation = || {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(environment.capture.root),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::Allocation,
            )
        };
        let mut pending = Vec::new();
        pending
            .try_reserve(packets.len())
            .map_err(|_| allocation())?;
        pending.extend(packets.into_iter().map(Some));
        let mut paired = Vec::new();
        paired.try_reserve(nodes.len()).map_err(|_| allocation())?;
        paired.resize(nodes.len(), None);
        let mut overflow = Vec::new();
        overflow
            .try_reserve(nodes.len())
            .map_err(|_| allocation())?;
        overflow.resize(nodes.len(), None);
        let mut waiting = Vec::new();
        waiting.try_reserve(nodes.len()).map_err(|_| allocation())?;
        waiting.resize_with(nodes.len(), Vec::new);
        let mut dependencies = Vec::new();
        dependencies
            .try_reserve(pending.len())
            .map_err(|_| allocation())?;
        dependencies.resize(pending.len(), 0usize);
        let mut effects = Vec::new();
        effects
            .try_reserve(pending.len())
            .map_err(|_| allocation())?;
        effects.resize_with(pending.len(), || None);
        let mut ready = VecDeque::new();
        ready
            .try_reserve(nodes.len().saturating_add(pending.len()))
            .map_err(|_| allocation())?;
        for (index, node) in nodes.iter().enumerate() {
            if matches!(node.geometry, Geometry::Overflow(_)) {
                match node.parent {
                    Some(parent) => {
                        waiting[parent].try_reserve(1).map_err(|_| allocation())?;
                        waiting[parent].push(RoundJob::Carrier(index));
                    }
                    None => ready.push_back(RoundJob::Carrier(index)),
                }
            }
        }
        for (packet, group) in pending.iter().enumerate() {
            let group = group.as_ref().expect("unrounded packet is owned");
            let mut parents = Vec::new();
            parents
                .try_reserve(group.packet.entries.len())
                .map_err(|_| allocation())?;
            for entry in &group.packet.entries {
                let source = environment
                    .inventory
                    .nodes
                    .iter()
                    .position(|source| source.node == entry.node())
                    .expect("scene source was validated");
                let node = lookup[&(group.frame, source)];
                let external = nodes[node]
                    .parent
                    .filter(|&parent| match nodes[parent].geometry {
                        Geometry::Entry { packet: owner, .. } => owner != packet,
                        Geometry::Overflow(_) => true,
                    });
                if let Some(parent) = external
                    && !parents.contains(&parent)
                {
                    parents.push(parent);
                }
            }
            dependencies[packet] = parents.len();
            if parents.is_empty() {
                ready.push_back(RoundJob::Packet(packet));
            }
            for parent in parents {
                waiting[parent].try_reserve(1).map_err(|_| allocation())?;
                waiting[parent].push(RoundJob::Packet(packet));
            }
        }
        let tree = CapturedTree {
            tree: environment.capture.tree,
            inventory: environment.inventory,
        };
        while let Some(job) = ready.pop_front() {
            match job {
                RoundJob::Carrier(index) => {
                    let node = &nodes[index];
                    let ancestor = node
                        .parent
                        .map_or_else(
                            || {
                                Some(crate::engine::FragmentAnchor::external(
                                    environment.frames[node.frame].origin,
                                ))
                            },
                            |parent| nodes[parent].anchor,
                        )
                        .ok_or_else(failure)?;
                    let Geometry::Overflow(raw) = node.geometry else {
                        return Err(failure());
                    };
                    let (final_geometry, anchor) = ancestor.overflow(raw).map_err(|_| failure())?;
                    overflow[index] = Some(OverflowGeometryPairOf {
                        unrounded: raw,
                        final_geometry,
                    });
                    nodes[index].anchor = Some(anchor);
                    ready
                        .try_reserve(waiting[index].len())
                        .map_err(|_| allocation())?;
                    ready.extend(std::mem::take(&mut waiting[index]));
                }
                RoundJob::Packet(index) => {
                    if dependencies[index] > 0 {
                        dependencies[index] -= 1;
                        if dependencies[index] > 0 {
                            continue;
                        }
                    }
                    let group = pending[index].take().ok_or_else(failure)?;
                    // An empty terminal content window carries effects
                    // without owning a principal, carrier or rounding root.
                    if group.packet.entries.is_empty() {
                        effects[index] = Some((group.packet.receipts, group.packet.clears));
                        continue;
                    }
                    let mut roots = Vec::new();
                    roots
                        .try_reserve(group.packet.entries.len())
                        .map_err(|_| allocation())?;
                    for entry in &group.packet.entries {
                        let source = environment
                            .inventory
                            .nodes
                            .iter()
                            .position(|source| source.node == entry.node())
                            .expect("scene source was validated");
                        let node = lookup[&(group.frame, source)];
                        if nodes[node].parent.is_some_and(|parent| matches!(nodes[parent].geometry, Geometry::Entry { packet, .. } if packet == index)) { continue; }
                        let anchor = nodes[node]
                            .parent
                            .map_or_else(
                                || {
                                    Some(crate::engine::FragmentAnchor::external(
                                        environment.frames[group.frame].origin,
                                    ))
                                },
                                |parent| nodes[parent].anchor,
                            )
                            .ok_or_else(failure)?;
                        roots.push((entry.node(), anchor));
                    }
                    let packet = group.packet.round(&tree, &roots)?;
                    if packet.target != environment.target {
                        return Err(failure());
                    }
                    for pair in packet.entries {
                        let source = environment
                            .inventory
                            .nodes
                            .iter()
                            .position(|source| source.node == pair.unrounded.node())
                            .expect("scene source was validated");
                        let node = lookup[&(group.frame, source)];
                        paired[node] = Some(pair);
                    }
                    for (node, anchor) in packet.anchors {
                        let source = environment
                            .inventory
                            .nodes
                            .iter()
                            .position(|source| source.node == node)
                            .expect("scene source was validated");
                        let node = lookup[&(group.frame, source)];
                        if nodes[node].anchor.replace(anchor).is_some() {
                            return Err(failure());
                        }
                        ready
                            .try_reserve(waiting[node].len())
                            .map_err(|_| allocation())?;
                        ready.extend(std::mem::take(&mut waiting[node]));
                    }
                    effects[index] = Some((packet.receipts, packet.clears));
                }
            }
        }
        if pending.iter().any(Option::is_some) || nodes.iter().any(|node| node.anchor.is_none()) {
            return Err(failure());
        }
        // Rounding dependencies do not reorder adopted packet effects.
        // Clearing/publication policy remains with the driver.
        let mut receipts = Vec::new();
        let mut clears = Vec::new();
        for effect in effects {
            let (packet_receipts, packet_clears) = effect.ok_or_else(failure)?;
            receipts
                .try_reserve(packet_receipts.len())
                .map_err(|_| allocation())?;
            receipts.extend(packet_receipts);
            clears
                .try_reserve(packet_clears.len())
                .map_err(|_| allocation())?;
            clears.extend(packet_clears);
        }
        let mut appearances = Vec::new();
        appearances
            .try_reserve(nodes.len())
            .map_err(|_| allocation())?;
        let mut metadata = Vec::new();
        metadata
            .try_reserve(nodes.len())
            .map_err(|_| allocation())?;
        let mut indices = Vec::new();
        indices.try_reserve(nodes.len()).map_err(|_| allocation())?;
        indices.resize(nodes.len(), None);
        for (index, node) in nodes.iter().enumerate() {
            if environment.inventory.nodes[node.source].hidden {
                continue;
            }
            indices[index] = Some(AppearanceIndex(appearances.len()));
            let payload = match node.geometry {
                Geometry::Entry { .. } => {
                    FragmentPayloadOf::ActualBox(paired[index].take().ok_or_else(failure)?)
                }
                Geometry::Overflow(_) => {
                    FragmentPayloadOf::OverflowOnly(overflow[index].take().ok_or_else(failure)?)
                }
            };
            appearances.push(PreparedAppearance {
                node: environment.inventory.nodes[node.source].node,
                frame: node.frame,
                parent: None,
                payload,
                source_children: Vec::new(),
                graphical: node.graphical,
            });
        }
        for (index, node) in nodes.iter().enumerate() {
            let Some(appearance) = indices[index] else {
                let pair = paired[index].take().ok_or_else(failure)?;
                metadata.push(PreparedMetadata {
                    kind: NonBoxKind::Hidden,
                    unrounded: pair.unrounded,
                    final_entry: pair.final_entry,
                });
                continue;
            };
            appearances[appearance.0].parent = node
                .parent
                .map(|parent| indices[parent].ok_or_else(failure))
                .transpose()?;
            appearances[appearance.0]
                .source_children
                .try_reserve(environment.inventory.nodes[node.source].children.len())
                .map_err(|_| allocation())?;
            for &child in &environment.inventory.nodes[node.source].children {
                if lookup.contains_key(&(node.frame, child))
                    || environment.inventory.nodes[child].hidden
                        && nodes.iter().any(|node| node.source == child)
                {
                    let source = &environment.inventory.nodes[child];
                    appearances[appearance.0]
                        .source_children
                        .push((source.source_index, source.node));
                }
            }
        }
        Ok(SceneOutput {
            appearances,
            metadata,
            receipts,
            clears,
        })
    }
}

fn finite_point<S: LayoutScalar>(value: Point<S>) -> bool {
    value.x.is_finite() && value.y.is_finite()
}
fn finite_size<S: LayoutScalar>(value: Size<S>) -> bool {
    value.width.is_finite()
        && value.height.is_finite()
        && value.width >= S::ZERO
        && value.height >= S::ZERO
}

struct CapturedTree<'a, Tree: LayoutTree> {
    tree: &'a Tree,
    inventory: &'a Inventory<Tree::Node>,
}
struct CapturedChildren<'a, Node> {
    inventory: &'a Inventory<Node>,
    children: std::slice::Iter<'a, usize>,
}
impl<Node: Copy> Iterator for CapturedChildren<'_, Node> {
    type Item = Node;
    fn next(&mut self) -> Option<Self::Item> {
        self.children
            .next()
            .map(|&child| self.inventory.nodes[child].node)
    }
}
impl<Tree: LayoutTree> CapturedTree<'_, Tree> {
    fn source(
        &self,
        node: Tree::Node,
    ) -> &crate::block_fragmentation::source::SourceNode<Tree::Node> {
        self.inventory
            .nodes
            .iter()
            .find(|source| source.node == node)
            .expect("scene validated every packet node")
    }
}
impl<Tree: LayoutTree> Traverse for CapturedTree<'_, Tree> {
    type Node = Tree::Node;
    type Scalar = Tree::Scalar;
    type Children<'a>
        = CapturedChildren<'a, Self::Node>
    where
        Self: 'a;
    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        CapturedChildren {
            inventory: self.inventory,
            children: self.source(node).children.iter(),
        }
    }
    fn child_count(&self, node: Self::Node) -> usize {
        self.source(node).children.len()
    }
    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.inventory.nodes[self.source(node).children[index]].node
    }
}
impl<Tree: LayoutTree> LayoutTree for CapturedTree<'_, Tree> {
    type MeasureError = Tree::MeasureError;
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: Self::Node) -> LayoutInputViewOf<'_, Self::Scalar> {
        self.tree.layout_input(node)
    }
    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.tree.has_leaf_measurement(node)
    }
    fn measure_leaf(
        &self,
        node: Self::Node,
        input: LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<Size<Self::Scalar>, Self::MeasureError>> {
        self.tree.measure_leaf(node, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::layout_tree::PublicLayoutTreeOf;

    fn n<S: LayoutScalar>(value: f64) -> S {
        S::from_f64(value)
    }
    fn p<S: LayoutScalar>(x: f64, y: f64) -> Point<S> {
        Point::new(n(x), n(y))
    }
    fn axes() -> FlowAxes {
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
    }
    fn tree<S: LayoutScalar>() -> PublicLayoutTreeOf<S> {
        let block = || NodeInputOf {
            display: Display::Block,
            ..Default::default()
        };
        let mut root = block();
        root.padding.left = LengthOf::px(n(1.25));
        root.padding.top = LengthOf::px(n(2.25));
        let mut parent = block();
        parent.padding.left = LengthOf::px(n(3.25));
        parent.padding.top = LengthOf::px(n(4.5));
        let mut leaf = block();
        leaf.size = Size::new(PreferredSizeOf::px(n(5.5)), PreferredSizeOf::px(n(7.25)));
        PublicLayoutTreeOf::new()
            .style(0, root)
            .style(1, parent)
            .style(2, leaf)
            .style(
                3,
                NodeInputOf {
                    display: Display::None,
                    ..block()
                },
            )
            .children(0, [1, 3])
            .children(1, [2])
    }
    fn with_environment<S: LayoutScalar>(
        tree: &PublicLayoutTreeOf<S>,
        run: impl FnOnce(&NativeProvider<'_, '_, PublicLayoutTreeOf<S>>),
    ) {
        let request = LayoutRootRequestOf::standalone(
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
            axes(),
            Size::NONE,
        );
        let frames = [
            BlockFragmentainerOf {
                origin: p(0.25, 0.25),
                size: Size::new(n(100.0), n(50.0)),
                breaks: FragmentationBreakKinds::NONE,
                page_side: None,
            },
            BlockFragmentainerOf {
                origin: p(0.25, 0.25),
                size: Size::new(n(80.0), n(80.0)),
                breaks: FragmentationBreakKinds::PAGE,
                page_side: None,
            },
        ];
        let bound = LayoutBoundRequestOf::block_fragmentation(
            LayoutSourceBasis::fresh(),
            request,
            FragmentationKind::Page,
            axes(),
            frames.to_vec(),
        )
        .unwrap();
        let capture = LayoutFlowInput::try_new_bound(
            tree,
            0,
            bound.clone(),
            Vec::new(),
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let inventory = Inventory::new(tree, &capture.graph, axes()).unwrap();
        let environment = NativeProvider {
            capture: &capture,
            inventory: &inventory,
            frames: &frames,
            axes: axes(),
            kind: FragmentationKind::Page,
            target: bound.target_basis(),
            template: request,
            intrinsics: Rc::new(RefCell::new(Vec::new())),
            progress: Rc::new(RefCell::new(Default::default())),
        };
        run(&environment);
    }

    /// Only the source window changes. Every output is still produced by the
    /// real scoped Block/root owners, with the original styles and target.
    struct Window<'a, S: LayoutScalar> {
        tree: &'a PublicLayoutTreeOf<S>,
        included: &'a [u32],
    }
    impl<S: LayoutScalar> Traverse for Window<'_, S> {
        type Node = u32;
        type Scalar = S;
        type Children<'a>
            = std::vec::IntoIter<u32>
        where
            Self: 'a;
        fn children(&self, node: u32) -> Self::Children<'_> {
            self.tree
                .children(node)
                .filter(|child| self.included.contains(child))
                .collect::<Vec<_>>()
                .into_iter()
        }
        fn child_count(&self, node: u32) -> usize {
            self.children(node).count()
        }
        fn child(&self, node: u32, index: usize) -> u32 {
            self.children(node).nth(index).unwrap()
        }
    }
    impl<S: LayoutScalar> LayoutTree for Window<'_, S> {
        type MeasureError = std::convert::Infallible;
        fn layout_input(&self, node: u32) -> crate::LayoutInputViewOf<'_, S> {
            self.tree.layout_input(node)
        }
        fn node_input(&self, node: u32) -> &NodeInputOf<S> {
            self.tree.node_input(node)
        }
    }
    fn produce<S: LayoutScalar>(
        environment: &NativeProvider<'_, '_, PublicLayoutTreeOf<S>>,
        included: &[u32],
        frame: usize,
    ) -> ScopePacket<u32, S> {
        let view = Window {
            tree: environment.capture.tree,
            included,
        };
        let bounds = environment.frames[frame];
        let target = environment.target.clone();
        let packet = crate::engine::compute_block_scope_root_frame(
            &view,
            crate::engine::BlockScopeRootFrameRequest {
                root: 0,
                request: environment.request(bounds),
                frame: bounds,
                axes: axes(),
                binding: crate::engine::BlockFrameBinding {
                    allocations: Vec::new(),
                    intrinsics: environment.intrinsics(bounds).unwrap(),
                    dispatch: crate::engine::block_scope::Dispatch {
                        scope: ReceiptId(0),
                        view: crate::engine::block_scope::SourceViewId(0),
                        target,
                        owners: Arc::from([]),
                        paths: Arc::from([]),
                        ready: Rc::new(RefCell::new(Vec::new())),
                        requests: Rc::new(
                            RefCell::new(crate::engine::block_scope::Requests::new()),
                        ),
                    },
                },
            },
        )
        .unwrap();
        ScopePacket {
            frame,
            packet,
            graphical: Vec::new(),
        }
    }
    fn select<S: LayoutScalar>(
        mut packet: ScopePacket<u32, S>,
        nodes: &[u32],
    ) -> ScopePacket<u32, S> {
        packet
            .packet
            .entries
            .retain(|entry| nodes.contains(&entry.node()));
        packet
    }
    fn actual<S: LayoutScalar>(
        scene: &SceneOutput<u32, S>,
        node: u32,
        frame: usize,
    ) -> BoxGeometryPairOf<u32, S> {
        match scene
            .appearances
            .iter()
            .find(|entry| entry.node == node && entry.frame == frame)
            .unwrap()
            .payload
        {
            FragmentPayloadOf::ActualBox(pair) => pair,
            FragmentPayloadOf::OverflowOnly(_) => panic!("expected an actual principal box"),
        }
    }
    fn carrier<S: LayoutScalar>(
        scene: &SceneOutput<u32, S>,
        node: u32,
        frame: usize,
    ) -> OverflowGeometryPairOf<S> {
        match scene
            .appearances
            .iter()
            .find(|entry| entry.node == node && entry.frame == frame)
            .unwrap()
            .payload
        {
            FragmentPayloadOf::OverflowOnly(pair) => pair,
            FragmentPayloadOf::ActualBox(_) => panic!("expected an overflow-only carrier"),
        }
    }
    fn assert_parent<S: LayoutScalar>(
        scene: &SceneOutput<u32, S>,
        child: u32,
        parent: u32,
        frame: usize,
    ) {
        let child = scene
            .appearances
            .iter()
            .find(|entry| entry.node == child && entry.frame == frame)
            .unwrap();
        let parent_index = child.parent.unwrap();
        assert_eq!(
            (
                scene.appearances[parent_index.0].node,
                scene.appearances[parent_index.0].frame
            ),
            (parent, frame)
        );
    }

    fn actual_ancestor_receives_descendant_before_rounding<S: LayoutScalar>() {
        with_environment(&tree::<S>(), |environment| {
            let mut parent = select(produce(environment, &[0, 1, 3], 0), &[0, 1, 3]);
            let before = parent
                .packet
                .entries
                .iter()
                .find(|entry| entry.node() == 1)
                .unwrap()
                .output();
            assert_eq!(before.size.height, n(4.5));
            let mut child = select(produce(environment, &[0, 1, 2], 0), &[2]);
            child.graphical.push((
                2,
                FragmentationGraphicalCoverageOf {
                    start: n(1.0),
                    end: n(3.0),
                    extent: n(4.0),
                },
            ));
            child.packet.receipts.push(ReceiptId(7));
            child.packet.clears.push(LayoutCacheClearEntry::new(2));
            parent.packet.receipts.push(ReceiptId(8));
            parent.packet.clears.push(LayoutCacheClearEntry::new(1));
            let result = Scene::new(environment, vec![child, parent], |_, _| {
                panic!("existing actual ancestor must be reused")
            })
            .unwrap()
            .round()
            .unwrap();
            assert_eq!(result.appearances.len(), 3);
            assert_eq!(result.metadata.len(), 1);
            assert_parent(&result, 2, 1, 0);
            assert_parent(&result, 1, 0, 0);
            let parent = actual(&result, 1, 0);
            assert_eq!(parent.unrounded.output().size, before.size);
            // The padding-box seed [0, 4.5] joins leaf bounds [4.5, 11.75].
            assert_eq!(parent.unrounded.output().content_size.height, n(11.75));
            let scroll = parent.unrounded.output().scroll_geometry.unwrap();
            assert_eq!(scroll.content_box().origin().y, n(4.5));
            assert_eq!(scroll.scrollable_overflow().origin().y, n(0.0));
            assert_eq!(scroll.scrollable_overflow().size().height, n(11.75));
            let leaf = actual(&result, 2, 0);
            assert_eq!(leaf.unrounded.output().location, p(3.25, 4.5));
            assert_eq!(leaf.final_entry.output().location, p(3.0, 4.0));
            assert_eq!(leaf.final_entry.output().size, Size::new(n(5.0), n(7.0)));
            assert_eq!(result.receipts, [ReceiptId(7), ReceiptId(8)]);
            assert_eq!(
                result
                    .clears
                    .iter()
                    .map(LayoutCacheClearEntry::node)
                    .collect::<Vec<_>>(),
                [2, 3, 1]
            );
            let leaf_appearance = result
                .appearances
                .iter()
                .find(|entry| entry.node == 2)
                .unwrap();
            assert_eq!(leaf_appearance.graphical.unwrap().start, n(1.0));
            let root = result
                .appearances
                .iter()
                .find(|entry| entry.node == 0)
                .unwrap();
            assert_eq!(
                root.source_children
                    .iter()
                    .map(|&(_, node)| node)
                    .collect::<Vec<_>>(),
                [1, 3]
            );
            let hidden = &result.metadata[0];
            assert_eq!(hidden.unrounded.node(), 3);
            assert_eq!(hidden.final_entry.non_box, Some(NonBoxKind::Hidden));
        });
    }
    fn missing_principal_gets_one_carrier_with_issued_origin<S: LayoutScalar>() {
        with_environment(&tree::<S>(), |environment| {
            let parent = produce(environment, &[0], 1);
            let before = parent
                .packet
                .entries
                .iter()
                .find(|entry| entry.node() == 0)
                .unwrap()
                .output();
            let child = select(produce(environment, &[0, 1, 2], 1), &[2]);
            let result = Scene::new(environment, vec![child, parent], |source, frame| {
                assert_eq!((environment.inventory.nodes[source].node, frame), (1, 1));
                Ok(p(1.25, 2.25))
            })
            .unwrap()
            .round()
            .unwrap();
            assert_eq!(result.appearances.len(), 3);
            assert_parent(&result, 2, 1, 1);
            assert_parent(&result, 1, 0, 1);
            let overflow = carrier(&result, 1, 1);
            assert_eq!(overflow.unrounded.origin, p(1.25, 2.25));
            assert_eq!(overflow.unrounded.content_bounds.origin(), p(3.25, 4.5));
            assert_eq!(
                overflow.unrounded.content_bounds.size(),
                Size::new(n(5.5), n(7.25))
            );
            assert_eq!(overflow.final_geometry.origin, p(2.0, 3.0));
            assert_eq!(overflow.final_geometry.content_bounds.origin(), p(3.0, 4.0));
            assert_eq!(
                overflow.final_geometry.content_bounds.size(),
                Size::new(n(5.0), n(7.0))
            );
            let root = actual(&result, 0, 1);
            assert_eq!(root.unrounded.output().size, before.size);
            // Root padding [0, 2.25] joins the carrier's translated [6.75, 14].
            assert_eq!(root.unrounded.output().content_size.height, n(14.0));
            assert_eq!(
                root.unrounded
                    .output()
                    .scroll_geometry
                    .unwrap()
                    .scrollable_overflow()
                    .origin()
                    .y
                    + root
                        .unrounded
                        .output()
                        .scroll_geometry
                        .unwrap()
                        .scrollable_overflow()
                        .size()
                        .height,
                n(14.0)
            );
        });
    }
    fn nested_carriers_share_the_actual_rounding_chain<S: LayoutScalar>() {
        with_environment(&tree::<S>(), |environment| {
            let child = select(produce(environment, &[0, 1, 2], 1), &[2]);
            let result = Scene::new(environment, vec![child], |source, _| {
                Ok(match environment.inventory.nodes[source].node {
                    0 => p(0.25, 0.25),
                    1 => p(1.25, 2.25),
                    _ => panic!("only missing ancestors need origins"),
                })
            })
            .unwrap()
            .round()
            .unwrap();
            assert_eq!(result.appearances.len(), 3);
            assert_parent(&result, 2, 1, 1);
            assert_parent(&result, 1, 0, 1);
            let root = carrier(&result, 0, 1);
            let parent = carrier(&result, 1, 1);
            assert_eq!(root.unrounded.content_bounds.origin(), p(4.5, 6.75));
            assert_eq!(
                root.unrounded.content_bounds.size(),
                Size::new(n(5.5), n(7.25))
            );
            assert_eq!(root.final_geometry.origin, p(1.0, 1.0));
            assert_eq!(root.final_geometry.content_bounds.origin(), p(4.0, 6.0));
            assert_eq!(
                root.final_geometry.content_bounds.size(),
                Size::new(n(6.0), n(8.0))
            );
            assert_eq!(parent.final_geometry.origin, p(1.0, 2.0));
            let leaf = actual(&result, 2, 1);
            assert_eq!(leaf.final_entry.output().location, p(3.0, 4.0));
            assert_eq!(leaf.final_entry.output().size, Size::new(n(6.0), n(8.0)));
            assert_eq!(
                parent.final_geometry.content_bounds.size(),
                Size::new(n(6.0), n(8.0))
            );
        });
    }
    fn zero_sized_principal_remains_actual<S: LayoutScalar>() {
        let mut tree = tree::<S>();
        tree.insert_input(
            0,
            LayoutInputOf::box_input(NodeInputOf {
                display: Display::Block,
                ..Default::default()
            }),
        );
        with_environment(&tree, |environment| {
            let parent = produce(environment, &[0], 0);
            let child = select(produce(environment, &[0, 1, 2], 0), &[2]);
            let scene = Scene::new(environment, vec![parent, child], |source, frame| {
                assert_eq!((environment.inventory.nodes[source].node, frame), (1, 0));
                Ok(Point::ZERO)
            })
            .unwrap()
            .round()
            .unwrap();
            assert_eq!(actual(&scene, 0, 0).unrounded.output().size.height, S::ZERO);
            assert_eq!(
                actual(&scene, 0, 0).final_entry.output().size.height,
                S::ZERO
            );
            assert_eq!(
                carrier(&scene, 1, 0).unrounded.content_bounds.size(),
                Size::new(n(5.5), n(7.25))
            );
            assert_parent(&scene, 2, 1, 0);
            assert_parent(&scene, 1, 0, 0);
        });
    }
    fn empty_content_packet_preserves_effects_without_geometry<S: LayoutScalar>() {
        with_environment(&tree::<S>(), |environment| {
            let view = Window {
                tree: environment.capture.tree,
                included: &[0],
            };
            let bounds = environment.frames[0];
            let request = environment.request(bounds);
            let input = ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::new(Some(bounds.size.width), None),
                Size::new(Some(bounds.size.width), None),
                ContainingLayoutContext::new(axes(), ParentFormattingContext::BlockFlow),
                request.sizing_request(),
            );
            let produced = crate::engine::compute_block_content_frame(
                &view,
                crate::engine::BlockContentFrameRequest {
                    root: 0,
                    input,
                    original: environment.capture.tree.node_input(0),
                    reference: S::ZERO,
                    opening: false,
                    complete: true,
                    principal_content: None,
                    accepted: None,
                    binding: crate::engine::BlockFrameBinding {
                        allocations: Vec::new(),
                        intrinsics: environment.intrinsics(bounds).unwrap(),
                        dispatch: crate::engine::block_scope::Dispatch {
                            scope: ReceiptId(0),
                            view: crate::engine::block_scope::SourceViewId(0),
                            target: environment.target.clone(),
                            owners: Arc::from([]),
                            paths: Arc::from([]),
                            ready: Rc::new(RefCell::new(Vec::new())),
                            requests: Rc::new(RefCell::new(
                                crate::engine::block_scope::Requests::new(),
                            )),
                        },
                    },
                },
            )
            .unwrap();
            assert!(produced.packet.entries.is_empty());
            let mut empty = ScopePacket {
                frame: 0,
                packet: produced.packet,
                graphical: Vec::new(),
            };
            empty.packet.receipts.push(ReceiptId(7));
            empty.packet.clears.push(LayoutCacheClearEntry::new(2));
            let mut principal = produce(environment, &[0], 0);
            let original = principal.packet.entries[0];
            principal.packet.receipts.push(ReceiptId(8));
            let scene = Scene::new(environment, vec![empty, principal], |_, _| {
                panic!("empty production cannot create an ancestor appearance")
            })
            .unwrap()
            .round()
            .unwrap();
            assert_eq!(scene.appearances.len(), 1);
            assert!(scene.metadata.is_empty());
            assert_eq!(actual(&scene, 0, 0).unrounded, original);
            assert_eq!(scene.receipts, [ReceiptId(7), ReceiptId(8)]);
            assert_eq!(
                scene
                    .clears
                    .iter()
                    .map(LayoutCacheClearEntry::node)
                    .collect::<Vec<_>>(),
                [2]
            );
        });
    }
    fn signed_visual_packets_compose_into_public_fragment_graph<S: LayoutScalar>() {
        let mut tree = tree::<S>();
        let mut leaf = tree.node_input(2).clone();
        leaf.inset.left = LengthAutoOf::px(n(-5.75));
        leaf.inset.top = LengthAutoOf::px(n(-7.25));
        tree.insert_input(2, LayoutInputOf::box_input(leaf));
        with_environment(&tree, |environment| {
            // Encounter frame1 first; assembly owns continuation ordering.
            let child1 = select(produce(environment, &[0, 1, 2], 1), &[2]);
            let parent1 = produce(environment, &[0], 1);
            let child0 = select(produce(environment, &[0, 1, 2], 0), &[2]);
            let parent0 = produce(environment, &[0, 1, 3], 0);
            let scene = Scene::new(
                environment,
                vec![child1, parent1, child0, parent0],
                |source, frame| {
                    assert_eq!((environment.inventory.nodes[source].node, frame), (1, 1));
                    Ok(p(1.25, 2.25))
                },
            )
            .unwrap()
            .round()
            .unwrap();
            // Padding(3.25,4.5) plus the completed relative inset gives
            // (-2.5,-2.75). The carrier's global raw origin is (1.5,2.5),
            // so its rounded origin is (2,3); the child starts globally at
            // (-1,-0.25), giving the parent-local published (-3,-3).
            let raw_bounds = carrier(&scene, 1, 1).unrounded.content_bounds;
            assert_eq!(raw_bounds.origin(), p(-2.5, -2.75));
            assert_eq!(raw_bounds.size(), Size::new(n(5.5), n(7.25)));
            let final_bounds = carrier(&scene, 1, 1).final_geometry.content_bounds;
            assert_eq!(final_bounds.origin(), p(-3.0, -3.0));
            assert_eq!(final_bounds.size(), Size::new(n(6.0), n(7.0)));
            for frame in 0..2 {
                let leaf = actual(&scene, 2, frame);
                assert_eq!(leaf.unrounded.output().location, p(-2.5, -2.75));
                assert_eq!(leaf.final_entry.output().location, p(-3.0, -3.0));
                assert_eq!(leaf.final_entry.output().size, Size::new(n(6.0), n(7.0)));
            }
            let SceneOutput {
                appearances,
                metadata,
                receipts,
                clears,
            } = scene;
            assert!(receipts.is_empty());
            assert_eq!(
                clears
                    .iter()
                    .map(LayoutCacheClearEntry::node)
                    .collect::<Vec<_>>(),
                [3]
            );
            let batch: CompletedLayoutBatchOf<u32, S> =
                crate::block_fragmentation::scope_assembly::assemble::<
                    u32,
                    S,
                    std::convert::Infallible,
                >(crate::block_fragmentation::scope_assembly::AssemblyInput {
                    root: 0,
                    inventory: environment.inventory,
                    appearances,
                    metadata,
                    frames: environment.frames,
                    axes: axes(),
                    kind: FragmentationKind::Page,
                    target: environment.target.clone(),
                    invalidated: vec![2],
                })
                .unwrap();
            assert_eq!(batch.target_basis(), Some(&environment.target));
            assert!(batch.replaces_whole_root());
            assert_eq!(batch.fragments().count(), 6);
            assert_eq!(
                batch.node_layout(3).unwrap().non_box().unwrap().kind(),
                NonBoxKind::Hidden
            );
            // Scene's performed hidden clear remains separate from the
            // caller's explicit invalidation/publication decision.
            assert_eq!(
                batch
                    .cache_clear_entries()
                    .iter()
                    .map(LayoutCacheClearEntry::node)
                    .collect::<Vec<_>>(),
                [2]
            );
            assert_eq!(batch.invalidated_nodes(), [2]);
            let roots = batch.node_layout(0).unwrap().fragments().unwrap();
            let parents = batch.node_layout(1).unwrap().fragments().unwrap();
            let leaves = batch.node_layout(2).unwrap().fragments().unwrap();
            for family in [roots, parents, leaves] {
                assert_eq!(family.len(), 2);
                assert_eq!(family[0].continuation_order(), 0);
                assert_eq!(family[1].continuation_order(), 1);
                assert_eq!(family[0].next(), Some(family[1].id()));
                assert_eq!(family[1].previous(), Some(family[0].id()));
                assert_eq!(
                    family[0].fragmentainer(),
                    Some(batch.fragmentainers()[0].id())
                );
                assert_eq!(
                    family[1].fragmentainer(),
                    Some(batch.fragmentainers()[1].id())
                );
                assert_ne!(family[0].id(), family[1].id());
            }
            assert!(parents[0].principal_box().is_some());
            assert!(parents[1].principal_box().is_none());
            assert_eq!(
                parents[1]
                    .overflow_only()
                    .unwrap()
                    .unrounded()
                    .content_bounds,
                raw_bounds
            );
            assert_eq!(
                parents[1]
                    .overflow_only()
                    .unwrap()
                    .final_geometry()
                    .content_bounds,
                final_bounds
            );
            for frame in 0..2 {
                assert_eq!(
                    parents[frame].coordinate_parent(),
                    Some(&FragmentCoordinateSpace::Fragment(
                        roots[frame].id().clone()
                    ))
                );
                assert_eq!(
                    leaves[frame].coordinate_parent(),
                    Some(&FragmentCoordinateSpace::Fragment(
                        parents[frame].id().clone()
                    ))
                );
                assert_eq!(
                    batch
                        .child_fragments(roots[frame].id())
                        .unwrap()
                        .map(|child| child.id())
                        .collect::<Vec<_>>(),
                    [parents[frame].id()]
                );
                assert_eq!(
                    batch
                        .child_fragments(parents[frame].id())
                        .unwrap()
                        .map(|child| child.id())
                        .collect::<Vec<_>>(),
                    [leaves[frame].id()]
                );
                assert_eq!(
                    roots[frame]
                        .source_children()
                        .iter()
                        .map(|&(_, node)| node)
                        .collect::<Vec<_>>(),
                    [1, 3]
                );
                assert_eq!(
                    leaves[frame].principal_box().unwrap().unrounded().location,
                    p(-2.5, -2.75)
                );
                assert_eq!(
                    leaves[frame]
                        .principal_box()
                        .unwrap()
                        .final_geometry()
                        .location,
                    p(-3.0, -3.0)
                );
                assert_eq!(batch.fragment(leaves[frame].id()).unwrap().node(), 2);
            }
            let all = batch.fragments().collect::<Vec<_>>();
            for (index, fragment) in all.iter().enumerate() {
                assert!(all[..index].iter().all(|other| other.id() != fragment.id()));
            }
        });
    }
    fn atomic_subtree_stays_in_one_accepted_packet<S: LayoutScalar>() {
        let mut tree = tree::<S>();
        let mut parent = tree.node_input(1).clone();
        parent.writing_mode = WritingMode::VerticalRl;
        parent.size = Size::new(PreferredSizeOf::px(n(12.5)), PreferredSizeOf::px(n(20.25)));
        tree.insert_input(1, LayoutInputOf::box_input(parent));
        let mut leaf = tree.node_input(2).clone();
        leaf.writing_mode = WritingMode::VerticalRl;
        tree.insert_input(2, LayoutInputOf::box_input(leaf));
        with_environment(&tree, |environment| {
            let owner = environment
                .inventory
                .nodes
                .iter()
                .position(|source| source.node == 1)
                .unwrap();
            let descendant = environment
                .inventory
                .nodes
                .iter()
                .find(|source| source.node == 2)
                .unwrap();
            assert_eq!(descendant.atomic_owner, Some(owner));
            let packet = produce(environment, &[0, 1, 2], 0);
            let original_owner = *packet
                .packet
                .entries
                .iter()
                .find(|entry| entry.node() == 1)
                .unwrap();
            let original_child = *packet
                .packet
                .entries
                .iter()
                .find(|entry| entry.node() == 2)
                .unwrap();
            let scene = Scene::new(environment, vec![packet], |_, _| {
                panic!("a complete atomic subtree needs no carrier")
            })
            .unwrap()
            .round()
            .unwrap();
            assert_eq!(actual(&scene, 1, 0).unrounded, original_owner);
            assert_eq!(actual(&scene, 2, 0).unrounded, original_child);
            assert_parent(&scene, 2, 1, 0);
            assert_parent(&scene, 1, 0, 0);
            assert!(
                scene.appearances.iter().all(|appearance| matches!(
                    appearance.payload,
                    FragmentPayloadOf::ActualBox(_)
                ))
            );
            let parent_packet = select(produce(environment, &[0, 1, 2], 0), &[0, 1]);
            let child_packet = select(produce(environment, &[0, 1, 2], 0), &[2]);
            let error = match Scene::new(environment, vec![parent_packet, child_packet], |_, _| {
                panic!("split atomic custody is rejected before origins")
            }) {
                Ok(_) => {
                    panic!("an atomic owner and descendant cannot be rounded as separate packets")
                }
                Err(error) => error,
            };
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidSource)
            ));
        });
    }
    fn malformed_bindings_fail_without_an_output_scene<S: LayoutScalar>() {
        with_environment(&tree::<S>(), |environment| {
            for invalid in 0..7 {
                let mut first = produce(environment, &[0], 0);
                match invalid {
                    0 => first.frame = environment.frames.len(),
                    1 => {
                        first.packet.target = LayoutBoundRequestOf::continuous(
                            LayoutSourceBasis::fresh(),
                            environment.template,
                        )
                        .target_basis()
                    }
                    2 => first.graphical.push((
                        2,
                        FragmentationGraphicalCoverageOf {
                            start: n(0.0),
                            end: n(1.0),
                            extent: n(1.0),
                        },
                    )),
                    3 => first.graphical.push((
                        0,
                        FragmentationGraphicalCoverageOf {
                            start: n(1.0),
                            end: n(1.0),
                            extent: n(2.0),
                        },
                    )),
                    5 => {
                        let old = first.packet.entries[0];
                        let crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } =
                            old.geometry_state()
                        else {
                            unreachable!()
                        };
                        first.packet.entries[0] = LayoutOutputEntryOf::ordinary_unrounded(
                            99,
                            measured_normal,
                            old.output(),
                        );
                    }
                    6 => first.packet.clears.push(LayoutCacheClearEntry::new(99)),
                    _ => {}
                }
                let packets = if invalid == 4 {
                    vec![first, produce(environment, &[0], 0)]
                } else {
                    vec![first]
                };
                let error = match Scene::new(environment, packets, |_, _| {
                    panic!("bad binding is rejected before origins")
                }) {
                    Ok(_) => panic!("malformed binding must fail"),
                    Err(error) => error,
                };
                assert!(matches!(
                    error.kind(),
                    LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidSource)
                ));
            }
            let child = select(produce(environment, &[0, 1, 2], 0), &[2]);
            let error = match Scene::new(environment, vec![child], |_, _| Ok(p(f64::NAN, 0.0))) {
                Ok(_) => panic!("nonfinite issued origin must fail"),
                Err(error) => error,
            };
            assert!(matches!(
                error.kind(),
                LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidGeometry)
            ));
            let mut canonical = produce(environment, &[0], 0);
            let finalized = produce(environment, &[0], 0)
                .packet
                .round(
                    environment.capture.tree,
                    &[(
                        0,
                        crate::engine::FragmentAnchor::external(environment.frames[0].origin),
                    )],
                )
                .unwrap();
            canonical.packet.entries = finalized
                .entries
                .into_iter()
                .map(|pair| pair.final_entry)
                .collect();
            assert!(
                Scene::new(environment, vec![canonical], |_, _| panic!(
                    "wrong phase has no origin demand"
                ))
                .is_err()
            );
        });
    }
    #[test]
    fn scope_scene_reuses_actual_ancestor_before_rounding_f32() {
        actual_ancestor_receives_descendant_before_rounding::<f32>();
    }
    #[test]
    fn scope_scene_reuses_actual_ancestor_before_rounding_f64() {
        actual_ancestor_receives_descendant_before_rounding::<f64>();
    }
    #[test]
    fn scope_scene_inserts_only_missing_principal_carrier_f32() {
        missing_principal_gets_one_carrier_with_issued_origin::<f32>();
    }
    #[test]
    fn scope_scene_inserts_only_missing_principal_carrier_f64() {
        missing_principal_gets_one_carrier_with_issued_origin::<f64>();
    }
    #[test]
    fn scope_scene_nested_carriers_share_rounding_f32() {
        nested_carriers_share_the_actual_rounding_chain::<f32>();
    }
    #[test]
    fn scope_scene_nested_carriers_share_rounding_f64() {
        nested_carriers_share_the_actual_rounding_chain::<f64>();
    }
    #[test]
    fn scope_scene_rejects_malformed_bindings_atomically_f32() {
        malformed_bindings_fail_without_an_output_scene::<f32>();
    }
    #[test]
    fn scope_scene_rejects_malformed_bindings_atomically_f64() {
        malformed_bindings_fail_without_an_output_scene::<f64>();
    }
    #[test]
    fn scope_scene_zero_sized_principal_stays_actual_f32() {
        zero_sized_principal_remains_actual::<f32>();
    }
    #[test]
    fn scope_scene_zero_sized_principal_stays_actual_f64() {
        zero_sized_principal_remains_actual::<f64>();
    }
    #[test]
    fn scope_scene_empty_content_has_effects_without_geometry_f32() {
        empty_content_packet_preserves_effects_without_geometry::<f32>();
    }
    #[test]
    fn scope_scene_empty_content_has_effects_without_geometry_f64() {
        empty_content_packet_preserves_effects_without_geometry::<f64>();
    }
    #[test]
    fn scope_scene_signed_offsets_survive_public_graph_assembly_f32() {
        signed_visual_packets_compose_into_public_fragment_graph::<f32>();
    }
    #[test]
    fn scope_scene_signed_offsets_survive_public_graph_assembly_f64() {
        signed_visual_packets_compose_into_public_fragment_graph::<f64>();
    }
    #[test]
    fn scope_scene_perpendicular_atomic_subtree_keeps_packet_custody_f32() {
        atomic_subtree_stays_in_one_accepted_packet::<f32>();
    }
    #[test]
    fn scope_scene_perpendicular_atomic_subtree_keeps_packet_custody_f64() {
        atomic_subtree_stays_in_one_accepted_packet::<f64>();
    }
}
