//! HTML 877d6146487f §6.3–6.3.2 inert attributes and document modal blocking.
//!
//! DOM owns document flattening (CSS Shadow 50337328484f §4.1). Excluded
//! document membership does not establish detached semantic inert ancestry:
//! inherited detached results remain explicitly unavailable until a separately
//! sourced DOM producer supplies that capability. Runtime owns modal transitions
//! and inert interaction effects; only the current blocking subject escapes.

use super::*;
use crate::{HtmlCapturedFragmentKind, HtmlCapturedHost};

/// Identity of the actual immutable flat-relation producer.
#[derive(Clone, Debug)]
pub struct HtmlFlatTreeProducerId(Arc<()>);
impl HtmlFlatTreeProducerId {
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for HtmlFlatTreeProducerId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlFlatTreeProducerId {}

/// The actual producer/configuration/revision key, independent of host state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlFlatTreeRevision {
    producer: HtmlFlatTreeProducerId,
    configuration: u64,
    revision: u64,
}
impl HtmlFlatTreeRevision {
    pub fn new(producer: HtmlFlatTreeProducerId, configuration: u64, revision: u64) -> Self {
        Self {
            producer,
            configuration,
            revision,
        }
    }
    pub fn producer(&self) -> &HtmlFlatTreeProducerId {
        &self.producer
    }
    pub fn configuration(&self) -> u64 {
        self.configuration
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}

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
    RecordCapacity { limit: usize, required: usize },
    ValidationWork { limit: usize, required: usize },
    RepresentationExhausted,
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
}
struct InertValidation {
    work: usize,
    limit: usize,
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
impl InertValidation {
    fn charge(&mut self, amount: usize) -> Result<(), HtmlInertFactError> {
        let required = self
            .work
            .checked_add(amount)
            .ok_or(HtmlInertFactError::RepresentationExhausted)?;
        if required > self.limit {
            return Err(HtmlInertFactError::ValidationWork {
                limit: self.limit,
                required,
            });
        }
        self.work = required;
        Ok(())
    }
    fn node<'a>(
        &mut self,
        capture: &'a HtmlCapture,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlCapturedNode<'a>, HtmlInertFactError> {
        self.charge(1)?;
        capture.node(id).map_err(HtmlInertFactError::Capture)
    }
    fn fact<'a>(
        &mut self,
        nodes: &'a [HtmlInertNodeFact],
        id: &HtmlCaptureSubjectId,
    ) -> Result<Option<&'a HtmlInertNodeFact>, HtmlInertFactError> {
        for fact in nodes {
            self.charge(1)?;
            if fact.subject == *id {
                return Ok(Some(fact));
            }
        }
        Ok(None)
    }
    fn connected(
        &mut self,
        capture: &HtmlCapture,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<Option<bool>, HtmlInertFactError> {
        Ok(self.connection(capture, subject)?.connected())
    }
    fn connection(
        &mut self,
        capture: &HtmlCapture,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<InertConnection, HtmlInertFactError> {
        let mut current = subject.clone();
        loop {
            let node = self.node(capture, &current)?;
            self.charge(1)?;
            if matches!(node.kind(), HtmlCapturedNodeKind::Document(_)) {
                return Ok(InertConnection::Document(current));
            }
            if let HtmlCapturedNodeKind::Fragment(fragment) = node.kind() {
                if matches!(fragment.kind(), HtmlCapturedFragmentKind::ShadowRoot(_)) {
                    self.charge(1)?;
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
            self.charge(1)?;
            match node.parent() {
                HtmlCapturedParent::Subject(parent) => current = parent,
                HtmlCapturedParent::Root => return Ok(InertConnection::Disconnected),
                HtmlCapturedParent::Unavailable => return Ok(InertConnection::Unavailable),
            }
        }
    }
    fn known(
        &mut self,
        element: &HtmlCapturedElement,
    ) -> Result<Option<KnownHtmlElement>, HtmlInertFactError> {
        let units = element
            .name()
            .local_name()
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(KnownHtmlElement::ALL.len()))
            .and_then(|n| {
                n.checked_add(
                    element
                        .name()
                        .namespace()
                        .uri()
                        .map_or(0, DocumentString::len),
                )
            })
            .ok_or(HtmlInertFactError::RepresentationExhausted)?;
        self.charge(units)?;
        Ok(KnownHtmlElement::from_name(element.name()))
    }
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
        if capture.host_facts_revision() != &host_revision {
            return Err(HtmlInertFactError::HostRevisionMismatch);
        }
        let required = nodes
            .capacity()
            .checked_add(documents.capacity())
            .ok_or(HtmlInertFactError::RepresentationExhausted)?;
        if required > limits.record_capacity {
            return Err(HtmlInertFactError::RecordCapacity {
                limit: limits.record_capacity,
                required,
            });
        }
        let mut validation = InertValidation {
            work: 0,
            limit: limits.validation_work,
        };
        for (index, fact) in nodes.iter().enumerate() {
            let node = validation.node(capture, &fact.subject)?;
            for previous in &nodes[..index] {
                validation.charge(1)?;
                if previous.subject == fact.subject {
                    return Err(HtmlInertFactError::DuplicateSubject(fact.subject.clone()));
                }
            }
            let connection = validation.connection(capture, &fact.subject)?;
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
                    for previous in &nodes[..index] {
                        validation.charge(1)?;
                        if previous.relation == HtmlDocumentFlatTreeRelation::Root {
                            let previous_node = validation.node(capture, &previous.subject)?;
                            validation.charge(1)?;
                            if previous_node.node_document() == node.node_document() {
                                return Err(HtmlInertFactError::InvalidFlatRoot(
                                    fact.subject.clone(),
                                ));
                            }
                        }
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
                    let parent_connection = validation.connection(capture, parent)?;
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
                    if let Some(parent_fact) = validation.fact(&nodes, parent)? {
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
            let mut current = fact.subject.clone();
            // Every complete link resolves to a unique record. More than the
            // record count cannot occur without revisiting a record (a cycle).
            for traversed in 0..=nodes.len() {
                let Some(current_fact) = validation.fact(&nodes, &current)? else {
                    break;
                };
                validation.charge(1)?;
                let HtmlDocumentFlatTreeRelation::Parent(parent) = &current_fact.relation else {
                    break;
                };
                if traversed == nodes.len() {
                    return Err(HtmlInertFactError::FlatCycle(fact.subject.clone()));
                }
                current = parent.clone();
            }
        }
        for (index, fact) in documents.iter().enumerate() {
            validation.charge(1)?;
            capture
                .document_id(&fact.document)
                .map_err(HtmlInertFactError::Capture)?;
            for previous in &documents[..index] {
                validation.charge(1)?;
                if previous.document == fact.document {
                    return Err(HtmlInertFactError::DuplicateSubject(fact.document.clone()));
                }
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
                if validation.connected(capture, blocker)? == Some(false) {
                    return Err(HtmlInertFactError::DisconnectedBlockingDialog(
                        blocker.clone(),
                    ));
                }
                if let Some(blocker_fact) = validation.fact(&nodes, blocker)?
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
        for fact in input.nodes() {
            self.charge(1)?;
            if fact.subject == *subject {
                return Ok(Some(fact));
            }
        }
        Ok(None)
    }
    fn inert_blocking(
        &mut self,
        document: &HtmlCaptureSubjectId,
    ) -> Result<Option<&'a HtmlDocumentModalBlocking>, HtmlSemanticFailure> {
        let Some(input) = self.context.inert() else {
            return Ok(None);
        };
        for fact in input.documents() {
            self.charge(1)?;
            if fact.document == *document {
                return Ok(Some(&fact.blocking));
            }
        }
        Ok(None)
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
                    return Ok(if blocker.is_some() && connected == Some(true) {
                        HtmlSemanticValue::Known(true)
                    } else {
                        HtmlSemanticValue::Missing(HtmlSemanticMissing::InertAncestry(current))
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
    pub(super) fn inertness(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        if self.own_inert(node)? {
            return Ok(HtmlSemanticValue::Known(true));
        }
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
