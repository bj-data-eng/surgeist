//! Exact encoding-parse observations for A/Area set-the-url. No URL parser,
//! CSP selection, navigable, resource effect or second URL identity is created.
use crate::{
    DocumentString, HtmlBaseEncoding, HtmlBaseFactsError, HtmlBaseFactsLimits,
    HtmlBaseFactsRevision, HtmlBaseFactsUsage, HtmlBaseRetainedStorageUsage, HtmlBaseUrlRecord,
};
use std::{error::Error, fmt, sync::Arc};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkParseOutcome {
    Failure,
    Parsed(HtmlBaseUrlRecord),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlHyperlinkParseObservation {
    raw: DocumentString,
    outcome: HtmlHyperlinkParseOutcome,
}
impl HtmlHyperlinkParseObservation {
    /// Attests the actual URL-standard result for this exact raw input and the
    /// enclosing facts' base/encoding/revision, including Blob-store effects.
    pub fn new(raw: DocumentString, outcome: HtmlHyperlinkParseOutcome) -> Self {
        Self { raw, outcome }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkFactsError {
    Formation(HtmlBaseFactsError),
    ForeignProducer,
    RevisionMismatch,
    InputsMismatch,
    MissingObservation,
}
impl From<HtmlBaseFactsError> for HtmlHyperlinkFactsError {
    fn from(value: HtmlBaseFactsError) -> Self {
        Self::Formation(value)
    }
}
impl fmt::Display for HtmlHyperlinkFactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "hyperlink facts: {self:?}")
    }
}
impl Error for HtmlHyperlinkFactsError {}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkLookupError<E> {
    Facts(HtmlHyperlinkFactsError),
    Interrupted(E),
}
impl<E: fmt::Display> fmt::Display for HtmlHyperlinkLookupError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Facts(e) => e.fmt(f),
            Self::Interrupted(e) => e.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for HtmlHyperlinkLookupError<E> {}

