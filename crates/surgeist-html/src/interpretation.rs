//! Source-qualified HTML value interpretation over exact raw values.
//!
//! Attribute-specific applicability, defaults, ranges and effective states are
//! separate HTML algorithms. Standalone operations never guess those rules or
//! select an element's captured child-text fallback.

use crate::color::{HtmlLegacyColor, LegacyColorParseError};
use crate::numeric::{
    HtmlDimension, HtmlDimensionList, HtmlFloatingPointList, HtmlFloatingPointNumber, HtmlInteger,
    HtmlNonNegativeInteger, NumericParseError,
};
use crate::temporal::{
    HtmlDate, HtmlDateOrTime, HtmlDateWithOptionalTime, HtmlDuration, HtmlGlobalDateTime,
    HtmlLocalDateTime, HtmlMachineReadableTime, HtmlMonth, HtmlParsedDuration,
    HtmlParsedGlobalDateTime, HtmlParsedLocalDateTime, HtmlParsedTime, HtmlTemporalError, HtmlTime,
    HtmlTimeElementValue, HtmlTimeZoneOffset, HtmlWeek, HtmlYearlessDate,
};
use crate::tokens::{HtmlCommaSeparatedTokens, HtmlSpaceSeparatedTokens};
use crate::{DocumentString, color, numeric, temporal, tokens};
use std::{error::Error, fmt};

/// The selected value clause, independent of an attribute's contextual overlay.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlValueSyntax {
    /// The pinned HTML legacy-color processing clause.
    LegacyColor,
    /// HTML space-separated tokens using ASCII whitespace.
    SpaceSeparatedTokens,
    /// HTML comma-separated tokens with Infra boundary trimming.
    CommaSeparatedTokens,
    /// The pinned HTML `signed-integers` clause.
    Integer,
    /// The pinned HTML `non-negative-integers` clause.
    NonnegativeInteger,
    /// The pinned HTML `floating-point-numbers` clause.
    FloatingPointNumber,
    /// The pinned HTML `lists-of-floating-point-numbers` clause.
    FloatingPointList,
    /// The pinned HTML `percentages-and-dimensions` clause.
    Dimension,
    /// The pinned HTML `nonzero-percentages-and-lengths` clause.
    NonzeroDimension,
    /// The pinned HTML `lists-of-dimensions` clause.
    DimensionList,
    /// The pinned HTML `months` clause.
    Month,
    /// The pinned HTML `dates` clause.
    Date,
    /// The pinned HTML `yearless-dates` clause.
    YearlessDate,
    /// The pinned HTML `times` clause.
    Time,
    /// The pinned HTML `local-dates-and-times` clause.
    LocalDateTime,
    /// The pinned HTML `local-dates-and-times` clause.
    NormalizedLocalDateTime,
    /// The pinned HTML `time-zones` clause.
    TimeZoneOffset,
    /// The pinned HTML `global-dates-and-times` clause.
    GlobalDateTime,
    /// The pinned HTML `weeks` clause.
    Week,
    /// The pinned HTML `durations` clause.
    Duration,
    /// The pinned HTML `vaguer-moments-in-time` clause.
    DateOrTime,
    /// The pinned HTML `vaguer-moments-in-time` clause.
    DateWithOptionalTime,
    /// The time element's ten authored syntaxes and ordered processing rule,
    /// defined in §4.5.14 rather than a separate common microsyntax.
    TimeElement,
}

impl HtmlValueSyntax {
    /// Returns the immutable selected HTML source clause.
    ///
    /// Dimension-list execution additionally applies the cursor correction
    /// recorded in issue #1133 and the retained dimension-list browser witness.
    pub fn source_clause(self) -> &'static str {
        match self {
            Self::LegacyColor => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#colours"
            }
            Self::SpaceSeparatedTokens => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#space-separated-tokens"
            }
            Self::CommaSeparatedTokens => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#comma-separated-tokens"
            }
            Self::Integer => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#signed-integers"
            }
            Self::NonnegativeInteger => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#non-negative-integers"
            }
            Self::FloatingPointNumber => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#floating-point-numbers"
            }
            Self::FloatingPointList => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#lists-of-floating-point-numbers"
            }
            Self::Dimension => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#percentages-and-dimensions"
            }
            Self::NonzeroDimension => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#nonzero-percentages-and-lengths"
            }
            Self::DimensionList => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#lists-of-dimensions"
            }
            Self::Month => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#months"
            }
            Self::Date => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dates"
            }
            Self::YearlessDate => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#yearless-dates"
            }
            Self::Time => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#times"
            }
            Self::LocalDateTime => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#local-dates-and-times"
            }
            Self::NormalizedLocalDateTime => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#local-dates-and-times"
            }
            Self::TimeZoneOffset => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#time-zones"
            }
            Self::GlobalDateTime => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#global-dates-and-times"
            }
            Self::Week => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#weeks"
            }
            Self::Duration => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#durations"
            }
            Self::DateOrTime => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#vaguer-moments-in-time"
            }
            Self::DateWithOptionalTime => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#vaguer-moments-in-time"
            }
            Self::TimeElement => {
                "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-time-element"
            }
        }
    }
}

