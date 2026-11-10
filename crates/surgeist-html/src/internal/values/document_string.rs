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
    pub fn from_shared_utf16(units: Arc<Vec<u16>>) -> Self {
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

    /// Returns the units of `range`, or `None` when it is reversed or leaves the block.
    pub fn get(&self, range: Range<usize>) -> Option<&[u16]> {
        self.units.get(range)
    }

    /// Returns a handle on the whole block. Every block has this representation.
    pub fn whole(&self) -> DocumentString {
        DocumentString {
            block: self.clone(),
            start: 0,
            len: WHOLE,
        }
    }

    /// Returns a handle on the units of `range`.
    ///
    /// The range must lie in the block. A range that is not the whole block also has to fit the
    /// handle's 32-bit start and have at most [`Self::MAX_RANGE_UNITS`] units.
    pub fn string(&self, range: Range<usize>) -> Result<DocumentString, TextRangeError> {
        let Range { start, end } = range;
        if start > end {
            return Err(TextRangeError::Reversed { start, end });
        }
        let block_len = self.units.len();
        if end > block_len {
            return Err(TextRangeError::OutOfBounds { end, block_len });
        }
        if start == 0 && end == block_len {
            return Ok(self.whole());
        }
        let len = end - start;
        match (u32::try_from(start), u32::try_from(len)) {
            (Ok(start), Ok(len)) if len != WHOLE => Ok(DocumentString {
                block: self.clone(),
                start,
                len,
            }),
            _ => Err(TextRangeError::Unrepresentable { start, len }),
        }
    }

    /// Reports whether both are the same allocation.
    pub fn shares_allocation_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.units, &other.units)
    }

    /// Returns an identity for the allocation, equal for the clones of one block and different
    /// for blocks alive at the same time.
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

    /// Returns the identity [`TextBlock::address`] gives the block.
    pub fn address(&self) -> usize {
        self.units.as_ptr() as usize
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
/// The value is a 16-byte handle: a [`TextBlock`] and the range of the block it covers.
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

    /// Reports whether this string covers its whole block, as a string that owns its allocation
    /// does.
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
    /// capacity included, however much of the block it covers.
    pub fn retained_units(&self) -> usize {
        self.block.capacity()
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
    /// A string that covers its whole block hands over the block's own allocation without
    /// copying. A partial range copies its units into a new vector that has no storage in common
    /// with the block.
    pub fn to_shared_utf16(&self) -> Arc<Vec<u16>> {
        if self.len == WHOLE {
            Arc::clone(&self.block.units)
        } else {
            Arc::new(self.code_units().to_vec())
        }
    }

    /// Returns the allocation of a string that covers its whole block. Crate-internal: the value
    /// pool holds whole blocks only.
    pub(crate) fn whole_block_vec(&self) -> &Arc<Vec<u16>> {
        &self.block.units
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
