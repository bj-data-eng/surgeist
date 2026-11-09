//! HTML 877d6146487f §6.3–6.3.2 inert attributes and document modal blocking,
//! and the #the-button-element rule for a select's first button.
//!
//! DOM owns document flattening (CSS Shadow 50337328484f §4.1). A node the
//! document flat tree excludes is no node's flat-tree descendant, so an
//! ancestor's `inert` attribute never reaches it; only modal blocking can, and
//! only while the node is known to be connected. Excluded detached nodes have no
//! defined flat tree: inherited detached results remain explicitly unavailable
//! until a separately sourced DOM producer supplies that capability. Runtime
//! owns modal transitions and inert interaction effects; only the current
//! blocking subject escapes.

use super::admission::producer_revision;
use super::index::FactIndex;
use super::scan::{Caps, ScanError, ScanMeter};
use super::seen::Seen;
use super::*;
use crate::{HtmlCapturedFragmentKind, HtmlCapturedHost};
use surgeist_html::accounting::HtmlAccountingFault;
use surgeist_html::accounting::{HtmlCapScope, HtmlResource};
use surgeist_html::{
    attributes::HtmlBooleanAttribute, bounded::select::HtmlSelectButton,
    vocabulary::KnownHtmlElement,
};

identity!(
    /// Identity of the actual immutable flat-relation producer.
    HtmlFlatTreeProducerId, fresh
);

producer_revision!(
    /// The actual producer/configuration/revision key, independent of host state.
    HtmlFlatTreeRevision,
    HtmlFlatTreeProducerId
);

/// Document flat-tree membership supplied by DOM, rather than an ordinary parent.
/// Known exclusion is not empty inert ancestry, including for detached nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlDocumentFlatTreeRelation {
    Unavailable,
    Excluded,
    Root,
    Parent(HtmlCaptureSubjectId),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlInertNodeFact {
    subject: HtmlCaptureSubjectId,
    connected: Option<bool>,
    relation: HtmlDocumentFlatTreeRelation,
}
impl HtmlInertNodeFact {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        connected: Option<bool>,
        relation: HtmlDocumentFlatTreeRelation,
    ) -> Self {
        Self {
            subject,
            connected,
            relation,
        }
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn connected(&self) -> Option<bool> {
        self.connected
    }
    pub fn relation(&self) -> &HtmlDocumentFlatTreeRelation {
        &self.relation
    }
}

/// Established current blocking subject; `open` and lower modal state do not decide it.
/// `Unblocked` is an affirmative host fact, distinct from unavailable top-layer state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlDocumentModalBlocking {
    Unavailable,
    Unblocked,
    BlockedBy(HtmlCaptureSubjectId),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDocumentModalFact {
    document: HtmlCaptureSubjectId,
    blocking: HtmlDocumentModalBlocking,
}
impl HtmlDocumentModalFact {
    pub fn new(document: HtmlCaptureSubjectId, blocking: HtmlDocumentModalBlocking) -> Self {
        Self { document, blocking }
    }
    pub fn document(&self) -> &HtmlCaptureSubjectId {
        &self.document
    }
    pub fn blocking(&self) -> &HtmlDocumentModalBlocking {
        &self.blocking
    }
}

