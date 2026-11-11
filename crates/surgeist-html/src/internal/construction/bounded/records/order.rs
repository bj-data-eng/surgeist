//! Insertion, detachment and the structural check of a transfer of children.
//!
//! The child order of a record is a deque: it makes room for an entry by moving the entries
//! between its place and the nearer end, so a place near either end shifts few entries. A place is
//! found by walking the order from both ends alternately, one entry per visit, starting at the
//! last entry, so locating a place costs about twice the entries between it and the nearer end,
//! which the edit shifts anyway. Appending a node costs a constant whatever the fanout.

use super::depth::{DepthPlan, Placement, Request};
use super::*;
use crate::internal::construction::bounded::operations::structural_parent;

/// The place of an entry in a child order, and what the edit of the order writes.
#[derive(Clone, Copy, Debug, Default)]
struct Edit {
    old_parent: Option<NodeIx>,
    old_position: usize,
    /// The place in the new parent's order, counted without the moving node.
    insertion_position: usize,
}

impl Edit {
    /// The entries the edit writes: the node's parent link, the removed and inserted entries, and
    /// every entry moved to close or open their places. A deque closes or opens a place by moving
    /// the entries between it and the nearer end.
    fn writes(
        &self,
        old_length: usize,
        new_length: usize,
        inserts: bool,
        same_parent: bool,
    ) -> Result<usize, HtmlConstructionError> {
        let mut writes = 1usize;
        let mut target = new_length;
        if self.old_parent.is_some() {
            let behind = old_length.saturating_sub(self.old_position.saturating_add(1));
            writes = plus(plus(writes, 1)?, behind.min(self.old_position))?;
            if same_parent {
                target = old_length.saturating_sub(1);
            }
        }
        if inserts {
            let behind = target.saturating_sub(self.insertion_position);
            writes = plus(plus(writes, 1)?, behind.min(self.insertion_position))?;
        }
        Ok(writes)
    }
}

