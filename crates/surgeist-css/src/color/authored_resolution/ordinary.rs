//! Ordinary ownership of the exact controlled authored operation.

use super::*;
use crate::CssColor;
use crate::color::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssAuthoredColorOrdinaryLimits {
    pub algorithm_records: usize,
    pub arithmetic_records: usize,
    pub evaluation_records: usize,
    pub path_steps: usize,
    pub actions: usize,
}
pub struct CssAuthoredColorResult<'authored, 'facts> {
    root: &'authored CssColor,
    facts: &'facts CssColorFacts<'facts>,
    outcome: CssAuthoredColorOutcome,
}
impl<'authored, 'facts> CssAuthoredColorResult<'authored, 'facts> {
    pub const fn authored(&self) -> &'authored CssColor {
        self.root
    }
    pub const fn facts(&self) -> &'facts CssColorFacts<'facts> {
        self.facts
    }
    pub const fn outcome(&self) -> &CssAuthoredColorOutcome {
        &self.outcome
    }
}
#[expect(
    clippy::result_large_err,
    reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
)]
fn admit(
    resource: CssColorOrdinaryResource,
    required: usize,
    allowed: usize,
) -> Result<(), CssColorOrdinaryError> {
    if required > allowed {
        Err(CssColorOrdinaryError::Limit {
            resource,
            required,
            allowed,
        })
    } else {
        Ok(())
    }
}
fn ordinal(bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => 0,
        CssColorBank::Arithmetic => 1,
        CssColorBank::Evaluation => 2,
    }
}
fn resource(bank: CssColorBank) -> CssColorOrdinaryResource {
    match bank {
        CssColorBank::Algorithm => CssColorOrdinaryResource::AlgorithmRecords,
        CssColorBank::Arithmetic => CssColorOrdinaryResource::ArithmeticRecords,
        CssColorBank::Evaluation => CssColorOrdinaryResource::EvaluationRecords,
    }
}
fn maximum(limits: CssAuthoredColorOrdinaryLimits, bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => limits.algorithm_records,
        CssColorBank::Arithmetic => limits.arithmetic_records,
        CssColorBank::Evaluation => limits.evaluation_records,
    }
}
/// Resolve borrowed checked values without serialization, source reconstruction,
/// or an additional controller. Required and actual retained capacities are
/// checked separately for each of the three banks and the real path allocation.
#[expect(
    clippy::result_large_err,
    reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
)]
pub fn resolve_authored_color<'authored, 'facts>(
    root: &'authored CssColor,
    facts: &'facts CssColorFacts<'facts>,
    expected: CssColorFactIdentity,
    limits: CssAuthoredColorOrdinaryLimits,
) -> Result<CssAuthoredColorResult<'authored, 'facts>, CssColorOrdinaryError> {
    use CssAuthoredColorActionDescription as A;
    let mut owner = CssColorOperationOwner::new();
    let mut path = CssColorPathBuffer::new();
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut operation = Some(CssAuthoredColorOperation::resolve_authored(
        &mut owner, root, facts, expected, &mut path,
    )?);
    let mut actions = 0usize;
    let execution = (|| {
        loop {
            let op = operation
                .as_mut()
                .expect("ordinary owner retains its live operation");
            if op.action() == A::Ready {
                break;
            }
            let required = actions
                .checked_add(1)
                .ok_or(CssColorProtocolError::Overflow)?;
            admit(CssColorOrdinaryResource::Actions, required, limits.actions)?;
            actions = required;
            let mut action = op.next_action();
            let _ = action.layout()?;
            match action.description() {
                A::ReadAuthored { .. } => action.read_authored()?,
                A::ReadSite { index } => action.read_slot(
                    CssColorBank::Algorithm,
                    index,
                    banks[0]
                        .get(index)
                        .ok_or(CssColorProtocolError::WrongIndex)?,
                )?,
                A::ReadSlot { bank, index } => action.read_slot(
                    bank,
                    index,
                    banks[ordinal(bank)]
                        .get(index)
                        .ok_or(CssColorProtocolError::WrongIndex)?,
                )?,
                A::AppendSlot { bank, index } => {
                    let slots = &mut banks[ordinal(bank)];
                    if slots.len() != index {
                        return Err(CssColorProtocolError::WrongIndex.into());
                    }
                    let required = index
                        .checked_add(1)
                        .ok_or(CssColorProtocolError::Overflow)?;
                    admit(resource(bank), required, maximum(limits, bank))?;
                    let result = slots.try_reserve_exact(1);
                    // A failed allocator attempt wins over any later observation
                    // refusal. Successful overshoot is a distinct capacity limit.
                    if result.is_err() {
                        return Err(CssColorOrdinaryError::Allocation {
                            resource: resource(bank),
                        });
                    }
                    admit(resource(bank), slots.capacity(), maximum(limits, bank))?;
                    slots.push(action.record()?);
                    action.written(bank, index, &slots[index])?;
                }
                A::UpdateSlot { bank, index } => {
                    let slot = banks[ordinal(bank)]
                        .get_mut(index)
                        .ok_or(CssColorProtocolError::WrongIndex)?;
                    *slot = action.record()?;
                    action.written(bank, index, slot)?;
                }
                A::ReservePath { required_len } => {
                    admit(
                        CssColorOrdinaryResource::PathSteps,
                        required_len,
                        limits.path_steps,
                    )?;
                    let _ = action.reservation_layout()?;
                    let receipt = action.reserve_path()?;
                    let failure =
                        if receipt.status() == CssColorPathReservationStatus::AllocationFailed {
                            Some(CssColorOrdinaryError::Allocation {
                                resource: CssColorOrdinaryResource::PathSteps,
                            })
                        } else {
                            admit(
                                CssColorOrdinaryResource::PathSteps,
                                receipt.actual_capacity(),
                                limits.path_steps,
                            )
                            .err()
                        };
                    action.capacity_observed()?;
                    if let Some(error) = failure {
                        return Err(error);
                    }
                }
                A::AppendPath(_) => action.append_path()?,
                A::TruncatePath(_) => action.truncate_path()?,
                A::PrepareSite => action.prepare_site()?,
                A::RestoreSite => action.restore_site()?,
                A::Compute(_) | A::ReadFact(_) => action.compute()?,
                _ => return Err(CssColorProtocolError::Terminal.into()),
            }
        }
        Ok(())
    })();
    let result = match execution {
        Ok(()) => CssAuthoredColorOperation::take(&mut operation).map_err(Into::into),
        Err(error) => {
            operation
                .as_mut()
                .expect("failed operation remains retained")
                .abandon();
            let _ = CssAuthoredColorOperation::recover(&mut operation)
                .expect("abandoned ordinary operation is recoverable");
            Err(error)
        }
    };
    drop(operation);
    drop(banks);
    // Any outstanding receipt came from an already completed allocator action.
    // Acknowledgement cannot replace the preserved producer/consumer failure.
    if path.capacity_observation().is_some() {
        path.capacity_observed().expect("own stored observation");
    }
    path.cleanup()
        .expect("all reservation effects were observed");
    drop(path);
    let checkpoint = result?;
    let (root, facts, outcome) = checkpoint.into_parts();
    Ok(CssAuthoredColorResult {
        root,
        facts,
        outcome,
    })
}
