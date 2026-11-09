use super::*;
use crate::{
    BoxFacts, ContextEffect, PaintLimits, PaintOptions, PaintPhase, PaintReport, Position,
    ProfileMode, ZIndex, accounting::OperationKind,
};
use surgeist_layout::ItemOrder;
use surgeist_ledger::{Limits, Tariffs};

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    fragments: Vec<usize>,
    lines: Vec<usize>,
    primitives: Vec<usize>,
    sources: Vec<usize>,
    content: Vec<ContentEntry>,
    fragment_ranges: Vec<Range<usize>>,
    line_ranges: Vec<Range<usize>>,
    primitive_ranges: Vec<Range<usize>>,
    source_ranges: Vec<Range<usize>>,
    content_ranges: Vec<Range<usize>>,
}

fn exercise(
    request: PaintRequest<'_>,
    limits: PaintLimits,
) -> (Result<Snapshot, PaintError>, PaintReport) {
    let mut meter = PaintOptions::new(limits, ProfileMode::Summary { timing: false })
        .operation(OperationKind::PaintOrder)
        .unwrap();
    let root = meter.root();
    let scopes = crate::algorithm::setup_scopes(&mut meter).unwrap();
    let validation = meter
        .child(
            root,
            PaintPhase::PaintValidation,
            Limits::unlimited(),
            Tariffs::identity(),
        )
        .unwrap();
    let input = match PreparedPaint::new(request, &mut meter, root, &scopes) {
        Continue(input) => input,
        Break(failure) => {
            let error = meter
                .with_phase(scopes[6], |p| failure.cleanup(p))
                .unwrap()
                .unwrap_err();
            let report = meter.finish().unwrap().unwrap();
            assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
            assert_eq!(report.live(ResourceKind::RetainedResultBytes), 0);
            return (Err(error), report);
        }
    };
    let result = meter
        .with_phase(validation, |p| {
            let index = build(&input, p, root, validation)?;
            let snapshot = Snapshot {
                fragments: index.fragments.clone(),
                lines: index.lines.clone(),
                primitives: index.primitives.clone(),
                sources: index.sources.clone(),
                content: index.content.clone(),
                fragment_ranges: index.fragment_ranges.clone(),
                line_ranges: index.line_ranges.clone(),
                primitive_ranges: index.primitive_ranges.clone(),
                source_ranges: index.source_ranges.clone(),
                content_ranges: index.content_ranges.clone(),
            };
            index.cleanup(p, root).unwrap();
            Ok(snapshot)
        })
        .unwrap();
    let mut cleanup = input.core.into_cleanup();
    meter
        .with_phase(scopes[6], |p| {
            cleanup.scratch_cleanup(p).unwrap();
            cleanup.result_cleanup(p).unwrap();
        })
        .unwrap();
    let report = meter.finish().unwrap().unwrap();
    assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
    assert_eq!(report.live(ResourceKind::RetainedResultBytes), 0);
    (result, report)
}

fn facts(parent: Option<usize>, order: usize) -> BoxFacts {
    BoxFacts::new(
        parent.map(BoxId::new),
        SourceIndex::new(order),
        ChildOrder::Source,
        Participation::Ordinary,
        Position::Static,
        ZIndex::Auto,
        ContextEffect::None,
    )
}
fn paint(kind: BoxPaintKind) -> BoxPaint {
    BoxPaint::new(kind, FloatParticipation::None)
}
fn b(owner: usize, parent: Option<usize>, line: Option<usize>, order: usize) -> PaintFragment {
    PaintFragment::box_fragment(
        BoxId::new(owner),
        parent.map(FragmentId::new),
        line.map(LineId::new),
        order,
    )
}
fn t(owner: usize, parent: usize, line: usize, order: usize) -> PaintFragment {
    PaintFragment::text_fragment(
        TextId::new(owner),
        FragmentId::new(parent),
        LineId::new(line),
        order,
    )
}
fn line(container: usize, order: usize) -> PaintLine {
    PaintLine::new(FragmentId::new(container), order)
}
fn primitive(fragment: usize, kind: FragmentPaintKind, order: usize) -> PaintPrimitive {
    PaintPrimitive::fragment(FragmentId::new(fragment), kind, order)
}

