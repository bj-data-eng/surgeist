//! One whole-color CSS operation, with consumer-owned actual banks and no fallback.
use super::*;
use std::mem::size_of;
use surgeist_color::Color;
use surgeist_css::{
    CssColorActionLayout, CssColorBank, CssColorError, CssColorFactIdentity, CssColorFacts,
    CssColorGrammarRequest, CssColorInputKind, CssColorOperationOwner, CssColorParse,
    CssColorParseAction, CssColorParseOutcome, CssColorParseStep, CssColorPropertyFacts,
    CssColorSchemeFact, CssColorSlot, CssDeviceColorFact,
};

fn error(error: CssColorError) -> DomError {
    DomError::Meta(if let Some(value) = error.unavailable() {
        Failure::ThemeColorUnavailable(value)
    } else if let Some(value) = error.unsupported() {
        Failure::ThemeColorUnsupported(value)
    } else if let Some(value) = error.fact_defect() {
        Failure::ThemeColorFacts(value)
    } else {
        // The adapter's UTF-16 input admits every unit, so a malformed-input
        // report is an adapter inconsistency.
        Failure::CssProcessingProtocol
    })
}
fn index(bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => 0,
        CssColorBank::Arithmetic => 1,
        CssColorBank::Evaluation => 2,
    }
}
struct Scratch<'a> {
    draft: &'a Draft,
    source: Option<source::formation::Formation>,
    live: usize,
    limit: usize,
    cancel: &'a AtomicBool,
}
impl Scratch<'_> {
    fn work(&self, bytes: usize) -> Result<(), DomError> {
        for _ in 0..bytes {
            self.draft.meta_step(self.cancel)?;
        }
        Ok(())
    }
    fn add(&mut self, bytes: usize) -> Result<(), DomError> {
        if bytes == 0 {
            return Ok(());
        }
        let next = self
            .live
            .checked_add(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.limit {
            return Err(DomError::Meta(Failure::ThemeColorScratchLimit {
                limit: self.limit,
            }));
        }
        self.work(bytes)?;
        if let Some(source) = &self.source {
            source.retain_bytes(bytes, self.cancel)?;
        }
        self.live = next;
        Ok(())
    }
    fn release(&mut self, bytes: usize) {
        self.live = self.live.checked_sub(bytes).expect("actual theme scratch");
        if let Some(source) = &self.source {
            source.release_payload(source::storage::Payload { bytes, units: 0 });
        }
    }
    fn reserve<T>(&mut self, values: &mut Vec<T>) -> Result<(), DomError> {
        if values.len() < values.capacity() {
            return Ok(());
        }
        let required = values
            .len()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let capacity = values
            .capacity()
            .checked_mul(2)
            .unwrap_or(required)
            .max(required);
        let bytes = capacity
            .checked_mul(size_of::<T>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomError::RepresentationExhausted)?;
        let old = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        // Old and prospective new allocations coexist during actual growth.
        self.add(bytes)?;
        #[cfg(test)]
        let previous = (values.as_ptr(), values.len(), values.capacity());
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|_| DomError::AllocationFailed)?;
        #[cfg(test)]
        {
            if super::super::scale_metrics::ENABLED
                && values.capacity() != previous.2
                && size_of::<T>() != 0
            {
                super::super::scale_metrics::record(
                    super::super::scale_metrics::Event::StructuralAllocation {
                        bytes: values.capacity() * size_of::<T>(),
                    },
                );
            }
            if values.as_ptr() != previous.0 {
                let mut moved = crate::store::CopyObservation::default();
                moved.extended::<T>(previous.1);
            }
        }
        let actual = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        self.add(
            actual
                .checked_sub(bytes)
                .ok_or(DomError::RepresentationExhausted)?,
        )?;
        self.release(old);
        Ok(())
    }
}

