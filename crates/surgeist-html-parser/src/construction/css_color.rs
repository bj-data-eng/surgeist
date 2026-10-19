//! Finite whole-color CSS action servicing shared by parser-owned consumers.
//! Consumers retain their own source policy, fallback, transformation and output.
use super::*;
use surgeist_css::{
    CssColorActionDescription as Action, CssColorActionLayout, CssColorBank, CssColorFacts,
    CssColorGrammarRequest, CssColorInputKind, CssColorOperation, CssColorOperationError,
    CssColorOperationOwner, CssColorParseOutcome, CssColorSlot,
};

pub(super) enum Failure {
    Session(SessionError),
    Processing(crate::parser::control::HtmlColorProcessingError),
}
impl From<SessionError> for Failure {
    fn from(value: SessionError) -> Self {
        Self::Session(value)
    }
}
impl From<ParseStop> for Failure {
    fn from(value: ParseStop) -> Self {
        Self::Session(value.into())
    }
}
/// This whole-color request resolves only source input. Broader CSS authored
/// sites are not part of this adapter and retain their owning invariant error.
pub(super) fn source_processing(c: &mut ParseControl, error: CssColorOperationError) -> Failure {
    match crate::parser::control::HtmlColorProcessingError::from_source(error) {
        Ok(error) => Failure::Processing(error),
        Err(error) => Failure::Session(c.fail_invariant(error).into()),
    }
}
fn integrity(c: &mut ParseControl) -> SessionError {
    c.fail_invariant(crate::parser::control::Invariant::InputCursorState)
        .into()
}
fn bank_index(bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => 0,
        CssColorBank::Arithmetic => 1,
        CssColorBank::Evaluation => 2,
    }
}
pub(super) type Banks<'a> = [Option<OwnedVec<CssColorSlot<'a>>>; 3];
fn banks<'a>() -> Result<Banks<'a>, SessionError> {
    Ok([
        Some(OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?),
        Some(OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?),
        Some(OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?),
    ])
}
fn bank<'a, 'b>(
    banks: &'b mut Banks<'a>,
    kind: CssColorBank,
    c: &mut ParseControl,
) -> Result<&'b mut OwnedVec<CssColorSlot<'a>>, SessionError> {
    banks[bank_index(kind)].as_mut().ok_or_else(|| integrity(c))
}
/// No alias/controller escapes. Each flat bank was initialized by this owner
/// and its admitted lengths/capacity exactly match the ledger. Release performs
/// checked cleanup without polling/useful work, including after sticky stop.
/// All three releases are attempted; a cleanup invariant remains an error.
fn release_banks(banks: &mut Banks<'_>, c: &mut ParseControl) -> Result<(), SessionError> {
    let mut failure = None;
    for bank in banks {
        if let Some(bank) = bank.take()
            && let Err(error) = bank.release(c)
        {
            failure.get_or_insert(SessionError::from(error));
        }
    }
    failure.map_or(Ok(()), Err)
}
pub(super) fn frame(
    c: &mut ParseControl,
    register: usize,
    guard: usize,
) -> Result<(), SessionError> {
    let Some(layout) = register.checked_add(guard) else {
        return c
            .representation(HtmlParseResource::AtomicLayoutBytes)
            .map_err(Into::into);
    };
    c.admit_atomic_layout(layout)?;
    c.charge(WorkCategory::RecordVisit, 1)?;
    c.charge(WorkCategory::InspectByte, register as u64)?;
    c.charge(WorkCategory::CopyByte, layout as u64)?;
    c.charge(WorkCategory::RecordWrite, 1)?;
    c.charge(WorkCategory::StateStep, 1)?;
    Ok(())
}
pub(super) fn action_layout(
    c: &mut ParseControl,
    layout: CssColorActionLayout,
) -> Result<(), SessionError> {
    c.charge(WorkCategory::InspectByte, layout.input_unit_bytes as u64)?;
    c.charge(WorkCategory::InspectByte, layout.slot_read_bytes as u64)?;
    c.charge(WorkCategory::CopyByte, layout.slot_write_bytes as u64)?;
    c.charge(
        WorkCategory::InspectByte,
        layout.numerical_read_bytes as u64,
    )?;
    c.charge(WorkCategory::CopyByte, layout.numerical_write_bytes as u64)?;
    if layout.slot_write_bytes != 0 || layout.numerical_write_bytes != 0 {
        c.charge(WorkCategory::RecordWrite, 1)?;
    }
    // At most three actual fixed fact records; no array/variable payload scan.
    for bytes in layout.fact_records {
        if bytes != 0 {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(WorkCategory::InspectByte, bytes as u64)?;
        }
    }
    c.charge(WorkCategory::CopyByte, layout.output_unit_bytes as u64)?;
    c.checkpoint()?;
    Ok(())
}
pub(super) fn parse<'owner, 'facts, E: From<SessionError> + From<ParseStop>>(
    c: &mut ParseControl,
    input: &[u16],
    owner: &'owner mut CssColorOperationOwner,
    facts: &'facts CssColorFacts<'facts>,
    on_processing: fn(&mut ParseControl, CssColorOperationError) -> E,
    consumer_layout: usize,
    #[cfg(test)] mut slot_commit: impl FnMut(),
) -> Result<CssColorParseOutcome<'owner, 'facts>, E> {
    let layout = CssColorOperation::initialization_layout();
    frame(
        c,
        layout.register_bytes,
        layout.owner_bytes
            + layout.fact_identity_bytes
            + mem::size_of::<CssColorFacts<'_>>()
            + mem::size_of::<Banks<'_>>()
            + mem::size_of_val(&on_processing)
            + consumer_layout,
    )?;
    let mut operation = Some(
        CssColorOperation::parse_used(
            owner,
            CssColorInputKind::Utf16,
            input.len(),
            CssColorGrammarRequest::WholeColor,
            facts,
            facts.identity(),
        )
        .map_err(|error| on_processing(c, error))?,
    );
    let mut banks = banks()?;
    let result = (|| {
        loop {
            frame(
                c,
                CssColorOperation::register_bytes(),
                CssColorOperation::action_bytes(),
            )?;
            let operation = operation.as_mut().ok_or_else(|| integrity(c))?;
            if matches!(
                operation.action(),
                Action::UsedReady | Action::StandardFailed
            ) {
                break;
            }
            let mut action = operation.next_action();
            action_layout(c, action.layout())?;
            match action.description() {
                Action::ReadInput { offset } => {
                    // All inspection/work/poll admission above precedes the
                    // actual original unit dereference, including UTF16.
                    let unit = *input.get(offset).ok_or_else(|| integrity(c))?;
                    action
                        .read_input(offset, unit)
                        .map_err(|error| on_processing(c, error))?;
                }
                Action::ReadSlot { bank: kind, index } => {
                    let bank = bank(&mut banks, kind, c)?;
                    bank.require_owner(c)?;
                    let slot = bank.as_slice().get(index).ok_or_else(|| integrity(c))?;
                    action
                        .read_slot(kind, index, slot)
                        .map_err(|error| on_processing(c, error))?;
                }
                Action::AppendSlot { bank: kind, index } => {
                    let bank = bank(&mut banks, kind, c)?;
                    if bank.len() != index {
                        return Err(integrity(c).into());
                    }
                    bank.reserve_geometric(c, 1)?;
                    c.check_acquire_many(&[(HtmlParseResource::AuxiliaryEntries, 1)])?;
                    let record = action.record().map_err(|error| on_processing(c, error))?;
                    if let Err((error, _uncommitted)) = bank.try_push_recover(c, record) {
                        return Err(error.into());
                    }
                    let slot = bank.as_slice().get(index).ok_or_else(|| integrity(c))?;
                    action
                        .written(kind, index, slot)
                        .map_err(|error| on_processing(c, error))?;
                    #[cfg(test)]
                    slot_commit();
                }
                Action::UpdateSlot { bank: kind, index } => {
                    let bank = bank(&mut banks, kind, c)?;
                    let slot = bank.get_mut(index, c)?.ok_or_else(|| integrity(c))?;
                    *slot = action.record().map_err(|error| on_processing(c, error))?;
                    action
                        .written(kind, index, slot)
                        .map_err(|error| on_processing(c, error))?;
                }
                Action::Compute(_) | Action::ReadFact(_) => {
                    action.compute().map_err(|error| on_processing(c, error))?
                }
                _ => return Err(integrity(c).into()),
            }
        }
        frame(
            c,
            CssColorOperation::register_bytes(),
            mem::size_of::<CssColorParseOutcome<'_, '_>>(),
        )?;
        CssColorOperation::take(&mut operation).map_err(|error| on_processing(c, error.into()))
    })();
    if result.is_err()
        && let Some(operation) = operation.as_mut()
    {
        operation.abandon();
    }
    let disposal = release_banks(&mut banks, c);
    disposal?;
    result
}