struct Input {
    boxes: Vec<BoxFacts>,
    paint: Vec<BoxPaint>,
    texts: Vec<TextSource>,
    fragments: Vec<PaintFragment>,
    lines: Vec<PaintLine>,
    primitives: Vec<PaintPrimitive>,
}
impl Input {
    fn request(&self) -> PaintRequest<'_> {
        PaintRequest::new(
            &self.boxes,
            &self.paint,
            &self.texts,
            &self.lines,
            &self.fragments,
            &self.primitives,
            BoxId::new(0),
        )
    }
    fn valid() -> Self {
        Self {
            boxes: vec![
                facts(None, 0),
                facts(Some(0), 0),
                facts(Some(0), 2),
                facts(Some(0), 3),
            ],
            paint: vec![
                paint(BoxPaintKind::Block),
                paint(BoxPaintKind::Inline),
                paint(BoxPaintKind::InlineBlock),
                paint(BoxPaintKind::BlockReplaced),
            ],
            texts: vec![
                TextSource::new(BoxId::new(0), SourceIndex::new(1)),
                TextSource::new(BoxId::new(1), SourceIndex::new(0)),
                TextSource::new(BoxId::new(2), SourceIndex::new(0)),
            ],
            fragments: vec![
                b(0, None, None, 0),
                b(1, Some(0), Some(0), 0),
                t(0, 0, 0, 0),
                t(1, 1, 0, 0),
                b(2, Some(0), Some(0), 0),
                b(3, Some(0), None, 0),
                t(2, 4, 1, 0),
            ],
            lines: vec![line(0, 0), line(4, 0), line(0, 10)],
            primitives: vec![
                PaintPrimitive::canvas(CanvasPaintKind::BackgroundImages, 0),
                PaintPrimitive::canvas(CanvasPaintKind::BackgroundColor, 0),
                primitive(3, FragmentPaintKind::Text, 0),
                PaintPrimitive::decoration(
                    FragmentId::new(3),
                    BoxId::new(1),
                    DecorationKind::Underline,
                    0,
                ),
                PaintPrimitive::decoration(
                    FragmentId::new(3),
                    BoxId::new(0),
                    DecorationKind::Underline,
                    0,
                ),
                PaintPrimitive::decoration(
                    FragmentId::new(3),
                    BoxId::new(0),
                    DecorationKind::LineThrough,
                    0,
                ),
                primitive(5, FragmentPaintKind::ReplacedContent, 0),
                primitive(1, FragmentPaintKind::Border, 0),
                primitive(1, FragmentPaintKind::BackgroundImages, 1),
                primitive(1, FragmentPaintKind::BackgroundImages, 0),
                primitive(1, FragmentPaintKind::BackgroundColor, 0),
                primitive(1, FragmentPaintKind::Outline, 0),
            ],
        }
    }
}

fn invalid(input: &Input, expected: PaintInputError) {
    assert_eq!(
        exercise(input.request(), PaintLimits::unlimited())
            .0
            .unwrap_err(),
        PaintError::Input(expected)
    );
}

#[test]
fn mixed_sources_occurrences_and_appearance_groups_preserve_exact_slots() {
    let input = Input::valid();
    let (result, _) = exercise(input.request(), PaintLimits::unlimited());
    let index = result.unwrap();
    assert_eq!(index.sources, [1, 4, 2, 3, 5, 6]);
    assert_eq!(index.fragments, [0, 1, 4, 5, 2, 3, 6]);
    assert_eq!(index.lines, [0, 2, 1]);
    assert_eq!(
        index.content,
        [
            ContentEntry::Line(LineId::new(0)),
            ContentEntry::Line(LineId::new(2)),
            ContentEntry::Line(LineId::new(1))
        ]
    );
    assert_eq!(index.content_ranges, index.line_ranges);
    assert_eq!(index.primitives, [1, 0, 10, 9, 8, 7, 11, 4, 3, 2, 5, 6]);
    assert_eq!(index.source_ranges, [0..4, 4..5, 5..6, 6..6]);
    assert_eq!(
        index.fragment_ranges,
        [0..1, 1..2, 2..3, 3..4, 4..5, 5..6, 6..7]
    );
    assert_eq!(
        index.line_ranges,
        [0..2, 2..2, 2..2, 2..2, 2..3, 3..3, 3..3]
    );
    assert_eq!(
        index.primitive_ranges,
        [0..2, 2..2, 2..7, 7..7, 7..11, 11..11, 11..12, 12..12]
    );
}

