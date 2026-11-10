//! Child-order edits of insert, move and detach.
//!
//! The parent's actual child order is edited in place by the atomic commit, so
//! nothing is written before it: a refused or aborted operation leaves the old
//! order untouched, and the work an operation does is the work of the entries
//! it shifts. An order is a deque, which makes room for an entry by moving the
//! entries between its place and the nearer end, so a place near either end
//! shifts few entries. Planning finds each position by walking the order from
//! both ends alternately, one entry per action, so locating a position costs
//! about twice the entries between it and the nearer end, which the edit
//! shifts anyway: moving every child from the front of one order to the end of
//! another costs a constant for each.
use super::*;

impl Phase {
    /// The walk over all `length` entries of `parent`'s order, which starts at
    /// the last entry: appending and reordering near the end are the common
    /// cases.
    pub(super) fn order(parent: usize, length: usize, old: bool) -> Self {
        Self::Order {
            parent,
            front: 0,
            back: length,
            old,
            from_back: true,
        }
    }
}

impl HtmlConstructionOperation {
    /// One step of the walk that finds the moving node's old place in its old
    /// parent (`old`), or the reference sibling's place in the new parent. The
    /// entries from `front` up to `back` have not been examined; the step
    /// examines the last of them when `from_back` and the first otherwise.
    pub(super) fn locate_order(
        &mut self,
        parent: usize,
        front: usize,
        back: usize,
        old: bool,
        from_back: bool,
    ) -> Result<Phase, HtmlConstructionError> {
        let wanted = if old {
            self.plan.positions[0]
        } else {
            self.plan.positions[2]
        };
        let index = if from_back {
            back.checked_sub(1).filter(|index| *index >= front)
        } else {
            Some(front).filter(|index| *index < back)
        };
        let Some(index) = index else {
            return Err(if old {
                HtmlConstructionCause::ParentChildConflict.error()
            } else {
                HtmlConstructionCause::WrongSibling.error()
            });
        };
        let found = self
            .arena()
            .records
            .get(parent)
            .ok_or(HtmlConstructionCause::UnknownNode)?
            .children
            .get(index)
            == Some(&wanted);
        if !found {
            return Ok(Phase::Order {
                parent,
                front: if from_back { front } else { index + 1 },
                back: if from_back { index } else { back },
                old,
                from_back: !from_back,
            });
        }
        if old {
            self.plan.old_position = index;
            self.after_old_place()
        } else {
            self.plan.insertion_position = index;
            self.after_new_place()
        }
    }
    /// Continues once the moving node's place in its old parent is known.
    pub(super) fn after_old_place(&mut self) -> Result<Phase, HtmlConstructionError> {
        if !matches!(self.command(), HtmlConstructionCommand::Insert { .. }) {
            let mut plan = std::mem::take(&mut self.plan);
            let result = self.after_topology(&mut plan);
            self.plan = plan;
            return result;
        }
        self.locate_new_place()
    }
    /// Starts finding the place in the new parent: before the reference
    /// sibling, or last when there is none.
    pub(super) fn locate_new_place(&mut self) -> Result<Phase, HtmlConstructionError> {
        let parent = self.plan.positions[1];
        if self.command().operand(2).is_some() {
            let remaining = self
                .arena()
                .records
                .get(parent)
                .ok_or(HtmlConstructionCause::UnknownNode)?
                .children
                .len();
            return Ok(Phase::order(parent, remaining, false));
        }
        self.after_new_place()
    }
    /// Continues once the place in the new parent is known. The place counts
    /// entries of the order before the node leaves its old place.
    fn after_new_place(&mut self) -> Result<Phase, HtmlConstructionError> {
        let parent = self.plan.positions[1];
        if self.plan.old_parent == Some(parent)
            && self.plan.old_position < self.plan.insertion_position
        {
            self.plan.insertion_position = self.plan.insertion_position.saturating_sub(1);
        }
        Ok(Phase::Cycle {
            cursor: Some(parent),
            target: self.plan.positions[0],
            steps: 0,
        })
    }
    /// Grows the parent's actual child order in place. The previous order stays
    /// intact: nothing is pushed before the atomic commit, and a grown bank
    /// survives an abort with its new incarnation, like the record bank.
    pub(super) fn reserve_children(
        &mut self,
    ) -> Result<
        (Phase, Option<HtmlConstructionAllocationSnapshot>),
        (
            HtmlConstructionError,
            Option<HtmlConstructionAllocationSnapshot>,
        ),
    > {
        let parent = self.plan.positions[1];
        let capacity = self.plan.children_capacity.unwrap_or(self.push_required());
        checked_layout(capacity, size_of::<usize>()).map_err(|error| (error, None))?;
        let generation = self
            .arena()
            .temporary_generation
            .checked_add(1)
            .ok_or((HtmlConstructionCause::RepresentationExhausted.error(), None))?;
        let arena = &mut self.owner.mutable().map_err(|error| (error, None))?.arena;
        let record = arena
            .records
            .get_mut(parent)
            .ok_or((HtmlConstructionCause::UnknownNode.error(), None))?;
        let additional = capacity
            .checked_sub(record.children.len())
            .ok_or((HtmlConstructionCause::CapacityMismatch.error(), None))?;
        let result = record.children.try_reserve_exact(additional);
        if result.is_ok() {
            record.children_generation = generation;
            arena.temporary_generation = generation;
        }
        let record = arena
            .records
            .get(parent)
            .ok_or((HtmlConstructionCause::UnknownNode.error(), None))?;
        let snapshot = HtmlConstructionAllocationSnapshot::deque(
            &arena.identity,
            AllocationRole::Children(parent),
            record.children_generation,
            &record.children,
        )
        .map_err(|error| (error, None))?;
        match result {
            Ok(()) => Ok((Phase::Commit, Some(snapshot))),
            Err(_) => Err((
                HtmlConstructionCause::AllocationFailed.error(),
                Some(snapshot),
            )),
        }
    }
}

