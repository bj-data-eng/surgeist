//! Candidate construction follows the transaction's exact sparse record set.
use super::super::*;
use super::{NodeReceipts, SequenceView};

#[cfg(test)]
#[derive(Default)]
struct CopyObservation {
    bytes: usize,
    relation: bool,
}
#[cfg(test)]
impl CopyObservation {
    fn relation() -> Self {
        Self {
            relation: true,
            ..Self::default()
        }
    }
    fn pushed<T>(&mut self) {
        self.bytes += size_of::<T>();
    }
}
#[cfg(test)]
impl Drop for CopyObservation {
    fn drop(&mut self) {
        if scale_metrics::ENABLED && self.bytes != 0 {
            scale_metrics::record(scale_metrics::Event::SequenceMaterialized { bytes: self.bytes });
            if self.relation {
                scale_metrics::record(scale_metrics::Event::RelationSequenceMaterialized {
                    bytes: self.bytes,
                });
            }
        }
    }
}

fn add_sequence<C, S>(
    usage: custody::Usage,
    view: SequenceView<'_, C, S>,
    width: usize,
    header: usize,
) -> Result<custody::Usage, DomError>
where
    C: Copy,
    S: Copy,
{
    let capacity = match view {
        SequenceView::Canonical(values, _) | SequenceView::AliasedCanonical(values, _, _, _) => {
            values.capacity()
        }
        SequenceView::Staged(values) | SequenceView::AliasedStaged(values, _, _) => values.len(),
    };
    usage.checked_add(custody::Usage::new(
        header
            .checked_add(
                capacity
                    .checked_mul(width)
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .ok_or(DomError::RepresentationExhausted)?,
        0,
    ))
}
fn keys(
    view: SequenceView<'_, NodeId, Key>,
    map: &impl Fn(Key) -> NodeId,
    debit: &mut custody::Debit,
    draft: &Draft,
    cancel: &AtomicBool,
) -> Result<SharedVec<NodeId>, DomError> {
    if let SequenceView::Canonical(values, _) | SequenceView::AliasedCanonical(values, _, _, _) =
        view
    {
        return Ok(values.clone());
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(view.len())
        .map_err(|_| DomError::AllocationFailed)?;
    #[cfg(test)]
    if scale_metrics::ENABLED && values.capacity() != 0 {
        scale_metrics::record(scale_metrics::Event::StructuralAllocation {
            bytes: values.capacity() * size_of::<NodeId>(),
        });
    }
    let extra = values
        .capacity()
        .checked_sub(view.len())
        .and_then(|n| n.checked_mul(size_of::<NodeId>()))
        .ok_or(DomError::RepresentationExhausted)?;
    if extra != 0 {
        debit.grow(custody::Usage::new(extra, 0), |usage| {
            draft.admit_custody_usage(usage, cancel)
        })?;
    }
    #[cfg(test)]
    let mut copied = CopyObservation::relation();
    for value in view.iter() {
        draft.source_step(cancel)?;
        values.push(map(value));
        #[cfg(test)]
        copied.pushed::<NodeId>();
    }
    let values = SharedVec::from(values);
    #[cfg(test)]
    if scale_metrics::ENABLED {
        scale_metrics::record(scale_metrics::Event::StructuralAllocation {
            bytes: size_of::<Vec<NodeId>>(),
        });
    }
    Ok(values)
}
fn events(
    view: SequenceView<'_, crate::DomFullscreenEvent, crate::DomFullscreenEvent>,
    debit: &mut custody::Debit,
    draft: &Draft,
    cancel: &AtomicBool,
) -> Result<SharedVec<crate::DomFullscreenEvent>, DomError> {
    if let SequenceView::Canonical(values, _) | SequenceView::AliasedCanonical(values, _, _, _) =
        view
    {
        return Ok(values.clone());
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(view.len())
        .map_err(|_| DomError::AllocationFailed)?;
    #[cfg(test)]
    if scale_metrics::ENABLED && values.capacity() != 0 {
        scale_metrics::record(scale_metrics::Event::StructuralAllocation {
            bytes: values.capacity() * size_of::<crate::DomFullscreenEvent>(),
        });
    }
    let extra = values
        .capacity()
        .checked_sub(view.len())
        .and_then(|n| n.checked_mul(size_of::<crate::DomFullscreenEvent>()))
        .ok_or(DomError::RepresentationExhausted)?;
    if extra != 0 {
        debit.grow(custody::Usage::new(extra, 0), |usage| {
            draft.admit_custody_usage(usage, cancel)
        })?;
    }
    #[cfg(test)]
    let mut copied = CopyObservation::relation();
    for value in view.iter() {
        draft.source_step(cancel)?;
        values.push(value);
        #[cfg(test)]
        copied.pushed::<crate::DomFullscreenEvent>();
    }
    let values = SharedVec::from(values);
    #[cfg(test)]
    if scale_metrics::ENABLED {
        scale_metrics::record(scale_metrics::Event::StructuralAllocation {
            bytes: size_of::<Vec<crate::DomFullscreenEvent>>(),
        });
    }
    Ok(values)
}
impl Draft {
    fn form_node(
        &self,
        key: Key,
        map: &impl Fn(Key) -> NodeId,
        cancel: &AtomicBool,
    ) -> Result<custody::Held<Slot>, DomError> {
        let source = self.node(key)?;
        if source
            .base
            .and_then(|v| v.frozen)
            .is_some_and(|v| v.response == crate::base::Response::Pending)
        {
            return Err(DomError::BaseFacts(
                crate::base::DomBaseFactsError::Missing(
                    surgeist_html_semantics::HtmlBaseMissing::ResponseDomain,
                ),
            ));
        }
        let private = self.records_for(key);
        let fields = private.fields.value();
        let mut field_usage = private
            .fields
            .debit()
            .usage()
            .checked_sub(custody::Usage::new(size_of::<super::NodeFields>(), 0))?;
        let mut names_receipt = custody::Usage::ZERO;
        let mut names_units = 0usize;
        if let Some(names) = &fields.form_past_names {
            names_units = names.logical_units();
            let old = names.usage();
            names_receipt = match &names.backing {
                super::names::NamesBacking::Canonical(_) => old,
                super::names::NamesBacking::Staged(values) => {
                    let old_bank = values
                        .entries
                        .capacity()
                        .checked_mul(size_of::<crate::form::FormPastName<Key>>())
                        .ok_or(DomError::RepresentationExhausted)?;
                    let new_bank = values
                        .entries
                        .len()
                        .checked_mul(size_of::<crate::form::FormPastName<NodeId>>())
                        .ok_or(DomError::RepresentationExhausted)?;
                    old.checked_replace(
                        custody::Usage::new(old_bank, 0),
                        custody::Usage::new(new_bank, 0),
                    )?
                }
            };
            field_usage = field_usage.checked_replace(old, names_receipt)?;
        }
        let mut payload = field_usage;
        payload = add_sequence(
            payload,
            source.children,
            size_of::<NodeId>(),
            size_of::<Vec<NodeId>>(),
        )?;
        payload = add_sequence(
            payload,
            source.manual_nodes,
            size_of::<NodeId>(),
            size_of::<Vec<NodeId>>(),
        )?;
        payload = add_sequence(
            payload,
            source.assigned_nodes,
            size_of::<NodeId>(),
            size_of::<Vec<NodeId>>(),
        )?;
        payload = add_sequence(
            payload,
            source.fullscreen_events,
            size_of::<crate::DomFullscreenEvent>(),
            size_of::<Vec<crate::DomFullscreenEvent>>(),
        )?;
        if let Some(base) = &source.base_document {
            payload = add_sequence(
                payload,
                base.render_blocking,
                size_of::<NodeId>(),
                size_of::<Vec<NodeId>>(),
            )?;
        }
        if let Some(super::AutofocusRead::Known { candidates, .. }) = source.autofocus_document {
            payload = add_sequence(
                payload,
                candidates,
                size_of::<NodeId>(),
                size_of::<Vec<NodeId>>(),
            )?;
        }
        if let Some(super::InteractiveRead::Known {
            open_dialogs,
            top_layer,
            pending_top_layer_removals,
        }) = source.interactive_document
        {
            for seq in [open_dialogs, top_layer, pending_top_layer_removals] {
                payload =
                    add_sequence(payload, seq, size_of::<NodeId>(), size_of::<Vec<NodeId>>())?;
            }
        }
        let total = payload.checked_add(custody::Usage::new(size_of::<Slot>(), 0))?;
        let mut debit = self
            .custody_ledger()
            .retain(total, |usage| self.admit_custody_usage(usage, cancel))?;
        let form_past_names = fields
            .form_past_names
            .as_ref()
            .map(|names| match &names.backing {
                super::names::NamesBacking::Canonical(values) => Ok(values.clone()),
                super::names::NamesBacking::Staged(values) => {
                    let mut out = Vec::new();
                    out.try_reserve_exact(values.entries.len())
                        .map_err(|_| DomError::AllocationFailed)?;
                    #[cfg(test)]
                    if scale_metrics::ENABLED && out.capacity() != 0 {
                        scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                            bytes: out.capacity() * size_of::<crate::form::FormPastName<NodeId>>(),
                        });
                    }
                    let extra = (out.capacity() - values.entries.len())
                        .checked_mul(size_of::<crate::form::FormPastName<NodeId>>())
                        .ok_or(DomError::RepresentationExhausted)?;
                    if extra != 0 {
                        debit.grow(custody::Usage::new(extra, 0), |usage| {
                            self.admit_custody_usage(usage, cancel)
                        })?;
                        names_receipt = names_receipt.checked_add(custody::Usage::new(extra, 0))?;
                        field_usage = field_usage.checked_add(custody::Usage::new(extra, 0))?;
                    }
                    #[cfg(test)]
                    let mut copied = CopyObservation::default();
                    for entry in values.entries.iter() {
                        self.source_step(cancel)?;
                        out.push(crate::form::FormPastName {
                            name: entry.name.clone(),
                            target: map(entry.target),
                        });
                        #[cfg(test)]
                        copied.pushed::<crate::form::FormPastName<NodeId>>();
                    }
                    let entries = Arc::new(out);
                    #[cfg(test)]
                    if scale_metrics::ENABLED {
                        scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                            bytes: size_of::<Vec<crate::form::FormPastName<NodeId>>>(),
                        });
                    }
                    Ok(crate::form::FormPastNames { entries })
                }
            })
            .transpose()?;
        let node = Node {
            receipts: NodeReceipts {
                fields: field_usage
                    .checked_add(custody::Usage::new(size_of::<super::NodeFields>(), 0))?,
                payload,
                names: names_receipt,
                name_units: names_units,
            },
            active_range: source.active_range,
            relevant_global: source.relevant_global,
            document: DocumentId(map(source.document)),
            kind: source.kind.clone(),
            parent: source.parent.map(map),
            children: keys(source.children, map, &mut debit, self, cancel)?,
            shadow_root: source.shadow_root.map(map),
            host: source.host.map(map),
            template_contents: source.template_contents.map(map),
            template_patch: source.template_patch.map(|v| v.map(map)),
            template_document: source.template_document.map(|v| v.map(map)),
            manual_nodes: keys(source.manual_nodes, map, &mut debit, self, cancel)?,
            assigned_nodes: keys(source.assigned_nodes, map, &mut debit, self, cancel)?,
            manual_slot: source.manual_slot.map(map),
            assigned_slot: source.assigned_slot.map(map),
            reflected_targets: source.reflected_targets.map(|v| v.map(map)),
            control_owner: source.control_owner.map(map),
            option_select: source.option_select.map(map),
            form_past_names,
            script: source.script.map(|v| v.map(map)),
            style_parser_document: source.style_parser_document.map(map),
            base: source.base.map(|v| crate::base::BaseState {
                frozen: v.frozen.map(|f| crate::base::Frozen {
                    value: f.value.clone(),
                    document: map(f.document),
                    revision: f.revision.clone(),
                    response: f.response,
                }),
            }),
            base_document: source
                .base_document
                .map(|v| {
                    Ok::<_, DomError>(canonical::BaseDocument {
                        facts: v.facts.cloned(),
                        first: v.first.map(map),
                        render_blocking: keys(v.render_blocking, map, &mut debit, self, cancel)?,
                    })
                })
                .transpose()?,
            link: source.link.map(|v| crate::base::LinkState {
                flags: v.flags,
                parser_document: v.parser_document.map(map),
                sheet: v.sheet.cloned(),
            }),
            image: source.image.map(|v| crate::image::ImageState {
                native: v.native.clone(),
                dimension_source: map(v.dimension_source),
                parser_inserted: v.parser_inserted,
                high_water: v.high_water,
                waiting: v.waiting,
            }),
            hyperlink: source.hyperlink.map(|v| crate::hyperlink::HyperlinkState {
                parsed: v.parsed.map(|p| crate::hyperlink::Parsed {
                    document: map(p.document),
                    value: p.value.clone(),
                }),
            }),
            encoding_url_document: source.encoding_url_document.cloned(),
            autofocus_document: source
                .autofocus_document
                .map(|v| match v {
                    super::AutofocusRead::Unavailable => Ok(canonical::Autofocus::Unavailable),
                    super::AutofocusRead::Known {
                        candidates,
                        processed,
                    } => Ok(canonical::Autofocus::Known {
                        candidates: keys(candidates, map, &mut debit, self, cancel)?,
                        processed,
                    }),
                })
                .transpose()?,
            interactive_document: source
                .interactive_document
                .map(|v| match v {
                    super::InteractiveRead::Unavailable => Ok(canonical::Interactive::Unavailable),
                    super::InteractiveRead::Known {
                        open_dialogs,
                        top_layer,
                        pending_top_layer_removals,
                    } => Ok(canonical::Interactive::Known {
                        open_dialogs: keys(open_dialogs, map, &mut debit, self, cancel)?,
                        top_layer: keys(top_layer, map, &mut debit, self, cancel)?,
                        pending_top_layer_removals: keys(
                            pending_top_layer_removals,
                            map,
                            &mut debit,
                            self,
                            cancel,
                        )?,
                    }),
                })
                .transpose()?,
            fullscreen_events: events(source.fullscreen_events, &mut debit, self, cancel)?,
        };
        let payload = debit
            .usage()
            .checked_sub(custody::Usage::new(size_of::<Slot>(), 0))?;
        let mut node = node;
        node.receipts.payload = payload;
        let id = map(key);
        Ok(custody::Held::new(
            Slot {
                generation: id.generation,
                retired: false,
                node: Some(node),
            },
            debit,
        ))
    }
}

