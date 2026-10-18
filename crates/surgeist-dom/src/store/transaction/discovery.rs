//! Prepare all derived participant changes against one complete candidate
//! topology, before committing the corresponding semantic fields or sequence.
use super::super::*;
use super::ordered_members::{OrderPair, OrderView};
use super::{NodeChange, RelationChange, Undo, family_discovery, role_facts};

pub(in crate::store) struct Prepared {
    named: Option<super::slot_names::Plan>,
    families: Option<family_discovery::FamilyPlan>,
    roles: Option<role_facts::Plan>,
    containers: Option<hosted::containers::ContainerPlan>,
}
impl Draft {
    pub(in crate::store) fn prepare_discovery_node(
        &self,
        change: NodeChange<'_>,
        after: OrderView<'_>,
        cancel: &AtomicBool,
    ) -> Result<Prepared, DomError> {
        let order = OrderPair {
            before: self.order_view(),
            after,
        };
        Ok(Prepared {
            named: self.slot_names.prepare_node(self, change, order, cancel)?,
            families: self.families.prepare_node(self, change, order, cancel)?,
            roles: self.role_facts.prepare_node(self, change, order, cancel)?,
            containers: self.containers.prepare_node(self, change, order, cancel)?,
        })
    }
    pub(in crate::store) fn prepare_discovery_relation(
        &self,
        change: RelationChange<'_>,
        after: OrderView<'_>,
        cancel: &AtomicBool,
    ) -> Result<Prepared, DomError> {
        let order = OrderPair {
            before: self.order_view(),
            after,
        };
        Ok(Prepared {
            named: self
                .slot_names
                .prepare_relation(self, change, order, cancel)?,
            families: self
                .families
                .prepare_relation(self, change, order, cancel)?,
            roles: self
                .role_facts
                .prepare_relation(self, change, order, cancel)?,
            containers: self
                .containers
                .prepare_relation(self, change, order, cancel)?,
        })
    }
    pub(in crate::store) fn prepare_discovery_visibility(
        &self,
        key: Key,
        was_private: bool,
        is_private: bool,
        cancel: &AtomicBool,
    ) -> Result<Prepared, DomError> {
        let order = OrderPair {
            before: self.order_view(),
            after: self.order_view(),
        };
        Ok(Prepared {
            named: self.slot_names.prepare_visibility(
                self,
                key,
                was_private,
                is_private,
                cancel,
            )?,
            families: self.families.prepare_visibility(
                self,
                key,
                was_private,
                is_private,
                order,
                cancel,
            )?,
            roles: self.role_facts.prepare_visibility(
                self,
                key,
                was_private,
                is_private,
                order,
                cancel,
            )?,
            containers: self.containers.prepare_visibility(
                self,
                key,
                was_private,
                is_private,
                order,
                cancel,
            )?,
        })
    }
    pub(in crate::store) fn commit_discovery(&mut self, prepared: Prepared, retain: bool) {
        if let Some(plan) = prepared.named {
            let old = plan.commit(&mut self.slot_names);
            if retain {
                self.journal.push_prepared(Undo::SlotNames(old));
            }
        }
        if let Some(plan) = prepared.families {
            let old = plan.commit(&mut self.families);
            if retain {
                self.journal.push_prepared(Undo::Families(old));
            }
        }
        if let Some(plan) = prepared.roles {
            let old = plan.commit(&mut self.role_facts);
            if retain {
                self.journal.push_prepared(Undo::Roles(old));
            }
        }
        if let Some(plan) = prepared.containers {
            let old = plan.commit(&mut self.containers);
            if retain {
                self.journal.push_prepared(Undo::Containers(old));
            }
        }
    }
}
