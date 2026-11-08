//! Consume only productions reached through accepted frame and child identities.
use super::*;

fn carrier_path<Tree: LayoutTree>(
    environment: &NativeProvider<'_, '_, Tree>,
    scopes: &Scopes<Tree::Node>,
    source: usize,
    frame: usize,
    binding: crate::block::content_insets::ContentInsetBinding<Tree::Node, Tree::Scalar>,
) -> NativeResult<Tree, crate::block::carrier::PreparedPath<Tree::Node, Tree::Scalar>> {
    let mut path = Vec::new();
    let mut ancestor = Some(source);
    while let Some(index) = ancestor {
        path.push(scopes.source.nodes[index].node);
        ancestor = scopes.source.nodes[index].parent;
    }
    path.reverse();
    crate::engine::prepare_block_carrier_path(
        environment.capture.tree,
        &path,
        environment.request(environment.frames[frame]),
        environment.frames[frame],
        environment.intrinsics(environment.frames[frame])?,
        binding,
    )
}
fn window_binding<Tree: LayoutTree>(
    environment: &NativeProvider<'_, '_, Tree>,
    scopes: &Scopes<Tree::Node>,
    completed: &Completed<Tree::Node, Tree::Scalar>,
    receipt: ReceiptId,
    window: AcceptedContentWindowId,
    view: SourceViewId,
) -> crate::block::content_insets::ContentInsetBinding<Tree::Node, Tree::Scalar> {
    let accepted = &completed.frames[window.0];
    let root = scopes.source.nodes[scopes.scopes[completed.scope.0].owner].node;
    let style = environment.capture.tree.node_input(root);
    crate::block::content_insets::ContentInsetBinding {
        owner: root,
        role: style.block_generation,
        target: environment.target.clone(),
        placement: receipt,
        trial: crate::block::content_insets::ContentTrialId(view.0),
        frame: accepted.frame,
        axes: FlowAxes::new(style.writing_mode, style.direction),
        start: match accepted.selection.begin {
            Some(progress) => FragmentationCutOf::Graphical {
                anchor: FragmentationPosition {
                    occurrence: accepted.selection.range.start,
                    offset: 0,
                },
                progress,
            },
            None => FragmentationCutOf::Source(FragmentationPosition {
                occurrence: accepted.selection.range.start,
                offset: 0,
            }),
        },
        end: accepted.selection.cut,
    }
}