/// Failure of the selected grammar or a source-defined rejection condition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlValueSyntaxError {
    /// Legacy-color processing rejects an originally empty raw value.
    EmptyLegacyColor,
    /// Legacy-color processing rejects the ASCII-insensitive transparent keyword.
    TransparentLegacyColor,
    /// The selected numeric syntax could not be recognized.
    InvalidNumericSyntax,
    /// Nonnegative-integer recovery produced a negative mathematical integer.
    NegativeNonnegativeInteger,
    /// Nonzero-dimension recovery produced exact zero.
    ZeroNonzeroDimension,
    /// The selected temporal valid-string grammar rejected the value.
    Temporal(HtmlTemporalError),
}

/// A mathematical value outside the selected machine representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlValueRepresentationError {
    /// The decimal number rounds to infinity in the selected binary64 model.
    FloatingPointOverflow,
}

/// A failed recovery operation, distinct from resource admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlValueFailure {
    /// Failure of source-defined syntax or a rejection condition.
    Syntax(HtmlValueSyntaxError),
    /// A value cannot be represented by the selected numeric model.
    Representation(HtmlValueRepresentationError),
}

/// Authored conformance and checked construction for one common value clause.
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlValueAuthorState<T> {
    /// No raw attribute value was supplied; no default is inferred.
    Missing,
    /// The entire raw value satisfies the selected author grammar.
    Valid(T),
    /// The raw value fails the selected author grammar.
    Invalid(HtmlValueSyntaxError),
    /// The entire author grammar is valid, but its value cannot be represented.
    Unrepresentable(HtmlValueRepresentationError),
    /// The selected common clause defines processing, not an author grammar.
    ///
    /// Attribute-specific conformance remains with that attribute's clause.
    NotDefined,
}

/// A separate source-defined processing/recovery attempt.
#[derive(Clone, Debug, PartialEq)]
pub enum HtmlValueRecoveryState<T> {
    /// No raw value was supplied; no effective default is inferred.
    Missing,
    /// The implemented source algorithm produced this value.
    Value(T),
    /// The source algorithm failed, without replacing the raw value.
    Failed(HtmlValueFailure),
}

/// An immutable raw value and the two distinct attempts to interpret it.
///
/// `A` is the authored value domain and `R` is the processing-result domain.
/// Temporal processing can preserve precision unavailable to authored values;
/// the report never coerces a recovered result into the authored domain.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlValueReport<A, R> {
    raw: Option<DocumentString>,
    syntax: HtmlValueSyntax,
    author: HtmlValueAuthorState<A>,
    recovery: HtmlValueRecoveryState<R>,
}

impl<A, R> HtmlValueReport<A, R> {
    /// Returns the exact raw value, preserving absent versus present empty.
    pub fn raw(&self) -> Option<&DocumentString> {
        self.raw.as_ref()
    }
    /// Returns the selected common microsyntax.
    pub fn syntax(&self) -> HtmlValueSyntax {
        self.syntax
    }
    /// Returns the whole-value author-conformance attempt.
    pub fn author(&self) -> &HtmlValueAuthorState<A> {
        &self.author
    }
    /// Returns the independent processing/recovery attempt.
    pub fn recovery(&self) -> &HtmlValueRecoveryState<R> {
        &self.recovery
    }
    /// Returns the supplied UTF-16 units admitted for this single operation.
    pub fn input_code_units(&self) -> usize {
        self.raw.as_ref().map_or(0, DocumentString::len)
    }
}

/// One value exceeds the caller's finite standalone input capacity.
///
/// No syntax algorithm has run and no successful partial report is returned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlValueAdmissionError {
    capacity: usize,
    required: usize,
}

