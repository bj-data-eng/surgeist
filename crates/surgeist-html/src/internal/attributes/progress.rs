//! Progress content-number author constraints from #the-progress-element.
//! The captured Progress query owns determinate state and effective quantities.

use super::*;
use crate::internal::microsyntax::numeric::{
    HtmlScientificDecimalKey, ScientificDecimalKeyError, strict_scientific_decimal_key,
};
use crate::{HtmlFloatingPointNumber, HtmlValueAuthorState, HtmlValueReport, HtmlValueSyntaxError};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

/// An individual content-number requirement, independently of the upper bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlProgressNumberAuthorError {
    /// The attribute's text is not a valid floating-point number; the payload is the first syntax
    /// failure.
    Syntax(HtmlValueSyntaxError),
    /// The `value` attribute is a valid number below zero. The standard requires it to be zero or
    /// greater (HTML877d #the-progress-element).
    NegativeValue,
    /// The `max` attribute is a valid number that is zero or below. The standard requires it to be
    /// greater than zero.
    NonpositiveMaximum,
}

impl fmt::Display for HtmlProgressNumberAuthorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Syntax(_) => "progress attribute is not a valid floating-point number",
            Self::NegativeValue => "progress value attribute is negative",
            Self::NonpositiveMaximum => "progress max attribute is not greater than zero",
        })
    }
}
impl Error for HtmlProgressNumberAuthorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Syntax(error) => Some(error),
            _ => None,
        }
    }
}

/// Strict grammar and the selected slot's sign requirement. Value conformance
/// here does not certify the separate value-versus-maximum requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlProgressNumberAuthorState {
    /// The attribute is absent, which is conforming for both `value` and `max`.
    Missing,
    /// The attribute is present, a valid floating-point number, and satisfies its sign requirement.
    Conforming,
    /// The attribute is present but violates the grammar or its sign requirement; the payload names
    /// which.
    Nonconforming(HtmlProgressNumberAuthorError),
    /// Exact mathematical keys exceeded checked native representation bounds.
    /// This is neither invalid grammar nor binary64 overflow.
    RepresentationUnavailable,
    /// The rule does not apply because the element is not an HTML `progress` or the supplied
    /// attribute is not one the rule covers.
    Unapplied,
}

/// Why an applicable upper-bound comparison could not be established.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlProgressUpperBoundUnavailable {
    /// The `value` text is not a valid number, so the comparison cannot be made.
    ValueSyntax,
    /// The `max` text is not a valid number, so the comparison cannot be made.
    MaximumSyntax,
    /// An exact key exceeds the checked representation bounds, so the comparison cannot be made.
    Representation,
}

/// Exact mathematical author comparison against the present authored maximum,
/// or one only when that attribute is actually absent. No recovery defaults
/// participate. A maximum's separate positivity failure can coexist here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlProgressUpperBoundState {
    /// An applicable value attribute is absent, so no comparison is required.
    NotRequired,
    /// The value does not exceed the maximum, or 1.0 when `max` is absent.
    Conforming,
    /// The value is greater than the maximum, or greater than 1.0 when `max` is absent, which the
    /// standard forbids.
    ValueAboveMaximum,
    /// The comparison could not be made; the payload gives the reason.
    Unavailable(HtmlProgressUpperBoundUnavailable),
    /// At least one required expanded-name role is not applicable.
    Unapplied,
}

/// One exact progress slot, its common attempts and independent sign check.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlProgressNumberReport {
    attribute: HtmlAttributeReport<()>,
    common: Option<HtmlValueReport<HtmlFloatingPointNumber, HtmlFloatingPointNumber>>,
    author: HtmlProgressNumberAuthorState,
}
impl HtmlProgressNumberReport {
    /// Metadata, exact raw presence/name/value and applicability. Its generic
    /// keyword author/recognition fields are Unapplied; use common() and author().
    pub fn attribute(&self) -> &HtmlAttributeReport<()> {
        &self.attribute
    }
    /// None means an unapplied rule; an applicable missing slot has a common
    /// report with Missing states. Binary64 representability is retained here.
    pub fn common(
        &self,
    ) -> Option<&HtmlValueReport<HtmlFloatingPointNumber, HtmlFloatingPointNumber>> {
        self.common.as_ref()
    }
    /// The grammar and sign verdict for this slot, independent of the value-versus-maximum
    /// comparison.
    pub fn author(&self) -> HtmlProgressNumberAuthorState {
        self.author
    }
}

