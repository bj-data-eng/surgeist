//! Owned scheduler work and exact-placement preparation.
use super::*;
use crate::engine::block_scope::{Request, ScopeRequestId};
use crate::fragmentation::ScopeHeadProjectionOf;

type ScopeProjections<S> = (Vec<ScopeProjectionOf<S>>, Vec<ScopeHeadProjectionOf<S>>);

pub(super) struct Preparation<Node, S: LayoutScalar> {
    request: Request<Node, S>,
    frame: usize,
    selection: Selection<S>,
    waiting: Option<ReceiptId>,
    ticket: Option<ScopeTicket<S>>,
}

enum Prepared {
    Complete,
    Child,
}

impl<Tree: LayoutTree> Driver<'_, '_, '_, Tree> {
    fn preparation(
        &self,
        id: ScopeRequestId,
        frame: usize,
        selection: Selection<Tree::Scalar>,
    ) -> NativeResult<Tree, Preparation<Tree::Node, Tree::Scalar>> {
        let request = self.requests.borrow_mut().take(id).ok_or_else(|| {
            self.environment
                .failure(BlockFragmentationFailure::InvalidSource)
        })?;
        Ok(Preparation {
            request,
            frame,
            selection,
            waiting: None,
            ticket: None,
        })
    }

    fn completed_work(
        &self,
        work: Work<Tree::Node, Tree::Scalar>,
    ) -> NativeResult<Tree, CompletedReceipt<Tree>> {
        let frames = std::mem::take(&mut work.progress.borrow_mut().completed);
        let timeline = match work.context.and_then(|context| context.limits) {
            Some(limits) => Some(crate::block::content::principal_timeline(
                self.scopes.source.nodes[self.scopes.scopes[work.scope.0].owner].node,
                limits,
                &frames
                    .iter()
                    .filter_map(|frame| frame.summary)
                    .collect::<Vec<_>>(),
            )?),
            None => None,
        };
        Ok(Arc::new(Completed {
            scope: work.scope,
            report: Arc::new(work.execution.finish()),
            placement: work.placement,
            frames,
            ready: work.ready,
            timeline,
            principals: RefCell::new(Vec::new()),
            bindings: work.bindings,
        }))
    }

    fn projection_key(
        &self,
        owner: usize,
        selection: &Selection<Tree::Scalar>,
        frame: usize,
        completed: &Completed<Tree::Node, Tree::Scalar>,
    ) -> ProjectionKey {
        ProjectionKey {
            frame,
            body: selection.range.start <= owner,
            closing: selection.range.end > owner + 1
                && selection.partial != Some(owner)
                && completed
                    .timeline
                    .as_ref()
                    .and_then(|timeline| timeline.spans.last())
                    .is_some_and(|span| span.frame <= frame),
        }
    }

    fn projections(
        &self,
        owner: usize,
        completed: &Completed<Tree::Node, Tree::Scalar>,
    ) -> NativeResult<Tree, ScopeProjections<Tree::Scalar>> {
        let timeline = completed.timeline.as_ref().ok_or_else(|| {
            self.environment
                .failure(BlockFragmentationFailure::InvalidSource)
        })?;
        let placement = completed.placement.as_ref().ok_or_else(|| {
            self.environment
                .failure(BlockFragmentationFailure::InvalidSource)
        })?;
        let report = completed.report.report();
        let initial = report
            .pieces
            .first()
            .is_some_and(|piece| piece.fragmentainer > placement.query.fragmentainer_index);
        if initial {
            let index = report
                .breaks
                .iter()
                .position(|record| {
                    record.from_fragmentainer == placement.query.fragmentainer_index
                        && record.reason == FragmentationBreakReason::Deferred
                })
                .ok_or_else(|| {
                    self.environment
                        .failure(BlockFragmentationFailure::InvalidSource)
                })?;
            return Ok((
                vec![ScopeProjectionOf {
                    child_break_index: index,
                    cut: FragmentationCutOf::Source(FragmentationPosition {
                        occurrence: owner,
                        offset: 0,
                    }),
                    block_offset: placement.outside,
                }],
                Vec::new(),
            ));
        }
        let mut projections = Vec::new();
        let mut heads = Vec::new();
        projections
            .try_reserve(timeline.spans.len())
            .and_then(|_| heads.try_reserve(timeline.spans.len()))
            .map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(placement.entry.pending.node),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
        let mut consumed = Tree::Scalar::ZERO;
        for span in &timeline.spans {
            consumed = consumed + span.opening + span.content;
            let origin = if span.frame == placement.query.fragmentainer_index {
                placement.origin.block
            } else {
                Tree::Scalar::ZERO
            };
            let end = completed.span_end(span);
            let cut = if consumed < timeline.body_extent {
                FragmentationCutOf::Graphical {
                    anchor: FragmentationPosition {
                        occurrence: owner,
                        offset: 0,
                    },
                    progress: FragmentationGraphicalProgressOf {
                        consumed,
                        extent: timeline.body_extent,
                    },
                }
            } else {
                FragmentationCutOf::Source(FragmentationPosition {
                    occurrence: owner + 1,
                    offset: 0,
                })
            };
            if span.remaining > Tree::Scalar::ZERO
                && consumed < timeline.body_extent
                && let Some((index, _)) = report
                    .breaks
                    .iter()
                    .enumerate()
                    .find(|(_, record)| record.from_fragmentainer == span.frame)
            {
                projections.push(ScopeProjectionOf {
                    child_break_index: index,
                    cut,
                    block_offset: end,
                });
            }
            for (index, piece) in report
                .pieces
                .iter()
                .enumerate()
                .filter(|(_, piece)| piece.fragmentainer == span.frame)
            {
                if !completed.report.head_qualified(index) {
                    continue;
                }
                // A head's cap projection is independent of the content flow's
                // occupied extent; cap1/child150 therefore remains principal1.
                let projected = (piece.block_offset + piece.block_size - origin)
                    .min(span.opening + span.content);
                let before = consumed - span.opening - span.content;
                let head_consumed = before + projected;
                let head_cut = if head_consumed < timeline.body_extent {
                    FragmentationCutOf::Graphical {
                        anchor: FragmentationPosition {
                            occurrence: owner,
                            offset: 0,
                        },
                        progress: FragmentationGraphicalProgressOf {
                            consumed: head_consumed,
                            extent: timeline.body_extent,
                        },
                    }
                } else {
                    FragmentationCutOf::Source(FragmentationPosition {
                        occurrence: owner + 1,
                        offset: 0,
                    })
                };
                if matches!(head_cut, FragmentationCutOf::Graphical { .. }) {
                    let occupied = if span.frame == placement.query.fragmentainer_index {
                        placement.outside
                    } else {
                        Tree::Scalar::ZERO
                    };
                    heads.push(ScopeHeadProjectionOf {
                        child_piece_index: index,
                        cut: head_cut,
                        block_offset: occupied.max(origin + projected),
                    });
                }
            }
        }
        Ok((projections, heads))
    }

    fn discard(&self, receipt: ReceiptId) -> NativeResult<Tree, ()> {
        let mut pending = vec![receipt];
        while let Some(id) = pending.pop() {
            let record = self
                .records
                .borrow_mut()
                .get_mut(id.0)
                .and_then(Option::take);
            let Some(record) = record else {
                continue;
            };
            let children = record.bindings.borrow();
            pending.try_reserve(children.len()).map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(self.environment.capture.root),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
            pending.extend(children.iter().map(|binding| binding.receipt));
        }
        Ok(())
    }

    fn prepare(
        &mut self,
        current: usize,
        preparation: &mut Preparation<Tree::Node, Tree::Scalar>,
    ) -> ScopeResult<Tree, Prepared> {
        let origin_id = preparation.request.scope;
        let origin_record = if self.stack[current].id == origin_id {
            None
        } else {
            Some(
                self.records
                    .borrow()
                    .get(origin_id.0)
                    .and_then(Option::as_ref)
                    .cloned()
                    .ok_or_else(|| {
                        self.environment
                            .failure(BlockFragmentationFailure::InvalidSource)
                    })?,
            )
        };
        let (scope, bindings, ready) = match &origin_record {
            Some(record) => (record.scope, record.bindings.clone(), record.ready.clone()),
            None => {
                let work = &self.stack[current];
                (work.scope, work.bindings.clone(), work.ready.clone())
            }
        };
        let node_index = self
            .scopes
            .source
            .nodes
            .iter()
            .position(|source| source.node == preparation.request.entry.node)
            .ok_or_else(|| {
                self.environment
                    .failure(BlockFragmentationFailure::InvalidSource)
            })?;
        let location = self.scopes.locations[node_index];
        if location.scope != scope
            || !matches!(
                self.scopes.scopes[scope.0].units.get(location.first),
                Some(Unit::Owner { .. })
            )
        {
            return Err(self
                .environment
                .failure(BlockFragmentationFailure::InvalidSource)
                .into());
        }
        let owner = location.first;
        if !self
            .views
            .borrow()
            .get(preparation.request.view)
            .is_some_and(|view| view.scope == scope && view.frame == preparation.frame)
        {
            return Err(self
                .environment
                .failure(BlockFragmentationFailure::InvalidSource)
                .into());
        }
        let mut prior = bindings
            .borrow()
            .iter()
            .find(|binding| binding.owner == owner)
            .copied();
        if preparation.waiting.is_none()
            && origin_record.is_none()
            && let Some(binding) = prior
        {
            let work = &self.stack[current];
            let query = work.query.get().ok_or_else(|| {
                self.environment
                    .failure(BlockFragmentationFailure::InvalidSource)
            })?;
            let participated = query.position.occurrence > owner
                || query.position.occurrence == owner && query.graphical_progress.is_some();
            if !participated {
                let resolved = self.resolve_entry(work, query, &preparation.request)?;
                let record = self
                    .records
                    .borrow()
                    .get(binding.receipt.0)
                    .and_then(Option::as_ref)
                    .cloned()
                    .ok_or_else(|| {
                        self.environment
                            .failure(BlockFragmentationFailure::InvalidSource)
                    })?;
                let same = record.placement.as_ref().is_some_and(|old| {
                    old.query.fragmentainer_index == preparation.frame
                        && old.origin == resolved.placement.origin
                        && old.outside == resolved.placement.outside
                        && old.entry.pending.input == resolved.placement.entry.pending.input
                        && old.entry.pending.inline_outer
                            == resolved.placement.entry.pending.inline_outer
                        && old.entry.pending.margin == resolved.placement.entry.pending.margin
                });
                if !same {
                    let generation = work.rebind_generation.checked_add(1).ok_or_else(|| {
                        self.environment
                            .failure(BlockFragmentationFailure::InvalidSource)
                    })?;
                    bindings
                        .borrow_mut()
                        .retain(|candidate| candidate.owner != owner);
                    ready
                        .borrow_mut()
                        .retain(|candidate| candidate.entry.node != preparation.request.entry.node);
                    self.stack[current].rebind_generation = generation;
                    *self.stack[current].discovery.borrow_mut() = None;
                    self.discard(binding.receipt)?;
                    prior = None;
                }
            }
        }
        if preparation.waiting.is_none() {
            if let Some(binding) = prior {
                preparation.waiting = Some(binding.receipt);
            } else {
                if origin_record.is_some() {
                    return Err(self
                        .environment
                        .failure(BlockFragmentationFailure::InvalidSource)
                        .into());
                }
                let work = &self.stack[current];
                let query = work.query.get().ok_or_else(|| {
                    self.environment
                        .failure(BlockFragmentationFailure::InvalidSource)
                })?;
                let resolved = self.resolve_entry(work, query, &preparation.request)?;
                let scope = self.scopes.owned[node_index].ok_or_else(|| {
                    self.environment
                        .failure(BlockFragmentationFailure::InvalidSource)
                })?;
                let ticket = work
                    .execution
                    .scope_ticket(
                        query,
                        FragmentationPosition {
                            occurrence: owner,
                            offset: 0,
                        },
                        FragmentationPosition {
                            occurrence: owner + 1,
                            offset: 0,
                        },
                    )
                    .map_err(|_| {
                        self.environment
                            .failure(BlockFragmentationFailure::InvalidSource)
                    })?;
                let id = ReceiptId(self.records.borrow().len());
                self.records.borrow_mut().try_reserve(1).map_err(|_| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(preparation.request.entry.node),
                        LayoutOperation::RootLayout,
                        LayoutErrorKindOf::Allocation,
                    )
                })?;
                self.records.borrow_mut().push(None);
                let context = FragmentationContext {
                    kind: self.environment.kind,
                    parent: None,
                    flow_axes: self.environment.axes,
                };
                let frame = self.environment.frames[preparation.frame];
                let intrinsics = self.environment.intrinsics(frame)?;
                let minimum_nodes =
                    minimum_nodes(self.environment, self.scopes, scope, &intrinsics)?;
                let mut child = Work::new(
                    id,
                    scope,
                    self.scopes,
                    context,
                    Some(resolved.placement),
                    minimum_nodes,
                )?;
                child.context = Some(resolved.context);
                preparation.waiting = Some(id);
                preparation.ticket = Some(ticket);
                self.stack.try_reserve(1).map_err(|_| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(preparation.request.entry.node),
                        LayoutOperation::RootLayout,
                        LayoutErrorKindOf::Allocation,
                    )
                })?;
                self.stack.push(child);
                return Ok(Prepared::Child);
            }
        }
        let id = preparation
            .waiting
            .expect("a prepared scope has an owned receipt identity");
        let completed = self
            .records
            .borrow()
            .get(id.0)
            .and_then(Option::as_ref)
            .cloned()
            .ok_or_else(|| {
                self.environment
                    .failure(BlockFragmentationFailure::InvalidSource)
            })?;
        let key = self.projection_key(owner, &preparation.selection, preparation.frame, &completed);
        if let Some(ticket) = preparation.ticket.take() {
            let (projections, heads) = self.projections(owner, &completed)?;
            let initial_deferred = projections.iter().any(|projection| {
                projection.cut
                    == FragmentationCutOf::Source(FragmentationPosition {
                        occurrence: owner,
                        offset: 0,
                    })
            });
            bindings.borrow_mut().try_reserve(1).map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(preparation.request.entry.node),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
            self.stack[current]
                .execution
                .register_scope(ticket, completed.report.clone(), projections, heads)
                .map_err(|_| {
                    self.environment
                        .failure(BlockFragmentationFailure::InvalidSource)
                })?;
            let mut bindings = bindings.borrow_mut();
            bindings.push(Binding {
                owner,
                receipt: id,
                frame: preparation.frame,
                initial_deferred,
                projection: Some(key),
            });
            if initial_deferred {
                return Ok(Prepared::Complete);
            }
        }
        let issued = self.principal(
            &completed,
            id,
            preparation.request.entry,
            key,
            preparation.request.view,
        )?;
        let mut ready = ready.borrow_mut();
        if let Some(slot) = ready
            .iter_mut()
            .find(|slot| slot.entry.node == issued.entry.node)
        {
            *slot = issued;
        } else {
            ready.try_reserve(1).map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(issued.entry.node),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
            ready.push(issued);
        }
        Ok(Prepared::Complete)
    }
}

