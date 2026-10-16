//! Completed parser reports admitted into the canonical DOM owner.
//!
//! A report is immutable construction history. Installation preserves that
//! history and its result-qualified identities; subsequent canonical edits do
//! not extend the historical source map or rerun parser construction.

use std::sync::Arc;

use crate::{
    DocumentId, DocumentString, DomChanges, DomDocumentStamp, DomError, DomNativeInputPlatform,
    DomOpaqueOrigin, DomSnapshot, DomStamp, NodeId,
};
use surgeist_html::HtmlNodeId;
use surgeist_html_parser::{HtmlParseReport, HtmlSourceId};
use surgeist_html_semantics::{HtmlBaseEncoding, HtmlBaseUrlRecord};

/// Explicit environment of a new, never-hosted ordinary Document.
///
/// This operation has no browsing context, navigable, executor or resource
/// service. These are selected creation facts, not conclusions drawn from a
/// missing loader. The encoding label and opaque encoding identity describe
/// the same caller-observed encoding; neither is inferred from decoded source
/// coordinates. The URL is a producer-qualified record, not a source identity.
/// Installation does not authorize future hosting or resource association.
#[derive(Clone, Debug)]
pub struct DomParsedDocumentEnvironment {
    pub(crate) origin: DomOpaqueOrigin,
    pub(crate) url: HtmlBaseUrlRecord,
    pub(crate) encoding: HtmlBaseEncoding,
    pub(crate) encoding_label: DocumentString,
    pub(crate) input: DomNativeInputPlatform,
}
impl DomParsedDocumentEnvironment {
    pub fn headless(
        origin: DomOpaqueOrigin,
        url: HtmlBaseUrlRecord,
        encoding: HtmlBaseEncoding,
        encoding_label: DocumentString,
        input: DomNativeInputPlatform,
    ) -> Self {
        Self {
            origin,
            url,
            encoding,
            encoding_label,
            input,
        }
    }
    pub fn origin(&self) -> &DomOpaqueOrigin {
        &self.origin
    }
    pub fn url(&self) -> &HtmlBaseUrlRecord {
        &self.url
    }
    pub fn encoding(&self) -> &HtmlBaseEncoding {
        &self.encoding
    }
    pub fn encoding_label(&self) -> &DocumentString {
        &self.encoding_label
    }
    pub fn input_platform(&self) -> DomNativeInputPlatform {
        self.input
    }
}

/// Independent adapter admission, in addition to the canonical [`crate::DomLimits`].
/// Zero is legal and refuses the first operation needing that resource.
///
/// Mapped nodes bound the retained subject closure. Work covers adapter visits,
/// source/metadata slots (including skipped slots), identity comparisons,
/// observed string extents, allocation copies and output formation. Scratch
/// bytes cover the peak of actual adapter/output and newly assembled child-vector
/// capacities, including both old and replacement banks during geometric growth;
/// canonical Draft/publication storage retains its existing DOM admission.
/// Historical bytes admit the report payload, parser's complete retained
/// construction receipt plus original source backing, separately lent root/inert
/// Base banks' owner-issued retained footprints, the complete contextual HTML
/// capture's owner-issued backing receipt, any retained unused captured Base
/// bank's complete receipt including its header once, and the environment URL
/// owner's footprint and encoding-label backing. These graph charges are
/// conservative per retained occurrence, without global Arc ownership accounting.
/// The complete contextual capture is charged once by its immutable identity;
/// its owner prices nested shared payloads per retained occurrence.
/// This deliberately
/// excludes allocator/Arc bookkeeping and does not claim an RSS bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomParsedHtmlInstallLimits {
    pub(crate) mapped_nodes: usize,
    pub(crate) work: u64,
    pub(crate) scratch_bytes: usize,
    pub(crate) historical_bytes: u64,
}
impl DomParsedHtmlInstallLimits {
    pub const fn new(
        mapped_nodes: usize,
        work: u64,
        scratch_bytes: usize,
        historical_bytes: u64,
    ) -> Self {
        Self {
            mapped_nodes,
            work,
            scratch_bytes,
            historical_bytes,
        }
    }
    pub const fn mapped_nodes(self) -> usize {
        self.mapped_nodes
    }
    pub const fn work(self) -> u64 {
        self.work
    }
    pub const fn scratch_bytes(self) -> usize {
        self.scratch_bytes
    }
    pub const fn historical_bytes(self) -> u64 {
        self.historical_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomParsedHtmlInstallResource {
    MappedNodes,
    Work,
    ScratchBytes,
    HistoricalBytes,
}

/// Refusal before any canonical tree, identity, revision or map publication.
#[derive(Debug)]
pub enum DomParsedHtmlInstallError {
    Target(DomError),
    StaleDestination {
        expected: DomStamp,
        actual: DomStamp,
    },
    NotCompletedOrdinaryReport,
    NotCompletedFragmentReport,
    ForeignContext,
    CurrentPolicyUnavailable,
    CapturedStateUnavailable,
    EnvironmentMismatch,
    MissingMetadata {
        subject: HtmlNodeId,
    },
    UnsupportedState {
        subject: HtmlNodeId,
    },
    FamilyStateUnavailable {
        subject: HtmlNodeId,
        class: surgeist_html::HtmlElementClass,
    },
    BaseStateUnavailable {
        subject: HtmlNodeId,
    },
    MetaStateUnavailable {
        document: HtmlNodeId,
        subject: HtmlNodeId,
        reason: surgeist_html_parser::HtmlParseMetaMissingKind,
    },
    HyperlinkUnavailable {
        subject: HtmlNodeId,
        reason: surgeist_html_parser::HtmlParseHyperlinkUnavailable,
    },
    ImageActivityUnavailable {
        subject: HtmlNodeId,
    },
    SourceLookup {
        subject: HtmlNodeId,
        error: surgeist_html::ForeignHtmlResult,
    },
    ResourceLimit {
        resource: DomParsedHtmlInstallResource,
        limit: u64,
    },
    Cancelled,
    AllocationFailed,
    RepresentationExhausted,
}
impl std::fmt::Display for DomParsedHtmlInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "completed HTML installation rejected: {self:?}")
    }
}
impl std::error::Error for DomParsedHtmlInstallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Target(error) => Some(error),
            Self::SourceLookup { error, .. } => Some(error),
            _ => None,
        }
    }
}

