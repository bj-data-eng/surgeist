use crate::node_projection::CommonBoxProjection;
use crate::scroll::{ScrollBoxProjection, ScrollTargetProjection};
use crate::{
    AtomicInlineParticipation, Clear, Compute, Display, Float, FloatExclusion, LayoutScalar,
    NodeInputOf, ScrollbarGutter, ScrollbarWidthOf, TextAlign, TextAlignLast, TextJustification,
    Traverse, VerticalAlign,
};

/// Block-container facts settled at the block algorithm entry.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BlockContainerProjection<'a, S: LayoutScalar> {
    pub(super) common: CommonBoxProjection<'a, S>,
    pub(super) display: Display,
    pub(super) block_generation: crate::BlockGeneration,
    pub(super) text_indent: crate::TextIndentOf<S>,
    pub(super) line_box_direction_policy: crate::LineBoxDirectionPolicy,
    pub(super) hyphenate_limit_lines: crate::HyphenateLimitLines,
    pub(super) hyphenate_limit_last: crate::HyphenateLimitLast,
    pub(super) hyphenate_limit_zone: crate::LengthPercentageOf<S>,
    pub(super) text_align: TextAlign,
    pub(super) text_align_last: TextAlignLast,
    pub(super) text_justification: TextJustification,
    pub(super) scrollbar_gutter: ScrollbarGutter,
    pub(super) scrollbar_width: ScrollbarWidthOf<S>,
}

impl<'a, S: LayoutScalar> BlockContainerProjection<'a, S> {
    #[must_use]
    pub(super) fn from_node(input: &'a NodeInputOf<S>) -> Self {
        Self {
            common: CommonBoxProjection::from_node(input),
            display: input.display,
            block_generation: input.block_generation,
            text_indent: input.text_indent,
            line_box_direction_policy: input.line_box_direction_policy,
            hyphenate_limit_lines: input.hyphenate_limit_lines,
            hyphenate_limit_last: input.hyphenate_limit_last,
            hyphenate_limit_zone: input.hyphenate_limit_zone,
            text_align: input.text_align,
            text_align_last: input.text_align_last,
            text_justification: input.text_justification,
            scrollbar_gutter: input.scrollbar_gutter,
            scrollbar_width: input.scrollbar_width,
        }
    }
}

/// Block-child facts retained after child role lookup.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BlockChildProjection<'a, S: LayoutScalar> {
    pub(super) common: CommonBoxProjection<'a, S>,
    pub(super) display: Display,
    pub(super) float: Float,
    pub(super) clear: Clear,
    pub(super) float_exclusion: FloatExclusion,
    pub(super) atomic_inline_participation: Option<AtomicInlineParticipation>,
    pub(super) vertical_align: VerticalAlign,
    pub(super) scrollbar_width: ScrollbarWidthOf<S>,
}

impl<'a, S: LayoutScalar> BlockChildProjection<'a, S> {
    #[must_use]
    pub(crate) fn from_node(input: &'a NodeInputOf<S>) -> Self {
        Self {
            common: CommonBoxProjection::from_node(input),
            display: input.display,
            float: input.float,
            clear: input.clear,
            float_exclusion: input.float_exclusion,
            atomic_inline_participation: input.atomic_inline_participation,
            vertical_align: input.vertical_align,
            scrollbar_width: input.scrollbar_width,
        }
    }
}

#[must_use]
pub(super) fn block_container_projection<'input, Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
) -> BlockContainerProjection<'input, Tree::Scalar>
where
    Tree: Compute<'input, M>,
{
    BlockContainerProjection::from_node(tree.node_input(node))
}