struct NodePublication {
    slots: bank::Bank<Slot>,
    custody: bank::BankCustody<Slot>,
    touched: bank::Bank<bool>,
    _touched_custody: bank::BankCustody<bool>,
    ids: source::AllocationIds,
}
impl Draft {
    fn install_candidate_slot(
        &self,
        slots: &mut bank::Bank<Slot>,
        custody: &mut bank::BankCustody<Slot>,
        index: usize,
        mut held: custody::Held<Slot>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if slots.get(index) == Some(held.value()) {
            return Ok(());
        }
        let payload = source::storage::canonical_slot(held.value(), || self.source_step(cancel))?;
        let reusable = held.value().node.is_none() && !held.value().retired;
        let shared = held
            .debit_mut()
            .share(|usage| self.admit_custody_usage(usage, cancel))?;
        let (value, zero) = held.into_parts();
        drop(zero);
        let record = bank::ControlledRecord::from_parts(bank::Record::new(value)?, shared);
        let (next, nextcustody) = slots.set_controlled(
            custody,
            index,
            record,
            reusable,
            bank::Totals {
                bytes: payload.bytes,
                units: payload.units,
                occurrences: 1,
            },
            self.custody_registry(),
            |usage| self.admit_custody_usage(usage, cancel),
        )?;
        *slots = next;
        *custody = nextcustody;
        Ok(())
    }
    fn reserve_public_allocations(
        &self,
        cancel: &AtomicBool,
    ) -> Result<
        (
            bank::Bank<Slot>,
            bank::BankCustody<Slot>,
            source::AllocationIds,
        ),
        DomError,
    > {
        let mut slots = self.base.slots.clone();
        let mut custody = self.base_custody.slots.clone();
        let mut ids = self
            .phase_map
            .as_ref()
            .map(|p| p.ids.clone())
            .unwrap_or_default();
        for (index, record_index) in self.existing.iter() {
            self.source_step(cancel)?;
            if self.records[*record_index].is_none() {
                let old = &self.base.slots[index];
                let slot = Slot {
                    generation: old.generation.checked_add(1).unwrap_or(old.generation),
                    retired: old.retired || old.generation == u64::MAX,
                    node: None,
                };
                let debit = self
                    .custody_ledger()
                    .retain(custody::Usage::new(size_of::<Slot>(), 0), |usage| {
                        self.admit_custody_usage(usage, cancel)
                    })?;
                self.install_candidate_slot(
                    &mut slots,
                    &mut custody,
                    index,
                    custody::Held::new(slot, debit),
                    cancel,
                )?;
            }
        }
        for (created, _) in self.created_dirty.iter() {
            self.source_step(cancel)?;
            if self.created[created].is_none() {
                if let Some(id) = ids.get(created) {
                    let old = &self.base.slots[id.slot];
                    let slot = Slot {
                        generation: old.generation.checked_add(1).unwrap_or(old.generation),
                        retired: old.retired || old.generation == u64::MAX,
                        node: None,
                    };
                    let debit = self
                        .custody_ledger()
                        .retain(custody::Usage::new(size_of::<Slot>(), 0), |usage| {
                            self.admit_custody_usage(usage, cancel)
                        })?;
                    self.install_candidate_slot(
                        &mut slots,
                        &mut custody,
                        id.slot,
                        custody::Held::new(slot, debit),
                        cancel,
                    )?;
                }
                continue;
            }
            if self.is_private(Key::Created(created)) || ids.get(created).is_some() {
                continue;
            }
            let slot = slots.lowest_vacant().unwrap_or(slots.len());
            let generation = slots.get(slot).map_or(0, |entry| entry.generation);
            ids.assign(
                created,
                NodeId {
                    owner: self.store,
                    slot,
                    generation,
                },
                self.custody_registry(),
                |usage| self.admit_custody_usage(usage, cancel),
            )?;
            // Occupancy is reserved without cloning any Node. The candidate is
            // private until its selected record is formed below.
            let record = bank::ControlledRecord::new(
                Slot {
                    generation,
                    retired: false,
                    node: None,
                },
                custody::Usage::new(size_of::<Slot>(), 0),
                self.custody_registry(),
                |usage| self.admit_custody_usage(usage, cancel),
            )?;
            let (next, nextcustody) = slots.set_controlled(
                &custody,
                slot,
                record,
                false,
                bank::Totals {
                    bytes: size_of::<Slot>(),
                    units: 0,
                    occurrences: 1,
                },
                self.custody_registry(),
                |usage| self.admit_custody_usage(usage, cancel),
            )?;
            slots = next;
            custody = nextcustody;
        }
        Ok((slots, custody, ids))
    }
    pub(in crate::store) fn allocation_ids(
        &self,
        _dom: &Dom,
        _budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<source::AllocationIds, DomError> {
        self.reserve_public_allocations(cancel)
            .map(|(_, _, ids)| ids)
    }
    fn finish_node_slots(&self, cancel: &AtomicBool) -> Result<NodePublication, DomError> {
        let (mut slots, mut custody, ids) = self.reserve_public_allocations(cancel)?;
        let mut touched = bank::Bank::default();
        let mut touched_custody = bank::BankCustody::default();
        let map = |key| match key {
            Key::Existing(id) => id,
            Key::Created(index) => ids.get(index).expect("surviving public creation reference"),
        };
        let keys = self
            .existing
            .iter()
            .map(|(slot, _)| {
                Key::Existing(NodeId {
                    owner: self.store,
                    slot,
                    generation: self.base.slots[slot].generation,
                })
            })
            .chain(self.created_dirty.iter().filter_map(|(index, _)| {
                (!self.is_private(Key::Created(index)) && ids.get(index).is_some())
                    .then_some(Key::Created(index))
            }));
        for key in keys {
            self.source_step(cancel)?;
            let index = map(key).slot;
            if self.node(key).is_ok() {
                let node = self.form_node(key, &map, cancel)?;
                self.install_candidate_slot(&mut slots, &mut custody, index, node, cancel)?;
            }
            if touched.get(index).is_none() {
                let record = bank::ControlledRecord::new(
                    true,
                    custody::Usage::new(size_of::<bool>(), 0),
                    self.custody_registry(),
                    |usage| self.admit_custody_usage(usage, cancel),
                )?;
                let (next, nextcustody) = touched.set_controlled(
                    &touched_custody,
                    index,
                    record,
                    false,
                    bank::Totals {
                        bytes: size_of::<bool>(),
                        units: 0,
                        occurrences: 1,
                    },
                    self.custody_registry(),
                    |usage| self.admit_custody_usage(usage, cancel),
                )?;
                touched = next;
                touched_custody = nextcustody;
            }
        }
        Ok(NodePublication {
            slots,
            custody,
            touched,
            _touched_custody: touched_custody,
            ids,
        })
    }
}

impl State {
    pub(in crate::store) fn root_usage(&self) -> Result<custody::Usage, DomError> {
        Self::root_usage_for_capacities([
            self.signal_slots.capacity(),
            self.native_tasks.capacity(),
            self.details_intentions.capacity(),
        ])
    }
    fn root_usage_for_capacities(capacities: [usize; 3]) -> Result<custody::Usage, DomError> {
        let mut usage = custody::Usage::new(size_of::<State>(), 0);
        for (capacity, width) in capacities.into_iter().zip([
            size_of::<NodeId>(),
            size_of::<select::Task<NodeId>>(),
            size_of::<crate::DomDetailsToggleIntention>(),
        ]) {
            usage = usage.checked_add(custody::Usage::new(
                size_of::<Vec<NodeId>>()
                    .checked_add(
                        capacity
                            .checked_mul(width)
                            .ok_or(DomError::RepresentationExhausted)?,
                    )
                    .ok_or(DomError::RepresentationExhausted)?,
                0,
            ))?;
        }
        Ok(usage)
    }
}

impl Draft {
    pub(in crate::store) fn finish_sparse(
        &self,
        dom: &Dom,
        cancel: &AtomicBool,
        requested: impl ExactSizeIterator<Item = Key>,
    ) -> Result<(Prepared, Vec<NodeId>), DomError> {
        self.range_publication_ready(cancel)?;
        self.admit_dom_resources(dom.limits)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let budget = formation.as_ref();
        let NodePublication {
            custody: slots_custody,
            _touched_custody,
            touched,
            ids: allocated_ids,
            slots,
        } = self.finish_node_slots(cancel)?;
        let id = |key| match key {
            Key::Existing(id) => id,
            Key::Created(index) => allocated_ids
                .get(index)
                .expect("surviving public creation reference"),
        };
        let participant_id = |key| {
            if self.is_private(self.live_key(key)) {
                None
            } else {
                match key {
                    Key::Existing(id) => Some(id),
                    Key::Created(index) => allocated_ids.get(index),
                }
            }
        };
        let mut slot_index = self.finish_slot_index(&participant_id, cancel)?;
        let order = super::ordered_members::OrderPair {
            before: super::ordered_members::OrderView::state(&self.base, self.store),
            after: super::ordered_members::OrderView::projected(
                self,
                &slot_index.value,
                &allocated_ids,
            ),
        };
        let families = self.families.finish(self, &participant_id, order, cancel)?;
        let role_facts = self
            .role_facts
            .finish(self, &participant_id, order.after, cancel)?;
        let containers = self
            .containers
            .finish(self, &participant_id, order, cancel)?;
        let named = self
            .slot_names
            .finish(self, &participant_id, order, cancel)?;
        slot_index.value.named = named.value;
        slot_index.custody.named = named.custody;
        let controls = self.controls.finish(
            participant_id,
            self.custody_registry(),
            |usage| self.admit_custody_usage(usage, cancel),
            || self.source_step(cancel),
        )?;
        let mut mapped = Vec::new();
        source::formation::reserve(&mut mapped, requested.len(), budget, cancel)?;
        #[cfg(test)]
        let mut mapped_written = super::sequence::CopyObservation::default();
        for key in requested {
            self.source_step(cancel)?;
            self.node(key)?;
            mapped.push(id(key));
            #[cfg(test)]
            mapped_written.pushed::<NodeId>();
        }
        let mut root_debit = self
            .custody_ledger()
            .retain(custody::Usage::ZERO, |usage| {
                self.admit_custody_usage(usage, cancel)
            })?;
        let select::FinishedTasks {
            output_debit: native_output,
            tasks: native_tasks,
            selected: native_selected,
            next: next_task,
            cancellations,
            ack: native_ack,
        } = self.finish_select_tasks(dom, &id, budget, cancel)?;
        let details::Finished {
            output_debit: details_output,
            intentions: details_intentions,
            cancellations: details_cancellations,
            next: next_details_intention,
            ack: details_ack,
        } = self.finish_details(dom, &id, budget, cancel)?;
        let finished = self.view_transitions.finish(
            dom.owner,
            &dom.state.view_transitions,
            &dom.state_custody.transitions,
            self.custody_registry(),
            |usage| self.admit_custody_usage(usage, cancel),
            || self.source_step(cancel),
        )?;
        let ranges = self.finish_ranges(&id, cancel)?;
        let mut created_ranges = Vec::new();
        source::formation::reserve(
            &mut created_ranges,
            self.range_tickets.len(),
            budget,
            cancel,
        )?;
        #[cfg(test)]
        let mut ranges_written = super::sequence::CopyObservation::default();
        for ticket in &self.range_tickets {
            self.source_step(cancel)?;
            created_ranges.push(self.range_record(*ticket).ok().map(|_| *ticket));
            #[cfg(test)]
            ranges_written.pushed::<Option<crate::DomRangeId>>();
        }
        let (inbound_custody, inbound) = {
            let (value, custody) = self.inbound.finish(
                |key| (!self.is_private(key) && self.node(key).is_ok()).then(|| id(key)),
                self.custody_registry(),
                |usage| self.admit_custody_usage(usage, cancel),
                || self.source_step(cancel),
            )?;
            (custody, value)
        };
        let mut counts = self
            .counts
            .checked_replace(self.private_counts, super::Counts::default())?;
        counts.select_tasks = native_selected;
        counts.details_intentions = details_intentions.len();
        let (transitions, records, pseudos) = finished.value.counts();
        counts.transitions = transitions;
        counts.records = records;
        counts.pseudos = pseudos;
        let (signal_targets_custody, signal_targets) = {
            let (value, custody) = self
                .finish_signal_targets(&id, budget, cancel)?
                .into_parts();
            (custody, value)
        };
        let (native_targets_custody, native_targets) = {
            let (value, custody) = self
                .finish_native_targets(&id, budget, cancel)?
                .into_parts();
            (custody, value)
        };
        let (details_targets_custody, details_targets) = {
            let (value, custody) = self
                .finish_details_targets(&id, budget, cancel)?
                .into_parts();
            (custody, value)
        };
        let mut signals = self.finish_signals(&id, budget, cancel)?;
        for incoming in [native_output, details_output, signals.output_debit.take()]
            .into_iter()
            .flatten()
        {
            root_debit
                .absorb(incoming)
                .expect("same candidate root ledger");
        }
        let identity = custody::MaterializationId::fresh()?;
        let units = self.public_units(cancel)?;
        let usage = State::root_usage_for_capacities([
            signals.values.capacity(),
            native_tasks.capacity(),
            details_intentions.capacity(),
        ])?;
        super::nodes::resize_debit(&mut root_debit, usage, |usage| {
            self.admit_custody_usage(usage, cancel)
        })?;
        let state = custody::Held::new(
            State {
                identity,
                counts,
                inbound,
                controls: controls.value,
                slot_index: slot_index.value,
                family_index: families.value,
                role_facts: role_facts.value,
                container_index: containers.index,
                range_dependencies: ranges.dependencies,
                signal_targets,
                native_targets,
                details_targets,
                slots,
                ranges: ranges.ranges,
                incomplete_source: None,
                incomplete_hosted: dom.state.incomplete_hosted,
                registries: self.registries.clone(),
                hosted_globals: self.hosted_globals.clone(),
                hosted_history: self.hosted_history.clone(),
                view_transitions: finished.value,
                signal_slots: signals.values,
                notification_queued: self.notification_queued,
                native_tasks,
                next_task,
                details_intentions,
                next_details_intention,
                nodes: self.public_nodes(),
                units,
            },
            root_debit,
        );
        let (state, retained) = state.into_parts();
        let candidate = CandidateState {
            state,
            frontiers: super::StateFrontiers {
                slots: slots_custody,
                ranges: ranges.custody,
                inbound: inbound_custody,
                controls: controls.custody,
                slot_index: slot_index.custody,
                families: families.custody,
                role_facts: role_facts.custody,
                containers: containers.custody,
                range_dependencies: ranges.dependency_custody,
                transitions: finished.custody,
                signal_targets: signal_targets_custody,
                native_targets: native_targets_custody,
                details_targets: details_targets_custody,
                auxiliary: self.auxiliary.clone(),
            },
            retained,
        };
        let state = &candidate.state;
        let mut changes = sparse_difference(
            dom.owner,
            &dom.state,
            state,
            budget,
            cancel,
            &touched,
            self.ranges.touched(),
        )?;
        #[cfg(test)]
        let mut changes_written = super::sequence::CopyObservation::default();
        // Transition-domain changes precede the exact Document-origin closure.
        source::formation::reserve(&mut changes, finished.changes.value().len(), budget, cancel)?;
        let (transition_changes, changes_debit) = finished.changes.into_parts();
        #[cfg(test)]
        let transition_count = transition_changes.len();
        changes.extend(transition_changes);
        #[cfg(test)]
        changes_written.extended::<DomChange>(transition_count);
        drop(changes_debit);
        self.append_origin_changes(&dom.state, state, &touched, &mut changes, budget, cancel)?;
        if let Some(images) = &self.cancelled_images {
            for item in images.iter() {
                self.source_step(cancel)?;
                source::formation::reserve(&mut changes, 1, budget, cancel)?;
                changes.push(DomChange::ImageUpdateCancelled {
                    update: item.update,
                    flags: item.flags,
                    reason: crate::DomImageUpdateCancellationReason::TargetDestroyed,
                });
                #[cfg(test)]
                changes_written.pushed::<DomChange>();
            }
            #[cfg(test)]
            image::publication_cancellation::at_evidence(cancel);
        }
        let (cancellations, cancellations_debit) = cancellations.into_parts();
        for (operation, select) in cancellations {
            source::formation::reserve(&mut changes, 1, budget, cancel)?;
            changes.push(DomChange::SelectTaskCancelled {
                operation: crate::DomNativeTaskId {
                    owner: dom.owner,
                    serial: operation,
                },
                select,
                reason: crate::DomSelectTaskCancellationReason::TargetDestroyed,
            });
            #[cfg(test)]
            changes_written.pushed::<DomChange>();
        }
        drop(cancellations_debit);
        let (details_cancellations, details_cancellations_debit) =
            details_cancellations.into_parts();
        for item in details_cancellations {
            source::formation::reserve(&mut changes, 1, budget, cancel)?;
            changes.push(DomChange::DetailsToggleIntentionCancelled {
                operation: item.operation,
                target: item.target,
                old: item.old,
                new: item.new,
                reason: crate::DomDetailsToggleCancellationReason::TargetDestroyed,
            });
            #[cfg(test)]
            changes_written.pushed::<DomChange>();
        }
        drop(details_cancellations_debit);
        for slot in self.cancelled_signals.iter() {
            source::formation::reserve(&mut changes, 1, budget, cancel)?;
            changes.push(DomChange::SlotSignalCancelled {
                slot,
                reason: crate::DomSlotSignalCancellationReason::Destroyed,
            });
            #[cfg(test)]
            changes_written.pushed::<DomChange>();
        }
        let mut created = Vec::new();
        let mut documents = Vec::new();
        source::formation::reserve(&mut created, self.tickets.len(), budget, cancel)?;
        source::formation::reserve(&mut documents, self.tickets.len(), budget, cancel)?;
        #[cfg(test)]
        let mut tickets_written = super::sequence::CopyObservation::default();
        for ticket in &self.tickets {
            let Key::Created(index) = *ticket else {
                unreachable!("creation ticket")
            };
            let live = self.node(*ticket).ok();
            created.push(live.as_ref().and_then(|_| allocated_ids.get(index)));
            #[cfg(test)]
            tickets_written.pushed::<Option<NodeId>>();
            documents.push(live.is_some_and(|n| matches!(n.kind, DomNodeKind::Document(_))));
            #[cfg(test)]
            tickets_written.pushed::<bool>();
        }
        let mut retirements = Vec::new();
        source::formation::reserve(
            &mut retirements,
            self.dispatch_retirements.len(),
            budget,
            cancel,
        )?;
        #[cfg(test)]
        let mut retirements_written = super::sequence::CopyObservation::default();
        for &(target, document) in &self.dispatch_retirements {
            self.source_step(cancel)?;
            if dom
                .dispatch_custody
                .tracks(target, || self.source_step(cancel))?
            {
                retirements.push(dispatch::Retirement {
                    target,
                    document: DocumentId(id(document)),
                });
                #[cfg(test)]
                retirements_written.pushed::<dispatch::Retirement>();
            }
        }
        let semantic_changed = !changes.is_empty();
        Ok((
            Prepared {
                created_ranges,
                retirements,
                formation,
                allocation_ids: allocated_ids,
                state: candidate.state,
                state_frontiers: candidate.frontiers,
                root_debit: candidate.retained,
                semantic_changed,
                owner: self.owner.clone(),
                created,
                documents,
                transition_ids: finished.transition_ids,
                record_ids: finished.record_ids,
                changes,
                signal_ack: signals.ack,
                native_ack,
                details_ack,
                transition_ack: finished.ack,
                _transition_outputs: finished.output_custody,
            },
            mapped,
        ))
    }
}

impl Draft {
    pub(in crate::store) fn prepare_rebase(
        &self,
        publication: &mut PreparedPublication,
        cancel: &AtomicBool,
    ) -> Result<super::PreparedRebase, DomError> {
        self.source_step(cancel)?;
        let journal = self.journal.prepare_commit()?;
        Ok(super::PreparedRebase {
            state: publication.state.clone(),
            custody: publication.state_custody.clone(),
            stamp: publication.to,
            journal,
            ids: publication.allocation_ids.clone(),
            signal_ack: publication.signal_ack.take(),
            native_ack: publication.native_ack.take(),
            details_ack: publication.details_ack.take(),
            transition_ack: publication.transition_ack.clone(),
        })
    }
    pub(in crate::store) fn commit_publication(
        &mut self,
        rebase: super::PreparedRebase,
        committed: &super::PublicationCommitted,
    ) {
        self.journal.commit(rebase.journal);
        // Old provisional records release before their matching root custody.
        self.records.clear();
        self.existing = bank::Bank::default();
        self.existing_custody = bank::BankCustody::default();
        for (index, _) in self.created_dirty.iter() {
            self.created[index] = None;
        }
        self.created_dirty = bank::Bank::default();
        self.created_dirty_custody = bank::BankCustody::default();
        self.base = rebase.state;
        self.base_custody = rebase.custody;
        self.registries = self.base.registries.clone();
        self.hosted_globals = self.base.hosted_globals.clone();
        self.hosted_history = self.base.hosted_history.clone();
        self.auxiliary = self.base_custody.auxiliary.clone();
        self.basis = rebase.stamp;
        if let Some(map) = &mut self.phase_map {
            map.ids = rebase.ids;
        }
        if let Some(ack) = rebase.signal_ack {
            self.acknowledge_signals_publication(ack, committed);
        }
        if let Some(ack) = rebase.native_ack {
            self.acknowledge_native_publication(ack, committed);
        }
        if let Some(ack) = rebase.details_ack {
            self.acknowledge_details_publication(ack, committed);
        }
        rebase.transition_ack.apply(&mut self.view_transitions);
        self.inbound
            .mark_published(&self.base.inbound, &self.base_custody.inbound);
        self.controls
            .mark_published(&self.base.controls, &self.base_custody.controls);
        self.slot_index
            .mark_published(&self.base.slot_index, &self.base_custody.slot_index);
        self.slot_names.mark_published(
            &self.base.slot_index.named,
            &self.base_custody.slot_index.named,
        );
        self.families
            .mark_published(&self.base.family_index, &self.base_custody.families);
        self.role_facts
            .mark_published(&self.base.role_facts, &self.base_custody.role_facts);
        self.containers
            .mark_published(&self.base.container_index, &self.base_custody.containers);
        self.ranges = super::ranges::Ranges::new(&self.base, &self.base_custody);
        let base = self.base.clone();
        let custody = self.base_custody.clone();
        self.acknowledge_queue_targets_publication(
            (&base.signal_targets, &custody.signal_targets),
            (&base.native_targets, &custody.native_targets),
            (&base.details_targets, &custody.details_targets),
        );
        self.counts = self
            .base
            .counts
            .checked_add(self.private_counts)
            .expect("admitted public and private domain contributions");
        self.counts.select_tasks = self.native_tasks.counts().selected;
        self.counts.details_intentions = self.details_intentions.len();
    }
}

impl Draft {
    fn append_origin_changes(
        &self,
        old: &State,
        new: &State,
        touched: &bank::Bank<bool>,
        changes: &mut Vec<DomChange>,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let owner = self.store;
        let origin = |state: &State, document: DocumentId| -> Result<Option<NodeId>, DomError> {
            let Ok(node) = state.node(owner, document.node()) else {
                return Ok(None);
            };
            if !matches!(node.kind, DomNodeKind::Document(_)) {
                return Ok(None);
            }
            for child in node.children.iter() {
                self.source_step(cancel)?;
                if matches!(
                    state.node(owner, *child).map(|n| &n.kind),
                    Ok(DomNodeKind::Element(_))
                ) {
                    return Ok(Some(*child));
                }
            }
            Ok(None)
        };
        let scratch = budget.map(source::formation::Formation::fork);
        let mut candidates = Vec::new();
        let mut announced = Vec::new();
        #[cfg(test)]
        let mut written = super::sequence::CopyObservation::default();
        for change in changes.iter() {
            if let DomChange::ViewTransition(id) = change {
                source::formation::reserve(&mut announced, 1, scratch.as_ref(), cancel)?;
                announced.push(id.generation);
                #[cfg(test)]
                written.pushed::<u64>();
            }
        }
        announced.sort_unstable();
        announced.dedup();
        for (slot, _) in touched.iter() {
            let document = |state: &State| {
                state.slots.get(slot).and_then(|s| {
                    s.node
                        .as_ref()
                        .filter(|n| matches!(n.kind, DomNodeKind::Document(_)))
                        .map(|_| {
                            DocumentId(NodeId {
                                owner,
                                slot,
                                generation: s.generation,
                            })
                        })
                })
            };
            let before = document(old);
            let after = document(new);
            if before == after && before.is_some() {
                let a = old
                    .slots
                    .get(slot)
                    .and_then(|s| s.node.as_ref())
                    .expect("selected Document");
                let b = new
                    .slots
                    .get(slot)
                    .and_then(|s| s.node.as_ref())
                    .expect("selected Document");
                if a.children.shares_backing(&b.children) {
                    continue;
                }
            }
            for doc in [before, after.filter(|id| Some(*id) != before)]
                .into_iter()
                .flatten()
            {
                if origin(old, doc)? == origin(new, doc)? {
                    continue;
                }
                for id in old
                    .view_transitions
                    .origin_transitions(doc)
                    .chain(new.view_transitions.origin_transitions(doc))
                {
                    self.source_step(cancel)?;
                    let index = usize::try_from(id.generation)
                        .map_err(|_| DomError::RepresentationExhausted)?;
                    if old
                        .view_transitions
                        .transitions
                        .get(index)
                        .is_some_and(|t| t.id == id)
                        && new
                            .view_transitions
                            .transitions
                            .get(index)
                            .is_some_and(|t| t.id == id)
                    {
                        source::formation::reserve(&mut candidates, 1, scratch.as_ref(), cancel)?;
                        candidates.push(id);
                        #[cfg(test)]
                        written.pushed::<DomViewTransitionId>();
                    }
                }
            }
        }
        candidates.sort_unstable_by_key(|id| id.generation);
        candidates.dedup();
        for id in candidates {
            self.source_step(cancel)?;
            if announced.binary_search(&id.generation).is_err() {
                source::formation::reserve(changes, 1, budget, cancel)?;
                changes.push(DomChange::ViewTransition(id));
                #[cfg(test)]
                written.pushed::<DomChange>();
            }
        }
        Ok(())
    }
}

struct CandidateState {
    state: State,
    frontiers: super::StateFrontiers,
    retained: custody::Debit,
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale_metrics::{Counters, Hooks, Measurement};

