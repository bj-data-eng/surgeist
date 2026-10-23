//! Document-owned native Meta processing history and explicit service boundaries.

use crate::DocumentString;
use std::sync::Arc;
use surgeist_html::{HtmlCspArtifactLayout, HtmlCspAssociation, HtmlReferrerPolicy};

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

/// Source-qualified knowledge of this Document's CSP policy container.
/// Absence of protocol qualification is never an empty-policy assertion.
#[derive(Clone, Copy, Debug)]
pub enum DomDocumentCspState<'a> {
    /// A real native constructor established the initial empty policy container.
    ConstructorInitialEmpty,
    /// No qualified policy association was supplied for this Document.
    Unavailable,
    /// An immutable original association transferred to this destination owner.
    Associated(&'a HtmlCspAssociation),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CspState {
    ConstructorInitialEmpty,
    Unavailable,
    Associated(SelectedCsp),
}

impl CspState {
    pub(crate) fn view(&self) -> DomDocumentCspState<'_> {
        match self {
            Self::ConstructorInitialEmpty => DomDocumentCspState::ConstructorInitialEmpty,
            Self::Unavailable => DomDocumentCspState::Unavailable,
            Self::Associated(value) => DomDocumentCspState::Associated(&value.0.association),
        }
    }
}

/// The complete producer receipt includes native graph storage and disposal.
/// Cloning metadata shares this immutable value, never the mutable issuer.
#[derive(Clone, Debug)]
pub(crate) struct SelectedCsp(Arc<CspAssociation>);
#[derive(Debug)]
struct CspAssociation {
    association: HtmlCspAssociation,
    layout: HtmlCspArtifactLayout,
}
impl PartialEq for SelectedCsp {
    fn eq(&self, other: &Self) -> bool {
        let left = &self.0.association;
        let right = &other.0.association;
        left.document() == right.document()
            && left.generation() == right.generation()
            && left.policy().id() == right.policy().id()
            && left.policy().revision() == right.policy().revision()
            && left.latest().map(|value| value.occurrence())
                == right.latest().map(|value| value.occurrence())
            && left.origin() == right.origin()
    }
}
impl Eq for SelectedCsp {}
impl SelectedCsp {
    pub(crate) fn new(association: HtmlCspAssociation, layout: HtmlCspArtifactLayout) -> Self {
        Self(Arc::new(CspAssociation {
            association,
            layout,
        }))
    }
    pub(crate) fn wrapper_bytes() -> usize {
        // The protocol receipt already includes the inline association.
        size_of::<CspAssociation>() - size_of::<HtmlCspAssociation>()
    }
    pub(crate) fn bytes(&self) -> Option<usize> {
        self.0
            .layout
            .storage()
            .bytes()
            .checked_add(Self::wrapper_bytes())
    }
    pub(crate) fn units(&self) -> usize {
        self.0.layout.storage().utf16_units()
    }
}

/// Canonical Document policy-container state supported by this owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomDocumentPolicyContainer {
    pub(crate) referrer: Option<HtmlReferrerPolicy>,
    pub(crate) csp: CspState,
}
impl DomDocumentPolicyContainer {
    pub(crate) const fn native() -> Self {
        Self {
            referrer: Some(surgeist_html::default_document_referrer_policy()),
            csp: CspState::ConstructorInitialEmpty,
        }
    }
    pub(crate) const fn unavailable() -> Self {
        Self {
            referrer: None,
            csp: CspState::Unavailable,
        }
    }
    /// Current referrer policy; `None` is unavailable history.
    pub const fn referrer_policy(&self) -> Option<HtmlReferrerPolicy> {
        self.referrer
    }
    /// Qualified original CSP knowledge, independent of current Meta nodes.
    pub fn csp(&self) -> DomDocumentCspState<'_> {
        self.csp.view()
    }
    pub(crate) fn csp_bytes(&self) -> Option<usize> {
        match &self.csp {
            CspState::Associated(value) => value.bytes(),
            _ => Some(0),
        }
    }
    pub(crate) fn csp_units(&self) -> usize {
        match &self.csp {
            CspState::Associated(value) => value.units(),
            _ => 0,
        }
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
    ThemeColorUnavailable(surgeist_css::CssColorUnavailable),
    ThemeColorUnsupported(surgeist_css::CssColorUnsupported),
    ThemeColorFacts(surgeist_css::CssColorFactError),
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
