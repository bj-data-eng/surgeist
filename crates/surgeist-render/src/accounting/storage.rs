use super::{
    Account, CpuAccount, CpuHold, CpuImport, PreparationHold, RenderAccountingError, RenderResource,
};
#[cfg(test)]
use crate::GlyphPreparationError;
use crate::Result;
use std::ops::{Deref, DerefMut};

mod packed;
pub(crate) use packed::PackedProducer;
#[cfg(test)]
pub(crate) use packed::packed_size;

pub(crate) struct StorageSizing {
    storage: StorageAccount,
    resource: RenderResource,
}
impl StorageSizing {
    pub(crate) fn new(account: &Account, resource: RenderResource) -> Self {
        Self {
            storage: account.storage(),
            resource,
        }
    }
    pub(crate) fn add(&self, a: usize, b: usize) -> Result<usize> {
        a.checked_add(b)
            .ok_or_else(|| self.storage.capacity_error(self.resource))
    }
    pub(crate) fn mul(&self, a: usize, b: usize) -> Result<usize> {
        a.checked_mul(b)
            .ok_or_else(|| self.storage.capacity_error(self.resource))
    }
}

/// A Render consumer violated its owned-vector contract. These are not defects
/// fabricated on behalf of Ledger; provider failures retain their own identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StorageContractError {
    MissingBacking,
    ReservedCapacity {
        needed: usize,
        capacity: usize,
    },
    InvalidInsertion {
        index: usize,
        len: usize,
    },
    InvalidPosition {
        index: usize,
        len: usize,
    },
    BackingCapacityMismatch {
        admitted_bytes: u64,
        actual_bytes: u64,
    },
    WrongRetentionLifetime,
    BackingResourceMismatch {
        admitted: RenderResource,
        requested: RenderResource,
    },
    ProducerCapacityChanged {
        expected: usize,
        actual: usize,
    },
    ProducerLengthChanged {
        expected: usize,
        actual: usize,
    },
}
impl std::fmt::Display for StorageContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "owned-vector consumer contract: {self:?}")
    }
}
impl std::error::Error for StorageContractError {}
impl From<StorageContractError> for crate::Error {
    fn from(value: StorageContractError) -> Self {
        Self::new(
            crate::BackendErrorCode::RenderFailed,
            "owned-vector storage contract failed",
        )
        .with_source(value)
    }
}

/// Ends lexical disposal after the actual payload and both backing locals.
#[derive(Default)]
pub(crate) struct DisposalCompletion(Option<super::cpu::MaintenancePhase>);

impl DisposalCompletion {
    /// Select the first actual backing, then the second, then import-only cleanup.
    /// The returned guard borrows only this completion slot, never the payload.
    pub(crate) fn start(
        &mut self,
        backing: [Option<&Hold>; 2],
        import: Option<&CpuImport>,
    ) -> DisposalStart<'_> {
        let account = match backing.into_iter().flatten().next() {
            Some(Hold::Renderer(hold)) => Some(hold.account()),
            Some(Hold::Preparation(_)) => None,
            None => import.map(CpuImport::disposal_account),
        };
        DisposalStart {
            completion: self,
            account,
        }
    }
}

/// Declared after a mandatory lexical payload; starts observation before its drop.
pub(crate) struct DisposalStart<'a> {
    completion: &'a mut DisposalCompletion,
    account: Option<CpuAccount>,
}
impl DisposalStart<'_> {
    /// Only infallible moves into the receiving armed owner may follow.
    pub(crate) fn disarm(mut self) {
        self.account = None;
    }
}
impl Drop for DisposalStart<'_> {
    fn drop(&mut self) {
        if let Some(account) = self.account.take() {
            self.completion.0 = Some(account.begin_maintenance_phase());
        }
    }
}

/// A complete standalone result with preparation-only custody.
/// Automatic field order destroys the mandatory value before both actual leases.
pub(crate) struct PreparationValue<T> {
    value: T,
    backing: [Option<PreparationHold>; 2],
}
impl<T> PreparationValue<T> {
    pub(crate) fn new(value: T, backing: [Option<PreparationHold>; 2]) -> Self {
        Self { value, backing }
    }
    pub(crate) fn into_retained(self) -> T {
        let Self { value, backing } = self;
        for hold in backing.into_iter().flatten() {
            hold.retain();
        }
        value
    }
}
impl<T> Deref for PreparationValue<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

/// Narrow optional allocation custody without releasing a rejected renderer lease.
/// The caller keeps its complete lexical payload armed throughout this check.
pub(crate) fn preparation_backing(backing: &mut Option<Hold>) -> Result<Option<PreparationHold>> {
    match backing.take() {
        Some(Hold::Preparation(hold)) => Ok(Some(hold)),
        Some(hold @ Hold::Renderer(_)) => {
            *backing = Some(hold);
            Err(StorageContractError::WrongRetentionLifetime.into())
        }
        None => Ok(None),
    }
}

/// One explicit fixed-size heap payload. The value is disposed before its
/// admission; nested values retain their independent backing guards.
pub(crate) struct OwnedBox<T> {
    // Rust destroys fields in order, including during payload unwind.
    value: Box<T>,
    allocation: AllocationCleanup,
}
impl<T> OwnedBox<T> {
    pub(crate) fn new(value: T, account: &StorageAccount) -> Result<Self> {
        account.phase(|| {
            let hold =
                account.acquire(RenderResource::RetainedCpuBytes, std::mem::size_of::<T>())?;
            Ok(Self {
                value: Box::new(value),
                allocation: AllocationCleanup {
                    hold,
                    completion: None,
                },
            })
        })
    }
}
impl<T> Drop for OwnedBox<T> {
    fn drop(&mut self) {
        self.allocation.begin_cleanup();
    }
}
impl<T> Deref for OwnedBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T> DerefMut for OwnedBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for OwnedBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T: PartialEq> PartialEq for OwnedBox<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
impl<T: Eq> Eq for OwnedBox<T> {}

/// Immutable sharing of one actual owned payload allocation. Cloning shares
/// exactly that allocation and its backing/import custody; it never clones T.
pub(crate) struct SharedValue<T>(std::sync::Arc<SharedValueData<T>>);
struct SharedValueData<T> {
    // Automatic field destruction preserves actual payload, direct backing,
    // allocation release/phase completion, then separate import cleanup.
    value: T,
    _backing: Option<Hold>,
    _allocation: AllocationCleanup,
    _import: Option<CpuImport>,
}
struct AllocationCleanup {
    // This field releases before the trailing completion marker ends the phase.
    hold: Hold,
    completion: Option<super::cpu::MaintenancePhase>,
}
impl AllocationCleanup {
    fn begin_cleanup(&mut self) {
        if let Hold::Renderer(hold) = &self.hold {
            self.completion = Some(hold.account().begin_maintenance_phase());
        }
    }
}
impl<T: AsRef<[u8]>> AsRef<[u8]> for SharedValueData<T> {
    fn as_ref(&self) -> &[u8] {
        self.value.as_ref()
    }
}
impl<T> SharedValue<T> {
    pub(crate) fn new(value: T, account: &StorageAccount) -> Result<Self> {
        Self::with_parts(value, None, None, account)
    }
    pub(crate) fn with_parts(
        value: T,
        backing: Option<Hold>,
        import: Option<CpuImport>,
        account: &StorageAccount,
    ) -> Result<Self> {
        // Bind incoming arguments in disposal order before any rejectable entry.
        let import = import;
        let mut completion = DisposalCompletion::default();
        let backing = backing;
        let value = value;
        let start = completion.start([backing.as_ref(), None], import.as_ref());
        // This later local closes forward observation before failure disposal.
        let _phase = account.begin_phase()?;
        let allocation = account.acquire(
            RenderResource::RetainedCpuBytes,
            std::mem::size_of::<SharedValueData<T>>(),
        )?;
        start.disarm();
        // Stable safe allocation follows the existing admitted payload boundary.
        Ok(Self(std::sync::Arc::new(SharedValueData {
            value,
            _backing: backing,
            _allocation: AllocationCleanup {
                hold: allocation,
                completion: None,
            },
            _import: import,
        })))
    }

