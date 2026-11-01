//! Native allocation and conditional fitting over genuine selected liga bases.
use super::*;
use surgeist_text as text;

#[path = "inter_character/fixtures.rs"]
mod fixtures;
use fixtures::{AHEM, liga, qualify_metrics};

#[path = "inter_character/orientations.rs"]
mod orientations;
#[path = "inter_character/replay.rs"]
mod replay;
#[path = "inter_character/response.rs"]
mod response;
#[path = "inter_character/retry.rs"]
mod retry;

fn own(tracking: f64, wrap: Wrap, overflow: text::OverflowWrap, forced_liga: bool) -> text::Style {
    let mut style = text::Style {
        white_space: text::WhiteSpace::Preserve,
        wrap,
        overflow_wrap: overflow,
        letter_spacing: tracking,
        ..ahem_style(10.0, 10.0)
    };
    if forced_liga {
        style.font = style.font.feature("\"liga\" 1");
    }
    style
}

fn paragraph(bytes: &[u8], source: &str, style: text::Style) -> PreparedParagraph {
    PreparedParagraph::with_registered_font(
        vec![ParagraphPart::text(1, Source::new(source), style.clone())],
        style,
        ParagraphBaseDirection::LeftToRight,
        bytes,
    )
}

fn run<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    available: AvailableOf<S>,
    width: PreferredSizeOf<S>,
    last: TextAlignLast,
) -> LayoutResultOf<u32, CompletedLayoutBatchOf<u32, S>, S> {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                text_align: TextAlign::Justify,
                text_align_last: last,
                text_justification: TextJustification::InterCharacter,
                size: Size::new(width, PreferredSizeOf::AUTO),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap(),
    )
}

fn finite<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    width: f64,
    last: TextAlignLast,
) -> CompletedLayoutBatchOf<u32, S> {
    run(
        paragraph,
        AvailableOf::definite(S::from_f64(width)),
        PreferredSizeOf::px(S::from_f64(width)),
        last,
    )
    .unwrap()
}

fn lines<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::FinalizedLine> {
    assert_eq!(batch.placed_text().len(), 1);
    let mut lines = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        lines[fragment.line_index()] = Some(fragment.finalized_fragment().finalized_line());
    }
    lines.into_iter().map(Option::unwrap).collect()
}

fn raw(line: &text::FinalizedLine, bytes: &[u8]) -> Vec<(text::Range, u32, f32)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.font_size(), 10.0);
            run.glyphs()
                .map(|glyph| {
                    assert_eq!((glyph.x(), glyph.y()), (0.0, 0.0));
                    (glyph.range(), glyph.id(), glyph.advance())
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn paints<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> Vec<(usize, text::Range, u32, f64)> {
    let mut facts = Vec::new();
    for run in batch.placed_text()[0].layout().glyph_runs() {
        assert_eq!(run.orientation(), text::LineOrientation::Horizontal);
        for glyph in run.glyphs() {
            assert_eq!((glyph.x(), glyph.y()), (0.0, 0.0));
            facts.push((
                run.line(),
                glyph.range(),
                glyph.id(),
                run.origin().x + glyph.pen(),
            ));
        }
    }
    facts.sort_by_key(|fact| (fact.0, fact.1.start));
    facts
}

fn copy<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, source: &str) {
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.source().text(), &text::Text::from(source));
    let endpoint = |index, affinity| {
        layout
            .positions()
            .find(|p| p.source_cursor() == text::Cursor::new(index, affinity))
            .unwrap()
    };
    let selection = layout
        .position_selection(
            &endpoint(0, text::Affinity::After),
            &endpoint(source.encode_utf16().count(), text::Affinity::Before),
        )
        .unwrap();
    assert_eq!(selection.source_text(), layout.source().text());
    assert_eq!(selection.plaintext(), &text::Text::from(source));
}

