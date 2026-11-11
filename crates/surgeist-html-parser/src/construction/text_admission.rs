//! The session's side of an append to a text: its admission and accounting.
use super::*;
use crate::parser::control::HtmlParseInvariant;
use surgeist_html::construction::HtmlConstructionCause;
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

impl SessionStorage {
    /// Appends `units` to the text of the record at `node` and returns the length the text had.
    /// The text, its length, the usage and the accounts are unchanged by a refusal, except for
    /// the capacity a growth recorded and reserved before the refusal, which the text keeps.
    pub(super) fn append_text(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        units: &[u16],
    ) -> Result<usize, SessionError> {
        control.require_identity(&self.control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        let record = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?;
        if !matches!(record.kind(), ConstructionNodeKind::Text(_)) {
            return Err(SessionError::WrongToken);
        }
        let handle = record.handle();
        let Self {
            construction: storage,
            holds,
            ..
        } = self;
        let mut admission = Admission {
            control,
            units: &mut holds.string_units,
            bytes: &mut holds.string_bytes,
            growing: None,
        };
        storage
            .append_text(&handle, units, &mut admission)
            .map_err(|error| admission.error(error))
    }
}