    /// Identifies the same immutable allocation without inspecting its content.
    /// Ordinary `PartialEq` remains content equality.
    pub(crate) fn same_allocation(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
    #[cfg(test)]
    pub(crate) fn allocation_liveness_for_test(&self) -> impl Fn() -> bool + use<T> {
        let weak = std::sync::Arc::downgrade(&self.0);
        move || weak.strong_count() != 0
    }
    #[cfg(test)]
    pub(crate) const fn allocation_size_for_test() -> usize {
        std::mem::size_of::<SharedValueData<T>>()
    }
    pub(crate) fn from_accounted(
        values: AccountedVec<T>,
        account: &StorageAccount,
    ) -> Result<SharedValue<Vec<T>>> {
        let (values, hold) = values.into_parts()?;
        SharedValue::with_parts(values, Some(hold), None, account)
    }
    /// Moves the same counted payload into a dependency's immutable byte handle.
    /// The trait-object coercion allocates no additional owner or backing.
    pub(crate) fn into_bytes_arc(self) -> std::sync::Arc<dyn AsRef<[u8]> + Send + Sync>
    where
        T: AsRef<[u8]> + Send + Sync + 'static,
    {
        self.0
    }
}
impl<T> Drop for SharedValueData<T> {
    fn drop(&mut self) {
        // Start before Rust destroys the actual T and its backing. Preparation
        // allocation custody leaves the inline completion component dormant.
        self._allocation.begin_cleanup();
    }
}
impl<T> Clone for SharedValue<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
#[cfg(test)]
impl<T: Clone> SharedValue<Vec<T>> {
    pub(crate) fn edit_for_test(
        &mut self,
        additional: usize,
        account: &StorageAccount,
    ) -> Result<SharedVecFixtureEdit<'_, T>> {
        self.try_edit_for_test(additional, account, |_| {})?;
        let data = std::sync::Arc::get_mut(&mut self.0).expect("private fixture copy is unique");
        let values = &mut data.value;
        let capacity = values.capacity();
        Ok(SharedVecFixtureEdit { values, capacity })
    }
    /// The malformed-graph fixture harness edits a private admitted copy. The
    /// declared spare capacity bounds every fixture insertion; production has no
    /// mutable Vec access through this immutable shared owner.
    pub(crate) fn try_edit_for_test(
        &mut self,
        additional: usize,
        account: &StorageAccount,
        edit: impl FnOnce(&mut Vec<T>),
    ) -> Result<()> {
        let count = self
            .len()
            .checked_add(additional)
            .ok_or_else(|| account.capacity_error(RenderResource::RetainedCpuBytes))?;
        let mut replacement =
            AccountedVec::new_in(account, count, RenderResource::RetainedCpuBytes)?;
        replacement.extend_from_slice(self.as_slice())?;
        let (mut values, hold) = replacement.into_parts()?;
        let capacity = values.capacity();
        edit(&mut values);
        if values.capacity() != capacity {
            return Err(RenderAccountingError::Invariant.into());
        }
        *self = SharedValue::with_parts(values, Some(hold), None, account)?;
        Ok(())
    }
}
#[cfg(test)]
pub(crate) struct SharedVecFixtureEdit<'a, T> {
    values: &'a mut Vec<T>,
    capacity: usize,
}
#[cfg(test)]
impl<T> Deref for SharedVecFixtureEdit<'_, T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        self.values
    }
}
#[cfg(test)]
impl<T> DerefMut for SharedVecFixtureEdit<'_, T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        self.values
    }
}
#[cfg(test)]
impl<T> Drop for SharedVecFixtureEdit<'_, T> {
    fn drop(&mut self) {
        assert_eq!(
            self.values.capacity(),
            self.capacity,
            "fixture edit exceeded its admitted capacity"
        );
    }
}
impl<T> Deref for SharedValue<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0.value
    }
}
impl<'a, T> IntoIterator for &'a SharedValue<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&self.0.value).into_iter()
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for SharedValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.value.fmt(f)
    }
}
impl<T: PartialEq> PartialEq for SharedValue<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.value == other.0.value
    }
}
impl<T: Eq> Eq for SharedValue<T> {}
impl<T: std::hash::Hash> std::hash::Hash for SharedValue<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.value.hash(state);
    }
}

/// The concrete backing authority. Renderer storage retains lifetime custody,
/// never the ephemeral forward participant.
#[derive(Clone)]
pub(crate) enum StorageAccount {
    Preparation(Account),
    Renderer(CpuAccount),
}
impl StorageAccount {
    pub(crate) fn capacity_error(&self, resource: RenderResource) -> crate::Error {
        match self {
            Self::Preparation(account) => account.capacity_error(resource),
            Self::Renderer(account) => account.capacity_error(resource),
        }
    }
    pub(crate) fn bytes<T>(&self, count: usize, resource: RenderResource) -> Result<usize> {
        match self {
            Self::Preparation(account) => count
                .checked_mul(std::mem::size_of::<T>())
                .ok_or_else(|| account.capacity_error(resource)),
            Self::Renderer(account) => account.sized::<T>(count, resource),
        }
    }
    fn begin_phase(&self) -> Result<Option<super::cpu::MaintenancePhase>> {
        match self {
            Self::Preparation(_) => Ok(None),
            Self::Renderer(account) => account.begin_checked_phase().map(Some),
        }
    }
    pub(crate) fn phase<T>(&self, run: impl FnOnce() -> Result<T>) -> Result<T> {
        match self {
            Self::Preparation(_) => run(),
            Self::Renderer(account) => account.phase(run),
        }
    }
    pub(crate) fn preview(&self, resource: RenderResource, amount: usize) -> Result<()> {
        match self {
            Self::Preparation(account) => account.preview(resource, amount),
            Self::Renderer(account) => account.preview(resource, amount),
        }
    }
    pub(crate) fn acquire(&self, resource: RenderResource, amount: usize) -> Result<Hold> {
        match self {
            Self::Preparation(account) => account.acquire(resource, amount),
            Self::Renderer(account) => account.acquire(resource, amount).map(Hold::Renderer),
        }
    }
    fn relocate(&self, count: usize, resource: RenderResource) -> Result<()> {
        if count == 0 {
            return Ok(());
        }
        match self {
            Self::Preparation(account) => {
                account.work(super::RenderWork::StorageRelocations, count)
            }
            Self::Renderer(account) => {
                let amount = u64::try_from(count).map_err(|_| self.capacity_error(resource))?;
                account.checked_work(super::RenderWork::StorageRelocations, amount)
            }
        }
    }
    pub(crate) fn allocation_error(&self, resource: RenderResource) -> crate::Error {
        match self {
            Self::Preparation(account) => account.allocation_error(resource),
            Self::Renderer(account) => account.allocation_error(resource),
        }
    }
}

