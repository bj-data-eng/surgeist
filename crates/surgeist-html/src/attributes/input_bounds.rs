//! Number/Range content limits and step, from HTML #the-min-and-max-attributes,
//! #the-step-attribute and the two state definitions at the selected HTML pin.
//! These are immutable content observations, not current value sanitization,
//! validity, stepping, WebIDL reflection or runtime transitions. Attribution:
//! `licenses/whatwg/NOTICE.md`.

use super::*;
use crate::{
    HtmlFloatingPointNumber, HtmlValueAuthorState, HtmlValueFailure, HtmlValueRecoveryState,
    HtmlValueReport,
};
use std::{error::Error, fmt};

/// Per-value raw admission precedes name, state and grammar interpretation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNumericConstraintsError {
    Admission(HtmlValueAdmissionError),
    InputCodeUnitsOverflow,
}
impl From<HtmlValueAdmissionError> for HtmlInputNumericConstraintsError {
    fn from(value: HtmlValueAdmissionError) -> Self {
        Self::Admission(value)
    }
}
impl fmt::Display for HtmlInputNumericConstraintsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Admission(error) => error.fmt(f),
            Self::InputCodeUnitsOverflow => f.write_str("input numeric UTF-16 length overflow"),
        }
    }
}
impl Error for HtmlInputNumericConstraintsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Admission(error) => Some(error),
            Self::InputCodeUnitsOverflow => None,
        }
    }
}

/// Whole authored value, independent of prefix recovery and effective defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNumericAuthorState {
    Missing,
    /// Meets this role's author conditions. Its numeric representation can still
    /// be unavailable; the common report retains that independent outcome.
    Conforming,
    Nonconforming(HtmlValueFailure),
    /// A grammatical step rounded to zero, or to a negative number.
    StepMustBePositive,
    /// The actual state grammar is outside this Number/Range operation.
    NotChecked,
    Unapplied,
}

/// One role's exact shared attribute observation and optional common numeric report.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlInputNumericAttributeReport {
    attribute: HtmlAttributeReport<()>,
    common: Option<HtmlValueReport<HtmlFloatingPointNumber, HtmlFloatingPointNumber>>,
    author: HtmlInputNumericAuthorState,
    any_keyword: bool,
}
impl HtmlInputNumericAttributeReport {
    pub fn element(&self) -> &QualifiedName {
        self.attribute.element()
    }
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.attribute.raw_attribute()
    }
    pub fn raw(&self) -> Option<&DocumentString> {
        self.attribute.raw()
    }
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.attribute.observation()
    }
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.attribute.applicability()
    }
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.attribute.attribute()
    }
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.attribute.inventory_rows()
    }
    pub fn source_clause(&self) -> &'static str {
        self.attribute.source_clause()
    }
    pub fn input_state(&self) -> HtmlInputType {
        match self.attribute.supplied_context {
            SuppliedContext::Input(state) => state,
            _ => unreachable!("input numeric reports always carry their supplied state"),
        }
    }
    pub fn input_state_source_clause(&self) -> &'static str {
        input_state_source(self.input_state())
    }
    pub fn input_code_units(&self) -> usize {
        self.attribute.input_code_units()
    }
    /// None means numeric grammar was not attempted (including exact step `any`).
    pub fn common(
        &self,
    ) -> Option<&HtmlValueReport<HtmlFloatingPointNumber, HtmlFloatingPointNumber>> {
        self.common.as_ref()
    }
    pub fn author(&self) -> HtmlInputNumericAuthorState {
        self.author
    }
    pub fn context_author(&self) -> HtmlAttributeContextAuthorState {
        self.attribute.context_author()
    }
    /// True only for the step role's exact ASCII-case-insensitive keyword.
    pub fn any_keyword(&self) -> bool {
        self.any_keyword
    }
    pub fn recovered(&self) -> Option<HtmlFloatingPointNumber> {
        match self.common.as_ref()?.recovery() {
            HtmlValueRecoveryState::Value(value) => Some(*value),
            _ => None,
        }
    }
}

