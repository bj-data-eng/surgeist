//! Numeric and temporal input content limits and step, from HTML
//! #the-min-and-max-attributes, #the-step-attribute and selected state definitions.
//! These are immutable content observations, not current value sanitization,
//! validity, stepping, WebIDL reflection or runtime transitions. Attribution:
//! `licenses/whatwg/NOTICE.md`.

use super::*;
use crate::{
    internal::microsyntax::numeric::HtmlFloatingPointNumber,
    internal::microsyntax::numeric::HtmlNonNegativeDecimal,
    internal::microsyntax::temporal::HtmlDate, internal::microsyntax::temporal::HtmlLocalDateTime,
    internal::microsyntax::temporal::HtmlMonth,
    internal::microsyntax::temporal::HtmlParsedLocalDateTime,
    internal::microsyntax::temporal::HtmlParsedTime, internal::microsyntax::temporal::HtmlTime,
    internal::microsyntax::temporal::HtmlWeek,
    internal::microsyntax::temporal::input_coordinate::HtmlInputTemporalCoordinate,
    internal::microsyntax::temporal::input_coordinate::HtmlInputTemporalCoordinateUnit,
    internal::values::interpretation::HtmlValueAuthorState,
    internal::values::interpretation::HtmlValueFailure,
    internal::values::interpretation::HtmlValueRecoveryState,
    internal::values::interpretation::HtmlValueReport,
};
use std::{cmp::Ordering, error::Error, fmt};

/// Per-value raw admission precedes name, state and grammar interpretation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlInputNumericConstraintsError {
    /// A supplied raw value was refused by raw-value admission, before any
    /// name, state or grammar processing.
    Admission(HtmlValueAdmissionError),
    /// The sum of the supplied raw UTF-16 lengths cannot be represented by
    /// `usize`.
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
            Self::Admission(_) => {
                f.write_str("input numeric constraint value was refused by raw-value admission")
            }
            Self::InputCodeUnitsOverflow => {
                f.write_str("input numeric UTF-16 length cannot be represented")
            }
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
    /// The attribute is absent, so nothing is authored.
    Missing,
    /// Meets this role's author conditions. Its numeric representation can still
    /// be unavailable; the common report retains that independent outcome.
    Conforming,
    /// The whole value breaks the role's author grammar. The payload is the
    /// syntax or representation cause.
    Nonconforming(HtmlValueFailure),
    /// A grammatical step rounded to zero, or to a negative number.
    StepMustBePositive,
    /// The actual state grammar is outside this numeric/temporal operation.
    NotChecked,
    /// The rule does not apply to these names or this context.
    Unapplied,
}

/// Actual typed common evidence selected by input state, or by step's grammar.
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlInputNumericCommonReport {
    /// The floating-point number report, for Number and Range and for every `step` value except
    /// the keyword `any`, which is recognized before any common report is made.
    FloatingPoint(HtmlValueReport<HtmlFloatingPointNumber, HtmlFloatingPointNumber>),
    /// The date report, for the Date state.
    Date(HtmlValueReport<HtmlDate, HtmlDate>),
    /// The month report, for the Month state.
    Month(HtmlValueReport<HtmlMonth, HtmlMonth>),
    /// The week report, for the Week state.
    Week(HtmlValueReport<HtmlWeek, HtmlWeek>),
    /// The time report, for the Time state. Authoring and recovery differ in
    /// type: authoring checks a time, recovery yields a parsed time.
    Time(HtmlValueReport<HtmlTime, HtmlParsedTime>),
    /// The local date and time report, for the Local Date and Time state.
    LocalDateTime(HtmlValueReport<HtmlLocalDateTime, HtmlParsedLocalDateTime>),
}

/// A successful content conversion, distinct from the typed grammar evidence.
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlInputNumericAttributeValue {
    /// A finite binary64 number converted from the content.
    FloatingPoint(HtmlFloatingPointNumber),
    /// An exact temporal coordinate converted from the content.
    Temporal(HtmlInputTemporalCoordinate),
}

