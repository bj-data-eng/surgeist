//! Native layout algorithm boundary for Surgeist.
//!
//! The public physical geometry contract uses x/y points, width/height sizes,
//! and top/right/bottom/left edges. Public layout outputs, cached geometry, and
//! scroll geometry remain physical. Layout algorithms may use
//! crate-private logical algorithm geometry while working in inline/block
//! coordinates. Those carriers stay private until the owning [`FlowAxes`](crate::geometry::FlowAxes)
//! projects them to physical geometry at a contextual boundary.
//!
//! [`FlowAxes`](crate::geometry::FlowAxes) is the sole production owner of writing-mode mapping for
//! [`WritingMode::HorizontalTb`](crate::geometry::WritingMode::HorizontalTb), [`WritingMode::VerticalRl`](crate::geometry::WritingMode::VerticalRl),
//! [`WritingMode::VerticalLr`](crate::geometry::WritingMode::VerticalLr), [`WritingMode::SidewaysRl`](crate::geometry::WritingMode::SidewaysRl), and
//! [`WritingMode::SidewaysLr`](crate::geometry::WritingMode::SidewaysLr). Its [`Direction`](crate::geometry::Direction) is the already-resolved
//! used inline direction, not authored or otherwise unresolved CSS. Layout owns
//! its consuming computed-style adapter; callers supply the resolved value at
//! the current normalized input boundary.
//!
//! Scroll input is normalized, computed or otherwise layout-ready input rather
//! than authored CSS. [`ComputedOverflow`](crate::overflow::ComputedOverflow) is an atomic canonical pair; layout
//! privately derives used overflow from that pair and box-generation facts.
//! [`ScrollbarWidthOf`](crate::overflow::ScrollbarWidthOf) is the explicit finite thickness selected by the caller's
//! scrollbar environment, including zero for overlay or disabled scrollbars.
//! Layout does not discover host scrollbar metrics or guess a missing policy.
//!
//! [`ScrollGeometryOf`](crate::scroll::ScrollGeometryOf) is immutable, layout-produced physical geometry. Its
//! signed x/y range contains the zero initial anchor, and an axis's scroll size
//! is the span `maximum - minimum`, including an explicit zero span. Nested
//! [`ScrollTargetGeometryOf`](crate::scroll::ScrollTargetGeometryOf) preserves the target border box, physical scroll
//! margin, flow axes, and snap metadata used to integrate that target later.
//! The general physical and flow-relative range types also keep finite ordered
//! bounds; when an axis runs in reverse, [`FlowAxes`](crate::geometry::FlowAxes) swaps and negates endpoints
//! so signed minima and maxima retain their meaning.
//!
//! [`PreferredSizeOf`](crate::sizing::PreferredSizeOf), [`MinSizeOf`](crate::sizing::MinSizeOf), [`MaxSizeOf`](crate::sizing::MaxSizeOf), and [`FlexBasisOf`](crate::sizing::FlexBasisOf) are
//! distinct closed property domains with only their role-valid keywords.
//! Direct [`FlexBasisOf::MIN_CONTENT`](crate::sizing::FlexBasisOf::MIN_CONTENT) and [`FlexBasisOf::MAX_CONTENT`](crate::sizing::FlexBasisOf::MAX_CONTENT) values
//! retain their distinct intrinsic measurement constraints through the public
//! layout front door; neither is normalized to the generic content basis.
//! [`SizingCalculationOf`](crate::sizing::SizingCalculationOf) preserves finite affine leaves and nested `min`,
//! `max`, and `clamp` in a validated program evaluated iteratively. Percentages
//! remain symbolic until an explicit [`PercentageBasisOf`](crate::length::PercentageBasisOf) is available; a
//! required missing basis remains unresolved rather than being guessed.
//!
//! [`NodeInputOf::flex_item_collapse`](crate::node::NodeInputOf::flex_item_collapse) is a normalized, layout-ready flex
//! effect, and [`FlexItemCollapse::Normal`](crate::flex::FlexItemCollapse::Normal) is its default. A collapsed in-flow
//! flex item participates through a finite cross-size strut replay, publishes
//! zero committed collapsed geometry, and hides its descendants. Layout
//! owns lowering a flex item's computed `visibility: collapse` into this
//! normalized state, while rendering owns painting. This leaf does not parse
//! authored CSS or provide a general visibility model.
//!
//! Canonical layout-ready `calc-size()` input pairs the property's calc-size
//! basis with a validated [`CalcSizeCalculationOf`](crate::sizing::CalcSizeCalculationOf) containing finite length,
//! percentage, and size coefficients. Track flex remains track-only: callers
//! validate a finite, non-negative [`TrackFlexFactorOf`](crate::grid::TrackFlexFactorOf) and place it only in a
//! [`MaxTrackSizingOf`](crate::grid::MaxTrackSizingOf) breadth. Later-owned valid behavior returns
//! [`LayoutUnsupportedCapability::SizingBehavior`](crate::error::LayoutUnsupportedCapability::SizingBehavior) with an exact
//! [`UnsupportedSizingBehavior`](crate::error::UnsupportedSizingBehavior) property, behavior, algorithm, and axis.
//!
//! [`LayoutRootRequestOf`](crate::request::LayoutRootRequestOf) carries validated dimensions and explicit root context
//! for [`compute_layout`], which
//! returns either a complete [`CompletedLayoutBatchOf`](crate::batch::CompletedLayoutBatchOf) or a typed
//! [`LayoutErrorOf`](crate::error::LayoutErrorOf) with no partial public result. Recursive compute modes are
//! internal.
//!
//! [`LayoutFlowInput`](crate::text_layout::LayoutFlowInput) binds immutable Text paragraphs and declared
//! [`WrappingScopeInput`](crate::inline_wrapping::WrappingScopeInput) boxes to one borrowed tree. Numeric box/control flow
//! can supply no Text paragraphs. [`compute_layout_with_flow`] consumes that
//! checked flow and selects Text-issued candidates when present. It retains the exact
//! placed [`surgeist_text::layout::Layout`] in each [`PlacedTextOutput`](crate::text_layout::PlacedTextOutput), alongside
//! source-associated fragment handles and canonical atomic allocations.
//! Text owns shaping and source mappings; Layout owns fitting and placement,
//! Render owns paint conversion, and Runtime owns coherent publication.
//! The prepared-text profile supports horizontal LTR and RTL flow, signed
//! indentation, and font-advance or CSS-stop tabs. [`TextAlign`](crate::text_flow::TextAlign) and
//! [`TextAlignLast`](crate::text_flow::TextAlignLast) select line alignment. [`TextWrapStyle`](crate::text_flow::TextWrapStyle) selects ordinary
//! or balanced breaks; [`TextWrapStyle::Stable`](crate::text_flow::TextWrapStyle::Stable) retains ordinary line-local
//! quality selection. [`TextWrapStyle::AvoidShortLastLine`](crate::text_flow::TextWrapStyle::AvoidShortLastLine) attempts bounded
//! final-pair repair in a Paragraph-ending group. [`TextWrapStyle::Pretty`](crate::text_flow::TextWrapStyle::Pretty)
//! composes that repair with bounded body-line improvement at each actual group
//! entry. See the [native quality policy](../docs/reference.md#node-input).
//! These choices do not
//! change the real containing geometry
//! or wrapping permission. [`TextJustification::Spaces`](crate::text_flow::TextJustification::Spaces) permits
//! explicit expansion of actual retained U+0020 slots during justification.
//! [`TextJustification::InterWord`](crate::text_flow::TextJustification::InterWord) consumes Text-qualified separators and exact
//! alignment feedback, including actual tab response and unexpandable fallback.
//! [`TextJustification::InterCharacter`](crate::text_flow::TextJustification::InterCharacter) uses the same allocator over Text-qualified
//! visual character boundaries, retaining protected/constrained provider geometry.
//! [`TextJustification::None`](crate::text_flow::TextJustification::None)
//! adds no expansion. Text owns line-edge and trailing-space decisions.
//! Other Text orientations return a precise unsupported error.
//! [`NodeInputOf::non_box`](crate::node::NodeInputOf::non_box) is the canonical tree companion for text and inline
//! controls.
//!
//! Completed batches keep unrounded and final [`InlineFragmentOutputEntryOf`](crate::batch::InlineFragmentOutputEntryOf)
//! phases alongside node and cache state. [`compute_layout_invalidated`] stages
//! the exact invalidation closure, and [`CompletedLayoutBatchOf::apply_to`](crate::batch::CompletedLayoutBatchOf::apply_to)
//! uses [`LayoutBatchSink`](crate::tree::LayoutBatchSink) to prepare an owned replacement immutably before an
//! infallible exclusive commit.
//!
//! Floating boxes select the closed [`FloatExclusion`](crate::box_model::FloatExclusion) contract. `MarginBox` is
//! the default; `Shape` promises a tree provider that accepts a validated
//! physical [`FloatExclusionQueryOf`](crate::box_model::FloatExclusionQueryOf) and returns a clipped
//! [`FloatExclusionIntervalOf`](crate::box_model::FloatExclusionIntervalOf). Float placement and independent block sizing
//! and collision use the floats' physical margin boxes for both variants,
//! without shape queries. Layout invokes the provider for inline wrapping
//! across an overlapping finite candidate band. Missing providers, provider
//! failures, and mismatched interval queries retain typed float/query diagnostics
//! and return no partial batch. Required shape intervals never fall back to
//! rectangular exclusion.
//! Provider facts change through explicit tree invalidation rather than a
//! cache-key revision.
//!
//! `compute_leaf` is the direct fallible measurement boundary: providers receive
//! non-negative content-space constraints and provider failures or invalid output
//! become typed layout errors. `DefaultScalar` uses `f32`; generic
//! `*Of<S>` contracts support end-to-end `f32` and `f64` scalar lanes.
//!
//! [`ItemOrder`](crate::node::ItemOrder) is the layout-ready signed order value. [`SourceIndex`](crate::node::SourceIndex) is
//! stable source-sibling identity: outputs remain source-associated while flex,
//! ordinary grid, and grid-lanes consume one stable order-modified traversal
//! sorted by item order and then source index.
//!
//! [`NodeInputOf::item_is_replaced`](crate::node::NodeInputOf::item_is_replaced) is an independent box-generation fact, not
//! inferred from table role, measurement, aspect ratio, or stretch. Block and
//! root sizing use it to avoid ordinary auto-inline fill; flex uses it for
//! automatic main-size suggestion selection; and grid and grid-lanes use it for
//! normal alignment while preserving explicit stretch.
//!
//! [`ContainingLayoutContext`](crate::request::ContainingLayoutContext) and [`ParentFormattingContext`](crate::request::ParentFormattingContext) form the complete
//! containing context and cache identity, including explicit containing flow and
//! parent role. [`SizingRequestOf`](crate::constraint::SizingRequestOf) independently carries physical available space,
//! per-axis sizing operations, and independent ICB, nearest-scrollport and local
//! containing-bound authority for orthogonal fallback.
//! [`FlexItemRootContext`](crate::request::FlexItemRootContext) keeps actual parent flow axes, imposed host allocation,
//! and percentage bases separate from those request dimensions and operations.
//!
//! Browser-parity generation is crate-local tooling. `generate` is the
//! managed-pinned mode and may use the configured fetcher for the exact manifest
//! pin. `generate-existing` is the existing-pinned, no-fetch mode and accepts
//! only a repository-relative executable under that manifest cache whose exact
//! `--version` matches the pin. Both use the shared headless launch profile,
//! including the mock-keychain argument; corpus freshness checks remain
//! browser-free.
//!
//! Layout owns its direct adapters from lower-level producers. Callers supply
//! normalized style/box facts, host scrollbar policy and retained association;
//! the respective consumers own live scrolling, CSSOM, rendering and host events.
//! Runtime coordinates publication; root `surgeist` provides optional facade
//! composition and bootstrap. This crate emits canonical geometry and target
//! metadata; it does not parse authored CSS, resolve computed style or own a live
//! scrolling runtime.
//!
//! The normalized scroll input and read-only output surface composes through the
//! crate root in either scalar lane without constructing derived geometry:
//!
//! ```
//! use surgeist_layout::{node::{NodeInputOf, NodeOutputOf}, overflow::{ComputedOverflow, Overflow, OverflowClipMarginOf, ScrollbarWidthOf, ScrollMarginOf, ScrollPaddingOf}, scroll::ScrollGeometryOf};
//!
//! fn normalized_input<S: surgeist_layout::scalar::LayoutScalar>() -> NodeInputOf<S> {
//! let mut input = NodeInputOf::<S>::default();
//! input.overflow = ComputedOverflow::try_new(Overflow::Auto, Overflow::Scroll).unwrap();
//! input.overflow_clip_margin = OverflowClipMarginOf::try_new(
//! surgeist_layout::overflow::OverflowClipBox::PaddingBox,
//! S::ZERO,
//! ).unwrap();
//! input.scrollbar_width = ScrollbarWidthOf::try_new(S::ZERO).unwrap();
//! input.scroll_padding = ScrollPaddingOf::default();
//! input.scroll_margin = ScrollMarginOf::try_new(S::ZERO, S::ZERO, S::ZERO, S::ZERO).unwrap();
//! input
//! }
//!
//! fn inspect<S: surgeist_layout::scalar::LayoutScalar>(output: NodeOutputOf<S>) {
//! let geometry: Option<ScrollGeometryOf<S>> = output.scroll_geometry;
//! if let Some(geometry) = geometry {
//! let x = geometry.physical_range().x();
//! let target = geometry.target();
//! let _ = (x.minimum(), x.maximum(), target.border_box(), target.snap_align());
//! }
//! }
//!
//! let _ = normalized_input::<f32>();
//! let _ = normalized_input::<f64>();
//! inspect(NodeOutputOf::<f64>::default());
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::{geometry::Point, node::NodeInput, overflow::Overflow};
//! let mut input = NodeInput::DEFAULT;
//! input.overflow = Point::new(Overflow::Visible, Overflow::Visible);
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::{geometry::Size, node::NodeOutput};
//! let mut output = NodeOutput::default();
//! output.scrollbar_size = Size::ZERO;
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::scroll::{ScrollbarGutterRects, ScrollGeometry};
//! let _ = ScrollGeometry::default();
//! let _ = ScrollbarGutterRects::default();
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::overflow::ScrollPaddingValue;
//! let _ = ScrollPaddingValue::Deferred;
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::node::NodeOutput;
//! let output = NodeOutput::default();
//! let _ = output.current_scroll_offset;
//! ```
//!
//! ```compile_fail
//! use surgeist_layout::{LogicalEdgesOf, LogicalPointOf, LogicalRectOf, LogicalSizeOf};
//! ```
//!
//! The imports of the failing examples above resolve:
//!
//! ```no_run
//! use surgeist_layout::geometry::{Point, Size};
//! use surgeist_layout::node::{NodeInput, NodeOutput};
//! use surgeist_layout::overflow::{Overflow, ScrollPaddingValue};
//! use surgeist_layout::scroll::{ScrollGeometry, ScrollbarGutterRects};
//! ```

