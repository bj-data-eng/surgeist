use crate::r#type::{Latin1, StringType, Utf16};
use crate::{JsStr, JsStrVariant, JsString, Units};
use std::ops::{Add, AddAssign};

/// A mutable builder for initialized JavaScript string code units.
#[derive(Debug)]
pub struct JsStringBuilder<D: StringType> {
    units: Vec<D::Byte>,
}

impl<D: StringType> Default for JsStringBuilder<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D: StringType> JsStringBuilder<D> {
    /// Creates an empty builder.
    #[must_use]
    pub const fn new() -> Self {
        Self { units: Vec::new() }
    }

    /// Creates an empty builder with space for at least `capacity` code units.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            units: Vec::with_capacity(capacity),
        }
    }

    /// Returns the number of code units.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.units.len()
    }

    /// Returns the number of code units that fit without reallocating.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.units.capacity()
    }

    /// Returns whether there are any code units.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    /// Reserves capacity for at least `additional` more code units.
    pub fn reserve(&mut self, additional: usize) {
        self.units.reserve(additional);
    }

    /// Reserves capacity without asking for speculative extra space.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.units.reserve_exact(additional);
    }

    /// Appends one code unit.
    pub fn push(&mut self, unit: D::Byte) {
        self.units.push(unit);
    }

    /// Appends code units from a slice.
    pub fn extend_from_slice(&mut self, units: &[D::Byte]) {
        self.units.extend_from_slice(units);
    }

    /// Borrows the initialized code units.
    #[must_use]
    pub fn as_slice(&self) -> &[D::Byte] {
        &self.units
    }

    /// Mutably borrows the initialized code units.
    pub fn as_mut_slice(&mut self) -> &mut [D::Byte] {
        &mut self.units
    }

    fn build_inner(self, units: impl FnOnce(Box<[D::Byte]>) -> Units) -> JsString {
        if self.units.is_empty() {
            JsString::default()
        } else {
            JsString::from_units(units(self.units.into_boxed_slice()))
        }
    }
}

impl<D: StringType> AddAssign<&JsStringBuilder<D>> for JsStringBuilder<D> {
    #[inline]
    fn add_assign(&mut self, rhs: &JsStringBuilder<D>) {
        self.extend_from_slice(rhs.as_slice());
    }
}

impl<D: StringType> AddAssign<&[D::Byte]> for JsStringBuilder<D> {
    #[inline]
    fn add_assign(&mut self, rhs: &[D::Byte]) {
        self.extend_from_slice(rhs);
    }
}

impl<D: StringType> Add<&JsStringBuilder<D>> for JsStringBuilder<D> {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: &JsStringBuilder<D>) -> Self::Output {
        self.extend_from_slice(rhs.as_slice());
        self
    }
}

impl<D: StringType> Add<&[D::Byte]> for JsStringBuilder<D> {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: &[D::Byte]) -> Self::Output {
        self.extend_from_slice(rhs);
        self
    }
}

impl<D: StringType> Extend<D::Byte> for JsStringBuilder<D> {
    #[inline]
    fn extend<I: IntoIterator<Item = D::Byte>>(&mut self, iter: I) {
        self.units.extend(iter);
    }
}

impl<D: StringType> FromIterator<D::Byte> for JsStringBuilder<D> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = D::Byte>>(iter: T) -> Self {
        let mut builder = Self::new();
        builder.extend(iter);
        builder
    }
}

impl<D: StringType> From<&[D::Byte]> for JsStringBuilder<D> {
    #[inline]
    fn from(value: &[D::Byte]) -> Self {
        Self {
            units: value.to_vec(),
        }
    }
}

impl<D: StringType> PartialEq for JsStringBuilder<D>
where
    D::Byte: Eq + PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let slice: &[D::Byte] = self.as_slice();
        let other_slice: &[D::Byte] = other.as_slice();
        slice.eq(other_slice)
    }
}

impl<D: StringType> Clone for JsStringBuilder<D> {
    #[inline]
    fn clone(&self) -> Self {
        let mut builder = Self::with_capacity(self.capacity());
        builder.extend_from_slice(self.as_slice());
        builder
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.units.clone_from(&source.units);
    }
}

impl JsStringBuilder<Latin1> {
    /// Checks if all bytes in inner allocation's data are ascii.
    #[inline]
    #[must_use]
    pub fn is_ascii(&self) -> bool {
        self.as_slice().is_ascii()
    }
}

impl JsStringBuilder<Utf16> {
    /// Checks if all u16 in inner allocation's data are ascii (<= 0x7F).
    #[inline]
    #[must_use]
    pub fn is_ascii(&self) -> bool {
        self.as_slice().iter().all(|&c| c <= 0x7F)
    }
}

