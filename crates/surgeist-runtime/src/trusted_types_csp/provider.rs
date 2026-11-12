//! Native policy-list custody for response delivery and qualified meta insertions.

use super::{Policy, PolicyDelivery};
use std::{error::Error, fmt, mem::size_of, sync::Arc};
use surgeist_dom::DomDocumentCspState;
use surgeist_html::{
    security::csp::{
        HtmlCspArtifactLayout, HtmlCspAssociation, HtmlCspDisposalWork, HtmlCspError,
        HtmlCspLayout, HtmlCspNativePolicy, HtmlCspOwner, HtmlCspPolicyArtifact,
        HtmlCspProcessorAuthority, HtmlCspProcessorProfile, HtmlCspProtocolError,
        HtmlCspSerializedPolicy, HtmlCspStorage, HtmlMetaCspOutcome, HtmlMetaCspProcessor,
        HtmlMetaCspRequest,
    },
    values::DocumentString,
};

/// Failure to represent or reserve the policy list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyListCapacity {
    /// The required graph extent cannot be represented.
    Representation,
    /// The allocator could not reserve the backing.
    Allocation,
}

impl fmt::Display for PolicyListCapacity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Representation => "CSP policy-list extent is not representable",
            Self::Allocation => "CSP policy-list backing could not be reserved",
        })
    }
}

impl Error for PolicyListCapacity {}

/// Why response policy delivery could not produce a document association.
#[derive(Debug)]
pub enum PolicyListError<E> {
    /// The CSP protocol or the original admission controller refused.
    Csp(HtmlCspError<E>),
    /// The native policy list could not be represented or reserved.
    Capacity(PolicyListCapacity),
}

impl<E> From<HtmlCspError<E>> for PolicyListError<E> {
    fn from(error: HtmlCspError<E>) -> Self {
        Self::Csp(error)
    }
}

impl<E> From<HtmlCspProtocolError> for PolicyListError<E> {
    fn from(error: HtmlCspProtocolError) -> Self {
        Self::Csp(HtmlCspError::Protocol(error))
    }
}

impl<E> From<PolicyListCapacity> for PolicyListError<E> {
    fn from(error: PolicyListCapacity) -> Self {
        Self::Capacity(error)
    }
}

impl<E> fmt::Display for PolicyListError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Csp(error) => error.fmt(f),
            Self::Capacity(error) => error.fmt(f),
        }
    }
}

impl<E: Error + 'static> Error for PolicyListError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Csp(error) => Some(error),
            Self::Capacity(error) => Some(error),
        }
    }
}

/// The issuer and reader of an admitted Trusted Types CSP list.
pub struct PolicyProcessor {
    authority: HtmlCspProcessorAuthority,
    profile: HtmlCspProcessorProfile,
}

