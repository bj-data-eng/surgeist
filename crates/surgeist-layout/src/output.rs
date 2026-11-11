use super::{
    AvailableSpaceOf, AxisSizingRequestOf, CacheKeyContext, DefaultScalar, Edges, LayoutScalar,
    NonNegativeFiniteOf, NonNegativeFiniteScalarErrorOf, PhysicalAxis, Point, ScrollGeometryOf,
    ScrollRectOf, Size, SizingOperation, SizingRequestOf,
};
use crate::geometry::{FlowAxes, PhysicalSide};
use crate::scroll::SettledAutoScrollbarState;

/// The resolved formatting algorithm of the generated parent containing a box.
///
/// This layout-ready role describes the parent that schedules the current box;
/// it does not describe the current box's own display or whether the box is in
/// flow. [`Self::Grid`] covers both ordinary grid and grid-lanes scheduling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParentFormattingContext {
    /// No generated parent contains the box.
    NoParent,
    /// A block formatting algorithm contains the box.
    BlockFlow,
    /// A flex formatting algorithm contains the box.
    Flex,
    /// A grid or grid-lanes formatting algorithm contains the box.
    Grid,
}

/// Resolved, layout-ready context supplied by the box's containing boundary.
///
/// The containing flow mapping and parent formatting role travel as one value.
/// Every combination is valid: flow axes remain meaningful for percentage and
/// logical-axis resolution even when [`ParentFormattingContext::NoParent`] is
/// selected.
///
/// This context has no implicit default; its owner must supply both resolved
/// components explicitly.
///
/// ```compile_fail
/// use surgeist_layout::ContainingLayoutContext;
///
/// let _ = ContainingLayoutContext::default();
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContainingLayoutContext {
    flow_axes: FlowAxes,
    formatting_context: ParentFormattingContext,
}

impl ContainingLayoutContext {
    /// Keeps the resolved containing flow and parent role together.
    #[must_use]
    pub const fn new(flow_axes: FlowAxes, formatting_context: ParentFormattingContext) -> Self {
        Self {
            flow_axes,
            formatting_context,
        }
    }

    /// Returns the resolved flow mapping of the containing boundary.
    #[must_use]
    pub const fn flow_axes(self) -> FlowAxes {
        self.flow_axes
    }

