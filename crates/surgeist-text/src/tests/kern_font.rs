//! Real native Text qualification of the retained Ahem kerning derivative.
//!
//! The font's only pair is A35/period16/-200 at UPEM1000. The pinned
//! Harfrust horizontal kern implementation splits the adjustment equally:
//! at size10, each contribution is9 and period's raw x offset is-1. This is
//! a font/provider fixture contract, not a hanging-punctuation implementation.

use crate::*;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
};

const ORIGINAL: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
const KERN_FONT: &[u8] = include_bytes!("../../tests/fixtures/fonts/kern-ahem/KernAhem.ttf");
const FAMILY: &str = "Surgeist Kern Ahem";

#[test]
fn kern_font_retains_actual_cmap_outlines_and_unscaled_metrics() {
    let original = skrifa::FontRef::new(ORIGINAL).unwrap();
    let font = skrifa::FontRef::new(KERN_FONT).unwrap();
    assert_eq!(
        font.metrics(FontSize::unscaled(), LocationRef::default())
            .units_per_em,
        1000
    );
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for (ch, id) in [('A', 35), ('.', 16)] {
        assert_eq!(original.charmap().map(ch), Some(GlyphId::new(id)));
        assert_eq!(font.charmap().map(ch), Some(GlyphId::new(id)));
        assert_eq!(metrics.advance_width(GlyphId::new(id)), Some(1000.0));
    }
    for tag in [*b"cmap", *b"glyf", *b"loca", *b"hmtx", *b"hhea", *b"maxp"] {
        assert_eq!(
            crate::test_fonts::table(KERN_FONT, tag),
            crate::test_fonts::table(ORIGINAL, tag),
            "the original table is retained: {tag:?}"
        );
    }
    // OpenType v0; one horizontal format0 subtable; one actual sorted pair.
    assert_eq!(
        crate::test_fonts::table(KERN_FONT, *b"kern"),
        [
            0, 0, 0, 1, 0, 0, 0, 20, 0, 1, 0, 1, 0, 6, 0, 0, 0, 0, 0, 35, 0, 16, 0xff, 0x38,
        ]
    );
}

