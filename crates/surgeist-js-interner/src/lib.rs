//! Boa's **`surgeist_js_interner`** is a string interner for compiler performance.
//!
//! # Crate Overview
//!
//! The idea behind using a string interner is that in most of the code, strings such as
//! identifiers and literals are often repeated. This causes extra burden when comparing them and
//! storing them. A string interner stores a unique `usize` symbol for each string, making sure
//! that there are no duplicates. This makes it much easier to compare, since it's just comparing
//! to `usize`, and also it's easier to store, since instead of a heap-allocated string, you only
//! need to store a `usize`. This reduces memory consumption and improves performance in the
//! compiler.
#![forbid(unsafe_code)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![allow(
    clippy::redundant_pub_crate,
    // TODO deny once false positive is fixed (https://github.com/rust-lang/rust-clippy/issues/9626).
    clippy::trait_duplication_in_bounds,
    // Field names intentionally mirror the encoding type they store.
    clippy::struct_field_names
)]
#![cfg_attr(not(feature = "arbitrary"), no_std)]

extern crate alloc;

mod index;
mod sym;
mod text;

#[cfg(test)]
mod tests;

use alloc::{borrow::Cow, format, string::String, vec, vec::Vec};
use core::{convert::Infallible, ops::Range};

use index::{CommonIndex, Probe, hash_latin1, hash_utf8, hash_utf16, probe};

pub use sym::*;

/// Encoding of the input units in a work event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InternerEncoding {
    /// UTF-8 bytes.
    Utf8,
    /// UTF-16 code units.
    Utf16,
    /// Latin1 bytes.
    Latin1,
}

/// Storage subject to an interner capacity request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InternerBuffer {
    /// Scratch UTF-8 conversion output, measured in bytes.
    TemporaryUtf8,
    /// Scratch UTF-16 conversion output, measured in code units.
    TemporaryUtf16,
    /// Retained UTF-8 spelling storage, measured in bytes.
    Utf8,
    /// Retained UTF-16 spelling storage, measured in code units.
    Utf16,
    /// Retained symbol records, measured in entries.
    Entries,
}

/// Work to admit before an interner operation performs it.
///
/// The caller owns limits and accumulation. These units describe the
/// algorithm's work and storage requests independently of allocator behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InternerWork {
    /// A complete spelling hash, charged by the input's encoded length.
    Hash {
        /// Input encoding.
        encoding: InternerEncoding,
        /// Number of source units hashed.
        units: usize,
    },
    /// One slot inspection, including a vacant slot.
    Probe,
    /// One equality block between spellings of equal length.
    Compare {
        /// Encoding of the requested spelling.
        encoding: InternerEncoding,
        /// Number of unit pairs covered by this block.
        units: usize,
    },
    /// Source units inspected for a spelling conversion.
    Convert {
        /// Source encoding.
        encoding: InternerEncoding,
        /// Number of units to inspect.
        units: usize,
    },
    /// One block of UTF-16 units inspected for Latin1 classification.
    Classify {
        /// Number of units covered by this block.
        units: usize,
    },
    /// Validate the complete input pair to the static spelling API.
    ValidateStatic {
        /// UTF-8 input bytes.
        utf8_bytes: usize,
        /// UTF-16 input code units.
        utf16_units: usize,
    },
    /// Initialize a replacement index with vacant slots.
    InitializeSlots {
        /// Number of slots initialized.
        slots: usize,
    },
    /// Read and place one existing symbol during growth, with probes separate.
    Reinsert,
    /// Request capacity before writing to a buffer with insufficient room.
    Reserve {
        /// Buffer that requires capacity.
        buffer: InternerBuffer,
        /// Minimum additional capacity requested beyond current length.
        additional: usize,
        /// Existing initialized units the allocator may have to relocate.
        existing: usize,
    },
    /// Publish one new symbol, its slot, and any newly copied spelling text.
    Publish {
        /// Newly retained UTF-8 bytes.
        utf8_bytes: usize,
        /// Newly retained UTF-16 code units.
        utf16_units: usize,
    },
}

#[inline]
fn unmetered(_: InternerWork) -> Result<(), Infallible> {
    Ok(())
}

fn infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

