//! CSS Text4 §9.3 keeps line-box geometry independent of body UBA. These
//! functional tests supply the new resolved fact explicitly, with real Ahem.

#[path = "line_box_direction/alignment_frame.rs"]
mod alignment_frame;
#[path = "line_box_direction/post_expansion_overflow.rs"]
mod post_expansion_overflow;

use super::*;
use crate::*;

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");
const BOXES: [LineBoxDirection; 2] = [LineBoxDirection::LeftToRight, LineBoxDirection::RightToLeft];
const BODIES: [ParagraphBaseDirection; 2] = [
    ParagraphBaseDirection::LeftToRight,
    ParagraphBaseDirection::RightToLeft,
];

fn capture(
    value: &str,
    body: ParagraphBaseDirection,
    whitespace: WhiteSpace,
    tabs: bool,
) -> PreparedText {
    let source = Source::identified(value, Id::from_u64(71001), 9);
    let len = source.text().len();
    let base = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: whitespace,
        overflow_wrap: OverflowWrap::Anywhere,
        tab_size: TabSize::length(40.0).unwrap(),
        ..Style::default()
    };
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: base.clone(),
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            base_direction: body,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: ParagraphTerminalEdge::BlockEnd,
            tab_reference: tabs.then(|| {
                ParagraphTabReference::new(base, ParagraphBaseDirection::LeftToRight).unwrap()
            }),
            origins: vec![ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, len),
                aggregate: Range::new(0, len),
                kind: ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    system
        .prepare(
            &analyzed,
            if tabs {
                PreparationOptions::try_new(1.0, TabPreparation::CssStops).unwrap()
            } else {
                PreparationOptions::default()
            },
        )
        .unwrap()
}

fn context(
    p: &PreparedText,
    direction: LineBoxDirection,
    width: Option<f64>,
    indent: f64,
    alignment: Alignment,
) -> LineSelectionRequest {
    let mut r = request(p);
    r.context.base_direction = if p.analyzed_source().direction() == Direction::RightToLeft {
        ParagraphBaseDirection::RightToLeft
    } else {
        ParagraphBaseDirection::LeftToRight
    };
    r.context.line_box_direction = direction;
    r.context.available_inline = width;
    r.context.indent = indent;
    r.context.alignment = alignment;
    r.context.spacing = SpacingPolicy::None;
    r
}

