//! Exact depths across the topology changes of the record operations.
//!
//! Every record caches its depth within its own tree, and the usage depth is the greatest cached
//! depth. A change moves at most two subtrees to new depths, so only their caches change. The
//! change is planned before anything is written: one walk of each moved subtree counts its
//! records at each old level and finds its height, and a scan down the depth histogram finds the
//! usage depth when a moved subtree held the deepest record and moved shallower. The writes then
//! place each record of the moved subtrees at its new depth.
//!
//! A fresh childless node, or a template with only its empty contents, joins below its parent at
//! one known depth and walks nothing.

use super::shape::{next_edge, owning_parent, structural_parent, walk_budget};
use super::*;

/// Where a moved subtree goes.
#[derive(Clone, Copy, Debug)]
pub(super) enum Placement {
    /// Below `parent`, whose depth is read as it is.
    Under(NodeIx),
    /// To depth zero, as the root of a tree of its own.
    Detached,
    /// Below a host whose depth is measured by a walk up its owners as they stand after the
    /// change: the contents `skip` that the change releases no longer own it.
    AfterHost(NodeIx),
}

/// A subtree to move.
#[derive(Clone, Copy, Debug)]
pub(super) struct Request {
    pub(super) root: NodeIx,
    pub(super) placement: Placement,
}

/// One subtree a change moves, and what planning learned of it.
#[derive(Clone, Copy, Debug)]
struct Shift {
    root: NodeIx,
    /// The root's cached depth before the change.
    from: usize,
    /// The root's depth after the change.
    to: usize,
    /// The deepest level reached below the root.
    height: usize,
    /// Records reached, including the root.
    size: usize,
    /// Edge lookups the walk made, which the writes' walk repeats.
    calls: usize,
}
impl Shift {
    fn walked(&self) -> bool {
        self.from != self.to
    }
    fn top(&self) -> usize {
        self.from.saturating_add(self.height)
    }
    fn new_top(&self) -> usize {
        self.to.saturating_add(self.height)
    }
    /// Whether the walk counted records at the old depth `level`.
    fn counted(&self, level: usize) -> bool {
        self.walked() && level >= self.from && level <= self.top()
    }
}

#[derive(Clone, Copy, Debug)]
enum Plan {
    /// A fresh leaf, or a template with only its empty contents, joins at one depth.
    Fast {
        node: NodeIx,
        depth: usize,
        /// The node, and its contents when it is a template.
        records: usize,
    },
    Shifts {
        shifts: [Option<Shift>; 2],
        /// The contents the change releases, which the walks leave out.
        skip: Option<NodeIx>,
    },
}

/// A planned change of depths: the usage depth it leaves and the work of its writes.
#[derive(Clone, Copy, Debug)]
pub(super) struct DepthPlan {
    plan: Plan,
    pub(super) depth: usize,
}

impl DepthPlan {
    /// The record visits and writes of placing the records: per edge lookup two visits and one
    /// cursor write, and per placed record a cursor reset, its depth, and the two histogram
    /// counts it leaves and joins.
    pub(super) fn work(&self) -> Result<(usize, usize), HtmlConstructionError> {
        match &self.plan {
            Plan::Fast { records, .. } => Ok((0, times(*records, 3)?)),
            Plan::Shifts { shifts, .. } => {
                let (mut visits, mut writes) = (0usize, 0usize);
                for shift in shifts.iter().flatten().filter(|shift| shift.walked()) {
                    visits = plus(visits, times(shift.calls, 2)?)?;
                    writes = plus(plus(writes, shift.calls)?, times(shift.size, 4)?)?;
                }
                Ok((visits, writes))
            }
        }
    }

    /// Places the planned records at their planned depths. Planning read every record placed
    /// here, so the unknown-node and conflict defects are the only failures, and they are
    /// returned.
    pub(super) fn apply(&self, arena: &mut BuildArena) -> Result<(), HtmlConstructionError> {
        match &self.plan {
            Plan::Fast { node, depth, .. } => {
                arena.place(*node, *depth)?;
                if let Some(TemplateContents::Owned(contents)) = arena
                    .records
                    .get(node.position())
                    .and_then(|record| record.template_contents)
                {
                    arena.place(contents, plus(*depth, 1)?)?;
                }
            }
            Plan::Shifts { shifts, skip } => {
                for shift in shifts.iter().flatten().filter(|shift| shift.walked()) {
                    arena.relocate_subtree(shift.root, shift.to, *skip)?;
                }
            }
        }
        Ok(())
    }
}