fn recovered_value<A, R>(report: &HtmlValueReport<A, R>) -> Option<&R> {
    match report.recovery() {
        HtmlValueRecoveryState::Value(value) => Some(value),
        _ => None,
    }
}
impl HtmlInputNumericCommonReport {
    fn author(&self, role: KnownHtmlAttribute) -> HtmlInputNumericAuthorState {
        match self {
            Self::FloatingPoint(report) => author(report, role),
            Self::Date(report) => author(report, role),
            Self::Month(report) => author(report, role),
            Self::Week(report) => author(report, role),
            Self::Time(report) => author(report, role),
            Self::LocalDateTime(report) => author(report, role),
        }
    }
    fn converted(&self) -> Option<HtmlInputNumericAttributeValue> {
        use HtmlInputNumericAttributeValue::{FloatingPoint, Temporal};
        Some(match self {
            Self::FloatingPoint(report) => FloatingPoint(*recovered_value(report)?),
            Self::Date(report) => Temporal(recovered_value(report)?.input_coordinate()),
            Self::Month(report) => Temporal(recovered_value(report)?.input_coordinate()),
            Self::Week(report) => Temporal(recovered_value(report)?.input_coordinate()),
            Self::Time(report) => Temporal(recovered_value(report)?.input_coordinate()),
            Self::LocalDateTime(report) => Temporal(recovered_value(report)?.input_coordinate()),
        })
    }
}

/// One role's exact shared attribute observation and optional common report.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlInputNumericAttributeReport {
    attribute: HtmlAttributeReport<()>,
    common: Option<HtmlInputNumericCommonReport>,
    converted: Option<HtmlInputNumericAttributeValue>,
    author: HtmlInputNumericAuthorState,
    any_keyword: bool,
}
impl HtmlInputNumericAttributeReport {
    /// The exact element name, including any stored prefix.
    pub fn element(&self) -> &QualifiedName {
        self.attribute.element()
    }
    /// The exact supplied attribute name, prefix and value, if present.
    pub fn raw_attribute(&self) -> Option<&Attribute> {
        self.attribute.raw_attribute()
    }
    /// The exact supplied raw value. `None` is an absent attribute; an empty
    /// value is `Some`.
    pub fn raw(&self) -> Option<&DocumentString> {
        self.attribute.raw()
    }
    /// Whether the attribute is absent, empty or nonempty, independent of
    /// validity.
    pub fn observation(&self) -> HtmlAttributeObservation {
        self.attribute.observation()
    }
    /// Whether the rule applies to these expanded names: only HTML `input`
    /// elements carry these attributes.
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.attribute.applicability()
    }
    /// The role this report observes: `min`, `max`, `step` or `value`.
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.attribute.attribute()
    }
    /// The original index row identifiers that this defining rule covers.
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.attribute.inventory_rows()
    }
    /// The normative clause in the pinned edition that defines this attribute.
    pub fn source_clause(&self) -> &'static str {
        self.attribute.source_clause()
    }
    /// The actual input state that the caller supplied. It is never inferred
    /// from a `type` attribute.
    pub fn input_state(&self) -> HtmlInputType {
        match self.attribute.supplied_context {
            SuppliedContext::Input(state) => state,
            _ => unreachable!("input numeric reports always carry their supplied state"),
        }
    }
    /// The clause that defines the supplied input state.
    pub fn input_state_source_clause(&self) -> &'static str {
        input_state_source(self.input_state())
    }
    /// The UTF-16 code units in the raw value that was admitted, or zero when
    /// the attribute is absent.
    pub fn input_code_units(&self) -> usize {
        self.attribute.input_code_units()
    }
    /// None means numeric grammar was not attempted (including exact step `any`).
    pub fn common(&self) -> Option<&HtmlInputNumericCommonReport> {
        self.common.as_ref()
    }
    /// Whether the whole authored value meets the role's grammar, independent
    /// of prefix recovery and defaults.
    pub fn author(&self) -> HtmlInputNumericAuthorState {
        self.author
    }
    /// Whether the supplied input state permits this attribute. A state that is
    /// neither Number, Range nor a temporal state forbids `min`, `max` and
    /// `step`; `value` is always permitted.
    pub fn context_author(&self) -> HtmlAttributeContextAuthorState {
        self.attribute.context_author()
    }
    /// True only for the step role's exact ASCII-case-insensitive keyword.
    pub fn any_keyword(&self) -> bool {
        self.any_keyword
    }
    /// The content converted to a number or coordinate, or `None` when the
    /// attribute is absent, unapplied, `any`, or did not convert.
    pub fn converted(&self) -> Option<&HtmlInputNumericAttributeValue> {
        self.converted.as_ref()
    }
    fn floating_point(&self) -> Option<HtmlFloatingPointNumber> {
        match self.converted()? {
            HtmlInputNumericAttributeValue::FloatingPoint(value) => Some(*value),
            _ => None,
        }
    }
    fn temporal(&self) -> Option<&HtmlInputTemporalCoordinate> {
        match self.converted()? {
            HtmlInputNumericAttributeValue::Temporal(value) => Some(value),
            _ => None,
        }
    }
}

