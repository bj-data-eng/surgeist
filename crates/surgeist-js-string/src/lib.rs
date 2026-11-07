//! A Latin1 or UTF-16 encoded, reference counted, immutable string.

#![forbid(unsafe_code)]
#![allow(clippy::module_name_repetitions)]

mod admitted;
mod builder;
mod code_point;
mod common;
mod display;
mod iter;
mod storage;
mod str;
mod r#type;

#[cfg(test)]
mod tests;

use self::iter::Windows;
use crate::display::{JsStrDisplayEscaped, JsStrDisplayLossy, JsStringDebugInfo};
use crate::iter::CodePointsIter;
pub use crate::storage::StaticString;
use crate::storage::{Root, SliceString, Storage, Units};
#[doc(inline)]
pub use crate::{
    builder::{CommonJsStringBuilder, Latin1JsStringBuilder, Utf16JsStringBuilder},
    code_point::CodePoint,
    common::StaticJsStrings,
    iter::Iter,
    str::{JsStr, JsStrVariant},
};
pub use admitted::{StringError, StringWork};
use std::borrow::Cow;
use std::{
    convert::Infallible,
    hash::{Hash, Hasher},
    rc::Rc,
    str::FromStr,
};

fn alloc_overflow() -> ! {
    panic!("detected overflow during string allocation")
}

/// Helper function to check if a `char` is trimmable.
pub(crate) const fn is_trimmable_whitespace(c: char) -> bool {
    // The rust implementation of `trim` does not regard the same characters whitespace as
    // ecma standard does.
    //
    // Rust uses \p{White_Space} by default, which also includes:
    // `\u{0085}' (next line)
    // And does not include:
    // '\u{FEFF}' (zero width non-breaking space)
    // Explicit whitespace: https://tc39.es/ecma262/#sec-white-space
    matches!(
        c,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{FEFF}' |
    // Unicode Space_Separator category
    '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' |
    // Line terminators: https://tc39.es/ecma262/#sec-line-terminators
    '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}'
    )
}

/// Helper function to check if a `u8` latin1 character is trimmable.
pub(crate) const fn is_trimmable_whitespace_latin1(c: u8) -> bool {
    // The rust implementation of `trim` does not regard the same characters whitespace as
    // ecma standard does.
    //
    // Rust uses \p{White_Space} by default, which also includes:
    // `\u{0085}' (next line)
    // And does not include:
    // '\u{FEFF}' (zero width non-breaking space)
    // Explicit whitespace: https://tc39.es/ecma262/#sec-white-space
    matches!(
        c,
        0x09 | 0x0B | 0x0C | 0x20 | 0xA0 |
        // Line terminators: https://tc39.es/ecma262/#sec-line-terminators
        0x0A | 0x0D
    )
}

/// Strings can be represented internally by multiple kinds. This is used to identify
/// the storage kind of string.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub(crate) enum JsStringKind {
    /// A sequence of Latin1 bytes.
    Latin1Sequence = 0,

    /// A sequence of UTF-16 code units.
    Utf16Sequence = 1,

    /// A slice of an existing string. See [`SliceString`].
    Slice = 2,

    /// A static string that is valid for `'static` lifetime.
    Static = 3,
}

/// A Latin1 or UTF-16–encoded, reference counted, immutable string.
///
/// Strings own their code units, refer to static storage, or share a view of an
/// existing string. A view preserves the backing string's code-unit encoding.
#[derive(Clone)]
pub struct JsString {
    storage: Storage,
}

impl<'a> From<&'a JsString> for JsStr<'a> {
    #[inline]
    fn from(value: &'a JsString) -> Self {
        value.as_str()
    }
}

impl<'a> IntoIterator for &'a JsString {
    type Item = u16;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl JsString {
    /// Create an iterator over the [`JsString`].
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        self.as_str().iter()
    }

