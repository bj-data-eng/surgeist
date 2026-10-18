//! Immutable sequence backing shared by canonical records and captures.
use std::{ops::Deref, sync::Arc};

#[cfg(test)]
mod allocation_observation;
#[cfg(test)]
mod storage_tests;

#[derive(Debug)]
pub(crate) struct SharedVec<T>(Arc<Vec<T>>);

impl<T> Clone for SharedVec<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
impl<T> Default for SharedVec<T> {
    fn default() -> Self {
        Self::from(Vec::new())
    }
}
impl<T> SharedVec<T> {
    pub(crate) fn shares_backing(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl<T> From<Vec<T>> for SharedVec<T> {
    fn from(value: Vec<T>) -> Self {
        let backing = Arc::new(value);
        #[cfg(test)]
        if super::scale_metrics::ENABLED {
            allocation_observation::allocated::<T>(backing.capacity());
        }
        Self(backing)
    }
}
impl<T> Deref for SharedVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: PartialEq> PartialEq for SharedVec<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}
impl<T: Eq> Eq for SharedVec<T> {}
impl<'a, T> IntoIterator for &'a SharedVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_capture_clone_retains_the_actual_sequence_and_capacity() {
        let mut values = Vec::with_capacity(1024);
        values.extend([3, 5, 8]);
        let first = SharedVec::from(values);
        let capture = first.clone();
        assert_eq!(first.as_slice(), &[3, 5, 8]);
        assert_eq!(capture.as_slice(), &[3, 5, 8]);
        assert_eq!(first.as_ptr(), capture.as_ptr());
        assert_eq!(capture.capacity(), 1024);
        drop(first);
        assert_eq!(capture.as_slice(), &[3, 5, 8]);
    }

    #[test]
    fn equal_sequences_with_different_backing_are_semantically_equal() {
        let first = SharedVec::from(vec![1, 2]);
        let mut other = Vec::with_capacity(64);
        other.extend([1, 2]);
        let other = SharedVec::from(other);
        assert_ne!(first.as_ptr(), other.as_ptr());
        assert_eq!(first, other);
    }
}
