//! Explicit fresh-native installation, distinct from complete parser reports.
use crate::{
    DocumentId, DomChanges, DomCommit, DomDocumentStamp, DomError, DomSnapshot, DomStamp, NodeId,
};
use surgeist_html::HtmlNodeId;

/// Adapter-owned source/map work and simultaneous retained heap storage.
/// Canonical Draft/tree/name/family work remains bounded by DomLimits and its
/// cancellation checks; this counter does not claim to meter that owner's work.
///
/// Mapped nodes include the source fragment and owned Template contents, but not
/// private auxiliary Documents. Work charges one unit per visited node, each of
/// five association observations, child/contents edge, attribute, previous-map
/// comparison, map write, final remap and binding; name/string fields charge
/// their UTF-16 extent. An empty ordinary fragment costs nine work units.
/// Scratch bytes bound the simultaneous actual capacities of adapter traversal,
/// source rows, final IDs and returned bindings. Canonical staged tree and
/// publication storage use [`crate::DomLimits`] separately. Zero limits are legal
/// and refuse any operation requiring that resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomNativeHtmlInstallLimits {
    pub(crate) mapped_nodes: usize,
    pub(crate) work: usize,
    pub(crate) scratch_bytes: usize,
}
impl DomNativeHtmlInstallLimits {
    pub fn new(mapped_nodes: usize, work: usize, scratch_bytes: usize) -> Self {
        Self {
            mapped_nodes,
            work,
            scratch_bytes,
        }
    }
    pub fn mapped_nodes(self) -> usize {
        self.mapped_nodes
    }
    pub fn work(self) -> usize {
        self.work
    }
    pub fn scratch_bytes(self) -> usize {
        self.scratch_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomNativeHtmlInstallResource {
    MappedNodes,
    Work,
    ScratchBytes,
}

/// Exact relationships that cannot be converted into fresh native ownership.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomNativeHtmlAssociation {
    HostedSourceRoot,
    DeclarativeShadow,
    TemplateShadowReference,
    TemplateOwnership,
}

/// No refused installation publishes a partial map, tree, revision or history.
#[derive(Debug)]
pub enum DomNativeHtmlInstallError {
    Target(DomError),
    StaleDestination {
        expected: DomDocumentStamp,
        actual: DomStamp,
    },
    Source {
        subject: HtmlNodeId,
        error: DomError,
    },
    SourceLookup {
        subject: HtmlNodeId,
        error: surgeist_html::ForeignHtmlResult,
    },
    UnsupportedAssociation {
        subject: HtmlNodeId,
        association: DomNativeHtmlAssociation,
    },
    ResourceLimit {
        resource: DomNativeHtmlInstallResource,
        limit: usize,
    },
    Cancelled,
    AllocationFailed,
    RepresentationExhausted,
}
impl std::fmt::Display for DomNativeHtmlInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native HTML installation rejected: {self:?}")
    }
}
impl std::error::Error for DomNativeHtmlInstallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Target(error) | Self::Source { error, .. } => Some(error),
            Self::SourceLookup { error, .. } => Some(error),
            _ => None,
        }
    }
}

/// A historical authoring identity bound to the installed snapshot, not later edits.
#[derive(Debug)]
pub struct DomNativeHtmlBinding {
    pub(crate) source: HtmlNodeId,
    pub(crate) node: NodeId,
}
impl DomNativeHtmlBinding {
    pub fn source(&self) -> &HtmlNodeId {
        &self.source
    }
    pub fn node(&self) -> NodeId {
        self.node
    }
}

/// Successful fresh-native creation and fragment transfer through one publication.
/// The mapped source wrapper is a real detached, empty canonical fragment.
/// Only result-qualified IDs are retained; no second source tree is kept alive.
/// Bindings follow source preorder, with Template owned contents before ordinary
/// children. Later edits do not update this historical map or snapshot. Dropping
/// the source construction does not invalidate either result.
#[derive(Debug)]
pub struct DomNativeHtmlInstallation {
    pub(crate) destination: DomDocumentStamp,
    pub(crate) source_root: HtmlNodeId,
    pub(crate) root: NodeId,
    pub(crate) snapshot: DomSnapshot,
    pub(crate) commit: DomCommit,
    pub(crate) bindings: Vec<DomNativeHtmlBinding>,
}
impl DomNativeHtmlInstallation {
    pub fn destination(&self) -> DomDocumentStamp {
        self.destination
    }
    pub fn document(&self) -> DocumentId {
        self.destination.document()
    }
    pub fn source_root(&self) -> &HtmlNodeId {
        &self.source_root
    }
    pub fn root(&self) -> NodeId {
        self.root
    }
    pub fn snapshot(&self) -> &DomSnapshot {
        &self.snapshot
    }
    pub fn changes(&self) -> &DomChanges {
        self.commit.changes()
    }
    pub fn bindings(&self) -> &[DomNativeHtmlBinding] {
        &self.bindings
    }
    /// Foreign-result and outside-closure subjects never alias a local index.
    pub fn node_for(&self, source: &HtmlNodeId) -> Option<NodeId> {
        self.bindings
            .iter()
            .find(|row| row.source == *source)
            .map(|row| row.node)
    }
}