/// A historical subject in this exact immutable construction result.
#[derive(Debug)]
pub struct DomParsedHtmlBinding {
    pub(crate) source: HtmlNodeId,
    pub(crate) node: NodeId,
}
impl DomParsedHtmlBinding {
    pub fn source(&self) -> &HtmlNodeId {
        &self.source
    }
    pub fn node(&self) -> NodeId {
        self.node
    }
}

/// A captured historical subject resolved against the original canonical basis.
#[derive(Debug)]
pub struct DomParsedHtmlCapturedBinding {
    pub(crate) subject: surgeist_html_semantics::HtmlCaptureSubjectId,
    pub(crate) node: NodeId,
}
impl DomParsedHtmlCapturedBinding {
    pub fn subject(&self) -> &surgeist_html_semantics::HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn node(&self) -> NodeId {
        self.node
    }
}

/// Intrinsic installation does not execute or certify downstream associations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomParsedHtmlAssociationReadiness {
    Pending,
}

/// One complete result published once; ordinary reports create a new Document,
/// contextual reports create a detached fragment in their actual target Document.
///
/// The original report is historical backing, including lexical/retired/staged
/// destinations that have no live NodeId. Its source identity and actual result
/// root qualify the bindings. Later edits leave this snapshot and history intact.
/// Context captures are borrowed only for admission. The report retains its
/// original HTML capture; the result keeps its exact canonical basis stamp and
/// resolved captured bindings without pinning the old canonical State.
#[derive(Debug)]
pub struct DomParsedHtmlInstallation {
    pub(crate) report: Arc<HtmlParseReport>,
    pub(crate) source_root: HtmlNodeId,
    pub(crate) document: DocumentId,
    pub(crate) root: NodeId,
    pub(crate) context_basis: Option<DomDocumentStamp>,
    pub(crate) captured_bindings: Vec<DomParsedHtmlCapturedBinding>,
    pub(crate) snapshot: DomSnapshot,
    pub(crate) changes: DomChanges,
    pub(crate) bindings: Vec<DomParsedHtmlBinding>,
}
impl DomParsedHtmlInstallation {
    pub fn source(&self) -> &HtmlSourceId {
        self.report.source().identity()
    }
    pub fn source_root(&self) -> &HtmlNodeId {
        &self.source_root
    }
    pub fn document(&self) -> DocumentId {
        self.document
    }
    /// Installed ordinary Document or detached contextual DocumentFragment.
    pub fn root(&self) -> NodeId {
        self.root
    }
    pub fn context_basis(&self) -> Option<DomDocumentStamp> {
        self.context_basis
    }
    pub fn captured_bindings(&self) -> &[DomParsedHtmlCapturedBinding] {
        &self.captured_bindings
    }
    pub fn captured_node_for(
        &self,
        subject: &surgeist_html_semantics::HtmlCaptureSubjectId,
    ) -> Option<NodeId> {
        self.captured_bindings
            .iter()
            .find(|binding| binding.subject == *subject)
            .map(|binding| binding.node)
    }
    pub fn snapshot(&self) -> &DomSnapshot {
        &self.snapshot
    }
    pub fn changes(&self) -> &DomChanges {
        &self.changes
    }
    pub fn report(&self) -> &HtmlParseReport {
        &self.report
    }
    pub fn bindings(&self) -> &[DomParsedHtmlBinding] {
        &self.bindings
    }
    pub fn association_readiness(&self) -> DomParsedHtmlAssociationReadiness {
        DomParsedHtmlAssociationReadiness::Pending
    }
    pub fn node_for(&self, source: &HtmlNodeId) -> Option<NodeId> {
        self.bindings
            .iter()
            .find(|binding| binding.source == *source)
            .map(|binding| binding.node)
    }
}