    /// Returns the resolved formatting role of the generated parent.
    #[must_use]
    pub const fn formatting_context(self) -> ParentFormattingContext {
        self.formatting_context
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RunMode {
    PerformRootLayout,
    PerformLayout,
    ComputeSize,
    PerformHiddenLayout,
}

impl RunMode {
    pub const fn is_perform_layout(self) -> bool {
        matches!(self, Self::PerformRootLayout | Self::PerformLayout)
    }

    pub const fn for_child(self) -> Self {
        match self {
            Self::PerformRootLayout => Self::PerformLayout,
            mode => mode,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SizingMode {
    ContentSize,
    InherentSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RequestedAxis {
    Horizontal,
    Vertical,
    Both,
}

/// One actual consumer's resolved source set, independent of child context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NamedBaselineSet {
    First,
    Last,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NamedBaselineDemand {
    None,
    Fact {
        name: crate::BaselineName,
        set: NamedBaselineSet,
    },
}

impl NamedBaselineDemand {
    pub(crate) fn for_source(
        name: crate::BaselineName,
        source: crate::BaselineSource,
        display: crate::Display,
    ) -> Self {
        Self::Fact {
            name,
            set: if source.prefers_last(display) {
                NamedBaselineSet::Last
            } else {
                NamedBaselineSet::First
            },
        }
    }

    pub(crate) fn selected_fact<S: LayoutScalar>(
        self,
        facts: NamedBaselinesOf<S>,
    ) -> Option<(crate::BaselineName, BaselineFactOf<S>)> {
        match self {
            Self::None => None,
            Self::Fact { name, set } => {
                let pair = facts.get(name);
                Some((
                    name,
                    match set {
                        NamedBaselineSet::First => pair.first(),
                        NamedBaselineSet::Last => pair.last(),
                    },
                ))
            }
        }
    }
}

pub(crate) const fn omit_indefinite_percentage_axis<S: LayoutScalar>(
    size: Size<Option<S>>,
    axis: Option<PhysicalAxis>,
) -> Size<Option<S>> {
    match axis {
        Some(PhysicalAxis::Horizontal) => Size::new(None, size.height),
        Some(PhysicalAxis::Vertical) => Size::new(size.width, None),
        None => size,
    }
}

/// Direct and recursive compute state.
///
/// Direct leaf callers construct this state with `leaf_layout` or
/// `leaf_content_size` and must supply the complete containing context explicitly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeInputOf<S: LayoutScalar = DefaultScalar> {
    run_mode: RunMode,
    sizing_mode: SizingMode,
    axis: RequestedAxis,
    known: Size<Option<S>>,
    indefinite_percentage_axis: Option<PhysicalAxis>,
    border_box_offer_axis: Option<PhysicalAxis>,
    parent: Size<Option<S>>,
    containing_layout_context: ContainingLayoutContext,
    sizing_request: SizingRequestOf<S>,
    named_baseline_demand: NamedBaselineDemand,
    physical_baseline_demand: bool,
    requires_fixed_grid_tracks: bool,
    block_indent_context: Option<crate::text_indent::BlockIndentContext<S>>,
    // Settlement owned by the node being computed.
    settled_auto_scrollbars: SettledAutoScrollbarState,
    // Settlement of the immediate containing formatting pass that produced this request.
    containing_auto_scrollbar_pass: SettledAutoScrollbarState,
}

pub type ComputeInput = ComputeInputOf<DefaultScalar>;

impl<S: LayoutScalar> ComputeInputOf<S> {
    /// Constructs direct leaf input for a full layout operation.
    ///
    /// `containing_layout_context` describes the containing boundary and is
    /// never inferred from the measured leaf.
    pub fn leaf_layout(
        known: Size<Option<S>>,
        parent: Size<Option<S>>,
        containing_layout_context: ContainingLayoutContext,
        sizing_request: SizingRequestOf<S>,
    ) -> Result<Self, LayoutSizeInputErrorOf<S>> {
        Ok(Self {
            run_mode: RunMode::PerformLayout,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known: validate_optional_size(LayoutSizeInput::Known, known)?,
            indefinite_percentage_axis: None,
            border_box_offer_axis: None,
            parent: validate_optional_size(LayoutSizeInput::PercentageParent, parent)?,
            containing_layout_context,
            sizing_request,
            named_baseline_demand: NamedBaselineDemand::None,
            physical_baseline_demand: false,
            requires_fixed_grid_tracks: false,
            block_indent_context: None,
            settled_auto_scrollbars: SettledAutoScrollbarState::INITIAL,
            containing_auto_scrollbar_pass: SettledAutoScrollbarState::INITIAL,
        })
    }

    /// Constructs direct leaf input for a content-size operation.
    ///
    /// `containing_layout_context` describes the containing boundary and is
    /// never inferred from the measured leaf.
    pub fn leaf_content_size(
        known: Size<Option<S>>,
        parent: Size<Option<S>>,
        containing_layout_context: ContainingLayoutContext,
        sizing_request: SizingRequestOf<S>,
    ) -> Result<Self, LayoutSizeInputErrorOf<S>> {
        Ok(Self {
            run_mode: RunMode::ComputeSize,
            sizing_mode: SizingMode::ContentSize,
            axis: RequestedAxis::Both,
            known: validate_optional_size(LayoutSizeInput::Known, known)?,
            indefinite_percentage_axis: None,
            border_box_offer_axis: None,
            parent: validate_optional_size(LayoutSizeInput::PercentageParent, parent)?,
            containing_layout_context,
            sizing_request,
            named_baseline_demand: NamedBaselineDemand::None,
            physical_baseline_demand: false,
            requires_fixed_grid_tracks: false,
            block_indent_context: None,
            settled_auto_scrollbars: SettledAutoScrollbarState::INITIAL,
            containing_auto_scrollbar_pass: SettledAutoScrollbarState::INITIAL,
        })
    }

    /// Returns the complete explicitly supplied containing context.
    #[must_use]
    pub const fn containing_layout_context(&self) -> ContainingLayoutContext {
        self.containing_layout_context
    }

    /// Returns the explicitly supplied parent formatting role.
    #[must_use]
    pub const fn parent_formatting_context(&self) -> ParentFormattingContext {
        self.containing_layout_context.formatting_context()
    }

    /// Returns the explicitly supplied containing flow mapping projection.
    #[must_use]
    pub const fn containing_flow_axes(&self) -> FlowAxes {
        self.containing_layout_context.flow_axes()
    }

    #[must_use]
    pub(crate) const fn root_layout(
        known: Size<Option<S>>,
        parent: Size<Option<S>>,
        containing_layout_context: ContainingLayoutContext,
        sizing_request: SizingRequestOf<S>,
    ) -> Self {
        Self::for_child(
            RunMode::PerformRootLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            known,
            parent,
            containing_layout_context,
            sizing_request,
        )
    }

    #[must_use]
    pub(crate) const fn for_child(
        run_mode: RunMode,
        sizing_mode: SizingMode,
        axis: RequestedAxis,
        known: Size<Option<S>>,
        parent: Size<Option<S>>,
        containing_layout_context: ContainingLayoutContext,
        sizing_request: SizingRequestOf<S>,
    ) -> Self {
        Self {
            run_mode,
            sizing_mode,
            axis,
            known,
            indefinite_percentage_axis: None,
            border_box_offer_axis: None,
            parent,
            containing_layout_context,
            sizing_request,
            named_baseline_demand: NamedBaselineDemand::None,
            physical_baseline_demand: false,
            requires_fixed_grid_tracks: false,
            block_indent_context: None,
            settled_auto_scrollbars: SettledAutoScrollbarState::INITIAL,
            containing_auto_scrollbar_pass: SettledAutoScrollbarState::INITIAL,
        }
    }

    #[must_use]
    pub(crate) const fn hidden(containing_layout_context: ContainingLayoutContext) -> Self {
        Self::for_child(
            RunMode::PerformHiddenLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::NONE,
            containing_layout_context,
            SizingRequestOf::new(
                Size::splat(AxisSizingRequestOf::new(
                    AvailableSpaceOf::Indefinite,
                    SizingOperation::Normal,
                )),
                crate::OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Unknown,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        )
    }

    #[must_use]
    pub(crate) const fn hidden_in_containing_pass(
        containing_layout_context: ContainingLayoutContext,
        containing_auto_scrollbar_pass: SettledAutoScrollbarState,
    ) -> Self {
        Self::hidden(containing_layout_context)
            .with_containing_auto_scrollbar_pass(containing_auto_scrollbar_pass)
    }

    pub(crate) const fn block_indent_context(
        &self,
    ) -> Option<crate::text_indent::BlockIndentContext<S>> {
        self.block_indent_context
    }

    pub(crate) const fn named_baseline_demand(&self) -> NamedBaselineDemand {
        self.named_baseline_demand
    }

    pub(crate) const fn requires_fixed_grid_tracks(&self) -> bool {
        self.requires_fixed_grid_tracks
    }

    pub(crate) const fn with_fixed_grid_track_requirement(mut self, required: bool) -> Self {
        self.requires_fixed_grid_tracks = required;
        self
    }

    pub(crate) const fn with_named_baseline_demand(mut self, demand: NamedBaselineDemand) -> Self {
        self.named_baseline_demand = demand;
        self
    }

    pub(crate) const fn physical_baseline_demand(&self) -> bool {
        self.physical_baseline_demand
    }

    pub(crate) const fn with_physical_baseline_demand(mut self, demand: bool) -> Self {
        self.physical_baseline_demand = demand;
        self
    }

    pub(crate) const fn requires_content_baseline(&self) -> bool {
        self.physical_baseline_demand
            || !matches!(self.named_baseline_demand, NamedBaselineDemand::None)
    }

    pub(crate) const fn with_baseline_demand_from(mut self, source: Self) -> Self {
        self.named_baseline_demand = source.named_baseline_demand;
        self.physical_baseline_demand = source.physical_baseline_demand;
        self
    }

    pub(crate) const fn with_block_indent_context(
        mut self,
        context: crate::text_indent::BlockIndentContext<S>,
    ) -> Self {
        self.block_indent_context = Some(context);
        self
    }

    #[must_use]
    pub(crate) const fn run_mode(&self) -> RunMode {
        self.run_mode
    }

    #[must_use]
    pub(crate) const fn with_run_mode(mut self, run_mode: RunMode) -> Self {
        self.run_mode = run_mode;
        self
    }

    #[must_use]
    pub(crate) const fn sizing_mode(&self) -> SizingMode {
        self.sizing_mode
    }

    #[must_use]
    pub(crate) const fn requested_axis(&self) -> RequestedAxis {
        self.axis
    }

    #[must_use]
    pub(crate) const fn known(&self) -> Size<Option<S>> {
        self.known
    }

    pub(crate) fn with_fragment_allocation(
        mut self,
        known: Size<Option<S>>,
        block_axis: crate::PhysicalAxis,
    ) -> Self {
        self.known.width = known.width.or(self.known.width);
        self.known.height = known.height.or(self.known.height);
        self.indefinite_percentage_axis = Some(block_axis);
        self
    }

    #[must_use]
    pub(crate) const fn with_indefinite_percentage_axis(
        mut self,
        axis: Option<PhysicalAxis>,
    ) -> Self {
        self.indefinite_percentage_axis = axis;
        self
    }

    #[must_use]
    pub(crate) const fn indefinite_percentage_axis(&self) -> Option<PhysicalAxis> {
        self.indefinite_percentage_axis
    }

    /// Marks an available axis whose offer already excludes the node's margins.
    #[must_use]
    pub(crate) const fn with_border_box_offer_axis(mut self, axis: Option<PhysicalAxis>) -> Self {
        self.border_box_offer_axis = axis;
        self
    }

    #[must_use]
    pub(crate) const fn border_box_offer_axis(&self) -> Option<PhysicalAxis> {
        self.border_box_offer_axis
    }

    /// Applies the containing algorithm's percentage restriction while retaining
    /// the allocated geometry in `known`.
    #[must_use]
    pub(crate) const fn descendant_percentage_basis(
        &self,
        size: Size<Option<S>>,
    ) -> Size<Option<S>> {
        omit_indefinite_percentage_axis(size, self.indefinite_percentage_axis)
    }

    #[must_use]
    pub(crate) const fn parent(&self) -> Size<Option<S>> {
        self.parent
    }

    #[must_use]
    pub(crate) const fn sizing_request(&self) -> SizingRequestOf<S> {
        self.sizing_request
    }

    #[must_use]
    pub(crate) const fn settled_auto_scrollbars(&self) -> SettledAutoScrollbarState {
        self.settled_auto_scrollbars
    }

    #[must_use]
    pub(crate) const fn with_settled_auto_scrollbars(
        mut self,
        settled_auto_scrollbars: SettledAutoScrollbarState,
    ) -> Self {
        self.settled_auto_scrollbars = settled_auto_scrollbars;
        self
    }

    #[must_use]
    pub(crate) const fn containing_auto_scrollbar_pass(&self) -> SettledAutoScrollbarState {
        self.containing_auto_scrollbar_pass
    }

    #[must_use]
    pub(crate) const fn with_containing_auto_scrollbar_pass(
        mut self,
        containing_auto_scrollbar_pass: SettledAutoScrollbarState,
    ) -> Self {
        self.containing_auto_scrollbar_pass = containing_auto_scrollbar_pass;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LayoutRoundingMode {
    #[default]
    NearestCssPixel,
}

/// The independent scalar-sized input rejected by a direct compute constructor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutSizeInput {
    Known,
    PercentageParent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutSizeInputErrorOf<S: LayoutScalar = DefaultScalar> {
    input: LayoutSizeInput,
    axis: PhysicalAxis,
    scalar: NonNegativeFiniteScalarErrorOf<S>,
}

pub type LayoutSizeInputError = LayoutSizeInputErrorOf<DefaultScalar>;

impl<S: LayoutScalar> LayoutSizeInputErrorOf<S> {
    #[must_use]
    pub const fn input(&self) -> LayoutSizeInput {
        self.input
    }
    #[must_use]
    pub const fn axis(&self) -> PhysicalAxis {
        self.axis
    }
    #[must_use]
    pub const fn scalar(&self) -> NonNegativeFiniteScalarErrorOf<S> {
        self.scalar
    }
}

/// Actual flex-parent authority, independent from the current sizing operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexItemRootContextOf<S: LayoutScalar = DefaultScalar> {
    parent_flow_axes: FlowAxes,
    percentage_parent: Size<Option<NonNegativeFiniteOf<S>>>,
    allocated_size: Size<Option<NonNegativeFiniteOf<S>>>,
}

pub type FlexItemRootContext = FlexItemRootContextOf<DefaultScalar>;

impl<S: LayoutScalar> FlexItemRootContextOf<S> {
    #[must_use]
    pub const fn new(
        parent_flow_axes: FlowAxes,
        percentage_parent: Size<Option<NonNegativeFiniteOf<S>>>,
        allocated_size: Size<Option<NonNegativeFiniteOf<S>>>,
    ) -> Self {
        Self {
            parent_flow_axes,
            percentage_parent,
            allocated_size,
        }
    }
    #[must_use]
    pub const fn parent_flow_axes(self) -> FlowAxes {
        self.parent_flow_axes
    }
    #[must_use]
    pub const fn percentage_parent(self) -> Size<Option<NonNegativeFiniteOf<S>>> {
        self.percentage_parent
    }
    #[must_use]
    pub const fn allocated_size(self) -> Size<Option<NonNegativeFiniteOf<S>>> {
        self.allocated_size
    }
}

/// First zero-edge in-flow child of a horizontal block parent with Auto block size.
///
/// The producer certifies the initial normal-flow origin, absence of preceding
/// flow/float displacement, and the stated parent geometry. HTML/body proof is
/// the capture adapter's responsibility, not part of this host-agnostic value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockFlowRootContextOf<S: LayoutScalar = DefaultScalar> {
    parent_inline_extent: NonNegativeFiniteOf<S>,
    parent_direction: crate::Direction,
    orthogonal_fallback: crate::OrthogonalFallbackOf<S>,
}

pub type BlockFlowRootContext = BlockFlowRootContextOf<DefaultScalar>;

impl<S: LayoutScalar> BlockFlowRootContextOf<S> {
    #[must_use]
    pub const fn horizontal_auto_block(
        parent_inline_extent: NonNegativeFiniteOf<S>,
        parent_direction: crate::Direction,
        orthogonal_fallback: crate::OrthogonalFallbackOf<S>,
    ) -> Self {
        Self {
            parent_inline_extent,
            parent_direction,
            orthogonal_fallback,
        }
    }
    #[must_use]
    pub const fn parent_flow_axes(self) -> FlowAxes {
        FlowAxes::new(crate::WritingMode::HorizontalTb, self.parent_direction)
    }
    #[must_use]
    pub const fn parent_inline_extent(self) -> NonNegativeFiniteOf<S> {
        self.parent_inline_extent
    }
    #[must_use]
    pub const fn orthogonal_fallback(self) -> crate::OrthogonalFallbackOf<S> {
        self.orthogonal_fallback
    }
}

/// Zero-edge, non-replaced horizontal absolute root with Auto insets, at the
/// initial containing/static-position frame's initial block/inline start.
///
/// Containing extents include applicable scrollbar reservation. The producer
/// certifies the real unshifted frame and shared containing/static-position
/// direction; no used/expected root rectangle supplies these operands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionedRootContextOf<S: LayoutScalar = DefaultScalar> {
    containing_size: Size<NonNegativeFiniteOf<S>>,
    containing_direction: crate::Direction,
    orthogonal_fallback: crate::OrthogonalFallbackOf<S>,
}

pub type PositionedRootContext = PositionedRootContextOf<DefaultScalar>;

impl<S: LayoutScalar> PositionedRootContextOf<S> {
    #[must_use]
    pub const fn horizontal_auto_insets(
        containing_size: Size<NonNegativeFiniteOf<S>>,
        containing_direction: crate::Direction,
        orthogonal_fallback: crate::OrthogonalFallbackOf<S>,
    ) -> Self {
        Self {
            containing_size,
            containing_direction,
            orthogonal_fallback,
        }
    }
    #[must_use]
    pub const fn containing_size(self) -> Size<NonNegativeFiniteOf<S>> {
        self.containing_size
    }
    #[must_use]
    pub const fn containing_flow_axes(self) -> FlowAxes {
        FlowAxes::new(crate::WritingMode::HorizontalTb, self.containing_direction)
    }
    #[must_use]
    pub const fn orthogonal_fallback(self) -> crate::OrthogonalFallbackOf<S> {
        self.orthogonal_fallback
    }
}

/// An absolute root observed in its own border-box frame.
///
/// The producer certifies the authentic horizontal containing block and a
/// signed offset along the independently authenticated hypothetical inline
/// progression, mapped over the same full physical containing width. This progression need not have
/// the containing block's direction. The producer retains the hypothetical
/// direction, anchor, and mapping proof; the projected scalar is sufficient for
/// local sizing. The containing direction still supplies the actual containing
/// flow axes, not a global static-anchor alignment.
///
/// Root-local coordinates omit document-space translation and establish the
/// continuous border-box frame before publication rounding. The producer
/// supplies the hypothetical frame and proves that the subtree's observations
/// can use local coordinates.
///
/// All root insets are Auto and margins are zero. A horizontal root can carry
/// padding and border when its inline preferred size resolves definite against
/// the full containing width. Auto and intrinsic inline sizing require zero
/// padding and border. Orthogonal roots require zero padding and border.
/// Their preferred sizes must resolve definite on both physical axes, except
/// for two cases with an explicit hypothetical block-start offset. A
/// vertical-lr or vertical-rl principal Block can fit its Auto inline size
/// with a definite opposite preferred size. A Grid with two Auto preferred
/// sizes, fixed templates on both axes, and no leaf measurement uses normal
/// Grid sizing when every track after placement is fixed.
///
/// The offset is the hypothetical position relative to the containing block's
/// physical top. A supplied offset requires both preferred sizes to resolve
/// definite unless one of those Auto sizing cases applies. Its nonnegative
/// remaining height supplies physical availability. Each percentage uses the
/// full containing-block extent on its physical axis.
/// Normal sizing applies authored box sizing and min/max constraints.
/// Without a supplied block-start offset the request's physical height
/// availability is indefinite.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RootLocalPositionedContextOf<S: LayoutScalar = DefaultScalar> {
    containing_size: Size<NonNegativeFiniteOf<S>>,
    containing_direction: crate::Direction,
    hypothetical_inline_offset: S,
    hypothetical_block_offset: Option<S>,
    orthogonal_fallback: crate::OrthogonalFallbackOf<S>,
}

pub type RootLocalPositionedContext = RootLocalPositionedContextOf<DefaultScalar>;

impl<S: LayoutScalar> RootLocalPositionedContextOf<S> {
    pub fn horizontal_auto_insets(
        containing_size: Size<NonNegativeFiniteOf<S>>,
        containing_direction: crate::Direction,
        hypothetical_inline_offset: S,
        orthogonal_fallback: crate::OrthogonalFallbackOf<S>,
    ) -> Result<Self, crate::FiniteScalarErrorOf<S>> {
        if !hypothetical_inline_offset.is_finite() {
            return Err(crate::FiniteScalarErrorOf::NonFinite {
                value: hypothetical_inline_offset,
            });
        }
        Ok(Self {
            containing_size,
            containing_direction,
            hypothetical_inline_offset: crate::scalar::canonical_zero(hypothetical_inline_offset),
            hypothetical_block_offset: None,
            orthogonal_fallback,
        })
    }
    /// Supplies the hypothetical block-start offset in the horizontal
    /// containing frame. It is relative to the absolute containing block's
    /// physical top edge and independent of the published root-local origin.
    pub fn with_hypothetical_block_offset(
        mut self,
        offset: S,
    ) -> Result<Self, crate::FiniteScalarErrorOf<S>> {
        if !offset.is_finite() {
            return Err(crate::FiniteScalarErrorOf::NonFinite { value: offset });
        }
        self.hypothetical_block_offset = Some(crate::scalar::canonical_zero(offset));
        Ok(self)
    }
    #[must_use]
    pub const fn containing_size(self) -> Size<NonNegativeFiniteOf<S>> {
        self.containing_size
    }
    #[must_use]
    pub const fn containing_flow_axes(self) -> FlowAxes {
        FlowAxes::new(crate::WritingMode::HorizontalTb, self.containing_direction)
    }
    #[must_use]
    pub const fn hypothetical_inline_offset(self) -> S {
        self.hypothetical_inline_offset
    }
    /// The supplied physical top offset, when available.
    #[must_use]
    pub const fn hypothetical_block_offset(self) -> Option<S> {
        self.hypothetical_block_offset
    }
    #[must_use]
    pub const fn orthogonal_fallback(self) -> crate::OrthogonalFallbackOf<S> {
        self.orthogonal_fallback
    }
}

/// A declared root context disagrees with the actual root input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootContextMismatch {
    Position,
    WritingMode,
    Display,
    BoxEdges,
    Insets,
    AspectRatio,
    Replaced,
    Float,
    Clearance,
    Alignment,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutRootContextOf<S: LayoutScalar = DefaultScalar> {
    Standalone {
        containing_flow_axes: FlowAxes,
        percentage_parent: Size<Option<NonNegativeFiniteOf<S>>>,
    },
    FlexItem(FlexItemRootContextOf<S>),
    BlockFlow(BlockFlowRootContextOf<S>),
    PositionedAutoInsets(PositionedRootContextOf<S>),
    RootLocalPositioned(RootLocalPositionedContextOf<S>),
}

pub type LayoutRootContext = LayoutRootContextOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutRootRequestOf<S: LayoutScalar = DefaultScalar> {
    sizing_request: SizingRequestOf<S>,
    context: LayoutRootContextOf<S>,
    rounding_mode: LayoutRoundingMode,
}

pub type LayoutRootRequest = LayoutRootRequestOf<DefaultScalar>;

impl<S: LayoutScalar> LayoutRootRequestOf<S> {
    #[must_use]
    pub const fn standalone(
        sizing_request: SizingRequestOf<S>,
        containing_flow_axes: FlowAxes,
        percentage_parent: Size<Option<NonNegativeFiniteOf<S>>>,
    ) -> Self {
        Self::new(
            sizing_request,
            LayoutRootContextOf::Standalone {
                containing_flow_axes,
                percentage_parent,
            },
        )
    }

    #[must_use]
    pub const fn flex_item(
        sizing_request: SizingRequestOf<S>,
        context: FlexItemRootContextOf<S>,
    ) -> Self {
        Self::new(sizing_request, LayoutRootContextOf::FlexItem(context))
    }

    #[must_use]
    pub const fn block_flow(
        operations: Size<SizingOperation>,
        context: BlockFlowRootContextOf<S>,
    ) -> Self {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(context.parent_inline_extent),
                operations.width,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, operations.height),
        );
        Self::new(
            SizingRequestOf::new(axes, context.orthogonal_fallback),
            LayoutRootContextOf::BlockFlow(context),
        )
    }

    #[must_use]
    pub const fn positioned_auto_insets(context: PositionedRootContextOf<S>) -> Self {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(context.containing_size.width),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(context.containing_size.height),
                SizingOperation::Normal,
            ),
        );
        Self::new(
            SizingRequestOf::new(axes, context.orthogonal_fallback),
            LayoutRootContextOf::PositionedAutoInsets(context),
        )
    }

