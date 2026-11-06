//! Display implementations for [`JsString`].

use crate::storage::{Root, SliceString, Storage};
use crate::{CodePoint, JsStr, JsStrVariant, JsString, JsStringKind};
use std::fmt;
use std::fmt::Write;

/// `Display` implementation for [`JsString`] that escapes unicode characters.
// This should not implement debug, only be shown as a standard display.
#[allow(missing_debug_implementations)]
pub struct JsStrDisplayEscaped<'a> {
    inner: &'a JsString,
}

impl fmt::Display for JsStrDisplayEscaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.variant() {
            // SAFETY: `JsStrVariant::Latin1` does not contain any unpaired surrogates, so need to check.
            JsStrVariant::Latin1(v) => v
                .iter()
                .copied()
                .map(char::from)
                .try_for_each(|c| f.write_char(c)),
            JsStrVariant::Utf16(_) => self.inner.code_points().try_for_each(|r| match r {
                CodePoint::Unicode(c) => f.write_char(c),
                CodePoint::UnpairedSurrogate(c) => {
                    write!(f, "\\u{c:04X}")
                }
            }),
        }
    }
}

impl<'a> From<&'a JsString> for JsStrDisplayEscaped<'a> {
    fn from(inner: &'a JsString) -> Self {
        Self { inner }
    }
}

/// `Display` implementation for [`JsString`] that escapes unicode characters.
// This should not implement debug, only be shown as a standard display.
#[allow(missing_debug_implementations)]
pub struct JsStrDisplayLossy<'a> {
    inner: JsStr<'a>,
}

impl fmt::Display for JsStrDisplayLossy<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No need to optimize latin1.
        self.inner
            .code_points_lossy()
            .try_for_each(|c| f.write_char(c))
    }
}

impl<'a> From<JsStr<'a>> for JsStrDisplayLossy<'a> {
    fn from(inner: JsStr<'a>) -> Self {
        Self { inner }
    }
}

/// Debug displayable for [`JsString`] which shows more information than
/// debug displaying the original string.
pub struct JsStringDebugInfo<'a> {
    inner: DebugStorage<'a>,
}

enum DebugStorage<'a> {
    Root(&'a Root),
    Slice(&'a SliceString),
}

impl fmt::Debug for JsStringDebugInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (content, kind) = match self.inner {
            DebugStorage::Root(root) => (root.as_str(), root.kind()),
            DebugStorage::Slice(slice) => (slice.as_str(), JsStringKind::Slice),
        };
        let s_repr = if content.len() > 30 {
            let it = content
                .code_points()
                .map(|c| c.as_char().unwrap_or('\u{FFFD}'));
            it.clone()
                .take(20)
                .chain("/* ... */".chars())
                .chain(it.skip(content.len() - 20))
                .collect()
        } else {
            JsStrDisplayLossy::from(content).to_string()
        };

        let mut dbg = f.debug_struct("JsString");
        dbg.field("kind", &kind)
            .field("length", &content.len())
            .field("content", &s_repr);

        match self.inner {
            DebugStorage::Root(root) => {
                if let Some(count) = root.refcount() {
                    dbg.field("refcount", &count);
                }
            }
            DebugStorage::Slice(slice) => {
                let original = slice
                    .parent
                    .as_deref()
                    .map_or(DebugStorage::Root(&slice.root), DebugStorage::Slice);
                dbg.field("original", &Self { inner: original });
            }
        }

        dbg.finish()
    }
}

impl<'a> From<&'a JsString> for JsStringDebugInfo<'a> {
    fn from(inner: &'a JsString) -> Self {
        let inner = match &inner.storage {
            Storage::Root(root) => DebugStorage::Root(root),
            Storage::Slice(slice) => DebugStorage::Slice(slice),
        };
        Self { inner }
    }
}

#[test]
fn latin1() {
    // 0xE9 is `é` in ISO-8859-1 (see https://www.ascii-code.com/ISO-8859-1).
    let s = JsString::from("Hello \u{E9} world!");

    let rust_str = format!("{}", JsStrDisplayEscaped { inner: &s });
    assert_eq!(rust_str, "Hello é world!");

    let rust_str = format!("{}", JsStrDisplayLossy { inner: s.as_str() });
    assert_eq!(rust_str, "Hello é world!");
}

#[test]
fn emoji() {
    // 0x1F600 is `😀` (see https://www.fileformat.info/info/unicode/char/1f600/index.htm).
    let s = JsString::from(&[0xD83D, 0xDE00]);

    let rust_str = format!("{}", JsStrDisplayEscaped { inner: &s });
    assert_eq!(rust_str, "😀");

    let rust_str = format!("{}", JsStrDisplayLossy { inner: s.as_str() });
    assert_eq!(rust_str, "😀");
}

#[test]
fn unpaired_surrogates() {
    // 0xD800 is an unpaired surrogate (see https://www.fileformat.info/info/unicode/char/d800/index.htm).
    let s = JsString::from(&[0xD800]);

    let rust_str = format!("{}", JsStrDisplayEscaped { inner: &s });
    assert_eq!(rust_str, "\\uD800");

    let rust_str = format!("{}", JsStrDisplayLossy { inner: s.as_str() });
    assert_eq!(rust_str, "�");
}