    /// Create an iterator over overlapping subslices of length size.
    #[inline]
    #[must_use]
    pub fn windows(&self, size: usize) -> Windows<'_> {
        self.as_str().windows(size)
    }

    /// Decodes a [`JsString`] into a [`String`], replacing invalid data with its escaped representation
    /// in 4 digit hexadecimal.
    #[inline]
    #[must_use]
    pub fn to_std_string_escaped(&self) -> String {
        self.display_escaped().to_string()
    }

    /// Decodes a [`JsString`] into a [`String`], replacing invalid data with the
    /// replacement character U+FFFD.
    #[inline]
    #[must_use]
    pub fn to_std_string_lossy(&self) -> String {
        self.display_lossy().to_string()
    }

    /// Decodes a [`JsString`] into a [`String`], returning an error if the string contains unpaired
    /// surrogates.
    ///
    /// # Errors
    ///
    /// [`FromUtf16Error`][std::string::FromUtf16Error] if it contains any invalid data.
    #[inline]
    pub fn to_std_string(&self) -> Result<String, std::string::FromUtf16Error> {
        self.as_str().to_std_string()
    }

    /// Decodes a [`JsString`] into an iterator of [`Result<String, u16>`], returning surrogates as
    /// errors.
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn to_std_string_with_surrogates(
        &self,
    ) -> impl Iterator<Item = Result<String, u16>> + use<'_> {
        let mut iter = self.code_points().peekable();

        std::iter::from_fn(move || {
            let cp = iter.next()?;
            let char = match cp {
                CodePoint::Unicode(c) => c,
                CodePoint::UnpairedSurrogate(surr) => return Some(Err(surr)),
            };

            let mut string = String::from(char);

            while let Some(cp) = iter.peek().and_then(|cp| match cp {
                CodePoint::Unicode(c) => Some(*c),
                CodePoint::UnpairedSurrogate(_) => None,
            }) {
                string.push(cp);
                iter.next().expect("iter.peek() ensures that next is Some");
            }

            Some(Ok(string))
        })
    }

    /// Maps the valid segments of an UTF16 string and leaves the unpaired surrogates unchanged.
    #[inline]
    #[must_use]
    pub fn map_valid_segments<F>(&self, mut f: F) -> Self
    where
        F: FnMut(String) -> String,
    {
        let mut text = Vec::new();

        for part in self.to_std_string_with_surrogates() {
            match part {
                Ok(string) => text.extend(f(string).encode_utf16()),
                Err(surr) => text.push(surr),
            }
        }

        Self::from(&text[..])
    }

    /// Gets an iterator of all the Unicode codepoints of a [`JsString`].
    #[inline]
    #[must_use]
    pub fn code_points(&self) -> CodePointsIter<'_> {
        self.as_str().code_points()
    }

    /// Get the variant of this string.
    #[inline]
    #[must_use]
    pub fn variant(&self) -> JsStrVariant<'_> {
        self.as_str().variant()
    }

    /// Abstract operation `StringIndexOf ( string, searchValue, fromIndex )`
    ///
    /// Note: Instead of returning an isize with `-1` as the "not found" value, we make use of the
    /// type system and return <code>[Option]\<usize\></code> with [`None`] as the "not found" value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringindexof
    #[inline]
    #[must_use]
    pub fn index_of(&self, search_value: JsStr<'_>, from_index: usize) -> Option<usize> {
        self.as_str().index_of(search_value, from_index)
    }

    /// Abstract operation `CodePointAt( string, position )`.
    ///
    /// The abstract operation `CodePointAt` takes arguments `string` (a String) and `position` (a
    /// non-negative integer) and returns a Record with fields `[[CodePoint]]` (a code point),
    /// `[[CodeUnitCount]]` (a positive integer), and `[[IsUnpairedSurrogate]]` (a Boolean). It
    /// interprets string as a sequence of UTF-16 encoded code points, as described in 6.1.4, and reads
    /// from it a single code point starting with the code unit at index `position`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-codepointat
    ///
    /// # Panics
    ///
    /// If `position` is smaller than size of string.
    #[inline]
    #[must_use]
    pub fn code_point_at(&self, position: usize) -> CodePoint {
        self.as_str().code_point_at(position)
    }

    /// Abstract operation `StringToNumber ( str )`
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringtonumber
    #[inline]
    #[must_use]
    pub fn to_number(&self) -> f64 {
        self.as_str().to_number()
    }

    /// Get the length of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        match &self.storage {
            Storage::Root(root) => root.as_str().len(),
            Storage::Slice(slice) => slice.len(),
        }
    }

    /// Return true if the [`JsString`] is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert the [`JsString`] into a [`Vec<U16>`].
    #[inline]
    #[must_use]
    pub fn to_vec(&self) -> Vec<u16> {
        self.as_str().to_vec()
    }

    /// Check if the [`JsString`] contains a byte.
    #[inline]
    #[must_use]
    pub fn contains(&self, element: u8) -> bool {
        self.as_str().contains(element)
    }

    /// Trim whitespace from the start and end of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn trim(&self) -> JsString {
        // Calculate both bounds directly to avoid intermediate allocations.
        let (start, end) = match self.variant() {
            JsStrVariant::Latin1(v) => {
                let Some(start) = v.iter().position(|c| !is_trimmable_whitespace_latin1(*c)) else {
                    return StaticJsStrings::EMPTY_STRING;
                };
                let end = v
                    .iter()
                    .rposition(|c| !is_trimmable_whitespace_latin1(*c))
                    .unwrap_or(start);
                (start, end)
            }
            JsStrVariant::Utf16(v) => {
                let Some(start) = v.iter().copied().position(|r| {
                    !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)
                }) else {
                    return StaticJsStrings::EMPTY_STRING;
                };
                let end = v
                    .iter()
                    .copied()
                    .rposition(|r| {
                        !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)
                    })
                    .unwrap_or(start);
                (start, end)
            }
        };

        self.slice_range(start, end + 1)
    }

    /// Trim whitespace from the start of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn trim_start(&self) -> JsString {
        let Some(start) = (match self.variant() {
            JsStrVariant::Latin1(v) => v.iter().position(|c| !is_trimmable_whitespace_latin1(*c)),
            JsStrVariant::Utf16(v) => v
                .iter()
                .copied()
                .position(|r| !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)),
        }) else {
            return StaticJsStrings::EMPTY_STRING;
        };

        self.slice_range(start, self.len())
    }

    /// Trim whitespace from the end of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn trim_end(&self) -> JsString {
        let Some(end) = (match self.variant() {
            JsStrVariant::Latin1(v) => v.iter().rposition(|c| !is_trimmable_whitespace_latin1(*c)),
            JsStrVariant::Utf16(v) => v
                .iter()
                .copied()
                .rposition(|r| !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)),
        }) else {
            return StaticJsStrings::EMPTY_STRING;
        };

        self.slice_range(0, end + 1)
    }

    /// Returns true if needle is a prefix of the [`JsStr`].
    #[inline]
    #[must_use]
    // We check the size, so this should never panic.
    #[allow(clippy::missing_panics_doc)]
    pub fn starts_with(&self, needle: JsStr<'_>) -> bool {
        self.as_str().starts_with(needle)
    }

    /// Returns `true` if `needle` is a suffix of the [`JsStr`].
    #[inline]
    #[must_use]
    // We check the size, so this should never panic.
    #[allow(clippy::missing_panics_doc)]
    pub fn ends_with(&self, needle: JsStr<'_>) -> bool {
        self.as_str().ends_with(needle)
    }

    /// Get the `u16` code unit at index. This does not parse any characters if there
    /// are pairs, it is simply the index of the `u16` elements.
    #[inline]
    #[must_use]
    pub fn code_unit_at(&self, index: usize) -> Option<u16> {
        self.as_str().get(index)
    }

    /// Get the element at the given index, or [`None`] if the index is out of range.
    #[inline]
    #[must_use]
    pub fn get<I>(&self, index: I) -> Option<JsString>
    where
        I: JsStringSliceIndex,
    {
        index.get(self)
    }

    /// Get the element at the given index, or panic.
    ///
    /// # Panics
    /// If the index returns `None`, this will panic.
    #[inline]
    #[must_use]
    pub fn get_expect<I>(&self, index: I) -> JsString
    where
        I: JsStringSliceIndex,
    {
        index.get(self).expect("Unexpected get()")
    }

    /// Gets a displayable escaped string. This may be faster and has fewer
    /// allocations than `format!("{}", str.to_string_escaped())` when
    /// displaying.
    #[inline]
    #[must_use]
    pub fn display_escaped(&self) -> JsStrDisplayEscaped<'_> {
        JsStrDisplayEscaped::from(self)
    }

    /// Gets a displayable lossy string. This may be faster and has fewer
    /// allocations than `format!("{}", str.to_string_lossy())` when displaying.
    #[inline]
    #[must_use]
    pub fn display_lossy(&self) -> JsStrDisplayLossy<'_> {
        self.as_str().display_lossy()
    }

    /// Get a debug displayable info and metadata for this string.
    #[inline]
    #[must_use]
    pub fn debug_info(&self) -> JsStringDebugInfo<'_> {
        self.into()
    }
}