fn amounts(line: &text::FinalizedLine, expected: &[f64]) {
    let selected = line.selected_line();
    let rows = line.spacing_slots().collect::<Vec<_>>();
    assert_eq!(rows.len(), expected.len());
    assert_eq!(
        selected.spacing_slots().collect::<Vec<_>>(),
        rows.iter().map(|row| row.slot.clone()).collect::<Vec<_>>()
    );
    assert_eq!(
        rows.iter().map(|row| row.allocation).collect::<Vec<_>>(),
        expected
    );
    for row in rows {
        assert_eq!(row.slot.selected_line(), selected);
    }
}

fn endpoint(
    unit: &text::SpacingUnit,
    paragraph: &PreparedParagraph,
    range: text::Range,
    tracking: f64,
) {
    assert_eq!(unit.letter_spacing_before(), tracking);
    assert_eq!(unit.letter_spacing_after(), tracking);
    let pieces = unit.pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    assert_eq!(
        pieces[0].owner(),
        text::SpacingSlotOwner::Source {
            origin: paragraph.origin(1),
            local: range,
            aggregate: range
        }
    );
    assert_eq!(
        (
            pieces[0].presentation_range().start(),
            pieces[0].presentation_range().end()
        ),
        (range.start, range.end)
    );
    let participant = paragraph
        .prepared
        .unit_views()
        .find(|unit| unit.source_range() == range)
        .unwrap()
        .handle();
    assert_eq!(
        unit.participants().cloned().collect::<Vec<_>>(),
        [text::PreparedParticipant::Text(participant)]
    );
}

fn boundary(slot: &text::SpacingSlot, paragraph: &PreparedParagraph, at: usize, tracking: f64) {
    let text::SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = slot.target()
    else {
        panic!("InterCharacter retains actual adjacent unit targets")
    };
    endpoint(
        &visual_before,
        paragraph,
        text::Range::new(at, at + 1),
        tracking,
    );
    endpoint(
        &visual_after,
        paragraph,
        text::Range::new(at + 1, at + 2),
        tracking,
    );
}

