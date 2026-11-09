//! Checked decoded requests. Construction selects context and configuration;
//! source admission, preprocessing and parsing remain execution operations.

use crate::accounting::RequestAccounting;
use crate::{
    HtmlCapture, HtmlCaptureDocumentId, HtmlCaptureError, HtmlCaptureFact, HtmlCaptureSubjectKind,
    HtmlCapturedDocument, HtmlCapturedNodeKind, HtmlFragmentContext, HtmlParseCancellation,
    HtmlParseLimits, HtmlScriptingStatus, HtmlSource,
};
use std::{error::Error, fmt};
use surgeist_html::parsing::HtmlParserScriptingMode;

/// Private qualification selection. Every mode still executes terminal
/// promotion and both pools' fixed lifecycle; only optional reuse differs.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ValueSharingMode {
    Off,
    BankOnly,
    #[default]
    Combined,
}

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

fn validate_constructor_base_facts(
    facts: &crate::HtmlParseBaseFacts,
) -> Result<(), HtmlParseRequestError> {
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
    Ok(())
}

/// The selected document's facts, read once when a request is constructed so
/// that no later accessor repeats the lookup or can fail.
fn document_facts<'a>(
    capture: &'a HtmlCapture,
    document: &HtmlCaptureDocumentId,
) -> Result<&'a HtmlCapturedDocument, HtmlCaptureError> {
    match capture.node(document.subject())?.kind() {
        HtmlCapturedNodeKind::Document(facts) => Ok(facts),
        HtmlCapturedNodeKind::Element(_) => Err(not_a_document(HtmlCaptureSubjectKind::Element)),
        HtmlCapturedNodeKind::Fragment(_) => Err(not_a_document(HtmlCaptureSubjectKind::Fragment)),
        HtmlCapturedNodeKind::Text(_) => Err(not_a_document(HtmlCaptureSubjectKind::Text)),
        HtmlCapturedNodeKind::Comment(_) => Err(not_a_document(HtmlCaptureSubjectKind::Comment)),
        HtmlCapturedNodeKind::ProcessingInstruction(_) => Err(not_a_document(
            HtmlCaptureSubjectKind::ProcessingInstruction,
        )),
        HtmlCapturedNodeKind::Doctype(_) => Err(not_a_document(HtmlCaptureSubjectKind::Doctype)),
    }
}
fn not_a_document(actual: HtmlCaptureSubjectKind) -> HtmlCaptureError {
    HtmlCaptureError::WrongKind {
        expected: HtmlCaptureSubjectKind::Document,
        actual,
    }
}

#[derive(Debug)]
enum DocumentContext {
    Offline,
    Captured {
        capture: HtmlCapture,
        document: HtmlCaptureDocumentId,
        image_support: crate::HtmlImageSupport,
    },
}

/// The supplied observation banks, in exactly one of two states. They are
/// edited while the caller builds the request, and then shared once with
/// execution. A request never carries both an editable and a shared bank; an
/// edit that reaches a bank execution also holds changes nothing: the request
/// keeps the held bank and refuses to start a parse.
#[derive(Debug)]
enum InputBank {
    Building(Option<crate::construction::associated::Inputs>),
    Shared(Option<std::sync::Arc<crate::construction::associated::Inputs>>),
    /// An edit reached a shared bank that execution also holds. The bank is
    /// exactly the one held; the edit was dropped and is reported at entry.
    Refused(std::sync::Arc<crate::construction::associated::Inputs>),
}
impl Default for InputBank {
    fn default() -> Self {
        Self::Building(None)
    }
}
impl InputBank {
    /// Applies a builder's edit. A shared bank that nothing else holds reopens
    /// for the edit. One that execution also holds is neither edited nor
    /// copied: the request is marked refused, so the parse that would run on
    /// it fails with the ownership defect instead of silently ignoring the
    /// edit.
    fn edited(self, edit: impl FnOnce(&mut crate::construction::associated::Inputs)) -> Self {
        let mut bank = match self {
            Self::Building(bank) => bank.unwrap_or_default(),
            Self::Shared(None) => Default::default(),
            Self::Shared(Some(shared)) => match std::sync::Arc::try_unwrap(shared) {
                Ok(bank) => bank,
                Err(held) => return Self::Refused(held),
            },
            Self::Refused(held) => return Self::Refused(held),
        };
        edit(&mut bank);
        Self::Building(Some(bank))
    }
    fn is_refused(&self) -> bool {
        matches!(self, Self::Refused(_))
    }
    /// Ends editing: a supplied bank becomes the one execution shares.
    fn share(self) -> Self {
        match self {
            refused @ Self::Refused(_) => refused,
            other => Self::Shared(other.into_shared()),
        }
    }
    fn inputs(&self) -> Option<&crate::construction::associated::Inputs> {
        match self {
            Self::Building(bank) => bank.as_ref(),
            Self::Shared(bank) => bank.as_deref(),
            Self::Refused(held) => Some(held),
        }
    }
    fn shared(&self) -> Option<&std::sync::Arc<crate::construction::associated::Inputs>> {
        match self {
            Self::Building(_) => None,
            Self::Shared(bank) => bank.as_ref(),
            Self::Refused(held) => Some(held),
        }
    }
    fn into_shared(self) -> Option<std::sync::Arc<crate::construction::associated::Inputs>> {
        match self {
            Self::Building(bank) => bank.map(std::sync::Arc::new),
            Self::Shared(bank) => bank,
            Self::Refused(held) => Some(held),
        }
    }
}

