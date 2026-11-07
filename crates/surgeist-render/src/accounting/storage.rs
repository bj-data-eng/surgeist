use super::{Account, Hold, RenderResource, bytes};
use crate::{GlyphPreparationError, Result};
use std::ops::{Deref, DerefMut};

/// Private explicit backing owner. Growth allocates a replacement so both
/// actual capacities remain attributable until the old backing is dropped.
pub(crate) struct AccountedVec<T> {
    values: Vec<T>,
    hold: Hold,
}
impl<T> AccountedVec<T> {
    pub(crate) fn new(account: &Account, count: usize, resource: RenderResource) -> Result<Self> {
        account.preview(resource, bytes::<T>(count)?)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| account.allocation_error(resource))?;
        let hold = account.acquire(resource, bytes::<T>(values.capacity())?)?;
        Ok(Self { values, hold })
    }
    pub(crate) fn grow(&mut self, additional: usize) -> Result<()> {
        let count = self
            .values
            .len()
            .checked_add(additional)
            .ok_or(GlyphPreparationError::CapacityOverflow)?;
        if count <= self.capacity() {
            return Ok(());
        }
        let mut next = Self::new(&self.hold.account, count, self.hold.resource)?;
        next.values.append(&mut self.values);
        std::mem::swap(self, &mut next);
        Ok(())
    }
    pub(crate) fn capacity(&self) -> usize {
        self.values.capacity()
    }
    pub(crate) fn as_slice(&self) -> &[T] {
        self.values.as_slice()
    }
    pub(crate) fn push(&mut self, value: T) -> Result<()> {
        self.grow(1)?;
        self.values.push(value);
        Ok(())
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
    #[cfg(feature = "render-text")]
    pub(crate) fn clear(&mut self) {
        self.values.clear();
    }
    pub(crate) fn truncate(&mut self, len: usize) {
        self.values.truncate(len);
    }
    /// Used only to lend the unchanged backing through the validated TextInstance
    /// constructor. Its paired hold must outlive that temporary instance.
    #[cfg(feature = "render-text")]
    pub(crate) fn into_parts(self) -> (Vec<T>, Hold) {
        (self.values, self.hold)
    }
    /// The result retains admitted backing until its whole attempt transfers it.
    pub(crate) fn into_retained(self) -> Vec<T> {
        let Self { values, hold } = self;
        hold.retain();
        values
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