/// Why a minimum/maximum is present or absent. Missing and failed parsing remain distinct.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNumericBoundOrigin {
    /// The content converted, so the bound is that value.
    Recovered,
    /// The attribute is absent, so the state's fallback applies: none for
    /// Number and the temporal states, 0 or 100 for Range.
    MissingFallback,
    /// The attribute is present but did not convert, so the state's fallback
    /// applies as for an absent one.
    ParseFailureFallback,
}
/// Why the allowed value step is what it is, following HTML877d #the-step-attribute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNumericStepOrigin {
    /// The value is the keyword `any`, so there is no allowed value step.
    Any,
    /// The value parsed to a number greater than zero, scaled by the step
    /// scale factor.
    Recovered,
    /// The attribute is absent, so the default step scaled by the step scale
    /// factor applies.
    MissingDefault,
    /// Parsing the value failed, so the scaled default step applies.
    ParseFailureDefault,
    /// The value parsed to zero or a negative number, so the scaled default
    /// step applies.
    NonPositiveDefault,
}
/// Where the step base comes from, following HTML877d #the-step-attribute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNumericStepBaseOrigin {
    /// The `min` content converted, and it is the step base.
    MinimumContent,
    /// `min` did not convert and the `value` content did, so it is the base.
    ValueContent,
    /// Neither converted and the state defines no explicit default step base,
    /// so the base is zero.
    Zero,
    /// Week's explicitly defined base, after absent/failed min and value.
    StateDefault,
}
/// The nonperiodic author relation compares effective limits, including Range
/// fallbacks. It is separate from each value's grammar; NotRequired means no
/// comparable pair, not that all supplied attributes are author-conforming.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNumericLimitAuthorState {
    /// Either effective limit is absent, so there is no pair to compare. Range
    /// always has both through its fallbacks.
    NotRequired,
    /// The pair is comparable and the minimum does not exceed the maximum, or
    /// the domain is periodic and a reversed range is allowed.
    Conforming,
    /// The domain is not periodic and the minimum exceeds the maximum, which
    /// the source forbids (HTML877d #the-min-and-max-attributes).
    MinimumExceedsMaximum,
}

/// Source-defined Number/Range static quantities in the common parser's finite
/// binary64 domain. A missing Number limit is None; Range falls back to 0/100.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlInputNumericConstraints {
    minimum: NumericBound,
    maximum: NumericBound,
    allowed_step: Option<HtmlFloatingPointNumber>,
    step_origin: HtmlInputNumericStepOrigin,
    step_base: HtmlFloatingPointNumber,
    step_base_origin: HtmlInputNumericStepBaseOrigin,
    limit_author: HtmlInputNumericLimitAuthorState,
}
/// One effective bound together with why it is that value. A recovered bound always carries its
/// value, so the content that the step base reads back is never separate from the bound.
#[derive(Clone, Copy, Debug, PartialEq)]
enum NumericBound {
    Recovered(HtmlFloatingPointNumber),
    MissingFallback(Option<HtmlFloatingPointNumber>),
    ParseFailureFallback(Option<HtmlFloatingPointNumber>),
}
impl NumericBound {
    fn select(side: HtmlInputNumericContent, default: Option<HtmlFloatingPointNumber>) -> Self {
        match side {
            HtmlInputNumericContent::Value(value) => Self::Recovered(value),
            HtmlInputNumericContent::Absent => Self::MissingFallback(default),
            HtmlInputNumericContent::Failed => Self::ParseFailureFallback(default),
        }
    }
    fn value(self) -> Option<HtmlFloatingPointNumber> {
        match self {
            Self::Recovered(value) => Some(value),
            Self::MissingFallback(value) | Self::ParseFailureFallback(value) => value,
        }
    }
    fn origin(self) -> HtmlInputNumericBoundOrigin {
        match self {
            Self::Recovered(_) => HtmlInputNumericBoundOrigin::Recovered,
            Self::MissingFallback(_) => HtmlInputNumericBoundOrigin::MissingFallback,
            Self::ParseFailureFallback(_) => HtmlInputNumericBoundOrigin::ParseFailureFallback,
        }
    }
    /// The content the step base takes: only a recovered bound is content.
    fn recovered_content(self) -> HtmlInputNumericContent {
        match self {
            Self::Recovered(value) => HtmlInputNumericContent::Value(value),
            Self::MissingFallback(_) | Self::ParseFailureFallback(_) => {
                HtmlInputNumericContent::Absent
            }
        }
    }
}
impl HtmlInputNumericConstraints {
    /// The effective minimum, or `None` when Number has none.
    pub fn minimum(&self) -> Option<HtmlFloatingPointNumber> {
        self.minimum.value()
    }
    /// The effective maximum, or `None` when Number has none.
    pub fn maximum(&self) -> Option<HtmlFloatingPointNumber> {
        self.maximum.value()
    }
    /// Why the minimum is what it is.
    pub fn minimum_origin(&self) -> HtmlInputNumericBoundOrigin {
        self.minimum.origin()
    }
    /// Why the maximum is what it is.
    pub fn maximum_origin(&self) -> HtmlInputNumericBoundOrigin {
        self.maximum.origin()
    }
    /// None is the source's `any` result, not a failed or unavailable computation.
    pub fn allowed_step(&self) -> Option<HtmlFloatingPointNumber> {
        self.allowed_step
    }
    /// Why the allowed step is what it is.
    pub fn step_origin(&self) -> HtmlInputNumericStepOrigin {
        self.step_origin
    }
    /// The step base of HTML877d #the-step-attribute.
    pub fn step_base(&self) -> HtmlFloatingPointNumber {
        self.step_base
    }
    /// Where the step base comes from.
    pub fn step_base_origin(&self) -> HtmlInputNumericStepBaseOrigin {
        self.step_base_origin
    }
    /// Whether the minimum does not exceed the maximum.
    pub fn limit_author(&self) -> HtmlInputNumericLimitAuthorState {
        self.limit_author
    }
}

