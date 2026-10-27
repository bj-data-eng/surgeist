//! Lazy ElementCSSInlineStyle identity and immutable composed observations.
//!
//! Bindings initialize declarations from the current canonical attribute.
//! Context-bearing ordinary batches synchronize real ordered DOM inputs atomically.
//! Context-free edits refuse reached bound changes. CSSOM-origin edits retain
//! their actual owner-write obligation until canonical feedback completes.
use crate::{
    DomBatch, DomCommit, DomDocumentStamp, DomError, DomSnapshot, DomStamp, DomTreeScope, NodeId,
    NodeOperand,
};
use std::sync::Arc;
use surgeist_css::CssParserContext;
use surgeist_css::source::{CssUtf16InputError, CssUtf16InputLimits};
use surgeist_cssom::{
    CssStyleProperties, CssomCapture, CssomDeclarationRequestLimits, CssomDeclarationSupport,
    CssomError, CssomInputRole, CssomInputVersion, CssomLimits, CssomStyleAttributeError,
};

/// Explicit grammar/support selection for one Element in one staged Document.
/// The ordinary operation derives parser mode from the actual staged Document.
/// Creation operands must belong to the operation's exact batch.
#[derive(Clone, Debug)]
pub struct DomInlineStyleProfile {
    pub(crate) element: NodeOperand,
    pub(crate) document: NodeOperand,
    pub(crate) parser: CssParserContext,
    pub(crate) support_version: CssomInputVersion,
    pub(crate) support: CssomDeclarationSupport,
}
impl DomInlineStyleProfile {
    /// Select grammar features and independently qualified support. The parser
    /// context's mode is replaced by the actual staged mode at each ingestion.
    pub fn new(
        element: impl Into<NodeOperand>,
        document: impl Into<NodeOperand>,
        parser: CssParserContext,
        support_version: CssomInputVersion,
        support: CssomDeclarationSupport,
    ) -> Result<Self, DomInlineStyleError> {
        if support_version.role != CssomInputRole::Support {
            return Err(DomInlineStyleError::InvalidSupportRole);
        }
        Ok(Self {
            element: element.into(),
            document: document.into(),
            parser,
            support_version,
            support,
        })
    }
}

/// Immutable profiles qualified by one original canonical basis and batch.
/// A Document creation ticket describes its actual staged facts without an
/// invented prepublication DocumentId or Document stamp.
#[derive(Clone)]
pub struct DomInlineStyleBatchContext {
    pub(crate) basis: DomStamp,
    pub(crate) batch: Arc<crate::batch::BatchIdentity>,
    pub(crate) profiles: Vec<DomInlineStyleProfile>,
}
impl DomInlineStyleBatchContext {
    /// Qualify the explicit rows. Duplicate or missing applicable rows are
    /// diagnosed under the operation's finite work policy.
    pub fn new(
        batch: &DomBatch,
        basis: DomStamp,
        profiles: Vec<DomInlineStyleProfile>,
    ) -> Result<Self, DomInlineStyleError> {
        for row in &profiles {
            for operand in [&row.element, &row.document] {
                match operand {
                    NodeOperand::Existing(id) if id.owner() != basis.owner() => {
                        return Err(DomError::ForeignOwner.into());
                    }
                    NodeOperand::Created(ticket) if !Arc::ptr_eq(&ticket.owner, &batch.owner) => {
                        return Err(DomError::ForeignBatch.into());
                    }
                    _ => {}
                }
            }
        }
        Ok(Self {
            basis,
            batch: batch.owner.clone(),
            profiles,
        })
    }
}

/// Explicit inline profiles for one original Source invocation. Source roots
/// accept published operands only; the original basis is checked at entry and
/// the same profiles survive that invocation's own publications and resumes.
#[derive(Clone, Debug)]
pub struct DomInlineStyleSourceContext {
    pub(crate) basis: DomStamp,
    pub(crate) profiles: Vec<DomInlineStyleProfile>,
}
impl DomInlineStyleSourceContext {
    /// Qualify independently selected grammar/support for existing Elements and
    /// Documents. Actual staged mode, base and topology remain DOM-owned facts.
    pub fn new(
        basis: DomStamp,
        profiles: Vec<DomInlineStyleProfile>,
    ) -> Result<Self, DomInlineStyleError> {
        for row in &profiles {
            for operand in [&row.element, &row.document] {
                match operand {
                    NodeOperand::Existing(id) if id.owner() == basis.owner() => {}
                    NodeOperand::Existing(_) => return Err(DomError::ForeignOwner.into()),
                    NodeOperand::Created(_) => return Err(DomError::ForeignBatch.into()),
                }
            }
        }
        Ok(Self { basis, profiles })
    }
}

