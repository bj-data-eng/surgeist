//! Exact UTF-16 strings as shared slices of immutable text blocks.

use std::{
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    ops::Range,
    sync::{Arc, Weak},
};

/// The `len` field of a handle that covers its whole block.
const WHOLE: u32 = u32::MAX;

/// A shared, immutable block of UTF-16 code units that [`DocumentString`] handles index by range.
///
/// Cloning shares the block. Its units never change while another holder has it, so every handle
/// observes the same units for as long as it lives. Isolated surrogates, whitespace and null code
/// units are preserved.
#[derive(Clone, Debug, Default)]
pub struct TextBlock {
    units: Arc<Vec<u16>>,
}

impl TextBlock {
    /// The longest range of a block that a handle other than the whole block can represent.
    pub const MAX_RANGE_UNITS: usize = (WHOLE - 1) as usize;

    /// Creates a block that adopts `units` without copying or normalizing them. The block keeps
    /// the vector's spare capacity.
    pub fn from_utf16(units: impl Into<Vec<u16>>) -> Self {
        Self {
            units: Arc::new(units.into()),
        }
    }

    /// Adopts a shared vector of units, for example one a CSS input holds, without copying it.
    fn from_shared_utf16(units: Arc<Vec<u16>>) -> Self {
        Self { units }
    }

    /// Returns every unit of the block.
    pub fn units(&self) -> &[u16] {
        &self.units
    }

    /// Returns the number of units in the block.
    pub fn len(&self) -> usize {
        self.units.len()
    }

    /// Reports whether the block holds no units.
    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    /// Returns the number of units the block's allocation holds, including spare capacity.
    pub fn capacity(&self) -> usize {
        self.units.capacity()
    }

    /// Takes the units back from a block that no other holder has, or returns the block when
    /// another holder keeps it.
    pub(crate) fn into_units(self) -> Result<Vec<u16>, Self> {
        Arc::try_unwrap(self.units).map_err(|units| Self { units })
    }

    /// Returns the units of `range`, or `None` when it is reversed or leaves the block.
    pub fn get(&self, range: Range<usize>) -> Option<&[u16]> {
        self.units.get(range)
    }

    /// Returns a handle on the whole block that owns it: the form of a string made by
    /// [`DocumentString::from_utf16`], which accounts for the block's whole capacity itself.
    ///
    /// Use [`Self::string`] for a handle that shares the block with other handles.
    pub fn whole(&self) -> DocumentString {
        DocumentString {
            block: self.clone(),
            start: 0,
            len: WHOLE,
        }
    }

    /// Returns a handle on the units of `range`, a borrower of the block.
    ///
    /// The range must lie in the block, start at most at `u32::MAX` and have at most
    /// [`Self::MAX_RANGE_UNITS`] units. A range that covers the whole block is a borrower like any
    /// other. An owner that lists the blocks of its borrowers charges each block once and each
    /// borrower by [`DocumentString::occurrence_units`] (see [`TextBlockTable`]); any other holder
    /// keeps charging [`DocumentString::retained_units`], the capacity of the block.
    pub fn string(&self, range: Range<usize>) -> Result<DocumentString, TextRangeError> {
        let (start, len) = range_fields(range.start, range.end, self.units.len())?;
        Ok(DocumentString {
            block: self.clone(),
            start,
            len,
        })
    }

    /// Returns the bytes the block keeps alive: its [capacity](Self::capacity) and the vector
    /// header that shares it. A holder of the block accounts for these bytes and that capacity in
    /// units once.
    /// The value saturates, so a block too large to measure compares as the largest extent.
    pub fn retained_bytes(&self) -> usize {
        self.units
            .capacity()
            .saturating_mul(size_of::<u16>())
            .saturating_add(size_of::<Vec<u16>>())
    }

