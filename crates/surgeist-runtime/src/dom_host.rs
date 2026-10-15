//! Checked consumer adapter from Runtime selections to DOM host publications.

use crate::{Runtime, SurfaceElementRef, SurfaceError, SurfaceRef, UiSurface, WindowId};
use std::{fmt, sync::Arc};
use surgeist_dom::{
    DomDocument, DomDocumentStamp, DomError, DomHostCapture, DomHostChangeCoverage, DomHostDomains,
    DomHostError, DomHostObservations, DomHostPublication, DomHostPublisher, DomHostStamp, NodeId,
    Observed,
};

/// Finite admission for actual variable-size surface-to-DOM associations.
///
/// Bind and rebind charge one credit for each pair dispatch, earlier-pair
/// comparison, source-reference validation, canonical subject validation,
/// registry-entry dispatch and association comparison during registry coverage.
/// Capture charges one credit for each association examined for each requested
/// selected target. An absent selection needs no lookup. Credits are charged
/// before the operation; constant-size identity and stamp checks need no credits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SurfaceDomBindingLimits {
    association_capacity: usize,
    validation_work: usize,
}
impl SurfaceDomBindingLimits {
    /// Selects the retained Vec capacity and per-operation comparison/work limits.
    /// Zero permits an empty binding and no variable work.
    pub const fn new(association_capacity: usize, validation_work: usize) -> Self {
        Self {
            association_capacity,
            validation_work,
        }
    }
    /// Returns the maximum retained association Vec capacity, not merely its length.
    pub fn association_capacity(self) -> usize {
        self.association_capacity
    }
    /// Returns the work limit for each bind, rebind or capture operation.
    pub fn validation_work(self) -> usize {
        self.validation_work
    }
}

/// Exclusive association and publisher bound to one Runtime instance and surface.
#[derive(Debug)]
pub struct RuntimeDomHost {
    owner: RuntimeDomHostOwner,
    surface: SurfaceRef,
    document: DomDocumentStamp,
    associations: Vec<(SurfaceElementRef, NodeId)>,
    limits: SurfaceDomBindingLimits,
    publisher: DomHostPublisher,
}
struct RuntimeDomHostOwner(Arc<()>);
impl fmt::Debug for RuntimeDomHostOwner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RuntimeDomHostOwner")
    }
}
impl RuntimeDomHost {
    /// Returns the generation-qualified source surface.
    pub fn surface(&self) -> SurfaceRef {
        self.surface
    }
    /// Returns the exact canonical basis required by capture.
    pub fn document(&self) -> DomDocumentStamp {
        self.document
    }
    /// Returns the immutable external domain configuration.
    pub fn domains(&self) -> DomHostDomains {
        self.publisher.domains()
    }
    /// Borrows the last successful publication, which need not be current Runtime state.
    pub fn latest(&self) -> Option<&DomHostCapture> {
        self.publisher.latest()
    }
    /// Obtains complete adjacent coverage or explicit recomputation fallback.
    pub fn changes_since(
        &self,
        from: Option<&DomHostStamp>,
    ) -> Result<DomHostChangeCoverage, DomHostError> {
        self.publisher.changes_since(from)
    }
}

/// A coherent Runtime selection capture with surface/window provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDomHostPublication {
    surface: SurfaceRef,
    window: WindowId,
    publication: DomHostPublication,
}
impl RuntimeDomHostPublication {
    /// Returns the source surface and checked generation.
    pub fn surface(&self) -> SurfaceRef {
        self.surface
    }
    /// Returns the actual source window.
    pub fn window(&self) -> WindowId {
        self.window
    }
    /// Borrows the canonical-document-qualified external publication.
    pub fn publication(&self) -> &DomHostPublication {
        &self.publication
    }
}