    /// Constructs the actual root-local Normal request. Checked subtraction is
    /// performed once here; a negative remaining span is exhausted space, while
    /// unrepresentable arithmetic refuses before a request is issued.
    pub fn positioned_root_local(
        context: RootLocalPositionedContextOf<S>,
    ) -> Result<Self, NonNegativeFiniteScalarErrorOf<S>> {
        let remaining = |extent: NonNegativeFiniteOf<S>, offset: S| match NonNegativeFiniteOf::new(
            extent.get() - offset,
        ) {
            Ok(value) => Ok(AvailableSpaceOf::Definite(value)),
            Err(NonNegativeFiniteScalarErrorOf::Negative { .. }) => {
                Ok(AvailableSpaceOf::Definite(NonNegativeFiniteOf::ZERO))
            }
            Err(error @ NonNegativeFiniteScalarErrorOf::NonFinite { .. }) => Err(error),
        };
        let width = remaining(
            context.containing_size.width,
            context.hypothetical_inline_offset,
        )?;
        let height = context
            .hypothetical_block_offset
            .map(|offset| remaining(context.containing_size.height, offset))
            .transpose()?
            .unwrap_or(AvailableSpaceOf::Indefinite);
        Ok(Self::new(
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(width, SizingOperation::Normal),
                    AxisSizingRequestOf::new(height, SizingOperation::Normal),
                ),
                context.orthogonal_fallback,
            ),
            LayoutRootContextOf::RootLocalPositioned(context),
        ))
    }

    #[must_use]
    pub const fn with_rounding_mode(mut self, rounding_mode: LayoutRoundingMode) -> Self {
        self.rounding_mode = rounding_mode;
        self
    }

    const fn new(sizing_request: SizingRequestOf<S>, context: LayoutRootContextOf<S>) -> Self {
        Self {
            sizing_request,
            context,
            rounding_mode: LayoutRoundingMode::NearestCssPixel,
        }
    }

    #[must_use]
    pub const fn sizing_request(&self) -> SizingRequestOf<S> {
        self.sizing_request
    }
    #[must_use]
    pub const fn context(&self) -> LayoutRootContextOf<S> {
        self.context
    }
    #[must_use]
    pub const fn rounding_mode(&self) -> LayoutRoundingMode {
        self.rounding_mode
    }
}