pub(super) fn with_block_scroll_projections<'input, Tree, M, Output>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    consume: impl FnOnce(
        ScrollBoxProjection<'_, Tree::Scalar>,
        ScrollTargetProjection<'_, Tree::Scalar>,
    ) -> Output,
) -> Output
where
    Tree: Compute<'input, M>,
{
    let input = tree.node_input(node);
    consume(
        ScrollBoxProjection::from_node(input),
        ScrollTargetProjection::from_node(input),
    )
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use super::*;
    use crate::inline::{InlineParticipantKindOf, InlineParticipantProjection};
    use crate::{
        AspectRatioOf, BidiLevel, BoxSizing, ComputeInputOf, ComputeOutputOf, ComputedOverflow,
        Direction, Edges, FlowAxes, InlineBreakOpportunity, LayoutInputOf, LayoutResultOf,
        LengthAutoOf, LengthOf, LengthPercentageOf, MaxSizeOf, MinSizeOf, NodeOutputOf, Overflow,
        Position, PreferredSizeOf, Size, WritingMode,
    };

    struct ProjectionTree<'input, S: LayoutScalar> {
        layout_input: &'input LayoutInputOf<S>,
    }

    impl<'input, S: LayoutScalar> Traverse for ProjectionTree<'input, S> {
        type Node = ();
        type Scalar = S;
        type Children<'a>
            = core::iter::Empty<()>
        where
            Self: 'a;

        fn children(&self, _node: Self::Node) -> Self::Children<'_> {
            core::iter::empty()
        }

        fn child_count(&self, _node: Self::Node) -> usize {
            0
        }

        fn child(&self, _node: Self::Node, _index: usize) -> Self::Node {
            panic!("projection test tree has no children")
        }
    }

    impl<'input, S: LayoutScalar> Compute<'input, Infallible> for ProjectionTree<'input, S> {
        fn observe_grid_overflow(
            &mut self,
            _node: Self::Node,
            _probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
        ) -> crate::LayoutResultOf<
            Self::Node,
            crate::grid::GridOverflowObservation<Self::Scalar>,
            Self::Scalar,
            Infallible,
        > {
            panic!(
                "this focused computation fixture does not admit intrinsic Grid Auto overflow observation"
            );
        }

        fn node_input(&self, _node: Self::Node) -> &'input NodeInputOf<S> {
            self.layout_input.as_box().expect("box projection source")
        }

        fn layout_input(&self, _node: Self::Node) -> crate::LayoutInputViewOf<'input, S> {
            self.layout_input.as_view()
        }

        fn set_unrounded(&mut self, _node: Self::Node, _layout: NodeOutputOf<S>) {}

        fn compute_child(
            &mut self,
            _node: Self::Node,
            _input: ComputeInputOf<S>,
        ) -> LayoutResultOf<Self::Node, ComputeOutputOf<S>, S, Infallible> {
            panic!("projection lookup does not compute children")
        }
    }

    fn scalar<S: LayoutScalar>(value: f64) -> S {
        S::from_f64(value)
    }

    fn length_percentage<S: LayoutScalar>(value: f64) -> LengthPercentageOf<S> {
        LengthPercentageOf::px(scalar(value)).expect("finite test length")
    }

    fn length<S: LayoutScalar>(value: f64) -> LengthOf<S> {
        LengthOf::value(length_percentage(value))
    }

    fn length_auto<S: LayoutScalar>(value: f64) -> LengthAutoOf<S> {
        LengthAutoOf::value(length_percentage(value))
    }

    fn assert_non_default_projection_values<S: LayoutScalar>() {
        let expected_size = Size::new(
            PreferredSizeOf::value(length_percentage(11.0)),
            PreferredSizeOf::value(length_percentage(12.0)),
        );
        let expected_min_size = Size::new(
            MinSizeOf::value(length_percentage(13.0)),
            MinSizeOf::value(length_percentage(14.0)),
        );
        let expected_max_size = Size::new(
            MaxSizeOf::value(length_percentage(101.0)),
            MaxSizeOf::value(length_percentage(102.0)),
        );
        let expected_aspect_ratio =
            Some(AspectRatioOf::new(scalar(1.25)).expect("positive finite aspect ratio"));
        let expected_margin = Edges::new(
            length_auto(1.0),
            length_auto(2.0),
            length_auto(3.0),
            length_auto(4.0),
        );
        let expected_padding = Edges::new(length(5.0), length(6.0), length(7.0), length(8.0));
        let expected_border = Edges::new(length(9.0), length(10.0), length(15.0), length(16.0));
        let expected_inset = Edges::new(
            length_auto(21.0),
            length_auto(22.0),
            length_auto(23.0),
            length_auto(24.0),
        );
        let expected_overflow = ComputedOverflow::try_new(Overflow::Hidden, Overflow::Auto)
            .expect("canonical scrollable overflow pair");
        let expected_scrollbar_width =
            ScrollbarWidthOf::try_new(scalar(32.0)).expect("finite non-negative scrollbar width");
        let expected_atomic_inline_participation = Some(AtomicInlineParticipation::new(
            BidiLevel::try_new(3).expect("valid bidi level"),
            InlineBreakOpportunity::Mandatory,
        ));

        let input = NodeInputOf::<S> {
            display: Display::InlineBlock,
            size: expected_size.clone(),
            min_size: expected_min_size.clone(),
            max_size: expected_max_size.clone(),
            aspect_ratio: expected_aspect_ratio,
            margin: expected_margin,
            padding: expected_padding,
            border: expected_border,
            box_sizing: BoxSizing::ContentBox,
            writing_mode: WritingMode::SidewaysLr,
            direction: Direction::Rtl,
            overflow: expected_overflow,
            position: Position::Absolute,
            inset: expected_inset,
            item_is_replaced: true,
            item_is_table: true,
            text_align: TextAlign::LegacyCenter,
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: expected_scrollbar_width,
            float: Float::Right,
            clear: Clear::Both,
            float_exclusion: FloatExclusion::Shape,
            atomic_inline_participation: expected_atomic_inline_participation,
            vertical_align: VerticalAlign::Bottom,
            ..NodeInputOf::default()
        };

        let common = CommonBoxProjection::from_node(&input);
        let container = BlockContainerProjection::from_node(&input);
        let child = BlockChildProjection::from_node(&input);

        assert_eq!(*common.size, expected_size);
        assert_eq!(*common.min_size, expected_min_size);
        assert_eq!(*common.max_size, expected_max_size);
        assert_eq!(*common.aspect_ratio, expected_aspect_ratio);
        assert_eq!(*common.margin, expected_margin);
        assert_eq!(*common.padding, expected_padding);
        assert_eq!(*common.border, expected_border);
        assert_eq!(common.box_sizing, BoxSizing::ContentBox);
        assert_eq!(
            common.flow_axes,
            FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl)
        );
        assert_eq!(common.overflow, expected_overflow);
        assert_eq!(common.position, Position::Absolute);
        assert_eq!(*common.inset, expected_inset);
        assert!(common.item_is_replaced);
        assert!(common.item_is_table);

        assert_eq!(container.common, common);
        assert!(core::ptr::eq(container.common.size, common.size));
        assert_eq!(container.display, Display::InlineBlock);
        assert_eq!(container.text_align, TextAlign::LegacyCenter);
        assert_eq!(container.scrollbar_gutter, ScrollbarGutter::StableBothEdges);
        assert_eq!(container.scrollbar_width, expected_scrollbar_width);

        assert_eq!(child.common, common);
        assert!(core::ptr::eq(child.common.margin, common.margin));
        assert_eq!(child.display, Display::InlineBlock);
        assert_eq!(child.float, Float::Right);
        assert_eq!(child.clear, Clear::Both);
        assert_eq!(child.float_exclusion, FloatExclusion::Shape);
        assert_eq!(
            child.atomic_inline_participation,
            expected_atomic_inline_participation
        );
        assert_eq!(child.vertical_align, VerticalAlign::Bottom);
        assert_eq!(child.scrollbar_width, expected_scrollbar_width);

        let tree = ProjectionTree {
            layout_input: &LayoutInputOf::box_input(input.clone()),
        };
        let participant = InlineParticipantProjection::lookup::<_, Infallible>(&tree, ());
        let InlineParticipantKindOf::Box(inline_child) = participant.kind() else {
            panic!("box input must project as an inline box participant");
        };

        assert_eq!(inline_child.common, common);
        assert_eq!(inline_child.display, Display::InlineBlock);
        assert_eq!(inline_child.float, Float::Right);
        assert_eq!(inline_child.clear, Clear::Both);
        assert_eq!(inline_child.float_exclusion, FloatExclusion::Shape);
        assert_eq!(
            inline_child.atomic_inline_participation,
            expected_atomic_inline_participation
        );
        assert_eq!(inline_child.vertical_align, VerticalAlign::Bottom);
        assert_eq!(inline_child.scrollbar_width, expected_scrollbar_width);
    }

    #[test]
    fn node_projection_block_inline_f32_selects_exact_box_role_values() {
        assert_non_default_projection_values::<f32>();
    }

    #[test]
    fn node_projection_block_inline_f64_selects_exact_box_role_values() {
        assert_non_default_projection_values::<f64>();
    }
}