impl HtmlValueAdmissionError {
    /// Returns the configured maximum input UTF-16 units.
    pub fn capacity(self) -> usize {
        self.capacity
    }
    /// Returns the supplied value's UTF-16 units.
    pub fn required(self) -> usize {
        self.required
    }
}

impl fmt::Display for HtmlValueAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTML value requires {} UTF-16 units; capacity is {}",
            self.required, self.capacity
        )
    }
}
impl Error for HtmlValueAdmissionError {}

/// A concrete front door for independent common value interpretation.
///
/// Each operation admits its exact raw input length before scanning it. The
/// caller selects the finite capacity; zero admits missing and empty values.
/// There is no implicit default capacity. Exact integers, decimals and years
/// grow with admitted digit length; list cardinality is bounded by admitted
/// input. The raw string is shared, and derived values leave it unchanged.
///
/// This input-unit admission is not parser resource accounting or cancellation,
/// deterministic work credits, an allocation-failure guarantee, or an RSS
/// bound. Tree parsing must use its own centralized request controls before
/// invoking private value algorithms. Attribute defaults and contextual
/// semantic operations are outside these common clauses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlValueInterpreter {
    max_input_code_units: usize,
}

impl HtmlValueInterpreter {
    /// Applies the legacy-color processing algorithm, producing opaque RGB.
    ///
    /// The original raw UTF-16 length is admitted before HTML trimming,
    /// code-point replacement or truncation. This selected clause defines no
    /// author grammar. This operation does not validate CSS color syntax or
    /// implement the separate color input-state algorithms.
    pub fn legacy_color(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlLegacyColor, HtmlLegacyColor>, HtmlValueAdmissionError> {
        let mut report = self.admit(raw, HtmlValueSyntax::LegacyColor)?;
        if let Some(raw) = raw {
            report.author = HtmlValueAuthorState::NotDefined;
            report.recovery = match color::parse_legacy_color(raw) {
                Ok(value) => HtmlValueRecoveryState::Value(value),
                Err(error) => {
                    let error = match error {
                        LegacyColorParseError::EmptyInput => HtmlValueSyntaxError::EmptyLegacyColor,
                        LegacyColorParseError::Transparent => {
                            HtmlValueSyntaxError::TransparentLegacyColor
                        }
                    };
                    HtmlValueRecoveryState::Failed(HtmlValueFailure::Syntax(error))
                }
            };
        }
        Ok(report)
    }

    /// Splits on ASCII whitespace, preserving token order and repetitions.
    ///
    /// The splitter does not certify attribute keywords, cardinality, unique
    /// membership or an attribute's author-conformance requirements.
    pub fn space_separated_tokens(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<
        HtmlValueReport<HtmlSpaceSeparatedTokens, HtmlSpaceSeparatedTokens>,
        HtmlValueAdmissionError,
    > {
        let mut report = self.admit(raw, HtmlValueSyntax::SpaceSeparatedTokens)?;
        if let Some(raw) = raw {
            report.author = HtmlValueAuthorState::NotDefined;
            report.recovery = HtmlValueRecoveryState::Value(tokens::split_space_tokens(raw));
        }
        Ok(report)
    }

    /// Splits on commas and trims only ASCII whitespace at field boundaries.
    ///
    /// Empty fields and repetitions follow Infra splitting; no attribute
    /// keyword, cardinality or external grammar conformance is inferred.
    pub fn comma_separated_tokens(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<
        HtmlValueReport<HtmlCommaSeparatedTokens, HtmlCommaSeparatedTokens>,
        HtmlValueAdmissionError,
    > {
        let mut report = self.admit(raw, HtmlValueSyntax::CommaSeparatedTokens)?;
        if let Some(raw) = raw {
            report.author = HtmlValueAuthorState::NotDefined;
            report.recovery = HtmlValueRecoveryState::Value(tokens::split_comma_tokens(raw));
        }
        Ok(report)
    }

    /// Requires an explicit finite maximum raw length for each operation.
    pub fn new(max_input_code_units: usize) -> Self {
        Self {
            max_input_code_units,
        }
    }

    /// Returns the caller-selected maximum raw UTF-16 length.
    pub fn max_input_code_units(self) -> usize {
        self.max_input_code_units
    }

    /// Checks the common finite raw-input capacity before owner-specific work.
    pub fn admit_attribute_raw(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<(), HtmlValueAdmissionError> {
        let required = raw.map_or(0, DocumentString::len);
        if required > self.max_input_code_units {
            return Err(HtmlValueAdmissionError {
                capacity: self.max_input_code_units,
                required,
            });
        }
        Ok(())
    }

    fn admit<A, R>(
        &self,
        raw: Option<&DocumentString>,
        syntax: HtmlValueSyntax,
    ) -> Result<HtmlValueReport<A, R>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw)?;
        Ok(HtmlValueReport {
            raw: raw.cloned(),
            syntax,
            author: HtmlValueAuthorState::Missing,
            recovery: HtmlValueRecoveryState::Missing,
        })
    }

    /// Checks authored integer syntax where defined and applies its recovery algorithm.
    pub fn integer(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlInteger, HtmlInteger>, HtmlValueAdmissionError> {
        let mut report = self.admit(raw, HtmlValueSyntax::Integer)?;
        if let Some(raw) = raw {
            report.author = numeric_author(numeric::validate_integer(raw));
            report.recovery = numeric_recovery(numeric::parse_integer(raw));
        }
        Ok(report)
    }

    /// Checks authored nonnegative integer syntax where defined and applies its recovery algorithm.
    pub fn nonnegative_integer(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<
        HtmlValueReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>,
        HtmlValueAdmissionError,
    > {
        let mut report = self.admit(raw, HtmlValueSyntax::NonnegativeInteger)?;
        if let Some(raw) = raw {
            report.author = numeric_author(numeric::validate_nonnegative_integer(raw));
            report.recovery = numeric_recovery(numeric::parse_nonnegative_integer(raw));
        }
        Ok(report)
    }

    /// Checks authored floating point number syntax where defined and applies its recovery algorithm.
    pub fn floating_point_number(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<
        HtmlValueReport<HtmlFloatingPointNumber, HtmlFloatingPointNumber>,
        HtmlValueAdmissionError,
    > {
        let mut report = self.admit(raw, HtmlValueSyntax::FloatingPointNumber)?;
        if let Some(raw) = raw {
            report.author = numeric_author(numeric::validate_floating_point_number(raw));
            report.recovery = numeric_recovery(numeric::parse_floating_point_number(raw));
        }
        Ok(report)
    }

    /// Checks authored floating point list syntax where defined and applies its recovery algorithm.
    pub fn floating_point_list(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<
        HtmlValueReport<HtmlFloatingPointList, HtmlFloatingPointList>,
        HtmlValueAdmissionError,
    > {
        let mut report = self.admit(raw, HtmlValueSyntax::FloatingPointList)?;
        if let Some(raw) = raw {
            report.author = numeric_author(numeric::validate_floating_point_list(raw));
            report.recovery =
                HtmlValueRecoveryState::Value(numeric::parse_floating_point_list(raw));
        }
        Ok(report)
    }

    /// Checks authored dimension syntax where defined and applies its recovery algorithm.
    pub fn dimension(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlDimension, HtmlDimension>, HtmlValueAdmissionError> {
        let mut report = self.admit(raw, HtmlValueSyntax::Dimension)?;
        if let Some(raw) = raw {
            report.author = HtmlValueAuthorState::NotDefined;
            report.recovery = numeric_recovery(numeric::parse_dimension(raw));
        }
        Ok(report)
    }

    /// Checks authored nonzero dimension syntax where defined and applies its recovery algorithm.
    pub fn nonzero_dimension(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlDimension, HtmlDimension>, HtmlValueAdmissionError> {
        let mut report = self.admit(raw, HtmlValueSyntax::NonzeroDimension)?;
        if let Some(raw) = raw {
            report.author = HtmlValueAuthorState::NotDefined;
            report.recovery = numeric_recovery(numeric::parse_nonzero_dimension(raw));
        }
        Ok(report)
    }

    /// Checks authored dimension list syntax where defined and applies its recovery algorithm.
    pub fn dimension_list(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlDimensionList, HtmlDimensionList>, HtmlValueAdmissionError>
    {
        let mut report = self.admit(raw, HtmlValueSyntax::DimensionList)?;
        if let Some(raw) = raw {
            report.author = HtmlValueAuthorState::NotDefined;
            report.recovery = HtmlValueRecoveryState::Value(numeric::parse_dimension_list(raw));
        }
        Ok(report)
    }

    /// Checks month author syntax and separately applies temporal processing.
    pub fn month(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlMonth, HtmlMonth>, HtmlValueAdmissionError> {
        let mut report = self.admit(raw, HtmlValueSyntax::Month)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_month(raw));
            report.recovery = temporal_recovery(temporal::parse_month(raw));
        }
        Ok(report)
    }

    /// Checks date author syntax and separately applies temporal processing.
    pub fn date(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlDate, HtmlDate>, HtmlValueAdmissionError> {
        let mut report = self.admit(raw, HtmlValueSyntax::Date)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_date(raw));
            report.recovery = temporal_recovery(temporal::parse_date(raw));
        }
        Ok(report)
    }

    /// Checks yearless date author syntax and separately applies temporal processing.
    pub fn yearless_date(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlYearlessDate, HtmlYearlessDate>, HtmlValueAdmissionError> {
        let mut report = self.admit(raw, HtmlValueSyntax::YearlessDate)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_yearless_date(raw));
            report.recovery = temporal_recovery(temporal::parse_yearless_date(raw));
        }
        Ok(report)
    }

    /// Checks time author syntax and separately applies temporal processing.
    pub fn time(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlTime, HtmlParsedTime>, HtmlValueAdmissionError> {
        let mut report = self.admit(raw, HtmlValueSyntax::Time)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_time(raw));
            report.recovery = temporal_recovery(temporal::parse_time(raw));
        }
        Ok(report)
    }

    /// Checks local date time author syntax and separately applies temporal processing.
    pub fn local_date_time(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlLocalDateTime, HtmlParsedLocalDateTime>, HtmlValueAdmissionError>
    {
        let mut report = self.admit(raw, HtmlValueSyntax::LocalDateTime)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_local_date_time(raw));
            report.recovery = temporal_recovery(temporal::parse_local_date_time(raw));
        }
        Ok(report)
    }

    /// Checks normalized local date time author syntax and separately applies temporal processing.
    /// Recovery uses the local-date-time algorithm, including nonnormalized spellings.
    pub fn normalized_local_date_time(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlLocalDateTime, HtmlParsedLocalDateTime>, HtmlValueAdmissionError>
    {
        let mut report = self.admit(raw, HtmlValueSyntax::NormalizedLocalDateTime)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_normalized_local_date_time(raw));
            report.recovery = temporal_recovery(temporal::parse_local_date_time(raw));
        }
        Ok(report)
    }

    /// Checks time zone offset author syntax and separately applies temporal processing.
    pub fn time_zone_offset(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlTimeZoneOffset, HtmlTimeZoneOffset>, HtmlValueAdmissionError>
    {
        let mut report = self.admit(raw, HtmlValueSyntax::TimeZoneOffset)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_time_zone_offset(raw));
            report.recovery = temporal_recovery(temporal::parse_time_zone_offset(raw));
        }
        Ok(report)
    }

    /// Checks global date time author syntax and separately applies temporal processing.
    pub fn global_date_time(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<
        HtmlValueReport<HtmlGlobalDateTime, HtmlParsedGlobalDateTime>,
        HtmlValueAdmissionError,
    > {
        let mut report = self.admit(raw, HtmlValueSyntax::GlobalDateTime)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_global_date_time(raw));
            report.recovery = temporal_recovery(temporal::parse_global_date_time(raw));
        }
        Ok(report)
    }

    /// Checks week author syntax and separately applies temporal processing.
    pub fn week(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlWeek, HtmlWeek>, HtmlValueAdmissionError> {
        let mut report = self.admit(raw, HtmlValueSyntax::Week)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_week(raw));
            report.recovery = temporal_recovery(temporal::parse_week(raw));
        }
        Ok(report)
    }

