//! Snapshot 2026's selected authored text-box family from Inline3 WD20241218.
//! Only `<text-edge>` is imported from §5.2; line fitting and font metrics are
//! downstream. §6.3's normative table/index and change log define inheritance.
//! The conflicting note remains informative. Editorial commit 97441f74 / PR12765
//! corrects §6.1's omitted-trim spelling to `trim-both`, without adding `both`.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

/// Authored trimming selection, before any content-edge or font computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextBoxTrim {
    None,
    TrimStart,
    TrimEnd,
    TrimBoth,
}

/// Metrics valid as a single `<text-edge>` value for both edges.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextEdgeMetric {
    Text,
    Ideographic,
    IdeographicInk,
}

/// The over role of an explicitly authored `<text-edge>` pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextOverEdge {
    Text,
    Ideographic,
    IdeographicInk,
    Cap,
    Ex,
}

/// The under role of an explicitly authored `<text-edge>` pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextUnderEdge {
    Text,
    Ideographic,
    IdeographicInk,
    Alphabetic,
}

/// Exact authored metric arity and roles; invalid single/pair choices cannot
/// be constructed. An explicit equal pair remains distinct from one keyword.
///
/// ```
/// use surgeist_css::{CssTextEdge, CssTextOverEdge, CssTextUnderEdge};
/// let edge = CssTextEdge::Pair {
///     over: CssTextOverEdge::Cap, under: CssTextUnderEdge::Alphabetic,
/// };
/// assert_eq!(edge.serialize_specified().unwrap(), "cap alphabetic");
/// ```
///
/// Over-only choices cannot become a single metric:
///
/// ```compile_fail
/// use surgeist_css::{CssTextEdge, CssTextOverEdge};
/// let _ = CssTextEdge::Single(CssTextOverEdge::Cap);
/// ```
///
/// Pair roles cannot be exchanged:
///
/// ```compile_fail
/// use surgeist_css::{CssTextEdge, CssTextOverEdge, CssTextUnderEdge};
/// let _ = CssTextEdge::Pair {
///     over: CssTextUnderEdge::Alphabetic, under: CssTextOverEdge::Cap,
/// };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextEdge {
    Single(CssTextEdgeMetric),
    Pair {
        over: CssTextOverEdge,
        under: CssTextUnderEdge,
    },
}

impl CssTextEdge {
    /// Returns the symbolic over metric, without consulting font metrics.
    pub const fn over(&self) -> CssTextOverEdge {
        match self {
            Self::Pair { over, .. } => *over,
            Self::Single(CssTextEdgeMetric::Text) => CssTextOverEdge::Text,
            Self::Single(CssTextEdgeMetric::Ideographic) => CssTextOverEdge::Ideographic,
            Self::Single(CssTextEdgeMetric::IdeographicInk) => CssTextOverEdge::IdeographicInk,
        }
    }

    /// Returns the symbolic under metric, without consulting font metrics.
    pub const fn under(&self) -> CssTextUnderEdge {
        match self {
            Self::Pair { under, .. } => *under,
            Self::Single(CssTextEdgeMetric::Text) => CssTextUnderEdge::Text,
            Self::Single(CssTextEdgeMetric::Ideographic) => CssTextUnderEdge::Ideographic,
            Self::Single(CssTextEdgeMetric::IdeographicInk) => CssTextUnderEdge::IdeographicInk,
        }
    }
}

/// Authored text-box metrics. `auto` stays symbolic until downstream line fit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextBoxEdge {
    Auto,
    Edge(CssTextEdge),
}

/// Nonempty authored shorthand constituents, retaining either omitted slot.
///
/// ```
/// use surgeist_css::{CssTextBoxValues, CssTextBoxEdge};
/// assert!(CssTextBoxValues::try_new(None, None).is_none());
/// let edge_only = CssTextBoxValues::try_new(None, Some(CssTextBoxEdge::Auto)).unwrap();
/// assert_eq!(edge_only.trim(), None);
/// assert_eq!(edge_only.serialize_specified().unwrap(), "auto");
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssTextBoxValues {
    trim: Option<CssTextBoxTrim>,
    edge: Option<CssTextBoxEdge>,
}

