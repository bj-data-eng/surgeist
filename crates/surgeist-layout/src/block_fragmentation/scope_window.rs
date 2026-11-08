//! Mode-owned discovery and accepted-frame accounting.
use super::*;

pub(super) struct Discovery<Node, S: LayoutScalar> {
    query: FragmentationQueryOf<S>,
    generation: u64,
    selection: Selection<S>,
    minima: Vec<(Node, S)>,
    end: usize,
    boundary: usize,
    tail: usize,
    extent: S,
    head: S,
    head_end: usize,
    opportunities: Vec<FragmentationOpportunityOf<S>>,
    preview: Option<Produced<Node, S>>,
    reply: Option<FragmentationReplyOf<S>>,
}
impl<Node, S: LayoutScalar> Discovery<Node, S> {
    fn matches(&self, mut query: FragmentationQueryOf<S>, generation: u64) -> bool {
        query.stage = self.query.stage;
        self.query == query && self.generation == generation
    }
}

impl<Tree: LayoutTree> FragmentationProvider<Tree::Scalar> for Provider<'_, '_, '_, Tree> {
    type Error = ComputeStop<Tree::Node, Tree::Scalar, Tree::MeasureError>;
    fn fragmentainer(
        &mut self,
        _: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<Tree::Scalar>>, Self::Error> {
        Ok(self
            .environment
            .frames
            .get(index)
            .map(|frame| FragmentainerOf {
                inline_size: self.environment.axes.logical_size(frame.size).inline,
                block_size: self.environment.axes.logical_size(frame.size).block,
                breaks: frame.breaks,
                parent_fragmentainer: None,
                page_side: frame.page_side,
            }))
    }
    fn layout(
        &mut self,
        query: FragmentationQueryOf<Tree::Scalar>,
    ) -> Result<FragmentationReplyOf<Tree::Scalar>, Self::Error> {
        self.query.set(Some(query));
        if query.position.offset != 0 {
            return Err(self
                .failure(BlockFragmentationFailure::InvalidSource)
                .into());
        }
        let start = query.position.occurrence;
        let scope = &self.scopes.scopes[self.scope.0];
        let flow_end = scope.units.len();
        let restriction = self
            .bindings
            .borrow()
            .iter()
            .filter(|binding| {
                binding.frame == query.fragmentainer_index
                    && binding.initial_deferred
                    && binding.owner >= start
            })
            .map(|binding| binding.owner)
            .min();
        let (before, after, ancestors) =
            self.scopes
                .controls(self.environment.capture.tree, self.scope, start);
        let has_controls = |before: &[FragmentationBreakControl],
                            after: &[FragmentationBreakControl]| {
            before
                .iter()
                .chain(after)
                .any(|value| value.value != FragmentationBreak::Auto)
        };
        // Raw controls precede producer work even when a measured reply exists.
        if query.scan_after.is_none()
            && (has_controls(&before, &after) || restriction == Some(start))
        {
            return Ok(FragmentationReplyOf::ControlAtStart(
                FragmentationBoundaryControls {
                    kind: FragmentationControlKind::Sibling,
                    before,
                    after,
                },
            ));
        }
        let mut end = restriction
            .unwrap_or(flow_end)
            .min(self.discovery_limit(query)?);
        for boundary in start + 1..end {
            let position = FragmentationPosition {
                occurrence: boundary,
                offset: 0,
            };
            if query.scan_after.is_some_and(|old| position <= old) {
                continue;
            }
            let (before, after, _) =
                self.scopes
                    .controls(self.environment.capture.tree, self.scope, boundary);
            if has_controls(&before, &after) {
                end = boundary;
                break;
            }
        }
        let prior = self.discovery.borrow_mut().take();
        let mut discovery = match prior.filter(|state| state.matches(query, self.rebind_generation))
        {
            Some(mut state) => {
                if end < state.end {
                    // A newly ready initial deferral stops at the still-missing
                    // owner. Already measured preceding prefixes remain valid.
                    if state.boundary > end + 1 {
                        return Err(self
                            .failure(BlockFragmentationFailure::InvalidSource)
                            .into());
                    }
                    state.end = end;
                    state.reply = None;
                }
                state
            }
            None => {
                let state = self.progress.borrow();
                let (frame_start, resumed, provenance, begin) = state
                    .active
                    .as_ref()
                    .filter(|active| active.frame == query.fragmentainer_index)
                    .map_or(
                        (
                            start,
                            self.resumed_owner(query),
                            query.start_provenance,
                            query.graphical_progress,
                        ),
                        |active| {
                            (
                                active.selection.range.start,
                                active.selection.resumed,
                                active.selection.provenance,
                                active.selection.begin,
                            )
                        },
                    );
                drop(state);
                Discovery {
                    query,
                    generation: self.rebind_generation,
                    selection: Selection {
                        range: frame_start..start,
                        partial: None,
                        resumed,
                        provenance,
                        begin,
                        cut: FragmentationCutOf::Source(query.position),
                    },
                    minima: self.minimum_snapshot(query.fragmentainer_index)?,
                    end,
                    boundary: start + 1,
                    tail: start,
                    extent: Tree::Scalar::ZERO,
                    head: Tree::Scalar::ZERO,
                    head_end: start + 1,
                    opportunities: vec![FragmentationOpportunityOf {
                        cut: FragmentationCutOf::Source(query.position),
                        block_offset: Tree::Scalar::ZERO,
                        kind: FragmentationOpportunityKindOf::Sibling,
                        before,
                        after,
                        ancestors,
                    }],
                    preview: None,
                    reply: None,
                }
            }
        };
        let result = self.discover(query, &mut discovery);
        match result {
            Ok(reply) => {
                discovery.reply = Some(reply.clone());
                *self.discovery.borrow_mut() = Some(discovery);
                Ok(reply)
            }
            Err(stop @ ComputeStop::NeedScope(_)) => {
                *self.discovery.borrow_mut() = Some(discovery);
                Err(stop)
            }
            Err(stop) => Err(stop),
        }
    }
}

