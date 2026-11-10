//! Selection of the optional parse detail.
//!
//! A parse records its detail only when its request selects it, so a parse
//! that selects none allocates no storage for it and does none of the work of
//! producing it. The `detail` cargo feature removes the detail types and makes
//! a non-empty selection a typed refusal.

use std::{error::Error, fmt, ops};

/// The kinds of optional detail a parse can record.
///
/// Each flag selects one kind of record:
///
/// - [`Self::SOURCE_MAP`]: the association inventory, which maps each retained
///   field to the source ranges and origin kinds its code units come from.
/// - [`Self::LEXICAL`]: the lexical occurrences and parts, with the omission
///   records of the markup the tree builder dropped.
/// - [`Self::HISTORY`]: the construction history, which keeps every version of
///   a retained field and the construction causes that produced it.
/// - [`Self::DIAGNOSTICS`]: the parse-error diagnostics.
///
/// The source-map, lexical and history records name the origin groups and
/// construction causes of one shared graph, which a parse records when it
/// selects any of the three. Each flag adds only its own records and the
/// reader of them.
///
/// The resources a kind charges, as the `usage` of a report states them:
///
/// - `SourceMapEntries` and `SourceMapPayloadUnits`: the source map, the lexical
///   and omission records and the history, which share the origin and cause
///   graph and its rows, and the locations of the diagnostics.
/// - `Diagnostics` and `DiagnosticPayloadUnits`: the diagnostics.
///
/// A parse that selects none charges and reaches none of the four. The start
/// positions of scripts, styles and event handler content attributes are kept
/// whatever is selected and charge `AuxiliaryEntries`, 16 bytes for each handler
/// attribute, and the pending association values charge the managed storage.
///
/// The default selection is [`Self::NONE`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[must_use]
pub struct DetailSet(u8);

impl DetailSet {
    /// No detail.
    pub const NONE: Self = Self(0);
    /// The association inventory with its source ranges and origin kinds.
    pub const SOURCE_MAP: Self = Self(1);
    /// The lexical occurrences and the omission records.
    pub const LEXICAL: Self = Self(1 << 1);
    /// The construction history and the causes of retained values.
    pub const HISTORY: Self = Self(1 << 2);
    /// The parse-error diagnostics.
    pub const DIAGNOSTICS: Self = Self(1 << 3);
    /// Every kind of detail.
    pub const ALL: Self =
        Self(Self::SOURCE_MAP.0 | Self::LEXICAL.0 | Self::HISTORY.0 | Self::DIAGNOSTICS.0);

    /// Whether no kind is selected.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    /// Whether every kind of `other` is selected.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    /// Whether any kind of `other` is selected.
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
    /// The kinds selected by `self` or by `other`.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    /// Whether the parse records the origin and cause graph that the
    /// source-map, lexical and history records name.
    pub(crate) const fn records_graph(self) -> bool {
        self.intersects(Self(Self::SOURCE_MAP.0 | Self::LEXICAL.0 | Self::HISTORY.0))
    }
}

impl ops::BitOr for DetailSet {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        self.union(other)
    }
}

impl ops::BitOrAssign for DetailSet {
    fn bitor_assign(&mut self, other: Self) {
        *self = self.union(other);
    }
}

/// A request selected detail that this build of the parser does not provide.
///
/// The `detail` cargo feature of `surgeist-html-parser` provides it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlParseDetailUnavailable {
    requested: DetailSet,
}

impl HtmlParseDetailUnavailable {
    pub(crate) fn new(requested: DetailSet) -> Self {
        Self { requested }
    }
    /// The selection that was refused.
    pub fn requested(&self) -> DetailSet {
        self.requested
    }
}

impl fmt::Display for HtmlParseDetailUnavailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("parse detail was selected, but the `detail` feature of surgeist-html-parser is not enabled")
    }
}

impl Error for HtmlParseDetailUnavailable {}

/// The selection a request holds, refused when the build cannot provide it.
pub(crate) fn admit(selected: DetailSet) -> Result<DetailSet, HtmlParseDetailUnavailable> {
    if cfg!(feature = "detail") || selected.is_empty() {
        Ok(selected)
    } else {
        Err(HtmlParseDetailUnavailable::new(selected))
    }
}