    /// Checks duration author syntax and separately applies temporal processing.
    pub fn duration(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlDuration, HtmlParsedDuration>, HtmlValueAdmissionError> {
        let mut report = self.admit(raw, HtmlValueSyntax::Duration)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_duration(raw));
            report.recovery = temporal_recovery(temporal::parse_duration(raw));
        }
        Ok(report)
    }

    /// Applies the source's date-or-time processing algorithm without inventing author syntax.
    pub fn date_or_time(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlDateOrTime, HtmlDateOrTime>, HtmlValueAdmissionError> {
        let mut report = self.admit(raw, HtmlValueSyntax::DateOrTime)?;
        if let Some(raw) = raw {
            report.author = HtmlValueAuthorState::NotDefined;
            report.recovery = temporal_recovery(temporal::parse_date_or_time(raw));
        }
        Ok(report)
    }

    /// Checks date-with-optional-time author syntax and retains the full common processing result.
    /// A recovered standalone time is retained; attribute-specific effective filtering is separate.
    pub fn date_with_optional_time(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlValueReport<HtmlDateWithOptionalTime, HtmlDateOrTime>, HtmlValueAdmissionError>
    {
        let mut report = self.admit(raw, HtmlValueSyntax::DateWithOptionalTime)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_date_with_optional_time(raw));
            report.recovery = temporal_recovery(temporal::parse_date_or_time(raw));
        }
        Ok(report)
    }

    /// Checks all ten time-element author alternatives and separately obtains
    /// the ordered machine-readable equivalent of a supplied exact value.
    ///
    /// None is missing standalone input, not an absent datetime attribute's
    /// captured child-text fallback. This operation does not check element
    /// descendants or that a value represents human-readable contents. The
    /// whole raw UTF-16 value is admitted before parsing; native processing uses
    /// ordinary allocation without a work, cancellation or physical-memory cap.
    pub fn time_element_value(
        &self,
        raw: Option<&DocumentString>,
    ) -> Result<
        HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>,
        HtmlValueAdmissionError,
    > {
        let mut report = self.admit(raw, HtmlValueSyntax::TimeElement)?;
        if let Some(raw) = raw {
            report.author = temporal_author(temporal::validate_time_element_value(raw));
            report.recovery = temporal_recovery(temporal::parse_time_element_value(raw));
        }
        Ok(report)
    }

    /// Assembles the same source report from admitted controlled native attempts.
    /// Numeric and temporal materialization remain with their actual owners.
    pub(crate) fn time_element_report_from_attempts(
        &self,
        raw: &DocumentString,
        author: Result<HtmlTimeElementValue, HtmlTemporalError>,
        recovery: Result<HtmlMachineReadableTime, HtmlTemporalError>,
    ) -> Result<
        HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>,
        HtmlValueAdmissionError,
    > {
        let mut report = self.admit(Some(raw), HtmlValueSyntax::TimeElement)?;
        report.author = temporal_author(author);
        report.recovery = temporal_recovery(recovery);
        Ok(report)
    }
}

