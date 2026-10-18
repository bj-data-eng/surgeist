//! Immutable sequence backing shared by canonical records and captures.
//! Only zero-capacity Vecs normalize to absence. Reserved empty buffers and
//! zero-sized-element Vec capacities retain their real shared target.
use std::{ops::Deref, sync::Arc};

#[cfg(test)]
mod allocation_observation;
#[cfg(test)]
mod storage_tests;

#[derive(Debug)]
pub(crate) struct SharedVec<T>(Option<Arc<Vec<T>>>);

impl<T> Clone for SharedVec<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> Default for SharedVec<T> {
    fn default() -> Self {
        Self::from(Vec::new())
    }
}
impl<T> SharedVec<T> {
    /// Absence is one neutral storage state, rather than a backing identity.
    pub(crate) fn same_storage(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        self.0.as_deref().map_or(&[], Vec::as_slice)
    }

    pub(crate) fn capacity(&self) -> usize {
        self.0.as_deref().map_or(0, Vec::capacity)
    }

    pub(crate) fn has_backing(&self) -> bool {
        self.0.is_some()
    }

    #[cfg(test)]
    pub(in crate::store) fn inspect_raw_allocations(
        &self,
        graph: &mut super::compact_tests::AllocationGraph,
    ) {
        self.inspect_named_allocations(graph, "sequence.target", "sequence.buffer");
    }

    #[cfg(test)]
    pub(in crate::store) fn inspect_named_allocations(
        &self,
        graph: &mut super::compact_tests::AllocationGraph,
        target: &'static str,
        buffer: &'static str,
    ) {
        if let Some(values) = &self.0 {
            graph.vector(values, target, buffer);
        }
    }

    /// The retained target body and actual buffer, excluding Arc metadata.
    pub(crate) fn retained_bytes(&self) -> Result<usize, super::DomError> {
        if !self.has_backing() {
            return Ok(0);
        }
        self.capacity()
            .checked_mul(size_of::<T>())
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<T>>()))
            .ok_or(super::DomError::RepresentationExhausted)
    }
}
impl<T> From<Vec<T>> for SharedVec<T> {
    fn from(value: Vec<T>) -> Self {
        if value.capacity() == 0 {
            return Self(None);
        }
        let backing = Arc::new(value);
        #[cfg(test)]
        if super::scale_metrics::ENABLED {
            allocation_observation::allocated::<T>(backing.capacity());
        }
        Self(Some(backing))
    }
}
impl<T> Deref for SharedVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T: PartialEq> PartialEq for SharedVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.same_storage(other) || self.as_slice() == other.as_slice()
    }
}
impl<T: Eq> Eq for SharedVec<T> {}
impl<'a, T> IntoIterator for &'a SharedVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
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
