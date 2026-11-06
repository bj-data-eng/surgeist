//! Boa parser input source types.

use crate::frontend::FrontendLimit;
use std::{
    fmt,
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

pub use utf8::UTF8Input;
pub use utf16::UTF16Input;

mod utf16;
mod utf8;

/// A source of ECMAScript code.
///
/// [`Source`]s can be created from plain [`str`]s, file [`Path`]s or more generally, any [`Read`]
/// instance.
#[derive(Debug)]
pub struct Source<'path, R> {
    pub(crate) reader: R,
    pub(crate) path: Option<&'path Path>,
}

impl<'bytes> Source<'static, UTF8Input<&'bytes [u8]>> {
    /// Creates a new `Source` from any type equivalent to a slice of bytes e.g. [`&str`][str],
    /// <code>[Vec]<[u8]></code>, <code>[Box]<[\[u8\]][slice]></code> or a plain slice
    /// <code>[&\[u8\]][slice]</code>.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js_parser::Source;
    /// let code = r#"var array = [5, 4, 3, 2, 1];"#;
    /// let source = Source::from_bytes(code);
    /// ```
    ///
    /// [slice]: std::slice
    pub fn from_bytes<T: AsRef<[u8]> + ?Sized>(source: &'bytes T) -> Self {
        Self {
            reader: UTF8Input::new(source.as_ref()),
            path: None,
        }
    }
}

impl<'input> Source<'static, UTF16Input<'input>> {
    /// Creates a new `Source` from a UTF-16 encoded slice e.g. <code>[&\[u16\]][slice]</code>.
    ///
    /// # Examples
    ///
    /// ```
    /// # use surgeist_js_parser::Source;
    /// let utf16: Vec<u16> =
    ///     "var array = [5, 4, 3, 2, 1];".encode_utf16().collect();
    /// let source = Source::from_utf16(&utf16);
    /// ```
    ///
    /// [slice]: std::slice
    #[must_use]
    pub fn from_utf16(input: &'input [u16]) -> Self {
        Self {
            reader: UTF16Input::new(input),
            path: None,
        }
    }
}

impl<'path> Source<'path, UTF8Input<BufReader<File>>> {
    /// Creates a new `Source` from a `Path` to a file.
    ///
    /// # Errors
    ///
    /// See [`File::open`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use surgeist_js_parser::Source;
    /// # use std::{fs::File, path::Path};
    /// # fn main() -> std::io::Result<()> {
    /// let path = Path::new("script.js");
    /// let source = Source::from_filepath(path)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_filepath(source: &'path Path) -> io::Result<Self> {
        let reader = File::open(source)?;
        Ok(Self {
            reader: UTF8Input::new(BufReader::new(reader)),
            path: Some(source),
        })
    }
}

impl<'path, R: Read> Source<'path, UTF8Input<R>> {
    /// Creates a new `Source` from a [`Read`] instance and an optional [`Path`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use surgeist_js_parser::Source;
    /// # use std::{fs::File, io::Read, path::Path};
    /// # fn main() -> std::io::Result<()> {
    /// let strictler = r#""use strict""#;
    ///
    /// let path = Path::new("no_strict.js");
    /// let file = File::open(path)?;
    /// let strict = strictler.as_bytes().chain(file);
    ///
    /// let source = Source::from_reader(strict, Some(path));
    /// #    Ok(())
    /// # }
    /// ```
    pub fn from_reader(reader: R, path: Option<&'path Path>) -> Self {
        Self {
            reader: UTF8Input::new(reader),
            path,
        }
    }
}

impl<'path, F: FnMut() -> io::Result<Option<u32>>> Source<'path, HostInput<F>> {
    /// Creates source from a host decoder that supplies one code point per call.
    ///
    /// One read unit admits the entire callback, including an end-of-input probe.
    /// Surrogate values are preserved. Values above U+10FFFF are invalid input.
    pub fn from_decoder(decoder: F, path: Option<&'path Path>) -> Self {
        Self {
            reader: HostInput { decoder },
            path,
        }
    }
}