/// Why a minimum/maximum is present or absent. Missing and failed parsing remain distinct.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNumericBoundOrigin {
    Recovered,
    MissingFallback,
    ParseFailureFallback,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNumericStepOrigin {
    Any,
    Recovered,
    MissingDefault,
    ParseFailureDefault,
    NonPositiveDefault,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNumericStepBaseOrigin {
    MinimumContent,
    ValueContent,
    Zero,
}
/// The nonperiodic author relation compares effective limits, including Range
/// fallbacks. It is separate from each value's grammar; NotRequired means no
/// comparable pair, not that all supplied attributes are author-conforming.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNumericLimitAuthorState {
    NotRequired,
    Conforming,
    MinimumExceedsMaximum,
}

/// Source-defined Number/Range static quantities in the common parser's finite
/// binary64 domain. A missing Number limit is None; Range falls back to 0/100.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlInputNumericConstraints {
    minimum: Option<HtmlFloatingPointNumber>,
    maximum: Option<HtmlFloatingPointNumber>,
    minimum_origin: HtmlInputNumericBoundOrigin,
    maximum_origin: HtmlInputNumericBoundOrigin,
    allowed_step: Option<HtmlFloatingPointNumber>,
    step_origin: HtmlInputNumericStepOrigin,
    step_base: HtmlFloatingPointNumber,
    step_base_origin: HtmlInputNumericStepBaseOrigin,
    limit_author: HtmlInputNumericLimitAuthorState,
}
impl HtmlInputNumericConstraints {
    pub fn minimum(&self) -> Option<HtmlFloatingPointNumber> {
        self.minimum
    }
    pub fn maximum(&self) -> Option<HtmlFloatingPointNumber> {
        self.maximum
    }
    pub fn minimum_origin(&self) -> HtmlInputNumericBoundOrigin {
        self.minimum_origin
    }
    pub fn maximum_origin(&self) -> HtmlInputNumericBoundOrigin {
        self.maximum_origin
    }
    /// None is the source's `any` result, not a failed or unavailable computation.
    pub fn allowed_step(&self) -> Option<HtmlFloatingPointNumber> {
        self.allowed_step
    }
    pub fn step_origin(&self) -> HtmlInputNumericStepOrigin {
        self.step_origin
    }
    pub fn step_base(&self) -> HtmlFloatingPointNumber {
        self.step_base
    }
    pub fn step_base_origin(&self) -> HtmlInputNumericStepBaseOrigin {
        self.step_base_origin
    }
    pub fn limit_author(&self) -> HtmlInputNumericLimitAuthorState {
        self.limit_author
    }
}

/// Capability, source applicability and expanded-name failures are distinct.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HtmlInputNumericConstraintsOutcome {
    Computed(HtmlInputNumericConstraints),
    /// Min/max/step apply in this state; its temporal grammar is not executed here.
    UnsupportedNumericState(HtmlInputType),
    InapplicableInputState(HtmlInputType),
    /// First failure in minimum, maximum, step, value order.
    Unapplied {
        attribute: KnownHtmlAttribute,
        reason: HtmlAttributeApplicability,
    },
}

