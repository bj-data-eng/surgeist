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
#[non_exhaustive]
pub enum HtmlCspProtocolError {
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
impl fmt::Display for HtmlCspProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTML CSP protocol: {self:?}")
    }
}
impl Error for HtmlCspProtocolError {}

/// Original-controller interruption remains distinct from protocol misuse.
#[derive(Debug)]
pub enum HtmlCspError<E> {
    Protocol(HtmlCspProtocolError),
    Interrupted(E),
}
impl<E: fmt::Display> fmt::Display for HtmlCspError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => e.fmt(f),
            Self::Interrupted(e) => write!(f, "HTML CSP interrupted: {e}"),
        }
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
            .ok_or(HtmlCspProtocolError::WrongNativeArtifactType)?;
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