    /// Reports whether both are the same allocation.
    pub fn shares_allocation_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.units, &other.units)
    }

    /// Returns an identity for the allocation, equal for the clones of one block and different
    /// for blocks alive at the same time. While a [`WeakTextBlock`] of the block exists, no other
    /// block has this address, because the allocation outlives its units until the last weak
    /// reference is dropped. Without one, a block allocated after this one is freed may reuse it.
    pub fn address(&self) -> usize {
        Arc::as_ptr(&self.units) as usize
    }

    /// Returns the number of strong holders of the allocation, this one included.
    pub fn holders(&self) -> usize {
        Arc::strong_count(&self.units)
    }

    /// Returns the number of weak holders of the allocation.
    pub fn weak_holders(&self) -> usize {
        Arc::weak_count(&self.units)
    }

    /// Returns a reference that does not keep the units alive.
    pub fn downgrade(&self) -> WeakTextBlock {
        WeakTextBlock {
            units: Arc::downgrade(&self.units),
        }
    }
}

/// A reference to a [`TextBlock`] that does not keep its units alive.
#[derive(Clone, Debug)]
pub struct WeakTextBlock {
    units: Weak<Vec<u16>>,
}

impl WeakTextBlock {
    /// Returns the block when a strong holder keeps it alive.
    pub fn upgrade(&self) -> Option<TextBlock> {
        self.units.upgrade().map(|units| TextBlock { units })
    }

    /// Returns the number of strong holders, zero once the units are freed.
    pub fn holders(&self) -> usize {
        self.units.strong_count()
    }

    /// Returns the identity [`TextBlock::address`] gives the block. It identifies this block for as
    /// long as this reference exists, even after [`Self::holders`] reaches zero, because a weak
    /// reference keeps the allocation, and so its address, from being reused.
    pub fn address(&self) -> usize {
        self.units.as_ptr() as usize
    }
}

/// The blocks one owner has accounted for, each with the value its owner keeps for it.
///
/// An owner of many borrowers charges a block once, when its table first meets a borrower of the
/// block, and releases the charge when no string holds the block any more. Entries are sorted by
/// block address. Each entry holds a [`WeakTextBlock`], which keeps the block's allocation, and so
/// its address, from being reused while the entry exists: a block found by address is the block the
/// entry was made for, and a block freed while listed can be neither mistaken for a newer block
/// nor charged to it. [`Self::sweep`] removes the entries of blocks that no string holds.
#[derive(Debug)]
pub struct TextBlockTable<T> {
    entries: Vec<TableEntry<T>>,
}

#[derive(Debug)]
struct TableEntry<T> {
    address: usize,
    block: WeakTextBlock,
    value: T,
}

/// A refusal to list a block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextBlockTableError {
    /// The block is listed already.
    Present,
    /// The table could not reserve room for the entry.
    AllocationFailed,
}

impl fmt::Display for TextBlockTableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Present => formatter.write_str("the text block is listed already"),
            Self::AllocationFailed => {
                formatter.write_str("the text block table could not reserve an entry")
            }
        }
    }
}

impl Error for TextBlockTableError {}

impl<T> Default for TextBlockTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TextBlockTable<T> {
    /// Makes an empty table, which allocates nothing.
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// The bytes one entry takes in the table, and so the bytes an owner counts for each entry of
    /// capacity: the DOM store charges the table by its capacity, in one debit that grows with the
    /// table beside the debit of each block, and a capture counts the capacity of its table when
    /// it measures its retained storage.
    pub const ENTRY_BYTES: usize = size_of::<TableEntry<T>>();

    /// Returns the number of listed blocks.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Reports whether no block is listed.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The position of the entry of `address`, or the position it would take. The positions come
    /// from `binary_search`, which keeps them within the entries.
    fn position(&self, address: usize) -> Result<usize, usize> {
        self.entries
            .binary_search_by_key(&address, |entry| entry.address)
    }

    /// Returns the value kept for `block`, which is listed only if it is the block of an entry:
    /// equal addresses of a listed block name one allocation.
    pub fn find(&self, block: &TextBlock) -> Option<&T> {
        let position = self.position(block.address()).ok()?;
        self.entries.get(position).map(|entry| &entry.value)
    }

    /// Returns the number of entries the table holds room for.
    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    /// Returns the value kept for `block` for change, under the same rule as [`Self::find`].
    pub fn find_mut(&mut self, block: &TextBlock) -> Option<&mut T> {
        let position = self.position(block.address()).ok()?;
        self.entries.get_mut(position).map(|entry| &mut entry.value)
    }

