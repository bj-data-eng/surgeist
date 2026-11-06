use crate::{JsStr, JsStringKind};
use std::{
    hash::{Hash, Hasher},
    ops::Range,
    rc::Rc,
};

/// A JavaScript string backed by static code units.
#[derive(Debug, Clone, Copy)]
pub struct StaticString {
    pub(crate) str: JsStr<'static>,
}

impl StaticString {
    /// Creates a static string from borrowed code units.
    #[must_use]
    pub const fn new(str: JsStr<'static>) -> Self {
        Self { str }
    }
}

impl Hash for StaticString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.str.hash(state);
    }
}

impl PartialEq for StaticString {
    fn eq(&self, other: &Self) -> bool {
        self.str == other.str
    }
}

impl Eq for StaticString {}

impl std::borrow::Borrow<JsStr<'static>> for &'static StaticString {
    fn borrow(&self) -> &JsStr<'static> {
        &self.str
    }
}

pub(super) enum Units {
    Latin1(Box<[u8]>),
    Utf16(Box<[u16]>),
}

#[derive(Clone)]
pub(super) enum Root {
    Static(&'static StaticString),
    Sequence(Rc<Units>),
}

impl Root {
    pub(super) fn as_str(&self) -> JsStr<'_> {
        match self {
            Self::Static(string) => string.str,
            Self::Sequence(units) => match units.as_ref() {
                Units::Latin1(units) => JsStr::latin1(units),
                Units::Utf16(units) => JsStr::utf16(units),
            },
        }
    }

    pub(super) fn kind(&self) -> JsStringKind {
        match self {
            Self::Static(_) => JsStringKind::Static,
            Self::Sequence(units) => match units.as_ref() {
                Units::Latin1(_) => JsStringKind::Latin1Sequence,
                Units::Utf16(_) => JsStringKind::Utf16Sequence,
            },
        }
    }

    pub(super) fn refcount(&self) -> Option<usize> {
        match self {
            Self::Static(_) => None,
            Self::Sequence(units) => Some(Rc::strong_count(units)),
        }
    }
}

#[derive(Clone)]
pub(super) enum Storage {
    Root(Root),
    Slice(Rc<SliceString>),
}

pub(super) struct SliceString {
    // The first slice takes one root reference. Descendants share that reference
    // for constant-time views and retain their immediate parent independently.
    pub(super) root: Rc<Root>,
    pub(super) parent: Option<Rc<Self>>,
    range: Range<usize>,
}

impl SliceString {
    pub(super) fn new(source: &Storage, range: Range<usize>) -> Self {
        match source {
            Storage::Root(root) => Self {
                root: Rc::new(root.clone()),
                parent: None,
                range,
            },
            Storage::Slice(parent) => {
                let base = parent.range.start;
                Self {
                    root: Rc::clone(&parent.root),
                    parent: Some(Rc::clone(parent)),
                    range: base + range.start..base + range.end,
                }
            }
        }
    }

    pub(super) fn len(&self) -> usize {
        self.range.len()
    }

    pub(super) fn as_str(&self) -> JsStr<'_> {
        self.root.as_str().get_expect(self.range.clone())
    }
}

impl Drop for SliceString {
    fn drop(&mut self) {
        // Stop when an ancestor is still shared. Uniquely owned ancestors can
        // be detached before dropping them so deep chains use constant stack.
        let mut parent = self.parent.take();
        while let Some(link) = parent {
            let Some(mut node) = Rc::into_inner(link) else {
                break;
            };
            parent = node.parent.take();
        }
    }
}
