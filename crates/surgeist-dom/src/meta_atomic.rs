//! Explicit native Meta composition against actual CSSOM owners.
use crate::{
    DocumentString, DomCommit, DomError, DomInlineStyleBatchContext, DomInlineStyleError,
    DomInlineStyleRequestLimits, NodeId,
};
use surgeist_cssom::{
    CssomAdmission, CssomCommit, CssomContext, CssomDeclarationCandidate, CssomError,
};

/// Finite additional holder, UTF-16, work and output admission for one native join.
/// This envelope does not replace canonical DOM, Source, Runtime or producer limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomMetaAtomicLimits {
    pub(crate) retained_bytes: usize,
    pub(crate) retained_units: usize,
    pub(crate) work: u64,
    pub(crate) outputs: usize,
}
impl DomMetaAtomicLimits {
    pub fn new(
        retained_bytes: usize,
        retained_units: usize,
        work: u64,
        outputs: usize,
    ) -> Result<Self, DomError> {
        retained_units
            .checked_mul(2)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(Self {
            retained_bytes,
            retained_units,
            work,
            outputs,
        })
    }
    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
    pub const fn retained_units(self) -> usize {
        self.retained_units
    }
    pub const fn work(self) -> u64 {
        self.work
    }
    pub const fn outputs(self) -> usize {
        self.outputs
    }
}
/// Actual simultaneous Runtime-owned bank/name/output overlap, excluding the
/// producer's private graph. Repeated reports replace the prior whole report;
/// the composition retains the checked high-water until its fence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomMetaRuntimeUsage {
    pub(crate) bytes: usize,
    pub(crate) units: usize,
    pub(crate) outputs: usize,
}
impl DomMetaRuntimeUsage {
    pub fn new(bytes: usize, units: usize, outputs: usize) -> Result<Self, DomError> {
        units
            .checked_mul(2)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(Self {
            bytes,
            units,
            outputs,
        })
    }
    pub const fn bytes(self) -> usize {
        self.bytes
    }
    pub const fn units(self) -> usize {
        self.units
    }
    pub const fn outputs(self) -> usize {
        self.outputs
    }
}
/// Immutable declaration of actual Runtime bank storage and already-prepaid
/// formation work. It proves no execution/currentness and owns no counter.
/// Preparation revalidates it against its own actual limits before lower work.
pub struct DomMetaRuntimePrefix {
    pub(crate) usage: DomMetaRuntimeUsage,
    pub(crate) work: u64,
}
impl DomMetaRuntimePrefix {
    pub fn new(
        usage: DomMetaRuntimeUsage,
        work: u64,
        limits: DomMetaAtomicLimits,
    ) -> Result<Self, DomError> {
        crate::store::admit(
            usage.bytes,
            limits.retained_bytes,
            crate::DomResource::MetaAtomicRetainedBytes,
        )?;
        crate::store::admit(
            usage.units,
            limits.retained_units,
            crate::DomResource::MetaAtomicRetainedUnits,
        )?;
        crate::store::admit(
            usage.outputs,
            limits.outputs,
            crate::DomResource::MetaAtomicOutputs,
        )?;
        if work > limits.work {
            return Err(DomError::Meta(crate::DomMetaError::WorkLimit {
                limit: limits.work,
            }));
        }
        Ok(Self { usage, work })
    }
    pub const fn usage(&self) -> DomMetaRuntimeUsage {
        self.usage
    }
    pub const fn work(&self) -> u64 {
        self.work
    }
}
/// Borrowed pure inline admission at this invocation's actual prepublication step.
pub type DomMetaInlineResolver<'r> = dyn FnMut(
        &[CssomDeclarationCandidate],
        &CssomContext,
    ) -> Result<CssomAdmission, surgeist_cssom::CssomError>
    + 'r;
