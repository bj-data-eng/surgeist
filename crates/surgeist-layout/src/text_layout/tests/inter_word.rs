//! Native InterWord composition through real registered-font paragraph capture.
//! Literal raw-font geometry distinguishes actual alignment fill from allocated
//! sums, tab absorption and excluded trailing flow.
use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn direction(rtl: bool) -> ParagraphBaseDirection {
    if rtl {
        ParagraphBaseDirection::RightToLeft
    } else {
        ParagraphBaseDirection::LeftToRight
    }
}

fn style(word_spacing: f64, wrap: Wrap) -> text::Style {
    text::Style {
        white_space: text::WhiteSpace::Preserve,
        word_spacing,
        wrap,
        ..ahem_style(10.0, 10.0)
    }
}

fn paragraph(source: &str, word_spacing: f64, wrap: Wrap) -> PreparedParagraph {
    PreparedParagraph::plain(1, Source::new(source), style(word_spacing, wrap))
}

fn block<S: LayoutScalar>(width: f64, last: TextAlignLast, rtl: bool) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        direction: if rtl { Direction::Rtl } else { Direction::Ltr },
        text_align: TextAlign::Justify,
        text_align_last: last,
        text_justification: TextJustification::InterWord,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    }
}

fn tree<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    input: NodeInputOf<S>,
) -> PublicLayoutTreeOf<S> {
    PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [])
        .style(0, input)
        .input(1, paragraph.text_input(1))
}

fn compute<Tree: LayoutTree<Node = u32>>(
    tree: &Tree,
    paragraph: &PreparedParagraph,
    available: AvailableOf<Tree::Scalar>,
    dirty: &[u32],
) -> CompletedLayoutBatchOf<u32, Tree::Scalar>
where
    Tree::MeasureError: std::fmt::Debug,
{
    let capture = LayoutFlowInput::try_new(
        tree,
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
    compute_layout_with_flow_invalidated(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap(),
        dirty,
    )
    .unwrap()
}

fn finite<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    width: f64,
    input: NodeInputOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    compute(
        &tree(paragraph, input),
        paragraph,
        AvailableOf::definite(S::from_f64(width)),
        &[],
    )
}

fn lines<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::FinalizedLine> {
    let mut result = vec![None; batch.placed_text()[0].layout().lines().len()];
    for row in batch.final_inline_fragments() {
        let fragment = row.fragment();
        result[fragment.line_index()] = Some(fragment.finalized_fragment().finalized_line());
    }
    result.into_iter().map(Option::unwrap).collect()
}

#[derive(Clone, Debug, PartialEq)]
struct GlyphFact {
    line: usize,
    source: text::Range,
    id: u32,
    x: f64,
    y: f64,
    raw_advance: f32,
}

fn glyphs<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<GlyphFact> {
    let mut facts = Vec::new();
    for run in batch.placed_text()[0].layout().glyph_runs() {
        assert_eq!(run.orientation(), text::LineOrientation::Horizontal);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs() {
            assert_eq!((glyph.x(), glyph.y()), (0.0, 0.0));
            assert_eq!(glyph.range().len(), 1);
            facts.push(GlyphFact {
                line: run.line(),
                source: glyph.range(),
                id: glyph.id(),
                x: run.origin().x + glyph.pen() + f64::from(glyph.x()),
                y: run.origin().y + f64::from(glyph.y()),
                raw_advance: glyph.advance(),
            });
        }
    }
    facts.sort_by_key(|fact| (fact.line, fact.source.start));
    facts
}

fn positions<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<(usize, usize, f64)> {
    glyphs(batch)
        .into_iter()
        .map(|fact| (fact.line, fact.source.start, fact.x))
        .collect()
}