/// An enumeration of all slice types [`Interner`] can internally store.
///
/// This struct allows us to intern either `UTF-8` or `UTF-16` str references, which are the two
/// encodings [`Interner`] can store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JStrRef<'a> {
    /// A `UTF-8` string reference.
    Utf8(&'a str),

    /// A `UTF-16` string reference.
    Utf16(&'a [u16]),
}

impl<'a> From<&'a str> for JStrRef<'a> {
    fn from(s: &'a str) -> Self {
        JStrRef::Utf8(s)
    }
}

impl<'a> From<&'a [u16]> for JStrRef<'a> {
    fn from(s: &'a [u16]) -> Self {
        JStrRef::Utf16(s)
    }
}

impl<'a, const N: usize> From<&'a [u16; N]> for JStrRef<'a> {
    fn from(s: &'a [u16; N]) -> Self {
        JStrRef::Utf16(s)
    }
}

#[derive(Clone, Copy)]
enum LookupKey<'a> {
    Encoded(JStrRef<'a>),
    Latin1(&'a [u8]),
}

impl LookupKey<'_> {
    fn hash<E>(self, admit: &mut impl FnMut(InternerWork) -> Result<(), E>) -> Result<u64, E> {
        let (encoding, units) = match self {
            Self::Encoded(JStrRef::Utf8(value)) => (InternerEncoding::Utf8, value.len()),
            Self::Encoded(JStrRef::Utf16(value)) => (InternerEncoding::Utf16, value.len()),
            Self::Latin1(value) => (InternerEncoding::Latin1, value.len()),
        };
        admit(InternerWork::Hash { encoding, units })?;
        Ok(match self {
            Self::Encoded(JStrRef::Utf8(value)) => hash_utf8(value),
            Self::Encoded(JStrRef::Utf16(value)) => hash_utf16(value),
            Self::Latin1(value) => hash_latin1(value),
        })
    }

    fn matches<E>(
        self,
        spelling: JSInternedStrRef<'_, '_>,
        admit: &mut impl FnMut(InternerWork) -> Result<(), E>,
    ) -> Result<bool, E> {
        match self {
            Self::Encoded(JStrRef::Utf8(value)) => {
                if let Some(stored) = spelling.utf8() {
                    text::equal(
                        value.as_bytes(),
                        stored.as_bytes(),
                        InternerEncoding::Utf8,
                        admit,
                    )
                } else {
                    Ok(false)
                }
            }
            Self::Encoded(JStrRef::Utf16(value)) => {
                text::equal(value, spelling.utf16(), InternerEncoding::Utf16, admit)
            }
            Self::Latin1(value) => text::equal_latin1(value, spelling.utf16(), admit),
        }
    }
}

/// A double reference to an interned string inside [`Interner`].
///
/// [`JSInternedStrRef::utf8`] returns an [`Option`], since not every `UTF-16` string is fully
/// representable as a `UTF-8` string (because of unpaired surrogates). However, every `UTF-8`
/// string is representable as a `UTF-16` string, so `JSInternedStrRef::utf8` returns a
/// [<code>&\[u16\]</code>][core::slice].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JSInternedStrRef<'a, 'b> {
    utf8: Option<&'a str>,
    utf16: &'b [u16],
}

