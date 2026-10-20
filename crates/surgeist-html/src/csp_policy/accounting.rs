use super::*;

/// Known retained storage, excluding opaque allocator and Arc bookkeeping.
/// Graphs shared by different occurrences may be conservatively reserved twice.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlCspStorage {
    bytes: usize,
    utf16_units: usize,
    entries: usize,
}
impl HtmlCspStorage {
    pub fn try_new(
        bytes: usize,
        utf16_units: usize,
        entries: usize,
    ) -> Result<Self, HtmlCspProtocolError> {
        if utf16_units.checked_mul(2).is_none_or(|n| n > bytes) {
            return Err(HtmlCspProtocolError::InvalidStorage);
        }
        Ok(Self {
            bytes,
            utf16_units,
            entries,
        })
    }
    pub fn bytes(self) -> usize {
        self.bytes
    }
    pub fn utf16_units(self) -> usize {
        self.utf16_units
    }
    pub fn entries(self) -> usize {
        self.entries
    }
    pub(super) fn add(self, other: Self) -> Result<Self, HtmlCspProtocolError> {
        Ok(Self {
            bytes: self
                .bytes
                .checked_add(other.bytes)
                .ok_or(HtmlCspProtocolError::RepresentationExhausted)?,
            utf16_units: self
                .utf16_units
                .checked_add(other.utf16_units)
                .ok_or(HtmlCspProtocolError::RepresentationExhausted)?,
            entries: self
                .entries
                .checked_add(other.entries)
                .ok_or(HtmlCspProtocolError::RepresentationExhausted)?,
        })
    }
    pub(super) fn inline<T>() -> Self {
        Self {
            bytes: size_of::<T>(),
            utf16_units: 0,
            entries: 1,
        }
    }
    pub(super) fn text(text: &DocumentString) -> Result<Self, HtmlCspProtocolError> {
        let units = text.shared_code_units().capacity();
        Self::try_new(
            units
                .checked_mul(2)
                .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                .ok_or(HtmlCspProtocolError::RepresentationExhausted)?,
            units,
            1,
        )
    }
}

/// Fixed upper bound on a possibly last strong occurrence's destruction.
/// Credits include transitive native release. Atomic bytes bound its cleanup
/// frame/stack; neither quantity is inferred from graph storage.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlCspDisposalWork {
    work_credits: u64,
    atomic_bytes: usize,
}
impl HtmlCspDisposalWork {
    pub fn try_new(work_credits: u64, atomic_bytes: usize) -> Result<Self, HtmlCspProtocolError> {
        // Keep the two independently meaningful quantities representable in
        // the common checked action extent used by consuming controllers.
        usize::try_from(work_credits).map_err(|_| HtmlCspProtocolError::RepresentationExhausted)?;
        Ok(Self {
            work_credits,
            atomic_bytes,
        })
    }
    pub fn work_credits(self) -> u64 {
        self.work_credits
    }
    pub fn atomic_bytes(self) -> usize {
        self.atomic_bytes
    }
    pub(super) fn add(self, other: Self) -> Result<Self, HtmlCspProtocolError> {
        Self::try_new(
            self.work_credits
                .checked_add(other.work_credits)
                .ok_or(HtmlCspProtocolError::RepresentationExhausted)?,
            self.atomic_bytes.max(other.atomic_bytes),
        )
    }
    pub(super) fn fixed<T>(credits: u64) -> Self {
        Self {
            work_credits: credits,
            atomic_bytes: size_of::<T>(),
        }
    }
}