/// Exact static temporal quantities, in the base coordinate's qualified state/unit.
/// These do not calculate periodic membership, current validity or stepping.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlInputTemporalConstraints {
    minimum: Option<HtmlInputTemporalCoordinate>,
    maximum: Option<HtmlInputTemporalCoordinate>,
    minimum_origin: HtmlInputNumericBoundOrigin,
    maximum_origin: HtmlInputNumericBoundOrigin,
    allowed_step: Option<HtmlNonNegativeDecimal>,
    step_origin: HtmlInputNumericStepOrigin,
    step_base: HtmlInputTemporalCoordinate,
    step_base_origin: HtmlInputNumericStepBaseOrigin,
    limit_author: HtmlInputNumericLimitAuthorState,
    reversed_range: bool,
}
impl HtmlInputTemporalConstraints {
    /// The temporal input state that all coordinates belong to.
    pub fn state(&self) -> HtmlInputType {
        self.step_base.state()
    }
    /// The unit of the state's coordinates.
    pub fn unit(&self) -> HtmlInputTemporalCoordinateUnit {
        self.step_base.unit()
    }
    /// The minimum, or `None` when the attribute is absent or did not convert.
    pub fn minimum(&self) -> Option<&HtmlInputTemporalCoordinate> {
        self.minimum.as_ref()
    }
    /// The maximum, or `None` when the attribute is absent or did not convert.
    pub fn maximum(&self) -> Option<&HtmlInputTemporalCoordinate> {
        self.maximum.as_ref()
    }
    /// Why the minimum is present or absent.
    pub fn minimum_origin(&self) -> HtmlInputNumericBoundOrigin {
        self.minimum_origin
    }
    /// Why the maximum is present or absent.
    pub fn maximum_origin(&self) -> HtmlInputNumericBoundOrigin {
        self.maximum_origin
    }
    /// None means exact `any`; defaults and positive recovered steps are exact decimals.
    pub fn allowed_step(&self) -> Option<&HtmlNonNegativeDecimal> {
        self.allowed_step.as_ref()
    }
    /// Why the allowed step is what it is.
    pub fn step_origin(&self) -> HtmlInputNumericStepOrigin {
        self.step_origin
    }
    /// The step base of HTML877d #the-step-attribute, in the state's coordinates.
    pub fn step_base(&self) -> &HtmlInputTemporalCoordinate {
        &self.step_base
    }
    /// Where the step base comes from.
    pub fn step_base_origin(&self) -> HtmlInputNumericStepBaseOrigin {
        self.step_base_origin
    }
    /// Whether the minimum does not exceed the maximum. A reversed range on a
    /// periodic domain conforms.
    pub fn limit_author(&self) -> HtmlInputNumericLimitAuthorState {
        self.limit_author
    }
    /// Whether the state's domain is periodic, so that ranges may span its
    /// boundary (HTML877d #the-min-and-max-attributes).
    pub fn has_periodic_domain(&self) -> bool {
        self.step_base.has_periodic_domain()
    }
    /// Whether the domain is periodic and the maximum is less than the minimum,
    /// which the source calls a reversed range (HTML877d #the-min-and-max-attributes).
    pub fn has_reversed_range(&self) -> bool {
        self.reversed_range
    }
}