impl<'a, 'b> JSInternedStrRef<'a, 'b> {
    /// Returns the inner reference to the interned string in `UTF-8` encoding.
    /// if the string is not representable in `UTF-8`, returns [`None`]
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let interned = interner.resolve_expect(sym);
    /// assert_eq!(interned.utf8(), Some("hello"));
    /// ```
    #[inline]
    #[must_use]
    pub const fn utf8(&self) -> Option<&'a str> {
        self.utf8
    }

    /// Returns the inner reference to the interned string in `UTF-16` encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let interned = interner.resolve_expect(sym);
    /// let utf16: Vec<u16> = "hello".encode_utf16().collect();
    /// assert_eq!(interned.utf16(), utf16.as_slice());
    /// ```
    #[inline]
    #[must_use]
    pub const fn utf16(&self) -> &'b [u16] {
        self.utf16
    }

    /// Joins the result of both possible strings into a common type.
    ///
    /// If `self` is representable by a `UTF-8` string and the `prioritize_utf8` argument is set,
    /// it will prioritize calling `f`, and will only call `g` if `self` is only representable by a
    /// `UTF-16` string. Otherwise, it will directly call `g`.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let interned = interner.resolve_expect(sym);
    /// let result = interned.join(
    ///     |utf8| utf8.to_uppercase(),
    ///     |utf16| String::from_utf16_lossy(utf16).to_uppercase(),
    ///     true,
    /// );
    /// assert_eq!(result, "HELLO");
    /// ```
    pub fn join<F, G, T>(self, f: F, g: G, prioritize_utf8: bool) -> T
    where
        F: FnOnce(&'a str) -> T,
        G: FnOnce(&'b [u16]) -> T,
    {
        if prioritize_utf8 && let Some(str) = self.utf8 {
            return f(str);
        }
        g(self.utf16)
    }

    /// Same as [`join`][`JSInternedStrRef::join`], but where you can pass an additional context.
    ///
    /// Useful when you have a `&mut Context` context that cannot be borrowed by both closures at
    /// the same time.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let interned = interner.resolve_expect(sym);
    /// let mut output = String::new();
    /// interned.join_with_context(
    ///     |utf8, buf: &mut String| buf.push_str(&utf8.to_uppercase()),
    ///     |utf16, buf: &mut String| buf.push_str(&String::from_utf16_lossy(utf16).to_uppercase()),
    ///     &mut output,
    ///     true,
    /// );
    /// assert_eq!(output, "HELLO");
    /// ```
    pub fn join_with_context<C, F, G, T>(self, f: F, g: G, ctx: C, prioritize_utf8: bool) -> T
    where
        F: FnOnce(&'a str, C) -> T,
        G: FnOnce(&'b [u16], C) -> T,
    {
        if prioritize_utf8 && let Some(str) = self.utf8 {
            return f(str, ctx);
        }
        g(self.utf16, ctx)
    }

    /// Converts both string types into a common type `C`.
    ///
    /// If `self` is representable by a `UTF-8` string and the `prioritize_utf8` argument is set, it
    /// will prioritize converting its `UTF-8` representation first, and will only convert its
    /// `UTF-16` representation if it is only representable by a `UTF-16` string. Otherwise, it will
    /// directly convert its `UTF-16` representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// enum JsString<'a> {
    ///     Utf8(&'a str),
    ///     Utf16(&'a [u16]),
    /// }
    ///
    /// impl<'a> From<&'a str> for JsString<'a> {
    ///     fn from(s: &'a str) -> Self {
    ///         JsString::Utf8(s)
    ///     }
    /// }
    ///
    /// impl<'a> From<&'a [u16]> for JsString<'a> {
    ///     fn from(s: &'a [u16]) -> Self {
    ///         JsString::Utf16(s)
    ///     }
    /// }
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let interned = interner.resolve_expect(sym);
    /// let result: JsString<'_> = interned.into_common(true);
    /// assert!(matches!(result, JsString::Utf8("hello")));
    /// ```
    pub fn into_common<C>(self, prioritize_utf8: bool) -> C
    where
        C: From<&'a str> + From<&'b [u16]>,
    {
        self.join(Into::into, Into::into, prioritize_utf8)
    }
}

impl core::fmt::Display for JSInternedStrRef<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.join_with_context(
            core::fmt::Display::fmt,
            |js, f| {
                char::decode_utf16(js.iter().copied())
                    .map(|r| match r {
                        Ok(c) => String::from(c),
                        Err(e) => format!("\\u{:04X}", e.unpaired_surrogate()),
                    })
                    .collect::<String>()
                    .fmt(f)
            },
            f,
            true,
        )
    }
}

#[derive(Debug)]
enum StoredText {
    Static {
        utf8: &'static str,
        utf16: &'static [u16],
    },
    Buffered {
        utf8: Option<Range<usize>>,
        utf16: Range<usize>,
    },
}

impl StoredText {
    fn utf8<'a>(&'a self, buffer: &'a str) -> Option<&'a str> {
        match self {
            Self::Static { utf8, .. } => Some(utf8),
            Self::Buffered { utf8, .. } => utf8.as_ref().map(|range| &buffer[range.clone()]),
        }
    }

    fn utf16<'a>(&'a self, buffer: &'a [u16]) -> &'a [u16] {
        match self {
            Self::Static { utf16, .. } => utf16,
            Self::Buffered { utf16, .. } => &buffer[utf16.clone()],
        }
    }
}

enum Spelling<'a> {
    Static {
        utf8: &'static str,
        utf16: &'static [u16],
    },
    Buffered {
        utf8: Option<&'a str>,
        utf16: &'a [u16],
    },
}