    /// Reserves room for `additional` more entries, so that as many [`Self::insert`] calls cannot
    /// refuse for lack of room. A refusal leaves the table as it was.
    pub fn reserve(&mut self, additional: usize) -> Result<(), TextBlockTableError> {
        self.entries
            .try_reserve(additional)
            .map_err(|_| TextBlockTableError::AllocationFailed)
    }

    /// Reserves room for exactly `additional` more entries, so that an owner that charges the
    /// table by its capacity knows the capacity it reserves. A refusal leaves the table as it was.
    pub fn reserve_exact(&mut self, additional: usize) -> Result<(), TextBlockTableError> {
        self.entries
            .try_reserve_exact(additional)
            .map_err(|_| TextBlockTableError::AllocationFailed)
    }

    /// Iterates the values of the entries in block address order.
    pub fn values(&self) -> impl ExactSizeIterator<Item = &T> {
        self.entries.iter().map(|entry| &entry.value)
    }

    /// Returns the number of entries after the one `block` would take, which an insertion moves.
    /// An owner that meters its work charges this number before it calls [`Self::insert`].
    pub fn shifted_by_insert(&self, block: &TextBlock) -> usize {
        let position = match self.position(block.address()) {
            Ok(position) | Err(position) => position,
        };
        self.entries.iter().skip(position).count()
    }

    /// Lists `block` with `value`, reserving room for the entry first. A refusal drops `value`
    /// and leaves the table as it was.
    ///
    /// The entries after the new one move up by one place, so an insertion takes time in
    /// proportion to [`Self::shifted_by_insert`]; the table keeps the few blocks of one owner,
    /// and an owner that meters its work charges that number first.
    pub fn insert(&mut self, block: &TextBlock, value: T) -> Result<(), TextBlockTableError> {
        let address = block.address();
        let Err(position) = self.position(address) else {
            return Err(TextBlockTableError::Present);
        };
        self.entries
            .try_reserve(1)
            .map_err(|_| TextBlockTableError::AllocationFailed)?;
        self.entries.insert(
            position,
            TableEntry {
                address,
                block: block.downgrade(),
                value,
            },
        );
        Ok(())
    }

    /// Keeps the entries whose value `keep` accepts and drops the others, which release what they
    /// hold. An owner rolls back to a checkpoint by dropping the entries made after it.
    pub fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        self.entries.retain(|entry| keep(&entry.value));
    }

    /// Removes the entry of `block` and returns its value.
    pub fn remove(&mut self, block: &TextBlock) -> Option<T> {
        let position = self.position(block.address()).ok()?;
        Some(self.entries.remove(position).value)
    }

    /// Removes every entry whose block no string holds any more and hands its value to
    /// `release`, in address order.
    pub fn sweep(&mut self, mut release: impl FnMut(T)) {
        for entry in self
            .entries
            .extract_if(.., |entry| entry.block.holders() == 0)
        {
            release(entry.value);
        }
    }
}

/// Returns the `start` and `len` fields of a handle on `start..end` in a block of `block_len`
/// units.
///
/// A range needs a start that fits `u32` and a length below [`WHOLE`], which marks the handle that
/// owns its block.
fn range_fields(start: usize, end: usize, block_len: usize) -> Result<(u32, u32), TextRangeError> {
    if start > end {
        return Err(TextRangeError::Reversed { start, end });
    }
    if end > block_len {
        return Err(TextRangeError::OutOfBounds { end, block_len });
    }
    let len = end - start;
    match (u32::try_from(start), u32::try_from(len)) {
        (Ok(start), Ok(len)) if len != WHOLE => Ok((start, len)),
        _ => Err(TextRangeError::Unrepresentable { start, len }),
    }
}