impl DocumentContext {
    fn image_support(&self) -> crate::HtmlImageSupport {
        match self {
            Self::Offline => crate::HtmlImageSupport::Disabled,
            Self::Captured { image_support, .. } => *image_support,
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
    initial_policy_base: bool,
    associated: InputBank,
    accounting: RequestAccounting,
    #[cfg(test)]
    value_sharing: ValueSharingMode,
}
impl HtmlDocumentParseRequest {
    /// Charges the parse in the operation and scope of `accounting` instead of
    /// an operation of its own. See [`HtmlParseAccounting`](crate::HtmlParseAccounting)
    /// for the scopes the parse adds and the custody its report keeps.
    #[must_use]
    pub fn with_accounting(mut self, accounting: crate::HtmlParseAccounting) -> Self {
        self.accounting = RequestAccounting::host(accounting);
        self
    }
    /// Profiles the parse's own operation in `mode`; [`ProfileMode::Off`](surgeist_ledger::ProfileMode::Off) reads
    /// no clock and keeps no observation. The profile is read from the report
    /// or failure. A request that supplies accounting is profiled by the
    /// operation of its host instead.
    #[must_use]
    pub fn with_profile(mut self, mode: surgeist_ledger::ProfileMode) -> Self {
        self.accounting = RequestAccounting::profiled(mode);
        self
    }
    pub(crate) fn accounting(&self) -> &RequestAccounting {
        &self.accounting
    }
    pub(crate) fn initial_policy_base(&self) -> bool {
        self.initial_policy_base
    }
    #[cfg(test)]
    pub(crate) fn with_value_sharing(mut self, mode: ValueSharingMode) -> Self {
        self.value_sharing = mode;
        self
    }
    /// Parse the owned decoded source into a complete immutable report.
    #[expect(
        clippy::result_large_err,
        reason = "returns the actual original request, stopped control and cleanup owner without allocating after a stop"
    )]
    pub fn parse(self) -> Result<crate::HtmlParseReport, crate::HtmlParseFailure> {
        crate::parser::tree_builder::parse::document(self, None, 0)
    }
    /// Process reached original Meta CSP through the explicitly supplied service.
    /// The processor is borrowed only during this consuming ordinary parse.
    #[expect(
        clippy::result_large_err,
        reason = "returns the actual original request, controller and native error custody without allocating after a stop"
    )]
    pub fn parse_with_meta_csp<P: surgeist_html::security::csp::HtmlMetaCspProcessor>(
        self,
        processor: &mut P,
    ) -> Result<crate::HtmlParseReport, crate::HtmlParseCspFailure<P::Error>> {
        crate::meta_csp::parse(self, processor)
    }
    /// Qualifies this ordinary Base bank against the actual fresh constructor's
    /// initial policy. A changed policy never retargets this observation bank.
    pub fn with_initial_policy_base_facts(
        self,
        facts: crate::HtmlParseBaseFacts,
    ) -> Result<Self, HtmlParseRequestError> {
        let mut request = self.with_base_facts(facts)?;
        request.initial_policy_base = true;
        Ok(request)
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
            associated: self.associated.into_shared(),
            accounting: self.accounting,
            #[cfg(test)]
            value_sharing: self.value_sharing,
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
            initial_policy_base: false,
            associated: InputBank::default(),
            accounting: RequestAccounting::default(),
            #[cfg(test)]
            value_sharing: ValueSharingMode::Combined,
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
        let facts = document_facts(&capture, &document)?;
        let image_support = facts.parser_creation_image_support();
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
            context: DocumentContext::Captured {
                capture,
                document,
                image_support,
            },
            base_facts: None,
            initial_policy_base: false,
            associated: InputBank::default(),
            accounting: RequestAccounting::default(),
            #[cfg(test)]
            value_sharing: ValueSharingMode::Combined,
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
        self.initial_policy_base = false;
        Ok(self)
    }
    pub fn base_facts(&self) -> Option<&crate::HtmlParseBaseFacts> {
        self.base_facts.as_ref()
    }
    /// Supplies immutable URL observations; admission/qualification is performed
    /// by the original execution control, never by an extra URL interpreter.
    pub fn with_encoding_url_facts(
        mut self,
        facts: Vec<surgeist_html_semantics::HtmlEncodingUrlParseFacts>,
    ) -> Self {
        self.edit_associated(|bank| bank.encoding_urls = facts);
        self
    }
    /// Attests the actual default URL/encoding context of this parse's separately
    /// created associated template-contents Document. No owner is created here.
    pub fn with_template_base_facts(
        mut self,
        facts: crate::HtmlParseBaseFacts,
    ) -> Result<Self, HtmlParseRequestError> {
        validate_constructor_base_facts(&facts)?;
        self.edit_associated(|bank| bank.template = Some(facts));
        Ok(self)
    }
    fn edit_associated(&mut self, edit: impl FnOnce(&mut crate::construction::associated::Inputs)) {
        self.associated = std::mem::take(&mut self.associated).edited(edit);
    }
    pub(crate) fn associated_inputs(&self) -> Option<&crate::construction::associated::Inputs> {
        self.associated.inputs()
    }
    /// The bank execution shares with the session; present only after
    /// `share_associated_inputs`, which every parse owner calls on entry.
    pub(crate) fn shared_associated_inputs(
        &self,
    ) -> Option<&std::sync::Arc<crate::construction::associated::Inputs>> {
        self.associated.shared()
    }
    /// A test entry borrows its request, so it cannot share the banks for it
    /// and would skip supplied inputs without a trace; it refuses such a request.
    #[cfg(test)]
    pub(crate) fn require_inputs_reach_execution(&self) {
        assert!(
            self.associated.inputs().is_none() || self.associated.shared().is_some(),
            "a borrowed test entry needs a request whose input banks are shared"
        );
    }
    /// Ends the caller's editing of the supplied banks and shares them with
    /// execution. No builder is reachable afterwards: builders consume the
    /// request, and only the parse owners call this.
    pub(crate) fn share_associated_inputs(mut self) -> Self {
        self.associated = std::mem::take(&mut self.associated).share();
        self
    }
    /// An input edit reached a bank execution also holds, so no parse may run.
    pub(crate) fn input_bank_refused(&self) -> bool {
        self.associated.is_refused()
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
    accounting: RequestAccounting,
    #[cfg(test)]
    value_sharing: ValueSharingMode,
}
impl DocumentRequestParts {
    pub(crate) fn accounting(&self) -> &RequestAccounting {
        &self.accounting
    }
    #[cfg(test)]
    pub(crate) fn value_sharing(&self) -> ValueSharingMode {
        self.value_sharing
    }
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
    image_support: crate::HtmlImageSupport,
    base_facts: Option<std::sync::Arc<crate::HtmlCapturedBaseFacts>>,
    associated: InputBank,
    accounting: RequestAccounting,
    #[cfg(test)]
    value_sharing: ValueSharingMode,
}
impl HtmlFragmentParseRequest {
    /// Charges the parse in the operation and scope of `accounting` instead of
    /// an operation of its own; see [`HtmlDocumentParseRequest::with_accounting`].
    #[must_use]
    pub fn with_accounting(mut self, accounting: crate::HtmlParseAccounting) -> Self {
        self.accounting = RequestAccounting::host(accounting);
        self
    }
    /// Profiles the parse's own operation in `mode`; see
    /// [`HtmlDocumentParseRequest::with_profile`].
    #[must_use]
    pub fn with_profile(mut self, mode: surgeist_ledger::ProfileMode) -> Self {
        self.accounting = RequestAccounting::profiled(mode);
        self
    }
    pub(crate) fn accounting(&self) -> &RequestAccounting {
        &self.accounting
    }
    #[cfg(test)]
    pub(crate) fn with_value_sharing(mut self, mode: ValueSharingMode) -> Self {
        self.value_sharing = mode;
        self
    }
    /// Capture/revision-qualified attestation for this entry Document's parser
    /// creation environment. Target-only image support is insufficient.
    pub fn image_support(&self) -> crate::HtmlImageSupport {
        self.image_support
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
            image_support: self.image_support,
            base_facts: self.base_facts,
            associated: self.associated.into_shared(),
            accounting: self.accounting,
            #[cfg(test)]
            value_sharing: self.value_sharing,
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
        let image_support = document_facts(context.capture(), context.target_document())?
            .parser_creation_image_support();
        Ok(Self {
            source,
            limits,
            cancellation: HtmlParseCancellation::new(),
            context,
            requested,
            effective,
            allow_declarative_shadow_roots,
            image_support,
            base_facts: None,
            associated: InputBank::default(),
            accounting: RequestAccounting::default(),
            #[cfg(test)]
            value_sharing: ValueSharingMode::Combined,
        })
    }
    /// Execution-only observations, resolved against the actual reached Document.
    pub fn with_encoding_url_facts(
        mut self,
        facts: Vec<surgeist_html_semantics::HtmlEncodingUrlParseFacts>,
    ) -> Self {
        self.edit_associated(|bank| bank.encoding_urls = facts);
        self
    }
    /// Sealed immutable semantic query results for independently qualified
    /// captured Documents. Original execution admits scopes, duplicates and backing.
    pub fn with_encoding_url_contexts(
        mut self,
        contexts: Vec<surgeist_html_semantics::HtmlDocumentEncodingUrlContext>,
    ) -> Self {
        self.edit_associated(|bank| bank.captured = contexts);
        self
    }
    /// Actual external observations for this operation's auxiliary §13.4
    /// Document constructor. This does not create a URL/encoding identity.
    pub fn with_parser_document_base_facts(
        mut self,
        facts: crate::HtmlParseBaseFacts,
    ) -> Result<Self, HtmlParseRequestError> {
        validate_constructor_base_facts(&facts)?;
        self.edit_associated(|bank| bank.parser_document = Some(facts));
        Ok(self)
    }
    /// Actual observations for separately source-created inert template owners;
    /// existing captured inert owners use their own selected semantic context.
    pub fn with_template_base_facts(
        mut self,
        facts: crate::HtmlParseBaseFacts,
    ) -> Result<Self, HtmlParseRequestError> {
        validate_constructor_base_facts(&facts)?;
        self.edit_associated(|bank| bank.template = Some(facts));
        Ok(self)
    }
    fn edit_associated(&mut self, edit: impl FnOnce(&mut crate::construction::associated::Inputs)) {
        self.associated = std::mem::take(&mut self.associated).edited(edit);
    }
    pub(crate) fn associated_inputs(&self) -> Option<&crate::construction::associated::Inputs> {
        self.associated.inputs()
    }
    /// The bank execution shares with the session; present only after
    /// `share_associated_inputs`, which every parse owner calls on entry.
    pub(crate) fn shared_associated_inputs(
        &self,
    ) -> Option<&std::sync::Arc<crate::construction::associated::Inputs>> {
        self.associated.shared()
    }
    /// A test entry borrows its request, so it cannot share the banks for it
    /// and would skip supplied inputs without a trace; it refuses such a request.
    #[cfg(test)]
    pub(crate) fn require_inputs_reach_execution(&self) {
        assert!(
            self.associated.inputs().is_none() || self.associated.shared().is_some(),
            "a borrowed test entry needs a request whose input banks are shared"
        );
    }
    /// Ends the caller's editing of the supplied banks and shares them with
    /// execution. No builder is reachable afterwards: builders consume the
    /// request, and only the parse owners call this.
    pub(crate) fn share_associated_inputs(mut self) -> Self {
        self.associated = std::mem::take(&mut self.associated).share();
        self
    }
    /// An input edit reached a bank execution also holds, so no parse may run.
    pub(crate) fn input_bank_refused(&self) -> bool {
        self.associated.is_refused()
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
    image_support: crate::HtmlImageSupport,
    base_facts: Option<std::sync::Arc<crate::HtmlCapturedBaseFacts>>,
    associated: Option<std::sync::Arc<crate::construction::associated::Inputs>>,
    accounting: RequestAccounting,
    #[cfg(test)]
    value_sharing: ValueSharingMode,
}
impl FragmentRequestParts {
    pub(crate) fn accounting(&self) -> &RequestAccounting {
        &self.accounting
    }
    #[cfg(test)]
    pub(crate) fn value_sharing(&self) -> ValueSharingMode {
        self.value_sharing
    }
    pub(crate) fn associated_inputs(
        &self,
    ) -> Option<&std::sync::Arc<crate::construction::associated::Inputs>> {
        self.associated.as_ref()
    }
    pub(crate) fn release_associated_inputs(&mut self) {
        self.associated = None;
    }

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
    /// The entry Document's image support, resolved when the request was made.
    pub(crate) fn image_support(&self) -> crate::HtmlImageSupport {
        self.image_support
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HtmlParseResource;
    use crate::parser::control::{HtmlParseStop, HtmlParseWorkCategory, ParseControl};

    fn limits(source_units: u64, work_credits: u64) -> HtmlParseLimits {
        HtmlParseLimits::builder()
            .source_units(source_units)
            .buffered_utf16_units(0)
            .token_attributes(0)
            .auxiliary_entries(0)
            .active_formatting_entries(0)
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
            Err(HtmlParseStop::Cancelled)
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
        assert_eq!(control.stop(), Some(HtmlParseStop::Cancelled));
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
        assert_eq!(control.usage().work(HtmlParseWorkCategory::RecordVisit), 1);
        assert_eq!(
            control.usage().work(HtmlParseWorkCategory::InputUnitRead),
            0
        );
        assert_eq!(control.stop(), None);
    }

    #[test]
    fn input_banks_are_editable_until_shared_and_visible_through_one_view_each() {
        let none = HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), limits(6, 1));
        assert!(none.associated_inputs().is_none());
        assert!(none.shared_associated_inputs().is_none());
        let none = none.share_associated_inputs();
        assert!(none.associated_inputs().is_none());
        assert!(none.shared_associated_inputs().is_none());