impl<Tree: LayoutTree> Driver<'_, '_, '_, Tree> {
    fn overflow_window(
        &self,
        completed: &Completed<Tree::Node, Tree::Scalar>,
        receipt: ReceiptId,
        window: AcceptedContentWindowId,
    ) -> ScopeResult<Tree, ProductionId> {
        if let Some((_, id)) = completed
            .overflow
            .borrow()
            .iter()
            .find(|(id, _)| *id == window)
        {
            return Ok(*id);
        }
        let accepted = &completed.frames[window.0];
        let scope = &self.scopes.scopes[completed.scope.0];
        let root = self.scopes.source.nodes[scope.owner].node;
        completed.placement.as_ref().ok_or_else(|| {
            self.environment
                .failure(BlockFragmentationFailure::InvalidSource)
        })?;
        let frame = self.environment.frames[accepted.frame];
        let intrinsics = self.environment.intrinsics(frame)?;
        let input = intrinsics
            .entries
            .iter()
            .find(|entry| entry.node == root)
            .map(|entry| entry.input.with_run_mode(RunMode::PerformLayout))
            .ok_or_else(|| unsupported(root))?;
        let key = ViewKey {
            scope: completed.scope,
            frame: accepted.frame,
            range: accepted.selection.range.clone(),
            partial: accepted.selection.partial,
            resumed: accepted.selection.resumed,
            opening: false,
            closing: Some(false),
            trimmed: self
                .views
                .borrow()
                .get(self.productions.borrow()[accepted.production.0].view)
                .ok_or_else(|| {
                    self.environment
                        .failure(BlockFragmentationFailure::InvalidSource)
                })?
                .trimmed,
        };
        let view_id = self.views.borrow_mut().issue(key.clone()).map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(root),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::Allocation,
            )
        })?;
        let view = ScopeView::new(
            self.environment.capture.tree,
            self.scopes,
            &key,
            self.environment.axes,
            &margins::owners(self.scopes, &self.views.borrow().trim_edges(key.trimmed)),
        );
        let binding = window_binding(
            self.environment,
            self.scopes,
            completed,
            receipt,
            window,
            view_id,
        );
        let candidate = carrier_path(
            self.environment,
            self.scopes,
            scope.owner,
            accepted.frame,
            binding.clone(),
        )?;
        let produced = crate::engine::compute_block_content_frame(
            &view,
            crate::engine::BlockContentFrameRequest {
                root,
                input,
                original: self.environment.capture.tree.node_input(root),
                reference: Tree::Scalar::ZERO,
                opening: false,
                complete: accepted.selection.range.end == scope.units.len()
                    && accepted.selection.partial.is_none(),
                principal_content: None,
                accepted: Some(binding),
                carrier: Some(candidate),
                binding: crate::engine::BlockFrameBinding {
                    allocations: accepted.allocations.clone(),
                    intrinsics,
                    dispatch: Dispatch {
                        scope: receipt,
                        view: view_id,
                        target: self.environment.target.clone(),
                        owners: self.owners.clone(),
                        paths: self.paths.clone(),
                        ready: completed.ready.clone(),
                        requests: self.requests.clone(),
                    },
                },
            },
        )?;
        completed
            .overflow
            .borrow_mut()
            .try_reserve(1)
            .map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(root),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
        let id = entry::retain_production::<_, _, Tree::MeasureError>(
            &self.productions,
            entry::Production {
                receipt,
                scope: completed.scope,
                view: view_id,
                projection: None,
                input: ProductionInput::Content(input),
                carrier: produced.carrier,
                packet: produced.packet,
            },
            root,
        )?;
        completed.overflow.borrow_mut().push((window, id));
        Ok(id)
    }

    pub(super) fn complete_overflow(&self, current: usize) -> ScopeResult<Tree, ()> {
        for frame in &self.stack[current].progress.borrow().completed {
            let arena = self.productions.borrow();
            let production = arena.get(frame.production.0).ok_or_else(|| {
                self.environment
                    .failure(BlockFragmentationFailure::InvalidSource)
            })?;
            if production.packet.receipts != frame.receipts
                || production.packet.projections != frame.projections
            {
                return Err(self
                    .environment
                    .failure(BlockFragmentationFailure::InvalidSource)
                    .into());
            }
        }
        let mut roots: Vec<_> = self.stack[current]
            .progress
            .borrow()
            .completed
            .iter()
            .map(|frame| frame.production)
            .collect();
        loop {
            let mut pending = roots.clone();
            let mut visited = Vec::new();
            let mut receipts = Vec::new();
            let mut windows = Vec::new();
            while let Some(id) = pending.pop() {
                if visited.contains(&id) {
                    continue;
                }
                visited.push(id);
                let arena = self.productions.borrow();
                let production = arena.get(id.0).ok_or_else(|| {
                    self.environment
                        .failure(BlockFragmentationFailure::InvalidSource)
                })?;
                pending.extend(production.packet.projections.iter().map(|(_, id)| *id));
                if let Some((key, _)) = production.projection {
                    if !receipts.contains(&production.receipt) {
                        receipts.push(production.receipt);
                    }
                    if let Some(window) = key.window {
                        windows.push((production.receipt, window));
                    }
                }
            }
            let mut added = false;
            for receipt in receipts {
                let record = self
                    .records
                    .borrow()
                    .get(receipt.0)
                    .and_then(Option::as_ref)
                    .cloned()
                    .ok_or_else(|| {
                        self.environment
                            .failure(BlockFragmentationFailure::InvalidSource)
                    })?;
                let last = record
                    .timeline
                    .as_ref()
                    .and_then(|timeline| timeline.spans.last())
                    .map(|span| span.frame)
                    .ok_or_else(|| {
                        self.environment
                            .failure(BlockFragmentationFailure::InvalidSource)
                    })?;
                for (index, frame) in record.frames.iter().enumerate() {
                    let window = AcceptedContentWindowId(index);
                    if frame.frame > last && !windows.contains(&(receipt, window)) {
                        let id = self.overflow_window(&record, receipt, window)?;
                        if !roots.contains(&id) {
                            roots.push(id);
                            added = true;
                        }
                    }
                }
            }
            if !added {
                break;
            }
        }

        Ok(())
    }
}

