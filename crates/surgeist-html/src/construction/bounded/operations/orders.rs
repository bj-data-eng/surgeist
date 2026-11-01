//! Actual pending child orders, not another construction result or tree.
use super::*;

#[derive(Debug)]
pub struct HtmlConstructionChildOrderTransfer {
    position: usize,
    source: HtmlConstructionAllocationSnapshot,
    target: HtmlConstructionAllocationSnapshot,
}
impl HtmlConstructionChildOrderTransfer {
    pub fn into_parts(
        self,
    ) -> (
        usize,
        HtmlConstructionAllocationSnapshot,
        HtmlConstructionAllocationSnapshot,
    ) {
        (self.position, self.source, self.target)
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn source(&self) -> &HtmlConstructionAllocationSnapshot {
        &self.source
    }
    pub fn target(&self) -> &HtmlConstructionAllocationSnapshot {
        &self.target
    }
}
#[derive(Debug)]
struct RetiringOrder {
    values: Vec<usize>,
    snapshot: HtmlConstructionAllocationSnapshot,
}
/// Actual flat pending/retired banks, including their initialized stop prefix.
#[derive(Debug, Default)]
pub struct HtmlConstructionOrderRetirement {
    orders: [Option<RetiringOrder>; 2],
}
pub struct HtmlConstructionOrderRetirementAction<'a> {
    owner: &'a mut HtmlConstructionOrderRetirement,
    slot: usize,
}
impl HtmlConstructionOrderRetirement {
    pub fn next_action(&mut self) -> Option<HtmlConstructionOrderRetirementAction<'_>> {
        (0..2)
            .find(|&slot| self.orders[slot].is_some())
            .map(|slot| HtmlConstructionOrderRetirementAction { owner: self, slot })
    }
}
impl HtmlConstructionOrderRetirementAction<'_> {
    pub fn allocation(&self) -> &HtmlConstructionAllocationSnapshot {
        &self.owner.orders[self.slot]
            .as_ref()
            .expect("retained actual order")
            .snapshot
    }
    pub fn perform(self) -> HtmlConstructionRetired {
        let RetiringOrder { values, snapshot } = self.owner.orders[self.slot]
            .take()
            .expect("retained actual order");
        drop(values);
        HtmlConstructionRetired { snapshot }
    }
}

