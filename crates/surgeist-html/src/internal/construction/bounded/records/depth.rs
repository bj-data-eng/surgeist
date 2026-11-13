//! Exact depths across the topology changes of the record operations.
//!
//! Every record has its depth within its own tree in the depth column, and the usage depth is the
//! greatest depth in it. A change moves at most two subtrees to new depths, so only their entries
//! change. The change is planned before anything is written: one walk of each moved subtree finds
//! its height and size, and, when a moved subtree held the deepest record and every moved subtree
//! went shallower, a scan down the counts of the depth index finds the usage depth. The writes
//! then place each record of the moved subtrees at its new depth.
//!
//! While only creations and fresh insertions happen, the usage depth is a running maximum and is
//! exact without a count, because those changes only add records at known depths and move records
//! deeper. The first change that needs the count to find the usage depth makes the depth index,
//! one pass over the depth column, and every change after it keeps the index equal to the
//! multiset of depths: a creation adds one at its depth, and a move subtracts each moved record
//! at its old depth and adds it at the new one in one write phase that cannot refuse.
//!
//! A fresh childless node, or a template with only its empty contents, joins below its parent at
//! one known depth and walks nothing.

use super::shape::owning_parent;
use super::*;
use crate::internal::construction::rows::NodeFlags;

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
    /// The root's depth before the change.
    from: usize,
    /// The root's depth after the change.
    to: usize,
    /// The deepest level reached below the root.
    height: usize,
    /// Records reached, including the root.
    size: usize,
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
    /// Whether the storage has a depth index, which the writes keep equal to the depths.
    indexed: bool,
}

impl DepthPlan {
    /// The record visits and writes of placing the records: per placed record the visits of the
    /// walk that finds it, its depth, and the two counts it leaves and joins when the storage has
    /// a depth index.
    pub(super) fn work(&self) -> Result<(usize, usize), HtmlConstructionError> {
        let per_record = plus(1, times(usize::from(self.indexed), 2)?)?;
        match &self.plan {
            Plan::Fast { records, .. } => Ok((0, times(*records, per_record)?)),
            Plan::Shifts { shifts, .. } => {
                let (mut visits, mut writes) = (0usize, 0usize);
                for shift in shifts.iter().flatten().filter(|shift| shift.walked()) {
                    visits = plus(visits, times(shift.size, 2)?)?;
                    writes = plus(writes, times(shift.size, per_record)?)?;
                }
                Ok((visits, writes))
            }
        }
    }

    /// The levels the depth index must hold after the change.
    pub(super) fn levels(&self) -> Result<usize, HtmlConstructionError> {
        plus(self.depth, 1)
    }