    fn fixture() -> (Dom, DocumentId, NodeId, NodeId) {
        let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 32, 16, 8, 65536).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let form = batch.create_element_ns(&document, crate::Namespace::html(), "form");
        let button = batch.create_element_ns(&document, crate::Namespace::html(), "button");
        batch.insert_before(&form, &document, None);
        batch.insert_before(&button, &form, None);
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        (
            dom,
            commit.document(&document).unwrap(),
            commit.created(&form).unwrap().unwrap(),
            commit.created(&button).unwrap().unwrap(),
        )
    }

    fn existing(key: Key) -> NodeId {
        match key {
            Key::Existing(id) => id,
            Key::Created(_) => panic!("fixture operands are published identities"),
        }
    }

    fn measured<T>(operation: impl FnOnce() -> T) -> (T, Counters) {
        let measurement = Measurement::begin().unwrap();
        let result = operation();
        let counters = measurement.finish(Hooks::default()).unwrap();
        (result, counters)
    }

    #[test]
    fn finish_reports_initialized_selection_changes_and_public_receipts() {
        if !scale_metrics::ENABLED {
            return;
        }
        let (dom, document, _, _) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        let key = draft
            .allocate(
                DomNodeKind::Comment("new".into()),
                Key::Existing(document.node()),
                dom.limits,
                &cancel,
            )
            .unwrap();
        draft.register_ticket(key, &cancel).unwrap();
        let (result, counters) =
            measured(|| draft.finish_sparse(&dom, &cancel, std::iter::once(key)));
        let (prepared, mapped) = result.unwrap();
        assert_eq!(mapped.len(), 1);
        assert_eq!(prepared.created, [Some(mapped[0])]);
        assert_eq!(prepared.documents, [false]);
        assert_eq!(prepared.changes, [DomChange::Created(mapped[0])]);
        // One selected slot, one change, one requested ID and the two actual
        // public ticket fields are initialized. Empty relation fields alias
        // or initialize no elements, and no journal mark is present.
        assert_eq!(
            counters.sequence_materialized_bytes,
            size_of::<usize>()
                + size_of::<DomChange>()
                + size_of::<NodeId>()
                + size_of::<Option<NodeId>>()
                + size_of::<bool>()
        );
        assert_eq!(counters.relation_sequence_materialized_bytes, 0);
    }