fn finish(p: &PreparedText, request: LineSelectionRequest) -> FinalizedLine {
    p.select(request)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

fn glyphs(line: &FinalizedLine) -> Vec<(Range, u32, f32, f64)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.font_size(), 10.0);
            run.glyphs()
                .map(|g| {
                    (
                        g.range(),
                        g.id(),
                        g.advance(),
                        run.origin().x + g.pen() + f64::from(g.x()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn xs(line: &FinalizedLine) -> Vec<f64> {
    glyphs(line).into_iter().map(|g| g.3).collect()
}

#[test]
fn same_rtl_body_has_independent_line_box_geometry_and_unchanged_order_source_and_carets() {
    let p = capture("XA", BODIES[1], WhiteSpace::Preserve, false);
    assert_eq!(p.analyzed_source().direction(), Direction::RightToLeft);
    let ltr = finish(&p, context(&p, BOXES[0], Some(40.0), 5.0, Alignment::Start));
    let rtl = finish(&p, context(&p, BOXES[1], Some(40.0), 5.0, Alignment::Start));
    assert_eq!(
        glyphs(&ltr),
        [
            (Range::new(0, 1), 58, 10.0, 5.0),
            (Range::new(1, 2), 35, 10.0, 15.0)
        ]
    );
    assert_eq!(
        glyphs(&rtl),
        [
            (Range::new(0, 1), 58, 10.0, 15.0),
            (Range::new(1, 2), 35, 10.0, 25.0)
        ]
    );
    assert_eq!(ltr.feedback().advance, 20.0);
    assert_eq!(rtl.feedback().advance, 20.0);
    assert_eq!(ltr.prepared_text(), rtl.prepared_text());
    assert_eq!(
        ltr.selected_line().request().context.base_direction,
        BODIES[1]
    );
    assert_eq!(
        rtl.selected_line().request().context.base_direction,
        BODIES[1]
    );
    let levels = |line: &FinalizedLine| {
        line.inner
            .data
            .clusters
            .iter()
            .map(|c| (c.source, c.level))
            .collect::<Vec<_>>()
    };
    assert_eq!(levels(&ltr), levels(&rtl));
    assert_eq!(
        ltr.mapping_views()
            .map(|m| (m.source_range(), m.presentation_range()))
            .collect::<Vec<_>>(),
        rtl.mapping_views()
            .map(|m| (m.source_range(), m.presentation_range()))
            .collect::<Vec<_>>()
    );
    let left = place(p.clone(), ltr, 0.0, 8.0).unwrap();
    let right = place(p.clone(), rtl, 0.0, 8.0).unwrap();
    assert_eq!(left.source(), right.source());
    assert_eq!(left.source().text().code_units(), [88, 65]);
    for index in 0..=2 {
        for affinity in [Affinity::Before, Affinity::After] {
            let a = left.cursor(Cursor::new(index, affinity)).unwrap().rect();
            let b = right.cursor(Cursor::new(index, affinity)).unwrap().rect();
            assert_eq!(b.origin.x - a.origin.x, 10.0);
            assert_eq!(a.origin.y, b.origin.y);
            assert_eq!(a.size, b.size);
        }
    }
    let mut contradiction = context(&p, BOXES[0], Some(40.0), 5.0, Alignment::Start);
    contradiction.context.base_direction = BODIES[0];
    assert_eq!(
        reason(&p.select(contradiction).unwrap_err()),
        &ProtocolError::MismatchedContext {
            field: ProtocolContextField::BaseDirection
        }
    );
}

#[test]
fn logical_physical_center_justify_fallback_and_overflow_use_the_frame_in_both_body_bases() {
    for body in BODIES {
        let p = capture("XA", body, WhiteSpace::Preserve, false);
        for (alignment, expected) in [
            (Alignment::Start, [5.0, 15.0]),
            (Alignment::End, [20.0, 0.0]),
            (Alignment::Left, [5.0, 0.0]),
            (Alignment::Right, [20.0, 15.0]),
            (Alignment::Center, [12.5, 7.5]),
            (Alignment::Justify, [5.0, 15.0]),
        ] {
            for (i, direction) in BOXES.into_iter().enumerate() {
                let line = finish(&p, context(&p, direction, Some(40.0), 5.0, alignment));
                assert_eq!(
                    xs(&line),
                    [expected[i], expected[i] + 10.0],
                    "{body:?}/{direction:?}/{alignment:?}"
                );
                assert_eq!(line.feedback().advance, 20.0);
            }
        }
        for alignment in [
            Alignment::Start,
            Alignment::End,
            Alignment::Left,
            Alignment::Right,
            Alignment::Center,
        ] {
            assert_eq!(
                xs(&finish(
                    &p,
                    context(&p, BOXES[0], Some(15.0), 5.0, alignment)
                )),
                [5.0, 15.0]
            );
            assert_eq!(
                xs(&finish(
                    &p,
                    context(&p, BOXES[1], Some(15.0), 5.0, alignment)
                )),
                [-10.0, 0.0]
            );
        }
    }
}

#[test]
fn signed_indentation_and_unconstrained_metric_frames_keep_the_resolved_side() {
    for body in BODIES {
        let p = capture("XA", body, WhiteSpace::Preserve, false);
        for (width, indent, expected) in [
            (Some(40.0), -5.0, [-5.0, 25.0]),
            (Some(40.0), 0.0, [0.0, 20.0]),
            (None, 5.0, [5.0, 0.0]),
            (None, -5.0, [-5.0, 5.0]),
        ] {
            for (i, direction) in BOXES.into_iter().enumerate() {
                let line = finish(&p, context(&p, direction, width, indent, Alignment::Start));
                assert_eq!(xs(&line), [expected[i], expected[i] + 10.0]);
                assert_eq!(
                    line.measurements().min_content_contribution().advance(),
                    20.0
                );
                assert_eq!(
                    line.measurements().max_content_contribution().advance(),
                    20.0
                );
            }
        }
    }
}

#[test]
fn opposed_soft_preserved_space_keeps_its_physical_body_hanging_owner() {
    let p = capture("A A", BODIES[1], WhiteSpace::Preserve, false);
    let offer = p
        .opportunity_views()
        .find(|offer| {
            offer.kind() == PreparedBreakKind::Allowed
                && offer.boundary().view().source_after().get() == 2
        })
        .unwrap()
        .handle();
    let mut r = context(&p, BOXES[0], Some(40.0), 5.0, Alignment::Start);
    r.end = offer.view().boundary();
    r.participants.truncate(r.end.view().participant_offset());
    r.termination = PreparedTermination::Ordinary(offer);
    r.context.end = LineEnd::Soft;
    let line = finish(&p, r);
    let glyphs = glyphs(&line);
    assert_eq!(
        glyphs.iter().find(|g| g.0 == Range::new(0, 1)).unwrap().3,
        5.0
    );
    assert_eq!(
        glyphs.iter().find(|g| g.0 == Range::new(1, 2)).unwrap().3,
        -5.0
    );
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(line.measurements().alignment().advance(), 10.0);
    assert_eq!(
        (line.feedback().hanging_start, line.feedback().hanging_end),
        (10.0, 0.0)
    );
    let edge = line
        .edge_units()
        .find(|edge| {
            edge.source_pieces()
                .any(|source| source == Range::new(1, 2))
        })
        .unwrap();
    assert_eq!(
        edge.disposition(),
        LineEdgeDisposition::Unconditional {
            side: LineEdgeSide::Start,
            terminal: true
        }
    );
    assert_eq!(
        p.analyzed_source().source().text().code_units(),
        [65, 32, 65]
    );
}

#[test]
fn conditional_preserved_space_eligibility_and_exclusion_do_not_flip_with_the_frame() {
    let p = capture("A ", BODIES[1], WhiteSpace::Preserve, false);
    for (width, eligible, excluded, a_positions) in [
        (40.0, false, 0.0, [15.0, 25.0]),
        (15.0, true, 10.0, [5.0, 0.0]),
    ] {
        for (i, direction) in BOXES.into_iter().enumerate() {
            let line = finish(
                &p,
                context(&p, direction, Some(width), 5.0, Alignment::Start),
            );
            let edge = line
                .edge_units()
                .find(|edge| {
                    edge.source_pieces()
                        .any(|source| source == Range::new(1, 2))
                })
                .unwrap();
            assert_eq!(
                edge.disposition(),
                LineEdgeDisposition::Conditional {
                    side: LineEdgeSide::Start,
                    eligible
                }
            );
            assert_eq!(edge.excluded_contribution(), excluded);
            assert_eq!(
                (line.feedback().hanging_start, line.feedback().hanging_end),
                (excluded, 0.0)
            );
            assert_eq!(line.feedback().advance, 20.0);
            assert_eq!(line.measurements().alignment().advance(), 20.0 - excluded);
            assert_eq!(
                glyphs(&line)
                    .iter()
                    .find(|g| g.0 == Range::new(0, 1))
                    .unwrap()
                    .3,
                a_positions[i]
            );
        }
    }
}

#[test]
fn css_tab_reference_and_body_traversal_are_independent_of_line_box_alignment() {
    let p = capture("A\tX", BODIES[0], WhiteSpace::Preserve, true);
    for (direction, expected) in [(BOXES[0], [0.0, 40.0]), (BOXES[1], [30.0, 70.0])] {
        let mut r = context(&p, direction, Some(80.0), 0.0, Alignment::Start);
        r.context.tabs = TabContext::CssStops { origin: Some(0.0) };
        let line = finish(&p, r);
        assert_eq!(line.feedback().advance, 50.0);
        assert_eq!(
            glyphs(&line).iter().map(|g| (g.1, g.2)).collect::<Vec<_>>(),
            [(35, 10.0), (58, 10.0)]
        );
        assert_eq!(xs(&line), expected);
        assert_eq!(
            line.selected_line().request().context.tabs,
            TabContext::CssStops { origin: Some(0.0) }
        );
    }
}

#[test]
fn conditional_prefix_can_resolve_opposite_to_its_failed_ordinary_frame_and_keeps_receipt_identity()
{
    let p = capture("123A", BODIES[0], WhiteSpace::Preserve, false);
    let start = p.boundary_views().next().unwrap().handle();
    let ceiling = p
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let ordinary = finish(&p, context(&p, BOXES[0], Some(25.0), 0.0, Alignment::Start));
    let other_frame = finish(&p, context(&p, BOXES[1], Some(25.0), 0.0, Alignment::Start));
    assert_eq!(ordinary.feedback().advance, 40.0);
    let search = p
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&ordinary),
        )
        .unwrap();
    let other = p
        .conditional_search(
            &start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            Some(&other_frame),
        )
        .unwrap();
    assert_ne!(
        search, other,
        "the actual receipt identity includes line-box direction"
    );
    let batch = search.begin(&[]).unwrap();
    let cut = batch
        .opportunities()
        .iter()
        .find(|cut| cut.boundary().view().source_after().get() == 2)
        .unwrap();
    let mut r = context(&p, BOXES[1], Some(25.0), 0.0, Alignment::Start);
    r.start = cut.start();
    r.end = cut.boundary();
    r.participants.truncate(r.end.view().participant_offset());
    r.termination = PreparedTermination::Conditional(cut.clone());
    r.context.end = LineEnd::Soft;
    let first = finish(&p, r);
    assert_eq!(xs(&first), [5.0, 15.0]);
    assert_eq!(
        first.selected_line().request().context.base_direction,
        BODIES[0]
    );
    let mut r = context(&p, BOXES[0], Some(25.0), 0.0, Alignment::Start);
    r.start = cut.boundary();
    r.participants = p
        .participants()
        .skip(r.start.view().participant_offset())
        .collect();
    r.context.start = LineStart::Continuation;
    let second = finish(&p, r);
    assert_eq!(xs(&second), [0.0, 10.0]);
    assert_eq!(
        second.selected_line().request().context.base_direction,
        BODIES[0]
    );
}

#[test]
fn raw_standalone_explicitly_retains_its_own_resolved_profile() {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    for (text, body, frame) in [
        ("XA", BODIES[0], BOXES[0]),
        ("\u{05d0}\u{05d1}", BODIES[1], BOXES[1]),
    ] {
        let layout = system
            .layout(
                Source::new(text),
                Style {
                    font: Font::new().family("Ahem"),
                    size: 10.0,
                    ..Style::default()
                },
                Options {
                    width: Some(40.0),
                    indent: Indent {
                        amount: 5.0,
                        ..Indent::default()
                    },
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        let context = &layout.inner.lines[0].finalized.selected.request.context;
        assert_eq!(context.base_direction, body);
        assert_eq!(context.line_box_direction, frame);
        assert_eq!(
            layout.source().text().code_units(),
            text.encode_utf16().collect::<Vec<_>>()
        );
        // The RTL control asserts raw direction custody only. Ahem has no
        // Hebrew cmap; no Hebrew glyph-coverage/ink/numeric claim is made.
    }
}