pub(crate) enum Hold {
    Preparation(PreparationHold),
    Renderer(CpuHold),
}
impl std::fmt::Debug for Hold {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hold")
            .field("resource", &self.resource())
            .field("amount", &self.amount())
            .finish_non_exhaustive()
    }
}
impl Hold {
    pub(crate) fn storage(&self) -> StorageAccount {
        match self {
            Self::Preparation(hold) => StorageAccount::Preparation(hold.account.clone()),
            Self::Renderer(hold) => StorageAccount::Renderer(hold.account()),
        }
    }
    pub(crate) fn resource(&self) -> RenderResource {
        match self {
            Self::Preparation(hold) => hold.resource,
            Self::Renderer(hold) => hold.resource(),
        }
    }
    fn amount(&self) -> Result<u64> {
        match self {
            Self::Preparation(hold) => Ok(hold.amount),
            Self::Renderer(hold) => hold
                .active_amount()
                .ok_or_else(|| StorageContractError::MissingBacking.into()),
        }
    }
    pub(crate) fn dispose<T>(self, run: impl FnOnce() -> T) -> T {
        match self {
            Self::Preparation(hold) => {
                let value = run();
                drop(hold);
                value
            }
            Self::Renderer(hold) => {
                let account = hold.account();
                account.maintenance_phase(|| {
                    let value = run();
                    drop(hold);
                    value
                })
            }
        }
    }
    /// Narrows a real standalone preparation lease before plain-result transfer.
    pub(crate) fn into_preparation(self) -> Result<PreparationHold> {
        match self {
            Self::Preparation(hold) => Ok(hold),
            Self::Renderer(_) => Err(StorageContractError::WrongRetentionLifetime.into()),
        }
    }
    /// Transfers only a complete explicit-readback output after report closure.
    pub(crate) fn transfer_readback(&mut self) -> Result<()> {
        match self {
            Self::Renderer(hold) => hold.transfer_readback(),
            Self::Preparation(_) => Err(RenderAccountingError::Invariant.into()),
        }
    }
}

/// Admits actual replacement capacity before the consumer admits the exact
/// element-movement prefix. Refusal preserves the old values and admission.
/// The producer proves sufficient capacity before lending a Vec to dependency
/// code; an already unguarded allocation is never admitted retroactively.
pub(crate) fn reserve_backing_with<T>(
    values: &mut Vec<T>,
    hold: &mut Option<Hold>,
    needed: usize,
    account: &StorageAccount,
    resource: RenderResource,
    before_move: impl FnOnce(usize) -> Result<()>,
) -> Result<()> {
    account.phase(|| reserve_backing_inner(values, hold, needed, account, resource, before_move))
}
fn reserve_backing_inner<T>(
    values: &mut Vec<T>,
    hold: &mut Option<Hold>,
    needed: usize,
    account: &StorageAccount,
    resource: RenderResource,
    before_move: impl FnOnce(usize) -> Result<()>,
) -> Result<()> {
    let current = account.bytes::<T>(values.capacity(), resource)?;
    let current = u64::try_from(current).map_err(|_| account.capacity_error(resource))?;
    match hold {
        Some(hold) => {
            if hold.resource() != resource {
                return Err(StorageContractError::BackingResourceMismatch {
                    admitted: hold.resource(),
                    requested: resource,
                }
                .into());
            }
            let admitted_bytes = hold.amount()?;
            if admitted_bytes != current {
                return Err(StorageContractError::BackingCapacityMismatch {
                    admitted_bytes,
                    actual_bytes: current,
                }
                .into());
            }
        }
        None if current != 0 => return Err(StorageContractError::MissingBacking.into()),
        None => (),
    }
    if needed <= values.capacity() {
        return Ok(());
    }
    account.preview(resource, account.bytes::<T>(needed, resource)?)?;
    let mut replacement = Vec::new();
    replacement
        .try_reserve_exact(needed)
        .map_err(|_| account.allocation_error(resource))?;
    let replacement_hold = account.acquire(
        resource,
        account.bytes::<T>(replacement.capacity(), resource)?,
    )?;
    if let Err(error) = before_move(values.len()) {
        replacement_hold.dispose(|| drop(replacement));
        return Err(error);
    }
    if let Err(error) = account.relocate(values.len(), resource) {
        replacement_hold.dispose(|| drop(replacement));
        return Err(error);
    }
    replacement.append(values);
    std::mem::swap(values, &mut replacement);
    // Actual old-backing disposal and release share the same lifetime phase.
    if let Some(old) = hold.replace(replacement_hold) {
        old.dispose(|| drop(replacement));
    } else {
        drop(replacement);
    }
    Ok(())
}

/// Private explicit backing owner. Growth allocates a replacement so both
/// actual capacities remain attributable until the old backing is dropped.
pub(crate) struct AccountedVec<T> {
    values: Vec<T>,
    hold: Option<Hold>,
}
pub(crate) struct AppendSlot<'a, T> {
    values: &'a mut Vec<T>,
}
impl<T> AppendSlot<'_, T> {
    pub(crate) fn write(self, value: T) {
        self.values.push(value);
    }
}
pub(crate) struct InsertSlot<'a, T> {
    values: &'a mut Vec<T>,
    index: usize,
    movements: usize,
}
impl<T> InsertSlot<'_, T> {
    pub(crate) fn movement_count(&self) -> usize {
        self.movements
    }
    pub(crate) fn write(self, value: T) {
        self.values.insert(self.index, value);
    }
}
pub(crate) struct CopySlot<'a, 'b, T> {
    values: &'a mut Vec<T>,
    source: &'b [T],
}
impl<T: Copy> CopySlot<'_, '_, T> {
    pub(crate) fn element_count(&self) -> usize {
        self.source.len()
    }
    pub(crate) fn write(self) {
        self.values.extend_from_slice(self.source);
    }
}
/// A standalone vector result whose lease cannot belong to a renderer lifetime.
/// No custom destructor is needed: values drop before preparation custody.
pub(crate) struct PreparationVec<T> {
    values: Vec<T>,
    hold: PreparationHold,
}
impl<T> PreparationVec<T> {
    pub(crate) fn into_parts(self) -> (Vec<T>, PreparationHold) {
        (self.values, self.hold)
    }
    pub(crate) fn into_retained(self) -> Vec<T> {
        let (values, hold) = self.into_parts();
        hold.retain();
        values
    }
}