fn abc<S: LayoutScalar>() {
    let paragraph = paragraph(
        AHEM,
        "ABC",
        own(4.0, Wrap::None, text::OverflowWrap::Normal, false),
    );
    let natural = finite::<S>(&paragraph, 42.0, TextAlignLast::Auto);
    let retained = lines(&natural).remove(0);
    let expected_raw = [
        (text::Range::new(0, 1), 35, 10.0),
        (text::Range::new(1, 2), 36, 10.0),
        (text::Range::new(2, 3), 37, 10.0),
    ];
    assert_eq!(raw(&retained, AHEM), expected_raw);
    assert_eq!(retained.feedback().advance, 38.0);
    assert_eq!(
        retained.selected_line().request().context.spacing,
        text::SpacingPolicy::None
    );
    assert_eq!(
        paints(&natural)
            .iter()
            .map(|fact| fact.3)
            .collect::<Vec<_>>(),
        [0.0, 14.0, 28.0]
    );
    let actual = finite::<S>(&paragraph, 42.0, TextAlignLast::Justify);
    let line = lines(&actual).remove(0);
    assert_eq!(raw(&line, AHEM), expected_raw);
    assert_eq!(
        line.selected_line().request().context.spacing,
        text::SpacingPolicy::InterCharacter
    );
    assert_eq!(
        line.selected_line().request().context.alignment,
        text::Alignment::Justify
    );
    amounts(&line, &[2.0, 2.0]);
    for (index, row) in line.spacing_slots().enumerate() {
        boundary(&row.slot, &paragraph, index, 4.0);
        assert_eq!(
            row.geometry,
            text::SpacingSlotGeometry::Boundary(text::SpacingBoundaryGeometry::Independent)
        );
        assert_eq!(row.stretchability, text::SpacingStretchability::Direct);
        assert!(row.can_expand());
        assert_eq!(
            (
                row.authored_word_spacing,
                row.authored_letter_spacing,
                row.advance
            ),
            (0.0, 4.0, 6.0)
        );
    }
    assert_eq!(line.measurements().alignment().advance(), 42.0);
    assert_eq!(line.feedback().advance, 42.0);
    assert_eq!(
        paints(&actual)
            .iter()
            .map(|fact| fact.3)
            .collect::<Vec<_>>(),
        [0.0, 16.0, 32.0]
    );
    let layout = actual.placed_text()[0].layout();
    for (at, x) in [0.0, 16.0, 32.0, 42.0].into_iter().enumerate() {
        assert_eq!(
            layout
                .cursor(text::Cursor::new(at, text::Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .x,
            x
        );
    }
    let restored = line.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(restored.selected_line(), line.selected_line());
    amounts(&restored, &[0.0, 0.0]);
    assert_eq!(restored.measurements().alignment().advance(), 38.0);
    assert_eq!(raw(&restored, AHEM), expected_raw);
    assert_eq!(retained.feedback().advance, 38.0);
    copy(&actual, "ABC");
}

#[test]
fn inter_character_two_direct_boundaries_add_two_each_after_authored_tracking() {
    qualify_metrics(AHEM, 1000);
    abc::<f32>();
    abc::<f64>();
}

fn bases<S: LayoutScalar>() {
    let bytes = liga(1000);
    let paragraph = paragraph(
        &bytes,
        "AAB",
        own(0.0, Wrap::None, text::OverflowWrap::Normal, false),
    );
    let ordinary_raw = [
        (text::Range::new(0, 2), 58, 10.0),
        (text::Range::new(2, 3), 36, 10.0),
    ];
    let capable_raw = [
        (text::Range::new(0, 1), 35, 10.0),
        (text::Range::new(1, 2), 35, 10.0),
        (text::Range::new(2, 3), 36, 10.0),
    ];
    for last in [TextAlignLast::Auto, TextAlignLast::Start] {
        let ordinary = finite::<S>(&paragraph, 42.0, last);
        let line = lines(&ordinary).remove(0);
        assert_eq!(raw(&line, &bytes), ordinary_raw);
        assert_eq!(line.measurements().alignment().advance(), 20.0);
        assert_eq!(
            line.selected_line().request().context.spacing,
            text::SpacingPolicy::None
        );
        assert_eq!(
            line.selected_line().request().context.alignment,
            text::Alignment::Start
        );
        amounts(&line, &[]);
        assert_eq!(
            paints(&ordinary)
                .iter()
                .map(|fact| fact.3)
                .collect::<Vec<_>>(),
            [0.0, 10.0]
        );
    }
    let capable = finite::<S>(&paragraph, 42.0, TextAlignLast::Justify);
    let line = lines(&capable).remove(0);
    assert_eq!(raw(&line, &bytes), capable_raw);
    assert_eq!(
        line.selected_line().request().context.spacing,
        text::SpacingPolicy::InterCharacter
    );
    assert_eq!(line.measurements().alignment().advance(), 42.0);
    amounts(&line, &[6.0, 6.0]);
    assert_eq!(
        line.with_spacing(text::SpacingAllocations::None)
            .unwrap()
            .measurements()
            .alignment()
            .advance(),
        30.0
    );
    assert_eq!(
        paints(&capable)
            .iter()
            .map(|fact| fact.3)
            .collect::<Vec<_>>(),
        [0.0, 16.0, 32.0]
    );
    for available in [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT] {
        for (last, method, width) in [
            (TextAlignLast::Auto, text::SpacingPolicy::None, 20.0),
            (
                TextAlignLast::Justify,
                text::SpacingPolicy::InterCharacter,
                30.0,
            ),
        ] {
            let actual = run::<S>(&paragraph, available, PreferredSizeOf::AUTO, last).unwrap();
            let root = actual
                .final_entries()
                .iter()
                .find(|row| row.node() == 0)
                .unwrap();
            assert_eq!(root.output().size.width, S::from_f64(width));
            let line = lines(&actual).remove(0);
            assert_eq!(line.selected_line().request().context.spacing, method);
            assert_eq!(line.measurements().alignment().advance(), width);
            if last == TextAlignLast::Auto {
                assert_eq!(raw(&line, &bytes), ordinary_raw);
                amounts(&line, &[]);
            } else {
                assert_eq!(raw(&line, &bytes), capable_raw);
                amounts(&line, &[0.0, 0.0]);
            }
        }
    }
    copy(&capable, "AAB");
}

#[test]
fn effective_terminal_and_intrinsic_context_choose_real_ordinary_or_spacing_capable_liga_basis() {
    bases::<f32>();
    bases::<f64>();
}

fn constrained<S: LayoutScalar>() {
    let bytes = liga(1000);
    let style = own(0.0, Wrap::None, text::OverflowWrap::Normal, true);
    let paragraph = paragraph(&bytes, "AAB", style.clone());
    let ordinary = finite::<S>(&paragraph, 42.0, TextAlignLast::Auto);
    let retained = lines(&ordinary).remove(0);
    let expected_raw = [
        (text::Range::new(0, 2), 58, 10.0),
        (text::Range::new(2, 3), 36, 10.0),
    ];
    assert_eq!(raw(&retained, &bytes), expected_raw);
    assert_eq!(retained.feedback().advance, 20.0);
    let actual = finite::<S>(&paragraph, 42.0, TextAlignLast::Justify);
    let line = lines(&actual).remove(0);
    assert_eq!(raw(&line, &bytes), expected_raw);
    assert_eq!(
        line.selected_line().request().context.spacing,
        text::SpacingPolicy::InterCharacter
    );
    assert_eq!(
        line.selected_line().request().context.alignment,
        text::Alignment::Justify
    );
    amounts(&line, &[0.0, 22.0]);
    let rows = line.spacing_slots().collect::<Vec<_>>();
    boundary(&rows[0].slot, &paragraph, 0, 0.0);
    boundary(&rows[1].slot, &paragraph, 1, 0.0);
    assert_eq!(
        rows[0].geometry,
        text::SpacingSlotGeometry::Boundary(text::SpacingBoundaryGeometry::Constrained)
    );
    assert!(!rows[0].can_expand());
    assert_eq!(rows[0].advance, 0.0);
    assert_eq!(
        rows[1].geometry,
        text::SpacingSlotGeometry::Boundary(text::SpacingBoundaryGeometry::Independent)
    );
    assert_eq!(rows[1].stretchability, text::SpacingStretchability::Direct);
    assert!(rows[1].can_expand());
    assert_eq!(rows[1].advance, 22.0);
    assert_eq!(line.measurements().alignment().advance(), 42.0);
    assert_eq!(line.feedback().advance, 42.0);
    assert_eq!(
        paints(&actual)
            .iter()
            .map(|fact| fact.3)
            .collect::<Vec<_>>(),
        [0.0, 32.0]
    );

    let only = self::paragraph(&bytes, "AA", style);
    let natural = finite::<S>(&only, 20.0, TextAlignLast::Auto);
    assert_eq!(
        raw(&lines(&natural)[0], &bytes),
        [(text::Range::new(0, 2), 58, 10.0)]
    );
    let zero = finite::<S>(&only, 10.0, TextAlignLast::Justify);
    let zero = lines(&zero).remove(0);
    assert_eq!(
        zero.selected_line().request().context.spacing,
        text::SpacingPolicy::InterCharacter
    );
    assert_eq!(
        zero.selected_line().request().context.alignment,
        text::Alignment::Justify
    );
    assert_eq!(zero.measurements().alignment().advance(), 10.0);
    amounts(&zero, &[0.0]);
    assert_eq!(
        zero.spacing_slots().next().unwrap().geometry,
        text::SpacingSlotGeometry::Boundary(text::SpacingBoundaryGeometry::Constrained)
    );
    assert_eq!(raw(&zero, &bytes), [(text::Range::new(0, 2), 58, 10.0)]);
    let error = run::<S>(
        &only,
        AvailableOf::definite(S::from_f64(20.0)),
        PreferredSizeOf::px(S::from_f64(20.0)),
        TextAlignLast::Justify,
    )
    .unwrap_err();
    let LayoutErrorKindOf::Text(error) = error.kind() else {
        panic!("material constrained boundary must retain Text refusal: {error:?}")
    };
    assert_eq!(error.code, text::ErrorCode::LayoutFailed);
    let Some(text::ErrorDetail::UnsupportedSpacing {
        stage,
        operation,
        pieces,
        provider_atoms,
        reason,
        constraints,
        ..
    }) = error.detail()
    else {
        panic!("expected exact preserved-optional refusal: {error:?}")
    };
    assert_eq!(*stage, text::SpacingStage::SelectedShaping);
    assert_eq!(*operation, text::SpacingOperation::InterCharacterAllocation);
    assert_eq!(
        *reason,
        text::SpacingFailureReason::PreservedOptionalLigature
    );
    assert_eq!(
        pieces.as_ref(),
        [
            text::SpacingPiece::Source {
                origin: 0,
                local: text::Range::new(0, 1),
                aggregate: text::Range::new(0, 1)
            },
            text::SpacingPiece::Source {
                origin: 0,
                local: text::Range::new(1, 2),
                aggregate: text::Range::new(1, 2)
            }
        ]
    );
    assert_eq!(provider_atoms.as_ref(), &[0..1, 1..2]);
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0].kind, text::SpacingConstraintKind::Ligature);
    assert_eq!(constraints[0].components.as_ref(), &[0..1, 1..2]);
    assert!(
        constraints[0]
            .provenance
            .iter()
            .any(|feature| feature.tag().to_be_bytes() == *b"liga")
    );
    assert_eq!(retained.feedback().advance, 20.0);
    assert_eq!(raw(&retained, &bytes), expected_raw);
    copy(&actual, "AAB");
}

