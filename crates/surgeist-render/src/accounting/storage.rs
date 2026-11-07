use super::{
    Account, CpuAccount, CpuHold, CpuImport, PreparationHold, RenderAccountingError, RenderResource,
};
use crate::{GlyphPreparationError, Result};
use std::ops::{Deref, DerefMut};

/// One explicit fixed-size heap payload. The value is disposed before its
/// admission; nested values retain their independent backing guards.
pub(crate) struct OwnedBox<T> {
    value: Option<Box<T>>,
    hold: Option<Hold>,
}
impl<T> OwnedBox<T> {
    pub(crate) fn new(value: T, account: &StorageAccount) -> Result<Self> {
        account.phase(|| {
            let hold =
                account.acquire(RenderResource::RetainedCpuBytes, std::mem::size_of::<T>())?;
            Ok(Self {
                value: Some(Box::new(value)),
                hold: Some(hold),
            })
        })
    }
    pub(crate) fn into_inner(mut self) -> T {
        let value = self.value.take().expect("owned box payload");
        self.hold
            .take()
            .expect("owned box admission")
            .dispose(|| *value)
    }
}
impl<T> Drop for OwnedBox<T> {
    fn drop(&mut self) {
        if let Some(hold) = self.hold.take() {
            let value = self.value.take();
            hold.dispose(|| drop(value));
        }
    }
}
impl<T> Deref for OwnedBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value.as_ref().expect("owned box payload")
    }
}
impl<T> DerefMut for OwnedBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value.as_mut().expect("owned box payload")
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for OwnedBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.as_ref().expect("owned box payload").fmt(f)
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
    value: Option<T>,
    _backing: Option<Hold>,
    _import: Option<CpuImport>,
    _allocation: Option<Hold>,
}
impl<T: AsRef<[u8]>> AsRef<[u8]> for SharedValueData<T> {
    fn as_ref(&self) -> &[u8] {
        self.value.as_ref().expect("shared byte payload").as_ref()
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
        account.phase(|| {
            let allocation = account.acquire(
                RenderResource::RetainedCpuBytes,
                std::mem::size_of::<SharedValueData<T>>(),
            )?;
            // Stable safe Arc allocation follows the existing immutable-value
            // allocator boundary. Its explicit typed payload is admitted first;
            // no opaque Arc/allocator header is estimated.
            Ok(Self(std::sync::Arc::new(SharedValueData {
                value: Some(value),
                _backing: backing,
                _import: import,
                _allocation: Some(allocation),
            })))
        })
    }
    pub(crate) fn from_accounted(
        values: AccountedVec<T>,
        account: &StorageAccount,
    ) -> Result<SharedValue<Vec<T>>> {
        let (values, hold) = values.into_parts();
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
        if let Some(allocation) = self._allocation.take() {
            let value = self.value.take();
            let backing = self._backing.take();
            let import = self._import.take();
            allocation.dispose(|| {
                drop(value);
                drop(backing);
                drop(import);
            });
        }
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
        let values = data.value.as_mut().expect("shared fixture payload");
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
        let (mut values, hold) = replacement.into_parts();
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
        self.0.value.as_ref().expect("shared value payload")
    }
}
impl<'a, T> IntoIterator for &'a SharedValue<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0
            .value
            .as_ref()
            .expect("shared value payload")
            .into_iter()
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for SharedValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.value.as_ref().expect("shared value payload").fmt(f)
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
        self.0
            .value
            .as_ref()
            .expect("shared value payload")
            .hash(state);
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
            Self::Preparation(_) => GlyphPreparationError::CapacityOverflow.into(),
            Self::Renderer(account) => account.capacity_error(resource),
        }
    }
    pub(crate) fn bytes<T>(&self, count: usize, resource: RenderResource) -> Result<usize> {
        match self {
            Self::Preparation(_) => super::bytes::<T>(count),
            Self::Renderer(account) => account.sized::<T>(count, resource),
        }
    }
    fn invariant_error(&self) -> crate::Error {
        if let Self::Renderer(account) = self {
            account.note_fault(RenderAccountingError::Invariant);
        }
        RenderAccountingError::Invariant.into()
    }
    fn phase<T>(&self, run: impl FnOnce() -> Result<T>) -> Result<T> {
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
    fn amount(&self) -> u64 {
        match self {
            Self::Preparation(hold) => hold.amount,
            Self::Renderer(hold) => hold.amount(),
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
    /// Standalone preparation transfers its operation's result backing at the
    /// outer return. Renderer backing must retain its paired lifetime hold.
    pub(crate) fn retain(self) {
        match self {
            Self::Preparation(hold) => hold.retain(),
            Self::Renderer(_) => panic!("renderer backing requires its lifetime hold"),
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
    if match hold {
        Some(hold) => {
            hold.resource() != resource
                || hold.amount()
                    != u64::try_from(current).map_err(|_| account.capacity_error(resource))?
        }
        None => current != 0,
    } {
        return Err(account.invariant_error());
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
        let hold = self.hold.as_ref().expect("owned vector admission");
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
            let mut next = Self::allocate(
                &storage,
                count,
                self.hold
                    .as_ref()
                    .expect("owned vector admission")
                    .resource(),
            )?;
            before_move(self.values.len())?;
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
    /// Appends after the caller has proved spare capacity and admitted its work.
    pub(crate) fn push_reserved(&mut self, value: T) {
        assert!(
            self.values.len() < self.values.capacity(),
            "admitted spare capacity"
        );
        self.values.push(value);
    }
    /// Inserts after capacity and the exact suffix movement have been admitted.
    pub(crate) fn insert_reserved(&mut self, index: usize, value: T) {
        assert!(
            self.values.len() < self.values.capacity(),
            "admitted spare capacity"
        );
        assert!(index <= self.values.len(), "validated insertion index");
        self.values.insert(index, value);
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
    /// Lending transfers the backing and its exact admission together.
    pub(crate) fn into_parts(mut self) -> (Vec<T>, Hold) {
        (
            std::mem::take(&mut self.values),
            self.hold.take().expect("owned vector admission"),
        )
    }
    /// Rejoins a producer backing which was proved not to grow while lent.
    pub(crate) fn from_parts(values: Vec<T>, hold: Hold) -> Result<Self> {
        let account = hold.storage();
        let resource = hold.resource();
        let expected = account
            .bytes::<T>(values.capacity(), resource)
            .and_then(|amount| u64::try_from(amount).map_err(|_| account.capacity_error(resource)));
        match expected {
            Ok(amount) if hold.amount() == amount => Ok(Self {
                values,
                hold: Some(hold),
            }),
            expected => {
                let error = match expected {
                    Err(error) => error,
                    Ok(_) => account.invariant_error(),
                };
                hold.dispose(|| drop(values));
                Err(error)
            }
        }
    }
    /// Standalone result retention is finalized by the outer preparation owner.
    pub(crate) fn into_retained(self) -> Vec<T> {
        let (values, hold) = self.into_parts();
        hold.retain();
        values
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
    values: Option<std::vec::IntoIter<T>>,
    hold: Option<Hold>,
}
impl<T> Iterator for AccountedIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.values.as_mut().expect("owned iterator").next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.values.as_ref().expect("owned iterator").size_hint()
    }
}
impl<T> ExactSizeIterator for AccountedIntoIter<T> {}
impl<T> DoubleEndedIterator for AccountedIntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.values.as_mut().expect("owned iterator").next_back()
    }
}
impl<T> std::iter::FusedIterator for AccountedIntoIter<T> {}
impl<T> Drop for AccountedIntoIter<T> {
    fn drop(&mut self) {
        if let Some(hold) = self.hold.take() {
            let values = self.values.take();
            hold.dispose(|| drop(values));
        }
    }
}
impl<T> IntoIterator for AccountedVec<T> {
    type Item = T;
    type IntoIter = AccountedIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        let (values, hold) = self.into_parts();
        AccountedIntoIter {
            values: Some(values.into_iter()),
            hold: Some(hold),
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
    fn boxed_payload_releases_fixed_allocation_when_moved_back_inline() {
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
        assert_eq!(OwnedBox::into_inner(value), 73);
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
}