impl PolicyProcessor {
    /// Admits the processor and its supported semantic profile before retaining them.
    pub fn new_with<E>(
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<Self, HtmlCspError<E>> {
        let mut storage = HtmlCspStorage::default();
        let authority = HtmlCspProcessorAuthority::new_with(&mut |layout| {
            admit(layout)?;
            storage = layout.retained().storage();
            Ok(())
        })?;
        match authority.profile_with(1, admit) {
            Ok(profile) => Ok(Self { authority, profile }),
            Err(error) => {
                drop(authority);
                release(storage);
                Err(error)
            }
        }
    }

    /// Establishes a new association for a response whose header-list members
    /// have already been extracted and isomorphically decoded by the delivery
    /// adapter. Each header group is supplied in its original order.
    pub fn response_with<E>(
        &mut self,
        owner: &mut HtmlCspOwner,
        enforcing: &[DocumentString],
        report_only: &[DocumentString],
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlCspAssociation, PolicyListError<E>> {
        let count = enforcing
            .len()
            .checked_add(report_only.len())
            .ok_or(PolicyListCapacity::Representation)?;
        let entries = enforcing
            .iter()
            .map(|text| Entry::Delivered(text, PolicyDelivery::EnforcingHeader))
            .chain(
                report_only
                    .iter()
                    .map(|text| Entry::Delivered(text, PolicyDelivery::ReportOnlyHeader)),
            );
        let artifact = build_list(count, entries, admit, release)?;
        let storage = artifact.retained_layout().storage();
        match self
            .authority
            .attest_response_policy_with(owner, &self.profile, artifact, admit)
        {
            Ok(association) => Ok(association),
            Err(rejected) => {
                drop(rejected.retained);
                release(storage);
                Err(rejected.error.into())
            }
        }
    }

    /// Qualifies a read of the actual current Document CSP container.
    /// An unavailable container returns no list. A foreign profile or artifact
    /// type is a protocol error, even when the opaque policy might be empty.
    pub fn document_policies_with<'a, E>(
        &self,
        state: DomDocumentCspState<'a>,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<Option<&'a [Policy]>, HtmlCspError<E>> {
        tick(admit)?;
        match state {
            DomDocumentCspState::ConstructorInitialEmpty => Ok(Some(&[])),
            DomDocumentCspState::Unavailable => Ok(None),
            DomDocumentCspState::Associated(association) => {
                let native = association
                    .policy()
                    .borrow_native_with::<PolicyList, _>(&self.profile, admit)?;
                Ok(Some(native_policies(native)))
            }
        }
    }
}

impl HtmlMetaCspProcessor for PolicyProcessor {
    type Error = PolicyListCapacity;

    fn profile(&self) -> &HtmlCspProcessorProfile {
        &self.profile
    }

    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        release: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<Self::Error>, HtmlCspError<E>> {
        request
            .qualification()
            .processor()
            .validate_selected_with(&self.profile, admit)?;
        let native = request
            .prior_policy()
            .borrow_native_with::<PolicyList, _>(&self.profile, admit)?;
        let prior = native_policies(native);
        let Some(count) = prior.len().checked_add(1) else {
            return Ok(HtmlMetaCspOutcome::Failed(
                PolicyListCapacity::Representation,
            ));
        };
        let entries = prior
            .iter()
            .map(Entry::Prior)
            .chain(std::iter::once(Entry::Delivered(
                request.qualification().content(),
                PolicyDelivery::Meta,
            )));
        let artifact = match build_list(count, entries, admit, release) {
            Ok(artifact) => artifact,
            Err(PolicyListError::Capacity(error)) => return Ok(HtmlMetaCspOutcome::Failed(error)),
            Err(PolicyListError::Csp(error)) => return Err(error),
        };
        let storage = artifact.retained_layout().storage();
        match self.authority.attest_changed_with(request, artifact, admit) {
            Ok(prepared) => Ok(HtmlMetaCspOutcome::Completed(prepared)),
            Err(rejected) => {
                drop(rejected.retained);
                release(storage);
                Err(rejected.error)
            }
        }
    }
}

fn native_policies(native: HtmlCspNativePolicy<'_, PolicyList>) -> &[Policy] {
    match native {
        HtmlCspNativePolicy::InitialEmpty => &[],
        HtmlCspNativePolicy::External(list) => &list.policies,
    }
}

#[derive(Debug, Default)]
struct PolicyList {
    policies: Vec<Policy>,
    layout: HtmlCspArtifactLayout,
}
impl HtmlCspPolicyArtifact for PolicyList {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        self.layout
    }
}

enum Entry<'a> {
    Prior(&'a Policy),
    Delivered(&'a DocumentString, PolicyDelivery),
}

// Artifact/Vec destruction plus dropping each inline policy and its retained string block.
const LIST_CLOSE: u64 = 3;
const POLICY_CLOSE: u64 = 5;
const POLICY_FRAME: usize = size_of::<Policy>() + size_of::<Vec<u16>>();
const LIST_FRAME: usize = size_of::<PolicyList>() + POLICY_FRAME;

fn tick<E>(admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>) -> Result<(), HtmlCspError<E>> {
    admit(HtmlCspLayout::processing(
        1,
        HtmlCspStorage::default(),
        HtmlCspDisposalWork::default(),
        0,
    )?)
    .map_err(HtmlCspError::Interrupted)
}

fn policy_bytes(count: usize) -> Result<usize, PolicyListCapacity> {
    count
        .checked_mul(size_of::<Policy>())
        .ok_or(PolicyListCapacity::Representation)
}

fn admit_storage<E>(
    storage: HtmlCspStorage,
    disposal: HtmlCspDisposalWork,
    admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
) -> Result<(), HtmlCspError<E>> {
    admit(HtmlCspLayout::processing(
        1,
        storage,
        disposal,
        disposal.atomic_bytes(),
    )?)
    .map_err(HtmlCspError::Interrupted)
}

fn build_list<'a, E>(
    count: usize,
    entries: impl Iterator<Item = Entry<'a>>,
    admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    release: &mut impl FnMut(HtmlCspStorage),
) -> Result<Arc<PolicyList>, PolicyListError<E>> {
    let mut list = PolicyList::default();
    let result = (|| {
        let bytes = policy_bytes(count)?
            .checked_add(size_of::<PolicyList>())
            .ok_or(PolicyListCapacity::Representation)?;
        let storage = HtmlCspStorage::try_new(bytes, 0, 1)?;
        let disposal = HtmlCspDisposalWork::try_new(LIST_CLOSE, LIST_FRAME)?;
        let layout = HtmlCspArtifactLayout::try_new(storage, disposal)?;
        admit_storage(storage, disposal, admit)?;
        list.layout = layout;
        list.policies
            .try_reserve_exact(count)
            .map_err(|_| PolicyListCapacity::Allocation)?;
        let actual = list.policies.capacity();
        if actual > count {
            let extra = policy_bytes(actual - count)?;
            let backing = HtmlCspStorage::try_new(extra, 0, 0)?;
            let bytes = bytes
                .checked_add(extra)
                .ok_or(PolicyListCapacity::Representation)?;
            let layout =
                HtmlCspArtifactLayout::try_new(HtmlCspStorage::try_new(bytes, 0, 1)?, disposal)?;
            admit_storage(backing, HtmlCspDisposalWork::default(), admit)?;
            list.layout = layout;
        }
        for entry in entries {
            tick(admit)?;
            let policy = match entry {
                Entry::Prior(policy) => {
                    list.admit_text(&policy.serialized, admit)?;
                    policy.clone()
                }
                Entry::Delivered(text, delivery) => {
                    if delivery != PolicyDelivery::Meta
                        && HtmlCspSerializedPolicy::new(text.code_units(), delivery.source())
                            .directives()
                            .next_with(&mut || tick(admit))?
                            .is_none()
                    {
                        continue;
                    }
                    list.admit_text(text, admit)?;
                    Policy::parse(text.clone(), delivery, &mut || tick(admit))?
                }
            };
            list.policies.push(policy);
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(Arc::new(list)),
        Err(error) => {
            let storage = list.layout.storage();
            drop(list);
            if storage != HtmlCspStorage::default() {
                release(storage);
            }
            Err(error)
        }
    }
}

impl PolicyList {
    fn admit_text<E>(
        &mut self,
        text: &DocumentString,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<(), PolicyListError<E>> {
        let units = text.retained_units();
        let bytes = units
            .checked_mul(size_of::<u16>())
            .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
            .ok_or(PolicyListCapacity::Representation)?;
        let old = self.layout.storage();
        let next = HtmlCspStorage::try_new(
            old.bytes()
                .checked_add(bytes)
                .ok_or(PolicyListCapacity::Representation)?,
            old.utf16_units()
                .checked_add(units)
                .ok_or(PolicyListCapacity::Representation)?,
            old.entries()
                .checked_add(1)
                .ok_or(PolicyListCapacity::Representation)?,
        )?;
        let count = u64::try_from(old.entries()).map_err(|_| PolicyListCapacity::Representation)?;
        let close = POLICY_CLOSE
            .checked_mul(count)
            .and_then(|n| n.checked_add(LIST_CLOSE))
            .ok_or(PolicyListCapacity::Representation)?;
        let layout =
            HtmlCspArtifactLayout::try_new(next, HtmlCspDisposalWork::try_new(close, LIST_FRAME)?)?;
        admit_storage(
            HtmlCspStorage::try_new(bytes, units, 1)?,
            HtmlCspDisposalWork::try_new(POLICY_CLOSE, POLICY_FRAME)?,
            admit,
        )?;
        self.layout = layout;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