#[test]
fn explicit_optional_liga_keeps_constrained_zero_with_outside_fill_and_refuses_needed_internal_extra()
 {
    constrained::<f32>();
    constrained::<f64>();
}

fn conditional<S: LayoutScalar>() {
    let bytes = liga(3000);
    let paragraph = paragraph(
        &bytes,
        "AAB",
        own(0.0, Wrap::Word, text::OverflowWrap::Anywhere, false),
    );
    assert_eq!(
        paragraph
            .prepared
            .opportunity_views()
            .map(|view| view.kind())
            .collect::<Vec<_>>(),
        [text::PreparedBreakKind::ParagraphEnd]
    );
    assert_eq!(
        paragraph.prepared.analyzed_source().style().overflow_wrap,
        text::OverflowWrap::Anywhere
    );
    let ordinary = finite::<S>(&paragraph, 40.0, TextAlignLast::Auto);
    let ordinary_line = lines(&ordinary).remove(0);
    assert_eq!(
        raw(&ordinary_line, &bytes),
        [
            (text::Range::new(0, 2), 58, 30.0),
            (text::Range::new(2, 3), 36, 10.0)
        ]
    );
    assert_eq!(ordinary_line.measurements().alignment().advance(), 40.0);
    let capable = finite::<S>(&paragraph, 30.0, TextAlignLast::Justify);
    let capable_line = lines(&capable).remove(0);
    assert_eq!(
        raw(&capable_line, &bytes),
        [
            (text::Range::new(0, 1), 35, 10.0),
            (text::Range::new(1, 2), 35, 10.0),
            (text::Range::new(2, 3), 36, 10.0)
        ]
    );
    assert_eq!(capable_line.measurements().alignment().advance(), 30.0);
    amounts(&capable_line, &[0.0, 0.0]);
    for last in [TextAlignLast::Justify, TextAlignLast::Auto] {
        let actual = finite::<S>(&paragraph, 25.0, last);
        let selected = lines(&actual);
        assert_eq!(selected.len(), 2);
        assert_eq!(
            actual.placed_text()[0]
                .layout()
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [text::Range::new(0, 2), text::Range::new(2, 3)]
        );
        let first = &selected[0];
        assert_eq!(
            raw(first, &bytes),
            [
                (text::Range::new(0, 1), 35, 10.0),
                (text::Range::new(1, 2), 35, 10.0)
            ]
        );
        assert_eq!(first.measurements().alignment().advance(), 25.0);
        assert_eq!(first.feedback().advance, 25.0);
        amounts(first, &[5.0]);
        let selected_line = first.selected_line();
        let request = selected_line.request();
        assert_eq!(request.context.spacing, text::SpacingPolicy::InterCharacter);
        assert_eq!(request.context.alignment, text::Alignment::Justify);
        assert_eq!(request.context.available_inline, Some(25.0));
        assert_eq!(request.context.end, text::LineEnd::Soft);
        let text::PreparedTermination::Conditional(opportunity) = &request.termination else {
            panic!("cut2 must retain its genuine conditional authority")
        };
        assert_eq!(opportunity.boundary(), request.end);
        assert_eq!(
            (
                request.end.view().source_before().get(),
                request.end.view().source_after().get()
            ),
            (2, 2)
        );
        boundary(
            &first.spacing_slots().next().unwrap().slot,
            &paragraph,
            0,
            0.0,
        );
        assert_eq!(
            raw(&selected[1], &bytes),
            [(text::Range::new(2, 3), 36, 10.0)]
        );
        assert_eq!(selected[1].measurements().alignment().advance(), 10.0);
        assert_eq!(
            selected[1].selected_line().request().context.spacing,
            text::SpacingPolicy::None
        );
        amounts(&selected[1], &[]);
        assert_eq!(
            paints(&actual),
            [
                (0, text::Range::new(0, 1), 35, 0.0),
                (0, text::Range::new(1, 2), 35, 15.0),
                (
                    1,
                    text::Range::new(2, 3),
                    36,
                    if last == TextAlignLast::Justify {
                        7.5
                    } else {
                        0.0
                    }
                )
            ]
        );
        copy(&actual, "AAB");
    }
    assert_eq!(ordinary_line.feedback().advance, 40.0);
    assert_eq!(capable_line.feedback().advance, 30.0);
}