pub(super) struct ResolvedScopes<Node, S: LayoutScalar> {
    pub(super) root: ReceiptId,
    pub(super) records: Records<Node, S>,
}
impl<Tree: LayoutTree> Driver<'_, '_, '_, Tree> {
    fn push_preparation(
        &mut self,
        current: usize,
        id: ScopeRequestId,
        frame: usize,
        selection: Selection<Tree::Scalar>,
    ) -> NativeResult<Tree, ()> {
        let preparation = self.preparation(id, frame, selection)?;
        let root = preparation.request.entry.node;
        self.stack[current]
            .preparations
            .try_reserve(1)
            .map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(root),
                    LayoutOperation::RootLayout,
                    LayoutErrorKindOf::Allocation,
                )
            })?;
        self.stack[current].preparations.push(preparation);
        Ok(())
    }
    pub(super) fn run(mut self) -> NativeResult<Tree, ResolvedScopes<Tree::Node, Tree::Scalar>> {
        loop {
            let current = self.stack.len() - 1;
            if let Some(mut preparation) = self.stack[current].preparations.pop() {
                match self.prepare(current, &mut preparation) {
                    Ok(Prepared::Complete) => continue,
                    Ok(Prepared::Child) => {
                        self.stack[current].preparations.push(preparation);
                        continue;
                    }
                    Err(ComputeStop::NeedScope(id)) => {
                        let frame = preparation.frame;
                        let source = self
                            .requests
                            .borrow()
                            .peek(id)
                            .map(|request| request.scope)
                            .ok_or_else(|| {
                                self.environment
                                    .failure(BlockFragmentationFailure::InvalidSource)
                            })?;
                        let selection = if source == self.stack[current].id {
                            preparation.selection.clone()
                        } else {
                            self.records
                                .borrow()
                                .get(source.0)
                                .and_then(Option::as_ref)
                                .and_then(|record| {
                                    record.frames.iter().find(|entry| entry.frame == frame)
                                })
                                .map(|entry| entry.selection.clone())
                                .ok_or_else(|| {
                                    self.environment
                                        .failure(BlockFragmentationFailure::InvalidSource)
                                })?
                        };
                        self.stack[current].preparations.push(preparation);
                        self.push_preparation(current, id, frame, selection)?;
                        continue;
                    }
                    Err(ComputeStop::Layout(error)) => return Err(error),
                }
            }
            let dispatch = self.dispatch(&self.stack[current]);
            let result = {
                let work = &mut self.stack[current];
                let mut provider = Provider {
                    environment: self.environment,
                    scopes: self.scopes,
                    scope: work.scope,
                    placement: work.placement.as_ref(),
                    progress: work.progress.clone(),
                    dispatch,
                    query: work.query.clone(),
                    bindings: work.bindings.clone(),
                    records: self.records.clone(),
                    discovery: work.discovery.clone(),
                    rebind_generation: work.rebind_generation,
                    trial_selection: work.trial_selection.clone(),
                    views: self.views.clone(),
                    qualifications: self.qualifications.clone(),
                    qualification_uses: work.qualifications.clone(),
                    bodies: self.bodies.clone(),
                };
                let observer = provider.observer();
                work.execution
                    .poll(&mut provider, &mut |event| observer.commit(event))
            };
            match result {
                Ok(false) => {}
                Ok(true) => {
                    let work = self
                        .stack
                        .pop()
                        .expect("the current scope work is owned by the driver");
                    let id = work.id;
                    let completed = self.completed_work(work)?;
                    self.records.borrow_mut()[id.0] = Some(completed);
                    if self.stack.is_empty() {
                        return Ok(ResolvedScopes {
                            root: id,
                            records: self.records,
                        });
                    }
                }
                Err(error) => match self.map_error(error) {
                    ComputeStop::Layout(error) => return Err(error),
                    ComputeStop::NeedScope(id) => {
                        let work = &self.stack[current];
                        let query = work.query.get().ok_or_else(|| {
                            self.environment
                                .failure(BlockFragmentationFailure::InvalidSource)
                        })?;
                        let selection = work.trial_selection.borrow().clone().ok_or_else(|| {
                            self.environment
                                .failure(BlockFragmentationFailure::InvalidSource)
                        })?;
                        self.push_preparation(current, id, query.fragmentainer_index, selection)?;
                    }
                },
            }
        }
    }
}

#[cfg(test)]
#[path = "scope_work_tests.rs"]
mod tests;