/// All four observations survive an unapplied/unsupported compound outcome.
/// The input count is metadata, not another admission cap or work/memory ledger.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlInputNumericConstraintsReport {
    minimum: HtmlInputNumericAttributeReport,
    maximum: HtmlInputNumericAttributeReport,
    step: HtmlInputNumericAttributeReport,
    value: HtmlInputNumericAttributeReport,
    outcome: HtmlInputNumericConstraintsOutcome,
    input_code_units: usize,
}
impl HtmlInputNumericConstraintsReport {
    pub fn minimum(&self) -> &HtmlInputNumericAttributeReport {
        &self.minimum
    }
    pub fn maximum(&self) -> &HtmlInputNumericAttributeReport {
        &self.maximum
    }
    pub fn step(&self) -> &HtmlInputNumericAttributeReport {
        &self.step
    }
    /// Value content is observed only as authored content and a potential step base.
    pub fn value(&self) -> &HtmlInputNumericAttributeReport {
        &self.value
    }
    pub fn outcome(&self) -> HtmlInputNumericConstraintsOutcome {
        self.outcome
    }
    pub fn input_code_units(&self) -> usize {
        self.input_code_units
    }
    pub fn limits_source_clause(&self) -> &'static str {
        source!("the-min-and-max-attributes")
    }
    pub fn step_source_clause(&self) -> &'static str {
        source!("the-step-attribute")
    }
}

fn rule(attribute: KnownHtmlAttribute) -> RuleMetadata {
    let (rows, source): (&'static [u16], _) = match attribute {
        KnownHtmlAttribute::Min => (&[97], source!("attr-input-min")),
        KnownHtmlAttribute::Max => (&[92], source!("attr-input-max")),
        KnownHtmlAttribute::Step => (&[154], source!("attr-input-step")),
        KnownHtmlAttribute::Value => (&[175], source!("attr-input-value")),
        _ => unreachable!("only the four selected input roles have rules"),
    };
    RuleMetadata {
        attribute,
        rows,
        source,
        context: Context::Elements(&[KnownHtmlElement::Input]),
        vocabulary_source: None,
    }
}

fn computed_state(state: HtmlInputType) -> bool {
    matches!(state, HtmlInputType::Number | HtmlInputType::Range)
}
fn temporal_state(state: HtmlInputType) -> bool {
    matches!(
        state,
        HtmlInputType::Date
            | HtmlInputType::Month
            | HtmlInputType::Week
            | HtmlInputType::Time
            | HtmlInputType::LocalDateTime
    )
}
fn author(
    common: &HtmlValueReport<HtmlFloatingPointNumber, HtmlFloatingPointNumber>,
    role: KnownHtmlAttribute,
) -> HtmlInputNumericAuthorState {
    match common.author() {
        HtmlValueAuthorState::Missing => HtmlInputNumericAuthorState::Missing,
        HtmlValueAuthorState::Valid(_) => HtmlInputNumericAuthorState::Conforming,
        HtmlValueAuthorState::Invalid(error) => {
            HtmlInputNumericAuthorState::Nonconforming(HtmlValueFailure::Syntax(*error))
        }
        HtmlValueAuthorState::Unrepresentable(error) if role == KnownHtmlAttribute::Step => {
            // Step requires successful parsing to a number greater than zero.
            HtmlInputNumericAuthorState::Nonconforming(HtmlValueFailure::Representation(*error))
        }
        // Min/max/value require the whole author grammar, not successful
        // binary64 conversion. Unrepresentable explicitly certifies that grammar.
        HtmlValueAuthorState::Unrepresentable(_) => HtmlInputNumericAuthorState::Conforming,
        HtmlValueAuthorState::NotDefined => unreachable!("floating-point grammar is defined"),
    }
}

