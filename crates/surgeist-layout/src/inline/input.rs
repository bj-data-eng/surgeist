use crate::block::BlockChildProjection;
use crate::{
    Compute, InlineBoundaryInputOf, LayoutInputViewOf, LayoutScalar, LineBreakInputOf, Traverse,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InlineParticipantKindOf<'a, S: LayoutScalar> {
    Box(BlockChildProjection<'a, S>),
    InlineText(&'a crate::text_layout::TextParticipation),
    TextControl(&'a crate::text_layout::TextControlParticipation),
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
            LayoutInputViewOf::TextControl(input) => InlineParticipantKindOf::TextControl(input),
            LayoutInputViewOf::LineBreak(input) => InlineParticipantKindOf::LineBreak(input),
            LayoutInputViewOf::InlineBoundary(input) => {
                InlineParticipantKindOf::InlineBoundary(input)
            }
        }
    }
}

pub(crate) fn starts_run<S: LayoutScalar>(input: LayoutInputViewOf<'_, S>) -> bool {
    match input {
        LayoutInputViewOf::Box(input) => {
            input.display.is_inline_level()
                && input.position != crate::Position::Absolute
                && input.float.is_none()
        }
        LayoutInputViewOf::InlineText(_)
        | LayoutInputViewOf::TextControl(_)
        | LayoutInputViewOf::InlineBoundary(_) => true,
        LayoutInputViewOf::LineBreak(input) => !input.display().is_none(),
    }
}

pub(crate) fn continues_run<S: LayoutScalar>(input: LayoutInputViewOf<'_, S>) -> bool {
    match input {
        LayoutInputViewOf::Box(input) => {
            input.display == crate::Display::None
                || input.position == crate::Position::Absolute
                || (input.display.is_inline_level() && input.float.is_none())
        }
        LayoutInputViewOf::InlineText(_)
        | LayoutInputViewOf::TextControl(_)
        | LayoutInputViewOf::InlineBoundary(_)
        | LayoutInputViewOf::LineBreak(_) => true,
    }
}

pub(crate) fn run_ranges<'input, Node: Copy, S: LayoutScalar>(
    nodes: &[Node],
    mut input: impl FnMut(Node) -> LayoutInputViewOf<'input, S>,
) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut index = 0;
    while index < nodes.len() {
        if !starts_run(input(nodes[index])) {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < nodes.len() && continues_run(input(nodes[index])) {
            index += 1;
        }
        ranges.push(start..index);
    }
    ranges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LayoutInputOf;
    use crate::test_support::text::{PreparedParagraph, ahem_style};
    use crate::{
        Clear, Direction, InlineBoundaryKind, InlineMetricsOf, VerticalAlign, WritingMode,
    };
    use surgeist_text::Source;

    fn projection_from_input<S: LayoutScalar>(
        input: &LayoutInputOf<S>,
    ) -> InlineParticipantProjection<'_, S> {
        InlineParticipantProjection {
            input: input.as_view(),
        }
    }

    fn assert_inline_kind_reuses_occurrence_loan<S: LayoutScalar>() {
        let paragraph =
            PreparedParagraph::plain(7, Source::new("actual source"), ahem_style(10.0, 12.0));
        let input = paragraph.text_input::<S>(7);
        let projection = projection_from_input(&input);
        let original = input
            .as_inline_text()
            .expect("actual normalized Text occurrence");
        let InlineParticipantKindOf::InlineText(first) = projection.kind() else {
            panic!("retained Text role");
        };
        let InlineParticipantKindOf::InlineText(second) = projection.kind() else {
            panic!("retained Text role");
        };
        assert!(core::ptr::eq(original, first));
        assert!(core::ptr::eq(original, second));
        assert_eq!(first.origin(), &paragraph.origin(7));
    }

    #[test]
    fn inline_kind_reuses_occurrence_loan_f32() {
        assert_inline_kind_reuses_occurrence_loan::<f32>();
    }
    #[test]
    fn inline_kind_reuses_occurrence_loan_f64() {
        assert_inline_kind_reuses_occurrence_loan::<f64>();
    }

    fn assert_non_box_projection_values<S: LayoutScalar>() {
        let paragraph = PreparedParagraph::plain(17, Source::new("source"), ahem_style(10.0, 11.0));
        let text_input = paragraph.text_input::<S>(17);
        let text_projection = projection_from_input(&text_input);
        let InlineParticipantKindOf::InlineText(projected_text) = text_projection.kind() else {
            panic!("retained Text role");
        };
        assert_eq!(projected_text.origin(), &paragraph.origin(17));

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
        let boundary = InlineBoundaryInputOf::new(
            InlineBoundaryKind::End,
            boundary_metrics,
            crate::InlineBoundaryEdgeOf::ZERO,
        )
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
