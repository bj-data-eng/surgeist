//! Shared encoding-parse observations for actual Document-relative URL parsing.
//! A/Area history and Meta refresh consume the same qualified bank. No URL parser,
//! CSP selection, navigable, resource effect or second URL identity is created.
use crate::semantics::index::ContentChains;
use crate::{
    HtmlBaseEncoding, HtmlBaseFactsError, HtmlBaseFactsLimits, HtmlBaseFactsRevision,
    HtmlBaseFactsUsage, HtmlBaseRetainedStorageUsage, HtmlBaseUrlRecord,
};
use std::{error::Error, fmt, sync::Arc};
use surgeist_html::accounting::HtmlHostScope;
use surgeist_html::values::DocumentString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlEncodingUrlParseOutcome {
    Failure,
    Parsed(HtmlBaseUrlRecord),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlEncodingUrlParseObservation {
    raw: DocumentString,
    outcome: HtmlEncodingUrlParseOutcome,
}
impl HtmlEncodingUrlParseObservation {
    /// Attests the actual URL-standard result for this exact raw input and the
    /// enclosing facts' base/encoding/revision, including Blob-store effects.
    pub fn new(raw: DocumentString, outcome: HtmlEncodingUrlParseOutcome) -> Self {
        Self { raw, outcome }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlEncodingUrlFactsError {
    Formation(HtmlBaseFactsError),
    ForeignProducer,
    RevisionMismatch,
    InputsMismatch,
    MissingObservation,
}
impl HtmlEncodingUrlFactsError {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Formation(error) => error.accounting_fault(),
            Self::ForeignProducer
            | Self::RevisionMismatch
            | Self::InputsMismatch
            | Self::MissingObservation => None,
        }
    }
}
impl From<HtmlBaseFactsError> for HtmlEncodingUrlFactsError {
    fn from(value: HtmlBaseFactsError) -> Self {
        Self::Formation(value)
    }
}
impl fmt::Display for HtmlEncodingUrlFactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "encoding URL facts: {self:?}")
    }
}
impl Error for HtmlEncodingUrlFactsError {}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlEncodingUrlLookupError<E> {
    Facts(HtmlEncodingUrlFactsError),
    Interrupted(E),
}
impl<E> HtmlEncodingUrlLookupError<E> {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Facts(error) => error.accounting_fault(),
            Self::Interrupted(_) => None,
        }
    }
}
impl<E: fmt::Display> fmt::Display for HtmlEncodingUrlLookupError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Facts(e) => e.fmt(f),
            Self::Interrupted(e) => e.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for HtmlEncodingUrlLookupError<E> {}

