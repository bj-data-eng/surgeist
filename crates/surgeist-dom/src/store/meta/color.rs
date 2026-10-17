//! One whole-color CSS operation, with consumer-owned actual banks and no fallback.
use super::*;
use std::mem::size_of;
use surgeist_color::Color;
use surgeist_css::{
    CssColorActionDescription as Action, CssColorActionLayout, CssColorBank, CssColorFactIdentity,
    CssColorFacts, CssColorGrammarRequest, CssColorInputKind, CssColorOperation,
    CssColorOperationError, CssColorOperationOwner, CssColorParseOutcome, CssColorPropertyFacts,
    CssColorResolutionSite, CssColorSchemeFact, CssColorSlot, CssDeviceColorFact,
};

fn error(error: CssColorOperationError) -> DomError {
    use CssColorOperationError as E;
    DomError::Meta(match error {
        E::Protocol(value) => Failure::ThemeColorProtocol(value),
        E::Unavailable(value) => Failure::ThemeColorUnavailable(value),
        E::Unsupported(value) => Failure::ThemeColorUnsupported(value),
        E::Facts(value) => Failure::ThemeColorFacts(value),
        E::Numerical {
            error,
            site: CssColorResolutionSite::Source(input),
        } => Failure::ThemeColorNumerical { error, input },
        E::Numerical { .. } => Failure::CssProcessingProtocol,
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
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|_| DomError::AllocationFailed)?;
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
            + size_of::<Option<CssColorOperation<'_, '_>>>()
            + size_of::<[Vec<CssColorSlot<'_>>; 3]>()
            + size_of::<CssColorParseOutcome<'_, '_>>(),
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
    let mut banks: [Vec<CssColorSlot<'_>>; 3] = std::array::from_fn(|_| Vec::new());
    let mut operation = Some(
        CssColorOperation::parse_used(
            &mut owner,
            CssColorInputKind::Utf16,
            input.len(),
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity(),
        )
        .map_err(error)?,
    );
    let result = (|| {
        loop {
            draft.meta_step(cancel)?;
            let actual = operation
                .as_mut()
                .ok_or(DomError::Meta(Failure::CssProcessingProtocol))?;
            if matches!(actual.action(), Action::UsedReady | Action::StandardFailed) {
                break;
            }
            scratch.work(
                CssColorOperation::register_bytes()
                    .checked_mul(2)
                    .ok_or(DomError::RepresentationExhausted)?,
            )?;
            let fixed = CssColorOperation::action_bytes()
                + size_of::<Action>()
                + size_of::<CssColorActionLayout>();
            scratch.add(fixed)?;
            {
                let mut action = actual.next_action();
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
                match action.description() {
                    Action::ReadInput { offset } => action
                        .read_input(
                            offset,
                            *input
                                .get(offset)
                                .ok_or(DomError::Meta(Failure::CssProcessingProtocol))?,
                        )
                        .map_err(error)?,
                    Action::ReadSlot { bank, index: at } => action
                        .read_slot(
                            bank,
                            at,
                            banks[index(bank)]
                                .get(at)
                                .ok_or(DomError::Meta(Failure::CssProcessingProtocol))?,
                        )
                        .map_err(error)?,
                    Action::AppendSlot { bank, index: at } => {
                        let storage = &mut banks[index(bank)];
                        if storage.len() != at {
                            return Err(DomError::Meta(Failure::CssProcessingProtocol));
                        }
                        scratch.reserve(storage)?;
                        storage.push(action.record().map_err(error)?);
                        action.written(bank, at, &storage[at]).map_err(error)?;
                    }
                    Action::UpdateSlot { bank, index: at } => {
                        let stored = banks[index(bank)]
                            .get_mut(at)
                            .ok_or(DomError::Meta(Failure::CssProcessingProtocol))?;
                        *stored = action.record().map_err(error)?;
                        action.written(bank, at, stored).map_err(error)?;
                    }
                    Action::Compute(_) | Action::ReadFact(_) => action.compute().map_err(error)?,
                    _ => return Err(DomError::Meta(Failure::CssProcessingProtocol)),
                }
            }
            scratch.release(fixed);
        }
        scratch.work(size_of::<CssColorParseOutcome<'_, '_>>())?;
        match CssColorOperation::take(&mut operation)
            .map_err(|e| DomError::Meta(Failure::ThemeColorProtocol(e)))?
        {
            CssColorParseOutcome::Used(value) => Ok(Some(value.value())),
            CssColorParseOutcome::StandardFailure(_) => Ok(None),
        }
    })();
    if result.is_err()
        && let Some(operation) = operation.as_mut()
    {
        operation.abandon();
    }
    // Banks/operation drop before their original formation custody.
    result
}
