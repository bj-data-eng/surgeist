//! Strict Brotli with a complete, fallibly prepared allocation inventory.
//!
//! This census is specific to brotli-decompressor 5.0.0 with default features
//! disabled. Only `new_strict` and the stream entry point may use these pools;
//! the state stays private and never receives a custom dictionary.

use std::mem::size_of;

use brotli_decompressor::{
    Allocator, BrotliDecompressStream, BrotliResult, BrotliState, HuffmanCode, SliceWrapper,
    SliceWrapperMut,
};

use super::{DecodeError, DecodeLimits, InvalidKind, LimitKind, add, check, invalid, multiply};

// Upstream decode.rs:1855,1293,3033; state.rs:410; huffman/mod.rs:61.
// Strict windows are <=24 bits, block/tree counts <=256, and each tree needs
// <=1080 HuffmanCode entries. These bounds also precede allocation on malformed
// input. The ring's 66 extra bytes cover write-ahead slack and dictionary words.
const BYTE_SLOTS: [usize; 4] = [256, 1024, 16_384, 16_777_282];
const OFFSET_SLOTS: [usize; 3] = [256; 3];
const CODE_SLOTS: [usize; 6] = [1080, 3240, 3240, 276_480, 276_480, 276_480];

type State = BrotliState<Pool<u8, 4>, Pool<u32, 3>, Pool<HuffmanCode, 6>>;

/// Decodes one complete transformed stream, not the final reconstructed SFNT.
/// `already_workspace` is the portion of `already_live` that is temporary.
/// The caller charges the returned Vec's capacity after this function's pool
/// and state have dropped. Input backing must already be included by the caller.
pub(super) fn decompress(
    input: &[u8],
    expected_len: usize,
    limits: DecodeLimits,
    already_live: usize,
    already_workspace: usize,
) -> Result<Vec<u8>, DecodeError> {
    let mut budget = Budget::new(expected_len, limits, already_live, already_workspace)?;
    let mut output = prepared_vec::<u8>(expected_len, &mut budget)?;
    let bytes = Pool::prepare(BYTE_SLOTS, &mut budget)?;
    let offsets = Pool::prepare(OFFSET_SLOTS, &mut budget)?;
    let codes = Pool::prepare(CODE_SLOTS, &mut budget)?;
    // No allocation is possible beyond this boundary: callbacks only move and
    // reset cells prepared above. Do not mutate provider state/configuration.
    let mut state = State::new_strict(bytes, offsets, codes);
    let mut available_in = input.len();
    let mut input_offset = 0;
    let mut available_out = output.len();
    let mut output_offset = 0;
    let mut total_out = 0;
    let result = BrotliDecompressStream(
        &mut available_in,
        &mut input_offset,
        input,
        &mut available_out,
        &mut output_offset,
        &mut output,
        &mut total_out,
        &mut state,
    );
    if !matches!(result, BrotliResult::ResultSuccess)
        || available_in != 0
        || input_offset != input.len()
        || available_out != 0
        || output_offset != expected_len
        || total_out != expected_len
    {
        return Err(invalid(InvalidKind::CompressedData, input_offset));
    }
    Ok(output)
}

/// Tracks complete requested storage before any allocation, then incorporates
/// actual Vec capacities. The state size includes all pool slots and cells;
/// output and preparation metadata are separate. This is owned payload/metadata
/// accounting, not a compiler-dependent bound on machine stack-frame size.
struct Budget {
    limits: DecodeLimits,
    already_live: usize,
    already_workspace: usize,
    adapter_bytes: usize,
}

impl Budget {
    fn new(
        output_len: usize,
        limits: DecodeLimits,
        already_live: usize,
        already_workspace: usize,
    ) -> Result<Self, DecodeError> {
        let mut adapter_bytes = add(size_of::<State>(), size_of::<Self>())?;
        adapter_bytes = add(adapter_bytes, size_of::<Vec<u8>>())?;
        adapter_bytes = add(adapter_bytes, output_len)?;
        for len in BYTE_SLOTS {
            adapter_bytes = add(adapter_bytes, len)?;
        }
        for len in OFFSET_SLOTS {
            adapter_bytes = add(adapter_bytes, multiply(len, size_of::<u32>())?)?;
        }
        for len in CODE_SLOTS {
            adapter_bytes = add(adapter_bytes, multiply(len, size_of::<HuffmanCode>())?)?;
        }
        let budget = Self {
            limits,
            already_live,
            already_workspace,
            adapter_bytes,
        };
        budget.check()?;
        Ok(budget)
    }

