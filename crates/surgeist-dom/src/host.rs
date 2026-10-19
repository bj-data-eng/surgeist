//! Fixed-domain external observations qualified by one canonical document.

use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::{DomDocument, DomDocumentStamp, DomError, DomRevision, NodeId};

/// A producer's explicit availability and observed value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Observed<T> {
    /// The producer supplies this value, including an established absence.
    Supported(T),
    /// The producer cannot supply this domain.
    Unsupported,
    /// The producer establishes that this domain does not apply.
    NotApplicable,
}

/// An external element-selection domain, without focus or navigation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostDomain {
    /// The producer's exact selected focus element.
    ElementFocusSelection,
    /// The producer's exact selected hover element.
    ElementHoverSelection,
    /// The producer's exact document target Element, if any.
    DocumentTargetElementSelection,
}

/// Immutable, nonempty domain configuration of one publisher.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostDomains {
    /// Focus selection only.
    Focus,
    /// Hover selection only.
    Hover,
    /// Focus and hover selection domains.
    FocusAndHover,
    /// Document target Element selection only.
    DocumentTarget,
}

/// Complete observations for exactly the domains named by its variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostObservations {
    /// One focus-selection observation.
    Focus(Observed<Option<NodeId>>),
    /// One hover-selection observation.
    Hover(Observed<Option<NodeId>>),
    /// The externally established document target Element, including known absence.
    /// This does not resolve a URL fragment, navigate or infer a target from an ID.
    /// HTML's target-elements list contains this one Element or is empty:
    /// <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#selector-target>.
    DocumentTarget(Observed<Option<NodeId>>),
    /// Both observations, with independent availability.
    FocusAndHover {
        /// Focus-selection observation.
        focus: Observed<Option<NodeId>>,
        /// Hover-selection observation.
        hover: Observed<Option<NodeId>>,
    },
}
impl DomHostObservations {
    /// Returns the domains supplied completely by this variant.
    pub fn domains(&self) -> DomHostDomains {
        match self {
            Self::Focus(_) => DomHostDomains::Focus,
            Self::Hover(_) => DomHostDomains::Hover,
            Self::FocusAndHover { .. } => DomHostDomains::FocusAndHover,
            Self::DocumentTarget(_) => DomHostDomains::DocumentTarget,
        }
    }

    fn values(self) -> [Option<Observed<Option<NodeId>>>; 3] {
        match self {
            Self::Focus(focus) => [Some(focus), None, None],
            Self::Hover(hover) => [None, Some(hover), None],
            Self::FocusAndHover { focus, hover } => [Some(focus), Some(hover), None],
            Self::DocumentTarget(target) => [None, None, Some(target)],
        }
    }
}

/// Opaque identity of one publisher and its immutable domain configuration.
#[derive(Clone)]
pub struct DomHostSourceId(Arc<()>);
impl PartialEq for DomHostSourceId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for DomHostSourceId {}
impl Hash for DomHostSourceId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
impl fmt::Debug for DomHostSourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DomHostSourceId")
    }
}

/// Checked publication counter for changed observations, independent of rendering.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HostStateRevision(u64);
impl HostStateRevision {
    /// Returns the counter for diagnostics; it cannot construct a revision.
    pub fn get(self) -> u64 {
        self.0
    }
}

/// Exact canonical basis, external publication revision and source provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomHostStamp {
    document: DomDocumentStamp,
    host_revision: HostStateRevision,
    source: DomHostSourceId,
}
impl DomHostStamp {
    /// Returns the exact owner/document/revision basis.
    pub fn document(&self) -> DomDocumentStamp {
        self.document
    }
    /// Returns the external fact publication revision.
    pub fn host_revision(&self) -> HostStateRevision {
        self.host_revision
    }
    /// Borrows the publisher identity, including immutable configuration.
    pub fn source(&self) -> &DomHostSourceId {
        &self.source
    }
}