/// Capability, source applicability and expanded-name failures are distinct.
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlInputNumericConstraintsOutcome {
    /// Quantities for the Number and Range states, in finite binary64.
    Computed(HtmlInputNumericConstraints),
    /// Exact quantities for the Date, Month, Week, Time and Local Date and Time
    /// states.
    ComputedTemporal(HtmlInputTemporalConstraints),
    /// The supplied state is neither Number, Range nor a temporal state, so no
    /// quantities exist. The payload is that state.
    InapplicableInputState(HtmlInputType),
    /// First failure in minimum, maximum, step, value order.
    Unapplied {
        /// The first role (`min`, `max`, `step` or `value`) whose rule was
        /// unapplied.
        attribute: KnownHtmlAttribute,
        /// Why that rule was unapplied.
        reason: HtmlAttributeApplicability,
    },
}

/// All four observations survive an unapplied/source-inapplicable compound outcome.
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
    /// The observation of the `min` attribute.
    pub fn minimum(&self) -> &HtmlInputNumericAttributeReport {
        &self.minimum
    }
    /// The observation of the `max` attribute.
    pub fn maximum(&self) -> &HtmlInputNumericAttributeReport {
        &self.maximum
    }
    /// The observation of the `step` attribute.
    pub fn step(&self) -> &HtmlInputNumericAttributeReport {
        &self.step
    }
    /// Value content is observed only as authored content and a potential step base.
    pub fn value(&self) -> &HtmlInputNumericAttributeReport {
        &self.value
    }
    /// The computed quantities, or the reason that none exist.
    pub fn outcome(&self) -> &HtmlInputNumericConstraintsOutcome {
        &self.outcome
    }
    /// The checked sum of the admitted raw value lengths in UTF-16 code units:
    /// `min`, `max`, `step` and `value`.
    pub fn input_code_units(&self) -> usize {
        self.input_code_units
    }
    /// The clause that defines the `min` and `max` attributes (HTML877d
    /// #the-min-and-max-attributes).
    pub fn limits_source_clause(&self) -> &'static str {
        source!("the-min-and-max-attributes")
    }
    /// The clause that defines the `step` attribute (HTML877d #the-step-attribute).
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
fn author<A, R>(
    common: &HtmlValueReport<A, R>,
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
        HtmlValueAuthorState::NotDefined => unreachable!("selected author grammars are defined"),
    }
}