/// Immutable native storage and independent disposal obligations, cached once.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlCspArtifactLayout {
    storage: HtmlCspStorage,
    disposal: HtmlCspDisposalWork,
}
impl HtmlCspArtifactLayout {
    pub fn try_new(
        storage: HtmlCspStorage,
        disposal: HtmlCspDisposalWork,
    ) -> Result<Self, HtmlCspProtocolError> {
        storage
            .bytes
            .checked_add(disposal.atomic_bytes)
            .ok_or(HtmlCspProtocolError::RepresentationExhausted)?;
        Ok(Self { storage, disposal })
    }
    pub fn storage(self) -> HtmlCspStorage {
        self.storage
    }
    pub fn disposal(self) -> HtmlCspDisposalWork {
        self.disposal
    }
    pub(super) fn add(self, other: Self) -> Result<Self, HtmlCspProtocolError> {
        Self::try_new(
            self.storage.add(other.storage)?,
            self.disposal.add(other.disposal)?,
        )
    }
    pub(super) fn inline<T>(credits: u64) -> Self {
        Self {
            storage: HtmlCspStorage::inline::<T>(),
            disposal: HtmlCspDisposalWork::fixed::<T>(credits),
        }
    }
    pub(super) fn text(text: &DocumentString) -> Result<Self, HtmlCspProtocolError> {
        Self::try_new(
            HtmlCspStorage::text(text)?,
            HtmlCspDisposalWork::fixed::<Arc<Vec<u16>>>(3),
        )
    }
}

/// Admission before an operation. `retained` describes its complete returned
/// custody; `transferred` is incoming owned custody that moves with its existing
/// receipt. Charge closing work only for new occurrences, never for that move.
/// `peak` includes incoming and outgoing overlap, with conservative graph pricing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlCspLayout {
    work_credits: u64,
    retained: HtmlCspArtifactLayout,
    transferred: HtmlCspArtifactLayout,
    peak: HtmlCspStorage,
    closing_work: HtmlCspDisposalWork,
    atomic_bytes: usize,
}
impl HtmlCspLayout {
    /// External processor admission before its next bounded native action.
    /// `live` includes all native custody introduced by that action; closing
    /// work is prepaid even for partial construction or an immediate refusal.
    pub fn processing(
        work_credits: u64,
        live: HtmlCspStorage,
        closing_work: HtmlCspDisposalWork,
        atomic_bytes: usize,
    ) -> Result<Self, HtmlCspProtocolError> {
        work_credits
            .checked_add(closing_work.work_credits)
            .ok_or(HtmlCspProtocolError::RepresentationExhausted)?;
        let retained = HtmlCspArtifactLayout::try_new(live, closing_work)?;
        Ok(Self {
            work_credits,
            retained,
            transferred: HtmlCspArtifactLayout::default(),
            peak: live,
            closing_work,
            atomic_bytes: atomic_bytes.max(closing_work.atomic_bytes),
        })
    }
    pub fn work_credits(self) -> u64 {
        self.work_credits
    }
    pub fn retained(self) -> HtmlCspArtifactLayout {
        self.retained
    }
    pub fn transferred(self) -> HtmlCspArtifactLayout {
        self.transferred
    }
    pub fn peak(self) -> HtmlCspStorage {
        self.peak
    }
    pub fn closing_work(self) -> HtmlCspDisposalWork {
        self.closing_work
    }
    pub fn atomic_bytes(self) -> usize {
        self.atomic_bytes
    }
    pub(super) fn read<T>(work_credits: u64) -> Self {
        Self {
            work_credits,
            atomic_bytes: size_of::<T>(),
            ..Self::default()
        }
    }
    pub(super) fn retain<T>(
        retained: HtmlCspArtifactLayout,
        transferred: HtmlCspArtifactLayout,
        overlap: HtmlCspStorage,
        closing_work: HtmlCspDisposalWork,
    ) -> Result<Self, HtmlCspProtocolError> {
        8_u64
            .checked_add(closing_work.work_credits)
            .ok_or(HtmlCspProtocolError::RepresentationExhausted)?;
        let peak = retained.storage.add(overlap)?;
        Ok(Self {
            work_credits: 8,
            retained,
            transferred,
            peak,
            closing_work,
            atomic_bytes: size_of::<T>().max(closing_work.atomic_bytes),
        })
    }
}
