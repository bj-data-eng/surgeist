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
#[non_exhaustive]
pub enum CssColorOrdinaryResource {
    /// Steps of the authored component path.
    PathSteps,
    /// Original input units.
    InputUnits,
    /// Records of the algorithm bank.
    AlgorithmRecords,
    /// Records of the arithmetic bank.
    ArithmeticRecords,
    /// Records of the evaluation bank.
    EvaluationRecords,
    /// Serviced parse actions.
    Actions,
}
impl CssColorOrdinaryResource {
    const fn description(self) -> &'static str {
        match self {
            Self::PathSteps => "path steps",
            Self::InputUnits => "input units",
            Self::AlgorithmRecords => "algorithm records",
            Self::ArithmeticRecords => "arithmetic records",
            Self::EvaluationRecords => "evaluation records",
            Self::Actions => "actions",
        }
    }
}
/// An ordinary colour resolution that reached no outcome: a resource ceiling or
/// allocation refused it, or the colour parse failed semantically.
///
/// Exactly one accessor returns a value; [`std::error::Error::source`] is the
/// colour error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorResolveError {
    kind: ResolveErrorKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum ResolveErrorKind {
    Color(CssColorError),
    Limit {
        resource: CssColorOrdinaryResource,
        required: usize,
        allowed: usize,
    },
    Allocation {
        resource: CssColorOrdinaryResource,
    },
}
impl CssColorResolveError {
    pub(crate) const fn limit(
        resource: CssColorOrdinaryResource,
        required: usize,
        allowed: usize,
    ) -> Self {
        Self {
            kind: ResolveErrorKind::Limit {
                resource,
                required,
                allowed,
            },
        }
    }
    pub(crate) const fn allocation(resource: CssColorOrdinaryResource) -> Self {
        Self {
            kind: ResolveErrorKind::Allocation { resource },
        }
    }
    /// Returns the exhausted resource with its allowed and required amounts.
    #[must_use]
    pub const fn exhausted_limit(&self) -> Option<(CssColorOrdinaryResource, usize, usize)> {
        match self.kind {
            ResolveErrorKind::Limit {
                resource,
                required,
                allowed,
            } => Some((resource, allowed, required)),
            _ => None,
        }
    }
    /// Returns the resource whose storage could not be allocated.
    #[must_use]
    pub const fn failed_allocation(&self) -> Option<CssColorOrdinaryResource> {
        match self.kind {
            ResolveErrorKind::Allocation { resource } => Some(resource),
            _ => None,
        }
    }
    /// Returns the semantic failure of the colour parse.
    #[must_use]
    pub const fn color_error(&self) -> Option<&CssColorError> {
        match &self.kind {
            ResolveErrorKind::Color(error) => Some(error),
            _ => None,
        }
    }
}
impl std::fmt::Display for CssColorResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ResolveErrorKind::Color(error) => std::fmt::Display::fmt(error, f),
            ResolveErrorKind::Limit {
                resource,
                required,
                allowed,
            } => write!(
                f,
                "color resolution needs {required} {}, exceeding the limit of {allowed}",
                resource.description()
            ),
            ResolveErrorKind::Allocation { resource } => write!(
                f,
                "color resolution could not allocate {}",
                resource.description()
            ),
        }
    }
}
impl std::error::Error for CssColorResolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.color_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}
impl From<CssColorError> for CssColorResolveError {
    fn from(error: CssColorError) -> Self {
        Self {
            kind: ResolveErrorKind::Color(error),
        }
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
) -> Result<(), CssColorResolveError> {
    if required <= allowed {
        Ok(())
    } else {
        Err(CssColorResolveError::limit(resource, required, allowed))
    }
}
fn action_count(count: &mut usize, maximum: usize) -> Result<(), CssColorResolveError> {
    let required = count
        .checked_add(1)
        .expect("an ordinary parse admits at most usize::MAX actions");
    admit(CssColorOrdinaryResource::Actions, required, maximum)?;
    *count = required;
    Ok(())
}
/// Services every action through the ordinary local banks. A refusal drops
/// the pending action, which abandons the parse.
fn drive<'owner, 'facts>(
    mut parse: CssColorParse<'owner, 'facts>,
    input: CssColorInput<'_>,
    banks: &mut [Vec<CssColorSlot<'owner>>; 3],
    limits: CssColorOrdinaryLimits,
) -> Result<CssColorParseOutcome<'facts>, CssColorResolveError> {
    let mut actions = 0;
    loop {
        let action = match parse.step() {
            CssColorParseStep::Complete(outcome) => return Ok(outcome),
            CssColorParseStep::Pending(action) => action,
        };
        action_count(&mut actions, limits.actions)?;
        parse = match action {
            CssColorParseAction::ReadInput(read) => {
                let unit = input
                    .unit(read.offset())
                    .expect("a parse reads only offsets below its declared length");
                read.read(unit)?
            }
            CssColorParseAction::ReadSlot(read) => {
                let slot = banks[ordinal(read.bank())]
                    .get(read.index())
                    .expect("a parse reads only records it stored");
                read.read(slot)?
            }
            CssColorParseAction::AppendSlot(write) => {
                let (bank, index) = (write.bank(), write.index());
                let target = &mut banks[ordinal(bank)];
                assert_eq!(index, target.len(), "a parse appends at its bank's end");
                // Logical/write/allocation admission precedes generation and
                // push; acknowledgement follows actual ownership transfer.
                let required = index
                    .checked_add(1)
                    .expect("an appended index is below usize::MAX");
                admit(resource(bank), required, allowed(limits, bank))?;
                target
                    .try_reserve(1)
                    .map_err(|_| CssColorResolveError::allocation(resource(bank)))?;
                let (record, receipt) = write.record();
                target.push(record);
                receipt.stored(&target[index])
            }
            CssColorParseAction::UpdateSlot(write) => {
                let (bank, index) = (write.bank(), write.index());
                let old = banks[ordinal(bank)]
                    .get_mut(index)
                    .expect("a parse replaces only records it stored");
                let (record, receipt) = write.record();
                *old = record;
                receipt.stored(old)
            }
            CssColorParseAction::ReadFact(read) => read.read()?,
            CssColorParseAction::Compute(compute) => compute.compute()?,
        };
    }
}
/// Parse to a fixed used value without planning, admitting or creating output.
/// Every finite action uses the canonical operation and the original source.
pub fn parse_used_color<'source, 'facts>(
    input: CssColorInput<'source>,
    request: CssColorGrammarRequest,
    facts: &'facts CssColorFacts<'facts>,
    expected: CssColorFactIdentity,
    limits: CssColorOrdinaryLimits,
) -> Result<CssColorOrdinaryResult<'source, 'facts>, CssColorResolveError> {
    admit(
        CssColorOrdinaryResource::InputUnits,
        input.len(),
        limits.input_units,
    )?;
    let mut owner = CssColorOperationOwner::new();
    let parse = CssColorParse::start(
        &mut owner,
        input.kind(),
        input.len(),
        request,
        facts,
        expected,
    )?;
    let mut banks: [Vec<_>; 3] = std::array::from_fn(|_| Vec::new());
    let parsed = match drive(parse, input, &mut banks, limits) {
        Ok(parsed) => parsed,
        Err(error) => {
            banks.iter_mut().for_each(Vec::clear);
            return Err(error);
        }
    };
    let (source, outcome) = match parsed {
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
            Err(ref error)
                if error.exhausted_limit() == Some((CssColorOrdinaryResource::InputUnits, 2, 3))
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
            Err(ref error) if error.exhausted_limit()
                == Some((CssColorOrdinaryResource::AlgorithmRecords, 0, 1))
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
            Err(ref error)
                if error.exhausted_limit() == Some((CssColorOrdinaryResource::Actions, 0, 1))
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