impl CssTextBoxValues {
    /// Rejects two omitted slots. Intrinsic expansion, rather than construction,
    /// supplies the omitted trim (`trim-both`) and edge (`auto`).
    #[must_use]
    pub const fn try_new(
        trim: Option<CssTextBoxTrim>,
        edge: Option<CssTextBoxEdge>,
    ) -> Option<Self> {
        if trim.is_some() || edge.is_some() {
            Some(Self { trim, edge })
        } else {
            None
        }
    }

    /// Returns the authored trim slot; omission is not the `none` initial.
    pub const fn trim(&self) -> Option<CssTextBoxTrim> {
        self.trim
    }

    /// Returns the authored edge slot; omission remains explicit.
    pub const fn edge(&self) -> Option<CssTextBoxEdge> {
        self.edge
    }
}

/// The authored shorthand, with exclusive `normal` or nonempty constituents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTextBox {
    Normal,
    Components(CssTextBoxValues),
}

macro_rules! provider {
    ($ty:ty, $value:ident, $writer:ident => $body:expr) => {
        impl $ty {
            /// Emits canonical authored syntax without contextual resolution.
            pub fn serialize_specified(&self) -> Result<String> {
                self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
            }
            /// Emits atomically under shared node and UTF-8 byte limits.
            /// Each emitted keyword costs one input and projection node.
            pub fn serialize_specified_with_limits(&self, limits: CssSpecifiedValueSerializationLimits) -> Result<String> {
                let mut writer = SpecifiedRuleWriter::new(limits);
                self.append_to_rule_writer(&mut writer)?;
                Ok(writer.css)
            }
            pub(crate) fn append_to_rule_writer(&self, $writer: &mut SpecifiedRuleWriter) -> Result<()> {
                let $value = self;
                $body
            }
        }
    };
}
macro_rules! keywords {
    ($ty:ty, $($variant:ident => $text:literal),+ $(,)?) => {
        provider!($ty, value, writer => writer.keyword(match value { $(Self::$variant => $text),+ }));
    };
}
keywords!(CssTextBoxTrim, None => "none", TrimStart => "trim-start", TrimEnd => "trim-end", TrimBoth => "trim-both");
keywords!(CssTextEdgeMetric, Text => "text", Ideographic => "ideographic", IdeographicInk => "ideographic-ink");
keywords!(CssTextOverEdge, Text => "text", Ideographic => "ideographic", IdeographicInk => "ideographic-ink", Cap => "cap", Ex => "ex");
keywords!(CssTextUnderEdge, Text => "text", Ideographic => "ideographic", IdeographicInk => "ideographic-ink", Alphabetic => "alphabetic");
provider!(CssTextEdge, value, writer => match value {
    Self::Single(metric) => metric.append_to_rule_writer(writer),
    Self::Pair { over, under } => { over.append_to_rule_writer(writer)?; writer.append(" ")?; under.append_to_rule_writer(writer) }
});
provider!(CssTextBoxEdge, value, writer => match value {
    Self::Auto => writer.keyword("auto"), Self::Edge(edge) => edge.append_to_rule_writer(writer)
});
provider!(CssTextBoxValues, value, writer => {
    if let Some(trim) = value.trim { writer.source_member(0, |writer| trim.append_to_rule_writer(writer))?; }
    if let Some(edge) = value.edge { writer.source_member(1, |writer| {
        if value.trim.is_some() { writer.append(" ")?; }
        edge.append_to_rule_writer(writer)
    })?; }
    Ok(())
});
provider!(CssTextBox, value, writer => match value {
    Self::Normal => writer.keyword("normal"), Self::Components(values) => values.append_to_rule_writer(writer)
});
