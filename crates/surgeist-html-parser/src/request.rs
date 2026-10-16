//! Checked decoded requests. Construction selects context and configuration;
//! source admission, preprocessing and parsing remain execution operations.

use crate::{
    HtmlCapture, HtmlCaptureDocumentId, HtmlCaptureError, HtmlCaptureFact, HtmlCapturedNodeKind,
    HtmlFragmentContext, HtmlParseCancellation, HtmlParseLimits, HtmlParserScriptingMode,
    HtmlScriptingStatus, HtmlSource,
};
use std::{error::Error, fmt};

/// Captured facts rejected during request selection, before parse execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlParseRequestError {
    Capture(HtmlCaptureError),
    Base(crate::HtmlBaseFactsError),
}
impl From<HtmlCaptureError> for HtmlParseRequestError {
    fn from(error: HtmlCaptureError) -> Self {
        Self::Capture(error)
    }
}
impl fmt::Display for HtmlParseRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Capture(error) => write!(f, "HTML parse request rejected: {error}"),
            Self::Base(error) => write!(f, "HTML parse request rejected: {error}"),
        }
    }
}
impl Error for HtmlParseRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Capture(error) => Some(error),
            Self::Base(error) => Some(error),
        }
    }
}

#[derive(Debug)]
enum DocumentContext {
    Offline,
    Captured {
        capture: HtmlCapture,
        document: HtmlCaptureDocumentId,
    },
}

