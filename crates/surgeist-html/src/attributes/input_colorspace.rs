//! Input colorspace content states and the independent Color-state author rule.
//! CSS parsing, conversion, color-well updates and live reflection are separate.

use super::*;

/// Conforming input colorspace keywords and their intrinsic content states.
/// LimitedSrgb is also the missing/invalid fallback. A native attribute proves
/// local spelling, not that its actual input is in the Color state. No CSS color
/// value or converted/serialized color is produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputColorSpace {
    LimitedSrgb,
    DisplayP3,
}

impl HtmlInputColorSpace {
    /// Returns the source's canonical conforming keyword.
    pub fn canonical_keyword(self) -> &'static str {
        match self {
            Self::LimitedSrgb => "limited-srgb",
            Self::DisplayP3 => "display-p3",
        }
    }

    /// Emits the canonical no-namespace attribute using ordinary allocation.
    /// Its independent Color-only author restriction remains unproved.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Colorspace.qualified_name(),
            self.canonical_keyword(),
        )
    }
}

enum_rule!(INPUT_COLORSPACE, HtmlInputColorSpace, Colorspace,
    Context::Elements(&[KnownHtmlElement::Input]), [25], "attr-input-colorspace",
    Some(HtmlInputColorSpace::LimitedSrgb), Some(HtmlInputColorSpace::LimitedSrgb), None,
    ["limited-srgb" => ("limited-srgb", HtmlInputColorSpace::LimitedSrgb),
     "display-p3" => ("display-p3", HtmlInputColorSpace::DisplayP3)]);

pub(super) fn colorspace_keyword(index: usize) -> Option<(&'static str, HtmlInputColorSpace)> {
    INPUT_COLORSPACE
        .keywords
        .get(index)
        .map(|row| (row.spelling, row.state))
}
pub(super) fn colorspace_invalid_default() -> HtmlInputColorSpace {
    INPUT_COLORSPACE
        .invalid
        .expect("source-defined colorspace invalid default")
}
pub(super) fn colorspace_default() -> HtmlInputColorSpace {
    INPUT_COLORSPACE
        .missing
        .expect("source-defined colorspace default")
}

impl HtmlValueInterpreter {
    /// Interprets local colorspace content through the shared enumeration owner.
    /// Missing/invalid select LimitedSrgb while recognition and author spelling
    /// remain distinct. Present applicable content leaves the independent
    /// Color-only author presence rule Unchecked; no input state is inferred.
    ///
    /// Entire original raw UTF-16 admission precedes expanded-name applicability.
    /// Reports use ordinary shared allocation plus the existing per-value cap,
    /// without work, cancellation or physical-memory accounting. This produces
    /// no CSS parsing/conversion, color-well update, picker or live IDL outcome.
    pub fn input_colorspace(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlInputColorSpace>, HtmlValueAdmissionError> {
        let mut report = self.enumerated(&INPUT_COLORSPACE, element, raw)?;
        if report.applicability == HtmlAttributeApplicability::Applicable && raw.is_some() {
            report.context_author = HtmlAttributeContextAuthorState::Unchecked;
        }
        Ok(report)
    }

    /// Observes colorspace with an explicitly supplied actual input type state.
    /// This ordinary state input can come from input_type; it is not a captured
    /// subject/revision certificate. Present Color content satisfies the selected
    /// presence rule; other states prohibit its presence independently of spelling.
    /// Missing imposes no presence error. Intrinsic content states remain intact,
    /// including fallback or DisplayP3 on a forbidden state, without applying any
    /// color effects there. Wrong roles retain state/raw evidence but are Unapplied.
    /// Resource and effect limits are the same as input_colorspace.
    pub fn input_colorspace_for_input(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        state: HtmlInputType,
    ) -> Result<HtmlAttributeReport<HtmlInputColorSpace>, HtmlValueAdmissionError> {
        let mut report = self.input_colorspace(element, raw)?;
        report.supplied_context = SuppliedContext::Input(state);
        if report.applicability == HtmlAttributeApplicability::Applicable && raw.is_some() {
            report.context_author = if state == HtmlInputType::Color {
                HtmlAttributeContextAuthorState::Conforming
            } else {
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                )
            };
        }
        Ok(report)
    }
}