fn prepare(kerning: bool) -> PreparedText {
    let font = Font::new().family(FAMILY);
    let style = Style {
        font: if kerning {
            font
        } else {
            font.feature(r#""kern" off"#)
        },
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        wrap: Wrap::None,
        ..Style::default()
    };
    let source = Source::identified("A.", Id::from_u64(71001), 3);
    let input = ParagraphInput {
        source: source.clone(),
        base_style: style,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![ParagraphOrigin {
            source,
            local: Range::new(0, 2),
            aggregate: Range::new(0, 2),
            kind: ParagraphOriginKind::Authored,
            text_node: None,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    let analyzed = AnalyzedSource::from_paragraph(input).unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::None);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    // Original and derivative coexist. Every later face observation must be
    // the derivative selected by its distinct family, never host fallback.
    system.register_fonts(ORIGINAL.to_vec()).unwrap();
    system.register_fonts(KERN_FONT.to_vec()).unwrap();
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}

fn finalize(prepared: &PreparedText) -> FinalizedLine {
    let end = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap();
    prepared
        .select(LineSelectionRequest {
            boundary_context: vec![],
            incoming: None,
            start: prepared.boundary_views().next().unwrap().handle(),
            end: end.boundary(),
            termination: PreparedTermination::Ordinary(end.handle()),
            alternative: None,
            participants: prepared.participants().collect(),
            atomics: vec![],
            controls: vec![],
            line_strut: None,
            context: LineContext {
                formatted_line: crate::FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: LineBoxDirection::LeftToRight,
                orientation: LineOrientation::Horizontal,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Left,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}

#[test]
fn native_default_kern_and_explicit_off_preserve_actual_glyph_and_tcu_contributions() {
    for (kerning, advance, offset, width) in [(true, 9.0, -1.0, 18.0), (false, 10.0, 0.0, 20.0)] {
        let prepared = prepare(kerning);
        let analyzed = prepared.analyzed_source();
        assert_eq!(analyzed.source().id(), Some(Id::from_u64(71001)));
        assert_eq!(analyzed.source().revision(), 3);
        assert_eq!(analyzed.source().text().code_units(), [0x41, 0x2e]);
        assert_eq!(
            analyzed.processed_source().processed_text().code_units(),
            [0x41, 0x2e]
        );
        assert_eq!(
            analyzed.graphemes().collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 2)]
        );
        assert_eq!(
            analyzed.style().font.features,
            if kerning {
                vec![]
            } else {
                vec![r#""kern" off"#.to_owned()]
            }
        );
        let units = prepared.unit_views().collect::<Vec<_>>();
        assert_eq!(units.len(), 2);
        let origin = prepared.origin_views().next().unwrap().handle();
        for (i, unit) in units.iter().enumerate() {
            assert_eq!(unit.source_range(), Range::new(i, i + 1));
            assert_eq!(unit.advance(), advance);
            let associations = unit.origins().collect::<Vec<_>>();
            assert_eq!(associations.len(), 1);
            assert_eq!(associations[0].origin(), origin);
            assert_eq!(associations[0].local_range(), Range::new(i, i + 1));
            assert_eq!(associations[0].aggregate_range(), Range::new(i, i + 1));
            let instances = unit.font_instances().collect::<Result<Vec<_>>>().unwrap();
            assert_eq!(instances.len(), 1);
            assert_eq!(instances[0].face().bytes(), KERN_FONT);
            assert_eq!(instances[0].font_size(), 10.0);
        }
        let line = finalize(&prepared);
        assert_eq!(line.prepared_text(), prepared);
        let mut actual = Vec::new();
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), KERN_FONT);
            assert_eq!(run.face().index(), 0);
            assert_eq!(
                run.face().catalogue_identity(),
                prepared.font_catalogue_identity()
            );
            let instance = run.effective_instance().unwrap();
            assert!(instance.face().is_same_face(&run.face()));
            assert_eq!(instance.font_size(), 10.0);
            assert_eq!(instance.axis_count(), 0);
            assert_eq!(
                instance
                    .metrics(prepared.font_catalogue_identity())
                    .unwrap()
                    .units_per_em(),
                1000
            );
            assert_eq!(run.advance(), width);
            for glyph in run.glyphs() {
                actual.push((
                    glyph.range(),
                    glyph.id(),
                    glyph.advance(),
                    glyph.x(),
                    glyph.y(),
                    glyph.pen(),
                    run.origin().x + glyph.pen() + f64::from(glyph.x()),
                ));
            }
        }
        assert_eq!(
            actual,
            [
                (Range::new(0, 1), 35, advance as f32, 0.0, 0.0, 0.0, 0.0),
                (
                    Range::new(1, 2),
                    16,
                    advance as f32,
                    offset,
                    0.0,
                    advance,
                    advance + f64::from(offset)
                ),
            ]
        );
        assert_eq!(line.feedback().advance, width);
        assert_eq!(line.measurements().fit().advance(), width);
        assert_eq!(line.measurements().alignment().advance(), width);
        let edges = line.edge_units().collect::<Vec<_>>();
        assert_eq!(edges.len(), 2);
        for (i, edge) in edges.iter().enumerate() {
            assert_eq!(edge.advance(), advance);
            assert_eq!(
                edge.source_pieces().collect::<Vec<_>>(),
                [Range::new(i, i + 1)]
            );
            assert_eq!(edge.presentation_range().start(), i);
            assert_eq!(edge.presentation_range().end(), i + 1);
            assert_eq!(
                edge.owners().collect::<Vec<_>>(),
                [LineEdgeOwner::Participant(PreparedParticipant::Text(
                    units[i].handle()
                ))]
            );
            assert_eq!(edge.disposition(), LineEdgeDisposition::Ordinary);
            assert_eq!(edge.excluded_contribution(), 0.0);
        }
        eprintln!(
            "kern={kerning}: finalized glyphs={actual:?}, raw/fit={width}, TCU contributions=[{advance},{advance}]"
        );
    }
}

#[test]
fn kerned_period_keeps_parent_source_carets_at_the_logical_pen() {
    for (kerning, advance, width) in [(true, 9.0, 18.0), (false, 10.0, 20.0)] {
        let prepared = prepare(kerning);
        let source = prepared.analyzed_source().source().clone();
        let line = finalize(&prepared);
        let layout = Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            lines: vec![LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 8.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            }],
        })
        .unwrap();
        assert_eq!(layout.source(), &source);
        assert_eq!(layout.metrics().full_width(), width);
        for (index, x) in [(0, 0.0), (1, advance), (2, width)] {
            for affinity in [Affinity::Before, Affinity::After] {
                assert_eq!(
                    layout
                        .cursor(Cursor::new(index, affinity))
                        .unwrap()
                        .rect()
                        .origin
                        .x,
                    x
                );
            }
        }
        let endpoint = |index, affinity| {
            layout
                .positions()
                .find(|position| position.source_cursor() == Cursor::new(index, affinity))
                .unwrap()
        };
        let selection = layout
            .position_selection(
                &endpoint(0, Affinity::After),
                &endpoint(2, Affinity::Before),
            )
            .unwrap();
        assert_eq!(selection.source_text(), &Text::from("A."));
        assert_eq!(selection.plaintext(), &Text::from("A."));
        let glyphs = layout
            .glyph_runs()
            .into_iter()
            .flat_map(|run| run.glyphs().to_vec())
            .collect::<Vec<_>>();
        assert_eq!(glyphs.len(), 2);
        assert_eq!(glyphs[1].range(), Range::new(1, 2));
        assert_eq!(glyphs[1].id(), 16);
        assert_eq!(glyphs[1].pen(), advance);
        assert_eq!(glyphs[1].x(), if kerning { -1.0 } else { 0.0 });
    }
}