impl<Tree: LayoutTree> Provider<'_, '_, '_, Tree> {
    fn resumed_owner(&self, query: FragmentationQueryOf<Tree::Scalar>) -> Option<usize> {
        if query.graphical_progress.is_some() {
            return Some(query.position.occurrence);
        }
        self.bindings.borrow().iter().find_map(|binding| {
            (binding.owner == query.position.occurrence
                && binding.frame < query.fragmentainer_index
                && self.progress.borrow().completed.iter().any(|accepted| {
                    accepted.frame < query.fragmentainer_index
                        && accepted.selection.partial == Some(binding.owner)
                        && accepted.receipts.contains(&binding.receipt)
                }))
            .then_some(binding.owner)
        })
    }

    fn discovery_limit(
        &self,
        query: FragmentationQueryOf<Tree::Scalar>,
    ) -> ScopeResult<Tree, usize> {
        let scope = &self.scopes.scopes[self.scope.0];
        let mut end = scope.units.len();
        let bindings = self.bindings.borrow().clone();
        for binding in bindings {
            if binding.initial_deferred || binding.owner < query.position.occurrence {
                continue;
            }
            let child = self
                .records
                .borrow()
                .get(binding.receipt.0)
                .and_then(Option::as_ref)
                .cloned()
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
            let Some(timeline) = &child.timeline else {
                return Err(self
                    .failure(BlockFragmentationFailure::InvalidSource)
                    .into());
            };
            let continues = timeline.spans.iter().any(|span| {
                span.frame == query.fragmentainer_index && span.remaining > Tree::Scalar::ZERO
            }) && timeline
                .spans
                .iter()
                .any(|span| span.frame > query.fragmentainer_index);
            if !continues {
                continue;
            }
            let mut boundary = binding.owner + 1;
            if !matches!(scope.units.get(boundary), Some(Unit::PrincipalEnd(_))) {
                return Err(self
                    .failure(BlockFragmentationFailure::InvalidSource)
                    .into());
            }
            boundary += 1;
            while matches!(scope.units.get(boundary), Some(Unit::End(_))) {
                boundary += 1;
            }
            end = end.min(boundary);
        }
        Ok(end)
    }
    fn closing_gap(&self, principal_end: usize, frame: usize) -> ScopeResult<Tree, Tree::Scalar> {
        let Some(binding) = self
            .bindings
            .borrow()
            .iter()
            .find(|binding| binding.owner + 1 == principal_end)
            .copied()
        else {
            return Ok(Tree::Scalar::ZERO);
        };
        let completed = self
            .records
            .borrow()
            .get(binding.receipt.0)
            .and_then(Option::as_ref)
            .cloned()
            .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
        Ok(completed
            .timeline
            .as_ref()
            .and_then(|timeline| timeline.spans.iter().find(|span| span.frame == frame))
            .map_or(Tree::Scalar::ZERO, |span| span.gap))
    }
    fn discover(
        &self,
        query: FragmentationQueryOf<Tree::Scalar>,
        state: &mut Discovery<Tree::Node, Tree::Scalar>,
    ) -> ScopeResult<Tree, FragmentationReplyOf<Tree::Scalar>> {
        if let Some(reply) = &state.reply {
            return Ok(reply.clone());
        }
        let start = query.position.occurrence;
        let scope = &self.scopes.scopes[self.scope.0];
        while state.boundary <= state.end {
            let boundary = state.boundary;
            let mut selection = state.selection.clone();
            selection.range.end = boundary;
            selection.cut = FragmentationCutOf::Source(FragmentationPosition {
                occurrence: boundary,
                offset: 0,
            });
            let trial = self.measure(&selection, query.fragmentainer_index, &state.minima)?;
            state.end = state.end.min(self.discovery_limit(query)?);
            let absolute =
                self.extent(&trial, &selection, query.fragmentainer_index, &state.minima)?;
            let body = (absolute - query.used_block_offset).max(Tree::Scalar::ZERO);
            let attach = if boundary == start + 1 {
                true
            } else if boundary == state.head_end + 1 {
                match scope.units[state.head_end] {
                    Unit::End(index) => {
                        let node = self.scopes.source.nodes[index].node;
                        let minimum = state
                            .minima
                            .iter()
                            .find(|(key, _)| *key == node)
                            .map_or(Tree::Scalar::ZERO, |(_, value)| *value);
                        let content = state
                            .preview
                            .as_ref()
                            .map(|preview| self.content_extent(preview, node))
                            .transpose()?
                            .flatten();
                        content.is_some_and(|content| content >= minimum)
                    }
                    Unit::PrincipalEnd(_) => {
                        self.closing_gap(state.head_end, query.fragmentainer_index)?
                            == Tree::Scalar::ZERO
                    }
                    _ => false,
                }
            } else {
                false
            };
            let opportunity = if boundary < scope.units.len() {
                let (before, after, ancestors) =
                    self.scopes
                        .controls(self.environment.capture.tree, self.scope, boundary);
                let kind = match scope.units[boundary] {
                    Unit::End(index) => {
                        let node = self.scopes.source.nodes[index].node;
                        let minimum = state
                            .minima
                            .iter()
                            .find(|(key, _)| *key == node)
                            .map_or(Tree::Scalar::ZERO, |(_, value)| *value);
                        let content = self
                            .content_extent(&trial, node)?
                            .unwrap_or(Tree::Scalar::ZERO);
                        FragmentationOpportunityKindOf::ContentEdge {
                            gap: (minimum - content).max(Tree::Scalar::ZERO),
                        }
                    }
                    Unit::PrincipalEnd(_) => FragmentationOpportunityKindOf::ContentEdge {
                        gap: self.closing_gap(boundary, query.fragmentainer_index)?,
                    },
                    _ => FragmentationOpportunityKindOf::Sibling,
                };
                Some(FragmentationOpportunityOf {
                    cut: FragmentationCutOf::Source(FragmentationPosition {
                        occurrence: boundary,
                        offset: 0,
                    }),
                    block_offset: body,
                    kind,
                    before,
                    after,
                    ancestors,
                })
            } else {
                None
            };
            // Advance the cursor only after every fallible fact for this prefix
            // is complete; a suspension retains exactly the preceding prefix.
            state.extent = state.extent.max(body);
            if attach {
                state.head = body;
                state.head_end = boundary;
            }
            if let Some(opportunity) = opportunity {
                state.opportunities.push(opportunity);
            }
            state.preview = Some(trial);
            state.boundary += 1;
        }
        while state.tail < state.end {
            self.discover_tail(query, state)?;
            state.tail += 1;
        }
        self.owner_opportunities(query, &mut state.opportunities)?;
        if state.head_end > state.end {
            return Err(self
                .failure(BlockFragmentationFailure::InvalidSource)
                .into());
        }
        Ok(FragmentationReplyOf::Measured(FragmentationWindowOf {
            end: if state.end == scope.units.len() {
                FragmentationWindowEnd::FlowEnd
            } else {
                FragmentationWindowEnd::PotentialControl(FragmentationPosition {
                    occurrence: state.end,
                    offset: 0,
                })
            },
            block_size: state.extent,
            head: FragmentationMeasuredHeadOf {
                end: FragmentationPosition {
                    occurrence: state.head_end,
                    offset: 0,
                },
                block_size: state.head,
            },
            fallback: FragmentationFallback::Unsliceable,
            opportunities: state.opportunities.clone(),
            cloned_block_start: Tree::Scalar::ZERO,
            cloned_block_end: Tree::Scalar::ZERO,
            emergency_slice: None,
        }))
    }
    fn discover_tail(
        &self,
        query: FragmentationQueryOf<Tree::Scalar>,
        state: &mut Discovery<Tree::Node, Tree::Scalar>,
    ) -> ScopeResult<Tree, ()> {
        let unit = state.tail;
        let Unit::End(index) = self.scopes.scopes[self.scope.0].units[unit] else {
            return Ok(());
        };
        let key = self.scopes.source.nodes[index].node;
        let Some(&(_, minimum)) = state.minima.iter().find(|(node, _)| *node == key) else {
            return Ok(());
        };
        let mut prefix = state.selection.clone();
        prefix.range.end = unit;
        prefix.cut = FragmentationCutOf::Source(FragmentationPosition {
            occurrence: unit,
            offset: 0,
        });
        let packet = self.measure(&prefix, query.fragmentainer_index, &state.minima)?;
        let content = self
            .content_extent(&packet, key)?
            .unwrap_or(Tree::Scalar::ZERO);
        let gap = (minimum - content).max(Tree::Scalar::ZERO);
        let beginning = self.extent(&packet, &prefix, query.fragmentainer_index, &state.minima)?;
        let taken = query.used_block_offset + query.available_block_size - beginning;
        if taken <= Tree::Scalar::ZERO || taken >= gap {
            return Ok(());
        }
        let old = if unit == query.position.occurrence {
            query
                .graphical_progress
                .map_or(Tree::Scalar::ZERO, |progress| progress.consumed)
        } else {
            Tree::Scalar::ZERO
        };
        let total = self.progress.borrow().tails[unit].unwrap_or(old + gap);
        let (_, _, ancestors) =
            self.scopes
                .controls(self.environment.capture.tree, self.scope, unit);
        state.opportunities.push(FragmentationOpportunityOf {
            cut: FragmentationCutOf::Graphical {
                anchor: FragmentationPosition {
                    occurrence: unit,
                    offset: 0,
                },
                progress: FragmentationGraphicalProgressOf {
                    consumed: old + taken,
                    extent: total,
                },
            },
            block_offset: query.available_block_size,
            kind: FragmentationOpportunityKindOf::ContentEdge { gap },
            before: Vec::new(),
            after: Vec::new(),
            ancestors,
        });
        Ok(())
    }
    fn owner_opportunities(
        &self,
        query: FragmentationQueryOf<Tree::Scalar>,
        opportunities: &mut Vec<FragmentationOpportunityOf<Tree::Scalar>>,
    ) -> ScopeResult<Tree, ()> {
        let bindings = self.bindings.borrow().clone();
        for binding in bindings {
            if binding.initial_deferred || binding.owner < query.position.occurrence {
                continue;
            }
            let child = self
                .records
                .borrow()
                .get(binding.receipt.0)
                .and_then(Option::as_ref)
                .cloned()
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
            let Some(timeline) = &child.timeline else {
                return Err(self
                    .failure(BlockFragmentationFailure::InvalidSource)
                    .into());
            };
            let mut consumed = Tree::Scalar::ZERO;
            for span in &timeline.spans {
                consumed = consumed + span.opening + span.content;
                if span.frame != query.fragmentainer_index
                    || span.remaining == Tree::Scalar::ZERO
                    || consumed <= Tree::Scalar::ZERO
                    || consumed >= timeline.body_extent
                    || binding.owner == query.position.occurrence
                        && query
                            .graphical_progress
                            .is_some_and(|old| consumed <= old.consumed)
                {
                    continue;
                }
                let origin = child
                    .placement
                    .as_ref()
                    .filter(|placement| placement.query.fragmentainer_index == span.frame)
                    .map_or(Tree::Scalar::ZERO, |placement| placement.origin.block);
                let absolute = origin + span.opening + span.content;
                let (_, _, ancestors) =
                    self.scopes
                        .controls(self.environment.capture.tree, self.scope, binding.owner);
                opportunities.push(FragmentationOpportunityOf {
                    cut: FragmentationCutOf::Graphical {
                        anchor: FragmentationPosition {
                            occurrence: binding.owner,
                            offset: 0,
                        },
                        progress: FragmentationGraphicalProgressOf {
                            consumed,
                            extent: timeline.body_extent,
                        },
                    },
                    block_offset: (absolute - query.used_block_offset).max(Tree::Scalar::ZERO),
                    kind: FragmentationOpportunityKindOf::ContentEdge { gap: span.gap },
                    before: Vec::new(),
                    after: Vec::new(),
                    ancestors,
                });
            }
        }
        Ok(())
    }
}

