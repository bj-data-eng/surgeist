use super::*;

/// One consumer session's non-Clone issuer. It has no map of current policy.
/// Actual adapters bind its tokens to their canonical/construction Documents.
#[derive(Debug)]
pub struct HtmlCspOwner {
    identity: HtmlCspOwnerId,
    document_serial: u64,
    occurrence_serial: u64,
}

/// Why a distinct Document association shares an existing immutable artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCspAssociationCause {
    Installation,
    Inheritance,
}

/// Bounded transfer provenance, without a chain of former associations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCspAssociationOrigin {
    original: HtmlCspDocumentId,
    from: Option<(
        HtmlCspDocumentId,
        HtmlCspAssociationGeneration,
        HtmlCspAssociationCause,
    )>,
}
impl HtmlCspAssociationOrigin {
    pub fn original_document(&self) -> &HtmlCspDocumentId {
        &self.original
    }
    pub fn transferred_from(
        &self,
    ) -> Option<(
        &HtmlCspDocumentId,
        &HtmlCspAssociationGeneration,
        HtmlCspAssociationCause,
    )> {
        self.from
            .as_ref()
            .map(|(document, generation, cause)| (document, generation, *cause))
    }
}

/// A read-only snapshot or private transactional value. A Clone is not canonical
/// authority: the real adapter selects its actual current State/Draft/session.
#[derive(Clone, Debug)]
pub struct HtmlCspAssociation {
    document: HtmlCspDocumentId,
    generation: HtmlCspAssociationGeneration,
    policy: HtmlCspPolicyState,
    latest: Option<HtmlMetaCspQualification>,
    origin: HtmlCspAssociationOrigin,
}
impl HtmlCspAssociation {
    pub fn document(&self) -> &HtmlCspDocumentId {
        &self.document
    }
    pub fn generation(&self) -> &HtmlCspAssociationGeneration {
        &self.generation
    }
    pub fn policy(&self) -> &HtmlCspPolicyState {
        &self.policy
    }
    pub fn latest(&self) -> Option<&HtmlMetaCspQualification> {
        self.latest.as_ref()
    }
    pub fn origin(&self) -> &HtmlCspAssociationOrigin {
        &self.origin
    }
    pub fn retained_layout(&self) -> Result<HtmlCspArtifactLayout, HtmlCspProtocolError> {
        association_layout(&self.policy, self.latest.as_ref())
    }
}
fn association_layout(
    policy: &HtmlCspPolicyState,
    latest: Option<&HtmlMetaCspQualification>,
) -> Result<HtmlCspArtifactLayout, HtmlCspProtocolError> {
    let mut layout =
        HtmlCspArtifactLayout::inline::<HtmlCspAssociation>(12).add(policy.retained_layout())?;
    if let Some(latest) = latest {
        layout = layout.add(latest.text_layout()?)?;
    }
    Ok(layout)
}