/// Typed adapter rejection; failed operations preserve binding and publisher state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeDomHostError {
    /// Runtime has no producer for the requested external domain configuration.
    UnsupportedDomains(DomHostDomains),
    /// The adapter belongs to another Runtime instance, even if numeric IDs coincide.
    ForeignRuntime,
    /// The source surface/reference/lifecycle failed its owning validation.
    Surface(SurfaceError),
    /// The canonical external publication failed its owning validation.
    Host(DomHostError),
    /// Capture requires the exact canonical basis admitted by bind/rebind.
    BindingDomBasisMismatch {
        /// Admitted canonical basis.
        expected: DomDocumentStamp,
        /// Supplied canonical basis.
        actual: DomDocumentStamp,
    },
    /// An association names a foreign, stale, wrong-document or non-Element subject.
    DomSubject {
        /// Responsible canonical identity.
        node: NodeId,
        /// Original owning validation error.
        error: DomError,
    },
    /// A registered source element has no canonical association.
    MissingAssociation(SurfaceElementRef),
    /// A source reference occurs more than once.
    DuplicateSurfaceElement(SurfaceElementRef),
    /// A canonical element occurs more than once.
    DuplicateDomElement(NodeId),
    /// The retained association allocation exceeds its explicit limit.
    AssociationCapacity {
        /// Configured Vec capacity limit.
        limit: usize,
        /// Actual incoming Vec capacity.
        required: usize,
    },
    /// The next validation/comparison operation would exceed its finite budget.
    ValidationWork {
        /// Configured per-operation work limit.
        limit: usize,
        /// Credits required through the rejected operation.
        required: usize,
    },
    /// Checked work accounting overflowed.
    RepresentationExhausted,
}
impl fmt::Display for RuntimeDomHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime DOM host operation rejected: {self:?}")
    }
}
impl std::error::Error for RuntimeDomHostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Surface(error) => Some(error),
            Self::Host(error) => Some(error),
            Self::DomSubject { error, .. } => Some(error),
            _ => None,
        }
    }
}

impl<State, R, Input> Runtime<State, R, Input> {
    /// Admits a complete one-to-one association for the current root registry.
    ///
    /// References are validated by their actual owners. Numeric IDs are never
    /// converted between Runtime and DOM. The domain configuration is immutable.
    /// Document target observations have no Runtime producer and are rejected
    /// before surface/association admission; use the native DOM publisher when
    /// an actual external host supplies that selection.
    pub fn bind_dom_host(
        &self,
        surface: SurfaceRef,
        document: DomDocument<'_>,
        associations: Vec<(SurfaceElementRef, NodeId)>,
        domains: DomHostDomains,
        limits: SurfaceDomBindingLimits,
    ) -> Result<RuntimeDomHost, RuntimeDomHostError> {
        if domains == DomHostDomains::DocumentTarget {
            return Err(RuntimeDomHostError::UnsupportedDomains(domains));
        }
        let current = self
            .current_surface(surface)
            .map_err(RuntimeDomHostError::Surface)?;
        validate_associations(current, document, &associations, limits)?;
        Ok(RuntimeDomHost {
            owner: RuntimeDomHostOwner(self.dom_host_owner.clone()),
            surface,
            document: document.stamp(),
            associations,
            limits,
            publisher: DomHostPublisher::new(domains),
        })
    }

    /// Atomically replaces associations for the same Runtime/surface/document.
    /// Forward canonical revisions are allowed; old captures and source identity
    /// survive. Surface generation or domain changes require a new adapter.
    pub fn rebind_dom_host(
        &self,
        host: &mut RuntimeDomHost,
        document: DomDocument<'_>,
        associations: Vec<(SurfaceElementRef, NodeId)>,
        limits: SurfaceDomBindingLimits,
    ) -> Result<(), RuntimeDomHostError> {
        let current = self.dom_host_surface(host)?;
        let expected = host.document;
        let actual = document.stamp();
        if expected.store().owner() != actual.store().owner() {
            return Err(RuntimeDomHostError::Host(DomHostError::ForeignOwner));
        }
        if expected.document() != actual.document() {
            return Err(RuntimeDomHostError::Host(DomHostError::WrongDocument));
        }
        let revision = expected.store().revision();
        if actual.store().revision() < revision {
            return Err(RuntimeDomHostError::Host(DomHostError::StaleDomRevision {
                current: revision,
                actual: actual.store().revision(),
            }));
        }
        validate_associations(current, document, &associations, limits)?;
        host.associations = associations;
        host.document = actual;
        host.limits = limits;
        Ok(())
    }

