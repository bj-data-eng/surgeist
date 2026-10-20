use super::*;

/// One neutral consuming owner/session identity; not a canonical-state registry.
#[derive(Clone, Debug)]
pub struct HtmlCspOwnerId(pub(super) Arc<()>);
impl PartialEq for HtmlCspOwnerId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlCspOwnerId {}

/// A subject issued for one actual Document binding by its consuming adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCspDocumentId {
    pub(super) owner: HtmlCspOwnerId,
    pub(super) serial: u64,
}
impl HtmlCspDocumentId {
    pub fn owner(&self) -> &HtmlCspOwnerId {
        &self.owner
    }
}

#[derive(Clone, Debug)]
pub(super) struct ProcessorId(pub Arc<()>);
impl PartialEq for ProcessorId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ProcessorId {}

/// An external backend's declared semantic version under its actual issuer.
/// Declaring a profile attests backend support; it does not activate a service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCspProcessorProfile {
    pub(super) issuer: ProcessorId,
    pub(super) version: u64,
}
impl HtmlCspProcessorProfile {
    pub fn semantic_version(&self) -> u64 {
        self.version
    }
    /// Validate the explicitly selected service before invoking it. This checks
    /// only immutable qualification, not backend availability or processing.
    pub fn validate_selected_with<E>(
        &self,
        selected: &Self,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<(), HtmlCspError<E>> {
        admit(HtmlCspLayout::read::<Self>(4)).map_err(HtmlCspError::Interrupted)?;
        self.require(selected)?;
        Ok(())
    }
    pub(super) fn require(&self, actual: &Self) -> Result<(), HtmlCspProtocolError> {
        if self.issuer != actual.issuer {
            return Err(HtmlCspProtocolError::WrongProcessor);
        }
        if self.version != actual.version {
            return Err(HtmlCspProtocolError::WrongProfile);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum PolicyIssuer {
    Initial(HtmlCspDocumentId),
    External(ProcessorId),
}

/// Identity of one immutable policy state, independent of its text or Rust type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCspPolicyId {
    pub(super) issuer: PolicyIssuer,
    pub(super) serial: u64,
}

/// Issuer-scoped immutable-state revision; no caller arithmetic is defined.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCspPolicyRevision(pub(super) u64);

/// Per-Document accepted-occurrence generation, distinct from policy revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCspAssociationGeneration {
    pub(super) document: HtmlCspDocumentId,
    pub(super) serial: u64,
}

/// A nonreused reached insertion identity, not execution authority on its own.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMetaCspOccurrence {
    pub(super) document: HtmlCspDocumentId,
    pub(super) serial: u64,
}
