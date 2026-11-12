//! The capacity policy of bounded vectors that grow one record at a time.
//!
//! `Vec::try_reserve_exact(1)` on a full vector sets the capacity to `len + 1`, so `n` pushes
//! request `n` reallocations. Growing a full vector to at least twice its capacity instead
//! requests `O(log n)` of them, whether or not an allocator copies the records.

use std::collections::TryReserveError;

/// The least capacity a full vector grows to. Below it, doubling would request a reallocation for
/// each of the first few records.
const MINIMUM_GROWTH: usize = 4;

/// The capacity a vector of `len` records and `capacity` slots has once room for one more record
/// is made.
///
/// A vector with room keeps its capacity. A full vector at least doubles, from a minimum, and
/// always reaches `len + 1`. The result depends only on the vector, never on a limit: a consumer
/// that admits the target against a limit then sees the same allocation path under every limit,
/// so a query succeeds exactly when every limit holds the usage that path needs. Arithmetic
/// saturates, so no input overflows; whether the layout can be allocated is the allocator's
/// answer.
pub(crate) fn amortized_capacity(len: usize, capacity: usize) -> usize {
    let needed = len.saturating_add(1);
    if needed <= capacity {
        return capacity;
    }
    needed.max(capacity.saturating_mul(2).max(MINIMUM_GROWTH))
}

/// Makes room for one more record in `values` by growing to
/// [`amortized_capacity`]`(values.len(), values.capacity())`; see [`reserve_to`].
pub(crate) fn reserve_amortized<T>(values: &mut Vec<T>) -> Result<(), TryReserveError> {
    let target = amortized_capacity(values.len(), values.capacity());
    reserve_to(values, target)
}

/// Grows `values` to hold at least `target` records when it holds fewer, and at least one more
/// record than it contains either way.
///
/// When the allocator refuses the larger allocation, the single record that is needed is requested
/// exactly, so an allocator refusal of the spare room never fails where exact growth would have
/// succeeded. An error leaves `values` unchanged. This is the only fallback: a limit that refuses
/// a target is the consumer's decision and refuses the reservation.
///
/// A caller that keeps a second vector sized to the first reserves that one with its own
/// `reserve_to` call after the first, with the first's resulting capacity, so a fallback in the
/// first lowers the second's target.
pub(crate) fn reserve_to<T>(values: &mut Vec<T>, target: usize) -> Result<(), TryReserveError> {
    // Only a call that reaches the allocator is a request: one that finds room returns without
    // asking, so the allocator could never refuse it and the test counter must not count it.
    #[cfg(test)]
    if (target > values.capacity() || values.len() == values.capacity())
        && fault::refuses_next_reservation()
    {
        return Err(fault::refusal());
    }
    if target > values.capacity() {
        let growth = target.saturating_sub(values.len());
        if growth > 1 && values.try_reserve_exact(growth).is_ok() {
            return Ok(());
        }
        return values.try_reserve_exact(1);
    }
    if values.len() == values.capacity() {
        return values.try_reserve_exact(1);
    }
    Ok(())
}

/// A deterministic allocator refusal for the reservations of the crate's own storage.
///
/// The real allocator cannot be made to refuse a small request, so a test arms this thread's
/// counter and the reservation helpers fail the chosen request, and every later one, with the
/// error a refused allocation reports. Nothing outside tests can arm it.
#[cfg(test)]
pub(crate) mod fault {
    use std::cell::Cell;
    use std::collections::TryReserveError;

    thread_local! {
        static REFUSE_FROM: Cell<Option<usize>> = const { Cell::new(None) };
        static REQUESTS: Cell<usize> = const { Cell::new(0) };
        static REFUSE_CHILDREN: Cell<bool> = const { Cell::new(false) };
        static REFUSE_TEXT_SHARE: Cell<bool> = const { Cell::new(false) };
    }

    /// Makes a publication refuse to share its block with a text record until the guard drops,
    /// even if the test panics. The refusal is the one a range the block cannot hold gives, which
    /// publication's own validation makes unreachable.
    pub(crate) struct TextShareRefused;
    impl Drop for TextShareRefused {
        fn drop(&mut self) {
            REFUSE_TEXT_SHARE.with(|cell| cell.set(false));
        }
    }

    /// Refuses every share of a published text with its block while the returned guard lives.
    pub(crate) fn refuse_text_share() -> TextShareRefused {
        REFUSE_TEXT_SHARE.with(|cell| cell.set(true));
        TextShareRefused
    }

    /// Whether the share of a published text with its block is refused.
    pub(crate) fn refuses_text_share() -> bool {
        REFUSE_TEXT_SHARE.with(Cell::get)
    }

    /// Makes the allocator refuse every reservation of a child order, the room needed as well as
    /// spare room, until the guard drops, even if the test panics.
    pub(crate) struct ChildrenRefused;
    impl Drop for ChildrenRefused {
        fn drop(&mut self) {
            REFUSE_CHILDREN.with(|cell| cell.set(false));
        }
    }