impl HtmlConstructionStorage {
    /// The record at `index`, or the unknown-node error.
    pub(super) fn record(&self, index: NodeIx) -> Result<&Record, HtmlConstructionError> {
        self.arena
            .records
            .get(index.position())
            .ok_or(HtmlConstructionCause::UnknownNode.error())
    }

    /// Whether `node` has nothing below it: no children, no shadow root, and contents that it
    /// owns, if any, that have neither. Such a node cannot be an ancestor of anything.
    pub(super) fn is_shallow(&self, node: NodeIx) -> Result<bool, HtmlConstructionError> {
        let record = self.record(node)?;
        if !record.children.is_empty() || record.shadow.is_some() {
            return Ok(false);
        }
        match record.template_contents {
            Some(TemplateContents::Owned(contents)) => {
                let contents = self.record(contents)?;
                Ok(contents.children.is_empty() && contents.shadow.is_none())
            }
            _ => Ok(true),
        }
    }

    /// Plans the depths after moving the subtrees of `requests`, leaving the contents `skip` out of
    /// every walk, charging the records it visits. `fresh` says that a single subtree has no old
    /// place, so a shallow one needs no walk.
    pub(super) fn plan_depths<C: Charge<HtmlWork, HtmlResource>>(
        &mut self,
        requests: &[Request],
        skip: Option<NodeIx>,
        fresh: bool,
        charge: &mut C,
    ) -> Result<DepthPlan, HtmlConstructionRecordError<C::Stop>> {
        let usage_depth = self.arena.usage.depth;
        if fresh
            && let [
                Request {
                    root,
                    placement: Placement::Under(parent),
                },
            ] = *requests
        {
            Work {
                record_visits: 3,
                ..Work::default()
            }
            .charge(charge)?;
            if self.is_shallow(root).map_err(faulted)? {
                let depth =
                    plus(self.record(parent).map_err(faulted)?.depth, 1).map_err(faulted)?;
                let template = usize::from(matches!(
                    self.record(root).map_err(faulted)?.template_contents,
                    Some(TemplateContents::Owned(_))
                ));
                return Ok(DepthPlan {
                    plan: Plan::Fast {
                        node: root,
                        depth,
                        records: 1 + template,
                    },
                    depth: usage_depth.max(plus(depth, template).map_err(faulted)?),
                });
            }
        }
        let mut shifts: [Option<Shift>; 2] = [None, None];
        for (slot, request) in shifts.iter_mut().zip(requests) {
            Work {
                record_visits: 2,
                ..Work::default()
            }
            .charge(charge)?;
            let from = self.record(request.root).map_err(faulted)?.depth;
            let to = match request.placement {
                Placement::Detached | Placement::AfterHost(_) => 0,
                Placement::Under(parent) => {
                    plus(self.record(parent).map_err(faulted)?.depth, 1).map_err(faulted)?
                }
            };
            *slot = Some(Shift {
                root: request.root,
                from,
                to,
                height: 0,
                size: 1,
                calls: 0,
            });
        }
        for (slot, request) in shifts.iter_mut().zip(requests) {
            if let (Placement::AfterHost(host), Some(shift)) = (request.placement, slot.as_mut()) {
                shift.to = self.host_depth(host, skip, charge)?;
            }
        }
        for shift in shifts.iter_mut().flatten().filter(|shift| shift.walked()) {
            self.measure(shift, skip, charge)?;
        }
        let depth = self.settle(&shifts, usage_depth, charge)?;
        Ok(DepthPlan {
            plan: Plan::Shifts { shifts, skip },
            depth,
        })
    }

