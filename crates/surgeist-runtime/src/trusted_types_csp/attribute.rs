//! Trusted Types CSP decisions on the parked native attribute operation.

use super::{Decision, Policy, PolicyProcessor, Violation, requires_script_values};
use std::{error::Error, fmt, sync::atomic::AtomicBool};
use surgeist_dom::{
    DocumentId, DomDocumentCspState, DomDocumentScriptRegistration, DomError,
    DomSourceAttributePolicyRead, DomSourceAttributeVerificationRequest, DomSourceLease,
};
use surgeist_html::security::csp::{HtmlCspError, HtmlCspProtocolError};

/// Why the registered Global's current CSP could not be used.
#[derive(Debug)]
pub enum AttributePolicyError {
    /// The native lease, registration, or original Source controller refused.
    Dom(DomError),
    /// The policy container is unavailable.
    ContainerUnavailable,
    /// The associated policy requires a configured processor.
    ProcessorUnavailable,
    /// The selected processor could not qualify the associated artifact.
    Csp(HtmlCspProtocolError),
}

impl fmt::Display for AttributePolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dom(error) => error.fmt(f),
            Self::ContainerUnavailable => {
                f.write_str("the registered Global's CSP container is unavailable")
            }
            Self::ProcessorUnavailable => {
                f.write_str("the associated CSP processor is unavailable")
            }
            Self::Csp(error) => error.fmt(f),
        }
    }
}

impl Error for AttributePolicyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            Self::Csp(error) => Some(error),
            Self::ContainerUnavailable | Self::ProcessorUnavailable => None,
        }
    }
}

impl From<DomError> for AttributePolicyError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}

impl From<HtmlCspError<DomError>> for AttributePolicyError {
    fn from(error: HtmlCspError<DomError>) -> Self {
        match error {
            HtmlCspError::Protocol(error) => Self::Csp(error),
            HtmlCspError::Interrupted(error) => Self::Dom(error),
        }
    }
}

/// One qualified read of the original attribute verifier's live Global.
/// The lease borrow must end before a guest callback or reentrant mutation.
pub struct AttributePolicies<'a> {
    read: DomSourceAttributePolicyRead<'a>,
    policies: &'a [Policy],
}

impl<'a> AttributePolicies<'a> {
    /// Resolve the actual CSP list for the exact pending attribute request.
    pub fn read(
        lease: &'a DomSourceLease<'_>,
        request: DomSourceAttributeVerificationRequest,
        registration: &DomDocumentScriptRegistration,
        processor: Option<&PolicyProcessor>,
        cancel: &AtomicBool,
    ) -> Result<Self, AttributePolicyError> {
        let read = lease.attribute_verification_policy(request, registration, cancel)?;
        let policies = match read.state() {
            DomDocumentCspState::ConstructorInitialEmpty => &[][..],
            DomDocumentCspState::Unavailable => {
                return Err(AttributePolicyError::ContainerUnavailable);
            }
            state @ DomDocumentCspState::Associated(association) => {
                read.step(cancel)?;
                if association.policy().profile().is_none() {
                    &[]
                } else {
                    processor
                        .ok_or(AttributePolicyError::ProcessorUnavailable)?
                        .document_policies_with(state, &mut |layout| {
                            read.admit_csp_read(layout, cancel)
                        })?
                        .ok_or(AttributePolicyError::ContainerUnavailable)?
                }
            }
        };
        Ok(Self { read, policies })
    }

    /// The Document associated with the original Global at this read.
    pub const fn document(&self) -> DocumentId {
        self.read.document()
    }

    /// Whether a mismatched input should be offered to the default policy.
    /// Report-only policies participate in this decision.
    pub fn requires_default_policy(&self, cancel: &AtomicBool) -> Result<bool, DomError> {
        self.read.step(cancel)?;
        requires_script_values(self.policies, true, &mut || self.read.step(cancel))
    }

    /// Begins the mismatch decision on a read taken after guest execution.
    /// Every reached violation is yielded in order before the decision completes.
    pub fn mismatch(&self) -> AttributeMismatch<'_, 'a> {
        AttributeMismatch {
            read: &self.read,
            decision: Decision::script_mismatch(self.policies),
        }
    }
}

/// The original Source operation's ordered mismatch decision.
pub struct AttributeMismatch<'read, 'policies> {
    read: &'read DomSourceAttributePolicyRead<'policies>,
    decision: Decision<'policies, 'static>,
}

impl<'policies> AttributeMismatch<'_, 'policies> {
    /// Returns the next violation candidate under the original Source controller.
    pub fn next_violation(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<Option<Violation<'policies>>, DomError> {
        self.read.step(cancel)?;
        self.decision.next_violation(&mut || self.read.step(cancel))
    }

    /// The final enforcement decision, available after the policy scan.
    pub fn blocked(&self) -> Option<bool> {
        self.decision.blocked()
    }
}