    /// Refuses every reservation of a child order while the returned guard lives.
    pub(crate) fn refuse_children() -> ChildrenRefused {
        REFUSE_CHILDREN.with(|cell| cell.set(true));
        ChildrenRefused
    }

    /// Whether child-order reservations are refused.
    pub(crate) fn refuses_children() -> bool {
        REFUSE_CHILDREN.with(Cell::get)
    }

    /// Disarms the counter when dropped, even if the test panics.
    pub(crate) struct Armed;
    impl Drop for Armed {
        fn drop(&mut self) {
            REFUSE_FROM.with(|cell| cell.set(None));
        }
    }

    /// Counts requests from zero. The `refuse_from`-th request (one-based) and every later one is
    /// refused.
    pub(crate) fn arm(refuse_from: usize) -> Armed {
        REQUESTS.with(|cell| cell.set(0));
        REFUSE_FROM.with(|cell| cell.set(Some(refuse_from)));
        Armed
    }

    /// Counts requests from zero and refuses none.
    pub(crate) fn count_only() -> Armed {
        arm(usize::MAX)
    }

    /// The requests made since the counter was armed.
    pub(crate) fn requests() -> usize {
        REQUESTS.with(Cell::get)
    }

    /// Counts one request and reports whether it is refused.
    pub(crate) fn refuses_next_reservation() -> bool {
        let Some(from) = REFUSE_FROM.with(Cell::get) else {
            return false;
        };
        let request = REQUESTS.with(|cell| {
            cell.set(cell.get() + 1);
            cell.get()
        });
        request >= from
    }

    /// The error of a refused allocation.
    pub(crate) fn refusal() -> TryReserveError {
        match Vec::<u8>::new().try_reserve(usize::MAX) {
            Err(error) => error,
            Ok(()) => unreachable!("a request of usize::MAX bytes always overflows"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_vector_with_room_keeps_its_capacity() {
        assert_eq!(amortized_capacity(3, 4), 4);
        assert_eq!(amortized_capacity(0, 8), 8);
    }

    #[test]
    fn a_full_vector_doubles_from_a_minimum_and_never_stops_short_of_the_record() {
        assert_eq!(amortized_capacity(0, 0), MINIMUM_GROWTH);
        assert_eq!(amortized_capacity(4, 4), 8);
        assert_eq!(amortized_capacity(1_000, 1_000), 2_000);
        // Capacity below the length cannot happen for a vector, yet the record is still reached.
        assert_eq!(amortized_capacity(10, 2), 10 + 1);
    }

    #[test]
    fn arithmetic_saturates_instead_of_overflowing() {
        assert_eq!(amortized_capacity(usize::MAX, usize::MAX), usize::MAX);
        assert_eq!(
            amortized_capacity(0, usize::MAX / 2 + 1),
            usize::MAX / 2 + 1
        );
        assert_eq!(
            amortized_capacity(usize::MAX / 2 + 1, usize::MAX / 2 + 1),
            usize::MAX
        );
    }

    #[test]
    fn n_pushes_request_a_logarithmic_number_of_reallocations() {
        let mut values: Vec<u64> = Vec::new();
        let mut changes = 0;
        let mut capacity = 0;
        for index in 0..10_000_u64 {
            reserve_amortized(&mut values).unwrap();
            if values.capacity() != capacity {
                capacity = values.capacity();
                changes += 1;
            }
            values.push(index);
        }
        let allowed = 10_000_usize.ilog2() as usize + 2;
        assert!(changes <= allowed, "{changes} changes, more than {allowed}");
    }

    #[test]
    fn a_refused_larger_target_still_makes_room_for_the_needed_record() {
        // A target of `usize::MAX` records of eight bytes exceeds `isize::MAX` bytes, so
        // `try_reserve_exact` refuses it as a capacity overflow before any allocator call. The
        // single record that is needed must still be reserved.
        let mut values: Vec<u64> = vec![1, 2, 3];
        values.resize(values.capacity(), 0);
        let before = values.clone();
        let full = values.len();
        assert!(values.try_reserve_exact(usize::MAX - full).is_err());
        assert_eq!(values.capacity(), full);
        assert_eq!(reserve_to(&mut values, usize::MAX), Ok(()));
        assert!(values.capacity() > full);
        assert_eq!(values, before);
    }

    #[test]
    fn reserving_the_next_record_grows_exactly() {
        let mut values: Vec<u8> = Vec::new();
        reserve_to(&mut values, 1).unwrap();
        assert!(values.capacity() >= 1);
        values.push(0);
        let before = values.capacity();
        values.resize(before, 0);
        reserve_to(&mut values, before + 1).unwrap();
        assert_eq!(values.capacity(), before + 1);
    }
}
