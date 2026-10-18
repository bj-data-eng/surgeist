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
        payload.add_bytes(size_of::<Vec<crate::form::FormPastName<Key>>>())?;
        payload.bank::<crate::form::FormPastName<Key>>(values.entries.capacity())?;
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
