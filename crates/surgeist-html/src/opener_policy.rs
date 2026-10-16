//! Pinned HTML opener-policy language values and the initial immutable record.
//! HTML877d raw64771–64827. No header parsing or policy effects occur here.

use crate::DocumentString;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlOpenerPolicyValue {
    UnsafeNone,
    SameOriginAllowPopups,
    SameOrigin,
    SameOriginPlusCoep,
    NoopenerAllowPopups,
}

impl HtmlOpenerPolicyValue {
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

    pub const fn value(&self) -> HtmlOpenerPolicyValue {
        self.value
    }
    pub const fn reporting_endpoint(&self) -> Option<&DocumentString> {
        self.reporting_endpoint.as_ref()
    }
    pub const fn report_only_value(&self) -> HtmlOpenerPolicyValue {
        self.report_only_value
    }
    pub const fn report_only_reporting_endpoint(&self) -> Option<&DocumentString> {
        self.report_only_reporting_endpoint.as_ref()
    }
}
