//! Checked byte windows over a shared backing of single-width atomic words.

#![forbid(unsafe_code)]

use portable_atomic::AtomicU64;
use std::{
    collections::TryReserveError,
    ops::{Bound, RangeBounds},
    sync::atomic::Ordering,
};

const WORD_BYTES: usize = size_of::<u64>();

/// The logical byte count excludes unused bytes in the last atomic word.
#[derive(Debug, Default)]
pub(crate) struct SharedByteBlock {
    words: Vec<AtomicU64>,
    len: usize,
}

impl SharedByteBlock {
    pub(crate) fn zeroed(len: usize) -> Result<Self, TryReserveError> {
        let count = len.div_ceil(WORD_BYTES);
        let mut words = Vec::new();
        words.try_reserve_exact(count)?;
        words.resize_with(count, || AtomicU64::new(0));
        Ok(Self { words, len })
    }

    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    #[track_caller]
    pub(crate) fn bytes(&self, len: usize) -> SharedBytes<'_> {
        assert!(len <= self.len);
        SharedBytes {
            words: &self.words,
            start: 0,
            end: len,
        }
    }
}

/// A checked byte range. Words are encoded in increasing byte-address order.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SharedBytes<'a> {
    words: &'a [AtomicU64],
    start: usize,
    end: usize,
}

impl<'a> SharedBytes<'a> {
    pub(crate) const fn len(self) -> usize {
        self.end - self.start
    }

    #[track_caller]
    pub(crate) fn subslice(self, range: impl RangeBounds<usize>) -> Self {
        let start = match range.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start.checked_add(1).expect("start overflow"),
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&end) => end.checked_add(1).expect("end overflow"),
            Bound::Excluded(&end) => end,
            Bound::Unbounded => self.len(),
        };
        assert!(start <= end && end <= self.len());
        Self {
            words: self.words,
            start: self.start + start,
            end: self.start + end,
        }
    }

    pub(crate) fn load_byte(self, index: usize, order: Ordering) -> u8 {
        self.lane_at(index, 1).load(order) as u8
    }

    pub(crate) fn store_byte(self, index: usize, value: u8, order: Ordering) {
        self.lane_at(index, 1).store(u64::from(value), order);
    }

    pub(crate) fn lane(self, width: usize) -> AtomicLane<'a> {
        self.lane_at(0, width)
    }

    #[track_caller]
    fn lane_at(self, index: usize, width: usize) -> AtomicLane<'a> {
        assert!(matches!(width, 1 | 2 | 4 | 8));
        assert!(width <= self.len() && index <= self.len() - width);
        let offset = self.start + index;
        assert_eq!(offset % width, 0);
        AtomicLane {
            word: &self.words[offset / WORD_BYTES],
            shift: (offset % WORD_BYTES) * 8,
            mask: u64::MAX >> ((WORD_BYTES - width) * 8),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AtomicLane<'a> {
    word: &'a AtomicU64,
    shift: usize,
    mask: u64,
}

impl AtomicLane<'_> {
    fn extract(self, word: u64) -> u64 {
        (word >> self.shift) & self.mask
    }

    pub(crate) fn load(self, order: Ordering) -> u64 {
        self.extract(self.word.load(order))
    }

    pub(crate) fn store(self, value: u64, order: Ordering) {
        if self.mask == u64::MAX {
            self.word.store(value, order);
        } else {
            self.update(order, |_| Some(value));
        }
    }

    /// Updates just this lane and returns its observed value. An update may be
    /// retried if another agent writes any lane of the same word.
    pub(crate) fn update(self, order: Ordering, update: impl Fn(u64) -> Option<u64>) -> u64 {
        let read_order = match order {
            Ordering::Release => Ordering::Relaxed,
            Ordering::AcqRel => Ordering::Acquire,
            other => other,
        };
        let mut word = self.word.load(read_order);
        loop {
            let old = self.extract(word);
            let Some(replacement) = update(old) else {
                return old;
            };
            let replacement =
                (word & !(self.mask << self.shift)) | ((replacement & self.mask) << self.shift);
            match self
                .word
                .compare_exchange_weak(word, replacement, order, read_order)
            {
                Ok(_) => return old,
                Err(current) => word = current,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Ordering, SharedByteBlock};

    #[test]
    fn concurrent_updates_preserve_adjacent_lanes() {
        let backing = SharedByteBlock::zeroed(8).unwrap();
        let bytes = backing.bytes(8);
        let low = bytes.lane(4);
        let high = bytes.subslice(4..).lane(4);
        std::thread::scope(|scope| {
            scope.spawn(move || {
                for _ in 0..20_000 {
                    low.update(Ordering::SeqCst, |value| Some(value + 1));
                }
            });
            for _ in 0..20_000 {
                high.update(Ordering::SeqCst, |value| Some(value + 1));
            }
        });

        assert_eq!(low.load(Ordering::SeqCst), 20_000);
        assert_eq!(high.load(Ordering::SeqCst), 20_000);
    }
}
