//! Document-owned native Meta processing history and explicit service boundaries.

use crate::DocumentString;
use surgeist_html::HtmlReferrerPolicy;

/// Last completed theme selection at this publication, independently qualified
/// from Meta processing history. Current-tree interpretation requires a complete
/// Source snapshot; `Known` does not certify a paused Source's unfinished tree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DomMetaThemeColorState {
    /// The original producer has not supplied this selection.
    Unavailable,
    /// A completed selection; `None` means it found no valid candidate.
    Known(Option<surgeist_color::Color>),
}

/// Last completed color-scheme selection and its original CSS input/context.
/// Current-tree interpretation requires a complete Source snapshot.
#[derive(Clone, Copy, Debug)]
pub enum DomMetaColorSchemeState<'a> {
    Unavailable,
    Known(Option<&'a surgeist_css::CssColorSchemeParseReport>),
}

/// The CSS owner seals the report; this receipt records its admitted backing.
#[derive(Clone, Debug)]
pub(crate) struct SelectedColorScheme {
    pub report: std::sync::Arc<surgeist_css::CssColorSchemeParseReport>,
    pub bytes: usize,
    pub units: usize,
}
impl PartialEq for SelectedColorScheme {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.report, &other.report)
    }
}

/// Established native Meta history, captured with its actual Document revision.
/// This is processing history, rather than a summary of currently attached Meta nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomMetaDocumentState {
    pub(crate) pragma_language: Option<DocumentString>,
    pub(crate) will_declaratively_refresh: bool,
}
impl DomMetaDocumentState {
    pub(crate) const fn native() -> Self {
        Self {
            pragma_language: None,
            will_declaratively_refresh: false,
        }
    }
    /// Exact first token established by a successful content-language insertion.
    pub fn pragma_language(&self) -> Option<&DocumentString> {
        self.pragma_language.as_ref()
    }
    /// The source flag set before the selected Do-nothing refresh action.
    /// It does not establish a timer, task, navigation or resource request.
    pub const fn will_declaratively_refresh(&self) -> bool {
        self.will_declaratively_refresh
    }
}

/// Canonical Document policy-container state supported by this owner.
/// Parsed policy history remains unavailable until its producer supplies it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDocumentPolicyContainer {
    pub(crate) referrer: Option<HtmlReferrerPolicy>,
}
impl DomDocumentPolicyContainer {
    pub(crate) const fn native() -> Self {
        Self {
            referrer: Some(surgeist_html::default_document_referrer_policy()),
        }
    }
    pub(crate) const fn unavailable() -> Self {
        Self { referrer: None }
    }
    /// Current policy-container referrer policy; `None` is unavailable history.
    /// The native initial policy is distinct from an attribute's empty fallback.
    pub const fn referrer_policy(self) -> Option<HtmlReferrerPolicy> {
        self.referrer
    }
}

/// A reached native Meta operation whose real dependency is not admitted.
/// Such refusal aborts the entire canonical batch before publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomMetaError {
    /// The actual Document has no supplied processing history.
    HistoryUnavailable,
    /// The preferred stylesheet-set operation requires its actual CSSOM owner.
    DefaultStyleUnsupported,
    /// The reached CSP parser and policy enforcement are not supplied.
    ContentSecurityPolicyUnsupported,
    /// Media beyond HTML's empty/ASCII-whitespace match needs actual matching facts.
    ThemeColorMediaUnavailable,
    ThemeColorProtocol(surgeist_css::CssColorProtocolError),
    ThemeColorUnavailable(surgeist_css::CssColorUnavailable),
    ThemeColorUnsupported(surgeist_css::CssColorUnsupported),
    ThemeColorFacts(surgeist_css::CssColorFactError),
    ThemeColorNumerical {
        error: surgeist_color::ColorError,
        input: surgeist_css::CssColorInputRange,
    },
    ThemeColorScratchLimit {
        limit: usize,
    },
    /// No caller-selected finite CSS property processing allowance is present.
    ColorSchemeProcessingUnavailable,
    ColorSchemeLimit(&'static str),
    CssProcessingProtocol,
    /// Cumulative current-batch Meta processing exhausted its caller allowance.
    WorkLimit {
        /// Caller-selected cumulative current-batch work allowance.
        limit: u64,
    },
}

/// Rebuild a producer fact from actual captured history, never current Meta nodes.
pub(crate) fn language_fallback(
    metadata: &crate::DomDocumentMetadata,
    document: surgeist_html_semantics::HtmlCaptureSubjectId,
) -> surgeist_html_semantics::HtmlDocumentLanguageFallback {
    use surgeist_html_semantics::{
        HtmlDocumentLanguageFallback, HtmlPragmaLanguage, HtmlProtocolLanguage,
    };
    let pragma = match metadata.meta_state() {
        None => HtmlPragmaLanguage::Unavailable,
        Some(state) => match state.pragma_language() {
            None => HtmlPragmaLanguage::Absent,
            Some(language) => HtmlPragmaLanguage::Tag(language.clone()),
        },
    };
    HtmlDocumentLanguageFallback::new(document, pragma, HtmlProtocolLanguage::Unavailable)
}