impl HtmlConstructionStorage {
    /// The place of `wanted` among the children of `parent`, found by the alternating walk from
    /// the last entry, charging each entry examined. `not_found` is the cause when it is not there.
    fn locate<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        parent: NodeIx,
        wanted: NodeIx,
        not_found: HtmlConstructionCause,
        charge: &mut C,
    ) -> Result<usize, HtmlConstructionRecordError<C::Stop>> {
        let children = &self
            .arena
            .records
            .get(parent.position())
            .ok_or_else(|| fault(HtmlConstructionCause::UnknownNode))?
            .children;
        let (mut front, mut back, mut from_back) = (0usize, children.len(), true);
        loop {
            let index = if from_back {
                back.checked_sub(1).filter(|index| *index >= front)
            } else {
                Some(front).filter(|index| *index < back)
            };
            let Some(index) = index else {
                return Err(fault(not_found));
            };
            Work::visit(charge)?;
            if children.get(index) == Some(&wanted) {
                return Ok(index);
            }
            if from_back {
                back = index;
            } else {
                front = plus(index, 1).map_err(faulted)?;
            }
            from_back = !from_back;
        }
    }

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
            if steps >= self.arena.records.len() {
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
            if let Some(TemplateContents::Owned(contents)) =
                self.record(target).map_err(faulted)?.template_contents
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
            HtmlConstructionNode {
                arena: &self.arena,
                index,
            }
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
        let view = |index| HtmlConstructionNode {
            arena: &self.arena,
            index,
        };
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
        let old_parent = self.record(node).map_err(faulted)?.parent;
        let mut after = before_usage;
        after.edges = plus(after.edges, usize::from(old_parent.is_none())).map_err(faulted)?;
        let mut edit = Edit {
            old_parent,
            ..Edit::default()
        };
        if let Some(old) = old_parent {
            edit.old_position = self.locate(
                old,
                node,
                HtmlConstructionCause::ParentChildConflict,
                admission,
            )?;
        }
        let parent_length = self.record(parent).map_err(faulted)?.children.len();
        edit.insertion_position = match before {
            Some(before) => self.locate(
                parent,
                before,
                HtmlConstructionCause::WrongSibling,
                admission,
            )?,
            None => parent_length,
        };
        if old_parent == Some(parent) && edit.old_position < edit.insertion_position {
            edit.insertion_position = minus(edit.insertion_position, 1).map_err(faulted)?;
        }
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
        let same_parent = old_parent == Some(parent);
        let old_length = match old_parent {
            Some(old) => self.record(old).map_err(faulted)?.children.len(),
            None => 0,
        };
        let writes = edit
            .writes(old_length, parent_length, true, same_parent)
            .map_err(faulted)?;
        let growth = if same_parent {
            None
        } else {
            self.children_growth(parent, plus(parent_length, 1).map_err(faulted)?)
        };
        self.commit_order(
            OrderChange {
                node,
                parent: Some(parent),
                edit,
                writes,
                depths,
                growth,
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
        HtmlConstructionNode {
            arena: &self.arena,
            index: node,
        }
        .check_detachment()
        .map_err(faulted)?;
        let Some(old) = self.record(node).map_err(faulted)?.parent else {
            return Ok(());
        };
        let before_usage = self.arena.usage;
        let mut after = before_usage;
        after.edges = minus(after.edges, 1).map_err(faulted)?;
        let edit = Edit {
            old_parent: Some(old),
            old_position: self.locate(
                old,
                node,
                HtmlConstructionCause::ParentChildConflict,
                admission,
            )?,
            insertion_position: 0,
        };
        let old_length = self.record(old).map_err(faulted)?.children.len();
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
        let writes = edit.writes(old_length, 0, false, false).map_err(faulted)?;
        self.commit_order(
            OrderChange {
                node,
                parent: None,
                edit,
                writes,
                depths,
                growth: None,
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
            edit,
            writes,
            depths,
            growth,
        } = change;
        let before = self.arena.usage;
        let (depth_visits, depth_writes) = depths.work().map_err(faulted)?;
        let record_writes =
            plus(plus(writes, depth_writes).map_err(faulted)?, 1).map_err(faulted)?;
        let usage = HtmlConstructionChange { before, after };
        admission
            .record_largest(HtmlResource::TreeDepth, wide(after.depth))
            .map_err(stopped)?;
        admission.admit_change(self, usage).map_err(stopped)?;
        // The writes are charged once the limits admitted the change, and before any bank grows.
        Work {
            record_visits: depth_visits,
            provider_calls: usize::from(growth.is_some()),
            copied_bytes: plus(
                plus(
                    times(writes, size_of::<NodeIx>()).map_err(faulted)?,
                    times(depth_writes, size_of::<usize>()).map_err(faulted)?,
                )
                .map_err(faulted)?,
                size_of::<HtmlConstructionUsage>(),
            )
            .map_err(faulted)?,
            record_writes,
            ..Work::default()
        }
        .charge(admission)?;
        if let (Some(growth), Some(parent)) = (growth, parent) {
            let chosen = Self::admit_bank(growth, admission)?;
            self.grow(growth, chosen, admission, |storage, additional| {
                storage.reserve_child_bank(parent, additional)
            })?;
        }
        // The entries are where planning found them and the order has the room: checked before
        // the last step that can refuse, so that the writes find what they name.
        self.verify_order(node, parent, &edit).map_err(faulted)?;
        admission.record_change(self, usage).map_err(stopped)?;
        depths.apply(&mut self.arena);
        self.apply_order(node, parent, &edit).map_err(faulted)?;
        self.arena.usage = after;
        Ok(())
    }

    /// Checks that the edit applies as planned: the node is the entry planning located, the
    /// position in the new parent's order exists, and the order has room for the entry.
    fn verify_order(
        &self,
        node: NodeIx,
        parent: Option<NodeIx>,
        edit: &Edit,
    ) -> Result<(), HtmlConstructionError> {
        self.record(node)?;
        let mut removed = 0usize;
        if let Some(old) = edit.old_parent {
            if self.record(old)?.children.get(edit.old_position) != Some(&node) {
                return Err(HtmlConstructionCause::ParentChildConflict.error());
            }
            if Some(old) == parent {
                removed = 1;
            }
        }
        if let Some(parent) = parent {
            let record = self.record(parent)?;
            let length = minus(record.children.len(), removed)?;
            if edit.insertion_position > length {
                return Err(HtmlConstructionCause::ParentChildConflict.error());
            }
            if length >= record.children.capacity() {
                return Err(HtmlConstructionCause::CapacityMismatch.error());
            }
        }
        Ok(())
    }

    /// Applies the edit `verify_order` accepted. A lookup that fails here is a defect and is
    /// returned, never skipped.
    fn apply_order(
        &mut self,
        node: NodeIx,
        parent: Option<NodeIx>,
        edit: &Edit,
    ) -> Result<(), HtmlConstructionError> {
        if let Some(old) = edit.old_parent {
            self.record_mut(old)?
                .children
                .remove(edit.old_position)
                .ok_or(HtmlConstructionCause::ParentChildConflict.error())?;
        }
        if let Some(parent) = parent {
            let children = &mut self.record_mut(parent)?.children;
            if edit.insertion_position > children.len() || children.len() >= children.capacity() {
                return Err(HtmlConstructionCause::CapacityMismatch.error());
            }
            children.insert(edit.insertion_position, node);
        }
        self.record_mut(node)?.parent = parent;
        Ok(())
    }
}

/// An insertion or a detachment, planned.
struct OrderChange {
    node: NodeIx,
    parent: Option<NodeIx>,
    edit: Edit,
    writes: usize,
    depths: DepthPlan,
    growth: Option<HtmlConstructionBankGrowth>,
}
