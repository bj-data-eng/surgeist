//! All decoder allocations pass through this requested-payload ledger.
use super::{DecodeError, DecodeLimits, LimitKind, add, check, multiply};
use std::{
    cell::Cell,
    mem::size_of,
    ops::{Deref, DerefMut},
};

pub(super) struct Budget {
    pub limits: DecodeLimits,
    live: Cell<usize>,
    workspace: Cell<usize>,
}
impl Budget {
    pub fn new(limits: DecodeLimits, retained: usize) -> Result<Self, DecodeError> {
        check(LimitKind::LiveBytes, retained, limits.live_bytes())?;
        Ok(Self {
            limits,
            live: Cell::new(retained),
            workspace: Cell::new(0),
        })
    }
    pub fn usage(&self) -> (usize, usize) {
        (self.live.get(), self.workspace.get())
    }
    fn charge(&self, bytes: usize, temporary: bool) -> Result<(), DecodeError> {
        let live = add(self.live.get(), bytes)?;
        let workspace = add(self.workspace.get(), if temporary { bytes } else { 0 })?;
        check(LimitKind::LiveBytes, live, self.limits.live_bytes())?;
        check(
            LimitKind::WorkspaceBytes,
            workspace,
            self.limits.workspace_bytes(),
        )?;
        self.live.set(live);
        self.workspace.set(workspace);
        Ok(())
    }
    fn release(&self, bytes: usize, temporary: bool) {
        self.live.set(self.live.get() - bytes);
        if temporary {
            self.workspace.set(self.workspace.get() - bytes);
        }
    }
}

pub(super) struct Buffer<'a, T> {
    data: Vec<T>,
    budget: &'a Budget,
    temporary: bool,
}
impl<'a, T: Copy> Buffer<'a, T> {
    pub fn new(budget: &'a Budget) -> Self {
        Self {
            data: Vec::new(),
            budget,
            temporary: true,
        }
    }
    pub fn output(budget: &'a Budget) -> Self {
        Self {
            data: Vec::new(),
            budget,
            temporary: false,
        }
    }
    pub fn from_vec(data: Vec<T>, budget: &'a Budget) -> Result<Self, DecodeError> {
        budget.charge(multiply(data.capacity(), size_of::<T>())?, true)?;
        Ok(Self {
            data,
            budget,
            temporary: true,
        })
    }
    pub fn reserve(&mut self, required: usize) -> Result<(), DecodeError> {
        if required <= self.data.capacity() {
            return Ok(());
        }
        // Allocate a distinct replacement: the old and new payload coexist and
        // are both charged before the allocator is called. No hidden realloc.
        let bytes = multiply(required, size_of::<T>())?;
        self.budget.charge(bytes, self.temporary)?;
        let mut replacement = Vec::new();
        if replacement.try_reserve_exact(required).is_err() {
            self.budget.release(bytes, self.temporary);
            return Err(DecodeError::AllocationFailed);
        }
        // try_reserve_exact requests precisely the charged element payload.
        // Vec may expose allocator slack; charge that too before retaining it.
        let actual = multiply(replacement.capacity(), size_of::<T>())?;
        if let Err(error) = self.budget.charge(actual - bytes, self.temporary) {
            self.budget.release(bytes, self.temporary);
            return Err(error);
        }
        replacement.extend_from_slice(&self.data);
        let old = std::mem::replace(&mut self.data, replacement);
        self.budget
            .release(multiply(old.capacity(), size_of::<T>())?, self.temporary);
        drop(old);
        Ok(())
    }
    pub fn push(&mut self, value: T) -> Result<(), DecodeError> {
        self.reserve(add(self.data.len(), 1)?)?;
        self.data.push(value);
        Ok(())
    }
    pub fn extend(&mut self, values: &[T]) -> Result<(), DecodeError> {
        self.reserve(add(self.data.len(), values.len())?)?;
        self.data.extend_from_slice(values);
        Ok(())
    }
    pub fn resize(&mut self, length: usize, value: T) -> Result<(), DecodeError> {
        self.reserve(length)?;
        self.data.resize(length, value);
        Ok(())
    }
    pub fn clear(&mut self) {
        self.data.clear();
    }
    pub fn into_vec(mut self) -> Vec<T> {
        let data = std::mem::take(&mut self.data);
        self.budget
            .release(data.capacity() * size_of::<T>(), self.temporary);
        data
    }
}
impl<T> Deref for Buffer<'_, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.data
    }
}
impl<T> DerefMut for Buffer<'_, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}
impl<T> Drop for Buffer<'_, T> {
    fn drop(&mut self) {
        self.budget
            .release(self.data.capacity() * size_of::<T>(), self.temporary);
    }
}