/// Original evidence from one Source segment that synchronized inline inputs.
/// The CSSOM receipt retains its producer-owned logical/output contract; this
/// wrapper does not claim a physical bound on the producer's private backing.
pub struct DomInlineStyleSourcePublication {
    pub(crate) dom: crate::DomChanges,
    pub(crate) inline: surgeist_cssom::CssomCommit,
}
impl std::fmt::Debug for DomInlineStyleSourcePublication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomInlineStyleSourcePublication")
            .field("dom", &self.dom)
            .field("inline_publication", self.inline.publication())
            .field("owner_effects", &self.inline.owner_effects())
            .finish_non_exhaustive()
    }
}
impl DomInlineStyleSourcePublication {
    /// Exact canonical segment paired with this inline publication.
    pub fn dom(&self) -> &crate::DomChanges {
        &self.dom
    }
    /// The original producer receipt, returned once at its publication boundary.
    pub fn inline(&self) -> &surgeist_cssom::CssomCommit {
        &self.inline
    }
    /// Transfer the two original owners' publication evidence.
    pub fn into_parts(self) -> (crate::DomChanges, surgeist_cssom::CssomCommit) {
        (self.dom, self.inline)
    }
}

/// Progress of the original Source invocation and its optional inline evidence.
/// A repeated external request does not repeat an already returned publication.
pub struct DomInlineStyleSourceProgress {
    pub(crate) source: crate::DomSourceProgress,
    pub(crate) inline: Option<DomInlineStyleSourcePublication>,
}
impl DomInlineStyleSourceProgress {
    /// The original Source controller's request or completion.
    pub fn source(&self) -> &crate::DomSourceProgress {
        &self.source
    }
    /// Original inline evidence produced by this advance, if any.
    pub fn inline(&self) -> Option<&DomInlineStyleSourcePublication> {
        self.inline.as_ref()
    }
    /// Transfer progress and its original publication without retaining history.
    pub fn into_parts(
        self,
    ) -> (
        crate::DomSourceProgress,
        Option<DomInlineStyleSourcePublication>,
    ) {
        (self.source, self.inline)
    }
}

/// Actual publication evidence from one ordinary composed operation. Inline
/// absence means no CSSOM batch was needed; it is not a synthesized commit.
pub struct DomInlineStyleCommit {
    pub(crate) dom: DomCommit,
    pub(crate) inline: Option<surgeist_cssom::CssomCommit>,
}
impl DomInlineStyleCommit {
    /// Canonical DOM publication and creation outputs.
    pub fn dom(&self) -> &DomCommit {
        &self.dom
    }
    /// The producer's original inline publication/change evidence, if reached.
    pub fn inline(&self) -> Option<&surgeist_cssom::CssomCommit> {
        self.inline.as_ref()
    }
    /// Consume both real publications without replacing either owner's result.
    pub fn into_parts(self) -> (DomCommit, Option<surgeist_cssom::CssomCommit>) {
        (self.dom, self.inline)
    }
}

/// Explicit limits for one DOM owner's inline identity domain. CSSOM retains
/// its independent logical/source quotas. `formation_bytes` bounds the original
/// DOM custody ledger's prospective union during inline formation, including
/// canonical data and replacement overlap; it is not a process memory ceiling.
#[derive(Clone, Debug)]
pub struct DomInlineStyleLimits {
    pub(crate) cssom: CssomLimits,
    pub(crate) bindings: usize,
    pub(crate) formation_bytes: usize,
    pub(crate) scratch_bytes: usize,
    pub(crate) work: u64,
}
impl DomInlineStyleLimits {
    /// Configure finite owner, formation and temporary-operation allowances.
    pub fn new(
        cssom: CssomLimits,
        bindings: usize,
        formation_bytes: usize,
        scratch_bytes: usize,
        work: u64,
    ) -> Result<Self, DomError> {
        for amount in [bindings, formation_bytes, scratch_bytes] {
            if amount > isize::MAX as usize {
                return Err(DomError::InvalidLimit(
                    crate::DomResource::InlineStyleStorage,
                ));
            }
        }
        Ok(Self {
            cssom,
            bindings,
            formation_bytes,
            scratch_bytes,
            work,
        })
    }
}

