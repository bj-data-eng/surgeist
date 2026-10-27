//! Flex, grid, alignment, column, item-flow and scroll-snap values.
//!
//! # Authored Grid repetition and keyframe structure
//!
//! The six Grid repetition consumers expose authored values through
//! their `value()` accessors. [`CssGridTrackList`] represents `none`, general
//! track lists, lists containing exactly one [`CssGridAutoRepeat`], and subgrid
//! line-name lists. [`CssGridSubgridComponent`] retains ordered groups and
//! [`CssGridNameRepeat`] values, allowing at most one auto-fill name repeat.
//! Counted repeats use checked [`CssNumeric<CssInteger, CssPositive>`](crate::numeric::CssNumeric) children: bare roots
//! must be positive, while function math retains its graph and deferred range.
//! Integer
//! and automatic repetition are non-recursive. Under the
//! [selected Grid 3
//! grammar](https://www.w3.org/TR/2026/WD-css-grid-3-20260121/#intrinsic-auto-repeat),
//! automatic bodies admit general track sizes, including intrinsic and flexible
//! sizes, while surrounding tracks and integer repeats retain fixed sizes.
//! `grid-auto-rows` and `grid-auto-columns` expose
//! [`CssGridTrackSizeList`] values without `repeat()`.
//! [`CssGridAutoRepeat::content`](crate::values::layout::CssGridAutoRepeat::content) therefore borrows
//! [`CssGridTrackRepeatContent`], while surrounding fixed repeats retain
//! [`CssGridFixedRepeatContent`].
//! Track breadths retain exact ordinary `fr`, length, and percentage quantities
//! in checked specified scalars. Flex-result math follows the pinned WebKit
//! behavior where [Grid 2's `fr` rule](https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#fr-unit)
//! and [Values 4 math
//! typing](https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-type-checking)
//! conflict. A flex calculation remains symbolic and cannot enter an inflexible
//! `minmax()` minimum or `fit-content()`. Grid track models provide checked
//! constructors and bounded canonical `serialize_specified()` separately from
//! declaration `as_css()`.
//!
//! # Grid templates
//!
//! [`CssGridTemplate`] also retains area-string rows with optional authored
//! sizes and boundary names through [`CssGridTemplateAreaTrack`], reusing the
//! existing checked area matrix and repeat-free column list. Intrinsic expansion
//! emits three ordered template members or six grid members through shared
//! metadata, strict pending reentry and normalization. Template axes and areas
//! start at `none`; implicit sizes start at `auto`, with the accepted flow initial
//! `normal`. The selected authored repeat and six consuming property productions
//! are Complete. `grid-auto-flow` retains its separate unresolved orientation
//! boundary. Repeat evaluation and used track sizing remain downstream.
//!
//! # Authored item flow
//!
//! Grid3's provisional Appendix A supplies the selected flow-oriented
//! `item-direction`, `item-wrap`, `item-pack` and `item-flow` grammars.
//! Direction, wrap, packing and signed flow tolerance remain authored values;
//! physical axes and used placement require downstream context. A shorthand
//! preserves complete wrap/pack groups and emits all four constituents.
//!
//! ```rust
//! use surgeist_css::property::CssKnownPropertyValueRef;
//! use surgeist_css::values::layout::{CssItemWrap, CssItemWrapOrder};
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents("item-flow: balance dense reverse -2px row");
//! assert!(report.is_clean());
//! let CssKnownPropertyValueRef::ItemFlow(flow) = report.syntax()[0]
//!     .known().unwrap().property_value().unwrap()
//! else { panic!("item-flow value") };
//! assert_eq!(
//!     flow.value().serialize_specified(Default::default()).unwrap(),
//!     "row reverse dense balance -2px",
//! );
//! let order_only = CssItemWrap::Order(CssItemWrapOrder::Reverse);
//! assert_eq!(order_only.serialize_specified(Default::default()).unwrap(), "reverse");
//! ```
//!
//! # Flexbox, multicolumn, and official grammar closure
//!
//! Flexbox 1 `flex-flow` and all nine Multicolumn 1 properties expose typed
//! authored values without performing layout, pagination, or painting.
//! The Sizing 4 `column-width` extension uses the shared `CssSizeValue` box-size
//! grammar; `CssColumns` exposes both effective values and exact positive counts
//! retain ordinary digits beyond `i32`. The generic Syntax 3 authored shells
//! and the remaining selected Values 3 records
//! are public Complete atomic metadata.
//!
//! ```
//! use surgeist_css::values::layout::{CssColumnCount, CssFlexDirection};
//! use surgeist_css::property::CssKnownPropertyValueRef;
//! use surgeist_css::conformance::{CssSupportStatus, feature_metadata};
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents("flex-flow: column wrap; columns: 3 12em");
//! assert!(report.is_clean(), "{:?}", report.diagnostics());
//!
//! let CssKnownPropertyValueRef::FlexFlow(flow) = report.syntax()[0]
//!     .known().expect("known flex-flow")
//!     .property_value().expect("ordinary flex-flow")
//! else { panic!("expected flex-flow") };
//! assert_eq!(flow.value().direction(), CssFlexDirection::Column);
//!
//! let CssKnownPropertyValueRef::Columns(columns) = report.syntax()[1]
//!     .known().expect("known columns")
//!     .property_value().expect("ordinary columns")
//! else { panic!("expected columns") };
//! assert!(matches!(columns.value().count(), CssColumnCount::Count(_)));
//!
//! let metadata = feature_metadata("official.property.flex-flow")
//!     .expect("public Flexbox metadata");
//! assert_eq!(metadata.source().id().as_str(), "O-FLEXBOX1");
//! assert_eq!(metadata.status(), CssSupportStatus::Complete);
//!
//! let shared = feature_metadata("official.value.syntax-token-stream")
//!     .expect("public Syntax 3 value metadata");
//! assert_eq!(shared.status(), CssSupportStatus::Complete);
//!
//! let extension = feature_metadata("ext.value.relative-color")
//!     .expect("preserved extension metadata");
//! assert_eq!(extension.status(), CssSupportStatus::Partial);
//! assert!(extension.supported_subset().is_some());
//! assert!(extension.unsupported_remainder().is_some());
//!
//! let feature_values = feature_metadata("later.rule.font-feature-values")
//!     .expect("authored font-feature-values metadata");
//! assert_eq!(feature_values.status(), CssSupportStatus::Complete);
//! assert_eq!(feature_values.source().id().as_str(), "I-FONTS4-20260907");
//! assert!(feature_values.unsupported_remainder().is_none());
//! ```