#[test]
fn root_only_empty_tables_and_empty_occurrences_need_no_draw() {
    let mut input = Input::valid();
    input.boxes.truncate(1);
    input.paint.truncate(1);
    input.texts.clear();
    input.fragments.clear();
    input.lines.clear();
    input.primitives.clear();
    assert!(
        exercise(input.request(), PaintLimits::unlimited())
            .0
            .unwrap()
            .primitives
            .is_empty()
    );
    input.fragments.push(b(0, None, None, usize::MAX));
    input.lines.push(line(0, usize::MAX));
    assert!(
        exercise(input.request(), PaintLimits::unlimited())
            .0
            .unwrap()
            .primitives
            .is_empty()
    );
}

#[test]
fn invalid_box_categories_and_unsupported_tables_are_typed() {
    for kind in [BoxPaintKind::Table, BoxPaintKind::InlineTable] {
        let mut input = Input::valid();
        input.paint[1] = paint(kind);
        invalid(
            &input,
            PaintInputError::UnsupportedBoxKind {
                box_id: BoxId::new(1),
                kind,
            },
        );
    }
    let cases = [
        (
            0,
            paint(BoxPaintKind::InlineBlock),
            BoxCategoryError::RootMustBeBlock,
        ),
        (
            0,
            BoxPaint::new(BoxPaintKind::Block, FloatParticipation::Floating),
            BoxCategoryError::FloatingRoot,
        ),
        (
            1,
            BoxPaint::new(BoxPaintKind::Inline, FloatParticipation::Floating),
            BoxCategoryError::FloatingInline,
        ),
    ];
    for (slot, paint, reason) in cases {
        let mut input = Input::valid();
        input.paint[slot] = paint;
        invalid(
            &input,
            PaintInputError::InvalidBoxCategory {
                box_id: BoxId::new(slot),
                reason,
            },
        );
    }
    let mut input = Input::valid();
    input.paint.pop();
    invalid(
        &input,
        PaintInputError::BoxPaintCount {
            boxes: 4,
            box_paint: 3,
        },
    );
    let mut input = Input::valid();
    input.boxes[1].position = Position::Absolute;
    invalid(
        &input,
        PaintInputError::InvalidBoxCategory {
            box_id: BoxId::new(1),
            reason: BoxCategoryError::OutOfFlowInline,
        },
    );
    let mut input = Input::valid();
    input.boxes[0].child_order = ChildOrder::Flex;
    for facts in &mut input.boxes[1..] {
        facts.participation = Participation::FlexItem(ItemOrder::new(0));
    }
    input.paint[1] = BoxPaint::new(BoxPaintKind::Block, FloatParticipation::Floating);
    invalid(
        &input,
        PaintInputError::InvalidBoxCategory {
            box_id: BoxId::new(1),
            reason: BoxCategoryError::FloatingItem,
        },
    );
    let mut input = Input::valid();
    input.boxes.push(facts(Some(3), 0));
    input.paint.push(paint(BoxPaintKind::Block));
    invalid(
        &input,
        PaintInputError::InvalidBoxCategory {
            box_id: BoxId::new(3),
            reason: BoxCategoryError::ReplacedHasChildren,
        },
    );
}

#[test]
fn text_parent_and_combined_source_positions_are_validated() {
    for parent in [3, 99] {
        let mut input = Input::valid();
        input.texts[0] = TextSource::new(BoxId::new(parent), SourceIndex::new(1));
        let expected = if parent == 99 {
            PaintInputError::MissingTextParent {
                text: TextId::new(0),
                parent: BoxId::new(parent),
            }
        } else {
            PaintInputError::InvalidTextParent {
                text: TextId::new(0),
                parent: BoxId::new(parent),
            }
        };
        invalid(&input, expected);
    }
    let mut input = Input::valid();
    input.texts[0] = TextSource::new(BoxId::new(0), SourceIndex::new(0));
    invalid(
        &input,
        PaintInputError::DuplicateMixedSourceIndex {
            parent: BoxId::new(0),
            source_index: SourceIndex::new(0),
            first: SourceChild::Box(BoxId::new(1)),
            other: SourceChild::Text(TextId::new(0)),
        },
    );
    let mut input = Input::valid();
    input.texts.push(input.texts[0]);
    invalid(
        &input,
        PaintInputError::DuplicateMixedSourceIndex {
            parent: BoxId::new(0),
            source_index: SourceIndex::new(1),
            first: SourceChild::Text(TextId::new(0)),
            other: SourceChild::Text(TextId::new(3)),
        },
    );
    let mut input = Input::valid();
    input.boxes[2].child_order = ChildOrder::Grid;
    invalid(
        &input,
        PaintInputError::InvalidTextParent {
            text: TextId::new(2),
            parent: BoxId::new(2),
        },
    );
}