impl<T> AccountedVec<T> {
    pub(crate) fn new(account: &Account, count: usize, resource: RenderResource) -> Result<Self> {
        Self::new_in(&account.storage(), count, resource)
    }
    pub(crate) fn new_in(
        account: &StorageAccount,
        count: usize,
        resource: RenderResource,
    ) -> Result<Self> {
        account.phase(|| Self::allocate(account, count, resource))
    }
    fn allocate(account: &StorageAccount, count: usize, resource: RenderResource) -> Result<Self> {
        account.preview(resource, account.bytes::<T>(count, resource)?)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| account.allocation_error(resource))?;
        let hold = account.acquire(resource, account.bytes::<T>(values.capacity(), resource)?)?;
        Ok(Self {
            values,
            hold: Some(hold),
        })
    }
    pub(crate) fn grow(&mut self, additional: usize) -> Result<()> {
        self.grow_with(additional, |_| Ok(()))
    }
    /// Admits a replacement first, then lets the consumer admit real element
    /// movement before the nonfallible move. A failed callback preserves values.
    pub(crate) fn grow_with(
        &mut self,
        additional: usize,
        before_move: impl FnOnce(usize) -> Result<()>,
    ) -> Result<()> {
        let hold = self.backing()?;
        let storage = hold.storage();
        let resource = hold.resource();
        let count = self
            .values
            .len()
            .checked_add(additional)
            .ok_or_else(|| storage.capacity_error(resource))?;
        if count <= self.capacity() {
            return Ok(());
        }
        storage.phase(|| {
            let mut next = Self::allocate(&storage, count, resource)?;
            before_move(self.values.len())?;
            // Capacity/overlap, then the caller's admission callback, then
            // central relocation admission all precede the physical move.
            storage.relocate(self.values.len(), resource)?;
            next.values.append(&mut self.values);
            std::mem::swap(self, &mut next);
            drop(next);
            Ok(())
        })
    }
    pub(crate) fn capacity(&self) -> usize {
        self.values.capacity()
    }
    /// Disposal/read-receipt work belongs to the persistent CPU lifetime and
    /// cannot spend a frame's forward limits. Standalone preparation has no
    /// separate renderer lifecycle participant.
    pub(crate) fn renderer_maintenance(&self, work: super::RenderWork, amount: u64) {
        if let Some(Hold::Renderer(hold)) = &self.hold {
            hold.account().maintenance(work, amount);
        }
    }
    pub(crate) fn as_slice(&self) -> &[T] {
        self.values.as_slice()
    }
    pub(crate) fn push(&mut self, value: T) -> Result<()> {
        self.grow(1)?;
        self.values.push(value);
        Ok(())
    }
    fn backing(&self) -> Result<&Hold> {
        let hold = self
            .hold
            .as_ref()
            .ok_or(StorageContractError::MissingBacking)?;
        let storage = hold.storage();
        let resource = hold.resource();
        let actual_bytes = storage.bytes::<T>(self.values.capacity(), resource)?;
        let actual_bytes =
            u64::try_from(actual_bytes).map_err(|_| storage.capacity_error(resource))?;
        let admitted_bytes = hold.amount()?;
        if admitted_bytes != actual_bytes {
            return Err(StorageContractError::BackingCapacityMismatch {
                admitted_bytes,
                actual_bytes,
            }
            .into());
        }
        Ok(hold)
    }
    fn checked_needed(&self, additional: usize) -> Result<usize> {
        let hold = self.backing()?;
        let needed = self
            .values
            .len()
            .checked_add(additional)
            .ok_or_else(|| hold.storage().capacity_error(hold.resource()))?;
        if needed > self.values.capacity() {
            return Err(StorageContractError::ReservedCapacity {
                needed,
                capacity: self.values.capacity(),
            }
            .into());
        }
        Ok(needed)
    }
    /// A single checked, exclusive append capability. Growth remains explicit.
    pub(crate) fn append_slot(&mut self) -> Result<AppendSlot<'_, T>> {
        self.checked_needed(1)?;
        Ok(AppendSlot {
            values: &mut self.values,
        })
    }
    /// Bind both insertion position and spare capacity until the one write.
    pub(crate) fn insert_slot(&mut self, index: usize) -> Result<InsertSlot<'_, T>> {
        let len = self.values.len();
        if index > len {
            return Err(StorageContractError::InvalidInsertion { index, len }.into());
        }
        self.checked_needed(1)?;
        let movements = len
            .checked_sub(index)
            .and_then(|n| n.checked_add(1))
            .ok_or(StorageContractError::InvalidInsertion { index, len })?;
        Ok(InsertSlot {
            values: &mut self.values,
            index,
            movements,
        })
    }
    /// Bind the actual immutable copy source, not a separately claimed length.
    pub(crate) fn copy_slot<'a, 'b>(&'a mut self, source: &'b [T]) -> Result<CopySlot<'a, 'b, T>>
    where
        T: Copy,
    {
        self.checked_needed(source.len())?;
        Ok(CopySlot {
            values: &mut self.values,
            source,
        })
    }
    /// The locked native producer temporarily borrows this intact owner. A
    /// callback's original failure takes priority; successful calls must retain
    /// admitted capacity and produce exactly their qualified serialized length.
    fn with_producer<R>(
        &mut self,
        expected_len: usize,
        producer: impl FnOnce(&mut Vec<T>) -> Result<R>,
    ) -> Result<R> {
        self.backing()?;
        let capacity = self.values.capacity();
        if expected_len > capacity {
            return Err(StorageContractError::ReservedCapacity {
                needed: expected_len,
                capacity,
            }
            .into());
        }
        let result = producer(&mut self.values)?;
        if self.values.capacity() != capacity {
            return Err(StorageContractError::ProducerCapacityChanged {
                expected: capacity,
                actual: self.values.capacity(),
            }
            .into());
        }
        if self.values.len() != expected_len {
            return Err(StorageContractError::ProducerLengthChanged {
                expected: expected_len,
                actual: self.values.len(),
            }
            .into());
        }
        Ok(result)
    }
    pub(crate) fn extend_from_slice(&mut self, values: &[T]) -> Result<()>
    where
        T: Clone,
    {
        self.grow(values.len())?;
        self.values.extend_from_slice(values);
        Ok(())
    }
    pub(crate) fn resize(&mut self, len: usize, value: T) -> Result<()>
    where
        T: Clone,
    {
        self.grow(len.saturating_sub(self.values.len()))?;
        self.values.resize(len, value);
        Ok(())
    }
    pub(crate) fn pop(&mut self) -> Option<T> {
        self.values.pop()
    }
    pub(crate) fn clear(&mut self) {
        self.values.clear();
    }
    pub(crate) fn truncate(&mut self, len: usize) {
        self.values.truncate(len);
    }
    /// Qualifies actual storage before an owning consumer enters its CPU gate.
    pub(crate) fn qualified_storage(&self) -> Result<StorageAccount> {
        Ok(self.backing()?.storage())
    }
    /// Lending transfers the backing and its exact admission together.
    pub(crate) fn into_parts(mut self) -> Result<(Vec<T>, Hold)> {
        self.backing()?;
        let hold = self
            .hold
            .take()
            .ok_or(StorageContractError::MissingBacking)?;
        Ok((std::mem::take(&mut self.values), hold))
    }
    /// Rejoins a producer backing which was proved not to grow while lent.
    pub(crate) fn from_parts(values: Vec<T>, hold: Hold) -> Result<Self> {
        let account = hold.storage();
        let resource = hold.resource();
        let expected = account
            .bytes::<T>(values.capacity(), resource)
            .and_then(|amount| u64::try_from(amount).map_err(|_| account.capacity_error(resource)));
        let checked = expected.and_then(|actual_bytes| {
            let admitted_bytes = hold.amount()?;
            if admitted_bytes == actual_bytes {
                Ok(())
            } else {
                Err(StorageContractError::BackingCapacityMismatch {
                    admitted_bytes,
                    actual_bytes,
                }
                .into())
            }
        });
        match checked {
            Ok(()) => Ok(Self {
                values,
                hold: Some(hold),
            }),
            Err(error) => {
                hold.dispose(|| drop(values));
                Err(error)
            }
        }
    }
    /// Reject a renderer lease while this complete payload/credit owner is armed.
    pub(crate) fn into_preparation(mut self) -> Result<PreparationVec<T>> {
        self.backing()?;
        match self.hold.take() {
            Some(Hold::Preparation(hold)) => Ok(PreparationVec {
                values: std::mem::take(&mut self.values),
                hold,
            }),
            Some(hold @ Hold::Renderer(_)) => {
                self.hold = Some(hold);
                Err(StorageContractError::WrongRetentionLifetime.into())
            }
            None => Err(StorageContractError::MissingBacking.into()),
        }
    }
}
impl AccountedVec<u8> {
    /// Final checked handoff; failure keeps the complete vector armed.
    pub(crate) fn transfer_readback(mut self) -> Result<Vec<u8>> {
        self.backing()?;
        self.hold
            .as_mut()
            .ok_or(StorageContractError::MissingBacking)?
            .transfer_readback()?;
        // Remove inactive custody before the empty residual owner is destroyed.
        drop(self.hold.take());
        Ok(std::mem::take(&mut self.values))
    }
}
impl<T> Drop for AccountedVec<T> {
    fn drop(&mut self) {
        if let Some(hold) = self.hold.take() {
            let values = std::mem::take(&mut self.values);
            hold.dispose(|| drop(values));
        }
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for AccountedVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.values.fmt(f)
    }
}
impl<T: PartialEq> PartialEq for AccountedVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl<T: Eq> Eq for AccountedVec<T> {}
pub(crate) struct AccountedIntoIter<T> {
    values: std::vec::IntoIter<T>,
    hold: Option<Hold>,
}
impl<T> Iterator for AccountedIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.values.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.values.size_hint()
    }
}
impl<T> ExactSizeIterator for AccountedIntoIter<T> {}
impl<T> DoubleEndedIterator for AccountedIntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.values.next_back()
    }
}
impl<T> std::iter::FusedIterator for AccountedIntoIter<T> {}
impl<T> Drop for AccountedIntoIter<T> {
    fn drop(&mut self) {
        if let Some(hold) = self.hold.take() {
            let values = std::mem::take(&mut self.values);
            hold.dispose(|| drop(values));
        }
    }
}
impl<T> IntoIterator for AccountedVec<T> {
    type Item = T;
    type IntoIter = AccountedIntoIter<T>;
    fn into_iter(mut self) -> Self::IntoIter {
        AccountedIntoIter {
            values: std::mem::take(&mut self.values).into_iter(),
            hold: self.hold.take(),
        }
    }
}
impl<T> Deref for AccountedVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}
impl<T> DerefMut for AccountedVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.values
    }
}
impl<'a, T> IntoIterator for &'a AccountedVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::{RenderAccountingOptions, RenderCpuScope, RendererCpu};

    #[test]
    fn shared_value_clones_share_one_fixed_payload_and_vec_backing_until_final_drop() {
        let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let baseline = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let mut values =
            AccountedVec::new_in(&storage, 2, RenderResource::RetainedCpuBytes).unwrap();
        values.extend_from_slice(&[10u32, 20]).unwrap();
        let expected = baseline
            + (values.capacity() * 4) as u64
            + std::mem::size_of::<SharedValueData<Vec<u32>>>() as u64;
        let shared = SharedValue::from_accounted(values, &storage).unwrap();
        let clone = shared.clone();
        assert_eq!(&**clone, &[10, 20]);
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            expected
        );
        drop(shared);
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            expected
        );
        drop(clone);
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            baseline
        );
    }

    #[test]
    fn boxed_payload_projects_the_actual_value_and_releases_after_disposal() {
        use crate::accounting::{RenderClock, RenderProfile};
        use std::sync::{
            Arc,
            atomic::{AtomicU64, AtomicUsize, Ordering},
        };
        use std::time::Duration;
        static TIME: AtomicU64 = AtomicU64::new(0);
        fn now() -> Duration {
            Duration::from_millis(TIME.load(Ordering::SeqCst))
        }
        fn forbidden_clock() -> Duration {
            panic!("untimed boxed payload read the clock")
        }
        struct Probe {
            cpu: RendererCpu,
            drops: Arc<AtomicUsize>,
            live: Arc<AtomicU64>,
            unwind: bool,
        }
        impl Drop for Probe {
            fn drop(&mut self) {
                self.live.store(
                    self.cpu
                        .snapshot()
                        .unwrap()
                        .usage()
                        .live(RenderResource::RetainedCpuBytes),
                    Ordering::SeqCst,
                );
                self.drops.fetch_add(1, Ordering::SeqCst);
                TIME.fetch_add(7, Ordering::SeqCst);
                assert!(!self.unwind, "boxed payload destructor unwind");
            }
        }
        for profile in [
            RenderProfile::Off,
            RenderProfile::Summary { timing: false },
            RenderProfile::Summary { timing: true },
        ] {
            let timed = matches!(profile, RenderProfile::Summary { timing: true });
            for exit in 0..3 {
                let unwind = exit == 2;
                TIME.store(0, Ordering::SeqCst);
                let cpu = RendererCpu::new(
                    RenderAccountingOptions::new()
                        .with_profile(profile)
                        .with_clock(RenderClock::Host(if timed { now } else { forbidden_clock })),
                )
                .unwrap();
                let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
                let mut value = OwnedBox::new(73u32, &storage).unwrap();
                assert_eq!(*value, 73);
                *value = 91;
                let equal = OwnedBox::new(91u32, &storage).unwrap();
                assert_eq!(value, equal);
                assert_eq!(format!("{value:?}"), "91");
                drop((value, equal));
                let drops = Arc::new(AtomicUsize::new(0));
                let live = Arc::new(AtomicU64::new(0));
                let baseline = cpu
                    .snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes);
                let value = OwnedBox::new(
                    Probe {
                        cpu: cpu.clone(),
                        drops: drops.clone(),
                        live: live.clone(),
                        unwind,
                    },
                    &storage,
                )
                .unwrap();
                let interval = cpu.begin_interval().unwrap();
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _value = value;
                    if exit == 1 {
                        Err(crate::Error::new(
                            crate::BackendErrorCode::RenderFailed,
                            "boxed owner caller failed",
                        ))
                    } else {
                        Ok(())
                    }
                }));
                assert_eq!(result.is_err(), unwind);
                if !unwind {
                    assert_eq!(result.unwrap().is_err(), exit == 1);
                }
                assert_eq!(drops.load(Ordering::SeqCst), 1);
                assert_eq!(
                    live.load(Ordering::SeqCst),
                    std::mem::size_of::<Probe>() as u64 + baseline
                );
                let interval = interval.finish().unwrap();
                assert_eq!(
                    interval.scope_elapsed(RenderCpuScope::FrameBacking),
                    timed.then_some(Duration::from_millis(7))
                );
                assert_eq!(
                    interval.usage().live(RenderResource::RetainedCpuBytes),
                    baseline
                );
                assert_eq!(cpu.snapshot().unwrap().fault(), None);
            }
        }
    }

    #[test]
    fn boxed_constructor_refusal_destroys_incoming_value_once_and_preserves_cause() {
        use crate::accounting::RenderLimits;
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        struct Probe(Arc<AtomicUsize>);
        impl Drop for Probe {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let baseline = RendererCpu::new(RenderAccountingOptions::new())
            .unwrap()
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        for entry_failure in [false, true] {
            let cpu = RendererCpu::new(
                RenderAccountingOptions::new()
                    .with_limits(RenderLimits::new().with_retained_bytes(baseline)),
            )
            .unwrap();
            let account = cpu.at(RenderCpuScope::FrameBacking);
            if entry_failure {
                account.note_fault(RenderAccountingError::Invariant);
            }
            let drops = Arc::new(AtomicUsize::new(0));
            let error = OwnedBox::new(Probe(drops.clone()), &StorageAccount::Renderer(account))
                .err()
                .unwrap();
            assert_eq!(drops.load(Ordering::SeqCst), 1);
            if entry_failure {
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::Invariant)
                );
            } else {
                assert!(
                    matches!(error.accounting_diagnostic(), Some(RenderAccountingError::CpuRefused { current, amount, .. }) if current == baseline && amount == std::mem::size_of::<Probe>() as u64)
                );
            }
            assert_eq!(
                cpu.snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes),
                baseline
            );
        }
    }

    #[test]
    fn boxed_payload_retains_fixed_allocation_until_its_owner_drops() {
        eprintln!(
            "boxed native layout: wrapper={}, former optional-box/hold fields={}, payload u32={}, exact graph={}, semantic graph={}, FramePlan={}, VelloScene={}",
            std::mem::size_of::<OwnedBox<u32>>(),
            std::mem::size_of::<(Option<Box<u32>>, Option<Hold>)>(),
            std::mem::size_of::<u32>(),
            std::mem::size_of::<crate::backend::RendererExactSurfaceGraph>(),
            std::mem::size_of::<crate::frame::GpuRenderGraph>(),
            std::mem::size_of::<crate::frame::FramePlan>(),
            std::mem::size_of::<crate::vello_engine::scene::VelloScene>(),
        );
        let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let baseline = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let value = OwnedBox::new(73u32, &storage).unwrap();
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            baseline + 4
        );
        assert_eq!(*value, 73);
        drop(value);
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            baseline
        );
    }

    #[test]
    fn backing_phase_includes_the_real_value_destructor() {
        use crate::accounting::{RenderClock, RenderProfile};
        use std::sync::atomic::{AtomicU64, Ordering};
        static TIME: AtomicU64 = AtomicU64::new(0);
        fn now() -> std::time::Duration {
            std::time::Duration::from_millis(TIME.load(Ordering::Relaxed))
        }
        struct AdvancesTime;
        impl Drop for AdvancesTime {
            fn drop(&mut self) {
                TIME.store(13, Ordering::Relaxed);
            }
        }
        TIME.store(0, Ordering::Relaxed);
        let cpu = RendererCpu::new(
            RenderAccountingOptions::default()
                .with_clock(RenderClock::Host(now))
                .with_profile(RenderProfile::Summary { timing: true }),
        )
        .unwrap();
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let value = SharedValue::new(AdvancesTime, &storage).unwrap();
        TIME.store(5, Ordering::Relaxed);
        drop(value);
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .scope_elapsed(RenderCpuScope::FrameBacking),
            Some(std::time::Duration::from_millis(8))
        );
    }

    #[test]
    fn shared_value_factory_health_refusal_disposes_admitted_payload_in_its_cpu_phase() {
        use crate::accounting::{RenderClock, RenderProfile};
        use std::sync::{
            Arc,
            atomic::{AtomicU64, AtomicUsize, Ordering},
        };
        use std::time::Duration;

        static TIME: AtomicU64 = AtomicU64::new(0);
        fn now() -> Duration {
            Duration::from_millis(TIME.load(Ordering::SeqCst))
        }

        struct DropProbe {
            bytes: Vec<u8>,
            cpu: RendererCpu,
            drops: Arc<AtomicUsize>,
            observed_live: Arc<AtomicU64>,
        }
        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.observed_live.store(
                    self.cpu
                        .snapshot()
                        .unwrap()
                        .scope_usage(RenderCpuScope::FrameBacking)
                        .unwrap()
                        .live(RenderResource::RetainedCpuBytes),
                    Ordering::SeqCst,
                );
                drop(std::mem::take(&mut self.bytes));
                TIME.store(7, Ordering::SeqCst);
                self.drops.fetch_add(1, Ordering::SeqCst);
            }
        }

        TIME.store(0, Ordering::SeqCst);
        let cpu = RendererCpu::new(
            RenderAccountingOptions::new()
                .with_profile(RenderProfile::Summary { timing: true })
                .with_clock(RenderClock::Host(now)),
        )
        .unwrap();
        let account = cpu.at(RenderCpuScope::FrameBacking);
        let storage = StorageAccount::Renderer(account.clone());
        let mut bytes =
            AccountedVec::new_in(&storage, 3, RenderResource::RetainedCpuBytes).unwrap();
        bytes.extend_from_slice(&[1u8, 2, 3]).unwrap();
        let admitted_bytes = bytes.capacity() as u64;
        let (bytes, hold) = bytes.into_parts().unwrap();
        let drops = Arc::new(AtomicUsize::new(0));
        let observed_live = Arc::new(AtomicU64::new(u64::MAX));
        let value = DropProbe {
            bytes,
            cpu: cpu.clone(),
            drops: Arc::clone(&drops),
            observed_live: Arc::clone(&observed_live),
        };
        account.note_fault(RenderAccountingError::Invariant);

        let error = match SharedValue::with_parts(value, Some(hold), None, &storage) {
            Err(error) => error,
            Ok(_) => panic!("a faulted CPU owner must refuse immutable-value construction"),
        };
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Invariant)
        );
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        let snapshot = cpu.snapshot().unwrap();
        assert_eq!(snapshot.fault(), Some(RenderAccountingError::Invariant));
        // The rejected factory callback never starts; mandatory destruction
        // must nevertheless include the actual seven-millisecond value drop.
        // Check timing before custody so RED does not depend on closure-capture
        // destruction order for the separately captured raw value and hold.
        assert_eq!(
            snapshot.scope_elapsed(RenderCpuScope::FrameBacking),
            Some(Duration::from_millis(7))
        );
        assert_eq!(observed_live.load(Ordering::SeqCst), admitted_bytes);
        assert_eq!(
            snapshot
                .scope_usage(RenderCpuScope::FrameBacking)
                .unwrap()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
    }

    #[test]
    fn element_size_overflow_identifies_cpu_scope_before_backing_admission() {
        let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let before = cpu.snapshot().unwrap().usage();
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let error =
            AccountedVec::<u16>::new_in(&storage, usize::MAX, RenderResource::RetainedCpuBytes)
                .unwrap_err();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::CpuCapacityOverflow {
                scope: RenderCpuScope::FrameBacking,
                resource: RenderResource::RetainedCpuBytes,
            })
        );
        assert_eq!(cpu.snapshot().unwrap().usage(), before);
        assert_eq!(cpu.snapshot().unwrap().fault(), None);
    }

    #[test]
    fn preparation_element_size_overflow_keeps_its_existing_typed_error() {
        use crate::accounting::{RenderAccountingContext, RenderAttemptKind};
        let mut context = RenderAccountingContext::default();
        let result: Result<()> = context.prepare(
            RenderAttemptKind::GlyphRun,
            crate::GlyphRunLimits::new(0, 0, 0, 0),
            #[cfg(feature = "render-text")]
            None,
            |account| {
                let error =
                    AccountedVec::<u16>::new(account, usize::MAX, RenderResource::RetainedCpuBytes)
                        .unwrap_err();
                assert!(matches!(
                    error.glyph_preparation_diagnostic(),
                    Some(GlyphPreparationError::CapacityOverflow)
                ));
                Err(error)
            },
        );
        assert!(result.is_err());
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes),
            0
        );
    }

    #[test]
    fn byte_handle_keeps_the_same_counted_owner_until_its_final_clone_drops() {
        let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let baseline = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let mut bytes =
            AccountedVec::new_in(&storage, 3, RenderResource::RetainedCpuBytes).unwrap();
        bytes.extend_from_slice(&[4u8, 5, 6]).unwrap();
        let expected = baseline
            + bytes.capacity() as u64
            + std::mem::size_of::<SharedValueData<Vec<u8>>>() as u64;
        let value = SharedValue::from_accounted(bytes, &storage).unwrap();
        let identity = std::sync::Arc::as_ptr(&value.0).cast::<()>();
        let payload = value.into_bytes_arc();
        assert_eq!(std::sync::Arc::as_ptr(&payload).cast::<()>(), identity);
        let clone = payload.clone();
        drop(payload);
        assert_eq!(clone.as_ref().as_ref(), &[4, 5, 6]);
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            expected
        );
        drop(clone);
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            baseline
        );
    }

    #[test]
    fn allocation_identity_preserves_independent_equal_content() {
        let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let value = SharedValue::new(17u32, &storage).unwrap();
        let clone = value.clone();
        let independent = SharedValue::new(17u32, &storage).unwrap();
        assert!(value.same_allocation(&clone));
        assert!(!value.same_allocation(&independent));
        assert_eq!(value, independent);
    }

    #[test]
    fn shared_payload_admission_uses_actual_layout_and_retries_after_one_under_refusal() {
        let baseline = RendererCpu::new(RenderAccountingOptions::default())
            .unwrap()
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let allocation = std::mem::size_of::<SharedValueData<Vec<u8>>>() as u64;
        let bytes = vec![4u8, 5, 6];
        let capacity = bytes.capacity();
        let exact = baseline + capacity as u64 + allocation;
        eprintln!(
            "inline bytes: completion={}, start={}, forward={}, preparation_gradient={}, accounted_image={}, readback_output={}, accounted_vec={}, image_buffer={}; charged shared_vec_payload={}, shared_scalar_payload={}",
            std::mem::size_of::<DisposalCompletion>(),
            std::mem::size_of::<DisposalStart<'_>>(),
            std::mem::size_of::<crate::accounting::ForwardPhase<'_>>(),
            std::mem::size_of::<PreparationValue<crate::Gradient>>(),
            std::mem::size_of::<crate::image::AccountedImageBuffer>(),
            std::mem::size_of::<crate::readback::ReadbackOutput>(),
            std::mem::size_of::<AccountedVec<u8>>(),
            std::mem::size_of::<crate::ImageBuffer>(),
            std::mem::size_of::<SharedValueData<Vec<u8>>>(),
            std::mem::size_of::<SharedValueData<u32>>()
        );
        eprintln!(
            "actual shared payload bytes: Vec<u8>={}, u32={}, Image={}; backing capacity={capacity}, baseline={baseline}, exact={exact}",
            allocation,
            std::mem::size_of::<SharedValueData<u32>>(),
            std::mem::size_of::<SharedValueData<crate::Image>>()
        );
        for limit in [exact - 1, exact] {
            let cpu = RendererCpu::new(
                RenderAccountingOptions::new()
                    .with_limits(crate::RenderLimits::new().with_retained_bytes(limit)),
            )
            .unwrap();
            let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
            let mut values =
                AccountedVec::new_in(&storage, capacity, RenderResource::RetainedCpuBytes).unwrap();
            assert_eq!(values.capacity(), capacity);
            values.extend_from_slice(&bytes).unwrap();
            let result = SharedValue::from_accounted(values, &storage);
            if limit < exact {
                let error = result.unwrap_err();
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::CpuRefused {
                        scope: RenderCpuScope::Root,
                        quantity: crate::RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
                        current: baseline + capacity as u64,
                        amount: allocation,
                        reason: crate::RenderRefusalReason::Limit { limit },
                    })
                );
            } else {
                let value = result.unwrap();
                assert_eq!(value.as_slice(), bytes);
                assert_eq!(
                    cpu.snapshot()
                        .unwrap()
                        .usage()
                        .live(RenderResource::RetainedCpuBytes),
                    exact
                );
                drop(value);
            }
            let snapshot = cpu.snapshot().unwrap();
            assert_eq!(
                snapshot.usage().live(RenderResource::RetainedCpuBytes),
                baseline
            );
            assert_eq!(snapshot.fault(), None);
            // A refused admission is retryable on this same healthy owner.
            drop(SharedValue::new(9u8, &storage).unwrap());
            assert_eq!(
                cpu.snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes),
                baseline
            );
        }
    }

    #[test]
    fn shared_payload_retries_the_same_content_after_real_blocking_backing_is_released() {
        let source = [4u8, 5, 6];
        let capacity = source.to_vec().capacity();
        let baseline = RendererCpu::new(RenderAccountingOptions::default())
            .unwrap()
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let allocation = std::mem::size_of::<SharedValueData<Vec<u8>>>() as u64;
        let limit = baseline + capacity as u64 + allocation;
        let cpu = RendererCpu::new(
            RenderAccountingOptions::new()
                .with_limits(crate::RenderLimits::new().with_retained_bytes(limit)),
        )
        .unwrap();
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let mut blocker =
            AccountedVec::new_in(&storage, 1, RenderResource::RetainedCpuBytes).unwrap();
        blocker.extend_from_slice(&[17u8]).unwrap();
        let blocking_capacity = blocker.capacity() as u64;
        eprintln!(
            "shared retry bytes: baseline={baseline}, payload_backing={capacity}, blocker_backing={blocking_capacity}, allocation={allocation}, limit={limit}"
        );
        let make_values = || {
            let mut values =
                AccountedVec::new_in(&storage, capacity, RenderResource::RetainedCpuBytes).unwrap();
            assert_eq!(values.capacity(), capacity);
            values.extend_from_slice(&source).unwrap();
            values
        };
        let error = SharedValue::from_accounted(make_values(), &storage).unwrap_err();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::CpuRefused {
                scope: RenderCpuScope::Root,
                quantity: crate::RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
                current: baseline + blocking_capacity + capacity as u64,
                amount: allocation,
                reason: crate::RenderRefusalReason::Limit { limit },
            })
        );
        let refused = cpu.snapshot().unwrap();
        assert_eq!(
            refused.usage().live(RenderResource::RetainedCpuBytes),
            baseline + blocking_capacity
        );
        assert_eq!(refused.fault(), None);
        assert_eq!(blocker.as_slice(), &[17]);
        drop(blocker);
        let values = make_values();
        let backing_identity = values.as_ptr();
        let accepted = SharedValue::from_accounted(values, &storage).unwrap();
        assert_eq!(accepted.as_slice(), source);
        assert_eq!(accepted.as_ptr(), backing_identity);
        assert_eq!(accepted.capacity(), capacity);
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            limit
        );
        drop(accepted);
        let finished = cpu.snapshot().unwrap();
        assert_eq!(
            finished.usage().live(RenderResource::RetainedCpuBytes),
            baseline
        );
        assert_eq!(finished.fault(), None);
    }

    #[test]
    fn final_shared_payload_drop_keeps_backing_and_import_live_then_closes_before_import_cleanup() {
        use crate::accounting::{RenderClock, RenderCpuEvent, RenderPhaseFilter, RenderProfile};
        use std::{
            sync::{
                Arc,
                atomic::{AtomicU64, AtomicUsize, Ordering},
            },
            time::Duration,
        };
        static TIME: AtomicU64 = AtomicU64::new(0);
        fn now() -> Duration {
            Duration::from_millis(TIME.load(Ordering::SeqCst))
        }
        fn forbidden_clock() -> Duration {
            panic!("untimed shared payload read the clock")
        }
        struct Probe {
            bytes: Vec<u8>,
            image: Option<crate::Image>,
            cpu: RendererCpu,
            drops: Arc<AtomicUsize>,
            observed: Arc<AtomicU64>,
            unwind: bool,
        }
        impl Drop for Probe {
            fn drop(&mut self) {
                let before = self
                    .cpu
                    .snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes);
                self.observed.store(before, Ordering::SeqCst);
                drop(std::mem::take(&mut self.bytes));
                drop(self.image.take());
                assert_eq!(
                    self.cpu
                        .snapshot()
                        .unwrap()
                        .usage()
                        .live(RenderResource::RetainedCpuBytes),
                    before
                );
                TIME.fetch_add(7, Ordering::SeqCst);
                self.drops.fetch_add(1, Ordering::SeqCst);
                if self.unwind {
                    panic!("actual shared payload destructor fixture unwind");
                }
            }
        }
        fn require_send_sync<T: Send + Sync>() {}
        require_send_sync::<SharedValue<Probe>>();
        for profile in [
            RenderProfile::Off,
            RenderProfile::Summary { timing: false },
            RenderProfile::Summary { timing: true },
            RenderProfile::Detailed {
                timing: false,
                event_limit: 256,
                filter: RenderPhaseFilter::All,
            },
            RenderProfile::Detailed {
                timing: true,
                event_limit: 256,
                filter: RenderPhaseFilter::All,
            },
        ] {
            for unwind in [false, true] {
                let timed = matches!(
                    profile,
                    RenderProfile::Summary { timing: true }
                        | RenderProfile::Detailed { timing: true, .. }
                );
                TIME.store(0, Ordering::SeqCst);
                let cpu = RendererCpu::new(
                    RenderAccountingOptions::new()
                        .with_profile(profile)
                        .with_clock(RenderClock::Host(if timed { now } else { forbidden_clock })),
                )
                .unwrap();
                let account = cpu.at(RenderCpuScope::FrameBacking);
                let storage = StorageAccount::Renderer(account.clone());
                let image = crate::Image::from_rgba(crate::Size::new(1.0, 1.0), vec![0, 0, 0, 255])
                    .unwrap();
                let import = account.import_image(&image, None).unwrap();
                let imported = cpu
                    .snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes);
                let mut bytes =
                    AccountedVec::new_in(&storage, 3, RenderResource::RetainedCpuBytes).unwrap();
                bytes.extend_from_slice(&[1u8, 2, 3]).unwrap();
                let capacity = bytes.capacity();
                let (bytes, hold) = bytes.into_parts().unwrap();
                let drops = Arc::new(AtomicUsize::new(0));
                let observed = Arc::new(AtomicU64::new(0));
                let value = SharedValue::with_parts(
                    Probe {
                        bytes,
                        image: Some(image.clone()),
                        cpu: cpu.clone(),
                        drops: drops.clone(),
                        observed: observed.clone(),
                        unwind,
                    },
                    Some(hold),
                    Some(import),
                    &storage,
                )
                .unwrap();
                let expected = imported
                    + capacity as u64
                    + std::mem::size_of::<SharedValueData<Probe>>() as u64;
                let clone = value.clone();
                drop(value);
                assert_eq!(drops.load(Ordering::SeqCst), 0);
                assert_eq!(
                    cpu.snapshot()
                        .unwrap()
                        .usage()
                        .live(RenderResource::RetainedCpuBytes),
                    expected
                );
                let interval = cpu.begin_interval().unwrap();
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(clone)));
                assert_eq!(result.is_err(), unwind);
                assert_eq!(drops.load(Ordering::SeqCst), 1);
                assert_eq!(observed.load(Ordering::SeqCst), expected);
                let interval = interval.finish().unwrap();
                assert_eq!(
                    interval.scope_elapsed(RenderCpuScope::FrameBacking),
                    timed.then_some(Duration::from_millis(7))
                );
                assert_eq!(
                    interval.scope_elapsed(RenderCpuScope::Cleanup),
                    timed.then_some(Duration::ZERO)
                );
                assert_eq!(
                    interval.usage().live(RenderResource::RetainedCpuBytes)
                        + image.bytes.len() as u64
                        + std::mem::size_of::<Arc<[u8]>>() as u64,
                    imported
                );
                if matches!(profile, RenderProfile::Detailed { .. }) {
                    let events: Vec<_> = interval.events().collect();
                    let boundary = events
                        .iter()
                        .position(|event| {
                            matches!(
                                event,
                                RenderCpuEvent::Phase {
                                    scope: RenderCpuScope::FrameBacking,
                                    ..
                                }
                            )
                        })
                        .unwrap();
                    assert_eq!(
                        events[..boundary]
                            .iter()
                            .filter(|event| matches!(
                                event,
                                RenderCpuEvent::Change {
                                    scope: RenderCpuScope::FrameBacking,
                                    kind: crate::RenderChangeKind::Release,
                                    ..
                                }
                            ))
                            .count(),
                        2
                    );
                    assert!(events[boundary + 1..].iter().any(|event| matches!(
                        event,
                        RenderCpuEvent::Change {
                            scope: RenderCpuScope::ImagePayloads,
                            kind: crate::RenderChangeKind::Release,
                            ..
                        }
                    )));
                    assert!(
                        matches!(events.last(), Some(RenderCpuEvent::Phase { scope: RenderCpuScope::Cleanup, duration, .. }) if *duration == timed.then_some(Duration::ZERO))
                    );
                }
                assert_eq!(cpu.snapshot().unwrap().fault(), None);
            }
        }
    }
}

#[cfg(test)]
mod vector_capability_tests;

#[cfg(test)]
mod lexical_disposal_tests;
