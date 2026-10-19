//! Source-ordered Range sanitation decisions, HTML877d §4.10.5.1.13.
//! Arithmetic, strings, storage, provenance and admission remain caller-owned.
use crate::{
    HtmlFloatingPointNumber, HtmlInputNumericBoundOrigin, HtmlInputNumericConstraints,
    HtmlInputNumericContent, HtmlInputNumericStep, HtmlInputType, HtmlNonFiniteNumberError,
    select_input_numeric_constraints,
};

/// A completed finite Range calculation value, preserving actual signed-zero bits.
///
/// Parsing never produces −0: the rules for parsing floating-point number values
/// convert into the finite binary64 values except −0, for both ordinary
/// `HtmlFloatingPointNumber` values and Range's bounded ingress. Range's lattice
/// operations retain the signed-zero bits their arithmetic produces until an
/// actual replacement requests best-number serialization. This type carries only
/// that algorithm value; it does not parse or calculate numbers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputRangeNumber(u64);

impl HtmlInputRangeNumber {
    /// Returns the original finite binary64 value, including its zero sign.
    pub fn value(self) -> f64 {
        f64::from_bits(self.0)
    }
}

impl TryFrom<f64> for HtmlInputRangeNumber {
    type Error = HtmlNonFiniteNumberError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        // Reuse the ordinary owner's finite admission; retain the actual bits
        // rather than its deliberately normalized-zero representation.
        HtmlFloatingPointNumber::try_from(value)?;
        Ok(Self(value.to_bits()))
    }
}

impl From<HtmlFloatingPointNumber> for HtmlInputRangeNumber {
    fn from(value: HtmlFloatingPointNumber) -> Self {
        Self(value.value().to_bits())
    }
}

/// Completed Author-grammar and finite-number ingress observations.
/// Capability/admission/cancellation failure is not a reply to this operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HtmlInputRangeValue {
    InvalidGrammar,
    Finite(HtmlInputRangeNumber),
    /// Grammar succeeded (HTML877d3216–3242) but standard conversion returned
    /// its overflow error (3325–3329; Range37061–37065 delegates to it).
    /// Range36939–36940 defaults only invalid grammar. The under/overflow
    /// predicates38737–38749 and mismatch38839–38843 require a number result,
    /// so this observation proves none of those conditions. It is not an
    /// unavailable computation, resource refusal, infinity or opaque default.
    ConversionFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputRangeReplacement {
    InvalidDefaultMinimum,
    InvalidDefaultMidpoint,
    Underflow,
    Overflow,
    StepCorrection,
}

/// Fulfill the pending action before replying. Numeric replies must be obtained
/// from the corresponding existing HTML bounded numeric operation, not guessed
/// from platform floating arithmetic or supplied by a generic result provider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HtmlInputRangeSanitizationAction {
    Midpoint {
        minimum: HtmlInputRangeNumber,
        maximum: HtmlInputRangeNumber,
    },
    /// Serialize through BestFiniteNumberCursor and install the actual string.
    Replace {
        value: HtmlInputRangeNumber,
        branch: HtmlInputRangeReplacement,
    },
    /// Observe exact whole `any`, or Recovery grammar + finite ingress.
    /// This late read retains the selected controlled consumer order; HTML's
    /// source defines dependencies, not a universal eager evaluation order.
    ReadStep,
    /// Read the actual content value only when min supplied no finite base.
    ReadValueContent,
    /// RangeLatticeCursor selects actual admissible finite candidates, ties up.
    Lattice {
        current: HtmlInputRangeNumber,
        minimum: HtmlInputRangeNumber,
        maximum: Option<HtmlInputRangeNumber>,
        base: HtmlInputRangeNumber,
        step: HtmlInputRangeNumber,
    },
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HtmlInputRangeSanitizationReply {
    Midpoint(HtmlInputRangeNumber),
    Applied,
    Step(HtmlInputNumericStep),
    ValueContent(HtmlInputNumericContent),
    /// None is a completed proof of no admissible candidate, not unavailable.
    Lattice(Option<HtmlInputRangeNumber>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputRangeSanitizationReplyError;
impl std::fmt::Display for HtmlInputRangeSanitizationReplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Range sanitation reply does not fulfill the pending action")
    }
}
impl std::error::Error for HtmlInputRangeSanitizationReplyError {}