fn validate_optional_size<S: LayoutScalar>(
    input: LayoutSizeInput,
    size: Size<Option<S>>,
) -> Result<Size<Option<S>>, LayoutSizeInputErrorOf<S>> {
    let axis = |axis, value: Option<S>| {
        value
            .map(|value| {
                NonNegativeFiniteOf::new(value)
                    .map(NonNegativeFiniteOf::get)
                    .map_err(|scalar| LayoutSizeInputErrorOf {
                        input,
                        axis,
                        scalar,
                    })
            })
            .transpose()
    };
    Ok(Size::new(
        axis(PhysicalAxis::Horizontal, size.width)?,
        axis(PhysicalAxis::Vertical, size.height)?,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollapsibleMarginOf<S: LayoutScalar = DefaultScalar> {
    positive: S,
    negative: S,
}

pub type CollapsibleMargin = CollapsibleMarginOf<DefaultScalar>;

impl<S: LayoutScalar> CollapsibleMarginOf<S> {
    pub const ZERO: Self = Self {
        positive: S::ZERO,
        negative: S::ZERO,
    };

    #[must_use]
    pub fn from_margin(margin: S) -> Self {
        if margin >= S::ZERO {
            Self {
                positive: margin,
                negative: S::ZERO,
            }
        } else {
            Self {
                positive: S::ZERO,
                negative: margin,
            }
        }
    }

    #[must_use]
    pub fn collapse_with_margin(self, margin: S) -> Self {
        self.collapse_with(Self::from_margin(margin))
    }

    #[must_use]
    pub fn collapse_with(self, other: Self) -> Self {
        Self {
            positive: self.positive.max(other.positive),
            negative: self.negative.min(other.negative),
        }
    }

    #[must_use]
    pub fn resolve(self) -> S {
        self.positive + self.negative
    }
}

/// Collapsible block margins reported on their physical output sides.
///
/// The carrier records the physical axis selected by the reporting box's
/// `FlowAxes`. Callers select physical sides directly and provide their own
/// containing flow when asking whether collapse may continue through the box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicalBlockMarginCollapseOf<S: LayoutScalar = DefaultScalar> {
    margins: Edges<CollapsibleMarginOf<S>>,
    block_axis: PhysicalAxis,
    can_collapse_through: bool,
}

/// Default-scalar physical block-margin-collapse output.
pub type PhysicalBlockMarginCollapse = PhysicalBlockMarginCollapseOf<DefaultScalar>;

impl<S: LayoutScalar> PhysicalBlockMarginCollapseOf<S> {
    /// A non-reporting collapse result with zero margins and no eligibility.
    pub const NONE: Self = Self {
        margins: Edges::all(CollapsibleMarginOf::ZERO),
        block_axis: PhysicalAxis::Vertical,
        can_collapse_through: false,
    };

    /// Constructs physical block-start and block-end margin state for `flow_axes`.
    #[must_use]
    pub const fn from_block_flow(
        flow_axes: FlowAxes,
        block_start: CollapsibleMarginOf<S>,
        block_end: CollapsibleMarginOf<S>,
        can_collapse_through: bool,
    ) -> Self {
        let zero = CollapsibleMarginOf::ZERO;
        let margins = match flow_axes.block_start() {
            PhysicalSide::Top => Edges::new(block_start, zero, block_end, zero),
            PhysicalSide::Right => Edges::new(zero, block_start, zero, block_end),
            PhysicalSide::Bottom => Edges::new(block_end, zero, block_start, zero),
            PhysicalSide::Left => Edges::new(zero, block_end, zero, block_start),
        };

        Self {
            margins,
            block_axis: flow_axes.block_axis(),
            can_collapse_through,
        }
    }

    /// Returns the margin set stored on `side`.
    #[must_use]
    pub const fn at(self, side: PhysicalSide) -> CollapsibleMarginOf<S> {
        match side {
            PhysicalSide::Top => self.margins.top,
            PhysicalSide::Right => self.margins.right,
            PhysicalSide::Bottom => self.margins.bottom,
            PhysicalSide::Left => self.margins.left,
        }
    }

    /// Returns whether a containing flow on the same physical block axis may collapse through.
    #[must_use]
    pub const fn can_collapse_through(self, containing_flow: FlowAxes) -> bool {
        self.can_collapse_through
            && matches!(
                (containing_flow.block_axis(), self.block_axis),
                (PhysicalAxis::Horizontal, PhysicalAxis::Horizontal)
                    | (PhysicalAxis::Vertical, PhysicalAxis::Vertical)
            )
    }
}

/// A finite named baseline coordinate relative to the physical border-box origin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineCoordinateOf<S: LayoutScalar = DefaultScalar> {
    axis: PhysicalAxis,
    offset: S,
}

impl<S: LayoutScalar> BaselineCoordinateOf<S> {
    pub fn try_new(axis: PhysicalAxis, offset: S) -> Result<Self, crate::FiniteScalarErrorOf<S>> {
        if !offset.is_finite() {
            return Err(crate::FiniteScalarErrorOf::NonFinite { value: offset });
        }
        Ok(Self {
            axis,
            offset: if offset == S::ZERO { S::ZERO } else { offset },
        })
    }
    #[must_use]
    pub const fn axis(self) -> PhysicalAxis {
        self.axis
    }
    #[must_use]
    pub const fn offset(self) -> S {
        self.offset
    }
    /// Named scroll exposure clamps only actual coordinates, in their own
    /// border frame. Ordinary absent facts remain absent until demanded.
    pub(crate) fn clamped_to_border(
        self,
        size: Size<S>,
    ) -> Result<Self, crate::FiniteScalarErrorOf<S>> {
        let extent = match self.axis {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        };
        if !extent.is_finite() {
            return Err(crate::FiniteScalarErrorOf::NonFinite { value: extent });
        }
        Self::try_new(self.axis, self.offset.max(S::ZERO).min(extent))
    }
}

/// Unknown authority differs from the affirmative absence permitting synthesis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BaselineFactOf<S: LayoutScalar = DefaultScalar> {
    Unknown,
    Absent,
    Available(BaselineCoordinateOf<S>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselinePairOf<S: LayoutScalar = DefaultScalar> {
    first: BaselineFactOf<S>,
    last: BaselineFactOf<S>,
}
impl<S: LayoutScalar> BaselinePairOf<S> {
    #[must_use]
    pub const fn new(first: BaselineFactOf<S>, last: BaselineFactOf<S>) -> Self {
        Self { first, last }
    }
    #[must_use]
    pub const fn first(self) -> BaselineFactOf<S> {
        self.first
    }
    #[must_use]
    pub const fn last(self) -> BaselineFactOf<S> {
        self.last
    }
}

/// Named facts independent of the generic Flex/Grid physical projections.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NamedBaselinesOf<S: LayoutScalar = DefaultScalar> {
    alphabetic: BaselinePairOf<S>,
    central: BaselinePairOf<S>,
}
impl<S: LayoutScalar> NamedBaselinesOf<S> {
    #[must_use]
    pub const fn unknown() -> Self {
        let pair = BaselinePairOf::new(BaselineFactOf::Unknown, BaselineFactOf::Unknown);
        Self {
            alphabetic: pair,
            central: pair,
        }
    }
    #[must_use]
    pub const fn absent() -> Self {
        let pair = BaselinePairOf::new(BaselineFactOf::Absent, BaselineFactOf::Absent);
        Self {
            alphabetic: pair,
            central: pair,
        }
    }
    #[must_use]
    pub const fn with(mut self, name: crate::BaselineName, pair: BaselinePairOf<S>) -> Self {
        match name {
            crate::BaselineName::Alphabetic => self.alphabetic = pair,
            crate::BaselineName::Central => self.central = pair,
        }
        self
    }
    #[must_use]
    pub const fn get(self, name: crate::BaselineName) -> BaselinePairOf<S> {
        match name {
            crate::BaselineName::Alphabetic => self.alphabetic,
            crate::BaselineName::Central => self.central,
        }
    }
    pub(crate) fn clamped_to_border(
        self,
        size: Size<S>,
    ) -> Result<Self, crate::FiniteScalarErrorOf<S>> {
        let clamp = |fact: BaselineFactOf<S>| match fact {
            BaselineFactOf::Available(coordinate) => coordinate
                .clamped_to_border(size)
                .map(BaselineFactOf::Available),
            other => Ok(other),
        };
        let mut result = self;
        for name in [
            crate::BaselineName::Alphabetic,
            crate::BaselineName::Central,
        ] {
            let pair = self.get(name);
            result = result.with(
                name,
                BaselinePairOf::new(clamp(pair.first())?, clamp(pair.last())?),
            );
        }
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselinesOf<S: LayoutScalar = DefaultScalar> {
    pub first: Point<Option<S>>,
    pub last: Point<Option<S>>,
}

pub type Baselines = BaselinesOf<DefaultScalar>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PhysicalBaseline<S: LayoutScalar> {
    point: Point<Option<S>>,
    axis: PhysicalAxis,
}

impl<S: LayoutScalar> PhysicalBaseline<S> {
    pub(crate) fn new(axis: PhysicalAxis, coordinate: S) -> Self {
        let point = match axis {
            PhysicalAxis::Horizontal => Point::new(Some(coordinate), None),
            PhysicalAxis::Vertical => Point::new(None, Some(coordinate)),
        };
        Self { point, axis }
    }

    #[must_use]
    pub(crate) fn axis(self) -> PhysicalAxis {
        self.axis
    }

    #[must_use]
    pub(crate) fn coordinate(self) -> S {
        match (self.axis, self.point) {
            (PhysicalAxis::Horizontal, Point { x: Some(value), .. })
            | (PhysicalAxis::Vertical, Point { y: Some(value), .. }) => value,
            _ => unreachable!("physical baseline construction stores its tagged coordinate"),
        }
    }

    #[must_use]
    pub(crate) fn coordinate_on(self, expected_axis: PhysicalAxis) -> Option<S> {
        (self.axis == expected_axis).then_some(self.coordinate())
    }

    #[must_use]
    pub(crate) fn translated(self, location: Point<S>) -> Point<Option<S>> {
        match self.axis {
            PhysicalAxis::Horizontal => Point::new(Some(location.x + self.coordinate()), None),
            PhysicalAxis::Vertical => Point::new(None, Some(location.y + self.coordinate())),
        }
    }
}

impl<S: LayoutScalar> BaselinesOf<S> {
    pub const NONE: Self = Self {
        first: Point::NONE,
        last: Point::NONE,
    };

    #[must_use]
    pub const fn first(first: Point<Option<S>>) -> Self {
        Self {
            first,
            last: Point::NONE,
        }
    }

    #[must_use]
    pub fn synthesized(flow_axes: FlowAxes, size: Size<S>) -> Self {
        Self {
            first: Self::block_point(
                flow_axes,
                Some(Self::side_coordinate(flow_axes.line_under(), size)),
            ),
            last: Self::block_point(
                flow_axes,
                Some(Self::side_coordinate(flow_axes.line_under(), size)),
            ),
        }
    }

    #[must_use]
    pub fn first_or_synthesize_block(self, flow_axes: FlowAxes, size: Size<S>) -> S {
        self.first_or_synthesize_block_baseline(flow_axes, size)
            .coordinate()
    }

    #[must_use]
    pub fn last_or_synthesize_block(self, flow_axes: FlowAxes, size: Size<S>) -> S {
        self.last_or_synthesize_block_baseline(flow_axes, size)
            .coordinate()
    }

    #[must_use]
    pub fn first_block(self, flow_axes: FlowAxes) -> Option<S> {
        self.first_block_baseline(flow_axes)
            .map(PhysicalBaseline::coordinate)
    }

    #[must_use]
    pub fn last_block(self, flow_axes: FlowAxes) -> Option<S> {
        self.last_block_baseline(flow_axes)
            .map(PhysicalBaseline::coordinate)
    }

    #[must_use]
    pub(crate) fn first_block_baseline(self, flow_axes: FlowAxes) -> Option<PhysicalBaseline<S>> {
        Self::block_coordinate(flow_axes, self.first)
            .map(|coordinate| PhysicalBaseline::new(flow_axes.block_axis(), coordinate))
    }

    #[must_use]
    pub(crate) fn last_block_baseline(self, flow_axes: FlowAxes) -> Option<PhysicalBaseline<S>> {
        Self::block_coordinate(flow_axes, self.last)
            .map(|coordinate| PhysicalBaseline::new(flow_axes.block_axis(), coordinate))
    }

    #[must_use]
    pub(crate) fn first_or_synthesize_block_baseline(
        self,
        flow_axes: FlowAxes,
        size: Size<S>,
    ) -> PhysicalBaseline<S> {
        self.first_block_baseline(flow_axes).unwrap_or_else(|| {
            PhysicalBaseline::new(
                flow_axes.block_axis(),
                Self::side_coordinate(flow_axes.line_under(), size),
            )
        })
    }

    #[must_use]
    pub(crate) fn last_or_synthesize_block_baseline(
        self,
        flow_axes: FlowAxes,
        size: Size<S>,
    ) -> PhysicalBaseline<S> {
        self.last_block_baseline(flow_axes).unwrap_or_else(|| {
            PhysicalBaseline::new(
                flow_axes.block_axis(),
                Self::side_coordinate(flow_axes.line_under(), size),
            )
        })
    }

    #[must_use]
    pub(crate) fn from_block_coordinates(
        flow_axes: FlowAxes,
        first: Option<S>,
        last: Option<S>,
    ) -> Self {
        Self {
            first: Self::block_point(flow_axes, first),
            last: Self::block_point(flow_axes, last),
        }
    }

    pub(crate) fn record_first(&mut self, point: Point<Option<S>>) {
        if self.first.x.is_none() {
            self.first.x = point.x;
        }
        if self.first.y.is_none() {
            self.first.y = point.y;
        }
    }

    pub(crate) fn record_last(&mut self, point: Point<Option<S>>) {
        if point.x.is_some() {
            self.last.x = point.x;
        }
        if point.y.is_some() {
            self.last.y = point.y;
        }
    }

    pub(crate) fn replace_first_axis(&mut self, point: Point<Option<S>>) {
        if point.x.is_some() {
            self.first.x = point.x;
        }
        if point.y.is_some() {
            self.first.y = point.y;
        }
    }

    pub(crate) fn replace_last_axis(&mut self, point: Point<Option<S>>) {
        if point.x.is_some() {
            self.last.x = point.x;
        }
        if point.y.is_some() {
            self.last.y = point.y;
        }
    }

    fn block_coordinate(flow_axes: FlowAxes, point: Point<Option<S>>) -> Option<S> {
        flow_axes.block_axis_coordinate(point)
    }

    fn block_point(flow_axes: FlowAxes, value: Option<S>) -> Point<Option<S>> {
        match flow_axes.block_axis() {
            PhysicalAxis::Horizontal => Point::new(value, None),
            PhysicalAxis::Vertical => Point::new(None, value),
        }
    }

    fn side_coordinate(side: PhysicalSide, size: Size<S>) -> S {
        match side {
            PhysicalSide::Top | PhysicalSide::Left => S::ZERO,
            PhysicalSide::Right => size.width,
            PhysicalSide::Bottom => size.height,
        }
    }
}

#[cfg(test)]
mod baseline_tests {
    use super::*;
    use crate::geometry::{PhysicalAxis, PhysicalSide};
    use crate::{Direction, WritingMode};

    fn point_coordinate<S: LayoutScalar>(point: Point<Option<S>>, axis: PhysicalAxis) -> Option<S> {
        match axis {
            PhysicalAxis::Horizontal => point.x,
            PhysicalAxis::Vertical => point.y,
        }
    }

    fn side_coordinate<S: LayoutScalar>(side: PhysicalSide, size: Size<S>) -> S {
        match side {
            PhysicalSide::Top | PhysicalSide::Left => S::ZERO,
            PhysicalSide::Right => size.width,
            PhysicalSide::Bottom => size.height,
        }
    }

    fn assert_baseline_selection_and_synthesis<S: LayoutScalar>() {
        let size = Size::new(S::from_f64(70.0), S::from_f64(110.0));
        let rows = [
            (WritingMode::HorizontalTb, Direction::Ltr),
            (WritingMode::HorizontalTb, Direction::Rtl),
            (WritingMode::VerticalRl, Direction::Ltr),
            (WritingMode::VerticalRl, Direction::Rtl),
            (WritingMode::VerticalLr, Direction::Ltr),
            (WritingMode::VerticalLr, Direction::Rtl),
            (WritingMode::SidewaysRl, Direction::Ltr),
            (WritingMode::SidewaysRl, Direction::Rtl),
            (WritingMode::SidewaysLr, Direction::Ltr),
            (WritingMode::SidewaysLr, Direction::Rtl),
        ];

        for (writing_mode, direction) in rows {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let block_axis = flow_axes.block_axis();
            let non_block_axis = block_axis.other();
            let baselines = match block_axis {
                PhysicalAxis::Horizontal => BaselinesOf {
                    first: Point::new(Some(S::from_f64(17.0)), None),
                    last: Point::new(Some(S::from_f64(29.0)), None),
                },
                PhysicalAxis::Vertical => BaselinesOf {
                    first: Point::new(None, Some(S::from_f64(23.0))),
                    last: Point::new(None, Some(S::from_f64(31.0))),
                },
            };
            let expected_first = point_coordinate(baselines.first, block_axis)
                .expect("the mapped physical block coordinate is present");
            let expected_last = point_coordinate(baselines.last, block_axis)
                .expect("the mapped physical block coordinate is present");

            assert_eq!(point_coordinate(baselines.first, non_block_axis), None);
            assert_eq!(point_coordinate(baselines.last, non_block_axis), None);

            assert_eq!(
                baselines.first_or_synthesize_block(flow_axes, size),
                expected_first
            );
            assert_eq!(
                baselines.last_or_synthesize_block(flow_axes, size),
                expected_last
            );

            // Align3 synthesis uses line-under for either baseline set.
            // A supplied zero remains authority, not missing data.
            let zero_last = match block_axis {
                PhysicalAxis::Horizontal => Point::new(Some(S::ZERO), None),
                PhysicalAxis::Vertical => Point::new(None, Some(S::ZERO)),
            };
            assert_eq!(
                BaselinesOf {
                    first: baselines.first,
                    last: zero_last
                }
                .last_or_synthesize_block(flow_axes, size),
                S::ZERO,
            );
            assert_eq!(
                BaselinesOf::<S>::NONE.first_or_synthesize_block(flow_axes, size),
                side_coordinate(flow_axes.line_under(), size),
            );
            assert_eq!(
                BaselinesOf::<S>::NONE.last_or_synthesize_block(flow_axes, size),
                side_coordinate(flow_axes.line_under(), size),
            );
            let synthesized = BaselinesOf::synthesized(flow_axes, size);
            assert_eq!(
                point_coordinate(synthesized.first, block_axis),
                Some(side_coordinate(flow_axes.line_under(), size))
            );
            assert_eq!(
                point_coordinate(synthesized.last, block_axis),
                Some(side_coordinate(flow_axes.line_under(), size))
            );
            assert_eq!(point_coordinate(synthesized.first, non_block_axis), None);
            assert_eq!(point_coordinate(synthesized.last, non_block_axis), None);
        }
    }

    #[test]
    fn baseline_selection_and_synthesis_follow_all_flow_axes_for_f32() {
        assert_baseline_selection_and_synthesis::<f32>();
    }

    #[test]
    fn baseline_selection_and_synthesis_follow_all_flow_axes_for_f64() {
        assert_baseline_selection_and_synthesis::<f64>();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComputeOutputOf<S: LayoutScalar = DefaultScalar> {
    /// Unsnapped facts relative to this computed border-box origin.
    pub named_baselines: NamedBaselinesOf<S>,
    pub size: Size<S>,
    pub content_size: Size<S>,
    pub scroll_geometry: Option<ScrollGeometryOf<S>>,
    pub first_baselines: Point<Option<S>>,
    pub last_baselines: Point<Option<S>>,
    pub block_margin_collapse: PhysicalBlockMarginCollapseOf<S>,
}

pub type ComputeOutput = ComputeOutputOf<DefaultScalar>;

impl<S: LayoutScalar> ComputeOutputOf<S> {
    pub const HIDDEN: Self = Self {
        named_baselines: NamedBaselinesOf::absent(),
        size: Size::<S>::ZERO,
        content_size: Size::<S>::ZERO,
        scroll_geometry: None,
        first_baselines: Point::NONE,
        last_baselines: Point::NONE,
        block_margin_collapse: PhysicalBlockMarginCollapseOf::NONE,
    };

    /// Neutral generic output does not prove that a named baseline is absent.
    pub const DEFAULT: Self = Self {
        named_baselines: NamedBaselinesOf::unknown(),
        ..Self::HIDDEN
    };

    #[must_use]
    pub const fn from_sizes_and_baselines(
        size: Size<S>,
        content_size: Size<S>,
        baselines: BaselinesOf<S>,
    ) -> Self {
        Self {
            named_baselines: NamedBaselinesOf::unknown(),
            size,
            content_size,
            scroll_geometry: None,
            first_baselines: baselines.first,
            last_baselines: baselines.last,
            block_margin_collapse: PhysicalBlockMarginCollapseOf::NONE,
        }
    }

    #[must_use]
    pub const fn from_sizes_and_first_baselines(
        size: Size<S>,
        content_size: Size<S>,
        first_baselines: Point<Option<S>>,
    ) -> Self {
        Self::from_sizes_and_baselines(size, content_size, BaselinesOf::first(first_baselines))
    }

    #[must_use]
    pub const fn from_sizes(size: Size<S>, content_size: Size<S>) -> Self {
        Self::from_sizes_and_baselines(size, content_size, BaselinesOf::NONE)
    }

    #[must_use]
    pub const fn from_outer_size(size: Size<S>) -> Self {
        Self::from_sizes(size, Size::<S>::ZERO)
    }

    #[must_use]
    pub const fn baselines(&self) -> BaselinesOf<S> {
        BaselinesOf {
            first: self.first_baselines,
            last: self.last_baselines,
        }
    }

    #[must_use]
    pub const fn with_named_baselines(mut self, facts: NamedBaselinesOf<S>) -> Self {
        self.named_baselines = facts;
        self
    }
}

/// A node's zero-based source-sibling index, independent of item order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceIndex(usize);

impl SourceIndex {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeOutputOf<S: LayoutScalar = DefaultScalar> {
    /// Actual named facts relative to this packet's border-box origin.
    /// Unrounded packets retain unsnapped facts; final packets snap the shared
    /// global named anchor once and subtract the actual published border origin.
    /// Generic physical projections never establish these names.
    pub named_baselines: NamedBaselinesOf<S>,
    /// Zero-based index among the node's source siblings.
    ///
    /// Root and standalone outputs use [`SourceIndex::ZERO`]. This identity is
    /// independent of CSS item order and output slice position.
    pub source_index: SourceIndex,
    pub location: Point<S>,
    pub size: Size<S>,
    pub content_size: Size<S>,
    /// Immutable canonical local geometry produced by a completed layout.
    ///
    /// `None` means this output carries no performed scroll geometry. A present
    /// value includes used overflow, boxes, clips, signed ranges, gutters and
    /// nested target metadata; it never contains a live current offset.
    pub scroll_geometry: Option<ScrollGeometryOf<S>>,
    pub border: Edges<S>,
    pub padding: Edges<S>,
    pub margin: Edges<S>,
}

pub type NodeOutput = NodeOutputOf<DefaultScalar>;

impl<S: LayoutScalar> NodeOutputOf<S> {
    #[must_use]
    pub const fn new() -> Self {
        Self::with_source_index(SourceIndex::ZERO)
    }

    #[must_use]
    pub const fn with_source_index(source_index: SourceIndex) -> Self {
        Self {
            named_baselines: NamedBaselinesOf::unknown(),
            source_index,
            location: Point::<S>::ZERO,
            size: Size::<S>::ZERO,
            content_size: Size::<S>::ZERO,
            scroll_geometry: None,
            border: Edges::<S>::ZERO,
            padding: Edges::<S>::ZERO,
            margin: Edges::<S>::ZERO,
        }
    }

    #[must_use]
    pub(crate) fn with_scroll_geometry(
        mut self,
        scroll_geometry: Option<ScrollGeometryOf<S>>,
    ) -> Self {
        self.scroll_geometry = scroll_geometry;
        self
    }

    /// Returns the canonical content-box size when geometry is present.
    ///
    /// The fallback derives a saturated box from ordinary output edges; callers
    /// do not need to reconstruct canonical scroll geometry.
    #[must_use]
    pub fn content_box_size(self) -> Size<S> {
        if let Some(geometry) = self.scroll_geometry {
            return geometry.content_box().size();
        }

        Size::new(
            (self.size.width
                - self.padding.left
                - self.padding.right
                - self.border.left
                - self.border.right)
                .max(S::ZERO),
            (self.size.height
                - self.padding.top
                - self.padding.bottom
                - self.border.top
                - self.border.bottom)
                .max(S::ZERO),
        )
    }

    /// Returns the reservation derived from canonical gutter geometry.
    ///
    /// This is a read-only projection, not independently mutable output state.
    #[must_use]
    pub const fn scrollbar_size(self) -> Size<S> {
        match self.scroll_geometry {
            Some(geometry) => geometry.scrollbar_size(),
            None => Size::ZERO,
        }
    }
}

impl<S: LayoutScalar> Default for NodeOutputOf<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable geometry associated with one actual Text-issued fragment.
///
/// The retained handle identifies the exact finalized line and origin, control,
/// virtual separator or alternative occurrence, including empty occurrences. The association table
/// does not supply a mixed visual-order ordinal. No glyph packet or caller-local
/// segment ID is copied into this Layout output.
#[derive(Clone, Debug, PartialEq)]
pub struct InlineFragmentOutputOf<S: LayoutScalar = DefaultScalar> {
    fragment: surgeist_text::FinalizedFragment,
    rect: Option<ScrollRectOf<S>>,
    baseline: Point<S>,
    line_index: usize,
}

pub type InlineFragmentOutput = InlineFragmentOutputOf<DefaultScalar>;

impl<S: LayoutScalar> InlineFragmentOutputOf<S> {
    pub(crate) fn new(
        fragment: surgeist_text::FinalizedFragment,
        rect: Option<ScrollRectOf<S>>,
        baseline: Point<S>,
        line_index: usize,
    ) -> Self {
        Self {
            fragment,
            rect,
            baseline,
            line_index,
        }
    }

    /// The exact issued fragment and its retained finalized-line owner.
    #[must_use]
    pub fn finalized_fragment(&self) -> &surgeist_text::FinalizedFragment {
        &self.fragment
    }

    /// Physical bounds in the paragraph container's coordinate system.
    #[must_use]
    pub const fn rect(&self) -> Option<ScrollRectOf<S>> {
        self.rect
    }
    /// Physical baseline anchor in the same paragraph-container-local coordinates.
    #[must_use]
    pub const fn baseline(&self) -> Point<S> {
        self.baseline
    }
    #[must_use]
    pub const fn line_index(&self) -> usize {
        self.line_index
    }
}

/// The actual semantic owner of one retained finalized fragment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InlineFragmentOwner<Node> {
    Node(Node),
    Scope(crate::TextScopeBinding<Node>),
}

/// Associates one retained finalized fragment with its actual semantic owner.
#[derive(Clone, Debug, PartialEq)]
pub struct InlineFragmentOutputEntryOf<Node, S: LayoutScalar = DefaultScalar> {
    owner: InlineFragmentOwner<Node>,
    fragment: InlineFragmentOutputOf<S>,
}

pub type InlineFragmentOutputEntry<Node> = InlineFragmentOutputEntryOf<Node, DefaultScalar>;

impl<Node: Copy, S: LayoutScalar> InlineFragmentOutputEntryOf<Node, S> {
    pub(crate) fn new(
        owner: InlineFragmentOwner<Node>,
        fragment: InlineFragmentOutputOf<S>,
    ) -> Self {
        Self { owner, fragment }
    }
    #[must_use]
    pub const fn owner(&self) -> &InlineFragmentOwner<Node> {
        &self.owner
    }
    #[must_use]
    pub fn fragment(&self) -> &InlineFragmentOutputOf<S> {
        &self.fragment
    }
}

/// The exact placed Text result published with its containing Layout node.
#[derive(Clone, Debug)]
pub struct PlacedTextOutput<Node> {
    container: Node,
    nodes: Vec<Node>,
    layout: surgeist_text::Layout,
    inline_boxes: crate::text_layout::AdmittedInlineBoxes,
}

impl<Node: Copy> PlacedTextOutput<Node> {
    pub(crate) fn new(
        container: Node,
        nodes: Vec<Node>,
        layout: surgeist_text::Layout,
        inline_boxes: crate::text_layout::AdmittedInlineBoxes,
    ) -> Self {
        Self {
            container,
            nodes,
            layout,
            inline_boxes,
        }
    }
    #[must_use]
    pub const fn container(&self) -> Node {
        self.container
    }
    /// Exact captured run membership, retaining the node-to-placement join.
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    #[must_use]
    pub fn layout(&self) -> &surgeist_text::Layout {
        &self.layout
    }

    /// Complete-body fragment identity attached to this exact placed binding.
    /// None means not requested; a demanded result can have no placed pieces.
    #[must_use]
    pub fn inline_box_fragments(&self) -> Option<PlacedInlineBoxFragments<'_>> {
        self.inline_boxes
            .family()
            .map(|family| PlacedInlineBoxFragments {
                layout: &self.layout,
                family,
            })
    }
}

/// Borrowed Text-owned piece readback from one exact Layout binding result.
#[derive(Clone, Copy)]
pub struct PlacedInlineBoxFragments<'layout> {
    layout: &'layout surgeist_text::Layout,
    family: &'layout surgeist_text::PreparedInlineBoxFamily,
}
impl PlacedInlineBoxFragments<'_> {
    #[must_use]
    pub fn family(&self) -> &surgeist_text::PreparedInlineBoxFamily {
        self.family
    }
    pub fn pieces(&self) -> impl Iterator<Item = surgeist_text::PlacedInlineBoxPiece<'_>> + '_ {
        self.layout.inline_box_pieces()
    }
}

/// Normal border geometry retained by its actual placement producer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NormalGeometryOf<S: LayoutScalar> {
    pub(crate) location: Point<S>,
    pub(crate) size: Size<S>,
    pub(crate) content: NormalContentOf<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NormalContentOf<S: LayoutScalar> {
    Box(crate::ScrollRectOf<S>),
    Metadata,
    Hidden,
}

impl<S: LayoutScalar> NormalGeometryOf<S> {
    pub(crate) const fn box_geometry(
        location: Point<S>,
        size: Size<S>,
        content_box: crate::ScrollRectOf<S>,
    ) -> Self {
        Self {
            location,
            size,
            content: NormalContentOf::Box(content_box),
        }
    }

    pub(crate) const fn metadata(location: Point<S>, size: Size<S>) -> Self {
        Self {
            location,
            size,
            content: NormalContentOf::Metadata,
        }
    }

    pub(crate) const fn content_box(self) -> Option<crate::ScrollRectOf<S>> {
        match self.content {
            NormalContentOf::Box(content) => Some(content),
            NormalContentOf::Metadata | NormalContentOf::Hidden => None,
        }
    }

    pub(crate) const fn is_hidden(self) -> bool {
        matches!(self.content, NormalContentOf::Hidden)
    }
}

/// Final used normal content extent before publication corner rounding.
/// Issued only from accepted measured geometry, never canonical pixel extents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeasuredInsetBasisOf<S: LayoutScalar> {
    size: Size<S>,
}

impl<S: LayoutScalar> MeasuredInsetBasisOf<S> {
    pub(crate) const fn from_used_content_size(size: Size<S>) -> Self {
        Self { size }
    }

    pub(crate) const fn size(self) -> Size<S> {
        self.size
    }
}

/// The actual paired corner evaluation retained in an accepted atomic source.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CanonicalAtomicGeometryOf<S: LayoutScalar> {
    pub(crate) visual_readback_offset: Point<f64>,
    pub(crate) measured_normal: NormalGeometryOf<S>,
    pub(crate) normal: NormalGeometryOf<S>,
    pub(crate) normal_origin: Point<f64>,
    pub(crate) visual_origin: Point<f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AtomicReadbackPhase {
    Pending,
    Finalized,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GeometryStateOf<S: LayoutScalar> {
    OrdinaryUnrounded {
        measured_normal: NormalGeometryOf<S>,
    },
    OrdinaryCanonical {
        normal: NormalGeometryOf<S>,
    },
    CanonicalAtomic {
        measured_normal: NormalGeometryOf<S>,
        normal: NormalGeometryOf<S>,
        normal_origin: Point<f64>,
        visual_origin: Point<f64>,
        readback: AtomicReadbackPhase,
    },
}

impl<S: LayoutScalar> GeometryStateOf<S> {
    pub(crate) const fn is_unrounded(self) -> bool {
        matches!(
            self,
            Self::OrdinaryUnrounded { .. }
                | Self::CanonicalAtomic {
                    readback: AtomicReadbackPhase::Pending,
                    ..
                }
        )
    }
}

/// One node's coupled geometry and private rounding custody.
/// Public readback exposes the visual packet; warm restoration retains this
/// complete opaque entry rather than reconstructing it from that packet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutOutputEntryOf<Node, S: LayoutScalar = DefaultScalar> {
    node: Node,
    output: NodeOutputOf<S>,
    geometry: GeometryStateOf<S>,
    pub(crate) non_box: Option<crate::NonBoxKind>,
}

pub type LayoutOutputEntry<Node> = LayoutOutputEntryOf<Node, DefaultScalar>;

impl<Node: Copy, S: LayoutScalar> LayoutOutputEntryOf<Node, S> {
    pub(crate) const fn ordinary_unrounded(
        node: Node,
        measured_normal: NormalGeometryOf<S>,
        output: NodeOutputOf<S>,
    ) -> Self {
        Self {
            node,
            output,
            non_box: None,
            geometry: GeometryStateOf::OrdinaryUnrounded { measured_normal },
        }
    }

    pub(crate) const fn hidden_unrounded(node: Node, source_index: crate::SourceIndex) -> Self {
        Self::ordinary_unrounded(
            node,
            NormalGeometryOf {
                location: Point::ZERO,
                size: Size::ZERO,
                content: NormalContentOf::Hidden,
            },
            NodeOutputOf::with_source_index(source_index),
        )
    }

    pub(crate) const fn ordinary_canonical(
        node: Node,
        normal: NormalGeometryOf<S>,
        output: NodeOutputOf<S>,
    ) -> Self {
        Self {
            node,
            output,
            non_box: None,
            geometry: GeometryStateOf::OrdinaryCanonical { normal },
        }
    }

    pub(crate) const fn canonical_atomic(
        node: Node,
        measured_normal: NormalGeometryOf<S>,
        normal: NormalGeometryOf<S>,
        normal_origin: Point<f64>,
        visual_origin: Point<f64>,
        output: NodeOutputOf<S>,
    ) -> Self {
        Self {
            node,
            output,
            non_box: None,
            geometry: GeometryStateOf::CanonicalAtomic {
                measured_normal,
                normal,
                normal_origin,
                visual_origin,
                readback: AtomicReadbackPhase::Pending,
            },
        }
    }

    /// Extract measured custody only from a current unrounded stage. The outer
    /// Option distinguishes wrong phase from non-box metadata.
    pub(crate) fn measured_inset_basis(&self) -> Option<Option<MeasuredInsetBasisOf<S>>> {
        let measured = match self.geometry {
            GeometryStateOf::OrdinaryUnrounded { measured_normal }
            | GeometryStateOf::CanonicalAtomic {
                measured_normal,
                readback: AtomicReadbackPhase::Pending,
                ..
            } => measured_normal,
            _ => return None,
        };
        Some(
            measured
                .content_box()
                .map(|rect| MeasuredInsetBasisOf::from_used_content_size(rect.size())),
        )
    }

    pub(crate) const fn geometry_state(&self) -> GeometryStateOf<S> {
        self.geometry
    }

    pub(crate) const fn finalized_atomic(self, output: NodeOutputOf<S>) -> Option<Self> {
        match self.geometry {
            GeometryStateOf::CanonicalAtomic {
                measured_normal,
                normal,
                normal_origin,
                visual_origin,
                readback: AtomicReadbackPhase::Pending,
            } => Some(Self {
                output,
                geometry: GeometryStateOf::CanonicalAtomic {
                    measured_normal,
                    normal,
                    normal_origin,
                    visual_origin,
                    readback: AtomicReadbackPhase::Finalized,
                },
                ..self
            }),
            _ => None,
        }
    }

    #[must_use]
    pub const fn node(&self) -> Node {
        self.node
    }

    #[must_use]
    pub(crate) fn with_source_index(mut self, index: SourceIndex) -> Self {
        self.output.source_index = index;
        self
    }

    pub(crate) const fn output(&self) -> NodeOutputOf<S> {
        self.output
    }

    #[must_use]
    pub const fn box_output(&self) -> Option<NodeOutputOf<S>> {
        if self.non_box.is_none() {
            Some(self.output)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutCacheStoreEntryOf<Node, S: LayoutScalar = DefaultScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    context: CacheKeyContext,
    output: ComputeOutputOf<S>,
}

pub type LayoutCacheStoreEntry<Node> = LayoutCacheStoreEntryOf<Node, DefaultScalar>;

impl<Node, S> LayoutCacheStoreEntryOf<Node, S>
where
    Node: Copy,
    S: LayoutScalar,
{
    pub(crate) const fn new(
        node: Node,
        input: ComputeInputOf<S>,
        context: CacheKeyContext,
        output: ComputeOutputOf<S>,
    ) -> Self {
        Self {
            node,
            input,
            context,
            output,
        }
    }

    #[must_use]
    pub const fn node(&self) -> Node {
        self.node
    }

    #[must_use]
    pub const fn input(&self) -> &ComputeInputOf<S> {
        &self.input
    }

    #[must_use]
    pub const fn context(&self) -> CacheKeyContext {
        self.context
    }

    #[must_use]
    pub const fn output(&self) -> ComputeOutputOf<S> {
        self.output
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutCacheClearEntry<Node> {
    node: Node,
}

impl<Node> LayoutCacheClearEntry<Node>
where
    Node: Copy,
{
    pub(crate) const fn new(node: Node) -> Self {
        Self { node }
    }

    #[must_use]
    pub const fn node(&self) -> Node {
        self.node
    }
}

#[derive(Clone)]
struct NativeFamilyContext<Node>(std::sync::Arc<crate::inline_wrapping::WrappingContext<Node>>);
impl<Node> core::fmt::Debug for NativeFamilyContext<Node> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeFamilyContext")
            .field("families", &self.0.families.len())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct CompletedLayoutBatchOf<Node, S: LayoutScalar = DefaultScalar> {
    pub(crate) root: Node,
    pub(crate) result: std::sync::Arc<crate::block_fragmentation::ResultIdentity>,
    pub(crate) nodes: Vec<crate::block_fragmentation::NodePublication<Node, S>>,
    pub(crate) target: Option<crate::LayoutTargetBasis>,
    pub(crate) fragmentainers: Vec<crate::LayoutFragmentainerOutputOf<S>>,
    unrounded_inline_fragments: Vec<InlineFragmentOutputEntryOf<Node, S>>,
    final_inline_fragments: Vec<InlineFragmentOutputEntryOf<Node, S>>,
    cache_store_entries: Vec<LayoutCacheStoreEntryOf<Node, S>>,
    cache_clear_entries: Vec<LayoutCacheClearEntry<Node>>,
    invalidated_nodes: Vec<Node>,
    placed_text: Vec<PlacedTextOutput<Node>>,
    native_families: Option<NativeFamilyContext<Node>>,
    native_runs: Vec<crate::inline_wrapping::family::NativePublication<S>>,
    native_completions: Vec<crate::inline_wrapping::family::NativeFormatterCompletion<S>>,
}

pub type CompletedLayoutBatch<Node> = CompletedLayoutBatchOf<Node, DefaultScalar>;

pub(crate) struct CompletedBatchEntries<Node, S: LayoutScalar> {
    pub(crate) unrounded: Vec<LayoutOutputEntryOf<Node, S>>,
    pub(crate) final_entries: Vec<LayoutOutputEntryOf<Node, S>>,
    pub(crate) unrounded_inline: Vec<InlineFragmentOutputEntryOf<Node, S>>,
    pub(crate) final_inline: Vec<InlineFragmentOutputEntryOf<Node, S>>,
    pub(crate) stores: Vec<LayoutCacheStoreEntryOf<Node, S>>,
    pub(crate) clears: Vec<LayoutCacheClearEntry<Node>>,
    pub(crate) invalidated: Vec<Node>,
}

impl<Node: Copy + Eq, S: LayoutScalar> CompletedLayoutBatchOf<Node, S> {
    pub(crate) fn from_entries(root: Node, entries: CompletedBatchEntries<Node, S>) -> Self {
        use crate::block_fragmentation::{NodePublication, ResultIdentity};
        let result = std::sync::Arc::new(ResultIdentity);
        let mut finals = entries.final_entries;
        let nodes = entries
            .unrounded
            .into_iter()
            .enumerate()
            .map(|(index, unrounded)| {
                let final_index = finals
                    .iter()
                    .position(|entry| entry.node() == unrounded.node())
                    .expect("completed rounding paired every published source participant");
                let final_entry = finals.remove(final_index);
                if let Some(kind) = unrounded.non_box {
                    NodePublication::NonBox {
                        kind,
                        unrounded,
                        final_entry,
                    }
                } else {
                    NodePublication::One(crate::FragmentOutputOf {
                        id: crate::LayoutFragmentId {
                            result: result.clone(),
                            index,
                        },
                        node: unrounded.node(),
                        role: crate::BlockGeneration::Principal,
                        frame: None,
                        parent: (unrounded.node() == root)
                            .then_some(crate::FragmentCoordinateSpace::Root),
                        order: 0,
                        previous: None,
                        next: None,
                        children: Vec::new(),
                        graphical: None,
                        payload: crate::FragmentPayloadOf::ActualBox(crate::BoxGeometryPairOf {
                            unrounded,
                            final_entry,
                        }),
                    })
                }
            })
            .collect();
        Self {
            root,
            result,
            nodes,
            target: None,
            fragmentainers: Vec::new(),
            unrounded_inline_fragments: entries.unrounded_inline,
            final_inline_fragments: entries.final_inline,
            cache_store_entries: entries.stores,
            cache_clear_entries: entries.clears,
            invalidated_nodes: entries.invalidated,
            placed_text: Vec::new(),
            native_families: None,
            native_runs: Vec::new(),
            native_completions: Vec::new(),
        }
    }

    pub(crate) fn with_placed_text(mut self, placed_text: Vec<PlacedTextOutput<Node>>) -> Self {
        self.placed_text = placed_text;
        self
    }

    pub(crate) fn with_native_inline_boxes(
        mut self,
        context: Option<std::sync::Arc<crate::inline_wrapping::WrappingContext<Node>>>,
        runs: Vec<crate::inline_wrapping::family::NativePublication<S>>,
        completions: Vec<crate::inline_wrapping::family::NativeFormatterCompletion<S>>,
    ) -> Self {
        self.native_families = context
            .filter(|context| !context.families.is_empty())
            .map(NativeFamilyContext);
        self.native_runs = runs;
        self.native_completions = completions;
        self
    }

    /// Families with an actual successful formatter execution, including
    /// empty completion with no pieces. Order is completion order, not source or
    /// painting order. Mere admission does not certify formatter execution.
    pub fn native_inline_box_families(
        &self,
    ) -> impl Iterator<Item = crate::NativeInlineBoxFamily<Node>> + '_
    where
        Node: Copy + Eq,
    {
        self.native_completions.iter().map(|completion| {
            self.native_families
                .as_ref()
                .expect("observed native formatter retains its context")
                .0
                .families[completion.family]
                .clone()
        })
    }

    /// Borrowed native pieces from actual accepted execution, without inventing
    /// Text fragments or physical rectangles from native scalar observations.
    pub fn native_inline_box_pieces(
        &self,
    ) -> impl Iterator<Item = crate::NativeInlineBoxPieceView<'_, Node, S>> + '_
    where
        Node: Copy + Eq,
    {
        let families = self
            .native_families
            .as_ref()
            .map_or(&[][..], |owner| owner.0.families.as_slice());
        crate::inline_wrapping::family::pieces(families, &self.native_runs)
    }

    /// Actual Text layouts retained in the same atomic node/cache batch.
    #[must_use]
    pub fn placed_text(&self) -> &[PlacedTextOutput<Node>] {
        &self.placed_text
    }

    #[must_use]
    pub const fn root(&self) -> Node {
        self.root
    }

    #[must_use]
    pub fn target_basis(&self) -> Option<&crate::LayoutTargetBasis> {
        self.target.as_ref()
    }

    #[must_use]
    pub fn replaces_whole_root(&self) -> bool {
        self.target.is_some()
    }

    #[must_use]
    pub fn node_layout(&self, node: Node) -> Option<crate::NodeLayoutRefOf<'_, Node, S>> {
        self.nodes
            .iter()
            .find(|entry| entry.node() == node)
            .map(|entry| entry.view())
    }

    pub fn node_layouts(
        &self,
    ) -> impl ExactSizeIterator<Item = (Node, crate::NodeLayoutRefOf<'_, Node, S>)> {
        self.nodes.iter().map(|entry| (entry.node(), entry.view()))
    }

    /// Resolves any fragment, including a non-fragmented principal box.
    pub fn fragment(
        &self,
        id: &crate::LayoutFragmentId,
    ) -> Result<&crate::FragmentOutputOf<Node, S>, crate::LayoutHandleError> {
        if !std::sync::Arc::ptr_eq(&self.result, &id.result) {
            return Err(crate::LayoutHandleError::WrongResult);
        }
        self.fragments()
            .find(|fragment| fragment.id == *id)
            .ok_or(crate::LayoutHandleError::MissingFragment)
    }

    /// All actual fragment occurrences; this order does not specify paint order.
    pub fn fragments(&self) -> impl Iterator<Item = &crate::FragmentOutputOf<Node, S>> {
        use crate::block_fragmentation::NodePublication as N;
        self.nodes.iter().flat_map(|node| match node {
            N::One(value) => core::slice::from_ref(value),
            N::Fragments { fragments, .. } => fragments.as_slice(),
            N::NonBox { .. } => &[],
        })
    }

    /// Accepted child occurrences derived from canonical coordinate-parent links.
    pub fn child_fragments(
        &self,
        parent: &crate::LayoutFragmentId,
    ) -> Result<impl Iterator<Item = &crate::FragmentOutputOf<Node, S>>, crate::LayoutHandleError>
    {
        let parent = self.fragment(parent)?.id().clone();
        Ok(self.fragments().filter(move |fragment| {
            matches!(fragment.coordinate_parent(), Some(crate::FragmentCoordinateSpace::Fragment(id)) if *id == parent)
        }))
    }

    pub fn fragmentainer(
        &self,
        id: &crate::LayoutFragmentainerId,
    ) -> Result<&crate::LayoutFragmentainerOutputOf<S>, crate::LayoutHandleError> {
        if !std::sync::Arc::ptr_eq(&self.result, &id.result) {
            return Err(crate::LayoutHandleError::WrongResult);
        }
        self.fragmentainers
            .get(id.index)
            .ok_or(crate::LayoutHandleError::MissingFragmentainer)
    }

    pub fn fragmentainers(&self) -> &[crate::LayoutFragmentainerOutputOf<S>] {
        &self.fragmentainers
    }

    /// Returns unrounded fragments in source-tree then segment source order.
    #[must_use]
    pub fn unrounded_inline_fragments(&self) -> &[InlineFragmentOutputEntryOf<Node, S>] {
        &self.unrounded_inline_fragments
    }

    /// Returns final fragments in source-tree then segment source order.
    #[must_use]
    pub fn final_inline_fragments(&self) -> &[InlineFragmentOutputEntryOf<Node, S>] {
        &self.final_inline_fragments
    }

    #[must_use]
    pub fn cache_store_entries(&self) -> &[LayoutCacheStoreEntryOf<Node, S>] {
        &self.cache_store_entries
    }

    /// Full cache invalidations to apply before the pending stores.
    ///
    /// Each entry clears measurements and performed layout, including for bound
    /// batches. Whole-root replacement separately removes performed reuse for
    /// otherwise unchanged members; it does not downgrade these explicit effects.
    #[must_use]
    pub fn cache_clear_entries(&self) -> &[LayoutCacheClearEntry<Node>] {
        &self.cache_clear_entries
    }

    /// Returns the inclusive invalidation closure in source-tree order.
    #[must_use]
    pub fn invalidated_nodes(&self) -> &[Node] {
        &self.invalidated_nodes
    }

    /// Checks current authorization, prepares the complete replacement, rechecks
    /// authorization, then commits with a non-retainable call-scoped carrier.
    /// The host keeps target and visible state exclusively owned through commit;
    /// a tracked root accepts only its currently installed bound request.
    pub fn apply_to<Sink>(
        &self,
        sink: &mut Sink,
    ) -> Result<(), crate::LayoutBatchApplyError<Sink::Error>>
    where
        Sink: crate::tree::LayoutBatchSink<Node, S>,
    {
        use crate::LayoutBatchApplyError as E;
        let target = sink.publication_targets().get(self.root).cloned();
        crate::block_fragmentation::check_target(self.target.as_ref(), target.as_ref())?;
        let mut members = Vec::new();
        members
            .try_reserve(self.nodes.len())
            .map_err(|_| E::Allocation)?;
        members.extend(self.nodes.iter().map(|entry| entry.node()));
        if sink
            .publication_targets()
            .overlaps(self.root, &members, self.target.is_some())
        {
            return Err(E::OverlappingRoot);
        }
        let mut previous = Vec::new();
        let prior = sink.publication_targets().published_members(self.root);
        previous
            .try_reserve(prior.len())
            .map_err(|_| E::Allocation)?;
        previous.extend_from_slice(prior);
        let prepared = sink
            .prepare_layout_batch(crate::LayoutBatchPreparation {
                batch: self,
                target: &target,
                previous: &previous,
            })
            .map_err(E::Preparation)?;
        crate::block_fragmentation::check_target(
            self.target.as_ref(),
            sink.publication_targets().get(self.root),
        )?;
        if sink
            .publication_targets()
            .overlaps(self.root, &members, self.target.is_some())
        {
            return Err(E::OverlappingRoot);
        }
        sink.publication_targets_mut()
            .reserve_root(self.root)
            .map_err(|_| E::Allocation)?;
        sink.commit_layout_batch(crate::LayoutBatchCommit {
            batch: self,
            target: &target,
            previous: &previous,
            prepared,
        });
        sink.publication_targets_mut()
            .commit_members(self.root, members);
        Ok(())
    }
}