fn source_copy<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, source: &str) {
    let layout = batch.placed_text()[0].layout();
    assert_eq!(layout.source().text(), &text::Text::from(source));
    let endpoint = |offset, affinity| {
        layout
            .positions()
            .find(|position| position.source_cursor() == text::Cursor::new(offset, affinity))
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
    assert_eq!(
        selected.request().context.spacing,
        text::SpacingPolicy::InterWord
    );
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    let rows = line.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), expected.len());
    assert_eq!(rows.len(), slots.len());
    for ((slot, row), &expected) in slots.iter().zip(&rows).zip(expected) {
        assert_eq!(&row.slot, slot);
        assert_eq!(row.slot.selected_line(), selected);
        assert_eq!(row.allocation, expected);
    }
}

fn one_slot(line: &text::FinalizedLine, authored: f64, allocation: f64, advance: f64, at: usize) {
    amounts(line, &[allocation]);
    let row = line.spacing_slots().next().unwrap();
    assert_eq!(
        row.geometry,
        text::SpacingSlotGeometry::Independent {
            intrinsic_advance: 10.0
        }
    );
    assert_eq!(row.authored_word_spacing, authored);
    assert_eq!(row.advance, advance);
    let pieces = row.slot.pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 1);
    let text::SpacingSlotOwner::Source {
        local, aggregate, ..
    } = pieces[0].owner()
    else {
        panic!("actual authored separator must retain its source owner")
    };
    assert_eq!(local, text::Range::new(at, at + 1));
    assert_eq!(aggregate, local);
}

// Inspect the licensed fixture's actual design units, rather than derive the
// geometry oracle from a Text width query. Glyph0 is .notdef, not Hebrew coverage.
fn table(font: &[u8], tag: &[u8; 4]) -> (usize, usize, usize) {
    let u16_at = |at| u16::from_be_bytes(font[at..at + 2].try_into().unwrap());
    let u32_at = |at| u32::from_be_bytes(font[at..at + 4].try_into().unwrap()) as usize;
    for index in 0..usize::from(u16_at(4)) {
        let record = 12 + index * 16;
        if &font[record..record + 4] == tag {
            return (record, u32_at(record + 8), u32_at(record + 12));
        }
    }
    panic!("missing fixture table {tag:?}")
}

fn qualify_ahem() {
    let (_, head, _) = table(AHEM, b"head");
    let (_, hhea, _) = table(AHEM, b"hhea");
    let (_, hmtx, _) = table(AHEM, b"hmtx");
    assert_eq!(
        u16::from_be_bytes(AHEM[head + 18..head + 20].try_into().unwrap()),
        1000
    );
    let count = u16::from_be_bytes(AHEM[hhea + 34..hhea + 36].try_into().unwrap());
    assert!(count > 153);
    // cmap: SPACE3, ZERO18, A35/B36/C37, NBSP153.
    for glyph in [0, 3, 18, 35, 36, 37, 153] {
        let at = hmtx + glyph * 4;
        assert_eq!(
            u16::from_be_bytes(AHEM[at..at + 2].try_into().unwrap()),
            1000
        );
    }
}