fn accounted_minima<Node: Copy + Eq, S: LayoutScalar>(
    minima: &[(Node, S)],
    prior: &[(Node, S)],
    current: &[(Node, S)],
) -> Option<Vec<(Node, S)>> {
    minima
        .iter()
        .map(|&(node, spent)| {
            let old = prior
                .iter()
                .find(|(key, _)| *key == node)
                .map_or(S::ZERO, |(_, value)| *value);
            let now = current
                .iter()
                .find(|(key, _)| *key == node)
                .map_or(old, |(_, value)| *value);
            let spent = (spent - old).max(S::ZERO) + now;
            spent.is_finite().then_some((node, spent))
        })
        .collect()
}

impl<Tree: LayoutTree> Provider<'_, '_, '_, Tree> {
    pub(super) fn commit(
        &self,
        event: crate::fragmentation::AcceptedStep<'_, SourceKey<Tree::Node>, Tree::Scalar>,
    ) -> ScopeResult<Tree, ()> {
        if event.flow != 0 || (!event.breaks.is_empty()) != (event.from_frame != event.to_frame) {
            return Err(self
                .failure(BlockFragmentationFailure::InvalidSource)
                .into());
        }
        let query = self
            .query
            .get()
            .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
        let old = self.progress.borrow();
        let active = old
            .active
            .as_ref()
            .filter(|active| active.frame == event.from_frame);
        let prior_accounted = active
            .map(|active| active.accounted.clone())
            .unwrap_or_default();
        let mut selection = active.map(|active| active.selection.clone());
        for piece in event.pieces {
            if piece.fragmentainer != event.from_frame {
                return Err(self
                    .failure(BlockFragmentationFailure::InvalidSource)
                    .into());
            }
            let (end, partial, cut) = match piece.kind {
                FragmentationPieceKindOf::GraphicalSlice { anchor, coverage }
                    if coverage.end < coverage.extent =>
                {
                    (
                        anchor.occurrence + 1,
                        Some(anchor.occurrence),
                        FragmentationCutOf::Graphical {
                            anchor,
                            progress: FragmentationGraphicalProgressOf {
                                consumed: coverage.end,
                                extent: coverage.extent,
                            },
                        },
                    )
                }
                _ => (
                    piece.end.occurrence,
                    None,
                    FragmentationCutOf::Source(piece.end),
                ),
            };
            match &mut selection {
                Some(selection) => {
                    selection.range.end = end;
                    selection.partial = partial;
                    selection.cut = cut;
                }
                None => {
                    selection = Some(Selection {
                        range: piece.start.occurrence..end,
                        partial,
                        resumed: self.resumed_owner(query),
                        provenance: piece.start_provenance,
                        begin: query.graphical_progress,
                        cut,
                    })
                }
            }
        }
        for appearance in event.appearances {
            let checkpoint = appearance.checkpoint.source_position();
            let end = (appearance.owner.occurrence + 1).max(checkpoint.occurrence + 1);
            match &mut selection {
                Some(selection) => {
                    selection.range.end = end;
                    selection.partial = Some(end - 1);
                    selection.cut = appearance.checkpoint;
                }
                None => {
                    selection = Some(Selection {
                        range: query.position.occurrence..end,
                        partial: Some(end - 1),
                        resumed: self.resumed_owner(query),
                        provenance: query.start_provenance,
                        begin: query.graphical_progress,
                        cut: appearance.checkpoint,
                    })
                }
            }
        }
        drop(old);
        let Some(selection) = selection else {
            return Ok(());
        };
        let actual_cut = match event.graphical_progress {
            Some(progress) => FragmentationCutOf::Graphical {
                anchor: event.position,
                progress,
            },
            None => FragmentationCutOf::Source(event.position),
        };
        if selection.cut != actual_cut {
            return Err(self
                .failure(BlockFragmentationFailure::InvalidSource)
                .into());
        }
        for (index, source) in self.scopes.source.nodes.iter().enumerate() {
            let Some(local) = self.scopes.range(self.scope, index) else {
                continue;
            };
            let continuing = local.start < selection.range.end
                && local.end > selection.range.start
                && (local.start < selection.range.start
                    || local.end > selection.range.end
                    || selection.partial.is_some_and(|unit| local.end == unit + 1));
            if !continuing || source.atomic_owner.is_some() || self.scopes.owned[index].is_some() {
                continue;
            }
            if super::super::super::source::unsupported_continuation(
                self.environment.capture.tree.node_input(source.node),
                self.environment.axes,
                super::super::super::source::ContinuationProfile::Minimum,
            )
            .map_err(|error| crate::error::sizing_resolution_error(source.node, error))?
            {
                return Err(unsupported(source.node).into());
            }
        }
        let minima = self.minimum_snapshot(event.from_frame)?;
        let mut produced = self.measure(&selection, event.from_frame, &minima)?;
        for (node, start) in self.positions(&produced, event.from_frame)? {
            let index = self
                .scopes
                .source
                .nodes
                .iter()
                .position(|source| source.node == node)
                .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
            if self.scopes.source.nodes[index].atomic_owner.is_some()
                || self.scopes.owned[index].is_some()
                || minima.iter().any(|(key, _)| *key == node)
            {
                continue;
            }
            let style = self.environment.capture.tree.node_input(node);
            let axes = self.environment.axes;
            let constrained = axes.logical_size(style.min_size.clone()).block != MinSizeOf::AUTO
                || axes.logical_size(style.max_size.clone()).block != MaxSizeOf::NONE
                || style.aspect_ratio.is_some();
            let available = (axes
                .logical_size(self.environment.frames[event.from_frame].size)
                .block
                .max(Tree::Scalar::ONE)
                - start)
                .max(Tree::Scalar::ZERO);
            if constrained
                && produced
                    .output(node)
                    .is_some_and(|output| axes.logical_size(output.size).block > available)
            {
                return Err(unsupported(node).into());
            }
        }
        let leaving = event.from_frame != event.to_frame;
        let scope = &self.scopes.scopes[self.scope.0];
        let complete = selection.range.end == scope.units.len() && selection.partial.is_none();
        let mut allocations = self.allocations(&minima);
        if leaving {
            for (node, start) in self.positions(&produced, event.from_frame)? {
                let index = self
                    .scopes
                    .source
                    .nodes
                    .iter()
                    .position(|source| source.node == node)
                    .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidSource))?;
                let source = &self.scopes.source.nodes[index];
                let Some(local) = self.scopes.range(self.scope, index) else {
                    continue;
                };
                if source.atomic_owner.is_some() || self.scopes.owned[index].is_some() {
                    continue;
                }
                let continuing = local.end > selection.range.end
                    || selection.partial.is_some_and(|unit| local.end == unit + 1);
                if !continuing {
                    continue;
                }
                if super::super::super::source::unsupported_continuation(
                    self.environment.capture.tree.node_input(node),
                    self.environment.axes,
                    super::super::super::source::ContinuationProfile::Minimum,
                )
                .map_err(|error| crate::error::sizing_resolution_error(node, error))?
                {
                    return Err(unsupported(node).into());
                }
                let output = produced
                    .output(node)
                    .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidGeometry))?;
                let available = (self
                    .environment
                    .axes
                    .logical_size(self.environment.frames[event.from_frame].size)
                    .block
                    .max(Tree::Scalar::ONE)
                    - start)
                    .max(Tree::Scalar::ZERO);
                let used = self
                    .environment
                    .axes
                    .logical_size(output.size)
                    .block
                    .max(available);
                let known = self
                    .environment
                    .axes
                    .physical_size(crate::geometry::LogicalSizeOf::new(None, Some(used)));
                if let Some(allocation) = allocations
                    .iter_mut()
                    .find(|allocation| allocation.node == node)
                {
                    allocation.known = known;
                } else {
                    allocations.push(super::super::FragmentBoxAllocation {
                        node,
                        known,
                        indefinite_axis: self.environment.axes.block_axis(),
                        minimum_content: None,
                    });
                }
            }
            // A terminal child's measured extent remains fixed while broken
            // ancestors receive their accepted frame-fill allocation.
            for (index, source) in self.scopes.source.nodes.iter().enumerate().rev() {
                let Some(local) = self.scopes.range(self.scope, index) else {
                    continue;
                };
                if local.end <= selection.range.end
                    && !selection.partial.is_some_and(|unit| local.end == unit + 1)
                    && let Some(allocation) = allocations
                        .iter_mut()
                        .find(|allocation| allocation.node == source.node)
                    && allocation.known == Size::NONE
                    && let Some(output) = produced.output(source.node)
                {
                    allocation.known =
                        self.environment
                            .axes
                            .physical_size(crate::geometry::LogicalSizeOf::new(
                                None,
                                Some(self.environment.axes.logical_size(output.size).block),
                            ));
                }
            }
            produced = self.trial(
                &selection,
                event.from_frame,
                allocations.clone(),
                Tree::Scalar::ZERO,
            )?;
        }
        if !event.appearances.is_empty() {
            let peak = self.extent(&produced, &selection, event.from_frame, &minima)?;
            if event
                .appearances
                .iter()
                .any(|site| site.normal_end > site.occupied_end || site.occupied_end > peak)
            {
                return Err(self
                    .failure(BlockFragmentationFailure::InvalidGeometry)
                    .into());
            }
        }
        let summary = if let Some(content) = &produced.content {
            let participates = content.facts.participated || complete;
            let opening = if participates {
                content.context.opening()
            } else {
                Tree::Scalar::ZERO
            };
            let origin = self
                .placement
                .as_ref()
                .filter(|placement| placement.query.fragmentainer_index == event.from_frame)
                .map_or(Tree::Scalar::ZERO, |placement| placement.origin.block);
            let available = (self
                .environment
                .axes
                .logical_size(self.environment.frames[event.from_frame].size)
                .block
                .max(Tree::Scalar::ONE)
                - origin
                - opening)
                .max(Tree::Scalar::ZERO);
            let minimum = if complete {
                minima
                    .iter()
                    .find(|(node, _)| *node == self.root())
                    .map_or(Tree::Scalar::ZERO, |(_, value)| *value)
            } else {
                Tree::Scalar::ZERO
            };
            let natural = content.facts.block_extent;
            let allocated = if leaving && !complete && participates {
                natural.max(available)
            } else {
                natural.max(minimum)
            };
            if !allocated.is_finite() {
                return Err(self
                    .failure(BlockFragmentationFailure::InvalidGeometry)
                    .into());
            }
            Some(crate::block::content::AcceptedContent {
                frame: event.from_frame,
                natural,
                allocated,
                opening,
                participated: content.facts.participated,
                complete,
            })
        } else {
            None
        };
        let mut accounted = Vec::new();
        for &(node, _) in &minima {
            if node == self.root()
                && let Some(summary) = summary
            {
                accounted.push((node, summary.allocated));
            } else if let Some(content) = self.content_extent(&produced, node)? {
                accounted.push((node, content));
            }
        }
        let old = self.progress.borrow();
        let spent = accounted_minima(&old.minima, &prior_accounted, &accounted)
            .ok_or_else(|| self.failure(BlockFragmentationFailure::InvalidGeometry))?;
        let mut tails = old.tails.clone();
        for piece in event.pieces {
            if let FragmentationPieceKindOf::GraphicalSlice { anchor, coverage } = piece.kind {
                let Some(tail) = tails.get_mut(anchor.occurrence) else {
                    return Err(self
                        .failure(BlockFragmentationFailure::InvalidSource)
                        .into());
                };
                *tail = Some(coverage.extent);
            }
        }
        drop(old);
        if leaving || event.flow_complete {
            self.validate_qualifications(&produced.packet, &selection, event.from_frame)?;
        }
        let receipts = produced.packet.receipts.clone();
        let projections = produced.packet.projections.clone();
        // No mode accounting changes until every actual production and numeric
        // check above has succeeded, including a possible scope suspension.
        let mut progress = self.progress.borrow_mut();
        if leaving || event.flow_complete {
            progress.completed.try_reserve(1).map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(self.root()),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
        }
        let production = if leaving || event.flow_complete {
            Some(entry::retain_production::<_, _, Tree::MeasureError>(
                &self.productions,
                entry::Production {
                    receipt: self.dispatch.scope,
                    scope: self.scope,
                    view: produced.view,
                    projection: None,
                    input: produced.input,
                    carrier: None,
                    packet: produced.packet,
                },
                self.root(),
            )?)
        } else {
            None
        };
        progress.minima = spent;
        progress.tails = tails;
        if leaving || event.flow_complete {
            if summary.is_some_and(|summary| summary.participated || summary.complete) {
                progress.opened = true;
            }
            progress.completed.push(AcceptedFrame {
                frame: event.from_frame,
                selection,
                allocations,
                summary,
                receipts,
                projections,
                production: production.expect("completed frame retained its production"),
            });
            progress.active = None;
        } else {
            progress.active = Some(ActiveFrame {
                frame: event.from_frame,
                selection,
                accounted,
            });
        }
        drop(progress);
        *self.discovery.borrow_mut() = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimum_accounting<S: LayoutScalar>() {
        let minima = [(1, S::from_f64(13.0)), (2, S::from_f64(8.0))];
        let prior = [(1, S::from_f64(3.0)), (2, S::from_f64(2.0))];
        let current = [(1, S::from_f64(7.0))];
        let updated = accounted_minima(&minima, &prior, &current).unwrap();
        // Replacing this frame's accepted 3 by 7 adds 4, not another 7. Missing
        // current production retains the prior accepted amount for node 2.
        assert_eq!(updated, [(1, S::from_f64(17.0)), (2, S::from_f64(8.0))]);
        let next = accounted_minima(&updated, &[], &[(1, S::from_f64(4.0))]).unwrap();
        assert_eq!(next, [(1, S::from_f64(21.0)), (2, S::from_f64(8.0))]);
        assert!(accounted_minima(&minima, &prior, &[(1, S::from_f64(f64::NAN))]).is_none());
    }

    fn query<S: LayoutScalar>() -> FragmentationQueryOf<S> {
        FragmentationQueryOf {
            flow: 0,
            context: 0,
            fragmentainer_index: 0,
            fragmentainer: FragmentainerOf {
                inline_size: S::from_f64(100.0),
                block_size: S::from_f64(50.0),
                breaks: FragmentationBreakKinds::NONE,
                parent_fragmentainer: None,
                page_side: None,
            },
            position: FragmentationPosition {
                occurrence: 0,
                offset: 0,
            },
            graphical_progress: None,
            used_block_offset: S::ZERO,
            available_block_size: S::from_f64(50.0),
            stage: FragmentationStage::Strict,
            scan_after: None,
            start_provenance: FragmentationStart::FlowStart,
        }
    }

    fn discovery_binding<S: LayoutScalar>() {
        let original = query();
        let state = Discovery::<usize, S> {
            query: original,
            generation: 7,
            selection: Selection {
                range: 0..0,
                partial: None,
                resumed: None,
                provenance: FragmentationStart::FlowStart,
                begin: None,
                cut: FragmentationCutOf::Source(original.position),
            },
            minima: Vec::new(),
            end: 3,
            boundary: 2,
            tail: 0,
            extent: S::from_f64(30.0),
            head: S::from_f64(30.0),
            head_end: 1,
            opportunities: Vec::new(),
            preview: None,
            reply: None,
        };
        // A newly ready child does not change placement identity. Stage-only
        // relaxation also retains Native's already measured physical geometry.
        assert!(state.matches(original, 7));
        let mut changed = original;
        changed.stage = FragmentationStage::Emergency;
        assert!(state.matches(changed, 7));
        assert!(!state.matches(original, 8));
        changed = original;
        changed.used_block_offset = S::ONE;
        assert!(!state.matches(changed, 7));
        changed = original;
        changed.available_block_size = S::from_f64(40.0);
        assert!(!state.matches(changed, 7));
        changed = original;
        changed.fragmentainer.inline_size = S::from_f64(60.0);
        assert!(!state.matches(changed, 7));
        changed = original;
        changed.fragmentainer_index = 1;
        assert!(!state.matches(changed, 7));
        changed = original;
        changed.position.occurrence = 1;
        assert!(!state.matches(changed, 7));
        changed = original;
        changed.scan_after = Some(FragmentationPosition {
            occurrence: 1,
            offset: 0,
        });
        assert!(!state.matches(changed, 7));
        changed = original;
        changed.graphical_progress = Some(FragmentationGraphicalProgressOf {
            consumed: S::ONE,
            extent: S::from_f64(5.0),
        });
        assert!(!state.matches(changed, 7));
        changed = original;
        changed.start_provenance =
            FragmentationStart::AfterBreak(FragmentationBreakReason::Unforced);
        assert!(!state.matches(changed, 7));
    }

    #[test]
    fn accepted_minimum_frame_deltas_f32() {
        minimum_accounting::<f32>();
    }
    #[test]
    fn accepted_minimum_frame_deltas_f64() {
        minimum_accounting::<f64>();
    }
    #[test]
    fn discovery_reuses_only_bound_geometry_f32() {
        discovery_binding::<f32>();
    }
    #[test]
    fn discovery_reuses_only_bound_geometry_f64() {
        discovery_binding::<f64>();
    }
}

