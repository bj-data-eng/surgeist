//! Progress content-number author constraints from #the-progress-element.
//! The captured Progress query owns determinate state and effective quantities.

use super::*;
use crate::numeric::{
    HtmlScientificDecimalKey, ScientificDecimalKeyError, strict_scientific_decimal_key,
};
use crate::{HtmlFloatingPointNumber, HtmlValueAuthorState, HtmlValueReport, HtmlValueSyntaxError};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

/// An individual content-number requirement, independently of the upper bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlProgressNumberAuthorError {
    Syntax(HtmlValueSyntaxError),
    NegativeValue,
    NonpositiveMaximum,
}

/// Strict grammar and the selected slot's sign requirement. Value conformance
/// here does not certify the separate value-versus-maximum requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlProgressNumberAuthorState {
    Missing,
    Conforming,
    Nonconforming(HtmlProgressNumberAuthorError),
    /// Exact mathematical keys exceeded checked native representation bounds.
    /// This is neither invalid grammar nor binary64 overflow.
    RepresentationUnavailable,
    Unapplied,
}

/// Why an applicable upper-bound comparison could not be established.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlProgressUpperBoundUnavailable {
    ValueSyntax,
    MaximumSyntax,
    Representation,
}

/// Exact mathematical author comparison against the present authored maximum,
/// or one only when that attribute is actually absent. No recovery defaults
/// participate. A maximum's separate positivity failure can coexist here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlProgressUpperBoundState {
    /// An applicable value attribute is absent, so no comparison is required.
    NotRequired,
    Conforming,
    ValueAboveMaximum,
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
    pub fn value(&self) -> &HtmlProgressNumberReport {
        &self.value
    }
    pub fn maximum(&self) -> &HtmlProgressNumberReport {
        &self.maximum
    }
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
        element: &QualifiedName,
        value: Option<&Attribute>,
        maximum: Option<&Attribute>,
    ) -> Result<HtmlProgressAttributeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(value.map(Attribute::value))?;
        self.admit_attribute_raw(maximum.map(Attribute::value))?;
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
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
pub enum HtmlProgressNativeError {
    NegativeValue,
    NonpositiveMaximum,
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
                .map(|number| strict_scientific_decimal_key(&number.as_document_string()))
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
    pub fn value(&self) -> Option<HtmlFloatingPointNumber> {
        self.value
    }
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
                value.as_document_string(),
            ));
        }
        if let Some(maximum) = self.maximum {
            attributes.push(Attribute::new(
                KnownHtmlAttribute::Max.qualified_name(),
                maximum.as_document_string(),
            ));
        }
        attributes
    }
}