#[cfg(test)]
#[path = "../tests/layout/browser_parity_modules.rs"]
mod browser_parity_modules;
#[cfg(test)]
use browser_parity_modules::*;

pub mod baseline;
pub mod batch;
pub mod box_model;
pub mod cache;
pub mod constraint;
pub mod error;
pub mod flex;
pub mod fragmentation;
pub mod geometry;
pub mod grid;
pub mod inline;
pub mod inline_wrapping;
pub mod length;
pub mod node;
pub mod overflow;
pub mod request;
pub mod resize_observer;
pub mod scalar;
pub mod scroll;
pub mod sizing;
pub mod text_flow;
pub mod text_layout;
pub mod tree;

mod accounting;
mod baseline_offset;
mod block;
mod block_fragmentation;
mod engine;
mod group_alignment;
mod hyphenation;
mod layout_math;
mod line_box;
mod measurement;
mod node_input;
mod node_projection;
mod output;
mod text_indent;
mod value;

pub use engine::{
    compute_layout, compute_layout_invalidated, compute_layout_with_flow,
    compute_layout_with_flow_invalidated,
};

#[cfg(test)]
mod block_tests;
#[cfg(test)]
mod cache_tests;
#[cfg(test)]
mod compute_tests;
#[cfg(test)]
mod contract_tests;
#[cfg(test)]
mod flex_tests;
#[cfg(test)]
mod inline_tests;
#[cfg(test)]
mod invalidation_transaction_tests;
#[cfg(test)]
mod leaf_tests;
#[cfg(test)]
mod lib_tests;
#[cfg(test)]
mod root_tests;
#[cfg(test)]
mod scroll_tests;
#[cfg(test)]
mod test_support;
