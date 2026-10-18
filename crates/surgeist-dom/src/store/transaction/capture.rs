//! Controlled whole-snapshot inputs. Semantic State roots contain identity and
//! receipts only; a live owner recovers or admits the actual missing frontier.
use std::{mem::size_of, ops::Deref, sync::Arc};

use super::super::*;
use super::{inbound, queues, ranges};

/// Independently owned frontiers. Nested bank records keep their child controls
/// as dependencies, so adopting an active subtree retains all of its custody.
#[derive(Clone, Default)]
pub(in crate::store) struct StateFrontiers {
    pub auxiliary: super::auxiliary::Auxiliary,
    pub slots: bank::BankCustody<Slot>,
    pub ranges: bank::BankCustody<crate::range::Slot<NodeId>>,
    pub inbound: inbound::InboundCustody,
    pub controls: super::controls::ControlCustody,
    pub slot_index: super::slot_index::SlotCustody,
    pub families: super::family_discovery::FamilyCustody,
    pub role_facts: super::role_facts::Custody,
    pub containers: hosted::containers::ContainerCustody,
    pub range_dependencies: ranges::DependencyCustody,
    pub transitions: view_transition::TransitionCustody,
    pub signal_targets: queues::TargetsCustody,
    pub native_targets: queues::TargetsCustody,
    pub details_targets: queues::TargetsCustody,
}

struct CapturedFrontier {
    // Attached<T> retains the root cache outside this data target, so child
    // controls and the actual target drop before the root's weak-body debit.
    frontiers: StateFrontiers,
}

#[derive(Clone)]
pub(in crate::store) struct StateCustody(bank::Attached<CapturedFrontier>);

impl Deref for StateCustody {
    type Target = StateFrontiers;
    fn deref(&self) -> &Self::Target {
        &self.0.frontiers
    }
}

impl StateCustody {
    /// Register an actual newly materialized root whose payload occurrence has
    /// already been admitted by its constructing owner. The caller keeps the
    /// semantic State and this receipt together until the returned capsule is
    /// moved into their final owning holder, including on every failure path.
    pub fn from_parts_retained(
        state: &State,
        frontiers: StateFrontiers,
        registry: &bank::CustodyRegistry,
        retained: &custody::SharedDebit,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        assert_eq!(
            retained.usage(),
            state
                .root_usage()?
                .checked_add(custody::SharedDebit::administration_usage())?,
            "a moved root receipt represents exactly its materialized occurrence"
        );
        let result = (|| {
            let root = registry.adopt_object_retained(state.identity, retained, &mut admit)?;
            Self::attach_frontiers(root, frontiers, registry, &mut admit)
        })();
        registry.reconcile_releases();
        result
    }