#[derive(Debug)]
struct Facts {
    base: HtmlBaseUrlRecord,
    encoding: HtmlBaseEncoding,
    revision: HtmlBaseFactsRevision,
    observations: Vec<HtmlEncodingUrlParseObservation>,
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
pub struct HtmlEncodingUrlParseFacts(Arc<Facts>);
impl PartialEq for HtmlEncodingUrlParseFacts {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlEncodingUrlParseFacts {}
impl HtmlEncodingUrlParseFacts {
    pub fn new(
        base: HtmlBaseUrlRecord,
        encoding: HtmlBaseEncoding,
        revision: HtmlBaseFactsRevision,
        observations: Vec<HtmlEncodingUrlParseObservation>,
        limits: HtmlBaseFactsLimits,
    ) -> Result<Self, HtmlEncodingUrlFactsError> {
        Self::build(base, encoding, revision, observations, limits, None)
    }
    /// Charges below the scope `accounting` names, in the host's operation, as [`Self::new`]
    /// charges an operation of its own. The schema module documentation of
    /// `surgeist_html::accounting` lists the scopes it creates and what the host keeps.
    pub fn with_accounting(
        base: HtmlBaseUrlRecord,
        encoding: HtmlBaseEncoding,
        revision: HtmlBaseFactsRevision,
        observations: Vec<HtmlEncodingUrlParseObservation>,
        limits: HtmlBaseFactsLimits,
        accounting: &HtmlHostScope,
    ) -> Result<Self, HtmlEncodingUrlFactsError> {
        Self::build(
            base,
            encoding,
            revision,
            observations,
            limits,
            Some(accounting),
        )
    }
    fn build(
        base: HtmlBaseUrlRecord,
        encoding: HtmlBaseEncoding,
        revision: HtmlBaseFactsRevision,
        observations: Vec<HtmlEncodingUrlParseObservation>,
        limits: HtmlBaseFactsLimits,
        hosted: Option<&HtmlHostScope>,
    ) -> Result<Self, HtmlEncodingUrlFactsError> {
        let mut v = crate::base::Validation::charging(limits, hosted)?;
        v.records(observations.capacity())?;
        let mut retained = HtmlBaseRetainedStorageUsage {
            bytes: size_of::<Facts>(),
            units: 0,
        };
        retained.add(
            observations
                .capacity()
                .checked_mul(size_of::<HtmlEncodingUrlParseObservation>())
                .ok_or(HtmlBaseFactsError::RepresentationExhausted)?,
        )?;
        retained.url(&base)?;
        v.retained_string(base.serialization())?;
        // Observations are told apart by their raw text, so each is compared
        // only with the earlier observations of the same hash of it.
        let mut chains = ContentChains::new(
            observations.len(),
            |n| v.steps(n),
            HtmlBaseFactsError::RepresentationExhausted,
        )?;
        for observation in &observations {
            v.steps(observation.raw.len())?;
            chains.link(observation.raw.code_units());
        }
        for (i, observation) in observations.iter().enumerate() {
            v.step()?;
            retained.string(&observation.raw)?;
            v.retained_string(&observation.raw)?;
            if let HtmlEncodingUrlParseOutcome::Parsed(record) = &observation.outcome {
                if record.revision() != &revision {
                    return Err(HtmlEncodingUrlFactsError::RevisionMismatch);
                }
                retained.url(record)?;
                v.retained_string(record.serialization())?;
            }
            let mut earlier = chains.before(i);
            while let Some(position) = earlier {
                v.step()?;
                if v.equal(&observations[position].raw, &observation.raw)? {
                    return Err(HtmlBaseFactsError::DuplicateObservation.into());
                }
                earlier = chains.before(position);
            }
        }
        let usage = v.usage(observations.capacity())?;
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
    ) -> Result<(), HtmlEncodingUrlFactsError> {
        if self.revision().producer() != expected.producer() {
            return Err(HtmlEncodingUrlFactsError::ForeignProducer);
        }
        if self.revision() != expected {
            return Err(HtmlEncodingUrlFactsError::RevisionMismatch);
        }
        if self.base() != base || self.encoding() != encoding {
            return Err(HtmlEncodingUrlFactsError::InputsMismatch);
        }
        Ok(())
    }
    /// Borrow the exact observed outcome for these original UTF-16 units.
    /// Context qualification, observation headers, compared operands and the
    /// selected result are admitted before reading. No input copy, URL parse,
    /// normalization, retained history or allocation occurs. Failure differs
    /// from missing/foreign/stale evidence; caller interruption propagates first.
    pub fn lookup_units_with<E>(
        &self,
        raw: &[u16],
        base: &HtmlBaseUrlRecord,
        encoding: &HtmlBaseEncoding,
        expected: &HtmlBaseFactsRevision,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<&HtmlEncodingUrlParseOutcome, HtmlEncodingUrlLookupError<E>> {
        self.find_with(raw, base, encoding, expected, &mut step)
            .map(|observation| &observation.outcome)
    }
    /// Select actual A/Area history, retaining only the selected input/outcome.
    /// The original qualification, tariff and owner-computed footprint are
    /// preserved; this does not retain the observation Vec or create a URL.
    /// After qualification and checked footprint calculation, `retain` receives
    /// the exact selected snapshot payload, including Self and actual UTF-16
    /// capacities. It runs once before any selected handle is cloned; its first
    /// refusal propagates without retaining the snapshot. Earlier lookup or
    /// footprint failures skip it. This adds no lookup work steps. The caller
    /// supplies its retention/cancellation policy and owns resulting custody.
    /// An observed raw value that borrows a block of text is retained as an exact copy of its
    /// units. `retain` receives the length of the copy before its backing is reserved, so a
    /// refusal allocates nothing; the lookup has already visited every unit of it. A failed
    /// reservation, or one whose capacity exceeds the admitted length, is
    /// [`HtmlBaseFactsError::AllocationFailed`].
    pub fn resolve_hyperlink_with<E>(
        &self,
        raw: &DocumentString,
        base: &HtmlBaseUrlRecord,
        encoding: &HtmlBaseEncoding,
        expected: &HtmlBaseFactsRevision,
        mut step: impl FnMut() -> Result<(), E>,
        retain: impl FnOnce(HtmlBaseRetainedStorageUsage) -> Result<(), E>,
    ) -> Result<HtmlHyperlinkUrlSnapshot, HtmlEncodingUrlLookupError<E>> {
        let observation = self.find_with(raw.code_units(), base, encoding, expected, &mut step)?;
        // A raw value that borrows a block of text is retained as an exact copy of its units, so
        // the snapshot neither holds the block nor reports its capacity. The copy is admitted by
        // its length before its backing is reserved, and the capacity that is reserved must not
        // exceed what was admitted.
        let copied = observation.raw.borrowed_block().is_some();
        let raw_units = if copied {
            observation.raw.len()
        } else {
            observation.raw.retained_units()
        };
        let result_units = match &observation.outcome {
            HtmlEncodingUrlParseOutcome::Failure => 0,
            HtmlEncodingUrlParseOutcome::Parsed(record) => record.serialization().retained_units(),
        };
        let units = raw_units
            .checked_add(base.serialization().retained_units())
            .and_then(|n| n.checked_add(result_units))
            .ok_or_else(|| {
                HtmlEncodingUrlLookupError::Facts(
                    HtmlBaseFactsError::RepresentationExhausted.into(),
                )
            })?;
        let mut retained = HtmlBaseRetainedStorageUsage {
            bytes: size_of::<HtmlHyperlinkUrlSnapshot>(),
            units: 0,
        };
        let footprint = (|| {
            retained.string_of_units(raw_units)?;
            retained.url(self.base())?;
            if let HtmlEncodingUrlParseOutcome::Parsed(record) = &observation.outcome {
                retained.url(record)?;
            }
            Ok::<_, HtmlBaseFactsError>(())
        })();
        footprint.map_err(|e| HtmlEncodingUrlLookupError::Facts(e.into()))?;
        debug_assert_eq!(retained.utf16_extent(), units);
        retain(retained).map_err(HtmlEncodingUrlLookupError::Interrupted)?;
        let raw = if copied {
            let mut units = Vec::new();
            units.try_reserve_exact(raw_units).map_err(|_| {
                HtmlEncodingUrlLookupError::Facts(HtmlBaseFactsError::AllocationFailed.into())
            })?;
            if units.capacity() > raw_units {
                return Err(HtmlEncodingUrlLookupError::Facts(
                    HtmlBaseFactsError::AllocationFailed.into(),
                ));
            }
            units.extend_from_slice(observation.raw.code_units());
            DocumentString::from_utf16(units)
        } else {
            observation.raw.clone()
        };
        Ok(HtmlHyperlinkUrlSnapshot {
            raw,
            base: self.base().clone(),
            encoding: self.encoding().clone(),
            revision: self.revision().clone(),
            outcome: observation.outcome.clone(),
            retained_bytes: retained.retained_bytes(),
        })
    }
    fn find_with<E>(
        &self,
        raw: &[u16],
        base: &HtmlBaseUrlRecord,
        encoding: &HtmlBaseEncoding,
        expected: &HtmlBaseFactsRevision,
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<&HtmlEncodingUrlParseObservation, HtmlEncodingUrlLookupError<E>> {
        step().map_err(HtmlEncodingUrlLookupError::Interrupted)?;
        self.validate_inputs(base, encoding, expected)
            .map_err(HtmlEncodingUrlLookupError::Facts)?;
        for observation in &self.0.observations {
            step().map_err(HtmlEncodingUrlLookupError::Interrupted)?;
            if raw.len() != observation.raw.len() {
                continue;
            }
            let mut equal = true;
            for (a, b) in raw.iter().zip(observation.raw.code_units()) {
                step().map_err(HtmlEncodingUrlLookupError::Interrupted)?;
                if a != b {
                    equal = false;
                    break;
                }
            }
            if equal {
                step().map_err(HtmlEncodingUrlLookupError::Interrupted)?;
                return Ok(observation);
            }
        }
        Err(HtmlEncodingUrlLookupError::Facts(
            HtmlEncodingUrlFactsError::MissingObservation,
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
    outcome: HtmlEncodingUrlParseOutcome,
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
    pub fn outcome(&self) -> &HtmlEncodingUrlParseOutcome {
        &self.outcome
    }
    pub fn url(&self) -> Option<&HtmlBaseUrlRecord> {
        match &self.outcome {
            HtmlEncodingUrlParseOutcome::Failure => None,
            HtmlEncodingUrlParseOutcome::Parsed(record) => Some(record),
        }
    }
    pub fn retained_utf16_extent(&self) -> usize {
        // These exact immutable three backings were checked during resolution.
        // Reusing that invariant keeps one cached word instead of enlarging
        // every inline captured Element merely for derived usage readback.
        // Each backing is a `u16` allocation of at most `isize::MAX` bytes, so each capacity is at
        // most `isize::MAX / 2` and the sum of three is below `usize::MAX`: it cannot overflow.
        self.raw.retained_units()
            + self.base.serialization().retained_units()
            + self.url().map_or(0, |r| r.serialization().retained_units())
    }
}

/// Capture formation preserves the consumer's first interruption exactly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlHyperlinkCaptureError<E> {
    Capture(crate::HtmlCaptureError),
    Interrupted(E),
}
impl<E> HtmlHyperlinkCaptureError<E> {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Capture(error) => error.accounting_fault(),
            Self::Interrupted(_) => None,
        }
    }
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