    fn check(&self) -> Result<(), DecodeError> {
        check(
            LimitKind::WorkspaceBytes,
            add(self.already_workspace, self.adapter_bytes)?,
            self.limits.workspace_bytes(),
        )?;
        check(
            LimitKind::LiveBytes,
            add(self.already_live, self.adapter_bytes)?,
            self.limits.live_bytes(),
        )
    }

    fn capacity<T>(&mut self, requested: usize, capacity: usize) -> Result<(), DecodeError> {
        // try_reserve_exact promises at least the requested capacity. std does
        // not expose any allocator-granted excess before the reservation; charge
        // it immediately and refuse before initialization or provider entry.
        let excess = capacity
            .checked_sub(requested)
            .expect("Brotli preparation capacity covers its request");
        self.adapter_bytes = add(self.adapter_bytes, multiply(excess, size_of::<T>())?)?;
        self.check()
    }
}

fn prepared_vec<T: Copy + Default>(len: usize, budget: &mut Budget) -> Result<Vec<T>, DecodeError> {
    // The complete remaining inventory, not merely this cell, is checked before
    // every reservation. Initialization cannot allocate after sufficient reserve.
    budget.check()?;
    let mut storage = Vec::new();
    storage
        .try_reserve_exact(len)
        .map_err(|_| DecodeError::AllocationFailed)?;
    budget.capacity::<T>(len, storage.capacity())?;
    storage.resize(len, T::default());
    Ok(storage)
}

/// Only provider-owned default cells have no slot. A checked-out cell retains
/// its entire initialized backing Vec but exposes exactly the requested length.
struct Cell<T> {
    storage: Vec<T>,
    slot: Option<usize>,
    len: usize,
}

impl<T> Default for Cell<T> {
    fn default() -> Self {
        Self {
            storage: Vec::new(),
            slot: None,
            len: 0,
        }
    }
}

impl<T> SliceWrapper<T> for Cell<T> {
    fn slice(&self) -> &[T] {
        &self.storage[..self.len]
    }
}

impl<T> SliceWrapperMut<T> for Cell<T> {
    fn slice_mut(&mut self) -> &mut [T] {
        &mut self.storage[..self.len]
    }
}

struct Pool<T, const N: usize> {
    slots: [Option<Cell<T>>; N],
    lengths: [usize; N],
}

impl<T: Copy + Default, const N: usize> Pool<T, N> {
    fn prepare(lengths: [usize; N], budget: &mut Budget) -> Result<Self, DecodeError> {
        let mut slots = std::array::from_fn(|_| None);
        for (slot, len) in lengths.iter().copied().enumerate() {
            slots[slot] = Some(Cell {
                storage: prepared_vec(len, budget)?,
                slot: Some(slot),
                len: 0,
            });
        }
        Ok(Self { slots, lengths })
    }
}

impl<T: Copy + Default, const N: usize> Allocator<T> for Pool<T, N> {
    type AllocatedMemory = Cell<T>;

    fn alloc_cell(&mut self, len: usize) -> Cell<T> {
        assert!(len != 0, "Brotli nonempty-allocation proof defect");
        // new_strict allocates the three mandatory Huffman cells first. Ring
        // allocation precedes modes/maps. All nine metablock cells are freed
        // before the next block (state.rs:434-460); the ring never grows.
        // Smallest fit preserves that complete simultaneous-allocation census.
        let slot = self
            .slots
            .iter()
            .enumerate()
            .filter(|(slot, cell)| cell.is_some() && self.lengths[*slot] >= len)
            .min_by_key(|(slot, _)| self.lengths[*slot])
            .map(|(slot, _)| slot)
            .expect("Brotli 5.0.0 strict allocation inventory proof defect");
        let mut cell = self.slots[slot]
            .take()
            .expect("Brotli selected an available pool slot");
        cell.len = len;
        cell.slice_mut().fill(T::default());
        cell
    }

