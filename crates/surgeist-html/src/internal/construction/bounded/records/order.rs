//! Insertion, detachment and the structural check of a transfer of children.
//!
//! The ordinary children of a record are a doubly linked list through the rows: the parent holds
//! its first and last child, and each child its next and previous sibling. Appending a node, inserting
//! it before a sibling and detaching it are edits of a fixed number of links, so their work does
//! not depend on the fanout of the parent or on the place.

use super::depth::{DepthPlan, Placement, Request};
use super::shape::structural_parent;
use super::*;

/// The link writes of removing a node from its parent's list: the neighbour before it or the
/// parent's first child, and the neighbour after it or the parent's last child.
const UNLINK_WRITES: usize = 2;
/// The link writes of placing a node: its parent, its previous and its next sibling, and the
/// previous sibling of the place or the parent's first child with the node it goes before or the
/// parent's last child.
const LINK_WRITES: usize = 5;

impl HtmlConstructionStorage {
    /// Whether `target` lies on the chain of owners above `start`, including `start` itself,
    /// walking one record per visit. The chain has at most as many records as the storage.
    fn is_above<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        start: NodeIx,
        target: NodeIx,
        charge: &mut C,
    ) -> Result<bool, HtmlConstructionRecordError<C::Stop>> {
        let (mut cursor, mut steps) = (Some(start), 0usize);
        while let Some(current) = cursor {
            if current == target {
                return Ok(true);
            }
            if steps >= self.arena.rows.len() {
                // A chain longer than the storage is a cycle of its own.
                return Err(fault(HtmlConstructionCause::Cycle));
            }
            Work::visit(charge)?;
            cursor = structural_parent(&self.arena, current);
            steps = plus(steps, 1).map_err(faulted)?;
        }
        Ok(false)
    }

    /// Refuses with the cycle error when `target` would become its own ancestor by taking
    /// `start` below it. A target with nothing below it cannot, except as `start` itself or as the
    /// owner of the contents `start` is.
    pub(super) fn require_acyclic<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        start: NodeIx,
        target: NodeIx,
        charge: &mut C,
    ) -> Result<(), HtmlConstructionRecordError<C::Stop>> {
        if start == target {
            return Err(fault(HtmlConstructionCause::Cycle));
        }
        Work {
            record_visits: 2,
            ..Work::default()
        }
        .charge(charge)?;
        if self.is_shallow(target).map_err(faulted)? {
            if let Some(BuildAssoc {
                contents,
                kind: BuildAssocKind::Owned,
                ..
            }) = self.arena.template_contents_of(target)
                && contents == start
            {
                return Err(fault(HtmlConstructionCause::Cycle));
            }
            return Ok(());
        }
        if self.is_above(start, target, charge)? {
            return Err(fault(HtmlConstructionCause::Cycle));
        }
        Ok(())
    }

    /// Checks, without changing anything, that moving the children of `from` into `to` is
    /// structurally possible: both can contain children, and `to` does not lie inside `from`'s
    /// subtree.
    pub fn check_child_transfer<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        from: &HtmlBuilderNode,
        to: &HtmlBuilderNode,
        charge: &mut C,
    ) -> Result<(), HtmlConstructionRecordError<C::Stop>> {
        let from = self.operand(from, charge)?;
        let to = self.operand(to, charge)?;
        Work {
            record_visits: 2,
            ..Work::default()
        }
        .charge(charge)?;
        for index in [from, to] {
            HtmlConstructionNode::building(&self.arena, &self.text, &self.attrs, index)
                .check_container()
                .map_err(faulted)?;
        }
        if self.is_above(to, from, charge)? {
            return Err(fault(HtmlConstructionCause::Cycle));
        }
        Ok(())
    }

    /// Inserts `node` into the ordinary children of `parent`, first detaching it from any previous
    /// parent: before `before`, which must be a current child of `parent`, or last when there is
    /// none. Refused with the cycle error when `parent` is `node` or lies inside `node`'s subtree.
    /// Only inserting `node` before itself changes nothing and admits nothing; every other
    /// insertion runs the full ordering and ancestry checks.
    ///
    /// The edit is a fixed number of link writes: the work of an insertion does not depend on the
    /// fanout of `parent`, on the place, or on the number of siblings of `node`.
    ///
    /// Documents and fragments, including template contents and shadow contents, cannot be moved
    /// this way. A `template` element or a shadow host can, because moving it leaves its
    /// association in place.
    pub fn insert<A: HtmlRecordAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        parent: &HtmlBuilderNode,
        before: Option<&HtmlBuilderNode>,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let node = self.operand(node, admission)?;
        let parent = self.operand(parent, admission)?;
        let before = before
            .map(|before| self.operand(before, admission))
            .transpose()?;
        let view =
            |index| HtmlConstructionNode::building(&self.arena, &self.text, &self.attrs, index);
        view(parent).check_container().map_err(faulted)?;
        view(node).check_detachment().map_err(faulted)?;
        if let Some(before) = before {
            view(before)
                .check_reference_child_of(view(parent))
                .map_err(faulted)?;
            if before == node {
                return Ok(());
            }
        }
        let before_usage = self.arena.usage;
        let old_parent = self.row(node).map_err(faulted)?.parent;
        let mut after = before_usage;
        after.edges = plus(after.edges, usize::from(old_parent.is_none())).map_err(faulted)?;
        self.require_acyclic(parent, node, admission)?;
        let depths = self.plan_depths(
            &[Request {
                root: node,
                placement: Placement::Under(parent),
            }],
            None,
            old_parent.is_none(),
            admission,
        )?;
        after.depth = depths.depth;
        let writes = if old_parent.is_some() {
            plus(UNLINK_WRITES, LINK_WRITES).map_err(faulted)?
        } else {
            LINK_WRITES
        };
        self.commit_order(
            OrderChange {
                node,
                parent: Some(parent),
                before,
                writes,
                depths,
            },
            after,
            admission,
        )
    }

    /// Removes `node` from its ordinary parent. The node must be movable: a Document, a fragment
    /// or an owned association fragment is refused even when it has no parent. A movable node
    /// that has no parent is left unchanged, and nothing is admitted.
    pub fn detach<A: HtmlRecordAdmission>(
        &mut self,
        node: &HtmlBuilderNode,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let node = self.operand(node, admission)?;
        HtmlConstructionNode::building(&self.arena, &self.text, &self.attrs, node)
            .check_detachment()
            .map_err(faulted)?;
        if self.row(node).map_err(faulted)?.parent.is_none() {
            return Ok(());
        }
        let before_usage = self.arena.usage;
        let mut after = before_usage;
        after.edges = minus(after.edges, 1).map_err(faulted)?;
        let depths = self.plan_depths(
            &[Request {
                root: node,
                placement: Placement::Detached,
            }],
            None,
            false,
            admission,
        )?;
        after.depth = depths.depth;
        self.commit_order(
            OrderChange {
                node,
                parent: None,
                before: None,
                writes: plus(UNLINK_WRITES, 3).map_err(faulted)?,
                depths,
            },
            after,
            admission,
        )
    }

    /// The charges, admissions, reservation and writes that follow the planning of an insertion
    /// or a detachment.
    fn commit_order<A: HtmlRecordAdmission>(
        &mut self,
        change: OrderChange,
        after: HtmlConstructionUsage,
        admission: &mut A,
    ) -> Result<(), HtmlConstructionRecordError<A::Stop>> {
        let OrderChange {
            node,
            parent,
            before: place,
            writes,
            depths,
        } = change;
        let before = self.arena.usage;
        let (depth_visits, depth_writes) = depths.work().map_err(faulted)?;
        let record_writes =
            plus(plus(writes, depth_writes).map_err(faulted)?, 1).map_err(faulted)?;
        let index_growth = self.index_growth(depths.levels().map_err(faulted)?);
        let usage = HtmlConstructionChange { before, after };
        admission
            .record_largest(HtmlResource::TreeDepth, wide(after.depth))
            .map_err(stopped)?;
        admission.admit_change(self, usage).map_err(stopped)?;
        // The writes are charged once the limits admitted the change, and before any bank grows.
        Work {
            record_visits: depth_visits,
            provider_calls: index_growth.into_iter().flatten().count(),
            copied_bytes: plus(
                plus(
                    times(writes, size_of::<Option<NodeIx>>()).map_err(faulted)?,
                    times(depth_writes, size_of::<u32>()).map_err(faulted)?,
                )
                .map_err(faulted)?,
                size_of::<HtmlConstructionUsage>(),
            )
            .map_err(faulted)?,
            record_writes,
            ..Work::default()
        }
        .charge(admission)?;
        let index_chosen = Self::admit_index(index_growth, admission)?;
        self.grow_index(index_growth, index_chosen, admission)?;
        // The rows the edit reads and the links it rewrites are where planning found them: checked
        // before the last step that can refuse, so that the writes find what they name.
        self.verify_order(node, parent, place).map_err(faulted)?;
        admission.record_change(self, usage).map_err(stopped)?;
        depths.apply(&mut self.arena).map_err(faulted)?;
        self.apply_order(node, parent, place).map_err(faulted)?;
        self.arena.usage = after;
        Ok(())
    }

    /// Checks that the edit applies as planned: every row it reads exists, the node is the entry
    /// of its neighbours that its links say, and the place in the new parent's list exists.
    pub(super) fn verify_order(
        &self,
        node: NodeIx,
        parent: Option<NodeIx>,
        place: Option<NodeIx>,
    ) -> Result<(), HtmlConstructionError> {
        let conflict = || HtmlConstructionCause::ParentChildConflict.error();
        let row = self.row(node)?;
        if let Some(old) = row.parent {
            let old_row = self.row(old)?;
            let before_node = match row.prev {
                Some(prev) => self.row(prev)?.next,
                None => old_row.first(),
            };
            let after_node = match row.next {
                Some(next) => self.row(next)?.prev,
                None => old_row.last(),
            };
            if before_node != Some(node) || after_node != Some(node) {
                return Err(conflict());
            }
        } else if row.prev.is_some() || row.next.is_some() {
            return Err(conflict());
        }
        if let Some(parent) = parent {
            let parent_row = self.row(parent)?;
            match place {
                Some(place) => {
                    let place_row = self.row(place)?;
                    if place_row.parent != Some(parent) {
                        return Err(conflict());
                    }
                    if let Some(prev) = place_row.prev {
                        self.row(prev)?;
                    }
                }
                None => {
                    if let Some(last) = parent_row.last() {
                        self.row(last)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Applies the edit `verify_order` accepted. A lookup that fails here is a defect and is
    /// returned, never skipped. The node is first taken out of its old list, and then put where
    /// the links of the new list are, as they stand.
    fn apply_order(
        &mut self,
        node: NodeIx,
        parent: Option<NodeIx>,
        place: Option<NodeIx>,
    ) -> Result<(), HtmlConstructionError> {
        let row = *self.arena.row(node)?;
        if let Some(old) = row.parent {
            match row.prev {
                Some(prev) => self.arena.row_mut(prev)?.next = row.next,
                None => self.arena.row_mut(old)?.first_child = row.next,
            }
            match row.next {
                Some(next) => self.arena.row_mut(next)?.prev = row.prev,
                None => self.arena.row_mut(old)?.last_child = row.prev,
            }
        }
        {
            let row = self.arena.row_mut(node)?;
            row.parent = None;
            row.prev = None;
            row.next = None;
        }
        let Some(parent) = parent else {
            return Ok(());
        };
        let (prev, next) = match place {
            Some(place) => (self.arena.row(place)?.prev, Some(place)),
            None => (self.arena.row(parent)?.last_child, None),
        };
        match prev {
            Some(prev) => self.arena.row_mut(prev)?.next = Some(node),
            None => self.arena.row_mut(parent)?.first_child = Some(node),
        }
        match next {
            Some(next) => self.arena.row_mut(next)?.prev = Some(node),
            None => self.arena.row_mut(parent)?.last_child = Some(node),
        }
        let row = self.arena.row_mut(node)?;
        row.parent = Some(parent);
        row.prev = prev;
        row.next = next;
        Ok(())
    }
}

/// An insertion or a detachment, planned.
struct OrderChange {
    node: NodeIx,
    parent: Option<NodeIx>,
    /// The sibling the node goes before, or `None` for the end of the list.
    before: Option<NodeIx>,
    writes: usize,
    depths: DepthPlan,
}