    /// Publishes the actual requested UiSurface element selections coherently.
    ///
    /// These are Runtime selections, not complete HTML focus, focus-visible or
    /// pointer processing. Rendering and its acknowledgements are independent.
    pub fn capture_dom_host(
        &self,
        host: &mut RuntimeDomHost,
        document: DomDocument<'_>,
    ) -> Result<RuntimeDomHostPublication, RuntimeDomHostError> {
        let current = self.dom_host_surface(host)?;
        if host.document != document.stamp() {
            return Err(RuntimeDomHostError::BindingDomBasisMismatch {
                expected: host.document,
                actual: document.stamp(),
            });
        }
        let mut work = BindingWork::new(host.limits.validation_work);
        let observations = match host.publisher.domains() {
            DomHostDomains::DocumentTarget => {
                return Err(RuntimeDomHostError::UnsupportedDomains(
                    DomHostDomains::DocumentTarget,
                ));
            }
            DomHostDomains::Focus => DomHostObservations::Focus(Observed::Supported(selected(
                current.focused_element(),
                &host.associations,
                &mut work,
            )?)),
            DomHostDomains::Hover => DomHostObservations::Hover(Observed::Supported(selected(
                current.hovered_element(),
                &host.associations,
                &mut work,
            )?)),
            DomHostDomains::FocusAndHover => DomHostObservations::FocusAndHover {
                focus: Observed::Supported(selected(
                    current.focused_element(),
                    &host.associations,
                    &mut work,
                )?),
                hover: Observed::Supported(selected(
                    current.hovered_element(),
                    &host.associations,
                    &mut work,
                )?),
            },
        };
        let publication = host
            .publisher
            .publish(document, host.document, observations)
            .map_err(RuntimeDomHostError::Host)?;
        Ok(RuntimeDomHostPublication {
            surface: host.surface,
            window: current.window_id(),
            publication,
        })
    }

    fn dom_host_surface(&self, host: &RuntimeDomHost) -> Result<&UiSurface, RuntimeDomHostError> {
        if !Arc::ptr_eq(&self.dom_host_owner, &host.owner.0) {
            return Err(RuntimeDomHostError::ForeignRuntime);
        }
        let current = self
            .current_surface(host.surface)
            .map_err(RuntimeDomHostError::Surface)?;
        current
            .ensure_not_terminal()
            .map_err(RuntimeDomHostError::Surface)?;
        Ok(current)
    }
}

struct BindingWork {
    used: usize,
    limit: usize,
}
impl BindingWork {
    fn new(limit: usize) -> Self {
        Self { used: 0, limit }
    }
    fn charge(&mut self) -> Result<(), RuntimeDomHostError> {
        let required = self
            .used
            .checked_add(1)
            .ok_or(RuntimeDomHostError::RepresentationExhausted)?;
        if required > self.limit {
            return Err(RuntimeDomHostError::ValidationWork {
                limit: self.limit,
                required,
            });
        }
        self.used = required;
        Ok(())
    }
}

fn validate_associations(
    surface: &UiSurface,
    document: DomDocument<'_>,
    associations: &Vec<(SurfaceElementRef, NodeId)>,
    limits: SurfaceDomBindingLimits,
) -> Result<(), RuntimeDomHostError> {
    surface
        .ensure_not_terminal()
        .map_err(RuntimeDomHostError::Surface)?;
    if associations.capacity() > limits.association_capacity {
        return Err(RuntimeDomHostError::AssociationCapacity {
            limit: limits.association_capacity,
            required: associations.capacity(),
        });
    }
    let mut work = BindingWork::new(limits.validation_work);
    for (index, &(reference, node)) in associations.iter().enumerate() {
        work.charge()?;
        for &(prior_reference, prior_node) in &associations[..index] {
            work.charge()?;
            if reference == prior_reference {
                return Err(RuntimeDomHostError::DuplicateSurfaceElement(reference));
            }
            if node == prior_node {
                return Err(RuntimeDomHostError::DuplicateDomElement(node));
            }
        }
        work.charge()?;
        surface
            .validate_element_ref(reference)
            .map_err(RuntimeDomHostError::Surface)?;
        work.charge()?;
        let subject = document
            .node(node)
            .map_err(|error| RuntimeDomHostError::DomSubject { node, error })?;
        if subject.as_element().is_none() {
            return Err(RuntimeDomHostError::DomSubject {
                node,
                error: DomError::WrongNodeKind,
            });
        }
    }
    for registration in surface.root().elements().iter() {
        work.charge()?;
        let reference = SurfaceElementRef::new(surface.surface_ref(), registration.id());
        selected(Some(reference), associations, &mut work)?;
    }
    Ok(())
}

fn selected(
    reference: Option<SurfaceElementRef>,
    associations: &[(SurfaceElementRef, NodeId)],
    work: &mut BindingWork,
) -> Result<Option<NodeId>, RuntimeDomHostError> {
    let Some(reference) = reference else {
        return Ok(None);
    };
    for &(candidate, node) in associations {
        work.charge()?;
        if candidate == reference {
            return Ok(Some(node));
        }
    }
    Err(RuntimeDomHostError::MissingAssociation(reference))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn work_accounting_exhaustion_is_checked() {
        let mut work = BindingWork {
            used: usize::MAX,
            limit: usize::MAX,
        };
        assert_eq!(
            work.charge(),
            Err(RuntimeDomHostError::RepresentationExhausted)
        );
        assert_eq!(work.used, usize::MAX);
    }
}
