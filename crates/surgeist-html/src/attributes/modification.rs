//! ins/del datetime content interpretation from HTML §4.7.3.
//!
//! The common date-or-time processing result remains authoritative. This
//! overlay selects only Date and Global as associated modification timestamps;
//! a recovered standalone Time is retained as evidence but supplies no timestamp.

use super::*;
use crate::{
    HtmlDate, HtmlDateOrTime, HtmlDateWithOptionalTime, HtmlParsedGlobalDateTime,
    HtmlValueRecoveryState, HtmlValueReport,
};

/// Why a present, applicable datetime supplies no modification timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlModificationTimestampAbsence {
    /// The common date-or-time processing algorithm failed.
    ParseFailed,
    /// The common algorithm recovered a Time rather than a Date or Global.
    RecoveredTimeOnly,
}

/// The source-defined associated timestamp, borrowed from the common result.
/// No host time zone is selected and a date is not converted to a moment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlModificationTimestamp<'a> {
    /// The `datetime` attribute is absent.
    Missing,
    /// The value recovered a Date, which is the timestamp.
    Date(&'a HtmlDate),
    /// Exact local components, UTC instant and explicit source offset.
    Global(&'a HtmlParsedGlobalDateTime),
    /// The attribute is present but supplies no timestamp. The payload says why.
    NoTimestamp(HtmlModificationTimestampAbsence),
    /// The rule does not apply to these names. The payload is the reason.
    Unapplied(HtmlAttributeApplicability),
}

/// Exact ins/del datetime observation, author check and common recovery.
///
/// The raw value is admitted through the interpreter's per-value UTF-16 limit.
/// Native records and processing results use ordinary Rust allocation; this
/// operation makes no controlled-memory, cancellation or host-time promise.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlModificationDatetimeReport {
    attribute: HtmlAttributeReport<()>,
    common: Option<HtmlValueReport<HtmlDateWithOptionalTime, HtmlDateOrTime>>,
}

impl HtmlModificationDatetimeReport {
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
    /// Whether the rule applies to these expanded names. This rule covers only HTML `ins` and
    /// `del` elements; `time` also carries a `datetime` attribute, which a separate rule covers.
    pub fn applicability(&self) -> HtmlAttributeApplicability {
        self.attribute.applicability()
    }
    /// The selected standard attribute, `datetime`.
    pub fn attribute(&self) -> KnownHtmlAttribute {
        self.attribute.attribute()
    }
    /// The original index row identifiers that this defining rule covers.
    pub fn inventory_rows(&self) -> &'static [u16] {
        self.attribute.inventory_rows()
    }
    /// The normative clause in the pinned edition that defines the attribute.
    pub fn source_clause(&self) -> &'static str {
        self.attribute.source_clause()
    }
    /// The UTF-16 code units in the raw value that was admitted, or zero when
    /// the attribute is absent.
    pub fn input_code_units(&self) -> usize {
        self.attribute.input_code_units()
    }
    /// Returns both strict authored evidence and exact common processing.
    /// None means the rule was unapplied; an applicable absent attribute has a
    /// common report with Missing author and recovery states.
    pub fn common(&self) -> Option<&HtmlValueReport<HtmlDateWithOptionalTime, HtmlDateOrTime>> {
        self.common.as_ref()
    }
    /// Applies only the modification clause's Date/Global filter. Author
    /// conformance remains independent in common().author().
    pub fn timestamp(&self) -> HtmlModificationTimestamp<'_> {
        use HtmlModificationTimestamp as Timestamp;
        let Some(common) = &self.common else {
            return Timestamp::Unapplied(self.applicability());
        };
        match common.recovery() {
            HtmlValueRecoveryState::Missing => Timestamp::Missing,
            HtmlValueRecoveryState::Value(HtmlDateOrTime::Date(date)) => Timestamp::Date(date),
            HtmlValueRecoveryState::Value(HtmlDateOrTime::Global(global)) => {
                Timestamp::Global(global)
            }
            HtmlValueRecoveryState::Value(HtmlDateOrTime::Time(_)) => {
                Timestamp::NoTimestamp(HtmlModificationTimestampAbsence::RecoveredTimeOnly)
            }
            HtmlValueRecoveryState::Failed(_) => {
                Timestamp::NoTimestamp(HtmlModificationTimestampAbsence::ParseFailed)
            }
        }
    }
}

impl HtmlValueInterpreter {
    /// Interprets datetime only on exact HTML ins/del expanded names.
    /// Raw admission precedes applicability; wrong names do not run a grammar.
    pub fn modification_datetime(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlModificationDatetimeReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let attribute = report(
            RuleMetadata {
                attribute: KnownHtmlAttribute::Datetime,
                rows: &[36],
                source: source!("attr-mod-datetime"),
                context: Context::Elements(&[KnownHtmlElement::Ins, KnownHtmlElement::Del]),
                vocabulary_source: None,
            },
            element,
            raw,
        );
        let common = if attribute.applicability() == HtmlAttributeApplicability::Applicable {
            Some(self.date_with_optional_time(raw.map(HtmlAttributeInput::value))?)
        } else {
            None
        };
        Ok(HtmlModificationDatetimeReport { attribute, common })
    }
}

impl HtmlDateWithOptionalTime {
    /// Serializes an intrinsically valid authored value into a datetime content
    /// attribute. This does not select an element or reparse the native value.
    pub fn into_modification_datetime_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Datetime.qualified_name(),
            self.to_document_string(),
        )
    }
}