/// Finite admission for moved facts and actual constructor validation work.
/// Capacity counts both vectors' allocated record slots, including unused slots.
/// Work charges each inspected record, capture/connection hop, identifier
/// comparison and a conservative envelope for each HTML element-name lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInertFactLimits {
    record_capacity: usize,
    validation_work: usize,
}
impl HtmlInertFactLimits {
    pub fn new(record_capacity: usize, validation_work: usize) -> Self {
        Self {
            record_capacity,
            validation_work,
        }
    }
    pub fn record_capacity(self) -> usize {
        self.record_capacity
    }
    pub fn validation_work(self) -> usize {
        self.validation_work
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlInertFactError {
    Capture(HtmlCaptureError),
    HostRevisionMismatch,
    RecordCapacity {
        limit: usize,
        required: usize,
    },
    ValidationWork {
        limit: usize,
        required: usize,
    },
    RepresentationExhausted,
    /// The ledger that carries the accounting failed in a way that is not a refused limit: a release
    /// that matches nothing held, a ledger that cannot allocate, a scope that is not the
    /// operation's, or a change the operation does not charge. It carries the accounting fault.
    Accounting(HtmlAccountingFault),
    DuplicateSubject(HtmlCaptureSubjectId),
    InvalidFlatRoot(HtmlCaptureSubjectId),
    InvalidFlatParent(HtmlCaptureSubjectId),
    DocumentMismatch(HtmlCaptureSubjectId),
    ContradictoryConnectedness(HtmlCaptureSubjectId),
    ContradictoryMembership(HtmlCaptureSubjectId),
    FlatCycle(HtmlCaptureSubjectId),
    InvalidBlockingDialog(HtmlCaptureSubjectId),
    DisconnectedBlockingDialog(HtmlCaptureSubjectId),
}
impl HtmlInertFactError {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Accounting(fault) => Some(*fault),
            Self::Capture(error) => error.accounting_fault(),
            Self::HostRevisionMismatch
            | Self::RecordCapacity { .. }
            | Self::ValidationWork { .. }
            | Self::RepresentationExhausted
            | Self::DuplicateSubject(_)
            | Self::InvalidFlatRoot(_)
            | Self::InvalidFlatParent(_)
            | Self::DocumentMismatch(_)
            | Self::ContradictoryConnectedness(_)
            | Self::ContradictoryMembership(_)
            | Self::FlatCycle(_)
            | Self::InvalidBlockingDialog(_)
            | Self::DisconnectedBlockingDialog(_) => None,
        }
    }
}
impl fmt::Display for HtmlInertFactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlInertFactError {}

