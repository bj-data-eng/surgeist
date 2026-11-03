//! Pinned HTML opener-policy language values and the initial immutable record.
//! HTML877d raw64771–64827. No header parsing or policy effects occur here.

use crate::internal::values::DocumentString;

/// An opener policy value of HTML877d #cross-origin-opener-policies, which lets a document
/// navigated to in a top-level browsing context force a new top-level browsing context and group.
/// The set is the five values the pinned edition lists; the type records the value only and applies
/// no navigation or grouping effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlOpenerPolicyValue {
    /// `unsafe-none`, the default: the document shares its predecessor's
    /// top-level browsing context unless the predecessor specified a different
    /// opener policy.
    UnsafeNone,
    /// `same-origin-allow-popups`: forces a new top-level browsing context
    /// unless the predecessor specified the same opener policy and is same
    /// origin.
    SameOriginAllowPopups,
    /// `same-origin`: behaves as `same-origin-allow-popups`, and additionally
    /// requires auxiliary browsing contexts to hold same-origin documents with
    /// the same opener policy, or they appear closed to the opener.
    SameOrigin,
    /// `same-origin-plus-COEP`: behaves as `same-origin` and sets the new
    /// group's cross-origin isolation mode to logical or concrete. No header
    /// sets it directly; it results from `same-origin` combined with a
    /// compatible embedder policy.
    SameOriginPlusCoep,
    /// `noopener-allow-popups`: forces a new top-level browsing context
    /// regardless of the predecessor, severing the opener relationship without
    /// creating a security boundary between same-origin documents.
    NoopenerAllowPopups,
}

impl HtmlOpenerPolicyValue {
    /// The value's exact spelling in the pinned edition, including the capital
    /// letters of `same-origin-plus-COEP`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsafeNone => "unsafe-none",
            Self::SameOriginAllowPopups => "same-origin-allow-popups",
            Self::SameOrigin => "same-origin",
            Self::SameOriginPlusCoep => "same-origin-plus-COEP",
            Self::NoopenerAllowPopups => "noopener-allow-popups",
        }
    }
}

/// Actual four-field opener-policy record in the admitted initial domain.
/// Copying a source-selected parent policy uses Clone, not a fresh default.
/// This value establishes no Document provenance or opener/group transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlOpenerPolicy {
    value: HtmlOpenerPolicyValue,
    reporting_endpoint: Option<DocumentString>,
    report_only_value: HtmlOpenerPolicyValue,
    report_only_reporting_endpoint: Option<DocumentString>,
}

impl HtmlOpenerPolicy {
    /// A new record has both values unsafe-none and both endpoints null.
    /// A native no-opener relationship does not select noopener-allow-popups.
    pub const fn initial() -> Self {
        Self {
            value: HtmlOpenerPolicyValue::UnsafeNone,
            reporting_endpoint: None,
            report_only_value: HtmlOpenerPolicyValue::UnsafeNone,
            report_only_reporting_endpoint: None,
        }
    }

    /// The enforced opener policy value; `unsafe-none` in the initial record.
    pub const fn value(&self) -> HtmlOpenerPolicyValue {
        self.value
    }
    /// The reporting endpoint for the enforced value, or `None` for the
    /// standard's null. An endpoint keeps its exact UTF-16 code units.
    pub const fn reporting_endpoint(&self) -> Option<&DocumentString> {
        self.reporting_endpoint.as_ref()
    }
    /// The report-only opener policy value; `unsafe-none` in the initial record.
    pub const fn report_only_value(&self) -> HtmlOpenerPolicyValue {
        self.report_only_value
    }
    /// The reporting endpoint for the report-only value, or `None` for null.
    pub const fn report_only_reporting_endpoint(&self) -> Option<&DocumentString> {
        self.report_only_reporting_endpoint.as_ref()
    }
}