    /// The depth below a host after a change: the climbs up its owners as they stand once the
    /// contents `skip` stop owning, plus one.
    fn host_depth<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        host: NodeIx,
        skip: Option<NodeIx>,
        charge: &mut C,
    ) -> Result<usize, HtmlConstructionRecordError<C::Stop>> {
        let records = self.arena.records.len();
        let (mut cursor, mut climbed) = (host, 0usize);
        loop {
            Work::visit(charge)?;
            if cursor.position() >= records {
                return Err(fault(HtmlConstructionCause::UnknownNode));
            }
            if climbed >= records {
                return Err(fault(HtmlConstructionCause::Cycle));
            }
            // The contents the template gives up stop being owned by it.
            let parent = if Some(cursor) == skip {
                None
            } else {
                structural_parent(&self.arena, cursor)
            };
            match parent {
                Some(parent) => {
                    cursor = parent;
                    climbed = plus(climbed, 1).map_err(faulted)?;
                }
                None => return plus(climbed, 1).map_err(faulted),
            }
        }
    }

    /// Walks the subtree of `shift`, counting its records at each old level into the record at
    /// that index, and finds its height and size.
    fn measure<C: Charge<HtmlWork, HtmlResource>>(
        &mut self,
        shift: &mut Shift,
        skip: Option<NodeIx>,
        charge: &mut C,
    ) -> Result<(), HtmlConstructionRecordError<C::Stop>> {
        let budget = walk_budget(self.arena.records.len());
        let records = &mut self.arena.records;
        records
            .get_mut(shift.root.position())
            .ok_or_else(|| fault(HtmlConstructionCause::UnknownNode))?
            .walk_cursor = 0;
        records
            .get_mut(shift.from)
            .ok_or_else(|| fault(HtmlConstructionCause::UnknownNode))?
            .level_count = 1;
        let (mut node, mut level, mut steps) = (shift.root, 0usize, 0usize);
        loop {
            if steps > budget {
                return Err(fault(HtmlConstructionCause::Cycle));
            }
            Work::visit(charge)?;
            let Some(child) = next_edge(&mut self.arena, node, skip).map_err(faulted)? else {
                if node == shift.root {
                    shift.calls = plus(steps, 1).map_err(faulted)?;
                    return Ok(());
                }
                node = self
                    .arena
                    .records
                    .get(node.position())
                    .and_then(owning_parent)
                    .ok_or_else(|| fault(HtmlConstructionCause::ParentChildConflict))?;
                level = level
                    .checked_sub(1)
                    .ok_or_else(|| fault(HtmlConstructionCause::ParentChildConflict))?;
                steps = plus(steps, 1).map_err(faulted)?;
                continue;
            };
            level = plus(level, 1).map_err(faulted)?;
            let absolute = plus(shift.from, level).map_err(faulted)?;
            // A walk descends one level at a time, so a deeper level is new.
            let fresh = level > shift.height;
            if fresh {
                shift.height = level;
            }
            let records = &mut self.arena.records;
            records
                .get_mut(child.position())
                .ok_or_else(|| fault(HtmlConstructionCause::UnknownNode))?
                .walk_cursor = 0;
            let bucket = records
                .get_mut(absolute)
                .ok_or_else(|| fault(HtmlConstructionCause::UnknownNode))?;
            if fresh {
                bucket.level_count = 0;
            }
            bucket.level_count = plus(bucket.level_count, 1).map_err(faulted)?;
            shift.size = plus(shift.size, 1).map_err(faulted)?;
            node = child;
            steps = plus(steps, 1).map_err(faulted)?;
        }
    }

    /// The usage depth the change leaves: the greater of the moved subtrees' new depths and the
    /// greatest depth of every other record. The latter is the old usage depth unless a moved
    /// subtree held the deepest record and every one moved shallower, when a scan down the depth
    /// histogram, less the moved subtrees' own counts at each level, finds it.
    ///
    /// Each level's count of moved records lives in the record at that index. A second walk may
    /// overwrite the first's counts, but only at levels no deeper than its own new top: it starts
    /// at a fresh root, depth zero, and moves deeper, so every level it counts is at most that
    /// top. The scan reads only levels deeper than the greatest new top, which no overwrite
    /// reaches.
    fn settle<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        shifts: &[Option<Shift>; 2],
        current: usize,
        charge: &mut C,
    ) -> Result<usize, HtmlConstructionRecordError<C::Stop>> {
        let moved = || shifts.iter().flatten().filter(|shift| shift.walked());
        let Some(new_top) = moved().map(Shift::new_top).max() else {
            return Ok(current);
        };
        if new_top >= current || moved().all(|shift| shift.top() < current) {
            // Some record outside the moved subtrees is as deep as the old usage depth, or a
            // moved subtree itself reaches the new one.
            return Ok(current.max(new_top));
        }
        let mut level = current;
        loop {
            if level <= new_top {
                return Ok(new_top);
            }
            Work::visit(charge)?;
            let inside = if moved().any(|shift| shift.counted(level)) {
                self.arena
                    .records
                    .get(level)
                    .map_or(0, |bucket| bucket.level_count)
            } else {
                0
            };
            if self.arena.population(level) > inside {
                return Ok(level);
            }
            level = level.saturating_sub(1);
        }
    }
}
