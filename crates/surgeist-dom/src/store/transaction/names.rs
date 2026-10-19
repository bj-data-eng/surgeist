//! Form history is an independently materialized leaf. Staging and undo pair
//! its immutable semantics with custody; semantic nodes never hold a lease.
use super::super::*;
use crate::form::{FormPastName, FormPastNames, History};

#[cfg(test)]
pub(in crate::store) mod mapped_output_tests;

#[derive(Clone)]
pub(in crate::store) enum NamesBacking {
    Canonical(FormPastNames<NodeId>),
    Staged(FormPastNames<Key>),
}
#[derive(Clone)]
pub(in crate::store) struct Names {
    pub(super) backing: NamesBacking,
    custody: Option<bank::ObjectCustody>,
}
pub(in crate::store) struct FormedNames {
    pub value: FormPastNames<NodeId>,
    pub _custody: Option<bank::ObjectCustody>,
}

pub(in crate::store) fn describe<K>(
    entries: &Vec<FormPastName<K>>,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<(custody::Usage, usize), DomError> {
    if entries.capacity() == 0 {
        return Ok((custody::Usage::ZERO, 0));
    }
    let mut payload = source::storage::Payload::default();
    payload.add_bytes(size_of::<payload::Payload<History<K>>>())?;
    payload.bank::<FormPastName<K>>(entries.capacity())?;
    let mut units = 0usize;
    for entry in entries {
        step()?;
        payload.string(&entry.name)?;
        units = units
            .checked_add(entry.name.len())
            .ok_or(DomError::RepresentationExhausted)?;
    }
    Ok((custody::Usage::new(payload.bytes, payload.units), units))
}

fn history<K>(
    entries: custody::Held<Vec<FormPastName<K>>>,
    logical_units: usize,
) -> custody::Held<History<K>> {
    let (entries, debit) = entries.into_parts();
    custody::Held::new(
        History {
            entries,
            logical_units,
        },
        debit,
    )
}
fn seal<K>(
    entries: custody::Held<Vec<FormPastName<K>>>,
    usage: custody::Usage,
    logical_units: usize,
    draft: &Draft,
    cancel: &AtomicBool,
) -> Result<(FormPastNames<K>, Option<bank::ObjectCustody>), DomError> {
    if entries.value().capacity() == 0 {
        drop(entries);
        return Ok((FormPastNames::fresh(), None));
    }
    let sealed = payload::Pending::new(history(entries, logical_units), usage, |next| {
        draft.admit_custody_usage(next, cancel)
    })?
    .seal(draft.custody_registry(), |next| {
        draft.admit_custody_usage(next, cancel)
    })?;
    Ok((
        FormPastNames::from_payload(sealed.value),
        Some(sealed.custody),
    ))
}

pub(in crate::store) enum PreparedNames {
    Alias(FormedNames),
    Pending(payload::Pending<History<NodeId>>),
}
impl PreparedNames {
    fn seal(self, draft: &Draft, cancel: &AtomicBool) -> Result<FormedNames, DomError> {
        match self {
            Self::Alias(formed) => Ok(formed),
            Self::Pending(pending) => {
                let sealed = pending.seal(draft.custody_registry(), |next| {
                    draft.admit_custody_usage(next, cancel)
                })?;
                Ok(FormedNames {
                    value: FormPastNames::from_payload(sealed.value),
                    _custody: Some(sealed.custody),
                })
            }
        }
    }
}

impl Names {
    #[cfg(test)]
    pub(in crate::store) fn identity(&self) -> Option<custody::MaterializationId> {
        match &self.backing {
            NamesBacking::Canonical(value) => value.payload().map(|value| value.identity),
            NamesBacking::Staged(value) => value.payload().map(|value| value.identity),
        }
    }
    pub fn fresh() -> Self {
        Self {
            backing: NamesBacking::Staged(FormPastNames::fresh()),
            custody: None,
        }
    }
    pub fn from_read(
        value: super::records::NamesRead<'_>,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        use super::records::NamesRead;
        let (backing, custody) = match value {
            NamesRead::Empty => return Ok(Self::fresh()),
            NamesRead::Canonical(value) | NamesRead::AliasedCanonical(value, _) => {
                let custody = value
                    .payload()
                    .map(|v| registry.adopt_object(v.identity, v.own_usage, &mut admit))
                    .transpose()?;
                (NamesBacking::Canonical(value.clone()), custody)
            }
            NamesRead::Staged(value) | NamesRead::AliasedStaged(value, _) => {
                let custody = value
                    .payload()
                    .map(|v| registry.adopt_object(v.identity, v.own_usage, &mut admit))
                    .transpose()?;
                (NamesBacking::Staged(value.clone()), custody)
            }
        };
        Ok(Self { backing, custody })
    }
    pub fn staged(
        entries: Vec<FormPastName<Key>>,
        formation: Option<source::formation::Formation>,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let debit = formation
            .and_then(source::formation::Formation::into_debit)
            .map_or_else(
                || {
                    draft.custody_ledger().retain(custody::Usage::ZERO, |next| {
                        draft.admit_custody_usage(next, cancel)
                    })
                },
                Ok,
            )?;
        let entries = custody::Held::new(entries, debit);
        let (usage, units) = describe(entries.value(), || draft.source_step(cancel))?;
        let (value, custody) = seal(entries, usage, units, draft, cancel)?;
        Ok(Self {
            backing: NamesBacking::Staged(value),
            custody,
        })
    }
    pub fn project(
        &self,
        draft: &Draft,
        map: &impl Fn(Key) -> NodeId,
        cancel: &AtomicBool,
    ) -> Result<FormedNames, DomError> {
        self.prepare_projection(draft, map, cancel)?
            .seal(draft, cancel)
    }
    pub fn prepare_projection(
        &self,
        draft: &Draft,
        map: &impl Fn(Key) -> NodeId,
        cancel: &AtomicBool,
    ) -> Result<PreparedNames, DomError> {
        let NamesBacking::Staged(values) = &self.backing else {
            let NamesBacking::Canonical(value) = &self.backing else {
                unreachable!()
            };
            return Ok(PreparedNames::Alias(FormedNames {
                value: value.clone(),
                _custody: self.custody.clone(),
            }));
        };
        if values.entries().is_empty() {
            return Ok(PreparedNames::Alias(FormedNames {
                value: FormPastNames::fresh(),
                _custody: None,
            }));
        }
        let old_bank = values
            .capacity()
            .checked_mul(size_of::<FormPastName<Key>>())
            .and_then(|bytes| bytes.checked_add(size_of::<payload::Payload<History<Key>>>()))
            .ok_or(DomError::RepresentationExhausted)?;
        let new_bank = values
            .entries()
            .len()
            .checked_mul(size_of::<FormPastName<NodeId>>())
            .and_then(|bytes| bytes.checked_add(size_of::<payload::Payload<History<NodeId>>>()))
            .ok_or(DomError::RepresentationExhausted)?;
        let usage = values.own_usage().checked_replace(
            custody::Usage::new(old_bank, 0),
            custody::Usage::new(new_bank, 0),
        )?;
        let reserved = payload::Reserved::new()?;
        let debit = draft
            .custody_ledger()
            .retain(usage, |next| draft.admit_custody_usage(next, cancel))?;
        let output = Vec::new();
        #[cfg(test)]
        let output = mapped_output_tests::output_buffer(output);
        let mut out = custody::Held::new(output, debit);
        #[cfg(test)]
        let old_capacity = out.value().capacity();
        out.value_mut()
            .try_reserve_exact(values.entries().len())
            .map_err(|_| DomError::AllocationFailed)?;
        #[cfg(test)]
        if scale_metrics::ENABLED && old_capacity != out.value().capacity() {
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: out.value().capacity() * size_of::<FormPastName<NodeId>>(),
            });
        }
        let extra = (out.value().capacity() - values.entries().len())
            .checked_mul(size_of::<FormPastName<NodeId>>())
            .ok_or(DomError::RepresentationExhausted)?;
        out.debit_mut()
            .grow(custody::Usage::new(extra, 0), |next| {
                draft.admit_custody_usage(next, cancel)
            })?;
        let usage = usage.checked_add(custody::Usage::new(extra, 0))?;
        #[cfg(test)]
        let mut copied = CopyObservation::default();
        for entry in values.entries() {
            draft.source_step(cancel)?;
            out.value_mut().push(FormPastName {
                name: entry.name.clone(),
                target: map(entry.target),
            });
            #[cfg(test)]
            copied.pushed::<FormPastName<NodeId>>();
        }
        Ok(PreparedNames::Pending(reserved.materialize(
            history(out, values.logical_units()),
            usage,
            |next| draft.admit_custody_usage(next, cancel),
        )?))
    }
    pub fn view(&self) -> super::records::NamesRead<'_> {
        match &self.backing {
            NamesBacking::Canonical(values) => super::records::NamesRead::Canonical(values),
            NamesBacking::Staged(values) => super::records::NamesRead::Staged(values),
        }
    }
    #[cfg(test)]
    pub fn logical_units(&self) -> usize {
        match &self.backing {
            NamesBacking::Canonical(values) => values.logical_units(),
            NamesBacking::Staged(values) => values.logical_units(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner(source: bool) -> (Dom, Draft) {
        let dom = Dom::new(
            DomLimits::new(
                32,
                4096,
                64,
                0,
                0,
                16,
                1_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let batch = DomBatch::new();
        let draft = if source {
            Draft::new_source(
                &dom,
                batch.owner,
                &batch.commands,
                crate::DomSourceLimits::new(16_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
                &cancel,
            )
            .unwrap()
            .0
        } else {
            Draft::new(&dom, batch.owner, &cancel).unwrap()
        };
        (dom, draft)
    }
    fn staged(names: &Names) -> &FormPastNames<Key> {
        let NamesBacking::Staged(value) = &names.backing else {
            panic!("staged fixture")
        };
        value
    }

    #[test]
    fn absence_reserved_and_aliased_name_occurrences_have_exact_leaf_receipts() {
        let (_dom, draft) = owner(false);
        let cancel = AtomicBool::new(false);
        draft.custody_registry().reconcile_releases();
        let baseline = draft.custody_ledger().total();
        let empty = Names::staged(Vec::new(), None, &draft, &cancel).unwrap();
        assert!(staged(&empty).payload().is_none());
        assert_eq!(draft.custody_ledger().total(), baseline);
        assert_eq!(empty.logical_units(), 0);
        let entries = Vec::<FormPastName<Key>>::with_capacity(7);
        let capacity = entries.capacity();
        let reserved = Names::staged(entries, None, &draft, &cancel).unwrap();
        assert_eq!(
            staged(&reserved).own_usage(),
            custody::Usage::new(
                size_of::<payload::Payload<History<Key>>>()
                    + capacity * size_of::<FormPastName<Key>>(),
                0
            )
        );
        assert_eq!(staged(&reserved).entries().len(), 0);
        assert!(!staged(&empty).same_storage(staged(&reserved)));
        assert!(!staged(&reserved).same_storage(staged(&empty)));
        assert!(staged(&reserved).same_storage(staged(&reserved.clone())));
        let mut raw_name = Vec::with_capacity(19);
        raw_name.extend([0xd800, 0, 0xdc00]);
        let name_capacity = raw_name.capacity();
        let name = DocumentString::from_utf16(raw_name);
        let mut entries = Vec::with_capacity(7);
        for target in [Key::Created(0), Key::Created(1)] {
            entries.push(FormPastName {
                name: name.clone(),
                target,
            });
        }
        let capacity = entries.capacity();
        let names = Names::staged(entries, None, &draft, &cancel).unwrap();
        assert_eq!(
            staged(&names).own_usage(),
            custody::Usage::new(
                size_of::<payload::Payload<History<Key>>>()
                    + capacity * size_of::<FormPastName<Key>>()
                    + 2 * (size_of::<Vec<u16>>() + name_capacity * size_of::<u16>()),
                2 * name_capacity
            )
        );
        assert_eq!(names.logical_units(), 6);
        let before_alias = draft.custody_ledger().total();
        let alias = names.clone();
        assert!(staged(&names).same_storage(staged(&alias)));
        assert_eq!(draft.custody_ledger().total(), before_alias);
        assert_eq!(
            alias
                .view()
                .iter()
                .map(|entry| entry.target)
                .collect::<Vec<_>>(),
            [Key::Created(0), Key::Created(1)]
        );
        drop(alias);
        drop(names);
        drop(reserved);
        drop(empty);
        draft.custody_registry().reconcile_releases();
        assert_eq!(draft.custody_ledger().total(), baseline);
    }

    #[test]
    fn source_formation_transfers_to_leaf_and_projection_is_a_new_materialization() {
        let (_dom, draft) = owner(true);
        let cancel = AtomicBool::new(false);
        draft.custody_registry().reconcile_releases();
        let baseline = draft.custody_ledger().total();
        let formation = source::formation::Formation::new(&draft, &cancel)
            .unwrap()
            .unwrap();
        let mut entries = Vec::new();
        source::formation::reserve(&mut entries, 2, Some(&formation), &cancel).unwrap();
        formation
            .retain_bytes(size_of::<Vec<FormPastName<Key>>>(), &cancel)
            .unwrap();
        for target in [Key::Created(0), Key::Created(1)] {
            let name = DocumentString::from("same");
            let mut usage = source::storage::Payload::default();
            usage.string(&name).unwrap();
            formation.retain_payload(usage, &cancel).unwrap();
            entries.push(FormPastName { name, target });
        }
        let names = Names::staged(entries, Some(formation), &draft, &cancel).unwrap();
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        let original = staged(&names).payload().unwrap();
        assert_eq!(original.own_usage.units(), 8);
        let id = NodeId {
            owner: draft.store,
            slot: 7,
            generation: 3,
        };
        let stable = draft.custody_ledger().total();
        let issuer = payload::identity_refusal::Refusal::begin();
        assert!(matches!(
            names.prepare_projection(&draft, &|_| id, &cancel),
            Err(DomError::RepresentationExhausted)
        ));
        assert!(issuer.reached());
        assert_eq!(
            draft.custody_ledger().total(),
            stable,
            "projection reserves identity before acquiring/copying its replacement"
        );
        drop(issuer);
        let projected = names.project(&draft, &|_| id, &cancel).unwrap();
        let final_payload = projected.value.payload().unwrap();
        assert_ne!(original.identity, final_payload.identity);
        assert_eq!(
            projected
                .value
                .entries()
                .iter()
                .map(|entry| entry.target)
                .collect::<Vec<_>>(),
            [id, id]
        );
        assert_eq!(projected.value.logical_units(), 8);
        assert_eq!(original.value.entries[0].target, Key::Created(0));
        let pending_bytes = draft.custody_ledger().total().bytes();
        assert!(
            pending_bytes
                >= baseline.bytes() + original.own_usage.bytes() + final_payload.own_usage.bytes()
        );
        drop(projected);
        drop(names);
        draft.custody_registry().reconcile_releases();
        assert_eq!(draft.custody_ledger().total(), baseline);
    }
}