impl DocumentContext {
    fn image_support(&self) -> crate::HtmlImageSupport {
        match self {
            Self::Offline => crate::HtmlImageSupport::Disabled,
            Self::Captured { capture, document } => match capture
                .node(document.subject())
                .expect("checked request document")
                .kind()
            {
                HtmlCapturedNodeKind::Document(facts) => facts.parser_creation_image_support(),
                _ => unreachable!("checked request document"),
            },
        }
    }
}
/// One owned decoded document request with fresh cancellation capability.
/// This is configuration, not an admitted source or a successful parse result.
/// Requests cannot be cloned; an explicit retry constructs a fresh request.
#[derive(Debug)]
pub struct HtmlDocumentParseRequest {
    source: HtmlSource,
    limits: HtmlParseLimits,
    cancellation: HtmlParseCancellation,
    context: DocumentContext,
    base_facts: Option<crate::HtmlParseBaseFacts>,
    associated: Option<std::sync::Arc<crate::construction::associated::Inputs>>,
}
impl HtmlDocumentParseRequest {
    /// Parse the owned decoded source into a complete immutable report.
    #[expect(
        clippy::result_large_err,
        reason = "returns the actual original request, stopped control and cleanup owner without allocating after a stop"
    )]
    pub fn parse(self) -> Result<crate::HtmlParseReport, crate::HtmlParseFailure> {
        crate::parser::tree_builder::parse::document(self)
    }
    /// Consumes the original configuration after borrowed execution ends.
    /// This move does not clone source backing or imply parse admission.
    pub(crate) fn into_parts(self) -> DocumentRequestParts {
        DocumentRequestParts {
            source: self.source,
            limits: self.limits,
            cancellation: self.cancellation,
            context: self.context,
            base_facts: self.base_facts,
            associated: self.associated,
        }
    }
    /// Selects the no-browsing-context operation: scripting is Disabled and
    /// declarative shadow roots are disallowed. No URL/origin is fabricated.
    pub fn offline(source: HtmlSource, limits: HtmlParseLimits) -> Self {
        Self {
            source,
            limits,
            cancellation: HtmlParseCancellation::new(),
            context: DocumentContext::Offline,
            base_facts: None,
            associated: None,
        }
    }
    /// Selects one document from the supplied immutable capture. Enabled
    /// scripting requires the unsupported Normal executor; unavailable status
    /// remains a typed missing fact. No source scan or parse admission occurs.
    pub fn captured(
        source: HtmlSource,
        capture: HtmlCapture,
        document: HtmlCaptureDocumentId,
        limits: HtmlParseLimits,
    ) -> Result<Self, HtmlParseRequestError> {
        capture.document_id(document.subject())?;
        let node = capture.node(document.subject())?;
        let HtmlCapturedNodeKind::Document(facts) = node.kind() else {
            unreachable!("capture.document_id checked immutable document kind")
        };
        match facts.scripting() {
            HtmlScriptingStatus::Disabled => {}
            HtmlScriptingStatus::Enabled => {
                return Err(HtmlCaptureError::UnsupportedExecution(
                    HtmlParserScriptingMode::Normal,
                )
                .into());
            }
            HtmlScriptingStatus::Unavailable => {
                return Err(HtmlCaptureError::MissingFact {
                    subject: Some(document.subject().clone()),
                    fact: HtmlCaptureFact::ScriptingStatus,
                }
                .into());
            }
        }
        Ok(Self {
            source,
            limits,
            cancellation: HtmlParseCancellation::new(),
            context: DocumentContext::Captured { capture, document },
            base_facts: None,
            associated: None,
        })
    }
    /// Supplies actual ordinary-document URL/encoding/CSP observations. This
    /// does not change fresh parser initialization into iframe-srcdoc.
    pub fn with_base_facts(
        mut self,
        facts: crate::HtmlParseBaseFacts,
    ) -> Result<Self, HtmlParseRequestError> {
        if facts.document().iframe_srcdoc() {
            return Err(HtmlParseRequestError::Base(
                crate::HtmlBaseFactsError::ContradictoryObservation,
            ));
        }
        self.base_facts = Some(facts);
        Ok(self)
    }
    pub fn base_facts(&self) -> Option<&crate::HtmlParseBaseFacts> {
        self.base_facts.as_ref()
    }
    /// Supplies immutable URL observations; admission/qualification is performed
    /// by the original execution control, never by an extra URL interpreter.
    pub fn with_hyperlink_facts(
        mut self,
        facts: Vec<surgeist_html_semantics::HtmlHyperlinkParseFacts>,
    ) -> Self {
        self.associated_mut().hyperlinks = facts;
        self
    }
    /// Attests the actual default URL/encoding context of this parse's separately
    /// created associated template-contents Document. No owner is created here.
    pub fn with_template_base_facts(
        mut self,
        facts: crate::HtmlParseBaseFacts,
    ) -> Result<Self, HtmlParseRequestError> {
        let document = facts.document();
        if document.iframe_srcdoc()
            || document
                .document_url()
                .is_some_and(|url| !url.matches_about_blank())
            || matches!(
                document.about_base_url(),
                crate::HtmlAboutBaseUrl::KnownRecord(_)
            )
        {
            return Err(HtmlParseRequestError::Base(
                crate::HtmlBaseFactsError::ContradictoryObservation,
            ));
        }
        self.associated_mut().template = Some(facts);
        Ok(self)
    }
    fn associated_mut(&mut self) -> &mut crate::construction::associated::Inputs {
        let inputs = self.associated.get_or_insert_with(|| {
            std::sync::Arc::new(crate::construction::associated::Inputs::default())
        });
        std::sync::Arc::get_mut(inputs).expect("unexecuted non-Clone request owns its input bank")
    }
    pub(crate) fn associated_inputs(
        &self,
    ) -> Option<&std::sync::Arc<crate::construction::associated::Inputs>> {
        self.associated.as_ref()
    }
    /// Exact supplied source, including its immutable identity and coordinates.
    pub fn source(&self) -> &HtmlSource {
        &self.source
    }
    /// Checked finite configuration; no execution usage is implied.
    pub fn limits(&self) -> &HtmlParseLimits {
        &self.limits
    }
    /// Clone this handle to request cancellation externally; there is no reset.
    pub fn cancellation(&self) -> &HtmlParseCancellation {
        &self.cancellation
    }
    /// This supported document boundary always selects Disabled scripting.
    pub fn scripting_mode(&self) -> HtmlParserScriptingMode {
        HtmlParserScriptingMode::Disabled
    }
    /// Fixed fresh-offline images-disabled policy; captures retain their own fact.
    pub fn image_support(&self) -> crate::HtmlImageSupport {
        self.context.image_support()
    }
    /// Document requests retain the standard's initial false permission.
    pub fn allow_declarative_shadow_roots(&self) -> bool {
        false
    }
    /// Selected immutable capture, or absent for the offline operation.
    pub fn capture(&self) -> Option<&HtmlCapture> {
        match &self.context {
            DocumentContext::Offline => None,
            DocumentContext::Captured { capture, .. } => Some(capture),
        }
    }
    /// Selected checked document identity, or absent for offline parsing.
    pub fn document(&self) -> Option<&HtmlCaptureDocumentId> {
        match &self.context {
            DocumentContext::Offline => None,
            DocumentContext::Captured { document, .. } => Some(document),
        }
    }
}