/// Borrowed-input progress attribute interpretation with independently retained
/// grammar, mathematical author requirements and common binary64 recovery.
///
/// Each original raw slot obeys the interpreter's per-value UTF-16 cap. Reports
/// and exact keys use ordinary allocation, without a controlled-memory ledger,
/// work budget or cancellation guarantee. This does not certify task purpose,
/// element content, rendering or effective Progress state.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlProgressAttributeReport {
    value: HtmlProgressNumberReport,
    maximum: HtmlProgressNumberReport,
    upper_bound: HtmlProgressUpperBoundState,
}
impl HtmlProgressAttributeReport {
    /// The report for the `value` attribute.
    pub fn value(&self) -> &HtmlProgressNumberReport {
        &self.value
    }
    /// The report for the `max` attribute.
    pub fn maximum(&self) -> &HtmlProgressNumberReport {
        &self.maximum
    }
    /// The value-versus-maximum verdict. It is independent of the sign verdicts of the two slots,
    /// so a nonpositive `max` can coexist with a comparison result.
    pub fn upper_bound(&self) -> HtmlProgressUpperBoundState {
        self.upper_bound
    }
}

type ExactNumber = Result<Option<HtmlScientificDecimalKey>, ExactUnavailable>;
#[derive(Clone, Copy)]
enum ExactUnavailable {
    Syntax,
    Representation,
    Unapplied,
}

impl HtmlValueInterpreter {
    /// Interprets only exact HTML progress value/max roles. Both original raw
    /// slots are admitted before applicability; wrong roles run no grammar.
    /// A malformed present max is not an absent max or an authored default one.
    pub fn progress_attributes(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        value: Option<&impl crate::HtmlAttributeInput>,
        maximum: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlProgressAttributeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(value.map(HtmlAttributeInput::value))?;
        self.admit_attribute_raw(maximum.map(HtmlAttributeInput::value))?;
        let (value, value_key) = self.progress_number(element, value, false)?;
        let (maximum, maximum_key) = self.progress_number(element, maximum, true)?;
        let upper_bound = upper_bound(&value_key, &maximum_key);
        Ok(HtmlProgressAttributeReport {
            value,
            maximum,
            upper_bound,
        })
    }

    fn progress_number(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
        maximum: bool,
    ) -> Result<(HtmlProgressNumberReport, ExactNumber), HtmlValueAdmissionError> {
        let attribute = report(
            RuleMetadata {
                attribute: if maximum {
                    KnownHtmlAttribute::Max
                } else {
                    KnownHtmlAttribute::Value
                },
                rows: if maximum { &[93] } else { &[177] },
                source: if maximum {
                    source!("attr-progress-max")
                } else {
                    source!("attr-progress-value")
                },
                context: Context::Elements(&[KnownHtmlElement::Progress]),
                vocabulary_source: None,
            },
            element,
            raw,
        );
        let mut output = HtmlProgressNumberReport {
            attribute,
            common: None,
            author: HtmlProgressNumberAuthorState::Unapplied,
        };
        if output.attribute.applicability() != HtmlAttributeApplicability::Applicable {
            return Ok((output, Err(ExactUnavailable::Unapplied)));
        }
        let common = self.floating_point_number(output.attribute.raw())?;
        let (author, key) = match common.author() {
            HtmlValueAuthorState::Missing => (HtmlProgressNumberAuthorState::Missing, Ok(None)),
            HtmlValueAuthorState::Invalid(error) => (
                HtmlProgressNumberAuthorState::Nonconforming(
                    HtmlProgressNumberAuthorError::Syntax(*error),
                ),
                Err(ExactUnavailable::Syntax),
            ),
            _ => match strict_scientific_decimal_key(
                output.attribute.raw().expect("present numeric grammar"),
            ) {
                Ok(key) => (sign_author(&key, maximum), Ok(Some(key))),
                Err(ScientificDecimalKeyError::RepresentationExhausted) => (
                    HtmlProgressNumberAuthorState::RepresentationUnavailable,
                    Err(ExactUnavailable::Representation),
                ),
                Err(ScientificDecimalKeyError::Syntax(_)) => (
                    HtmlProgressNumberAuthorState::Nonconforming(
                        HtmlProgressNumberAuthorError::Syntax(
                            HtmlValueSyntaxError::InvalidNumericSyntax,
                        ),
                    ),
                    Err(ExactUnavailable::Syntax),
                ),
            },
        };
        output.common = Some(common);
        output.author = author;
        Ok((output, key))
    }
}

fn sign_author(key: &HtmlScientificDecimalKey, maximum: bool) -> HtmlProgressNumberAuthorState {
    if maximum && !key.is_positive() {
        HtmlProgressNumberAuthorState::Nonconforming(
            HtmlProgressNumberAuthorError::NonpositiveMaximum,
        )
    } else if !maximum && !key.is_positive() && !key.is_zero() {
        HtmlProgressNumberAuthorState::Nonconforming(HtmlProgressNumberAuthorError::NegativeValue)
    } else {
        HtmlProgressNumberAuthorState::Conforming
    }
}