/// Explicit producer limits for initialization of the actual canonical text.
#[derive(Clone, Copy, Debug)]
pub struct DomInlineStyleRequestLimits {
    /// CSS owns lossless UTF-16 preprocessing and its retained source/map.
    pub input: CssUtf16InputLimits,
    /// CSSOM owns declaration admission, expansion and serialization.
    pub declaration: CssomDeclarationRequestLimits,
}

/// External support/profile selection qualified by one actual current Document.
/// DOM derives owner, Document, tree-scope and base facts itself. Parser mode
/// must match that Document; its independent grammar profile remains explicit.
#[derive(Clone, Debug)]
pub struct DomInlineStyleInputs {
    pub(crate) basis: DomDocumentStamp,
    pub(crate) parser: CssParserContext,
    pub(crate) support_version: CssomInputVersion,
    pub(crate) support: CssomDeclarationSupport,
}
impl DomInlineStyleInputs {
    /// Missing support is not interpreted as usable support; an explicitly
    /// empty membership is valid. Currentness is checked at the DOM operation.
    pub fn new(
        basis: DomDocumentStamp,
        parser: CssParserContext,
        support_version: CssomInputVersion,
        support: CssomDeclarationSupport,
    ) -> Result<Self, DomInlineStyleError> {
        if support_version.role != CssomInputRole::Support {
            return Err(DomInlineStyleError::InvalidSupportRole);
        }
        Ok(Self {
            basis,
            parser,
            support_version,
            support,
        })
    }
}

/// Reached work requiring a context-bearing synchronization front.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomInlineStyleChange {
    /// An actual null-namespace style-attribute notification.
    Attribute,
    /// Node-document, tree-scope or relevant Document facts would change.
    Context,
    /// The exact bound Element would retire.
    Retirement,
}

/// Binding, observation or synchronization refusal. The refused canonical DOM
/// segment does not publish; successful preceding Source prefixes remain
/// canonical. An Incomplete mutation carries a refusal after CSSOM has committed.
#[derive(Debug)]
pub enum DomInlineStyleError {
    /// The canonical owner rejected input or admission.
    Dom(DomError),
    /// The declaration producer rejected the operation.
    Cssom(CssomError),
    /// Original CSS input preparation or its DOM poll failed.
    Input(CssUtf16InputError<DomError>),
    /// No finite inline policy has been installed.
    Unconfigured,
    /// An existing owner already has its fixed policy.
    AlreadyConfigured,
    /// This namespace has no supported ElementCSSInlineStyle capability.
    Inapplicable,
    /// The supplied Document observation is no longer current.
    StaleContext,
    /// The selected mode does not describe the actual Document.
    ParserModeMismatch,
    /// The supplied version is not a Support qualification.
    InvalidSupportRole,
    /// A bound block already exists with different explicit external inputs.
    ContextUpdateRequired,
    /// The Element has no allocated same-object inline declaration block.
    Unbound,
    /// Another actual inline owner-write outcome is still resident.
    OperationPending { element: NodeId },
    /// Reconciliation requires a failed terminal effect, not a live/applied one.
    ReconciliationRequiresTerminal {
        status: surgeist_cssom::CssomOwnerEffectStatus,
    },
    /// An effect-free operation has no owner synchronization to reconcile.
    ReconciliationNotRequired,
    /// This effect is terminal and cannot be retried as a Pending write.
    TerminalEffect {
        status: surgeist_cssom::CssomOwnerEffectStatus,
    },
}
impl From<DomError> for DomInlineStyleError {
    fn from(value: DomError) -> Self {
        Self::Dom(value)
    }
}
impl From<CssomError> for DomInlineStyleError {
    fn from(value: CssomError) -> Self {
        Self::Cssom(value)
    }
}
impl From<CssomStyleAttributeError<DomError>> for DomInlineStyleError {
    fn from(value: CssomStyleAttributeError<DomError>) -> Self {
        match value {
            CssomStyleAttributeError::Cssom(error) => Self::Cssom(error),
            CssomStyleAttributeError::Input(error) => Self::Input(error),
        }
    }
}
impl std::fmt::Display for DomInlineStyleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for DomInlineStyleError {}