#[derive(Debug)]
struct Facts {
    base: HtmlBaseUrlRecord,
    encoding: HtmlBaseEncoding,
    revision: HtmlBaseFactsRevision,
    observations: Vec<HtmlHyperlinkParseObservation>,
    usage: HtmlBaseFactsUsage,
    retained: HtmlBaseRetainedStorageUsage,
}
/// Immutable actual observations for ONE base/encoding/full revision context.
/// The adapter separately binds this context to its actual Document metadata,
/// current first-Base state and canonical revision. This constructor attests no
/// Document identity. Base is the selected Document base, not merely fallback;
/// it may be a historical frozen record. The URL revision must include every
/// influencing URL/Blob-store change; encoding is opaque identity, not a label.
#[derive(Clone, Debug)]
pub struct HtmlHyperlinkParseFacts(Arc<Facts>);
impl PartialEq for HtmlHyperlinkParseFacts {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlHyperlinkParseFacts {}
impl HtmlHyperlinkParseFacts {
    pub fn new(
        base: HtmlBaseUrlRecord,
        encoding: HtmlBaseEncoding,
        revision: HtmlBaseFactsRevision,
        observations: Vec<HtmlHyperlinkParseObservation>,
        limits: HtmlBaseFactsLimits,
    ) -> Result<Self, HtmlHyperlinkFactsError> {
        let mut v = crate::base::Validation::new(limits);
        v.records(observations.capacity())?;
        let mut retained = HtmlBaseRetainedStorageUsage {
            bytes: size_of::<Facts>(),
            units: 0,
        };
        retained.add(
            observations
                .capacity()
                .checked_mul(size_of::<HtmlHyperlinkParseObservation>())
                .ok_or(HtmlBaseFactsError::RepresentationExhausted)?,
        )?;
        retained.url(&base)?;
        v.retained_string(base.serialization())?;
        for (i, observation) in observations.iter().enumerate() {
            v.step()?;
            retained.string(&observation.raw)?;
            v.retained_string(&observation.raw)?;
            if let HtmlHyperlinkParseOutcome::Parsed(record) = &observation.outcome {
                if record.revision() != &revision {
                    return Err(HtmlHyperlinkFactsError::RevisionMismatch);
                }
                retained.url(record)?;
                v.retained_string(record.serialization())?;
            }
            for previous in &observations[..i] {
                v.step()?;
                if v.equal(&previous.raw, &observation.raw)? {
                    return Err(HtmlBaseFactsError::DuplicateObservation.into());
                }
            }
        }
        let usage = v.usage(observations.capacity());
        Ok(Self(Arc::new(Facts {
            base,
            encoding,
            revision,
            observations,
            usage,
            retained,
        })))
    }
    /// Owner-computed payload footprint, including private bank headers and
    /// actual retained UTF-16 capacity. Logical formation usage is unchanged.
    pub fn retained_storage_usage(&self) -> HtmlBaseRetainedStorageUsage {
        self.0.retained
    }
    pub fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn base(&self) -> &HtmlBaseUrlRecord {
        &self.0.base
    }
    pub fn encoding(&self) -> &HtmlBaseEncoding {
        &self.0.encoding
    }
    pub fn revision(&self) -> &HtmlBaseFactsRevision {
        &self.0.revision
    }
    pub fn formation_usage(&self) -> HtmlBaseFactsUsage {
        self.0.usage
    }
    /// Constant-work opaque identity qualification; serialization equality is
    /// insufficient. Expected revision comes from the independently bound owner.
    pub fn validate_inputs(
        &self,
        base: &HtmlBaseUrlRecord,
        encoding: &HtmlBaseEncoding,
        expected: &HtmlBaseFactsRevision,
    ) -> Result<(), HtmlHyperlinkFactsError> {
        if self.revision().producer() != expected.producer() {
            return Err(HtmlHyperlinkFactsError::ForeignProducer);
        }
        if self.revision() != expected {
            return Err(HtmlHyperlinkFactsError::RevisionMismatch);
        }
        if self.base() != base || self.encoding() != encoding {
            return Err(HtmlHyperlinkFactsError::InputsMismatch);
        }
        Ok(())
    }
    /// Charge/poll before context qualification, each observation and every
    /// compared raw code unit. No allocation or normalization is hidden; the
    /// returned snapshot shares only the selected inputs/outcome, not this Vec.
    /// Callback errors propagate exactly, including first failure/cancellation.
    pub fn resolve_with<E>(
        &self,
        raw: &DocumentString,
        base: &HtmlBaseUrlRecord,
        encoding: &HtmlBaseEncoding,
        expected: &HtmlBaseFactsRevision,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<HtmlHyperlinkUrlSnapshot, HtmlHyperlinkLookupError<E>> {
        step().map_err(HtmlHyperlinkLookupError::Interrupted)?;
        self.validate_inputs(base, encoding, expected)
            .map_err(HtmlHyperlinkLookupError::Facts)?;
        for observation in &self.0.observations {
            step().map_err(HtmlHyperlinkLookupError::Interrupted)?;
            if raw.len() != observation.raw.len() {
                continue;
            }
            let mut equal = true;
            for (a, b) in raw.code_units().iter().zip(observation.raw.code_units()) {
                step().map_err(HtmlHyperlinkLookupError::Interrupted)?;
                if a != b {
                    equal = false;
                    break;
                }
            }
            if equal {
                step().map_err(HtmlHyperlinkLookupError::Interrupted)?;
                let result_units = match &observation.outcome {
                    HtmlHyperlinkParseOutcome::Failure => 0,
                    HtmlHyperlinkParseOutcome::Parsed(record) => {
                        record.serialization().shared_code_units().capacity()
                    }
                };
                let units = observation
                    .raw
                    .shared_code_units()
                    .capacity()
                    .checked_add(base.serialization().shared_code_units().capacity())
                    .and_then(|n| n.checked_add(result_units))
                    .ok_or_else(|| {
                        HtmlHyperlinkLookupError::Facts(
                            HtmlBaseFactsError::RepresentationExhausted.into(),
                        )
                    })?;
                let mut retained = HtmlBaseRetainedStorageUsage {
                    bytes: size_of::<HtmlHyperlinkUrlSnapshot>(),
                    units: 0,
                };
                let footprint = (|| {
                    retained.string(&observation.raw)?;
                    retained.url(self.base())?;
                    if let HtmlHyperlinkParseOutcome::Parsed(record) = &observation.outcome {
                        retained.url(record)?;
                    }
                    Ok::<_, HtmlBaseFactsError>(())
                })();
                footprint.map_err(|e| HtmlHyperlinkLookupError::Facts(e.into()))?;
                debug_assert_eq!(retained.utf16_extent(), units);
                return Ok(HtmlHyperlinkUrlSnapshot {
                    raw: observation.raw.clone(),
                    base: self.base().clone(),
                    encoding: self.encoding().clone(),
                    revision: self.revision().clone(),
                    outcome: observation.outcome.clone(),
                    retained_bytes: retained.retained_bytes(),
                });
            }
        }
        Err(HtmlHyperlinkLookupError::Facts(
            HtmlHyperlinkFactsError::MissingObservation,
        ))
    }
}

/// Owner-issued exact parse history for canonical storage/capture. A failure is
/// an observed null URL; it differs from fresh/cleared storage with no snapshot.
/// There is no unchecked constructor or conversion from URL spelling/Rel IDs.
#[derive(Clone, Debug)]
pub struct HtmlHyperlinkUrlSnapshot {
    raw: DocumentString,
    base: HtmlBaseUrlRecord,
    encoding: HtmlBaseEncoding,
    revision: HtmlBaseFactsRevision,
    outcome: HtmlHyperlinkParseOutcome,
    retained_bytes: usize,
}
impl PartialEq for HtmlHyperlinkUrlSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
            && self.base == other.base
            && self.encoding == other.encoding
            && self.revision == other.revision
            && self.outcome == other.outcome
    }
}
impl Eq for HtmlHyperlinkUrlSnapshot {}
impl HtmlHyperlinkUrlSnapshot {
    /// Footprint of this selected observation only, without the original bank.
    pub fn retained_storage_usage(&self) -> HtmlBaseRetainedStorageUsage {
        HtmlBaseRetainedStorageUsage {
            bytes: self.retained_bytes,
            units: self.retained_utf16_extent(),
        }
    }
    pub fn raw(&self) -> &DocumentString {
        &self.raw
    }
    pub fn base(&self) -> &HtmlBaseUrlRecord {
        &self.base
    }
    pub fn encoding(&self) -> &HtmlBaseEncoding {
        &self.encoding
    }
    pub fn revision(&self) -> &HtmlBaseFactsRevision {
        &self.revision
    }
    pub fn outcome(&self) -> &HtmlHyperlinkParseOutcome {
        &self.outcome
    }
    pub fn url(&self) -> Option<&HtmlBaseUrlRecord> {
        match &self.outcome {
            HtmlHyperlinkParseOutcome::Failure => None,
            HtmlHyperlinkParseOutcome::Parsed(record) => Some(record),
        }
    }
    pub fn retained_utf16_extent(&self) -> usize {
        // These exact immutable three backings were checked during resolution.
        // Reusing that invariant keeps one cached word instead of enlarging
        // every inline captured Element merely for derived usage readback.
        self.raw
            .shared_code_units()
            .capacity()
            .checked_add(self.base.serialization().shared_code_units().capacity())
            .and_then(|n| {
                n.checked_add(
                    self.url()
                        .map_or(0, |r| r.serialization().shared_code_units().capacity()),
                )
            })
            .expect("qualified snapshot retained extent was checked")
    }
}

/// Capture formation preserves the consumer's first interruption exactly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkCaptureError<E> {
    Capture(crate::HtmlCaptureError),
    Interrupted(E),
}
impl<E: fmt::Display> fmt::Display for HtmlHyperlinkCaptureError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Capture(e) => e.fmt(f),
            Self::Interrupted(e) => e.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for HtmlHyperlinkCaptureError<E> {}
impl<E> From<crate::HtmlCaptureError> for HtmlHyperlinkCaptureError<E> {
    fn from(value: crate::HtmlCaptureError) -> Self {
        Self::Capture(value)
    }
}
/// Established cache history, independent of the current node Document.
/// Outer absence on a captured Element means unavailable evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedHyperlinkUrl {
    Null,
    Observed {
        document: crate::HtmlCaptureDocumentId,
        snapshot: HtmlHyperlinkUrlSnapshot,
    },
}