/// Immutable external observations; retaining this does not retain live nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomHostCapture {
    stamp: DomHostStamp,
    observations: DomHostObservations,
}
impl DomHostCapture {
    /// Borrows the full publication qualification.
    pub fn stamp(&self) -> &DomHostStamp {
        &self.stamp
    }
    /// Borrows the exact observations, including availability.
    pub fn observations(&self) -> &DomHostObservations {
        &self.observations
    }
    /// Checks the canonical basis only, not whether this is the latest host state.
    pub fn validate_for(&self, document: DomDocument<'_>) -> Result<(), DomHostError> {
        validate_basis(self.stamp.document, document.stamp())
    }
}

/// Complete change of one external domain between adjacent publications.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostChange {
    /// Both observations are supported; these are their old and new selections.
    Target {
        /// Changed domain.
        domain: DomHostDomain,
        /// Previous selected element, if any.
        previous: Option<NodeId>,
        /// Current selected element, if any.
        current: Option<NodeId>,
    },
    /// Availability changed; the whole document/domain requires invalidation.
    Availability {
        /// Domain whose availability changed.
        domain: DomHostDomain,
    },
}

/// Constant-size complete adjacent coverage, ordered focus, hover, then target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomHostChanges {
    from: DomHostStamp,
    to: DomHostStamp,
    // FocusAndHover is the largest configured group. DocumentTarget is separate.
    changes: [Option<DomHostChange>; 2],
}
impl DomHostChanges {
    /// Borrows the starting qualification.
    pub fn from(&self) -> &DomHostStamp {
        &self.from
    }
    /// Borrows the resulting qualification.
    pub fn to(&self) -> &DomHostStamp {
        &self.to
    }
    /// Iterates complete domain changes in fixed domain order.
    pub fn changes(&self) -> impl Iterator<Item = &DomHostChange> {
        self.changes.iter().flatten()
    }
}

/// Why a complete adjacent delta cannot describe the requested basis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostRecomputeReason {
    /// No earlier capture was supplied.
    InitialCapture,
    /// Canonical DOM state changed independently of external facts.
    DomBasisChanged,
    /// The requested transition predates the retained adjacent transition.
    HistoryUnavailable,
}

/// Explicit complete coverage or whole-domain recomputation fallback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomHostChangeCoverage {
    /// All external changes between these stamps are listed.
    Complete(DomHostChanges),
    /// Recompute the declared domains rather than treating omitted history as empty.
    FullRecomputeRequired {
        /// Requested starting qualification, when supplied.
        from: Option<DomHostStamp>,
        /// Current publication qualification.
        to: DomHostStamp,
        /// Complete domain configuration requiring recomputation.
        domains: DomHostDomains,
        /// Reason no complete delta is available.
        reason: DomHostRecomputeReason,
    },
}

/// One coherent capture and its coverage from the preceding publication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomHostPublication {
    capture: DomHostCapture,
    changes: DomHostChangeCoverage,
}
impl DomHostPublication {
    /// Borrows the immutable observations.
    pub fn capture(&self) -> &DomHostCapture {
        &self.capture
    }
    /// Borrows the coverage emitted by this publication.
    pub fn changes(&self) -> &DomHostChangeCoverage {
        &self.changes
    }
}