    #[test]
    fn refused_mapped_output_reports_its_completed_initialized_prefix() {
        if !scale_metrics::ENABLED {
            return;
        }
        let (dom, document, _, _) = fixture();
        let cancel = AtomicBool::new(false);
        let draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        let stale = NodeId {
            owner: dom.owner,
            slot: dom.state.slots.len(),
            generation: 0,
        };
        let before = dom.snapshot().stamp();
        let (result, counters) = measured(|| {
            draft.finish_sparse(
                &dom,
                &cancel,
                [Key::Existing(document.node()), Key::Existing(stale)].into_iter(),
            )
        });
        assert!(matches!(result, Err(DomError::StaleNode)));
        assert_eq!(dom.snapshot().stamp(), before);
        assert_eq!(counters.sequence_materialized_bytes, size_of::<NodeId>());
        assert_eq!(counters.relation_sequence_materialized_bytes, 0);
    }

    #[test]
    fn key_and_event_publication_report_actual_capacity_and_retain_aliases_without_allocation() {
        if !scale_metrics::ENABLED {
            return;
        }
        let (dom, document, form, button) = fixture();
        let cancel = AtomicBool::new(false);
        let draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        let operands = vec![
            Key::Existing(document.node()),
            Key::Existing(form),
            Key::Existing(button),
        ];
        let mut debit = draft
            .custody_ledger()
            .retain(
                custody::Usage::new(
                    size_of::<Vec<NodeId>>() + operands.len() * size_of::<NodeId>(),
                    0,
                ),
                |_| Ok(()),
            )
            .unwrap();
        let (result, counters) = measured(|| {
            keys(
                SequenceView::Staged(&operands),
                &existing,
                &mut debit,
                &draft,
                &cancel,
            )
        });
        let values = result.unwrap();
        assert_eq!(values.as_slice(), &[document.node(), form, button]);
        assert_eq!(counters.structural_allocations, 2);
        assert_eq!(
            counters.structural_bytes,
            values.capacity() * size_of::<NodeId>() + size_of::<Vec<NodeId>>()
        );
        assert_eq!(
            counters.sequence_materialized_bytes,
            3 * size_of::<NodeId>()
        );
        assert_eq!(
            counters.relation_sequence_materialized_bytes,
            3 * size_of::<NodeId>()
        );
        let mut alias_debit = draft
            .custody_ledger()
            .retain(
                custody::Usage::new(
                    size_of::<Vec<NodeId>>() + values.capacity() * size_of::<NodeId>(),
                    0,
                ),
                |_| Ok(()),
            )
            .unwrap();
        let (alias, counters) = measured(|| {
            keys(
                SequenceView::Canonical(&values, Key::Existing),
                &existing,
                &mut alias_debit,
                &draft,
                &cancel,
            )
        });
        assert!(alias.unwrap().shares_backing(&values));
        drop(alias_debit);
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materialized_bytes, 0);
        assert_eq!(counters.relation_sequence_materialized_bytes, 0);

