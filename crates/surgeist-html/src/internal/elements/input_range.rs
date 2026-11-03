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
    /// The value is not a valid floating-point number. Range replaces it with its default value
    /// (HTML877d #concept-input-value-default-range).
    InvalidGrammar,
    /// The value is a valid floating-point number that converts to a finite binary64 value.
    Finite(HtmlInputRangeNumber),
    /// Grammar succeeded (HTML877d3216–3242) but standard conversion returned
    /// its overflow error (3325–3329; Range37061–37065 delegates to it).
    /// Range36939–36940 defaults only invalid grammar. The under/overflow
    /// predicates38737–38749 and mismatch38839–38843 require a number result,
    /// so this observation proves none of those conditions. It is not an
    /// unavailable computation, resource refusal, infinity or opaque default.
    ConversionFailed,
}

/// The reason the Range sanitization replaces the value, which selects the serialization the caller
/// installs. See HTML877d #value-sanitization-algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputRangeReplacement {
    /// The value was invalid and the maximum is less than the minimum, so the default is the
    /// minimum.
    InvalidDefaultMinimum,
    /// The value was invalid and the maximum is not less than the minimum, so the default is the
    /// minimum plus half the difference between the minimum and maximum.
    InvalidDefaultMidpoint,
    /// The value is below the minimum (suffering from an underflow), so it becomes the minimum.
    Underflow,
    /// The value is above the maximum (suffering from an overflow) and the maximum is not less than
    /// the minimum, so it becomes the maximum.
    Overflow,
    /// The value has a step mismatch, so it becomes the nearest admissible number; a tie takes the
    /// one nearer positive infinity.
    StepCorrection,
}

/// Fulfill the pending action before replying. Numeric replies must be obtained
/// from the corresponding existing HTML bounded numeric operation, not guessed
/// from platform floating arithmetic or supplied by a generic result provider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HtmlInputRangeSanitizationAction {
    /// Ask the caller for the midpoint of `minimum` and `maximum`, the default value when the
    /// maximum is not less than the minimum.
    Midpoint {
        /// The lower bound of the range.
        minimum: HtmlInputRangeNumber,
        /// The upper bound of the range.
        maximum: HtmlInputRangeNumber,
    },
    /// Serialize through HtmlBestFiniteNumberCursor and install the actual string.
    Replace {
        /// The number to serialize as the best floating-point representation and install as the
        /// value.
        value: HtmlInputRangeNumber,
        /// Why the value is replaced.
        branch: HtmlInputRangeReplacement,
    },
    /// Observe exact whole `any`, or Recovery grammar + finite ingress.
    /// This late read retains the selected controlled consumer order; HTML's
    /// source defines dependencies, not a universal eager evaluation order.
    ReadStep,
    /// Read the actual content value only when min supplied no finite base.
    ReadValueContent,
    /// HtmlRangeLatticeCursor selects actual admissible finite candidates, ties up.
    Lattice {
        /// The current value that the selected number must replace or equal.
        current: HtmlInputRangeNumber,
        /// The inclusive lower bound of the admissible candidates.
        minimum: HtmlInputRangeNumber,
        /// The inclusive upper bound of the admissible candidates, or `None` when the maximum is
        /// less than the minimum and so imposes no bound.
        maximum: Option<HtmlInputRangeNumber>,
        /// The step base from which admissible candidates are counted.
        base: HtmlInputRangeNumber,
        /// The allowed step between admissible candidates.
        step: HtmlInputRangeNumber,
    },
    /// The sanitization is done. No further reply is accepted.
    Complete,
}

/// The caller's reply to the cursor's pending action. Each variant answers one action, and any
/// other pairing is rejected with `HtmlInputRangeSanitizationReplyError`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HtmlInputRangeSanitizationReply {
    /// The midpoint for `Midpoint`. It must lie within the minimum and maximum, inclusive.
    Midpoint(HtmlInputRangeNumber),
    /// The replacement requested by `Replace` was installed.
    Applied,
    /// The step observation requested by `ReadStep`.
    Step(HtmlInputNumericStep),
    /// The `value` content observation requested by `ReadValueContent`, used as the step base when
    /// the minimum supplied none.
    ValueContent(HtmlInputNumericContent),
    /// None is a completed proof of no admissible candidate, not unavailable.
    Lattice(Option<HtmlInputRangeNumber>),
}

/// The refusal of a reply that does not fulfill the cursor's pending action, including a reply
/// after the cursor is complete or a candidate outside the admissible bounds. The cursor is left
/// unchanged.
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
    /// Carries the current value, absent until a finite or replacement value.
    Step(Option<HtmlInputRangeNumber>),
    ValueContent(HtmlInputNumericStep, Option<HtmlInputRangeNumber>),
    Lattice {
        current: HtmlInputRangeNumber,
        step: HtmlInputRangeNumber,
    },
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
    replaced: bool,
    phase: Phase,
}
impl HtmlInputRangeSanitizationCursor {
    /// Starts the sanitization from the completed `value` observation and the `min` and `max`
    /// content observations. It selects the numeric bounds for the Range state immediately; the
    /// step and value-content facts follow through `action` and `reply`.
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
            replaced: false,
            phase: Phase::Complete,
        };
        cursor.phase = match value {
            HtmlInputRangeValue::Finite(value) => cursor.clamp(value),
            HtmlInputRangeValue::ConversionFailed => Phase::Step(None),
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
            Phase::Step(Some(value))
        }
    }
    fn begin_lattice(&self, current: Option<HtmlInputRangeNumber>) -> Phase {
        match (current, self.constraints.allowed_step()) {
            (Some(current), Some(step)) => Phase::Lattice {
                current,
                step: step.into(),
            },
            _ => Phase::Complete,
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
            Phase::Lattice { .. }
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
    /// The action the caller must fulfill next, then answer with `reply`. It is `Complete` once
    /// sanitization is done.
    pub fn action(&self) -> HtmlInputRangeSanitizationAction {
        use HtmlInputRangeSanitizationAction as A;
        match self.phase {
            Phase::Midpoint => A::Midpoint {
                minimum: self.minimum(),
                maximum: self.maximum(),
            },
            Phase::Replace { value, branch, .. } => A::Replace { value, branch },
            Phase::Step(_) => A::ReadStep,
            Phase::ValueContent(..) => A::ReadValueContent,
            Phase::Lattice { current, step } => A::Lattice {
                current,
                minimum: self.minimum(),
                maximum: self.effective_maximum(),
                base: self.constraints.step_base().into(),
                step,
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
                self.replaced = true;
                match then {
                    Continue::Clamp => self.clamp(value),
                    Continue::Step => Phase::Step(Some(value)),
                    Continue::Complete => Phase::Complete,
                }
            }
            (Phase::Step(current), R::Step(step)) => {
                if self.constraints.minimum_origin() == HtmlInputNumericBoundOrigin::Recovered {
                    self.constraints
                        .update_step_value(step, HtmlInputNumericContent::Absent);
                    self.begin_lattice(current)
                } else {
                    Phase::ValueContent(step, current)
                }
            }
            (Phase::ValueContent(step, current), R::ValueContent(value)) => {
                self.constraints.update_step_value(step, value);
                self.begin_lattice(current)
            }
            (Phase::Lattice { current, .. }, R::Lattice(selected)) => {
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
                    if value.value().to_bits() != current.value().to_bits() {
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