#[test]
fn fragment_owners_parents_and_lines_are_bounded_before_use() {
    let cases = [
        (
            b(99, Some(0), None, 0),
            PaintInputError::MissingFragmentOwner {
                fragment: FragmentId::new(1),
                owner: FragmentOwner::Box(BoxId::new(99)),
            },
        ),
        (
            t(99, 0, 0, 0),
            PaintInputError::MissingFragmentOwner {
                fragment: FragmentId::new(1),
                owner: FragmentOwner::Text(TextId::new(99)),
            },
        ),
        (
            b(1, Some(99), Some(0), 0),
            PaintInputError::MissingFragmentParent {
                fragment: FragmentId::new(1),
                parent: FragmentId::new(99),
            },
        ),
        (
            b(1, Some(4), Some(0), 0),
            PaintInputError::InvalidFragmentParent {
                fragment: FragmentId::new(1),
                parent: Some(FragmentId::new(4)),
            },
        ),
        (
            b(1, None, Some(0), 0),
            PaintInputError::InvalidFragmentParent {
                fragment: FragmentId::new(1),
                parent: None,
            },
        ),
        (
            b(1, Some(0), Some(99), 0),
            PaintInputError::MissingFragmentLine {
                fragment: FragmentId::new(1),
                line: LineId::new(99),
            },
        ),
    ];
    for (fragment, error) in cases {
        let mut input = Input::valid();
        input.fragments[1] = fragment;
        invalid(&input, error);
    }
    let mut input = Input::valid();
    input.fragments[0] = b(0, Some(0), None, 0);
    invalid(
        &input,
        PaintInputError::InvalidFragmentParent {
            fragment: FragmentId::new(0),
            parent: Some(FragmentId::new(0)),
        },
    );
}

#[test]
fn duplicate_and_backward_occurrence_order_are_rejected() {
    let mut input = Input::valid();
    input.fragments.push(input.fragments[1]);
    invalid(
        &input,
        PaintInputError::DuplicateFragmentOrder {
            owner: FragmentOwner::Box(BoxId::new(1)),
            logical_order: 0,
            first: FragmentId::new(1),
            other: FragmentId::new(7),
        },
    );
    let mut input = Input::valid();
    input.fragments.push(b(1, Some(0), Some(2), 1));
    assert!(
        exercise(input.request(), PaintLimits::unlimited())
            .0
            .is_ok()
    );
    input.fragments[1] = b(1, Some(0), Some(2), 0);
    input.fragments[7] = b(1, Some(0), Some(0), 1);
    invalid(
        &input,
        PaintInputError::InconsistentFragmentOrder {
            first: FragmentId::new(1),
            other: FragmentId::new(7),
        },
    );
    let mut input = Input::valid();
    input.fragments.push(b(0, None, None, 1));
    input.lines.push(line(7, 0));
    input.fragments[1] = b(1, Some(7), Some(3), 0);
    input.fragments.push(b(1, Some(0), Some(0), 1));
    invalid(
        &input,
        PaintInputError::InconsistentFragmentOrder {
            first: FragmentId::new(1),
            other: FragmentId::new(8),
        },
    );
}

#[test]
fn lines_require_valid_formatter_containers_and_unique_logical_order() {
    for (container, missing) in [(99, true), (1, false), (2, false), (5, false)] {
        let mut input = Input::valid();
        input.lines[0] = line(container, 0);
        invalid(
            &input,
            if missing {
                PaintInputError::MissingLineContainer {
                    line: LineId::new(0),
                    container: FragmentId::new(container),
                }
            } else {
                PaintInputError::InvalidLineContainer {
                    line: LineId::new(0),
                    container: FragmentId::new(container),
                }
            },
        );
    }
    let mut input = Input::valid();
    input.lines[2] = line(0, 0);
    invalid(
        &input,
        PaintInputError::DuplicateLineOrder {
            container: FragmentId::new(0),
            logical_order: 0,
            first: LineId::new(0),
            other: LineId::new(2),
        },
    );
}

