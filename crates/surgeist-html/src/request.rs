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
        }
    }
}
impl Error for HtmlParseRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Capture(error) => Some(error),
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

/// One owned decoded document request with fresh cancellation capability.
/// This is configuration, not an admitted source or a successful parse result.
/// Requests cannot be cloned; an explicit retry constructs a fresh request.
#[derive(Debug)]
pub struct HtmlDocumentParseRequest {
    source: HtmlSource,
    limits: HtmlParseLimits,
    cancellation: HtmlParseCancellation,
    context: DocumentContext,
}
impl HtmlDocumentParseRequest {
    /// Selects the no-browsing-context operation: scripting is Disabled and
    /// declarative shadow roots are disallowed. No URL/origin is fabricated.
    pub fn offline(source: HtmlSource, limits: HtmlParseLimits) -> Self {
        Self {
            source,
            limits,
            cancellation: HtmlParseCancellation::new(),
            context: DocumentContext::Offline,
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
        })
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
}
impl HtmlFragmentParseRequest {
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
        })
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
