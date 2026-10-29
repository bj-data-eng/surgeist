//! Actual selected shaping, captured terminal authority and numeric refusals
//! must survive InterWord allocation, including final-line Justify.
use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn style(word_spacing: f64) -> text::Style {
    text::Style {
        white_space: text::WhiteSpace::Preserve,
        wrap: Wrap::Word,
        word_spacing,
        ..ahem_style(10.0, 10.0)
    }
}

fn captured(
    source: &str,
    own: text::Style,
    terminal: text::ParagraphTerminalEdge,
) -> PreparedParagraph {
    // The native producer captures this paragraph's terminal fact before
    // preparation. The physical containing block cannot promote it later.
    let paragraph = PreparedParagraph::at_boundary(
        vec![ParagraphPart::text(1, Source::new(source), own.clone())],
        own,
        ParagraphBaseDirection::LeftToRight,
        terminal,
    );
    assert_eq!(paragraph.prepared.terminal_edge(), terminal);
    paragraph
}

fn compute<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    width: f64,
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
                text_justification: TextJustification::InterWord,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    compute_layout_with_flow(&capture, 0, request(width))
}

fn line<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> text::FinalizedLine {
    assert_eq!(batch.placed_text().len(), 1);
    assert_eq!(batch.placed_text()[0].layout().lines().len(), 1);
    let line = batch.final_inline_fragments()[0]
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert_eq!(
        line.selected_line().request().context.spacing,
        text::SpacingPolicy::InterWord
    );
    assert_eq!(
        line.selected_line().request().context.end,
        text::LineEnd::Paragraph
    );
    assert_eq!(
        line.selected_line().spacing_slots().collect::<Vec<_>>(),
        line.spacing_slots().map(|row| row.slot).collect::<Vec<_>>()
    );
    line
}