#[derive(Clone, Copy, Debug)]
enum Continue {
    Clamp,
    Step,
    Complete,
}
#[derive(Clone, Copy, Debug)]
enum Phase {
    Midpoint,
    Replace {
        value: HtmlInputRangeNumber,
        branch: HtmlInputRangeReplacement,
        then: Continue,
    },
    Step,
    ValueContent(HtmlInputNumericStep),
    Lattice,
    Complete,
}

/// Fixed acyclic language policy, with no source reads, loops, allocations,
/// numeric scratch, source identity or mutation owner. Admit the fixed receiver,
/// action/reply copies and reached scalar step before each call. The consumer
/// first completes current Author grammar/finite ingress, then min and max
/// Recovery ingress, in that order. Missing observations must stop there; they
/// cannot be represented as ordinary invalid input in this cursor.
#[derive(Clone, Copy, Debug)]
pub struct HtmlInputRangeSanitizationCursor {
    constraints: HtmlInputNumericConstraints,
    current: Option<HtmlInputRangeNumber>,
    replaced: bool,
    phase: Phase,
}
impl HtmlInputRangeSanitizationCursor {
    pub fn new(
        value: HtmlInputRangeValue,
        min: HtmlInputNumericContent,
        max: HtmlInputNumericContent,
    ) -> Self {
        let constraints = select_input_numeric_constraints(
            min,
            max,
            HtmlInputNumericStep::Absent,
            HtmlInputNumericContent::Absent,
            HtmlInputType::Range,
        )
        .expect("literal Range always selects numeric constraints");
        let mut cursor = Self {
            constraints,
            current: None,
            replaced: false,
            phase: Phase::Complete,
        };
        cursor.phase = match value {
            HtmlInputRangeValue::Finite(value) => {
                cursor.current = Some(value);
                cursor.clamp(value)
            }
            HtmlInputRangeValue::ConversionFailed => Phase::Step,
            HtmlInputRangeValue::InvalidGrammar
                if cursor.maximum().value() < cursor.minimum().value() =>
            {
                Phase::Replace {
                    value: cursor.minimum(),
                    branch: HtmlInputRangeReplacement::InvalidDefaultMinimum,
                    then: Continue::Clamp,
                }
            }
            HtmlInputRangeValue::InvalidGrammar => Phase::Midpoint,
        };
        cursor
    }
    fn minimum(&self) -> HtmlInputRangeNumber {
        self.constraints
            .minimum()
            .expect("Range has a minimum")
            .into()
    }
    fn maximum(&self) -> HtmlInputRangeNumber {
        self.constraints
            .maximum()
            .expect("Range has a maximum")
            .into()
    }
    fn effective_maximum(&self) -> Option<HtmlInputRangeNumber> {
        (self.maximum().value() >= self.minimum().value()).then(|| self.maximum())
    }
    fn clamp(&self, value: HtmlInputRangeNumber) -> Phase {
        if value.value() < self.minimum().value() {
            Phase::Replace {
                value: self.minimum(),
                branch: HtmlInputRangeReplacement::Underflow,
                then: Continue::Step,
            }
        } else if self
            .effective_maximum()
            .is_some_and(|max| value.value() > max.value())
        {
            Phase::Replace {
                value: self.maximum(),
                branch: HtmlInputRangeReplacement::Overflow,
                then: Continue::Step,
            }
        } else {
            Phase::Step
        }
    }
    fn begin_lattice(&self) -> Phase {
        if self.current.is_some() && self.constraints.allowed_step().is_some() {
            Phase::Lattice
        } else {
            Phase::Complete
        }
    }
    /// Actual bound origins are available before late step/base observations.
    pub fn bound_origins(&self) -> (HtmlInputNumericBoundOrigin, HtmlInputNumericBoundOrigin) {
        (
            self.constraints.minimum_origin(),
            self.constraints.maximum_origin(),
        )
    }
    /// Full parameters are observable only after actual step/base observations.
    /// Early private bound selection does not certify missing step/value facts.
    pub fn constraints(&self) -> Option<HtmlInputNumericConstraints> {
        matches!(
            self.phase,
            Phase::Lattice
                | Phase::Replace {
                    branch: HtmlInputRangeReplacement::StepCorrection,
                    ..
                }
                | Phase::Complete
        )
        .then_some(self.constraints)
    }
    /// Whether an actual replacement was acknowledged, not raw input inequality.
    pub fn replaced(&self) -> bool {
        self.replaced
    }
    pub fn action(&self) -> HtmlInputRangeSanitizationAction {
        use HtmlInputRangeSanitizationAction as A;
        match self.phase {
            Phase::Midpoint => A::Midpoint {
                minimum: self.minimum(),
                maximum: self.maximum(),
            },
            Phase::Replace { value, branch, .. } => A::Replace { value, branch },
            Phase::Step => A::ReadStep,
            Phase::ValueContent(_) => A::ReadValueContent,
            Phase::Lattice => A::Lattice {
                current: self.current.expect("finite current selected lattice"),
                minimum: self.minimum(),
                maximum: self.effective_maximum(),
                base: self.constraints.step_base().into(),
                step: self
                    .constraints
                    .allowed_step()
                    .expect("allowed step selected lattice")
                    .into(),
            },
            Phase::Complete => A::Complete,
        }
    }
    /// Rejected replies, including replies after Complete, preserve the cursor.
    pub fn reply(
        &mut self,
        reply: HtmlInputRangeSanitizationReply,
    ) -> Result<(), HtmlInputRangeSanitizationReplyError> {
        let mut next = *self;
        next.advance(reply)?;
        *self = next;
        Ok(())
    }
    fn advance(
        &mut self,
        reply: HtmlInputRangeSanitizationReply,
    ) -> Result<(), HtmlInputRangeSanitizationReplyError> {
        use HtmlInputRangeSanitizationReply as R;
        self.phase = match (self.phase, reply) {
            (Phase::Midpoint, R::Midpoint(value))
                if value.value() >= self.minimum().value()
                    && value.value() <= self.maximum().value() =>
            {
                Phase::Replace {
                    value,
                    branch: HtmlInputRangeReplacement::InvalidDefaultMidpoint,
                    then: Continue::Clamp,
                }
            }
            (Phase::Replace { value, then, .. }, R::Applied) => {
                self.current = Some(value);
                self.replaced = true;
                match then {
                    Continue::Clamp => self.clamp(value),
                    Continue::Step => Phase::Step,
                    Continue::Complete => Phase::Complete,
                }
            }
            (Phase::Step, R::Step(step)) => {
                if self.constraints.minimum_origin() == HtmlInputNumericBoundOrigin::Recovered {
                    self.constraints
                        .update_step_value(step, HtmlInputNumericContent::Absent);
                    self.begin_lattice()
                } else {
                    Phase::ValueContent(step)
                }
            }
            (Phase::ValueContent(step), R::ValueContent(value)) => {
                self.constraints.update_step_value(step, value);
                self.begin_lattice()
            }
            (Phase::Lattice, R::Lattice(selected)) => {
                if let Some(value) = selected {
                    if value.value() < self.minimum().value()
                        || self
                            .effective_maximum()
                            .is_some_and(|max| value.value() > max.value())
                    {
                        return Err(HtmlInputRangeSanitizationReplyError);
                    }
                    // The existing lattice owner returns the exact current
                    // bits on numeric equality, including signed zero. A
                    // no-mismatch result therefore never canonicalizes -0.
                    if value.value().to_bits()
                        != self.current.expect("lattice has current").value().to_bits()
                    {
                        Phase::Replace {
                            value,
                            branch: HtmlInputRangeReplacement::StepCorrection,
                            then: Continue::Complete,
                        }
                    } else {
                        Phase::Complete
                    }
                } else {
                    Phase::Complete
                }
            }
            _ => return Err(HtmlInputRangeSanitizationReplyError),
        };
        Ok(())
    }
}