/// Dealing with inner types.
impl JsString {
    /// Check if this is a static string.
    #[inline]
    #[must_use]
    pub fn is_static(&self) -> bool {
        matches!(self.storage, Storage::Root(Root::Static(_)))
    }

    /// Creates a JavaScript string from static code units.
    #[inline]
    #[must_use]
    pub const fn from_static(str: &'static StaticString) -> Self {
        Self {
            storage: Storage::Root(Root::Static(str)),
        }
    }

    /// Builds a slice with an exclusive end. Invalid ranges panic.
    #[inline]
    #[must_use]
    fn slice_range(&self, start: usize, end: usize) -> Self {
        assert!(start <= end && end <= self.len(), "Index out of bounds");
        let slice = SliceString::new(&self.storage, start..end);
        Self {
            storage: Storage::Slice(Rc::new(slice)),
        }
    }

    /// Slices the string, clamping the end to the string length.
    /// Returns the empty string if the start is at or beyond the clamped end.
    #[inline]
    #[must_use]
    pub fn slice(&self, start: usize, end: usize) -> JsString {
        let end = end.min(self.len());
        if start >= end {
            StaticJsStrings::EMPTY_STRING
        } else {
            self.slice_range(start, end)
        }
    }

    #[cfg(test)]
    fn kind(&self) -> JsStringKind {
        match &self.storage {
            Storage::Root(root) => root.kind(),
            Storage::Slice(_) => JsStringKind::Slice,
        }
    }
}

