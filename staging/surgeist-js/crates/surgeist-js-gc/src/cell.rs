//! Borrowable cells used by the collector.

#![forbid(unsafe_code)]

use crate::Trace;
use std::{
    cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut},
    cmp::Ordering,
    fmt::{self, Debug},
};

/// A dynamically borrowed cell whose contents participate in garbage collection.
pub struct GcRefCell<T: ?Sized + 'static> {
    cell: RefCell<T>,
}

impl<T> GcRefCell<T> {
    /// Creates a cell containing `value`.
    pub const fn new(value: T) -> Self {
        Self {
            cell: RefCell::new(value),
        }
    }

    /// Consumes the cell, returning its value.
    pub fn into_inner(self) -> T {
        self.cell.into_inner()
    }
}

impl<T: ?Sized> GcRefCell<T> {
    /// Borrows the value immutably.
    ///
    /// # Panics
    ///
    /// Panics if the value is mutably borrowed.
    #[track_caller]
    pub fn borrow(&self) -> Ref<'_, T> {
        self.cell.borrow()
    }

    /// Borrows the value mutably.
    ///
    /// # Panics
    ///
    /// Panics if the value is already borrowed.
    #[track_caller]
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.cell.borrow_mut()
    }

    /// Borrows the value immutably if available.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is mutably borrowed.
    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        self.cell.try_borrow()
    }

    /// Borrows the value mutably if available.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is already borrowed.
    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        self.cell.try_borrow_mut()
    }
}

impl<T: Trace + Clone> Clone for GcRefCell<T> {
    fn clone(&self) -> Self {
        Self::new(self.borrow().clone())
    }
}

impl<T: Default> Default for GcRefCell<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[allow(clippy::inline_always)]
impl<T: ?Sized + PartialEq> PartialEq for GcRefCell<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl<T: ?Sized + Eq> Eq for GcRefCell<T> {}

#[allow(clippy::inline_always)]
impl<T: ?Sized + PartialOrd> PartialOrd for GcRefCell<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (*self.borrow()).partial_cmp(&*other.borrow())
    }

    #[inline(always)]
    fn lt(&self, other: &Self) -> bool {
        *self.borrow() < *other.borrow()
    }

    #[inline(always)]
    fn le(&self, other: &Self) -> bool {
        *self.borrow() <= *other.borrow()
    }

    #[inline(always)]
    fn gt(&self, other: &Self) -> bool {
        *self.borrow() > *other.borrow()
    }

    #[inline(always)]
    fn ge(&self, other: &Self) -> bool {
        *self.borrow() >= *other.borrow()
    }
}

impl<T: ?Sized + Ord> Ord for GcRefCell<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self.borrow()).cmp(&*other.borrow())
    }
}

impl<T: ?Sized + Debug> Debug for GcRefCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GcRefCell")
            .field("cell", &&self.cell)
            .finish()
    }
}