/// The request owner keeps source and context together across private report
/// assembly. Promotion belongs to the eventual complete public parse boundary.
pub(crate) struct DocumentRequestParts {
    source: HtmlSource,
    limits: HtmlParseLimits,
    cancellation: HtmlParseCancellation,
    context: DocumentContext,
    base_facts: Option<crate::HtmlParseBaseFacts>,
    associated: Option<std::sync::Arc<crate::construction::associated::Inputs>>,
}
impl DocumentRequestParts {
    pub(crate) fn associated_inputs(
        &self,
    ) -> Option<&std::sync::Arc<crate::construction::associated::Inputs>> {
        self.associated.as_ref()
    }
    pub(crate) fn release_associated_inputs(&mut self) {
        self.associated = None;
    }
    pub(crate) fn image_support(&self) -> crate::HtmlImageSupport {
        self.context.image_support()
    }
    pub(crate) fn base_facts(&self) -> Option<&crate::HtmlParseBaseFacts> {
        self.base_facts.as_ref()
    }
    pub(crate) fn source(&self) -> &HtmlSource {
        &self.source
    }
    pub(crate) fn limits(&self) -> &HtmlParseLimits {
        &self.limits
    }
    pub(crate) fn cancellation(&self) -> &HtmlParseCancellation {
        &self.cancellation
    }
    pub(crate) fn is_offline(&self) -> bool {
        matches!(self.context, DocumentContext::Offline)
    }
    pub(crate) fn capture(&self) -> Option<&HtmlCapture> {
        match &self.context {
            DocumentContext::Captured { capture, .. } => Some(capture),
            DocumentContext::Offline => None,
        }
    }
    pub(crate) fn document(&self) -> Option<&HtmlCaptureDocumentId> {
        match &self.context {
            DocumentContext::Captured { document, .. } => Some(document),
            DocumentContext::Offline => None,
        }
    }
}

/// One owned fragment request retaining the published checked context.
/// Requested and effective scripting modes remain distinct after resolution.
#[derive(Debug)]
pub struct HtmlFragmentParseRequest {
    source: HtmlSource,
    limits: HtmlParseLimits,
    cancellation: HtmlParseCancellation,
    context: HtmlFragmentContext,
    requested: HtmlParserScriptingMode,
    effective: HtmlParserScriptingMode,
    allow_declarative_shadow_roots: bool,
    base_facts: Option<std::sync::Arc<crate::HtmlCapturedBaseFacts>>,
}
impl HtmlFragmentParseRequest {
    /// Capture/revision-qualified attestation for this entry Document's parser
    /// creation environment. Target-only image support is insufficient.
    pub fn image_support(&self) -> crate::HtmlImageSupport {
        match self
            .context
            .capture()
            .node(self.context.target_document().subject())
            .expect("checked entry document")
            .kind()
        {
            HtmlCapturedNodeKind::Document(facts) => facts.parser_creation_image_support(),
            _ => unreachable!("checked entry document"),
        }
    }

    /// Parse in the actual retained context, returning no partial tree on failure.
    #[expect(
        clippy::result_large_err,
        reason = "returns the actual original request, stopped control and cleanup owner without allocating after a stop"
    )]
    pub fn parse(self) -> Result<crate::HtmlParseReport, crate::HtmlParseFailure> {
        crate::parser::tree_builder::parse::fragment(self)
    }
    /// Moves the original checked request after borrowed execution has ended.
    /// No source backing, capture, or cancellation authority is reconstructed.
    pub(crate) fn into_parts(self) -> FragmentRequestParts {
        FragmentRequestParts {
            source: self.source,
            limits: self.limits,
            cancellation: self.cancellation,
            context: self.context,
            requested: self.requested,
            effective: self.effective,
            allow_declarative_shadow_roots: self.allow_declarative_shadow_roots,
            base_facts: self.base_facts,
        }
    }
    /// Selects Inert mode and disallows declarative shadow roots. A disabled
    /// context still forces effective Disabled under the fragment algorithm.
    pub fn new(
        source: HtmlSource,
        context: HtmlFragmentContext,
        limits: HtmlParseLimits,
    ) -> Result<Self, HtmlParseRequestError> {
        Self::with_options(
            source,
            context,
            limits,
            HtmlParserScriptingMode::Inert,
            false,
        )
    }
    /// Resolves requested mode through the context's source-owned resolver.
    /// Enabled Fragment execution is unsupported; it is never downgraded.
    pub fn with_options(
        source: HtmlSource,
        context: HtmlFragmentContext,
        limits: HtmlParseLimits,
        requested: HtmlParserScriptingMode,
        allow_declarative_shadow_roots: bool,
    ) -> Result<Self, HtmlParseRequestError> {
        let effective = context.resolve_scripting_mode(requested)?;
        Ok(Self {
            source,
            limits,
            cancellation: HtmlParseCancellation::new(),
            context,
            requested,
            effective,
            allow_declarative_shadow_roots,
            base_facts: None,
        })
    }
    pub fn with_base_facts(
        mut self,
        facts: crate::HtmlCapturedBaseFacts,
    ) -> Result<Self, HtmlParseRequestError> {
        if facts.capture() != self.context.capture().identity() {
            return Err(HtmlParseRequestError::Base(
                crate::HtmlBaseFactsError::Capture(HtmlCaptureError::ForeignCapture),
            ));
        }
        if facts.host_revision() != self.context.capture().host_facts_revision() {
            return Err(HtmlParseRequestError::Base(
                crate::HtmlBaseFactsError::HostRevisionMismatch,
            ));
        }
        self.base_facts = Some(std::sync::Arc::new(facts));
        Ok(self)
    }
    pub fn base_facts(&self) -> Option<&crate::HtmlCapturedBaseFacts> {
        self.base_facts.as_deref()
    }
    /// Exact supplied source; request construction does not normalize it.
    pub fn source(&self) -> &HtmlSource {
        &self.source
    }
    /// Checked finite configuration, independent of capture admission limits.
    pub fn limits(&self) -> &HtmlParseLimits {
        &self.limits
    }
    /// Clone this fresh request's handle for safe external cancellation.
    pub fn cancellation(&self) -> &HtmlParseCancellation {
        &self.cancellation
    }
    /// Original checked selection, including target/context document identities.
    pub fn context(&self) -> &HtmlFragmentContext {
        &self.context
    }
    /// Mode explicitly selected for the fragment algorithm.
    pub fn requested_scripting_mode(&self) -> HtmlParserScriptingMode {
        self.requested
    }
    /// Mode resolved against the captured context's scripting status.
    pub fn effective_scripting_mode(&self) -> HtmlParserScriptingMode {
        self.effective
    }
    /// Explicit permission retained for the future source-owned tree builder.
    pub fn allow_declarative_shadow_roots(&self) -> bool {
        self.allow_declarative_shadow_roots
    }
}