/// Original ordinary-batch inline profiles and a resolver borrowed only during
/// preparation. Neither preparation nor published custody stores the callback.
pub struct DomMetaInlineBatch<'r> {
    pub(crate) context: DomInlineStyleBatchContext,
    pub(crate) limits: DomInlineStyleRequestLimits,
    pub(crate) resolver: &'r mut DomMetaInlineResolver<'r>,
}
impl<'r> DomMetaInlineBatch<'r> {
    pub fn new(
        context: DomInlineStyleBatchContext,
        limits: DomInlineStyleRequestLimits,
        resolver: &'r mut DomMetaInlineResolver<'r>,
    ) -> Self {
        Self {
            context,
            limits,
            resolver,
        }
    }
}
/// Exact historical Draft subject, without execution or currentness authority.
/// A private creation may never receive a published NodeId. Its original batch
/// qualification and actual allocation index remain distinct from another batch.
#[derive(Clone)]
pub struct DomMetaSubject {
    pub(crate) kind: MetaSubjectKind,
}
#[derive(Clone)]
pub(crate) enum MetaSubjectKind {
    Existing(NodeId),
    Created {
        owner: std::sync::Arc<crate::batch::BatchIdentity>,
        index: usize,
    },
}
impl PartialEq for DomMetaSubject {
    fn eq(&self, other: &Self) -> bool {
        match (&self.kind, &other.kind) {
            (MetaSubjectKind::Existing(a), MetaSubjectKind::Existing(b)) => a == b,
            (
                MetaSubjectKind::Created {
                    owner: a,
                    index: ai,
                },
                MetaSubjectKind::Created {
                    owner: b,
                    index: bi,
                },
            ) => std::sync::Arc::ptr_eq(a, b) && ai == bi,
            _ => false,
        }
    }
}
impl Eq for DomMetaSubject {}
impl std::fmt::Debug for DomMetaSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            MetaSubjectKind::Existing(id) => f.debug_tuple("Existing").field(id).finish(),
            MetaSubjectKind::Created { index, .. } => f
                .debug_struct("Created")
                .field("allocation", index)
                .finish_non_exhaustive(),
        }
    }
}
/// One source-required preferred-name invocation, in actual execution order.
/// Removal later in the same batch does not erase this history.
#[derive(Debug)]
pub struct DomMetaPreferredOccurrence {
    pub(crate) document: NodeId,
    pub(crate) meta: DomMetaSubject,
    pub(crate) content: DocumentString,
}
impl DomMetaPreferredOccurrence {
    pub const fn document(&self) -> NodeId {
        self.document
    }
    pub fn meta(&self) -> &DomMetaSubject {
        &self.meta
    }
    pub fn content(&self) -> &DocumentString {
        &self.content
    }
}
/// A prepublication failure at its actual owning boundary.
#[derive(Debug)]
pub enum DomMetaAtomicError {
    Dom(DomError),
    Inline(DomInlineStyleError),
    /// The actual root is already borrowed; no producer operation was attempted.
    OwnerBusy {
        root: NodeId,
    },
    /// The actual root is poisoned; no producer operation was attempted.
    OwnerPoisoned {
        root: NodeId,
    },
    /// The attempted producer operation returned its original typed refusal.
    Cssom {
        root: NodeId,
        error: CssomError,
    },
    MissingOwner {
        root: NodeId,
    },
    MismatchedOwner {
        root: NodeId,
    },
    StaleProducer {
        root: NodeId,
    },
}
impl From<DomError> for DomMetaAtomicError {
    fn from(value: DomError) -> Self {
        Self::Dom(value)
    }
}
impl From<DomInlineStyleError> for DomMetaAtomicError {
    fn from(value: DomInlineStyleError) -> Self {
        Self::Inline(value)
    }
}
impl std::fmt::Display for DomMetaAtomicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native Meta composition refused: {self:?}")
    }
}
impl std::error::Error for DomMetaAtomicError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(e) => Some(e),
            Self::Inline(e) => Some(e),
            Self::Cssom { error, .. } => Some(error),
            _ => None,
        }
    }
}
/// Genuine producer output for one actual Document, in first-occurrence order.
/// Each distinct reached Document appears once, including unchanged publications.
pub struct DomMetaRootCommit {
    pub(crate) root: NodeId,
    pub(crate) cssom: CssomCommit,
}
impl DomMetaRootCommit {
    pub const fn root(&self) -> NodeId {
        self.root
    }
    pub fn cssom(&self) -> &CssomCommit {
        &self.cssom
    }
    pub fn into_parts(self) -> (NodeId, CssomCommit) {
        (self.root, self.cssom)
    }
}
/// The actual ordinary DOM and producer outputs, released from published custody.
pub struct DomMetaOrdinaryCommit {
    pub(crate) dom: DomCommit,
    pub(crate) roots: Vec<DomMetaRootCommit>,
    pub(crate) inline: Option<CssomCommit>,
}
impl DomMetaOrdinaryCommit {
    pub fn dom(&self) -> &DomCommit {
        &self.dom
    }
    pub fn roots(&self) -> &[DomMetaRootCommit] {
        &self.roots
    }
    pub fn inline(&self) -> Option<&CssomCommit> {
        self.inline.as_ref()
    }
    pub fn into_parts(self) -> (DomCommit, Vec<DomMetaRootCommit>, Option<CssomCommit>) {
        (self.dom, self.roots, self.inline)
    }
}
/// CSSOM dead backing retained through the composed fence. This is not a claim
/// that all existing DOM publication backing has bounded disposal.
pub struct DomMetaRetirement {
    pub(crate) _cssom: Vec<surgeist_cssom::CssomCommitRetirement>,
}