fn basic<S: LayoutScalar>() {
    qualify_ahem();
    let p = paragraph("A\u{a0}B", 4.0, Wrap::None);
    let actual = finite::<S>(&p, 40.0, block(40.0, TextAlignLast::Justify, false));
    let line = &lines(&actual)[0];
    assert!(line.run_views().all(|run| run.face().bytes() == AHEM));
    one_slot(line, 4.0, 6.0, 20.0, 1);
    assert_eq!(line.measurements().alignment().advance(), 40.0);
    assert_eq!(line.feedback().advance, 40.0);
    assert_eq!(
        line.selected_line().request().context.alignment,
        text::Alignment::Justify
    );
    assert_eq!(
        positions(&actual),
        [(0, 0, 0.0), (0, 1, 15.0), (0, 2, 30.0)]
    );
    assert_eq!(
        glyphs(&actual).iter().map(|g| g.id).collect::<Vec<_>>(),
        [35, 153, 36]
    );
    assert!(
        glyphs(&actual)
            .iter()
            .all(|g| g.raw_advance == 10.0 && g.y == 8.0)
    );
    let row = line.spacing_slots().next().unwrap();
    assert_eq!(row.stretchability, text::SpacingStretchability::Direct);
    assert!(row.can_expand());
    assert_eq!(
        row.slot
            .pieces()
            .map(|piece| piece.owner())
            .collect::<Vec<_>>(),
        [text::SpacingSlotOwner::Source {
            origin: p.origin(1),
            local: text::Range::new(1, 2),
            aggregate: text::Range::new(1, 2)
        }]
    );
    let placed = actual.placed_text()[0].layout();
    for (at, x) in [(1, 10.0), (2, 30.0)] {
        assert_eq!(
            placed
                .cursor(text::Cursor::new(at, text::Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .x,
            x
        );
    }
    let text::Hit::Text(hit) = placed.hit(text::Point::new(31.0, 5.0)) else {
        panic!("actual allocated B cell must retain text hit ownership")
    };
    assert_eq!(hit.source_cursor().index(), 2);
    source_copy(&actual, "A\u{a0}B");

    let mut spaces = block(40.0, TextAlignLast::Justify, false);
    spaces.text_justification = TextJustification::Spaces;
    let distinct = finite::<S>(&p, 40.0, spaces);
    let natural = &lines(&distinct)[0];
    assert_eq!(
        natural.selected_line().request().context.spacing,
        text::SpacingPolicy::Spaces
    );
    assert_eq!(natural.spacing_slots().len(), 0);
    assert_eq!(natural.measurements().alignment().advance(), 34.0);
    assert_eq!(
        natural.selected_line().request().context.alignment,
        text::Alignment::Center
    );
    assert_eq!(
        positions(&distinct),
        [(0, 0, 3.0), (0, 1, 15.0), (0, 2, 27.0)]
    );

    // Cancellation changes adjusted width, not the actual intrinsic10 proof.
    let cancelled = paragraph("A\u{a0}B", -10.0, Wrap::None);
    let actual = finite::<S>(&cancelled, 40.0, block(40.0, TextAlignLast::Justify, false));
    one_slot(&lines(&actual)[0], -10.0, 20.0, 20.0, 1);
    assert_eq!(lines(&actual)[0].measurements().alignment().advance(), 40.0);
    assert_eq!(
        positions(&actual),
        [(0, 0, 0.0), (0, 1, 15.0), (0, 2, 30.0)]
    );
}

#[test]
fn nbsp_is_inter_word_but_not_spaces_and_cancelled_intrinsic_width_still_expands() {
    basic::<f32>();
    basic::<f64>();
}

fn line_roles<S: LayoutScalar>() {
    let p = paragraph("A\u{a0}B\nA\u{a0}B", 4.0, Wrap::None);
    for (last, alignment, allocation, advance, x) in [
        (
            TextAlignLast::Justify,
            text::Alignment::Justify,
            6.0,
            40.0,
            [0.0, 15.0, 30.0],
        ),
        (
            TextAlignLast::Auto,
            text::Alignment::Start,
            0.0,
            34.0,
            [0.0, 12.0, 24.0],
        ),
    ] {
        let actual = finite::<S>(&p, 40.0, block(40.0, last, false));
        let selected = lines(&actual);
        assert_eq!(selected.len(), 2);
        assert_eq!(
            selected[0].selected_line().request().context.end,
            text::LineEnd::Forced
        );
        assert_eq!(
            selected[1].selected_line().request().context.end,
            text::LineEnd::Paragraph
        );
        for line in &selected {
            amounts(line, &[allocation]);
            assert_eq!(line.measurements().alignment().advance(), advance);
            assert_eq!(line.selected_line().request().context.alignment, alignment);
        }
        assert_eq!(
            positions(&actual),
            [
                (0, 0, x[0]),
                (0, 1, x[1]),
                (0, 2, x[2]),
                (1, 4, x[0]),
                (1, 5, x[1]),
                (1, 6, x[2])
            ]
        );
        assert!(
            glyphs(&actual)
                .iter()
                .all(|g| g.y == 8.0 + g.line as f64 * 10.0)
        );
        source_copy(&actual, "A\u{a0}B\nA\u{a0}B");
    }
    let no_separator = paragraph("ABC", 4.0, Wrap::None);
    for (last, alignment, xs) in [
        (
            TextAlignLast::Justify,
            text::Alignment::Center,
            [10.0, 20.0, 30.0],
        ),
        (TextAlignLast::End, text::Alignment::End, [20.0, 30.0, 40.0]),
    ] {
        let actual = finite::<S>(&no_separator, 50.0, block(50.0, last, false));
        let line = &lines(&actual)[0];
        amounts(line, &[]);
        assert_eq!(line.measurements().alignment().advance(), 30.0);
        assert_eq!(line.selected_line().request().context.alignment, alignment);
        assert_eq!(
            positions(&actual),
            [(0, 0, xs[0]), (0, 1, xs[1]), (0, 2, xs[2])]
        );
    }
    let mut input = block::<S>(34.0, TextAlignLast::Justify, false);
    input.size.width = PreferredSizeOf::AUTO;
    let p = paragraph("A\u{a0}B", 4.0, Wrap::None);
    for available in [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT] {
        let actual = compute(&tree(&p, input.clone()), &p, available, &[]);
        let root = actual
            .final_entries()
            .iter()
            .find(|row| row.node() == 0)
            .unwrap();
        assert_eq!(root.output().size.width, S::from_f64(34.0));
        let selected = lines(&actual);
        assert_eq!(selected.len(), 1);
        for line in selected {
            amounts(&line, &[0.0]);
            assert_eq!(line.measurements().alignment().advance(), 34.0);
        }
    }
}

#[test]
fn forced_final_intrinsic_and_no_opportunity_follow_their_actual_alignment_roles() {
    line_roles::<f32>();
    line_roles::<f64>();
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

#[test]
fn genuine_zero_intrinsic_separator_suppresses_author_and_uses_center_fallback() {
    qualify_ahem();
    // Change this one actual provider metric, retaining the licensed face and
    // its cmap. Directory and whole-font checksums remain valid.
    let mut font = AHEM.to_vec();
    let (record, hmtx, length) = table(&font, b"hmtx");
    font[hmtx + 153 * 4..hmtx + 153 * 4 + 2].copy_from_slice(&0u16.to_be_bytes());
    let sum = checksum(&font[hmtx..hmtx + length]);
    font[record + 4..record + 8].copy_from_slice(&sum.to_be_bytes());
    let (_, head, _) = table(&font, b"head");
    font[head + 8..head + 12].fill(0);
    let adjustment = 0xb1b0_afbau32.wrapping_sub(checksum(&font));
    font[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&font), 0xb1b0_afba);
    assert_eq!(&font[hmtx + 153 * 4..hmtx + 153 * 4 + 2], &[0, 0]);
    let own = style(4.0, Wrap::None);
    let p = PreparedParagraph::with_registered_font(
        vec![ParagraphPart::text(1, Source::new("A\u{a0}B"), own.clone())],
        own,
        direction(false),
        &font,
    );
    let actual = finite::<f64>(&p, 40.0, block(40.0, TextAlignLast::Justify, false));
    let line = &lines(&actual)[0];
    assert!(
        line.run_views()
            .all(|run| run.face().bytes() == font.as_slice())
    );
    amounts(line, &[0.0]);
    let row = line.spacing_slots().next().unwrap();
    assert_eq!(
        row.geometry,
        text::SpacingSlotGeometry::Independent {
            intrinsic_advance: 0.0
        }
    );
    assert_eq!((row.authored_word_spacing, row.advance), (0.0, 0.0));
    assert!(!row.can_expand());
    assert_eq!(line.measurements().alignment().advance(), 20.0);
    assert_eq!(
        line.selected_line().request().context.alignment,
        text::Alignment::Center
    );
    assert_eq!(
        positions(&actual),
        [(0, 0, 10.0), (0, 1, 20.0), (0, 2, 20.0)]
    );
    assert_eq!(
        glyphs(&actual)
            .iter()
            .map(|g| g.raw_advance)
            .collect::<Vec<_>>(),
        [10.0, 0.0, 10.0]
    );
    source_copy(&actual, "A\u{a0}B");
}

fn tab_paragraph(rtl: bool, word_spacing: f64) -> PreparedParagraph {
    let base = style(0.0, Wrap::None);
    let mut source = Source::new(if rtl {
        "\u{5d0} \t\u{5d1}\t\u{5d2}"
    } else {
        "A \tB\tC"
    });
    source.span(text::Range::new(1, 2), style(word_spacing, Wrap::None));
    for (range, period) in [(text::Range::new(2, 3), 4.0), (text::Range::new(4, 5), 3.0)] {
        let mut tab = base.clone();
        tab.tab_size = text::TabSize::spaces(period).unwrap();
        source.span(range, tab);
    }
    PreparedParagraph::with_css_tabs(
        vec![ParagraphPart::text(1, source, base.clone())],
        base,
        direction(rtl),
        direction(rtl),
    )
}

#[test]
fn actual_mixed_period_tabs_require_response_correction_and_prove_skipped_fallback() {
    qualify_ahem();
    for rtl in [false, true] {
        for (authored, probe_amount) in [(0.0, 20.0), (4.0, 16.0)] {
            let p = tab_paragraph(rtl, authored);
            // These actual Hebrew inputs have level1; Ahem's glyph0 below is
            // deliberately a .notdef metric control, never script coverage.
            assert!(
                p.prepared
                    .unit_views()
                    .all(|unit| unit.level().is_rtl() == rtl)
            );
            let natural = finite::<f64>(&p, 70.0, block(70.0, TextAlignLast::Justify, rtl));
            let n = &lines(&natural)[0];
            assert!(n.run_views().all(|run| run.face().bytes() == AHEM));
            one_slot(n, authored, 0.0, 10.0 + authored, 1);
            assert_eq!(n.measurements().alignment().advance(), 70.0);
            assert_eq!(
                n.spacing_slots().next().unwrap().stretchability,
                text::SpacingStretchability::TabDependent
            );
            assert_eq!(
                n.selected_line().request().context.tabs,
                text::TabContext::CssStops { origin: Some(0.0) }
            );
            // A real positive amount is absorbed before the first stop. This
            // authentic response probe cannot establish unexpandability.
            let slot = n.spacing_slots().next().unwrap().slot;
            let absorbed = n
                .with_spacing(text::SpacingAllocations::Complete(vec![
                    text::SpacingAllocation { slot, amount: 10.0 },
                ]))
                .unwrap();
            assert_eq!(absorbed.measurements().alignment().advance(), 70.0);
            one_slot(&absorbed, authored, 10.0, 20.0 + authored, 1);
            assert_eq!(n.spacing_slots().next().unwrap().allocation, 0.0);
            // Incoming35 leaves exactly5 to the first stop. Equality with
            // half of actual ZERO10 does not take the strict threshold bump.
            let threshold = n
                .with_spacing(text::SpacingAllocations::Complete(vec![
                    text::SpacingAllocation {
                        slot: n.spacing_slots().next().unwrap().slot,
                        amount: 15.0 - authored,
                    },
                ]))
                .unwrap();
            assert_eq!(threshold.measurements().alignment().advance(), 70.0);
            let exact_probe = n
                .with_spacing(text::SpacingAllocations::Complete(vec![
                    text::SpacingAllocation {
                        slot: n.spacing_slots().next().unwrap().slot,
                        amount: probe_amount,
                    },
                ]))
                .unwrap();
            assert_eq!(exact_probe.measurements().alignment().advance(), 130.0);
            one_slot(&exact_probe, authored, probe_amount, 30.0, 1);

            let actual = finite::<f64>(&p, 130.0, block(130.0, TextAlignLast::Justify, rtl));
            let line = &lines(&actual)[0];
            let row = line.spacing_slots().next().unwrap();
            amounts(line, &[row.allocation]);
            assert!(row.can_expand());
            assert_eq!(
                row.stretchability,
                text::SpacingStretchability::TabDependent
            );
            assert_eq!(row.authored_word_spacing, authored);
            assert_eq!(row.advance, 10.0 + (authored + row.allocation));
            // Independent response preimage: first incoming pen is in (35,75],
            // giving first stop80, then B90, second stop120, final C130.
            // Test the actual rounded pen, not a fragile ideal-real j boundary.
            let incoming = 10.0 + row.advance;
            assert!(
                35.0 < incoming && incoming <= 75.0,
                "actual incoming {incoming}"
            );
            assert_ne!(
                row.allocation, 60.0,
                "naive natural residual overshoots to160"
            );
            assert_eq!(line.measurements().alignment().advance(), 130.0);
            assert_eq!(line.feedback().advance, 130.0);
            assert_eq!(
                line.selected_line().request().context.alignment,
                text::Alignment::Justify
            );
            let half = (authored + row.allocation) / 2.0;
            let expected = if rtl {
                [120.0, 110.0 - half, 40.0, 0.0]
            } else {
                [0.0, 10.0 + half, 80.0, 120.0]
            };
            assert_eq!(
                positions(&actual),
                [
                    (0, 0, expected[0]),
                    (0, 1, expected[1]),
                    (0, 3, expected[2]),
                    (0, 5, expected[3])
                ]
            );
            let facts = glyphs(&actual);
            assert!(facts.iter().all(|g| g.raw_advance == 10.0));
            assert_eq!(
                facts.iter().map(|g| g.id).collect::<Vec<_>>(),
                if rtl {
                    vec![0, 3, 0, 0]
                } else {
                    vec![35, 3, 36, 37]
                }
            );
            source_copy(
                &actual,
                if rtl {
                    "\u{5d0} \t\u{5d1}\t\u{5d2}"
                } else {
                    "A \tB\tC"
                },
            );

            // 100 is skipped: the last30-grid stop90 cannot follow a first
            // 40-grid stop plus B10. It lies between real responses70 and130.
            let skipped = finite::<f64>(&p, 100.0, block(100.0, TextAlignLast::Justify, rtl));
            let line = &lines(&skipped)[0];
            one_slot(line, authored, 0.0, 10.0 + authored, 1);
            assert_eq!(line.measurements().alignment().advance(), 70.0);
            assert_eq!(
                line.selected_line().request().context.alignment,
                text::Alignment::Center
            );
            let expected = if rtl {
                [75.0, 65.0 - authored / 2.0, 35.0, 15.0]
            } else {
                [15.0, 25.0 + authored / 2.0, 55.0, 75.0]
            };
            assert_eq!(
                positions(&skipped),
                [
                    (0, 0, expected[0]),
                    (0, 1, expected[1]),
                    (0, 3, expected[2]),
                    (0, 5, expected[3])
                ]
            );
        }
    }
}

fn trailing<S: LayoutScalar>() {
    let mut own = style(4.0, Wrap::Word);
    own.tab_size = text::TabSize::length(40.0).unwrap();
    let p = PreparedParagraph::with_css_tabs(
        vec![ParagraphPart::text(1, Source::new("A B\tCCC"), own.clone())],
        own,
        direction(false),
        direction(false),
    );
    let actual = finite::<S>(&p, 50.0, block(50.0, TextAlignLast::Auto, false));
    let selected = lines(&actual);
    assert_eq!(selected.len(), 2);
    assert_eq!(
        actual.placed_text()[0].layout().lines()[0].range(),
        text::Range::new(0, 4)
    );
    let first = &selected[0];
    one_slot(first, 4.0, 16.0, 30.0, 1);
    assert_eq!(
        first.spacing_slots().next().unwrap().stretchability,
        text::SpacingStretchability::Direct
    );
    assert_eq!(first.measurements().alignment().advance(), 50.0);
    assert_eq!(first.feedback().advance, 80.0);
    assert_eq!(
        positions(&actual),
        [
            (0, 0, 0.0),
            (0, 1, 20.0),
            (0, 2, 40.0),
            (1, 4, 0.0),
            (1, 5, 10.0),
            (1, 6, 20.0)
        ]
    );
    amounts(&selected[1], &[]);
    assert_eq!(selected[1].measurements().alignment().advance(), 30.0);
    source_copy(&actual, "A B\tCCC");

    let hanging = paragraph("A CCCC", 4.0, Wrap::Word);
    let actual = finite::<S>(&hanging, 40.0, block(40.0, TextAlignLast::Justify, false));
    let first = &lines(&actual)[0];
    amounts(first, &[0.0]);
    let row = first.spacing_slots().next().unwrap();
    assert_eq!(row.stretchability, text::SpacingStretchability::Excluded);
    assert!(!row.can_expand());
    assert_eq!(first.measurements().alignment().advance(), 10.0);
    assert_eq!(first.feedback().advance, 24.0);
    assert_eq!(
        first.selected_line().request().context.alignment,
        text::Alignment::Center
    );
    assert_eq!(&positions(&actual)[..2], &[(0, 0, 15.0), (0, 1, 27.0)]);
}

#[test]
fn excluded_trailing_tab_preserves_direct_stretch_and_hanging_only_is_not_a_fill() {
    trailing::<f32>();
    trailing::<f64>();
}

fn insets<S: LayoutScalar>() {
    let p = paragraph("A B\nAAAAA", 4.0, Wrap::None);
    let mut input = block(80.0, TextAlignLast::Justify, false);
    input.border = Edges::new(
        LengthOf::ZERO,
        LengthOf::px(S::from_f64(3.0)),
        LengthOf::ZERO,
        LengthOf::px(S::from_f64(2.0)),
    );
    input.padding = Edges::new(
        LengthOf::ZERO,
        LengthOf::px(S::from_f64(4.0)),
        LengthOf::ZERO,
        LengthOf::px(S::from_f64(3.0)),
    );
    input.text_indent = TextIndentOf {
        amount: LengthPercentageOf::px(S::from_f64(8.0)).unwrap(),
        each_line: true,
        hanging: false,
    };
    input.text_group_align = TextGroupAlign::Center;
    let actual = finite::<S>(&p, 80.0, input);
    let selected = lines(&actual);
    assert_eq!(selected.len(), 2);
    for line in &selected {
        let selected = line.selected_line();
        let context = &selected.request().context;
        assert_eq!(context.available_inline, Some(68.0));
        assert_eq!(context.indent, 8.0);
        assert_eq!(
            context.alignment_insets,
            text::LineAlignmentInsets::try_new(5.0, 5.0).unwrap()
        );
        assert_eq!(line.measurements().alignment().advance(), 50.0);
    }
    one_slot(&selected[0], 4.0, 16.0, 30.0, 1);
    amounts(&selected[1], &[]);
    assert_eq!(
        positions(&actual),
        [
            (0, 0, 18.0),
            (0, 1, 38.0),
            (0, 2, 58.0),
            (1, 4, 18.0),
            (1, 5, 28.0),
            (1, 6, 38.0),
            (1, 7, 48.0),
            (1, 8, 58.0)
        ]
    );
    source_copy(&actual, "A B\nAAAAA");
}

#[test]
fn actual_group_replay_uses_capacity_after_content_edges_indent_and_group_insets() {
    insets::<f32>();
    insets::<f64>();
}

fn float_input<S: LayoutScalar>(height: f64) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        float: Float::Left,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(30.0)),
            PreferredSizeOf::px(S::from_f64(height)),
        ),
        ..NodeInputOf::default()
    }
}