fn raw(line: &text::FinalizedLine) -> Vec<(text::Range, u32, f32)> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| {
                    assert_eq!((glyph.x(), glyph.y()), (0.0, 0.0));
                    (glyph.range(), glyph.id(), glyph.advance())
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn text_error<S: LayoutScalar>(error: &LayoutErrorOf<u32, S>) -> &text::Error {
    let LayoutErrorKindOf::Text(error) = error.kind() else {
        panic!("actual Text refusal must remain its owning error: {error:?}")
    };
    error
}

fn source_piece(
    row: &text::SpacingSlotFeedback,
    paragraph: &PreparedParagraph,
    range: text::Range,
) {
    assert_eq!(
        row.slot
            .pieces()
            .map(|piece| piece.owner())
            .collect::<Vec<_>>(),
        [text::SpacingSlotOwner::Source {
            origin: paragraph.origin(1),
            local: range,
            aggregate: range,
        }]
    );
}

fn deferred_refusal<S: LayoutScalar>() {
    for (terminal, cause) in [
        (
            text::ParagraphTerminalEdge::Unknown,
            text::SpacingDeferredCause::MissingTerminalBlockEdge,
        ),
        (
            text::ParagraphTerminalEdge::NonBlockBoundary,
            text::SpacingDeferredCause::NonBlockParagraphBoundary,
        ),
    ] {
        let paragraph = captured(" ", style(-10.0), terminal);
        let natural = compute::<S>(&paragraph, 15.0, TextAlignLast::Auto).unwrap();
        let retained = line(&natural);
        assert_eq!(
            retained.selected_line().request().context.alignment,
            text::Alignment::Start
        );
        assert_eq!(retained.feedback().advance, 0.0);
        let rows = retained.spacing_slots().collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        source_piece(row, &paragraph, text::Range::new(0, 1));
        assert_eq!(
            row.geometry,
            text::SpacingSlotGeometry::Independent {
                intrinsic_advance: 10.0
            }
        );
        assert_eq!(
            row.stretchability,
            text::SpacingStretchability::Deferred(cause)
        );
        assert!(!row.can_expand());
        assert_eq!(
            (row.authored_word_spacing, row.allocation, row.advance),
            (-10.0, 0.0, 0.0)
        );
        assert_eq!(raw(&retained), [(text::Range::new(0, 1), 3, 10.0)]);

        // Extra0 is a real admitted Justify operation, even with unavailable
        // terminal response. A positive need must consume the exact cause.
        let exact = compute::<S>(&paragraph, 0.0, TextAlignLast::Justify).unwrap();
        let exact = line(&exact);
        assert_eq!(
            exact.selected_line().request().context.alignment,
            text::Alignment::Justify
        );
        assert_eq!(exact.measurements().alignment().advance(), 0.0);
        assert_eq!(exact.spacing_slots().next().unwrap().allocation, 0.0);
        let error = compute::<S>(&paragraph, 15.0, TextAlignLast::Justify).unwrap_err();
        let expected = match cause {
            text::SpacingDeferredCause::MissingTerminalBlockEdge => {
                text::ProtocolError::MissingPhaseTwoContext(
                    text::PhaseTwoContext::TerminalBlockEdge,
                )
            }
            text::SpacingDeferredCause::NonBlockParagraphBoundary => {
                text::ProtocolError::UnsupportedContext {
                    field: text::ProtocolContextField::CssParagraphBoundary,
                }
            }
        };
        assert!(
            matches!(text_error(&error).detail(), Some(text::ErrorDetail::InvalidTextProtocol { reason }) if *reason == expected),
            "{error:?}"
        );
        assert_eq!(retained.feedback().advance, 0.0);
        assert_eq!(raw(&retained), [(text::Range::new(0, 1), 3, 10.0)]);
        let retry = compute::<S>(&paragraph, 15.0, TextAlignLast::Auto).unwrap();
        assert_eq!(line(&retry).feedback().advance, 0.0);
    }
    let paragraph = captured(" ", style(-10.0), text::ParagraphTerminalEdge::BlockEnd);
    let actual = compute::<S>(&paragraph, 15.0, TextAlignLast::Justify).unwrap();
    let actual = line(&actual);
    assert_eq!(actual.measurements().alignment().advance(), 15.0);
    assert_eq!(
        actual.spacing_slots().next().unwrap().stretchability,
        text::SpacingStretchability::Direct
    );
    assert_eq!(actual.spacing_slots().next().unwrap().allocation, 15.0);
}

#[test]
fn deferred_only_final_justification_preserves_exact_terminal_refusal_and_zero_control() {
    deferred_refusal::<f32>();
    deferred_refusal::<f64>();
}

fn direct_with_deferred<S: LayoutScalar>() {
    for (terminal, cause) in [
        (
            text::ParagraphTerminalEdge::Unknown,
            text::SpacingDeferredCause::MissingTerminalBlockEdge,
        ),
        (
            text::ParagraphTerminalEdge::NonBlockBoundary,
            text::SpacingDeferredCause::NonBlockParagraphBoundary,
        ),
    ] {
        let paragraph = captured(" X ", style(-10.0), terminal);
        let natural = compute::<S>(&paragraph, 20.0, TextAlignLast::Auto).unwrap();
        let natural = line(&natural);
        assert_eq!(natural.measurements().alignment().advance(), 10.0);
        assert_eq!(
            natural
                .spacing_slots()
                .map(|row| row.allocation)
                .collect::<Vec<_>>(),
            [0.0, 0.0]
        );
        let actual = compute::<S>(&paragraph, 20.0, TextAlignLast::Justify).unwrap();
        let actual = line(&actual);
        assert_eq!(
            actual.selected_line().request().context.alignment,
            text::Alignment::Justify
        );
        assert_eq!(actual.measurements().alignment().advance(), 20.0);
        assert_eq!(actual.feedback().advance, 20.0);
        let rows = actual.spacing_slots().collect::<Vec<_>>();
        assert_eq!(rows.len(), 2);
        for (row, range) in rows
            .iter()
            .zip([text::Range::new(0, 1), text::Range::new(2, 3)])
        {
            source_piece(row, &paragraph, range);
            assert_eq!(
                row.geometry,
                text::SpacingSlotGeometry::Independent {
                    intrinsic_advance: 10.0
                }
            );
            assert_eq!(row.authored_word_spacing, -10.0);
        }
        assert_eq!(rows[0].stretchability, text::SpacingStretchability::Direct);
        assert!(rows[0].can_expand());
        assert_eq!((rows[0].allocation, rows[0].advance), (10.0, 10.0));
        assert_eq!(
            rows[1].stretchability,
            text::SpacingStretchability::Deferred(cause)
        );
        assert!(!rows[1].can_expand());
        assert_eq!((rows[1].allocation, rows[1].advance), (0.0, 0.0));
        assert_eq!(
            raw(&actual),
            [
                (text::Range::new(0, 1), 3, 10.0),
                (text::Range::new(1, 2), 58, 10.0),
                (text::Range::new(2, 3), 3, 10.0)
            ]
        );
        assert_eq!(
            actual.edge_units().last().unwrap().disposition(),
            text::LineEdgeDisposition::UnconsumedTerminal {
                side: text::LineEdgeSide::End
            }
        );
        assert_eq!(natural.measurements().alignment().advance(), 10.0);
    }
}

#[test]
fn known_direct_opportunity_fills_while_genuine_deferred_tail_receives_complete_zero() {
    direct_with_deferred::<f32>();
    direct_with_deferred::<f64>();
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn table(bytes: &[u8], tag: &[u8; 4]) -> (usize, usize, usize) {
    let record = (0..usize::from(u16_at(bytes, 4)))
        .map(|index| 12 + index * 16)
        .find(|&record| &bytes[record..record + 4] == tag)
        .unwrap();
    (
        record,
        u32_at(bytes, record + 8) as usize,
        u32_at(bytes, record + 12) as usize,
    )
}
fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0u32, |sum, chunk| {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}
fn qualify_metrics(bytes: &[u8]) {
    let (_, head, _) = table(bytes, b"head");
    let (_, hmtx, _) = table(bytes, b"hmtx");
    assert_eq!(u16_at(bytes, head + 18), 1000);
    for glyph in [3, 35, 58, 153] {
        assert_eq!(u16_at(bytes, hmtx + glyph * 4), 1000);
    }
}

fn shared_separator_font() -> Vec<u8> {
    // Reuse the owning Layout fixture's real ccmp A+A -> X lookup. Change its
    // actual second component to SPACE3, giving A+SPACE -> X; cmap is untouched.
    let mut bytes = super::canonical::fonts::ligature_ahem();
    let (record, gsub, length) = table(&bytes, b"GSUB");
    let list = gsub + usize::from(u16_at(&bytes, gsub + 8));
    assert_eq!(u16_at(&bytes, list), 1);
    let lookup = list + usize::from(u16_at(&bytes, list + 2));
    assert_eq!(u16_at(&bytes, lookup), 4);
    let subtable = lookup + usize::from(u16_at(&bytes, lookup + 6));
    assert_eq!(u16_at(&bytes, subtable), 1);
    let set = subtable + usize::from(u16_at(&bytes, subtable + 6));
    let ligature = set + usize::from(u16_at(&bytes, set + 2));
    assert_eq!(u16_at(&bytes, ligature), 58);
    assert_eq!(u16_at(&bytes, ligature + 2), 2);
    assert_eq!(u16_at(&bytes, ligature + 4), 35);
    bytes[ligature + 4..ligature + 6].copy_from_slice(&3u16.to_be_bytes());
    let sum = checksum(&bytes[gsub..gsub + length]);
    bytes[record + 4..record + 8].copy_from_slice(&sum.to_be_bytes());
    let (_, head, _) = table(&bytes, b"head");
    bytes[head + 8..head + 12].fill(0);
    let adjustment = 0xb1b0_afbau32.wrapping_sub(checksum(&bytes));
    bytes[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xb1b0_afba);
    qualify_metrics(&bytes);
    bytes
}

fn shared_refusal<S: LayoutScalar>() {
    let font = shared_separator_font();
    let mut own = style(0.0);
    own.font = own.font.feature("\"ccmp\" 1");
    let paragraph = PreparedParagraph::with_registered_font(
        vec![ParagraphPart::text(1, Source::new("A A"), own.clone())],
        own,
        ParagraphBaseDirection::LeftToRight,
        &font,
    );
    let natural = compute::<S>(&paragraph, 30.0, TextAlignLast::Auto).unwrap();
    let retained = line(&natural);
    assert!(
        retained
            .run_views()
            .all(|run| run.face().bytes() == font.as_slice())
    );
    assert_eq!(
        raw(&retained),
        [
            (text::Range::new(0, 2), 58, 10.0),
            (text::Range::new(2, 3), 35, 10.0)
        ]
    );
    assert_eq!(retained.measurements().alignment().advance(), 20.0);
    let rows = retained.spacing_slots().collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    source_piece(&rows[0], &paragraph, text::Range::new(1, 2));
    assert_eq!(
        rows[0].geometry,
        text::SpacingSlotGeometry::SharedProviderGroup
    );
    assert_eq!(rows[0].allocation, 0.0);
    assert!(!rows[0].can_expand());
    let exact = compute::<S>(&paragraph, 20.0, TextAlignLast::Justify).unwrap();
    assert_eq!(line(&exact).measurements().alignment().advance(), 20.0);
    assert_eq!(
        line(&exact).selected_line().request().context.alignment,
        text::Alignment::Justify
    );
    let error = compute::<S>(&paragraph, 30.0, TextAlignLast::Justify).unwrap_err();
    let error = text_error(&error);
    assert_eq!(error.code, text::ErrorCode::LayoutFailed);
    let Some(text::ErrorDetail::UnsupportedSpacing {
        stage,
        operation,
        pieces,
        provider_atoms,
        ..
    }) = error.detail()
    else {
        panic!("shared provider geometry requires its precise refusal: {error:?}")
    };
    assert_eq!(*stage, text::SpacingStage::SelectedShaping);
    assert_eq!(*operation, text::SpacingOperation::InterWordAllocation);
    assert_eq!(
        pieces.as_ref(),
        [text::SpacingPiece::Source {
            origin: 0,
            local: text::Range::new(1, 2),
            aggregate: text::Range::new(1, 2)
        }]
    );
    assert_eq!(provider_atoms.len(), 1);
    assert_eq!(provider_atoms[0], 0..2);
    assert_eq!(retained.feedback().advance, 20.0);
    assert_eq!(
        raw(&retained),
        raw(&line(
            &compute::<S>(&paragraph, 30.0, TextAlignLast::Auto).unwrap()
        ))
    );
}

#[test]
fn actual_shared_gsub_group_stays_natural_but_final_justify_propagates_material_refusal() {
    shared_refusal::<f32>();
    shared_refusal::<f64>();
}

#[test]
fn direct_subnormal_allocation_and_equal_share_underflow_refuse_instead_of_falling_back() {
    qualify_metrics(AHEM);
    let tiny = f64::from_bits(1);
    assert!(tiny > 0.0);
    assert_eq!(tiny * 0.5, 0.0);
    for (source, tracking, slots, cell) in [(" A", -20.0, 1, -10.0), (" \u{a0}A", -15.0, 2, -5.0)] {
        // Real internal gaps cancel the complete line: [10-20,10] or
        // [10-15,10-15,10]. Every glyph remains10; the final A has no edge gap.
        // Retained SPACE/NBSP own one/two Direct slots independently of A.
        let mut own = style(0.0);
        own.letter_spacing = tracking;
        // A Soft all-SPACE prefix hangs and falls back before ParagraphEnd.
        // No wrapping isolates the final Direct allocation qualified here.
        own.wrap = Wrap::None;
        let paragraph = captured(source, own, text::ParagraphTerminalEdge::BlockEnd);
        let natural = compute::<f64>(&paragraph, tiny, TextAlignLast::Auto).unwrap();
        let retained = line(&natural);
        assert_eq!(retained.measurements().alignment().advance(), 0.0);
        let rows = retained.spacing_slots().collect::<Vec<_>>();
        assert_eq!(rows.len(), slots);
        for (index, row) in rows.iter().enumerate() {
            source_piece(row, &paragraph, text::Range::new(index, index + 1));
            assert_eq!(
                row.geometry,
                text::SpacingSlotGeometry::Independent {
                    intrinsic_advance: 10.0
                }
            );
            assert_eq!(row.stretchability, text::SpacingStretchability::Direct);
            assert!(row.can_expand());
            assert_eq!(
                (row.authored_word_spacing, row.allocation, row.advance),
                (0.0, 0.0, cell)
            );
        }
        assert!(
            raw(&retained)
                .iter()
                .all(|(_, _, advance)| *advance == 10.0)
        );
        let error = compute::<f64>(&paragraph, tiny, TextAlignLast::Justify).unwrap_err();
        if slots == 1 {
            let error = text_error(&error);
            assert_eq!(error.code, text::ErrorCode::InvalidTextNumeric);
            assert!(
                matches!(error.detail(), Some(text::ErrorDetail::InvalidTextNumeric {
                field: "word spacing half", phase: text::TextNumericPhase::Finalization,
                category: text::TextNumericError::NonZeroLost, value,
            }) if *value == tiny),
                "{error:?}"
            );
        } else {
            assert!(
                matches!(error.kind(), LayoutErrorKindOf::TextNumeric {
                value, reason: LayoutTextNumericError::NonZeroLost,
            } if *value == tiny),
                "{error:?}"
            );
        }
        assert_eq!(retained.measurements().alignment().advance(), 0.0);
        assert_eq!(
            line(&compute::<f64>(&paragraph, tiny, TextAlignLast::Auto).unwrap())
                .measurements()
                .alignment()
                .advance(),
            0.0
        );
    }
}