/// Typed rejection; all failed publications preserve the complete previous state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomHostError {
    /// The canonical store owner differs.
    ForeignOwner,
    /// The canonical document differs.
    WrongDocument,
    /// The supplied observations and canonical view have different revisions.
    DomRevisionMismatch {
        expected: DomRevision,
        actual: DomRevision,
    },
    /// Publication would move the canonical basis backwards.
    StaleDomRevision {
        current: DomRevision,
        actual: DomRevision,
    },
    /// The variant does not completely supply this source's immutable domains.
    DomainMismatch {
        expected: DomHostDomains,
        actual: DomHostDomains,
    },
    /// A requested stamp belongs to another publisher/configuration.
    ForeignSource,
    /// A requested stamp has a later external revision.
    FutureHostRevision,
    /// A requested stamp has a later canonical revision.
    FutureDomRevision,
    /// This publisher has not published a capture yet.
    Unpublished,
    /// A selected subject is not a current same-document Element.
    Subject { node: NodeId, error: DomError },
    /// Changed observations cannot advance the external revision.
    HostRevisionExhausted,
}
impl fmt::Display for DomHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DOM host publication rejected: {self:?}")
    }
}
impl std::error::Error for DomHostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Subject { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct PublisherState {
    current: DomHostCapture,
    previous: Option<DomHostStamp>,
    transition: DomHostChangeCoverage,
}

/// Exclusive fixed-domain publisher bound to one document after first publication.
#[derive(Debug)]
pub struct DomHostPublisher {
    source: DomHostSourceId,
    domains: DomHostDomains,
    state: Option<PublisherState>,
}
impl DomHostPublisher {
    /// Mints a source whose domain configuration cannot change.
    pub fn new(domains: DomHostDomains) -> Self {
        Self {
            source: DomHostSourceId(Arc::new(())),
            domains,
            state: None,
        }
    }
    /// Returns this source's immutable domain configuration.
    pub fn domains(&self) -> DomHostDomains {
        self.domains
    }
    /// Borrows the latest successful capture, if any.
    pub fn latest(&self) -> Option<&DomHostCapture> {
        self.state.as_ref().map(|state| &state.current)
    }

    /// Validates and atomically publishes complete observations for an exact basis.
    ///
    /// A changed observation advances the checked host revision once. DOM-only
    /// changes retain it and emit full recomputation coverage. Equal recaptures
    /// preserve the actual predecessor history and return empty current coverage.
    pub fn publish(
        &mut self,
        document: DomDocument<'_>,
        observed_basis: DomDocumentStamp,
        observations: DomHostObservations,
    ) -> Result<DomHostPublication, DomHostError> {
        let basis = document.stamp();
        validate_basis(observed_basis, basis)?;
        if let Some(state) = &self.state {
            validate_document(state.current.stamp.document, basis)?;
            let current = state.current.stamp.document.store().revision();
            let actual = basis.store().revision();
            if actual < current {
                return Err(DomHostError::StaleDomRevision { current, actual });
            }
        }
        if observations.domains() != self.domains {
            return Err(DomHostError::DomainMismatch {
                expected: self.domains,
                actual: observations.domains(),
            });
        }
        for value in observations.values().into_iter().flatten() {
            if let Observed::Supported(Some(node)) = value {
                let subject = document
                    .node(node)
                    .map_err(|error| DomHostError::Subject { node, error })?;
                if subject.as_element().is_none() {
                    return Err(DomHostError::Subject {
                        node,
                        error: DomError::WrongNodeKind,
                    });
                }
            }
        }
        let revision = match &self.state {
            None => HostStateRevision(0),
            Some(state) if state.current.observations == observations => {
                state.current.stamp.host_revision
            }
            Some(state) => HostStateRevision(
                state
                    .current
                    .stamp
                    .host_revision
                    .0
                    .checked_add(1)
                    .ok_or(DomHostError::HostRevisionExhausted)?,
            ),
        };
        let stamp = DomHostStamp {
            document: basis,
            host_revision: revision,
            source: self.source.clone(),
        };
        if let Some(state) = &self.state
            && stamp == state.current.stamp
        {
            return Ok(DomHostPublication {
                capture: state.current.clone(),
                changes: empty_changes(&stamp),
            });
        }
        let capture = DomHostCapture {
            stamp: stamp.clone(),
            observations,
        };
        let previous = self.latest().map(|capture| capture.stamp.clone());
        let changes = match &self.state {
            None => self.fallback(None, &stamp, DomHostRecomputeReason::InitialCapture),
            Some(state) if state.current.stamp.document != basis => self.fallback(
                previous.clone(),
                &stamp,
                DomHostRecomputeReason::DomBasisChanged,
            ),
            Some(state) => {
                let old = state.current.observations.values();
                let new = observations.values();
                DomHostChangeCoverage::Complete(DomHostChanges {
                    from: state.current.stamp.clone(),
                    to: stamp,
                    changes: match self.domains {
                        DomHostDomains::DocumentTarget => [
                            change(
                                DomHostDomain::DocumentTargetElementSelection,
                                old[2],
                                new[2],
                            ),
                            None,
                        ],
                        DomHostDomains::Focus
                        | DomHostDomains::Hover
                        | DomHostDomains::FocusAndHover => [
                            change(DomHostDomain::ElementFocusSelection, old[0], new[0]),
                            change(DomHostDomain::ElementHoverSelection, old[1], new[1]),
                        ],
                    },
                })
            }
        };
        self.state = Some(PublisherState {
            current: capture.clone(),
            previous,
            transition: changes.clone(),
        });
        Ok(DomHostPublication { capture, changes })
    }