impl HtmlValueInterpreter {
    fn input_numeric_attribute(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::internal::values::semantic_input::HtmlAttributeInput>,
        role: KnownHtmlAttribute,
        state: HtmlInputType,
    ) -> Result<HtmlInputNumericAttributeReport, HtmlValueAdmissionError> {
        let mut attribute = report(rule(role), element, raw);
        attribute.supplied_context = SuppliedContext::Input(state);
        let mut result = HtmlInputNumericAttributeReport {
            attribute,
            common: None,
            converted: None,
            author: HtmlInputNumericAuthorState::Unapplied,
            any_keyword: false,
        };
        if result.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        if !computed_state(state) && !temporal_state(state) {
            result.author = if raw.is_none() {
                HtmlInputNumericAuthorState::Missing
            } else {
                HtmlInputNumericAuthorState::NotChecked
            };
            result.attribute.context_author = if raw.is_some() && role != KnownHtmlAttribute::Value
            {
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                )
            } else {
                HtmlAttributeContextAuthorState::NotRequired
            };
            return Ok(result);
        }
        if role == KnownHtmlAttribute::Step
            && raw.is_some_and(|raw| ascii_keyword(raw.value(), HtmlInputNumericStep::ANY_KEYWORD))
        {
            result.any_keyword = true;
            result.author = HtmlInputNumericAuthorState::Conforming;
            return Ok(result);
        }
        let input = raw.map(HtmlAttributeInput::value);
        let common = if role == KnownHtmlAttribute::Step || computed_state(state) {
            HtmlInputNumericCommonReport::FloatingPoint(self.floating_point_number(input)?)
        } else {
            match state {
                HtmlInputType::Date => HtmlInputNumericCommonReport::Date(self.date(input)?),
                HtmlInputType::Month => HtmlInputNumericCommonReport::Month(self.month(input)?),
                HtmlInputType::Week => HtmlInputNumericCommonReport::Week(self.week(input)?),
                HtmlInputType::Time => HtmlInputNumericCommonReport::Time(self.time(input)?),
                HtmlInputType::LocalDateTime => {
                    HtmlInputNumericCommonReport::LocalDateTime(self.local_date_time(input)?)
                }
                _ => unreachable!("only supported temporal states dispatch here"),
            }
        };
        result.author = common.author(role);
        if role == KnownHtmlAttribute::Value
            && state != HtmlInputType::Range
            && raw.is_some_and(|raw| raw.value().is_empty())
        {
            result.author = HtmlInputNumericAuthorState::Conforming;
        } else if role == KnownHtmlAttribute::Step
            && let HtmlInputNumericCommonReport::FloatingPoint(ref float) = common
            && let HtmlValueAuthorState::Valid(value) = float.author()
            && value.value() <= 0.0
        {
            result.author = HtmlInputNumericAuthorState::StepMustBePositive;
        }
        result.converted = common.converted();
        result.common = Some(common);
        Ok(result)
    }

    /// Interpret numeric/temporal content limits and step together, using supplied
    /// actual input state. Every supplied UTF-16 value is admitted first. The
    /// standalone cap does not claim parser/session allocation or cancellation bounds.
    pub fn input_numeric_constraints<A: HtmlAttributeInput>(
        &self,
        element: &(impl crate::internal::values::semantic_input::HtmlNameInput + ?Sized),
        minimum: Option<&A>,
        maximum: Option<&A>,
        step: Option<&A>,
        value: Option<&A>,
        state: HtmlInputType,
    ) -> Result<HtmlInputNumericConstraintsReport, HtmlInputNumericConstraintsError> {
        let raws = [minimum, maximum, step, value];
        for raw in raws {
            self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
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
            HtmlInputNumericConstraintsOutcome::ComputedTemporal(temporal_constraints(
                &minimum, &maximum, &step, &value, state,
            ))
        } else if !computed_state(state) {
            HtmlInputNumericConstraintsOutcome::InapplicableInputState(state)
        } else {
            HtmlInputNumericConstraintsOutcome::Computed(
                select_input_numeric_constraints(
                    minimum.numeric_content(),
                    maximum.numeric_content(),
                    step.numeric_step(),
                    value.numeric_content(),
                    state,
                )
                .expect("computed_state qualified Number or Range"),
            )
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
/// Already-converted content facts. Ordinary reports and controlled Range
/// construction share this fixed selection owner; no raw value is copied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HtmlInputNumericContent {
    /// The attribute is absent.
    Absent,
    /// The attribute is present but did not convert to a number.
    Failed,
    /// The attribute converted to this finite number.
    Value(HtmlFloatingPointNumber),
}
/// Already-converted `step` content, as `HtmlInputNumericContent` is for the
/// other roles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HtmlInputNumericStep {
    /// The attribute is absent.
    Absent,
    /// The attribute is present but did not convert to a number and is not
    /// `any`.
    Failed,
    /// The attribute is the keyword `any`.
    Any,
    /// The attribute converted to this finite number, which may be zero or
    /// negative.
    Value(HtmlFloatingPointNumber),
}
impl HtmlInputNumericStep {
    /// The single authoritative spelling of the unrestricted step keyword.
    pub const ANY_KEYWORD: &'static str = "any";

    /// Tests one keyword position using ASCII case-insensitive comparison.
    /// Callers must check whole-value length and admit each reached unit first.
    /// An index outside the keyword never matches.
    pub fn any_keyword_unit_matches(index: usize, unit: u16) -> bool {
        Self::ANY_KEYWORD
            .as_bytes()
            .get(index)
            .is_some_and(|&byte| ascii_case_insensitive_unit(unit, byte))
    }
}
impl HtmlInputNumericAttributeReport {
    fn numeric_content(&self) -> HtmlInputNumericContent {
        if self.raw().is_none() {
            HtmlInputNumericContent::Absent
        } else if let Some(value) = self.floating_point() {
            HtmlInputNumericContent::Value(value)
        } else {
            HtmlInputNumericContent::Failed
        }
    }
    fn numeric_step(&self) -> HtmlInputNumericStep {
        if self.any_keyword {
            HtmlInputNumericStep::Any
        } else {
            match self.numeric_content() {
                HtmlInputNumericContent::Absent => HtmlInputNumericStep::Absent,
                HtmlInputNumericContent::Failed => HtmlInputNumericStep::Failed,
                HtmlInputNumericContent::Value(value) => HtmlInputNumericStep::Value(value),
            }
        }
    }
}
/// Selects Number/Range limits from already-converted exact content facts.
/// No source is scanned, copied or reparsed. Other input states return None.
pub fn select_input_numeric_constraints(
    min: HtmlInputNumericContent,
    max: HtmlInputNumericContent,
    step: HtmlInputNumericStep,
    value: HtmlInputNumericContent,
    state: HtmlInputType,
) -> Option<HtmlInputNumericConstraints> {
    if !computed_state(state) {
        return None;
    }
    if state == HtmlInputType::Range {
        return Some(select_range_constraints(min, max, step, value).constraints);
    }
    Some(assemble_constraints(
        NumericBound::select(min, None),
        NumericBound::select(max, None),
        min,
        step,
        value,
    ))
}