impl HtmlConstructionOperation {
    pub(super) fn initialize_order(
        &mut self,
        slot: usize,
    ) -> Result<
        (Phase, Option<HtmlConstructionAllocationSnapshot>),
        (
            HtmlConstructionError,
            Option<HtmlConstructionAllocationSnapshot>,
        ),
    > {
        let parent = self.plan.order_parents[slot];
        let storage = self.owner.as_mut().expect("pending storage").mutable();
        let generation = storage
            .arena
            .temporary_generation
            .checked_add(1)
            .ok_or((HtmlConstructionError::RepresentationExhausted, None))?;
        storage.arena.temporary_generation = generation;
        let snapshot = HtmlConstructionAllocationSnapshot::vector(
            &storage.arena.identity,
            AllocationRole::PendingChildOrder { slot, parent },
            generation,
            &self.plan.orders[slot],
        )
        .map_err(|error| (error, None))?;
        self.plan.order_snapshots[slot] = Some(snapshot.clone());
        Ok((Phase::ReserveOrder(slot), Some(snapshot)))
    }
    pub(super) fn reserve_order(
        &mut self,
        slot: usize,
    ) -> Result<
        (Phase, Option<HtmlConstructionAllocationSnapshot>),
        (
            HtmlConstructionError,
            Option<HtmlConstructionAllocationSnapshot>,
        ),
    > {
        let required = self.plan.order_required[slot];
        checked_layout(required, size_of::<usize>()).map_err(|error| (error, None))?;
        let generation = self
            .arena()
            .temporary_generation
            .checked_add(1)
            .ok_or((HtmlConstructionError::RepresentationExhausted, None))?;
        let order = &mut self.plan.orders[slot];
        let result = order.try_reserve_exact(required - order.len());
        let snapshot = self.plan.order_snapshots[slot]
            .as_mut()
            .expect("initialized actual order bank");
        if result.is_ok() {
            snapshot.id.generation = generation;
        }
        snapshot.capacity = order.capacity();
        snapshot.payload_bytes = order
            .capacity()
            .checked_mul(size_of::<usize>())
            .ok_or((HtmlConstructionError::RepresentationExhausted, None))?;
        let snapshot = snapshot.clone();
        if result.is_ok() {
            self.owner
                .as_mut()
                .expect("pending storage")
                .mutable()
                .arena
                .temporary_generation = generation;
        }
        match result {
            Ok(()) => Ok((Phase::CopyOrder { slot, offset: 0 }, Some(snapshot))),
            Err(_) => Err((HtmlConstructionError::AllocationFailed, Some(snapshot))),
        }
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
        let capacity = self
            .plan
            .children_capacity
            .unwrap_or(self.append_required());
        checked_layout(capacity, size_of::<usize>()).map_err(|error| (error, None))?;
        let generation = self
            .arena()
            .temporary_generation
            .checked_add(1)
            .ok_or((HtmlConstructionError::RepresentationExhausted, None))?;
        let arena = &mut self
            .owner
            .as_mut()
            .expect("pending storage")
            .mutable()
            .arena;
        let record = arena
            .records
            .get_mut(parent)
            .ok_or((HtmlConstructionError::UnknownNode, None))?;
        let additional = capacity
            .checked_sub(record.children.len())
            .ok_or((HtmlConstructionError::CapacityMismatch, None))?;
        let result = record.children.try_reserve_exact(additional);
        if result.is_ok() {
            record.children_generation = generation;
            arena.temporary_generation = generation;
        }
        let record = arena
            .records
            .get(parent)
            .ok_or((HtmlConstructionError::UnknownNode, None))?;
        let snapshot = HtmlConstructionAllocationSnapshot::vector(
            &arena.identity,
            AllocationRole::Children(parent),
            record.children_generation,
            &record.children,
        )
        .map_err(|error| (error, None))?;
        match result {
            Ok(()) => Ok((Phase::Commit, Some(snapshot))),
            Err(_) => Err((HtmlConstructionError::AllocationFailed, Some(snapshot))),
        }
    }
    pub(super) fn read_order(
        &mut self,
        slot: usize,
        offset: usize,
    ) -> Result<Phase, HtmlConstructionError> {
        let parent = self.plan.order_parents[slot];
        let children = &self.arena().records[parent].children;
        if let Some(&child) = children.get(offset) {
            if child == self.plan.positions[0] {
                return Ok(Phase::CopyOrder {
                    slot,
                    offset: offset + 1,
                });
            }
            if slot == 0
                && matches!(self.command(), HtmlConstructionCommand::Insert { .. })
                && self.command().operand(2).is_some()
                && child == self.plan.positions[2]
            {
                return Ok(Phase::PushInserted {
                    slot,
                    offset,
                    following: Some(child),
                });
            }
            Ok(Phase::PushExisting {
                slot,
                offset,
                child,
            })
        } else if slot == 0
            && matches!(self.command(), HtmlConstructionCommand::Insert { .. })
            && !self.plan.order_inserted[slot]
        {
            if self.command().operand(2).is_some() {
                return Err(HtmlConstructionError::WrongSibling);
            }
            Ok(Phase::PushInserted {
                slot,
                offset,
                following: None,
            })
        } else {
            Ok(self.after_order(slot))
        }
    }
    fn after_order(&self, slot: usize) -> Phase {
        if slot + 1 < self.plan.order_count {
            Phase::InitializeOrder(slot + 1)
        } else {
            Phase::TransferOrders
        }
    }
    pub(super) fn push_order(&mut self, slot: usize, value: usize) {
        self.plan.orders[slot].push(value);
        self.plan.order_snapshots[slot]
            .as_mut()
            .expect("initialized order bank")
            .initialized_len = self.plan.orders[slot].len();
    }
    pub(super) fn push_inserted(
        &mut self,
        slot: usize,
        offset: usize,
        following: Option<usize>,
    ) -> Phase {
        self.push_order(slot, self.plan.positions[0]);
        self.plan.order_inserted[slot] = true;
        if let Some(child) = following {
            Phase::PushExisting {
                slot,
                offset,
                child,
            }
        } else {
            self.after_order(slot)
        }
    }
    pub(super) fn prepare_order_transfers(&mut self) -> Result<Phase, HtmlConstructionError> {
        let mut entries = 0;
        for slot in 0..self.plan.order_count {
            let parent = self.plan.order_parents[slot];
            let record = &self.arena().records[parent];
            let generation = self
                .arena()
                .temporary_generation
                .checked_add(1)
                .ok_or(HtmlConstructionError::RepresentationExhausted)?;
            let mut retired = HtmlConstructionAllocationSnapshot::vector(
                &self.arena().identity,
                AllocationRole::Children(parent),
                record.children_generation,
                &record.children,
            )?;
            retired.initialized_len = 0;
            let target = HtmlConstructionAllocationSnapshot::vector(
                &self.arena().identity,
                AllocationRole::Children(parent),
                generation,
                &self.plan.orders[slot],
            )?;
            entries = add(entries, target.initialized_len())?;
            self.owner
                .as_mut()
                .expect("pending owner")
                .mutable()
                .arena
                .temporary_generation = generation;
            self.plan.order_retired_snapshots[slot] = Some(retired);
            self.plan.transfers[slot] = Some(HtmlConstructionChildOrderTransfer {
                position: parent,
                source: self.plan.order_snapshots[slot]
                    .as_ref()
                    .expect("actual pending order")
                    .clone(),
                target,
            });
        }
        self.plan.scratch_entries = entries;
        Ok(Phase::Commit)
    }
    pub(super) fn abort_orders(&mut self) -> HtmlConstructionOrderRetirement {
        let mut orders = HtmlConstructionOrderRetirement::default();
        for slot in 0..2 {
            if let Some(snapshot) = self.plan.order_snapshots[slot].take() {
                orders.orders[slot] = Some(RetiringOrder {
                    values: std::mem::take(&mut self.plan.orders[slot]),
                    snapshot,
                });
            }
        }
        orders
    }
}
impl Plan {
    pub(super) fn commit_orders(&mut self, arena: &mut Arena) -> HtmlConstructionOrderRetirement {
        let mut orders = HtmlConstructionOrderRetirement::default();
        for slot in 0..self.order_count {
            let parent = self.order_parents[slot];
            let record = &mut arena.records[parent];
            let mut values =
                std::mem::replace(&mut record.children, std::mem::take(&mut self.orders[slot]));
            values.clear();
            record.children_generation = self.transfers[slot]
                .as_ref()
                .expect("real prepared transfer")
                .target
                .id
                .generation;
            orders.orders[slot] = Some(RetiringOrder {
                values,
                snapshot: self.order_retired_snapshots[slot]
                    .take()
                    .expect("actual old bank snapshot"),
            });
        }
        orders
    }
}