#[test]
fn required_forbidden_and_exact_inline_line_incidence_are_checked() {
    let mut input = Input::valid();
    input.fragments[1] = b(1, Some(0), None, 0);
    invalid(
        &input,
        PaintInputError::MissingLineIncidence {
            fragment: FragmentId::new(1),
        },
    );
    let mut input = Input::valid();
    input.fragments[5] = b(3, Some(0), Some(0), 0);
    invalid(
        &input,
        PaintInputError::UnexpectedLineIncidence {
            fragment: FragmentId::new(5),
            line: LineId::new(0),
        },
    );
    let mut input = Input::valid();
    input.fragments[3] = t(1, 1, 2, 0);
    invalid(
        &input,
        PaintInputError::InvalidLineIncidence {
            fragment: FragmentId::new(3),
            line: LineId::new(2),
        },
    );
    let mut input = Input::valid();
    input.fragments[2] = t(0, 0, 1, 0);
    invalid(
        &input,
        PaintInputError::InvalidLineIncidence {
            fragment: FragmentId::new(2),
            line: LineId::new(1),
        },
    );
    let mut input = Input::valid();
    input.paint[2] = BoxPaint::new(BoxPaintKind::Block, FloatParticipation::Floating);
    invalid(
        &input,
        PaintInputError::UnexpectedLineIncidence {
            fragment: FragmentId::new(4),
            line: LineId::new(0),
        },
    );
    input.fragments[4] = b(2, Some(0), None, 0);
    assert!(
        exercise(input.request(), PaintLimits::unlimited())
            .0
            .is_ok()
    );
}

#[test]
fn atomic_items_floats_and_out_of_flow_boxes_have_no_outer_line() {
    for mode in 0..4 {
        let mut parent = facts(None, 0);
        let mut child = facts(Some(0), 0);
        // These roles have already blockified their outer display category.
        let mut appearance = paint(BoxPaintKind::Block);
        match mode {
            0 => appearance = BoxPaint::new(BoxPaintKind::Block, FloatParticipation::Floating),
            1 => child.position = Position::Absolute,
            2 => {
                parent.child_order = ChildOrder::Flex;
                child.participation = Participation::FlexItem(ItemOrder::new(-1));
            }
            3 => {
                parent.child_order = ChildOrder::Grid;
                child.participation = Participation::GridItem(ItemOrder::new(1));
            }
            _ => unreachable!(),
        }
        let mut input = Input {
            boxes: vec![parent, child],
            paint: vec![paint(BoxPaintKind::Block), appearance],
            texts: vec![TextSource::new(BoxId::new(1), SourceIndex::new(0))],
            fragments: vec![b(0, None, None, 0), b(1, Some(0), None, 0), t(0, 1, 0, 0)],
            lines: vec![line(1, 0)],
            primitives: Vec::new(),
        };
        assert!(
            exercise(input.request(), PaintLimits::unlimited())
                .0
                .is_ok(),
            "mode {mode}"
        );
        input.fragments[1] = b(1, Some(0), Some(0), 0);
        invalid(
            &input,
            PaintInputError::UnexpectedLineIncidence {
                fragment: FragmentId::new(1),
                line: LineId::new(0),
            },
        );
    }
}

#[test]
fn primitive_target_root_background_and_applicator_errors_are_typed() {
    let cases = [
        (
            primitive(99, FragmentPaintKind::Text, 0),
            PaintInputError::MissingPrimitiveFragment {
                primitive: PrimitiveId::new(0),
                fragment: FragmentId::new(99),
            },
        ),
        (
            primitive(1, FragmentPaintKind::Text, 0),
            PaintInputError::InvalidPrimitiveTarget {
                primitive: PrimitiveId::new(0),
                fragment: FragmentId::new(1),
            },
        ),
        (
            primitive(1, FragmentPaintKind::ReplacedContent, 0),
            PaintInputError::InvalidPrimitiveTarget {
                primitive: PrimitiveId::new(0),
                fragment: FragmentId::new(1),
            },
        ),
        (
            primitive(3, FragmentPaintKind::Border, 0),
            PaintInputError::InvalidPrimitiveTarget {
                primitive: PrimitiveId::new(0),
                fragment: FragmentId::new(3),
            },
        ),
        (
            primitive(0, FragmentPaintKind::BackgroundColor, 0),
            PaintInputError::RootFragmentBackground {
                primitive: PrimitiveId::new(0),
                fragment: FragmentId::new(0),
            },
        ),
        (
            primitive(0, FragmentPaintKind::BackgroundImages, 0),
            PaintInputError::RootFragmentBackground {
                primitive: PrimitiveId::new(0),
                fragment: FragmentId::new(0),
            },
        ),
        (
            PaintPrimitive::decoration(
                FragmentId::new(1),
                BoxId::new(0),
                DecorationKind::Underline,
                0,
            ),
            PaintInputError::InvalidPrimitiveTarget {
                primitive: PrimitiveId::new(0),
                fragment: FragmentId::new(1),
            },
        ),
        (
            PaintPrimitive::decoration(
                FragmentId::new(3),
                BoxId::new(99),
                DecorationKind::Underline,
                0,
            ),
            PaintInputError::MissingDecorationApplicator {
                primitive: PrimitiveId::new(0),
                applying_box: BoxId::new(99),
            },
        ),
        (
            PaintPrimitive::decoration(
                FragmentId::new(3),
                BoxId::new(2),
                DecorationKind::Underline,
                0,
            ),
            PaintInputError::InvalidDecorationApplicator {
                primitive: PrimitiveId::new(0),
                text_fragment: FragmentId::new(3),
                applying_box: BoxId::new(2),
            },
        ),
    ];
    for (primitive, error) in cases {
        let mut input = Input::valid();
        input.primitives[0] = primitive;
        invalid(&input, error);
    }
    let mut input = Input::valid();
    input.primitives.push(input.primitives[3]);
    invalid(
        &input,
        PaintInputError::DuplicatePrimitiveOrder {
            first: PrimitiveId::new(3),
            other: PrimitiveId::new(12),
            record_order: 0,
        },
    );
}

