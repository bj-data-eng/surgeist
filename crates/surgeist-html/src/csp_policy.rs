//! Neutral qualified Meta CSP transitions over caller-owned native artifacts.
//! No CSP grammar, live policy registry, tree guard, or execution service lives here.
use crate::DocumentString;
use std::{any::Any, error::Error, fmt, mem::size_of, sync::Arc};

mod accounting;
mod identity;
mod transition;
pub use accounting::*;
pub use identity::*;
pub use transition::*;

/// Invalid protocol input or exhausted checked representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCspProtocolError {
    kind: HtmlCspProtocolErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlCspProtocolErrorKind {
    ForeignOwner,
    ForeignDocument,
    WrongProcessor,
    WrongProfile,
    WrongRequest,
    StaleAssociation,
    PriorPolicyMismatch,
    WrongNativeArtifactType,
    EmptyContent,
    RepresentationExhausted,
    InvalidStorage,
}

impl HtmlCspProtocolErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlCspProtocolError {
        HtmlCspProtocolError { kind: self }
    }
}

impl From<HtmlCspProtocolErrorKind> for HtmlCspProtocolError {
    fn from(kind: HtmlCspProtocolErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlCspProtocolError {
    /// Whether the cause is: policy state belongs to a different CSP owner.
    pub fn is_foreign_owner(&self) -> bool {
        matches!(self.kind, HtmlCspProtocolErrorKind::ForeignOwner)
    }
    /// Whether the cause is: association belongs to a different document.
    pub fn is_foreign_document(&self) -> bool {
        matches!(self.kind, HtmlCspProtocolErrorKind::ForeignDocument)
    }
    /// Whether the cause is: processor does not match the request.
    pub fn is_wrong_processor(&self) -> bool {
        matches!(self.kind, HtmlCspProtocolErrorKind::WrongProcessor)
    }
    /// Whether the cause is: processor profile does not match the request.
    pub fn is_wrong_profile(&self) -> bool {
        matches!(self.kind, HtmlCspProtocolErrorKind::WrongProfile)
    }
    /// Whether the cause is: request does not match the pending meta element state.
    pub fn is_wrong_request(&self) -> bool {
        matches!(self.kind, HtmlCspProtocolErrorKind::WrongRequest)
    }
    /// Whether the cause is: association is stale.
    pub fn is_stale_association(&self) -> bool {
        matches!(self.kind, HtmlCspProtocolErrorKind::StaleAssociation)
    }
    /// Whether the cause is: prior policy does not match the document's current policy.
    pub fn is_prior_policy_mismatch(&self) -> bool {
        matches!(self.kind, HtmlCspProtocolErrorKind::PriorPolicyMismatch)
    }
    /// Whether the cause is: native policy artifact has the wrong type.
    pub fn is_wrong_native_artifact_type(&self) -> bool {
        matches!(self.kind, HtmlCspProtocolErrorKind::WrongNativeArtifactType)
    }
    /// Whether the cause is: meta CSP content is empty.
    pub fn is_empty_content(&self) -> bool {
        matches!(self.kind, HtmlCspProtocolErrorKind::EmptyContent)
    }
    /// Whether the cause is: CSP accounting size cannot be represented on this platform.
    pub fn is_representation_exhausted(&self) -> bool {
        matches!(self.kind, HtmlCspProtocolErrorKind::RepresentationExhausted)
    }
    /// Whether the cause is: CSP policy storage is invalid.
    pub fn is_invalid_storage(&self) -> bool {
        matches!(self.kind, HtmlCspProtocolErrorKind::InvalidStorage)
    }
}

impl std::fmt::Display for HtmlCspProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlCspProtocolErrorKind::ForeignOwner => {
                f.write_str("policy state belongs to a different CSP owner")
            }
            HtmlCspProtocolErrorKind::ForeignDocument => {
                f.write_str("association belongs to a different document")
            }
            HtmlCspProtocolErrorKind::WrongProcessor => {
                f.write_str("processor does not match the request")
            }
            HtmlCspProtocolErrorKind::WrongProfile => {
                f.write_str("processor profile does not match the request")
            }
            HtmlCspProtocolErrorKind::WrongRequest => {
                f.write_str("request does not match the pending meta element state")
            }
            HtmlCspProtocolErrorKind::StaleAssociation => f.write_str("association is stale"),
            HtmlCspProtocolErrorKind::PriorPolicyMismatch => {
                f.write_str("prior policy does not match the document's current policy")
            }
            HtmlCspProtocolErrorKind::WrongNativeArtifactType => {
                f.write_str("native policy artifact has the wrong type")
            }
            HtmlCspProtocolErrorKind::EmptyContent => f.write_str("meta CSP content is empty"),
            HtmlCspProtocolErrorKind::RepresentationExhausted => {
                f.write_str("CSP accounting size cannot be represented on this platform")
            }
            HtmlCspProtocolErrorKind::InvalidStorage => {
                f.write_str("CSP policy storage is invalid")
            }
        }
    }
}