enum SlotPlan {
    Existing(usize),
    Replacement {
        slots: Vec<Option<Sym>>,
        vacant: usize,
    },
}

fn admit_reserve<E>(
    buffer: InternerBuffer,
    existing: usize,
    capacity: usize,
    additional: usize,
    admit: &mut impl FnMut(InternerWork) -> Result<(), E>,
) -> Result<(), E> {
    if additional > capacity - existing {
        admit(InternerWork::Reserve {
            buffer,
            additional,
            existing,
        })?;
    }
    Ok(())
}

fn vacant_slot<E>(
    slots: &[Option<Sym>],
    hash: u64,
    admit: &mut impl FnMut(InternerWork) -> Result<(), E>,
) -> Result<usize, E> {
    match probe(slots, hash, admit, |_, _| Ok(false))? {
        Probe::Vacant(slot) => Ok(slot),
        Probe::Found(_) => unreachable!("placement does not match occupied slots"),
    }
}

#[derive(Debug)]
struct Entry {
    text: StoredText,
    latin1: bool,
    hash: u64,
}

const COMMON_COUNT: usize = COMMON_STRINGS_UTF16.len();
static COMMON_INDEX: CommonIndex<COMMON_COUNT, { COMMON_COUNT.next_power_of_two() * 2 }> =
    CommonIndex::new(COMMON_STRINGS_UTF16);

/// The string interner for the JavaScript compiler.
#[derive(Debug, Default)]
pub struct Interner {
    entries: Vec<Entry>,
    slots: Vec<Option<Sym>>,
    utf8_buffer: String,
    utf16_buffer: Vec<u16>,
}

