//! The session's side of the strings of non-element fields: the units and capacity bytes the
//! parse holds for them, and the admission and accounting of an append to a text.
use super::*;
use crate::parser::control::{HtmlParseInvariant, TextAccount};
use surgeist_html::{
    accounting::{HtmlChange, HtmlChargedError, HtmlResource, HtmlWork},
    bounded::construction::{
        HtmlConstructionTextCapacity, HtmlConstructionTextError, HtmlConstructionTextGrowth,
        HtmlTextAdmission,
    },
    construction::ConstructionNodeKind,
};
use surgeist_ledger::Charge;

/// What the controller admits and records for one append to a text.
struct Admission<'a> {
    control: &'a mut ParseControl,
    /// The units the parse holds for all strings of non-element fields.
    units: &'a mut u64,
    /// The bytes the parse holds for all of them.
    bytes: &'a mut u64,
    /// The growth under way: the text before it, and the bytes of the layout it was admitted to.
    growing: Option<(HtmlConstructionTextCapacity, u64)>,
}

impl Charge<HtmlWork, HtmlResource> for Admission<'_> {
    type Stop = HtmlParseStop;

    fn charge(&mut self, batch: &[HtmlChange]) -> Result<(), HtmlParseStop> {
        Charge::charge(&mut *self.control, batch)
    }

    fn record_largest(&mut self, resource: HtmlResource, value: u64) -> Result<(), HtmlParseStop> {
        Charge::record_largest(&mut *self.control, resource, value)
    }
}

impl HtmlTextAdmission for Admission<'_> {
    fn admit_append(&mut self, growth: HtmlConstructionTextGrowth) -> Result<usize, HtmlParseStop> {
        let (target, expected) = self.control.admit_text_append(
            growth.before(),
            growth.added(),
            growth.minimum_capacity(),
        )?;
        if growth.grows() {
            self.growing = Some((growth.before(), expected));
        }
        Ok(target)
    }

    fn record_growth(
        &mut self,
        reached: HtmlConstructionTextCapacity,
    ) -> Result<(), HtmlParseStop> {
        let Some((before, expected)) = self.growing.take() else {
            return Err(self
                .control
                .fail_invariant(HtmlParseInvariant::CapacityMismatch));
        };
        let recorded = self
            .control
            .record_text_growth(before, reached, expected, self.bytes);
        // The capacity just recorded is the one the next record replaces, and an allocator that
        // refused the reservation reports the layout it was asked for.
        self.growing = Some((reached, expected));
        recorded
    }

    fn record_units(&mut self, added: usize) -> Result<(), HtmlParseStop> {
        let held = u64::try_from(added)
            .ok()
            .and_then(|added| self.units.checked_add(added))
            .ok_or_else(|| {
                self.control
                    .stop_representation(HtmlParseResource::ManagedUtf16Units)
            })?;
        self.control.acquire_text_units(added)?;
        *self.units = held;
        Ok(())
    }
}

impl Admission<'_> {
    fn error(self, error: HtmlConstructionTextError<HtmlParseStop>) -> SessionError {
        match error {
            HtmlChargedError::Stopped(stop) => SessionError::Stop(stop),
            HtmlChargedError::Faulted(error) => {
                if error.cause() == HtmlConstructionCause::AllocationFailed {
                    let layout = self
                        .growing
                        .map_or(0, |(_, bytes)| usize::try_from(bytes).unwrap_or(usize::MAX));
                    return SessionError::Stop(self.control.construction_allocation_failed(layout));
                }
                construction_error(self.control, error)
            }
        }
    }
}

impl ConstructionOwner {
    /// Takes the accounting of a string that the parse built and handed to a record of a
    /// non-element field: its units and the bytes of its capacity and wrapper, which the parse
    /// releases with the strings it holds. The string is already counted by the managed units.
    pub(in crate::construction) fn absorb_text(
        &mut self,
        control: &mut ParseControl,
        account: TextAccount,
    ) -> Result<(), SessionError> {
        let held = self
            .string_units
            .checked_add(account.units)
            .zip(self.string_bytes.checked_add(account.bytes));
        let Some((units, bytes)) = held else {
            return Err(control
                .stop_representation(HtmlParseResource::ManagedCapacityBytes)
                .into());
        };
        self.string_units = units;
        self.string_bytes = bytes;
        Ok(())
    }

    /// Releases the units and the bytes the parse holds for the strings of non-element fields.
    /// This is the only place they are credited.
    pub(super) fn release_text_accounts(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let units = std::mem::take(&mut self.string_units);
        let bytes = std::mem::take(&mut self.string_bytes);
        control
            .release_text_accounts(units, bytes)
            .map_err(SessionError::Stop)
    }

    /// Appends `units` to the text of the record at `node` and returns the length the text had.
    /// The text, its length, the usage and the accounts are unchanged by a refusal, except for
    /// the capacity a growth recorded and reserved before the refusal, which the text keeps.
    pub(in crate::construction) fn append_text(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        units: &[u16],
    ) -> Result<usize, SessionError> {
        control.require_identity(&self.control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        let record = self
            .storage(control)?
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?;
        if !matches!(record.kind(), ConstructionNodeKind::Text(_)) {
            return Err(SessionError::WrongToken);
        }
        let handle = record.handle();
        let Self {
            owner,
            string_units,
            string_bytes,
            ..
        } = self;
        let Some(storage) = owner.as_mut() else {
            return Err(latch_registry_row(control));
        };
        let mut admission = Admission {
            control,
            units: string_units,
            bytes: string_bytes,
            growing: None,
        };
        storage
            .append_text(&handle, units, &mut admission)
            .map_err(|error| admission.error(error))
    }
}