fn upper_bound(value: &ExactNumber, maximum: &ExactNumber) -> HtmlProgressUpperBoundState {
    use HtmlProgressUpperBoundState as State;
    use HtmlProgressUpperBoundUnavailable as Missing;
    let value = match value {
        Ok(None) => return State::NotRequired,
        Ok(Some(value)) => value,
        Err(ExactUnavailable::Unapplied) => return State::Unapplied,
        Err(ExactUnavailable::Syntax) => return State::Unavailable(Missing::ValueSyntax),
        Err(ExactUnavailable::Representation) => {
            return State::Unavailable(Missing::Representation);
        }
    };
    let one;
    let maximum = match maximum {
        Ok(Some(maximum)) => maximum,
        Ok(None) => {
            one = match strict_scientific_decimal_key(&DocumentString::from("1")) {
                Ok(one) => one,
                Err(_) => return State::Unavailable(Missing::Representation),
            };
            &one
        }
        Err(ExactUnavailable::Unapplied) => return State::Unapplied,
        Err(ExactUnavailable::Syntax) => return State::Unavailable(Missing::MaximumSyntax),
        Err(ExactUnavailable::Representation) => {
            return State::Unavailable(Missing::Representation);
        }
    };
    match value.compare(maximum) {
        Ok(Ordering::Greater) => State::ValueAboveMaximum,
        Ok(_) => State::Conforming,
        Err(_) => State::Unavailable(Missing::Representation),
    }
}

/// Rejection of a coupled finite native progress attribute set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlProgressNativeError {
    /// The `value` attribute is negative.
    NegativeValue,
    /// The `max` attribute is zero or negative.
    NonpositiveMaximum,
    /// The value is greater than the maximum, or greater than 1.0 when no maximum is given.
    ValueAboveMaximum,
    /// Canonical emitted numbers could not be represented as exact author keys.
    RepresentationUnavailable,
}
impl fmt::Display for HtmlProgressNativeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NegativeValue => "progress value must be nonnegative",
            Self::NonpositiveMaximum => "progress maximum must be positive",
            Self::ValueAboveMaximum => "progress value exceeds its authored upper bound",
            Self::RepresentationUnavailable => "exact progress author comparison unavailable",
        })
    }
}
impl Error for HtmlProgressNativeError {}

/// Conforming coupled value/max content attributes in the finite binary64
/// native subset. Absence is retained. This proves emitted number constraints,
/// not the progress element's task purpose, children, rendering or live state.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlProgressAttributes {
    value: Option<HtmlFloatingPointNumber>,
    maximum: Option<HtmlFloatingPointNumber>,
}
impl HtmlProgressAttributes {
    /// Checks the actual canonical decimal spellings and their exact mathematical
    /// constraints. A value with no maximum must not exceed one.
    pub fn new(
        value: Option<HtmlFloatingPointNumber>,
        maximum: Option<HtmlFloatingPointNumber>,
    ) -> Result<Self, HtmlProgressNativeError> {
        let key = |number: Option<HtmlFloatingPointNumber>| {
            number
                .map(|number| strict_scientific_decimal_key(&number.to_plain_decimal_string()))
                .transpose()
                .map_err(|_| HtmlProgressNativeError::RepresentationUnavailable)
        };
        let value_key = key(value)?;
        let maximum_key = key(maximum)?;
        if let Some(key) = &maximum_key
            && sign_author(key, true) != HtmlProgressNumberAuthorState::Conforming
        {
            return Err(HtmlProgressNativeError::NonpositiveMaximum);
        }
        if let Some(key) = &value_key
            && sign_author(key, false) != HtmlProgressNumberAuthorState::Conforming
        {
            return Err(HtmlProgressNativeError::NegativeValue);
        }
        match upper_bound(&Ok(value_key), &Ok(maximum_key)) {
            HtmlProgressUpperBoundState::ValueAboveMaximum => {
                return Err(HtmlProgressNativeError::ValueAboveMaximum);
            }
            HtmlProgressUpperBoundState::Unavailable(_)
            | HtmlProgressUpperBoundState::Unapplied => {
                return Err(HtmlProgressNativeError::RepresentationUnavailable);
            }
            _ => {}
        }
        Ok(Self { value, maximum })
    }
    /// The authored value, or `None` when the attribute is absent.
    pub fn value(&self) -> Option<HtmlFloatingPointNumber> {
        self.value
    }
    /// The authored maximum, or `None` when the attribute is absent.
    pub fn maximum(&self) -> Option<HtmlFloatingPointNumber> {
        self.maximum
    }
    /// Emits only present attributes, in value then max order. Reuses canonical
    /// native serialization; no grammar, relation or recovery is rerun here.
    pub fn into_attributes(self) -> Vec<Attribute> {
        let mut attributes = Vec::with_capacity(
            usize::from(self.value.is_some()) + usize::from(self.maximum.is_some()),
        );
        if let Some(value) = self.value {
            attributes.push(Attribute::new(
                KnownHtmlAttribute::Value.qualified_name(),
                value.to_plain_decimal_string(),
            ));
        }
        if let Some(maximum) = self.maximum {
            attributes.push(Attribute::new(
                KnownHtmlAttribute::Max.qualified_name(),
                maximum.to_plain_decimal_string(),
            ));
        }
        attributes
    }
}
