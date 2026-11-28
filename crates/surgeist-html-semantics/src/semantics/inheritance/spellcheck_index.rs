//! Metered indexing of the spellcheck rows belonging to one HTML capture.

use std::num::NonZeroUsize;

use super::{HtmlGlobalInheritanceError, HtmlSpellcheckDefaultFact, ScanMeter};
use crate::HtmlCaptureSubjectId;
use surgeist_html::accounting::HtmlResource;

type Error = HtmlGlobalInheritanceError;

#[derive(Debug)]
pub(super) struct SpellcheckIndex(Vec<Option<NonZeroUsize>>);
impl SpellcheckIndex {
    /// The next power of two at or above twice the row count uses at most
    /// four seats per row. Empty policies need no index backing.
    pub(super) fn byte_ceiling(rows: usize) -> usize {
        rows.saturating_mul(4 * size_of::<Option<NonZeroUsize>>())
    }

    pub(super) fn new(rows: usize, meter: &mut ScanMeter<Error>) -> Result<Self, Error> {
        if rows == 0 {
            return Ok(Self(Vec::new()));
        }
        let slots = rows
            .checked_mul(2)
            .and_then(usize::checked_next_power_of_two)
            .ok_or(Error::RepresentationExhausted)?;
        let bytes = slots
            .checked_mul(size_of::<Option<NonZeroUsize>>())
            .ok_or(Error::RepresentationExhausted)?;
        meter.acquire(HtmlResource::FactBytes, bytes, |limit, required| {
            Error::SpellcheckIndexBytes { limit, required }
        })?;
        meter.charge(slots)?;
        let mut positions = Vec::new();
        positions
            .try_reserve_exact(slots)
            .map_err(|_| Error::RepresentationExhausted)?;
        if positions.capacity() != slots {
            return Err(Error::RepresentationExhausted);
        }
        positions.resize(slots, None);
        Ok(Self(positions))
    }

    pub(super) fn insert(
        &mut self,
        facts: &[HtmlSpellcheckDefaultFact],
        position: usize,
        meter: &mut ScanMeter<Error>,
    ) -> Result<(), Error> {
        let subject = facts
            .get(position)
            .ok_or(Error::RepresentationExhausted)?
            .subject();
        let (slot, previous) = self
            .find(facts, subject, || meter.charge(1))?
            .ok_or(Error::RepresentationExhausted)?;
        if previous.is_some() {
            return Err(Error::DuplicateSubject(subject.clone()));
        }
        meter.charge(1)?;
        let entry = position
            .checked_add(1)
            .and_then(NonZeroUsize::new)
            .ok_or(Error::RepresentationExhausted)?;
        self.0[slot] = Some(entry);
        Ok(())
    }

    pub(super) fn position_with<E>(
        &self,
        facts: &[HtmlSpellcheckDefaultFact],
        subject: &HtmlCaptureSubjectId,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<usize>, E> {
        Ok(self
            .find(facts, subject, step)?
            .and_then(|(_, position)| position))
    }

    fn find<E>(
        &self,
        facts: &[HtmlSpellcheckDefaultFact],
        subject: &HtmlCaptureSubjectId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<(usize, Option<usize>)>, E> {
        let len = self.0.len();
        let Some(mask) = len.checked_sub(1) else {
            return Ok(None);
        };
        let mut slot = subject.ordinal() & mask;
        for _ in 0..len {
            step()?;
            let Some(at) = self.0[slot] else {
                return Ok(Some((slot, None)));
            };
            let position = at.get() - 1;
            if facts
                .get(position)
                .is_some_and(|fact| fact.subject() == subject)
            {
                return Ok(Some((slot, Some(position))));
            }
            slot = (slot + 1) & mask;
        }
        Ok(None)
    }
}