impl HtmlValueInterpreter {
    fn input_numeric_attribute(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        role: KnownHtmlAttribute,
        state: HtmlInputType,
    ) -> Result<HtmlInputNumericAttributeReport, HtmlValueAdmissionError> {
        let mut attribute = report(rule(role), element, raw);
        attribute.supplied_context = SuppliedContext::Input(state);
        let mut result = HtmlInputNumericAttributeReport {
            attribute,
            common: None,
            author: HtmlInputNumericAuthorState::Unapplied,
            any_keyword: false,
        };
        if result.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        if !computed_state(state) {
            result.author = if raw.is_none() {
                HtmlInputNumericAuthorState::Missing
            } else {
                HtmlInputNumericAuthorState::NotChecked
            };
            result.attribute.context_author = if raw.is_some() && role != KnownHtmlAttribute::Value
            {
                if temporal_state(state) {
                    HtmlAttributeContextAuthorState::Unchecked
                } else {
                    HtmlAttributeContextAuthorState::Nonconforming(
                        HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                    )
                }
            } else {
                HtmlAttributeContextAuthorState::NotRequired
            };
            return Ok(result);
        }
        if role == KnownHtmlAttribute::Step
            && raw.is_some_and(|raw| ascii_keyword(raw.value(), "any"))
        {
            result.any_keyword = true;
            result.author = HtmlInputNumericAuthorState::Conforming;
            return Ok(result);
        }
        let common = self.floating_point_number(raw.map(Attribute::value))?;
        result.author = author(&common, role);
        if role == KnownHtmlAttribute::Value
            && state == HtmlInputType::Number
            && raw.is_some_and(|raw| raw.value().is_empty())
        {
            result.author = HtmlInputNumericAuthorState::Conforming;
        } else if role == KnownHtmlAttribute::Step
            && let HtmlValueAuthorState::Valid(value) = common.author()
            && value.value() <= 0.0
        {
            result.author = HtmlInputNumericAuthorState::StepMustBePositive;
        }
        result.common = Some(common);
        Ok(result)
    }

    /// Interpret Number/Range content limits and step together, using supplied
    /// actual input state. Every supplied UTF-16 value is admitted first. The
    /// standalone cap does not claim parser/session allocation or cancellation bounds.
    pub fn input_numeric_constraints(
        &self,
        element: &QualifiedName,
        minimum: Option<&Attribute>,
        maximum: Option<&Attribute>,
        step: Option<&Attribute>,
        value: Option<&Attribute>,
        state: HtmlInputType,
    ) -> Result<HtmlInputNumericConstraintsReport, HtmlInputNumericConstraintsError> {
        let raws = [minimum, maximum, step, value];
        for raw in raws {
            self.admit_attribute_raw(raw.map(Attribute::value))?;
        }
        let input_code_units = raws.into_iter().try_fold(0usize, |sum, raw| {
            sum.checked_add(raw.map_or(0, |raw| raw.value().code_units().len()))
                .ok_or(HtmlInputNumericConstraintsError::InputCodeUnitsOverflow)
        })?;
        let minimum =
            self.input_numeric_attribute(element, minimum, KnownHtmlAttribute::Min, state)?;
        let maximum =
            self.input_numeric_attribute(element, maximum, KnownHtmlAttribute::Max, state)?;
        let step = self.input_numeric_attribute(element, step, KnownHtmlAttribute::Step, state)?;
        let value =
            self.input_numeric_attribute(element, value, KnownHtmlAttribute::Value, state)?;
        let failure = [&minimum, &maximum, &step, &value]
            .into_iter()
            .find(|side| side.applicability() != HtmlAttributeApplicability::Applicable);
        let outcome = if let Some(side) = failure {
            HtmlInputNumericConstraintsOutcome::Unapplied {
                attribute: side.attribute(),
                reason: side.applicability(),
            }
        } else if temporal_state(state) {
            HtmlInputNumericConstraintsOutcome::UnsupportedNumericState(state)
        } else if !computed_state(state) {
            HtmlInputNumericConstraintsOutcome::InapplicableInputState(state)
        } else {
            HtmlInputNumericConstraintsOutcome::Computed(constraints(
                &minimum, &maximum, &step, &value, state,
            ))
        };
        Ok(HtmlInputNumericConstraintsReport {
            minimum,
            maximum,
            step,
            value,
            outcome,
            input_code_units,
        })
    }
}

