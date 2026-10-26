//! Real transformed glyphs and distinct original/Phase-I/presentation custody.
use super::*;
use crate::{Id, LineHeight, Range, TextTransform};

fn kana_style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        text_transform: TextTransform::Set(
            crate::TextTransformSet::none().with_full_size_kana(true),
        ),
        ..Style::default()
    }
}
fn input(source: Source, style: Style) -> ParagraphInput {
    let len = source.text().len();
    ParagraphInput {
        source: source.clone(),
        base_style: style,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        replacements: vec![],
        boundary_policies: vec![],
        anchor_order: vec![],
    }
}
fn captured(source: Source, style: Style) -> PreparedText {
    capture_input(input(source, style), crate::shaping_test_fonts::kana_ahem())
}
fn capture_input(mut input: ParagraphInput, bytes: Vec<u8>) -> PreparedText {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    input.boundary_policies = input
        .connections
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == crate::ParagraphConnectionKind::Continuous)
        .map(|(index, _)| crate::ParagraphBoundaryPolicy {
            definition: crate::ParagraphBoundaryDefinition::Connection(index),
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect();
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    system
        .prepare(
            &crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}
fn whole(p: &PreparedText) -> FinalizedLine {
    let mut r = request(p);
    r.atomics = r
        .participants
        .iter()
        .filter_map(|part| match part {
            PreparedParticipant::Atomic(occurrence) => Some(AtomicAllocation {
                occurrence: occurrence.clone(),
                allocation: AtomicLineAllocation::OutOfFlow,
            }),
            _ => None,
        })
        .collect();
    p.select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn glyphs(line: &FinalizedLine) -> Vec<(u32, f32, f64, Range)> {
    line.run_views()
        .flat_map(|r| {
            r.glyphs()
                .map(|g| {
                    (
                        g.id(),
                        g.advance(),
                        r.origin().x + g.pen() + f64::from(g.x()),
                        g.range(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
#[test]
fn transformed_native_glyphs_metrics_mapping_and_partial_surrogate_ranges_share_real_result() {
    let source = Source::identified("A\u{1b132}C", Id::from_u64(6302), 12);
    let p = captured(source.clone(), kana_style());
    assert_eq!(p.analyzed_source().source(), &source);
    assert_eq!(
        p.analyzed_source()
            .processed_source()
            .processed_text()
            .len(),
        4
    );
    assert_eq!(
        p.boundary_views()
            .last()
            .unwrap()
            .processed_position()
            .get(),
        4
    );
    assert_eq!(
        p.unit_views()
            .map(|u| {
                let r = u.processed_range();
                (r.start().get(), r.end().get())
            })
            .collect::<Vec<_>>(),
        [(0, 1), (1, 3), (3, 4)]
    );
    let line = p
        .select(request(&p))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 25.0);
    assert_eq!(
        glyphs(&line),
        [
            (35, 10.0, 0.0, Range::new(0, 1)),
            (36, 5.0, 10.0, Range::new(1, 3)),
            (37, 10.0, 15.0, Range::new(3, 4))
        ]
    );
    let mapping: Vec<_> = line
        .mapping_views()
        .map(|m| {
            let a = m.processed_range();
            let b = m.presentation_range();
            (
                m.source_range(),
                a.start().get(),
                a.end().get(),
                b.start(),
                b.end(),
            )
        })
        .collect();
    assert_eq!(
        mapping,
        [
            (Range::new(0, 1), 0, 1, 0, 1),
            (Range::new(1, 3), 1, 3, 1, 2),
            (Range::new(3, 4), 3, 4, 2, 3)
        ]
    );
    let origin = p.origin_views().next().unwrap().handle();
    let layout = place(p, line, 0.0, 8.0).unwrap();
    let full = layout
        .source_range_fragments(&origin, Range::new(1, 3))
        .unwrap();
    let full_unit = full.units().next().unwrap();
    for range in [Range::new(1, 3), Range::new(1, 2), Range::new(2, 3)] {
        let fragments = layout.source_range_fragments(&origin, range).unwrap();
        let units: Vec<_> = fragments.units().collect();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].pieces().next().unwrap().rect().size.width, 5.0);
        assert!(units[0].same_unit(&full_unit));
    }
    assert_eq!(layout.source(), &source);
    for (affinity, endpoint, x) in [
        (crate::Affinity::Before, 1, 10.0),
        (crate::Affinity::After, 3, 15.0),
    ] {
        let interior = layout.cursor(crate::Cursor::new(2, affinity)).unwrap();
        assert_eq!(
            interior,
            layout
                .cursor(crate::Cursor::new(endpoint, affinity))
                .unwrap()
        );
        assert_eq!(interior.rect().origin.x, x);
    }
    let positions: Vec<_> = layout.positions().collect();
    assert!(
        positions
            .iter()
            .any(|p| p.source_cursor() == crate::Cursor::new(1, crate::Affinity::After))
    );
    assert!(
        positions
            .iter()
            .any(|p| p.source_cursor() == crate::Cursor::new(3, crate::Affinity::Before))
    );
}

#[test]
fn actual_enabled_and_neutral_styles_cross_origins_and_transparent_occurrences() {
    for enabled in [false, true] {
        let mut source = Source::new("A\u{1b132}C");
        source.span(
            Range::new(1, 3),
            Style {
                text_transform: TextTransform::Set(
                    crate::TextTransformSet::none().with_full_size_kana(enabled),
                ),
                ..kana_style()
            },
        );
        source.inline_box(crate::InlineBox::new(
            Id::from_u64(6303),
            crate::InlineBoxKind::OutOfFlow,
            3,
            crate::Size::new(0.0, 0.0),
        ));
        let mut i = input(source.clone(), kana_style());
        i.origins = [(0, 1), (1, 3), (3, 3), (3, 4)]
            .into_iter()
            .map(|(a, b)| paragraph::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(a, b),
                aggregate: Range::new(a, b),
                kind: paragraph::ParagraphOriginKind::Authored,
            })
            .collect();
        i.connections = [1, 3, 3]
            .into_iter()
            .map(|at| crate::ParagraphConnection {
                at,
                kind: crate::ParagraphConnectionKind::Continuous,
            })
            .collect();
        i.anchor_order = vec![
            paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::EmptyOrigin(2),
                affinity: crate::Affinity::After,
            },
            paragraph::ParagraphAnchor {
                item: paragraph::ParagraphAnchorItem::Atomic(0),
                affinity: crate::Affinity::After,
            },
        ];
        let p = capture_input(i, crate::shaping_test_fonts::kana_ahem());
        assert_eq!(
            p.unit_views()
                .filter(|u| u.source_range() == Range::new(3, 3))
                .count(),
            1
        );
        let line = whole(&p);
        assert_eq!(line.feedback().advance, if enabled { 25.0 } else { 30.0 });
        assert_eq!(
            glyphs(&line).iter().map(|g| g.0).collect::<Vec<_>>(),
            if enabled {
                vec![35, 36, 37]
            } else {
                vec![35, 82, 37]
            }
        );
        let middle = p
            .unit_views()
            .find(|u| u.source_range() == Range::new(1, 3))
            .unwrap();
        assert_eq!(
            middle
                .origins()
                .map(|o| o.origin().view().local_range())
                .collect::<Vec<_>>(),
            [Range::new(1, 3)]
        );
        assert_eq!(p.analyzed_source().source(), &source);
    }
}

#[test]
fn transformed_base_and_cross_origin_mark_keep_one_real_gpos_unit_and_component_height() {
    let bytes = crate::shaping_test_fonts::kana_mark_ahem();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    use skrifa::MetadataProvider;
    assert_eq!(font.charmap().map(0x3053_u32).unwrap().to_u32(), 35);
    assert_eq!(font.charmap().map(0x301_u32).unwrap().to_u32(), 100);
    let mut source = Source::new("\u{1b132}\u{301}");
    source.span(
        Range::new(2, 3),
        Style {
            line_height: LineHeight::Absolute(20.0),
            text_transform: TextTransform::none(),
            ..kana_style()
        },
    );
    let mut i = input(source.clone(), kana_style());
    i.origins = [(0, 2), (2, 3)]
        .into_iter()
        .map(|(a, b)| paragraph::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(a, b),
            aggregate: Range::new(a, b),
            kind: paragraph::ParagraphOriginKind::Authored,
        })
        .collect();
    i.connections = vec![crate::ParagraphConnection {
        at: 2,
        kind: crate::ParagraphConnectionKind::Continuous,
    }];
    let p = capture_input(i, bytes);
    assert_eq!(p.inner.analyzed.analysis.graphemes.len(), 1);
    assert_eq!(p.inner.analyzed.analysis.graphemes[0], 0..2);
    let line = whole(&p);
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(line.feedback().line_height, 20.0);
    let g: Vec<_> = line
        .run_views()
        .flat_map(|r| {
            r.glyphs()
                .map(|g| {
                    (
                        g.id(),
                        g.advance(),
                        g.pen() + f64::from(g.x()),
                        g.y(),
                        g.range(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        g,
        [
            (35, 10.0, 0.0, 0.0, Range::new(0, 3)),
            (100, 0.0, 3.0, -4.0, Range::new(0, 3))
        ]
    );
    assert_eq!(line.inner.data.font_runs.len(), 1);
    let first = p.origin_views().next().unwrap().handle();
    let last = p.origin_views().last().unwrap().handle();
    let layout = place(p, line, 0.0, 18.0).unwrap();
    let a = layout
        .source_range_fragments(&first, Range::new(0, 2))
        .unwrap();
    let b = layout
        .source_range_fragments(&last, Range::new(2, 3))
        .unwrap();
    assert!(
        a.units()
            .next()
            .unwrap()
            .same_unit(&b.units().next().unwrap())
    );
    assert_eq!(layout.source(), &source);
}

#[test]
fn standalone_wrap_and_warm_cache_use_transformed_metrics_without_mutating_source() {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::kana_ahem())
        .unwrap();
    let source = Source::identified("A\u{1b132}C", Id::from_u64(6304), 8);
    for (enabled, width, glyph) in [
        (false, 30.0, 82),
        (true, 25.0, 36),
        (false, 30.0, 82),
        (true, 25.0, 36),
    ] {
        let st = Style {
            text_transform: TextTransform::Set(
                crate::TextTransformSet::none().with_full_size_kana(enabled),
            ),
            ..kana_style()
        };
        let options = crate::Options {
            quantize: false,
            alignment: crate::Alignment::Left,
            ..crate::Options::default()
        };
        let l = system.layout(source.clone(), st.clone(), options).unwrap();
        assert_eq!(l.source(), &source);
        assert_eq!(l.lines()[0].bounds().size.width, width);
        assert_eq!(
            l.glyph_runs()
                .iter()
                .flat_map(|r| r.glyphs().iter().map(|g| g.id()))
                .collect::<Vec<_>>(),
            [35, glyph, 37]
        );
        let mut fresh = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        fresh
            .register_fonts(crate::shaping_test_fonts::kana_ahem())
            .unwrap();
        assert_eq!(
            fresh.layout(source.clone(), st, options).unwrap().lines()[0].bounds(),
            l.lines()[0].bounds()
        );
    }
    let wrapped = system
        .layout(
            source.clone(),
            Style {
                word_break: crate::WordBreak::BreakAll,
                ..kana_style()
            },
            crate::Options {
                width: Some(15.0),
                quantize: false,
                ..crate::Options::default()
            },
        )
        .unwrap();
    assert_eq!(
        wrapped
            .lines()
            .iter()
            .map(|l| l.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 3), Range::new(3, 4)]
    );
    assert_eq!(
        wrapped
            .lines()
            .iter()
            .map(|l| l.bounds().size.width)
            .collect::<Vec<_>>(),
        [15.0, 15.0]
    );
    // Bounds retain the finite alignment frame; actual glyph advances remain15/10.
    assert_eq!(
        wrapped
            .inner
            .lines
            .iter()
            .map(|l| l
                .finalized
                .data
                .clusters
                .iter()
                .map(|c| c.advance)
                .sum::<f64>())
            .collect::<Vec<_>>(),
        [15.0, 10.0]
    );
    assert_eq!(wrapped.source(), &source);
}

#[test]
fn kana_components_retain_one_provider_ligature_across_transform_and_height_styles() {
    let bytes =
        crate::shaping_test_fonts::range_ligature_ahem(&[35, 35], &[(0x1b132, 82), (0x3053, 35)]);
    let mut source = Source::new("\u{1b132}\u{3053}");
    source.span(
        Range::new(2, 3),
        Style {
            text_transform: TextTransform::none(),
            line_height: LineHeight::Absolute(20.0),
            ..kana_style()
        },
    );
    let p = capture_input(input(source.clone(), kana_style()), bytes);
    assert_eq!(p.inner.analyzed.projection.text, "ここ");
    assert_eq!(p.inner.analyzed.analysis.graphemes, [0..1, 1..2]);
    let line = whole(&p);
    assert_eq!(glyphs(&line), [(58, 10.0, 0.0, Range::new(0, 3))]);
    assert_eq!(line.feedback().line_height, 20.0);
    assert_eq!(line.inner.data.font_runs.len(), 1);
    assert_eq!(line.inner.data.clusters.len(), 2);
    assert_eq!(p.analyzed_source().source(), &source);
}

fn part(
    p: &PreparedText,
    start: PreparedBoundary,
    offer: PreparedOpportunity,
    incoming: Option<HyphenationContinuation>,
    take: bool,
) -> FinalizedLine {
    let mut r = request(p);
    r.start = start;
    r.end = offer.view().boundary();
    r.termination = PreparedTermination::Ordinary(offer.clone());
    r.alternative = take.then(|| offer.view().alternatives().next().unwrap().handle());
    r.incoming = incoming;
    let a = r.start.view().participant_offset();
    let b = r.end.view().participant_offset();
    r.participants = p.participants().skip(a).take(b - a).collect();
    r.context.start = if a == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    r.context.end = if offer.view().kind() == PreparedBreakKind::ParagraphEnd {
        LineEnd::Paragraph
    } else {
        LineEnd::Soft
    };
    r.context.available_inline = None;
    p.select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
#[test]
fn transformed_receiving_body_and_exact_before_after_both_indications_share_custody() {
    use crate::{
        HyphenateCharacter, HyphenationPlacement as P, Hyphens, NativeHyphenationConvention, Text,
    };
    for (placement, first_advance, last_advance) in [
        (P::LogicalBefore, 30.0, 45.0),
        (P::LogicalAfter, 20.0, 55.0),
        (P::BothSides, 30.0, 55.0),
    ] {
        let source = Source::new("AA\u{ad}CCCC\u{1b132}");
        let st = Style {
            hyphens: Hyphens::Manual,
            locale: Some("und-Latn".into()),
            hyphenate_character: HyphenateCharacter::Auto,
            native_hyphenation_convention: Some(
                NativeHyphenationConvention::language_script(
                    "und-Latn",
                    "Latn",
                    Text::from("C"),
                    placement,
                )
                .unwrap(),
            ),
            ..kana_style()
        };
        let p = captured(source.clone(), st);
        let cut = p
            .hyphenation_cut_views()
            .find(|c| c.boundary().view().source_before().get() == 3)
            .unwrap();
        assert_eq!(
            cut.counts(),
            Some(crate::HyphenationCounts {
                total: 6,
                before: 2,
                after: 4
            })
        );
        let offer = cut.opportunity().unwrap();
        let first = part(
            &p,
            p.boundary_views().next().unwrap().handle(),
            offer.clone(),
            None,
            true,
        );
        assert_eq!(first.feedback().advance, first_advance);
        let last = part(
            &p,
            offer.view().boundary(),
            p.opportunity_views()
                .find(|o| o.kind() == PreparedBreakKind::ParagraphEnd)
                .unwrap()
                .handle(),
            first.hyphenation_continuation(),
            false,
        );
        assert_eq!(last.feedback().advance, last_advance);
        let middle = last
            .mapping_views()
            .find(|m| m.source_range() == Range::new(7, 9))
            .unwrap();
        assert_eq!(
            (
                middle.processed_range().start().get(),
                middle.processed_range().end().get()
            ),
            (7, 9)
        );
        assert_eq!(middle.presentation_range().len(), 1);
        assert_eq!(glyphs(&last).iter().filter(|g| g.0 == 36).count(), 1);
        let origin = p.origin_views().next().unwrap().handle();
        let layout = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared: p,
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![
                crate::PlacementEntry::Formatted(LinePlacement {
                    line: first,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 8.0,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                }),
                crate::PlacementEntry::Formatted(LinePlacement {
                    line: last,
                    anchor: LineAnchor {
                        inline_origin: 0.0,
                        baseline: 18.0,
                    },
                    orientation: LineOrientation::Horizontal,
                    atomics: vec![],
                }),
            ],
        })
        .unwrap();
        let range = layout
            .source_range_fragments(&origin, Range::new(7, 9))
            .unwrap();
        let unit = range.units().next().unwrap();
        assert_eq!(unit.line_index(), 1);
        assert_eq!(unit.pieces().next().unwrap().rect().size.width, 5.0);
        assert_eq!(layout.source(), &source);
    }
}

#[cfg(feature = "text-accessibility")]
#[test]
fn kana_accessibility_exposes_original_supplementary_unit_at_transformed_geometry() {
    let source = Source::new("A\u{1b132}C");
    let p = captured(source.clone(), kana_style());
    let line = whole(&p);
    let layout = place(p, line, 0.0, 8.0).unwrap();
    let mut state = crate::Accessibility::default();
    let parent = accesskit::NodeId(10000);
    let update = layout
        .accessibility_update(&mut state, parent, parent, crate::Point::default())
        .unwrap();
    let node = &update
        .nodes
        .iter()
        .find(|(_, node)| node.value() == Some("\u{1b132}"))
        .unwrap()
        .1;
    assert_eq!(node.character_lengths(), [4]);
    assert_eq!(node.bounds().unwrap().width(), 5.0);
    assert_eq!(layout.source(), &source);
}

#[test]
fn native_rtl_frame_and_foreign_positions_keep_exact_transformed_source_identity() {
    let source = Source::identified("A\u{1b132}C", Id::from_u64(6305), 1);
    let mut i = input(source.clone(), kana_style());
    i.base_direction = ParagraphBaseDirection::RightToLeft;
    let p = capture_input(i, crate::shaping_test_fonts::kana_ahem());
    let mut r = request(&p);
    r.context.base_direction = ParagraphBaseDirection::RightToLeft;
    r.context.line_box_direction = crate::LineBoxDirection::RightToLeft;
    r.context.alignment = crate::Alignment::Right;
    let line = p
        .select(r)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(line.feedback().advance, 25.0);
    // Actual Latin/kana L characters remain one LTR sequence at level2 in RTL.
    assert_eq!(
        glyphs(&line),
        [
            (35, 10.0, 175.0, Range::new(0, 1)),
            (36, 5.0, 185.0, Range::new(1, 3)),
            (37, 10.0, 190.0, Range::new(3, 4))
        ]
    );
    let layout = place(p, line, 0.0, 8.0).unwrap();
    assert_eq!(
        layout
            .cursor(crate::Cursor::new(2, crate::Affinity::Before))
            .unwrap()
            .rect()
            .origin
            .x,
        185.0
    );
    assert_eq!(
        layout
            .cursor(crate::Cursor::new(2, crate::Affinity::After))
            .unwrap()
            .rect()
            .origin
            .x,
        190.0
    );
    let other = captured(source, kana_style());
    let other_line = whole(&other);
    let other_layout = place(other, other_line, 0.0, 8.0).unwrap();
    assert!(
        layout
            .position_geometry(&other_layout.positions().next().unwrap())
            .is_err()
    );
}
