//! Non-consuming ordinary qualification and physical entry composition.
use super::*;

type EntryFacts<S> = (
    crate::block::entry::StartFact<S>,
    crate::block::content::Context<S>,
);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct QualificationId(usize);
pub(super) struct QualifiedBody<Node, S: LayoutScalar> {
    frame: usize,
    data: crate::engine::BlockScopeBodyFrame<Node, S>,
}
struct BodyRef<Node, S: LayoutScalar> {
    qualified: QualificationId,
    node: Node,
    output: ComputeOutputOf<S>,
}
#[derive(Clone, Copy)]
pub(super) struct QualificationUse<Node, S: LayoutScalar> {
    candidate: QualificationId,
    pending: crate::block::scope::Pending<Node, S>,
    target: Node,
    frame: usize,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ProductionInput<S: LayoutScalar> {
    Root(LayoutRootRequestOf<S>),
    Content(ComputeInputOf<S>),
}
pub(super) struct Production<Node, S: LayoutScalar> {
    pub(super) receipt: ReceiptId,
    pub(super) scope: ScopeId,
    pub(super) view: SourceViewId,
    pub(super) projection: Option<(ProjectionKey, crate::block::scope::Pending<Node, S>)>,
    pub(super) packet: Packet<Node, S>,
    pub(super) input: ProductionInput<S>,
    pub(super) carrier: Option<crate::block::carrier::PerformedPath<Node, S>>,
}
pub(super) struct ResolvedEntry<Node, S: LayoutScalar> {
    pub(super) placement: Placement<Node, S>,
    pub(super) context: crate::block::content::Context<S>,
}

impl<Tree: LayoutTree> Driver<'_, '_, '_, Tree> {
    fn preflight_body(&self, node: Tree::Node) -> ScopeResult<Tree, ()> {
        let source = self
            .scopes
            .source
            .nodes
            .iter()
            .find(|source| source.node == node)
            .ok_or_else(|| {
                self.environment
                    .failure(BlockFragmentationFailure::InvalidSource)
            })?;
        let available = if matches!(self.environment.kind, FragmentationKind::Page { .. }) {
            FragmentationBreakKinds::SPREAD
        } else {
            self.environment.kind.breaks()
        };
        // This is the original descendant census, not the partition's visible
        // Owner slots. Interior controls in an owned content scope remain here.
        for boundary in source.first + 1..source.end {
            let (before, after, _) = self
                .scopes
                .source
                .controls(self.environment.capture.tree, boundary);
            if crate::fragmentation::controls_require_break(&before, &after, available) {
                return Err(unsupported(node).into());
            }
        }
        Ok(())
    }
    fn body(
        &self,
        work: &Work<Tree::Node, Tree::Scalar>,
        pending: crate::block::scope::Pending<Tree::Node, Tree::Scalar>,
        frame: usize,
    ) -> ScopeResult<Tree, BodyRef<Tree::Node, Tree::Scalar>> {
        for (index, candidate) in self
            .qualifications
            .borrow()
            .iter()
            .enumerate()
            .filter(|(_, candidate)| candidate.frame == frame)
        {
            if let Some(body) = candidate.data.packet.bodies.iter().find(|body| {
                body.node == pending.node
                    && body.input == pending.input.with_run_mode(RunMode::PerformLayout)
            }) {
                return Ok(BodyRef {
                    qualified: QualificationId(index),
                    node: pending.node,
                    output: body.output,
                });
            }
        }
        self.preflight_body(pending.node)?;
        let bounds = *self.environment.frames.get(frame).ok_or_else(|| {
            self.environment
                .failure(BlockFragmentationFailure::MissingFragmentainer { index: frame })
        })?;
        let data = crate::engine::qualify_block_scope_body(
            self.environment.capture.tree,
            crate::engine::BlockScopeEntryRequest {
                pending,
                binding: crate::engine::BlockFrameBinding {
                    allocations: Vec::new(),
                    intrinsics: self.environment.intrinsics(bounds)?,
                    dispatch: self.dispatch(work),
                },
            },
        )?;
        let output = data.output;
        let candidate = QualifiedBody { frame, data };
        self.qualifications
            .borrow_mut()
            .try_reserve(1)
            .map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(pending.node),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
        let id = QualificationId(self.qualifications.borrow().len());
        self.qualifications.borrow_mut().push(candidate);
        Ok(BodyRef {
            qualified: id,
            node: pending.node,
            output,
        })
    }
    fn entry_fact(
        &self,
        work: &Work<Tree::Node, Tree::Scalar>,
        pending: crate::block::scope::Pending<Tree::Node, Tree::Scalar>,
        frame: usize,
        target: Tree::Node,
        uses: &mut Vec<QualificationUse<Tree::Node, Tree::Scalar>>,
    ) -> ScopeResult<Tree, EntryFacts<Tree::Scalar>> {
        let bounds = *self.environment.frames.get(frame).ok_or_else(|| {
            self.environment
                .failure(BlockFragmentationFailure::MissingFragmentainer { index: frame })
        })?;
        let (fact, context) = crate::engine::qualify_block_scope_entry(
            self.environment.capture.tree,
            crate::engine::BlockScopeEntryRequest {
                pending,
                binding: crate::engine::BlockFrameBinding {
                    allocations: Vec::new(),
                    intrinsics: self.environment.intrinsics(bounds)?,
                    dispatch: self.dispatch(work),
                },
            },
        )?;
        let fact = match fact {
            crate::block::entry::Qualification::Known(fact) => fact,
            crate::block::entry::Qualification::RequiresBody => {
                let body = self.body(work, pending, frame)?;
                self.use_body(uses, &body, pending, target, frame)?;
                crate::block::entry::StartFact {
                    margin: body
                        .output
                        .block_margin_collapse
                        .at(pending.axes.block_start()),
                }
            }
        };
        Ok((fact, context))
    }
    fn local_normal(
        &self,
        body: &BodyRef<Tree::Node, Tree::Scalar>,
        node: Tree::Node,
    ) -> ScopeResult<Tree, crate::output::NormalGeometryOf<Tree::Scalar>> {
        if node == body.node {
            return Ok(crate::output::NormalGeometryOf::box_geometry(
                Point::ZERO,
                body.output.size,
                body.output
                    .scroll_geometry
                    .ok_or_else(|| {
                        self.environment
                            .failure(BlockFragmentationFailure::InvalidGeometry)
                    })?
                    .content_box(),
            ));
        }
        self.qualifications.borrow()[body.qualified.0]
            .data
            .packet
            .entries
            .iter()
            .find(|entry| entry.node() == node)
            .copied()
            .map(super::NativeProvider::<Tree>::normal)
            .ok_or_else(|| {
                self.environment
                    .failure(BlockFragmentationFailure::InvalidSource)
                    .into()
            })
    }
    fn local_origin(
        &self,
        body: &BodyRef<Tree::Node, Tree::Scalar>,
        target: usize,
    ) -> ScopeResult<Tree, Point<Tree::Scalar>> {
        let mut index = target;
        let mut point = Point::ZERO;
        loop {
            let source = &self.scopes.source.nodes[index];
            if source.node == body.node {
                return Ok(point);
            }
            let normal = self.local_normal(body, source.node)?;
            point.x = point.x + normal.location.x;
            point.y = point.y + normal.location.y;
            index = source.parent.ok_or_else(|| {
                self.environment
                    .failure(BlockFragmentationFailure::InvalidSource)
            })?;
        }
    }
    fn inline_offset(
        &self,
        parent: crate::block::scope::Pending<Tree::Node, Tree::Scalar>,
        location: Tree::Scalar,
    ) -> Tree::Scalar {
        if parent.axes.inline_start() == self.environment.axes.inline_start() {
            location
        } else {
            parent.axes.logical_size(parent.reference).inline - location - parent.inline_outer
        }
    }
    pub(super) fn resolve_entry(
        &self,
        work: &Work<Tree::Node, Tree::Scalar>,
        query: FragmentationQueryOf<Tree::Scalar>,
        request: &crate::engine::block_scope::Request<Tree::Node, Tree::Scalar>,
    ) -> ScopeResult<Tree, ResolvedEntry<Tree::Node, Tree::Scalar>> {
        if request.scope != work.id {
            return Err(self
                .environment
                .failure(BlockFragmentationFailure::InvalidSource)
                .into());
        }
        let axes = self.environment.axes;
        let frame = self.environment.frames[query.fragmentainer_index];
        let base = work
            .placement
            .as_ref()
            .filter(|placement| placement.query.fragmentainer_index == query.fragmentainer_index);
        let mut origin = base.map_or(
            crate::geometry::LogicalPointOf::new(Tree::Scalar::ZERO, Tree::Scalar::ZERO),
            |placement| placement.origin,
        );
        let mut occupied = base.map_or(Tree::Scalar::ZERO, |placement| placement.outside);
        let mut uses = Vec::new();
        for caller in &request.frames {
            let Some(pending) = caller.entry else {
                continue;
            };
            if caller.node != pending.node || caller.input != pending.input {
                return Err(self
                    .environment
                    .failure(BlockFragmentationFailure::InvalidSource)
                    .into());
            }
            let own = FlowAxes::new(
                self.environment
                    .capture
                    .tree
                    .node_input(caller.node)
                    .writing_mode,
                self.environment
                    .capture
                    .tree
                    .node_input(caller.node)
                    .direction,
            );
            if own.block_start() != axes.block_start() {
                let body = self.body(work, pending, query.fragmentainer_index)?;
                self.use_body(
                    &mut uses,
                    &body,
                    pending,
                    request.entry.node,
                    query.fragmentainer_index,
                )?;
                let fact = crate::block::entry::StartFact {
                    margin: body
                        .output
                        .block_margin_collapse
                        .at(pending.axes.block_start()),
                };
                let placement = pending.resolve(fact)?;
                occupied = occupied.max(origin.block + pending.occupied);
                let root_logical = crate::geometry::LogicalPointOf::new(
                    origin.inline + self.inline_offset(pending, placement.location.inline),
                    origin.block + placement.location.block,
                );
                let root_physical = axes.physical_point(
                    root_logical,
                    axes.logical_size(body.output.size),
                    frame.size,
                );
                let target = self
                    .scopes
                    .source
                    .nodes
                    .iter()
                    .position(|source| source.node == request.entry.node)
                    .ok_or_else(|| {
                        self.environment
                            .failure(BlockFragmentationFailure::InvalidSource)
                    })?;
                let normal = self.local_normal(&body, request.entry.node)?;
                let relative = self.local_origin(&body, target)?;
                let physical =
                    Point::new(root_physical.x + relative.x, root_physical.y + relative.y);
                let logical = axes.logical_point(physical, normal.size, frame.size);
                let target_location = self.scopes.locations[target];
                for (index, source) in self.scopes.source.nodes.iter().enumerate() {
                    let location = self.scopes.locations[index];
                    if source.hidden
                        || location.scope != target_location.scope
                        || location.end > target_location.first
                    {
                        continue;
                    }
                    let Some(_) = self.qualifications.borrow()[body.qualified.0]
                        .data
                        .packet
                        .entries
                        .iter()
                        .find(|entry| entry.node() == source.node)
                    else {
                        continue;
                    };
                    let before = self.local_normal(&body, source.node)?;
                    let before_origin = self.local_origin(&body, index)?;
                    let absolute = Point::new(
                        root_physical.x + before_origin.x,
                        root_physical.y + before_origin.y,
                    );
                    let start = axes.logical_point(absolute, before.size, frame.size).block;
                    occupied = occupied.max(start + axes.logical_size(before.size).block);
                }
                let (reported, context) = self.entry_fact(
                    work,
                    request.entry,
                    query.fragmentainer_index,
                    request.entry.node,
                    &mut uses,
                )?;
                let local = request.entry.resolve(reported)?;
                self.retain_uses(work, request.entry.node, uses)?;
                return Ok(ResolvedEntry {
                    placement: Placement {
                        query,
                        entry: local,
                        origin: logical,
                        outside: occupied.max(Tree::Scalar::ZERO),
                    },
                    context,
                });
            }
            let (fact, _) = self.entry_fact(
                work,
                pending,
                query.fragmentainer_index,
                request.entry.node,
                &mut uses,
            )?;
            let placement = pending.resolve(fact)?;
            occupied = occupied.max(origin.block + pending.occupied);
            origin.inline = origin.inline + self.inline_offset(pending, placement.location.inline);
            origin.block = origin.block + placement.location.block;
        }
        let (fact, context) = self.entry_fact(
            work,
            request.entry,
            query.fragmentainer_index,
            request.entry.node,
            &mut uses,
        )?;
        let placement = request.entry.resolve(fact)?;
        occupied = occupied.max(origin.block + request.entry.occupied);
        origin.inline =
            origin.inline + self.inline_offset(request.entry, placement.location.inline);
        origin.block = origin.block + placement.location.block;
        if !origin.inline.is_finite() || !origin.block.is_finite() || !occupied.is_finite() {
            return Err(self
                .environment
                .failure(BlockFragmentationFailure::InvalidGeometry)
                .into());
        }
        self.retain_uses(work, request.entry.node, uses)?;
        Ok(ResolvedEntry {
            placement: Placement {
                query,
                entry: placement,
                origin,
                outside: occupied.max(Tree::Scalar::ZERO),
            },
            context,
        })
    }
}

