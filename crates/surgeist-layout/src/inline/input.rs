use crate::block::BlockChildProjection;
use crate::{
    Compute, InlineBoundaryInputOf, InlineTextInputOf, LayoutInputViewOf, LayoutScalar,
    LineBreakInputOf, Traverse,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InlineParticipantKindOf<'a, S: LayoutScalar> {
    Box(BlockChildProjection<'a, S>),
    InlineText(&'a InlineTextInputOf<S>),
    LineBreak(LineBreakInputOf<S>),
    InlineBoundary(InlineBoundaryInputOf<S>),
}

/// One settled box, text, break, or boundary participant in an inline context.
///
/// The view retains the immutable source loan used by [`Self::kind`].
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InlineParticipantProjection<'input, S: LayoutScalar> {
    input: LayoutInputViewOf<'input, S>,
}

impl<'input, S: LayoutScalar> InlineParticipantProjection<'input, S> {
    #[must_use]
    pub(crate) fn lookup<Tree, M>(tree: &Tree, node: <Tree as Traverse>::Node) -> Self
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        Self {
            input: tree.layout_input(node),
        }
    }

    #[must_use]
    pub(crate) fn kind(&self) -> InlineParticipantKindOf<'input, S> {
        match self.input {
            LayoutInputViewOf::Box(input) => {
                InlineParticipantKindOf::Box(BlockChildProjection::from_node(input))
            }
            LayoutInputViewOf::InlineText(input) => InlineParticipantKindOf::InlineText(input),
            LayoutInputViewOf::LineBreak(input) => InlineParticipantKindOf::LineBreak(input),
            LayoutInputViewOf::InlineBoundary(input) => {
                InlineParticipantKindOf::InlineBoundary(input)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LayoutInputOf;
    use crate::{
        BidiLevel, Clear, Direction, InlineBoundaryKind, InlineBreakOpportunityOf, InlineMetricsOf,
        InlineSegmentId, InlineWhitespaceEdge, ShapedInlineSegmentOf, VerticalAlign, WritingMode,
    };

    fn projection_from_input<S: LayoutScalar>(
        input: &LayoutInputOf<S>,
    ) -> InlineParticipantProjection<'_, S> {
        InlineParticipantProjection {
            input: input.as_view(),
        }
    }

    fn assert_inline_kind_reuses_segment_backing<S: LayoutScalar>() {
        let metrics = InlineMetricsOf::try_new(S::from_f64(7.0), S::from_f64(11.0))
            .expect("valid inline metrics");
        let expected_segments = [
            ShapedInlineSegmentOf::try_new(
                InlineSegmentId::new(17),
                S::from_f64(23.0),
                metrics,
                BidiLevel::try_new(5).expect("valid bidi level"),
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::try_allowed_with_replacement(S::from_f64(2.0))
                    .expect("valid replacement break"),
            )
            .expect("valid first shaped segment"),
            ShapedInlineSegmentOf::try_new(
                InlineSegmentId::new(19),
                S::from_f64(31.0),
                metrics,
                BidiLevel::try_new(2).expect("valid bidi level"),
                InlineWhitespaceEdge::Preserve,
                InlineBreakOpportunityOf::try_allowed_with_replacement(S::from_f64(3.0))
                    .expect("valid replacement break"),
            )
            .expect("valid second shaped segment"),
        ];
        let text = InlineTextInputOf::try_new(expected_segments.to_vec())
            .expect("nonempty text with distinct segment identities");
        // Construct the product projection directly to exclude any provider clone.
        let projection = InlineParticipantProjection {
            input: LayoutInputViewOf::InlineText(&text),
        };
        let original = projection
            .input
            .as_inline_text()
            .expect("stored inline text");
        let InlineParticipantKindOf::InlineText(first) = projection.kind() else {
            panic!("inline text must retain its participant kind");
        };
        let InlineParticipantKindOf::InlineText(second) = projection.kind() else {
            panic!("repeated access must retain the inline-text kind");
        };

        assert_eq!(original.segments(), expected_segments);
        assert_eq!(first.segments(), expected_segments);
        assert_eq!(second.segments(), expected_segments);
        assert!(
            core::ptr::eq(first.segments(), original.segments()),
            "kind inspection must reuse the projection's nonempty segment backing"
        );
        assert!(core::ptr::eq(second.segments(), original.segments()));
        assert!(core::ptr::eq(first.segments(), second.segments()));
    }

    #[test]
    fn inline_kind_reuses_segment_backing_f32() {
        assert_inline_kind_reuses_segment_backing::<f32>();
    }

    #[test]
    fn inline_kind_reuses_segment_backing_f64() {
        assert_inline_kind_reuses_segment_backing::<f64>();
    }

    fn assert_non_box_projection_values<S: LayoutScalar>() {
        let text_metrics = InlineMetricsOf::try_new(S::from_f64(7.0), S::from_f64(11.0))
            .expect("valid non-default text metrics");
        let text_segment = ShapedInlineSegmentOf::try_new(
            InlineSegmentId::new(17),
            S::from_f64(23.0),
            text_metrics,
            BidiLevel::try_new(5).expect("valid bidi level"),
            InlineWhitespaceEdge::Preserve,
            InlineBreakOpportunityOf::try_allowed_with_replacement(S::from_f64(2.0))
                .expect("valid replacement break"),
        )
        .expect("valid shaped inline segment");
        let text = InlineTextInputOf::try_new(vec![text_segment])
            .expect("nonempty inline text with unique segment identity");
        let text_input = LayoutInputOf::inline_text(text.clone());
        let text_projection = projection_from_input(&text_input);
        let InlineParticipantKindOf::InlineText(projected_text) = text_projection.kind() else {
            panic!("inline text input must retain its participant kind");
        };
        assert_eq!(projected_text, &text);
        assert_eq!(projected_text.segments(), &[text_segment]);

        let break_metrics = InlineMetricsOf::try_new(S::from_f64(13.0), S::from_f64(19.0))
            .expect("valid non-default line-break metrics");
        let line_break = LineBreakInputOf::new()
            .with_metrics(break_metrics)
            .hidden()
            .with_direction(Direction::Rtl)
            .with_writing_mode(WritingMode::VerticalLr)
            .with_vertical_align(VerticalAlign::Top)
            .with_clear(Clear::Right);
        let line_break_input = LayoutInputOf::line_break(line_break);
        let line_break_projection = projection_from_input(&line_break_input);
        let InlineParticipantKindOf::LineBreak(projected_break) = line_break_projection.kind()
        else {
            panic!("line-break input must retain its participant kind");
        };
        assert_eq!(projected_break, line_break);
        assert_eq!(projected_break.metrics(), break_metrics);
        assert_eq!(projected_break.direction(), Direction::Rtl);
        assert_eq!(projected_break.writing_mode(), WritingMode::VerticalLr);
        assert_eq!(projected_break.vertical_align(), VerticalAlign::Top);
        assert_eq!(projected_break.clear(), Clear::Right);

        let boundary_metrics = InlineMetricsOf::try_new(S::from_f64(29.0), S::from_f64(31.0))
            .expect("valid non-default inline-boundary metrics");
        let boundary = InlineBoundaryInputOf::new(InlineBoundaryKind::End, boundary_metrics)
            .with_writing_mode(WritingMode::SidewaysRl)
            .with_direction(Direction::Rtl)
            .with_vertical_align(VerticalAlign::Bottom);
        let boundary_input = LayoutInputOf::inline_boundary(boundary);
        let boundary_projection = projection_from_input(&boundary_input);
        let InlineParticipantKindOf::InlineBoundary(projected_boundary) =
            boundary_projection.kind()
        else {
            panic!("inline-boundary input must retain its participant kind");
        };
        assert_eq!(projected_boundary, boundary);
        assert_eq!(projected_boundary.kind(), InlineBoundaryKind::End);
        assert_eq!(projected_boundary.metrics(), boundary_metrics);
        assert_eq!(projected_boundary.writing_mode(), WritingMode::SidewaysRl);
        assert_eq!(projected_boundary.direction(), Direction::Rtl);
        assert_eq!(projected_boundary.vertical_align(), VerticalAlign::Bottom);
    }

    #[test]
    fn node_projection_block_inline_non_box_f32_retains_each_participant_value() {
        assert_non_box_projection_values::<f32>();
    }

    #[test]
    fn node_projection_block_inline_non_box_f64_retains_each_participant_value() {
        assert_non_box_projection_values::<f64>();
    }
}