    /// Returns complete current/adjacent coverage or explicit bounded-history fallback.
    pub fn changes_since(
        &self,
        from: Option<&DomHostStamp>,
    ) -> Result<DomHostChangeCoverage, DomHostError> {
        let state = self.state.as_ref().ok_or(DomHostError::Unpublished)?;
        let to = &state.current.stamp;
        let Some(from) = from else {
            return Ok(self.fallback(None, to, DomHostRecomputeReason::InitialCapture));
        };
        if from.source != self.source {
            return Err(DomHostError::ForeignSource);
        }
        validate_document(to.document, from.document)?;
        if from.host_revision > to.host_revision {
            return Err(DomHostError::FutureHostRevision);
        }
        if from.document.store().revision() > to.document.store().revision() {
            return Err(DomHostError::FutureDomRevision);
        }
        if from == to {
            return Ok(empty_changes(to));
        }
        if state.previous.as_ref() == Some(from) {
            return Ok(state.transition.clone());
        }
        let reason = if from.document != to.document {
            DomHostRecomputeReason::DomBasisChanged
        } else {
            DomHostRecomputeReason::HistoryUnavailable
        };
        Ok(self.fallback(Some(from.clone()), to, reason))
    }

    fn fallback(
        &self,
        from: Option<DomHostStamp>,
        to: &DomHostStamp,
        reason: DomHostRecomputeReason,
    ) -> DomHostChangeCoverage {
        DomHostChangeCoverage::FullRecomputeRequired {
            from,
            to: to.clone(),
            domains: self.domains,
            reason,
        }
    }
}

fn validate_document(
    expected: DomDocumentStamp,
    actual: DomDocumentStamp,
) -> Result<(), DomHostError> {
    if expected.store().owner() != actual.store().owner() {
        return Err(DomHostError::ForeignOwner);
    }
    if expected.document() != actual.document() {
        return Err(DomHostError::WrongDocument);
    }
    Ok(())
}
fn validate_basis(
    expected: DomDocumentStamp,
    actual: DomDocumentStamp,
) -> Result<(), DomHostError> {
    validate_document(expected, actual)?;
    if expected.store().revision() != actual.store().revision() {
        return Err(DomHostError::DomRevisionMismatch {
            expected: expected.store().revision(),
            actual: actual.store().revision(),
        });
    }
    Ok(())
}
fn empty_changes(stamp: &DomHostStamp) -> DomHostChangeCoverage {
    DomHostChangeCoverage::Complete(DomHostChanges {
        from: stamp.clone(),
        to: stamp.clone(),
        changes: [None, None],
    })
}
fn change(
    domain: DomHostDomain,
    old: Option<Observed<Option<NodeId>>>,
    new: Option<Observed<Option<NodeId>>>,
) -> Option<DomHostChange> {
    if old == new {
        return None;
    }
    match (old, new) {
        (Some(Observed::Supported(previous)), Some(Observed::Supported(current))) => {
            Some(DomHostChange::Target {
                domain,
                previous,
                current,
            })
        }
        _ => Some(DomHostChange::Availability { domain }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dom, DomBatch, DomLimits};
    use std::sync::atomic::AtomicBool;

    #[test]
    fn target_revision_exhaustion_preserves_history_and_allows_dom_only_rebase() {
        let mut dom = Dom::new(
            DomLimits::new(
                8,
                512,
                16,
                0,
                0,
                2,
                4096,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let element = batch.create_element(&document, "target");
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let element = commit.created(&element).unwrap().unwrap();
        let id = commit.document(&document).unwrap();
        let old = dom.snapshot();
        let document = old.document(id).unwrap();
        let mut publisher = DomHostPublisher::new(DomHostDomains::DocumentTarget);
        let observations = DomHostObservations::DocumentTarget(Observed::Supported(None));
        publisher
            .publish(document, document.stamp(), observations)
            .unwrap();
        publisher
            .state
            .as_mut()
            .unwrap()
            .current
            .stamp
            .host_revision = HostStateRevision(u64::MAX);
        let before = publisher.latest().unwrap().clone();
        let coverage = publisher.changes_since(None).unwrap();
        assert_eq!(
            publisher.publish(
                document,
                document.stamp(),
                DomHostObservations::DocumentTarget(Observed::Supported(Some(element)))
            ),
            Err(DomHostError::HostRevisionExhausted)
        );
        assert_eq!(publisher.latest(), Some(&before));
        assert_eq!(publisher.changes_since(None).unwrap(), coverage);
        assert_eq!(
            publisher
                .publish(document, document.stamp(), observations)
                .unwrap()
                .capture(),
            &before
        );
        let mut batch = DomBatch::new();
        batch.create_comment(id, "later");
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let snapshot = dom.snapshot();
        let document = snapshot.document(id).unwrap();
        let rebased = publisher
            .publish(document, document.stamp(), observations)
            .unwrap();
        assert_eq!(rebased.capture().stamp().host_revision().get(), u64::MAX);
        assert!(matches!(
            rebased.changes(),
            DomHostChangeCoverage::FullRecomputeRequired {
                domains: DomHostDomains::DocumentTarget,
                reason: DomHostRecomputeReason::DomBasisChanged,
                ..
            }
        ));
        assert_eq!(before.observations(), &observations);
    }

    #[test]
    fn exhausted_host_revision_allows_equal_recapture_and_rejects_changed_facts_atomically() {
        let mut dom = Dom::new(
            DomLimits::new(
                8,
                512,
                8,
                0,
                0,
                2,
                4096,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let snapshot = dom.snapshot();
        let document = snapshot
            .document(commit.document(&document).unwrap())
            .unwrap();
        let mut publisher = DomHostPublisher::new(DomHostDomains::Focus);
        publisher
            .publish(
                document,
                document.stamp(),
                DomHostObservations::Focus(Observed::Supported(None)),
            )
            .unwrap();
        publisher
            .state
            .as_mut()
            .unwrap()
            .current
            .stamp
            .host_revision = HostStateRevision(u64::MAX);
        let before = publisher.latest().unwrap().clone();
        let equal = publisher
            .publish(document, document.stamp(), *before.observations())
            .unwrap();
        assert_eq!(equal.capture(), &before);
        let coverage = publisher.changes_since(None).unwrap();
        assert_eq!(
            publisher.publish(
                document,
                document.stamp(),
                DomHostObservations::Focus(Observed::Unsupported)
            ),
            Err(DomHostError::HostRevisionExhausted)
        );
        assert_eq!(publisher.latest(), Some(&before));
        assert_eq!(publisher.changes_since(None).unwrap(), coverage);
        let mut future = before.stamp.clone();
        let mut batch = DomBatch::new();
        batch.create_comment(document.id(), "later");
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
        future.document = dom.snapshot().document(document.id()).unwrap().stamp();
        assert_eq!(
            publisher.changes_since(Some(&future)),
            Err(DomHostError::FutureDomRevision)
        );
        publisher
            .state
            .as_mut()
            .unwrap()
            .current
            .stamp
            .host_revision = HostStateRevision(0);
        assert_eq!(
            publisher.changes_since(Some(&before.stamp)),
            Err(DomHostError::FutureHostRevision)
        );
    }
}