impl<Tree: LayoutTree> Driver<'_, '_, '_, Tree> {
    fn use_body(
        &self,
        uses: &mut Vec<QualificationUse<Tree::Node, Tree::Scalar>>,
        body: &BodyRef<Tree::Node, Tree::Scalar>,
        pending: crate::block::scope::Pending<Tree::Node, Tree::Scalar>,
        target: Tree::Node,
        frame: usize,
    ) -> ScopeResult<Tree, ()> {
        if uses.iter().any(|entry| {
            entry.candidate == body.qualified
                && entry.pending == pending
                && entry.target == target
                && entry.frame == frame
        }) {
            return Ok(());
        }
        uses.try_reserve(1).map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(target),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::Allocation,
            )
        })?;
        uses.push(QualificationUse {
            candidate: body.qualified,
            pending,
            target,
            frame,
        });
        Ok(())
    }
    fn retain_uses(
        &self,
        work: &Work<Tree::Node, Tree::Scalar>,
        target: Tree::Node,
        uses: Vec<QualificationUse<Tree::Node, Tree::Scalar>>,
    ) -> ScopeResult<Tree, ()> {
        let mut retained = work.qualifications.borrow_mut();
        retained.try_reserve(uses.len()).map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(self.environment.capture.root),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::Allocation,
            )
        })?;
        retained.retain(|old| old.target != target);
        for entry in uses {
            if !retained.iter().any(|old| {
                old.candidate == entry.candidate
                    && old.pending == entry.pending
                    && old.target == entry.target
                    && old.frame == entry.frame
            }) {
                retained.push(entry);
            }
        }
        Ok(())
    }
}

