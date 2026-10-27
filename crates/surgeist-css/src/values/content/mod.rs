//! Generated content, quotes, counters and list values.
//!
//! Generated-content values: `content` items (strings, counters, `target-*()` references, named
//! strings and leaders), `quotes`, the counter properties, list styles and markers, `string-set` and
//! bookmark properties. Counters and quotes are retained symbolically; none is evaluated.

pub(crate) mod content_serialization;
pub(crate) mod content_values;
pub(crate) mod counter_changes;
pub(crate) mod counter_changes_serialization;
pub(crate) mod list_style_serialization;
pub(crate) mod list_styles;
pub(crate) mod quotes;
pub(crate) mod syntax;

#[doc(inline)]
pub use content_values::{
    CssBookmarkLevel, CssBookmarkState, CssContentAlternative, CssContentAlternativeItem,
    CssContentCounter, CssContentCounterName, CssContentCounters, CssContentList, CssContentName,
    CssContentReferenceMode, CssContentTarget, CssContentValue, CssContentValueItem,
    CssCounterStyleReference, CssCounterStyleValue, CssCounterSymbolValue, CssGeneratedContent,
    CssGeneratedContentBodyRef, CssLeaderValue, CssNamedString, CssNamedStringMode, CssStringSet,
    CssStringSetEntry, CssSymbolsStyleValue, CssSymbolsSystem, CssTargetCounter, CssTargetCounters,
    CssTargetText, CssTargetTextMode,
};
#[doc(inline)]
pub use counter_changes::{CssCounterChangeValue, CssCounterChangesValue, CssCounterProperty};
#[doc(inline)]
pub use list_styles::{CssListStyleTypeValue, CssListStyleValue, CssMarkerSide};
#[doc(inline)]
pub use syntax::{
    CssContentString, CssListStylePosition, CssQuotePair, CssQuotePairList, CssQuotes,
};