    fn attach_frontiers(
        root: bank::ObjectCustody,
        frontiers: StateFrontiers,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        if let Some(existing) = root.attachment::<CapturedFrontier>() {
            return Ok(Self(existing));
        }
        let debit = registry.ledger().retain(
            custody::Usage::new(size_of::<CapturedFrontier>(), 0),
            &mut admit,
        )?;
        let captured = Arc::new(CapturedFrontier { frontiers });
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: size_of::<CapturedFrontier>(),
            });
        }
        Ok(Self(root.attach(custody::Held::new(captured, debit))))
    }

    pub fn initial(
        state: &State,
        registry: &bank::CustodyRegistry,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        assert!(
            state.slots.is_empty() && state.ranges.is_empty(),
            "initial State has no records"
        );
        Self::from_parts(state, StateFrontiers::default(), registry, admit)
    }

    pub fn from_parts(
        state: &State,
        frontiers: StateFrontiers,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let result = (|| {
            let root = registry.adopt_object(state.identity, state.root_usage()?, &mut admit)?;
            Self::attach_frontiers(root, frontiers, registry, &mut admit)
        })();
        registry.reconcile_releases();
        result
    }

    /// Borrow input throughout admission. Only after success may a caller
    /// retain its semantic Arc alongside this capsule; descendants share it.
    pub fn adopt(
        state: &Arc<State>,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let result = (|| {
            step()?;
            let root = registry.adopt_object(state.identity, state.root_usage()?, &mut admit)?;
            if let Some(existing) = root.attachment::<CapturedFrontier>() {
                return Ok(Self(existing));
            }
            let auxiliary =
                super::auxiliary::Auxiliary::adopt(state, registry, &mut admit, &mut step)?;
            let slots = {
                let slot_admit = std::cell::RefCell::new(&mut admit);
                state.slots.adopt_controlled(
                    registry,
                    |slot| {
                        step()?;
                        payload::describe_slot(slot, registry, |next| {
                            (*slot_admit.borrow_mut())(next)
                        })
                    },
                    |next| (*slot_admit.borrow_mut())(next),
                )?
            };
            let ranges = state.ranges.adopt_controlled(
                registry,
                |_| {
                    step()?;
                    Ok(custody::Usage::new(
                        size_of::<crate::range::Slot<NodeId>>(),
                        0,
                    ))
                },
                &mut admit,
            )?;
            let inbound = state
                .inbound
                .adopt_custody(registry, &mut admit, &mut step)?;
            let controls = state
                .controls
                .adopt_custody(registry, &mut admit, &mut step)?;
            let slot_index = state
                .slot_index
                .adopt_custody(registry, &mut admit, &mut step)?;
            let families = state
                .family_index
                .adopt_custody(registry, &mut admit, &mut step)?;
            let role_facts = state
                .role_facts
                .adopt_custody(registry, &mut admit, &mut step)?;
            let containers = state
                .container_index
                .adopt_custody(registry, &mut admit, &mut step)?;
            let range_dependencies = state
                .range_dependencies
                .adopt_custody(registry, &mut admit, &mut step)?;
            let transitions = state
                .view_transitions
                .adopt_custody(registry, &mut admit, &mut step)?;
            let signal_targets = state
                .signal_targets
                .adopt_custody(registry, &mut admit, &mut step)?;
            let native_targets = state
                .native_targets
                .adopt_custody(registry, &mut admit, &mut step)?;
            let details_targets = state
                .details_targets
                .adopt_custody(registry, &mut admit, &mut step)?;
            Self::from_parts(
                state,
                StateFrontiers {
                    auxiliary,
                    slots,
                    ranges,
                    inbound,
                    controls,
                    slot_index,
                    families,
                    role_facts,
                    containers,
                    range_dependencies,
                    transitions,
                    signal_targets,
                    native_targets,
                    details_targets,
                },
                registry,
                &mut admit,
            )
        })();
        registry.reconcile_releases();
        result
    }
}

#[cfg(test)]
impl Dom {
    /// Seed an unreachable counter distance in an already public, empty slot.
    /// Creation and exhaustion still run through the real public allocator and
    /// retirement operations. No live identity or reverse reference is forged.
    pub(in crate::store) fn scale_seed_vacant_generation(&mut self, slot: usize, generation: u64) {
        assert!(generation > 0 && generation < u64::MAX);
        assert!(self.source_operation.is_none());
        let current = self
            .state
            .slots
            .get(slot)
            .expect("publicly allocated vacancy");
        assert!(current.node.is_none() && !current.retired);
        let replacement = Slot {
            generation,
            retired: false,
            node: None,
        };
        let registry = self.custody_registry.clone();
        let record = bank::ControlledRecord::new(
            replacement,
            custody::Usage::new(size_of::<Slot>(), 0),
            &registry,
            |_| Ok(()),
        )
        .unwrap();
        let (slots, slots_custody) = self
            .state
            .slots
            .set_controlled(
                &self.state_custody.slots,
                slot,
                record,
                true,
                bank::Totals {
                    bytes: size_of::<Slot>(),
                    units: 0,
                    occurrences: 1,
                },
                &registry,
                |_| Ok(()),
            )
            .unwrap();
        let mut frontiers = (*self.state_custody).clone();
        frontiers.slots = slots_custody;
        let state = Arc::get_mut(&mut self.state)
            .expect("counter seeding precedes retained scale snapshots");
        state.identity = custody::MaterializationId::fresh().unwrap();
        state.slots = slots;
        self.state_custody =
            StateCustody::from_parts(state, frontiers, &registry, |_| Ok(())).unwrap();
        registry.reconcile_releases();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(unrelated: usize) -> (Dom, NodeId) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                unrelated + 4,
                1_000_000,
                unrelated + 8,
                0,
                0,
                32,
                10_000_000,
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let subject = batch.create_element(&document, "subject");
        batch.set_attribute(&subject, "data-local", "before");
        for _ in 0..unrelated {
            batch.create_comment(&document, "unrelated");
        }
        let commit = dom.apply(batch, &cancel).unwrap();
        let subject = commit.created(&subject).unwrap().unwrap();
        (dom, subject)
    }