#[test]
fn decoration_target_errors_precede_invalid_applicators() {
    for (target, expected) in [
        (
            99,
            PaintInputError::MissingPrimitiveFragment {
                primitive: PrimitiveId::new(0),
                fragment: FragmentId::new(99),
            },
        ),
        (
            1,
            PaintInputError::InvalidPrimitiveTarget {
                primitive: PrimitiveId::new(0),
                fragment: FragmentId::new(1),
            },
        ),
    ] {
        // Neither a missing nor an unrelated applicator masks a target error.
        for applying_box in [BoxId::new(99), BoxId::new(2)] {
            let mut input = Input::valid();
            input.primitives[0] = PaintPrimitive::decoration(
                FragmentId::new(target),
                applying_box,
                DecorationKind::Overline,
                0,
            );
            invalid(&input, expected.clone());
        }
    }
}

fn promoted_input() -> Input {
    Input {
        boxes: vec![facts(None, 0), facts(Some(0), 0), facts(Some(1), 1)],
        paint: vec![
            paint(BoxPaintKind::Block),
            paint(BoxPaintKind::Inline),
            paint(BoxPaintKind::Block),
        ],
        texts: vec![
            TextSource::new(BoxId::new(1), SourceIndex::new(0)),
            TextSource::new(BoxId::new(2), SourceIndex::new(0)),
        ],
        fragments: vec![
            b(0, None, None, 0),
            b(1, Some(0), Some(0), 0),
            PaintFragment::block_in_inline(BoxId::new(2), FragmentId::new(0), 1, 0),
            b(1, Some(0), Some(1), 1),
            t(0, 1, 0, 0),
            t(0, 3, 1, 1),
            t(1, 2, 2, 0),
        ],
        lines: vec![line(0, 0), line(0, 2), line(2, 0)],
        primitives: vec![
            primitive(4, FragmentPaintKind::Text, 0),
            primitive(5, FragmentPaintKind::Text, 0),
            primitive(6, FragmentPaintKind::Text, 0),
        ],
    }
}

#[test]
fn promoted_owner_and_container_errors_precede_placement_eligibility() {
    let mut input = promoted_input();
    input.fragments[2] = PaintFragment::block_in_inline(BoxId::new(99), FragmentId::new(99), 1, 0);
    invalid(
        &input,
        PaintInputError::MissingFragmentOwner {
            fragment: FragmentId::new(2),
            owner: FragmentOwner::Box(BoxId::new(99)),
        },
    );
    input.fragments[2] = PaintFragment::block_in_inline(BoxId::new(2), FragmentId::new(99), 1, 0);
    input.boxes[2].position = Position::Relative;
    invalid(
        &input,
        PaintInputError::MissingFragmentParent {
            fragment: FragmentId::new(2),
            parent: FragmentId::new(99),
        },
    );
}