/// The Range state's constraints together with its two bounds. The state defaults a missing or
/// unparsable bound, so both bounds always exist and need no `Option`.
pub(crate) struct HtmlInputRangeBounds {
    pub(crate) constraints: HtmlInputNumericConstraints,
    pub(crate) minimum: HtmlFloatingPointNumber,
    pub(crate) maximum: HtmlFloatingPointNumber,
}

/// Selects the Range state's limits from already-converted exact content facts, defaulting the
/// minimum to 0 and the maximum to 100.
pub(crate) fn select_range_constraints(
    min: HtmlInputNumericContent,
    max: HtmlInputNumericContent,
    step: HtmlInputNumericStep,
    value: HtmlInputNumericContent,
) -> HtmlInputRangeBounds {
    let (default_minimum, default_maximum) = (number(0.0), number(100.0));
    let minimum = NumericBound::select(min, Some(default_minimum));
    let maximum = NumericBound::select(max, Some(default_maximum));
    HtmlInputRangeBounds {
        minimum: minimum.value().unwrap_or(default_minimum),
        maximum: maximum.value().unwrap_or(default_maximum),
        constraints: assemble_constraints(minimum, maximum, min, step, value),
    }
}

fn assemble_constraints(
    minimum: NumericBound,
    maximum: NumericBound,
    min: HtmlInputNumericContent,
    step: HtmlInputNumericStep,
    value: HtmlInputNumericContent,
) -> HtmlInputNumericConstraints {
    let (allowed_step, step_origin) = select_numeric_step(step);
    let (step_base, step_base_origin) = select_numeric_step_base(min, value);
    let limit_author = match (minimum.value(), maximum.value()) {
        (Some(minimum), Some(maximum)) if minimum.value() > maximum.value() => {
            HtmlInputNumericLimitAuthorState::MinimumExceedsMaximum
        }
        (Some(_), Some(_)) => HtmlInputNumericLimitAuthorState::Conforming,
        _ => HtmlInputNumericLimitAuthorState::NotRequired,
    };
    HtmlInputNumericConstraints {
        minimum,
        maximum,
        allowed_step,
        step_origin,
        step_base,
        step_base_origin,
        limit_author,
    }
}

fn select_numeric_step(
    step: HtmlInputNumericStep,
) -> (Option<HtmlFloatingPointNumber>, HtmlInputNumericStepOrigin) {
    match step {
        HtmlInputNumericStep::Any => (None, HtmlInputNumericStepOrigin::Any),
        HtmlInputNumericStep::Absent => (
            Some(number(1.0)),
            HtmlInputNumericStepOrigin::MissingDefault,
        ),
        HtmlInputNumericStep::Failed => (
            Some(number(1.0)),
            HtmlInputNumericStepOrigin::ParseFailureDefault,
        ),
        HtmlInputNumericStep::Value(value) if value.value() > 0.0 => {
            (Some(value), HtmlInputNumericStepOrigin::Recovered)
        }
        HtmlInputNumericStep::Value(_) => (
            Some(number(1.0)),
            HtmlInputNumericStepOrigin::NonPositiveDefault,
        ),
    }
}

fn select_numeric_step_base(
    min: HtmlInputNumericContent,
    value: HtmlInputNumericContent,
) -> (HtmlFloatingPointNumber, HtmlInputNumericStepBaseOrigin) {
    match (min, value) {
        (HtmlInputNumericContent::Value(value), _) => {
            (value, HtmlInputNumericStepBaseOrigin::MinimumContent)
        }
        (_, HtmlInputNumericContent::Value(value)) => {
            (value, HtmlInputNumericStepBaseOrigin::ValueContent)
        }
        _ => (number(0.0), HtmlInputNumericStepBaseOrigin::Zero),
    }
}

impl HtmlInputNumericConstraints {
    /// Update only the later observed step/base facts. Bound observations and
    /// their missing/failed origins remain exactly those already selected.
    pub(crate) fn update_step_value(
        &mut self,
        step: HtmlInputNumericStep,
        value: HtmlInputNumericContent,
    ) {
        let minimum_content = self.minimum.recovered_content();
        let (allowed_step, step_origin) = select_numeric_step(step);
        let (step_base, step_base_origin) = select_numeric_step_base(minimum_content, value);
        self.allowed_step = allowed_step;
        self.step_origin = step_origin;
        self.step_base = step_base;
        self.step_base_origin = step_base_origin;
    }
}

fn bound_origin(side: &HtmlInputNumericAttributeReport) -> HtmlInputNumericBoundOrigin {
    if side.temporal().is_some() {
        HtmlInputNumericBoundOrigin::Recovered
    } else if side.raw().is_none() {
        HtmlInputNumericBoundOrigin::MissingFallback
    } else {
        HtmlInputNumericBoundOrigin::ParseFailureFallback
    }
}