/// Source-owning private assembly input, not a public parse-success proof.
pub(crate) struct FragmentRequestParts {
    source: HtmlSource,
    limits: HtmlParseLimits,
    cancellation: HtmlParseCancellation,
    context: HtmlFragmentContext,
    requested: HtmlParserScriptingMode,
    effective: HtmlParserScriptingMode,
    allow_declarative_shadow_roots: bool,
    base_facts: Option<std::sync::Arc<crate::HtmlCapturedBaseFacts>>,
}
impl FragmentRequestParts {
    pub(crate) fn base_facts(&self) -> Option<&crate::HtmlCapturedBaseFacts> {
        self.base_facts.as_deref()
    }
    pub(crate) fn source(&self) -> &HtmlSource {
        &self.source
    }
    pub(crate) fn limits(&self) -> &HtmlParseLimits {
        &self.limits
    }
    pub(crate) fn cancellation(&self) -> &HtmlParseCancellation {
        &self.cancellation
    }
    pub(crate) fn context(&self) -> &HtmlFragmentContext {
        &self.context
    }
    pub(crate) fn requested_scripting_mode(&self) -> HtmlParserScriptingMode {
        self.requested
    }
    pub(crate) fn effective_scripting_mode(&self) -> HtmlParserScriptingMode {
        self.effective
    }
    pub(crate) fn allow_declarative_shadow_roots(&self) -> bool {
        self.allow_declarative_shadow_roots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HtmlParseResource;
    use crate::parser::control::{ParseControl, ParseStop, WorkCategory};

    fn limits(source_units: u64, work_credits: u64) -> HtmlParseLimits {
        HtmlParseLimits::builder()
            .source_units(source_units)
            .buffered_utf16_units(0)
            .token_attributes(0)
            .auxiliary_entries(0)
            .managed_nodes(0)
            .managed_edges(0)
            .managed_attributes(0)
            .managed_utf16_units(0)
            .source_map_entries(0)
            .source_map_payload_units(0)
            .diagnostics(0)
            .diagnostic_payload_units(0)
            .context_effects(0)
            .open_elements(0)
            .tree_depth(0)
            .managed_capacity_bytes(0)
            .peak_transient_bytes(0)
            .atomic_layout_bytes(0)
            .work_credits(work_credits)
            .finish()
            .unwrap()
    }

    #[test]
    fn request_handle_cancels_the_existing_source_admission_owner() {
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8("x"), limits(0, 0));
        let mut control = ParseControl::new(*request.limits(), request.cancellation().clone());
        request.cancellation().cancel();
        assert_eq!(
            control.admit_source(request.source()),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            0
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            0
        );
        assert_eq!(control.stop(), Some(ParseStop::Cancelled));
    }

    #[test]
    fn checked_request_configuration_is_consumed_without_second_validation_or_source_scan() {
        let source = HtmlSource::utf8("😀\r\n");
        let request = HtmlDocumentParseRequest::offline(source.clone(), limits(6, 1));
        let mut control = ParseControl::new(*request.limits(), request.cancellation().clone());
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            0
        );
        control.admit_source(request.source()).unwrap();
        control.require_source(&source).unwrap();
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            6
        );
        assert_eq!(control.usage().work(WorkCategory::RecordVisit), 1);
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
        assert_eq!(control.stop(), None);
    }
}