impl std::error::Error for HtmlCspProtocolError {}

/// Original-controller interruption remains distinct from protocol misuse.
#[derive(Debug)]
pub enum HtmlCspError<E> {
    Protocol(HtmlCspProtocolError),
    Interrupted(E),
}
impl<E> fmt::Display for HtmlCspError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Protocol(_) => "CSP protocol was violated",
            Self::Interrupted(_) => "CSP processing was interrupted by the caller's admission",
        })
    }
}
impl<E: Error + 'static> Error for HtmlCspError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Protocol(e) => Some(e),
            Self::Interrupted(e) => Some(e),
        }
    }
}
impl<E> From<HtmlCspProtocolErrorKind> for HtmlCspError<E> {
    fn from(kind: HtmlCspProtocolErrorKind) -> Self {
        Self::Protocol(kind.into())
    }
}
impl<E> From<HtmlCspProtocolError> for HtmlCspError<E> {
    fn from(error: HtmlCspProtocolError) -> Self {
        Self::Protocol(error)
    }
}

/// Actual owned inputs returned without allocation on refusal.
#[derive(Debug)]
pub struct HtmlCspRejected<T, E> {
    pub retained: T,
    pub error: HtmlCspError<E>,
}

impl<T, E: fmt::Display> fmt::Display for HtmlCspRejected<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CSP request rejected and its input retained: {}",
            self.error
        )
    }
}
impl<T: fmt::Debug, E: Error + 'static> Error for HtmlCspRejected<T, E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

/// Required external capability; absence never certifies empty policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlCspUnavailable {
    Processor,
    Profile,
    InitialEmptyProcessing,
    NativeOwner,
}

/// Trusted immutable backend artifact. The fixed descriptor must be infallible,
/// nonpanicking and allocation-free. Native destruction, including transitive
/// last-owner release, must fit its cached bound without allocation, panic,
/// blocking, callbacks, policy processing or enforcement, even during unwind.
/// The provider prepays partial construction before creating native custody.
pub trait HtmlCspPolicyArtifact: Any + fmt::Debug + Send + Sync {
    fn retained_layout(&self) -> HtmlCspArtifactLayout;
}
trait ErasedArtifact: fmt::Debug + Send + Sync {
    fn native(&self) -> &dyn Any;
}
impl<A: HtmlCspPolicyArtifact> ErasedArtifact for A {
    fn native(&self) -> &dyn Any {
        self
    }
}

/// A qualified borrow into the actual retained native allocation.
#[derive(Debug)]
pub enum HtmlCspNativePolicy<'a, A: HtmlCspPolicyArtifact> {
    InitialEmpty,
    External(&'a A),
}

#[derive(Debug)]
struct PolicyRecord {
    id: HtmlCspPolicyId,
    revision: HtmlCspPolicyRevision,
    profile: Option<HtmlCspProcessorProfile>,
    native: Option<Arc<dyn ErasedArtifact>>,
    origin: Option<HtmlMetaCspQualification>,
    layout: HtmlCspArtifactLayout,
}

