use super::*;
use crate::internal::accounting::{HtmlChargeOverflow, HtmlChargedError, HtmlResource, HtmlWork};
use surgeist_ledger::{Change, Charge};

/// Known retained storage, excluding opaque allocator and Arc bookkeeping.
/// Graphs shared by different occurrences may be conservatively reserved twice.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlCspStorage {
    bytes: usize,
    utf16_units: usize,
    entries: usize,
}
impl HtmlCspStorage {
    /// Validates and builds a storage measure. A UTF-16 unit occupies two bytes, so
    /// `utf16_units * 2` must not exceed `bytes`.
    ///
    /// # Errors
    ///
    /// Returns the invalid-storage cause when the units cannot fit in `bytes`, which
    /// includes a unit count whose byte size overflows `usize`.
    pub fn try_new(
        bytes: usize,
        utf16_units: usize,
        entries: usize,
    ) -> Result<Self, HtmlCspProtocolError> {
        if utf16_units.checked_mul(2).is_none_or(|n| n > bytes) {
            return Err(HtmlCspProtocolErrorKind::InvalidStorage.error());
        }
        Ok(Self {
            bytes,
            utf16_units,
            entries,
        })
    }
    /// Returns the total known retained bytes, including the bytes that hold UTF-16
    /// units.
    pub fn bytes(self) -> usize {
        self.bytes
    }
    /// Returns the number of retained UTF-16 code units. Their two-byte size is
    /// already part of [`bytes`](Self::bytes).
    pub fn utf16_units(self) -> usize {
        self.utf16_units
    }
    /// Returns the number of separately counted retained entries (occurrences).
    pub fn entries(self) -> usize {
        self.entries
    }
    pub(super) fn add(self, other: Self) -> Result<Self, HtmlCspProtocolError> {
        Ok(Self {
            bytes: self
                .bytes
                .checked_add(other.bytes)
                .ok_or(HtmlCspProtocolErrorKind::RepresentationExhausted)?,
            utf16_units: self
                .utf16_units
                .checked_add(other.utf16_units)
                .ok_or(HtmlCspProtocolErrorKind::RepresentationExhausted)?,
            entries: self
                .entries
                .checked_add(other.entries)
                .ok_or(HtmlCspProtocolErrorKind::RepresentationExhausted)?,
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
        let units = text.retained_units();
        Self::try_new(
            units
                .checked_mul(2)
                .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                .ok_or(HtmlCspProtocolErrorKind::RepresentationExhausted)?,
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
    /// Builds a disposal bound from fixed work credits and the cleanup frame size.
    ///
    /// # Errors
    ///
    /// Returns the representation-exhausted cause when `work_credits` does not fit in
    /// `usize`, so that consuming controllers can charge it as one checked extent.
    pub fn try_new(work_credits: u64, atomic_bytes: usize) -> Result<Self, HtmlCspProtocolError> {
        // Keep the two independently meaningful quantities representable in
        // the common checked action extent used by consuming controllers.
        usize::try_from(work_credits)
            .map_err(|_| HtmlCspProtocolErrorKind::RepresentationExhausted)?;
        Ok(Self {
            work_credits,
            atomic_bytes,
        })
    }
    /// Returns the fixed credits that bound destroying a possibly last strong
    /// occurrence, including transitive native release.
    #[cfg(test)]
    pub(crate) fn work_credits(self) -> u64 {
        self.work_credits
    }
    /// Charges the credits that bound destroying a possibly last strong occurrence, plus `calls`
    /// provider calls of the caller's own that belong to the same action, as one change of
    /// provider calls. A sum that does not fit the ledger's `u64` is a fault of the charge.
    pub fn charge_with_calls<C: Charge<HtmlWork, HtmlResource>>(
        self,
        calls: u64,
        charge: &mut C,
    ) -> Result<(), HtmlChargedError<C::Stop, HtmlChargeOverflow>> {
        let credits = self
            .work_credits
            .checked_add(calls)
            .ok_or(HtmlChargedError::Faulted(HtmlChargeOverflow))?;
        charge
            .charge(&[Change::Work(HtmlWork::ProviderCall, credits)])
            .map_err(HtmlChargedError::Stopped)
    }
    /// Returns the bytes that bound the cleanup frame or stack of that destruction.
    pub fn atomic_bytes(self) -> usize {
        self.atomic_bytes
    }
    pub(super) fn add(self, other: Self) -> Result<Self, HtmlCspProtocolError> {
        Self::try_new(
            self.work_credits
                .checked_add(other.work_credits)
                .ok_or(HtmlCspProtocolErrorKind::RepresentationExhausted)?,
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
    /// Combines retained storage with its independent disposal obligations.
    ///
    /// # Errors
    ///
    /// Returns the representation-exhausted cause when `storage.bytes()` plus the
    /// disposal's atomic bytes overflows `usize`.
    pub fn try_new(
        storage: HtmlCspStorage,
        disposal: HtmlCspDisposalWork,
    ) -> Result<Self, HtmlCspProtocolError> {
        storage
            .bytes
            .checked_add(disposal.atomic_bytes)
            .ok_or(HtmlCspProtocolErrorKind::RepresentationExhausted)?;
        Ok(Self { storage, disposal })
    }
    /// Returns the known retained storage.
    pub fn storage(self) -> HtmlCspStorage {
        self.storage
    }
    /// Returns the bound on destroying this artifact's last occurrence.
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
            .ok_or(HtmlCspProtocolErrorKind::RepresentationExhausted)?;
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
    /// Charges the action's own bounded work and the prepaid credits of its closing work, as one
    /// change of provider calls, before the action runs. A sum that does not fit the ledger's
    /// `u64` is a fault of the charge. The atomic bytes stay with the caller, which records them
    /// as its own layout bound.
    pub fn charge_work<C: Charge<HtmlWork, HtmlResource>>(
        self,
        charge: &mut C,
    ) -> Result<(), HtmlChargedError<C::Stop, HtmlChargeOverflow>> {
        self.closing_work
            .charge_with_calls(self.work_credits, charge)
    }
    #[cfg(test)]
    pub(crate) fn work_credits(self) -> u64 {
        self.work_credits
    }
    /// Returns the complete custody that the operation's result retains.
    pub fn retained(self) -> HtmlCspArtifactLayout {
        self.retained
    }
    /// Returns the incoming owned custody that moves into the result under its
    /// existing receipt. Closing work is never charged again for it.
    pub fn transferred(self) -> HtmlCspArtifactLayout {
        self.transferred
    }
    /// Returns the peak storage of the operation, including the overlap of incoming
    /// and outgoing custody under conservative graph pricing.
    pub fn peak(self) -> HtmlCspStorage {
        self.peak
    }
    /// Returns the prepaid work for destroying only the occurrences that the operation
    /// newly creates.
    pub fn closing_work(self) -> HtmlCspDisposalWork {
        self.closing_work
    }
    /// Returns the bytes bounding the action's frame, never less than the closing
    /// work's own atomic bytes.
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
            .ok_or(HtmlCspProtocolErrorKind::RepresentationExhausted)?;
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

#[cfg(test)]
mod schema_tests {
    use super::*;
    use crate::internal::accounting::HtmlLimits;
    use surgeist_ledger::Operation;

    fn ledger() -> Operation<HtmlWork, HtmlResource> {
        Operation::new("csp", HtmlLimits::unlimited(), 1).unwrap()
    }

    /// No constructor builds a layout whose own and closing credits overflow, so
    /// the charge's overflow path is reached with fields set directly.
    #[test]
    fn a_layout_whose_credits_overflow_charges_nothing_and_faults() {
        let layout = HtmlCspLayout {
            work_credits: u64::MAX,
            closing_work: HtmlCspDisposalWork {
                work_credits: 1,
                atomic_bytes: 0,
            },
            ..HtmlCspLayout::default()
        };
        let mut operation = ledger();
        let root = operation.root();
        assert_eq!(
            layout.charge_work(&mut operation.charge_at(root)),
            Err(HtmlChargedError::Faulted(HtmlChargeOverflow))
        );
        assert_eq!(
            operation.usage(root).unwrap().work(HtmlWork::ProviderCall),
            0
        );
    }

    #[test]
    fn a_layout_charges_its_own_and_its_closing_credits_as_one_provider_call_change() {
        let layout = HtmlCspLayout::processing(
            37,
            HtmlCspStorage::try_new(64, 8, 2).unwrap(),
            HtmlCspDisposalWork::try_new(5, 16).unwrap(),
            32,
        )
        .unwrap();
        let mut operation = ledger();
        let root = operation.root();
        layout.charge_work(&mut operation.charge_at(root)).unwrap();
        let usage = *operation.usage(root).unwrap();
        assert_eq!(usage.work(HtmlWork::ProviderCall), 42);
        for dimension in [
            HtmlWork::RecordVisit,
            HtmlWork::InspectByte,
            HtmlWork::CopyByte,
            HtmlWork::RecordWrite,
            HtmlWork::ReserveByte,
        ] {
            assert_eq!(usage.work(dimension), 0, "{dimension:?}");
        }
    }

    #[test]
    fn disposal_credits_charge_with_the_callers_own_calls_in_one_change() {
        let disposal = HtmlCspDisposalWork::try_new(5, 16).unwrap();
        let mut operation = ledger();
        let root = operation.root();
        disposal
            .charge_with_calls(1, &mut operation.charge_at(root))
            .unwrap();
        assert_eq!(
            operation.usage(root).unwrap().work(HtmlWork::ProviderCall),
            6
        );
        assert_eq!(
            HtmlCspDisposalWork::try_new(u64::MAX >> 1, 0)
                .unwrap()
                .charge_with_calls(u64::MAX, &mut operation.charge_at(root)),
            Err(HtmlChargedError::Faulted(HtmlChargeOverflow))
        );
    }
}