fn numeric_failure(error: NumericParseError) -> HtmlValueFailure {
    match error {
        NumericParseError::InvalidSyntax => {
            HtmlValueFailure::Syntax(HtmlValueSyntaxError::InvalidNumericSyntax)
        }
        NumericParseError::Negative => {
            HtmlValueFailure::Syntax(HtmlValueSyntaxError::NegativeNonnegativeInteger)
        }
        NumericParseError::ZeroDimension => {
            HtmlValueFailure::Syntax(HtmlValueSyntaxError::ZeroNonzeroDimension)
        }
        NumericParseError::FloatingPointOverflow => {
            HtmlValueFailure::Representation(HtmlValueRepresentationError::FloatingPointOverflow)
        }
    }
}

fn numeric_author<T>(result: Result<T, NumericParseError>) -> HtmlValueAuthorState<T> {
    match result {
        Ok(value) => HtmlValueAuthorState::Valid(value),
        Err(error) => match numeric_failure(error) {
            HtmlValueFailure::Syntax(error) => HtmlValueAuthorState::Invalid(error),
            HtmlValueFailure::Representation(error) => HtmlValueAuthorState::Unrepresentable(error),
        },
    }
}

fn numeric_recovery<T>(result: Result<T, NumericParseError>) -> HtmlValueRecoveryState<T> {
    match result {
        Ok(value) => HtmlValueRecoveryState::Value(value),
        Err(error) => HtmlValueRecoveryState::Failed(numeric_failure(error)),
    }
}

fn temporal_author<T>(result: Result<T, HtmlTemporalError>) -> HtmlValueAuthorState<T> {
    match result {
        Ok(value) => HtmlValueAuthorState::Valid(value),
        Err(error) => HtmlValueAuthorState::Invalid(HtmlValueSyntaxError::Temporal(error)),
    }
}

fn temporal_recovery<T>(result: Result<T, HtmlTemporalError>) -> HtmlValueRecoveryState<T> {
    match result {
        Ok(value) => HtmlValueRecoveryState::Value(value),
        Err(error) => HtmlValueRecoveryState::Failed(HtmlValueFailure::Syntax(
            HtmlValueSyntaxError::Temporal(error),
        )),
    }
}