        let event = crate::DomFullscreenEvent {
            document,
            subject: button,
            kind: crate::DomFullscreenEventKind::Change,
        };
        let operands = vec![event; 3];
        let mut debit = draft
            .custody_ledger()
            .retain(
                custody::Usage::new(
                    size_of::<Vec<crate::DomFullscreenEvent>>()
                        + operands.len() * size_of::<crate::DomFullscreenEvent>(),
                    0,
                ),
                |_| Ok(()),
            )
            .unwrap();
        let (result, counters) =
            measured(|| events(SequenceView::Staged(&operands), &mut debit, &draft, &cancel));
        let values = result.unwrap();
        assert_eq!(values.as_slice(), &[event; 3]);
        assert_eq!(counters.structural_allocations, 2);
        assert_eq!(
            counters.structural_bytes,
            values.capacity() * size_of::<crate::DomFullscreenEvent>()
                + size_of::<Vec<crate::DomFullscreenEvent>>()
        );
        assert_eq!(
            counters.sequence_materialized_bytes,
            3 * size_of::<crate::DomFullscreenEvent>()
        );
        assert_eq!(
            counters.relation_sequence_materialized_bytes,
            3 * size_of::<crate::DomFullscreenEvent>()
        );
        let mut alias_debit = draft
            .custody_ledger()
            .retain(
                custody::Usage::new(
                    size_of::<Vec<crate::DomFullscreenEvent>>()
                        + values.capacity() * size_of::<crate::DomFullscreenEvent>(),
                    0,
                ),
                |_| Ok(()),
            )
            .unwrap();
        let (alias, counters) = measured(|| {
            events(
                SequenceView::Canonical(&values, std::convert::identity),
                &mut alias_debit,
                &draft,
                &cancel,
            )
        });
        assert!(alias.unwrap().shares_backing(&values));
        drop(alias_debit);
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materialized_bytes, 0);
        assert_eq!(counters.relation_sequence_materialized_bytes, 0);
    }

    #[test]
    fn cancellation_after_reserve_reports_the_allocated_backing_without_copying_members() {
        if !scale_metrics::ENABLED {
            return;
        }
        let (dom, document, form, button) = fixture();
        let cancel = AtomicBool::new(false);
        let draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        let operands = vec![
            Key::Existing(document.node()),
            Key::Existing(form),
            Key::Existing(button),
        ];
        let mut debit = draft
            .custody_ledger()
            .retain(
                custody::Usage::new(
                    size_of::<Vec<NodeId>>() + operands.len() * size_of::<NodeId>(),
                    0,
                ),
                |_| Ok(()),
            )
            .unwrap();
        cancel.store(true, Ordering::Relaxed);
        let (result, counters) = measured(|| {
            keys(
                SequenceView::Staged(&operands),
                &existing,
                &mut debit,
                &draft,
                &cancel,
            )
        });
        assert!(matches!(result, Err(DomError::Cancelled)));
        assert_eq!(counters.structural_allocations, 1);
        assert!(counters.structural_bytes >= operands.len() * size_of::<NodeId>());
        assert_eq!(counters.structural_bytes % size_of::<NodeId>(), 0);
        assert_eq!(counters.sequence_materializations, 0);
        assert_eq!(counters.sequence_materialized_bytes, 0);
        assert_eq!(counters.relation_sequence_materializations, 0);
        assert_eq!(counters.relation_sequence_materialized_bytes, 0);

        cancel.store(false, Ordering::Relaxed);
        let (result, counters) = measured(|| {
            keys(
                SequenceView::Staged(&operands),
                &|key| {
                    // The first real mapped value is appended before the next
                    // work check observes cancellation.
                    cancel.store(true, Ordering::Relaxed);
                    existing(key)
                },
                &mut debit,
                &draft,
                &cancel,
            )
        });
        assert!(matches!(result, Err(DomError::Cancelled)));
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(counters.sequence_materializations, 1);
        assert_eq!(counters.sequence_materialized_bytes, size_of::<NodeId>());
        assert_eq!(counters.relation_sequence_materializations, 1);
        assert_eq!(
            counters.relation_sequence_materialized_bytes,
            size_of::<NodeId>()
        );
    }

    #[test]
    fn staged_past_names_report_the_actual_publication_backing_capacity() {
        if !scale_metrics::ENABLED {
            return;
        }
        let (dom, _, form, button) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        let mut entries = Vec::with_capacity(19);
        for name in ["first", "next", "last"] {
            entries.push(crate::form::FormPastName {
                name: name.into(),
                target: Key::Existing(button),
            });
        }
        let names = super::super::names::Names::staged(
            crate::form::FormPastNames {
                entries: Arc::new(entries),
            },
            || draft.source_step(&cancel),
        )
        .unwrap();
        draft
            .with_node_fields_formed(Key::Existing(form), &cancel, None, |fields| {
                fields.form_past_names = Some(names);
            })
            .unwrap();
        draft.units += 13;
        let (formed, counters) =
            measured(|| draft.form_node(Key::Existing(form), &existing, &cancel));
        let formed = formed.unwrap();
        let names = formed
            .value()
            .node
            .as_ref()
            .unwrap()
            .form_past_names
            .as_ref()
            .unwrap();
        assert_eq!(names.entries.len(), 3);
        assert!(names.entries.iter().all(|entry| entry.target == button));
        assert_eq!(counters.structural_allocations, 2);
        assert_eq!(
            counters.structural_bytes,
            names.entries.capacity() * size_of::<crate::form::FormPastName<NodeId>>()
                + size_of::<Vec<crate::form::FormPastName<NodeId>>>()
        );
        assert_eq!(
            counters.sequence_materialized_bytes,
            3 * size_of::<crate::form::FormPastName<NodeId>>()
        );
        assert_eq!(counters.relation_sequence_materialized_bytes, 0);
    }
}