impl JsString {
    /// Obtains the underlying [`&[u16]`][slice] slice of a [`JsString`]
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> JsStr<'_> {
        match &self.storage {
            Storage::Root(root) => root.as_str(),
            Storage::Slice(slice) => slice.as_str(),
        }
    }

    /// Creates a new [`JsString`] from the concatenation of `x` and `y`.
    #[inline]
    #[must_use]
    pub fn concat(x: JsStr<'_>, y: JsStr<'_>) -> Self {
        Self::concat_array(&[x, y])
    }

    /// Creates a new [`JsString`] from the concatenation of every element of
    /// `strings`.
    #[inline]
    #[must_use]
    pub fn concat_array(strings: &[JsStr<'_>]) -> Self {
        let mut latin1_encoding = true;
        let mut full_count = 0usize;
        for string in strings {
            let Some(sum) = full_count.checked_add(string.len()) else {
                alloc_overflow()
            };
            if !string.is_latin1() {
                latin1_encoding = false;
            }
            full_count = sum;
        }

        if latin1_encoding {
            let mut units = Vec::with_capacity(full_count);
            for string in strings {
                units.extend_from_slice(string.as_latin1().expect("encoding checked above"));
            }
            StaticJsStrings::get_string(&JsStr::latin1(&units))
                .unwrap_or_else(|| Self::from_units(Units::Latin1(units.into_boxed_slice())))
        } else {
            let mut units = Vec::with_capacity(full_count);
            for &string in strings {
                match string.variant() {
                    JsStrVariant::Latin1(s) => units.extend(s.iter().copied().map(u16::from)),
                    JsStrVariant::Utf16(s) => units.extend_from_slice(s),
                }
            }
            StaticJsStrings::get_string(&JsStr::utf16(&units))
                .unwrap_or_else(|| Self::from_units(Units::Utf16(units.into_boxed_slice())))
        }
    }

    fn from_units(units: Units) -> Self {
        Self {
            storage: Storage::Root(Root::Sequence(Rc::new(units))),
        }
    }

    /// Creates a sequence without consulting the static string lookup.
    fn from_slice_skip_interning(string: JsStr<'_>) -> Self {
        Self::from_units(match string.variant() {
            JsStrVariant::Latin1(s) => Units::Latin1(s.into()),
            JsStrVariant::Utf16(s) => Units::Utf16(s.into()),
        })
    }

    /// Creates a new [`JsString`] from `data`.
    fn from_js_str(string: JsStr<'_>) -> Self {
        if let Some(s) = StaticJsStrings::get_string(&string) {
            return s;
        }
        Self::from_slice_skip_interning(string)
    }

    /// Gets the number of `JsString`s which point to this allocation.
    #[inline]
    #[must_use]
    pub fn refcount(&self) -> Option<usize> {
        match &self.storage {
            Storage::Root(root) => root.refcount(),
            Storage::Slice(slice) => Some(Rc::strong_count(slice)),
        }
    }
}

