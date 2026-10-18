//! Exact active Document membership for the current initial-entry profile.
//!
//! Initial construction adds one navigable/entry pair. Active/current entry
//! identities never change in this profile; successful Document cleanup removes
//! its complete selected group. Ordinal keys retain the original first match.
use super::*;
use transaction::auxiliary::index::{self, Generations, Producer, U64Bank};

pub(in crate::store) struct Member {
    pub origin: custody::MaterializationId,
    pub navigable: usize,
    pub entry: usize,
    _retained: Retained,
}

pub(in crate::store) struct Members {
    pub origin: custody::MaterializationId,
    pub rows: bank::Bank<Member>,
    /// Pages and member bodies from this atomic call that still survive here.
    /// A different call receives a fresh origin, never used in an older group.
    owned_rows: custody::Usage,
    _retained: Retained,
}

#[derive(Clone, Default)]
pub(in crate::store) struct Index {
    owner: Option<DomOwnerId>,
    pub slots: bank::Bank<Generations<Members>>,
}
impl Index {
    pub fn get(&self, document: DocumentId) -> Option<&Members> {
        let node = document.node();
        if self.owner.is_some_and(|owner| owner != node.owner) {
            return None;
        }
        self.slots.get(node.slot)?.values.get(node.generation)
    }
    pub fn usage(&self) -> Result<custody::Usage, DomError> {
        index::bank_usage(&self.slots)
    }
    pub fn add(
        &mut self,
        document: DocumentId,
        navigable: usize,
        entry: usize,
        producer: &Producer<'_>,
    ) -> Result<custody::Usage, DomHostedError> {
        let old = self.get(document);
        if old.is_some_and(|members| members.rows.get(navigable).is_some()) {
            return Err(DomHostedError::WrongPhase);
        }
        let mut rows = old.map_or_else(bank::Bank::default, |members| members.rows.clone());
        let old_owned = old
            .filter(|members| members.origin == producer.origin)
            .map_or(custody::Usage::ZERO, |members| members.owned_rows);
        let replaced = custody::Usage::new(
            rows.origin_path_index_charge(navigable, producer.origin)?
                .bytes,
            0,
        );
        let member = producer.record(|retained| Member {
            origin: producer.origin,
            navigable,
            entry,
            _retained: retained,
        })?;
        let allocated = rows.set_hosted_admitted(
            navigable,
            member,
            false,
            bank::Totals {
                bytes: size_of::<Member>(),
                units: 0,
                occurrences: 0,
            },
            producer.origin,
            |charge| producer.retain(custody::Usage::new(charge.bytes, 0)),
        )?;
        let owned_rows = old_owned
            .checked_add(custody::Usage::new(allocated.bytes, 0))?
            .checked_add(custody::Usage::new(size_of::<Member>(), 0))?
            .checked_sub(replaced)?;
        let members = producer.record(|retained| Members {
            origin: producer.origin,
            rows,
            owned_rows,
            _retained: retained,
        })?;
        self.replace(document, members, producer)?
            .checked_add(replaced)
            .map_err(Into::into)
    }
    pub fn clear(
        &mut self,
        document: DocumentId,
        producer: &Producer<'_>,
    ) -> Result<custody::Usage, DomHostedError> {
        let Some(old) = self.get(document) else {
            return Ok(custody::Usage::ZERO);
        };
        if old.rows.is_empty() {
            return Ok(custody::Usage::ZERO);
        }
        let discarded = if old.origin == producer.origin {
            old.owned_rows
        } else {
            custody::Usage::ZERO
        };
        let members = producer.record(|retained| Members {
            origin: producer.origin,
            rows: bank::Bank::default(),
            owned_rows: custody::Usage::ZERO,
            _retained: retained,
        })?;
        self.replace(document, members, producer)?
            .checked_add(discarded)
            .map_err(Into::into)
    }
    fn replace(
        &mut self,
        document: DocumentId,
        members: bank::Record<Members>,
        producer: &Producer<'_>,
    ) -> Result<custody::Usage, DomHostedError> {
        let node = document.node();
        if self.owner.is_some_and(|owner| owner != node.owner) {
            return Err(DomError::ForeignOwner.into());
        }
        let old_slot = self.slots.get(node.slot);
        let mut discarded = custody::Usage::new(
            self.slots
                .origin_path_index_charge(node.slot, producer.origin)?
                .bytes,
            0,
        );
        if let Some(slot) = old_slot {
            if slot.origin == producer.origin {
                discarded = discarded
                    .checked_add(custody::Usage::new(size_of::<Generations<Members>>(), 0))?;
            }
            discarded =
                discarded.checked_add(slot.values.path_usage(node.generation, producer.origin)?)?;
            if slot
                .values
                .get(node.generation)
                .is_some_and(|old| old.origin == producer.origin)
            {
                discarded = discarded.checked_add(custody::Usage::new(size_of::<Members>(), 0))?;
            }
        }
        let mut values = old_slot.map_or_else(U64Bank::default, |slot| slot.values.clone());
        let usage = custody::Usage::new(size_of::<Members>(), 0)
            .checked_add(index::bank_usage(&members.value().rows)?)?;
        values.insert(node.generation, members, usage, producer)?;
        let usage = custody::Usage::new(size_of::<Generations<Members>>(), 0)
            .checked_add(values.usage()?)?;
        let slot = Generations::form(values, producer)?;
        let mut slots = self.slots.clone();
        producer.set(&mut slots, node.slot, slot, usage)?;
        self.slots = slots;
        self.owner = Some(node.owner);
        Ok(discarded)
    }
    #[cfg(test)]
    pub fn inspect_materializations(
        &self,
    ) -> Result<Vec<(custody::MaterializationId, custody::Usage)>, DomError> {
        let slots = self.slots.inspect_graph(|_| {
            Ok(bank::Totals {
                bytes: size_of::<Generations<Members>>(),
                units: 0,
                occurrences: 0,
            })
        })?;
        let mut output = slots.materializations().collect::<Vec<_>>();
        for (_, slot) in self.slots.iter() {
            output.extend(
                slot.values.inspect_materializations(&|_| {
                    Ok(custody::Usage::new(size_of::<Members>(), 0))
                })?,
            );
            slot.values.visit(&mut |members| {
                let rows = members.rows.inspect_graph(|_| {
                    Ok(bank::Totals {
                        bytes: size_of::<Member>(),
                        units: 0,
                        occurrences: 0,
                    })
                })?;
                output.extend(rows.materializations());
                Ok(())
            })?;
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn raw_owned(members: &Members) -> custody::Usage {
        let pages = members.rows.inspect_origin_pages().unwrap();
        let mut usage = pages
            .into_iter()
            .filter(|(_, origin, _)| *origin == Some(members.origin))
            .fold(custody::Usage::ZERO, |sum, (_, _, own)| {
                sum.checked_add(own).unwrap()
            });
        for (_, row) in members.rows.iter() {
            if row.origin == members.origin {
                usage = usage
                    .checked_add(custody::Usage::new(size_of::<Member>(), 0))
                    .unwrap();
            }
        }
        usage
    }
    fn raw_rows(members: Option<&Members>) -> Vec<(custody::MaterializationId, custody::Usage)> {
        members.map_or_else(Vec::new, |members| {
            members
                .rows
                .inspect_graph(|_| {
                    Ok(bank::Totals {
                        bytes: size_of::<Member>(),
                        units: 0,
                        occurrences: 0,
                    })
                })
                .unwrap()
                .materializations()
                .collect()
        })
    }
    #[test]
    fn reused_interleaved_contexts_keep_exact_new_call_member_receipts() {
        let cancel = AtomicBool::new(false);
        let mut fixture = super::super::tests::Fixture::new();
        let (creation, _, work) = fixture.issue(0);
        let a = Context::new(&fixture.source, &work, &cancel, 0);
        let b = Context::new(&fixture.source, &work, &cancel, 0);
        let mut index = Index::default();
        let mut origins = BTreeSet::new();
        for (context, ordinal) in [(&a, 0), (&b, 32), (&a, 64), (&a, 96)] {
            let before = raw_rows(index.get(creation.document))
                .into_iter()
                .map(|(id, _)| id)
                .collect::<BTreeSet<_>>();
            {
                let _call = context.call();
                let origin = context.origin().unwrap();
                let producer = Producer {
                    source: context.source,
                    work: context.work,
                    formation: None,
                    cancel: context.cancel,
                    origin,
                    scratch: &context.scratch,
                };
                index
                    .add(creation.document, ordinal, ordinal, &producer)
                    .unwrap();
                let group = index.get(creation.document).unwrap();
                assert_eq!(group.owned_rows, raw_owned(group));
                let actual_new = raw_rows(Some(group))
                    .into_iter()
                    .filter(|(id, _)| !before.contains(id))
                    .fold(custody::Usage::ZERO, |sum, (_, usage)| {
                        sum.checked_add(usage).unwrap()
                    });
                assert_eq!(group.owned_rows, actual_new);
                assert!(
                    origins.insert(origin),
                    "each synchronous call receives a fresh origin"
                );
            }
            assert_eq!(context.scratch.get(), 0);
        }
        assert!(a.origin.get().is_none());
        assert!(b.origin.get().is_none());
        assert_eq!(
            index
                .get(creation.document)
                .unwrap()
                .rows
                .iter()
                .map(|(_, row)| row.entry)
                .collect::<Vec<_>>(),
            vec![0, 32, 64, 96]
        );
    }

    #[test]
    fn member_receipt_cache_tracks_same_call_path_replacement_and_refusal_restores_context() {
        let cancel = AtomicBool::new(false);
        let mut fixture = super::super::tests::Fixture::new();
        let (creation, _, work) = fixture.issue(0);
        let context = Context::new(&fixture.source, &work, &cancel, 0);
        let mut index = Index::default();
        {
            let _call = context.call();
            let origin = context.origin().unwrap();
            let producer = Producer {
                source: context.source,
                work: context.work,
                formation: None,
                cancel: context.cancel,
                origin,
                scratch: &context.scratch,
            };
            for ordinal in [0, 1, 32, 33, 1024] {
                index
                    .add(creation.document, ordinal, ordinal, &producer)
                    .unwrap();
                let group = index.get(creation.document).unwrap();
                assert_eq!(group.owned_rows, raw_owned(group));
                let all = raw_rows(Some(group))
                    .into_iter()
                    .fold(custody::Usage::ZERO, |sum, (_, own)| {
                        sum.checked_add(own).unwrap()
                    });
                assert_eq!(group.owned_rows, all);
            }
        }
        let bytes = fixture.source.retained_storage_bytes();
        let before = index.inspect_materializations().unwrap();
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        {
            let _call = context.call();
            let producer = Producer {
                source: context.source,
                work: context.work,
                formation: None,
                cancel: context.cancel,
                origin: context.origin().unwrap(),
                scratch: &context.scratch,
            };
            assert!(matches!(
                index.add(creation.document, 2048, 2048, &producer),
                Err(DomHostedError::Dom(DomError::Cancelled))
            ));
        }
        assert!(context.origin.get().is_none());
        assert_eq!(context.scratch.get(), 0);
        assert_eq!(fixture.source.retained_storage_bytes(), bytes);
        assert_eq!(index.inspect_materializations().unwrap(), before);
        cancel.store(false, std::sync::atomic::Ordering::Relaxed);
        {
            let _call = context.call();
            let producer = Producer {
                source: context.source,
                work: context.work,
                formation: None,
                cancel: context.cancel,
                origin: context.origin().unwrap(),
                scratch: &context.scratch,
            };
            index.clear(creation.document, &producer).unwrap();
        }
        assert!(index.get(creation.document).unwrap().rows.is_empty());
        assert!(context.origin.get().is_none());
    }
    #[test]
    fn genuine_document_group_rejects_foreign_owner_and_every_wrong_generation_bit() {
        let cancel = AtomicBool::new(false);
        let mut fixture = super::super::tests::Fixture::new();
        let (missing, _, setup_work) = fixture.issue(2);
        drop(setup_work);
        let (creation, _, work) = fixture.issue(0);
        let context = Context::new(&fixture.source, &work, &cancel, 0);
        let mut index = Index::default();
        {
            let _call = context.call();
            let producer = Producer {
                source: context.source,
                work: context.work,
                formation: None,
                cancel: context.cancel,
                origin: context.origin().unwrap(),
                scratch: &context.scratch,
            };
            index.add(creation.document, 0, 0, &producer).unwrap();
            let before = index.inspect_materializations().unwrap();
            let bytes = fixture.source.retained_storage_bytes();
            assert_eq!(
                index.add(creation.document, 0, 1, &producer),
                Err(DomHostedError::WrongPhase)
            );
            assert_eq!(index.inspect_materializations().unwrap(), before);
            assert_eq!(fixture.source.retained_storage_bytes(), bytes);
        }
        assert_eq!(
            index
                .get(creation.document)
                .unwrap()
                .rows
                .get(0)
                .unwrap()
                .entry,
            0
        );
        assert!(index.get(missing.document).is_none());
        let mut foreign_fixture = super::super::tests::Fixture::new();
        let (foreign, _, _foreign_work) = foreign_fixture.issue(0);
        assert_eq!(foreign.document.node().slot, creation.document.node().slot);
        assert_ne!(
            foreign.document.node().owner,
            creation.document.node().owner
        );
        assert!(index.get(foreign.document).is_none());
        // These are explicit stale/invalid key negatives, never native Document
        // authority. Each single bit, including all high bits on 32-bit hosts,
        // must distinguish the genuine positive Document's generation.
        let actual = creation.document.node();
        for bit in 0..64 {
            let wrong = DocumentId(NodeId {
                generation: actual.generation ^ (1u64 << bit),
                ..actual
            });
            assert!(
                index.get(wrong).is_none(),
                "wrong generation bit {bit} matched"
            );
        }
        if actual.generation != u64::MAX {
            let terminal = DocumentId(NodeId {
                generation: u64::MAX,
                ..actual
            });
            assert!(index.get(terminal).is_none());
        }
    }

    #[test]
    fn empty_missing_foreign_cleared_and_present_document_queries_keep_step_and_cancel_behavior() {
        fn query(
            graph: &Graph,
            document: DocumentId,
            fail: Option<usize>,
        ) -> (Result<Option<DomHostedNavigableId>, DomHostedError>, usize) {
            let mut calls = 0;
            let result = graph.active_navigable_for_document_with(document, || {
                calls += 1;
                if fail == Some(calls) {
                    Err(DomHostedError::Dom(DomError::Cancelled))
                } else {
                    Ok(())
                }
            });
            (result, calls)
        }
        let cancel = AtomicBool::new(false);
        let mut fixture = super::super::tests::Fixture::new();
        let (missing, _, setup_work) = fixture.issue(2);
        drop(setup_work);
        let (creation, _, work) = fixture.issue(0);
        let context = Context::new(&fixture.source, &work, &cancel, 0);
        let empty = {
            let _call = context.call();
            Graph::form(None, None, None, &context).unwrap()
        };
        assert_eq!(query(&empty, creation.document, Some(1)), (Ok(None), 0));
        let graph = Graph::initial(None, fixture.input(creation, None), &context).unwrap();
        assert_eq!(
            query(&graph, creation.document, None),
            (Ok(Some(creation.navigable)), 2)
        );
        assert_eq!(
            query(&graph, creation.document, Some(2)),
            (Err(DomHostedError::Dom(DomError::Cancelled)), 2)
        );
        let mut foreign_fixture = super::super::tests::Fixture::new();
        let (foreign, _, _foreign_work) = foreign_fixture.issue(0);
        for missing in [missing.document, foreign.document] {
            assert_eq!(query(&graph, missing, None), (Ok(None), 1));
            assert_eq!(
                query(&graph, missing, Some(1)),
                (Err(DomHostedError::Dom(DomError::Cancelled)), 1)
            );
            let unchanged = graph.clear_document(missing, &context).unwrap();
            assert_eq!(*unchanged, *graph);
            assert_eq!(
                unchanged.entry(creation.entry).unwrap().document,
                Some(creation.document)
            );
        }
        let cleared = graph.clear_document(creation.document, &context).unwrap();
        assert_eq!(query(&cleared, creation.document, None), (Ok(None), 1));
        assert_eq!(
            query(&cleared, creation.document, Some(1)),
            (Err(DomHostedError::Dom(DomError::Cancelled)), 1)
        );
        assert_eq!(cleared.entry(creation.entry).unwrap().document, None);
        assert!(cleared.navigable(creation.navigable).unwrap().retired);
        assert_eq!(
            graph.entry(creation.entry).unwrap().document,
            Some(creation.document)
        );
        assert!(!graph.navigable(creation.navigable).unwrap().retired);
    }
}