/// **`Latin1`** encoded `JsStringBuilder`
/// # Warning
/// If you are not sure the characters that will be added and don't want to preprocess them,
/// use [`CommonJsStringBuilder`] instead.
/// ## Examples
///
/// ```rust
/// use surgeist_js_string::Latin1JsStringBuilder;
/// let mut s = Latin1JsStringBuilder::new();
/// s.push(b'x');
/// s.extend_from_slice(&[b'1', b'2', b'3']);
/// s.extend([b'1', b'2', b'3']);
/// let js_string = s.build();
/// ```
pub type Latin1JsStringBuilder = JsStringBuilder<Latin1>;

impl Latin1JsStringBuilder {
    /// Builds a `JsString` if the current instance is strictly `ASCII`.
    ///
    /// When the string contains characters outside the `ASCII` range, it cannot be determined
    /// whether the encoding is `Latin1` or others. Therefore, this method only returns a
    /// valid `JsString` when the instance is entirely `ASCII`. If any non-`ASCII` characters
    /// are present, it returns `None` to avoid ambiguity in encoding.
    ///
    /// If the caller is certain that the string is encoded in `Latin1`,
    /// [`build_as_latin1`](Self::build_as_latin1) can be used to avoid the `ASCII` check.
    #[inline]
    #[must_use]
    pub fn build(self) -> Option<JsString> {
        if self.is_ascii() {
            Some(self.build_inner(Units::Latin1))
        } else {
            None
        }
    }

    /// Interprets every byte as a Latin1 code unit and builds the string.
    #[inline]
    #[must_use]
    pub fn build_as_latin1(self) -> JsString {
        self.build_inner(Units::Latin1)
    }
}

/// **`UTF-16`** encoded `JsStringBuilder`
/// ## Examples
///
/// ```rust
/// use surgeist_js_string::Utf16JsStringBuilder;
/// let mut s = Utf16JsStringBuilder::new();
/// s.push(b'x' as u16);
/// s.extend_from_slice(&[b'1', b'2', b'3'].map(u16::from));
/// s.extend([0xD83C, 0xDFB9, 0xD83C, 0xDFB6, 0xD83C, 0xDFB5]); // 🎹🎶🎵
/// let js_string = s.build();
/// ```
pub type Utf16JsStringBuilder = JsStringBuilder<Utf16>;

impl Utf16JsStringBuilder {
    /// Builds `JsString` from `Utf16JsStringBuilder`
    #[inline]
    #[must_use]
    pub fn build(self) -> JsString {
        self.build_inner(Units::Utf16)
    }
}

/// Represents a segment of a string used to construct a [`JsString`].
#[derive(Clone, Debug)]
pub enum Segment<'a> {
    /// A string segment represented as a `JsString`.
    String(JsString),

    /// A string segment represented as a `JsStr`.
    Str(JsStr<'a>),

    /// A string segment represented as a byte.
    Latin1(u8),

    /// A Unicode code point segment represented as a character.
    CodePoint(char),
}

impl Segment<'_> {
    /// Checks if the segment can be represented as `Latin1` characters.
    #[inline]
    #[must_use]
    fn can_be_latin1(&self) -> bool {
        match self {
            Segment::String(s) => s.as_str().is_latin1(),
            Segment::Str(s) => s.is_latin1(),
            Segment::Latin1(_) => true,
            Segment::CodePoint(ch) => *ch as u32 <= 0xFF,
        }
    }
}

impl From<JsString> for Segment<'_> {
    #[inline]
    fn from(value: JsString) -> Self {
        Self::String(value)
    }
}

impl From<String> for Segment<'_> {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

impl From<&[u16]> for Segment<'_> {
    #[inline]
    fn from(value: &[u16]) -> Self {
        Self::String(value.into())
    }
}

impl From<&str> for Segment<'_> {
    #[inline]
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl<'seg, 'ref_str: 'seg> From<JsStr<'ref_str>> for Segment<'seg> {
    #[inline]
    fn from(value: JsStr<'ref_str>) -> Self {
        Self::Str(value)
    }
}

impl From<u8> for Segment<'_> {
    #[inline]
    fn from(value: u8) -> Self {
        Self::Latin1(value)
    }
}

impl From<char> for Segment<'_> {
    #[inline]
    fn from(value: char) -> Self {
        Self::CodePoint(value)
    }
}

/// Common `JsString` builder that accepts multiple variant of string or character.
///
/// Originally based on [kiesel-js](https://codeberg.org/kiesel-js/kiesel/src/branch/main/src/types/language/String/Builder.zig)
#[derive(Clone, Debug, Default)]
pub struct CommonJsStringBuilder<'a> {
    segments: Vec<Segment<'a>>,
}