impl Default for JsString {
    #[inline]
    fn default() -> Self {
        StaticJsStrings::EMPTY_STRING
    }
}

impl std::fmt::Debug for JsString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("JsString")
            .field(&self.display_escaped().to_string())
            .finish()
    }
}

impl Eq for JsString {}

macro_rules! impl_from_number_for_js_string {
    ($($module: ident => $($ty:ty),+)+) => {
        $(
            $(
                impl From<$ty> for JsString {
                    #[inline]
                    fn from(value: $ty) -> Self {
                        JsString::from_slice_skip_interning(JsStr::latin1(
                            $module::Buffer::new().format(value).as_bytes(),
                        ))
                    }
                }
            )+
        )+
    };
}

impl_from_number_for_js_string!(
    itoa => i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, isize, usize
    ryu_js => f32, f64
);

impl From<&[u16]> for JsString {
    #[inline]
    fn from(s: &[u16]) -> Self {
        JsString::from_js_str(JsStr::utf16(s))
    }
}

impl From<&str> for JsString {
    #[inline]
    fn from(s: &str) -> Self {
        if s.is_ascii() {
            let js_str = JsStr::latin1(s.as_bytes());
            return StaticJsStrings::get_string(&js_str)
                .unwrap_or_else(|| JsString::from_slice_skip_interning(js_str));
        }
        // Non-ASCII but still Latin1-encodable (U+0080..=U+00FF): chars map 1-to-1 to u8.
        if s.chars().all(|c| c as u32 <= 0xFF) {
            let bytes: Vec<u8> = s.chars().map(|c| c as u8).collect();
            let js_str = JsStr::latin1(&bytes);
            return StaticJsStrings::get_string(&js_str)
                .unwrap_or_else(|| JsString::from_units(Units::Latin1(bytes.into_boxed_slice())));
        }
        JsString::from_units(Units::Utf16(s.encode_utf16().collect()))
    }
}