#[cfg(test)]
mod provider_tests {
    use super::*;
    use crate::test_support::layout_tree::PublicLayoutTreeOf;

    struct Tree<S: LayoutScalar> {
        inner: PublicLayoutTreeOf<S>,
        measured: RefCell<Vec<u32>>,
        reject: Cell<Option<u32>>,
    }
    impl<S: LayoutScalar> Traverse for Tree<S> {
        type Node = u32;
        type Scalar = S;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;
        fn children(&self, node: u32) -> Self::Children<'_> {
            Traverse::children(&self.inner, node)
        }
        fn child_count(&self, node: u32) -> usize {
            self.inner.child_count(node)
        }
        fn child(&self, node: u32, index: usize) -> u32 {
            self.inner.child(node, index)
        }
    }
    impl<S: LayoutScalar> LayoutTree for Tree<S> {
        type MeasureError = &'static str;
        fn node_input(&self, node: u32) -> &NodeInputOf<S> {
            self.inner.node_input(node)
        }
        fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
            self.inner.layout_input(node)
        }
        fn has_leaf_measurement(&self, node: u32) -> bool {
            self.inner.has_leaf_measurement(node)
        }
        fn measure_leaf(
            &self,
            node: u32,
            input: LeafMeasureInputOf<S>,
        ) -> Option<Result<Size<S>, Self::MeasureError>> {
            self.inner.measure_leaf(node, input).map(|size| {
                self.measured.borrow_mut().push(node);
                if self.reject.get() == Some(node) {
                    Err("rejected actual trial")
                } else {
                    Ok(size.unwrap())
                }
            })
        }
    }
    fn maximum_cap<S: LayoutScalar>() -> NodeInputOf<S> {
        NodeInputOf {
            display: Display::Block,
            max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::px(S::from_f64(1000.0))),
            ..Default::default()
        }
    }
    fn tree<S: LayoutScalar>(owned: bool) -> Tree<S> {
        let block = || NodeInputOf::<S> {
            display: Display::Block,
            ..Default::default()
        };
        let mut root = block();
        root.min_size.height = MinSizeOf::px(S::from_f64(50.0));
        let mut inner = PublicLayoutTreeOf::new()
            .style(0, root)
            .style(1, block())
            .style(2, block())
            .measure(1, Size::new(S::from_f64(100.0), S::from_f64(10.0)))
            .measure(2, Size::new(S::from_f64(100.0), S::from_f64(20.0)));
        if owned {
            let mut cap = block();
            cap.max_size.height = MaxSizeOf::px(S::from_f64(1000.0));
            inner = inner
                .style(3, cap)
                .style(4, block())
                .measure(4, Size::new(S::from_f64(100.0), S::from_f64(5.0)))
                .style(5, maximum_cap::<S>())
                .style(6, block())
                .measure(6, Size::new(S::from_f64(100.0), S::from_f64(5.0)))
                .children(0, [1, 3, 2, 5])
                .children(3, [4])
                .children(5, [6]);
        } else {
            inner = inner.children(0, [1, 2]);
        }
        Tree {
            inner,
            measured: RefCell::new(Vec::new()),
            reject: Cell::new(None),
        }
    }
    fn with_provider<S: LayoutScalar>(
        tree: &Tree<S>,
        run: impl FnOnce(&mut Provider<'_, '_, '_, Tree<S>>),
    ) {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let fallback = OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        );
        let sizing = SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(
                        NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
                    ),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            fallback,
        );
        let root = LayoutRootRequestOf::standalone(sizing, axes, Size::NONE);
        let frames = [BlockFragmentainerOf {
            origin: Point::ZERO,
            size: Size::new(S::from_f64(100.0), S::from_f64(100.0)),
            breaks: FragmentationBreakKinds::NONE,
            page_side: None,
        }];
        let bound = LayoutBoundRequestOf::block_fragmentation(
            LayoutSourceBasis::fresh(),
            root,
            FragmentationKind::Page,
            axes,
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
        let inventory = Inventory::new(tree, &capture.graph, axes).unwrap();
        let scopes = Scopes::new(inventory, tree, axes).unwrap();
        let environment = NativeProvider {
            capture: &capture,
            inventory: &scopes.source,
            frames: &frames,
            axes,
            kind: FragmentationKind::Page,
            target: bound.target_basis(),
            template: root,
            intrinsics: Rc::new(RefCell::new(Vec::new())),
        };
        let progress = Rc::new(RefCell::new(Progress {
            minima: vec![(0, S::ZERO)],
            active: None,
            completed: Vec::new(),
            tails: vec![None; scopes.scopes[0].units.len()],
            opened: false,
        }));
        let ready = Rc::new(RefCell::new(Vec::new()));
        let owners: Vec<_> = scopes
            .scopes
            .iter()
            .skip(1)
            .map(|scope| scopes.source.nodes[scope.owner].node)
            .collect();
        let dispatch = Dispatch {
            scope: ReceiptId(0),
            view: crate::engine::block_scope::SourceViewId(0),
            target: bound.target_basis(),
            owners: owners.into(),
            paths: Arc::from([0]),
            ready,
            requests: Rc::new(RefCell::new(Requests::new())),
        };
        let mut provider = Provider {
            environment: &environment,
            scopes: &scopes,
            scope: ScopeId(0),
            placement: None,
            progress,
            dispatch,
            query: Rc::new(Cell::new(None)),
            bindings: Rc::new(RefCell::new(Vec::new())),
            records: Rc::new(RefCell::new(Vec::new())),
            discovery: Rc::new(RefCell::new(None)),
            rebind_generation: 0,
            trial_selection: Rc::new(RefCell::new(None)),
            views: Rc::new(RefCell::new(Views::new())),
            qualifications: Rc::new(RefCell::new(Vec::new())),
            qualification_uses: Rc::new(RefCell::new(Vec::new())),
            productions: Rc::new(RefCell::new(Vec::new())),
        };
        run(&mut provider);
    }
    fn query<S: LayoutScalar>() -> FragmentationQueryOf<S> {
        FragmentationQueryOf {
            flow: 0,
            context: 0,
            fragmentainer_index: 0,
            fragmentainer: FragmentainerOf {
                inline_size: S::from_f64(100.0),
                block_size: S::from_f64(100.0),
                breaks: FragmentationBreakKinds::NONE,
                parent_fragmentainer: None,
                page_side: None,
            },
            position: FragmentationPosition {
                occurrence: 0,
                offset: 0,
            },
            graphical_progress: None,
            used_block_offset: S::ZERO,
            available_block_size: S::from_f64(100.0),
            stage: FragmentationStage::Strict,
            scan_after: None,
            start_provenance: FragmentationStart::FlowStart,
        }
    }
    fn ready<S: LayoutScalar>(
        provider: &Provider<'_, '_, '_, Tree<S>>,
        id: crate::engine::block_scope::ScopeRequestId,
    ) -> Ready<u32, S> {
        let request = provider
            .dispatch
            .requests
            .borrow_mut()
            .take(id)
            .expect("suspension retains its owned request");
        let mut body =
            crate::engine::ComputeSession::new(provider.environment.capture.tree, Vec::new());
        let output =
            crate::block::compute_block_scoped(&mut body, request.entry.node, request.entry.input)
                .unwrap();
        Ready {
            id: ReceiptId(id.0),
            view: request.view,
            projection: None,
            entry: request.entry,
            output,
        }
    }
    fn minimum_retry_ready<S: LayoutScalar>(
        provider: &Provider<'_, '_, '_, Tree<S>>,
        id: crate::engine::block_scope::ScopeRequestId,
        node: u32,
    ) -> Ready<u32, S> {
        let request = provider
            .dispatch
            .requests
            .borrow_mut()
            .take(id)
            .expect("minimum retry retains its exact owned request");
        assert_eq!(request.entry.node, node);
        {
            let ready = provider.dispatch.ready.borrow();
            let prior = ready
                .iter()
                .find(|ready| ready.entry.node == node)
                .expect("minimum retry follows the node's natural-size production");
            assert_eq!(
                prior.entry.reference,
                Size::new(S::from_f64(100.0), S::ZERO)
            );
            assert_eq!(
                request.entry.reference,
                Size::new(S::from_f64(100.0), S::from_f64(50.0))
            );
            assert_eq!(request.entry.input, prior.entry.input);
            let mut normalized = request.entry;
            normalized.reference = prior.entry.reference;
            assert_eq!(
                normalized, prior.entry,
                "minimum retry changes only its Block reference"
            );
        }
        let mut body =
            crate::engine::ComputeSession::new(provider.environment.capture.tree, Vec::new());
        let output =
            crate::block::compute_block_scoped(&mut body, request.entry.node, request.entry.input)
                .unwrap();
        Ready {
            id: ReceiptId(id.0),
            view: request.view,
            projection: None,
            entry: request.entry,
            output,
        }
    }
    fn piece<S: LayoutScalar>(
        start: usize,
        end: usize,
        node: u32,
        size: f64,
    ) -> FragmentationPieceOf<SourceKey<u32>, S> {
        FragmentationPieceOf {
            flow: 0,
            context: 0,
            fragmentainer: 0,
            start: FragmentationPosition {
                occurrence: start,
                offset: 0,
            },
            end: FragmentationPosition {
                occurrence: end,
                offset: 0,
            },
            start_provenance: FragmentationStart::FlowStart,
            coverage: vec![FragmentationCoverage {
                key: SourceKey::Atomic(node),
                start: 0,
                end: 1,
            }],
            block_offset: S::ZERO,
            block_size: S::from_f64(size),
            cloned_block_start: S::ZERO,
            cloned_block_end: S::ZERO,
            stage: FragmentationStage::Strict,
            kind: FragmentationPieceKindOf::Content,
        }
    }
    fn accept<S: LayoutScalar>(
        provider: &Provider<'_, '_, '_, Tree<S>>,
        piece: FragmentationPieceOf<SourceKey<u32>, S>,
        complete: bool,
    ) -> ScopeResult<Tree<S>, ()> {
        provider.commit(crate::fragmentation::AcceptedStep {
            flow: 0,
            from_frame: 0,
            to_frame: 0,
            position: piece.end,
            graphical_progress: None,
            pieces: &[piece],
            breaks: &[],
            flow_complete: complete,
            appearances: &[],
        })
    }
    /// Mirror Native's principal-cache reissuance, without consuming another
    /// child source execution or recomputing its unchanged producer body.
    /// This fixture keeps both bodies whole from source start zero, with zero
    /// Slice decorations: expanding the parent window changes only the binding.
    fn layout_rebinding<S: LayoutScalar>(
        provider: &mut Provider<'_, '_, '_, Tree<S>>,
        query: FragmentationQueryOf<S>,
    ) -> ScopeResult<Tree<S>, FragmentationReplyOf<S>> {
        // This query has one fixed start and at most one distinct end per source
        // occurrence. Each owner can need one exact binding in each window.
        let limit = provider.scopes.scopes[provider.scope.0].units.len()
            * provider.dispatch.owners.len().max(1)
            + 1;
        for _ in 0..limit {
            let reply = provider.layout(query);
            let Err(ComputeStop::NeedScope(id)) = reply else {
                return reply;
            };
            let (entry, view) = {
                let requests = provider.dispatch.requests.borrow();
                let request = requests
                    .peek(id)
                    .expect("the exact suspension request remains owned");
                (request.entry, request.view)
            };
            let prior = provider
                .dispatch
                .ready
                .borrow()
                .iter()
                .find(|ready| ready.entry == entry)
                .copied();
            let Some(prior) = prior else {
                return Err(ComputeStop::NeedScope(id));
            };
            assert_ne!(
                prior.view, view,
                "an exact existing binding must already be consumable"
            );
            provider
                .dispatch
                .requests
                .borrow_mut()
                .take(id)
                .expect("this rebind consumes only its owned request");
            let rebound = Ready { view, ..prior };
            assert_eq!(rebound.entry, entry);
            assert_eq!(rebound.id, prior.id);
            assert_eq!(rebound.output, prior.output);
            provider.dispatch.ready.borrow_mut().push(rebound);
        }
        panic!("each unchanged owner body can be rebound only once per exact source window");
    }
    fn discovery_resume<S: LayoutScalar>() {
        let tree = tree::<S>(true);
        with_provider(&tree, |provider| {
            let query = query();
            let stop = layout_rebinding(provider, query).unwrap_err();
            let ComputeStop::NeedScope(id) = stop else {
                panic!("actual owner demands its child scope");
            };
            {
                let state = provider.discovery.borrow();
                let state = state.as_ref().unwrap();
                assert_eq!(state.boundary, 2);
                assert_eq!(state.extent, S::from_f64(10.0));
                assert!(state.preview.is_some());
            }
            let prepared = ready(provider, id);
            provider.dispatch.ready.borrow_mut().push(prepared);
            let stop = layout_rebinding(provider, query).unwrap_err();
            let ComputeStop::NeedScope(id) = stop else {
                panic!("second actual owner demands its independent receipt");
            };
            {
                let state = provider.discovery.borrow();
                let state = state.as_ref().unwrap();
                assert_eq!(state.boundary, 5);
                assert_eq!(state.extent, S::from_f64(35.0));
            }
            let prepared = ready(provider, id);
            provider.dispatch.ready.borrow_mut().push(prepared);
            // Closing the root's minimum retries its actual body at reference 50.
            // Each of the two owners receives one exact request for that reference.
            let stop = layout_rebinding(provider, query).unwrap_err();
            let ComputeStop::NeedScope(id) = stop else {
                panic!("minimum retry requires the first owner's bound receipt");
            };
            let prepared = minimum_retry_ready(provider, id, 3);
            provider.dispatch.ready.borrow_mut().push(prepared);
            let stop = layout_rebinding(provider, query).unwrap_err();
            let ComputeStop::NeedScope(id) = stop else {
                panic!("minimum retry requires the second owner's bound receipt");
            };
            let prepared = minimum_retry_ready(provider, id, 5);
            provider.dispatch.ready.borrow_mut().push(prepared);
            let reply = layout_rebinding(provider, query).unwrap();
            let FragmentationReplyOf::Measured(window) = reply else {
                panic!("resumed actual window is measured");
            };
            assert_eq!(window.block_size, S::from_f64(50.0));
            assert!(
                provider
                    .discovery
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .preview
                    .is_some()
            );
            let before = tree.measured.borrow().clone();
            // Receipt availability is not a placement replacement: the complete
            // geometry window is reusable without another host measurement.
            layout_rebinding(provider, query).unwrap();
            assert_eq!(*tree.measured.borrow(), before);
            // Discarding the prior placement's ready receipts starts a new
            // binding generation; its completed window cannot satisfy replay.
            provider.dispatch.ready.borrow_mut().clear();
            provider.rebind_generation += 1;
            assert!(matches!(
                layout_rebinding(provider, query),
                Err(ComputeStop::NeedScope(_))
            ));
            assert!(tree.measured.borrow().len() > before.len());
            assert_eq!(provider.discovery.borrow().as_ref().unwrap().boundary, 2);
        });
    }
    fn feedback_atomicity<S: LayoutScalar>() {
        let tree = tree::<S>(true);
        with_provider(&tree, |provider| {
            provider.query.set(Some(query()));
            accept(provider, piece(0, 1, 1, 10.0), false).unwrap();
            assert_eq!(provider.progress.borrow().minima, [(0, S::from_f64(10.0))]);
            let snapshot = provider
                .progress
                .borrow()
                .active
                .as_ref()
                .unwrap()
                .accounted
                .clone();
            let owner = || FragmentationPieceOf {
                block_offset: S::from_f64(10.0),
                coverage: vec![
                    FragmentationCoverage {
                        key: SourceKey::Owner(3),
                        start: 0,
                        end: 0,
                    },
                    FragmentationCoverage {
                        key: SourceKey::PrincipalEnd(3),
                        start: 0,
                        end: 0,
                    },
                ],
                ..piece(1, 3, 3, 5.0)
            };
            let stop = accept(provider, owner(), false).unwrap_err();
            assert!(matches!(stop, ComputeStop::NeedScope(_)));
            let progress = provider.progress.borrow();
            assert_eq!(progress.minima, [(0, S::from_f64(10.0))]);
            assert_eq!(progress.active.as_ref().unwrap().selection.range, 0..1);
            assert_eq!(progress.active.as_ref().unwrap().accounted, snapshot);
            assert!(progress.completed.is_empty());
            assert!(!progress.opened);
            assert!(progress.tails.iter().all(Option::is_none));
            drop(progress);
            tree.reject.set(Some(1));
            let stop = accept(provider, owner(), false).unwrap_err();
            assert!(matches!(stop, ComputeStop::Layout(_)));
            let progress = provider.progress.borrow();
            assert_eq!(progress.minima, [(0, S::from_f64(10.0))]);
            assert_eq!(progress.active.as_ref().unwrap().accounted, snapshot);
            assert!(progress.completed.is_empty());
            assert!(!progress.opened);
        });
    }
    fn feedback_frame_delta<S: LayoutScalar>() {
        let tree = tree::<S>(false);
        with_provider(&tree, |provider| {
            provider.query.set(Some(query()));
            accept(provider, piece(0, 1, 1, 10.0), false).unwrap();
            assert_eq!(provider.progress.borrow().minima, [(0, S::from_f64(10.0))]);
            let second = FragmentationPieceOf {
                block_offset: S::from_f64(10.0),
                ..piece(1, 2, 2, 20.0)
            };
            accept(provider, second, false).unwrap();
            assert_eq!(provider.progress.borrow().minima, [(0, S::from_f64(30.0))]);
            assert!(provider.progress.borrow().completed.is_empty());
            let closing = FragmentationPieceOf {
                block_offset: S::from_f64(30.0),
                coverage: vec![FragmentationCoverage {
                    key: SourceKey::BoxEnd(0),
                    start: 0,
                    end: 0,
                }],
                ..piece(2, 3, 0, 20.0)
            };
            accept(provider, closing, true).unwrap();
            let progress = provider.progress.borrow();
            assert_eq!(progress.minima, [(0, S::from_f64(50.0))]);
            assert!(progress.active.is_none());
            assert_eq!(progress.completed.len(), 1);
            assert_eq!(progress.completed[0].selection.range, 0..3);
            assert_eq!(progress.completed[0].frame, 0);
        });
    }
    #[test]
    fn provider_discovery_resume_f32() {
        discovery_resume::<f32>();
    }
    #[test]
    fn provider_discovery_resume_f64() {
        discovery_resume::<f64>();
    }
    #[test]
    fn provider_feedback_atomicity_f32() {
        feedback_atomicity::<f32>();
    }
    #[test]
    fn provider_feedback_atomicity_f64() {
        feedback_atomicity::<f64>();
    }
    #[test]
    fn provider_feedback_frame_delta_f32() {
        feedback_frame_delta::<f32>();
    }
    #[test]
    fn provider_feedback_frame_delta_f64() {
        feedback_frame_delta::<f64>();
    }
}