/// A range that [`TextBlock::string`] cannot turn into a handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextRangeError {
    /// The start follows the end.
    Reversed {
        /// The range start, in code units.
        start: usize,
        /// The range end, in code units.
        end: usize,
    },
    /// The end follows the last unit of the block.
    OutOfBounds {
        /// The range end, in code units.
        end: usize,
        /// The number of units in the block.
        block_len: usize,
    },
    /// The range lies in the block but its start or length does not fit a handle.
    Unrepresentable {
        /// The range start, in code units.
        start: usize,
        /// The range length, in code units.
        len: usize,
    },
}

impl fmt::Display for TextRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reversed { start, end } => {
                write!(formatter, "text range {start}..{end} ends before it starts")
            }
            Self::OutOfBounds { end, block_len } => write!(
                formatter,
                "text range ends at {end}, past the {block_len} code units of its block"
            ),
            Self::Unrepresentable { start, len } => write!(
                formatter,
                "text range of {len} code units starting at {start} does not fit a string handle"
            ),
        }
    }
}

impl Error for TextRangeError {}

/// An immutable sequence of exact UTF-16 code units.
///
/// The value is a 16-byte handle: a [`TextBlock`] and the range of the block it covers. A handle is
/// an owner when it covers the block it was made with ([`Self::from_utf16`], [`TextBlock::whole`])
/// and a borrower when it is a range made by [`TextBlock::string`].
/// Cloning copies the handle and shares the block. Equality and hashing compare the units, not
/// the block or the position in it.
///
/// Isolated surrogates, whitespace, and null code units are preserved. Lengths
/// and conversion-error offsets use code units, not Unicode scalar values.
#[derive(Clone)]
pub struct DocumentString {
    block: TextBlock,
    /// The first unit of the range. Zero when `len` is [`WHOLE`].
    start: u32,
    /// The length of the range, or [`WHOLE`] for the whole block however long it is.
    len: u32,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<DocumentString>() == 16);

impl DocumentString {
    /// Constructs a string without validating, replacing, or normalizing units.
    ///
    /// The string owns a private block of exactly these units, so an authored string is one
    /// allocation.
    pub fn from_utf16(code_units: impl Into<Vec<u16>>) -> Self {
        TextBlock::from_utf16(code_units).whole()
    }

    /// Returns the exact stored UTF-16 sequence.
    pub fn code_units(&self) -> &[u16] {
        if self.len == WHOLE {
            &self.block.units
        } else {
            // A range handle is built only by `TextBlock::string`, which checked it against the
            // block, and a block with another holder never changes.
            &self.block.units[self.range()]
        }
    }

    /// Returns the number of UTF-16 code units.
    pub fn len(&self) -> usize {
        if self.len == WHOLE {
            self.block.units.len()
        } else {
            self.len as usize
        }
    }

    /// Reports whether there are no code units.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the block this string indexes.
    pub fn block(&self) -> &TextBlock {
        &self.block
    }

    /// Returns the range of [`Self::block`] this string covers.
    pub fn range(&self) -> Range<usize> {
        let start = self.start as usize;
        start..start + self.len()
    }

    /// Reports whether this string owns its block, as a string made by [`Self::from_utf16`] does.
    /// A range made by [`TextBlock::string`] is not, even when it covers every unit.
    pub fn is_whole_block(&self) -> bool {
        self.len == WHOLE
    }

    /// Reports whether two strings are the same range of the same block allocation.
    ///
    /// Equal content in separate allocations is not shared storage.
    pub fn shares_storage_with(&self, other: &Self) -> bool {
        self.block.shares_allocation_with(&other.block)
            && self.start == other.start
            && self.len == other.len
    }

    /// Returns the number of units the string keeps alive: the capacity of its block, spare
    /// capacity included, however much of the block it covers. A holder that does not list the
    /// block of a borrower charges this, which can never charge less than the memory it holds.
    pub fn retained_units(&self) -> usize {
        self.block.capacity()
    }

    /// Returns the number of units this string accounts for when its holder charges the block of
    /// a borrower separately, once for all the borrowers it holds (see [`TextBlockTable`]): the
    /// length of a borrower, and the capacity of the block for an owner. Charge this only where
    /// the block is listed, and [`Self::retained_units`] everywhere else.
    pub fn occurrence_units(&self) -> usize {
        if self.len == WHOLE {
            self.block.capacity()
        } else {
            self.len as usize
        }
    }

