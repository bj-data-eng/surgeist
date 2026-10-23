//! Explicit ordinary ownership for the same bounded pure operations.
//!
//! These convenience loops own local vectors. Their ceilings cover logical
//! records/actions; they make no physical-allocator or HTML ledger claim.
//! The controlled HTML adapter services the public actions directly instead.

use super::*;

/// The actual borrowed original source; UTF16 is never converted lossily.
#[derive(Clone, Copy, Debug)]
pub enum CssColorInput<'source> {
    Utf8(&'source [u8]),
    Utf16(&'source [u16]),
}
impl CssColorInput<'_> {
    #[must_use]
    pub const fn len(self) -> usize {
        match self {
            Self::Utf8(input) => input.len(),
            Self::Utf16(input) => input.len(),
        }
    }
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }
    #[must_use]
    pub const fn kind(self) -> CssColorInputKind {
        match self {
            Self::Utf8(_) => CssColorInputKind::Utf8,
            Self::Utf16(_) => CssColorInputKind::Utf16,
        }
    }
    fn unit(self, index: usize) -> Option<u16> {
        match self {
            Self::Utf8(input) => input.get(index).copied().map(u16::from),
            Self::Utf16(input) => input.get(index).copied(),
        }
    }
}
/// Explicit ordinary logical ceilings. No shared/global controller is created.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorOrdinaryLimits {
    pub input_units: usize,
    pub algorithm_records: usize,
    pub arithmetic_records: usize,
    pub evaluation_records: usize,
    pub actions: usize,
}
/// The actual exhausted ordinary resource, separate from CSS invalidity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorOrdinaryResource {
    PathSteps,
    InputUnits,
    AlgorithmRecords,
    ArithmeticRecords,
    EvaluationRecords,
    Actions,
}
#[derive(Debug, Eq, PartialEq)]
pub enum CssColorOrdinaryError {
    Operation(CssColorOperationError),
    Limit {
        resource: CssColorOrdinaryResource,
        required: usize,
        allowed: usize,
    },
    Allocation {
        resource: CssColorOrdinaryResource,
    },
}
impl std::fmt::Display for CssColorOrdinaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Operation(error) => std::fmt::Display::fmt(error, f),
            Self::Limit {
                resource,
                required,
                allowed,
            } => write!(
                f,
                "color {resource:?} requires {required}, exceeding {allowed}"
            ),
            Self::Allocation { resource } => write!(f, "color {resource:?} allocation failed"),
        }
    }
}
impl std::error::Error for CssColorOrdinaryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Operation(error) => Some(error),
            _ => None,
        }
    }
}
impl From<CssColorOperationError> for CssColorOrdinaryError {
    fn from(error: CssColorOperationError) -> Self {
        Self::Operation(error)
    }
}
impl From<CssColorProtocolError> for CssColorOrdinaryError {
    fn from(error: CssColorProtocolError) -> Self {
        CssColorOperationError::Protocol(error).into()
    }
}
/// A fixed source-qualified outcome after all ordinary scratch is retired.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CssColorOrdinaryOutcome {
    Used(Color),
    StandardFailure(CssColorStandardFailure),
}
/// Borrows the actual original source and fact envelope; no graph/anchor/scratch
/// escapes. A failed result is not a native color or an inferred fallback.
pub struct CssColorOrdinaryResult<'source, 'facts> {
    input: CssColorInput<'source>,
    facts: &'facts CssColorFacts<'facts>,
    source: CssColorSourceDescriptor,
    outcome: CssColorOrdinaryOutcome,
}
impl<'source, 'facts> CssColorOrdinaryResult<'source, 'facts> {
    #[must_use]
    pub const fn input(&self) -> CssColorInput<'source> {
        self.input
    }
    #[must_use]
    pub const fn facts(&self) -> &'facts CssColorFacts<'facts> {
        self.facts
    }
    #[must_use]
    pub const fn source(&self) -> CssColorSourceDescriptor {
        self.source
    }
    #[must_use]
    pub const fn outcome(&self) -> CssColorOrdinaryOutcome {
        self.outcome
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
fn allowed(limits: CssColorOrdinaryLimits, bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => limits.algorithm_records,
        CssColorBank::Arithmetic => limits.arithmetic_records,
        CssColorBank::Evaluation => limits.evaluation_records,
    }
}
fn admit(
    resource: CssColorOrdinaryResource,
    required: usize,
    allowed: usize,
) -> Result<(), CssColorOrdinaryError> {
    if required <= allowed {
        Ok(())
    } else {
        Err(CssColorOrdinaryError::Limit {
            resource,
            required,
            allowed,
        })
    }
}
fn action_count(count: &mut usize, maximum: usize) -> Result<(), CssColorOrdinaryError> {
    let required = count
        .checked_add(1)
        .ok_or(CssColorProtocolError::Overflow)?;
    admit(CssColorOrdinaryResource::Actions, required, maximum)?;
    *count = required;
    Ok(())
}
/// Parse to a fixed used value without planning, admitting or creating output.
/// Every finite action uses the canonical operation and the original source.
pub fn parse_used_color<'source, 'facts>(
    input: CssColorInput<'source>,
    request: CssColorGrammarRequest,
    facts: &'facts CssColorFacts<'facts>,
    expected: CssColorFactIdentity,
    limits: CssColorOrdinaryLimits,
) -> Result<CssColorOrdinaryResult<'source, 'facts>, CssColorOrdinaryError> {
    admit(
        CssColorOrdinaryResource::InputUnits,
        input.len(),
        limits.input_units,
    )?;
    let mut owner = CssColorOperationOwner::new();
    let mut operation = CssColorOperation::parse_used(
        &mut owner,
        input.kind(),
        input.len(),
        request,
        facts,
        expected,
    )?;
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let mut actions = 0;
    let execution = (|| {
        loop {
            match operation.action() {
                CssColorActionDescription::UsedReady
                | CssColorActionDescription::StandardFailed => break,
                _ => {}
            }
            action_count(&mut actions, limits.actions)?;
            let mut action = operation.next_action();
            match action.description() {
                CssColorActionDescription::ReadInput { offset } => action.read_input(
                    offset,
                    input
                        .unit(offset)
                        .ok_or(CssColorProtocolError::WrongIndex)?,
                )?,
                CssColorActionDescription::ReadSlot { bank, index } => action.read_slot(
                    bank,
                    index,
                    banks[ordinal(bank)]
                        .get(index)
                        .ok_or(CssColorProtocolError::WrongIndex)?,
                )?,
                CssColorActionDescription::AppendSlot { bank, index } => {
                    if index != banks[ordinal(bank)].len() {
                        return Err(CssColorProtocolError::WrongIndex.into());
                    }
                    // Logical/write/allocation admission precedes generation and
                    // push; acknowledgement follows actual ownership transfer.
                    let required = index
                        .checked_add(1)
                        .ok_or(CssColorProtocolError::Overflow)?;
                    admit(resource(bank), required, allowed(limits, bank))?;
                    banks[ordinal(bank)].try_reserve(1).map_err(|_| {
                        CssColorOrdinaryError::Allocation {
                            resource: resource(bank),
                        }
                    })?;
                    let record = action.record()?;
                    banks[ordinal(bank)].push(record);
                    action.written(bank, index, &banks[ordinal(bank)][index])?;
                }
                CssColorActionDescription::UpdateSlot { bank, index } => {
                    let old = banks[ordinal(bank)]
                        .get_mut(index)
                        .ok_or(CssColorProtocolError::WrongIndex)?;
                    *old = action.record()?;
                    action.written(bank, index, old)?;
                }
                CssColorActionDescription::Compute(_) | CssColorActionDescription::ReadFact(_) => {
                    action.compute()?
                }
                _ => return Err(CssColorProtocolError::WrongAction.into()),
            }
        }
        Ok(())
    })();
    if let Err(error) = execution {
        operation.abandon();
        banks.iter_mut().for_each(Vec::clear);
        return Err(error);
    }
    let (source, outcome) = match CssColorOperation::take(&mut Some(operation))? {
        CssColorParseOutcome::Used(checkpoint) => (
            checkpoint.source(),
            CssColorOrdinaryOutcome::Used(checkpoint.value()),
        ),
        CssColorParseOutcome::StandardFailure(checkpoint) => {
            let failure = checkpoint.failure();
            (
                CssColorSourceDescriptor {
                    kind: input.kind(),
                    length: input.len(),
                    request,
                    consumed: CssColorInputRange {
                        start: 0,
                        end: input.len(),
                    },
                    facts: expected,
                },
                CssColorOrdinaryOutcome::StandardFailure(failure),
            )
        }
    };
    banks.iter_mut().for_each(Vec::clear);
    drop(banks);
    Ok(CssColorOrdinaryResult {
        input,
        facts,
        source,
        outcome,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn facts() -> CssColorFacts<'static> {
        CssColorFacts::new(
            CssColorFactIdentity::new(2, 3),
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &[],
            &[],
            CssDeviceColorFact::Unknown,
        )
    }
    fn limits() -> CssColorOrdinaryLimits {
        CssColorOrdinaryLimits {
            input_units: 128,
            algorithm_records: 4096,
            arithmetic_records: 512,
            evaluation_records: 512,
            actions: 1_000_000,
        }
    }
    #[test]
    fn parse_only_preserves_actual_input_and_facts() {
        let facts = facts();
        let input = b"rgb(1 0 0)";
        let result = parse_used_color(
            CssColorInput::Utf8(input),
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity,
            limits(),
        )
        .unwrap();
        let CssColorOrdinaryOutcome::Used(value) = result.outcome() else {
            panic!("typed used value")
        };
        assert_eq!(value.channels(), [Some(1.0 / 255.0), Some(0.0), Some(0.0)]);
        assert_eq!(value.alpha(), Some(1.0));
        let CssColorInput::Utf8(retained) = result.input() else {
            panic!("same original encoding")
        };
        assert!(std::ptr::eq(retained, input.as_slice()));
        assert!(std::ptr::eq(result.facts(), &facts));
        assert_eq!(
            result.source().consumed,
            CssColorInputRange {
                start: 0,
                end: input.len()
            }
        );
        let input = [114, 101, 100];
        let result = parse_used_color(
            CssColorInput::Utf16(&input),
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity,
            limits(),
        )
        .unwrap();
        assert_eq!(result.source().kind, CssColorInputKind::Utf16);
        assert_eq!(
            result.outcome(),
            CssColorOrdinaryOutcome::Used(Color::from_srgba8([255, 0, 0, 255]))
        );
    }
    #[test]
    fn logical_input_scratch_and_action_denials_remain_distinct_from_invalid_css() {
        let facts = facts();
        let mut limits = limits();
        limits.input_units = 2;
        assert!(matches!(
            parse_used_color(
                CssColorInput::Utf8(b"red"),
                CssColorGrammarRequest::WholeColor,
                &facts,
                facts.identity,
                limits
            ),
            Err(CssColorOrdinaryError::Limit {
                resource: CssColorOrdinaryResource::InputUnits,
                required: 3,
                allowed: 2
            })
        ));
        limits.input_units = 3;
        limits.algorithm_records = 0;
        assert!(matches!(
            parse_used_color(
                CssColorInput::Utf8(b"red"),
                CssColorGrammarRequest::WholeColor,
                &facts,
                facts.identity,
                limits
            ),
            Err(CssColorOrdinaryError::Limit {
                resource: CssColorOrdinaryResource::AlgorithmRecords,
                required: 1,
                allowed: 0
            })
        ));
        limits.algorithm_records = 32;
        limits.actions = 0;
        assert!(matches!(
            parse_used_color(
                CssColorInput::Utf8(b"red"),
                CssColorGrammarRequest::WholeColor,
                &facts,
                facts.identity,
                limits
            ),
            Err(CssColorOrdinaryError::Limit {
                resource: CssColorOrdinaryResource::Actions,
                required: 1,
                allowed: 0
            })
        ));
        limits.actions = 1000;
        let result = parse_used_color(
            CssColorInput::Utf8(b"bad"),
            CssColorGrammarRequest::WholeColor,
            &facts,
            facts.identity,
            limits,
        )
        .unwrap();
        assert!(matches!(
            result.outcome(),
            CssColorOrdinaryOutcome::StandardFailure(CssColorStandardFailure {
                kind: CssColorStandardFailureKind::ColorGrammar,
                ..
            })
        ));
    }
}