/// Checked immutable relation/modal supplement, without a second tree or callbacks.
///
/// DOM supplies document membership and connectedness; the host supplies current
/// document blocking state. Each is qualified by this exact capture and host
/// revision, and the relation producer's configuration/revision key. Known
/// contradictions are rejected; partial coverage remains missing at query time.
/// Detached inherited inertness still requires a future separately sourced DOM
/// ancestry capability, rather than inferring flattening from ordinary parents.
#[derive(Debug)]
pub struct HtmlInertFacts {
    capture: HtmlCaptureId,
    host_revision: HtmlHostFactsRevision,
    flat_revision: HtmlFlatTreeRevision,
    nodes: Vec<HtmlInertNodeFact>,
    documents: Vec<HtmlDocumentModalFact>,
    node_index: FactIndex,
    document_index: FactIndex,
}
enum InertConnection {
    Document(HtmlCaptureSubjectId),
    Disconnected,
    Unavailable,
}
impl InertConnection {
    fn connected(&self) -> Option<bool> {
        match self {
            Self::Document(_) => Some(true),
            Self::Disconnected => Some(false),
            Self::Unavailable => None,
        }
    }
}
impl ScanError for HtmlInertFactError {
    fn representation_exhausted() -> Self {
        Self::RepresentationExhausted
    }
    fn work_exceeded(limit: usize, required: usize) -> Self {
        Self::ValidationWork { limit, required }
    }
    fn capture(error: HtmlCaptureError) -> Self {
        Self::Capture(error)
    }
    fn accounting(fault: HtmlAccountingFault) -> Self {
        Self::Accounting(fault)
    }
}
type InertMeter = ScanMeter<HtmlInertFactError>;
fn inert_connected(
    validation: &mut InertMeter,
    capture: &HtmlCapture,
    subject: &HtmlCaptureSubjectId,
) -> Result<Option<bool>, HtmlInertFactError> {
    Ok(inert_connection(validation, capture, subject)?.connected())
}
fn inert_connection(
    validation: &mut InertMeter,
    capture: &HtmlCapture,
    subject: &HtmlCaptureSubjectId,
) -> Result<InertConnection, HtmlInertFactError> {
    let mut current = subject.clone();
    loop {
        let node = validation.node(capture, &current)?;
        validation.charge(1)?;
        if matches!(node.kind(), HtmlCapturedNodeKind::Document(_)) {
            return Ok(InertConnection::Document(current));
        }
        if let HtmlCapturedNodeKind::Fragment(fragment) = node.kind() {
            if matches!(fragment.kind(), HtmlCapturedFragmentKind::ShadowRoot(_)) {
                validation.charge(1)?;
                match node.host() {
                    Some(HtmlCapturedHost::Element(host)) => {
                        current = host.subject().clone();
                        continue;
                    }
                    Some(HtmlCapturedHost::Absent) => return Ok(InertConnection::Disconnected),
                    _ => return Ok(InertConnection::Unavailable),
                }
            }
            return Ok(InertConnection::Disconnected);
        }
        validation.charge(1)?;
        match node.parent() {
            HtmlCapturedParent::Subject(parent) => current = parent,
            HtmlCapturedParent::Root => return Ok(InertConnection::Disconnected),
            HtmlCapturedParent::Unavailable => return Ok(InertConnection::Unavailable),
        }
    }
}
/// The recorded fact of `id`, charging 1 for the probe of the index.
fn inert_node_lookup<'a>(
    validation: &mut InertMeter,
    nodes: &'a [HtmlInertNodeFact],
    index: &FactIndex,
    id: &HtmlCaptureSubjectId,
) -> Result<Option<&'a HtmlInertNodeFact>, HtmlInertFactError> {
    validation.charge(1)?;
    Ok(index.position(id).map(|position| &nodes[position]))
}
/// What a walk up the flat parents has learned about a record.
#[derive(Clone, Copy, Eq, PartialEq)]
enum FlatWalk {
    /// No walk has reached the record.
    Unreached,
    /// The walk in progress has passed through the record.
    Passing,
    /// The chain above the record is known to end without a cycle.
    Ends,
}
impl HtmlInertFacts {
    /// Moves finite supplied capacities. Partial coverage is permitted and remains
    /// missing during queries. Validates known contradictions without computing
    /// flattening or requiring unavailable records to certify another producer.
    /// Flat-parent consistency uses actual shadow-including connected documents,
    /// independently of node-document ownership. Unknown ancestry is not a
    /// document mismatch.
    pub fn new(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        flat_revision: HtmlFlatTreeRevision,
        nodes: Vec<HtmlInertNodeFact>,
        documents: Vec<HtmlDocumentModalFact>,
        limits: HtmlInertFactLimits,
    ) -> Result<Self, HtmlInertFactError> {
        Self::build(
            capture,
            host_revision,
            flat_revision,
            nodes,
            documents,
            limits,
            None,
        )
    }
    /// Charges below the scope `accounting` names, in the host's operation, as [`Self::new`]
    /// charges an operation of its own. The schema module documentation of
    /// `surgeist_html::accounting` lists the scopes it creates and what the host keeps.
    pub fn with_accounting(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        flat_revision: HtmlFlatTreeRevision,
        nodes: Vec<HtmlInertNodeFact>,
        documents: Vec<HtmlDocumentModalFact>,
        limits: HtmlInertFactLimits,
        accounting: &HtmlHostScope,
    ) -> Result<Self, HtmlInertFactError> {
        Self::build(
            capture,
            host_revision,
            flat_revision,
            nodes,
            documents,
            limits,
            Some(accounting),
        )
    }
    fn build(
        capture: &HtmlCapture,
        host_revision: HtmlHostFactsRevision,
        flat_revision: HtmlFlatTreeRevision,
        nodes: Vec<HtmlInertNodeFact>,
        documents: Vec<HtmlDocumentModalFact>,
        limits: HtmlInertFactLimits,
        hosted: Option<&HtmlHostScope>,
    ) -> Result<Self, HtmlInertFactError> {
        if capture.host_facts_revision() != &host_revision {
            return Err(HtmlInertFactError::HostRevisionMismatch);
        }
        let required = nodes
            .capacity()
            .checked_add(documents.capacity())
            .ok_or(HtmlInertFactError::RepresentationExhausted)?;
        let mut validation = InertMeter::charging(
            HtmlCapScope::InertFacts,
            Caps::work(limits.validation_work).records(limits.record_capacity),
            hosted,
        )?;
        validation.acquire(HtmlResource::FactRecords, required, |limit, required| {
            HtmlInertFactError::RecordCapacity { limit, required }
        })?;
        let mut index = FactIndex::new(
            capture.identity(),
            nodes.len(),
            |n| validation.charge(n),
            HtmlInertFactError::RepresentationExhausted,
        )?;
        let mut flat_roots = 0;
        for (position, fact) in nodes.iter().enumerate() {
            index.enter(&fact.subject, position);
            if fact.relation == HtmlDocumentFlatTreeRelation::Root {
                flat_roots += 1;
            }
        }
        // The documents that already have a flat root.
        let mut rooted = Seen::new(
            flat_roots,
            |n| validation.charge(n),
            HtmlInertFactError::RepresentationExhausted,
        )?;
        validation.charge(nodes.len())?;
        let mut walks = Vec::new();
        walks
            .try_reserve(nodes.len())
            .map_err(|_| HtmlInertFactError::RepresentationExhausted)?;
        walks.resize(nodes.len(), FlatWalk::Unreached);
        for (position, fact) in nodes.iter().enumerate() {
            let node = validation.node(capture, &fact.subject)?;
            if index.position(&fact.subject) != Some(position) {
                return Err(HtmlInertFactError::DuplicateSubject(fact.subject.clone()));
            }
            let connection = inert_connection(&mut validation, capture, &fact.subject)?;
            let established_connected = connection.connected();
            if let (Some(supplied), Some(established)) = (fact.connected, established_connected) {
                validation.charge(1)?;
                if supplied != established {
                    return Err(HtmlInertFactError::ContradictoryConnectedness(
                        fact.subject.clone(),
                    ));
                }
            }
            validation.charge(1)?;
            match &fact.relation {
                HtmlDocumentFlatTreeRelation::Root => {
                    if !matches!(node.kind(), HtmlCapturedNodeKind::Element(_)) {
                        return Err(HtmlInertFactError::InvalidFlatRoot(fact.subject.clone()));
                    }
                    validation.charge(1)?;
                    match node.parent() {
                        HtmlCapturedParent::Subject(parent) => {
                            let parent = validation.node(capture, &parent)?;
                            if !matches!(parent.kind(), HtmlCapturedNodeKind::Document(_)) {
                                return Err(HtmlInertFactError::InvalidFlatRoot(
                                    fact.subject.clone(),
                                ));
                            }
                        }
                        HtmlCapturedParent::Root => {
                            return Err(HtmlInertFactError::InvalidFlatRoot(fact.subject.clone()));
                        }
                        HtmlCapturedParent::Unavailable => {}
                    }
                    if fact.connected == Some(false) || established_connected == Some(false) {
                        return Err(HtmlInertFactError::ContradictoryMembership(
                            fact.subject.clone(),
                        ));
                    }
                    validation.charge(1)?;
                    if !rooted.first(node.node_document().subject().ordinal()) {
                        return Err(HtmlInertFactError::InvalidFlatRoot(fact.subject.clone()));
                    }
                }
                HtmlDocumentFlatTreeRelation::Parent(parent) => {
                    if matches!(
                        node.kind(),
                        HtmlCapturedNodeKind::Document(_) | HtmlCapturedNodeKind::Fragment(_)
                    ) {
                        return Err(HtmlInertFactError::InvalidFlatParent(fact.subject.clone()));
                    }
                    let parent_node = validation.node(capture, parent)?;
                    if !matches!(parent_node.kind(), HtmlCapturedNodeKind::Element(_)) {
                        return Err(HtmlInertFactError::InvalidFlatParent(parent.clone()));
                    }
                    validation.charge(1)?;
                    let parent_connection = inert_connection(&mut validation, capture, parent)?;
                    // Flat ancestry crosses actual shadow hosts. A retained
                    // root may have a different node document from its host;
                    // only known separate connected trees contradict this edge.
                    if let (InertConnection::Document(child), InertConnection::Document(parent)) =
                        (&connection, &parent_connection)
                        && child != parent
                    {
                        return Err(HtmlInertFactError::DocumentMismatch(fact.subject.clone()));
                    }
                    if fact.connected == Some(false) || established_connected == Some(false) {
                        return Err(HtmlInertFactError::ContradictoryMembership(
                            fact.subject.clone(),
                        ));
                    }
                    if parent_connection.connected() == Some(false) {
                        return Err(HtmlInertFactError::ContradictoryConnectedness(
                            parent.clone(),
                        ));
                    }
                    if let Some(parent_fact) =
                        inert_node_lookup(&mut validation, &nodes, &index, parent)?
                    {
                        if parent_fact.connected == Some(false) {
                            return Err(HtmlInertFactError::ContradictoryConnectedness(
                                parent.clone(),
                            ));
                        }
                        if parent_fact.relation == HtmlDocumentFlatTreeRelation::Excluded {
                            return Err(HtmlInertFactError::ContradictoryMembership(
                                parent.clone(),
                            ));
                        }
                    }
                }
                HtmlDocumentFlatTreeRelation::Unavailable
                | HtmlDocumentFlatTreeRelation::Excluded => {}
            }
            // The chain above this record, which can only be shorter than the
            // records. A hop to a record the walk is passing through closes a cycle;
            // a hop to a record whose chain is known to end ends this one too.
            let mut at = position;
            loop {
                validation.charge(1)?;
                match walks[at] {
                    FlatWalk::Ends => break,
                    FlatWalk::Passing => {
                        return Err(HtmlInertFactError::FlatCycle(fact.subject.clone()));
                    }
                    FlatWalk::Unreached => walks[at] = FlatWalk::Passing,
                }
                let HtmlDocumentFlatTreeRelation::Parent(parent) = &nodes[at].relation else {
                    break;
                };
                validation.charge(1)?;
                let Some(next) = index.position(parent) else {
                    break;
                };
                at = next;
            }
            // Every record the walk passed has a chain that ends.
            let mut at = position;
            loop {
                if walks[at] == FlatWalk::Ends {
                    break;
                }
                validation.charge(1)?;
                walks[at] = FlatWalk::Ends;
                let HtmlDocumentFlatTreeRelation::Parent(parent) = &nodes[at].relation else {
                    break;
                };
                let Some(next) = index.position(parent) else {
                    break;
                };
                at = next;
            }
        }
        let mut document_index = FactIndex::new(
            capture.identity(),
            documents.len(),
            |n| validation.charge(n),
            HtmlInertFactError::RepresentationExhausted,
        )?;
        for (position, fact) in documents.iter().enumerate() {
            document_index.enter(&fact.document, position);
        }
        for (position, fact) in documents.iter().enumerate() {
            validation.charge(1)?;
            capture
                .document_id(&fact.document)
                .map_err(HtmlInertFactError::Capture)?;
            if document_index.position(&fact.document) != Some(position) {
                return Err(HtmlInertFactError::DuplicateSubject(fact.document.clone()));
            }
            if let HtmlDocumentModalBlocking::BlockedBy(blocker) = &fact.blocking {
                let node = validation.node(capture, blocker)?;
                let HtmlCapturedNodeKind::Element(element) = node.kind() else {
                    return Err(HtmlInertFactError::InvalidBlockingDialog(blocker.clone()));
                };
                if validation.known(element)? != Some(KnownHtmlElement::Dialog) {
                    return Err(HtmlInertFactError::InvalidBlockingDialog(blocker.clone()));
                }
                validation.charge(1)?;
                if node.node_document().subject() != &fact.document {
                    return Err(HtmlInertFactError::DocumentMismatch(blocker.clone()));
                }
                if inert_connected(&mut validation, capture, blocker)? == Some(false) {
                    return Err(HtmlInertFactError::DisconnectedBlockingDialog(
                        blocker.clone(),
                    ));
                }
                if let Some(blocker_fact) =
                    inert_node_lookup(&mut validation, &nodes, &index, blocker)?
                    && blocker_fact.connected == Some(false)
                {
                    return Err(HtmlInertFactError::DisconnectedBlockingDialog(
                        blocker.clone(),
                    ));
                }
            }
        }
        Ok(Self {
            capture: capture.identity().clone(),
            host_revision,
            flat_revision,
            nodes,
            documents,
            node_index: index,
            document_index,
        })
    }
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.host_revision
    }
    pub fn flat_revision(&self) -> &HtmlFlatTreeRevision {
        &self.flat_revision
    }
    pub fn nodes(&self) -> &[HtmlInertNodeFact] {
        &self.nodes
    }
    pub fn documents(&self) -> &[HtmlDocumentModalFact] {
        &self.documents
    }
}