    /// Places the planned records at their planned depths. Planning read every record placed
    /// here, so the unknown-node and conflict defects are the only failures, and they are
    /// returned.
    pub(super) fn apply(&self, arena: &mut BuildArena) -> Result<(), HtmlConstructionError> {
        match &self.plan {
            Plan::Fast { node, depth, .. } => {
                arena.place(*node, *depth)?;
                if arena.row(*node)?.flags.has(NodeFlags::HAS_ASSOC)
                    && let Some(BuildAssoc {
                        contents,
                        kind: BuildAssocKind::Owned,
                        ..
                    }) = arena.template_contents_of(*node)
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

/// A walk of the subtree of a record, in preorder: the record, then its ordinary children in
/// order, then its shadow contents, then the template contents it owns, except `skip`. It keeps no
/// cursor in the records: the next record is the first child, the next sibling, or the next edge
/// of the nearest owner above that has one.
#[derive(Clone, Copy, Debug)]
pub(super) struct Walk {
    root: NodeIx,
    skip: Option<NodeIx>,
    node: Option<NodeIx>,
    level: usize,
    steps: usize,
    budget: usize,
    finished: bool,
}

/// The edges of a record that a walk takes after its ordinary children.
#[derive(Clone, Copy)]
enum After {
    Children,
    Shadow,
}

impl Walk {
    /// A walk of the subtree of `root` in `arena` that leaves out the contents `skip`.
    pub(super) fn new(arena: &BuildArena, root: NodeIx, skip: Option<NodeIx>) -> Self {
        Self {
            root,
            skip,
            node: None,
            level: 0,
            steps: 0,
            budget: arena.rows.len().saturating_mul(3).saturating_add(3),
            finished: false,
        }
    }

    /// The contents `node` reaches after `after`: its shadow contents and then the template
    /// contents it owns.
    fn edge(
        &self,
        arena: &BuildArena,
        node: NodeIx,
        after: After,
    ) -> Result<Option<NodeIx>, HtmlConstructionError> {
        if !arena.row(node)?.flags.has(NodeFlags::HAS_ASSOC) {
            return Ok(None);
        }
        if matches!(after, After::Children)
            && let Some(shadow) = arena.shadow_of(node)
        {
            return Ok(Some(shadow));
        }
        Ok(match arena.template_contents_of(node) {
            Some(BuildAssoc {
                contents,
                kind: BuildAssocKind::Owned,
                ..
            }) if Some(contents) != self.skip => Some(contents),
            _ => None,
        })
    }

    /// Counts one step of the walk, a descent to a first child or an edge, or the leaving of a
    /// record. A consistent forest of the storage's records is left at most once for each row and
    /// entered at most once for each row, so a walk that takes more steps than the budget is the
    /// cycle defect.
    fn step(&mut self) -> Result<(), HtmlConstructionError> {
        if self.steps > self.budget {
            return Err(HtmlConstructionCause::Cycle.error());
        }
        self.steps = plus(self.steps, 1)?;
        Ok(())
    }

    fn enter(&mut self, node: NodeIx, deeper: bool) -> Result<(), HtmlConstructionError> {
        if deeper {
            self.level = plus(self.level, 1)?;
        }
        self.node = Some(node);
        Ok(())
    }

    /// The next record and its level below the root, or `None` when the walk is complete. A walk
    /// that takes more steps than a consistent forest of the storage's records allows, going down
    /// or coming up, is the cycle defect.
    pub(super) fn next(
        &mut self,
        arena: &BuildArena,
    ) -> Result<Option<(NodeIx, usize)>, HtmlConstructionError> {
        if self.finished {
            return Ok(None);
        }
        let Some(mut node) = self.node else {
            arena.row(self.root)?;
            self.node = Some(self.root);
            return Ok(Some((self.root, 0)));
        };
        let row = arena.row(node)?;
        let child = match row.first() {
            Some(child) => Some(child),
            None => self.edge(arena, node, After::Children)?,
        };
        if let Some(child) = child {
            self.step()?;
            self.enter(child, true)?;
            return Ok(Some((child, self.level)));
        }
        loop {
            self.step()?;
            if node == self.root {
                self.finished = true;
                return Ok(None);
            }
            let row = arena.row(node)?;
            if let Some(next) = row.next {
                self.enter(next, false)?;
                return Ok(Some((next, self.level)));
            }
            match (row.parent, row.owner()) {
                (Some(parent), _) => {
                    self.ascend()?;
                    node = parent;
                    // The edges of the record that was left, which a walk that came up from a
                    // child takes after the children.
                    if let Some(next) = self.edge(arena, parent, After::Children)? {
                        self.enter(next, true)?;
                        return Ok(Some((next, self.level)));
                    }
                }
                (None, Some(owner)) => {
                    self.ascend()?;
                    node = owner;
                    // The shadow contents come before the template contents, which are the last
                    // edge and leave nothing after them.
                    if row.shadow().is_some()
                        && let Some(next) = self.edge(arena, owner, After::Shadow)?
                    {
                        self.enter(next, true)?;
                        return Ok(Some((next, self.level)));
                    }
                }
                (None, None) => {
                    return Err(HtmlConstructionCause::ParentChildConflict.error());
                }
            }
        }
    }

    fn ascend(&mut self) -> Result<(), HtmlConstructionError> {
        self.level = self
            .level
            .checked_sub(1)
            .ok_or(HtmlConstructionCause::ParentChildConflict.error())?;
        Ok(())
    }
}

impl BuildArena {
    /// Counts `by` more records at `depth` in the depth index, which the storage may not have.
    /// The index has room for the level: its growth was admitted with the change that counts.
    pub(in crate::internal::construction::bounded) fn count(
        &mut self,
        depth: usize,
        by: u32,
    ) -> Result<(), HtmlConstructionError> {
        let Some(index) = self.index.as_mut() else {
            return Ok(());
        };
        let levels = plus(depth, 1)?;
        if levels > index.counts.len() {
            if levels > index.counts.capacity() || levels > index.moved.capacity() {
                return Err(HtmlConstructionCause::CapacityMismatch.error());
            }
            index.counts.resize(levels, 0);
            index.moved.resize(levels, 0);
        }
        let slot = index
            .counts
            .get_mut(depth)
            .ok_or(HtmlConstructionCause::UnknownNode.error())?;
        *slot = slot
            .checked_add(by)
            .ok_or(HtmlConstructionCause::RepresentationExhausted.error())?;
        Ok(())
    }

    /// Counts one record fewer at `depth`; see [`Self::count`].
    pub(in crate::internal::construction::bounded) fn uncount(
        &mut self,
        depth: u32,
    ) -> Result<(), HtmlConstructionError> {
        let Some(index) = self.index.as_mut() else {
            return Ok(());
        };
        let slot = index
            .counts
            .get_mut(depth as usize)
            .ok_or(HtmlConstructionCause::UnknownNode.error())?;
        *slot = slot
            .checked_sub(1)
            .ok_or(HtmlConstructionCause::ReleaseUnderflow.error())?;
        Ok(())
    }

    /// Gives the record a new depth and moves it between the counts of the depth index.
    pub(in crate::internal::construction::bounded) fn place(
        &mut self,
        index: NodeIx,
        depth: usize,
    ) -> Result<(), HtmlConstructionError> {
        let new = u32::try_from(depth)
            .map_err(|_| HtmlConstructionCause::RepresentationExhausted.error())?;
        let slot = self
            .depths
            .get_mut(index.position())
            .ok_or(HtmlConstructionCause::UnknownNode.error())?;
        let old = std::mem::replace(slot, new);
        self.uncount(old)?;
        self.count(depth, 1)
    }

    /// Places the subtree rooted at `root` so that the root sits at depth `to`, leaving out the
    /// template contents `skip`. The walk was planned, so an inconsistent arena is the only way it
    /// fails, and it is reported as the defect it is.
    pub(in crate::internal::construction::bounded) fn relocate_subtree(
        &mut self,
        root: NodeIx,
        to: usize,
        skip: Option<NodeIx>,
    ) -> Result<(), HtmlConstructionError> {
        let mut walk = Walk::new(self, root, skip);
        while let Some((node, level)) = walk.next(self)? {
            self.place(node, plus(to, level)?)?;
        }
        Ok(())
    }
}

impl HtmlConstructionStorage {
    /// The row at `index`, or the unknown-node error.
    pub(super) fn row(&self, index: NodeIx) -> Result<&NodeRow, HtmlConstructionError> {
        self.arena.row(index)
    }

    /// Whether `node` has nothing below it: no children, no shadow root, and contents that it
    /// owns, if any, that have neither. Such a node cannot be an ancestor of anything.
    pub(super) fn is_shallow(&self, node: NodeIx) -> Result<bool, HtmlConstructionError> {
        let row = self.row(node)?;
        if !row.is_childless() {
            return Ok(false);
        }
        if !row.flags.has(NodeFlags::HAS_ASSOC) {
            return Ok(true);
        }
        if self.arena.shadow_of(node).is_some() {
            return Ok(false);
        }
        match self.arena.template_contents_of(node) {
            Some(BuildAssoc {
                contents,
                kind: BuildAssocKind::Owned,
                ..
            }) => {
                let contents = self.row(contents)?;
                Ok(contents.is_childless() && !contents.flags.has(NodeFlags::HAS_ASSOC))
            }
            _ => Ok(true),
        }
    }

    /// Plans the depths after moving the subtrees of `requests`, leaving the contents `skip` out of
    /// every walk, charging the records it visits. `fresh` says that a single subtree has no old
    /// place, so a shallow one needs no walk. The plan makes the depth index when finding the usage
    /// depth needs it, which is a bank growth and can be refused.
    pub(super) fn plan_depths<A: HtmlRecordAdmission>(
        &mut self,
        requests: &[Request],
        skip: Option<NodeIx>,
        fresh: bool,
        admission: &mut A,
    ) -> Result<DepthPlan, HtmlConstructionRecordError<A::Stop>> {
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
            .charge(admission)?;
            if self.is_shallow(root).map_err(faulted)? {
                let depth =
                    plus(self.arena.depth_of(parent).map_err(faulted)?, 1).map_err(faulted)?;
                let template = usize::from(matches!(
                    self.arena.template_contents_of(root),
                    Some(BuildAssoc {
                        kind: BuildAssocKind::Owned,
                        ..
                    })
                ));
                return Ok(DepthPlan {
                    plan: Plan::Fast {
                        node: root,
                        depth,
                        records: 1 + template,
                    },
                    depth: usage_depth.max(plus(depth, template).map_err(faulted)?),
                    indexed: self.arena.index.is_some(),
                });
            }
        }
        let mut shifts: [Option<Shift>; 2] = [None, None];
        for (slot, request) in shifts.iter_mut().zip(requests) {
            Work {
                record_visits: 2,
                ..Work::default()
            }
            .charge(admission)?;
            let from = self.arena.depth_of(request.root).map_err(faulted)?;
            let to = match request.placement {
                Placement::Detached | Placement::AfterHost(_) => 0,
                Placement::Under(parent) => {
                    plus(self.arena.depth_of(parent).map_err(faulted)?, 1).map_err(faulted)?
                }
            };
            *slot = Some(Shift {
                root: request.root,
                from,
                to,
                height: 0,
                size: 1,
            });
        }
        for (slot, request) in shifts.iter_mut().zip(requests) {
            if let (Placement::AfterHost(host), Some(shift)) = (request.placement, slot.as_mut()) {
                shift.to = self.host_depth(host, skip, admission)?;
            }
        }
        for shift in shifts.iter_mut().flatten().filter(|shift| shift.walked()) {
            self.measure(shift, skip, admission)?;
        }
        let depth = match self.settle(&shifts, usage_depth, admission)? {
            Some(depth) => depth,
            None => {
                // Finding the usage depth needs the count of records at each depth, which the
                // storage has not made. The walks count their records into the index as they go,
                // so they walk again.
                self.make_index(admission)?;
                for shift in shifts.iter_mut().flatten().filter(|shift| shift.walked()) {
                    self.measure(shift, skip, admission)?;
                }
                self.settle(&shifts, usage_depth, admission)?
                    .ok_or_else(|| fault(HtmlConstructionCause::CapacityMismatch))?
            }
        };
        Ok(DepthPlan {
            plan: Plan::Shifts { shifts, skip },
            depth,
            indexed: self.arena.index.is_some(),
        })
    }

    /// Makes the depth index: one pass over the depth column that counts the records at each
    /// depth. The pass and the two banks of the index, the counts and the scratch beside them, are
    /// charged and admitted like any other, before anything changes. A refusal leaves the storage
    /// without an index and without the vectors of the banks it grew, and the consumer takes back
    /// the dimensions it recorded for them, so the consumer holds nothing for an index the storage
    /// does not have.
    fn make_index<A: HtmlRecordAdmission>(
        &mut self,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let rows = self.arena.rows.len();
        let levels = plus(self.arena.usage.depth, 1).map_err(faulted)?;
        Work {
            record_visits: rows,
            provider_calls: 2,
            copied_bytes: times(rows, size_of::<u32>()).map_err(faulted)?,
            record_writes: rows,
            ..Work::default()
        }
        .charge(admission)?;
        let growth = |bank| HtmlConstructionBankGrowth {
            bank,
            before: HtmlConstructionBankCapacity::default(),
            minimum_capacity: levels,
            element_size: size_of::<u32>(),
            wrapper_bytes: 0,
        };
        let growths = [
            growth(HtmlConstructionBank::DepthIndex),
            growth(HtmlConstructionBank::DepthScratch),
        ];
        let [chosen_counts, chosen_scratch] = Self::admit_index(growths.map(Some), admission)?;
        let (Some(chosen_counts), Some(chosen_scratch)) = (chosen_counts, chosen_scratch) else {
            return Err(fault(HtmlConstructionCause::CapacityMismatch));
        };
        self.arena.index = Some(DepthIndex::default());
        let mut recorded = [growths[0].before, growths[1].before];
        let [recorded_counts, recorded_scratch] = &mut recorded;
        let outcome = self
            .grow_tracked(
                growths[0],
                chosen_counts,
                admission,
                |storage, additional| storage.reserve_counts_bank(additional),
                recorded_counts,
            )
            .and_then(|()| {
                self.grow_tracked(
                    growths[1],
                    chosen_scratch,
                    admission,
                    |storage, additional| storage.reserve_scratch_bank(additional),
                    recorded_scratch,
                )
            });
        if let Err(error) = outcome {
            self.arena.index = None;
            for (growth, recorded) in growths.iter().zip(recorded).rev() {
                if recorded != growth.before {
                    admission
                        .record_growth(HtmlConstructionGrowthRecord {
                            growth: *growth,
                            chosen: growth.before.capacity,
                            recorded,
                            reached: growth.before,
                        })
                        .map_err(stopped)?;
                }
            }
            return Err(error);
        }
        let Some(index) = self.arena.index.as_mut() else {
            return Err(fault(HtmlConstructionCause::CapacityMismatch));
        };
        index.counts.resize(levels, 0);
        index.moved.resize(levels, 0);
        for depth in &self.arena.depths {
            let slot = index
                .counts
                .get_mut(*depth as usize)
                .ok_or_else(|| fault(HtmlConstructionCause::UnknownNode))?;
            *slot = slot
                .checked_add(1)
                .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?;
        }
        Ok(())
    }

    /// The depth below a host after a change: the climbs up its owners as they stand once the
    /// contents `skip` stop owning, plus one.
    fn host_depth<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        host: NodeIx,
        skip: Option<NodeIx>,
        charge: &mut C,
    ) -> Result<usize, HtmlConstructionRecordError<C::Stop>> {
        let rows = self.arena.rows.len();
        let (mut cursor, mut climbed) = (host, 0usize);
        loop {
            Work::visit(charge)?;
            let row = self.arena.row(cursor).map_err(faulted)?;
            if climbed >= rows {
                return Err(fault(HtmlConstructionCause::Cycle));
            }
            // The contents the template gives up stop being owned by it.
            let parent = if Some(cursor) == skip {
                None
            } else {
                owning_parent(row)
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

    /// Walks the subtree of `shift`, finding its height and size, and counting its records at each
    /// old level into the scratch of the depth index when the storage has one.
    fn measure<C: Charge<HtmlWork, HtmlResource>>(
        &mut self,
        shift: &mut Shift,
        skip: Option<NodeIx>,
        charge: &mut C,
    ) -> Result<(), HtmlConstructionRecordError<C::Stop>> {
        shift.height = 0;
        shift.size = 0;
        let mut walk = Walk::new(&self.arena, shift.root, skip);
        loop {
            // The visits of the step: the record entered, and the edge lookups that find the next.
            Work {
                record_visits: 2,
                ..Work::default()
            }
            .charge(charge)?;
            let Some((_, level)) = walk.next(&self.arena).map_err(faulted)? else {
                return Ok(());
            };
            // A walk descends one level at a time, so a deeper level is new.
            let fresh = level > shift.height || shift.size == 0;
            if level > shift.height {
                shift.height = level;
            }
            shift.size = plus(shift.size, 1).map_err(faulted)?;
            if let Some(index) = self.arena.index.as_mut() {
                let absolute = plus(shift.from, level).map_err(faulted)?;
                let slot = index
                    .moved
                    .get_mut(absolute)
                    .ok_or_else(|| fault(HtmlConstructionCause::UnknownNode))?;
                if fresh {
                    *slot = 0;
                }
                *slot = plus(*slot as usize, 1)
                    .ok()
                    .and_then(|count| u32::try_from(count).ok())
                    .ok_or_else(|| fault(HtmlConstructionCause::RepresentationExhausted))?;
            }
        }
    }

    /// The usage depth the change leaves: the greater of the moved subtrees' new depths and the
    /// greatest depth of every other record. The latter is the old usage depth unless a moved
    /// subtree held the deepest record and every one moved shallower, when a scan down the counts
    /// of the depth index, less the moved subtrees' own counts at each level, finds it. It is
    /// `None` when that scan is needed and the storage has no index.
    ///
    /// Each level's count of moved records lives in the scratch of the index. A second walk may
    /// overwrite the first's counts, but only at levels no deeper than its own new top: it starts
    /// at a fresh root, depth zero, and moves deeper, so every level it counts is at most that
    /// top. The scan reads only levels deeper than the greatest new top, which no overwrite
    /// reaches.
    fn settle<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        shifts: &[Option<Shift>; 2],
        current: usize,
        charge: &mut C,
    ) -> Result<Option<usize>, HtmlConstructionRecordError<C::Stop>> {
        let moved = || shifts.iter().flatten().filter(|shift| shift.walked());
        let Some(new_top) = moved().map(Shift::new_top).max() else {
            return Ok(Some(current));
        };
        if new_top >= current || moved().all(|shift| shift.top() < current) {
            // Some record outside the moved subtrees is as deep as the old usage depth, or a
            // moved subtree itself reaches the new one.
            return Ok(Some(current.max(new_top)));
        }
        let Some(index) = self.arena.index.as_ref() else {
            return Ok(None);
        };
        let mut level = current;
        loop {
            if level <= new_top {
                return Ok(Some(new_top));
            }
            Work::visit(charge)?;
            let inside = if moved().any(|shift| shift.counted(level)) {
                index.moved.get(level).copied().unwrap_or(0)
            } else {
                0
            };
            let population = index.counts.get(level).copied().unwrap_or(0);
            if population > inside {
                return Ok(Some(level));
            }
            level = level.saturating_sub(1);
        }
    }
}

#[cfg(test)]
impl BuildArena {
    /// The depth index, when the storage has made it.
    pub(in crate::internal::construction::bounded) fn depth_index(&self) -> Option<&[u32]> {
        self.index.as_ref().map(|index| index.counts.as_slice())
    }
}