#[test]
fn promoted_block_content_interleaves_between_lines_without_changing_semantic_ancestry() {
    let mut input = promoted_input();
    input.boxes[1].position = Position::Relative;
    input.boxes[1].z_index = ZIndex::Integer(2);
    let (result, _) = exercise(input.request(), PaintLimits::unlimited());
    let index = result.unwrap();
    assert_eq!(
        index.content,
        [
            ContentEntry::Line(LineId::new(0)),
            ContentEntry::PromotedBlock(FragmentId::new(2)),
            ContentEntry::Line(LineId::new(1)),
            ContentEntry::Line(LineId::new(2))
        ]
    );
    assert_eq!(
        index.content_ranges,
        [0..3, 3..3, 3..4, 4..4, 4..4, 4..4, 4..4]
    );
    assert_eq!(input.boxes[2].parent(), Some(BoxId::new(1)));
    assert_eq!(input.fragments[2].parent(), Some(FragmentId::new(0)));
    // Empty split inline pieces and a block-replaced occurrence keep placement.
    input.texts.clear();
    input.primitives.clear();
    input.fragments.truncate(4);
    input.lines.truncate(2);
    input.paint[2] = paint(BoxPaintKind::BlockReplaced);
    assert!(
        exercise(input.request(), PaintLimits::unlimited())
            .0
            .is_ok()
    );
}

#[test]
fn promoted_content_collisions_and_ineligible_ancestry_are_typed() {
    let mut input = promoted_input();
    input.fragments[2] = PaintFragment::block_in_inline(BoxId::new(2), FragmentId::new(0), 0, 0);
    invalid(
        &input,
        PaintInputError::DuplicateContentOrder {
            container: FragmentId::new(0),
            logical_order: 0,
            first: ContentEntry::Line(LineId::new(0)),
            other: ContentEntry::PromotedBlock(FragmentId::new(2)),
        },
    );
    let mut input = promoted_input();
    input.fragments.push(PaintFragment::block_in_inline(
        BoxId::new(2),
        FragmentId::new(0),
        1,
        1,
    ));
    invalid(
        &input,
        PaintInputError::DuplicateContentOrder {
            container: FragmentId::new(0),
            logical_order: 1,
            first: ContentEntry::PromotedBlock(FragmentId::new(2)),
            other: ContentEntry::PromotedBlock(FragmentId::new(7)),
        },
    );
    for mode in 0..5 {
        let mut input = promoted_input();
        let mut container = FragmentId::new(0);
        match mode {
            0 => input.boxes[2].position = Position::Relative,
            1 => input.paint[2] = BoxPaint::new(BoxPaintKind::Block, FloatParticipation::Floating),
            2 => {
                input.boxes[2].parent = Some(BoxId::new(0));
            }
            3 => {
                input.paint[1] = paint(BoxPaintKind::Block);
            }
            4 => {
                container = FragmentId::new(1);
                input.fragments[2] = PaintFragment::block_in_inline(BoxId::new(2), container, 1, 0);
            }
            _ => unreachable!(),
        }
        invalid(
            &input,
            if mode == 3 {
                PaintInputError::UnsupportedFragmentTopology {
                    fragment: FragmentId::new(2),
                    container,
                    through: BoxId::new(1),
                }
            } else {
                PaintInputError::InvalidBlockInInlinePlacement {
                    fragment: FragmentId::new(2),
                    container,
                }
            },
        );
    }
    let mut input = promoted_input();
    input.boxes.push(facts(Some(0), 2));
    input.paint.push(paint(BoxPaintKind::Block));
    input.fragments.push(b(3, Some(0), None, 0));
    input.fragments[2] = PaintFragment::block_in_inline(BoxId::new(2), FragmentId::new(7), 1, 0);
    invalid(
        &input,
        PaintInputError::InvalidBlockInInlinePlacement {
            fragment: FragmentId::new(2),
            container: FragmentId::new(7),
        },
    );
}

#[test]
fn split_inline_block_fragments_require_explicit_placement_and_coherent_continuations() {
    let mut input = promoted_input();
    input.fragments[2] = b(2, Some(1), None, 0);
    invalid(
        &input,
        PaintInputError::InvalidBlockInInlinePlacement {
            fragment: FragmentId::new(2),
            container: FragmentId::new(1),
        },
    );
    let mut input = promoted_input();
    input.fragments.push(b(2, Some(3), None, 1));
    invalid(
        &input,
        PaintInputError::InvalidBlockInInlinePlacement {
            fragment: FragmentId::new(7),
            container: FragmentId::new(3),
        },
    );
    let mut input = promoted_input();
    input.fragments.push(PaintFragment::block_in_inline(
        BoxId::new(2),
        FragmentId::new(0),
        3,
        1,
    ));
    assert!(
        exercise(input.request(), PaintLimits::unlimited())
            .0
            .is_ok()
    );
    input.fragments[2] = PaintFragment::block_in_inline(BoxId::new(2), FragmentId::new(0), 3, 0);
    input.fragments[7] = PaintFragment::block_in_inline(BoxId::new(2), FragmentId::new(0), 1, 1);
    invalid(
        &input,
        PaintInputError::InconsistentFragmentOrder {
            first: FragmentId::new(2),
            other: FragmentId::new(7),
        },
    );
}