fn number(value: f64) -> HtmlFloatingPointNumber {
    HtmlFloatingPointNumber::try_from(value).expect("selected finite Number/Range constants")
}
fn bound(
    side: &HtmlInputNumericAttributeReport,
    default: Option<HtmlFloatingPointNumber>,
) -> (Option<HtmlFloatingPointNumber>, HtmlInputNumericBoundOrigin) {
    if let Some(value) = side.recovered() {
        (Some(value), HtmlInputNumericBoundOrigin::Recovered)
    } else {
        (
            default,
            if side.raw().is_none() {
                HtmlInputNumericBoundOrigin::MissingFallback
            } else {
                HtmlInputNumericBoundOrigin::ParseFailureFallback
            },
        )
    }
}
fn constraints(
    min: &HtmlInputNumericAttributeReport,
    max: &HtmlInputNumericAttributeReport,
    step: &HtmlInputNumericAttributeReport,
    value: &HtmlInputNumericAttributeReport,
    state: HtmlInputType,
) -> HtmlInputNumericConstraints {
    let range = state == HtmlInputType::Range;
    let (minimum, minimum_origin) = bound(min, range.then(|| number(0.0)));
    let (maximum, maximum_origin) = bound(max, range.then(|| number(100.0)));
    let (allowed_step, step_origin) = if step.any_keyword {
        (None, HtmlInputNumericStepOrigin::Any)
    } else if step.raw().is_none() {
        (
            Some(number(1.0)),
            HtmlInputNumericStepOrigin::MissingDefault,
        )
    } else if let Some(value) = step.recovered() {
        if value.value() > 0.0 {
            (Some(value), HtmlInputNumericStepOrigin::Recovered)
        } else {
            (
                Some(number(1.0)),
                HtmlInputNumericStepOrigin::NonPositiveDefault,
            )
        }
    } else {
        (
            Some(number(1.0)),
            HtmlInputNumericStepOrigin::ParseFailureDefault,
        )
    };
    let (step_base, step_base_origin) = if let Some(value) = min.recovered() {
        (value, HtmlInputNumericStepBaseOrigin::MinimumContent)
    } else if let Some(value) = value.recovered() {
        (value, HtmlInputNumericStepBaseOrigin::ValueContent)
    } else {
        (number(0.0), HtmlInputNumericStepBaseOrigin::Zero)
    };
    let limit_author = match (minimum, maximum) {
        (Some(minimum), Some(maximum)) if minimum.value() > maximum.value() => {
            HtmlInputNumericLimitAuthorState::MinimumExceedsMaximum
        }
        (Some(_), Some(_)) => HtmlInputNumericLimitAuthorState::Conforming,
        _ => HtmlInputNumericLimitAuthorState::NotRequired,
    };
    HtmlInputNumericConstraints {
        minimum,
        maximum,
        minimum_origin,
        maximum_origin,
        allowed_step,
        step_origin,
        step_base,
        step_base_origin,
        limit_author,
    }
}

/// Checked native author value: exact `any` or a finite strictly positive number.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlInputStep {
    number: Option<HtmlFloatingPointNumber>,
}
impl HtmlInputStep {
    pub fn any() -> Self {
        Self { number: None }
    }
    pub fn positive(
        number: HtmlFloatingPointNumber,
    ) -> Result<Self, HtmlNonPositiveInputStepError> {
        if number.value() > 0.0 {
            Ok(Self {
                number: Some(number),
            })
        } else {
            Err(HtmlNonPositiveInputStepError)
        }
    }
    pub fn number(&self) -> Option<HtmlFloatingPointNumber> {
        self.number
    }
    pub fn as_document_string(&self) -> DocumentString {
        self.number.map_or_else(
            || DocumentString::from("any"),
            |number| number.as_document_string(),
        )
    }
    pub fn into_step_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Step.qualified_name(),
            self.as_document_string(),
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNonPositiveInputStepError;
impl fmt::Display for HtmlNonPositiveInputStepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("input step must be greater than zero")
    }
}
impl Error for HtmlNonPositiveInputStepError {}
impl HtmlFloatingPointNumber {
    pub fn into_input_min_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Min.qualified_name(),
            self.as_document_string(),
        )
    }
    pub fn into_input_max_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Max.qualified_name(),
            self.as_document_string(),
        )
    }
}
