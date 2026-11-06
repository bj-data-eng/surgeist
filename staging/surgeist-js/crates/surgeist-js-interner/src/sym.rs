use core::num::NonZeroUsize;
use surgeist_js_gc::{Finalize, Trace, empty_trace};
use surgeist_js_macros::static_syms;

/// The string symbol type for Boa.
///
/// This symbol type is internally a `NonZeroUsize`, which makes it pointer-width in size and it's
/// optimized so that it can occupy 1 pointer width even in an `Option` type.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Finalize)]
pub struct Sym {
    value: NonZeroUsize,
}

// SAFETY: `NonZeroUsize` is a constrained `usize`, and all primitive types don't need to be traced
// by the garbage collector.
unsafe impl Trace for Sym {
    empty_trace!();
}

impl Sym {
    /// Creates a new [`Sym`] from the provided `value`, or returns `None` if `value` is zero.
    pub(super) const fn new(value: usize) -> Option<Self> {
        match NonZeroUsize::new(value) {
            Some(value) => Some(Self { value }),
            None => None,
        }
    }

    /// Checks if this symbol is one of the [reserved identifiers][spec] of the ECMAScript
    /// specification, excluding `await` and `yield`
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ReservedWord
    #[inline]
    #[must_use]
    pub fn is_reserved_identifier(self) -> bool {
        (Self::BREAK..=Self::WITH).contains(&self)
    }

    /// Checks if this symbol is one of the [strict reserved identifiers][spec] of the ECMAScript
    /// specification.
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ReservedWord
    #[inline]
    #[must_use]
    pub fn is_strict_reserved_identifier(self) -> bool {
        (Self::IMPLEMENTS..=Self::YIELD).contains(&self)
    }

    /// Returns the internal value of the [`Sym`]
    #[inline]
    #[must_use]
    pub const fn get(self) -> usize {
        self.value.get()
    }
}

static_syms! {
    // Reserved identifiers
    // See: <https://tc39.es/ecma262/#prod-ReservedWord>
    // Note, they must all be together.
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    // End reserved identifier

    // strict reserved identifiers.
    // See: <https://tc39.es/ecma262/#prod-Identifier>
    // Note, they must all be together.
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    // End strict reserved identifiers

    ("", EMPTY_STRING),
    "prototype",
    "constructor",
    "arguments",
    "eval",
    "RegExp",
    "get",
    "set",
    ("<main>", MAIN),
    "raw",
    "anonymous",
    "async",
    "of",
    "target",
    "as",
    "from",
    "__proto__",
    "name",
    "await",
    ("*default*", DEFAULT_EXPORT),
    "meta",
    "defer",
    "source",
    "using",
    "dispose",
    "asyncDispose"
}
