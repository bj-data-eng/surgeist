//! Explicit owner retention and per-operation allowances for required names.

use crate::{DomError, DomResource};
use std::mem::size_of;

/// Finite admission for one owner's exact-name registry.
///
/// Retention covers directly owned records, strings, index capacity and name
/// operation receipts, including canonical attribute-array storage and its
/// name-owned custody capsules. Attribute-value text keeps its separate payload
/// admission. Shared custody administration remains charged to its
/// existing owner and any enclosing Source allowance. This is not a total DOM
/// memory ceiling. Work and scratch apply to an ordinary batch or explicit
/// binding; nested Source operations also retain their cumulative allowances.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomNameLimits {
    retained_bytes: usize,
    entries: usize,
    operation_work: u64,
    operation_scratch: usize,
}

impl DomNameLimits {
    /// Require every allowance explicitly. Zero is a finite allowance; no
    /// sentinel value enables unlimited storage or work.
    pub fn new(
        retained_bytes: usize,
        entries: usize,
        operation_work: u64,
        operation_scratch: usize,
    ) -> Result<Self, DomError> {
        for (value, resource, width) in [
            (retained_bytes, DomResource::NameRetainedBytes, 1),
            (
                entries,
                DomResource::NameEntries,
                size_of::<crate::name::NameRecord>(),
            ),
            (operation_scratch, DomResource::NameScratch, 1),
        ] {
            if value > isize::MAX as usize / width {
                return Err(DomError::InvalidLimit(resource));
            }
        }
        Ok(Self {
            retained_bytes,
            entries,
            operation_work,
            operation_scratch,
        })
    }

    /// Name-owned retained resources, including pending records and reserved
    /// old/new overlap. Dead entries retain their allowance until removal.
    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }

    /// Live, dead and pending distinct weak-index entries combined.
    pub const fn entries(self) -> usize {
        self.entries
    }

    /// Cumulative name work within one operation and its nested operations.
    pub const fn operation_work(self) -> u64 {
        self.operation_work
    }

    /// Actual simultaneously retained or reserved temporary name storage.
    pub const fn operation_scratch(self) -> usize {
        self.operation_scratch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_and_full_width_work_are_explicit_finite_policies() {
        let zero = DomNameLimits::new(0, 0, 0, 0).unwrap();
        assert_eq!(zero.retained_bytes(), 0);
        assert_eq!(zero.entries(), 0);
        assert_eq!(zero.operation_work(), 0);
        assert_eq!(zero.operation_scratch(), 0);

        let finite = DomNameLimits::new(4096, 3, u64::MAX, 128).unwrap();
        assert_eq!(finite.retained_bytes(), 4096);
        assert_eq!(finite.entries(), 3);
        assert_eq!(finite.operation_work(), u64::MAX);
        assert_eq!(finite.operation_scratch(), 128);
    }

    #[test]
    fn unrepresentable_storage_is_rejected_by_its_named_resource() {
        for (result, resource) in [
            (
                DomNameLimits::new(usize::MAX, 0, 0, 0),
                DomResource::NameRetainedBytes,
            ),
            (
                DomNameLimits::new(0, usize::MAX, 0, 0),
                DomResource::NameEntries,
            ),
            (
                DomNameLimits::new(0, 0, 0, usize::MAX),
                DomResource::NameScratch,
            ),
        ] {
            assert_eq!(result, Err(DomError::InvalidLimit(resource)));
        }
    }
}
