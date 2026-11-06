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

mod sym;

#[cfg(test)]
mod tests;

use alloc::{borrow::Cow, format, string::String, vec::Vec};
use core::{
    hash::{BuildHasher, BuildHasherDefault, Hash},
    ops::Range,
};
use hashbrown::HashTable;
use rustc_hash::FxHasher;

pub use sym::*;

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

#[derive(Debug)]
struct Entry {
    text: StoredText,
    latin1: bool,
}

fn hash<T: Hash + ?Sized>(value: &T) -> u64 {
    BuildHasherDefault::<FxHasher>::default().hash_one(value)
}

/// The string interner for the JavaScript compiler.
#[derive(Debug, Default)]
pub struct Interner {
    // Both lookup tables refer to the same insertion-ordered entries.
    entries: Vec<Entry>,
    utf8_lookup: HashTable<usize>,
    utf16_lookup: HashTable<usize>,
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
            utf8_lookup: HashTable::new(),
            utf16_lookup: HashTable::new(),
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
        let string = string.into();
        Self::get_common(string).or_else(|| {
            let index = match string {
                JStrRef::Utf8(s) => self.utf8_lookup.find(hash(s), |&index| {
                    self.entries[index].text.utf8(&self.utf8_buffer) == Some(s)
                }),
                JStrRef::Utf16(s) => self.utf16_lookup.find(hash(s), |&index| {
                    self.entries[index].text.utf16(&self.utf16_buffer) == s
                }),
            };
            index.and_then(|index| Self::dynamic_symbol(*index))
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
        let string = string.into();
        self.get(string).unwrap_or_else(|| {
            let (utf8, utf16) = match string {
                JStrRef::Utf8(s) => (
                    Some(Cow::Borrowed(s)),
                    Cow::Owned(s.encode_utf16().collect()),
                ),
                JStrRef::Utf16(s) => (String::from_utf16(s).ok().map(Cow::Owned), Cow::Borrowed(s)),
            };

            let utf8 = utf8.map(|value| {
                let start = self.utf8_buffer.len();
                self.utf8_buffer.push_str(&value);
                start..self.utf8_buffer.len()
            });
            let utf16_start = self.utf16_buffer.len();
            self.utf16_buffer.extend_from_slice(&utf16);
            self.insert(StoredText::Buffered {
                utf8,
                utf16: utf16_start..self.utf16_buffer.len(),
            })
        })
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
        assert!(
            utf8.encode_utf16().eq(utf16.iter().copied()),
            "static UTF-8 and UTF-16 spellings must match"
        );
        self.get(utf8)
            .unwrap_or_else(|| self.insert(StoredText::Static { utf8, utf16 }))
    }

    fn dynamic_symbol(index: usize) -> Option<Sym> {
        index
            .checked_add(1 + COMMON_STRINGS_UTF8.len())
            .and_then(Sym::new)
    }

    fn insert(&mut self, text: StoredText) -> Sym {
        let index = self.entries.len();
        let symbol =
            Self::dynamic_symbol(index).expect("Cannot intern new string: integer overflow");
        let utf16 = text.utf16(&self.utf16_buffer);
        let utf16_hash = hash(utf16);
        let latin1 = utf16.iter().all(|&c| c <= 0xFF);
        let utf8_hash = text.utf8(&self.utf8_buffer).map(hash);
        self.entries.push(Entry { text, latin1 });

        self.utf16_lookup
            .insert_unique(utf16_hash, index, |&index| {
                hash(self.entries[index].text.utf16(&self.utf16_buffer))
            });
        if let Some(utf8_hash) = utf8_hash {
            self.utf8_lookup.insert_unique(utf8_hash, index, |&index| {
                hash(
                    self.entries[index]
                        .text
                        .utf8(&self.utf8_buffer)
                        .expect("UTF-8 lookup requires a UTF-8 spelling"),
                )
            });
        }
        symbol
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

        if let Some(utf8) = COMMON_STRINGS_UTF8.index(index).copied() {
            let utf16 = COMMON_STRINGS_UTF16
                .get_index(index)
                .copied()
                .expect("The sizes of both statics must be equal");
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

    fn get_common(string: JStrRef<'_>) -> Option<Sym> {
        let index = match string {
            JStrRef::Utf8(s) => COMMON_STRINGS_UTF8.get_index(s),
            JStrRef::Utf16(s) => COMMON_STRINGS_UTF16.get_index_of(&s),
        }?;
        Sym::new(index + 1)
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