fn cached_float<S: LayoutScalar>() {
    use super::float_encounters::CachedHost;
    let p = paragraph("A B\nA B", 4.0, Wrap::None);
    let tree = tree(&p, block(80.0, TextAlignLast::Justify, false))
        .children(0, [9, 1])
        .children(9, [])
        .style(9, float_input(10.0));
    let mut host = CachedHost::new(tree);
    let available = AvailableOf::definite(S::from_f64(80.0));
    let cold = compute(&host, &p, available, &[]);
    assert_eq!(
        positions(&cold),
        [
            (0, 0, 30.0),
            (0, 1, 50.0),
            (0, 2, 70.0),
            (1, 4, 0.0),
            (1, 5, 35.0),
            (1, 6, 70.0)
        ]
    );
    let selected = lines(&cold);
    one_slot(&selected[0], 4.0, 16.0, 30.0, 1);
    one_slot(&selected[1], 4.0, 46.0, 60.0, 5);
    assert_eq!(
        selected[0]
            .selected_line()
            .request()
            .context
            .available_inline,
        Some(50.0)
    );
    assert_eq!(
        selected[1]
            .selected_line()
            .request()
            .context
            .available_inline,
        Some(80.0)
    );
    assert_eq!(selected[0].measurements().alignment().advance(), 50.0);
    assert_eq!(selected[1].measurements().alignment().advance(), 80.0);
    cold.apply_to(&mut host).unwrap();
    let stored = cold
        .cache_store_entries()
        .iter()
        .find(|row| row.node() == 9)
        .unwrap();
    assert!(
        host.cache_get(9, stored.input(), stored.context())
            .is_some()
    );
    let warm = compute(&host, &p, available, &[]);
    assert_eq!(warm.final_entries(), cold.final_entries());
    assert_eq!(glyphs(&warm), glyphs(&cold));
    for (warm, cold) in lines(&warm).iter().zip(&selected) {
        assert_eq!(
            warm.spacing_slots()
                .map(|r| r.allocation)
                .collect::<Vec<_>>(),
            cold.spacing_slots()
                .map(|r| r.allocation)
                .collect::<Vec<_>>()
        );
        assert_eq!(warm.measurements().fit(), cold.measurements().fit());
        assert_eq!(
            warm.measurements().alignment(),
            cold.measurements().alignment()
        );
        assert_eq!(
            warm.measurements().min_content_contribution(),
            cold.measurements().min_content_contribution()
        );
        assert_eq!(
            warm.measurements().max_content_contribution(),
            cold.measurements().max_content_contribution()
        );
        assert_eq!(
            warm.measurements().formatted_line_usage(),
            cold.measurements().formatted_line_usage()
        );
    }
    warm.apply_to(&mut host).unwrap();
    host.tree
        .insert_input(9, LayoutInputOf::box_input(float_input(20.0)));
    let changed = compute(&host, &p, available, &[9]);
    let fresh = compute(&CachedHost::new(host.tree.clone()), &p, available, &[]);
    assert_eq!(changed.final_entries(), fresh.final_entries());
    assert_eq!(glyphs(&changed), glyphs(&fresh));
    assert_eq!(
        positions(&changed),
        [
            (0, 0, 30.0),
            (0, 1, 50.0),
            (0, 2, 70.0),
            (1, 4, 30.0),
            (1, 5, 50.0),
            (1, 6, 70.0)
        ]
    );
    for batch in [&changed, &fresh] {
        for (index, line) in lines(batch).iter().enumerate() {
            one_slot(line, 4.0, 16.0, 30.0, 1 + index * 4);
            assert_eq!(
                line.selected_line().request().context.available_inline,
                Some(50.0)
            );
            assert_eq!(line.measurements().alignment().advance(), 50.0);
        }
    }
    // The prior completed packet remains an immutable observation after a
    // changed-band replay; committing the replacement cannot rewrite it.
    changed.apply_to(&mut host).unwrap();
    assert_eq!(
        positions(&cold)[3..],
        [(1, 4, 0.0), (1, 5, 35.0), (1, 6, 70.0)]
    );
    source_copy(&changed, "A B\nA B");
}

#[test]
fn real_float_cache_replay_matches_changed_fresh_and_preserves_old_allocated_packet() {
    cached_float::<f32>();
    cached_float::<f64>();
}