impl Interner {
    /// Creates a new [`Interner`].
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// assert!(interner.resolve(sym).is_some());
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`Interner`] with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::with_capacity(10);
    /// let sym = interner.get_or_intern("hello");
    /// assert!(interner.resolve(sym).is_some());
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            slots: Vec::new(),
            utf8_buffer: String::with_capacity(capacity),
            utf16_buffer: Vec::with_capacity(capacity),
        }
    }

    /// Returns the number of strings interned by the interner.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let initial_len = interner.len();
    /// interner.get_or_intern("hello");
    /// assert_eq!(interner.len(), initial_len + 1);
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        COMMON_STRINGS_UTF8.len() + self.entries.len()
    }

    /// Returns `true` if the [`Interner`] contains no interned strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let interner = Interner::new();
    /// assert!(!interner.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        COMMON_STRINGS_UTF8.is_empty() && self.entries.is_empty()
    }

    /// Returns the symbol for the given string if any.
    ///
    /// Can be used to query if a string has already been interned without interning.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// assert!(interner.get("hello").is_none());
    /// interner.get_or_intern("hello");
    /// assert!(interner.get("hello").is_some());
    /// ```
    pub fn get<'a, T>(&self, string: T) -> Option<Sym>
    where
        T: Into<JStrRef<'a>>,
    {
        infallible(self.try_get(string, unmetered))
    }

    /// Looks up an encoded spelling under the caller's admission policy.
    ///
    /// # Errors
    ///
    /// Returns the first refused work event.
    pub fn try_get<'a, E>(
        &self,
        string: impl Into<JStrRef<'a>>,
        mut admit: impl FnMut(InternerWork) -> Result<(), E>,
    ) -> Result<Option<Sym>, E> {
        self.get_key(LookupKey::Encoded(string.into()), &mut admit)
    }

    /// Looks up a borrowed Latin1 spelling without widening or copying it.
    #[must_use]
    pub fn get_latin1(&self, string: &[u8]) -> Option<Sym> {
        infallible(self.try_get_latin1(string, unmetered))
    }

    /// Looks up a borrowed Latin1 spelling under the caller's admission policy.
    ///
    /// # Errors
    ///
    /// Returns the first refused work event.
    pub fn try_get_latin1<E>(
        &self,
        string: &[u8],
        mut admit: impl FnMut(InternerWork) -> Result<(), E>,
    ) -> Result<Option<Sym>, E> {
        self.get_key(LookupKey::Latin1(string), &mut admit)
    }

    fn get_key<E>(
        &self,
        key: LookupKey<'_>,
        admit: &mut impl FnMut(InternerWork) -> Result<(), E>,
    ) -> Result<Option<Sym>, E> {
        let hash = key.hash(admit)?;
        Ok(match self.lookup(key, hash, admit)? {
            Probe::Found(symbol) => Some(symbol),
            Probe::Vacant(_) => None,
        })
    }

    fn lookup<E>(
        &self,
        key: LookupKey<'_>,
        hash: u64,
        admit: &mut impl FnMut(InternerWork) -> Result<(), E>,
    ) -> Result<Probe, E> {
        let common = probe(&COMMON_INDEX.slots, hash, admit, |symbol, admit| {
            let index = symbol.get() - 1;
            if COMMON_INDEX.hashes[index] != hash {
                return Ok(false);
            }
            key.matches(
                JSInternedStrRef {
                    utf8: Some(COMMON_STRINGS_UTF8[index]),
                    utf16: COMMON_STRINGS_UTF16[index],
                },
                admit,
            )
        })?;
        if matches!(common, Probe::Found(_)) {
            return Ok(common);
        }

        probe(&self.slots, hash, admit, |symbol, admit| {
            let entry = &self.entries[symbol.get() - 1 - COMMON_COUNT];
            if entry.hash != hash {
                return Ok(false);
            }
            key.matches(
                JSInternedStrRef {
                    utf8: entry.text.utf8(&self.utf8_buffer),
                    utf16: entry.text.utf16(&self.utf16_buffer),
                },
                admit,
            )
        })
    }

    /// Interns the given string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible by the chosen symbol type.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym1 = interner.get_or_intern("hello");
    /// let sym2 = interner.get_or_intern("hello");
    /// assert_eq!(sym1, sym2);
    /// let sym3 = interner.get_or_intern("world");
    /// assert_ne!(sym1, sym3);
    /// ```
    pub fn get_or_intern<'a, T>(&mut self, string: T) -> Sym
    where
        T: Into<JStrRef<'a>>,
    {
        infallible(self.try_get_or_intern(string, unmetered))
    }

    /// Interns an encoded spelling under the caller's admission policy.
    ///
    /// # Errors
    ///
    /// Returns the first refused work event. Refusal preserves the live
    /// spellings, probe layout, and next symbol.
    ///
    /// # Panics
    ///
    /// Panics if the next symbol or storage size is not representable.
    pub fn try_get_or_intern<'a, E>(
        &mut self,
        string: impl Into<JStrRef<'a>>,
        mut admit: impl FnMut(InternerWork) -> Result<(), E>,
    ) -> Result<Sym, E> {
        let string = string.into();
        let key = LookupKey::Encoded(string);
        let hash = key.hash(&mut admit)?;
        let slot = match self.lookup(key, hash, &mut admit)? {
            Probe::Found(symbol) => return Ok(symbol),
            Probe::Vacant(slot) => slot,
        };
        let (utf8, utf16) = match string {
            JStrRef::Utf8(value) => (
                Some(Cow::Borrowed(value)),
                Cow::Owned(text::encode_utf16(value, &mut admit)?),
            ),
            JStrRef::Utf16(value) => (
                text::encode_utf8(value, &mut admit)?.map(Cow::Owned),
                Cow::Borrowed(value),
            ),
        };
        self.insert(
            Spelling::Buffered {
                utf8: utf8.as_deref(),
                utf16: &utf16,
            },
            hash,
            slot,
            &mut admit,
        )
    }

    /// Interns the given `'static` string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Note
    ///
    /// Stores references to both encodings without copying their contents.
    ///
    /// # Panics
    ///
    /// If the encodings differ or there are no more representable symbol IDs.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// static HELLO_UTF16: &[u16] = &[0x68, 0x65, 0x6C, 0x6C, 0x6F];
    ///
    /// let mut interner = Interner::new();
    /// let sym1 = interner.get_or_intern_static("hello", HELLO_UTF16);
    /// let sym2 = interner.get_or_intern("hello");
    /// assert_eq!(sym1, sym2);
    /// ```
    pub fn get_or_intern_static(&mut self, utf8: &'static str, utf16: &'static [u16]) -> Sym {
        infallible(self.try_get_or_intern_static(utf8, utf16, unmetered))
    }

    /// Interns a static spelling pair under the caller's admission policy.
    ///
    /// # Errors
    ///
    /// Returns the first refused work event without publishing a new symbol.
    ///
    /// # Panics
    ///
    /// Panics if the admitted encodings differ or the next symbol or storage
    /// size is not representable.
    pub fn try_get_or_intern_static<E>(
        &mut self,
        utf8: &'static str,
        utf16: &'static [u16],
        mut admit: impl FnMut(InternerWork) -> Result<(), E>,
    ) -> Result<Sym, E> {
        admit(InternerWork::ValidateStatic {
            utf8_bytes: utf8.len(),
            utf16_units: utf16.len(),
        })?;
        assert!(
            utf8.encode_utf16().eq(utf16.iter().copied()),
            "static UTF-8 and UTF-16 spellings must match"
        );
        let hash = LookupKey::Encoded(JStrRef::Utf16(utf16)).hash(&mut admit)?;
        match self.lookup(LookupKey::Encoded(JStrRef::Utf8(utf8)), hash, &mut admit)? {
            Probe::Found(symbol) => Ok(symbol),
            Probe::Vacant(slot) => {
                self.insert(Spelling::Static { utf8, utf16 }, hash, slot, &mut admit)
            }
        }
    }

    fn dynamic_symbol(index: usize) -> Option<Sym> {
        index.checked_add(1 + COMMON_COUNT).and_then(Sym::new)
    }

    fn prepare_slots<E>(
        &self,
        hash: u64,
        current_slot: usize,
        admit: &mut impl FnMut(InternerWork) -> Result<(), E>,
    ) -> Result<SlotPlan, E> {
        if self.entries.len() < self.slots.len() - self.slots.len() / 4 {
            return Ok(SlotPlan::Existing(current_slot));
        }

        let capacity = self
            .slots
            .len()
            .checked_mul(2)
            .expect("interner slot overflow")
            .max(8);
        admit(InternerWork::InitializeSlots { slots: capacity })?;
        let mut slots = vec![None; capacity];
        for index in 0..self.entries.len() {
            admit(InternerWork::Reinsert)?;
            let symbol = Self::dynamic_symbol(index).expect("stored symbols fit");
            let slot = vacant_slot(&slots, self.entries[index].hash, admit)?;
            slots[slot] = Some(symbol);
        }
        let vacant = vacant_slot(&slots, hash, admit)?;
        Ok(SlotPlan::Replacement { slots, vacant })
    }

    fn insert<E>(
        &mut self,
        spelling: Spelling<'_>,
        hash: u64,
        slot: usize,
        admit: &mut impl FnMut(InternerWork) -> Result<(), E>,
    ) -> Result<Sym, E> {
        let symbol = Self::dynamic_symbol(self.entries.len())
            .expect("Cannot intern new string: integer overflow");
        let (latin1, utf8_bytes, utf16_units) = match spelling {
            Spelling::Static { utf16, .. } => (text::is_latin1(utf16, admit)?, 0, 0),
            Spelling::Buffered { utf8, utf16 } => (
                text::is_latin1(utf16, admit)?,
                utf8.map_or(0, str::len),
                utf16.len(),
            ),
        };
        let slot = self.prepare_slots(hash, slot, admit)?;
        admit_reserve(
            InternerBuffer::Utf8,
            self.utf8_buffer.len(),
            self.utf8_buffer.capacity(),
            utf8_bytes,
            admit,
        )?;
        admit_reserve(
            InternerBuffer::Utf16,
            self.utf16_buffer.len(),
            self.utf16_buffer.capacity(),
            utf16_units,
            admit,
        )?;
        admit_reserve(
            InternerBuffer::Entries,
            self.entries.len(),
            self.entries.capacity(),
            1,
            admit,
        )?;
        admit(InternerWork::Publish {
            utf8_bytes,
            utf16_units,
        })?;

        // No callback can refuse once live storage begins changing.
        self.utf8_buffer.reserve(utf8_bytes);
        self.utf16_buffer.reserve(utf16_units);
        self.entries.reserve(1);
        let text = match spelling {
            Spelling::Static { utf8, utf16 } => StoredText::Static { utf8, utf16 },
            Spelling::Buffered { utf8, utf16 } => {
                let utf8 = utf8.map(|value| {
                    let start = self.utf8_buffer.len();
                    self.utf8_buffer.push_str(value);
                    start..self.utf8_buffer.len()
                });
                let start = self.utf16_buffer.len();
                self.utf16_buffer.extend_from_slice(utf16);
                StoredText::Buffered {
                    utf8,
                    utf16: start..self.utf16_buffer.len(),
                }
            }
        };
        self.entries.push(Entry { text, latin1, hash });
        match slot {
            SlotPlan::Existing(slot) => self.slots[slot] = Some(symbol),
            SlotPlan::Replacement { mut slots, vacant } => {
                slots[vacant] = Some(symbol);
                self.slots = slots;
            }
        }
        Ok(symbol)
    }

    /// Returns the string for the given symbol if any.
    ///
    /// # Panics
    ///
    /// Panics if the generated common UTF-8 and UTF-16 tables are out of sync.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let resolved = interner.resolve(sym);
    /// assert!(resolved.is_some());
    /// assert_eq!(resolved.unwrap().utf8(), Some("hello"));
    /// ```
    #[must_use]
    pub fn resolve(&self, symbol: Sym) -> Option<JSInternedStrRef<'_, '_>> {
        let index = symbol.get() - 1;

        if let Some(utf8) = COMMON_STRINGS_UTF8.get(index).copied() {
            let utf16 = COMMON_STRINGS_UTF16[index];
            return Some(JSInternedStrRef {
                utf8: Some(utf8),
                utf16,
            });
        }

        let entry = self.entries.get(index - COMMON_STRINGS_UTF8.len())?;
        Some(JSInternedStrRef {
            utf8: entry.text.utf8(&self.utf8_buffer),
            utf16: entry.text.utf16(&self.utf16_buffer),
        })
    }

    /// Returns the string for the given symbol.
    ///
    /// # Panics
    ///
    /// If the interner cannot resolve the given symbol.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let resolved = interner.resolve_expect(sym);
    /// assert_eq!(resolved.utf8(), Some("hello"));
    /// ```
    #[inline]
    #[must_use]
    pub fn resolve_expect(&self, symbol: Sym) -> JSInternedStrRef<'_, '_> {
        self.resolve(symbol).expect("string disappeared")
    }

    /// Returns `true` if the string identified by `symbol` can be encoded as Latin1
    /// (i.e. all code units are in the range `0x00..=0xFF`).
    ///
    /// This information is computed **once** when the string is first interned, so callers pay no
    /// O(n) scanning cost beyond the initial intern call.
    ///
    /// # Examples
    ///
    /// ```
    /// use surgeist_js_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let ascii = interner.get_or_intern("hello");
    /// assert!(interner.is_latin1(ascii));
    ///
    /// let non_latin1: Vec<u16> = vec![0x4e2d, 0x6587]; // "中文"
    /// let sym = interner.get_or_intern(non_latin1.as_slice());
    /// assert!(!interner.is_latin1(sym));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_latin1(&self, symbol: Sym) -> bool {
        let index = symbol.get() - 1;
        if index < COMMON_STRINGS_UTF8.len() {
            return true;
        }
        let dynamic_index = index - COMMON_STRINGS_UTF8.len();
        self.entries
            .get(dynamic_index)
            .is_some_and(|entry| entry.latin1)
    }
}

/// Implements the display formatting with indentation.
pub trait ToIndentedString {
    /// Converts the element to a string using an interner, with the given indentation.
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String;
}

/// Converts a given element to a string using an interner.
pub trait ToInternedString {
    /// Converts a given element to a string using an interner.
    fn to_interned_string(&self, interner: &Interner) -> String;
}

impl<T> ToInternedString for T
where
    T: ToIndentedString,
{
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}