impl Plan {
    /// The entries the commit's in-place edits write: the node's parent link,
    /// the removed and inserted entries, and every entry moved to close or
    /// open their places. A deque closes or opens a place by moving the
    /// entries between it and the nearer end of the order (documented for
    /// `VecDeque::remove` and `VecDeque::insert`).
    pub(super) fn order_writes(&self, arena: &Arena) -> Result<usize, HtmlConstructionError> {
        let length = |parent: usize| {
            arena
                .records
                .get(parent)
                .map(|record| record.children.len())
                .ok_or(HtmlConstructionCause::UnknownNode)
        };
        let parent = self.positions[1];
        let mut writes = 1usize;
        let mut target = if self.inserts { length(parent)? } else { 0 };
        if let Some(old) = self.old_parent {
            let old_length = length(old)?;
            let behind = old_length.saturating_sub(self.old_position.saturating_add(1));
            writes = add(add(writes, 1)?, behind.min(self.old_position))?;
            if old == parent {
                target = old_length.saturating_sub(1);
            }
        }
        if self.inserts {
            let behind = target.saturating_sub(self.insertion_position);
            writes = add(add(writes, 1)?, behind.min(self.insertion_position))?;
        }
        Ok(writes)
    }
    /// Whether the planned edits apply to `arena` exactly as located: every
    /// entry is where planning found it (`ParentChildConflict` otherwise) and
    /// the node and new parent exist and the push cannot reallocate
    /// (`CapacityMismatch` otherwise).
    pub(super) fn check_order_edits(&self, arena: &Arena) -> Result<(), HtmlConstructionError> {
        let [node, parent, before] = self.positions;
        if arena.records.get(node).is_none() {
            return Err(HtmlConstructionCause::CapacityMismatch.into());
        }
        let mut removed_before = None;
        if let Some(old) = self.old_parent {
            let children = &arena
                .records
                .get(old)
                .ok_or(HtmlConstructionCause::ParentChildConflict)?
                .children;
            if children.get(self.old_position) != Some(&node) {
                return Err(HtmlConstructionCause::ParentChildConflict.into());
            }
            removed_before = (old == parent).then_some(self.old_position);
        }
        if !self.inserts {
            return Ok(());
        }
        let record = arena
            .records
            .get(parent)
            .ok_or(HtmlConstructionCause::CapacityMismatch)?;
        let length = record
            .children
            .len()
            .saturating_sub(usize::from(removed_before.is_some()));
        if length >= record.children.capacity() {
            return Err(HtmlConstructionCause::CapacityMismatch.into());
        }
        let reference = self.has_before.then(|| {
            // The entry's place before the node leaves its own.
            match removed_before {
                Some(old) if self.insertion_position >= old => {
                    self.insertion_position.saturating_add(1)
                }
                _ => self.insertion_position,
            }
        });
        match reference {
            Some(place) if record.children.get(place) == Some(&before) => Ok(()),
            None if self.insertion_position == length => Ok(()),
            _ => Err(HtmlConstructionCause::ParentChildConflict.into()),
        }
    }
    /// Applies the edits `check_order_edits` accepted. Each guard repeats what
    /// that check proved, so no edit can reach outside an order or reallocate.
    pub(super) fn apply_order_edits(&self, arena: &mut Arena) {
        let [node, parent, _] = self.positions;
        if let Some(old) = self.old_parent
            && let Some(record) = arena.records.get_mut(old)
            && self.old_position < record.children.len()
        {
            record.children.remove(self.old_position);
        }
        if self.inserts
            && let Some(record) = arena.records.get_mut(parent)
            && self.insertion_position <= record.children.len()
            && record.children.len() < record.children.capacity()
        {
            record.children.insert(self.insertion_position, node);
        }
    }
}
