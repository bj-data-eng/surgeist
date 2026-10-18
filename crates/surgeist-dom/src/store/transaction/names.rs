//! Past-name operands preserve canonical identities until that selected map is
//! rewritten. Cached receipts describe each materialized field occurrence.
use super::super::*;

#[derive(Clone)]
pub(in crate::store) enum NamesBacking {
    Canonical(crate::form::FormPastNames<NodeId>),
    Staged(crate::form::FormPastNames<Key>),
}

#[derive(Clone)]
pub(in crate::store) struct Names {
    pub(super) backing: NamesBacking,
    pub(super) usage: custody::Usage,
    pub(super) logical_units: usize,
}
impl Names {
    pub fn canonical(
        values: &crate::form::FormPastNames<NodeId>,
        usage: custody::Usage,
        logical_units: usize,
    ) -> Self {
        Self {
            backing: NamesBacking::Canonical(values.clone()),
            usage,
            logical_units,
        }
    }
    pub fn staged(
        values: crate::form::FormPastNames<Key>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let mut payload = source::storage::Payload::default();
        payload.add_bytes(values.entries.retained_bytes()?)?;
        let mut logical_units = 0usize;
        for entry in values.entries.iter() {
            step()?;
            payload.string(&entry.name)?;
            logical_units = logical_units
                .checked_add(entry.name.len())
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(Self {
            backing: NamesBacking::Staged(values),
            usage: custody::Usage::new(payload.bytes, payload.units),
            logical_units,
        })
    }
    pub fn view(&self) -> super::records::NamesRead<'_> {
        match &self.backing {
            NamesBacking::Canonical(values) => super::records::NamesRead::Canonical(values),
            NamesBacking::Staged(values) => super::records::NamesRead::Staged(values),
        }
    }
    pub fn usage(&self) -> custody::Usage {
        self.usage
    }
    pub fn logical_units(&self) -> usize {
        self.logical_units
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::form::{FormPastName, FormPastNames};

    #[test]
    fn absence_and_reserved_histories_keep_exact_occurrence_receipts() {
        let empty = Names::staged(FormPastNames::fresh(), || Ok(())).unwrap();
        assert_eq!(empty.usage(), custody::Usage::ZERO);
        assert_eq!(empty.logical_units(), 0);
        assert_eq!(empty.view().len(), 0);

        let entries = Vec::<FormPastName<Key>>::with_capacity(7);
        let capacity = entries.capacity();
        let reserved = Names::staged(FormPastNames::from_entries(entries), || Ok(())).unwrap();
        assert_eq!(
            reserved.usage(),
            custody::Usage::new(
                size_of::<Vec<FormPastName<Key>>>() + capacity * size_of::<FormPastName<Key>>(),
                0
            )
        );
        assert_eq!(reserved.logical_units(), 0);
        assert_eq!(reserved.view().len(), 0);

        let mut raw_name = Vec::with_capacity(19);
        raw_name.extend([0xd800, 0, 0xdc00]);
        let name_capacity = raw_name.capacity();
        let name = DocumentString::from_utf16(raw_name);
        let mut entries = Vec::with_capacity(7);
        entries.push(FormPastName {
            name: name.clone(),
            target: Key::Created(0),
        });
        entries.push(FormPastName {
            name,
            target: Key::Created(1),
        });
        let entry_capacity = entries.capacity();
        let populated = Names::staged(FormPastNames::from_entries(entries), || Ok(())).unwrap();
        let expected = custody::Usage::new(
            size_of::<Vec<FormPastName<Key>>>()
                + entry_capacity * size_of::<FormPastName<Key>>()
                + 2 * (size_of::<Vec<u16>>() + name_capacity * size_of::<u16>()),
            2 * name_capacity,
        );
        assert_eq!(populated.usage(), expected);
        assert_eq!(populated.logical_units(), 6);
        assert_eq!(populated.view().len(), 2);
        let clone = populated.clone();
        assert_eq!(clone.usage(), expected);
        assert_eq!(
            clone
                .view()
                .iter()
                .map(|entry| entry.target)
                .collect::<Vec<_>>(),
            [Key::Created(0), Key::Created(1)]
        );
    }
}