    fn free_cell(&mut self, mut cell: Cell<T>) {
        let Some(slot) = cell.slot else {
            assert!(
                cell.len == 0 && cell.storage.capacity() == 0,
                "Brotli default-cell ownership proof defect"
            );
            return;
        };
        assert!(
            slot < N
                && self.slots[slot].is_none()
                && cell.storage.len() == self.lengths[slot]
                && cell.len != 0
                && cell.len <= cell.storage.len(),
            "Brotli pool recycling proof defect"
        );
        cell.len = 0;
        self.slots[slot] = Some(cell);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> DecodeLimits {
        DecodeLimits::new(usize::MAX, 0, 0, 0, 0, usize::MAX, usize::MAX)
    }

    // Independently written from RFC 7932 sections 9.1-9.2, least-significant
    // bit first: WBITS=16 (0), ISLAST=0, MNIBBLES=0, MLEN=2, ISUNCOMPRESSED=1,
    // zero alignment, "abc", then ISLAST=1 and ISLASTEMPTY=1. No encoder used.
    const ABC: &[u8] = &[0x20, 0, 0x10, b'a', b'b', b'c', 0x03];

    #[test]
    fn uncompressed_stream_produces_exact_bytes_without_sfnt_length_cap() {
        assert_eq!(decompress(ABC, 3, limits(), 0, 0), Ok(b"abc".to_vec()));
    }

    #[test]
    fn empty_stream_has_bounded_zero_length_output() {
        // WBITS=16, ISLAST=1, ISLASTEMPTY=1, zero padding.
        assert_eq!(decompress(&[0x06], 0, limits(), 0, 0), Ok(Vec::new()));
        assert!(decompress(ABC, 0, limits(), 0, 0).is_err());
    }

    #[test]
    fn repeated_uncompressed_blocks_preserve_stream_order() {
        // After the first aligned block there is no WBITS field; MLEN=0 and
        // ISUNCOMPRESSED=1 give 00 00 08 before the next single literal.
        let input = [0x20, 0, 0x10, b'a', b'b', b'c', 0, 0, 8, b'd', 3];
        assert_eq!(decompress(&input, 4, limits(), 0, 0), Ok(b"abcd".to_vec()));
    }

    #[test]
    fn tiny_nonfinal_block_accepts_maximum_strict_window() {
        // WBITS=24 (1111), followed by a one-byte nonfinal uncompressed block.
        let input = [0x0f, 0, 0x80, b'x', 3];
        assert_eq!(decompress(&input, 1, limits(), 0, 0), Ok(vec![b'x']));
    }

    #[test]
    fn truncation_trailing_data_length_mismatch_and_large_window_are_rejected() {
        for end in 0..ABC.len() {
            assert!(decompress(&ABC[..end], 3, limits(), 0, 0).is_err());
        }
        let mut trailing = ABC.to_vec();
        trailing.push(0);
        assert!(decompress(&trailing, 3, limits(), 0, 0).is_err());
        assert!(decompress(ABC, 2, limits(), 0, 0).is_err());
        assert!(decompress(ABC, 4, limits(), 0, 0).is_err());
        // WBITS escape 0010001 is forbidden by the strict constructor.
        assert!(decompress(&[0x11, 0], 0, limits(), 0, 0).is_err());
    }

    #[test]
    fn complete_inventory_and_caller_usage_are_checked_before_preparation() {
        let required = Budget::new(3, limits(), 0, 0).unwrap().adapter_bytes;
        let workspace = DecodeLimits::new(0, 0, 0, 0, 0, required + 6, usize::MAX);
        assert_eq!(
            decompress(ABC, 3, workspace, 7, 7),
            Err(DecodeError::LimitExceeded {
                kind: LimitKind::WorkspaceBytes,
                required: required + 7,
                limit: required + 6,
            })
        );
        let live = DecodeLimits::new(0, 0, 0, 0, 0, usize::MAX, required + 8);
        assert_eq!(
            decompress(ABC, 3, live, 9, 7),
            Err(DecodeError::LimitExceeded {
                kind: LimitKind::LiveBytes,
                required: required + 9,
                limit: required + 8,
            })
        );
        assert_eq!(
            decompress(ABC, usize::MAX, limits(), 0, 0),
            Err(DecodeError::SizeOverflow)
        );
    }

    #[test]
    fn allocator_reported_excess_capacity_is_charged_before_initialization() {
        let required = Budget::new(0, limits(), 0, 0).unwrap().adapter_bytes;
        let limits = DecodeLimits::new(0, 0, 0, 0, 0, required, usize::MAX);
        let mut budget = Budget::new(0, limits, 0, 0).unwrap();
        assert_eq!(
            budget.capacity::<u32>(1, 2),
            Err(DecodeError::LimitExceeded {
                kind: LimitKind::WorkspaceBytes,
                required: required + size_of::<u32>(),
                limit: required,
            })
        );
    }

    #[test]
    fn checked_out_cells_have_exact_lengths_and_recycle_without_capacity_growth() {
        let mut budget = Budget::new(0, limits(), 0, 0).unwrap();
        let mut pool: Pool<u8, 4> = Pool::prepare(BYTE_SLOTS, &mut budget).unwrap();
        // A small ring displaces the later literal map into the largest slot.
        let mut ring = pool.alloc_cell(1088);
        let pointer = ring.storage.as_ptr();
        let capacity = ring.storage.capacity();
        assert_eq!(ring.slice().len(), 1088);
        ring.slice_mut().fill(7);
        for _ in 0..3 {
            let modes = pool.alloc_cell(256);
            let literal_map = pool.alloc_cell(16_384);
            let distance_map = pool.alloc_cell(1024);
            assert_eq!(literal_map.slice().len(), 16_384);
            assert_eq!(distance_map.slice().len(), 1024);
            pool.free_cell(distance_map);
            pool.free_cell(literal_map);
            pool.free_cell(modes);
        }
        pool.free_cell(ring);
        let ring = pool.alloc_cell(1088);
        assert_eq!(ring.storage.as_ptr(), pointer);
        assert_eq!(ring.storage.capacity(), capacity);
        assert!(ring.slice().iter().all(|byte| *byte == 0));
        pool.free_cell(ring);
        pool.free_cell(Cell::default());
    }

    #[test]
    fn every_map_request_order_fits_after_small_or_maximum_ring_requests() {
        let mut budget = Budget::new(0, limits(), 0, 0).unwrap();
        let mut pool: Pool<u8, 4> = Pool::prepare(BYTE_SLOTS, &mut budget).unwrap();
        let orders = [
            [256, 16_384, 1024],
            [256, 1024, 16_384],
            [16_384, 256, 1024],
            [16_384, 1024, 256],
            [1024, 256, 16_384],
            [1024, 16_384, 256],
        ];
        // Final-block shrinking can make the ring smaller than a map. A
        // nonfinal 24-bit-window ring instead occupies the largest slot.
        for ring_len in [67, 1090, 16_450, 16_777_282] {
            let ring = pool.alloc_cell(ring_len);
            for order in orders {
                let cells = order.map(|len| pool.alloc_cell(len));
                for (cell, len) in cells.into_iter().zip(order) {
                    assert_eq!(cell.slice().len(), len);
                    pool.free_cell(cell);
                }
            }
            pool.free_cell(ring);
        }
    }

    #[test]
    fn complete_huffman_and_offset_inventory_can_be_reused_across_blocks() {
        let mut budget = Budget::new(0, limits(), 0, 0).unwrap();
        let mut codes: Pool<HuffmanCode, 6> = Pool::prepare(CODE_SLOTS, &mut budget).unwrap();
        let mut offsets: Pool<u32, 3> = Pool::prepare(OFFSET_SLOTS, &mut budget).unwrap();
        let context = codes.alloc_cell(1080);
        let block_type = codes.alloc_cell(3240);
        let block_length = codes.alloc_cell(3240);
        for _ in 0..3 {
            let trees = std::array::from_fn::<_, 3, _>(|_| codes.alloc_cell(276_480));
            let indices = std::array::from_fn::<_, 3, _>(|_| offsets.alloc_cell(256));
            for tree in trees {
                assert_eq!(tree.slice().len(), 276_480);
                codes.free_cell(tree);
            }
            for index in indices {
                assert_eq!(index.slice().len(), 256);
                offsets.free_cell(index);
            }
        }
        codes.free_cell(context);
        codes.free_cell(block_type);
        codes.free_cell(block_length);
    }
}