pub(crate) mod alignment;
pub(crate) mod column_rule;
pub(crate) mod column_sizing;
pub(crate) mod flex;
pub(crate) mod gap;
pub(crate) mod grid;
pub(crate) mod grid_placement;
pub(crate) mod grid_template_areas;
pub(crate) mod item_flow;
pub(crate) mod scroll_snap;
pub(crate) mod syntax;

#[doc(inline)]
pub use alignment::{
    CssAlignContentValue, CssAlignItemsValue, CssAlignSelfValue, CssAlignmentPosition,
    CssAlignmentValue, CssBaselinePosition, CssJustifyContentValue, CssJustifyItemsValue,
    CssJustifySelfValue, CssLegacyAlignment, CssOverflowPosition, CssPlaceContentValue,
    CssPlaceItemsValue, CssPlaceSelfValue,
};
#[doc(inline)]
pub use column_rule::CssColumnRule;
#[doc(inline)]
pub use flex::{
    CssFlexBasisRef, CssFlexBasisValue, CssFlexComponents, CssFlexDirection, CssFlexFlow,
    CssFlexValue, CssFlexWrap,
};
#[doc(inline)]
pub use gap::CssGapValue;
#[doc(inline)]
pub use grid::{
    CssGrid, CssGridAutoFlow, CssGridAutoFlowAxis, CssGridAutoFlowMode, CssGridAutoRepeat,
    CssGridAutoRepeatKind, CssGridAutoTrackComponent, CssGridAutoTrackList,
    CssGridFixedRepeatComponent, CssGridFixedRepeatContent, CssGridFixedSize,
    CssGridGeneralTrackComponent, CssGridGeneralTrackList, CssGridIntegerFixedRepeat,
    CssGridIntegerTrackRepeat, CssGridLineNames, CssGridNameRepeat, CssGridSubgridComponent,
    CssGridTemplate, CssGridTemplateAreaTrack, CssGridTrackBreadth, CssGridTrackBreadthKind,
    CssGridTrackList, CssGridTrackRepeatComponent, CssGridTrackRepeatContent, CssGridTrackSize,
    CssGridTrackSizeKind, CssGridTrackSizeList,
};
#[doc(inline)]
pub use grid_placement::{
    CssGridArea, CssGridLine, CssGridLineIndex, CssGridLineName, CssGridLineRange, CssGridLineSpan,
};
#[doc(inline)]
pub use grid_template_areas::{
    CssGridTemplateAreaCell, CssGridTemplateAreaError, CssGridTemplateAreaName,
    CssGridTemplateAreaRow, CssGridTemplateAreaRows, CssGridTemplateAreas,
};
#[doc(inline)]
pub use item_flow::{
    CssItemDirection, CssItemFlow, CssItemPack, CssItemWrap, CssItemWrapMode, CssItemWrapOrder,
};
#[doc(inline)]
pub use scroll_snap::{
    CssScrollSnapAlignment, CssScrollSnapAxis, CssScrollSnapStop, CssScrollSnapStrictness,
    CssScrollSnapType,
};
#[doc(inline)]
pub use syntax::{CssColumnCount, CssColumnFill, CssColumnSpan, CssColumns};