impl From<JsStr<'_>> for JsString {
    #[inline]
    fn from(value: JsStr<'_>) -> Self {
        StaticJsStrings::get_string(&value)
            .unwrap_or_else(|| JsString::from_slice_skip_interning(value))
    }
}

impl From<&[JsString]> for JsString {
    #[inline]
    fn from(value: &[JsString]) -> Self {
        Self::concat_array(&value.iter().map(Self::as_str).collect::<Vec<_>>()[..])
    }
}

impl<const N: usize> From<&[JsString; N]> for JsString {
    #[inline]
    fn from(value: &[JsString; N]) -> Self {
        Self::concat_array(&value.iter().map(Self::as_str).collect::<Vec<_>>()[..])
    }
}

impl From<String> for JsString {
    #[inline]
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl<'a> From<Cow<'a, str>> for JsString {
    #[inline]
    fn from(s: Cow<'a, str>) -> Self {
        match s {
            Cow::Borrowed(s) => s.into(),
            Cow::Owned(s) => s.into(),
        }
    }
}

impl<const N: usize> From<&[u16; N]> for JsString {
    #[inline]
    fn from(s: &[u16; N]) -> Self {
        Self::from(&s[..])
    }
}

impl Hash for JsString {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialOrd for JsStr<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JsString {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(&other.as_str())
    }
}

impl PartialEq for JsString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<JsString> for [u16] {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (x, y) in self.iter().copied().zip(other.iter()) {
            if x != y {
                return false;
            }
        }
        true
    }
}

impl<const N: usize> PartialEq<JsString> for [u16; N] {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
        self[..] == *other
    }
}

impl PartialEq<[u16]> for JsString {
    #[inline]
    fn eq(&self, other: &[u16]) -> bool {
        other == self
    }
}

impl<const N: usize> PartialEq<[u16; N]> for JsString {
    #[inline]
    fn eq(&self, other: &[u16; N]) -> bool {
        *self == other[..]
    }
}

impl PartialEq<str> for JsString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for JsString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<JsString> for str {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
        other == self
    }
}

impl PartialEq<JsStr<'_>> for JsString {
    #[inline]
    fn eq(&self, other: &JsStr<'_>) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<JsString> for JsStr<'_> {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
        other == self
    }
}

impl PartialOrd for JsString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for JsString {
    type Err = Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

/// Similar to [`std::ops::RangeBounds`] but custom implemented for getting direct indices.
// TODO: remove [`str::JsSliceIndex`] and rename this when `JsStr` is no more.
pub trait JsStringSliceIndex {
    /// Get the substring (or `None` if outside the string).
    fn get(self, str: &JsString) -> Option<JsString>;
}

macro_rules! impl_js_string_slice_index {
    ($($type:ty),+ $(,)?) => {
        $(
        impl JsStringSliceIndex for $type {
            fn get(self, str: &JsString) -> Option<JsString> {
                let start = match std::ops::RangeBounds::<usize>::start_bound(&self) {
                    std::ops::Bound::Included(start) => *start,
                    std::ops::Bound::Excluded(start) => start.checked_add(1)?,
                    std::ops::Bound::Unbounded => 0,
                };

                let end = match std::ops::RangeBounds::<usize>::end_bound(&self) {
                    std::ops::Bound::Included(end) => end.checked_add(1)?,
                    std::ops::Bound::Excluded(end) => *end,
                    std::ops::Bound::Unbounded => str.len(),
                };

                if end > str.len() || start > end {
                    None
                } else {
                    Some(str.slice_range(start, end))
                }
            }
        }
        )+
    };
}

impl_js_string_slice_index!(
    std::ops::Range<usize>,
    std::ops::RangeInclusive<usize>,
    std::ops::RangeTo<usize>,
    std::ops::RangeToInclusive<usize>,
    std::ops::RangeFrom<usize>,
    std::ops::RangeFull,
);