pub(in crate::block_fragmentation::compute) fn compute<Tree: LayoutTree>(
    environment: &NativeProvider<'_, '_, Tree>,
    scopes: &Scopes<Tree::Node>,
    invalidated: Vec<Tree::Node>,
) -> NativeResult<Tree, CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>> {
    let resolved = Driver::new(environment, scopes)?.run()?;
    assemble_resolved(environment, scopes, resolved, invalidated)
}

pub(super) fn assemble_resolved<Tree: LayoutTree>(
    environment: &NativeProvider<'_, '_, Tree>,
    scopes: &Scopes<Tree::Node>,
    resolved: work::ResolvedScopes<Tree::Node, Tree::Scalar>,
    invalidated: Vec<Tree::Node>,
) -> NativeResult<Tree, CompletedLayoutBatchOf<Tree::Node, Tree::Scalar>> {
    let records = resolved.records.borrow();
    let root = records[resolved.root.0]
        .as_ref()
        .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
    let mut pending: Vec<_> = root
        .frames
        .iter()
        .rev()
        .map(|frame| frame.production)
        .collect();
    let mut adopted = Vec::new();
    let mut expanded = Vec::new();
    let mut carriers = Vec::new();
    let views = resolved.views.borrow();
    let mut arena = resolved.productions.borrow_mut();
    while let Some(id) = pending.pop() {
        if adopted.contains(&id) {
            return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
        }
        let production = arena
            .get(id.0)
            .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
        let view = views
            .get(production.view)
            .filter(|view| view.scope == production.scope)
            .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
        let input_valid = match production.input {
            ProductionInput::Root(request) => {
                !scopes.scopes[production.scope.0].contents
                    && production.receipt == resolved.root
                    && request == environment.request(environment.frames[view.frame])
            }
            ProductionInput::Content(input) => {
                scopes.scopes[production.scope.0].contents
                    && production
                        .projection
                        .is_none_or(|(_, pending)| pending.input == input)
            }
        };
        if !input_valid || production.packet.target != environment.target {
            return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
        }
        if let Some((key, entry)) = production.projection {
            let record = records
                .get(production.receipt.0)
                .and_then(Option::as_ref)
                .filter(|record| record.scope == production.scope)
                .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
            if key.frame != view.frame
                || scopes.source.nodes[scopes.scopes[production.scope.0].owner].node != entry.node
            {
                return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
            }
            if let Some(window) = key.window {
                let accepted = record
                    .frames
                    .get(window.0)
                    .filter(|accepted| accepted.frame == key.frame)
                    .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
                if accepted.selection.range != view.range
                    || accepted.selection.partial != view.partial
                    || accepted.selection.resumed != view.resumed
                {
                    return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
                }
            } else if !view.range.is_empty() {
                return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
            }
            // The parent owns the staged principal and its parent-local visual
            // location. Block's content materializer retains only its body fact.
            if production
                .packet
                .entries
                .iter()
                .any(|output| output.node() == entry.node)
            {
                return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
            }
            if !expanded.contains(&production.receipt) {
                expanded.push(production.receipt);
                for &(_, child) in record.overflow.borrow().iter().rev() {
                    pending.push(child);
                }
            }
        }
        if production.packet.receipts.len() != production.packet.projections.len() {
            return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
        }
        for ((node, child), receipt) in production
            .packet
            .projections
            .iter()
            .zip(&production.packet.receipts)
            .rev()
        {
            let child_production = arena
                .get(child.0)
                .filter(|child| child.receipt == *receipt)
                .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
            let child_view = views
                .get(child_production.view)
                .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
            if child_view.frame != view.frame
                || !child_production
                    .projection
                    .is_some_and(|(_, entry)| entry.node == *node)
                || !production
                    .packet
                    .entries
                    .iter()
                    .any(|entry| entry.node() == *node)
            {
                return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
            }
            pending.push(*child);
        }
        adopted.push(id);
    }
    let mut owned: Vec<_> = std::mem::take(&mut *arena).into_iter().map(Some).collect();
    drop(arena);
    let mut packets = Vec::new();
    packets
        .try_reserve(adopted.len())
        .and_then(|_| carriers.try_reserve(adopted.len()))
        .map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(environment.capture.root),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::Allocation,
            )
        })?;
    for id in adopted {
        let production = owned[id.0]
            .take()
            .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
        let view = views
            .get(production.view)
            .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
        let report = records[production.receipt.0]
            .as_ref()
            .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?
            .report
            .report();
        let mut graphical = Vec::new();
        for piece in report
            .pieces
            .iter()
            .filter(|piece| piece.fragmentainer == view.frame)
        {
            if let FragmentationPieceKindOf::GraphicalSlice { anchor, coverage } = piece.kind {
                let source = scopes.scopes[production.scope.0]
                    .units
                    .get(anchor.occurrence)
                    .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?
                    .node();
                let node = scopes.source.nodes[source].node;
                if production
                    .packet
                    .entries
                    .iter()
                    .any(|entry| entry.node() == node)
                {
                    graphical.push((node, coverage));
                }
            }
        }
        if let Some(candidate) = production.carrier {
            let record = records[production.receipt.0]
                .as_ref()
                .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
            let window = record
                .overflow
                .borrow()
                .iter()
                .find(|(_, actual)| *actual == id)
                .map(|(window, _)| *window)
                .ok_or_else(|| environment.failure(BlockFragmentationFailure::InvalidSource))?;
            let expected = window_binding(
                environment,
                scopes,
                record,
                production.receipt,
                window,
                production.view,
            );
            let accepted = candidate.adopt(
                environment.capture.tree,
                id,
                &expected,
                environment.request(environment.frames[view.frame]),
                environment.frames[view.frame],
                &environment.intrinsics(environment.frames[view.frame])?,
            )?;
            carriers.push((view.frame, id, expected, accepted));
        }
        packets.push(scene::ScopePacket {
            frame: view.frame,
            packet: production.packet,
            graphical,
        });
    }
    drop(views);
    drop(records);
    let scene = scene::Scene::new(environment, packets, |source, frame, parent| {
        let node = scopes.source.nodes[source].node;
        let profile = if scopes.owned[source].is_some() {
            super::super::super::source::ContinuationProfile::Maximum
        } else {
            super::super::super::source::ContinuationProfile::Minimum
        };
        if super::super::super::source::unsupported_continuation(
            environment.capture.tree.node_input(node),
            environment.axes,
            profile,
        )
        .map_err(|error| crate::error::sizing_resolution_error(node, error))?
        {
            return Err(unsupported(node));
        }
        let (_, production, binding, accepted) = carriers
            .iter()
            .find(|(owner_frame, _, _, path)| *owner_frame == frame && path.contains(node))
            .ok_or_else(|| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::ValueResolution,
                    LayoutErrorKindOf::MissingContext(LayoutMissingContext::RequiredBasis),
                )
            })?;
        crate::engine::settle_block_carrier_origin(
            environment.capture.tree,
            accepted,
            *production,
            binding,
            node,
            parent,
        )
    })?
    .round()?;
    if scene
        .receipts
        .iter()
        .any(|receipt| !expanded.contains(receipt))
        || scene.clears.iter().any(|clear| {
            !scopes
                .source
                .nodes
                .iter()
                .any(|source| source.node == clear.node())
        })
    {
        return Err(environment.failure(BlockFragmentationFailure::InvalidSource));
    }
    super::super::super::scope_assembly::assemble(
        super::super::super::scope_assembly::AssemblyInput {
            root: environment.capture.root,
            inventory: &scopes.source,
            appearances: scene.appearances,
            metadata: scene.metadata,
            frames: environment.frames,
            axes: environment.axes,
            kind: environment.kind,
            target: environment.target.clone(),
            invalidated,
        },
    )
}