        let supplied = HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), limits(6, 1))
            .with_encoding_url_facts(Vec::new());
        // Being built, the bank is editable and not yet shared with execution.
        assert!(supplied.associated_inputs().is_some());
        assert!(supplied.shared_associated_inputs().is_none());
        let shared = supplied.share_associated_inputs();
        let seen = shared.shared_associated_inputs().unwrap();
        assert!(std::ptr::eq(
            std::sync::Arc::as_ptr(seen),
            shared.associated_inputs().unwrap()
        ));
    }

    #[test]
    fn an_edit_of_a_bank_execution_also_holds_changes_nothing_and_fails_the_parse() {
        use crate::{HtmlParseEngineCondition, HtmlParseFailureReason};
        let shared = HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), limits(6, 1))
            .with_encoding_url_facts(Vec::new())
            .share_associated_inputs();
        let held = std::sync::Arc::clone(shared.shared_associated_inputs().unwrap());
        let refused = shared.with_encoding_url_facts(Vec::new());
        // Nothing was edited and nothing was copied: the request still names
        // the bank execution holds.
        assert!(std::sync::Arc::ptr_eq(
            refused
                .shared_associated_inputs()
                .expect("the held bank stays the request's bank"),
            &held
        ));
        // The refusal is a typed defect of the parse, never a silent success.
        let Err(failure) = refused.parse() else {
            panic!("a refused edit fails the parse")
        };
        assert_eq!(
            failure.reason(),
            HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Ownership)
        );
    }

    #[test]
    fn an_edit_after_sharing_reopens_a_bank_nothing_else_holds() {
        let shared = HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), limits(6, 1))
            .with_encoding_url_facts(Vec::new())
            .share_associated_inputs();
        let reopened = shared.with_encoding_url_facts(Vec::new());
        assert!(reopened.shared_associated_inputs().is_none());
        assert!(reopened.associated_inputs().is_some());
    }
}