    #[test]
    fn capturing_the_current_state_reuses_its_exact_live_frontier() {
        for unrelated in [0, 4096] {
            let (dom, _) = fixture(unrelated);
            let registry = &dom.custody_registry;
            registry.reconcile_releases();
            let before = registry.total();
            let mut steps = 0;
            let captured = StateCustody::adopt(
                &dom.state,
                registry,
                |_| Ok(()),
                || {
                    steps += 1;
                    Ok(())
                },
            )
            .unwrap();
            assert!(captured.0.shares_target(&dom.state_custody.0));
            assert_eq!(steps, 1);
            assert_eq!(registry.total(), before);
            drop(captured);
            registry.reconcile_releases();
            assert_eq!(registry.total(), before);
        }
    }

    #[test]
    fn historical_adoption_visits_only_the_missing_changed_frontier() {
        for unrelated in [32, 4096] {
            let (mut dom, subject) = fixture(unrelated);
            let historical = dom.snapshot();
            let mut batch = DomBatch::new();
            batch.set_attribute(subject, "data-local", "after");
            dom.apply(batch, &AtomicBool::new(false)).unwrap();
            let mut steps = 0;
            let captured = StateCustody::adopt(
                &historical.state,
                &dom.custody_registry,
                |_| Ok(()),
                || {
                    steps += 1;
                    Ok(())
                },
            )
            .unwrap();
            assert!(
                steps <= 16,
                "unrelated={unrelated}, reached records={steps}"
            );
            assert_eq!(
                historical
                    .node(subject)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .attribute(&"data-local".into())
                    .unwrap(),
                &DocumentString::from("before")
            );
            assert_eq!(
                dom.snapshot()
                    .node(subject)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .attribute(&"data-local".into())
                    .unwrap(),
                &DocumentString::from("after")
            );
            drop(captured);
        }
    }

    #[test]
    fn an_external_snapshot_never_retains_its_former_owners_custody() {
        let (dom, subject) = fixture(128);
        let snapshot = dom.snapshot();
        let ledger = dom.custody_registry.ledger().clone();
        assert!(ledger.total().bytes() > 0);
        drop(dom);
        assert_eq!(ledger.total(), custody::Usage::ZERO);
        assert!(snapshot.node(subject).unwrap().as_element().is_some());
    }

    #[test]
    fn refused_historical_capture_restores_the_exact_controlled_union() {
        let (mut dom, subject) = fixture(128);
        let historical = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_attribute(subject, "data-local", "after");
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
        dom.custody_registry.reconcile_releases();
        let before = dom.custody_registry.total();
        let result = StateCustody::adopt(
            &historical.state,
            &dom.custody_registry,
            |usage| {
                admit(
                    usage.bytes(),
                    before.bytes(),
                    DomResource::SourceRetainedBytes,
                )
            },
            || Ok(()),
        );
        assert!(matches!(
            result,
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })
        ));
        dom.custody_registry.reconcile_releases();
        assert_eq!(dom.custody_registry.total(), before);
        let mut steps = 0;
        StateCustody::adopt(
            &historical.state,
            &dom.custody_registry,
            |_| Ok(()),
            || {
                steps += 1;
                Ok(())
            },
        )
        .unwrap();
        assert!(steps <= 16);
    }
}
