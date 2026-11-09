//! Pure HTML877d noopener invocation over qualified actual URL/settings facts.
use super::super::index::SubjectChains;
use super::*;
use surgeist_html::{
    bounded::rel::{HtmlRelInputs, HtmlRelInterpretation, HtmlRelMeter, HtmlRelReservation},
    elements::link::{HtmlRelKeyword, HtmlRelRole, HtmlRelSpelling},
    values::{Attribute, DocumentString},
};

identity!(HtmlRelUrlId, fresh);
identity!(HtmlRelOriginId, fresh);
identity!(HtmlRelSettingsId, fresh);
/// Actual URL record identity and its producer revision, not a serialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRelUrlKey {
    identity: HtmlRelUrlId,
    revision: HtmlRelRevision,
}
impl HtmlRelUrlKey {
    pub fn new(identity: HtmlRelUrlId, revision: HtmlRelRevision) -> Self {
        Self { identity, revision }
    }
    pub fn identity(&self) -> &HtmlRelUrlId {
        &self.identity
    }
    pub fn revision(&self) -> &HtmlRelRevision {
        &self.revision
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlRelNoopenerInput {
    capture: HtmlCaptureId,
    host: HtmlHostFactsRevision,
    subject: HtmlCaptureSubjectId,
    url: HtmlRelUrlKey,
    target: DocumentString,
}
impl HtmlRelNoopenerInput {
    /// Formation admits the concrete request payload and binds the actual role,
    /// capture, host and independently expected URL producer revision.
    pub fn new(
        capture: &HtmlCapture,
        expected_host: &HtmlHostFactsRevision,
        subject: HtmlCaptureSubjectId,
        expected_url: &HtmlRelRevision,
        url: HtmlRelUrlKey,
        target: DocumentString,
        limits: HtmlRelFactsLimits,
    ) -> Result<Self, HtmlRelFactsError> {
        FactsAdmission::host(capture, expected_host)?;
        if &url.revision != expected_url {
            return Err(HtmlRelFactsError::UrlRevisionMismatch);
        }
        let mut a = FactsAdmission::new(limits);
        a.work(1)?;
        if limits.records < 1 {
            return Err(HtmlRelFactsError::RecordLimit);
        }
        a.bytes = std::mem::size_of::<Self>();
        if a.bytes > limits.bytes {
            return Err(HtmlRelFactsError::ByteLimit);
        }
        if a.role(capture, &subject)? == HtmlRelRole::Link {
            return Err(HtmlRelFactsError::InvalidRole);
        }
        a.raw(&target)?;
        Ok(Self {
            capture: capture.identity().clone(),
            host: expected_host.clone(),
            subject,
            url,
            target,
        })
    }
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn url(&self) -> &HtmlRelUrlKey {
        &self.url
    }
    pub fn target(&self) -> &DocumentString {
        &self.target
    }
}
#[derive(Debug)]
pub enum HtmlRelBlobEntry {
    Unavailable,
    Null,
    NonNull { environment_origin: HtmlRelOriginId },
}
#[derive(Debug)]
pub struct HtmlRelUrlObservation {
    subject: HtmlCaptureSubjectId,
    url: HtmlRelUrlKey,
    blob: HtmlRelBlobEntry,
}
impl HtmlRelUrlObservation {
    pub fn new(subject: HtmlCaptureSubjectId, url: HtmlRelUrlKey, blob: HtmlRelBlobEntry) -> Self {
        Self { subject, url, blob }
    }
}
/// An actual settings/top-level-origin comparison for this URL's blob origin.
/// HTML consumes the qualified result and does not implement origin comparison.
#[derive(Debug)]
pub struct HtmlRelSameSiteObservation {
    subject: HtmlCaptureSubjectId,
    url: HtmlRelUrlKey,
    settings: HtmlRelSettingsId,
    blob_origin: HtmlRelOriginId,
    top_level_origin: HtmlRelOriginId,
    same_site: bool,
}
impl HtmlRelSameSiteObservation {
    pub fn new(
        subject: HtmlCaptureSubjectId,
        url: HtmlRelUrlKey,
        settings: HtmlRelSettingsId,
        blob_origin: HtmlRelOriginId,
        top_level_origin: HtmlRelOriginId,
        same_site: bool,
    ) -> Self {
        Self {
            subject,
            url,
            settings,
            blob_origin,
            top_level_origin,
            same_site,
        }
    }
    pub fn settings(&self) -> &HtmlRelSettingsId {
        &self.settings
    }
    pub fn top_level_origin(&self) -> &HtmlRelOriginId {
        &self.top_level_origin
    }
}
pub(super) fn validate(
    input: &HtmlRelFactsInput,
    capture: &HtmlCapture,
    a: &mut FactsAdmission,
) -> Result<(), HtmlRelFactsError> {
    a.vector(&input.urls)?;
    a.vector(&input.same_site)?;
    // Observations are told apart by their subject and an opaque URL identity,
    // so each is compared only with the others of its subject.
    let mut url_chains = SubjectChains::new(
        capture.identity(),
        input.urls.len(),
        |n| a.work(n),
        HtmlRelFactsError::RepresentationExhausted,
    )?;
    for o in &input.urls {
        url_chains.link(&o.subject);
    }
    let mut same_site_chains = SubjectChains::new(
        capture.identity(),
        input.same_site.len(),
        |n| a.work(n),
        HtmlRelFactsError::RepresentationExhausted,
    )?;
    for o in &input.same_site {
        same_site_chains.link(&o.subject);
    }
    for (i, o) in input.urls.iter().enumerate() {
        if a.role(capture, &o.subject)? == HtmlRelRole::Link {
            return Err(HtmlRelFactsError::InvalidRole);
        }
        a.work(1)?;
        if input.revisions.url.as_ref() != Some(&o.url.revision) {
            return Err(HtmlRelFactsError::UrlRevisionMismatch);
        }
        let mut earlier = url_chains.before(i);
        while let Some(position) = earlier {
            a.work(2)?;
            if input.urls[position].url == o.url {
                return Err(HtmlRelFactsError::DuplicateObservation);
            }
            earlier = url_chains.before(position);
        }
    }
    for (i, o) in input.same_site.iter().enumerate() {
        if a.role(capture, &o.subject)? == HtmlRelRole::Link {
            return Err(HtmlRelFactsError::InvalidRole);
        }
        a.work(1)?;
        if input.revisions.url.as_ref() != Some(&o.url.revision) {
            return Err(HtmlRelFactsError::UrlRevisionMismatch);
        }
        let mut earlier = same_site_chains.before(i);
        while let Some(position) = earlier {
            a.work(2)?;
            if input.same_site[position].url == o.url {
                return Err(HtmlRelFactsError::DuplicateObservation);
            }
            earlier = same_site_chains.before(position);
        }
        let mut matched = false;
        let mut candidate = url_chains.latest(&o.subject);
        while let Some(position) = candidate {
            a.work(3)?;
            let url = &input.urls[position];
            if url.url == o.url {
                if let HtmlRelBlobEntry::NonNull { environment_origin } = &url.blob {
                    a.work(1)?;
                    matched = environment_origin == &o.blob_origin
                }
                break;
            }
            candidate = url_chains.before(position);
        }
        if !matched {
            return Err(HtmlRelFactsError::UrlBindingMismatch);
        }
    }
    Ok(())
}
struct ScratchControl<'s, 'a> {
    session: &'s mut HtmlSemanticSession<'a>,
    backing: ScratchBacking,
}
impl HtmlRelMeter for ScratchControl<'_, '_> {
    type Error = HtmlSemanticFailure;
    fn work(&mut self, n: usize) -> Result<(), Self::Error> {
        self.session.charge(n)
    }
    fn reserve(&mut self, guard: &mut HtmlRelReservation<'_>) -> Result<(), Self::Error> {
        let before = guard.capacity();
        let target = guard.target_capacity();
        let result = self.session.reserve_record_capacity(
            guard.len(),
            guard.capacity(),
            guard.record_size(),
            target.saturating_sub(guard.len()),
            HtmlSemanticResource::ScratchRecords,
            || {
                guard.try_reserve().map_err(|_| ())?;
                Ok(guard.capacity())
            },
        );
        // A refusal already holds its failure slot; the backing is still recorded when it can be.
        let Some(growth) = guard.capacity().checked_sub(before) else {
            return result.and(Err(self
                .session
                .stop(HtmlSemanticFailureKind::ProtocolViolated)));
        };
        let Some(backing) = self.backing.grown(growth, guard.record_size()) else {
            return result.and(Err(self
                .session
                .stop(HtmlSemanticFailureKind::RepresentationExhausted)));
        };
        self.backing = backing;
        result
    }
}
impl Drop for ScratchControl<'_, '_> {
    fn drop(&mut self) {
        self.session.release_scratch_backing(self.backing)
    }
}
impl<'a> HtmlSemanticSession<'a> {
    pub(in super::super) fn rel_noopener(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
        input: Option<&HtmlRelNoopenerInput>,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let outcome = self.rel_noopener_inner(subject, element, input);
        match outcome {
            Ok(value) => Ok(value),
            Err(error) => {
                // The inner scratch owner has now retired all actual temporary
                // backing. Preserve the first kind, work and qualification while
                // reporting the final owned scratch count at this child boundary.
                let scratch = self.snapshot().scratch_records;
                if let Some(first) = self.failure.as_mut() {
                    first.usage.scratch_records = scratch;
                    Err(first.clone())
                } else {
                    Err(error)
                }
            }
        }
    }
    fn rel_noopener_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        element: &'a HtmlCapturedElement,
        input: Option<&HtmlRelNoopenerInput>,
    ) -> Result<HtmlSemanticValue<bool>, HtmlSemanticFailure> {
        let rel = self.attribute(element, "rel")?.map(Attribute::value);
        let rev = self.attribute(element, "rev")?.map(Attribute::value);
        for raw in [rel, rev, input.map(|i| &i.target)] {
            self.charge(1)?;
            self.interpreter
                .admit_attribute_raw(raw)
                .map_err(|e| self.stop(HtmlSemanticFailureKind::Attribute(e)))?;
        }
        self.charge(1)?;
        let Some(role) = self.known(element)?.and_then(HtmlRelRole::from_known) else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if role == HtmlRelRole::Link {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let Some(input) = input else {
            return Ok(HtmlSemanticValue::Missing(Self::rel_missing(
                subject,
                HtmlRelMissingFact::NoopenerInput,
            )));
        };
        self.charge(4)?;
        if input.capture != *self.capture.identity()
            || input.host != *self.context.host_revision()
            || &input.subject != subject
        {
            return Err(self.stop(HtmlSemanticFailureKind::NoopenerRequestMismatch));
        }
        if let Some(expected) = self.context.rel_revisions.url.as_ref()
            && expected != &input.url.revision
        {
            return Err(self.stop(HtmlSemanticFailureKind::NoopenerRequestMismatch));
        }
        let mut meter = ScratchControl {
            session: self,
            backing: ScratchBacking::default(),
        };
        let mut owner =
            HtmlRelInterpretation::new(HtmlRelInputs::new(Some(role)).with_rel(rel).with_rev(rev));
        owner
            .interpret(&mut meter)
            .map_err(|error| meter.session.rel_action_failure(error))?;
        let local = owner
            .finish()
            .map_err(|failure| meter.session.rel_interpretation_failure(*failure.cause()))?;
        let mut no_opener = false;
        let mut opener = false;
        let mut unknown = false;
        for token in local.occurrences() {
            meter.work(3)?;
            let mut keyword = token.keyword();
            let mut unresolved = token.spelling() == HtmlRelSpelling::Unknown;
            if unresolved && let Some(facts) = meter.session.context.rel_facts.as_ref() {
                for observed in &facts.input.occurrences {
                    meter.work(3)?;
                    if &observed.subject != subject
                        || observed.start != token.code_unit_range().start
                        || observed.end != token.code_unit_range().end
                    {
                        continue;
                    }
                    match &observed.state {
                        RegistryOccurrence::Unavailable => {}
                        RegistryOccurrence::ProvenAbsent => unresolved = false,
                        RegistryOccurrence::Definition { key, .. } => {
                            for definition in &facts.input.definitions {
                                meter.work(1)?;
                                if &definition.key == key {
                                    keyword = definition.standard;
                                    unresolved = false;
                                    break;
                                }
                            }
                        }
                    }
                    break;
                }
            }
            meter.work(4)?;
            unknown |= unresolved;
            no_opener |= matches!(
                keyword,
                Some(HtmlRelKeyword::NoOpener | HtmlRelKeyword::NoReferrer)
            );
            opener |= keyword == Some(HtmlRelKeyword::Opener);
        }
        if no_opener {
            return Ok(HtmlSemanticValue::Known(true));
        }
        let envelope = input.target.len().checked_add(7).ok_or_else(|| {
            meter
                .session
                .stop(HtmlSemanticFailureKind::RepresentationExhausted)
        })?;
        meter.work(envelope)?;
        if !opener
            && !unknown
            && surgeist_html::bounded::rel::rel_ascii_keyword(input.target.code_units(), "_blank")
        {
            return Ok(HtmlSemanticValue::Known(true));
        }
        let mut blob = None;
        if let Some(facts) = meter.session.context.rel_facts.as_ref() {
            for record in &facts.input.urls {
                meter.work(2)?;
                if &record.subject == subject && record.url == input.url {
                    blob = Some(&record.blob);
                    break;
                }
            }
        }
        let result = match blob {
            Some(HtmlRelBlobEntry::Null) => Some(false),
            Some(HtmlRelBlobEntry::NonNull { environment_origin }) => {
                let mut same_site = None;
                if let Some(facts) = meter.session.context.rel_facts.as_ref() {
                    for record in &facts.input.same_site {
                        meter.work(3)?;
                        if &record.subject == subject
                            && record.url == input.url
                            && &record.blob_origin == environment_origin
                        {
                            same_site = Some(record.same_site);
                            break;
                        }
                    }
                }
                same_site.map(|v| !v)
            }
            _ => None,
        };
        meter.work(2)?;
        Ok(match result {
            Some(true) => HtmlSemanticValue::Known(true),
            Some(false) if !unknown => HtmlSemanticValue::Known(false),
            Some(false) => {
                HtmlSemanticValue::Missing(Self::rel_missing(subject, HtmlRelMissingFact::Registry))
            }
            None => HtmlSemanticValue::Missing(Self::rel_missing(
                subject,
                HtmlRelMissingFact::UrlRecord,
            )),
        })
    }
}