fn temporal_constraints(
    min: &HtmlInputNumericAttributeReport,
    max: &HtmlInputNumericAttributeReport,
    step: &HtmlInputNumericAttributeReport,
    value: &HtmlInputNumericAttributeReport,
    state: HtmlInputType,
) -> HtmlInputTemporalConstraints {
    let fallback = HtmlInputTemporalCoordinate::default_step_base_for(state)
        .expect("temporal dispatch requires a selected temporal state");
    let (step_base, step_base_origin) = if let Some(value) = min.temporal() {
        (
            value.clone(),
            HtmlInputNumericStepBaseOrigin::MinimumContent,
        )
    } else if let Some(value) = value.temporal() {
        (value.clone(), HtmlInputNumericStepBaseOrigin::ValueContent)
    } else {
        (
            fallback,
            if state == HtmlInputType::Week {
                HtmlInputNumericStepBaseOrigin::StateDefault
            } else {
                HtmlInputNumericStepBaseOrigin::Zero
            },
        )
    };
    let (allowed_step, step_origin) = if step.any_keyword {
        (None, HtmlInputNumericStepOrigin::Any)
    } else if step.raw().is_none() {
        (
            Some(step_base.default_allowed_step()),
            HtmlInputNumericStepOrigin::MissingDefault,
        )
    } else if let Some(number) = step.floating_point() {
        if number.value() > 0.0 {
            (
                Some(
                    step_base
                        .scale_step(number)
                        .expect("recovered step was checked positive"),
                ),
                HtmlInputNumericStepOrigin::Recovered,
            )
        } else {
            (
                Some(step_base.default_allowed_step()),
                HtmlInputNumericStepOrigin::NonPositiveDefault,
            )
        }
    } else {
        (
            Some(step_base.default_allowed_step()),
            HtmlInputNumericStepOrigin::ParseFailureDefault,
        )
    };
    let comparison = min
        .temporal()
        .zip(max.temporal())
        .map(|(min, max)| min.value().compare(max.value()));
    let periodic = step_base.has_periodic_domain();
    let reversed_range = periodic && comparison == Some(Ordering::Greater);
    let limit_author = match comparison {
        None => HtmlInputNumericLimitAuthorState::NotRequired,
        Some(Ordering::Greater) if !periodic => {
            HtmlInputNumericLimitAuthorState::MinimumExceedsMaximum
        }
        _ => HtmlInputNumericLimitAuthorState::Conforming,
    };
    HtmlInputTemporalConstraints {
        minimum: min.temporal().cloned(),
        maximum: max.temporal().cloned(),
        minimum_origin: bound_origin(min),
        maximum_origin: bound_origin(max),
        allowed_step,
        step_origin,
        step_base,
        step_base_origin,
        limit_author,
        reversed_range,
    }
}

/// Checked native author value: exact `any` or a finite strictly positive number.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlInputStep {
    number: Option<HtmlFloatingPointNumber>,
}
impl HtmlInputStep {
    /// The keyword `any`, which asks for no stepping constraint.
    pub fn any() -> Self {
        Self { number: None }
    }
    /// A finite, strictly positive step. Zero and negative numbers are refused.
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
    /// The positive number, or `None` for `any`.
    pub fn number(&self) -> Option<HtmlFloatingPointNumber> {
        self.number
    }
    /// The canonical spelling: `any`, or the number in plain decimal notation.
    pub fn as_document_string(&self) -> DocumentString {
        self.number.map_or_else(
            || DocumentString::from(HtmlInputNumericStep::ANY_KEYWORD),
            |number| number.to_plain_decimal_string(),
        )
    }
    /// Builds the no-namespace `step` attribute from the canonical spelling.
    pub fn into_step_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Step.qualified_name(),
            self.as_document_string(),
        )
    }
}
/// The refusal to build an `HtmlInputStep` from zero or a negative number.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlNonPositiveInputStepError;
impl fmt::Display for HtmlNonPositiveInputStepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("input step must be greater than zero")
    }
}
impl Error for HtmlNonPositiveInputStepError {}
impl HtmlFloatingPointNumber {
    /// Builds the no-namespace `min` attribute with the number in plain decimal
    /// notation. Applicability to an input state is not checked.
    pub fn into_input_min_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Min.qualified_name(),
            self.to_plain_decimal_string(),
        )
    }
    /// Builds the no-namespace `max` attribute with the number in plain decimal
    /// notation. Applicability to an input state is not checked.
    pub fn into_input_max_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Max.qualified_name(),
            self.to_plain_decimal_string(),
        )
    }
}
