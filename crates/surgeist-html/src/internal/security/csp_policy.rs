//! Neutral qualified Meta CSP transitions over caller-owned native artifacts.
//! No CSP grammar, live policy registry, tree guard, or execution service lives here.
use crate::internal::values::DocumentString;
use std::{any::Any, error::Error, fmt, mem::size_of, sync::Arc};

pub(crate) mod accounting;
pub(crate) mod identity;
pub(crate) mod transition;
use accounting::*;
use identity::*;
use transition::*;

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCspError<E> {
    /// The request violated the CSP protocol: an input was foreign, stale or mismatched, the
    /// content was empty, the native artifact had the wrong type, the policy storage was invalid,
    /// or checked accounting could not be represented. The wrapped error names the cause.
    Protocol(HtmlCspProtocolError),
    /// The caller's own admission callback refused the next admission, or the
    /// caller's controller stopped the call. `E` is that callback's error, returned
    /// unchanged and never wrapped in a protocol cause.
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
    /// The caller's owned input, handed back unconsumed. Refusal performs no
    /// allocation, so the caller can retry or dispose of it.
    pub retained: T,
    /// Why the operation refused: a protocol violation or the caller's own
    /// interruption.
    pub error: HtmlCspError<E>,
}

impl<T, E> fmt::Display for HtmlCspRejected<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CSP request rejected and its input retained")
    }
}
impl<T: fmt::Debug, E: Error + 'static> Error for HtmlCspRejected<T, E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

/// Required external capability; absence never certifies empty policy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum HtmlCspUnavailable {
    /// The call required an external processor and none was supplied. Absence is not
    /// evidence that the policy is empty.
    Processor,
    /// The call required a processor profile and none was available. Absence is not
    /// evidence that the policy is empty.
    Profile,
    /// The capability to process a document's initial empty policy was unavailable,
    /// so that policy cannot be certified as processed.
    InitialEmptyProcessing,
    /// The backend's native owner of policy artifacts was unavailable, for example a
    /// closed service, so no native policy could be produced.
    NativeOwner,
}

/// Trusted immutable backend artifact. The fixed descriptor must be infallible,
/// nonpanicking and allocation-free. Native destruction, including transitive
/// last-owner release, must fit its cached bound without allocation, panic,
/// blocking, callbacks, policy processing or enforcement, even during unwind.
/// The provider prepays partial construction before creating native custody.
pub trait HtmlCspPolicyArtifact: Any + fmt::Debug + Send + Sync {
    /// Returns this artifact's native storage and disposal bounds. The value must be
    /// fixed for the artifact's lifetime, must not allocate or panic, and is read once
    /// and then cached by the policy state that retains the artifact.
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
    /// The document has its initial empty policy and no native artifact exists.
    InitialEmpty,
    /// The retained native artifact, borrowed from the policy state's allocation.
    /// Its type was checked against the requested artifact type.
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
    /// Returns the identity of this immutable policy state. Equal identities denote
    /// the same state issued by the same owner or processor authority.
    pub fn id(&self) -> &HtmlCspPolicyId {
        &self.0.id
    }
    /// Returns the issuer-scoped revision of this policy state. Revisions are opaque
    /// and only comparable for equality.
    pub fn revision(&self) -> HtmlCspPolicyRevision {
        self.0.revision
    }
    /// Returns the profile of the processor that produced this state, or `None` for
    /// an initial empty policy that no processor has touched.
    pub fn profile(&self) -> Option<&HtmlCspProcessorProfile> {
        self.0.profile.as_ref()
    }
    /// Returns the qualification of the meta insertion that produced this state.
    /// Initial empty and response policy states have no meta qualification.
    pub fn processing_origin(&self) -> Option<&HtmlMetaCspQualification> {
        self.0.origin.as_ref()
    }
    /// Returns the cached native storage and disposal bounds that this state
    /// retains, including its wrapper and the artifact's own layout.
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
    /// The processor's own typed error, carried by the `Refused` and `Failed`
    /// outcomes.
    type Error;
    /// Returns the profile this processor declares. A request's qualification must
    /// carry an equal profile (same issuer and semantic version) to be processed.
    fn profile(&self) -> &HtmlCspProcessorProfile;
    /// Processes one qualified meta insertion request against its retained prior
    /// policy. `admit` receives every native allocation, read and work step before it
    /// happens, and `release` returns custody after actual disposal. The result
    /// states whether processing completed, was unavailable, was refused or failed
    /// with the processor's error, or was cancelled.
    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<Self::Error>, HtmlCspError<E>>;
}

/// How an external processor ended one meta request. Only `Completed` yields a
/// policy; the other outcomes leave the prior policy in force.
#[derive(Debug)]
pub enum HtmlMetaCspOutcome<E> {
    /// Processing finished. The prepared value carries the qualification and the
    /// resulting policy, and owns the completed native custody.
    Completed(HtmlMetaCspPrepared),
    /// A required capability was missing, so nothing was processed. This is distinct
    /// from an empty policy.
    Unavailable(HtmlCspUnavailable),
    /// The processor declined the request with its own typed error. Remaining
    /// admitted custody transfers with the original owned error.
    Refused(E),
    /// The processor stopped without a result because the controller cancelled it.
    /// Admitted native temporaries were disposed and released first.
    Cancelled,
    /// Processing failed with the processor's own typed error. Remaining admitted
    /// custody transfers with the original owned error.
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