#[test]
fn conditional_cut_uses_the_real_prospective_character_basis_for_justify_and_terminal_auto() {
    conditional::<f32>();
    conditional::<f64>();
}

fn ordinary_fallback<S: LayoutScalar>() {
    let bytes = liga(1000);
    let paragraph = paragraph(
        &bytes,
        "AAB",
        own(0.0, Wrap::Word, text::OverflowWrap::Anywhere, false),
    );
    let capable = finite::<S>(&paragraph, 30.0, TextAlignLast::Justify);
    let capable = lines(&capable).remove(0);
    assert_eq!(capable.measurements().alignment().advance(), 30.0);
    assert_eq!(
        raw(&capable, &bytes),
        [
            (text::Range::new(0, 1), 35, 10.0),
            (text::Range::new(1, 2), 35, 10.0),
            (text::Range::new(2, 3), 36, 10.0)
        ]
    );
    let actual = finite::<S>(&paragraph, 25.0, TextAlignLast::Justify);
    let selected = lines(&actual);
    assert_eq!(selected.len(), 1);
    assert_eq!(
        actual.placed_text()[0].layout().lines()[0].range(),
        text::Range::new(0, 3)
    );
    let line = &selected[0];
    assert_eq!(
        raw(line, &bytes),
        [
            (text::Range::new(0, 2), 58, 10.0),
            (text::Range::new(2, 3), 36, 10.0)
        ]
    );
    assert_eq!(line.measurements().fit().advance(), 20.0);
    assert_eq!(line.measurements().alignment().advance(), 20.0);
    assert_eq!(
        line.selected_line().request().context.spacing,
        text::SpacingPolicy::None
    );
    assert_eq!(
        line.selected_line().request().context.alignment,
        text::Alignment::Center
    );
    assert_eq!(
        line.selected_line().request().context.end,
        text::LineEnd::Paragraph
    );
    amounts(line, &[]);
    assert_eq!(
        paints(&actual),
        [
            (0, text::Range::new(0, 2), 58, 2.5),
            (0, text::Range::new(2, 3), 36, 12.5)
        ]
    );
    assert_eq!(capable.feedback().advance, 30.0);
    copy(&actual, "AAB");
}

#[test]
fn overfull_character_basis_is_remeasured_on_ordinary_fallback_before_accepting_the_whole_source() {
    ordinary_fallback::<f32>();
    ordinary_fallback::<f64>();
}