    /// Returns the block this string borrows, or `None` for an owner. An owner is charged the
    /// capacity of its block at every occurrence.
    pub fn borrowed_block(&self) -> Option<&TextBlock> {
        (self.len != WHOLE).then_some(&self.block)
    }

    /// Returns the units of a string that nothing else observes, for the construction owner to
    /// change in place.
    ///
    /// This is `Some` only when the string covers its whole block and no other string or weak
    /// reference holds the block. Any other string returns `None`, and its units never change.
    pub fn unique_units_mut(&mut self) -> Option<&mut Vec<u16>> {
        if self.len != WHOLE {
            return None;
        }
        Arc::get_mut(&mut self.block.units)
    }

    /// Adopts a shared vector of units, for example one a CSS input holds, without copying it.
    /// The string covers the whole block.
    pub fn from_shared_utf16(units: Arc<Vec<u16>>) -> Self {
        TextBlock::from_shared_utf16(units).whole()
    }

    /// Returns the units as a shared vector, for interfaces that take `Arc<Vec<u16>>`.
    ///
    /// An owner hands over the block's own allocation without copying. A borrower, including a
    /// range over every unit of its block, copies its units into a new vector that has no storage
    /// in common with the block.
    pub fn to_shared_utf16(&self) -> Arc<Vec<u16>> {
        if self.len == WHOLE {
            Arc::clone(&self.block.units)
        } else {
            Arc::new(self.code_units().to_vec())
        }
    }

    /// Iterates code points, combining surrogate pairs and retaining isolated
    /// surrogates as their numeric code points.
    ///
    /// Unlike an iterator of Rust `char`, this can represent every input unit.
    pub fn code_points(&self) -> impl Iterator<Item = u32> + Clone + '_ {
        char::decode_utf16(self.code_units().iter().copied()).map(|decoded| match decoded {
            Ok(scalar) => u32::from(scalar),
            Err(error) => u32::from(error.unpaired_surrogate()),
        })
    }

    /// Converts to UTF-8, rejecting the first isolated surrogate without loss.
    pub fn to_utf8(&self) -> Result<String, Utf8ConversionError> {
        let mut result = String::new();
        let mut code_unit_offset = 0;
        for decoded in char::decode_utf16(self.code_units().iter().copied()) {
            match decoded {
                Ok(scalar) => {
                    result.push(scalar);
                    code_unit_offset += scalar.len_utf16();
                }
                Err(error) => {
                    return Err(Utf8ConversionError {
                        code_unit_offset,
                        surrogate: error.unpaired_surrogate(),
                    });
                }
            }
        }
        Ok(result)
    }
}

impl Default for DocumentString {
    fn default() -> Self {
        Self::from_utf16(Vec::new())
    }
}

impl fmt::Debug for DocumentString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DocumentString")
            .field("code_units", &self.code_units())
            .finish()
    }
}

impl PartialEq for DocumentString {
    fn eq(&self, other: &Self) -> bool {
        self.shares_storage_with(other) || self.code_units() == other.code_units()
    }
}

impl Eq for DocumentString {}

impl Hash for DocumentString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.code_units().hash(state);
    }
}

impl From<&str> for DocumentString {
    fn from(value: &str) -> Self {
        Self::from_utf16(value.encode_utf16().collect::<Vec<_>>())
    }
}

impl From<String> for DocumentString {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

/// The first isolated surrogate preventing an exact UTF-8 conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Utf8ConversionError {
    code_unit_offset: usize,
    surrogate: u16,
}

impl Utf8ConversionError {
    /// Returns the zero-based UTF-16 code-unit offset.
    pub fn code_unit_offset(&self) -> usize {
        self.code_unit_offset
    }

    /// Returns the isolated surrogate code unit.
    pub fn surrogate(&self) -> u16 {
        self.surrogate
    }
}

impl fmt::Display for Utf8ConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "isolated surrogate U+{:04X} at UTF-16 code-unit offset {}",
            self.surrogate, self.code_unit_offset
        )
    }
}

impl Error for Utf8ConversionError {}

#[cfg(test)]
mod tests;