fn child_packets<Node: Copy + Eq, S: LayoutScalar>(
    packet: &Packet<Node, S>,
) -> Option<Vec<ProductionId>> {
    if packet.receipts.len() != packet.projections.len() {
        return None;
    }
    Some(packet.projections.iter().map(|(_, id)| *id).collect())
}
pub(super) fn retain_production<Node: Copy + Eq, S: LayoutScalar, M>(
    arena: &ProductionArena<Node, S>,
    production: Production<Node, S>,
    root: Node,
) -> LayoutResultOf<Node, ProductionId, S, M> {
    if child_packets(&production.packet).is_none() {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(root),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::BlockFragmentation(BlockFragmentationFailure::InvalidSource),
        ));
    }
    let mut arena = arena.borrow_mut();
    arena.try_reserve(1).map_err(|_| {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(root),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::Allocation,
        )
    })?;
    let id = ProductionId(arena.len());
    arena.push(production);
    Ok(id)
}
impl<Tree: LayoutTree> Provider<'_, '_, '_, Tree> {
    pub(super) fn validate_qualifications(
        &self,
        packet: &Packet<Tree::Node, Tree::Scalar>,
        selection: &Selection<Tree::Scalar>,
        frame: usize,
    ) -> ScopeResult<Tree, ()> {
        let uses = self.qualification_uses.borrow();
        let qualifications = self.qualifications.borrow();
        let bodies = self.productions.borrow();
        for used in uses.iter().filter(|used| used.frame == frame) {
            let target = self
                .scopes
                .source
                .nodes
                .iter()
                .position(|source| source.node == used.target)
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
            let target_range = self
                .scopes
                .range(self.scope, target)
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
            if target_range.start >= selection.range.end
                || target_range.end <= selection.range.start
            {
                continue;
            }
            let root = self
                .scopes
                .source
                .nodes
                .iter()
                .position(|source| source.node == used.pending.node)
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
            let range = self
                .scopes
                .range(self.scope, root)
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
            if range.start < selection.range.start
                || range.end > selection.range.end
                || selection.partial.is_some_and(|unit| range.end == unit + 1)
            {
                return Err(unsupported(used.pending.node).into());
            }
            let qualified = qualifications
                .get(used.candidate.0)
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
            let mut actual: Vec<&crate::engine::block_scope::Body<Tree::Node, Tree::Scalar>> =
                Vec::new();
            actual.try_reserve(packet.bodies.len()).map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(used.pending.node),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
            actual.extend(packet.bodies.iter());
            let mut pending = child_packets(packet)
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
            let mut visited = Vec::new();
            while let Some(id) = pending.pop() {
                if visited.contains(&id) {
                    continue;
                }
                visited.push(id);
                let body = bodies
                    .get(id.0)
                    .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
                actual
                    .try_reserve(body.packet.bodies.len())
                    .and_then(|_| pending.try_reserve(body.packet.projections.len()))
                    .map_err(|_| {
                        LayoutErrorOf::new(
                            LayoutErrorSiteOf::Node(used.pending.node),
                            LayoutOperation::RootLayout,
                            LayoutErrorKindOf::Allocation,
                        )
                    })?;
                actual.extend(body.packet.bodies.iter());
                pending.extend(body.packet.projections.iter().map(|(_, id)| *id));
            }
            for expected in &qualified.data.packet.bodies {
                let mut ancestor = self
                    .scopes
                    .source
                    .nodes
                    .iter()
                    .position(|source| source.node == expected.node);
                let mut descendant = false;
                while let Some(index) = ancestor {
                    if index == root {
                        descendant = true;
                        break;
                    }
                    ancestor = self.scopes.source.nodes[index].parent;
                }
                if !descendant {
                    continue;
                }
                let matching: Vec<_> = actual
                    .iter()
                    .filter(|body| body.node == expected.node)
                    .collect();
                if matching.is_empty()
                    || matching.iter().any(|body| {
                        body.input != expected.input
                            || body.output.size != expected.output.size
                            || body.output.block_margin_collapse
                                != expected.output.block_margin_collapse
                    })
                {
                    return Err(self
                        .failure(BlockFragmentationFailure::InvalidGeometry)
                        .into());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "scope_entry_tests.rs"]
mod tests;