pub(super) fn parse(
    draft: &Draft,
    input: &DocumentString,
    limits: DomLimits,
    cancel: &AtomicBool,
) -> Result<Option<Color>, DomError> {
    // HTML selects the original UTF16 subrange by its own ASCII-whitespace
    // predicate; CSS still owns all preprocessing and the whole-color grammar.
    let units = input.code_units();
    let mut start = 0;
    let mut end = units.len();
    while start < end {
        draft.meta_step(cancel)?;
        if !surgeist_html::bounded::is_ascii_whitespace(units[start]) {
            break;
        }
        start += 1;
    }
    while start < end {
        draft.meta_step(cancel)?;
        if !surgeist_html::bounded::is_ascii_whitespace(units[end - 1]) {
            break;
        }
        end -= 1;
    }
    let input = &units[start..end];
    let mut scratch = Scratch {
        draft,
        source: source::formation::Formation::new(draft, cancel)?,
        live: 0,
        limit: limits.meta_theme_scratch,
        cancel,
    };
    scratch.add(
        size_of::<CssColorOperationOwner>()
            + size_of::<CssColorFacts<'_>>()
            + size_of::<CssColorParse<'_, '_>>()
            + size_of::<[Vec<CssColorSlot<'_>>; 3]>()
            + size_of::<CssColorParseOutcome<'_>>(),
    )?;
    let mut owner = CssColorOperationOwner::new();
    let facts = CssColorFacts::new(
        CssColorFactIdentity::new(0x444f_4d5f_4d45_5441, 1),
        CssColorPropertyFacts::InitialProperties,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    );
    #[cfg(test)]
    let mut written = crate::store::CopyObservation::default();
    let mut banks: [Vec<CssColorSlot<'_>>; 3] = std::array::from_fn(|_| Vec::new());
    let mut parse = CssColorParse::start(
        &mut owner,
        CssColorInputKind::Utf16,
        input.len(),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
    )
    .map_err(error)?;
    // A refusal drops the pending parse or action, which abandons the parse.
    let result = (|| {
        let outcome = loop {
            draft.meta_step(cancel)?;
            let action = match parse.step() {
                CssColorParseStep::Complete(outcome) => break outcome,
                CssColorParseStep::Pending(action) => action,
            };
            scratch.work(
                CssColorParse::register_bytes()
                    .checked_mul(2)
                    .ok_or(DomError::RepresentationExhausted)?,
            )?;
            // A typed action is its own description; it is observed with its layout.
            let fixed = CssColorParse::action_bytes() + size_of::<CssColorActionLayout>();
            scratch.add(fixed)?;
            let layout = action.layout();
            for bytes in [
                layout.input_unit_bytes,
                layout.slot_read_bytes,
                layout.slot_write_bytes,
                layout.numerical_read_bytes,
                layout.numerical_write_bytes,
                layout.output_unit_bytes,
                layout.fact_records[0],
                layout.fact_records[1],
                layout.fact_records[2],
            ] {
                scratch.work(bytes)?;
            }
            parse = match action {
                CssColorParseAction::ReadInput(read) => {
                    let unit = *input
                        .get(read.offset())
                        .ok_or(DomError::Meta(Failure::CssProcessingProtocol))?;
                    read.read(unit).map_err(error)?
                }
                CssColorParseAction::ReadSlot(read) => {
                    let slot = banks[index(read.bank())]
                        .get(read.index())
                        .ok_or(DomError::Meta(Failure::CssProcessingProtocol))?;
                    read.read(slot).map_err(error)?
                }
                CssColorParseAction::AppendSlot(write) => {
                    let at = write.index();
                    let storage = &mut banks[index(write.bank())];
                    if storage.len() != at {
                        return Err(DomError::Meta(Failure::CssProcessingProtocol));
                    }
                    scratch.reserve(storage)?;
                    let (record, receipt) = write.record();
                    storage.push(record);
                    #[cfg(test)]
                    written.pushed::<CssColorSlot<'_>>();
                    receipt.stored(&storage[at])
                }
                CssColorParseAction::UpdateSlot(write) => {
                    let at = write.index();
                    let stored = banks[index(write.bank())]
                        .get_mut(at)
                        .ok_or(DomError::Meta(Failure::CssProcessingProtocol))?;
                    let (record, receipt) = write.record();
                    *stored = record;
                    receipt.stored(stored)
                }
                CssColorParseAction::ReadFact(read) => read.read().map_err(error)?,
                CssColorParseAction::Compute(compute) => compute.compute().map_err(error)?,
            };
            scratch.release(fixed);
        };
        scratch.work(size_of::<CssColorParseOutcome<'_>>())?;
        Ok(match outcome {
            CssColorParseOutcome::Used(value) => Some(value.value()),
            CssColorParseOutcome::StandardFailure(_) => None,
        })
    })();
    // Banks/parse drop before their original formation custody.
    result
}