impl<'seg, 'ref_str: 'seg> CommonJsStringBuilder<'seg> {
    /// Creates a new `CommonJsStringBuilder` with capacity of zero.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Similar to `Vec::with_capacity`.
    ///
    /// Creates a new `CommonJsStringBuilder` with given capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            segments: Vec::with_capacity(capacity),
        }
    }

    /// Similar to `Vec::reserve`.
    ///
    /// Reserves additional capacity for the inner vector.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.segments.reserve(additional);
    }

    /// Similar to `Vec::reserve_exact`.
    ///
    /// Reserves the minimum capacity for the inner vector.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.segments.reserve_exact(additional);
    }

    /// Appends string segments to the back of the inner vector.
    #[inline]
    pub fn push<T: Into<Segment<'ref_str>>>(&mut self, seg: T) {
        self.segments.push(seg.into());
    }

    /// Checks if all string segments can be represented as `Latin1` characters.
    #[inline]
    #[must_use]
    pub fn can_be_latin1(&self) -> bool {
        self.segments.iter().all(Segment::can_be_latin1)
    }

    /// Returns the number of string segment in inner vector.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Returns true if this `CommonJsStringBuilder` has a length of zero, and false otherwise.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Builds `Latin1` encoded `JsString` from string segments.
    ///
    /// This doesn't consume the builder itself because it may fails to build
    /// and the caller may wants to keep the builder for further operations.
    ///
    /// This processes the following types of segments:
    ///
    /// - `Segment::String(s)`: Encodes the string if it can be represented in `Latin1`.
    /// - `Segment::Str(s)`: Encodes the string slice if it can be represented in `Latin1`.
    /// - `Segment::Latin1(b)`: Encodes the byte if it's within the `ASCII` range.
    /// - `Segment::CodePoint(ch)`: Encodes the code point by converting it to a byte if it's within the `ASCII` range.
    ///
    /// Return `None` if any segment fails to encode.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_lossless)]
    pub fn build_from_latin1(&self) -> Option<JsString> {
        let mut builder = Latin1JsStringBuilder::new();
        for seg in &self.segments {
            match seg {
                Segment::String(s) => {
                    builder.extend_from_slice(s.as_str().as_latin1()?);
                }
                Segment::Str(s) => {
                    let data = s.as_latin1()?;
                    builder.extend_from_slice(data);
                }
                Segment::Latin1(b) => {
                    if *b <= 0x7f {
                        builder.push(*b);
                    } else {
                        return None;
                    }
                }
                Segment::CodePoint(ch) => {
                    if let Ok(b) = u8::try_from(*ch as u32) {
                        builder.push(b);
                    } else {
                        return None;
                    }
                }
            }
        }
        builder.build()
    }

    /// Builds `Utf-16` encoded `JsString` from string segments.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn build_from_utf16(self) -> JsString {
        let mut builder = Utf16JsStringBuilder::new();
        for seg in self.segments {
            match seg {
                Segment::String(s) => {
                    let js_str = s.as_str();
                    match js_str.variant() {
                        JsStrVariant::Latin1(s) => builder.extend(s.iter().copied().map(u16::from)),
                        JsStrVariant::Utf16(s) => builder.extend_from_slice(s),
                    }
                }
                Segment::Str(s) => match s.variant() {
                    JsStrVariant::Latin1(s) => builder.extend(s.iter().copied().map(u16::from)),
                    JsStrVariant::Utf16(s) => builder.extend_from_slice(s),
                },
                Segment::Latin1(latin1) => builder.push(u16::from(latin1)),
                Segment::CodePoint(code_point) => {
                    builder.extend_from_slice(code_point.encode_utf16(&mut [0_u16; 2]));
                }
            }
        }
        builder.build()
    }

    /// Builds `JsString` from `CommonJsStringBuilder`,
    ///
    /// This function first checks if the instance is empty:
    /// - If it is empty, it returns the default `JsString`.
    /// - If it can be represented as Latin1 characters, it safely encodes it as `Latin1`.
    /// - Otherwise, it falls back to encoding using `UTF-16`.
    #[inline]
    #[must_use]
    pub fn build(self) -> JsString {
        if self.is_empty() {
            JsString::default()
        } else if self.can_be_latin1() {
            self.build_latin1_segments()
        } else {
            self.build_from_utf16()
        }
    }

    fn build_latin1_segments(self) -> JsString {
        let mut builder = Latin1JsStringBuilder::new();
        for seg in self.segments {
            match seg {
                Segment::String(s) => {
                    let js_str = s.as_str();
                    let Some(s) = js_str.as_latin1() else {
                        unreachable!("string segment should be latin1")
                    };
                    builder.extend_from_slice(s);
                }
                Segment::Str(s) => {
                    let Some(s) = s.as_latin1() else {
                        unreachable!("string segment should be latin1")
                    };
                    builder.extend_from_slice(s);
                }
                Segment::Latin1(latin1) => builder.push(latin1),
                Segment::CodePoint(code_point) => builder
                    .push(u8::try_from(u32::from(code_point)).expect("segment must fit in Latin1")),
            }
        }
        builder.build_as_latin1()
    }
}

impl<'ref_str, T: Into<Segment<'ref_str>>> AddAssign<T> for CommonJsStringBuilder<'ref_str> {
    #[inline]
    fn add_assign(&mut self, rhs: T) {
        self.push(rhs);
    }
}

impl<'ref_str, T: Into<Segment<'ref_str>>> Add<T> for CommonJsStringBuilder<'ref_str> {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: T) -> Self::Output {
        self.push(rhs);
        self
    }
}