#[test]
fn a_semantic_block_wrapper_requires_typed_topology_capability() {
    let mut input = promoted_input();
    input.boxes.push(facts(Some(0), 0));
    input.paint.push(paint(BoxPaintKind::Block));
    input.boxes[1].parent = Some(BoxId::new(3));
    input.fragments.push(b(3, Some(0), None, 0));
    input.fragments[1] = b(1, Some(7), Some(0), 0);
    input.fragments[3] = b(1, Some(7), Some(1), 1);
    input.lines[0] = line(7, 0);
    input.lines[1] = line(7, 2);
    invalid(
        &input,
        PaintInputError::UnsupportedFragmentTopology {
            fragment: FragmentId::new(2),
            container: FragmentId::new(0),
            through: BoxId::new(3),
        },
    );
}

#[test]
fn promoted_content_refusal_releases_scratch_and_allows_unchanged_retry() {
    let input = promoted_input();
    let (baseline, report) = exercise(input.request(), PaintLimits::unlimited());
    let baseline = baseline.unwrap();
    for kind in [
        WorkKind::PaintValidation,
        WorkKind::PaintOrderComparison,
        WorkKind::OrderMove,
    ] {
        let used = report.work(kind);
        for limit in 0..used {
            assert!(matches!(
                exercise(
                    input.request(),
                    PaintLimits::unlimited().with_work_limit(kind, limit)
                )
                .0,
                Err(PaintError::Budget(_))
            ));
        }
        assert_eq!(
            exercise(
                input.request(),
                PaintLimits::unlimited().with_work_limit(kind, used)
            )
            .0
            .unwrap(),
            baseline
        );
    }
    assert_eq!(
        exercise(input.request(), PaintLimits::unlimited())
            .0
            .unwrap(),
        baseline
    );
}

#[test]
fn every_validation_work_refusal_cleans_up_and_allows_unchanged_retry() {
    let input = Input::valid();
    let (baseline, report) = exercise(input.request(), PaintLimits::unlimited());
    let baseline = baseline.unwrap();
    for kind in [
        WorkKind::PaintValidation,
        WorkKind::FragmentVisit,
        WorkKind::LineVisit,
        WorkKind::InlineMemberVisit,
        WorkKind::PrimitiveVisit,
        WorkKind::PaintOrderComparison,
        WorkKind::DecorationComparison,
        WorkKind::OrderMove,
    ] {
        let used = report.work(kind);
        assert!(used > 0);
        let exact = PaintLimits::unlimited().with_work_limit(kind, used);
        assert_eq!(exercise(input.request(), exact).0.unwrap(), baseline);
        for limit in 0..used {
            let limited = PaintLimits::unlimited().with_work_limit(kind, limit);
            assert!(
                matches!(
                    exercise(input.request(), limited).0,
                    Err(PaintError::Budget(_))
                ),
                "{kind:?} limit {limit}"
            );
        }
    }
    assert_eq!(
        exercise(input.request(), PaintLimits::unlimited())
            .0
            .unwrap(),
        baseline
    );
}

#[test]
fn exact_scratch_peak_and_every_lower_limit_release_all_admitted_backing() {
    let input = Input::valid();
    let (baseline, report) = exercise(input.request(), PaintLimits::unlimited());
    let baseline = baseline.unwrap();
    let peak = report.peak(ResourceKind::ScratchBytes);
    assert!(peak > 0);
    assert_eq!(
        exercise(
            input.request(),
            PaintLimits::unlimited().with_resource_limit(ResourceKind::ScratchBytes, peak)
        )
        .0
        .unwrap(),
        baseline
    );
    for limit in 0..peak {
        assert!(
            matches!(
                exercise(
                    input.request(),
                    PaintLimits::unlimited().with_resource_limit(ResourceKind::ScratchBytes, limit)
                )
                .0,
                Err(PaintError::Budget(_))
            ),
            "scratch limit {limit}"
        );
    }
    assert_eq!(
        exercise(input.request(), PaintLimits::unlimited())
            .0
            .unwrap(),
        baseline
    );
}