impl<'path, R> Source<'path, R> {
    /// Sets the path of this [`Source`].
    pub fn with_path(self, new_path: &Path) -> Source<'_, R> {
        Source {
            reader: self.reader,
            path: Some(new_path),
        }
    }

    /// Returns the path (if any) of this source file.
    pub fn path(&self) -> Option<&'path Path> {
        self.path
    }
}

/// One primitive read attempted by a source decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadUnit {
    /// Read one byte, including an end-of-input probe.
    Utf8Byte,
    /// Inspect one UTF-16 code unit, including an end-of-input probe.
    Utf16Unit,
    /// Perform a read using a host-provided decoder.
    HostRead,
}

/// A decoder's failure to supply the next character.
#[derive(Debug)]
pub enum ReadError {
    /// The backing reader failed.
    Io(io::Error),
    /// A read was refused before contacting the backing reader.
    Limit(FrontendLimit),
}

impl fmt::Display for ReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Limit(limit) => limit.fmt(formatter),
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Limit(limit) => Some(limit),
        }
    }
}

impl From<io::Error> for ReadError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<FrontendLimit> for ReadError {
    fn from(limit: FrontendLimit) -> Self {
        Self::Limit(limit)
    }
}

/// This trait is used to abstract over the different types of input readers.
pub trait ReadChar {
    /// Retrieves the next unicode code point. Returns `None` if the end of the input is reached.
    ///
    /// Call `admit` before each primitive read. A decoder whose inner reader
    /// is opaque admits one `ReadUnit::HostRead` before calling it.
    ///
    /// # Errors
    ///
    /// Returns the first read error or refusal.
    fn next_char(
        &mut self,
        admit: &mut impl FnMut(ReadUnit) -> Result<(), FrontendLimit>,
    ) -> Result<Option<u32>, ReadError>;
}

/// Source input provided by an opaque host decoder.
pub struct HostInput<F> {
    decoder: F,
}

impl<F: FnMut() -> io::Result<Option<u32>>> ReadChar for HostInput<F> {
    fn next_char(
        &mut self,
        admit: &mut impl FnMut(ReadUnit) -> Result<(), FrontendLimit>,
    ) -> Result<Option<u32>, ReadError> {
        admit(ReadUnit::HostRead)?;
        let code_point = (self.decoder)()?;
        if code_point.is_some_and(|value| value > 0x10_FFFF) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "host decoder returned a code point outside Unicode",
            )
            .into());
        }
        Ok(code_point)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn from_bytes() {
        let mut source = Source::from_bytes("'Hello' + 'World';");

        assert!(source.path.is_none());

        let mut content = String::new();
        while let Some(c) = source.reader.next_char(&mut |_| Ok(())).unwrap() {
            content.push(char::from_u32(c).unwrap());
        }

        assert_eq!(content, "'Hello' + 'World';");
    }

    #[test]
    fn from_filepath() {
        let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"));
        let filepath = manifest_path.join("src/parser/tests/test.js");
        let mut source = Source::from_filepath(&filepath).unwrap();

        assert_eq!(source.path, Some(&*filepath));

        let mut content = String::new();
        while let Some(c) = source.reader.next_char(&mut |_| Ok(())).unwrap() {
            content.push(char::from_u32(c).unwrap());
        }

        assert_eq!(content, "\"Hello\" + \"World\";\n");
    }

    #[test]
    fn from_reader() {
        // Without path
        let mut source = Source::from_reader(Cursor::new("'Hello' + 'World';"), None);

        assert!(source.path.is_none());

        let mut content = String::new();
        while let Some(c) = source.reader.next_char(&mut |_| Ok(())).unwrap() {
            content.push(char::from_u32(c).unwrap());
        }

        assert_eq!(content, "'Hello' + 'World';");

        // With path
        let mut source =
            Source::from_reader(Cursor::new("'Hello' + 'World';"), Some("test.js".as_ref()));

        assert_eq!(source.path, Some("test.js".as_ref()));

        let mut content = String::new();
        while let Some(c) = source.reader.next_char(&mut |_| Ok(())).unwrap() {
            content.push(char::from_u32(c).unwrap());
        }

        assert_eq!(content, "'Hello' + 'World';");
    }
}