impl<'a> HtmlSemanticSession<'a> {
    fn inert_node_fact(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<Option<&'a HtmlInertNodeFact>, HtmlSemanticFailure> {
        let Some(input) = self.context.inert() else {
            return Ok(None);
        };
        self.charge(1)?;
        Ok(input
            .node_index
            .position(subject)
            .map(|at| &input.nodes[at]))
    }
    fn inert_blocking(
        &mut self,
        document: &HtmlCaptureSubjectId,
    ) -> Result<Option<&'a HtmlDocumentModalBlocking>, HtmlSemanticFailure> {
        let Some(input) = self.context.inert() else {
            return Ok(None);
        };
        self.charge(1)?;
        Ok(input
            .document_index
            .position(document)
            .map(|at| &input.documents[at].blocking))
    }
    fn own_inert(&mut self, node: HtmlCapturedNode<'a>) -> Result<bool, HtmlSemanticFailure> {
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(false);
        };
        if !self.html_namespace(element)? {
            return Ok(false);
        }
        let raw = self.attribute(element, "inert")?;
        self.admit_intrinsic(element, raw, 2)?;
        let report = self
            .interpreter
            .boolean_attribute(HtmlBooleanAttribute::Inert, element.name(), raw)
            .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        report
            .intrinsic()
            .copied()
            .ok_or_else(|| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))
    }
    fn inert_ancestry(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        blocker: Option<&HtmlCaptureSubjectId>,
        connected: Option<bool>,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let mut current = subject.clone();
        loop {
            self.charge(1)?;
            // Own attributes were inspected before advancing to this boundary.
            if blocker == Some(&current) {
                return Ok(HtmlSemanticValue::Known(false));
            }
            let Some(fact) = self.inert_node_fact(&current)? else {
                return Ok(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::InertNodeFacts(current),
                ));
            };
            self.charge(1)?;
            match &fact.relation {
                HtmlDocumentFlatTreeRelation::Parent(parent) => {
                    let node = self.node(parent)?;
                    if self.own_inert(node)? {
                        return Ok(HtmlSemanticValue::Known(true));
                    }
                    current = parent.clone();
                }
                HtmlDocumentFlatTreeRelation::Root => {
                    return Ok(match (blocker, connected) {
                        (None, _) => HtmlSemanticValue::Known(false),
                        (Some(_), Some(true)) => HtmlSemanticValue::Known(true),
                        (Some(_), _) => HtmlSemanticValue::Missing(
                            HtmlSemanticMissing::Connectedness(subject.clone()),
                        ),
                    });
                }
                HtmlDocumentFlatTreeRelation::Excluded => {
                    // Known exclusion from the document flat tree: no flat-tree
                    // ancestor exists, so no ancestor attribute applies. A
                    // connected node is inert exactly when a modal dialog
                    // blocks the document. Without known connectedness the
                    // exclusion may be detachment, for which no flat tree is
                    // defined.
                    return Ok(match (blocker, connected) {
                        (Some(_), Some(true)) => HtmlSemanticValue::Known(true),
                        (None, Some(true)) => HtmlSemanticValue::Known(false),
                        _ => {
                            HtmlSemanticValue::Missing(HtmlSemanticMissing::InertAncestry(current))
                        }
                    });
                }
                HtmlDocumentFlatTreeRelation::Unavailable => {
                    return Ok(HtmlSemanticValue::Missing(
                        HtmlSemanticMissing::InertAncestry(current),
                    ));
                }
            }
        }
    }
    /// HTML #the-button-element: a button that is the first child which is an
    /// element of a select is inert. The html owner decides the rule; this
    /// supplies only the ordinary-tree parent and first element child (never
    /// flat-tree edges or the enclosing slot/shadow structure) and charges each
    /// step it takes. The rule marks the button itself, not its descendants.
    fn select_first_button_inert(
        &mut self,
        node: HtmlCapturedNode<'a>,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Known(false));
        };
        let kind = self.known(element)?;
        let Some(button) = HtmlSelectButton::new(kind) else {
            return Ok(HtmlSemanticValue::Known(false));
        };
        let parent = match self.parent(node)? {
            Ok(Some(parent)) => parent,
            Ok(None) => return Ok(HtmlSemanticValue::Known(false)),
            Err(missing) => return Ok(HtmlSemanticValue::Missing(missing)),
        };
        let parent_node = self.node(&parent)?;
        let HtmlCapturedNodeKind::Element(parent_element) = parent_node.kind() else {
            return Ok(HtmlSemanticValue::Known(false));
        };
        let parent_kind = self.known(parent_element)?;
        let Some(select_button) = button.child_of(parent_kind) else {
            return Ok(HtmlSemanticValue::Known(false));
        };
        self.charge(1)?;
        let mut children = match parent_node.children() {
            HtmlCapturedChildren::Complete(children) => children,
            HtmlCapturedChildren::Unavailable => {
                return Ok(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::OrderedChildren(parent),
                ));
            }
        };
        let first_element_child = loop {
            self.charge(1)?;
            let Some(child) = children.next() else {
                break None;
            };
            let child_node = self.node(&child)?;
            if matches!(child_node.kind(), HtmlCapturedNodeKind::Element(_)) {
                break Some(child);
            }
        };
        Ok(HtmlSemanticValue::Known(select_button.is_inert(
            first_element_child.as_ref() == Some(&node.id()),
        )))
    }
    pub(super) fn inertness(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        if self.own_inert(node)? {
            return Ok(HtmlSemanticValue::Known(true));
        }
        // Like the node's own attribute, this needs no flat relation or modal
        // payload. Unavailable ordinary-tree evidence only matters when no
        // other source proves inertness.
        let button_rule = self.select_first_button_inert(node)?;
        if button_rule == HtmlSemanticValue::Known(true) {
            return Ok(button_rule);
        }
        let inherited = self.inherited_inertness(subject, node)?;
        Ok(match (button_rule, inherited) {
            (_, inherited @ HtmlSemanticValue::Known(true)) => inherited,
            (missing @ HtmlSemanticValue::Missing(_), _) => missing,
            (_, inherited) => inherited,
        })
    }
    fn inherited_inertness(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        node: HtmlCapturedNode<'a>,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        // A Document is intrinsically connected and cannot be the dialog or
        // any element's flat descendant. It needs only its blocking state.
        if matches!(node.kind(), HtmlCapturedNodeKind::Document(_)) {
            self.charge(1)?;
            return Ok(match self.inert_blocking(subject)? {
                Some(HtmlDocumentModalBlocking::Unblocked) => HtmlSemanticValue::Known(false),
                Some(HtmlDocumentModalBlocking::BlockedBy(_)) => HtmlSemanticValue::Known(true),
                Some(HtmlDocumentModalBlocking::Unavailable) | None => {
                    HtmlSemanticValue::Missing(HtmlSemanticMissing::ModalBlocking(subject.clone()))
                }
            });
        }
        let connected = self
            .inert_node_fact(subject)?
            .and_then(HtmlInertNodeFact::connected);
        if connected == Some(false) {
            return self.inert_ancestry(subject, None, connected);
        }
        self.charge(1)?;
        let document = node.node_document();
        let blocker = match self.inert_blocking(document.subject())? {
            Some(HtmlDocumentModalBlocking::Unblocked) => None,
            Some(HtmlDocumentModalBlocking::BlockedBy(blocker)) => Some(blocker),
            Some(HtmlDocumentModalBlocking::Unavailable) | None => {
                return Ok(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::ModalBlocking(document.subject().clone()),
                ));
            }
        };
        self.inert_ancestry(subject, blocker, connected)
    }
}