/// Immutable policy artifact custody, not a processor or live Document owner.
#[derive(Clone, Debug)]
pub struct HtmlCspPolicyState(Arc<PolicyRecord>);
impl HtmlCspPolicyState {
    pub fn id(&self) -> &HtmlCspPolicyId {
        &self.0.id
    }
    pub fn revision(&self) -> HtmlCspPolicyRevision {
        self.0.revision
    }
    pub fn profile(&self) -> Option<&HtmlCspProcessorProfile> {
        self.0.profile.as_ref()
    }
    pub fn processing_origin(&self) -> Option<&HtmlMetaCspQualification> {
        self.0.origin.as_ref()
    }
    pub fn retained_layout(&self) -> HtmlCspArtifactLayout {
        self.0.layout
    }
    /// Read-only recovery, with profile qualification before native type testing.
    /// The admission callback supplies only controller admission/cancellation.
    /// A successful borrow establishes neither service availability nor currentness.
    pub fn borrow_native_with<'a, A: HtmlCspPolicyArtifact, E>(
        &'a self,
        processor: &HtmlCspProcessorProfile,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<HtmlCspNativePolicy<'a, A>, HtmlCspError<E>> {
        admit(HtmlCspLayout::read::<Self>(6)).map_err(HtmlCspError::Interrupted)?;
        let Some(profile) = &self.0.profile else {
            return Ok(HtmlCspNativePolicy::InitialEmpty);
        };
        profile.require(processor)?;
        let native = self
            .0
            .native
            .as_ref()
            .and_then(|native| native.native().downcast_ref::<A>())
            .ok_or(HtmlCspProtocolErrorKind::WrongNativeArtifactType)?;
        Ok(HtmlCspNativePolicy::External(native))
    }
}
impl PartialEq for HtmlCspPolicyState {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlCspPolicyState {}

/// External actual processing, not a successful capability flag. Implementors
/// admit all native reads/work/allocation before doing them and preserve typed
/// failures. The explicitly supplied service is never retained by policy values.
///
/// Each call declares its actual native custody as an aggregate: retained
/// allocations include all backing and prepaid disposal, including a returned
/// owned `Error`. A processing action ends before the next admission or release
/// callback; temporary peak-only backing must have dropped by that boundary.
/// `release` follows actual disposal, never merely a decision to discard.
/// Completed custody transfers to the prepared result; remaining admitted
/// custody on Refused/Failed transfers with the original owned error. Other
/// outcomes and interrupted/protocol errors must first dispose and release all
/// call-specific native temporaries. Persistent provider caches require their
/// own external ownership and must not be refunded while still live.
/// Returned owned errors obey the artifact disposal contract too: destruction
/// is bounded, allocation-free, nonpanicking, nonblocking, and performs no
/// callbacks. Its complete destruction work is prepaid before formation, so
/// the original consumer may safely destroy it after terminal interruption.
pub trait HtmlMetaCspProcessor {
    type Error;
    fn profile(&self) -> &HtmlCspProcessorProfile;
    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<Self::Error>, HtmlCspError<E>>;
}

#[derive(Debug)]
pub enum HtmlMetaCspOutcome<E> {
    Completed(HtmlMetaCspPrepared),
    Unavailable(HtmlCspUnavailable),
    Refused(E),
    Cancelled,
    Failed(E),
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod cause_tests {
    use super::*;

    /// Every cause states its documented meaning, and exactly one of the
    /// cause predicates, its own, recognizes it.
    #[test]
    fn every_cause_has_its_message_and_exactly_one_matching_predicate() {
        type Predicate = fn(&HtmlCspProtocolError) -> bool;
        let cases: Vec<(HtmlCspProtocolError, &str, Predicate)> = vec![
            (
                HtmlCspProtocolErrorKind::ForeignOwner.error(),
                "policy state belongs to a different CSP owner",
                HtmlCspProtocolError::is_foreign_owner,
            ),
            (
                HtmlCspProtocolErrorKind::ForeignDocument.error(),
                "association belongs to a different document",
                HtmlCspProtocolError::is_foreign_document,
            ),
            (
                HtmlCspProtocolErrorKind::WrongProcessor.error(),
                "processor does not match the request",
                HtmlCspProtocolError::is_wrong_processor,
            ),
            (
                HtmlCspProtocolErrorKind::WrongProfile.error(),
                "processor profile does not match the request",
                HtmlCspProtocolError::is_wrong_profile,
            ),
            (
                HtmlCspProtocolErrorKind::WrongRequest.error(),
                "request does not match the pending meta element state",
                HtmlCspProtocolError::is_wrong_request,
            ),
            (
                HtmlCspProtocolErrorKind::StaleAssociation.error(),
                "association is stale",
                HtmlCspProtocolError::is_stale_association,
            ),
            (
                HtmlCspProtocolErrorKind::PriorPolicyMismatch.error(),
                "prior policy does not match the document's current policy",
                HtmlCspProtocolError::is_prior_policy_mismatch,
            ),
            (
                HtmlCspProtocolErrorKind::WrongNativeArtifactType.error(),
                "native policy artifact has the wrong type",
                HtmlCspProtocolError::is_wrong_native_artifact_type,
            ),
            (
                HtmlCspProtocolErrorKind::EmptyContent.error(),
                "meta CSP content is empty",
                HtmlCspProtocolError::is_empty_content,
            ),
            (
                HtmlCspProtocolErrorKind::RepresentationExhausted.error(),
                "CSP accounting size cannot be represented on this platform",
                HtmlCspProtocolError::is_representation_exhausted,
            ),
            (
                HtmlCspProtocolErrorKind::InvalidStorage.error(),
                "CSP policy storage is invalid",
                HtmlCspProtocolError::is_invalid_storage,
            ),
        ];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }
}