/// Fixed HTML Meta processing vocabulary; callers cannot select other modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaCspSource {
    Meta,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaCspDisposition {
    Enforce,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaCspRemovedDirective {
    ReportUri,
    FrameAncestors,
    Sandbox,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMetaCspTrigger {
    Insertion,
}

/// Exact input and issuer-bound source occurrence. No prior artifact is stored
/// here, so retaining latest qualification does not retain a policy-history chain.
#[derive(Clone, Debug)]
pub struct HtmlMetaCspQualification(Arc<QualificationRecord>);
#[derive(Debug)]
struct QualificationRecord {
    document: HtmlCspDocumentId,
    generation: HtmlCspAssociationGeneration,
    prior_policy: HtmlCspPolicyId,
    prior_revision: HtmlCspPolicyRevision,
    processor: HtmlCspProcessorProfile,
    occurrence: HtmlMetaCspOccurrence,
    content: DocumentString,
}
impl HtmlMetaCspQualification {
    pub fn document(&self) -> &HtmlCspDocumentId {
        &self.0.document
    }
    pub fn generation(&self) -> &HtmlCspAssociationGeneration {
        &self.0.generation
    }
    pub fn prior_policy(&self) -> &HtmlCspPolicyId {
        &self.0.prior_policy
    }
    pub fn prior_revision(&self) -> HtmlCspPolicyRevision {
        self.0.prior_revision
    }
    pub fn processor(&self) -> &HtmlCspProcessorProfile {
        &self.0.processor
    }
    pub fn occurrence(&self) -> &HtmlMetaCspOccurrence {
        &self.0.occurrence
    }
    pub fn content(&self) -> &DocumentString {
        &self.0.content
    }
    pub fn source(&self) -> HtmlMetaCspSource {
        HtmlMetaCspSource::Meta
    }
    pub fn disposition(&self) -> HtmlMetaCspDisposition {
        HtmlMetaCspDisposition::Enforce
    }
    pub fn trigger(&self) -> HtmlMetaCspTrigger {
        HtmlMetaCspTrigger::Insertion
    }
    pub fn removed_directives(&self) -> [HtmlMetaCspRemovedDirective; 3] {
        [
            HtmlMetaCspRemovedDirective::ReportUri,
            HtmlMetaCspRemovedDirective::FrameAncestors,
            HtmlMetaCspRemovedDirective::Sandbox,
        ]
    }
    fn text_layout(&self) -> Result<HtmlCspArtifactLayout, HtmlCspProtocolError> {
        HtmlCspArtifactLayout::inline::<QualificationRecord>(12)
            .add(HtmlCspArtifactLayout::text(&self.0.content)?)
    }
    fn matches(&self, other: &Self) -> bool {
        // The unforgeable issued occurrence binds the exact original content;
        // do not perform an unbounded text comparison during publication.
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// One affine reached insertion, retaining the actual prior native artifact.
#[derive(Debug)]
#[must_use = "retain the request for processing or discard its admitted custody"]
pub struct HtmlMetaCspRequest {
    qualification: HtmlMetaCspQualification,
    prior: HtmlCspPolicyState,
}
impl HtmlMetaCspRequest {
    pub fn qualification(&self) -> &HtmlMetaCspQualification {
        &self.qualification
    }
    pub fn prior_policy(&self) -> &HtmlCspPolicyState {
        &self.prior
    }
    pub fn retained_layout(&self) -> Result<HtmlCspArtifactLayout, HtmlCspProtocolError> {
        HtmlCspArtifactLayout::inline::<Self>(10)
            .add(self.prior.retained_layout())?
            .add(self.qualification.text_layout()?)
    }
}

/// A trusted external adapter's immutable completed processing result.
#[derive(Debug)]
#[must_use = "prepare publication or discard the admitted native result"]
pub struct HtmlMetaCspPrepared {
    qualification: HtmlMetaCspQualification,
    policy: HtmlCspPolicyState,
}
impl HtmlMetaCspPrepared {
    pub fn qualification(&self) -> &HtmlMetaCspQualification {
        &self.qualification
    }
    pub fn policy(&self) -> &HtmlCspPolicyState {
        &self.policy
    }
    pub fn retained_layout(&self) -> Result<HtmlCspArtifactLayout, HtmlCspProtocolError> {
        prepared_layout(&self.policy, &self.qualification)
    }
}
fn prepared_layout(
    policy: &HtmlCspPolicyState,
    q: &HtmlMetaCspQualification,
) -> Result<HtmlCspArtifactLayout, HtmlCspProtocolError> {
    HtmlCspArtifactLayout::inline::<HtmlMetaCspPrepared>(10)
        .add(policy.retained_layout())?
        .add(q.text_layout()?)
}

/// Non-Clone external backend issuer. Supported semantic profiles are explicit
/// declarations, not a mutable active-version registry or policy processor.
#[derive(Debug)]
pub struct HtmlCspProcessorAuthority {
    identity: ProcessorId,
    policy_serial: u64,
}
impl HtmlCspProcessorAuthority {
    pub fn new_with<E>(
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<Self, HtmlCspError<E>> {
        let retained = HtmlCspArtifactLayout::inline::<Self>(2);
        admit(HtmlCspLayout::retain::<Self>(
            retained,
            HtmlCspArtifactLayout::default(),
            HtmlCspStorage::default(),
            retained.disposal(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        Ok(Self {
            identity: ProcessorId(Arc::new(())),
            policy_serial: 0,
        })
    }
    /// Declare a semantic profile actually supported by this external backend.
    /// The explicitly supplied processor exposes its selected profile by borrow.
    pub fn profile_with<E>(
        &self,
        semantic_version: u64,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<HtmlCspProcessorProfile, HtmlCspError<E>> {
        let retained = HtmlCspArtifactLayout::inline::<HtmlCspProcessorProfile>(1);
        admit(HtmlCspLayout::retain::<HtmlCspProcessorProfile>(
            retained,
            HtmlCspArtifactLayout::default(),
            HtmlCspStorage::default(),
            retained.disposal(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        Ok(HtmlCspProcessorProfile {
            issuer: self.identity.clone(),
            version: semantic_version,
        })
    }
    fn require(&self, request: &HtmlMetaCspRequest) -> Result<(), HtmlCspProtocolError> {
        if self.identity != request.qualification.0.processor.issuer {
            return Err(HtmlCspProtocolError::WrongProcessor);
        }
        if let Some(prior) = request.prior.profile() {
            prior.require(&request.qualification.0.processor)?;
        }
        Ok(())
    }
    /// Attest actual complete changed processing. Native graph construction and
    /// its partial cleanup were already admitted by the supplied processor.
    /// Incoming native custody moves; only additional wrapper/clone closing
    /// work is new. The descriptor is read exactly once and then cached.
    pub fn attest_changed_with<A: HtmlCspPolicyArtifact, E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        artifact: Arc<A>,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<HtmlMetaCspPrepared, HtmlCspRejected<Arc<A>, E>> {
        let ready = (|| {
            admit(HtmlCspLayout::read::<Self>(12)).map_err(HtmlCspError::Interrupted)?;
            self.require(request)?;
            let next = self
                .policy_serial
                .checked_add(1)
                .ok_or(HtmlCspProtocolError::RepresentationExhausted)?;
            // The artifact contract makes this one fixed, nonpanicking metadata
            // read. It is not a processing or destruction callback.
            let native = artifact.retained_layout();
            let own = HtmlCspArtifactLayout::inline::<PolicyRecord>(12)
                .add(request.qualification.text_layout()?)?;
            let policy_layout = own.add(native)?;
            let output_own = HtmlCspArtifactLayout::inline::<HtmlMetaCspPrepared>(10)
                .add(request.qualification.text_layout()?)?
                .add(own)?;
            let output = output_own.add(native)?;
            admit(HtmlCspLayout::retain::<HtmlMetaCspPrepared>(
                output,
                native,
                request.retained_layout()?.storage(),
                output_own.disposal(),
            )?)
            .map_err(HtmlCspError::Interrupted)?;
            Ok((next, policy_layout))
        })();
        let (next, layout) = match ready {
            Ok(ready) => ready,
            Err(error) => {
                return Err(HtmlCspRejected {
                    retained: artifact,
                    error,
                });
            }
        };
        let policy = HtmlCspPolicyState(Arc::new(PolicyRecord {
            id: HtmlCspPolicyId {
                issuer: PolicyIssuer::External(self.identity.clone()),
                serial: next,
            },
            revision: HtmlCspPolicyRevision(next),
            profile: Some(request.qualification.0.processor.clone()),
            native: Some(artifact),
            origin: Some(request.qualification.clone()),
            layout,
        }));
        let result = HtmlMetaCspPrepared {
            qualification: request.qualification.clone(),
            policy,
        };
        self.policy_serial = next;
        Ok(result)
    }
    /// Attest genuine completed processing whose exact prior artifact is unchanged.
    pub fn attest_unchanged_with<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<HtmlMetaCspPrepared, HtmlCspError<E>> {
        admit(HtmlCspLayout::read::<Self>(8)).map_err(HtmlCspError::Interrupted)?;
        self.require(request)?;
        let output = prepared_layout(&request.prior, &request.qualification)?;
        admit(HtmlCspLayout::retain::<HtmlMetaCspPrepared>(
            output,
            HtmlCspArtifactLayout::default(),
            request.retained_layout()?.storage(),
            output.disposal(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        Ok(HtmlMetaCspPrepared {
            qualification: request.qualification.clone(),
            policy: request.prior.clone(),
        })
    }
}

impl HtmlCspOwner {
    pub fn new_with<E>(
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<Self, HtmlCspError<E>> {
        let retained = HtmlCspArtifactLayout::inline::<Self>(2);
        admit(HtmlCspLayout::retain::<Self>(
            retained,
            HtmlCspArtifactLayout::default(),
            HtmlCspStorage::default(),
            retained.disposal(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        Ok(Self {
            identity: HtmlCspOwnerId(Arc::new(())),
            document_serial: 0,
            occurrence_serial: 0,
        })
    }
    pub fn identity(&self) -> &HtmlCspOwnerId {
        &self.identity
    }
    /// Establish only at actual controlled creation; not a substitute for
    /// unknown imported history. The adapter owns the actual Document binding.
    pub fn initial_document_with<E>(
        &mut self,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<HtmlCspAssociation, HtmlCspError<E>> {
        admit(HtmlCspLayout::read::<Self>(6)).map_err(HtmlCspError::Interrupted)?;
        let next = self
            .document_serial
            .checked_add(1)
            .ok_or(HtmlCspProtocolError::RepresentationExhausted)?;
        let policy_layout = HtmlCspArtifactLayout::inline::<PolicyRecord>(8);
        let retained =
            HtmlCspArtifactLayout::inline::<HtmlCspAssociation>(12).add(policy_layout)?;
        admit(HtmlCspLayout::retain::<HtmlCspAssociation>(
            retained,
            HtmlCspArtifactLayout::default(),
            HtmlCspStorage::default(),
            retained.disposal(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        let document = HtmlCspDocumentId {
            owner: self.identity.clone(),
            serial: next,
        };
        let policy = HtmlCspPolicyState(Arc::new(PolicyRecord {
            id: HtmlCspPolicyId {
                issuer: PolicyIssuer::Initial(document.clone()),
                serial: 0,
            },
            revision: HtmlCspPolicyRevision(0),
            profile: None,
            native: None,
            origin: None,
            layout: policy_layout,
        }));
        let result = HtmlCspAssociation {
            generation: HtmlCspAssociationGeneration {
                document: document.clone(),
                serial: 0,
            },
            origin: HtmlCspAssociationOrigin {
                original: document.clone(),
                from: None,
            },
            document,
            policy,
            latest: None,
        };
        self.document_serial = next;
        Ok(result)
    }
    /// Retain the actual artifact without replay, issuing new destination
    /// authority. The adapter establishes the actual transfer/inheritance relation.
    pub fn associate_document_with<E>(
        &mut self,
        source: &HtmlCspAssociation,
        cause: HtmlCspAssociationCause,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<HtmlCspAssociation, HtmlCspError<E>> {
        admit(HtmlCspLayout::read::<Self>(10)).map_err(HtmlCspError::Interrupted)?;
        let next = self
            .document_serial
            .checked_add(1)
            .ok_or(HtmlCspProtocolError::RepresentationExhausted)?;
        let retained = source.retained_layout()?;
        admit(HtmlCspLayout::retain::<HtmlCspAssociation>(
            retained,
            HtmlCspArtifactLayout::default(),
            retained.storage(),
            retained.disposal(),
        )?)
        .map_err(HtmlCspError::Interrupted)?;
        let document = HtmlCspDocumentId {
            owner: self.identity.clone(),
            serial: next,
        };
        let result = HtmlCspAssociation {
            generation: HtmlCspAssociationGeneration {
                document: document.clone(),
                serial: 0,
            },
            origin: HtmlCspAssociationOrigin {
                original: source.origin.original.clone(),
                from: Some((source.document.clone(), source.generation.clone(), cause)),
            },
            document,
            policy: source.policy.clone(),
            latest: source.latest.clone(),
        };
        self.document_serial = next;
        Ok(result)
    }
    /// Attest the actual reached insertion guards; the owning adapter pairs this
    /// occurrence with its native Source/construction cause. Content stays exact.
    pub fn request_meta_insertion_with<E>(
        &mut self,
        current: &HtmlCspAssociation,
        processor: &HtmlCspProcessorProfile,
        content: DocumentString,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<HtmlMetaCspRequest, HtmlCspRejected<DocumentString, E>> {
        let ready = (|| {
            admit(HtmlCspLayout::read::<Self>(14)).map_err(HtmlCspError::Interrupted)?;
            if current.document.owner != self.identity {
                return Err(HtmlCspProtocolError::ForeignOwner.into());
            }
            if let Some(profile) = current.policy.profile() {
                profile.require(processor)?;
            }
            if content.is_empty() {
                return Err(HtmlCspProtocolError::EmptyContent.into());
            }
            current
                .generation
                .serial
                .checked_add(1)
                .ok_or(HtmlCspProtocolError::RepresentationExhausted)?;
            let next = self
                .occurrence_serial
                .checked_add(1)
                .ok_or(HtmlCspProtocolError::RepresentationExhausted)?;
            let input = HtmlCspArtifactLayout::text(&content)?;
            let additional = HtmlCspArtifactLayout::inline::<HtmlMetaCspRequest>(10)
                .add(current.policy.retained_layout())?
                .add(HtmlCspArtifactLayout::inline::<QualificationRecord>(12))?;
            let retained = additional.add(input)?;
            admit(HtmlCspLayout::retain::<HtmlMetaCspRequest>(
                retained,
                input,
                current.retained_layout()?.storage(),
                additional.disposal(),
            )?)
            .map_err(HtmlCspError::Interrupted)?;
            Ok(next)
        })();
        let next = match ready {
            Ok(next) => next,
            Err(error) => {
                return Err(HtmlCspRejected {
                    retained: content,
                    error,
                });
            }
        };
        let result = HtmlMetaCspRequest {
            qualification: HtmlMetaCspQualification(Arc::new(QualificationRecord {
                document: current.document.clone(),
                generation: current.generation.clone(),
                prior_policy: current.policy.id().clone(),
                prior_revision: current.policy.revision(),
                processor: processor.clone(),
                occurrence: HtmlMetaCspOccurrence {
                    document: current.document.clone(),
                    serial: next,
                },
                content,
            })),
            prior: current.policy.clone(),
        };
        self.occurrence_serial = next;
        Ok(result)
    }
}

#[cfg(test)]
mod representation_tests {
    use super::*;
    fn allow(_: HtmlCspLayout) -> Result<(), ()> {
        Ok(())
    }

    #[test]
    fn exhausted_document_and_occurrence_issuance_refuse_without_reusing_identity() {
        let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
        let association = owner.initial_document_with(&mut allow).unwrap();
        owner.document_serial = u64::MAX;
        assert!(matches!(
            owner.initial_document_with(&mut allow),
            Err(HtmlCspError::Protocol(
                HtmlCspProtocolError::RepresentationExhausted
            ))
        ));
        let authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(0, &mut allow).unwrap();
        owner.occurrence_serial = u64::MAX;
        let rejected = owner
            .request_meta_insertion_with(
                &association,
                &profile,
                DocumentString::from("exact input"),
                &mut allow,
            )
            .unwrap_err();
        assert!(matches!(
            rejected.error,
            HtmlCspError::Protocol(HtmlCspProtocolError::RepresentationExhausted)
        ));
        assert_eq!(rejected.retained, DocumentString::from("exact input"));
        assert!(association.latest().is_none());
        assert_eq!(owner.occurrence_serial, u64::MAX);
    }

    #[test]
    fn exhausted_association_generation_refuses_before_native_processing() {
        let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
        let mut association = owner.initial_document_with(&mut allow).unwrap();
        let authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(0, &mut allow).unwrap();
        association.generation.serial = u64::MAX;
        let input = DocumentString::from("input");
        let rejected = owner
            .request_meta_insertion_with(&association, &profile, input, &mut allow)
            .unwrap_err();
        assert!(matches!(
            rejected.error,
            HtmlCspError::Protocol(HtmlCspProtocolError::RepresentationExhausted)
        ));
        assert_eq!(owner.occurrence_serial, 0);
        assert!(association.policy().profile().is_none());
    }

    #[derive(Debug)]
    struct Native;
    impl HtmlCspPolicyArtifact for Native {
        fn retained_layout(&self) -> HtmlCspArtifactLayout {
            HtmlCspArtifactLayout::default()
        }
    }
    #[test]
    fn exhausted_policy_revision_returns_native_allocation_without_issuing_state() {
        let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
        let association = owner.initial_document_with(&mut allow).unwrap();
        let mut authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(0, &mut allow).unwrap();
        let request = owner
            .request_meta_insertion_with(
                &association,
                &profile,
                DocumentString::from("input"),
                &mut allow,
            )
            .unwrap();
        authority.policy_serial = u64::MAX;
        let native = Arc::new(Native);
        let pointer = Arc::as_ptr(&native);
        let rejected = authority
            .attest_changed_with(&request, native, &mut allow)
            .unwrap_err();
        assert!(matches!(
            rejected.error,
            HtmlCspError::Protocol(HtmlCspProtocolError::RepresentationExhausted)
        ));
        assert_eq!(Arc::as_ptr(&rejected.retained), pointer);
        assert_eq!(authority.policy_serial, u64::MAX);
    }
}

/// Actual receipt: original exact qualification and real resulting policy.
#[derive(Clone, Debug)]
pub struct HtmlMetaCspReceipt {
    qualification: HtmlMetaCspQualification,
    policy: HtmlCspPolicyState,
}
impl HtmlMetaCspReceipt {
    pub fn qualification(&self) -> &HtmlMetaCspQualification {
        &self.qualification
    }
    pub fn policy(&self) -> &HtmlCspPolicyState {
        &self.policy
    }
    pub fn retained_layout(&self) -> Result<HtmlCspArtifactLayout, HtmlCspProtocolError> {
        receipt_layout(&self.policy, &self.qualification)
    }
}
fn receipt_layout(
    policy: &HtmlCspPolicyState,
    q: &HtmlMetaCspQualification,
) -> Result<HtmlCspArtifactLayout, HtmlCspProtocolError> {
    HtmlCspArtifactLayout::inline::<HtmlMetaCspReceipt>(10)
        .add(policy.retained_layout())?
        .add(q.text_layout()?)
}

/// A prepared local association change. Its borrow excludes intervening shared
/// transitions; actual canonical currentness remains the consuming adapter's job.
#[derive(Debug)]
#[must_use = "publish or abort this prepared local association change"]
pub struct HtmlMetaCspPublication<'a> {
    _owner: &'a mut HtmlCspOwner,
    current: &'a mut HtmlCspAssociation,
    next: HtmlCspAssociation,
    receipt: HtmlMetaCspReceipt,
    request: HtmlMetaCspRequest,
    prepared: HtmlMetaCspPrepared,
}

/// Owned publication evidence and the actual retired whole association. Keeping
/// retired custody out of the tail avoids a last native destructor during commit.
#[derive(Debug)]
pub struct HtmlMetaCspCompletion {
    receipt: HtmlMetaCspReceipt,
    retired: HtmlCspAssociation,
    // Move all input custody out of the publication tail too.
    request: HtmlMetaCspRequest,
    prepared: HtmlMetaCspPrepared,
}
impl HtmlMetaCspCompletion {
    pub fn receipt(&self) -> &HtmlMetaCspReceipt {
        &self.receipt
    }
    pub fn retired(&self) -> &HtmlCspAssociation {
        &self.retired
    }
    /// Returns every actual owned occurrence, allowing the adapter to drop each
    /// backing before refunding that occurrence's original receipt.
    pub fn into_parts(
        self,
    ) -> (
        HtmlMetaCspReceipt,
        HtmlCspAssociation,
        HtmlMetaCspRequest,
        HtmlMetaCspPrepared,
    ) {
        (self.receipt, self.retired, self.request, self.prepared)
    }
}
impl HtmlMetaCspPublication<'_> {
    /// Only moves and a prechecked swap. No callback, allocation, fallible work,
    /// or last backing destruction occurs after changing the supplied association.
    pub fn publish(self) -> HtmlMetaCspCompletion {
        let Self {
            current,
            next,
            receipt,
            request,
            prepared,
            ..
        } = self;
        let retired = std::mem::replace(current, next);
        HtmlMetaCspCompletion {
            receipt,
            retired,
            request,
            prepared,
        }
    }
    pub fn abort(self) -> (HtmlMetaCspRequest, HtmlMetaCspPrepared) {
        (self.request, self.prepared)
    }
}
impl HtmlCspOwner {
    pub fn prepare_publication_with<'a, E>(
        &'a mut self,
        current: &'a mut HtmlCspAssociation,
        request: HtmlMetaCspRequest,
        prepared: HtmlMetaCspPrepared,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
    ) -> Result<
        HtmlMetaCspPublication<'a>,
        HtmlCspRejected<(HtmlMetaCspRequest, HtmlMetaCspPrepared), E>,
    > {
        let ready = (|| {
            admit(HtmlCspLayout::read::<HtmlMetaCspPublication<'_>>(24))
                .map_err(HtmlCspError::Interrupted)?;
            let q = &request.qualification;
            if current.document.owner != self.identity || q.0.document.owner != self.identity {
                return Err(HtmlCspProtocolError::ForeignOwner.into());
            }
            if current.document != q.0.document {
                return Err(HtmlCspProtocolError::ForeignDocument.into());
            }
            if !q.matches(&prepared.qualification) {
                return Err(HtmlCspProtocolError::WrongRequest.into());
            }
            if current.generation != q.0.generation {
                return Err(HtmlCspProtocolError::StaleAssociation.into());
            }
            if current.policy.id() != &q.0.prior_policy
                || current.policy.revision() != q.0.prior_revision
                || current.policy != request.prior
            {
                return Err(HtmlCspProtocolError::PriorPolicyMismatch.into());
            }
            if let Some(profile) = current.policy.profile() {
                profile.require(&q.0.processor)?;
            }
            if let Some(profile) = prepared.policy.profile() {
                profile.require(&q.0.processor)?;
            }
            let generation = current
                .generation
                .serial
                .checked_add(1)
                .ok_or(HtmlCspProtocolError::RepresentationExhausted)?;
            let next_layout = association_layout(&prepared.policy, Some(q))?;
            let receipt_layout = receipt_layout(&prepared.policy, q)?;
            let additional = next_layout.add(receipt_layout)?;
            let transferred = request
                .retained_layout()?
                .add(prepared.retained_layout()?)?
                .add(current.retained_layout()?)?;
            // The component headers become the actual completion/current
            // fields. Do not price the enclosing completion a second time.
            let output = additional.add(transferred)?;
            admit(HtmlCspLayout::retain::<HtmlMetaCspPublication<'_>>(
                output,
                transferred,
                HtmlCspStorage::inline::<(&mut HtmlCspOwner, &mut HtmlCspAssociation)>(),
                additional.disposal(),
            )?)
            .map_err(HtmlCspError::Interrupted)?;
            Ok(generation)
        })();
        let generation = match ready {
            Ok(generation) => generation,
            Err(error) => {
                return Err(HtmlCspRejected {
                    retained: (request, prepared),
                    error,
                });
            }
        };
        let next = HtmlCspAssociation {
            document: current.document.clone(),
            generation: HtmlCspAssociationGeneration {
                document: current.document.clone(),
                serial: generation,
            },
            policy: prepared.policy.clone(),
            latest: Some(request.qualification.clone()),
            origin: current.origin.clone(),
        };
        let receipt = HtmlMetaCspReceipt {
            qualification: request.qualification.clone(),
            policy: prepared.policy.clone(),
        };
        Ok(HtmlMetaCspPublication {
            _owner: self,
            current,
            next,
            receipt,
            request,
            prepared,
        })
    }
}