/// An immutable, jointly qualified DOM/CSSOM observation. It retains no live
/// writer. Later refusal, changes or owner destruction cannot rewrite it.
#[derive(Clone, Debug)]
pub struct DomInlineStyleCapture {
    pub(crate) element: NodeId,
    pub(crate) scope: DomTreeScope,
    pub(crate) dom: DomSnapshot,
    pub(crate) properties: CssomCapture<CssStyleProperties>,
}
impl DomInlineStyleCapture {
    /// Exact bound Element generation.
    pub fn element(&self) -> NodeId {
        self.element
    }
    /// Actual ordinary tree scope at this observation.
    pub fn tree_scope(&self) -> DomTreeScope {
        self.scope
    }
    /// Canonical observation paired with the declarations.
    pub fn dom(&self) -> &DomSnapshot {
        &self.dom
    }
    /// Actual typed CSSOM capture and its same-object handle.
    pub fn properties(&self) -> &CssomCapture<CssStyleProperties> {
        &self.properties
    }
}

/// DOM-owned envelope for one CSSOM-origin feedback operation. CSSOM keeps its
/// independent declaration, diagnostic, publication and owner-effect quotas.
/// Work is cumulative across feedback refusal, parking, reconciliation and retry.
/// Successful reconciliation emits one terminal outcome; final completion counts
/// its producer outcome and actual DOM changes against the same output allowance.
/// Storage uses
/// the original DOM custody ledger, including canonical replacement overlap.
#[derive(Clone, Copy, Debug)]
pub struct DomInlineStyleFeedbackLimits {
    pub(crate) retained_bytes: usize,
    pub(crate) retained_units: usize,
    pub(crate) work: u64,
    pub(crate) outputs: usize,
}
impl DomInlineStyleFeedbackLimits {
    /// Configure finite DOM feedback admission before the standard mutation.
    pub const fn new(
        retained_bytes: usize,
        retained_units: usize,
        work: u64,
        outputs: usize,
    ) -> Self {
        Self {
            retained_bytes,
            retained_units,
            work,
            outputs,
        }
    }
}
/// Independent producer request limits and the DOM-owned feedback envelope.
#[derive(Clone, Copy, Debug)]
pub struct DomInlineStyleMutationLimits {
    /// Actual standard declaration executor's producer limits.
    pub declaration: CssomDeclarationRequestLimits,
    /// Original DOM wrapper, preparation and continuation admission.
    pub feedback: DomInlineStyleFeedbackLimits,
}
/// Arguments of the standard setProperty operation. Priority None is the
/// producer's standard empty-priority input.
#[derive(Clone, Copy, Debug)]
pub struct DomInlineStyleProperty<'a> {
    /// Exact property spelling passed to CSSOM.
    pub name: &'a str,
    /// Exact value passed to CSSOM.
    pub value: &'a str,
    /// Standard optional priority spelling.
    pub priority: Option<&'a str>,
}
/// Original standard return value, moved from the actual producer outcome.
#[derive(Debug)]
pub enum DomInlineStyleReturn {
    /// setCssText and setProperty return no value.
    Unit,
    /// removeProperty returns the original serialized removed value.
    Removed(String),
}
/// Authentic producer evidence for one inline owner-write attempt.
pub enum DomInlineStyleOutcome {
    /// Components moved unchanged from the standard declaration executor.
    Standard {
        /// Original standard return value.
        value: DomInlineStyleReturn,
        /// Original committed publication.
        publication: surgeist_cssom::CssomPublication,
        /// Original diagnostics, including their retained source provenance.
        diagnostics: Vec<surgeist_css::diagnostic::CssRecoveryDiagnostic>,
        /// Actual affine effects.
        effects: Vec<surgeist_cssom::CssomOwnerEffect>,
    },
    /// Fresh current-state synchronization, without a standard mutation result.
    Synchronization {
        /// Actual producer commit; its effects were detached into `effects`.
        commit: surgeist_cssom::CssomCommit,
        /// Actual effects taken once from this commit.
        effects: Vec<surgeist_cssom::CssomOwnerEffect>,
    },
}
impl DomInlineStyleOutcome {
    /// Publication owned by the actual standard outcome or synchronization commit.
    pub fn publication(&self) -> &surgeist_cssom::CssomPublication {
        match self {
            Self::Standard { publication, .. } => publication,
            Self::Synchronization { commit, .. } => commit.publication(),
        }
    }
    /// Actual affine effects, including their unchanged failure status.
    pub fn owner_effects(&self) -> &[surgeist_cssom::CssomOwnerEffect] {
        match self {
            Self::Standard { effects, .. } | Self::Synchronization { effects, .. } => effects,
        }
    }
    pub(crate) fn effects_mut(&mut self) -> &mut [surgeist_cssom::CssomOwnerEffect] {
        match self {
            Self::Standard { effects, .. } | Self::Synchronization { effects, .. } => effects,
        }
    }
}
/// Displaced failed attempt. Reconciliation preserves this evidence and does
/// not turn its terminal effect into a successful application.
pub struct DomInlineStyleTerminal {
    pub(crate) element: NodeId,
    pub(crate) outcome: DomInlineStyleOutcome,
    pub(crate) spent_work: u64,
    pub(crate) _custody: crate::store::custody::Debit,
}
impl DomInlineStyleTerminal {
    /// Original exact target.
    pub fn element(&self) -> NodeId {
        self.element
    }
    /// Original terminal producer evidence.
    pub fn outcome(&self) -> &DomInlineStyleOutcome {
        &self.outcome
    }
    /// Cumulative lineage work at displacement, not a live counter.
    pub fn spent_work(&self) -> u64 {
        self.spent_work
    }
    /// Transfer the authentic terminal outcome once.
    pub fn into_outcome(self) -> DomInlineStyleOutcome {
        self.outcome
    }
}
/// One producer attempt and its canonical feedback. An effect-free standard
/// no-op needs no DOM publication.
pub struct DomInlineStyleCompletion {
    pub(crate) outcome: DomInlineStyleOutcome,
    pub(crate) dom: Option<DomCommit>,
    pub(crate) _custody: crate::store::custody::Debit,
}
impl DomInlineStyleCompletion {
    /// Authentic standard or synchronization result.
    pub fn outcome(&self) -> &DomInlineStyleOutcome {
        &self.outcome
    }
    /// Original CSSOM publication, never synthesized from DOM changes.
    pub fn publication(&self) -> &surgeist_cssom::CssomPublication {
        self.outcome.publication()
    }
    /// Actual effects and application states.
    pub fn owner_effects(&self) -> &[surgeist_cssom::CssomOwnerEffect] {
        self.outcome.owner_effects()
    }
    /// Canonical feedback publication, absent for an effect-free operation.
    pub fn dom(&self) -> Option<&DomCommit> {
        self.dom.as_ref()
    }
    /// Move the producer evidence and canonical publication exactly once.
    pub fn into_parts(self) -> (DomInlineStyleOutcome, Option<DomCommit>) {
        (self.outcome, self.dom)
    }
}
/// A committed standard mutation or fresh synchronization completed feedback or
/// remains owned by the same DOM. The latter is not a prepublication error.
#[must_use = "complete or retain the original inline feedback obligation"]
pub enum DomInlineStyleMutation<'a> {
    /// Both owners are coherent and the actual components transfer to caller.
    Complete(DomInlineStyleCompletion),
    /// CSSOM committed; DOM feedback refused and the original operation parks
    /// on lease drop. Its original error describes this attempt only.
    Incomplete {
        /// Exclusive access to the retained operation; Drop parks it.
        operation: crate::DomInlineStyleOperation<'a>,
        /// The exact postpublication refusal.
        reason: DomInlineStyleError,
    },
}
