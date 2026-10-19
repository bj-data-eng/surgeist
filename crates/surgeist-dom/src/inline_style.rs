//! Lazy ElementCSSInlineStyle identity and immutable composed observations.
//!
//! Bindings initialize declarations from the current canonical attribute.
//! Context-bearing ordinary batches synchronize real ordered DOM inputs atomically.
//! Context-free and Source edits refuse reached bound changes; CSSOM owner feedback
//! and bound retirement remain separate capabilities.
use crate::{
    DomBatch, DomCommit, DomDocumentStamp, DomError, DomSnapshot, DomStamp, DomTreeScope, NodeId,
    NodeOperand,
};
use std::sync::Arc;
use surgeist_css::{CssParserContext, CssUtf16InputLimits};
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

/// Binding/observation refusal. No CSSOM or DOM publication occurs on error.
#[derive(Debug)]
pub enum DomInlineStyleError {
    /// The canonical owner rejected input or admission.
    Dom(DomError),
    /// The declaration producer rejected the operation.
    Cssom(CssomError),
    /// Original CSS input preparation or its DOM poll failed.
    Input(CssomStyleAttributeError<DomError>),
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
        Self::Input(value)
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
